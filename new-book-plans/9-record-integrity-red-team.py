#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate, execute, and render the bounded flat-snapshot red-team.

The reviewed JSON source owns the threat postures and expected outcomes.  This
program validates its upstream digests, exact snapshot transformations,
scenario coverage, failure polarity, temporal limits, and narrowness impacts. With
``--execute`` it builds ephemeral knowledge bases and asks the release
``nibli-pin`` binary to check every declared query.

T1/T3 are implemented and are tested by the staged temporal assurance harness.
This program deliberately stays below that layer: it reproduces selected
same-snapshot levers and indistinguishability boundaries, and confirms that a
bare ``rotten`` report is inert. It does not duplicate witnessed carry,
omission, status-conflict, or case-bound-power tests owned by script 12.
It checks itemised floor debts and their opaque event-abstraction entitlements
together against the full source. This keeps the standing and release harms
coupled to the exact floor projection they affect.

Usage:
    python3 new-book-plans/9-record-integrity-red-team.py
    python3 new-book-plans/9-record-integrity-red-team.py --check
    python3 new-book-plans/9-record-integrity-red-team.py --check --execute

The default regeneration path executes the scenarios before writing the report.
``--check`` alone is intentionally fast for ``./verify.sh --quick``; the full
verifier adds ``--execute``.
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
import subprocess
import sys
import tempfile
from typing import Callable, Iterable, Mapping, Sequence


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("new-book-plans/record-integrity-red-team.json")
DEFAULT_KB = pathlib.Path("new-book-plans/constitution.nibli")
DEFAULT_LEDGER = pathlib.Path("new-book-plans/assertion-surface-contracts.json")
DEFAULT_ASSURANCE = pathlib.Path(
    "new-book-plans/record-integrity-assurance-case.json"
)
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/record-integrity-red-team.md")

ROOT_KEYS = {
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "constitution_sha256",
    "assertion_surface_contracts_sha256",
    "record_integrity_assurance_case_sha256",
    "posture_meanings",
    "required_routes",
    "required_scenarios",
    "limits",
    "routes",
    "snapshots",
    "scenarios",
    "observational_equivalence",
    "temporal_handoff",
    "narrowness_impacts",
    "acceptance_result",
}
POSTURE_KEYS = {
    "current_harm_reproduced",
    "flat_snapshot_boundary_confirmed",
    "negative_control_preserved",
}
LIMIT_KEYS = {
    "flat_snapshot",
    "attribution",
    "temporal_coverage",
    "liveness",
    "scope",
    "no_new_gate",
}
TEMPORAL_HANDOFF_KEYS = {
    "owner_ref",
    "owned_cases",
    "current_contract",
    "residual_boundary",
}
ROUTE_KEYS = {
    "id",
    "title",
    "premises",
    "tested_delta_polarities",
    "assertion_harm",
    "withholding_deletion_harm",
    "claimant_public_power_polarity",
    "current_detectability",
    "safe_default",
    "authorised_disposition_boundary",
    "opposite_failure_test",
    "residual_limit",
    "owner_ref",
    "temporal_status",
    "scenario_refs",
}
SNAPSHOT_KEYS = {"id", "description", "additions", "deletions"}
SCENARIO_KEYS = {
    "id",
    "title",
    "kind",
    "result",
    "attribution",
    "route_refs",
    "state_refs",
    "queries",
    "comparisons",
    "preserved_invariants",
    "interpretation",
    "residual_limit",
    "authorised_disposition_boundary",
    "opposite_failure",
}
QUERY_KEYS = {"state", "expression", "expected", "purpose"}
SHORT_QUERY_KEYS = {"expression", "expected"}
COMPARISON_KEYS = {
    "expression",
    "from_state",
    "from_expected",
    "to_state",
    "to_expected",
    "claim",
}
INVARIANT_KEYS = {
    "expression",
    "from_state",
    "to_state",
    "expected",
    "claim",
}
OBSERVATIONAL_KEYS = {
    "id",
    "title",
    "route_ref",
    "world_descriptions",
    "snapshot_ref",
    "queries",
    "boundary",
    "prohibited_inference",
}
NARROWNESS_KEYS = {
    "artifact_ref",
    "current_claim",
    "classification",
    "reason",
    "future_trigger",
}
ACCEPTANCE_KEYS = {"result", "claim", "does_not_establish", "remaining_owner"}

