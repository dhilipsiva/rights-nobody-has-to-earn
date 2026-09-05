// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native execution of the repository-owned pin inventories.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::checks::state_form;
use crate::cli::Error;
use crate::context::Context;
use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine, RunOutput};
use crate::scheduler::{
    CancellationToken, ScheduleError, ScheduleOptions, run_bounded_controlled,
    run_bounded_with_state_controlled,
};

const KB_PATH: &str = "new-book-plans/constitution.nibli";
const COUNTERFACTUAL_DIR: &str = "new-book-plans/counterfactual";

const LIVE_FAMILIES: [(&str, &str); 35] = [
    (
        "universal-standing",
        "new-book-plans/universal-standing.pins.nibli",
    ),
    (
        "liberty and ecological",
        "new-book-plans/liberty-environment.pins.nibli",
    ),
    (
        "substantive equality",
        "new-book-plans/substantive-equality.pins.nibli",
    ),
    (
        "economic, labour, property, and fiscal",
        "new-book-plans/economic-constitution.pins.nibli",
    ),
    (
        "economic FS-POW-061",
        "new-book-plans/economic-power-061.pins.nibli",
    ),
    (
        "economic FS-POW-062",
        "new-book-plans/economic-power-062.pins.nibli",
    ),
    (
        "economic FS-POW-063",
        "new-book-plans/economic-power-063.pins.nibli",
    ),
    (
        "economic FS-POW-064",
        "new-book-plans/economic-power-064.pins.nibli",
    ),
    (
        "economic FS-POW-065",
        "new-book-plans/economic-power-065.pins.nibli",
    ),
    (
        "economic FS-POW-066",
        "new-book-plans/economic-power-066.pins.nibli",
    ),
    (
        "economic FS-POW-067",
        "new-book-plans/economic-power-067.pins.nibli",
    ),
    (
        "economic FS-POW-068",
        "new-book-plans/economic-power-068.pins.nibli",
    ),
    (
        "economic FS-POW-069",
        "new-book-plans/economic-power-069.pins.nibli",
    ),
    (
        "economic FS-POW-070",
        "new-book-plans/economic-power-070.pins.nibli",
    ),
    (
        "economic FS-POW-071",
        "new-book-plans/economic-power-071.pins.nibli",
    ),
    (
        "economic FS-POW-072",
        "new-book-plans/economic-power-072.pins.nibli",
    ),
    (
        "economic FS-POW-073",
        "new-book-plans/economic-power-073.pins.nibli",
    ),
    (
        "economic FS-POW-074",
        "new-book-plans/economic-power-074.pins.nibli",
    ),
    (
        "economic FS-POW-075",
        "new-book-plans/economic-power-075.pins.nibli",
    ),
    (
        "economic FS-POW-076",
        "new-book-plans/economic-power-076.pins.nibli",
    ),
    (
        "economic FS-POW-077",
        "new-book-plans/economic-power-077.pins.nibli",
    ),
    (
        "economic FS-POW-078",
        "new-book-plans/economic-power-078.pins.nibli",
    ),
    (
        "economic FS-POW-079",
        "new-book-plans/economic-power-079.pins.nibli",
    ),
    (
        "economic FS-POW-080",
        "new-book-plans/economic-power-080.pins.nibli",
    ),
    (
        "economic FS-POW-081",
        "new-book-plans/economic-power-081.pins.nibli",
    ),
    (
        "economic FS-POW-082",
        "new-book-plans/economic-power-082.pins.nibli",
    ),
    (
        "economic FS-POW-083",
        "new-book-plans/economic-power-083.pins.nibli",
    ),
    (
        "economic FS-POW-084",
        "new-book-plans/economic-power-084.pins.nibli",
    ),
    (
        "economic FS-POW-085",
        "new-book-plans/economic-power-085.pins.nibli",
    ),
    (
        "economic FS-POW-086",
        "new-book-plans/economic-power-086.pins.nibli",
    ),
    (
        "economic FS-POW-087",
        "new-book-plans/economic-power-087.pins.nibli",
    ),
    (
        "economic FS-POW-088",
        "new-book-plans/economic-power-088.pins.nibli",
    ),
    (
        "delivery and receipt",
        "new-book-plans/delivery-receipt.pins.nibli",
    ),
    (
        "family and life-course",
        "new-book-plans/family-life-course.pins.nibli",
    ),
    (
        "income security and social insurance",
        "new-book-plans/income-security.pins.nibli",
    ),
];

