// SPDX-License-Identifier: MIT OR Apache-2.0

//! Git-common-directory lock for heavyweight verification.
//!
//! The kernel lock is authoritative. The adjacent JSON document is deliberately
//! diagnostic only and never contains the ownership token or raw command line.

use std::ffi::{CString, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::process;

pub(crate) const EX_TEMPFAIL: u8 = 75;

const LOCK_SUBDIR: &str = "rights-verification";
const LOCK_FILENAME: &str = "heavyweight.lock";
const OWNER_FILENAME: &str = "heavyweight-owner.json";
const TOKEN_ENV: &str = "RIGHTS_VERIFY_LOCK_TOKEN";
const OWNER_PID_ENV: &str = "RIGHTS_VERIFY_LOCK_OWNER_PID";
const OWNER_START_ENV: &str = "RIGHTS_VERIFY_LOCK_OWNER_START";
const COMMON_DIR_ENV: &str = "RIGHTS_VERIFY_LOCK_COMMON_DIR";
const NAME_ENV: &str = "RIGHTS_VERIFY_LOCK_NAME";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileIdentity {
    basename: String,
    exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CommandDetails {
    executable: Option<FileIdentity>,
    argument_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OwnerDocument {
    schema_version: u8,
    name: String,
    owner_pid: u32,
    owner_process_start_ticks: String,
    owner_process_group_id: i32,
    started_at_utc: String,
    token_sha256: String,
    source_sha256: String,
    engine: Option<FileIdentity>,
    command: CommandDetails,
}

#[derive(Debug, Serialize)]
struct SanitizedOwnerDetails<'a> {
    schema_version: u8,
    name: &'a str,
    owner_pid: u32,
    owner_process_start_ticks: &'a str,
    owner_process_group_id: i32,
    started_at_utc: &'a str,
    source_sha256: &'a str,
    engine: &'a Option<FileIdentity>,
    command: &'a CommandDetails,
}

pub(crate) struct VerificationLock {
    handle: Option<File>,
    owner_path: PathBuf,
    token_digest: Option<String>,
    inherited: bool,
}

impl VerificationLock {
    pub(crate) fn acquire(context: &Context, name: &str, wait_seconds: f64) -> Result<Self, Error> {
        Self::acquire_at(context.root(), name, wait_seconds)
    }

    fn acquire_at(root: &Path, name: &str, wait_seconds: f64) -> Result<Self, Error> {
        validate_name(name)?;
        validate_wait(wait_seconds)?;
        let common = git_common_dir(root)?;
        let directory = common.join(LOCK_SUBDIR);
        let lock_path = directory.join(LOCK_FILENAME);
        let owner_path = directory.join(OWNER_FILENAME);

        if inherited_context(&common, &lock_path, &owner_path, Some(name))? {
            return Ok(Self {
                handle: None,
                owner_path,
                token_digest: None,
                inherited: true,
            });
        }

        secure_lock_directory(&common, &directory)?;
        let handle = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&lock_path)?;
        let deadline = Instant::now() + Duration::from_secs_f64(wait_seconds);
        loop {
            match flock(handle.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) {
                Ok(()) => break,
                Err(error) if error.raw_os_error() == Some(libc::EWOULDBLOCK) => {
                    if Instant::now() >= deadline {
                        let details = sanitized_owner_details(&owner_path);
                        return Err(Error::with_exit_code(
                            format!(
                                "verification lock: heavyweight verifier lock is busy: {}",
                                details
                            ),
                            EX_TEMPFAIL,
                        ));
                    }
                    thread::sleep(
                        deadline
                            .saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(100)),
                    );
                }
                Err(error) => {
                    return Err(Error::usage(format!(
                        "verification lock: cannot acquire {}: {error}",
                        lock_path.display()
                    )));
                }
            }
        }

        let mut token = [0_u8; 64];
        if let Err(error) = getrandom::fill(&mut token) {
            let _ = flock(handle.as_raw_fd(), libc::LOCK_UN);
            return Err(Error::usage(format!(
                "verification lock: cannot create ownership token: {error}"
            )));
        }
        let token = hex(&token);
        let token_digest = sha256(token.as_bytes());
        let owner = owner_document(root, name, &token_digest)?;
        if let Err(error) = atomic_json(&owner_path, &owner) {
            let _ = flock(handle.as_raw_fd(), libc::LOCK_UN);
            return Err(error);
        }

        Ok(Self {
            handle: Some(handle),
            owner_path,
            token_digest: Some(token_digest),
            inherited: false,
        })
    }

    pub(crate) fn inherited(&self) -> bool {
        self.inherited
    }
}

