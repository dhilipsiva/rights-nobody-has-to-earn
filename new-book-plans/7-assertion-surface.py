#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate and check Book 1's assertion-surface audit.

The engine answers which relations are rule-produced.  Article 0a answers which
canonical names may be asserted, and Article 0 independently answers which names
are conclusion-only.  The reviewed JSON ledger supplies the normative contracts
that none of those mechanical sources can derive.

Usage:
    python3 new-book-plans/7-assertion-surface.py
    python3 new-book-plans/7-assertion-surface.py --check
    python3 new-book-plans/7-assertion-surface.py --fingerprints

Relative paths are resolved from the repository root.  ``--check`` also runs the
negative controls for the inventory and schema gates.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import Callable, Iterable


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_KB = pathlib.Path("new-book-plans/constitution.nibli")
DEFAULT_CONTRACT = pathlib.Path("new-book-plans/assertion-surface-contracts.json")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/assertion-surface-audit.md")

CLASSIFICATIONS = {"derived_only", "mixed_base_fact", "pending_interface"}
DISPOSITIONS = {"patchable", "external", "deliberately_refused"}
OPERATIONS = {"assert", "withhold_or_delete"}
REQUIRED_TAG_BINDINGS = {
    "adulthood": {"at"},
    "amendment": {"adjust", "approves", "permanent", "suggest"},
    "epoch-carry": {"rotten"},
    "placement": {"at", "attack", "cruel", "injure", "put"},
    "public-body": {"public"},
    "release": {"free"},
    "roster-person": {"person"},
    "seating": {"choose"},
}
BUILTINS = {"equals"}
PLACEHOLDER = re.compile(r"^(?:tbd|todo|unknown|n/?a|pending)$", re.I)
NAME = re.compile(r"^[a-z_][a-z0-9_]*$")
CALL = re.compile(r"(?P<negative>~\s*)?(?P<name>[a-z_][a-z0-9_]*)\s*\(")
DECL = re.compile(r'^(admits|derived_only)\(\"([a-z_][a-z0-9_]*)\"\)$')
FLOOR = re.compile(
    r"^entitled\(\s*every\s+([a-z_][a-z0-9_]*)\s*,\s*"
    r"event\s*\{\s*([a-z_][a-z0-9_]*)\s*\([^)]*\)\s*\}\s*\)$"
)
EVERY = re.compile(r"\bevery\s+([a-z_][a-z0-9_]*)\b")


class AuditError(RuntimeError):
    """A contract, inventory, or generated-artifact failure."""


@dataclass(frozen=True)
class Statement:
    text: str
    line: int


@dataclass(frozen=True)
class Edge:
    dependency: str
    negative: bool


@dataclass
class SourceInventory:
    admitted: set[str] = field(default_factory=set)
    derived_only: set[str] = field(default_factory=set)
    ground_asserted: set[str] = field(default_factory=set)
    producers: dict[str, list[str]] = field(default_factory=lambda: defaultdict(list))
    consumers: dict[str, list[tuple[bool, str]]] = field(
        default_factory=lambda: defaultdict(list)
    )
    rules: list[str] = field(default_factory=list)
    facts: list[str] = field(default_factory=list)


@dataclass
class Inventory:
    strata: dict[str, int]
    derived: set[str]
    edges: dict[str, list[Edge]]
    admitted: set[str]
    derived_only: set[str]
    ground_asserted: set[str]
    producers: dict[str, list[str]]
    consumers: dict[str, list[tuple[bool, str]]]
    rules_sha256: str
    facts_sha256: str
    route_fingerprints: dict[str, str]
    statement_fingerprints: list[dict[str, object]]

    @property
    def writable(self) -> set[str]:
        return self.admitted - self.derived_only


def is_artifact(name: str) -> bool:
    return name == "event" or name.startswith("__abs_")


def resolve(path: pathlib.Path) -> pathlib.Path:
    return path if path.is_absolute() else ROOT / path