const COUNTERFACTUAL_SPECS: [(&str, usize, usize); 58] = [
    ("no-person-line", 1, 0),
    ("no-public-court", 1, 0),
    ("no-choose-boss", 1, 0),
    ("no-first-contact-standing", 1, 0),
    ("no-environmental-right", 1, 0),
    ("no-class9-climate-axis", 1, 0),
    ("no-direct-equality", 1, 0),
    ("no-equality-data-wall", 1, 0),
    ("no-positive-measure-end", 1, 0),
    ("no-automatic-adulthood", 1, 0),
    ("no-family-confinement-wall", 1, 0),
    ("no-missing-kinship-independence", 1, 0),
    ("no-pregnancy-authority", 1, 0),
    ("no-economic-floor-gate", 1, 0),
    ("no-economic-data-wall", 1, 0),
    ("no-economic-work-freedom", 1, 0),
    ("no-economic-direct-effects", 145, 0),
    ("no-economic-carry-results", 6, 0),
    ("no-economic-power-duty-bridges", 171, 0),
    ("no-economic-independent-current-review-061", 2, 2),
    ("no-economic-independent-current-review-062", 2, 2),
    ("no-economic-independent-current-review-063", 2, 2),
    ("no-economic-independent-current-review-064", 2, 2),
    ("no-economic-independent-current-review-065", 2, 2),
    ("no-economic-independent-current-review-066", 2, 2),
    ("no-economic-independent-current-review-067", 2, 2),
    ("no-economic-independent-current-review-068", 2, 2),
    ("no-economic-independent-current-review-069", 2, 2),
    ("no-economic-independent-current-review-070", 2, 2),
    ("no-economic-independent-current-review-071", 2, 2),
    ("no-economic-independent-current-review-072", 2, 2),
    ("no-economic-independent-current-review-073", 2, 2),
    ("no-economic-independent-current-review-074", 2, 2),
    ("no-economic-independent-current-review-075", 2, 2),
    ("no-economic-independent-current-review-076", 2, 2),
    ("no-economic-independent-current-review-077", 2, 2),
    ("no-economic-independent-current-review-078", 2, 2),
    ("no-economic-independent-current-review-079", 2, 2),
    ("no-economic-independent-current-review-080", 2, 2),
    ("no-economic-independent-current-review-081", 2, 2),
    ("no-economic-independent-current-review-082", 2, 2),
    ("no-economic-independent-current-review-083", 2, 2),
    ("no-economic-independent-current-review-084", 2, 2),
    ("no-economic-independent-current-review-085", 2, 2),
    ("no-economic-independent-current-review-086", 2, 2),
    ("no-economic-independent-current-review-087", 2, 2),
    ("no-economic-independent-current-review-088", 2, 2),
    ("no-dead-conjuncts", 1, 1),
    ("no-delivery-independence", 1, 1),
    ("no-state-form-independent-current-review", 1, 1),
    ("no-obligations-independent-source-review", 35, 35),
    ("no-obligations-source", 35, 0),
    ("no-obligations-finding-reader", 1, 0),
    ("unguarded-pen", 0, 1),
    ("undelivered-marker", 0, 1),
    ("no-income-supplement-rule", 1, 0),
    ("no-income-adjudicator-independence", 1, 1),
    ("unguarded-contribution-reader", 0, 1),
];

#[derive(Debug)]
pub(crate) struct LiveReport {
    pub(crate) chapter_pins: usize,
    pub(crate) family_results: Vec<(String, usize)>,
    /// Measured wall time per pin file, in execution order. Diagnostics only.
    pub(crate) file_timings: Vec<(String, u64)>,
}

#[derive(Debug)]
pub(crate) struct CounterfactualReport {
    pub(crate) executed: Vec<String>,
    pub(crate) delegated: Vec<String>,
    /// Measured wall time per counterfactual suite, in canonical order.
    /// Diagnostics only.
    pub(crate) timings: Vec<(String, u64)>,
}

