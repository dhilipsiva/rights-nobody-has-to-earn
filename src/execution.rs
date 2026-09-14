// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cli::{Args, Error};
use crate::context::Context;
use crate::pin::{
    CompiledSource, LoadedSource, PinCaseGroup, PinOptions, PreparedPinEngine, RunOutput,
};
use crate::scheduler::{self, CancellationToken};

const INVENTORY: &str = "tests/pins/suites.json";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub(crate) struct Edit {
    pub(crate) before: String,
    pub(crate) after: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Base {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) base: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) edits: Vec<Edit>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    pub(crate) id: String,
    pub(crate) base: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) fixtures: Vec<String>,
    pub(crate) pins: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) edits: Vec<Edit>,
    #[serde(default = "yes")]
    pub(crate) scan: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) allow_shell: bool,
}

impl Case {
    fn preparation_key(&self) -> Option<(&str, &[Edit])> {
        // The live model is already retained separately on every worker. Only
        // variants need affinity; fixtures and pin files still run in isolation.
        (self.base != "live" || !self.edits.is_empty())
            .then_some((self.base.as_str(), self.edits.as_slice()))
    }
}

// Only consecutive cases are combined: original case order and first-failure
// priority remain unchanged. A prefix is data from the captured input set,
// not an alternative constitution or a persisted verification result.
fn fixture_batches(cases: &[Case]) -> Vec<&[Case]> {
    let mut result = Vec::new();
    let mut start = 0;
    while start < cases.len() {
        let first = &cases[start];
        let mut stop = start + 1;
        if first.fixtures.len() > 1 {
            while stop < cases.len() {
                let next = &cases[stop];
                if next.base != first.base
                    || next.edits != first.edits
                    || next.scan != first.scan
                    || next.fixtures.len() < 2
                    || next.fixtures[0] != first.fixtures[0]
                {
                    break;
                }
                stop += 1;
            }
        }
        result.push(&cases[start..stop]);
        start = stop;
    }
    result
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Inventory {
    pub(crate) bases: BTreeMap<String, Base>,
    pub(crate) cases: Vec<Case>,
}

struct Inputs {
    inventory: Inventory,
    files: BTreeMap<String, Arc<str>>,
}

fn local_path(path: &str) -> Result<&Path, Error> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err(Error::usage(format!(
            "test input must be repository-relative: {}",
            path.display()
        )));
    }
    Ok(path)
}

impl Inputs {
    fn load(context: &Context, only: Option<&str>) -> Result<Self, Error> {
        let mut inventory: Inventory = serde_json::from_str(&context.read(INVENTORY)?)?;
        if inventory.cases.is_empty() {
            return Err(Error::usage("pin inventory is empty"));
        }
        let mut names = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for case in &inventory.cases {
            if case.id.is_empty() || !names.insert(&case.id) || case.pins.is_empty() {
                return Err(Error::usage(format!(
                    "empty/duplicate case ID or missing pins: {:?}",
                    case.id
                )));
            }
            if !inventory.bases.contains_key(&case.base) {
                return Err(Error::usage(format!(
                    "{}: unknown base {}",
                    case.id, case.base
                )));
            }
        }
        for (name, base) in &inventory.bases {
            if base.path.is_some() == base.base.is_some() {
                return Err(Error::usage(format!(
                    "base {name}: specify exactly one of path or base"
                )));
            }
        }
        if let Some(path) = only {
            inventory
                .cases
                .retain(|case| case.pins.iter().any(|pin| pin == path));
            for case in &mut inventory.cases {
                case.pins.retain(|pin| pin == path);
            }
        }
        if inventory.cases.is_empty() {
            return Err(Error::usage(format!(
                "pin file is not in {INVENTORY}: {}",
                only.unwrap_or_default()
            )));
        }
        let mut inputs = Self {
            inventory,
            files: BTreeMap::new(),
        };
        for case in &inputs.inventory.cases {
            inputs.check_base_chain(&case.base, &mut BTreeSet::new())?;
            paths.extend(case.fixtures.iter().chain(&case.pins).cloned());
            let mut name = case.base.as_str();
            loop {
                let base = &inputs.inventory.bases[name];
                if let Some(path) = &base.path {
                    paths.insert(path.clone());
                    break;
                }
                name = base.base.as_deref().expect("validated base chain");
            }
        }
        let mut files = BTreeMap::new();
        for path in paths {
            let content = context
                .read(local_path(&path)?)
                .map_err(|error| Error::usage(format!("{path}: {error}")))?;
            files.insert(path, Arc::from(content));
        }
        inputs.files = files;
        Ok(inputs)
    }