fn secure_lock_directory(common: &Path, directory: &Path) -> Result<(), Error> {
    let common_metadata = fs::symlink_metadata(common)?;
    if !common_metadata.file_type().is_dir() || common_metadata.file_type().is_symlink() {
        return Err(lock_usage("Git common directory is not a real directory"));
    }
    match fs::symlink_metadata(directory) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                return Err(lock_usage(
                    "verification lock directory must not be a symbolic link",
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(directory)?;
        }
        Err(error) => return Err(error.into()),
    }
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

impl Drop for VerificationLock {
    fn drop(&mut self) {
        if self.inherited {
            return;
        }
        if let Some(expected) = &self.token_digest
            && read_owner(&self.owner_path)
                .map(|owner| owner.token_sha256)
                .as_deref()
                == Some(expected)
        {
            let _ = fs::remove_file(&self.owner_path);
        }
        if let Some(handle) = &self.handle {
            let _ = flock(handle.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

pub(crate) fn validate_inherited(context: &Context, name: &str) -> Result<(), Error> {
    validate_name(name)?;
    let common = git_common_dir(context.root())?;
    let directory = common.join(LOCK_SUBDIR);
    let lock_path = directory.join(LOCK_FILENAME);
    let owner_path = directory.join(OWNER_FILENAME);
    if inherited_context(&common, &lock_path, &owner_path, Some(name))? {
        Ok(())
    } else {
        Err(lock_usage("no inherited verification lock is active"))
    }
}

fn validate_name(name: &str) -> Result<(), Error> {
    let valid = !name.is_empty()
        && name.len() <= 96
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(lock_usage(
            "lock name must be 1-96 safe identifier characters",
        ))
    }
}

fn validate_wait(wait_seconds: f64) -> Result<(), Error> {
    if wait_seconds.is_finite() && wait_seconds >= 0.0 {
        Ok(())
    } else {
        Err(lock_usage("wait timeout must be finite and nonnegative"))
    }
}

fn git_common_dir(root: &Path) -> Result<PathBuf, Error> {
    let output = process::checked_stdout(
        root,
        "git",
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    fs::canonicalize(output.trim()).map_err(|error| {
        lock_usage(format!(
            "cannot resolve Git common directory {}: {error}",
            output.trim()
        ))
    })
}

fn repository_source_digest(root: &Path) -> Result<String, Error> {
    let head = process::checked_stdout(root, "git", ["rev-parse", "HEAD"])?;
    let index = process::output(root, "git", ["ls-files", "-s", "-z"])?;
    if !index.status.success() {
        return Err(lock_usage("cannot bind the repository index identity"));
    }
    let mut binding = head.trim().as_bytes().to_vec();
    binding.push(0);
    binding.extend(index.stdout);
    Ok(sha256(binding))
}

fn owner_document(root: &Path, name: &str, token_digest: &str) -> Result<OwnerDocument, Error> {
    let pid = std::process::id();
    let executable = mapped_executable_identity()?;
    Ok(OwnerDocument {
        schema_version: 2,
        name: name.to_owned(),
        owner_pid: pid,
        owner_process_start_ticks: process_start_ticks(pid)?,
        owner_process_group_id: unsafe { libc::getpgrp() },
        started_at_utc: utc_now()?,
        token_sha256: token_digest.to_owned(),
        source_sha256: repository_source_digest(root)?,
        engine: executable.clone(),
        command: CommandDetails {
            executable,
            argument_count: std::env::args_os().len().saturating_sub(1),
        },
    })
}

fn mapped_executable_identity() -> Result<Option<FileIdentity>, Error> {
    static IDENTITY: OnceLock<Result<FileIdentity, String>> = OnceLock::new();
    IDENTITY
        .get_or_init(|| {
            let display = std::env::current_exe()
                .map_err(|error| format!("cannot locate rights-verify: {error}"))?;
            let body = fs::read("/proc/self/exe")
                .map_err(|error| format!("cannot read mapped rights-verify: {error}"))?;
            Ok(FileIdentity {
                basename: display
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                exists: true,
                sha256: Some(sha256(&body)),
                size: Some(body.len()),
            })
        })
        .clone()
        .map(Some)
        .map_err(lock_usage)
}

fn atomic_json(path: &Path, value: &OwnerDocument) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| lock_usage("lock owner path has no parent"))?;
    fs::create_dir_all(parent)?;
    let mut nonce = [0_u8; 16];
    getrandom::fill(&mut nonce)
        .map_err(|error| lock_usage(format!("cannot create owner filename: {error}")))?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| lock_usage("lock owner filename is not UTF-8"))?;
    let temporary = parent.join(format!(
        ".{filename}.{}.{}.tmp",
        std::process::id(),
        hex(&nonce)
    ));
    let result = (|| {
        let mut handle = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        let mut body = serde_json::to_vec_pretty(value)?;
        body.push(b'\n');
        handle.write_all(&body)?;
        handle.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok::<_, Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn read_owner(path: &Path) -> Option<OwnerDocument> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub(crate) fn sanitized_owner_details(path: &Path) -> String {
    let Some(owner) = read_owner(path) else {
        return "{}".to_owned();
    };
    canonical_json(&SanitizedOwnerDetails {
        schema_version: owner.schema_version,
        name: &owner.name,
        owner_pid: owner.owner_pid,
        owner_process_start_ticks: &owner.owner_process_start_ticks,
        owner_process_group_id: owner.owner_process_group_id,
        started_at_utc: &owner.started_at_utc,
        source_sha256: &owner.source_sha256,
        engine: &owner.engine,
        command: &owner.command,
    })
}

fn inherited_context(
    common: &Path,
    lock_path: &Path,
    owner_path: &Path,
    required_name: Option<&str>,
) -> Result<bool, Error> {
    let Some(claim) = inherited_claim_from(|name| std::env::var_os(name))? else {
        return Ok(false);
    };
    inherited_context_for_claim(common, lock_path, owner_path, required_name, &claim)
}

#[derive(Clone, Debug)]
struct InheritedClaim {
    token_sha256: String,
    owner_pid_raw: String,
    owner_start: String,
    common_dir: String,
    name: String,
}

fn inherited_claim_from(
    mut value: impl FnMut(&str) -> Option<OsString>,
) -> Result<Option<InheritedClaim>, Error> {
    let Some(token) = value(TOKEN_ENV) else {
        return Ok(None);
    };
    let token = token
        .into_string()
        .map_err(|_| lock_usage("inherited lock token is not valid UTF-8"))?;
    let field = |value: Option<OsString>| -> Result<String, Error> {
        value
            .ok_or_else(|| lock_usage("inherited lock ownership is incomplete"))?
            .into_string()
            .map_err(|_| lock_usage("inherited lock ownership is incomplete"))
    };
    let owner_pid_raw = field(value(OWNER_PID_ENV))?;
    let owner_start = field(value(OWNER_START_ENV))?;
    let common_dir = field(value(COMMON_DIR_ENV))?;
    let name = field(value(NAME_ENV))?;
    if token.is_empty()
        || owner_start.is_empty()
        || name.is_empty()
        || !owner_pid_raw.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(lock_usage("inherited lock ownership is incomplete"));
    }
    Ok(Some(InheritedClaim {
        token_sha256: sha256(token.as_bytes()),
        owner_pid_raw,
        owner_start,
        common_dir,
        name,
    }))
}

fn inherited_context_for_claim(
    common: &Path,
    lock_path: &Path,
    owner_path: &Path,
    required_name: Option<&str>,
    claim: &InheritedClaim,
) -> Result<bool, Error> {
    let inherited_path = fs::canonicalize(&claim.common_dir)
        .map_err(|_| lock_usage("inherited lock common directory is invalid"))?;
    if inherited_path != common {
        return Err(lock_usage(
            "inherited lock belongs to a different Git common dir",
        ));
    }
    let owner_pid: u32 = claim
        .owner_pid_raw
        .parse()
        .map_err(|_| lock_usage("inherited lock ownership is incomplete"))?;
    if owner_pid <= 1 || !is_ancestor(owner_pid, std::process::id()) {
        return Err(lock_usage("inherited lock owner is not a live ancestor"));
    }
    if process_start_ticks(owner_pid)? != claim.owner_start {
        return Err(lock_usage("inherited lock owner identity is stale"));
    }
    let owner = read_owner(owner_path)
        .ok_or_else(|| lock_usage("inherited lock owner metadata is missing"))?;
    if owner.name != claim.name
        || owner.owner_pid != owner_pid
        || owner.owner_process_start_ticks != claim.owner_start
        || owner.token_sha256 != claim.token_sha256
    {
        return Err(lock_usage(
            "inherited lock ownership does not match metadata",
        ));
    }
    if let Some(name) = required_name
        && owner.name != name
    {
        return Err(lock_usage(
            "inherited lock owner name does not match the internal verifier",
        ));
    }

    let probe = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .mode(0o600)
        .open(lock_path)?;
    match flock(probe.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) {
        Err(error) if error.raw_os_error() == Some(libc::EWOULDBLOCK) => Ok(true),
        Ok(()) => {
            let _ = flock(probe.as_raw_fd(), libc::LOCK_UN);
            Err(lock_usage(
                "inherited owner metadata exists without a held kernel lock",
            ))
        }
        Err(error) => Err(lock_usage(format!(
            "cannot probe inherited kernel lock: {error}"
        ))),
    }
}

fn process_stat_tail(pid: u32) -> Result<Vec<String>, Error> {
    let raw = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|_| lock_usage(format!("cannot inspect lock process {pid}")))?;
    let close = raw
        .rfind(')')
        .ok_or_else(|| lock_usage(format!("malformed process identity for {pid}")))?;
    let tail = raw
        .get(close + 2..)
        .ok_or_else(|| lock_usage(format!("malformed process identity for {pid}")))?;
    let fields = tail
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if fields.len() < 20 {
        return Err(lock_usage(format!("malformed process identity for {pid}")));
    }
    Ok(fields)
}

fn process_start_ticks(pid: u32) -> Result<String, Error> {
    Ok(process_stat_tail(pid)?[19].clone())
}

fn parent_pid(pid: u32) -> u32 {
    process_stat_tail(pid)
        .ok()
        .and_then(|fields| fields[1].parse().ok())
        .unwrap_or(0)
}

fn is_ancestor(ancestor: u32, descendant: u32) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut current = descendant;
    while current > 1 && seen.insert(current) {
        if current == ancestor {
            return true;
        }
        current = parent_pid(current);
    }
    current == ancestor
}

fn utc_now() -> Result<String, Error> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| lock_usage(format!("system clock predates Unix epoch: {error}")))?
        .as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let within = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        within / 3_600,
        (within % 3_600) / 60,
        within % 60
    ))
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}

