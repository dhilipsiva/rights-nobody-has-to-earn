// SPDX-License-Identifier: MIT OR Apache-2.0

//! Atomic generated-artifact refresh with immutable-input drift detection.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, Metadata};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::{Builder, NamedTempFile};

use crate::cli::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatSignature {
    device: u64,
    inode: u64,
    mode: u32,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl StatSignature {
    fn read(path: &Path) -> Result<Self, RefreshError> {
        let metadata = fs::metadata(path).map_err(|error| {
            RefreshError::new(format!(
                "cannot stat immutable input {}: {error}",
                path.display()
            ))
        })?;
        Ok(Self::from_metadata(&metadata))
    }

    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode() & 0o7777,
            size: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

#[derive(Clone, Debug)]
struct CachedInput {
    bytes: Vec<u8>,
    metadata: StatSignature,
    initial_reads: usize,
    rehashes: usize,
}

#[derive(Debug)]
pub(crate) struct ImmutableRepositoryInputs {
    root: PathBuf,
    head: String,
    inputs: BTreeMap<PathBuf, CachedInput>,
}

impl ImmutableRepositoryInputs {
    pub(crate) fn new(root: &Path) -> Result<Self, Error> {
        let root = fs::canonicalize(root).map_err(|error| {
            Error::new(format!(
                "verification refresh: cannot resolve repository root {}: {error}",
                root.display()
            ))
        })?;
        let head = git_head(&root).map_err(public_error)?;
        Ok(Self {
            root,
            head,
            inputs: BTreeMap::new(),
        })
    }

    fn resolve(&self, value: &Path) -> Result<PathBuf, RefreshError> {
        let candidate = if value.is_absolute() {
            value.to_path_buf()
        } else {
            self.root.join(value)
        };
        let resolved = if candidate.exists() {
            fs::canonicalize(&candidate).map_err(|error| {
                RefreshError::new(format!("cannot resolve {}: {error}", candidate.display()))
            })?
        } else {
            let parent = candidate.parent().ok_or_else(|| {
                RefreshError::new(format!(
                    "output path has no parent: {}",
                    candidate.display()
                ))
            })?;
            let parent = fs::canonicalize(parent).map_err(|error| {
                RefreshError::new(format!("cannot resolve {}: {error}", parent.display()))
            })?;
            let name = candidate.file_name().ok_or_else(|| {
                RefreshError::new(format!(
                    "output path has no file name: {}",
                    candidate.display()
                ))
            })?;
            parent.join(name)
        };
        if !resolved.starts_with(&self.root) {
            return Err(RefreshError::new(format!(
                "refresh path escapes the repository root: {}",
                resolved.display()
            )));
        }
        Ok(resolved)
    }

    pub(crate) fn read_bytes(&mut self, value: &Path) -> Result<&[u8], Error> {
        self.read_bytes_inner(value).map_err(public_error)
    }

    fn read_bytes_inner(&mut self, value: &Path) -> Result<&[u8], RefreshError> {
        let path = self.resolve(value)?;
        if !self.inputs.contains_key(&path) {
            let before = StatSignature::read(&path)?;
            let bytes = fs::read(&path).map_err(|error| {
                RefreshError::new(format!(
                    "cannot read immutable input {}: {error}",
                    path.display()
                ))
            })?;
            let after = StatSignature::read(&path)?;
            if before != after {
                return Err(RefreshError::new(format!(
                    "immutable input drifted during initial read: {}",
                    path.display()
                )));
            }
            self.inputs.insert(
                path.clone(),
                CachedInput {
                    bytes,
                    metadata: after,
                    initial_reads: 1,
                    rehashes: 0,
                },
            );
        }
        Ok(&self.inputs.get(&path).expect("cached input inserted").bytes)
    }

    pub(crate) fn read_text(&mut self, value: &Path) -> Result<String, Error> {
        let bytes = self.read_bytes_inner(value).map_err(public_error)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            Error::new(format!(
                "verification refresh: immutable input is not UTF-8: {}",
                value.display()
            ))
        })
    }

    pub(crate) fn adopt_bytes(&mut self, value: &Path, bytes: &[u8]) -> Result<(), Error> {
        self.adopt_bytes_inner(value, bytes).map_err(public_error)
    }

    fn adopt_bytes_inner(&mut self, value: &Path, bytes: &[u8]) -> Result<(), RefreshError> {
        let path = self.resolve(value)?;
        if let Some(cached) = self.inputs.get(&path) {
            if cached.bytes != bytes {
                return Err(RefreshError::new(format!(
                    "conflicting immutable input bytes adopted for {}",
                    path.display()
                )));
            }
            return Ok(());
        }
        self.inputs.insert(
            path.clone(),
            CachedInput {
                bytes: bytes.to_vec(),
                metadata: StatSignature::read(&path)?,
                initial_reads: 1,
                rehashes: 0,
            },
        );
        Ok(())
    }

    fn mode_inner(&mut self, value: &Path) -> Result<u32, RefreshError> {
        let path = self.resolve(value)?;
        if !self.inputs.contains_key(&path) {
            self.read_bytes_inner(&path)?;
        }
        Ok(self
            .inputs
            .get(&path)
            .expect("cached input inserted")
            .metadata
            .mode)
    }

    fn advance_replacement_inner(
        &mut self,
        value: &Path,
        bytes: &[u8],
    ) -> Result<(), RefreshError> {
        let path = self.resolve(value)?;
        self.inputs.insert(
            path.clone(),
            CachedInput {
                bytes: bytes.to_vec(),
                metadata: StatSignature::read(&path)?,
                initial_reads: self
                    .inputs
                    .get(&path)
                    .map_or(1, |cached| cached.initial_reads),
                rehashes: 0,
            },
        );
        Ok(())
    }

    fn assert_metadata_unchanged_inner(&self) -> Result<(), RefreshError> {
        for (path, expected) in &self.inputs {
            let current = StatSignature::read(path).map_err(|_| {
                RefreshError::new(format!(
                    "immutable input disappeared before refresh: {}",
                    path.display()
                ))
            })?;
            if current != expected.metadata {
                return Err(RefreshError::new(format!(
                    "immutable input metadata drifted before refresh: {}",
                    path.display()
                )));
            }
        }
        if git_head(&self.root)? != self.head {
            return Err(RefreshError::new("Git HEAD changed before refresh"));
        }
        Ok(())
    }

    fn assert_unchanged_inner(&mut self) -> Result<(), RefreshError> {
        for (path, expected) in &mut self.inputs {
            let before = StatSignature::read(path).map_err(|_| {
                RefreshError::new(format!(
                    "immutable input disappeared during validation: {}",
                    path.display()
                ))
            })?;
            let current = fs::read(path).map_err(|error| {
                RefreshError::new(format!(
                    "cannot re-read immutable input {}: {error}",
                    path.display()
                ))
            })?;
            let after = StatSignature::read(path)?;
            expected.rehashes += 1;
            if before != after {
                return Err(RefreshError::new(format!(
                    "immutable input drifted during final rehash: {}",
                    path.display()
                )));
            }
            if current != expected.bytes {
                return Err(RefreshError::new(format!(
                    "immutable input drifted during validation: {}",
                    path.display()
                )));
            }
            if after != expected.metadata {
                return Err(RefreshError::new(format!(
                    "immutable input metadata drifted during validation: {}",
                    path.display()
                )));
            }
        }
        if git_head(&self.root)? != self.head {
            return Err(RefreshError::new("Git HEAD changed during validation"));
        }
        if self.inputs.values().any(|input| input.initial_reads != 1) {
            return Err(RefreshError::new(
                "immutable input cache performed a duplicate disk read",
            ));
        }
        if self.inputs.values().any(|input| input.rehashes != 1) {
            return Err(RefreshError::new(
                "an immutable input was not rehashed exactly once",
            ));
        }
        Ok(())
    }

    pub(crate) fn assert_unchanged(&mut self) -> Result<(), Error> {
        self.assert_unchanged_inner().map_err(public_error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RefreshEvent {
    BeforeReplace(usize),
    AfterReplace(usize),
}

struct PreparedOutput {
    path: PathBuf,
    payload: Vec<u8>,
    mode: u32,
    existed: bool,
    original: Option<Vec<u8>>,
    temporary: Option<NamedTempFile>,
    backup: Option<NamedTempFile>,
    replaced: bool,
}

pub(crate) fn atomic_refresh_and_check(
    outputs: &[(PathBuf, Vec<u8>)],
    snapshot: &mut ImmutableRepositoryInputs,
) -> Result<(), Error> {
    atomic_refresh_with_hook(outputs, snapshot, |_, _| Ok(())).map_err(public_error)
}

fn atomic_refresh_with_hook<F>(
    outputs: &[(PathBuf, Vec<u8>)],
    snapshot: &mut ImmutableRepositoryInputs,
    mut hook: F,
) -> Result<(), RefreshError>
where
    F: FnMut(RefreshEvent, &Path) -> Result<(), RefreshError>,
{
    let mut prepared = Vec::<PreparedOutput>::new();
    let mut seen = BTreeSet::new();
    let operation = (|| {
        for (raw_path, payload) in outputs {
            let path = snapshot.resolve(raw_path)?;
            if !seen.insert(path.clone()) {
                return Err(RefreshError::new(format!(
                    "duplicate atomic output path: {}",
                    path.display()
                )));
            }
            let existed = path.exists();
            let original = if existed {
                Some(snapshot.read_bytes_inner(&path)?.to_vec())
            } else {
                None
            };
            let mode = if existed {
                snapshot.mode_inner(&path)?
            } else {
                0o644
            };
            let temporary = write_temp(&path, "refresh", payload, mode)?;
            let backup = original
                .as_ref()
                .map(|bytes| write_temp(&path, "backup", bytes, mode))
                .transpose()?;
            prepared.push(PreparedOutput {
                path,
                payload: payload.clone(),
                mode,
                existed,
                original,
                temporary: Some(temporary),
                backup,
                replaced: false,
            });
        }

        snapshot.assert_metadata_unchanged_inner()?;
        for entry in &prepared {
            if !entry.existed && entry.path.exists() {
                return Err(RefreshError::new(format!(
                    "new output appeared before atomic refresh: {}",
                    entry.path.display()
                )));
            }
        }
        for (index, entry) in prepared.iter_mut().enumerate() {
            hook(RefreshEvent::BeforeReplace(index), &entry.path)?;
            entry.replaced = true;
            let temporary = entry
                .temporary
                .as_ref()
                .expect("prepared refresh temporary exists");
            fs::rename(temporary.path(), &entry.path).map_err(|error| {
                RefreshError::new(format!(
                    "cannot install refreshed output {}: {error}",
                    entry.path.display()
                ))
            })?;
            entry.temporary = None;
            hook(RefreshEvent::AfterReplace(index), &entry.path)?;
            if fs::read(&entry.path).map_err(io_refresh_error)? != entry.payload {
                return Err(RefreshError::new(format!(
                    "refreshed output failed its byte check: {}",
                    entry.path.display()
                )));
            }
            if StatSignature::read(&entry.path)?.mode != entry.mode {
                return Err(RefreshError::new(format!(
                    "refreshed output mode drifted: {}",
                    entry.path.display()
                )));
            }
            snapshot.advance_replacement_inner(&entry.path, &entry.payload)?;
        }
        snapshot.assert_unchanged_inner()
    })();

    if let Err(error) = operation {
        if let Err(rollback_error) = rollback(&mut prepared) {
            return Err(RefreshError::new(format!(
                "atomic refresh failed and rollback failed: {rollback_error}"
            )));
        }
        return Err(error);
    }
    Ok(())
}

fn write_temp(
    output: &Path,
    purpose: &str,
    payload: &[u8],
    mode: u32,
) -> Result<NamedTempFile, RefreshError> {
    let parent = output.parent().expect("resolved output has a parent");
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    let mut temporary = Builder::new()
        .prefix(&format!(".{name}.{purpose}-"))
        .tempfile_in(parent)
        .map_err(io_refresh_error)?;
    temporary.write_all(payload).map_err(io_refresh_error)?;
    temporary
        .as_file_mut()
        .sync_all()
        .map_err(io_refresh_error)?;
    fs::set_permissions(temporary.path(), fs::Permissions::from_mode(mode))
        .map_err(io_refresh_error)?;
    if fs::read(temporary.path()).map_err(io_refresh_error)? != payload {
        return Err(RefreshError::new(format!(
            "temporary {purpose} byte check failed: {}",
            output.display()
        )));
    }
    Ok(temporary)
}

fn rollback(prepared: &mut [PreparedOutput]) -> Result<(), RefreshError> {
    for entry in prepared.iter_mut().rev() {
        if !entry.replaced {
            continue;
        }
        if entry.existed {
            let backup = entry.backup.as_ref().ok_or_else(|| {
                RefreshError::new(format!(
                    "missing refresh backup for {}",
                    entry.path.display()
                ))
            })?;
            fs::rename(backup.path(), &entry.path).map_err(io_refresh_error)?;
            entry.backup = None;
        } else {
            match fs::remove_file(&entry.path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(io_refresh_error(error)),
            }
        }
    }
    for entry in prepared {
        if entry.existed {
            let bytes = fs::read(&entry.path).map_err(io_refresh_error)?;
            let mode = StatSignature::read(&entry.path)?.mode;
            if entry.original.as_deref() != Some(bytes.as_slice()) || mode != entry.mode {
                return Err(RefreshError::new(format!(
                    "atomic refresh rollback verification failed: {}",
                    entry.path.display()
                )));
            }
        } else if entry.path.exists() {
            return Err(RefreshError::new(format!(
                "atomic refresh rollback left a new output: {}",
                entry.path.display()
            )));
        }
    }
    Ok(())
}

fn git_head(root: &Path) -> Result<String, RefreshError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(io_refresh_error)?;
    let head = String::from_utf8(output.stdout)
        .map_err(|_| RefreshError::new("Git HEAD output is not UTF-8"))?;
    let head = head.trim();
    if !output.status.success()
        || head.len() != 40
        || !head.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(RefreshError::new("cannot snapshot the current Git HEAD"));
    }
    Ok(head.to_owned())
}

fn io_refresh_error(error: std::io::Error) -> RefreshError {
    RefreshError::new(error.to_string())
}

#[derive(Clone, Debug)]
struct RefreshError(String);

impl RefreshError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for RefreshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn public_error(error: RefreshError) -> Error {
    Error::new(format!("verification refresh: {error}"))
}

pub(crate) fn self_test() -> Result<String, Error> {
    successful_refresh_control()?;
    preinstall_drift_control()?;
    replacement_failure_control()?;
    postinstall_drift_control()?;
    Ok("native atomic refresh self-test passes (4 controls)".to_owned())
}

struct Fixture {
    temporary: tempfile::TempDir,
    source: PathBuf,
    first: PathBuf,
    second: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Error> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path();
        let source = root.join("immutable-source.txt");
        let first = root.join("first-report.md");
        let second = root.join("second-report.md");
        fs::write(&source, b"source-v1\n")?;
        fs::write(&first, b"already-current\n")?;
        fs::write(&second, b"old-second\n")?;
        fs::set_permissions(&first, fs::Permissions::from_mode(0o640))?;
        fs::set_permissions(&second, fs::Permissions::from_mode(0o750))?;
        for arguments in [
            &["init", "--quiet"][..],
            &["config", "user.name", "Verification Refresh Test"][..],
            &["config", "user.email", "refresh-test@example.invalid"][..],
            &["add", "."][..],
            &["commit", "--quiet", "-m", "fixture"][..],
        ] {
            let status = Command::new("git")
                .args(arguments)
                .current_dir(root)
                .status()?;
            if !status.success() {
                return Err(Error::new("verification refresh fixture Git setup failed"));
            }
        }
        Ok(Self {
            temporary,
            source,
            first,
            second,
        })
    }

    fn root(&self) -> &Path {
        self.temporary.path()
    }

    fn snapshot(&self) -> Result<ImmutableRepositoryInputs, Error> {
        let mut snapshot = ImmutableRepositoryInputs::new(self.root())?;
        snapshot.read_bytes(&self.source)?;
        Ok(snapshot)
    }

    fn outputs(&self) -> Vec<(PathBuf, Vec<u8>)> {
        vec![
            (self.first.clone(), b"already-current\n".to_vec()),
            (self.second.clone(), b"new-second\n".to_vec()),
        ]
    }

    fn output_state(&self) -> Result<Vec<(Vec<u8>, u32)>, Error> {
        [&self.first, &self.second]
            .into_iter()
            .map(|path| {
                Ok((
                    fs::read(path)?,
                    fs::metadata(path)?.permissions().mode() & 0o7777,
                ))
            })
            .collect()
    }

    fn assert_no_artifacts(&self) -> Result<(), Error> {
        let leftovers = fs::read_dir(self.root())?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.contains(".refresh-") || name.contains(".backup-"))
            .collect::<Vec<_>>();
        if leftovers.is_empty() {
            Ok(())
        } else {
            Err(Error::new(format!(
                "verification refresh left temporary artifacts: {leftovers:?}"
            )))
        }
    }
}

