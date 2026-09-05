// SPDX-License-Identifier: MIT OR Apache-2.0

//! In-process parity runner for Nibli behavioural pin files.
//!
//! The API consumes already-loaded source text. This keeps filesystem access in
//! the verifier's immutable-input layer and avoids process startup, while still
//! cloning a fresh prepared engine snapshot for every pin file. A fixture is
//! therefore never re-read or recompiled per file, and state asserted by one
//! pin file cannot leak into the next.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use nibli_engine::EngineError;
use nibli_reason::KnowledgeBase;
use nibli_session::CoreSession;

pub(crate) const EXIT_OK: u8 = 0;
pub(crate) const EXIT_FINDING: u8 = 1;
pub(crate) const EXIT_HARNESS: u8 = 2;
pub(crate) const EXIT_DEFECT_RESOLVED: u8 = 3;

/// One immutable, caller-loaded input.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LoadedSource<'a> {
    pub(crate) display_name: &'a str,
    pub(crate) source: &'a str,
}

impl<'a> LoadedSource<'a> {
    pub(crate) const fn new(display_name: &'a str, source: &'a str) -> Self {
        Self {
            display_name,
            source,
        }
    }
}

/// Controls the only side-effecting pin directive, `:require`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PinOptions<'a> {
    pub(crate) allow_shell: bool,
    pub(crate) working_directory: Option<&'a Path>,
    pub(crate) cancellation: Option<&'a crate::scheduler::CancellationToken>,
}

/// Captured CLI-compatible output and machine-readable aggregate counts.
#[derive(Debug, Default)]
pub(crate) struct RunOutput {
    pub(crate) exit_code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) pins: usize,
    pub(crate) defects: usize,
    pub(crate) findings: Vec<String>,
    pub(crate) resolved: Vec<String>,
    pub(crate) harness: Vec<String>,
    pub(crate) files: Vec<FileOutput>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FileOutput {
    pub(crate) display_name: String,
    pub(crate) pins: usize,
    pub(crate) defects: usize,
    pub(crate) findings: usize,
    pub(crate) resolved: usize,
    pub(crate) harness: usize,
    /// Measured wall time for this file's execution. Diagnostics only: no
    /// verdict, count, or comparison reads it.
    pub(crate) elapsed_ms: u64,
}

/// A compiled knowledge-base snapshot reusable across independent pin suites.
///
/// The verifier owns one of these for the live constitution. Structural
/// checkers can therefore share the same parsed and materialised base instead
/// of paying the dominant fixture-load cost at every call boundary.
pub(crate) struct PreparedPinEngine {
    base: PreparedBase,
}

