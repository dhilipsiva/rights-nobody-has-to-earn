#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate, execute, and render the bounded amendment-semantics audit.

The reviewed JSON source owns exact candidate-source mutations and expected
engine verdicts.  This program keeps Article 9's label verdict separate from
the independently authored source effect, checks every byte-level mutation,
and executes each candidate in a fresh release-engine process when requested.

Usage:
    python3 new-book-plans/10-amendment-semantics.py
    python3 new-book-plans/10-amendment-semantics.py --check
    python3 new-book-plans/10-amendment-semantics.py --check --execute

``--check`` is the fast structural/report-freshness path.  The authoritative
suite adds ``--execute``; regeneration itself remains deterministic and does
not need the engine.
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
import stat
import subprocess
import sys
import tempfile
from unittest import mock
from collections.abc import Iterable, Mapping, Sequence
from typing import Callable


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("new-book-plans/amendment-semantics-audit.json")
DEFAULT_KB = pathlib.Path("new-book-plans/constitution.nibli")
DEFAULT_LEDGER = pathlib.Path("new-book-plans/assertion-surface-contracts.json")
DEFAULT_ASSURANCE = pathlib.Path("new-book-plans/record-integrity-assurance-case.json")
DEFAULT_OUTPUT = pathlib.Path("new-book-plans/amendment-semantics-audit.md")
SURFACE_SCRIPT = pathlib.Path("new-book-plans/7-assertion-surface.py")

ROOT_KEYS = {
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "subprocess_timeout_seconds",
    "constitution_sha256",
    "assertion_surface_contracts_sha256",
    "record_integrity_assurance_case_sha256",
    "label_verdict_meanings",
    "limits",
    "required_cases",
    "cases",
    "narrowness_impacts",
    "acceptance_result",
}
LABEL_MEANING_KEYS = {"blocked_label", "passing_label", "no_amendment_label"}
LIMIT_KEYS = {
    "manual_delta",
    "semantic_completeness",
    "source_history",
    "engine_scope",
    "no_new_gate",
}
CASE_KEYS = {
    "id",
    "title",
    "declared_label",
    "source_effect",
    "mutations",
    "mutation_sha256",
    "expected_source_sha256",
    "source_assertions",
    "steps",
    "assertion_surface_expectation",
}
LABEL_KEYS = {"amendment", "declared_target", "verdict", "summary"}
EFFECT_KEYS = {
    "kind",
    "summary",
    "label_matches_effect",
    "protected_consequence",
}
MUTATION_KEYS = {"op", "before", "after", "before_sha256", "after_sha256"}
ASSERTION_KEYS = {"kind", "relation", "subject", "expected"}
QUERY_KEYS = {"type", "expression", "expected", "purpose"}
CONTROL_KEYS = {
    "type",
    "directive",
    "statement",
    "error_pattern",
    "purpose",
}
ACCEPTANCE_KEYS = {"result", "claim", "does_not_establish", "remaining_boundary"}
NARROWNESS_KEYS = {
    "artifact_ref",
    "current_claim",
    "classification",
    "reason",
    "future_trigger",
}

REQUIRED_CASE_IDS = {f"AS-{number:02d}" for number in range(1, 10)}
LABEL_VERDICTS = LABEL_MEANING_KEYS
EFFECT_KINDS = {
    "unchanged_control",
    "direct_deletion",
    "direct_replacement",
    "ground_label_addition",
    "vocabulary_widening",
}
MATCH_POSTURES = {"true", "false", "not_applicable"}
MUTATION_OPS = {"delete_exact", "replace_exact", "append_exact"}
STEP_TYPES = {"query", "accept", "refuse"}
EXPECTED = {"TRUE", "FALSE"}
SURFACE_EXPECTATIONS = {"not_run", "reject_unreconciled_inventory_relation"}
NARROWNESS_CLASSIFICATIONS = {"preserved_but_scoped", "revised_and_scoped"}
REQUIRED_NARROWNESS_REFERENCES = {
    "book-1/01-what-counts-as-evidence.md::That is a check on declared names, not protection of their meaning.",
    "book-1/08-what-you-are-owed.md::At the floor layer, three hostile routes remain syntactically reachable.",
    "book-1/12-changing-the-rules.md::The recorded label verdict is real.",
    "book-1/15-the-five-joints.md::The consequential surfaces that remain are the constitutional source",
    "book-1/15-the-five-joints.md::That refusal does not deliver the floor",
    "book-1/15-the-five-joints.md::That review is not constitutional approval",
    "book-1/14-when-the-system-notices-it-broke.md::The amendment machinery decides which",
    "book-1/method.md::the candidate edits are applied manually by the audit, not",
}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
CASE_ID = re.compile(r"^AS-[0-9]{2}$")
RELATION = re.compile(r"^[a-z][a-z0-9_]*$")
CONSTANT = re.compile(r"^[A-Z][A-Za-z0-9_]*$")
PLACEHOLDER = re.compile(r"^(?:tbd|todo|unknown|n/?a|pending|placeholder)$", re.I)

# These duplicate a deliberately small headline vector so a reversed JSON
# verdict cannot merely regenerate a self-consistent false report.
SEMANTIC_SENTINELS = {
    ("AS-01", "false(Amend_Floor)"): "TRUE",
    ("AS-01", "become(Amend_Floor, Law)"): "FALSE",
    ("AS-02", "false(Amend_Mint)"): "FALSE",
    ("AS-02", "become(Amend_Mint, Law)"): "TRUE",
    ("AS-03", "become(Amend_Targetless, Law)"): "TRUE",
    ("AS-03", "adjust(Amend_Targetless, $t)"): "FALSE",
    ("AS-03", "entitled(Adam, event { eats() })"): "FALSE",
    ("AS-04", "adjust(Amend_Misdirect, Art_Mint)"): "TRUE",
    ("AS-04", "adjust(Amend_Misdirect, Art_Floor)"): "FALSE",
    ("AS-04", "become(Amend_Misdirect, Law)"): "TRUE",
    ("AS-05", "entitled(Adam, event { eats() })"): "FALSE",
    ("AS-05", "eats(Adam)"): "FALSE",
    ("AS-06", "false(Amend_Mint)"): "TRUE",
    ("AS-06", "become(Amend_Mint, Law)"): "FALSE",
    ("AS-08", "permanent(Art_Evidence)"): "TRUE",
    ("AS-08", "rich(Adam)"): "TRUE",
    ("AS-09", "false(Amend_Floor)"): "TRUE",
    ("AS-09", "become(Amend_Floor, Law)"): "FALSE",
    ("AS-09", "entitled(Adam, event { eats() })"): "FALSE",
}

FLOOR_LINE = "entitled(every person, event { eats() }).\n"
THREADED_FLOOR_LINE = "entitled(every person, event { eats($x) }).\n"
TARGETLESS_APPEND = (
    "\n# Amendment-semantics candidate additions (generated, not enacted).\n"
    "suggest(Assembly, Amend_Targetless).\n"
    "ratifies(Electorate, Amend_Targetless).\n"
)
MISDIRECT_APPEND = (
    "\n# Amendment-semantics candidate additions (generated, not enacted).\n"
    "suggest(Assembly, Amend_Misdirect).\n"
    "adjust(Amend_Misdirect, Art_Mint).\n"
    "ratifies(Electorate, Amend_Misdirect).\n"
)
SYNTAX_APPEND = (
    "\n# Amendment-semantics candidate additions (generated, not enacted).\n"
    "suggest(Assembly, Amend_Syntax).\n"
    "adjust(Amend_Syntax, Art_Syntax).\n"
    "ratifies(Electorate, Amend_Syntax).\n"
)
ART_FLOOR_LABEL_APPEND = (
    "\n# Amendment-semantics candidate addition (generated, not enacted).\n"
    "adjust(Amend_Mint, Art_Floor).\n"
)
ADMITS_ANCHOR = (
    'admits("deceive").    admits("family").     admits("forgive").\n'
)
ADMITS_RICH = ADMITS_ANCHOR + 'admits("rich").\n'
RICH_FACT_APPEND = (
    "\n# Amendment-semantics candidate facts (generated, not enacted).\n"
    "permanent(Art_Evidence).\n"
    "rich(Adam).\n"
)
REQUIRED_MUTATION_SHAPES: dict[str, list[tuple[str, str, str]]] = {
    "AS-01": [],
    "AS-02": [],
    "AS-03": [
        ("delete_exact", FLOOR_LINE, ""),
        ("append_exact", "", TARGETLESS_APPEND),
    ],
    "AS-04": [
        ("delete_exact", FLOOR_LINE, ""),
        ("append_exact", "", MISDIRECT_APPEND),
    ],
    "AS-05": [
        ("replace_exact", FLOOR_LINE, THREADED_FLOOR_LINE),
        ("append_exact", "", SYNTAX_APPEND),
    ],
    "AS-06": [("append_exact", "", ART_FLOOR_LABEL_APPEND)],
    "AS-07": [],
    "AS-08": [
        ("replace_exact", ADMITS_ANCHOR, ADMITS_RICH),
        ("append_exact", "", RICH_FACT_APPEND),
    ],
    "AS-09": [("delete_exact", FLOOR_LINE, "")],
}
REQUIRED_EFFECT_KINDS = {
    "AS-01": "unchanged_control",
    "AS-02": "unchanged_control",
    "AS-03": "direct_deletion",
    "AS-04": "direct_deletion",
    "AS-05": "direct_replacement",
    "AS-06": "ground_label_addition",
    "AS-07": "unchanged_control",
    "AS-08": "vocabulary_widening",
    "AS-09": "direct_deletion",
}
REQUIRED_LABEL_MANIFEST = {
    "AS-01": ("Amend_Floor", "Art_Floor", "blocked_label", "not_applicable"),
    "AS-02": ("Amend_Mint", "Art_Mint", "passing_label", "not_applicable"),
    "AS-03": ("Amend_Targetless", "none", "passing_label", "false"),
    "AS-04": ("Amend_Misdirect", "Art_Mint", "passing_label", "false"),
    "AS-05": ("Amend_Syntax", "Art_Syntax", "passing_label", "false"),
    "AS-06": (
        "Amend_Mint",
        "Art_Mint and Art_Floor",
        "blocked_label",
        "not_applicable",
    ),
    "AS-07": ("none", "none", "no_amendment_label", "not_applicable"),
    "AS-08": ("none", "none", "no_amendment_label", "not_applicable"),
    "AS-09": ("Amend_Floor", "Art_Floor", "blocked_label", "true"),
}
REVIEWED_TIMEOUT_SECONDS = 60
SABOTAGE_FINAL_SUMMARY = "nibli-pin: 1 FINDING(S) (exit 1)"
SURFACE_SEAM_FRAGMENT = (
    "uses relations absent from engine inventory or alias contract: rich"
)


