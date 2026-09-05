// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native staged T1/T2/T3 temporal-assurance verifier.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Condvar, LazyLock, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine, run_pin_files};
use crate::scheduler::{
    CancellationHookGuard, CancellationToken, ScheduleError, ScheduleOptions,
    run_bounded_with_state_controlled,
};

pub(crate) const STEP_NAME: &str = "temporal assurance";

const DEFAULT_SOURCE: &str = "new-book-plans/temporal-assurance-case.json";
const DEFAULT_KB: &str = "new-book-plans/constitution.nibli";
const DEFAULT_OUTPUT: &str = "new-book-plans/temporal-assurance-case.md";

const BOUND_SOURCES: [(&str, &str); 6] = [
    ("time_model", "new-book-plans/book-1-time-model-decision.md"),
    (
        "assertion_surface",
        "new-book-plans/assertion-surface-contracts.json",
    ),
    (
        "record_assurance",
        "new-book-plans/record-integrity-assurance-case.json",
    ),
    (
        "record_red_team",
        "new-book-plans/record-integrity-red-team.json",
    ),
    (
        "amendment_semantics",
        "new-book-plans/amendment-semantics-audit.json",
    ),
    (
        "placement_exhaustiveness",
        "new-book-plans/placement-exhaustiveness-audit.json",
    ),
];
const STAGES: [&str; 3] = ["T1", "T2", "T3"];
const ALL_MARKERS: [&str; 12] = [
    "T1-ADMISSIONS",
    "T1-DERIVED",
    "T1-RULES",
    "T1-FACTS",
    "T2-DERIVED",
    "T2-RULES",
    "T2-FACTS",
    "T3-DERIVED",
    "T3-RULES",
    "T3-FACTS",
    "T3-COURT-GATE",
    "T3-LEASE-RULE",
];
const REQUIRED_INPUTS: [&str; 10] = [
    "transition_link",
    "snapshot_manifest",
    "independent_observation",
    "event_order",
    "record_order",
    "review_window",
    "case_record",
    "case_renewal",
    "effective_source",
    "challenge",
];
const REQUIRED_ATTACKS: [&str; 33] = [
    "carry_omission",
    "carry_forgery",
    "status_conflict",
    "cross_snapshot_disappearance",
    "post_attestation_injection",
    "authority_from_missing_carry",
    "replay",
    "divergence",
    "frozen_transition",
    "order_conflict",
    "backdating",
    "release_withholding",
    "maturity_forgery",
    "source_mismatch",
    "renewal_withholding",
    "power_witness_withholding",
    "status_tuple_aliasing",
    "canonical_lineage",
    "successor_without_renewal",
    "manifest_ambiguity",
    "typed_collision_fail_closed",
    "judgment_basis_withholding",
    "wrong_case_reuse",
    "self_review",
    "stale_authority",
    "query_shape_explosion",
    "personal_time_score",
    "emergency_redeclaration",
    "office_succession",
    "finding_action_route",
    "external_liveness",
    "standing_witness_withholding",
    "challenge_intake_withholding",
];
const REQUIRED_NARROWNESS: [&str; 10] = [
    "book-1/01-what-counts-as-evidence.md",
    "book-1/04-the-shield.md",
    "book-1/05-voiding.md",
    "book-1/07-a-prisoner-is-a-person.md",
    "book-1/08-what-you-are-owed.md",
    "book-1/09-the-vote-conviction-does-not-take.md",
    "book-1/13-the-one-thing-taken.md",
    "book-1/14-when-the-system-notices-it-broke.md",
    "book-1/15-the-five-joints.md",
    "book-1/method.md",
];
const LIMIT_KEYS: [&str; 9] = [
    "safety_not_liveness",
    "snapshot_scope",
    "external_time",
    "physical_effect",
    "institutional_action",
    "record_truth",
    "query_shape",
    "source_transition",
    "trust_root",
];
const POSTURES: [&str; 4] = [
    "rejected_by_current_source",
    "detected_as_named_error",
    "preserved_by_narrowness_test",
    "exposed_external_boundary",
];
const CLASSIFICATIONS: [&str; 3] = [
    "revised_and_pinned",
    "preserved_and_pinned",
    "boundary_rewritten",
];
const COLLISION_TAGS: [&str; 11] = [
    "CaseBinding",
    "EventOrder",
    "LeaseBinding",
    "LeaseSuspended",
    "LineageBinding",
    "ManifestBinding",
    "RecordOrder",
    "SourceBinding",
    "TransitionLineage",
    "WindowDefinition",
    "WindowOrder",
];
const FORBIDDEN_CASE_SCOPE_TOKENS: [&str; 5] = [
    "EventOrderScope",
    "RecordOrderScope",
    "WindowScope",
    "SourceScope",
    "SourceLeaseScope",
];
const FORBIDDEN_TEMPORAL_HEADS: [&str; 12] = [
    "-> complete($item, $evidence",
    "-> succeed($after, Transition,",
    "-> precede($before_event, $after_event, EventPath,",
    "-> precede($before_entry, $after_entry, RecordPath,",
    "-> time($window, $start,",
    "-> succeed($lease, $power,",
    "-> succeed($lease, $authority,",
    "-> related($case, $subject,",
    "-> orderly($lease, $window)",
    "-> concurrent($lease, $current)",
    "-> reference($binding, Constitution_Temporal,",
    "-> correct($lease, $power,",
];

const REQUIRED_SOURCE_SENTINELS: &[&str] = &[
    "derived_only(\"collide\").",
    "-> complete($item, ManifestScope).",
    "complete($before_manifest, ManifestScope) & observe(Chronicle, $before_manifest, $before, ManifestScope)",
    "complete($after_manifest, ManifestScope) & observe(Chronicle, $after_manifest, $after, ManifestScope)",
    "-> collide($after, TransitionLineage).",
    "-> collide($manifest, ManifestBinding).",
    "-> collide($epoch, ManifestBinding).",
    "-> succeed($after, Transition).",
    "replace($later, $after, Chronicle) -> complete($after, HasSuccessor).",
    "-> collide(Binding_Chronicle, LineageBinding).",
    "~collide(Binding_Chronicle, LineageBinding) -> succeed($after, TerminalTransition).",
    "succeed($after, TerminalTransition) & replace($after, $before, Chronicle)",
    "succeed($after, Transition) & replace($after, $before, Chronicle)",
    "replace($first, $before, Chronicle) & replace($second, $before, Chronicle)",
    "list($manifest, $first, ManifestOrder, Chronicle) & list($manifest, $second, ManifestOrder, Chronicle)",
    "list($first, $epoch, ManifestOrder, Chronicle) & list($second, $epoch, ManifestOrder, Chronicle)",
    "observe(TemporalReview, Binding_Chronicle, $second_epoch, LineageEpochScope) & ~($first_family = $second_family) -> collide(Binding_Chronicle, LineageBinding).",
    "observe(TemporalReview, Binding_Chronicle, $second_epoch, LineageEpochScope) & ~($first_version = $second_version) -> collide(Binding_Chronicle, LineageBinding).",
    "observe(TemporalReview, Binding_Chronicle, $second_epoch, LineageEpochScope) & ~($first_epoch = $second_epoch) -> collide(Binding_Chronicle, LineageBinding).",
    "match($x, CarriedVoid) & match($x, CarriedClear) -> err($x, StatusConflict)",
    "observe(Chronicle, $claim, $before_event, EventStartScope)",
    "observe(Chronicle, $claim, $after_event, EventEndScope)",
    "-> precede($claim, EventSequence).",
    "observe(Chronicle, $claim, $before_entry, RecordStartScope)",
    "observe(Chronicle, $claim, $after_entry, RecordEndScope)",
    "-> precede($claim, RecordSequence).",
    "observe(Chronicle, $window, $start, WindowStartScope)",
    "observe(Chronicle, $window, $end, WindowEndScope)",
    "-> time($window, ReviewedInterval).",
    "-> precede($before_event, $after_event, EventPath).",
    "-> precede($before_entry, $after_entry, RecordPath).",
    "precede($first, $middle, EventPath) & precede($middle, $last, EventPath)",
    "precede($first, $middle, RecordPath) & precede($middle, $last, RecordPath)",
    "-> err($claim, OrderConflict).",
    "-> collide($before_event, EventOrder).",
    "-> collide($after_event, EventOrder).",
    "-> collide($before_entry, RecordOrder).",
    "-> collide($after_entry, RecordOrder).",
    "precede($start, $end, EventPath) & precede($start, $end, RecordPath)",
    "-> collide($window, WindowDefinition).",
    "observe(TemporalReview, $window, $end, WindowEndScope) & precede($end, $start, EventPath) -> collide($window, WindowOrder).",
    "observe(TemporalReview, $window, $end, WindowEndScope) & precede($end, $start, RecordPath) -> collide($window, WindowOrder).",
    "observe(TemporalReview, $window, $second_end, WindowEndScope) & ~($first_start = $second_start) -> collide($window, WindowDefinition).",
    "observe(TemporalReview, $window, $second_end, WindowEndScope) & ~($first_end = $second_end) -> collide($window, WindowDefinition).",
    "observe(Chronicle, $lease, $authority, PowerScope)",
    "observe(Chronicle, $lease, $case, CaseBindingScope)",
    "-> succeed($lease, PowerBound).",
    "observe(Chronicle, $case, $subject, CaseScope)",
    "observe(Chronicle, $case, $holder, HolderScope)",
    "-> related($case, CaseBound).",
    "observe(Chronicle, $lease, $window, LimitScope)",
    "-> orderly($lease, WindowBound).",
    "observe(Chronicle, $lease, $current, RenewalScope)",
    "-> concurrent($lease, RenewalBound).",
    "observe(Chronicle, $binding, TemporalLeaseFamily, SourceFamilyScope)",
    "observe(Chronicle, $binding, Constitution_Temporal, SourceVersionScope)",
    "observe(Chronicle, $binding, $current, SourceEpochScope)",
    "-> collide($binding, SourceBinding).",
    "observe(TemporalReview, $binding, $second_current, SourceEpochScope) & ~($first_family = $second_family) -> collide($binding, SourceBinding).",
    "observe(TemporalReview, $binding, $second_current, SourceEpochScope) & ~($first_version = $second_version) -> collide($binding, SourceBinding).",
    "observe(TemporalReview, $binding, $second_current, SourceEpochScope) & ~($first_current = $second_current) -> collide($binding, SourceBinding).",
    "~collide($binding, SourceBinding) -> reference($binding, SourceBound).",
    "-> reference($binding, SourceBound).",
    "succeed($lease, PowerBound) & authorized($lease, ActiveCustody, $case)",
    "authorized($lease, ActiveCustody, $case) & observe(Chronicle, $lease, ActiveCustody, PowerScope)",
    "related($case, CaseBound) & cite(Court, $case, $subject)",
    "cite(Court, $case, $subject) & observe(Chronicle, $case, $subject, CaseScope)",
    "limit($lease, $case, $window) & orderly($lease, WindowBound)",
    "orderly($lease, WindowBound) & observe(Chronicle, $lease, $window, LimitScope)",
    "continue($lease, $current) & concurrent($lease, RenewalBound)",
    "concurrent($lease, RenewalBound) & observe(Chronicle, $lease, $current, RenewalScope)",
    "passport($binding, TemporalLeaseFamily, Constitution_Temporal, $current) & reference($binding, SourceBound)",
    "reference($binding, SourceBound) & ~collide($binding, SourceBinding) & observe(Chronicle, $binding, TemporalLeaseFamily, SourceFamilyScope)",
    "date($window, $current, $end, TimeService) & time($window, ReviewedInterval)",
    "time($window, ReviewedInterval) & ~collide($window, WindowDefinition) & ~collide($window, WindowOrder) & observe(Chronicle, $window, $current, WindowStartScope)",
    "-> collide($case, CaseBinding).",
    "-> collide($lease, LeaseBinding).",
    "-> collide($lease, LeaseSuspended).",
    "observe(TemporalReview, $lease, $second_case, CaseBindingScope) & ~($first_authority = $second_authority) -> collide($lease, LeaseBinding).",
    "observe(TemporalReview, $lease, $second_case, CaseBindingScope) & ~($first_case = $second_case) -> collide($lease, LeaseBinding).",
    "observe(TemporalReview, $lease, $second_window, LimitScope) & ~($first_case = $second_case) -> collide($lease, LeaseBinding).",
    "observe(TemporalReview, $lease, $second_window, LimitScope) & ~($first_window = $second_window) -> collide($lease, LeaseBinding).",
    "observe(TemporalReview, $lease, $second, RenewalScope) & ~($first = $second) -> collide($lease, LeaseBinding).",
    "~collide($case, CaseBinding) & ~collide($lease, LeaseBinding) & ~collide($lease, LeaseSuspended)",
    "~collide($window, WindowDefinition) & ~collide($window, WindowOrder)",
    "replace($current, $previous, Chronicle) & succeed($current, TerminalTransition) -> correct($lease, ActivePower).",
    "-> correct($lease, ActivePower).",
    "judge(Court, $subject) & injure($subject, $victim) & observe(Chronicle, $case, CourtJudgment, JudgmentScope)",
    "all $lease: all $case: all $subject: related($case, CaseBound) & cite(Court, $case, $subject)",
    "observe(TemporalReview, $case, Court, HolderScope) & authorized($lease, ActiveCustody, $case) & ~collide($case, CaseBinding) -> match($subject, CaseRecorded).",
    "all $lease: all $case: all $subject: authorized($lease, ActiveCustody, $case) & related($case, CaseBound) & cite(Court, $case, $subject)",
    "~match($lease, ActivePower) -> err($subject, TemporalAuthority).",
    "authorized($lease, ActiveCustody, $case) & ~match($lease, ActivePower) -> err($lease, TemporalRecord).",
    "-> match($offender, ConvictionRecorded).",
    "match($offender, ConvictionRecorded) & observe(Chronicle, $case, $offender, CaseScope)",
    "~match($offender, ConvictionRecorded) -> err($offender, TemporalAuthority).",
    "observe(TemporalReview, $case, CourtJudgment, JudgmentScope)",
    "observe(TemporalReview, $case, $victim, InjuryVictimScope)",
];

static VARIABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$[A-Za-z_][A-Za-z0-9_]*").expect("valid regex"));
static ALLOWED_EVENT_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^precede\(\$[A-Za-z_][A-Za-z0-9_]*, \$[A-Za-z_][A-Za-z0-9_]*, EventPath\)\.$")
        .expect("valid regex")
});
static ALLOWED_RECORD_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^precede\(\$[A-Za-z_][A-Za-z0-9_]*, \$[A-Za-z_][A-Za-z0-9_]*, RecordPath\)\.$")
        .expect("valid regex")
});
static COLLISION_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^collide\((?:\$[A-Za-z_][A-Za-z0-9_]*|[A-Za-z][A-Za-z0-9_]*), ([A-Za-z][A-Za-z0-9_]*)\)\.$",
    )
    .expect("valid regex")
});
static COLLISION_QUERY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^collide\([^,]+, ([A-Za-z][A-Za-z0-9_]*)\)$").expect("valid regex")
});
static TRANSITION_QUERY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^succeed\([^,]+, (?:Transition|TerminalTransition)\)$").expect("valid regex")
});
static TRANSITION_COLLISION_QUERY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^collide\([^,]+, TransitionLineage\)$").expect("valid regex"));
static EVENT_ENTITLEMENT_QUERY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^entitled\([A-Za-z][A-Za-z0-9_]*, event \{ [a-z][a-z0-9_]*\(\) \}\)$")
        .expect("valid regex")
});