REQUIRED_ROUTE_IDS = {f"RT-{number}" for number in range(1, 6)}
# Vocabulary that must appear across route inventories. Execution coverage is
# declared separately and reconciled to exact snapshot deltas.
REQUIRED_SCENARIO_IDS = {
    "RS-01",
    "RS-02",
    "RS-03",
    "RS-04",
    "RS-05",
    "RS-07",
    "RS-08",
    "RS-16",
}
REQUIRED_OBSERVATIONAL_IDS = {f"OE-{number}" for number in range(1, 5)}
REQUIRED_PREMISES = {"free", "at", "person", "rotten", "forgive", "judge", "clear"}
REQUIRED_NARROWNESS_FILES = {
    "book-1/01-what-counts-as-evidence.md",
    "book-1/02-standing.md",
    "book-1/03-who-holds-the-pen.md",
    "book-1/03-who-holds-the-pen.pins.nibli",
    "book-1/05-voiding.md",
    "book-1/06-clawback.md",
    "book-1/07-a-prisoner-is-a-person.md",
    "book-1/09-the-vote-conviction-does-not-take.md",
    "book-1/10-contribution.md",
    "book-1/13-the-one-thing-taken.md",
    "book-1/15-the-five-joints.md",
    "book-1/method.md",
}
SCENARIO_KINDS = {
    "assertion",
    "disappearance",
    "two_entry_matrix",
    "companion_reuse",
    "negative_control",
}
ATTRIBUTIONS = {
    "writer_and_authority_not_attributable_in_flat_snapshot",
    "constructed_source_delta_not_runtime_attribution",
    "writer_independence_not_represented_in_flat_snapshot",
    "purpose_and_case_not_represented_in_flat_snapshot",
    "raw_temporal_input_boundary",
}
NARROWNESS_CLASSIFICATIONS = {"preserved_but_scoped", "revised_and_scoped"}
EXPECTED = {"TRUE", "FALSE"}
ID = re.compile(r"^[A-Z]{2}-[0-9]{1,2}$")
SNAPSHOT_ID = re.compile(r"^[a-z][a-z0-9_]*$")
RELATION = re.compile(r"^[a-z][a-z0-9_]*$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
PLACEHOLDER = re.compile(r"^(?:tbd|todo|unknown|n/?a|pending)$", re.I)

# A small set of load-bearing semantic sentinels is intentionally duplicated in
# code.  Full execution checks every reviewed query; these sentinels make quick
# validation reject a reversed headline verdict instead of merely rendering it.
SEMANTIC_SENTINELS = {
    ("RS-01", "base", "prisoner(Adam)"): "TRUE",
    ("RS-01", "free_adam", "prisoner(Adam)"): "FALSE",
    ("RS-02", "base", "decide(Cira, Ballot)"): "FALSE",
    ("RS-02", "adulthood_cira", "decide(Cira, Ballot)"): "TRUE",
    ("RS-03", "no_adulthood_hano", "decide(Hano, Ballot)"): "FALSE",
    ("RS-04", "no_person_bela", "owe(State, Eats, Bela)"): "FALSE",
    ("RS-04", "no_person_bela", "entitled(Bela, event { eats() })"): "FALSE",
    ("RS-04", "no_person_bela", "false(Bela)"): "TRUE",
    ("RS-05", "carry_forge_marked", "false(Carry_Forge)"): "FALSE",
    ("RS-05", "carry_forge_marked", "match(Carry_Forge, CarriedVoid)"): "FALSE",
    ("RS-07", "vex_forgive_only", "clean(Vex)"): "FALSE",
    ("RS-07", "vex_judgment_only", "clean(Vex)"): "FALSE",
    ("RS-07", "vex_both", "clean(Vex)"): "TRUE",
    ("RS-07", "vex_both", "permits(Review, Vex)"): "FALSE",
    ("RS-08", "nia_precleared", "clean(Nia)"): "TRUE",
    ("RS-08", "nia_forgive_without_judgment", "clean(Nia)"): "FALSE",
    ("RS-16", "base", "permits(Appeals, Nia)"): "TRUE",
    ("RS-16", "nia_relief_clear_only", "permits(Appeals, Nia)"): "FALSE",
    ("RS-16", "nia_relief_judgment_only", "permits(Appeals, Nia)"): "FALSE",
}


class RedTeamError(RuntimeError):
    """Invalid reviewed source, stale generated report, or failed execution."""


def resolve(path: pathlib.Path) -> pathlib.Path:
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
        raise RedTeamError(f"{path}: {'; '.join(details)}")


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise RedTeamError(f"{path}: expected an object")
    if not all(isinstance(key, str) for key in value):
        raise RedTeamError(f"{path}: object keys must be strings")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise RedTeamError(f"{path}: expected an array")
    return value


def as_text(value: object, path: str) -> str:
    if not isinstance(value, str):
        raise RedTeamError(f"{path}: expected a string")
    text = value.strip()
    if not text or PLACEHOLDER.fullmatch(text):
        raise RedTeamError(f"{path}: requires reviewed, non-placeholder text")
    return value


def text_list(
    value: object,
    path: str,
    *,
    nonempty: bool = True,
    unique: bool = True,
) -> list[str]:
    values = as_list(value, path)
    if nonempty and not values:
        raise RedTeamError(f"{path}: must not be empty")
    result = [as_text(item, f"{path}[{index}]") for index, item in enumerate(values)]
    if unique and len(result) != len(set(result)):
        raise RedTeamError(f"{path}: duplicate values are not allowed")
    return result


def sha256(path: pathlib.Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as exc:
        raise RedTeamError(f"cannot read {path}: {exc}") from exc


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve().relative_to(ROOT.resolve())
    except (OSError, ValueError) as exc:
        raise RedTeamError(f"path escapes repository: {path}") from exc


def validate_reference(value: object, path: str) -> str:
    """Validate a stable ``path::unique literal needle`` reference."""
    reference = as_text(value, path)
    if reference.count("::") != 1:
        raise RedTeamError(
            f"{path}: reference must be repo-local path::unique literal needle"
        )
    raw_file, needle = reference.split("::", 1)
    if not raw_file or not needle or "\\" in raw_file:
        raise RedTeamError(f"{path}: invalid reference path or empty needle")
    candidate = pathlib.Path(raw_file)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise RedTeamError(f"{path}: reference must stay inside the repository")
    target = ROOT / candidate
    repo_relative(target)
    if not target.is_file():
        raise RedTeamError(f"{path}: referenced file does not exist: {raw_file}")
    try:
        body = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise RedTeamError(f"{path}: cannot read {raw_file}: {exc}") from exc
    count = body.count(needle)
    if count != 1:
        raise RedTeamError(
            f"{path}: needle must occur exactly once in {raw_file}; found {count}"
        )
    return reference


def validate_identifier(value: object, path: str, expected_prefix: str) -> str:
    identifier = as_text(value, path)
    if not ID.fullmatch(identifier) or not identifier.startswith(expected_prefix):
        raise RedTeamError(f"{path}: invalid stable identifier {identifier!r}")
    return identifier


def validate_expected(value: object, path: str) -> str:
    expected = as_text(value, path)
    if expected not in EXPECTED:
        raise RedTeamError(f"{path}: expected TRUE or FALSE, got {expected!r}")
    return expected


def validate_ground_atom(value: object, path: str) -> tuple[str, str]:
    """Require exactly one ground atom, with balanced event-term delimiters."""
    atom = as_text(value, path)
    if "\n" in atom or "\r" in atom:
        raise RedTeamError(f"{path}: ground atom must stay on one line")
    head = re.match(r"^([a-z][a-z0-9_]*)\(", atom)
    if head is None:
        raise RedTeamError(f"{path}: expected one relation atom")
    if re.fullmatch(r"[A-Za-z0-9_(),{} \t]+", atom) is None:
        raise RedTeamError(f"{path}: operator, directive, or statement injection rejected")
    stack: list[str] = []
    opening = {"(": ")", "{": "}"}
    closing = {")": "(", "}": "{"}
    for index, character in enumerate(atom):
        if character in opening:
            stack.append(character)
        elif character in closing:
            if not stack or stack.pop() != closing[character]:
                raise RedTeamError(f"{path}: unbalanced delimiters")
            if not stack and index != len(atom) - 1:
                raise RedTeamError(f"{path}: multiple atoms or trailing content rejected")
    if stack or not atom.endswith(")"):
        raise RedTeamError(f"{path}: unbalanced or incomplete ground atom")
    return head.group(1), atom


def validate_expression(value: object, path: str) -> str:
    expression = as_text(value, path)
    _, atom = validate_ground_atom(expression, path)
    return atom


def apply_snapshot(
    base_text: str,
    snapshot: Mapping[str, object],
    path: str,
) -> str:
    """Apply exact-once ground-fact deltas to the live source."""
    lines = base_text.splitlines()
    additions = text_list(snapshot["additions"], f"{path}.additions", nonempty=False)
    deletions = text_list(snapshot["deletions"], f"{path}.deletions", nonempty=False)
    if set(additions) & set(deletions):
        raise RedTeamError(f"{path}: the same statement cannot be added and deleted")
    if snapshot["id"] == "base":
        if additions or deletions:
            raise RedTeamError(f"{path}: base snapshot must have no transformation")
    elif not additions and not deletions:
        raise RedTeamError(f"{path}: non-base snapshot transformation is a no-op")

    for index, statement in enumerate(additions):
        if not statement.endswith("."):
            raise RedTeamError(f"{path}.additions[{index}]: ground fact must end in '.'")
        validate_ground_atom(statement[:-1], f"{path}.additions[{index}]")
        count = lines.count(statement)
        if count != 0:
            raise RedTeamError(
                f"{path}.additions[{index}]: addition is not exact and new; found {count} existing line(s)"
            )

    remove: set[str] = set()
    for index, statement in enumerate(deletions):
        if not statement.endswith("."):
            raise RedTeamError(f"{path}.deletions[{index}]: ground fact must end in '.'")
        validate_ground_atom(statement[:-1], f"{path}.deletions[{index}]")
        count = lines.count(statement)
        if count != 1:
            raise RedTeamError(
                f"{path}.deletions[{index}]: deletion must match exactly once; found {count}"
            )
        remove.add(statement)

    result = [line for line in lines if line not in remove]
    if additions:
        result.extend(["", "# Red-team snapshot additions (generated, not enacted)."])
        result.extend(additions)
    transformed = "\n".join(result) + "\n"
    if snapshot["id"] != "base" and transformed == base_text:
        raise RedTeamError(f"{path}: transformation produced byte-identical source")
    return transformed


def scenario_query_map(
    scenario: Mapping[str, object],
    path: str,
    snapshot_ids: set[str],
) -> dict[tuple[str, str], str]:
    result: dict[tuple[str, str], str] = {}
    state_refs = set(text_list(scenario["state_refs"], f"{path}.state_refs"))
    unknown_states = sorted(state_refs - snapshot_ids)
    if unknown_states:
        raise RedTeamError(f"{path}.state_refs: unknown snapshot(s): {', '.join(unknown_states)}")
    if len(state_refs) < 2:
        raise RedTeamError(f"{path}.state_refs: every scenario needs at least two states")
    for index, raw_query in enumerate(as_list(scenario["queries"], f"{path}.queries")):
        query_path = f"{path}.queries[{index}]"
        query = as_object(raw_query, query_path)
        exact_keys(query, QUERY_KEYS, query_path)
        state = as_text(query["state"], f"{query_path}.state")
        if state not in state_refs:
            raise RedTeamError(f"{query_path}.state: {state!r} is not in state_refs")
        expression = validate_expression(query["expression"], f"{query_path}.expression")
        expected = validate_expected(query["expected"], f"{query_path}.expected")
        as_text(query["purpose"], f"{query_path}.purpose")
        key = (state, expression)
        if key in result and result[key] != expected:
            raise RedTeamError(f"{query_path}: conflicting expected result for {state}/{expression}")
        if key in result:
            raise RedTeamError(f"{query_path}: duplicate query in one scenario")
        result[key] = expected
    if not result:
        raise RedTeamError(f"{path}.queries: must not be empty")
    for state in state_refs:
        if not any(query_state == state for query_state, _ in result):
            raise RedTeamError(f"{path}.queries: state {state!r} has no executable query")
    return result


def validate_comparisons(
    scenario: Mapping[str, object],
    query_map: Mapping[tuple[str, str], str],
    path: str,
) -> None:
    comparisons = as_list(scenario["comparisons"], f"{path}.comparisons")
    kind = str(scenario["kind"])
    if kind == "negative_control":
        if comparisons:
            raise RedTeamError(f"{path}.comparisons: negative control must not claim a flip")
    elif not comparisons:
        raise RedTeamError(f"{path}.comparisons: non-vacuous scenario needs a flip")
    for index, raw in enumerate(comparisons):
        item_path = f"{path}.comparisons[{index}]"
        item = as_object(raw, item_path)
        exact_keys(item, COMPARISON_KEYS, item_path)
        expression = validate_expression(item["expression"], f"{item_path}.expression")
        from_state = as_text(item["from_state"], f"{item_path}.from_state")
        to_state = as_text(item["to_state"], f"{item_path}.to_state")
        from_expected = validate_expected(item["from_expected"], f"{item_path}.from_expected")
        to_expected = validate_expected(item["to_expected"], f"{item_path}.to_expected")
        as_text(item["claim"], f"{item_path}.claim")
        if from_state == to_state or from_expected == to_expected:
            raise RedTeamError(f"{item_path}: comparison must discriminate two states")
        if query_map.get((from_state, expression)) != from_expected:
            raise RedTeamError(f"{item_path}: from result does not match a declared query")
        if query_map.get((to_state, expression)) != to_expected:
            raise RedTeamError(f"{item_path}: to result does not match a declared query")

    invariants = as_list(
        scenario["preserved_invariants"], f"{path}.preserved_invariants"
    )
    if not invariants:
        raise RedTeamError(f"{path}.preserved_invariants: positive control required")
    for index, raw in enumerate(invariants):
        item_path = f"{path}.preserved_invariants[{index}]"
        item = as_object(raw, item_path)
        exact_keys(item, INVARIANT_KEYS, item_path)
        expression = validate_expression(item["expression"], f"{item_path}.expression")
        from_state = as_text(item["from_state"], f"{item_path}.from_state")
        to_state = as_text(item["to_state"], f"{item_path}.to_state")
        expected = validate_expected(item["expected"], f"{item_path}.expected")
        as_text(item["claim"], f"{item_path}.claim")
        if from_state == to_state:
            raise RedTeamError(f"{item_path}: invariant must span two states")
        if query_map.get((from_state, expression)) != expected:
            raise RedTeamError(f"{item_path}: from invariant lacks matching query")
        if query_map.get((to_state, expression)) != expected:
            raise RedTeamError(f"{item_path}: to invariant lacks matching query")


def validate_source(
    source: dict[str, object],
    kb_text: str,
    kb_digest: str,
    ledger_digest: str,
    assurance_digest: str,
) -> dict[str, dict[tuple[str, str], str]]:
    """Validate authored structure and return each scenario's query vector."""
    exact_keys(source, ROOT_KEYS, "root")
    if source["spdx"] != "CC-BY-4.0":
        raise RedTeamError("spdx: reviewed source must be CC-BY-4.0")
    if type(source["schema_version"]) is not int or source["schema_version"] != 2:
        raise RedTeamError("schema_version: only version 2 is supported")
    as_text(source["title"], "title")
    if source["status"] != "bounded_flat_snapshot_red_team_not_assurance":
        raise RedTeamError(
            "status: this artifact must remain bounded flat-snapshot red-team evidence"
        )
    if source["evidence_role"] != "exposes_gap_and_tests_boundary":
        raise RedTeamError(
            "evidence_role: mixed gap/boundary evidence may not be promoted to assurance"
        )
    digest_checks = (
        ("constitution_sha256", kb_digest),
        ("assertion_surface_contracts_sha256", ledger_digest),
        ("record_integrity_assurance_case_sha256", assurance_digest),
    )
    for key, actual in digest_checks:
        declared = as_text(source[key], key)
        if not SHA256.fullmatch(declared):
            raise RedTeamError(f"{key}: expected a lowercase SHA-256 digest")
        if declared != actual:
            raise RedTeamError(f"{key}: stale; declared {declared}, actual {actual}")

    postures = as_object(source["posture_meanings"], "posture_meanings")
    exact_keys(postures, POSTURE_KEYS, "posture_meanings")
    for key, value in postures.items():
        as_text(value, f"posture_meanings.{key}")
    limits = as_object(source["limits"], "limits")
    exact_keys(limits, LIMIT_KEYS, "limits")
    for key, value in limits.items():
        as_text(value, f"limits.{key}")
    handoff = as_object(source["temporal_handoff"], "temporal_handoff")
    exact_keys(handoff, TEMPORAL_HANDOFF_KEYS, "temporal_handoff")
    validate_reference(handoff["owner_ref"], "temporal_handoff.owner_ref")
    owned_cases = text_list(handoff["owned_cases"], "temporal_handoff.owned_cases")
    if set(owned_cases) != {"TA-02", "TA-03", "TA-04", "TA-08", "TA-25"}:
        raise RedTeamError(
            "temporal_handoff.owned_cases: must name the exact delegated carry/status cases"
        )
    for field in ("current_contract", "residual_boundary"):
        as_text(handoff[field], f"temporal_handoff.{field}")

    declared_routes = set(text_list(source["required_routes"], "required_routes"))
    declared_scenarios = set(
        text_list(source["required_scenarios"], "required_scenarios")
    )
    if declared_routes != REQUIRED_ROUTE_IDS:
        raise RedTeamError(
            "required_routes: must name exactly " + ", ".join(sorted(REQUIRED_ROUTE_IDS))
        )
    if declared_scenarios != REQUIRED_SCENARIO_IDS:
        raise RedTeamError(
            "required_scenarios: must name exactly "
            + ", ".join(sorted(REQUIRED_SCENARIO_IDS))
        )

    routes: dict[str, dict[str, object]] = {}
    premise_coverage: set[str] = set()
    for index, raw_route in enumerate(as_list(source["routes"], "routes")):
        path = f"routes[{index}]"
        route = as_object(raw_route, path)
        exact_keys(route, ROUTE_KEYS, path)
        route_id = validate_identifier(route["id"], f"{path}.id", "RT-")
        if route_id in routes:
            raise RedTeamError(f"{path}.id: duplicate {route_id}")
        as_text(route["title"], f"{path}.title")
        premises = text_list(route["premises"], f"{path}.premises")
        for premise in premises:
            if not RELATION.fullmatch(premise):
                raise RedTeamError(f"{path}.premises: invalid relation {premise!r}")
        premise_coverage.update(premises)
        tested_deltas = as_object(
            route["tested_delta_polarities"], f"{path}.tested_delta_polarities"
        )
        if set(tested_deltas) != set(premises):
            raise RedTeamError(
                f"{path}.tested_delta_polarities: must name every and only route premise"
            )
        for premise, raw_polarities in tested_deltas.items():
            polarities = set(
                text_list(
                    raw_polarities,
                    f"{path}.tested_delta_polarities.{premise}",
                )
            )
            if not polarities <= {"addition", "deletion"}:
                raise RedTeamError(
                    f"{path}.tested_delta_polarities.{premise}: unknown delta polarity"
                )
        for field in ROUTE_KEYS - {
            "id",
            "title",
            "premises",
            "tested_delta_polarities",
            "scenario_refs",
            "owner_ref",
        }:
            as_text(route[field], f"{path}.{field}")
        validate_reference(route["owner_ref"], f"{path}.owner_ref")
        scenario_refs = set(text_list(route["scenario_refs"], f"{path}.scenario_refs"))
        unknown = sorted(scenario_refs - REQUIRED_SCENARIO_IDS)
        if unknown:
            raise RedTeamError(f"{path}.scenario_refs: unknown scenario(s): {', '.join(unknown)}")
        routes[route_id] = route
    if set(routes) != REQUIRED_ROUTE_IDS:
        raise RedTeamError("routes: missing or unexpected required route IDs")
    if premise_coverage != REQUIRED_PREMISES:
        missing = sorted(REQUIRED_PREMISES - premise_coverage)
        extra = sorted(premise_coverage - REQUIRED_PREMISES)
        raise RedTeamError(
            "routes: premise coverage mismatch; missing "
            + (", ".join(missing) or "none")
            + "; extra "
            + (", ".join(extra) or "none")
        )

    snapshots: dict[str, dict[str, object]] = {}
    for index, raw_snapshot in enumerate(as_list(source["snapshots"], "snapshots")):
        path = f"snapshots[{index}]"
        snapshot = as_object(raw_snapshot, path)
        exact_keys(snapshot, SNAPSHOT_KEYS, path)
        snapshot_id = as_text(snapshot["id"], f"{path}.id")
        if not SNAPSHOT_ID.fullmatch(snapshot_id):
            raise RedTeamError(f"{path}.id: invalid snapshot identifier")
        if snapshot_id in snapshots:
            raise RedTeamError(f"{path}.id: duplicate {snapshot_id}")
        as_text(snapshot["description"], f"{path}.description")
        apply_snapshot(kb_text, snapshot, path)
        snapshots[snapshot_id] = snapshot
    if "base" not in snapshots:
        raise RedTeamError("snapshots: base snapshot is required")

    scenarios: dict[str, dict[str, object]] = {}
    query_vectors: dict[str, dict[tuple[str, str], str]] = {}
    scenario_to_routes: dict[str, set[str]] = {}
    for index, raw_scenario in enumerate(as_list(source["scenarios"], "scenarios")):
        path = f"scenarios[{index}]"
        scenario = as_object(raw_scenario, path)
        exact_keys(scenario, SCENARIO_KEYS, path)
        scenario_id = validate_identifier(scenario["id"], f"{path}.id", "RS-")
        if scenario_id in scenarios:
            raise RedTeamError(f"{path}.id: duplicate {scenario_id}")
        as_text(scenario["title"], f"{path}.title")
        kind = as_text(scenario["kind"], f"{path}.kind")
        if kind not in SCENARIO_KINDS:
            raise RedTeamError(f"{path}.kind: unknown kind {kind!r}")
        result = as_text(scenario["result"], f"{path}.result")
        if result not in POSTURE_KEYS:
            raise RedTeamError(f"{path}.result: unknown posture {result!r}")
        if kind == "negative_control" and result != "negative_control_preserved":
            raise RedTeamError(f"{path}.result: negative control must preserve its control")
        if kind != "negative_control" and result == "negative_control_preserved":
            raise RedTeamError(f"{path}.result: only negative control may use this posture")
        attribution = as_text(scenario["attribution"], f"{path}.attribution")
        if attribution not in ATTRIBUTIONS:
            raise RedTeamError(f"{path}.attribution: attribution overclaim or unknown value")
        if kind == "disappearance" and attribution != (
            "constructed_source_delta_not_runtime_attribution"
        ):
            raise RedTeamError(
                f"{path}.attribution: disappearance may not be attributed as live deletion or withholding"
            )
        route_refs = set(text_list(scenario["route_refs"], f"{path}.route_refs"))
        unknown_routes = sorted(route_refs - set(routes))
        if unknown_routes:
            raise RedTeamError(f"{path}.route_refs: unknown route(s): {', '.join(unknown_routes)}")
        query_map = scenario_query_map(scenario, path, set(snapshots))
        validate_comparisons(scenario, query_map, path)
        for field in (
            "interpretation",
            "residual_limit",
            "authorised_disposition_boundary",
            "opposite_failure",
        ):
            as_text(scenario[field], f"{path}.{field}")
        scenarios[scenario_id] = scenario
        query_vectors[scenario_id] = query_map
        scenario_to_routes[scenario_id] = route_refs
    if set(scenarios) != REQUIRED_SCENARIO_IDS:
        raise RedTeamError("scenarios: missing or unexpected required scenario IDs")

    for route_id, route in routes.items():
        declared = set(str(value) for value in route["scenario_refs"])
        actual = {
            scenario_id
            for scenario_id, route_refs in scenario_to_routes.items()
            if route_id in route_refs
        }
        if declared != actual:
            raise RedTeamError(
                f"routes[{route_id}].scenario_refs: does not reconcile with scenario route_refs"
            )
        premises = set(str(value) for value in route["premises"])
        actual_polarities = {premise: set() for premise in premises}
        route_states = {
            str(state)
            for scenario_id in actual
            for state in scenarios[scenario_id]["state_refs"]
        }
        for state in route_states:
            snapshot = snapshots[state]
            for field, polarity in (("additions", "addition"), ("deletions", "deletion")):
                for fact in snapshot[field]:
                    fact_text = str(fact)
                    relation, _ = validate_ground_atom(
                        fact_text[:-1], f"snapshot {state}.{field}"
                    )
                    if relation in actual_polarities:
                        actual_polarities[relation].add(polarity)
        declared_polarities = {
            str(premise): set(str(value) for value in values)
            for premise, values in route["tested_delta_polarities"].items()
        }
        if declared_polarities != actual_polarities:
            raise RedTeamError(
                f"routes[{route_id}].tested_delta_polarities: does not reconcile with referenced snapshot deltas"
            )

    for (scenario_id, state, expression), expected in SEMANTIC_SENTINELS.items():
        actual = query_vectors.get(scenario_id, {}).get((state, expression))
        if actual != expected:
            raise RedTeamError(
                f"{scenario_id}: semantic sentinel {state}/{expression} must be {expected}, got {actual}"
            )

    correction_states = set(str(value) for value in scenarios["RS-07"]["state_refs"])
    if correction_states != {"base", "vex_forgive_only", "vex_judgment_only", "vex_both"}:
        raise RedTeamError(
            "RS-07: correction matrix must contain neither, first-only, second-only, and both"
        )
    reuse_states = set(str(value) for value in scenarios["RS-08"]["state_refs"])
    if reuse_states != {"base", "nia_precleared", "nia_forgive_without_judgment"}:
        raise RedTeamError(
            "RS-08: pre-clear and generic-companion removal controls are mandatory"
        )
    relief_states = set(str(value) for value in scenarios["RS-16"]["state_refs"])
    if relief_states != {
        "base",
        "nia_relief_neither",
        "nia_relief_clear_only",
        "nia_relief_judgment_only",
    }:
        raise RedTeamError(
            "RS-16: relief matrix must contain neither, clear-only, judgment-only, and both"
        )

    observational: dict[str, dict[str, object]] = {}
    for index, raw_entry in enumerate(
        as_list(source["observational_equivalence"], "observational_equivalence")
    ):
        path = f"observational_equivalence[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, OBSERVATIONAL_KEYS, path)
        entry_id = validate_identifier(entry["id"], f"{path}.id", "OE-")
        if entry_id in observational:
            raise RedTeamError(f"{path}.id: duplicate {entry_id}")
        as_text(entry["title"], f"{path}.title")
        route_ref = as_text(entry["route_ref"], f"{path}.route_ref")
        if route_ref not in routes:
            raise RedTeamError(f"{path}.route_ref: unknown route {route_ref}")
        worlds = text_list(entry["world_descriptions"], f"{path}.world_descriptions")
        if len(worlds) < 2:
            raise RedTeamError(f"{path}.world_descriptions: at least two worlds required")
        snapshot_ref = as_text(entry["snapshot_ref"], f"{path}.snapshot_ref")
        if snapshot_ref not in snapshots:
            raise RedTeamError(f"{path}.snapshot_ref: unknown snapshot {snapshot_ref}")
        queries = as_list(entry["queries"], f"{path}.queries")
        if not queries:
            raise RedTeamError(f"{path}.queries: identical executable vector required")
        for query_index, raw_query in enumerate(queries):
            query_path = f"{path}.queries[{query_index}]"
            query = as_object(raw_query, query_path)
            exact_keys(query, SHORT_QUERY_KEYS, query_path)
            validate_expression(query["expression"], f"{query_path}.expression")
            validate_expected(query["expected"], f"{query_path}.expected")
        as_text(entry["boundary"], f"{path}.boundary")
        as_text(entry["prohibited_inference"], f"{path}.prohibited_inference")
        observational[entry_id] = entry
    if set(observational) != REQUIRED_OBSERVATIONAL_IDS:
        raise RedTeamError("observational_equivalence: required indistinguishability cases missing")

    narrowness_files: set[str] = set()
    seen_references: set[str] = set()
    for index, raw_entry in enumerate(as_list(source["narrowness_impacts"], "narrowness_impacts")):
        path = f"narrowness_impacts[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, NARROWNESS_KEYS, path)
        reference = validate_reference(entry["artifact_ref"], f"{path}.artifact_ref")
        if reference in seen_references:
            raise RedTeamError(f"{path}.artifact_ref: duplicate reference")
        seen_references.add(reference)
        narrowness_files.add(reference.split("::", 1)[0])
        if entry["classification"] not in NARROWNESS_CLASSIFICATIONS:
            raise RedTeamError(
                f"{path}.classification: unknown narrowness disposition"
            )
        for field in ("current_claim", "reason", "future_trigger"):
            as_text(entry[field], f"{path}.{field}")
    if narrowness_files != REQUIRED_NARROWNESS_FILES:
        missing = sorted(REQUIRED_NARROWNESS_FILES - narrowness_files)
        extra = sorted(narrowness_files - REQUIRED_NARROWNESS_FILES)
        raise RedTeamError(
            "narrowness_impacts: file coverage mismatch; missing "
            + (", ".join(missing) or "none")
            + "; extra "
            + (", ".join(extra) or "none")
        )

    acceptance = as_object(source["acceptance_result"], "acceptance_result")
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance_result")
    if acceptance["result"] != "current_harm_reproduced":
        raise RedTeamError("acceptance_result.result: may not claim assurance")
    as_text(acceptance["claim"], "acceptance_result.claim")
    residuals = text_list(
        acceptance["does_not_establish"], "acceptance_result.does_not_establish"
    )
    residual_text = " ".join(residuals).lower()
    for required_term in (
        "authorship",
        "deletion",
        "liveness",
        "recovery",
        "general",
        "deployment",
    ):
        if required_term not in residual_text:
            raise RedTeamError(
                f"acceptance_result.does_not_establish: must retain {required_term!r} boundary"
            )
    validate_reference(acceptance["remaining_owner"], "acceptance_result.remaining_owner")
    return query_vectors


def collect_queries(source: Mapping[str, object]) -> dict[str, dict[str, str]]:
    """Reconcile all executable queries by snapshot."""
    result: dict[str, dict[str, str]] = {}

    def add(state: str, expression: str, expected: str, path: str) -> None:
        state_queries = result.setdefault(state, {})
        prior = state_queries.get(expression)
        if prior is not None and prior != expected:
            raise RedTeamError(
                f"{path}: global query conflict for {state}/{expression}: {prior} vs {expected}"
            )
        state_queries[expression] = expected

    for scenario_index, raw_scenario in enumerate(source["scenarios"]):
        scenario = as_object(raw_scenario, f"scenarios[{scenario_index}]")
        for query_index, raw_query in enumerate(scenario["queries"]):
            query = as_object(raw_query, "query")
            add(
                str(query["state"]),
                str(query["expression"]),
                str(query["expected"]),
                f"scenarios[{scenario_index}].queries[{query_index}]",
            )
    for entry_index, raw_entry in enumerate(source["observational_equivalence"]):
        entry = as_object(raw_entry, f"observational_equivalence[{entry_index}]")
        state = str(entry["snapshot_ref"])
        for query_index, raw_query in enumerate(entry["queries"]):
            query = as_object(raw_query, "query")
            add(
                state,
                str(query["expression"]),
                str(query["expected"]),
                f"observational_equivalence[{entry_index}].queries[{query_index}]",
            )
    return result


def execute_scenarios(
    source: Mapping[str, object],
    kb_text: str,
    pin_binary: pathlib.Path,
) -> tuple[int, int, int]:
    """Execute every snapshot and require one inverted pin to fail loudly."""
    if not pin_binary.is_file():
        raise RedTeamError(f"no nibli-pin at {pin_binary}")
    if not os.access(pin_binary, os.X_OK):
        raise RedTeamError(f"nibli-pin is not executable: {pin_binary}")
    snapshots = {
        str(snapshot["id"]): snapshot
        for snapshot in source["snapshots"]
        if isinstance(snapshot, dict)
    }
    queries = collect_queries(source)
    executed_pins = 0
    with tempfile.TemporaryDirectory(prefix="record-integrity-red-team-") as raw_tmp:
        temp = pathlib.Path(raw_tmp)
        jobs: list[tuple[str, pathlib.Path, pathlib.Path, int]] = []
        for state in sorted(queries):
            snapshot = snapshots[state]
            kb_path = temp / f"{state}.nibli"
            pin_path = temp / f"{state}.pins.nibli"
            transformed = apply_snapshot(kb_text, snapshot, f"snapshot {state}")
            kb_path.write_text(transformed, encoding="utf-8", newline="\n")
            state_queries = queries[state]
            pin_lines = [
                f":expect-pins {len(state_queries)}",
                f"# Generated red-team queries for snapshot {state}.",
                "# This file is ephemeral and outside chapter pin reconciliation.",
                "",
            ]
            for expression, expected in sorted(state_queries.items()):
                pin_lines.extend(
                    [
                        f"# Reviewed expected consequence in {state}.",
                        f"? {expression}.",
                        f"# => {expected}",
                        "",
                    ]
                )
            pin_path.write_text("\n".join(pin_lines), encoding="utf-8", newline="\n")
            jobs.append((state, kb_path, pin_path, len(state_queries)))

        def run_snapshot(
            job: tuple[str, pathlib.Path, pathlib.Path, int]
        ) -> tuple[str, int]:
            state, kb_path, pin_path, expected_pins = job
            completed = subprocess.run(
                [str(pin_binary), "--kb", str(kb_path), str(pin_path)],
                cwd=ROOT,
                text=True,
                encoding="utf-8",
                errors="replace",
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )
            output = completed.stdout
            if completed.returncode != 0:
                tail = "\n".join(output.splitlines()[-12:])
                raise RedTeamError(
                    f"snapshot {state}: nibli-pin exited {completed.returncode}\n{tail}"
                )
            match = re.search(r"PASS\s+[—-]\s+([0-9]+)\s+pins?", output)
            if not match:
                tail = "\n".join(output.splitlines()[-12:])
                raise RedTeamError(f"snapshot {state}: no PASS count\n{tail}")
            actual = int(match.group(1))
            if actual != expected_pins:
                raise RedTeamError(
                    f"snapshot {state}: ran {actual} pins, expected {expected_pins}"
                )
            return state, actual

        try:
            requested_jobs = int(os.environ.get("RED_TEAM_JOBS", "4"))
        except ValueError as exc:
            raise RedTeamError("RED_TEAM_JOBS must be a positive integer") from exc
        if requested_jobs < 1:
            raise RedTeamError("RED_TEAM_JOBS must be a positive integer")
        workers = min(requested_jobs, len(jobs))
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            futures = [pool.submit(run_snapshot, job) for job in jobs]
            for future in concurrent.futures.as_completed(futures):
                _, actual = future.result()
                executed_pins += actual

        base_job = next(job for job in jobs if job[0] == "base")
        sabotage_path = temp / "inverted-sentinel.pins.nibli"
        sabotage_path.write_text(
            ":expect-pins 1\n"
            "# Executable negative control: Adam is a prisoner in the base.\n\n"
            "? prisoner(Adam).\n"
            "# => FALSE\n",
            encoding="utf-8",
            newline="\n",
        )
        sabotage = subprocess.run(
            [str(pin_binary), "--kb", str(base_job[1]), str(sabotage_path)],
            cwd=ROOT,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        if sabotage.returncode == 0 or "FINDING" not in sabotage.stdout:
            tail = "\n".join(sabotage.stdout.splitlines()[-12:])
            raise RedTeamError(
                "executable inverted-sentinel control did not fail as a finding\n"
                + tail
            )
    return len(queries), executed_pins, 1


def markdown(value: object) -> str:
    return " ".join(str(value).split()).replace("|", "\\|")


def code(value: object) -> str:
    text = str(value)
    fence = "``" if "`" in text else "`"
    return f"{fence}{text}{fence}"


def bullets(values: Sequence[object]) -> list[str]:
    return [f"- {markdown(value)}" for value in values]


def render(
    source: Mapping[str, object],
    source_path: pathlib.Path,
    kb_path: pathlib.Path,
    ledger_path: pathlib.Path,
    assurance_path: pathlib.Path,
) -> str:
    postures = as_object(source["posture_meanings"], "posture_meanings")
    limits = as_object(source["limits"], "limits")
    lines = [
        f"<!-- SPDX-License-Identifier: {source['spdx']} -->",
        "<!-- Generated by new-book-plans/9-record-integrity-red-team.py; do not edit. -->",
        "",
        f"# {source['title']}",
        "",
        "## Verdict and scope",
        "",
        "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED — bounded gap and boundary evidence, not record-integrity assurance.**",
        "",
        markdown(source["acceptance_result"]["claim"]),
        "",
        "A green executable run means the release engine produced every reviewed",
        "consequence for the constructed snapshots. It does not authenticate those",
        "snapshots, attribute a write or absence, supersede the implemented T1/T3",
        "assurance case, or prove that an institution acts on a finding.",
        "",
        "| posture | meaning |",
        "| --- | --- |",
    ]
    for posture in sorted(POSTURE_KEYS):
        lines.append(f"| {code(posture)} | {markdown(postures[posture])} |")
    lines.extend(["", "## Limits", ""])
    for key in (
        "flat_snapshot",
        "attribution",
        "temporal_coverage",
        "liveness",
        "scope",
        "no_new_gate",
    ):
        lines.append(f"- **{key.replace('_', ' ').title()}:** {markdown(limits[key])}")

    lines.extend(["", "## Route postures", ""])
    for route in source["routes"]:
        delta_coverage = "; ".join(
            f"{code(premise)}: {', '.join(str(value) for value in polarities)}"
            for premise, polarities in route["tested_delta_polarities"].items()
        )
        lines.extend(
            [
                f"### {route['id']} — {route['title']}",
                "",
                f"- **Writable premise(s):** {', '.join(code(value) for value in route['premises'])}",
                f"- **Executed delta coverage:** {delta_coverage}",
                f"- **Assertion harm:** {markdown(route['assertion_harm'])}",
                f"- **Withholding/deletion harm:** {markdown(route['withholding_deletion_harm'])}",
                f"- **Claimant/public-power polarity:** {markdown(route['claimant_public_power_polarity'])}",
                f"- **Current detectability:** {markdown(route['current_detectability'])}",
                f"- **Safe default:** {markdown(route['safe_default'])}",
                f"- **Authorised-disposition boundary:** {markdown(route['authorised_disposition_boundary'])}",
                f"- **Opposite-failure test:** {markdown(route['opposite_failure_test'])}",
                f"- **Temporal status:** {markdown(route['temporal_status'])}",
                f"- **Residual limit:** {markdown(route['residual_limit'])}",
                f"- **Assurance or repair owner:** {code(route['owner_ref'])}",
                f"- **Executable scenarios:** {', '.join(code(value) for value in route['scenario_refs'])}",
                "",
            ]
        )

    lines.extend(
        [
            "## Executed snapshot manifest",
            "",
            "Each authored delta below is validated as exactly one ground atom. The",
            "full verifier executes the resulting ephemeral snapshot independently.",
            "",
            "| state | exact additions | exact deletions |",
            "| --- | --- | --- |",
        ]
    )
    for snapshot in source["snapshots"]:
        additions = "<br>".join(code(value) for value in snapshot["additions"]) or "â€”"
        deletions = "<br>".join(code(value) for value in snapshot["deletions"]) or "â€”"
        lines.append(f"| {code(snapshot['id'])} | {additions} | {deletions} |")

    lines.extend(
        [
            "",
            "## Executable scenario summary",
            "",
            "| scenario | route(s) | kind | result | attribution limit |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for scenario in source["scenarios"]:
        lines.append(
            f"| {code(scenario['id'])} {markdown(scenario['title'])} | "
            f"{', '.join(code(value) for value in scenario['route_refs'])} | "
            f"{code(scenario['kind'])} | {code(scenario['result'])} | "
            f"{code(scenario['attribution'])} |"
        )

    lines.extend(["", "## Executable scenario details", ""])
    for scenario in source["scenarios"]:
        lines.extend(
            [
                f"### {scenario['id']} — {scenario['title']}",
                "",
                f"**{str(scenario['result']).replace('_', ' ').upper()}**",
                "",
                f"- **Attribution:** {code(scenario['attribution'])}",
                f"- **States:** {', '.join(code(value) for value in scenario['state_refs'])}",
                f"- **Interpretation:** {markdown(scenario['interpretation'])}",
                f"- **Authorised-disposition boundary:** {markdown(scenario['authorised_disposition_boundary'])}",
                f"- **Opposite failure:** {markdown(scenario['opposite_failure'])}",
                f"- **Residual limit:** {markdown(scenario['residual_limit'])}",
                "",
                "| state | query | expected | purpose |",
                "| --- | --- | --- | --- |",
            ]
        )
        for query in scenario["queries"]:
            lines.append(
                f"| {code(query['state'])} | {code(query['expression'])} | "
                f"**{query['expected']}** | {markdown(query['purpose'])} |"
            )
        if scenario["comparisons"]:
            lines.extend(["", "**Discriminating flips**", ""])
            for item in scenario["comparisons"]:
                lines.append(
                    f"- {code(item['expression'])}: {code(item['from_state'])} "
                    f"{item['from_expected']} → {code(item['to_state'])} "
                    f"{item['to_expected']} — {markdown(item['claim'])}"
                )
        lines.extend(["", "**Preserved controls**", ""])
        for item in scenario["preserved_invariants"]:
            lines.append(
                f"- {code(item['expression'])} stays **{item['expected']}** from "
                f"{code(item['from_state'])} to {code(item['to_state'])} — "
                f"{markdown(item['claim'])}"
            )
        lines.append("")

    lines.extend(["## Flat-snapshot indistinguishability boundary", ""])
    lines.extend(
        [
            "Each case deliberately maps multiple real-world descriptions to one",
            "identical snapshot and query vector outside the currently witnessed",
            "temporal scopes. No extra fact identifies which world occurred.",
            "",
        ]
    )
    for entry in source["observational_equivalence"]:
        lines.extend(
            [
                f"### {entry['id']} — {entry['title']}",
                "",
                f"- **One snapshot:** {code(entry['snapshot_ref'])}",
                f"- **Worlds with the same record:** {'; '.join(markdown(value) for value in entry['world_descriptions'])}",
                f"- **Boundary:** {markdown(entry['boundary'])}",
                f"- **Prohibited inference:** {markdown(entry['prohibited_inference'])}",
                "",
                "| query | expected in every described world |",
                "| --- | --- |",
            ]
        )
        for query in entry["queries"]:
            lines.append(f"| {code(query['expression'])} | **{query['expected']}** |")
        lines.append("")

    handoff = source["temporal_handoff"]
    lines.extend(
        [
            "## Temporal assurance handoff",
            "",
            markdown(handoff["current_contract"]),
            "",
            f"- **Owner:** {code(handoff['owner_ref'])}",
            f"- **Owned executable cases:** {', '.join(code(value) for value in handoff['owned_cases'])}",
            f"- **Residual boundary:** {markdown(handoff['residual_boundary'])}",
        ]
    )

    lines.extend(
        [
            "",
            "## Narrowness impacts",
            "",
            "This red-team changes no formal rule. It exposed several prose claims as",
            "too broad; those claims were revised and scoped in the same change. Other",
            "standing claims remain true only with the limits recorded here.",
            "",
            "| artifact | current claim | classification | reason | future trigger |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for entry in source["narrowness_impacts"]:
        lines.append(
            f"| {code(entry['artifact_ref'])} | {markdown(entry['current_claim'])} | "
            f"{code(entry['classification'])} | {markdown(entry['reason'])} | "
            f"{markdown(entry['future_trigger'])} |"
        )

    acceptance = source["acceptance_result"]
    lines.extend(
        [
            "",
            "## Acceptance result",
            "",
            "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED.**",
            "",
            markdown(acceptance["claim"]),
            "",
            "This artifact does **not** establish:",
            "",
            *bullets(acceptance["does_not_establish"]),
            "",
            f"Remaining gap owner: {code(acceptance['remaining_owner'])}.",
            "",
            "## Maintenance",
            "",
            f"- Reviewed source: {code(source_path.as_posix())}.",
            f"- Constitution: {code(kb_path.as_posix())}, SHA-256 {code(source['constitution_sha256'])}.",
            f"- Assertion ledger: {code(ledger_path.as_posix())}, SHA-256 {code(source['assertion_surface_contracts_sha256'])}.",
            f"- Assurance source: {code(assurance_path.as_posix())}, SHA-256 {code(source['record_integrity_assurance_case_sha256'])}.",
            "- Regenerate only through `python3 new-book-plans/9-record-integrity-red-team.py`.",
            "- Fast freshness/schema check: `python3 new-book-plans/9-record-integrity-red-team.py --check`.",
            "- Executable check: `python3 new-book-plans/9-record-integrity-red-team.py --check --execute`.",
            "- The executable snapshots are temporary and remain outside chapter `:expect-pins` reconciliation.",
            "",
        ]
    )
    return "\n".join(lines)


def expect_failure(label: str, action: Callable[[], object]) -> None:
    try:
        action()
    except RedTeamError:
        return
    raise RedTeamError(f"negative control did not fail: {label}")


def negative_controls(
    source: dict[str, object],
    kb_text: str,
    kb_digest: str,
    ledger_digest: str,
    assurance_digest: str,
) -> int:
    controls = 0

    def validate(candidate: dict[str, object]) -> None:
        validate_source(candidate, kb_text, kb_digest, ledger_digest, assurance_digest)

    for key, label in (
        ("constitution_sha256", "constitution digest drift"),
        ("assertion_surface_contracts_sha256", "assertion-ledger digest drift"),
        ("record_integrity_assurance_case_sha256", "assurance-source digest drift"),
    ):
        changed = copy.deepcopy(source)
        changed[key] = "0" * 64
        expect_failure(label, lambda changed=changed: validate(changed))
        controls += 1

    changed = copy.deepcopy(source)
    changed["schema_version"] = True
    expect_failure("boolean schema version", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["routes"] = changed["routes"][1:]
    expect_failure("required route deleted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["scenarios"] = changed["scenarios"][1:]
    expect_failure("required scenario deleted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["routes"].append(copy.deepcopy(changed["routes"][0]))
    expect_failure("duplicate route ID", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["scenarios"][0]["route_refs"] = ["RT-99"]
    expect_failure("dangling route reference", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    del changed["routes"][0]["authorised_disposition_boundary"]
    expect_failure("missing authorised-disposition boundary", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["routes"][0]["premises"] = ["free", "invented_relation"]
    expect_failure("required premise coverage drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["routes"][0]["tested_delta_polarities"]["free"] = ["deletion"]
    expect_failure("declared delta coverage drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    snapshot = next(item for item in changed["snapshots"] if item["id"] == "free_adam")
    snapshot["additions"].append("person(Adam).")
    expect_failure("no-op existing addition", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    snapshot = next(item for item in changed["snapshots"] if item["id"] == "free_adam")
    snapshot["additions"] = ["free(Probe). person(Probe_Injected)."]
    expect_failure("multiple statements hidden in one addition", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    snapshot = next(item for item in changed["snapshots"] if item["id"] == "no_adulthood_hano")
    snapshot["deletions"] = ["at(NeverThere, GeneralAdulthood)."]
    expect_failure("deletion with zero exact matches", lambda: validate(changed))
    controls += 1

    duplicate_base = kb_text + "at(Hano, GeneralAdulthood).\n"
    snapshot = next(item for item in source["snapshots"] if item["id"] == "no_adulthood_hano")
    expect_failure(
        "deletion with multiple exact matches",
        lambda: apply_snapshot(duplicate_base, snapshot, "duplicate deletion control"),
    )
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-01")
    query = next(
        item
        for item in scenario["queries"]
        if item["state"] == "base" and item["expression"] == "prisoner(Adam)"
    )
    query["expected"] = "FALSE"
    expect_failure("reversed expected verdict", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-01")
    scenario["queries"][0]["expression"] = "free(Adam). ? person(Eve)"
    expect_failure("query statement injection", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-01")
    scenario["comparisons"] = []
    expect_failure("missing discriminating flip", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-01")
    scenario["comparisons"][0]["to_expected"] = scenario["comparisons"][0]["from_expected"]
    expect_failure("non-discriminating comparison", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-01")
    scenario["preserved_invariants"] = []
    expect_failure("missing preserved positive control", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    del changed["scenarios"][0]["opposite_failure"]
    expect_failure("missing opposite-failure analysis", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-07")
    scenario["state_refs"].remove("vex_judgment_only")
    expect_failure("incomplete two-entry matrix", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    scenario = next(item for item in changed["scenarios"] if item["id"] == "RS-03")
    scenario["attribution"] = "deletion_proved"
    expect_failure("disappearance falsely attributed", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["status"] = "general_temporal_assurance"
    expect_failure("bounded red-team promoted to general assurance", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["evidence_role"] = "supports_current"
    expect_failure("gap evidence promoted to assurance", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"] = changed["narrowness_impacts"][1:]
    expect_failure("standing narrowness claim omitted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"][0]["classification"] = "preserved_unqualified"
    expect_failure("unknown narrowness disposition", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["acceptance_result"]["does_not_establish"] = [
        value
        for value in changed["acceptance_result"]["does_not_establish"]
        if "recovery" not in value
    ]
    expect_failure("residual recovery boundary erased", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["routes"][0]["owner_ref"] = "TODO.md::heading that does not exist"
    expect_failure("dangling route owner", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["temporal_handoff"]["owned_cases"] = ["TA-02"]
    expect_failure("delegated temporal coverage erased", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["temporal_handoff"]["owner_ref"] = (
        "new-book-plans/12-temporal-assurance.py::heading that does not exist"
    )
    expect_failure("dangling temporal-assurance owner", lambda: validate(changed))
    controls += 1

    expect_failure(
        "duplicate JSON object key",
        lambda: json.loads(
            '{"status": "bounded", "status": "assured"}',
            object_pairs_hook=reject_duplicate_keys,
        ),
    )
    controls += 1
    return controls


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise RedTeamError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json(path: pathlib.Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_keys,
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise RedTeamError(f"cannot read {label} {path}: {exc}") from exc
    return as_object(value, label)


def default_pin() -> pathlib.Path:
    explicit = os.environ.get("NIBLI_PIN")
    if explicit:
        return pathlib.Path(explicit)
    on_path = shutil.which("nibli-pin")
    if on_path:
        return pathlib.Path(on_path)
    source = pathlib.Path(
        os.environ.get("NIBLI_SRC", str(pathlib.Path.home() / "projects/dhilipsiva/nibli"))
    )
    return source / "target/release/nibli-pin"


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, default=DEFAULT_SOURCE)
    parser.add_argument("--kb", type=pathlib.Path, default=DEFAULT_KB)
    parser.add_argument("--ledger", type=pathlib.Path, default=DEFAULT_LEDGER)
    parser.add_argument("--assurance", type=pathlib.Path, default=DEFAULT_ASSURANCE)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--pin", type=pathlib.Path, default=None)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args(argv)

    source_path = resolve(args.source)
    kb_path = resolve(args.kb)
    ledger_path = resolve(args.ledger)
    assurance_path = resolve(args.assurance)
    output_path = resolve(args.output)
    paths = [source_path, kb_path, ledger_path, assurance_path, output_path]
    for path in paths:
        repo_relative(path)
    if output_path in {source_path, kb_path, ledger_path, assurance_path}:
        raise RedTeamError("output path must not overwrite an authored source")
    if output_path.suffix.lower() != ".md":
        raise RedTeamError("output path must end in .md")

    source = load_json(source_path, "red-team source")
    try:
        kb_text = kb_path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise RedTeamError(f"cannot read constitution {kb_path}: {exc}") from exc
    kb_digest = sha256(kb_path)
    ledger_digest = sha256(ledger_path)
    assurance_digest = sha256(assurance_path)
    validate_source(source, kb_text, kb_digest, ledger_digest, assurance_digest)
    generated = render(
        source,
        repo_relative(source_path),
        repo_relative(kb_path),
        repo_relative(ledger_path),
        repo_relative(assurance_path),
    )
    controls = negative_controls(
        source, kb_text, kb_digest, ledger_digest, assurance_digest
    )

    snapshots_run = 0
    pins_run = 0
    execution_controls = 0
    should_execute = args.execute or not args.check
    if should_execute:
        pin = args.pin if args.pin is not None else default_pin()
        snapshots_run, pins_run, execution_controls = execute_scenarios(
            source, kb_text, pin
        )

    output_relative = repo_relative(output_path)
    if args.check:
        try:
            current = output_path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise RedTeamError(f"cannot read generated report {output_path}: {exc}") from exc
        if current != generated:
            raise RedTeamError(f"{output_relative} is STALE — rerun without --check")
        suffix = (
            f"; {snapshots_run} snapshots / {pins_run} pins execute; "
            f"{execution_controls} executable sabotage passes"
            if should_execute
            else "; execution skipped"
        )
        print(
            f"{output_relative} is current; {controls} structural negative controls pass{suffix}"
        )
        return 0

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(generated, encoding="utf-8", newline="\n")
    print(
        f"{output_relative}: regenerated after {snapshots_run} snapshots / {pins_run} pins; "
        f"{controls} structural and {execution_controls} executable negative controls pass"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RedTeamError as exc:
        print(f"9-record-integrity-red-team: {exc}", file=sys.stderr)
        raise SystemExit(1)