    fn check_base_chain<'a>(
        &'a self,
        name: &'a str,
        seen: &mut BTreeSet<&'a str>,
    ) -> Result<(), Error> {
        if !seen.insert(name) {
            return Err(Error::usage(format!("cyclic test base: {name}")));
        }
        let base = self
            .inventory
            .bases
            .get(name)
            .ok_or_else(|| Error::usage(format!("unknown test base: {name}")))?;
        if let Some(parent) = &base.base {
            self.check_base_chain(parent, seen)?;
        }
        seen.remove(name);
        Ok(())
    }

    fn base_source(&self, name: &str) -> Result<Arc<str>, Error> {
        let base = self
            .inventory
            .bases
            .get(name)
            .ok_or_else(|| Error::usage(format!("unknown test base: {name}")))?;
        let source = if let Some(path) = &base.path {
            Arc::clone(&self.files[path])
        } else {
            self.base_source(base.base.as_deref().expect("validated base"))?
        };
        if base.edits.is_empty() {
            Ok(source)
        } else {
            Ok(Arc::from(apply_edits(&source, &base.edits, name)?))
        }
    }

    fn sources<'a>(&'a self, paths: &'a [String]) -> Vec<LoadedSource<'a>> {
        paths
            .iter()
            .map(|path| LoadedSource::new(path, &self.files[path]))
            .collect()
    }
}

/// Edits are exact semantic statements, independent of blank lines and comments.
/// Accept raw blocks as well so existing authoring generators can supply them.
pub(crate) fn apply_edits(source: &str, edits: &[Edit], name: &str) -> Result<String, Error> {
    let mut output = source.to_owned();
    for (index, edit) in edits.iter().enumerate() {
        if edit.before.is_empty() {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(&edit.after);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            continue;
        }
        // A comment-only authoring edit must not stale a counterfactual.
        let normalized = statements(&output);
        let before = statements(&edit.before);
        let after = statements(&edit.after);
        let matches: Vec<_> = if before.is_empty() {
            Vec::new()
        } else {
            normalized
                .windows(before.len())
                .enumerate()
                .filter_map(|(i, window)| (window == before).then_some(i))
                .collect()
        };
        if matches.len() != 1 {
            return Err(Error::usage(format!(
                "{name}: edit {} matches {} statement blocks; expected exactly one",
                index + 1,
                matches.len()
            )));
        }
        let start = matches[0];
        let mut updated = normalized[..start].to_vec();
        updated.extend(after);
        updated.extend_from_slice(&normalized[start + before.len()..]);
        output = updated.join("\n") + "\n";
    }
    Ok(output)
}