class AmendmentAuditError(RuntimeError):
    """Invalid source, stale report, or failed executable control."""


def resolve(path: pathlib.Path) -> pathlib.Path:
    return path if path.is_absolute() else ROOT / path


def repo_relative(path: pathlib.Path) -> pathlib.Path:
    try:
        return path.resolve().relative_to(ROOT.resolve())
    except (OSError, ValueError) as exc:
        raise AmendmentAuditError(f"path escapes repository: {path}") from exc


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


FileIdentity = tuple[int, int]


def read_bound_file(path: pathlib.Path, label: str) -> tuple[bytes, FileIdentity]:
    """Read one regular-file handle once and bind bytes to its device/inode."""
    try:
        with path.open("rb") as stream:
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode):
                raise AmendmentAuditError(f"{label} must be a regular file: {path}")
            value = stream.read()
            after = os.fstat(stream.fileno())
    except OSError as exc:
        raise AmendmentAuditError(f"cannot read {label} {path}: {exc}") from exc
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
        raise AmendmentAuditError(f"{label} changed while its bound bytes were read")
    return value, (after.st_dev, after.st_ino)


def decode_utf8_exact(value: bytes, label: str) -> str:
    try:
        text = value.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise AmendmentAuditError(f"{label}: invalid UTF-8: {exc}") from exc
    if text.encode("utf-8") != value:
        raise AmendmentAuditError(f"{label}: UTF-8 did not round-trip byte-exactly")
    return text


def decode_constitution_bytes(value: bytes, label: str) -> str:
    if b"\r" in value:
        raise AmendmentAuditError(
            f"{label}: carriage-return bytes are forbidden; exact source requires LF"
        )
    return decode_utf8_exact(value, label)


def require_distinct_identities(
    named_identities: Sequence[tuple[str, FileIdentity]],
) -> set[FileIdentity]:
    seen: dict[FileIdentity, str] = {}
    for label, identity in named_identities:
        if identity in seen:
            raise AmendmentAuditError(
                f"resolved input identity collision: {label} aliases {seen[identity]}"
            )
        seen[identity] = label
    return set(seen)


def validate_output_target(
    path: pathlib.Path, input_identities: set[FileIdentity]
) -> None:
    if path.is_symlink():
        raise AmendmentAuditError("generated output may not be a symlink")
    if not path.exists():
        return
    try:
        details = path.stat()
    except OSError as exc:
        raise AmendmentAuditError(f"cannot inspect generated output {path}: {exc}") from exc
    validate_output_details(details, input_identities)


def validate_output_details(
    details: os.stat_result, input_identities: set[FileIdentity]
) -> None:
    if not stat.S_ISREG(details.st_mode):
        raise AmendmentAuditError("generated output must be a regular file")
    if details.st_nlink != 1:
        raise AmendmentAuditError(
            "generated output must have exactly one hard link"
        )
    if (details.st_dev, details.st_ino) in input_identities:
        raise AmendmentAuditError("generated output identity collides with an input")


