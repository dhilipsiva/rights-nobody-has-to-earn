// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local run diagnostics: measured phase timing, progress, and ETA.
//!
//! Everything here is observational. Progress, heartbeat, queued-lock, and
//! timing-summary lines go to stderr only, through one best-effort writer
//! that ignores write errors; the machine-readable document goes to one
//! canonical JSON file under the Git common directory, outside the working
//! tree. Nothing in this module writes to the stdout transcript that receipts
//! bind, nothing reads recorded timings into any verdict, and a recording
//! failure is swallowed after at most a best-effort stderr note, so no exit
//! status can depend on diagnostics. The document says the same thing about
//! itself in `evidence_scope` so a reader cannot mistake one machine's
//! wall-clock measurements for platform-independent assurance evidence.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cli::Error;

pub(crate) const SCHEMA: &str = "rights-verify-run-diagnostics/1";

/// The self-description every diagnostics document must carry verbatim.
pub(crate) const EVIDENCE_SCOPE: &str = "Wall-clock diagnostics from one local run on one \
machine. Values are platform- and load-dependent, change no verdict, and are not assurance \
evidence. Nothing here is receipt-bound: the only timing a verification receipt carries is \
its own run's command times beside its transcript digest, never this file.";

/// Best-effort stderr line. Write errors are deliberately ignored so that a
/// closed, broken, or blocked stderr can never panic the observational layer
/// or change an exit status — `eprintln!` would abort under this binary's
/// `panic = "abort"` profile the moment stderr failed.
pub(crate) fn stderr_note(line: &str) {
    let _ = writeln!(std::io::stderr().lock(), "{line}");
}

const BUILD_ENTRY_NAME: &str = "cargo build (verify.sh)";
pub(crate) const BUILD_STARTED_ENV: &str = "RIGHTS_VERIFY_BUILD_STARTED";
pub(crate) const BUILD_FINISHED_ENV: &str = "RIGHTS_VERIFY_BUILD_FINISHED";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunLabel {
    Quick,
    Full,
    EmitReceipt,
    CommitGate,
}