fn statements(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

#[derive(Default)]
struct Worker {
    canonical: Option<PreparedPinEngine>,
    variant: Option<(String, Vec<Edit>, PreparedPinEngine)>,
}

impl Worker {
    fn engine<'a>(
        &'a mut self,
        inputs: &Inputs,
        case: &Case,
        cancel: &CancellationToken,
        compiled: &CompiledSource,
    ) -> Result<&'a PreparedPinEngine, Error> {
        if case.base == "live" && case.edits.is_empty() {
            if self.canonical.is_none() {
                let source = inputs.base_source("live")?;
                self.canonical = Some(PreparedPinEngine::new_cached(
                    &[LoadedSource::new(
                        "new-book-plans/constitution.nibli",
                        &source,
                    )],
                    cancel.flag(),
                    compiled,
                ));
            }
            let engine = self.canonical.as_ref().expect("prepared live base");
            engine.set_cancel_flag(cancel.flag());
            return Ok(engine);
        }
        if !self
            .variant
            .as_ref()
            .is_some_and(|(base, edits, _)| base == &case.base && edits == &case.edits)
        {
            // Bound retained variants to one per worker rather than retaining a
            // full engine for every counterfactual in the inventory.
            self.variant = None;
            let source = inputs.base_source(&case.base)?;
            let source = apply_edits(&source, &case.edits, &case.id)?;
            self.variant = Some((
                case.base.clone(),
                case.edits.clone(),
                PreparedPinEngine::new_cached(
                    &[LoadedSource::new(&case.base, &source)],
                    cancel.flag(),
                    compiled,
                ),
            ));
        }
        let engine = &self.variant.as_ref().expect("prepared variant").2;
        engine.set_cancel_flag(cancel.flag());
        Ok(engine)
    }
}

struct Progress {
    finished: AtomicUsize,
    last: Mutex<Instant>,
    total: usize,
    started: Instant,
}

impl Progress {
    fn completed(&self, name: &str) {
        let finished = self.finished.fetch_add(1, Ordering::Relaxed) + 1;
        let mut last = self.last.lock().expect("progress lock");
        if last.elapsed() >= Duration::from_secs(10) || finished == self.total {
            eprintln!(
                "{finished}/{} cases finished ({:.1}s); latest: {name}",
                self.total,
                self.started.elapsed().as_secs_f64()
            );
            *last = Instant::now();
        }
    }
}