def write_generated_output(
    path: pathlib.Path, value: bytes, input_identities: set[FileIdentity]
) -> None:
    flags = os.O_WRONLY | os.O_CREAT
    if hasattr(os, "O_BINARY"):
        flags |= os.O_BINARY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags, 0o666)
    except OSError as exc:
        raise AmendmentAuditError(f"cannot open generated output {path}: {exc}") from exc
    try:
        details = os.fstat(descriptor)
        if not stat.S_ISREG(details.st_mode):
            raise AmendmentAuditError("generated output must be a regular file")
        if details.st_nlink != 1:
            raise AmendmentAuditError(
                "generated output must have exactly one hard link"
            )
        if (details.st_dev, details.st_ino) in input_identities:
            raise AmendmentAuditError("generated output identity collides with an input")
        with os.fdopen(descriptor, "wb", closefd=True) as stream:
            descriptor = -1
            stream.seek(0)
            stream.truncate(0)
            stream.write(value)
            stream.flush()
    except OSError as exc:
        raise AmendmentAuditError(f"cannot write generated output {path}: {exc}") from exc
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def sha256_json(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return sha256_bytes(encoded)


def exact_keys(value: Mapping[str, object], expected: set[str], path: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append("missing " + ", ".join(missing))
        if extra:
            details.append("unknown " + ", ".join(extra))
        raise AmendmentAuditError(f"{path}: {'; '.join(details)}")


def as_object(value: object, path: str) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise AmendmentAuditError(f"{path}: expected an object with string keys")
    return value


def as_list(value: object, path: str) -> list[object]:
    if not isinstance(value, list):
        raise AmendmentAuditError(f"{path}: expected an array")
    return value


def as_text(value: object, path: str, *, allow_none_word: bool = False) -> str:
    if not isinstance(value, str):
        raise AmendmentAuditError(f"{path}: expected a string")
    text = value.strip()
    if not text or (PLACEHOLDER.fullmatch(text) and not allow_none_word):
        raise AmendmentAuditError(f"{path}: requires reviewed, non-placeholder text")
    return value


def text_list(value: object, path: str, *, nonempty: bool = True) -> list[str]:
    raw = as_list(value, path)
    if nonempty and not raw:
        raise AmendmentAuditError(f"{path}: must not be empty")
    result = [as_text(item, f"{path}[{index}]") for index, item in enumerate(raw)]
    if len(result) != len(set(result)):
        raise AmendmentAuditError(f"{path}: duplicate values are not allowed")
    return result


def validate_sha(value: object, path: str, expected: str | None = None) -> str:
    digest = as_text(value, path)
    if not SHA256.fullmatch(digest):
        raise AmendmentAuditError(f"{path}: expected lowercase SHA-256")
    if expected is not None and digest != expected:
        raise AmendmentAuditError(f"{path}: stale; declared {digest}, actual {expected}")
    return digest


def validate_reference(value: object, path: str) -> str:
    """Require a repository-local ``path::unique literal needle`` reference."""
    reference = as_text(value, path)
    if reference.count("::") != 1:
        raise AmendmentAuditError(
            f"{path}: reference must use repo-local path::unique literal needle"
        )
    raw_file, needle = reference.split("::", 1)
    if not raw_file or not needle or "\\" in raw_file:
        raise AmendmentAuditError(f"{path}: invalid path or empty reference needle")
    candidate = pathlib.Path(raw_file)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise AmendmentAuditError(f"{path}: reference must stay inside repository")
    target = ROOT / candidate
    repo_relative(target)
    if not target.is_file():
        raise AmendmentAuditError(f"{path}: referenced file does not exist: {raw_file}")
    try:
        body = target.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise AmendmentAuditError(f"{path}: cannot read {raw_file}: {exc}") from exc
    count = body.count(needle)
    if count != 1:
        raise AmendmentAuditError(
            f"{path}: needle must occur exactly once in {raw_file}; found {count}"
        )
    return reference


def balanced_single_atom(expression: object, path: str) -> str:
    atom = as_text(expression, path)
    if "\n" in atom or "\r" in atom or re.fullmatch(
        r"[A-Za-z0-9_$(),{} \t]+", atom
    ) is None:
        raise AmendmentAuditError(f"{path}: expected one injection-free query atom")
    without_variables = re.sub(r"\$[a-z][a-z0-9_]*", "", atom)
    if "$" in without_variables:
        raise AmendmentAuditError(f"{path}: invalid query variable")
    if re.match(r"^[a-z][a-z0-9_]*\(", atom) is None:
        raise AmendmentAuditError(f"{path}: query must begin with a relation atom")
    stack: list[str] = []
    pairs = {")": "(", "}": "{"}
    for index, character in enumerate(atom):
        if character in "({":
            stack.append(character)
        elif character in ")}":
            if not stack or stack.pop() != pairs[character]:
                raise AmendmentAuditError(f"{path}: unbalanced query delimiters")
            if not stack and index != len(atom) - 1:
                raise AmendmentAuditError(f"{path}: trailing or multiple query atoms")
    if stack or not atom.endswith(")"):
        raise AmendmentAuditError(f"{path}: incomplete query atom")
    return atom


def single_statement(statement: object, path: str) -> str:
    text = as_text(statement, path)
    if "\n" in text or "\r" in text or "#" in text or "?" in text or ";" in text:
        raise AmendmentAuditError(f"{path}: statement injection rejected")
    if not text.endswith(".") or text.count(".") != 1:
        raise AmendmentAuditError(f"{path}: expected exactly one dot-terminated statement")
    body = text[:-1].strip()
    if body.startswith(":") or not (
        re.match(r"^[a-z][a-z0-9_]*\s*\(", body)
        or re.match(r"^all\s+\$[a-z][a-z0-9_]*\s*:", body)
    ):
        raise AmendmentAuditError(
            f"{path}: statement must start with one relation atom or universal rule"
        )
    if re.fullmatch(r"[A-Za-z0-9_$(),{}:&~> .-]+", text) is None:
        raise AmendmentAuditError(f"{path}: unsupported statement character")
    stack: list[str] = []
    pairs = {")": "(", "}": "{"}
    for character in text[:-1]:
        if character in "({":
            stack.append(character)
        elif character in ")}":
            if not stack or stack.pop() != pairs[character]:
                raise AmendmentAuditError(f"{path}: unbalanced statement delimiters")
    if stack:
        raise AmendmentAuditError(f"{path}: unbalanced statement delimiters")
    return text


def apply_mutations(base: str, mutations: Sequence[object], path: str) -> str:
    current = base
    for index, raw in enumerate(mutations):
        item_path = f"{path}[{index}]"
        mutation = as_object(raw, item_path)
        exact_keys(mutation, MUTATION_KEYS, item_path)
        op = as_text(mutation["op"], f"{item_path}.op")
        if op not in MUTATION_OPS:
            raise AmendmentAuditError(f"{item_path}.op: unknown exact mutation {op!r}")
        before = mutation["before"]
        after = mutation["after"]
        if not isinstance(before, str) or not isinstance(after, str):
            raise AmendmentAuditError(f"{item_path}: before and after must be exact text")
        validate_sha(
            mutation["before_sha256"],
            f"{item_path}.before_sha256",
            sha256_text(before),
        )
        validate_sha(
            mutation["after_sha256"],
            f"{item_path}.after_sha256",
            sha256_text(after),
        )
        if op == "append_exact":
            if before or not after or not after.startswith("\n") or not after.endswith("\n"):
                raise AmendmentAuditError(
                    f"{item_path}: append_exact needs empty before and newline-bounded after"
                )
            if after in current:
                raise AmendmentAuditError(f"{item_path}: appended fragment already exists")
            current += after
            continue
        if not before or not before.endswith("\n"):
            raise AmendmentAuditError(f"{item_path}: exact source fragment must end in newline")
        count = current.count(before)
        if count != 1:
            raise AmendmentAuditError(
                f"{item_path}: before fragment must match exactly once; found {count}"
            )
        if op == "delete_exact":
            if after:
                raise AmendmentAuditError(f"{item_path}: delete_exact requires empty after")
        elif not after or not after.endswith("\n") or after == before:
            raise AmendmentAuditError(
                f"{item_path}: replace_exact needs a distinct newline-terminated after"
            )
        current = current.replace(before, after, 1)
    return current


def active_statements(source: str) -> list[str]:
    """Lex dot-terminated active statements without trusting comments."""
    result: list[str] = []
    buffer: list[str] = []
    in_string = False
    escaped = False
    in_comment = False
    for character in source:
        if in_comment:
            if character == "\n":
                in_comment = False
                if buffer:
                    buffer.append(" ")
            continue
        if not in_string and character == "#":
            in_comment = True
            continue
        if character == "\n":
            if buffer:
                buffer.append(" ")
            escaped = False
            continue
        if character == '"' and not escaped:
            in_string = not in_string
        if character == "." and not in_string:
            normalized = " ".join("".join(buffer).split())
            if normalized:
                result.append(normalized)
            buffer = []
            escaped = False
            continue
        buffer.append(character)
        escaped = character == "\\" and not escaped
        if character != "\\":
            escaped = False
    if in_string or "".join(buffer).strip():
        raise AmendmentAuditError("candidate source contains an unterminated statement")
    return result


def validate_source_assertions(
    assertions: Sequence[object], candidate: str, path: str
) -> None:
    statements = active_statements(candidate)
    for index, raw in enumerate(assertions):
        item_path = f"{path}[{index}]"
        item = as_object(raw, item_path)
        exact_keys(item, ASSERTION_KEYS, item_path)
        if item["kind"] != "no_relation_first_argument_atom":
            raise AmendmentAuditError(f"{item_path}.kind: unsupported source assertion")
        relation = as_text(item["relation"], f"{item_path}.relation")
        subject = as_text(item["subject"], f"{item_path}.subject")
        if not RELATION.fullmatch(relation) or not CONSTANT.fullmatch(subject):
            raise AmendmentAuditError(f"{item_path}: invalid relation or subject")
        if item["expected"] != "absent":
            raise AmendmentAuditError(f"{item_path}.expected: must remain absent")
        pattern = re.compile(
            rf"(?<![A-Za-z0-9_]){re.escape(relation)}\s*\(\s*"
            rf"{re.escape(subject)}\s*,"
        )
        matches = [statement for statement in statements if pattern.search(statement)]
        if matches:
            raise AmendmentAuditError(
                f"{item_path}: targetless subject has target atom(s): {matches}"
            )


def validate_steps(steps: Sequence[object], path: str) -> dict[str, list[str]]:
    if not steps:
        raise AmendmentAuditError(f"{path}: every case needs an executable step")
    queries: dict[str, list[str]] = {}
    for index, raw in enumerate(steps):
        item_path = f"{path}[{index}]"
        step = as_object(raw, item_path)
        step_type = as_text(step.get("type"), f"{item_path}.type")
        if step_type not in STEP_TYPES:
            raise AmendmentAuditError(f"{item_path}.type: unsupported step {step_type!r}")
        if step_type == "query":
            exact_keys(step, QUERY_KEYS, item_path)
            expression = balanced_single_atom(step["expression"], f"{item_path}.expression")
            expected = as_text(step["expected"], f"{item_path}.expected")
            if expected not in EXPECTED:
                raise AmendmentAuditError(f"{item_path}.expected: expected TRUE or FALSE")
            purpose = as_text(step["purpose"], f"{item_path}.purpose")
            if "\n" in purpose or "\r" in purpose:
                raise AmendmentAuditError(f"{item_path}.purpose: pin comment must stay on one line")
            queries.setdefault(expression, []).append(expected)
            continue
        exact_keys(step, CONTROL_KEYS, item_path)
        directive = as_text(step["directive"], f"{item_path}.directive")
        if directive != step_type:
            raise AmendmentAuditError(f"{item_path}.directive: must equal step type")
        single_statement(step["statement"], f"{item_path}.statement")
        pattern = as_text(
            step["error_pattern"], f"{item_path}.error_pattern", allow_none_word=True
        )
        if step_type == "accept" and pattern != "none":
            raise AmendmentAuditError(f"{item_path}.error_pattern: accept must use none")
        if step_type == "refuse" and (
            pattern == "none"
            or "/" in pattern
            or "\n" in pattern
            or "\r" in pattern
            or re.fullmatch(r"[A-Za-z0-9_' `>-]+", pattern) is None
        ):
            raise AmendmentAuditError(
                f"{item_path}.error_pattern: refuse needs a slash-free error fragment"
            )
        purpose = as_text(step["purpose"], f"{item_path}.purpose")
        if "\n" in purpose or "\r" in purpose:
            raise AmendmentAuditError(f"{item_path}.purpose: pin comment must stay on one line")
    return queries


def validate_source(
    source: dict[str, object],
    kb_text: str,
    kb_digest: str,
    ledger: dict[str, object],
    ledger_digest: str,
    assurance_digest: str,
) -> dict[str, str]:
    exact_keys(source, ROOT_KEYS, "root")
    if source["spdx"] != "CC-BY-4.0":
        raise AmendmentAuditError("spdx: reviewed source must be CC-BY-4.0")
    if type(source["schema_version"]) is not int or source["schema_version"] != 1:
        raise AmendmentAuditError("schema_version: only integer version 1 is supported")
    as_text(source["title"], "title")
    if source["status"] != "bounded_source_mutation_audit_not_amendment_assurance":
        raise AmendmentAuditError("status: bounded non-assurance posture must remain explicit")
    if source["evidence_role"] != "exposes_semantic_gap":
        raise AmendmentAuditError("evidence_role: gap evidence may not become assurance")
    timeout_seconds = source["subprocess_timeout_seconds"]
    if type(timeout_seconds) is not int or timeout_seconds != REVIEWED_TIMEOUT_SECONDS:
        raise AmendmentAuditError(
            "subprocess_timeout_seconds: must equal the reviewed 60-second bound"
        )
    for key, actual in (
        ("constitution_sha256", kb_digest),
        ("assertion_surface_contracts_sha256", ledger_digest),
        ("record_integrity_assurance_case_sha256", assurance_digest),
    ):
        validate_sha(source[key], key, actual)

    meanings = as_object(source["label_verdict_meanings"], "label_verdict_meanings")
    exact_keys(meanings, LABEL_MEANING_KEYS, "label_verdict_meanings")
    for key, value in meanings.items():
        as_text(value, f"label_verdict_meanings.{key}")
    limits = as_object(source["limits"], "limits")
    exact_keys(limits, LIMIT_KEYS, "limits")
    for key, value in limits.items():
        as_text(value, f"limits.{key}")

    required = set(text_list(source["required_cases"], "required_cases"))
    if required != REQUIRED_CASE_IDS:
        raise AmendmentAuditError("required_cases: exact AS-01 through AS-09 set required")

    candidate_digests: dict[str, str] = {}
    cases: dict[str, dict[str, object]] = {}
    query_vectors: dict[str, dict[str, list[str]]] = {}
    for index, raw in enumerate(as_list(source["cases"], "cases")):
        path = f"cases[{index}]"
        case = as_object(raw, path)
        exact_keys(case, CASE_KEYS, path)
        case_id = as_text(case["id"], f"{path}.id")
        if not CASE_ID.fullmatch(case_id) or case_id not in REQUIRED_CASE_IDS:
            raise AmendmentAuditError(f"{path}.id: unexpected stable case ID")
        if case_id in cases:
            raise AmendmentAuditError(f"{path}.id: duplicate {case_id}")
        as_text(case["title"], f"{path}.title")

        label = as_object(case["declared_label"], f"{path}.declared_label")
        exact_keys(label, LABEL_KEYS, f"{path}.declared_label")
        for field in ("amendment", "declared_target", "summary"):
            as_text(
                label[field],
                f"{path}.declared_label.{field}",
                allow_none_word=field != "summary",
            )
        if label["verdict"] not in LABEL_VERDICTS:
            raise AmendmentAuditError(f"{path}.declared_label.verdict: unknown verdict")
        if label["verdict"] == "no_amendment_label" and label["amendment"] != "none":
            raise AmendmentAuditError(f"{path}.declared_label: no-label case must name none")
        if label["verdict"] != "no_amendment_label" and label["amendment"] == "none":
            raise AmendmentAuditError(f"{path}.declared_label: amendment name required")

        effect = as_object(case["source_effect"], f"{path}.source_effect")
        exact_keys(effect, EFFECT_KEYS, f"{path}.source_effect")
        if effect["kind"] not in EFFECT_KINDS:
            raise AmendmentAuditError(f"{path}.source_effect.kind: unknown effect kind")
        if effect["label_matches_effect"] not in MATCH_POSTURES:
            raise AmendmentAuditError(f"{path}.source_effect.label_matches_effect: invalid")
        for field in ("summary", "protected_consequence"):
            as_text(effect[field], f"{path}.source_effect.{field}")

        label_manifest = (
            label["amendment"],
            label["declared_target"],
            label["verdict"],
            effect["label_matches_effect"],
        )
        if label_manifest != REQUIRED_LABEL_MANIFEST[case_id]:
            raise AmendmentAuditError(
                f"{path}: reviewed amendment/target/verdict/effect-match manifest drifted"
            )

        mutations = as_list(case["mutations"], f"{path}.mutations")
        actual_shape = [
            (
                as_object(mutation, f"{path}.mutations[{mutation_index}]").get("op"),
                as_object(mutation, f"{path}.mutations[{mutation_index}]").get("before"),
                as_object(mutation, f"{path}.mutations[{mutation_index}]").get("after"),
            )
            for mutation_index, mutation in enumerate(mutations)
        ]
        if actual_shape != REQUIRED_MUTATION_SHAPES[case_id]:
            raise AmendmentAuditError(
                f"{path}.mutations: reviewed exact operation shape drifted"
            )
        if effect["kind"] != REQUIRED_EFFECT_KINDS[case_id]:
            raise AmendmentAuditError(
                f"{path}.source_effect.kind: does not match the reviewed exact mutation shape"
            )
        candidate = apply_mutations(kb_text, mutations, f"{path}.mutations")
        declared_mutation_digest = validate_sha(
            case["mutation_sha256"], f"{path}.mutation_sha256"
        )
        actual_mutation_digest = sha256_json(mutations)
        if declared_mutation_digest != actual_mutation_digest:
            raise AmendmentAuditError(
                f"{path}.mutation_sha256: stale; declared {declared_mutation_digest}, actual {actual_mutation_digest}"
            )
        expected_source = validate_sha(
            case["expected_source_sha256"], f"{path}.expected_source_sha256"
        )
        actual_source = sha256_text(candidate)
        if expected_source != actual_source:
            raise AmendmentAuditError(
                f"{path}.expected_source_sha256: stale; declared {expected_source}, actual {actual_source}"
            )
        if effect["kind"] == "unchanged_control":
            if mutations or candidate != kb_text:
                raise AmendmentAuditError(f"{path}: unchanged control must be byte-identical")
        elif not mutations or candidate == kb_text:
            raise AmendmentAuditError(f"{path}: source-effect case requires a real exact delta")
        validate_source_assertions(
            as_list(case["source_assertions"], f"{path}.source_assertions"),
            candidate,
            f"{path}.source_assertions",
        )
        case_queries = validate_steps(
            as_list(case["steps"], f"{path}.steps"), f"{path}.steps"
        )
        query_vectors[case_id] = case_queries
        if label["verdict"] != "no_amendment_label":
            amendment = str(label["amendment"])
            expected_pair = (
                ("TRUE", "FALSE")
                if label["verdict"] == "blocked_label"
                else ("FALSE", "TRUE")
            )
            actual_false = case_queries.get(f"false({amendment})", [])
            actual_become = case_queries.get(f"become({amendment}, Law)", [])
            if expected_pair[0] not in actual_false or expected_pair[1] not in actual_become:
                raise AmendmentAuditError(
                    f"{path}: declared label verdict is not reconciled to false/become queries"
                )
        surface = as_text(
            case["assertion_surface_expectation"],
            f"{path}.assertion_surface_expectation",
        )
        if surface not in SURFACE_EXPECTATIONS:
            raise AmendmentAuditError(f"{path}.assertion_surface_expectation: invalid")
        expected_surface = (
            "reject_unreconciled_inventory_relation"
            if case_id == "AS-08"
            else "not_run"
        )
        if surface != expected_surface:
            raise AmendmentAuditError(
                f"{path}.assertion_surface_expectation: only AS-08 may run the seam"
            )
        cases[case_id] = case
        candidate_digests[case_id] = actual_source

    if set(cases) != REQUIRED_CASE_IDS:
        raise AmendmentAuditError("cases: exact AS-01 through AS-09 cases required")
    for (case_id, expression), expected in SEMANTIC_SENTINELS.items():
        actual = query_vectors[case_id].get(expression, [])
        if expected not in actual:
            raise AmendmentAuditError(
                f"{case_id}: semantic sentinel {expression} must include {expected}"
            )

    vocabulary_cases = {
        case_id
        for case_id, case in cases.items()
        if case["source_effect"]["kind"] == "vocabulary_widening"
    }
    if vocabulary_cases != {"AS-08"}:
        raise AmendmentAuditError(
            "source_effect.kind: AS-08 alone must be vocabulary_widening"
        )

    targetless = cases["AS-03"]
    assertions = targetless["source_assertions"]
    if len(assertions) != 1 or assertions[0].get("kind") != (
        "no_relation_first_argument_atom"
    ):
        raise AmendmentAuditError("AS-03: structural targetlessness assertion is mandatory")
    for case_id, expected_match in (
        ("AS-03", "false"),
        ("AS-04", "false"),
        ("AS-09", "true"),
    ):
        effect = cases[case_id]["source_effect"]
        if (
            effect["kind"] != "direct_deletion"
            or effect["label_matches_effect"] != expected_match
        ):
            raise AmendmentAuditError(
                f"{case_id}: reviewed direct-deletion match posture must remain explicit"
            )
        steps = cases[case_id]["steps"]
        if not any(
            step.get("type") == "accept"
            and step.get("statement")
            == "all $x: person($x) & ~eats($x) -> prisoner($x)."
            for step in steps
        ):
            raise AmendmentAuditError(
                f"{case_id}: executable adverse-rule acceptance is mandatory"
            )
        prisoner_results = query_vectors[case_id].get("prisoner(Cira)", [])
        if not {"TRUE", "FALSE"} <= set(prisoner_results):
            raise AmendmentAuditError(
                f"{case_id}: adverse rule needs discriminating pre/post prisoner queries"
            )
        if "FALSE" not in query_vectors[case_id].get(
            "entitled(Adam, event { eats() })", []
        ):
            raise AmendmentAuditError(
                f"{case_id}: deleted floor needs an executable entitlement consequence"
            )
    if cases["AS-05"]["source_effect"]["kind"] != "direct_replacement":
        raise AmendmentAuditError("AS-05: concealed exact replacement is mandatory")
    if not any(
        step.get("type") == "refuse" and "'prisoner' -> 'eats'" in step.get("error_pattern", "")
        for step in cases["AS-05"]["steps"]
    ):
        raise AmendmentAuditError("AS-05: structural-wall refusal control is mandatory")
    if not any(
        step.get("type") == "refuse"
        and step.get("statement") == "rich(Adam)."
        and step.get("error_pattern") == "not admitted vocabulary"
        for step in cases["AS-07"]["steps"]
    ):
        raise AmendmentAuditError("AS-07: exact closed-vocabulary refusal is mandatory")
    premises = as_object(ledger.get("premises"), "assertion ledger premises")
    if "rich" in premises:
        raise AmendmentAuditError(
            "AS-08: live ledger already contracts rich; reviewed seam no longer applies"
        )
    candidate_as08 = apply_mutations(
        kb_text, cases["AS-08"]["mutations"], "AS-08 mutations"
    )
    if 'admits("rich")' not in active_statements(candidate_as08):
        raise AmendmentAuditError("AS-08: exact candidate must directly admit rich")
    if "permanent(Art_Evidence)" not in active_statements(candidate_as08):
        raise AmendmentAuditError(
            "AS-08: exact candidate must directly register Art_Evidence"
        )

    as09_sequence = [
        (
            step.get("type"),
            step.get("expression", step.get("statement")),
            step.get("expected", step.get("directive")),
        )
        for step in cases["AS-09"]["steps"]
    ]
    if as09_sequence != [
        ("query", "false(Amend_Floor)", "TRUE"),
        ("query", "become(Amend_Floor, Law)", "FALSE"),
        ("query", "entitled(Adam, event { eats() })", "FALSE"),
        ("query", "prisoner(Cira)", "FALSE"),
        (
            "accept",
            "all $x: person($x) & ~eats($x) -> prisoner($x).",
            "accept",
        ),
        ("query", "prisoner(Cira)", "TRUE"),
    ]:
        raise AmendmentAuditError(
            "AS-09: blocked-label, deleted-floor, and pre/rule/post harm sequence drifted"
        )

    seen_narrowness: set[str] = set()
    for index, raw_entry in enumerate(
        as_list(source["narrowness_impacts"], "narrowness_impacts")
    ):
        path = f"narrowness_impacts[{index}]"
        entry = as_object(raw_entry, path)
        exact_keys(entry, NARROWNESS_KEYS, path)
        reference = validate_reference(entry["artifact_ref"], f"{path}.artifact_ref")
        if reference in seen_narrowness:
            raise AmendmentAuditError(f"{path}.artifact_ref: duplicate reference")
        seen_narrowness.add(reference)
        if entry["classification"] not in NARROWNESS_CLASSIFICATIONS:
            raise AmendmentAuditError(f"{path}.classification: unknown classification")
        for field in ("current_claim", "reason", "future_trigger"):
            as_text(entry[field], f"{path}.{field}")
    missing_narrowness = sorted(REQUIRED_NARROWNESS_REFERENCES - seen_narrowness)
    unexpected_narrowness = sorted(seen_narrowness - REQUIRED_NARROWNESS_REFERENCES)
    if missing_narrowness or unexpected_narrowness:
        details: list[str] = []
        if missing_narrowness:
            details.append("required standing claim omitted: " + ", ".join(missing_narrowness))
        if unexpected_narrowness:
            details.append("unreviewed standing claim added: " + ", ".join(unexpected_narrowness))
        raise AmendmentAuditError(
            "narrowness_impacts: " + "; ".join(details)
        )

    acceptance = as_object(source["acceptance_result"], "acceptance_result")
    exact_keys(acceptance, ACCEPTANCE_KEYS, "acceptance_result")
    if acceptance["result"] != "semantic_gap_reproduced":
        raise AmendmentAuditError("acceptance_result.result: may not claim assurance")
    as_text(acceptance["claim"], "acceptance_result.claim")
    residuals = text_list(
        acceptance["does_not_establish"], "acceptance_result.does_not_establish"
    )
    residual_text = " ".join(residuals).lower()
    for term in ("become", "semantic completeness", "source author", "withholding gate"):
        if term not in residual_text:
            raise AmendmentAuditError(
                f"acceptance_result.does_not_establish: missing {term!r} boundary"
            )
    as_text(acceptance["remaining_boundary"], "acceptance_result.remaining_boundary")
    return candidate_digests


def pin_lines(
    case: Mapping[str, object],
    steps: Sequence[object],
    scope: str,
) -> list[str]:
    lines = [
        f":expect-pins {len(steps)}",
        f"# Generated {scope} amendment-semantics pins for {case['id']}.",
        "# Candidate source deltas are authored by the audit, never enacted by become.",
        "",
    ]
    for raw in steps:
        step = as_object(raw, "step")
        lines.append(f"# {step['purpose']}")
        if step["type"] == "query":
            lines.extend([f"? {step['expression']}.", f"# => {step['expected']}", ""])
        elif step["type"] == "accept":
            lines.extend([":accept", str(step["statement"]), ""])
        else:
            lines.extend(
                [
                    f":refuse reasoning /{step['error_pattern']}/",
                    str(step["statement"]),
                    "",
                ]
            )
    return lines


def parse_pass_count(output: str, label: str) -> int:
    # nibli-pin reports the literal words "findings" and "harness errors" in
    # its clean zero-count file summary.  Remove only that exact clean shape;
    # any nonzero, malformed, or free-standing failure marker remains fatal.
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
        raise AmendmentAuditError(
            f"{label}: failure marker appeared despite process success: "
            f"{forbidden.group(0)}"
        )
    matches = re.findall(
        r"(?m)^nibli-pin:\s+PASS\s+[—-]\s+([0-9]+)\s+pins?\s*$",
        output,
    )
    if len(matches) != 1:
        tail = "\n".join(output.splitlines()[-12:])
        raise AmendmentAuditError(
            f"{label}: expected exactly one anchored PASS summary; found "
            f"{len(matches)}\n{tail}"
        )
    return int(matches[0])


def validate_sabotage_failure(returncode: int, output: str) -> None:
    if returncode != 1:
        raise AmendmentAuditError(
            f"executable inverted-verdict sabotage exited {returncode}, expected 1"
        )
    marker_surface = re.sub(
        r"(?i)\b0\s+harness errors?\b",
        "",
        output,
    )
    forbidden = re.search(
        r"(?i)\b(?:HARNESS|NO LONGER|TRACEBACK|PANIC)\b",
        marker_surface,
    )
    if forbidden is not None:
        raise AmendmentAuditError(
            "executable inverted-verdict sabotage emitted forbidden marker: "
            + forbidden.group(0)
        )
    lines = output.rstrip("\r\n").splitlines()
    matches = [line for line in lines if line == SABOTAGE_FINAL_SUMMARY]
    if len(matches) != 1 or not lines or lines[-1] != SABOTAGE_FINAL_SUMMARY:
        raise AmendmentAuditError(
            "executable inverted-verdict sabotage requires exactly one final "
            + SABOTAGE_FINAL_SUMMARY
        )


def validate_surface_seam_failure(
    returncode: int, output: str, expected_fragment: str
) -> None:
    if returncode != 1:
        raise AmendmentAuditError(
            f"AS-08 assertion-surface seam exited {returncode}, expected 1"
        )
    forbidden = re.search(r"(?i)\b(?:TRACEBACK|PANIC)\b", output)
    if forbidden is not None:
        raise AmendmentAuditError(
            "AS-08 assertion-surface seam emitted forbidden marker: "
            + forbidden.group(0)
        )
    pattern = re.compile(
        r"^7-assertion-surface:\s+[^\r\n]*"
        + re.escape(expected_fragment)
        + r"[^\r\n]*$"
    )
    lines = output.rstrip("\r\n").splitlines()
    matches = [line for line in lines if pattern.fullmatch(line)]
    if len(matches) != 1 or not lines or not pattern.fullmatch(lines[-1]):
        raise AmendmentAuditError(
            "AS-08 assertion-surface seam requires exactly one final anchored "
            "7-assertion-surface error containing the reviewed fragment"
        )


def terminate_process_tree(process: subprocess.Popen[str]) -> None:
    """Kill the isolated process group/tree created by ``run_process``."""
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
                raise AmendmentAuditError(
                    "timed-out subprocess tree could not be terminated"
                ) from fallback_exc
        if process.poll() is None:
            raise AmendmentAuditError(
                "timed-out subprocess tree could not be terminated"
            ) from exc


def run_process(
    command: Sequence[str],
    *,
    label: str,
    timeout_seconds: int,
    environment: Mapping[str, str] | None = None,
    tree_terminator: Callable[[subprocess.Popen[str]], None] | None = None,
) -> subprocess.CompletedProcess[str]:
    popen_options: dict[str, object] = {}
    if os.name == "posix":
        popen_options["start_new_session"] = True
    elif os.name == "nt":
        popen_options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    try:
        process = subprocess.Popen(
            list(command),
            cwd=ROOT,
            env=None if environment is None else dict(environment),
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            **popen_options,
        )
    except OSError as exc:
        raise AmendmentAuditError(f"{label}: could not start subprocess: {exc}") from exc
    try:
        output, _ = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired as exc:
        terminator = tree_terminator or terminate_process_tree
        terminator(process)
        # Reap the direct child and close its captured pipe after the whole
        # isolated tree has been terminated.
        process.communicate()
        raise AmendmentAuditError(
            f"{label}: subprocess timed out after {timeout_seconds} seconds"
        ) from exc
    return subprocess.CompletedProcess(
        list(command), process.returncode, output
    )


def execute_cases(
    source: Mapping[str, object],
    kb_bytes: bytes,
    kb_text: str,
    pin_binary: pathlib.Path,
    ledger_path: pathlib.Path,
    timeout_seconds: int,
) -> tuple[int, int, int, int]:
    if not pin_binary.is_absolute():
        raise AmendmentAuditError("selected nibli-pin path must be absolute")
    if not pin_binary.is_file() or not os.access(pin_binary, os.X_OK):
        raise AmendmentAuditError(f"release nibli-pin is missing or not executable: {pin_binary}")
    cases = {
        str(case["id"]): as_object(case, "case")
        for case in as_list(source["cases"], "cases")
    }
    try:
        requested_jobs = int(os.environ.get("AMENDMENT_AUDIT_JOBS", "4"))
    except ValueError as exc:
        raise AmendmentAuditError("AMENDMENT_AUDIT_JOBS must be a positive integer") from exc
    if requested_jobs < 1:
        raise AmendmentAuditError("AMENDMENT_AUDIT_JOBS must be a positive integer")

    with tempfile.TemporaryDirectory(
        prefix=".amendment-semantics-", dir=ROOT / "new-book-plans"
    ) as raw_temp:
        temp = pathlib.Path(raw_temp)

        def run_case(case_id: str) -> tuple[str, int]:
            case = cases[case_id]
            case_dir = temp / case_id.lower()
            case_dir.mkdir()
            candidate = apply_mutations(kb_text, case["mutations"], f"{case_id}.mutations")
            steps = as_list(case["steps"], f"{case_id}.steps")
            scope = "full-source"
            kb_path = case_dir / f"{scope}.nibli"
            pin_path = case_dir / f"{scope}.pins.nibli"
            kb_path.write_bytes(candidate.encode("utf-8"))
            pin_path.write_bytes(
                "\n".join(pin_lines(case, steps, scope)).encode("utf-8")
            )
            completed = run_process(
                [str(pin_binary), "--kb", str(kb_path), str(pin_path)],
                label=f"{case_id} {scope} engine case",
                timeout_seconds=timeout_seconds,
            )
            if completed.returncode != 0:
                tail = "\n".join(completed.stdout.splitlines()[-16:])
                raise AmendmentAuditError(
                    f"{case_id}: {scope} nibli-pin exited "
                    f"{completed.returncode}\n{tail}"
                )
            actual = parse_pass_count(completed.stdout, f"{case_id}/{scope}")
            if actual != len(steps):
                raise AmendmentAuditError(
                    f"{case_id}: {scope} engine ran {actual} pins, expected {len(steps)}"
                )
            return case_id, actual

        pins = 0
        with concurrent.futures.ThreadPoolExecutor(
            max_workers=min(requested_jobs, len(cases))
        ) as pool:
            futures = [pool.submit(run_case, case_id) for case_id in sorted(cases)]
            for future in concurrent.futures.as_completed(futures):
                _, count = future.result()
                pins += count

        # Executable sabotage: reverse the ordinary passing-label verdict.  A
        # green harness that accepts this inversion is not measuring semantics.
        sabotage_dir = temp / "sabotage"
        sabotage_dir.mkdir()
        ordinary = cases["AS-02"]
        sabotage_kb = sabotage_dir / "candidate.nibli"
        sabotage_pins = sabotage_dir / "inverted.pins.nibli"
        sabotage_kb.write_bytes(kb_bytes)
        sabotage_pins.write_bytes(
            (
                ":expect-pins 1\n"
                "# Deliberately inverted: the ordinary label currently passes.\n\n"
                "? become(Amend_Mint, Law).\n"
                "# => FALSE\n"
            ).encode("utf-8")
        )
        sabotage = run_process(
            [str(pin_binary), "--kb", str(sabotage_kb), str(sabotage_pins)],
            label="executable inverted-verdict sabotage",
            timeout_seconds=timeout_seconds,
        )
        validate_sabotage_failure(sabotage.returncode, sabotage.stdout)

        # Article 0a widening must be accepted by the engine but rejected by
        # the unchanged reviewed assertion ledger.  Run the real generator in
        # another fresh process; matching an error string in this process would
        # only restate its contract.
        seam = cases["AS-08"]
        seam_dir = temp / "surface-seam"
        seam_dir.mkdir()
        seam_kb = seam_dir / "candidate.nibli"
        seam_output = seam_dir / "candidate-surface.md"
        seam_kb.write_bytes(
            apply_mutations(
                kb_text, seam["mutations"], "AS-08.mutations"
            ).encode("utf-8")
        )
        environment = dict(os.environ)
        environment["NIBLI_PIN"] = str(pin_binary)
        # verify.sh shares baseline strata with generators.  This seam must
        # measure the mutated candidate instead of silently reusing that cache.
        environment.pop("NIBLI_STRATA_FILE", None)
        surface = run_process(
            [
                sys.executable,
                str(ROOT / SURFACE_SCRIPT),
                "--kb",
                str(seam_kb),
                "--contract",
                str(ledger_path),
                "--output",
                str(seam_output),
            ],
            label="AS-08 assertion-surface seam",
            timeout_seconds=timeout_seconds,
            environment=environment,
        )
        validate_surface_seam_failure(
            surface.returncode, surface.stdout, SURFACE_SEAM_FRAGMENT
        )
    return len(cases), pins, 1, 1


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
    ledger_path: pathlib.Path,
    assurance_path: pathlib.Path,
) -> str:
    lines = [
        f"<!-- SPDX-License-Identifier: {source['spdx']} -->",
        "<!-- Generated by new-book-plans/10-amendment-semantics.py; do not edit. -->",
        "",
        f"# {source['title']}",
        "",
        "## Verdict and scope",
        "",
        "**SEMANTIC GAP REPRODUCED — bounded source-mutation evidence, not amendment assurance.**",
        "",
        markdown(source["acceptance_result"]["claim"]),
        "",
        "Each candidate diff is authored directly by this audit. A TRUE",
        "`become(_, Law)` result is a label verdict only: the current constitution",
        "has no reader that applies source text. A `false(_)` result likewise does",
        "not prove that an independently supplied source transition was prevented.",
        "",
        "## Label-verdict meanings",
        "",
        "| label verdict | exact meaning |",
        "| --- | --- |",
    ]
    for verdict in ("blocked_label", "passing_label", "no_amendment_label"):
        lines.append(
            f"| {code(verdict)} | "
            f"{markdown(source['label_verdict_meanings'][verdict])} |"
        )
    lines.extend(
        [
        "",
        "## Label verdicts and source effects",
        "",
        "These columns are intentionally separate. The first is derived from",
        "Article 9's self-declared labels; the second is the exact candidate text",
        "the harness independently constructs.",
        "Declared-target match is the reviewed test author's classification; Article 9",
        "does not derive it, and the harness is not a semantic oracle. It says only",
        "whether the declared target accurately describes the independently applied",
        "effect—not that the label verdict controlled a source transition.",
        "",
        "| case | declared label and verdict | exact source effect | declared target matches effect |",
        "| --- | --- | --- | --- |",
        ]
    )
    for case in source["cases"]:
        label = case["declared_label"]
        effect = case["source_effect"]
        label_text = (
            f"{label['amendment']} → {label['declared_target']}; {label['verdict']}"
        )
        lines.append(
            f"| {code(case['id'])} {markdown(case['title'])} | "
            f"{markdown(label_text)} | {code(effect['kind'])}: "
            f"{markdown(effect['summary'])} | {code(effect['label_matches_effect'])} |"
        )

    lines.extend(["", "## Limits", ""])
    for key, value in source["limits"].items():
        lines.append(f"- **{key.replace('_', ' ').title()}:** {markdown(value)}")

    lines.extend(
        [
            "",
            "## Exact mutation manifest",
            "",
            "Every fragment digest is checked before application; deletions and",
            "replacements must match exactly once, and each final candidate digest",
            "is reviewed in the JSON source.",
            "",
            "| case | operation | before SHA-256 | after SHA-256 | candidate SHA-256 |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for case in source["cases"]:
        if not case["mutations"]:
            lines.append(
                f"| {code(case['id'])} | byte-identical control | — | — | "
                f"{code(case['expected_source_sha256'])} |"
            )
            continue
        for index, mutation in enumerate(case["mutations"]):
            candidate = case["expected_source_sha256"] if index == 0 else "↳"
            lines.append(
                f"| {code(case['id'])} | {code(mutation['op'])} | "
                f"{code(mutation['before_sha256'])} | {code(mutation['after_sha256'])} | "
                f"{code(candidate)} |"
            )

    lines.extend(
        [
            "",
            "## Executable cases",
            "",
            "Every ordinary and opaque food-entitlement verdict runs in one process",
            "against the full exact candidate source. This keeps an amendment's",
            "floor effect coupled to all temporal and non-temporal rules that coexist",
            "with it in the same full-candidate process.",
            "",
        ]
    )
    for case in source["cases"]:
        label = case["declared_label"]
        effect = case["source_effect"]
        lines.extend(
            [
                f"### {case['id']} — {case['title']}",
                "",
                f"- **Label verdict:** {code(label['verdict'])} — {markdown(label['summary'])}",
                f"- **Source effect:** {code(effect['kind'])} — {markdown(effect['summary'])}",
                f"- **Protected/adverse consequence:** {markdown(effect['protected_consequence'])}",
                f"- **Mutation contract:** {code(case['mutation_sha256'])}",
                "",
                "| check | expected | purpose |",
                "| --- | --- | --- |",
            ]
        )
        for step in case["steps"]:
            if step["type"] == "query":
                check = code(step["expression"])
                expected = f"**{step['expected']}**"
            elif step["type"] == "accept":
                check = f"accept {code(step['statement'])}"
                expected = "**ACCEPTED**"
            else:
                check = f"refuse {code(step['statement'])}"
                expected = f"**REFUSED** ({code(step['error_pattern'])})"
            lines.append(
                f"| {check} | {expected} | {markdown(step['purpose'])} |"
            )
        for assertion in case["source_assertions"]:
            lines.append(
                f"| structural: no {code(assertion['relation'])} atom whose first argument is "
                f"{code(assertion['subject'])} | **ABSENT** | Targetlessness is "
                "checked across active facts, compounds, and rule statements, not "
                "inferred from a finite query list. |"
            )
        if case["assertion_surface_expectation"] != "not_run":
            lines.append(
                "| live assertion-surface pipeline | **REJECTS** `rich` during "
                "source/inventory reconciliation | The engine accepts the widening, "
                "but `rich` is absent from the audit's engine inventory and alias "
                "contract; premise-card validation is not reached. |"
            )
        lines.append("")

    lines.extend(
        [
            "## Narrowness impacts",
            "",
            "The audit changes no live constitutional rule. These reviewed entries",
            "record every standing claim whose scope depends on the label/effect",
            "boundary or on the current source remaining narrow.",
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
    lines.append("")

    acceptance = source["acceptance_result"]
    lines.extend(
        [
            "## Acceptance result",
            "",
            "**SEMANTIC GAP REPRODUCED.**",
            "",
            markdown(acceptance["claim"]),
            "",
            "This artifact does **not** establish:",
            "",
            *[f"- {markdown(value)}" for value in acceptance["does_not_establish"]],
            "",
            f"Remaining boundary: {markdown(acceptance['remaining_boundary'])}",
            "",
            "## Maintenance",
            "",
            f"- Reviewed source: {code(source_path.as_posix())}.",
            f"- Constitution: {code(kb_path.as_posix())}, SHA-256 {code(source['constitution_sha256'])}.",
            f"- Assertion ledger: {code(ledger_path.as_posix())}, SHA-256 {code(source['assertion_surface_contracts_sha256'])}.",
            f"- Assurance source: {code(assurance_path.as_posix())}, SHA-256 {code(source['record_integrity_assurance_case_sha256'])}.",
            f"- Reviewed subprocess timeout: {code(source['subprocess_timeout_seconds'])} seconds for every isolated case, sabotage, and live seam.",
            "- Bound input bytes are read once, decoded strictly, and hashed without newline translation; constitution CR bytes are refused and candidates are written as exact UTF-8 bytes.",
            "- Existing inputs must have distinct device/inode identities; the generated output must be a single-link regular file distinct from every input.",
            "- Expected failures are narrow contracts: exit 1 plus one final Nibli finding summary for sabotage, and exit 1 plus one anchored assertion-generator error for the live seam.",
            "- Timed-out subprocesses run in an isolated process group and the whole process tree is terminated before the harness fails.",
            "- Regenerate: `python3 new-book-plans/10-amendment-semantics.py`.",
            "- Fast structural/freshness check: `python3 new-book-plans/10-amendment-semantics.py --check`.",
            "- Authoritative execution: `python3 new-book-plans/10-amendment-semantics.py --check --execute`.",
            "- Each candidate and the executable sabotage run in fresh engine processes.",
            "",
        ]
    )
    return "\n".join(lines)


def expect_failure(
    label: str,
    action: Callable[[], object],
    *,
    contains: str | None = None,
) -> None:
    try:
        action()
    except AmendmentAuditError as exc:
        if contains is not None and contains not in str(exc):
            raise AmendmentAuditError(
                f"negative control failed for the wrong reason: {label}: {exc}"
            ) from exc
        return
    raise AmendmentAuditError(f"negative control did not fail: {label}")


def negative_controls(
    source: dict[str, object],
    kb_bytes: bytes,
    kb_text: str,
    kb_digest: str,
    ledger: dict[str, object],
    ledger_digest: str,
    assurance_digest: str,
) -> int:
    controls = 0

    def validate(candidate: dict[str, object], candidate_ledger: dict[str, object] = ledger) -> None:
        validate_source(
            candidate,
            kb_text,
            kb_digest,
            candidate_ledger,
            ledger_digest,
            assurance_digest,
        )

    def case_with_id(container: Mapping[str, object], case_id: str) -> dict[str, object]:
        matches = [
            as_object(raw, f"case {case_id}")
            for raw in as_list(container["cases"], "cases")
            if isinstance(raw, dict) and raw.get("id") == case_id
        ]
        if len(matches) != 1:
            raise AmendmentAuditError(
                f"negative-control lookup: expected one {case_id}; found {len(matches)}"
            )
        return matches[0]

    def mutation_with(
        case: Mapping[str, object],
        *,
        op: str,
        before: str | None = None,
        after: str | None = None,
    ) -> dict[str, object]:
        matches: list[dict[str, object]] = []
        for raw in as_list(case["mutations"], f"{case.get('id')}.mutations"):
            mutation = as_object(raw, f"{case.get('id')}.mutation")
            if mutation.get("op") != op:
                continue
            if before is not None and mutation.get("before") != before:
                continue
            if after is not None and mutation.get("after") != after:
                continue
            matches.append(mutation)
        if len(matches) != 1:
            raise AmendmentAuditError(
                "negative-control lookup: expected one semantic mutation in "
                f"{case.get('id')}; found {len(matches)}"
            )
        return matches[0]

    def query_with(
        case: Mapping[str, object], expression: str, expected: str | None = None
    ) -> dict[str, object]:
        matches = [
            as_object(raw, f"{case.get('id')}.step")
            for raw in as_list(case["steps"], f"{case.get('id')}.steps")
            if isinstance(raw, dict)
            and raw.get("type") == "query"
            and raw.get("expression") == expression
            and (expected is None or raw.get("expected") == expected)
        ]
        if len(matches) != 1:
            raise AmendmentAuditError(
                "negative-control lookup: expected one query "
                f"{expression!r} in {case.get('id')}; found {len(matches)}"
            )
        return matches[0]

    def control_with(
        case: Mapping[str, object], step_type: str, statement: str
    ) -> dict[str, object]:
        matches = [
            as_object(raw, f"{case.get('id')}.step")
            for raw in as_list(case["steps"], f"{case.get('id')}.steps")
            if isinstance(raw, dict)
            and raw.get("type") == step_type
            and raw.get("statement") == statement
        ]
        if len(matches) != 1:
            raise AmendmentAuditError(
                "negative-control lookup: expected one "
                f"{step_type} statement in {case.get('id')}; found {len(matches)}"
            )
        return matches[0]

    def impact_with_ref(
        container: Mapping[str, object], artifact_ref: str
    ) -> dict[str, object]:
        matches = [
            as_object(raw, f"narrowness impact {artifact_ref}")
            for raw in as_list(container["narrowness_impacts"], "narrowness_impacts")
            if isinstance(raw, dict) and raw.get("artifact_ref") == artifact_ref
        ]
        if len(matches) != 1:
            raise AmendmentAuditError(
                "negative-control lookup: expected one narrowness impact "
                f"{artifact_ref!r}; found {len(matches)}"
            )
        return matches[0]

    for key, label in (
        ("constitution_sha256", "constitution digest drift"),
        ("assertion_surface_contracts_sha256", "assertion-ledger digest drift"),
        ("record_integrity_assurance_case_sha256", "assurance-source digest drift"),
    ):
        changed = copy.deepcopy(source)
        changed[key] = "0" * 64
        expect_failure(label, lambda changed=changed: validate(changed))
        controls += 1

    crlf_bytes = kb_bytes.replace(b"\n", b"\r\n")
    if crlf_bytes == kb_bytes:
        raise AmendmentAuditError("CRLF negative control requires at least one LF byte")
    expect_failure(
        "CRLF constitution rejected before newline normalization",
        lambda: decode_constitution_bytes(crlf_bytes, "CRLF control"),
        contains="carriage-return bytes are forbidden",
    )
    controls += 1

    with tempfile.TemporaryDirectory(
        prefix=".amendment-hardlink-control-", dir=ROOT / "new-book-plans"
    ) as raw_hardlink_temp:
        hardlink_temp = pathlib.Path(raw_hardlink_temp)
        first = hardlink_temp / "first.json"
        alias = hardlink_temp / "alias.json"
        first.write_bytes(b"{}\n")
        _, first_identity = read_bound_file(first, "hardlink control first")
        synthetic_details: object | None = None
        try:
            os.link(first, alias)
        except OSError as exc:
            if os.name != "nt" or getattr(exc, "winerror", None) != 50:
                raise AmendmentAuditError(
                    f"hardlink negative control could not create its alias: {exc}"
                ) from exc
            # The Windows WSL UNC provider does not expose link(2). Preserve
            # the same semantic controls there; authoritative WSL execution
            # creates and inspects the real temporary hardlink above.
            alias_identity = first_identity
            synthetic_details = mock.Mock(
                st_mode=stat.S_IFREG | 0o600,
                st_nlink=2,
                st_dev=first_identity[0],
                st_ino=first_identity[1],
            )
        else:
            _, alias_identity = read_bound_file(alias, "hardlink control alias")
        expect_failure(
            "hardlinked input identities collide",
            lambda: require_distinct_identities(
                [("first", first_identity), ("alias", alias_identity)]
            ),
        )
        controls += 1
        expect_failure(
            "hardlinked generated output rejected",
            lambda: (
                validate_output_target(alias, set())
                if synthetic_details is None
                else validate_output_details(synthetic_details, set())
            ),
            contains="exactly one hard link",
        )
        controls += 1
        collision_details = mock.Mock(
            st_mode=stat.S_IFREG | 0o600,
            st_nlink=1,
            st_dev=first_identity[0],
            st_ino=first_identity[1],
        )
        expect_failure(
            "generated output identity collides with input",
            lambda: validate_output_details(
                collision_details, {first_identity}
            ),
            contains="identity collides with an input",
        )
        controls += 1

    changed = copy.deepcopy(source)
    changed["schema_version"] = True
    expect_failure("boolean schema version", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["cases"] = [
        case for case in changed["cases"] if case.get("id") != "AS-01"
    ]
    expect_failure("required case deleted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["cases"].append(copy.deepcopy(case_with_id(changed, "AS-01")))
    expect_failure("duplicate case ID", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-01")["unexpected"] = "field"
    expect_failure("unknown case field", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    query_with(case_with_id(changed, "AS-01"), "false(Amend_Floor)")[
        "expected"
    ] = "FALSE"
    expect_failure("reversed semantic sentinel", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    mutation_with(
        case_with_id(changed, "AS-03"), op="delete_exact", before=FLOOR_LINE
    )["before_sha256"] = "0" * 64
    expect_failure("exact fragment digest drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-03")["expected_source_sha256"] = "0" * 64
    expect_failure("candidate source digest drift", lambda: validate(changed))
    controls += 1

    missing_mutation = copy.deepcopy(
        mutation_with(
            case_with_id(source, "AS-03"), op="delete_exact", before=FLOOR_LINE
        )
    )
    missing_mutation["before"] = "missing exact line\n"
    missing_mutation["before_sha256"] = sha256_text("missing exact line\n")
    expect_failure(
        "exact deletion with zero matches",
        lambda: apply_mutations(kb_text, [missing_mutation], "zero-match control"),
    )
    controls += 1

    unknown_mutation = copy.deepcopy(
        mutation_with(
            case_with_id(source, "AS-03"), op="append_exact", after=TARGETLESS_APPEND
        )
    )
    unknown_mutation["op"] = "rewrite_semantically"
    expect_failure(
        "unknown mutation operation",
        lambda: apply_mutations(kb_text, [unknown_mutation], "unknown-op control"),
    )
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-03")["source_assertions"] = []
    expect_failure("targetlessness assertion removed", lambda: validate(changed))
    controls += 1

    targetless_assertions = as_list(
        case_with_id(source, "AS-03")["source_assertions"],
        "AS-03.source_assertions",
    )
    expect_failure(
        "targetless atom hidden in a compound fact",
        lambda: validate_source_assertions(
            targetless_assertions,
            kb_text + "\nperson(Cira) & adjust (Amend_Targetless, Art_Mint).\n",
            "compound-target control",
        ),
    )
    controls += 1

    expect_failure(
        "targetless atom derived in a rule head",
        lambda: validate_source_assertions(
            targetless_assertions,
            kb_text
            + "\nall $x: person($x) -> adjust (Amend_Targetless, Art_Mint).\n",
            "rule-target control",
        ),
    )
    controls += 1

    changed = copy.deepcopy(source)
    del case_with_id(changed, "AS-04")["source_effect"]["label_matches_effect"]
    expect_failure("label and source effect collapsed", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    as09_label = case_with_id(changed, "AS-09")["declared_label"]
    as09_label["amendment"] = "none"
    as09_label["declared_target"] = "none"
    as09_label["verdict"] = "no_amendment_label"
    expect_failure("AS-09 relabelled as no-amendment", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    as08_label = case_with_id(changed, "AS-08")["declared_label"]
    as08_label["amendment"] = "Amend_Evidence"
    as08_label["declared_target"] = "Art_Evidence"
    as08_label["verdict"] = "blocked_label"
    expect_failure("AS-08 relabelled as blocked", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-08")["source_effect"]["label_matches_effect"] = "true"
    expect_failure("AS-08 declared-target match promoted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    concealed = case_with_id(changed, "AS-05")
    concealed["steps"] = [
        step
        for step in concealed["steps"]
        if not (
            step.get("type") == "refuse"
            and step.get("error_pattern") == "'prisoner' -> 'eats'"
        )
    ]
    expect_failure("concealed structural-wall control removed", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-08")["assertion_surface_expectation"] = "not_run"
    expect_failure("assertion-surface seam disabled", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-01")["assertion_surface_expectation"] = (
        "reject_unreconciled_inventory_relation"
    )
    expect_failure("non-AS-08 assertion-surface seam enabled", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    case_with_id(changed, "AS-06")["source_effect"]["kind"] = "vocabulary_widening"
    expect_failure("non-AS-08 vocabulary-widening classification", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    mutation_with(
        case_with_id(changed, "AS-06"),
        op="append_exact",
        after=ART_FLOOR_LABEL_APPEND,
    )["after"] = ART_FLOOR_LABEL_APPEND.replace("Art_Floor", "Art_Mint")
    expect_failure("Art_Floor append operation shape drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    mutation_with(
        case_with_id(changed, "AS-08"),
        op="replace_exact",
        before=ADMITS_ANCHOR,
        after=ADMITS_RICH,
    )["after"] = ADMITS_ANCHOR + 'admits("wealthy").\n'
    expect_failure("exact admits-rich widening drift", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    query_with(case_with_id(changed, "AS-01"), "false(Amend_Floor)")[
        "expression"
    ] = (
        "false(Amend_Floor). ? become(Amend_Floor, Law)"
    )
    expect_failure("query injection", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    query_with(case_with_id(changed, "AS-03"), "adjust(Amend_Targetless, $t)")[
        "expression"
    ] = "adjust(Amend_Targetless, $T)"
    expect_failure("invalid query variable", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    query_with(case_with_id(changed, "AS-01"), "false(Amend_Floor)")[
        "purpose"
    ] += "\n? rich(Adam)."
    expect_failure("pin-comment injection", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    accepted = control_with(
        case_with_id(changed, "AS-03"),
        "accept",
        "all $x: person($x) & ~eats($x) -> prisoner($x).",
    )
    accepted["statement"] += " rich(Adam)."
    expect_failure("statement injection", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    control_with(case_with_id(changed, "AS-07"), "refuse", "rich(Adam).")[
        "statement"
    ] = ":accept."
    expect_failure("directive-shaped statement", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    control_with(case_with_id(changed, "AS-07"), "refuse", "rich(Adam).")[
        "statement"
    ] = "All $x: person($x) -> rich($x)."
    expect_failure("invalid statement start", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    as09 = case_with_id(changed, "AS-09")
    pre_harm = query_with(as09, "prisoner(Cira)", "FALSE")
    adverse_rule = control_with(
        as09,
        "accept",
        "all $x: person($x) & ~eats($x) -> prisoner($x).",
    )
    as09["steps"] = [
        adverse_rule if step is pre_harm else pre_harm if step is adverse_rule else step
        for step in as09["steps"]
    ]
    expect_failure("AS-09 pre/rule/post sequence reversed", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["status"] = "semantic_entrenchment_established"
    expect_failure("assurance promotion", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    del changed["limits"]["no_new_gate"]
    expect_failure("no-new-gate boundary removed", lambda: validate(changed))
    controls += 1

    for invalid_timeout in (0, True, REVIEWED_TIMEOUT_SECONDS + 1):
        changed = copy.deepcopy(source)
        changed["subprocess_timeout_seconds"] = invalid_timeout
        expect_failure(
            f"invalid subprocess timeout {invalid_timeout!r}",
            lambda changed=changed: validate(changed),
        )
        controls += 1

    timeout_process = mock.Mock()
    timeout_process.communicate.side_effect = [
        subprocess.TimeoutExpired(cmd=["nibli-pin"], timeout=1),
        ("", None),
    ]
    timeout_process.returncode = -9
    terminated_trees: list[object] = []
    with mock.patch.object(
        subprocess, "Popen", return_value=timeout_process
    ) as popen_control:
        expect_failure(
            "subprocess timeout is named and fail-closed",
            lambda: run_process(
                ["nibli-pin"],
                label="timeout control",
                timeout_seconds=1,
                tree_terminator=lambda process: terminated_trees.append(process),
            ),
            contains="timeout control: subprocess timed out after 1 seconds",
        )
    if terminated_trees != [timeout_process]:
        raise AmendmentAuditError(
            "timeout control did not invoke whole-tree termination exactly once"
        )
    process_options = popen_control.call_args.kwargs
    if os.name == "posix" and process_options.get("start_new_session") is not True:
        raise AmendmentAuditError(
            "timeout control did not isolate the child in a POSIX process group"
        )
    if os.name == "nt" and process_options.get("creationflags") != (
        subprocess.CREATE_NEW_PROCESS_GROUP
    ):
        raise AmendmentAuditError(
            "timeout control did not isolate the child in a Windows process group"
        )
    controls += 1

    changed = copy.deepcopy(source)
    changed["acceptance_result"]["does_not_establish"] = [
        value
        for value in changed["acceptance_result"]["does_not_establish"]
        if "withholding gate" not in value
    ]
    expect_failure("withholding boundary removed", lambda: validate(changed))
    controls += 1

    changed_ledger = copy.deepcopy(ledger)
    changed_ledger["premises"]["rich"] = {}
    expect_failure(
        "vocabulary seam silently contracted",
        lambda: validate(source, changed_ledger),
    )
    controls += 1

    duplicate_mutation = copy.deepcopy(
        mutation_with(
            case_with_id(source, "AS-03"), op="delete_exact", before=FLOOR_LINE
        )
    )
    expect_failure(
        "exact deletion with duplicate matches",
        lambda: apply_mutations(
            kb_text + FLOOR_LINE,
            [duplicate_mutation],
            "duplicate-match control",
        ),
    )
    controls += 1

    for marker in (
        "FINDING",
        "HARNESS ERROR",
        "NO LONGER REPRODUCE",
        "Traceback",
        "panic",
    ):
        expect_failure(
            f"successful pin output containing {marker!r}",
            lambda marker=marker: parse_pass_count(
                f"nibli-pin: PASS — 1 pin\n{marker}\n", "parser control"
            ),
        )
        controls += 1

    expect_failure(
        "multiple PASS summaries",
        lambda: parse_pass_count(
            "nibli-pin: PASS — 1 pin\nnibli-pin: PASS — 1 pin\n",
            "parser control",
        ),
    )
    controls += 1

    valid_sabotage_output = (
        "FINDINGS (1) — a pinned property regressed:\n"
        + SABOTAGE_FINAL_SUMMARY
        + "\n"
    )
    for wrong_returncode in (0, 2):
        expect_failure(
            f"sabotage marker paired with wrong rc {wrong_returncode}",
            lambda wrong_returncode=wrong_returncode: validate_sabotage_failure(
                wrong_returncode, valid_sabotage_output
            ),
        )
        controls += 1
    expect_failure(
        "duplicate sabotage final summaries",
        lambda: validate_sabotage_failure(
            1, valid_sabotage_output + SABOTAGE_FINAL_SUMMARY + "\n"
        ),
    )
    controls += 1
    expect_failure(
        "sabotage final summary followed by output",
        lambda: validate_sabotage_failure(
            1, valid_sabotage_output + "unexpected trailing line\n"
        ),
    )
    controls += 1
    for forbidden_marker in (
        "HARNESS ERROR",
        "NO LONGER REPRODUCE",
        "Traceback",
        "panic",
    ):
        expect_failure(
            f"sabotage output contains {forbidden_marker!r}",
            lambda forbidden_marker=forbidden_marker: validate_sabotage_failure(
                1, forbidden_marker + "\n" + valid_sabotage_output
            ),
        )
        controls += 1

    valid_seam_output = (
        "7-assertion-surface: source statement uses "
        + SURFACE_SEAM_FRAGMENT
        + "\n"
    )
    for wrong_returncode in (0, 2):
        expect_failure(
            f"assertion seam marker paired with wrong rc {wrong_returncode}",
            lambda wrong_returncode=wrong_returncode: validate_surface_seam_failure(
                wrong_returncode, valid_seam_output, SURFACE_SEAM_FRAGMENT
            ),
        )
        controls += 1
    expect_failure(
        "unanchored assertion seam error",
        lambda: validate_surface_seam_failure(
            1,
            "prefix " + valid_seam_output,
            SURFACE_SEAM_FRAGMENT,
        ),
    )
    controls += 1
    expect_failure(
        "duplicate assertion seam errors",
        lambda: validate_surface_seam_failure(
            1,
            valid_seam_output + valid_seam_output,
            SURFACE_SEAM_FRAGMENT,
        ),
    )
    controls += 1
    for forbidden_marker in ("Traceback", "panic"):
        expect_failure(
            f"assertion seam output contains {forbidden_marker!r}",
            lambda forbidden_marker=forbidden_marker: validate_surface_seam_failure(
                1,
                forbidden_marker + "\n" + valid_seam_output,
                SURFACE_SEAM_FRAGMENT,
            ),
        )
        controls += 1

    chapter_one_ref = (
        "book-1/01-what-counts-as-evidence.md::"
        "That is a check on declared names, not protection of their meaning."
    )
    chapter_eight_ref = (
        "book-1/08-what-you-are-owed.md::"
        "At the floor layer, three hostile routes remain syntactically reachable."
    )
    method_ref = (
        "book-1/method.md::the candidate edits are applied manually by the audit, not"
    )

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"] = [
        entry
        for entry in changed["narrowness_impacts"]
        if entry.get("artifact_ref") != chapter_one_ref
    ]
    expect_failure("required narrowness impact deleted", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"] = [
        entry
        for entry in changed["narrowness_impacts"]
        if entry.get("artifact_ref") != chapter_eight_ref
    ]
    expect_failure(
        "Chapter 8 narrowness impact deleted", lambda: validate(changed)
    )
    controls += 1

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"].append(
        copy.deepcopy(impact_with_ref(changed, chapter_one_ref))
    )
    expect_failure("duplicate narrowness artifact reference", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    impact_with_ref(changed, chapter_one_ref)["classification"] = "unchanged"
    expect_failure("invalid narrowness classification", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    impact_with_ref(changed, chapter_one_ref)["artifact_ref"] = (
        "book-1/01-what-counts-as-evidence.md::"
        "THIS REVIEWED ANCHOR DOES NOT EXIST"
    )
    expect_failure("stale narrowness anchor", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    impact_with_ref(changed, chapter_one_ref)["artifact_ref"] = (
        "book-1/01-what-counts-as-evidence.md::the"
    )
    expect_failure("ambiguous narrowness anchor", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["cases"] = list(reversed(changed["cases"]))
    query_with(case_with_id(changed, "AS-03"), "adjust(Amend_Targetless, $t)")[
        "expected"
    ] = "TRUE"
    expect_failure("reordered cases retain semantic controls", lambda: validate(changed))
    controls += 1

    changed = copy.deepcopy(source)
    changed["narrowness_impacts"] = list(reversed(changed["narrowness_impacts"]))
    impact_with_ref(changed, method_ref)["classification"] = "unknown"
    expect_failure(
        "reordered narrowness entries retain semantic controls",
        lambda: validate(changed),
    )
    controls += 1

    expect_failure(
        "duplicate JSON object key",
        lambda: json.loads(
            '{"status":"bounded","status":"assured"}',
            object_pairs_hook=reject_duplicate_keys,
        ),
    )
    controls += 1
    return controls


def reject_duplicate_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise AmendmentAuditError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_json_bytes(value: bytes, label: str) -> dict[str, object]:
    try:
        value = json.loads(
            decode_utf8_exact(value, label), object_pairs_hook=reject_duplicate_keys
        )
    except json.JSONDecodeError as exc:
        raise AmendmentAuditError(f"cannot parse {label}: {exc}") from exc
    return as_object(value, label)


def select_pin(cli_pin: pathlib.Path | None) -> pathlib.Path:
    """Select the release engine once, in the reviewed precedence order."""
    candidate: pathlib.Path
    if cli_pin is not None:
        candidate = cli_pin
    elif "NIBLI_PIN" in os.environ:
        raw_pin = os.environ["NIBLI_PIN"].strip()
        if not raw_pin:
            raise AmendmentAuditError("NIBLI_PIN is set but empty")
        candidate = pathlib.Path(raw_pin)
    elif "NIBLI_SRC" in os.environ:
        raw_source = os.environ["NIBLI_SRC"].strip()
        if not raw_source:
            raise AmendmentAuditError("NIBLI_SRC is set but empty")
        candidate = pathlib.Path(raw_source) / "target/release/nibli-pin"
    else:
        on_path = shutil.which("nibli-pin")
        if on_path:
            candidate = pathlib.Path(on_path)
        else:
            candidate = (
                pathlib.Path.home()
                / "projects/dhilipsiva/nibli/target/release/nibli-pin"
            )
    return candidate.expanduser().resolve()


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
    parser.add_argument("--wait-for-lock", type=float, default=0.0)
    args = parser.parse_args(argv)

    source_path = resolve(args.source)
    kb_path = resolve(args.kb)
    ledger_path = resolve(args.ledger)
    assurance_path = resolve(args.assurance)
    output_path = resolve(args.output)
    input_specs = (
        ("amendment-semantics source", source_path),
        ("constitution", kb_path),
        ("assertion ledger", ledger_path),
        ("assurance source", assurance_path),
    )
    for _, path in input_specs:
        repo_relative(path)

    default_output = resolve(DEFAULT_OUTPUT)
    requested_output_identity = output_path.resolve(strict=False)
    default_output_identity = default_output.resolve(strict=False)
    if requested_output_identity != default_output_identity:
        raise AmendmentAuditError(
            "--output is fixed to new-book-plans/amendment-semantics-audit.md"
        )
    repo_relative(default_output)
    if output_path.is_symlink() or default_output.is_symlink():
        raise AmendmentAuditError("generated output may not be a symlink")
    output_path = default_output

    bound_inputs: dict[str, bytes] = {}
    named_identities: list[tuple[str, FileIdentity]] = []
    for label, path in input_specs:
        value, identity = read_bound_file(path, label)
        bound_inputs[label] = value
        named_identities.append((label, identity))
    input_identities = require_distinct_identities(named_identities)
    validate_output_target(output_path, input_identities)

    source_bytes = bound_inputs["amendment-semantics source"]
    kb_bytes = bound_inputs["constitution"]
    ledger_bytes = bound_inputs["assertion ledger"]
    assurance_bytes = bound_inputs["assurance source"]
    source = load_json_bytes(source_bytes, "amendment-semantics source")
    ledger = load_json_bytes(ledger_bytes, "assertion ledger")
    kb_text = decode_constitution_bytes(kb_bytes, "constitution")
    decode_utf8_exact(assurance_bytes, "assurance source")
    kb_digest = sha256_bytes(kb_bytes)
    ledger_digest = sha256_bytes(ledger_bytes)
    assurance_digest = sha256_bytes(assurance_bytes)
    validate_source(
        source, kb_text, kb_digest, ledger, ledger_digest, assurance_digest
    )
    generated = render(
        source,
        repo_relative(source_path),
        repo_relative(kb_path),
        repo_relative(ledger_path),
        repo_relative(assurance_path),
    )
    controls = negative_controls(
        source,
        kb_bytes,
        kb_text,
        kb_digest,
        ledger,
        ledger_digest,
        assurance_digest,
    )

    cases_run = pins_run = sabotage_controls = seam_controls = 0
    if args.execute:
        pin = select_pin(args.pin)
        cases_run, pins_run, sabotage_controls, seam_controls = execute_cases(
            source,
            kb_bytes,
            kb_text,
            pin,
            ledger_path,
            int(source["subprocess_timeout_seconds"]),
        )

    output_relative = repo_relative(output_path)
    generated_bytes = generated.encode("utf-8")
    validate_output_target(output_path, input_identities)
    if args.check:
        current, _ = read_bound_file(output_path, "generated report")
        if current != generated_bytes:
            raise AmendmentAuditError(f"{output_relative} is STALE — rerun without --check")
        suffix = (
            f"; {cases_run} isolated cases / {pins_run} pins execute; "
            f"{sabotage_controls} sabotage and {seam_controls} assertion-surface seam pass"
            if args.execute
            else "; execution skipped"
        )
        print(
            f"{output_relative} is current; {controls} structural negative controls pass{suffix}"
        )
        return 0

    write_generated_output(output_path, generated_bytes, input_identities)
    suffix = (
        f" after {cases_run} isolated cases / {pins_run} pins"
        if args.execute
        else " (structural generation; execution not requested)"
    )
    print(
        f"{output_relative}: regenerated{suffix}; {controls} structural negative controls pass"
    )
    return 0


if __name__ == "__main__":
    try:
        from verification_lock_client import run_checker

        raise SystemExit(run_checker(main, sys.argv[1:]))
    except AmendmentAuditError as exc:
        print(f"10-amendment-semantics: {exc}", file=sys.stderr)
        raise SystemExit(1)
