// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native execution of the repository-owned pin inventories.

use std::fs;
use std::path::Path;

use crate::checks::state_form;
use crate::cli::Error;
use crate::context::Context;
use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine, RunOutput};
use crate::scheduler::{ScheduleError, run_bounded};

const KB_PATH: &str = "new-book-plans/constitution.nibli";
const COUNTERFACTUAL_DIR: &str = "new-book-plans/counterfactual";

const LIVE_FAMILIES: [(&str, &str); 5] = [
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
        "delivery and receipt",
        "new-book-plans/delivery-receipt.pins.nibli",
    ),
    (
        "family and life-course",
        "new-book-plans/family-life-course.pins.nibli",
    ),
];

const COUNTERFACTUAL_SPECS: [(&str, usize, usize); 21] = [
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
    ("no-dead-conjuncts", 1, 1),
    ("no-delivery-independence", 1, 1),
    ("no-state-form-independent-current-review", 1, 1),
    ("no-obligations-independent-source-review", 35, 35),
    ("no-obligations-source", 35, 0),
    ("no-obligations-finding-reader", 1, 0),
    ("unguarded-pen", 0, 1),
    ("undelivered-marker", 0, 1),
];

#[derive(Debug)]
pub(crate) struct LiveReport {
    pub(crate) chapter_pins: usize,
    pub(crate) family_results: Vec<(String, usize)>,
}

#[derive(Debug)]
pub(crate) struct CounterfactualReport {
    pub(crate) executed: Vec<String>,
    pub(crate) delegated: Vec<String>,
}

#[derive(Debug)]
struct Artifact {
    path: String,
    source: String,
}

#[derive(Debug)]
pub(crate) struct LivePlan {
    artifacts: Vec<Artifact>,
    chapter_file_count: usize,
    declared_chapter_pins: usize,
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

pub(crate) fn prepare_live_families(context: &Context) -> Result<LivePlan, Error> {
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
    Ok(LivePlan {
        artifacts,
        chapter_file_count,
        declared_chapter_pins: declared,
    })
}

pub(crate) fn execute_live_families(
    context: &Context,
    engine: &PreparedPinEngine,
    plan: &LivePlan,
) -> Result<LiveReport, Error> {
    let artifacts = &plan.artifacts;
    let loaded = artifacts
        .iter()
        .map(|artifact| LoadedSource::new(&artifact.path, &artifact.source))
        .collect::<Vec<_>>();
    let output = engine.run_files(
        &loaded,
        PinOptions {
            allow_shell: true,
            working_directory: Some(context.root()),
        },
    );
    require_clean_output("live pin families", &output)?;

    let chapter_pins = output.files[..plan.chapter_file_count]
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
        .zip(&output.files[plan.chapter_file_count..])
        .map(|((label, _), file)| ((*label).to_owned(), file.pins))
        .collect();
    Ok(LiveReport {
        chapter_pins,
        family_results,
    })
}

pub(crate) fn run_live_families(
    context: &Context,
    engine: &PreparedPinEngine,
) -> Result<LiveReport, Error> {
    let plan = prepare_live_families(context)?;
    execute_live_families(context, engine, &plan)
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
    context: &Context,
    plan: CounterfactualPlan,
) -> Result<CounterfactualReport, Error> {
    let workers = std::env::var("RIGHTS_VERIFY_JOBS")
        .ok()
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| Error::usage("RIGHTS_VERIFY_JOBS must be a positive integer"))
        })
        .transpose()?
        .unwrap_or(4);
    if workers == 0 {
        return Err(Error::usage(
            "RIGHTS_VERIFY_JOBS must be a positive integer",
        ));
    }
    let root = context.root().to_path_buf();
    let executed = run_bounded(plan.tasks, workers, move |_, task, cancellation| {
        if cancellation.is_cancelled() {
            return Err(Error::new("counterfactual execution cancelled"));
        }
        let engine = PreparedPinEngine::new(&[LoadedSource::new(&task.kb_path, &task.kb)]);
        let loaded = task
            .pins
            .iter()
            .map(|pin| LoadedSource::new(&pin.path, &pin.source))
            .collect::<Vec<_>>();
        let output = engine.run_files(
            &loaded,
            PinOptions {
                allow_shell: false,
                working_directory: Some(&root),
            },
        );
        require_clean_output(&task.name, &output)?;
        Ok(task.name)
    })
    .map_err(|error| match error {
        ScheduleError::JobFailed { source, .. } => source,
        other => Error::new(format!("counterfactual scheduler: {other}")),
    })?;
    Ok(CounterfactualReport {
        executed,
        delegated: plan.delegated,
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
        source: fs::read_to_string(path)?,
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
mod tests {
    use super::*;

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
}