fn flock(fd: std::os::fd::RawFd, operation: libc::c_int) -> std::io::Result<()> {
    if unsafe { libc::flock(fd, operation) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

fn canonical_json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_owned())
}

fn lock_usage(message: impl Into<String>) -> Error {
    Error::usage(format!("verification lock: {}", message.into()))
}

const RECEIPT_LOCK_SELF_TEST_SCENARIOS: usize = 6;

fn lock_self_test_error(number: usize, name: &str, message: impl Into<String>) -> Error {
    Error::new(format!(
        "verification receipt self-test scenario {number:02} ({name}): {}",
        message.into()
    ))
}

fn lock_self_test_git(
    root: &Path,
    arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) -> Result<(), Error> {
    let output = process::output(root, "git", arguments)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::new(format!(
            "disposable Git setup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn initialise_lock_self_test_repository(root: &Path) -> Result<(), Error> {
    fs::create_dir_all(root)?;
    lock_self_test_git(root, ["init", "--quiet"])?;
    lock_self_test_git(root, ["config", "user.name", "Receipt Tests"])?;
    lock_self_test_git(
        root,
        ["config", "user.email", "receipt-tests@example.invalid"],
    )?;
    fs::write(root.join("tracked.txt"), "base\n")?;
    lock_self_test_git(root, ["add", "tracked.txt"])?;
    lock_self_test_git(root, ["commit", "--quiet", "-m", "base"])
}

fn signal_releases_kernel_lock(path: &Path, signal: libc::c_int) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::new("signal lock path has no parent"))?;
    fs::create_dir_all(parent)?;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| Error::new("signal lock path contains NUL"))?;
    let mut ready = [0_i32; 2];
    if unsafe { libc::pipe2(ready.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let child = unsafe { libc::fork() };
    if child == -1 {
        unsafe {
            libc::close(ready[0]);
            libc::close(ready[1]);
        }
        return Err(std::io::Error::last_os_error().into());
    }
    if child == 0 {
        unsafe {
            libc::close(ready[0]);
            let descriptor = libc::open(
                path.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_CLOEXEC,
                0o600,
            );
            if descriptor < 0 || libc::flock(descriptor, libc::LOCK_EX) != 0 {
                libc::_exit(101);
            }
            let marker = [b'R'];
            if libc::write(ready[1], marker.as_ptr().cast(), marker.len()) != 1 {
                libc::_exit(102);
            }
            libc::close(ready[1]);
            loop {
                libc::pause();
            }
        }
    }

    unsafe {
        libc::close(ready[1]);
    }
    let mut poll = libc::pollfd {
        fd: ready[0],
        events: libc::POLLIN,
        revents: 0,
    };
    let poll_result = unsafe { libc::poll(&mut poll, 1, 2_000) };
    let mut marker = [0_u8; 1];
    let read_result = if poll_result > 0 {
        unsafe { libc::read(ready[0], marker.as_mut_ptr().cast(), marker.len()) }
    } else {
        -1
    };
    unsafe {
        libc::close(ready[0]);
    }
    if read_result != 1 || marker != [b'R'] {
        unsafe {
            libc::kill(child, libc::SIGKILL);
            libc::waitpid(child, std::ptr::null_mut(), 0);
        }
        return Err(Error::new(
            "signal-holder child did not acquire the kernel lock",
        ));
    }
    if unsafe { libc::kill(child, signal) } != 0 {
        unsafe {
            libc::kill(child, libc::SIGKILL);
            libc::waitpid(child, std::ptr::null_mut(), 0);
        }
        return Err(std::io::Error::last_os_error().into());
    }
    let mut status = 0;
    if unsafe { libc::waitpid(child, &mut status, 0) } != child {
        return Err(std::io::Error::last_os_error().into());
    }
    let handle = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .mode(0o600)
        .open(Path::new(std::ffi::OsStr::from_bytes(path.as_bytes())))?;
    flock(handle.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB)?;
    flock(handle.as_raw_fd(), libc::LOCK_UN)?;
    Ok(())
}

/// Run the six lock cases formerly owned by the Python receipt self-test.
///
/// The native verifier does not launch a heavyweight child, so its signal
/// controls exercise release of the owning kernel descriptor directly. This
/// is the isolation-equivalent contract for the single-process binary.
pub(crate) fn receipt_protocol_self_test() -> Result<usize, Error> {
    let temporary = tempfile::Builder::new()
        .prefix("rights-lock-self-test-")
        .tempdir()?;
    let repository = temporary.path().join("repo");
    initialise_lock_self_test_repository(&repository).map_err(|error| {
        lock_self_test_error(
            1,
            "normal nested contention and metadata",
            error.to_string(),
        )
    })?;
    let common = git_common_dir(&repository).map_err(|error| {
        lock_self_test_error(
            1,
            "normal nested contention and metadata",
            error.to_string(),
        )
    })?;
    let directory = common.join(LOCK_SUBDIR);
    let lock_path = directory.join(LOCK_FILENAME);
    let owner_path = directory.join(OWNER_FILENAME);

    let outer = VerificationLock::acquire_at(&repository, "verify", 0.0).map_err(|error| {
        lock_self_test_error(
            1,
            "normal nested contention and metadata",
            error.to_string(),
        )
    })?;
    if outer.inherited() {
        return Err(lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "initial lock was incorrectly inherited",
        ));
    }
    let owner = read_owner(&owner_path).ok_or_else(|| {
        lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "owner metadata is absent or malformed",
        )
    })?;
    let sanitized = sanitized_owner_details(&owner_path);
    if sanitized.contains("token_sha256")
        || sanitized.contains(&owner.token_sha256)
        || sanitized.contains("argv")
    {
        return Err(lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "sanitized owner details exposed token or argv material",
        ));
    }
    let claim = InheritedClaim {
        token_sha256: outer.token_digest.clone().ok_or_else(|| {
            lock_self_test_error(
                1,
                "normal nested contention and metadata",
                "owning lock has no token digest",
            )
        })?,
        owner_pid_raw: owner.owner_pid.to_string(),
        owner_start: owner.owner_process_start_ticks.clone(),
        common_dir: common.to_string_lossy().into_owned(),
        name: owner.name.clone(),
    };
    if !inherited_context_for_claim(&common, &lock_path, &owner_path, Some("verify"), &claim)
        .map_err(|error| {
            lock_self_test_error(
                1,
                "normal nested contention and metadata",
                error.to_string(),
            )
        })?
    {
        return Err(lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "valid nested ownership was not inherited",
        ));
    }
    if inherited_context_for_claim(
        &common,
        &lock_path,
        &owner_path,
        Some("forged-name"),
        &claim,
    )
    .is_ok()
    {
        return Err(lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "wrong inherited owner name was accepted",
        ));
    }
    let started = Instant::now();
    let contention = VerificationLock::acquire_at(&repository, "contender", 0.02)
        .err()
        .ok_or_else(|| {
            lock_self_test_error(
                1,
                "normal nested contention and metadata",
                "contender acquired an already-held lock",
            )
        })?;
    if contention.exit_code() != EX_TEMPFAIL || started.elapsed() < Duration::from_millis(15) {
        return Err(lock_self_test_error(
            1,
            "normal nested contention and metadata",
            "bounded contention did not wait and fail with EX_TEMPFAIL",
        ));
    }

    let mut forged = claim.clone();
    forged.token_sha256 = "f".repeat(64);
    if inherited_context_for_claim(&common, &lock_path, &owner_path, Some("verify"), &forged)
        .is_ok()
    {
        return Err(lock_self_test_error(
            2,
            "forged inheritance fails",
            "forged ownership token was accepted",
        ));
    }

    let linked = temporary.path().join("linked");
    let linked_arguments = [
        std::ffi::OsStr::new("worktree"),
        std::ffi::OsStr::new("add"),
        std::ffi::OsStr::new("--quiet"),
        std::ffi::OsStr::new("-b"),
        std::ffi::OsStr::new("linked-self-test"),
        linked.as_os_str(),
    ];
    lock_self_test_git(&repository, linked_arguments).map_err(|error| {
        lock_self_test_error(3, "linked worktree common lock", error.to_string())
    })?;
    if git_common_dir(&linked).map_err(|error| {
        lock_self_test_error(3, "linked worktree common lock", error.to_string())
    })? != common
        || VerificationLock::acquire_at(&linked, "linked-contender", 0.0)
            .err()
            .map(|error| error.exit_code())
            != Some(EX_TEMPFAIL)
    {
        return Err(lock_self_test_error(
            3,
            "linked worktree common lock",
            "linked worktree did not share lock contention",
        ));
    }
    drop(outer);
    let after =
        VerificationLock::acquire_at(&repository, "after-normal-exit", 0.0).map_err(|error| {
            lock_self_test_error(
                1,
                "normal nested contention and metadata",
                error.to_string(),
            )
        })?;
    drop(after);

    signal_releases_kernel_lock(&directory.join("signal-term.lock"), libc::SIGTERM).map_err(
        |error| lock_self_test_error(4, "SIGTERM releases kernel lock", error.to_string()),
    )?;
    signal_releases_kernel_lock(&directory.join("signal-kill.lock"), libc::SIGKILL).map_err(
        |error| lock_self_test_error(5, "SIGKILL releases kernel lock", error.to_string()),
    )?;

    let incomplete = inherited_claim_from(|name| {
        (name == TOKEN_ENV).then(|| OsString::from("forged-incomplete-owner"))
    });
    if incomplete.is_ok() {
        return Err(lock_self_test_error(
            6,
            "incomplete inherited owner rejected",
            "incomplete inherited ownership was accepted",
        ));
    }
    Ok(RECEIPT_LOCK_SELF_TEST_SCENARIOS)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process::Command;

    use tempfile::tempdir;

    use super::{
        EX_TEMPFAIL, VerificationLock, civil_from_days, hex, is_ancestor, validate_name,
        validate_wait,
    };
    use crate::context::Context;

    fn init_repository(path: &std::path::Path) {
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(path)
            .status()
            .expect("run git init");
        assert!(status.success());
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=Verification Lock Test",
                "-c",
                "user.email=lock-test@example.invalid",
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                "fixture",
            ])
            .current_dir(path)
            .status()
            .expect("create fixture commit");
        assert!(status.success());
    }

    #[test]
    fn validates_names_and_wait_bounds() {
        assert!(validate_name("verify:full-1").is_ok());
        assert!(validate_name("bad name").is_err());
        assert!(validate_wait(0.0).is_ok());
        assert!(validate_wait(86_400.0).is_ok());
        assert!(validate_wait(86_401.0).is_ok());
        assert!(validate_wait(f64::NAN).is_err());
        assert!(validate_wait(f64::INFINITY).is_err());
        assert!(validate_wait(-1.0).is_err());
    }

    #[test]
    fn process_identity_sees_self_as_ancestor() {
        assert!(is_ancestor(std::process::id(), std::process::id()));
    }

    #[test]
    fn utc_calendar_and_hex_are_stable() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_000), (2024, 10, 4));
        assert_eq!(hex(&[0, 15, 16, 255]), "000f10ff");
    }

    #[test]
    fn production_receipt_lock_scenarios_pass() {
        assert_eq!(super::receipt_protocol_self_test().unwrap(), 6);
    }

    #[test]
    fn kernel_contention_fails_with_tempfail_and_releases_cleanly() {
        let temporary = tempdir().expect("temporary repository");
        init_repository(temporary.path());
        let context = Context::from_test_root(temporary.path().to_path_buf());
        let first = VerificationLock::acquire(&context, "verify", 0.0).expect("first lock");
        let error = VerificationLock::acquire(&context, "verify", 0.0)
            .err()
            .expect("second lock must contend");
        assert_eq!(error.exit_code(), EX_TEMPFAIL);
        drop(first);
        let reacquired = VerificationLock::acquire(&context, "verify", 0.0).expect("reacquire");
        assert!(!reacquired.inherited());
    }

    #[test]
    fn symbolic_lock_directory_is_rejected() {
        let temporary = tempdir().expect("temporary repository");
        init_repository(temporary.path());
        let target = temporary.path().join("outside");
        fs::create_dir(&target).expect("outside directory");
        symlink(&target, temporary.path().join(".git/rights-verification")).expect("lock symlink");
        let context = Context::from_test_root(temporary.path().to_path_buf());
        let error = VerificationLock::acquire(&context, "verify", 0.0)
            .err()
            .expect("symlink must fail");
        assert!(error.to_string().contains("must not be a symbolic link"));
    }
}
