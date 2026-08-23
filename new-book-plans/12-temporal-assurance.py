#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate, execute, and render the staged T1/T2/T3 assurance case.

The reviewed JSON owns the temporal-input contracts, exact source/effect
binding, fresh-process predecessor/successor fixtures, adversarial matrix,
narrowness manifest, and residual boundary.  The executable path constructs
cumulative T1, T2, and T3 sources from marked regions of the live
constitution and runs every fixture in a fresh release ``nibli-pin`` process.

This is repository evidence for the exact bound source.  It establishes
fail-safe derivations for supplied, independently represented records.  It
does not authenticate the outside witnesses, make a successor arrive, advance
a clock, release a person physically, or make an institution perform a duty.
Every executable pin is ground. The coexistence control includes the opaque
floor-event surface alongside standing, authority, custody, recognition,
severity, and the bounded temporal identifier-status heads.

Usage:
    python3 new-book-plans/12-temporal-assurance.py
    python3 new-book-plans/12-temporal-assurance.py --check
    python3 new-book-plans/12-temporal-assurance.py --check --execute
    python3 new-book-plans/12-temporal-assurance.py --fingerprints
"""

from __future__ import annotations

import argparse
import concurrent.futures
import copy
import hashlib
import json
import os
import pathlib
import re
import shutil
import signal
import subprocess
import sys
import tempfile
from collections.abc import Iterable, Mapping, Sequence
from typing import Callable


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("new-book-plans/temporal-assurance-case.json")
DEFAULT_KB = pathlib.Path("new-book-plans/constitution.nibli")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/temporal-assurance-case.md")

BOUND_SOURCES = {
    "time_model": pathlib.Path("new-book-plans/book-1-time-model-decision.md"),
    "assertion_surface": pathlib.Path("new-book-plans/assertion-surface-contracts.json"),
    "record_assurance": pathlib.Path("new-book-plans/record-integrity-assurance-case.json"),
    "record_red_team": pathlib.Path("new-book-plans/record-integrity-red-team.json"),
    "amendment_semantics": pathlib.Path("new-book-plans/amendment-semantics-audit.json"),
    "placement_exhaustiveness": pathlib.Path("new-book-plans/placement-exhaustiveness-audit.json"),
}
STAGES = ("T1", "T2", "T3")
STAGE_MARKERS = {
    "T1": ("T1-ADMISSIONS", "T1-DERIVED", "T1-RULES", "T1-FACTS"),
    "T2": ("T2-DERIVED", "T2-RULES", "T2-FACTS"),
    "T3": (
        "T3-DERIVED",
        "T3-RULES",
        "T3-FACTS",
        "T3-COURT-GATE",
        "T3-LEASE-RULE",
    ),
}
ALL_MARKERS = tuple(marker for stage in STAGES for marker in STAGE_MARKERS[stage])
REQUIRED_SOURCE_SENTINELS = (
    'derived_only("collide").',
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
)
FORBIDDEN_TEMPORAL_HEADS = (
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
)
FORBIDDEN_CASE_SCOPE_TOKENS = {
    "EventOrderScope",
    "RecordOrderScope",
    "WindowScope",
    "SourceScope",
    "SourceLeaseScope",
}
COLLISION_TAGS = {
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
}
ALLOWED_MULTI_VARIABLE_HEADS = (
    r"precede\(\$[A-Za-z_][A-Za-z0-9_]*, \$[A-Za-z_][A-Za-z0-9_]*, EventPath\)\.",
    r"precede\(\$[A-Za-z_][A-Za-z0-9_]*, \$[A-Za-z_][A-Za-z0-9_]*, RecordPath\)\.",
)
REQUIRED_CASE_SCOPES = {
    "TA-09": {"ManifestScope"},
    "TA-13": {
        "EventStartScope",
        "EventEndScope",
        "RecordStartScope",
        "RecordEndScope",
    },
    "TA-14": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
        "LimitScope",
        "RenewalScope",
    },
    "TA-15": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
    },
    "TA-16": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
        "LimitScope",
        "RenewalScope",
    },
    "TA-17": {
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
    },
    "TA-18": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
        "LimitScope",
        "RenewalScope",
    },
    "TA-19": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
        "LimitScope",
        "RenewalScope",
    },
    "TA-25": {"VoidScope", "ClearScope"},
    "TA-26": {
        "EventStartScope",
        "EventEndScope",
        "RecordStartScope",
        "RecordEndScope",
    },
    "TA-27": {"PowerScope"},
    "TA-28": {"HolderScope"},
    "TA-29": {"SourceEpochScope"},
    "TA-33": {"ManifestScope"},
    "TA-34": {"SourceVersionScope"},
    "TA-35": {"WindowEndScope"},
    "TA-36": {
        "EventStartScope",
        "EventEndScope",
        "RecordStartScope",
        "RecordEndScope",
    },
    "TA-37": {"JudgmentScope"},
    "TA-38": {"InjuryVictimScope"},
    "TA-39": {"CaseScope"},
    "TA-40": {"LineageVersionScope"},
    "TA-41": {
        "CaseScope",
        "HolderScope",
        "JudgmentScope",
        "InjuryVictimScope",
        "PowerScope",
        "CaseBindingScope",
        "LimitScope",
        "RenewalScope",
    },
}
REQUIRED_INPUTS = {
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
}
INPUT_FIELDS = {
    "id",
    "stage",
    "writer",
    "evidence",
    "forge_route",
    "withholding_route",
    "correction",
    "appeal",
    "cross_epoch_handoff",
    "residual_external_assurance",
    "book_boundary",
}
REQUIRED_ATTACKS = {
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
    "unread_duty",
    "external_liveness",
    "standing_witness_withholding",
    "challenge_intake_withholding",
}
REQUIRED_NARROWNESS = {
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
}
ROOT_KEYS = {
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "subprocess_timeout_seconds",
    "constitution_sha256",
    "bound_sources_sha256",
    "marker_sha256",
    "stage_source_sha256",
    "source_effect_binding",
    "pre_t3_custody_rule",
    "temporal_input_contracts",
    "stages",
    "cases",
    "fresh_process_pairs",
    "attacks",
    "narrowness_impacts",
    "limits",
    "acceptance_result",
}
SOURCE_BINDING_KEYS = {
    "effective_version",
    "case_bound_rule_fragment",
    "case_bound_rule_sha256",
    "source_binding_fragment",
    "source_binding_sha256",
    "semantic_effect",
    "compatibility_review",
    "article9_boundary",
}
STAGE_KEYS = {"id", "cumulative", "claim", "does_not_establish"}
CASE_KEYS = {
    "id",
    "stage",
    "title",
    "process_role",
    "description",
    "deletions",
    "additions",
    "checks",
}
CHECK_KEYS = {"expression", "expected", "purpose"}
PAIR_KEYS = {"id", "predecessor_case", "successor_case", "purpose"}
ATTACK_KEYS = {"id", "stage", "control", "case_refs", "posture", "boundary"}
NARROWNESS_KEYS = {
    "artifact_ref",
    "standing_claim",
    "classification",
    "test_route",
    "change_trigger",
}
LIMIT_KEYS = {
    "safety_not_liveness",
    "snapshot_scope",
    "external_time",
    "physical_effect",
    "institutional_action",
    "record_truth",
    "query_shape",
    "source_transition",
    "trust_root",
}
ACCEPTANCE_KEYS = {"result", "claim", "does_not_establish", "remaining_boundary"}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
IDENTIFIER = re.compile(r"^[A-Za-z][A-Za-z0-9_-]*$")
GROUND = re.compile(r"^[a-z][a-z0-9_]*\([^\n]*\)\.$")
GROUND_QUERY = re.compile(r"^[a-z][a-z0-9_]*\([A-Za-z0-9_, ]+\)$")
EVENT_ENTITLEMENT_QUERY = re.compile(
    r"^entitled\([A-Za-z][A-Za-z0-9_]*, event \{ [a-z][a-z0-9_]*\(\) \}\)$"
)
VERDICTS = {"TRUE", "FALSE"}
REQUIRED_BOUNDARY_VERDICTS = {
    ("TA-10", "person(TA_Unregistered)"): "TRUE",
    ("TA-10", "err(TA_Unregistered, StandingOmission)"): "TRUE",
    ("TA-10", "person(TA_Unwitnessed_Standing)"): "FALSE",
    ("TA-10", "err(TA_Unwitnessed_Standing, StandingOmission)"): "FALSE",
    ("TA-41", "collide(TA_Challenge_Case, LeaseSuspended)"): "TRUE",
    ("TA-41", "correct(TA_Challenge_Case, ActivePower)"): "FALSE",
    ("TA-41", "match(TA_Challenge_Case, ActivePower)"): "FALSE",
    ("TA-41", "prisoner(TA_Challenge_Subject)"): "FALSE",
    ("TA-41", "err(TA_Challenge_Subject, TemporalDispute)"): "TRUE",
    ("TA-41", "obliged(Appeals, TA_Challenge_Subject)"): "TRUE",
    ("TA-41", "free(TA_Challenge_Subject)"): "FALSE",
    ("TA-41", "person(TA_Challenge_Subject)"): "TRUE",
}
REQUIRED_BOUNDARY_PAIR = {
    "id": "TP-12",
    "predecessor_case": "TA-41",
    "successor_case": "TA-14",
}
REQUIRED_BOUNDARY_ATTACKS = {
    "standing_witness_withholding": {
        "stage": "T1",
        "case_refs": ["TA-10"],
        "posture": "exposed_external_boundary",
    },
    "challenge_intake_withholding": {
        "stage": "T3",
        "case_refs": ["TA-41", "TA-14"],
        "posture": "exposed_external_boundary",
    },
}
POSTURES = {
    "rejected_by_current_source",
    "detected_as_named_error",
    "preserved_by_narrowness_test",
    "exposed_external_boundary",
}
CLASSIFICATIONS = {"revised_and_pinned", "preserved_and_pinned", "boundary_rewritten"}


class TemporalAssuranceError(RuntimeError):
    """Invalid reviewed source, stale report, or failed executable control."""


def resolve(path: pathlib.Path) -> pathlib.Path:
    return path if path.is_absolute() else ROOT / path


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve().relative_to(ROOT.resolve())
    except (OSError, ValueError) as exc:
        raise TemporalAssuranceError(f"path escapes repository: {path}") from exc


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


def read_bytes(path: pathlib.Path, label: str) -> bytes:
    if path.is_symlink():
        raise TemporalAssuranceError(f"{label} may not be a symlink: {path}")
    try:
        value = path.read_bytes()
    except OSError as exc:
        raise TemporalAssuranceError(f"cannot read {label}: {exc}") from exc
    if b"\r" in value:
        raise TemporalAssuranceError(f"{label} must use LF line endings")
    return value


def decode(value: bytes, label: str) -> str:
    try:
        text = value.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise TemporalAssuranceError(f"{label} is not UTF-8: {exc}") from exc
    if text.encode("utf-8") != value:
        raise TemporalAssuranceError(f"{label} is not canonical UTF-8")
    return text


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise TemporalAssuranceError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json(value: bytes, label: str) -> dict[str, object]:
    try:
        parsed = json.loads(decode(value, label), object_pairs_hook=reject_duplicate_keys)
    except json.JSONDecodeError as exc:
        raise TemporalAssuranceError(f"cannot parse {label}: {exc}") from exc
    if not isinstance(parsed, dict):
        raise TemporalAssuranceError(f"{label} must be an object")
    return parsed


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    actual = set(value)
    if actual != expected:
        raise TemporalAssuranceError(
            f"{path} keys differ: missing {sorted(expected - actual)}, extra {sorted(actual - expected)}"
        )


def object_value(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise TemporalAssuranceError(f"{path} must be an object")
    return value


def list_value(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise TemporalAssuranceError(f"{path} must be a list")
    return value


def text_value(value: object, path: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise TemporalAssuranceError(f"{path} must be non-empty text")
    if re.fullmatch(r"(?i)(?:todo|tbd|pending|unknown|n/?a|placeholder)", value.strip()):
        raise TemporalAssuranceError(f"{path} contains a placeholder")
    return value


def string_list(value: object, path: str, *, allow_empty: bool = False) -> list[str]:
    items = list_value(value, path)
    if not items and not allow_empty:
        raise TemporalAssuranceError(f"{path} may not be empty")
    result = [text_value(item, f"{path}[{index}]") for index, item in enumerate(items)]
    if len(result) != len(set(result)):
        raise TemporalAssuranceError(f"{path} contains duplicates")
    return result


def checked_sha(value: object, path: str, expected: str | None = None) -> str:
    digest = text_value(value, path)
    if SHA256.fullmatch(digest) is None:
        raise TemporalAssuranceError(f"{path} must be a lowercase SHA-256")
    if expected is not None and digest != expected:
        raise TemporalAssuranceError(f"{path} is stale: reviewed {digest}, actual {expected}")
    return digest


def marker_block(source: str, marker: str) -> str:
    begin = f"# <{marker}-BEGIN>\n"
    end = f"# <{marker}-END>\n"
    if source.count(begin) != 1 or source.count(end) != 1:
        raise TemporalAssuranceError(f"source must contain exactly one {marker} marker pair")
    start = source.index(begin)
    stop = source.index(end, start) + len(end)
    if stop <= start:
        raise TemporalAssuranceError(f"invalid {marker} marker order")
    return source[start:stop]


def replace_once(source: str, before: str, after: str, label: str) -> str:
    count = source.count(before)
    if count != 1:
        raise TemporalAssuranceError(f"{label}: expected exact fragment once, found {count}")
    return source.replace(before, after, 1)


def build_stages(source: str, pre_t3_rule: str) -> dict[str, str]:
    for marker in ALL_MARKERS:
        marker_block(source, marker)
    gate = marker_block(source, "T3-COURT-GATE")
    legacy = (
        "# Staged temporal assurance: T3 custody gate intentionally absent.\n"
        + pre_t3_rule.rstrip()
        + "\n"
    )
    result: dict[str, str] = {"T3": source}
    t2 = source
    for marker in ("T3-DERIVED", "T3-RULES", "T3-FACTS"):
        t2 = replace_once(t2, marker_block(t2, marker), "", f"T2 remove {marker}")
    t2 = replace_once(t2, gate, legacy, "T2 replace custody gate")
    result["T2"] = t2
    t1 = t2
    for marker in ("T2-DERIVED", "T2-RULES", "T2-FACTS"):
        t1 = replace_once(t1, marker_block(t1, marker), "", f"T1 remove {marker}")
    result["T1"] = t1
    if not (len(result["T1"]) < len(result["T2"]) < len(result["T3"])):
        raise TemporalAssuranceError("cumulative stage sources do not grow T1 -> T2 -> T3")
    return result


def apply_case(stage_source: str, case: Mapping[str, object]) -> str:
    candidate = stage_source
    for deletion in string_list(case["deletions"], f"case {case['id']}.deletions", allow_empty=True):
        candidate = replace_once(candidate, deletion + "\n", "", f"case {case['id']} deletion")
    additions = string_list(case["additions"], f"case {case['id']}.additions", allow_empty=True)
    if additions:
        candidate += "\n# Temporal-assurance case overlay (generated; never enacted).\n"
        candidate += "\n".join(additions) + "\n"
    return candidate


def validate_ground_facts(values: object, path: str) -> list[str]:
    facts = string_list(values, path, allow_empty=True)
    for fact in facts:
        if GROUND.fullmatch(fact) is None or "$" in fact or "->" in fact:
            raise TemporalAssuranceError(f"{path}: not a ground fact: {fact!r}")
    return facts


def core_fingerprints(source: str, dependencies: Mapping[str, bytes], pre_t3_rule: str) -> dict[str, object]:
    stages = build_stages(source, pre_t3_rule)
    return {
        "constitution_sha256": sha256_text(source),
        "bound_sources_sha256": {key: sha256_bytes(dependencies[key]) for key in sorted(dependencies)},
        "marker_sha256": {marker: sha256_text(marker_block(source, marker)) for marker in ALL_MARKERS},
        "stage_source_sha256": {stage: sha256_text(stages[stage]) for stage in STAGES},
    }


def validate_source(
    reviewed: Mapping[str, object],
    constitution: str,
    dependencies: Mapping[str, bytes],
) -> tuple[dict[str, str], list[dict[str, object]]]:
    exact_keys(reviewed, ROOT_KEYS, "root")
    if reviewed["spdx"] != "CC-BY-4.0" or reviewed["schema_version"] != 1:
        raise TemporalAssuranceError("reviewed source must be CC-BY-4.0 schema version 1")
    if reviewed["status"] != "staged_t3_repository_assurance":
        raise TemporalAssuranceError("unexpected assurance status")
    if reviewed["evidence_role"] != "current_verified_narrowly":
        raise TemporalAssuranceError("temporal assurance must remain narrowly current-verified")
    timeout = reviewed["subprocess_timeout_seconds"]
    if not isinstance(timeout, int) or isinstance(timeout, bool) or not 1 <= timeout <= 600:
        raise TemporalAssuranceError("subprocess_timeout_seconds must be an integer from 1 to 600")
    text_value(reviewed["title"], "title")
    pre_t3_rule = text_value(reviewed["pre_t3_custody_rule"], "pre_t3_custody_rule")
    if "-> prisoner($offender)." not in pre_t3_rule or any(
        token in pre_t3_rule for token in ("correct(", "precede(", "time(", "ActiveCustody")
    ):
        raise TemporalAssuranceError("pre-T3 custody rule is not the exact non-temporal fallback")
    if 'admits("record")' in constitution:
        raise TemporalAssuranceError(
            "generic record admission is forbidden in the narrowed temporal source"
        )
    if "correct($after, Transition" in constitution:
        raise TemporalAssuranceError(
            "transition acceptance must use bounded succeed/2, not recursive correct/4"
        )
    unbounded_heads = [
        fragment for fragment in FORBIDDEN_TEMPORAL_HEADS if fragment in constitution
    ]
    if unbounded_heads:
        raise TemporalAssuranceError(
            "temporal derived heads violate the reviewed typed-status/path shapes: "
            + ", ".join(unbounded_heads)
        )
    for marker in ("T1-RULES", "T2-RULES", "T3-RULES", "T3-COURT-GATE"):
        for line in marker_block(constitution, marker).splitlines():
            if not line.startswith("all ") or "->" not in line:
                continue
            head = line.rsplit("->", 1)[1]
            variables = set(re.findall(r"\$[A-Za-z_][A-Za-z0-9_]*", head))
            normalized_head = head.strip()
            if len(variables) > 1 and not any(
                re.fullmatch(pattern, normalized_head)
                for pattern in ALLOWED_MULTI_VARIABLE_HEADS
            ):
                raise TemporalAssuranceError(
                    f"{marker} has an unreviewed multi-variable head: {normalized_head}"
                )
            if normalized_head.startswith("collide("):
                collision = re.fullmatch(
                    r"collide\((?:\$[A-Za-z_][A-Za-z0-9_]*|[A-Za-z][A-Za-z0-9_]*), ([A-Za-z][A-Za-z0-9_]*)\)\.",
                    normalized_head,
                )
                if collision is None or collision.group(1) not in COLLISION_TAGS:
                    raise TemporalAssuranceError(
                        f"{marker} has an unreviewed collision head: {normalized_head}"
                    )
    succeed_closure = 'derived_only("succeed").'
    if (
        marker_block(constitution, "T1-DERIVED").count(succeed_closure) != 1
        or succeed_closure in marker_block(constitution, "T3-DERIVED")
    ):
        raise TemporalAssuranceError(
            "succeed must be derived-only at T1 so every cumulative stage is closed"
        )
    missing_source_sentinels = [
        sentinel
        for sentinel in REQUIRED_SOURCE_SENTINELS
        if sentinel not in constitution
    ]
    if missing_source_sentinels:
        raise TemporalAssuranceError(
            "exact temporal source sentinels missing: "
            + ", ".join(missing_source_sentinels)
        )
    fingerprints = core_fingerprints(constitution, dependencies, pre_t3_rule)
    checked_sha(reviewed["constitution_sha256"], "constitution_sha256", str(fingerprints["constitution_sha256"]))
    bound = object_value(reviewed["bound_sources_sha256"], "bound_sources_sha256")
    if set(bound) != set(BOUND_SOURCES):
        raise TemporalAssuranceError("bound_sources_sha256 must bind every reviewed dependency")
    for key, actual in object_value(fingerprints["bound_sources_sha256"], "fingerprints.bound_sources").items():
        checked_sha(bound[key], f"bound_sources_sha256.{key}", str(actual))
    marker_hashes = object_value(reviewed["marker_sha256"], "marker_sha256")
    if set(marker_hashes) != set(ALL_MARKERS):
        raise TemporalAssuranceError("marker_sha256 must bind every staged source marker")
    for marker, actual in object_value(fingerprints["marker_sha256"], "fingerprints.markers").items():
        checked_sha(marker_hashes[marker], f"marker_sha256.{marker}", str(actual))
    stage_hashes = object_value(reviewed["stage_source_sha256"], "stage_source_sha256")
    if set(stage_hashes) != set(STAGES):
        raise TemporalAssuranceError("stage_source_sha256 must bind T1, T2, and T3")
    for stage, actual in object_value(fingerprints["stage_source_sha256"], "fingerprints.stages").items():
        checked_sha(stage_hashes[stage], f"stage_source_sha256.{stage}", str(actual))

    binding = object_value(reviewed["source_effect_binding"], "source_effect_binding")
    exact_keys(binding, SOURCE_BINDING_KEYS, "source_effect_binding")
    if binding["effective_version"] != "Constitution_Temporal":
        raise TemporalAssuranceError("effective source version must be Constitution_Temporal")
    case_fragment = text_value(binding["case_bound_rule_fragment"], "source_effect_binding.case_bound_rule_fragment")
    source_fragment = text_value(binding["source_binding_fragment"], "source_effect_binding.source_binding_fragment")
    if constitution.count(case_fragment) != 1:
        raise TemporalAssuranceError("case-bound custody fragment must occur exactly once")
    if constitution.count(source_fragment) != 1:
        raise TemporalAssuranceError("effective-source fragment must occur exactly once")
    checked_sha(binding["case_bound_rule_sha256"], "case_bound_rule_sha256", sha256_text(case_fragment))
    checked_sha(binding["source_binding_sha256"], "source_binding_sha256", sha256_text(source_fragment))
    if not all(
        token in case_fragment
        for token in (
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
        )
    ):
        raise TemporalAssuranceError("custody is not positively bound to a case, renewal, and subject")
    if (
        "~correct(" in case_fragment
        or "permits(CourtPower, Court)" in case_fragment
        or "record(" in case_fragment
    ):
        raise TemporalAssuranceError("custody may not use an absent or global temporal permission")
    if not all(
        token in source_fragment
        for token in (
            "passport(",
            "TemporalLeaseFamily",
            "Constitution_Temporal",
            "$current",
            "~collide($binding, SourceBinding)",
        )
    ):
        raise TemporalAssuranceError(
            "effective-source binding must use the narrowed family passport relation"
        )
    for key in ("semantic_effect", "compatibility_review", "article9_boundary"):
        text_value(binding[key], f"source_effect_binding.{key}")

    inputs = [object_value(item, "temporal_input_contract") for item in list_value(reviewed["temporal_input_contracts"], "temporal_input_contracts")]
    input_ids: set[str] = set()
    for entry in inputs:
        exact_keys(entry, INPUT_FIELDS, f"input {entry.get('id', '?')}")
        identifier = text_value(entry["id"], "input.id")
        if identifier in input_ids:
            raise TemporalAssuranceError(f"duplicate temporal input contract {identifier}")
        input_ids.add(identifier)
        if entry["stage"] not in STAGES:
            raise TemporalAssuranceError(f"input {identifier} has invalid stage")
        for field in INPUT_FIELDS - {"id", "stage"}:
            text_value(entry[field], f"input {identifier}.{field}")
    if input_ids != REQUIRED_INPUTS:
        raise TemporalAssuranceError(f"temporal input contracts differ: {sorted(input_ids ^ REQUIRED_INPUTS)}")

    stages_raw = [object_value(item, "stage") for item in list_value(reviewed["stages"], "stages")]
    if [entry.get("id") for entry in stages_raw] != list(STAGES):
        raise TemporalAssuranceError("stages must be ordered exactly T1, T2, T3")
    for index, entry in enumerate(stages_raw):
        exact_keys(entry, STAGE_KEYS, f"stage {entry.get('id')}")
        expected = list(STAGES[: index + 1])
        if entry["cumulative"] != expected:
            raise TemporalAssuranceError(f"stage {entry['id']} is not cumulative")
        text_value(entry["claim"], f"stage {entry['id']}.claim")
        text_value(entry["does_not_establish"], f"stage {entry['id']}.does_not_establish")

    cases = [object_value(item, "case") for item in list_value(reviewed["cases"], "cases")]
    case_ids: set[str] = set()
    sentinels: dict[tuple[str, str], str] = {}
    stage_sources = build_stages(constitution, pre_t3_rule)
    for case in cases:
        exact_keys(case, CASE_KEYS, f"case {case.get('id', '?')}")
        identifier = text_value(case["id"], "case.id")
        if IDENTIFIER.fullmatch(identifier) is None or identifier in case_ids:
            raise TemporalAssuranceError(f"invalid or duplicate case id {identifier}")
        case_ids.add(identifier)
        stage = text_value(case["stage"], f"case {identifier}.stage")
        if stage not in STAGES:
            raise TemporalAssuranceError(f"case {identifier} has invalid stage")
        if case["process_role"] not in {"predecessor", "successor", "attack", "control"}:
            raise TemporalAssuranceError(f"case {identifier} has invalid process_role")
        text_value(case["title"], f"case {identifier}.title")
        text_value(case["description"], f"case {identifier}.description")
        deletions = validate_ground_facts(case["deletions"], f"case {identifier}.deletions")
        additions = validate_ground_facts(case["additions"], f"case {identifier}.additions")
        if any(fact.startswith("record(") for fact in additions):
            raise TemporalAssuranceError(
                f"case {identifier} reintroduces the refused generic record surface"
            )
        case_text = "\n".join((*deletions, *additions))
        legacy_scopes = sorted(
            scope for scope in FORBIDDEN_CASE_SCOPE_TOKENS if scope in case_text
        )
        if legacy_scopes:
            raise TemporalAssuranceError(
                f"case {identifier} uses legacy aggregate scopes: {legacy_scopes}"
            )
        required_scopes = REQUIRED_CASE_SCOPES.get(identifier, set())
        missing_scopes = sorted(
            scope for scope in required_scopes if f", {scope})." not in case_text
        )
        if missing_scopes:
            raise TemporalAssuranceError(
                f"case {identifier} is missing exact-field scopes: {missing_scopes}"
            )
        if set(deletions) & set(additions):
            raise TemporalAssuranceError(f"case {identifier} deletes and adds the same fact")
        candidate = apply_case(stage_sources[stage], case)
        checks = [object_value(item, f"case {identifier}.check") for item in list_value(case["checks"], f"case {identifier}.checks")]
        if not checks:
            raise TemporalAssuranceError(f"case {identifier} has no executable checks")
        expressions: set[str] = set()
        for check in checks:
            exact_keys(check, CHECK_KEYS, f"case {identifier}.check")
            expression = text_value(check["expression"], f"case {identifier}.expression")
            if expression.endswith(".") or "\n" in expression:
                raise TemporalAssuranceError(f"case {identifier}: expression must omit final period")
            if (
                GROUND_QUERY.fullmatch(expression) is None
                and EVENT_ENTITLEMENT_QUERY.fullmatch(expression) is None
            ):
                raise TemporalAssuranceError(
                    f"case {identifier}: temporal checks must be simple ground relation "
                    "queries or exact ground event-entitlement queries"
                )
            if "Transition" in expression and not (
                re.fullmatch(
                    r"succeed\([^,]+, (?:Transition|TerminalTransition)\)", expression
                )
                or re.fullmatch(r"collide\([^,]+, TransitionLineage\)", expression)
            ):
                raise TemporalAssuranceError(
                    f"case {identifier}: transition checks must use typed succeed/2 or collide/2"
                )
            if expression.startswith("collide("):
                collision = re.fullmatch(r"collide\([^,]+, ([A-Za-z][A-Za-z0-9_]*)\)", expression)
                if collision is None or collision.group(1) not in COLLISION_TAGS:
                    raise TemporalAssuranceError(
                        f"case {identifier}: collide checks must name a reviewed collision tag"
                    )
            if check["expected"] not in VERDICTS:
                raise TemporalAssuranceError(f"case {identifier}: invalid expected verdict")
            text_value(check["purpose"], f"case {identifier}.purpose")
            if expression in expressions:
                raise TemporalAssuranceError(f"case {identifier}: duplicate expression {expression}")
            expressions.add(expression)
            sentinels[(identifier, expression)] = str(check["expected"])
        if sha256_text(candidate) == sha256_text(stage_sources[stage]) and (deletions or additions):
            raise TemporalAssuranceError(f"case {identifier} overlay made no source change")
    for sentinel, expected in REQUIRED_BOUNDARY_VERDICTS.items():
        if sentinels.get(sentinel) != expected:
            raise TemporalAssuranceError(
                f"boundary verdict differs for {sentinel}: "
                f"{sentinels.get(sentinel)!r} != {expected!r}"
            )
    if len(cases) < 12 or {str(case["stage"]) for case in cases} != set(STAGES):
        raise TemporalAssuranceError("cases must cover all stages with a substantial adversarial set")

    pairs = [object_value(item, "fresh_process_pair") for item in list_value(reviewed["fresh_process_pairs"], "fresh_process_pairs")]
    pair_ids: set[str] = set()
    paired_cases: set[str] = set()
    for pair in pairs:
        exact_keys(pair, PAIR_KEYS, f"pair {pair.get('id', '?')}")
        identifier = text_value(pair["id"], "pair.id")
        if identifier in pair_ids:
            raise TemporalAssuranceError(f"duplicate pair id {identifier}")
        pair_ids.add(identifier)
        before = text_value(pair["predecessor_case"], f"pair {identifier}.predecessor")
        after = text_value(pair["successor_case"], f"pair {identifier}.successor")
        if before == after or before not in case_ids or after not in case_ids:
            raise TemporalAssuranceError(f"pair {identifier} must name two distinct cases")
        paired_cases.update((before, after))
        text_value(pair["purpose"], f"pair {identifier}.purpose")
    boundary_pairs = [pair for pair in pairs if pair["id"] == REQUIRED_BOUNDARY_PAIR["id"]]
    if len(boundary_pairs) != 1 or any(
        boundary_pairs[0][field] != value
        for field, value in REQUIRED_BOUNDARY_PAIR.items()
    ):
        raise TemporalAssuranceError("challenge-intake fresh-process pair differs")
    roles = {str(case["id"]): str(case["process_role"]) for case in cases}
    if not any(roles[case_id] == "predecessor" for case_id in paired_cases) or not any(roles[case_id] == "successor" for case_id in paired_cases):
        raise TemporalAssuranceError("fresh-process pairs must include predecessor and successor roles")

    attacks = [object_value(item, "attack") for item in list_value(reviewed["attacks"], "attacks")]
    attack_ids: set[str] = set()
    covered_cases: set[str] = set()
    for attack in attacks:
        exact_keys(attack, ATTACK_KEYS, f"attack {attack.get('id', '?')}")
        identifier = text_value(attack["id"], "attack.id")
        if identifier in attack_ids:
            raise TemporalAssuranceError(f"duplicate attack id {identifier}")
        attack_ids.add(identifier)
        if attack["stage"] not in {*STAGES, "external"} or attack["posture"] not in POSTURES:
            raise TemporalAssuranceError(f"attack {identifier} has invalid stage or posture")
        refs = string_list(attack["case_refs"], f"attack {identifier}.case_refs", allow_empty=True)
        if any(ref not in case_ids for ref in refs):
            raise TemporalAssuranceError(f"attack {identifier} names an unknown case")
        if attack["posture"] != "exposed_external_boundary" and not refs:
            raise TemporalAssuranceError(f"attack {identifier} needs an executable case")
        covered_cases.update(refs)
        text_value(attack["control"], f"attack {identifier}.control")
        text_value(attack["boundary"], f"attack {identifier}.boundary")
    if attack_ids != REQUIRED_ATTACKS:
        raise TemporalAssuranceError(f"attack matrix differs: {sorted(attack_ids ^ REQUIRED_ATTACKS)}")
    attack_by_id = {str(attack["id"]): attack for attack in attacks}
    for identifier, required in REQUIRED_BOUNDARY_ATTACKS.items():
        actual = attack_by_id[identifier]
        if any(actual[field] != value for field, value in required.items()):
            raise TemporalAssuranceError(f"boundary attack policy differs for {identifier}")
    if not case_ids <= covered_cases:
        raise TemporalAssuranceError(f"cases missing from attack matrix: {sorted(case_ids - covered_cases)}")

    impacts = [object_value(item, "narrowness impact") for item in list_value(reviewed["narrowness_impacts"], "narrowness_impacts")]
    impact_refs: set[str] = set()
    for impact in impacts:
        exact_keys(impact, NARROWNESS_KEYS, f"narrowness {impact.get('artifact_ref', '?')}")
        ref = text_value(impact["artifact_ref"], "narrowness.artifact_ref")
        if ref in impact_refs:
            raise TemporalAssuranceError(f"duplicate narrowness artifact {ref}")
        impact_refs.add(ref)
        path = resolve(pathlib.Path(ref))
        repo_relative(path)
        if not path.is_file():
            raise TemporalAssuranceError(f"narrowness artifact does not exist: {ref}")
        if impact["classification"] not in CLASSIFICATIONS:
            raise TemporalAssuranceError(f"invalid narrowness classification for {ref}")
        for field in NARROWNESS_KEYS - {"artifact_ref", "classification"}:
            text_value(impact[field], f"narrowness {ref}.{field}")
    if impact_refs != REQUIRED_NARROWNESS:
        raise TemporalAssuranceError(f"narrowness manifest differs: {sorted(impact_refs ^ REQUIRED_NARROWNESS)}")

    limits = object_value(reviewed["limits"], "limits")
    exact_keys(limits, LIMIT_KEYS, "limits")
    for key in LIMIT_KEYS:
        text_value(limits[key], f"limits.{key}")
    acceptance = object_value(reviewed["acceptance_result"], "acceptance_result")
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance_result")
    if acceptance["result"] != "ESTABLISHED FOR SUPPLIED RECORDS; EXTERNAL LIVENESS NOT ESTABLISHED":
        raise TemporalAssuranceError("acceptance result overstates or changes the reviewed boundary")
    for key in ACCEPTANCE_KEYS - {"result"}:
        text_value(acceptance[key], f"acceptance_result.{key}")
    return stage_sources, cases


def case_pin(case: Mapping[str, object]) -> str:
    checks = [object_value(item, "check") for item in list_value(case["checks"], "checks")]
    lines = [f":expect-pins {len(checks)}", f"# Fresh-process temporal case {case['id']}.", ""]
    for check in checks:
        lines.extend([f"# {check['purpose']}", f"? {check['expression']}.", f"# => {check['expected']}", ""])
    return "\n".join(lines)


def terminate_process_tree(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    try:
        if os.name == "posix":
            os.killpg(process.pid, signal.SIGKILL)
        elif os.name == "nt":
            subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], check=False, timeout=10, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        else:
            process.kill()
        process.wait(timeout=10)
    except (OSError, subprocess.SubprocessError) as exc:
        raise TemporalAssuranceError("timed-out engine process could not be terminated") from exc


def run_process(command: Sequence[str], label: str, timeout: int) -> subprocess.CompletedProcess[str]:
    options: dict[str, object] = {}
    if os.name == "posix":
        options["start_new_session"] = True
    elif os.name == "nt":
        options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    try:
        process = subprocess.Popen(list(command), cwd=ROOT, text=True, encoding="utf-8", errors="replace", stdout=subprocess.PIPE, stderr=subprocess.STDOUT, **options)
    except OSError as exc:
        raise TemporalAssuranceError(f"{label}: cannot start engine: {exc}") from exc
    try:
        output, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        terminate_process_tree(process)
        raise TemporalAssuranceError(f"{label}: timed out after {timeout} seconds") from exc
    return subprocess.CompletedProcess(list(command), process.returncode, output)


def parse_pass(output: str, label: str) -> int:
    clean = re.sub(
        r"(?im)^\s*[^\r\n]+:\s+[0-9]+\s+pins?,\s+0\s+findings?,\s+"
        r"0\s+harness errors?\s*$",
        "",
        output,
    )
    if re.search(
        r"(?im)(?:FINDING|HARNESS ERROR|TRACEBACK|PANIC)",
        clean,
    ):
        raise TemporalAssuranceError(f"{label}: engine output contains a failure marker")
    matches = re.findall(r"(?m)^nibli-pin:\s+PASS\s+[—-]\s+([0-9]+)\s+pins?\s*$", output)
    if len(matches) != 1:
        raise TemporalAssuranceError(f"{label}: expected one PASS summary\n" + "\n".join(output.splitlines()[-12:]))
    return int(matches[0])


def select_pin(cli_pin: pathlib.Path | None) -> pathlib.Path:
    if cli_pin is not None:
        candidate = cli_pin
    elif os.environ.get("NIBLI_PIN", "").strip():
        candidate = pathlib.Path(os.environ["NIBLI_PIN"].strip())
    elif os.environ.get("NIBLI_SRC", "").strip():
        candidate = pathlib.Path(os.environ["NIBLI_SRC"].strip()) / "target/release/nibli-pin"
    else:
        found = shutil.which("nibli-pin")
        candidate = pathlib.Path(found) if found else pathlib.Path.home() / "projects/dhilipsiva/nibli/target/release/nibli-pin"
    candidate = candidate.expanduser().resolve()
    if not candidate.is_absolute() or not candidate.is_file() or not os.access(candidate, os.X_OK):
        raise TemporalAssuranceError(f"release nibli-pin is missing or not executable: {candidate}")
    return candidate


def execute_cases(
    reviewed: Mapping[str, object],
    stage_sources: Mapping[str, str],
    cases: Sequence[Mapping[str, object]],
    pin: pathlib.Path,
) -> tuple[int, int]:
    timeout = int(reviewed["subprocess_timeout_seconds"])
    try:
        # The staged source is intentionally exercised in fresh processes.  Keep
        # the default serial: concurrent compiler processes can multiply the
        # engine's peak memory and turn an assurance run into a host-level OOM.
        jobs = int(os.environ.get("TEMPORAL_ASSURANCE_JOBS", "1"))
    except ValueError as exc:
        raise TemporalAssuranceError("TEMPORAL_ASSURANCE_JOBS must be a positive integer") from exc
    if jobs < 1:
        raise TemporalAssuranceError("TEMPORAL_ASSURANCE_JOBS must be a positive integer")
    with tempfile.TemporaryDirectory(prefix=".temporal-assurance-", dir=ROOT / "new-book-plans") as raw:
        temp = pathlib.Path(raw)
        work: list[tuple[str, pathlib.Path, pathlib.Path, int]] = []
        for case in cases:
            identifier = str(case["id"])
            directory = temp / identifier
            directory.mkdir()
            kb_path = directory / "candidate.nibli"
            pin_path = directory / "case.pins.nibli"
            candidate = apply_case(stage_sources[str(case["stage"])], case)
            pins = case_pin(case)
            kb_path.write_text(candidate, encoding="utf-8", newline="\n")
            pin_path.write_text(pins, encoding="utf-8", newline="\n")
            expected = len(list_value(case["checks"], f"case {identifier}.checks"))
            work.append((identifier, kb_path, pin_path, expected))

        def run(job: tuple[str, pathlib.Path, pathlib.Path, int]) -> int:
            identifier, kb_path, pin_path, expected = job
            completed = run_process([str(pin), "--kb", str(kb_path), str(pin_path)], f"case {identifier}", timeout)
            if completed.returncode != 0:
                raise TemporalAssuranceError(f"case {identifier}: engine exited {completed.returncode}\n" + "\n".join(completed.stdout.splitlines()[-18:]))
            count = parse_pass(completed.stdout, f"case {identifier}")
            if count != expected:
                raise TemporalAssuranceError(f"case {identifier}: expected {expected} pins, engine reported {count}")
            return count

        with concurrent.futures.ThreadPoolExecutor(max_workers=min(jobs, len(work))) as executor:
            pins = list(executor.map(run, work))
    return len(work), sum(pins)


def markdown(value: object) -> str:
    return str(value).replace("|", "\\|")


def code(value: object) -> str:
    return "`" + str(value).replace("`", "\\`") + "`"


def render(reviewed: Mapping[str, object], source_path: pathlib.Path, kb_path: pathlib.Path) -> str:
    acceptance = object_value(reviewed["acceptance_result"], "acceptance_result")
    lines = [
        f"<!-- SPDX-License-Identifier: {reviewed['spdx']} -->",
        "<!-- Generated by new-book-plans/12-temporal-assurance.py; do not edit. -->",
        "",
        f"# {reviewed['title']}",
        "",
        "## Verdict and boundary",
        "",
        "**ESTABLISHED FOR SUPPLIED RECORDS; EXTERNAL LIVENESS NOT ESTABLISHED.**",
        "",
        markdown(acceptance["claim"]),
        "",
        f"- Reviewed source: {code(source_path.as_posix())}.",
        f"- Constitution: {code(kb_path.as_posix())} at {code(reviewed['constitution_sha256'])}.",
        "- Every executable case uses a fresh engine process; paired labels describe a reviewed differential, not shared engine state.",
        "",
        "## Cumulative formal stages",
        "",
        "| stage | cumulative source | bounded claim | boundary |",
        "| --- | --- | --- | --- |",
    ]
    for entry_raw in list_value(reviewed["stages"], "stages"):
        entry = object_value(entry_raw, "stage")
        cumulative = " + ".join(code(item) for item in entry["cumulative"])
        lines.append(f"| {code(entry['id'])} | {cumulative} | {markdown(entry['claim'])} | {markdown(entry['does_not_establish'])} |")
    lines.extend(["", "## Temporal-input contracts", "", "| input | stage | writer and evidence | attack surface | correction, appeal, and handoff |", "| --- | --- | --- | --- | --- |"])
    for raw in list_value(reviewed["temporal_input_contracts"], "temporal_input_contracts"):
        entry = object_value(raw, "input")
        evidence = f"{entry['writer']} {entry['evidence']}"
        attacks = f"Forge: {entry['forge_route']} Withhold: {entry['withholding_route']}"
        path = f"{entry['correction']} {entry['appeal']} {entry['cross_epoch_handoff']} {entry['residual_external_assurance']} {entry['book_boundary']}"
        lines.append(f"| {code(entry['id'])} | {code(entry['stage'])} | {markdown(evidence)} | {markdown(attacks)} | {markdown(path)} |")
    binding = object_value(reviewed["source_effect_binding"], "source_effect_binding")
    lines.extend(["", "## Exact source and effect binding", "", f"- Effective version label: {code(binding['effective_version'])}.", f"- Case-bound rule SHA-256: {code(binding['case_bound_rule_sha256'])}.", f"- Source-binding rule SHA-256: {code(binding['source_binding_sha256'])}.", f"- Semantic effect: {markdown(binding['semantic_effect'])}", f"- Compatibility review: {markdown(binding['compatibility_review'])}", f"- Article 9 boundary: {markdown(binding['article9_boundary'])}"])
    lines.extend(["", "## Fresh-process differential pairs", "", "| pair | predecessor process | successor process | purpose |", "| --- | --- | --- | --- |"])
    for raw in list_value(reviewed["fresh_process_pairs"], "fresh_process_pairs"):
        pair = object_value(raw, "pair")
        lines.append(f"| {code(pair['id'])} | {code(pair['predecessor_case'])} | {code(pair['successor_case'])} | {markdown(pair['purpose'])} |")
    lines.extend(["", "## Executable cases", "", "| case | stage | process role | checks | purpose |", "| --- | --- | --- | ---: | --- |"])
    for raw in list_value(reviewed["cases"], "cases"):
        case = object_value(raw, "case")
        lines.append(f"| {code(case['id'])} | {code(case['stage'])} | {code(case['process_role'])} | {len(case['checks'])} | {markdown(case['description'])} |")
    lines.extend(["", "## Adversarial matrix", "", "| attack | stage | posture | executable cases | control and boundary |", "| --- | --- | --- | --- | --- |"])
    for raw in list_value(reviewed["attacks"], "attacks"):
        attack = object_value(raw, "attack")
        refs = ", ".join(code(item) for item in attack["case_refs"]) or "—"
        lines.append(f"| {code(attack['id'])} | {code(attack['stage'])} | {code(attack['posture'])} | {refs} | {markdown(attack['control'])} {markdown(attack['boundary'])} |")
    lines.extend(["", "## Narrowness-impact manifest", ""])
    for raw in list_value(reviewed["narrowness_impacts"], "narrowness_impacts"):
        impact = object_value(raw, "impact")
        lines.extend([f"### {code(impact['artifact_ref'])}", "", f"- **Standing claim:** {markdown(impact['standing_claim'])}", f"- **Classification:** {code(impact['classification'])}.", f"- **Test route:** {markdown(impact['test_route'])}", f"- **Change trigger:** {markdown(impact['change_trigger'])}", ""])
    lines.extend(["## Residual limits", ""])
    for key in sorted(LIMIT_KEYS):
        lines.append(f"- **{key.replace('_', ' ').title()}:** {markdown(reviewed['limits'][key])}")
    lines.extend(["", "## Reproduce", "", "```bash", "python3 new-book-plans/12-temporal-assurance.py --check", "python3 new-book-plans/12-temporal-assurance.py --check --execute", "```", "", markdown(acceptance["does_not_establish"]), "", f"**Remaining boundary:** {markdown(acceptance['remaining_boundary'])}", ""])
    return "\n".join(lines)


def expect_failure(label: str, action: Callable[[], object]) -> None:
    try:
        action()
    except TemporalAssuranceError:
        return
    raise TemporalAssuranceError(f"negative control did not fail: {label}")


def negative_controls(reviewed: Mapping[str, object], constitution: str, dependencies: Mapping[str, bytes]) -> int:
    pre_t3_rule = text_value(reviewed["pre_t3_custody_rule"], "pre_t3_custody_rule")

    def rebound_source(candidate: str) -> dict[str, object]:
        """Rebind source digests so semantic controls cannot pass on staleness alone."""
        changed = copy.deepcopy(reviewed)
        fingerprints = core_fingerprints(candidate, dependencies, pre_t3_rule)
        changed["constitution_sha256"] = fingerprints["constitution_sha256"]
        changed["bound_sources_sha256"] = fingerprints["bound_sources_sha256"]
        changed["marker_sha256"] = fingerprints["marker_sha256"]
        changed["stage_source_sha256"] = fingerprints["stage_source_sha256"]
        return changed

    controls = 0
    changed = copy.deepcopy(reviewed)
    changed["constitution_sha256"] = "0" * 64
    expect_failure("stale constitution digest", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["bound_sources_sha256"]["time_model"] = "0" * 64
    expect_failure("stale time-model digest", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["temporal_input_contracts"] = changed["temporal_input_contracts"][:-1]
    expect_failure("missing input contract", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["stages"][1]["cumulative"] = ["T2"]
    expect_failure("non-cumulative T2", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["cases"][0]["checks"] = []
    expect_failure("case with no checks", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["cases"][0]["checks"][0]["expression"] = "entitled(Adam, event { eats(Adam) })"
    expect_failure("malformed event-abstraction temporal pin", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    terminal_case = next(case for case in changed["cases"] if case["id"] == "TA-02")
    terminal_check = next(
        check
        for check in terminal_case["checks"]
        if "TerminalTransition" in check["expression"]
    )
    terminal_check["expression"] = "succeed(Epoch_TA_2, TerminalTransition, Epoch_TA_1)"
    expect_failure(
        "malformed terminal-transition pin",
        lambda: validate_source(changed, constitution, dependencies),
    )
    controls += 1
    changed = copy.deepcopy(reviewed)
    exact_case = next(case for case in changed["cases"] if case["id"] == "TA-14")
    exact_case["additions"] = [
        fact.replace("RenewalScope", "WindowScope")
        for fact in exact_case["additions"]
    ]
    expect_failure("legacy aggregate case scope", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["fresh_process_pairs"][0]["successor_case"] = changed["fresh_process_pairs"][0]["predecessor_case"]
    expect_failure("same-process pair alias", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["attacks"] = [entry for entry in changed["attacks"] if entry["id"] != "frozen_transition"]
    expect_failure("missing frozen-transition attack", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    standing_case = next(case for case in changed["cases"] if case["id"] == "TA-10")
    standing_check = next(
        check
        for check in standing_case["checks"]
        if check["expression"] == "person(TA_Unwitnessed_Standing)"
    )
    standing_check["expected"] = "TRUE"
    expect_failure("reversed standing-witness polarity", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    boundary_pair = next(pair for pair in changed["fresh_process_pairs"] if pair["id"] == "TP-12")
    boundary_pair["successor_case"] = "TA-22"
    expect_failure("challenge-intake pair drift", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    boundary_attack = next(
        attack for attack in changed["attacks"]
        if attack["id"] == "challenge_intake_withholding"
    )
    boundary_attack["case_refs"] = ["TA-41"]
    expect_failure("challenge-intake attack loses withheld case", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["narrowness_impacts"] = changed["narrowness_impacts"][:-1]
    expect_failure("pruned narrowness manifest", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    changed = copy.deepcopy(reviewed)
    changed["acceptance_result"]["result"] = "ESTABLISHED"
    expect_failure("liveness overclaim", lambda: validate_source(changed, constitution, dependencies))
    controls += 1
    case_fragment = str(reviewed["source_effect_binding"]["case_bound_rule_fragment"])
    candidate = constitution.replace(case_fragment, "", 1)
    expect_failure(
        "missing case-bound source fragment",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    source_fragment = str(reviewed["source_effect_binding"]["source_binding_fragment"])
    candidate = constitution.replace(source_fragment, "", 1)
    expect_failure(
        "missing effective-source fragment",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    status_fragment = "match($x, CarriedVoid) & match($x, CarriedClear) -> err($x, StatusConflict)"
    candidate = constitution.replace(status_fragment, "", 1)
    expect_failure(
        "missing status-conflict reader",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    visibility_fragment = "all $lease: all $case: all $subject: authorized($lease, ActiveCustody, $case) & related($case, CaseBound) & cite(Court, $case, $subject) & observe(Chronicle, $case, $subject, CaseScope) & observe(TemporalReview, $case, $subject, CaseScope) & observe(Chronicle, $case, Court, HolderScope) & observe(TemporalReview, $case, Court, HolderScope) & ~match($lease, ActivePower) -> err($subject, TemporalAuthority)."
    candidate = constitution.replace(visibility_fragment, "", 1)
    expect_failure(
        "missing inactive-authority reader",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    typed_transition = "-> succeed($after, Transition)."
    recursive_transition = "-> correct($after, Transition, $before, TemporalStandard)."
    candidate = constitution.replace(typed_transition, recursive_transition, 1)
    expect_failure(
        "recursive transition conclusion",
        lambda: validate_source(
            rebound_source(candidate),
            candidate,
            dependencies,
        ),
    )
    controls += 1
    candidate = constitution.replace(
        "# <T1-RULES-END>",
        "all $after: succeed($after, Transition) -> collide($after, UntypedCollision).\n# <T1-RULES-END>",
        1,
    )
    expect_failure(
        "unreviewed collide head",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    candidate = constitution.replace(
        "# <T2-RULES-END>",
        "all $first: all $middle: all $last: precede($first, $middle, EventPath) & precede($middle, $last, EventPath) -> precede($first, $middle, $last, EventPath).\n# <T2-RULES-END>",
        1,
    )
    expect_failure(
        "unreviewed multi-variable path head",
        lambda: validate_source(rebound_source(candidate), candidate, dependencies),
    )
    controls += 1
    expect_failure("missing T2 marker", lambda: marker_block(constitution.replace("# <T2-RULES-BEGIN>\n", "", 1), "T2-RULES"))
    controls += 1
    return controls


def write_output(path: pathlib.Path, value: str) -> None:
    if path.is_symlink():
        raise TemporalAssuranceError("generated report may not be a symlink")
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
    parser.add_argument("--kb", type=pathlib.Path, default=DEFAULT_KB)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--pin", type=pathlib.Path, default=None)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--wait-for-lock", type=float, default=0.0)
    parser.add_argument("--fingerprints", action="store_true")
    args = parser.parse_args(argv)
    source_path = resolve(args.source)
    kb_path = resolve(args.kb)
    output_path = resolve(args.output)
    for path in (source_path, kb_path, output_path, *(resolve(path) for path in BOUND_SOURCES.values())):
        repo_relative(path)
    if output_path.resolve(strict=False) != resolve(DEFAULT_OUTPUT).resolve(strict=False):
        raise TemporalAssuranceError("--output is fixed to new-book-plans/temporal-assurance-case.md")
    reviewed = load_json(read_bytes(source_path, "temporal assurance source"), "temporal assurance source")
    constitution = decode(read_bytes(kb_path, "constitution"), "constitution")
    dependencies = {key: read_bytes(resolve(path), key) for key, path in BOUND_SOURCES.items()}
    if args.fingerprints:
        pre_t3_rule = text_value(reviewed.get("pre_t3_custody_rule"), "pre_t3_custody_rule")
        result = core_fingerprints(constitution, dependencies, pre_t3_rule)
        binding = object_value(reviewed.get("source_effect_binding"), "source_effect_binding")
        case_fragment = text_value(binding.get("case_bound_rule_fragment"), "case_bound_rule_fragment")
        source_fragment = text_value(binding.get("source_binding_fragment"), "source_binding_fragment")
        result["case_bound_rule_sha256"] = sha256_text(case_fragment)
        result["source_binding_sha256"] = sha256_text(source_fragment)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    stage_sources, cases = validate_source(reviewed, constitution, dependencies)
    generated = render(reviewed, repo_relative(source_path), repo_relative(kb_path))
    controls = negative_controls(reviewed, constitution, dependencies)
    processes = pins = 0
    if args.execute:
        processes, pins = execute_cases(reviewed, stage_sources, cases, select_pin(args.pin))
    if args.check:
        current = decode(read_bytes(output_path, "generated temporal report"), "generated temporal report")
        if current != generated:
            raise TemporalAssuranceError(f"{repo_relative(output_path)} is STALE — rerun without --check")
        suffix = f"; {processes} fresh processes / {pins} pins pass" if args.execute else "; execution skipped"
        print(f"{repo_relative(output_path)} is current; {controls} structural negative controls pass{suffix}")
        return 0
    write_output(output_path, generated)
    print(f"{repo_relative(output_path)}: regenerated; {controls} structural negative controls pass")
    return 0


if __name__ == "__main__":
    try:
        from verification_lock_client import run_checker

        raise SystemExit(
            run_checker(main, sys.argv[1:], nonheavy_flags=("--fingerprints",))
        )
    except TemporalAssuranceError as exc:
        print(f"12-temporal-assurance: {exc}", file=sys.stderr)
        raise SystemExit(1)