#[derive(Clone, Debug)]
pub(crate) struct Paths {
    pub(crate) source: PathBuf,
    pub(crate) kb: PathBuf,
    pub(crate) output: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            source: PathBuf::from(DEFAULT_SOURCE),
            kb: PathBuf::from(DEFAULT_KB),
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExecutionReport {
    pub(crate) cases: usize,
    pub(crate) pins: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
    pub(crate) execution: Option<ExecutionReport>,
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is current; {} structural negative controls pass",
            self.output, self.structural_controls
        )?;
        match &self.execution {
            Some(execution) => write!(
                formatter,
                "; {} fresh processes / {} pins pass",
                execution.cases, execution.pins
            ),
            None => formatter.write_str("; execution skipped"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenerationReport {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
}

impl fmt::Display for GenerationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: regenerated; {} structural negative controls pass",
            self.output, self.structural_controls
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FingerprintReport(String);

impl fmt::Display for FingerprintReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalSource {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    evidence_role: String,
    subprocess_timeout_seconds: u64,
    constitution_sha256: String,
    bound_sources_sha256: BTreeMap<String, String>,
    marker_sha256: BTreeMap<String, String>,
    stage_source_sha256: BTreeMap<String, String>,
    source_effect_binding: SourceEffectBinding,
    pre_t3_custody_rule: String,
    temporal_input_contracts: Vec<TemporalInputContract>,
    stages: Vec<Stage>,
    cases: Vec<Case>,
    fresh_process_pairs: Vec<FreshProcessPair>,
    attacks: Vec<Attack>,
    narrowness_impacts: Vec<NarrownessImpact>,
    limits: BTreeMap<String, String>,
    acceptance_result: AcceptanceResult,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEffectBinding {
    effective_version: String,
    case_bound_rule_fragment: String,
    case_bound_rule_sha256: String,
    source_binding_fragment: String,
    source_binding_sha256: String,
    semantic_effect: String,
    compatibility_review: String,
    article9_boundary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalInputContract {
    id: String,
    stage: String,
    writer: String,
    evidence: String,
    forge_route: String,
    withholding_route: String,
    correction: String,
    appeal: String,
    cross_epoch_handoff: String,
    residual_external_assurance: String,
    book_boundary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stage {
    id: String,
    cumulative: Vec<String>,
    claim: String,
    does_not_establish: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    stage: String,
    title: String,
    process_role: String,
    description: String,
    deletions: Vec<String>,
    additions: Vec<String>,
    checks: Vec<Check>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    expression: String,
    expected: String,
    purpose: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FreshProcessPair {
    id: String,
    predecessor_case: String,
    successor_case: String,
    purpose: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Attack {
    id: String,
    stage: String,
    control: String,
    case_refs: Vec<String>,
    posture: String,
    boundary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NarrownessImpact {
    artifact_ref: String,
    standing_claim: String,
    classification: String,
    test_route: String,
    change_trigger: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceResult {
    result: String,
    claim: String,
    does_not_establish: String,
    remaining_boundary: String,
}

#[derive(Clone, Debug)]
struct Snapshot {
    reviewed: TemporalSource,
    constitution: String,
    dependencies: BTreeMap<String, Vec<u8>>,
    narrowness_files: BTreeSet<String>,
    source_relative: String,
    kb_relative: String,
    output_relative: String,
    output_path: PathBuf,
    current_output: Option<String>,
}

#[derive(Clone, Debug)]
struct Fingerprints {
    constitution_sha256: String,
    bound_sources_sha256: BTreeMap<String, String>,
    marker_sha256: BTreeMap<String, String>,
    stage_source_sha256: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct Validated {
    stage_sources: BTreeMap<String, String>,
}

fn ensure_execution_active(cancellation: Option<&CancellationToken>) -> Result<(), Error> {
    if cancellation.is_some_and(CancellationToken::is_cancelled) {
        Err(temporal_error("temporal execution cancelled"))
    } else {
        Ok(())
    }
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_with_paths(context, &Paths::default(), false)
}

pub(crate) fn check_execute(context: &Context) -> Result<Report, Error> {
    check_with_paths(context, &Paths::default(), true)
}

pub(crate) fn check_execute_with_allocation(
    context: &Context,
    workers: usize,
    cancellation: CancellationToken,
) -> Result<Report, Error> {
    check_with_paths_and_allocation(context, &Paths::default(), Some((workers, cancellation)))
}

pub(crate) fn check_with_paths(
    context: &Context,
    paths: &Paths,
    execute: bool,
) -> Result<Report, Error> {
    let allocation = execute
        .then(|| {
            crate::scheduler::configured_workers()
                .map(|workers| (workers, CancellationToken::new()))
        })
        .transpose()?;
    check_with_paths_and_allocation(context, paths, allocation)
}

fn check_with_paths_and_allocation(
    context: &Context,
    paths: &Paths,
    allocation: Option<(usize, CancellationToken)>,
) -> Result<Report, Error> {
    let cancellation = allocation.as_ref().map(|(_, token)| token.clone());
    ensure_execution_active(cancellation.as_ref())?;
    let snapshot = load_snapshot_with_cancellation(context, paths, true, cancellation.as_ref())?;
    ensure_execution_active(cancellation.as_ref())?;
    let validated = validate_source_with_cancellation(&snapshot, cancellation.as_ref())?;
    ensure_execution_active(cancellation.as_ref())?;
    let generated = render(
        &snapshot.reviewed,
        &snapshot.source_relative,
        &snapshot.kb_relative,
    );
    ensure_execution_active(cancellation.as_ref())?;
    let structural_controls =
        negative_controls_with_cancellation(&snapshot, cancellation.as_ref())?;
    ensure_execution_active(cancellation.as_ref())?;
    let execution = allocation
        .map(|(workers, cancellation)| {
            execute_cases_with_allocation(
                &snapshot.reviewed,
                &validated.stage_sources,
                workers,
                cancellation,
            )
        })
        .transpose()?;
    ensure_execution_active(cancellation.as_ref())?;
    if snapshot.current_output.as_deref() != Some(generated.as_str()) {
        return Err(temporal_error(format!(
            "{} is STALE — rerun without --check",
            snapshot.output_relative
        )));
    }
    Ok(Report {
        output: snapshot.output_relative,
        structural_controls,
        execution,
    })
}

pub(crate) fn generate(context: &Context) -> Result<GenerationReport, Error> {
    generate_with_paths(context, &Paths::default())
}

pub(crate) fn generate_with_paths(
    context: &Context,
    paths: &Paths,
) -> Result<GenerationReport, Error> {
    let snapshot = load_snapshot(context, paths, false)?;
    validate_source(&snapshot)?;
    let generated = render(
        &snapshot.reviewed,
        &snapshot.source_relative,
        &snapshot.kb_relative,
    );
    let structural_controls = negative_controls(&snapshot)?;
    write_output(&snapshot.output_path, &generated)?;
    Ok(GenerationReport {
        output: snapshot.output_relative,
        structural_controls,
    })
}

pub(crate) fn fingerprints(context: &Context) -> Result<FingerprintReport, Error> {
    fingerprints_with_paths(context, &Paths::default())
}

pub(crate) fn fingerprints_with_paths(
    context: &Context,
    paths: &Paths,
) -> Result<FingerprintReport, Error> {
    let snapshot = load_snapshot(context, paths, false)?;
    text_value(
        &snapshot.reviewed.pre_t3_custody_rule,
        "pre_t3_custody_rule",
    )?;
    let fingerprints = core_fingerprints(
        &snapshot.constitution,
        &snapshot.dependencies,
        &snapshot.reviewed.pre_t3_custody_rule,
    )?;
    let mut value = serde_json::json!({
        "constitution_sha256": fingerprints.constitution_sha256,
        "bound_sources_sha256": fingerprints.bound_sources_sha256,
        "marker_sha256": fingerprints.marker_sha256,
        "stage_source_sha256": fingerprints.stage_source_sha256,
        "case_bound_rule_sha256": sha256(snapshot.reviewed.source_effect_binding.case_bound_rule_fragment.as_bytes()),
        "source_binding_sha256": sha256(snapshot.reviewed.source_effect_binding.source_binding_fragment.as_bytes()),
    });
    sort_json(&mut value);
    Ok(FingerprintReport(serde_json::to_string_pretty(&value)?))
}

fn load_snapshot(context: &Context, paths: &Paths, read_output: bool) -> Result<Snapshot, Error> {
    load_snapshot_with_cancellation(context, paths, read_output, None)
}

fn load_snapshot_with_cancellation(
    context: &Context,
    paths: &Paths,
    read_output: bool,
    cancellation: Option<&CancellationToken>,
) -> Result<Snapshot, Error> {
    ensure_execution_active(cancellation)?;
    let source_path = resolve_path(context, &paths.source);
    let kb_path = resolve_path(context, &paths.kb);
    let output_path = resolve_path(context, &paths.output);
    let source_relative = repo_relative(context.root(), &source_path)?;
    let kb_relative = repo_relative(context.root(), &kb_path)?;
    let output_relative = repo_relative(context.root(), &output_path)?;
    if output_relative != DEFAULT_OUTPUT {
        return Err(temporal_error(
            "--output is fixed to new-book-plans/temporal-assurance-case.md",
        ));
    }

    let source_bytes = read_bytes(&source_path, "temporal assurance source")?;
    ensure_execution_active(cancellation)?;
    parse_json_no_duplicates(&source_bytes).map_err(|error| {
        temporal_error(format!("cannot parse temporal assurance source: {error}"))
    })?;
    let reviewed: TemporalSource = serde_json::from_slice(&source_bytes).map_err(|error| {
        temporal_error(format!("cannot parse temporal assurance source: {error}"))
    })?;
    let constitution = decode(&read_bytes(&kb_path, "constitution")?, "constitution")?;
    ensure_execution_active(cancellation)?;
    let mut dependencies = BTreeMap::new();
    for (key, relative) in BOUND_SOURCES {
        ensure_execution_active(cancellation)?;
        let path = context.path(relative);
        repo_relative(context.root(), &path)?;
        dependencies.insert(key.to_owned(), read_bytes(&path, key)?);
        ensure_execution_active(cancellation)?;
    }

    let mut narrowness_files = BTreeSet::new();
    for impact in &reviewed.narrowness_impacts {
        ensure_execution_active(cancellation)?;
        let candidate = resolve_path(context, Path::new(&impact.artifact_ref));
        if let Ok(relative) = repo_relative(context.root(), &candidate)
            && candidate.is_file()
        {
            narrowness_files.insert(relative);
        }
    }
    let current_output = if read_output {
        ensure_execution_active(cancellation)?;
        Some(decode(
            &read_bytes(&output_path, "generated temporal report")?,
            "generated temporal report",
        )?)
    } else {
        None
    };
    ensure_execution_active(cancellation)?;
    Ok(Snapshot {
        reviewed,
        constitution,
        dependencies,
        narrowness_files,
        source_relative,
        kb_relative,
        output_relative,
        output_path,
        current_output,
    })
}

fn resolve_path(context: &Context, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        context.path(path)
    }
}

fn repo_relative(root: &Path, path: &Path) -> Result<String, Error> {
    let root = root.canonicalize()?;
    let resolved = canonicalize_allow_missing(path)?;
    resolved
        .strip_prefix(&root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| temporal_error(format!("path escapes repository: {}", path.display())))
}

fn canonicalize_allow_missing(path: &Path) -> Result<PathBuf, Error> {
    let normalized = lexical_normalize(path)?;
    let mut existing = normalized.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            temporal_error(format!("path escapes repository: {}", path.display()))
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            temporal_error(format!("path escapes repository: {}", path.display()))
        })?;
    }
    let mut resolved = existing.canonicalize()?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn lexical_normalize(path: &Path) -> Result<PathBuf, Error> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(temporal_error(format!(
                        "path escapes repository: {}",
                        path.display()
                    )));
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn read_bytes(path: &Path, label: &str) -> Result<Vec<u8>, Error> {
    if path.is_symlink() {
        return Err(temporal_error(format!(
            "{label} may not be a symlink: {}",
            path.display()
        )));
    }
    let value = std::fs::read(path)
        .map_err(|error| temporal_error(format!("cannot read {label}: {error}")))?;
    if value.contains(&b'\r') {
        return Err(temporal_error(format!("{label} must use LF line endings")));
    }
    Ok(value)
}

fn decode(value: &[u8], label: &str) -> Result<String, Error> {
    String::from_utf8(value.to_vec())
        .map_err(|error| temporal_error(format!("{label} is not UTF-8: {error}")))
}

fn marker_block(source: &str, marker: &str) -> Result<String, Error> {
    let begin = format!("# <{marker}-BEGIN>\n");
    let end = format!("# <{marker}-END>\n");
    if source.matches(&begin).count() != 1 || source.matches(&end).count() != 1 {
        return Err(temporal_error(format!(
            "source must contain exactly one {marker} marker pair"
        )));
    }
    let start = source.find(&begin).expect("count checked");
    let stop = source[start..]
        .find(&end)
        .map(|offset| start + offset + end.len())
        .ok_or_else(|| temporal_error(format!("invalid {marker} marker order")))?;
    if stop <= start {
        return Err(temporal_error(format!("invalid {marker} marker order")));
    }
    Ok(source[start..stop].to_owned())
}

fn replace_once(source: &str, before: &str, after: &str, label: &str) -> Result<String, Error> {
    let count = source.matches(before).count();
    if count != 1 {
        return Err(temporal_error(format!(
            "{label}: expected exact fragment once, found {count}"
        )));
    }
    Ok(source.replacen(before, after, 1))
}

fn build_stages(source: &str, pre_t3_rule: &str) -> Result<BTreeMap<String, String>, Error> {
    for marker in ALL_MARKERS {
        marker_block(source, marker)?;
    }
    let gate = marker_block(source, "T3-COURT-GATE")?;
    let legacy = format!(
        "# Staged temporal assurance: T3 custody gate intentionally absent.\n{}\n",
        pre_t3_rule.trim_end()
    );
    let mut result = BTreeMap::from([("T3".to_owned(), source.to_owned())]);
    let mut t2 = source.to_owned();
    for marker in ["T3-DERIVED", "T3-RULES", "T3-FACTS"] {
        let block = marker_block(&t2, marker)?;
        t2 = replace_once(&t2, &block, "", &format!("T2 remove {marker}"))?;
    }
    t2 = replace_once(&t2, &gate, &legacy, "T2 replace custody gate")?;
    result.insert("T2".to_owned(), t2.clone());
    let mut t1 = t2;
    for marker in ["T2-DERIVED", "T2-RULES", "T2-FACTS"] {
        let block = marker_block(&t1, marker)?;
        t1 = replace_once(&t1, &block, "", &format!("T1 remove {marker}"))?;
    }
    result.insert("T1".to_owned(), t1);
    if !(result["T1"].len() < result["T2"].len() && result["T2"].len() < result["T3"].len()) {
        return Err(temporal_error(
            "cumulative stage sources do not grow T1 -> T2 -> T3",
        ));
    }
    Ok(result)
}

fn apply_case(stage_source: &str, case: &Case) -> Result<String, Error> {
    let mut candidate = stage_source.to_owned();
    for deletion in validate_ground_facts(&case.deletions, &format!("case {}.deletions", case.id))?
    {
        candidate = replace_once(
            &candidate,
            &format!("{deletion}\n"),
            "",
            &format!("case {} deletion", case.id),
        )?;
    }
    let additions = validate_ground_facts(&case.additions, &format!("case {}.additions", case.id))?;
    if !additions.is_empty() {
        candidate.push_str("\n# Temporal-assurance case overlay (generated; never enacted).\n");
        candidate.push_str(&additions.join("\n"));
        candidate.push('\n');
    }
    Ok(candidate)
}

fn core_fingerprints(
    source: &str,
    dependencies: &BTreeMap<String, Vec<u8>>,
    pre_t3_rule: &str,
) -> Result<Fingerprints, Error> {
    let stages = build_stages(source, pre_t3_rule)?;
    let bound_sources_sha256 = dependencies
        .iter()
        .map(|(key, value)| (key.clone(), sha256(value)))
        .collect();
    let mut marker_sha256 = BTreeMap::new();
    for marker in ALL_MARKERS {
        marker_sha256.insert(
            marker.to_owned(),
            sha256(marker_block(source, marker)?.as_bytes()),
        );
    }
    let stage_source_sha256 = STAGES
        .into_iter()
        .map(|stage| (stage.to_owned(), sha256(stages[stage].as_bytes())))
        .collect();
    Ok(Fingerprints {
        constitution_sha256: sha256(source.as_bytes()),
        bound_sources_sha256,
        marker_sha256,
        stage_source_sha256,
    })
}

fn validate_source(snapshot: &Snapshot) -> Result<Validated, Error> {
    validate_source_with_cancellation(snapshot, None)
}

fn validate_source_with_cancellation(
    snapshot: &Snapshot,
    cancellation: Option<&CancellationToken>,
) -> Result<Validated, Error> {
    validate_source_parts_with_cancellation(
        &snapshot.reviewed,
        &snapshot.constitution,
        &snapshot.dependencies,
        &snapshot.narrowness_files,
        cancellation,
    )
}

fn validate_source_parts(
    reviewed: &TemporalSource,
    constitution: &str,
    dependencies: &BTreeMap<String, Vec<u8>>,
    narrowness_files: &BTreeSet<String>,
) -> Result<Validated, Error> {
    validate_source_parts_with_cancellation(
        reviewed,
        constitution,
        dependencies,
        narrowness_files,
        None,
    )
}

fn validate_source_parts_with_cancellation(
    reviewed: &TemporalSource,
    constitution: &str,
    dependencies: &BTreeMap<String, Vec<u8>>,
    narrowness_files: &BTreeSet<String>,
    cancellation: Option<&CancellationToken>,
) -> Result<Validated, Error> {
    ensure_execution_active(cancellation)?;
    if reviewed.spdx != "CC-BY-4.0" || reviewed.schema_version != 1 {
        return Err(temporal_error(
            "reviewed source must be CC-BY-4.0 schema version 1",
        ));
    }
    if reviewed.status != "staged_t3_repository_assurance" {
        return Err(temporal_error("unexpected assurance status"));
    }
    if reviewed.evidence_role != "current_verified_narrowly" {
        return Err(temporal_error(
            "temporal assurance must remain narrowly current-verified",
        ));
    }
    if !(1..=600).contains(&reviewed.subprocess_timeout_seconds) {
        return Err(temporal_error(
            "subprocess_timeout_seconds must be an integer from 1 to 600",
        ));
    }
    text_value(&reviewed.title, "title")?;
    let pre_t3_rule = text_value(&reviewed.pre_t3_custody_rule, "pre_t3_custody_rule")?;
    if !pre_t3_rule.contains("-> prisoner($offender).")
        || ["correct(", "precede(", "time(", "ActiveCustody"]
            .into_iter()
            .any(|token| pre_t3_rule.contains(token))
    {
        return Err(temporal_error(
            "pre-T3 custody rule is not the exact non-temporal fallback",
        ));
    }
    if constitution.contains("admits(\"record\")") {
        return Err(temporal_error(
            "generic record admission is forbidden in the narrowed temporal source",
        ));
    }
    if constitution.contains("correct($after, Transition") {
        return Err(temporal_error(
            "transition acceptance must use bounded succeed/2, not recursive correct/4",
        ));
    }
    let unbounded_heads = FORBIDDEN_TEMPORAL_HEADS
        .into_iter()
        .filter(|fragment| constitution.contains(fragment))
        .collect::<Vec<_>>();
    if !unbounded_heads.is_empty() {
        return Err(temporal_error(format!(
            "temporal derived heads violate the reviewed typed-status/path shapes: {}",
            unbounded_heads.join(", ")
        )));
    }
    validate_rule_heads(constitution)?;
    ensure_execution_active(cancellation)?;
    let succeed_closure = "derived_only(\"succeed\").";
    if marker_block(constitution, "T1-DERIVED")?
        .matches(succeed_closure)
        .count()
        != 1
        || marker_block(constitution, "T3-DERIVED")?.contains(succeed_closure)
    {
        return Err(temporal_error(
            "succeed must be derived-only at T1 so every cumulative stage is closed",
        ));
    }
    let missing_sentinels = REQUIRED_SOURCE_SENTINELS
        .iter()
        .filter(|sentinel| !constitution.contains(**sentinel))
        .copied()
        .collect::<Vec<_>>();
    if !missing_sentinels.is_empty() {
        return Err(temporal_error(format!(
            "exact temporal source sentinels missing: {}",
            missing_sentinels.join(", ")
        )));
    }

    let fingerprints = core_fingerprints(constitution, dependencies, pre_t3_rule)?;
    ensure_execution_active(cancellation)?;
    checked_sha(
        &reviewed.constitution_sha256,
        "constitution_sha256",
        Some(&fingerprints.constitution_sha256),
    )?;
    require_map_keys(
        &reviewed.bound_sources_sha256,
        &BOUND_SOURCES.map(|entry| entry.0),
        "bound_sources_sha256 must bind every reviewed dependency",
    )?;
    for (key, actual) in &fingerprints.bound_sources_sha256 {
        checked_sha(
            &reviewed.bound_sources_sha256[key],
            &format!("bound_sources_sha256.{key}"),
            Some(actual),
        )?;
    }
    require_map_keys(
        &reviewed.marker_sha256,
        &ALL_MARKERS,
        "marker_sha256 must bind every staged source marker",
    )?;
    for (marker, actual) in &fingerprints.marker_sha256 {
        checked_sha(
            &reviewed.marker_sha256[marker],
            &format!("marker_sha256.{marker}"),
            Some(actual),
        )?;
    }
    require_map_keys(
        &reviewed.stage_source_sha256,
        &STAGES,
        "stage_source_sha256 must bind T1, T2, and T3",
    )?;
    for (stage, actual) in &fingerprints.stage_source_sha256 {
        checked_sha(
            &reviewed.stage_source_sha256[stage],
            &format!("stage_source_sha256.{stage}"),
            Some(actual),
        )?;
    }

    validate_source_binding(&reviewed.source_effect_binding, constitution)?;
    ensure_execution_active(cancellation)?;
    validate_inputs(&reviewed.temporal_input_contracts)?;
    ensure_execution_active(cancellation)?;
    validate_stages(&reviewed.stages)?;
    ensure_execution_active(cancellation)?;
    let stage_sources = build_stages(constitution, pre_t3_rule)?;
    ensure_execution_active(cancellation)?;
    let case_ids = validate_cases(&reviewed.cases, &stage_sources)?;
    ensure_execution_active(cancellation)?;
    validate_pairs(&reviewed.fresh_process_pairs, &reviewed.cases, &case_ids)?;
    ensure_execution_active(cancellation)?;
    validate_attacks(&reviewed.attacks, &case_ids)?;
    ensure_execution_active(cancellation)?;
    validate_narrowness(&reviewed.narrowness_impacts, narrowness_files)?;
    ensure_execution_active(cancellation)?;
    validate_limits_and_acceptance(reviewed)?;
    ensure_execution_active(cancellation)?;
    Ok(Validated { stage_sources })
}

fn validate_rule_heads(constitution: &str) -> Result<(), Error> {
    for marker in ["T1-RULES", "T2-RULES", "T3-RULES", "T3-COURT-GATE"] {
        for line in marker_block(constitution, marker)?.lines() {
            if !line.starts_with("all ") || !line.contains("->") {
                continue;
            }
            let head = line.rsplit_once("->").expect("contains arrow").1.trim();
            let variables = VARIABLE
                .find_iter(head)
                .map(|match_| match_.as_str())
                .collect::<HashSet<_>>();
            if variables.len() > 1
                && !ALLOWED_EVENT_HEAD.is_match(head)
                && !ALLOWED_RECORD_HEAD.is_match(head)
            {
                return Err(temporal_error(format!(
                    "{marker} has an unreviewed multi-variable head: {head}"
                )));
            }
            if head.starts_with("collide(") {
                let tag = COLLISION_HEAD
                    .captures(head)
                    .and_then(|captures| captures.get(1))
                    .map(|capture| capture.as_str());
                if tag.is_none_or(|tag| !COLLISION_TAGS.contains(&tag)) {
                    return Err(temporal_error(format!(
                        "{marker} has an unreviewed collision head: {head}"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn validate_source_binding(binding: &SourceEffectBinding, constitution: &str) -> Result<(), Error> {
    if binding.effective_version != "Constitution_Temporal" {
        return Err(temporal_error(
            "effective source version must be Constitution_Temporal",
        ));
    }
    let case_fragment = text_value(
        &binding.case_bound_rule_fragment,
        "source_effect_binding.case_bound_rule_fragment",
    )?;
    let source_fragment = text_value(
        &binding.source_binding_fragment,
        "source_effect_binding.source_binding_fragment",
    )?;
    if constitution.matches(case_fragment).count() != 1 {
        return Err(temporal_error(
            "case-bound custody fragment must occur exactly once",
        ));
    }
    if constitution.matches(source_fragment).count() != 1 {
        return Err(temporal_error(
            "effective-source fragment must occur exactly once",
        ));
    }
    checked_sha(
        &binding.case_bound_rule_sha256,
        "case_bound_rule_sha256",
        Some(&sha256(case_fragment.as_bytes())),
    )?;
    checked_sha(
        &binding.source_binding_sha256,
        "source_binding_sha256",
        Some(&sha256(source_fragment.as_bytes())),
    )?;
    let required_case_tokens = [
        "$case",
        "$renewal",
        "cite(Court,",
        "ConvictionRecorded",
        "JudgmentScope",
        "InjuryVictimScope",
        "authorized($renewal, ActiveCustody, $case)",
        "correct(",
        "ActivePower",
        "prisoner(",
    ];
    if required_case_tokens
        .into_iter()
        .any(|token| !case_fragment.contains(token))
    {
        return Err(temporal_error(
            "custody is not positively bound to a case, renewal, and subject",
        ));
    }
    if ["~correct(", "permits(CourtPower, Court)", "record("]
        .into_iter()
        .any(|token| case_fragment.contains(token))
    {
        return Err(temporal_error(
            "custody may not use an absent or global temporal permission",
        ));
    }
    if [
        "passport(",
        "TemporalLeaseFamily",
        "Constitution_Temporal",
        "$current",
        "~collide($binding, SourceBinding)",
    ]
    .into_iter()
    .any(|token| !source_fragment.contains(token))
    {
        return Err(temporal_error(
            "effective-source binding must use the narrowed family passport relation",
        ));
    }
    for (key, value) in [
        ("semantic_effect", &binding.semantic_effect),
        ("compatibility_review", &binding.compatibility_review),
        ("article9_boundary", &binding.article9_boundary),
    ] {
        text_value(value, &format!("source_effect_binding.{key}"))?;
    }
    Ok(())
}

fn validate_inputs(inputs: &[TemporalInputContract]) -> Result<(), Error> {
    let mut identifiers = BTreeSet::new();
    for entry in inputs {
        let identifier = text_value(&entry.id, "input.id")?;
        if !identifiers.insert(identifier.to_owned()) {
            return Err(temporal_error(format!(
                "duplicate temporal input contract {identifier}"
            )));
        }
        if !STAGES.contains(&entry.stage.as_str()) {
            return Err(temporal_error(format!(
                "input {identifier} has invalid stage"
            )));
        }
        for (field, value) in [
            ("writer", &entry.writer),
            ("evidence", &entry.evidence),
            ("forge_route", &entry.forge_route),
            ("withholding_route", &entry.withholding_route),
            ("correction", &entry.correction),
            ("appeal", &entry.appeal),
            ("cross_epoch_handoff", &entry.cross_epoch_handoff),
            (
                "residual_external_assurance",
                &entry.residual_external_assurance,
            ),
            ("book_boundary", &entry.book_boundary),
        ] {
            text_value(value, &format!("input {identifier}.{field}"))?;
        }
    }
    let required = string_set(REQUIRED_INPUTS);
    if identifiers != required {
        return Err(temporal_error(format!(
            "temporal input contracts differ: {:?}",
            symmetric_difference(&identifiers, &required)
        )));
    }
    Ok(())
}

fn validate_stages(stages: &[Stage]) -> Result<(), Error> {
    if stages
        .iter()
        .map(|stage| stage.id.as_str())
        .collect::<Vec<_>>()
        != STAGES
    {
        return Err(temporal_error("stages must be ordered exactly T1, T2, T3"));
    }
    for (index, stage) in stages.iter().enumerate() {
        let expected = STAGES[..=index]
            .iter()
            .map(|stage| (*stage).to_owned())
            .collect::<Vec<_>>();
        if stage.cumulative != expected {
            return Err(temporal_error(format!(
                "stage {} is not cumulative",
                stage.id
            )));
        }
        text_value(&stage.claim, &format!("stage {}.claim", stage.id))?;
        text_value(
            &stage.does_not_establish,
            &format!("stage {}.does_not_establish", stage.id),
        )?;
    }
    Ok(())
}

fn validate_cases(
    cases: &[Case],
    stage_sources: &BTreeMap<String, String>,
) -> Result<BTreeSet<String>, Error> {
    let mut case_ids = BTreeSet::new();
    let mut sentinels = BTreeMap::new();
    for case in cases {
        let identifier = text_value(&case.id, "case.id")?;
        if !valid_identifier(identifier) || !case_ids.insert(identifier.to_owned()) {
            return Err(temporal_error(format!(
                "invalid or duplicate case id {identifier}"
            )));
        }
        if !STAGES.contains(&case.stage.as_str()) {
            return Err(temporal_error(format!(
                "case {identifier} has invalid stage"
            )));
        }
        if !["predecessor", "successor", "attack", "control"].contains(&case.process_role.as_str())
        {
            return Err(temporal_error(format!(
                "case {identifier} has invalid process_role"
            )));
        }
        text_value(&case.title, &format!("case {identifier}.title"))?;
        text_value(&case.description, &format!("case {identifier}.description"))?;
        let deletions =
            validate_ground_facts(&case.deletions, &format!("case {identifier}.deletions"))?;
        let additions =
            validate_ground_facts(&case.additions, &format!("case {identifier}.additions"))?;
        if additions.iter().any(|fact| fact.starts_with("record(")) {
            return Err(temporal_error(format!(
                "case {identifier} reintroduces the refused generic record surface"
            )));
        }
        let case_text = deletions
            .iter()
            .chain(additions.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        let legacy_scopes = FORBIDDEN_CASE_SCOPE_TOKENS
            .into_iter()
            .filter(|scope| case_text.contains(scope))
            .collect::<Vec<_>>();
        if !legacy_scopes.is_empty() {
            return Err(temporal_error(format!(
                "case {identifier} uses legacy aggregate scopes: {}",
                python_debug_strings(&legacy_scopes)
            )));
        }
        let missing_scopes = required_case_scopes(identifier)
            .iter()
            .filter(|scope| !case_text.contains(&format!(", {scope}).")))
            .copied()
            .collect::<Vec<_>>();
        if !missing_scopes.is_empty() {
            return Err(temporal_error(format!(
                "case {identifier} is missing exact-field scopes: {}",
                python_debug_strings(&missing_scopes)
            )));
        }
        if deletions.iter().any(|fact| additions.contains(fact)) {
            return Err(temporal_error(format!(
                "case {identifier} deletes and adds the same fact"
            )));
        }
        let candidate = apply_case(&stage_sources[&case.stage], case)?;
        if case.checks.is_empty() {
            return Err(temporal_error(format!(
                "case {identifier} has no executable checks"
            )));
        }
        let mut expressions = BTreeSet::new();
        for check in &case.checks {
            let expression =
                text_value(&check.expression, &format!("case {identifier}.expression"))?;
            if expression.ends_with('.') || expression.contains('\n') {
                return Err(temporal_error(format!(
                    "case {identifier}: expression must omit final period"
                )));
            }
            if !ground_query(expression) && !EVENT_ENTITLEMENT_QUERY.is_match(expression) {
                return Err(temporal_error(format!(
                    "case {identifier}: temporal checks must be simple ground relation queries or exact ground event-entitlement queries"
                )));
            }
            if expression.contains("Transition")
                && !TRANSITION_QUERY.is_match(expression)
                && !TRANSITION_COLLISION_QUERY.is_match(expression)
            {
                return Err(temporal_error(format!(
                    "case {identifier}: transition checks must use typed succeed/2 or collide/2"
                )));
            }
            if expression.starts_with("collide(") {
                let tag = COLLISION_QUERY
                    .captures(expression)
                    .and_then(|captures| captures.get(1))
                    .map(|capture| capture.as_str());
                if tag.is_none_or(|tag| !COLLISION_TAGS.contains(&tag)) {
                    return Err(temporal_error(format!(
                        "case {identifier}: collide checks must name a reviewed collision tag"
                    )));
                }
            }
            if !["TRUE", "FALSE"].contains(&check.expected.as_str()) {
                return Err(temporal_error(format!(
                    "case {identifier}: invalid expected verdict"
                )));
            }
            text_value(&check.purpose, &format!("case {identifier}.purpose"))?;
            if !expressions.insert(expression.to_owned()) {
                return Err(temporal_error(format!(
                    "case {identifier}: duplicate expression {expression}"
                )));
            }
            sentinels.insert(
                (identifier.to_owned(), expression.to_owned()),
                check.expected.clone(),
            );
        }
        if candidate == stage_sources[&case.stage]
            && (!deletions.is_empty() || !additions.is_empty())
        {
            return Err(temporal_error(format!(
                "case {identifier} overlay made no source change"
            )));
        }
    }
    for (case, expression, expected) in required_boundary_verdicts() {
        let actual = sentinels.get(&(case.to_owned(), expression.to_owned()));
        if actual.map(String::as_str) != Some(expected) {
            return Err(temporal_error(format!(
                "boundary verdict differs for ({case:?}, {expression:?}): {:?} != {expected:?}",
                actual.map(String::as_str)
            )));
        }
    }
    let covered_stages = cases
        .iter()
        .map(|case| case.stage.as_str())
        .collect::<BTreeSet<_>>();
    if cases.len() < 12 || covered_stages != STAGES.into_iter().collect() {
        return Err(temporal_error(
            "cases must cover all stages with a substantial adversarial set",
        ));
    }
    Ok(case_ids)
}

fn validate_pairs(
    pairs: &[FreshProcessPair],
    cases: &[Case],
    case_ids: &BTreeSet<String>,
) -> Result<(), Error> {
    let mut pair_ids = BTreeSet::new();
    let mut paired_cases = BTreeSet::new();
    for pair in pairs {
        let identifier = text_value(&pair.id, "pair.id")?;
        if !pair_ids.insert(identifier.to_owned()) {
            return Err(temporal_error(format!("duplicate pair id {identifier}")));
        }
        let before = text_value(
            &pair.predecessor_case,
            &format!("pair {identifier}.predecessor"),
        )?;
        let after = text_value(
            &pair.successor_case,
            &format!("pair {identifier}.successor"),
        )?;
        if before == after || !case_ids.contains(before) || !case_ids.contains(after) {
            return Err(temporal_error(format!(
                "pair {identifier} must name two distinct cases"
            )));
        }
        paired_cases.insert(before.to_owned());
        paired_cases.insert(after.to_owned());
        text_value(&pair.purpose, &format!("pair {identifier}.purpose"))?;
    }
    let boundary_pairs = pairs
        .iter()
        .filter(|pair| pair.id == "TP-12")
        .collect::<Vec<_>>();
    if boundary_pairs.len() != 1
        || boundary_pairs[0].predecessor_case != "TA-41"
        || boundary_pairs[0].successor_case != "TA-14"
    {
        return Err(temporal_error(
            "challenge-intake fresh-process pair differs",
        ));
    }
    let roles = cases
        .iter()
        .map(|case| (case.id.as_str(), case.process_role.as_str()))
        .collect::<BTreeMap<_, _>>();
    if !paired_cases
        .iter()
        .any(|case| roles[case.as_str()] == "predecessor")
        || !paired_cases
            .iter()
            .any(|case| roles[case.as_str()] == "successor")
    {
        return Err(temporal_error(
            "fresh-process pairs must include predecessor and successor roles",
        ));
    }
    Ok(())
}

fn validate_attacks(attacks: &[Attack], case_ids: &BTreeSet<String>) -> Result<(), Error> {
    let mut attack_ids = BTreeSet::new();
    let mut covered_cases = BTreeSet::new();
    for attack in attacks {
        let identifier = text_value(&attack.id, "attack.id")?;
        if !attack_ids.insert(identifier.to_owned()) {
            return Err(temporal_error(format!("duplicate attack id {identifier}")));
        }
        if !(STAGES.contains(&attack.stage.as_str()) || attack.stage == "external")
            || !POSTURES.contains(&attack.posture.as_str())
        {
            return Err(temporal_error(format!(
                "attack {identifier} has invalid stage or posture"
            )));
        }
        validate_string_list(
            &attack.case_refs,
            &format!("attack {identifier}.case_refs"),
            true,
        )?;
        if attack.case_refs.iter().any(|case| !case_ids.contains(case)) {
            return Err(temporal_error(format!(
                "attack {identifier} names an unknown case"
            )));
        }
        if attack.posture != "exposed_external_boundary" && attack.case_refs.is_empty() {
            return Err(temporal_error(format!(
                "attack {identifier} needs an executable case"
            )));
        }
        covered_cases.extend(attack.case_refs.iter().cloned());
        text_value(&attack.control, &format!("attack {identifier}.control"))?;
        text_value(&attack.boundary, &format!("attack {identifier}.boundary"))?;
    }
    let required_attacks = string_set(REQUIRED_ATTACKS);
    if attack_ids != required_attacks {
        return Err(temporal_error(format!(
            "attack matrix differs: {:?}",
            symmetric_difference(&attack_ids, &required_attacks)
        )));
    }
    let attacks_by_id = attacks
        .iter()
        .map(|attack| (attack.id.as_str(), attack))
        .collect::<BTreeMap<_, _>>();
    let standing = attacks_by_id["standing_witness_withholding"];
    if standing.stage != "T1"
        || standing.case_refs != ["TA-10"]
        || standing.posture != "exposed_external_boundary"
    {
        return Err(temporal_error(
            "boundary attack policy differs for standing_witness_withholding",
        ));
    }
    let challenge = attacks_by_id["challenge_intake_withholding"];
    if challenge.stage != "T3"
        || challenge.case_refs != ["TA-41", "TA-14"]
        || challenge.posture != "exposed_external_boundary"
    {
        return Err(temporal_error(
            "boundary attack policy differs for challenge_intake_withholding",
        ));
    }
    if !case_ids.is_subset(&covered_cases) {
        return Err(temporal_error(format!(
            "cases missing from attack matrix: {:?}",
            case_ids.difference(&covered_cases).collect::<Vec<_>>()
        )));
    }
    Ok(())
}

fn validate_narrowness(
    impacts: &[NarrownessImpact],
    narrowness_files: &BTreeSet<String>,
) -> Result<(), Error> {
    let mut impact_refs = BTreeSet::new();
    for impact in impacts {
        let reference = text_value(&impact.artifact_ref, "narrowness.artifact_ref")?;
        if !impact_refs.insert(reference.to_owned()) {
            return Err(temporal_error(format!(
                "duplicate narrowness artifact {reference}"
            )));
        }
        if !narrowness_files.contains(reference) {
            return Err(temporal_error(format!(
                "narrowness artifact does not exist: {reference}"
            )));
        }
        if !CLASSIFICATIONS.contains(&impact.classification.as_str()) {
            return Err(temporal_error(format!(
                "invalid narrowness classification for {reference}"
            )));
        }
        for (field, value) in [
            ("standing_claim", &impact.standing_claim),
            ("test_route", &impact.test_route),
            ("change_trigger", &impact.change_trigger),
        ] {
            text_value(value, &format!("narrowness {reference}.{field}"))?;
        }
    }
    let required = string_set(REQUIRED_NARROWNESS);
    if impact_refs != required {
        return Err(temporal_error(format!(
            "narrowness manifest differs: {:?}",
            symmetric_difference(&impact_refs, &required)
        )));
    }
    Ok(())
}

fn validate_limits_and_acceptance(reviewed: &TemporalSource) -> Result<(), Error> {
    require_map_keys(
        &reviewed.limits,
        &LIMIT_KEYS,
        "limits keys differ from reviewed schema",
    )?;
    for key in LIMIT_KEYS {
        text_value(&reviewed.limits[key], &format!("limits.{key}"))?;
    }
    let acceptance = &reviewed.acceptance_result;
    if acceptance.result != "ESTABLISHED FOR SUPPLIED RECORDS; EXTERNAL LIVENESS NOT ESTABLISHED" {
        return Err(temporal_error(
            "acceptance result overstates or changes the reviewed boundary",
        ));
    }
    text_value(&acceptance.claim, "acceptance_result.claim")?;
    text_value(
        &acceptance.does_not_establish,
        "acceptance_result.does_not_establish",
    )?;
    text_value(
        &acceptance.remaining_boundary,
        "acceptance_result.remaining_boundary",
    )?;
    Ok(())
}

#[derive(Clone, Debug)]
struct PreparedCase {
    id: String,
    stage_source: Arc<str>,
    case: Case,
    pin_source: String,
    expected_pins: usize,
    timeout: Duration,
}

impl PreparedCase {
    /// Keep the immutable cumulative stage separate from its overlay. The
    /// current runner materialises the candidate because `crate::pin` accepts
    /// text; a future compiled-base API can consume these two parts directly.
    fn candidate_source(&self) -> Result<String, Error> {
        apply_case(&self.stage_source, &self.case)
    }
}

struct PreparedTemporalStage {
    stage: String,
    engine: PreparedPinEngine,
}

#[derive(Default)]
struct TemporalWorkerState {
    prepared: Option<PreparedTemporalStage>,
}

impl TemporalWorkerState {
    fn prepare_stage(&mut self, stage: &str, source: &str, cancellation: &CancellationToken) {
        if self
            .prepared
            .as_ref()
            .is_some_and(|prepared| prepared.stage == stage)
        {
            return;
        }

        // Drop the prior stage before constructing its replacement. Besides
        // keeping the persistent cache to one entry, this avoids a transient
        // two-engine memory peak on every T1/T2/T3 transition.
        drop(self.prepared.take());
        self.prepared = Some(PreparedTemporalStage {
            stage: stage.to_owned(),
            engine: PreparedPinEngine::new_cancellable(
                &[LoadedSource::new("candidate.nibli", source)],
                cancellation.flag(),
            ),
        });
    }

    fn engine(&self, stage: &str) -> &PreparedPinEngine {
        let prepared = self
            .prepared
            .as_ref()
            .expect("temporal worker stage was prepared");
        debug_assert_eq!(prepared.stage, stage);
        &prepared.engine
    }

    #[cfg(test)]
    fn retained_engine_slots(&self) -> usize {
        usize::from(self.prepared.is_some())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaseDeadlineOutcome {
    Completed,
    TimedOut,
    ExternallyCancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaseDeadlineState {
    Pending,
    Completed,
    TimedOut,
    ExternallyCancelled,
    Stopped,
}

struct CaseDeadlineShared {
    state: Mutex<CaseDeadlineState>,
    changed: Condvar,
}

#[cfg(test)]
#[derive(Clone, Default)]
struct DeadlineThreadTracker {
    active: Arc<std::sync::atomic::AtomicUsize>,
    peak: Arc<std::sync::atomic::AtomicUsize>,
    started: Arc<std::sync::atomic::AtomicUsize>,
    finished: Arc<std::sync::atomic::AtomicUsize>,
    joined: Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(test)]
impl DeadlineThreadTracker {
    fn thread_started(&self) {
        use std::sync::atomic::Ordering;

        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.started.fetch_add(1, Ordering::SeqCst);
        self.peak.fetch_max(active, Ordering::SeqCst);
    }

    fn thread_finished(&self) {
        use std::sync::atomic::Ordering;

        self.active.fetch_sub(1, Ordering::SeqCst);
        self.finished.fetch_add(1, Ordering::SeqCst);
    }

    fn thread_joined(&self) {
        self.joined
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn counts(&self) -> (usize, usize, usize, usize, usize) {
        use std::sync::atomic::Ordering;

        (
            self.active.load(Ordering::SeqCst),
            self.peak.load(Ordering::SeqCst),
            self.started.load(Ordering::SeqCst),
            self.finished.load(Ordering::SeqCst),
            self.joined.load(Ordering::SeqCst),
        )
    }
}

#[cfg(test)]
struct DeadlineThreadGuard(DeadlineThreadTracker);

#[cfg(test)]
impl DeadlineThreadGuard {
    fn new(tracker: DeadlineThreadTracker) -> Self {
        tracker.thread_started();
        Self(tracker)
    }
}

#[cfg(test)]
impl Drop for DeadlineThreadGuard {
    fn drop(&mut self) {
        self.0.thread_finished();
    }
}

/// One joined deadline controller for one actively executing temporal case.
///
/// The controller is created only after the worker's stage engine exists. A
/// normal completion or external cancellation wakes it immediately. It owns a
/// per-case child token, so deadline expiry cannot poison the scheduler's
/// family/job token. `Drop` also wakes and joins it, so a panic cannot detach
/// the timer.
struct CaseDeadline {
    started: Instant,
    deadline: Instant,
    cancellation: CancellationToken,
    shared: Arc<CaseDeadlineShared>,
    handle: Option<JoinHandle<()>>,
    parent_link: Option<CancellationHookGuard>,
    #[cfg(test)]
    tracker: Option<DeadlineThreadTracker>,
}

impl CaseDeadline {
    fn start(timeout: Duration, parent: &CancellationToken) -> Result<Self, Error> {
        #[cfg(not(test))]
        {
            Self::start_inner(timeout, parent)
        }
        #[cfg(test)]
        {
            Self::start_inner(timeout, parent, None)
        }
    }

    #[cfg(test)]
    fn start_tracked(
        timeout: Duration,
        parent: &CancellationToken,
        tracker: DeadlineThreadTracker,
    ) -> Result<Self, Error> {
        Self::start_inner(timeout, parent, Some(tracker))
    }

    fn start_inner(
        timeout: Duration,
        parent: &CancellationToken,
        #[cfg(test)] tracker: Option<DeadlineThreadTracker>,
    ) -> Result<Self, Error> {
        // This is the sole clock origin for both the controller and the
        // caller's elapsed-time classification. It is deliberately sampled
        // after cold engine construction.
        let started = Instant::now();
        let deadline = started.checked_add(timeout).ok_or_else(|| {
            temporal_error("temporal case timeout exceeds the monotonic clock range")
        })?;
        let cancellation = CancellationToken::new();
        let shared = Arc::new(CaseDeadlineShared {
            state: Mutex::new(CaseDeadlineState::Pending),
            changed: Condvar::new(),
        });
        let parent_link = {
            let child = cancellation.clone();
            let shared = Arc::clone(&shared);
            parent.on_cancel(move || {
                let cancel_child = {
                    let mut state = shared
                        .state
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    let cancel_child = *state == CaseDeadlineState::Pending;
                    if cancel_child {
                        *state = CaseDeadlineState::ExternallyCancelled;
                    }
                    shared.changed.notify_all();
                    cancel_child
                };
                if cancel_child {
                    child.cancel();
                }
            })
        };
        let shared_for_thread = Arc::clone(&shared);
        let cancellation_for_thread = cancellation.clone();
        #[cfg(test)]
        let tracker_for_thread = tracker.clone();
        let handle = thread::Builder::new()
            .name("rights-temporal-case-deadline".to_owned())
            .spawn(move || {
                #[cfg(test)]
                let _thread_guard = tracker_for_thread.map(DeadlineThreadGuard::new);
                let mut state = shared_for_thread
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                loop {
                    if *state != CaseDeadlineState::Pending {
                        return;
                    }
                    let now = Instant::now();
                    if now >= deadline {
                        *state = CaseDeadlineState::TimedOut;
                        shared_for_thread.changed.notify_all();
                        drop(state);
                        cancellation_for_thread.cancel();
                        return;
                    }

                    let remaining = deadline.saturating_duration_since(now);
                    (state, _) = shared_for_thread
                        .changed
                        .wait_timeout(state, remaining)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
            })
            .map_err(|error| {
                temporal_error(format!(
                    "cannot start temporal case deadline controller: {error}"
                ))
            })?;
        Ok(Self {
            started,
            deadline,
            cancellation,
            shared,
            handle: Some(handle),
            parent_link: Some(parent_link),
            #[cfg(test)]
            tracker,
        })
    }

    fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// Record the engine's return before doing any teardown that may be
    /// preempted. The supplied instant and the timer share `self.deadline`, so
    /// a completion at the boundary wins over a later controller wake-up.
    fn record_completion(&mut self, completed_at: Instant) -> Duration {
        let elapsed = completed_at.saturating_duration_since(self.started);
        let cancel_child = {
            let mut state = self
                .shared
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let cancel_child = if completed_at <= self.deadline {
                if *state != CaseDeadlineState::ExternallyCancelled
                    && *state != CaseDeadlineState::Stopped
                {
                    *state = CaseDeadlineState::Completed;
                }
                false
            } else if *state == CaseDeadlineState::Pending {
                *state = CaseDeadlineState::TimedOut;
                true
            } else {
                false
            };
            self.shared.changed.notify_all();
            cancel_child
        };
        // Unregister immediately after recording completion. If the parent is
        // cancelled while controller teardown is preempted, it must not turn a
        // completed case's child token into a lasting family cancellation.
        drop(self.parent_link.take());
        if cancel_child {
            self.cancellation.cancel();
        }
        elapsed
    }

    fn stop_if_pending(&self) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *state == CaseDeadlineState::Pending {
            *state = CaseDeadlineState::Stopped;
        }
        self.shared.changed.notify_all();
    }

    fn wake_and_join(&mut self) -> Result<(), Error> {
        self.stop_if_pending();
        drop(self.parent_link.take());
        if let Some(handle) = self.handle.take() {
            let joined = handle.join();
            #[cfg(test)]
            if let Some(tracker) = &self.tracker {
                tracker.thread_joined();
            }
            if joined.is_err() {
                return Err(temporal_error("temporal case deadline controller panicked"));
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<CaseDeadlineOutcome, Error> {
        self.wake_and_join()?;
        let state = *self
            .shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match state {
            CaseDeadlineState::Completed => Ok(CaseDeadlineOutcome::Completed),
            CaseDeadlineState::TimedOut => Ok(CaseDeadlineOutcome::TimedOut),
            CaseDeadlineState::ExternallyCancelled => Ok(CaseDeadlineOutcome::ExternallyCancelled),
            CaseDeadlineState::Pending | CaseDeadlineState::Stopped => Err(temporal_error(
                "temporal case deadline finished before execution completed",
            )),
        }
    }
}

impl Drop for CaseDeadline {
    fn drop(&mut self) {
        let _ = self.wake_and_join();
    }
}

fn execute_cases(
    reviewed: &TemporalSource,
    stage_sources: &BTreeMap<String, String>,
) -> Result<ExecutionReport, Error> {
    execute_cases_with_allocation(
        reviewed,
        stage_sources,
        crate::scheduler::configured_workers()?,
        CancellationToken::new(),
    )
}

fn execute_cases_with_allocation(
    reviewed: &TemporalSource,
    stage_sources: &BTreeMap<String, String>,
    workers: usize,
    cancellation: CancellationToken,
) -> Result<ExecutionReport, Error> {
    let stages = stage_sources
        .iter()
        .map(|(stage, source)| (stage.clone(), Arc::<str>::from(source.as_str())))
        .collect::<BTreeMap<_, _>>();
    let timeout = Duration::from_secs(reviewed.subprocess_timeout_seconds);
    let work = reviewed
        .cases
        .iter()
        .map(|case| PreparedCase {
            id: case.id.clone(),
            stage_source: stages[&case.stage].clone(),
            case: case.clone(),
            pin_source: case_pin(case),
            expected_pins: case.checks.len(),
            timeout,
        })
        .collect::<Vec<_>>();
    let case_count = work.len();
    let options = temporal_schedule_options(cancellation, timeout);
    let counts = run_bounded_with_state_controlled(
        work,
        workers.min(case_count.max(1)),
        options,
        |_| TemporalWorkerState::default(),
        |_, worker, case, cancellation| {
            if cancellation.is_cancelled() {
                return Err(temporal_error("temporal execution cancelled"));
            }
            let stage = case.case.stage.clone();
            worker.prepare_stage(&stage, &case.stage_source, &cancellation);
            ensure_execution_active(Some(&cancellation))?;
            let prepared = worker.engine(&stage);
            let count = execute_case_with_prepared(&case, prepared, &cancellation)?;
            if cancellation.is_cancelled() {
                return Err(temporal_error("temporal execution cancelled"));
            }
            Ok(count)
        },
    )
    .map_err(map_schedule_error)?;
    Ok(ExecutionReport {
        cases: case_count,
        pins: counts.into_iter().sum(),
    })
}

fn temporal_schedule_options(
    cancellation: CancellationToken,
    _reviewed_case_timeout: Duration,
) -> ScheduleOptions {
    // The persistent scheduler's clock starts when it dispatches a job, which
    // includes cold construction of that worker's stage engine. The reviewed
    // timeout applies only to the case execution measured below, after that
    // engine exists; the scheduler supplies external cancellation only.
    ScheduleOptions::cancelled_by(cancellation)
}

fn execute_case_with_prepared(
    case: &PreparedCase,
    prepared: &PreparedPinEngine,
    cancellation: &CancellationToken,
) -> Result<usize, Error> {
    let deletions = case
        .case
        .deletions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let additions = case
        .case
        .additions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    ensure_execution_active(Some(cancellation))?;
    let mut deadline = CaseDeadline::start(case.timeout, cancellation)?;
    prepared.set_cancel_flag(deadline.cancellation().flag());
    let output = prepared.run_patched_files(
        &deletions,
        &additions,
        &[LoadedSource::new("case.pins.nibli", &case.pin_source)],
        PinOptions {
            cancellation: Some(deadline.cancellation()),
            ..PinOptions::default()
        },
    );
    let elapsed = deadline.record_completion(Instant::now());
    match deadline.finish()? {
        CaseDeadlineOutcome::Completed => {}
        CaseDeadlineOutcome::TimedOut => return Err(case_timeout_error(case)),
        CaseDeadlineOutcome::ExternallyCancelled => {
            ensure_execution_active(Some(cancellation))?;
            return Err(temporal_error("temporal execution cancelled"));
        }
    }
    ensure_execution_active(Some(cancellation))?;
    validate_case_output(case, elapsed, output)
}

fn execute_case(case: PreparedCase) -> Result<usize, Error> {
    let candidate = case.candidate_source()?;
    let started = Instant::now();
    let output = run_pin_files(
        &[LoadedSource::new("candidate.nibli", &candidate)],
        &[LoadedSource::new("case.pins.nibli", &case.pin_source)],
        PinOptions::default(),
    );
    validate_case_output(&case, started.elapsed(), output)
}

fn validate_case_output(
    case: &PreparedCase,
    elapsed: Duration,
    output: crate::pin::RunOutput,
) -> Result<usize, Error> {
    if elapsed > case.timeout {
        return Err(case_timeout_error(case));
    }
    if output.exit_code != 0 {
        let combined = format!("{}{}", output.stdout, output.stderr);
        return Err(temporal_error(format!(
            "case {}: engine exited {}\n{}",
            case.id,
            output.exit_code,
            tail_lines(&combined, 18)
        )));
    }
    if output.pins != case.expected_pins {
        return Err(temporal_error(format!(
            "case {}: expected {} pins, engine reported {}",
            case.id, case.expected_pins, output.pins
        )));
    }
    Ok(output.pins)
}

fn case_timeout_error(case: &PreparedCase) -> Error {
    temporal_error(format!(
        "case {}: timed out after {} seconds",
        case.id,
        case.timeout.as_secs()
    ))
}

fn map_schedule_error(error: ScheduleError<Error>) -> Error {
    match error {
        ScheduleError::JobFailed { source, .. } => source,
        ScheduleError::JobTimedOut { index, timeout } => temporal_error(format!(
            "temporal execution worker {index} exceeded its {timeout:?} timeout"
        )),
        ScheduleError::InvalidWorkerCount => {
            temporal_error("RIGHTS_VERIFY_JOBS must be an integer from 1 through 4")
        }
        ScheduleError::Cancelled => temporal_error("temporal execution cancelled"),
        ScheduleError::WorkerPanicked { index, message } => temporal_error(format!(
            "temporal execution worker {index} panicked: {message}"
        )),
        ScheduleError::WorkerTeardownPanicked { worker, message } => temporal_error(format!(
            "temporal execution worker {worker} teardown panicked: {message}"
        )),
        ScheduleError::CoordinatorLostWorker { active_indices } => temporal_error(format!(
            "temporal execution coordinator lost workers {active_indices:?}"
        )),
    }
}

fn case_pin(case: &Case) -> String {
    let mut lines = vec![
        format!(":expect-pins {}", case.checks.len()),
        format!("# Fresh-process temporal case {}.", case.id),
        String::new(),
    ];
    for check in &case.checks {
        lines.extend([
            format!("# {}", check.purpose),
            format!("? {}.", check.expression),
            format!("# => {}", check.expected),
            String::new(),
        ]);
    }
    lines.join("\n")
}

fn render(reviewed: &TemporalSource, source_path: &str, kb_path: &str) -> String {
    let acceptance = &reviewed.acceptance_result;
    let mut lines = vec![
        format!("<!-- SPDX-License-Identifier: {} -->", reviewed.spdx),
        "<!-- Generated by the native rights-verify temporal-assurance refresh; do not edit. -->"
            .to_owned(),
        String::new(),
        format!("# {}", reviewed.title),
        String::new(),
        "## Verdict and boundary".to_owned(),
        String::new(),
        "**ESTABLISHED FOR SUPPLIED RECORDS; EXTERNAL LIVENESS NOT ESTABLISHED.**"
            .to_owned(),
        String::new(),
        markdown(&acceptance.claim),
        String::new(),
        format!("- Reviewed source: {}.", code(source_path)),
        format!(
            "- Constitution: {} at {}.",
            code(kb_path),
            code(&reviewed.constitution_sha256)
        ),
        "- Every executable case uses a fresh engine process; paired labels describe a reviewed differential, not shared engine state."
            .to_owned(),
        String::new(),
        "## Cumulative formal stages".to_owned(),
        String::new(),
        "| stage | cumulative source | bounded claim | boundary |".to_owned(),
        "| --- | --- | --- | --- |".to_owned(),
    ];
    for stage in &reviewed.stages {
        let cumulative = stage
            .cumulative
            .iter()
            .map(|item| code(item))
            .collect::<Vec<_>>()
            .join(" + ");
        lines.push(format!(
            "| {} | {cumulative} | {} | {} |",
            code(&stage.id),
            markdown(&stage.claim),
            markdown(&stage.does_not_establish)
        ));
    }
    lines.extend([
        String::new(),
        "## Temporal-input contracts".to_owned(),
        String::new(),
        "| input | stage | writer and evidence | attack surface | correction, appeal, and handoff |"
            .to_owned(),
        "| --- | --- | --- | --- | --- |".to_owned(),
    ]);
    for input in &reviewed.temporal_input_contracts {
        let evidence = format!("{} {}", input.writer, input.evidence);
        let attacks = format!(
            "Forge: {} Withhold: {}",
            input.forge_route, input.withholding_route
        );
        let path = format!(
            "{} {} {} {} {}",
            input.correction,
            input.appeal,
            input.cross_epoch_handoff,
            input.residual_external_assurance,
            input.book_boundary
        );
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code(&input.id),
            code(&input.stage),
            markdown(&evidence),
            markdown(&attacks),
            markdown(&path)
        ));
    }
    let binding = &reviewed.source_effect_binding;
    lines.extend([
        String::new(),
        "## Exact source and effect binding".to_owned(),
        String::new(),
        format!(
            "- Effective version label: {}.",
            code(&binding.effective_version)
        ),
        format!(
            "- Case-bound rule SHA-256: {}.",
            code(&binding.case_bound_rule_sha256)
        ),
        format!(
            "- Source-binding rule SHA-256: {}.",
            code(&binding.source_binding_sha256)
        ),
        format!("- Semantic effect: {}", markdown(&binding.semantic_effect)),
        format!(
            "- Compatibility review: {}",
            markdown(&binding.compatibility_review)
        ),
        format!(
            "- Article 9 boundary: {}",
            markdown(&binding.article9_boundary)
        ),
        String::new(),
        "## Fresh-process differential pairs".to_owned(),
        String::new(),
        "| pair | predecessor process | successor process | purpose |".to_owned(),
        "| --- | --- | --- | --- |".to_owned(),
    ]);
    for pair in &reviewed.fresh_process_pairs {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            code(&pair.id),
            code(&pair.predecessor_case),
            code(&pair.successor_case),
            markdown(&pair.purpose)
        ));
    }
    lines.extend([
        String::new(),
        "## Executable cases".to_owned(),
        String::new(),
        "| case | stage | process role | checks | purpose |".to_owned(),
        "| --- | --- | --- | ---: | --- |".to_owned(),
    ]);
    for case in &reviewed.cases {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code(&case.id),
            code(&case.stage),
            code(&case.process_role),
            case.checks.len(),
            markdown(&case.description)
        ));
    }
    lines.extend([
        String::new(),
        "## Adversarial matrix".to_owned(),
        String::new(),
        "| attack | stage | posture | executable cases | control and boundary |".to_owned(),
        "| --- | --- | --- | --- | --- |".to_owned(),
    ]);
    for attack in &reviewed.attacks {
        let references = if attack.case_refs.is_empty() {
            "—".to_owned()
        } else {
            attack
                .case_refs
                .iter()
                .map(|item| code(item))
                .collect::<Vec<_>>()
                .join(", ")
        };
        lines.push(format!(
            "| {} | {} | {} | {references} | {} {} |",
            code(&attack.id),
            code(&attack.stage),
            code(&attack.posture),
            markdown(&attack.control),
            markdown(&attack.boundary)
        ));
    }
    lines.extend([
        String::new(),
        "## Narrowness-impact manifest".to_owned(),
        String::new(),
    ]);
    for impact in &reviewed.narrowness_impacts {
        lines.extend([
            format!("### {}", code(&impact.artifact_ref)),
            String::new(),
            format!("- **Standing claim:** {}", markdown(&impact.standing_claim)),
            format!("- **Classification:** {}.", code(&impact.classification)),
            format!("- **Test route:** {}", markdown(&impact.test_route)),
            format!("- **Change trigger:** {}", markdown(&impact.change_trigger)),
            String::new(),
        ]);
    }
    lines.extend(["## Residual limits".to_owned(), String::new()]);
    let mut limit_keys = LIMIT_KEYS;
    limit_keys.sort_unstable();
    for key in limit_keys {
        lines.push(format!(
            "- **{}:** {}",
            title_words(&key.replace('_', " ")),
            markdown(&reviewed.limits[key])
        ));
    }
    lines.extend([
        String::new(),
        "## Reproduce".to_owned(),
        String::new(),
        "```bash".to_owned(),
        "./verify.sh --refresh temporal-assurance".to_owned(),
        "./verify.sh --quick".to_owned(),
        "./verify.sh".to_owned(),
        "```".to_owned(),
        String::new(),
        markdown(&acceptance.does_not_establish),
        String::new(),
        format!(
            "**Remaining boundary:** {}",
            markdown(&acceptance.remaining_boundary)
        ),
        String::new(),
    ]);
    lines.join("\n")
}

fn negative_controls(snapshot: &Snapshot) -> Result<usize, Error> {
    negative_controls_with_cancellation(snapshot, None)
}

fn negative_controls_with_cancellation(
    snapshot: &Snapshot,
    cancellation: Option<&CancellationToken>,
) -> Result<usize, Error> {
    ensure_execution_active(cancellation)?;
    let reviewed = &snapshot.reviewed;
    let constitution = &snapshot.constitution;
    let dependencies = &snapshot.dependencies;
    let files = &snapshot.narrowness_files;
    let validate = |candidate: &TemporalSource, source: &str| {
        validate_source_parts_with_cancellation(
            candidate,
            source,
            dependencies,
            files,
            cancellation,
        )
        .map(|_| ())
    };
    let mut controls = 0;

    let mut changed = reviewed.clone();
    changed.constitution_sha256 = "0".repeat(64);
    expect_failure(
        "stale constitution digest",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed
        .bound_sources_sha256
        .insert("time_model".to_owned(), "0".repeat(64));
    expect_failure("stale time-model digest", validate(&changed, constitution))?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.temporal_input_contracts.pop();
    expect_failure("missing input contract", validate(&changed, constitution))?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.stages[1].cumulative = vec!["T2".to_owned()];
    expect_failure("non-cumulative T2", validate(&changed, constitution))?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.cases[0].checks.clear();
    expect_failure("case with no checks", validate(&changed, constitution))?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.cases[0].checks[0].expression = "entitled(Adam, event { eats(Adam) })".to_owned();
    expect_failure(
        "malformed event-abstraction temporal pin",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    case_mut(&mut changed, "TA-02")
        .checks
        .iter_mut()
        .find(|check| check.expression.contains("TerminalTransition"))
        .expect("reviewed terminal check")
        .expression = "succeed(Epoch_TA_2, TerminalTransition, Epoch_TA_1)".to_owned();
    expect_failure(
        "malformed terminal-transition pin",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    let exact = case_mut(&mut changed, "TA-14");
    exact.additions = exact
        .additions
        .iter()
        .map(|fact| fact.replace("RenewalScope", "WindowScope"))
        .collect();
    expect_failure(
        "legacy aggregate case scope",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.fresh_process_pairs[0].successor_case =
        changed.fresh_process_pairs[0].predecessor_case.clone();
    expect_failure("same-process pair alias", validate(&changed, constitution))?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed
        .attacks
        .retain(|attack| attack.id != "frozen_transition");
    expect_failure(
        "missing frozen-transition attack",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    case_mut(&mut changed, "TA-10")
        .checks
        .iter_mut()
        .find(|check| check.expression == "person(TA_Unwitnessed_Standing)")
        .expect("reviewed standing check")
        .expected = "TRUE".to_owned();
    expect_failure(
        "reversed standing-witness polarity",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    pair_mut(&mut changed, "TP-12").successor_case = "TA-22".to_owned();
    expect_failure(
        "challenge-intake pair drift",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    attack_mut(&mut changed, "challenge_intake_withholding").case_refs = vec!["TA-41".to_owned()];
    expect_failure(
        "challenge-intake attack loses withheld case",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.narrowness_impacts.pop();
    expect_failure(
        "pruned narrowness manifest",
        validate(&changed, constitution),
    )?;
    controls += 1;

    let mut changed = reviewed.clone();
    changed.acceptance_result.result = "ESTABLISHED".to_owned();
    expect_failure("liveness overclaim", validate(&changed, constitution))?;
    controls += 1;

    ensure_execution_active(cancellation)?;
    let candidate = constitution.replacen(
        &reviewed.source_effect_binding.case_bound_rule_fragment,
        "",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "missing case-bound source fragment",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    let candidate = constitution.replacen(
        &reviewed.source_effect_binding.source_binding_fragment,
        "",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "missing effective-source fragment",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    let candidate = constitution.replacen(
        "match($x, CarriedVoid) & match($x, CarriedClear) -> err($x, StatusConflict)",
        "",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "missing status-conflict reader",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    let visibility_fragment = "all $lease: all $case: all $subject: authorized($lease, ActiveCustody, $case) & related($case, CaseBound) & cite(Court, $case, $subject) & observe(Chronicle, $case, $subject, CaseScope) & observe(TemporalReview, $case, $subject, CaseScope) & observe(Chronicle, $case, Court, HolderScope) & observe(TemporalReview, $case, Court, HolderScope) & ~match($lease, ActivePower) -> err($subject, TemporalAuthority).";
    let candidate = constitution.replacen(visibility_fragment, "", 1);
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "missing inactive-authority reader",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    let candidate = constitution.replacen(
        "-> succeed($after, Transition).",
        "-> correct($after, Transition, $before, TemporalStandard).",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "recursive transition conclusion",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    let candidate = constitution.replacen(
        "# <T1-RULES-END>",
        "all $after: succeed($after, Transition) -> collide($after, UntypedCollision).\n# <T1-RULES-END>",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure("unreviewed collide head", validate(&changed, &candidate))?;
    controls += 1;

    let candidate = constitution.replacen(
        "# <T2-RULES-END>",
        "all $first: all $middle: all $last: precede($first, $middle, EventPath) & precede($middle, $last, EventPath) -> precede($first, $middle, $last, EventPath).\n# <T2-RULES-END>",
        1,
    );
    let changed = rebound_source(reviewed, &candidate, dependencies)?;
    ensure_execution_active(cancellation)?;
    expect_failure(
        "unreviewed multi-variable path head",
        validate(&changed, &candidate),
    )?;
    controls += 1;

    ensure_execution_active(cancellation)?;
    expect_failure(
        "missing T2 marker",
        marker_block(
            &constitution.replacen("# <T2-RULES-BEGIN>\n", "", 1),
            "T2-RULES",
        ),
    )?;
    controls += 1;
    ensure_execution_active(cancellation)?;
    Ok(controls)
}

fn rebound_source(
    reviewed: &TemporalSource,
    candidate: &str,
    dependencies: &BTreeMap<String, Vec<u8>>,
) -> Result<TemporalSource, Error> {
    let mut changed = reviewed.clone();
    let fingerprints = core_fingerprints(candidate, dependencies, &reviewed.pre_t3_custody_rule)?;
    changed.constitution_sha256 = fingerprints.constitution_sha256;
    changed.bound_sources_sha256 = fingerprints.bound_sources_sha256;
    changed.marker_sha256 = fingerprints.marker_sha256;
    changed.stage_source_sha256 = fingerprints.stage_source_sha256;
    Ok(changed)
}

fn expect_failure<T>(label: &str, result: Result<T, Error>) -> Result<(), Error> {
    match result {
        Err(error)
            if error.to_string() == "12-temporal-assurance: temporal execution cancelled" =>
        {
            Err(error)
        }
        Err(_) => Ok(()),
        Ok(_) => Err(temporal_error(format!(
            "negative control did not fail: {label}"
        ))),
    }
}

fn case_mut<'a>(source: &'a mut TemporalSource, identifier: &str) -> &'a mut Case {
    source
        .cases
        .iter_mut()
        .find(|case| case.id == identifier)
        .expect("reviewed case exists")
}

fn pair_mut<'a>(source: &'a mut TemporalSource, identifier: &str) -> &'a mut FreshProcessPair {
    source
        .fresh_process_pairs
        .iter_mut()
        .find(|pair| pair.id == identifier)
        .expect("reviewed pair exists")
}

fn attack_mut<'a>(source: &'a mut TemporalSource, identifier: &str) -> &'a mut Attack {
    source
        .attacks
        .iter_mut()
        .find(|attack| attack.id == identifier)
        .expect("reviewed attack exists")
}

fn required_case_scopes(identifier: &str) -> &'static [&'static str] {
    match identifier {
        "TA-09" | "TA-33" => &["ManifestScope"],
        "TA-13" | "TA-26" | "TA-36" => &[
            "EventStartScope",
            "EventEndScope",
            "RecordStartScope",
            "RecordEndScope",
        ],
        "TA-14" | "TA-16" | "TA-18" | "TA-19" | "TA-41" => &[
            "CaseScope",
            "HolderScope",
            "JudgmentScope",
            "InjuryVictimScope",
            "PowerScope",
            "CaseBindingScope",
            "LimitScope",
            "RenewalScope",
        ],
        "TA-15" => &[
            "CaseScope",
            "HolderScope",
            "JudgmentScope",
            "InjuryVictimScope",
            "PowerScope",
            "CaseBindingScope",
        ],
        "TA-17" => &[
            "CaseScope",
            "HolderScope",
            "JudgmentScope",
            "InjuryVictimScope",
            "PowerScope",
            "CaseBindingScope",
            "LimitScope",
            "RenewalScope",
            "SourceFamilyScope",
            "SourceVersionScope",
            "SourceEpochScope",
        ],
        "TA-25" => &["VoidScope", "ClearScope"],
        "TA-27" => &["PowerScope"],
        "TA-28" => &["HolderScope"],
        "TA-29" => &["SourceEpochScope"],
        "TA-34" => &["SourceVersionScope"],
        "TA-35" => &["WindowEndScope"],
        "TA-37" => &["JudgmentScope"],
        "TA-38" => &["InjuryVictimScope"],
        "TA-39" => &["CaseScope"],
        "TA-40" => &["LineageVersionScope"],
        _ => &[],
    }
}

fn required_boundary_verdicts() -> [(&'static str, &'static str, &'static str); 12] {
    [
        ("TA-10", "person(TA_Unregistered)", "TRUE"),
        ("TA-10", "err(TA_Unregistered, StandingOmission)", "TRUE"),
        ("TA-10", "person(TA_Unwitnessed_Standing)", "FALSE"),
        (
            "TA-10",
            "err(TA_Unwitnessed_Standing, StandingOmission)",
            "FALSE",
        ),
        (
            "TA-41",
            "collide(TA_Challenge_Case, LeaseSuspended)",
            "TRUE",
        ),
        ("TA-41", "correct(TA_Challenge_Case, ActivePower)", "FALSE"),
        ("TA-41", "match(TA_Challenge_Case, ActivePower)", "FALSE"),
        ("TA-41", "prisoner(TA_Challenge_Subject)", "FALSE"),
        (
            "TA-41",
            "err(TA_Challenge_Subject, TemporalDispute)",
            "TRUE",
        ),
        ("TA-41", "obliged(Appeals, TA_Challenge_Subject)", "TRUE"),
        ("TA-41", "free(TA_Challenge_Subject)", "FALSE"),
        ("TA-41", "person(TA_Challenge_Subject)", "TRUE"),
    ]
}

fn validate_ground_facts<'a>(values: &'a [String], path: &str) -> Result<Vec<&'a str>, Error> {
    validate_string_list(values, path, true)?;
    for fact in values {
        if !ground_fact(fact) || fact.contains('$') || fact.contains("->") {
            return Err(temporal_error(format!(
                "{path}: not a ground fact: {fact:?}"
            )));
        }
    }
    Ok(values.iter().map(String::as_str).collect())
}

fn ground_fact(value: &str) -> bool {
    let Some(body) = value.strip_suffix(".") else {
        return false;
    };
    let Some((head, tail)) = body.split_once('(') else {
        return false;
    };
    valid_relation_name(head) && tail.ends_with(')') && !tail.contains('\n')
}

fn ground_query(value: &str) -> bool {
    let Some((head, tail)) = value.split_once('(') else {
        return false;
    };
    let Some(arguments) = tail.strip_suffix(')') else {
        return false;
    };
    valid_relation_name(head)
        && !arguments.is_empty()
        && arguments
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b',' | b' '))
}

fn valid_relation_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn validate_string_list(values: &[String], path: &str, allow_empty: bool) -> Result<(), Error> {
    if values.is_empty() && !allow_empty {
        return Err(temporal_error(format!("{path} may not be empty")));
    }
    let mut unique = HashSet::new();
    for (index, value) in values.iter().enumerate() {
        text_value(value, &format!("{path}[{index}]"))?;
        if !unique.insert(value) {
            return Err(temporal_error(format!("{path} contains duplicates")));
        }
    }
    Ok(())
}

fn text_value<'a>(value: &'a str, path: &str) -> Result<&'a str, Error> {
    let stripped = value.trim();
    if stripped.is_empty() {
        return Err(temporal_error(format!("{path} must be non-empty text")));
    }
    if [
        "todo",
        "tbd",
        "pending",
        "unknown",
        "n/a",
        "na",
        "placeholder",
    ]
    .contains(&stripped.to_lowercase().as_str())
    {
        return Err(temporal_error(format!("{path} contains a placeholder")));
    }
    Ok(value)
}

fn checked_sha<'a>(value: &'a str, path: &str, expected: Option<&str>) -> Result<&'a str, Error> {
    text_value(value, path)?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(temporal_error(format!(
            "{path} must be a lowercase SHA-256"
        )));
    }
    if expected.is_some_and(|expected| value != expected) {
        return Err(temporal_error(format!(
            "{path} is stale: reviewed {value}, actual {}",
            expected.expect("checked")
        )));
    }
    Ok(value)
}

fn require_map_keys<const N: usize, V>(
    map: &BTreeMap<String, V>,
    expected: &[&str; N],
    message: &str,
) -> Result<(), Error> {
    if map.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected.iter().copied().collect()
    {
        Ok(())
    } else {
        Err(temporal_error(message))
    }
}

fn string_set<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
    values.into_iter().map(str::to_owned).collect()
}

fn symmetric_difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.symmetric_difference(right).cloned().collect()
}

fn python_debug_strings(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("'{value}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn markdown(value: &str) -> String {
    value.replace('|', "\\|")
}

fn code(value: &str) -> String {
    format!("`{}`", value.replace('`', "\\`"))
}

fn title_words(value: &str) -> String {
    value
        .split(' ')
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase(),
                    characters.as_str().to_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn tail_lines(value: &str, count: usize) -> String {
    let lines = value.lines().collect::<Vec<_>>();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

fn write_output(path: &Path, value: &str) -> Result<(), Error> {
    use std::io::Write;

    if path.is_symlink() {
        return Err(temporal_error("generated report may not be a symlink"));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(value.as_bytes())?;
        temporary.flush()?;
        temporary
            .persist(path)
            .map_err(|error| temporal_error(error.error.to_string()))?;
    }
    Ok(())
}

fn sort_json(value: &mut Value) {
    match value {
        Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (_, child) in &mut entries {
                sort_json(child);
            }
            object.extend(entries);
        }
        Value::Array(array) => {
            for child in array {
                sort_json(child);
            }
        }
        _ => {}
    }
}

struct JsonSeed;

impl<'de> DeserializeSeed<'de> for JsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonSeed.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(JsonSeed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, map.next_value_seed(JsonSeed)?);
        }
        Ok(Value::Object(values))
    }
}

fn parse_json_no_duplicates(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = JsonSeed.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

fn temporal_error(message: impl Into<String>) -> Error {
    Error::new(format!("12-temporal-assurance: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    fn snapshot() -> Snapshot {
        load_snapshot(&context(), &Paths::default(), true).expect("load temporal snapshot")
    }

    #[test]
    fn pre_cancelled_execution_stops_before_reading_inputs() {
        let temporary = tempfile::tempdir().expect("temporary temporal repository");
        let isolated = Context::from_test_root(temporary.path().to_path_buf());
        let cancellation = CancellationToken::new();
        cancellation.cancel();

        let error = check_execute_with_allocation(&isolated, 1, cancellation)
            .expect_err("pre-cancelled execution must stop before loading inputs");
        assert_eq!(
            error.to_string(),
            "12-temporal-assurance: temporal execution cancelled"
        );
    }

    #[test]
    fn cold_worker_state_construction_is_outside_the_reviewed_case_timeout() {
        let timeout = Duration::from_millis(50);
        let setup_started = Instant::now();
        let result = run_bounded_with_state_controlled(
            [()],
            1,
            temporal_schedule_options(CancellationToken::new(), timeout),
            move |_| {
                std::thread::sleep(Duration::from_millis(75));
                setup_started.elapsed()
            },
            move |_, setup_elapsed, _, cancellation| {
                if *setup_elapsed <= timeout {
                    Err("cold worker-state construction did not exceed the test timeout")
                } else {
                    let mut deadline = CaseDeadline::start(timeout, &cancellation)
                        .map_err(|_| "could not start case deadline")?;
                    deadline.record_completion(Instant::now());
                    if deadline
                        .finish()
                        .map_err(|_| "could not join case deadline")?
                        == CaseDeadlineOutcome::Completed
                        && !cancellation.is_cancelled()
                    {
                        Ok(())
                    } else {
                        Err("cold setup consumed the case deadline")
                    }
                }
            },
        )
        .expect("cold worker-state construction must not consume case timeout");
        assert_eq!(result, [()]);
    }

    #[test]
    fn reviewed_case_deadline_actively_cancels_and_joins_cooperative_work() {
        let parent = CancellationToken::new();
        let deadline =
            CaseDeadline::start(Duration::from_millis(15), &parent).expect("start case deadline");
        let cancellation = deadline.cancellation().clone();
        let started = Instant::now();
        while !cancellation.is_cancelled() {
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "deadline did not actively cancel cooperative execution"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            deadline.finish().expect("join case deadline"),
            CaseDeadlineOutcome::TimedOut
        );
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(
            !parent.is_cancelled(),
            "case timeout must not cancel its parent family/job token"
        );
    }

    #[test]
    fn external_cancellation_wakes_long_deadline_and_stays_distinct() {
        let parent = CancellationToken::new();
        let deadline = CaseDeadline::start(Duration::from_secs(600), &parent)
            .expect("start long case deadline");
        let child = deadline.cancellation().clone();
        let started = Instant::now();
        assert!(parent.cancel());
        assert_eq!(
            deadline
                .finish()
                .expect("join externally cancelled deadline"),
            CaseDeadlineOutcome::ExternallyCancelled
        );
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(child.is_cancelled());
        assert_eq!(
            ensure_execution_active(Some(&parent))
                .expect_err("external cancellation remains visible")
                .to_string(),
            "12-temporal-assurance: temporal execution cancelled"
        );
    }

    #[test]
    fn completion_before_deadline_wins_during_delayed_teardown() {
        let timeout = Duration::from_millis(250);
        let parent = CancellationToken::new();
        let mut deadline = CaseDeadline::start(timeout, &parent).expect("start case deadline");
        let child = deadline.cancellation().clone();

        // Capture the instant at which the engine returned, then simulate
        // preemption before that completion can acquire the controller state.
        let completed_at = Instant::now();
        let wait_started = Instant::now();
        while !child.is_cancelled() {
            assert!(
                wait_started.elapsed() < Duration::from_secs(2),
                "deadline controller did not cancel the case child"
            );
            std::thread::yield_now();
        }
        let elapsed = deadline.record_completion(completed_at);
        assert!(elapsed <= timeout, "test completion missed its deadline");
        assert_eq!(
            deadline.finish().expect("join completed deadline"),
            CaseDeadlineOutcome::Completed
        );
        assert!(
            child.is_cancelled(),
            "test did not reproduce the late timer race"
        );
        assert!(!parent.is_cancelled());
    }

    #[test]
    fn deadline_controller_threads_are_bounded_and_joined_for_w1_through_w4() {
        for workers in 1..=crate::scheduler::MAX_WORKERS {
            let jobs = workers * 2;
            let tracker = DeadlineThreadTracker::default();
            let first_barrier = Arc::new(std::sync::Barrier::new(workers));
            let second_barrier = Arc::new(std::sync::Barrier::new(workers));
            let tracker_for_jobs = tracker.clone();
            let first_barrier_for_jobs = Arc::clone(&first_barrier);
            let second_barrier_for_jobs = Arc::clone(&second_barrier);

            let results = run_bounded_with_state_controlled(
                0..jobs,
                workers,
                ScheduleOptions::default(),
                |_| (),
                move |_, _, _, cancellation| {
                    let mut deadline = CaseDeadline::start_tracked(
                        Duration::from_secs(10),
                        &cancellation,
                        tracker_for_jobs.clone(),
                    )
                    .map_err(|_| "could not start tracked deadline")?;
                    first_barrier_for_jobs.wait();
                    let wait_started = Instant::now();
                    while tracker_for_jobs.counts().0 < workers {
                        if wait_started.elapsed() >= Duration::from_secs(1) {
                            return Err("deadline controller threads did not all start");
                        }
                        std::thread::yield_now();
                    }
                    second_barrier_for_jobs.wait();
                    deadline.record_completion(Instant::now());
                    if deadline
                        .finish()
                        .map_err(|_| "could not join tracked deadline")?
                        == CaseDeadlineOutcome::Completed
                    {
                        Ok(())
                    } else {
                        Err("tracked deadline did not complete")
                    }
                },
            )
            .expect("bounded deadline controller schedule");

            assert_eq!(results.len(), jobs);
            let (active, peak, started, finished, joined) = tracker.counts();
            assert_eq!(active, 0, "W{workers} left a controller thread active");
            assert_eq!(peak, workers, "W{workers} did not exercise its full bound");
            assert_eq!(started, jobs, "W{workers} did not start every controller");
            assert_eq!(finished, jobs, "W{workers} did not finish every controller");
            assert_eq!(joined, jobs, "W{workers} did not join every controller");
        }
    }

    #[test]
    fn interleaved_stages_retain_at_most_one_engine_per_worker() {
        let workers = crate::scheduler::MAX_WORKERS;
        let mut states = (0..workers)
            .map(|_| TemporalWorkerState::default())
            .collect::<Vec<_>>();
        let stages = [
            ("T1", "person(Ara).\n"),
            ("T2", "person(Ara).\nperson(Bea).\n"),
            ("T3", "person(Ara).\nperson(Bea).\nperson(Cai).\n"),
        ];

        for index in 0..(workers * stages.len()) {
            let worker = index % workers;
            let (stage, source) = stages[index % stages.len()];
            let cancellation = CancellationToken::new();
            states[worker].prepare_stage(stage, source, &cancellation);
            assert_eq!(states[worker].retained_engine_slots(), 1);
            assert_eq!(
                states[worker]
                    .prepared
                    .as_ref()
                    .map(|prepared| prepared.stage.as_str()),
                Some(stage)
            );
            assert!(
                states
                    .iter()
                    .map(TemporalWorkerState::retained_engine_slots)
                    .sum::<usize>()
                    <= workers
            );
        }
        assert!(
            states
                .iter()
                .all(|state| state.retained_engine_slots() == 1)
        );
    }

    #[test]
    fn live_structural_check_and_exact_success_text_match_python() {
        let report = check(&context()).expect("live temporal check");
        assert_eq!(report.structural_controls, 23);
        assert_eq!(report.output, DEFAULT_OUTPUT);
        assert_eq!(
            report.to_string(),
            "new-book-plans/temporal-assurance-case.md is current; 23 structural negative controls pass; execution skipped"
        );
    }

    #[test]
    fn renderer_matches_committed_report_byte_for_byte() {
        let snapshot = snapshot();
        let rendered = render(
            &snapshot.reviewed,
            &snapshot.source_relative,
            &snapshot.kb_relative,
        );
        assert!(
            rendered.contains("Generated by the native rights-verify temporal-assurance refresh")
        );
        assert!(rendered.contains("./verify.sh --refresh temporal-assurance"));
        assert!(rendered.contains("./verify.sh --quick"));
        assert!(rendered.lines().any(|line| line == "./verify.sh"));
        assert!(!rendered.contains("python3 "));
        assert_eq!(rendered, snapshot.current_output.expect("committed report"));
    }

    #[test]
    fn all_twenty_three_structural_mutations_fail() {
        assert_eq!(negative_controls(&snapshot()).expect("controls"), 23);
    }

    #[test]
    fn fingerprint_projection_matches_reviewed_live_digests() {
        let report = fingerprints(&context()).expect("fingerprints");
        let value: Value = serde_json::from_str(&report.to_string()).expect("fingerprint JSON");
        let snapshot = snapshot();
        assert_eq!(
            value["constitution_sha256"],
            snapshot.reviewed.constitution_sha256
        );
        assert_eq!(
            value["case_bound_rule_sha256"],
            snapshot
                .reviewed
                .source_effect_binding
                .case_bound_rule_sha256
        );
        assert_eq!(
            value["source_binding_sha256"],
            snapshot
                .reviewed
                .source_effect_binding
                .source_binding_sha256
        );
    }

    #[test]
    fn missing_required_exact_field_scope_is_rejected() {
        let snapshot = snapshot();
        let mut reviewed = snapshot.reviewed.clone();
        let case = case_mut(&mut reviewed, "TA-14");
        case.additions = case
            .additions
            .iter()
            .map(|fact| fact.replace("RenewalScope", "RenewalMissingScope"))
            .collect();
        let error = validate_source_parts(
            &reviewed,
            &snapshot.constitution,
            &snapshot.dependencies,
            &snapshot.narrowness_files,
        )
        .expect_err("scope deletion must fail");
        assert!(error.to_string().contains("missing exact-field scopes"));
    }

    #[test]
    fn duplicate_json_keys_are_rejected_recursively() {
        let error = parse_json_no_duplicates(br#"{"outer":{"stage":"T1","stage":"T2"}}"#)
            .expect_err("duplicate key must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: stage")
        );
    }

    #[test]
    #[ignore = "full 41-case temporal engine suite; run explicitly in release mode"]
    fn live_in_process_execution_runs_all_cases_and_pins() {
        let snapshot = snapshot();
        let validated = validate_source(&snapshot).expect("validated temporal source");
        let execution =
            execute_cases(&snapshot.reviewed, &validated.stage_sources).expect("execute cases");
        assert_eq!(execution.cases, 41);
        assert_eq!(execution.pins, 244);
        assert_eq!(
            Report {
                output: DEFAULT_OUTPUT.to_owned(),
                structural_controls: 23,
                execution: Some(execution),
            }
            .to_string(),
            "new-book-plans/temporal-assurance-case.md is current; 23 structural negative controls pass; 41 fresh processes / 244 pins pass"
        );
    }

    #[test]
    #[ignore = "live temporal engine mutation; run explicitly in release mode"]
    fn flipped_executable_verdict_is_a_finding() {
        let snapshot = snapshot();
        let validated = validate_source(&snapshot).expect("validated temporal source");
        let mut case = snapshot.reviewed.cases[0].clone();
        case.checks[0].expected = if case.checks[0].expected == "TRUE" {
            "FALSE".to_owned()
        } else {
            "TRUE".to_owned()
        };
        let prepared = PreparedCase {
            id: case.id.clone(),
            stage_source: Arc::from(validated.stage_sources[&case.stage].as_str()),
            pin_source: case_pin(&case),
            expected_pins: case.checks.len(),
            timeout: Duration::from_secs(180),
            case,
        };
        let error = execute_case(prepared).expect_err("flipped pin must fail");
        assert!(error.to_string().contains("engine exited 1"));
        assert!(error.to_string().contains("FINDING"));
    }

    #[test]
    fn atomic_writer_preserves_exact_lf_bytes() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let output = directory.path().join("report.md");
        write_output(&output, "alpha\nbeta\n").expect("write report");
        write_output(&output, "gamma\n").expect("replace report");
        assert_eq!(std::fs::read(output).expect("read report"), b"gamma\n");
    }
}