impl RunLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Full => "full",
            Self::EmitReceipt => "emit-receipt",
            Self::CommitGate => "commit-gate",
        }
    }

    fn file_name(self) -> String {
        format!("{}.json", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Canonical document model
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
    schema: String,
    evidence_scope: String,
    mode: String,
    run: RunRecord,
    build: Option<BuildRecord>,
    lock: Option<LockRecord>,
    phases: Vec<PhaseRecord>,
    critical_path: Vec<CriticalPathEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RunRecord {
    started_at_utc: String,
    finished_at_utc: String,
    elapsed_ms: u64,
    outcome: String,
    failed_phase: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct BuildRecord {
    elapsed_ms: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LockRecord {
    /// Elapsed wall time of the whole lock acquisition, including its
    /// uncontended overhead (Git subprocesses, token generation); `contended`
    /// alone marks an actual queue behind another holder.
    acquire_elapsed_ms: u64,
    contended: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PhaseRecord {
    name: String,
    started_offset_ms: u64,
    elapsed_ms: u64,
    outcome: String,
    details: Vec<DetailRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DetailRecord {
    name: String,
    elapsed_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CriticalPathEntry {
    name: String,
    elapsed_ms: u64,
    share_percent: u8,
}

/// Rank build and phases by measured duration against the run's total time.
///
/// The top-level pipeline is serial, so the whole sequence is the critical
/// path; this ranking identifies where its time went. Shares are of the total
/// including the build, rounded to whole percent.
fn critical_path(
    build: Option<BuildRecord>,
    phases: &[PhaseRecord],
    run_elapsed_ms: u64,
) -> Vec<CriticalPathEntry> {
    let total = run_elapsed_ms
        .saturating_add(build.map_or(0, |record| record.elapsed_ms))
        .max(1);
    let mut entries: Vec<CriticalPathEntry> = build
        .map(|record| CriticalPathEntry {
            name: BUILD_ENTRY_NAME.to_owned(),
            elapsed_ms: record.elapsed_ms,
            share_percent: share_percent(record.elapsed_ms, total),
        })
        .into_iter()
        .chain(phases.iter().map(|phase| CriticalPathEntry {
            name: phase.name.clone(),
            elapsed_ms: phase.elapsed_ms,
            share_percent: share_percent(phase.elapsed_ms, total),
        }))
        .collect();
    entries.sort_by(|left, right| {
        right
            .elapsed_ms
            .cmp(&left.elapsed_ms)
            .then_with(|| left.name.cmp(&right.name))
    });
    entries
}

fn share_percent(elapsed_ms: u64, total_ms: u64) -> u8 {
    let total = total_ms.max(1) as u128;
    let share = (elapsed_ms as u128 * 100 + total / 2) / total;
    share.min(100) as u8
}

/// Strict structural validation for a diagnostics document.
///
/// The self-test feeds this doctored documents that must be rejected; the
/// previous-run loader uses it so a stale or foreign file degrades to "no
/// prior measurement" instead of a wrong estimate.
pub(crate) fn validate_document(bytes: &[u8]) -> Result<Document, String> {
    let document: Document =
        serde_json::from_slice(bytes).map_err(|error| format!("not a diagnostics document: {error}"))?;
    if document.schema != SCHEMA {
        return Err(format!("unknown diagnostics schema: {}", document.schema));
    }
    if document.evidence_scope != EVIDENCE_SCOPE {
        return Err("diagnostics evidence scope does not carry the exact disclaimer".to_owned());
    }
    if document.mode.is_empty() {
        return Err("diagnostics mode is empty".to_owned());
    }
    match document.run.outcome.as_str() {
        "completed" => {
            if document.run.failed_phase.is_some() {
                return Err("completed run must not name a failed phase".to_owned());
            }
        }
        "failed" => {}
        other => return Err(format!("unknown run outcome: {other}")),
    }
    let mut previous_offset = 0;
    for (index, phase) in document.phases.iter().enumerate() {
        if phase.name.is_empty() {
            return Err("diagnostics phase has an empty name".to_owned());
        }
        if phase.started_offset_ms < previous_offset {
            return Err(format!(
                "phase offsets are not monotone at {}",
                phase.name
            ));
        }
        previous_offset = phase.started_offset_ms;
        match phase.outcome.as_str() {
            "completed" => {}
            "failed" if index + 1 == document.phases.len() => {}
            "failed" => return Err("a failed phase must be the last phase".to_owned()),
            other => return Err(format!("unknown phase outcome: {other}")),
        }
    }
    let expected = critical_path(document.build, &document.phases, document.run.elapsed_ms);
    if document.critical_path != expected {
        return Err("critical path does not match the recorded phases".to_owned());
    }
    Ok(document)
}

// ---------------------------------------------------------------------------
// Formatting and estimates (pure, self-tested)
// ---------------------------------------------------------------------------

fn seconds(ms: u64) -> String {
    format!("{}.{}s", ms / 1_000, (ms % 1_000) / 100)
}

#[derive(Clone, Debug, Default)]
struct Previous {
    total_ms: u64,
    phases: Vec<(String, u64)>,
}

impl Previous {
    fn from_document(document: &Document) -> Self {
        Self {
            total_ms: document.run.elapsed_ms,
            phases: document
                .phases
                .iter()
                .map(|phase| (phase.name.clone(), phase.elapsed_ms))
                .collect(),
        }
    }

    fn elapsed_of(&self, name: &str) -> Option<u64> {
        self.phases
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, elapsed)| *elapsed)
    }

    /// Previous durations of every phase not yet completed in this run.
    ///
    /// Matching is by name, so an added, removed, or reordered step degrades
    /// the estimate instead of breaking it.
    fn remaining_ms(&self, completed: &[PhaseRecord]) -> u64 {
        self.phases
            .iter()
            .filter(|(name, _)| !completed.iter().any(|phase| &phase.name == name))
            .map(|(_, elapsed)| *elapsed)
            .sum()
    }
}

fn progress_line(
    index: usize,
    name: &str,
    previous: Option<&Previous>,
    completed: &[PhaseRecord],
) -> String {
    let Some(previous) = previous else {
        return format!("progress: [{index}] {name}");
    };
    let last = match previous.elapsed_of(name) {
        Some(elapsed) => format!("last run {}", seconds(elapsed)),
        None => "no prior measurement".to_owned(),
    };
    format!(
        "progress: [{index}] {name} — {last}; ~{} remaining",
        seconds(previous.remaining_ms(completed))
    )
}

fn heartbeat_line(name: &str, active_ms: u64, previous_ms: Option<u64>) -> String {
    let comparison = match previous_ms {
        Some(elapsed) => format!("last run {}", seconds(elapsed)),
        None => "no prior measurement".to_owned(),
    };
    format!(
        "progress: {name} still active after {} ({comparison})",
        seconds(active_ms)
    )
}

fn estimate_source_line(mode: RunLabel, previous: &Previous) -> String {
    format!(
        "progress: estimates from the last recorded {} run ({} total)",
        mode.as_str(),
        seconds(previous.total_ms)
    )
}

fn summary_line(document: &Document, target: Option<&Path>) -> String {
    let mut line = String::new();
    if document.run.outcome == "failed" {
        let location = document
            .run
            .failed_phase
            .as_deref()
            .map(|name| format!(" in {name}"))
            .unwrap_or_default();
        line.push_str(&format!(
            "timing: {} verification failed{location} after {}",
            document.mode,
            seconds(document.run.elapsed_ms)
        ));
    } else {
        line.push_str(&format!(
            "timing: {} verification {}",
            document.mode,
            seconds(document.run.elapsed_ms)
        ));
        if let Some(build) = document.build {
            line.push_str(&format!(" (+ {} build)", seconds(build.elapsed_ms)));
        }
        let top = document
            .critical_path
            .iter()
            .take(3)
            .map(|entry| {
                format!(
                    "{} {} ({}%)",
                    entry.name,
                    seconds(entry.elapsed_ms),
                    entry.share_percent
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        if !top.is_empty() {
            line.push_str(&format!("; critical path: {top}"));
        }
    }
    if let Some(target) = target {
        line.push_str(&format!("; diagnostics: {}", target.display()));
    }
    line
}

/// Parse the two `EPOCHREALTIME`-shaped stamps verify.sh exports around the
/// cargo build. Any malformed, missing, or non-monotone pair yields `None`;
/// diagnostics never fail a run over a bad stamp.
fn build_record(started: Option<&str>, finished: Option<&str>) -> Option<BuildRecord> {
    let started = parse_epoch_micros(started?)?;
    let finished = parse_epoch_micros(finished?)?;
    let elapsed_us = finished.checked_sub(started)?;
    Some(BuildRecord {
        elapsed_ms: (elapsed_us / 1_000) as u64,
    })
}

fn parse_epoch_micros(value: &str) -> Option<u128> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let seconds: u128 = whole.parse().ok()?;
    let mut digits = fraction.bytes().take(6).fold(0_u128, |value, byte| {
        value * 10 + u128::from(byte - b'0')
    });
    for _ in fraction.len()..6 {
        digits *= 10;
    }
    Some(seconds * 1_000_000 + digits)
}

// ---------------------------------------------------------------------------
// Recorder
// ---------------------------------------------------------------------------

struct ActivePhase {
    name: String,
    started: Instant,
    started_offset_ms: u64,
    details: Vec<DetailRecord>,
}

struct Inner {
    mode: RunLabel,
    run_started: Instant,
    started_at_utc: String,
    quiet: bool,
    build: Option<BuildRecord>,
    lock: Option<LockRecord>,
    phases: Vec<PhaseRecord>,
    active: Option<ActivePhase>,
    previous: Option<Previous>,
    target: Option<PathBuf>,
    finished: bool,
}

impl Inner {
    fn close_active(&mut self, outcome: &str) {
        if let Some(active) = self.active.take() {
            self.phases.push(PhaseRecord {
                name: active.name,
                started_offset_ms: active.started_offset_ms,
                elapsed_ms: elapsed_ms(active.started.elapsed()),
                outcome: outcome.to_owned(),
                details: active.details,
            });
        }
    }
}

fn elapsed_ms(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

#[derive(Clone)]
pub(crate) struct Recorder {
    inner: Arc<Mutex<Inner>>,
}

impl Recorder {
    /// Production recorder for one instrumented run.
    ///
    /// The previous same-mode document, when present and valid, supplies the
    /// per-phase estimates; a missing or invalid one only mutes them.
    fn start(mode: RunLabel, root: &Path) -> Self {
        let target = crate::lock::git_common_dir(root)
            .ok()
            .map(|common| {
                common
                    .join("rights-verification/diagnostics")
                    .join(mode.file_name())
            });
        let previous = target
            .as_deref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| validate_document(&bytes).ok())
            .map(|document| Previous::from_document(&document));
        let build = build_record(
            std::env::var(BUILD_STARTED_ENV).ok().as_deref(),
            std::env::var(BUILD_FINISHED_ENV).ok().as_deref(),
        );
        let recorder = Self::assemble(mode, target, build, previous, false);
        {
            let inner = recorder.lock();
            if let Some(previous) = &inner.previous {
                stderr_note(&estimate_source_line(inner.mode, previous));
            }
        }
        recorder
    }

    fn assemble(
        mode: RunLabel,
        target: Option<PathBuf>,
        build: Option<BuildRecord>,
        previous: Option<Previous>,
        quiet: bool,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                mode,
                run_started: Instant::now(),
                started_at_utc: crate::lock::utc_now().unwrap_or_default(),
                quiet,
                build,
                lock: None,
                phases: Vec::new(),
                active: None,
                previous,
                target,
                finished: false,
            })),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn begin_phase(&self, name: &str) {
        let mut inner = self.lock();
        if inner.finished {
            return;
        }
        inner.close_active("completed");
        let index = inner.phases.len() + 1;
        let line = (!inner.quiet)
            .then(|| progress_line(index, name, inner.previous.as_ref(), &inner.phases));
        inner.active = Some(ActivePhase {
            name: name.to_owned(),
            started: Instant::now(),
            started_offset_ms: elapsed_ms(inner.run_started.elapsed()),
            details: Vec::new(),
        });
        drop(inner);
        if let Some(line) = line {
            stderr_note(&line);
        }
    }

    pub(crate) fn add_details(&self, details: impl IntoIterator<Item = (String, u64)>) {
        let mut inner = self.lock();
        if let Some(active) = inner.active.as_mut() {
            active
                .details
                .extend(details.into_iter().map(|(name, elapsed_ms)| DetailRecord {
                    name,
                    elapsed_ms,
                }));
        }
    }

    pub(crate) fn note_lock_contended(&self) {
        let mut inner = self.lock();
        inner.lock.get_or_insert(LockRecord {
            acquire_elapsed_ms: 0,
            contended: false,
        });
        if let Some(lock) = inner.lock.as_mut() {
            lock.contended = true;
        }
    }

    pub(crate) fn note_lock_acquired(&self, elapsed: Duration) {
        let mut inner = self.lock();
        let acquire_elapsed_ms = elapsed_ms(elapsed);
        match inner.lock.as_mut() {
            Some(lock) => lock.acquire_elapsed_ms = acquire_elapsed_ms,
            None => {
                inner.lock = Some(LockRecord {
                    acquire_elapsed_ms,
                    contended: false,
                });
            }
        }
    }

    /// Close the run, write the canonical document, and print one summary.
    ///
    /// Infallible by design: a write failure becomes a stderr warning so the
    /// run's exit status can never depend on diagnostics.
    pub(crate) fn finish(&self, failed: bool) {
        let quiet = self.lock().quiet;
        if let Err(warning) = self.finish_inner(failed)
            && !quiet
        {
            stderr_note(&format!("timing: diagnostics not recorded — {warning}"));
        }
    }

    fn finish_inner(&self, failed: bool) -> Result<(), String> {
        let mut inner = self.lock();
        if inner.finished {
            return Ok(());
        }
        inner.finished = true;
        let failed_phase = failed
            .then(|| inner.active.as_ref().map(|active| active.name.clone()))
            .flatten();
        inner.close_active(if failed { "failed" } else { "completed" });
        let run_elapsed = elapsed_ms(inner.run_started.elapsed());
        let document = Document {
            schema: SCHEMA.to_owned(),
            evidence_scope: EVIDENCE_SCOPE.to_owned(),
            mode: inner.mode.as_str().to_owned(),
            run: RunRecord {
                started_at_utc: inner.started_at_utc.clone(),
                finished_at_utc: crate::lock::utc_now().unwrap_or_default(),
                elapsed_ms: run_elapsed,
                outcome: if failed { "failed" } else { "completed" }.to_owned(),
                failed_phase,
            },
            build: inner.build,
            lock: inner.lock,
            phases: inner.phases.clone(),
            critical_path: critical_path(inner.build, &inner.phases, run_elapsed),
        };
        let target = inner.target.clone();
        let quiet = inner.quiet;
        drop(inner);
        let written = match &target {
            Some(path) => write_document(path, &document).map(|()| Some(path.as_path())),
            None => Ok(None),
        };
        if !quiet {
            stderr_note(&summary_line(
                &document,
                written.as_ref().ok().copied().flatten(),
            ));
        }
        written.map(|_| ())
    }

    /// Quiet recorder for the self-test's scratch scenarios. Never installed
    /// globally, so it cannot disturb the live run's own recording.
    fn scratch(mode: RunLabel, target: Option<PathBuf>) -> Self {
        Self::assemble(mode, target, None, None, true)
    }
}

fn write_document(target: &Path, document: &Document) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let parent = target
        .parent()
        .ok_or_else(|| "diagnostics target has no parent".to_owned())?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let mut bytes =
        serde_json::to_vec_pretty(document).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    temporary
        .as_file()
        .set_permissions(std::fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    temporary
        .write_all(&bytes)
        .and_then(|()| temporary.flush())
        .map_err(|error| error.to_string())?;
    temporary
        .persist(target)
        .map_err(|error| error.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Heartbeat
// ---------------------------------------------------------------------------

/// Periodic stderr note naming the active phase and its measured age.
///
/// The comparison with the last recorded duration gives a reader grounds to
/// suspect a stall; it cannot prove one — the process is alive either way,
/// and only the phase's eventual completion or failure settles it. The
/// queued state is reported separately by the verification lock.
pub(crate) struct Heartbeat {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}

impl Heartbeat {
    fn start(recorder: Recorder, interval: Duration) -> Self {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let observer = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            let (flag, signal) = &*observer;
            let mut stopped = flag.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            loop {
                // The stop flag can already be raised before this thread first
                // holds the lock; waiting first would sleep through the only
                // notification and block the joining dropper for a full
                // interval.
                if *stopped {
                    return;
                }
                let (next, timeout) = signal
                    .wait_timeout(stopped, interval)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                stopped = next;
                if *stopped {
                    return;
                }
                if !timeout.timed_out() {
                    continue;
                }
                let line = {
                    let inner = recorder.lock();
                    if inner.finished || inner.quiet {
                        None
                    } else {
                        inner.active.as_ref().map(|active| {
                            heartbeat_line(
                                &active.name,
                                elapsed_ms(active.started.elapsed()),
                                inner
                                    .previous
                                    .as_ref()
                                    .and_then(|previous| previous.elapsed_of(&active.name)),
                            )
                        })
                    }
                };
                // Release both locks before touching stderr: a blocked stderr
                // write must not be able to hold the stop flag hostage.
                drop(stopped);
                if let Some(line) = line {
                    stderr_note(&line);
                }
                stopped = flag.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Heartbeat {
    fn drop(&mut self) {
        let (flag, signal) = &*self.stop;
        *flag.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        signal.notify_all();
        // Signal and detach rather than join: the thread exits promptly on
        // its next wakeup, and a pathologically blocked stderr write must not
        // be able to stall the run's completion behind an unbounded join.
        drop(self.handle.take());
    }
}

// ---------------------------------------------------------------------------
// Process-global observation
// ---------------------------------------------------------------------------

static GLOBAL: OnceLock<Recorder> = OnceLock::new();

/// Install the one run recorder. Later calls are ignored; observation points
/// stay inert in uninstrumented modes and in tests that never initialise.
pub(crate) fn initialise(mode: RunLabel, root: &Path) {
    let _ = GLOBAL.set(Recorder::start(mode, root));
}

pub(crate) fn observe() -> Option<Recorder> {
    GLOBAL.get().cloned()
}

pub(crate) fn begin_phase(name: &str) {
    if let Some(recorder) = observe() {
        recorder.begin_phase(name);
    }
}

pub(crate) fn add_details(details: impl IntoIterator<Item = (String, u64)>) {
    if let Some(recorder) = observe() {
        recorder.add_details(details);
    }
}

pub(crate) fn note_lock_contended() {
    if let Some(recorder) = observe() {
        recorder.note_lock_contended();
    }
}

pub(crate) fn note_lock_acquired(elapsed: Duration) {
    if let Some(recorder) = observe() {
        recorder.note_lock_acquired(elapsed);
    }
}

pub(crate) fn finish(failed: bool) {
    if let Some(recorder) = observe() {
        recorder.finish(failed);
    }
}

pub(crate) fn start_heartbeat() -> Option<Heartbeat> {
    observe().map(|recorder| Heartbeat::start(recorder, HEARTBEAT_INTERVAL))
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

fn self_test_error(index: usize, name: &str, detail: impl Into<String>) -> Error {
    Error::new(format!(
        "run-diagnostics self-test control {index} ({name}) failed: {}",
        detail.into()
    ))
}

fn fixed_document() -> Document {
    let phases = vec![
        PhaseRecord {
            name: "alpha".to_owned(),
            started_offset_ms: 0,
            elapsed_ms: 100,
            outcome: "completed".to_owned(),
            details: vec![DetailRecord {
                name: "alpha/detail".to_owned(),
                elapsed_ms: 60,
            }],
        },
        PhaseRecord {
            name: "beta".to_owned(),
            started_offset_ms: 100,
            elapsed_ms: 300,
            outcome: "completed".to_owned(),
            details: Vec::new(),
        },
        PhaseRecord {
            name: "gamma".to_owned(),
            started_offset_ms: 400,
            elapsed_ms: 100,
            outcome: "completed".to_owned(),
            details: Vec::new(),
        },
    ];
    let build = Some(BuildRecord { elapsed_ms: 500 });
    Document {
        schema: SCHEMA.to_owned(),
        evidence_scope: EVIDENCE_SCOPE.to_owned(),
        mode: "full".to_owned(),
        run: RunRecord {
            started_at_utc: "2026-08-31T00:00:00Z".to_owned(),
            finished_at_utc: "2026-08-31T00:00:01Z".to_owned(),
            elapsed_ms: 500,
            outcome: "completed".to_owned(),
            failed_phase: None,
        },
        build,
        lock: Some(LockRecord {
            acquire_elapsed_ms: 40,
            contended: true,
        }),
        critical_path: critical_path(build, &phases, 500),
        phases,
    }
}

/// In-process behavioral controls for the diagnostics layer, run by both the
/// quick and full verification paths. Kernel-lock contention semantics remain
/// owned by `lock::receipt_protocol_self_test`; the controls here prove that
/// the instrumentation observes without influencing: identical report bytes,
/// unchanged failure selection under cancellation, atomic last-run documents,
/// and fail-open recording that can never turn into an exit-status change.
pub(crate) fn self_test() -> Result<String, Error> {
    // 1. Canonical serialisation, exact critical path, and round-trip.
    let document = fixed_document();
    if document.critical_path.len() != 4
        || document.critical_path[0].name != "cargo build (verify.sh)"
        || document.critical_path[0].share_percent != 50
        || document.critical_path[1].name != "beta"
        || document.critical_path[1].share_percent != 30
        || document.critical_path[2].name != "alpha"
        || document.critical_path[3].name != "gamma"
        || document.critical_path[2].elapsed_ms != 100
    {
        return Err(self_test_error(1, "critical path", "ranking drifted"));
    }
    let bytes = serde_json::to_vec_pretty(&document)
        .map_err(|error| self_test_error(1, "critical path", error.to_string()))?;
    validate_document(&bytes)
        .map_err(|error| self_test_error(1, "critical path", error))?;

    // 2. Watched-failing validation controls: each doctored document must be
    // rejected.
    let good: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| self_test_error(2, "validation controls", error.to_string()))?;
    let mutations: Vec<(&str, Box<dyn Fn(&mut serde_json::Value)>)> = vec![
        (
            "altered evidence scope",
            Box::new(|value| value["evidence_scope"] = "timings are evidence".into()),
        ),
        (
            "altered schema",
            Box::new(|value| value["schema"] = "rights-verify-run-diagnostics/0".into()),
        ),
        (
            "reordered critical path",
            Box::new(|value| {
                let path = value["critical_path"].as_array_mut().expect("array");
                path.swap(0, 1);
            }),
        ),
        (
            "unknown field",
            Box::new(|value| value["assurance"] = "granted".into()),
        ),
        (
            "non-terminal failed phase",
            Box::new(|value| value["phases"][0]["outcome"] = "failed".into()),
        ),
        (
            "completed run with failed phase",
            Box::new(|value| value["run"]["failed_phase"] = "beta".into()),
        ),
    ];
    for (label, mutate) in mutations {
        let mut copy = good.clone();
        mutate(&mut copy);
        let doctored = serde_json::to_vec(&copy)
            .map_err(|error| self_test_error(2, "validation controls", error.to_string()))?;
        if validate_document(&doctored).is_ok() {
            return Err(self_test_error(
                2,
                "validation controls",
                format!("doctored document was accepted: {label}"),
            ));
        }
    }

    // 3. Reporter byte-equivalence with and without an attached recorder.
    let recorder = Recorder::assemble(RunLabel::Full, None, None, None, true);
    let mut plain = Vec::new();
    let mut attached = Vec::new();
    for (buffer, recorder) in [
        (&mut plain, None),
        (&mut attached, Some(recorder.clone())),
    ] {
        let mut report = crate::report::Reporter::with_recorder(buffer, recorder);
        report
            .step("one")
            .and_then(|()| report.pass("first"))
            .and_then(|()| report.step("two"))
            .and_then(|()| report.pass("second"))
            .and_then(|()| report.flush())
            .map_err(|error| self_test_error(3, "reporter equivalence", error.to_string()))?;
    }
    if plain != attached {
        return Err(self_test_error(
            3,
            "reporter equivalence",
            "attached recorder changed the report bytes",
        ));
    }
    {
        let inner = recorder.lock();
        let seen: Vec<&str> = inner
            .phases
            .iter()
            .map(|phase| phase.name.as_str())
            .chain(inner.active.as_ref().map(|active| active.name.as_str()))
            .collect();
        if seen != ["one", "two"] {
            return Err(self_test_error(
                3,
                "reporter equivalence",
                format!("recorder saw phases {seen:?}"),
            ));
        }
    }

    // 4. First-failure recording: the active phase is marked failed, the
    // document says so, and a second finish is inert.
    let temporary = tempfile::Builder::new()
        .prefix("rights-diagnostics-self-test-")
        .tempdir()
        .map_err(|error| self_test_error(4, "first failure", error.to_string()))?;
    let target = temporary.path().join("failed.json");
    let failing = Recorder::scratch(RunLabel::Full, Some(target.clone()));
    failing.begin_phase("alpha");
    failing.begin_phase("beta");
    failing
        .finish_inner(true)
        .map_err(|error| self_test_error(4, "first failure", error))?;
    let written = std::fs::read(&target)
        .map_err(|error| self_test_error(4, "first failure", error.to_string()))?;
    let parsed = validate_document(&written)
        .map_err(|error| self_test_error(4, "first failure", error))?;
    if parsed.run.outcome != "failed"
        || parsed.run.failed_phase.as_deref() != Some("beta")
        || parsed.phases.last().map(|phase| phase.outcome.as_str()) != Some("failed")
        || parsed.phases.first().map(|phase| phase.outcome.as_str()) != Some("completed")
    {
        return Err(self_test_error(
            4,
            "first failure",
            "failed run was not recorded as its failing phase",
        ));
    }
    failing.begin_phase("gamma");
    failing.finish(false);
    let after = std::fs::read(&target)
        .map_err(|error| self_test_error(4, "first failure", error.to_string()))?;
    if after != written {
        return Err(self_test_error(
            4,
            "first failure",
            "a finished recorder accepted further recording",
        ));
    }

    // 5. Interrupted run: an unfinished recorder leaves the previous document
    // byte-identical and no partial file behind.
    {
        let interrupted = Recorder::scratch(RunLabel::Full, Some(target.clone()));
        interrupted.begin_phase("alpha");
        drop(interrupted);
    }
    let after_interrupt = std::fs::read(&target)
        .map_err(|error| self_test_error(5, "interrupted run", error.to_string()))?;
    let survivors = std::fs::read_dir(temporary.path())
        .map_err(|error| self_test_error(5, "interrupted run", error.to_string()))?
        .count();
    if after_interrupt != written || survivors != 1 {
        return Err(self_test_error(
            5,
            "interrupted run",
            "an interrupted recorder disturbed the previous document",
        ));
    }

    // 6. Unwritable target: recording degrades without an error the caller
    // could turn into an exit status.
    let blocked = Recorder::scratch(
        RunLabel::Full,
        Some(target.join("child-of-a-regular-file.json")),
    );
    blocked.begin_phase("alpha");
    if blocked.finish_inner(false).is_ok() {
        return Err(self_test_error(
            6,
            "unwritable target",
            "write through a regular file unexpectedly succeeded",
        ));
    }
    let degraded = Recorder::scratch(
        RunLabel::Full,
        Some(target.join("child-of-a-regular-file.json")),
    );
    degraded.begin_phase("alpha");
    degraded.finish(false);

    // 7. Cancellation passthrough: recording under the bounded scheduler
    // changes neither the failure selection nor completion, and the recorder
    // still finalises.
    let concurrent = Recorder::assemble(RunLabel::Full, None, None, None, true);
    concurrent.begin_phase("scheduled");
    let observer = concurrent.clone();
    let failure = crate::scheduler::run_bounded(0..4_usize, 2, move |_, job, cancellation| {
        observer.add_details([(format!("job-{job}"), job as u64)]);
        if job == 1 {
            return Err("watched failure");
        }
        while !cancellation.is_cancelled() {
            std::thread::yield_now();
        }
        Ok(job)
    });
    if !matches!(
        failure,
        Err(crate::scheduler::ScheduleError::JobFailed {
            source: "watched failure",
            ..
        })
    ) {
        return Err(self_test_error(
            7,
            "cancellation passthrough",
            "recording changed the scheduler's failure selection",
        ));
    }
    concurrent
        .finish_inner(true)
        .map_err(|error| self_test_error(7, "cancellation passthrough", error))?;

    // 8. Estimate and formatting fixtures, including build-stamp parsing.
    let previous = Previous {
        total_ms: 312_400,
        phases: vec![
            ("alpha".to_owned(), 100_000),
            ("beta".to_owned(), 118_200),
            ("gamma".to_owned(), 94_200),
        ],
    };
    let completed = [PhaseRecord {
        name: "alpha".to_owned(),
        started_offset_ms: 0,
        elapsed_ms: 99_000,
        outcome: "completed".to_owned(),
        details: Vec::new(),
    }];
    let fixtures = [
        (
            progress_line(2, "beta", Some(&previous), &completed),
            "progress: [2] beta — last run 118.2s; ~212.4s remaining",
        ),
        (
            progress_line(2, "delta", Some(&previous), &completed),
            "progress: [2] delta — no prior measurement; ~212.4s remaining",
        ),
        (
            progress_line(2, "beta", None, &completed),
            "progress: [2] beta",
        ),
        (
            heartbeat_line("beta", 94_000, Some(118_200)),
            "progress: beta still active after 94.0s (last run 118.2s)",
        ),
        (
            heartbeat_line("beta", 94_000, None),
            "progress: beta still active after 94.0s (no prior measurement)",
        ),
        (
            estimate_source_line(RunLabel::Full, &previous),
            "progress: estimates from the last recorded full run (312.4s total)",
        ),
        (
            crate::lock::queued_note("held by verify (pid 1)", 30.0),
            "queued: heavyweight verifier lock is busy; waiting up to 30s — held by verify (pid 1)",
        ),
        (
            summary_line(&fixed_document(), Some(Path::new("/tmp/full.json"))),
            "timing: full verification 0.5s (+ 0.5s build); critical path: \
             cargo build (verify.sh) 0.5s (50%), beta 0.3s (30%), alpha 0.1s (10%); \
             diagnostics: /tmp/full.json",
        ),
    ];
    for (actual, expected) in fixtures {
        if actual != expected {
            return Err(self_test_error(
                8,
                "formatting fixtures",
                format!("{actual:?} != {expected:?}"),
            ));
        }
    }
    let mut failed_document = fixed_document();
    failed_document.run.outcome = "failed".to_owned();
    failed_document.run.failed_phase = Some("beta".to_owned());
    if summary_line(&failed_document, None) != "timing: full verification failed in beta after 0.5s"
    {
        return Err(self_test_error(
            8,
            "formatting fixtures",
            "failure summary drifted",
        ));
    }
    if build_record(Some("12.345678"), Some("13.345678"))
        != Some(BuildRecord { elapsed_ms: 1_000 })
        || build_record(Some("13.0"), Some("12.0")).is_some()
        || build_record(Some("not-a-stamp"), Some("13.0")).is_some()
        || build_record(None, Some("13.0")).is_some()
    {
        return Err(self_test_error(
            8,
            "formatting fixtures",
            "build-stamp parsing drifted",
        ));
    }

    // 9. Lock observation lands in the document, and the heartbeat thread
    // starts and stops cleanly.
    let observed = Recorder::scratch(RunLabel::Quick, None);
    observed.note_lock_contended();
    observed.note_lock_acquired(Duration::from_millis(1_234));
    let heartbeat = Heartbeat::start(observed.clone(), Duration::from_secs(3_600));
    drop(heartbeat);
    {
        let inner = observed.lock();
        if inner.lock
            != Some(LockRecord {
                acquire_elapsed_ms: 1_234,
                contended: true,
            })
        {
            return Err(self_test_error(
                9,
                "lock observation",
                "queued lock wait was not recorded",
            ));
        }
    }

    Ok("run-diagnostics self-test: PASS — observation without influence".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_self_test_passes() {
        self_test().expect("run-diagnostics self-test");
    }

    #[test]
    fn share_percent_rounds_and_clamps() {
        assert_eq!(share_percent(0, 100), 0);
        assert_eq!(share_percent(1, 200), 1);
        assert_eq!(share_percent(50, 100), 50);
        assert_eq!(share_percent(100, 100), 100);
        assert_eq!(share_percent(300, 100), 100);
        assert_eq!(share_percent(5, 0), 100);
    }

    #[test]
    fn parse_epoch_micros_handles_epochrealtime_shapes() {
        assert_eq!(parse_epoch_micros("12.345678"), Some(12_345_678));
        assert_eq!(parse_epoch_micros("12"), Some(12_000_000));
        assert_eq!(parse_epoch_micros("12.3"), Some(12_300_000));
        assert_eq!(parse_epoch_micros("12.3456789"), Some(12_345_678));
        assert_eq!(parse_epoch_micros(""), None);
        assert_eq!(parse_epoch_micros("."), None);
        assert_eq!(parse_epoch_micros("12.34a"), None);
    }

    #[test]
    fn remaining_estimate_matches_by_name() {
        let previous = Previous {
            total_ms: 1_000,
            phases: vec![
                ("alpha".to_owned(), 100),
                ("beta".to_owned(), 300),
                ("gamma".to_owned(), 600),
            ],
        };
        let completed = [PhaseRecord {
            name: "beta".to_owned(),
            started_offset_ms: 0,
            elapsed_ms: 250,
            outcome: "completed".to_owned(),
            details: Vec::new(),
        }];
        assert_eq!(previous.remaining_ms(&completed), 700);
        assert_eq!(previous.remaining_ms(&[]), 1_000);
    }
}
