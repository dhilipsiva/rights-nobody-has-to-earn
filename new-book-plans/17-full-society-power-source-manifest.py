#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Validate the reviewed full-society power source-anchor manifest."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "new-book-plans/full-society-power-source-manifest.json"
EXPECTED_MANIFEST_SHA256 = (
    "fcb8b0d7052d7b35c41040a8040fa1f82ccf22d3d48fe6213fcc601bb44b9fbe"
)
EXPECTED_SOURCE_COMMIT = "36ed92c58877cffa5a11928ad200f0ca9a604820"
STATUS = (
    "reviewed-inventory-input-not-law-not-operation-"
    "not-completeness-beyond-bound-version"
)
ALLOWED_DISPOSITIONS = [
    "card-required",
    "power-contract-template",
    "existing-formal-crosswalk",
    "explicit-refusal-limit",
]
SOURCE_FAMILIES = {
    "new-book-plans/constitution.nibli": "current-formal-constitution",
    "new-book-plans/book-1-state-form-and-political-membership-decision.md":
        "state-form-and-political-membership",
    "new-book-plans/book-1-substantive-equality-and-anti-subordination-decision.md":
        "substantive-equality-and-anti-subordination",
    "new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md":
        "economic-pluralism-and-protected-private-sphere",
    "new-book-plans/book-1-family-dependency-reproduction-and-collective-plurality-decision.md":
        "family-dependency-reproduction-and-collective-plurality",
    "new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md":
        "ecological-commons-and-non-human-animal",
    "new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md":
        "public-safety-defence-emergency-and-external-power",
    "new-book-plans/book-1-time-model-decision.md": "time-model",
}
EXPECTED_SOURCE_SHA256 = {
    "new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md":
        "d9bda040307eed017b55be751b2a99a73910568b0e8fbafb27c5c185c4f5b13c",
    "new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md":
        "bd461df84e8aead78206c9c1653ac5b0f5fe9345c566a1beef1fa2fa8112f2be",
    "new-book-plans/book-1-family-dependency-reproduction-and-collective-plurality-decision.md":
        "6b42ef36e0ab54b8391f1d2bc174836a8f6f6130dfba4786c8e0483c10b34c59",
    "new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md":
        "eb5919bbe57107ca7b61df0aa367a44354771c1d60a963e9a98c305b78dd93a4",
    "new-book-plans/book-1-state-form-and-political-membership-decision.md":
        "55d921cdbf914f3b63bda91f20a29a54b4e9f219f8ffc18de11b89c2e89cc2e3",
    "new-book-plans/book-1-substantive-equality-and-anti-subordination-decision.md":
        "c27e4175ccbd3034a04d9cebbd80d314400fc290cff07997c1e314d04db28979",
    "new-book-plans/book-1-time-model-decision.md":
        "049f8d1ff0dda90e50751768c108d704d5eb908caf8a8b663092cb8bb79e7e44",
    "new-book-plans/constitution.nibli":
        "a1151d7b0865785b099ffa2ce9ea48bb0d92655006bdf7b186930ae776a554d9",
}
EXPECTED_BY_DISPOSITION = {
    "card-required": 209,
    "power-contract-template": 1,
    "existing-formal-crosswalk": 8,
    "explicit-refusal-limit": 19,
}
EXPECTED_BY_FAMILY = {
    "current-formal-constitution": 8,
    "ecological-commons-and-non-human-animal": 43,
    "economic-pluralism-and-protected-private-sphere": 29,
    "family-dependency-reproduction-and-collective-plurality": 31,
    "public-safety-defence-emergency-and-external-power": 64,
    "state-form-and-political-membership": 51,
    "substantive-equality-and-anti-subordination": 9,
    "time-model": 2,
}
EXPECTED_BY_FAMILY_AND_DISPOSITION = {
    "current-formal-constitution": {
        "card-required": 0, "power-contract-template": 0,
        "existing-formal-crosswalk": 8, "explicit-refusal-limit": 0,
    },
    "ecological-commons-and-non-human-animal": {
        "card-required": 40, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 3,
    },
    "economic-pluralism-and-protected-private-sphere": {
        "card-required": 28, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 1,
    },
    "family-dependency-reproduction-and-collective-plurality": {
        "card-required": 31, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 0,
    },
    "public-safety-defence-emergency-and-external-power": {
        "card-required": 50, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 14,
    },
    "state-form-and-political-membership": {
        "card-required": 51, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 0,
    },
    "substantive-equality-and-anti-subordination": {
        "card-required": 9, "power-contract-template": 0,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 0,
    },
    "time-model": {
        "card-required": 0, "power-contract-template": 1,
        "existing-formal-crosswalk": 0, "explicit-refusal-limit": 1,
    },
}
TOP_KEYS = {
    "spdx", "schema_version", "title", "status", "source_commit",
    "source_sha256", "allowed_dispositions", "grain_rule_anchor",
    "scope_note", "row_count", "coverage_summary", "rows",
}
ROW_KEYS = {
    "provisional_key", "title", "disposition", "source_anchor",
    "source_path", "source_needle", "legal_effect_and_grain", "source_family",
}


