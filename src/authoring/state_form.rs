// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Error;
use crate::context::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fmt::Write as _;
use std::sync::OnceLock;

pub(crate) const CONSTITUTION_PATH: &str = "new-book-plans/constitution.nibli";

pub(crate) const MAIN_PINS_PATH: &str = "new-book-plans/state-form.pins.nibli";

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

const ACCEPTANCE_CASE_IDS: &[&str] = &[
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
    "FSACC-020-office-integrity",
    "FSACC-021-political-finance",
    "FSACC-022-districting-purpose",
];

type Field = [String; 2];

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SemanticSource {
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
    body . extend (["observe($source, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)" , "observe($review, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)" , "observe($source, $result_reconciliation, $result, ResultScope)" , "observe($review, $result_reconciliation, $result, ResultScope)" , "observe($source, $result_reconciliation, $record, StateFormRecordScope)" , "observe($review, $result_reconciliation, $record, StateFormRecordScope)" , "observe($source, $result_reconciliation, $version, SourceVersionScope)" , "observe($review, $result_reconciliation, $version, SourceVersionScope)" ,] . into_iter () . map (str :: to_owned)) ;
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
        if let Some(vocabulary) = vocabulary_for_scope(&field[1]) {
            body.push(format!("member({}, {vocabulary})", field[0]));
        }
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
        rules . push (format ! ("{authority_quantifier}{result_head} & {authority_body} -> authority({holder}, {power}, $record).")) ;
    }
    Ok(rules)
}

#[doc = " The appointment anti-capture finding's named grounds. The ratified sentence"]
#[doc = " enumerates five source kinds and requires both control modes to be"]
#[doc = " observable; before this table the whole incompatibility was one attested"]
#[doc = " token, so a finding that examined one appointing source and never looked at"]
#[doc = " the coalition passed by attestation. Each member is concluded by exactly one"]
#[doc = " ground rule and gates the reviewed-result rule through `member/2`."]
const APPOINTMENT_CONTROL_VOCABULARIES: [(&str, &str, &[&str]); 2] = [
    (
        "AppointmentControlSourceKindScope",
        "AppointmentControlSourceKindVocabulary",
        &[
            "CurrentGovernmentAppointmentSource",
            "ChamberAppointmentSource",
            "PartyCoalitionAppointmentSource",
            "ProfessionAppointmentSource",
            "SingleAppointingBodyAppointmentSource",
        ],
    ),
    (
        "AppointmentControlModeScope",
        "AppointmentControlModeVocabulary",
        &["DirectAppointmentControl", "DeFactoAppointmentControl"],
    ),
];

#[doc = " The office-integrity finding's named grounds, from the 2026-09-09 ruling on"]
#[doc = " conflicts, gifts, and revolving doors. The bearer is always the office: the"]
#[doc = " interest kinds are the holder's own, the household's, or a controlled"]
#[doc = " entity's; the counterparty relationships are what make a benefit or a"]
#[doc = " post-office move an incompatibility; the modes are the three ratified"]
#[doc = " incompatibilities. Amounts and periods are democratic law and appear nowhere"]
#[doc = " here. Every branch carries this family because every branch is an office act."]
const OFFICE_INTEGRITY_VOCABULARIES: [(&str, &str, &[&str]); 3] = [
    (
        "MaterialInterestKindScope",
        "MaterialInterestKindVocabulary",
        &[
            "OwnMaterialInterest",
            "HouseholdMaterialInterest",
            "ControlledEntityMaterialInterest",
        ],
    ),
    (
        "CounterpartyRelationshipKindScope",
        "CounterpartyRelationshipKindVocabulary",
        &[
            "RegulatedCounterparty",
            "ContractingCounterparty",
            "AdjudicatedCounterparty",
            "AppointedCounterparty",
        ],
    ),
    (
        "OfficeIntegrityModeScope",
        "OfficeIntegrityModeVocabulary",
        &[
            "ConflictedAct",
            "CounterpartyBenefit",
            "FormerHolderCounterpartyDealing",
        ],
    ),
];