fn successful_refresh_control() -> Result<(), Error> {
    let fixture = Fixture::new()?;
    let mut snapshot = fixture.snapshot()?;
    atomic_refresh_and_check(&fixture.outputs(), &mut snapshot)?;
    if fixture.output_state()?
        != vec![
            (b"already-current\n".to_vec(), 0o640),
            (b"new-second\n".to_vec(), 0o750),
        ]
        || snapshot
            .inputs
            .values()
            .any(|input| input.initial_reads != 1 || input.rehashes != 1)
    {
        return Err(Error::new(
            "verification refresh success control violated byte/mode/read invariants",
        ));
    }
    fixture.assert_no_artifacts()
}

fn preinstall_drift_control() -> Result<(), Error> {
    let fixture = Fixture::new()?;
    let mut snapshot = fixture.snapshot()?;
    let original = fixture.output_state()?;
    fs::write(&fixture.source, b"source-drift-before-install\n")?;
    if atomic_refresh_and_check(&fixture.outputs(), &mut snapshot).is_ok()
        || fixture.output_state()? != original
    {
        return Err(Error::new(
            "verification refresh accepted pre-install input drift",
        ));
    }
    fixture.assert_no_artifacts()
}

fn replacement_failure_control() -> Result<(), Error> {
    let fixture = Fixture::new()?;
    let mut snapshot = fixture.snapshot()?;
    let original = fixture.output_state()?;
    let result =
        atomic_refresh_with_hook(&fixture.outputs(), &mut snapshot, |event, _| match event {
            RefreshEvent::BeforeReplace(1) => {
                Err(RefreshError::new("injected second replacement failure"))
            }
            _ => Ok(()),
        });
    if result.is_ok() || fixture.output_state()? != original {
        return Err(Error::new(
            "verification refresh did not roll back a replacement failure",
        ));
    }
    fixture.assert_no_artifacts()
}

fn postinstall_drift_control() -> Result<(), Error> {
    let fixture = Fixture::new()?;
    let mut snapshot = fixture.snapshot()?;
    let original = fixture.output_state()?;
    let source = fixture.source.clone();
    let result = atomic_refresh_with_hook(&fixture.outputs(), &mut snapshot, move |event, _| {
        if event == RefreshEvent::AfterReplace(1) {
            fs::write(&source, b"source-drift-after-install\n").map_err(io_refresh_error)?;
        }
        Ok(())
    });
    if result.is_ok() || fixture.output_state()? != original {
        return Err(Error::new(
            "verification refresh did not roll back post-install input drift",
        ));
    }
    fixture.assert_no_artifacts()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behavioral_self_test_passes() {
        assert_eq!(
            self_test().expect("refresh self-test"),
            "native atomic refresh self-test passes (4 controls)"
        );
    }
}