class ManifestError(RuntimeError):
    pass


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: pathlib.Path = SOURCE) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ManifestError(f"cannot load {path}: {exc}") from exc


def require_text(value, context: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ManifestError(f"{context}: non-empty text required")
    return value


def derived_summary(rows: list[dict]) -> dict:
    families = sorted(SOURCE_FAMILIES.values())
    return {
        "by_disposition": {
            value: sum(row["disposition"] == value for row in rows)
            for value in ALLOWED_DISPOSITIONS
        },
        "by_source_family": {
            family: sum(row["source_family"] == family for row in rows)
            for family in families
        },
        "by_source_family_and_disposition": {
            family: {
                value: sum(
                    row["source_family"] == family
                    and row["disposition"] == value
                    for row in rows
                )
                for value in ALLOWED_DISPOSITIONS
            }
            for family in families
        },
    }


def validate(source: dict, *, check_git: bool = True) -> None:
    if not isinstance(source, dict) or set(source) != TOP_KEYS:
        raise ManifestError("manifest must use the exact reviewed top-level schema")
    if source["spdx"] != "CC0-1.0" or source["schema_version"] != 1:
        raise ManifestError("manifest licence/schema must be CC0-1.0 / version 1")
    if source["title"] != "Provisional FS-POW source-anchor manifest":
        raise ManifestError("manifest title is not the reviewed title")
    if source["status"] != STATUS:
        raise ManifestError("manifest status must preserve the non-law ceiling")
    if source["source_commit"] != EXPECTED_SOURCE_COMMIT:
        raise ManifestError("source_commit differs from the reviewed base")
    if source["source_sha256"] != EXPECTED_SOURCE_SHA256:
        raise ManifestError("source_sha256 differs from the reviewed source set")
    if source["allowed_dispositions"] != ALLOWED_DISPOSITIONS:
        raise ManifestError("allowed_dispositions must equal the closed vocabulary")
    require_text(source["grain_rule_anchor"], "grain_rule_anchor")
    require_text(source["scope_note"], "scope_note")
    if any(term not in source["scope_note"] for term in (
            "creates no law", "Gate A result",
            "power-contract-template rows constrain cards")):
        raise ManifestError("scope_note lost an inventory-only boundary")

    rows = source["rows"]
    if not isinstance(rows, list) or len(rows) != 237 or source["row_count"] != 237:
        raise ManifestError("manifest must contain the reviewed 237-row population")
    summary = derived_summary(rows)
    if source["coverage_summary"] != summary:
        raise ManifestError("coverage_summary is stale relative to rows")
    if summary["by_disposition"] != EXPECTED_BY_DISPOSITION:
        raise ManifestError("disposition totals differ from the reviewed population")
    if summary["by_source_family"] != EXPECTED_BY_FAMILY:
        raise ManifestError("source-family totals differ from the reviewed population")
    if (summary["by_source_family_and_disposition"]
            != EXPECTED_BY_FAMILY_AND_DISPOSITION):
        raise ManifestError(
            "family/disposition matrix differs from the reviewed population"
        )

    for relative, expected in EXPECTED_SOURCE_SHA256.items():
        path = ROOT / relative
        if not path.is_file() or digest(path) != expected:
            raise ManifestError(f"source digest mismatch: {relative}")

    keys, titles = set(), set()
    for index, row in enumerate(rows):
        context = f"rows[{index}]"
        if not isinstance(row, dict) or set(row) != ROW_KEYS:
            raise ManifestError(f"{context}: exact row schema required")
        key = require_text(row["provisional_key"], f"{context}.provisional_key")
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", key):
            raise ManifestError(f"{context}: invalid provisional_key")
        if key in keys:
            raise ManifestError(f"{context}: duplicate provisional_key {key}")
        keys.add(key)
        title = require_text(row["title"], f"{context}.title")
        normalized = " ".join(title.casefold().split())
        if normalized in titles:
            raise ManifestError(f"{context}: duplicate normalized title")
        titles.add(normalized)
        disposition = row["disposition"]
        if disposition not in ALLOWED_DISPOSITIONS:
            raise ManifestError(f"{context}: unknown disposition {disposition!r}")
        relative = require_text(row["source_path"], f"{context}.source_path")
        if relative not in SOURCE_FAMILIES:
            raise ManifestError(f"{context}: source_path outside reviewed source set")
        if row["source_family"] != SOURCE_FAMILIES[relative]:
            raise ManifestError(f"{context}: source_family mismatches source_path")
        if ((relative == "new-book-plans/constitution.nibli")
                != (disposition == "existing-formal-crosswalk")):
            raise ManifestError(
                f"{context}: current-formal crosswalk disposition is misclassified"
            )
        needle = require_text(row["source_needle"], f"{context}.source_needle")
        if row["source_anchor"] != f"{relative}::{needle}":
            raise ManifestError(f"{context}: source_anchor is not path::needle exact")
        if (ROOT / relative).read_text(encoding="utf-8").count(needle) != 1:
            raise ManifestError(f"{context}: source needle must occur exactly once")
        require_text(
            row["legal_effect_and_grain"], f"{context}.legal_effect_and_grain"
        )

    grain_path, grain_needle = source["grain_rule_anchor"].split("::", 1)
    if grain_path != "new-book-plans/book-1-constitutional-coverage-map.md":
        raise ManifestError("grain_rule_anchor must name the coverage map")
    if (ROOT / grain_path).read_text(encoding="utf-8").count(grain_needle) != 1:
        raise ManifestError("grain_rule_anchor must resolve exactly once")

    if check_git:
        proc = subprocess.run(
            [
                "git", "-C", str(ROOT), "merge-base", "--is-ancestor",
                source["source_commit"], "HEAD",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if proc.returncode != 0:
            raise ManifestError("source_commit must be an ancestor of HEAD")


def negative_controls(source: dict) -> int:
    controls = []

    def add(name, mutate):
        controls.append((name, mutate))

    add("row removed", lambda s: s["rows"].pop())
    add("reviewed row count changed", lambda s: s.__setitem__("row_count", 236))
    add(
        "summary changed",
        lambda s: s["coverage_summary"]["by_disposition"].__setitem__(
            "card-required", 208
        ),
    )
    add(
        "duplicate key",
        lambda s: s["rows"][1].__setitem__(
            "provisional_key", s["rows"][0]["provisional_key"]
        ),
    )
    add(
        "duplicate title",
        lambda s: s["rows"][1].__setitem__("title", s["rows"][0]["title"]),
    )
    add(
        "unknown disposition",
        lambda s: s["rows"][0].__setitem__("disposition", "passed"),
    )
    add(
        "source family drift",
        lambda s: s["rows"][0].__setitem__("source_family", "time-model"),
    )
    add(
        "source anchor drift",
        lambda s: s["rows"][0].__setitem__("source_anchor", "TODO.md::missing"),
    )
    add(
        "missing source needle",
        lambda s: s["rows"][0].__setitem__("source_needle", "definitely absent"),
    )
    add(
        "source digest drift",
        lambda s: s["source_sha256"].__setitem__(
            next(iter(s["source_sha256"])), "0" * 64
        ),
    )
    add("ceiling removed", lambda s: s.__setitem__("scope_note", "Inventory."))
    add(
        "template promoted to power",
        lambda s: next(
            row for row in s["rows"]
            if row["provisional_key"] == "time-power-specific-t3-contract"
        ).__setitem__("disposition", "card-required"),
    )
    add("status promoted", lambda s: s.__setitem__("status", "complete"))
    add(
        "source commit drift",
        lambda s: s.__setitem__("source_commit", "0" * 40),
    )

    for name, mutate in controls:
        candidate = copy.deepcopy(source)
        mutate(candidate)
        try:
            validate(candidate, check_git=False)
        except ManifestError:
            continue
        raise ManifestError(f"negative control did not fail: {name}")
    return len(controls)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true", help="validate the reviewed manifest"
    )
    parser.parse_args()
    if digest(SOURCE) != EXPECTED_MANIFEST_SHA256:
        raise ManifestError(
            "reviewed manifest digest differs from the checker-bound artifact"
        )
    source = load()
    validate(source)
    count = negative_controls(source)
    print(
        "full-society power source manifest is current: 237 reviewed rows "
        "(209 card-required powers, 1 cross-power contract template, "
        "19 refusal/limit, 8 current-formal crosswalk); "
        f"{count} watched-failing mutations pass; inventory only -- no law, "
        "operation, FS-POW completion, or Gate A result"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ManifestError as exc:
        print(f"power manifest error: {exc}", file=sys.stderr)
        raise SystemExit(1)