pub(crate) fn run(args: Args) -> Result<(), Error> {
    let started = Instant::now();
    let context = Context::discover()?;
    let only = args.only.as_ref().map(|path| {
        path.strip_prefix(context.root())
            .unwrap_or(path)
            .to_string_lossy()
            .trim_start_matches("./")
            .to_owned()
    });
    let inputs = Inputs::load(&context, only.as_deref())?;
    let cases = &inputs.inventory.cases;
    if args.list {
        for case in cases {
            println!("{}\t{}\t{}", case.id, case.base, case.pins.join(", "));
        }
        println!("{} substantive cases", cases.len());
        return Ok(());
    }
    let workers = scheduler::configured_workers()?;
    let compiled = inputs
        .files
        .get("new-book-plans/constitution.nibli")
        .map(|source| CompiledSource::new(source))
        .unwrap_or_default();
    eprintln!(
        "Running {} substantive cases with {workers} workers",
        cases.len()
    );
    let progress = Progress {
        finished: AtomicUsize::new(0),
        last: Mutex::new(Instant::now()),
        total: cases.len(),
        started,
    };
    let batches = fixture_batches(cases);
    let outcomes = scheduler::run_grouped(
        &batches,
        workers,
        Worker::default,
        |batch| batch[0].preparation_key(),
        |worker, batch, cancel| {
            let first = &batch[0];
            #[cfg(test)]
            crate::pin::profile_case_started(&first.id);
            #[cfg(test)]
            let prepare_started = Instant::now();
            let engine = worker.engine(&inputs, first, cancel, &compiled)?;
            #[cfg(test)]
            let preparation = prepare_started.elapsed();
            let run =
                |group: Option<&PinCaseGroup<'_>>, skip: usize| -> Result<Vec<RunOutput>, Error> {
                    let mut outcomes = Vec::new();
                    for case in batch.iter() {
                        #[cfg(test)]
                        crate::pin::profile_case_started(&case.id);
                        #[cfg(test)]
                        let run_started = Instant::now();
                        let fixtures = inputs.sources(&case.fixtures[skip..]);
                        let pins = inputs.sources(&case.pins);
                        let options = PinOptions {
                            allow_shell: case.allow_shell,
                            working_directory: Some(context.root()),
                            cancellation: Some(cancel),
                        };
                        let result = match group {
                            Some(group) => group.run_case(&fixtures, &pins, options, case.scan),
                            None => engine.run_case(&fixtures, &pins, options, case.scan),
                        };
                        #[cfg(test)]
                        crate::pin::profile_case_finished(
                            &case.id,
                            if outcomes.is_empty() {
                                preparation
                            } else {
                                Duration::ZERO
                            },
                            run_started.elapsed(),
                        );
                        progress.completed(&case.id);
                        if result.exit_code != 0 {
                            return Err(Error::with_exit_code(
                                format!("{}\n{}{}", case.id, result.stdout, result.stderr),
                                result.exit_code,
                            ));
                        }
                        outcomes.push(result);
                    }
                    Ok(outcomes)
                };
            if batch.len() > 1 {
                engine
                    .with_fixture_prefix(
                        &inputs.sources(&first.fixtures[..1]),
                        batch.iter().any(|case| case.scan),
                        |group| run(Some(group), 1),
                    )
                    .map_err(|error| Error::new(format!("{} shared prefix: {error}", first.id)))?
            } else {
                run(None, 0)
            }
        },
    )?;
    let outcomes: Vec<_> = outcomes.into_iter().flatten().collect();
    let pins: usize = outcomes.iter().map(|result: &RunOutput| result.pins).sum();
    let defects: usize = outcomes.iter().map(|result| result.defects).sum();
    if args.only.is_some() {
        println!(
            "PARTIAL: {pins} pins pass across {} selected cases ({:.2}s). Run ./verify.sh for the complete book.",
            outcomes.len(),
            started.elapsed().as_secs_f64()
        );
    } else {
        println!(
            "{pins} pins pass across {} cases; contradiction checks complete with no findings ({:.2}s).",
            outcomes.len(),
            started.elapsed().as_secs_f64()
        );
    }
    if defects != 0 {
        println!("{defects} pins describe known defects that still reproduce.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_batches_preserve_case_order_boundaries_and_per_case_options() {
        let case = Case {
            id: "a".into(),
            base: "live".into(),
            fixtures: vec!["shared.nibli".into(), "a.nibli".into()],
            pins: vec!["a.pins.nibli".into()],
            edits: vec![],
            scan: true,
            allow_shell: false,
        };
        let mut second = case.clone();
        second.id = "b".into();
        second.allow_shell = true;
        second.fixtures[1] = "b.nibli".into();
        let mut third = case.clone();
        third.id = "c".into();
        third.scan = false;
        let mut fourth = case.clone();
        fourth.id = "d".into();
        let all = [case, second, third, fourth];
        let groups = fixture_batches(&all);
        assert_eq!(
            groups.iter().map(|group| group.len()).collect::<Vec<_>>(),
            [2, 1, 1]
        );
        assert_eq!(
            groups
                .iter()
                .flat_map(|group| group.iter().map(|case| case.id.as_str()))
                .collect::<Vec<_>>(),
            ["a", "b", "c", "d"]
        );
        assert!(!groups[0][0].allow_shell);
        assert!(groups[0][1].allow_shell);
        let mut changed = all[1].clone();
        changed.edits.push(Edit {
            before: "person(A).".into(),
            after: "person(B).".into(),
        });
        assert_eq!(fixture_batches(&[all[0].clone(), changed]).len(), 2);
        let mut changed = all[1].clone();
        changed.base = "variant".into();
        assert_eq!(fixture_batches(&[all[0].clone(), changed]).len(), 2);
        let mut changed = all[1].clone();
        changed.fixtures[0] = "different.nibli".into();
        assert_eq!(fixture_batches(&[all[0].clone(), changed]).len(), 2);
    }

    #[test]
    fn preparation_affinity_uses_exact_base_and_ordered_edits_only() {
        let mut case = Case {
            id: "case".into(),
            base: "live".into(),
            fixtures: Vec::new(),
            pins: vec!["expect.pins.nibli".into()],
            edits: Vec::new(),
            scan: true,
            allow_shell: false,
        };
        assert!(case.preparation_key().is_none());
        case.base = "variant".into();
        let mut sibling = case.clone();
        sibling.id = "sibling".into();
        sibling.fixtures.push("another-fixture.nibli".into());
        sibling.pins = vec!["another.pins.nibli".into()];
        assert_eq!(case.preparation_key(), sibling.preparation_key());
        sibling.base = "another-base".into();
        assert_ne!(case.preparation_key(), sibling.preparation_key());
        case.base = "live".into();
        case.edits = vec![
            Edit {
                before: "person(A).".into(),
                after: "person(B).".into(),
            },
            Edit {
                before: "person(B).".into(),
                after: "person(C).".into(),
            },
        ];
        assert!(case.preparation_key().is_some());
        sibling = case.clone();
        sibling.edits.reverse();
        assert_ne!(case.preparation_key(), sibling.preparation_key());
        sibling = case.clone();
        sibling.edits[0].after = "person(D).".into();
        assert_ne!(case.preparation_key(), sibling.preparation_key());
    }

    #[test]
    fn edits_never_match_comments_or_partial_statements() {
        for before in ["person(A).", "person(A).\n", "person(A)"] {
            assert!(
                apply_edits(
                    "# person(A).\nperson(B).\n",
                    &[Edit {
                        before: before.into(),
                        after: "person(C).".into(),
                    }],
                    "case"
                )
                .is_err()
            );
        }
        assert!(
            apply_edits(
                "person(A).",
                &[Edit {
                    before: "person(A)".into(),
                    after: "person(B)".into(),
                }],
                "case"
            )
            .is_err()
        );
    }

    #[test]
    fn focused_inputs_load_only_selected_pins_and_dependencies() {
        let directory = tempfile::tempdir().unwrap();
        let context = Context::from_test_root(directory.path().to_owned());
        std::fs::create_dir_all(context.path("tests/pins")).unwrap();
        std::fs::write(context.path("base.nibli"), "person(Ara).\n").unwrap();
        std::fs::write(
            context.path("wanted.pins.nibli"),
            ":expect-pins 1\n? person(Ara).\n# => TRUE\n",
        )
        .unwrap();
        let inventory = serde_json::json!({
            "bases": {"live": {"path": "base.nibli"}, "unused": {"path": "missing-base.nibli"}},
            "cases": [
                {"id": "paired", "base": "live", "pins": ["wanted.pins.nibli", "missing-paired.pins.nibli"]},
                {"id": "unrelated", "base": "unused", "pins": ["missing.pins.nibli"]}
            ]
        });
        std::fs::write(context.path(INVENTORY), inventory.to_string()).unwrap();
        let inputs = Inputs::load(&context, Some("wanted.pins.nibli")).unwrap();
        assert_eq!(inputs.inventory.cases.len(), 1);
        assert_eq!(inputs.inventory.cases[0].pins, ["wanted.pins.nibli"]);
        assert_eq!(inputs.files.len(), 2);
        assert!(Inputs::load(&context, None).is_err());
        assert!(Inputs::load(&context, Some("unlisted.pins.nibli")).is_err());
    }

    #[test]
    fn counterfactual_edits_survive_comment_changes_and_reject_ambiguity() {
        let edits = [Edit {
            before: "person(A).\n# old comment\nperson(B).".into(),
            after: "person(C).".into(),
        }];
        assert_eq!(
            apply_edits(
                "person(A).\n# revised comment\nperson(B).\n",
                &edits,
                "case"
            )
            .unwrap(),
            "person(C).\n"
        );
        assert!(
            apply_edits(
                "person(A).\nperson(B).\nperson(A).\nperson(B).",
                &edits,
                "case"
            )
            .is_err()
        );
        assert!(apply_edits("person(X).", &edits, "case").is_err());
    }

    #[test]
    fn append_and_replacement_preserve_order() {
        let edits = [
            Edit {
                before: "person(A).".into(),
                after: "person(B).".into(),
            },
            Edit {
                before: String::new(),
                after: "person(C).".into(),
            },
        ];
        assert_eq!(
            apply_edits("person(A).\n", &edits, "case").unwrap(),
            "person(B).\nperson(C).\n"
        );
    }
}