#[doc = " The political-finance finding's named grounds, from the 2026-09-09 ruling on"]
#[doc = " the money-and-influence record. The ruling makes the record fully"]
#[doc = " consequential: a finding that a recipient of a named kind took an instrument"]
#[doc = " of a named kind from a payer of a named kind withholds the office's or"]
#[doc = " candidacy's authority, which is what makes the three inherited treasury"]
#[doc = " barriers catchable — each is otherwise one rule reading no treasury,"]
#[doc = " payment, candidate, or party. `ControllingPartyPayer` is the shell-actor"]
#[doc = " test: where the nominal payer is controlled by another, the controlling"]
#[doc = " party is what the attesters must name. No amount, threshold, or publication"]
#[doc = " floor appears here; those are democratic law."]
const POLITICAL_FINANCE_VOCABULARIES: [(&str, &str, &[&str]); 3] = [
    (
        "PoliticalFinancePayerKindScope",
        "PoliticalFinancePayerKindVocabulary",
        &[
            "PersonPayer",
            "EnterpriseTreasuryPayer",
            "UnionPayer",
            "VoluntaryCivicAssociationPayer",
            "OutsideJurisdictionPayer",
            "ControllingPartyPayer",
        ],
    ),
    (
        "PoliticalFinanceInstrumentKindScope",
        "PoliticalFinanceInstrumentKindVocabulary",
        &[
            "ContributionInstrument",
            "ExpenditureInstrument",
            "InKindProvisionInstrument",
            "LoanOrGuaranteeInstrument",
            "PurchasedIndependentAdvocacyInstrument",
        ],
    ),
    (
        "PoliticalFinanceRecipientKindScope",
        "PoliticalFinanceRecipientKindVocabulary",
        &[
            "CandidateRecipient",
            "PartyOrCoalitionRecipient",
            "BallotQuestionRecipient",
            "OfficeHolderRecipient",
            "PublicDecisionRecipient",
        ],
    ),
];

#[doc = " The districting finding's named grounds, from the 2026-09-09 ruling on"]
#[doc = " district plans. The ruling is deliberately qualitative: a plan drawn for a"]
#[doc = " forbidden purpose is a legal incompatibility, and no metric, threshold,"]
#[doc = " tolerance, or compactness value appears anywhere — democratic law chooses"]
#[doc = " whatever measure it uses to *evidence* a purpose, and the constitution"]
#[doc = " names only the purposes. `DistrictMagnitudeScope`, the electoral card's"]
#[doc = " testability slot, is untouched. Unlike office integrity and political"]
#[doc = " finance this family is **anchored**, not universal: a district plan is not"]
#[doc = " every office act, so it binds card 35's four seat-allocation branches."]
const DISTRICTING_PURPOSE_VOCABULARIES: [(&str, &str, &[&str]); 1] = [(
    "ForbiddenDistrictingPurposeScope",
    "ForbiddenDistrictingPurposeVocabulary",
    &[
        "PartyOrCoalitionEntrenchment",
        "IncumbentEntrenchment",
        "ProtectedGroundDilution",
    ],
)];

#[doc = " A family of examined kinds behind one absence anchor. `universal` families"]
#[doc = " bind every branch; the others bind the branches that carry the anchor."]
struct ExaminedKindFamily {
    name: &'static str,
    anchor: (&'static str, &'static str),
    universal: bool,
    vocabularies: &'static [(&'static str, &'static str, &'static [&'static str])],
}

const EXAMINED_KIND_FAMILIES: [ExaminedKindFamily; 4] = [
    ExaminedKindFamily {
        name: "appointment anti-capture",
        anchor: ("NoMajorityDirectOrDeFactoControl", "AntiCaptureScope"),
        universal: false,
        vocabularies: &APPOINTMENT_CONTROL_VOCABULARIES,
    },
    ExaminedKindFamily {
        name: "office integrity",
        anchor: (
            "NoConflictGiftOrRevolvingDoorIncompatibility",
            "OfficeIntegrityScope",
        ),
        universal: true,
        vocabularies: &OFFICE_INTEGRITY_VOCABULARIES,
    },
    ExaminedKindFamily {
        name: "political finance",
        anchor: (
            "NoProhibitedPoliticalFinanceIncompatibility",
            "PoliticalFinanceScope",
        ),
        universal: true,
        vocabularies: &POLITICAL_FINANCE_VOCABULARIES,
    },
    ExaminedKindFamily {
        name: "districting purpose",
        anchor: ("NoForbiddenDistrictingPurpose", "DistrictingPurposeScope"),
        universal: false,
        vocabularies: &DISTRICTING_PURPOSE_VOCABULARIES,
    },
];

fn vocabulary_for_scope(scope: &str) -> Option<&'static str> {
    EXAMINED_KIND_FAMILIES
        .iter()
        .flat_map(|family| family.vocabularies.iter())
        .find(|(field_scope, _, _)| *field_scope == scope)
        .map(|(_, vocabulary, _)| *vocabulary)
}

fn render_vocabulary_rules() -> Vec<String> {
    let mut rules = Vec::new();
    for family in &EXAMINED_KIND_FAMILIES {
        for (scope, vocabulary, members) in family.vocabularies {
            for member in *members {
                rules . push (format ! ("all $source: all $record: observe($source, $record, {member}, {scope}) -> member({member}, {vocabulary}).")) ;
            }
        }
    }
    rules
}