def sha256_json(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def engine_text(kb: pathlib.Path) -> str:
    cached = os.environ.get("NIBLI_STRATA_FILE")
    if cached:
        try:
            return pathlib.Path(cached).read_text(encoding="utf-8")
        except OSError as exc:
            raise AuditError(f"cannot read NIBLI_STRATA_FILE {cached}: {exc}") from exc

    pin = (
        os.environ.get("NIBLI_PIN")
        or shutil.which("nibli-pin")
        or os.path.expanduser("~/projects/dhilipsiva/nibli/target/release/nibli-pin")
    )
    if not os.path.exists(pin):
        raise AuditError(
            f"no nibli-pin at {pin} — build it release, or set NIBLI_PIN"
        )
    result = subprocess.run(
        [pin, "--strata", "--kb", str(kb)],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise AuditError(
            "nibli-pin --strata failed:\n" + result.stdout + result.stderr
        )
    return result.stdout


def parse_engine(text: str) -> tuple[dict[str, int], set[str], dict[str, list[Edge]]]:
    strata: dict[str, int] = {}
    derived: set[str] = set()
    edges: dict[str, list[Edge]] = {}
    for line in text.splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        name, stratum, kind = parts[0], parts[1], parts[2]
        if is_artifact(name) or name in BUILTINS:
            continue
        try:
            strata[name] = int(stratum)
        except ValueError as exc:
            raise AuditError(f"invalid stratum row: {line}") from exc
        if kind != "base":
            derived.add(name)
        parsed: list[Edge] = []
        if len(parts) > 3 and parts[3]:
            for raw in parts[3].split(","):
                raw = raw.strip()
                if not raw:
                    continue
                if raw[0] not in "+-":
                    raise AuditError(f"invalid signed edge in strata row: {line}")
                dependency = raw[1:]
                if is_artifact(dependency) or dependency in BUILTINS:
                    continue
                parsed.append(Edge(dependency, raw[0] == "-"))
        edges[name] = parsed
    if not strata:
        raise AuditError("nibli-pin --strata produced no relation rows")
    return strata, derived, edges


def lex_statements(source: str) -> list[Statement]:
    """Return active dot-terminated statements, stripping comments safely."""
    statements: list[Statement] = []
    buf: list[str] = []
    in_string = False
    escaped = False
    in_comment = False
    line = 1
    start_line: int | None = None

    for char in source:
        if in_comment:
            if char == "\n":
                in_comment = False
                if buf:
                    buf.append(" ")
                line += 1
            continue
        if not in_string and char == "#":
            in_comment = True
            continue
        if char == "\n":
            if buf:
                buf.append(" ")
            line += 1
            escaped = False
            continue
        if start_line is None and not char.isspace():
            start_line = line
        if char == '"' and not escaped:
            in_string = not in_string
        if char == "." and not in_string:
            normalized = " ".join("".join(buf).split())
            if normalized:
                statements.append(Statement(normalized, start_line or line))
            buf = []
            start_line = None
            escaped = False
            continue
        buf.append(char)
        escaped = char == "\\" and not escaped
        if char != "\\":
            escaped = False

    residue = "".join(buf).strip()
    if in_string:
        raise AuditError("unterminated string while reading constitution")
    if residue:
        raise AuditError(
            f"unterminated active statement at line {start_line or line}: {residue[:80]}"
        )
    return statements


def canonical(name: str, aliases: dict[str, str]) -> str:
    return aliases.get(name, name)


def calls(text: str, aliases: dict[str, str]) -> list[tuple[str, bool]]:
    return [
        (canonical(match.group("name"), aliases), bool(match.group("negative")))
        for match in CALL.finditer(text)
    ]


def parse_source(
    source: str,
    known_relations: set[str],
    expected_heads: set[str],
    aliases: dict[str, str],
) -> SourceInventory:
    result = SourceInventory()
    declarations: dict[str, set[str]] = {
        "admits": result.admitted,
        "derived_only": result.derived_only,
    }
    executable_seen = False

    for statement in lex_statements(source):
        declaration = DECL.fullmatch(statement.text)
        if declaration:
            kind, raw_name = declaration.groups()
            name = canonical(raw_name, aliases)
            if executable_seen:
                raise AuditError(
                    f"late {kind} declaration for {name} at constitution line "
                    f"{statement.line}; declarations must precede executable content"
                )
            if name in declarations[kind]:
                raise AuditError(f"duplicate {kind} declaration for {name}")
            declarations[kind].add(name)
            continue

        executable_seen = True
        text = statement.text
        producers: list[str] = []
        consumers: list[tuple[str, bool]] = []
        ground_names: list[str] = []
        is_rule = False

        if "->" in text:
            body, head = text.rsplit("->", 1)
            producers = [name for name, _ in calls(head, aliases)]
            consumers = calls(body, aliases)
            is_rule = True
        else:
            floor = FLOOR.fullmatch(text)
            if floor:
                domain, actuality = floor.groups()
                producers = ["entitled", canonical(actuality, aliases)]
                consumers = [(canonical(domain, aliases), False)]
                is_rule = True
            else:
                universal = EVERY.search(text)
                outer = calls(text, aliases)
                if universal and outer:
                    producers = [outer[0][0]]
                    consumers = [(canonical(universal.group(1), aliases), False)]
                    is_rule = True
                elif outer:
                    ground_names = [
                        name for name, _ in outer if name not in BUILTINS
                    ]
                    result.ground_asserted.update(ground_names)
                    result.facts.append(text)

        mentioned = {name for name, _ in consumers} | set(producers) | set(ground_names)
        unknown = mentioned - known_relations - BUILTINS
        if unknown:
            raise AuditError(
                f"source statement at line {statement.line} uses relations absent "
                f"from engine inventory or alias contract: {', '.join(sorted(unknown))}"
            )
        if is_rule:
            if not producers:
                raise AuditError(f"rule at line {statement.line} has no parsed head: {text}")
            result.rules.append(text)
            for name in producers:
                result.producers[name].append(text)
            for name, negative in consumers:
                if name not in BUILTINS:
                    result.consumers[name].append((negative, text))
        elif not outer:
            raise AuditError(
                f"unrecognized active statement at line {statement.line}: {text}"
            )

    source_heads = set(result.producers)
    if source_heads != expected_heads:
        missing = sorted(expected_heads - source_heads)
        extra = sorted(source_heads - expected_heads)
        details = []
        if missing:
            details.append("engine-only heads: " + ", ".join(missing))
        if extra:
            details.append("source-only heads: " + ", ".join(extra))
        raise AuditError("authored-head reconciliation failed: " + "; ".join(details))
    return result


def route_payload(
    relation: str,
    source: SourceInventory,
    edges: dict[str, list[Edge]],
) -> dict[str, object]:
    readers: list[tuple[str, bool]] = []
    for head, dependencies in edges.items():
        for edge in dependencies:
            if edge.dependency == relation:
                readers.append((head, edge.negative))
    return {
        "source_producers": sorted(source.producers.get(relation, [])),
        "source_consumers": sorted(
            {
                f"{'-' if negative else '+'}\t{rule}"
                for negative, rule in source.consumers.get(relation, [])
            }
        ),
        "engine_dependencies": sorted(
            f"{'-' if edge.negative else '+'}{edge.dependency}"
            for edge in edges.get(relation, [])
        ),
        "engine_readers": sorted(
            f"{'-' if negative else '+'}{head}" for head, negative in readers
        ),
    }


def statement_fingerprint_records(source_text: str) -> list[dict[str, object]]:
    occurrences: dict[str, int] = defaultdict(int)
    records = []
    for statement in lex_statements(source_text):
        occurrence = occurrences[statement.text]
        occurrences[statement.text] += 1
        records.append({
            "id": sha256_json([statement.text, occurrence]),
            "statement_sha256": sha256_json(statement.text),
            "occurrence": occurrence,
            "kind": (
                "rule" if statement.text.lstrip().startswith(("all ", "any "))
                else "fact"
            ),
            "statement": statement.text,
        })
    return records


def make_inventory(kb: pathlib.Path, contract: dict[str, object]) -> Inventory:
    strata, derived, edges = parse_engine(engine_text(kb))
    aliases = contract.get("aliases", {})
    if not isinstance(aliases, dict) or any(
        not isinstance(key, str)
        or not isinstance(value, str)
        or not NAME.fullmatch(key)
        or not NAME.fullmatch(value)
        for key, value in aliases.items()
    ):
        raise AuditError("aliases must map relation names to relation names")
    alias_targets = set(aliases.values())
    if not alias_targets <= set(strata):
        raise AuditError(
            "aliases target relations absent from the engine inventory: "
            + ", ".join(sorted(alias_targets - set(strata)))
        )
    alias_conflicts = set(aliases) & set(strata)
    if alias_conflicts:
        raise AuditError(
            "alias names collide with canonical engine relations: "
            + ", ".join(sorted(alias_conflicts))
        )
    source_text = kb.read_text(encoding="utf-8")
    source = parse_source(
        source_text,
        set(strata),
        derived,
        aliases,
    )
    relations = derived | (source.admitted - source.derived_only)
    fingerprints = {
        relation: sha256_json(route_payload(relation, source, edges))
        for relation in sorted(relations)
    }
    return Inventory(
        strata=strata,
        derived=derived,
        edges=edges,
        admitted=source.admitted,
        derived_only=source.derived_only,
        ground_asserted=source.ground_asserted,
        producers=dict(source.producers),
        consumers=dict(source.consumers),
        rules_sha256=sha256_json(sorted(source.rules)),
        facts_sha256=sha256_json(sorted(source.facts)),
        route_fingerprints=fingerprints,
        statement_fingerprints=statement_fingerprint_records(source_text),
    )


def require_text(value: object, path: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise AuditError(f"{path} must be non-empty text")
    if PLACEHOLDER.fullmatch(value.strip()):
        raise AuditError(f"{path} contains placeholder value {value!r}")
    return value.strip()


def validate_reference(value: object, path: str) -> str:
    reference = require_text(value, path)
    if "::" not in reference:
        raise AuditError(f"{path} must use path::stable text, not a line number")
    raw_file, needle = reference.split("::", 1)
    if not raw_file.strip() or not needle.strip():
        raise AuditError(f"{path} must name both a file and non-empty stable text")
    target = resolve(pathlib.Path(raw_file)).resolve()
    try:
        target.relative_to(ROOT.resolve())
    except ValueError as exc:
        raise AuditError(f"{path} must reference a repository-local file") from exc
    if not target.is_file():
        raise AuditError(f"{path} references missing file {raw_file}")
    occurrences = target.read_text(encoding="utf-8").count(needle)
    if occurrences == 0:
        raise AuditError(f"{path} reference text is stale in {raw_file}: {needle!r}")
    if occurrences != 1:
        raise AuditError(
            f"{path} reference text must identify one location in {raw_file}; "
            f"matched {occurrences}"
        )
    return reference


def shortest_path(
    inventory: Inventory, source: str, target: str
) -> list[tuple[str, str, bool]] | None:
    if source == target:
        return []
    reverse: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    for head, dependencies in inventory.edges.items():
        for edge in dependencies:
            reverse[edge.dependency].append((head, edge.negative))
    queue: deque[tuple[str, list[tuple[str, str, bool]]]] = deque([(source, [])])
    seen = {source}
    while queue:
        current, path = queue.popleft()
        for head, negative in sorted(reverse.get(current, [])):
            step = (current, head, negative)
            if head == target:
                return path + [step]
            if head not in seen:
                seen.add(head)
                queue.append((head, path + [step]))
    return None


def validate_scenario(
    relation: str,
    expected_operation: str,
    scenario: object,
    inventory: Inventory,
    path: str,
) -> None:
    if not isinstance(scenario, dict):
        raise AuditError(f"{path} must be an object")
    operation = require_text(scenario.get("operation"), f"{path}.operation")
    if operation not in OPERATIONS:
        raise AuditError(f"{path}.operation must be one of {sorted(OPERATIONS)}")
    if operation != expected_operation:
        raise AuditError(f"{path}.operation must be {expected_operation!r}")
    operations = scenario.get("operations")
    if not isinstance(operations, list) or not operations:
        raise AuditError(f"{path}.operations must be a non-empty list")
    normalized_operations: list[str] = []
    for index, value in enumerate(operations):
        normalized_operations.append(
            require_text(value, f"{path}.operations[{index}]")
        )
    if len(normalized_operations) != len(set(normalized_operations)):
        raise AuditError(f"{path}.operations contains duplicates")
    relation_call = re.compile(rf"\b{re.escape(relation)}\s*\(")
    if not any(relation_call.search(value) for value in normalized_operations):
        raise AuditError(
            f"{path}.operations must include an operation on {relation}(...)"
        )
    require_text(scenario.get("effect"), f"{path}.effect")
    target = require_text(scenario.get("target"), f"{path}.target")
    if target not in inventory.derived:
        raise AuditError(f"{path}.target {target!r} is not a current derived relation")
    route = shortest_path(inventory, relation, target)
    if route is None:
        raise AuditError(
            f"{path}.target {target!r} has no engine dependency path from {relation!r}"
        )
    validate_reference(scenario.get("evidence_ref"), f"{path}.evidence_ref")


def validate_contract(contract: dict[str, object], inventory: Inventory) -> None:
    if contract.get("schema_version") != 1:
        raise AuditError("schema_version must be 1")
    if contract.get("spdx") != "CC-BY-4.0":
        raise AuditError("contract spdx must be CC-BY-4.0")
    require_text(contract.get("cheapest_harm_metric"), "cheapest_harm_metric")
    disposition_meanings = contract.get("risk_disposition_meanings")
    if not isinstance(disposition_meanings, dict) or set(disposition_meanings) != DISPOSITIONS:
        raise AuditError(
            "risk_disposition_meanings must define every allowed disposition exactly"
        )
    for disposition, meaning in disposition_meanings.items():
        require_text(meaning, f"risk_disposition_meanings.{disposition}")

    derived_entries = contract.get("derived_relations")
    if not isinstance(derived_entries, dict):
        raise AuditError("derived_relations must be an object")
    actual_names = set(inventory.derived)
    registered_names = set(derived_entries)
    if actual_names != registered_names:
        missing = sorted(actual_names - registered_names)
        extra = sorted(registered_names - actual_names)
        detail = []
        if missing:
            detail.append("unclassified derived relations: " + ", ".join(missing))
        if extra:
            detail.append("stale derived contracts: " + ", ".join(extra))
        raise AuditError("; ".join(detail))

    ids: set[str] = set()
    for relation in sorted(actual_names):
        path = f"derived_relations.{relation}"
        entry = derived_entries[relation]
        if not isinstance(entry, dict):
            raise AuditError(f"{path} must be an object")
        classification = require_text(entry.get("classification"), f"{path}.classification")
        if classification not in CLASSIFICATIONS:
            raise AuditError(f"{path}.classification must be one of {sorted(CLASSIFICATIONS)}")
        contract_id = require_text(entry.get("contract_id"), f"{path}.contract_id")
        if contract_id in ids:
            raise AuditError(f"duplicate contract_id {contract_id!r}")
        ids.add(contract_id)
        validate_reference(entry.get("decision_ref"), f"{path}.decision_ref")
        expected = entry.get("expected")
        if not isinstance(expected, dict):
            raise AuditError(f"{path}.expected must be an object")
        actual = {
            "rule_produced": True,
            "admitted": relation in inventory.admitted,
            "derived_only": relation in inventory.derived_only,
            "ground_asserted": relation in inventory.ground_asserted,
        }
        if expected != actual:
            raise AuditError(
                f"{relation} posture changed: expected {expected}, actual {actual}"
            )
        if classification == "derived_only" and not actual["derived_only"]:
            raise AuditError(f"{relation} is classified derived_only without the declaration")
        if classification == "mixed_base_fact" and not (
            actual["admitted"] and not actual["derived_only"]
        ):
            raise AuditError(f"{relation} mixed contract does not match raw posture")
        if classification == "pending_interface" and (
            actual["admitted"] or actual["derived_only"]
        ):
            raise AuditError(f"{relation} pending contract does not match raw posture")

    undeclared_guards = inventory.derived_only - inventory.derived
    if undeclared_guards:
        raise AuditError(
            "derived_only declarations without a current producer: "
            + ", ".join(sorted(undeclared_guards))
        )

    additional = contract.get("additional_writable_channels")
    if additional != []:
        raise AuditError(
            "additional_writable_channels must currently be []; add schema support "
            "and a named channel contract before using it"
        )
    premise_entries = contract.get("premises")
    if not isinstance(premise_entries, dict):
        raise AuditError("premises must be an object")
    expected_premises = inventory.writable
    registered_premises = set(premise_entries)
    if expected_premises != registered_premises:
        missing = sorted(expected_premises - registered_premises)
        extra = sorted(registered_premises - expected_premises)
        detail = []
        if missing:
            detail.append("unreviewed writable premises: " + ", ".join(missing))
        if extra:
            detail.append("stale/non-writable premise contracts: " + ", ".join(extra))
        raise AuditError("; ".join(detail))

    seen_tags: set[str] = set()
    required_fields = (
        "claimed_actor",
        "tuple_claim",
        "current_writer_authority",
        "required_writer_authority",
        "current_provenance",
        "required_provenance",
        "current_challenge_route",
        "required_challenge_route",
    )
    for relation in sorted(expected_premises):
        path = f"premises.{relation}"
        entry = premise_entries[relation]
        if not isinstance(entry, dict):
            raise AuditError(f"{path} must be an object")
        for field_name in required_fields:
            require_text(entry.get(field_name), f"{path}.{field_name}")
        validate_scenario(
            relation,
            "assert",
            entry.get("cheapest_harm"),
            inventory,
            f"{path}.cheapest_harm",
        )
        validate_scenario(
            relation,
            "withhold_or_delete",
            entry.get("withholding_deletion_harm"),
            inventory,
            f"{path}.withholding_deletion_harm",
        )
        dispositions_raw = entry.get("risk_dispositions")
        if not isinstance(dispositions_raw, list) or not dispositions_raw:
            raise AuditError(f"{path}.risk_dispositions must be a non-empty list")
        dispositions = [
            require_text(value, f"{path}.risk_dispositions[{index}]")
            for index, value in enumerate(dispositions_raw)
        ]
        if len(dispositions) != len(set(dispositions)):
            raise AuditError(f"{path}.risk_dispositions contains duplicates")
        unknown_dispositions = set(dispositions) - DISPOSITIONS
        if unknown_dispositions:
            raise AuditError(
                f"{path}.risk_dispositions contains invalid values: "
                + ", ".join(sorted(unknown_dispositions))
            )
        refused_alternative = entry.get("refused_alternative")
        if "deliberately_refused" in dispositions:
            require_text(refused_alternative, f"{path}.refused_alternative")
        elif refused_alternative is not None:
            raise AuditError(
                f"{path}.refused_alternative is present without deliberately_refused"
            )
        validate_reference(entry.get("owner_ref"), f"{path}.owner_ref")
        tags = entry.get("tags")
        if not isinstance(tags, list) or not tags:
            raise AuditError(f"{path}.tags must be a non-empty list")
        normalized_tags: list[str] = []
        for index, tag in enumerate(tags):
            normalized_tags.append(require_text(tag, f"{path}.tags[{index}]"))
        if len(normalized_tags) != len(set(normalized_tags)):
            raise AuditError(f"{path}.tags contains duplicates")
        seen_tags.update(normalized_tags)

    required_tags_raw = contract.get("required_semantic_tags")
    if not isinstance(required_tags_raw, list):
        raise AuditError("required_semantic_tags must be a list")
    required_tags = {
        require_text(value, f"required_semantic_tags[{index}]")
        for index, value in enumerate(required_tags_raw)
    }
    if len(required_tags) != len(required_tags_raw):
        raise AuditError("required_semantic_tags contains duplicates")
    if required_tags != set(REQUIRED_TAG_BINDINGS):
        raise AuditError(
            "required_semantic_tags must name the mandated audit tags exactly"
        )
    missing_tags = required_tags - seen_tags
    if missing_tags:
        raise AuditError(
            "required semantic tags have no premise row: " + ", ".join(sorted(missing_tags))
        )
    for tag, relations in REQUIRED_TAG_BINDINGS.items():
        missing_relations = sorted(
            relation
            for relation in relations
            if tag not in premise_entries[relation]["tags"]
        )
        if missing_relations:
            raise AuditError(
                f"semantic tag {tag!r} missing from required premise rows: "
                + ", ".join(missing_relations)
            )

    expected_rules = require_text(contract.get("rules_sha256"), "rules_sha256")
    if expected_rules != inventory.rules_sha256:
        raise AuditError(
            "authored-rule fingerprint changed: expected "
            f"{expected_rules}, actual {inventory.rules_sha256}; review changed "
            "producers, consumers, constants, bindings, and polarity before updating"
        )
    expected_facts = require_text(contract.get("facts_sha256"), "facts_sha256")
    if expected_facts != inventory.facts_sha256:
        raise AuditError(
            "authored-fact fingerprint changed: expected "
            f"{expected_facts}, actual {inventory.facts_sha256}; review every "
            "current-snapshot cheapest-harm scenario before updating"
        )
    expected_routes = contract.get("route_fingerprints")
    if not isinstance(expected_routes, dict):
        raise AuditError("route_fingerprints must be an object")
    if set(expected_routes) != set(inventory.route_fingerprints):
        raise AuditError("route_fingerprints keys do not match the audited relation surface")
    drift = [
        relation
        for relation, actual in inventory.route_fingerprints.items()
        if expected_routes.get(relation) != actual
    ]
    if drift:
        raise AuditError(
            "producer/consumer route changed without reviewed fingerprint update: "
            + ", ".join(sorted(drift))
        )


def path_text(inventory: Inventory, source: str, target: str) -> str:
    path = shortest_path(inventory, source, target)
    if path is None:
        return "no route"
    if not path:
        return f"`{source}` (direct writable relation)"
    rendered = f"`{source}`"
    for _, head, negative in path:
        rendered += f" {'─| ' if negative else '→ '}`{head}`"
    return rendered


def readers_text(inventory: Inventory, relation: str) -> str:
    readers: list[str] = []
    for head, dependencies in inventory.edges.items():
        for edge in dependencies:
            if edge.dependency == relation:
                readers.append(f"{'negative' if edge.negative else 'positive'} `{head}`")
    return ", ".join(sorted(readers)) or "none"


def yes_no(value: bool) -> str:
    return "yes" if value else "no"


def esc(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", "<br>")


def render(contract: dict[str, object], inventory: Inventory) -> str:
    derived_entries = contract["derived_relations"]
    premise_entries = contract["premises"]
    lines = [
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->",
        "<!-- GENERATED by 7-assertion-surface.py; edit "
        "assertion-surface-contracts.json, not this file. -->",
        "",
        "# Assertion Surface and High-Consequence Premise Audit",
        "",
        "This technical artifact reconciles three independent facts: what the engine",
        "reports as rule-produced, what Article 0a admits as ground vocabulary, and",
        "what Article 0 reserves for rule conclusions. The reviewed contract ledger",
        "supplies authority, provenance, harm, challenge, and risk judgments that the",
        "engine cannot derive.",
        "",
        "Run `python3 new-book-plans/7-assertion-surface.py --check`. The verifier",
        "also runs the check and its negative controls. A changed rule, ground fact,",
        "constant, binding, polarity, declaration, or relation requires an explicit ledger",
        "review before this report can be regenerated.",
        "After that review, `--fingerprints` prints candidate digests; it never updates",
        "the contract ledger.",
        "",
        "## Measurement contract",
        "",
        f"- Cheapest harm means: {contract['cheapest_harm_metric']}",
        "- Risk dispositions mean:",
        *[
            f"  - `{disposition}` — {contract['risk_disposition_meanings'][disposition]}"
            for disposition in sorted(DISPOSITIONS)
        ],
        "- `→` is a positive dependency; `─|` is a negative dependency, where",
        "  asserting the premise can suppress the downstream conclusion.",
        f"- Authored-rule fingerprint: `{inventory.rules_sha256}`.",
        f"- Authored-fact fingerprint: `{inventory.facts_sha256}`.",
        "- Rule-head writability remains open for every derived relation;",
        "  `derived_only` blocks ground assertions, not rules.",
        "",
        "## Derived-relation assertion posture",
        "",
        "| relation | stratum | admitted | derived-only | ground facts | "
        "classification | contract | direct readers |",
        "| --- | ---: | --- | --- | --- | --- | --- | --- |",
    ]
    for relation in sorted(inventory.derived):
        entry = derived_entries[relation]
        lines.append(
            f"| `{relation}` | {inventory.strata[relation]} | "
            f"{yes_no(relation in inventory.admitted)} | "
            f"{yes_no(relation in inventory.derived_only)} | "
            f"{yes_no(relation in inventory.ground_asserted)} | "
            f"`{entry['classification']}` | `{entry['contract_id']}` | "
            f"{readers_text(inventory, relation)} |"
        )

    lines.extend(
        [
            "",
            "## Writable-premise index",
            "",
            "The effective ground-writable surface is the active `admits` roster minus",
            "any active `derived_only` override, plus explicitly registered exceptional",
            "channels. No exceptional channel exists in the current contract.",
            "",
            "| premise | tags | cheapest target | dispositions | direct readers | "
            "route fingerprint |",
            "| --- | --- | --- | --- | --- | --- |",
        ]
    )
    for relation in sorted(inventory.writable):
        entry = premise_entries[relation]
        target = entry["cheapest_harm"]["target"]
        lines.append(
            f"| `{relation}` | {esc(', '.join(entry['tags']))} | `{target}` | "
            f"{esc(', '.join(entry['risk_dispositions']))} | "
            f"{readers_text(inventory, relation)} | "
            f"`{inventory.route_fingerprints[relation][:16]}` |"
        )

    lines.extend(["", "## Premise contracts", ""])
    for relation in sorted(inventory.writable):
        entry = premise_entries[relation]
        cheap = entry["cheapest_harm"]
        withheld = entry["withholding_deletion_harm"]
        lines.extend(
            [
                f"### `{relation}`",
                "",
                f"- **Tuple claim:** {entry['tuple_claim']}",
                f"- **Claimed actor:** {entry['claimed_actor']}",
                f"- **Current writer/authority:** {entry['current_writer_authority']}",
                f"- **Required writer/authority:** {entry['required_writer_authority']}",
                f"- **Current provenance:** {entry['current_provenance']}",
                f"- **Required provenance:** {entry['required_provenance']}",
                f"- **Cheapest harmful {cheap['operation']}:** "
                f"{len(cheap['operations'])} operation(s): "
                f"{'; '.join(cheap['operations'])}. {cheap['effect']} "
                f"Structural route: {path_text(inventory, relation, cheap['target'])}. "
                f"Evidence: `{cheap['evidence_ref']}`.",
                f"- **Withholding/deletion harm:** "
                f"{len(withheld['operations'])} operation(s): "
                f"{'; '.join(withheld['operations'])}. {withheld['effect']} "
                f"Structural route: {path_text(inventory, relation, withheld['target'])}. "
                f"Evidence: `{withheld['evidence_ref']}`.",
                f"- **Current challenge route:** {entry['current_challenge_route']}",
                f"- **Required challenge route:** {entry['required_challenge_route']}",
                f"- **Risk disposition:** {', '.join(entry['risk_dispositions'])}.",
                *(
                    [f"- **Refused alternative:** {entry['refused_alternative']}"]
                    if "deliberately_refused" in entry["risk_dispositions"]
                    else []
                ),
                f"- **Owner:** `{entry['owner_ref']}`.",
                f"- **Reviewed route fingerprint:** `{inventory.route_fingerprints[relation]}`.",
                "",
            ]
        )

    lines.extend(
        [
            "## Limits",
            "",
            "This audit proves inventory completeness and makes reviewed assumptions",
            "drift-sensitive. Dependency reach is structural; it does not authenticate a",
            "fact, prove a scenario's real-world truth, or establish that an external",
            "clock or record advances. The operation sets remain a reviewed threat-model",
            "inventory rather than executable proof. The bounded record-integrity red-team",
            "executes only its named flat-snapshot release, adulthood, roster, relief,",
            "and forgiveness subset; the temporal suite separately owns carry, order,",
            "and renewal cases. All other ledger scenarios remain reviewed-only. The",
            "generated assurance case owns the wider control argument and records that",
            "overall deployed-record assurance is not established despite the staged",
            "T1/T2/T3 repository result.",
            "",
        ]
    )
    return "\n".join(lines)


def expect_failure(label: str, action: Callable[[], object]) -> None:
    try:
        action()
    except AuditError:
        return
    raise AuditError(f"negative control did not fail: {label}")


def negative_controls(
    contract: dict[str, object], inventory: Inventory, source_text: str
) -> int:
    controls = 0
    derived_entries = contract["derived_relations"]
    premise_entries = contract["premises"]
    guarded_relation = next(
        (
            relation
            for relation, entry in sorted(derived_entries.items())
            if entry["classification"] == "derived_only"
        ),
        None,
    )
    pending_relation = next(
        (
            relation
            for relation, entry in sorted(derived_entries.items())
            if entry["classification"] == "pending_interface"
        ),
        None,
    )
    premise_relation = sorted(inventory.writable)[0]
    route_relation = sorted(inventory.route_fingerprints)[0]
    refused_relation = next(
        (
            relation
            for relation, entry in sorted(premise_entries.items())
            if "deliberately_refused" in entry["risk_dispositions"]
        ),
        None,
    )
    unregistered_head = "audit_probe_result"
    while unregistered_head in inventory.strata:
        unregistered_head += "_next"
    unregistered_ground = "audit_probe_ground"
    while unregistered_ground in inventory.admitted:
        unregistered_ground += "_next"

    changed = copy.deepcopy(inventory)
    changed.derived = set(changed.derived) | {unregistered_head}
    changed.strata = dict(changed.strata) | {unregistered_head: 0}
    changed.edges = dict(changed.edges) | {unregistered_head: []}
    expect_failure("new unclassified rule head", lambda: validate_contract(contract, changed))
    controls += 1

    if guarded_relation is not None:
        changed = copy.deepcopy(inventory)
        changed.admitted = set(changed.admitted) ^ {guarded_relation}
        expect_failure(
            "raw admits drift under derived_only",
            lambda: validate_contract(contract, changed),
        )
        controls += 1

    if pending_relation is not None:
        changed = copy.deepcopy(inventory)
        changed.admitted = set(changed.admitted) | {pending_relation}
        expect_failure(
            "pending relation admitted", lambda: validate_contract(contract, changed)
        )
        controls += 1

    changed = copy.deepcopy(inventory)
    changed.admitted = set(changed.admitted) | {unregistered_ground}
    expect_failure(
        "new ground-only admitted relation",
        lambda: validate_contract(contract, changed),
    )
    controls += 1

    changed = copy.deepcopy(inventory)
    changed.admitted = set(changed.admitted) - {premise_relation}
    expect_failure("admission removed", lambda: validate_contract(contract, changed))
    controls += 1

    changed = copy.deepcopy(inventory)
    changed.route_fingerprints = dict(changed.route_fingerprints)
    changed.route_fingerprints[route_relation] = "0" * 64
    expect_failure("producer/consumer route drift", lambda: validate_contract(contract, changed))
    controls += 1

    changed = copy.deepcopy(inventory)
    changed.facts_sha256 = "0" * 64
    expect_failure("current-snapshot fact drift", lambda: validate_contract(contract, changed))
    controls += 1

    changed_contract = copy.deepcopy(contract)
    del changed_contract["premises"][premise_relation]["required_provenance"]
    expect_failure(
        "missing semantic field", lambda: validate_contract(changed_contract, inventory)
    )
    controls += 1

    changed_contract = copy.deepcopy(contract)
    changed_contract["premises"][premise_relation]["cheapest_harm"][
        "operation"
    ] = "withhold_or_delete"
    expect_failure(
        "scenario operation swapped",
        lambda: validate_contract(changed_contract, inventory),
    )
    controls += 1

    changed_contract = copy.deepcopy(contract)
    changed_contract["premises"][premise_relation]["cheapest_harm"][
        "evidence_ref"
    ] = "TODO.md::"
    expect_failure(
        "empty reference anchor",
        lambda: validate_contract(changed_contract, inventory),
    )
    controls += 1

    changed_contract = copy.deepcopy(contract)
    changed_contract["premises"][premise_relation]["cheapest_harm"]["operations"] = [
        "assert `unrelated(Fresh)`"
    ]
    expect_failure(
        "scenario omits audited relation",
        lambda: validate_contract(changed_contract, inventory),
    )
    controls += 1

    changed_contract = copy.deepcopy(contract)
    operation = changed_contract["premises"][premise_relation]["cheapest_harm"][
        "operations"
    ][0]
    changed_contract["premises"][premise_relation]["cheapest_harm"]["operations"] = [
        operation,
        operation,
    ]
    expect_failure(
        "duplicate scenario operation",
        lambda: validate_contract(changed_contract, inventory),
    )
    controls += 1

    if refused_relation is not None:
        changed_contract = copy.deepcopy(contract)
        changed_contract["premises"][refused_relation].pop("refused_alternative")
        expect_failure(
            "unexplained deliberate refusal",
            lambda: validate_contract(changed_contract, inventory),
        )
        controls += 1

    baseline = [statement.text for statement in lex_statements(source_text)]
    commented = [
        statement.text
        for statement in lex_statements(
            source_text
            + '\n# admits("phantom"). derived_only("phantom").\n'
            + '# all $x: person($x) -> phantom($x).\n'
        )
    ]
    if commented != baseline:
        raise AuditError("negative control failed: commented pseudo-statements were active")
    controls += 1

    sample = 'admits("alpha"). admits("beta"). fact(One). derived_only("late").'
    sample_statements = lex_statements(sample)
    names = [DECL.fullmatch(item.text).group(2) for item in sample_statements[:2]]
    if names != ["alpha", "beta"]:
        raise AuditError("negative control failed: same-line declarations were missed")
    controls += 1

    compound = parse_source(
        "alpha(One) & beta(Two).",
        {"alpha", "beta"},
        set(),
        {},
    )
    if compound.ground_asserted != {"alpha", "beta"}:
        raise AuditError("negative control failed: compound ground facts were missed")
    controls += 1

    expect_failure(
        "late declaration",
        lambda: parse_source(sample, {"fact"}, set(), {}),
    )
    controls += 1

    expect_failure(
        "unrecognized active statement",
        lambda: parse_source("opaque directive.", set(), set(), {}),
    )
    controls += 1
    return controls


def load_contract(path: pathlib.Path) -> dict[str, object]:
    def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise AuditError(f"duplicate JSON object key: {key}")
            result[key] = value
        return result

    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_keys,
        )
    except (OSError, json.JSONDecodeError) as exc:
        raise AuditError(f"cannot read contract {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise AuditError("contract root must be an object")
    return value


def fingerprint_output(inventory: Inventory) -> str:
    return json.dumps(
        {
            "facts_sha256": inventory.facts_sha256,
            "rules_sha256": inventory.rules_sha256,
            "route_fingerprints": inventory.route_fingerprints,
            "statement_fingerprints": inventory.statement_fingerprints,
        },
        indent=2,
        sort_keys=True,
    )


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kb", type=pathlib.Path, default=DEFAULT_KB)
    parser.add_argument("--contract", type=pathlib.Path, default=DEFAULT_CONTRACT)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--fingerprints", action="store_true")
    args = parser.parse_args(argv)

    kb = resolve(args.kb)
    contract_path = resolve(args.contract)
    output = resolve(args.output)
    contract = load_contract(contract_path)
    inventory = make_inventory(kb, contract)
    if args.fingerprints:
        print(fingerprint_output(inventory))
        return 0

    validate_contract(contract, inventory)
    generated = render(contract, inventory)
    if args.check:
        controls = negative_controls(contract, inventory, kb.read_text(encoding="utf-8"))
        try:
            current = output.read_text(encoding="utf-8")
        except OSError as exc:
            raise AuditError(f"cannot read generated audit {output}: {exc}") from exc
        if current != generated:
            raise AuditError(
                f"{output.relative_to(ROOT)} is STALE — rerun without --check"
            )
        print(
            f"{output.relative_to(ROOT)} is current; "
            f"{controls} negative controls pass"
        )
        return 0

    output.write_text(generated, encoding="utf-8")
    print(f"{output.relative_to(ROOT)}: regenerated")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AuditError as exc:
        print(f"7-assertion-surface: {exc}", file=sys.stderr)
        raise SystemExit(1)
