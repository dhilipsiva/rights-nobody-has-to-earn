// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::Write;
use std::path::Path;

use crate::checks::{
    amendment, assertion_surface, assurance, claim_table, ledger, obligations, pilot, pins,
    placement, power_manifest, reader, red_team, registry, repository, spine, state_form, temporal,
};
use crate::cli::Error;
use crate::context::Context;
use crate::report::Reporter;

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

/// Run the complete native verification pipeline in its required order.
///
/// Every checker is linked into this process. Reviewed JSON is decoded into
/// each checker's strict serde model before semantic validation begins.
pub(crate) fn run<W: Write>(context: &Context, mode: RunMode, output: W) -> Result<(), Error> {
    let mut report = Reporter::with_recorder(output, crate::diagnostics::observe());
    let execute = mode.executes();

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
    drop(constitution);

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
    report.pass(obligations::check(context, &obligation_snapshot)?.to_string())?;

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
        // Resolve every executable input while the suite is still in its
        // preflight phase. No Nibli query runs until all structural,
        // repository, claim-table, registry, and counterfactual-shape guards
        // above have passed.
        let live_plan = pins::prepare_live_families(context)?;
        let counterfactual_plan = pins::prepare_counterfactuals(context)?;

        report.step("reader-evidence evaluator controls")?;
        report.pass(reader::check(
            context,
            true,
            reader::InputSnapshot::default(),
        )?)?;

        report.step("chapter, floor, and family pins")?;
        let live = pins::execute_live_families(context, &engine, &live_plan)?;
        crate::diagnostics::add_details(live.file_timings);
        report.pass(format!("{} chapter/floor pins pass", live.chapter_pins))?;
        for (family, count) in live.family_results {
            report.pass(format!("{family}: {count} pins pass"))?;
        }

        report.step("obligations executions")?;
        let obligation_result = obligations::execute(context, &obligation_snapshot)?;
        report.pass(format!(
            "obligations: PASS — {} isolated cases and {} counterfactual suites execute",
            obligation_result.cases, obligation_result.counterfactual_suites
        ))?;

        report.step("state-form executions")?;
        let state_result =
            state_form::execute_validated(context, &state_snapshot, &validated_state_form)?;
        crate::diagnostics::add_details(state_result.shard_timings);
        report.pass(format!(
            "state-form: PASS — {} main shards / {} pins and {} counterfactual shards / {} pins",
            state_result.main_shards,
            state_result.main_pins,
            state_result.counterfactual_shards,
            state_result.counterfactual_pins,
        ))?;

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