fn draft_rule_block(source: &SemanticSource) -> StateFormResult<Vec<String>> {
    let mut rules = vec![render_current_rule()];
    rules.extend(render_vocabulary_rules());
    for branch in &source.branches {
        rules.extend(v2_rules_for_branch(branch)?);
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
    for number in branches
        .iter()
        .map(|branch| branch.card)
        .collect::<BTreeSet<_>>()
    {
        let (index, branch) = branches
            .iter()
            .enumerate()
            .find(|(_, branch)| branch.card == number)
            .ok_or_else(|| state_form_error(format!("FSPOW_{number:03} has no branch")))?;
        let holder = branch.authority_holders.first().ok_or_else(|| {
            state_form_error(format!("FSPOW_{number:03} has no authority holder"))
        })?;
        rows.push((index, holder.as_str()));
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
    builder.header(ACCEPTANCE_CASE_IDS[19]);
    builder.existing(7, "appropriation_authorization", "FSBOD_02")?;
    builder.negative(
        7,
        "appropriation_authorization",
        "FSBOD_02",
        "SFAcc020NoOfficeIntegrity",
        &[AtomSelector::all(&["OfficeIntegrityScope"])],
    )?;
    builder.negative(
        7,
        "appropriation_authorization",
        "FSBOD_02",
        "SFAcc020InterestUnexamined",
        &[AtomSelector::all(&["MaterialInterestKindScope"])],
    )?;
    builder.negative(
        7,
        "appropriation_authorization",
        "FSBOD_02",
        "SFAcc020CounterpartyUnexamined",
        &[AtomSelector::all(&["CounterpartyRelationshipKindScope"])],
    )?;
    builder.negative(
        7,
        "appropriation_authorization",
        "FSBOD_02",
        "SFAcc020ModeUnexamined",
        &[AtomSelector::all(&["OfficeIntegrityModeScope"])],
    )?;
    builder.header(ACCEPTANCE_CASE_IDS[20]);
    builder.existing(36, "adult_resident_candidacy", "FSBOD_06")?;
    builder.negative(
        36,
        "adult_resident_candidacy",
        "FSBOD_06",
        "SFAcc021NoPoliticalFinance",
        &[AtomSelector::all(&["PoliticalFinanceScope"])],
    )?;
    builder.negative(
        36,
        "adult_resident_candidacy",
        "FSBOD_06",
        "SFAcc021PayerUnexamined",
        &[AtomSelector::all(&["PoliticalFinancePayerKindScope"])],
    )?;
    builder.negative(
        36,
        "adult_resident_candidacy",
        "FSBOD_06",
        "SFAcc021InstrumentUnexamined",
        &[AtomSelector::all(&["PoliticalFinanceInstrumentKindScope"])],
    )?;
    builder.negative(
        36,
        "adult_resident_candidacy",
        "FSBOD_06",
        "SFAcc021RecipientUnexamined",
        &[AtomSelector::all(&["PoliticalFinanceRecipientKindScope"])],
    )?;
    builder.header(ACCEPTANCE_CASE_IDS[21]);
    builder.existing(35, "people_seat_allocation", "FSBOD_01")?;
    builder.negative(
        35,
        "people_seat_allocation",
        "FSBOD_01",
        "SFAcc022NoDistrictingPurpose",
        &[AtomSelector::all(&["DistrictingPurposeScope"])],
    )?;
    builder.negative(
        35,
        "people_seat_allocation",
        "FSBOD_01",
        "SFAcc022PurposeUnexamined",
        &[AtomSelector::all(&["ForbiddenDistrictingPurposeScope"])],
    )?;
    builder
        .lines
        .push("# <STATE-FORM-ACCEPTANCE-CASES-END>".to_owned());
    builder.lines.push(String::new());
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
        ":expect-pins 0".to_owned(),
        String::new(),
        "# <STATE-FORM-GENERIC-POSITIVE-BEGIN>".to_owned(),
    ];
    let rows = branch_holder_rows(&source.branches);
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
    }
    lines.push("# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-END>".to_owned());
    lines.push(String::new());
    let (acceptance, _) = render_acceptance_cases(&source.branches, &registry)?;
    lines.extend(acceptance);
    let rendered = format!("{}\n", lines.join("\n").trim_end());
    Ok(rendered.replacen(
        ":expect-pins 0",
        &format!(":expect-pins {}", query_count(&rendered)),
        1,
    ))
}

