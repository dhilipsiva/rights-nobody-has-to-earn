#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Deterministic admission gate for Book 1 reader evidence.

The gate accepts one canonical JSON envelope on standard input.  It independently
recomputes the holdout verdict from the frozen rule and coded public records,
checks the evidence bindings, and emits one canonical admit/reject receipt.
External custody truth remains outside this executable contract.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import pathlib
import re
import sys
from typing import Mapping


ROOT = pathlib.Path(__file__).resolve().parents[1]
CHECKER = ROOT / "new-book-plans/14-reader-evidence.py"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
OPAQUE_ID = re.compile(r"^RE-[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)+$")

INPUT_KEYS = {
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
}
OUTPUT_KEYS = {
    "schema_version",
    "input_sha256",
    "evidence_gate_sha256",
    "decision",
    "receipt_sha256",
}


class GateError(RuntimeError):
    """The admission envelope or gate execution is invalid."""


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise GateError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    if missing or extra:
        detail = []
        if missing:
            detail.append("missing " + ", ".join(missing))
        if extra:
            detail.append("unexpected " + ", ".join(extra))
        raise GateError(f"{path}: {'; '.join(detail)}")


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise GateError(f"{path}: expected an object with string keys")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise GateError(f"{path}: expected an array")
    return value


def sha(value: object, path: str, expected: str | None = None) -> str:
    if not isinstance(value, str) or not SHA256.fullmatch(value):
        raise GateError(f"{path}: expected a lowercase SHA-256 digest")
    if expected is not None and value != expected:
        raise GateError(f"{path}: digest mismatch")
    return value