#[derive(Clone, Debug)]
struct Artifact {
    path: String,
    source: Arc<str>,
}

#[derive(Debug)]
pub(crate) struct LivePlan {
    constitution: Arc<str>,
    working_tree: CapturedWorkingTree,
    artifacts: Vec<Artifact>,
    chapter_file_count: usize,
    declared_chapter_pins: usize,
}

#[derive(Debug)]
struct CapturedWorkingTree {
    temporary: tempfile::TempDir,
}

impl CapturedWorkingTree {
    fn new(constitution: &str) -> Result<Self, Error> {
        let temporary = tempfile::Builder::new()
            .prefix("rights-verify-live-pins-")
            .tempdir()?;
        let captured = Self { temporary };
        let source_dir = captured.root().join("new-book-plans");
        fs::create_dir_all(&source_dir)?;
        let constitution_path = source_dir.join("constitution.nibli");
        fs::write(&constitution_path, constitution.as_bytes())?;
        make_snapshot_read_only(&constitution_path, false)?;
        make_snapshot_read_only(&source_dir, true)?;
        make_snapshot_read_only(captured.root(), true)?;
        Ok(captured)
    }

    fn root(&self) -> &Path {
        self.temporary.path()
    }
}

impl Drop for CapturedWorkingTree {
    fn drop(&mut self) {
        let source_dir = self.temporary.path().join("new-book-plans");
        let _ = make_snapshot_writable(self.temporary.path(), true);
        let _ = make_snapshot_writable(&source_dir, true);
        let _ = make_snapshot_writable(&source_dir.join("constitution.nibli"), false);
    }
}

#[cfg(unix)]
fn make_snapshot_read_only(path: &Path, directory: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if directory { 0o500 } else { 0o400 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn make_snapshot_read_only(path: &Path, _directory: bool) -> std::io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

#[cfg(unix)]
fn make_snapshot_writable(path: &Path, directory: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if directory { 0o700 } else { 0o600 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn make_snapshot_writable(path: &Path, _directory: bool) -> std::io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions)
}

#[derive(Debug)]
struct CounterfactualTask {
    name: String,
    kb_path: String,
    kb: String,
    pins: Vec<Artifact>,
}

#[derive(Debug)]
pub(crate) struct CounterfactualPlan {
    tasks: Vec<CounterfactualTask>,
    delegated: Vec<String>,
}

/// Load the canonical constitution and prepare it once for the whole verifier.
pub(crate) fn prepare_live_engine(context: &Context) -> Result<(String, PreparedPinEngine), Error> {
    let source = context.read(KB_PATH)?;
    let engine = PreparedPinEngine::new(&[LoadedSource::new(KB_PATH, &source)]);
    Ok((source, engine))
}

pub(crate) fn prepare_live_families_with_constitution(
    context: &Context,
    constitution: Arc<str>,
) -> Result<LivePlan, Error> {
    let mut artifacts = Vec::new();
    artifacts.push(load(context, "new-book-plans/rights-floor.pins.nibli")?);
    let mut chapters = fs::read_dir(context.path("book-1"))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()?;
    chapters.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".pins.nibli"))
    });
    chapters.sort();
    for path in chapters {
        artifacts.push(load_absolute(context, &path)?);
    }
    let chapter_file_count = artifacts.len();
    let declared = artifacts
        .iter()
        .map(|artifact| expected_pins(&artifact.path, &artifact.source))
        .sum::<Result<usize, Error>>()?;

    for (_, path) in LIVE_FAMILIES {
        artifacts.push(load(context, path)?);
    }
    let working_tree = CapturedWorkingTree::new(&constitution)?;
    Ok(LivePlan {
        constitution,
        working_tree,
        artifacts,
        chapter_file_count,
        declared_chapter_pins: declared,
    })
}

pub(crate) fn execute_live_families_with_allocation(
    plan: &LivePlan,
    workers: usize,
    cancellation: CancellationToken,
) -> Result<LiveReport, Error> {
    execute_live_families_inner(plan, workers, ScheduleOptions::cancelled_by(cancellation))
}