fn render_counterfactual_pins(source: &SemanticSource) -> StateFormResult<String> {
    let mut lines = vec![
        SPDX_HEADER.to_owned(),
        COUNTERFACTUAL_HEADER.to_owned(),
        ":expect-pins 0".to_owned(),
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
        lines . push (format ! ("# FS-POW-{:03} counterfactual: removing the independent-current-review guard derives authority." , branch . card)) ;
        append_fixture_query(&mut lines, branch, holder, &fixture, true)?;
        count += 1;
    }
    let rendered = format!("{}\n", lines.join("\n").trim_end());
    Ok(rendered.replacen(":expect-pins 0", &format!(":expect-pins {count}"), 1))
}

fn variable_regex() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").expect("valid variable regex"))
}

fn query_count(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("? ")).count()
}

#[derive(Clone, Debug)]
struct ProjectionQuery<'a> {
    query_line: &'a str,
    expectation_line: &'a str,
    facts: Vec<&'a str>,
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

pub(crate) fn generate(
    context: &Context,
    export: &mut crate::authoring::Export,
) -> Result<(), Error> {
    let source: SemanticSource =
        serde_json::from_str(&context.read("new-book-plans/state-form-source.json")?)?;
    let rules = render_formal_block(&source).map_err(public_error)?;
    let main = render_main_pins(&source).map_err(public_error)?;
    let counterfactual = render_counterfactual_pins(&source).map_err(public_error)?;
    let constitution = context.read(CONSTITUTION_PATH)?;
    let start = constitution
        .find(BEGIN)
        .ok_or_else(|| Error::new("missing state-form opening marker"))?;
    let stop = constitution[start..]
        .find(END)
        .map(|offset| start + offset + END.len())
        .ok_or_else(|| Error::new("missing state-form closing marker"))?;
    let updated = format!(
        "{}{}{}",
        &constitution[..start],
        rules.trim_end(),
        &constitution[stop..]
    );
    std::fs::write(context.path(CONSTITUTION_PATH), updated)?;
    std::fs::write(context.path(MAIN_PINS_PATH), main)?;
    std::fs::write(context.path(COUNTERFACTUAL_PINS_PATH), counterfactual)?;
    export_cases(context, export)
}

pub(crate) fn export_cases(
    context: &Context,
    export: &mut crate::authoring::Export,
) -> Result<(), Error> {
    use crate::authoring::{Base, Edit};
    let constitution = context.read(CONSTITUTION_PATH)?;
    let current = render_current_rule();
    let edits = vec![Edit {
        before: format!("{current}\n"),
        after: format!("{}\n", current.replacen(CURRENT_REVIEW_GUARD, "", 1)),
    }];
    crate::authoring::apply_edits(&constitution, &edits)?;
    export.bases.insert(
        "state-form-no-independent-review".into(),
        Base {
            base: Some("live".into()),
            edits,
            path: None,
        },
    );
    let main = context.read(MAIN_PINS_PATH)?;
    let counterfactual = context.read(COUNTERFACTUAL_PINS_PATH)?;
    for (family, base, canonical, shards, scan) in [
        ("main", "live", main.as_str(), MAIN_SHARD_COUNT, true),
        (
            "counterfactual",
            "state-form-no-independent-review",
            counterfactual.as_str(),
            COUNTERFACTUAL_SHARD_COUNT,
            false,
        ),
    ] {
        let blocks = canonical_pin_query_blocks(canonical).map_err(public_error)?;
        let mut actual_pairs = Vec::new();
        let mut actual_facts = HashSet::new();
        for (index, (start, end)) in byte_balanced_pin_slices(&blocks, shards)
            .map_err(public_error)?
            .into_iter()
            .enumerate()
        {
            let (projection, facts, pairs) =
                render_pin_projection(&blocks[start..end]).map_err(public_error)?;
            actual_pairs.extend(pairs);
            actual_facts.extend(facts);
            let pins = format!(
                "{SPDX_HEADER}\n# State-form {family} isolated examples.\n:expect-pins {}\n\n{projection}\n",
                end - start
            );
            export.add_case(
                context,
                &format!("state-form/{family}-{:02}", index + 1),
                base,
                "",
                &[("expect", &pins)],
                Vec::new(),
                scan,
            )?;
        }
        if actual_pairs != canonical_pin_query_pairs(canonical).map_err(public_error)?
            || actual_facts
                != canonical_pin_facts(canonical)
                    .map_err(public_error)?
                    .into_iter()
                    .collect()
        {
            return Err(Error::new(format!(
                "state-form {family} extraction lost an expectation or fixture"
            )));
        }
    }
    Ok(())
}
