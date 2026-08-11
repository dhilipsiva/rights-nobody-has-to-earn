#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate and render Book 1's reader-evidence contract.

The reviewed JSON is the sole machine-readable source for the pilot receipt,
threshold rule, author ratification, and fresh-holdout pre-registration.  The
initial source is deliberately dormant: it contains the already-ratified rule
form, but no pilot, severity taxonomy, threshold value, ratification, holdout,
or reader result.

``--check`` validates the source, watches structural mutations fail, and checks
that the generated report is current.  ``--check --execute`` additionally
executes state-transition and eventual numeric-boundary controls.  Neither mode
runs a reader study or verifies the truth of an externally held freshness,
identity, consent, or custody attestation.
"""

from __future__ import annotations

import argparse
import copy
import datetime as dt
from decimal import Decimal, InvalidOperation
import hashlib
import json
import os
import pathlib
import re
import sys
import subprocess
from typing import Iterable, Mapping


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("new-book-plans/reader-evidence.json")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/reader-evidence.md")
PROTOCOL_DECISION = pathlib.Path(
    "new-book-plans/book-1-reader-evidence-protocol-decision.md"
)

ROOT_KEYS = {
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
}
HISTORY_TRANSITION_KEYS = {
    "previous_source_commit",
    "previous_source_sha256",
    "previous_history_head_sha256",
    "history_head_sha256",
}
ROUTE_KEYS = {
    "route_id",
    "route_status",
    "evidence_contract_status",
    "structural_checker_binding",
    "reviewer_custody_attestation",
    "evidence_admission_gate_binding",
    "negative_control_status",
}
CLAIM_KEYS = {"claim_id", "posture", "disposition", "result_ref"}
PRIVACY_KEYS = {
    "public_record_policy",
    "allowed_public_record_kinds",
    "excluded_from_repository",
    "freshness_attestation_boundary",
}
PROTOCOL_KEYS = {
    "decision_sha256",
    "method",
    "evaluation_order",
    "aggregate_offset_prohibited",
    "required_targets",
    "disclosed_limits",
    "ethics_terms",
    "freshness_terms",
    "non_substitution",
}
TARGET_KEYS = {"target_id", "description"}
PILOT_KEYS = {
    "pilot_status",
    "control_status",
    "active_attempt_id",
    "attempts",
}
PILOT_ATTEMPT_KEYS = {
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
}
PREREQUISITE_KEYS = {
    "readers_map_ref",
    "glossary_ref",
    "accessible_navigation_ref",
}
THRESHOLD_RULE_KEYS = {
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
}
POLICY_KEYS = {
    "missing",
    "ambiguous",
    "multiply_coded",
    "withdrawn",
    "excluded",
    "unclassified",
    "rounding",
    "coder_adjudication",
}
SEVERITY_KEYS = {
    "severity_id",
    "label",
    "definition",
    "classification_boundary",
}
MISCONCEPTION_KEYS = {
    "misconception_id",
    "definition",
    "severity_id",
    "core",
}
THRESHOLD_SPEC_KEYS = {
    "threshold_id",
    "metric",
    "operator",
    "value_kind",
    "value",
    "unit",
    "denominator",
    "scope_refs",
    "evaluator_ref",
}
TARGET_THRESHOLD_KEYS = {"target_id", "threshold"}
SEVERITY_THRESHOLD_KEYS = {"severity_id", "threshold"}
ARTIFACT_KEYS = {"artifact_id", "ref", "sha256"}
FREEZE_BINDING_KEYS = {
    "binding_id",
    "binding_type",
    "bound_payload_sha256",
    "attested_payload_sha256",
    "ref",
    "attestation_sha256",
    "frozen_at",
}
REVIEWER_ATTESTATION_KEYS = {
    "attestation_id",
    "scope",
    "evidence_gate_sha256",
    "ref",
    "sha256",
    "attested_date",
}
PILOT_PRE_REGISTRATION_KEYS = {
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
}
SESSION_KEYS = {
    "study_id",
    "record_commitment_sha256",
    "admissibility",
    "target_outcomes",
    "misconception_outcomes",
    "deviation_ids",
    "custody_attestation_ids",
}
TARGET_OUTCOME_KEYS = {"target_id", "status", "adjudication"}
MISCONCEPTION_OUTCOME_KEYS = {
    "misconception_id",
    "status",
    "occurrences",
    "opportunities",
    "adjudication",
}
PILOT_RECEIPT_KEYS = {
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
}
DECISION_PACKET_KEYS = {
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
}
RATIFICATION_KEYS = {
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
}
HOLDOUT_KEYS = {
    "active_attempt_id",
    "attempts",
}
HOLDOUT_ATTEMPT_KEYS = {
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
}
HOLDOUT_PRE_REGISTRATION_KEYS = {
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
}
RELEASE_CANDIDATE_KEYS = {"candidate_id", "manifest_sha256", "artifacts"}
COMMITMENT_KEYS = {
    "commitment_id",
    "nonce_commitment_sha256",
    "committed_preimage_sha256",
    "custody_attestation_sha256",
}
COMMITMENT_REVEAL_KEYS = {
    "commitment_id",
    "revealed_at",
    "nonce_hex",
    "preimage",
    "custody_attestation_id",
    "reveal_sha256",
}
DEVIATION_KEYS = {
    "deviation_id",
    "code",
    "impact",
    "custody_attestation_id",
}
CUSTODY_KEYS = {
    "attestation_id",
    "study_id",
    "scope",
    "record_commitment_sha256",
    "ref",
    "sha256",
    "freshness_attested",
    "record_sha256",
}
RESULT_RECEIPT_KEYS = {
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
}
ACCEPTANCE_KEYS = {"gate_c_satisfied", "permitted_claim", "limits"}
GATE_ADMISSION_RECEIPT_KEYS = {
    "schema_version",
    "input_sha256",
    "evidence_gate_sha256",
    "decision",
    "receipt_sha256",
}

THRESHOLD_STATUSES = {"pending-pilot", "candidate", "author-ratified"}
HOLDOUT_STATUSES = {"not-frozen", "frozen", "completed", "void"}
RESULTS = {"not-run", "not-evaluable", "fail", "pass"}
ROUTE_STATUSES = {"unbuilt", "available"}
PILOT_STATUSES = {"not-run", "completed", "void"}
HOLDOUT_ATTEMPT_STATUSES = {"frozen", "completed", "void"}
CUSTODY_SCOPES = {"session-record", "study-freshness", "deviation", "commitment"}
REVIEWER_SCOPE = "reader-evidence-gate-review"
CONTROL_STATUSES = {
    "not-run",
    "watched-failing",
    "failed-to-fail",
    "indeterminate",
}
ADMISSIBILITY = {"admissible", "inadmissible", "withdrawn"}
TARGET_STATUSES = {
    "identified",
    "not-identified",
    "missing",
    "ambiguous",
    "multiply-coded",
    "unclassified",
}
MISCONCEPTION_STATUSES = {
    "present",
    "absent",
    "missing",
    "ambiguous",
    "multiply-coded",
    "unclassified",
}
ADJUDICATION_STATES = {"not-required", "resolved", "unresolved"}
FINAL_TARGET_STATUSES = {"identified", "not-identified"}
FINAL_MISCONCEPTION_STATUSES = {"present", "absent"}
OUTCOME_POLICY_ACTIONS = {
    "count-adverse",
    "exclude-observation",
    "study-not-evaluable",
    "require-adjudication",
}
CODER_ADJUDICATION_ACTIONS = {
    "unresolved-count-adverse",
    "unresolved-exclude-observation",
    "unresolved-not-evaluable",
}
CORE_REPETITION_UNITS = {"admissible-session", "coded-opportunity"}
THRESHOLD_METRICS = {
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
}
EVALUATION_ORDER = [
    "protocol-validity",
    "evaluability",
    "core-veto",
    "required-targets",
    "non-core-rules",
    "pass",
]
REQUIRED_TARGETS = {
    "RE-TGT-ORDINARY-LIFE": "ordinary constructive life",
    "RE-TGT-DEMOCRATIC-CHOICE": "democratic choice",
    "RE-TGT-PRIVATE-FREEDOM": "private freedom",
    "RE-TGT-SUCCESSFUL-PROVISION": "successful provision",
    "RE-TGT-REPAIR": "repair",
    "RE-TGT-PRISONER-STRESS-TEST": (
        "the prisoner as a stress test rather than the central inhabitant"
    ),
}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
PUBLIC_RECORD_POLICY = "privacy-minimal-coded-records-only"
ALLOWED_PUBLIC_RECORD_KINDS = [
    "opaque study identifiers",
    "coded target and misconception outcomes",
    "artifact and commitment digests",
    "coded deviations",
    "custody attestations without identity material",
]
EXCLUDED_FROM_REPOSITORY = [
    (
        "participant, session, coder, reviewer, and custodian names, "
        "pseudonyms, and identity mappings"
    ),
    "raw responses and free text",
    "consent and withdrawal forms",
    "direct contact, demographic, and accessibility records",
]
FRESHNESS_ATTESTATION_BOUNDARY = (
    "The checker validates the evidence contract and the attestation binding; "
    "it cannot establish the truth of an externally held freshness or identity "
    "attestation."
)
DISCLOSED_LIMITS = [
    "The ordinary-life account rests on unimplemented families at the time of testing.",
    "Every tested snapshot must carry its exact version identity.",
    "The evidence is usability evidence about the tested audience, not population statistics.",
    "Sampling and method limits bound every permitted reader claim.",
    (
        "No reader result enters the reasoning engine or establishes a domain "
        "assigned to another assurance route."
    ),
]
ETHICS_TERMS = [
    "informed consent",
    "withdrawal",
    "data minimisation and protection",
    "accessible participation",
    "fair compensation",
    "non-retaliation",
    "trauma safeguards where coercive experience is discussed",
    "independent ethics and safety review where appropriate",
]
FRESHNESS_TERMS = [
    (
        "Holdout participants have no prior exposure to drafts, previews, "
        "the pilot, or the reviews corpus."
    ),
    "Pilot participants are excluded from the holdout.",
    "The in-repository reviewer corpus is never admissible reader-study evidence.",
]
NON_SUBSTITUTION = (
    "Reader evidence warrants only comprehension, balance, and human effects "
    "for the tested audience within the disclosed sampling and method limits."
)


DATE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")
UTC_TIMESTAMP = re.compile(
    r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"
)
OPAQUE_ID = re.compile(r"^RE-[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+$")
COMMIT = re.compile(r"^(?:[0-9a-f]{40}|[0-9a-f]{64})$")
DECIMAL = re.compile(r"^(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$")
INTEGER = re.compile(r"^(?:0|[1-9][0-9]*)$")
PLACEHOLDER = re.compile(r"^(?:tbd|todo|unknown|n/?a|pending)$", re.I)
NONCE_PREIMAGE = re.compile(r"^(?:[0-9a-f]{2}){32,}$")
STRUCTURAL_CHECKER_ARTIFACT_ID = "RE-ART-STRUCTURAL-CHECKER"
STRUCTURAL_CHECKER_REF = (
    "new-book-plans/14-reader-evidence.py::" + "def " + "main("
)
EVIDENCE_GATE_ARTIFACT_ID = "RE-ART-EVIDENCE-ADMISSION-GATE"
EVIDENCE_GATE_REF = "new-book-plans/reader-evidence-admission-gate.py::def evaluate_reader_evidence("

FORBIDDEN_SCORE_KEYS = {
    "score",
    "aggregate_score",
    "overall_score",
    "weighted_score",
    "average_score",
    "total_score",
}
FORBIDDEN_PRIVATE_KEYS = {
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
}


class ReaderEvidenceError(RuntimeError):
    """The reviewed reader-evidence source or generated report is invalid."""


def resolve(path: pathlib.Path) -> pathlib.Path:
    return path if path.is_absolute() else ROOT / path


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve(strict=False).relative_to(ROOT.resolve())
    except ValueError as exc:
        raise ReaderEvidenceError(f"path escapes repository: {path}") from exc


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append("missing " + ", ".join(missing))
        if extra:
            details.append("unexpected " + ", ".join(extra))
        raise ReaderEvidenceError(f"{path}: {'; '.join(details)}")


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise ReaderEvidenceError(f"{path}: expected an object with string keys")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise ReaderEvidenceError(f"{path}: expected an array")
    return value


def as_text(value: object, path: str) -> str:
    if not isinstance(value, str) or not value.strip() or PLACEHOLDER.fullmatch(value.strip()):
        raise ReaderEvidenceError(f"{path}: expected substantive text")
    return value


def text_list(value: object, path: str, *, nonempty: bool = True) -> list[str]:
    values = as_list(value, path)
    if nonempty and not values:
        raise ReaderEvidenceError(f"{path}: must not be empty")
    result = [as_text(item, f"{path}[{index}]") for index, item in enumerate(values)]
    if len(result) != len(set(result)):
        raise ReaderEvidenceError(f"{path}: duplicate values")
    return result


def enum(value: object, allowed: set[str], path: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ReaderEvidenceError(f"{path}: expected one of {sorted(allowed)}")
    return value


def boolean(value: object, path: str) -> bool:
    if not isinstance(value, bool):
        raise ReaderEvidenceError(f"{path}: expected a boolean")
    return value


def sha(value: object, path: str, expected: str | None = None) -> str:
    if not isinstance(value, str) or not SHA256.fullmatch(value):
        raise ReaderEvidenceError(f"{path}: expected 64 lowercase hexadecimal characters")
    if expected is not None and value != expected:
        raise ReaderEvidenceError(f"{path}: stale; declared {value}, actual {expected}")
    return value


def date(value: object, path: str) -> str:
    text = as_text(value, path)
    if not DATE.fullmatch(text):
        raise ReaderEvidenceError(f"{path}: expected YYYY-MM-DD")
    try:
        parsed = dt.date.fromisoformat(text)
    except ValueError as exc:
        raise ReaderEvidenceError(f"{path}: invalid calendar date") from exc
    if parsed.isoformat() != text:
        raise ReaderEvidenceError(f"{path}: date must be canonical YYYY-MM-DD")
    return text


def utc_timestamp(value: object, path: str) -> str:
    text = as_text(value, path)
    if not UTC_TIMESTAMP.fullmatch(text):
        raise ReaderEvidenceError(f"{path}: expected canonical UTC YYYY-MM-DDTHH:MM:SSZ")
    try:
        parsed = dt.datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError as exc:
        raise ReaderEvidenceError(f"{path}: invalid UTC timestamp") from exc
    if parsed.strftime("%Y-%m-%dT%H:%M:%SZ") != text:
        raise ReaderEvidenceError(f"{path}: timestamp must be canonical UTC")
    return text


def opaque_id(value: object, path: str) -> str:
    text = as_text(value, path)
    if not OPAQUE_ID.fullmatch(text):
        raise ReaderEvidenceError(f"{path}: expected an opaque RE-* identifier")
    return text


def integer_text(value: object, path: str, *, positive: bool = False) -> str:
    text = as_text(value, path)
    if not INTEGER.fullmatch(text):
        raise ReaderEvidenceError(f"{path}: expected a canonical non-negative integer string")
    if positive and text == "0":
        raise ReaderEvidenceError(f"{path}: expected a positive integer string")
    return text


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical_sha(value: object, *, omit: str | None = None) -> str:
    if omit is not None:
        obj = dict(as_object(value, "canonical digest object"))
        obj.pop(omit, None)
        value = obj
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return sha256_bytes(encoded)


def canonical_sha_omitting(value: object, omitted: set[str]) -> str:
    obj = dict(as_object(value, "canonical digest object"))
    for key in omitted:
        obj.pop(key, None)
    return canonical_sha(obj)


def history_head_sha256(
    pilot_attempt_sha256s: Iterable[str],
    holdout_attempt_sha256s: Iterable[str],
) -> str:
    """Derive the immutable head of both ordered attempt histories."""
    return canonical_sha(
        {
            "pilot_attempt_sha256s": list(pilot_attempt_sha256s),
            "holdout_attempt_sha256s": list(holdout_attempt_sha256s),
        }
    )


def validate_preregistration_history_binding(
    registration: dict[str, object],
    path: str,
    *,
    expected_predecessor_attempt_sha256: str | None,
    expected_prior_history_head_sha256: str | None,
    enforce_expected: bool,
) -> None:
    predecessor = registration["predecessor_attempt_sha256"]
    if predecessor is not None:
        sha(predecessor, f"{path}.predecessor_attempt_sha256")
    prior_head = sha(
        registration["prior_history_head_sha256"],
        f"{path}.prior_history_head_sha256",
    )
    if not enforce_expected:
        return
    if expected_predecessor_attempt_sha256 is None:
        if predecessor is not None:
            raise ReaderEvidenceError(
                f"{path}.predecessor_attempt_sha256: first attempt must be null"
            )
    else:
        sha(
            predecessor,
            f"{path}.predecessor_attempt_sha256",
            expected_predecessor_attempt_sha256,
        )
    sha(
        prior_head,
        f"{path}.prior_history_head_sha256",
        expected_prior_history_head_sha256,
    )


def validate_repo_reference(value: object, path: str) -> str:
    ref = as_text(value, path)
    if "::" not in ref:
        raise ReaderEvidenceError(f"{path}: repository reference needs path::exact-anchor")
    relative, anchor = ref.split("::", 1)
    if not relative or not anchor:
        raise ReaderEvidenceError(f"{path}: incomplete repository reference")
    target = resolve(pathlib.Path(relative))
    repo_relative(target)
    try:
        contents = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise ReaderEvidenceError(f"{path}: cannot read referenced file {relative}: {exc}") from exc
    count = contents.count(anchor)
    if count != 1:
        raise ReaderEvidenceError(
            f"{path}: anchor must occur exactly once in {relative}; found {count}"
        )
    return ref


def validate_external_or_repo_reference(value: object, path: str) -> str:
    ref = as_text(value, path)
    if ref.startswith("custody:"):
        suffix = ref.removeprefix("custody:")
        if not re.fullmatch(r"[A-Z0-9][A-Z0-9-]*", suffix):
            raise ReaderEvidenceError(f"{path}: malformed opaque custody reference")
        return ref
    return validate_repo_reference(ref, path)


def validate_artifact(value: object, path: str, *, verify_live: bool = True) -> dict[str, object]:
    artifact = as_object(value, path)
    exact_keys(artifact, ARTIFACT_KEYS, path)
    opaque_id(artifact["artifact_id"], f"{path}.artifact_id")
    ref = validate_external_or_repo_reference(artifact["ref"], f"{path}.ref")
    digest = sha(artifact["sha256"], f"{path}.sha256")
    if verify_live and not ref.startswith("custody:"):
        relative = ref.split("::", 1)[0]
        actual = sha256_bytes(resolve(pathlib.Path(relative)).read_bytes())
        if digest != actual:
            raise ReaderEvidenceError(
                f"{path}.sha256: stale; declared {digest}, actual {actual}"
            )
    return artifact

def contains_frozen_payload(
    value: object,
    expected_keys: set[str],
    expected_payload_sha256: str,
    digest_field: str,
) -> bool:
    if isinstance(value, dict):
        if set(value) == expected_keys:
            actual = canonical_sha_omitting(
                value, {"freeze_binding", digest_field}
            )
            if actual == expected_payload_sha256:
                return True
        return any(
            contains_frozen_payload(
                child,
                expected_keys,
                expected_payload_sha256,
                digest_field,
            )
            for child in value.values()
        )
    if isinstance(value, list):
        return any(
            contains_frozen_payload(
                child,
                expected_keys,
                expected_payload_sha256,
                digest_field,
            )
            for child in value
        )
    return False


def validate_git_freeze(
    commit: str,
    path: str,
    frozen_value: dict[str, object],
    expected_payload_sha256: str,
    digest_field: str,
) -> None:
    try:
        head = subprocess.run(
            ["git", "-C", str(ROOT), "rev-parse", "HEAD"],
            check=False,
            capture_output=True,
            timeout=10,
        )
        ancestor = subprocess.run(
            ["git", "-C", str(ROOT), "merge-base", "--is-ancestor", commit, "HEAD"],
            check=False,
            capture_output=True,
            timeout=10,
        )
        shown = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "show",
                f"{commit}:{DEFAULT_SOURCE.as_posix()}",
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(f"{path}: cannot inspect git freeze: {exc}") from exc
    if head.returncode != 0:
        raise ReaderEvidenceError(f"{path}: cannot resolve current git HEAD")
    current_head = head.stdout.decode("utf-8", errors="replace").strip()
    if commit == current_head:
        raise ReaderEvidenceError(
            f"{path}: git freeze must cite a commit strictly before the current checkout"
        )
    if ancestor.returncode != 0:
        raise ReaderEvidenceError(
            f"{path}: git freeze must cite a prior ancestor commit"
        )
    if shown.returncode != 0:
        detail = shown.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            f"{path}: git freeze source is unavailable: {detail}"
        )
    try:
        committed_source = json.loads(
            shown.stdout.decode("utf-8"),
            object_pairs_hook=reject_duplicate_keys,
        )
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReaderEvidenceError(
            f"{path}: git freeze source is invalid JSON"
        ) from exc
    if not contains_frozen_payload(
        committed_source,
        set(frozen_value),
        expected_payload_sha256,
        digest_field,
    ):
        raise ReaderEvidenceError(
            f"{path}: prior git commit does not contain the exact frozen payload"
        )


def validate_freeze_binding(
    value: object,
    path: str,
    *,
    frozen_value: dict[str, object],
    digest_field: str,
) -> dict[str, object]:
    binding = as_object(value, path)
    exact_keys(binding, FREEZE_BINDING_KEYS, path)
    opaque_id(binding["binding_id"], f"{path}.binding_id")
    binding_type = enum(
        binding["binding_type"],
        {"git-commit", "external-custody"},
        f"{path}.binding_type",
    )
    expected_payload = canonical_sha_omitting(
        frozen_value, {"freeze_binding", digest_field}
    )
    sha(
        binding["bound_payload_sha256"],
        f"{path}.bound_payload_sha256",
        expected_payload,
    )
    sha(
        binding["attested_payload_sha256"],
        f"{path}.attested_payload_sha256",
        expected_payload,
    )
    ref = as_text(binding["ref"], f"{path}.ref")
    if binding_type == "git-commit":
        if not ref.startswith("git:") or not COMMIT.fullmatch(ref.removeprefix("git:")):
            raise ReaderEvidenceError(
                f"{path}.ref: git binding requires git:<full-commit>"
            )
        validate_git_freeze(
            ref.removeprefix("git:"),
            path,
            frozen_value,
            expected_payload,
            digest_field,
        )
    else:
        if ref != "custody:READER-EVIDENCE-FREEZE":
            raise ReaderEvidenceError(
                f"{path}.ref: external freeze must use the fixed custody channel"
            )
        validate_external_or_repo_reference(ref, f"{path}.ref")
    frozen_at = utc_timestamp(binding["frozen_at"], f"{path}.frozen_at")
    envelope = {
        "binding_id": binding["binding_id"],
        "binding_type": binding_type,
        "attested_payload_sha256": expected_payload,
        "bound_payload_sha256": expected_payload,
        "ref": ref,
        "frozen_at": frozen_at,
    }
    if binding_type == "git-commit":
        sha(
            binding["attestation_sha256"],
            f"{path}.attestation_sha256",
            canonical_sha(envelope),
        )
    else:
        # The repository binds the external receipt digest but cannot attest
        # that the independent custodian actually published or retained it.
        sha(binding["attestation_sha256"], f"{path}.attestation_sha256")
    return binding




def walk_keys(value: object, path: str = "root") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            normal = key.lower().replace("-", "_")
            if normal in FORBIDDEN_SCORE_KEYS:
                raise ReaderEvidenceError(
                    f"{path}.{key}: aggregate or person scoring fields are prohibited"
                )
            if normal in FORBIDDEN_PRIVATE_KEYS:
                raise ReaderEvidenceError(
                    f"{path}.{key}: identifying or raw participant material belongs outside the repository"
                )
            walk_keys(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk_keys(child, f"{path}[{index}]")


def validate_protocol(
    source: dict[str, object],
    *,
    decision_bytes: bytes | None = None,
) -> None:
    if source["spdx"] != "CC-BY-4.0":
        raise ReaderEvidenceError("spdx must be CC-BY-4.0")
    if source["schema_version"] != 1 or isinstance(
        source["schema_version"], bool
    ):
        raise ReaderEvidenceError("schema_version must be integer 1")
    if source["contract_id"] != "book-1-reader-evidence-v1":
        raise ReaderEvidenceError("contract_id must be book-1-reader-evidence-v1")
    if decision_bytes is None:
        decision_ref = validate_repo_reference(
            source["protocol_decision_ref"], "protocol_decision_ref"
        )
        decision_digest = sha256_bytes(resolve(PROTOCOL_DECISION).read_bytes())
    else:
        decision_ref = as_text(
            source["protocol_decision_ref"], "protocol_decision_ref"
        )
        if "::" not in decision_ref:
            raise ReaderEvidenceError(
                "protocol_decision_ref needs path::exact-anchor"
            )
        relative, anchor = decision_ref.split("::", 1)
        if relative != PROTOCOL_DECISION.as_posix() or not anchor:
            raise ReaderEvidenceError(
                "protocol_decision_ref must cite the controlling decision"
            )
        try:
            decision_text = decision_bytes.decode("utf-8")
        except UnicodeError as exc:
            raise ReaderEvidenceError(
                "candidate protocol decision is not valid UTF-8"
            ) from exc
        count = decision_text.count(anchor)
        if count != 1:
            raise ReaderEvidenceError(
                "protocol_decision_ref anchor must occur exactly once "
                f"in the candidate decision; found {count}"
            )
        decision_digest = sha256_bytes(decision_bytes)
    if not decision_ref.startswith(PROTOCOL_DECISION.as_posix() + "::"):
        raise ReaderEvidenceError(
            "protocol_decision_ref must cite the controlling decision"
        )

    protocol = as_object(source["protocol"], "protocol")
    exact_keys(protocol, PROTOCOL_KEYS, "protocol")
    sha(protocol["decision_sha256"], "protocol.decision_sha256", decision_digest)
    if protocol["method"] != "pre-registered-pilot-and-fresh-holdout":
        raise ReaderEvidenceError("protocol.method drifted from the ratified method")
    if protocol["evaluation_order"] != EVALUATION_ORDER:
        raise ReaderEvidenceError("protocol.evaluation_order must preserve the ratified order")
    if protocol["aggregate_offset_prohibited"] is not True:
        raise ReaderEvidenceError("protocol must prohibit aggregate offset of a core finding")
    targets = as_list(protocol["required_targets"], "protocol.required_targets")
    found: dict[str, str] = {}
    for index, raw in enumerate(targets):
        path = f"protocol.required_targets[{index}]"
        target = as_object(raw, path)
        exact_keys(target, TARGET_KEYS, path)
        target_id = as_text(target["target_id"], f"{path}.target_id")
        description = as_text(target["description"], f"{path}.description")
        if target_id in found:
            raise ReaderEvidenceError(f"{path}.target_id: duplicate {target_id}")
        found[target_id] = description
    if found != REQUIRED_TARGETS:
        raise ReaderEvidenceError("protocol.required_targets drifted from the ratified minimum")
    pinned_lists = {
        "disclosed_limits": DISCLOSED_LIMITS,
        "ethics_terms": ETHICS_TERMS,
        "freshness_terms": FRESHNESS_TERMS,
    }
    for key, expected in pinned_lists.items():
        actual = text_list(protocol[key], f"protocol.{key}")
        if actual != expected:
            raise ReaderEvidenceError(f"protocol.{key} drifted from the ratified terms")
    if protocol["non_substitution"] != NON_SUBSTITUTION:
        raise ReaderEvidenceError("protocol.non_substitution drifted from the ratified boundary")


def validate_privacy(source: dict[str, object]) -> None:
    privacy = as_object(source["privacy"], "privacy")
    exact_keys(privacy, PRIVACY_KEYS, "privacy")
    if privacy["public_record_policy"] != PUBLIC_RECORD_POLICY:
        raise ReaderEvidenceError("privacy.public_record_policy drifted")
    allowed = text_list(
        privacy["allowed_public_record_kinds"],
        "privacy.allowed_public_record_kinds",
    )
    if allowed != ALLOWED_PUBLIC_RECORD_KINDS:
        raise ReaderEvidenceError("privacy.allowed_public_record_kinds drifted")
    excluded = text_list(
        privacy["excluded_from_repository"],
        "privacy.excluded_from_repository",
    )
    if excluded != EXCLUDED_FROM_REPOSITORY:
        raise ReaderEvidenceError("privacy.excluded_from_repository drifted")
    if privacy["freshness_attestation_boundary"] != FRESHNESS_ATTESTATION_BOUNDARY:
        raise ReaderEvidenceError("privacy.freshness_attestation_boundary drifted")


def validate_session_records(
    value: object,
    path: str,
    *,
    expected_study_id: str | None = None,
    known_misconceptions: set[str] | None = None,
) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    record_commitments: set[str] = set()
    for index, raw in enumerate(as_list(value, path)):
        item_path = f"{path}[{index}]"
        record = as_object(raw, item_path)
        exact_keys(record, SESSION_KEYS, item_path)
        study_id = opaque_id(record["study_id"], f"{item_path}.study_id")
        commitment = sha(
            record["record_commitment_sha256"],
            f"{item_path}.record_commitment_sha256",
        )
        if commitment in record_commitments:
            raise ReaderEvidenceError(
                f"{item_path}.record_commitment_sha256: duplicate coded session"
            )
        record_commitments.add(commitment)
        if expected_study_id is not None and study_id != expected_study_id:
            raise ReaderEvidenceError(f"{item_path}.study_id: does not match pre-registration")
        admissibility = enum(record["admissibility"], ADMISSIBILITY, f"{item_path}.admissibility")

        target_outcomes = as_list(record["target_outcomes"], f"{item_path}.target_outcomes")
        seen_targets: set[str] = set()
        for outcome_index, raw_outcome in enumerate(target_outcomes):
            outcome_path = f"{item_path}.target_outcomes[{outcome_index}]"
            outcome = as_object(raw_outcome, outcome_path)
            exact_keys(outcome, TARGET_OUTCOME_KEYS, outcome_path)
            target_id = as_text(outcome["target_id"], f"{outcome_path}.target_id")
            if target_id not in REQUIRED_TARGETS:
                raise ReaderEvidenceError(f"{outcome_path}.target_id: unknown target")
            if target_id in seen_targets:
                raise ReaderEvidenceError(f"{outcome_path}.target_id: duplicate target")
            seen_targets.add(target_id)
            status = enum(outcome["status"], TARGET_STATUSES, f"{outcome_path}.status")
            adjudication = enum(
                outcome["adjudication"],
                ADJUDICATION_STATES,
                f"{outcome_path}.adjudication",
            )
            if adjudication == "resolved" and status not in FINAL_TARGET_STATUSES:
                raise ReaderEvidenceError(
                    f"{outcome_path}: a resolved target outcome must carry a final status"
                )
            if adjudication == "unresolved" and status in FINAL_TARGET_STATUSES:
                raise ReaderEvidenceError(
                    f"{outcome_path}: a final target outcome cannot remain unresolved"
                )

        misconception_outcomes = as_list(
            record["misconception_outcomes"],
            f"{item_path}.misconception_outcomes",
        )
        seen_misconceptions: set[str] = set()
        for outcome_index, raw_outcome in enumerate(misconception_outcomes):
            outcome_path = f"{item_path}.misconception_outcomes[{outcome_index}]"
            outcome = as_object(raw_outcome, outcome_path)
            exact_keys(outcome, MISCONCEPTION_OUTCOME_KEYS, outcome_path)
            misconception_id = opaque_id(
                outcome["misconception_id"],
                f"{outcome_path}.misconception_id",
            )
            if misconception_id in seen_misconceptions:
                raise ReaderEvidenceError(
                    f"{outcome_path}.misconception_id: duplicate misconception"
                )
            if known_misconceptions is not None and misconception_id not in known_misconceptions:
                raise ReaderEvidenceError(
                    f"{outcome_path}.misconception_id: unknown misconception"
                )
            seen_misconceptions.add(misconception_id)
            status = enum(
                outcome["status"],
                MISCONCEPTION_STATUSES,
                f"{outcome_path}.status",
            )
            adjudication = enum(
                outcome["adjudication"],
                ADJUDICATION_STATES,
                f"{outcome_path}.adjudication",
            )
            occurrences = int(
                integer_text(outcome["occurrences"], f"{outcome_path}.occurrences")
            )
            opportunities = int(
                integer_text(
                    outcome["opportunities"],
                    f"{outcome_path}.opportunities",
                    positive=True,
                )
            )
            if occurrences > opportunities:
                raise ReaderEvidenceError(
                    f"{outcome_path}: occurrences cannot exceed opportunities"
                )
            if status == "absent" and occurrences != 0:
                raise ReaderEvidenceError(
                    f"{outcome_path}: absent requires zero occurrences"
                )
            if status == "present" and occurrences == 0:
                raise ReaderEvidenceError(
                    f"{outcome_path}: present requires at least one occurrence"
                )
            if adjudication == "resolved" and status not in FINAL_MISCONCEPTION_STATUSES:
                raise ReaderEvidenceError(
                    f"{outcome_path}: a resolved misconception outcome must carry a final status"
                )
            if adjudication == "unresolved" and status in FINAL_MISCONCEPTION_STATUSES:
                raise ReaderEvidenceError(
                    f"{outcome_path}: a final misconception outcome cannot remain unresolved"
                )

        for key in ("deviation_ids", "custody_attestation_ids"):
            ids = text_list(record[key], f"{item_path}.{key}", nonempty=False)
            for id_index, item_id in enumerate(ids):
                opaque_id(item_id, f"{item_path}.{key}[{id_index}]")

        if admissibility != "admissible":
            if target_outcomes or misconception_outcomes:
                raise ReaderEvidenceError(
                    f"{item_path}: inadmissible or withdrawn sessions may not publish coded outcomes"
                )
        else:
            if seen_targets != set(REQUIRED_TARGETS):
                raise ReaderEvidenceError(
                    f"{item_path}: every admissible session must explicitly code every required target"
                )
            if (
                known_misconceptions is not None
                and seen_misconceptions != known_misconceptions
            ):
                raise ReaderEvidenceError(
                    f"{item_path}: every holdout session must explicitly code every ratified misconception"
                )
        records.append(record)
    return records


def validate_pilot_pre_registration(
    value: object,
    path: str,
    fixed_protocol_sha256: str,
    *,
    expected_predecessor_attempt_sha256: str | None,
    expected_prior_history_head_sha256: str,
) -> dict[str, object]:
    registration = as_object(value, path)
    exact_keys(registration, PILOT_PRE_REGISTRATION_KEYS, path)
    opaque_id(registration["study_id"], f"{path}.study_id")
    registered_date = date(registration["registered_date"], f"{path}.registered_date")
    validate_preregistration_history_binding(
        registration,
        path,
        expected_predecessor_attempt_sha256=expected_predecessor_attempt_sha256,
        expected_prior_history_head_sha256=expected_prior_history_head_sha256,
        enforce_expected=True,
    )

    sha(
        registration["fixed_protocol_sha256"],
        f"{path}.fixed_protocol_sha256",
        fixed_protocol_sha256,
    )
    for key in (
        "protocol",
        "instrument",
        "rubric",
        "sample_rule",
        "disclosure_set",
        "ethics_terms",
        "provisional_rule",
    ):
        validate_artifact(registration[key], f"{path}.{key}")
    binding = validate_freeze_binding(
        registration["freeze_binding"],
        f"{path}.freeze_binding",
        frozen_value=registration,
        digest_field="pre_registration_sha256",
    )
    if str(binding["frozen_at"])[:10] < registered_date:
        raise ReaderEvidenceError(
            f"{path}.freeze_binding: external freeze cannot precede registration"
        )
    sha(
        registration["pre_registration_sha256"],
        f"{path}.pre_registration_sha256",
        canonical_sha(registration, omit="pre_registration_sha256"),
    )
    return registration

def validate_decision_packet(value: object, path: str) -> dict[str, object]:
    packet = as_object(value, path)
    exact_keys(packet, DECISION_PACKET_KEYS, path)
    opaque_id(packet["packet_id"], f"{path}.packet_id")
    frozen_date = date(packet["frozen_date"], f"{path}.frozen_date")
    sha(packet["pilot_pre_registration_sha256"], f"{path}.pilot_pre_registration_sha256")
    sha(packet["tested_snapshot_sha256"], f"{path}.tested_snapshot_sha256")
    for key in (
        "coded_evidence",
        "exclusions",
        "coder_disagreements",
        "deviations",
        "revised_instrument",
        "control_transcript",
    ):
        validate_artifact(packet[key], f"{path}.{key}")
    binding = validate_freeze_binding(
        packet["freeze_binding"],
        f"{path}.freeze_binding",
        frozen_value=packet,
        digest_field="packet_sha256",
    )
    if str(binding["frozen_at"])[:10] != frozen_date:
        raise ReaderEvidenceError(
            f"{path}.frozen_date must equal the freeze binding calendar date"
        )
    sha(
        packet["packet_sha256"],
        f"{path}.packet_sha256",
        canonical_sha(packet, omit="packet_sha256"),
    )
    return packet

def validate_pilot_receipt(
    value: object,
    path: str,
    registration: dict[str, object],
    snapshot: dict[str, object],
    sessions: list[dict[str, object]],
    raw_deviations: list[object],
    raw_custody: list[object],
    packet: dict[str, object],
) -> dict[str, object]:
    receipt = as_object(value, path)
    exact_keys(receipt, PILOT_RECEIPT_KEYS, path)
    opaque_id(receipt["receipt_id"], f"{path}.receipt_id")
    utc_timestamp(receipt["completed_at"], f"{path}.completed_at")
    study_id = opaque_id(receipt["study_id"], f"{path}.study_id")
    if study_id != registration["study_id"]:
        raise ReaderEvidenceError(f"{path}.study_id: does not match pre-registration")
    if receipt["protocol_validity"] not in {"valid", "invalid"}:
        raise ReaderEvidenceError(f"{path}.protocol_validity: expected valid or invalid")
    digest_links = {
        "pre_registration_sha256": registration["pre_registration_sha256"],
        "snapshot_sha256": snapshot["sha256"],
        "instrument_sha256": as_object(registration["instrument"], "instrument")["sha256"],
        "rubric_sha256": as_object(registration["rubric"], "rubric")["sha256"],
        "coded_evidence_sha256": as_object(packet["coded_evidence"], "coded evidence")["sha256"],
        "coded_records_sha256": canonical_sha(sessions),
        "deviations_sha256": canonical_sha(raw_deviations),
        "control_transcript_sha256": as_object(packet["control_transcript"], "control transcript")["sha256"],
        "decision_packet_sha256": packet["packet_sha256"],
        "custody_records_sha256": canonical_sha(raw_custody),
    }
    for key, expected in digest_links.items():
        sha(receipt[key], f"{path}.{key}", str(expected))
    sha(
        receipt["session_classification_sha256"],
        f"{path}.session_classification_sha256",
        canonical_sha(
            [
                {
                    "record_commitment_sha256": record["record_commitment_sha256"],
                    "admissibility": record["admissibility"],
                }
                for record in sessions
            ]
        ),
    )
    sha(receipt["coder_sha256"], f"{path}.coder_sha256")
    custody_digests = text_list(
        receipt["custody_attestation_sha256s"],
        f"{path}.custody_attestation_sha256s",
    )
    expected_custody_digests = [
        str(as_object(item, "pilot custody record")["sha256"])
        for item in raw_custody
    ]
    if custody_digests != expected_custody_digests:
        raise ReaderEvidenceError(
            f"{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )
    for index, digest in enumerate(custody_digests):
        sha(digest, f"{path}.custody_attestation_sha256s[{index}]")
    sha(
        receipt["receipt_sha256"],
        f"{path}.receipt_sha256",
        canonical_sha(receipt, omit="receipt_sha256"),
    )
    return receipt


def validate_pilot_attempt(
    raw: object,
    path: str,
    source: dict[str, object],
    previous_sha256: str | None,
    *,
    prior_history_head_sha256: str,
    first: bool,
    active: bool,
) -> tuple[
    str,
    str,
    dict[str, object] | None,
    dict[str, object] | None,
    dict[str, object] | None,
    str,
    str,
]:
    attempt = as_object(raw, path)
    exact_keys(attempt, PILOT_ATTEMPT_KEYS, path)
    attempt_id = opaque_id(attempt["attempt_id"], f"{path}.attempt_id")
    declared_previous = attempt["previous_attempt_sha256"]
    if first:
        if declared_previous is not None:
            raise ReaderEvidenceError(
                f"{path}.previous_attempt_sha256: first attempt must be null"
            )
    else:
        sha(
            declared_previous,
            f"{path}.previous_attempt_sha256",
            previous_sha256,
        )
    status = enum(
        attempt["attempt_status"], PILOT_STATUSES, f"{path}.attempt_status"
    )
    control = enum(
        attempt["control_status"], CONTROL_STATUSES, f"{path}.control_status"
    )
    if not active and status == "not-run":
        raise ReaderEvidenceError(
            f"{path}: only the active final pilot attempt may remain not-run"
        )
    void_reason = attempt["void_reason_code"]
    if void_reason is not None:
        opaque_id(void_reason, f"{path}.void_reason_code")
    voided_at = attempt["voided_at"]
    if status == "void":
        voided_at = utc_timestamp(voided_at, f"{path}.voided_at")
    elif voided_at is not None:
        raise ReaderEvidenceError(
            f"{path}.voided_at: only a void pilot attempt may carry a terminal time"
        )

    prerequisites = as_object(attempt["prerequisites"], f"{path}.prerequisites")
    exact_keys(prerequisites, PREREQUISITE_KEYS, f"{path}.prerequisites")
    for key, ref in prerequisites.items():
        if ref is not None:
            validate_repo_reference(ref, f"{path}.prerequisites.{key}")

    registration = None
    if attempt["pre_registration"] is not None:
        registration = validate_pilot_pre_registration(
            attempt["pre_registration"],
            f"{path}.pre_registration",
            canonical_sha_omitting(source["protocol"], {"decision_sha256"}),
            expected_predecessor_attempt_sha256=previous_sha256,
            expected_prior_history_head_sha256=(
                prior_history_head_sha256
            ),
        )
    snapshot = None
    if attempt["tested_snapshot"] is not None:
        snapshot = validate_artifact(
            attempt["tested_snapshot"], f"{path}.tested_snapshot"
        )
    study_id = str(registration["study_id"]) if registration else None
    sessions = validate_session_records(
        attempt["session_records"],
        f"{path}.session_records",
        expected_study_id=study_id,
    )
    raw_deviations = as_list(attempt["deviations"], f"{path}.deviations")
    deviations = validate_deviations(raw_deviations, f"{path}.deviations")
    raw_custody = as_list(
        attempt["custody_attestations"], f"{path}.custody_attestations"
    )
    custody = validate_custody(raw_custody, f"{path}.custody_attestations")
    validate_record_links(
        sessions,
        deviations,
        custody,
        f"{path}.record_links",
        expected_study_id=study_id,
        commitment=None,
    )

    packet = (
        validate_decision_packet(
            attempt["decision_packet"], f"{path}.decision_packet"
        )
        if attempt["decision_packet"] is not None
        else None
    )
    if registration is not None and snapshot is not None and packet is not None:
        sha(
            packet["pilot_pre_registration_sha256"],
            f"{path}.decision_packet.pilot_pre_registration_sha256",
            str(registration["pre_registration_sha256"]),
        )
        sha(
            packet["tested_snapshot_sha256"],
            f"{path}.decision_packet.tested_snapshot_sha256",
            str(snapshot["sha256"]),
        )
    sensitivity = (
        validate_artifact(
            attempt["sensitivity_brief"], f"{path}.sensitivity_brief"
        )
        if attempt["sensitivity_brief"] is not None
        else None
    )
    receipt = None
    if attempt["receipt"] is not None:
        if registration is None or snapshot is None or packet is None:
            raise ReaderEvidenceError(
                f"{path}.receipt requires pre-registration, snapshot, and packet"
            )
        receipt = validate_pilot_receipt(
            attempt["receipt"],
            f"{path}.receipt",
            registration,
            snapshot,
            sessions,
            raw_deviations,
            raw_custody,
            packet,
        )

    payload_present = any(
        item is not None
        for item in (registration, snapshot, receipt, packet, sensitivity)
    ) or bool(sessions or raw_deviations or raw_custody)
    if status == "not-run":
        if void_reason is not None or control != "not-run":
            raise ReaderEvidenceError(
                f"{path}: pre-run pilot cannot carry a void reason or control result"
            )
        if registration is None or snapshot is None:
            raise ReaderEvidenceError(
                f"{path}: recorded pre-run pilot requires frozen preregistration and snapshot"
            )
        if any(ref is None for ref in prerequisites.values()):
            raise ReaderEvidenceError(
                f"{path}: recorded pre-run pilot requires all prerequisites"
            )
        if sessions or raw_deviations or raw_custody or receipt or packet or sensitivity:
            raise ReaderEvidenceError(
                f"{path}: pre-run pilot cannot carry as-run or decision evidence"
            )
    elif status == "completed":
        if void_reason is not None:
            raise ReaderEvidenceError(
                f"{path}: completed pilot cannot carry a void reason"
            )
        if None in (registration, snapshot, receipt, packet, sensitivity):
            raise ReaderEvidenceError(
                f"{path}: completed pilot requires every frozen artifact"
            )
        if any(ref is None for ref in prerequisites.values()):
            raise ReaderEvidenceError(
                f"{path}: completed pilot requires every prerequisite"
            )
        if not sessions or not any(
            record["admissibility"] == "admissible" for record in sessions
        ):
            raise ReaderEvidenceError(
                f"{path}: completed pilot requires admitted coded evidence"
            )
        if receipt["protocol_validity"] != "valid" or control != "watched-failing":
            raise ReaderEvidenceError(
                f"{path}: completed pilot requires valid protocol and watched-failing control"
            )
        if str(receipt["completed_at"])[:10] < str(registration["registered_date"]):
            raise ReaderEvidenceError(
                f"{path}: completion cannot precede preregistration"
            )
        freeze_at = str(as_object(
            registration["freeze_binding"], f"{path}.pre_registration.freeze_binding"
        )["frozen_at"])
        if str(receipt["completed_at"]) <= freeze_at:
            raise ReaderEvidenceError(
                f"{path}: pilot completion must follow the frozen preregistration"
            )
        packet_frozen_at = str(
            as_object(
                packet["freeze_binding"],
                f"{path}.decision_packet.freeze_binding",
            )["frozen_at"]
        )
        if packet_frozen_at <= str(receipt["completed_at"]):
            raise ReaderEvidenceError(
                f"{path}: decision packet freeze must strictly follow pilot completion"
            )
    else:
        if void_reason is None or not payload_present:
            raise ReaderEvidenceError(
                f"{path}: void pilot requires a coded reason and preserved evidence"
            )
        if (
            control == "watched-failing"
            and receipt is not None
            and receipt["protocol_validity"] == "valid"
        ):
            raise ReaderEvidenceError(
                f"{path}: fully valid pilot cannot be relabelled void"
            )
        if registration is not None:
            freeze_at = str(as_object(
                registration["freeze_binding"], f"{path}.pre_registration.freeze_binding"
            )["frozen_at"])
            if str(voided_at) <= freeze_at:
                raise ReaderEvidenceError(
                    f"{path}: pilot void time must follow the frozen preregistration"
                )
        if receipt is not None and str(receipt["completed_at"]) > str(voided_at):
            raise ReaderEvidenceError(
                f"{path}: pilot receipt completion cannot follow its void time"
            )

    attempt_sha = sha(
        attempt["attempt_sha256"],
        f"{path}.attempt_sha256",
        canonical_sha(attempt, omit="attempt_sha256"),
    )
    return (
        attempt_id,
        status,
        packet,
        sensitivity,
        receipt,
        control,
        attempt_sha,
    )

def validate_pilot(
    source: dict[str, object],
) -> tuple[
    bool,
    dict[str, object] | None,
    dict[str, object] | None,
    str | None,
]:
    pilot = as_object(source["pilot"], "pilot")
    exact_keys(pilot, PILOT_KEYS, "pilot")
    summary_status = enum(
        pilot["pilot_status"], PILOT_STATUSES, "pilot.pilot_status"
    )
    summary_control = enum(
        pilot["control_status"], CONTROL_STATUSES, "pilot.control_status"
    )
    active_attempt_id = pilot["active_attempt_id"]
    if active_attempt_id is not None:
        active_attempt_id = opaque_id(
            active_attempt_id, "pilot.active_attempt_id"
        )
    attempts = as_list(pilot["attempts"], "pilot.attempts")
    if not attempts:
        if active_attempt_id is not None:
            raise ReaderEvidenceError(
                "pilot.active_attempt_id requires an attempt"
            )
        if summary_status != "not-run" or summary_control != "not-run":
            raise ReaderEvidenceError(
                "empty pilot history must remain not-run/not-run"
            )
        return False, None, None, None

    seen_ids: set[str] = set()
    previous_sha: str | None = None
    pilot_attempt_sha256s: list[str] = []
    active_packet = None
    active_sensitivity = None
    active_receipt = None
    active_status = ""
    active_control = ""
    active_id = ""
    for index, raw in enumerate(attempts):
        (
            attempt_id,
            attempt_status,
            packet,
            sensitivity,
            receipt,
            control,
            attempt_sha,
        ) = validate_pilot_attempt(
            raw,
            f"pilot.attempts[{index}]",
            source,
            previous_sha,
            prior_history_head_sha256=history_head_sha256(
                pilot_attempt_sha256s, []
            ),
            first=index == 0,
            active=index == len(attempts) - 1,
        )
        if attempt_id in seen_ids:
            raise ReaderEvidenceError(
                f"pilot.attempts[{index}].attempt_id: duplicate"
            )
        seen_ids.add(attempt_id)
        previous_sha = attempt_sha
        pilot_attempt_sha256s.append(attempt_sha)
        active_id = attempt_id
        active_status = attempt_status
        active_control = control
        active_packet = packet
        active_sensitivity = sensitivity
        active_receipt = receipt

    if active_attempt_id != active_id:
        raise ReaderEvidenceError(
            "pilot.active_attempt_id must identify the final append-only attempt"
        )
    if summary_status != active_status or summary_control != active_control:
        raise ReaderEvidenceError(
            "pilot summary status/control must equal the active attempt"
        )
    valid = (
        active_status == "completed"
        and active_control == "watched-failing"
        and active_receipt is not None
        and active_receipt["protocol_validity"] == "valid"
    )
    return valid, active_packet, active_sensitivity, active_id


def empty_threshold_content(rule: dict[str, object]) -> bool:
    return (
        rule["rule_id"] is None
        and rule["severity_taxonomy"] == []
        and rule["misconceptions"] == []
        and rule["core_misconception_ids"] == []
        and rule["core_failure_mode"] is None
        and rule["repetition_unit"] is None
        and rule["denominator"] is None
        and rule["core_failure_threshold"] is None
        and rule["required_target_thresholds"] == []
        and rule["non_core_thresholds"] == []
        and rule["minimum_evaluable_evidence"] is None
        and all(value is None for value in as_object(rule["policies"], "threshold_rule.policies").values())
        and rule["rule_sha256"] is None
    )


def validate_threshold_spec(
    value: object,
    path: str,
    *,
    allowed_metrics: set[str],
    scope_refs: set[str],
) -> dict[str, object]:
    if not allowed_metrics or not allowed_metrics <= THRESHOLD_METRICS:
        raise ReaderEvidenceError(f"{path}: internal metric registry is incomplete")
    spec = as_object(value, path)
    exact_keys(spec, THRESHOLD_SPEC_KEYS, path)
    opaque_id(spec["threshold_id"], f"{path}.threshold_id")
    metric = enum(spec["metric"], allowed_metrics, f"{path}.metric")
    operator = enum(spec["operator"], {"lt", "lte", "eq", "gte", "gt"}, f"{path}.operator")
    value_kind = enum(
        spec["value_kind"], {"integer", "decimal", "qualitative"}, f"{path}.value_kind"
    )
    value_text = as_text(spec["value"], f"{path}.value")
    unit = as_text(spec["unit"], f"{path}.unit")
    denominator = as_text(spec["denominator"], f"{path}.denominator")
    refs = text_list(spec["scope_refs"], f"{path}.scope_refs")
    if set(refs) != scope_refs:
        raise ReaderEvidenceError(f"{path}.scope_refs: must exactly match its rule scope")
    if spec["evaluator_ref"] is not None:
        raise ReaderEvidenceError(
            f"{path}.evaluator_ref: release thresholds use the deterministic built-in evaluator"
        )

    count_contracts = {
        "admissible-session-count": ("sessions", "none"),
        "target-identification-count": ("identified-sessions", "none"),
        "core-finding-count": ("findings", "none"),
        "severity-session-finding-count": ("sessions", "none"),
        "severity-occurrence-count": ("occurrences", "none"),
    }
    rate_contracts = {
        "target-identification-rate": {"coded-target-observations"},
        "core-finding-rate": {
            "eligible-admissible-sessions",
            "coded-opportunities",
        },
        "severity-session-finding-rate": {"eligible-admissible-sessions"},
        "severity-occurrence-rate": {"coded-opportunities"},
    }
    if metric == "core-finding-present":
        if (
            value_kind != "qualitative"
            or operator != "eq"
            or value_text != "present"
            or unit != "presence"
            or denominator != "none"
        ):
            raise ReaderEvidenceError(
                f"{path}: single-finding core veto must compare presence exactly"
            )
    elif metric in count_contracts:
        expected_unit, expected_denominator = count_contracts[metric]
        if (
            value_kind != "integer"
            or unit != expected_unit
            or denominator != expected_denominator
            or not INTEGER.fullmatch(value_text)
            or Decimal(value_text) <= 0
        ):
            raise ReaderEvidenceError(
                f"{path}: count threshold must admit reachable below, exact, and above cases"
            )
    elif metric in rate_contracts:
        if (
            value_kind != "decimal"
            or unit != "proportion"
            or denominator not in rate_contracts[metric]
            or not DECIMAL.fullmatch(value_text)
            or not Decimal(0) < Decimal(value_text) < Decimal(1)
        ):
            raise ReaderEvidenceError(
                f"{path}: rate threshold must admit reachable below, exact, and above cases"
            )
    else:
        raise ReaderEvidenceError(f"{path}.metric: unsupported deterministic metric")
    return spec


def validate_threshold_rule(
    source: dict[str, object], valid_pilot: bool
) -> tuple[dict[str, object], set[str]]:
    rule = as_object(source["threshold_rule"], "threshold_rule")
    exact_keys(rule, THRESHOLD_RULE_KEYS, "threshold_rule")
    policies = as_object(rule["policies"], "threshold_rule.policies")
    exact_keys(policies, POLICY_KEYS, "threshold_rule.policies")
    if rule["evaluation_order"] != EVALUATION_ORDER:
        raise ReaderEvidenceError("threshold_rule.evaluation_order must preserve the fixed order")
    if rule["aggregate_offset_prohibited"] is not True:
        raise ReaderEvidenceError("threshold_rule must preserve the no-aggregate core veto")
    status = enum(source["threshold_status"], THRESHOLD_STATUSES, "threshold_status")
    if not valid_pilot:
        if status != "pending-pilot" or not empty_threshold_content(rule):
            raise ReaderEvidenceError(
                "threshold taxonomy and values are prohibited until a valid completed pilot exists"
            )
        if source["ratification"] is not None:
            raise ReaderEvidenceError("ratification is prohibited before a valid completed pilot")
        return rule, set()
    if status == "pending-pilot":
        if not empty_threshold_content(rule) or source["ratification"] is not None:
            raise ReaderEvidenceError("pending-pilot must not carry a candidate rule or ratification")
        return rule, set()

    rule_id = opaque_id(rule["rule_id"], "threshold_rule.rule_id")
    severities: dict[str, dict[str, object]] = {}
    for index, raw in enumerate(
        as_list(rule["severity_taxonomy"], "threshold_rule.severity_taxonomy")
    ):
        path = f"threshold_rule.severity_taxonomy[{index}]"
        item = as_object(raw, path)
        exact_keys(item, SEVERITY_KEYS, path)
        severity_id = opaque_id(item["severity_id"], f"{path}.severity_id")
        if severity_id in severities:
            raise ReaderEvidenceError(f"{path}.severity_id: duplicate {severity_id}")
        for key in ("label", "definition", "classification_boundary"):
            as_text(item[key], f"{path}.{key}")
        severities[severity_id] = item
    if not severities:
        raise ReaderEvidenceError("candidate threshold rule requires a severity taxonomy")

    misconceptions: dict[str, dict[str, object]] = {}
    for index, raw in enumerate(
        as_list(rule["misconceptions"], "threshold_rule.misconceptions")
    ):
        path = f"threshold_rule.misconceptions[{index}]"
        item = as_object(raw, path)
        exact_keys(item, MISCONCEPTION_KEYS, path)
        misconception_id = opaque_id(item["misconception_id"], f"{path}.misconception_id")
        if misconception_id in misconceptions:
            raise ReaderEvidenceError(f"{path}.misconception_id: duplicate {misconception_id}")
        as_text(item["definition"], f"{path}.definition")
        severity_id = opaque_id(item["severity_id"], f"{path}.severity_id")
        if severity_id not in severities:
            raise ReaderEvidenceError(f"{path}.severity_id: unknown severity")
        boolean(item["core"], f"{path}.core")
        misconceptions[misconception_id] = item
    if not misconceptions:
        raise ReaderEvidenceError("candidate threshold rule requires stable misconception IDs")

    declared_core = set(
        text_list(rule["core_misconception_ids"], "threshold_rule.core_misconception_ids")
    )
    actual_core = {
        misconception_id
        for misconception_id, item in misconceptions.items()
        if item["core"] is True
    }
    if declared_core != actual_core or not actual_core:
        raise ReaderEvidenceError(
            "core_misconception_ids must exactly match non-empty core mappings"
        )

    core_mode = enum(
        rule["core_failure_mode"],
        {"single", "repeated"},
        "threshold_rule.core_failure_mode",
    )
    repetition_unit = enum(
        rule["repetition_unit"],
        CORE_REPETITION_UNITS,
        "threshold_rule.repetition_unit",
    )
    denominator = as_text(rule["denominator"], "threshold_rule.denominator")
    core_metrics = (
        {"core-finding-present"}
        if core_mode == "single"
        else {"core-finding-count", "core-finding-rate"}
    )
    core_spec = validate_threshold_spec(
        rule["core_failure_threshold"],
        "threshold_rule.core_failure_threshold",
        allowed_metrics=core_metrics,
        scope_refs=actual_core,
    )
    core_metric = str(core_spec["metric"])
    if core_metric != "core-finding-present":
        if (
            core_spec["operator"] not in {"gte", "gt"}
            or Decimal(str(core_spec["value"])) <= 0
            or (
                core_metric == "core-finding-rate"
                and core_spec["operator"] == "gt"
                and Decimal(str(core_spec["value"])) >= 1
            )
        ):
            raise ReaderEvidenceError(
                "repeated core veto must use a positive, reachable adverse boundary"
            )
    expected_denominator = "none"
    if core_metric == "core-finding-rate":
        expected_denominator = (
            "eligible-admissible-sessions"
            if repetition_unit == "admissible-session"
            else "coded-opportunities"
        )
    if denominator != expected_denominator or core_spec["denominator"] != denominator:
        raise ReaderEvidenceError(
            "threshold_rule.denominator must match the selected core branch and metric"
        )

    policy_actions = {
        "missing",
        "ambiguous",
        "multiply_coded",
        "unclassified",
    }
    for key in sorted(policy_actions):
        enum(
            policies[key],
            OUTCOME_POLICY_ACTIONS,
            f"threshold_rule.policies.{key}",
        )
    for key in ("withdrawn", "excluded"):
        if policies[key] != "exclude-session":
            raise ReaderEvidenceError(
                f"threshold_rule.policies.{key} must preserve session exclusion"
            )
    if policies["rounding"] != "exact-decimal-no-rounding":
        raise ReaderEvidenceError(
            "threshold_rule.policies.rounding must preserve exact comparison"
        )
    enum(
        policies["coder_adjudication"],
        CODER_ADJUDICATION_ACTIONS,
        "threshold_rule.policies.coder_adjudication",
    )

    required: set[str] = set()
    threshold_ids: set[str] = {str(core_spec["threshold_id"])}
    for index, raw in enumerate(
        as_list(rule["required_target_thresholds"], "threshold_rule.required_target_thresholds")
    ):
        path = f"threshold_rule.required_target_thresholds[{index}]"
        item = as_object(raw, path)
        exact_keys(item, TARGET_THRESHOLD_KEYS, path)
        target_id = as_text(item["target_id"], f"{path}.target_id")
        if target_id not in REQUIRED_TARGETS or target_id in required:
            raise ReaderEvidenceError(f"{path}.target_id: unknown or duplicate target")
        required.add(target_id)
        spec = validate_threshold_spec(
            item["threshold"],
            f"{path}.threshold",
            allowed_metrics={
                "target-identification-count",
                "target-identification-rate",
            },
            scope_refs={target_id},
        )
        if (
            spec["operator"] not in {"gte", "gt"}
            or Decimal(str(spec["value"])) <= 0
            or (
                spec["metric"] == "target-identification-rate"
                and spec["operator"] == "gt"
                and Decimal(str(spec["value"])) >= 1
            )
        ):
            raise ReaderEvidenceError(
                f"{path}.threshold: target success boundary must be positive and reachable"
            )
        threshold_ids.add(str(spec["threshold_id"]))
    if required != set(REQUIRED_TARGETS):
        raise ReaderEvidenceError(
            "required_target_thresholds must cover every required target"
        )

    non_core_severities = {
        str(item["severity_id"])
        for item in misconceptions.values()
        if item["core"] is False
    }
    mapped_non_core: set[str] = set()
    for index, raw in enumerate(
        as_list(rule["non_core_thresholds"], "threshold_rule.non_core_thresholds")
    ):
        path = f"threshold_rule.non_core_thresholds[{index}]"
        item = as_object(raw, path)
        exact_keys(item, SEVERITY_THRESHOLD_KEYS, path)
        severity_id = opaque_id(item["severity_id"], f"{path}.severity_id")
        if severity_id not in non_core_severities or severity_id in mapped_non_core:
            raise ReaderEvidenceError(
                f"{path}.severity_id: unknown, core, or duplicate severity"
            )
        mapped_non_core.add(severity_id)
        spec = validate_threshold_spec(
            item["threshold"],
            f"{path}.threshold",
            allowed_metrics={
                "severity-session-finding-count",
                "severity-session-finding-rate",
                "severity-occurrence-count",
                "severity-occurrence-rate",
            },
            scope_refs={severity_id},
        )
        if (
            spec["operator"] not in {"lt", "lte"}
            or (
                spec["operator"] == "lt"
                and Decimal(str(spec["value"])) <= 0
            )
            or (
                str(spec["metric"]).endswith("-rate")
                and spec["operator"] == "lte"
                and Decimal(str(spec["value"])) >= 1
            )
        ):
            raise ReaderEvidenceError(
                f"{path}.threshold: non-core boundary must be adverse and falsifiable"
            )
        threshold_ids.add(str(spec["threshold_id"]))
    if mapped_non_core != non_core_severities:
        raise ReaderEvidenceError(
            "non_core_thresholds must cover every used non-core severity"
        )

    minimum = validate_threshold_spec(
        rule["minimum_evaluable_evidence"],
        "threshold_rule.minimum_evaluable_evidence",
        allowed_metrics={"admissible-session-count"},
        scope_refs={rule_id},
    )
    if (
        minimum["operator"] not in {"gte", "gt"}
        or Decimal(str(minimum["value"])) <= 0
    ):
        raise ReaderEvidenceError(
            "minimum evaluable evidence must require a positive admitted count"
        )
    threshold_ids.add(str(minimum["threshold_id"]))
    expected_threshold_count = 2 + len(required) + len(mapped_non_core)
    if len(threshold_ids) != expected_threshold_count:
        raise ReaderEvidenceError(
            "threshold IDs must be unique across the complete rule"
        )
    expected_digest = canonical_sha(rule, omit="rule_sha256")
    sha(rule["rule_sha256"], "threshold_rule.rule_sha256", expected_digest)
    return rule, set(misconceptions)



def committed_file_bytes(
    commit: str,
    relative: pathlib.Path,
    path: str,
) -> bytes:
    try:
        shown = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "show",
                f"{commit}:{relative.as_posix()}",
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(
            f"{path}: cannot inspect candidate commit artifact: {exc}"
        ) from exc
    if shown.returncode != 0:
        detail = shown.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            f"{path}: candidate commit artifact is unavailable"
            + (f": {detail}" if detail else "")
        )
    return shown.stdout


def validate_candidate_relevant_state(
    candidate: dict[str, object],
    *,
    candidate_commit: str,
    decision_bytes: bytes,
    checker_bytes: bytes,
    candidate_raw: bytes,
    valid_pilot: bool,
) -> None:
    """Validate candidate-only domains without recursing through ratification."""
    exact_keys(candidate, ROOT_KEYS, "ratification candidate source")
    walk_keys(candidate, "ratification candidate source")
    validate_protocol(candidate, decision_bytes=decision_bytes)
    validate_privacy(candidate)
    route = as_object(candidate["route"], "ratification candidate route")
    if (
        route["reviewer_custody_attestation"] is not None
        or route["evidence_admission_gate_binding"] is not None
    ):
        raise ReaderEvidenceError(
            "ratification candidate route must precede reviewer and gate availability bindings"
        )
    route_status, _gate_sha256, _checker_sha256 = validate_route_readiness(
        candidate,
        valid_pilot,
        verify_live_artifacts=False,
        expected_structural_checker_sha256=sha256_bytes(checker_bytes),
    )
    validate_claim(candidate, route_status, False)
    validate_acceptance(candidate)
    validate_history_closure(
        candidate,
        source_commit=candidate_commit,
        source_raw=candidate_raw,
    )


def validate_candidate_commit(
    candidate_commit: str,
    expected_rule_sha256: str,
    expected_pilot_attempt_id: str,
    expected_packet_sha256: str,
    expected_sensitivity_sha256: str,
    expected_fixed_protocol_sha256: str,
) -> dict[str, object]:
    try:
        ancestor = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "merge-base",
                "--is-ancestor",
                candidate_commit,
                "HEAD",
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
        completed = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "show",
                f"{candidate_commit}:{DEFAULT_SOURCE.as_posix()}",
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(
            f"ratification.candidate_commit cannot be inspected: {exc}"
        ) from exc
    if ancestor.returncode != 0:
        raise ReaderEvidenceError(
            "ratification.candidate_commit must be an ancestor of the current checkout"
        )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            f"ratification.candidate_commit has no candidate source: {detail}"
        )
    try:
        candidate = as_object(
            json.loads(
                completed.stdout.decode("utf-8"),
                object_pairs_hook=reject_duplicate_keys,
            ),
            "ratification candidate source",
        )
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReaderEvidenceError(
            "ratification.candidate_commit contains invalid candidate JSON"
        ) from exc
    candidate_decision_bytes = committed_file_bytes(
        candidate_commit,
        PROTOCOL_DECISION,
        "ratification.candidate_commit.protocol_decision",
    )
    candidate_checker_bytes = committed_file_bytes(
        candidate_commit,
        pathlib.Path(STRUCTURAL_CHECKER_REF.split("::", 1)[0]),
        "ratification.candidate_commit.structural_checker",
    )
    exact_keys(candidate, ROOT_KEYS, "ratification candidate source")
    walk_keys(candidate, "ratification candidate source")
    validate_protocol(candidate, decision_bytes=candidate_decision_bytes)
    validate_privacy(candidate)

    if candidate.get("threshold_status") != "candidate":
        raise ReaderEvidenceError(
            "ratification.candidate_commit must record candidate threshold status"
        )
    if candidate.get("ratification") is not None:
        raise ReaderEvidenceError(
            "ratification.candidate_commit must precede author ratification"
        )
    if (
        candidate.get("holdout_status") != "not-frozen"
        or candidate.get("result") != "not-run"
    ):
        raise ReaderEvidenceError(
            "ratification.candidate_commit may contain no holdout result"
        )
    candidate_holdout = as_object(
        candidate.get("holdout"), "ratification candidate holdout"
    )
    exact_keys(
        candidate_holdout,
        HOLDOUT_KEYS,
        "ratification candidate holdout",
    )
    if (
        candidate_holdout["active_attempt_id"] is not None
        or candidate_holdout["attempts"]
    ):
        raise ReaderEvidenceError(
            "ratification.candidate_commit may contain no holdout attempt"
        )

    candidate_protocol = as_object(
        candidate.get("protocol"), "ratification candidate protocol"
    )
    exact_keys(
        candidate_protocol,
        PROTOCOL_KEYS,
        "ratification candidate protocol",
    )
    if (
        canonical_sha_omitting(
            candidate_protocol, {"decision_sha256"}
        )
        != expected_fixed_protocol_sha256
    ):
        raise ReaderEvidenceError(
            "ratification.candidate_commit fixed protocol differs from the ratified basis"
        )
    (
        candidate_valid_pilot,
        validated_packet,
        validated_sensitivity,
        validated_attempt_id,
    ) = validate_pilot(candidate)
    if (
        not candidate_valid_pilot
        or validated_attempt_id != expected_pilot_attempt_id
        or validated_packet is None
        or validated_packet["packet_sha256"] != expected_packet_sha256
        or validated_sensitivity is None
        or validated_sensitivity["sha256"]
        != expected_sensitivity_sha256
    ):
        raise ReaderEvidenceError(
            "ratification.candidate_commit does not contain the same fully validated pilot basis"
        )
    validate_candidate_relevant_state(
        candidate,
        candidate_commit=candidate_commit,
        decision_bytes=candidate_decision_bytes,
        checker_bytes=candidate_checker_bytes,
        candidate_raw=completed.stdout,
        valid_pilot=candidate_valid_pilot,
    )

    validated_rule, _ = validate_threshold_rule(
        candidate, candidate_valid_pilot
    )

    candidate_rule = as_object(
        candidate.get("threshold_rule"),
        "ratification candidate threshold_rule",
    )
    exact_keys(
        candidate_rule,
        THRESHOLD_RULE_KEYS,
        "ratification candidate threshold_rule",
    )
    if candidate_rule != validated_rule:
        raise ReaderEvidenceError(
            "ratification candidate rule changed during validation"
        )
    actual_rule_sha256 = canonical_sha(
        candidate_rule, omit="rule_sha256"
    )
    sha(
        candidate_rule["rule_sha256"],
        "ratification candidate threshold_rule.rule_sha256",
        actual_rule_sha256,
    )
    if actual_rule_sha256 != expected_rule_sha256:
        raise ReaderEvidenceError(
            "ratification.candidate_commit rule differs from the ratified rule"
        )

    pilot = as_object(
        candidate.get("pilot"), "ratification candidate pilot"
    )
    exact_keys(pilot, PILOT_KEYS, "ratification candidate pilot")
    attempts = as_list(
        pilot["attempts"], "ratification candidate pilot.attempts"
    )
    if not attempts or pilot["active_attempt_id"] != expected_pilot_attempt_id:
        raise ReaderEvidenceError(
            "ratification.candidate_commit must bind the same active pilot attempt"
        )
    active = as_object(
        attempts[-1], "ratification candidate active pilot attempt"
    )
    exact_keys(
        active,
        PILOT_ATTEMPT_KEYS,
        "ratification candidate active pilot attempt",
    )
    if (
        active["attempt_id"] != expected_pilot_attempt_id
        or active["attempt_status"] != "completed"
        or active["control_status"] != "watched-failing"
    ):
        raise ReaderEvidenceError(
            "ratification candidate active pilot is not valid and completed"
        )
    sha(
        active["attempt_sha256"],
        "ratification candidate active pilot attempt_sha256",
        canonical_sha(active, omit="attempt_sha256"),
    )
    receipt = as_object(
        active["receipt"], "ratification candidate pilot receipt"
    )
    if receipt.get("protocol_validity") != "valid":
        raise ReaderEvidenceError(
            "ratification candidate pilot protocol is not valid"
        )
    packet = as_object(
        active["decision_packet"], "ratification candidate pilot packet"
    )
    sha(
        packet.get("packet_sha256"),
        "ratification candidate pilot packet_sha256",
        canonical_sha(packet, omit="packet_sha256"),
    )
    if packet["packet_sha256"] != expected_packet_sha256:
        raise ReaderEvidenceError(
            "ratification candidate cites a different pilot packet"
        )
    sensitivity = as_object(
        active["sensitivity_brief"],
        "ratification candidate sensitivity brief",
    )
    sha(
        sensitivity.get("sha256"),
        "ratification candidate sensitivity brief sha256",
        expected_sensitivity_sha256,
    )
    return candidate


def validate_ratification(
    source: dict[str, object],
    rule: dict[str, object],
    pilot_packet: dict[str, object] | None,
    sensitivity_brief: dict[str, object] | None,
    pilot_attempt_id: str | None,
) -> None:
    status = str(source["threshold_status"])
    ratification = source["ratification"]
    if status in {"candidate", "pending-pilot"}:
        if ratification is not None:
            raise ReaderEvidenceError(
                f"{status} threshold may not carry ratification"
            )
        return
    record = as_object(ratification, "ratification")
    exact_keys(record, RATIFICATION_KEYS, "ratification")
    ruling_id = opaque_id(record["ruling_id"], "ratification.ruling_id")
    bound_attempt_id = opaque_id(
        record["pilot_attempt_id"], "ratification.pilot_attempt_id"
    )
    if pilot_attempt_id is None or bound_attempt_id != pilot_attempt_id:
        raise ReaderEvidenceError(
            "ratification must bind the active valid pilot attempt"
        )
    date(record["ratified_date"], "ratification.ratified_date")
    candidate_commit = record["candidate_commit"]
    if not isinstance(candidate_commit, str) or not COMMIT.fullmatch(candidate_commit):
        raise ReaderEvidenceError(
            "ratification.candidate_commit must be a full lowercase commit digest"
        )
    for key in ("author_statement", "question_answered", "rationale"):
        as_text(record[key], f"ratification.{key}")
    if pilot_packet is None or sensitivity_brief is None:
        raise ReaderEvidenceError(
            "ratification requires the frozen pilot packet and sensitivity brief"
        )
    sha(
        record["pilot_packet_sha256"],
        "ratification.pilot_packet_sha256",
        str(pilot_packet["packet_sha256"]),
    )
    sha(
        record["sensitivity_brief_sha256"],
        "ratification.sensitivity_brief_sha256",
        str(sensitivity_brief["sha256"]),
    )
    sha(
        record["rule_sha256"],
        "ratification.rule_sha256",
        str(rule["rule_sha256"]),
    )
    validate_candidate_commit(
        candidate_commit,
        str(rule["rule_sha256"]),
        bound_attempt_id,
        str(pilot_packet["packet_sha256"]),
        str(sensitivity_brief["sha256"]),
        canonical_sha_omitting(
            source["protocol"], {"decision_sha256"}
        ),
    )
    if str(record["ratified_date"]) < str(pilot_packet["frozen_date"]):
        raise ReaderEvidenceError(
            "ratification must follow the frozen pilot decision packet"
        )
    decision_ref = validate_repo_reference(
        record["decision_ref"], "ratification.decision_ref"
    )
    if not decision_ref.startswith(PROTOCOL_DECISION.as_posix() + "::"):
        raise ReaderEvidenceError(
            "ratification.decision_ref must cite the controlling decision record"
        )
    if ruling_id not in decision_ref:
        raise ReaderEvidenceError(
            "ratification.decision_ref must cite the exact second ruling anchor"
        )
    if record["no_holdout_evidence_attestation"] is not True:
        raise ReaderEvidenceError(
            "ratification must attest no holdout evidence existed or was inspected"
        )
    sha(
        record["ratification_sha256"],
        "ratification.ratification_sha256",
        canonical_sha(record, omit="ratification_sha256"),
    )




def validate_frozen_ratification(
    value: object,
    path: str,
    rule: dict[str, object],
    fixed_protocol_sha256: str,
) -> dict[str, object]:
    """Validate one immutable ratification embedded in a holdout attempt."""
    record = as_object(value, path)
    exact_keys(record, RATIFICATION_KEYS, path)
    ruling_id = opaque_id(record["ruling_id"], f"{path}.ruling_id")
    pilot_attempt_id = opaque_id(
        record["pilot_attempt_id"], f"{path}.pilot_attempt_id"
    )
    ratified_date = date(record["ratified_date"], f"{path}.ratified_date")
    candidate_commit = record["candidate_commit"]
    if not isinstance(candidate_commit, str) or not COMMIT.fullmatch(candidate_commit):
        raise ReaderEvidenceError(
            f"{path}.candidate_commit must be a full lowercase commit digest"
        )
    for key in ("author_statement", "question_answered", "rationale"):
        as_text(record[key], f"{path}.{key}")
    packet_sha256 = sha(
        record["pilot_packet_sha256"], f"{path}.pilot_packet_sha256"
    )
    sensitivity_sha256 = sha(
        record["sensitivity_brief_sha256"],
        f"{path}.sensitivity_brief_sha256",
    )
    sha(
        record["rule_sha256"],
        f"{path}.rule_sha256",
        str(rule["rule_sha256"]),
    )
    candidate = validate_candidate_commit(
        candidate_commit,
        str(rule["rule_sha256"]),
        pilot_attempt_id,
        packet_sha256,
        sensitivity_sha256,
        fixed_protocol_sha256,
    )
    candidate_pilot = as_object(
        candidate["pilot"], f"{path}.candidate_pilot"
    )
    candidate_attempts = as_list(
        candidate_pilot["attempts"], f"{path}.candidate_pilot.attempts"
    )
    candidate_attempt = as_object(
        candidate_attempts[-1], f"{path}.candidate_pilot.active_attempt"
    )
    candidate_packet = as_object(
        candidate_attempt["decision_packet"],
        f"{path}.candidate_pilot.decision_packet",
    )
    if ratified_date < str(candidate_packet["frozen_date"]):
        raise ReaderEvidenceError(
            f"{path}: ratification must follow its frozen pilot decision packet"
        )
    decision_ref = validate_repo_reference(
        record["decision_ref"], f"{path}.decision_ref"
    )
    if not decision_ref.startswith(PROTOCOL_DECISION.as_posix() + "::"):
        raise ReaderEvidenceError(
            f"{path}.decision_ref must cite the controlling decision record"
        )
    if ruling_id not in decision_ref:
        raise ReaderEvidenceError(
            f"{path}.decision_ref must cite the exact ruling anchor"
        )
    if record["no_holdout_evidence_attestation"] is not True:
        raise ReaderEvidenceError(
            f"{path} must attest no holdout evidence existed or was inspected"
        )
    sha(
        record["ratification_sha256"],
        f"{path}.ratification_sha256",
        canonical_sha(record, omit="ratification_sha256"),
    )
    return record

def validate_route_readiness(
    source: dict[str, object],
    valid_pilot: bool,
    *,
    verify_live_artifacts: bool = True,
    expected_structural_checker_sha256: str | None = None,
) -> tuple[str, str | None, str]:
    route = as_object(source["route"], "route")
    exact_keys(route, ROUTE_KEYS, "route")
    if route["route_id"] != "FS-RTE-06":
        raise ReaderEvidenceError("route.route_id must be FS-RTE-06")
    route_status = enum(route["route_status"], ROUTE_STATUSES, "route.route_status")
    evidence_status = enum(
        route["evidence_contract_status"],
        {"unbuilt", "implemented"},
        "route.evidence_contract_status",
    )
    if evidence_status != "implemented":
        raise ReaderEvidenceError(
            "route.evidence_contract_status must record this implemented contract"
        )
    structural = validate_artifact(
        route["structural_checker_binding"],
        "route.structural_checker_binding",
        verify_live=verify_live_artifacts,
    )
    if (
        structural["artifact_id"] != STRUCTURAL_CHECKER_ARTIFACT_ID
        or structural["ref"] != STRUCTURAL_CHECKER_REF
    ):
        raise ReaderEvidenceError(
            "route.structural_checker_binding must bind the fixed structural checker"
        )
    if expected_structural_checker_sha256 is not None:
        sha(
            structural["sha256"],
            "route.structural_checker_binding.sha256",
            expected_structural_checker_sha256,
        )


    gate_digest = None
    gate = route["evidence_admission_gate_binding"]
    if gate is not None:
        gate_obj = validate_artifact(
            gate,
            "route.evidence_admission_gate_binding",
            verify_live=verify_live_artifacts,
        )
        if (
            gate_obj["artifact_id"] != EVIDENCE_GATE_ARTIFACT_ID
            or gate_obj["ref"] != EVIDENCE_GATE_REF
        ):
            raise ReaderEvidenceError(
                "route.evidence_admission_gate_binding must bind the fixed executable gate"
            )
        gate_digest = str(gate_obj["sha256"])
        gate_path = resolve(
            pathlib.Path(EVIDENCE_GATE_REF.split("::", 1)[0])
        )
        try:
            gate_test = subprocess.run(
                [sys.executable, str(gate_path), "--self-test"],
                check=False,
                capture_output=True,
                timeout=30,
            )
        except (OSError, subprocess.SubprocessError) as exc:
            raise ReaderEvidenceError(
                f"route evidence gate self-test could not run: {exc}"
            ) from exc
        expected_output = "reader-evidence-admission-gate: self-test passed"
        if (
            gate_test.returncode != 0
            or gate_test.stdout.decode("utf-8", errors="replace").strip()
            != expected_output
        ):
            detail = gate_test.stderr.decode(
                "utf-8", errors="replace"
            ).strip()
            raise ReaderEvidenceError(
                "route evidence gate must pass its fixed executable self-test"
                + (f": {detail}" if detail else "")
            )

    reviewer = route["reviewer_custody_attestation"]
    if reviewer is not None:
        reviewer_obj = as_object(
            reviewer, "route.reviewer_custody_attestation"
        )
        exact_keys(
            reviewer_obj,
            REVIEWER_ATTESTATION_KEYS,
            "route.reviewer_custody_attestation",
        )
        opaque_id(
            reviewer_obj["attestation_id"],
            "route.reviewer_custody_attestation.attestation_id",
        )
        if reviewer_obj["scope"] != REVIEWER_SCOPE:
            raise ReaderEvidenceError(
                "route reviewer attestation has the wrong closed scope"
            )
        if gate_digest is None:
            raise ReaderEvidenceError(
                "route reviewer attestation requires the executable gate binding"
            )
        sha(
            reviewer_obj["evidence_gate_sha256"],
            "route.reviewer_custody_attestation.evidence_gate_sha256",
            gate_digest,
        )
        ref = as_text(
            reviewer_obj["ref"],
            "route.reviewer_custody_attestation.ref",
        )
        if ref != "custody:READER-EVIDENCE-GATE-REVIEW":
            raise ReaderEvidenceError(
                "route reviewer attestation must use the fixed external custody channel"
            )
        validate_external_or_repo_reference(
            ref, "route.reviewer_custody_attestation.ref"
        )
        attested_date = date(
            reviewer_obj["attested_date"],
            "route.reviewer_custody_attestation.attested_date",
        )
        # The envelope fields are validated here; sha256 identifies the
        # externally held reviewer attestation whose truth is out of scope.
        sha(
            reviewer_obj["sha256"],
            "route.reviewer_custody_attestation.sha256",
        )

    control_status = enum(
        route["negative_control_status"],
        CONTROL_STATUSES,
        "route.negative_control_status",
    )
    pilot_control = as_object(source["pilot"], "pilot")["control_status"]
    if control_status != pilot_control:
        raise ReaderEvidenceError(
            "route.negative_control_status must equal the active pilot control"
        )
    available = (
        reviewer is not None
        and gate_digest is not None
        and valid_pilot
        and control_status == "watched-failing"
        and source["threshold_status"] == "author-ratified"
    )
    expected_route = "available" if available else "unbuilt"
    if route_status != expected_route:
        raise ReaderEvidenceError(
            f"route.route_status must be {expected_route} for its complete tuple"
        )
    return route_status, gate_digest, str(structural["sha256"])


def validate_claim(
    source: dict[str, object],
    route_status: str,
    valid_holdout_pass: bool,
) -> None:
    claim = as_object(source["claim"], "claim")
    exact_keys(claim, CLAIM_KEYS, "claim")
    if claim["claim_id"] != "FS-CLM-37":
        raise ReaderEvidenceError("claim.claim_id must be FS-CLM-37")
    validate_repo_reference(claim["result_ref"], "claim.result_ref")
    if valid_holdout_pass:
        expected = ("Evidenced", "none")
    elif route_status == "available":
        expected = ("Unestablished", "evidence-pending")
    else:
        expected = ("Unestablished", "route-unbuilt")
    actual = (claim["posture"], claim["disposition"])
    if actual != expected:
        raise ReaderEvidenceError(
            f"claim posture/disposition must be {expected[0]}/{expected[1]} for current evidence state"
        )

def validate_holdout_pre_registration(
    value: object,
    path: str,
    *,
    verify_live: bool,
    fixed_protocol_sha256: str,
    expected_structural_checker_sha256: str | None,
    expected_predecessor_attempt_sha256: str | None = None,
    expected_prior_history_head_sha256: str | None = None,
    enforce_history_binding: bool = False,
) -> dict[str, object]:
    registration = as_object(value, path)
    exact_keys(registration, HOLDOUT_PRE_REGISTRATION_KEYS, path)
    opaque_id(registration["study_id"], f"{path}.study_id")
    registered_date = date(registration["registered_date"], f"{path}.registered_date")
    validate_preregistration_history_binding(
        registration,
        path,
        expected_predecessor_attempt_sha256=expected_predecessor_attempt_sha256,
        expected_prior_history_head_sha256=expected_prior_history_head_sha256,
        enforce_expected=enforce_history_binding,
    )

    sha(
        registration["fixed_protocol_sha256"],
        f"{path}.fixed_protocol_sha256",
        fixed_protocol_sha256,
    )
    sha(registration["rule_sha256"], f"{path}.rule_sha256")
    sha(
        registration["ratification_sha256"],
        f"{path}.ratification_sha256",
    )
    sha(registration["evidence_gate_sha256"], f"{path}.evidence_gate_sha256")
    if expected_structural_checker_sha256 is None:
        sha(registration["structural_checker_sha256"], f"{path}.structural_checker_sha256")
    else:
        sha(
            registration["structural_checker_sha256"],
            f"{path}.structural_checker_sha256",
            expected_structural_checker_sha256,
        )
    for key in (
        "revised_instrument",
        "rubric",
        "sample_rule",
        "recruitment_rule",
        "disclosure_set",
        "study_protocol",
    ):
        validate_artifact(
            registration[key], f"{path}.{key}", verify_live=verify_live
        )
    candidate = as_object(
        registration["release_candidate"], f"{path}.release_candidate"
    )
    exact_keys(candidate, RELEASE_CANDIDATE_KEYS, f"{path}.release_candidate")
    opaque_id(
        candidate["candidate_id"], f"{path}.release_candidate.candidate_id"
    )
    artifacts = as_list(
        candidate["artifacts"], f"{path}.release_candidate.artifacts"
    )
    if not artifacts:
        raise ReaderEvidenceError(
            f"{path}.release_candidate.artifacts: must not be empty"
        )
    artifact_ids: set[str] = set()
    artifact_refs: set[str] = set()
    for index, artifact in enumerate(artifacts):
        artifact_obj = validate_artifact(
            artifact,
            f"{path}.release_candidate.artifacts[{index}]",
            verify_live=verify_live,
        )
        artifact_id = str(artifact_obj["artifact_id"])
        artifact_ref = str(artifact_obj["ref"])
        if artifact_id in artifact_ids or artifact_ref in artifact_refs:
            raise ReaderEvidenceError(
                f"{path}.release_candidate.artifacts[{index}]: duplicate identity or reference"
            )
        artifact_ids.add(artifact_id)
        artifact_refs.add(artifact_ref)
    sha(
        candidate["manifest_sha256"],
        f"{path}.release_candidate.manifest_sha256",
        canonical_sha(candidate, omit="manifest_sha256"),
    )

    commitment = registration["commitment"]
    if commitment is not None:
        commitment_obj = as_object(commitment, f"{path}.commitment")
        exact_keys(commitment_obj, COMMITMENT_KEYS, f"{path}.commitment")
        opaque_id(
            commitment_obj["commitment_id"],
            f"{path}.commitment.commitment_id",
        )
        for key in (
            "nonce_commitment_sha256",
            "committed_preimage_sha256",
            "custody_attestation_sha256",
        ):
            sha(commitment_obj[key], f"{path}.commitment.{key}")

    binding = validate_freeze_binding(
        registration["freeze_binding"],
        f"{path}.freeze_binding",
        frozen_value=registration,
        digest_field="pre_registration_sha256",
    )
    if str(binding["frozen_at"])[:10] < registered_date:
        raise ReaderEvidenceError(
            f"{path}.freeze_binding: freeze cannot precede registration"
        )
    sha(
        registration["pre_registration_sha256"],
        f"{path}.pre_registration_sha256",
        canonical_sha(registration, omit="pre_registration_sha256"),
    )
    return registration

def validate_deviations(
    value: object, path: str
) -> dict[str, dict[str, object]]:
    result: dict[str, dict[str, object]] = {}
    for index, raw in enumerate(as_list(value, path)):
        item_path = f"{path}[{index}]"
        item = as_object(raw, item_path)
        exact_keys(item, DEVIATION_KEYS, item_path)
        deviation_id = opaque_id(
            item["deviation_id"], f"{item_path}.deviation_id"
        )
        if deviation_id in result:
            raise ReaderEvidenceError(f"{item_path}.deviation_id: duplicate")
        code_value = opaque_id(item["code"], f"{item_path}.code")
        if not code_value.startswith("RE-DEV-CODE-"):
            raise ReaderEvidenceError(
                f"{item_path}.code: expected a closed RE-DEV-CODE-* value"
            )
        enum(
            item["impact"],
            {"none", "session-inadmissible", "holdout-void"},
            f"{item_path}.impact",
        )
        opaque_id(
            item["custody_attestation_id"],
            f"{item_path}.custody_attestation_id",
        )
        result[deviation_id] = item
    return result


def validate_custody(
    value: object, path: str
) -> dict[str, dict[str, object]]:
    result: dict[str, dict[str, object]] = {}
    external_digests: set[str] = set()
    refs = {
        "session-record": "custody:READER-EVIDENCE-SESSION",
        "study-freshness": "custody:READER-EVIDENCE-FRESHNESS",
        "deviation": "custody:READER-EVIDENCE-DEVIATION",
        "commitment": "custody:READER-EVIDENCE-COMMITMENT",
    }
    for index, raw in enumerate(as_list(value, path)):
        item_path = f"{path}[{index}]"
        item = as_object(raw, item_path)
        exact_keys(item, CUSTODY_KEYS, item_path)
        attestation_id = opaque_id(
            item["attestation_id"], f"{item_path}.attestation_id"
        )
        if attestation_id in result:
            raise ReaderEvidenceError(
                f"{item_path}.attestation_id: duplicate"
            )
        opaque_id(item["study_id"], f"{item_path}.study_id")
        scope = enum(item["scope"], CUSTODY_SCOPES, f"{item_path}.scope")
        commitment = item["record_commitment_sha256"]
        if scope == "session-record":
            sha(commitment, f"{item_path}.record_commitment_sha256")
        elif commitment is not None:
            raise ReaderEvidenceError(
                f"{item_path}.record_commitment_sha256: only session custody may bind a record"
            )
        ref = as_text(item["ref"], f"{item_path}.ref")
        if ref != refs[scope]:
            raise ReaderEvidenceError(
                f"{item_path}.ref: custody scope requires its fixed external channel"
            )
        validate_external_or_repo_reference(ref, f"{item_path}.ref")
        digest = sha(item["sha256"], f"{item_path}.sha256")
        if digest in external_digests:
            raise ReaderEvidenceError(
                f"{item_path}.sha256: duplicate external attestation digest"
            )
        external_digests.add(digest)
        freshness = boolean(
            item["freshness_attested"], f"{item_path}.freshness_attested"
        )
        if freshness and scope != "study-freshness":
            raise ReaderEvidenceError(
                f"{item_path}: only a study-freshness attestation may attest freshness"
            )
        sha(
            item["record_sha256"],
            f"{item_path}.record_sha256",
            canonical_sha(item, omit="record_sha256"),
        )
        result[attestation_id] = item
    return result


def validate_record_links(
    sessions: list[dict[str, object]],
    deviations: dict[str, dict[str, object]],
    custody: dict[str, dict[str, object]],
    path: str,
    *,
    expected_study_id: str | None,
    commitment: dict[str, object] | None,
) -> None:
    referenced: set[str] = set()
    for attestation_id, item in custody.items():
        if expected_study_id is not None and item["study_id"] != expected_study_id:
            raise ReaderEvidenceError(
                f"{path}: custody attestation cites a different study"
            )
        if item["scope"] == "study-freshness":
            referenced.add(attestation_id)
    for deviation_id, deviation in deviations.items():
        attestation_id = str(deviation["custody_attestation_id"])
        item = custody.get(attestation_id)
        if item is None or item["scope"] != "deviation":
            raise ReaderEvidenceError(
                f"{path}: deviation {deviation_id} lacks deviation custody"
            )
        referenced.add(attestation_id)
    for session in sessions:
        for deviation_id in session["deviation_ids"]:
            if deviation_id not in deviations:
                raise ReaderEvidenceError(
                    f"{path}: session cites an unknown deviation"
                )
            if (
                session["admissibility"] == "admissible"
                and deviations[deviation_id]["impact"] == "session-inadmissible"
            ):
                raise ReaderEvidenceError(
                    f"{path}: session-inadmissible deviation remains admitted"
                )
        matching_session_custody = False
        for attestation_id in session["custody_attestation_ids"]:
            item = custody.get(attestation_id)
            if item is None:
                raise ReaderEvidenceError(
                    f"{path}: session cites unknown custody"
                )
            if (
                item["scope"] == "session-record"
                and item["record_commitment_sha256"]
                == session["record_commitment_sha256"]
            ):
                matching_session_custody = True
            referenced.add(attestation_id)
        if session["admissibility"] == "admissible" and not matching_session_custody:
            raise ReaderEvidenceError(
                f"{path}: admitted session lacks matching record custody"
            )
        if (
            session["admissibility"] == "inadmissible"
            and not any(
                deviations[item]["impact"] == "session-inadmissible"
                for item in session["deviation_ids"]
            )
        ):
            raise ReaderEvidenceError(
                f"{path}: inadmissible session lacks a coded exclusion deviation"
            )
    if commitment is not None:
        matches = [
            attestation_id
            for attestation_id, item in custody.items()
            if item["scope"] == "commitment"
            and item["sha256"] == commitment["custody_attestation_sha256"]
        ]
        if len(matches) != 1:
            raise ReaderEvidenceError(
                f"{path}: commitment must bind exactly one custody attestation"
            )
        referenced.add(matches[0])
    if referenced != set(custody):
        raise ReaderEvidenceError(
            f"{path}: every public custody record must have a closed evidence role"
        )


def resolved_policy_action(
    status: str,
    adjudication: str,
    policies: dict[str, object],
    final_statuses: set[str],
) -> str:
    if status in final_statuses:
        return "final"
    if adjudication == "unresolved":
        return {
            "unresolved-count-adverse": "count-adverse",
            "unresolved-exclude-observation": "exclude-observation",
            "unresolved-not-evaluable": "study-not-evaluable",
        }[str(policies["coder_adjudication"])]
    action = str(policies[status.replace("-", "_")])
    return "study-not-evaluable" if action == "require-adjudication" else action


def issue_summary(issues: list[str]) -> list[dict[str, object]]:
    return [
        {"issue": issue, "count": issues.count(issue)}
        for issue in sorted(set(issues))
    ]


def metric_observation(
    spec: dict[str, object],
    rule: dict[str, object],
    target_values: dict[str, list[bool | None]],
    misconception_values: list[dict[str, tuple[int, int] | None]],
    misconceptions: dict[str, dict[str, object]],
) -> dict[str, object]:
    metric = str(spec["metric"])
    scope = set(str(item) for item in spec["scope_refs"])
    if metric == "admissible-session-count":
        count = len(misconception_values)
        return {
            "kind": "numeric",
            "numerator": count,
            "denominator": None,
            "eligible": count,
        }
    if metric.startswith("target-identification-"):
        target_id = next(iter(scope))
        included = [item for item in target_values[target_id] if item is not None]
        numerator = sum(1 for item in included if item)
        denominator = len(included) if metric.endswith("-rate") else None
        return {
            "kind": "numeric",
            "numerator": numerator,
            "denominator": denominator,
            "eligible": len(included),
        }

    if metric.startswith("core-"):
        misconception_ids = scope
    else:
        severity_id = next(iter(scope))
        misconception_ids = {
            misconception_id
            for misconception_id, item in misconceptions.items()
            if item["severity_id"] == severity_id and item["core"] is False
        }

    session_findings = 0
    eligible_sessions = 0
    occurrences = 0
    opportunities = 0
    for session in misconception_values:
        included = [
            session[misconception_id]
            for misconception_id in misconception_ids
            if session[misconception_id] is not None
        ]
        if included:
            eligible_sessions += 1
            if any(item[0] > 0 for item in included):
                session_findings += 1
            occurrences += sum(item[0] for item in included)
            opportunities += sum(item[1] for item in included)

    if metric == "core-finding-present":
        eligible = (
            eligible_sessions
            if rule["repetition_unit"] == "admissible-session"
            else opportunities
        )
        return {
            "kind": "qualitative",
            "token": "present" if occurrences > 0 else "absent",
            "eligible": eligible,
        }
    if metric == "core-finding-count":
        session_unit = rule["repetition_unit"] == "admissible-session"
        numerator = session_findings if session_unit else occurrences
        eligible = eligible_sessions if session_unit else opportunities
        return {
            "kind": "numeric",
            "numerator": numerator,
            "denominator": None,
            "eligible": eligible,
        }
    if metric == "core-finding-rate":
        if rule["repetition_unit"] == "admissible-session":
            return {
                "kind": "numeric",
                "numerator": session_findings,
                "denominator": eligible_sessions,
                "eligible": eligible_sessions,
            }
        return {
            "kind": "numeric",
            "numerator": occurrences,
            "denominator": opportunities,
            "eligible": opportunities,
        }
    if metric == "severity-session-finding-count":
        return {
            "kind": "numeric",
            "numerator": session_findings,
            "denominator": None,
            "eligible": eligible_sessions,
        }
    if metric == "severity-session-finding-rate":
        return {
            "kind": "numeric",
            "numerator": session_findings,
            "denominator": eligible_sessions,
            "eligible": eligible_sessions,
        }
    if metric == "severity-occurrence-count":
        return {
            "kind": "numeric",
            "numerator": occurrences,
            "denominator": None,
            "eligible": opportunities,
        }
    if metric == "severity-occurrence-rate":
        return {
            "kind": "numeric",
            "numerator": occurrences,
            "denominator": opportunities,
            "eligible": opportunities,
        }
    raise ReaderEvidenceError(f"unsupported metric during evaluation: {metric}")


def compare_threshold(
    spec: dict[str, object],
    observation: dict[str, object],
) -> tuple[bool | None, dict[str, object]]:
    if observation["kind"] == "qualitative":
        observed = str(observation["token"])
    else:
        numerator = int(observation["numerator"])
        denominator = observation["denominator"]
        observed = str(numerator)
        if denominator is not None:
            observed = f"{numerator}/{int(denominator)}"
    trace = {
        "threshold_id": spec["threshold_id"],
        "metric": spec["metric"],
        "observed": observed,
        "comparison": None,
    }
    if int(observation["eligible"]) == 0:
        return None, trace

    if observation["kind"] == "qualitative":
        passed = observed == str(spec["value"])
        trace["comparison"] = passed
        return passed, trace

    numerator = int(observation["numerator"])
    denominator = observation["denominator"]
    boundary = Decimal(str(spec["value"]))
    left = Decimal(numerator)
    right = (
        boundary
        if denominator is None
        else boundary * Decimal(int(denominator))
    )
    passed = comparator(str(spec["operator"]), left, right)
    trace["comparison"] = passed
    return passed, trace


def ordered_evaluation_trace(
    *,
    protocol_valid: bool,
    evaluable: bool,
    core_veto: bool,
    required_targets_pass: bool,
    non_core_pass: bool,
    checks: dict[str, list[dict[str, object]]],
) -> dict[str, object]:
    statuses: dict[str, str] = {}
    reached = True
    verdict = "pass"
    for stage in EVALUATION_ORDER:
        if not reached:
            statuses[stage] = "not-reached"
            continue
        if stage == "protocol-validity":
            passed = protocol_valid
            if not passed:
                verdict = "not-evaluable"
        elif stage == "evaluability":
            passed = evaluable
            if not passed:
                verdict = "not-evaluable"
        elif stage == "core-veto":
            passed = not core_veto
            if not passed:
                verdict = "fail"
        elif stage == "required-targets":
            passed = required_targets_pass
            if not passed:
                verdict = "fail"
        elif stage == "non-core-rules":
            passed = non_core_pass
            if not passed:
                verdict = "fail"
        else:
            passed = True
            verdict = "pass"
        statuses[stage] = "pass" if passed else "fail"
        reached = passed
    return {
        "order": list(EVALUATION_ORDER),
        "stages": [
            {
                "stage": stage,
                "status": statuses[stage],
                "checks": checks.get(stage, []) if statuses[stage] != "not-reached" else [],
            }
            for stage in EVALUATION_ORDER
        ],
        "verdict": verdict,
    }


def evaluate_holdout(
    rule: dict[str, object],
    sessions: list[dict[str, object]],
    protocol_validity: str,
) -> dict[str, object]:
    protocol_valid = protocol_validity == "valid"
    protocol_checks = [
        {
            "check": "protocol-validity",
            "observed": protocol_validity,
            "comparison": protocol_valid,
        }
    ]
    if not protocol_valid:
        return ordered_evaluation_trace(
            protocol_valid=False,
            evaluable=False,
            core_veto=False,
            required_targets_pass=False,
            non_core_pass=False,
            checks={"protocol-validity": protocol_checks},
        )

    policies = as_object(rule["policies"], "threshold_rule.policies")
    misconceptions = {
        str(item["misconception_id"]): as_object(item, "threshold misconception")
        for item in as_list(rule["misconceptions"], "threshold_rule.misconceptions")
    }
    admitted = [
        session for session in sessions if session["admissibility"] == "admissible"
    ]
    target_values: dict[str, list[bool | None]] = {
        target_id: [] for target_id in REQUIRED_TARGETS
    }
    misconception_values: list[dict[str, tuple[int, int] | None]] = []
    issues: list[str] = []

    for session in admitted:
        for outcome in session["target_outcomes"]:
            target_id = str(outcome["target_id"])
            status = str(outcome["status"])
            action = resolved_policy_action(
                status,
                str(outcome["adjudication"]),
                policies,
                FINAL_TARGET_STATUSES,
            )
            if action == "final":
                value: bool | None = status == "identified"
            elif action == "count-adverse":
                value = False
            elif action == "exclude-observation":
                value = None
            else:
                value = None
                issues.append(f"target-{status}-not-evaluable")
            target_values[target_id].append(value)

        session_values: dict[str, tuple[int, int] | None] = {}
        for outcome in session["misconception_outcomes"]:
            misconception_id = str(outcome["misconception_id"])
            status = str(outcome["status"])
            action = resolved_policy_action(
                status,
                str(outcome["adjudication"]),
                policies,
                FINAL_MISCONCEPTION_STATUSES,
            )
            occurrences = int(str(outcome["occurrences"]))
            opportunities = int(str(outcome["opportunities"]))
            if action == "final":
                value = (occurrences, opportunities)
            elif action == "count-adverse":
                value = (opportunities, opportunities)
            elif action == "exclude-observation":
                value = None
            else:
                value = None
                issues.append(f"misconception-{status}-not-evaluable")
            session_values[misconception_id] = value
        misconception_values.append(session_values)

    minimum_spec = as_object(
        rule["minimum_evaluable_evidence"], "minimum evaluable evidence"
    )
    minimum_observation = metric_observation(
        minimum_spec,
        rule,
        target_values,
        misconception_values,
        misconceptions,
    )
    minimum_pass, minimum_check = compare_threshold(
        minimum_spec, minimum_observation
    )

    core_spec = as_object(rule["core_failure_threshold"], "core threshold")
    core_observation = metric_observation(
        core_spec,
        rule,
        target_values,
        misconception_values,
        misconceptions,
    )
    core_veto, core_check = compare_threshold(core_spec, core_observation)

    target_checks: list[dict[str, object]] = []
    target_results: list[bool | None] = []
    for item in as_list(
        rule["required_target_thresholds"], "required target thresholds"
    ):
        entry = as_object(item, "required target threshold")
        spec = as_object(entry["threshold"], "required target threshold spec")
        observation = metric_observation(
            spec,
            rule,
            target_values,
            misconception_values,
            misconceptions,
        )
        passed, check = compare_threshold(spec, observation)
        target_results.append(passed)
        target_checks.append(check)

    non_core_checks: list[dict[str, object]] = []
    non_core_results: list[bool | None] = []
    for item in as_list(rule["non_core_thresholds"], "non-core thresholds"):
        entry = as_object(item, "non-core threshold")
        spec = as_object(entry["threshold"], "non-core threshold spec")
        observation = metric_observation(
            spec,
            rule,
            target_values,
            misconception_values,
            misconceptions,
        )
        passed, check = compare_threshold(spec, observation)
        non_core_results.append(passed)
        non_core_checks.append(check)

    all_results = [minimum_pass, core_veto, *target_results, *non_core_results]
    evaluable = not issues and minimum_pass is True and all(
        result is not None for result in all_results
    )
    evaluability_checks = [
        minimum_check,
        *issue_summary(issues),
    ]
    return ordered_evaluation_trace(
        protocol_valid=True,
        evaluable=evaluable,
        core_veto=core_veto is True,
        required_targets_pass=all(result is True for result in target_results),
        non_core_pass=all(result is True for result in non_core_results),
        checks={
            "protocol-validity": protocol_checks,
            "evaluability": evaluability_checks,
            "core-veto": [core_check],
            "required-targets": target_checks,
            "non-core-rules": non_core_checks,
            "pass": [],
        },
    )


def validate_commitment_reveal(
    value: object,
    path: str,
    commitment: dict[str, object] | None,
    custody: dict[str, dict[str, object]],
    attempt_status: str,
    *,
    reveal_required: bool,
    verify_live: bool,
    terminal_at: str | None,
) -> dict[str, object] | None:
    if commitment is None:
        if value is not None:
            raise ReaderEvidenceError(
                f"{path}: reveal cannot exist without a preregistered commitment"
            )
        return None
    if attempt_status == "frozen":
        if value is not None:
            raise ReaderEvidenceError(
                f"{path}: nonce preimage cannot be revealed before the holdout ends"
            )
        return None
    if value is None:
        if reveal_required:
            raise ReaderEvidenceError(
                f"{path}: a completed or run-void holdout must reveal its commitment"
            )
        return None

    reveal = as_object(value, path)
    exact_keys(reveal, COMMITMENT_REVEAL_KEYS, path)
    commitment_id = opaque_id(
        reveal["commitment_id"], f"{path}.commitment_id"
    )
    if commitment_id != commitment["commitment_id"]:
        raise ReaderEvidenceError(
            f"{path}.commitment_id: does not open the preregistered commitment"
        )
    revealed_at = utc_timestamp(reveal["revealed_at"], f"{path}.revealed_at")
    if terminal_at is None or revealed_at <= terminal_at:
        raise ReaderEvidenceError(
            f"{path}.revealed_at: reveal must strictly follow the attempt terminal time"
        )
    nonce_hex = as_text(reveal["nonce_hex"], f"{path}.nonce_hex")
    if not NONCE_PREIMAGE.fullmatch(nonce_hex):
        raise ReaderEvidenceError(
            f"{path}.nonce_hex: expected at least 32 bytes of lowercase hex"
        )
    preimage = validate_artifact(
        reveal["preimage"], f"{path}.preimage", verify_live=verify_live
    )
    sha(
        preimage["sha256"],
        f"{path}.preimage.sha256",
        str(commitment["committed_preimage_sha256"]),
    )
    opening_digest = sha256_bytes(
        bytes.fromhex(nonce_hex)
        + b"\0"
        + bytes.fromhex(str(preimage["sha256"]))
    )
    sha(
        commitment["nonce_commitment_sha256"],
        "holdout commitment nonce_commitment_sha256",
        opening_digest,
    )
    custody_id = opaque_id(
        reveal["custody_attestation_id"],
        f"{path}.custody_attestation_id",
    )
    custody_record = custody.get(custody_id)
    if (
        custody_record is None
        or custody_record["scope"] != "commitment"
        or custody_record["sha256"]
        != commitment["custody_attestation_sha256"]
    ):
        raise ReaderEvidenceError(
            f"{path}: reveal lacks the exact commitment custody attestation"
        )
    sha(
        reveal["reveal_sha256"],
        f"{path}.reveal_sha256",
        canonical_sha(reveal, omit="reveal_sha256"),
    )
    return reveal


def validate_result_receipt(
    value: object,
    path: str,
    registration: dict[str, object],
    rule: dict[str, object],
    sessions: list[dict[str, object]],
    raw_deviations: list[object],
    raw_custody: list[object],
) -> dict[str, object]:
    receipt = as_object(value, path)
    exact_keys(receipt, RESULT_RECEIPT_KEYS, path)
    opaque_id(receipt["receipt_id"], f"{path}.receipt_id")
    utc_timestamp(receipt["completed_at"], f"{path}.completed_at")
    if receipt["study_id"] != registration["study_id"]:
        raise ReaderEvidenceError(
            f"{path}.study_id: does not match holdout preregistration"
        )
    digest_links = {
        "pre_registration_sha256": registration["pre_registration_sha256"],
        "rule_sha256": registration["rule_sha256"],
        "candidate_manifest_sha256": as_object(
            registration["release_candidate"], "release candidate"
        )["manifest_sha256"],
        "evidence_gate_sha256": registration["evidence_gate_sha256"],
        "coded_records_sha256": canonical_sha(sessions),
        "structural_checker_sha256": registration["structural_checker_sha256"],
        "deviations_sha256": canonical_sha(raw_deviations),
        "custody_records_sha256": canonical_sha(raw_custody),
    }
    for key, expected in digest_links.items():
        sha(receipt[key], f"{path}.{key}", str(expected))

    protocol_validity = enum(
        receipt["protocol_validity"],
        {"valid", "invalid"},
        f"{path}.protocol_validity",
    )
    verdict = enum(
        receipt["verdict"],
        {"not-evaluable", "fail", "pass"},
        f"{path}.verdict",
    )
    trace = evaluate_holdout(rule, sessions, protocol_validity)
    sha(
        receipt["evaluation_trace_sha256"],
        f"{path}.evaluation_trace_sha256",
        canonical_sha(trace),
    )
    if verdict != trace["verdict"]:
        raise ReaderEvidenceError(
            f"{path}.verdict: differs from deterministic evaluation"
        )
    sha(
        receipt["session_classification_sha256"],
        f"{path}.session_classification_sha256",
        canonical_sha(
            [
                {
                    "record_commitment_sha256": record["record_commitment_sha256"],
                    "admissibility": record["admissibility"],
                }
                for record in sessions
            ]
        ),
    )
    custody_digests = text_list(
        receipt["custody_attestation_sha256s"],
        f"{path}.custody_attestation_sha256s",
    )
    expected_custody_digests = [
        str(as_object(item, "holdout custody record")["sha256"])
        for item in raw_custody
    ]
    if custody_digests != expected_custody_digests:
        raise ReaderEvidenceError(
            f"{path}.custody_attestation_sha256s: must exactly bind every custody record"
        )
    for index, digest in enumerate(custody_digests):
        sha(digest, f"{path}.custody_attestation_sha256s[{index}]")
    sha(
        receipt["receipt_sha256"],
        f"{path}.receipt_sha256",
        canonical_sha(receipt, omit="receipt_sha256"),
    )
    return receipt





def build_gate_input(
    attempt_id: str,
    rule: dict[str, object],
    frozen_ratification: dict[str, object],
    registration: dict[str, object],
    sessions: list[dict[str, object]],
    raw_deviations: list[object],
    raw_custody: list[object],
    receipt: dict[str, object],
    commitment_reveal: object,
) -> dict[str, object]:
    """Build the immutable envelope evaluated by the dedicated gate."""
    return {
        "schema_version": 1,
        "attempt_id": attempt_id,
        "active_attempt": True,
        "attempt_status": "completed",
        "threshold_rule": rule,
        "frozen_ratification": frozen_ratification,
        "pre_registration": registration,
        "session_records": sessions,
        "deviations": raw_deviations,
        "custody_attestations": raw_custody,
        "result_receipt": receipt,
        "commitment_reveal": commitment_reveal,
        "current_rule_sha256": rule["rule_sha256"],
        "current_ratification_sha256": frozen_ratification[
            "ratification_sha256"
        ],
        "evidence_gate_sha256": registration["evidence_gate_sha256"],
        "structural_checker_sha256": registration[
            "structural_checker_sha256"
        ],
    }


def validate_gate_admission_receipt(
    value: object,
    path: str,
    gate_input: dict[str, object],
    *,
    expected_decision: str,
    execute_live: bool,
) -> dict[str, object]:
    """Validate the stored gate receipt and replay the bound gate when active."""
    receipt = as_object(value, path)
    exact_keys(receipt, GATE_ADMISSION_RECEIPT_KEYS, path)
    if receipt["schema_version"] != 1 or isinstance(
        receipt["schema_version"], bool
    ):
        raise ReaderEvidenceError(f"{path}.schema_version must be integer 1")
    expected_input_sha256 = canonical_sha(gate_input)
    sha(
        receipt["input_sha256"],
        f"{path}.input_sha256",
        expected_input_sha256,
    )
    gate_sha256 = sha(
        receipt["evidence_gate_sha256"],
        f"{path}.evidence_gate_sha256",
        str(gate_input["evidence_gate_sha256"]),
    )
    decision = enum(
        receipt["decision"], {"admit", "reject"}, f"{path}.decision"
    )
    if decision != expected_decision:
        raise ReaderEvidenceError(
            f"{path}.decision must be {expected_decision} for the validated result"
        )
    sha(
        receipt["receipt_sha256"],
        f"{path}.receipt_sha256",
        canonical_sha(receipt, omit="receipt_sha256"),
    )
    if not execute_live:
        return receipt

    gate_path = resolve(pathlib.Path(EVIDENCE_GATE_REF.split("::", 1)[0]))
    try:
        actual_gate_sha256 = sha256_bytes(gate_path.read_bytes())
        completed = subprocess.run(
            [sys.executable, str(gate_path), "--evaluate"],
            input=(
                json.dumps(
                    gate_input,
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
                + b"\n"
            ),
            check=False,
            capture_output=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(
            f"{path}: bound evidence gate could not run: {exc}"
        ) from exc
    if actual_gate_sha256 != gate_sha256:
        raise ReaderEvidenceError(
            f"{path}: live evidence gate differs from its frozen digest"
        )
    if completed.returncode != 0:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            f"{path}: bound evidence gate failed closed"
            + (f": {detail}" if detail else "")
        )
    try:
        live_receipt = as_object(
            json.loads(
                completed.stdout.decode("utf-8"),
                object_pairs_hook=reject_duplicate_keys,
            ),
            f"{path}.live_output",
        )
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReaderEvidenceError(
            f"{path}: bound evidence gate emitted invalid JSON"
        ) from exc
    exact_keys(
        live_receipt,
        GATE_ADMISSION_RECEIPT_KEYS,
        f"{path}.live_output",
    )
    if live_receipt != receipt:
        raise ReaderEvidenceError(
            f"{path}: stored receipt differs from the bound gate output"
        )
    return receipt

def validate_holdout(
    source: dict[str, object],
    rule: dict[str, object],
    known_misconceptions: set[str],
    route_status: str,
    evidence_gate_sha256: str | None,
    structural_checker_sha256: str,
) -> bool:
    holdout = as_object(source["holdout"], "holdout")
    exact_keys(holdout, HOLDOUT_KEYS, "holdout")
    summary_status = enum(
        source["holdout_status"], HOLDOUT_STATUSES, "holdout_status"
    )
    summary_result = enum(source["result"], RESULTS, "result")
    active_attempt_id = holdout["active_attempt_id"]
    if active_attempt_id is not None:
        active_attempt_id = opaque_id(
            active_attempt_id, "holdout.active_attempt_id"
        )
    attempts = as_list(holdout["attempts"], "holdout.attempts")
    if not attempts:
        if active_attempt_id is not None:
            raise ReaderEvidenceError(
                "holdout.active_attempt_id requires an attempt"
            )
        if summary_status != "not-frozen" or summary_result != "not-run":
            raise ReaderEvidenceError(
                "empty holdout history must remain not-frozen/not-run"
            )
        return False

    if source["threshold_status"] != "author-ratified":
        raise ReaderEvidenceError(
            "every holdout attempt requires an author-ratified rule"
        )
    current_ratification = as_object(source["ratification"], "ratification")
    current_ratification_sha = sha(
        current_ratification["ratification_sha256"],
        "ratification.ratification_sha256",
    )
    fixed_protocol_sha256 = canonical_sha_omitting(
        source["protocol"], {"decision_sha256"}
    )

    seen_ids: set[str] = set()
    previous_sha256: str | None = None
    pilot_attempt_sha256s = [
        sha(
            as_object(raw, f"pilot.attempts[{index}]")["attempt_sha256"],
            f"pilot.attempts[{index}].attempt_sha256",
        )
        for index, raw in enumerate(
            as_list(source["pilot"]["attempts"], "pilot.attempts")
        )
    ]
    holdout_attempt_sha256s: list[str] = []

    active_id = ""
    active_status = ""
    latest_completed_result = "not-run"
    valid_active_pass = False

    for index, raw in enumerate(attempts):
        path = f"holdout.attempts[{index}]"
        attempt = as_object(raw, path)
        exact_keys(attempt, HOLDOUT_ATTEMPT_KEYS, path)
        active = index == len(attempts) - 1

        attempt_id = opaque_id(attempt["attempt_id"], f"{path}.attempt_id")
        if attempt_id in seen_ids:
            raise ReaderEvidenceError(f"{path}.attempt_id: duplicate")
        seen_ids.add(attempt_id)

        declared_previous = attempt["previous_attempt_sha256"]
        if index == 0:
            if declared_previous is not None:
                raise ReaderEvidenceError(
                    f"{path}.previous_attempt_sha256: first attempt must be null"
                )
        else:
            sha(
                declared_previous,
                f"{path}.previous_attempt_sha256",
                previous_sha256,
            )

        attempt_status = enum(
            attempt["attempt_status"],
            HOLDOUT_ATTEMPT_STATUSES,
            f"{path}.attempt_status",
        )
        attempt_result = enum(
            attempt["attempt_result"], RESULTS, f"{path}.attempt_result"
        )
        if not active and attempt_status == "frozen":
            raise ReaderEvidenceError(
                f"{path}: a superseded holdout attempt cannot remain frozen"
            )

        void_reason = attempt["void_reason_code"]
        voided_at = attempt["voided_at"]
        if attempt_status == "void":
            void_code = opaque_id(void_reason, f"{path}.void_reason_code")
            if not void_code.startswith("RE-VOID-"):
                raise ReaderEvidenceError(
                    f"{path}.void_reason_code: expected a closed RE-VOID-* code"
                )
            voided_at = utc_timestamp(voided_at, f"{path}.voided_at")
        else:
            if void_reason is not None:
                raise ReaderEvidenceError(
                    f"{path}.void_reason_code: only a void attempt may carry a reason"
                )
            if voided_at is not None:
                raise ReaderEvidenceError(
                    f"{path}.voided_at: only a void attempt may carry a terminal time"
                )

        shadow = copy.deepcopy(source)
        shadow["threshold_status"] = "author-ratified"
        shadow["threshold_rule"] = attempt["frozen_rule"]
        frozen_rule, frozen_misconceptions = validate_threshold_rule(
            shadow, True
        )
        frozen_ratification = validate_frozen_ratification(
            attempt["frozen_ratification"],
            f"{path}.frozen_ratification",
            frozen_rule,
            fixed_protocol_sha256,
        )

        registration = validate_holdout_pre_registration(
            attempt["pre_registration"],
            f"{path}.pre_registration",
            verify_live=active and attempt_status != "void",
            fixed_protocol_sha256=fixed_protocol_sha256,
            expected_structural_checker_sha256=(
                structural_checker_sha256
                if active and attempt_status != "void"
                else None
            ),
            expected_predecessor_attempt_sha256=previous_sha256,
            expected_prior_history_head_sha256=history_head_sha256(
                pilot_attempt_sha256s, holdout_attempt_sha256s
            ),
            enforce_history_binding=True,
        )
        sha(
            registration["rule_sha256"],
            f"{path}.pre_registration.rule_sha256",
            str(frozen_rule["rule_sha256"]),
        )
        registered_ratification_sha = sha(
            registration["ratification_sha256"],
            f"{path}.pre_registration.ratification_sha256",
            str(frozen_ratification["ratification_sha256"]),
        )
        study_id = str(registration["study_id"])

        sessions = validate_session_records(
            attempt["session_records"],
            f"{path}.session_records",
            expected_study_id=study_id,
            known_misconceptions=frozen_misconceptions,
        )
        raw_deviations = as_list(
            attempt["deviations"], f"{path}.deviations"
        )
        deviations = validate_deviations(
            raw_deviations, f"{path}.deviations"
        )
        raw_custody = as_list(
            attempt["custody_attestations"], f"{path}.custody_attestations"
        )
        custody = validate_custody(
            raw_custody, f"{path}.custody_attestations"
        )
        commitment = (
            as_object(
                registration["commitment"],
                f"{path}.pre_registration.commitment",
            )
            if registration["commitment"] is not None
            else None
        )
        validate_record_links(
            sessions,
            deviations,
            custody,
            f"{path}.record_links",
            expected_study_id=study_id,
            commitment=commitment,
        )

        receipt = None
        if attempt["result_receipt"] is not None:
            receipt = validate_result_receipt(
                attempt["result_receipt"],
                f"{path}.result_receipt",
                registration,
                frozen_rule,
                sessions,
                raw_deviations,
                raw_custody,
            )

        ran = bool(sessions or receipt is not None)
        if attempt_status == "completed" and receipt is not None:
            terminal_at: str | None = str(receipt["completed_at"])
        elif attempt_status == "void":
            terminal_at = str(voided_at)
        else:
            terminal_at = None
        validate_commitment_reveal(
            attempt["commitment_reveal"],
            f"{path}.commitment_reveal",
            commitment,
            custody,
            attempt_status,
            reveal_required=attempt_status in {"completed", "void"},
            verify_live=active,
            terminal_at=terminal_at,
        )

        rule_match = (
            frozen_rule["rule_sha256"] == rule["rule_sha256"]
            and registration["rule_sha256"] == rule["rule_sha256"]
        )
        gate_match = (
            evidence_gate_sha256 is not None
            and registration["evidence_gate_sha256"]
            == evidence_gate_sha256
        )
        structural_match = (
            registration["structural_checker_sha256"]
            == structural_checker_sha256
        )
        ratification_match = (
            registered_ratification_sha == current_ratification_sha
            and frozen_ratification["ratification_sha256"]
            == current_ratification_sha
        )
        current_binding = (
            rule_match
            and gate_match
            and structural_match
            and ratification_match
        )

        if active and attempt_status != "void" and not current_binding:
            raise ReaderEvidenceError(
                f"{path}: active holdout must bind the current rule, ratification, gate, and checker"
            )
        if str(registration["registered_date"]) < str(
            frozen_ratification["ratified_date"]
        ):
            raise ReaderEvidenceError(
                f"{path}: pre-registration cannot precede its frozen ratification"
            )

        voiding_deviation = any(
            item["impact"] == "holdout-void"
            for item in deviations.values()
        )
        freshness_records = [
            item
            for item in custody.values()
            if item["scope"] == "study-freshness"
        ]
        if ran and len(freshness_records) != 1:
            raise ReaderEvidenceError(
                f"{path}: a run holdout requires exactly one freshness attestation"
            )
        freshness_bound = (
            len(freshness_records) == 1
            and freshness_records[0]["freshness_attested"] is True
        )
        freeze_at = str(
            as_object(
                registration["freeze_binding"],
                f"{path}.pre_registration.freeze_binding",
            )["frozen_at"]
        )

        if attempt_status == "frozen":
            if (
                attempt_result != "not-run"
                or sessions
                or raw_deviations
                or raw_custody
                or receipt is not None
                or attempt["commitment_reveal"] is not None
                or attempt["gate_admission_receipt"] is not None
            ):
                raise ReaderEvidenceError(
                    f"{path}: frozen holdout cannot carry run evidence or a result"
                )
        elif attempt_status == "completed":
            if active and route_status != "available":
                raise ReaderEvidenceError(
                    f"{path}: the reader route must be available before the active holdout runs"
                )
            if voiding_deviation:
                raise ReaderEvidenceError(
                    f"{path}: a voiding deviation cannot remain completed"
                )
            if receipt is None or not sessions:
                raise ReaderEvidenceError(
                    f"{path}: completed holdout requires coded sessions and a receipt"
                )
            completed_at = str(receipt["completed_at"])
            if completed_at <= freeze_at:
                raise ReaderEvidenceError(
                    f"{path}: completion must strictly follow the frozen preregistration"
                )
            if completed_at[:10] < str(registration["registered_date"]):
                raise ReaderEvidenceError(
                    f"{path}: completion cannot precede pre-registration"
                )
            if attempt_result != receipt["verdict"]:
                raise ReaderEvidenceError(
                    f"{path}.attempt_result: must equal the recomputed receipt verdict"
                )
            if receipt["protocol_validity"] == "valid" and not freshness_bound:
                raise ReaderEvidenceError(
                    f"{path}: a protocol-valid result requires bound freshness custody"
                )
            gate_input = build_gate_input(
                attempt_id,
                frozen_rule,
                frozen_ratification,
                registration,
                sessions,
                raw_deviations,
                raw_custody,
                receipt,
                attempt["commitment_reveal"],
            )
            expected_gate_decision = (
                "admit"
                if (
                    receipt["protocol_validity"] == "valid"
                    and attempt_result == "pass"
                    and freshness_bound
                )
                else "reject"
            )
            gate_receipt = validate_gate_admission_receipt(
                attempt["gate_admission_receipt"],
                f"{path}.gate_admission_receipt",
                gate_input,
                expected_decision=expected_gate_decision,
                execute_live=active,
            )
            latest_completed_result = attempt_result
            valid_active_pass = (
                active
                and current_binding
                and gate_receipt["decision"] == "admit"
            )
        else:
            if str(voided_at) <= freeze_at:
                raise ReaderEvidenceError(
                    f"{path}: void time must strictly follow the frozen preregistration"
                )
            if attempt["gate_admission_receipt"] is not None:
                raise ReaderEvidenceError(
                    f"{path}: a void holdout may not carry an admission receipt"
                )
            if receipt is None:
                if attempt_result != "not-run":
                    raise ReaderEvidenceError(
                        f"{path}: a pre-result void attempt must remain not-run"
                    )
            else:
                if (
                    receipt["protocol_validity"] != "invalid"
                    or receipt["verdict"] != "not-evaluable"
                    or attempt_result != "not-evaluable"
                ):
                    raise ReaderEvidenceError(
                        f"{path}: a run void must preserve an invalid, not-evaluable receipt"
                    )
                if str(receipt["completed_at"]) > str(voided_at):
                    raise ReaderEvidenceError(
                        f"{path}: receipt completion cannot follow the void time"
                    )
            if ran and not voiding_deviation:
                raise ReaderEvidenceError(
                    f"{path}: a run void requires a custody-linked holdout-void deviation"
                )

        attempt_sha256 = sha(
            attempt["attempt_sha256"],
            f"{path}.attempt_sha256",
            canonical_sha(attempt, omit="attempt_sha256"),
        )
        previous_sha256 = attempt_sha256
        holdout_attempt_sha256s.append(attempt_sha256)
        active_id = attempt_id
        active_status = attempt_status

    if active_attempt_id != active_id:
        raise ReaderEvidenceError(
            "holdout.active_attempt_id must identify the final append-only attempt"
        )
    if summary_status != active_status:
        raise ReaderEvidenceError(
            "holdout_status must equal the active attempt lifecycle"
        )
    if summary_result != latest_completed_result:
        raise ReaderEvidenceError(
            "result must preserve the latest completed non-void outcome"
        )
    return valid_active_pass





def validated_history_snapshot(
    source: dict[str, object],
    path: str,
) -> tuple[
    list[dict[str, object]],
    list[dict[str, object]],
    list[str],
    list[str],
    str,
]:
    """Validate attempt self-digests and derive the ordered two-stream head."""

    def stream(
        container_key: str,
        container_keys: set[str],
        attempt_keys: set[str],
    ) -> tuple[list[dict[str, object]], list[str]]:
        container_path = f"{path}.{container_key}"
        container = as_object(source[container_key], container_path)
        exact_keys(container, container_keys, container_path)
        attempts: list[dict[str, object]] = []
        digests: list[str] = []
        previous: str | None = None
        for index, raw in enumerate(
            as_list(container["attempts"], f"{container_path}.attempts")
        ):
            attempt_path = f"{container_path}.attempts[{index}]"
            attempt = as_object(raw, attempt_path)
            exact_keys(attempt, attempt_keys, attempt_path)
            declared_previous = attempt["previous_attempt_sha256"]
            if index == 0:
                if declared_previous is not None:
                    raise ReaderEvidenceError(
                        f"{attempt_path}.previous_attempt_sha256: first attempt must be null"
                    )
            else:
                sha(
                    declared_previous,
                    f"{attempt_path}.previous_attempt_sha256",
                    previous,
                )
            digest = sha(
                attempt["attempt_sha256"],
                f"{attempt_path}.attempt_sha256",
                canonical_sha(attempt, omit="attempt_sha256"),
            )
            attempts.append(attempt)
            digests.append(digest)
            previous = digest
        return attempts, digests

    pilot_attempts, pilot_digests = stream(
        "pilot", PILOT_KEYS, PILOT_ATTEMPT_KEYS
    )
    holdout_attempts, holdout_digests = stream(
        "holdout", HOLDOUT_KEYS, HOLDOUT_ATTEMPT_KEYS
    )
    return (
        pilot_attempts,
        holdout_attempts,
        pilot_digests,
        holdout_digests,
        history_head_sha256(pilot_digests, holdout_digests),
    )


def committed_reader_evidence(
    commit: str,
    path: str,
) -> tuple[bytes, dict[str, object]]:
    try:
        shown = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "show",
                f"{commit}:{DEFAULT_SOURCE.as_posix()}",
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(
            f"{path}: cannot inspect committed reader-evidence source: {exc}"
        ) from exc
    if shown.returncode != 0:
        detail = shown.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            f"{path}: committed reader-evidence source is unavailable"
            + (f": {detail}" if detail else "")
        )
    try:
        source = as_object(
            json.loads(
                shown.stdout.decode("utf-8"),
                object_pairs_hook=reject_duplicate_keys,
            ),
            path,
        )
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReaderEvidenceError(
            f"{path}: committed reader-evidence source is invalid JSON"
        ) from exc
    return shown.stdout, source


def validation_source_bytes(source: dict[str, object]) -> bytes:
    """Use live bytes only for the exact parsed source; mark synthetic mutations."""
    current_source, current_raw = load_json(resolve(DEFAULT_SOURCE))
    if current_source == source:
        return current_raw
    encoded = json.dumps(
        source, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return b"\x00synthetic-reader-evidence\x00" + encoded


def history_predecessor_index(
    first_committed_raw: bytes,
    source_raw: bytes,
) -> int:
    """Select prior history by exact bytes, never parsed-object equality."""
    if not isinstance(source_raw, bytes):
        raise ReaderEvidenceError(
            "history_transition: exact source bytes are required"
        )
    return 1 if first_committed_raw == source_raw else 0


def nearest_previous_reader_evidence(
    source: dict[str, object],
    *,
    source_commit: str | None,
    source_raw: bytes | None = None,
) -> tuple[str, bytes, dict[str, object]] | None:
    """Resolve the nearest prior JSON-changing commit on first-parent history."""
    anchor = source_commit if source_commit is not None else "HEAD"
    try:
        history = subprocess.run(
            [
                "git",
                "-C",
                str(ROOT),
                "log",
                "--first-parent",
                "--format=%H",
                anchor,
                "--",
                DEFAULT_SOURCE.as_posix(),
            ],
            check=False,
            capture_output=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReaderEvidenceError(
            f"history_transition: cannot inspect first-parent source history: {exc}"
        ) from exc
    if history.returncode != 0:
        detail = history.stderr.decode("utf-8", errors="replace").strip()
        raise ReaderEvidenceError(
            "history_transition: cannot inspect first-parent source history"
            + (f": {detail}" if detail else "")
        )
    commits = [
        line
        for line in history.stdout.decode(
            "utf-8", errors="replace"
        ).splitlines()
        if COMMIT.fullmatch(line)
    ]
    if not commits:
        return None
    first_raw, _first_source = committed_reader_evidence(
        commits[0], "history_transition.current_committed_source"
    )
    if source_raw is None:
        source_raw = validation_source_bytes(source)
    predecessor_index = history_predecessor_index(
        first_raw, source_raw
    )
    if predecessor_index >= len(commits):
        return None
    commit = commits[predecessor_index]
    raw, previous_source = committed_reader_evidence(
        commit, "history_transition.previous_source"
    )
    return commit, raw, previous_source


def validate_history_stream_transition(
    name: str,
    previous_attempts: list[dict[str, object]],
    current_attempts: list[dict[str, object]],
) -> str:
    if len(current_attempts) < len(previous_attempts):
        raise ReaderEvidenceError(
            f"history_transition.{name}: prior attempt history must be prefix-preserved"
        )
    if len(current_attempts) > len(previous_attempts) + 1:
        raise ReaderEvidenceError(
            f"history_transition.{name}: only one successor may be appended per transition"
        )
    if len(current_attempts) == len(previous_attempts) + 1:
        if current_attempts[: len(previous_attempts)] != previous_attempts:
            raise ReaderEvidenceError(
                f"history_transition.{name}: prior attempt history must be prefix-preserved"
            )
        new_attempt = current_attempts[-1]
        expected_status = "not-run" if name == "pilot" else "frozen"
        if new_attempt["attempt_status"] != expected_status:
            raise ReaderEvidenceError(
                f"history_transition.{name}: a successor must begin {expected_status}"
            )
        return "append"

    differing = [
        index
        for index, previous in enumerate(previous_attempts)
        if previous != current_attempts[index]
    ]
    if not differing:
        return "unchanged"
    if differing != [len(previous_attempts) - 1]:
        raise ReaderEvidenceError(
            f"history_transition.{name}: terminal and superseded attempts are immutable"
        )
    previous = previous_attempts[-1]
    current = current_attempts[-1]
    nonterminal = "not-run" if name == "pilot" else "frozen"
    if (
        previous["attempt_status"] != nonterminal
        or current["attempt_status"] not in {"completed", "void"}
    ):
        raise ReaderEvidenceError(
            f"history_transition.{name}: only the active {nonterminal} attempt may become terminal"
        )
    immutable_keys = (
        {
            "attempt_id",
            "previous_attempt_sha256",
            "prerequisites",
            "pre_registration",
            "tested_snapshot",
        }
        if name == "pilot"
        else {
            "attempt_id",
            "previous_attempt_sha256",
            "pre_registration",
            "frozen_rule",
            "frozen_ratification",
        }
    )
    if any(previous[key] != current[key] for key in immutable_keys):
        raise ReaderEvidenceError(
            f"history_transition.{name}: frozen attempt identity and inputs are immutable"
        )
    return "terminal"




def validate_history_delta(
    previous_pilot: list[dict[str, object]],
    previous_holdout: list[dict[str, object]],
    current_pilot: list[dict[str, object]],
    current_holdout: list[dict[str, object]],
) -> None:
    pilot_action = validate_history_stream_transition(
        "pilot", previous_pilot, current_pilot
    )
    holdout_action = validate_history_stream_transition(
        "holdout", previous_holdout, current_holdout
    )
    if pilot_action != "unchanged" and holdout_action != "unchanged":
        raise ReaderEvidenceError(
            "history_transition: pilot and holdout histories may not change in one transition"
        )

def validate_history_transition(
    source: dict[str, object],
    *,
    source_commit: str | None = None,
    source_raw: bytes | None = None,
) -> None:
    transition = as_object(source["history_transition"], "history_transition")
    exact_keys(transition, HISTORY_TRANSITION_KEYS, "history_transition")
    (
        current_pilot,
        current_holdout,
        _current_pilot_digests,
        _current_holdout_digests,
        current_head,
    ) = validated_history_snapshot(source, "root")
    sha(
        transition["history_head_sha256"],
        "history_transition.history_head_sha256",
        current_head,
    )

    previous = nearest_previous_reader_evidence(
        source,
        source_commit=source_commit,
        source_raw=source_raw,
    )
    predecessor_values = (
        transition["previous_source_commit"],
        transition["previous_source_sha256"],
        transition["previous_history_head_sha256"],
    )
    if previous is None:
        if any(value is not None for value in predecessor_values):
            raise ReaderEvidenceError(
                "history_transition: bootstrap source must have null predecessor fields"
            )
        pilot = as_object(source["pilot"], "pilot")
        holdout = as_object(source["holdout"], "holdout")
        if (
            current_pilot
            or current_holdout
            or source["threshold_status"] != "pending-pilot"
            or source["holdout_status"] != "not-frozen"
            or source["result"] != "not-run"
            or source["ratification"] is not None
            or pilot["pilot_status"] != "not-run"
            or pilot["control_status"] != "not-run"
            or pilot["active_attempt_id"] is not None
            or holdout["active_attempt_id"] is not None
        ):
            raise ReaderEvidenceError(
                "history_transition: only the initial dormant empty source may bootstrap"
            )
        return

    expected_commit, previous_raw, previous_source = previous
    declared_commit = transition["previous_source_commit"]
    if (
        not isinstance(declared_commit, str)
        or not COMMIT.fullmatch(declared_commit)
        or declared_commit != expected_commit
    ):
        raise ReaderEvidenceError(
            "history_transition.previous_source_commit must cite the nearest prior JSON-changing commit"
        )
    sha(
        transition["previous_source_sha256"],
        "history_transition.previous_source_sha256",
        sha256_bytes(previous_raw),
    )
    exact_keys(previous_source, ROOT_KEYS, "history_transition.previous_source")
    previous_transition = as_object(
        previous_source["history_transition"],
        "history_transition.previous_source.history_transition",
    )
    exact_keys(
        previous_transition,
        HISTORY_TRANSITION_KEYS,
        "history_transition.previous_source.history_transition",
    )
    (
        previous_pilot,
        previous_holdout,
        _previous_pilot_digests,
        _previous_holdout_digests,
        previous_head,
    ) = validated_history_snapshot(
        previous_source, "history_transition.previous_source"
    )
    sha(
        previous_transition["history_head_sha256"],
        "history_transition.previous_source.history_head_sha256",
        previous_head,
    )
    sha(
        transition["previous_history_head_sha256"],
        "history_transition.previous_history_head_sha256",
        previous_head,
    )

    validate_history_delta(
        previous_pilot,
        previous_holdout,
        current_pilot,
        current_holdout,
    )

def validate_history_closure(
    source: dict[str, object],
    *,
    source_commit: str | None = None,
    source_raw: bytes | None = None,
) -> None:
    """Enforce cross-attempt uniqueness and strict append-only chronology."""
    global_ids: set[str] = set()
    attempt_digests: set[str] = set()
    preregistration_digests: set[str] = set()
    receipt_digests: set[str] = set()
    session_commitments: set[str] = set()
    custody_external_digests: set[str] = set()
    custody_record_digests: set[str] = set()
    gate_input_digests: set[str] = set()

    def unique(values: set[str], value: object, path: str) -> str:
        text = str(value)
        if text in values:
            raise ReaderEvidenceError(f"{path}: duplicate across attempt history")
        values.add(text)
        return text

    def unique_id(value: object, path: str) -> str:
        return unique(global_ids, opaque_id(value, path), path)

    def collect_common(attempt: dict[str, object], path: str) -> None:
        unique_id(attempt["attempt_id"], f"{path}.attempt_id")
        unique(
            attempt_digests,
            sha(attempt["attempt_sha256"], f"{path}.attempt_sha256"),
            f"{path}.attempt_sha256",
        )
        for index, raw in enumerate(
            as_list(attempt["session_records"], f"{path}.session_records")
        ):
            record = as_object(raw, f"{path}.session_records[{index}]")
            unique(
                session_commitments,
                sha(
                    record["record_commitment_sha256"],
                    f"{path}.session_records[{index}].record_commitment_sha256",
                ),
                f"{path}.session_records[{index}].record_commitment_sha256",
            )
        for index, raw in enumerate(
            as_list(
                attempt["custody_attestations"],
                f"{path}.custody_attestations",
            )
        ):
            record = as_object(raw, f"{path}.custody_attestations[{index}]")
            unique_id(
                record["attestation_id"],
                f"{path}.custody_attestations[{index}].attestation_id",
            )
            unique(
                custody_external_digests,
                sha(
                    record["sha256"],
                    f"{path}.custody_attestations[{index}].sha256",
                ),
                f"{path}.custody_attestations[{index}].sha256",
            )
            unique(
                custody_record_digests,
                sha(
                    record["record_sha256"],
                    f"{path}.custody_attestations[{index}].record_sha256",
                ),
                f"{path}.custody_attestations[{index}].record_sha256",
            )
        for index, raw in enumerate(
            as_list(attempt["deviations"], f"{path}.deviations")
        ):
            deviation = as_object(raw, f"{path}.deviations[{index}]")
            unique_id(
                deviation["deviation_id"],
                f"{path}.deviations[{index}].deviation_id",
            )

    previous_terminal: str | None = None
    pilot_attempts = as_list(source["pilot"]["attempts"], "pilot.attempts")
    for index, raw in enumerate(pilot_attempts):
        path = f"pilot.attempts[{index}]"
        attempt = as_object(raw, path)
        collect_common(attempt, path)
        registration = (
            as_object(attempt["pre_registration"], f"{path}.pre_registration")
            if attempt["pre_registration"] is not None
            else None
        )
        freeze_at = None
        if registration is not None:
            unique_id(
                registration["study_id"], f"{path}.pre_registration.study_id"
            )
            unique(
                preregistration_digests,
                sha(
                    registration["pre_registration_sha256"],
                    f"{path}.pre_registration.pre_registration_sha256",
                ),
                f"{path}.pre_registration.pre_registration_sha256",
            )
            binding = as_object(
                registration["freeze_binding"],
                f"{path}.pre_registration.freeze_binding",
            )
            unique_id(
                binding["binding_id"],
                f"{path}.pre_registration.freeze_binding.binding_id",
            )
            unique(
                custody_external_digests,
                sha(
                    binding["attestation_sha256"],
                    f"{path}.pre_registration.freeze_binding.attestation_sha256",
                ),
                f"{path}.pre_registration.freeze_binding.attestation_sha256",
            )
            freeze_at = str(binding["frozen_at"])
        if previous_terminal is not None:
            if freeze_at is None or freeze_at <= previous_terminal:
                raise ReaderEvidenceError(
                    f"{path}: successor freeze must strictly follow the prior pilot terminal time"
                )
        status = str(attempt["attempt_status"])
        if status == "completed":
            receipt = as_object(attempt["receipt"], f"{path}.receipt")
            unique_id(receipt["receipt_id"], f"{path}.receipt.receipt_id")
            unique(
                receipt_digests,
                sha(receipt["receipt_sha256"], f"{path}.receipt.receipt_sha256"),
                f"{path}.receipt.receipt_sha256",
            )
            packet = as_object(
                attempt["decision_packet"], f"{path}.decision_packet"
            )
            unique_id(packet["packet_id"], f"{path}.decision_packet.packet_id")
            packet_binding = as_object(
                packet["freeze_binding"],
                f"{path}.decision_packet.freeze_binding",
            )
            unique_id(
                packet_binding["binding_id"],
                f"{path}.decision_packet.freeze_binding.binding_id",
            )
            unique(
                custody_external_digests,
                sha(
                    packet_binding["attestation_sha256"],
                    f"{path}.decision_packet.freeze_binding.attestation_sha256",
                ),
                f"{path}.decision_packet.freeze_binding.attestation_sha256",
            )
            previous_terminal = str(packet_binding["frozen_at"])
        elif status == "void":
            previous_terminal = str(attempt["voided_at"])

    previous_terminal = None
    holdout_attempts = as_list(
        source["holdout"]["attempts"], "holdout.attempts"
    )
    for index, raw in enumerate(holdout_attempts):
        path = f"holdout.attempts[{index}]"
        attempt = as_object(raw, path)
        collect_common(attempt, path)
        registration = as_object(
            attempt["pre_registration"], f"{path}.pre_registration"
        )
        unique_id(
            registration["study_id"], f"{path}.pre_registration.study_id"
        )
        unique(
            preregistration_digests,
            sha(
                registration["pre_registration_sha256"],
                f"{path}.pre_registration.pre_registration_sha256",
            ),
            f"{path}.pre_registration.pre_registration_sha256",
        )
        binding = as_object(
            registration["freeze_binding"],
            f"{path}.pre_registration.freeze_binding",
        )
        unique_id(
            binding["binding_id"],
            f"{path}.pre_registration.freeze_binding.binding_id",
        )
        unique(
            custody_external_digests,
            sha(
                binding["attestation_sha256"],
                f"{path}.pre_registration.freeze_binding.attestation_sha256",
            ),
            f"{path}.pre_registration.freeze_binding.attestation_sha256",
        )
        freeze_at = str(binding["frozen_at"])
        if previous_terminal is not None and freeze_at <= previous_terminal:
            raise ReaderEvidenceError(
                f"{path}: successor freeze must strictly follow the prior holdout terminal time"
            )
        commitment = registration["commitment"]
        if commitment is not None:
            commitment_record = as_object(
                commitment, f"{path}.pre_registration.commitment"
            )
            unique_id(
                commitment_record["commitment_id"],
                f"{path}.pre_registration.commitment.commitment_id",
            )
        status = str(attempt["attempt_status"])
        if status == "completed":
            receipt = as_object(attempt["result_receipt"], f"{path}.result_receipt")
            unique_id(
                receipt["receipt_id"], f"{path}.result_receipt.receipt_id"
            )
            unique(
                receipt_digests,
                sha(
                    receipt["receipt_sha256"],
                    f"{path}.result_receipt.receipt_sha256",
                ),
                f"{path}.result_receipt.receipt_sha256",
            )
            gate_receipt = as_object(
                attempt["gate_admission_receipt"],
                f"{path}.gate_admission_receipt",
            )
            unique(
                gate_input_digests,
                sha(
                    gate_receipt["input_sha256"],
                    f"{path}.gate_admission_receipt.input_sha256",
                ),
                f"{path}.gate_admission_receipt.input_sha256",
            )
            unique(
                receipt_digests,
                sha(
                    gate_receipt["receipt_sha256"],
                    f"{path}.gate_admission_receipt.receipt_sha256",
                ),
                f"{path}.gate_admission_receipt.receipt_sha256",
            )
            previous_terminal = str(receipt["completed_at"])
        elif status == "void":
            if attempt["result_receipt"] is not None:
                receipt = as_object(
                    attempt["result_receipt"], f"{path}.result_receipt"
                )
                unique_id(
                    receipt["receipt_id"],
                    f"{path}.result_receipt.receipt_id",
                )
                unique(
                    receipt_digests,
                    sha(
                        receipt["receipt_sha256"],
                        f"{path}.result_receipt.receipt_sha256",
                    ),
                    f"{path}.result_receipt.receipt_sha256",
                )
            previous_terminal = str(attempt["voided_at"])
        if attempt["commitment_reveal"] is not None:
            reveal = as_object(
                attempt["commitment_reveal"],
                f"{path}.commitment_reveal",
            )
            previous_terminal = str(reveal["revealed_at"])

    validate_history_transition(
        source,
        source_commit=source_commit,
        source_raw=source_raw,
    )




def validate_acceptance(source: dict[str, object]) -> None:
    acceptance = as_object(source["acceptance"], "acceptance")
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance")
    if acceptance["gate_c_satisfied"] is not False:
        raise ReaderEvidenceError("reader evidence alone may never satisfy Gate C")
    if acceptance["permitted_claim"] != "none":
        raise ReaderEvidenceError("this contract may not rewrite Gate C's permitted claim")
    text_list(acceptance["limits"], "acceptance.limits")


def validate(
    source: dict[str, object],
    *,
    source_raw: bytes | None = None,
) -> tuple[bool, bool]:
    exact_keys(source, ROOT_KEYS, "root")
    walk_keys(source)
    validate_protocol(source)
    (
        valid_pilot,
        pilot_packet,
        sensitivity_brief,
        pilot_attempt_id,
    ) = validate_pilot(source)
    validate_privacy(source)
    rule, known_misconceptions = validate_threshold_rule(source, valid_pilot)
    validate_ratification(
        source,
        rule,
        pilot_packet,
        sensitivity_brief,
        pilot_attempt_id,
    )
    (
        route_status,
        evidence_gate_sha256,
        structural_checker_sha256,
    ) = validate_route_readiness(
        source,
        valid_pilot,
    )
    valid_holdout_pass = validate_holdout(
        source,
        rule,
        known_misconceptions,
        route_status,
        evidence_gate_sha256,
        structural_checker_sha256,
    )
    validate_history_closure(source, source_raw=source_raw)
    validate_claim(source, route_status, valid_holdout_pass)
    validate_acceptance(source)
    return valid_pilot, valid_holdout_pass


def escape(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def code(value: object) -> str:
    return f"`{str(value).replace('`', '')}`"


def render(source: dict[str, object], source_digest: str) -> str:
    route = as_object(source["route"], "route")
    pilot = as_object(source["pilot"], "pilot")
    rule = as_object(source["threshold_rule"], "threshold_rule")
    privacy = as_object(source["privacy"], "privacy")
    protocol = as_object(source["protocol"], "protocol")
    acceptance = as_object(source["acceptance"], "acceptance")
    history_transition = as_object(
        source["history_transition"], "history_transition"
    )
    dormant = (
        source["threshold_status"] == "pending-pilot"
        and pilot["pilot_status"] == "not-run"
        and source["holdout_status"] == "not-frozen"
        and source["result"] == "not-run"
    )
    banner = (
        "**DORMANT PRE-PILOT CONTRACT: no reader result and no release threshold.**"
        if dormant
        else "**REVIEWED READER-EVIDENCE STATE: bounded by the statuses below.**"
    )
    intro_lines = (
        [
            "This report renders the reviewed evidence contract. It does not run a",
            "reader study, ratify a taxonomy or value, make R6 available, establish",
            "FS-CLM-37, or satisfy Gate C.",
        ]
        if dormant
        else [
            "This report renders the current reviewed evidence state. Rendering does",
            "not itself run a reader study, admit evidence, or satisfy Gate C.",
        ]
    )
    state_note = (
        [
            "The threshold fields are empty by design. No pilot receipt, author",
            "ratification, holdout pre-registration, session record, or result receipt",
            "is present.",
        ]
        if dormant
        else [
            "The canonical machine source owns the exact artifacts and current state.",
        ]
    )
    route_note = {
        "unbuilt": [
            "R6 remains unbuilt because its availability tuple is incomplete.",
            "Structural checker controls do not substitute for the seeded pilot control",
            "or any missing external admission component.",
        ],
        "built": [
            "R6 is built but not available to admit holdout evidence. The remaining",
            "availability requirements must be satisfied before evidence can be admitted.",
        ],
        "available": [
            "R6 is available for a matching holdout under the bound admission route.",
            "Availability alone does not admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
    }[route["route_status"]]
    threshold_note = {
        "pending-pilot": [
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. Exact severity labels, classification",
            "boundaries, core mappings, policies, and threshold values remain absent",
            "until a valid pilot supplies the basis for a candidate rule.",
        ],
        "candidate": [
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are a post-pilot candidate only; they are not author-ratified",
            "and cannot govern a holdout.",
        ],
        "author-ratified": [
            "A core misconception cannot be offset, averaged away, or outvoted by",
            "favourable outcomes elsewhere. The generated taxonomy, mappings, policies,",
            "and values are author-ratified. Ratification alone does not make R6",
            "available, admit evidence, establish FS-CLM-37, or satisfy Gate C.",
        ],
    }[source["threshold_status"]]
    lines = [
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->",
        "<!-- Generated by new-book-plans/14-reader-evidence.py from reader-evidence.json. Do not edit. -->",
        "",
        "# Reader Evidence Contract",
        "",
        banner,
        "",
        *intro_lines,
        "",
        "## Current state",
        "",
        "| field | value |",
        "| --- | --- |",
        f"| Threshold | {code(source['threshold_status'])} |",
        f"| Pilot | {code(pilot['pilot_status'])} |",
        f"| Seeded instrument control | {code(pilot['control_status'])} |",
        f"| Reader route | {code(route['route_status'])} |",
        f"| Holdout | {code(source['holdout_status'])} |",
        f"| Result | {code(source['result'])} |",
        f"| History head | {code(history_transition['history_head_sha256'])} |",
        f"| Prior source transition | {code(history_transition['previous_source_commit'] or 'initial-bootstrap')} |",
        f"| Claim | {code(source['claim']['posture'])}/{code(source['claim']['disposition'])} |",
        f"| Gate C satisfied here | {code(str(acceptance['gate_c_satisfied']).lower())} |",
        "",
        *state_note,
        "",
        "Attempt histories are prefix-preserved against the nearest prior",
        "JSON-changing commit; every frozen preregistration binds its predecessor",
        "attempt and prior two-stream history head. This is a repository-relative",
        "commitment, not proof against rewritten Git or external custody history.",
        "",
        "## Route availability",
        "",
        "| component | recorded state |",
        "| --- | --- |",
        f"| Structural checker | {code('bound' if route['structural_checker_binding'] else 'absent')} |",
        f"| Evidence contract | {code(route['evidence_contract_status'])} |",
        f"| Threshold rule | {code(source['threshold_status'])} |",
        f"| Named reviewer custody binding | {code('present' if route['reviewer_custody_attestation'] else 'absent')} |",
        f"| Evidence-admission gate route binding | {code('present' if route['evidence_admission_gate_binding'] else 'absent')} |",
        f"| Instrument control watched failing | {code(route['negative_control_status'])} |",
        "",
        *route_note,
        "",
        "## Fixed pass-rule form",
        "",
        "Evaluation order: " + " → ".join(code(item) for item in EVALUATION_ORDER) + ".",
        "",
        *threshold_note,
        "",
        "### Minimum identification targets",
        "",
    ]
    for item in protocol["required_targets"]:
        lines.append(f"- {escape(item['description'])} ({code(item['target_id'])})")
    lines.extend([
        "",
        "## Privacy boundary",
        "",
        f"Public policy: {code(privacy['public_record_policy'])}.",
        "",
        "The public source may hold only opaque study identifiers, coded target and",
        "misconception outcomes, artifact and commitment digests, coded deviations,",
        "and custody attestations without identity material. Everything below is private:",
        "",
    ])
    for item in privacy["excluded_from_repository"]:
        lines.append(f"- {escape(item)}")
    lines.extend([
        "",
        escape(privacy["freshness_attestation_boundary"]),
        "",
        "## What still does not follow",
        "",
    ])
    for item in acceptance["limits"]:
        lines.append(f"- {escape(item)}")
    if source["threshold_status"] != "pending-pilot":
        projection = (
            "Ratified"
            if source["threshold_status"] == "author-ratified"
            else "Candidate"
        )
        lines.extend([
            "",
            f"## {projection} threshold projection",
            "",
            f"Rule: {code(rule['rule_id'])}; SHA-256 {code(rule['rule_sha256'])}.",
            "",
            "Every value below is generated from the canonical machine source.",
            "",
            "### Severity taxonomy and misconception mapping",
            "",
        ])
        for item in rule["severity_taxonomy"]:
            lines.append(
                f"- {code(item['severity_id'])}: {escape(item['label'])} - "
                f"{escape(item['definition'])} Boundary: "
                f"{escape(item['classification_boundary'])}"
            )
        for item in rule["misconceptions"]:
            lines.append(
                f"- {code(item['misconception_id'])} -> "
                f"{code(item['severity_id'])}; core "
                f"{code(str(item['core']).lower())}. "
                f"{escape(item['definition'])}"
            )

        def append_threshold(
            label: str, raw_spec: object
        ) -> None:
            spec = as_object(raw_spec, label)
            lines.append(
                f"- {escape(label)}: {code(spec['threshold_id'])}; "
                f"{code(spec['metric'])} {code(spec['operator'])} "
                f"{code(spec['value'])} {escape(spec['unit'])}; "
                f"denominator {code(spec['denominator'])}; scope "
                + ", ".join(code(item) for item in spec["scope_refs"])
                + "."
            )

        lines.extend([
            "",
            "### Complete deterministic rule",
            "",
            f"- Core branch: {code(rule['core_failure_mode'])}; repetition "
            f"unit {code(rule['repetition_unit'])}; denominator "
            f"{code(rule['denominator'])}.",
            f"- Evaluation order: "
            + " -> ".join(code(item) for item in rule["evaluation_order"])
            + ".",
            f"- Aggregate offset prohibited: "
            f"{code(str(rule['aggregate_offset_prohibited']).lower())}.",
        ])
        append_threshold(
            "Core veto", rule["core_failure_threshold"]
        )
        append_threshold(
            "Minimum evaluable evidence",
            rule["minimum_evaluable_evidence"],
        )
        for item in rule["required_target_thresholds"]:
            append_threshold(
                f"Required target {item['target_id']}",
                item["threshold"],
            )
        for item in rule["non_core_thresholds"]:
            append_threshold(
                f"Non-core severity {item['severity_id']}",
                item["threshold"],
            )
        lines.extend(["", "Policies:", ""])
        for key, value in rule["policies"].items():
            lines.append(f"- {code(key)}: {code(value)}")

        if source["ratification"] is not None:
            ratification = as_object(
                source["ratification"], "ratification"
            )
            lines.extend([
                "",
                "### Author ratification basis",
                "",
                f"- Ruling: {code(ratification['ruling_id'])}; date "
                f"{code(ratification['ratified_date'])}; candidate commit "
                f"{code(ratification['candidate_commit'])}.",
                f"- Pilot attempt {code(ratification['pilot_attempt_id'])}; "
                f"packet {code(ratification['pilot_packet_sha256'])}; "
                f"sensitivity brief "
                f"{code(ratification['sensitivity_brief_sha256'])}.",
                f"- Rule digest {code(ratification['rule_sha256'])}; "
                f"ratification receipt "
                f"{code(ratification['ratification_sha256'])}.",
                f"- Decision record: {code(ratification['decision_ref'])}.",
                f"- No holdout evidence existed or was inspected: "
                f"{code(str(ratification['no_holdout_evidence_attestation']).lower())}.",
            ])

        holdout = as_object(source["holdout"], "holdout")
        lines.extend([
            "",
            "### Append-only holdout history",
            "",
            f"Active attempt: {code(holdout['active_attempt_id'])}; "
            f"lifecycle {code(source['holdout_status'])}; latest completed "
            f"non-void result {code(source['result'])}.",
        ])
        for attempt in holdout["attempts"]:
            lines.append(
                f"- {code(attempt['attempt_id'])}: "
                f"{code(attempt['attempt_status'])}/"
                f"{code(attempt['attempt_result'])}; frozen rule "
                f"{code(attempt['frozen_rule']['rule_sha256'])}; attempt "
                f"{code(attempt['attempt_sha256'])}."
            )
    lines.extend([
        "",
        "## Provenance and verification",
        "",
        f"- Source: {code(DEFAULT_SOURCE.as_posix())}, SHA-256 {code(source_digest)}.",
        f"- Controlling decision: {code(source['protocol_decision_ref'])}, SHA-256 {code(protocol['decision_sha256'])}.",
        "- Structural/freshness check: `python3 new-book-plans/14-reader-evidence.py --check`.",
        "- Executable contract controls: `python3 new-book-plans/14-reader-evidence.py --check --execute`.",
        "",
    ])
    return "\n".join(lines)


def expect_failure(label: str, action, expected: str | None = None) -> None:
    try:
        action()
    except ReaderEvidenceError as exc:
        if expected is not None and expected not in str(exc):
            raise ReaderEvidenceError(
                f"negative control failed for the wrong reason: {label}: {exc}"
            ) from exc
        return
    raise ReaderEvidenceError(f"negative control did not fail: {label}")


def structural_controls(
    source: dict[str, object],
    *,
    source_raw: bytes | None = None,
) -> int:
    controls: list[tuple[str, object, str | None]] = []

    def add(label, mutate, expected=None):
        controls.append((label, mutate, expected))

    def set_threshold_value(
        value: object,
        threshold_id: str,
        new_value: str,
    ) -> bool:
        if isinstance(value, dict):
            if value.get("threshold_id") == threshold_id:
                value["value"] = new_value
                return True
            return any(
                set_threshold_value(child, threshold_id, new_value)
                for child in value.values()
            )
        if isinstance(value, list):
            return any(
                set_threshold_value(child, threshold_id, new_value)
                for child in value
            )
        return False


    add("unknown root key", lambda s: s.update({"extra": None}), "unexpected")
    add("boolean schema version", lambda s: s.update({"schema_version": True}), "integer 1")
    add("history head digest drift", lambda s: s["history_transition"].update({"history_head_sha256": "0" * 64}), "stale")

    add("unknown threshold status", lambda s: s.update({"threshold_status": "settled"}), "expected one of")
    add("unknown holdout status", lambda s: s.update({"holdout_status": "running"}), "expected one of")
    add("unknown result", lambda s: s.update({"result": "success"}), "expected one of")
    add("stale protocol digest", lambda s: s["protocol"].update({"decision_sha256": "0" * 64}), "stale")
    add("disclosed limits drift", lambda s: s["protocol"]["disclosed_limits"].append("unratified limit"), "drifted")
    add("ethics terms drift", lambda s: s["protocol"]["ethics_terms"].reverse(), "drifted")
    add("freshness terms drift", lambda s: s["protocol"]["freshness_terms"].pop(), "drifted")
    add("non-substitution drift", lambda s: s["protocol"].update({"non_substitution": "reader evidence substitutes"}), "drifted")
    add("public record kinds drift", lambda s: s["privacy"]["allowed_public_record_kinds"].append("identity mapping"), "drifted")
    add("broken protocol anchor", lambda s: s.update({"protocol_decision_ref": str(PROTOCOL_DECISION) + "::missing-anchor"}), "exactly once")
    add("aggregate score field", lambda s: s["threshold_rule"].update({"aggregate_score": None}), "scoring fields")
    add("private raw-response field", lambda s: s["holdout"].update({"raw_responses": []}), "outside the repository")
    add("Gate C overclaim", lambda s: s["acceptance"].update({"gate_c_satisfied": True}), "never satisfy")
    add("aggregate veto removed", lambda s: s["threshold_rule"].update({"aggregate_offset_prohibited": False}), "preserve")
    add("evaluation order changed", lambda s: s["threshold_rule"]["evaluation_order"].reverse(), "fixed order")
    add("structural checker digest drift", lambda s: s["route"]["structural_checker_binding"].update({"sha256": "0" * 64}), "stale")
    add("implemented contract hidden", lambda s: s["route"].update({"evidence_contract_status": "unbuilt"}), "must record")
    add(
        "route control diverges from pilot",
        lambda s: s["route"].update({
            "negative_control_status": (
                "indeterminate"
                if s["pilot"]["control_status"] != "indeterminate"
                else "failed-to-fail"
            )
        }),
        "must equal",
    )
    for policy_key in sorted(POLICY_KEYS):
        add(
            f"threshold policy key retained: {policy_key}",
            lambda s, policy_key=policy_key: s["threshold_rule"]["policies"].pop(policy_key),
            "missing",
        )

    pilot_attempts = as_list(source["pilot"]["attempts"], "pilot.attempts")
    holdout_attempts = as_list(source["holdout"]["attempts"], "holdout.attempts")
    dormant = (
        source["threshold_status"] == "pending-pilot"
        and not pilot_attempts
        and not holdout_attempts
    )
    direct_controls = 0
    committed_probe = b'{"value":1}\n'
    byte_only_probe = b'{ "value": 1 }\n'
    if json.loads(committed_probe) != json.loads(byte_only_probe):
        raise ReaderEvidenceError(
            "byte-only predecessor selector control fixture is invalid"
        )
    before_commit = [("prior", committed_probe)]
    after_commit = [
        ("byte-only", byte_only_probe),
        ("prior", committed_probe),
    ]
    before_choice = before_commit[
        history_predecessor_index(before_commit[0][1], byte_only_probe)
    ][0]
    after_choice = after_commit[
        history_predecessor_index(after_commit[0][1], byte_only_probe)
    ][0]
    if before_choice != "prior" or after_choice != "prior":
        raise ReaderEvidenceError(
            "byte-only predecessor selector changed across commit boundary"
        )
    direct_controls += 1
    if dormant:
        add("threshold content before valid pilot", lambda s: s["threshold_rule"].update({"rule_id": s["contract_id"]}), "prohibited until")
        add("severity entry before valid pilot", lambda s: s["threshold_rule"]["severity_taxonomy"].append({}), "prohibited until")
        add("candidate status before valid pilot", lambda s: s.update({"threshold_status": "candidate"}), "prohibited until")
        add("ratification before valid pilot", lambda s: s.update({"ratification": {}}), "prohibited")
        add("pilot summary without attempt", lambda s: s["pilot"].update({"pilot_status": "completed"}), "empty pilot history")
        add("control result without pilot", lambda s: s["pilot"].update({"control_status": "watched-failing"}), "empty pilot history")
        add("holdout lifecycle without attempt", lambda s: s.update({"holdout_status": "frozen"}), "empty holdout history")
        add("result without completed attempt", lambda s: s.update({"result": "pass"}), "empty holdout history")
        add("route availability without components", lambda s: s["route"].update({"route_status": "available"}), "must be unbuilt")
        add("claim evidence pending on unbuilt route", lambda s: s["claim"].update({"disposition": "evidence-pending"}), "route-unbuilt")
        add("claim evidenced without a pass", lambda s: s["claim"].update({"posture": "Evidenced", "disposition": "none"}), "Unestablished")
        add("bootstrap predecessor injected", lambda s: s["history_transition"].update({"previous_source_commit": "0" * 40}), "null predecessor")
    else:
        transition_record = as_object(
            source["history_transition"], "history_transition"
        )
        if transition_record["previous_source_commit"] is not None:
            add(
                "history predecessor commit drift",
                lambda s: s["history_transition"].update(
                    {"previous_source_commit": "0" * 40}
                ),
                "nearest prior JSON-changing commit",
            )
            add(
                "history predecessor source digest drift",
                lambda s: s["history_transition"].update(
                    {"previous_source_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "history predecessor head drift",
                lambda s: s["history_transition"].update(
                    {"previous_history_head_sha256": "0" * 64}
                ),
                "stale",
            )

        if pilot_attempts:
            expect_failure(
                "pilot history deletion/reset",
                lambda: validate_history_stream_transition(
                    "pilot", pilot_attempts, pilot_attempts[:-1]
                ),
                "prefix-preserved",
            )
            direct_controls += 1
            terminal_index = next(
                (
                    index
                    for index in range(len(pilot_attempts) - 1, -1, -1)
                    if pilot_attempts[index]["attempt_status"]
                    in {"completed", "void"}
                ),
                None,
            )
            if terminal_index is not None:
                changed_pilot = copy.deepcopy(pilot_attempts)
                changed_pilot[terminal_index]["void_reason_code"] = (
                    "RE-VOID-HISTORY-MUTATION"
                )
                expect_failure(
                    "pilot terminal history mutation",
                    lambda changed_pilot=changed_pilot: validate_history_stream_transition(
                        "pilot", pilot_attempts, changed_pilot
                    ),
                    "only the active",
                )
                direct_controls += 1

        if holdout_attempts:
            expect_failure(
                "holdout history deletion/reset",
                lambda: validate_history_stream_transition(
                    "holdout", holdout_attempts, holdout_attempts[:-1]
                ),
                "prefix-preserved",
            )
            direct_controls += 1
            terminal_index = next(
                (
                    index
                    for index in range(len(holdout_attempts) - 1, -1, -1)
                    if holdout_attempts[index]["attempt_status"]
                    in {"completed", "void"}
                ),
                None,
            )
            if terminal_index is not None:
                changed_holdout = copy.deepcopy(holdout_attempts)
                changed_holdout[terminal_index]["void_reason_code"] = (
                    "RE-VOID-HISTORY-MUTATION"
                )
                expect_failure(
                    "holdout terminal history mutation",
                    lambda changed_holdout=changed_holdout: validate_history_stream_transition(
                        "holdout", holdout_attempts, changed_holdout
                    ),
                    "only the active",
                )
                direct_controls += 1

        if (
            pilot_attempts
            and holdout_attempts
            and pilot_attempts[-1]["attempt_status"] in {"completed", "void"}
            and holdout_attempts[-1]["attempt_status"] in {"completed", "void"}
        ):
            previous_pilot = copy.deepcopy(pilot_attempts)
            previous_holdout = copy.deepcopy(holdout_attempts)
            previous_pilot[-1]["attempt_status"] = "not-run"
            previous_holdout[-1]["attempt_status"] = "frozen"
            expect_failure(
                "combined pilot and holdout history transition",
                lambda: validate_history_delta(
                    previous_pilot,
                    previous_holdout,
                    pilot_attempts,
                    holdout_attempts,
                ),
                "may not change in one transition",
            )
            direct_controls += 1

        if pilot_attempts:
            add("pilot active pointer drift", lambda s: s["pilot"].update({"active_attempt_id": "RE-PILOT-NOT-ACTIVE"}), "final append-only attempt")
            add("pilot attempt digest drift", lambda s: s["pilot"]["attempts"][-1].update({"attempt_sha256": "0" * 64}), "stale")
        if holdout_attempts:
            add("holdout active pointer drift", lambda s: s["holdout"].update({"active_attempt_id": "RE-HOLDOUT-NOT-ACTIVE"}), "final append-only attempt")
            add("holdout attempt digest drift", lambda s: s["holdout"]["attempts"][-1].update({"attempt_sha256": "0" * 64}), "stale")
        if source["threshold_status"] != "pending-pilot":
            add(
                "populated rule digest drift",
                lambda s: s["threshold_rule"].update(
                    {"rule_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "count threshold endpoint rejected",
                lambda s: s["threshold_rule"][
                    "minimum_evaluable_evidence"
                ].update({"value": "0"}),
                "below, exact, and above cases",
            )
            control_rule = as_object(
                source["threshold_rule"], "threshold_rule"
            )
            control_specs = [
                as_object(
                    control_rule["core_failure_threshold"],
                    "control core threshold",
                ),
                as_object(
                    control_rule["minimum_evaluable_evidence"],
                    "control minimum threshold",
                ),
            ]
            control_specs.extend(
                as_object(item, "control target threshold")["threshold"]
                for item in as_list(
                    control_rule["required_target_thresholds"],
                    "control target thresholds",
                )
            )
            control_specs.extend(
                as_object(item, "control severity threshold")["threshold"]
                for item in as_list(
                    control_rule["non_core_thresholds"],
                    "control non-core thresholds",
                )
            )
            rate_spec = next(
                (
                    as_object(item, "control numeric threshold")
                    for item in control_specs
                    if str(
                        as_object(item, "control numeric threshold")[
                            "metric"
                        ]
                    ).endswith("-rate")
                ),
                None,
            )
            if rate_spec is not None:
                rate_threshold_id = str(rate_spec["threshold_id"])
                add(
                    "rate threshold endpoint rejected",
                    lambda s, threshold_id=rate_threshold_id: set_threshold_value(
                        s["threshold_rule"], threshold_id, "1"
                    ),
                    "below, exact, and above cases",
                )

        if source["ratification"] is not None:
            add(
                "author ratification receipt drift",
                lambda s: s["ratification"].update(
                    {"ratification_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "candidate ancestry drift",
                lambda s: s["ratification"].update(
                    {"candidate_commit": "0" * 40}
                ),
                "ancestor",
            )
            candidate_record = as_object(
                source["ratification"], "ratification"
            )
            candidate_commit = str(candidate_record["candidate_commit"])
            candidate_control_raw, candidate_control_source = (
                committed_reader_evidence(
                    candidate_commit,
                    "candidate control source",
                )
            )
            candidate_control_decision = committed_file_bytes(
                candidate_commit,
                PROTOCOL_DECISION,
                "candidate control protocol decision",
            )
            candidate_control_checker = committed_file_bytes(
                candidate_commit,
                pathlib.Path(
                    STRUCTURAL_CHECKER_REF.split("::", 1)[0]
                ),
                "candidate control structural checker",
            )
            validate_candidate_relevant_state(
                candidate_control_source,
                candidate_commit=candidate_commit,
                decision_bytes=candidate_control_decision,
                checker_bytes=candidate_control_checker,
                candidate_raw=candidate_control_raw,
                valid_pilot=True,
            )
            direct_controls += 1

            candidate_mutations = [
                (
                    "candidate protocol decision digest drift",
                    lambda s: s["protocol"].update(
                        {"decision_sha256": "0" * 64}
                    ),
                    "stale",
                ),
                (
                    "candidate Gate C overclaim",
                    lambda s: s["acceptance"].update(
                        {"gate_c_satisfied": True}
                    ),
                    "never satisfy",
                ),
                (
                    "candidate route overclaim",
                    lambda s: s["route"].update(
                        {"route_status": "available"}
                    ),
                    "must be unbuilt",
                ),
                (
                    "candidate claim overclaim",
                    lambda s: s["claim"].update(
                        {"posture": "Evidenced", "disposition": "none"}
                    ),
                    "Unestablished/route-unbuilt",
                ),
                (
                    "candidate history transition drift",
                    lambda s: s["history_transition"].update(
                        {"history_head_sha256": "0" * 64}
                    ),
                    "stale",
                ),
            ]
            for label, mutate, expected in candidate_mutations:
                changed_candidate = copy.deepcopy(
                    candidate_control_source
                )
                mutate(changed_candidate)
                expect_failure(
                    label,
                    lambda changed_candidate=changed_candidate: validate_candidate_relevant_state(
                        changed_candidate,
                        candidate_commit=candidate_commit,
                        decision_bytes=candidate_control_decision,
                        checker_bytes=candidate_control_checker,
                        valid_pilot=True,
                        candidate_raw=candidate_control_raw,
                    ),
                    expected,
                )
                direct_controls += 1


        if pilot_attempts:
            pilot_prereg = next(
                (
                    index
                    for index in range(len(pilot_attempts) - 1, -1, -1)
                    if pilot_attempts[index]["pre_registration"] is not None
                ),
                None,
            )
            if pilot_prereg is not None:
                pilot_predecessor_error = (
                    "first attempt must be null"
                    if pilot_prereg == 0
                    else "stale"
                )
                add(
                    "pilot prereg predecessor mismatch",
                    lambda s, index=pilot_prereg: s["pilot"]["attempts"][
                        index
                    ]["pre_registration"].update(
                        {"predecessor_attempt_sha256": "0" * 64}
                    ),
                    pilot_predecessor_error,
                )
                add(
                    "pilot prereg prior history head mismatch",
                    lambda s, index=pilot_prereg: s["pilot"]["attempts"][
                        index
                    ]["pre_registration"].update(
                        {"prior_history_head_sha256": "0" * 64}
                    ),
                    "stale",
                )

                add(
                    "pilot freeze payload drift",
                    lambda s, index=pilot_prereg: s["pilot"]["attempts"][
                        index
                    ]["pre_registration"]["freeze_binding"].update(
                        {"bound_payload_sha256": "0" * 64}
                    ),
                    "stale",
                )
                add(
                    "pilot attested payload drift",
                    lambda s, index=pilot_prereg: s["pilot"]["attempts"][
                        index
                    ]["pre_registration"]["freeze_binding"].update(
                        {"attested_payload_sha256": "0" * 64}
                    ),
                    "stale",
                )
            pilot_packet = next(
                (
                    index
                    for index in range(len(pilot_attempts) - 1, -1, -1)
                    if pilot_attempts[index]["decision_packet"] is not None
                ),
                None,
            )
            if pilot_packet is not None:
                add(
                    "pilot packet freeze date drift",
                    lambda s, index=pilot_packet: s["pilot"]["attempts"][
                        index
                    ]["decision_packet"].update(
                        {"frozen_date": "1970-01-01"}
                    ),
                    "must equal",
                )
            pilot_receipt = next(
                (
                    index
                    for index in range(len(pilot_attempts) - 1, -1, -1)
                    if pilot_attempts[index]["receipt"] is not None
                ),
                None,
            )
            if pilot_receipt is not None:
                add(
                    "pilot completion chronology drift",
                    lambda s, index=pilot_receipt: s["pilot"]["attempts"][
                        index
                    ]["receipt"].update(
                        {
                            "completed_at": s["pilot"]["attempts"][index][
                                "pre_registration"
                            ]["freeze_binding"]["frozen_at"]
                        }
                    ),
                    "stale",
                )
            pilot_custody = next(
                (
                    index
                    for index, attempt in enumerate(pilot_attempts)
                    if attempt["custody_attestations"]
                ),
                None,
            )
            if pilot_custody is not None:
                add(
                    "pilot custody record duplication",
                    lambda s, index=pilot_custody: s["pilot"]["attempts"][
                        index
                    ]["custody_attestations"].append(
                        copy.deepcopy(
                            s["pilot"]["attempts"][index][
                                "custody_attestations"
                            ][0]
                        )
                    ),
                    "duplicate",
                )

        if holdout_attempts:
            active_index = len(holdout_attempts) - 1
            holdout_predecessor_error = (
                "first attempt must be null"
                if active_index == 0
                else "stale"
            )
            add(
                "holdout prereg predecessor mismatch",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["pre_registration"].update(
                    {"predecessor_attempt_sha256": "0" * 64}
                ),
                holdout_predecessor_error,
            )
            add(
                "holdout prereg prior history head mismatch",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["pre_registration"].update(
                    {"prior_history_head_sha256": "0" * 64}
                ),
                "stale",
            )

            add(
                "frozen holdout rule digest drift",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["frozen_rule"].update({"rule_sha256": "0" * 64}),
                "stale",
            )
            add(
                "frozen ratification rule drift",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["frozen_ratification"].update(
                    {"rule_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "holdout freeze payload drift",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["pre_registration"]["freeze_binding"].update(
                    {"bound_payload_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "holdout attested payload drift",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["pre_registration"]["freeze_binding"].update(
                    {"attested_payload_sha256": "0" * 64}
                ),
                "stale",
            )
            add(
                "holdout checker dependency drift",
                lambda s, index=active_index: s["holdout"]["attempts"][
                    index
                ]["pre_registration"].update(
                    {"structural_checker_sha256": "0" * 64}
                ),
            )
            holdout_custody = next(
                (
                    index
                    for index, attempt in enumerate(holdout_attempts)
                    if attempt["custody_attestations"]
                ),
                None,
            )
            if holdout_custody is not None:
                add(
                    "holdout custody record duplication",
                    lambda s, index=holdout_custody: s["holdout"][
                        "attempts"
                    ][index]["custody_attestations"].append(
                        copy.deepcopy(
                            s["holdout"]["attempts"][index][
                                "custody_attestations"
                            ][0]
                        )
                    ),
                    "duplicate",
                )
                freshness_index = next(
                    (
                        item_index
                        for item_index, item in enumerate(
                            holdout_attempts[holdout_custody][
                                "custody_attestations"
                            ]
                        )
                        if item["scope"] == "study-freshness"
                    ),
                    None,
                )
                if freshness_index is not None:
                    add(
                        "holdout freshness custody removed",
                        lambda s, attempt_index=holdout_custody, item_index=freshness_index: s[
                            "holdout"
                        ]["attempts"][attempt_index][
                            "custody_attestations"
                        ].pop(item_index),
                    )
            receipt_index = next(
                (
                    index
                    for index, attempt in enumerate(holdout_attempts)
                    if attempt["result_receipt"] is not None
                ),
                None,
            )
            if receipt_index is not None:
                add(
                    "holdout result receipt self-digest drift",
                    lambda s, index=receipt_index: s["holdout"][
                        "attempts"
                    ][index]["result_receipt"].update(
                        {"receipt_sha256": "0" * 64}
                    ),
                    "stale",
                )
                add(
                    "holdout result checker binding drift",
                    lambda s, index=receipt_index: s["holdout"][
                        "attempts"
                    ][index]["result_receipt"].update(
                        {"structural_checker_sha256": "0" * 64}
                    ),
                    "stale",
                )
            reveal_index = next(
                (
                    index
                    for index, attempt in enumerate(holdout_attempts)
                    if attempt["commitment_reveal"] is not None
                ),
                None,
            )
            if reveal_index is not None:
                add(
                    "commitment nonce reveal drift",
                    lambda s, index=reveal_index: s["holdout"][
                        "attempts"
                    ][index]["commitment_reveal"].update(
                        {"nonce_hex": "00" * 32}
                    ),
                )
            gate_receipt_index = next(
                (
                    index
                    for index, attempt in enumerate(holdout_attempts)
                    if attempt["gate_admission_receipt"] is not None
                ),
                None,
            )
            if gate_receipt_index is not None:
                add(
                    "gate input receipt drift",
                    lambda s, index=gate_receipt_index: s["holdout"][
                        "attempts"
                    ][index]["gate_admission_receipt"].update(
                        {"input_sha256": "0" * 64}
                    ),
                    "stale",
                )
                add(
                    "gate receipt self-digest drift",
                    lambda s, index=gate_receipt_index: s["holdout"][
                        "attempts"
                    ][index]["gate_admission_receipt"].update(
                        {"receipt_sha256": "0" * 64}
                    ),
                    "stale",
                )
            if source["result"] != "not-run":
                add(
                    "persistent top-level result erased",
                    lambda s: s.update({"result": "not-run"}),
                    "latest completed",
                )

        if len(pilot_attempts) > 1:
            changed = copy.deepcopy(source)
            previous = changed["pilot"]["attempts"][-2]
            successor = changed["pilot"]["attempts"][-1]
            previous_terminal = (
                previous["decision_packet"]["freeze_binding"]["frozen_at"]
                if previous["attempt_status"] == "completed"
                else previous["voided_at"]
            )
            successor["pre_registration"]["freeze_binding"][
                "frozen_at"
            ] = previous_terminal
            expect_failure(
                "pilot successor chronology",
                lambda changed=changed: validate_history_closure(changed, source_raw=source_raw),
                "strictly follow",
            )
            direct_controls += 1
        if len(holdout_attempts) > 1:
            changed = copy.deepcopy(source)
            previous = changed["holdout"]["attempts"][-2]
            successor = changed["holdout"]["attempts"][-1]
            if previous["commitment_reveal"] is not None:
                previous_terminal = previous["commitment_reveal"][
                    "revealed_at"
                ]
            elif previous["attempt_status"] == "completed":
                previous_terminal = previous["result_receipt"][
                    "completed_at"
                ]
            else:
                previous_terminal = previous["voided_at"]
            successor["pre_registration"]["freeze_binding"][
                "frozen_at"
            ] = previous_terminal
            expect_failure(
                "holdout successor chronology",
                lambda changed=changed: validate_history_closure(changed, source_raw=source_raw),
                "strictly follow",
            )
            direct_controls += 1
        if pilot_attempts and holdout_attempts:
            changed = copy.deepcopy(source)
            changed["holdout"]["attempts"][0]["attempt_id"] = changed[
                "pilot"
            ]["attempts"][0]["attempt_id"]
            expect_failure(
                "cross-study attempt identity reuse",
                lambda changed=changed: validate_history_closure(changed, source_raw=source_raw),
                "duplicate across",
            )
            direct_controls += 1

    for label, mutate, expected in controls:
        changed = copy.deepcopy(source)
        mutate(changed)
        expect_failure(label, lambda changed=changed: validate(changed, source_raw=source_raw), expected)
    expect_failure(
        "duplicate JSON object key",
        lambda: json.loads(
            '{"result":"not-run","result":"pass"}',
            object_pairs_hook=reject_duplicate_keys,
        ),
        "duplicate",
    )
    return len(controls) + direct_controls + 1



def validate_state_tuple(
    threshold_status: str,
    holdout_status: str,
    result: str,
    route_status: str,
    posture: str,
    disposition: str,
    valid_pass: bool,
) -> None:
    enum(threshold_status, THRESHOLD_STATUSES, "state.threshold_status")
    enum(holdout_status, HOLDOUT_STATUSES, "state.holdout_status")
    enum(result, RESULTS, "state.result")
    enum(route_status, ROUTE_STATUSES, "state.route_status")
    if holdout_status == "not-frozen" and result != "not-run":
        raise ReaderEvidenceError(
            "an empty holdout history must remain not-run"
        )
    if holdout_status == "completed" and result == "not-run":
        raise ReaderEvidenceError(
            "completed holdout must preserve its result"
        )
    if threshold_status != "author-ratified" and holdout_status != "not-frozen":
        raise ReaderEvidenceError(
            "holdout state requires an author-ratified rule"
        )
    if route_status == "available" and threshold_status != "author-ratified":
        raise ReaderEvidenceError(
            "route availability requires an author-ratified rule"
        )
    if valid_pass and (
        holdout_status != "completed"
        or result != "pass"
        or route_status != "available"
    ):
        raise ReaderEvidenceError(
            "valid pass requires the active completed pass on an available route"
        )
    if (
        holdout_status == "completed"
        and result == "pass"
        and not valid_pass
    ):
        raise ReaderEvidenceError(
            "an active completed pass must be the matching admitted pass"
        )
    expected = (
        ("Evidenced", "none")
        if valid_pass
        else (
            ("Unestablished", "evidence-pending")
            if route_status == "available"
            else ("Unestablished", "route-unbuilt")
        )
    )
    if (posture, disposition) != expected:
        raise ReaderEvidenceError(
            "claim state contradicts route and admitted evidence"
        )



def comparator(operator: str, observed: Decimal, boundary: Decimal) -> bool:
    return {
        "lt": observed < boundary,
        "lte": observed <= boundary,
        "eq": observed == boundary,
        "gte": observed >= boundary,
        "gt": observed > boundary,
    }[operator]





def derived_boundary_evaluator_controls(
    rule: dict[str, object],
    known_misconceptions: set[str],
) -> int:
    """Exercise every ratified threshold through coded sessions end to end."""
    minimum_spec = as_object(
        rule["minimum_evaluable_evidence"], "boundary minimum threshold"
    )

    def passing_count(spec: dict[str, object]) -> int:
        boundary = int(str(spec["value"]))
        return boundary + (1 if spec["operator"] == "gt" else 0)

    baseline_count = max(1, passing_count(minimum_spec))
    target_entries: list[tuple[dict[str, object], str]] = []
    for raw in as_list(
        rule["required_target_thresholds"], "boundary target thresholds"
    ):
        entry = as_object(raw, "boundary target threshold")
        spec = as_object(entry["threshold"], "boundary target threshold spec")
        target_id = str(entry["target_id"])
        target_entries.append((spec, target_id))
        if spec["metric"] == "target-identification-count":
            baseline_count = max(baseline_count, passing_count(spec))

    misconception_items = [
        as_object(item, "boundary misconception")
        for item in as_list(rule["misconceptions"], "boundary misconceptions")
    ]
    core_ids = set(str(item) for item in rule["core_misconception_ids"])
    severity_ids: dict[str, set[str]] = {}
    for item in misconception_items:
        if item["core"] is False:
            severity_ids.setdefault(str(item["severity_id"]), set()).add(
                str(item["misconception_id"])
            )

    study_id = "RE-STUDY-BOUNDARY-CONTROL"

    def make_records(count: int, salt: str) -> list[dict[str, object]]:
        return [
            {
                "study_id": study_id,
                "record_commitment_sha256": sha256_bytes(
                    f"reader-boundary-{salt}-{index}".encode("utf-8")
                ),
                "admissibility": "admissible",
                "target_outcomes": [
                    {
                        "target_id": target_id,
                        "status": "identified",
                        "adjudication": "not-required",
                    }
                    for target_id in REQUIRED_TARGETS
                ],
                "misconception_outcomes": [
                    {
                        "misconception_id": item["misconception_id"],
                        "status": "absent",
                        "occurrences": "0",
                        "opportunities": "1",
                        "adjudication": "not-required",
                    }
                    for item in misconception_items
                ],
                "deviation_ids": [],
                "custody_attestation_ids": [],
            }
            for index in range(count)
        ]

    def relevant_outcomes(
        records: list[dict[str, object]], scope_ids: set[str]
    ) -> list[dict[str, object]]:
        return [
            outcome
            for record in records
            for outcome in record["misconception_outcomes"]
            if outcome["misconception_id"] in scope_ids
        ]

    def configure_occurrences(
        records: list[dict[str, object]],
        scope_ids: set[str],
        numerator: int,
        denominator: int | None,
    ) -> None:
        outcomes = relevant_outcomes(records, scope_ids)
        if not outcomes:
            raise ReaderEvidenceError(
                "boundary fixture has no scoped misconception outcomes"
            )
        total_opportunities = (
            max(len(outcomes), numerator)
            if denominator is None
            else denominator
        )
        if total_opportunities < len(outcomes):
            raise ReaderEvidenceError(
                "boundary opportunity denominator is not reachable"
            )
        for outcome in outcomes:
            outcome["status"] = "absent"
            outcome["occurrences"] = "0"
            outcome["opportunities"] = "1"
        outcomes[0]["opportunities"] = str(
            1 + total_opportunities - len(outcomes)
        )
        remaining = numerator
        for outcome in outcomes:
            capacity = int(str(outcome["opportunities"]))
            observed = min(remaining, capacity)
            outcome["occurrences"] = str(observed)
            outcome["status"] = "present" if observed else "absent"
            remaining -= observed
        if remaining:
            raise ReaderEvidenceError(
                "boundary occurrence numerator exceeds its denominator"
            )

    def find_check(
        trace: dict[str, object], threshold_id: str
    ) -> dict[str, object]:
        for raw_stage in as_list(trace["stages"], "boundary trace stages"):
            stage = as_object(raw_stage, "boundary trace stage")
            for raw_check in as_list(
                stage["checks"], "boundary trace checks"
            ):
                check = as_object(raw_check, "boundary trace check")
                if check.get("threshold_id") == threshold_id:
                    return check
        raise ReaderEvidenceError(
            f"boundary evaluator did not reach threshold {threshold_id}"
        )

    entries: list[tuple[str, dict[str, object], str | None]] = [
        ("minimum", minimum_spec, None)
    ]
    entries.extend(
        ("target", spec, target_id) for spec, target_id in target_entries
    )
    core_spec = as_object(
        rule["core_failure_threshold"], "boundary core threshold"
    )
    entries.append(("core", core_spec, None))
    for raw in as_list(
        rule["non_core_thresholds"], "boundary non-core thresholds"
    ):
        entry = as_object(raw, "boundary non-core threshold")
        entries.append(
            (
                "severity",
                as_object(
                    entry["threshold"], "boundary non-core threshold spec"
                ),
                str(entry["severity_id"]),
            )
        )

    controls = 0
    for kind, spec, scope_ref in entries:
        threshold_id = str(spec["threshold_id"])
        metric = str(spec["metric"])
        if spec["value_kind"] == "qualitative":
            for token in ("absent", "present"):
                records = make_records(
                    baseline_count, f"{threshold_id}-{token}"
                )
                if token == "present":
                    first = relevant_outcomes(records, core_ids)[0]
                    first["status"] = "present"
                    first["occurrences"] = "1"
                validated = validate_session_records(
                    records,
                    f"boundary fixture {threshold_id}/{token}",
                    expected_study_id=study_id,
                    known_misconceptions=known_misconceptions,
                )
                check = find_check(
                    evaluate_holdout(rule, validated, "valid"),
                    threshold_id,
                )
                expected = token == str(spec["value"])
                if (
                    check["observed"] != token
                    or check["comparison"] is not expected
                ):
                    raise ReaderEvidenceError(
                        f"qualitative boundary control failed for {threshold_id}/{token}"
                    )
                controls += 1
            continue

        boundary = Decimal(str(spec["value"]))
        denominator: int | None = None
        if spec["value_kind"] == "integer":
            exact_numerator = int(boundary)
            if exact_numerator <= 0:
                raise ReaderEvidenceError(
                    f"integer boundary lacks three reachable cases for {threshold_id}"
                )
            points = [
                ("below", exact_numerator - 1),
                ("exact", exact_numerator),
                ("above", exact_numerator + 1),
            ]
        else:
            scale = max(0, -boundary.as_tuple().exponent)
            unit = 10 ** scale
            minimum_denominator = baseline_count
            if kind == "core" and rule["repetition_unit"] == "coded-opportunity":
                minimum_denominator *= len(core_ids)
            if kind == "severity" and metric.startswith(
                "severity-occurrence-"
            ):
                minimum_denominator *= len(severity_ids[str(scope_ref)])
            multiplier = max(
                1, (minimum_denominator + unit - 1) // unit
            )
            denominator = unit * multiplier
            exact_decimal = boundary * Decimal(denominator)
            if exact_decimal != exact_decimal.to_integral_value():
                raise ReaderEvidenceError(
                    f"decimal boundary is not exactly representable for {threshold_id}"
                )
            exact_numerator = int(exact_decimal)
            if not 0 < exact_numerator < denominator:
                raise ReaderEvidenceError(
                    f"decimal boundary lacks three reachable cases for {threshold_id}"
                )
            points = [
                ("below", exact_numerator - 1),
                ("exact", exact_numerator),
                ("above", exact_numerator + 1),
            ]

        if [position for position, _ in points] != [
            "below", "exact", "above"
        ]:
            raise ReaderEvidenceError(
                f"boundary fixture must execute exactly three cases for {threshold_id}"
            )


        for position, numerator in points:
            if kind == "minimum":
                session_count = numerator
            elif metric.endswith("-rate") and (
                metric.startswith("target-")
                or metric.startswith("severity-session-")
                or (
                    metric.startswith("core-")
                    and rule["repetition_unit"] == "admissible-session"
                )
            ):
                if denominator is None:
                    raise ReaderEvidenceError(
                        "rate boundary fixture lost its denominator"
                    )
                session_count = denominator
            elif metric in {
                "target-identification-count",
                "severity-session-finding-count",
            } or (
                metric == "core-finding-count"
                and rule["repetition_unit"] == "admissible-session"
            ):
                session_count = max(baseline_count, numerator, 1)
            else:
                session_count = baseline_count

            records = make_records(
                session_count,
                f"{threshold_id}-{position}-{numerator}",
            )
            if kind == "target":
                for index, record in enumerate(records):
                    for outcome in record["target_outcomes"]:
                        if outcome["target_id"] == scope_ref:
                            outcome["status"] = (
                                "identified"
                                if index < numerator
                                else "not-identified"
                            )
            elif kind in {"core", "severity"}:
                scope_ids = (
                    core_ids
                    if kind == "core"
                    else severity_ids[str(scope_ref)]
                )
                session_metric = metric.startswith(
                    "severity-session-"
                ) or (
                    metric.startswith("core-")
                    and rule["repetition_unit"] == "admissible-session"
                )
                if session_metric:
                    for index, record in enumerate(records):
                        if index >= numerator:
                            continue
                        for outcome in record["misconception_outcomes"]:
                            if outcome["misconception_id"] in scope_ids:
                                outcome["status"] = "present"
                                outcome["occurrences"] = "1"
                                break
                else:
                    configure_occurrences(
                        records, scope_ids, numerator, denominator
                    )

            validated = validate_session_records(
                records,
                f"boundary fixture {threshold_id}/{position}",
                expected_study_id=study_id,
                known_misconceptions=known_misconceptions,
            )
            trace = evaluate_holdout(rule, validated, "valid")
            check = find_check(trace, threshold_id)
            observed = (
                str(numerator)
                if denominator is None
                else f"{numerator}/{denominator}"
            )
            expected_comparison: bool | None = comparator(
                str(spec["operator"]),
                Decimal(numerator),
                (
                    boundary
                    if denominator is None
                    else boundary * Decimal(denominator)
                ),
            )
            if kind == "minimum" and numerator == 0:
                expected_comparison = None
            if (
                check["observed"] != observed
                or check["comparison"] is not expected_comparison
            ):
                raise ReaderEvidenceError(
                    f"end-to-end {position} boundary control failed for {threshold_id}"
                )
            controls += 1
    return controls

def derived_evaluator_controls(
    rule: dict[str, object],
    known_misconceptions: set[str],
) -> int:
    """Exercise the ratified evaluator using only its eventual exact values."""
    specs: list[tuple[dict[str, object], str]] = [
        (
            as_object(
                rule["minimum_evaluable_evidence"],
                "minimum evaluator fixture threshold",
            ),
            "pass-high",
        )
    ]
    specs.extend(
        (
            as_object(
                as_object(item, "target fixture threshold")["threshold"],
                "target fixture threshold spec",
            ),
            "pass-high",
        )
        for item in as_list(
            rule["required_target_thresholds"],
            "required target fixture thresholds",
        )
    )
    core_spec = as_object(
        rule["core_failure_threshold"], "core fixture threshold"
    )
    specs.append((core_spec, "fail-high"))
    specs.extend(
        (
            as_object(
                as_object(item, "non-core fixture threshold")["threshold"],
                "non-core fixture threshold spec",
            ),
            "fail-high",
        )
        for item in as_list(
            rule["non_core_thresholds"],
            "non-core fixture thresholds",
        )
    )

    session_count = 1
    for spec, direction in specs:
        if spec["value_kind"] != "integer":
            continue
        boundary = int(str(spec["value"]))
        operator = str(spec["operator"])
        if direction == "pass-high":
            needed = boundary + (1 if operator == "gt" else 0)
        else:
            needed = boundary + (1 if operator in {"gt", "lte"} else 0)
        session_count = max(session_count, needed)

    misconception_items = [
        as_object(item, "fixture misconception")
        for item in as_list(rule["misconceptions"], "fixture misconceptions")
    ]
    study_id = "RE-STUDY-EVALUATOR-CONTROL"
    raw_records: list[dict[str, object]] = []
    for index in range(session_count):
        raw_records.append(
            {
                "study_id": study_id,
                "record_commitment_sha256": sha256_bytes(
                    f"reader-evaluator-fixture-{index}".encode("utf-8")
                ),
                "admissibility": "admissible",
                "target_outcomes": [
                    {
                        "target_id": target_id,
                        "status": "identified",
                        "adjudication": "not-required",
                    }
                    for target_id in REQUIRED_TARGETS
                ],
                "misconception_outcomes": [
                    {
                        "misconception_id": item["misconception_id"],
                        "status": "absent",
                        "occurrences": "0",
                        "opportunities": "1",
                        "adjudication": "not-required",
                    }
                    for item in misconception_items
                ],
                "deviation_ids": [],
                "custody_attestation_ids": [],
            }
        )
    records = validate_session_records(
        raw_records,
        "derived evaluator fixture",
        expected_study_id=study_id,
        known_misconceptions=known_misconceptions,
    )

    controls = 0

    def expect_trace(
        label: str,
        fixture: list[dict[str, object]],
        expected_verdict: str,
        failed_stage: str | None,
        protocol_validity: str = "valid",
    ) -> None:
        nonlocal controls
        trace = evaluate_holdout(rule, fixture, protocol_validity)
        if trace["verdict"] != expected_verdict:
            raise ReaderEvidenceError(
                f"{label}: evaluator returned {trace['verdict']}, expected {expected_verdict}"
            )
        stage_statuses = {
            str(item["stage"]): str(item["status"])
            for item in trace["stages"]
        }
        if failed_stage is not None and stage_statuses[failed_stage] != "fail":
            raise ReaderEvidenceError(
                f"{label}: evaluator did not fail at {failed_stage}"
            )
        controls += 1

    expect_trace("favourable ratified fixture", records, "pass", None)
    expect_trace(
        "invalid protocol dominates favourable evidence",
        records,
        "not-evaluable",
        "protocol-validity",
        protocol_validity="invalid",
    )
    expect_trace(
        "zero admitted denominator",
        [],
        "not-evaluable",
        "evaluability",
    )

    core_records = copy.deepcopy(records)
    core_ids = set(str(item) for item in rule["core_misconception_ids"])
    for record_index, record in enumerate(core_records):
        for outcome in record["misconception_outcomes"]:
            if outcome["misconception_id"] in core_ids and (
                rule["core_failure_mode"] == "repeated"
                or record_index == 0
            ):
                outcome["status"] = "present"
                outcome["occurrences"] = "1"
    expect_trace(
        f"selected {rule['core_failure_mode']} core veto",
        core_records,
        "fail",
        "core-veto",
    )

    for target_id in REQUIRED_TARGETS:
        target_records = copy.deepcopy(records)
        for record in target_records:
            for outcome in record["target_outcomes"]:
                if outcome["target_id"] == target_id:
                    outcome["status"] = "not-identified"
        expect_trace(
            f"required target boundary {target_id}",
            target_records,
            "fail",
            "required-targets",
        )

    misconception_by_id = {
        str(item["misconception_id"]): item
        for item in misconception_items
    }
    for raw in as_list(
        rule["non_core_thresholds"], "non-core fixture thresholds"
    ):
        entry = as_object(raw, "non-core fixture threshold")
        severity_id = str(entry["severity_id"])
        severity_records = copy.deepcopy(records)
        for record in severity_records:
            for outcome in record["misconception_outcomes"]:
                item = misconception_by_id[str(outcome["misconception_id"])]
                if (
                    item["severity_id"] == severity_id
                    and item["core"] is False
                ):
                    outcome["status"] = "present"
                    outcome["occurrences"] = "1"
        expect_trace(
            f"non-core boundary {severity_id}",
            severity_records,
            "fail",
            "non-core-rules",
        )

    policies = as_object(rule["policies"], "fixture policies")
    first_target = next(iter(REQUIRED_TARGETS))
    for policy_key, status in {
        "missing": "missing",
        "ambiguous": "ambiguous",
        "multiply_coded": "multiply-coded",
        "unclassified": "unclassified",
    }.items():
        policy_records = copy.deepcopy(records)
        for record in policy_records:
            for outcome in record["target_outcomes"]:
                if outcome["target_id"] == first_target:
                    outcome["status"] = status
                    outcome["adjudication"] = "not-required"
        action = resolved_policy_action(
            status,
            "not-required",
            policies,
            FINAL_TARGET_STATUSES,
        )
        expected = "fail" if action == "count-adverse" else "not-evaluable"
        stage = "required-targets" if expected == "fail" else "evaluability"
        expect_trace(
            f"ratified {policy_key} policy",
            policy_records,
            expected,
            stage,
        )

    coder_records = copy.deepcopy(records)
    for record in coder_records:
        for outcome in record["target_outcomes"]:
            if outcome["target_id"] == first_target:
                outcome["status"] = "ambiguous"
                outcome["adjudication"] = "unresolved"
    coder_action = resolved_policy_action(
        "ambiguous",
        "unresolved",
        policies,
        FINAL_TARGET_STATUSES,
    )
    coder_expected = (
        "fail" if coder_action == "count-adverse" else "not-evaluable"
    )
    expect_trace(
        "ratified coder-adjudication policy",
        coder_records,
        coder_expected,
        (
            "required-targets"
            if coder_expected == "fail"
            else "evaluability"
        ),
    )

    excluded_records = copy.deepcopy(raw_records)
    for suffix, admissibility in (
        ("WITHDRAWN", "withdrawn"),
        ("EXCLUDED", "inadmissible"),
    ):
        excluded_records.append(
            {
                "study_id": study_id,
                "record_commitment_sha256": sha256_bytes(
                    f"reader-evaluator-fixture-{suffix}".encode("utf-8")
                ),
                "admissibility": admissibility,
                "target_outcomes": [],
                "misconception_outcomes": [],
                "deviation_ids": [],
                "custody_attestation_ids": [],
            }
        )
    validated_exclusions = validate_session_records(
        excluded_records,
        "derived exclusion fixture",
        expected_study_id=study_id,
        known_misconceptions=known_misconceptions,
    )
    expect_trace(
        "withdrawn and excluded sessions stay outside denominators",
        validated_exclusions,
        "pass",
        None,
    )
    controls += derived_boundary_evaluator_controls(
        rule, known_misconceptions
    )
    return controls


def executable_controls(source: dict[str, object]) -> int:
    controls = 0
    invalid_states = [
        ("pending-pilot", "not-frozen", "pass", "unbuilt", "Unestablished", "route-unbuilt", False),
        ("pending-pilot", "void", "not-run", "unbuilt", "Unestablished", "route-unbuilt", False),
        ("author-ratified", "completed", "not-run", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "completed", "fail", "available", "Unestablished", "route-unbuilt", False),
        ("author-ratified", "completed", "pass", "available", "Unestablished", "evidence-pending", True),
        ("author-ratified", "completed", "pass", "unbuilt", "Evidenced", "none", True),
        ("author-ratified", "completed", "pass", "available", "Evidenced", "none", False),
        ("author-ratified", "frozen", "fail", "available", "Evidenced", "none", True),
    ]
    for index, state in enumerate(invalid_states):
        expect_failure(
            f"invalid state transition {index}",
            lambda state=state: validate_state_tuple(*state),
        )
        controls += 1

    valid_states = [
        ("pending-pilot", "not-frozen", "not-run", "unbuilt", "Unestablished", "route-unbuilt", False),
        ("author-ratified", "frozen", "not-run", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "frozen", "fail", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "frozen", "pass", "unbuilt", "Unestablished", "route-unbuilt", False),
        ("author-ratified", "completed", "not-evaluable", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "completed", "fail", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "completed", "pass", "available", "Evidenced", "none", True),
        ("author-ratified", "void", "not-run", "available", "Unestablished", "evidence-pending", False),
        ("author-ratified", "void", "fail", "available", "Unestablished", "evidence-pending", False),
    ]
    for state in valid_states:
        validate_state_tuple(*state)
        controls += 1

    stage_cases = [
        (
            {
                "protocol_valid": False,
                "evaluable": True,
                "core_veto": False,
                "required_targets_pass": True,
                "non_core_pass": True,
            },
            "not-evaluable",
            "protocol-validity",
        ),
        (
            {
                "protocol_valid": True,
                "evaluable": False,
                "core_veto": False,
                "required_targets_pass": True,
                "non_core_pass": True,
            },
            "not-evaluable",
            "evaluability",
        ),
        (
            {
                "protocol_valid": True,
                "evaluable": True,
                "core_veto": True,
                "required_targets_pass": True,
                "non_core_pass": True,
            },
            "fail",
            "core-veto",
        ),
        (
            {
                "protocol_valid": True,
                "evaluable": True,
                "core_veto": False,
                "required_targets_pass": False,
                "non_core_pass": True,
            },
            "fail",
            "required-targets",
        ),
        (
            {
                "protocol_valid": True,
                "evaluable": True,
                "core_veto": False,
                "required_targets_pass": True,
                "non_core_pass": False,
            },
            "fail",
            "non-core-rules",
        ),
        (
            {
                "protocol_valid": True,
                "evaluable": True,
                "core_veto": False,
                "required_targets_pass": True,
                "non_core_pass": True,
            },
            "pass",
            None,
        ),
    ]
    for inputs, expected_verdict, failed_stage in stage_cases:
        trace = ordered_evaluation_trace(
            **inputs,
            checks={stage: [] for stage in EVALUATION_ORDER},
        )
        if trace["verdict"] != expected_verdict:
            raise ReaderEvidenceError(
                "ordered evaluator produced the wrong verdict"
            )
        stages = {
            item["stage"]: item["status"]
            for item in trace["stages"]
        }
        if failed_stage is not None:
            if stages[failed_stage] != "fail":
                raise ReaderEvidenceError(
                    "ordered evaluator failed at the wrong stage"
                )
            failed_index = EVALUATION_ORDER.index(failed_stage)
            if any(
                stages[stage] != "not-reached"
                for stage in EVALUATION_ORDER[failed_index + 1 :]
            ):
                raise ReaderEvidenceError(
                    "ordered evaluator continued after a decisive stage"
                )
        controls += 1

    policy_template = {
        "missing": "study-not-evaluable",
        "ambiguous": "study-not-evaluable",
        "multiply_coded": "study-not-evaluable",
        "unclassified": "study-not-evaluable",
        "coder_adjudication": "unresolved-not-evaluable",
    }
    policy_statuses = {
        "missing": "missing",
        "ambiguous": "ambiguous",
        "multiply_coded": "multiply-coded",
        "unclassified": "unclassified",
    }
    for policy_key, status in policy_statuses.items():
        for action in sorted(OUTCOME_POLICY_ACTIONS):
            policies = dict(policy_template)
            policies[policy_key] = action
            observed = resolved_policy_action(
                status,
                "not-required",
                policies,
                FINAL_TARGET_STATUSES,
            )
            expected = (
                "study-not-evaluable"
                if action == "require-adjudication"
                else action
            )
            if observed != expected:
                raise ReaderEvidenceError(
                    f"policy action failed for {policy_key}/{action}"
                )
            controls += 1
    for action, expected in {
        "unresolved-count-adverse": "count-adverse",
        "unresolved-exclude-observation": "exclude-observation",
        "unresolved-not-evaluable": "study-not-evaluable",
    }.items():
        policies = dict(policy_template)
        policies["coder_adjudication"] = action
        observed = resolved_policy_action(
            "ambiguous",
            "unresolved",
            policies,
            FINAL_TARGET_STATUSES,
        )
        if observed != expected:
            raise ReaderEvidenceError(
                f"coder-adjudication action failed for {action}"
            )
        controls += 1

    rule = as_object(source["threshold_rule"], "threshold_rule")
    specs: list[dict[str, object]] = []
    if source["threshold_status"] != "pending-pilot":
        specs.append(as_object(rule["core_failure_threshold"], "core threshold"))
        specs.append(as_object(rule["minimum_evaluable_evidence"], "minimum evidence"))
        specs.extend(
            as_object(item, "target threshold")["threshold"]
            for item in as_list(rule["required_target_thresholds"], "required targets")
        )
        specs.extend(
            as_object(item, "severity threshold")["threshold"]
            for item in as_list(rule["non_core_thresholds"], "non-core thresholds")
        )
    for raw in specs:
        spec = as_object(raw, "boundary spec")
        if spec["value_kind"] not in {"integer", "decimal"}:
            continue
        try:
            boundary = Decimal(str(spec["value"]))
        except InvalidOperation as exc:
            raise ReaderEvidenceError(
                "validated numeric threshold stopped parsing"
            ) from exc
        step = (
            Decimal(1)
            if spec["value_kind"] == "integer"
            else Decimal(1).scaleb(boundary.as_tuple().exponent)
        )
        below, exact, above = boundary - step, boundary, boundary + step
        actual = [
            comparator(str(spec["operator"]), item, boundary)
            for item in (below, exact, above)
        ]
        expected = {
            "lt": [True, False, False],
            "lte": [True, True, False],
            "eq": [False, True, False],
            "gte": [False, True, True],
            "gt": [False, False, True],
        }[str(spec["operator"])]
        if actual != expected:
            raise ReaderEvidenceError(
                "derived below/exact/above boundary control failed"
            )
        controls += 1

    if source["threshold_status"] == "pending-pilot":
        dormant_trace = evaluate_holdout({}, [], "invalid")
        if dormant_trace["verdict"] != "not-evaluable":
            raise ReaderEvidenceError(
                "invalid-protocol end-to-end evaluator control failed"
            )
        controls += 1
    else:
        known_misconceptions = {
            str(item["misconception_id"])
            for item in as_list(
                rule["misconceptions"], "eventual evaluator misconceptions"
            )
        }
        controls += derived_evaluator_controls(
            rule, known_misconceptions
        )
    return controls


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ReaderEvidenceError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json(path: pathlib.Path) -> tuple[dict[str, object], bytes]:
    try:
        raw = path.read_bytes()
        decoded = raw.decode("utf-8")
        value = json.loads(decoded, object_pairs_hook=reject_duplicate_keys)
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ReaderEvidenceError(f"cannot read reader-evidence source {path}: {exc}") from exc
    return as_object(value, "root"), raw


def write_output(path: pathlib.Path, value: str) -> None:
    if path.is_symlink():
        raise ReaderEvidenceError("generated report may not be a symlink")
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + f".tmp-{os.getpid()}")
    try:
        temporary.write_text(value, encoding="utf-8", newline="\n")
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args(argv)

    source_path = resolve(args.source)
    output_path = resolve(args.output)
    repo_relative(source_path)
    repo_relative(output_path)
    if output_path.resolve(strict=False) != resolve(DEFAULT_OUTPUT).resolve(strict=False):
        raise ReaderEvidenceError("--output is fixed to new-book-plans/reader-evidence.md")
    if output_path.resolve(strict=False) == source_path.resolve(strict=False):
        raise ReaderEvidenceError("generated report may not overwrite its source")
    source, raw = load_json(source_path)
    validate(source, source_raw=raw)
    source_digest = sha256_bytes(raw)
    generated = render(source, source_digest)
    structural = structural_controls(source, source_raw=raw)
    executed = executable_controls(source) if args.execute else 0

    if args.check:
        try:
            current = output_path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise ReaderEvidenceError(f"cannot read generated report {output_path}: {exc}") from exc
        if current != generated:
            raise ReaderEvidenceError(f"{repo_relative(output_path)} is STALE — rerun without --check")
        suffix = (
            f"; {executed} executable contract controls pass; no reader study executed"
            if args.execute
            else "; execution skipped"
        )
        print(
            f"{repo_relative(output_path)} is current; "
            f"{structural} watched-failing structural controls pass{suffix}"
        )
        return 0

    write_output(output_path, generated)
    suffix = f"; {executed} executable contract controls pass" if args.execute else ""
    print(
        f"{repo_relative(output_path)}: regenerated; "
        f"{structural} watched-failing structural controls pass{suffix}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReaderEvidenceError as exc:
        print(f"14-reader-evidence: {exc}", file=sys.stderr)
        raise SystemExit(1)