fn execute_live_families_inner(
    plan: &LivePlan,
    workers: usize,
    options: ScheduleOptions,
) -> Result<LiveReport, Error> {
    let root = plan.working_tree.root().to_path_buf();
    let constitution = Arc::clone(&plan.constitution);
    let outputs = run_bounded_with_state_controlled(
        plan.artifacts.clone(),
        workers,
        options,
        |_| None::<PreparedPinEngine>,
        move |_, engine, artifact, cancellation| {
            let prepared = engine.get_or_insert_with(|| {
                PreparedPinEngine::new_cancellable(
                    &[LoadedSource::new(KB_PATH, &constitution)],
                    cancellation.flag(),
                )
            });
            prepared.set_cancel_flag(cancellation.flag());
            let output = prepared.run_files(
                &[LoadedSource::new(&artifact.path, &artifact.source)],
                PinOptions {
                    allow_shell: true,
                    working_directory: Some(&root),
                    cancellation: Some(&cancellation),
                },
            );
            if cancellation.is_cancelled() {
                return Err(Error::new("live pin execution cancelled"));
            }
            require_clean_output("live pin families", &output)?;
            if output.files.len() != 1 {
                return Err(Error::new(format!(
                    "live pin family {} returned {} file reports, expected one",
                    artifact.path,
                    output.files.len()
                )));
            }
            Ok(output
                .files
                .into_iter()
                .next()
                .expect("one file was checked"))
        },
    )
    .map_err(|error| match error {
        ScheduleError::JobFailed { source, .. } => source,
        other => Error::new(format!("live pin scheduler: {other}")),
    })?;

    let chapter_pins = outputs[..plan.chapter_file_count]
        .iter()
        .map(|file| file.pins)
        .sum::<usize>();
    if chapter_pins != plan.declared_chapter_pins {
        return Err(Error::new(format!(
            "ran {chapter_pins} chapter/floor pins but the files declare {}",
            plan.declared_chapter_pins
        )));
    }
    let family_results = LIVE_FAMILIES
        .iter()
        .zip(&outputs[plan.chapter_file_count..])
        .map(|((label, _), file)| ((*label).to_owned(), file.pins))
        .collect();
    let file_timings = outputs
        .iter()
        .map(|file| (file.display_name.clone(), file.elapsed_ms))
        .collect();
    Ok(LiveReport {
        chapter_pins,
        family_results,
        file_timings,
    })
}

pub(crate) fn run_only(context: &Context, relative: &Path) -> Result<RunOutput, Error> {
    let display = relative.to_string_lossy().into_owned();
    if !display.ends_with(".pins.nibli") {
        return Err(Error::usage(format!(
            "not a pin file: {display}; expected a path ending .pins.nibli"
        )));
    }
    if matches!(
        display.as_str(),
        state_form::MAIN_PINS_PATH | state_form::COUNTERFACTUAL_PINS_PATH
    ) {
        let snapshot = state_form::load_snapshot(context)?;
        return state_form::execute_focused_pin(context, &display, &snapshot);
    }
    let pin = load(context, &display)?;
    let kb_path = if display.starts_with(&format!("{COUNTERFACTUAL_DIR}/")) {
        display
            .strip_suffix(".pins.nibli")
            .map(|value| format!("{value}.nibli"))
            .expect("suffix checked")
    } else {
        KB_PATH.to_owned()
    };
    let kb = load(context, &kb_path)?;
    let engine = PreparedPinEngine::new(&[LoadedSource::new(&kb.path, &kb.source)]);
    Ok(engine.run_files(
        &[LoadedSource::new(&pin.path, &pin.source)],
        PinOptions {
            allow_shell: true,
            working_directory: Some(context.root()),
            cancellation: None,
        },
    ))
}

