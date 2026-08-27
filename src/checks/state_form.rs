// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native owner for the FS-CVF-003 state-form verification family.
//!
//! The rule family and aggregate pin inventories are frozen, reviewed inputs.
//! This module validates those bytes, renders the one-line counterfactual,
//! creates lossless execution projections, and runs them without Python or
//! `nibli-pin` subprocesses.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fmt::Write as _;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{FileOutput, LoadedSource, PinOptions, PreparedPinEngine, RunOutput};
use crate::scheduler::{ScheduleError, run_bounded};

pub(crate) const CONSTITUTION_PATH: &str = "new-book-plans/constitution.nibli";
pub(crate) const MAIN_PINS_PATH: &str = "new-book-plans/state-form.pins.nibli";
pub(crate) const COUNTERFACTUAL_PATH: &str =
    "new-book-plans/counterfactual/no-state-form-independent-current-review.nibli";
pub(crate) const COUNTERFACTUAL_PINS_PATH: &str =
    "new-book-plans/counterfactual/no-state-form-independent-current-review.pins.nibli";

const MAIN_HEADER: &str = "# State-form and political-membership family - executable coverage pins";
const COUNTERFACTUAL_HEADER: &str =
    "# Counterfactual: state-form source writer serves as temporal reviewer";
const BEGIN: &str = "# <STATE-FORM-RULES-BEGIN>";
const END: &str = "# <STATE-FORM-RULES-END>";
const CURRENT_REVIEW_GUARD: &str = " & ~($source = $temporal_review)";
const SPDX_HEADER: &str = "# SPDX-License-Identifier: MIT OR Apache-2.0";

pub(crate) const MAIN_SHARD_COUNT: usize = 64;
pub(crate) const COUNTERFACTUAL_SHARD_COUNT: usize = 17;
pub(crate) const MAIN_PIN_COUNT: usize = 391;
pub(crate) const COUNTERFACTUAL_PIN_COUNT: usize = 51;
const GENERIC_MAIN_PIN_COUNT: usize = 335;
const ACCEPTANCE_PIN_COUNT: usize = 56;
const CARD_COUNT: usize = 51;
const RESULT_COUNT: usize = 131;
const AUTHORITY_COUNT: usize = 142;
const STATEMENT_COUNT: usize = 274;

const EXPECTED_MAIN_PINS_SHA256: &str =
    "41c36aa72b5330bd515363bade95ff118492e60d3e8ba76735c6c3aa2bebfbc2";
const EXPECTED_COUNTERFACTUAL_SHA256: &str =
    "7baf7bf7d82526c7d6935f407c2b9f00612bb10a0efa7b8b79eafb7151055b2c";
const EXPECTED_COUNTERFACTUAL_PINS_SHA256: &str =
    "4b4910d71aaa9baa8606900131b95606756bc56d7dd5fc69902bd1da1d351fd5";
const EXPECTED_CONSTITUTION_SHA256: &str =
    "4f09cdb7320c492eba55809df337eab4a4e3a464193b355781ddc9ea04115ace";
const EXPECTED_RULE_BLOCK_SHA256: &str =
    "98ea81f52420e67d994ab32058280f3ae789855208750e2ae7ab1556005e4ab6";
const EXPECTED_RENDERED_BLOCK_SHA256: &str =
    "6e91abf13097850b6d24c8c58eef7425723b9095bf0d2f3a71b1ca83f7b0d3d9";
const EXPECTED_BRANCH_IR_SHA256: &str =
    "3624348425931f9acef19a77a1bb7c840f321d9b892d68fcf6f756c26b1b1522";
const EXPECTED_BYTE_INDEX_SHA256: &str =
    "4ac6bce6eeaf3a337eb7f21c854fa65f86e80577255d54b3c3a882f841338f6b";
const EXPECTED_COUNT_INDEX_SHA256: &str =
    "62bb24cf9134964fc79280e0d4452aaf4b19989dda9ac15217b2af8c2224e664";

const REVIEWED_SEMANTIC_SOURCE: &str = include_str!("../../new-book-plans/state-form-source.json");

const DELEGATION_MARKER: &str = concat!(
    "# FS-CVF-003 executable coverage is delegated to ",
    "new-book-plans/state-form.pins.nibli."
);
const DELEGATED_PIN_PATHS: [&str; 5] = [
    "book-1/01-what-counts-as-evidence.pins.nibli",
    "book-1/02-public-answerability.pins.nibli",
    "book-1/03-who-holds-the-pen.pins.nibli",
    "book-1/09-the-vote-conviction-does-not-take.pins.nibli",
    "book-1/12-changing-the-rules.pins.nibli",
];
const ACCEPTANCE_CASE_IDS: [&str; 19] = [
    "FSACC-001-prisoner-franchise-candidacy",
    "FSACC-002-custody-home-continuity",
    "FSACC-003-nonconventional-residence",
    "FSACC-004-claimant-chosen-multiple-residence",
    "FSACC-005-atomic-move-no-double-no-gap",
    "FSACC-006-adulthood-evidence-continuity",
    "FSACC-007-office-move-continuity",
    "FSACC-008-nonresident-rights",
    "FSACC-009-anti-capture-appointment",
    "FSACC-010-formation-failure",
    "FSACC-011-nonblocking-presidency",
    "FSACC-012-one-return-council",
    "FSACC-013-proportional-certified-assembly",
    "FSACC-014-budget-continuity",
    "FSACC-015-alternate-court-panel",
    "FSACC-016-rights-corridor",
    "FSACC-017-negotiated-secession",
    "FSACC-018-duplicate-submission",
    "FSACC-019-missing-conflicting-certificate",
];

