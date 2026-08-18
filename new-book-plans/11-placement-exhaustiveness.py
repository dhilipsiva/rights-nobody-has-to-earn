#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate, execute, and render the placement-exhaustiveness audit.

The reviewed JSON source owns the current severity/family/placement-home matrix, exact
producer and destination manifest, bounded source mutations, narrowness
impacts, and limits.  This program generates every matrix case, builds an
ephemeral knowledge base, and asks the release ``nibli-pin`` binary to check
the reviewed outcomes when ``--execute`` is requested.

The audit is repository-level acceptance evidence for the exact current
source.  It adds no runtime exclusivity rule, authenticates no placement
report, supplies no appeal or remedy, and does not deliver housing to a free
person. Every matrix row checks the itemised shelter debt in its exact synthetic
case, then runs a cold one-pin opaque entitlement probe against that same full
candidate. The subject's actual standing route must derive; no standing overlay
or floor-source extraction is supplied.

Usage:
    python3 new-book-plans/11-placement-exhaustiveness.py
    python3 new-book-plans/11-placement-exhaustiveness.py --check
    python3 new-book-plans/11-placement-exhaustiveness.py --check --execute
    python3 new-book-plans/11-placement-exhaustiveness.py --fingerprints
"""

from __future__ import annotations

import argparse
import concurrent.futures
import copy
import hashlib
import itertools
import json
import os
import pathlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from typing import Callable


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("new-book-plans/placement-exhaustiveness-audit.json")
DEFAULT_KB = pathlib.Path("new-book-plans/constitution.nibli")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/placement-exhaustiveness-audit.md")

STATUS = "bounded_current_source_repository_assurance"
EVIDENCE_ROLE = "current_verified_narrowly"
REVIEWED_TIMEOUT_SECONDS = 180
TARGET_RELATIONS = ("fit", "dwell", "building")
AXIS_ORDER = ("severe", "family", "home")
AXIS_VALUES = {
    "severe": ("not_derived", "derived"),
    "family": ("absent", "present"),
    "home": ("absent", "present"),
}
SUBJECT_KINDS = ("confined", "registered_free", "registered_person")
SUBJECT_PREFIXES = {
    "confined": "Confined",
    "registered_free": "Free",
    "registered_person": "Registered",
}
VERDICTS = {"TRUE", "FALSE"}
SHA256 = re.compile(r"[0-9a-f]{64}")
NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
HEAD = re.compile(r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<args>.*)\)")

ROOT_KEYS = {
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "subprocess_timeout_seconds",
    "constitution_sha256",
    "producer_fingerprints",
    "destination_constants",
    "destination_constants_sha256",
    "subject_contract",
    "axis_contract",
    "limits",
    "matrix",
    "required_mutations",
    "mutations",
    "narrowness_impacts",
    "acceptance_result",
}
AXIS_CONTRACT_KEYS = {"order", "states", "semantics"}
SUBJECT_CONTRACT_KEYS = {"states", "semantics"}
LIMIT_KEYS = {
    "bounded_absence_meaning",
    "current_source",
    "runtime",
    "records_and_remedy",
    "housing_delivery",
    "future_delivery",
    "scope",
    "temporal_fixture",
    "trust_root",
}
MATRIX_KEYS = {
    "id",
    "subject_kind",
    "axes",
    "fit_homestay",
    "dwell",
    "destinations",
    "placement_err",
    "interpretation",
}
MUTATION_KEYS = {
    "id",
    "title",
    "kind",
    "mutations",
    "mutation_sha256",
    "expected_source_sha256",
    "observations",
    "baseline_flips",
    "err_absence_case_refs",
    "alarm_setup_facts",
    "interpretation",
}
DELTA_KEYS = {"op", "before", "after", "before_sha256", "after_sha256"}
OBSERVATION_KEYS = {"expression", "expected", "purpose"}
FLIP_KEYS = {"expression", "baseline_expected", "candidate_expected"}
NARROWNESS_KEYS = {
    "artifact_ref",
    "current_claim",
    "classification",
    "reason",
    "future_trigger",
}
ACCEPTANCE_KEYS = {"result", "claim", "does_not_establish", "remaining_boundary"}
NARROWNESS_CLASSIFICATIONS = {"preserved", "revised_and_scoped"}

HISTORICAL_DWELL_LINE = (
    "all $x: prisoner($x) & fit($x, Homestay) & ~at($x, PlacementHome) -> dwell($x).\n"
)
HOMESTAY_LINE = (
    "all $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) "
    "-> building(Homestay, $x).\n"
)
REVERSED_HOMESTAY_LINE = (
    "all $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) "
    "-> building(HighSec, $x).\n"
)
DUPLICATE_APPEND = (
    "\n# Placement-exhaustiveness duplicate-destination mutation "
    "(generated, not enacted).\n"
    "all $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) "
    "-> building(HighSec, $x).\n"
)
PAINTED_DELIVERY_APPEND = (
    "\n# Placement-exhaustiveness painted-delivery mutation "
    "(generated, not enacted).\n"
    "all $x: person($x) -> dwell($x).\n"
)

REQUIRED_MUTATION_SHAPES: dict[str, tuple[str, list[tuple[str, str, str]]]] = {
    "duplicate-destination": (
        "duplicate_destination",
        [("append_exact", "", DUPLICATE_APPEND)],
    ),
    "historical-missing-dwell": (
        "historical_missing_dwell",
        [("delete_exact", HISTORICAL_DWELL_LINE, "")],
    ),
    "missing-required-destination": (
        "missing_required_destination",
        [("delete_exact", HOMESTAY_LINE, "")],
    ),
    "opposite-destination": (
        "opposite_destination",
        [("replace_exact", HOMESTAY_LINE, REVERSED_HOMESTAY_LINE)],
    ),
    "painted-free-person-delivery": (
        "painted_free_person_delivery",
        [("append_exact", "", PAINTED_DELIVERY_APPEND)],
    ),
}
REQUIRED_ALARM_DESTINATIONS = {
    "duplicate-destination": "Homestay",
    "historical-missing-dwell": "Homestay",
    "missing-required-destination": "Homestay",
    "opposite-destination": "HighSec",
    "painted-free-person-delivery": "LowSec",
}


class PlacementAuditError(RuntimeError):
    """Invalid reviewed source, stale report, or failed executable control."""


@dataclass(frozen=True)
class Statement:
    text: str
    line: int


@dataclass
class SourceInventory:
    producers: dict[str, list[str]] = field(default_factory=dict)
    fingerprints: dict[str, str] = field(default_factory=dict)
    destinations: list[str] = field(default_factory=list)
    destinations_sha256: str = ""


@dataclass(frozen=True)
class Query:
    expression: str
    expected: str
    purpose: str


@dataclass(frozen=True)
class T3TemporalPrerequisites:
    """Shared ordered transition required by synthetic custody fixtures."""

    previous_epoch: str
    current_epoch: str
    review_epoch: str
    previous_manifest: str
    current_manifest: str
    event_order_claim: str
    record_order_claim: str
    review_window: str
    source_binding: str


@dataclass(frozen=True)
class T3CustodyPrerequisites:
    """Exact case/lease identity for one synthetic prisoner."""

    subject: str
    victim: str
    case: str
    lease: str


PLACEMENT_TEMPORAL_PREREQUISITES = T3TemporalPrerequisites(
    previous_epoch="Epoch_Previous",
    current_epoch="Epoch_Current",
    review_epoch="Epoch_Review",
    previous_manifest="Manifest_Previous",
    current_manifest="Manifest_Current",
    event_order_claim="Order_Court_A",
    record_order_claim="Order_Log_Court_A",
    review_window="Window_Custody",
    source_binding="Binding_Temporal",
)


FileIdentity = tuple[int, int]


def resolve(path: pathlib.Path) -> pathlib.Path:
    return path if path.is_absolute() else ROOT / path


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve().relative_to(ROOT.resolve())
    except (OSError, ValueError) as exc:
        raise PlacementAuditError(f"path escapes repository: {path}") from exc


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


def sha256_json(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return sha256_bytes(encoded)


def read_bound_file(path: pathlib.Path, label: str) -> tuple[bytes, FileIdentity]:
    """Read one regular-file handle once and bind bytes to device/inode."""
    try:
        with path.open("rb") as stream:
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode):
                raise PlacementAuditError(f"{label} must be a regular file: {path}")
            value = stream.read()
            after = os.fstat(stream.fileno())
    except OSError as exc:
        raise PlacementAuditError(f"cannot read {label} {path}: {exc}") from exc
    before_state = (
        before.st_dev,
        before.st_ino,
        before.st_size,
        before.st_mtime_ns,
        before.st_ctime_ns,
    )
    after_state = (
        after.st_dev,
        after.st_ino,
        after.st_size,
        after.st_mtime_ns,
        after.st_ctime_ns,
    )
    if before_state != after_state or len(value) != after.st_size:
        raise PlacementAuditError(f"{label} changed while its bound bytes were read")
    return value, (after.st_dev, after.st_ino)


def decode_utf8_exact(value: bytes, label: str) -> str:
    try:
        text = value.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise PlacementAuditError(f"{label}: invalid UTF-8: {exc}") from exc
    if text.encode("utf-8") != value:
        raise PlacementAuditError(f"{label}: UTF-8 did not round-trip byte-exactly")
    return text


def decode_constitution(value: bytes) -> str:
    if b"\r" in value:
        raise PlacementAuditError("constitution contains carriage-return bytes")
    return decode_utf8_exact(value, "constitution")


def require_distinct_identities(
    named: Sequence[tuple[str, FileIdentity]],
) -> set[FileIdentity]:
    seen: dict[FileIdentity, str] = {}
    for label, identity in named:
        if identity in seen:
            raise PlacementAuditError(
                f"resolved input identity collision: {label} aliases {seen[identity]}"
            )
        seen[identity] = label
    return set(seen)


def validate_output_target(path: pathlib.Path, inputs: set[FileIdentity]) -> None:
    if path.is_symlink():
        raise PlacementAuditError("generated output may not be a symlink")
    if not path.exists():
        return
    try:
        details = path.stat()
    except OSError as exc:
        raise PlacementAuditError(f"cannot inspect generated output {path}: {exc}") from exc
    if not stat.S_ISREG(details.st_mode):
        raise PlacementAuditError("generated output must be a regular file")
    if details.st_nlink != 1:
        raise PlacementAuditError("generated output must have exactly one hard link")
    if (details.st_dev, details.st_ino) in inputs:
        raise PlacementAuditError("generated output identity collides with an input")


def write_generated_output(
    path: pathlib.Path, value: bytes, inputs: set[FileIdentity]
) -> None:
    flags = os.O_WRONLY | os.O_CREAT
    if hasattr(os, "O_BINARY"):
        flags |= os.O_BINARY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags, 0o666)
    except OSError as exc:
        raise PlacementAuditError(f"cannot open generated output {path}: {exc}") from exc
    try:
        details = os.fstat(descriptor)
        if not stat.S_ISREG(details.st_mode):
            raise PlacementAuditError("generated output must be a regular file")
        if details.st_nlink != 1:
            raise PlacementAuditError("generated output must have exactly one hard link")
        if (details.st_dev, details.st_ino) in inputs:
            raise PlacementAuditError("generated output identity collides with an input")
        with os.fdopen(descriptor, "wb", closefd=True) as stream:
            descriptor = -1
            stream.seek(0)
            stream.truncate(0)
            stream.write(value)
            stream.flush()
    except OSError as exc:
        raise PlacementAuditError(f"cannot write generated output {path}: {exc}") from exc
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append("missing " + ", ".join(missing))
        if extra:
            details.append("unknown " + ", ".join(extra))
        raise PlacementAuditError(f"{path}: " + "; ".join(details))


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise PlacementAuditError(f"{path}: expected object")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise PlacementAuditError(f"{path}: expected list")
    return value


def as_text(value: object, path: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise PlacementAuditError(f"{path}: expected non-empty text")
    return value.strip()


def as_string_list(value: object, path: str, *, nonempty: bool = True) -> list[str]:
    values = as_list(value, path)
    if nonempty and not values:
        raise PlacementAuditError(f"{path}: must not be empty")
    result: list[str] = []
    for index, raw in enumerate(values):
        result.append(as_text(raw, f"{path}[{index}]"))
    if len(result) != len(set(result)):
        raise PlacementAuditError(f"{path}: duplicate values")
    return result


def as_sha(value: object, path: str, expected: str | None = None) -> str:
    digest = as_text(value, path)
    if not SHA256.fullmatch(digest):
        raise PlacementAuditError(f"{path}: expected lowercase SHA-256")
    if expected is not None and digest != expected:
        raise PlacementAuditError(f"{path}: stale; declared {digest}, actual {expected}")
    return digest


def validate_reference(value: object, path: str) -> str:
    reference = as_text(value, path)
    if "::" not in reference:
        raise PlacementAuditError(f"{path}: expected path::stable text")
    raw_file, needle = reference.split("::", 1)
    target = resolve(pathlib.Path(raw_file.strip())).resolve()
    repo_relative(target)
    try:
        content = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise PlacementAuditError(f"{path}: cannot read {target}: {exc}") from exc
    stable_text = needle.strip()
    if not stable_text:
        raise PlacementAuditError(f"{path}: stable text must not be empty")
    occurrences = content.count(stable_text)
    if occurrences != 1:
        raise PlacementAuditError(
            f"{path}: stable text occurs {occurrences} times in {raw_file.strip()}, "
            "expected exactly once"
        )
    return reference


def lex_statements(source: str) -> list[Statement]:
    statements: list[Statement] = []
    buffer: list[str] = []
    in_string = False
    escaped = False
    in_comment = False
    line = 1
    start_line: int | None = None
    for char in source:
        if in_comment:
            if char == "\n":
                in_comment = False
                if buffer:
                    buffer.append(" ")
                line += 1
            continue
        if not in_string and char == "#":
            in_comment = True
            continue
        if char == "\n":
            if buffer:
                buffer.append(" ")
            line += 1
            escaped = False
            continue
        if start_line is None and not char.isspace():
            start_line = line
        if char == '"' and not escaped:
            in_string = not in_string
        if char == "." and not in_string:
            normalized = " ".join("".join(buffer).split())
            if normalized:
                statements.append(Statement(normalized, start_line or line))
            buffer = []
            start_line = None
            escaped = False
            continue
        buffer.append(char)
        escaped = char == "\\" and not escaped
        if char != "\\":
            escaped = False
    residue = "".join(buffer).strip()
    if in_string:
        raise PlacementAuditError("unterminated string in constitution")
    if residue:
        raise PlacementAuditError(
            f"unterminated active statement at line {start_line or line}: {residue[:80]}"
        )
    return statements


def split_arguments(value: str, path: str) -> list[str]:
    result: list[str] = []
    buffer: list[str] = []
    depths = {"(": 0, "{": 0, "[": 0}
    closing = {")": "(", "}": "{", "]": "["}
    in_string = False
    escaped = False
    for char in value:
        if char == '"' and not escaped:
            in_string = not in_string
        if not in_string:
            if char in depths:
                depths[char] += 1
            elif char in closing:
                opener = closing[char]
                depths[opener] -= 1
                if depths[opener] < 0:
                    raise PlacementAuditError(f"{path}: unbalanced arguments")
            elif char == "," and all(depth == 0 for depth in depths.values()):
                result.append("".join(buffer).strip())
                buffer = []
                continue
        buffer.append(char)
        escaped = char == "\\" and not escaped
        if char != "\\":
            escaped = False
    if in_string or any(depth != 0 for depth in depths.values()):
        raise PlacementAuditError(f"{path}: unbalanced arguments")
    result.append("".join(buffer).strip())
    if any(not item for item in result):
        raise PlacementAuditError(f"{path}: empty argument")
    return result


def source_inventory(source: str) -> SourceInventory:
    producers = {relation: [] for relation in TARGET_RELATIONS}
    destinations: set[str] = set()
    for statement in lex_statements(source):
        # A ground fact is a producer too.  Restricting the manifest to rule
        # heads would let a direct fit/dwell/building entry disappear from the
        # reviewed surface after somebody refreshed only the whole-file digest.
        head = (
            statement.text.rsplit("->", 1)[1].strip()
            if "->" in statement.text
            else statement.text
        )
        match = HEAD.fullmatch(head)
        if match is None:
            continue
        relation = match.group("name")
        if relation not in producers:
            continue
        producers[relation].append(statement.text)
        if relation == "building":
            arguments = split_arguments(
                match.group("args"), f"constitution line {statement.line} building head"
            )
            if len(arguments) != 2:
                raise PlacementAuditError(
                    f"constitution line {statement.line}: building head must have arity 2"
                )
            destination = arguments[0]
            if not NAME.fullmatch(destination) or destination.startswith("$"):
                raise PlacementAuditError(
                    f"constitution line {statement.line}: building destination must be "
                    "a reviewed literal constant"
                )
            destinations.add(destination)
    for relation, rules in producers.items():
        if not rules:
            raise PlacementAuditError(f"constitution has no {relation} producer")
        if len(rules) != len(set(rules)):
            raise PlacementAuditError(f"constitution repeats an identical {relation} producer")
    ordered_producers = {
        relation: sorted(rules) for relation, rules in producers.items()
    }
    ordered_destinations = sorted(destinations)
    return SourceInventory(
        producers=ordered_producers,
        fingerprints={
            relation: sha256_json(ordered_producers[relation])
            for relation in TARGET_RELATIONS
        },
        destinations=ordered_destinations,
        destinations_sha256=sha256_json(ordered_destinations),
    )


def axis_tuple(raw_axes: object, path: str) -> tuple[str, str, str]:
    axes = as_object(raw_axes, path)
    exact_keys(axes, set(AXIS_ORDER), path)
    values: list[str] = []
    for axis in AXIS_ORDER:
        value = as_text(axes[axis], f"{path}.{axis}")
        if value not in AXIS_VALUES[axis]:
            raise PlacementAuditError(
                f"{path}.{axis}: expected one of {AXIS_VALUES[axis]}, got {value!r}"
            )
        values.append(value)
    return values[0], values[1], values[2]


def case_subject(kind: str, axes: tuple[str, str, str]) -> str:
    severe, family, home = axes
    parts = [
        SUBJECT_PREFIXES[kind],
        "Severe" if severe == "derived" else "NotSevere",
        "Family" if family == "present" else "NoFamily",
        "Home" if home == "present" else "NoHome",
    ]
    return "_".join(parts)


def case_id(kind: str, axes: tuple[str, str, str]) -> str:
    return case_subject(kind, axes).lower().replace("_", "-")


def required_outcome(
    kind: str, axes: tuple[str, str, str]
) -> tuple[str, str, tuple[str, ...]]:
    severe, family, home = axes
    if kind != "confined":
        return "FALSE", "FALSE", ()
    if severe == "derived":
        return "FALSE", "TRUE", ("HighSec",)
    if home == "present":
        return "TRUE", "TRUE", ("Homestay",)
    return "TRUE", "TRUE", ()


def all_axis_tuples() -> list[tuple[str, str, str]]:
    return list(itertools.product(*(AXIS_VALUES[axis] for axis in AXIS_ORDER)))


def validate_axis_contract(source: Mapping[str, object]) -> None:
    contract = as_object(source["axis_contract"], "axis_contract")
    exact_keys(contract, AXIS_CONTRACT_KEYS, "axis_contract")
    if as_string_list(contract["order"], "axis_contract.order") != list(AXIS_ORDER):
        raise PlacementAuditError(f"axis_contract.order must equal {list(AXIS_ORDER)}")
    states = as_object(contract["states"], "axis_contract.states")
    semantics = as_object(contract["semantics"], "axis_contract.semantics")
    exact_keys(states, set(AXIS_ORDER), "axis_contract.states")
    exact_keys(semantics, set(AXIS_ORDER), "axis_contract.semantics")
    for axis in AXIS_ORDER:
        if as_string_list(states[axis], f"axis_contract.states.{axis}") != list(
            AXIS_VALUES[axis]
        ):
            raise PlacementAuditError(
                f"axis_contract.states.{axis} must equal {list(AXIS_VALUES[axis])}"
            )
        as_text(semantics[axis], f"axis_contract.semantics.{axis}")


def validate_subject_contract(source: Mapping[str, object]) -> None:
    contract = as_object(source["subject_contract"], "subject_contract")
    exact_keys(contract, SUBJECT_CONTRACT_KEYS, "subject_contract")
    if as_string_list(contract["states"], "subject_contract.states") != list(
        SUBJECT_KINDS
    ):
        raise PlacementAuditError(
            f"subject_contract.states must equal {list(SUBJECT_KINDS)}"
        )
    semantics = as_object(contract["semantics"], "subject_contract.semantics")
    exact_keys(semantics, set(SUBJECT_KINDS), "subject_contract.semantics")
    for kind in SUBJECT_KINDS:
        as_text(semantics[kind], f"subject_contract.semantics.{kind}")


def validate_matrix(source: Mapping[str, object], inventory: SourceInventory) -> list[dict[str, object]]:
    raw_matrix = as_list(source["matrix"], "matrix")
    expected_keys = {
        (kind, axes) for kind in SUBJECT_KINDS for axes in all_axis_tuples()
    }
    seen_keys: set[tuple[str, tuple[str, str, str]]] = set()
    seen_ids: set[str] = set()
    result: list[dict[str, object]] = []
    for index, raw_case in enumerate(raw_matrix):
        path = f"matrix[{index}]"
        case = as_object(raw_case, path)
        exact_keys(case, MATRIX_KEYS, path)
        identifier = as_text(case["id"], f"{path}.id")
        if identifier in seen_ids:
            raise PlacementAuditError(f"{path}.id: duplicate {identifier}")
        seen_ids.add(identifier)
        kind = as_text(case["subject_kind"], f"{path}.subject_kind")
        if kind not in SUBJECT_KINDS:
            raise PlacementAuditError(f"{path}.subject_kind: unknown {kind!r}")
        axes = axis_tuple(case["axes"], f"{path}.axes")
        if identifier != case_id(kind, axes):
            raise PlacementAuditError(
                f"{path}.id: expected generated id {case_id(kind, axes)!r}"
            )
        key = (kind, axes)
        if key in seen_keys:
            raise PlacementAuditError(f"{path}: duplicate matrix tuple {key}")
        seen_keys.add(key)
        expected_fit, expected_dwell, expected_destinations = required_outcome(kind, axes)
        fit = as_text(case["fit_homestay"], f"{path}.fit_homestay")
        dwell = as_text(case["dwell"], f"{path}.dwell")
        placement_err = as_text(case["placement_err"], f"{path}.placement_err")
        if fit not in VERDICTS or dwell not in VERDICTS or placement_err not in VERDICTS:
            raise PlacementAuditError(f"{path}: outcomes must be TRUE or FALSE")
        destinations = as_string_list(
            case["destinations"], f"{path}.destinations", nonempty=False
        )
        if any(value not in inventory.destinations for value in destinations):
            raise PlacementAuditError(f"{path}.destinations: unknown destination")
        actual = (fit, dwell, tuple(destinations))
        required = (expected_fit, expected_dwell, expected_destinations)
        if actual != required:
            raise PlacementAuditError(
                f"{path}: current required outcome is {required}, got {actual}"
            )
        if placement_err != "FALSE":
            raise PlacementAuditError(
                f"{path}.placement_err: current base matrix must remain FALSE"
            )
        as_text(case["interpretation"], f"{path}.interpretation")
        result.append(case)
    if seen_keys != expected_keys:
        missing = sorted(expected_keys - seen_keys)
        extra = sorted(seen_keys - expected_keys)
        raise PlacementAuditError(f"matrix is not the exact Cartesian product; missing={missing}, extra={extra}")
    return sorted(result, key=lambda case: str(case["id"]))


def independently_observed(item: str, evidence: str, scope: str) -> list[str]:
    """Return the exact two-witness premises for one reviewed field."""

    return [
        f"observe(Chronicle, {item}, {evidence}, {scope}).",
        f"observe(TemporalReview, {item}, {evidence}, {scope}).",
    ]


def t3_temporal_fixture_facts(
    prerequisites: T3TemporalPrerequisites,
) -> list[str]:
    """Supply raw fields rejoined by the reduced T1/T2/T3 conclusions."""

    facts = [
        f"replace({prerequisites.current_epoch}, {prerequisites.previous_epoch}, Chronicle).",
        f"list({prerequisites.previous_manifest}, {prerequisites.previous_epoch}, ManifestOrder, Chronicle).",
        f"list({prerequisites.current_manifest}, {prerequisites.current_epoch}, ManifestOrder, Chronicle).",
        f"passport(Binding_Chronicle, ChronicleLineage, Constitution_Temporal, {prerequisites.current_epoch}).",
    ]
    facts.extend(
        independently_observed(
            "Binding_Chronicle",
            "ChronicleLineage",
            "LineageFamilyScope",
        )
    )
    facts.extend(
        independently_observed(
            "Binding_Chronicle",
            "Constitution_Temporal",
            "LineageVersionScope",
        )
    )
    facts.extend(
        independently_observed(
            "Binding_Chronicle",
            prerequisites.current_epoch,
            "LineageEpochScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.previous_manifest,
            prerequisites.previous_epoch,
            "ManifestScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.current_manifest,
            prerequisites.current_epoch,
            "ManifestScope",
        )
    )
    facts.append(
        f"list({prerequisites.event_order_claim}, {prerequisites.current_epoch}, "
        f"{prerequisites.review_epoch}, EventSequence)."
    )
    facts.extend(
        independently_observed(
            prerequisites.event_order_claim,
            prerequisites.current_epoch,
            "EventStartScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.event_order_claim,
            prerequisites.review_epoch,
            "EventEndScope",
        )
    )
    facts.append(
        f"list({prerequisites.record_order_claim}, {prerequisites.current_epoch}, "
        f"{prerequisites.review_epoch}, RecordSequence)."
    )
    facts.extend(
        independently_observed(
            prerequisites.record_order_claim,
            prerequisites.current_epoch,
            "RecordStartScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.record_order_claim,
            prerequisites.review_epoch,
            "RecordEndScope",
        )
    )
    facts.append(
        f"date({prerequisites.review_window}, {prerequisites.current_epoch}, "
        f"{prerequisites.review_epoch}, TimeService)."
    )
    facts.extend(
        independently_observed(
            prerequisites.review_window,
            prerequisites.current_epoch,
            "WindowStartScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.review_window,
            prerequisites.review_epoch,
            "WindowEndScope",
        )
    )
    facts.append(
        f"passport({prerequisites.source_binding}, TemporalLeaseFamily, "
        f"Constitution_Temporal, {prerequisites.current_epoch})."
    )
    facts.extend(
        independently_observed(
            prerequisites.source_binding,
            "TemporalLeaseFamily",
            "SourceFamilyScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.source_binding,
            "Constitution_Temporal",
            "SourceVersionScope",
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.source_binding,
            prerequisites.current_epoch,
            "SourceEpochScope",
        )
    )
    return facts


def t3_custody_prerequisites(subject: str) -> T3CustodyPrerequisites:
    """Name the case-bound custody objects for one generated matrix subject."""

    case_and_lease = f"Placement_Case_{subject}"
    return T3CustodyPrerequisites(
        subject=subject,
        victim=f"Victim_{subject}",
        case=case_and_lease,
        lease=case_and_lease,
    )


def t3_prisoner_fixture_facts(
    prerequisites: T3CustodyPrerequisites,
    temporal: T3TemporalPrerequisites,
) -> list[str]:
    """Supply raw fields rejoined before ``correct(lease, ActivePower)``."""

    if prerequisites.case != prerequisites.lease:
        raise PlacementAuditError(
            "the current T3 custody fixture requires one identifier to name "
            "both the case and lease"
        )
    facts = [
        f"injure({prerequisites.subject}, {prerequisites.victim}).",
        f"judge(Court, {prerequisites.subject}).",
        f"cite(Court, {prerequisites.case}, {prerequisites.subject}).",
    ]
    facts.extend(
        independently_observed(prerequisites.case, prerequisites.subject, "CaseScope")
    )
    facts.extend(independently_observed(prerequisites.case, "Court", "HolderScope"))
    facts.extend(
        independently_observed(
            prerequisites.case, "CourtJudgment", "JudgmentScope"
        )
    )
    facts.extend(
        independently_observed(
            prerequisites.case, prerequisites.victim, "InjuryVictimScope"
        )
    )
    facts.append(
        f"authorized({prerequisites.lease}, ActiveCustody, {prerequisites.case})."
    )
    facts.extend(
        independently_observed(prerequisites.lease, "ActiveCustody", "PowerScope")
    )
    facts.extend(
        independently_observed(
            prerequisites.lease, prerequisites.case, "CaseBindingScope"
        )
    )
    facts.append(
        f"limit({prerequisites.lease}, {prerequisites.case}, {temporal.review_window})."
    )
    facts.extend(
        independently_observed(
            prerequisites.lease, temporal.review_window, "LimitScope"
        )
    )
    facts.append(f"continue({prerequisites.lease}, {temporal.current_epoch}).")
    facts.extend(
        independently_observed(
            prerequisites.lease, temporal.current_epoch, "RenewalScope"
        )
    )
    facts.extend(
        [
            f"endorses(Electorate, {prerequisites.lease}).",
            f"endorses(TemporalReview, {prerequisites.lease}).",
        ]
    )
    return facts


def case_queries(case: Mapping[str, object], inventory: SourceInventory) -> list[Query]:
    kind = str(case["subject_kind"])
    axes = axis_tuple(case["axes"], f"matrix.{case['id']}.axes")
    subject = case_subject(kind, axes)
    severe, family, home = axes
    custody = t3_custody_prerequisites(subject)
    queries = [
        Query(f"person({subject})", "TRUE", "Every generated subject has standing."),
        Query(
            f"prisoner({subject})",
            "TRUE" if kind == "confined" else "FALSE",
            "Confinement is the switch separating placement from the free mirror.",
        ),
        Query(
            f"correct({custody.lease}, ActivePower)",
            "TRUE" if kind == "confined" else "FALSE",
            "Only confined rows receive the exact current case-bound custody lease.",
        ),
        Query(
            f"free({subject})",
            "TRUE" if kind == "registered_free" else "FALSE",
            "Affirmative freedom is distinguished from both confinement and personhood alone.",
        ),
        Query(
            f"severe({subject})",
            "TRUE" if severe == "derived" else "FALSE",
            "The generated severity setup reaches the declared axis state.",
        ),
        Query(
            f"family({subject})",
            "TRUE" if family == "present" else "FALSE",
            "The family axis matches the generated fixture record.",
        ),
        Query(
            f"at({subject}, PlacementHome)",
            "TRUE" if home == "present" else "FALSE",
            "The placement-home availability axis matches the generated fixture record.",
        ),
        Query(
            f"owe(State, Dwell, {subject})",
            "TRUE",
            "The itemised shelter debt survives in confined and free rows.",
        ),
        Query(
            f"fit({subject}, Homestay)",
            str(case["fit_homestay"]),
            "Home-confinement eligibility matches the reviewed row.",
        ),
        Query(
            f"dwell({subject})",
            str(case["dwell"]),
            "Housing actuality matches the reviewed row.",
        ),
    ]
    selected = set(str(value) for value in case["destinations"])
    for destination in inventory.destinations:
        queries.append(
            Query(
                f"building({destination}, {subject})",
                "TRUE" if destination in selected else "FALSE",
                "Every discovered destination is queried, including every opposite outcome.",
            )
        )
    queries.append(
        Query(
            f"err({subject}, Placement)",
            str(case["placement_err"]),
            "The current placement alarm remains silent in the accepted matrix.",
        )
    )
    return queries


def matrix_query_map(
    cases: Sequence[Mapping[str, object]], inventory: SourceInventory
) -> dict[str, str]:
    result: dict[str, str] = {}
    for case in cases:
        for query in case_queries(case, inventory):
            prior = result.get(query.expression)
            if prior is not None and prior != query.expected:
                raise PlacementAuditError(
                    f"matrix query conflict for {query.expression}: {prior} vs {query.expected}"
                )
            result[query.expression] = query.expected
    return result


def matrix_facts(cases: Sequence[Mapping[str, object]]) -> str:
    lines = [
        "",
        "# Placement-exhaustiveness matrix facts (generated, not enacted).",
        "# Missing axis entries model bounded non-derivability, not classical negation.",
        "",
    ]
    if any(str(case["subject_kind"]) == "confined" for case in cases):
        lines.extend(
            [
                "# Shared T1/T2/T3 prerequisites for synthetic custody cases.",
                *t3_temporal_fixture_facts(PLACEMENT_TEMPORAL_PREREQUISITES),
                "",
            ]
        )
    for case in cases:
        kind = str(case["subject_kind"])
        axes = axis_tuple(case["axes"], f"matrix.{case['id']}.axes")
        severe, family, home = axes
        subject = case_subject(kind, axes)
        custody = t3_custody_prerequisites(subject)
        lines.append(f"# {case['id']}")
        if kind == "confined":
            lines.extend(
                t3_prisoner_fixture_facts(
                    custody,
                    PLACEMENT_TEMPORAL_PREREQUISITES,
                )
            )
        elif kind == "registered_free":
            lines.append(f"free({subject}).")
        else:
            lines.append(f"person({subject}).")
        if severe == "derived":
            lines.extend(
                [
                    f"attack({subject}, {custody.victim}).",
                    f"cruel({subject}, {custody.victim}).",
                ]
            )
        if family == "present":
            lines.append(f"family({subject}).")
        if home == "present":
            lines.append(f"at({subject}, PlacementHome).")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def matrix_pin_lines(
    cases: Sequence[Mapping[str, object]], inventory: SourceInventory
) -> list[str]:
    query_count = sum(len(case_queries(case, inventory)) for case in cases)
    lines = [
        f":expect-pins {query_count}",
        "# Generated placement-exhaustiveness matrix pins.",
        "# These ephemeral pins are outside chapter pin-count reconciliation.",
        "",
    ]
    for case in cases:
        lines.append(f"# {case['id']}")
        for query in case_queries(case, inventory):
            lines.extend(
                [
                    f"# {query.purpose}",
                    f"? {query.expression}.",
                    f"# => {query.expected}",
                    "",
                ]
            )
    return lines


def composed_floor_pin_lines(case: Mapping[str, object]) -> list[str]:
    subject = case_subject(
        str(case["subject_kind"]),
        axis_tuple(case["axes"], f"matrix.{case['id']}.axes"),
    )
    return [
        ":expect-pins 1",
        "# Full-source floor projection from the generated matrix subject's actual standing route.",
        "# No standing overlay or extracted floor source is supplied.",
        f"? entitled({subject}, event {{ dwell() }}).",
        "# => TRUE",
        "",
    ]


def validate_composed_floor_pin_lines(
    case: Mapping[str, object], lines: Sequence[str]
) -> None:
    subject = case_subject(
        str(case["subject_kind"]),
        axis_tuple(case["axes"], f"matrix.{case['id']}.axes"),
    )
    active = [line for line in lines if line and not line.startswith("#")]
    expected = [
        ":expect-pins 1",
        f"? entitled({subject}, event {{ dwell() }}).",
    ]
    if active != expected or lines.count("# => TRUE") != 1:
        raise PlacementAuditError(
            f"matrix.{case['id']}: composed floor probe must be query-only"
        )


def apply_mutations(
    source: str,
    raw_mutations: object,
    path: str,
    *,
    validate_fragment_hashes: bool = True,
) -> str:
    result = source
    mutations = as_list(raw_mutations, path)
    for index, raw in enumerate(mutations):
        mutation_path = f"{path}[{index}]"
        mutation = as_object(raw, mutation_path)
        exact_keys(mutation, DELTA_KEYS, mutation_path)
        operation = as_text(mutation["op"], f"{mutation_path}.op")
        before = mutation["before"]
        after = mutation["after"]
        if not isinstance(before, str) or not isinstance(after, str):
            raise PlacementAuditError(f"{mutation_path}: before/after must be strings")
        if validate_fragment_hashes:
            as_sha(
                mutation["before_sha256"],
                f"{mutation_path}.before_sha256",
                sha256_text(before),
            )
            as_sha(
                mutation["after_sha256"],
                f"{mutation_path}.after_sha256",
                sha256_text(after),
            )
        if operation == "append_exact":
            if before:
                raise PlacementAuditError(f"{mutation_path}: append before must be empty")
            result += after
        elif operation in {"delete_exact", "replace_exact"}:
            if not before:
                raise PlacementAuditError(f"{mutation_path}: exact mutation needs before")
            count = result.count(before)
            if count != 1:
                raise PlacementAuditError(
                    f"{mutation_path}: before fragment occurs {count} times, expected once"
                )
            replacement = "" if operation == "delete_exact" else after
            if operation == "delete_exact" and after:
                raise PlacementAuditError(f"{mutation_path}: delete after must be empty")
            result = result.replace(before, replacement, 1)
        else:
            raise PlacementAuditError(f"{mutation_path}.op: unknown {operation!r}")
    return result


def mutation_payload(raw_mutations: object, path: str) -> list[dict[str, str]]:
    result: list[dict[str, str]] = []
    for index, raw in enumerate(as_list(raw_mutations, path)):
        mutation = as_object(raw, f"{path}[{index}]")
        result.append(
            {
                "op": str(mutation.get("op", "")),
                "before": str(mutation.get("before", "")),
                "after": str(mutation.get("after", "")),
                "before_sha256": str(mutation.get("before_sha256", "")),
                "after_sha256": str(mutation.get("after_sha256", "")),
            }
        )
    return result


def required_mutation_contract(
    identifier: str,
) -> tuple[list[tuple[str, str, str]], list[str]]:
    if identifier == "duplicate-destination":
        case_ref = "confined-notsevere-nofamily-home"
        return [("building(HighSec, Confined_NotSevere_NoFamily_Home)", "FALSE", "TRUE")], [case_ref]
    if identifier == "historical-missing-dwell":
        case_ref = "confined-notsevere-nofamily-nohome"
        return [("dwell(Confined_NotSevere_NoFamily_NoHome)", "TRUE", "FALSE")], [case_ref]
    home_refs = ["confined-notsevere-nofamily-home", "confined-notsevere-family-home"]
    home_subjects = ["Confined_NotSevere_NoFamily_Home", "Confined_NotSevere_Family_Home"]
    if identifier == "missing-required-destination":
        return [(f"building(Homestay, {subject})", "TRUE", "FALSE") for subject in home_subjects], home_refs
    if identifier == "opposite-destination":
        flips: list[tuple[str, str, str]] = []
        for subject in home_subjects:
            flips.extend([(f"building(Homestay, {subject})", "TRUE", "FALSE"), (f"building(HighSec, {subject})", "FALSE", "TRUE")])
        return flips, home_refs
    if identifier == "painted-free-person-delivery":
        cases = [(kind, axes) for kind in ("registered_free", "registered_person") for axes in all_axis_tuples()]
        return [(f"dwell({case_subject(kind, axes)})", "FALSE", "TRUE") for kind, axes in cases], [case_id(kind, axes) for kind, axes in cases]
    raise PlacementAuditError(f"no code-owned mutation contract for {identifier!r}")

def validate_mutations(
    source: Mapping[str, object],
    kb_text: str,
    base_queries: Mapping[str, str],
    cases_by_id: Mapping[str, Mapping[str, object]],
) -> list[dict[str, object]]:
    required_ids = as_string_list(source["required_mutations"], "required_mutations")
    if required_ids != list(REQUIRED_MUTATION_SHAPES):
        raise PlacementAuditError(
            f"required_mutations must equal {list(REQUIRED_MUTATION_SHAPES)}"
        )
    raw_entries = as_list(source["mutations"], "mutations")
    seen: set[str] = set()
    result: list[dict[str, object]] = []
    for index, raw_entry in enumerate(raw_entries):
        path = f"mutations[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, MUTATION_KEYS, path)
        identifier = as_text(entry["id"], f"{path}.id")
        if identifier in seen:
            raise PlacementAuditError(f"{path}.id: duplicate {identifier}")
        seen.add(identifier)
        if identifier not in REQUIRED_MUTATION_SHAPES:
            raise PlacementAuditError(f"{path}.id: unknown {identifier!r}")
        as_text(entry["title"], f"{path}.title")
        required_kind, required_shapes = REQUIRED_MUTATION_SHAPES[identifier]
        kind = as_text(entry["kind"], f"{path}.kind")
        if kind != required_kind:
            raise PlacementAuditError(
                f"{path}.kind: expected {required_kind!r}, got {kind!r}"
            )
        mutations = as_list(entry["mutations"], f"{path}.mutations")
        actual_shapes: list[tuple[str, str, str]] = []
        for mutation_index, raw_mutation in enumerate(mutations):
            mutation = as_object(raw_mutation, f"{path}.mutations[{mutation_index}]")
            exact_keys(mutation, DELTA_KEYS, f"{path}.mutations[{mutation_index}]")
            actual_shapes.append(
                (str(mutation["op"]), str(mutation["before"]), str(mutation["after"]))
            )
        if actual_shapes != required_shapes:
            raise PlacementAuditError(f"{path}.mutations: required exact shape changed")
        payload = mutation_payload(entry["mutations"], f"{path}.mutations")
        as_sha(
            entry["mutation_sha256"],
            f"{path}.mutation_sha256",
            sha256_json(payload),
        )
        candidate = apply_mutations(kb_text, entry["mutations"], f"{path}.mutations")
        as_sha(
            entry["expected_source_sha256"],
            f"{path}.expected_source_sha256",
            sha256_text(candidate),
        )
        observations: dict[str, str] = {}
        for observation_index, raw_observation in enumerate(
            as_list(entry["observations"], f"{path}.observations")
        ):
            observation_path = f"{path}.observations[{observation_index}]"
            observation = as_object(raw_observation, observation_path)
            exact_keys(observation, OBSERVATION_KEYS, observation_path)
            expression = as_text(observation["expression"], f"{observation_path}.expression")
            expected = as_text(observation["expected"], f"{observation_path}.expected")
            if expected not in VERDICTS:
                raise PlacementAuditError(f"{observation_path}.expected: invalid verdict")
            if expression in observations:
                raise PlacementAuditError(f"{observation_path}: duplicate expression")
            observations[expression] = expected
            as_text(observation["purpose"], f"{observation_path}.purpose")
        if not observations:
            raise PlacementAuditError(f"{path}.observations: must not be empty")
        flips: list[tuple[str, str, str]] = []
        for flip_index, raw_flip in enumerate(
            as_list(entry["baseline_flips"], f"{path}.baseline_flips")
        ):
            flip_path = f"{path}.baseline_flips[{flip_index}]"
            flip = as_object(raw_flip, flip_path)
            exact_keys(flip, FLIP_KEYS, flip_path)
            expression = as_text(flip["expression"], f"{flip_path}.expression")
            baseline = as_text(flip["baseline_expected"], f"{flip_path}.baseline_expected")
            candidate_expected = as_text(
                flip["candidate_expected"], f"{flip_path}.candidate_expected"
            )
            if expression not in base_queries:
                raise PlacementAuditError(f"{flip_path}: expression absent from baseline matrix")
            if base_queries[expression] != baseline:
                raise PlacementAuditError(
                    f"{flip_path}: baseline says {base_queries[expression]}, not {baseline}"
                )
            if baseline == candidate_expected:
                raise PlacementAuditError(f"{flip_path}: candidate does not flip baseline")
            if observations.get(expression) != candidate_expected:
                raise PlacementAuditError(
                    f"{flip_path}: candidate observation is missing or inconsistent"
                )
            flips.append((expression, baseline, candidate_expected))
        if not flips or len(flips) != len({item[0] for item in flips}):
            raise PlacementAuditError(f"{path}.baseline_flips: must be non-empty and unique")
        required_flips, required_err_refs = required_mutation_contract(identifier)
        if flips != required_flips:
            raise PlacementAuditError(
                f"{path}.baseline_flips: exact code-owned affected set changed"
            )
        err_refs = as_string_list(
            entry["err_absence_case_refs"], f"{path}.err_absence_case_refs"
        )
        if err_refs != required_err_refs:
            raise PlacementAuditError(
                f"{path}.err_absence_case_refs: exact code-owned affected set changed"
            )
        setup_facts = as_object(entry["alarm_setup_facts"], f"{path}.alarm_setup_facts")
        if set(setup_facts) != set(err_refs):
            raise PlacementAuditError(
                f"{path}.alarm_setup_facts: keys must equal err_absence_case_refs"
            )
        for case_ref in err_refs:
            if case_ref not in cases_by_id:
                raise PlacementAuditError(f"{path}.err_absence_case_refs: unknown {case_ref}")
            case = cases_by_id[case_ref]
            subject = case_subject(
                str(case["subject_kind"]),
                axis_tuple(case["axes"], f"matrix.{case_ref}.axes"),
            )
            err_expression = f"err({subject}, Placement)"
            if base_queries.get(err_expression) != "FALSE":
                raise PlacementAuditError(f"{path}: {err_expression} is not a FALSE baseline")
            if observations.get(err_expression) not in {None, "FALSE"}:
                raise PlacementAuditError(f"{path}: {err_expression} must remain FALSE")
            setup_expression = as_text(
                setup_facts[case_ref], f"{path}.alarm_setup_facts.{case_ref}"
            )
            setup_match = HEAD.fullmatch(setup_expression)
            if setup_match is None or setup_match.group("name") != "put":
                raise PlacementAuditError(
                    f"{path}.alarm_setup_facts.{case_ref}: expected put(actor, subject, destination)"
                )
            setup_arguments = split_arguments(
                setup_match.group("args"), f"{path}.alarm_setup_facts.{case_ref}"
            )
            if len(setup_arguments) != 3 or setup_arguments[1] != subject:
                raise PlacementAuditError(
                    f"{path}.alarm_setup_facts.{case_ref}: fact must name generated subject {subject}"
                )
            if any(not NAME.fullmatch(argument) for argument in setup_arguments):
                raise PlacementAuditError(
                    f"{path}.alarm_setup_facts.{case_ref}: arguments must be literal constants"
                )
            required_alarm_destination = REQUIRED_ALARM_DESTINATIONS[identifier]
            if setup_arguments[2] != required_alarm_destination:
                raise PlacementAuditError(
                    f"{path}.alarm_setup_facts.{case_ref}: expected reviewed destination "
                    f"{required_alarm_destination}, got {setup_arguments[2]}"
                )
            if setup_expression in observations and observations[setup_expression] != "TRUE":
                raise PlacementAuditError(
                    f"{path}.alarm_setup_facts.{case_ref}: authored observation must be TRUE"
                )
        as_text(entry["interpretation"], f"{path}.interpretation")
        result.append(entry)
    if seen != set(REQUIRED_MUTATION_SHAPES):
        raise PlacementAuditError(
            "mutations do not cover the exact required set: "
            + ", ".join(sorted(set(REQUIRED_MUTATION_SHAPES) - seen))
        )
    return sorted(result, key=lambda entry: required_ids.index(str(entry["id"])))


def validate_source(
    source: Mapping[str, object],
    kb_text: str,
    kb_digest: str,
    inventory: SourceInventory,
) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    exact_keys(source, ROOT_KEYS, "root")
    if source["spdx"] != "CC-BY-4.0":
        raise PlacementAuditError("spdx must be CC-BY-4.0")
    if type(source["schema_version"]) is not int or source["schema_version"] != 2:
        raise PlacementAuditError("schema_version must equal 2")
    as_text(source["title"], "title")
    if source["status"] != STATUS:
        raise PlacementAuditError(f"status must equal {STATUS}")
    if source["evidence_role"] != EVIDENCE_ROLE:
        raise PlacementAuditError(f"evidence_role must equal {EVIDENCE_ROLE}")
    if (
        type(source["subprocess_timeout_seconds"]) is not int
        or source["subprocess_timeout_seconds"] != REVIEWED_TIMEOUT_SECONDS
    ):
        raise PlacementAuditError(
            f"subprocess_timeout_seconds must equal {REVIEWED_TIMEOUT_SECONDS}"
        )
    as_sha(source["constitution_sha256"], "constitution_sha256", kb_digest)
    fingerprints = as_object(source["producer_fingerprints"], "producer_fingerprints")
    exact_keys(fingerprints, set(TARGET_RELATIONS), "producer_fingerprints")
    for relation in TARGET_RELATIONS:
        as_sha(
            fingerprints[relation],
            f"producer_fingerprints.{relation}",
            inventory.fingerprints[relation],
        )
    destinations = as_string_list(source["destination_constants"], "destination_constants")
    if destinations != inventory.destinations:
        raise PlacementAuditError(
            f"destination_constants: declared {destinations}, discovered {inventory.destinations}"
        )
    as_sha(
        source["destination_constants_sha256"],
        "destination_constants_sha256",
        inventory.destinations_sha256,
    )
    validate_axis_contract(source)
    validate_subject_contract(source)
    limits = as_object(source["limits"], "limits")
    exact_keys(limits, LIMIT_KEYS, "limits")
    for key in sorted(LIMIT_KEYS):
        as_text(limits[key], f"limits.{key}")
    cases = validate_matrix(source, inventory)
    cases_by_id = {str(case["id"]): case for case in cases}
    base_queries = matrix_query_map(cases, inventory)
    mutations = validate_mutations(source, kb_text, base_queries, cases_by_id)
    narrowness = as_list(source["narrowness_impacts"], "narrowness_impacts")
    if not narrowness:
        raise PlacementAuditError("narrowness_impacts must not be empty")
    seen_refs: set[str] = set()
    for index, raw in enumerate(narrowness):
        path = f"narrowness_impacts[{index}]"
        entry = as_object(raw, path)
        exact_keys(entry, NARROWNESS_KEYS, path)
        reference = validate_reference(entry["artifact_ref"], f"{path}.artifact_ref")
        if reference in seen_refs:
            raise PlacementAuditError(f"{path}.artifact_ref: duplicate")
        seen_refs.add(reference)
        as_text(entry["current_claim"], f"{path}.current_claim")
        classification = as_text(entry["classification"], f"{path}.classification")
        if classification not in NARROWNESS_CLASSIFICATIONS:
            raise PlacementAuditError(f"{path}.classification: unknown {classification}")
        as_text(entry["reason"], f"{path}.reason")
        as_text(entry["future_trigger"], f"{path}.future_trigger")
    acceptance = as_object(source["acceptance_result"], "acceptance_result")
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance_result")
    if acceptance["result"] != STATUS:
        raise PlacementAuditError(f"acceptance_result.result must equal {STATUS}")
    as_text(acceptance["claim"], "acceptance_result.claim")
    residual = as_text(
        acceptance["does_not_establish"], "acceptance_result.does_not_establish"
    ).lower()
    for term in (
        "runtime exclusivity",
        "authorship",
        "appeal",
        "remedy",
        "free-person housing delivery",
    ):
        if term not in residual:
            raise PlacementAuditError(
                f"acceptance_result.does_not_establish: missing {term!r} boundary"
            )
    as_text(acceptance["remaining_boundary"], "acceptance_result.remaining_boundary")
    return cases, mutations


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise PlacementAuditError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json_bytes(value: bytes, label: str) -> dict[str, object]:
    try:
        parsed = json.loads(
            decode_utf8_exact(value, label), object_pairs_hook=reject_duplicate_keys
        )
    except json.JSONDecodeError as exc:
        raise PlacementAuditError(f"cannot parse {label}: {exc}") from exc
    return as_object(parsed, label)


def expect_failure(label: str, function: Callable[[], object]) -> None:
    try:
        function()
    except PlacementAuditError:
        return
    raise PlacementAuditError(f"structural negative control did not fail: {label}")


def negative_controls(
    source: Mapping[str, object],
    kb_text: str,
    kb_digest: str,
    inventory: SourceInventory,
) -> int:
    controls = 0

    def validate(candidate: Mapping[str, object], inv: SourceInventory = inventory) -> None:
        validate_source(candidate, kb_text, kb_digest, inv)

    changed = copy.deepcopy(source)
    changed["constitution_sha256"] = "0" * 64
    expect_failure("constitution digest drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["schema_version"] = True
    expect_failure("boolean schema version", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["subprocess_timeout_seconds"] = 180.0
    expect_failure("floating subprocess timeout", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["producer_fingerprints"]["building"] = "0" * 64
    expect_failure("building producer drift", lambda: validate(changed))
    controls += 1

    fact_source = (
        kb_text
        + "\n# Ground-producer discovery controls (temporary, not enacted).\n"
        + "fit(GroundPlacementProbe, Homestay).\n"
        + "dwell(GroundPlacementProbe).\n"
        + "building(MedSec, GroundPlacementProbe).\n"
    )
    fact_inventory = source_inventory(fact_source)
    for relation in TARGET_RELATIONS:
        expect_failure(
            f"ground {relation} producer discovered",
            lambda relation=relation: as_sha(
                source["producer_fingerprints"][relation],
                f"ground-control producer_fingerprints.{relation}",
                fact_inventory.fingerprints[relation],
            ),
        )
        controls += 1
    changed = copy.deepcopy(source)
    changed["producer_fingerprints"] = copy.deepcopy(fact_inventory.fingerprints)

    def reject_stale_ground_destination() -> None:
        declared = as_string_list(
            changed["destination_constants"], "ground-control destination_constants"
        )
        if declared != fact_inventory.destinations:
            raise PlacementAuditError("ground building destination changed the manifest")

    expect_failure("ground building destination discovered", reject_stale_ground_destination)
    controls += 1

    changed = copy.deepcopy(source)
    changed["destination_constants"] = changed["destination_constants"][:-1]
    expect_failure("hidden destination", lambda: validate(changed))
    controls += 1

    widened = copy.deepcopy(inventory)
    widened.destinations = sorted(inventory.destinations + ["MedSec"])
    widened.destinations_sha256 = sha256_json(widened.destinations)
    expect_failure("new destination discovered", lambda: validate(source, widened))
    controls += 1

    changed = copy.deepcopy(source)
    changed["matrix"].pop()
    expect_failure("missing Cartesian row", lambda: validate(changed))
    controls += 1

    for subject_kind in ("registered_free", "registered_person"):
        changed = copy.deepcopy(source)
        nonconfined_case = next(
            case
            for case in changed["matrix"]
            if case["subject_kind"] == subject_kind
        )
        nonconfined_case["dwell"] = "TRUE"
        expect_failure(
            f"painted {subject_kind} delivery accepted", lambda: validate(changed)
        )
        controls += 1

    changed = copy.deepcopy(source)
    changed["mutations"].pop()
    expect_failure("missing mutation class", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["mutations"][0]["mutation_sha256"] = "0" * 64
    expect_failure("mutation digest drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["mutations"][0]["baseline_flips"][0]["candidate_expected"] = changed[
        "mutations"
    ][0]["baseline_flips"][0]["baseline_expected"]
    expect_failure("mutation no longer flips baseline", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    painted = next(
        entry
        for entry in changed["mutations"]
        if entry["id"] == "painted-free-person-delivery"
    )
    painted["baseline_flips"].pop()
    expect_failure("painted-delivery row pruned", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    missing = next(
        entry
        for entry in changed["mutations"]
        if entry["id"] == "missing-required-destination"
    )
    pruned_case_ref = missing["err_absence_case_refs"].pop()
    missing["alarm_setup_facts"].pop(pruned_case_ref)
    expect_failure("missing-destination case pruned", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    first_mutation = changed["mutations"][0]
    first_case_ref = first_mutation["err_absence_case_refs"][0]
    first_mutation["alarm_setup_facts"].pop(first_case_ref)
    expect_failure("vacuous placement-alarm setup", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    first_mutation = changed["mutations"][0]
    first_case_ref = first_mutation["err_absence_case_refs"][0]
    first_mutation["alarm_setup_facts"][first_case_ref] = (
        "put(State, Confined_NotSevere_NoFamily_Home, HighSec)"
    )
    expect_failure("downgraded placement-alarm setup", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["acceptance_result"]["does_not_establish"] = "Everything is assured."
    expect_failure("assurance overclaim", lambda: validate(changed))
    controls += 1

    probe_case = as_object(
        as_list(source["matrix"], "matrix")[0], "matrix[0]"
    )
    probe_subject = case_subject(
        str(probe_case["subject_kind"]),
        axis_tuple(probe_case["axes"], "matrix[0].axes"),
    )
    overlay_lines = composed_floor_pin_lines(probe_case)
    overlay_lines.insert(3, f"person({probe_subject}).")
    expect_failure(
        "composed floor standing overlay",
        lambda: validate_composed_floor_pin_lines(probe_case, overlay_lines),
    )
    controls += 1

    expect_failure(
        "duplicate JSON key",
        lambda: json.loads(
            '{"status":"bounded","status":"assured"}',
            object_pairs_hook=reject_duplicate_keys,
        ),
    )
    controls += 1
    return controls


def terminate_process_tree(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    try:
        if os.name == "posix":
            os.killpg(process.pid, signal.SIGKILL)
        elif os.name == "nt":
            cleanup = subprocess.run(
                ["taskkill", "/PID", str(process.pid), "/T", "/F"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=10,
            )
            if cleanup.returncode != 0 and process.poll() is None:
                process.kill()
        else:
            process.kill()
        process.wait(timeout=10)
    except (OSError, subprocess.SubprocessError) as exc:
        if process.poll() is None:
            try:
                process.kill()
                process.wait(timeout=5)
            except (OSError, subprocess.SubprocessError) as fallback_exc:
                raise PlacementAuditError(
                    "timed-out subprocess tree could not be terminated"
                ) from fallback_exc
        if process.poll() is None:
            raise PlacementAuditError(
                "timed-out subprocess tree could not be terminated"
            ) from exc


def run_process(
    command: Sequence[str],
    *,
    label: str,
    timeout_seconds: int,
) -> subprocess.CompletedProcess[str]:
    options: dict[str, object] = {}
    if os.name == "posix":
        options["start_new_session"] = True
    elif os.name == "nt":
        options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    try:
        process = subprocess.Popen(
            list(command),
            cwd=ROOT,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            **options,
        )
    except OSError as exc:
        raise PlacementAuditError(f"{label}: could not start subprocess: {exc}") from exc
    try:
        output, _ = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired as exc:
        terminate_process_tree(process)
        process.communicate()
        raise PlacementAuditError(
            f"{label}: subprocess timed out after {timeout_seconds} seconds"
        ) from exc
    return subprocess.CompletedProcess(list(command), process.returncode, output)


def parse_pass_count(output: str, label: str) -> int:
    marker_surface = re.sub(
        r"(?im)^\s*[^\r\n]+:\s+[0-9]+\s+pins?,\s+0\s+findings?,\s+"
        r"0\s+harness errors?\s*$",
        "",
        output,
    )
    forbidden = re.search(
        r"(?im)(?:FINDING|HARNESS ERROR|NO LONGER REPRODUCE|TRACEBACK|PANIC)",
        marker_surface,
    )
    if forbidden is not None:
        raise PlacementAuditError(
            f"{label}: failure marker appeared despite process success: {forbidden.group(0)}"
        )
    matches = re.findall(
        r"(?m)^nibli-pin:\s+PASS\s+[—-]\s+([0-9]+)\s+pins?\s*$", output
    )
    if len(matches) != 1:
        tail = "\n".join(output.splitlines()[-12:])
        raise PlacementAuditError(
            f"{label}: expected one anchored PASS summary; found {len(matches)}\n{tail}"
        )
    return int(matches[0])


def validate_expected_findings(
    returncode: int, output: str, expected: int, label: str
) -> None:
    if returncode != 1:
        raise PlacementAuditError(f"{label}: exited {returncode}, expected finding exit 1")
    forbidden = re.search(r"(?i)\b(?:HARNESS ERROR|TRACEBACK|PANIC)\b", output)
    if forbidden is not None:
        raise PlacementAuditError(f"{label}: emitted forbidden marker {forbidden.group(0)}")
    lines = output.rstrip("\r\n").splitlines()
    summary = re.compile(r"^nibli-pin:\s+([0-9]+)\s+FINDING\(S\)\s+\(exit 1\)$")
    matches = [summary.fullmatch(line) for line in lines]
    counts = [int(match.group(1)) for match in matches if match is not None]
    if counts != [expected] or not lines or summary.fullmatch(lines[-1]) is None:
        tail = "\n".join(lines[-12:])
        raise PlacementAuditError(
            f"{label}: expected one final {expected}-finding summary\n{tail}"
        )


def select_pin(cli_pin: pathlib.Path | None) -> pathlib.Path:
    if cli_pin is not None:
        candidate = cli_pin
    elif "NIBLI_PIN" in os.environ:
        raw = os.environ["NIBLI_PIN"].strip()
        if not raw:
            raise PlacementAuditError("NIBLI_PIN is set but empty")
        candidate = pathlib.Path(raw)
    elif "NIBLI_SRC" in os.environ:
        raw = os.environ["NIBLI_SRC"].strip()
        if not raw:
            raise PlacementAuditError("NIBLI_SRC is set but empty")
        candidate = pathlib.Path(raw) / "target/release/nibli-pin"
    else:
        on_path = shutil.which("nibli-pin")
        candidate = (
            pathlib.Path(on_path)
            if on_path
            else pathlib.Path.home()
            / "projects/dhilipsiva/nibli/target/release/nibli-pin"
        )
    return candidate.expanduser().resolve()


def mutation_observation_lines(
    entry: Mapping[str, object],
    cases_by_id: Mapping[str, Mapping[str, object]],
) -> list[str]:
    observations = [
        as_object(value, "mutation observation")
        for value in as_list(entry["observations"], "mutation observations")
    ]
    queries: list[tuple[str, str, str]] = [
        (str(item["expression"]), str(item["expected"]), str(item["purpose"]))
        for item in observations
    ]
    known = {expression for expression, _, _ in queries}
    for case_ref in entry["err_absence_case_refs"]:
        case = cases_by_id[str(case_ref)]
        subject = case_subject(
            str(case["subject_kind"]),
            axis_tuple(case["axes"], f"matrix.{case_ref}.axes"),
        )
        expression = f"err({subject}, Placement)"
        setup_expression = str(entry["alarm_setup_facts"][case_ref])
        if setup_expression not in known:
            queries.append(
                (
                    setup_expression,
                    "TRUE",
                    "A positive reviewed placement report makes the alarm-silence probe non-vacuous.",
                )
            )
            known.add(setup_expression)
        if expression not in known:
            queries.append(
                (
                    expression,
                    "FALSE",
                    "The constitutional placement alarm remains silent in this harmful candidate.",
                )
            )
    lines = [
        f":expect-pins {len(queries)}",
        f"# Generated observations for mutation {entry['id']}.",
        "# The source change is constructed by this audit and is never enacted.",
        "",
    ]
    for expression, expected, purpose in queries:
        lines.extend([f"# {purpose}", f"? {expression}.", f"# => {expected}", ""])
    return lines


def mutation_baseline_lines(entry: Mapping[str, object]) -> list[str]:
    flips = [
        as_object(value, "mutation baseline flip")
        for value in as_list(entry["baseline_flips"], "mutation baseline flips")
    ]
    lines = [
        f":expect-pins {len(flips)}",
        f"# Baseline acceptance expectations for mutation {entry['id']}.",
        "# Every query must become a finding against the harmful candidate.",
        "",
    ]
    for flip in flips:
        lines.extend(
            [
                f"? {flip['expression']}.",
                f"# => {flip['baseline_expected']}",
                "",
            ]
        )
    return lines


def execute_audit(
    source: Mapping[str, object],
    kb_text: str,
    inventory: SourceInventory,
    cases: Sequence[Mapping[str, object]],
    mutations: Sequence[Mapping[str, object]],
    pin_binary: pathlib.Path,
) -> tuple[int, int, int, int, int, int, int]:
    if not pin_binary.is_absolute():
        raise PlacementAuditError("selected nibli-pin path must be absolute")
    if not pin_binary.is_file() or not os.access(pin_binary, os.X_OK):
        raise PlacementAuditError(f"release nibli-pin is missing or not executable: {pin_binary}")
    timeout = int(source["subprocess_timeout_seconds"])
    base_pin_count = sum(len(case_queries(case, inventory)) for case in cases)
    composed_floor_pin_count = len(cases)
    cases_by_id = {str(case["id"]): case for case in cases}
    default_jobs = min(8, max(1, os.cpu_count() or 1))
    try:
        requested_jobs = int(
            os.environ.get("PLACEMENT_AUDIT_JOBS", str(default_jobs))
        )
    except ValueError as exc:
        raise PlacementAuditError("PLACEMENT_AUDIT_JOBS must be a positive integer") from exc
    if requested_jobs < 1:
        raise PlacementAuditError("PLACEMENT_AUDIT_JOBS must be a positive integer")
    with tempfile.TemporaryDirectory(
        prefix=".placement-exhaustiveness-", dir=ROOT / "new-book-plans"
    ) as raw_temp:
        temp = pathlib.Path(raw_temp)
        base_jobs: list[tuple[Mapping[str, object], pathlib.Path, pathlib.Path]] = []
        composed_floor_jobs: list[
            tuple[Mapping[str, object], pathlib.Path, pathlib.Path]
        ] = []
        for case in cases:
            case_dir = temp / "base" / str(case["id"])
            case_dir.mkdir(parents=True)
            case_kb = case_dir / "candidate.nibli"
            case_pin = case_dir / "matrix.pins.nibli"
            case_kb.write_text(
                kb_text + matrix_facts([case]), encoding="utf-8", newline="\n"
            )
            case_pin.write_text(
                "\n".join(matrix_pin_lines([case], inventory)),
                encoding="utf-8",
                newline="\n",
            )
            base_jobs.append((case, case_kb, case_pin))
            floor_dir = temp / "floor" / str(case["id"])
            floor_dir.mkdir(parents=True)
            floor_pin = floor_dir / "entitlement.pins.nibli"
            floor_lines = composed_floor_pin_lines(case)
            validate_composed_floor_pin_lines(case, floor_lines)
            floor_pin.write_text(
                "\n".join(floor_lines),
                encoding="utf-8",
                newline="\n",
            )
            composed_floor_jobs.append((case, case_kb, floor_pin))

        def run_base_case(
            job: tuple[Mapping[str, object], pathlib.Path, pathlib.Path]
        ) -> int:
            case, case_kb, case_pin = job
            identifier = str(case["id"])
            completed = run_process(
                [str(pin_binary), "--kb", str(case_kb), str(case_pin)],
                label=f"base matrix {identifier}",
                timeout_seconds=timeout,
            )
            if completed.returncode != 0:
                tail = "\n".join(completed.stdout.splitlines()[-18:])
                raise PlacementAuditError(
                    f"base matrix {identifier} exited {completed.returncode}\n{tail}"
                )
            actual = parse_pass_count(completed.stdout, f"base matrix {identifier}")
            expected = len(case_queries(case, inventory))
            if actual != expected:
                raise PlacementAuditError(
                    f"base matrix {identifier} ran {actual} pins, expected {expected}"
                )
            return actual

        executed_base_pins = 0
        with concurrent.futures.ThreadPoolExecutor(
            max_workers=min(requested_jobs, len(base_jobs))
        ) as pool:
            futures = [pool.submit(run_base_case, job) for job in base_jobs]
            for future in concurrent.futures.as_completed(futures):
                executed_base_pins += future.result()
        if executed_base_pins != base_pin_count:
            raise PlacementAuditError(
                f"base matrix ran {executed_base_pins} pins, expected {base_pin_count}"
            )

        def run_composed_floor_case(
            job: tuple[Mapping[str, object], pathlib.Path, pathlib.Path]
        ) -> int:
            case, case_kb, floor_pin = job
            identifier = str(case["id"])
            completed = run_process(
                [str(pin_binary), "--kb", str(case_kb), str(floor_pin)],
                label=f"composed full-source floor {identifier}",
                timeout_seconds=timeout,
            )
            if completed.returncode != 0:
                tail = "\n".join(completed.stdout.splitlines()[-18:])
                raise PlacementAuditError(
                    f"composed full-source floor {identifier} exited "
                    f"{completed.returncode}\n{tail}"
                )
            actual = parse_pass_count(
                completed.stdout, f"composed full-source floor {identifier}"
            )
            if actual != 1:
                raise PlacementAuditError(
                    f"composed full-source floor {identifier} ran {actual} pins, expected 1"
                )
            return actual

        executed_composed_floor_pins = 0
        with concurrent.futures.ThreadPoolExecutor(
            max_workers=min(requested_jobs, len(composed_floor_jobs))
        ) as pool:
            futures = [
                pool.submit(run_composed_floor_case, job)
                for job in composed_floor_jobs
            ]
            for future in concurrent.futures.as_completed(futures):
                executed_composed_floor_pins += future.result()
        if executed_composed_floor_pins != composed_floor_pin_count:
            raise PlacementAuditError(
                "composed full-source floor probes ran "
                f"{executed_composed_floor_pins} pins, "
                f"expected {composed_floor_pin_count}"
            )

        standing_case = cases_by_id["confined-notsevere-nofamily-home"]
        standing_subject = case_subject(
            str(standing_case["subject_kind"]),
            axis_tuple(
                standing_case["axes"],
                "matrix.confined-notsevere-nofamily-home.axes",
            ),
        )
        standing_fact = f"judge(Court, {standing_subject}).\n"
        standing_facts = matrix_facts([standing_case])
        if standing_facts.count(standing_fact) != 1:
            raise PlacementAuditError(
                "composed-standing sabotage requires one exact Court judgment fact"
            )
        standing_sabotage_dir = temp / "composed-standing-sabotage"
        standing_sabotage_dir.mkdir()
        standing_sabotage_kb = standing_sabotage_dir / "candidate.nibli"
        standing_sabotage_pin = standing_sabotage_dir / "entitlement.pins.nibli"
        standing_sabotage_kb.write_text(
            kb_text + standing_facts.replace(standing_fact, "", 1),
            encoding="utf-8",
            newline="\n",
        )
        standing_sabotage_lines = composed_floor_pin_lines(standing_case)
        validate_composed_floor_pin_lines(
            standing_case, standing_sabotage_lines
        )
        standing_sabotage_pin.write_text(
            "\n".join(standing_sabotage_lines),
            encoding="utf-8",
            newline="\n",
        )
        standing_sabotage = run_process(
            [
                str(pin_binary),
                "--kb",
                str(standing_sabotage_kb),
                str(standing_sabotage_pin),
            ],
            label="composed-standing-removal sabotage",
            timeout_seconds=timeout,
        )
        validate_expected_findings(
            standing_sabotage.returncode,
            standing_sabotage.stdout,
            1,
            "composed-standing-removal sabotage",
        )

        def run_mutation(entry: Mapping[str, object]) -> int:
            identifier = str(entry["id"])
            candidate_dir = temp / identifier
            candidate_dir.mkdir()
            candidate_kb = candidate_dir / "candidate.nibli"
            observation_pin = candidate_dir / "observations.pins.nibli"
            baseline_pin = candidate_dir / "baseline-acceptance.pins.nibli"
            selected_cases = [
                cases_by_id[str(case_ref)]
                for case_ref in entry["err_absence_case_refs"]
            ]
            mutated = apply_mutations(
                kb_text, entry["mutations"], f"mutation {identifier}.mutations"
            )
            candidate_kb.write_text(
                mutated
                + matrix_facts(selected_cases)
                + "\n# Positive placement-report probes (generated, not enacted).\n"
                + "\n".join(
                    f"{entry['alarm_setup_facts'][case_ref]}."
                    for case_ref in entry["err_absence_case_refs"]
                )
                + "\n",
                encoding="utf-8",
                newline="\n",
            )
            observation_lines = mutation_observation_lines(entry, cases_by_id)
            baseline_lines = mutation_baseline_lines(entry)
            observation_pin.write_text(
                "\n".join(observation_lines), encoding="utf-8", newline="\n"
            )
            baseline_pin.write_text(
                "\n".join(baseline_lines), encoding="utf-8", newline="\n"
            )
            observation = run_process(
                [str(pin_binary), "--kb", str(candidate_kb), str(observation_pin)],
                label=f"{identifier} observations",
                timeout_seconds=timeout,
            )
            if observation.returncode != 0:
                tail = "\n".join(observation.stdout.splitlines()[-18:])
                raise PlacementAuditError(
                    f"{identifier} observations exited {observation.returncode}\n{tail}"
                )
            expected_observation_pins = int(observation_lines[0].split()[1])
            actual_observation_pins = parse_pass_count(
                observation.stdout, f"{identifier} observations"
            )
            if actual_observation_pins != expected_observation_pins:
                raise PlacementAuditError(
                    f"{identifier} observations ran {actual_observation_pins}, "
                    f"expected {expected_observation_pins}"
                )
            sabotage = run_process(
                [str(pin_binary), "--kb", str(candidate_kb), str(baseline_pin)],
                label=f"{identifier} baseline sabotage",
                timeout_seconds=timeout,
            )
            validate_expected_findings(
                sabotage.returncode,
                sabotage.stdout,
                len(entry["baseline_flips"]),
                f"{identifier} baseline sabotage",
            )
            return actual_observation_pins

        observation_pins = 0
        with concurrent.futures.ThreadPoolExecutor(
            max_workers=min(requested_jobs, len(mutations))
        ) as pool:
            futures = [pool.submit(run_mutation, entry) for entry in mutations]
            for future in concurrent.futures.as_completed(futures):
                observation_pins += future.result()
    return (
        len(cases),
        base_pin_count,
        len(cases),
        composed_floor_pin_count,
        len(mutations),
        observation_pins,
        len(mutations) + 1,
    )


def markdown(value: object) -> str:
    return " ".join(str(value).split()).replace("|", "\\|")


def code(value: object) -> str:
    text = str(value)
    fence = "``" if "`" in text else "`"
    return f"{fence}{text}{fence}"


def render(
    source: Mapping[str, object],
    source_path: pathlib.Path,
    kb_path: pathlib.Path,
    inventory: SourceInventory,
    cases: Sequence[Mapping[str, object]],
    mutations: Sequence[Mapping[str, object]],
) -> str:
    lines = [
        f"<!-- SPDX-License-Identifier: {source['spdx']} -->",
        "<!-- Generated by new-book-plans/11-placement-exhaustiveness.py; do not edit. -->",
        "",
        f"# {source['title']}",
        "",
        "## Verdict and scope",
        "",
        "**BOUNDED CURRENT-SOURCE REPOSITORY ASSURANCE — not a runtime placement guarantee or housing-delivery assurance.**",
        "",
        markdown(source["acceptance_result"]["claim"]),
        "",
        "The accepted matrix is exhaustive for the declared current axes and the exact",
        "current producer surface. `FALSE` below means *not derivable from the supplied",
        "bounded fixture*, not classical negation or an independently established fact.",
        "",
        "## Bound source manifest",
        "",
        f"- Reviewed source: {code(source_path.as_posix())}.",
        f"- Constitution: {code(kb_path.as_posix())} at SHA-256 {code(source['constitution_sha256'])}.",
        f"- Destination manifest: {', '.join(code(value) for value in inventory.destinations)}.",
        f"- Destination-manifest SHA-256: {code(inventory.destinations_sha256)}.",
        "",
        "| produced relation | reviewed producer fingerprint | active producers |",
        "| --- | --- | ---: |",
    ]
    for relation in TARGET_RELATIONS:
        lines.append(
            f"| {code(relation)} | {code(inventory.fingerprints[relation])} | "
            f"{len(inventory.producers[relation])} |"
        )
    lines.extend(["", "### Active producer statements", ""])
    for relation in TARGET_RELATIONS:
        lines.extend([f"#### {code(relation)}", ""])
        for rule in inventory.producers[relation]:
            lines.append(f"- {code(rule)}")

    semantics = source["axis_contract"]["semantics"]
    lines.extend(["", "## Subject-status contract", ""])
    subject_semantics = source["subject_contract"]["semantics"]
    for kind in SUBJECT_KINDS:
        lines.append(f"- **{kind}:** {markdown(subject_semantics[kind])}")

    lines.extend(["", "## Axis contract", ""])
    for axis in AXIS_ORDER:
        states = " / ".join(code(value) for value in AXIS_VALUES[axis])
        lines.append(
            f"- **{axis}:** {states}. {markdown(semantics[axis])}"
        )

    for kind, heading in (
        ("confined", "Confined matrix"),
        ("registered_free", "Affirmatively free mirror"),
        ("registered_person", "Person-only mirror"),
    ):
        lines.extend(
            [
                "",
                f"## {heading}",
                "",
                "| case | severe | family | home | fit Homestay | dwell | destinations | placement err |",
                "| --- | --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        for case in cases:
            if case["subject_kind"] != kind:
                continue
            axes = case["axes"]
            destinations = ", ".join(code(value) for value in case["destinations"])
            if not destinations:
                destinations = "—"
            lines.append(
                f"| {code(case['id'])} | {code(axes['severe'])} | "
                f"{code(axes['family'])} | {code(axes['home'])} | "
                f"{code(case['fit_homestay'])} | {code(case['dwell'])} | "
                f"{destinations} | {code(case['placement_err'])} |"
            )
        lines.extend(
            [
                "",
                "Every row also checks standing, affirmative freedom or confinement,",
                "the presence or absence of an exact case-bound custody lease,",
                "the itemised shelter debt, each axis result, and every",
                "discovered non-selected destination.",
                "A fresh one-pin probe checks the same subject's exact opaque shelter",
                "entitlement against that row's full generated candidate, including",
                "its actual standing route; it supplies no standing overlay and",
                "extracts no floor rules.",
            ]
        )
    lines.extend(
        [
            "",
            "The two non-confined mirrors are current-source narrowness tripwires. They",
            "record the present gap between entitlement, itemised debt, and actuality.",
            "They are not a",
            "permanent ban on a future valid free-person delivery route.",
            "",
            "## Executable source mutations",
            "",
            "Each candidate is an exact temporary source edit. Its harmful observations",
            "must pass, while every listed baseline matrix expectation must fail. The",
            "candidate also asks the current placement alarm about",
            "every affected subject and requires it to remain silent.",
            "",
            "| mutation | kind | baseline flips | alarm-silence cases | candidate source SHA-256 |",
            "| --- | --- | ---: | ---: | --- |",
        ]
    )
    for entry in mutations:
        lines.append(
            f"| {code(entry['id'])} | {code(entry['kind'])} | "
            f"{len(entry['baseline_flips'])} | {len(entry['err_absence_case_refs'])} | "
            f"{code(entry['expected_source_sha256'])} |"
        )
    for entry in mutations:
        lines.extend(
            [
                "",
                f"### {entry['id']} — {entry['title']}",
                "",
                markdown(entry["interpretation"]),
                "",
                f"- Mutation fingerprint: {code(entry['mutation_sha256'])}.",
                "- Baseline acceptance flips:",
            ]
        )
        for flip in entry["baseline_flips"]:
            lines.append(
                f"  - {code(flip['expression'])}: {code(flip['baseline_expected'])} → "
                f"{code(flip['candidate_expected'])}."
            )
        lines.append("- Candidate observations:")
        for observation in entry["observations"]:
            lines.append(
                f"  - {code(observation['expression'])} = {code(observation['expected'])}: "
                f"{markdown(observation['purpose'])}"
            )
        lines.append(
            "- Placement-alarm silence checked for: "
            + ", ".join(code(value) for value in entry["err_absence_case_refs"])
            + "."
        )
        lines.append("- Positive placement-report probes:")
        for case_ref in entry["err_absence_case_refs"]:
            lines.append(
                f"  - {code(entry['alarm_setup_facts'][case_ref])}."
            )

    lines.extend(["", "## Limits", ""])
    for key in sorted(LIMIT_KEYS):
        lines.append(f"- **{key.replace('_', ' ').title()}:** {markdown(source['limits'][key])}")

    lines.extend(["", "## Narrowness impacts", ""])
    for entry in source["narrowness_impacts"]:
        lines.extend(
            [
                f"### {code(entry['artifact_ref'])}",
                "",
                f"- **Current claim:** {markdown(entry['current_claim'])}",
                f"- **Classification:** {code(entry['classification'])}.",
                f"- **Reason:** {markdown(entry['reason'])}",
                f"- **Future trigger:** {markdown(entry['future_trigger'])}",
                "",
            ]
        )

    lines.extend(
        [
            "## Reproduce",
            "",
            "```bash",
            "python3 new-book-plans/11-placement-exhaustiveness.py --check",
            "python3 new-book-plans/11-placement-exhaustiveness.py --check --execute",
            "```",
            "",
            markdown(source["acceptance_result"]["does_not_establish"]),
            "",
            f"**Remaining boundary:** {markdown(source['acceptance_result']['remaining_boundary'])}",
            "",
        ]
    )
    return "\n".join(lines)


def fingerprint_output(
    source: Mapping[str, object], kb_text: str, kb_digest: str, inventory: SourceInventory
) -> str:
    mutations: dict[str, object] = {}
    for raw_entry in as_list(source.get("mutations", []), "mutations"):
        entry = as_object(raw_entry, "mutation")
        identifier = str(entry.get("id", "missing-id"))
        raw_mutations = entry.get("mutations", [])
        normalized: list[dict[str, str]] = []
        for raw_mutation in as_list(raw_mutations, f"mutation {identifier}"):
            mutation = as_object(raw_mutation, f"mutation {identifier}")
            before = str(mutation.get("before", ""))
            after = str(mutation.get("after", ""))
            normalized.append(
                {
                    "op": str(mutation.get("op", "")),
                    "before": before,
                    "after": after,
                    "before_sha256": sha256_text(before),
                    "after_sha256": sha256_text(after),
                }
            )
        candidate = apply_mutations(
            kb_text,
            raw_mutations,
            f"mutation {identifier}",
            validate_fragment_hashes=False,
        )
        mutations[identifier] = {
            "mutation_sha256": sha256_json(normalized),
            "expected_source_sha256": sha256_text(candidate),
            "fragments": [
                {
                    "before_sha256": sha256_text(str(item.get("before", ""))),
                    "after_sha256": sha256_text(str(item.get("after", ""))),
                }
                for item in as_list(raw_mutations, f"mutation {identifier}")
                if isinstance(item, dict)
            ],
        }
    return json.dumps(
        {
            "constitution_sha256": kb_digest,
            "producer_fingerprints": inventory.fingerprints,
            "destination_constants": inventory.destinations,
            "destination_constants_sha256": inventory.destinations_sha256,
            "mutations": mutations,
        },
        ensure_ascii=False,
        indent=2,
        sort_keys=True,
    )


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    parser.add_argument("--kb", type=pathlib.Path, default=DEFAULT_KB)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--pin", type=pathlib.Path, default=None)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--fingerprints", action="store_true")
    args = parser.parse_args(argv)

    source_path = resolve(args.source)
    kb_path = resolve(args.kb)
    output_path = resolve(args.output)
    repo_relative(source_path)
    repo_relative(kb_path)
    repo_relative(output_path)
    default_output = resolve(DEFAULT_OUTPUT)
    if output_path.resolve(strict=False) != default_output.resolve(strict=False):
        raise PlacementAuditError(
            "--output is fixed to new-book-plans/placement-exhaustiveness-audit.md"
        )
    if output_path.is_symlink() or default_output.is_symlink():
        raise PlacementAuditError("generated output may not be a symlink")
    output_path = default_output

    source_bytes, source_identity = read_bound_file(source_path, "placement audit source")
    kb_bytes, kb_identity = read_bound_file(kb_path, "constitution")
    inputs = require_distinct_identities(
        (("placement audit source", source_identity), ("constitution", kb_identity))
    )
    validate_output_target(output_path, inputs)
    source = load_json_bytes(source_bytes, "placement audit source")
    kb_text = decode_constitution(kb_bytes)
    kb_digest = sha256_bytes(kb_bytes)
    inventory = source_inventory(kb_text)

    if args.fingerprints:
        print(fingerprint_output(source, kb_text, kb_digest, inventory))
        return 0

    cases, mutations = validate_source(source, kb_text, kb_digest, inventory)
    generated = render(
        source,
        repo_relative(source_path),
        repo_relative(kb_path),
        inventory,
        cases,
        mutations,
    )
    controls = negative_controls(source, kb_text, kb_digest, inventory)

    base_runs = base_pins = composed_floor_runs = composed_floor_pins = 0
    candidate_runs = candidate_pins = sabotage_runs = 0
    if args.execute:
        pin = select_pin(args.pin)
        (
            base_runs,
            base_pins,
            composed_floor_runs,
            composed_floor_pins,
            candidate_runs,
            candidate_pins,
            sabotage_runs,
        ) = execute_audit(source, kb_text, inventory, cases, mutations, pin)

    generated_bytes = generated.encode("utf-8")
    validate_output_target(output_path, inputs)
    output_relative = repo_relative(output_path).as_posix()
    if args.check:
        current, _ = read_bound_file(output_path, "generated placement report")
        if current != generated_bytes:
            raise PlacementAuditError(f"{output_relative} is STALE — rerun without --check")
        suffix = (
            f"; {base_runs} matrix / {base_pins} pins, {composed_floor_runs} "
            f"direct composed floor probes / {composed_floor_pins} pins, "
            f"{candidate_runs} mutation "
            f"observation runs / {candidate_pins} pins, {sabotage_runs - 1} "
            "executable mutation-baseline sabotages, and 1 "
            "composed-standing-removal sabotage pass"
            if args.execute
            else "; execution skipped"
        )
        print(
            f"{output_relative} is current; {controls} structural negative controls pass{suffix}"
        )
        return 0

    write_generated_output(output_path, generated_bytes, inputs)
    print(
        f"{output_relative}: regenerated (structural generation; execution not requested); "
        f"{controls} structural negative controls pass"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except PlacementAuditError as exc:
        print(f"11-placement-exhaustiveness: {exc}", file=sys.stderr)
        raise SystemExit(1)