/// Validate every persistent counterfactual's edit shape and execute the
/// ordinary fixtures. State-form and obligations are returned as delegated
/// because their checker-owned lossless projections execute those four rows.
pub(crate) fn prepare_counterfactuals(context: &Context) -> Result<CounterfactualPlan, Error> {
    let live = context.read(KB_PATH)?;
    let mut tasks = Vec::new();
    let mut delegated = Vec::new();
    for (name, expected_removed, expected_added) in COUNTERFACTUAL_SPECS {
        let kb_path = format!("{COUNTERFACTUAL_DIR}/{name}.nibli");
        let candidate = context.read(&kb_path)?;
        let (removed, added) = diff_shape(&live, &candidate);
        if (removed, added) != (expected_removed, expected_added) {
            return Err(Error::new(format!(
                "{name} does not differ from the constitution the way its class requires: \
                 {removed} removed, {added} added; expected {expected_removed} removed, \
                 {expected_added} added"
            )));
        }
        if matches!(
            name,
            "no-state-form-independent-current-review"
                | "no-obligations-independent-source-review"
                | "no-obligations-source"
                | "no-obligations-finding-reader"
        ) {
            delegated.push(name.to_owned());
            continue;
        }
        let pin_paths = if name == "no-dead-conjuncts" {
            vec![
                "book-1/05-voiding.pins.nibli".to_owned(),
                "book-1/04-the-shield.pins.nibli".to_owned(),
            ]
        } else {
            vec![format!("{COUNTERFACTUAL_DIR}/{name}.pins.nibli")]
        };
        let pins = pin_paths
            .iter()
            .map(|path| load(context, path))
            .collect::<Result<Vec<_>, _>>()?;
        tasks.push(CounterfactualTask {
            name: name.to_owned(),
            kb_path,
            kb: candidate,
            pins,
        });
    }
    Ok(CounterfactualPlan { tasks, delegated })
}

pub(crate) fn execute_counterfactuals(
    _context: &Context,
    plan: CounterfactualPlan,
) -> Result<CounterfactualReport, Error> {
    let workers = crate::scheduler::configured_workers()?;
    execute_counterfactuals_inner(plan, workers, ScheduleOptions::default())
}

pub(crate) fn execute_counterfactuals_with_allocation(
    _context: &Context,
    plan: CounterfactualPlan,
    workers: usize,
    cancellation: CancellationToken,
) -> Result<CounterfactualReport, Error> {
    execute_counterfactuals_inner(plan, workers, ScheduleOptions::cancelled_by(cancellation))
}

fn execute_counterfactuals_inner(
    plan: CounterfactualPlan,
    workers: usize,
    options: ScheduleOptions,
) -> Result<CounterfactualReport, Error> {
    let outcomes = run_bounded_controlled(
        plan.tasks,
        workers,
        options,
        move |_, task, cancellation| {
            if cancellation.is_cancelled() {
                return Err(Error::new("counterfactual execution cancelled"));
            }
            let started = std::time::Instant::now();
            let engine = PreparedPinEngine::new_cancellable(
                &[LoadedSource::new(&task.kb_path, &task.kb)],
                cancellation.flag(),
            );
            let loaded = task
                .pins
                .iter()
                .map(|pin| LoadedSource::new(&pin.path, &pin.source))
                .collect::<Vec<_>>();
            let output = engine.run_files(
                &loaded,
                PinOptions {
                    allow_shell: false,
                    working_directory: None,
                    cancellation: Some(&cancellation),
                },
            );
            if cancellation.is_cancelled() {
                return Err(Error::new("counterfactual execution cancelled"));
            }
            require_clean_output(&task.name, &output)?;
            let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            Ok((task.name, elapsed_ms))
        },
    )
    .map_err(|error| match error {
        ScheduleError::JobFailed { source, .. } => source,
        other => Error::new(format!("counterfactual scheduler: {other}")),
    })?;
    let executed = outcomes.iter().map(|(name, _)| name.clone()).collect();
    Ok(CounterfactualReport {
        executed,
        delegated: plan.delegated,
        timings: outcomes,
    })
}

pub(crate) fn run_counterfactuals(context: &Context) -> Result<CounterfactualReport, Error> {
    let plan = prepare_counterfactuals(context)?;
    execute_counterfactuals(context, plan)
}

fn load(context: &Context, relative: &str) -> Result<Artifact, Error> {
    let path = context.path(relative);
    load_absolute(context, &path)
}

fn load_absolute(context: &Context, path: &Path) -> Result<Artifact, Error> {
    let relative = path
        .strip_prefix(context.root())
        .map_err(|_| Error::new(format!("pin path escaped repository: {}", path.display())))?
        .to_string_lossy()
        .into_owned();
    Ok(Artifact {
        path: relative,
        source: Arc::from(fs::read_to_string(path)?),
    })
}

