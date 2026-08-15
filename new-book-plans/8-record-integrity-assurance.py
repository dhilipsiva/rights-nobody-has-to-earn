#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate and render Book 1's record-integrity assurance case.

The reviewed JSON source states the argument.  This program checks its coverage,
traceability, posture, and dependency on the assertion-surface ledger, then
renders the Markdown report.  It does not test forged constitutional facts or
claim that an in-snapshot model can recover omitted or deleted records.

Usage:
    python3 new-book-plans/8-record-integrity-assurance.py
    python3 new-book-plans/8-record-integrity-assurance.py --check

Relative paths are resolved from the repository root.  ``--check`` also runs
in-memory negative controls and fails when the generated report has drifted.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import sys
from typing import Callable, Iterable, Mapping, Sequence


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path(
    "new-book-plans/record-integrity-assurance-case.json"
)
DEFAULT_LEDGER = pathlib.Path("new-book-plans/assertion-surface-contracts.json")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/record-integrity-assurance-case.md")

POSTURES = {
    "current_verified",
    "external_verified",
    "book1_target_unimplemented",
    "book2_external_assumption",
    "refused_or_unprovable",
}
EVIDENCE_ROLES = {
    "supports_current",
    "supports_external",
    "exposes_gap",
    "sets_boundary",
}
EVIDENCE_KINDS = {
    "counterfactual",
    "decision",
    "formal",
    "generated",
    "operational",
    "prose",
    "reviewed",
}
MANDATORY_DIMENSIONS = {
    "surface_completeness",
    "authorship",
    "authority",
    "permitted_basis",
    "provenance_authenticity",
    "visibility_privacy",
    "independent_witnessing",
    "separation_of_functions",
    "challenge",
    "append_only_correction_history",
    "retention",
    "deletion_control",
    "reconciliation",
    "external_assurance",
    "independent_recipient",
    "action_duty",
    "continuity_remedy",
    "omission_or_deletion_recovery",
    "temporal_status",
    "failure_polarity",
    "rule_integrity",
    "negative_premise_admissibility",
}
OPTIONAL_DIMENSIONS: set[str] = set()
REQUIRED_CLAIM_IDS = {f"RI-{number}" for number in range(1, 14)}
REQUIRED_REFUSED_IDS = {"RI-11"}
REQUIRED_EXTERNAL_IDS = {"RI-10"}
REQUIRED_RECORD_IDS = {f"RC-{number}" for number in range(1, 6)}
REQUIRED_DEFEATER_IDS = {f"RD-{number}" for number in range(1, 16)}
REQUIRED_FAIL_SAFE_IDS = {f"RF-{number}" for number in range(1, 7)}
REQUIRED_ACCEPTANCE_IDS = {f"RA-{number}" for number in range(1, 9)}
REQUIRED_NARROWNESS_FILES = {
    "book-1/01-what-counts-as-evidence.md",
    "book-1/03-who-holds-the-pen.md",
    "book-1/05-voiding.md",
    "book-1/09-the-vote-conviction-does-not-take.md",
    "book-1/12-changing-the-rules.md",
    "book-1/14-when-the-system-notices-it-broke.md",
    "book-1/15-the-five-joints.md",
    "book-1/method.md",
}
LIMITATION_KEYS = (
    "in_snapshot_absence",
    "t1_boundary",
    "monotone_derivation",
    "independence",
    "authenticity",
    "genesis",
    "classification_choice",
    "assurance_meta_root",
)
ROOT_KEYS = {
    "spdx",
    "schema_version",
    "assertion_surface_contracts_sha256",
    "title",
    "top_claim",
    "status_meanings",
    "required_dimensions",
    "limitations",
    "boundary",
    "claims",
    "record_classes",
    "premise_classes",
    "defeaters",
    "fail_safe_defaults",
    "narrowness_impacts",
    "acceptance_gate",
}
CLAIM_KEYS = {
    "id",
    "title",
    "claim",
    "argument",
    "posture",
    "dimensions",
    "current_evidence",
    "known_failure",
    "target_contract",
    "acceptance_evidence",
    "residual_assumption",
    "owner_ref",
    "temporal_status",
    "book2_handoff",
}
CLAIM_ID = re.compile(r"^RI-[0-9]+$")
RECORD_ID = re.compile(r"^RC-[0-9]+$")
DEFEATER_ID = re.compile(r"^RD-[0-9]+$")
FAIL_SAFE_ID = re.compile(r"^RF-[0-9]+$")
ACCEPTANCE_ID = re.compile(r"^RA-[0-9]+$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
PLACEHOLDER = re.compile(r"^(?:tbd|todo|unknown|n/?a|pending)$", re.I)


class AssuranceError(RuntimeError):
    """An invalid source, evidence reference, or generated artifact."""


def resolve(path: pathlib.Path) -> pathlib.Path:
    """Resolve a CLI path against the repository root."""
    return path if path.is_absolute() else ROOT / path


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append("missing " + ", ".join(missing))
        if extra:
            details.append("unknown " + ", ".join(extra))
        raise AssuranceError(f"{path}: {'; '.join(details)}")


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise AssuranceError(f"{path}: expected an object")
    if not all(isinstance(key, str) for key in value):
        raise AssuranceError(f"{path}: object keys must be strings")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise AssuranceError(f"{path}: expected an array")
    return value


def as_text(value: object, path: str, *, allow_empty: bool = False) -> str:
    if not isinstance(value, str):
        raise AssuranceError(f"{path}: expected a string")
    if not value.strip():
        if allow_empty:
            return value
        raise AssuranceError(f"{path}: requires reviewed, non-placeholder text")
    if PLACEHOLDER.fullmatch(value.strip()):
        raise AssuranceError(f"{path}: requires reviewed, non-placeholder text")
    return value


def text_list(
    value: object,
    path: str,
    *,
    nonempty: bool = True,
    unique: bool = True,
) -> list[str]:
    items = as_list(value, path)
    if nonempty and not items:
        raise AssuranceError(f"{path}: must not be empty")
    result = [as_text(item, f"{path}[{index}]") for index, item in enumerate(items)]
    if unique and len(result) != len(set(result)):
        raise AssuranceError(f"{path}: duplicate values are not allowed")
    return result


def validate_id(
    value: object, path: str, pattern: re.Pattern[str], family: str
) -> str:
    identifier = as_text(value, path)
    if not pattern.fullmatch(identifier):
        raise AssuranceError(
            f"{path}: {identifier!r} must be a stable {family} identifier"
        )
    return identifier


def sha256(path: pathlib.Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as exc:
        raise AssuranceError(f"cannot read {path}: {exc}") from exc


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve().relative_to(ROOT.resolve())
    except (OSError, ValueError) as exc:
        raise AssuranceError(f"reference path escapes the repository: {path}") from exc


def validate_reference(value: object, path: str) -> str:
    """Validate a stable ``path::unique literal needle`` reference."""
    reference = as_text(value, path)
    if reference.count("::") != 1:
        raise AssuranceError(
            f"{path}: reference must be repo-local path::unique literal needle"
        )
    raw_file, needle = reference.split("::", 1)
    if not raw_file or not needle:
        raise AssuranceError(f"{path}: reference path and needle must both be non-empty")
    if "\\" in raw_file:
        raise AssuranceError(f"{path}: reference paths must use forward slashes")
    candidate = pathlib.Path(raw_file)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise AssuranceError(f"{path}: reference path must stay inside the repository")
    target = ROOT / candidate
    repo_relative(target)
    if not target.is_file():
        raise AssuranceError(f"{path}: referenced file does not exist: {raw_file}")
    try:
        body = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise AssuranceError(f"{path}: cannot read referenced file {raw_file}: {exc}") from exc
    count = body.count(needle)
    if count != 1:
        raise AssuranceError(
            f"{path}: needle must occur exactly once in {raw_file}; found {count}"
        )
    return reference


def validate_id_references(
    value: object, path: str, known_claims: set[str]
) -> list[str]:
    references = text_list(value, path)
    unknown = sorted(set(references) - known_claims)
    if unknown:
        raise AssuranceError(f"{path}: unknown claim ID(s): {', '.join(unknown)}")
    return references


def validate_top_claim(value: object) -> dict[str, object]:
    top = as_object(value, "top_claim")
    exact_keys(top, {"id", "claim", "argument", "current_verdict"}, "top_claim")
    validate_id(top["id"], "top_claim.id", CLAIM_ID, "RI-N claim")
    as_text(top["claim"], "top_claim.claim")
    as_text(top["argument"], "top_claim.argument")
    verdict = as_text(top["current_verdict"], "top_claim.current_verdict")
    if verdict not in {"established", "not_established"}:
        raise AssuranceError(
            "top_claim.current_verdict: expected established or not_established"
        )
    return top


def validate_claim_shape(
    value: object,
    index: int,
    forbidden_evidence_files: set[str],
) -> dict[str, object]:
    path = f"claims[{index}]"
    claim = as_object(value, path)
    exact_keys(claim, CLAIM_KEYS, path)
    validate_id(claim["id"], f"{path}.id", CLAIM_ID, "RI-N claim")
    for key in (
        "title",
        "claim",
        "argument",
        "known_failure",
        "target_contract",
        "residual_assumption",
        "temporal_status",
        "book2_handoff",
    ):
        as_text(claim[key], f"{path}.{key}")
    posture = as_text(claim["posture"], f"{path}.posture")
    if posture not in POSTURES:
        raise AssuranceError(
            f"{path}.posture: expected one of {', '.join(sorted(POSTURES))}"
        )
    dimensions = text_list(claim["dimensions"], f"{path}.dimensions")
    unknown_dimensions = sorted(
        set(dimensions) - MANDATORY_DIMENSIONS - OPTIONAL_DIMENSIONS
    )
    if unknown_dimensions:
        raise AssuranceError(
            f"{path}.dimensions: unknown dimension(s): "
            + ", ".join(unknown_dimensions)
        )
    evidence = as_list(claim["current_evidence"], f"{path}.current_evidence")
    if not evidence:
        raise AssuranceError(f"{path}.current_evidence: must not be empty")
    for evidence_index, raw_entry in enumerate(evidence):
        evidence_path = f"{path}.current_evidence[{evidence_index}]"
        entry = as_object(raw_entry, evidence_path)
        exact_keys(entry, {"kind", "role", "supports", "ref"}, evidence_path)
        kind = as_text(entry["kind"], f"{evidence_path}.kind")
        if kind not in EVIDENCE_KINDS:
            raise AssuranceError(
                f"{evidence_path}.kind: expected one of "
                + ", ".join(sorted(EVIDENCE_KINDS))
            )
        role = as_text(entry["role"], f"{evidence_path}.role")
        if role not in EVIDENCE_ROLES:
            raise AssuranceError(
                f"{evidence_path}.role: expected one of "
                + ", ".join(sorted(EVIDENCE_ROLES))
            )
        validate_id(
            entry["supports"],
            f"{evidence_path}.supports",
            CLAIM_ID,
            "RI-N claim",
        )
        reference = validate_reference(entry["ref"], f"{evidence_path}.ref")
        evidence_file = reference.split("::", 1)[0]
        if evidence_file in forbidden_evidence_files:
            raise AssuranceError(
                f"{evidence_path}.ref: the assurance source or its generated "
                "report cannot serve as current evidence for itself"
            )
    text_list(
        claim["acceptance_evidence"],
        f"{path}.acceptance_evidence",
        nonempty=False,
    )
    validate_reference(claim["owner_ref"], f"{path}.owner_ref")
    return claim


def explicit_impossibility(claim: Mapping[str, object]) -> bool:
    combined = str(claim["claim"]).lower()
    markers = (
        "cannot",
        "can not",
        "unprovable",
        "not provable",
        "impossible",
        "does not distinguish",
        "no in-snapshot",
        "no internal",
        "no rule",
        "refused",
    )
    return any(marker in combined for marker in markers)


def validate_posture(claim: Mapping[str, object], index: int) -> None:
    path = f"claims[{index}]"
    posture = str(claim["posture"])
    current_evidence = claim["current_evidence"]
    acceptance = claim["acceptance_evidence"]
    roles = {
        str(as_object(entry, f"{path}.current_evidence")["role"])
        for entry in current_evidence
    }
    if posture == "current_verified":
        if "supports_current" not in roles:
            raise AssuranceError(
                f"{path}: current_verified requires supports_current evidence"
            )
        if "exposes_gap" in roles:
            raise AssuranceError(
                f"{path}: current_verified cannot rely on evidence that exposes a gap"
            )
        if "supports_external" in roles:
            raise AssuranceError(
                f"{path}: current_verified cannot substitute deployed evidence "
                "for a repository invariant"
            )
        if acceptance:
            raise AssuranceError(
                f"{path}: current_verified cannot retain unmet acceptance evidence"
            )
    elif posture == "external_verified":
        if claim["id"] not in REQUIRED_EXTERNAL_IDS:
            raise AssuranceError(
                f"{path}: {claim['id']} is not a reviewed schema-v1 external claim"
            )
        operational_support = any(
            as_object(entry, f"{path}.current_evidence")["role"]
            == "supports_external"
            and as_object(entry, f"{path}.current_evidence")["kind"]
            == "operational"
            for entry in current_evidence
        )
        if not operational_support:
            raise AssuranceError(
                f"{path}: external_verified requires reviewed operational evidence"
            )
        if roles & {"exposes_gap", "supports_current"}:
            raise AssuranceError(
                f"{path}: external_verified cannot rely on gap evidence or a "
                "repository-only invariant"
            )
        if acceptance:
            raise AssuranceError(
                f"{path}: external_verified cannot retain unmet acceptance evidence"
            )
        if not str(claim["owner_ref"]).startswith("book-2/"):
            raise AssuranceError(
                f"{path}: external_verified owner must be in book-2/"
            )
    elif posture == "book1_target_unimplemented":
        if not acceptance:
            raise AssuranceError(
                f"{path}: book1_target_unimplemented requires acceptance evidence"
            )
        if not str(claim["target_contract"]).strip():
            raise AssuranceError(
                f"{path}: book1_target_unimplemented requires a target contract"
            )
        if not roles & {"exposes_gap", "sets_boundary"}:
            raise AssuranceError(
                f"{path}: book1_target_unimplemented requires gap or boundary evidence"
            )
    elif posture == "book2_external_assumption":
        if claim["id"] not in REQUIRED_EXTERNAL_IDS:
            raise AssuranceError(
                f"{path}: {claim['id']} is not a reviewed schema-v1 external claim"
            )
        if not str(claim["book2_handoff"]).strip():
            raise AssuranceError(
                f"{path}: book2_external_assumption requires a Book 2 handoff"
            )
        if not str(claim["residual_assumption"]).strip():
            raise AssuranceError(
                f"{path}: book2_external_assumption requires a residual assumption"
            )
        if not acceptance:
            raise AssuranceError(
                f"{path}: book2_external_assumption requires acceptance evidence"
            )
        if "sets_boundary" not in roles or roles & {
            "supports_current",
            "supports_external",
        }:
            raise AssuranceError(
                f"{path}: book2_external_assumption requires boundary evidence, "
                "not evidence claiming the external service is current"
            )
        if not str(claim["owner_ref"]).startswith("book-2/"):
            raise AssuranceError(
                f"{path}: book2_external_assumption owner must be in book-2/"
            )
    elif posture == "refused_or_unprovable":
        if claim["id"] not in REQUIRED_REFUSED_IDS:
            raise AssuranceError(
                f"{path}: {claim['id']} is not a reviewed schema-v1 refusal"
            )
        if not explicit_impossibility(claim):
            raise AssuranceError(
                f"{path}: refused_or_unprovable must state the impossibility "
                "explicitly in its claim"
            )
        if acceptance:
            raise AssuranceError(
                f"{path}: refused_or_unprovable cannot retain an implementation gate"
            )
        if not roles & {"supports_current", "supports_external", "sets_boundary"}:
            raise AssuranceError(
                f"{path}: refused_or_unprovable requires evidence for its boundary"
            )


def validate_record_classes(
    value: object,
    known_claims: set[str],
    claims_by_id: Mapping[str, Mapping[str, object]],
) -> tuple[list[dict[str, object]], set[str]]:
    classes = as_list(value, "record_classes")
    if not classes:
        raise AssuranceError("record_classes: must not be empty")
    result: list[dict[str, object]] = []
    identifiers: set[str] = set()
    expected = {
        "id",
        "title",
        "description",
        "assurance_claims",
        "failure_posture",
    }
    for index, raw_class in enumerate(classes):
        path = f"record_classes[{index}]"
        record_class = as_object(raw_class, path)
        exact_keys(record_class, expected, path)
        identifier = validate_id(
            record_class["id"], f"{path}.id", RECORD_ID, "RC-N record-class"
        )
        if identifier in identifiers:
            raise AssuranceError(f"{path}.id: duplicate record-class ID {identifier}")
        identifiers.add(identifier)
        as_text(record_class["title"], f"{path}.title")
        as_text(record_class["description"], f"{path}.description")
        claim_references = validate_id_references(
            record_class["assurance_claims"],
            f"{path}.assurance_claims",
            known_claims,
        )
        covered_dimensions = {
            str(dimension)
            for claim_id in claim_references
            for dimension in claims_by_id[claim_id]["dimensions"]
        }
        missing_dimensions = sorted(MANDATORY_DIMENSIONS - covered_dimensions)
        if missing_dimensions:
            raise AssuranceError(
                f"{path}.assurance_claims: record class bypasses mandatory "
                "assurance dimension(s): " + ", ".join(missing_dimensions)
            )
        as_text(record_class["failure_posture"], f"{path}.failure_posture")
        result.append(record_class)
    return result, identifiers


def validate_premise_classes(
    value: object,
    ledger: Mapping[str, object],
    record_ids: set[str],
) -> dict[str, object]:
    mapping = as_object(value, "premise_classes")
    premises = as_object(ledger.get("premises"), "assertion ledger premises")
    ledger_keys = set(premises)
    mapping_keys = set(mapping)
    missing = sorted(ledger_keys - mapping_keys)
    extra = sorted(mapping_keys - ledger_keys)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append("uncovered premise(s): " + ", ".join(missing))
        if extra:
            details.append("unknown premise(s): " + ", ".join(extra))
        raise AssuranceError("premise_classes: " + "; ".join(details))
    for premise, raw_class in sorted(mapping.items()):
        class_id = validate_id(
            raw_class,
            f"premise_classes.{premise}",
            RECORD_ID,
            "RC-N record-class",
        )
        if class_id not in record_ids:
            raise AssuranceError(
                f"premise_classes.{premise}: unknown record-class ID {class_id}"
            )
    return mapping


def validate_defeaters(
    value: object, known_claims: set[str]
) -> list[dict[str, object]]:
    entries = as_list(value, "defeaters")
    if not entries:
        raise AssuranceError("defeaters: must not be empty")
    expected = {
        "id",
        "title",
        "attack",
        "disposition",
        "owner_claims",
        "failure_consequence",
    }
    result: list[dict[str, object]] = []
    identifiers: set[str] = set()
    for index, raw_entry in enumerate(entries):
        path = f"defeaters[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, expected, path)
        identifier = validate_id(
            entry["id"], f"{path}.id", DEFEATER_ID, "RD-N defeater"
        )
        if identifier in identifiers:
            raise AssuranceError(f"{path}.id: duplicate defeater ID {identifier}")
        identifiers.add(identifier)
        for key in ("title", "attack", "disposition", "failure_consequence"):
            as_text(entry[key], f"{path}.{key}")
        if entry["disposition"] not in POSTURES:
            raise AssuranceError(
                f"{path}.disposition: expected one of "
                + ", ".join(sorted(POSTURES))
            )
        validate_id_references(
            entry["owner_claims"], f"{path}.owner_claims", known_claims
        )
        result.append(entry)
    missing = sorted(REQUIRED_DEFEATER_IDS - identifiers)
    if missing:
        raise AssuranceError("defeaters: missing required ID(s): " + ", ".join(missing))
    return result


def validate_fail_safe_defaults(
    value: object, known_claims: set[str]
) -> list[dict[str, object]]:
    entries = as_list(value, "fail_safe_defaults")
    if not entries:
        raise AssuranceError("fail_safe_defaults: must not be empty")
    expected = {"id", "condition", "required_default", "rationale", "owner_claims"}
    result: list[dict[str, object]] = []
    identifiers: set[str] = set()
    for index, raw_entry in enumerate(entries):
        path = f"fail_safe_defaults[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, expected, path)
        identifier = validate_id(
            entry["id"], f"{path}.id", FAIL_SAFE_ID, "RF-N fail-safe"
        )
        if identifier in identifiers:
            raise AssuranceError(f"{path}.id: duplicate fail-safe ID {identifier}")
        identifiers.add(identifier)
        for key in ("condition", "required_default", "rationale"):
            as_text(entry[key], f"{path}.{key}")
        validate_id_references(
            entry["owner_claims"], f"{path}.owner_claims", known_claims
        )
        result.append(entry)
    missing = sorted(REQUIRED_FAIL_SAFE_IDS - identifiers)
    if missing:
        raise AssuranceError(
            "fail_safe_defaults: missing required ID(s): " + ", ".join(missing)
        )
    return result


def validate_narrowness(value: object) -> list[dict[str, object]]:
    entries = as_list(value, "narrowness_impacts")
    if not entries:
        raise AssuranceError("narrowness_impacts: must not be empty")
    expected = {
        "artifact_ref",
        "current_claim",
        "classification",
        "reason",
        "future_trigger",
    }
    result: list[dict[str, object]] = []
    seen: set[str] = set()
    for index, raw_entry in enumerate(entries):
        path = f"narrowness_impacts[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, expected, path)
        reference = validate_reference(entry["artifact_ref"], f"{path}.artifact_ref")
        if reference in seen:
            raise AssuranceError(f"{path}.artifact_ref: duplicate narrowness artifact")
        seen.add(reference)
        for key in ("current_claim", "classification", "reason", "future_trigger"):
            as_text(entry[key], f"{path}.{key}")
        if entry["classification"] not in {"preserved", "revised", "retired"}:
            raise AssuranceError(
                f"{path}.classification: expected preserved, revised, or retired"
            )
        result.append(entry)
    covered_files = {reference.split("::", 1)[0] for reference in seen}
    missing = sorted(REQUIRED_NARROWNESS_FILES - covered_files)
    if missing:
        raise AssuranceError(
            "narrowness_impacts: missing standing artifact(s): " + ", ".join(missing)
        )
    return result


def validate_acceptance_gate(value: object) -> list[dict[str, object]]:
    entries = as_list(value, "acceptance_gate")
    if not entries:
        raise AssuranceError("acceptance_gate: must not be empty")
    expected = {"id", "requirement", "evidence_needed", "owner_ref"}
    result: list[dict[str, object]] = []
    identifiers: set[str] = set()
    for index, raw_entry in enumerate(entries):
        path = f"acceptance_gate[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, expected, path)
        identifier = validate_id(
            entry["id"], f"{path}.id", ACCEPTANCE_ID, "RA-N acceptance-gate"
        )
        if identifier in identifiers:
            raise AssuranceError(f"{path}.id: duplicate acceptance-gate ID {identifier}")
        identifiers.add(identifier)
        as_text(entry["requirement"], f"{path}.requirement")
        as_text(entry["evidence_needed"], f"{path}.evidence_needed")
        validate_reference(entry["owner_ref"], f"{path}.owner_ref")
        result.append(entry)
    missing = sorted(REQUIRED_ACCEPTANCE_IDS - identifiers)
    if missing:
        raise AssuranceError(
            "acceptance_gate: missing required ID(s): " + ", ".join(missing)
        )
    return result


def validate_source(
    source: dict[str, object],
    ledger: dict[str, object],
    ledger_digest: str,
    forbidden_evidence_files: set[str] | None = None,
) -> None:
    exact_keys(source, ROOT_KEYS, "root")
    if as_text(source["spdx"], "spdx") != "CC-BY-4.0":
        raise AssuranceError("spdx: assurance-case prose must be CC-BY-4.0")
    version = source["schema_version"]
    if not isinstance(version, int) or isinstance(version, bool) or version != 1:
        raise AssuranceError("schema_version: expected integer 1")
    expected_digest = as_text(
        source["assertion_surface_contracts_sha256"],
        "assertion_surface_contracts_sha256",
    )
    if not SHA256.fullmatch(expected_digest):
        raise AssuranceError(
            "assertion_surface_contracts_sha256: expected 64 lowercase hex characters"
        )
    if expected_digest != ledger_digest:
        raise AssuranceError(
            "assertion_surface_contracts_sha256: assertion ledger drifted; "
            f"review premise classifications and update digest to {ledger_digest}"
        )
    as_text(source["title"], "title")

    top = validate_top_claim(source["top_claim"])
    if top["id"] != "RI-0":
        raise AssuranceError("top_claim.id: expected RI-0")
    meanings = as_object(source["status_meanings"], "status_meanings")
    exact_keys(meanings, POSTURES, "status_meanings")
    for posture, meaning in meanings.items():
        as_text(meaning, f"status_meanings.{posture}")

    dimensions = text_list(source["required_dimensions"], "required_dimensions")
    missing_dimensions = sorted(MANDATORY_DIMENSIONS - set(dimensions))
    extra_dimensions = sorted(set(dimensions) - MANDATORY_DIMENSIONS)
    if missing_dimensions or extra_dimensions:
        details: list[str] = []
        if missing_dimensions:
            details.append("missing " + ", ".join(missing_dimensions))
        if extra_dimensions:
            details.append("not mandatory here " + ", ".join(extra_dimensions))
        raise AssuranceError("required_dimensions: " + "; ".join(details))

    limitations = as_object(source["limitations"], "limitations")
    exact_keys(limitations, set(LIMITATION_KEYS), "limitations")
    for key, limitation in limitations.items():
        as_text(limitation, f"limitations.{key}")
    absence = str(limitations["in_snapshot_absence"]).lower()
    required_phrases = ("in-snapshot", "cannot distinguish", "deleted", "never written")
    absent_phrases = [phrase for phrase in required_phrases if phrase not in absence]
    if absent_phrases:
        raise AssuranceError(
            "limitations.in_snapshot_absence: must state that an in-snapshot rule "
            "cannot distinguish a deleted entry from one never written; missing "
            + ", ".join(absent_phrases)
        )

    boundary = as_object(source["boundary"], "boundary")
    exact_keys(boundary, {"book1", "book2"}, "boundary")
    text_list(boundary["book1"], "boundary.book1")
    book2 = text_list(boundary["book2"], "boundary.book2")
    book2_text = " ".join(book2).lower()
    handoff_markers = {
        "storage": ("storage",),
        "identity": ("identity",),
        "cryptographic": ("cryptograph", "signature", "credential", "key lifecycle"),
        "operational": ("operational", "operate", "operation"),
    }
    missing_handoffs = [
        label
        for label, alternatives in handoff_markers.items()
        if not any(marker in book2_text for marker in alternatives)
    ]
    if missing_handoffs:
        raise AssuranceError(
            "boundary.book2: must hand off storage, identity, cryptographic, and "
            "operational mechanisms; missing " + ", ".join(missing_handoffs)
        )

    raw_claims = as_list(source["claims"], "claims")
    if not raw_claims:
        raise AssuranceError("claims: must not be empty")
    forbidden_files = {
        DEFAULT_SOURCE.as_posix(),
        DEFAULT_OUTPUT.as_posix(),
        *(forbidden_evidence_files or set()),
    }
    claims = [
        validate_claim_shape(raw, index, forbidden_files)
        for index, raw in enumerate(raw_claims)
    ]
    claim_ids = [str(top["id"])] + [str(claim["id"]) for claim in claims]
    duplicates = sorted({identifier for identifier in claim_ids if claim_ids.count(identifier) > 1})
    if duplicates:
        raise AssuranceError("claims: duplicate claim ID(s): " + ", ".join(duplicates))
    known_claims = set(claim_ids)
    missing_claims = sorted(REQUIRED_CLAIM_IDS - known_claims)
    if missing_claims:
        raise AssuranceError(
            "claims: missing required claim ID(s): " + ", ".join(missing_claims)
        )
    claims_by_identifier = {str(claim["id"]): claim for claim in claims}
    wrong_refusals = sorted(
        claim_id
        for claim_id in REQUIRED_REFUSED_IDS
        if claims_by_identifier[claim_id]["posture"] != "refused_or_unprovable"
    )
    if wrong_refusals:
        raise AssuranceError(
            "claims: reviewed impossibility boundary must remain refused: "
            + ", ".join(wrong_refusals)
        )
    wrong_external = sorted(
        claim_id
        for claim_id in REQUIRED_EXTERNAL_IDS
        if claims_by_identifier[claim_id]["posture"]
        not in {"book2_external_assumption", "external_verified"}
    )
    if wrong_external:
        raise AssuranceError(
            "claims: reviewed external control must remain external: "
            + ", ".join(wrong_external)
        )
    covered_dimensions: set[str] = set()
    for index, claim in enumerate(claims):
        covered_dimensions.update(str(item) for item in claim["dimensions"])
        for evidence_index, raw_evidence in enumerate(claim["current_evidence"]):
            evidence = as_object(
                raw_evidence, f"claims[{index}].current_evidence[{evidence_index}]"
            )
            if evidence["supports"] != claim["id"]:
                raise AssuranceError(
                    f"claims[{index}].current_evidence[{evidence_index}].supports: "
                    f"must name its owning claim {claim['id']}"
                )
        validate_posture(claim, index)
    uncovered_dimensions = sorted(MANDATORY_DIMENSIONS - covered_dimensions)
    if uncovered_dimensions:
        raise AssuranceError(
            "claims: no assurance claim covers required dimension(s): "
            + ", ".join(uncovered_dimensions)
        )

    non_refused = [
        claim for claim in claims if claim["posture"] != "refused_or_unprovable"
    ]
    computed_verdict = (
        "established"
        if non_refused
        and all(
            claim["posture"] in {"current_verified", "external_verified"}
            for claim in non_refused
        )
        else "not_established"
    )
    if top["current_verdict"] != computed_verdict:
        raise AssuranceError(
            "top_claim.current_verdict: source says "
            f"{top['current_verdict']}, but claim postures require {computed_verdict}"
        )

    claims_by_id = {str(claim["id"]): claim for claim in claims}
    _, record_ids = validate_record_classes(
        source["record_classes"], set(claims_by_id), claims_by_id
    )
    missing_record_ids = sorted(REQUIRED_RECORD_IDS - record_ids)
    if missing_record_ids:
        raise AssuranceError(
            "record_classes: missing required ID(s): " + ", ".join(missing_record_ids)
        )
    temporal_claim = claims_by_id.get("RI-7")
    if temporal_claim is None:
        raise AssuranceError("claims: RI-7 must own the temporal transition gate")
    temporal_refs = {
        str(as_object(item, "RI-7 evidence")["ref"])
        for item in temporal_claim["current_evidence"]
    }
    if not any(
        ref.startswith("new-book-plans/book-1-time-model-decision.md::")
        for ref in temporal_refs
    ):
        raise AssuranceError("claims: RI-7 must cite the ratified time-model decision")
    if temporal_claim["owner_ref"] != (
        "new-book-plans/book-1-time-model-decision.md::"
        "## 7. Formal implementation and verification gate"
    ):
        raise AssuranceError("claims: RI-7 must remain owned by the ratified time-model gate")
    validate_premise_classes(source["premise_classes"], ledger, record_ids)
    validate_defeaters(source["defeaters"], known_claims)
    validate_fail_safe_defaults(source["fail_safe_defaults"], known_claims)
    validate_narrowness(source["narrowness_impacts"])
    validate_acceptance_gate(source["acceptance_gate"])


def markdown(value: object) -> str:
    return " ".join(str(value).split()).replace("|", "\\|")


def code(value: object) -> str:
    text = str(value)
    fence = "``" if "`" in text else "`"
    return f"{fence}{text}{fence}"


def bullets(lines: Sequence[object]) -> list[str]:
    return [f"- {markdown(line)}" for line in lines]


def render(
    source: dict[str, object],
    ledger_digest: str,
    source_path: pathlib.Path = DEFAULT_SOURCE,
    ledger_path: pathlib.Path = DEFAULT_LEDGER,
) -> str:
    top = as_object(source["top_claim"], "top_claim")
    claims = [as_object(value, "claim") for value in as_list(source["claims"], "claims")]
    classes = [
        as_object(value, "record class")
        for value in as_list(source["record_classes"], "record_classes")
    ]
    class_by_id = {str(entry["id"]): entry for entry in classes}
    limitations = as_object(source["limitations"], "limitations")
    boundary = as_object(source["boundary"], "boundary")
    meanings = as_object(source["status_meanings"], "status_meanings")
    lines = [
        f"<!-- SPDX-License-Identifier: {source['spdx']} -->",
        "<!-- Generated by new-book-plans/8-record-integrity-assurance.py; do not edit. -->",
        "",
        f"# {source['title']}",
        "",
        "## Verdict and scope",
        "",
        f"**{str(top['current_verdict']).replace('_', ' ').upper()} — {top['claim']}**",
        "",
        markdown(top["argument"]),
        "",
        "The verdict becomes **ESTABLISHED** only when every non-refused control is",
        "`current_verified` or `external_verified`. A refusal can mark an honest",
        "model boundary; it cannot",
        "silently satisfy a control that the design claims to provide.",
        "",
        "| posture | meaning |",
        "| --- | --- |",
    ]
    for posture in sorted(POSTURES):
        lines.append(f"| {code(posture)} | {markdown(meanings[posture])} |")

    lines.extend(
        [
            "",
            "## Limitations and Book 1/Book 2 boundary",
            "",
            "These are load-bearing limitations, not implementation notes:",
            "",
        ]
    )
    for key in LIMITATION_KEYS:
        lines.append(f"- **{markdown(key.replace('_', ' ').title())}:** {markdown(limitations[key])}")
    lines.extend(["", "### Book 1 owns", "", *bullets(boundary["book1"])])
    lines.extend(["", "### Book 2 owns", "", *bullets(boundary["book2"])])

    lines.extend(
        [
            "",
            "## Claim summary",
            "",
            "| claim | title | posture | assurance dimensions |",
            "| --- | --- | --- | --- |",
        ]
    )
    for claim in claims:
        dims = ", ".join(code(item) for item in claim["dimensions"])
        lines.append(
            f"| {code(claim['id'])} | {markdown(claim['title'])} | "
            f"{code(claim['posture'])} | {dims} |"
        )

    lines.extend(["", "## Claim details", ""])
    for claim in claims:
        lines.extend(
            [
                f"### {claim['id']} — {claim['title']}",
                "",
                f"**Claim.** {markdown(claim['claim'])}",
                "",
                f"**Argument.** {markdown(claim['argument'])}",
                "",
                f"- **Posture:** {code(claim['posture'])}",
                f"- **Current failure:** {markdown(claim['known_failure']) or 'None recorded.'}",
                f"- **Target contract:** {markdown(claim['target_contract']) or 'None.'}",
                f"- **Residual assumption:** {markdown(claim['residual_assumption']) or 'None.'}",
                f"- **Temporal status:** {markdown(claim['temporal_status']) or 'None.'}",
                f"- **Book 2 handoff:** {markdown(claim['book2_handoff']) or 'None.'}",
                f"- **Owner:** {code(claim['owner_ref'])}",
                "",
                "**Current evidence**",
                "",
            ]
        )
        evidence = claim["current_evidence"]
        if evidence:
            lines.extend(
                [
                    "| kind | evidence role | supports | reference |",
                    "| --- | --- | --- | --- |",
                ]
            )
            for raw_evidence in evidence:
                entry = as_object(raw_evidence, "current evidence")
                lines.append(
                    f"| {markdown(entry['kind'])} | {code(entry['role'])} | "
                    f"{code(entry['supports'])} | "
                    f"{code(entry['ref'])} |"
                )
        else:
            lines.append("None. This absence is part of the stated posture.")
        lines.extend(["", "**Acceptance evidence still required**", ""])
        acceptance = claim["acceptance_evidence"]
        if acceptance:
            lines.extend(bullets(acceptance))
        else:
            lines.append("None.")
        lines.append("")

    lines.extend(
        [
            "## Record classes and premise coverage",
            "",
            "Every writable premise in the reviewed assertion-surface ledger belongs",
            "to exactly one record class here. Classification does not authenticate an",
            "entry or make the chosen classes exhaustive; it selects the reviewed",
            "assurance argument that must govern it.",
            "",
        ]
    )
    for record_class in classes:
        owners = ", ".join(code(item) for item in record_class["assurance_claims"])
        lines.extend(
            [
                f"### {record_class['id']} — {record_class['title']}",
                "",
                markdown(record_class["description"]),
                "",
                f"- **Assurance claims:** {owners}",
                f"- **Failure posture:** {markdown(record_class['failure_posture'])}",
                "",
            ]
        )
    lines.extend(
        [
            "| writable premise | record class | class title |",
            "| --- | --- | --- |",
        ]
    )
    for premise, class_id in sorted(as_object(source["premise_classes"], "premise_classes").items()):
        lines.append(
            f"| {code(premise)} | {code(class_id)} | "
            f"{markdown(class_by_id[str(class_id)]['title'])} |"
        )

    lines.extend(["", "## Defeaters", ""])
    for raw_entry in as_list(source["defeaters"], "defeaters"):
        entry = as_object(raw_entry, "defeater")
        lines.extend(
            [
                f"### {entry['id']} — {entry['title']}",
                "",
                f"- **Attack:** {markdown(entry['attack'])}",
                f"- **Disposition:** {markdown(entry['disposition'])}",
                f"- **Owned by:** {', '.join(code(item) for item in entry['owner_claims'])}",
                f"- **If unresolved:** {markdown(entry['failure_consequence'])}",
                "",
            ]
        )

    lines.extend(["## Fail-safe defaults", ""])
    for raw_entry in as_list(source["fail_safe_defaults"], "fail_safe_defaults"):
        entry = as_object(raw_entry, "fail-safe default")
        lines.extend(
            [
                f"### {entry['id']}",
                "",
                f"- **Condition:** {markdown(entry['condition'])}",
                f"- **Required default:** {markdown(entry['required_default'])}",
                f"- **Reason:** {markdown(entry['rationale'])}",
                f"- **Owned by:** {', '.join(code(item) for item in entry['owner_claims'])}",
                "",
            ]
        )

    lines.extend(
        [
            "## Narrowness impacts",
            "",
            "These standing claims must be re-reviewed when the named trigger occurs,",
            "even if their own numbered chapter derivations do not change.",
            "",
            "| artifact | current claim | classification | reason | future trigger |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for raw_entry in as_list(source["narrowness_impacts"], "narrowness_impacts"):
        entry = as_object(raw_entry, "narrowness impact")
        lines.append(
            f"| {code(entry['artifact_ref'])} | {markdown(entry['current_claim'])} | "
            f"{markdown(entry['classification'])} | {markdown(entry['reason'])} | "
            f"{markdown(entry['future_trigger'])} |"
        )

    lines.extend(
        [
            "",
            "## Acceptance gate",
            "",
            "The top verdict stays **NOT ESTABLISHED** until every applicable item has",
            "reviewed evidence and the claim postures are updated without weakening a",
            "refusal or moving a Book 2 assumption into the constitutional kernel.",
            "",
            "| gate | requirement | evidence needed | owner |",
            "| --- | --- | --- | --- |",
        ]
    )
    for raw_entry in as_list(source["acceptance_gate"], "acceptance_gate"):
        entry = as_object(raw_entry, "acceptance gate")
        lines.append(
            f"| {code(entry['id'])} | {markdown(entry['requirement'])} | "
            f"{markdown(entry['evidence_needed'])} | {code(entry['owner_ref'])} |"
        )

    lines.extend(
        [
            "",
            "## Maintenance and limits",
            "",
            f"- Source: {code(source_path.as_posix())}.",
            f"- Assertion ledger: {code(ledger_path.as_posix())}, exact SHA-256 "
            f"{code(ledger_digest)}.",
            "- Regenerate after reviewing the JSON source; never hand-edit this report.",
            "- Run `python3 new-book-plans/8-record-integrity-assurance.py --check`.",
            "- The checker proves schema coverage, traceability, ledger coupling, and",
            "  report freshness. It does not prove real authorship, witness independence,",
            "  storage integrity, clock progress, omission recovery, or deletion recovery.",
            "- The bounded report at `new-book-plans/record-integrity-red-team.md`",
            "  executes selected release, adulthood, roster, relief, and forgiveness harms,",
            "  plus a negative control proving that bare `rotten` is inert. Those cases",
            "  expose flat-snapshot gaps and one input boundary; they do not establish",
            "  authorship, runtime attribution, recovery, liveness, or operational integrity,",
            "  and they do not duplicate the staged temporal assurance harness.",
            "",
        ]
    )
    return "\n".join(lines)


def expect_failure(label: str, action: Callable[[], object]) -> None:
    try:
        action()
    except AssuranceError:
        return
    raise AssuranceError(f"negative control did not fail: {label}")


def negative_controls(
    source: dict[str, object],
    ledger: dict[str, object],
    ledger_digest: str,
    forbidden_evidence_files: set[str] | None = None,
) -> int:
    controls = 0

    def validate(candidate: dict[str, object]) -> None:
        validate_source(
            candidate,
            ledger,
            ledger_digest,
            forbidden_evidence_files,
        )

    changed = copy.deepcopy(source)
    changed["assertion_surface_contracts_sha256"] = "0" * 64
    expect_failure("assertion-ledger drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    premise = sorted(as_object(changed["premise_classes"], "premise_classes"))[0]
    del changed["premise_classes"][premise]
    expect_failure("uncovered writable premise", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["required_dimensions"].remove("retention")
    expect_failure("missing lifecycle dimension", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    record_class = next(
        item for item in changed["record_classes"] if item["id"] == "RC-1"
    )
    record_class["assurance_claims"].remove("RI-5")
    expect_failure(
        "record class bypasses a mandatory dimension",
        lambda: validate(changed),
    )
    controls += 1

    changed = copy.deepcopy(source)
    changed["claims"].append(copy.deepcopy(changed["claims"][0]))
    expect_failure("duplicate claim ID", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    current = next(claim for claim in changed["claims"] if claim["id"] == "RI-1")
    current["owner_ref"] = "TODO.md::negative-control-anchor-does-not-exist"
    expect_failure("dangling evidence/owner reference", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    candidate = copy.deepcopy(
        next(claim for claim in changed["claims"] if claim["id"] == "RI-1")
    )
    candidate["posture"] = "current_verified"
    candidate["acceptance_evidence"] = []
    for evidence in candidate["current_evidence"]:
        evidence["role"] = "exposes_gap"
    expect_failure(
        "gap evidence promoted to current",
        lambda: validate_posture(candidate, 0),
    )
    controls += 1

    candidate = copy.deepcopy(
        next(claim for claim in source["claims"] if claim["id"] == "RI-10")
    )
    candidate["posture"] = "book2_external_assumption"
    candidate["book2_handoff"] = ""
    candidate["acceptance_evidence"] = ["Independent deployment evidence."]
    for evidence in candidate["current_evidence"]:
        evidence["role"] = "sets_boundary"
    expect_failure(
        "external assumption without Book 2 handoff",
        lambda: validate_posture(candidate, 0),
    )
    controls += 1

    candidate = copy.deepcopy(
        next(claim for claim in source["claims"] if claim["id"] == "RI-10")
    )
    candidate["posture"] = "external_verified"
    candidate["acceptance_evidence"] = []
    for evidence in candidate["current_evidence"]:
        evidence["role"] = "sets_boundary"
    expect_failure(
        "external claim promoted without operational evidence",
        lambda: validate_posture(candidate, 0),
    )
    controls += 1
    for evidence in candidate["current_evidence"]:
        evidence["role"] = "supports_external"
        evidence["kind"] = "operational"
    validate_posture(candidate, 0)

    candidate = copy.deepcopy(
        next(claim for claim in source["claims"] if claim["id"] == "RI-1")
    )
    candidate["posture"] = "refused_or_unprovable"
    candidate["acceptance_evidence"] = []
    for evidence in candidate["current_evidence"]:
        evidence["role"] = "sets_boundary"
    expect_failure(
        "implementable control disposition-washed as a refusal",
        lambda: validate_posture(candidate, 0),
    )
    controls += 1

    changed = copy.deepcopy(source)
    del changed["limitations"]["in_snapshot_absence"]
    expect_failure("missing deletion limitation", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["record_classes"][0]["assurance_claims"].append("RI-C-DOES-NOT-EXIST")
    expect_failure("dangling claim reference", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    first_premise = sorted(changed["premise_classes"])[0]
    changed["premise_classes"][first_premise] = "RI-R-DOES-NOT-EXIST"
    expect_failure("dangling record-class reference", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["top_claim"]["current_verdict"] = (
        "established"
        if changed["top_claim"]["current_verdict"] == "not_established"
        else "not_established"
    )
    expect_failure("top verdict inconsistent with postures", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["schema_version"] = 999
    expect_failure("unknown schema version", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    current = next(claim for claim in changed["claims"] if claim["id"] == "RI-1")
    current["title"] = "pending"
    expect_failure("blank or placeholder claim prose", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["defeaters"][0]["disposition"] = "banana"
    expect_failure("unknown defeater disposition", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    current = next(claim for claim in changed["claims"] if claim["id"] == "RI-1")
    current["current_evidence"][0]["ref"] = (
        "new-book-plans/record-integrity-assurance-case.json::\"schema_version\": 1"
    )
    expect_failure("assurance case used as self-evidence", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["claims"] = [
        claim for claim in changed["claims"] if claim["id"] != "RI-12"
    ]
    for record_class in changed["record_classes"]:
        if "RI-12" in record_class["assurance_claims"]:
            record_class["assurance_claims"].remove("RI-12")
    expect_failure("required claim deleted", lambda: validate(changed))
    controls += 1

    required_deletions = (
        ("defeaters", "id", "RD-1", "required defeater deleted"),
        ("fail_safe_defaults", "id", "RF-1", "required fail-safe deleted"),
        ("acceptance_gate", "id", "RA-1", "required acceptance gate deleted"),
        (
            "narrowness_impacts",
            "artifact_ref",
            "book-1/01-what-counts-as-evidence.md::Neither mechanism can find an encounter nobody reports, detect a",
            "standing narrowness review deleted",
        ),
    )
    for key, field, required_value, label in required_deletions:
        changed = copy.deepcopy(source)
        changed[key] = [
            item for item in changed[key] if item[field] != required_value
        ]
        expect_failure(label, lambda changed=changed: validate(changed))
        controls += 1

    changed = copy.deepcopy(source)
    temporal = next(claim for claim in changed["claims"] if claim["id"] == "RI-7")
    current = next(claim for claim in changed["claims"] if claim["id"] == "RI-1")
    temporal["owner_ref"] = current["owner_ref"]
    expect_failure("T3 transition owner bypassed", lambda: validate(changed))
    controls += 1

    expect_failure(
        "duplicate JSON object key",
        lambda: json.loads(
            '{"premise_classes": {}, "premise_classes": {}}',
            object_pairs_hook=reject_duplicate_keys,
        ),
    )
    controls += 1

    return controls


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise AssuranceError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json(path: pathlib.Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_keys,
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise AssuranceError(f"cannot read {label} {path}: {exc}") from exc
    return as_object(value, label)


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    parser.add_argument("--ledger", type=pathlib.Path, default=DEFAULT_LEDGER)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)

    source_path = resolve(args.source)
    ledger_path = resolve(args.ledger)
    output_path = resolve(args.output)
    source_relative = repo_relative(source_path)
    ledger_relative = repo_relative(ledger_path)
    output_relative = repo_relative(output_path)
    if output_path.resolve() in {source_path.resolve(), ledger_path.resolve()}:
        raise AssuranceError("output path must not overwrite the source or assertion ledger")
    if output_path.suffix.lower() != ".md":
        raise AssuranceError("output path must end in .md")
    forbidden_evidence_files = {
        source_relative.as_posix(),
        output_relative.as_posix(),
    }
    source = load_json(source_path, "assurance source")
    ledger = load_json(ledger_path, "assertion ledger")
    ledger_digest = sha256(ledger_path)
    validate_source(source, ledger, ledger_digest, forbidden_evidence_files)
    generated = render(source, ledger_digest, source_relative, ledger_relative)

    if args.check:
        controls = negative_controls(
            source,
            ledger,
            ledger_digest,
            forbidden_evidence_files,
        )
        try:
            current = output_path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise AssuranceError(f"cannot read generated report {output_path}: {exc}") from exc
        if current != generated:
            try:
                display = output_path.resolve().relative_to(ROOT.resolve())
            except ValueError:
                display = output_path
            raise AssuranceError(f"{display} is STALE — rerun without --check")
        print(
            f"{output_relative} is current; "
            f"{controls} negative controls pass"
        )
        return 0

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(generated, encoding="utf-8", newline="\n")
    print(f"{output_relative}: regenerated")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssuranceError as exc:
        print(f"8-record-integrity-assurance: {exc}", file=sys.stderr)
        raise SystemExit(1)
