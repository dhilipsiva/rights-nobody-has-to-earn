// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use crate::checks::{
    amendment, assertion_surface, assurance, claim_table, ledger, obligations, pilot, pins,
    placement, power_manifest, reader, red_team, registry, repository, spine, state_form, temporal,
};
use crate::cli::Error;
use crate::context::Context;
use crate::report::Reporter;
use crate::scheduler::{
    CancellationToken, DagFailure, DagFailureKind, DagJob, DagRun, ExecutionGraph,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunMode {
    Quick,
    Full,
}

impl RunMode {
    fn executes(self) -> bool {
        self == Self::Full
    }
}

enum FamilyJob {
    LivePins(pins::LivePlan),
    Obligations(obligations::ExecutionPlan),
    StateForm {
        snapshot: state_form::SourceSnapshot,
        validated: state_form::ValidatedStateForm,
    },
    #[cfg(test)]
    Synthetic(Box<dyn FnOnce(usize, CancellationToken) -> Result<FamilyOutput, Error> + Send>),
}

const LIVE_PHASE: &str = "chapter, floor, and family pins";
const OBLIGATIONS_PHASE: &str = "obligations executions";
const STATE_FORM_PHASE: &str = "state-form executions";

struct FamilyOutput {
    phase: &'static str,
    pass_lines: Vec<String>,
    details: Vec<(String, u64)>,
}

fn ensure_active(cancellation: &CancellationToken, phase: &str) -> Result<(), Error> {
    if cancellation.is_cancelled() {
        Err(Error::new(format!("{phase} cancelled")))
    } else {
        Ok(())
    }
}

fn execute_family_job(
    lane_allocation: usize,
    job: FamilyJob,
    cancellation: CancellationToken,
) -> Result<FamilyOutput, Error> {
    ensure_active(&cancellation, "executable verifier family")?;
    let output = match job {
        FamilyJob::LivePins(plan) => {
            let live = pins::execute_live_families_with_allocation(
                &plan,
                lane_allocation,
                cancellation.clone(),
            )?;
            let mut pass_lines = vec![format!("{} chapter/floor pins pass", live.chapter_pins)];
            pass_lines.extend(
                live.family_results
                    .into_iter()
                    .map(|(family, count)| format!("{family}: {count} pins pass")),
            );
            FamilyOutput {
                phase: LIVE_PHASE,
                pass_lines,
                details: live.file_timings,
            }
        }
        FamilyJob::Obligations(plan) => {
            let result = obligations::execute_plan_with_cancellation(&plan, cancellation.clone())?;
            FamilyOutput {
                phase: OBLIGATIONS_PHASE,
                pass_lines: vec![format!(
                    "obligations: PASS — {} isolated cases and {} counterfactual suites execute",
                    result.cases, result.counterfactual_suites
                )],
                details: Vec::new(),
            }
        }
        FamilyJob::StateForm {
            snapshot,
            validated,
        } => {
            let result = state_form::execute_validated_with_allocation(
                &snapshot,
                &validated,
                lane_allocation,
                cancellation.clone(),
            )?;
            FamilyOutput {
                phase: STATE_FORM_PHASE,
                pass_lines: vec![format!(
                    "state-form: PASS — {} main shards / {} pins and {} counterfactual shards / {} pins",
                    result.main_shards,
                    result.main_pins,
                    result.counterfactual_shards,
                    result.counterfactual_pins,
                )],
                details: result.shard_timings,
            }
        }
        #[cfg(test)]
        FamilyJob::Synthetic(run) => run(lane_allocation, cancellation.clone())?,
    };
    ensure_active(&cancellation, output.phase)?;
    Ok(output)
}

fn family_graph(
    lane_capacity: usize,
    live: pins::LivePlan,
    obligations: obligations::ExecutionPlan,
    state_snapshot: state_form::SourceSnapshot,
    validated_state_form: state_form::ValidatedStateForm,
) -> Result<ExecutionGraph<FamilyJob>, Error> {
    family_graph_from_jobs(
        lane_capacity,
        FamilyJob::LivePins(live),
        FamilyJob::Obligations(obligations),
        FamilyJob::StateForm {
            snapshot: state_snapshot,
            validated: validated_state_form,
        },
    )
}

fn family_graph_from_jobs(
    lane_capacity: usize,
    live: FamilyJob,
    obligations: FamilyJob,
    state_form: FamilyJob,
) -> Result<ExecutionGraph<FamilyJob>, Error> {
    let live_lanes = crate::scheduler::live_worker_allocation(lane_capacity);
    ExecutionGraph::derive(
        vec![
            DagJob::new(LIVE_PHASE, [], live_lanes, live),
            DagJob::new(OBLIGATIONS_PHASE, [], 1, obligations),
            DagJob::new(STATE_FORM_PHASE, [], 1, state_form),
        ],
        lane_capacity,
    )
    .map_err(|error| Error::new(format!("executable verifier graph: {error}")))
}

fn execution_details(run: &DagRun<FamilyOutput, Error>) -> Vec<(String, u64)> {
    let mut details = Vec::new();
    for result in &run.completed {
        details.push((result.id.to_owned(), result.timing.elapsed_ms));
        details.extend(
            result
                .value
                .details
                .iter()
                .map(|(name, elapsed)| (format!("{}/{}", result.id, name), *elapsed)),
        );
    }
    if let Some(failure) = &run.failure
        && let Some(id) = failure.id
    {
        details.push((format!("{id}/failed"), failure.timing.elapsed_ms));
    }
    details
}

fn failure_error(failure: DagFailure<Error>) -> Error {
    match failure.kind {
        DagFailureKind::Job(error) => error,
        DagFailureKind::TimedOut(timeout) => Error::new(format!(
            "{} exceeded its {:?} cooperative timeout",
            failure.id.unwrap_or("executable verifier family"),
            timeout
        )),
        DagFailureKind::Panicked(message) => Error::new(format!(
            "{} panicked: {message}",
            failure.id.unwrap_or("executable verifier family")
        )),
        DagFailureKind::LostWorker(active) => Error::new(format!(
            "executable verifier graph lost workers for {active:?}"
        )),
        DagFailureKind::Cancelled => Error::new("executable verifier graph cancelled"),
    }
}

/// Run the complete native verification pipeline in its required order.
///
/// Every checker is linked into this process. Reviewed JSON is decoded into
/// each checker's strict serde model before semantic validation begins.
pub(crate) fn run<W: Write>(context: &Context, mode: RunMode, output: W) -> Result<(), Error> {
    let mut report = Reporter::with_recorder(output, crate::diagnostics::observe());
    let execute = mode.executes();
    let lane_capacity = crate::scheduler::configured_workers()?;

    report.step("verification infrastructure")?;
    report.pass(crate::receipt::self_test()?)?;
    report.pass(crate::scheduler::self_test()?)?;
    report.pass(crate::refresh::self_test()?)?;
    report.pass(crate::diagnostics::self_test()?)?;

    report.step("engine")?;
    let (constitution, engine) = pins::prepare_live_engine(context)?;
    let strata = engine.dump_strata();
    if strata.exit_code != 0 {
        return Err(Error::new(format!(
            "in-process strata failed\n{}{}",
            strata.stdout, strata.stderr
        )));
    }
    report.pass("rights-verify embedded Nibli engine prepared")?;

    report.step("spine")?;
    report.pass(spine::run_with_strata(
        context,
        Path::new("new-book-plans/constitution.nibli"),
        Path::new("new-book-plans/3-spine.md"),
        &strata.stdout,
        true,
    )?)?;

    report.step("assertion surface")?;
    report.pass(assertion_surface::check(
        context,
        Some(&strata.stdout),
        Some(&constitution),
    )?)?;
    drop(strata);
    drop(engine);
    let constitution: Arc<str> = Arc::from(constitution);

    report.step("record-integrity assurance")?;
    report.pass(assurance::check(context)?.to_string())?;

    report.step("record-integrity red-team contract")?;
    report.pass(red_team::check(
        context,
        false,
        red_team::InputSnapshot::default(),
    )?)?;

    report.step("amendment-semantics contract")?;
    report.pass(amendment::check(context)?.to_string())?;

    report.step("placement-exhaustiveness contract")?;
    report.pass(placement::check(context)?.to_string())?;

    report.step("temporal-assurance contract")?;
    report.pass(temporal::check(context)?.to_string())?;

    report.step("state-form contract")?;
    let state_snapshot = state_form::load_snapshot(context)?;
    let validated_state_form = state_form::validate(&state_snapshot)?;
    report.pass(validated_state_form.report().to_string())?;

    report.step("obligations contract")?;
    let ledger_snapshot = ledger::load_and_validate(context)?;
    let obligation_snapshot = obligations::load_snapshot_with_ledger(context, &ledger_snapshot)?;
    let (obligation_report, obligation_execution_plan) = if execute {
        let (report, plan) =
            obligations::check_and_prepare_execution(context, &obligation_snapshot)?;
        (report, Some(plan))
    } else {
        (obligations::check(context, &obligation_snapshot)?, None)
    };
    report.pass(obligation_report.to_string())?;

    report.step("evidence vocabulary")?;
    report.pass(evidence_vocabulary(context)?)?;

    report.step("reader-evidence contract")?;
    report.pass(reader::check(
        context,
        false,
        reader::InputSnapshot::default(),
    )?)?;

    report.step("reader-evidence admission-gate component")?;
    report.pass(reader::admission_gate_self_test(context)?)?;

    report.step("pilot reader artifacts")?;
    report.pass(pilot::check(context)?.to_string())?;

    report.step("full-society power source manifest")?;
    report.pass(power_manifest::check(context)?.to_string())?;

    report.step(ledger::STEP_NAME)?;
    report.pass(ledger::check_validated(context, &ledger_snapshot)?.to_string())?;

    report.step(ledger::closure::STEP_NAME)?;
    report.pass(ledger::closure::check_validated(context, &ledger_snapshot)?.to_string())?;

    report.step("repository guards")?;
    for message in repository::check(context)?.messages {
        report.pass(message)?;
    }
    report.pass(obligations::check_consumers(context, &obligation_snapshot)?)?;

    report.step("claim table")?;
    report.pass(claim_table::check(context)?)?;

    report.step("registry")?;
    report.pass(registry::check(context)?)?;

    if execute {
        // Resolve the scheduler-owned executable plans while the suite is still
        // in its preflight phase. No Nibli query runs until all structural,
        // repository, claim-table, registry, and counterfactual-shape guards
        // above have passed. Receipt mode additionally rechecks the complete
        // staged manifest after every joined family has returned.
        let live_plan =
            pins::prepare_live_families_with_constitution(context, Arc::clone(&constitution))?;
        let counterfactual_plan = pins::prepare_counterfactuals(context)?;

        report.step("reader-evidence evaluator controls")?;
        report.pass(reader::check(
            context,
            true,
            reader::InputSnapshot::default(),
        )?)?;

        let graph = family_graph(
            lane_capacity,
            live_plan,
            obligation_execution_plan.expect("full mode prepares obligations execution"),
            state_snapshot.clone(),
            validated_state_form.clone(),
        )?;
        crate::diagnostics::begin_phase("executable verifier families");
        let run = crate::scheduler::run_dag(graph, None, |_, _, lanes, job, cancellation| {
            execute_family_job(lanes, job, cancellation)
        });
        crate::diagnostics::add_details(execution_details(&run));
        if run.maximum_active_weight > lane_capacity
            || run.maximum_active_jobs > lane_capacity
            || run.maximum_managed_thread_upper_bound > lane_capacity * 2
        {
            return Err(Error::new(
                "executable verifier graph exceeded its lane or managed-thread bound",
            ));
        }
        for result in run.completed {
            if result.id != result.value.phase {
                return Err(Error::new(format!(
                    "executable verifier graph returned {} for canonical phase {}",
                    result.value.phase, result.id
                )));
            }
            report.step_observed(result.value.phase)?;
            for line in result.value.pass_lines {
                report.pass(line)?;
            }
        }
        if let Some(failure) = run.failure {
            if let Some(phase) = failure.id {
                report.step_observed(phase)?;
            }
            return Err(failure_error(failure));
        }

        report.step("record-integrity red-team snapshots")?;
        report.pass(red_team::check(
            context,
            true,
            red_team::InputSnapshot::default(),
        )?)?;

        report.step("temporal-assurance executions")?;
        report.pass(temporal::check_execute(context)?.to_string())?;

        report.step("amendment-semantics executions")?;
        report.pass(amendment::check_execute(context)?.to_string())?;

        report.step("placement-exhaustiveness executions")?;
        report.pass(placement::check_execute(context)?.to_string())?;

        report.step("counterfactuals")?;
        let result = pins::execute_counterfactuals(context, counterfactual_plan)?;
        crate::diagnostics::add_details(result.timings);
        report.pass(format!(
            "{} counterfactual suites execute; {} checker-owned suites delegated",
            result.executed.len(),
            result.delegated.len()
        ))?;
    }

    let label = match mode {
        RunMode::Quick => "quick structural verification passed",
        RunMode::Full => "full verification passed",
    };
    report.line(format!("\n\x1b[32m{label}\x1b[0m"))?;
    report.flush()
}

fn evidence_vocabulary(context: &Context) -> Result<String, Error> {
    let spine = context.read("new-book-plans/3-spine.md")?;
    let marker = "Evidence predicates (";
    if spine.matches(marker).count() != 1 {
        return Err(Error::new(
            "spine must contain exactly one evidence-predicate count",
        ));
    }
    let start = spine
        .find(marker)
        .ok_or_else(|| Error::new("spine has no evidence-predicate count"))?
        + marker.len();
    let end = spine[start..]
        .find(')')
        .map(|offset| start + offset)
        .ok_or_else(|| Error::new("spine evidence-predicate count is malformed"))?;
    let count = spine[start..end]
        .parse::<usize>()
        .map_err(|_| Error::new("spine evidence-predicate count is not an integer"))?;
    if count != 42 {
        return Err(Error::new(format!(
            "evidence vocabulary is {count}, not 42; chapters 1, 3 and 5 must be re-read"
        )));
    }
    Ok("evidence vocabulary is 42".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    fn synthetic_pass(
        phase: &'static str,
        allocations: Arc<Mutex<Vec<(&'static str, usize)>>>,
    ) -> FamilyJob {
        FamilyJob::Synthetic(Box::new(move |lanes, cancellation| {
            allocations
                .lock()
                .expect("allocation observations")
                .push((phase, lanes));
            for _ in 0..20 {
                ensure_active(&cancellation, phase)?;
                thread::sleep(Duration::from_millis(1));
            }
            Ok(FamilyOutput {
                phase,
                pass_lines: vec![format!("{phase}: PASS")],
                details: Vec::new(),
            })
        }))
    }

    fn pass_graph(
        workers: usize,
    ) -> (
        DagRun<FamilyOutput, Error>,
        Arc<Mutex<Vec<(&'static str, usize)>>>,
    ) {
        let allocations = Arc::new(Mutex::new(Vec::new()));
        let graph = family_graph_from_jobs(
            workers,
            synthetic_pass(LIVE_PHASE, Arc::clone(&allocations)),
            synthetic_pass(OBLIGATIONS_PHASE, Arc::clone(&allocations)),
            synthetic_pass(STATE_FORM_PHASE, Arc::clone(&allocations)),
        )
        .expect("real family graph shape");
        let run = crate::scheduler::run_dag(graph, None, |_, _, lanes, job, cancellation| {
            execute_family_job(lanes, job, cancellation)
        });
        (run, allocations)
    }

    #[test]
    fn captured_production_adapters_are_equivalent_for_w1_through_w4() {
        let mut baseline = None;
        for workers in 1..=crate::scheduler::MAX_WORKERS {
            let (state_snapshot, validated_state_form) =
                state_form::synthetic_execution_fixture_for_suite();
            let graph = family_graph(
                workers,
                pins::synthetic_live_plan_for_suite(),
                obligations::synthetic_execution_plan_for_suite(),
                state_snapshot,
                validated_state_form,
            )
            .expect("captured production adapter graph");
            let run = crate::scheduler::run_dag(graph, None, |_, _, lanes, job, cancellation| {
                execute_family_job(lanes, job, cancellation)
            });
            assert!(run.failure.is_none(), "W{workers} unexpectedly failed");
            assert!(run.maximum_active_weight <= workers, "W{workers}");
            assert!(run.maximum_managed_thread_upper_bound <= workers * 2);
            let semantics = run
                .completed
                .into_iter()
                .map(|result| (result.id, result.value.phase, result.value.pass_lines))
                .collect::<Vec<_>>();
            if let Some(expected) = &baseline {
                assert_eq!(&semantics, expected, "W{workers}");
            } else {
                baseline = Some(semantics);
            }
        }
    }

    #[test]
    fn real_family_graph_is_canonical_and_bounded_for_w1_through_w4() {
        let expected_semantics = vec![
            (LIVE_PHASE, LIVE_PHASE, format!("{LIVE_PHASE}: PASS")),
            (
                OBLIGATIONS_PHASE,
                OBLIGATIONS_PHASE,
                format!("{OBLIGATIONS_PHASE}: PASS"),
            ),
            (
                STATE_FORM_PHASE,
                STATE_FORM_PHASE,
                format!("{STATE_FORM_PHASE}: PASS"),
            ),
        ];

        for workers in 1..=crate::scheduler::MAX_WORKERS {
            let (run, allocations) = pass_graph(workers);
            assert!(run.failure.is_none(), "W{workers} unexpectedly failed");
            assert_eq!(run.maximum_active_weight, workers, "W{workers}");
            assert_eq!(
                run.maximum_active_jobs,
                if workers == 1 { 1 } else { 2 },
                "W{workers} independent-job overlap"
            );
            assert!(run.maximum_managed_thread_upper_bound <= workers * 2);

            let semantics = run
                .completed
                .into_iter()
                .map(|result| {
                    assert_eq!(result.value.pass_lines.len(), 1);
                    (
                        result.id,
                        result.value.phase,
                        result.value.pass_lines.into_iter().next().unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(semantics, expected_semantics, "W{workers}");

            let allocation_by_phase = allocations
                .lock()
                .expect("allocation observations")
                .iter()
                .copied()
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                allocation_by_phase,
                BTreeMap::from([
                    (
                        LIVE_PHASE,
                        crate::scheduler::live_worker_allocation(workers)
                    ),
                    (OBLIGATIONS_PHASE, 1),
                    (STATE_FORM_PHASE, 1),
                ]),
                "W{workers} adapter allocations"
            );
        }
    }

    fn synthetic_failure(
        phase: &'static str,
        delay: Duration,
        started: Arc<Mutex<Vec<&'static str>>>,
    ) -> FamilyJob {
        FamilyJob::Synthetic(Box::new(move |_, cancellation| {
            started.lock().expect("start observations").push(phase);
            let deadline = std::time::Instant::now() + delay;
            while std::time::Instant::now() < deadline {
                if phase != LIVE_PHASE {
                    ensure_active(&cancellation, phase)?;
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(Error::new(format!("{phase}: watched failure")))
        }))
    }

    #[test]
    fn real_family_graph_selects_the_w1_failure_for_w1_through_w4() {
        for workers in 1..=crate::scheduler::MAX_WORKERS {
            let started = Arc::new(Mutex::new(Vec::new()));
            let graph = family_graph_from_jobs(
                workers,
                synthetic_failure(LIVE_PHASE, Duration::from_millis(20), Arc::clone(&started)),
                synthetic_failure(OBLIGATIONS_PHASE, Duration::ZERO, Arc::clone(&started)),
                synthetic_pass(STATE_FORM_PHASE, Arc::new(Mutex::new(Vec::new()))),
            )
            .expect("real family graph shape");
            let run = crate::scheduler::run_dag(graph, None, |_, _, lanes, job, cancellation| {
                execute_family_job(lanes, job, cancellation)
            });

            assert!(run.completed.is_empty(), "W{workers}");
            let failure = run.failure.expect("watched family failure");
            assert_eq!(failure.id, Some(LIVE_PHASE), "W{workers}");
            match failure.kind {
                DagFailureKind::Job(error) => assert_eq!(
                    error.to_string(),
                    format!("{LIVE_PHASE}: watched failure"),
                    "W{workers}"
                ),
                other => panic!("W{workers} selected {other:?}"),
            }
            let starts = started.lock().expect("start observations");
            assert!(starts.contains(&LIVE_PHASE));
            assert!(!starts.contains(&STATE_FORM_PHASE));
        }
    }

    fn synthetic_wait_for_cancellation(
        phase: &'static str,
        started: Arc<AtomicUsize>,
        stopped: Arc<AtomicUsize>,
    ) -> FamilyJob {
        FamilyJob::Synthetic(Box::new(move |_, cancellation| {
            started.fetch_add(1, Ordering::SeqCst);
            while !cancellation.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
            stopped.fetch_add(1, Ordering::SeqCst);
            Err(Error::new(format!("{phase}: cancelled")))
        }))
    }

    #[test]
    fn real_family_graph_cancels_and_joins_every_started_adapter() {
        for workers in 1..=crate::scheduler::MAX_WORKERS {
            let started = Arc::new(AtomicUsize::new(0));
            let stopped = Arc::new(AtomicUsize::new(0));
            let graph = family_graph_from_jobs(
                workers,
                synthetic_wait_for_cancellation(
                    LIVE_PHASE,
                    Arc::clone(&started),
                    Arc::clone(&stopped),
                ),
                synthetic_wait_for_cancellation(
                    OBLIGATIONS_PHASE,
                    Arc::clone(&started),
                    Arc::clone(&stopped),
                ),
                synthetic_wait_for_cancellation(
                    STATE_FORM_PHASE,
                    Arc::clone(&started),
                    Arc::clone(&stopped),
                ),
            )
            .expect("real family graph shape");
            let external = CancellationToken::new();
            let canceller = external.clone();
            let cancel_thread = thread::spawn(move || {
                thread::sleep(Duration::from_millis(30));
                canceller.cancel();
            });
            let run = crate::scheduler::run_dag(
                graph,
                Some(external),
                |_, _, lanes, job, cancellation| execute_family_job(lanes, job, cancellation),
            );
            cancel_thread.join().expect("cancellation thread");

            assert!(run.completed.is_empty(), "W{workers}");
            assert!(matches!(
                run.failure.map(|failure| failure.kind),
                Some(DagFailureKind::Cancelled)
            ));
            let expected_started = if workers <= 2 { 1 } else { 2 };
            assert_eq!(
                started.load(Ordering::SeqCst),
                expected_started,
                "W{workers}"
            );
            assert_eq!(
                stopped.load(Ordering::SeqCst),
                expected_started,
                "W{workers}"
            );
        }
    }
}