impl PreparedPinEngine {
    pub(crate) fn new(knowledge_bases: &[LoadedSource<'_>]) -> Self {
        Self {
            base: PreparedBase::new(knowledge_bases),
        }
    }

    /// Prepare an engine whose fixture load, snapshots, and queries all share
    /// one cooperative cancellation flag.
    pub(crate) fn new_cancellable(
        knowledge_bases: &[LoadedSource<'_>],
        cancellation: Arc<AtomicBool>,
    ) -> Self {
        Self {
            base: PreparedBase::new_cancellable(knowledge_bases, cancellation),
        }
    }

    /// Replace the cooperative flag before reusing this worker-local base for
    /// another independent job. Every snapshot cloned for that job inherits it.
    pub(crate) fn set_cancel_flag(&self, cancellation: Arc<AtomicBool>) {
        self.base.engine.kb().set_cancel_flag(cancellation);
    }

    pub(crate) fn run_files(
        &self,
        pin_files: &[LoadedSource<'_>],
        options: PinOptions<'_>,
    ) -> RunOutput {
        run_prepared_pin_files(&self.base, pin_files, options)
    }

    /// Run pins against a line-oriented derivative of the prepared source.
    ///
    /// Each deletion names one exact, trimmed source-line occurrence; each
    /// addition is one complete Nibli statement. The base is cloned, patched,
    /// and discarded, so callers can execute many mutation cases without
    /// reparsing and recompiling every unchanged source line.
    pub(crate) fn run_patched_files(
        &self,
        deletions: &[&str],
        additions: &[&str],
        pin_files: &[LoadedSource<'_>],
        options: PinOptions<'_>,
    ) -> RunOutput {
        if pin_files.is_empty() {
            return harness_only("no pin files supplied");
        }
        match self
            .base
            .run_patched_files(deletions, additions, pin_files, options)
        {
            Ok(reports) => finish_file_reports(pin_files, reports),
            Err(error) => harness_only(&error),
        }
    }

    /// Render stratification for a line-oriented derivative of the prepared
    /// source without changing the reusable base.
    ///
    /// Patch matching and diagnostics are identical to `run_patched_files`:
    /// deletions address exact trimmed source-line occurrences and additions
    /// are complete Nibli statements.
    pub(crate) fn dump_patched_strata(&self, deletions: &[&str], additions: &[&str]) -> RunOutput {
        match self.base.patched_strata_dump(deletions, additions) {
            Ok((stdout, harness)) if harness.is_empty() => RunOutput {
                exit_code: EXIT_OK,
                stdout,
                ..RunOutput::default()
            },
            Ok((_, harness)) => strata_harness_output(harness),
            Err(error) => strata_harness_output(vec![error]),
        }
    }

    pub(crate) fn dump_strata(&self) -> RunOutput {
        let (stdout, harness) = self.base.strata_dump();
        if harness.is_empty() {
            RunOutput {
                exit_code: EXIT_OK,
                stdout,
                ..RunOutput::default()
            }
        } else {
            strata_harness_output(harness)
        }
    }
}

#[derive(Debug, Default)]
struct Report {
    pins: usize,
    defects: usize,
    findings: Vec<String>,
    resolved: Vec<String>,
    harness: Vec<String>,
    elapsed_ms: u64,
}

/// Run pin files in input order, with a fresh engine for each file.
pub(crate) fn run_pin_files(
    knowledge_bases: &[LoadedSource<'_>],
    pin_files: &[LoadedSource<'_>],
    options: PinOptions<'_>,
) -> RunOutput {
    PreparedPinEngine::new(knowledge_bases).run_files(pin_files, options)
}

fn run_prepared_pin_files(
    prepared: &PreparedBase,
    pin_files: &[LoadedSource<'_>],
    options: PinOptions<'_>,
) -> RunOutput {
    if pin_files.is_empty() {
        return harness_only("no pin files supplied");
    }
    let reports = prepared.run_files(pin_files, options);
    finish_file_reports(pin_files, reports)
}

fn finish_file_reports(pin_files: &[LoadedSource<'_>], reports: Vec<Report>) -> RunOutput {
    let mut stdout = String::new();
    let mut total = Report::default();
    let mut files = Vec::with_capacity(pin_files.len());
    for (pin_file, report) in pin_files.iter().zip(reports) {
        files.push(FileOutput {
            display_name: pin_file.display_name.to_owned(),
            pins: report.pins,
            defects: report.defects,
            findings: report.findings.len(),
            resolved: report.resolved.len(),
            harness: report.harness.len(),
            elapsed_ms: report.elapsed_ms,
        });
        if report.defects == 0 && report.resolved.is_empty() {
            let _ = writeln!(
                stdout,
                "  {}: {} pins, {} findings, {} harness errors",
                pin_file.display_name,
                report.pins,
                report.findings.len(),
                report.harness.len(),
            );
        } else {
            let _ = writeln!(
                stdout,
                "  {}: {} pins ({} defects), {} findings, {} resolved, {} harness errors",
                pin_file.display_name,
                report.pins,
                report.defects,
                report.findings.len(),
                report.resolved.len(),
                report.harness.len(),
            );
        }
        total.pins += report.pins;
        total.defects += report.defects;
        total.findings.extend(report.findings);
        total.resolved.extend(report.resolved);
        total.harness.extend(report.harness);
    }

    let mut output = finish_run(total, stdout);
    output.files = files;
    output
}

/// Render the engine's stable stratification TSV for already-loaded fixtures.
pub(crate) fn dump_strata(knowledge_bases: &[LoadedSource<'_>]) -> RunOutput {
    if knowledge_bases.is_empty() {
        return harness_only("--strata needs at least one --kb <file.nibli>");
    }
    PreparedPinEngine::new(knowledge_bases).dump_strata()
}

fn strata_harness_output(harness: Vec<String>) -> RunOutput {
    let mut stderr = String::new();
    let _ = writeln!(stderr, "\nHARNESS/SCRIPT ERRORS ({}):", harness.len());
    for error in &harness {
        let _ = writeln!(stderr, "  ! {error}");
    }
    let _ = writeln!(
        stderr,
        "\nnibli-pin: HARNESS ERROR (exit {EXIT_HARNESS}) — dump not trustworthy"
    );
    RunOutput {
        exit_code: EXIT_HARNESS,
        stderr,
        harness,
        ..RunOutput::default()
    }
}

fn harness_only(message: &str) -> RunOutput {
    finish_run(
        Report {
            harness: vec![message.to_owned()],
            ..Report::default()
        },
        String::new(),
    )
}

fn finish_run(total: Report, mut stdout: String) -> RunOutput {
    let mut stderr = String::new();
    if !total.harness.is_empty() {
        let _ = writeln!(stderr, "\nHARNESS/SCRIPT ERRORS ({}):", total.harness.len());
        for error in &total.harness {
            let _ = writeln!(stderr, "  ! {error}");
        }
    }
    if !total.resolved.is_empty() {
        let _ = writeln!(
            stderr,
            "\nRESOLVED DEFECTS ({}) — a pinned FLAW no longer reproduces:",
            total.resolved.len()
        );
        for resolved in &total.resolved {
            let _ = writeln!(stderr, "  ✓ {resolved}");
        }
    }
    if !total.findings.is_empty() {
        let _ = writeln!(
            stderr,
            "\nFINDINGS ({}) — a pinned property regressed:",
            total.findings.len()
        );
        for finding in &total.findings {
            let _ = writeln!(stderr, "  ✗ {finding}");
        }
    }

    let exit_code = if !total.harness.is_empty() {
        let _ = writeln!(
            stderr,
            "\nnibli-pin: HARNESS ERROR (exit {EXIT_HARNESS}) — pins not trustworthy"
        );
        EXIT_HARNESS
    } else if !total.findings.is_empty() {
        let _ = writeln!(
            stderr,
            "\nnibli-pin: {} FINDING(S) (exit {EXIT_FINDING})",
            total.findings.len()
        );
        EXIT_FINDING
    } else if !total.resolved.is_empty() {
        let _ = writeln!(
            stderr,
            "\nnibli-pin: {} PINNED DEFECT(S) NO LONGER REPRODUCE (exit {EXIT_DEFECT_RESOLVED}) \
             — the artifact improved; update the pin and the prose that describes it",
            total.resolved.len()
        );
        EXIT_DEFECT_RESOLVED
    } else {
        if total.defects == 0 {
            let _ = writeln!(stdout, "nibli-pin: PASS — {} pins", total.pins);
        } else {
            let _ = writeln!(
                stdout,
                "nibli-pin: PASS — {} pins ({} encode defects that still reproduce)",
                total.pins, total.defects
            );
        }
        EXIT_OK
    };

    RunOutput {
        exit_code,
        stdout,
        stderr,
        pins: total.pins,
        defects: total.defects,
        findings: total.findings,
        resolved: total.resolved,
        harness: total.harness,
        files: Vec::new(),
    }
}

/// What the next statement is expected to do. Directives are one-shot.
enum Expect {
    Default,
    Accept,
    AcceptScoped,
    Refuse { class: Class, needle: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Class {
    Syntax,
    Semantic,
    Reasoning,
    Backend,
}

impl Class {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "syntax" => Some(Self::Syntax),
            "semantic" => Some(Self::Semantic),
            "reasoning" => Some(Self::Reasoning),
            "backend" => Some(Self::Backend),
            _ => None,
        }
    }

    fn of(error: &EngineError) -> Self {
        match error {
            EngineError::Syntax(_) => Self::Syntax,
            EngineError::Semantic(_) => Self::Semantic,
            EngineError::Reasoning(_) => Self::Reasoning,
            EngineError::Backend(_) => Self::Backend,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Semantic => "semantic",
            Self::Reasoning => "reasoning",
            Self::Backend => "backend",
        }
    }
}

fn parse_refuse(rest: &str) -> Result<Expect, String> {
    let rest = rest.trim();
    let (class_token, pattern) = rest
        .split_once(char::is_whitespace)
        .ok_or_else(|| format!(":refuse needs a class and /pattern/ (got {rest:?})"))?;
    let class = Class::parse(class_token.trim()).ok_or_else(|| {
        format!(
            ":refuse unknown class {:?} (want syntax|semantic|reasoning|backend)",
            class_token.trim()
        )
    })?;
    let pattern = pattern.trim();
    let needle = pattern
        .strip_prefix('/')
        .and_then(|value| value.strip_suffix('/'))
        .ok_or_else(|| format!(":refuse pattern must be /slash-delimited/ (got {pattern:?})"))?;
    if needle.is_empty() {
        return Err(":refuse pattern must not be empty".to_owned());
    }
    Ok(Expect::Refuse {
        class,
        needle: needle.to_owned(),
    })
}

fn parse_quoted(rest: &str) -> Option<String> {
    let value = rest.trim();
    let inner = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.trim().to_owned())
}

fn is_pinnable_verdict(verdict: &str) -> bool {
    verdict == "TRUE"
        || verdict == "FALSE"
        || verdict == "UNKNOWN"
        || verdict.starts_with("UNKNOWN (")
}

fn verdict_matches(pinned: &str, actual: &str) -> bool {
    if pinned == "UNKNOWN" {
        actual == "UNKNOWN" || actual.starts_with("UNKNOWN (")
    } else {
        pinned == actual
    }
}

fn one_way_declaration(line: &str) -> Option<&'static str> {
    let line = line.trim_start();
    ["derived_only", "admits"].into_iter().find(|declaration| {
        line.starts_with(declaration) && line[declaration.len()..].trim_start().starts_with('(')
    })
}

fn load_fixtures(
    engine: &CoreSession,
    knowledge_bases: &[LoadedSource<'_>],
    harness: &mut Vec<String>,
    source_fact_ids: &mut BTreeMap<String, Vec<Vec<u64>>>,
    cancellation: Option<&AtomicBool>,
) {
    // Reject pin-language material before compiling anything. The valid source
    // is loaded statement-by-statement: Nibli's grammar accepts a multi-root
    // unit, but very large units currently scale worse than the linear loader.
    // The prepared result is cloned for every pin file, so this parse happens
    // once per distinct KB rather than once per file.
    for knowledge_base in knowledge_bases {
        for (index, raw) in knowledge_base.source.lines().enumerate() {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                harness.push("fixture loading cancelled".to_owned());
                return;
            }
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with(':') || line.starts_with('?') {
                harness.push(format!(
                    "{}:{}: a --kb fixture is plain KB text — directives and `?` \
                     queries belong in the pin file, not the artifact under test",
                    knowledge_base.display_name,
                    index + 1
                ));
            }
        }
    }
    if !harness.is_empty() {
        return;
    }

    load_fixtures_linewise(
        engine,
        knowledge_bases,
        harness,
        source_fact_ids,
        cancellation,
    );
}

fn load_fixtures_linewise(
    engine: &CoreSession,
    knowledge_bases: &[LoadedSource<'_>],
    harness: &mut Vec<String>,
    source_fact_ids: &mut BTreeMap<String, Vec<Vec<u64>>>,
    cancellation: Option<&AtomicBool>,
) {
    for knowledge_base in knowledge_bases {
        for (index, raw) in knowledge_base.source.lines().enumerate() {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                harness.push("fixture loading cancelled".to_owned());
                return;
            }
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('#') {
                source_fact_ids
                    .entry(line.to_owned())
                    .or_default()
                    .push(Vec::new());
                continue;
            }
            match engine.assert_text(line) {
                Ok(asserted) => {
                    source_fact_ids
                        .entry(line.to_owned())
                        .or_default()
                        .push(asserted.into_iter().map(|(id, _)| id).collect());
                }
                Err(error) => {
                    harness.push(format!(
                        "{}:{}: fixture line failed to load — [{}] {error}",
                        knowledge_base.display_name,
                        index + 1,
                        Class::of(&error).name()
                    ));
                }
            }
        }
    }
}

enum PreconditionOutcome {
    Met,
    Unmet(String),
    Broken(String),
}

fn collect_precondition_output(
    reader: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
) -> Vec<u8> {
    match reader {
        Some(reader) => reader.join().ok().and_then(Result::ok).unwrap_or_default(),
        None => Vec::new(),
    }
}

fn run_precondition(
    command: &str,
    working_directory: Option<&Path>,
    cancellation: Option<&crate::scheduler::CancellationToken>,
) -> PreconditionOutcome {
    if cancellation.is_some_and(crate::scheduler::CancellationToken::is_cancelled) {
        return PreconditionOutcome::Broken("cancelled before launch".to_owned());
    }
    let mut child = Command::new("sh");
    child
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        child.process_group(0);
    }
    if let Some(working_directory) = working_directory {
        child.current_dir(working_directory);
    }
    let mut child = match child.spawn() {
        Ok(child) => child,
        Err(error) => return PreconditionOutcome::Broken(format!("could not run: {error}")),
    };
    let stdout = child.stdout.take().map(|mut output| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            output.read_to_end(&mut bytes).map(|_| bytes)
        })
    });
    let stderr = child.stderr.take().map(|mut output| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            output.read_to_end(&mut bytes).map(|_| bytes)
        })
    });
    let waited = match cancellation {
        Some(cancellation) => {
            #[cfg(unix)]
            let result = crate::scheduler::wait_for_child_group(
                &mut child,
                cancellation,
                Duration::from_millis(10),
            );
            #[cfg(not(unix))]
            let result = crate::scheduler::wait_for_child(
                &mut child,
                cancellation,
                Duration::from_millis(10),
            );
            result
        }
        None => child.wait().map(crate::scheduler::ChildWait::Exited),
    };
    let stdout = collect_precondition_output(stdout);
    let stderr = collect_precondition_output(stderr);
    match waited {
        Err(error) => PreconditionOutcome::Broken(format!("could not wait: {error}")),
        Ok(waited) if waited.was_cancelled() => {
            PreconditionOutcome::Broken("cancelled; child terminated and reaped".to_owned())
        }
        Ok(waited) if waited.status().code() == Some(127) => PreconditionOutcome::Broken(
            "exited 127 (command not found) — the check itself is broken".to_owned(),
        ),
        Ok(waited) if waited.status().success() => PreconditionOutcome::Met,
        Ok(waited) => {
            let code = waited
                .status()
                .code()
                .map_or_else(|| "signal".to_owned(), |code| code.to_string());
            let stdout = String::from_utf8_lossy(&stdout);
            let stderr = String::from_utf8_lossy(&stderr);
            let tail = stdout
                .lines()
                .chain(stderr.lines())
                .take(3)
                .collect::<Vec<_>>()
                .join(" / ");
            PreconditionOutcome::Unmet(if tail.is_empty() {
                format!("failed (exit {code})")
            } else {
                format!("failed (exit {code}): {tail}")
            })
        }
    }
}