type Field = [String; 2];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticSource {
    schema_version: String,
    branch_ir_sha256: String,
    branches: Vec<Branch>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Branch {
    card: usize,
    key: String,
    fields: Vec<Field>,
    dynamic: bool,
    dynamic_subtype: String,
    authority_holders: Vec<String>,
    authorizations: Vec<[String; 2]>,
    observations: Vec<[String; 4]>,
    marker: String,
    jurisdiction_kind: String,
    legal_scope_kind: String,
    decision_lineage: Option<DecisionLineage>,
}

impl Branch {
    fn power(&self) -> String {
        format!("FSPOW_{:03}", self.card)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DecisionLineage {
    kind: String,
    rationale: String,
    interfaces: Vec<DecisionInterface>,
    upstream_links: Vec<CertificateLink>,
    certificate_set: Field,
    result_certificate: Field,
    certified_result: Field,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DecisionInterface {
    identity: Field,
    configurations: Vec<Field>,
    rosters: Vec<Field>,
    submissions: Vec<Field>,
    outcomes: Vec<Field>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CertificateLink {
    certificate: Field,
    result: Field,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceSnapshot {
    constitution: Arc<str>,
    main_pins: Arc<str>,
    counterfactual: Arc<str>,
    counterfactual_pins: Arc<str>,
    chapter_pins: Arc<[(String, Arc<str>)]>,
}

impl SourceSnapshot {
    pub(crate) fn from_sources(
        constitution: impl Into<Arc<str>>,
        main_pins: impl Into<Arc<str>>,
        counterfactual: impl Into<Arc<str>>,
        counterfactual_pins: impl Into<Arc<str>>,
        chapter_pins: Vec<(String, Arc<str>)>,
    ) -> Self {
        Self {
            constitution: constitution.into(),
            main_pins: main_pins.into(),
            counterfactual: counterfactual.into(),
            counterfactual_pins: counterfactual_pins.into(),
            chapter_pins: chapter_pins.into(),
        }
    }

    pub(crate) fn constitution(&self) -> &str {
        &self.constitution
    }

    pub(crate) fn main_pins(&self) -> &str {
        &self.main_pins
    }

    pub(crate) fn counterfactual(&self) -> &str {
        &self.counterfactual
    }

    pub(crate) fn counterfactual_pins(&self) -> &str {
        &self.counterfactual_pins
    }
}

pub(crate) fn load_snapshot(context: &Context) -> Result<SourceSnapshot, Error> {
    let mut chapter_pins = Vec::new();
    for entry in std::fs::read_dir(context.path("book-1"))? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".pins.nibli") {
            continue;
        }
        let relative = format!("book-1/{name}");
        chapter_pins.push((relative, Arc::from(std::fs::read_to_string(path)?)));
    }
    chapter_pins.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(SourceSnapshot::from_sources(
        context.read(CONSTITUTION_PATH)?,
        context.read(MAIN_PINS_PATH)?,
        context.read(COUNTERFACTUAL_PATH)?,
        context.read(COUNTERFACTUAL_PINS_PATH)?,
        chapter_pins,
    ))
}

#[derive(Clone, Debug)]
struct StateFormError(String);

impl fmt::Display for StateFormError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

type StateFormResult<T> = Result<T, StateFormError>;

fn state_form_error(message: impl Into<String>) -> StateFormError {
    StateFormError(message.into())
}

fn public_error(error: StateFormError) -> Error {
    Error::new(format!("state-form: {error}"))
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ShardPartition {
    Bytes,
    Count,
}

impl ShardPartition {
    pub(crate) fn from_name(name: &str) -> Result<Self, Error> {
        match name {
            "bytes" => Ok(Self::Bytes),
            "count" => Ok(Self::Count),
            _ => Err(Error::usage(format!(
                "unknown state-form shard partition mode: {name:?}"
            ))),
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Bytes => "bytes",
            Self::Count => "count",
        }
    }
}

impl Default for ShardPartition {
    fn default() -> Self {
        Self::Bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedCall {
    name: String,
    args: Vec<String>,
}

#[derive(Clone, Debug)]
struct ParsedRule {
    quantified: Vec<String>,
    body_calls: Vec<ParsedCall>,
    disequalities: Vec<(String, String)>,
    head: ParsedCall,
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

fn canonicalize_json(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_json).collect()),
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            let mut sorted = Map::new();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_json(&map[key]));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

fn semantic_source() -> StateFormResult<&'static SemanticSource> {
    static SOURCE: OnceLock<StateFormResult<SemanticSource>> = OnceLock::new();
    match SOURCE.get_or_init(|| {
        let source: SemanticSource = serde_json::from_str(REVIEWED_SEMANTIC_SOURCE)
            .map_err(|error| state_form_error(format!("invalid state-form semantic IR: {error}")))?;
        let branches = serde_json::to_value(&source.branches)
            .map_err(|error| state_form_error(error.to_string()))?;
        let encoded = serde_json::to_string(&canonicalize_json(&branches))
            .map_err(|error| state_form_error(error.to_string()))?;
        let actual_digest = sha256(encoded.as_bytes());
        if source.schema_version != "state-form-semantic-ir-v1" {
            return Err(state_form_error(format!(
                "state-form semantic IR schema changed: {}",
                source.schema_version
            )));
        }
        if source.branch_ir_sha256 != EXPECTED_BRANCH_IR_SHA256
            || actual_digest != EXPECTED_BRANCH_IR_SHA256
        {
            return Err(state_form_error(format!(
                "state-form branch IR changed: expected {EXPECTED_BRANCH_IR_SHA256}, found {actual_digest}"
            )));
        }
        validate_branch_inventory(&source.branches)?;
        Ok(source)
    }) {
        Ok(source) => Ok(source),
        Err(error) => Err(error.clone()),
    }
}

fn branch_lookup<'a>(
    branches: &'a [Branch],
    number: usize,
    key: &str,
) -> StateFormResult<&'a Branch> {
    let matches = branches
        .iter()
        .filter(|branch| branch.card == number && branch.key == key)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(state_form_error(format!(
            "expected one FSPOW_{number:03}/{key} branch, found {}",
            matches.len()
        )));
    }
    Ok(matches[0])
}

fn declared_branch_role(field: &Field) -> Option<&'static str> {
    const ADDITIONAL_OUTCOME_SCOPES: [&str; 12] = [
        "ActualCollectiveConsentScope",
        "CertifiedFormationFailureScope",
        "CertifiedGovernmentScope",
        "CompleteSecessionSettlementScope",
        "CurrentAssemblyMandateScope",
        "CurrentMandateScope",
        "CurrentPredecessorMandateScope",
        "ExecutiveCompositionScope",
        "FederalAgreementScope",
        "MemberReplacementScope",
        "OfficeTransferRecordScope",
        "RightsAndMinorityReviewScope",
    ];
    if !field[0].starts_with('$') {
        None
    } else if field[1].contains("RosterScope") {
        Some("roster")
    } else if field[1].contains("SubmissionSetScope") {
        Some("submission")
    } else if (field[1].contains("Result") && !field[1].contains("Certificate"))
        || ADDITIONAL_OUTCOME_SCOPES.contains(&field[1].as_str())
    {
        Some("outcome")
    } else {
        None
    }
}

fn validate_declared_role_ownership(branches: &[Branch]) -> StateFormResult<()> {
    let mut rows = Vec::<(usize, &str, &str, &str, &str)>::new();
    for branch in branches.iter().filter(|branch| branch.dynamic) {
        for field in &branch.fields {
            if let Some(role) = declared_branch_role(field) {
                rows.push((
                    branch.card,
                    branch.key.as_str(),
                    role,
                    field[0].as_str(),
                    field[1].as_str(),
                ));
            }
        }
    }
    let counts = ["roster", "submission", "outcome"]
        .into_iter()
        .map(|role| (role, rows.iter().filter(|row| row.2 == role).count()))
        .collect::<BTreeMap<_, _>>();
    let expected_counts = [("roster", 30), ("submission", 6), ("outcome", 61)]
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    if counts != expected_counts {
        return Err(state_form_error(format!(
            "branch-declared role census changed: {counts:?}"
        )));
    }
    let encoded = format!(
        "{}\n",
        serde_json::to_string(&rows).map_err(|error| state_form_error(error.to_string()))?
    );
    const EXPECTED_ROLE_SHA256: &str =
        "d87260a7fde70b35842c39edd825d6ced74b1346fbf51e60f11a9f3322800b66";
    let actual = sha256(encoded.as_bytes());
    if actual != EXPECTED_ROLE_SHA256 {
        return Err(state_form_error(format!(
            "branch-declared role surface changed: expected {EXPECTED_ROLE_SHA256}, found {actual}"
        )));
    }

    let expected_exceptions = [
        (
            41,
            "last_lawful_government_caretaker",
            "outcome",
            "$current_mandate",
            "CurrentMandateScope",
        ),
        (
            44,
            "common_office_transfer",
            "outcome",
            "$predecessor_mandate",
            "CurrentPredecessorMandateScope",
        ),
        (
            44,
            "regional_local_office_transfer",
            "outcome",
            "$predecessor_mandate",
            "CurrentPredecessorMandateScope",
        ),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut used_exceptions = BTreeSet::new();
    for &(number, key, role, value, scope) in &rows {
        let branch = branch_lookup(branches, number, key)?;
        let lineage = branch.decision_lineage.as_ref().ok_or_else(|| {
            state_form_error(format!("{} has no decision lineage", branch.marker))
        })?;
        let field = [value.to_owned(), scope.to_owned()];
        let mut owners = lineage
            .interfaces
            .iter()
            .filter(|interface| {
                let category = match role {
                    "roster" => &interface.rosters,
                    "submission" => &interface.submissions,
                    "outcome" => &interface.outcomes,
                    _ => unreachable!("reviewed role"),
                };
                category.contains(&field)
            })
            .map(|interface| format!("interface:{}", interface.identity[0]))
            .collect::<Vec<_>>();
        if role == "outcome" && owners.is_empty() && field == lineage.certified_result {
            owners.push("certified-result".to_owned());
        }
        if role == "outcome" && owners.is_empty() {
            owners.extend(
                lineage
                    .upstream_links
                    .iter()
                    .filter(|link| link.result == field)
                    .map(|link| format!("upstream:{}", link.certificate[0])),
            );
        }
        let exception = (number, key, role, value, scope);
        if owners.is_empty() && expected_exceptions.contains(&exception) {
            used_exceptions.insert(exception);
        } else if owners.len() != 1 {
            return Err(state_form_error(format!(
                "FSPOW_{number:03}/{key} must assign declared {role} {value}@{scope} to one primary owner; found {owners:?}"
            )));
        }
    }
    if used_exceptions != expected_exceptions {
        return Err(state_form_error(format!(
            "declared-role exceptions changed or became unnecessary: used={used_exceptions:?}"
        )));
    }
    Ok(())
}

fn validate_decision_lineage_manifest(branches: &[Branch]) -> StateFormResult<()> {
    validate_declared_role_ownership(branches)?;
    let dynamic = branches
        .iter()
        .filter(|branch| branch.dynamic)
        .collect::<Vec<_>>();
    if dynamic.len() != 57
        || dynamic
            .iter()
            .any(|branch| branch.decision_lineage.is_none())
    {
        return Err(state_form_error(
            "decision-lineage manifest keys differ from the 57 dynamic identities",
        ));
    }
    let kind_counts = ["collective-result", "record-certificate-consumption"]
        .into_iter()
        .map(|kind| {
            (
                kind,
                dynamic
                    .iter()
                    .filter(|branch| {
                        branch
                            .decision_lineage
                            .as_ref()
                            .is_some_and(|lineage| lineage.kind == kind)
                    })
                    .count(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if kind_counts
        != [
            ("collective-result", 50),
            ("record-certificate-consumption", 7),
        ]
        .into_iter()
        .collect()
    {
        return Err(state_form_error(format!(
            "decision-lineage kind census changed: {kind_counts:?}"
        )));
    }
    let record_identities = dynamic
        .iter()
        .filter(|branch| {
            branch
                .decision_lineage
                .as_ref()
                .is_some_and(|lineage| lineage.kind == "record-certificate-consumption")
        })
        .map(|branch| (branch.card, branch.key.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_record_identities = [
        (18, "formal_government_appointment"),
        (33, "executive_composition"),
        (33, "executive_member_replacement"),
        (41, "last_lawful_government_caretaker"),
        (44, "common_office_transfer"),
        (44, "regional_local_office_transfer"),
        (45, "completed_negotiation"),
    ]
    .into_iter()
    .collect();
    if record_identities != expected_record_identities {
        return Err(state_form_error(
            "record/certificate-consumption classification changed",
        ));
    }

    let mut zero_voter_identities = BTreeSet::new();
    for branch in dynamic {
        let identity = (branch.card, branch.key.as_str());
        let lineage = branch.decision_lineage.as_ref().expect("validated lineage");
        if lineage.rationale.is_empty() {
            return Err(state_form_error(format!(
                "{identity:?} lacks a lineage rationale"
            )));
        }
        if lineage
            .interfaces
            .iter()
            .map(|interface| &interface.identity)
            .collect::<HashSet<_>>()
            .len()
            != lineage.interfaces.len()
        {
            return Err(state_form_error(format!(
                "{identity:?} repeats an interface identity"
            )));
        }
        let roster_count = lineage
            .interfaces
            .iter()
            .map(|interface| interface.rosters.len())
            .sum::<usize>();
        let submission_count = lineage
            .interfaces
            .iter()
            .map(|interface| interface.submissions.len())
            .sum::<usize>();
        if roster_count == 0 || submission_count == 0 {
            zero_voter_identities.insert(identity);
        }
        for interface in &lineage.interfaces {
            if interface.configurations.is_empty()
                || interface.rosters.is_empty()
                || interface.submissions.is_empty()
                || interface.outcomes.is_empty()
            {
                return Err(state_form_error(format!(
                    "{identity:?} has an incomplete decision interface"
                )));
            }
            for category in [
                &interface.configurations,
                &interface.rosters,
                &interface.submissions,
                &interface.outcomes,
            ] {
                if category.iter().collect::<HashSet<_>>().len() != category.len() {
                    return Err(state_form_error(format!(
                        "{identity:?}/{} repeats a term",
                        interface.identity[0]
                    )));
                }
            }
        }
        for (category_name, category) in [
            (
                "rosters",
                lineage
                    .interfaces
                    .iter()
                    .flat_map(|interface| &interface.rosters)
                    .collect::<Vec<_>>(),
            ),
            (
                "submissions",
                lineage
                    .interfaces
                    .iter()
                    .flat_map(|interface| &interface.submissions)
                    .collect::<Vec<_>>(),
            ),
            (
                "outcomes",
                lineage
                    .interfaces
                    .iter()
                    .flat_map(|interface| &interface.outcomes)
                    .collect::<Vec<_>>(),
            ),
        ] {
            if category.iter().collect::<HashSet<_>>().len() != category.len() {
                return Err(state_form_error(format!(
                    "{identity:?} assigns one {category_name} term twice"
                )));
            }
        }
        let lineage_text = lineage_fields(lineage)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
        if lineage_text.contains("$evidence_set")
            || lineage_text.contains("$decision_configuration")
        {
            return Err(state_form_error(format!(
                "{identity:?} acquired a generic lineage fallback"
            )));
        }
        if lineage_text.contains("$submission_set") && identity != (10, "assembly_election") {
            return Err(state_form_error(format!(
                "{identity:?} acquired an unowned generic submission"
            )));
        }
    }
    if zero_voter_identities
        != [
            (44, "common_office_transfer"),
            (44, "regional_local_office_transfer"),
            (45, "completed_negotiation"),
        ]
        .into_iter()
        .collect()
    {
        return Err(state_form_error(format!(
            "empty-voter lineage set changed: {zero_voter_identities:?}"
        )));
    }
    let certified_government = [
        "$certified_government".to_owned(),
        "CertifiedGovernmentScope".to_owned(),
    ];
    for (number, key) in [
        (15, "confidence_certification"),
        (18, "formal_government_appointment"),
    ] {
        let branch = branch_lookup(branches, number, key)?;
        let lineage = branch.decision_lineage.as_ref().expect("dynamic branch");
        let owners = lineage
            .interfaces
            .iter()
            .filter(|interface| interface.outcomes.contains(&certified_government))
            .count();
        if owners != 1 {
            return Err(state_form_error(format!(
                "({number}, {key:?}) must bind certified government to one exact decision interface; found {owners}"
            )));
        }
    }
    let operative = [
        "initiative_only_wins",
        "counterproposal_only_wins",
        "both_pass_initiative_larger_share",
        "both_pass_counterproposal_larger_share",
    ];
    for branch in branches.iter().filter(|branch| branch.card == 39) {
        let has_positive = branch.fields.iter().any(|field| {
            field[0] == "PositiveCompatibilityAndCorridorReviewPassed"
                && field[1] == "CompatibilityReviewDispositionScope"
        });
        if has_positive != operative.contains(&branch.key.as_str()) {
            return Err(state_form_error(format!(
                "{} compatibility/corridor polarity changed",
                branch.marker
            )));
        }
    }
    Ok(())
}

fn validate_branch_inventory(branches: &[Branch]) -> StateFormResult<()> {
    validate_decision_lineage_manifest(branches)?;
    if branches.len() != RESULT_COUNT {
        return Err(state_form_error(format!(
            "expected {RESULT_COUNT} result branches, found {}",
            branches.len()
        )));
    }
    let authority_count = branches
        .iter()
        .map(|branch| branch.authority_holders.len())
        .sum::<usize>();
    if authority_count != AUTHORITY_COUNT {
        return Err(state_form_error(format!(
            "expected {AUTHORITY_COUNT} authority heads, found {authority_count}"
        )));
    }
    if branches.windows(2).any(|pair| pair[0].card > pair[1].card) {
        return Err(state_form_error(
            "state-form branch cards are not in nondecreasing order",
        ));
    }
    let identities = branches
        .iter()
        .map(|branch| (branch.card, branch.key.as_str()))
        .collect::<Vec<_>>();
    if identities.iter().collect::<HashSet<_>>().len() != identities.len() {
        return Err(state_form_error(
            "state-form branch identities are not unique",
        ));
    }
    if identities
        .iter()
        .map(|(number, _)| *number)
        .collect::<BTreeSet<_>>()
        != (1..=CARD_COUNT).collect()
    {
        return Err(state_form_error(
            "one or more state-form powers has no branch",
        ));
    }
    let markers = branches
        .iter()
        .map(|branch| branch.marker.as_str())
        .collect::<HashSet<_>>();
    let scopes = branches
        .iter()
        .map(|branch| branch.legal_scope_kind.as_str())
        .collect::<HashSet<_>>();
    if markers.len() != branches.len() || scopes.len() != branches.len() {
        return Err(state_form_error(
            "state-form branch marker or authority-scope inventory is not unique",
        ));
    }
    for branch in branches {
        if branch.authority_holders.is_empty()
            || branch
                .authority_holders
                .iter()
                .any(|holder| !holder.starts_with("FSBOD_"))
        {
            return Err(state_form_error(format!(
                "{} has no valid direct-effect holder",
                branch.marker
            )));
        }
        if !matches!(
            branch.dynamic_subtype.as_str(),
            "static" | "collective" | "certificate"
        ) || branch.dynamic != (branch.dynamic_subtype != "static")
            || branch.dynamic != branch.decision_lineage.is_some()
        {
            return Err(state_form_error(format!(
                "{} has inconsistent dynamic metadata",
                branch.marker
            )));
        }
        if let Some(lineage) = &branch.decision_lineage {
            if lineage.kind.is_empty() || lineage.rationale.is_empty() {
                return Err(state_form_error(format!(
                    "{} has incomplete decision lineage metadata",
                    branch.marker
                )));
            }
        }
        if branch.fields.iter().collect::<HashSet<_>>().len() != branch.fields.len() {
            return Err(state_form_error(format!(
                "{} repeats an exact field binding",
                branch.marker
            )));
        }
        if branch.fields.iter().any(|field| {
            field[0].contains("Parameter_")
                || matches!(
                    field[1].as_str(),
                    "SourceTransitionScope" | "CertificateTransitionScope"
                )
        }) {
            return Err(state_form_error(format!(
                "{} contains an opaque or source-transition field",
                branch.marker
            )));
        }
    }
    for (start, end, expected_results, expected_authorities) in [
        (1, 25, 35, 36),
        (26, 35, 47, 49),
        (36, 36, 16, 16),
        (37, 51, 33, 41),
    ] {
        let actual_results = branches
            .iter()
            .filter(|branch| (start..=end).contains(&branch.card))
            .count();
        let actual_authorities = branches
            .iter()
            .filter(|branch| (start..=end).contains(&branch.card))
            .map(|branch| branch.authority_holders.len())
            .sum::<usize>();
        if (actual_results, actual_authorities) != (expected_results, expected_authorities) {
            return Err(state_form_error(format!(
                "state-form band {start:03}..{end:03} changed: {actual_results} results, {actual_authorities} authorities"
            )));
        }
    }
    for (holder, selector) in [
        ("FSBOD_01", "people"),
        ("FSBOD_02", "assembly"),
        ("FSBOD_03", "council"),
        ("FSBOD_21", "regional_local"),
    ] {
        let key = format!("{selector}_appointment_selection");
        let branch = branch_lookup(branches, 28, &key)?;
        let actual = branch
            .fields
            .iter()
            .map(|field| (field[0].as_str(), field[1].as_str()))
            .collect::<HashSet<_>>();
        let required = [
            ("$selector_configuration", "SelectorConfigurationScope"),
            ("$qualification_authority", "QualificationAuthorityScope"),
            ("$fallback_configuration", "FallbackConfigurationScope"),
            (holder, "SelectedHolderScope"),
        ];
        if required.iter().any(|field| !actual.contains(field)) {
            return Err(state_form_error(format!(
                "{} lost its source-bound selector interface",
                branch.marker
            )));
        }
    }
    let fs036 = branches
        .iter()
        .filter(|branch| branch.card == 36)
        .map(|branch| branch.key.as_str())
        .collect::<Vec<_>>();
    if fs036
        != [
            "ordinary_resident_membership",
            "accessible_nonconventional_residence",
            "multiple_residences_first_choice",
            "multiple_residences_second_choice",
            "compelled_placement_nonchange",
            "last_uncontested_home_during_dispute",
            "provisional_first_home",
            "atomic_home_transfer",
            "adult_resident_franchise",
            "adult_resident_candidacy",
            "unique_accepted_submission",
            "established_adulthood_continuity",
            "provisional_adulthood_expiring_opportunity",
            "positive_nonresident_disposition",
            "former_resident_return_without_ballot",
            "office_move_continuity",
        ]
    {
        return Err(state_form_error(format!(
            "FSPOW_036 branch set changed: {fs036:?}"
        )));
    }
    for branch in branches.iter().filter(|branch| branch.card == 36) {
        let choice = matches!(
            branch.key.as_str(),
            "multiple_residences_first_choice" | "multiple_residences_second_choice"
        );
        if choice {
            if branch.authorizations
                != [[
                    "$subject".to_owned(),
                    "PoliticalHomeChoiceAuthority".to_owned(),
                ]]
                || branch.observations.len() != 1
                || branch.observations[0][3] != "ChoiceScope"
            {
                return Err(state_form_error(format!(
                    "{} lost claimant choice authority",
                    branch.marker
                )));
            }
        } else if !branch.authorizations.is_empty() || !branch.observations.is_empty() {
            return Err(state_form_error(format!(
                "{} gained an unreviewed special witness",
                branch.marker
            )));
        }
    }
    if branch_lookup(branches, 36, "ordinary_resident_membership")?
        .fields
        .iter()
        .any(|field| field[0].contains("Adult"))
    {
        return Err(state_form_error(
            "political home or membership was gated on adulthood",
        ));
    }
    if branch_lookup(branches, 36, "atomic_home_transfer")?
        .fields
        .iter()
        .any(|field| {
            matches!(
                field[1].as_str(),
                "PriorSubmissionScope" | "EffectiveSubmissionScope"
            )
        })
    {
        return Err(state_form_error(
            "atomic political-home transfer improperly reads a submission",
        ));
    }
    for (number, key, expected) in [
        (32, "finite_delegation_tenure", &["FSBOD_03"][..]),
        (32, "instruction_scope", &["FSBOD_21"]),
        (32, "proportional_replacement", &["FSBOD_21"]),
        (32, "delegation_vacancy_fill", &["FSBOD_21"]),
        (33, "executive_composition", &["FSBOD_02"]),
        (33, "executive_member_replacement", &["FSBOD_04"]),
        (33, "coordinator_incapacity", &["FSBOD_04"]),
        (33, "coordinator_power_boundary", &["FSBOD_04"]),
        (44, "common_office_transfer", &["FSBOD_05"]),
        (44, "regional_local_office_transfer", &["FSBOD_21"]),
        (45, "opening_referendum", &["FSBOD_01"]),
        (45, "completed_negotiation", &["FSBOD_02", "FSBOD_03"]),
        (45, "final_exit_no_collective_impact", &["FSBOD_01"]),
        (
            45,
            "final_exit_with_collective_consent",
            &["FSBOD_01", "FSBOD_21"],
        ),
    ] {
        let actual = &branch_lookup(branches, number, key)?.authority_holders;
        if actual
            .iter()
            .map(String::as_str)
            .ne(expected.iter().copied())
        {
            return Err(state_form_error(format!(
                "({number}, {key:?}) holder mapping changed"
            )));
        }
    }
    let fs045 = branches
        .iter()
        .filter(|branch| branch.card == 45)
        .map(|branch| branch.key.as_str())
        .collect::<Vec<_>>();
    if fs045
        != [
            "opening_referendum",
            "completed_negotiation",
            "final_exit_no_collective_impact",
            "final_exit_with_collective_consent",
        ]
    {
        return Err(state_form_error(format!(
            "FSPOW_045 stage set changed: {fs045:?}"
        )));
    }
    for (key, required) in [
        (
            "opening_referendum",
            &[
                "CompleteUniqueOpeningSubmissions",
                "OpeningAffirmativeExceedsNegative",
                "OpeningReferendumIsNotExit",
            ][..],
        ),
        (
            "completed_negotiation",
            &[
                "OpeningAffirmativeExceedsNegative",
                "PositiveFederalAgreementComplete",
                "PositiveRightsAndMinorityReviewPassed",
                "PositiveSettlementComplete",
            ],
        ),
        (
            "final_exit_no_collective_impact",
            &[
                "OpeningAffirmativeExceedsNegative",
                "PositiveFederalAgreementComplete",
                "PositiveRightsAndMinorityReviewPassed",
                "PositiveSettlementComplete",
                "PositiveFinalAffectedPopulationRatificationPassed",
                "PositiveNoCollectiveTitleOrSovereigntyImpact",
            ],
        ),
        (
            "final_exit_with_collective_consent",
            &[
                "OpeningAffirmativeExceedsNegative",
                "PositiveFederalAgreementComplete",
                "PositiveRightsAndMinorityReviewPassed",
                "PositiveSettlementComplete",
                "PositiveCollectiveTitleOrSovereigntyImpact",
                "PositiveActualCollectiveConsent",
                "PositiveFinalAffectedPopulationRatificationPassed",
            ],
        ),
    ] {
        let actual = branch_lookup(branches, 45, key)?
            .fields
            .iter()
            .map(|field| field[0].as_str())
            .collect::<HashSet<_>>();
        let missing = required
            .iter()
            .filter(|value| !actual.contains(**value))
            .copied()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(state_form_error(format!(
                "FSPOW_045/{key} lacks {missing:?}"
            )));
        }
    }
    for key in ["common_office_transfer", "regional_local_office_transfer"] {
        if branch_lookup(branches, 44, key)?.dynamic_subtype != "certificate" {
            return Err(state_form_error(format!(
                "FSPOW_044/{key} must use the certificate pipeline"
            )));
        }
    }
    if branches
        .iter()
        .any(|branch| branch.dynamic_subtype == "certificate" && branch.card != 44)
    {
        return Err(state_form_error(
            "certificate subtype escaped the FSPOW_044 boundary",
        ));
    }
    validate_explicit_lineage_rule_seams(branches)?;
    validate_explicit_lineage_self_controls(branches)?;
    Ok(())
}

fn push_unique(lines: &mut Vec<String>, value: String) {
    if !lines.contains(&value) {
        lines.push(value);
    }
}

fn extend_unique<I>(lines: &mut Vec<String>, values: I)
where
    I: IntoIterator<Item = String>,
{
    for value in values {
        push_unique(lines, value);
    }
}

fn observed(actors: &[&str], subject: &str, value: &str, scope: &str) -> Vec<String> {
    actors
        .iter()
        .map(|actor| format!("observe(${actor}, {subject}, {value}, {scope})"))
        .collect()
}

fn observed_fields(
    actors: &[&str],
    subject: &str,
    fields: impl IntoIterator<Item = Field>,
) -> Vec<String> {
    let mut result = Vec::new();
    for field in fields {
        result.extend(observed(actors, subject, &field[0], &field[1]));
    }
    result
}

fn distinct(names: &[&str]) -> Vec<String> {
    let mut result = Vec::new();
    for (index, left) in names.iter().enumerate() {
        for right in &names[index + 1..] {
            result.push(format!("~(${left} = ${right})"));
        }
    }
    result
}

fn quantified(names: &[String]) -> String {
    names.iter().map(|name| format!("all ${name}: ")).collect()
}

fn current_rule_premises() -> Vec<String> {
    let mut body = [
        "authorized($source, StateFormSourceAuthority, $record)",
        "authorized($temporal, StateFormTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, StateFormTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, StateFormRecordReviewAuthority, $record)",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for (actors, subject, value, scope) in [
        (
            &["source", "record_review"][..],
            "$record",
            "Constitution_StateForm",
            "SourceFamilyScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "Constitution_StateForm",
            "SourceFamilyScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$temporal_record",
            "TemporalRecordScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$record",
            "StateFormRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$power",
            "PowerScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$power",
            "PowerScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$epoch",
            "SourceEpochScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$epoch",
            "SourceEpochScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "StateFormCurrentSelection",
            "EffectiveSelectionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "StateFormCurrentSelection",
            "EffectiveSelectionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$reconciliation",
            "ReconciliationRecordScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$reconciliation",
            "ReconciliationRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "StateFormRecordReconciled",
            "ReconciliationStatusScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$record",
            "StateFormRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$power",
            "PowerScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$legal_scope",
            "AuthorityScope",
        ),
    ] {
        body.extend(observed(actors, subject, value, scope));
    }
    body.extend(distinct(&[
        "source",
        "temporal",
        "temporal_review",
        "record_review",
    ]));
    body
}

fn render_current_rule() -> String {
    let names = [
        "record",
        "source",
        "temporal",
        "temporal_review",
        "record_review",
        "version",
        "epoch",
        "temporal_record",
        "power",
        "jurisdiction",
        "legal_scope",
        "reconciliation",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    format!(
        "{}{} -> complete($record, StateFormCurrent, $temporal_record).",
        quantified(&names),
        current_rule_premises().join(" & ")
    )
}

fn current_rejoin_premises(branch: &Branch) -> Vec<String> {
    let power = branch.power();
    let mut body = [
        "authorized($source, StateFormSourceAuthority, $record)",
        "authorized($temporal, StateFormTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, StateFormTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, StateFormRecordReviewAuthority, $record)",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for (actors, subject, value, scope) in [
        (
            &["source", "record_review"][..],
            "$record",
            "Constitution_StateForm",
            "SourceFamilyScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "Constitution_StateForm",
            "SourceFamilyScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$temporal_record",
            "TemporalRecordScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$record",
            "StateFormRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            &power,
            "PowerScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            &power,
            "PowerScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            &branch.jurisdiction_kind,
            "JurisdictionKindScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            &branch.jurisdiction_kind,
            "JurisdictionKindScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            &branch.legal_scope_kind,
            "AuthorityScopeKindScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            &branch.legal_scope_kind,
            "AuthorityScopeKindScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$epoch",
            "SourceEpochScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$epoch",
            "SourceEpochScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "StateFormCurrentSelection",
            "EffectiveSelectionScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "StateFormCurrentSelection",
            "EffectiveSelectionScope",
        ),
        (
            &["source", "record_review"][..],
            "$record",
            "$reconciliation",
            "ReconciliationRecordScope",
        ),
        (
            &["temporal", "temporal_review"][..],
            "$temporal_record",
            "$reconciliation",
            "ReconciliationRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "StateFormRecordReconciled",
            "ReconciliationStatusScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$record",
            "StateFormRecordScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            &power,
            "PowerScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            &branch.jurisdiction_kind,
            "JurisdictionKindScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["source", "record_review"][..],
            "$reconciliation",
            &branch.legal_scope_kind,
            "AuthorityScopeKindScope",
        ),
    ] {
        body.extend(observed(actors, subject, value, scope));
    }
    body
}

fn lineage_fields(lineage: &DecisionLineage) -> Vec<Field> {
    let mut fields = Vec::new();
    for interface in &lineage.interfaces {
        for field in std::iter::once(&interface.identity)
            .chain(&interface.configurations)
            .chain(&interface.rosters)
            .chain(&interface.submissions)
            .chain(&interface.outcomes)
        {
            if !fields.contains(field) {
                fields.push(field.clone());
            }
        }
    }
    for link in &lineage.upstream_links {
        for field in [&link.certificate, &link.result] {
            if !fields.contains(field) {
                fields.push(field.clone());
            }
        }
    }
    for field in [
        &lineage.certificate_set,
        &lineage.result_certificate,
        &lineage.certified_result,
    ] {
        if !fields.contains(field) {
            fields.push(field.clone());
        }
    }
    fields
}

fn decision_lineage_premises(branch: &Branch) -> StateFormResult<Vec<String>> {
    let lineage = branch
        .decision_lineage
        .as_ref()
        .ok_or_else(|| state_form_error(format!("{} has no decision lineage", branch.marker)))?;
    let power = branch.power();
    let dynamic_actors = ["admin", "assurer", "service"];
    let all_actors = [
        "source", "evidence", "admin", "assurer", "service", "review",
    ];
    let outcome_actors = ["source", "evidence", "service", "review"];
    let certificate_actors = ["source", "evidence", "assurer", "service", "review"];
    let mut body = [
        "authorized($admin, DecisionAdministrationAuthority, $record)",
        "authorized($assurer, IndependentCompletenessAssuranceAuthority, $record)",
        "authorized($service, ResultServiceAuthority, $record)",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for (subject, value, scope) in [
        ("$record", "Constitution_StateForm", "SourceFamilyScope"),
        ("$record", "$version", "SourceVersionScope"),
        ("$record", "$epoch", "SourceEpochScope"),
        ("$record", "$temporal_record", "TemporalRecordScope"),
        ("$record", power.as_str(), "PowerScope"),
        ("$record", "$jurisdiction", "JurisdictionScope"),
        (
            "$record",
            branch.jurisdiction_kind.as_str(),
            "JurisdictionKindScope",
        ),
        ("$record", "$legal_scope", "AuthorityScope"),
        (
            "$record",
            branch.legal_scope_kind.as_str(),
            "AuthorityScopeKindScope",
        ),
        ("$record", "$reconciliation", "ReconciliationRecordScope"),
        ("$record", "$result", "ResultScope"),
        (
            "$result",
            "$result_reconciliation",
            "ReconciliationRecordScope",
        ),
    ] {
        body.extend(observed(&dynamic_actors, subject, value, scope));
    }
    for interface in &lineage.interfaces {
        body.extend(observed_fields(
            &all_actors,
            "$result",
            std::iter::once(interface.identity.clone()),
        ));
        body.extend(observed_fields(
            &all_actors,
            "$result",
            interface
                .configurations
                .iter()
                .chain(&interface.rosters)
                .chain(&interface.submissions)
                .cloned(),
        ));
        body.extend(observed_fields(
            &outcome_actors,
            "$result",
            interface.outcomes.iter().cloned(),
        ));
        for roster in &interface.rosters {
            body.extend(observed(
                &["assurer", "review"],
                &roster[0],
                "CompleteAndNonzeroEligibleRoster",
                "RosterCompletenessDispositionScope",
            ));
        }
        for submission in &interface.submissions {
            body.extend(observed(
                &["assurer", "review"],
                &submission[0],
                "CompleteUniqueSubmissionSet",
                "SubmissionCompletenessDispositionScope",
            ));
        }
        let interface_terms = interface
            .configurations
            .iter()
            .chain(&interface.rosters)
            .chain(&interface.submissions)
            .chain(&interface.outcomes)
            .cloned()
            .collect::<Vec<_>>();
        body.extend(observed_fields(
            &["service", "review"],
            &interface.identity[0],
            interface_terms.iter().cloned(),
        ));
        body.extend(observed(
            &["service", "review"],
            &lineage.result_certificate[0],
            &interface.identity[0],
            "DecisionInterfaceScope",
        ));
        body.extend(observed_fields(
            &["service", "review"],
            &lineage.result_certificate[0],
            interface_terms,
        ));
    }
    body.extend(observed_fields(
        &all_actors,
        "$result",
        std::iter::once(lineage.certificate_set.clone()),
    ));
    body.extend(observed(
        &["assurer", "review"],
        &lineage.certificate_set[0],
        "CompleteUniqueCertificateSet",
        "CertificateCompletenessDispositionScope",
    ));
    body.extend(observed_fields(
        &certificate_actors,
        "$result",
        std::iter::once(lineage.result_certificate.clone()),
    ));
    body.extend(observed_fields(
        &outcome_actors,
        "$result",
        std::iter::once(lineage.certified_result.clone()),
    ));
    body.extend(observed(
        &["service", "review"],
        &lineage.certificate_set[0],
        &lineage.result_certificate[0],
        "ResultCertificateScope",
    ));
    body.extend(observed(
        &["service", "review"],
        &lineage.result_certificate[0],
        &lineage.certified_result[0],
        "ResultScope",
    ));
    for link in &lineage.upstream_links {
        body.extend(observed_fields(
            &certificate_actors,
            "$result",
            std::iter::once(link.certificate.clone()),
        ));
        body.extend(observed_fields(
            &outcome_actors,
            "$result",
            std::iter::once(link.result.clone()),
        ));
        body.extend(observed(
            &["service", "review"],
            &link.certificate[0],
            &link.result[0],
            "ResultScope",
        ));
        body.extend(observed_fields(
            &["service", "review"],
            &lineage.result_certificate[0],
            [link.certificate.clone(), link.result.clone()],
        ));
    }
    body.extend(observed(
        &["service", "review"],
        "$result",
        "UniqueCertifiedResult",
        "ResultDispositionScope",
    ));
    body.extend(distinct(&dynamic_actors));
    body.extend([
        "~($review = $admin)".to_owned(),
        "~($review = $assurer)".to_owned(),
        "~($review = $service)".to_owned(),
    ]);
    let mut unique = Vec::new();
    extend_unique(&mut unique, body);
    Ok(unique)
}

fn push_variable(names: &mut Vec<String>, name: impl Into<String>) {
    let name = name.into();
    if !names.contains(&name) {
        names.push(name);
    }
}

fn variables_in(text: &str) -> Vec<String> {
    variable_regex()
        .captures_iter(text)
        .map(|capture| capture[1].to_owned())
        .collect()
}

fn branch_variable_names(branch: &Branch, authority_stage: bool) -> StateFormResult<Vec<String>> {
    let mut names = [
        "record",
        "source",
        "temporal",
        "temporal_review",
        "record_review",
        "version",
        "epoch",
        "temporal_record",
        "jurisdiction",
        "legal_scope",
        "reconciliation",
        "result",
        "evidence",
        "review",
        "challenge_record",
        "correction_record",
        "remedy_record",
        "end",
        "result_reconciliation",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    if authority_stage {
        names.push("executor".to_owned());
    }
    if branch.dynamic {
        names.extend(
            ["admin", "assurer", "service"]
                .into_iter()
                .map(str::to_owned),
        );
        let lineage = branch.decision_lineage.as_ref().ok_or_else(|| {
            state_form_error(format!("{} has no decision lineage", branch.marker))
        })?;
        for field in lineage_fields(lineage) {
            if let Some(variable) = field[0].strip_prefix('$') {
                push_variable(&mut names, variable);
            }
        }
    }
    for field in &branch.fields {
        if let Some(variable) = field[0].strip_prefix('$') {
            push_variable(&mut names, variable);
        }
    }
    for authorization in &branch.authorizations {
        for variable in variables_in(&format!("{} {}", authorization[0], authorization[1])) {
            push_variable(&mut names, variable);
        }
    }
    for observation in &branch.observations {
        for variable in variables_in(&observation.join(" ")) {
            push_variable(&mut names, variable);
        }
    }
    Ok(names)
}

fn result_raw_premises(branch: &Branch) -> StateFormResult<Vec<String>> {
    let power = branch.power();
    let result_actors = ["source", "evidence", "review"];
    let mut body = vec!["complete($record, StateFormCurrent, $temporal_record)".to_owned()];
    body.extend(current_rejoin_premises(branch));
    body.extend([
        "authorized($evidence, StateFormEvidenceAuthority, $record)".to_owned(),
        "authorized($review, IndependentStateFormReviewAuthority, $record)".to_owned(),
    ]);
    for (actors, subject, value, scope) in [
        (
            &["evidence", "review"][..],
            "$record",
            "Constitution_StateForm",
            "SourceFamilyScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$version",
            "SourceVersionScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$epoch",
            "SourceEpochScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$temporal_record",
            "TemporalRecordScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            power.as_str(),
            "PowerScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$jurisdiction",
            "JurisdictionScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            branch.jurisdiction_kind.as_str(),
            "JurisdictionKindScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$legal_scope",
            "AuthorityScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            branch.legal_scope_kind.as_str(),
            "AuthorityScopeKindScope",
        ),
        (
            &["evidence", "review"][..],
            "$record",
            "$reconciliation",
            "ReconciliationRecordScope",
        ),
        (&result_actors[..], "$record", "$result", "ResultScope"),
        (
            &result_actors[..],
            "$result",
            branch.marker.as_str(),
            "StateFormBranchScope",
        ),
        (
            &result_actors[..],
            "$result",
            "$challenge_record",
            "ChallengeScope",
        ),
        (
            &result_actors[..],
            "$result",
            "$correction_record",
            "CorrectionScope",
        ),
        (
            &result_actors[..],
            "$result",
            "$remedy_record",
            "RemedyScope",
        ),
        (
            &["source", "review"][..],
            "$result",
            "$end",
            "EndConditionScope",
        ),
    ] {
        body.extend(observed(actors, subject, value, scope));
    }
    body.extend([
        "observe($temporal, $temporal_record, $end, EndConditionScope)".to_owned(),
        "observe($temporal_review, $temporal_record, $end, EndConditionScope)".to_owned(),
    ]);
    let failure_polarity = format!("{power}FailureWithholdsOnly");
    for (value, scope) in [
        ("IndependentReviewComplete", "ReviewDispositionScope"),
        (failure_polarity.as_str(), "FailurePolarityScope"),
        ("$result_reconciliation", "ReconciliationRecordScope"),
    ] {
        body.extend(observed(&["source", "review"], "$result", value, scope));
    }
    body.extend([
        "observe($source, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)",
        "observe($review, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)",
        "observe($source, $result_reconciliation, $result, ResultScope)",
        "observe($review, $result_reconciliation, $result, ResultScope)",
        "observe($source, $result_reconciliation, $record, StateFormRecordScope)",
        "observe($review, $result_reconciliation, $record, StateFormRecordScope)",
        "observe($source, $result_reconciliation, $version, SourceVersionScope)",
        "observe($review, $result_reconciliation, $version, SourceVersionScope)",
    ].into_iter().map(str::to_owned));
    for (value, scope) in [
        (power.as_str(), "PowerScope"),
        ("$jurisdiction", "JurisdictionScope"),
        (branch.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
        ("$legal_scope", "AuthorityScope"),
        (branch.legal_scope_kind.as_str(), "AuthorityScopeKindScope"),
    ] {
        body.extend(observed(
            &["source", "review"],
            "$result_reconciliation",
            value,
            scope,
        ));
    }
    body.extend(distinct(&["source", "evidence", "review"]));
    for field in &branch.fields {
        body.extend(observed(&result_actors, "$result", &field[0], &field[1]));
    }
    for authorization in &branch.authorizations {
        body.push(format!(
            "authorized({}, {}, $record)",
            authorization[0], authorization[1]
        ));
    }
    for observation in &branch.observations {
        body.push(format!(
            "observe({}, {}, {}, {})",
            observation[0], observation[1], observation[2], observation[3]
        ));
    }
    if branch.dynamic {
        body.extend(decision_lineage_premises(branch)?);
    }
    let mut unique = Vec::new();
    extend_unique(&mut unique, body);
    Ok(unique)
}

fn authority_raw_premises(branch: &Branch) -> StateFormResult<Vec<String>> {
    let power = branch.power();
    let mut body = result_raw_premises(branch)?;
    body.extend([
        "authorized($executor, InstitutionalExecutionAuthority, $record)".to_owned(),
        "observe($executor, $record, Constitution_StateForm, SourceFamilyScope)".to_owned(),
        "observe($executor, $record, $version, SourceVersionScope)".to_owned(),
        "observe($executor, $record, $epoch, SourceEpochScope)".to_owned(),
        "observe($executor, $record, $temporal_record, TemporalRecordScope)".to_owned(),
        format!("observe($executor, $record, {power}, PowerScope)"),
        "observe($executor, $record, $jurisdiction, JurisdictionScope)".to_owned(),
        format!(
            "observe($executor, $record, {}, JurisdictionKindScope)",
            branch.jurisdiction_kind
        ),
        "observe($executor, $record, $legal_scope, AuthorityScope)".to_owned(),
        format!(
            "observe($executor, $record, {}, AuthorityScopeKindScope)",
            branch.legal_scope_kind
        ),
        "observe($executor, $record, $reconciliation, ReconciliationRecordScope)".to_owned(),
        "observe($executor, $record, $result, ResultScope)".to_owned(),
        "observe($executor, $result, $end, EndConditionScope)".to_owned(),
        "observe($executor, $result, $result_reconciliation, ReconciliationRecordScope)".to_owned(),
        "~($executor = $source)".to_owned(),
        "~($executor = $evidence)".to_owned(),
        "~($executor = $review)".to_owned(),
    ]);
    for holder in &branch.authority_holders {
        body.extend(observed(
            &["source", "review", "executor"],
            "$record",
            holder,
            "HolderScope",
        ));
    }
    if branch.dynamic {
        body.extend([
            "~($executor = $admin)".to_owned(),
            "~($executor = $assurer)".to_owned(),
            "~($executor = $service)".to_owned(),
        ]);
    }
    let mut unique = Vec::new();
    extend_unique(&mut unique, body);
    Ok(unique)
}

fn validate_explicit_lineage_contract(
    branch: &Branch,
    result_raw: &[String],
) -> StateFormResult<()> {
    if !branch.dynamic {
        return Ok(());
    }
    let atoms = result_raw
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let expected = decision_lineage_premises(branch)?;
    let missing = expected
        .iter()
        .filter(|atom| !atoms.contains(atom.as_str()))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(state_form_error(format!(
            "{} lost explicit decision lineage: {missing:?}",
            branch.marker
        )));
    }
    let combined = result_raw.join("\n");
    if combined.contains("$evidence_set") || combined.contains("$decision_configuration") {
        return Err(state_form_error(format!(
            "{} contains a generic lineage fallback",
            branch.marker
        )));
    }
    if combined.contains("$submission_set")
        && (branch.card, branch.key.as_str()) != (10, "assembly_election")
    {
        return Err(state_form_error(format!(
            "{} contains an unowned submission set",
            branch.marker
        )));
    }
    let declared = branch_variable_names(branch, false)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let used = variable_regex()
        .captures_iter(&combined)
        .map(|capture| capture[1].to_owned())
        .collect::<BTreeSet<_>>();
    if declared != used {
        return Err(state_form_error(format!(
            "{} quantified/used variables differ: unused={:?}, unquantified={:?}",
            branch.marker,
            declared.difference(&used).collect::<Vec<_>>(),
            used.difference(&declared).collect::<Vec<_>>()
        )));
    }
    if branch.card == 39
        && [
            "initiative_only_wins",
            "counterproposal_only_wins",
            "both_pass_initiative_larger_share",
            "both_pass_counterproposal_larger_share",
        ]
        .contains(&branch.key.as_str())
    {
        for actor in ["source", "evidence", "review"] {
            let required = format!(
                "observe(${actor}, $result, PositiveCompatibilityAndCorridorReviewPassed, CompatibilityReviewDispositionScope)"
            );
            if !atoms.contains(required.as_str()) {
                return Err(state_form_error(format!(
                    "{} lost positive compatibility/corridor review",
                    branch.marker
                )));
            }
        }
    }
    Ok(())
}

fn expect_explicit_lineage_failure(
    branch: &Branch,
    result_raw: Vec<String>,
    label: impl fmt::Display,
) -> StateFormResult<()> {
    if validate_explicit_lineage_contract(branch, &result_raw).is_ok() {
        return Err(state_form_error(format!(
            "watched state-form mutation survived: {label}"
        )));
    }
    Ok(())
}

fn remove_atom(atoms: &[String], needle: &str) -> Vec<String> {
    atoms
        .iter()
        .filter(|atom| atom.as_str() != needle)
        .cloned()
        .collect()
}

fn replace_atom(atoms: &[String], needle: &str, replacement: String) -> Vec<String> {
    atoms
        .iter()
        .map(|atom| {
            if atom == needle {
                replacement.clone()
            } else {
                atom.clone()
            }
        })
        .collect()
}

fn require_one_atom(
    branch: &Branch,
    baseline: &[String],
    needle: &str,
    label: &str,
) -> StateFormResult<()> {
    if baseline
        .iter()
        .filter(|atom| atom.as_str() == needle)
        .count()
        != 1
    {
        return Err(state_form_error(format!(
            "{} {label} fixture drifted: {needle}",
            branch.marker
        )));
    }
    Ok(())
}

fn validate_explicit_lineage_rule_seams(branches: &[Branch]) -> StateFormResult<()> {
    if current_rejoin_premises(&branches[0])
        .iter()
        .any(|atom| atom.starts_with("~("))
    {
        return Err(state_form_error(
            "current rejoin duplicated a current-role disequality",
        ));
    }
    for branch in branches {
        let result_raw = result_raw_premises(branch)?;
        let authority_raw = authority_raw_premises(branch)?;
        if !authority_raw.starts_with(&result_raw) {
            return Err(state_form_error(format!(
                "{} authority raw prefix differs",
                branch.marker
            )));
        }
        if !result_raw
            .iter()
            .any(|atom| atom == "complete($record, StateFormCurrent, $temporal_record)")
        {
            return Err(state_form_error(format!(
                "{} lost current-record consumption",
                branch.marker
            )));
        }
        if result_raw.iter().any(|atom| atom.contains("$executor")) {
            return Err(state_form_error(format!(
                "{} result depends on executor",
                branch.marker
            )));
        }
        for field in &branch.fields {
            for actor in ["source", "evidence", "review"] {
                let expected = format!("observe(${actor}, $result, {}, {})", field[0], field[1]);
                if !result_raw.contains(&expected) {
                    return Err(state_form_error(format!(
                        "{} lost exact field witness {expected}",
                        branch.marker
                    )));
                }
            }
        }
        for holder in &branch.authority_holders {
            for actor in ["source", "review", "executor"] {
                let expected = format!("observe(${actor}, $record, {holder}, HolderScope)");
                if !authority_raw.contains(&expected) {
                    return Err(state_form_error(format!(
                        "{} lost holder witness {expected}",
                        branch.marker
                    )));
                }
            }
        }
        if branch.dynamic {
            validate_explicit_lineage_contract(branch, &result_raw)?;
        } else if [
            "DecisionAdministrationAuthority",
            "IndependentCompletenessAssuranceAuthority",
            "ResultServiceAuthority",
        ]
        .iter()
        .any(|name| result_raw.iter().any(|atom| atom.contains(name)))
        {
            return Err(state_form_error(format!(
                "{} static rule leaked dynamic roles",
                branch.marker
            )));
        }
    }
    Ok(())
}

fn validate_explicit_lineage_self_controls(branches: &[Branch]) -> StateFormResult<()> {
    for branch in branches.iter().filter(|branch| branch.dynamic) {
        let lineage = branch.decision_lineage.as_ref().expect("dynamic branch");
        let baseline = result_raw_premises(branch)?;
        validate_explicit_lineage_contract(branch, &baseline)?;

        let mut set_terms = lineage
            .interfaces
            .iter()
            .flat_map(|interface| interface.rosters.iter().chain(&interface.submissions))
            .collect::<Vec<_>>();
        set_terms.push(&lineage.certificate_set);
        for field in set_terms {
            for actor in [
                "source", "evidence", "admin", "assurer", "service", "review",
            ] {
                let needle = format!("observe(${actor}, $result, {}, {})", field[0], field[1]);
                require_one_atom(branch, &baseline, &needle, "set self-control")?;
                expect_explicit_lineage_failure(
                    branch,
                    remove_atom(&baseline, &needle),
                    format!("{} removed {actor} witness for {}", branch.marker, field[0]),
                )?;
                let rebound = needle.replace(&field[0], "MismatchedDecisionSet");
                expect_explicit_lineage_failure(
                    branch,
                    replace_atom(&baseline, &needle, rebound),
                    format!("{} rebound {actor} witness for {}", branch.marker, field[0]),
                )?;
            }
        }

        for interface in &lineage.interfaces {
            for (field, disposition, scope) in interface
                .rosters
                .iter()
                .map(|field| {
                    (
                        field,
                        "CompleteAndNonzeroEligibleRoster",
                        "RosterCompletenessDispositionScope",
                    )
                })
                .chain(interface.submissions.iter().map(|field| {
                    (
                        field,
                        "CompleteUniqueSubmissionSet",
                        "SubmissionCompletenessDispositionScope",
                    )
                }))
            {
                for actor in ["assurer", "review"] {
                    let needle = format!("observe(${actor}, {}, {disposition}, {scope})", field[0]);
                    require_one_atom(branch, &baseline, &needle, "completeness")?;
                    expect_explicit_lineage_failure(
                        branch,
                        remove_atom(&baseline, &needle),
                        format!(
                            "{} removed {actor} completeness for {}",
                            branch.marker, field[0]
                        ),
                    )?;
                }
            }
        }
        for actor in ["assurer", "review"] {
            let needle = format!(
                "observe(${actor}, {}, CompleteUniqueCertificateSet, CertificateCompletenessDispositionScope)",
                lineage.certificate_set[0]
            );
            require_one_atom(branch, &baseline, &needle, "certificate completeness")?;
            expect_explicit_lineage_failure(
                branch,
                remove_atom(&baseline, &needle),
                format!("{} removed {actor} certificate completeness", branch.marker),
            )?;
        }

        for interface in &lineage.interfaces {
            for field in &interface.outcomes {
                let witnesses = [
                    ("source", "$result", "result witness"),
                    ("evidence", "$result", "result witness"),
                    ("service", "$result", "result witness"),
                    ("review", "$result", "result witness"),
                    ("service", interface.identity[0].as_str(), "interface link"),
                    ("review", interface.identity[0].as_str(), "interface link"),
                    (
                        "service",
                        lineage.result_certificate[0].as_str(),
                        "certificate link",
                    ),
                    (
                        "review",
                        lineage.result_certificate[0].as_str(),
                        "certificate link",
                    ),
                ];
                for (actor, subject, label) in witnesses {
                    let needle =
                        format!("observe(${actor}, {subject}, {}, {})", field[0], field[1]);
                    require_one_atom(branch, &baseline, &needle, &format!("outcome {label}"))?;
                    expect_explicit_lineage_failure(
                        branch,
                        remove_atom(&baseline, &needle),
                        format!(
                            "{} removed {actor} outcome {label} for {}",
                            branch.marker, field[0]
                        ),
                    )?;
                    let rebound = format!(
                        "observe(${actor}, {subject}, CrossSwappedOutcome, {})",
                        field[1]
                    );
                    expect_explicit_lineage_failure(
                        branch,
                        replace_atom(&baseline, &needle, rebound),
                        format!(
                            "{} rebound {actor} outcome {label} for {}",
                            branch.marker, field[0]
                        ),
                    )?;
                }
            }
        }

        for actor in ["service", "review"] {
            for (subject, value, scope, label) in [
                (
                    lineage.certificate_set[0].as_str(),
                    lineage.result_certificate[0].as_str(),
                    "ResultCertificateScope",
                    "certificate-set link",
                ),
                (
                    lineage.result_certificate[0].as_str(),
                    lineage.certified_result[0].as_str(),
                    "ResultScope",
                    "certified-result link",
                ),
            ] {
                let needle = format!("observe(${actor}, {subject}, {value}, {scope})");
                require_one_atom(branch, &baseline, &needle, "certificate-link")?;
                expect_explicit_lineage_failure(
                    branch,
                    remove_atom(&baseline, &needle),
                    format!("{} removed {actor} {label}", branch.marker),
                )?;
                let rebound = needle.replace(value, "CrossSwappedDecisionResult");
                expect_explicit_lineage_failure(
                    branch,
                    replace_atom(&baseline, &needle, rebound),
                    format!("{} cross-swapped {actor} {label}", branch.marker),
                )?;
            }
        }
        for link in &lineage.upstream_links {
            for actor in ["service", "review"] {
                let needle = format!(
                    "observe(${actor}, {}, {}, ResultScope)",
                    link.certificate[0], link.result[0]
                );
                require_one_atom(branch, &baseline, &needle, "upstream-link")?;
                expect_explicit_lineage_failure(
                    branch,
                    remove_atom(&baseline, &needle),
                    format!("{} removed upstream certificate link", branch.marker),
                )?;
            }
        }
        if lineage.interfaces.len() > 1 {
            let first = &lineage.interfaces[0];
            let second = &lineage.interfaces[1];
            let needle = format!(
                "observe($service, {}, {}, DecisionInterfaceScope)",
                lineage.result_certificate[0], first.identity[0]
            );
            require_one_atom(branch, &baseline, &needle, "decision-stage")?;
            let rebound = needle.replace(&first.identity[0], &second.identity[0]);
            expect_explicit_lineage_failure(
                branch,
                replace_atom(&baseline, &needle, rebound),
                format!("{} cross-swapped decision stages", branch.marker),
            )?;
        }
        if branch.card == 39
            && [
                "initiative_only_wins",
                "counterproposal_only_wins",
                "both_pass_initiative_larger_share",
                "both_pass_counterproposal_larger_share",
            ]
            .contains(&branch.key.as_str())
        {
            let needle = concat!(
                "observe($source, $result, PositiveCompatibilityAndCorridorReviewPassed, ",
                "CompatibilityReviewDispositionScope)"
            );
            require_one_atom(branch, &baseline, needle, "compatibility review")?;
            expect_explicit_lineage_failure(
                branch,
                remove_atom(&baseline, needle),
                format!("{} omitted positive compatibility review", branch.marker),
            )?;
            expect_explicit_lineage_failure(
                branch,
                replace_atom(
                    &baseline,
                    needle,
                    needle.replace(
                        "PositiveCompatibilityAndCorridorReviewPassed",
                        "AdverseCompatibilityReview",
                    ),
                ),
                format!("{} rebound compatibility review", branch.marker),
            )?;
        }
    }
    Ok(())
}

fn v2_rules_for_branch(branch: &Branch) -> StateFormResult<Vec<String>> {
    let power = branch.power();
    let result_head = format!("complete($result, {power}, $record)");
    let result_rule = format!(
        "{}{} -> {result_head}.",
        quantified(&branch_variable_names(branch, false)?),
        result_raw_premises(branch)?.join(" & ")
    );
    let authority_quantifier = quantified(&branch_variable_names(branch, true)?);
    let authority_body = authority_raw_premises(branch)?.join(" & ");
    let mut rules = vec![result_rule];
    for holder in &branch.authority_holders {
        rules.push(format!(
            "{authority_quantifier}{result_head} & {authority_body} -> authority({holder}, {power}, $record)."
        ));
    }
    Ok(rules)
}

fn draft_rule_block(source: &SemanticSource) -> StateFormResult<Vec<String>> {
    let mut rules = vec![render_current_rule()];
    for branch in &source.branches {
        rules.extend(v2_rules_for_branch(branch)?);
    }
    if rules.len() != STATEMENT_COUNT {
        return Err(state_form_error(format!(
            "expected {STATEMENT_COUNT} statements, found {}",
            rules.len()
        )));
    }
    Ok(rules)
}

fn render_formal_block(source: &SemanticSource) -> StateFormResult<String> {
    let comments = [
        BEGIN,
        "# [2026-08-21] FS-CVF-003. Supplied records only: these rules reuse",
        "# admitted authorized/3 and observe/4, and derived complete/3 and",
        "# authority/3. They add no relation name, arity, admission, or fact.",
        "# The conclusions are bounded legal declarations and authority only:",
        "# never authentication, computation, action, delivery, liveness,",
        "# feasibility, or outside time. A falsely supplied current or reconciled",
        "# attestation remains an external trust-root failure. No producer reads",
        "# a negative predicate, diagnostic conflict, or legacy conclusion.",
    ];
    let mut lines = comments.into_iter().map(str::to_owned).collect::<Vec<_>>();
    lines.extend(draft_rule_block(source)?);
    lines.push(END.to_owned());
    Ok(format!("{}\n", lines.join("\n")))
}

#[derive(Clone, Debug)]
struct AtomSelector<'a> {
    exact: Option<&'a str>,
    all_of: &'a [&'a str],
    any_of: &'a [&'a str],
}

impl<'a> AtomSelector<'a> {
    const fn exact(value: &'a str) -> Self {
        Self {
            exact: Some(value),
            all_of: &[],
            any_of: &[],
        }
    }

    const fn all(values: &'a [&'a str]) -> Self {
        Self {
            exact: None,
            all_of: values,
            any_of: &[],
        }
    }

    const fn any(values: &'a [&'a str]) -> Self {
        Self {
            exact: None,
            all_of: &[],
            any_of: values,
        }
    }

    fn matches(&self, atom: &str) -> bool {
        self.exact.is_none_or(|exact| atom == exact)
            && self.all_of.iter().all(|needle| atom.contains(needle))
            && (self.any_of.is_empty() || self.any_of.iter().any(|needle| atom.contains(needle)))
    }
}

#[derive(Clone, Debug)]
struct GroundedFixture {
    facts: Vec<String>,
    mapping: BTreeMap<String, String>,
}

impl GroundedFixture {
    fn term(&self, variable: &str) -> StateFormResult<&str> {
        self.mapping
            .get(variable)
            .map(String::as_str)
            .ok_or_else(|| state_form_error(format!("fixture has no {variable}")))
    }
}

fn raw_fixture_atoms(branch: &Branch) -> StateFormResult<Vec<String>> {
    let premises = current_rule_premises()
        .into_iter()
        .chain(authority_raw_premises(branch)?);
    let mut atoms = Vec::new();
    for atom in premises {
        let atom = atom.trim();
        if (atom.starts_with("authorized(") || atom.starts_with("observe("))
            && !atoms.iter().any(|existing| existing == atom)
        {
            atoms.push(atom.to_owned());
        }
    }
    Ok(atoms)
}

fn omit_fixture_atoms(
    atoms: Vec<String>,
    selectors: &[AtomSelector<'_>],
) -> StateFormResult<Vec<String>> {
    let mut removed = HashSet::new();
    for selector in selectors {
        let hits = atoms
            .iter()
            .filter(|atom| selector.matches(atom))
            .map(String::as_str)
            .collect::<Vec<_>>();
        if hits.is_empty() {
            return Err(state_form_error(format!(
                "state-form omission selector matched no atom: {selector:?}"
            )));
        }
        if selector.exact.is_some() && hits.len() != 1 {
            return Err(state_form_error(format!(
                "exact state-form omission selector is not unique: {:?}",
                selector.exact
            )));
        }
        removed.extend(hits.into_iter().map(str::to_owned));
    }
    let remaining = atoms
        .into_iter()
        .filter(|atom| !removed.contains(atom))
        .collect::<Vec<_>>();
    if selectors
        .iter()
        .any(|selector| remaining.iter().any(|atom| selector.matches(atom)))
    {
        return Err(state_form_error(
            "state-form omission selector left a matching atom",
        ));
    }
    Ok(remaining)
}

fn fixture_variable_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\$[a-z][a-z0-9_]*").expect("valid fixture regex"))
}

fn fixture_constant(prefix: &str, variable: &str) -> String {
    let mut result = prefix.to_owned();
    for word in variable.trim_start_matches('$').split('_') {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.extend(chars.map(|character| character.to_ascii_lowercase()));
        }
    }
    result
}

fn ground_fixture(
    branch: &Branch,
    prefix: &str,
    fused_current_review: bool,
    selectors: &[AtomSelector<'_>],
    overrides: &[(&str, &str)],
) -> StateFormResult<GroundedFixture> {
    let atoms = omit_fixture_atoms(raw_fixture_atoms(branch)?, selectors)?;
    let mut variables = Vec::new();
    let joined_atoms = atoms.join("\n");
    for matched in fixture_variable_regex().find_iter(&joined_atoms) {
        let variable = matched.as_str();
        if !variables.contains(&variable) {
            variables.push(variable);
        }
    }
    let mut mapping = variables
        .into_iter()
        .map(|variable| (variable.to_owned(), fixture_constant(prefix, variable)))
        .collect::<BTreeMap<_, _>>();
    mapping.insert("$power".to_owned(), branch.power());
    for (variable, value) in overrides {
        mapping.insert((*variable).to_owned(), (*value).to_owned());
    }
    if fused_current_review {
        let source = mapping
            .get("$source")
            .cloned()
            .ok_or_else(|| state_form_error("fixture has no $source"))?;
        mapping.insert("$temporal_review".to_owned(), source);
    }
    let mut facts = Vec::new();
    for atom in atoms {
        let grounded = fixture_variable_regex()
            .replace_all(&atom, |captures: &regex::Captures<'_>| {
                mapping
                    .get(&captures[0])
                    .unwrap_or_else(|| panic!("generated fixture lacks {}", &captures[0]))
                    .to_owned()
            })
            .into_owned();
        if !facts.contains(&grounded) {
            facts.push(grounded);
        }
    }
    if facts.iter().any(|fact| fact.contains('$')) {
        return Err(state_form_error(format!(
            "ungrounded state-form fixture: {prefix}"
        )));
    }
    if facts
        .iter()
        .any(|fact| !fact.starts_with("authorized(") && !fact.starts_with("observe("))
    {
        return Err(state_form_error(format!(
            "non-base state-form fixture fact: {prefix}"
        )));
    }
    Ok(GroundedFixture { facts, mapping })
}

fn authority_query(
    branch: &Branch,
    holder: &str,
    fixture: &GroundedFixture,
) -> StateFormResult<String> {
    Ok(format!(
        "authority({holder}, {}, {})",
        branch.power(),
        fixture.term("$record")?
    ))
}

fn complete_query(
    branch: &Branch,
    fixture: &GroundedFixture,
    result: Option<&str>,
) -> StateFormResult<String> {
    Ok(format!(
        "complete({}, {}, {})",
        result.map_or_else(|| fixture.term("$result"), Ok)?,
        branch.power(),
        fixture.term("$record")?
    ))
}

fn append_facts(lines: &mut Vec<String>, fixture: &GroundedFixture) {
    lines.extend(fixture.facts.iter().map(|fact| format!("{fact}.")));
}

fn append_query(lines: &mut Vec<String>, query: &str, expected: bool) {
    lines.push(format!("? {query}."));
    lines.push(format!("# => {}", if expected { "TRUE" } else { "FALSE" }));
    lines.push(String::new());
}

fn append_fixture_query(
    lines: &mut Vec<String>,
    branch: &Branch,
    holder: &str,
    fixture: &GroundedFixture,
    expected: bool,
) -> StateFormResult<()> {
    append_facts(lines, fixture);
    append_query(lines, &authority_query(branch, holder, fixture)?, expected);
    Ok(())
}

type FixtureRegistry = BTreeMap<(usize, String, String), (usize, GroundedFixture)>;

fn branch_holder_rows(branches: &[Branch]) -> Vec<(usize, &str)> {
    branches
        .iter()
        .enumerate()
        .flat_map(|(index, branch)| {
            branch
                .authority_holders
                .iter()
                .map(move |holder| (index, holder.as_str()))
        })
        .collect()
}

fn canonical_power_rows(branches: &[Branch]) -> StateFormResult<Vec<(usize, &str)>> {
    let mut rows = Vec::new();
    for number in 1..=CARD_COUNT {
        let (index, branch) = branches
            .iter()
            .enumerate()
            .find(|(_, branch)| branch.card == number)
            .ok_or_else(|| state_form_error(format!("FSPOW_{number:03} has no branch")))?;
        rows.push((index, branch.authority_holders[0].as_str()));
    }
    Ok(rows)
}

fn positive_fixture_registry(branches: &[Branch]) -> StateFormResult<FixtureRegistry> {
    let mut registry = BTreeMap::new();
    for (zero_index, (branch_index, holder)) in branch_holder_rows(branches).into_iter().enumerate()
    {
        let branch = &branches[branch_index];
        registry.insert(
            (branch.card, branch.key.clone(), holder.to_owned()),
            (
                branch_index,
                ground_fixture(
                    branch,
                    &format!("SFMainP{:03}", zero_index + 1),
                    false,
                    &[],
                    &[],
                )?,
            ),
        );
    }
    Ok(registry)
}

struct AcceptanceBuilder<'a> {
    branches: &'a [Branch],
    registry: &'a FixtureRegistry,
    lines: Vec<String>,
    count: usize,
}

impl<'a> AcceptanceBuilder<'a> {
    fn new(branches: &'a [Branch], registry: &'a FixtureRegistry) -> Self {
        Self {
            branches,
            registry,
            lines: vec![
                "# <STATE-FORM-ACCEPTANCE-CASES-BEGIN>".to_owned(),
                String::new(),
            ],
            count: 0,
        }
    }

    fn header(&mut self, case_id: &str) {
        self.lines.push(format!("# {case_id}"));
    }

    fn existing(
        &mut self,
        number: usize,
        key: &str,
        holder: &str,
    ) -> StateFormResult<GroundedFixture> {
        let (branch_index, fixture) = self
            .registry
            .get(&(number, key.to_owned(), holder.to_owned()))
            .ok_or_else(|| {
                state_form_error(format!(
                    "unknown state-form positive fixture {number:03}/{key}/{holder}"
                ))
            })?;
        let fixture = fixture.clone();
        let query = authority_query(&self.branches[*branch_index], holder, &fixture)?;
        append_query(&mut self.lines, &query, true);
        self.count += 1;
        Ok(fixture)
    }

    fn negative(
        &mut self,
        number: usize,
        key: &str,
        holder: &str,
        prefix: &str,
        selectors: &[AtomSelector<'_>],
    ) -> StateFormResult<GroundedFixture> {
        let branch = branch_lookup(self.branches, number, key)?;
        let fixture = ground_fixture(branch, prefix, false, selectors, &[])?;
        append_fixture_query(&mut self.lines, branch, holder, &fixture, false)?;
        self.count += 1;
        Ok(fixture)
    }

    fn query(&mut self, query: &str, expected: bool) {
        append_query(&mut self.lines, query, expected);
        self.count += 1;
    }
}

fn render_acceptance_cases(
    branches: &[Branch],
    registry: &FixtureRegistry,
) -> StateFormResult<(Vec<String>, usize)> {
    let mut builder = AcceptanceBuilder::new(branches, registry);

    builder.header(ACCEPTANCE_CASE_IDS[0]);
    let franchise_branch = branch_lookup(branches, 36, "adult_resident_franchise")?;
    let franchise = ground_fixture(
        franchise_branch,
        "SFAcc001Franchise",
        false,
        &[],
        &[("$subject", "Ruk")],
    )?;
    let candidacy_branch = branch_lookup(branches, 36, "adult_resident_candidacy")?;
    let candidacy = ground_fixture(
        candidacy_branch,
        "SFAcc001Candidacy",
        false,
        &[],
        &[("$subject", "Ruk")],
    )?;
    append_facts(&mut builder.lines, &franchise);
    append_facts(&mut builder.lines, &candidacy);
    builder.query("prisoner(Ruk)", true);
    builder.query(
        &authority_query(franchise_branch, "FSBOD_06", &franchise)?,
        true,
    );
    builder.query(
        &authority_query(candidacy_branch, "FSBOD_06", &candidacy)?,
        true,
    );

    builder.header(ACCEPTANCE_CASE_IDS[1]);
    let custody_branch = branch_lookup(branches, 36, "compelled_placement_nonchange")?;
    let custody = ground_fixture(
        custody_branch,
        "SFAcc002Custody",
        false,
        &[],
        &[("$subject", "Ruk")],
    )?;
    append_facts(&mut builder.lines, &custody);
    builder.query("prisoner(Ruk)", true);
    builder.query(
        &authority_query(custody_branch, "FSBOD_06", &custody)?,
        true,
    );

    builder.header(ACCEPTANCE_CASE_IDS[2]);
    builder.existing(36, "accessible_nonconventional_residence", "FSBOD_06")?;

    builder.header(ACCEPTANCE_CASE_IDS[3]);
    builder.existing(36, "multiple_residences_first_choice", "FSBOD_06")?;
    builder.existing(36, "multiple_residences_second_choice", "FSBOD_06")?;
    builder.negative(
        36,
        "multiple_residences_first_choice",
        "FSBOD_06",
        "SFAcc004NoChoice",
        &[AtomSelector::exact(
            "authorized($subject, PoliticalHomeChoiceAuthority, $record)",
        )],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[4]);
    builder.existing(36, "atomic_home_transfer", "FSBOD_06")?;
    builder.existing(36, "unique_accepted_submission", "FSBOD_06")?;

    builder.header(ACCEPTANCE_CASE_IDS[5]);
    builder.existing(36, "established_adulthood_continuity", "FSBOD_06")?;
    builder.existing(36, "provisional_adulthood_expiring_opportunity", "FSBOD_06")?;

    builder.header(ACCEPTANCE_CASE_IDS[6]);
    builder.existing(36, "office_move_continuity", "FSBOD_06")?;

    builder.header(ACCEPTANCE_CASE_IDS[7]);
    let (nonresident_index, nonresident) = builder
        .registry
        .get(&(
            36,
            "positive_nonresident_disposition".to_owned(),
            "FSBOD_06".to_owned(),
        ))
        .ok_or_else(|| state_form_error("missing positive nonresident fixture"))?;
    let nonresident = nonresident.clone();
    let nonresident_branch = &branches[*nonresident_index];
    builder.query(
        &authority_query(nonresident_branch, "FSBOD_06", &nonresident)?,
        true,
    );
    builder.query(
        &complete_query(nonresident_branch, &nonresident, None)?,
        true,
    );
    builder.query(
        &complete_query(
            nonresident_branch,
            &nonresident,
            Some("SFAcc008FranchiseResult"),
        )?,
        false,
    );
    builder.existing(36, "former_resident_return_without_ballot", "FSBOD_06")?;

    builder.header(ACCEPTANCE_CASE_IDS[8]);
    builder.existing(28, "assembly_appointment_selection", "FSBOD_02")?;
    builder.existing(
        30,
        "assembly_captured_source_fallback_appointment",
        "FSBOD_02",
    )?;
    builder.existing(35, "assembly_seat_allocation", "FSBOD_02")?;
    builder.negative(
        28,
        "assembly_appointment_selection",
        "FSBOD_02",
        "SFAcc009NoAntiCapture",
        &[AtomSelector::all(&["AntiCaptureScope"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[9]);
    let caretaker = builder.existing(41, "last_lawful_government_caretaker", "FSBOD_04")?;
    builder.query(
        &format!(
            "authority(FSBOD_04, FSPOW_014, {})",
            caretaker.term("$record")?
        ),
        false,
    );
    builder.existing(42, "fresh_election_call", "FSBOD_06")?;
    builder.negative(
        42,
        "fresh_election_call",
        "FSBOD_06",
        "SFAcc010NoDeadline",
        &[AtomSelector::all(&["PositiveDeadlinePassed"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[10]);
    builder.existing(18, "formal_government_appointment", "FSBOD_05")?;
    builder.existing(19, "promulgation", "FSBOD_05")?;
    builder.existing(20, "certificate_receipt", "FSBOD_05")?;
    builder.existing(21, "refusal_trigger", "FSBOD_26")?;
    builder.existing(44, "common_office_transfer", "FSBOD_05")?;

    builder.header(ACCEPTANCE_CASE_IDS[11]);
    builder.existing(12, "one_time_return", "FSBOD_03")?;
    builder.existing(5, "same_rule_repassage", "FSBOD_02")?;
    builder.negative(
        12,
        "one_time_return",
        "FSBOD_03",
        "SFAcc012NoUnusedReturn",
        &[AtomSelector::all(&["UnusedReturnScope"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[12]);
    builder.existing(10, "assembly_election", "FSBOD_06")?;
    builder.negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc013NoProportionalOutcome",
        &[AtomSelector::all(&["ProportionalOutcome"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[13]);
    builder.existing(43, "essential_budget_continuity", "FSBOD_07")?;
    builder.existing(43, "valid_budget_ends_continuity", "FSBOD_07")?;
    builder.existing(43, "continuity_limit_ends_authority", "FSBOD_07")?;

    builder.header(ACCEPTANCE_CASE_IDS[14]);
    builder.existing(25, "alternate_composition_panel", "FSBOD_25")?;
    builder.negative(
        25,
        "alternate_composition_panel",
        "FSBOD_25",
        "SFAcc015NoAlternatePanel",
        &[AtomSelector::all(&["UninvolvedAlternatePanel"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[15]);
    builder.negative(
        5,
        "same_rule_repassage",
        "FSBOD_02",
        "SFAcc016NoLegislativeCorridor",
        &[AtomSelector::all(&["EntrenchedDemocraticCorridor"])],
    )?;
    builder.negative(
        37,
        "ordinary_amendment",
        "FSBOD_01",
        "SFAcc016NoAmendmentCorridor",
        &[AtomSelector::all(&[
            "CompatibilityAndCorridorReviewComplete",
        ])],
    )?;
    builder.negative(
        39,
        "initiative_only_wins",
        "FSBOD_01",
        "SFAcc016NoInitiativeCorridor",
        &[AtomSelector::all(&[
            "PositiveCompatibilityAndCorridorReviewPassed",
        ])],
    )?;
    builder.negative(
        45,
        "final_exit_no_collective_impact",
        "FSBOD_01",
        "SFAcc016NoSecessionRightsReview",
        &[AtomSelector::all(&[
            "PositiveRightsAndMinorityReviewPassed",
        ])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[16]);
    builder.existing(45, "opening_referendum", "FSBOD_01")?;
    builder.existing(45, "completed_negotiation", "FSBOD_02")?;
    builder.existing(45, "completed_negotiation", "FSBOD_03")?;
    builder.existing(45, "final_exit_no_collective_impact", "FSBOD_01")?;
    builder.existing(45, "final_exit_with_collective_consent", "FSBOD_01")?;
    builder.existing(45, "final_exit_with_collective_consent", "FSBOD_21")?;
    builder.negative(
        45,
        "final_exit_with_collective_consent",
        "FSBOD_01",
        "SFAcc017NoSettlement",
        &[AtomSelector::all(&["PositiveSettlementComplete"])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[17]);
    builder.negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc018NoExactRoster",
        &[AtomSelector::all(&[
            "$eligible_roster",
            "CompleteAndNonzeroEligibleRoster",
        ])],
    )?;
    builder.negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc018NoExactSubmissions",
        &[AtomSelector::all(&[
            "$submission_set",
            "CompleteUniqueSubmissionSet",
        ])],
    )?;

    builder.header(ACCEPTANCE_CASE_IDS[18]);
    let lineage = branch_lookup(branches, 44, "common_office_transfer")?
        .decision_lineage
        .as_ref()
        .ok_or_else(|| state_form_error("common office transfer has no lineage"))?;
    let any_values = [
        "$successor",
        "SuccessorScope",
        lineage.certificate_set[0].as_str(),
        lineage.certificate_set[1].as_str(),
        lineage.result_certificate[0].as_str(),
        lineage.result_certificate[1].as_str(),
        lineage.certified_result[0].as_str(),
        lineage.certified_result[1].as_str(),
        "CompleteUniqueCertificateSet",
    ];
    builder.negative(
        44,
        "common_office_transfer",
        "FSBOD_05",
        "SFAcc019NoCertificateChain",
        &[AtomSelector::any(&any_values)],
    )?;
    builder.negative(
        44,
        "common_office_transfer",
        "FSBOD_05",
        "SFAcc019NoCertificateCompleteness",
        &[AtomSelector::all(&["CompleteUniqueCertificateSet"])],
    )?;

    builder
        .lines
        .push("# <STATE-FORM-ACCEPTANCE-CASES-END>".to_owned());
    builder.lines.push(String::new());
    let manifest = builder
        .lines
        .iter()
        .filter_map(|line| line.strip_prefix("# FSACC-"))
        .map(|suffix| format!("FSACC-{suffix}"))
        .collect::<Vec<_>>();
    if manifest
        != ACCEPTANCE_CASE_IDS
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
    {
        return Err(state_form_error(
            "state-form acceptance case manifest drifted",
        ));
    }
    Ok((builder.lines, builder.count))
}

fn render_main_pins(source: &SemanticSource) -> StateFormResult<String> {
    let registry = positive_fixture_registry(&source.branches)?;
    let mut lines = vec![
        SPDX_HEADER.to_owned(),
        MAIN_HEADER.to_owned(),
        "#".to_owned(),
        "# These fixtures supply bounded source records. They do not prove that".to_owned(),
        "# any institution, roster, result, office, or remedy exists outside".to_owned(),
        "# this executable probe.".to_owned(),
        format!(":expect-pins {MAIN_PIN_COUNT}"),
        String::new(),
        "# <STATE-FORM-GENERIC-POSITIVE-BEGIN>".to_owned(),
    ];
    let rows = branch_holder_rows(&source.branches);
    let mut generic_count = 0;
    for (branch_index, holder) in &rows {
        let branch = &source.branches[*branch_index];
        let (_, fixture) = registry
            .get(&(branch.card, branch.key.clone(), (*holder).to_owned()))
            .expect("registry covers every branch/holder");
        lines.push(format!(
            "# {}/{}/{} positive authority.",
            branch.power(),
            branch.key,
            holder
        ));
        append_fixture_query(&mut lines, branch, holder, fixture, true)?;
        generic_count += 1;
    }
    lines.push("# <STATE-FORM-GENERIC-POSITIVE-END>".to_owned());
    lines.push(String::new());
    lines.push("# <STATE-FORM-MISSING-REVIEW-BEGIN>".to_owned());
    for (zero_index, (branch_index, holder)) in rows.iter().enumerate() {
        let branch = &source.branches[*branch_index];
        let fixture = ground_fixture(
            branch,
            &format!("SFMainN{:03}", zero_index + 1),
            false,
            &[AtomSelector::exact(
                "authorized($review, IndependentStateFormReviewAuthority, $record)",
            )],
            &[],
        )?;
        lines.push(format!(
            "# {}/{}/{} missing independent review.",
            branch.power(),
            branch.key,
            holder
        ));
        append_fixture_query(&mut lines, branch, holder, &fixture, false)?;
        generic_count += 1;
    }
    lines.push("# <STATE-FORM-MISSING-REVIEW-END>".to_owned());
    lines.push(String::new());
    lines.push("# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-BEGIN>".to_owned());
    for (branch_index, holder) in canonical_power_rows(&source.branches)? {
        let branch = &source.branches[branch_index];
        let fixture = ground_fixture(
            branch,
            &format!("SFMainF{:03}", branch.card),
            true,
            &[],
            &[],
        )?;
        lines.push(format!(
            "# FS-POW-{:03} negative: fused source/current reviewer cannot derive authority.",
            branch.card
        ));
        append_fixture_query(&mut lines, branch, holder, &fixture, false)?;
        generic_count += 1;
    }
    lines.push("# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-END>".to_owned());
    lines.push(String::new());
    if generic_count != GENERIC_MAIN_PIN_COUNT {
        return Err(state_form_error(format!(
            "expected {GENERIC_MAIN_PIN_COUNT} generic pins, found {generic_count}"
        )));
    }
    let (acceptance, acceptance_count) = render_acceptance_cases(&source.branches, &registry)?;
    if acceptance_count != ACCEPTANCE_PIN_COUNT {
        return Err(state_form_error(format!(
            "expected {ACCEPTANCE_PIN_COUNT} acceptance pins, found {acceptance_count}"
        )));
    }
    lines.extend(acceptance);
    let rendered = format!("{}\n", lines.join("\n").trim_end());
    if query_count(&rendered) != MAIN_PIN_COUNT {
        return Err(state_form_error(
            "rendered state-form main pin count drifted",
        ));
    }
    Ok(rendered)
}

fn render_counterfactual_pins(source: &SemanticSource) -> StateFormResult<String> {
    let mut lines = vec![
        SPDX_HEADER.to_owned(),
        COUNTERFACTUAL_HEADER.to_owned(),
        format!(":expect-pins {COUNTERFACTUAL_PIN_COUNT}"),
        String::new(),
    ];
    let mut count = 0;
    for (branch_index, holder) in canonical_power_rows(&source.branches)? {
        let branch = &source.branches[branch_index];
        let fixture = ground_fixture(
            branch,
            &format!("SFMainF{:03}", branch.card),
            true,
            &[],
            &[],
        )?;
        lines.push(format!(
            "# FS-POW-{:03} counterfactual: removing the independent-current-review guard derives authority.",
            branch.card
        ));
        append_fixture_query(&mut lines, branch, holder, &fixture, true)?;
        count += 1;
    }
    if count != COUNTERFACTUAL_PIN_COUNT {
        return Err(state_form_error(
            "rendered state-form counterfactual pin count drifted",
        ));
    }
    let rendered = format!("{}\n", lines.join("\n").trim_end());
    if query_count(&rendered) != COUNTERFACTUAL_PIN_COUNT {
        return Err(state_form_error(
            "rendered state-form counterfactual query count drifted",
        ));
    }
    Ok(rendered)
}

fn canonical_main_pins() -> StateFormResult<&'static str> {
    static PINS: OnceLock<StateFormResult<String>> = OnceLock::new();
    match PINS.get_or_init(|| render_main_pins(semantic_source()?)) {
        Ok(pins) => Ok(pins),
        Err(error) => Err(error.clone()),
    }
}

fn canonical_counterfactual_pins() -> StateFormResult<&'static str> {
    static PINS: OnceLock<StateFormResult<String>> = OnceLock::new();
    match PINS.get_or_init(|| render_counterfactual_pins(semantic_source()?)) {
        Ok(pins) => Ok(pins),
        Err(error) => Err(error.clone()),
    }
}

fn extract_block(text: &str) -> StateFormResult<&str> {
    let begin = format!("{BEGIN}\n");
    let end = format!("{END}\n");
    if count_occurrences(text, &begin) != 1 || count_occurrences(text, &end) != 1 {
        return Err(state_form_error(
            "constitution must contain one ordered state-form marker pair",
        ));
    }
    let start = text
        .find(&begin)
        .expect("count proved state-form begin exists");
    let relative_end = text[start..]
        .find(&end)
        .ok_or_else(|| state_form_error("constitution state-form markers are reversed"))?;
    let stop = start + relative_end + end.len();
    Ok(&text[start..stop])
}

fn canonical_rendered_block() -> StateFormResult<String> {
    render_formal_block(semantic_source()?)
}

pub(crate) fn rendered_block() -> Result<String, Error> {
    canonical_rendered_block().map_err(public_error)
}

fn formal_statements(block: &str) -> Vec<&str> {
    block
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

fn split_top_level<'a>(text: &'a str, separator: &str) -> StateFormResult<Vec<&'a str>> {
    if separator.is_empty() {
        return Err(state_form_error("separator must not be empty"));
    }
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Err(state_form_error(format!(
                        "unbalanced closing parenthesis in {text:?}"
                    )));
                }
                depth -= 1;
            }
            _ if depth == 0 && text[index..].starts_with(separator) => {
                parts.push(&text[start..index]);
                index += separator.len();
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    if depth != 0 {
        return Err(state_form_error(format!(
            "unbalanced parentheses in {text:?}"
        )));
    }
    parts.push(&text[start..]);
    Ok(parts)
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn parse_call(text: &str) -> StateFormResult<ParsedCall> {
    let open = text
        .find('(')
        .ok_or_else(|| state_form_error(format!("not a positive relation call: {text:?}")))?;
    if !text.ends_with(')') || !valid_identifier(&text[..open]) {
        return Err(state_form_error(format!(
            "not a positive relation call: {text:?}"
        )));
    }
    let args = split_top_level(&text[open + 1..text.len() - 1], ",")?
        .into_iter()
        .map(str::trim)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if args.iter().any(String::is_empty) {
        return Err(state_form_error(format!(
            "empty relation argument in {text:?}"
        )));
    }
    Ok(ParsedCall {
        name: text[..open].to_owned(),
        args,
    })
}

fn parse_disequality(atom: &str) -> Option<(String, String)> {
    let inner = atom.strip_prefix("~($")?.strip_suffix(')')?;
    let (left, right) = inner.split_once(" = $")?;
    if valid_identifier(left) && valid_identifier(right) {
        Some((left.to_owned(), right.to_owned()))
    } else {
        None
    }
}

fn variable_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").expect("valid variable regex"))
}

fn parse_rule(statement: &str) -> StateFormResult<ParsedRule> {
    let mut remainder = statement
        .strip_suffix('.')
        .ok_or_else(|| state_form_error("state-form statement lacks a final period"))?;
    let mut quantified = Vec::new();
    while let Some(after_all) = remainder.strip_prefix("all $") {
        let (name, rest) = after_all.split_once(": ").ok_or_else(|| {
            state_form_error(format!("malformed universal quantifier in {statement:?}"))
        })?;
        if !valid_identifier(name) {
            return Err(state_form_error(format!(
                "malformed universal quantifier in {statement:?}"
            )));
        }
        quantified.push(name.to_owned());
        remainder = rest;
    }
    if remainder.contains("any ") || statement.starts_with("any ") {
        return Err(state_form_error(
            "state-form rules may use universal quantification only",
        ));
    }
    if quantified.iter().collect::<HashSet<_>>().len() != quantified.len() {
        return Err(state_form_error(
            "state-form rule quantifies a variable more than once",
        ));
    }
    let implication = split_top_level(remainder, " -> ")?;
    if implication.len() != 2 {
        return Err(state_form_error(
            "state-form rule must have exactly one top-level implication",
        ));
    }
    let atoms = split_top_level(implication[0], " & ")?;
    if atoms.iter().collect::<HashSet<_>>().len() != atoms.len() {
        return Err(state_form_error(
            "state-form rule repeats an exact body atom",
        ));
    }
    let mut body_calls = Vec::new();
    let mut disequalities = Vec::new();
    for atom in atoms {
        if let Some(pair) = parse_disequality(atom) {
            disequalities.push(pair);
        } else if atom.starts_with('~') {
            return Err(state_form_error(
                "negative predicate premises are forbidden",
            ));
        } else {
            body_calls.push(parse_call(atom)?);
        }
    }
    let head = parse_call(implication[1])?;
    let used: BTreeSet<_> = variable_regex()
        .captures_iter(remainder)
        .map(|capture| capture[1].to_owned())
        .collect();
    let declared: BTreeSet<_> = quantified.iter().cloned().collect();
    if used != declared {
        return Err(state_form_error(format!(
            "quantified/used variable mismatch: quantified={declared:?}, used={used:?}"
        )));
    }
    Ok(ParsedRule {
        quantified,
        body_calls,
        disequalities,
        head,
    })
}

fn validate_call(call: &ParsedCall, in_head: bool) -> StateFormResult<()> {
    const BANNED: [&str; 8] = [
        "match", "collide", "public", "choose", "decide", "broken", "approves", "mature",
    ];
    if BANNED.contains(&call.name.as_str()) {
        return Err(state_form_error(format!(
            "legacy or diagnostic relation {}/{} is forbidden",
            call.name,
            call.args.len()
        )));
    }
    let expected_arity = match call.name.as_str() {
        "authorized" => 3,
        "observe" => 4,
        "complete" | "authority" => 3,
        _ => {
            return Err(state_form_error(format!(
                "unapproved relation signature {}/{}",
                call.name,
                call.args.len()
            )));
        }
    };
    if call.args.len() != expected_arity {
        return Err(state_form_error(format!(
            "unapproved relation signature {}/{}",
            call.name,
            call.args.len()
        )));
    }
    if call.name == "authority" && !in_head {
        return Err(state_form_error(
            "authority/3 may appear only as a direct-effect head",
        ));
    }
    if matches!(call.name.as_str(), "authorized" | "observe") && in_head {
        return Err(state_form_error(format!(
            "{}/{} may not appear in a head",
            call.name,
            call.args.len()
        )));
    }
    if call
        .args
        .iter()
        .any(|argument| argument.chars().all(|character| character.is_ascii_digit()))
    {
        return Err(state_form_error(
            "standalone numeric literals are forbidden",
        ));
    }
    Ok(())
}

fn validate_rule_surface(statements: &[&str]) -> StateFormResult<Vec<ParsedRule>> {
    if statements.len() != STATEMENT_COUNT {
        return Err(state_form_error(format!(
            "expected {STATEMENT_COUNT} statements, found {}",
            statements.len()
        )));
    }
    if statements.iter().collect::<HashSet<_>>().len() != statements.len() {
        return Err(state_form_error("state-form statements are not unique"));
    }
    let guard_count = statements
        .iter()
        .map(|statement| count_occurrences(statement, CURRENT_REVIEW_GUARD.trim_start()))
        .sum::<usize>();
    if guard_count != 1 {
        return Err(state_form_error(
            "independent-current-review guard must occur exactly once",
        ));
    }
    if statements
        .iter()
        .any(|statement| statement.contains("FALSE"))
    {
        return Err(state_form_error(
            "state-form rules may not derive or consume FALSE",
        ));
    }
    let parsed = statements
        .iter()
        .map(|statement| parse_rule(statement))
        .collect::<StateFormResult<Vec<_>>>()?;
    let review_guards = parsed
        .iter()
        .enumerate()
        .flat_map(|(index, rule)| {
            rule.disequalities
                .iter()
                .filter(|(left, right)| {
                    (left == "source" && right == "temporal_review")
                        || (left == "temporal_review" && right == "source")
                })
                .map(move |_| index)
        })
        .collect::<Vec<_>>();
    if review_guards != [0] {
        return Err(state_form_error(
            "source/temporal-review separation must occur once, in the shared current rule only",
        ));
    }
    let actual_pairs = parsed[0]
        .disequalities
        .iter()
        .map(|(left, right)| {
            if left <= right {
                (left.as_str(), right.as_str())
            } else {
                (right.as_str(), left.as_str())
            }
        })
        .collect::<BTreeSet<_>>();
    let expected_pairs = [
        ("source", "temporal"),
        ("source", "temporal_review"),
        ("record_review", "source"),
        ("temporal", "temporal_review"),
        ("record_review", "temporal"),
        ("record_review", "temporal_review"),
    ]
    .into_iter()
    .map(|(left, right)| {
        if left <= right {
            (left, right)
        } else {
            (right, left)
        }
    })
    .collect::<BTreeSet<_>>();
    if actual_pairs != expected_pairs {
        return Err(state_form_error(format!(
            "shared current separation set changed: {actual_pairs:?}"
        )));
    }
    let mut signatures = BTreeSet::new();
    for rule in &parsed {
        let _ = rule.quantified.len();
        for call in &rule.body_calls {
            validate_call(call, false)?;
            signatures.insert((call.name.as_str(), call.args.len()));
        }
        validate_call(&rule.head, true)?;
        signatures.insert((rule.head.name.as_str(), rule.head.args.len()));
    }
    let expected_signatures = [
        ("authorized", 3),
        ("observe", 4),
        ("complete", 3),
        ("authority", 3),
    ]
    .into_iter()
    .collect();
    if signatures != expected_signatures {
        return Err(state_form_error(format!(
            "state-form relation signatures changed: {signatures:?}"
        )));
    }
    let expected_current = ParsedCall {
        name: "complete".to_owned(),
        args: vec![
            "$record".to_owned(),
            "StateFormCurrent".to_owned(),
            "$temporal_record".to_owned(),
        ],
    };
    if parsed[0].head != expected_current {
        return Err(state_form_error(
            "first statement is not the shared current declaration",
        ));
    }

    let mut results = 0;
    let mut authorities = 0;
    let mut power_results = BTreeMap::<usize, usize>::new();
    let mut power_authorities = BTreeMap::<usize, usize>::new();
    for rule in &parsed[1..] {
        let map = match rule.head.name.as_str() {
            "complete" => {
                results += 1;
                &mut power_results
            }
            "authority" => {
                authorities += 1;
                let expected_complete = ParsedCall {
                    name: "complete".to_owned(),
                    args: vec![
                        "$result".to_owned(),
                        rule.head.args[1].clone(),
                        "$record".to_owned(),
                    ],
                };
                if !rule.body_calls.contains(&expected_complete) {
                    return Err(state_form_error(format!(
                        "{} direct-effect rule lost its result premise",
                        rule.head.args[1]
                    )));
                }
                &mut power_authorities
            }
            _ => {
                return Err(state_form_error(
                    "state-form head is neither complete/3 nor authority/3",
                ));
            }
        };
        let power = rule.head.args.get(1).ok_or_else(|| {
            state_form_error("state-form result or authority head has no power argument")
        })?;
        let number = power
            .strip_prefix("FSPOW_")
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .filter(|number| (1..=CARD_COUNT).contains(number))
            .ok_or_else(|| state_form_error(format!("state-form power head drifted: {power}")))?;
        *map.entry(number).or_default() += 1;
    }
    if results != RESULT_COUNT || authorities != AUTHORITY_COUNT {
        return Err(state_form_error(format!(
            "state-form head census changed: {results} result declarations, {authorities} authority heads"
        )));
    }
    if power_results.len() != CARD_COUNT || power_authorities.len() != CARD_COUNT {
        return Err(state_form_error("state-form card coverage changed"));
    }
    for (start, end, expected_results, expected_authorities) in [
        (1, 25, 35, 36),
        (26, 35, 47, 49),
        (36, 36, 16, 16),
        (37, 51, 33, 41),
    ] {
        let actual_results = (start..=end)
            .map(|number| power_results.get(&number).copied().unwrap_or(0))
            .sum::<usize>();
        let actual_authorities = (start..=end)
            .map(|number| power_authorities.get(&number).copied().unwrap_or(0))
            .sum::<usize>();
        if (actual_results, actual_authorities) != (expected_results, expected_authorities) {
            return Err(state_form_error(format!(
                "state-form band {start:03}..{end:03} changed: {actual_results} results, {actual_authorities} authorities"
            )));
        }
    }
    Ok(parsed)
}

fn validate_formal_source(source: &str) -> StateFormResult<Vec<String>> {
    if sha256(source.as_bytes()) != EXPECTED_CONSTITUTION_SHA256 {
        return Err(state_form_error(format!(
            "constitution SHA-256 drifted for state-form artifacts: expected {EXPECTED_CONSTITUTION_SHA256}, found {}",
            sha256(source.as_bytes())
        )));
    }
    let actual = extract_block(source)?;
    let expected = canonical_rendered_block()?;
    if actual != expected {
        return Err(state_form_error(
            "constitution state-form block differs from checker-owned exact block",
        ));
    }
    if sha256(actual.as_bytes()) != EXPECTED_RENDERED_BLOCK_SHA256 {
        return Err(state_form_error("state-form rendered block digest changed"));
    }
    let statements = formal_statements(actual);
    validate_rule_surface(&statements)?;
    let joined = format!("{}\n", statements.join("\n"));
    if sha256(joined.as_bytes()) != EXPECTED_RULE_BLOCK_SHA256 {
        return Err(state_form_error(format!(
            "state-form exact rule block changed: expected {EXPECTED_RULE_BLOCK_SHA256}, found {}",
            sha256(joined.as_bytes())
        )));
    }
    Ok(statements.into_iter().map(str::to_owned).collect())
}

fn render_counterfactual(source: &str) -> StateFormResult<String> {
    let rule = render_current_rule();
    if count_occurrences(source, &rule) != 1 {
        return Err(state_form_error(
            "state-form current rule must occur once in the constitution",
        ));
    }
    if count_occurrences(&rule, CURRENT_REVIEW_GUARD) != 1 {
        return Err(state_form_error(
            "state-form current rule must contain one independent-review guard",
        ));
    }
    let mutated = rule.replacen(CURRENT_REVIEW_GUARD, "", 1);
    if mutated.contains(CURRENT_REVIEW_GUARD) {
        return Err(state_form_error(
            "state-form counterfactual guard removal failed",
        ));
    }
    let counterfactual = source.replacen(&rule, &mutated, 1);
    let source_lines = source.lines().collect::<Vec<_>>();
    let counterfactual_lines = counterfactual.lines().collect::<Vec<_>>();
    if source_lines.len() != counterfactual_lines.len() {
        return Err(state_form_error(
            "state-form counterfactual changed line count",
        ));
    }
    let differences = source_lines
        .iter()
        .zip(&counterfactual_lines)
        .filter(|(left, right)| left != right)
        .collect::<Vec<_>>();
    if differences.len() != 1 || *differences[0].0 != rule || *differences[0].1 != mutated.as_str()
    {
        return Err(state_form_error(
            "state-form counterfactual is not the exact one-line mutation",
        ));
    }
    Ok(counterfactual)
}

fn validate_counterfactual_shape(source: &str, counterfactual: &str) -> StateFormResult<()> {
    let rendered = render_counterfactual(source)?;
    if counterfactual != rendered {
        return Err(state_form_error(
            "counterfactual differs outside the one current-review guard",
        ));
    }
    Ok(())
}

fn query_count(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("? ")).count()
}

fn validate_pin_surface(
    text: &str,
    header: &str,
    expected_count: usize,
    allow_prisoner: bool,
) -> StateFormResult<()> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.first().copied() != Some(SPDX_HEADER) {
        return Err(state_form_error("state-form pin SPDX header drifted"));
    }
    if lines.iter().filter(|line| **line == header).count() != 1 {
        return Err(state_form_error(format!(
            "state-form pin family header drifted: {header}"
        )));
    }
    let expectation = format!(":expect-pins {expected_count}");
    if lines.iter().filter(|line| **line == expectation).count() != 1 {
        return Err(state_form_error(
            "state-form pin expectation directive drifted",
        ));
    }
    if query_count(text) != expected_count {
        return Err(state_form_error(format!(
            "expected {expected_count} state-form queries, found {}",
            query_count(text)
        )));
    }
    for line in lines {
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(':')
            || line.starts_with("? ")
        {
            continue;
        }
        let atom = line.strip_suffix('.').ok_or_else(|| {
            state_form_error(format!("malformed state-form fixture fact: {line:?}"))
        })?;
        if atom.starts_with("complete(") || atom.starts_with("authority(") {
            return Err(state_form_error(
                "state-form fixtures must not assert derived declarations or authority",
            ));
        }
        if atom.starts_with("authorized(") || atom.starts_with("observe(") {
            continue;
        }
        if allow_prisoner && atom.starts_with("prisoner(") {
            continue;
        }
        return Err(state_form_error(format!(
            "unexpected state-form fixture fact: {atom:?}"
        )));
    }
    Ok(())
}

fn validate_main_manifest(text: &str) -> StateFormResult<()> {
    for case_id in ACCEPTANCE_CASE_IDS {
        if count_occurrences(text, &format!("# {case_id}\n")) != 1 {
            return Err(state_form_error(format!(
                "state-form acceptance case header drifted: {case_id}"
            )));
        }
    }
    for number in 1..=CARD_COUNT {
        let comment = format!(
            "# FS-POW-{number:03} negative: fused source/current reviewer cannot derive authority.\n"
        );
        if count_occurrences(text, &comment) != 1 {
            return Err(state_form_error(format!(
                "state-form main negative comment drifted: FS-POW-{number:03}"
            )));
        }
    }
    for (begin, end) in [
        (
            "# <STATE-FORM-GENERIC-POSITIVE-BEGIN>",
            "# <STATE-FORM-GENERIC-POSITIVE-END>",
        ),
        (
            "# <STATE-FORM-MISSING-REVIEW-BEGIN>",
            "# <STATE-FORM-MISSING-REVIEW-END>",
        ),
        (
            "# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-BEGIN>",
            "# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-END>",
        ),
        (
            "# <STATE-FORM-ACCEPTANCE-CASES-BEGIN>",
            "# <STATE-FORM-ACCEPTANCE-CASES-END>",
        ),
    ] {
        if count_occurrences(text, begin) != 1 || count_occurrences(text, end) != 1 {
            return Err(state_form_error(format!(
                "state-form pin marker drifted: {begin} / {end}"
            )));
        }
        if text.find(begin) >= text.find(end) {
            return Err(state_form_error(format!(
                "state-form pin marker order drifted: {begin} / {end}"
            )));
        }
    }
    let acceptance_begin = text
        .find("# <STATE-FORM-ACCEPTANCE-CASES-BEGIN>")
        .expect("marker validated");
    let generic = query_count(&text[..acceptance_begin]);
    let acceptance = query_count(&text[acceptance_begin..]);
    if (generic, acceptance) != (GENERIC_MAIN_PIN_COUNT, ACCEPTANCE_PIN_COUNT) {
        return Err(state_form_error(format!(
            "state-form main pin partition changed: {generic} generic, {acceptance} acceptance"
        )));
    }
    Ok(())
}

fn validate_counterfactual_manifest(text: &str) -> StateFormResult<()> {
    for number in 1..=CARD_COUNT {
        let comment = format!(
            "# FS-POW-{number:03} counterfactual: removing the independent-current-review guard derives authority.\n"
        );
        if count_occurrences(text, &comment) != 1 {
            return Err(state_form_error(format!(
                "state-form counterfactual comment drifted: FS-POW-{number:03}"
            )));
        }
    }
    Ok(())
}

fn validate_delegation_markers(snapshot: &SourceSnapshot) -> StateFormResult<()> {
    let actual = snapshot
        .chapter_pins
        .iter()
        .filter(|(_, source)| source.contains(DELEGATION_MARKER))
        .map(|(path, _)| path.as_str())
        .collect::<Vec<_>>();
    if actual != DELEGATED_PIN_PATHS {
        return Err(state_form_error(format!(
            "state-form chapter-pin delegation markers changed: {actual:?}"
        )));
    }
    for expected in DELEGATED_PIN_PATHS {
        let source = snapshot
            .chapter_pins
            .iter()
            .find(|(path, _)| path == expected)
            .map(|(_, source)| source)
            .ok_or_else(|| state_form_error(format!("missing delegated pin file: {expected}")))?;
        if count_occurrences(source, DELEGATION_MARKER) != 1 {
            return Err(state_form_error(format!(
                "{expected} must contain one exact state-form delegation marker"
            )));
        }
    }
    Ok(())
}

fn validate_artifact(
    path: &str,
    actual: &str,
    expected: &str,
    expected_sha256: &str,
) -> StateFormResult<()> {
    if actual != expected {
        return Err(state_form_error(format!(
            "state-form artifact differs from checker-owned bytes: {path}"
        )));
    }
    let digest = sha256(actual.as_bytes());
    if digest != expected_sha256 {
        return Err(state_form_error(format!(
            "state-form artifact SHA-256 drifted for {path}: expected {expected_sha256}, found {digest}"
        )));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct ProjectionQuery<'a> {
    query_line: &'a str,
    expectation_line: &'a str,
    facts: Vec<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderedPinShard {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) query_count: usize,
    pub(crate) fixture_fact_count: usize,
    pub(crate) relation_call_count: usize,
    pub(crate) projection_utf8_bytes: usize,
    pub(crate) utf8_bytes: usize,
    pub(crate) fixture_facts_sha256: String,
    pub(crate) query_stream_sha256: String,
    pub(crate) projection_sha256: String,
    pub(crate) partition_strategy: ShardPartition,
}

fn fixture_term_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\bSF(?:Main|Acc)[A-Za-z0-9_]*\b").expect("valid fixture-term regex")
    })
}

fn canonical_pin_facts(text: &str) -> StateFormResult<Vec<&str>> {
    let facts = text
        .lines()
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with(':')
                && !line.starts_with("? ")
        })
        .collect::<Vec<_>>();
    if facts.iter().collect::<HashSet<_>>().len() != facts.len() {
        return Err(state_form_error(
            "canonical state-form pins contain duplicate facts",
        ));
    }
    Ok(facts)
}

fn canonical_pin_query_pairs(text: &str) -> StateFormResult<Vec<(&str, &str)>> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut pairs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !line.starts_with("? ") {
            continue;
        }
        let expectation = lines
            .get(index + 1)
            .copied()
            .ok_or_else(|| state_form_error("state-form query lacks an expected verdict"))?;
        if !matches!(expectation, "# => TRUE" | "# => FALSE") {
            return Err(state_form_error(format!(
                "state-form query verdict drifted after {line:?}"
            )));
        }
        pairs.push((*line, expectation));
    }
    Ok(pairs)
}

fn pin_pairs_sha256(pairs: &[(&str, &str)]) -> String {
    let mut serialized = String::new();
    for (query, expectation) in pairs {
        let _ = writeln!(serialized, "{query}");
        let _ = writeln!(serialized, "{expectation}");
    }
    sha256(serialized.as_bytes())
}

fn pin_query_stream_sha256(text: &str) -> StateFormResult<String> {
    Ok(pin_pairs_sha256(&canonical_pin_query_pairs(text)?))
}

fn canonical_pin_query_blocks(text: &str) -> StateFormResult<Vec<ProjectionQuery<'_>>> {
    let facts = canonical_pin_facts(text)?;
    let fact_tokens = facts
        .iter()
        .map(|fact| {
            fixture_term_regex()
                .find_iter(fact)
                .map(|matched| matched.as_str())
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>();
    let mut token_to_fact_indices = HashMap::<&str, Vec<usize>>::new();
    for (index, tokens) in fact_tokens.iter().enumerate() {
        if tokens.is_empty() {
            return Err(state_form_error(format!(
                "state-form fact has no isolated fixture term: {}",
                facts[index]
            )));
        }
        for token in tokens {
            token_to_fact_indices.entry(token).or_default().push(index);
        }
    }

    let mut blocks = Vec::new();
    let mut used_fact_indices = HashSet::new();
    for (query, expectation) in canonical_pin_query_pairs(text)? {
        let mut pending = fixture_term_regex()
            .find_iter(query)
            .map(|matched| matched.as_str())
            .collect::<Vec<_>>();
        if pending.is_empty() {
            if query != "? prisoner(Ruk)." {
                return Err(state_form_error(format!(
                    "state-form query has no isolated fixture term: {query}"
                )));
            }
            blocks.push(ProjectionQuery {
                query_line: query,
                expectation_line: expectation,
                facts: Vec::new(),
            });
            continue;
        }
        let mut known_tokens = HashSet::new();
        let mut selected = BTreeSet::new();
        while let Some(token) = pending.pop() {
            if !known_tokens.insert(token) {
                continue;
            }
            for fact_index in token_to_fact_indices
                .get(token)
                .into_iter()
                .flatten()
                .copied()
            {
                if !selected.insert(fact_index) {
                    continue;
                }
                for linked in &fact_tokens[fact_index] {
                    if !known_tokens.contains(linked) {
                        pending.push(linked);
                    }
                }
            }
        }
        if selected.is_empty() {
            return Err(state_form_error(format!(
                "state-form query has no fixture fact closure: {query}"
            )));
        }
        used_fact_indices.extend(selected.iter().copied());
        blocks.push(ProjectionQuery {
            query_line: query,
            expectation_line: expectation,
            facts: selected.into_iter().map(|index| facts[index]).collect(),
        });
    }
    if used_fact_indices.len() != facts.len() {
        let unused = facts
            .iter()
            .enumerate()
            .filter(|(index, _)| !used_fact_indices.contains(index))
            .map(|(_, fact)| *fact)
            .take(3)
            .collect::<Vec<_>>();
        return Err(state_form_error(format!(
            "canonical state-form facts are not exercised by a query: {unused:?}"
        )));
    }
    Ok(blocks)
}

fn balanced_pin_slices(total: usize, shard_count: usize) -> StateFormResult<Vec<(usize, usize)>> {
    if shard_count == 0 || total < shard_count {
        return Err(state_form_error(format!(
            "cannot divide {total} queries into {shard_count} shards"
        )));
    }
    let quotient = total / shard_count;
    let remainder = total % shard_count;
    let mut slices = Vec::with_capacity(shard_count);
    let mut start = 0;
    for index in 0..shard_count {
        let size = quotient + usize::from(index < remainder);
        slices.push((start, start + size));
        start += size;
    }
    if start != total {
        return Err(state_form_error("state-form shard slice census drifted"));
    }
    Ok(slices)
}

fn render_pin_projection<'a>(
    blocks: &[ProjectionQuery<'a>],
) -> StateFormResult<(String, Vec<&'a str>, Vec<(&'a str, &'a str)>)> {
    if blocks.is_empty() {
        return Err(state_form_error(
            "state-form shard projection must not be empty",
        ));
    }
    let mut rendered = String::new();
    let mut emitted_fact_set = HashSet::new();
    let mut emitted_facts = Vec::new();
    let mut pairs = Vec::new();
    for block in blocks {
        for fact in &block.facts {
            if emitted_fact_set.insert(*fact) {
                emitted_facts.push(*fact);
                let _ = writeln!(rendered, "{fact}");
            }
        }
        let _ = writeln!(rendered, "{}", block.query_line);
        let _ = writeln!(rendered, "{}", block.expectation_line);
        rendered.push('\n');
        pairs.push((block.query_line, block.expectation_line));
    }
    while rendered.ends_with("\n\n") {
        rendered.pop();
    }
    Ok((rendered, emitted_facts, pairs))
}

fn projection_utf8_bytes(blocks: &[ProjectionQuery<'_>]) -> StateFormResult<usize> {
    Ok(render_pin_projection(blocks)?.0.len())
}

fn greedy_byte_slices(
    blocks: &[ProjectionQuery<'_>],
    capacity: usize,
) -> StateFormResult<Vec<(usize, usize)>> {
    let mut slices = Vec::new();
    let mut start = 0;
    while start < blocks.len() {
        let mut end = start;
        let mut emitted_facts = HashSet::new();
        let mut current_bytes = 0;
        while end < blocks.len() {
            let block = &blocks[end];
            let new_facts = block
                .facts
                .iter()
                .filter(|fact| !emitted_facts.contains(*fact))
                .copied()
                .collect::<Vec<_>>();
            let mut increment = new_facts.iter().map(|fact| fact.len() + 1).sum::<usize>();
            increment += block.query_line.len() + 1;
            increment += block.expectation_line.len() + 1;
            increment += usize::from(end > start);
            if current_bytes + increment > capacity {
                break;
            }
            current_bytes += increment;
            emitted_facts.extend(new_facts);
            end += 1;
        }
        if end == start {
            return Err(state_form_error(
                "state-form byte capacity cannot hold one query block",
            ));
        }
        slices.push((start, end));
        start = end;
    }
    Ok(slices)
}

fn byte_balanced_pin_slices(
    blocks: &[ProjectionQuery<'_>],
    shard_count: usize,
) -> StateFormResult<Vec<(usize, usize)>> {
    if shard_count == 0 || blocks.len() < shard_count {
        return Err(state_form_error(format!(
            "cannot divide {} queries into {shard_count} shards",
            blocks.len()
        )));
    }
    let mut lower = blocks
        .iter()
        .map(|block| projection_utf8_bytes(std::slice::from_ref(block)))
        .collect::<StateFormResult<Vec<_>>>()?
        .into_iter()
        .max()
        .expect("nonempty blocks");
    let mut upper = projection_utf8_bytes(blocks)?;
    while lower < upper {
        let candidate = (lower + upper) / 2;
        if greedy_byte_slices(blocks, candidate)?.len() <= shard_count {
            upper = candidate;
        } else {
            lower = candidate + 1;
        }
    }
    let capacity = lower;
    let mut slices = greedy_byte_slices(blocks, capacity)?;
    while slices.len() < shard_count {
        let mut selected = None;
        for (index, &(start, end)) in slices.iter().enumerate() {
            if end - start <= 1 {
                continue;
            }
            let key = (
                projection_utf8_bytes(&blocks[start..end])?,
                end - start,
                usize::MAX - start,
            );
            if selected
                .as_ref()
                .is_none_or(|(_, _, _, selected_key)| key > *selected_key)
            {
                selected = Some((index, start, end, key));
            }
        }
        let (selected_index, start, end, _) = selected.ok_or_else(|| {
            state_form_error("state-form byte partition cannot reach shard census")
        })?;
        let mut best = None;
        for split in start + 1..end {
            let left = projection_utf8_bytes(&blocks[start..split])?;
            let right = projection_utf8_bytes(&blocks[split..end])?;
            let key = (left.max(right), left.abs_diff(right), split);
            if best.as_ref().is_none_or(|(_, best_key)| key < *best_key) {
                best = Some((split, key));
            }
        }
        let midpoint = best.expect("splittable slice has a midpoint").0;
        slices.splice(
            selected_index..=selected_index,
            [(start, midpoint), (midpoint, end)],
        );
    }
    if slices.first().map(|slice| slice.0) != Some(0)
        || slices.last().map(|slice| slice.1) != Some(blocks.len())
        || slices.windows(2).any(|window| window[0].1 != window[1].0)
    {
        return Err(state_form_error("state-form byte shard contiguity drifted"));
    }
    if slices
        .iter()
        .map(|&(start, end)| projection_utf8_bytes(&blocks[start..end]))
        .collect::<StateFormResult<Vec<_>>>()?
        .into_iter()
        .max()
        .is_some_and(|maximum| maximum > capacity)
    {
        return Err(state_form_error("state-form byte shard capacity drifted"));
    }
    Ok(slices)
}

fn pin_slices(
    blocks: &[ProjectionQuery<'_>],
    shard_count: usize,
    partition: ShardPartition,
) -> StateFormResult<Vec<(usize, usize)>> {
    match partition {
        ShardPartition::Bytes => byte_balanced_pin_slices(blocks, shard_count),
        ShardPartition::Count => balanced_pin_slices(blocks.len(), shard_count),
    }
}

fn relation_call_count(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    for index in 0..bytes.len() {
        if !bytes[index].is_ascii_lowercase()
            || index.checked_sub(1).is_some_and(|previous| {
                bytes[previous].is_ascii_alphanumeric() || bytes[previous] == b'_'
            })
        {
            continue;
        }
        let mut cursor = index + 1;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
        {
            cursor += 1;
        }
        count += usize::from(bytes.get(cursor) == Some(&b'('));
    }
    count
}

fn render_pin_shards(
    canonical: &str,
    family: &str,
    shard_count: usize,
    allow_prisoner: bool,
    partition: ShardPartition,
) -> StateFormResult<Vec<RenderedPinShard>> {
    let blocks = canonical_pin_query_blocks(canonical)?;
    let canonical_pairs = canonical_pin_query_pairs(canonical)?;
    let canonical_facts = canonical_pin_facts(canonical)?
        .into_iter()
        .collect::<HashSet<_>>();
    let aggregate_sha256 = sha256(canonical.as_bytes());
    let stream_sha256 = pin_query_stream_sha256(canonical)?;
    let mut shards = Vec::with_capacity(shard_count);
    let mut projected_pairs = Vec::new();
    let mut projected_facts = HashSet::new();
    for (zero_index, (start, end)) in pin_slices(&blocks, shard_count, partition)?
        .into_iter()
        .enumerate()
    {
        let selected = &blocks[start..end];
        let (projection, emitted_facts, selected_pairs) = render_pin_projection(selected)?;
        let index = zero_index + 1;
        let header =
            format!("# State-form {family} execution shard {index:02} of {shard_count:02}");
        let mut rendered = String::new();
        let _ = writeln!(rendered, "{SPDX_HEADER}");
        let _ = writeln!(rendered, "{header}");
        let _ = writeln!(rendered, "#");
        let _ = writeln!(
            rendered,
            "# Ephemeral lossless projection of the canonical aggregate pins."
        );
        let _ = writeln!(
            rendered,
            "# Canonical aggregate SHA-256: {aggregate_sha256}"
        );
        let _ = writeln!(
            rendered,
            "# Canonical query-stream SHA-256: {stream_sha256}"
        );
        let _ = writeln!(rendered, "# Partition strategy: {}", partition.as_str());
        let _ = writeln!(rendered, ":expect-pins {}", selected.len());
        rendered.push('\n');
        rendered.push_str(projection.trim_end());
        rendered.push('\n');
        validate_pin_surface(&rendered, &header, selected.len(), allow_prisoner)?;

        let mut serialized_facts = String::new();
        for fact in &emitted_facts {
            let _ = writeln!(serialized_facts, "{fact}");
        }
        let projection_utf8_bytes = projection.len();
        shards.push(RenderedPinShard {
            name: format!("{family}-{index:02}.pins.nibli"),
            query_count: selected.len(),
            fixture_fact_count: emitted_facts.len(),
            relation_call_count: relation_call_count(&projection),
            projection_utf8_bytes,
            utf8_bytes: rendered.len(),
            fixture_facts_sha256: sha256(serialized_facts.as_bytes()),
            query_stream_sha256: pin_pairs_sha256(&selected_pairs),
            projection_sha256: sha256(projection.as_bytes()),
            partition_strategy: partition,
            text: rendered,
        });
        projected_facts.extend(emitted_facts);
        projected_pairs.extend(selected_pairs);
    }
    if projected_pairs != canonical_pairs {
        return Err(state_form_error(format!(
            "state-form {family} shard query stream is not lossless"
        )));
    }
    if projected_facts != canonical_facts {
        return Err(state_form_error(format!(
            "state-form {family} shard fact union is not lossless"
        )));
    }
    Ok(shards)
}

fn render_shard_bundle_inner(partition: ShardPartition) -> StateFormResult<Vec<RenderedPinShard>> {
    let main = canonical_main_pins()?;
    let counterfactual = canonical_counterfactual_pins()?;
    let mut shards = render_pin_shards(main, "main", MAIN_SHARD_COUNT, false, partition)?;
    shards.extend(render_pin_shards(
        counterfactual,
        "counterfactual",
        COUNTERFACTUAL_SHARD_COUNT,
        false,
        partition,
    )?);
    let expected_names = (1..=MAIN_SHARD_COUNT)
        .map(|index| format!("main-{index:02}.pins.nibli"))
        .chain(
            (1..=COUNTERFACTUAL_SHARD_COUNT)
                .map(|index| format!("counterfactual-{index:02}.pins.nibli")),
        )
        .collect::<Vec<_>>();
    if shards
        .iter()
        .map(|shard| &shard.name)
        .ne(expected_names.iter())
    {
        return Err(state_form_error("state-form shard path inventory drifted"));
    }
    Ok(shards)
}

pub(crate) fn render_shard_bundle(
    partition: ShardPartition,
) -> Result<Vec<RenderedPinShard>, Error> {
    render_shard_bundle_inner(partition).map_err(public_error)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalShardIndexEntry {
    fixture_fact_count: usize,
    path: String,
    query_count: usize,
    query_stream_sha256: String,
    relation_call_count: usize,
    sha256: String,
    utf8_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalShardIndex {
    counterfactual: CanonicalShardIndexEntry,
    main: CanonicalShardIndexEntry,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShardPartitionIndex {
    byte_basis: String,
    contiguous_query_blocks: bool,
    counterfactual_shard_count: usize,
    main_shard_count: usize,
    strategy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShardIndexEntry {
    fixture_fact_count: usize,
    fixture_facts_sha256: String,
    path: String,
    projection_sha256: String,
    projection_utf8_bytes: usize,
    query_count: usize,
    query_stream_sha256: String,
    relation_call_count: usize,
    sha256: String,
    utf8_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShardIndex {
    canonical: CanonicalShardIndex,
    partition: ShardPartitionIndex,
    schema_version: String,
    shards: Vec<ShardIndexEntry>,
}

fn canonical_index_entry(
    path: &str,
    text: &str,
    expected_sha256: &str,
) -> StateFormResult<CanonicalShardIndexEntry> {
    Ok(CanonicalShardIndexEntry {
        fixture_fact_count: canonical_pin_facts(text)?.len(),
        path: path.to_owned(),
        query_count: query_count(text),
        query_stream_sha256: pin_query_stream_sha256(text)?,
        relation_call_count: relation_call_count(text),
        sha256: expected_sha256.to_owned(),
        utf8_bytes: text.len(),
    })
}

fn shard_index_entry(shard: &RenderedPinShard) -> ShardIndexEntry {
    ShardIndexEntry {
        fixture_fact_count: shard.fixture_fact_count,
        fixture_facts_sha256: shard.fixture_facts_sha256.clone(),
        path: shard.name.clone(),
        projection_sha256: shard.projection_sha256.clone(),
        projection_utf8_bytes: shard.projection_utf8_bytes,
        query_count: shard.query_count,
        query_stream_sha256: shard.query_stream_sha256.clone(),
        relation_call_count: shard.relation_call_count,
        sha256: sha256(shard.text.as_bytes()),
        utf8_bytes: shard.utf8_bytes,
    }
}

fn render_shard_index_inner(shards: &[RenderedPinShard]) -> StateFormResult<String> {
    let strategies = shards
        .iter()
        .map(|shard| shard.partition_strategy)
        .collect::<HashSet<_>>();
    if strategies.len() != 1 {
        return Err(state_form_error(
            "state-form shard partition strategy drifted",
        ));
    }
    let partition = *strategies.iter().next().expect("one strategy");

    let main = canonical_main_pins()?;
    let counterfactual = canonical_counterfactual_pins()?;
    let canonical = CanonicalShardIndex {
        counterfactual: canonical_index_entry(
            COUNTERFACTUAL_PINS_PATH,
            counterfactual,
            EXPECTED_COUNTERFACTUAL_PINS_SHA256,
        )?,
        main: canonical_index_entry(MAIN_PINS_PATH, main, EXPECTED_MAIN_PINS_SHA256)?,
    };

    let partition = ShardPartitionIndex {
        byte_basis: concat!(
            "exact rendered UTF-8 query projection with transitive ",
            "fixture closure and per-shard fact deduplication"
        )
        .to_owned(),
        contiguous_query_blocks: true,
        counterfactual_shard_count: COUNTERFACTUAL_SHARD_COUNT,
        main_shard_count: MAIN_SHARD_COUNT,
        strategy: partition.as_str().to_owned(),
    };
    let index = ShardIndex {
        canonical,
        partition,
        schema_version: "state-form-pin-shards-v2".to_owned(),
        shards: shards.iter().map(shard_index_entry).collect(),
    };
    let mut rendered = serde_json::to_string_pretty(&index)
        .map_err(|error| state_form_error(error.to_string()))?;
    rendered.push('\n');
    Ok(rendered)
}

pub(crate) fn render_shard_index(shards: &[RenderedPinShard]) -> Result<String, Error> {
    render_shard_index_inner(shards).map_err(public_error)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckReport {
    pub(crate) cards: usize,
    pub(crate) statements: usize,
    pub(crate) main_pins: usize,
    pub(crate) counterfactual_pins: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedStateForm {
    report: CheckReport,
    shards: Vec<RenderedPinShard>,
}

impl ValidatedStateForm {
    pub(crate) fn report(&self) -> &CheckReport {
        &self.report
    }
}

impl fmt::Display for CheckReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "state-form: PASS — {} cards, {} exact statements, {} main pins, {} counterfactual pins",
            self.cards, self.statements, self.main_pins, self.counterfactual_pins
        )
    }
}

fn validate_shard_bundle(shards: &[RenderedPinShard]) -> StateFormResult<()> {
    if shards.len() != MAIN_SHARD_COUNT + COUNTERFACTUAL_SHARD_COUNT {
        return Err(state_form_error(format!(
            "state-form shard inventory drifted: expected {}, found {}",
            MAIN_SHARD_COUNT + COUNTERFACTUAL_SHARD_COUNT,
            shards.len()
        )));
    }
    let main = shards[..MAIN_SHARD_COUNT]
        .iter()
        .map(|shard| shard.query_count)
        .sum::<usize>();
    let counterfactual = shards[MAIN_SHARD_COUNT..]
        .iter()
        .map(|shard| shard.query_count)
        .sum::<usize>();
    if main != MAIN_PIN_COUNT {
        return Err(state_form_error("state-form main shard count drifted"));
    }
    if counterfactual != COUNTERFACTUAL_PIN_COUNT {
        return Err(state_form_error(
            "state-form counterfactual shard count drifted",
        ));
    }
    let index = render_shard_index_inner(shards)?;
    serde_json::from_str::<ShardIndex>(&index)
        .map_err(|error| state_form_error(format!("invalid state-form shard index: {error}")))?;
    let expected = match shards[0].partition_strategy {
        ShardPartition::Bytes => EXPECTED_BYTE_INDEX_SHA256,
        ShardPartition::Count => EXPECTED_COUNT_INDEX_SHA256,
    };
    let actual = sha256(index.as_bytes());
    if actual != expected {
        return Err(state_form_error(format!(
            "state-form shard index changed: expected {expected}, found {actual}"
        )));
    }
    Ok(())
}

fn check_and_render_shards(
    snapshot: &SourceSnapshot,
) -> StateFormResult<(CheckReport, Vec<RenderedPinShard>)> {
    let semantic = semantic_source()?;
    let statements = validate_formal_source(snapshot.constitution())?;
    validate_delegation_markers(snapshot)?;

    let main = canonical_main_pins()?;
    validate_pin_surface(main, MAIN_HEADER, MAIN_PIN_COUNT, false)?;
    validate_main_manifest(main)?;
    let counterfactual_pins = canonical_counterfactual_pins()?;
    validate_pin_surface(
        counterfactual_pins,
        COUNTERFACTUAL_HEADER,
        COUNTERFACTUAL_PIN_COUNT,
        false,
    )?;
    validate_counterfactual_manifest(counterfactual_pins)?;
    let counterfactual = render_counterfactual(snapshot.constitution())?;
    validate_counterfactual_shape(snapshot.constitution(), &counterfactual)?;

    validate_artifact(
        MAIN_PINS_PATH,
        snapshot.main_pins(),
        main,
        EXPECTED_MAIN_PINS_SHA256,
    )?;
    validate_artifact(
        COUNTERFACTUAL_PATH,
        snapshot.counterfactual(),
        &counterfactual,
        EXPECTED_COUNTERFACTUAL_SHA256,
    )?;
    validate_artifact(
        COUNTERFACTUAL_PINS_PATH,
        snapshot.counterfactual_pins(),
        counterfactual_pins,
        EXPECTED_COUNTERFACTUAL_PINS_SHA256,
    )?;
    if semantic.branches.len() != RESULT_COUNT {
        return Err(state_form_error(
            "state-form result declaration count drifted",
        ));
    }
    let shards = render_shard_bundle_inner(ShardPartition::Bytes)?;
    validate_shard_bundle(&shards)?;
    Ok((
        CheckReport {
            cards: CARD_COUNT,
            statements: statements.len(),
            main_pins: query_count(main),
            counterfactual_pins: query_count(counterfactual_pins),
        },
        shards,
    ))
}

fn check_inner(snapshot: &SourceSnapshot) -> StateFormResult<CheckReport> {
    check_and_render_shards(snapshot).map(|(report, _)| report)
}

pub(crate) fn validate(snapshot: &SourceSnapshot) -> Result<ValidatedStateForm, Error> {
    check_and_render_shards(snapshot)
        .map(|(report, shards)| ValidatedStateForm { report, shards })
        .map_err(public_error)
}

pub(crate) fn check(_context: &Context, snapshot: &SourceSnapshot) -> Result<CheckReport, Error> {
    check_inner(snapshot).map_err(public_error)
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct FingerprintReport {
    branch_ir_sha256: String,
    constitution_sha256: String,
    counterfactual_pins_sha256: String,
    counterfactual_sha256: String,
    main_pins_sha256: String,
    rule_block_sha256: String,
    statement_fingerprints: Vec<String>,
}

pub(crate) fn fingerprints(_context: &Context, snapshot: &SourceSnapshot) -> Result<String, Error> {
    let result = (|| -> StateFormResult<String> {
        let statements = validate_formal_source(snapshot.constitution())?;
        let main = canonical_main_pins()?;
        let counterfactual = render_counterfactual(snapshot.constitution())?;
        let counterfactual_pins = canonical_counterfactual_pins()?;
        let report = FingerprintReport {
            branch_ir_sha256: EXPECTED_BRANCH_IR_SHA256.to_owned(),
            constitution_sha256: sha256(snapshot.constitution().as_bytes()),
            counterfactual_pins_sha256: sha256(counterfactual_pins.as_bytes()),
            counterfactual_sha256: sha256(counterfactual.as_bytes()),
            main_pins_sha256: sha256(main.as_bytes()),
            rule_block_sha256: EXPECTED_RULE_BLOCK_SHA256.to_owned(),
            statement_fingerprints: statements
                .iter()
                .map(|statement| sha256(statement.as_bytes()))
                .collect(),
        };
        let mut rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| state_form_error(error.to_string()))?;
        rendered.push('\n');
        Ok(rendered)
    })();
    result.map_err(public_error)
}

pub(crate) fn write_artifacts(
    context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<Vec<String>, Error> {
    let result = (|| -> StateFormResult<Vec<String>> {
        let rendered = [
            (MAIN_PINS_PATH, canonical_main_pins()?.to_owned()),
            (
                COUNTERFACTUAL_PATH,
                render_counterfactual(snapshot.constitution())?,
            ),
            (
                COUNTERFACTUAL_PINS_PATH,
                canonical_counterfactual_pins()?.to_owned(),
            ),
        ];
        let mut messages = Vec::new();
        for (relative, text) in rendered {
            let path = context.path(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| state_form_error(error.to_string()))?;
            }
            std::fs::write(&path, text.as_bytes())
                .map_err(|error| state_form_error(error.to_string()))?;
            let installed =
                std::fs::read(&path).map_err(|error| state_form_error(error.to_string()))?;
            if installed != text.as_bytes() {
                return Err(state_form_error(format!(
                    "state-form artifact write drifted: {}",
                    path.display()
                )));
            }
            messages.push(format!("wrote {relative}"));
        }
        Ok(messages)
    })();
    result.map_err(public_error)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShardWriteReport {
    pub(crate) main_shards: usize,
    pub(crate) counterfactual_shards: usize,
    pub(crate) queries: usize,
}

impl fmt::Display for ShardWriteReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "wrote state-form execution bundle: {} main shards, {} counterfactual shards, {} lossless queries",
            self.main_shards, self.counterfactual_shards, self.queries
        )
    }
}

pub(crate) fn write_shards(
    _context: &Context,
    snapshot: &SourceSnapshot,
    output_dir: &Path,
    partition: ShardPartition,
) -> Result<ShardWriteReport, Error> {
    let result = (|| -> StateFormResult<ShardWriteReport> {
        check_inner(snapshot)?;
        let shards = render_shard_bundle_inner(partition)?;
        validate_shard_bundle(&shards)?;
        let index = render_shard_index_inner(&shards)?;
        if output_dir.exists() && !output_dir.is_dir() {
            return Err(state_form_error(format!(
                "state-form shard output is not a directory: {}",
                output_dir.display()
            )));
        }
        std::fs::create_dir_all(output_dir).map_err(|error| state_form_error(error.to_string()))?;
        let mut existing =
            std::fs::read_dir(output_dir).map_err(|error| state_form_error(error.to_string()))?;
        if existing
            .next()
            .transpose()
            .map_err(|error| state_form_error(error.to_string()))?
            .is_some()
        {
            return Err(state_form_error(format!(
                "state-form shard output directory must be empty: {}",
                output_dir.display()
            )));
        }
        for shard in &shards {
            let path = output_dir.join(&shard.name);
            std::fs::write(&path, shard.text.as_bytes())
                .map_err(|error| state_form_error(error.to_string()))?;
            let installed =
                std::fs::read(&path).map_err(|error| state_form_error(error.to_string()))?;
            if installed != shard.text.as_bytes() {
                return Err(state_form_error(format!(
                    "state-form shard write drifted: {}",
                    path.display()
                )));
            }
        }
        let index_path = output_dir.join("index.json");
        std::fs::write(&index_path, index.as_bytes())
            .map_err(|error| state_form_error(error.to_string()))?;
        let installed =
            std::fs::read(&index_path).map_err(|error| state_form_error(error.to_string()))?;
        if installed != index.as_bytes() {
            return Err(state_form_error(format!(
                "state-form shard index write drifted: {}",
                index_path.display()
            )));
        }
        Ok(ShardWriteReport {
            main_shards: MAIN_SHARD_COUNT,
            counterfactual_shards: COUNTERFACTUAL_SHARD_COUNT,
            queries: MAIN_PIN_COUNT + COUNTERFACTUAL_PIN_COUNT,
        })
    })();
    result.map_err(public_error)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionReport {
    pub(crate) main_shards: usize,
    pub(crate) counterfactual_shards: usize,
    pub(crate) main_pins: usize,
    pub(crate) counterfactual_pins: usize,
    pub(crate) lines: Vec<String>,
}

impl fmt::Display for ExecutionReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.lines.join("\n"))
    }
}

fn require_execution_output(
    family: &str,
    shards: &[RenderedPinShard],
    output: RunOutput,
) -> StateFormResult<Vec<String>> {
    if output.exit_code != crate::pin::EXIT_OK {
        return Err(state_form_error(format!(
            "state-form {family} sharded execution failed with exit {}\n{}{}",
            output.exit_code, output.stdout, output.stderr
        )));
    }
    if output.files.len() != shards.len() {
        return Err(state_form_error(format!(
            "state-form {family} returned {} file reports for {} shards",
            output.files.len(),
            shards.len()
        )));
    }
    shards
        .iter()
        .zip(output.files)
        .map(|(shard, file)| {
            if file.display_name != shard.name
                || file.pins != shard.query_count
                || file.defects != 0
                || file.findings != 0
                || file.resolved != 0
                || file.harness != 0
            {
                return Err(state_form_error(format!(
                    "state-form {family} produced an unexpected report for {}: {file:?}",
                    shard.name
                )));
            }
            Ok(format!(
                "{}: nibli-pin: PASS — {} pins",
                shard.name, file.pins
            ))
        })
        .collect()
}

fn execute_family(
    context: &Context,
    family: &str,
    kb_name: &str,
    kb: &str,
    shards: &[RenderedPinShard],
) -> StateFormResult<Vec<String>> {
    let workers = match std::env::var("STATE_FORM_MAX_PARALLEL") {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                state_form_error("STATE_FORM_MAX_PARALLEL must be a positive integer")
            })?,
        Err(std::env::VarError::NotPresent) => 4,
        Err(error) => {
            return Err(state_form_error(format!(
                "cannot read STATE_FORM_MAX_PARALLEL: {error}"
            )));
        }
    }
    .min(shards.len().max(1));

    let family = family.to_owned();
    let kb_name = kb_name.to_owned();
    let kb = Arc::<str>::from(kb);
    let root = context.root().to_path_buf();
    let shards = Arc::<[RenderedPinShard]>::from(shards.to_vec());
    let next = Arc::new(AtomicUsize::new(0));
    let results = Arc::new(Mutex::new(
        std::iter::repeat_with(|| None)
            .take(shards.len())
            .collect::<Vec<Option<StateFormResult<String>>>>(),
    ));

    let scheduled = run_bounded(0..workers, workers, {
        let family = family.clone();
        let kb_name = kb_name.clone();
        let kb = Arc::clone(&kb);
        let root = root.clone();
        let shards = Arc::clone(&shards);
        let next = Arc::clone(&next);
        let results = Arc::clone(&results);
        move |_, _, cancellation| {
            let engine = PreparedPinEngine::new(&[LoadedSource::new(&kb_name, &kb)]);
            loop {
                if cancellation.is_cancelled() {
                    return Ok(());
                }
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(shard) = shards.get(index) else {
                    return Ok(());
                };
                if cancellation.is_cancelled() {
                    return Ok(());
                }
                let output = engine.run_files(
                    &[LoadedSource::new(&shard.name, &shard.text)],
                    PinOptions {
                        allow_shell: true,
                        working_directory: Some(&root),
                    },
                );
                let checked =
                    require_execution_output(&family, std::slice::from_ref(shard), output)
                        .and_then(|mut lines| {
                            lines.pop().ok_or_else(|| {
                                state_form_error(format!(
                                    "state-form {family} shard {} returned no summary",
                                    shard.name
                                ))
                            })
                        });
                let failure = checked.as_ref().err().cloned();
                results
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())[index] = Some(checked);
                if let Some(error) = failure {
                    return Err(error);
                }
            }
        }
    });

    let mut results = results
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Err(schedule_error) = scheduled {
        // Several already-running shards can finish together. Prefer the
        // lowest reviewed shard index among concrete failures so diagnostics
        // do not depend on worker completion order.
        if let Some(error) = results.iter().find_map(|result| match result {
            Some(Err(error)) => Some(error.clone()),
            _ => None,
        }) {
            return Err(error);
        }
        return Err(match schedule_error {
            ScheduleError::JobFailed { source, .. } => source,
            ScheduleError::InvalidWorkerCount => {
                state_form_error("state-form worker count must be positive")
            }
            ScheduleError::WorkerPanicked { index, message } => state_form_error(format!(
                "state-form execution worker {index} panicked: {message}"
            )),
            ScheduleError::CoordinatorLostWorker { active_indices } => state_form_error(format!(
                "state-form scheduler lost workers {active_indices:?}"
            )),
        });
    }

    results
        .iter_mut()
        .enumerate()
        .map(|(index, result)| {
            result.take().ok_or_else(|| {
                state_form_error(format!(
                    "state-form {family} shard {} returned no result",
                    shards[index].name
                ))
            })?
        })
        .collect()
}

/// Execute one reviewed state-form aggregate through its typed native shards.
/// This keeps focused verification faithful to the selected artifact while
/// avoiding the multi-minute single-engine aggregate bottleneck.
pub(crate) fn execute_focused_pin(
    context: &Context,
    pin_path: &str,
    snapshot: &SourceSnapshot,
) -> Result<RunOutput, Error> {
    let result = (|| -> StateFormResult<RunOutput> {
        // Focus narrows execution only. The complete source family and the
        // exact reviewed shard index must validate before either aggregate is
        // selected, matching the full verifier's fail-closed artifact gate.
        let validated = check_and_render_shards(snapshot)?;
        let shards = validated.1;
        let (family, kb_path, kb_text, selected, expected_pins) = if pin_path == MAIN_PINS_PATH {
            (
                "main",
                CONSTITUTION_PATH,
                snapshot.constitution(),
                &shards[..MAIN_SHARD_COUNT],
                MAIN_PIN_COUNT,
            )
        } else if pin_path == COUNTERFACTUAL_PINS_PATH {
            (
                "counterfactual",
                COUNTERFACTUAL_PATH,
                snapshot.counterfactual(),
                &shards[MAIN_SHARD_COUNT..],
                COUNTERFACTUAL_PIN_COUNT,
            )
        } else {
            return Err(state_form_error(format!(
                "not a state-form aggregate pin path: {pin_path}"
            )));
        };
        let lines = execute_family(context, family, kb_path, kb_text, selected)?;
        let mut stdout = lines.join("\n");
        if !stdout.is_empty() {
            stdout.push('\n');
        }
        let _ = writeln!(stdout, "nibli-pin: PASS — {expected_pins} pins");
        Ok(RunOutput {
            exit_code: crate::pin::EXIT_OK,
            stdout,
            pins: expected_pins,
            files: vec![FileOutput {
                display_name: pin_path.to_owned(),
                pins: expected_pins,
                ..FileOutput::default()
            }],
            ..RunOutput::default()
        })
    })();
    result.map_err(public_error)
}

fn execute_validated_inner(
    context: &Context,
    snapshot: &SourceSnapshot,
    validated: &ValidatedStateForm,
) -> StateFormResult<ExecutionReport> {
    let (main, counterfactual) = validated.shards.split_at(MAIN_SHARD_COUNT);
    let mut lines = execute_family(
        context,
        "main",
        CONSTITUTION_PATH,
        snapshot.constitution(),
        main,
    )?;
    lines.extend(execute_family(
        context,
        "counterfactual",
        COUNTERFACTUAL_PATH,
        snapshot.counterfactual(),
        counterfactual,
    )?);
    Ok(ExecutionReport {
        main_shards: main.len(),
        counterfactual_shards: counterfactual.len(),
        main_pins: main.iter().map(|shard| shard.query_count).sum(),
        counterfactual_pins: counterfactual.iter().map(|shard| shard.query_count).sum(),
        lines,
    })
}

pub(crate) fn execute_validated(
    context: &Context,
    snapshot: &SourceSnapshot,
    validated: &ValidatedStateForm,
) -> Result<ExecutionReport, Error> {
    execute_validated_inner(context, snapshot, validated).map_err(public_error)
}

pub(crate) fn execute(
    context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<ExecutionReport, Error> {
    let validated = validate(snapshot)?;
    execute_validated(context, snapshot, &validated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    fn snapshot(context: &Context) -> SourceSnapshot {
        load_snapshot(context).expect("load state-form sources")
    }

    fn assert_text_eq(actual: &str, expected: &str, label: &str) {
        if actual == expected {
            return;
        }
        let offset = actual
            .as_bytes()
            .iter()
            .zip(expected.as_bytes())
            .position(|(left, right)| left != right)
            .unwrap_or_else(|| actual.len().min(expected.len()));
        let line = actual.as_bytes()[..actual.len().min(offset)]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
            + 1;
        let start = offset.saturating_sub(120);
        let actual_end = actual.len().min(offset + 300);
        let expected_end = expected.len().min(offset + 300);
        panic!(
            "{label} differs at byte {offset}, line {line}; actual bytes={}, expected bytes={}\nactual: {:?}\nexpected: {:?}",
            actual.len(),
            expected.len(),
            String::from_utf8_lossy(&actual.as_bytes()[start.min(actual.len())..actual_end]),
            String::from_utf8_lossy(&expected.as_bytes()[start.min(expected.len())..expected_end]),
        );
    }

    #[test]
    fn reviewed_ir_regenerates_the_live_formal_block_byte_for_byte() {
        let context = context();
        let snapshot = snapshot(&context);
        let semantic = semantic_source().expect("validate reviewed semantic IR");
        assert_eq!(semantic.branches.len(), RESULT_COUNT);
        let rendered = canonical_rendered_block().expect("render formal block");
        let actual = extract_block(snapshot.constitution()).expect("extract live formal block");
        assert_text_eq(&rendered, actual, "formal block");
        assert_eq!(sha256(rendered.as_bytes()), EXPECTED_RENDERED_BLOCK_SHA256);
        assert_eq!(formal_statements(&rendered).len(), STATEMENT_COUNT);
    }

    #[test]
    fn reviewed_ir_regenerates_every_live_artifact_byte_for_byte() {
        let context = context();
        let snapshot = snapshot(&context);
        let main = canonical_main_pins().expect("render main pins");
        assert_text_eq(main, snapshot.main_pins(), MAIN_PINS_PATH);
        let counterfactual =
            render_counterfactual(snapshot.constitution()).expect("render counterfactual");
        assert_text_eq(
            &counterfactual,
            snapshot.counterfactual(),
            COUNTERFACTUAL_PATH,
        );
        let counterfactual_pins =
            canonical_counterfactual_pins().expect("render counterfactual pins");
        assert_text_eq(
            counterfactual_pins,
            snapshot.counterfactual_pins(),
            COUNTERFACTUAL_PINS_PATH,
        );
    }

    #[test]
    fn focused_execution_preflight_rejects_nonselected_artifact_drift() {
        let context = context();
        let mut snapshot = snapshot(&context);
        snapshot.counterfactual_pins = Arc::from(format!(
            "{}# non-selected artifact drift\n",
            snapshot.counterfactual_pins()
        ));
        let error = execute_focused_pin(&context, MAIN_PINS_PATH, &snapshot)
            .expect_err("focused main execution must validate the whole source family first");
        assert!(
            error.to_string().contains(COUNTERFACTUAL_PINS_PATH),
            "unexpected focused preflight error: {error}"
        );
    }

    #[test]
    fn check_and_fingerprint_surfaces_preserve_the_contract() {
        let context = context();
        let snapshot = snapshot(&context);
        let report = check(&context, &snapshot).expect("check state-form family");
        assert_eq!(
            report.to_string(),
            "state-form: PASS — 51 cards, 274 exact statements, 391 main pins, 51 counterfactual pins"
        );
        let output = fingerprints(&context, &snapshot).expect("render fingerprints");
        let decoded: Value = serde_json::from_str(&output).expect("parse fingerprints");
        assert_eq!(decoded["branch_ir_sha256"], EXPECTED_BRANCH_IR_SHA256);
        assert_eq!(decoded["rule_block_sha256"], EXPECTED_RULE_BLOCK_SHA256);
        assert_eq!(
            decoded["statement_fingerprints"]
                .as_array()
                .expect("fingerprint array")
                .len(),
            STATEMENT_COUNT
        );
    }

    #[test]
    fn structural_mutations_are_rejected() {
        let rendered = canonical_rendered_block().expect("render formal block");
        let without_guard = rendered.replacen(CURRENT_REVIEW_GUARD, "", 1);
        let statements = formal_statements(&without_guard);
        assert_eq!(
            validate_rule_surface(&statements)
                .expect_err("missing independent-review guard must fail")
                .to_string(),
            "independent-current-review guard must occur exactly once"
        );

        let main = canonical_main_pins().expect("render main pins");
        let injected = main.replacen(
            &format!(":expect-pins {MAIN_PIN_COUNT}\n"),
            &format!(":expect-pins {MAIN_PIN_COUNT}\ncomplete(Injected, FSPOW_001, Record).\n"),
            1,
        );
        assert_eq!(
            validate_pin_surface(&injected, MAIN_HEADER, MAIN_PIN_COUNT, false)
                .expect_err("derived fixture injection must fail")
                .to_string(),
            "state-form fixtures must not assert derived declarations or authority"
        );

        let context = context();
        let mut snapshot = snapshot(&context);
        let mut chapters = snapshot.chapter_pins.to_vec();
        let delegated = chapters
            .iter_mut()
            .find(|(path, _)| path == DELEGATED_PIN_PATHS[0])
            .expect("delegated chapter pin");
        delegated.1 = Arc::from(delegated.1.replacen(DELEGATION_MARKER, "", 1));
        snapshot.chapter_pins = chapters.into();
        assert!(validate_delegation_markers(&snapshot).is_err());
    }

    #[test]
    fn byte_and_count_shards_are_lossless_and_match_reviewed_indexes() {
        for (partition, expected_index) in [
            (ShardPartition::Bytes, EXPECTED_BYTE_INDEX_SHA256),
            (ShardPartition::Count, EXPECTED_COUNT_INDEX_SHA256),
        ] {
            let shards = render_shard_bundle_inner(partition).expect("render shard bundle");
            validate_shard_bundle(&shards).expect("validate shard losslessness");
            assert_eq!(shards.len(), MAIN_SHARD_COUNT + COUNTERFACTUAL_SHARD_COUNT);
            assert_eq!(
                shards[..MAIN_SHARD_COUNT]
                    .iter()
                    .map(|shard| shard.query_count)
                    .sum::<usize>(),
                MAIN_PIN_COUNT
            );
            assert_eq!(
                shards[MAIN_SHARD_COUNT..]
                    .iter()
                    .map(|shard| shard.query_count)
                    .sum::<usize>(),
                COUNTERFACTUAL_PIN_COUNT
            );
            let index = render_shard_index_inner(&shards).expect("render shard index");
            assert_eq!(sha256(index.as_bytes()), expected_index);
        }
    }

    #[test]
    fn artifact_and_shard_writers_install_the_rendered_bytes() {
        let live_context = context();
        let snapshot = snapshot(&live_context);
        let temporary = tempfile::tempdir().expect("temporary directory");
        let isolated = Context::from_test_root(temporary.path().to_path_buf());

        let messages = write_artifacts(&isolated, &snapshot).expect("write artifacts");
        assert_eq!(messages.len(), 3);
        assert_eq!(
            std::fs::read_to_string(isolated.path(MAIN_PINS_PATH)).expect("read main pins"),
            canonical_main_pins().expect("render main pins")
        );
        assert_eq!(
            std::fs::read_to_string(isolated.path(COUNTERFACTUAL_PATH))
                .expect("read counterfactual"),
            render_counterfactual(snapshot.constitution()).expect("render counterfactual")
        );
        assert_eq!(
            std::fs::read_to_string(isolated.path(COUNTERFACTUAL_PINS_PATH))
                .expect("read counterfactual pins"),
            canonical_counterfactual_pins().expect("render counterfactual pins")
        );

        let output = temporary.path().join("shards");
        let report = write_shards(&isolated, &snapshot, &output, ShardPartition::Bytes)
            .expect("write shards");
        assert_eq!(report.main_shards, MAIN_SHARD_COUNT);
        assert_eq!(report.counterfactual_shards, COUNTERFACTUAL_SHARD_COUNT);
        assert_eq!(report.queries, MAIN_PIN_COUNT + COUNTERFACTUAL_PIN_COUNT);
        let expected =
            render_shard_bundle_inner(ShardPartition::Bytes).expect("render expected shard bundle");
        for shard in &expected {
            assert_eq!(
                std::fs::read(output.join(&shard.name)).expect("read installed shard"),
                shard.text.as_bytes(),
                "installed shard differs: {}",
                shard.name
            );
        }
        assert_eq!(
            std::fs::read_to_string(output.join("index.json")).expect("read shard index"),
            render_shard_index_inner(&expected).expect("render expected index")
        );
    }

    #[test]
    #[ignore = "loads the live 5.6 MB knowledge bases into two engines"]
    fn live_first_main_and_counterfactual_shards_execute_in_process() {
        let context = context();
        let snapshot = snapshot(&context);
        check(&context, &snapshot).expect("check state-form family");
        let shards = render_shard_bundle_inner(ShardPartition::Bytes).expect("render shards");
        let main = execute_family(
            &context,
            "main",
            CONSTITUTION_PATH,
            snapshot.constitution(),
            &shards[..1],
        )
        .expect("execute first main shard");
        let counterfactual = execute_family(
            &context,
            "counterfactual",
            COUNTERFACTUAL_PATH,
            snapshot.counterfactual(),
            &shards[MAIN_SHARD_COUNT..MAIN_SHARD_COUNT + 1],
        )
        .expect("execute first counterfactual shard");
        assert_eq!(main.len(), 1);
        assert_eq!(counterfactual.len(), 1);
    }
}