fn expected_pins(path: &str, source: &str) -> Result<usize, Error> {
    let values = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix(":expect-pins"))
        .map(|value| {
            value
                .trim()
                .parse::<usize>()
                .map_err(|_| Error::new(format!("{path}: :expect-pins needs an integer")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [value] => Ok(*value),
        [] => Err(Error::new(format!("{path}: no :expect-pins declaration"))),
        _ => Err(Error::new(format!(
            "{path}: multiple :expect-pins declarations"
        ))),
    }
}

fn require_clean_output(label: &str, output: &RunOutput) -> Result<(), Error> {
    if output.exit_code == 0 {
        return Ok(());
    }
    let detail = format!("{}{}", output.stdout, output.stderr);
    Err(Error::with_exit_code(
        format!("{label} failed\n{}", detail.trim_end()),
        output.exit_code,
    ))
}

/// Count removed and added lines in a shortest line edit script.
///
/// Keeping each line terminator in its token preserves GNU `diff`'s identity:
/// an otherwise identical final line with and without a newline is one changed
/// line, not a common line.
fn diff_shape(before: &str, after: &str) -> (usize, usize) {
    let left = before.split_inclusive('\n').collect::<Vec<_>>();
    let right = after.split_inclusive('\n').collect::<Vec<_>>();
    let mut row = vec![0usize; right.len() + 1];
    for left_line in &left {
        let mut diagonal = 0;
        for (column, right_line) in right.iter().enumerate() {
            let prior = row[column + 1];
            row[column + 1] = if left_line == right_line {
                diagonal + 1
            } else {
                row[column + 1].max(row[column])
            };
            diagonal = prior;
        }
    }
    let common = row[right.len()];
    (left.len() - common, right.len() - common)
}

#[cfg(test)]
pub(crate) fn synthetic_live_plan_for_suite() -> LivePlan {
    let constitution = Arc::<str>::from("person(Ara).\n");
    let working_tree =
        CapturedWorkingTree::new(&constitution).expect("captured synthetic working tree");
    let source = Arc::<str>::from("? person(Ara).\n# => TRUE\n:expect-pins 1\n");
    let artifacts = (0..=LIVE_FAMILIES.len())
        .map(|index| Artifact {
            path: format!("synthetic-{index}.pins.nibli"),
            source: Arc::clone(&source),
        })
        .collect();
    LivePlan {
        constitution,
        working_tree,
        artifacts,
        chapter_file_count: 1,
        declared_chapter_pins: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_live_plan(failing_index: Option<usize>) -> LivePlan {
        let passing = Arc::<str>::from("? person(Ara).\n# => TRUE\n:expect-pins 1\n");
        let failing = Arc::<str>::from("? person(Ara).\n# => FALSE\n:expect-pins 1\n");
        let constitution = Arc::<str>::from("person(Ara).\n");
        let working_tree =
            CapturedWorkingTree::new(&constitution).expect("captured synthetic working tree");
        let artifacts = (0..=LIVE_FAMILIES.len())
            .map(|index| Artifact {
                path: format!("synthetic-{index}.pins.nibli"),
                source: if failing_index == Some(index) {
                    Arc::clone(&failing)
                } else {
                    Arc::clone(&passing)
                },
            })
            .collect();
        LivePlan {
            constitution,
            working_tree,
            artifacts,
            chapter_file_count: 1,
            declared_chapter_pins: 1,
        }
    }

    fn semantic_live_report(report: LiveReport) -> (usize, Vec<(String, usize)>, Vec<String>) {
        (
            report.chapter_pins,
            report.family_results,
            report
                .file_timings
                .into_iter()
                .map(|(name, _)| name)
                .collect(),
        )
    }

    #[test]
    fn diff_shape_counts_deletions_additions_and_changes() {
        assert_eq!(diff_shape("a\nb\nc\n", "a\nc\n"), (1, 0));
        assert_eq!(diff_shape("a\nb\n", "a\nx\n"), (1, 1));
        assert_eq!(diff_shape("a\n", "a\nb\n"), (0, 1));
    }

    #[test]
    fn diff_shape_preserves_terminal_newline_identity() {
        assert_eq!(diff_shape("a\n", "a"), (1, 1));
        assert_eq!(diff_shape("a", "a\n"), (1, 1));
        assert_eq!(diff_shape("a\nb\n", "a\nb"), (1, 1));
        assert_eq!(diff_shape("", ""), (0, 0));
    }

    #[test]
    fn current_counterfactual_shapes_match_the_contract() {
        let context = Context::discover().expect("repository");
        let live = context.read(KB_PATH).expect("constitution");
        for (name, removed, added) in COUNTERFACTUAL_SPECS {
            let candidate = context
                .read(format!("{COUNTERFACTUAL_DIR}/{name}.nibli"))
                .expect("counterfactual");
            assert_eq!(diff_shape(&live, &candidate), (removed, added), "{name}");
        }
    }

    #[test]
    fn captured_counterfactual_executes_without_a_working_directory() {
        let plan = CounterfactualPlan {
            tasks: vec![CounterfactualTask {
                name: "captured-counterfactual".to_owned(),
                kb_path: "captured-counterfactual.nibli".to_owned(),
                kb: "person(Ara).\n".to_owned(),
                pins: vec![Artifact {
                    path: "captured-counterfactual.pins.nibli".to_owned(),
                    source: Arc::from("? person(Ara).\n# => TRUE\n:expect-pins 1\n"),
                }],
            }],
            delegated: Vec::new(),
        };
        let report = execute_counterfactuals_inner(plan, 1, ScheduleOptions::default())
            .expect("captured counterfactual plan");
        assert_eq!(report.executed, ["captured-counterfactual"]);
    }

    #[test]
    fn live_inventory_has_one_floor_and_all_numbered_chapters() {
        let context = Context::discover().expect("repository");
        let floor = load(&context, "new-book-plans/rights-floor.pins.nibli").unwrap();
        assert_eq!(expected_pins(&floor.path, &floor.source).unwrap(), 100);
        let chapters = fs::read_dir(context.path("book-1"))
            .unwrap()
            .flatten()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.ends_with(".pins.nibli"))
            })
            .count();
        assert_eq!(chapters, 14);
    }

    #[test]
    fn live_family_semantics_match_for_every_supported_worker_count() {
        let baseline = semantic_live_report(
            execute_live_families_with_allocation(
                &synthetic_live_plan(None),
                1,
                CancellationToken::new(),
            )
            .expect("single-worker live plan"),
        );
        for workers in 2..=crate::scheduler::MAX_WORKERS {
            let parallel = semantic_live_report(
                execute_live_families_with_allocation(
                    &synthetic_live_plan(None),
                    workers,
                    CancellationToken::new(),
                )
                .expect("parallel live plan"),
            );
            assert_eq!(parallel, baseline, "worker count {workers}");
        }
    }

    #[test]
    fn live_require_uses_the_captured_constitution_tree() {
        let mut plan = synthetic_live_plan(None);
        plan.artifacts[0].source = Arc::from(
            ":require grep -Fx 'person(Ara).' new-book-plans/constitution.nibli\n\
             ? person(Ara).\n\
             # => TRUE\n\
             :expect-pins 2\n",
        );
        plan.declared_chapter_pins = 2;

        let report = execute_live_families_with_allocation(&plan, 1, CancellationToken::new())
            .expect("captured :require precondition");
        assert_eq!(report.chapter_pins, 2);
    }

    #[test]
    fn live_family_failure_and_cancellation_match_every_worker_count() {
        let mut failures = Vec::new();
        let mut cancellations = Vec::new();
        for workers in 1..=crate::scheduler::MAX_WORKERS {
            failures.push(
                execute_live_families_with_allocation(
                    &synthetic_live_plan(Some(2)),
                    workers,
                    CancellationToken::new(),
                )
                .expect_err("watched live finding must fail")
                .to_string(),
            );
            let cancellation = CancellationToken::new();
            cancellation.cancel();
            cancellations.push(
                execute_live_families_with_allocation(
                    &synthetic_live_plan(None),
                    workers,
                    cancellation,
                )
                .expect_err("pre-raised cancellation must stop live work")
                .to_string(),
            );
        }
        assert!(failures.iter().all(|failure| failure == &failures[0]));
        assert!(failures[0].contains("synthetic-2.pins.nibli"));
        assert!(
            cancellations
                .iter()
                .all(|cancellation| cancellation == &cancellations[0])
        );
    }
}