struct PreparedBase {
    engine: CoreSession,
    harness: Vec<String>,
    source_fact_ids: BTreeMap<String, Vec<Vec<u64>>>,
}

impl PreparedBase {
    fn new(knowledge_bases: &[LoadedSource<'_>]) -> Self {
        let engine = CoreSession::new();
        Self::load(engine, knowledge_bases, None)
    }

    fn new_cancellable(
        knowledge_bases: &[LoadedSource<'_>],
        cancellation: Arc<AtomicBool>,
    ) -> Self {
        let engine = CoreSession::new();
        engine.kb().set_cancel_flag(Arc::clone(&cancellation));
        Self::load(engine, knowledge_bases, Some(&cancellation))
    }

    fn load(
        engine: CoreSession,
        knowledge_bases: &[LoadedSource<'_>],
        cancellation: Option<&AtomicBool>,
    ) -> Self {
        let mut harness = Vec::new();
        let mut source_fact_ids = BTreeMap::new();
        load_fixtures(
            &engine,
            knowledge_bases,
            &mut harness,
            &mut source_fact_ids,
            cancellation,
        );
        Self {
            engine,
            harness,
            source_fact_ids,
        }
    }

    fn run_files(&self, pin_files: &[LoadedSource<'_>], options: PinOptions<'_>) -> Vec<Report> {
        if !self.harness.is_empty() {
            return pin_files
                .iter()
                .map(|_| Report {
                    harness: self.harness.clone(),
                    ..Report::default()
                })
                .collect();
        }

        // Assertion-bearing files already receive their own clone in
        // `run_files_against_base`. When every supplied file can assert, an
        // additional outer query-only snapshot is therefore redundant. This
        // matters for the state-form worker pool: each shard keeps its fresh
        // engine boundary while avoiding a second clone of the 5.6 MiB source
        // and its fully materialised knowledge base.
        if pin_files
            .iter()
            .all(|pin_file| pin_file_can_assert(pin_file.source))
        {
            return run_files_against_base(&self.engine, self.engine.kb(), pin_files, options);
        }

        // A query-only pin file cannot change the engine. Run every such file
        // against one prepared snapshot, while assertion-bearing files retain
        // their own clone. Generated executable families are overwhelmingly
        // query-only, so this removes hundreds of deep KB clones without
        // weakening fresh-engine isolation for files that can mutate state.
        // Keeping the whole loop inside the outer snapshot also preserves file
        // order for `:require` checks.
        self.engine
            .kb()
            .with_assumptions(&[], |query_only_base| {
                run_files_against_base(&self.engine, query_only_base, pin_files, options)
            })
            .unwrap_or_else(|error| {
                pin_files
                    .iter()
                    .map(|pin_file| Report {
                        harness: vec![format!(
                            "{}: could not create a fresh engine snapshot — [{}] {error}",
                            pin_file.display_name,
                            Class::of(&error).name()
                        )],
                        ..Report::default()
                    })
                    .collect()
            })
    }

    fn run_patched_files(
        &self,
        deletions: &[&str],
        additions: &[&str],
        pin_files: &[LoadedSource<'_>],
        options: PinOptions<'_>,
    ) -> Result<Vec<Report>, String> {
        self.with_patched(deletions, additions, |knowledge_base| {
            run_files_against_base(&self.engine, knowledge_base, pin_files, options)
        })
    }

    fn patched_strata_dump(
        &self,
        deletions: &[&str],
        additions: &[&str],
    ) -> Result<(String, Vec<String>), String> {
        self.with_patched(deletions, additions, strata_dump)
    }

    fn with_patched<T>(
        &self,
        deletions: &[&str],
        additions: &[&str],
        action: impl FnOnce(&KnowledgeBase) -> T,
    ) -> Result<T, String> {
        if !self.harness.is_empty() {
            return Err(format!(
                "prepared knowledge base has {} fixture error(s): {}",
                self.harness.len(),
                self.harness.join("; ")
            ));
        }

        self.engine
            .kb()
            .with_assumptions(&[], |knowledge_base| {
                let mut used_occurrences: BTreeMap<&str, usize> = BTreeMap::new();
                for raw in deletions {
                    let line = raw.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let occurrence = used_occurrences.entry(line).or_default();
                    let Some(ids) = self
                        .source_fact_ids
                        .get(line)
                        .and_then(|matches| matches.get(*occurrence))
                    else {
                        return Err(format!(
                            "patch deletion occurrence {} is absent from the prepared source: {line:?}",
                            *occurrence + 1
                        ));
                    };
                    *occurrence += 1;
                    for id in ids {
                        knowledge_base.retract_fact(*id).map_err(|error| {
                            format!("could not retract patch source line {line:?} (fact #{id}): {error}")
                        })?;
                    }
                }

                let engine = EngineView {
                    compiler: &self.engine,
                    knowledge_base,
                };
                for raw in additions {
                    let line = raw.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    engine.assert_text(line).map_err(|error| {
                        format!(
                            "patch addition failed to load — [{}] {error}: {line:?}",
                            Class::of(&error).name()
                        )
                    })?;
                }

                Ok(action(knowledge_base))
            })
            .map_err(|error| {
                format!(
                    "could not create a patched engine snapshot — [{}] {error}",
                    Class::of(&error).name()
                )
            })?
    }

    fn strata_dump(&self) -> (String, Vec<String>) {
        if !self.harness.is_empty() {
            return (String::new(), self.harness.clone());
        }
        strata_dump(self.engine.kb())
    }
}

fn run_files_against_base(
    compiler: &CoreSession,
    query_only_base: &KnowledgeBase,
    pin_files: &[LoadedSource<'_>],
    options: PinOptions<'_>,
) -> Vec<Report> {
    pin_files
        .iter()
        .map(|pin_file| {
            let started = Instant::now();
            let mut report = if pin_file_can_assert(pin_file.source) {
                query_only_base
                    .with_assumptions(&[], |isolated| {
                        run_file_with_engine(pin_file, compiler, isolated, options)
                    })
                    .unwrap_or_else(|error| Report {
                        harness: vec![format!(
                            "{}: could not create a fresh engine snapshot — [{}] {error}",
                            pin_file.display_name,
                            Class::of(&error).name()
                        )],
                        ..Report::default()
                    })
            } else {
                run_file_with_engine(pin_file, compiler, query_only_base, options)
            };
            report.elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            report
        })
        .collect()
}

fn pin_file_can_assert(source: &str) -> bool {
    source.lines().any(|raw| {
        let line = raw.trim();
        !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with('?')
            && !line.starts_with(':')
    })
}

struct EngineView<'a> {
    compiler: &'a CoreSession,
    knowledge_base: &'a KnowledgeBase,
}

impl EngineView<'_> {
    fn assert_text(&self, text: &str) -> Result<Vec<u64>, EngineError> {
        let buffer = self.compiler.compile_text(text)?;
        self.knowledge_base.validate_assertion(&buffer)?;
        let mut ids = Vec::new();
        for root in buffer.split_roots() {
            ids.push(self.knowledge_base.assert_fact(root, text.to_owned())?);
        }
        Ok(ids)
    }

    fn query_holds(&self, text: &str) -> Result<nibli_engine::EngineQueryResult, EngineError> {
        let buffer = self.compiler.compile_query_text(text)?;
        self.knowledge_base.query_entailment(buffer)
    }

    fn retract_fact(&self, id: u64) -> Result<(), EngineError> {
        self.knowledge_base.retract_fact(id)
    }
}

fn run_file_with_engine(
    pin_file: &LoadedSource<'_>,
    compiler: &CoreSession,
    knowledge_base: &KnowledgeBase,
    options: PinOptions<'_>,
) -> Report {
    let name = pin_file.display_name;
    let source = pin_file.source;
    let mut report = Report::default();
    let engine = EngineView {
        compiler,
        knowledge_base,
    };

    let mut expect = Expect::Default;
    let mut defect: Option<String> = None;
    let mut expected_pin_count: Option<usize> = None;
    let lines: Vec<&str> = source.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        if options
            .cancellation
            .is_some_and(crate::scheduler::CancellationToken::is_cancelled)
        {
            report.harness.push(format!("{name}: execution cancelled"));
            break;
        }
        let raw = lines[index];
        let line = raw.trim();
        index += 1;
        if line.is_empty() {
            continue;
        }
        if let Some(annotation) = line.strip_prefix("# =>") {
            report.harness.push(format!(
                "{name}:{index}: stray `# =>{}` annotation — no `?` query precedes it",
                annotation.trim_end()
            ));
            continue;
        }
        if line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix(":refuse") {
            if !matches!(expect, Expect::Default) {
                report.harness.push(format!(
                    "{name}:{index}: directive follows an unconsumed directive — each applies to the NEXT statement only"
                ));
            }
            match parse_refuse(rest) {
                Ok(parsed) => expect = parsed,
                Err(error) => report.harness.push(format!("{name}:{index}: {error}")),
            }
            continue;
        }
        if line == ":accept-scoped" {
            if !matches!(expect, Expect::Default) {
                report.harness.push(format!(
                    "{name}:{index}: directive follows an unconsumed directive — each applies to the NEXT statement only"
                ));
            }
            expect = Expect::AcceptScoped;
            continue;
        }
        if line == ":accept" {
            if !matches!(expect, Expect::Default) {
                report.harness.push(format!(
                    "{name}:{index}: directive follows an unconsumed directive — each applies to the NEXT statement only"
                ));
            }
            expect = Expect::Accept;
            continue;
        }
        if let Some(rest) = line.strip_prefix(":expect-pins") {
            match rest.trim().parse::<usize>() {
                Ok(count) => expected_pin_count = Some(count),
                Err(_) => report.harness.push(format!(
                    "{name}:{index}: :expect-pins needs a number (got {rest:?})"
                )),
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix(":defect") {
            if defect.is_some() {
                report.harness.push(format!(
                    "{name}:{index}: `:defect` follows an unconsumed `:defect` — each applies \
                     to the NEXT pin only"
                ));
            }
            match parse_quoted(rest) {
                Some(reason) if !reason.is_empty() => defect = Some(reason),
                _ => report.harness.push(format!(
                    "{name}:{index}: `:defect` needs a non-empty quoted reason saying WHAT WOULD \
                     FLIP IT, e.g. `:defect \"narrowing the contamination rule\"` — the \
                     reason is the whole value, since a bare marker cannot tell a reader \
                     what to do when it fires"
                )),
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix(":require") {
            let command = rest.trim();
            if command.is_empty() {
                report
                    .harness
                    .push(format!("{name}:{index}: `:require` needs a shell command"));
                continue;
            }
            if !options.allow_shell {
                report.harness.push(format!(
                    "{name}:{index}: `:require` needs --allow-shell. The pin language is closed \
                     on purpose: nothing in a pin file may execute shell during `just ci`. \
                     Pass --allow-shell to opt in for a suite you control."
                ));
                continue;
            }
            let marked = defect.take();
            report.pins += 1;
            match run_precondition(&command, options.working_directory, options.cancellation) {
                PreconditionOutcome::Met => {
                    if marked.is_some() {
                        report.defects += 1;
                    }
                }
                PreconditionOutcome::Unmet(detail) => {
                    let message = format!("{name}:{index}: precondition {command:?} {detail}");
                    match marked {
                        Some(reason) => report
                            .resolved
                            .push(format!("{message} — pinned as a defect ({reason})")),
                        None => report.findings.push(message),
                    }
                }
                PreconditionOutcome::Broken(detail) => report
                    .harness
                    .push(format!("{name}:{index}: precondition {command:?} {detail}")),
            }
            continue;
        }
        if line.starts_with(':') {
            report
                .harness
                .push(format!("{name}:{index}: unknown directive {line:?}"));
            continue;
        }

        if let Some(query) = line.strip_prefix('?') {
            if !matches!(expect, Expect::Default) {
                report.harness.push(format!(
                    "{name}:{index}: :accept/:refuse applies to an ASSERTION, not a `?` query"
                ));
                expect = Expect::Default;
            }
            let query = query.trim();
            let mut annotation_index = index;
            while annotation_index < lines.len() && lines[annotation_index].trim().is_empty() {
                annotation_index += 1;
            }
            let Some(pinned) = lines
                .get(annotation_index)
                .map(|line| line.trim())
                .and_then(|line| line.strip_prefix("# =>"))
                .map(str::trim)
            else {
                report.harness.push(format!(
                    "{name}:{index}: query {query:?} has no `# => <verdict>` annotation (pins are mandatory)"
                ));
                continue;
            };
            index = annotation_index + 1;

            if !is_pinnable_verdict(&pinned) {
                report.harness.push(format!(
                    "{name}:{index}: {pinned:?} is not a pinnable verdict \
                     (TRUE|FALSE|UNKNOWN only — RESOURCE_EXCEEDED is a resource outcome, \
                     not a logical one, and is runtime-dependent by design)"
                ));
                continue;
            }

            report.pins += 1;
            let marked = defect.take();
            match engine.query_holds(query) {
                Err(_)
                    if options
                        .cancellation
                        .is_some_and(crate::scheduler::CancellationToken::is_cancelled) =>
                {
                    report.harness.push(format!("{name}: execution cancelled"));
                }
                Err(error) => report.harness.push(format!(
                    "{name}:{index}: query {query:?} failed to compile, so it has no verdict to \
                     compare with the pinned {pinned:?} — [{}] {error}",
                    Class::of(&error).name()
                )),
                Ok(result) => {
                    let actual = nibli_engine::display_query_result(&result);
                    if actual.starts_with("RESOURCE_EXCEEDED") {
                        report.harness.push(format!(
                            "{name}:{index}: query {query:?} exhausted a resource ({actual}) — \
                             infrastructure outcome, not a verdict; pin excluded"
                        ));
                    } else if !verdict_matches(&pinned, &actual) {
                        match marked {
                            Some(reason) => report.resolved.push(format!(
                                "{name}:{index}: query {query:?} was pinned {pinned:?} as a DEFECT \
                                 ({reason}) but now answers {actual:?}"
                            )),
                            None => report.findings.push(format!(
                                "{name}:{index}: query {query:?} pinned {pinned:?} but got {actual:?}"
                            )),
                        }
                    } else if marked.is_some() {
                        report.defects += 1;
                    }
                }
            }
            continue;
        }

        let outcome = engine.assert_text(line);
        let expectation = std::mem::replace(&mut expect, Expect::Default);
        let marked = defect.take();
        let pins_before = report.pins;
        let failures_before = report.findings.len() + report.resolved.len();
        let fail = |report: &mut Report, message: String| match &marked {
            Some(reason) => report
                .resolved
                .push(format!("{message} — pinned as a DEFECT ({reason})")),
            None => report.findings.push(message),
        };
        match (expectation, outcome) {
            (Expect::Default, Ok(_)) => {}
            (Expect::Default, Err(error)) => fail(
                &mut report,
                format!(
                    "{name}:{index}: {line:?} failed to load — [{}] {error}",
                    Class::of(&error).name()
                ),
            ),
            (Expect::Accept, Ok(_)) => report.pins += 1,
            (Expect::AcceptScoped, Ok(ids)) => {
                report.pins += 1;
                if let Some(declaration) = one_way_declaration(line) {
                    report.harness.push(format!(
                        "{name}:{index}: `:accept-scoped` cannot scope a `{declaration}` declaration — it is \
                         one-way by design and survives the retraction, so the scope would be a \
                         silent no-op. Use plain `:accept` and put it where its effect belongs."
                    ));
                }
                for id in ids {
                    if let Err(error) = engine.retract_fact(id) {
                        report.harness.push(format!(
                            "{name}:{index}: `:accept-scoped` could not discard {line:?} (fact #{id}): \
                             {error} — the knowledge base is no longer clean, so pins below it \
                             cannot be trusted"
                        ));
                    }
                }
            }
            (Expect::AcceptScoped, Err(error)) => {
                report.pins += 1;
                fail(
                    &mut report,
                    format!(
                        "{name}:{index}: :accept-scoped but {line:?} was REFUSED — [{}] {error}",
                        Class::of(&error).name()
                    ),
                );
            }
            (Expect::Accept, Err(error)) => {
                report.pins += 1;
                fail(
                    &mut report,
                    format!(
                        "{name}:{index}: :accept but {line:?} was REFUSED — [{}] {error}",
                        Class::of(&error).name()
                    ),
                );
            }
            (Expect::Refuse { class, needle }, Ok(_)) => {
                report.pins += 1;
                fail(
                    &mut report,
                    format!(
                        "{name}:{index}: :refuse {} /{needle}/ but {line:?} was ACCEPTED \
                         — the guarantee this pin protects is GONE",
                        class.name()
                    ),
                );
            }
            (Expect::Refuse { class, needle }, Err(error)) => {
                report.pins += 1;
                let actual_class = Class::of(&error);
                let message = error.to_string();
                if actual_class != class {
                    fail(
                        &mut report,
                        format!(
                            "{name}:{index}: :refuse {} but {line:?} failed as [{}] instead — {message} \
                             (a different error class is NOT the property under test)",
                            class.name(),
                            actual_class.name()
                        ),
                    );
                } else if !message.contains(&needle) {
                    fail(
                        &mut report,
                        format!(
                            "{name}:{index}: :refuse {} /{needle}/ matched the class but not the message — got {message}",
                            class.name()
                        ),
                    );
                }
            }
        }
        if marked.is_some() {
            let counted = report.pins > pins_before;
            let failed = report.findings.len() + report.resolved.len() > failures_before;
            if !counted {
                report.harness.push(format!(
                    "{name}:{index}: `:defect` marks {line:?}, which is not a pin — put it before a \
                     `?` query or an :accept/:refuse statement"
                ));
            } else if !failed {
                report.defects += 1;
            }
        }
    }

    if !matches!(expect, Expect::Default) {
        report.harness.push(format!(
            "{name}: file ends with an unconsumed :accept/:refuse directive"
        ));
    }
    if let Some(reason) = defect {
        report.harness.push(format!(
            "{name}: file ends with an unconsumed `:defect` ({reason}) — it marks no pin"
        ));
    }
    match expected_pin_count {
        None => report.harness.push(format!(
            "{name}: no `:expect-pins <n>` floor — without it a hollowed-out file passes vacuously"
        )),
        Some(expected) if expected != report.pins => report.harness.push(format!(
            "{name}: :expect-pins {expected} but {} pins ran — pins were added or lost; \
             adjust the floor consciously in the same diff",
            report.pins
        )),
        Some(_) => {}
    }
    report
}

fn strata_dump(knowledge_base: &KnowledgeBase) -> (String, Vec<String>) {
    let rows = knowledge_base.stratification_report();
    let max_stratum = rows.iter().map(|row| row.stratum).max().unwrap_or(0);
    let base = rows.iter().filter(|row| row.base).count();
    let mut output = String::new();
    output.push_str("# nibli-strata v1\n");
    output.push_str("# Produced by `nibli-pin --strata` from the engine's own dependency graph\n");
    output.push_str("# (`pred_dep_graph`), the same one `check_stratification` gates rules on.\n");
    output.push_str("# columns: predicate <TAB> stratum <TAB> base|derived <TAB> edges\n");
    output.push_str("# edges:   comma-separated, `+name` positive, `-name` negative (NAF);\n");
    output.push_str("#          empty field = no outgoing edges. An edge means \"reads\".\n");
    output.push_str("# names:   SURFACE relations — event-decomposed role predicates (`p_x1`)\n");
    output.push_str("#          are collapsed onto their anchor (`p`). `event` and `__abs_<id>`\n");
    output
        .push_str("#          are compiler artifacts of `event { }` abstractions, and `equals`\n");
    output.push_str("#          is the `=` identity builtin — a disequality guard `~($a = $b)`\n");
    output.push_str("#          is a NEGATIVE edge to it and does raise the reader's stratum.\n");
    output
        .push_str("#          None of them are authored predicates; all are listed, not hidden,\n");
    output.push_str("#          because a dump that silently drops nodes is how re-derivations\n");
    output.push_str("#          come to disagree with the engine in the first place.\n");
    output.push_str("# order:   rows by predicate, edges by target — stable across runs.\n");
    let _ = writeln!(
        output,
        "# totals:  {} predicates, strata 0..{max_stratum}, {base} base, {} derived",
        rows.len(),
        rows.len() - base
    );
    for row in rows {
        let edges = row
            .edges
            .iter()
            .map(|edge| format!("{}{}", if edge.negative { '-' } else { '+' }, edge.to))
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(
            output,
            "{}\t{}\t{}\t{}",
            row.predicate,
            row.stratum,
            if row.base { "base" } else { "derived" },
            edges
        );
    }
    (output, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: &str = "person(Ara).\nchoose(Electorate, Gia).\n\
                      all $a: choose(Electorate, $a) -> permits(Review, $a).\n";

    fn source<'a>(name: &'a str, text: &'a str) -> LoadedSource<'a> {
        LoadedSource::new(name, text)
    }

    fn run(text: &str) -> RunOutput {
        run_pin_files(&[], &[source("t.pins.nibli", text)], PinOptions::default())
    }

    #[test]
    fn aggregate_output_is_canonical_and_files_get_fresh_engines() {
        let knowledge_bases = [source("fixture.nibli", KB)];
        let first = source(
            "first.pins.nibli",
            "person(Bet).\n? person(Bet).\n# => TRUE\n:expect-pins 1\n",
        );
        let second = source(
            "second.pins.nibli",
            "? person(Bet).\n# => FALSE\n? permits(Review, Gia).\n# => TRUE\n:expect-pins 2\n",
        );

        let output = run_pin_files(&knowledge_bases, &[first, second], PinOptions::default());

        assert_eq!(output.exit_code, EXIT_OK, "{}", output.stderr);
        assert_eq!(output.pins, 3);
        assert_eq!(
            output.stdout,
            concat!(
                "  first.pins.nibli: 1 pins, 0 findings, 0 harness errors\n",
                "  second.pins.nibli: 2 pins, 0 findings, 0 harness errors\n",
                "nibli-pin: PASS — 3 pins\n",
            )
        );
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn prepared_patch_retracts_and_adds_without_mutating_the_base() {
        let knowledge_bases = [source("fixture.nibli", KB)];
        let prepared = PreparedPinEngine::new(&knowledge_bases);
        let patched_pin = source(
            "patched.pins.nibli",
            "? permits(Review, Gia).\n# => FALSE\n\
             ? permits(Review, Bet).\n# => TRUE\n:expect-pins 2\n",
        );
        let patched = prepared.run_patched_files(
            &["choose(Electorate, Gia)."],
            &["choose(Electorate, Bet)."],
            &[patched_pin],
            PinOptions::default(),
        );
        assert_eq!(patched.exit_code, EXIT_OK, "{}", patched.stderr);

        let original_pin = source(
            "original.pins.nibli",
            "? permits(Review, Gia).\n# => TRUE\n\
             ? permits(Review, Bet).\n# => FALSE\n:expect-pins 2\n",
        );
        let original = prepared.run_files(&[original_pin], PinOptions::default());
        assert_eq!(original.exit_code, EXIT_OK, "{}", original.stderr);
    }

    #[test]
    fn prepared_patch_counts_duplicate_lines_and_rejects_absent_deletions() {
        let prepared =
            PreparedPinEngine::new(&[source("fixture.nibli", "person(Ara).\nperson(Ara).\n")]);
        let pin = source(
            "duplicate.pins.nibli",
            "? person(Ara).\n# => FALSE\n:expect-pins 1\n",
        );
        let removed = prepared.run_patched_files(
            &["person(Ara).", "person(Ara)."],
            &[],
            &[pin],
            PinOptions::default(),
        );
        assert_eq!(removed.exit_code, EXIT_OK, "{}", removed.stderr);

        let absent = prepared.run_patched_files(
            &["person(Ara).", "person(Ara).", "person(Ara)."],
            &[],
            &[pin],
            PinOptions::default(),
        );
        assert_eq!(absent.exit_code, EXIT_HARNESS);
        assert!(absent.stderr.contains("deletion occurrence 3 is absent"));
    }

    #[test]
    fn prepared_patch_dumps_strata_without_mutating_the_base() {
        let prepared = PreparedPinEngine::new(&[source("fixture.nibli", KB)]);
        let original = prepared.dump_strata();
        assert_eq!(original.exit_code, EXIT_OK, "{}", original.stderr);

        let patched = prepared.dump_patched_strata(
            &["all $a: choose(Electorate, $a) -> permits(Review, $a)."],
            &["all $a: choose(Electorate, $a) -> approves(Review, $a)."],
        );
        assert_eq!(patched.exit_code, EXIT_OK, "{}", patched.stderr);
        assert_ne!(patched.stdout, original.stdout);
        assert!(patched.stdout.lines().any(|line| {
            line.starts_with("approves\t")
                && line.contains("\tderived\t")
                && line.contains("+choose")
        }));

        let restored = prepared.dump_strata();
        assert_eq!(restored.stdout, original.stdout);

        let absent = prepared.dump_patched_strata(&["person(Bet)."], &[]);
        assert_eq!(absent.exit_code, EXIT_HARNESS);
        assert!(
            absent
                .stderr
                .contains("patch deletion occurrence 1 is absent from the prepared source")
        );
        assert!(
            absent
                .stderr
                .ends_with("nibli-pin: HARNESS ERROR (exit 2) — dump not trustworthy\n")
        );
    }

    #[test]
    fn exit_taxonomy_matches_findings_harness_and_resolved_defects() {
        let finding = run("person(Ara).\n? person(Ara).\n# => FALSE\n:expect-pins 1\n");
        assert_eq!(finding.exit_code, EXIT_FINDING);
        assert!(finding.stderr.contains("1 FINDING(S) (exit 1)"));

        let harness = run("person(Ara).\n? person(Ara).\n# => TRUE\n");
        assert_eq!(harness.exit_code, EXIT_HARNESS);
        assert!(harness.stderr.contains("HARNESS ERROR (exit 2)"));

        let resolved = run("person(Ara).\n:defect \"repairing person\"\n\
             ? person(Ara).\n# => FALSE\n:expect-pins 1\n");
        assert_eq!(resolved.exit_code, EXIT_DEFECT_RESOLVED);
        assert!(resolved.stderr.contains("NO LONGER REPRODUCE (exit 3)"));
        assert!(resolved.stderr.contains("repairing person"));
    }

    #[test]
    fn accept_scoped_refuse_and_defect_match_production_semantics() {
        let output = run("person(Adam).\n\
             :accept-scoped\n\
             all $x: person($x) -> prisoner($x).\n\
             ? prisoner(Adam).\n# => FALSE\n\
             :defect \"missing wealth still reproduces\"\n\
             ? rich(Adam).\n# => FALSE\n\
             :refuse syntax /unknown predicate/\n\
             not_a_word(Adam).\n\
             :expect-pins 4\n");

        assert_eq!(output.exit_code, EXIT_OK, "{}", output.stderr);
        assert_eq!(output.pins, 4);
        assert_eq!(output.defects, 1);
        assert!(output.stdout.contains("4 pins (1 defects)"));
        assert!(
            output
                .stdout
                .contains("1 encode defects that still reproduce")
        );
    }

    #[test]
    fn broken_fixture_is_harness_failure_and_runs_no_pins() {
        let output = run_pin_files(
            &[source("broken.nibli", "person(Ara).\nnot_a_word(Bet).\n")],
            &[source(
                "t.pins.nibli",
                "? person(Ara).\n# => TRUE\n:expect-pins 1\n",
            )],
            PinOptions::default(),
        );

        assert_eq!(output.exit_code, EXIT_HARNESS);
        assert_eq!(output.pins, 0);
        assert!(output.findings.is_empty());
        assert!(
            output
                .harness
                .iter()
                .any(|error| error.contains("broken.nibli:2")
                    && error.contains("fixture line failed to load"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn require_is_gated_and_runs_in_the_requested_directory() {
        let closed = run(":require true\n:expect-pins 1\n");
        assert_eq!(closed.exit_code, EXIT_HARNESS);
        assert!(closed.stderr.contains("needs --allow-shell"));

        let temporary = tempfile::tempdir().expect("temporary directory");
        let command = format!(
            ":require test \"$(pwd)\" = {:?}\n:expect-pins 1\n",
            temporary.path().display().to_string()
        );
        let opened = run_pin_files(
            &[],
            &[source("require.pins.nibli", &command)],
            PinOptions {
                allow_shell: true,
                working_directory: Some(temporary.path()),
                cancellation: None,
            },
        );
        assert_eq!(opened.exit_code, EXIT_OK, "{}", opened.stderr);
    }

    #[test]
    fn pre_raised_cancellation_skips_fixture_and_pin_execution() {
        let cancellation = crate::scheduler::CancellationToken::new();
        assert!(cancellation.cancel());
        let prepared = PreparedPinEngine::new_cancellable(
            &[source(
                "fixture.nibli",
                "person(Ara).\nnot_a_corpus_word(Bet).\n",
            )],
            cancellation.flag(),
        );

        let output = prepared.run_files(
            &[source(
                "cancelled.pins.nibli",
                "? person(Ara).\n# => FALSE\n:expect-pins 1\n",
            )],
            PinOptions {
                cancellation: Some(&cancellation),
                ..PinOptions::default()
            },
        );

        assert_eq!(output.exit_code, EXIT_HARNESS);
        assert_eq!(output.pins, 0);
        assert!(output.findings.is_empty());
        assert!(output.resolved.is_empty());
        assert_eq!(output.harness, ["fixture loading cancelled"]);
        assert!(!output.stderr.contains("fixture line failed to load"));
    }

    #[test]
    fn fresh_cancel_flag_restores_reused_prepared_engine() {
        let cancelled = crate::scheduler::CancellationToken::new();
        let prepared = PreparedPinEngine::new_cancellable(
            &[source("fixture.nibli", "person(Ara).\n")],
            cancelled.flag(),
        );
        assert!(cancelled.cancel());

        let skipped = prepared.run_files(
            &[source(
                "cancelled.pins.nibli",
                "? person(Ara).\n# => TRUE\n:expect-pins 1\n",
            )],
            PinOptions {
                cancellation: Some(&cancelled),
                ..PinOptions::default()
            },
        );
        assert_eq!(skipped.exit_code, EXIT_HARNESS);
        assert_eq!(skipped.pins, 0);
        assert!(skipped.findings.is_empty());
        assert!(skipped.resolved.is_empty());
        assert!(
            skipped
                .harness
                .iter()
                .any(|message| message == "cancelled.pins.nibli: execution cancelled")
        );

        let fresh = crate::scheduler::CancellationToken::new();
        prepared.set_cancel_flag(fresh.flag());
        let resumed = prepared.run_files(
            &[source(
                "resumed.pins.nibli",
                "? person(Ara).\n# => TRUE\n:expect-pins 1\n",
            )],
            PinOptions {
                cancellation: Some(&fresh),
                ..PinOptions::default()
            },
        );
        assert_eq!(resumed.exit_code, EXIT_OK, "{}", resumed.stderr);
        assert_eq!(resumed.pins, 1);
        assert!(resumed.harness.is_empty());
        assert!(!fresh.is_cancelled());
    }

    #[cfg(unix)]
    #[test]
    fn cancelled_require_reaps_shell_and_terminates_its_process_group() {
        fn process_exists(pid: i32) -> bool {
            // SAFETY: signal zero performs no state change; the positive PID was
            // written by the child started in this test.
            if unsafe { libc::kill(pid, 0) } == 0 {
                return true;
            }
            std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
        }

        fn process_is_running(pid: i32) -> bool {
            #[cfg(target_os = "linux")]
            {
                let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
                    return false;
                };
                let Some((_, fields)) = stat.rsplit_once(") ") else {
                    return process_exists(pid);
                };
                return !matches!(fields.as_bytes().first(), Some(b'Z' | b'X'));
            }
            #[cfg(not(target_os = "linux"))]
            {
                process_exists(pid)
            }
        }

        fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
            let deadline = Instant::now() + timeout;
            while Instant::now() < deadline {
                if condition() {
                    return true;
                }
                thread::sleep(Duration::from_millis(5));
            }
            condition()
        }

        let temporary = tempfile::tempdir().expect("temporary directory");
        let working_directory = temporary.path().to_path_buf();
        let ready = working_directory.join("ready");
        let leader_pid = working_directory.join("leader.pid");
        let grandchild_pid = working_directory.join("grandchild.pid");
        let cancellation = crate::scheduler::CancellationToken::new();
        let worker_cancellation = cancellation.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let pin = ":require printf '%s\\n' \"$$\" > leader.pid; sleep 30 & \
                       printf '%s\\n' \"$!\" > grandchild.pid; : > ready; wait\n\
                       :expect-pins 1\n";
            let output = run_pin_files(
                &[],
                &[source("require-cancel.pins.nibli", pin)],
                PinOptions {
                    allow_shell: true,
                    working_directory: Some(&working_directory),
                    cancellation: Some(&worker_cancellation),
                },
            );
            let _ = sender.send(output);
        });

        assert!(
            wait_until(Duration::from_secs(5), || ready.is_file()),
            "shell precondition did not publish its ready marker"
        );
        let leader = std::fs::read_to_string(&leader_pid)
            .expect("read shell pid")
            .trim()
            .parse::<i32>()
            .expect("shell pid is numeric");
        let grandchild = std::fs::read_to_string(&grandchild_pid)
            .expect("read grandchild pid")
            .trim()
            .parse::<i32>()
            .expect("grandchild pid is numeric");
        assert!(leader > 0 && grandchild > 0 && leader != grandchild);
        // SAFETY: both processes are known-live children of the fixed command.
        assert_eq!(unsafe { libc::getpgid(leader) }, leader);
        // SAFETY: the recorded grandchild is still blocked in `sleep` here.
        assert_eq!(unsafe { libc::getpgid(grandchild) }, leader);

        assert!(cancellation.cancel());
        let output = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("cancelled shell precondition did not return");
        worker.join().expect("pin worker joins");

        assert_eq!(output.exit_code, EXIT_HARNESS);
        assert_eq!(output.pins, 1);
        assert!(output.findings.is_empty());
        assert!(output.resolved.is_empty());
        assert!(
            output
                .harness
                .iter()
                .any(|message| message.contains("cancelled; child terminated and reaped")),
            "{}",
            output.stderr
        );
        assert!(
            wait_until(Duration::from_secs(5), || !process_exists(leader)),
            "the direct shell child was not reaped"
        );
        assert!(
            wait_until(Duration::from_secs(5), || !process_is_running(grandchild)),
            "the shell grandchild survived process-group cancellation"
        );
    }

    #[test]
    fn one_way_declarations_cannot_pretend_to_be_scoped() {
        let output =
            run("person(Adam).\n:accept-scoped\nderived_only(\"prisoner\").\n:expect-pins 1\n");
        assert_eq!(output.exit_code, EXIT_HARNESS);
        assert!(output.stderr.contains("one-way by design"));
    }

    #[test]
    fn strata_dump_is_stable_sorted_and_marks_edge_polarity() {
        let knowledge_base = source(
            "strata.nibli",
            "person(Adam).\n\
             all $x: person($x) & ~home($x) -> prisoner($x).\n\
             all $x: prisoner($x) -> reward($x).\n",
        );

        let first = dump_strata(&[knowledge_base]);
        let second = dump_strata(&[knowledge_base]);
        assert_eq!(first.exit_code, EXIT_OK, "{}", first.stderr);
        assert_eq!(first.stdout, second.stdout);
        assert!(first.stdout.starts_with("# nibli-strata v1\n"));
        assert!(first.stdout.lines().any(|line| {
            line.starts_with("prisoner\t") && line.contains("\tderived\t") && line.contains("-home")
        }));
        assert!(first.stdout.lines().any(|line| {
            line.starts_with("reward\t")
                && line.contains("\tderived\t")
                && line.contains("+prisoner")
        }));

        let broken = dump_strata(&[source("bad.nibli", "? person(Adam).\n")]);
        assert_eq!(broken.exit_code, EXIT_HARNESS);
        assert!(
            broken
                .stderr
                .ends_with("nibli-pin: HARNESS ERROR (exit 2) — dump not trustworthy\n")
        );

        let names = first
            .stdout
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.split('\t').next().expect("predicate"))
            .collect::<Vec<_>>();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }

    #[cfg(unix)]
    #[test]
    fn live_sibling_runner_matches_exit_and_report_bytes() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sibling = manifest.join("../nibli/target/release/nibli-pin");
        if !sibling.is_file() {
            eprintln!(
                "skipping live parity check because {} is not built",
                sibling.display()
            );
            return;
        }

        let pin_source = "person(Adam).\n\
                          :accept-scoped\n\
                          all $x: person($x) -> prisoner($x).\n\
                          ? prisoner(Adam).\n# => FALSE\n\
                          :defect \"missing wealth still reproduces\"\n\
                          ? rich(Adam).\n# => FALSE\n\
                          :refuse syntax /unknown predicate/\n\
                          not_a_word(Adam).\n\
                          :expect-pins 4\n";
        let temporary = tempfile::tempdir().expect("temporary directory");
        let pin_path = temporary.path().join("parity.pins.nibli");
        std::fs::write(&pin_path, pin_source).expect("write parity pin");

        let sibling_output = Command::new(&sibling)
            .arg(&pin_path)
            .output()
            .expect("run sibling nibli-pin");
        let in_process = run_pin_files(
            &[],
            &[source("parity.pins.nibli", pin_source)],
            PinOptions::default(),
        );

        assert_eq!(
            sibling_output.status.code(),
            Some(i32::from(in_process.exit_code))
        );
        assert_eq!(sibling_output.stdout, in_process.stdout.as_bytes());
        assert_eq!(sibling_output.stderr, in_process.stderr.as_bytes());
        assert_eq!(in_process.pins, 4);
    }

    #[test]
    fn engine_accepts_a_fixture_as_one_compilation_unit() {
        let engine = nibli_engine::NibliEngine::new();
        engine
            .assert_text("person(Ara).\nperson(Bet).\n")
            .expect("multi-statement fixture");
        assert_eq!(
            nibli_engine::display_query_result(&engine.query_holds("person(Bet).").unwrap()),
            "TRUE"
        );
    }

    #[test]
    #[ignore = "manual live performance and isolation check"]
    fn prepared_live_constitution_is_reused_across_pin_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let constitution =
            std::fs::read_to_string(root.join("new-book-plans/constitution.nibli")).unwrap();
        let floor =
            std::fs::read_to_string(root.join("new-book-plans/rights-floor.pins.nibli")).unwrap();
        let chapter =
            std::fs::read_to_string(root.join("book-1/01-what-counts-as-evidence.pins.nibli"))
                .unwrap();
        let output = run_pin_files(
            &[source("constitution.nibli", &constitution)],
            &[
                source("rights-floor.pins.nibli", &floor),
                source("01-what-counts-as-evidence.pins.nibli", &chapter),
            ],
            PinOptions {
                allow_shell: true,
                working_directory: Some(root),
                cancellation: None,
            },
        );
        assert_eq!(
            output.exit_code, EXIT_OK,
            "{}{}",
            output.stdout, output.stderr
        );
        assert_eq!(output.pins, 140);
    }
}
