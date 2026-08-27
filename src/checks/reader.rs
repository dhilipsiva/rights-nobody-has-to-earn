// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native reader-evidence contract validator, renderer, and evaluator.
//!
//! The reviewed JSON remains the sole machine-readable source. This module
//! deliberately keeps validation and deterministic evaluation reusable so the
//! independent evidence-admission gate can bind the same native contracts
//! without a Python import or subprocess.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Component, Path};
use std::process::Command;
use std::sync::OnceLock;

use regex::Regex;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::{canonical_json, sha256};

const DEFAULT_SOURCE: &str = "new-book-plans/reader-evidence.json";
const DEFAULT_OUTPUT: &str = "new-book-plans/reader-evidence.md";
const PROTOCOL_DECISION: &str = "new-book-plans/book-1-reader-evidence-protocol-decision.md";

const ROOT_KEYS: &[&str] = &[
    "spdx",
    "schema_version",
    "contract_id",
    "protocol_decision_ref",
    "threshold_status",
    "holdout_status",
    "result",
    "history_transition",
    "route",
    "claim",
    "privacy",
    "protocol",
    "pilot",
    "threshold_rule",
    "ratification",
    "holdout",
    "acceptance",
];
const HISTORY_TRANSITION_KEYS: &[&str] = &[
    "previous_source_commit",
    "previous_source_sha256",
    "previous_history_head_sha256",
    "history_head_sha256",
];
const ROUTE_KEYS: &[&str] = &[
    "route_id",
    "route_status",
    "evidence_contract_status",
    "structural_checker_binding",
    "reviewer_custody_attestation",
    "evidence_admission_gate_binding",
    "negative_control_status",
];
const CLAIM_KEYS: &[&str] = &["claim_id", "posture", "disposition", "result_ref"];
const PRIVACY_KEYS: &[&str] = &[
    "public_record_policy",
    "allowed_public_record_kinds",
    "excluded_from_repository",
    "freshness_attestation_boundary",
];
const PROTOCOL_KEYS: &[&str] = &[
    "decision_sha256",
    "method",
    "evaluation_order",
    "aggregate_offset_prohibited",
    "required_targets",
    "disclosed_limits",
    "ethics_terms",
    "freshness_terms",
    "non_substitution",
];
const TARGET_KEYS: &[&str] = &["target_id", "description"];
const PILOT_KEYS: &[&str] = &[
    "pilot_status",
    "control_status",
    "active_attempt_id",
    "attempts",
];
const PILOT_ATTEMPT_KEYS: &[&str] = &[
    "attempt_id",
    "previous_attempt_sha256",
    "attempt_status",
    "control_status",
    "void_reason_code",
    "voided_at",
    "prerequisites",
    "pre_registration",
    "tested_snapshot",
    "session_records",
    "deviations",
    "custody_attestations",
    "receipt",
    "decision_packet",
    "sensitivity_brief",
    "attempt_sha256",
];
const PREREQUISITE_KEYS: &[&str] = &[
    "readers_map_ref",
    "glossary_ref",
    "accessible_navigation_ref",
];
const THRESHOLD_RULE_KEYS: &[&str] = &[
    "rule_id",
    "severity_taxonomy",
    "misconceptions",
    "core_misconception_ids",
    "core_failure_mode",
    "repetition_unit",
    "denominator",
    "core_failure_threshold",
    "required_target_thresholds",
    "non_core_thresholds",
    "minimum_evaluable_evidence",
    "policies",
    "evaluation_order",
    "aggregate_offset_prohibited",
    "rule_sha256",
];
const POLICY_KEYS: &[&str] = &[
    "missing",
    "ambiguous",
    "multiply_coded",
    "withdrawn",
    "excluded",
    "unclassified",
    "rounding",
    "coder_adjudication",
];
const SEVERITY_KEYS: &[&str] = &[
    "severity_id",
    "label",
    "definition",
    "classification_boundary",
];
const MISCONCEPTION_KEYS: &[&str] = &["misconception_id", "definition", "severity_id", "core"];
const THRESHOLD_SPEC_KEYS: &[&str] = &[
    "threshold_id",
    "metric",
    "operator",
    "value_kind",
    "value",
    "unit",
    "denominator",
    "scope_refs",
    "evaluator_ref",
];
const TARGET_THRESHOLD_KEYS: &[&str] = &["target_id", "threshold"];
const SEVERITY_THRESHOLD_KEYS: &[&str] = &["severity_id", "threshold"];
const ARTIFACT_KEYS: &[&str] = &["artifact_id", "ref", "sha256"];
const FREEZE_BINDING_KEYS: &[&str] = &[
    "binding_id",
    "binding_type",
    "bound_payload_sha256",
    "attested_payload_sha256",
    "ref",
    "attestation_sha256",
    "frozen_at",
];
const REVIEWER_ATTESTATION_KEYS: &[&str] = &[
    "attestation_id",
    "scope",
    "evidence_gate_sha256",
    "ref",
    "sha256",
    "attested_date",
];
const PILOT_PRE_REGISTRATION_KEYS: &[&str] = &[
    "study_id",
    "registered_date",
    "predecessor_attempt_sha256",
    "prior_history_head_sha256",
    "fixed_protocol_sha256",
    "protocol",
    "instrument",
    "rubric",
    "sample_rule",
    "disclosure_set",
    "ethics_terms",
    "provisional_rule",
    "freeze_binding",
    "pre_registration_sha256",
];
const SESSION_KEYS: &[&str] = &[
    "study_id",
    "record_commitment_sha256",
    "admissibility",
    "target_outcomes",
    "misconception_outcomes",
    "deviation_ids",
    "custody_attestation_ids",
];
const TARGET_OUTCOME_KEYS: &[&str] = &["target_id", "status", "adjudication"];
const MISCONCEPTION_OUTCOME_KEYS: &[&str] = &[
    "misconception_id",
    "status",
    "occurrences",
    "opportunities",
    "adjudication",
];
const PILOT_RECEIPT_KEYS: &[&str] = &[
    "receipt_id",
    "completed_at",
    "study_id",
    "protocol_validity",
    "pre_registration_sha256",
    "snapshot_sha256",
    "instrument_sha256",
    "rubric_sha256",
    "coded_evidence_sha256",
    "coded_records_sha256",
    "deviations_sha256",
    "control_transcript_sha256",
    "decision_packet_sha256",
    "session_classification_sha256",
    "coder_sha256",
    "custody_records_sha256",
    "custody_attestation_sha256s",
    "receipt_sha256",
];
const DECISION_PACKET_KEYS: &[&str] = &[
    "packet_id",
    "frozen_date",
    "pilot_pre_registration_sha256",
    "tested_snapshot_sha256",
    "coded_evidence",
    "exclusions",
    "coder_disagreements",
    "deviations",
    "revised_instrument",
    "control_transcript",
    "freeze_binding",
    "packet_sha256",
];
const RATIFICATION_KEYS: &[&str] = &[
    "ruling_id",
    "pilot_attempt_id",
    "ratified_date",
    "candidate_commit",
    "author_statement",
    "question_answered",
    "rationale",
    "pilot_packet_sha256",
    "sensitivity_brief_sha256",
    "rule_sha256",
    "decision_ref",
    "no_holdout_evidence_attestation",
    "ratification_sha256",
];
const HOLDOUT_KEYS: &[&str] = &["active_attempt_id", "attempts"];
const HOLDOUT_ATTEMPT_KEYS: &[&str] = &[
    "attempt_id",
    "previous_attempt_sha256",
    "attempt_status",
    "attempt_result",
    "void_reason_code",
    "voided_at",
    "pre_registration",
    "frozen_rule",
    "frozen_ratification",
    "session_records",
    "deviations",
    "custody_attestations",
    "result_receipt",
    "commitment_reveal",
    "gate_admission_receipt",
    "attempt_sha256",
];
const HOLDOUT_PRE_REGISTRATION_KEYS: &[&str] = &[
    "study_id",
    "registered_date",
    "predecessor_attempt_sha256",
    "prior_history_head_sha256",
    "fixed_protocol_sha256",
    "rule_sha256",
    "ratification_sha256",
    "evidence_gate_sha256",
    "structural_checker_sha256",
    "revised_instrument",
    "rubric",
    "release_candidate",
    "sample_rule",
    "recruitment_rule",
    "disclosure_set",
    "study_protocol",
    "commitment",
    "freeze_binding",
    "pre_registration_sha256",
];
const RELEASE_CANDIDATE_KEYS: &[&str] = &["candidate_id", "manifest_sha256", "artifacts"];
const COMMITMENT_KEYS: &[&str] = &[
    "commitment_id",
    "nonce_commitment_sha256",
    "committed_preimage_sha256",
    "custody_attestation_sha256",
];
const COMMITMENT_REVEAL_KEYS: &[&str] = &[
    "commitment_id",
    "revealed_at",
    "nonce_hex",
    "preimage",
    "custody_attestation_id",
    "reveal_sha256",
];
const DEVIATION_KEYS: &[&str] = &["deviation_id", "code", "impact", "custody_attestation_id"];
const CUSTODY_KEYS: &[&str] = &[
    "attestation_id",
    "study_id",
    "scope",
    "record_commitment_sha256",
    "ref",
    "sha256",
    "freshness_attested",
    "record_sha256",
];
const RESULT_RECEIPT_KEYS: &[&str] = &[
    "receipt_id",
    "completed_at",
    "study_id",
    "pre_registration_sha256",
    "rule_sha256",
    "candidate_manifest_sha256",
    "evidence_gate_sha256",
    "coded_records_sha256",
    "structural_checker_sha256",
    "deviations_sha256",
    "custody_records_sha256",
    "protocol_validity",
    "verdict",
    "evaluation_trace_sha256",
    "session_classification_sha256",
    "custody_attestation_sha256s",
    "receipt_sha256",
];
const ACCEPTANCE_KEYS: &[&str] = &["gate_c_satisfied", "permitted_claim", "limits"];
const GATE_ADMISSION_RECEIPT_KEYS: &[&str] = &[
    "schema_version",
    "input_sha256",
    "evidence_gate_sha256",
    "decision",
    "receipt_sha256",
];

const EVALUATION_ORDER: &[&str] = &[
    "protocol-validity",
    "evaluability",
    "core-veto",
    "required-targets",
    "non-core-rules",
    "pass",
];
const REQUIRED_TARGETS: &[(&str, &str)] = &[
    ("RE-TGT-ORDINARY-LIFE", "ordinary constructive life"),
    ("RE-TGT-DEMOCRATIC-CHOICE", "democratic choice"),
    ("RE-TGT-PRIVATE-FREEDOM", "private freedom"),
    ("RE-TGT-SUCCESSFUL-PROVISION", "successful provision"),
    ("RE-TGT-REPAIR", "repair"),
    (
        "RE-TGT-PRISONER-STRESS-TEST",
        "the prisoner as a stress test rather than the central inhabitant",
    ),
];
const ALLOWED_PUBLIC_RECORD_KINDS: &[&str] = &[
    "opaque study identifiers",
    "coded target and misconception outcomes",
    "artifact and commitment digests",
    "coded deviations",
    "custody attestations without identity material",
];
const EXCLUDED_FROM_REPOSITORY: &[&str] = &[
    "participant, session, coder, reviewer, and custodian names, pseudonyms, and identity mappings",
    "raw responses and free text",
    "consent and withdrawal forms",
    "direct contact, demographic, and accessibility records",
];
const DISCLOSED_LIMITS: &[&str] = &[
    "The ordinary-life account rests on unimplemented families at the time of testing.",
    "Every tested snapshot must carry its exact version identity.",
    "The evidence is usability evidence about the tested audience, not population statistics.",
    "Sampling and method limits bound every permitted reader claim.",
    "No reader result enters the reasoning engine or establishes a domain assigned to another assurance route.",
];
const ETHICS_TERMS: &[&str] = &[
    "informed consent",
    "withdrawal",
    "data minimisation and protection",
    "accessible participation",
    "fair compensation",
    "non-retaliation",
    "trauma safeguards where coercive experience is discussed",
    "independent ethics and safety review where appropriate",
];
const FRESHNESS_TERMS: &[&str] = &[
    "Holdout participants have no prior exposure to drafts, previews, the pilot, or the reviews corpus.",
    "Pilot participants are excluded from the holdout.",
    "The in-repository reviewer corpus is never admissible reader-study evidence.",
];
const NON_SUBSTITUTION: &str = "Reader evidence warrants only comprehension, balance, and human effects for the tested audience within the disclosed sampling and method limits.";
const FRESHNESS_BOUNDARY: &str = "The checker validates the evidence contract and the attestation binding; it cannot establish the truth of an externally held freshness or identity attestation.";
const STRUCTURAL_CHECKER_ARTIFACT_ID: &str = "RE-ART-STRUCTURAL-CHECKER";
const STRUCTURAL_CHECKER_REF: &str = "new-book-plans/14-reader-evidence.py::def main(";
const EVIDENCE_GATE_ARTIFACT_ID: &str = "RE-ART-EVIDENCE-ADMISSION-GATE";
const EVIDENCE_GATE_REF: &str =
    "new-book-plans/reader-evidence-admission-gate.py::def evaluate_reader_evidence(";

const FORBIDDEN_SCORE_KEYS: &[&str] = &[
    "score",
    "aggregate_score",
    "overall_score",
    "weighted_score",
    "average_score",
    "total_score",
];
const FORBIDDEN_PRIVATE_KEYS: &[&str] = &[
    "participant_name",
    "participant_id",
    "participant_ids",
    "session_id",
    "session_ids",
    "coder_id",
    "coder_ids",
    "reviewer_id",
    "reviewer_ids",
    "custodian_id",
    "custodian_ids",
    "participant_names",
    "coder_name",
    "coder_names",
    "reviewer_name",
    "reviewer_names",
    "custodian_name",
    "custodian_names",
    "raw_response",
    "raw_responses",
    "free_text",
    "consent_form",
    "consent_forms",
    "withdrawal_form",
    "withdrawal_forms",
    "identity_mapping",
    "identity_mappings",
    "contact_details",
    "demographics",
    "accessibility_record",
    "accessibility_records",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Check,
    CheckExecute,
    Generate,
    GenerateExecute,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputSnapshot<'a> {
    pub(crate) source_json: Option<&'a [u8]>,
    pub(crate) generated_report: Option<&'a str>,
    pub(crate) protocol_decision: Option<&'a [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Validation {
    pub(crate) valid_pilot: bool,
    pub(crate) valid_holdout_pass: bool,
}

/// One consequential enum value projected from the typed reader contract.
///
/// Script 13 combines these entries with the corresponding projections from
/// the other reviewed sources. Keeping the field name separate from the value
/// preserves its exact `(source_file, field, value)` mapping contract without
/// asking that consumer to walk JSON.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ReaderEnumEntry {
    pub(crate) field: &'static str,
    pub(crate) value: String,
}

/// The small validated reader-evidence surface consumed by the society ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReaderLedgerProjection {
    pub(crate) enum_inventory: BTreeSet<ReaderEnumEntry>,
    pub(crate) route_id: String,
    pub(crate) route_status: String,
    pub(crate) claim_id: String,
    pub(crate) claim_posture: String,
    pub(crate) claim_disposition: String,
    pub(crate) result: String,
    pub(crate) valid_holdout_pass: bool,
}

/// The ledger-owned R6 row supplied to the reader alignment check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReaderRouteAlignment<'a> {
    pub(crate) id: &'a str,
    pub(crate) status: &'a str,
    pub(crate) route_status: &'a str,
}

/// The ledger-owned FS-CLM-37 row supplied to the reader alignment check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReaderClaimAlignment<'a> {
    pub(crate) id: &'a str,
    pub(crate) route_ref: &'a str,
    pub(crate) posture: &'a str,
    pub(crate) unestablished_disposition: Option<&'a str>,
}

fn deserialize_required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThresholdSpec {
    pub(crate) threshold_id: String,
    pub(crate) metric: String,
    pub(crate) operator: String,
    pub(crate) value_kind: String,
    pub(crate) value: String,
    pub(crate) unit: String,
    pub(crate) denominator: String,
    pub(crate) scope_refs: Vec<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) evaluator_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SeverityDefinition {
    pub(crate) severity_id: String,
    pub(crate) label: String,
    pub(crate) definition: String,
    pub(crate) classification_boundary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MisconceptionDefinition {
    pub(crate) misconception_id: String,
    pub(crate) definition: String,
    pub(crate) severity_id: String,
    pub(crate) core: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThresholdPolicies {
    pub(crate) missing: String,
    pub(crate) ambiguous: String,
    pub(crate) multiply_coded: String,
    pub(crate) withdrawn: String,
    pub(crate) excluded: String,
    pub(crate) unclassified: String,
    pub(crate) rounding: String,
    pub(crate) coder_adjudication: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TargetThreshold {
    pub(crate) target_id: String,
    pub(crate) threshold: ThresholdSpec,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SeverityThreshold {
    pub(crate) severity_id: String,
    pub(crate) threshold: ThresholdSpec,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThresholdRule {
    pub(crate) rule_id: String,
    pub(crate) severity_taxonomy: Vec<SeverityDefinition>,
    pub(crate) misconceptions: Vec<MisconceptionDefinition>,
    pub(crate) core_misconception_ids: Vec<String>,
    pub(crate) core_failure_mode: String,
    pub(crate) repetition_unit: String,
    pub(crate) denominator: String,
    pub(crate) core_failure_threshold: ThresholdSpec,
    pub(crate) required_target_thresholds: Vec<TargetThreshold>,
    pub(crate) non_core_thresholds: Vec<SeverityThreshold>,
    pub(crate) minimum_evaluable_evidence: ThresholdSpec,
    pub(crate) policies: ThresholdPolicies,
    pub(crate) evaluation_order: Vec<String>,
    pub(crate) aggregate_offset_prohibited: bool,
    pub(crate) rule_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TargetOutcome {
    pub(crate) target_id: String,
    pub(crate) status: String,
    pub(crate) adjudication: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MisconceptionOutcome {
    pub(crate) misconception_id: String,
    pub(crate) status: String,
    pub(crate) occurrences: String,
    pub(crate) opportunities: String,
    pub(crate) adjudication: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionRecord {
    pub(crate) study_id: String,
    pub(crate) record_commitment_sha256: String,
    pub(crate) admissibility: String,
    pub(crate) target_outcomes: Vec<TargetOutcome>,
    pub(crate) misconception_outcomes: Vec<MisconceptionOutcome>,
    pub(crate) deviation_ids: Vec<String>,
    pub(crate) custody_attestation_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum EvaluationCheck {
    Protocol {
        check: String,
        observed: String,
        comparison: bool,
    },
    Threshold {
        threshold_id: String,
        metric: String,
        observed: String,
        comparison: Option<bool>,
    },
    Issue {
        issue: String,
        count: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluationStage {
    pub(crate) stage: String,
    pub(crate) status: String,
    pub(crate) checks: Vec<EvaluationCheck>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluationTrace {
    pub(crate) order: Vec<String>,
    pub(crate) stages: Vec<EvaluationStage>,
    pub(crate) verdict: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Artifact {
    pub(crate) artifact_id: String,
    pub(crate) r#ref: String,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreezeBinding {
    pub(crate) binding_id: String,
    pub(crate) binding_type: String,
    pub(crate) bound_payload_sha256: String,
    pub(crate) attested_payload_sha256: String,
    pub(crate) r#ref: String,
    pub(crate) attestation_sha256: String,
    pub(crate) frozen_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReleaseCandidate {
    pub(crate) candidate_id: String,
    pub(crate) manifest_sha256: String,
    pub(crate) artifacts: Vec<Artifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Commitment {
    pub(crate) commitment_id: String,
    pub(crate) nonce_commitment_sha256: String,
    pub(crate) committed_preimage_sha256: String,
    pub(crate) custody_attestation_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RatificationRecord {
    pub(crate) ruling_id: String,
    pub(crate) pilot_attempt_id: String,
    pub(crate) ratified_date: String,
    pub(crate) candidate_commit: String,
    pub(crate) author_statement: String,
    pub(crate) question_answered: String,
    pub(crate) rationale: String,
    pub(crate) pilot_packet_sha256: String,
    pub(crate) sensitivity_brief_sha256: String,
    pub(crate) rule_sha256: String,
    pub(crate) decision_ref: String,
    pub(crate) no_holdout_evidence_attestation: bool,
    pub(crate) ratification_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HoldoutPreRegistration {
    pub(crate) study_id: String,
    pub(crate) registered_date: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) predecessor_attempt_sha256: Option<String>,
    pub(crate) prior_history_head_sha256: String,
    pub(crate) fixed_protocol_sha256: String,
    pub(crate) rule_sha256: String,
    pub(crate) ratification_sha256: String,
    pub(crate) evidence_gate_sha256: String,
    pub(crate) structural_checker_sha256: String,
    pub(crate) revised_instrument: Artifact,
    pub(crate) rubric: Artifact,
    pub(crate) release_candidate: ReleaseCandidate,
    pub(crate) sample_rule: Artifact,
    pub(crate) recruitment_rule: Artifact,
    pub(crate) disclosure_set: Artifact,
    pub(crate) study_protocol: Artifact,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) commitment: Option<Commitment>,
    pub(crate) freeze_binding: FreezeBinding,
    pub(crate) pre_registration_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeviationRecord {
    pub(crate) deviation_id: String,
    pub(crate) code: String,
    pub(crate) impact: String,
    pub(crate) custody_attestation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustodyRecord {
    pub(crate) attestation_id: String,
    pub(crate) study_id: String,
    pub(crate) scope: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) record_commitment_sha256: Option<String>,
    pub(crate) r#ref: String,
    pub(crate) sha256: String,
    pub(crate) freshness_attested: bool,
    pub(crate) record_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResultReceipt {
    pub(crate) receipt_id: String,
    pub(crate) completed_at: String,
    pub(crate) study_id: String,
    pub(crate) pre_registration_sha256: String,
    pub(crate) rule_sha256: String,
    pub(crate) candidate_manifest_sha256: String,
    pub(crate) evidence_gate_sha256: String,
    pub(crate) coded_records_sha256: String,
    pub(crate) structural_checker_sha256: String,
    pub(crate) deviations_sha256: String,
    pub(crate) custody_records_sha256: String,
    pub(crate) protocol_validity: String,
    pub(crate) verdict: String,
    pub(crate) evaluation_trace_sha256: String,
    pub(crate) session_classification_sha256: String,
    pub(crate) custody_attestation_sha256s: Vec<String>,
    pub(crate) receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommitmentReveal {
    pub(crate) commitment_id: String,
    pub(crate) revealed_at: String,
    pub(crate) nonce_hex: String,
    pub(crate) preimage: Artifact,
    pub(crate) custody_attestation_id: String,
    pub(crate) reveal_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GateInput {
    pub(crate) schema_version: u64,
    pub(crate) attempt_id: String,
    pub(crate) active_attempt: bool,
    pub(crate) attempt_status: String,
    pub(crate) threshold_rule: ThresholdRule,
    pub(crate) frozen_ratification: RatificationRecord,
    pub(crate) pre_registration: HoldoutPreRegistration,
    pub(crate) session_records: Vec<SessionRecord>,
    pub(crate) deviations: Vec<DeviationRecord>,
    pub(crate) custody_attestations: Vec<CustodyRecord>,
    pub(crate) result_receipt: ResultReceipt,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) commitment_reveal: Option<CommitmentReveal>,
    pub(crate) current_rule_sha256: String,
    pub(crate) current_ratification_sha256: String,
    pub(crate) evidence_gate_sha256: String,
    pub(crate) structural_checker_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GateReceipt {
    pub(crate) schema_version: u64,
    pub(crate) input_sha256: String,
    pub(crate) evidence_gate_sha256: String,
    pub(crate) decision: String,
    pub(crate) receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryTransition {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    previous_source_commit: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    previous_source_sha256: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    previous_history_head_sha256: Option<String>,
    history_head_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewerCustodyAttestation {
    attestation_id: String,
    scope: String,
    evidence_gate_sha256: String,
    r#ref: String,
    sha256: String,
    attested_date: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct RouteRecord {
    route_id: String,
    route_status: String,
    evidence_contract_status: String,
    structural_checker_binding: Artifact,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    reviewer_custody_attestation: Option<ReviewerCustodyAttestation>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    evidence_admission_gate_binding: Option<Artifact>,
    negative_control_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ClaimRecord {
    claim_id: String,
    posture: String,
    disposition: String,
    result_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PrivacyRecord {
    public_record_policy: String,
    allowed_public_record_kinds: Vec<String>,
    excluded_from_repository: Vec<String>,
    freshness_attestation_boundary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct RequiredTarget {
    target_id: String,
    description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ProtocolRecord {
    decision_sha256: String,
    method: String,
    evaluation_order: Vec<String>,
    aggregate_offset_prohibited: bool,
    required_targets: Vec<RequiredTarget>,
    disclosed_limits: Vec<String>,
    ethics_terms: Vec<String>,
    freshness_terms: Vec<String>,
    non_substitution: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Prerequisites {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    readers_map_ref: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    glossary_ref: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    accessible_navigation_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PilotPreRegistration {
    study_id: String,
    registered_date: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    predecessor_attempt_sha256: Option<String>,
    prior_history_head_sha256: String,
    fixed_protocol_sha256: String,
    protocol: Artifact,
    instrument: Artifact,
    rubric: Artifact,
    sample_rule: Artifact,
    disclosure_set: Artifact,
    ethics_terms: Artifact,
    provisional_rule: Artifact,
    freeze_binding: FreezeBinding,
    pre_registration_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DecisionPacket {
    packet_id: String,
    frozen_date: String,
    pilot_pre_registration_sha256: String,
    tested_snapshot_sha256: String,
    coded_evidence: Artifact,
    exclusions: Artifact,
    coder_disagreements: Artifact,
    deviations: Artifact,
    revised_instrument: Artifact,
    control_transcript: Artifact,
    freeze_binding: FreezeBinding,
    packet_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PilotReceipt {
    receipt_id: String,
    completed_at: String,
    study_id: String,
    protocol_validity: String,
    pre_registration_sha256: String,
    snapshot_sha256: String,
    instrument_sha256: String,
    rubric_sha256: String,
    coded_evidence_sha256: String,
    coded_records_sha256: String,
    deviations_sha256: String,
    control_transcript_sha256: String,
    decision_packet_sha256: String,
    session_classification_sha256: String,
    coder_sha256: String,
    custody_records_sha256: String,
    custody_attestation_sha256s: Vec<String>,
    receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PilotAttemptRecord {
    attempt_id: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    previous_attempt_sha256: Option<String>,
    attempt_status: String,
    control_status: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    void_reason_code: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    voided_at: Option<String>,
    prerequisites: Prerequisites,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pre_registration: Option<PilotPreRegistration>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    tested_snapshot: Option<Artifact>,
    session_records: Vec<SessionRecord>,
    deviations: Vec<DeviationRecord>,
    custody_attestations: Vec<CustodyRecord>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    receipt: Option<PilotReceipt>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    decision_packet: Option<DecisionPacket>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    sensitivity_brief: Option<Artifact>,
    attempt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PilotRecord {
    pilot_status: String,
    control_status: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    active_attempt_id: Option<String>,
    attempts: Vec<PilotAttemptRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewedThresholdPolicies {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    missing: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    ambiguous: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    multiply_coded: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    withdrawn: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    excluded: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    unclassified: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    rounding: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    coder_adjudication: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewedThresholdRule {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    rule_id: Option<String>,
    severity_taxonomy: Vec<SeverityDefinition>,
    misconceptions: Vec<MisconceptionDefinition>,
    core_misconception_ids: Vec<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    core_failure_mode: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    repetition_unit: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    denominator: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    core_failure_threshold: Option<ThresholdSpec>,
    required_target_thresholds: Vec<TargetThreshold>,
    non_core_thresholds: Vec<SeverityThreshold>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    minimum_evaluable_evidence: Option<ThresholdSpec>,
    policies: ReviewedThresholdPolicies,
    evaluation_order: Vec<String>,
    aggregate_offset_prohibited: bool,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    rule_sha256: Option<String>,
}

impl ReviewedThresholdRule {
    fn populated(&self, path: &str) -> ReaderResult<ThresholdRule> {
        let required = |value: &Option<String>, field: &str| {
            value
                .clone()
                .ok_or_else(|| ReaderError::new(format!("{path}.{field}: expected text")))
        };
        Ok(ThresholdRule {
            rule_id: required(&self.rule_id, "rule_id")?,
            severity_taxonomy: self.severity_taxonomy.clone(),
            misconceptions: self.misconceptions.clone(),
            core_misconception_ids: self.core_misconception_ids.clone(),
            core_failure_mode: required(&self.core_failure_mode, "core_failure_mode")?,
            repetition_unit: required(&self.repetition_unit, "repetition_unit")?,
            denominator: required(&self.denominator, "denominator")?,
            core_failure_threshold: self.core_failure_threshold.clone().ok_or_else(|| {
                ReaderError::new(format!("{path}.core_failure_threshold: expected object"))
            })?,
            required_target_thresholds: self.required_target_thresholds.clone(),
            non_core_thresholds: self.non_core_thresholds.clone(),
            minimum_evaluable_evidence: self.minimum_evaluable_evidence.clone().ok_or_else(
                || {
                    ReaderError::new(format!(
                        "{path}.minimum_evaluable_evidence: expected object"
                    ))
                },
            )?,
            policies: ThresholdPolicies {
                missing: required(&self.policies.missing, "policies.missing")?,
                ambiguous: required(&self.policies.ambiguous, "policies.ambiguous")?,
                multiply_coded: required(&self.policies.multiply_coded, "policies.multiply_coded")?,
                withdrawn: required(&self.policies.withdrawn, "policies.withdrawn")?,
                excluded: required(&self.policies.excluded, "policies.excluded")?,
                unclassified: required(&self.policies.unclassified, "policies.unclassified")?,
                rounding: required(&self.policies.rounding, "policies.rounding")?,
                coder_adjudication: required(
                    &self.policies.coder_adjudication,
                    "policies.coder_adjudication",
                )?,
            },
            evaluation_order: self.evaluation_order.clone(),
            aggregate_offset_prohibited: self.aggregate_offset_prohibited,
            rule_sha256: required(&self.rule_sha256, "rule_sha256")?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct HoldoutAttempt {
    attempt_id: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    previous_attempt_sha256: Option<String>,
    attempt_status: String,
    attempt_result: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    void_reason_code: Option<String>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    voided_at: Option<String>,
    pre_registration: HoldoutPreRegistration,
    frozen_rule: ThresholdRule,
    frozen_ratification: RatificationRecord,
    session_records: Vec<SessionRecord>,
    deviations: Vec<DeviationRecord>,
    custody_attestations: Vec<CustodyRecord>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    result_receipt: Option<ResultReceipt>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    commitment_reveal: Option<CommitmentReveal>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    gate_admission_receipt: Option<GateReceipt>,
    attempt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct HoldoutRecord {
    #[serde(deserialize_with = "deserialize_required_nullable")]
    active_attempt_id: Option<String>,
    attempts: Vec<HoldoutAttempt>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceRecord {
    gate_c_satisfied: bool,
    permitted_claim: String,
    limits: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReaderEvidenceSource {
    spdx: String,
    schema_version: u64,
    contract_id: String,
    protocol_decision_ref: String,
    threshold_status: String,
    holdout_status: String,
    result: String,
    history_transition: HistoryTransition,
    route: RouteRecord,
    claim: ClaimRecord,
    privacy: PrivacyRecord,
    protocol: ProtocolRecord,
    pilot: PilotRecord,
    threshold_rule: ReviewedThresholdRule,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    ratification: Option<RatificationRecord>,
    holdout: HoldoutRecord,
    acceptance: AcceptanceRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReaderError(String);

impl ReaderError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ReaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

type ReaderResult<T> = Result<T, ReaderError>;

fn regex(value: &'static str, slot: &'static OnceLock<Regex>) -> &'static Regex {
    slot.get_or_init(|| Regex::new(value).expect("static regex is valid"))
}

fn sha_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^[0-9a-f]{64}$", &SLOT)
}

fn opaque_id_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^RE-[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+$", &SLOT)
}

fn commit_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^(?:[0-9a-f]{40}|[0-9a-f]{64})$", &SLOT)
}

fn decimal_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$", &SLOT)
}

fn integer_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^(?:0|[1-9][0-9]*)$", &SLOT)
}

fn placeholder_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"(?i)^(?:tbd|todo|unknown|n/?a|pending)$", &SLOT)
}

fn nonce_regex() -> &'static Regex {
    static SLOT: OnceLock<Regex> = OnceLock::new();
    regex(r"^(?:[0-9a-f]{2}){32,}$", &SLOT)
}

fn keys<'a>(values: &'a [&'a str]) -> BTreeSet<&'a str> {
    values.iter().copied().collect()
}

fn exact_keys(value: &Map<String, Value>, expected: &[&str], path: &str) -> ReaderResult<()> {
    let expected = keys(expected);
    let actual: BTreeSet<_> = value.keys().map(String::as_str).collect();
    let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
    let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }
    let mut details = Vec::new();
    if !missing.is_empty() {
        details.push(format!("missing {}", missing.join(", ")));
    }
    if !extra.is_empty() {
        details.push(format!("unexpected {}", extra.join(", ")));
    }
    Err(ReaderError::new(format!("{path}: {}", details.join("; "))))
}

fn object<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| ReaderError::new(format!("{path}: expected an object with string keys")))
}

fn array<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| ReaderError::new(format!("{path}: expected an array")))
}

fn text<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a str> {
    let value = value
        .as_str()
        .ok_or_else(|| ReaderError::new(format!("{path}: expected substantive text")))?;
    if value.trim().is_empty() || placeholder_regex().is_match(value.trim()) {
        return Err(ReaderError::new(format!(
            "{path}: expected substantive text"
        )));
    }
    Ok(value)
}

fn text_list<'a>(value: &'a Value, path: &str, nonempty: bool) -> ReaderResult<Vec<&'a str>> {
    let values = array(value, path)?;
    if nonempty && values.is_empty() {
        return Err(ReaderError::new(format!("{path}: must not be empty")));
    }
    let mut result = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        result.push(text(value, &format!("{path}[{index}]"))?);
    }
    if result.iter().copied().collect::<HashSet<_>>().len() != result.len() {
        return Err(ReaderError::new(format!("{path}: duplicate values")));
    }
    Ok(result)
}

fn enumeration<'a>(value: &'a Value, allowed: &[&str], path: &str) -> ReaderResult<&'a str> {
    let Some(value) = value.as_str() else {
        return Err(ReaderError::new(format!(
            "{path}: expected one of {:?}",
            keys(allowed)
        )));
    };
    if !allowed.contains(&value) {
        return Err(ReaderError::new(format!(
            "{path}: expected one of {:?}",
            keys(allowed)
        )));
    }
    Ok(value)
}

fn boolean(value: &Value, path: &str) -> ReaderResult<bool> {
    value
        .as_bool()
        .ok_or_else(|| ReaderError::new(format!("{path}: expected a boolean")))
}

fn digest<'a>(value: &'a Value, path: &str, expected: Option<&str>) -> ReaderResult<&'a str> {
    let Some(value) = value.as_str().filter(|value| sha_regex().is_match(value)) else {
        return Err(ReaderError::new(format!(
            "{path}: expected 64 lowercase hexadecimal characters"
        )));
    };
    if let Some(expected) = expected
        && value != expected
    {
        return Err(ReaderError::new(format!(
            "{path}: stale; declared {value}, actual {expected}"
        )));
    }
    Ok(value)
}

fn valid_date_components(year: i32, month: u32, day: u32) -> bool {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    year >= 1 && day >= 1 && day <= days
}

fn date<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a str> {
    let value = text(value, path)?;
    let bytes = value.as_bytes();
    let canonical = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !canonical {
        return Err(ReaderError::new(format!("{path}: expected YYYY-MM-DD")));
    }
    let year = value[..4].parse().expect("digits");
    let month = value[5..7].parse().expect("digits");
    let day = value[8..10].parse().expect("digits");
    if !valid_date_components(year, month, day) {
        return Err(ReaderError::new(format!("{path}: invalid calendar date")));
    }
    Ok(value)
}

fn utc_timestamp<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a str> {
    let value = text(value, path)?;
    let bytes = value.as_bytes();
    let canonical = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        });
    if !canonical {
        return Err(ReaderError::new(format!(
            "{path}: expected canonical UTC YYYY-MM-DDTHH:MM:SSZ"
        )));
    }
    let date_value = Value::String(value[..10].to_owned());
    if date(&date_value, path).is_err()
        || value[11..13].parse::<u32>().expect("digits") > 23
        || value[14..16].parse::<u32>().expect("digits") > 59
        || value[17..19].parse::<u32>().expect("digits") > 59
    {
        return Err(ReaderError::new(format!("{path}: invalid UTC timestamp")));
    }
    Ok(value)
}

fn opaque_id<'a>(value: &'a Value, path: &str) -> ReaderResult<&'a str> {
    let value = text(value, path)?;
    if !opaque_id_regex().is_match(value) {
        return Err(ReaderError::new(format!(
            "{path}: expected an opaque RE-* identifier"
        )));
    }
    Ok(value)
}

fn integer_text<'a>(value: &'a Value, path: &str, positive: bool) -> ReaderResult<&'a str> {
    let value = text(value, path)?;
    if !integer_regex().is_match(value) {
        return Err(ReaderError::new(format!(
            "{path}: expected a canonical non-negative integer string"
        )));
    }
    if positive && value == "0" {
        return Err(ReaderError::new(format!(
            "{path}: expected a positive integer string"
        )));
    }
    Ok(value)
}

fn canonical_sha(value: &Value, omit: Option<&str>) -> ReaderResult<String> {
    let owned;
    let value = if let Some(omit) = omit {
        let mut object = object(value, "canonical digest object")?.clone();
        object.remove(omit);
        owned = Value::Object(object);
        &owned
    } else {
        value
    };
    Ok(sha256(canonical_json(value)))
}

fn canonical_sha_omitting(value: &Value, omitted: &[&str]) -> ReaderResult<String> {
    let mut object = object(value, "canonical digest object")?.clone();
    for key in omitted {
        object.remove(*key);
    }
    canonical_sha(&Value::Object(object), None)
}

pub(crate) fn history_head_sha256<'a>(
    pilot_attempt_sha256s: impl IntoIterator<Item = &'a str>,
    holdout_attempt_sha256s: impl IntoIterator<Item = &'a str>,
) -> String {
    let value = serde_json::json!({
        "pilot_attempt_sha256s": pilot_attempt_sha256s.into_iter().collect::<Vec<_>>(),
        "holdout_attempt_sha256s": holdout_attempt_sha256s.into_iter().collect::<Vec<_>>(),
    });
    sha256(canonical_json(&value))
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVisitor)
    }
}

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut entries: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, entries.next_value::<UniqueValue>()?.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

pub(crate) fn parse_source(bytes: &[u8]) -> ReaderResult<Value> {
    parse_unique_json(bytes, DEFAULT_SOURCE)
}

fn parse_unique_json(bytes: &[u8], label: &str) -> ReaderResult<Value> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueValue::deserialize(&mut deserializer)
        .map_err(|error| ReaderError::new(format!("cannot read {label}: {error}")))?
        .0;
    deserializer
        .end()
        .map_err(|error| ReaderError::new(format!("cannot read {label}: {error}")))?;
    object(&value, label)?;
    Ok(value)
}

fn walk_keys(value: &Value, path: &str) -> ReaderResult<()> {
    match value {
        Value::Object(values) => {
            for (key, child) in values {
                let normal = key.to_lowercase().replace('-', "_");
                if FORBIDDEN_SCORE_KEYS.contains(&normal.as_str()) {
                    return Err(ReaderError::new(format!(
                        "{path}.{key}: aggregate or person scoring fields are prohibited"
                    )));
                }
                if FORBIDDEN_PRIVATE_KEYS.contains(&normal.as_str()) {
                    return Err(ReaderError::new(format!(
                        "{path}.{key}: identifying or raw participant material belongs outside the repository"
                    )));
                }
                walk_keys(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                walk_keys(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

struct ValidationEnv<'a> {
    context: &'a Context,
    protocol_decision: &'a [u8],
    verify_live: bool,
}

fn safe_relative_path<'a>(value: &'a str, path: &str) -> ReaderResult<&'a Path> {
    let candidate = Path::new(value);
    if candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ReaderError::new(format!(
            "path escapes repository: {value} ({path})"
        )));
    }
    Ok(candidate)
}

fn validate_repo_reference(
    env: &ValidationEnv<'_>,
    value: &Value,
    path: &str,
) -> ReaderResult<String> {
    let reference = text(value, path)?;
    let Some((relative, anchor)) = reference.split_once("::") else {
        return Err(ReaderError::new(format!(
            "{path}: repository reference needs path::exact-anchor"
        )));
    };
    if relative.is_empty() || anchor.is_empty() {
        return Err(ReaderError::new(format!(
            "{path}: incomplete repository reference"
        )));
    }
    let relative_path = safe_relative_path(relative, path)?;
    let target = env.context.path(relative_path);
    let root = std::fs::canonicalize(env.context.root()).map_err(|error| {
        ReaderError::new(format!("{path}: cannot resolve repository root: {error}"))
    })?;
    let canonical = std::fs::canonicalize(&target).map_err(|error| {
        ReaderError::new(format!(
            "{path}: cannot read referenced file {relative}: {error}"
        ))
    })?;
    if !canonical.starts_with(root) {
        return Err(ReaderError::new(format!(
            "path escapes repository: {target:?}"
        )));
    }
    let contents = std::fs::read_to_string(&canonical).map_err(|error| {
        ReaderError::new(format!(
            "{path}: cannot read referenced file {relative}: {error}"
        ))
    })?;
    let count = contents.matches(anchor).count();
    if count != 1 {
        return Err(ReaderError::new(format!(
            "{path}: anchor must occur exactly once in {relative}; found {count}"
        )));
    }
    Ok(reference.to_owned())
}

fn validate_external_or_repo_reference(
    env: &ValidationEnv<'_>,
    value: &Value,
    path: &str,
) -> ReaderResult<String> {
    let reference = text(value, path)?;
    if let Some(suffix) = reference.strip_prefix("custody:") {
        let valid = !suffix.is_empty()
            && suffix.chars().all(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
            })
            && suffix.chars().next().is_some_and(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit()
            });
        if !valid {
            return Err(ReaderError::new(format!(
                "{path}: malformed opaque custody reference"
            )));
        }
        return Ok(reference.to_owned());
    }
    validate_repo_reference(env, value, path)
}

fn validate_artifact<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    verify_live: bool,
) -> ReaderResult<&'a Map<String, Value>> {
    let artifact = object(value, path)?;
    exact_keys(artifact, ARTIFACT_KEYS, path)?;
    opaque_id(&artifact["artifact_id"], &format!("{path}.artifact_id"))?;
    let reference =
        validate_external_or_repo_reference(env, &artifact["ref"], &format!("{path}.ref"))?;
    let declared = digest(&artifact["sha256"], &format!("{path}.sha256"), None)?;
    if verify_live && env.verify_live && !reference.starts_with("custody:") {
        let relative = reference.split_once("::").expect("repository ref").0;
        let bytes = std::fs::read(env.context.path(relative)).map_err(|error| {
            ReaderError::new(format!("{path}.sha256: cannot read {relative}: {error}"))
        })?;
        let actual = sha256(bytes);
        if declared != actual {
            return Err(ReaderError::new(format!(
                "{path}.sha256: stale; declared {declared}, actual {actual}"
            )));
        }
    }
    Ok(artifact)
}

fn validate_preregistration_history_binding(
    registration: &Map<String, Value>,
    path: &str,
    expected_predecessor_attempt_sha256: Option<&str>,
    expected_prior_history_head_sha256: Option<&str>,
    enforce_expected: bool,
) -> ReaderResult<()> {
    let predecessor = &registration["predecessor_attempt_sha256"];
    if !predecessor.is_null() {
        digest(
            predecessor,
            &format!("{path}.predecessor_attempt_sha256"),
            None,
        )?;
    }
    let prior_head = digest(
        &registration["prior_history_head_sha256"],
        &format!("{path}.prior_history_head_sha256"),
        None,
    )?;
    if !enforce_expected {
        return Ok(());
    }
    match expected_predecessor_attempt_sha256 {
        None if !predecessor.is_null() => {
            return Err(ReaderError::new(format!(
                "{path}.predecessor_attempt_sha256: first attempt must be null"
            )));
        }
        Some(expected) => {
            digest(
                predecessor,
                &format!("{path}.predecessor_attempt_sha256"),
                Some(expected),
            )?;
        }
        None => {}
    }
    digest(
        &Value::String(prior_head.to_owned()),
        &format!("{path}.prior_history_head_sha256"),
        expected_prior_history_head_sha256,
    )?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HistoricalPayloadKind {
    PilotPreRegistration,
    PilotDecisionPacket,
    HoldoutPreRegistration,
}

enum HistoricalReaderEvidence {
    V1(ReaderEvidenceSource),
}

impl HistoricalReaderEvidence {
    fn source(&self) -> &ReaderEvidenceSource {
        match self {
            Self::V1(source) => source,
        }
    }
}

fn decode_historical_reader_evidence(
    bytes: &[u8],
    path: &str,
) -> ReaderResult<HistoricalReaderEvidence> {
    // Retain the duplicate-key preflight before serde's strict typed decode.
    parse_source(bytes).map_err(|error| {
        ReaderError::new(format!(
            "{path}: git freeze source is invalid JSON: {error}"
        ))
    })?;
    let source: ReaderEvidenceSource = serde_json::from_slice(bytes).map_err(|error| {
        ReaderError::new(format!(
            "{path}: git freeze source violates the typed reader-evidence contract: {error}"
        ))
    })?;
    match source.schema_version {
        1 => Ok(HistoricalReaderEvidence::V1(source)),
        version => Err(ReaderError::new(format!(
            "{path}: unsupported historical reader-evidence schema version {version}"
        ))),
    }
}

fn frozen_payload_sha<T: serde::Serialize>(
    value: &T,
    path: &str,
    digest_field: &str,
) -> ReaderResult<String> {
    canonical_sha_omitting(
        &typed_value(value, path)?,
        &["freeze_binding", digest_field],
    )
}

fn any_historical_payload_matches<'a, T: serde::Serialize + 'a>(
    candidates: impl IntoIterator<Item = &'a T>,
    expected_payload_sha256: &str,
    digest_field: &str,
) -> ReaderResult<bool> {
    for candidate in candidates {
        if frozen_payload_sha(candidate, "historical frozen payload", digest_field)?
            == expected_payload_sha256
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn contains_typed_historical_payload(
    historical: &HistoricalReaderEvidence,
    kind: HistoricalPayloadKind,
    expected_payload_sha256: &str,
    digest_field: &str,
) -> ReaderResult<bool> {
    let source = historical.source();
    match kind {
        HistoricalPayloadKind::PilotPreRegistration => any_historical_payload_matches(
            source
                .pilot
                .attempts
                .iter()
                .filter_map(|attempt| attempt.pre_registration.as_ref()),
            expected_payload_sha256,
            digest_field,
        ),
        HistoricalPayloadKind::PilotDecisionPacket => any_historical_payload_matches(
            source
                .pilot
                .attempts
                .iter()
                .filter_map(|attempt| attempt.decision_packet.as_ref()),
            expected_payload_sha256,
            digest_field,
        ),
        HistoricalPayloadKind::HoldoutPreRegistration => any_historical_payload_matches(
            source
                .holdout
                .attempts
                .iter()
                .map(|attempt| &attempt.pre_registration),
            expected_payload_sha256,
            digest_field,
        ),
    }
}

fn git_output(context: &Context, args: &[&str], path: &str) -> ReaderResult<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(context.root())
        .args(args)
        .output()
        .map_err(|error| ReaderError::new(format!("{path}: cannot inspect git freeze: {error}")))
}

fn validate_git_freeze(
    env: &ValidationEnv<'_>,
    commit: &str,
    path: &str,
    payload_kind: HistoricalPayloadKind,
    expected_payload_sha256: &str,
    digest_field: &str,
) -> ReaderResult<()> {
    let head = git_output(env.context, &["rev-parse", "HEAD"], path)?;
    let ancestor = git_output(
        env.context,
        &["merge-base", "--is-ancestor", commit, "HEAD"],
        path,
    )?;
    let object_spec = format!("{commit}:{DEFAULT_SOURCE}");
    let shown = git_output(env.context, &["show", &object_spec], path)?;
    if !head.status.success() {
        return Err(ReaderError::new(format!(
            "{path}: cannot resolve current git HEAD"
        )));
    }
    let current_head = String::from_utf8_lossy(&head.stdout).trim().to_owned();
    if commit == current_head {
        return Err(ReaderError::new(format!(
            "{path}: git freeze must cite a commit strictly before the current checkout"
        )));
    }
    if !ancestor.status.success() {
        return Err(ReaderError::new(format!(
            "{path}: git freeze must cite a prior ancestor commit"
        )));
    }
    if !shown.status.success() {
        let detail = String::from_utf8_lossy(&shown.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "{path}: git freeze source is unavailable: {detail}"
        )));
    }
    let committed_source = decode_historical_reader_evidence(&shown.stdout, path)?;
    if !contains_typed_historical_payload(
        &committed_source,
        payload_kind,
        expected_payload_sha256,
        digest_field,
    )? {
        return Err(ReaderError::new(format!(
            "{path}: prior git commit does not contain the exact frozen payload"
        )));
    }
    Ok(())
}

fn validate_freeze_binding<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    frozen_value: &Map<String, Value>,
    digest_field: &str,
    historical_payload_kind: HistoricalPayloadKind,
) -> ReaderResult<&'a Map<String, Value>> {
    let binding = object(value, path)?;
    exact_keys(binding, FREEZE_BINDING_KEYS, path)?;
    opaque_id(&binding["binding_id"], &format!("{path}.binding_id"))?;
    let binding_type = enumeration(
        &binding["binding_type"],
        &["git-commit", "external-custody"],
        &format!("{path}.binding_type"),
    )?;
    let expected_payload = canonical_sha_omitting(
        &Value::Object(frozen_value.clone()),
        &["freeze_binding", digest_field],
    )?;
    digest(
        &binding["bound_payload_sha256"],
        &format!("{path}.bound_payload_sha256"),
        Some(&expected_payload),
    )?;
    digest(
        &binding["attested_payload_sha256"],
        &format!("{path}.attested_payload_sha256"),
        Some(&expected_payload),
    )?;
    let reference = text(&binding["ref"], &format!("{path}.ref"))?;
    if binding_type == "git-commit" {
        let Some(commit) = reference.strip_prefix("git:") else {
            return Err(ReaderError::new(format!(
                "{path}.ref: git binding requires git:<full-commit>"
            )));
        };
        if !commit_regex().is_match(commit) {
            return Err(ReaderError::new(format!(
                "{path}.ref: git binding requires git:<full-commit>"
            )));
        }
        validate_git_freeze(
            env,
            commit,
            path,
            historical_payload_kind,
            &expected_payload,
            digest_field,
        )?;
    } else {
        if reference != "custody:READER-EVIDENCE-FREEZE" {
            return Err(ReaderError::new(format!(
                "{path}.ref: external freeze must use the fixed custody channel"
            )));
        }
        validate_external_or_repo_reference(env, &binding["ref"], &format!("{path}.ref"))?;
    }
    let frozen_at = utc_timestamp(&binding["frozen_at"], &format!("{path}.frozen_at"))?;
    if binding_type == "git-commit" {
        let envelope = serde_json::json!({
            "binding_id": binding["binding_id"],
            "binding_type": binding_type,
            "attested_payload_sha256": expected_payload,
            "bound_payload_sha256": expected_payload,
            "ref": reference,
            "frozen_at": frozen_at,
        });
        let expected = canonical_sha(&envelope, None)?;
        digest(
            &binding["attestation_sha256"],
            &format!("{path}.attestation_sha256"),
            Some(&expected),
        )?;
    } else {
        digest(
            &binding["attestation_sha256"],
            &format!("{path}.attestation_sha256"),
            None,
        )?;
    }
    Ok(binding)
}

fn validate_text_str<'a>(value: &'a str, path: &str) -> ReaderResult<&'a str> {
    if value.trim().is_empty() || placeholder_regex().is_match(value.trim()) {
        return Err(ReaderError::new(format!(
            "{path}: expected non-placeholder text"
        )));
    }
    Ok(value)
}

fn validate_id_str<'a>(value: &'a str, path: &str) -> ReaderResult<&'a str> {
    if !opaque_id_regex().is_match(value) {
        return Err(ReaderError::new(format!(
            "{path}: expected an opaque RE-* identifier"
        )));
    }
    Ok(value)
}

fn validate_digest_str<'a>(
    value: &'a str,
    path: &str,
    expected: Option<&str>,
) -> ReaderResult<&'a str> {
    if !sha_regex().is_match(value) {
        return Err(ReaderError::new(format!(
            "{path}: expected a lowercase SHA-256 digest"
        )));
    }
    if expected.is_some_and(|expected| value != expected) {
        return Err(ReaderError::new(format!(
            "{path}: stale; declared {value}, actual {}",
            expected.unwrap()
        )));
    }
    Ok(value)
}

fn validate_date_str<'a>(value: &'a str, path: &str) -> ReaderResult<&'a str> {
    validate_text_str(value, path)?;
    let bytes = value.as_bytes();
    let canonical = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !canonical {
        return Err(ReaderError::new(format!("{path}: expected YYYY-MM-DD")));
    }
    let year = value[..4].parse().expect("validated digits");
    let month = value[5..7].parse().expect("validated digits");
    let day = value[8..10].parse().expect("validated digits");
    if !valid_date_components(year, month, day) {
        return Err(ReaderError::new(format!("{path}: invalid calendar date")));
    }
    Ok(value)
}

fn validate_timestamp_str<'a>(value: &'a str, path: &str) -> ReaderResult<&'a str> {
    validate_text_str(value, path)?;
    let bytes = value.as_bytes();
    let canonical = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        });
    if !canonical {
        return Err(ReaderError::new(format!(
            "{path}: expected canonical UTC YYYY-MM-DDTHH:MM:SSZ"
        )));
    }
    if validate_date_str(&value[..10], path).is_err()
        || value[11..13].parse::<u32>().expect("validated digits") > 23
        || value[14..16].parse::<u32>().expect("validated digits") > 59
        || value[17..19].parse::<u32>().expect("validated digits") > 59
    {
        return Err(ReaderError::new(format!("{path}: invalid UTC timestamp")));
    }
    Ok(value)
}

fn typed_value<T: serde::Serialize>(value: &T, path: &str) -> ReaderResult<Value> {
    serde_json::to_value(value)
        .map_err(|error| ReaderError::new(format!("{path}: cannot encode typed value: {error}")))
}

fn typed_canonical_sha<T: serde::Serialize>(
    value: &T,
    path: &str,
    omit: Option<&str>,
) -> ReaderResult<String> {
    canonical_sha(&typed_value(value, path)?, omit)
}

fn validate_repo_reference_str(
    env: &ValidationEnv<'_>,
    reference: &str,
    path: &str,
) -> ReaderResult<String> {
    validate_text_str(reference, path)?;
    let Some((relative, anchor)) = reference.split_once("::") else {
        return Err(ReaderError::new(format!(
            "{path}: repository reference needs path::exact-anchor"
        )));
    };
    if relative.is_empty() || anchor.is_empty() {
        return Err(ReaderError::new(format!(
            "{path}: incomplete repository reference"
        )));
    }
    let relative_path = safe_relative_path(relative, path)?;
    let target = env.context.path(relative_path);
    let root = std::fs::canonicalize(env.context.root()).map_err(|error| {
        ReaderError::new(format!("{path}: cannot resolve repository root: {error}"))
    })?;
    let canonical = std::fs::canonicalize(&target).map_err(|error| {
        ReaderError::new(format!(
            "{path}: cannot read referenced file {relative}: {error}"
        ))
    })?;
    if !canonical.starts_with(root) {
        return Err(ReaderError::new(format!(
            "path escapes repository: {target:?}"
        )));
    }
    let contents = std::fs::read_to_string(&canonical).map_err(|error| {
        ReaderError::new(format!(
            "{path}: cannot read referenced file {relative}: {error}"
        ))
    })?;
    let count = contents.matches(anchor).count();
    if count != 1 {
        return Err(ReaderError::new(format!(
            "{path}: anchor must occur exactly once in {relative}; found {count}"
        )));
    }
    Ok(reference.to_owned())
}

fn validate_external_or_repo_reference_str(
    env: &ValidationEnv<'_>,
    reference: &str,
    path: &str,
) -> ReaderResult<String> {
    validate_text_str(reference, path)?;
    if let Some(suffix) = reference.strip_prefix("custody:") {
        let valid = !suffix.is_empty()
            && suffix.chars().all(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
            })
            && suffix.chars().next().is_some_and(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit()
            });
        if !valid {
            return Err(ReaderError::new(format!(
                "{path}: malformed opaque custody reference"
            )));
        }
        return Ok(reference.to_owned());
    }
    validate_repo_reference_str(env, reference, path)
}

fn validate_artifact_typed(
    env: &ValidationEnv<'_>,
    artifact: &Artifact,
    path: &str,
    verify_live: bool,
) -> ReaderResult<()> {
    validate_id_str(&artifact.artifact_id, &format!("{path}.artifact_id"))?;
    let reference =
        validate_external_or_repo_reference_str(env, &artifact.r#ref, &format!("{path}.ref"))?;
    validate_digest_str(&artifact.sha256, &format!("{path}.sha256"), None)?;
    if verify_live && env.verify_live && !reference.starts_with("custody:") {
        let relative = reference.split_once("::").expect("repository ref").0;
        let actual = sha256(std::fs::read(env.context.path(relative)).map_err(|error| {
            ReaderError::new(format!("{path}.sha256: cannot read {relative}: {error}"))
        })?);
        validate_digest_str(&artifact.sha256, &format!("{path}.sha256"), Some(&actual))?;
    }
    Ok(())
}

fn validate_history_binding_typed(
    predecessor: Option<&str>,
    prior_head: &str,
    path: &str,
    expected_predecessor: Option<&str>,
    expected_prior_head: Option<&str>,
    enforce: bool,
) -> ReaderResult<()> {
    if let Some(predecessor) = predecessor {
        validate_digest_str(
            predecessor,
            &format!("{path}.predecessor_attempt_sha256"),
            None,
        )?;
    }
    validate_digest_str(
        prior_head,
        &format!("{path}.prior_history_head_sha256"),
        None,
    )?;
    if enforce {
        match (predecessor, expected_predecessor) {
            (Some(value), Some(expected)) => {
                validate_digest_str(
                    value,
                    &format!("{path}.predecessor_attempt_sha256"),
                    Some(expected),
                )?;
            }
            (None, Some(_)) => {
                return Err(ReaderError::new(format!(
                    "{path}.predecessor_attempt_sha256: expected a digest"
                )));
            }
            (Some(_), None) => {
                return Err(ReaderError::new(format!(
                    "{path}.predecessor_attempt_sha256: first attempt must be null"
                )));
            }
            (None, None) => {}
        }
        if let Some(expected) = expected_prior_head {
            validate_digest_str(
                prior_head,
                &format!("{path}.prior_history_head_sha256"),
                Some(expected),
            )?;
        }
    }
    Ok(())
}

fn validate_freeze_binding_typed<T: serde::Serialize>(
    env: &ValidationEnv<'_>,
    binding: &FreezeBinding,
    frozen_value: &T,
    path: &str,
    digest_field: &str,
    historical_payload_kind: HistoricalPayloadKind,
) -> ReaderResult<()> {
    validate_id_str(&binding.binding_id, &format!("{path}.binding_id"))?;
    if !matches!(
        binding.binding_type.as_str(),
        "git-commit" | "external-custody"
    ) {
        return Err(ReaderError::new(format!(
            "{path}.binding_type: expected one of external-custody, git-commit"
        )));
    }
    let expected_payload = frozen_payload_sha(frozen_value, path, digest_field)?;
    validate_digest_str(
        &binding.bound_payload_sha256,
        &format!("{path}.bound_payload_sha256"),
        Some(&expected_payload),
    )?;
    validate_digest_str(
        &binding.attested_payload_sha256,
        &format!("{path}.attested_payload_sha256"),
        Some(&expected_payload),
    )?;
    if binding.binding_type == "git-commit" {
        let commit = binding.r#ref.strip_prefix("git:").ok_or_else(|| {
            ReaderError::new(format!(
                "{path}.ref: git binding requires git:<full-commit>"
            ))
        })?;
        if !commit_regex().is_match(commit) {
            return Err(ReaderError::new(format!(
                "{path}.ref: git binding requires git:<full-commit>"
            )));
        }
        validate_git_freeze(
            env,
            commit,
            path,
            historical_payload_kind,
            &expected_payload,
            digest_field,
        )?;
    } else {
        if binding.r#ref != "custody:READER-EVIDENCE-FREEZE" {
            return Err(ReaderError::new(format!(
                "{path}.ref: external freeze must use the fixed custody channel"
            )));
        }
        validate_external_or_repo_reference_str(env, &binding.r#ref, &format!("{path}.ref"))?;
    }
    validate_timestamp_str(&binding.frozen_at, &format!("{path}.frozen_at"))?;
    if binding.binding_type == "git-commit" {
        let envelope = serde_json::json!({
            "binding_id": binding.binding_id,
            "binding_type": binding.binding_type,
            "attested_payload_sha256": expected_payload,
            "bound_payload_sha256": expected_payload,
            "ref": binding.r#ref,
            "frozen_at": binding.frozen_at,
        });
        validate_digest_str(
            &binding.attestation_sha256,
            &format!("{path}.attestation_sha256"),
            Some(&canonical_sha(&envelope, None)?),
        )?;
    } else {
        validate_digest_str(
            &binding.attestation_sha256,
            &format!("{path}.attestation_sha256"),
            None,
        )?;
    }
    Ok(())
}

fn validate_protocol_typed(
    env: &ValidationEnv<'_>,
    source: &ReaderEvidenceSource,
) -> ReaderResult<()> {
    if source.spdx != "CC-BY-4.0" {
        return Err(ReaderError::new("spdx must be CC-BY-4.0"));
    }
    if source.schema_version != 1 {
        return Err(ReaderError::new("schema_version must be integer 1"));
    }
    if source.contract_id != "book-1-reader-evidence-v1" {
        return Err(ReaderError::new(
            "contract_id must be book-1-reader-evidence-v1",
        ));
    }
    let reference = &source.protocol_decision_ref;
    let Some((relative, anchor)) = reference.split_once("::") else {
        return Err(ReaderError::new(
            "protocol_decision_ref needs path::exact-anchor",
        ));
    };
    if relative != PROTOCOL_DECISION || anchor.is_empty() {
        return Err(ReaderError::new(
            "protocol_decision_ref must cite the controlling decision",
        ));
    }
    let decision_text = std::str::from_utf8(env.protocol_decision)
        .map_err(|_| ReaderError::new("candidate protocol decision is not valid UTF-8"))?;
    let count = decision_text.matches(anchor).count();
    if count != 1 {
        return Err(ReaderError::new(format!(
            "protocol_decision_ref anchor must occur exactly once in the candidate decision; found {count}"
        )));
    }
    validate_digest_str(
        &source.protocol.decision_sha256,
        "protocol.decision_sha256",
        Some(&sha256(env.protocol_decision)),
    )?;
    if source.protocol.method != "pre-registered-pilot-and-fresh-holdout" {
        return Err(ReaderError::new(
            "protocol.method drifted from the ratified method",
        ));
    }
    if source.protocol.evaluation_order != EVALUATION_ORDER {
        return Err(ReaderError::new(
            "protocol.evaluation_order must preserve the ratified order",
        ));
    }
    if !source.protocol.aggregate_offset_prohibited {
        return Err(ReaderError::new(
            "protocol must prohibit aggregate offset of a core finding",
        ));
    }
    let mut found = BTreeMap::new();
    for (index, target) in source.protocol.required_targets.iter().enumerate() {
        validate_text_str(
            &target.target_id,
            &format!("protocol.required_targets[{index}].target_id"),
        )?;
        validate_text_str(
            &target.description,
            &format!("protocol.required_targets[{index}].description"),
        )?;
        if found
            .insert(target.target_id.as_str(), target.description.as_str())
            .is_some()
        {
            return Err(ReaderError::new(format!(
                "protocol.required_targets[{index}].target_id: duplicate {}",
                target.target_id
            )));
        }
    }
    let expected: BTreeMap<_, _> = REQUIRED_TARGETS.iter().copied().collect();
    if found != expected {
        return Err(ReaderError::new(
            "protocol.required_targets drifted from the ratified minimum",
        ));
    }
    if source.protocol.disclosed_limits != DISCLOSED_LIMITS {
        return Err(ReaderError::new("protocol.disclosed_limits drifted"));
    }
    if source.protocol.ethics_terms != ETHICS_TERMS {
        return Err(ReaderError::new("protocol.ethics_terms drifted"));
    }
    if source.protocol.freshness_terms != FRESHNESS_TERMS {
        return Err(ReaderError::new("protocol.freshness_terms drifted"));
    }
    if source.protocol.non_substitution != NON_SUBSTITUTION {
        return Err(ReaderError::new(
            "protocol.non_substitution drifted from the ratified boundary",
        ));
    }
    Ok(())
}

fn validate_privacy_typed(source: &ReaderEvidenceSource) -> ReaderResult<()> {
    if source.privacy.public_record_policy != "privacy-minimal-coded-records-only" {
        return Err(ReaderError::new("privacy.public_record_policy drifted"));
    }
    if source.privacy.allowed_public_record_kinds != ALLOWED_PUBLIC_RECORD_KINDS {
        return Err(ReaderError::new(
            "privacy.allowed_public_record_kinds drifted",
        ));
    }
    if source.privacy.excluded_from_repository != EXCLUDED_FROM_REPOSITORY {
        return Err(ReaderError::new("privacy.excluded_from_repository drifted"));
    }
    if source.privacy.freshness_attestation_boundary != FRESHNESS_BOUNDARY {
        return Err(ReaderError::new(
            "privacy.freshness_attestation_boundary drifted",
        ));
    }
    Ok(())
}

fn validate_sessions_typed(
    records: &[SessionRecord],
    path: &str,
    expected_study_id: Option<&str>,
    known_misconceptions: Option<&BTreeSet<String>>,
) -> ReaderResult<()> {
    let required_targets: BTreeSet<_> = REQUIRED_TARGETS.iter().map(|item| item.0).collect();
    let mut commitments = HashSet::new();
    for (index, record) in records.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        validate_id_str(&record.study_id, &format!("{item_path}.study_id"))?;
        validate_digest_str(
            &record.record_commitment_sha256,
            &format!("{item_path}.record_commitment_sha256"),
            None,
        )?;
        if !commitments.insert(record.record_commitment_sha256.as_str()) {
            return Err(ReaderError::new(format!(
                "{item_path}.record_commitment_sha256: duplicate coded session"
            )));
        }
        if expected_study_id.is_some_and(|expected| record.study_id != expected) {
            return Err(ReaderError::new(format!(
                "{item_path}.study_id: does not match pre-registration"
            )));
        }
        if !matches!(
            record.admissibility.as_str(),
            "admissible" | "inadmissible" | "withdrawn"
        ) {
            return Err(ReaderError::new(format!(
                "{item_path}.admissibility: expected one of admissible, inadmissible, withdrawn"
            )));
        }
        let mut targets = BTreeSet::new();
        for (outcome_index, outcome) in record.target_outcomes.iter().enumerate() {
            let outcome_path = format!("{item_path}.target_outcomes[{outcome_index}]");
            if !required_targets.contains(outcome.target_id.as_str()) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.target_id: unknown target"
                )));
            }
            if !targets.insert(outcome.target_id.as_str()) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.target_id: duplicate target"
                )));
            }
            if !matches!(
                outcome.status.as_str(),
                "identified"
                    | "not-identified"
                    | "missing"
                    | "ambiguous"
                    | "multiply-coded"
                    | "unclassified"
            ) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.status: expected a closed target status"
                )));
            }
            if !matches!(
                outcome.adjudication.as_str(),
                "not-required" | "resolved" | "unresolved"
            ) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.adjudication: expected a closed adjudication state"
                )));
            }
            let final_status = matches!(outcome.status.as_str(), "identified" | "not-identified");
            if outcome.adjudication == "resolved" && !final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a resolved target outcome must carry a final status"
                )));
            }
            if outcome.adjudication == "unresolved" && final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a final target outcome cannot remain unresolved"
                )));
            }
        }
        let mut misconceptions = BTreeSet::new();
        for (outcome_index, outcome) in record.misconception_outcomes.iter().enumerate() {
            let outcome_path = format!("{item_path}.misconception_outcomes[{outcome_index}]");
            validate_id_str(
                &outcome.misconception_id,
                &format!("{outcome_path}.misconception_id"),
            )?;
            if !misconceptions.insert(outcome.misconception_id.as_str()) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.misconception_id: duplicate misconception"
                )));
            }
            if known_misconceptions.is_some_and(|known| !known.contains(&outcome.misconception_id))
            {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.misconception_id: unknown misconception"
                )));
            }
            if !matches!(
                outcome.status.as_str(),
                "present" | "absent" | "missing" | "ambiguous" | "multiply-coded" | "unclassified"
            ) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.status: expected a closed misconception status"
                )));
            }
            if !matches!(
                outcome.adjudication.as_str(),
                "not-required" | "resolved" | "unresolved"
            ) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.adjudication: expected a closed adjudication state"
                )));
            }
            if !integer_regex().is_match(&outcome.occurrences)
                || !integer_regex().is_match(&outcome.opportunities)
                || outcome.opportunities == "0"
            {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: occurrences and opportunities require canonical integer text"
                )));
            }
            let occurrences = BigNat::from_decimal(&outcome.occurrences);
            let opportunities = BigNat::from_decimal(&outcome.opportunities);
            if occurrences > opportunities {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: occurrences cannot exceed opportunities"
                )));
            }
            if outcome.status == "absent" && !occurrences.is_zero() {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: absent requires zero occurrences"
                )));
            }
            if outcome.status == "present" && occurrences.is_zero() {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: present requires at least one occurrence"
                )));
            }
            let final_status = matches!(outcome.status.as_str(), "present" | "absent");
            if outcome.adjudication == "resolved" && !final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a resolved misconception outcome must carry a final status"
                )));
            }
            if outcome.adjudication == "unresolved" && final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a final misconception outcome cannot remain unresolved"
                )));
            }
        }
        for (field, identifiers) in [
            ("deviation_ids", &record.deviation_ids),
            ("custody_attestation_ids", &record.custody_attestation_ids),
        ] {
            let mut seen = HashSet::new();
            for (identifier_index, identifier) in identifiers.iter().enumerate() {
                validate_id_str(
                    identifier,
                    &format!("{item_path}.{field}[{identifier_index}]"),
                )?;
                if !seen.insert(identifier) {
                    return Err(ReaderError::new(format!(
                        "{item_path}.{field}[{identifier_index}]: duplicate"
                    )));
                }
            }
        }
        if record.admissibility == "admissible" {
            if targets != required_targets {
                return Err(ReaderError::new(format!(
                    "{item_path}: every admissible session needs every required target exactly once"
                )));
            }
            if let Some(known) = known_misconceptions {
                let seen: BTreeSet<_> = misconceptions.iter().copied().collect();
                let expected: BTreeSet<_> = known.iter().map(String::as_str).collect();
                if seen != expected {
                    return Err(ReaderError::new(format!(
                        "{item_path}: every admissible session needs every ratified misconception exactly once"
                    )));
                }
            }
        } else if !record.target_outcomes.is_empty() || !record.misconception_outcomes.is_empty() {
            return Err(ReaderError::new(format!(
                "{item_path}: inadmissible or withdrawn sessions may not publish coded outcomes"
            )));
        }
    }
    Ok(())
}

fn validate_deviations_typed<'a>(
    deviations: &'a [DeviationRecord],
    path: &str,
) -> ReaderResult<BTreeMap<&'a str, &'a DeviationRecord>> {
    let mut result = BTreeMap::new();
    for (index, deviation) in deviations.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        validate_id_str(
            &deviation.deviation_id,
            &format!("{item_path}.deviation_id"),
        )?;
        validate_id_str(&deviation.code, &format!("{item_path}.code"))?;
        if !deviation.code.starts_with("RE-DEV-CODE-") {
            return Err(ReaderError::new(format!(
                "{item_path}.code: expected a closed RE-DEV-CODE-* value"
            )));
        }
        if !matches!(
            deviation.impact.as_str(),
            "none" | "session-inadmissible" | "holdout-void"
        ) {
            return Err(ReaderError::new(format!(
                "{item_path}.impact: expected a closed impact"
            )));
        }
        validate_id_str(
            &deviation.custody_attestation_id,
            &format!("{item_path}.custody_attestation_id"),
        )?;
        if result
            .insert(deviation.deviation_id.as_str(), deviation)
            .is_some()
        {
            return Err(ReaderError::new(format!(
                "{item_path}.deviation_id: duplicate"
            )));
        }
    }
    Ok(result)
}

fn validate_custody_typed<'a>(
    env: &ValidationEnv<'_>,
    records: &'a [CustodyRecord],
    path: &str,
) -> ReaderResult<BTreeMap<&'a str, &'a CustodyRecord>> {
    let mut result = BTreeMap::new();
    let mut external_digests = HashSet::new();
    let refs = BTreeMap::from([
        ("session-record", "custody:READER-EVIDENCE-SESSION"),
        ("study-freshness", "custody:READER-EVIDENCE-FRESHNESS"),
        ("deviation", "custody:READER-EVIDENCE-DEVIATION"),
        ("commitment", "custody:READER-EVIDENCE-COMMITMENT"),
    ]);
    for (index, record) in records.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        validate_id_str(
            &record.attestation_id,
            &format!("{item_path}.attestation_id"),
        )?;
        validate_id_str(&record.study_id, &format!("{item_path}.study_id"))?;
        if !refs.contains_key(record.scope.as_str()) {
            return Err(ReaderError::new(format!(
                "{item_path}.scope: expected a closed custody scope"
            )));
        }
        if record.scope == "session-record" {
            let commitment = record.record_commitment_sha256.as_ref().ok_or_else(|| {
                ReaderError::new(format!(
                    "{item_path}.record_commitment_sha256: session custody must bind a record"
                ))
            })?;
            validate_digest_str(
                commitment,
                &format!("{item_path}.record_commitment_sha256"),
                None,
            )?;
        } else if record.record_commitment_sha256.is_some() {
            return Err(ReaderError::new(format!(
                "{item_path}.record_commitment_sha256: only session custody may bind a record"
            )));
        }
        if refs.get(record.scope.as_str()).copied() != Some(record.r#ref.as_str()) {
            return Err(ReaderError::new(format!(
                "{item_path}.ref: custody scope requires its fixed external channel"
            )));
        }
        validate_external_or_repo_reference_str(env, &record.r#ref, &format!("{item_path}.ref"))?;
        validate_digest_str(&record.sha256, &format!("{item_path}.sha256"), None)?;
        if !external_digests.insert(record.sha256.as_str()) {
            return Err(ReaderError::new(format!(
                "{item_path}.sha256: duplicate external attestation digest"
            )));
        }
        if record.freshness_attested && record.scope != "study-freshness" {
            return Err(ReaderError::new(format!(
                "{item_path}: only a study-freshness attestation may attest freshness"
            )));
        }
        validate_digest_str(
            &record.record_sha256,
            &format!("{item_path}.record_sha256"),
            Some(&typed_canonical_sha(
                record,
                &item_path,
                Some("record_sha256"),
            )?),
        )?;
        if result
            .insert(record.attestation_id.as_str(), record)
            .is_some()
        {
            return Err(ReaderError::new(format!(
                "{item_path}.attestation_id: duplicate"
            )));
        }
    }
    Ok(result)
}

fn validate_record_links_typed(
    sessions: &[SessionRecord],
    deviations: &BTreeMap<&str, &DeviationRecord>,
    custody: &BTreeMap<&str, &CustodyRecord>,
    path: &str,
    expected_study_id: Option<&str>,
    commitment: Option<&Commitment>,
) -> ReaderResult<()> {
    let mut referenced = BTreeSet::new();
    for (attestation_id, item) in custody {
        if expected_study_id.is_some_and(|expected| item.study_id != expected) {
            return Err(ReaderError::new(format!(
                "{path}: custody attestation cites a different study"
            )));
        }
        if item.scope == "study-freshness" {
            referenced.insert(*attestation_id);
        }
    }
    for (deviation_id, deviation) in deviations {
        let attestation_id = deviation.custody_attestation_id.as_str();
        if custody
            .get(attestation_id)
            .is_none_or(|item| item.scope != "deviation")
        {
            return Err(ReaderError::new(format!(
                "{path}: deviation {deviation_id} lacks deviation custody"
            )));
        }
        referenced.insert(attestation_id);
    }
    for session in sessions {
        for deviation_id in &session.deviation_ids {
            let Some(deviation) = deviations.get(deviation_id.as_str()) else {
                return Err(ReaderError::new(format!(
                    "{path}: session cites an unknown deviation"
                )));
            };
            if session.admissibility == "admissible" && deviation.impact == "session-inadmissible" {
                return Err(ReaderError::new(format!(
                    "{path}: session-inadmissible deviation remains admitted"
                )));
            }
        }
        let mut matching_session_custody = false;
        for attestation_id in &session.custody_attestation_ids {
            let Some(item) = custody.get(attestation_id.as_str()) else {
                return Err(ReaderError::new(format!(
                    "{path}: session cites unknown custody"
                )));
            };
            if item.scope == "session-record"
                && item.record_commitment_sha256.as_deref()
                    == Some(session.record_commitment_sha256.as_str())
            {
                matching_session_custody = true;
            }
            referenced.insert(attestation_id.as_str());
        }
        if session.admissibility == "admissible" && !matching_session_custody {
            return Err(ReaderError::new(format!(
                "{path}: admitted session lacks matching record custody"
            )));
        }
        if session.admissibility == "inadmissible"
            && !session.deviation_ids.iter().any(|identifier| {
                deviations
                    .get(identifier.as_str())
                    .is_some_and(|item| item.impact == "session-inadmissible")
            })
        {
            return Err(ReaderError::new(format!(
                "{path}: inadmissible session lacks a coded exclusion deviation"
            )));
        }
    }
    if let Some(commitment) = commitment {
        let matches = custody
            .iter()
            .filter(|(_, item)| {
                item.scope == "commitment" && item.sha256 == commitment.custody_attestation_sha256
            })
            .map(|(identifier, _)| *identifier)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(ReaderError::new(format!(
                "{path}: commitment must bind exactly one custody attestation"
            )));
        }
        referenced.insert(matches[0]);
    }
    if referenced != custody.keys().copied().collect() {
        return Err(ReaderError::new(format!(
            "{path}: every public custody record must have a closed evidence role"
        )));
    }
    Ok(())
}

fn validate_pilot_run_freshness_typed(
    custody: &BTreeMap<&str, &CustodyRecord>,
    path: &str,
    expected_study_id: Option<&str>,
    run_evidence: bool,
    require_attested: bool,
) -> ReaderResult<bool> {
    let freshness = custody
        .values()
        .filter(|item| item.scope == "study-freshness")
        .copied()
        .collect::<Vec<_>>();
    if run_evidence && freshness.len() != 1 {
        return Err(ReaderError::new(format!(
            "{path}: pilot run evidence requires exactly one study-freshness custody attestation"
        )));
    }
    if !run_evidence {
        return Ok(false);
    }
    let item = freshness[0];
    if expected_study_id.is_some_and(|expected| item.study_id != expected) {
        return Err(ReaderError::new(format!(
            "{path}: pilot freshness custody cites a different study"
        )));
    }
    if require_attested && !item.freshness_attested {
        return Err(ReaderError::new(format!(
            "{path}: a completed valid pilot requires freshness_attested true"
        )));
    }
    Ok(item.freshness_attested)
}

fn empty_threshold_content_typed(rule: &ReviewedThresholdRule) -> bool {
    rule.rule_id.is_none()
        && rule.severity_taxonomy.is_empty()
        && rule.misconceptions.is_empty()
        && rule.core_misconception_ids.is_empty()
        && rule.core_failure_mode.is_none()
        && rule.repetition_unit.is_none()
        && rule.denominator.is_none()
        && rule.core_failure_threshold.is_none()
        && rule.required_target_thresholds.is_empty()
        && rule.non_core_thresholds.is_empty()
        && rule.minimum_evaluable_evidence.is_none()
        && rule.policies.missing.is_none()
        && rule.policies.ambiguous.is_none()
        && rule.policies.multiply_coded.is_none()
        && rule.policies.withdrawn.is_none()
        && rule.policies.excluded.is_none()
        && rule.policies.unclassified.is_none()
        && rule.policies.rounding.is_none()
        && rule.policies.coder_adjudication.is_none()
        && rule.rule_sha256.is_none()
}

fn validate_threshold_spec_typed(
    spec: &ThresholdSpec,
    path: &str,
    allowed_metrics: &[&str],
    scope_refs: &BTreeSet<String>,
) -> ReaderResult<()> {
    const THRESHOLD_METRICS: &[&str] = &[
        "admissible-session-count",
        "target-identification-count",
        "target-identification-rate",
        "core-finding-present",
        "core-finding-count",
        "core-finding-rate",
        "severity-session-finding-count",
        "severity-session-finding-rate",
        "severity-occurrence-count",
        "severity-occurrence-rate",
    ];
    if allowed_metrics.is_empty()
        || allowed_metrics
            .iter()
            .any(|metric| !THRESHOLD_METRICS.contains(metric))
    {
        return Err(ReaderError::new(format!(
            "{path}: internal metric registry is incomplete"
        )));
    }
    validate_id_str(&spec.threshold_id, &format!("{path}.threshold_id"))?;
    if !allowed_metrics.contains(&spec.metric.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.metric: unsupported deterministic metric"
        )));
    }
    if !["lt", "lte", "eq", "gte", "gt"].contains(&spec.operator.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.operator: invalid operator"
        )));
    }
    if !["integer", "decimal", "qualitative"].contains(&spec.value_kind.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.value_kind: invalid value kind"
        )));
    }
    validate_text_str(&spec.value, &format!("{path}.value"))?;
    validate_text_str(&spec.unit, &format!("{path}.unit"))?;
    validate_text_str(&spec.denominator, &format!("{path}.denominator"))?;
    let refs: BTreeSet<_> = spec.scope_refs.iter().cloned().collect();
    if refs.len() != spec.scope_refs.len() || &refs != scope_refs {
        return Err(ReaderError::new(format!(
            "{path}.scope_refs: must exactly match its rule scope"
        )));
    }
    if spec.evaluator_ref.is_some() {
        return Err(ReaderError::new(format!(
            "{path}.evaluator_ref: release thresholds use the deterministic built-in evaluator"
        )));
    }
    let count_contract = match spec.metric.as_str() {
        "admissible-session-count" => Some(("sessions", "none")),
        "target-identification-count" => Some(("identified-sessions", "none")),
        "core-finding-count" => Some(("findings", "none")),
        "severity-session-finding-count" => Some(("sessions", "none")),
        "severity-occurrence-count" => Some(("occurrences", "none")),
        _ => None,
    };
    let rate_denominators: &[&str] = match spec.metric.as_str() {
        "target-identification-rate" => &["coded-target-observations"],
        "core-finding-rate" => &["eligible-admissible-sessions", "coded-opportunities"],
        "severity-session-finding-rate" => &["eligible-admissible-sessions"],
        "severity-occurrence-rate" => &["coded-opportunities"],
        _ => &[],
    };
    if spec.metric == "core-finding-present" {
        if spec.value_kind != "qualitative"
            || spec.operator != "eq"
            || spec.value != "present"
            || spec.unit != "presence"
            || spec.denominator != "none"
        {
            return Err(ReaderError::new(format!(
                "{path}: single-finding core veto must compare presence exactly"
            )));
        }
    } else if let Some((expected_unit, expected_denominator)) = count_contract {
        if spec.value_kind != "integer"
            || spec.unit != expected_unit
            || spec.denominator != expected_denominator
            || !integer_regex().is_match(&spec.value)
            || !decimal_positive(&spec.value)
        {
            return Err(ReaderError::new(format!(
                "{path}: count threshold must admit reachable below, exact, and above cases"
            )));
        }
    } else if !rate_denominators.is_empty() {
        if spec.value_kind != "decimal"
            || spec.unit != "proportion"
            || !rate_denominators.contains(&spec.denominator.as_str())
            || !decimal_positive(&spec.value)
            || !decimal_less_than_one(&spec.value)
        {
            return Err(ReaderError::new(format!(
                "{path}: rate threshold must admit reachable below, exact, and above cases"
            )));
        }
    } else {
        return Err(ReaderError::new(format!(
            "{path}.metric: unsupported deterministic metric"
        )));
    }
    Ok(())
}

fn validate_threshold_rule_contract_typed(
    rule: &ReviewedThresholdRule,
    threshold_status: &str,
    ratification_present: bool,
    valid_pilot: bool,
) -> ReaderResult<BTreeSet<String>> {
    if rule.evaluation_order != EVALUATION_ORDER {
        return Err(ReaderError::new(
            "threshold_rule.evaluation_order must preserve the fixed order",
        ));
    }
    if !rule.aggregate_offset_prohibited {
        return Err(ReaderError::new(
            "threshold_rule must preserve the no-aggregate core veto",
        ));
    }
    if !["pending-pilot", "candidate", "author-ratified"].contains(&threshold_status) {
        return Err(ReaderError::new("threshold_status: invalid state"));
    }
    if !valid_pilot {
        if threshold_status != "pending-pilot" || !empty_threshold_content_typed(rule) {
            return Err(ReaderError::new(
                "threshold taxonomy and values are prohibited until a valid completed pilot exists",
            ));
        }
        if ratification_present {
            return Err(ReaderError::new(
                "ratification is prohibited before a valid completed pilot",
            ));
        }
        return Ok(BTreeSet::new());
    }
    if threshold_status == "pending-pilot" {
        if !empty_threshold_content_typed(rule) || ratification_present {
            return Err(ReaderError::new(
                "pending-pilot must not carry a candidate rule or ratification",
            ));
        }
        return Ok(BTreeSet::new());
    }
    let populated = rule.populated("threshold_rule")?;
    validate_id_str(&populated.rule_id, "threshold_rule.rule_id")?;
    let mut severities = BTreeSet::new();
    for (index, item) in populated.severity_taxonomy.iter().enumerate() {
        let path = format!("threshold_rule.severity_taxonomy[{index}]");
        validate_id_str(&item.severity_id, &format!("{path}.severity_id"))?;
        if !severities.insert(item.severity_id.clone()) {
            return Err(ReaderError::new(format!(
                "{path}.severity_id: duplicate {}",
                item.severity_id
            )));
        }
        validate_text_str(&item.label, &format!("{path}.label"))?;
        validate_text_str(&item.definition, &format!("{path}.definition"))?;
        validate_text_str(
            &item.classification_boundary,
            &format!("{path}.classification_boundary"),
        )?;
    }
    if severities.is_empty() {
        return Err(ReaderError::new(
            "candidate threshold rule requires a severity taxonomy",
        ));
    }
    let mut misconceptions = BTreeMap::new();
    for (index, item) in populated.misconceptions.iter().enumerate() {
        let path = format!("threshold_rule.misconceptions[{index}]");
        validate_id_str(&item.misconception_id, &format!("{path}.misconception_id"))?;
        validate_text_str(&item.definition, &format!("{path}.definition"))?;
        validate_id_str(&item.severity_id, &format!("{path}.severity_id"))?;
        if !severities.contains(&item.severity_id) {
            return Err(ReaderError::new(format!(
                "{path}.severity_id: unknown severity"
            )));
        }
        if misconceptions
            .insert(item.misconception_id.clone(), item)
            .is_some()
        {
            return Err(ReaderError::new(format!(
                "{path}.misconception_id: duplicate {}",
                item.misconception_id
            )));
        }
    }
    if misconceptions.is_empty() {
        return Err(ReaderError::new(
            "candidate threshold rule requires stable misconception IDs",
        ));
    }
    let declared_core: BTreeSet<_> = populated.core_misconception_ids.iter().cloned().collect();
    let actual_core: BTreeSet<_> = misconceptions
        .iter()
        .filter(|(_, item)| item.core)
        .map(|(identifier, _)| identifier.clone())
        .collect();
    if declared_core.len() != populated.core_misconception_ids.len()
        || declared_core != actual_core
        || actual_core.is_empty()
    {
        return Err(ReaderError::new(
            "core_misconception_ids must exactly match non-empty core mappings",
        ));
    }
    if !["single", "repeated"].contains(&populated.core_failure_mode.as_str()) {
        return Err(ReaderError::new(
            "threshold_rule.core_failure_mode: invalid",
        ));
    }
    if !["admissible-session", "coded-opportunity"].contains(&populated.repetition_unit.as_str()) {
        return Err(ReaderError::new("threshold_rule.repetition_unit: invalid"));
    }
    let core_metrics: &[&str] = if populated.core_failure_mode == "single" {
        &["core-finding-present"]
    } else {
        &["core-finding-count", "core-finding-rate"]
    };
    validate_threshold_spec_typed(
        &populated.core_failure_threshold,
        "threshold_rule.core_failure_threshold",
        core_metrics,
        &actual_core,
    )?;
    let core = &populated.core_failure_threshold;
    if core.metric != "core-finding-present"
        && (!matches!(core.operator.as_str(), "gte" | "gt")
            || !decimal_positive(&core.value)
            || (core.metric == "core-finding-rate"
                && core.operator == "gt"
                && !decimal_less_than_one(&core.value)))
    {
        return Err(ReaderError::new(
            "repeated core veto must use a positive, reachable adverse boundary",
        ));
    }
    let expected_denominator = if core.metric == "core-finding-rate" {
        if populated.repetition_unit == "admissible-session" {
            "eligible-admissible-sessions"
        } else {
            "coded-opportunities"
        }
    } else {
        "none"
    };
    if populated.denominator != expected_denominator || core.denominator != expected_denominator {
        return Err(ReaderError::new(
            "threshold_rule.denominator must match the selected core branch and metric",
        ));
    }
    let policies = &populated.policies;
    for (path, policy) in [
        ("ambiguous", &policies.ambiguous),
        ("missing", &policies.missing),
        ("multiply_coded", &policies.multiply_coded),
        ("unclassified", &policies.unclassified),
    ] {
        if ![
            "count-adverse",
            "exclude-observation",
            "study-not-evaluable",
            "require-adjudication",
        ]
        .contains(&policy.as_str())
        {
            return Err(ReaderError::new(format!(
                "threshold_rule.policies.{path}: invalid policy"
            )));
        }
    }
    if policies.withdrawn != "exclude-session" || policies.excluded != "exclude-session" {
        return Err(ReaderError::new(
            "threshold_rule withdrawal/exclusion policy drifted",
        ));
    }
    if policies.rounding != "exact-decimal-no-rounding" {
        return Err(ReaderError::new(
            "threshold_rule.policies.rounding must preserve exact comparison",
        ));
    }
    if ![
        "unresolved-count-adverse",
        "unresolved-exclude-observation",
        "unresolved-not-evaluable",
    ]
    .contains(&policies.coder_adjudication.as_str())
    {
        return Err(ReaderError::new(
            "threshold_rule.policies.coder_adjudication: invalid policy",
        ));
    }
    let mut threshold_ids = BTreeSet::from([core.threshold_id.clone()]);
    let target_ids: BTreeSet<_> = REQUIRED_TARGETS
        .iter()
        .map(|(identifier, _)| (*identifier).to_owned())
        .collect();
    let mut required = BTreeSet::new();
    for (index, item) in populated.required_target_thresholds.iter().enumerate() {
        let path = format!("threshold_rule.required_target_thresholds[{index}]");
        if !target_ids.contains(&item.target_id) || !required.insert(item.target_id.clone()) {
            return Err(ReaderError::new(format!(
                "{path}.target_id: unknown or duplicate target"
            )));
        }
        validate_threshold_spec_typed(
            &item.threshold,
            &format!("{path}.threshold"),
            &["target-identification-count", "target-identification-rate"],
            &BTreeSet::from([item.target_id.clone()]),
        )?;
        if !matches!(item.threshold.operator.as_str(), "gte" | "gt")
            || !decimal_positive(&item.threshold.value)
            || (item.threshold.metric == "target-identification-rate"
                && item.threshold.operator == "gt"
                && !decimal_less_than_one(&item.threshold.value))
        {
            return Err(ReaderError::new(format!(
                "{path}.threshold: target success boundary must be positive and reachable"
            )));
        }
        if !threshold_ids.insert(item.threshold.threshold_id.clone()) {
            return Err(ReaderError::new(
                "threshold IDs must be unique across the complete rule",
            ));
        }
    }
    if required != target_ids {
        return Err(ReaderError::new(
            "required_target_thresholds must cover every required target",
        ));
    }
    let non_core_severities: BTreeSet<_> = misconceptions
        .values()
        .filter(|item| !item.core)
        .map(|item| item.severity_id.clone())
        .collect();
    let mut mapped_non_core = BTreeSet::new();
    for (index, item) in populated.non_core_thresholds.iter().enumerate() {
        let path = format!("threshold_rule.non_core_thresholds[{index}]");
        if !non_core_severities.contains(&item.severity_id)
            || !mapped_non_core.insert(item.severity_id.clone())
        {
            return Err(ReaderError::new(format!(
                "{path}.severity_id: unknown, core, or duplicate severity"
            )));
        }
        validate_threshold_spec_typed(
            &item.threshold,
            &format!("{path}.threshold"),
            &[
                "severity-session-finding-count",
                "severity-session-finding-rate",
                "severity-occurrence-count",
                "severity-occurrence-rate",
            ],
            &BTreeSet::from([item.severity_id.clone()]),
        )?;
        if !matches!(item.threshold.operator.as_str(), "lt" | "lte")
            || (item.threshold.operator == "lt" && !decimal_positive(&item.threshold.value))
            || (item.threshold.metric.ends_with("-rate")
                && item.threshold.operator == "lte"
                && !decimal_less_than_one(&item.threshold.value))
        {
            return Err(ReaderError::new(format!(
                "{path}.threshold: non-core boundary must be adverse and falsifiable"
            )));
        }
        if !threshold_ids.insert(item.threshold.threshold_id.clone()) {
            return Err(ReaderError::new(
                "threshold IDs must be unique across the complete rule",
            ));
        }
    }
    if mapped_non_core != non_core_severities {
        return Err(ReaderError::new(
            "non_core_thresholds must cover every used non-core severity",
        ));
    }
    validate_threshold_spec_typed(
        &populated.minimum_evaluable_evidence,
        "threshold_rule.minimum_evaluable_evidence",
        &["admissible-session-count"],
        &BTreeSet::from([populated.rule_id.clone()]),
    )?;
    let minimum = &populated.minimum_evaluable_evidence;
    if !matches!(minimum.operator.as_str(), "gte" | "gt") || !decimal_positive(&minimum.value) {
        return Err(ReaderError::new(
            "minimum evaluable evidence must require a positive admitted count",
        ));
    }
    if !threshold_ids.insert(minimum.threshold_id.clone()) {
        return Err(ReaderError::new(
            "threshold IDs must be unique across the complete rule",
        ));
    }
    validate_digest_str(
        &populated.rule_sha256,
        "threshold_rule.rule_sha256",
        Some(&typed_canonical_sha(
            &populated,
            "threshold_rule",
            Some("rule_sha256"),
        )?),
    )?;
    Ok(misconceptions.keys().cloned().collect())
}

fn validate_threshold_rule_typed(
    source: &ReaderEvidenceSource,
    valid_pilot: bool,
) -> ReaderResult<BTreeSet<String>> {
    validate_threshold_rule_contract_typed(
        &source.threshold_rule,
        &source.threshold_status,
        source.ratification.is_some(),
        valid_pilot,
    )
}

fn reviewed_rule_from_populated(rule: &ThresholdRule) -> ReviewedThresholdRule {
    ReviewedThresholdRule {
        rule_id: Some(rule.rule_id.clone()),
        severity_taxonomy: rule.severity_taxonomy.clone(),
        misconceptions: rule.misconceptions.clone(),
        core_misconception_ids: rule.core_misconception_ids.clone(),
        core_failure_mode: Some(rule.core_failure_mode.clone()),
        repetition_unit: Some(rule.repetition_unit.clone()),
        denominator: Some(rule.denominator.clone()),
        core_failure_threshold: Some(rule.core_failure_threshold.clone()),
        required_target_thresholds: rule.required_target_thresholds.clone(),
        non_core_thresholds: rule.non_core_thresholds.clone(),
        minimum_evaluable_evidence: Some(rule.minimum_evaluable_evidence.clone()),
        policies: ReviewedThresholdPolicies {
            missing: Some(rule.policies.missing.clone()),
            ambiguous: Some(rule.policies.ambiguous.clone()),
            multiply_coded: Some(rule.policies.multiply_coded.clone()),
            withdrawn: Some(rule.policies.withdrawn.clone()),
            excluded: Some(rule.policies.excluded.clone()),
            unclassified: Some(rule.policies.unclassified.clone()),
            rounding: Some(rule.policies.rounding.clone()),
            coder_adjudication: Some(rule.policies.coder_adjudication.clone()),
        },
        evaluation_order: rule.evaluation_order.clone(),
        aggregate_offset_prohibited: rule.aggregate_offset_prohibited,
        rule_sha256: Some(rule.rule_sha256.clone()),
    }
}

fn validate_populated_threshold_rule_typed(rule: &ThresholdRule) -> ReaderResult<BTreeSet<String>> {
    validate_threshold_rule_contract_typed(
        &reviewed_rule_from_populated(rule),
        "author-ratified",
        true,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_holdout_pre_registration_typed(
    env: &ValidationEnv<'_>,
    registration: &HoldoutPreRegistration,
    path: &str,
    verify_live: bool,
    fixed_protocol_sha256: &str,
    expected_structural_checker_sha256: Option<&str>,
    expected_predecessor_attempt_sha256: Option<&str>,
    expected_prior_history_head_sha256: Option<&str>,
    enforce_history_binding: bool,
) -> ReaderResult<()> {
    validate_id_str(&registration.study_id, &format!("{path}.study_id"))?;
    validate_date_str(
        &registration.registered_date,
        &format!("{path}.registered_date"),
    )?;
    validate_history_binding_typed(
        registration.predecessor_attempt_sha256.as_deref(),
        &registration.prior_history_head_sha256,
        path,
        expected_predecessor_attempt_sha256,
        expected_prior_history_head_sha256,
        enforce_history_binding,
    )?;
    validate_digest_str(
        &registration.fixed_protocol_sha256,
        &format!("{path}.fixed_protocol_sha256"),
        Some(fixed_protocol_sha256),
    )?;
    for (key, digest_value) in [
        ("rule_sha256", &registration.rule_sha256),
        ("ratification_sha256", &registration.ratification_sha256),
        ("evidence_gate_sha256", &registration.evidence_gate_sha256),
    ] {
        validate_digest_str(digest_value, &format!("{path}.{key}"), None)?;
    }
    validate_digest_str(
        &registration.structural_checker_sha256,
        &format!("{path}.structural_checker_sha256"),
        expected_structural_checker_sha256,
    )?;
    for (key, artifact) in [
        ("revised_instrument", &registration.revised_instrument),
        ("rubric", &registration.rubric),
        ("sample_rule", &registration.sample_rule),
        ("recruitment_rule", &registration.recruitment_rule),
        ("disclosure_set", &registration.disclosure_set),
        ("study_protocol", &registration.study_protocol),
    ] {
        validate_artifact_typed(env, artifact, &format!("{path}.{key}"), verify_live)?;
    }
    validate_id_str(
        &registration.release_candidate.candidate_id,
        &format!("{path}.release_candidate.candidate_id"),
    )?;
    if registration.release_candidate.artifacts.is_empty() {
        return Err(ReaderError::new(format!(
            "{path}.release_candidate.artifacts: must not be empty"
        )));
    }
    let mut artifact_ids = HashSet::new();
    let mut artifact_refs = HashSet::new();
    for (index, artifact) in registration.release_candidate.artifacts.iter().enumerate() {
        let item_path = format!("{path}.release_candidate.artifacts[{index}]");
        validate_artifact_typed(env, artifact, &item_path, verify_live)?;
        if !artifact_ids.insert(artifact.artifact_id.as_str())
            || !artifact_refs.insert(artifact.r#ref.as_str())
        {
            return Err(ReaderError::new(format!(
                "{item_path}: duplicate identity or reference"
            )));
        }
    }
    validate_digest_str(
        &registration.release_candidate.manifest_sha256,
        &format!("{path}.release_candidate.manifest_sha256"),
        Some(&typed_canonical_sha(
            &registration.release_candidate,
            &format!("{path}.release_candidate"),
            Some("manifest_sha256"),
        )?),
    )?;
    if let Some(commitment) = &registration.commitment {
        validate_id_str(
            &commitment.commitment_id,
            &format!("{path}.commitment.commitment_id"),
        )?;
        for (key, value) in [
            (
                "nonce_commitment_sha256",
                &commitment.nonce_commitment_sha256,
            ),
            (
                "committed_preimage_sha256",
                &commitment.committed_preimage_sha256,
            ),
            (
                "custody_attestation_sha256",
                &commitment.custody_attestation_sha256,
            ),
        ] {
            validate_digest_str(value, &format!("{path}.commitment.{key}"), None)?;
        }
    }
    validate_freeze_binding_typed(
        env,
        &registration.freeze_binding,
        registration,
        &format!("{path}.freeze_binding"),
        "pre_registration_sha256",
        HistoricalPayloadKind::HoldoutPreRegistration,
    )?;
    if &registration.freeze_binding.frozen_at[..10] < registration.registered_date.as_str() {
        return Err(ReaderError::new(format!(
            "{path}.freeze_binding: freeze cannot precede registration"
        )));
    }
    validate_digest_str(
        &registration.pre_registration_sha256,
        &format!("{path}.pre_registration_sha256"),
        Some(&typed_canonical_sha(
            registration,
            path,
            Some("pre_registration_sha256"),
        )?),
    )?;
    Ok(())
}

fn validate_frozen_holdout_payload_typed(attempt: &HoldoutAttempt, path: &str) -> ReaderResult<()> {
    if attempt.attempt_result != "not-run"
        || !attempt.session_records.is_empty()
        || !attempt.deviations.is_empty()
        || attempt.result_receipt.is_some()
        || attempt.commitment_reveal.is_some()
        || attempt.gate_admission_receipt.is_some()
    {
        return Err(ReaderError::new(format!(
            "{path}: frozen holdout cannot carry run evidence or a result"
        )));
    }
    let Some(commitment) = &attempt.pre_registration.commitment else {
        if !attempt.custody_attestations.is_empty() {
            return Err(ReaderError::new(format!(
                "{path}: frozen holdout without a commitment cannot carry custody evidence"
            )));
        }
        return Ok(());
    };
    let matching = attempt
        .custody_attestations
        .iter()
        .filter(|item| {
            item.scope == "commitment" && item.sha256 == commitment.custody_attestation_sha256
        })
        .count();
    if attempt.custody_attestations.len() != 1 || matching != 1 {
        return Err(ReaderError::new(format!(
            "{path}: frozen private commitment requires exactly one matching commitment custody attestation"
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_commitment_reveal_typed(
    env: &ValidationEnv<'_>,
    reveal: Option<&CommitmentReveal>,
    path: &str,
    commitment: Option<&Commitment>,
    custody: &BTreeMap<&str, &CustodyRecord>,
    attempt_status: &str,
    verify_live: bool,
    require_present: bool,
    completed_at: Option<&str>,
) -> ReaderResult<()> {
    let Some(reveal) = reveal else {
        if require_present {
            return Err(ReaderError::new(format!(
                "{path}: completed committed holdout requires a reveal"
            )));
        }
        return Ok(());
    };
    let Some(commitment) = commitment else {
        return Err(ReaderError::new(format!(
            "{path}: reveal requires a frozen commitment"
        )));
    };
    if attempt_status != "completed" {
        return Err(ReaderError::new(format!(
            "{path}: reveal is permitted only for a completed holdout"
        )));
    }
    if reveal.commitment_id != commitment.commitment_id {
        return Err(ReaderError::new(format!(
            "{path}.commitment_id: does not match frozen commitment"
        )));
    }
    validate_timestamp_str(&reveal.revealed_at, &format!("{path}.revealed_at"))?;
    if completed_at.is_some_and(|completed| reveal.revealed_at.as_str() <= completed) {
        return Err(ReaderError::new(format!(
            "{path}.revealed_at: reveal must strictly follow completion"
        )));
    }
    if !nonce_regex().is_match(&reveal.nonce_hex) {
        return Err(ReaderError::new(format!(
            "{path}.nonce_hex: expected at least 32 bytes of lowercase hex"
        )));
    }
    validate_artifact_typed(
        env,
        &reveal.preimage,
        &format!("{path}.preimage"),
        verify_live,
    )?;
    validate_digest_str(
        &reveal.preimage.sha256,
        &format!("{path}.preimage.sha256"),
        Some(&commitment.committed_preimage_sha256),
    )?;
    let mut opening = decode_hex(&reveal.nonce_hex).expect("validated nonce");
    opening.push(0);
    opening.extend(decode_hex(&reveal.preimage.sha256).expect("validated artifact digest"));
    validate_digest_str(
        &commitment.nonce_commitment_sha256,
        "holdout commitment nonce_commitment_sha256",
        Some(&sha256(opening)),
    )?;
    validate_id_str(
        &reveal.custody_attestation_id,
        &format!("{path}.custody_attestation_id"),
    )?;
    if custody
        .get(reveal.custody_attestation_id.as_str())
        .is_none_or(|record| {
            record.scope != "commitment" || record.sha256 != commitment.custody_attestation_sha256
        })
    {
        return Err(ReaderError::new(format!(
            "{path}: reveal lacks the exact commitment custody attestation"
        )));
    }
    validate_digest_str(
        &reveal.reveal_sha256,
        &format!("{path}.reveal_sha256"),
        Some(&typed_canonical_sha(reveal, path, Some("reveal_sha256"))?),
    )?;
    Ok(())
}

fn validate_result_receipt_typed(
    receipt: &ResultReceipt,
    path: &str,
    registration: &HoldoutPreRegistration,
    rule: &ThresholdRule,
    sessions: &[SessionRecord],
    deviations: &[DeviationRecord],
    custody: &[CustodyRecord],
) -> ReaderResult<()> {
    validate_id_str(&receipt.receipt_id, &format!("{path}.receipt_id"))?;
    validate_timestamp_str(&receipt.completed_at, &format!("{path}.completed_at"))?;
    if receipt.study_id != registration.study_id {
        return Err(ReaderError::new(format!(
            "{path}.study_id: does not match holdout preregistration"
        )));
    }
    let digest_links = [
        (
            "pre_registration_sha256",
            &receipt.pre_registration_sha256,
            registration.pre_registration_sha256.clone(),
        ),
        (
            "rule_sha256",
            &receipt.rule_sha256,
            registration.rule_sha256.clone(),
        ),
        (
            "candidate_manifest_sha256",
            &receipt.candidate_manifest_sha256,
            registration.release_candidate.manifest_sha256.clone(),
        ),
        (
            "evidence_gate_sha256",
            &receipt.evidence_gate_sha256,
            registration.evidence_gate_sha256.clone(),
        ),
        (
            "coded_records_sha256",
            &receipt.coded_records_sha256,
            typed_canonical_sha(&sessions, path, None)?,
        ),
        (
            "structural_checker_sha256",
            &receipt.structural_checker_sha256,
            registration.structural_checker_sha256.clone(),
        ),
        (
            "deviations_sha256",
            &receipt.deviations_sha256,
            typed_canonical_sha(&deviations, path, None)?,
        ),
        (
            "custody_records_sha256",
            &receipt.custody_records_sha256,
            typed_canonical_sha(&custody, path, None)?,
        ),
    ];
    for (key, declared, expected) in digest_links {
        validate_digest_str(declared, &format!("{path}.{key}"), Some(&expected))?;
    }
    if !["valid", "invalid"].contains(&receipt.protocol_validity.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.protocol_validity: invalid state"
        )));
    }
    if !["not-evaluable", "fail", "pass"].contains(&receipt.verdict.as_str()) {
        return Err(ReaderError::new(format!("{path}.verdict: invalid state")));
    }
    let trace = evaluate_holdout(rule, sessions, &receipt.protocol_validity)?;
    validate_digest_str(
        &receipt.evaluation_trace_sha256,
        &format!("{path}.evaluation_trace_sha256"),
        Some(&typed_canonical_sha(&trace, path, None)?),
    )?;
    if receipt.verdict != trace.verdict {
        return Err(ReaderError::new(format!(
            "{path}.verdict: differs from deterministic evaluation"
        )));
    }
    let classifications = sessions
        .iter()
        .map(|record| {
            serde_json::json!({
                "record_commitment_sha256": record.record_commitment_sha256,
                "admissibility": record.admissibility,
            })
        })
        .collect::<Vec<_>>();
    validate_digest_str(
        &receipt.session_classification_sha256,
        &format!("{path}.session_classification_sha256"),
        Some(&canonical_sha(&Value::Array(classifications), None)?),
    )?;
    let expected_custody_digests = custody
        .iter()
        .map(|record| record.sha256.clone())
        .collect::<Vec<_>>();
    if receipt.custody_attestation_sha256s != expected_custody_digests {
        return Err(ReaderError::new(format!(
            "{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )));
    }
    for (index, value) in receipt.custody_attestation_sha256s.iter().enumerate() {
        validate_digest_str(
            value,
            &format!("{path}.custody_attestation_sha256s[{index}]"),
            None,
        )?;
    }
    validate_digest_str(
        &receipt.receipt_sha256,
        &format!("{path}.receipt_sha256"),
        Some(&typed_canonical_sha(receipt, path, Some("receipt_sha256"))?),
    )?;
    Ok(())
}

fn validate_gate_admission_receipt_typed(
    env: &ValidationEnv<'_>,
    receipt: &GateReceipt,
    path: &str,
    gate_input: &GateInput,
    expected_decision: &str,
    execute_live: bool,
) -> ReaderResult<()> {
    if receipt.schema_version != 1 {
        return Err(ReaderError::new(format!(
            "{path}.schema_version must be integer 1"
        )));
    }
    validate_digest_str(
        &receipt.input_sha256,
        &format!("{path}.input_sha256"),
        Some(&typed_canonical_sha(gate_input, path, None)?),
    )?;
    validate_digest_str(
        &receipt.evidence_gate_sha256,
        &format!("{path}.evidence_gate_sha256"),
        Some(&gate_input.evidence_gate_sha256),
    )?;
    if !["admit", "reject"].contains(&receipt.decision.as_str()) {
        return Err(ReaderError::new(format!("{path}.decision: invalid state")));
    }
    if receipt.decision != expected_decision {
        return Err(ReaderError::new(format!(
            "{path}.decision must be {expected_decision} for the validated result"
        )));
    }
    validate_digest_str(
        &receipt.receipt_sha256,
        &format!("{path}.receipt_sha256"),
        Some(&typed_canonical_sha(receipt, path, Some("receipt_sha256"))?),
    )?;
    if execute_live {
        let live = evaluate_reader_evidence(env.context, env.protocol_decision, gate_input)?;
        if &live != receipt {
            return Err(ReaderError::new(format!(
                "{path}: stored receipt differs from the bound gate output"
            )));
        }
    }
    Ok(())
}

struct RouteValidationTyped<'a> {
    status: &'a str,
    gate_sha256: Option<&'a str>,
    checker_sha256: &'a str,
}

fn validate_route_readiness_typed<'a>(
    env: &ValidationEnv<'_>,
    source: &'a ReaderEvidenceSource,
    valid_pilot: bool,
    expected_structural_checker_sha256: Option<&str>,
) -> ReaderResult<RouteValidationTyped<'a>> {
    let route = &source.route;
    if route.route_id != "FS-RTE-06" {
        return Err(ReaderError::new("route.route_id must be FS-RTE-06"));
    }
    if !["unbuilt", "available"].contains(&route.route_status.as_str()) {
        return Err(ReaderError::new("route.route_status: invalid state"));
    }
    if route.evidence_contract_status != "implemented" {
        return Err(ReaderError::new(
            "route.evidence_contract_status must record this implemented contract",
        ));
    }
    validate_artifact_typed(
        env,
        &route.structural_checker_binding,
        "route.structural_checker_binding",
        true,
    )?;
    if route.structural_checker_binding.artifact_id != STRUCTURAL_CHECKER_ARTIFACT_ID
        || route.structural_checker_binding.r#ref != STRUCTURAL_CHECKER_REF
    {
        return Err(ReaderError::new(
            "route.structural_checker_binding must bind the fixed structural checker",
        ));
    }
    validate_digest_str(
        &route.structural_checker_binding.sha256,
        "route.structural_checker_binding.sha256",
        expected_structural_checker_sha256,
    )?;
    let gate_sha256 = if let Some(gate) = &route.evidence_admission_gate_binding {
        validate_artifact_typed(env, gate, "route.evidence_admission_gate_binding", true)?;
        if gate.artifact_id != EVIDENCE_GATE_ARTIFACT_ID || gate.r#ref != EVIDENCE_GATE_REF {
            return Err(ReaderError::new(
                "route.evidence_admission_gate_binding must bind the fixed executable gate",
            ));
        }
        native_gate_self_test(env.context).map_err(|error| {
            ReaderError::new(format!(
                "route evidence gate must pass its fixed executable self-test: {error}"
            ))
        })?;
        Some(gate.sha256.as_str())
    } else {
        None
    };
    if let Some(reviewer) = &route.reviewer_custody_attestation {
        validate_id_str(
            &reviewer.attestation_id,
            "route.reviewer_custody_attestation.attestation_id",
        )?;
        if reviewer.scope != "reader-evidence-gate-review" {
            return Err(ReaderError::new(
                "route reviewer attestation has the wrong closed scope",
            ));
        }
        let gate_digest = gate_sha256.ok_or_else(|| {
            ReaderError::new("route reviewer attestation requires the executable gate binding")
        })?;
        validate_digest_str(
            &reviewer.evidence_gate_sha256,
            "route.reviewer_custody_attestation.evidence_gate_sha256",
            Some(gate_digest),
        )?;
        if reviewer.r#ref != "custody:READER-EVIDENCE-GATE-REVIEW" {
            return Err(ReaderError::new(
                "route reviewer attestation must use the fixed external custody channel",
            ));
        }
        validate_external_or_repo_reference_str(
            env,
            &reviewer.r#ref,
            "route.reviewer_custody_attestation.ref",
        )?;
        validate_date_str(
            &reviewer.attested_date,
            "route.reviewer_custody_attestation.attested_date",
        )?;
        validate_digest_str(
            &reviewer.sha256,
            "route.reviewer_custody_attestation.sha256",
            None,
        )?;
    }
    if ![
        "not-run",
        "watched-failing",
        "failed-to-fail",
        "indeterminate",
    ]
    .contains(&route.negative_control_status.as_str())
    {
        return Err(ReaderError::new(
            "route.negative_control_status: invalid state",
        ));
    }
    if route.negative_control_status != source.pilot.control_status {
        return Err(ReaderError::new(
            "route.negative_control_status must equal the active pilot control",
        ));
    }
    let available = route.reviewer_custody_attestation.is_some()
        && gate_sha256.is_some()
        && valid_pilot
        && route.negative_control_status == "watched-failing"
        && source.threshold_status == "author-ratified";
    let expected_route = if available { "available" } else { "unbuilt" };
    if route.route_status != expected_route {
        return Err(ReaderError::new(format!(
            "route.route_status must be {expected_route} for its complete tuple"
        )));
    }
    Ok(RouteValidationTyped {
        status: &route.route_status,
        gate_sha256,
        checker_sha256: &route.structural_checker_binding.sha256,
    })
}

fn validate_claim_typed(
    env: &ValidationEnv<'_>,
    source: &ReaderEvidenceSource,
    route_status: &str,
    valid_holdout_pass: bool,
) -> ReaderResult<()> {
    if source.claim.claim_id != "FS-CLM-37" {
        return Err(ReaderError::new("claim.claim_id must be FS-CLM-37"));
    }
    validate_repo_reference_str(env, &source.claim.result_ref, "claim.result_ref")?;
    let expected = if valid_holdout_pass {
        ("Evidenced", "none")
    } else if route_status == "available" {
        ("Unestablished", "evidence-pending")
    } else {
        ("Unestablished", "route-unbuilt")
    };
    if source.claim.posture != expected.0 || source.claim.disposition != expected.1 {
        return Err(ReaderError::new(format!(
            "claim posture/disposition must be {}/{} for current evidence state",
            expected.0, expected.1
        )));
    }
    Ok(())
}

fn validate_acceptance_typed(source: &ReaderEvidenceSource) -> ReaderResult<()> {
    if source.acceptance.gate_c_satisfied {
        return Err(ReaderError::new(
            "reader evidence alone may never satisfy Gate C",
        ));
    }
    if source.acceptance.permitted_claim != "none" {
        return Err(ReaderError::new(
            "this contract may not rewrite Gate C's permitted claim",
        ));
    }
    if source.acceptance.limits.is_empty()
        || source
            .acceptance
            .limits
            .iter()
            .any(|item| validate_text_str(item, "acceptance.limits").is_err())
        || source
            .acceptance
            .limits
            .iter()
            .collect::<HashSet<_>>()
            .len()
            != source.acceptance.limits.len()
    {
        return Err(ReaderError::new(
            "acceptance.limits must be a non-empty unique text list",
        ));
    }
    Ok(())
}

struct HistorySnapshotTyped<'a> {
    pilot_attempts: &'a [PilotAttemptRecord],
    holdout_attempts: &'a [HoldoutAttempt],
    head: String,
}

fn validated_history_snapshot_typed<'a>(
    source: &'a ReaderEvidenceSource,
    path: &str,
) -> ReaderResult<HistorySnapshotTyped<'a>> {
    let mut pilot_digests = Vec::new();
    let mut previous = None;
    for (index, attempt) in source.pilot.attempts.iter().enumerate() {
        let item_path = format!("{path}.pilot.attempts[{index}]");
        match (index, attempt.previous_attempt_sha256.as_deref()) {
            (0, None) => {}
            (0, Some(_)) => {
                return Err(ReaderError::new(format!(
                    "{item_path}.previous_attempt_sha256: first attempt must be null"
                )));
            }
            (_, Some(declared)) => {
                validate_digest_str(
                    declared,
                    &format!("{item_path}.previous_attempt_sha256"),
                    previous,
                )?;
            }
            (_, None) => {
                return Err(ReaderError::new(format!(
                    "{item_path}.previous_attempt_sha256: expected prior digest"
                )));
            }
        }
        validate_digest_str(
            &attempt.attempt_sha256,
            &format!("{item_path}.attempt_sha256"),
            Some(&typed_canonical_sha(
                attempt,
                &item_path,
                Some("attempt_sha256"),
            )?),
        )?;
        previous = Some(attempt.attempt_sha256.as_str());
        pilot_digests.push(attempt.attempt_sha256.as_str());
    }
    let mut holdout_digests = Vec::new();
    previous = None;
    for (index, attempt) in source.holdout.attempts.iter().enumerate() {
        let item_path = format!("{path}.holdout.attempts[{index}]");
        match (index, attempt.previous_attempt_sha256.as_deref()) {
            (0, None) => {}
            (0, Some(_)) => {
                return Err(ReaderError::new(format!(
                    "{item_path}.previous_attempt_sha256: first attempt must be null"
                )));
            }
            (_, Some(declared)) => {
                validate_digest_str(
                    declared,
                    &format!("{item_path}.previous_attempt_sha256"),
                    previous,
                )?;
            }
            (_, None) => {
                return Err(ReaderError::new(format!(
                    "{item_path}.previous_attempt_sha256: expected prior digest"
                )));
            }
        }
        validate_digest_str(
            &attempt.attempt_sha256,
            &format!("{item_path}.attempt_sha256"),
            Some(&typed_canonical_sha(
                attempt,
                &item_path,
                Some("attempt_sha256"),
            )?),
        )?;
        previous = Some(attempt.attempt_sha256.as_str());
        holdout_digests.push(attempt.attempt_sha256.as_str());
    }
    Ok(HistorySnapshotTyped {
        pilot_attempts: &source.pilot.attempts,
        holdout_attempts: &source.holdout.attempts,
        head: history_head_sha256(pilot_digests, holdout_digests),
    })
}

fn nearest_previous_reader_evidence_typed(
    context: &Context,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<Option<(String, Vec<u8>, ReaderEvidenceSource)>> {
    nearest_previous_reader_evidence(context, source_raw, source_commit)?
        .map(|(commit, raw, _value)| {
            typed_reader_source_bytes(&raw, "history_transition.previous_source")
                .map(|source| (commit, raw, source))
        })
        .transpose()
}

fn validate_pilot_history_transition_typed(
    previous: &[PilotAttemptRecord],
    current: &[PilotAttemptRecord],
) -> ReaderResult<&'static str> {
    if current.len() < previous.len() {
        return Err(ReaderError::new(
            "history_transition.pilot: prior attempt history must be prefix-preserved",
        ));
    }
    if current.len() > previous.len() + 1 {
        return Err(ReaderError::new(
            "history_transition.pilot: only one successor may be appended per transition",
        ));
    }
    if current.len() == previous.len() + 1 {
        if current[..previous.len()] != *previous {
            return Err(ReaderError::new(
                "history_transition.pilot: prior attempt history must be prefix-preserved",
            ));
        }
        if current
            .last()
            .is_none_or(|attempt| attempt.attempt_status != "not-run")
        {
            return Err(ReaderError::new(
                "history_transition.pilot: a successor must begin not-run",
            ));
        }
        return Ok("append");
    }
    let differing = previous
        .iter()
        .zip(current)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if differing.is_empty() {
        return Ok("unchanged");
    }
    if previous.is_empty() || differing != [previous.len() - 1] {
        return Err(ReaderError::new(
            "history_transition.pilot: terminal and superseded attempts are immutable",
        ));
    }
    let previous = previous.last().unwrap();
    let current = current.last().unwrap();
    if previous.attempt_status != "not-run"
        || !matches!(current.attempt_status.as_str(), "completed" | "void")
    {
        return Err(ReaderError::new(
            "history_transition.pilot: only the active not-run attempt may become terminal",
        ));
    }
    if previous.attempt_id != current.attempt_id
        || previous.previous_attempt_sha256 != current.previous_attempt_sha256
        || previous.prerequisites != current.prerequisites
        || previous.pre_registration != current.pre_registration
        || previous.tested_snapshot != current.tested_snapshot
    {
        return Err(ReaderError::new(
            "history_transition.pilot: frozen attempt identity and inputs are immutable",
        ));
    }
    Ok("terminal")
}

fn validate_holdout_history_transition_typed(
    previous: &[HoldoutAttempt],
    current: &[HoldoutAttempt],
) -> ReaderResult<&'static str> {
    if current.len() < previous.len() {
        return Err(ReaderError::new(
            "history_transition.holdout: prior attempt history must be prefix-preserved",
        ));
    }
    if current.len() > previous.len() + 1 {
        return Err(ReaderError::new(
            "history_transition.holdout: only one successor may be appended per transition",
        ));
    }
    if current.len() == previous.len() + 1 {
        if current[..previous.len()] != *previous {
            return Err(ReaderError::new(
                "history_transition.holdout: prior attempt history must be prefix-preserved",
            ));
        }
        if current
            .last()
            .is_none_or(|attempt| attempt.attempt_status != "frozen")
        {
            return Err(ReaderError::new(
                "history_transition.holdout: a successor must begin frozen",
            ));
        }
        return Ok("append");
    }
    let differing = previous
        .iter()
        .zip(current)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if differing.is_empty() {
        return Ok("unchanged");
    }
    if previous.is_empty() || differing != [previous.len() - 1] {
        return Err(ReaderError::new(
            "history_transition.holdout: terminal and superseded attempts are immutable",
        ));
    }
    let previous = previous.last().unwrap();
    let current = current.last().unwrap();
    if previous.attempt_status != "frozen"
        || !matches!(current.attempt_status.as_str(), "completed" | "void")
    {
        return Err(ReaderError::new(
            "history_transition.holdout: only the active frozen attempt may become terminal",
        ));
    }
    if previous.attempt_id != current.attempt_id
        || previous.previous_attempt_sha256 != current.previous_attempt_sha256
        || previous.pre_registration != current.pre_registration
        || previous.frozen_rule != current.frozen_rule
        || previous.frozen_ratification != current.frozen_ratification
    {
        return Err(ReaderError::new(
            "history_transition.holdout: frozen attempt identity and inputs are immutable",
        ));
    }
    Ok("terminal")
}

fn validate_history_transition_typed(
    context: &Context,
    source: &ReaderEvidenceSource,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<()> {
    let current = validated_history_snapshot_typed(source, "root")?;
    validate_digest_str(
        &source.history_transition.history_head_sha256,
        "history_transition.history_head_sha256",
        Some(&current.head),
    )?;
    let previous = nearest_previous_reader_evidence_typed(context, source_raw, source_commit)?;
    let Some((expected_commit, previous_raw, previous_source)) = previous else {
        if source.history_transition.previous_source_commit.is_some()
            || source.history_transition.previous_source_sha256.is_some()
            || source
                .history_transition
                .previous_history_head_sha256
                .is_some()
        {
            return Err(ReaderError::new(
                "history_transition: bootstrap source must have null predecessor fields",
            ));
        }
        if !current.pilot_attempts.is_empty()
            || !current.holdout_attempts.is_empty()
            || source.threshold_status != "pending-pilot"
            || source.holdout_status != "not-frozen"
            || source.result != "not-run"
            || source.ratification.is_some()
            || source.pilot.pilot_status != "not-run"
            || source.pilot.control_status != "not-run"
            || source.pilot.active_attempt_id.is_some()
            || source.holdout.active_attempt_id.is_some()
        {
            return Err(ReaderError::new(
                "history_transition: only the initial dormant empty source may bootstrap",
            ));
        }
        return Ok(());
    };
    let declared_commit = source.history_transition.previous_source_commit.as_deref();
    if declared_commit != Some(expected_commit.as_str())
        || !declared_commit.is_some_and(|value| commit_regex().is_match(value))
    {
        return Err(ReaderError::new(
            "history_transition.previous_source_commit must cite the nearest prior JSON-changing commit",
        ));
    }
    let previous_digest = sha256(previous_raw);
    let previous_source_sha = source
        .history_transition
        .previous_source_sha256
        .as_deref()
        .ok_or_else(|| {
            ReaderError::new("history_transition.previous_source_sha256: expected digest")
        })?;
    validate_digest_str(
        previous_source_sha,
        "history_transition.previous_source_sha256",
        Some(&previous_digest),
    )?;
    let previous =
        validated_history_snapshot_typed(&previous_source, "history_transition.previous_source")?;
    validate_digest_str(
        &previous_source.history_transition.history_head_sha256,
        "history_transition.previous_source.history_transition.history_head_sha256",
        Some(&previous.head),
    )?;
    let previous_head = source
        .history_transition
        .previous_history_head_sha256
        .as_deref()
        .ok_or_else(|| {
            ReaderError::new("history_transition.previous_history_head_sha256: expected digest")
        })?;
    validate_digest_str(
        previous_head,
        "history_transition.previous_history_head_sha256",
        Some(&previous.head),
    )?;
    let pilot_action =
        validate_pilot_history_transition_typed(previous.pilot_attempts, current.pilot_attempts)?;
    let holdout_action = validate_holdout_history_transition_typed(
        previous.holdout_attempts,
        current.holdout_attempts,
    )?;
    if pilot_action != "unchanged" && holdout_action != "unchanged" {
        return Err(ReaderError::new(
            "history_transition: pilot and holdout histories may not change in one transition",
        ));
    }
    Ok(())
}

fn validate_history_closure_typed(
    context: &Context,
    source: &ReaderEvidenceSource,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<()> {
    fn unique(values: &mut HashSet<String>, value: &str, path: &str) -> ReaderResult<()> {
        if !values.insert(value.to_owned()) {
            return Err(ReaderError::new(format!(
                "{path}: duplicate across attempt history"
            )));
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn collect_common(
        attempt_id: &str,
        attempt_sha: &str,
        sessions: &[SessionRecord],
        deviations: &[DeviationRecord],
        custody: &[CustodyRecord],
        path: &str,
        global_ids: &mut HashSet<String>,
        attempt_digests: &mut HashSet<String>,
        session_commitments: &mut HashSet<String>,
        custody_external_digests: &mut HashSet<String>,
        custody_record_digests: &mut HashSet<String>,
    ) -> ReaderResult<()> {
        unique(global_ids, attempt_id, &format!("{path}.attempt_id"))?;
        unique(
            attempt_digests,
            attempt_sha,
            &format!("{path}.attempt_sha256"),
        )?;
        for (index, record) in sessions.iter().enumerate() {
            unique(
                session_commitments,
                &record.record_commitment_sha256,
                &format!("{path}.session_records[{index}].record_commitment_sha256"),
            )?;
        }
        for (index, record) in custody.iter().enumerate() {
            unique(
                global_ids,
                &record.attestation_id,
                &format!("{path}.custody_attestations[{index}].attestation_id"),
            )?;
            unique(
                custody_external_digests,
                &record.sha256,
                &format!("{path}.custody_attestations[{index}].sha256"),
            )?;
            unique(
                custody_record_digests,
                &record.record_sha256,
                &format!("{path}.custody_attestations[{index}].record_sha256"),
            )?;
        }
        for (index, deviation) in deviations.iter().enumerate() {
            unique(
                global_ids,
                &deviation.deviation_id,
                &format!("{path}.deviations[{index}].deviation_id"),
            )?;
        }
        Ok(())
    }
    let mut global_ids = HashSet::new();
    let mut attempt_digests = HashSet::new();
    let mut preregistration_digests = HashSet::new();
    let mut receipt_digests = HashSet::new();
    let mut session_commitments = HashSet::new();
    let mut custody_external_digests = HashSet::new();
    let mut custody_record_digests = HashSet::new();
    let mut gate_input_digests = HashSet::new();
    let mut previous_terminal: Option<&str> = None;
    for (index, attempt) in source.pilot.attempts.iter().enumerate() {
        let path = format!("pilot.attempts[{index}]");
        collect_common(
            &attempt.attempt_id,
            &attempt.attempt_sha256,
            &attempt.session_records,
            &attempt.deviations,
            &attempt.custody_attestations,
            &path,
            &mut global_ids,
            &mut attempt_digests,
            &mut session_commitments,
            &mut custody_external_digests,
            &mut custody_record_digests,
        )?;
        let freeze_at = if let Some(registration) = &attempt.pre_registration {
            unique(
                &mut global_ids,
                &registration.study_id,
                &format!("{path}.pre_registration.study_id"),
            )?;
            unique(
                &mut preregistration_digests,
                &registration.pre_registration_sha256,
                &format!("{path}.pre_registration.pre_registration_sha256"),
            )?;
            unique(
                &mut global_ids,
                &registration.freeze_binding.binding_id,
                &format!("{path}.pre_registration.freeze_binding.binding_id"),
            )?;
            unique(
                &mut custody_external_digests,
                &registration.freeze_binding.attestation_sha256,
                &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
            )?;
            Some(registration.freeze_binding.frozen_at.as_str())
        } else {
            None
        };
        if previous_terminal
            .is_some_and(|terminal| freeze_at.is_none_or(|frozen| frozen <= terminal))
        {
            return Err(ReaderError::new(format!(
                "{path}: successor freeze must strictly follow the prior pilot terminal time"
            )));
        }
        match attempt.attempt_status.as_str() {
            "completed" => {
                let receipt = attempt.receipt.as_ref().ok_or_else(|| {
                    ReaderError::new(format!("{path}.receipt: expected completed receipt"))
                })?;
                unique(
                    &mut global_ids,
                    &receipt.receipt_id,
                    &format!("{path}.receipt.receipt_id"),
                )?;
                unique(
                    &mut receipt_digests,
                    &receipt.receipt_sha256,
                    &format!("{path}.receipt.receipt_sha256"),
                )?;
                let packet = attempt.decision_packet.as_ref().ok_or_else(|| {
                    ReaderError::new(format!("{path}.decision_packet: expected packet"))
                })?;
                unique(
                    &mut global_ids,
                    &packet.packet_id,
                    &format!("{path}.decision_packet.packet_id"),
                )?;
                unique(
                    &mut global_ids,
                    &packet.freeze_binding.binding_id,
                    &format!("{path}.decision_packet.freeze_binding.binding_id"),
                )?;
                unique(
                    &mut custody_external_digests,
                    &packet.freeze_binding.attestation_sha256,
                    &format!("{path}.decision_packet.freeze_binding.attestation_sha256"),
                )?;
                previous_terminal = Some(&packet.freeze_binding.frozen_at);
            }
            "void" => previous_terminal = attempt.voided_at.as_deref(),
            _ => {}
        }
    }
    previous_terminal = None;
    for (index, attempt) in source.holdout.attempts.iter().enumerate() {
        let path = format!("holdout.attempts[{index}]");
        collect_common(
            &attempt.attempt_id,
            &attempt.attempt_sha256,
            &attempt.session_records,
            &attempt.deviations,
            &attempt.custody_attestations,
            &path,
            &mut global_ids,
            &mut attempt_digests,
            &mut session_commitments,
            &mut custody_external_digests,
            &mut custody_record_digests,
        )?;
        let registration = &attempt.pre_registration;
        unique(
            &mut global_ids,
            &registration.study_id,
            &format!("{path}.pre_registration.study_id"),
        )?;
        unique(
            &mut preregistration_digests,
            &registration.pre_registration_sha256,
            &format!("{path}.pre_registration.pre_registration_sha256"),
        )?;
        unique(
            &mut global_ids,
            &registration.freeze_binding.binding_id,
            &format!("{path}.pre_registration.freeze_binding.binding_id"),
        )?;
        unique(
            &mut custody_external_digests,
            &registration.freeze_binding.attestation_sha256,
            &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
        )?;
        let freeze_at = registration.freeze_binding.frozen_at.as_str();
        if previous_terminal.is_some_and(|terminal| freeze_at <= terminal) {
            return Err(ReaderError::new(format!(
                "{path}: successor freeze must strictly follow the prior holdout terminal time"
            )));
        }
        if let Some(commitment) = &registration.commitment {
            unique(
                &mut global_ids,
                &commitment.commitment_id,
                &format!("{path}.pre_registration.commitment.commitment_id"),
            )?;
        }
        match attempt.attempt_status.as_str() {
            "completed" => {
                let receipt = attempt.result_receipt.as_ref().ok_or_else(|| {
                    ReaderError::new(format!("{path}.result_receipt: expected receipt"))
                })?;
                unique(
                    &mut global_ids,
                    &receipt.receipt_id,
                    &format!("{path}.result_receipt.receipt_id"),
                )?;
                unique(
                    &mut receipt_digests,
                    &receipt.receipt_sha256,
                    &format!("{path}.result_receipt.receipt_sha256"),
                )?;
                let gate = attempt.gate_admission_receipt.as_ref().ok_or_else(|| {
                    ReaderError::new(format!("{path}.gate_admission_receipt: expected receipt"))
                })?;
                unique(
                    &mut gate_input_digests,
                    &gate.input_sha256,
                    &format!("{path}.gate_admission_receipt.input_sha256"),
                )?;
                unique(
                    &mut receipt_digests,
                    &gate.receipt_sha256,
                    &format!("{path}.gate_admission_receipt.receipt_sha256"),
                )?;
                previous_terminal = Some(&receipt.completed_at);
            }
            "void" => {
                if let Some(receipt) = &attempt.result_receipt {
                    unique(
                        &mut global_ids,
                        &receipt.receipt_id,
                        &format!("{path}.result_receipt.receipt_id"),
                    )?;
                    unique(
                        &mut receipt_digests,
                        &receipt.receipt_sha256,
                        &format!("{path}.result_receipt.receipt_sha256"),
                    )?;
                }
                previous_terminal = attempt.voided_at.as_deref();
            }
            _ => {}
        }
        if let Some(reveal) = &attempt.commitment_reveal {
            previous_terminal = Some(&reveal.revealed_at);
        }
    }
    validate_history_transition_typed(context, source, source_raw, source_commit)
}

#[allow(clippy::too_many_arguments)]
fn validate_candidate_commit_typed(
    env: &ValidationEnv<'_>,
    candidate_commit: &str,
    expected_rule_sha256: &str,
    expected_pilot_attempt_id: &str,
    expected_packet_sha256: &str,
    expected_sensitivity_sha256: &str,
    expected_fixed_protocol_sha256: &str,
) -> ReaderResult<ReaderEvidenceSource> {
    let ancestor = git_output(
        env.context,
        &["merge-base", "--is-ancestor", candidate_commit, "HEAD"],
        "ratification.candidate_commit",
    )?;
    if !ancestor.status.success() {
        return Err(ReaderError::new(
            "ratification.candidate_commit must be an ancestor of the current checkout",
        ));
    }
    let candidate_spec = format!("{candidate_commit}:{DEFAULT_SOURCE}");
    let completed = git_output(
        env.context,
        &["show", &candidate_spec],
        "ratification.candidate_commit",
    )?;
    if !completed.status.success() {
        let detail = String::from_utf8_lossy(&completed.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "ratification.candidate_commit has no candidate source: {detail}"
        )));
    }
    parse_source(&completed.stdout).map_err(|_| {
        ReaderError::new("ratification.candidate_commit contains invalid candidate JSON")
    })?;
    let candidate = typed_reader_source_bytes(&completed.stdout, "ratification candidate source")?;
    let candidate_decision = committed_file_bytes(
        env.context,
        candidate_commit,
        PROTOCOL_DECISION,
        "ratification.candidate_commit.protocol_decision",
    )?;
    let checker_path = STRUCTURAL_CHECKER_REF.split_once("::").unwrap().0;
    let candidate_checker = committed_file_bytes(
        env.context,
        candidate_commit,
        checker_path,
        "ratification.candidate_commit.structural_checker",
    )?;
    let candidate_env = ValidationEnv {
        context: env.context,
        protocol_decision: &candidate_decision,
        verify_live: true,
    };
    validate_protocol_typed(&candidate_env, &candidate)?;
    validate_privacy_typed(&candidate)?;
    if candidate.threshold_status != "candidate" {
        return Err(ReaderError::new(
            "ratification.candidate_commit must record candidate threshold status",
        ));
    }
    if candidate.ratification.is_some() {
        return Err(ReaderError::new(
            "ratification.candidate_commit must precede author ratification",
        ));
    }
    if candidate.holdout_status != "not-frozen" || candidate.result != "not-run" {
        return Err(ReaderError::new(
            "ratification.candidate_commit may contain no holdout result",
        ));
    }
    if candidate.holdout.active_attempt_id.is_some() || !candidate.holdout.attempts.is_empty() {
        return Err(ReaderError::new(
            "ratification.candidate_commit may contain no holdout attempt",
        ));
    }
    let candidate_fixed = canonical_sha_omitting(
        &typed_value(&candidate.protocol, "ratification candidate protocol")?,
        &["decision_sha256"],
    )?;
    if candidate_fixed != expected_fixed_protocol_sha256 {
        return Err(ReaderError::new(
            "ratification.candidate_commit fixed protocol differs from the ratified basis",
        ));
    }
    let pilot = validate_pilot_typed(&candidate_env, &candidate)?;
    if !pilot.valid
        || pilot.active_id != Some(expected_pilot_attempt_id)
        || pilot
            .packet
            .is_none_or(|packet| packet.packet_sha256 != expected_packet_sha256)
        || pilot
            .sensitivity
            .is_none_or(|artifact| artifact.sha256 != expected_sensitivity_sha256)
    {
        return Err(ReaderError::new(
            "ratification.candidate_commit does not contain the same fully validated pilot basis",
        ));
    }
    let known = validate_threshold_rule_typed(&candidate, pilot.valid)?;
    let _ = known;
    let route = validate_route_readiness_typed(
        &candidate_env,
        &candidate,
        pilot.valid,
        Some(&sha256(candidate_checker)),
    )?;
    if candidate.route.reviewer_custody_attestation.is_some()
        || candidate.route.evidence_admission_gate_binding.is_some()
    {
        return Err(ReaderError::new(
            "ratification candidate route must precede reviewer and gate availability bindings",
        ));
    }
    validate_claim_typed(&candidate_env, &candidate, route.status, false)?;
    validate_acceptance_typed(&candidate)?;
    validate_history_closure_typed(
        env.context,
        &candidate,
        &completed.stdout,
        Some(candidate_commit),
    )?;
    let rule = candidate
        .threshold_rule
        .populated("ratification candidate threshold_rule")?;
    if rule.rule_sha256 != expected_rule_sha256 {
        return Err(ReaderError::new(
            "ratification.candidate_commit rule differs from the ratified rule",
        ));
    }
    let active = candidate
        .pilot
        .attempts
        .last()
        .ok_or_else(|| ReaderError::new("ratification candidate has no active pilot"))?;
    if active.attempt_id != expected_pilot_attempt_id
        || active.attempt_status != "completed"
        || active.control_status != "watched-failing"
        || active
            .decision_packet
            .as_ref()
            .is_none_or(|packet| packet.packet_sha256 != expected_packet_sha256)
    {
        return Err(ReaderError::new(
            "ratification candidate active pilot is not valid and completed",
        ));
    }
    Ok(candidate)
}

#[allow(clippy::too_many_arguments)]
fn validate_ratification_payload_typed(
    env: &ValidationEnv<'_>,
    record: &RatificationRecord,
    path: &str,
    rule: &ThresholdRule,
    expected_pilot_attempt_id: &str,
    expected_packet_sha256: &str,
    expected_sensitivity_sha256: &str,
    fixed_protocol_sha256: &str,
) -> ReaderResult<()> {
    validate_id_str(&record.ruling_id, &format!("{path}.ruling_id"))?;
    validate_id_str(
        &record.pilot_attempt_id,
        &format!("{path}.pilot_attempt_id"),
    )?;
    if record.pilot_attempt_id != expected_pilot_attempt_id {
        return Err(ReaderError::new(format!(
            "{path}.pilot_attempt_id: does not bind the expected pilot"
        )));
    }
    validate_date_str(&record.ratified_date, &format!("{path}.ratified_date"))?;
    validate_text_str(
        &record.candidate_commit,
        &format!("{path}.candidate_commit"),
    )?;
    if !commit_regex().is_match(&record.candidate_commit) {
        return Err(ReaderError::new(format!(
            "{path}.candidate_commit must be a full lowercase commit digest"
        )));
    }
    for (key, value) in [
        ("author_statement", &record.author_statement),
        ("question_answered", &record.question_answered),
        ("rationale", &record.rationale),
    ] {
        validate_text_str(value, &format!("{path}.{key}"))?;
    }
    validate_digest_str(
        &record.pilot_packet_sha256,
        &format!("{path}.pilot_packet_sha256"),
        Some(expected_packet_sha256),
    )?;
    validate_digest_str(
        &record.sensitivity_brief_sha256,
        &format!("{path}.sensitivity_brief_sha256"),
        Some(expected_sensitivity_sha256),
    )?;
    validate_digest_str(
        &record.rule_sha256,
        &format!("{path}.rule_sha256"),
        Some(&rule.rule_sha256),
    )?;
    let candidate = validate_candidate_commit_typed(
        env,
        &record.candidate_commit,
        &rule.rule_sha256,
        &record.pilot_attempt_id,
        &record.pilot_packet_sha256,
        &record.sensitivity_brief_sha256,
        fixed_protocol_sha256,
    )?;
    let candidate_packet = candidate
        .pilot
        .attempts
        .last()
        .and_then(|attempt| attempt.decision_packet.as_ref())
        .ok_or_else(|| ReaderError::new(format!("{path}: candidate pilot packet missing")))?;
    if record.ratified_date < candidate_packet.frozen_date {
        return Err(ReaderError::new(format!(
            "{path}: ratification must follow its frozen pilot decision packet"
        )));
    }
    let decision_ref =
        validate_repo_reference_str(env, &record.decision_ref, &format!("{path}.decision_ref"))?;
    if !decision_ref.starts_with(&format!("{PROTOCOL_DECISION}::")) {
        return Err(ReaderError::new(format!(
            "{path}.decision_ref must cite the controlling decision record"
        )));
    }
    if !decision_ref.contains(&record.ruling_id) {
        return Err(ReaderError::new(format!(
            "{path}.decision_ref must cite the exact ruling anchor"
        )));
    }
    if !record.no_holdout_evidence_attestation {
        return Err(ReaderError::new(format!(
            "{path} must attest no holdout evidence existed or was inspected"
        )));
    }
    validate_digest_str(
        &record.ratification_sha256,
        &format!("{path}.ratification_sha256"),
        Some(&typed_canonical_sha(
            record,
            path,
            Some("ratification_sha256"),
        )?),
    )?;
    Ok(())
}

fn validate_ratification_typed(
    env: &ValidationEnv<'_>,
    source: &ReaderEvidenceSource,
    pilot: &PilotValidationTyped<'_>,
) -> ReaderResult<()> {
    if matches!(
        source.threshold_status.as_str(),
        "candidate" | "pending-pilot"
    ) {
        if source.ratification.is_some() {
            return Err(ReaderError::new(format!(
                "{} threshold may not carry ratification",
                source.threshold_status
            )));
        }
        return Ok(());
    }
    let record = source
        .ratification
        .as_ref()
        .ok_or_else(|| ReaderError::new("author-ratified threshold requires ratification"))?;
    let active_id = pilot
        .active_id
        .ok_or_else(|| ReaderError::new("ratification must bind the active valid pilot attempt"))?;
    let packet = pilot.packet.ok_or_else(|| {
        ReaderError::new("ratification requires the frozen pilot packet and sensitivity brief")
    })?;
    let sensitivity = pilot.sensitivity.ok_or_else(|| {
        ReaderError::new("ratification requires the frozen pilot packet and sensitivity brief")
    })?;
    let rule = source.threshold_rule.populated("threshold_rule")?;
    let fixed_protocol_sha256 = canonical_sha_omitting(
        &typed_value(&source.protocol, "protocol")?,
        &["decision_sha256"],
    )?;
    validate_ratification_payload_typed(
        env,
        record,
        "ratification",
        &rule,
        active_id,
        &packet.packet_sha256,
        &sensitivity.sha256,
        &fixed_protocol_sha256,
    )?;
    if record.ratified_date < packet.frozen_date {
        return Err(ReaderError::new(
            "ratification must follow the frozen pilot decision packet",
        ));
    }
    Ok(())
}

fn validate_frozen_ratification_typed(
    env: &ValidationEnv<'_>,
    record: &RatificationRecord,
    path: &str,
    rule: &ThresholdRule,
    fixed_protocol_sha256: &str,
) -> ReaderResult<()> {
    validate_ratification_payload_typed(
        env,
        record,
        path,
        rule,
        &record.pilot_attempt_id,
        &record.pilot_packet_sha256,
        &record.sensitivity_brief_sha256,
        fixed_protocol_sha256,
    )
}

fn validate_holdout_typed(
    env: &ValidationEnv<'_>,
    source: &ReaderEvidenceSource,
    known_misconceptions: &BTreeSet<String>,
    route: &RouteValidationTyped<'_>,
) -> ReaderResult<bool> {
    if !["not-frozen", "frozen", "completed", "void"].contains(&source.holdout_status.as_str()) {
        return Err(ReaderError::new("holdout_status: invalid state"));
    }
    if !["not-run", "pass", "fail", "not-evaluable"].contains(&source.result.as_str()) {
        return Err(ReaderError::new("result: invalid state"));
    }
    if source.holdout.attempts.is_empty() {
        if source.holdout.active_attempt_id.is_some() {
            return Err(ReaderError::new(
                "holdout.active_attempt_id requires an attempt",
            ));
        }
        if source.holdout_status != "not-frozen" || source.result != "not-run" {
            return Err(ReaderError::new(
                "empty holdout history must remain not-frozen/not-run",
            ));
        }
        return Ok(false);
    }
    if source.threshold_status != "author-ratified" {
        return Err(ReaderError::new(
            "every holdout attempt requires an author-ratified rule",
        ));
    }
    let current_ratification = source
        .ratification
        .as_ref()
        .ok_or_else(|| ReaderError::new("ratification: expected author ratification"))?;
    validate_digest_str(
        &current_ratification.ratification_sha256,
        "ratification.ratification_sha256",
        None,
    )?;
    let current_rule = source.threshold_rule.populated("threshold_rule")?;
    let fixed_protocol_sha256 = canonical_sha_omitting(
        &typed_value(&source.protocol, "protocol")?,
        &["decision_sha256"],
    )?;
    let pilot_attempt_sha256s = source
        .pilot
        .attempts
        .iter()
        .map(|attempt| attempt.attempt_sha256.as_str())
        .collect::<Vec<_>>();
    let mut seen_ids = HashSet::new();
    let mut previous_sha256 = None;
    let mut holdout_attempt_sha256s = Vec::new();
    let mut active_id = "";
    let mut active_status = "";
    let mut latest_completed_result = "not-run";
    let mut valid_active_pass = false;
    for (index, attempt) in source.holdout.attempts.iter().enumerate() {
        let path = format!("holdout.attempts[{index}]");
        let active = index + 1 == source.holdout.attempts.len();
        validate_id_str(&attempt.attempt_id, &format!("{path}.attempt_id"))?;
        if !seen_ids.insert(attempt.attempt_id.as_str()) {
            return Err(ReaderError::new(format!("{path}.attempt_id: duplicate")));
        }
        match (index, attempt.previous_attempt_sha256.as_deref()) {
            (0, None) => {}
            (0, Some(_)) => {
                return Err(ReaderError::new(format!(
                    "{path}.previous_attempt_sha256: first attempt must be null"
                )));
            }
            (_, Some(declared)) => {
                validate_digest_str(
                    declared,
                    &format!("{path}.previous_attempt_sha256"),
                    previous_sha256,
                )?;
            }
            (_, None) => {
                return Err(ReaderError::new(format!(
                    "{path}.previous_attempt_sha256: expected prior digest"
                )));
            }
        }
        if !["frozen", "completed", "void"].contains(&attempt.attempt_status.as_str()) {
            return Err(ReaderError::new(format!(
                "{path}.attempt_status: invalid state"
            )));
        }
        if !["not-run", "pass", "fail", "not-evaluable"].contains(&attempt.attempt_result.as_str())
        {
            return Err(ReaderError::new(format!(
                "{path}.attempt_result: invalid state"
            )));
        }
        if !active && attempt.attempt_status == "frozen" {
            return Err(ReaderError::new(format!(
                "{path}: a superseded holdout attempt cannot remain frozen"
            )));
        }
        if attempt.attempt_status == "void" {
            let code = attempt.void_reason_code.as_deref().ok_or_else(|| {
                ReaderError::new(format!("{path}.void_reason_code: expected reason"))
            })?;
            validate_id_str(code, &format!("{path}.void_reason_code"))?;
            if !code.starts_with("RE-VOID-") {
                return Err(ReaderError::new(format!(
                    "{path}.void_reason_code: expected a closed RE-VOID-* code"
                )));
            }
            validate_timestamp_str(
                attempt.voided_at.as_deref().ok_or_else(|| {
                    ReaderError::new(format!("{path}.voided_at: expected terminal time"))
                })?,
                &format!("{path}.voided_at"),
            )?;
        } else if attempt.void_reason_code.is_some() || attempt.voided_at.is_some() {
            return Err(ReaderError::new(format!(
                "{path}: only a void attempt may carry a reason or terminal time"
            )));
        }
        let frozen_misconceptions = validate_populated_threshold_rule_typed(&attempt.frozen_rule)?;
        validate_frozen_ratification_typed(
            env,
            &attempt.frozen_ratification,
            &format!("{path}.frozen_ratification"),
            &attempt.frozen_rule,
            &fixed_protocol_sha256,
        )?;
        let prior_head = history_head_sha256(
            pilot_attempt_sha256s.iter().copied(),
            holdout_attempt_sha256s.iter().copied(),
        );
        validate_holdout_pre_registration_typed(
            env,
            &attempt.pre_registration,
            &format!("{path}.pre_registration"),
            active && attempt.attempt_status != "void",
            &fixed_protocol_sha256,
            (active && attempt.attempt_status != "void").then_some(route.checker_sha256),
            previous_sha256,
            Some(&prior_head),
            true,
        )?;
        validate_digest_str(
            &attempt.pre_registration.rule_sha256,
            &format!("{path}.pre_registration.rule_sha256"),
            Some(&attempt.frozen_rule.rule_sha256),
        )?;
        validate_digest_str(
            &attempt.pre_registration.ratification_sha256,
            &format!("{path}.pre_registration.ratification_sha256"),
            Some(&attempt.frozen_ratification.ratification_sha256),
        )?;
        validate_sessions_typed(
            &attempt.session_records,
            &format!("{path}.session_records"),
            Some(&attempt.pre_registration.study_id),
            Some(&frozen_misconceptions),
        )?;
        let deviations =
            validate_deviations_typed(&attempt.deviations, &format!("{path}.deviations"))?;
        let custody = validate_custody_typed(
            env,
            &attempt.custody_attestations,
            &format!("{path}.custody_attestations"),
        )?;
        validate_record_links_typed(
            &attempt.session_records,
            &deviations,
            &custody,
            &format!("{path}.record_links"),
            Some(&attempt.pre_registration.study_id),
            attempt.pre_registration.commitment.as_ref(),
        )?;
        if let Some(receipt) = &attempt.result_receipt {
            validate_result_receipt_typed(
                receipt,
                &format!("{path}.result_receipt"),
                &attempt.pre_registration,
                &attempt.frozen_rule,
                &attempt.session_records,
                &attempt.deviations,
                &attempt.custody_attestations,
            )?;
        }
        let ran = !attempt.session_records.is_empty() || attempt.result_receipt.is_some();
        let terminal_at = if attempt.attempt_status == "completed" {
            attempt
                .result_receipt
                .as_ref()
                .map(|receipt| receipt.completed_at.as_str())
        } else if attempt.attempt_status == "void" {
            attempt.voided_at.as_deref()
        } else {
            None
        };
        validate_commitment_reveal_typed(
            env,
            attempt.commitment_reveal.as_ref(),
            &format!("{path}.commitment_reveal"),
            attempt.pre_registration.commitment.as_ref(),
            &custody,
            &attempt.attempt_status,
            matches!(attempt.attempt_status.as_str(), "completed" | "void"),
            active,
            terminal_at,
        )?;
        let rule_match = attempt.frozen_rule.rule_sha256 == current_rule.rule_sha256
            && attempt.pre_registration.rule_sha256 == current_rule.rule_sha256;
        let gate_match = route.gate_sha256.is_some()
            && route.gate_sha256 == Some(attempt.pre_registration.evidence_gate_sha256.as_str());
        let structural_match =
            attempt.pre_registration.structural_checker_sha256 == route.checker_sha256;
        let ratification_match = attempt.pre_registration.ratification_sha256
            == current_ratification.ratification_sha256
            && attempt.frozen_ratification.ratification_sha256
                == current_ratification.ratification_sha256;
        let current_binding = rule_match && gate_match && structural_match && ratification_match;
        if active && attempt.attempt_status != "void" && !current_binding {
            return Err(ReaderError::new(format!(
                "{path}: active holdout must bind the current rule, ratification, gate, and checker"
            )));
        }
        if attempt.pre_registration.registered_date < attempt.frozen_ratification.ratified_date {
            return Err(ReaderError::new(format!(
                "{path}: pre-registration cannot precede its frozen ratification"
            )));
        }
        let voiding_deviation = deviations
            .values()
            .any(|item| item.impact == "holdout-void");
        let freshness_records = custody
            .values()
            .filter(|item| item.scope == "study-freshness")
            .copied()
            .collect::<Vec<_>>();
        if ran && freshness_records.len() != 1 {
            return Err(ReaderError::new(format!(
                "{path}: a run holdout requires exactly one freshness attestation"
            )));
        }
        let freshness_bound =
            freshness_records.len() == 1 && freshness_records[0].freshness_attested;
        let freeze_at = attempt.pre_registration.freeze_binding.frozen_at.as_str();
        match attempt.attempt_status.as_str() {
            "frozen" => validate_frozen_holdout_payload_typed(attempt, &path)?,
            "completed" => {
                if active && route.status != "available" {
                    return Err(ReaderError::new(format!(
                        "{path}: the reader route must be available before the active holdout runs"
                    )));
                }
                if voiding_deviation {
                    return Err(ReaderError::new(format!(
                        "{path}: a voiding deviation cannot remain completed"
                    )));
                }
                let receipt = attempt.result_receipt.as_ref().ok_or_else(|| {
                    ReaderError::new(format!(
                        "{path}: completed holdout requires coded sessions and a receipt"
                    ))
                })?;
                if attempt.session_records.is_empty() {
                    return Err(ReaderError::new(format!(
                        "{path}: completed holdout requires coded sessions and a receipt"
                    )));
                }
                if receipt.completed_at.as_str() <= freeze_at {
                    return Err(ReaderError::new(format!(
                        "{path}: completion must strictly follow the frozen preregistration"
                    )));
                }
                if &receipt.completed_at[..10] < attempt.pre_registration.registered_date.as_str() {
                    return Err(ReaderError::new(format!(
                        "{path}: completion cannot precede pre-registration"
                    )));
                }
                if receipt.verdict != attempt.attempt_result {
                    return Err(ReaderError::new(format!(
                        "{path}.attempt_result: must equal the recomputed receipt verdict"
                    )));
                }
                if receipt.protocol_validity == "valid" && !freshness_bound {
                    return Err(ReaderError::new(format!(
                        "{path}: a protocol-valid result requires bound freshness custody"
                    )));
                }
                let gate_input = build_gate_input(
                    attempt.attempt_id.clone(),
                    attempt.frozen_rule.clone(),
                    attempt.frozen_ratification.clone(),
                    attempt.pre_registration.clone(),
                    attempt.session_records.clone(),
                    attempt.deviations.clone(),
                    attempt.custody_attestations.clone(),
                    receipt.clone(),
                    attempt.commitment_reveal.clone(),
                );
                let expected_decision = if receipt.protocol_validity == "valid"
                    && attempt.attempt_result == "pass"
                    && freshness_bound
                {
                    "admit"
                } else {
                    "reject"
                };
                let gate_receipt = attempt.gate_admission_receipt.as_ref().ok_or_else(|| {
                    ReaderError::new(format!(
                        "{path}.gate_admission_receipt: completed holdout requires receipt"
                    ))
                })?;
                validate_gate_admission_receipt_typed(
                    env,
                    gate_receipt,
                    &format!("{path}.gate_admission_receipt"),
                    &gate_input,
                    expected_decision,
                    active,
                )?;
                latest_completed_result = &attempt.attempt_result;
                valid_active_pass = active && current_binding && gate_receipt.decision == "admit";
            }
            "void" => {
                let voided_at = attempt.voided_at.as_deref().expect("validated void time");
                if voided_at <= freeze_at {
                    return Err(ReaderError::new(format!(
                        "{path}: void time must strictly follow the frozen preregistration"
                    )));
                }
                if attempt.gate_admission_receipt.is_some() {
                    return Err(ReaderError::new(format!(
                        "{path}: a void holdout may not carry an admission receipt"
                    )));
                }
                if let Some(receipt) = &attempt.result_receipt {
                    if receipt.protocol_validity != "invalid"
                        || receipt.verdict != "not-evaluable"
                        || attempt.attempt_result != "not-evaluable"
                    {
                        return Err(ReaderError::new(format!(
                            "{path}: a run void must preserve an invalid, not-evaluable receipt"
                        )));
                    }
                    if receipt.completed_at.as_str() > voided_at {
                        return Err(ReaderError::new(format!(
                            "{path}: receipt completion cannot follow the void time"
                        )));
                    }
                } else if attempt.attempt_result != "not-run" {
                    return Err(ReaderError::new(format!(
                        "{path}: a pre-result void attempt must remain not-run"
                    )));
                }
                if ran && !voiding_deviation {
                    return Err(ReaderError::new(format!(
                        "{path}: a run void requires a custody-linked holdout-void deviation"
                    )));
                }
            }
            _ => unreachable!(),
        }
        validate_digest_str(
            &attempt.attempt_sha256,
            &format!("{path}.attempt_sha256"),
            Some(&typed_canonical_sha(
                attempt,
                &path,
                Some("attempt_sha256"),
            )?),
        )?;
        previous_sha256 = Some(attempt.attempt_sha256.as_str());
        holdout_attempt_sha256s.push(attempt.attempt_sha256.as_str());
        active_id = &attempt.attempt_id;
        active_status = &attempt.attempt_status;
    }
    if source.holdout.active_attempt_id.as_deref() != Some(active_id) {
        return Err(ReaderError::new(
            "holdout.active_attempt_id must identify the final append-only attempt",
        ));
    }
    if source.holdout_status != active_status {
        return Err(ReaderError::new(
            "holdout_status must equal the active attempt lifecycle",
        ));
    }
    if source.result != latest_completed_result {
        return Err(ReaderError::new(
            "result must preserve the latest completed non-void outcome",
        ));
    }
    if known_misconceptions
        != &current_rule
            .misconceptions
            .iter()
            .map(|item| item.misconception_id.clone())
            .collect()
    {
        return Err(ReaderError::new(
            "current misconception registry changed during holdout validation",
        ));
    }
    Ok(valid_active_pass)
}

fn validate_source_typed(
    context: &Context,
    source: &ReaderEvidenceSource,
    source_raw: &[u8],
    protocol_decision: &[u8],
) -> ReaderResult<Validation> {
    let env = ValidationEnv {
        context,
        protocol_decision,
        verify_live: true,
    };
    validate_protocol_typed(&env, source)?;
    let pilot = validate_pilot_typed(&env, source)?;
    validate_privacy_typed(source)?;
    let known_misconceptions = validate_threshold_rule_typed(source, pilot.valid)?;
    validate_ratification_typed(&env, source, &pilot)?;
    let route = validate_route_readiness_typed(&env, source, pilot.valid, None)?;
    let valid_holdout_pass = validate_holdout_typed(&env, source, &known_misconceptions, &route)?;
    validate_history_closure_typed(context, source, source_raw, None)?;
    validate_claim_typed(&env, source, route.status, valid_holdout_pass)?;
    validate_acceptance_typed(source)?;
    Ok(Validation {
        valid_pilot: pilot.valid,
        valid_holdout_pass,
    })
}

fn validate_pilot_pre_registration_typed(
    env: &ValidationEnv<'_>,
    registration: &PilotPreRegistration,
    path: &str,
    fixed_protocol_sha256: &str,
    expected_predecessor_attempt_sha256: Option<&str>,
    expected_prior_history_head_sha256: &str,
) -> ReaderResult<()> {
    validate_id_str(&registration.study_id, &format!("{path}.study_id"))?;
    validate_date_str(
        &registration.registered_date,
        &format!("{path}.registered_date"),
    )?;
    validate_history_binding_typed(
        registration.predecessor_attempt_sha256.as_deref(),
        &registration.prior_history_head_sha256,
        path,
        expected_predecessor_attempt_sha256,
        Some(expected_prior_history_head_sha256),
        true,
    )?;
    validate_digest_str(
        &registration.fixed_protocol_sha256,
        &format!("{path}.fixed_protocol_sha256"),
        Some(fixed_protocol_sha256),
    )?;
    for (key, artifact) in [
        ("protocol", &registration.protocol),
        ("instrument", &registration.instrument),
        ("rubric", &registration.rubric),
        ("sample_rule", &registration.sample_rule),
        ("disclosure_set", &registration.disclosure_set),
        ("ethics_terms", &registration.ethics_terms),
        ("provisional_rule", &registration.provisional_rule),
    ] {
        validate_artifact_typed(env, artifact, &format!("{path}.{key}"), true)?;
    }
    validate_freeze_binding_typed(
        env,
        &registration.freeze_binding,
        registration,
        &format!("{path}.freeze_binding"),
        "pre_registration_sha256",
        HistoricalPayloadKind::PilotPreRegistration,
    )?;
    if &registration.freeze_binding.frozen_at[..10] < registration.registered_date.as_str() {
        return Err(ReaderError::new(format!(
            "{path}.freeze_binding: external freeze cannot precede registration"
        )));
    }
    validate_digest_str(
        &registration.pre_registration_sha256,
        &format!("{path}.pre_registration_sha256"),
        Some(&typed_canonical_sha(
            registration,
            path,
            Some("pre_registration_sha256"),
        )?),
    )?;
    Ok(())
}

fn validate_decision_packet_typed(
    env: &ValidationEnv<'_>,
    packet: &DecisionPacket,
    path: &str,
) -> ReaderResult<()> {
    validate_id_str(&packet.packet_id, &format!("{path}.packet_id"))?;
    validate_date_str(&packet.frozen_date, &format!("{path}.frozen_date"))?;
    validate_digest_str(
        &packet.pilot_pre_registration_sha256,
        &format!("{path}.pilot_pre_registration_sha256"),
        None,
    )?;
    validate_digest_str(
        &packet.tested_snapshot_sha256,
        &format!("{path}.tested_snapshot_sha256"),
        None,
    )?;
    for (key, artifact) in [
        ("coded_evidence", &packet.coded_evidence),
        ("exclusions", &packet.exclusions),
        ("coder_disagreements", &packet.coder_disagreements),
        ("deviations", &packet.deviations),
        ("revised_instrument", &packet.revised_instrument),
        ("control_transcript", &packet.control_transcript),
    ] {
        validate_artifact_typed(env, artifact, &format!("{path}.{key}"), true)?;
    }
    validate_freeze_binding_typed(
        env,
        &packet.freeze_binding,
        packet,
        &format!("{path}.freeze_binding"),
        "packet_sha256",
        HistoricalPayloadKind::PilotDecisionPacket,
    )?;
    if packet.freeze_binding.frozen_at[..10] != packet.frozen_date {
        return Err(ReaderError::new(format!(
            "{path}.frozen_date must equal the freeze binding calendar date"
        )));
    }
    validate_digest_str(
        &packet.packet_sha256,
        &format!("{path}.packet_sha256"),
        Some(&typed_canonical_sha(packet, path, Some("packet_sha256"))?),
    )?;
    Ok(())
}

fn validate_pilot_receipt_typed(
    receipt: &PilotReceipt,
    path: &str,
    registration: &PilotPreRegistration,
    snapshot: &Artifact,
    sessions: &[SessionRecord],
    deviations: &[DeviationRecord],
    custody: &[CustodyRecord],
    packet: &DecisionPacket,
) -> ReaderResult<()> {
    validate_id_str(&receipt.receipt_id, &format!("{path}.receipt_id"))?;
    validate_timestamp_str(&receipt.completed_at, &format!("{path}.completed_at"))?;
    validate_id_str(&receipt.study_id, &format!("{path}.study_id"))?;
    if receipt.study_id != registration.study_id {
        return Err(ReaderError::new(format!(
            "{path}.study_id: does not match pre-registration"
        )));
    }
    if !["valid", "invalid"].contains(&receipt.protocol_validity.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.protocol_validity: invalid state"
        )));
    }
    let digest_links = [
        (
            "pre_registration_sha256",
            &receipt.pre_registration_sha256,
            registration.pre_registration_sha256.clone(),
        ),
        (
            "snapshot_sha256",
            &receipt.snapshot_sha256,
            snapshot.sha256.clone(),
        ),
        (
            "instrument_sha256",
            &receipt.instrument_sha256,
            registration.instrument.sha256.clone(),
        ),
        (
            "rubric_sha256",
            &receipt.rubric_sha256,
            registration.rubric.sha256.clone(),
        ),
        (
            "coded_evidence_sha256",
            &receipt.coded_evidence_sha256,
            packet.coded_evidence.sha256.clone(),
        ),
        (
            "coded_records_sha256",
            &receipt.coded_records_sha256,
            typed_canonical_sha(&sessions, path, None)?,
        ),
        (
            "deviations_sha256",
            &receipt.deviations_sha256,
            typed_canonical_sha(&deviations, path, None)?,
        ),
        (
            "control_transcript_sha256",
            &receipt.control_transcript_sha256,
            packet.control_transcript.sha256.clone(),
        ),
        (
            "decision_packet_sha256",
            &receipt.decision_packet_sha256,
            packet.packet_sha256.clone(),
        ),
        (
            "custody_records_sha256",
            &receipt.custody_records_sha256,
            typed_canonical_sha(&custody, path, None)?,
        ),
    ];
    for (key, declared, expected) in digest_links {
        validate_digest_str(declared, &format!("{path}.{key}"), Some(&expected))?;
    }
    let classifications = sessions
        .iter()
        .map(|record| {
            serde_json::json!({
                "record_commitment_sha256": record.record_commitment_sha256,
                "admissibility": record.admissibility,
            })
        })
        .collect::<Vec<_>>();
    validate_digest_str(
        &receipt.session_classification_sha256,
        &format!("{path}.session_classification_sha256"),
        Some(&canonical_sha(&Value::Array(classifications), None)?),
    )?;
    validate_digest_str(&receipt.coder_sha256, &format!("{path}.coder_sha256"), None)?;
    if receipt.custody_attestation_sha256s
        != custody
            .iter()
            .map(|record| record.sha256.clone())
            .collect::<Vec<_>>()
    {
        return Err(ReaderError::new(format!(
            "{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )));
    }
    for (index, value) in receipt.custody_attestation_sha256s.iter().enumerate() {
        validate_digest_str(
            value,
            &format!("{path}.custody_attestation_sha256s[{index}]"),
            None,
        )?;
    }
    validate_digest_str(
        &receipt.receipt_sha256,
        &format!("{path}.receipt_sha256"),
        Some(&typed_canonical_sha(receipt, path, Some("receipt_sha256"))?),
    )?;
    Ok(())
}

struct PilotAttemptTyped<'a> {
    attempt_id: &'a str,
    status: &'a str,
    packet: Option<&'a DecisionPacket>,
    sensitivity: Option<&'a Artifact>,
    receipt: Option<&'a PilotReceipt>,
    control: &'a str,
    attempt_sha: &'a str,
}

fn validate_pilot_attempt_typed<'a>(
    env: &ValidationEnv<'_>,
    attempt: &'a PilotAttemptRecord,
    path: &str,
    source: &ReaderEvidenceSource,
    previous_sha256: Option<&str>,
    prior_history_head_sha256: &str,
    first: bool,
    active: bool,
) -> ReaderResult<PilotAttemptTyped<'a>> {
    validate_id_str(&attempt.attempt_id, &format!("{path}.attempt_id"))?;
    match (first, attempt.previous_attempt_sha256.as_deref()) {
        (true, Some(_)) => {
            return Err(ReaderError::new(format!(
                "{path}.previous_attempt_sha256: first attempt must be null"
            )));
        }
        (false, Some(declared)) => {
            validate_digest_str(
                declared,
                &format!("{path}.previous_attempt_sha256"),
                previous_sha256,
            )?;
        }
        (false, None) => {
            return Err(ReaderError::new(format!(
                "{path}.previous_attempt_sha256: expected prior digest"
            )));
        }
        (true, None) => {}
    }
    if !["not-run", "completed", "void"].contains(&attempt.attempt_status.as_str()) {
        return Err(ReaderError::new(format!(
            "{path}.attempt_status: invalid state"
        )));
    }
    if ![
        "not-run",
        "watched-failing",
        "failed-to-fail",
        "indeterminate",
    ]
    .contains(&attempt.control_status.as_str())
    {
        return Err(ReaderError::new(format!(
            "{path}.control_status: invalid state"
        )));
    }
    if !active && attempt.attempt_status == "not-run" {
        return Err(ReaderError::new(format!(
            "{path}: only the active final pilot attempt may remain not-run"
        )));
    }
    if let Some(reason) = &attempt.void_reason_code {
        validate_id_str(reason, &format!("{path}.void_reason_code"))?;
    }
    if attempt.attempt_status == "void" {
        let voided_at = attempt.voided_at.as_ref().ok_or_else(|| {
            ReaderError::new(format!(
                "{path}.voided_at: void attempt requires terminal time"
            ))
        })?;
        validate_timestamp_str(voided_at, &format!("{path}.voided_at"))?;
    } else if attempt.voided_at.is_some() {
        return Err(ReaderError::new(format!(
            "{path}.voided_at: only a void pilot attempt may carry a terminal time"
        )));
    }
    for (key, reference) in [
        ("readers_map_ref", &attempt.prerequisites.readers_map_ref),
        ("glossary_ref", &attempt.prerequisites.glossary_ref),
        (
            "accessible_navigation_ref",
            &attempt.prerequisites.accessible_navigation_ref,
        ),
    ] {
        if let Some(reference) = reference {
            validate_repo_reference_str(env, reference, &format!("{path}.prerequisites.{key}"))?;
        }
    }
    let fixed_protocol_sha256 = canonical_sha_omitting(
        &typed_value(&source.protocol, "protocol")?,
        &["decision_sha256"],
    )?;
    if let Some(registration) = &attempt.pre_registration {
        validate_pilot_pre_registration_typed(
            env,
            registration,
            &format!("{path}.pre_registration"),
            &fixed_protocol_sha256,
            previous_sha256,
            prior_history_head_sha256,
        )?;
    }
    if let Some(snapshot) = &attempt.tested_snapshot {
        validate_artifact_typed(env, snapshot, &format!("{path}.tested_snapshot"), true)?;
    }
    let study_id = attempt
        .pre_registration
        .as_ref()
        .map(|registration| registration.study_id.as_str());
    validate_sessions_typed(
        &attempt.session_records,
        &format!("{path}.session_records"),
        study_id,
        None,
    )?;
    let deviations = validate_deviations_typed(&attempt.deviations, &format!("{path}.deviations"))?;
    let custody = validate_custody_typed(
        env,
        &attempt.custody_attestations,
        &format!("{path}.custody_attestations"),
    )?;
    validate_record_links_typed(
        &attempt.session_records,
        &deviations,
        &custody,
        &format!("{path}.record_links"),
        study_id,
        None,
    )?;
    let pilot_run_evidence = !attempt.session_records.is_empty()
        || !attempt.deviations.is_empty()
        || !attempt.custody_attestations.is_empty()
        || attempt.receipt.is_some()
        || attempt.decision_packet.is_some()
        || attempt.sensitivity_brief.is_some();
    validate_pilot_run_freshness_typed(
        &custody,
        &format!("{path}.custody_attestations"),
        study_id,
        pilot_run_evidence,
        attempt.attempt_status == "completed",
    )?;
    if let Some(packet) = &attempt.decision_packet {
        validate_decision_packet_typed(env, packet, &format!("{path}.decision_packet"))?;
        if let Some(registration) = &attempt.pre_registration {
            validate_digest_str(
                &packet.pilot_pre_registration_sha256,
                &format!("{path}.decision_packet.pilot_pre_registration_sha256"),
                Some(&registration.pre_registration_sha256),
            )?;
        }
        if let Some(snapshot) = &attempt.tested_snapshot {
            validate_digest_str(
                &packet.tested_snapshot_sha256,
                &format!("{path}.decision_packet.tested_snapshot_sha256"),
                Some(&snapshot.sha256),
            )?;
        }
    }
    if let Some(sensitivity) = &attempt.sensitivity_brief {
        validate_artifact_typed(env, sensitivity, &format!("{path}.sensitivity_brief"), true)?;
    }
    if let Some(receipt) = &attempt.receipt {
        let (Some(registration), Some(snapshot), Some(packet)) = (
            &attempt.pre_registration,
            &attempt.tested_snapshot,
            &attempt.decision_packet,
        ) else {
            return Err(ReaderError::new(format!(
                "{path}.receipt requires pre-registration, snapshot, and packet"
            )));
        };
        validate_pilot_receipt_typed(
            receipt,
            &format!("{path}.receipt"),
            registration,
            snapshot,
            &attempt.session_records,
            &attempt.deviations,
            &attempt.custody_attestations,
            packet,
        )?;
    }
    let payload_present = attempt.pre_registration.is_some()
        || attempt.tested_snapshot.is_some()
        || attempt.receipt.is_some()
        || attempt.decision_packet.is_some()
        || attempt.sensitivity_brief.is_some()
        || !attempt.session_records.is_empty()
        || !attempt.deviations.is_empty()
        || !attempt.custody_attestations.is_empty();
    let prerequisites_complete = attempt.prerequisites.readers_map_ref.is_some()
        && attempt.prerequisites.glossary_ref.is_some()
        && attempt.prerequisites.accessible_navigation_ref.is_some();
    match attempt.attempt_status.as_str() {
        "not-run" => {
            if attempt.void_reason_code.is_some() || attempt.control_status != "not-run" {
                return Err(ReaderError::new(format!(
                    "{path}: pre-run pilot cannot carry a void reason or control result"
                )));
            }
            if attempt.pre_registration.is_none() || attempt.tested_snapshot.is_none() {
                return Err(ReaderError::new(format!(
                    "{path}: recorded pre-run pilot requires frozen preregistration and snapshot"
                )));
            }
            if !prerequisites_complete {
                return Err(ReaderError::new(format!(
                    "{path}: recorded pre-run pilot requires all prerequisites"
                )));
            }
            if !attempt.session_records.is_empty()
                || !attempt.deviations.is_empty()
                || !attempt.custody_attestations.is_empty()
                || attempt.receipt.is_some()
                || attempt.decision_packet.is_some()
                || attempt.sensitivity_brief.is_some()
            {
                return Err(ReaderError::new(format!(
                    "{path}: pre-run pilot cannot carry as-run or decision evidence"
                )));
            }
        }
        "completed" => {
            if attempt.void_reason_code.is_some() {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot cannot carry a void reason"
                )));
            }
            let (Some(registration), Some(_snapshot), Some(receipt), Some(packet), Some(_)) = (
                &attempt.pre_registration,
                &attempt.tested_snapshot,
                &attempt.receipt,
                &attempt.decision_packet,
                &attempt.sensitivity_brief,
            ) else {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires every frozen artifact"
                )));
            };
            if !prerequisites_complete {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires every prerequisite"
                )));
            }
            if attempt.session_records.is_empty()
                || !attempt
                    .session_records
                    .iter()
                    .any(|record| record.admissibility == "admissible")
            {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires admitted coded evidence"
                )));
            }
            if receipt.protocol_validity != "valid" || attempt.control_status != "watched-failing" {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires valid protocol and watched-failing control"
                )));
            }
            if &receipt.completed_at[..10] < registration.registered_date.as_str() {
                return Err(ReaderError::new(format!(
                    "{path}: completion cannot precede preregistration"
                )));
            }
            if receipt.completed_at <= registration.freeze_binding.frozen_at {
                return Err(ReaderError::new(format!(
                    "{path}: pilot completion must follow the frozen preregistration"
                )));
            }
            if packet.freeze_binding.frozen_at <= receipt.completed_at {
                return Err(ReaderError::new(format!(
                    "{path}: decision packet freeze must strictly follow pilot completion"
                )));
            }
        }
        "void" => {
            if attempt.void_reason_code.is_none() || !payload_present {
                return Err(ReaderError::new(format!(
                    "{path}: void pilot requires a coded reason and preserved evidence"
                )));
            }
            if attempt.control_status == "watched-failing"
                && attempt
                    .receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.protocol_validity == "valid")
            {
                return Err(ReaderError::new(format!(
                    "{path}: fully valid pilot cannot be relabelled void"
                )));
            }
            let voided_at = attempt
                .voided_at
                .as_deref()
                .expect("validated void timestamp");
            if let Some(registration) = &attempt.pre_registration
                && voided_at <= registration.freeze_binding.frozen_at.as_str()
            {
                return Err(ReaderError::new(format!(
                    "{path}: pilot void time must follow the frozen preregistration"
                )));
            }
            if attempt
                .receipt
                .as_ref()
                .is_some_and(|receipt| receipt.completed_at.as_str() > voided_at)
            {
                return Err(ReaderError::new(format!(
                    "{path}: pilot receipt completion cannot follow its void time"
                )));
            }
        }
        _ => unreachable!(),
    }
    validate_digest_str(
        &attempt.attempt_sha256,
        &format!("{path}.attempt_sha256"),
        Some(&typed_canonical_sha(attempt, path, Some("attempt_sha256"))?),
    )?;
    Ok(PilotAttemptTyped {
        attempt_id: &attempt.attempt_id,
        status: &attempt.attempt_status,
        packet: attempt.decision_packet.as_ref(),
        sensitivity: attempt.sensitivity_brief.as_ref(),
        receipt: attempt.receipt.as_ref(),
        control: &attempt.control_status,
        attempt_sha: &attempt.attempt_sha256,
    })
}

struct PilotValidationTyped<'a> {
    valid: bool,
    packet: Option<&'a DecisionPacket>,
    sensitivity: Option<&'a Artifact>,
    active_id: Option<&'a str>,
}

fn validate_pilot_typed<'a>(
    env: &ValidationEnv<'_>,
    source: &'a ReaderEvidenceSource,
) -> ReaderResult<PilotValidationTyped<'a>> {
    if !["not-run", "completed", "void"].contains(&source.pilot.pilot_status.as_str()) {
        return Err(ReaderError::new("pilot.pilot_status: invalid state"));
    }
    if ![
        "not-run",
        "watched-failing",
        "failed-to-fail",
        "indeterminate",
    ]
    .contains(&source.pilot.control_status.as_str())
    {
        return Err(ReaderError::new("pilot.control_status: invalid state"));
    }
    if source.pilot.attempts.is_empty() {
        if source.pilot.active_attempt_id.is_some()
            || source.pilot.pilot_status != "not-run"
            || source.pilot.control_status != "not-run"
        {
            return Err(ReaderError::new(
                "empty pilot history must remain not-run with no active attempt",
            ));
        }
        return Ok(PilotValidationTyped {
            valid: false,
            packet: None,
            sensitivity: None,
            active_id: None,
        });
    }
    let active_id =
        source.pilot.active_attempt_id.as_deref().ok_or_else(|| {
            ReaderError::new("non-empty pilot history requires active_attempt_id")
        })?;
    validate_id_str(active_id, "pilot.active_attempt_id")?;
    let mut previous_sha = None;
    let mut prior_head = history_head_sha256(std::iter::empty(), std::iter::empty());
    let mut ids = BTreeSet::new();
    let mut active = None;
    for (index, attempt) in source.pilot.attempts.iter().enumerate() {
        let path = format!("pilot.attempts[{index}]");
        let is_active = index + 1 == source.pilot.attempts.len();
        let validated = validate_pilot_attempt_typed(
            env,
            attempt,
            &path,
            source,
            previous_sha,
            &prior_head,
            index == 0,
            is_active,
        )?;
        if !ids.insert(validated.attempt_id) {
            return Err(ReaderError::new(format!(
                "{path}.attempt_id: duplicate pilot attempt"
            )));
        }
        previous_sha = Some(validated.attempt_sha);
        prior_head = history_head_sha256(
            source.pilot.attempts[..=index]
                .iter()
                .map(|item| item.attempt_sha256.as_str()),
            std::iter::empty(),
        );
        if is_active {
            active = Some(validated);
        }
    }
    let active = active.expect("non-empty attempts");
    if active.attempt_id != active_id {
        return Err(ReaderError::new(
            "pilot.active_attempt_id must identify the final preserved attempt",
        ));
    }
    if source.pilot.pilot_status != active.status || source.pilot.control_status != active.control {
        return Err(ReaderError::new(
            "pilot summary must exactly match the active attempt",
        ));
    }
    let valid = active.status == "completed"
        && active.control == "watched-failing"
        && active
            .receipt
            .is_some_and(|receipt| receipt.protocol_validity == "valid");
    Ok(PilotValidationTyped {
        valid,
        packet: active.packet,
        sensitivity: active.sensitivity,
        active_id: Some(active_id),
    })
}

fn validate_protocol(env: &ValidationEnv<'_>, source: &Map<String, Value>) -> ReaderResult<()> {
    if source["spdx"].as_str() != Some("CC-BY-4.0") {
        return Err(ReaderError::new("spdx must be CC-BY-4.0"));
    }
    if source["schema_version"].as_i64() != Some(1) {
        return Err(ReaderError::new("schema_version must be integer 1"));
    }
    if source["contract_id"].as_str() != Some("book-1-reader-evidence-v1") {
        return Err(ReaderError::new(
            "contract_id must be book-1-reader-evidence-v1",
        ));
    }
    let decision_ref = text(&source["protocol_decision_ref"], "protocol_decision_ref")?;
    let Some((relative, anchor)) = decision_ref.split_once("::") else {
        return Err(ReaderError::new(
            "protocol_decision_ref needs path::exact-anchor",
        ));
    };
    if relative != PROTOCOL_DECISION || anchor.is_empty() {
        return Err(ReaderError::new(
            "protocol_decision_ref must cite the controlling decision",
        ));
    }
    let decision_text = std::str::from_utf8(env.protocol_decision)
        .map_err(|_| ReaderError::new("candidate protocol decision is not valid UTF-8"))?;
    let count = decision_text.matches(anchor).count();
    if count != 1 {
        return Err(ReaderError::new(format!(
            "protocol_decision_ref anchor must occur exactly once in the candidate decision; found {count}"
        )));
    }
    let protocol = object(&source["protocol"], "protocol")?;
    exact_keys(protocol, PROTOCOL_KEYS, "protocol")?;
    let decision_digest = sha256(env.protocol_decision);
    digest(
        &protocol["decision_sha256"],
        "protocol.decision_sha256",
        Some(&decision_digest),
    )?;
    if protocol["method"].as_str() != Some("pre-registered-pilot-and-fresh-holdout") {
        return Err(ReaderError::new(
            "protocol.method drifted from the ratified method",
        ));
    }
    let evaluation = array(&protocol["evaluation_order"], "protocol.evaluation_order")?;
    if evaluation
        .iter()
        .map(Value::as_str)
        .ne(EVALUATION_ORDER.iter().copied().map(Some))
    {
        return Err(ReaderError::new(
            "protocol.evaluation_order must preserve the ratified order",
        ));
    }
    if protocol["aggregate_offset_prohibited"].as_bool() != Some(true) {
        return Err(ReaderError::new(
            "protocol must prohibit aggregate offset of a core finding",
        ));
    }
    let targets = array(&protocol["required_targets"], "protocol.required_targets")?;
    let mut found = BTreeMap::new();
    for (index, raw) in targets.iter().enumerate() {
        let item_path = format!("protocol.required_targets[{index}]");
        let target = object(raw, &item_path)?;
        exact_keys(target, TARGET_KEYS, &item_path)?;
        let identifier = text(&target["target_id"], &format!("{item_path}.target_id"))?;
        let description = text(&target["description"], &format!("{item_path}.description"))?;
        if found
            .insert(identifier.to_owned(), description.to_owned())
            .is_some()
        {
            return Err(ReaderError::new(format!(
                "{item_path}.target_id: duplicate {identifier}"
            )));
        }
    }
    let required: BTreeMap<_, _> = REQUIRED_TARGETS
        .iter()
        .map(|(identifier, description)| ((*identifier).to_owned(), (*description).to_owned()))
        .collect();
    if found != required {
        return Err(ReaderError::new(
            "protocol.required_targets drifted from the ratified minimum",
        ));
    }
    for (key, expected) in [
        ("disclosed_limits", DISCLOSED_LIMITS),
        ("ethics_terms", ETHICS_TERMS),
        ("freshness_terms", FRESHNESS_TERMS),
    ] {
        let actual = text_list(&protocol[key], &format!("protocol.{key}"), true)?;
        if actual != expected {
            return Err(ReaderError::new(format!(
                "protocol.{key} drifted from the ratified terms"
            )));
        }
    }
    if protocol["non_substitution"].as_str() != Some(NON_SUBSTITUTION) {
        return Err(ReaderError::new(
            "protocol.non_substitution drifted from the ratified boundary",
        ));
    }
    Ok(())
}

fn validate_privacy(source: &Map<String, Value>) -> ReaderResult<()> {
    let privacy = object(&source["privacy"], "privacy")?;
    exact_keys(privacy, PRIVACY_KEYS, "privacy")?;
    if privacy["public_record_policy"].as_str() != Some("privacy-minimal-coded-records-only") {
        return Err(ReaderError::new("privacy.public_record_policy drifted"));
    }
    if text_list(
        &privacy["allowed_public_record_kinds"],
        "privacy.allowed_public_record_kinds",
        true,
    )? != ALLOWED_PUBLIC_RECORD_KINDS
    {
        return Err(ReaderError::new(
            "privacy.allowed_public_record_kinds drifted",
        ));
    }
    if text_list(
        &privacy["excluded_from_repository"],
        "privacy.excluded_from_repository",
        true,
    )? != EXCLUDED_FROM_REPOSITORY
    {
        return Err(ReaderError::new("privacy.excluded_from_repository drifted"));
    }
    if privacy["freshness_attestation_boundary"].as_str() != Some(FRESHNESS_BOUNDARY) {
        return Err(ReaderError::new(
            "privacy.freshness_attestation_boundary drifted",
        ));
    }
    Ok(())
}

fn validate_session_records<'a>(
    value: &'a Value,
    path: &str,
    expected_study_id: Option<&str>,
    known_misconceptions: Option<&BTreeSet<String>>,
) -> ReaderResult<Vec<&'a Map<String, Value>>> {
    let mut records = Vec::new();
    let mut record_commitments = HashSet::new();
    let required_targets: BTreeSet<_> = REQUIRED_TARGETS.iter().map(|item| item.0).collect();
    for (index, raw) in array(value, path)?.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let record = object(raw, &item_path)?;
        exact_keys(record, SESSION_KEYS, &item_path)?;
        let study_id = opaque_id(&record["study_id"], &format!("{item_path}.study_id"))?;
        let commitment = digest(
            &record["record_commitment_sha256"],
            &format!("{item_path}.record_commitment_sha256"),
            None,
        )?;
        if !record_commitments.insert(commitment) {
            return Err(ReaderError::new(format!(
                "{item_path}.record_commitment_sha256: duplicate coded session"
            )));
        }
        if expected_study_id.is_some_and(|expected| study_id != expected) {
            return Err(ReaderError::new(format!(
                "{item_path}.study_id: does not match pre-registration"
            )));
        }
        let admissibility = enumeration(
            &record["admissibility"],
            &["admissible", "inadmissible", "withdrawn"],
            &format!("{item_path}.admissibility"),
        )?;

        let target_outcomes = array(
            &record["target_outcomes"],
            &format!("{item_path}.target_outcomes"),
        )?;
        let mut seen_targets = BTreeSet::new();
        for (outcome_index, raw_outcome) in target_outcomes.iter().enumerate() {
            let outcome_path = format!("{item_path}.target_outcomes[{outcome_index}]");
            let outcome = object(raw_outcome, &outcome_path)?;
            exact_keys(outcome, TARGET_OUTCOME_KEYS, &outcome_path)?;
            let target_id = text(&outcome["target_id"], &format!("{outcome_path}.target_id"))?;
            if !required_targets.contains(target_id) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.target_id: unknown target"
                )));
            }
            if !seen_targets.insert(target_id) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.target_id: duplicate target"
                )));
            }
            let status = enumeration(
                &outcome["status"],
                &[
                    "identified",
                    "not-identified",
                    "missing",
                    "ambiguous",
                    "multiply-coded",
                    "unclassified",
                ],
                &format!("{outcome_path}.status"),
            )?;
            let adjudication = enumeration(
                &outcome["adjudication"],
                &["not-required", "resolved", "unresolved"],
                &format!("{outcome_path}.adjudication"),
            )?;
            let final_status = matches!(status, "identified" | "not-identified");
            if adjudication == "resolved" && !final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a resolved target outcome must carry a final status"
                )));
            }
            if adjudication == "unresolved" && final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a final target outcome cannot remain unresolved"
                )));
            }
        }

        let misconception_outcomes = array(
            &record["misconception_outcomes"],
            &format!("{item_path}.misconception_outcomes"),
        )?;
        let mut seen_misconceptions = BTreeSet::new();
        for (outcome_index, raw_outcome) in misconception_outcomes.iter().enumerate() {
            let outcome_path = format!("{item_path}.misconception_outcomes[{outcome_index}]");
            let outcome = object(raw_outcome, &outcome_path)?;
            exact_keys(outcome, MISCONCEPTION_OUTCOME_KEYS, &outcome_path)?;
            let misconception_id = opaque_id(
                &outcome["misconception_id"],
                &format!("{outcome_path}.misconception_id"),
            )?;
            if !seen_misconceptions.insert(misconception_id.to_owned()) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.misconception_id: duplicate misconception"
                )));
            }
            if known_misconceptions.is_some_and(|known| !known.contains(misconception_id)) {
                return Err(ReaderError::new(format!(
                    "{outcome_path}.misconception_id: unknown misconception"
                )));
            }
            let status = enumeration(
                &outcome["status"],
                &[
                    "present",
                    "absent",
                    "missing",
                    "ambiguous",
                    "multiply-coded",
                    "unclassified",
                ],
                &format!("{outcome_path}.status"),
            )?;
            let adjudication = enumeration(
                &outcome["adjudication"],
                &["not-required", "resolved", "unresolved"],
                &format!("{outcome_path}.adjudication"),
            )?;
            let occurrences = BigNat::from_decimal(integer_text(
                &outcome["occurrences"],
                &format!("{outcome_path}.occurrences"),
                false,
            )?);
            let opportunities = BigNat::from_decimal(integer_text(
                &outcome["opportunities"],
                &format!("{outcome_path}.opportunities"),
                true,
            )?);
            if occurrences > opportunities {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: occurrences cannot exceed opportunities"
                )));
            }
            if status == "absent" && !occurrences.is_zero() {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: absent requires zero occurrences"
                )));
            }
            if status == "present" && occurrences.is_zero() {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: present requires at least one occurrence"
                )));
            }
            let final_status = matches!(status, "present" | "absent");
            if adjudication == "resolved" && !final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a resolved misconception outcome must carry a final status"
                )));
            }
            if adjudication == "unresolved" && final_status {
                return Err(ReaderError::new(format!(
                    "{outcome_path}: a final misconception outcome cannot remain unresolved"
                )));
            }
        }
        for key in ["deviation_ids", "custody_attestation_ids"] {
            for (id_index, identifier) in
                text_list(&record[key], &format!("{item_path}.{key}"), false)?
                    .iter()
                    .enumerate()
            {
                opaque_id(
                    &Value::String((*identifier).to_owned()),
                    &format!("{item_path}.{key}[{id_index}]"),
                )?;
            }
        }
        if admissibility != "admissible" {
            if !target_outcomes.is_empty() || !misconception_outcomes.is_empty() {
                return Err(ReaderError::new(format!(
                    "{item_path}: inadmissible or withdrawn sessions may not publish coded outcomes"
                )));
            }
        } else {
            if seen_targets != required_targets {
                return Err(ReaderError::new(format!(
                    "{item_path}: every admissible session must explicitly code every required target"
                )));
            }
            if known_misconceptions.is_some_and(|known| &seen_misconceptions != known) {
                return Err(ReaderError::new(format!(
                    "{item_path}: every holdout session must explicitly code every ratified misconception"
                )));
            }
        }
        records.push(record);
    }
    Ok(records)
}

fn empty_threshold_content(rule: &Map<String, Value>) -> ReaderResult<bool> {
    Ok(rule["rule_id"].is_null()
        && array(
            &rule["severity_taxonomy"],
            "threshold_rule.severity_taxonomy",
        )?
        .is_empty()
        && array(&rule["misconceptions"], "threshold_rule.misconceptions")?.is_empty()
        && array(
            &rule["core_misconception_ids"],
            "threshold_rule.core_misconception_ids",
        )?
        .is_empty()
        && rule["core_failure_mode"].is_null()
        && rule["repetition_unit"].is_null()
        && rule["denominator"].is_null()
        && rule["core_failure_threshold"].is_null()
        && array(
            &rule["required_target_thresholds"],
            "threshold_rule.required_target_thresholds",
        )?
        .is_empty()
        && array(
            &rule["non_core_thresholds"],
            "threshold_rule.non_core_thresholds",
        )?
        .is_empty()
        && rule["minimum_evaluable_evidence"].is_null()
        && object(&rule["policies"], "threshold_rule.policies")?
            .values()
            .all(Value::is_null)
        && rule["rule_sha256"].is_null())
}

fn decimal_positive(value: &str) -> bool {
    decimal_regex().is_match(value)
        && value
            .bytes()
            .any(|byte| byte.is_ascii_digit() && byte != b'0')
}

fn decimal_less_than_one(value: &str) -> bool {
    let (whole, _) = value.split_once('.').unwrap_or((value, ""));
    whole == "0" && decimal_positive(value)
}

fn validate_threshold_spec<'a>(
    value: &'a Value,
    path: &str,
    allowed_metrics: &[&str],
    scope_refs: &BTreeSet<String>,
) -> ReaderResult<&'a Map<String, Value>> {
    const THRESHOLD_METRICS: &[&str] = &[
        "admissible-session-count",
        "target-identification-count",
        "target-identification-rate",
        "core-finding-present",
        "core-finding-count",
        "core-finding-rate",
        "severity-session-finding-count",
        "severity-session-finding-rate",
        "severity-occurrence-count",
        "severity-occurrence-rate",
    ];
    if allowed_metrics.is_empty()
        || allowed_metrics
            .iter()
            .any(|metric| !THRESHOLD_METRICS.contains(metric))
    {
        return Err(ReaderError::new(format!(
            "{path}: internal metric registry is incomplete"
        )));
    }
    let spec = object(value, path)?;
    exact_keys(spec, THRESHOLD_SPEC_KEYS, path)?;
    opaque_id(&spec["threshold_id"], &format!("{path}.threshold_id"))?;
    let metric = enumeration(&spec["metric"], allowed_metrics, &format!("{path}.metric"))?;
    let operator = enumeration(
        &spec["operator"],
        &["lt", "lte", "eq", "gte", "gt"],
        &format!("{path}.operator"),
    )?;
    let value_kind = enumeration(
        &spec["value_kind"],
        &["integer", "decimal", "qualitative"],
        &format!("{path}.value_kind"),
    )?;
    let value_text = text(&spec["value"], &format!("{path}.value"))?;
    let unit = text(&spec["unit"], &format!("{path}.unit"))?;
    let denominator = text(&spec["denominator"], &format!("{path}.denominator"))?;
    let refs: BTreeSet<_> = text_list(&spec["scope_refs"], &format!("{path}.scope_refs"), true)?
        .into_iter()
        .map(str::to_owned)
        .collect();
    if &refs != scope_refs {
        return Err(ReaderError::new(format!(
            "{path}.scope_refs: must exactly match its rule scope"
        )));
    }
    if !spec["evaluator_ref"].is_null() {
        return Err(ReaderError::new(format!(
            "{path}.evaluator_ref: release thresholds use the deterministic built-in evaluator"
        )));
    }
    let count_contract = match metric {
        "admissible-session-count" => Some(("sessions", "none")),
        "target-identification-count" => Some(("identified-sessions", "none")),
        "core-finding-count" => Some(("findings", "none")),
        "severity-session-finding-count" => Some(("sessions", "none")),
        "severity-occurrence-count" => Some(("occurrences", "none")),
        _ => None,
    };
    let rate_denominators: &[&str] = match metric {
        "target-identification-rate" => &["coded-target-observations"],
        "core-finding-rate" => &["eligible-admissible-sessions", "coded-opportunities"],
        "severity-session-finding-rate" => &["eligible-admissible-sessions"],
        "severity-occurrence-rate" => &["coded-opportunities"],
        _ => &[],
    };
    if metric == "core-finding-present" {
        if value_kind != "qualitative"
            || operator != "eq"
            || value_text != "present"
            || unit != "presence"
            || denominator != "none"
        {
            return Err(ReaderError::new(format!(
                "{path}: single-finding core veto must compare presence exactly"
            )));
        }
    } else if let Some((expected_unit, expected_denominator)) = count_contract {
        if value_kind != "integer"
            || unit != expected_unit
            || denominator != expected_denominator
            || !integer_regex().is_match(value_text)
            || !decimal_positive(value_text)
        {
            return Err(ReaderError::new(format!(
                "{path}: count threshold must admit reachable below, exact, and above cases"
            )));
        }
    } else if !rate_denominators.is_empty() {
        if value_kind != "decimal"
            || unit != "proportion"
            || !rate_denominators.contains(&denominator)
            || !decimal_positive(value_text)
            || !decimal_less_than_one(value_text)
        {
            return Err(ReaderError::new(format!(
                "{path}: rate threshold must admit reachable below, exact, and above cases"
            )));
        }
    } else {
        return Err(ReaderError::new(format!(
            "{path}.metric: unsupported deterministic metric"
        )));
    }
    Ok(spec)
}

fn validate_threshold_rule(source: &Value, valid_pilot: bool) -> ReaderResult<BTreeSet<String>> {
    let source = object(source, "threshold source")?;
    let rule = object(&source["threshold_rule"], "threshold_rule")?;
    exact_keys(rule, THRESHOLD_RULE_KEYS, "threshold_rule")?;
    let policies = object(&rule["policies"], "threshold_rule.policies")?;
    exact_keys(policies, POLICY_KEYS, "threshold_rule.policies")?;
    if array(&rule["evaluation_order"], "threshold_rule.evaluation_order")?
        .iter()
        .map(Value::as_str)
        .ne(EVALUATION_ORDER.iter().copied().map(Some))
    {
        return Err(ReaderError::new(
            "threshold_rule.evaluation_order must preserve the fixed order",
        ));
    }
    if rule["aggregate_offset_prohibited"].as_bool() != Some(true) {
        return Err(ReaderError::new(
            "threshold_rule must preserve the no-aggregate core veto",
        ));
    }
    let status = enumeration(
        &source["threshold_status"],
        &["pending-pilot", "candidate", "author-ratified"],
        "threshold_status",
    )?;
    if !valid_pilot {
        if status != "pending-pilot" || !empty_threshold_content(rule)? {
            return Err(ReaderError::new(
                "threshold taxonomy and values are prohibited until a valid completed pilot exists",
            ));
        }
        if !source["ratification"].is_null() {
            return Err(ReaderError::new(
                "ratification is prohibited before a valid completed pilot",
            ));
        }
        return Ok(BTreeSet::new());
    }
    if status == "pending-pilot" {
        if !empty_threshold_content(rule)? || !source["ratification"].is_null() {
            return Err(ReaderError::new(
                "pending-pilot must not carry a candidate rule or ratification",
            ));
        }
        return Ok(BTreeSet::new());
    }

    let rule_id = opaque_id(&rule["rule_id"], "threshold_rule.rule_id")?;
    let mut severities = BTreeSet::new();
    for (index, raw) in array(
        &rule["severity_taxonomy"],
        "threshold_rule.severity_taxonomy",
    )?
    .iter()
    .enumerate()
    {
        let item_path = format!("threshold_rule.severity_taxonomy[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, SEVERITY_KEYS, &item_path)?;
        let identifier = opaque_id(&item["severity_id"], &format!("{item_path}.severity_id"))?;
        if !severities.insert(identifier.to_owned()) {
            return Err(ReaderError::new(format!(
                "{item_path}.severity_id: duplicate {identifier}"
            )));
        }
        for key in ["label", "definition", "classification_boundary"] {
            text(&item[key], &format!("{item_path}.{key}"))?;
        }
    }
    if severities.is_empty() {
        return Err(ReaderError::new(
            "candidate threshold rule requires a severity taxonomy",
        ));
    }
    let mut misconceptions = BTreeMap::new();
    for (index, raw) in array(&rule["misconceptions"], "threshold_rule.misconceptions")?
        .iter()
        .enumerate()
    {
        let item_path = format!("threshold_rule.misconceptions[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, MISCONCEPTION_KEYS, &item_path)?;
        let identifier = opaque_id(
            &item["misconception_id"],
            &format!("{item_path}.misconception_id"),
        )?;
        if misconceptions.contains_key(identifier) {
            return Err(ReaderError::new(format!(
                "{item_path}.misconception_id: duplicate {identifier}"
            )));
        }
        text(&item["definition"], &format!("{item_path}.definition"))?;
        let severity = opaque_id(&item["severity_id"], &format!("{item_path}.severity_id"))?;
        if !severities.contains(severity) {
            return Err(ReaderError::new(format!(
                "{item_path}.severity_id: unknown severity"
            )));
        }
        boolean(&item["core"], &format!("{item_path}.core"))?;
        misconceptions.insert(identifier.to_owned(), item);
    }
    if misconceptions.is_empty() {
        return Err(ReaderError::new(
            "candidate threshold rule requires stable misconception IDs",
        ));
    }
    let declared_core: BTreeSet<_> = text_list(
        &rule["core_misconception_ids"],
        "threshold_rule.core_misconception_ids",
        true,
    )?
    .into_iter()
    .map(str::to_owned)
    .collect();
    let actual_core: BTreeSet<_> = misconceptions
        .iter()
        .filter(|(_, item)| item["core"].as_bool() == Some(true))
        .map(|(identifier, _)| identifier.clone())
        .collect();
    if declared_core != actual_core || actual_core.is_empty() {
        return Err(ReaderError::new(
            "core_misconception_ids must exactly match non-empty core mappings",
        ));
    }
    let core_mode = enumeration(
        &rule["core_failure_mode"],
        &["single", "repeated"],
        "threshold_rule.core_failure_mode",
    )?;
    let repetition_unit = enumeration(
        &rule["repetition_unit"],
        &["admissible-session", "coded-opportunity"],
        "threshold_rule.repetition_unit",
    )?;
    let denominator = text(&rule["denominator"], "threshold_rule.denominator")?;
    let core_metrics: &[&str] = if core_mode == "single" {
        &["core-finding-present"]
    } else {
        &["core-finding-count", "core-finding-rate"]
    };
    let core_spec = validate_threshold_spec(
        &rule["core_failure_threshold"],
        "threshold_rule.core_failure_threshold",
        core_metrics,
        &actual_core,
    )?;
    let core_metric = core_spec["metric"].as_str().expect("validated metric");
    if core_metric != "core-finding-present" {
        let operator = core_spec["operator"].as_str().expect("validated operator");
        let value = core_spec["value"].as_str().expect("validated value");
        if !matches!(operator, "gte" | "gt")
            || !decimal_positive(value)
            || (core_metric == "core-finding-rate"
                && operator == "gt"
                && !decimal_less_than_one(value))
        {
            return Err(ReaderError::new(
                "repeated core veto must use a positive, reachable adverse boundary",
            ));
        }
    }
    let expected_denominator = if core_metric == "core-finding-rate" {
        if repetition_unit == "admissible-session" {
            "eligible-admissible-sessions"
        } else {
            "coded-opportunities"
        }
    } else {
        "none"
    };
    if denominator != expected_denominator
        || core_spec["denominator"].as_str() != Some(expected_denominator)
    {
        return Err(ReaderError::new(
            "threshold_rule.denominator must match the selected core branch and metric",
        ));
    }
    for key in ["ambiguous", "missing", "multiply_coded", "unclassified"] {
        enumeration(
            &policies[key],
            &[
                "count-adverse",
                "exclude-observation",
                "study-not-evaluable",
                "require-adjudication",
            ],
            &format!("threshold_rule.policies.{key}"),
        )?;
    }
    for key in ["withdrawn", "excluded"] {
        if policies[key].as_str() != Some("exclude-session") {
            return Err(ReaderError::new(format!(
                "threshold_rule.policies.{key} must preserve session exclusion"
            )));
        }
    }
    if policies["rounding"].as_str() != Some("exact-decimal-no-rounding") {
        return Err(ReaderError::new(
            "threshold_rule.policies.rounding must preserve exact comparison",
        ));
    }
    enumeration(
        &policies["coder_adjudication"],
        &[
            "unresolved-count-adverse",
            "unresolved-exclude-observation",
            "unresolved-not-evaluable",
        ],
        "threshold_rule.policies.coder_adjudication",
    )?;

    let mut required = BTreeSet::new();
    let mut threshold_ids = BTreeSet::from([core_spec["threshold_id"]
        .as_str()
        .expect("validated threshold ID")
        .to_owned()]);
    let target_ids: BTreeSet<_> = REQUIRED_TARGETS.iter().map(|item| item.0).collect();
    for (index, raw) in array(
        &rule["required_target_thresholds"],
        "threshold_rule.required_target_thresholds",
    )?
    .iter()
    .enumerate()
    {
        let item_path = format!("threshold_rule.required_target_thresholds[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, TARGET_THRESHOLD_KEYS, &item_path)?;
        let target_id = text(&item["target_id"], &format!("{item_path}.target_id"))?;
        if !target_ids.contains(target_id) || !required.insert(target_id) {
            return Err(ReaderError::new(format!(
                "{item_path}.target_id: unknown or duplicate target"
            )));
        }
        let scope = BTreeSet::from([target_id.to_owned()]);
        let spec = validate_threshold_spec(
            &item["threshold"],
            &format!("{item_path}.threshold"),
            &["target-identification-count", "target-identification-rate"],
            &scope,
        )?;
        let operator = spec["operator"].as_str().expect("validated operator");
        let value = spec["value"].as_str().expect("validated value");
        if !matches!(operator, "gte" | "gt")
            || !decimal_positive(value)
            || (spec["metric"].as_str() == Some("target-identification-rate")
                && operator == "gt"
                && !decimal_less_than_one(value))
        {
            return Err(ReaderError::new(format!(
                "{item_path}.threshold: target success boundary must be positive and reachable"
            )));
        }
        threshold_ids.insert(spec["threshold_id"].as_str().unwrap().to_owned());
    }
    if required != target_ids {
        return Err(ReaderError::new(
            "required_target_thresholds must cover every required target",
        ));
    }

    let non_core_severities: BTreeSet<_> = misconceptions
        .values()
        .filter(|item| item["core"].as_bool() == Some(false))
        .map(|item| item["severity_id"].as_str().unwrap().to_owned())
        .collect();
    let mut mapped_non_core = BTreeSet::new();
    for (index, raw) in array(
        &rule["non_core_thresholds"],
        "threshold_rule.non_core_thresholds",
    )?
    .iter()
    .enumerate()
    {
        let item_path = format!("threshold_rule.non_core_thresholds[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, SEVERITY_THRESHOLD_KEYS, &item_path)?;
        let severity_id = opaque_id(&item["severity_id"], &format!("{item_path}.severity_id"))?;
        if !non_core_severities.contains(severity_id)
            || !mapped_non_core.insert(severity_id.to_owned())
        {
            return Err(ReaderError::new(format!(
                "{item_path}.severity_id: unknown, core, or duplicate severity"
            )));
        }
        let scope = BTreeSet::from([severity_id.to_owned()]);
        let spec = validate_threshold_spec(
            &item["threshold"],
            &format!("{item_path}.threshold"),
            &[
                "severity-session-finding-count",
                "severity-session-finding-rate",
                "severity-occurrence-count",
                "severity-occurrence-rate",
            ],
            &scope,
        )?;
        let operator = spec["operator"].as_str().expect("validated operator");
        let value = spec["value"].as_str().expect("validated value");
        if !matches!(operator, "lt" | "lte")
            || (operator == "lt" && !decimal_positive(value))
            || (spec["metric"].as_str().unwrap().ends_with("-rate")
                && operator == "lte"
                && !decimal_less_than_one(value))
        {
            return Err(ReaderError::new(format!(
                "{item_path}.threshold: non-core boundary must be adverse and falsifiable"
            )));
        }
        threshold_ids.insert(spec["threshold_id"].as_str().unwrap().to_owned());
    }
    if mapped_non_core != non_core_severities {
        return Err(ReaderError::new(
            "non_core_thresholds must cover every used non-core severity",
        ));
    }
    let minimum_scope = BTreeSet::from([rule_id.to_owned()]);
    let minimum = validate_threshold_spec(
        &rule["minimum_evaluable_evidence"],
        "threshold_rule.minimum_evaluable_evidence",
        &["admissible-session-count"],
        &minimum_scope,
    )?;
    if !matches!(minimum["operator"].as_str(), Some("gte" | "gt"))
        || !decimal_positive(minimum["value"].as_str().unwrap())
    {
        return Err(ReaderError::new(
            "minimum evaluable evidence must require a positive admitted count",
        ));
    }
    threshold_ids.insert(minimum["threshold_id"].as_str().unwrap().to_owned());
    let expected_threshold_count = 2 + required.len() + mapped_non_core.len();
    if threshold_ids.len() != expected_threshold_count {
        return Err(ReaderError::new(
            "threshold IDs must be unique across the complete rule",
        ));
    }
    let expected_digest = canonical_sha(&Value::Object(rule.clone()), Some("rule_sha256"))?;
    digest(
        &rule["rule_sha256"],
        "threshold_rule.rule_sha256",
        Some(&expected_digest),
    )?;
    Ok(misconceptions.keys().cloned().collect())
}

fn validate_pilot_pre_registration<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    fixed_protocol_sha256: &str,
    expected_predecessor_attempt_sha256: Option<&str>,
    expected_prior_history_head_sha256: &str,
) -> ReaderResult<&'a Map<String, Value>> {
    let registration = object(value, path)?;
    exact_keys(registration, PILOT_PRE_REGISTRATION_KEYS, path)?;
    opaque_id(&registration["study_id"], &format!("{path}.study_id"))?;
    let registered_date = date(
        &registration["registered_date"],
        &format!("{path}.registered_date"),
    )?;
    validate_preregistration_history_binding(
        registration,
        path,
        expected_predecessor_attempt_sha256,
        Some(expected_prior_history_head_sha256),
        true,
    )?;
    digest(
        &registration["fixed_protocol_sha256"],
        &format!("{path}.fixed_protocol_sha256"),
        Some(fixed_protocol_sha256),
    )?;
    for key in [
        "protocol",
        "instrument",
        "rubric",
        "sample_rule",
        "disclosure_set",
        "ethics_terms",
        "provisional_rule",
    ] {
        validate_artifact(env, &registration[key], &format!("{path}.{key}"), true)?;
    }
    let binding = validate_freeze_binding(
        env,
        &registration["freeze_binding"],
        &format!("{path}.freeze_binding"),
        registration,
        "pre_registration_sha256",
        HistoricalPayloadKind::PilotPreRegistration,
    )?;
    if binding["frozen_at"]
        .as_str()
        .expect("validated timestamp")
        .get(..10)
        .expect("timestamp has date")
        < registered_date
    {
        return Err(ReaderError::new(format!(
            "{path}.freeze_binding: external freeze cannot precede registration"
        )));
    }
    let expected = canonical_sha(
        &Value::Object(registration.clone()),
        Some("pre_registration_sha256"),
    )?;
    digest(
        &registration["pre_registration_sha256"],
        &format!("{path}.pre_registration_sha256"),
        Some(&expected),
    )?;
    Ok(registration)
}

fn validate_decision_packet<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
) -> ReaderResult<&'a Map<String, Value>> {
    let packet = object(value, path)?;
    exact_keys(packet, DECISION_PACKET_KEYS, path)?;
    opaque_id(&packet["packet_id"], &format!("{path}.packet_id"))?;
    let frozen_date = date(&packet["frozen_date"], &format!("{path}.frozen_date"))?;
    digest(
        &packet["pilot_pre_registration_sha256"],
        &format!("{path}.pilot_pre_registration_sha256"),
        None,
    )?;
    digest(
        &packet["tested_snapshot_sha256"],
        &format!("{path}.tested_snapshot_sha256"),
        None,
    )?;
    for key in [
        "coded_evidence",
        "exclusions",
        "coder_disagreements",
        "deviations",
        "revised_instrument",
        "control_transcript",
    ] {
        validate_artifact(env, &packet[key], &format!("{path}.{key}"), true)?;
    }
    let binding = validate_freeze_binding(
        env,
        &packet["freeze_binding"],
        &format!("{path}.freeze_binding"),
        packet,
        "packet_sha256",
        HistoricalPayloadKind::PilotDecisionPacket,
    )?;
    if binding["frozen_at"]
        .as_str()
        .expect("validated timestamp")
        .get(..10)
        .expect("timestamp has date")
        != frozen_date
    {
        return Err(ReaderError::new(format!(
            "{path}.frozen_date must equal the freeze binding calendar date"
        )));
    }
    let expected = canonical_sha(&Value::Object(packet.clone()), Some("packet_sha256"))?;
    digest(
        &packet["packet_sha256"],
        &format!("{path}.packet_sha256"),
        Some(&expected),
    )?;
    Ok(packet)
}

fn validate_pilot_receipt<'a>(
    value: &'a Value,
    path: &str,
    registration: &Map<String, Value>,
    snapshot: &Map<String, Value>,
    sessions: &[&Map<String, Value>],
    raw_sessions: &[Value],
    raw_deviations: &[Value],
    raw_custody: &[Value],
    packet: &Map<String, Value>,
) -> ReaderResult<&'a Map<String, Value>> {
    let receipt = object(value, path)?;
    exact_keys(receipt, PILOT_RECEIPT_KEYS, path)?;
    opaque_id(&receipt["receipt_id"], &format!("{path}.receipt_id"))?;
    utc_timestamp(&receipt["completed_at"], &format!("{path}.completed_at"))?;
    let study_id = opaque_id(&receipt["study_id"], &format!("{path}.study_id"))?;
    if registration["study_id"].as_str() != Some(study_id) {
        return Err(ReaderError::new(format!(
            "{path}.study_id: does not match pre-registration"
        )));
    }
    enumeration(
        &receipt["protocol_validity"],
        &["valid", "invalid"],
        &format!("{path}.protocol_validity"),
    )?;
    let instrument = object(&registration["instrument"], "instrument")?;
    let rubric = object(&registration["rubric"], "rubric")?;
    let coded_evidence = object(&packet["coded_evidence"], "coded evidence")?;
    let control_transcript = object(&packet["control_transcript"], "control transcript")?;
    let digest_links = [
        (
            "pre_registration_sha256",
            registration["pre_registration_sha256"]
                .as_str()
                .unwrap()
                .to_owned(),
        ),
        (
            "snapshot_sha256",
            snapshot["sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "instrument_sha256",
            instrument["sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "rubric_sha256",
            rubric["sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "coded_evidence_sha256",
            coded_evidence["sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "coded_records_sha256",
            canonical_sha(&Value::Array(raw_sessions.to_vec()), None)?,
        ),
        (
            "deviations_sha256",
            canonical_sha(&Value::Array(raw_deviations.to_vec()), None)?,
        ),
        (
            "control_transcript_sha256",
            control_transcript["sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "decision_packet_sha256",
            packet["packet_sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "custody_records_sha256",
            canonical_sha(&Value::Array(raw_custody.to_vec()), None)?,
        ),
    ];
    for (key, expected) in digest_links {
        digest(&receipt[key], &format!("{path}.{key}"), Some(&expected))?;
    }
    let classifications = Value::Array(
        sessions
            .iter()
            .map(|record| {
                serde_json::json!({
                    "record_commitment_sha256": record["record_commitment_sha256"],
                    "admissibility": record["admissibility"],
                })
            })
            .collect(),
    );
    let expected = canonical_sha(&classifications, None)?;
    digest(
        &receipt["session_classification_sha256"],
        &format!("{path}.session_classification_sha256"),
        Some(&expected),
    )?;
    digest(
        &receipt["coder_sha256"],
        &format!("{path}.coder_sha256"),
        None,
    )?;
    let custody_digests = text_list(
        &receipt["custody_attestation_sha256s"],
        &format!("{path}.custody_attestation_sha256s"),
        true,
    )?;
    let expected_custody_digests: Vec<_> = raw_custody
        .iter()
        .map(|item| {
            object(item, "pilot custody record")
                .and_then(|record| digest(&record["sha256"], "pilot custody record.sha256", None))
        })
        .collect::<ReaderResult<_>>()?;
    if custody_digests != expected_custody_digests {
        return Err(ReaderError::new(format!(
            "{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )));
    }
    for (index, value) in custody_digests.iter().enumerate() {
        digest(
            &Value::String((*value).to_owned()),
            &format!("{path}.custody_attestation_sha256s[{index}]"),
            None,
        )?;
    }
    let expected = canonical_sha(&Value::Object(receipt.clone()), Some("receipt_sha256"))?;
    digest(
        &receipt["receipt_sha256"],
        &format!("{path}.receipt_sha256"),
        Some(&expected),
    )?;
    Ok(receipt)
}

struct PilotAttempt<'a> {
    attempt_id: &'a str,
    status: &'a str,
    packet: Option<&'a Map<String, Value>>,
    sensitivity: Option<&'a Map<String, Value>>,
    receipt: Option<&'a Map<String, Value>>,
    control: &'a str,
    attempt_sha: &'a str,
}

fn validate_pilot_attempt<'a>(
    env: &ValidationEnv<'_>,
    raw: &'a Value,
    path: &str,
    source: &'a Map<String, Value>,
    previous_sha256: Option<&str>,
    prior_history_head_sha256: &str,
    first: bool,
    active: bool,
) -> ReaderResult<PilotAttempt<'a>> {
    let attempt = object(raw, path)?;
    exact_keys(attempt, PILOT_ATTEMPT_KEYS, path)?;
    let attempt_id = opaque_id(&attempt["attempt_id"], &format!("{path}.attempt_id"))?;
    let declared_previous = &attempt["previous_attempt_sha256"];
    if first {
        if !declared_previous.is_null() {
            return Err(ReaderError::new(format!(
                "{path}.previous_attempt_sha256: first attempt must be null"
            )));
        }
    } else {
        digest(
            declared_previous,
            &format!("{path}.previous_attempt_sha256"),
            previous_sha256,
        )?;
    }
    let status = enumeration(
        &attempt["attempt_status"],
        &["not-run", "completed", "void"],
        &format!("{path}.attempt_status"),
    )?;
    let control = enumeration(
        &attempt["control_status"],
        &[
            "not-run",
            "watched-failing",
            "failed-to-fail",
            "indeterminate",
        ],
        &format!("{path}.control_status"),
    )?;
    if !active && status == "not-run" {
        return Err(ReaderError::new(format!(
            "{path}: only the active final pilot attempt may remain not-run"
        )));
    }
    let void_reason = if attempt["void_reason_code"].is_null() {
        None
    } else {
        Some(opaque_id(
            &attempt["void_reason_code"],
            &format!("{path}.void_reason_code"),
        )?)
    };
    let voided_at = if status == "void" {
        Some(utc_timestamp(
            &attempt["voided_at"],
            &format!("{path}.voided_at"),
        )?)
    } else {
        if !attempt["voided_at"].is_null() {
            return Err(ReaderError::new(format!(
                "{path}.voided_at: only a void pilot attempt may carry a terminal time"
            )));
        }
        None
    };
    let prerequisites = object(&attempt["prerequisites"], &format!("{path}.prerequisites"))?;
    exact_keys(
        prerequisites,
        PREREQUISITE_KEYS,
        &format!("{path}.prerequisites"),
    )?;
    for (key, reference) in prerequisites {
        if !reference.is_null() {
            validate_repo_reference(env, reference, &format!("{path}.prerequisites.{key}"))?;
        }
    }
    let fixed_protocol_sha256 = canonical_sha_omitting(&source["protocol"], &["decision_sha256"])?;
    let registration = if attempt["pre_registration"].is_null() {
        None
    } else {
        Some(validate_pilot_pre_registration(
            env,
            &attempt["pre_registration"],
            &format!("{path}.pre_registration"),
            &fixed_protocol_sha256,
            previous_sha256,
            prior_history_head_sha256,
        )?)
    };
    let snapshot = if attempt["tested_snapshot"].is_null() {
        None
    } else {
        Some(validate_artifact(
            env,
            &attempt["tested_snapshot"],
            &format!("{path}.tested_snapshot"),
            true,
        )?)
    };
    let study_id = registration.and_then(|value| value["study_id"].as_str());
    let raw_sessions = array(
        &attempt["session_records"],
        &format!("{path}.session_records"),
    )?;
    let sessions = validate_session_records(
        &attempt["session_records"],
        &format!("{path}.session_records"),
        study_id,
        None,
    )?;
    let raw_deviations = array(&attempt["deviations"], &format!("{path}.deviations"))?;
    let deviations = validate_deviations(&attempt["deviations"], &format!("{path}.deviations"))?;
    let raw_custody = array(
        &attempt["custody_attestations"],
        &format!("{path}.custody_attestations"),
    )?;
    let custody = validate_custody(
        env,
        &attempt["custody_attestations"],
        &format!("{path}.custody_attestations"),
    )?;
    validate_record_links(
        &sessions,
        &deviations,
        &custody,
        &format!("{path}.record_links"),
        study_id,
        None,
    )?;
    let pilot_run_evidence = !sessions.is_empty()
        || !raw_deviations.is_empty()
        || !raw_custody.is_empty()
        || !attempt["receipt"].is_null()
        || !attempt["decision_packet"].is_null()
        || !attempt["sensitivity_brief"].is_null();
    validate_pilot_run_freshness(
        &custody,
        &format!("{path}.custody_attestations"),
        study_id,
        pilot_run_evidence,
        status == "completed",
    )?;
    let packet = if attempt["decision_packet"].is_null() {
        None
    } else {
        Some(validate_decision_packet(
            env,
            &attempt["decision_packet"],
            &format!("{path}.decision_packet"),
        )?)
    };
    if let (Some(registration), Some(snapshot), Some(packet)) = (registration, snapshot, packet) {
        digest(
            &packet["pilot_pre_registration_sha256"],
            &format!("{path}.decision_packet.pilot_pre_registration_sha256"),
            registration["pre_registration_sha256"].as_str(),
        )?;
        digest(
            &packet["tested_snapshot_sha256"],
            &format!("{path}.decision_packet.tested_snapshot_sha256"),
            snapshot["sha256"].as_str(),
        )?;
    }
    let sensitivity = if attempt["sensitivity_brief"].is_null() {
        None
    } else {
        Some(validate_artifact(
            env,
            &attempt["sensitivity_brief"],
            &format!("{path}.sensitivity_brief"),
            true,
        )?)
    };
    let receipt = if attempt["receipt"].is_null() {
        None
    } else {
        let (Some(registration), Some(snapshot), Some(packet)) = (registration, snapshot, packet)
        else {
            return Err(ReaderError::new(format!(
                "{path}.receipt requires pre-registration, snapshot, and packet"
            )));
        };
        Some(validate_pilot_receipt(
            &attempt["receipt"],
            &format!("{path}.receipt"),
            registration,
            snapshot,
            &sessions,
            raw_sessions,
            raw_deviations,
            raw_custody,
            packet,
        )?)
    };
    let payload_present = registration.is_some()
        || snapshot.is_some()
        || receipt.is_some()
        || packet.is_some()
        || sensitivity.is_some()
        || !sessions.is_empty()
        || !raw_deviations.is_empty()
        || !raw_custody.is_empty();
    match status {
        "not-run" => {
            if void_reason.is_some() || control != "not-run" {
                return Err(ReaderError::new(format!(
                    "{path}: pre-run pilot cannot carry a void reason or control result"
                )));
            }
            if registration.is_none() || snapshot.is_none() {
                return Err(ReaderError::new(format!(
                    "{path}: recorded pre-run pilot requires frozen preregistration and snapshot"
                )));
            }
            if prerequisites.values().any(Value::is_null) {
                return Err(ReaderError::new(format!(
                    "{path}: recorded pre-run pilot requires all prerequisites"
                )));
            }
            if !sessions.is_empty()
                || !raw_deviations.is_empty()
                || !raw_custody.is_empty()
                || receipt.is_some()
                || packet.is_some()
                || sensitivity.is_some()
            {
                return Err(ReaderError::new(format!(
                    "{path}: pre-run pilot cannot carry as-run or decision evidence"
                )));
            }
        }
        "completed" => {
            if void_reason.is_some() {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot cannot carry a void reason"
                )));
            }
            let (
                Some(registration),
                Some(_snapshot),
                Some(receipt),
                Some(packet),
                Some(_sensitivity),
            ) = (registration, snapshot, receipt, packet, sensitivity)
            else {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires every frozen artifact"
                )));
            };
            if prerequisites.values().any(Value::is_null) {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires every prerequisite"
                )));
            }
            if sessions.is_empty()
                || !sessions
                    .iter()
                    .any(|record| record["admissibility"].as_str() == Some("admissible"))
            {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires admitted coded evidence"
                )));
            }
            if receipt["protocol_validity"].as_str() != Some("valid")
                || control != "watched-failing"
            {
                return Err(ReaderError::new(format!(
                    "{path}: completed pilot requires valid protocol and watched-failing control"
                )));
            }
            if receipt["completed_at"].as_str().unwrap()[..10]
                < registration["registered_date"].as_str().unwrap()[..]
            {
                return Err(ReaderError::new(format!(
                    "{path}: completion cannot precede preregistration"
                )));
            }
            let freeze = object(
                &registration["freeze_binding"],
                &format!("{path}.pre_registration.freeze_binding"),
            )?;
            if receipt["completed_at"].as_str().unwrap() <= freeze["frozen_at"].as_str().unwrap() {
                return Err(ReaderError::new(format!(
                    "{path}: pilot completion must follow the frozen preregistration"
                )));
            }
            let packet_freeze = object(
                &packet["freeze_binding"],
                &format!("{path}.decision_packet.freeze_binding"),
            )?;
            if packet_freeze["frozen_at"].as_str().unwrap()
                <= receipt["completed_at"].as_str().unwrap()
            {
                return Err(ReaderError::new(format!(
                    "{path}: decision packet freeze must strictly follow pilot completion"
                )));
            }
        }
        "void" => {
            if void_reason.is_none() || !payload_present {
                return Err(ReaderError::new(format!(
                    "{path}: void pilot requires a coded reason and preserved evidence"
                )));
            }
            if control == "watched-failing"
                && receipt.is_some_and(|value| value["protocol_validity"].as_str() == Some("valid"))
            {
                return Err(ReaderError::new(format!(
                    "{path}: fully valid pilot cannot be relabelled void"
                )));
            }
            if let Some(registration) = registration {
                let freeze = object(
                    &registration["freeze_binding"],
                    &format!("{path}.pre_registration.freeze_binding"),
                )?;
                if voided_at.unwrap() <= freeze["frozen_at"].as_str().unwrap() {
                    return Err(ReaderError::new(format!(
                        "{path}: pilot void time must follow the frozen preregistration"
                    )));
                }
            }
            if receipt
                .is_some_and(|value| value["completed_at"].as_str().unwrap() > voided_at.unwrap())
            {
                return Err(ReaderError::new(format!(
                    "{path}: pilot receipt completion cannot follow its void time"
                )));
            }
        }
        _ => unreachable!(),
    }
    let expected = canonical_sha(&Value::Object(attempt.clone()), Some("attempt_sha256"))?;
    let attempt_sha = digest(
        &attempt["attempt_sha256"],
        &format!("{path}.attempt_sha256"),
        Some(&expected),
    )?;
    Ok(PilotAttempt {
        attempt_id,
        status,
        packet,
        sensitivity,
        receipt,
        control,
        attempt_sha,
    })
}

struct PilotValidation<'a> {
    valid: bool,
    packet: Option<&'a Map<String, Value>>,
    sensitivity: Option<&'a Map<String, Value>>,
    active_id: Option<&'a str>,
}

fn validate_deviations<'a>(
    value: &'a Value,
    path: &str,
) -> ReaderResult<BTreeMap<&'a str, &'a Map<String, Value>>> {
    let mut result = BTreeMap::new();
    for (index, raw) in array(value, path)?.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, DEVIATION_KEYS, &item_path)?;
        let deviation_id = opaque_id(&item["deviation_id"], &format!("{item_path}.deviation_id"))?;
        if result.contains_key(deviation_id) {
            return Err(ReaderError::new(format!(
                "{item_path}.deviation_id: duplicate"
            )));
        }
        let code = opaque_id(&item["code"], &format!("{item_path}.code"))?;
        if !code.starts_with("RE-DEV-CODE-") {
            return Err(ReaderError::new(format!(
                "{item_path}.code: expected a closed RE-DEV-CODE-* value"
            )));
        }
        enumeration(
            &item["impact"],
            &["none", "session-inadmissible", "holdout-void"],
            &format!("{item_path}.impact"),
        )?;
        opaque_id(
            &item["custody_attestation_id"],
            &format!("{item_path}.custody_attestation_id"),
        )?;
        result.insert(deviation_id, item);
    }
    Ok(result)
}

fn validate_custody<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
) -> ReaderResult<BTreeMap<&'a str, &'a Map<String, Value>>> {
    let mut result = BTreeMap::new();
    let mut external_digests = HashSet::new();
    let refs = BTreeMap::from([
        ("session-record", "custody:READER-EVIDENCE-SESSION"),
        ("study-freshness", "custody:READER-EVIDENCE-FRESHNESS"),
        ("deviation", "custody:READER-EVIDENCE-DEVIATION"),
        ("commitment", "custody:READER-EVIDENCE-COMMITMENT"),
    ]);
    for (index, raw) in array(value, path)?.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, CUSTODY_KEYS, &item_path)?;
        let attestation_id = opaque_id(
            &item["attestation_id"],
            &format!("{item_path}.attestation_id"),
        )?;
        if result.contains_key(attestation_id) {
            return Err(ReaderError::new(format!(
                "{item_path}.attestation_id: duplicate"
            )));
        }
        opaque_id(&item["study_id"], &format!("{item_path}.study_id"))?;
        let scope = enumeration(
            &item["scope"],
            &[
                "session-record",
                "study-freshness",
                "deviation",
                "commitment",
            ],
            &format!("{item_path}.scope"),
        )?;
        if scope == "session-record" {
            digest(
                &item["record_commitment_sha256"],
                &format!("{item_path}.record_commitment_sha256"),
                None,
            )?;
        } else if !item["record_commitment_sha256"].is_null() {
            return Err(ReaderError::new(format!(
                "{item_path}.record_commitment_sha256: only session custody may bind a record"
            )));
        }
        let reference = text(&item["ref"], &format!("{item_path}.ref"))?;
        if refs.get(scope).copied() != Some(reference) {
            return Err(ReaderError::new(format!(
                "{item_path}.ref: custody scope requires its fixed external channel"
            )));
        }
        validate_external_or_repo_reference(env, &item["ref"], &format!("{item_path}.ref"))?;
        let external_digest = digest(&item["sha256"], &format!("{item_path}.sha256"), None)?;
        if !external_digests.insert(external_digest) {
            return Err(ReaderError::new(format!(
                "{item_path}.sha256: duplicate external attestation digest"
            )));
        }
        let freshness = boolean(
            &item["freshness_attested"],
            &format!("{item_path}.freshness_attested"),
        )?;
        if freshness && scope != "study-freshness" {
            return Err(ReaderError::new(format!(
                "{item_path}: only a study-freshness attestation may attest freshness"
            )));
        }
        let expected = canonical_sha(&Value::Object(item.clone()), Some("record_sha256"))?;
        digest(
            &item["record_sha256"],
            &format!("{item_path}.record_sha256"),
            Some(&expected),
        )?;
        result.insert(attestation_id, item);
    }
    Ok(result)
}

fn validate_record_links(
    sessions: &[&Map<String, Value>],
    deviations: &BTreeMap<&str, &Map<String, Value>>,
    custody: &BTreeMap<&str, &Map<String, Value>>,
    path: &str,
    expected_study_id: Option<&str>,
    commitment: Option<&Map<String, Value>>,
) -> ReaderResult<()> {
    let mut referenced = BTreeSet::new();
    for (attestation_id, item) in custody {
        if expected_study_id.is_some_and(|expected| item["study_id"].as_str() != Some(expected)) {
            return Err(ReaderError::new(format!(
                "{path}: custody attestation cites a different study"
            )));
        }
        if item["scope"].as_str() == Some("study-freshness") {
            referenced.insert(*attestation_id);
        }
    }
    for (deviation_id, deviation) in deviations {
        let attestation_id = deviation["custody_attestation_id"].as_str().unwrap();
        let item = custody.get(attestation_id);
        if item.is_none_or(|item| item["scope"].as_str() != Some("deviation")) {
            return Err(ReaderError::new(format!(
                "{path}: deviation {deviation_id} lacks deviation custody"
            )));
        }
        referenced.insert(attestation_id);
    }
    for session in sessions {
        for raw_id in array(&session["deviation_ids"], "session.deviation_ids")? {
            let deviation_id = raw_id.as_str().expect("validated session deviation ID");
            let Some(deviation) = deviations.get(deviation_id) else {
                return Err(ReaderError::new(format!(
                    "{path}: session cites an unknown deviation"
                )));
            };
            if session["admissibility"].as_str() == Some("admissible")
                && deviation["impact"].as_str() == Some("session-inadmissible")
            {
                return Err(ReaderError::new(format!(
                    "{path}: session-inadmissible deviation remains admitted"
                )));
            }
        }
        let mut matching_session_custody = false;
        for raw_id in array(
            &session["custody_attestation_ids"],
            "session.custody_attestation_ids",
        )? {
            let attestation_id = raw_id.as_str().expect("validated custody ID");
            let Some(item) = custody.get(attestation_id) else {
                return Err(ReaderError::new(format!(
                    "{path}: session cites unknown custody"
                )));
            };
            if item["scope"].as_str() == Some("session-record")
                && item["record_commitment_sha256"] == session["record_commitment_sha256"]
            {
                matching_session_custody = true;
            }
            referenced.insert(attestation_id);
        }
        if session["admissibility"].as_str() == Some("admissible") && !matching_session_custody {
            return Err(ReaderError::new(format!(
                "{path}: admitted session lacks matching record custody"
            )));
        }
        if session["admissibility"].as_str() == Some("inadmissible") {
            let has_exclusion = array(&session["deviation_ids"], "session.deviation_ids")?
                .iter()
                .filter_map(Value::as_str)
                .any(|identifier| {
                    deviations
                        .get(identifier)
                        .is_some_and(|item| item["impact"].as_str() == Some("session-inadmissible"))
                });
            if !has_exclusion {
                return Err(ReaderError::new(format!(
                    "{path}: inadmissible session lacks a coded exclusion deviation"
                )));
            }
        }
    }
    if let Some(commitment) = commitment {
        let matches: Vec<_> = custody
            .iter()
            .filter(|(_, item)| {
                item["scope"].as_str() == Some("commitment")
                    && item["sha256"] == commitment["custody_attestation_sha256"]
            })
            .map(|(identifier, _)| *identifier)
            .collect();
        if matches.len() != 1 {
            return Err(ReaderError::new(format!(
                "{path}: commitment must bind exactly one custody attestation"
            )));
        }
        referenced.insert(matches[0]);
    }
    if referenced != custody.keys().copied().collect() {
        return Err(ReaderError::new(format!(
            "{path}: every public custody record must have a closed evidence role"
        )));
    }
    Ok(())
}

fn validate_pilot_run_freshness(
    custody: &BTreeMap<&str, &Map<String, Value>>,
    path: &str,
    expected_study_id: Option<&str>,
    run_evidence: bool,
    require_attested: bool,
) -> ReaderResult<bool> {
    let freshness: Vec<_> = custody
        .values()
        .filter(|item| item["scope"].as_str() == Some("study-freshness"))
        .copied()
        .collect();
    if run_evidence && freshness.len() != 1 {
        return Err(ReaderError::new(format!(
            "{path}: pilot run evidence requires exactly one study-freshness custody attestation"
        )));
    }
    if !run_evidence {
        return Ok(false);
    }
    let item = freshness[0];
    if expected_study_id.is_some_and(|expected| item["study_id"].as_str() != Some(expected)) {
        return Err(ReaderError::new(format!(
            "{path}: pilot freshness custody cites a different study"
        )));
    }
    let attested = item["freshness_attested"].as_bool() == Some(true);
    if require_attested && !attested {
        return Err(ReaderError::new(format!(
            "{path}: a completed valid pilot requires freshness_attested true"
        )));
    }
    Ok(attested)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BigNat {
    // Base-ten digits, least significant first. This keeps all threshold
    // comparisons exact without imposing a machine-integer ceiling that the
    // canonical string contract does not have.
    digits: Vec<u8>,
}

impl BigNat {
    fn zero() -> Self {
        Self { digits: vec![0] }
    }

    fn from_decimal(value: &str) -> Self {
        let mut digits: Vec<_> = value.bytes().rev().map(|byte| byte - b'0').collect();
        while digits.len() > 1 && digits.last() == Some(&0) {
            digits.pop();
        }
        Self { digits }
    }

    fn from_usize(value: usize) -> Self {
        Self::from_decimal(&value.to_string())
    }

    fn is_zero(&self) -> bool {
        self.digits.len() == 1 && self.digits[0] == 0
    }

    fn add_assign(&mut self, other: &Self) {
        let mut carry = 0;
        let length = self.digits.len().max(other.digits.len());
        self.digits.resize(length, 0);
        for index in 0..length {
            let sum = self.digits[index] + other.digits.get(index).copied().unwrap_or(0) + carry;
            self.digits[index] = sum % 10;
            carry = sum / 10;
        }
        if carry != 0 {
            self.digits.push(carry);
        }
    }

    fn incremented(&self) -> Self {
        let mut result = self.clone();
        result.add_assign(&Self::from_usize(1));
        result
    }

    fn decremented(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let mut result = self.clone();
        let mut index = 0;
        loop {
            if result.digits[index] != 0 {
                result.digits[index] -= 1;
                break;
            }
            result.digits[index] = 9;
            index += 1;
        }
        while result.digits.len() > 1 && result.digits.last() == Some(&0) {
            result.digits.pop();
        }
        Some(result)
    }

    fn product(&self, other: &Self) -> Self {
        let mut accum = vec![0_u32; self.digits.len() + other.digits.len()];
        for (left_index, left) in self.digits.iter().copied().enumerate() {
            for (right_index, right) in other.digits.iter().copied().enumerate() {
                accum[left_index + right_index] += u32::from(left) * u32::from(right);
            }
        }
        for index in 0..accum.len() - 1 {
            let carry = accum[index] / 10;
            accum[index] %= 10;
            accum[index + 1] += carry;
        }
        while accum.last().copied().unwrap_or(0) >= 10 {
            let carry = accum.last().copied().unwrap() / 10;
            *accum.last_mut().unwrap() %= 10;
            accum.push(carry);
        }
        let mut digits: Vec<_> = accum.into_iter().map(|digit| digit as u8).collect();
        while digits.len() > 1 && digits.last() == Some(&0) {
            digits.pop();
        }
        Self { digits }
    }

    fn times_power_of_ten(&self, power: usize) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        let mut digits = vec![0; power];
        digits.extend_from_slice(&self.digits);
        Self { digits }
    }
}

impl Ord for BigNat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.digits
            .len()
            .cmp(&other.digits.len())
            .then_with(|| self.digits.iter().rev().cmp(other.digits.iter().rev()))
    }
}

impl PartialOrd for BigNat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for BigNat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for digit in self.digits.iter().rev() {
            write!(formatter, "{digit}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Observation {
    Numeric {
        numerator: BigNat,
        denominator: Option<BigNat>,
        eligible: BigNat,
    },
    Qualitative {
        token: &'static str,
        eligible: BigNat,
    },
}

fn policy_value<'a>(policies: &'a ThresholdPolicies, status: &str) -> &'a str {
    match status {
        "missing" => &policies.missing,
        "ambiguous" => &policies.ambiguous,
        "multiply-coded" => &policies.multiply_coded,
        "withdrawn" => &policies.withdrawn,
        "excluded" => &policies.excluded,
        "unclassified" => &policies.unclassified,
        _ => unreachable!("validated status"),
    }
}

fn resolved_policy_action<'a>(
    status: &str,
    adjudication: &str,
    policies: &'a ThresholdPolicies,
    final_statuses: &[&str],
) -> &'a str {
    if final_statuses.contains(&status) {
        return "final";
    }
    if adjudication == "unresolved" {
        return match policies.coder_adjudication.as_str() {
            "unresolved-count-adverse" => "count-adverse",
            "unresolved-exclude-observation" => "exclude-observation",
            "unresolved-not-evaluable" => "study-not-evaluable",
            _ => unreachable!("validated coder adjudication"),
        };
    }
    let action = policy_value(policies, status);
    if action == "require-adjudication" {
        "study-not-evaluable"
    } else {
        action
    }
}

fn metric_observation(
    spec: &ThresholdSpec,
    rule: &ThresholdRule,
    target_values: &BTreeMap<String, Vec<Option<bool>>>,
    misconception_values: &[BTreeMap<String, Option<(BigNat, BigNat)>>],
    misconceptions: &BTreeMap<String, &MisconceptionDefinition>,
) -> ReaderResult<Observation> {
    let scope: BTreeSet<_> = spec.scope_refs.iter().map(String::as_str).collect();
    if spec.metric == "admissible-session-count" {
        let count = BigNat::from_usize(misconception_values.len());
        return Ok(Observation::Numeric {
            numerator: count.clone(),
            denominator: None,
            eligible: count,
        });
    }
    if spec.metric.starts_with("target-identification-") {
        let target_id = scope.iter().next().expect("validated target scope");
        let included: Vec<_> = target_values[*target_id]
            .iter()
            .filter_map(|value| *value)
            .collect();
        let numerator = BigNat::from_usize(included.iter().filter(|value| **value).count());
        let eligible = BigNat::from_usize(included.len());
        return Ok(Observation::Numeric {
            numerator,
            denominator: spec.metric.ends_with("-rate").then(|| eligible.clone()),
            eligible,
        });
    }
    let misconception_ids: BTreeSet<_> = if spec.metric.starts_with("core-") {
        scope.into_iter().collect()
    } else {
        let severity_id = *scope.iter().next().expect("validated severity scope");
        misconceptions
            .iter()
            .filter(|(_, item)| item.severity_id == severity_id && !item.core)
            .map(|(identifier, _)| identifier.as_str())
            .collect()
    };
    let mut session_findings = BigNat::zero();
    let mut eligible_sessions = BigNat::zero();
    let mut occurrences = BigNat::zero();
    let mut opportunities = BigNat::zero();
    for session in misconception_values {
        let included: Vec<_> = misconception_ids
            .iter()
            .filter_map(|identifier| session.get(*identifier).and_then(Option::as_ref))
            .collect();
        if !included.is_empty() {
            eligible_sessions.add_assign(&BigNat::from_usize(1));
            if included.iter().any(|(count, _)| !count.is_zero()) {
                session_findings.add_assign(&BigNat::from_usize(1));
            }
            for (count, total) in included {
                occurrences.add_assign(count);
                opportunities.add_assign(total);
            }
        }
    }
    match spec.metric.as_str() {
        "core-finding-present" => {
            let eligible = if rule.repetition_unit == "admissible-session" {
                eligible_sessions
            } else {
                opportunities
            };
            Ok(Observation::Qualitative {
                token: if occurrences.is_zero() {
                    "absent"
                } else {
                    "present"
                },
                eligible,
            })
        }
        "core-finding-count" => {
            let session_unit = rule.repetition_unit == "admissible-session";
            Ok(Observation::Numeric {
                numerator: if session_unit {
                    session_findings
                } else {
                    occurrences
                },
                denominator: None,
                eligible: if session_unit {
                    eligible_sessions
                } else {
                    opportunities
                },
            })
        }
        "core-finding-rate" => {
            let session_unit = rule.repetition_unit == "admissible-session";
            let denominator = if session_unit {
                eligible_sessions
            } else {
                opportunities
            };
            Ok(Observation::Numeric {
                numerator: if session_unit {
                    session_findings
                } else {
                    occurrences
                },
                denominator: Some(denominator.clone()),
                eligible: denominator,
            })
        }
        "severity-session-finding-count" => Ok(Observation::Numeric {
            numerator: session_findings,
            denominator: None,
            eligible: eligible_sessions,
        }),
        "severity-session-finding-rate" => Ok(Observation::Numeric {
            numerator: session_findings,
            denominator: Some(eligible_sessions.clone()),
            eligible: eligible_sessions,
        }),
        "severity-occurrence-count" => Ok(Observation::Numeric {
            numerator: occurrences,
            denominator: None,
            eligible: opportunities,
        }),
        "severity-occurrence-rate" => Ok(Observation::Numeric {
            numerator: occurrences,
            denominator: Some(opportunities.clone()),
            eligible: opportunities,
        }),
        metric => Err(ReaderError::new(format!(
            "unsupported metric during evaluation: {metric}"
        ))),
    }
}

fn decimal_ratio(value: &str) -> (BigNat, usize) {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    (
        BigNat::from_decimal(&format!("{whole}{fraction}")),
        fraction.len(),
    )
}

fn threshold_comparison(operator: &str, left: &BigNat, right: &BigNat) -> ReaderResult<bool> {
    let ordering = left.cmp(right);
    match operator {
        "lt" => Ok(ordering.is_lt()),
        "lte" => Ok(!ordering.is_gt()),
        "gt" => Ok(ordering.is_gt()),
        "gte" => Ok(!ordering.is_lt()),
        "eq" => Ok(ordering.is_eq()),
        value => Err(ReaderError::new(format!(
            "unsupported threshold operator during evaluation: {value}"
        ))),
    }
}

fn compare_threshold(
    spec: &ThresholdSpec,
    observation: &Observation,
) -> ReaderResult<(Option<bool>, EvaluationCheck)> {
    let (observed, eligible) = match observation {
        Observation::Qualitative { token, eligible } => ((*token).to_owned(), eligible),
        Observation::Numeric {
            numerator,
            denominator,
            eligible,
        } => (
            denominator.as_ref().map_or_else(
                || numerator.to_string(),
                |denominator| format!("{numerator}/{denominator}"),
            ),
            eligible,
        ),
    };
    if eligible.is_zero() {
        return Ok((
            None,
            EvaluationCheck::Threshold {
                threshold_id: spec.threshold_id.clone(),
                metric: spec.metric.clone(),
                observed,
                comparison: None,
            },
        ));
    }
    let passed = match observation {
        Observation::Qualitative { token, .. } => *token == spec.value,
        Observation::Numeric {
            numerator,
            denominator,
            ..
        } => {
            let (boundary, scale) = decimal_ratio(&spec.value);
            let left = numerator.times_power_of_ten(scale);
            let right = denominator
                .as_ref()
                .map_or(boundary.clone(), |denominator| {
                    boundary.product(denominator)
                });
            threshold_comparison(&spec.operator, &left, &right)?
        }
    };
    Ok((
        Some(passed),
        EvaluationCheck::Threshold {
            threshold_id: spec.threshold_id.clone(),
            metric: spec.metric.clone(),
            observed,
            comparison: Some(passed),
        },
    ))
}

fn ordered_evaluation_trace(
    protocol_valid: bool,
    evaluable: bool,
    core_veto: bool,
    required_targets_pass: bool,
    non_core_pass: bool,
    mut checks: BTreeMap<&str, Vec<EvaluationCheck>>,
) -> EvaluationTrace {
    let mut reached = true;
    let mut verdict = "pass";
    let mut stages = Vec::new();
    for stage in EVALUATION_ORDER {
        if !reached {
            stages.push(EvaluationStage {
                stage: (*stage).to_owned(),
                status: "not-reached".to_owned(),
                checks: Vec::new(),
            });
            continue;
        }
        let passed = match *stage {
            "protocol-validity" => {
                if !protocol_valid {
                    verdict = "not-evaluable";
                }
                protocol_valid
            }
            "evaluability" => {
                if !evaluable {
                    verdict = "not-evaluable";
                }
                evaluable
            }
            "core-veto" => {
                if core_veto {
                    verdict = "fail";
                }
                !core_veto
            }
            "required-targets" => {
                if !required_targets_pass {
                    verdict = "fail";
                }
                required_targets_pass
            }
            "non-core-rules" => {
                if !non_core_pass {
                    verdict = "fail";
                }
                non_core_pass
            }
            "pass" => {
                verdict = "pass";
                true
            }
            _ => unreachable!(),
        };
        reached = passed;
        stages.push(EvaluationStage {
            stage: (*stage).to_owned(),
            status: if passed { "pass" } else { "fail" }.to_owned(),
            checks: checks.remove(stage).unwrap_or_default(),
        });
    }
    EvaluationTrace {
        order: EVALUATION_ORDER
            .iter()
            .map(|item| (*item).to_owned())
            .collect(),
        stages,
        verdict: verdict.to_owned(),
    }
}

pub(crate) fn evaluate_holdout(
    rule: &ThresholdRule,
    sessions: &[SessionRecord],
    protocol_validity: &str,
) -> ReaderResult<EvaluationTrace> {
    let protocol_valid = protocol_validity == "valid";
    let protocol_checks = vec![EvaluationCheck::Protocol {
        check: "protocol-validity".to_owned(),
        observed: protocol_validity.to_owned(),
        comparison: protocol_valid,
    }];
    if !protocol_valid {
        return Ok(ordered_evaluation_trace(
            false,
            false,
            false,
            false,
            false,
            BTreeMap::from([("protocol-validity", protocol_checks)]),
        ));
    }
    let misconceptions: BTreeMap<_, _> = rule
        .misconceptions
        .iter()
        .map(|item| (item.misconception_id.clone(), item))
        .collect();
    let admitted: Vec<_> = sessions
        .iter()
        .filter(|session| session.admissibility == "admissible")
        .collect();
    let mut target_values: BTreeMap<_, Vec<Option<bool>>> = REQUIRED_TARGETS
        .iter()
        .map(|(identifier, _)| ((*identifier).to_owned(), Vec::new()))
        .collect();
    let mut misconception_values = Vec::new();
    let mut issues = Vec::new();
    for session in admitted {
        for outcome in &session.target_outcomes {
            let action = resolved_policy_action(
                &outcome.status,
                &outcome.adjudication,
                &rule.policies,
                &["identified", "not-identified"],
            );
            let value = match action {
                "final" => Some(outcome.status == "identified"),
                "count-adverse" => Some(false),
                "exclude-observation" => None,
                _ => {
                    issues.push(format!("target-{}-not-evaluable", outcome.status));
                    None
                }
            };
            target_values
                .get_mut(&outcome.target_id)
                .expect("validated target")
                .push(value);
        }
        let mut values = BTreeMap::new();
        for outcome in &session.misconception_outcomes {
            let action = resolved_policy_action(
                &outcome.status,
                &outcome.adjudication,
                &rule.policies,
                &["absent", "present"],
            );
            let value = match action {
                "final" => Some((
                    BigNat::from_decimal(&outcome.occurrences),
                    BigNat::from_decimal(&outcome.opportunities),
                )),
                "count-adverse" => {
                    let opportunities = BigNat::from_decimal(&outcome.opportunities);
                    Some((opportunities.clone(), opportunities))
                }
                "exclude-observation" => None,
                _ => {
                    issues.push(format!("misconception-{}-not-evaluable", outcome.status));
                    None
                }
            };
            values.insert(outcome.misconception_id.clone(), value);
        }
        misconception_values.push(values);
    }
    let minimum_observation = metric_observation(
        &rule.minimum_evaluable_evidence,
        rule,
        &target_values,
        &misconception_values,
        &misconceptions,
    )?;
    let (minimum_pass, minimum_check) =
        compare_threshold(&rule.minimum_evaluable_evidence, &minimum_observation)?;
    let core_observation = metric_observation(
        &rule.core_failure_threshold,
        rule,
        &target_values,
        &misconception_values,
        &misconceptions,
    )?;
    let (core_veto, core_check) =
        compare_threshold(&rule.core_failure_threshold, &core_observation)?;
    let mut target_results = Vec::new();
    let mut target_checks = Vec::new();
    for item in &rule.required_target_thresholds {
        let observation = metric_observation(
            &item.threshold,
            rule,
            &target_values,
            &misconception_values,
            &misconceptions,
        )?;
        let (result, check) = compare_threshold(&item.threshold, &observation)?;
        target_results.push(result);
        target_checks.push(check);
    }
    let mut non_core_results = Vec::new();
    let mut non_core_checks = Vec::new();
    for item in &rule.non_core_thresholds {
        let observation = metric_observation(
            &item.threshold,
            rule,
            &target_values,
            &misconception_values,
            &misconceptions,
        )?;
        let (result, check) = compare_threshold(&item.threshold, &observation)?;
        non_core_results.push(result);
        non_core_checks.push(check);
    }
    let evaluable = issues.is_empty()
        && minimum_pass == Some(true)
        && std::iter::once(minimum_pass)
            .chain(std::iter::once(core_veto))
            .chain(target_results.iter().copied())
            .chain(non_core_results.iter().copied())
            .all(|result| result.is_some());
    let mut issue_counts = BTreeMap::new();
    for issue in issues {
        *issue_counts.entry(issue).or_insert(0_usize) += 1;
    }
    let mut evaluability_checks = vec![minimum_check];
    evaluability_checks.extend(
        issue_counts
            .into_iter()
            .map(|(issue, count)| EvaluationCheck::Issue { issue, count }),
    );
    Ok(ordered_evaluation_trace(
        true,
        evaluable,
        core_veto == Some(true),
        target_results.iter().all(|result| *result == Some(true)),
        non_core_results.iter().all(|result| *result == Some(true)),
        BTreeMap::from([
            ("protocol-validity", protocol_checks),
            ("evaluability", evaluability_checks),
            ("core-veto", vec![core_check]),
            ("required-targets", target_checks),
            ("non-core-rules", non_core_checks),
            ("pass", Vec::new()),
        ]),
    ))
}

fn typed_threshold_rule(value: &Value, path: &str) -> ReaderResult<ThresholdRule> {
    serde_json::from_value(value.clone())
        .map_err(|error| ReaderError::new(format!("{path}: invalid typed threshold rule: {error}")))
}

fn typed_reader_source(value: &Value, path: &str) -> ReaderResult<ReaderEvidenceSource> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ReaderError::new(format!(
            "{path}: invalid typed reader-evidence source: {error}"
        ))
    })
}

fn typed_reader_source_bytes(bytes: &[u8], path: &str) -> ReaderResult<ReaderEvidenceSource> {
    serde_json::from_slice(bytes).map_err(|error| {
        ReaderError::new(format!(
            "{path}: invalid typed reader-evidence source: {error}"
        ))
    })
}

fn typed_gate_input_bytes(bytes: &[u8], path: &str) -> ReaderResult<GateInput> {
    serde_json::from_slice(bytes)
        .map_err(|error| ReaderError::new(format!("{path}: typed contract mismatch: {error}")))
}

fn typed_sessions(value: &Value, path: &str) -> ReaderResult<Vec<SessionRecord>> {
    serde_json::from_value(value.clone()).map_err(|error| {
        ReaderError::new(format!("{path}: invalid typed session records: {error}"))
    })
}

fn validate_holdout_pre_registration_value<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    verify_live: bool,
    fixed_protocol_sha256: &str,
    expected_structural_checker_sha256: Option<&str>,
    expected_predecessor_attempt_sha256: Option<&str>,
    expected_prior_history_head_sha256: Option<&str>,
    enforce_history_binding: bool,
) -> ReaderResult<&'a Map<String, Value>> {
    let registration = object(value, path)?;
    exact_keys(registration, HOLDOUT_PRE_REGISTRATION_KEYS, path)?;
    opaque_id(&registration["study_id"], &format!("{path}.study_id"))?;
    let registered_date = date(
        &registration["registered_date"],
        &format!("{path}.registered_date"),
    )?;
    validate_preregistration_history_binding(
        registration,
        path,
        expected_predecessor_attempt_sha256,
        expected_prior_history_head_sha256,
        enforce_history_binding,
    )?;
    digest(
        &registration["fixed_protocol_sha256"],
        &format!("{path}.fixed_protocol_sha256"),
        Some(fixed_protocol_sha256),
    )?;
    for key in ["rule_sha256", "ratification_sha256", "evidence_gate_sha256"] {
        digest(&registration[key], &format!("{path}.{key}"), None)?;
    }
    digest(
        &registration["structural_checker_sha256"],
        &format!("{path}.structural_checker_sha256"),
        expected_structural_checker_sha256,
    )?;
    for key in [
        "revised_instrument",
        "rubric",
        "sample_rule",
        "recruitment_rule",
        "disclosure_set",
        "study_protocol",
    ] {
        validate_artifact(
            env,
            &registration[key],
            &format!("{path}.{key}"),
            verify_live,
        )?;
    }
    let candidate = object(
        &registration["release_candidate"],
        &format!("{path}.release_candidate"),
    )?;
    exact_keys(
        candidate,
        RELEASE_CANDIDATE_KEYS,
        &format!("{path}.release_candidate"),
    )?;
    opaque_id(
        &candidate["candidate_id"],
        &format!("{path}.release_candidate.candidate_id"),
    )?;
    let artifacts = array(
        &candidate["artifacts"],
        &format!("{path}.release_candidate.artifacts"),
    )?;
    if artifacts.is_empty() {
        return Err(ReaderError::new(format!(
            "{path}.release_candidate.artifacts: must not be empty"
        )));
    }
    let mut artifact_ids = HashSet::new();
    let mut artifact_refs = HashSet::new();
    for (index, raw) in artifacts.iter().enumerate() {
        let artifact = validate_artifact(
            env,
            raw,
            &format!("{path}.release_candidate.artifacts[{index}]"),
            verify_live,
        )?;
        let artifact_id = artifact["artifact_id"].as_str().unwrap();
        let artifact_ref = artifact["ref"].as_str().unwrap();
        if !artifact_ids.insert(artifact_id) || !artifact_refs.insert(artifact_ref) {
            return Err(ReaderError::new(format!(
                "{path}.release_candidate.artifacts[{index}]: duplicate identity or reference"
            )));
        }
    }
    let expected_manifest =
        canonical_sha(&Value::Object(candidate.clone()), Some("manifest_sha256"))?;
    digest(
        &candidate["manifest_sha256"],
        &format!("{path}.release_candidate.manifest_sha256"),
        Some(&expected_manifest),
    )?;
    if !registration["commitment"].is_null() {
        let commitment = object(&registration["commitment"], &format!("{path}.commitment"))?;
        exact_keys(commitment, COMMITMENT_KEYS, &format!("{path}.commitment"))?;
        opaque_id(
            &commitment["commitment_id"],
            &format!("{path}.commitment.commitment_id"),
        )?;
        for key in [
            "nonce_commitment_sha256",
            "committed_preimage_sha256",
            "custody_attestation_sha256",
        ] {
            digest(&commitment[key], &format!("{path}.commitment.{key}"), None)?;
        }
    }
    let binding = validate_freeze_binding(
        env,
        &registration["freeze_binding"],
        &format!("{path}.freeze_binding"),
        registration,
        "pre_registration_sha256",
        HistoricalPayloadKind::HoldoutPreRegistration,
    )?;
    if binding["frozen_at"].as_str().unwrap()[..10] < *registered_date {
        return Err(ReaderError::new(format!(
            "{path}.freeze_binding: freeze cannot precede registration"
        )));
    }
    let expected = canonical_sha(
        &Value::Object(registration.clone()),
        Some("pre_registration_sha256"),
    )?;
    digest(
        &registration["pre_registration_sha256"],
        &format!("{path}.pre_registration_sha256"),
        Some(&expected),
    )?;
    Ok(registration)
}

pub(crate) fn validate_holdout_pre_registration(
    context: &Context,
    protocol_decision: &[u8],
    registration: &HoldoutPreRegistration,
    fixed_protocol_sha256: &str,
    expected_structural_checker_sha256: Option<&str>,
    verify_live: bool,
) -> ReaderResult<()> {
    let env = ValidationEnv {
        context,
        protocol_decision,
        verify_live,
    };
    validate_holdout_pre_registration_typed(
        &env,
        registration,
        "pre_registration",
        verify_live,
        fixed_protocol_sha256,
        expected_structural_checker_sha256,
        None,
        None,
        false,
    )?;
    Ok(())
}

fn validate_frozen_holdout_payload(
    attempt_result: &str,
    sessions: &[&Map<String, Value>],
    deviations: &BTreeMap<&str, &Map<String, Value>>,
    custody: &BTreeMap<&str, &Map<String, Value>>,
    receipt_present: bool,
    commitment_reveal_present: bool,
    gate_receipt_present: bool,
    commitment: Option<&Map<String, Value>>,
    path: &str,
) -> ReaderResult<()> {
    if attempt_result != "not-run"
        || !sessions.is_empty()
        || !deviations.is_empty()
        || receipt_present
        || commitment_reveal_present
        || gate_receipt_present
    {
        return Err(ReaderError::new(format!(
            "{path}: frozen holdout cannot carry run evidence or a result"
        )));
    }
    let Some(commitment) = commitment else {
        if !custody.is_empty() {
            return Err(ReaderError::new(format!(
                "{path}: frozen holdout without a commitment cannot carry custody evidence"
            )));
        }
        return Ok(());
    };
    let matching = custody
        .values()
        .filter(|item| {
            item["scope"].as_str() == Some("commitment")
                && item["sha256"] == commitment["custody_attestation_sha256"]
        })
        .count();
    if custody.len() != 1 || matching != 1 {
        return Err(ReaderError::new(format!(
            "{path}: frozen private commitment requires exactly one matching commitment custody attestation"
        )));
    }
    Ok(())
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 {
        return None;
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some(((high << 4) | low) as u8)
        })
        .collect()
}

fn validate_commitment_reveal_value<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    commitment: Option<&Map<String, Value>>,
    custody: &BTreeMap<&str, &Map<String, Value>>,
    attempt_status: &str,
    reveal_required: bool,
    verify_live: bool,
    terminal_at: Option<&str>,
) -> ReaderResult<Option<&'a Map<String, Value>>> {
    let Some(commitment) = commitment else {
        if !value.is_null() {
            return Err(ReaderError::new(format!(
                "{path}: reveal cannot exist without a preregistered commitment"
            )));
        }
        return Ok(None);
    };
    if attempt_status == "frozen" {
        if !value.is_null() {
            return Err(ReaderError::new(format!(
                "{path}: nonce preimage cannot be revealed before the holdout ends"
            )));
        }
        return Ok(None);
    }
    if value.is_null() {
        if reveal_required {
            return Err(ReaderError::new(format!(
                "{path}: a completed or run-void holdout must reveal its commitment"
            )));
        }
        return Ok(None);
    }
    let reveal = object(value, path)?;
    exact_keys(reveal, COMMITMENT_REVEAL_KEYS, path)?;
    let commitment_id = opaque_id(&reveal["commitment_id"], &format!("{path}.commitment_id"))?;
    if commitment["commitment_id"].as_str() != Some(commitment_id) {
        return Err(ReaderError::new(format!(
            "{path}.commitment_id: does not open the preregistered commitment"
        )));
    }
    let revealed_at = utc_timestamp(&reveal["revealed_at"], &format!("{path}.revealed_at"))?;
    if terminal_at.is_none_or(|terminal| revealed_at <= terminal) {
        return Err(ReaderError::new(format!(
            "{path}.revealed_at: reveal must strictly follow the attempt terminal time"
        )));
    }
    let nonce_hex = text(&reveal["nonce_hex"], &format!("{path}.nonce_hex"))?;
    if !nonce_regex().is_match(nonce_hex) {
        return Err(ReaderError::new(format!(
            "{path}.nonce_hex: expected at least 32 bytes of lowercase hex"
        )));
    }
    let preimage = validate_artifact(
        env,
        &reveal["preimage"],
        &format!("{path}.preimage"),
        verify_live,
    )?;
    digest(
        &preimage["sha256"],
        &format!("{path}.preimage.sha256"),
        commitment["committed_preimage_sha256"].as_str(),
    )?;
    let mut opening = decode_hex(nonce_hex).expect("validated nonce");
    opening.push(0);
    opening.extend(
        decode_hex(preimage["sha256"].as_str().unwrap()).expect("validated artifact digest"),
    );
    let opening_digest = sha256(opening);
    digest(
        &commitment["nonce_commitment_sha256"],
        "holdout commitment nonce_commitment_sha256",
        Some(&opening_digest),
    )?;
    let custody_id = opaque_id(
        &reveal["custody_attestation_id"],
        &format!("{path}.custody_attestation_id"),
    )?;
    let custody_record = custody.get(custody_id);
    if custody_record.is_none_or(|record| {
        record["scope"].as_str() != Some("commitment")
            || record["sha256"] != commitment["custody_attestation_sha256"]
    }) {
        return Err(ReaderError::new(format!(
            "{path}: reveal lacks the exact commitment custody attestation"
        )));
    }
    let expected = canonical_sha(&Value::Object(reveal.clone()), Some("reveal_sha256"))?;
    digest(
        &reveal["reveal_sha256"],
        &format!("{path}.reveal_sha256"),
        Some(&expected),
    )?;
    Ok(Some(reveal))
}

fn validate_result_receipt_value<'a>(
    value: &'a Value,
    path: &str,
    registration: &Map<String, Value>,
    rule_value: &Value,
    rule: &ThresholdRule,
    sessions_value: &Value,
    sessions: &[SessionRecord],
    raw_deviations: &[Value],
    raw_custody: &[Value],
) -> ReaderResult<&'a Map<String, Value>> {
    let receipt = object(value, path)?;
    exact_keys(receipt, RESULT_RECEIPT_KEYS, path)?;
    opaque_id(&receipt["receipt_id"], &format!("{path}.receipt_id"))?;
    utc_timestamp(&receipt["completed_at"], &format!("{path}.completed_at"))?;
    if receipt["study_id"] != registration["study_id"] {
        return Err(ReaderError::new(format!(
            "{path}.study_id: does not match holdout preregistration"
        )));
    }
    let candidate = object(&registration["release_candidate"], "release candidate")?;
    let digest_links = [
        (
            "pre_registration_sha256",
            registration["pre_registration_sha256"]
                .as_str()
                .unwrap()
                .to_owned(),
        ),
        (
            "rule_sha256",
            registration["rule_sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "candidate_manifest_sha256",
            candidate["manifest_sha256"].as_str().unwrap().to_owned(),
        ),
        (
            "evidence_gate_sha256",
            registration["evidence_gate_sha256"]
                .as_str()
                .unwrap()
                .to_owned(),
        ),
        ("coded_records_sha256", canonical_sha(sessions_value, None)?),
        (
            "structural_checker_sha256",
            registration["structural_checker_sha256"]
                .as_str()
                .unwrap()
                .to_owned(),
        ),
        (
            "deviations_sha256",
            canonical_sha(&Value::Array(raw_deviations.to_vec()), None)?,
        ),
        (
            "custody_records_sha256",
            canonical_sha(&Value::Array(raw_custody.to_vec()), None)?,
        ),
    ];
    for (key, expected) in digest_links {
        digest(&receipt[key], &format!("{path}.{key}"), Some(&expected))?;
    }
    let protocol_validity = enumeration(
        &receipt["protocol_validity"],
        &["valid", "invalid"],
        &format!("{path}.protocol_validity"),
    )?;
    let verdict = enumeration(
        &receipt["verdict"],
        &["not-evaluable", "fail", "pass"],
        &format!("{path}.verdict"),
    )?;
    let trace = evaluate_holdout(rule, sessions, protocol_validity)?;
    let trace_value = serde_json::to_value(&trace)
        .map_err(|error| ReaderError::new(format!("cannot encode evaluation trace: {error}")))?;
    let trace_digest = canonical_sha(&trace_value, None)?;
    digest(
        &receipt["evaluation_trace_sha256"],
        &format!("{path}.evaluation_trace_sha256"),
        Some(&trace_digest),
    )?;
    if verdict != trace.verdict {
        return Err(ReaderError::new(format!(
            "{path}.verdict: differs from deterministic evaluation"
        )));
    }
    let classifications = Value::Array(
        sessions
            .iter()
            .map(|record| {
                serde_json::json!({
                    "record_commitment_sha256": record.record_commitment_sha256,
                    "admissibility": record.admissibility,
                })
            })
            .collect(),
    );
    let classification_digest = canonical_sha(&classifications, None)?;
    digest(
        &receipt["session_classification_sha256"],
        &format!("{path}.session_classification_sha256"),
        Some(&classification_digest),
    )?;
    let custody_digests = text_list(
        &receipt["custody_attestation_sha256s"],
        &format!("{path}.custody_attestation_sha256s"),
        true,
    )?;
    let expected_custody_digests: Vec<_> = raw_custody
        .iter()
        .map(|item| {
            object(item, "holdout custody record")
                .and_then(|record| digest(&record["sha256"], "holdout custody record.sha256", None))
        })
        .collect::<ReaderResult<_>>()?;
    if custody_digests != expected_custody_digests {
        return Err(ReaderError::new(format!(
            "{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )));
    }
    for (index, value) in custody_digests.iter().enumerate() {
        digest(
            &Value::String((*value).to_owned()),
            &format!("{path}.custody_attestation_sha256s[{index}]"),
            None,
        )?;
    }
    let expected = canonical_sha(&Value::Object(receipt.clone()), Some("receipt_sha256"))?;
    digest(
        &receipt["receipt_sha256"],
        &format!("{path}.receipt_sha256"),
        Some(&expected),
    )?;
    // Ensure the typed rule is exactly the rule whose receipt link was checked.
    if serde_json::to_value(rule)
        .map_err(|error| ReaderError::new(format!("cannot encode threshold rule: {error}")))?
        != *rule_value
    {
        return Err(ReaderError::new(format!(
            "{path}.rule_sha256: typed rule changed during evaluation"
        )));
    }
    Ok(receipt)
}

pub(crate) fn build_gate_input(
    attempt_id: String,
    rule: ThresholdRule,
    frozen_ratification: RatificationRecord,
    registration: HoldoutPreRegistration,
    sessions: Vec<SessionRecord>,
    deviations: Vec<DeviationRecord>,
    custody: Vec<CustodyRecord>,
    receipt: ResultReceipt,
    commitment_reveal: Option<CommitmentReveal>,
) -> GateInput {
    GateInput {
        schema_version: 1,
        attempt_id,
        active_attempt: true,
        attempt_status: "completed".to_owned(),
        current_rule_sha256: rule.rule_sha256.clone(),
        current_ratification_sha256: frozen_ratification.ratification_sha256.clone(),
        evidence_gate_sha256: registration.evidence_gate_sha256.clone(),
        structural_checker_sha256: registration.structural_checker_sha256.clone(),
        threshold_rule: rule,
        frozen_ratification,
        pre_registration: registration,
        session_records: sessions,
        deviations,
        custody_attestations: custody,
        result_receipt: receipt,
        commitment_reveal,
    }
}

fn decode_typed<T: serde::de::DeserializeOwned>(value: &Value, path: &str) -> ReaderResult<T> {
    serde_json::from_value(value.clone())
        .map_err(|error| ReaderError::new(format!("{path}: typed contract mismatch: {error}")))
}

pub(crate) fn evaluate_reader_evidence(
    context: &Context,
    protocol_decision: &[u8],
    envelope: &GateInput,
) -> ReaderResult<GateReceipt> {
    if envelope.schema_version != 1 {
        return Err(ReaderError::new(
            "gate input.schema_version must be integer 1",
        ));
    }
    if !opaque_id_regex().is_match(&envelope.attempt_id) {
        return Err(ReaderError::new(
            "gate input.attempt_id must be an opaque RE-* identifier",
        ));
    }
    if envelope.attempt_status != "completed" {
        return Err(ReaderError::new(
            "gate input.attempt_status must be completed",
        ));
    }
    let gate_path = EVIDENCE_GATE_REF.split_once("::").unwrap().0;
    let checker_path = STRUCTURAL_CHECKER_REF.split_once("::").unwrap().0;
    let actual_gate_digest = sha256(std::fs::read(context.path(gate_path)).map_err(|error| {
        ReaderError::new(format!(
            "cannot read bound executable {}: {error}",
            context.path(gate_path).display()
        ))
    })?);
    if envelope.evidence_gate_sha256 != actual_gate_digest {
        return Err(ReaderError::new(
            "gate input.evidence_gate_sha256: digest mismatch",
        ));
    }
    let actual_checker_digest =
        sha256(std::fs::read(context.path(checker_path)).map_err(|error| {
            ReaderError::new(format!(
                "cannot read bound executable {}: {error}",
                context.path(checker_path).display()
            ))
        })?);
    if envelope.structural_checker_sha256 != actual_checker_digest {
        return Err(ReaderError::new(
            "gate input.structural_checker_sha256: digest mismatch",
        ));
    }
    let known_misconceptions = validate_populated_threshold_rule_typed(&envelope.threshold_rule)?;
    let env = ValidationEnv {
        context,
        protocol_decision,
        verify_live: false,
    };
    let fixed_protocol_sha256 = validate_digest_str(
        &envelope.pre_registration.fixed_protocol_sha256,
        "gate input.pre_registration.fixed_protocol_sha256",
        None,
    )?;
    validate_frozen_ratification_typed(
        &env,
        &envelope.frozen_ratification,
        "gate input.frozen_ratification",
        &envelope.threshold_rule,
        fixed_protocol_sha256,
    )?;
    validate_holdout_pre_registration_typed(
        &env,
        &envelope.pre_registration,
        "gate input.pre_registration",
        false,
        fixed_protocol_sha256,
        Some(&actual_checker_digest),
        None,
        None,
        false,
    )?;
    for (key, declared, expected) in [
        (
            "rule_sha256",
            envelope.pre_registration.rule_sha256.as_str(),
            envelope.threshold_rule.rule_sha256.as_str(),
        ),
        (
            "ratification_sha256",
            envelope.pre_registration.ratification_sha256.as_str(),
            envelope.frozen_ratification.ratification_sha256.as_str(),
        ),
        (
            "evidence_gate_sha256",
            envelope.pre_registration.evidence_gate_sha256.as_str(),
            actual_gate_digest.as_str(),
        ),
    ] {
        validate_digest_str(
            declared,
            &format!("gate input.pre_registration.{key}"),
            Some(expected),
        )?;
    }
    if envelope.current_rule_sha256 != envelope.threshold_rule.rule_sha256 {
        return Err(ReaderError::new(
            "gate input.current_rule_sha256: digest mismatch",
        ));
    }
    if envelope.current_ratification_sha256 != envelope.frozen_ratification.ratification_sha256 {
        return Err(ReaderError::new(
            "gate input.current_ratification_sha256: digest mismatch",
        ));
    }
    let study_id = envelope.pre_registration.study_id.as_str();
    validate_sessions_typed(
        &envelope.session_records,
        "gate input.session_records",
        Some(study_id),
        Some(&known_misconceptions),
    )?;
    let deviations = validate_deviations_typed(&envelope.deviations, "gate input.deviations")?;
    let custody = validate_custody_typed(
        &env,
        &envelope.custody_attestations,
        "gate input.custody_attestations",
    )?;
    validate_record_links_typed(
        &envelope.session_records,
        &deviations,
        &custody,
        "gate input.record_links",
        Some(study_id),
        envelope.pre_registration.commitment.as_ref(),
    )?;
    validate_result_receipt_typed(
        &envelope.result_receipt,
        "gate input.result_receipt",
        &envelope.pre_registration,
        &envelope.threshold_rule,
        &envelope.session_records,
        &envelope.deviations,
        &envelope.custody_attestations,
    )?;
    let completed_at = envelope.result_receipt.completed_at.as_str();
    if completed_at <= envelope.pre_registration.freeze_binding.frozen_at.as_str() {
        return Err(ReaderError::new(
            "gate input completion must strictly follow the frozen preregistration",
        ));
    }
    if &completed_at[..10] < envelope.pre_registration.registered_date.as_str() {
        return Err(ReaderError::new(
            "gate input completion cannot precede the registration date",
        ));
    }
    validate_commitment_reveal_typed(
        &env,
        envelope.commitment_reveal.as_ref(),
        "gate input.commitment_reveal",
        envelope.pre_registration.commitment.as_ref(),
        &custody,
        "completed",
        true,
        false,
        Some(completed_at),
    )?;
    let trace = evaluate_holdout(
        &envelope.threshold_rule,
        &envelope.session_records,
        &envelope.result_receipt.protocol_validity,
    )?;
    let freshness: Vec<_> = custody
        .values()
        .filter(|item| item.scope == "study-freshness")
        .collect();
    let admitted = envelope.active_attempt
        && envelope.result_receipt.protocol_validity == "valid"
        && envelope.result_receipt.verdict == "pass"
        && trace.verdict == "pass"
        && freshness.len() == 1
        && freshness[0].freshness_attested;
    let mut output = GateReceipt {
        schema_version: 1,
        input_sha256: typed_canonical_sha(envelope, "gate input", None)?,
        evidence_gate_sha256: actual_gate_digest,
        decision: if admitted { "admit" } else { "reject" }.to_owned(),
        receipt_sha256: String::new(),
    };
    output.receipt_sha256 = typed_canonical_sha(&output, "gate output", Some("receipt_sha256"))?;
    Ok(output)
}

pub(crate) fn evaluate_gate_json(
    context: &Context,
    protocol_decision: &[u8],
    input: &[u8],
) -> Result<String, Error> {
    let result = (|| -> ReaderResult<String> {
        if input.is_empty() {
            return Err(ReaderError::new(
                "--evaluate requires one JSON object on standard input",
            ));
        }
        parse_unique_json(input, "gate input")?;
        let envelope = typed_gate_input_bytes(input, "gate input")?;
        let output = evaluate_reader_evidence(context, protocol_decision, &envelope)?;
        let output = serde_json::to_value(output)
            .map_err(|error| ReaderError::new(format!("cannot encode gate output: {error}")))?;
        String::from_utf8(canonical_json(&output))
            .map(|value| value + "\n")
            .map_err(|error| ReaderError::new(format!("cannot encode gate output: {error}")))
    })();
    result.map_err(|error| Error::new(format!("reader-evidence-admission-gate: {error}")))
}

pub(crate) fn admission_gate_self_test(context: &Context) -> Result<String, Error> {
    native_gate_self_test(context)
        .map(|()| "reader-evidence-admission-gate: self-test passed".to_owned())
        .map_err(|error| Error::new(format!("reader-evidence-admission-gate: {error}")))
}

fn validate_gate_admission_receipt_value<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    gate_input: &GateInput,
    expected_decision: &str,
    execute_live: bool,
) -> ReaderResult<&'a Map<String, Value>> {
    let receipt = object(value, path)?;
    exact_keys(receipt, GATE_ADMISSION_RECEIPT_KEYS, path)?;
    if receipt["schema_version"].as_u64() != Some(1) {
        return Err(ReaderError::new(format!(
            "{path}.schema_version must be integer 1"
        )));
    }
    let gate_input_value = serde_json::to_value(gate_input)
        .map_err(|error| ReaderError::new(format!("cannot encode gate input: {error}")))?;
    let input_digest = canonical_sha(&gate_input_value, None)?;
    digest(
        &receipt["input_sha256"],
        &format!("{path}.input_sha256"),
        Some(&input_digest),
    )?;
    digest(
        &receipt["evidence_gate_sha256"],
        &format!("{path}.evidence_gate_sha256"),
        Some(&gate_input.evidence_gate_sha256),
    )?;
    let decision = enumeration(
        &receipt["decision"],
        &["admit", "reject"],
        &format!("{path}.decision"),
    )?;
    if decision != expected_decision {
        return Err(ReaderError::new(format!(
            "{path}.decision must be {expected_decision} for the validated result"
        )));
    }
    let expected_receipt = canonical_sha(&Value::Object(receipt.clone()), Some("receipt_sha256"))?;
    digest(
        &receipt["receipt_sha256"],
        &format!("{path}.receipt_sha256"),
        Some(&expected_receipt),
    )?;
    if execute_live {
        let live = evaluate_reader_evidence(env.context, env.protocol_decision, gate_input)?;
        let live_value = serde_json::to_value(live).map_err(|error| {
            ReaderError::new(format!("cannot encode live gate output: {error}"))
        })?;
        if live_value != *value {
            return Err(ReaderError::new(format!(
                "{path}: stored receipt differs from the bound gate output"
            )));
        }
    }
    Ok(receipt)
}

fn validate_pilot<'a>(
    env: &ValidationEnv<'_>,
    source: &'a Map<String, Value>,
) -> ReaderResult<PilotValidation<'a>> {
    let pilot = object(&source["pilot"], "pilot")?;
    exact_keys(pilot, PILOT_KEYS, "pilot")?;
    let summary_status = enumeration(
        &pilot["pilot_status"],
        &["not-run", "completed", "void"],
        "pilot.pilot_status",
    )?;
    let summary_control = enumeration(
        &pilot["control_status"],
        &[
            "not-run",
            "watched-failing",
            "failed-to-fail",
            "indeterminate",
        ],
        "pilot.control_status",
    )?;
    let active_attempt_id = if pilot["active_attempt_id"].is_null() {
        None
    } else {
        Some(opaque_id(
            &pilot["active_attempt_id"],
            "pilot.active_attempt_id",
        )?)
    };
    let attempts = array(&pilot["attempts"], "pilot.attempts")?;
    if attempts.is_empty() {
        if active_attempt_id.is_some() {
            return Err(ReaderError::new(
                "pilot.active_attempt_id requires an attempt",
            ));
        }
        if summary_status != "not-run" || summary_control != "not-run" {
            return Err(ReaderError::new(
                "empty pilot history must remain not-run/not-run",
            ));
        }
        return Ok(PilotValidation {
            valid: false,
            packet: None,
            sensitivity: None,
            active_id: None,
        });
    }
    let mut seen_ids = HashSet::new();
    let mut previous_sha = None;
    let mut attempt_shas = Vec::new();
    let mut active = None;
    for (index, raw) in attempts.iter().enumerate() {
        let prior_head = history_head_sha256(attempt_shas.iter().copied(), std::iter::empty());
        let attempt = validate_pilot_attempt(
            env,
            raw,
            &format!("pilot.attempts[{index}]"),
            source,
            previous_sha,
            &prior_head,
            index == 0,
            index + 1 == attempts.len(),
        )?;
        if !seen_ids.insert(attempt.attempt_id) {
            return Err(ReaderError::new(format!(
                "pilot.attempts[{index}].attempt_id: duplicate"
            )));
        }
        previous_sha = Some(attempt.attempt_sha);
        attempt_shas.push(attempt.attempt_sha);
        active = Some(attempt);
    }
    let active = active.expect("nonempty attempts");
    if active_attempt_id != Some(active.attempt_id) {
        return Err(ReaderError::new(
            "pilot.active_attempt_id must identify the final append-only attempt",
        ));
    }
    if summary_status != active.status || summary_control != active.control {
        return Err(ReaderError::new(
            "pilot summary status/control must equal the active attempt",
        ));
    }
    let valid = active.status == "completed"
        && active.control == "watched-failing"
        && active
            .receipt
            .is_some_and(|receipt| receipt["protocol_validity"].as_str() == Some("valid"));
    Ok(PilotValidation {
        valid,
        packet: active.packet,
        sensitivity: active.sensitivity,
        active_id: Some(active.attempt_id),
    })
}

fn validate_ratification(
    env: &ValidationEnv<'_>,
    source: &Map<String, Value>,
    rule: &Map<String, Value>,
    pilot: &PilotValidation<'_>,
) -> ReaderResult<()> {
    let status = source["threshold_status"].as_str().unwrap_or_default();
    if matches!(status, "candidate" | "pending-pilot") {
        if !source["ratification"].is_null() {
            return Err(ReaderError::new(format!(
                "{status} threshold may not carry ratification"
            )));
        }
        return Ok(());
    }
    let record = object(&source["ratification"], "ratification")?;
    exact_keys(record, RATIFICATION_KEYS, "ratification")?;
    let ruling_id = opaque_id(&record["ruling_id"], "ratification.ruling_id")?;
    let bound_attempt_id = opaque_id(&record["pilot_attempt_id"], "ratification.pilot_attempt_id")?;
    if pilot.active_id != Some(bound_attempt_id) {
        return Err(ReaderError::new(
            "ratification must bind the active valid pilot attempt",
        ));
    }
    date(&record["ratified_date"], "ratification.ratified_date")?;
    let candidate_commit = text(&record["candidate_commit"], "ratification.candidate_commit")?;
    if !commit_regex().is_match(candidate_commit) {
        return Err(ReaderError::new(
            "ratification.candidate_commit must be a full lowercase commit digest",
        ));
    }
    for key in ["author_statement", "question_answered", "rationale"] {
        text(&record[key], &format!("ratification.{key}"))?;
    }
    let (Some(packet), Some(sensitivity)) = (pilot.packet, pilot.sensitivity) else {
        return Err(ReaderError::new(
            "ratification requires the frozen pilot packet and sensitivity brief",
        ));
    };
    digest(
        &record["pilot_packet_sha256"],
        "ratification.pilot_packet_sha256",
        packet["packet_sha256"].as_str(),
    )?;
    digest(
        &record["sensitivity_brief_sha256"],
        "ratification.sensitivity_brief_sha256",
        sensitivity["sha256"].as_str(),
    )?;
    digest(
        &record["rule_sha256"],
        "ratification.rule_sha256",
        rule["rule_sha256"].as_str(),
    )?;
    validate_candidate_commit(
        env,
        candidate_commit,
        rule["rule_sha256"].as_str().unwrap(),
        bound_attempt_id,
        packet["packet_sha256"].as_str().unwrap(),
        sensitivity["sha256"].as_str().unwrap(),
        &canonical_sha_omitting(&source["protocol"], &["decision_sha256"])?,
    )?;
    if record["ratified_date"].as_str().unwrap() < packet["frozen_date"].as_str().unwrap() {
        return Err(ReaderError::new(
            "ratification must follow the frozen pilot decision packet",
        ));
    }
    let decision_ref =
        validate_repo_reference(env, &record["decision_ref"], "ratification.decision_ref")?;
    if !decision_ref.starts_with(&format!("{PROTOCOL_DECISION}::")) {
        return Err(ReaderError::new(
            "ratification.decision_ref must cite the controlling decision record",
        ));
    }
    if !record["decision_ref"].as_str().unwrap().contains(ruling_id) {
        return Err(ReaderError::new(
            "ratification.decision_ref must cite the exact second ruling anchor",
        ));
    }
    if record["no_holdout_evidence_attestation"].as_bool() != Some(true) {
        return Err(ReaderError::new(
            "ratification must attest no holdout evidence existed or was inspected",
        ));
    }
    let expected = canonical_sha(&Value::Object(record.clone()), Some("ratification_sha256"))?;
    digest(
        &record["ratification_sha256"],
        "ratification.ratification_sha256",
        Some(&expected),
    )?;
    Ok(())
}

fn committed_file_bytes(
    context: &Context,
    commit: &str,
    relative: &str,
    path: &str,
) -> ReaderResult<Vec<u8>> {
    let object_spec = format!("{commit}:{relative}");
    let shown = git_output(context, &["show", &object_spec], path)?;
    if !shown.status.success() {
        let detail = String::from_utf8_lossy(&shown.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "{path}: candidate commit artifact is unavailable{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        )));
    }
    Ok(shown.stdout)
}

fn validate_candidate_relevant_state(
    context: &Context,
    candidate: &Value,
    candidate_commit: &str,
    decision_bytes: &[u8],
    checker_bytes: &[u8],
    candidate_raw: &[u8],
    valid_pilot: bool,
) -> ReaderResult<()> {
    let source = object(candidate, "ratification candidate source")?;
    exact_keys(source, ROOT_KEYS, "ratification candidate source")?;
    walk_keys(candidate, "ratification candidate source")?;
    let offline_env = ValidationEnv {
        context,
        protocol_decision: decision_bytes,
        verify_live: false,
    };
    validate_protocol(&offline_env, source)?;
    validate_privacy(source)?;
    let route = object(&source["route"], "ratification candidate route")?;
    if !route["reviewer_custody_attestation"].is_null()
        || !route["evidence_admission_gate_binding"].is_null()
    {
        return Err(ReaderError::new(
            "ratification candidate route must precede reviewer and gate availability bindings",
        ));
    }
    let route = validate_route_readiness(
        &offline_env,
        source,
        valid_pilot,
        Some(&sha256(checker_bytes)),
    )?;
    validate_claim(&offline_env, source, route.status, false)?;
    validate_acceptance(source)?;
    validate_history_closure_at(context, source, candidate_raw, Some(candidate_commit))
}

fn validate_candidate_commit(
    env: &ValidationEnv<'_>,
    candidate_commit: &str,
    expected_rule_sha256: &str,
    expected_pilot_attempt_id: &str,
    expected_packet_sha256: &str,
    expected_sensitivity_sha256: &str,
    expected_fixed_protocol_sha256: &str,
) -> ReaderResult<Value> {
    let ancestor = git_output(
        env.context,
        &["merge-base", "--is-ancestor", candidate_commit, "HEAD"],
        "ratification.candidate_commit",
    )?;
    if !ancestor.status.success() {
        return Err(ReaderError::new(
            "ratification.candidate_commit must be an ancestor of the current checkout",
        ));
    }
    let candidate_spec = format!("{candidate_commit}:{DEFAULT_SOURCE}");
    let completed = git_output(
        env.context,
        &["show", &candidate_spec],
        "ratification.candidate_commit",
    )?;
    if !completed.status.success() {
        let detail = String::from_utf8_lossy(&completed.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "ratification.candidate_commit has no candidate source: {detail}"
        )));
    }
    let candidate = parse_source(&completed.stdout).map_err(|_| {
        ReaderError::new("ratification.candidate_commit contains invalid candidate JSON")
    })?;
    let candidate_source = object(&candidate, "ratification candidate source")?;
    let candidate_decision = committed_file_bytes(
        env.context,
        candidate_commit,
        PROTOCOL_DECISION,
        "ratification.candidate_commit.protocol_decision",
    )?;
    let checker_path = STRUCTURAL_CHECKER_REF.split_once("::").unwrap().0;
    let candidate_checker = committed_file_bytes(
        env.context,
        candidate_commit,
        checker_path,
        "ratification.candidate_commit.structural_checker",
    )?;
    exact_keys(candidate_source, ROOT_KEYS, "ratification candidate source")?;
    walk_keys(&candidate, "ratification candidate source")?;
    let candidate_env = ValidationEnv {
        context: env.context,
        protocol_decision: &candidate_decision,
        verify_live: true,
    };
    validate_protocol(&candidate_env, candidate_source)?;
    validate_privacy(candidate_source)?;
    if candidate_source["threshold_status"].as_str() != Some("candidate") {
        return Err(ReaderError::new(
            "ratification.candidate_commit must record candidate threshold status",
        ));
    }
    if !candidate_source["ratification"].is_null() {
        return Err(ReaderError::new(
            "ratification.candidate_commit must precede author ratification",
        ));
    }
    if candidate_source["holdout_status"].as_str() != Some("not-frozen")
        || candidate_source["result"].as_str() != Some("not-run")
    {
        return Err(ReaderError::new(
            "ratification.candidate_commit may contain no holdout result",
        ));
    }
    let holdout = object(
        &candidate_source["holdout"],
        "ratification candidate holdout",
    )?;
    exact_keys(holdout, HOLDOUT_KEYS, "ratification candidate holdout")?;
    if !holdout["active_attempt_id"].is_null()
        || !array(
            &holdout["attempts"],
            "ratification candidate holdout.attempts",
        )?
        .is_empty()
    {
        return Err(ReaderError::new(
            "ratification.candidate_commit may contain no holdout attempt",
        ));
    }
    let candidate_fixed =
        canonical_sha_omitting(&candidate_source["protocol"], &["decision_sha256"])?;
    if candidate_fixed != expected_fixed_protocol_sha256 {
        return Err(ReaderError::new(
            "ratification.candidate_commit fixed protocol differs from the ratified basis",
        ));
    }
    let pilot = validate_pilot(&candidate_env, candidate_source)?;
    if !pilot.valid
        || pilot.active_id != Some(expected_pilot_attempt_id)
        || pilot
            .packet
            .is_none_or(|packet| packet["packet_sha256"].as_str() != Some(expected_packet_sha256))
        || pilot
            .sensitivity
            .is_none_or(|artifact| artifact["sha256"].as_str() != Some(expected_sensitivity_sha256))
    {
        return Err(ReaderError::new(
            "ratification.candidate_commit does not contain the same fully validated pilot basis",
        ));
    }
    validate_candidate_relevant_state(
        env.context,
        &candidate,
        candidate_commit,
        &candidate_decision,
        &candidate_checker,
        &completed.stdout,
        pilot.valid,
    )?;
    let _misconceptions = validate_threshold_rule(&candidate, pilot.valid)?;
    let candidate_rule = object(
        &candidate_source["threshold_rule"],
        "ratification candidate threshold_rule",
    )?;
    let actual_rule_sha =
        canonical_sha(&Value::Object(candidate_rule.clone()), Some("rule_sha256"))?;
    digest(
        &candidate_rule["rule_sha256"],
        "ratification candidate threshold_rule.rule_sha256",
        Some(&actual_rule_sha),
    )?;
    if actual_rule_sha != expected_rule_sha256 {
        return Err(ReaderError::new(
            "ratification.candidate_commit rule differs from the ratified rule",
        ));
    }
    let attempts = array(
        &object(&candidate_source["pilot"], "ratification candidate pilot")?["attempts"],
        "ratification candidate pilot.attempts",
    )?;
    let active = attempts
        .last()
        .ok_or_else(|| ReaderError::new("ratification candidate has no active pilot"))?;
    let active = object(active, "ratification candidate active pilot attempt")?;
    if active["attempt_id"].as_str() != Some(expected_pilot_attempt_id)
        || active["attempt_status"].as_str() != Some("completed")
        || active["control_status"].as_str() != Some("watched-failing")
    {
        return Err(ReaderError::new(
            "ratification candidate active pilot is not valid and completed",
        ));
    }
    let packet = object(
        &active["decision_packet"],
        "ratification candidate pilot packet",
    )?;
    if packet["packet_sha256"].as_str() != Some(expected_packet_sha256) {
        return Err(ReaderError::new(
            "ratification candidate cites a different pilot packet",
        ));
    }
    Ok(candidate)
}

fn validate_frozen_ratification_value<'a>(
    env: &ValidationEnv<'_>,
    value: &'a Value,
    path: &str,
    rule: &Map<String, Value>,
    fixed_protocol_sha256: &str,
) -> ReaderResult<&'a Map<String, Value>> {
    let record = object(value, path)?;
    exact_keys(record, RATIFICATION_KEYS, path)?;
    let ruling_id = opaque_id(&record["ruling_id"], &format!("{path}.ruling_id"))?;
    let pilot_attempt_id = opaque_id(
        &record["pilot_attempt_id"],
        &format!("{path}.pilot_attempt_id"),
    )?;
    let ratified_date = date(&record["ratified_date"], &format!("{path}.ratified_date"))?;
    let candidate_commit = text(
        &record["candidate_commit"],
        &format!("{path}.candidate_commit"),
    )?;
    if !commit_regex().is_match(candidate_commit) {
        return Err(ReaderError::new(format!(
            "{path}.candidate_commit must be a full lowercase commit digest"
        )));
    }
    for key in ["author_statement", "question_answered", "rationale"] {
        text(&record[key], &format!("{path}.{key}"))?;
    }
    let packet_sha = digest(
        &record["pilot_packet_sha256"],
        &format!("{path}.pilot_packet_sha256"),
        None,
    )?;
    let sensitivity_sha = digest(
        &record["sensitivity_brief_sha256"],
        &format!("{path}.sensitivity_brief_sha256"),
        None,
    )?;
    digest(
        &record["rule_sha256"],
        &format!("{path}.rule_sha256"),
        rule["rule_sha256"].as_str(),
    )?;
    let candidate = validate_candidate_commit(
        env,
        candidate_commit,
        rule["rule_sha256"].as_str().unwrap(),
        pilot_attempt_id,
        packet_sha,
        sensitivity_sha,
        fixed_protocol_sha256,
    )?;
    let candidate_source = object(&candidate, "frozen ratification candidate")?;
    let pilot = object(
        &candidate_source["pilot"],
        "frozen ratification candidate.pilot",
    )?;
    let attempts = array(
        &pilot["attempts"],
        "frozen ratification candidate.pilot.attempts",
    )?;
    let active = object(
        attempts.last().unwrap(),
        "frozen ratification candidate active pilot",
    )?;
    let packet = object(
        &active["decision_packet"],
        "frozen ratification candidate packet",
    )?;
    if ratified_date < packet["frozen_date"].as_str().unwrap() {
        return Err(ReaderError::new(format!(
            "{path}: ratification must follow its frozen pilot decision packet"
        )));
    }
    let decision_ref = validate_repo_reference(
        env,
        &record["decision_ref"],
        &format!("{path}.decision_ref"),
    )?;
    if !decision_ref.starts_with(&format!("{PROTOCOL_DECISION}::")) {
        return Err(ReaderError::new(format!(
            "{path}.decision_ref must cite the controlling decision record"
        )));
    }
    if !decision_ref.contains(ruling_id) {
        return Err(ReaderError::new(format!(
            "{path}.decision_ref must cite the exact ruling anchor"
        )));
    }
    if record["no_holdout_evidence_attestation"].as_bool() != Some(true) {
        return Err(ReaderError::new(format!(
            "{path} must attest no holdout evidence existed or was inspected"
        )));
    }
    let expected = canonical_sha(&Value::Object(record.clone()), Some("ratification_sha256"))?;
    digest(
        &record["ratification_sha256"],
        &format!("{path}.ratification_sha256"),
        Some(&expected),
    )?;
    Ok(record)
}

fn native_gate_self_test(context: &Context) -> ReaderResult<()> {
    let malformed = serde_json::json!({"schema_version": 1});
    if gate_input_shape(&malformed).is_ok() {
        return Err(ReaderError::new(
            "route evidence gate malformed-envelope self-test did not fail closed",
        ));
    }
    let checker_path = STRUCTURAL_CHECKER_REF.split_once("::").unwrap().0;
    let checker_digest = sha256(std::fs::read(context.path(checker_path)).map_err(|error| {
        ReaderError::new(format!(
            "cannot read bound executable {}: {error}",
            context.path(checker_path).display()
        ))
    })?);
    if validate_digest_str(
        &"0".repeat(64),
        "self-test.structural_checker_sha256",
        Some(&checker_digest),
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "dependency-digest mismatch self-test did not fail closed",
        ));
    }
    Ok(())
}

fn gate_input_shape(value: &Value) -> ReaderResult<&Map<String, Value>> {
    const INPUT_KEYS: &[&str] = &[
        "schema_version",
        "attempt_id",
        "active_attempt",
        "attempt_status",
        "threshold_rule",
        "frozen_ratification",
        "pre_registration",
        "session_records",
        "deviations",
        "custody_attestations",
        "result_receipt",
        "commitment_reveal",
        "current_rule_sha256",
        "current_ratification_sha256",
        "evidence_gate_sha256",
        "structural_checker_sha256",
    ];
    let envelope = object(value, "gate input")?;
    exact_keys(envelope, INPUT_KEYS, "gate input")?;
    Ok(envelope)
}

struct RouteValidation<'a> {
    status: &'a str,
    gate_sha256: Option<&'a str>,
    checker_sha256: &'a str,
}

fn validate_route_readiness<'a>(
    env: &ValidationEnv<'_>,
    source: &'a Map<String, Value>,
    valid_pilot: bool,
    expected_structural_checker_sha256: Option<&str>,
) -> ReaderResult<RouteValidation<'a>> {
    let route = object(&source["route"], "route")?;
    exact_keys(route, ROUTE_KEYS, "route")?;
    if route["route_id"].as_str() != Some("FS-RTE-06") {
        return Err(ReaderError::new("route.route_id must be FS-RTE-06"));
    }
    let route_status = enumeration(
        &route["route_status"],
        &["unbuilt", "available"],
        "route.route_status",
    )?;
    let evidence_status = enumeration(
        &route["evidence_contract_status"],
        &["unbuilt", "implemented"],
        "route.evidence_contract_status",
    )?;
    if evidence_status != "implemented" {
        return Err(ReaderError::new(
            "route.evidence_contract_status must record this implemented contract",
        ));
    }
    let structural = validate_artifact(
        env,
        &route["structural_checker_binding"],
        "route.structural_checker_binding",
        true,
    )?;
    if structural["artifact_id"].as_str() != Some(STRUCTURAL_CHECKER_ARTIFACT_ID)
        || structural["ref"].as_str() != Some(STRUCTURAL_CHECKER_REF)
    {
        return Err(ReaderError::new(
            "route.structural_checker_binding must bind the fixed structural checker",
        ));
    }
    if let Some(expected) = expected_structural_checker_sha256 {
        digest(
            &structural["sha256"],
            "route.structural_checker_binding.sha256",
            Some(expected),
        )?;
    }
    let gate_sha256 = if route["evidence_admission_gate_binding"].is_null() {
        None
    } else {
        let gate = validate_artifact(
            env,
            &route["evidence_admission_gate_binding"],
            "route.evidence_admission_gate_binding",
            true,
        )?;
        if gate["artifact_id"].as_str() != Some(EVIDENCE_GATE_ARTIFACT_ID)
            || gate["ref"].as_str() != Some(EVIDENCE_GATE_REF)
        {
            return Err(ReaderError::new(
                "route.evidence_admission_gate_binding must bind the fixed executable gate",
            ));
        }
        native_gate_self_test(env.context).map_err(|error| {
            ReaderError::new(format!(
                "route evidence gate must pass its fixed executable self-test: {error}"
            ))
        })?;
        gate["sha256"].as_str()
    };
    let reviewer_present = !route["reviewer_custody_attestation"].is_null();
    if reviewer_present {
        let reviewer = object(
            &route["reviewer_custody_attestation"],
            "route.reviewer_custody_attestation",
        )?;
        exact_keys(
            reviewer,
            REVIEWER_ATTESTATION_KEYS,
            "route.reviewer_custody_attestation",
        )?;
        opaque_id(
            &reviewer["attestation_id"],
            "route.reviewer_custody_attestation.attestation_id",
        )?;
        if reviewer["scope"].as_str() != Some("reader-evidence-gate-review") {
            return Err(ReaderError::new(
                "route reviewer attestation has the wrong closed scope",
            ));
        }
        let Some(gate_digest) = gate_sha256 else {
            return Err(ReaderError::new(
                "route reviewer attestation requires the executable gate binding",
            ));
        };
        digest(
            &reviewer["evidence_gate_sha256"],
            "route.reviewer_custody_attestation.evidence_gate_sha256",
            Some(gate_digest),
        )?;
        if reviewer["ref"].as_str() != Some("custody:READER-EVIDENCE-GATE-REVIEW") {
            return Err(ReaderError::new(
                "route reviewer attestation must use the fixed external custody channel",
            ));
        }
        validate_external_or_repo_reference(
            env,
            &reviewer["ref"],
            "route.reviewer_custody_attestation.ref",
        )?;
        date(
            &reviewer["attested_date"],
            "route.reviewer_custody_attestation.attested_date",
        )?;
        digest(
            &reviewer["sha256"],
            "route.reviewer_custody_attestation.sha256",
            None,
        )?;
    }
    let control_status = enumeration(
        &route["negative_control_status"],
        &[
            "not-run",
            "watched-failing",
            "failed-to-fail",
            "indeterminate",
        ],
        "route.negative_control_status",
    )?;
    let pilot_control = object(&source["pilot"], "pilot")?["control_status"]
        .as_str()
        .unwrap_or_default();
    if control_status != pilot_control {
        return Err(ReaderError::new(
            "route.negative_control_status must equal the active pilot control",
        ));
    }
    let available = reviewer_present
        && gate_sha256.is_some()
        && valid_pilot
        && control_status == "watched-failing"
        && source["threshold_status"].as_str() == Some("author-ratified");
    let expected_route = if available { "available" } else { "unbuilt" };
    if route_status != expected_route {
        return Err(ReaderError::new(format!(
            "route.route_status must be {expected_route} for its complete tuple"
        )));
    }
    Ok(RouteValidation {
        status: route_status,
        gate_sha256,
        checker_sha256: structural["sha256"].as_str().unwrap(),
    })
}

fn validate_claim(
    env: &ValidationEnv<'_>,
    source: &Map<String, Value>,
    route_status: &str,
    valid_holdout_pass: bool,
) -> ReaderResult<()> {
    let claim = object(&source["claim"], "claim")?;
    exact_keys(claim, CLAIM_KEYS, "claim")?;
    if claim["claim_id"].as_str() != Some("FS-CLM-37") {
        return Err(ReaderError::new("claim.claim_id must be FS-CLM-37"));
    }
    validate_repo_reference(env, &claim["result_ref"], "claim.result_ref")?;
    let expected = if valid_holdout_pass {
        ("Evidenced", "none")
    } else if route_status == "available" {
        ("Unestablished", "evidence-pending")
    } else {
        ("Unestablished", "route-unbuilt")
    };
    if claim["posture"].as_str() != Some(expected.0)
        || claim["disposition"].as_str() != Some(expected.1)
    {
        return Err(ReaderError::new(format!(
            "claim posture/disposition must be {}/{} for current evidence state",
            expected.0, expected.1
        )));
    }
    Ok(())
}

fn validate_acceptance(source: &Map<String, Value>) -> ReaderResult<()> {
    let acceptance = object(&source["acceptance"], "acceptance")?;
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance")?;
    if acceptance["gate_c_satisfied"].as_bool() != Some(false) {
        return Err(ReaderError::new(
            "reader evidence alone may never satisfy Gate C",
        ));
    }
    if acceptance["permitted_claim"].as_str() != Some("none") {
        return Err(ReaderError::new(
            "this contract may not rewrite Gate C's permitted claim",
        ));
    }
    text_list(&acceptance["limits"], "acceptance.limits", true)?;
    Ok(())
}

struct HistorySnapshot<'a> {
    pilot_attempts: Vec<&'a Map<String, Value>>,
    holdout_attempts: Vec<&'a Map<String, Value>>,
    head: String,
}

fn history_stream<'a>(
    source: &'a Map<String, Value>,
    root_path: &str,
    container_key: &str,
    container_keys: &[&str],
    attempt_keys: &[&str],
) -> ReaderResult<(Vec<&'a Map<String, Value>>, Vec<String>)> {
    let container_path = format!("{root_path}.{container_key}");
    let container = object(&source[container_key], &container_path)?;
    exact_keys(container, container_keys, &container_path)?;
    let mut attempts = Vec::new();
    let mut digests = Vec::new();
    let mut previous: Option<String> = None;
    for (index, raw) in array(
        &container["attempts"],
        &format!("{container_path}.attempts"),
    )?
    .iter()
    .enumerate()
    {
        let attempt_path = format!("{container_path}.attempts[{index}]");
        let attempt = object(raw, &attempt_path)?;
        exact_keys(attempt, attempt_keys, &attempt_path)?;
        if index == 0 {
            if !attempt["previous_attempt_sha256"].is_null() {
                return Err(ReaderError::new(format!(
                    "{attempt_path}.previous_attempt_sha256: first attempt must be null"
                )));
            }
        } else {
            digest(
                &attempt["previous_attempt_sha256"],
                &format!("{attempt_path}.previous_attempt_sha256"),
                previous.as_deref(),
            )?;
        }
        let expected = canonical_sha(&Value::Object(attempt.clone()), Some("attempt_sha256"))?;
        let actual = digest(
            &attempt["attempt_sha256"],
            &format!("{attempt_path}.attempt_sha256"),
            Some(&expected),
        )?
        .to_owned();
        attempts.push(attempt);
        previous = Some(actual.clone());
        digests.push(actual);
    }
    Ok((attempts, digests))
}

fn validated_history_snapshot<'a>(
    source: &'a Map<String, Value>,
    path: &str,
) -> ReaderResult<HistorySnapshot<'a>> {
    let (pilot_attempts, pilot_digests) =
        history_stream(source, path, "pilot", PILOT_KEYS, PILOT_ATTEMPT_KEYS)?;
    let (holdout_attempts, holdout_digests) =
        history_stream(source, path, "holdout", HOLDOUT_KEYS, HOLDOUT_ATTEMPT_KEYS)?;
    let head = history_head_sha256(
        pilot_digests.iter().map(String::as_str),
        holdout_digests.iter().map(String::as_str),
    );
    Ok(HistorySnapshot {
        pilot_attempts,
        holdout_attempts,
        head,
    })
}

fn committed_reader_evidence(
    context: &Context,
    commit: &str,
    path: &str,
) -> ReaderResult<(Vec<u8>, Value)> {
    let object_spec = format!("{commit}:{DEFAULT_SOURCE}");
    let shown = git_output(context, &["show", &object_spec], path)?;
    if !shown.status.success() {
        let detail = String::from_utf8_lossy(&shown.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "{path}: committed reader-evidence source is unavailable{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        )));
    }
    let source = parse_source(&shown.stdout).map_err(|_| {
        ReaderError::new(format!(
            "{path}: committed reader-evidence source is invalid JSON"
        ))
    })?;
    Ok((shown.stdout, source))
}

fn nearest_previous_reader_evidence(
    context: &Context,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<Option<(String, Vec<u8>, Value)>> {
    let anchor = source_commit.unwrap_or("HEAD");
    let history = git_output(
        context,
        &[
            "log",
            "--first-parent",
            "--format=%H",
            anchor,
            "--",
            DEFAULT_SOURCE,
        ],
        "history_transition",
    )?;
    if !history.status.success() {
        let detail = String::from_utf8_lossy(&history.stderr).trim().to_owned();
        return Err(ReaderError::new(format!(
            "history_transition: cannot inspect first-parent source history{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        )));
    }
    let commits: Vec<_> = String::from_utf8_lossy(&history.stdout)
        .lines()
        .filter(|line| commit_regex().is_match(line))
        .map(str::to_owned)
        .collect();
    let Some(first_commit) = commits.first() else {
        return Ok(None);
    };
    let (first_raw, _) = committed_reader_evidence(
        context,
        first_commit,
        "history_transition.current_committed_source",
    )?;
    let predecessor_index = usize::from(first_raw == source_raw);
    let Some(commit) = commits.get(predecessor_index) else {
        return Ok(None);
    };
    let (raw, source) =
        committed_reader_evidence(context, commit, "history_transition.previous_source")?;
    Ok(Some((commit.clone(), raw, source)))
}

fn validate_history_stream_transition(
    name: &str,
    previous_attempts: &[&Map<String, Value>],
    current_attempts: &[&Map<String, Value>],
) -> ReaderResult<&'static str> {
    if current_attempts.len() < previous_attempts.len() {
        return Err(ReaderError::new(format!(
            "history_transition.{name}: prior attempt history must be prefix-preserved"
        )));
    }
    if current_attempts.len() > previous_attempts.len() + 1 {
        return Err(ReaderError::new(format!(
            "history_transition.{name}: only one successor may be appended per transition"
        )));
    }
    if current_attempts.len() == previous_attempts.len() + 1 {
        if current_attempts[..previous_attempts.len()] != *previous_attempts {
            return Err(ReaderError::new(format!(
                "history_transition.{name}: prior attempt history must be prefix-preserved"
            )));
        }
        let expected_status = if name == "pilot" { "not-run" } else { "frozen" };
        if current_attempts.last().unwrap()["attempt_status"].as_str() != Some(expected_status) {
            return Err(ReaderError::new(format!(
                "history_transition.{name}: a successor must begin {expected_status}"
            )));
        }
        return Ok("append");
    }
    let differing: Vec<_> = previous_attempts
        .iter()
        .zip(current_attempts)
        .enumerate()
        .filter(|(_, (previous, current))| previous != current)
        .map(|(index, _)| index)
        .collect();
    if differing.is_empty() {
        return Ok("unchanged");
    }
    if differing != [previous_attempts.len() - 1] {
        return Err(ReaderError::new(format!(
            "history_transition.{name}: terminal and superseded attempts are immutable"
        )));
    }
    let previous = previous_attempts.last().unwrap();
    let current = current_attempts.last().unwrap();
    let nonterminal = if name == "pilot" { "not-run" } else { "frozen" };
    if previous["attempt_status"].as_str() != Some(nonterminal)
        || !matches!(
            current["attempt_status"].as_str(),
            Some("completed" | "void")
        )
    {
        return Err(ReaderError::new(format!(
            "history_transition.{name}: only the active {nonterminal} attempt may become terminal"
        )));
    }
    let immutable_keys: &[&str] = if name == "pilot" {
        &[
            "attempt_id",
            "previous_attempt_sha256",
            "prerequisites",
            "pre_registration",
            "tested_snapshot",
        ]
    } else {
        &[
            "attempt_id",
            "previous_attempt_sha256",
            "pre_registration",
            "frozen_rule",
            "frozen_ratification",
        ]
    };
    if immutable_keys
        .iter()
        .any(|key| previous[*key] != current[*key])
    {
        return Err(ReaderError::new(format!(
            "history_transition.{name}: frozen attempt identity and inputs are immutable"
        )));
    }
    Ok("terminal")
}

fn validate_history_transition(
    context: &Context,
    source: &Map<String, Value>,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<()> {
    let transition = object(&source["history_transition"], "history_transition")?;
    exact_keys(transition, HISTORY_TRANSITION_KEYS, "history_transition")?;
    let current = validated_history_snapshot(source, "root")?;
    digest(
        &transition["history_head_sha256"],
        "history_transition.history_head_sha256",
        Some(&current.head),
    )?;
    let previous = nearest_previous_reader_evidence(context, source_raw, source_commit)?;
    if previous.is_none() {
        if [
            "previous_source_commit",
            "previous_source_sha256",
            "previous_history_head_sha256",
        ]
        .iter()
        .any(|key| !transition[*key].is_null())
        {
            return Err(ReaderError::new(
                "history_transition: bootstrap source must have null predecessor fields",
            ));
        }
        let pilot = object(&source["pilot"], "pilot")?;
        let holdout = object(&source["holdout"], "holdout")?;
        if !current.pilot_attempts.is_empty()
            || !current.holdout_attempts.is_empty()
            || source["threshold_status"].as_str() != Some("pending-pilot")
            || source["holdout_status"].as_str() != Some("not-frozen")
            || source["result"].as_str() != Some("not-run")
            || !source["ratification"].is_null()
            || pilot["pilot_status"].as_str() != Some("not-run")
            || pilot["control_status"].as_str() != Some("not-run")
            || !pilot["active_attempt_id"].is_null()
            || !holdout["active_attempt_id"].is_null()
        {
            return Err(ReaderError::new(
                "history_transition: only the initial dormant empty source may bootstrap",
            ));
        }
        return Ok(());
    }
    let (expected_commit, previous_raw, previous_value) = previous.unwrap();
    let declared_commit = transition["previous_source_commit"].as_str();
    if declared_commit != Some(expected_commit.as_str())
        || !declared_commit.is_some_and(|value| commit_regex().is_match(value))
    {
        return Err(ReaderError::new(
            "history_transition.previous_source_commit must cite the nearest prior JSON-changing commit",
        ));
    }
    let previous_digest = sha256(&previous_raw);
    digest(
        &transition["previous_source_sha256"],
        "history_transition.previous_source_sha256",
        Some(&previous_digest),
    )?;
    let previous_source = object(&previous_value, "history_transition.previous_source")?;
    exact_keys(
        previous_source,
        ROOT_KEYS,
        "history_transition.previous_source",
    )?;
    let previous_transition = object(
        &previous_source["history_transition"],
        "history_transition.previous_source.history_transition",
    )?;
    exact_keys(
        previous_transition,
        HISTORY_TRANSITION_KEYS,
        "history_transition.previous_source.history_transition",
    )?;
    let previous =
        validated_history_snapshot(previous_source, "history_transition.previous_source")?;
    digest(
        &previous_transition["history_head_sha256"],
        "history_transition.previous_source.history_head_sha256",
        Some(&previous.head),
    )?;
    digest(
        &transition["previous_history_head_sha256"],
        "history_transition.previous_history_head_sha256",
        Some(&previous.head),
    )?;
    let pilot_action = validate_history_stream_transition(
        "pilot",
        &previous.pilot_attempts,
        &current.pilot_attempts,
    )?;
    let holdout_action = validate_history_stream_transition(
        "holdout",
        &previous.holdout_attempts,
        &current.holdout_attempts,
    )?;
    if pilot_action != "unchanged" && holdout_action != "unchanged" {
        return Err(ReaderError::new(
            "history_transition: pilot and holdout histories may not change in one transition",
        ));
    }
    Ok(())
}

fn validate_history_closure_at(
    context: &Context,
    source: &Map<String, Value>,
    source_raw: &[u8],
    source_commit: Option<&str>,
) -> ReaderResult<()> {
    fn unique(values: &mut HashSet<String>, value: &str, path: &str) -> ReaderResult<()> {
        if !values.insert(value.to_owned()) {
            return Err(ReaderError::new(format!(
                "{path}: duplicate across attempt history"
            )));
        }
        Ok(())
    }

    fn collect_common(
        attempt: &Map<String, Value>,
        path: &str,
        global_ids: &mut HashSet<String>,
        attempt_digests: &mut HashSet<String>,
        session_commitments: &mut HashSet<String>,
        custody_external_digests: &mut HashSet<String>,
        custody_record_digests: &mut HashSet<String>,
    ) -> ReaderResult<()> {
        unique(
            global_ids,
            opaque_id(&attempt["attempt_id"], &format!("{path}.attempt_id"))?,
            &format!("{path}.attempt_id"),
        )?;
        unique(
            attempt_digests,
            digest(
                &attempt["attempt_sha256"],
                &format!("{path}.attempt_sha256"),
                None,
            )?,
            &format!("{path}.attempt_sha256"),
        )?;
        for (index, raw) in array(
            &attempt["session_records"],
            &format!("{path}.session_records"),
        )?
        .iter()
        .enumerate()
        {
            let record = object(raw, &format!("{path}.session_records[{index}]"))?;
            unique(
                session_commitments,
                digest(
                    &record["record_commitment_sha256"],
                    &format!("{path}.session_records[{index}].record_commitment_sha256"),
                    None,
                )?,
                &format!("{path}.session_records[{index}].record_commitment_sha256"),
            )?;
        }
        for (index, raw) in array(
            &attempt["custody_attestations"],
            &format!("{path}.custody_attestations"),
        )?
        .iter()
        .enumerate()
        {
            let record = object(raw, &format!("{path}.custody_attestations[{index}]"))?;
            unique(
                global_ids,
                opaque_id(
                    &record["attestation_id"],
                    &format!("{path}.custody_attestations[{index}].attestation_id"),
                )?,
                &format!("{path}.custody_attestations[{index}].attestation_id"),
            )?;
            unique(
                custody_external_digests,
                digest(
                    &record["sha256"],
                    &format!("{path}.custody_attestations[{index}].sha256"),
                    None,
                )?,
                &format!("{path}.custody_attestations[{index}].sha256"),
            )?;
            unique(
                custody_record_digests,
                digest(
                    &record["record_sha256"],
                    &format!("{path}.custody_attestations[{index}].record_sha256"),
                    None,
                )?,
                &format!("{path}.custody_attestations[{index}].record_sha256"),
            )?;
        }
        for (index, raw) in array(&attempt["deviations"], &format!("{path}.deviations"))?
            .iter()
            .enumerate()
        {
            let deviation = object(raw, &format!("{path}.deviations[{index}]"))?;
            unique(
                global_ids,
                opaque_id(
                    &deviation["deviation_id"],
                    &format!("{path}.deviations[{index}].deviation_id"),
                )?,
                &format!("{path}.deviations[{index}].deviation_id"),
            )?;
        }
        Ok(())
    }

    let mut global_ids = HashSet::new();
    let mut attempt_digests = HashSet::new();
    let mut preregistration_digests = HashSet::new();
    let mut receipt_digests = HashSet::new();
    let mut session_commitments = HashSet::new();
    let mut custody_external_digests = HashSet::new();
    let mut custody_record_digests = HashSet::new();
    let mut gate_input_digests = HashSet::new();

    let pilot = object(&source["pilot"], "pilot")?;
    let mut previous_terminal: Option<String> = None;
    for (index, raw) in array(&pilot["attempts"], "pilot.attempts")?
        .iter()
        .enumerate()
    {
        let path = format!("pilot.attempts[{index}]");
        let attempt = object(raw, &path)?;
        collect_common(
            attempt,
            &path,
            &mut global_ids,
            &mut attempt_digests,
            &mut session_commitments,
            &mut custody_external_digests,
            &mut custody_record_digests,
        )?;
        let mut freeze_at = None;
        if !attempt["pre_registration"].is_null() {
            let registration = object(
                &attempt["pre_registration"],
                &format!("{path}.pre_registration"),
            )?;
            unique(
                &mut global_ids,
                opaque_id(
                    &registration["study_id"],
                    &format!("{path}.pre_registration.study_id"),
                )?,
                &format!("{path}.pre_registration.study_id"),
            )?;
            unique(
                &mut preregistration_digests,
                digest(
                    &registration["pre_registration_sha256"],
                    &format!("{path}.pre_registration.pre_registration_sha256"),
                    None,
                )?,
                &format!("{path}.pre_registration.pre_registration_sha256"),
            )?;
            let binding = object(
                &registration["freeze_binding"],
                &format!("{path}.pre_registration.freeze_binding"),
            )?;
            unique(
                &mut global_ids,
                opaque_id(
                    &binding["binding_id"],
                    &format!("{path}.pre_registration.freeze_binding.binding_id"),
                )?,
                &format!("{path}.pre_registration.freeze_binding.binding_id"),
            )?;
            unique(
                &mut custody_external_digests,
                digest(
                    &binding["attestation_sha256"],
                    &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
                    None,
                )?,
                &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
            )?;
            freeze_at = binding["frozen_at"].as_str();
        }
        if previous_terminal
            .as_deref()
            .is_some_and(|terminal| freeze_at.is_none_or(|frozen| frozen <= terminal))
        {
            return Err(ReaderError::new(format!(
                "{path}: successor freeze must strictly follow the prior pilot terminal time"
            )));
        }
        match attempt["attempt_status"].as_str() {
            Some("completed") => {
                let receipt = object(&attempt["receipt"], &format!("{path}.receipt"))?;
                unique(
                    &mut global_ids,
                    opaque_id(
                        &receipt["receipt_id"],
                        &format!("{path}.receipt.receipt_id"),
                    )?,
                    &format!("{path}.receipt.receipt_id"),
                )?;
                unique(
                    &mut receipt_digests,
                    digest(
                        &receipt["receipt_sha256"],
                        &format!("{path}.receipt.receipt_sha256"),
                        None,
                    )?,
                    &format!("{path}.receipt.receipt_sha256"),
                )?;
                let packet = object(
                    &attempt["decision_packet"],
                    &format!("{path}.decision_packet"),
                )?;
                unique(
                    &mut global_ids,
                    opaque_id(
                        &packet["packet_id"],
                        &format!("{path}.decision_packet.packet_id"),
                    )?,
                    &format!("{path}.decision_packet.packet_id"),
                )?;
                let binding = object(
                    &packet["freeze_binding"],
                    &format!("{path}.decision_packet.freeze_binding"),
                )?;
                unique(
                    &mut global_ids,
                    opaque_id(
                        &binding["binding_id"],
                        &format!("{path}.decision_packet.freeze_binding.binding_id"),
                    )?,
                    &format!("{path}.decision_packet.freeze_binding.binding_id"),
                )?;
                unique(
                    &mut custody_external_digests,
                    digest(
                        &binding["attestation_sha256"],
                        &format!("{path}.decision_packet.freeze_binding.attestation_sha256"),
                        None,
                    )?,
                    &format!("{path}.decision_packet.freeze_binding.attestation_sha256"),
                )?;
                previous_terminal = binding["frozen_at"].as_str().map(str::to_owned);
            }
            Some("void") => {
                previous_terminal = attempt["voided_at"].as_str().map(str::to_owned);
            }
            _ => {}
        }
    }

    previous_terminal = None;
    let holdout = object(&source["holdout"], "holdout")?;
    for (index, raw) in array(&holdout["attempts"], "holdout.attempts")?
        .iter()
        .enumerate()
    {
        let path = format!("holdout.attempts[{index}]");
        let attempt = object(raw, &path)?;
        collect_common(
            attempt,
            &path,
            &mut global_ids,
            &mut attempt_digests,
            &mut session_commitments,
            &mut custody_external_digests,
            &mut custody_record_digests,
        )?;
        let registration = object(
            &attempt["pre_registration"],
            &format!("{path}.pre_registration"),
        )?;
        unique(
            &mut global_ids,
            opaque_id(
                &registration["study_id"],
                &format!("{path}.pre_registration.study_id"),
            )?,
            &format!("{path}.pre_registration.study_id"),
        )?;
        unique(
            &mut preregistration_digests,
            digest(
                &registration["pre_registration_sha256"],
                &format!("{path}.pre_registration.pre_registration_sha256"),
                None,
            )?,
            &format!("{path}.pre_registration.pre_registration_sha256"),
        )?;
        let binding = object(
            &registration["freeze_binding"],
            &format!("{path}.pre_registration.freeze_binding"),
        )?;
        unique(
            &mut global_ids,
            opaque_id(
                &binding["binding_id"],
                &format!("{path}.pre_registration.freeze_binding.binding_id"),
            )?,
            &format!("{path}.pre_registration.freeze_binding.binding_id"),
        )?;
        unique(
            &mut custody_external_digests,
            digest(
                &binding["attestation_sha256"],
                &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
                None,
            )?,
            &format!("{path}.pre_registration.freeze_binding.attestation_sha256"),
        )?;
        let freeze_at = binding["frozen_at"].as_str().unwrap_or_default();
        if previous_terminal
            .as_deref()
            .is_some_and(|terminal| freeze_at <= terminal)
        {
            return Err(ReaderError::new(format!(
                "{path}: successor freeze must strictly follow the prior holdout terminal time"
            )));
        }
        if !registration["commitment"].is_null() {
            let commitment = object(
                &registration["commitment"],
                &format!("{path}.pre_registration.commitment"),
            )?;
            unique(
                &mut global_ids,
                opaque_id(
                    &commitment["commitment_id"],
                    &format!("{path}.pre_registration.commitment.commitment_id"),
                )?,
                &format!("{path}.pre_registration.commitment.commitment_id"),
            )?;
        }
        match attempt["attempt_status"].as_str() {
            Some("completed") => {
                let receipt = object(
                    &attempt["result_receipt"],
                    &format!("{path}.result_receipt"),
                )?;
                unique(
                    &mut global_ids,
                    opaque_id(
                        &receipt["receipt_id"],
                        &format!("{path}.result_receipt.receipt_id"),
                    )?,
                    &format!("{path}.result_receipt.receipt_id"),
                )?;
                unique(
                    &mut receipt_digests,
                    digest(
                        &receipt["receipt_sha256"],
                        &format!("{path}.result_receipt.receipt_sha256"),
                        None,
                    )?,
                    &format!("{path}.result_receipt.receipt_sha256"),
                )?;
                let gate = object(
                    &attempt["gate_admission_receipt"],
                    &format!("{path}.gate_admission_receipt"),
                )?;
                unique(
                    &mut gate_input_digests,
                    digest(
                        &gate["input_sha256"],
                        &format!("{path}.gate_admission_receipt.input_sha256"),
                        None,
                    )?,
                    &format!("{path}.gate_admission_receipt.input_sha256"),
                )?;
                unique(
                    &mut receipt_digests,
                    digest(
                        &gate["receipt_sha256"],
                        &format!("{path}.gate_admission_receipt.receipt_sha256"),
                        None,
                    )?,
                    &format!("{path}.gate_admission_receipt.receipt_sha256"),
                )?;
                previous_terminal = receipt["completed_at"].as_str().map(str::to_owned);
            }
            Some("void") => {
                if !attempt["result_receipt"].is_null() {
                    let receipt = object(
                        &attempt["result_receipt"],
                        &format!("{path}.result_receipt"),
                    )?;
                    unique(
                        &mut global_ids,
                        opaque_id(
                            &receipt["receipt_id"],
                            &format!("{path}.result_receipt.receipt_id"),
                        )?,
                        &format!("{path}.result_receipt.receipt_id"),
                    )?;
                    unique(
                        &mut receipt_digests,
                        digest(
                            &receipt["receipt_sha256"],
                            &format!("{path}.result_receipt.receipt_sha256"),
                            None,
                        )?,
                        &format!("{path}.result_receipt.receipt_sha256"),
                    )?;
                }
                previous_terminal = attempt["voided_at"].as_str().map(str::to_owned);
            }
            _ => {}
        }
        if !attempt["commitment_reveal"].is_null() {
            let reveal = object(
                &attempt["commitment_reveal"],
                &format!("{path}.commitment_reveal"),
            )?;
            previous_terminal = reveal["revealed_at"].as_str().map(str::to_owned);
        }
    }

    validate_history_transition(context, source, source_raw, source_commit)
}

fn validate_history_closure(
    context: &Context,
    source: &Map<String, Value>,
    source_raw: &[u8],
) -> ReaderResult<()> {
    validate_history_closure_at(context, source, source_raw, None)
}

fn validate_holdout(
    env: &ValidationEnv<'_>,
    source: &Map<String, Value>,
    rule: &Map<String, Value>,
    _known_misconceptions: &BTreeSet<String>,
    route: &RouteValidation<'_>,
) -> ReaderResult<bool> {
    let holdout = object(&source["holdout"], "holdout")?;
    exact_keys(holdout, HOLDOUT_KEYS, "holdout")?;
    let summary_status = enumeration(
        &source["holdout_status"],
        &["not-frozen", "frozen", "completed", "void"],
        "holdout_status",
    )?;
    let summary_result = enumeration(
        &source["result"],
        &["not-run", "pass", "fail", "not-evaluable"],
        "result",
    )?;
    let active_attempt_id = if holdout["active_attempt_id"].is_null() {
        None
    } else {
        Some(opaque_id(
            &holdout["active_attempt_id"],
            "holdout.active_attempt_id",
        )?)
    };
    let attempts = array(&holdout["attempts"], "holdout.attempts")?;
    if attempts.is_empty() {
        if active_attempt_id.is_some() {
            return Err(ReaderError::new(
                "holdout.active_attempt_id requires an attempt",
            ));
        }
        if summary_status != "not-frozen" || summary_result != "not-run" {
            return Err(ReaderError::new(
                "empty holdout history must remain not-frozen/not-run",
            ));
        }
        return Ok(false);
    }
    if source["threshold_status"].as_str() != Some("author-ratified") {
        return Err(ReaderError::new(
            "every holdout attempt requires an author-ratified rule",
        ));
    }
    let current_ratification = object(&source["ratification"], "ratification")?;
    let current_ratification_sha = digest(
        &current_ratification["ratification_sha256"],
        "ratification.ratification_sha256",
        None,
    )?;
    let fixed_protocol_sha256 = canonical_sha_omitting(&source["protocol"], &["decision_sha256"])?;
    let mut seen_ids = HashSet::new();
    let mut previous_sha256: Option<&str> = None;
    let pilot = object(&source["pilot"], "pilot")?;
    let mut pilot_attempt_sha256s = Vec::new();
    for (index, raw) in array(&pilot["attempts"], "pilot.attempts")?
        .iter()
        .enumerate()
    {
        let attempt = object(raw, &format!("pilot.attempts[{index}]"))?;
        pilot_attempt_sha256s.push(digest(
            &attempt["attempt_sha256"],
            &format!("pilot.attempts[{index}].attempt_sha256"),
            None,
        )?);
    }
    let mut holdout_attempt_sha256s: Vec<&str> = Vec::new();
    let mut active_id = "";
    let mut active_status = "";
    let mut latest_completed_result = "not-run";
    let mut valid_active_pass = false;
    for (index, raw) in attempts.iter().enumerate() {
        let path = format!("holdout.attempts[{index}]");
        let attempt = object(raw, &path)?;
        exact_keys(attempt, HOLDOUT_ATTEMPT_KEYS, &path)?;
        let active = index + 1 == attempts.len();
        let attempt_id = opaque_id(&attempt["attempt_id"], &format!("{path}.attempt_id"))?;
        if !seen_ids.insert(attempt_id) {
            return Err(ReaderError::new(format!("{path}.attempt_id: duplicate")));
        }
        if index == 0 {
            if !attempt["previous_attempt_sha256"].is_null() {
                return Err(ReaderError::new(format!(
                    "{path}.previous_attempt_sha256: first attempt must be null"
                )));
            }
        } else {
            digest(
                &attempt["previous_attempt_sha256"],
                &format!("{path}.previous_attempt_sha256"),
                previous_sha256,
            )?;
        }
        let attempt_status = enumeration(
            &attempt["attempt_status"],
            &["frozen", "completed", "void"],
            &format!("{path}.attempt_status"),
        )?;
        let attempt_result = enumeration(
            &attempt["attempt_result"],
            &["not-run", "pass", "fail", "not-evaluable"],
            &format!("{path}.attempt_result"),
        )?;
        if !active && attempt_status == "frozen" {
            return Err(ReaderError::new(format!(
                "{path}: a superseded holdout attempt cannot remain frozen"
            )));
        }
        let voided_at = if attempt_status == "void" {
            let code = opaque_id(
                &attempt["void_reason_code"],
                &format!("{path}.void_reason_code"),
            )?;
            if !code.starts_with("RE-VOID-") {
                return Err(ReaderError::new(format!(
                    "{path}.void_reason_code: expected a closed RE-VOID-* code"
                )));
            }
            Some(utc_timestamp(
                &attempt["voided_at"],
                &format!("{path}.voided_at"),
            )?)
        } else {
            if !attempt["void_reason_code"].is_null() {
                return Err(ReaderError::new(format!(
                    "{path}.void_reason_code: only a void attempt may carry a reason"
                )));
            }
            if !attempt["voided_at"].is_null() {
                return Err(ReaderError::new(format!(
                    "{path}.voided_at: only a void attempt may carry a terminal time"
                )));
            }
            None
        };
        let mut shadow = Value::Object(source.clone());
        let shadow_object = shadow.as_object_mut().unwrap();
        shadow_object.insert(
            "threshold_status".to_owned(),
            Value::String("author-ratified".to_owned()),
        );
        shadow_object.insert("threshold_rule".to_owned(), attempt["frozen_rule"].clone());
        let frozen_misconceptions = validate_threshold_rule(&shadow, true)?;
        let frozen_rule_map = object(&attempt["frozen_rule"], &format!("{path}.frozen_rule"))?;
        let frozen_rule =
            typed_threshold_rule(&attempt["frozen_rule"], &format!("{path}.frozen_rule"))?;
        let frozen_ratification = validate_frozen_ratification_value(
            env,
            &attempt["frozen_ratification"],
            &format!("{path}.frozen_ratification"),
            frozen_rule_map,
            &fixed_protocol_sha256,
        )?;
        let prior_head = history_head_sha256(
            pilot_attempt_sha256s.iter().copied(),
            holdout_attempt_sha256s.iter().copied(),
        );
        let registration = validate_holdout_pre_registration_value(
            env,
            &attempt["pre_registration"],
            &format!("{path}.pre_registration"),
            active && attempt_status != "void",
            &fixed_protocol_sha256,
            (active && attempt_status != "void").then_some(route.checker_sha256),
            previous_sha256,
            Some(&prior_head),
            true,
        )?;
        digest(
            &registration["rule_sha256"],
            &format!("{path}.pre_registration.rule_sha256"),
            frozen_rule_map["rule_sha256"].as_str(),
        )?;
        let registered_ratification_sha = digest(
            &registration["ratification_sha256"],
            &format!("{path}.pre_registration.ratification_sha256"),
            frozen_ratification["ratification_sha256"].as_str(),
        )?;
        let study_id = registration["study_id"].as_str().unwrap();
        let sessions_value = &attempt["session_records"];
        let sessions = validate_session_records(
            sessions_value,
            &format!("{path}.session_records"),
            Some(study_id),
            Some(&frozen_misconceptions),
        )?;
        let typed_sessions = typed_sessions(sessions_value, &format!("{path}.session_records"))?;
        let raw_deviations = array(&attempt["deviations"], &format!("{path}.deviations"))?;
        let deviations =
            validate_deviations(&attempt["deviations"], &format!("{path}.deviations"))?;
        let raw_custody = array(
            &attempt["custody_attestations"],
            &format!("{path}.custody_attestations"),
        )?;
        let custody = validate_custody(
            env,
            &attempt["custody_attestations"],
            &format!("{path}.custody_attestations"),
        )?;
        let commitment = if registration["commitment"].is_null() {
            None
        } else {
            Some(object(
                &registration["commitment"],
                &format!("{path}.pre_registration.commitment"),
            )?)
        };
        validate_record_links(
            &sessions,
            &deviations,
            &custody,
            &format!("{path}.record_links"),
            Some(study_id),
            commitment,
        )?;
        let receipt = if attempt["result_receipt"].is_null() {
            None
        } else {
            Some(validate_result_receipt_value(
                &attempt["result_receipt"],
                &format!("{path}.result_receipt"),
                registration,
                &attempt["frozen_rule"],
                &frozen_rule,
                sessions_value,
                &typed_sessions,
                raw_deviations,
                raw_custody,
            )?)
        };
        let ran = !sessions.is_empty() || receipt.is_some();
        let terminal_at = if attempt_status == "completed" {
            receipt.and_then(|value| value["completed_at"].as_str())
        } else if attempt_status == "void" {
            voided_at
        } else {
            None
        };
        validate_commitment_reveal_value(
            env,
            &attempt["commitment_reveal"],
            &format!("{path}.commitment_reveal"),
            commitment,
            &custody,
            attempt_status,
            matches!(attempt_status, "completed" | "void"),
            active,
            terminal_at,
        )?;
        let rule_match = frozen_rule_map["rule_sha256"] == rule["rule_sha256"]
            && registration["rule_sha256"] == rule["rule_sha256"];
        let gate_match = route.gate_sha256.is_some()
            && registration["evidence_gate_sha256"].as_str() == route.gate_sha256;
        let structural_match =
            registration["structural_checker_sha256"].as_str() == Some(route.checker_sha256);
        let ratification_match = registered_ratification_sha == current_ratification_sha
            && frozen_ratification["ratification_sha256"].as_str()
                == Some(current_ratification_sha);
        let current_binding = rule_match && gate_match && structural_match && ratification_match;
        if active && attempt_status != "void" && !current_binding {
            return Err(ReaderError::new(format!(
                "{path}: active holdout must bind the current rule, ratification, gate, and checker"
            )));
        }
        if registration["registered_date"].as_str().unwrap()
            < frozen_ratification["ratified_date"].as_str().unwrap()
        {
            return Err(ReaderError::new(format!(
                "{path}: pre-registration cannot precede its frozen ratification"
            )));
        }
        let voiding_deviation = deviations
            .values()
            .any(|item| item["impact"].as_str() == Some("holdout-void"));
        let freshness_records: Vec<_> = custody
            .values()
            .filter(|item| item["scope"].as_str() == Some("study-freshness"))
            .collect();
        if ran && freshness_records.len() != 1 {
            return Err(ReaderError::new(format!(
                "{path}: a run holdout requires exactly one freshness attestation"
            )));
        }
        let freshness_bound = freshness_records.len() == 1
            && freshness_records[0]["freshness_attested"].as_bool() == Some(true);
        let freeze = object(
            &registration["freeze_binding"],
            &format!("{path}.pre_registration.freeze_binding"),
        )?;
        let freeze_at = freeze["frozen_at"].as_str().unwrap();
        match attempt_status {
            "frozen" => validate_frozen_holdout_payload(
                attempt_result,
                &sessions,
                &deviations,
                &custody,
                receipt.is_some(),
                !attempt["commitment_reveal"].is_null(),
                !attempt["gate_admission_receipt"].is_null(),
                commitment,
                &path,
            )?,
            "completed" => {
                if active && route.status != "available" {
                    return Err(ReaderError::new(format!(
                        "{path}: the reader route must be available before the active holdout runs"
                    )));
                }
                if voiding_deviation {
                    return Err(ReaderError::new(format!(
                        "{path}: a voiding deviation cannot remain completed"
                    )));
                }
                let Some(receipt) = receipt else {
                    return Err(ReaderError::new(format!(
                        "{path}: completed holdout requires coded sessions and a receipt"
                    )));
                };
                if sessions.is_empty() {
                    return Err(ReaderError::new(format!(
                        "{path}: completed holdout requires coded sessions and a receipt"
                    )));
                }
                let completed_at = receipt["completed_at"].as_str().unwrap();
                if completed_at <= freeze_at {
                    return Err(ReaderError::new(format!(
                        "{path}: completion must strictly follow the frozen preregistration"
                    )));
                }
                if &completed_at[..10] < registration["registered_date"].as_str().unwrap() {
                    return Err(ReaderError::new(format!(
                        "{path}: completion cannot precede pre-registration"
                    )));
                }
                if receipt["verdict"].as_str() != Some(attempt_result) {
                    return Err(ReaderError::new(format!(
                        "{path}.attempt_result: must equal the recomputed receipt verdict"
                    )));
                }
                if receipt["protocol_validity"].as_str() == Some("valid") && !freshness_bound {
                    return Err(ReaderError::new(format!(
                        "{path}: a protocol-valid result requires bound freshness custody"
                    )));
                }
                let gate_input = build_gate_input(
                    attempt_id.to_owned(),
                    frozen_rule,
                    decode_typed(
                        &attempt["frozen_ratification"],
                        &format!("{path}.frozen_ratification"),
                    )?,
                    decode_typed(
                        &attempt["pre_registration"],
                        &format!("{path}.pre_registration"),
                    )?,
                    typed_sessions,
                    decode_typed(&attempt["deviations"], &format!("{path}.deviations"))?,
                    decode_typed(
                        &attempt["custody_attestations"],
                        &format!("{path}.custody_attestations"),
                    )?,
                    decode_typed(
                        &attempt["result_receipt"],
                        &format!("{path}.result_receipt"),
                    )?,
                    decode_typed(
                        &attempt["commitment_reveal"],
                        &format!("{path}.commitment_reveal"),
                    )?,
                );
                let expected_decision = if receipt["protocol_validity"].as_str() == Some("valid")
                    && attempt_result == "pass"
                    && freshness_bound
                {
                    "admit"
                } else {
                    "reject"
                };
                let gate_receipt = validate_gate_admission_receipt_value(
                    env,
                    &attempt["gate_admission_receipt"],
                    &format!("{path}.gate_admission_receipt"),
                    &gate_input,
                    expected_decision,
                    active,
                )?;
                latest_completed_result = attempt_result;
                valid_active_pass =
                    active && current_binding && gate_receipt["decision"].as_str() == Some("admit");
            }
            "void" => {
                let voided_at = voided_at.unwrap();
                if voided_at <= freeze_at {
                    return Err(ReaderError::new(format!(
                        "{path}: void time must strictly follow the frozen preregistration"
                    )));
                }
                if !attempt["gate_admission_receipt"].is_null() {
                    return Err(ReaderError::new(format!(
                        "{path}: a void holdout may not carry an admission receipt"
                    )));
                }
                if let Some(receipt) = receipt {
                    if receipt["protocol_validity"].as_str() != Some("invalid")
                        || receipt["verdict"].as_str() != Some("not-evaluable")
                        || attempt_result != "not-evaluable"
                    {
                        return Err(ReaderError::new(format!(
                            "{path}: a run void must preserve an invalid, not-evaluable receipt"
                        )));
                    }
                    if receipt["completed_at"].as_str().unwrap() > voided_at {
                        return Err(ReaderError::new(format!(
                            "{path}: receipt completion cannot follow the void time"
                        )));
                    }
                } else if attempt_result != "not-run" {
                    return Err(ReaderError::new(format!(
                        "{path}: a pre-result void attempt must remain not-run"
                    )));
                }
                if ran && !voiding_deviation {
                    return Err(ReaderError::new(format!(
                        "{path}: a run void requires a custody-linked holdout-void deviation"
                    )));
                }
            }
            _ => unreachable!(),
        }
        let expected = canonical_sha(&Value::Object(attempt.clone()), Some("attempt_sha256"))?;
        let attempt_sha = digest(
            &attempt["attempt_sha256"],
            &format!("{path}.attempt_sha256"),
            Some(&expected),
        )?;
        previous_sha256 = Some(attempt_sha);
        holdout_attempt_sha256s.push(attempt_sha);
        active_id = attempt_id;
        active_status = attempt_status;
    }
    if active_attempt_id != Some(active_id) {
        return Err(ReaderError::new(
            "holdout.active_attempt_id must identify the final append-only attempt",
        ));
    }
    if summary_status != active_status {
        return Err(ReaderError::new(
            "holdout_status must equal the active attempt lifecycle",
        ));
    }
    if summary_result != latest_completed_result {
        return Err(ReaderError::new(
            "result must preserve the latest completed non-void outcome",
        ));
    }
    Ok(valid_active_pass)
}

fn validate_source(
    context: &Context,
    source: &Value,
    source_raw: &[u8],
    protocol_decision: &[u8],
) -> ReaderResult<Validation> {
    let source = object(source, "root")?;
    exact_keys(source, ROOT_KEYS, "root")?;
    walk_keys(&Value::Object(source.clone()), "root")?;
    let env = ValidationEnv {
        context,
        protocol_decision,
        verify_live: true,
    };
    validate_protocol(&env, source)?;
    let pilot = validate_pilot(&env, source)?;
    validate_privacy(source)?;
    let known_misconceptions =
        validate_threshold_rule(&Value::Object(source.clone()), pilot.valid)?;
    let rule = object(&source["threshold_rule"], "threshold_rule")?;
    validate_ratification(&env, source, rule, &pilot)?;
    let route = validate_route_readiness(&env, source, pilot.valid, None)?;
    let valid_holdout_pass = validate_holdout(&env, source, rule, &known_misconceptions, &route)?;
    validate_history_closure(context, source, source_raw)?;
    validate_claim(&env, source, route.status, valid_holdout_pass)?;
    validate_acceptance(source)?;
    // Duplicate-key rejection and contract-specific diagnostics run against the
    // lossless JSON tree above. No reviewed record is accepted until the whole
    // nested source also satisfies the strict serde domain model.
    let typed = typed_reader_source(&Value::Object(source.clone()), "root")?;
    if typed.threshold_status != "pending-pilot" {
        typed.threshold_rule.populated("threshold_rule")?;
    }
    Ok(Validation {
        valid_pilot: pilot.valid,
        valid_holdout_pass,
    })
}

#[cfg(test)]
fn python_string(value: &Value) -> String {
    match value {
        Value::Null => "None".to_owned(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
fn escape(value: &Value) -> String {
    python_string(value).replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
fn code(value: &Value) -> String {
    format!("`{}`", python_string(value).replace('`', ""))
}

fn code_text(value: &str) -> String {
    format!("`{}`", value.replace('`', ""))
}

#[cfg(test)]
fn append_threshold(lines: &mut Vec<String>, label: &str, value: &Value) -> ReaderResult<()> {
    let spec = object(value, label)?;
    let scopes = array(&spec["scope_refs"], &format!("{label}.scope_refs"))?
        .iter()
        .map(code)
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!(
        "- {}: {}; {} {} {} {}; denominator {}; scope {}.",
        escape(&Value::String(label.to_owned())),
        code(&spec["threshold_id"]),
        code(&spec["metric"]),
        code(&spec["operator"]),
        code(&spec["value"]),
        escape(&spec["unit"]),
        code(&spec["denominator"]),
        scopes,
    ));
    Ok(())
}

fn escape_text(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn code_option_text(value: Option<&str>) -> String {
    code_text(value.unwrap_or("None"))
}

fn append_threshold_typed(lines: &mut Vec<String>, label: &str, spec: &ThresholdSpec) {
    let scopes = spec
        .scope_refs
        .iter()
        .map(|item| code_text(item))
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!(
        "- {}: {}; {} {} {} {}; denominator {}; scope {}.",
        escape_text(label),
        code_text(&spec.threshold_id),
        code_text(&spec.metric),
        code_text(&spec.operator),
        code_text(&spec.value),
        escape_text(&spec.unit),
        code_text(&spec.denominator),
        scopes,
    ));
}

fn render_typed(source: &ReaderEvidenceSource, source_digest: &str) -> ReaderResult<String> {
    let dormant = source.threshold_status == "pending-pilot"
        && source.pilot.pilot_status == "not-run"
        && source.holdout_status == "not-frozen"
        && source.result == "not-run";
    let banner = if dormant {
        "**DORMANT PRE-PILOT CONTRACT: no reader result and no release threshold.**"
    } else {
        "**REVIEWED READER-EVIDENCE STATE: bounded by the statuses below.**"
    };
    let intro = if dormant {
        vec![
            "This report renders the reviewed evidence contract. It does not run a",
            "reader study, ratify a taxonomy or value, make R6 available, establish",
            "FS-CLM-37, or satisfy Gate C.",
        ]
    } else {
        vec![
            "This report renders the current reviewed evidence state. Rendering does",
            "not itself run a reader study, admit evidence, or satisfy Gate C.",
        ]
    };
    let state_note = if dormant {
        vec![
            "The threshold fields are empty by design. No pilot receipt, author",
            "ratification, holdout pre-registration, session record, or result receipt",
            "is present.",
        ]
    } else {
        vec!["The canonical machine source owns the exact artifacts and current state."]
    };
    let route_note: Vec<&str> = match source.route.route_status.as_str() {
        "unbuilt" => vec![
            "R6 remains unbuilt because its availability tuple is incomplete.",
            "Structural checker controls do not substitute for the seeded pilot control",
            "or any missing external admission component.",
        ],
        "built" => vec![
            "R6 is built but not available to admit holdout evidence. The remaining",
            "availability requirements must be satisfied before evidence can be admitted.",
        ],
        "available" => vec![
            "R6 is available for a matching holdout under the bound admission route.",
            "Availability alone does not admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
        value => return Err(ReaderError::new(format!("unknown route state: {value}"))),
    };
    let threshold_note: Vec<&str> = match source.threshold_status.as_str() {
        "pending-pilot" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. Exact severity labels, classification",
            "boundaries, core mappings, policies, and threshold values remain absent",
            "until a valid pilot supplies the basis for a candidate rule.",
        ],
        "candidate" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are a post-pilot candidate only; they are not author-ratified",
            "and cannot govern a holdout.",
        ],
        "author-ratified" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are author-ratified. Ratification alone does not make R6",
            "available, admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
        value => {
            return Err(ReaderError::new(format!(
                "unknown threshold state: {value}"
            )));
        }
    };
    let prior = code_option_text(
        source
            .history_transition
            .previous_source_commit
            .as_deref()
            .or(Some("initial-bootstrap"))
            .filter(|_| source.history_transition.previous_source_commit.is_some()),
    );
    let prior = if source.history_transition.previous_source_commit.is_none() {
        code_text("initial-bootstrap")
    } else {
        prior
    };
    let claim_state = format!(
        "{}/{}",
        code_text(&source.claim.posture),
        code_text(&source.claim.disposition)
    );
    let gate_c = code_text(if source.acceptance.gate_c_satisfied {
        "true"
    } else {
        "false"
    });
    let mut lines = vec![
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->".to_owned(),
        "<!-- Generated by new-book-plans/14-reader-evidence.py from reader-evidence.json. Do not edit. -->".to_owned(),
        String::new(),
        "# Reader Evidence Contract".to_owned(),
        String::new(),
        banner.to_owned(),
        String::new(),
    ];
    lines.extend(intro.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "## Current state".to_owned(),
        String::new(),
        "| field | value |".to_owned(),
        "| --- | --- |".to_owned(),
        format!("| Threshold | {} |", code_text(&source.threshold_status)),
        format!("| Pilot | {} |", code_text(&source.pilot.pilot_status)),
        format!(
            "| Seeded instrument control | {} |",
            code_text(&source.pilot.control_status)
        ),
        format!(
            "| Reader route | {} |",
            code_text(&source.route.route_status)
        ),
        format!("| Holdout | {} |", code_text(&source.holdout_status)),
        format!("| Result | {} |", code_text(&source.result)),
        format!(
            "| History head | {} |",
            code_text(&source.history_transition.history_head_sha256)
        ),
        format!("| Prior source transition | {prior} |"),
        format!("| Claim | {claim_state} |"),
        format!("| Gate C satisfied here | {gate_c} |"),
        String::new(),
    ]);
    lines.extend(state_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "Attempt histories are prefix-preserved against the nearest prior".to_owned(),
        "JSON-changing commit; every frozen preregistration binds its predecessor".to_owned(),
        "attempt and prior two-stream history head. This is a repository-relative".to_owned(),
        "commitment, not proof against rewritten Git or external custody history.".to_owned(),
        String::new(),
        "## Route availability".to_owned(),
        String::new(),
        "| component | recorded state |".to_owned(),
        "| --- | --- |".to_owned(),
        format!("| Structural checker | {} |", code_text("bound")),
        format!(
            "| Evidence contract | {} |",
            code_text(&source.route.evidence_contract_status)
        ),
        format!(
            "| Threshold rule | {} |",
            code_text(&source.threshold_status)
        ),
        format!(
            "| Named reviewer custody binding | {} |",
            code_text(if source.route.reviewer_custody_attestation.is_some() {
                "present"
            } else {
                "absent"
            })
        ),
        format!(
            "| Evidence-admission gate route binding | {} |",
            code_text(if source.route.evidence_admission_gate_binding.is_some() {
                "present"
            } else {
                "absent"
            })
        ),
        format!(
            "| Instrument control watched failing | {} |",
            code_text(&source.route.negative_control_status)
        ),
        String::new(),
    ]);
    lines.extend(route_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "## Fixed pass-rule form".to_owned(),
        String::new(),
        format!(
            "Evaluation order: {}.",
            EVALUATION_ORDER
                .iter()
                .map(|item| code_text(item))
                .collect::<Vec<_>>()
                .join(" → ")
        ),
        String::new(),
    ]);
    lines.extend(threshold_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "### Minimum identification targets".to_owned(),
        String::new(),
    ]);
    for target in &source.protocol.required_targets {
        lines.push(format!(
            "- {} ({})",
            escape_text(&target.description),
            code_text(&target.target_id)
        ));
    }
    lines.extend([
        String::new(),
        "## Privacy boundary".to_owned(),
        String::new(),
        format!(
            "Public policy: {}.",
            code_text(&source.privacy.public_record_policy)
        ),
        String::new(),
        "The public source may hold only opaque study identifiers, coded target and".to_owned(),
        "misconception outcomes, artifact and commitment digests, coded deviations,".to_owned(),
        "and custody attestations without identity material. Everything below is private:"
            .to_owned(),
        String::new(),
    ]);
    for item in &source.privacy.excluded_from_repository {
        lines.push(format!("- {}", escape_text(item)));
    }
    lines.extend([
        String::new(),
        escape_text(&source.privacy.freshness_attestation_boundary),
        String::new(),
        "## What still does not follow".to_owned(),
        String::new(),
    ]);
    for item in &source.acceptance.limits {
        lines.push(format!("- {}", escape_text(item)));
    }
    if source.threshold_status != "pending-pilot" {
        let rule = source.threshold_rule.populated("threshold_rule")?;
        let projection = if source.threshold_status == "author-ratified" {
            "Ratified"
        } else {
            "Candidate"
        };
        lines.extend([
            String::new(),
            format!("## {projection} threshold projection"),
            String::new(),
            format!(
                "Rule: {}; SHA-256 {}.",
                code_text(&rule.rule_id),
                code_text(&rule.rule_sha256)
            ),
            String::new(),
            "Every value below is generated from the canonical machine source.".to_owned(),
            String::new(),
            "### Severity taxonomy and misconception mapping".to_owned(),
            String::new(),
        ]);
        for item in &rule.severity_taxonomy {
            lines.push(format!(
                "- {}: {} - {} Boundary: {}",
                code_text(&item.severity_id),
                escape_text(&item.label),
                escape_text(&item.definition),
                escape_text(&item.classification_boundary),
            ));
        }
        for item in &rule.misconceptions {
            lines.push(format!(
                "- {} -> {}; core {}. {}",
                code_text(&item.misconception_id),
                code_text(&item.severity_id),
                code_text(if item.core { "true" } else { "false" }),
                escape_text(&item.definition),
            ));
        }
        lines.extend([
            String::new(),
            "### Complete deterministic rule".to_owned(),
            String::new(),
            format!(
                "- Core branch: {}; repetition unit {}; denominator {}.",
                code_text(&rule.core_failure_mode),
                code_text(&rule.repetition_unit),
                code_text(&rule.denominator),
            ),
            format!(
                "- Evaluation order: {}.",
                rule.evaluation_order
                    .iter()
                    .map(|item| code_text(item))
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            format!(
                "- Aggregate offset prohibited: {}.",
                code_text(if rule.aggregate_offset_prohibited {
                    "true"
                } else {
                    "false"
                })
            ),
        ]);
        append_threshold_typed(&mut lines, "Core veto", &rule.core_failure_threshold);
        append_threshold_typed(
            &mut lines,
            "Minimum evaluable evidence",
            &rule.minimum_evaluable_evidence,
        );
        for item in &rule.required_target_thresholds {
            append_threshold_typed(
                &mut lines,
                &format!("Required target {}", item.target_id),
                &item.threshold,
            );
        }
        for item in &rule.non_core_thresholds {
            append_threshold_typed(
                &mut lines,
                &format!("Non-core severity {}", item.severity_id),
                &item.threshold,
            );
        }
        lines.extend([String::new(), "Policies:".to_owned(), String::new()]);
        for (key, value) in [
            ("missing", &rule.policies.missing),
            ("ambiguous", &rule.policies.ambiguous),
            ("multiply_coded", &rule.policies.multiply_coded),
            ("withdrawn", &rule.policies.withdrawn),
            ("excluded", &rule.policies.excluded),
            ("unclassified", &rule.policies.unclassified),
            ("rounding", &rule.policies.rounding),
            ("coder_adjudication", &rule.policies.coder_adjudication),
        ] {
            lines.push(format!("- {}: {}", code_text(key), code_text(value)));
        }
        if let Some(ratification) = &source.ratification {
            lines.extend([
                String::new(),
                "### Author ratification basis".to_owned(),
                String::new(),
                format!(
                    "- Ruling: {}; date {}; candidate commit {}.",
                    code_text(&ratification.ruling_id),
                    code_text(&ratification.ratified_date),
                    code_text(&ratification.candidate_commit),
                ),
                format!(
                    "- Pilot attempt {}; packet {}; sensitivity brief {}.",
                    code_text(&ratification.pilot_attempt_id),
                    code_text(&ratification.pilot_packet_sha256),
                    code_text(&ratification.sensitivity_brief_sha256),
                ),
                format!(
                    "- Rule digest {}; ratification receipt {}.",
                    code_text(&ratification.rule_sha256),
                    code_text(&ratification.ratification_sha256),
                ),
                format!(
                    "- Decision record: {}.",
                    code_text(&ratification.decision_ref)
                ),
                format!(
                    "- No holdout evidence existed or was inspected: {}.",
                    code_text(if ratification.no_holdout_evidence_attestation {
                        "true"
                    } else {
                        "false"
                    })
                ),
            ]);
        }
        lines.extend([
            String::new(),
            "### Append-only holdout history".to_owned(),
            String::new(),
            format!(
                "Active attempt: {}; lifecycle {}; latest completed non-void result {}.",
                code_option_text(source.holdout.active_attempt_id.as_deref()),
                code_text(&source.holdout_status),
                code_text(&source.result),
            ),
        ]);
        for attempt in &source.holdout.attempts {
            lines.push(format!(
                "- {}: {}/{}; frozen rule {}; attempt {}.",
                code_text(&attempt.attempt_id),
                code_text(&attempt.attempt_status),
                code_text(&attempt.attempt_result),
                code_text(&attempt.frozen_rule.rule_sha256),
                code_text(&attempt.attempt_sha256),
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Provenance and verification".to_owned(),
        String::new(),
        format!(
            "- Source: {}, SHA-256 {}.",
            code_text(DEFAULT_SOURCE),
            code_text(source_digest)
        ),
        format!(
            "- Controlling decision: {}, SHA-256 {}.",
            code_text(&source.protocol_decision_ref),
            code_text(&source.protocol.decision_sha256)
        ),
        "- Structural/freshness check: `python3 new-book-plans/14-reader-evidence.py --check`.".to_owned(),
        "- Executable contract controls: `python3 new-book-plans/14-reader-evidence.py --check --execute`.".to_owned(),
        String::new(),
    ]);
    Ok(lines.join("\n"))
}

#[cfg(test)]
fn render(source: &Value, source_digest: &str) -> ReaderResult<String> {
    let source = object(source, "root")?;
    let route = object(&source["route"], "route")?;
    let pilot = object(&source["pilot"], "pilot")?;
    let rule = object(&source["threshold_rule"], "threshold_rule")?;
    let privacy = object(&source["privacy"], "privacy")?;
    let protocol = object(&source["protocol"], "protocol")?;
    let acceptance = object(&source["acceptance"], "acceptance")?;
    let history = object(&source["history_transition"], "history_transition")?;
    let claim = object(&source["claim"], "claim")?;
    let dormant = source["threshold_status"].as_str() == Some("pending-pilot")
        && pilot["pilot_status"].as_str() == Some("not-run")
        && source["holdout_status"].as_str() == Some("not-frozen")
        && source["result"].as_str() == Some("not-run");
    let banner = if dormant {
        "**DORMANT PRE-PILOT CONTRACT: no reader result and no release threshold.**"
    } else {
        "**REVIEWED READER-EVIDENCE STATE: bounded by the statuses below.**"
    };
    let intro = if dormant {
        vec![
            "This report renders the reviewed evidence contract. It does not run a",
            "reader study, ratify a taxonomy or value, make R6 available, establish",
            "FS-CLM-37, or satisfy Gate C.",
        ]
    } else {
        vec![
            "This report renders the current reviewed evidence state. Rendering does",
            "not itself run a reader study, admit evidence, or satisfy Gate C.",
        ]
    };
    let state_note = if dormant {
        vec![
            "The threshold fields are empty by design. No pilot receipt, author",
            "ratification, holdout pre-registration, session record, or result receipt",
            "is present.",
        ]
    } else {
        vec!["The canonical machine source owns the exact artifacts and current state."]
    };
    let route_note: Vec<&str> = match route["route_status"].as_str().unwrap_or_default() {
        "unbuilt" => vec![
            "R6 remains unbuilt because its availability tuple is incomplete.",
            "Structural checker controls do not substitute for the seeded pilot control",
            "or any missing external admission component.",
        ],
        "built" => vec![
            "R6 is built but not available to admit holdout evidence. The remaining",
            "availability requirements must be satisfied before evidence can be admitted.",
        ],
        "available" => vec![
            "R6 is available for a matching holdout under the bound admission route.",
            "Availability alone does not admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
        value => return Err(ReaderError::new(format!("unknown route state: {value}"))),
    };
    let threshold_note: Vec<&str> = match source["threshold_status"].as_str().unwrap_or_default() {
        "pending-pilot" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. Exact severity labels, classification",
            "boundaries, core mappings, policies, and threshold values remain absent",
            "until a valid pilot supplies the basis for a candidate rule.",
        ],
        "candidate" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are a post-pilot candidate only; they are not author-ratified",
            "and cannot govern a holdout.",
        ],
        "author-ratified" => vec![
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are author-ratified. Ratification alone does not make R6",
            "available, admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
        value => {
            return Err(ReaderError::new(format!(
                "unknown threshold state: {value}"
            )));
        }
    };
    let prior = if history["previous_source_commit"].is_null() {
        code_text("initial-bootstrap")
    } else {
        code(&history["previous_source_commit"])
    };
    let claim_state = format!(
        "{}/{}",
        code(&claim["posture"]),
        code(&claim["disposition"])
    );
    let gate_c = code_text(if acceptance["gate_c_satisfied"].as_bool() == Some(true) {
        "true"
    } else {
        "false"
    });
    let mut lines = vec![
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->".to_owned(),
        "<!-- Generated by new-book-plans/14-reader-evidence.py from reader-evidence.json. Do not edit. -->".to_owned(),
        String::new(),
        "# Reader Evidence Contract".to_owned(),
        String::new(),
        banner.to_owned(),
        String::new(),
    ];
    lines.extend(intro.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "## Current state".to_owned(),
        String::new(),
        "| field | value |".to_owned(),
        "| --- | --- |".to_owned(),
        format!("| Threshold | {} |", code(&source["threshold_status"])),
        format!("| Pilot | {} |", code(&pilot["pilot_status"])),
        format!(
            "| Seeded instrument control | {} |",
            code(&pilot["control_status"])
        ),
        format!("| Reader route | {} |", code(&route["route_status"])),
        format!("| Holdout | {} |", code(&source["holdout_status"])),
        format!("| Result | {} |", code(&source["result"])),
        format!(
            "| History head | {} |",
            code(&history["history_head_sha256"])
        ),
        format!("| Prior source transition | {prior} |"),
        format!("| Claim | {claim_state} |"),
        format!("| Gate C satisfied here | {gate_c} |"),
        String::new(),
    ]);
    lines.extend(state_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "Attempt histories are prefix-preserved against the nearest prior".to_owned(),
        "JSON-changing commit; every frozen preregistration binds its predecessor".to_owned(),
        "attempt and prior two-stream history head. This is a repository-relative".to_owned(),
        "commitment, not proof against rewritten Git or external custody history.".to_owned(),
        String::new(),
        "## Route availability".to_owned(),
        String::new(),
        "| component | recorded state |".to_owned(),
        "| --- | --- |".to_owned(),
        format!(
            "| Structural checker | {} |",
            code_text(if route["structural_checker_binding"].is_null() {
                "absent"
            } else {
                "bound"
            })
        ),
        format!(
            "| Evidence contract | {} |",
            code(&route["evidence_contract_status"])
        ),
        format!("| Threshold rule | {} |", code(&source["threshold_status"])),
        format!(
            "| Named reviewer custody binding | {} |",
            code_text(if route["reviewer_custody_attestation"].is_null() {
                "absent"
            } else {
                "present"
            })
        ),
        format!(
            "| Evidence-admission gate route binding | {} |",
            code_text(if route["evidence_admission_gate_binding"].is_null() {
                "absent"
            } else {
                "present"
            })
        ),
        format!(
            "| Instrument control watched failing | {} |",
            code(&route["negative_control_status"])
        ),
        String::new(),
    ]);
    lines.extend(route_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "## Fixed pass-rule form".to_owned(),
        String::new(),
        format!(
            "Evaluation order: {}.",
            EVALUATION_ORDER
                .iter()
                .map(|item| code_text(item))
                .collect::<Vec<_>>()
                .join(" → ")
        ),
        String::new(),
    ]);
    lines.extend(threshold_note.into_iter().map(str::to_owned));
    lines.extend([
        String::new(),
        "### Minimum identification targets".to_owned(),
        String::new(),
    ]);
    for target in array(&protocol["required_targets"], "protocol.required_targets")? {
        let target = object(target, "protocol.required_targets item")?;
        lines.push(format!(
            "- {} ({})",
            escape(&target["description"]),
            code(&target["target_id"])
        ));
    }
    lines.extend([
        String::new(),
        "## Privacy boundary".to_owned(),
        String::new(),
        format!("Public policy: {}.", code(&privacy["public_record_policy"])),
        String::new(),
        "The public source may hold only opaque study identifiers, coded target and".to_owned(),
        "misconception outcomes, artifact and commitment digests, coded deviations,".to_owned(),
        "and custody attestations without identity material. Everything below is private:"
            .to_owned(),
        String::new(),
    ]);
    for item in array(
        &privacy["excluded_from_repository"],
        "privacy.excluded_from_repository",
    )? {
        lines.push(format!("- {}", escape(item)));
    }
    lines.extend([
        String::new(),
        escape(&privacy["freshness_attestation_boundary"]),
        String::new(),
        "## What still does not follow".to_owned(),
        String::new(),
    ]);
    for item in array(&acceptance["limits"], "acceptance.limits")? {
        lines.push(format!("- {}", escape(item)));
    }
    if source["threshold_status"].as_str() != Some("pending-pilot") {
        let projection = if source["threshold_status"].as_str() == Some("author-ratified") {
            "Ratified"
        } else {
            "Candidate"
        };
        lines.extend([
            String::new(),
            format!("## {projection} threshold projection"),
            String::new(),
            format!(
                "Rule: {}; SHA-256 {}.",
                code(&rule["rule_id"]),
                code(&rule["rule_sha256"])
            ),
            String::new(),
            "Every value below is generated from the canonical machine source.".to_owned(),
            String::new(),
            "### Severity taxonomy and misconception mapping".to_owned(),
            String::new(),
        ]);
        for item in array(
            &rule["severity_taxonomy"],
            "threshold_rule.severity_taxonomy",
        )? {
            let item = object(item, "severity")?;
            lines.push(format!(
                "- {}: {} - {} Boundary: {}",
                code(&item["severity_id"]),
                escape(&item["label"]),
                escape(&item["definition"]),
                escape(&item["classification_boundary"]),
            ));
        }
        for item in array(&rule["misconceptions"], "threshold_rule.misconceptions")? {
            let item = object(item, "misconception")?;
            lines.push(format!(
                "- {} -> {}; core {}. {}",
                code(&item["misconception_id"]),
                code(&item["severity_id"]),
                code_text(if item["core"].as_bool() == Some(true) {
                    "true"
                } else {
                    "false"
                }),
                escape(&item["definition"]),
            ));
        }
        lines.extend([
            String::new(),
            "### Complete deterministic rule".to_owned(),
            String::new(),
            format!(
                "- Core branch: {}; repetition unit {}; denominator {}.",
                code(&rule["core_failure_mode"]),
                code(&rule["repetition_unit"]),
                code(&rule["denominator"]),
            ),
            format!(
                "- Evaluation order: {}.",
                array(&rule["evaluation_order"], "threshold_rule.evaluation_order")?
                    .iter()
                    .map(code)
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            format!(
                "- Aggregate offset prohibited: {}.",
                code_text(
                    if rule["aggregate_offset_prohibited"].as_bool() == Some(true) {
                        "true"
                    } else {
                        "false"
                    }
                )
            ),
        ]);
        append_threshold(&mut lines, "Core veto", &rule["core_failure_threshold"])?;
        append_threshold(
            &mut lines,
            "Minimum evaluable evidence",
            &rule["minimum_evaluable_evidence"],
        )?;
        for item in array(
            &rule["required_target_thresholds"],
            "threshold_rule.required_target_thresholds",
        )? {
            let item = object(item, "required target threshold")?;
            append_threshold(
                &mut lines,
                &format!("Required target {}", python_string(&item["target_id"])),
                &item["threshold"],
            )?;
        }
        for item in array(
            &rule["non_core_thresholds"],
            "threshold_rule.non_core_thresholds",
        )? {
            let item = object(item, "non-core threshold")?;
            append_threshold(
                &mut lines,
                &format!("Non-core severity {}", python_string(&item["severity_id"])),
                &item["threshold"],
            )?;
        }
        lines.extend([String::new(), "Policies:".to_owned(), String::new()]);
        for (key, value) in object(&rule["policies"], "threshold_rule.policies")? {
            lines.push(format!("- {}: {}", code_text(key), code(value)));
        }
        if !source["ratification"].is_null() {
            let ratification = object(&source["ratification"], "ratification")?;
            lines.extend([
                String::new(),
                "### Author ratification basis".to_owned(),
                String::new(),
                format!(
                    "- Ruling: {}; date {}; candidate commit {}.",
                    code(&ratification["ruling_id"]),
                    code(&ratification["ratified_date"]),
                    code(&ratification["candidate_commit"]),
                ),
                format!(
                    "- Pilot attempt {}; packet {}; sensitivity brief {}.",
                    code(&ratification["pilot_attempt_id"]),
                    code(&ratification["pilot_packet_sha256"]),
                    code(&ratification["sensitivity_brief_sha256"]),
                ),
                format!(
                    "- Rule digest {}; ratification receipt {}.",
                    code(&ratification["rule_sha256"]),
                    code(&ratification["ratification_sha256"]),
                ),
                format!(
                    "- Decision record: {}.",
                    code(&ratification["decision_ref"])
                ),
                format!(
                    "- No holdout evidence existed or was inspected: {}.",
                    code_text(
                        if ratification["no_holdout_evidence_attestation"].as_bool() == Some(true) {
                            "true"
                        } else {
                            "false"
                        }
                    )
                ),
            ]);
        }
        let holdout = object(&source["holdout"], "holdout")?;
        lines.extend([
            String::new(),
            "### Append-only holdout history".to_owned(),
            String::new(),
            format!(
                "Active attempt: {}; lifecycle {}; latest completed non-void result {}.",
                code(&holdout["active_attempt_id"]),
                code(&source["holdout_status"]),
                code(&source["result"]),
            ),
        ]);
        for attempt in array(&holdout["attempts"], "holdout.attempts")? {
            let attempt = object(attempt, "holdout attempt")?;
            let frozen_rule = object(&attempt["frozen_rule"], "holdout attempt.frozen_rule")?;
            lines.push(format!(
                "- {}: {}/{}; frozen rule {}; attempt {}.",
                code(&attempt["attempt_id"]),
                code(&attempt["attempt_status"]),
                code(&attempt["attempt_result"]),
                code(&frozen_rule["rule_sha256"]),
                code(&attempt["attempt_sha256"]),
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Provenance and verification".to_owned(),
        String::new(),
        format!("- Source: {}, SHA-256 {}.", code_text(DEFAULT_SOURCE), code_text(source_digest)),
        format!(
            "- Controlling decision: {}, SHA-256 {}.",
            code(&source["protocol_decision_ref"]),
            code(&protocol["decision_sha256"])
        ),
        "- Structural/freshness check: `python3 new-book-plans/14-reader-evidence.py --check`.".to_owned(),
        "- Executable contract controls: `python3 new-book-plans/14-reader-evidence.py --check --execute`.".to_owned(),
        String::new(),
    ]);
    Ok(lines.join("\n"))
}

fn expect_validation_failure(
    context: &Context,
    source: &Value,
    source_raw: &[u8],
    protocol_decision: &[u8],
    label: &str,
    expected: Option<&str>,
    mutate: impl FnOnce(&mut Value),
) -> ReaderResult<()> {
    let mut changed = source.clone();
    mutate(&mut changed);
    // A schema mutation has done its job as soon as strict serde decoding
    // rejects it. Semantic mutations that still decode must fail for their
    // declared contract reason below.
    let typed = match typed_reader_source(&changed, "root") {
        Ok(typed) => typed,
        Err(_) => return Ok(()),
    };
    let typed_result =
        validate_source_typed(context, &typed, source_raw, protocol_decision).map(|_| ());
    match typed_result {
        Err(error) if expected.is_none_or(|fragment| error.to_string().contains(fragment)) => {
            Ok(())
        }
        Err(error) => Err(ReaderError::new(format!(
            "negative control failed for the wrong typed reason: {label}: {error}"
        ))),
        Ok(_) => Err(ReaderError::new(format!(
            "negative control did not fail: {label}"
        ))),
    }
}

fn expect_reader_failure<T>(
    label: &str,
    expected: Option<&str>,
    result: ReaderResult<T>,
) -> ReaderResult<()> {
    match result {
        Err(error) if expected.is_none_or(|fragment| error.to_string().contains(fragment)) => {
            Ok(())
        }
        Err(error) => Err(ReaderError::new(format!(
            "negative control failed for the wrong reason: {label}: {error}"
        ))),
        Ok(_) => Err(ReaderError::new(format!(
            "negative control did not fail: {label}"
        ))),
    }
}

fn set_threshold_value(value: &mut Value, threshold_id: &str, new_value: &str) -> bool {
    match value {
        Value::Object(object) => {
            if object.get("threshold_id").and_then(Value::as_str) == Some(threshold_id) {
                object.insert("value".to_owned(), Value::String(new_value.to_owned()));
                return true;
            }
            object
                .values_mut()
                .any(|child| set_threshold_value(child, threshold_id, new_value))
        }
        Value::Array(array) => array
            .iter_mut()
            .any(|child| set_threshold_value(child, threshold_id, new_value)),
        _ => false,
    }
}

fn rewrite_custody_record(record: &mut Value, updates: &[(&str, Value)]) -> ReaderResult<()> {
    let object = record
        .as_object_mut()
        .ok_or_else(|| ReaderError::new("custody mutation fixture is not an object"))?;
    for (key, value) in updates {
        object.insert((*key).to_owned(), value.clone());
    }
    let digest = canonical_sha(&Value::Object(object.clone()), Some("record_sha256"))?;
    object.insert("record_sha256".to_owned(), Value::String(digest));
    Ok(())
}

fn append_distinct_custody_record(
    records: &mut Vec<Value>,
    index: usize,
    updates: &[(&str, Value)],
) -> ReaderResult<()> {
    let mut record = records
        .get(index)
        .cloned()
        .ok_or_else(|| ReaderError::new("custody mutation index is absent"))?;
    let identifier = record["attestation_id"]
        .as_str()
        .ok_or_else(|| ReaderError::new("custody mutation identifier is absent"))?;
    let external_digest = record["sha256"]
        .as_str()
        .ok_or_else(|| ReaderError::new("custody mutation digest is absent"))?;
    let base_updates = [
        (
            "attestation_id",
            Value::String(format!("{identifier}-MUTATION")),
        ),
        (
            "sha256",
            Value::String(sha256(format!("{external_digest}-mutation").as_bytes())),
        ),
    ];
    rewrite_custody_record(&mut record, &base_updates)?;
    rewrite_custody_record(&mut record, updates)?;
    records.push(record);
    Ok(())
}

fn structural_controls(
    context: &Context,
    source: &Value,
    source_raw: &[u8],
    protocol_decision: &[u8],
) -> ReaderResult<usize> {
    let mut controls = 0;
    macro_rules! control {
        ($label:literal, $expected:expr, $mutation:expr) => {{
            expect_validation_failure(
                context,
                source,
                source_raw,
                protocol_decision,
                $label,
                $expected,
                $mutation,
            )?;
            controls += 1;
        }};
    }
    control!(
        "unknown root key",
        Some("unexpected"),
        |value: &mut Value| {
            value
                .as_object_mut()
                .unwrap()
                .insert("extra".to_owned(), Value::Null);
        }
    );
    control!(
        "boolean schema version",
        Some("integer 1"),
        |value: &mut Value| {
            value["schema_version"] = Value::Bool(true);
        }
    );
    control!(
        "history head digest drift",
        Some("stale"),
        |value: &mut Value| {
            value["history_transition"]["history_head_sha256"] = Value::String("0".repeat(64));
        }
    );
    control!(
        "unknown threshold status",
        Some("invalid state"),
        |value: &mut Value| {
            value["threshold_status"] = Value::String("settled".to_owned());
        }
    );
    control!(
        "unknown holdout status",
        Some("invalid state"),
        |value: &mut Value| {
            value["holdout_status"] = Value::String("running".to_owned());
        }
    );
    control!(
        "unknown result",
        Some("invalid state"),
        |value: &mut Value| {
            value["result"] = Value::String("success".to_owned());
        }
    );
    control!(
        "stale protocol digest",
        Some("stale"),
        |value: &mut Value| {
            value["protocol"]["decision_sha256"] = Value::String("0".repeat(64));
        }
    );
    control!(
        "disclosed limits drift",
        Some("drifted"),
        |value: &mut Value| {
            value["protocol"]["disclosed_limits"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("unratified limit".to_owned()));
        }
    );
    control!(
        "ethics terms drift",
        Some("drifted"),
        |value: &mut Value| {
            value["protocol"]["ethics_terms"]
                .as_array_mut()
                .unwrap()
                .reverse();
        }
    );
    control!(
        "freshness terms drift",
        Some("drifted"),
        |value: &mut Value| {
            value["protocol"]["freshness_terms"]
                .as_array_mut()
                .unwrap()
                .pop();
        }
    );
    control!(
        "non-substitution drift",
        Some("drifted"),
        |value: &mut Value| {
            value["protocol"]["non_substitution"] =
                Value::String("reader evidence substitutes".to_owned());
        }
    );
    control!(
        "public record kinds drift",
        Some("drifted"),
        |value: &mut Value| {
            value["privacy"]["allowed_public_record_kinds"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("identity mapping".to_owned()));
        }
    );
    control!(
        "broken protocol anchor",
        Some("exactly once"),
        |value: &mut Value| {
            value["protocol_decision_ref"] =
                Value::String(format!("{PROTOCOL_DECISION}::missing-anchor"));
        }
    );
    control!(
        "aggregate score field",
        Some("scoring fields"),
        |value: &mut Value| {
            value["threshold_rule"]
                .as_object_mut()
                .unwrap()
                .insert("aggregate_score".to_owned(), Value::Null);
        }
    );
    control!(
        "private raw-response field",
        Some("outside the repository"),
        |value: &mut Value| {
            value["holdout"]
                .as_object_mut()
                .unwrap()
                .insert("raw_responses".to_owned(), Value::Array(Vec::new()));
        }
    );
    control!(
        "Gate C overclaim",
        Some("never satisfy"),
        |value: &mut Value| {
            value["acceptance"]["gate_c_satisfied"] = Value::Bool(true);
        }
    );
    control!(
        "aggregate veto removed",
        Some("preserve"),
        |value: &mut Value| {
            value["threshold_rule"]["aggregate_offset_prohibited"] = Value::Bool(false);
        }
    );
    control!(
        "evaluation order changed",
        Some("fixed order"),
        |value: &mut Value| {
            value["threshold_rule"]["evaluation_order"]
                .as_array_mut()
                .unwrap()
                .reverse();
        }
    );
    control!(
        "structural checker digest drift",
        Some("stale"),
        |value: &mut Value| {
            value["route"]["structural_checker_binding"]["sha256"] = Value::String("0".repeat(64));
        }
    );
    control!(
        "implemented contract hidden",
        Some("must record"),
        |value: &mut Value| {
            value["route"]["evidence_contract_status"] = Value::String("unbuilt".to_owned());
        }
    );
    control!(
        "route control diverges from pilot",
        Some("must equal"),
        |value: &mut Value| {
            let replacement = if value["pilot"]["control_status"].as_str() != Some("indeterminate")
            {
                "indeterminate"
            } else {
                "failed-to-fail"
            };
            value["route"]["negative_control_status"] = Value::String(replacement.to_owned());
        }
    );
    for policy_key in [
        "ambiguous",
        "coder_adjudication",
        "excluded",
        "missing",
        "multiply_coded",
        "rounding",
        "unclassified",
        "withdrawn",
    ] {
        let label = format!("threshold policy key retained: {policy_key}");
        expect_validation_failure(
            context,
            source,
            source_raw,
            protocol_decision,
            &label,
            Some("missing"),
            |value| {
                value["threshold_rule"]["policies"]
                    .as_object_mut()
                    .unwrap()
                    .remove(policy_key);
            },
        )?;
        controls += 1;
    }
    let committed_probe = br#"{"value":1}
"#;
    let byte_only_probe = br#"{ "value": 1 }
"#;
    let before = usize::from(committed_probe.as_slice() == byte_only_probe.as_slice());
    let after = usize::from(byte_only_probe.as_slice() == byte_only_probe.as_slice());
    if before != 0 || after != 1 {
        return Err(ReaderError::new(
            "byte-only predecessor selector changed across commit boundary",
        ));
    }
    controls += 1;
    let pilot_attempts = array(&source["pilot"]["attempts"], "pilot.attempts")?;
    let holdout_attempts = array(&source["holdout"]["attempts"], "holdout.attempts")?;
    let dormant = source["threshold_status"].as_str() == Some("pending-pilot")
        && pilot_attempts.is_empty()
        && holdout_attempts.is_empty();
    if dormant {
        control!(
            "threshold content before valid pilot",
            Some("prohibited until"),
            |value: &mut Value| {
                value["threshold_rule"]["rule_id"] = value["contract_id"].clone();
            }
        );
        control!(
            "severity entry before valid pilot",
            Some("prohibited until"),
            |value: &mut Value| {
                value["threshold_rule"]["severity_taxonomy"]
                    .as_array_mut()
                    .unwrap()
                    .push(Value::Object(Map::new()));
            }
        );
        control!(
            "candidate status before valid pilot",
            Some("prohibited until"),
            |value: &mut Value| {
                value["threshold_status"] = Value::String("candidate".to_owned());
            }
        );
        control!(
            "ratification before valid pilot",
            Some("prohibited"),
            |value: &mut Value| {
                value["ratification"] = Value::Object(Map::new());
            }
        );
        control!(
            "pilot summary without attempt",
            Some("empty pilot history"),
            |value: &mut Value| {
                value["pilot"]["pilot_status"] = Value::String("completed".to_owned());
            }
        );
        control!(
            "control result without pilot",
            Some("empty pilot history"),
            |value: &mut Value| {
                value["pilot"]["control_status"] = Value::String("watched-failing".to_owned());
            }
        );
        control!(
            "holdout lifecycle without attempt",
            Some("empty holdout history"),
            |value: &mut Value| {
                value["holdout_status"] = Value::String("frozen".to_owned());
            }
        );
        control!(
            "result without completed attempt",
            Some("empty holdout history"),
            |value: &mut Value| {
                value["result"] = Value::String("pass".to_owned());
            }
        );
        control!(
            "route availability without components",
            Some("must be unbuilt"),
            |value: &mut Value| {
                value["route"]["route_status"] = Value::String("available".to_owned());
            }
        );
        control!(
            "claim evidence pending on unbuilt route",
            Some("route-unbuilt"),
            |value: &mut Value| {
                value["claim"]["disposition"] = Value::String("evidence-pending".to_owned());
            }
        );
        control!(
            "claim evidenced without a pass",
            Some("Unestablished"),
            |value: &mut Value| {
                value["claim"]["posture"] = Value::String("Evidenced".to_owned());
                value["claim"]["disposition"] = Value::String("none".to_owned());
            }
        );
        if source["history_transition"]["previous_source_commit"].is_null() {
            control!(
                "bootstrap predecessor injected",
                Some("null predecessor"),
                |value: &mut Value| {
                    value["history_transition"]["previous_source_commit"] =
                        Value::String("0".repeat(40));
                }
            );
        } else {
            control!(
                "dormant history predecessor commit drift",
                Some("nearest prior JSON-changing commit"),
                |value: &mut Value| {
                    value["history_transition"]["previous_source_commit"] =
                        Value::String("0".repeat(40));
                }
            );
            control!(
                "dormant history predecessor source digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["history_transition"]["previous_source_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
            control!(
                "dormant history predecessor head drift",
                Some("stale"),
                |value: &mut Value| {
                    value["history_transition"]["previous_history_head_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
        }
    } else {
        if !source["history_transition"]["previous_source_commit"].is_null() {
            control!(
                "history predecessor commit drift",
                Some("nearest prior JSON-changing commit"),
                |value: &mut Value| {
                    value["history_transition"]["previous_source_commit"] =
                        Value::String("0".repeat(40));
                }
            );
            control!(
                "history predecessor source digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["history_transition"]["previous_source_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
            control!(
                "history predecessor head drift",
                Some("stale"),
                |value: &mut Value| {
                    value["history_transition"]["previous_history_head_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
        }

        let pilot_maps = pilot_attempts
            .iter()
            .enumerate()
            .map(|(index, value)| object(value, &format!("pilot.attempts[{index}]")))
            .collect::<ReaderResult<Vec<_>>>()?;
        let holdout_maps = holdout_attempts
            .iter()
            .enumerate()
            .map(|(index, value)| object(value, &format!("holdout.attempts[{index}]")))
            .collect::<ReaderResult<Vec<_>>>()?;
        if !pilot_maps.is_empty() {
            expect_reader_failure(
                "pilot history deletion/reset",
                Some("prefix-preserved"),
                validate_history_stream_transition(
                    "pilot",
                    &pilot_maps,
                    &pilot_maps[..pilot_maps.len() - 1],
                ),
            )?;
            controls += 1;
            if let Some(index) = pilot_maps.iter().rposition(|attempt| {
                matches!(
                    attempt["attempt_status"].as_str(),
                    Some("completed" | "void")
                )
            }) {
                let mut changed_values = pilot_attempts.to_vec();
                changed_values[index]["void_reason_code"] =
                    Value::String("RE-VOID-HISTORY-MUTATION".to_owned());
                let changed_maps = changed_values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        object(value, &format!("changed pilot.attempts[{index}]"))
                    })
                    .collect::<ReaderResult<Vec<_>>>()?;
                expect_reader_failure(
                    "pilot terminal history mutation",
                    Some("only the active"),
                    validate_history_stream_transition("pilot", &pilot_maps, &changed_maps),
                )?;
                controls += 1;
            }
            control!(
                "pilot active pointer drift",
                Some("final append-only attempt"),
                |value: &mut Value| {
                    value["pilot"]["active_attempt_id"] =
                        Value::String("RE-PILOT-NOT-ACTIVE".to_owned());
                }
            );
            control!(
                "pilot attempt digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["pilot"]["attempts"]
                        .as_array_mut()
                        .unwrap()
                        .last_mut()
                        .unwrap()["attempt_sha256"] = Value::String("0".repeat(64));
                }
            );
        }
        if !holdout_maps.is_empty() {
            expect_reader_failure(
                "holdout history deletion/reset",
                Some("prefix-preserved"),
                validate_history_stream_transition(
                    "holdout",
                    &holdout_maps,
                    &holdout_maps[..holdout_maps.len() - 1],
                ),
            )?;
            controls += 1;
            if let Some(index) = holdout_maps.iter().rposition(|attempt| {
                matches!(
                    attempt["attempt_status"].as_str(),
                    Some("completed" | "void")
                )
            }) {
                let mut changed_values = holdout_attempts.to_vec();
                changed_values[index]["void_reason_code"] =
                    Value::String("RE-VOID-HISTORY-MUTATION".to_owned());
                let changed_maps = changed_values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        object(value, &format!("changed holdout.attempts[{index}]"))
                    })
                    .collect::<ReaderResult<Vec<_>>>()?;
                expect_reader_failure(
                    "holdout terminal history mutation",
                    Some("only the active"),
                    validate_history_stream_transition("holdout", &holdout_maps, &changed_maps),
                )?;
                controls += 1;
            }
            control!(
                "holdout active pointer drift",
                Some("final append-only attempt"),
                |value: &mut Value| {
                    value["holdout"]["active_attempt_id"] =
                        Value::String("RE-HOLDOUT-NOT-ACTIVE".to_owned());
                }
            );
            control!(
                "holdout attempt digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["holdout"]["attempts"]
                        .as_array_mut()
                        .unwrap()
                        .last_mut()
                        .unwrap()["attempt_sha256"] = Value::String("0".repeat(64));
                }
            );
        }

        if !pilot_maps.is_empty()
            && !holdout_maps.is_empty()
            && matches!(
                pilot_maps.last().unwrap()["attempt_status"].as_str(),
                Some("completed" | "void")
            )
            && matches!(
                holdout_maps.last().unwrap()["attempt_status"].as_str(),
                Some("completed" | "void")
            )
        {
            let mut previous_pilot_values = pilot_attempts.to_vec();
            let mut previous_holdout_values = holdout_attempts.to_vec();
            previous_pilot_values.last_mut().unwrap()["attempt_status"] =
                Value::String("not-run".to_owned());
            previous_holdout_values.last_mut().unwrap()["attempt_status"] =
                Value::String("frozen".to_owned());
            let previous_pilot = previous_pilot_values
                .iter()
                .map(|value| object(value, "previous pilot attempt"))
                .collect::<ReaderResult<Vec<_>>>()?;
            let previous_holdout = previous_holdout_values
                .iter()
                .map(|value| object(value, "previous holdout attempt"))
                .collect::<ReaderResult<Vec<_>>>()?;
            let result = (|| {
                let pilot_action =
                    validate_history_stream_transition("pilot", &previous_pilot, &pilot_maps)?;
                let holdout_action = validate_history_stream_transition(
                    "holdout",
                    &previous_holdout,
                    &holdout_maps,
                )?;
                if pilot_action != "unchanged" && holdout_action != "unchanged" {
                    return Err(ReaderError::new(
                        "history_transition: pilot and holdout histories may not change in one transition",
                    ));
                }
                Ok(())
            })();
            expect_reader_failure(
                "combined pilot and holdout history transition",
                Some("may not change in one transition"),
                result,
            )?;
            controls += 1;
        }

        if source["threshold_status"].as_str() != Some("pending-pilot") {
            control!(
                "populated rule digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["threshold_rule"]["rule_sha256"] = Value::String("0".repeat(64));
                }
            );
            control!(
                "count threshold endpoint rejected",
                Some("below, exact, and above cases"),
                |value: &mut Value| {
                    value["threshold_rule"]["minimum_evaluable_evidence"]["value"] =
                        Value::String("0".to_owned());
                }
            );
            let typed_rule = typed_threshold_rule(&source["threshold_rule"], "threshold_rule")?;
            if let Some(rate_spec) = std::iter::once(&typed_rule.core_failure_threshold)
                .chain(std::iter::once(&typed_rule.minimum_evaluable_evidence))
                .chain(
                    typed_rule
                        .required_target_thresholds
                        .iter()
                        .map(|entry| &entry.threshold),
                )
                .chain(
                    typed_rule
                        .non_core_thresholds
                        .iter()
                        .map(|entry| &entry.threshold),
                )
                .find(|spec| spec.metric.ends_with("-rate"))
            {
                let threshold_id = rate_spec.threshold_id.clone();
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "rate threshold endpoint rejected",
                    Some("below, exact, and above cases"),
                    |value| {
                        assert!(set_threshold_value(
                            &mut value["threshold_rule"],
                            &threshold_id,
                            "1"
                        ));
                    },
                )?;
                controls += 1;
            }
        }

        if !source["ratification"].is_null() {
            control!(
                "author ratification receipt drift",
                Some("stale"),
                |value: &mut Value| {
                    value["ratification"]["ratification_sha256"] = Value::String("0".repeat(64));
                }
            );
            control!(
                "candidate ancestry drift",
                Some("ancestor"),
                |value: &mut Value| {
                    value["ratification"]["candidate_commit"] = Value::String("0".repeat(40));
                }
            );
            let candidate_commit = source["ratification"]["candidate_commit"]
                .as_str()
                .ok_or_else(|| ReaderError::new("ratification.candidate_commit: expected text"))?;
            let (candidate_raw, candidate) =
                committed_reader_evidence(context, candidate_commit, "candidate control source")?;
            let candidate_decision = committed_file_bytes(
                context,
                candidate_commit,
                PROTOCOL_DECISION,
                "candidate control protocol decision",
            )?;
            let candidate_checker = committed_file_bytes(
                context,
                candidate_commit,
                STRUCTURAL_CHECKER_REF.split_once("::").unwrap().0,
                "candidate control structural checker",
            )?;
            validate_candidate_relevant_state(
                context,
                &candidate,
                candidate_commit,
                &candidate_decision,
                &candidate_checker,
                &candidate_raw,
                true,
            )?;
            controls += 1;
            for (label, expected, mutation) in [
                ("candidate protocol decision digest drift", "stale", 0_u8),
                ("candidate Gate C overclaim", "never satisfy", 1),
                ("candidate route overclaim", "must be unbuilt", 2),
                (
                    "candidate claim overclaim",
                    "Unestablished/route-unbuilt",
                    3,
                ),
                ("candidate history transition drift", "stale", 4),
            ] {
                let mut changed = candidate.clone();
                match mutation {
                    0 => {
                        changed["protocol"]["decision_sha256"] = Value::String("0".repeat(64));
                    }
                    1 => changed["acceptance"]["gate_c_satisfied"] = Value::Bool(true),
                    2 => {
                        changed["route"]["route_status"] = Value::String("available".to_owned());
                    }
                    3 => {
                        changed["claim"]["posture"] = Value::String("Evidenced".to_owned());
                        changed["claim"]["disposition"] = Value::String("none".to_owned());
                    }
                    4 => {
                        changed["history_transition"]["history_head_sha256"] =
                            Value::String("0".repeat(64));
                    }
                    _ => unreachable!(),
                }
                expect_reader_failure(
                    label,
                    Some(expected),
                    validate_candidate_relevant_state(
                        context,
                        &changed,
                        candidate_commit,
                        &candidate_decision,
                        &candidate_checker,
                        &candidate_raw,
                        true,
                    ),
                )?;
                controls += 1;
            }
        }

        if !pilot_maps.is_empty() {
            if let Some(index) = pilot_maps
                .iter()
                .rposition(|attempt| !attempt["pre_registration"].is_null())
            {
                let predecessor_error = if index == 0 {
                    "first attempt must be null"
                } else {
                    "stale"
                };
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot prereg predecessor mismatch",
                    Some(predecessor_error),
                    |value| {
                        value["pilot"]["attempts"][index]["pre_registration"]["predecessor_attempt_sha256"] =
                            Value::String("0".repeat(64));
                    },
                )?;
                controls += 1;
                for (label, field) in [
                    (
                        "pilot prereg prior history head mismatch",
                        "prior_history_head_sha256",
                    ),
                    ("pilot freeze payload drift", "bound_payload_sha256"),
                    ("pilot attested payload drift", "attested_payload_sha256"),
                ] {
                    expect_validation_failure(
                        context,
                        source,
                        source_raw,
                        protocol_decision,
                        label,
                        Some("stale"),
                        |value| {
                            if field == "prior_history_head_sha256" {
                                value["pilot"]["attempts"][index]["pre_registration"][field] =
                                    Value::String("0".repeat(64));
                            } else {
                                value["pilot"]["attempts"][index]["pre_registration"]["freeze_binding"]
                                    [field] = Value::String("0".repeat(64));
                            }
                        },
                    )?;
                    controls += 1;
                }
            }
            if let Some(index) = pilot_maps
                .iter()
                .rposition(|attempt| !attempt["decision_packet"].is_null())
            {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot packet freeze date drift",
                    Some("must equal"),
                    |value| {
                        value["pilot"]["attempts"][index]["decision_packet"]["frozen_date"] =
                            Value::String("1970-01-01".to_owned());
                    },
                )?;
                controls += 1;
            }
            if let Some(index) = pilot_maps
                .iter()
                .rposition(|attempt| !attempt["receipt"].is_null())
            {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot completion chronology drift",
                    Some("stale"),
                    |value| {
                        let frozen =
                            value["pilot"]["attempts"][index]["pre_registration"]["freeze_binding"]
                                ["frozen_at"]
                                .clone();
                        value["pilot"]["attempts"][index]["receipt"]["completed_at"] = frozen;
                    },
                )?;
                controls += 1;
            }
            if let Some(index) = pilot_maps.iter().position(|attempt| {
                attempt["custody_attestations"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty())
            }) {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot custody record duplication",
                    Some("duplicate"),
                    |value| {
                        let records = value["pilot"]["attempts"][index]["custody_attestations"]
                            .as_array_mut()
                            .unwrap();
                        records.push(records[0].clone());
                    },
                )?;
                controls += 1;
            }
            let pilot_freshness_attempt = pilot_maps
                .iter()
                .position(|attempt| attempt["attempt_status"].as_str() == Some("completed"))
                .or_else(|| {
                    pilot_maps.iter().position(|attempt| {
                        attempt["session_records"]
                            .as_array()
                            .is_some_and(|items| !items.is_empty())
                            || attempt["deviations"]
                                .as_array()
                                .is_some_and(|items| !items.is_empty())
                            || !attempt["receipt"].is_null()
                            || !attempt["decision_packet"].is_null()
                            || !attempt["sensitivity_brief"].is_null()
                    })
                });
            if let Some(attempt_index) = pilot_freshness_attempt {
                let freshness_index = pilot_maps[attempt_index]["custody_attestations"]
                    .as_array()
                    .and_then(|items| {
                        items
                            .iter()
                            .position(|item| item["scope"].as_str() == Some("study-freshness"))
                    })
                    .ok_or_else(|| {
                        ReaderError::new("validated pilot run lacks its freshness control record")
                    })?;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot freshness custody removed",
                    Some("exactly one study-freshness"),
                    |value| {
                        value["pilot"]["attempts"][attempt_index]["custody_attestations"]
                            .as_array_mut()
                            .unwrap()
                            .remove(freshness_index);
                    },
                )?;
                controls += 1;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot freshness custody duplicated",
                    Some("exactly one study-freshness"),
                    |value| {
                        append_distinct_custody_record(
                            value["pilot"]["attempts"][attempt_index]["custody_attestations"]
                                .as_array_mut()
                                .unwrap(),
                            freshness_index,
                            &[],
                        )
                        .expect("validated custody mutation");
                    },
                )?;
                controls += 1;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "pilot freshness custody wrong study",
                    Some("different study"),
                    |value| {
                        rewrite_custody_record(
                            &mut value["pilot"]["attempts"][attempt_index]["custody_attestations"]
                                [freshness_index],
                            &[("study_id", Value::String("RE-PILOT-WRONG-STUDY".to_owned()))],
                        )
                        .expect("validated custody mutation");
                    },
                )?;
                controls += 1;
                if pilot_maps[attempt_index]["attempt_status"].as_str() == Some("completed") {
                    expect_validation_failure(
                        context,
                        source,
                        source_raw,
                        protocol_decision,
                        "pilot freshness attestation false",
                        Some("freshness_attested true"),
                        |value| {
                            rewrite_custody_record(
                                &mut value["pilot"]["attempts"][attempt_index]
                                    ["custody_attestations"][freshness_index],
                                &[("freshness_attested", Value::Bool(false))],
                            )
                            .expect("validated custody mutation");
                        },
                    )?;
                    controls += 1;
                }
            }
        }

        if !holdout_maps.is_empty() {
            let active_index = holdout_maps.len() - 1;
            let predecessor_error = if active_index == 0 {
                "first attempt must be null"
            } else {
                "stale"
            };
            for (label, field, expected) in [
                (
                    "holdout prereg predecessor mismatch",
                    "predecessor_attempt_sha256",
                    Some(predecessor_error),
                ),
                (
                    "holdout prereg prior history head mismatch",
                    "prior_history_head_sha256",
                    Some("stale"),
                ),
                (
                    "holdout checker dependency drift",
                    "structural_checker_sha256",
                    None,
                ),
            ] {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    label,
                    expected,
                    |value| {
                        value["holdout"]["attempts"][active_index]["pre_registration"][field] =
                            Value::String("0".repeat(64));
                    },
                )?;
                controls += 1;
            }
            for (label, field) in [
                ("holdout freeze payload drift", "bound_payload_sha256"),
                ("holdout attested payload drift", "attested_payload_sha256"),
            ] {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    label,
                    Some("stale"),
                    |value| {
                        value["holdout"]["attempts"][active_index]["pre_registration"]["freeze_binding"]
                            [field] = Value::String("0".repeat(64));
                    },
                )?;
                controls += 1;
            }
            control!(
                "frozen holdout rule digest drift",
                Some("stale"),
                |value: &mut Value| {
                    value["holdout"]["attempts"][active_index]["frozen_rule"]["rule_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
            control!(
                "frozen ratification rule drift",
                Some("stale"),
                |value: &mut Value| {
                    value["holdout"]["attempts"][active_index]["frozen_ratification"]["rule_sha256"] =
                        Value::String("0".repeat(64));
                }
            );
            if let Some(index) = holdout_maps.iter().position(|attempt| {
                attempt["custody_attestations"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty())
            }) {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "holdout custody record duplication",
                    Some("duplicate"),
                    |value| {
                        let records = value["holdout"]["attempts"][index]["custody_attestations"]
                            .as_array_mut()
                            .unwrap();
                        records.push(records[0].clone());
                    },
                )?;
                controls += 1;
                if let Some(freshness_index) = holdout_maps[index]["custody_attestations"]
                    .as_array()
                    .and_then(|items| {
                        items
                            .iter()
                            .position(|item| item["scope"].as_str() == Some("study-freshness"))
                    })
                {
                    expect_validation_failure(
                        context,
                        source,
                        source_raw,
                        protocol_decision,
                        "holdout freshness custody removed",
                        None,
                        |value| {
                            value["holdout"]["attempts"][index]["custody_attestations"]
                                .as_array_mut()
                                .unwrap()
                                .remove(freshness_index);
                        },
                    )?;
                    controls += 1;
                }
            }
            if let Some(attempt_index) = holdout_maps.iter().position(|attempt| {
                attempt["attempt_status"].as_str() == Some("frozen")
                    && !attempt["pre_registration"]["commitment"].is_null()
            }) {
                let custody_index = holdout_maps[attempt_index]["custody_attestations"]
                    .as_array()
                    .and_then(|items| {
                        items
                            .iter()
                            .position(|item| item["scope"].as_str() == Some("commitment"))
                    })
                    .ok_or_else(|| {
                        ReaderError::new("validated frozen private commitment lacks custody")
                    })?;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "frozen commitment custody removed",
                    Some("commitment must bind exactly one"),
                    |value| {
                        value["holdout"]["attempts"][attempt_index]["custody_attestations"]
                            .as_array_mut()
                            .unwrap()
                            .remove(custody_index);
                    },
                )?;
                controls += 1;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "frozen holdout freshness custody injected",
                    Some("exactly one matching commitment custody"),
                    |value| {
                        append_distinct_custody_record(
                            value["holdout"]["attempts"][attempt_index]["custody_attestations"]
                                .as_array_mut()
                                .unwrap(),
                            custody_index,
                            &[
                                ("scope", Value::String("study-freshness".to_owned())),
                                (
                                    "ref",
                                    Value::String("custody:READER-EVIDENCE-FRESHNESS".to_owned()),
                                ),
                                ("freshness_attested", Value::Bool(true)),
                            ],
                        )
                        .expect("validated custody mutation");
                    },
                )?;
                controls += 1;
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "frozen holdout result injected",
                    Some("cannot carry run evidence or a result"),
                    |value| {
                        value["holdout"]["attempts"][attempt_index]["attempt_result"] =
                            Value::String("fail".to_owned());
                    },
                )?;
                controls += 1;
            }
            if let Some(index) = holdout_maps
                .iter()
                .position(|attempt| !attempt["result_receipt"].is_null())
            {
                for (label, field) in [
                    ("holdout result receipt self-digest drift", "receipt_sha256"),
                    (
                        "holdout result checker binding drift",
                        "structural_checker_sha256",
                    ),
                ] {
                    expect_validation_failure(
                        context,
                        source,
                        source_raw,
                        protocol_decision,
                        label,
                        Some("stale"),
                        |value| {
                            value["holdout"]["attempts"][index]["result_receipt"][field] =
                                Value::String("0".repeat(64));
                        },
                    )?;
                    controls += 1;
                }
            }
            if let Some(index) = holdout_maps
                .iter()
                .position(|attempt| !attempt["commitment_reveal"].is_null())
            {
                expect_validation_failure(
                    context,
                    source,
                    source_raw,
                    protocol_decision,
                    "commitment nonce reveal drift",
                    None,
                    |value| {
                        value["holdout"]["attempts"][index]["commitment_reveal"]["nonce_hex"] =
                            Value::String("00".repeat(32));
                    },
                )?;
                controls += 1;
            }
            if let Some(index) = holdout_maps
                .iter()
                .position(|attempt| !attempt["gate_admission_receipt"].is_null())
            {
                for (label, field) in [
                    ("gate input receipt drift", "input_sha256"),
                    ("gate receipt self-digest drift", "receipt_sha256"),
                ] {
                    expect_validation_failure(
                        context,
                        source,
                        source_raw,
                        protocol_decision,
                        label,
                        Some("stale"),
                        |value| {
                            value["holdout"]["attempts"][index]["gate_admission_receipt"][field] =
                                Value::String("0".repeat(64));
                        },
                    )?;
                    controls += 1;
                }
            }
            if source["result"].as_str() != Some("not-run") {
                control!(
                    "persistent top-level result erased",
                    Some("latest completed"),
                    |value: &mut Value| {
                        value["result"] = Value::String("not-run".to_owned());
                    }
                );
            }
        }

        if pilot_maps.len() > 1 {
            let mut changed = source.clone();
            let previous = &changed["pilot"]["attempts"][pilot_maps.len() - 2];
            let terminal = if previous["attempt_status"].as_str() == Some("completed") {
                previous["decision_packet"]["freeze_binding"]["frozen_at"].clone()
            } else {
                previous["voided_at"].clone()
            };
            changed["pilot"]["attempts"][pilot_maps.len() - 1]["pre_registration"]["freeze_binding"]
                ["frozen_at"] = terminal;
            expect_reader_failure(
                "pilot successor chronology",
                Some("strictly follow"),
                validate_history_closure(context, object(&changed, "changed source")?, source_raw),
            )?;
            controls += 1;
        }
        if holdout_maps.len() > 1 {
            let mut changed = source.clone();
            let previous = &changed["holdout"]["attempts"][holdout_maps.len() - 2];
            let terminal = if !previous["commitment_reveal"].is_null() {
                previous["commitment_reveal"]["revealed_at"].clone()
            } else if previous["attempt_status"].as_str() == Some("completed") {
                previous["result_receipt"]["completed_at"].clone()
            } else {
                previous["voided_at"].clone()
            };
            changed["holdout"]["attempts"][holdout_maps.len() - 1]["pre_registration"]["freeze_binding"]
                ["frozen_at"] = terminal;
            expect_reader_failure(
                "holdout successor chronology",
                Some("strictly follow"),
                validate_history_closure(context, object(&changed, "changed source")?, source_raw),
            )?;
            controls += 1;
        }
        if !pilot_maps.is_empty() && !holdout_maps.is_empty() {
            let mut changed = source.clone();
            changed["holdout"]["attempts"][0]["attempt_id"] =
                changed["pilot"]["attempts"][0]["attempt_id"].clone();
            expect_reader_failure(
                "cross-study attempt identity reuse",
                Some("duplicate across"),
                validate_history_closure(context, object(&changed, "changed source")?, source_raw),
            )?;
            controls += 1;
        }
    }
    match parse_source(br#"{"result":"not-run","result":"pass"}"#) {
        Err(error) if error.to_string().contains("duplicate") => controls += 1,
        Err(error) => {
            return Err(ReaderError::new(format!(
                "negative control failed for the wrong reason: duplicate JSON object key: {error}"
            )));
        }
        Ok(_) => {
            return Err(ReaderError::new(
                "negative control did not fail: duplicate JSON object key",
            ));
        }
    }
    Ok(controls)
}

fn validate_state_tuple(
    threshold_status: &str,
    holdout_status: &str,
    result: &str,
    route_status: &str,
    posture: &str,
    disposition: &str,
    valid_pass: bool,
) -> ReaderResult<()> {
    if !["pending-pilot", "candidate", "author-ratified"].contains(&threshold_status) {
        return Err(ReaderError::new("state.threshold_status: expected one of"));
    }
    if !["not-frozen", "frozen", "completed", "void"].contains(&holdout_status) {
        return Err(ReaderError::new("state.holdout_status: expected one of"));
    }
    if !["not-run", "not-evaluable", "fail", "pass"].contains(&result) {
        return Err(ReaderError::new("state.result: expected one of"));
    }
    if !["unbuilt", "available"].contains(&route_status) {
        return Err(ReaderError::new("state.route_status: expected one of"));
    }
    if holdout_status == "not-frozen" && result != "not-run" {
        return Err(ReaderError::new(
            "an empty holdout history must remain not-run",
        ));
    }
    if holdout_status == "completed" && result == "not-run" {
        return Err(ReaderError::new(
            "completed holdout must preserve its result",
        ));
    }
    if threshold_status != "author-ratified" && holdout_status != "not-frozen" {
        return Err(ReaderError::new(
            "holdout state requires an author-ratified rule",
        ));
    }
    if route_status == "available" && threshold_status != "author-ratified" {
        return Err(ReaderError::new(
            "route availability requires an author-ratified rule",
        ));
    }
    if valid_pass
        && (holdout_status != "completed" || result != "pass" || route_status != "available")
    {
        return Err(ReaderError::new(
            "valid pass requires the active completed pass on an available route",
        ));
    }
    if holdout_status == "completed" && result == "pass" && !valid_pass {
        return Err(ReaderError::new(
            "an active completed pass must be the matching admitted pass",
        ));
    }
    let expected = if valid_pass {
        ("Evidenced", "none")
    } else if route_status == "available" {
        ("Unestablished", "evidence-pending")
    } else {
        ("Unestablished", "route-unbuilt")
    };
    if (posture, disposition) != expected {
        return Err(ReaderError::new(
            "claim state contradicts route and admitted evidence",
        ));
    }
    Ok(())
}

fn empty_evaluator_rule() -> ThresholdRule {
    ThresholdRule {
        rule_id: String::new(),
        severity_taxonomy: Vec::new(),
        misconceptions: Vec::new(),
        core_misconception_ids: Vec::new(),
        core_failure_mode: String::new(),
        repetition_unit: String::new(),
        denominator: String::new(),
        core_failure_threshold: ThresholdSpec {
            threshold_id: String::new(),
            metric: String::new(),
            operator: String::new(),
            value_kind: String::new(),
            value: String::new(),
            unit: String::new(),
            denominator: String::new(),
            scope_refs: Vec::new(),
            evaluator_ref: None,
        },
        required_target_thresholds: Vec::new(),
        non_core_thresholds: Vec::new(),
        minimum_evaluable_evidence: ThresholdSpec {
            threshold_id: String::new(),
            metric: String::new(),
            operator: String::new(),
            value_kind: String::new(),
            value: String::new(),
            unit: String::new(),
            denominator: String::new(),
            scope_refs: Vec::new(),
            evaluator_ref: None,
        },
        policies: ThresholdPolicies {
            missing: String::new(),
            ambiguous: String::new(),
            multiply_coded: String::new(),
            withdrawn: String::new(),
            excluded: String::new(),
            unclassified: String::new(),
            rounding: String::new(),
            coder_adjudication: String::new(),
        },
        evaluation_order: Vec::new(),
        aggregate_offset_prohibited: false,
        rule_sha256: String::new(),
    }
}

fn fixture_sessions(
    rule: &ThresholdRule,
    count: usize,
    study_id: &str,
    salt: &str,
) -> Vec<SessionRecord> {
    (0..count)
        .map(|index| SessionRecord {
            study_id: study_id.to_owned(),
            record_commitment_sha256: sha256(format!("reader-{salt}-{index}").as_bytes()),
            admissibility: "admissible".to_owned(),
            target_outcomes: REQUIRED_TARGETS
                .iter()
                .map(|(target_id, _)| TargetOutcome {
                    target_id: (*target_id).to_owned(),
                    status: "identified".to_owned(),
                    adjudication: "not-required".to_owned(),
                })
                .collect(),
            misconception_outcomes: rule
                .misconceptions
                .iter()
                .map(|item| MisconceptionOutcome {
                    misconception_id: item.misconception_id.clone(),
                    status: "absent".to_owned(),
                    occurrences: "0".to_owned(),
                    opportunities: "1".to_owned(),
                    adjudication: "not-required".to_owned(),
                })
                .collect(),
            deviation_ids: Vec::new(),
            custody_attestation_ids: Vec::new(),
        })
        .collect()
}

fn validated_fixture_sessions(
    records: &[SessionRecord],
    path: &str,
    study_id: &str,
    known_misconceptions: &BTreeSet<String>,
) -> ReaderResult<Vec<SessionRecord>> {
    validate_sessions_typed(records, path, Some(study_id), Some(known_misconceptions))?;
    Ok(records.to_vec())
}

fn threshold_check<'a>(
    trace: &'a EvaluationTrace,
    threshold_id: &str,
) -> ReaderResult<&'a EvaluationCheck> {
    trace
        .stages
        .iter()
        .flat_map(|stage| &stage.checks)
        .find(|check| {
            matches!(
                check,
                EvaluationCheck::Threshold {
                    threshold_id: candidate,
                    ..
                } if candidate == threshold_id
            )
        })
        .ok_or_else(|| {
            ReaderError::new(format!(
                "boundary evaluator did not reach threshold {threshold_id}"
            ))
        })
}

fn parse_fixture_integer(value: &str, path: &str) -> ReaderResult<usize> {
    value.parse::<usize>().map_err(|_| {
        ReaderError::new(format!(
            "{path}: threshold is too large for an executable in-memory fixture"
        ))
    })
}

fn passing_fixture_count(spec: &ThresholdSpec, path: &str) -> ReaderResult<usize> {
    let boundary = parse_fixture_integer(&spec.value, path)?;
    if spec.operator == "gt" {
        boundary
            .checked_add(1)
            .ok_or_else(|| ReaderError::new(format!("{path}: threshold is too large")))
    } else {
        Ok(boundary)
    }
}

fn expect_evaluator_trace(
    rule: &ThresholdRule,
    label: &str,
    records: &[SessionRecord],
    expected_verdict: &str,
    failed_stage: Option<&str>,
    protocol_validity: &str,
) -> ReaderResult<()> {
    let trace = evaluate_holdout(rule, records, protocol_validity)?;
    if trace.verdict != expected_verdict {
        return Err(ReaderError::new(format!(
            "{label}: evaluator returned {}, expected {expected_verdict}",
            trace.verdict
        )));
    }
    if let Some(failed_stage) = failed_stage {
        if trace
            .stages
            .iter()
            .find(|stage| stage.stage == failed_stage)
            .is_none_or(|stage| stage.status != "fail")
        {
            return Err(ReaderError::new(format!(
                "{label}: evaluator did not fail at {failed_stage}"
            )));
        }
    }
    Ok(())
}

fn derived_boundary_evaluator_controls(
    rule: &ThresholdRule,
    known_misconceptions: &BTreeSet<String>,
) -> ReaderResult<usize> {
    #[derive(Clone, Copy)]
    enum Kind<'a> {
        Minimum,
        Target(&'a str),
        Core,
        Severity(&'a str),
    }

    let mut baseline_count = passing_fixture_count(
        &rule.minimum_evaluable_evidence,
        "boundary minimum threshold",
    )?
    .max(1);
    for entry in &rule.required_target_thresholds {
        if entry.threshold.metric == "target-identification-count" {
            baseline_count = baseline_count.max(passing_fixture_count(
                &entry.threshold,
                "boundary target threshold",
            )?);
        }
    }
    let core_ids: BTreeSet<_> = rule.core_misconception_ids.iter().cloned().collect();
    let mut severity_ids: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for item in &rule.misconceptions {
        if !item.core {
            severity_ids
                .entry(&item.severity_id)
                .or_default()
                .insert(item.misconception_id.clone());
        }
    }
    let mut entries = vec![(Kind::Minimum, &rule.minimum_evaluable_evidence)];
    entries.extend(
        rule.required_target_thresholds
            .iter()
            .map(|entry| (Kind::Target(entry.target_id.as_str()), &entry.threshold)),
    );
    entries.push((Kind::Core, &rule.core_failure_threshold));
    entries.extend(
        rule.non_core_thresholds
            .iter()
            .map(|entry| (Kind::Severity(entry.severity_id.as_str()), &entry.threshold)),
    );

    let study_id = "RE-STUDY-BOUNDARY-CONTROL";
    let mut controls = 0;
    for (kind, spec) in entries {
        let scoped_ids: &BTreeSet<String> = match kind {
            Kind::Core => &core_ids,
            Kind::Severity(severity_id) => severity_ids.get(severity_id).ok_or_else(|| {
                ReaderError::new("boundary fixture has no scoped misconception outcomes")
            })?,
            _ => &core_ids,
        };
        if spec.value_kind == "qualitative" {
            for token in ["absent", "present"] {
                let mut records = fixture_sessions(
                    rule,
                    baseline_count,
                    study_id,
                    &format!("boundary-{}-{token}", spec.threshold_id),
                );
                if token == "present" {
                    let outcome = records
                        .iter_mut()
                        .flat_map(|record| &mut record.misconception_outcomes)
                        .find(|outcome| core_ids.contains(&outcome.misconception_id))
                        .ok_or_else(|| {
                            ReaderError::new(
                                "boundary fixture has no scoped misconception outcomes",
                            )
                        })?;
                    outcome.status = "present".to_owned();
                    outcome.occurrences = "1".to_owned();
                }
                let records = validated_fixture_sessions(
                    &records,
                    &format!("boundary fixture {}/{token}", spec.threshold_id),
                    study_id,
                    known_misconceptions,
                )?;
                let trace = evaluate_holdout(rule, &records, "valid")?;
                let EvaluationCheck::Threshold {
                    observed,
                    comparison,
                    ..
                } = threshold_check(&trace, &spec.threshold_id)?
                else {
                    unreachable!()
                };
                if observed != token || *comparison != Some(token == spec.value) {
                    return Err(ReaderError::new(format!(
                        "qualitative boundary control failed for {}/{token}",
                        spec.threshold_id
                    )));
                }
                controls += 1;
            }
            continue;
        }

        let (boundary_digits, scale) = decimal_ratio(&spec.value);
        let (denominator, exact_numerator) = if spec.value_kind == "integer" {
            let boundary = parse_fixture_integer(&spec.value, &spec.threshold_id)?;
            if boundary == 0 {
                return Err(ReaderError::new(format!(
                    "integer boundary lacks three reachable cases for {}",
                    spec.threshold_id
                )));
            }
            (None, boundary)
        } else {
            let unit = 10_usize.checked_pow(scale as u32).ok_or_else(|| {
                ReaderError::new(format!(
                    "decimal boundary is too precise for fixture {}",
                    spec.threshold_id
                ))
            })?;
            let mut minimum_denominator = baseline_count;
            if matches!(kind, Kind::Core) && rule.repetition_unit == "coded-opportunity" {
                minimum_denominator = minimum_denominator
                    .checked_mul(core_ids.len())
                    .ok_or_else(|| ReaderError::new("boundary fixture size overflow"))?;
            }
            if matches!(kind, Kind::Severity(_)) && spec.metric.starts_with("severity-occurrence-")
            {
                minimum_denominator = minimum_denominator
                    .checked_mul(scoped_ids.len())
                    .ok_or_else(|| ReaderError::new("boundary fixture size overflow"))?;
            }
            let multiplier = minimum_denominator.div_ceil(unit).max(1);
            let denominator = unit
                .checked_mul(multiplier)
                .ok_or_else(|| ReaderError::new("boundary fixture size overflow"))?;
            let boundary_numerator =
                parse_fixture_integer(&boundary_digits.to_string(), &spec.threshold_id)?;
            let exact = boundary_numerator
                .checked_mul(multiplier)
                .ok_or_else(|| ReaderError::new("boundary fixture size overflow"))?;
            if exact == 0 || exact >= denominator {
                return Err(ReaderError::new(format!(
                    "decimal boundary lacks three reachable cases for {}",
                    spec.threshold_id
                )));
            }
            (Some(denominator), exact)
        };
        let points = [
            ("below", exact_numerator - 1),
            ("exact", exact_numerator),
            ("above", exact_numerator + 1),
        ];
        for (position, numerator) in points {
            let rate_metric = spec.metric.ends_with("-rate")
                && (spec.metric.starts_with("target-")
                    || spec.metric.starts_with("severity-session-")
                    || (spec.metric.starts_with("core-")
                        && rule.repetition_unit == "admissible-session"));
            let count_metric = matches!(
                spec.metric.as_str(),
                "target-identification-count" | "severity-session-finding-count"
            ) || (spec.metric == "core-finding-count"
                && rule.repetition_unit == "admissible-session");
            let session_count = match kind {
                Kind::Minimum => numerator,
                _ if rate_metric => denominator.ok_or_else(|| {
                    ReaderError::new("rate boundary fixture lost its denominator")
                })?,
                _ if count_metric => baseline_count.max(numerator).max(1),
                _ => baseline_count,
            };
            let mut records = fixture_sessions(
                rule,
                session_count,
                study_id,
                &format!("boundary-{}-{position}-{numerator}", spec.threshold_id),
            );
            match kind {
                Kind::Target(target_id) => {
                    for (index, record) in records.iter_mut().enumerate() {
                        let outcome = record
                            .target_outcomes
                            .iter_mut()
                            .find(|outcome| outcome.target_id == target_id)
                            .expect("required target fixture");
                        outcome.status = if index < numerator {
                            "identified"
                        } else {
                            "not-identified"
                        }
                        .to_owned();
                    }
                }
                Kind::Core | Kind::Severity(_) => {
                    let session_metric = spec.metric.starts_with("severity-session-")
                        || (spec.metric.starts_with("core-")
                            && rule.repetition_unit == "admissible-session");
                    if session_metric {
                        for record in records.iter_mut().take(numerator) {
                            let outcome = record
                                .misconception_outcomes
                                .iter_mut()
                                .find(|outcome| scoped_ids.contains(&outcome.misconception_id))
                                .ok_or_else(|| {
                                    ReaderError::new(
                                        "boundary fixture has no scoped misconception outcomes",
                                    )
                                })?;
                            outcome.status = "present".to_owned();
                            outcome.occurrences = "1".to_owned();
                        }
                    } else {
                        let mut outcomes: Vec<_> = records
                            .iter_mut()
                            .flat_map(|record| &mut record.misconception_outcomes)
                            .filter(|outcome| scoped_ids.contains(&outcome.misconception_id))
                            .collect();
                        if outcomes.is_empty() {
                            return Err(ReaderError::new(
                                "boundary fixture has no scoped misconception outcomes",
                            ));
                        }
                        let total = denominator.unwrap_or_else(|| outcomes.len().max(numerator));
                        if total < outcomes.len() {
                            return Err(ReaderError::new(
                                "boundary opportunity denominator is not reachable",
                            ));
                        }
                        outcomes[0].opportunities = (1 + total - outcomes.len()).to_string();
                        let mut remaining = numerator;
                        for outcome in outcomes {
                            let capacity = parse_fixture_integer(
                                &outcome.opportunities,
                                "boundary opportunity",
                            )?;
                            let observed = remaining.min(capacity);
                            outcome.occurrences = observed.to_string();
                            outcome.status =
                                if observed == 0 { "absent" } else { "present" }.to_owned();
                            remaining -= observed;
                        }
                        if remaining != 0 {
                            return Err(ReaderError::new(
                                "boundary occurrence numerator exceeds its denominator",
                            ));
                        }
                    }
                }
                Kind::Minimum => {}
            }
            let records = validated_fixture_sessions(
                &records,
                &format!("boundary fixture {}/{position}", spec.threshold_id),
                study_id,
                known_misconceptions,
            )?;
            let trace = evaluate_holdout(rule, &records, "valid")?;
            let EvaluationCheck::Threshold {
                observed,
                comparison,
                ..
            } = threshold_check(&trace, &spec.threshold_id)?
            else {
                unreachable!()
            };
            let expected_observed = denominator.map_or_else(
                || numerator.to_string(),
                |denominator| format!("{numerator}/{denominator}"),
            );
            let expected_comparison = if matches!(kind, Kind::Minimum) && numerator == 0 {
                None
            } else {
                let left = BigNat::from_usize(numerator).times_power_of_ten(scale);
                let right = denominator.map_or_else(
                    || boundary_digits.clone(),
                    |denominator| boundary_digits.product(&BigNat::from_usize(denominator)),
                );
                Some(threshold_comparison(&spec.operator, &left, &right)?)
            };
            if observed != &expected_observed || *comparison != expected_comparison {
                return Err(ReaderError::new(format!(
                    "end-to-end {position} boundary control failed for {}",
                    spec.threshold_id
                )));
            }
            controls += 1;
        }
    }
    Ok(controls)
}

fn derived_evaluator_controls(
    rule: &ThresholdRule,
    known_misconceptions: &BTreeSet<String>,
) -> ReaderResult<usize> {
    let mut session_count = 1_usize;
    let mut specs: Vec<(&ThresholdSpec, bool)> = vec![(&rule.minimum_evaluable_evidence, true)];
    specs.extend(
        rule.required_target_thresholds
            .iter()
            .map(|item| (&item.threshold, true)),
    );
    specs.push((&rule.core_failure_threshold, false));
    specs.extend(
        rule.non_core_thresholds
            .iter()
            .map(|item| (&item.threshold, false)),
    );
    for (spec, pass_high) in specs {
        if spec.value_kind != "integer" {
            continue;
        }
        let boundary = parse_fixture_integer(&spec.value, &spec.threshold_id)?;
        let increment = if pass_high {
            usize::from(spec.operator == "gt")
        } else {
            usize::from(matches!(spec.operator.as_str(), "gt" | "lte"))
        };
        session_count = session_count.max(
            boundary
                .checked_add(increment)
                .ok_or_else(|| ReaderError::new("evaluator fixture size overflow"))?,
        );
    }
    let study_id = "RE-STUDY-EVALUATOR-CONTROL";
    let raw_records = fixture_sessions(rule, session_count, study_id, "evaluator-fixture");
    let records = validated_fixture_sessions(
        &raw_records,
        "derived evaluator fixture",
        study_id,
        known_misconceptions,
    )?;
    let mut controls = 0;
    expect_evaluator_trace(
        rule,
        "favourable ratified fixture",
        &records,
        "pass",
        None,
        "valid",
    )?;
    controls += 1;
    expect_evaluator_trace(
        rule,
        "invalid protocol dominates favourable evidence",
        &records,
        "not-evaluable",
        Some("protocol-validity"),
        "invalid",
    )?;
    controls += 1;
    expect_evaluator_trace(
        rule,
        "zero admitted denominator",
        &[],
        "not-evaluable",
        Some("evaluability"),
        "valid",
    )?;
    controls += 1;

    let core_ids: BTreeSet<_> = rule.core_misconception_ids.iter().cloned().collect();
    let mut core_records = records.clone();
    for (record_index, record) in core_records.iter_mut().enumerate() {
        for outcome in &mut record.misconception_outcomes {
            if core_ids.contains(&outcome.misconception_id)
                && (rule.core_failure_mode == "repeated" || record_index == 0)
            {
                outcome.status = "present".to_owned();
                outcome.occurrences = "1".to_owned();
            }
        }
    }
    expect_evaluator_trace(
        rule,
        &format!("selected {} core veto", rule.core_failure_mode),
        &core_records,
        "fail",
        Some("core-veto"),
        "valid",
    )?;
    controls += 1;
    for (target_id, _) in REQUIRED_TARGETS {
        let mut target_records = records.clone();
        for record in &mut target_records {
            record
                .target_outcomes
                .iter_mut()
                .find(|outcome| outcome.target_id == *target_id)
                .expect("required target fixture")
                .status = "not-identified".to_owned();
        }
        expect_evaluator_trace(
            rule,
            &format!("required target boundary {target_id}"),
            &target_records,
            "fail",
            Some("required-targets"),
            "valid",
        )?;
        controls += 1;
    }
    let misconception_by_id: BTreeMap<_, _> = rule
        .misconceptions
        .iter()
        .map(|item| (item.misconception_id.as_str(), item))
        .collect();
    for entry in &rule.non_core_thresholds {
        let mut severity_records = records.clone();
        for record in &mut severity_records {
            for outcome in &mut record.misconception_outcomes {
                let item = misconception_by_id[outcome.misconception_id.as_str()];
                if item.severity_id == entry.severity_id && !item.core {
                    outcome.status = "present".to_owned();
                    outcome.occurrences = "1".to_owned();
                }
            }
        }
        expect_evaluator_trace(
            rule,
            &format!("non-core boundary {}", entry.severity_id),
            &severity_records,
            "fail",
            Some("non-core-rules"),
            "valid",
        )?;
        controls += 1;
    }
    let first_target = REQUIRED_TARGETS[0].0;
    for (policy_key, status) in [
        ("missing", "missing"),
        ("ambiguous", "ambiguous"),
        ("multiply_coded", "multiply-coded"),
        ("unclassified", "unclassified"),
    ] {
        let mut policy_records = records.clone();
        for record in &mut policy_records {
            let outcome = record
                .target_outcomes
                .iter_mut()
                .find(|outcome| outcome.target_id == first_target)
                .expect("required target fixture");
            outcome.status = status.to_owned();
            outcome.adjudication = "not-required".to_owned();
        }
        let action = resolved_policy_action(
            status,
            "not-required",
            &rule.policies,
            &["identified", "not-identified"],
        );
        let expected = if action == "count-adverse" {
            "fail"
        } else {
            "not-evaluable"
        };
        let stage = if expected == "fail" {
            "required-targets"
        } else {
            "evaluability"
        };
        expect_evaluator_trace(
            rule,
            &format!("ratified {policy_key} policy"),
            &policy_records,
            expected,
            Some(stage),
            "valid",
        )?;
        controls += 1;
    }
    let mut coder_records = records.clone();
    for record in &mut coder_records {
        let outcome = record
            .target_outcomes
            .iter_mut()
            .find(|outcome| outcome.target_id == first_target)
            .expect("required target fixture");
        outcome.status = "ambiguous".to_owned();
        outcome.adjudication = "unresolved".to_owned();
    }
    let coder_action = resolved_policy_action(
        "ambiguous",
        "unresolved",
        &rule.policies,
        &["identified", "not-identified"],
    );
    let coder_expected = if coder_action == "count-adverse" {
        "fail"
    } else {
        "not-evaluable"
    };
    expect_evaluator_trace(
        rule,
        "ratified coder-adjudication policy",
        &coder_records,
        coder_expected,
        Some(if coder_expected == "fail" {
            "required-targets"
        } else {
            "evaluability"
        }),
        "valid",
    )?;
    controls += 1;

    let mut excluded_records = raw_records;
    for (suffix, admissibility) in [("WITHDRAWN", "withdrawn"), ("EXCLUDED", "inadmissible")] {
        excluded_records.push(SessionRecord {
            study_id: study_id.to_owned(),
            record_commitment_sha256: sha256(
                format!("reader-evaluator-fixture-{suffix}").as_bytes(),
            ),
            admissibility: admissibility.to_owned(),
            target_outcomes: Vec::new(),
            misconception_outcomes: Vec::new(),
            deviation_ids: Vec::new(),
            custody_attestation_ids: Vec::new(),
        });
    }
    let excluded_records = validated_fixture_sessions(
        &excluded_records,
        "derived exclusion fixture",
        study_id,
        known_misconceptions,
    )?;
    expect_evaluator_trace(
        rule,
        "withdrawn and excluded sessions stay outside denominators",
        &excluded_records,
        "pass",
        None,
        "valid",
    )?;
    controls += 1;
    controls += derived_boundary_evaluator_controls(rule, known_misconceptions)?;
    Ok(controls)
}

fn executable_controls(source: &ReaderEvidenceSource) -> ReaderResult<usize> {
    let mut controls = 0;
    let pilot_study_id = "RE-PILOT-FRESHNESS-CONTROL";
    let freshness_value = serde_json::json!({
        "study_id": pilot_study_id,
        "scope": "study-freshness",
        "freshness_attested": true,
    });
    let freshness_item = object(&freshness_value, "pilot freshness fixture")?;
    let freshness = BTreeMap::from([("RE-CUSTODY-PILOT-FRESHNESS", freshness_item)]);
    validate_pilot_run_freshness(
        &freshness,
        "pilot freshness executable control",
        Some(pilot_study_id),
        true,
        true,
    )?;
    controls += 1;
    let empty_custody = BTreeMap::new();
    if validate_pilot_run_freshness(
        &empty_custody,
        "pilot freshness missing",
        Some(pilot_study_id),
        true,
        true,
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: pilot freshness missing",
        ));
    }
    controls += 1;
    let duplicate_value = freshness_value.clone();
    let duplicate_item = object(&duplicate_value, "duplicate pilot freshness fixture")?;
    let duplicate_freshness = BTreeMap::from([
        ("RE-CUSTODY-PILOT-FRESHNESS", freshness_item),
        ("RE-CUSTODY-PILOT-FRESHNESS-DUPLICATE", duplicate_item),
    ]);
    if validate_pilot_run_freshness(
        &duplicate_freshness,
        "pilot freshness duplicate",
        Some(pilot_study_id),
        true,
        true,
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: pilot freshness duplicate",
        ));
    }
    controls += 1;
    let wrong_value = serde_json::json!({
        "study_id": "RE-PILOT-WRONG-STUDY",
        "scope": "study-freshness",
        "freshness_attested": true,
    });
    let wrong_item = object(&wrong_value, "wrong-study freshness fixture")?;
    let wrong = BTreeMap::from([("RE-CUSTODY-PILOT-FRESHNESS", wrong_item)]);
    if validate_pilot_run_freshness(
        &wrong,
        "pilot freshness wrong study",
        Some(pilot_study_id),
        true,
        true,
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: pilot freshness wrong study",
        ));
    }
    controls += 1;
    let false_value = serde_json::json!({
        "study_id": pilot_study_id,
        "scope": "study-freshness",
        "freshness_attested": false,
    });
    let false_item = object(&false_value, "false freshness fixture")?;
    let false_freshness = BTreeMap::from([("RE-CUSTODY-PILOT-FRESHNESS", false_item)]);
    if validate_pilot_run_freshness(
        &false_freshness,
        "completed pilot freshness false",
        Some(pilot_study_id),
        true,
        true,
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: completed pilot freshness false",
        ));
    }
    controls += 1;

    let commitment_digest = "a".repeat(64);
    let commitment_value = serde_json::json!({
        "custody_attestation_sha256": commitment_digest,
    });
    let commitment = object(&commitment_value, "frozen commitment fixture")?;
    let commitment_custody_value = serde_json::json!({
        "scope": "commitment",
        "sha256": commitment_digest,
    });
    let commitment_custody_item = object(
        &commitment_custody_value,
        "frozen commitment custody fixture",
    )?;
    let commitment_custody =
        BTreeMap::from([("RE-CUSTODY-HOLDOUT-COMMITMENT", commitment_custody_item)]);
    let no_sessions: Vec<&Map<String, Value>> = Vec::new();
    let no_deviations = BTreeMap::new();
    validate_frozen_holdout_payload(
        "not-run",
        &no_sessions,
        &no_deviations,
        &commitment_custody,
        false,
        false,
        false,
        Some(commitment),
        "frozen holdout executable control",
    )?;
    controls += 1;
    if validate_frozen_holdout_payload(
        "not-run",
        &no_sessions,
        &no_deviations,
        &BTreeMap::new(),
        false,
        false,
        false,
        Some(commitment),
        "frozen commitment custody missing",
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: frozen commitment custody missing",
        ));
    }
    controls += 1;
    let duplicate_commitment_custody = BTreeMap::from([
        ("RE-CUSTODY-HOLDOUT-COMMITMENT", commitment_custody_item),
        (
            "RE-CUSTODY-HOLDOUT-COMMITMENT-DUPLICATE",
            commitment_custody_item,
        ),
    ]);
    if validate_frozen_holdout_payload(
        "not-run",
        &no_sessions,
        &no_deviations,
        &duplicate_commitment_custody,
        false,
        false,
        false,
        Some(commitment),
        "frozen commitment custody duplicate",
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: frozen commitment custody duplicate",
        ));
    }
    controls += 1;
    let freshness_custody_value = serde_json::json!({
        "scope": "study-freshness",
        "sha256": "b".repeat(64),
    });
    let freshness_custody_item = object(&freshness_custody_value, "frozen freshness fixture")?;
    let frozen_freshness = BTreeMap::from([
        ("RE-CUSTODY-HOLDOUT-COMMITMENT", commitment_custody_item),
        ("RE-CUSTODY-HOLDOUT-FRESHNESS", freshness_custody_item),
    ]);
    if validate_frozen_holdout_payload(
        "not-run",
        &no_sessions,
        &no_deviations,
        &frozen_freshness,
        false,
        false,
        false,
        Some(commitment),
        "frozen holdout freshness evidence",
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: frozen holdout freshness evidence",
        ));
    }
    controls += 1;
    let session_value = serde_json::json!({});
    let session_item = object(&session_value, "frozen session fixture")?;
    if validate_frozen_holdout_payload(
        "not-run",
        &[session_item],
        &no_deviations,
        &commitment_custody,
        false,
        false,
        false,
        Some(commitment),
        "frozen holdout session evidence",
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: frozen holdout session evidence",
        ));
    }
    controls += 1;
    let deviation_value = serde_json::json!({});
    let deviation_item = object(&deviation_value, "frozen deviation fixture")?;
    let one_deviation = BTreeMap::from([("RE-DEV-MUTATION", deviation_item)]);
    if validate_frozen_holdout_payload(
        "not-run",
        &no_sessions,
        &one_deviation,
        &commitment_custody,
        false,
        false,
        false,
        Some(commitment),
        "frozen holdout deviation evidence",
    )
    .is_ok()
    {
        return Err(ReaderError::new(
            "negative control did not fail: frozen holdout deviation evidence",
        ));
    }
    controls += 1;
    for (label, result, receipt_present) in [
        ("result", "fail", false),
        ("result receipt", "not-run", true),
    ] {
        if validate_frozen_holdout_payload(
            result,
            &no_sessions,
            &no_deviations,
            &commitment_custody,
            receipt_present,
            false,
            false,
            Some(commitment),
            &format!("frozen holdout {label} evidence"),
        )
        .is_ok()
        {
            return Err(ReaderError::new(format!(
                "negative control did not fail: frozen holdout {label} evidence"
            )));
        }
        controls += 1;
    }

    let invalid_states = [
        (
            "pending-pilot",
            "not-frozen",
            "pass",
            "unbuilt",
            "Unestablished",
            "route-unbuilt",
            false,
        ),
        (
            "pending-pilot",
            "void",
            "not-run",
            "unbuilt",
            "Unestablished",
            "route-unbuilt",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "not-run",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "fail",
            "available",
            "Unestablished",
            "route-unbuilt",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "pass",
            "available",
            "Unestablished",
            "evidence-pending",
            true,
        ),
        (
            "author-ratified",
            "completed",
            "pass",
            "unbuilt",
            "Evidenced",
            "none",
            true,
        ),
        (
            "author-ratified",
            "completed",
            "pass",
            "available",
            "Evidenced",
            "none",
            false,
        ),
        (
            "author-ratified",
            "frozen",
            "fail",
            "available",
            "Evidenced",
            "none",
            true,
        ),
    ];
    for state in invalid_states {
        if validate_state_tuple(
            state.0, state.1, state.2, state.3, state.4, state.5, state.6,
        )
        .is_ok()
        {
            return Err(ReaderError::new(
                "negative control did not fail: invalid state transition",
            ));
        }
        controls += 1;
    }
    let valid_states = [
        (
            "pending-pilot",
            "not-frozen",
            "not-run",
            "unbuilt",
            "Unestablished",
            "route-unbuilt",
            false,
        ),
        (
            "author-ratified",
            "frozen",
            "not-run",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "frozen",
            "fail",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "frozen",
            "pass",
            "unbuilt",
            "Unestablished",
            "route-unbuilt",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "not-evaluable",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "fail",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "completed",
            "pass",
            "available",
            "Evidenced",
            "none",
            true,
        ),
        (
            "author-ratified",
            "void",
            "not-run",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
        (
            "author-ratified",
            "void",
            "fail",
            "available",
            "Unestablished",
            "evidence-pending",
            false,
        ),
    ];
    for state in valid_states {
        validate_state_tuple(
            state.0, state.1, state.2, state.3, state.4, state.5, state.6,
        )?;
        controls += 1;
    }
    let stage_cases = [
        (
            false,
            true,
            false,
            true,
            true,
            "not-evaluable",
            Some("protocol-validity"),
        ),
        (
            true,
            false,
            false,
            true,
            true,
            "not-evaluable",
            Some("evaluability"),
        ),
        (true, true, true, true, true, "fail", Some("core-veto")),
        (
            true,
            true,
            false,
            false,
            true,
            "fail",
            Some("required-targets"),
        ),
        (
            true,
            true,
            false,
            true,
            false,
            "fail",
            Some("non-core-rules"),
        ),
        (true, true, false, true, true, "pass", None),
    ];
    for (protocol, evaluable, core, targets, non_core, expected, failed) in stage_cases {
        let checks = EVALUATION_ORDER
            .iter()
            .map(|stage| (*stage, Vec::new()))
            .collect();
        let trace = ordered_evaluation_trace(protocol, evaluable, core, targets, non_core, checks);
        if trace.verdict != expected {
            return Err(ReaderError::new(
                "ordered evaluator produced the wrong verdict",
            ));
        }
        if let Some(failed) = failed {
            let failed_index = EVALUATION_ORDER
                .iter()
                .position(|stage| *stage == failed)
                .unwrap();
            if trace.stages[failed_index].status != "fail"
                || trace.stages[failed_index + 1..]
                    .iter()
                    .any(|stage| stage.status != "not-reached")
            {
                return Err(ReaderError::new(
                    "ordered evaluator failed at the wrong stage",
                ));
            }
        }
        controls += 1;
    }
    for policy_key in ["missing", "ambiguous", "multiply_coded", "unclassified"] {
        let status = if policy_key == "multiply_coded" {
            "multiply-coded"
        } else {
            policy_key
        };
        for action in [
            "count-adverse",
            "exclude-observation",
            "require-adjudication",
            "study-not-evaluable",
        ] {
            let mut policies = ThresholdPolicies {
                missing: "study-not-evaluable".to_owned(),
                ambiguous: "study-not-evaluable".to_owned(),
                multiply_coded: "study-not-evaluable".to_owned(),
                withdrawn: "exclude-session".to_owned(),
                excluded: "exclude-session".to_owned(),
                unclassified: "study-not-evaluable".to_owned(),
                rounding: "exact-decimal-no-rounding".to_owned(),
                coder_adjudication: "unresolved-not-evaluable".to_owned(),
            };
            match policy_key {
                "missing" => policies.missing = action.to_owned(),
                "ambiguous" => policies.ambiguous = action.to_owned(),
                "multiply_coded" => policies.multiply_coded = action.to_owned(),
                "unclassified" => policies.unclassified = action.to_owned(),
                _ => unreachable!(),
            }
            let observed = resolved_policy_action(
                status,
                "not-required",
                &policies,
                &["identified", "not-identified"],
            );
            let expected = if action == "require-adjudication" {
                "study-not-evaluable"
            } else {
                action
            };
            if observed != expected {
                return Err(ReaderError::new(format!(
                    "policy action failed for {policy_key}/{action}"
                )));
            }
            controls += 1;
        }
    }
    for (action, expected) in [
        ("unresolved-count-adverse", "count-adverse"),
        ("unresolved-exclude-observation", "exclude-observation"),
        ("unresolved-not-evaluable", "study-not-evaluable"),
    ] {
        let policies = ThresholdPolicies {
            missing: "study-not-evaluable".to_owned(),
            ambiguous: "study-not-evaluable".to_owned(),
            multiply_coded: "study-not-evaluable".to_owned(),
            withdrawn: "exclude-session".to_owned(),
            excluded: "exclude-session".to_owned(),
            unclassified: "study-not-evaluable".to_owned(),
            rounding: "exact-decimal-no-rounding".to_owned(),
            coder_adjudication: action.to_owned(),
        };
        if resolved_policy_action(
            "ambiguous",
            "unresolved",
            &policies,
            &["identified", "not-identified"],
        ) != expected
        {
            return Err(ReaderError::new(format!(
                "coder-adjudication action failed for {action}"
            )));
        }
        controls += 1;
    }
    if source.threshold_status == "pending-pilot" {
        let trace = evaluate_holdout(&empty_evaluator_rule(), &[], "invalid")?;
        if trace.verdict != "not-evaluable" {
            return Err(ReaderError::new(
                "invalid-protocol end-to-end evaluator control failed",
            ));
        }
        controls += 1;
    } else {
        let rule = source.threshold_rule.populated("threshold_rule")?;
        let specs = std::iter::once(&rule.core_failure_threshold)
            .chain(std::iter::once(&rule.minimum_evaluable_evidence))
            .chain(
                rule.required_target_thresholds
                    .iter()
                    .map(|item| &item.threshold),
            )
            .chain(rule.non_core_thresholds.iter().map(|item| &item.threshold));
        for spec in specs {
            if !matches!(spec.value_kind.as_str(), "integer" | "decimal") {
                continue;
            }
            let (boundary, _) = decimal_ratio(&spec.value);
            let below = boundary.decremented().ok_or_else(|| {
                ReaderError::new("validated numeric threshold lacks a below boundary")
            })?;
            let actual = [below, boundary.clone(), boundary.incremented()]
                .iter()
                .map(|observed| threshold_comparison(&spec.operator, observed, &boundary))
                .collect::<ReaderResult<Vec<_>>>()?;
            let expected = match spec.operator.as_str() {
                "lt" => [true, false, false],
                "lte" => [true, true, false],
                "eq" => [false, true, false],
                "gte" => [false, true, true],
                "gt" => [false, false, true],
                _ => {
                    return Err(ReaderError::new(
                        "validated numeric threshold has an unsupported operator",
                    ));
                }
            };
            if actual.as_slice() != expected {
                return Err(ReaderError::new(
                    "derived below/exact/above boundary control failed",
                ));
            }
            controls += 1;
        }
        let known_misconceptions = rule
            .misconceptions
            .iter()
            .map(|item| item.misconception_id.clone())
            .collect();
        controls += derived_evaluator_controls(&rule, &known_misconceptions)?;
    }
    Ok(controls)
}

fn insert_reader_enum(inventory: &mut BTreeSet<ReaderEnumEntry>, field: &'static str, value: &str) {
    inventory.insert(ReaderEnumEntry {
        field,
        value: value.to_owned(),
    });
}

fn collect_threshold_spec_enums(
    inventory: &mut BTreeSet<ReaderEnumEntry>,
    threshold: &ThresholdSpec,
) {
    for (field, value) in [
        ("metric", threshold.metric.as_str()),
        ("operator", threshold.operator.as_str()),
        ("value_kind", threshold.value_kind.as_str()),
        ("unit", threshold.unit.as_str()),
        ("denominator", threshold.denominator.as_str()),
    ] {
        insert_reader_enum(inventory, field, value);
    }
}

fn collect_threshold_rule_enums(inventory: &mut BTreeSet<ReaderEnumEntry>, rule: &ThresholdRule) {
    insert_reader_enum(inventory, "core_failure_mode", &rule.core_failure_mode);
    insert_reader_enum(inventory, "repetition_unit", &rule.repetition_unit);
    insert_reader_enum(inventory, "denominator", &rule.denominator);
    collect_threshold_spec_enums(inventory, &rule.core_failure_threshold);
    for item in &rule.required_target_thresholds {
        collect_threshold_spec_enums(inventory, &item.threshold);
    }
    for item in &rule.non_core_thresholds {
        collect_threshold_spec_enums(inventory, &item.threshold);
    }
    collect_threshold_spec_enums(inventory, &rule.minimum_evaluable_evidence);
    for (field, value) in [
        ("missing", rule.policies.missing.as_str()),
        ("ambiguous", rule.policies.ambiguous.as_str()),
        ("multiply_coded", rule.policies.multiply_coded.as_str()),
        ("withdrawn", rule.policies.withdrawn.as_str()),
        ("excluded", rule.policies.excluded.as_str()),
        ("unclassified", rule.policies.unclassified.as_str()),
        ("rounding", rule.policies.rounding.as_str()),
        (
            "coder_adjudication",
            rule.policies.coder_adjudication.as_str(),
        ),
    ] {
        insert_reader_enum(inventory, field, value);
    }
}

fn collect_reviewed_threshold_rule_enums(
    inventory: &mut BTreeSet<ReaderEnumEntry>,
    rule: &ReviewedThresholdRule,
) {
    for (field, value) in [
        ("core_failure_mode", rule.core_failure_mode.as_deref()),
        ("repetition_unit", rule.repetition_unit.as_deref()),
        ("denominator", rule.denominator.as_deref()),
    ] {
        if let Some(value) = value {
            insert_reader_enum(inventory, field, value);
        }
    }
    if let Some(threshold) = &rule.core_failure_threshold {
        collect_threshold_spec_enums(inventory, threshold);
    }
    for item in &rule.required_target_thresholds {
        collect_threshold_spec_enums(inventory, &item.threshold);
    }
    for item in &rule.non_core_thresholds {
        collect_threshold_spec_enums(inventory, &item.threshold);
    }
    if let Some(threshold) = &rule.minimum_evaluable_evidence {
        collect_threshold_spec_enums(inventory, threshold);
    }
    for (field, value) in [
        ("missing", rule.policies.missing.as_deref()),
        ("ambiguous", rule.policies.ambiguous.as_deref()),
        ("multiply_coded", rule.policies.multiply_coded.as_deref()),
        ("withdrawn", rule.policies.withdrawn.as_deref()),
        ("excluded", rule.policies.excluded.as_deref()),
        ("unclassified", rule.policies.unclassified.as_deref()),
        ("rounding", rule.policies.rounding.as_deref()),
        (
            "coder_adjudication",
            rule.policies.coder_adjudication.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            insert_reader_enum(inventory, field, value);
        }
    }
}

fn collect_session_enums(inventory: &mut BTreeSet<ReaderEnumEntry>, sessions: &[SessionRecord]) {
    for session in sessions {
        insert_reader_enum(inventory, "admissibility", &session.admissibility);
        for outcome in &session.target_outcomes {
            insert_reader_enum(inventory, "status", &outcome.status);
            insert_reader_enum(inventory, "adjudication", &outcome.adjudication);
        }
        for outcome in &session.misconception_outcomes {
            insert_reader_enum(inventory, "status", &outcome.status);
            insert_reader_enum(inventory, "adjudication", &outcome.adjudication);
        }
    }
}

fn collect_deviation_enums(
    inventory: &mut BTreeSet<ReaderEnumEntry>,
    deviations: &[DeviationRecord],
) {
    for deviation in deviations {
        insert_reader_enum(inventory, "code", &deviation.code);
        insert_reader_enum(inventory, "impact", &deviation.impact);
    }
}

fn collect_custody_enums(inventory: &mut BTreeSet<ReaderEnumEntry>, custody: &[CustodyRecord]) {
    for record in custody {
        insert_reader_enum(inventory, "scope", &record.scope);
    }
}

fn collect_pilot_enums(inventory: &mut BTreeSet<ReaderEnumEntry>, pilot: &PilotRecord) {
    insert_reader_enum(inventory, "pilot_status", &pilot.pilot_status);
    insert_reader_enum(inventory, "control_status", &pilot.control_status);
    for attempt in &pilot.attempts {
        insert_reader_enum(inventory, "attempt_status", &attempt.attempt_status);
        insert_reader_enum(inventory, "control_status", &attempt.control_status);
        if let Some(value) = &attempt.void_reason_code {
            insert_reader_enum(inventory, "void_reason_code", value);
        }
        if let Some(registration) = &attempt.pre_registration {
            insert_reader_enum(
                inventory,
                "binding_type",
                &registration.freeze_binding.binding_type,
            );
        }
        collect_session_enums(inventory, &attempt.session_records);
        collect_deviation_enums(inventory, &attempt.deviations);
        collect_custody_enums(inventory, &attempt.custody_attestations);
        if let Some(receipt) = &attempt.receipt {
            insert_reader_enum(inventory, "protocol_validity", &receipt.protocol_validity);
        }
        if let Some(packet) = &attempt.decision_packet {
            insert_reader_enum(
                inventory,
                "binding_type",
                &packet.freeze_binding.binding_type,
            );
        }
    }
}

fn collect_holdout_enums(inventory: &mut BTreeSet<ReaderEnumEntry>, holdout: &HoldoutRecord) {
    for attempt in &holdout.attempts {
        insert_reader_enum(inventory, "attempt_status", &attempt.attempt_status);
        insert_reader_enum(inventory, "attempt_result", &attempt.attempt_result);
        if let Some(value) = &attempt.void_reason_code {
            insert_reader_enum(inventory, "void_reason_code", value);
        }
        insert_reader_enum(
            inventory,
            "binding_type",
            &attempt.pre_registration.freeze_binding.binding_type,
        );
        collect_threshold_rule_enums(inventory, &attempt.frozen_rule);
        collect_session_enums(inventory, &attempt.session_records);
        collect_deviation_enums(inventory, &attempt.deviations);
        collect_custody_enums(inventory, &attempt.custody_attestations);
        if let Some(receipt) = &attempt.result_receipt {
            insert_reader_enum(inventory, "protocol_validity", &receipt.protocol_validity);
            insert_reader_enum(inventory, "verdict", &receipt.verdict);
        }
    }
}

fn reader_enum_inventory(source: &ReaderEvidenceSource) -> BTreeSet<ReaderEnumEntry> {
    let mut inventory = BTreeSet::new();
    for (field, value) in [
        ("threshold_status", source.threshold_status.as_str()),
        ("holdout_status", source.holdout_status.as_str()),
        ("result", source.result.as_str()),
        ("route_status", source.route.route_status.as_str()),
        (
            "evidence_contract_status",
            source.route.evidence_contract_status.as_str(),
        ),
        (
            "negative_control_status",
            source.route.negative_control_status.as_str(),
        ),
        ("posture", source.claim.posture.as_str()),
        ("disposition", source.claim.disposition.as_str()),
    ] {
        insert_reader_enum(&mut inventory, field, value);
    }
    if let Some(attestation) = &source.route.reviewer_custody_attestation {
        insert_reader_enum(&mut inventory, "scope", &attestation.scope);
    }
    collect_pilot_enums(&mut inventory, &source.pilot);
    collect_reviewed_threshold_rule_enums(&mut inventory, &source.threshold_rule);
    collect_holdout_enums(&mut inventory, &source.holdout);
    inventory
}

/// Load and validate reader evidence once, then return only the typed fields
/// needed by script 13's native sibling-enum and alignment checks.
pub(crate) fn load_validated_reader_evidence(
    context: &Context,
    snapshot: InputSnapshot<'_>,
) -> Result<ReaderLedgerProjection, ReaderError> {
    let source_owned;
    let source_raw = if let Some(value) = snapshot.source_json {
        value
    } else {
        source_owned = read_bytes(context, DEFAULT_SOURCE, "reader-evidence source")?;
        &source_owned
    };
    let protocol_owned;
    let protocol_decision = if let Some(value) = snapshot.protocol_decision {
        value
    } else {
        protocol_owned = read_bytes(context, PROTOCOL_DECISION, "protocol decision")?;
        &protocol_owned
    };
    parse_source(source_raw)?;
    let source = typed_reader_source_bytes(source_raw, "root")?;
    let validation = validate_source_typed(context, &source, source_raw, protocol_decision)?;
    Ok(ReaderLedgerProjection {
        enum_inventory: reader_enum_inventory(&source),
        route_id: source.route.route_id,
        route_status: source.route.route_status,
        claim_id: source.claim.claim_id,
        claim_posture: source.claim.posture,
        claim_disposition: source.claim.disposition,
        result: source.result,
        valid_holdout_pass: validation.valid_holdout_pass,
    })
}

/// Apply script 13's R6/FS-CLM-37 alignment rules without traversing the
/// reader source as a generic JSON value.
pub(crate) fn validate_reader_evidence_alignment(
    reader: &ReaderLedgerProjection,
    route: ReaderRouteAlignment<'_>,
    claim: ReaderClaimAlignment<'_>,
) -> Result<(), ReaderError> {
    if route.route_status == "built" {
        return Err(ReaderError::new(
            "reader alignment: FS-RTE-06 is an external route and may never take the in-repository built status",
        ));
    }
    if route.status != route.route_status {
        return Err(ReaderError::new(
            "reader alignment: FS-RTE-06 status and route_status must agree",
        ));
    }
    if reader.route_id != route.id {
        return Err(ReaderError::new(
            "reader alignment: reader source names the wrong route",
        ));
    }
    if reader.route_status != route.route_status {
        return Err(ReaderError::new(
            "reader alignment: FS-RTE-06 route_status must match reader-evidence.json",
        ));
    }
    if claim.route_ref != route.id {
        return Err(ReaderError::new(
            "reader alignment: FS-CLM-37 must use FS-RTE-06",
        ));
    }
    if reader.claim_id != claim.id {
        return Err(ReaderError::new(
            "reader alignment: reader source names the wrong claim",
        ));
    }
    let expected = if reader.valid_holdout_pass {
        ("Evidenced", "none")
    } else if route.route_status == "available" {
        ("Unestablished", "evidence-pending")
    } else {
        ("Unestablished", "route-unbuilt")
    };
    if (
        reader.claim_posture.as_str(),
        reader.claim_disposition.as_str(),
    ) != expected
    {
        return Err(ReaderError::new(
            "reader alignment: reviewed reader claim contradicts its validated route and holdout state",
        ));
    }
    let ledger_state = (
        claim.posture,
        claim.unestablished_disposition.unwrap_or("none"),
    );
    if ledger_state != expected {
        return Err(ReaderError::new(format!(
            "reader alignment: FS-CLM-37 must be {}/{} for the live reader-evidence state",
            expected.0, expected.1
        )));
    }
    if reader.result == "fail"
        && reader.route_status == "available"
        && ledger_state != ("Unestablished", "evidence-pending")
    {
        return Err(ReaderError::new(
            "reader alignment: a persisted failure on an available route requires Unestablished/evidence-pending; active holdout status may not rewrite it as not-run",
        ));
    }
    Ok(())
}

fn read_bytes(context: &Context, path: &str, label: &str) -> ReaderResult<Vec<u8>> {
    std::fs::read(context.path(path)).map_err(|error| {
        ReaderError::new(format!(
            "cannot read {label} {}: {error}",
            context.path(path).display()
        ))
    })
}

fn atomic_write(context: &Context, value: &str) -> ReaderResult<()> {
    let output = context.path(DEFAULT_OUTPUT);
    if std::fs::symlink_metadata(&output).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(ReaderError::new("generated report may not be a symlink"));
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            ReaderError::new(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let temporary = output.with_file_name(format!(
        "{}.tmp-{}",
        output.file_name().unwrap().to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&temporary, value).map_err(|error| {
        ReaderError::new(format!("cannot write {}: {error}", temporary.display()))
    })?;
    std::fs::rename(&temporary, &output).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        ReaderError::new(format!("cannot replace {}: {error}", output.display()))
    })?;
    Ok(())
}

fn run_inner(context: &Context, mode: Mode, snapshot: InputSnapshot<'_>) -> ReaderResult<String> {
    let source_owned;
    let source_raw = if let Some(value) = snapshot.source_json {
        value
    } else {
        source_owned = read_bytes(context, DEFAULT_SOURCE, "reader-evidence source")?;
        &source_owned
    };
    let protocol_owned;
    let protocol_decision = if let Some(value) = snapshot.protocol_decision {
        value
    } else {
        protocol_owned = read_bytes(context, PROTOCOL_DECISION, "protocol decision")?;
        &protocol_owned
    };
    let source_value = parse_source(source_raw)?;
    let source = typed_reader_source_bytes(source_raw, "root")?;
    validate_source_typed(context, &source, source_raw, protocol_decision)?;
    let generated = render_typed(&source, &sha256(source_raw))?;
    let structural = structural_controls(context, &source_value, source_raw, protocol_decision)?;
    let execute = matches!(mode, Mode::CheckExecute | Mode::GenerateExecute);
    let executed = if execute {
        executable_controls(&source)?
    } else {
        0
    };
    match mode {
        Mode::Check | Mode::CheckExecute => {
            let report_owned;
            let current = if let Some(value) = snapshot.generated_report {
                value
            } else {
                report_owned =
                    std::fs::read_to_string(context.path(DEFAULT_OUTPUT)).map_err(|error| {
                        ReaderError::new(format!(
                            "cannot read generated report {}: {error}",
                            context.path(DEFAULT_OUTPUT).display()
                        ))
                    })?;
                &report_owned
            };
            if current != generated {
                return Err(ReaderError::new(format!(
                    "{DEFAULT_OUTPUT} is STALE — rerun without --check"
                )));
            }
            let suffix = if execute {
                format!("; {executed} executable contract controls pass; no reader study executed")
            } else {
                "; execution skipped".to_owned()
            };
            Ok(format!(
                "{DEFAULT_OUTPUT} is current; {structural} watched-failing structural controls pass{suffix}"
            ))
        }
        Mode::Generate | Mode::GenerateExecute => {
            atomic_write(context, &generated)?;
            let suffix = if execute {
                format!("; {executed} executable contract controls pass")
            } else {
                String::new()
            };
            Ok(format!(
                "{DEFAULT_OUTPUT}: regenerated; {structural} watched-failing structural controls pass{suffix}"
            ))
        }
    }
}

pub(crate) fn run(
    context: &Context,
    mode: Mode,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run_inner(context, mode, snapshot)
        .map_err(|error| Error::new(format!("14-reader-evidence: {error}")))
}

pub(crate) fn check(
    context: &Context,
    execute: bool,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run(
        context,
        if execute {
            Mode::CheckExecute
        } else {
            Mode::Check
        },
        snapshot,
    )
}

pub(crate) fn generate(
    context: &Context,
    execute: bool,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run(
        context,
        if execute {
            Mode::GenerateExecute
        } else {
            Mode::Generate
        },
        snapshot,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn historical_artifact(identifier: &str) -> Artifact {
        Artifact {
            artifact_id: identifier.to_owned(),
            r#ref: "custody:HISTORICAL-FIXTURE".to_owned(),
            sha256: "1".repeat(64),
        }
    }

    fn historical_pilot_pre_registration() -> PilotPreRegistration {
        PilotPreRegistration {
            study_id: "RE-PILOT-HISTORICAL-FIXTURE".to_owned(),
            registered_date: "2026-01-01".to_owned(),
            predecessor_attempt_sha256: None,
            prior_history_head_sha256: "2".repeat(64),
            fixed_protocol_sha256: "3".repeat(64),
            protocol: historical_artifact("RE-ART-HISTORICAL-PROTOCOL"),
            instrument: historical_artifact("RE-ART-HISTORICAL-INSTRUMENT"),
            rubric: historical_artifact("RE-ART-HISTORICAL-RUBRIC"),
            sample_rule: historical_artifact("RE-ART-HISTORICAL-SAMPLE"),
            disclosure_set: historical_artifact("RE-ART-HISTORICAL-DISCLOSURE"),
            ethics_terms: historical_artifact("RE-ART-HISTORICAL-ETHICS"),
            provisional_rule: historical_artifact("RE-ART-HISTORICAL-RULE"),
            freeze_binding: FreezeBinding {
                binding_id: "RE-FRZ-HISTORICAL-FIXTURE".to_owned(),
                binding_type: "git-commit".to_owned(),
                bound_payload_sha256: "4".repeat(64),
                attested_payload_sha256: "4".repeat(64),
                r#ref: format!("git:{}", "5".repeat(40)),
                attestation_sha256: "6".repeat(64),
                frozen_at: "2026-01-01T00:00:00Z".to_owned(),
            },
            pre_registration_sha256: "7".repeat(64),
        }
    }

    fn live_typed_source() -> ReaderEvidenceSource {
        let context = Context::discover().expect("discover repository");
        let raw = std::fs::read(context.path(DEFAULT_SOURCE)).expect("read reader source");
        typed_reader_source_bytes(&raw, "historical fixture").expect("decode reader source")
    }

    fn evaluator_rule_fixture() -> ThresholdRule {
        let threshold =
            |id: &str, metric: &str, operator: &str, kind: &str, value: &str, scope: &[&str]| {
                ThresholdSpec {
                    threshold_id: id.to_owned(),
                    metric: metric.to_owned(),
                    operator: operator.to_owned(),
                    value_kind: kind.to_owned(),
                    value: value.to_owned(),
                    unit: if metric.ends_with("-rate") {
                        "proportion"
                    } else {
                        "count"
                    }
                    .to_owned(),
                    denominator: if metric.ends_with("-rate") {
                        "eligible-observations"
                    } else {
                        "none"
                    }
                    .to_owned(),
                    scope_refs: scope.iter().map(|item| (*item).to_owned()).collect(),
                    evaluator_ref: Some(
                        "new-book-plans/14-reader-evidence.py::def evaluate_holdout(".to_owned(),
                    ),
                }
            };
        ThresholdRule {
            rule_id: "RE-RULE-EVALUATOR-FIXTURE".to_owned(),
            severity_taxonomy: vec![
                SeverityDefinition {
                    severity_id: "RE-SEV-CORE".to_owned(),
                    label: "core".to_owned(),
                    definition: "core fixture".to_owned(),
                    classification_boundary: "core boundary".to_owned(),
                },
                SeverityDefinition {
                    severity_id: "RE-SEV-NONCORE".to_owned(),
                    label: "non-core".to_owned(),
                    definition: "non-core fixture".to_owned(),
                    classification_boundary: "non-core boundary".to_owned(),
                },
            ],
            misconceptions: vec![
                MisconceptionDefinition {
                    misconception_id: "RE-MIS-CORE".to_owned(),
                    definition: "core misconception".to_owned(),
                    severity_id: "RE-SEV-CORE".to_owned(),
                    core: true,
                },
                MisconceptionDefinition {
                    misconception_id: "RE-MIS-NONCORE".to_owned(),
                    definition: "non-core misconception".to_owned(),
                    severity_id: "RE-SEV-NONCORE".to_owned(),
                    core: false,
                },
            ],
            core_misconception_ids: vec!["RE-MIS-CORE".to_owned()],
            core_failure_mode: "any".to_owned(),
            repetition_unit: "admissible-session".to_owned(),
            denominator: "admissible-sessions".to_owned(),
            core_failure_threshold: threshold(
                "RE-THR-CORE",
                "core-finding-present",
                "eq",
                "qualitative",
                "present",
                &["RE-MIS-CORE"],
            ),
            required_target_thresholds: REQUIRED_TARGETS
                .iter()
                .enumerate()
                .map(|(index, (target_id, _))| TargetThreshold {
                    target_id: (*target_id).to_owned(),
                    threshold: threshold(
                        &format!("RE-THR-TARGET-{index}"),
                        "target-identification-rate",
                        "gte",
                        "decimal",
                        "0.5",
                        &[*target_id],
                    ),
                })
                .collect(),
            non_core_thresholds: vec![SeverityThreshold {
                severity_id: "RE-SEV-NONCORE".to_owned(),
                threshold: threshold(
                    "RE-THR-NONCORE",
                    "severity-session-finding-rate",
                    "lt",
                    "decimal",
                    "0.5",
                    &["RE-SEV-NONCORE"],
                ),
            }],
            minimum_evaluable_evidence: threshold(
                "RE-THR-MINIMUM",
                "admissible-session-count",
                "gte",
                "integer",
                "2",
                &[],
            ),
            policies: ThresholdPolicies {
                missing: "study-not-evaluable".to_owned(),
                ambiguous: "study-not-evaluable".to_owned(),
                multiply_coded: "study-not-evaluable".to_owned(),
                withdrawn: "exclude-session".to_owned(),
                excluded: "exclude-session".to_owned(),
                unclassified: "study-not-evaluable".to_owned(),
                rounding: "exact-decimal-no-rounding".to_owned(),
                coder_adjudication: "unresolved-not-evaluable".to_owned(),
            },
            evaluation_order: EVALUATION_ORDER
                .iter()
                .map(|item| (*item).to_owned())
                .collect(),
            aggregate_offset_prohibited: true,
            rule_sha256: "0".repeat(64),
        }
    }

    #[test]
    fn duplicate_keys_are_rejected_recursively() {
        let error = parse_source(br#"{"outer":{"value":1,"value":2}}"#).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: value")
        );
    }

    #[test]
    fn historical_reader_source_rejects_unknown_fields() {
        let mut source = serde_json::to_value(live_typed_source()).expect("encode reader source");
        source
            .as_object_mut()
            .expect("reader source object")
            .insert("unreviewed".to_owned(), Value::Bool(true));
        let bytes = serde_json::to_vec(&source).expect("encode historical source");
        let error = decode_historical_reader_evidence(&bytes, "historical source")
            .err()
            .expect("unknown historical field must fail");
        assert!(error.to_string().contains("unknown field `unreviewed`"));
    }

    #[test]
    fn historical_payload_in_an_untyped_location_cannot_establish_a_freeze() {
        let payload = historical_pilot_pre_registration();
        let expected = frozen_payload_sha(
            &payload,
            "historical pilot pre-registration",
            "pre_registration_sha256",
        )
        .expect("hash payload");
        let mut source = serde_json::to_value(live_typed_source()).expect("encode reader source");
        source["acceptance"]
            .as_object_mut()
            .expect("acceptance object")
            .insert(
                "unrelated_payload".to_owned(),
                serde_json::to_value(&payload).expect("encode payload"),
            );
        let bytes = serde_json::to_vec(&source).expect("encode historical source");
        let error = decode_historical_reader_evidence(&bytes, "historical source")
            .err()
            .expect("payload outside a typed slot must fail");
        assert!(
            error
                .to_string()
                .contains("unknown field `unrelated_payload`")
        );

        let historical = HistoricalReaderEvidence::V1(live_typed_source());
        assert!(
            !contains_typed_historical_payload(
                &historical,
                HistoricalPayloadKind::PilotPreRegistration,
                &expected,
                "pre_registration_sha256",
            )
            .expect("search typed payload locations")
        );
    }

    #[test]
    fn historical_payload_matches_its_explicit_typed_location() {
        let payload = historical_pilot_pre_registration();
        let expected = frozen_payload_sha(
            &payload,
            "historical pilot pre-registration",
            "pre_registration_sha256",
        )
        .expect("hash payload");
        let mut source = live_typed_source();
        source.pilot.attempts.push(PilotAttemptRecord {
            attempt_id: "RE-PILOT-ATTEMPT-HISTORICAL-FIXTURE".to_owned(),
            previous_attempt_sha256: None,
            attempt_status: "not-run".to_owned(),
            control_status: "not-run".to_owned(),
            void_reason_code: None,
            voided_at: None,
            prerequisites: Prerequisites {
                readers_map_ref: None,
                glossary_ref: None,
                accessible_navigation_ref: None,
            },
            pre_registration: Some(payload),
            tested_snapshot: None,
            session_records: Vec::new(),
            deviations: Vec::new(),
            custody_attestations: Vec::new(),
            receipt: None,
            decision_packet: None,
            sensitivity_brief: None,
            attempt_sha256: "8".repeat(64),
        });
        let bytes = serde_json::to_vec(&source).expect("encode historical source");
        let historical = decode_historical_reader_evidence(&bytes, "historical source")
            .expect("decode historical source");
        assert!(
            contains_typed_historical_payload(
                &historical,
                HistoricalPayloadKind::PilotPreRegistration,
                &expected,
                "pre_registration_sha256",
            )
            .expect("search typed payload locations")
        );
    }

    #[test]
    fn live_report_and_stdout_match() {
        let context = Context::discover().expect("discover repository");
        let output = check(&context, false, InputSnapshot::default()).expect("live check");
        assert_eq!(
            output,
            "new-book-plans/reader-evidence.md is current; 45 watched-failing structural controls pass; execution skipped"
        );
    }

    #[test]
    fn live_execute_stdout_matches() {
        let context = Context::discover().expect("discover repository");
        let output = check(&context, true, InputSnapshot::default()).expect("live execute check");
        assert_eq!(
            output,
            "new-book-plans/reader-evidence.md is current; 45 watched-failing structural controls pass; 56 executable contract controls pass; no reader study executed"
        );
    }

    #[test]
    fn typed_renderer_matches_the_reviewed_report_contract() {
        let context = Context::discover().expect("discover repository");
        let raw = std::fs::read(context.path(DEFAULT_SOURCE)).unwrap();
        let source_value = parse_source(&raw).unwrap();
        let source = typed_reader_source(&source_value, "root").unwrap();
        let digest = sha256(&raw);
        assert_eq!(
            render_typed(&source, &digest).unwrap(),
            render(&source_value, &digest).unwrap()
        );
    }

    #[test]
    fn representative_private_field_mutation_fails() {
        let context = Context::discover().expect("discover repository");
        let raw = std::fs::read(context.path(DEFAULT_SOURCE)).unwrap();
        let mut source = parse_source(&raw).unwrap();
        source.as_object_mut().unwrap().insert(
            "participant_name".to_owned(),
            Value::String("private".to_owned()),
        );
        let error = typed_reader_source(&source, "root").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unknown field `participant_name`")
        );
    }

    #[test]
    fn admission_gate_self_test_matches_cli_contract() {
        let context = Context::discover().expect("discover repository");
        assert_eq!(
            admission_gate_self_test(&context).expect("native gate self-test"),
            "reader-evidence-admission-gate: self-test passed"
        );
    }

    #[test]
    fn admission_gate_rejects_duplicate_keys_before_typed_decode() {
        let context = Context::discover().expect("discover repository");
        let protocol = std::fs::read(context.path(PROTOCOL_DECISION)).unwrap();
        let error = evaluate_gate_json(
            &context,
            &protocol,
            br#"{"schema_version":1,"schema_version":1}"#,
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: schema_version")
        );
    }

    #[test]
    fn admission_gate_typed_root_rejects_unknown_fields() {
        let error = typed_gate_input_bytes(br#"{"unreviewed":true}"#, "gate input")
            .expect_err("unknown gate field must fail");
        assert!(error.to_string().contains("unknown field `unreviewed`"));
    }

    #[test]
    fn strict_root_model_rejects_unknown_nested_fields() {
        let context = Context::discover().expect("discover repository");
        let raw = std::fs::read(context.path(DEFAULT_SOURCE)).unwrap();
        let mut source = parse_source(&raw).unwrap();
        source["protocol"].as_object_mut().unwrap().insert(
            "unreviewed".to_owned(),
            Value::String("not in the domain model".to_owned()),
        );
        let error = typed_reader_source(&source, "root").unwrap_err();
        assert!(error.to_string().contains("unknown field `unreviewed`"));
    }

    #[test]
    fn strict_root_model_requires_explicit_nullable_fields() {
        let context = Context::discover().expect("discover repository");
        let raw = std::fs::read(context.path(DEFAULT_SOURCE)).unwrap();
        let mut source = parse_source(&raw).unwrap();
        source["threshold_rule"]["policies"]
            .as_object_mut()
            .unwrap()
            .remove("ambiguous");
        let error = typed_reader_source(&source, "root").unwrap_err();
        assert!(error.to_string().contains("missing field `ambiguous`"));
    }

    #[test]
    fn typed_ledger_projection_matches_live_alignment_surface() {
        let context = Context::discover().expect("discover repository");
        let projection = load_validated_reader_evidence(&context, InputSnapshot::default())
            .expect("validated projection");
        assert_eq!(projection.route_id, "FS-RTE-06");
        assert_eq!(projection.route_status, "unbuilt");
        assert_eq!(projection.claim_id, "FS-CLM-37");
        assert_eq!(projection.claim_posture, "Unestablished");
        assert_eq!(projection.claim_disposition, "route-unbuilt");
        assert_eq!(projection.result, "not-run");
        assert!(!projection.valid_holdout_pass);
        assert_eq!(projection.enum_inventory.len(), 10);
        assert!(projection.enum_inventory.contains(&ReaderEnumEntry {
            field: "threshold_status",
            value: "pending-pilot".to_owned(),
        }));
        assert!(projection.enum_inventory.contains(&ReaderEnumEntry {
            field: "posture",
            value: "Unestablished".to_owned(),
        }));
        validate_reader_evidence_alignment(
            &projection,
            ReaderRouteAlignment {
                id: "FS-RTE-06",
                status: "unbuilt",
                route_status: "unbuilt",
            },
            ReaderClaimAlignment {
                id: "FS-CLM-37",
                route_ref: "FS-RTE-06",
                posture: "Unestablished",
                unestablished_disposition: Some("route-unbuilt"),
            },
        )
        .expect("live reader alignment");
    }

    #[test]
    fn populated_rule_executes_all_derived_evaluator_controls() {
        let rule = evaluator_rule_fixture();
        let known = rule
            .misconceptions
            .iter()
            .map(|item| item.misconception_id.clone())
            .collect();
        let controls = derived_evaluator_controls(&rule, &known).expect("derived controls");
        assert_eq!(controls, 43);
    }
}