def canonical_bytes(value: object) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def canonical_sha(value: object, *, omit: str | None = None) -> str:
    if omit is not None:
        copied = dict(as_object(value, "canonical digest object"))
        copied.pop(omit, None)
        value = copied
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def file_sha(path: pathlib.Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as exc:
        raise GateError(f"cannot read bound executable {path}: {exc}") from exc


def load_checker():
    spec = importlib.util.spec_from_file_location(
        "reader_evidence_checker_for_gate", CHECKER
    )
    if spec is None or spec.loader is None:
        raise GateError("cannot load the bound reader-evidence evaluator")
    module = importlib.util.module_from_spec(spec)
    previous = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec.loader.exec_module(module)
    except Exception as exc:
        raise GateError(f"cannot load the bound reader-evidence evaluator: {exc}") from exc
    finally:
        sys.dont_write_bytecode = previous
    return module


def evaluate_reader_evidence(value: object) -> dict[str, object]:
    envelope = as_object(value, "gate input")
    exact_keys(envelope, INPUT_KEYS, "gate input")
    if envelope["schema_version"] != 1 or isinstance(
        envelope["schema_version"], bool
    ):
        raise GateError("gate input.schema_version must be integer 1")
    attempt_id = envelope["attempt_id"]
    if not isinstance(attempt_id, str) or not OPAQUE_ID.fullmatch(attempt_id):
        raise GateError("gate input.attempt_id must be an opaque RE-* identifier")
    if not isinstance(envelope["active_attempt"], bool):
        raise GateError("gate input.active_attempt must be boolean")
    if envelope["attempt_status"] != "completed":
        raise GateError("gate input.attempt_status must be completed")

    rule = as_object(envelope["threshold_rule"], "gate input.threshold_rule")
    ratification = as_object(
        envelope["frozen_ratification"], "gate input.frozen_ratification"
    )
    registration = as_object(
        envelope["pre_registration"], "gate input.pre_registration"
    )
    sessions = as_list(envelope["session_records"], "gate input.session_records")
    deviations = as_list(envelope["deviations"], "gate input.deviations")
    custody = as_list(
        envelope["custody_attestations"], "gate input.custody_attestations"
    )
    receipt = as_object(envelope["result_receipt"], "gate input.result_receipt")

    gate_digest = sha(
        envelope["evidence_gate_sha256"],
        "gate input.evidence_gate_sha256",
        file_sha(pathlib.Path(__file__).resolve()),
    )
    checker_digest = sha(
        envelope["structural_checker_sha256"],
        "gate input.structural_checker_sha256",
        file_sha(CHECKER),
    )
    raw_sessions = sessions
    raw_deviations = deviations
    raw_custody = custody
    checker = load_checker()
    try:
        validated_rule, known_misconceptions = checker.validate_threshold_rule(
            {
                "threshold_status": "author-ratified",
                "threshold_rule": rule,
                "ratification": ratification,
            },
            True,
        )
        if validated_rule != rule:
            raise GateError(
                "gate input threshold rule changed during validation"
            )
        fixed_protocol_sha256 = checker.sha(
            registration.get("fixed_protocol_sha256"),
            "gate input.pre_registration.fixed_protocol_sha256",
        )
        frozen_ratification = checker.validate_frozen_ratification(
            ratification,
            "gate input.frozen_ratification",
            rule,
            fixed_protocol_sha256,
        )
        registration = checker.validate_holdout_pre_registration(
            registration,
            "gate input.pre_registration",
            verify_live=False,
            fixed_protocol_sha256=fixed_protocol_sha256,
            expected_structural_checker_sha256=checker_digest,
        )
        checker.sha(
            registration["rule_sha256"],
            "gate input.pre_registration.rule_sha256",
            str(rule["rule_sha256"]),
        )
        checker.sha(
            registration["ratification_sha256"],
            "gate input.pre_registration.ratification_sha256",
            str(frozen_ratification["ratification_sha256"]),
        )
        checker.sha(
            registration["evidence_gate_sha256"],
            "gate input.pre_registration.evidence_gate_sha256",
            gate_digest,
        )
        checker.sha(
            envelope["current_rule_sha256"],
            "gate input.current_rule_sha256",
            str(rule["rule_sha256"]),
        )
        checker.sha(
            envelope["current_ratification_sha256"],
            "gate input.current_ratification_sha256",
            str(frozen_ratification["ratification_sha256"]),
        )

        study_id = str(registration["study_id"])
        sessions = checker.validate_session_records(
            raw_sessions,
            "gate input.session_records",
            expected_study_id=study_id,
            known_misconceptions=known_misconceptions,
        )
        deviations = checker.validate_deviations(
            raw_deviations, "gate input.deviations"
        )
        custody = checker.validate_custody(
            raw_custody, "gate input.custody_attestations"
        )
        commitment = (
            checker.as_object(
                registration["commitment"],
                "gate input.pre_registration.commitment",
            )
            if registration["commitment"] is not None
            else None
        )
        checker.validate_record_links(
            sessions,
            deviations,
            custody,
            "gate input.record_links",
            expected_study_id=study_id,
            commitment=commitment,
        )
        receipt = checker.validate_result_receipt(
            receipt,
            "gate input.result_receipt",
            registration,
            rule,
            sessions,
            raw_deviations,
            raw_custody,
        )
        completed_at = str(receipt["completed_at"])
        freeze_binding = checker.as_object(
            registration["freeze_binding"],
            "gate input.pre_registration.freeze_binding",
        )
        freeze_at = str(freeze_binding["frozen_at"])
        if completed_at <= freeze_at:
            raise GateError(
                "gate input completion must strictly follow the frozen preregistration"
            )
        if completed_at[:10] < str(registration["registered_date"]):
            raise GateError(
                "gate input completion cannot precede the registration date"
            )
        checker.validate_commitment_reveal(
            envelope["commitment_reveal"],
            "gate input.commitment_reveal",
            commitment,
            custody,
            "completed",
            reveal_required=True,
            verify_live=False,
            terminal_at=completed_at,
        )
        trace = checker.evaluate_holdout(
            rule, sessions, str(receipt["protocol_validity"])
        )
    except GateError:
        raise
    except Exception as exc:
        raise GateError(
            f"bound reader-evidence validation failed closed: {exc}"
        ) from exc

    freshness = [
        item
        for item in custody.values()
        if item["scope"] == "study-freshness"
    ]
    admitted = (
        envelope["active_attempt"] is True
        and receipt["protocol_validity"] == "valid"
        and receipt["verdict"] == "pass"
        and trace["verdict"] == "pass"
        and len(freshness) == 1
        and freshness[0]["freshness_attested"] is True
    )

    output: dict[str, object] = {
        "schema_version": 1,
        "input_sha256": canonical_sha(envelope),
        "evidence_gate_sha256": gate_digest,
        "decision": "admit" if admitted else "reject",
        "receipt_sha256": "",
    }
    output["receipt_sha256"] = canonical_sha(output, omit="receipt_sha256")
    exact_keys(output, OUTPUT_KEYS, "gate output")
    return output


def self_test() -> None:
    malformed = {"schema_version": 1}
    try:
        evaluate_reader_evidence(malformed)
    except GateError:
        pass
    else:
        raise GateError("malformed-envelope self-test did not fail closed")

    try:
        sha(
            "0" * 64,
            "self-test.structural_checker_sha256",
            file_sha(CHECKER),
        )
    except GateError:
        pass
    else:
        raise GateError(
            "dependency-digest mismatch self-test did not fail closed"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group(required=True)
    modes.add_argument("--self-test", action="store_true")
    modes.add_argument("--evaluate", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("reader-evidence-admission-gate: self-test passed")
        return 0
    try:
        raw = sys.stdin.buffer.read()
        if not raw:
            raise GateError("--evaluate requires one JSON object on standard input")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=reject_duplicate_keys)
        output = evaluate_reader_evidence(value)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise GateError(f"invalid gate input JSON: {exc}") from exc
    sys.stdout.buffer.write(canonical_bytes(output) + b"\n")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GateError as exc:
        print(f"reader-evidence-admission-gate: {exc}", file=sys.stderr)
        raise SystemExit(1)
