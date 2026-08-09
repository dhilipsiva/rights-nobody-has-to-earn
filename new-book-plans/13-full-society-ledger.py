#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Full-society domain-and-layer ledger — canonical source validator/renderer.

Validates the reviewed canonical source (full-society-ledger.json) and renders
the generated report (full-society-ledger.md). The ledger is a REVIEWED ROUTING
INVENTORY: it establishes nothing beyond each row's own posture, executes no
engine query, and makes no Gate A claim. Classification is routing, not
assurance.

Stage 1 seeded domains, legacy coverage rows and their one-posture splits,
bodies, routes, external assumptions, the envelope stub, every closed enum, the
compatibility table, and the mechanical enum mapping over the six sibling
reviewed JSONs (which are read LIVE at --check, never digest-pinned, so a new
reviewed enum value fails here until its mapping row lands in the same change).
Stage 2 backfills every declared defect, repair narrative, open gap, and
sibling residual as keyed FS-DFT rows under a live-read citation closure,
computes resolution_status and blocking (never hand-authored), and mints
resolution receipts only where a row's generated resolution permits one.
Stage 4's machinery makes the future independent scope review admissible and
the Gate A closure refusable-until-green: proposal and review-event schemas
stand validated and empty, the candidate severity rubric awaits author
confirmation, per-condition Gate A readiness is computed and rendered, and a
closure record is refused while any condition computes unmet — closing Gate A
is a deliberate future contract amendment, never a latent flip.
Deferred record types carry explicit deferral records with owners.

Usage:
  python3 new-book-plans/13-full-society-ledger.py            # regenerate MD
  python3 new-book-plans/13-full-society-ledger.py --check    # validate + fresh
"""

import argparse
import copy
import hashlib
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = pathlib.Path("new-book-plans/full-society-ledger.json")
OUTPUT = pathlib.Path("new-book-plans/full-society-ledger.md")

# The two rulings whose text defines the ledger's enums and stopping rule. An
# edit there must force a deliberate refresh here. The six sibling reviewed
# JSONs are deliberately NOT digest-bound: they are read live at --check so the
# enum-mapping closure fails the moment a new reviewed enum value appears,
# without putting this artifact at the end of the 7->12 refresh cascade.
BOUND_SOURCES = {
    "assurance_portfolio": pathlib.Path(
        "new-book-plans/book-1-assurance-portfolio-decision.md"
    ),
    "full_society_boundary": pathlib.Path(
        "new-book-plans/full-society-boundary-decision.md"
    ),
}

SIBLING_SOURCES = [
    pathlib.Path("new-book-plans/assertion-surface-contracts.json"),
    pathlib.Path("new-book-plans/record-integrity-assurance-case.json"),
    pathlib.Path("new-book-plans/record-integrity-red-team.json"),
    pathlib.Path("new-book-plans/amendment-semantics-audit.json"),
    pathlib.Path("new-book-plans/placement-exhaustiveness-audit.json"),
    pathlib.Path("new-book-plans/temporal-assurance-case.json"),
]

# ID prefixes already in use across the six sibling reviewed JSONs. A ledger ID
# may never collide with this space; the ledger's own IDs are FS-XXX-NN.
LIVE_SIBLING_PREFIXES = frozenset(
    ["AS", "OE", "RA", "RC", "RD", "RF", "RI", "RS", "RT", "TA", "TP"]
)
ID_RE = re.compile(r"^FS-[A-Z]{3}-[0-9]{2,3}$")

# The five ratified routing dispositions. These ARE the reader-facing five
# layers (author-ratified 2026-08-09); one enum, one key: scope_disposition.
SCOPE_DISPOSITIONS = [
    "constitutional-invariant",
    "democratic-ordinary-law-choice",
    "protected-private-civic-freedom",
    "book-2-operation",
    "external-assumption",
]
# A domain record spans every layer (its buckets are the layers), so its layer
# field carries this sentinel; only leaf records take one of the five values.
DOMAIN_LAYER_SENTINEL = "spans-all-layers"

ROUTING_MARKERS = [
    "not-constitutionally-prescribed",
    "democratic-ordinary-law-choice",
    "book-2-operation",
    "external-assumption",
]

POSTURES = ["Derived", "Checked", "Evidenced", "Specified", "Reasoned", "Unestablished"]
ESTABLISHED_POSTURES = {"Derived", "Checked", "Evidenced"}
UNESTABLISHED_DISPOSITIONS = [
    "routed-book-2",
    "external-assumption",
    "route-unbuilt",
    "author-ruling-pending",
    "refused",
    "not-establishable",
]
EVIDENCE_KINDS = ["executable", "pattern-guard", "freshness", "inventory"]
OVERLAYS = ["safety", "liveness", "feasibility", "none"]
ROUTE_STATUSES = ["built", "available", "unbuilt"]
DEFECT_DISPOSITIONS = [
    "eliminated-structurally",
    "prevented",
    "protected-consequence-contained",
    "remedied",
    "externally-bounded-assumption",
    "irreducible-limitation",
    "open-defect",
]
NON_RESOLUTION_DISPOSITIONS = {
    "externally-bounded-assumption",
    "irreducible-limitation",
    "open-defect",
}
RESPONSE_STAGES = [
    "detected",
    "interface-specified",
    "implemented-in-assigned-route",
    "operationally-assured-in-envelope",
]
RESOLUTION_STATUSES = ["resolved-for-claim", "unresolved-for-claim"]
PROPOSAL_DISPOSITIONS = ["added", "classified-out", "retained-limit"]

# Stage marker: the reviewed source's status and the report's stage label move in
# lockstep with the content stages; bump both here and in the JSON together.
EXPECTED_STATUS = "stage_4_review_machinery"
STAGE_LABEL = "stage 4 machinery"

# The rubric's status string is exact in both states. It shipped as a
# candidate; the author confirmed it on 2026-08-09, and the confirmation basis
# is recorded on the rubric itself. An unconfirmed rubric computes as an unmet
# closure condition.
RUBRIC_STATUS_CANDIDATE = "candidate — author confirmation pending"
RUBRIC_STATUS_CONFIRMED = "author-confirmed 2026-08-09 — basis recorded"
READINESS_MET = {"met-mechanically", "met-in-form"}

# Second output: the coverage map's section-3 table is a generated region of
# this ledger — the ratified cell texts live verbatim-frozen on the legacy-row
# records and render unchanged, plus a generated split-claims column. Only the
# region between the markers is machine-owned; the heading, canonical-source
# note, and legend above it stay hand text.
COVERAGE_MAP = pathlib.Path("new-book-plans/book-1-constitutional-coverage-map.md")
REGION_RE = re.compile(
    r"(<!-- BEGIN GENERATED: full-society-coverage -->)(.*?)"
    r"(<!-- END GENERATED: full-society-coverage -->)",
    re.S,
)

SEVERITY_CLASSES = ("critical", "material", "minor")
SLUG_RE = re.compile(r"^[a-z0-9-]+$")
IMPLEMENTED_STAGES = {"implemented-in-assigned-route",
                      "operationally-assured-in-envelope"}
# Which typed control field each resolution-eligible disposition must carry once
# its stage reaches an implemented level. Non-eligible dispositions carry an
# empty controls object; a control value with a "not-yet" prefix is refused —
# a control that has never been watched failing is not yet a control.
CONTROL_REQUIREMENTS = {
    "eliminated-structurally": "reintroduction_control_ref",
    "prevented": "initiation_control_ref",
    "protected-consequence-contained": "containment_control_refs",
    "remedied": "recovery_fields",
}
RECOVERY_FIELD_KEYS = ["actor", "trigger", "interim_continuity", "restoration",
                       "challenge", "recurrence_control", "evidence_ref"]

DEFERRABLE_ARRAYS = [
    "roles",
    "powers",
    "dependencies",
    "scenarios",
    "thresholds",
    "defects",
    "receipts",
    "proposals",
    "review_events",
]
RECORD_ARRAYS = [
    "domains",
    "legacy_rows",
    "claims",
    "bodies",
    "routes",
    "external_assumptions",
    "envelope",
] + DEFERRABLE_ARRAYS

ENVELOPE_STUB_ID = "FS-ENV-00"

VERDICT_LINE = (
    "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN "
    "POSTURE; GATE A NOT PASSED"
)

# Enum detection over the sibling JSONs: (a) every key of a top-level
# `*_meanings` dict is an enum value; (b) every string value of a field whose
# leaf key is exactly one of these names is an enum value. `*_meaning`
# (singular) prose fields and free-text fields are deliberately outside the
# rule; deliberate exclusions are recorded in the reviewed source.
ENUM_LEAF_KEYS = frozenset(["posture", "status", "disposition", "verdict"])

FORBIDDEN_SCORE_KEYS = frozenset(
    ["score", "percentage", "total", "coverage_figure", "coverage_percent", "rank"]
)


class LedgerError(Exception):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def load_json(path: pathlib.Path):
    try:
        return json.loads((ROOT / path).read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise LedgerError(f"missing reviewed source: {path}")
    except json.JSONDecodeError as exc:
        raise LedgerError(f"{path} is not valid JSON: {exc}")


def exact_keys(obj: dict, required, context: str, optional=()):
    req = set(required)
    opt = set(optional)
    got = set(obj)
    missing = req - got
    extra = got - req - opt
    if missing:
        raise LedgerError(f"{context}: missing keys {sorted(missing)}")
    if extra:
        raise LedgerError(f"{context}: unexpected keys {sorted(extra)}")


def require_str(obj: dict, key: str, context: str) -> str:
    val = obj.get(key)
    if not isinstance(val, str) or not val.strip():
        raise LedgerError(f"{context}: `{key}` must be a non-empty string")
    return val


def validate_reference(ref: str, context: str):
    """`path::literal needle` — the needle must occur exactly once in path."""
    if not isinstance(ref, str) or ref.count("::") != 1:
        raise LedgerError(f"{context}: reference must be `path::needle`, got {ref!r}")
    rel, needle = ref.split("::", 1)
    if not rel or not needle:
        raise LedgerError(f"{context}: empty path or needle in {ref!r}")
    if rel.startswith("/") or ".." in rel or "\\" in rel:
        raise LedgerError(f"{context}: path must be repo-relative: {rel!r}")
    target = ROOT / rel
    if not target.is_file():
        raise LedgerError(f"{context}: reference target missing: {rel}")
    body = target.read_text(encoding="utf-8")
    count = body.count(needle)
    if count != 1:
        raise LedgerError(
            f"{context}: needle must occur exactly once in {rel}; found {count}: "
            f"{needle!r}"
        )


def validate_unresolved(obj: dict, context: str):
    exact_keys(
        obj,
        ["severity", "consequence", "owner_ref", "closure_condition",
         "public_claim_limitation"],
        context,
    )
    for key in ("severity", "consequence", "closure_condition",
                "public_claim_limitation"):
        require_str(obj, key, context)
    validate_reference(obj["owner_ref"], f"{context}.owner_ref")


def validate_bucket(bucket, context: str):
    """A domain layer bucket is exactly one of: answered, routed, unresolved."""
    if not isinstance(bucket, dict):
        raise LedgerError(f"{context}: bucket must be an object")
    shapes = [k for k in ("answer", "routing_marker", "unresolved") if k in bucket]
    if len(shapes) != 1:
        raise LedgerError(
            f"{context}: bucket must carry exactly one of answer / routing_marker "
            f"/ unresolved, got {shapes}"
        )
    if "answer" in bucket:
        exact_keys(bucket, ["answer", "refs"], context)
        require_str(bucket, "answer", context)
        if not isinstance(bucket["refs"], list) or not bucket["refs"]:
            raise LedgerError(f"{context}: answered bucket needs at least one ref")
        for i, ref in enumerate(bucket["refs"]):
            validate_reference(ref, f"{context}.refs[{i}]")
    elif "routing_marker" in bucket:
        exact_keys(bucket, ["routing_marker", "note"], context)
        if bucket["routing_marker"] not in ROUTING_MARKERS:
            raise LedgerError(
                f"{context}: routing_marker must be one of {ROUTING_MARKERS}"
            )
        require_str(bucket, "note", context)
    else:
        exact_keys(bucket, ["unresolved"], context)
        validate_unresolved(bucket["unresolved"], f"{context}.unresolved")


def check_no_generic_disposition(obj, context: str):
    """The axes stay distinct: no record may carry a bare `disposition` key."""
    if isinstance(obj, dict):
        if "disposition" in obj:
            raise LedgerError(
                f"{context}: generic `disposition` key is refused — use "
                "scope_disposition / unestablished_disposition / "
                "proposal_disposition / defect_disposition"
            )
        for k, v in obj.items():
            check_no_generic_disposition(v, f"{context}.{k}")
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            check_no_generic_disposition(v, f"{context}[{i}]")


def check_no_score_fields(obj, context: str):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in FORBIDDEN_SCORE_KEYS:
                raise LedgerError(
                    f"{context}: forbidden aggregate-score key `{k}` — the ledger "
                    "produces no score, total, percentage, or coverage figure"
                )
            check_no_score_fields(v, f"{context}.{k}")
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            check_no_score_fields(v, f"{context}[{i}]")


def validate_common_record_fields(rec: dict, context: str):
    for key in ("id", "title", "applicability", "status", "severity",
                "consequence", "closure_condition"):
        require_str(rec, key, context)
    validate_reference(rec["owner_ref"], f"{context}.owner_ref")
    if "partial formalisation" in rec["status"].lower():
        raise LedgerError(
            f"{context}: `partial formalisation` is retired — split the row; "
            "it was never a posture but an unsplit claim"
        )


COMMON_KEYS = [
    "id", "title", "applicability", "layer", "status", "severity",
    "consequence", "owner_ref", "closure_condition",
]

DOMAIN_BUCKETS = [
    "constitutional_invariants",
    "ordinary_law_choices",
    "protected_private_civic",
    "book2_operations",
    "external_assumptions_note",
]


def validate_header(src: dict):
    if src.get("spdx") != "CC-BY-4.0":
        raise LedgerError('reviewed source must declare "spdx": "CC-BY-4.0"')
    if src.get("schema_version") != 1:
        raise LedgerError("schema_version must be 1")
    require_str(src, "title", "header")
    if src.get("status") != EXPECTED_STATUS:
        raise LedgerError(f"status must be {EXPECTED_STATUS}")
    if src.get("evidence_role") != "reviewed_inventory_not_assurance":
        raise LedgerError("evidence_role must be reviewed_inventory_not_assurance")
    require_str(src, "source_version", "header")


def validate_bound_sources(src: dict):
    declared = src.get("bound_sources_sha256")
    if not isinstance(declared, dict) or set(declared) != set(BOUND_SOURCES):
        raise LedgerError(
            f"bound_sources_sha256 must declare exactly {sorted(BOUND_SOURCES)}"
        )
    for name, path in BOUND_SOURCES.items():
        actual = sha256(path)
        if declared[name] != actual:
            raise LedgerError(
                f"bound source `{name}` ({path}) digest mismatch: reviewed "
                f"{declared[name][:12]}… actual {actual[:12]}… — re-review the "
                "ruling change, then refresh without --check"
            )


def validate_meanings(src: dict):
    expected = {
        "scope_disposition_meanings": SCOPE_DISPOSITIONS,
        "posture_meanings": POSTURES,
        "unestablished_disposition_meanings": UNESTABLISHED_DISPOSITIONS,
        "evidence_kind_meanings": EVIDENCE_KINDS,
        "overlay_meanings": OVERLAYS,
        "route_status_meanings": ROUTE_STATUSES,
        "defect_disposition_meanings": DEFECT_DISPOSITIONS,
        "response_stage_meanings": RESPONSE_STAGES,
        "resolution_status_meanings": RESOLUTION_STATUSES,
        "proposal_disposition_meanings": PROPOSAL_DISPOSITIONS,
        "routing_marker_meanings": ROUTING_MARKERS,
    }
    for key, values in expected.items():
        block = src.get(key)
        if not isinstance(block, dict) or sorted(block) != sorted(values):
            raise LedgerError(
                f"{key} must define exactly {sorted(values)}"
            )
        for value, meaning in block.items():
            if not isinstance(meaning, str) or not meaning.strip():
                raise LedgerError(f"{key}.{value}: meaning must be prose")


def validate_axes(src: dict):
    axes = src.get("axes")
    if not isinstance(axes, list) or not axes:
        raise LedgerError("axes must be a non-empty list — the named axes of the "
                          "stopping rule are declared here")
    seen = set()
    for i, axis in enumerate(axes):
        ctx = f"axes[{i}]"
        exact_keys(axis, ["id", "name", "values", "note"], ctx)
        require_str(axis, "id", ctx)
        require_str(axis, "name", ctx)
        require_str(axis, "values", ctx)
        require_str(axis, "note", ctx)
        if axis["id"] in seen:
            raise LedgerError(f"{ctx}: duplicate axis id {axis['id']}")
        seen.add(axis["id"])
    required_axes = {
        "legal-effect-class", "social-domain", "layer", "posture",
        "route", "overlay", "defect-disposition", "response-stage",
    }
    if not required_axes <= seen:
        raise LedgerError(
            f"axes must include {sorted(required_axes)}; missing "
            f"{sorted(required_axes - seen)}"
        )


def validate_compatibility(src: dict):
    table = src.get("compatibility_table")
    if not isinstance(table, list):
        raise LedgerError("compatibility_table must be a list")
    covered = set()
    for i, row in enumerate(table):
        ctx = f"compatibility_table[{i}]"
        exact_keys(
            row,
            ["defect_disposition", "allowed_response_stages",
             "resolution_eligible", "resolution_requirement"],
            ctx,
        )
        dd = row["defect_disposition"]
        if dd not in DEFECT_DISPOSITIONS:
            raise LedgerError(f"{ctx}: unknown defect_disposition {dd!r}")
        covered.add(dd)
        stages = row["allowed_response_stages"]
        if (not isinstance(stages, list) or not stages
                or any(s not in RESPONSE_STAGES for s in stages)):
            raise LedgerError(f"{ctx}: allowed_response_stages invalid")
        if not isinstance(row["resolution_eligible"], bool):
            raise LedgerError(f"{ctx}: resolution_eligible must be boolean")
        require_str(row, "resolution_requirement", ctx)
        if dd in NON_RESOLUTION_DISPOSITIONS and row["resolution_eligible"]:
            raise LedgerError(
                f"{ctx}: {dd} is an explicit non-resolution boundary and can "
                "never be resolution-eligible"
            )
        if dd in {"eliminated-structurally", "prevented",
                  "protected-consequence-contained"}:
            bad = {"detected", "interface-specified"} & set(stages)
            if bad:
                raise LedgerError(
                    f"{ctx}: {dd} with {sorted(bad)} is an invalid combination"
                )
        if dd == "remedied" and stages != ["operationally-assured-in-envelope"]:
            raise LedgerError(
                f"{ctx}: remedied is resolution-eligible only with "
                "operationally-assured-in-envelope"
            )
    if covered != set(DEFECT_DISPOSITIONS):
        raise LedgerError(
            f"compatibility_table must cover every defect disposition; missing "
            f"{sorted(set(DEFECT_DISPOSITIONS) - covered)}"
        )


def validate_id_registry(src: dict):
    registry = src.get("id_registry")
    if not isinstance(registry, dict) or not registry:
        raise LedgerError("id_registry must map FS prefixes to record types")
    for prefix in registry:
        if not re.match(r"^FS-[A-Z]{3}$", prefix):
            raise LedgerError(f"id_registry prefix {prefix!r} must be FS-XXX")
    ids = {}
    for array in RECORD_ARRAYS:
        for rec in src.get(array, []):
            rid = rec.get("id")
            if not isinstance(rid, str) or not ID_RE.match(rid):
                raise LedgerError(
                    f"{array}: id {rid!r} must match FS-XXX-NN"
                )
            prefix = rid[:6]
            if prefix not in registry:
                raise LedgerError(f"{array}: id {rid} uses unregistered prefix")
            if rid.split("-")[1] in LIVE_SIBLING_PREFIXES:
                raise LedgerError(
                    f"{array}: id {rid} collides with a live sibling prefix"
                )
            if rid in ids:
                raise LedgerError(f"duplicate id {rid} in {array} and {ids[rid]}")
            ids[rid] = array
    return ids


def validate_domains(src: dict, ids: dict):
    domains = src.get("domains", [])
    clusters = set()
    for i, rec in enumerate(domains):
        ctx = f"domains[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + DOMAIN_BUCKETS + [
                "class_refs", "bodies_refs", "external_assumption_refs",
                "legacy_row_refs", "scenario_applicability",
                "reader_destination", "source_refs",
            ],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != DOMAIN_LAYER_SENTINEL:
            raise LedgerError(
                f"{ctx}: a domain spans every layer; its layer field must be "
                f"the sentinel {DOMAIN_LAYER_SENTINEL!r}"
            )
        for bucket_key in DOMAIN_BUCKETS:
            validate_bucket(rec[bucket_key], f"{ctx}.{bucket_key}")
        for key in ("class_refs",):
            refs = rec[key]
            if (not isinstance(refs, list) or not refs
                    or any(not re.match(r"^class-(0[1-9]|10)$", c) for c in refs)):
                raise LedgerError(
                    f"{ctx}: class_refs must name taxonomy classes class-01..class-10"
                )
        for key in ("bodies_refs", "external_assumption_refs", "legacy_row_refs"):
            refs = rec[key]
            if not isinstance(refs, list):
                raise LedgerError(f"{ctx}: {key} must be a list")
            for ref in refs:
                if ref not in ids:
                    raise LedgerError(f"{ctx}: {key} names unknown id {ref}")
        sa = rec["scenario_applicability"]
        if not (isinstance(sa, dict) and
                (set(sa) == {"deferred_ref"} or set(sa) == {"answer"})):
            raise LedgerError(
                f"{ctx}: scenario_applicability must be an answer or a deferred_ref"
            )
        if "deferred_ref" in sa:
            validate_reference(sa["deferred_ref"], f"{ctx}.scenario_applicability")
        require_str(rec, "reader_destination", ctx)
        srcs = rec["source_refs"]
        if not isinstance(srcs, list) or not srcs:
            raise LedgerError(f"{ctx}: source_refs must be a non-empty list")
        for j, ref in enumerate(srcs):
            validate_reference(ref, f"{ctx}.source_refs[{j}]")
        clusters.add(rec["id"])
    if len(domains) < 12:
        raise LedgerError(
            "domains must cover at least the twelve-cluster minimum social-domain "
            "inventory the tracker declares"
        )
    return clusters


def validate_legacy_rows(src: dict, ids: dict):
    rows = src.get("legacy_rows", [])
    for i, rec in enumerate(rows):
        ctx = f"legacy_rows[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            ["id", "domain_title", "legacy_coverage", "legacy_scope_requirement",
             "legacy_status_cell", "legacy_gap", "legacy_status", "domain_refs",
             "split_claim_refs", "split_state", "source_ref"],
            ctx, optional=["unresolved"],
        )
        require_str(rec, "domain_title", ctx)
        require_str(rec, "legacy_status", ctx)
        for key in ("legacy_coverage", "legacy_scope_requirement",
                    "legacy_status_cell", "legacy_gap"):
            val = require_str(rec, key, ctx)
            if "|" in val or "\n" in val:
                raise LedgerError(
                    f"{ctx}: {key} may not contain a table pipe or newline"
                )
        validate_reference(rec["source_ref"], f"{ctx}.source_ref")
        if rec["split_state"] not in ("split", "split-deferred"):
            raise LedgerError(f"{ctx}: split_state must be split or split-deferred")
        for ref in rec["domain_refs"]:
            if ref not in ids or ids[ref] != "domains":
                raise LedgerError(f"{ctx}: domain_refs names unknown domain {ref}")
        if not rec["domain_refs"]:
            raise LedgerError(f"{ctx}: every legacy row maps to at least one domain")
        if rec["split_state"] == "split":
            if not rec["split_claim_refs"]:
                raise LedgerError(f"{ctx}: split row needs split_claim_refs")
            if "unresolved" in rec:
                raise LedgerError(f"{ctx}: split row may not carry unresolved")
            for ref in rec["split_claim_refs"]:
                if ref not in ids or ids[ref] != "claims":
                    raise LedgerError(f"{ctx}: unknown claim {ref}")
        else:
            if rec["split_claim_refs"]:
                raise LedgerError(
                    f"{ctx}: split-deferred row may not name claims"
                )
            if "unresolved" not in rec:
                raise LedgerError(
                    f"{ctx}: split-deferred row must carry an unresolved object"
                )
            validate_unresolved(rec["unresolved"], f"{ctx}.unresolved")


def validate_claims(src: dict, ids: dict, routes_by_id: dict):
    claims = src.get("claims", [])
    for i, rec in enumerate(claims):
        ctx = f"claims[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + [
                "claim", "domain_refs", "legacy_row_ref", "class_refs",
                "posture", "route_ref", "overlay", "scope_bound",
                "evidence_notes", "public_claim_restriction", "envelope_id",
            ],
            ctx,
            optional=["evidence_kind", "mutation_ref",
                      "unestablished_disposition", "unimplemented_marker"],
        )
        validate_common_record_fields(rec, ctx)
        require_str(rec, "claim", ctx)
        require_str(rec, "scope_bound", ctx)
        require_str(rec, "public_claim_restriction", ctx)
        if rec["layer"] not in SCOPE_DISPOSITIONS:
            raise LedgerError(
                f"{ctx}: a leaf record's layer must be one of the five "
                f"dispositions, got {rec['layer']!r}"
            )
        posture = rec["posture"]
        if posture not in POSTURES:
            raise LedgerError(f"{ctx}: claim with no recognised posture")
        if not isinstance(rec.get("evidence_notes"), list):
            raise LedgerError(f"{ctx}: evidence_notes must be a list of strings")
        if rec["envelope_id"] != ENVELOPE_STUB_ID:
            raise LedgerError(
                f"{ctx}: envelope_id must name the explicit stub "
                f"{ENVELOPE_STUB_ID} until the envelope is versioned"
            )
        for ref in rec["domain_refs"]:
            if ref not in ids or ids[ref] != "domains":
                raise LedgerError(f"{ctx}: unknown domain {ref}")
        lr = rec["legacy_row_ref"]
        if lr is not None and (lr not in ids or ids[lr] != "legacy_rows"):
            raise LedgerError(f"{ctx}: unknown legacy row {lr}")
        route = routes_by_id.get(rec["route_ref"])
        if route is None:
            raise LedgerError(f"{ctx}: route_ref names unknown route")
        if posture in ESTABLISHED_POSTURES and route["route_status"] == "unbuilt":
            raise LedgerError(
                f"{ctx}: established posture {posture} on route "
                f"{route['id']} which is neither built nor available"
            )
        if posture == "Derived":
            if rec.get("evidence_kind") != "executable":
                raise LedgerError(
                    f"{ctx}: a Derived row's evidence kind must be executable"
                )
            notes = " ".join(rec["evidence_notes"]).lower()
            if "mutation" in notes and "mutation_ref" not in rec:
                raise LedgerError(
                    f"{ctx}: Derived over a mutation needs a mutation_ref"
                )
        elif posture == "Checked":
            if rec.get("evidence_kind") not in ("pattern-guard", "freshness",
                                                "inventory"):
                raise LedgerError(
                    f"{ctx}: a Checked row's evidence kind must be pattern-guard, "
                    "freshness, or inventory"
                )
            if re.search(r"\bimpossib", rec["claim"], re.I):
                raise LedgerError(
                    f"{ctx}: a Checked row may not be phrased as an impossibility"
                )
        elif "evidence_kind" in rec:
            raise LedgerError(
                f"{ctx}: evidence_kind belongs only on Derived or Checked rows"
            )
        if posture == "Specified" and rec.get("unimplemented_marker") is not True:
            raise LedgerError(
                f"{ctx}: a Specified row needs its explicit unimplemented marker"
            )
        if posture == "Unestablished":
            ud = rec.get("unestablished_disposition")
            if ud not in UNESTABLISHED_DISPOSITIONS:
                raise LedgerError(
                    f"{ctx}: an Unestablished row needs one named disposition"
                )
            if ud == "route-unbuilt":
                # severity/consequence/owner/closure are already mandatory on
                # every record; the claim restriction is the extra field.
                if "restricted" not in rec["public_claim_restriction"].lower() \
                        and "no public" not in rec["public_claim_restriction"].lower():
                    raise LedgerError(
                        f"{ctx}: a route-unbuilt row must state its public-claim "
                        "restriction"
                    )
        elif "unestablished_disposition" in rec:
            raise LedgerError(
                f"{ctx}: unestablished_disposition belongs only on Unestablished rows"
            )
        overlay = rec["overlay"]
        if overlay not in OVERLAYS:
            raise LedgerError(f"{ctx}: unknown overlay {overlay!r}")
        if overlay == "liveness" and posture != "Unestablished":
            raise LedgerError(
                f"{ctx}: a liveness claim may never take an established, "
                "Specified, or Reasoned posture — it is Unestablished until "
                "operational assurance exists"
            )
        if overlay == "feasibility":
            raise LedgerError(
                f"{ctx}: a feasibility claim may not appear in Book 1 at all"
            )
        if "mutation_ref" in rec and posture != "Derived":
            raise LedgerError(f"{ctx}: mutation_ref belongs only on Derived rows")
        if "resolution_status" in rec:
            raise LedgerError(
                f"{ctx}: resolution_status is generated, never hand-authored"
            )


def validate_bodies(src: dict):
    for i, rec in enumerate(src.get("bodies", [])):
        ctx = f"bodies[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["job", "may_not_do_alone", "required_check",
                           "source_ref"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != "constitutional-invariant":
            raise LedgerError(
                f"{ctx}: a required body is constitutional architecture; its "
                "layer is constitutional-invariant"
            )
        for key in ("job", "may_not_do_alone", "required_check"):
            require_str(rec, key, ctx)
        validate_reference(rec["source_ref"], f"{ctx}.source_ref")


def validate_routes(src: dict):
    routes = src.get("routes", [])
    by_id = {}
    for i, rec in enumerate(routes):
        ctx = f"routes[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["route_status", "warrants", "cannot_warrant",
                           "falsification_condition", "negative_control",
                           "source_ref"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["route_status"] not in ROUTE_STATUSES:
            raise LedgerError(f"{ctx}: unknown route_status")
        require_str(rec, "warrants", ctx)
        require_str(rec, "cannot_warrant", ctx)
        validate_reference(rec["source_ref"], f"{ctx}.source_ref")
        if rec["route_status"] in ("built", "available"):
            for key in ("falsification_condition", "negative_control"):
                val = rec.get(key)
                if not isinstance(val, str) or not val.strip() \
                        or val.strip().lower().startswith("not-yet"):
                    raise LedgerError(
                        f"{ctx}: a {rec['route_status']} route must declare its "
                        f"{key} — a route with no watched-failing control is not "
                        "yet a route"
                    )
        by_id[rec["id"]] = rec
    if len(routes) != 7:
        raise LedgerError("routes must carry exactly the seven ratified routes")
    return by_id


def validate_external_assumptions(src: dict):
    for i, rec in enumerate(src.get("external_assumptions", [])):
        ctx = f"external_assumptions[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec, COMMON_KEYS + ["assumption", "failure_consequence", "source_ref"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != "external-assumption":
            raise LedgerError(f"{ctx}: layer must be external-assumption")
        require_str(rec, "assumption", ctx)
        require_str(rec, "failure_consequence", ctx)
        validate_reference(rec["source_ref"], f"{ctx}.source_ref")


def validate_envelope(src: dict):
    env = src.get("envelope", [])
    if len(env) != 1 or env[0].get("id") != ENVELOPE_STUB_ID:
        raise LedgerError(
            f"envelope must hold exactly the {ENVELOPE_STUB_ID} stub until the "
            "reference-envelope item versions it"
        )
    rec = env[0]
    ctx = f"envelope ({ENVELOPE_STUB_ID})"
    exact_keys(rec, COMMON_KEYS + ["note"], ctx)
    validate_common_record_fields(rec, ctx)
    if rec["status"] != "not-yet-versioned":
        raise LedgerError(f"{ctx}: the stub's status is not-yet-versioned")
    if rec["layer"] != "external-assumption":
        raise LedgerError(
            f"{ctx}: until versioned, the envelope is an external assumption"
        )
    require_str(rec, "note", ctx)


def severity_class(rec: dict) -> str:
    sev = rec.get("severity", "")
    for cls in SEVERITY_CLASSES:
        if sev.startswith(cls + " — "):
            return cls
    raise LedgerError(
        f"defect {rec.get('id', '?')}: severity must carry a class prefix "
        f"({' / '.join(SEVERITY_CLASSES)}) followed by ' — ' and prose"
    )


def _validate_controls(rec: dict, compat_row: dict, ctx: str):
    controls = rec.get("controls")
    if not isinstance(controls, dict):
        raise LedgerError(f"{ctx}: controls must be an object")
    for val in json.dumps(controls).split('"'):
        if val.startswith("not-yet"):
            raise LedgerError(
                f"{ctx}: a control with a not-yet prefix is not a control"
            )
    dd = rec["defect_disposition"]
    required = CONTROL_REQUIREMENTS.get(dd)
    needs_controls = (compat_row["resolution_eligible"]
                      and rec["response_stage"] in IMPLEMENTED_STAGES)
    if not needs_controls:
        if controls:
            raise LedgerError(
                f"{ctx}: controls belong only on resolution-eligible rows at an "
                "implemented stage"
            )
        return
    if set(controls) != {required}:
        raise LedgerError(
            f"{ctx}: {dd} at an implemented stage requires exactly the "
            f"`{required}` control"
        )
    val = controls[required]
    if required == "containment_control_refs":
        if not isinstance(val, list) or not val:
            raise LedgerError(f"{ctx}: containment needs a non-empty control list")
        for j, ref in enumerate(val):
            validate_reference(ref, f"{ctx}.controls[{j}]")
    elif required == "recovery_fields":
        exact_keys(val, RECOVERY_FIELD_KEYS, f"{ctx}.controls.recovery_fields")
        for key in RECOVERY_FIELD_KEYS[:-1]:
            require_str(val, key, f"{ctx}.controls.recovery_fields")
        validate_reference(val["evidence_ref"],
                           f"{ctx}.controls.recovery_fields.evidence_ref")
    else:
        validate_reference(val, f"{ctx}.controls.{required}")


def validate_defect_rows(src: dict, ids: dict):
    """The ratified stage-2 keying, control, and polarity rules."""
    defect_ids = {rec.get("id") for rec in src.get("defects", [])}
    seen_tuples = {}
    for i, rec in enumerate(src.get("defects", [])):
        ctx = f"defects[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["defect_id", "defect_disposition", "response_stage",
                           "affected_claim_ref", "consequence_id", "scope_id",
                           "envelope_id", "source_version", "history",
                           "evidence_notes", "residual_citations", "controls"],
            ctx, optional=["book2_crosswalk"],
        )
        validate_common_record_fields(rec, ctx)
        severity_class(rec)
        if rec["layer"] not in SCOPE_DISPOSITIONS:
            raise LedgerError(f"{ctx}: a defect row's layer must be one of the "
                              "five dispositions")
        if rec["defect_id"] not in defect_ids:
            raise LedgerError(
                f"{ctx}: defect_id must name a defects row (the family's "
                "primary row; residual siblings share it)"
            )
        for key in ("consequence_id", "scope_id"):
            val = rec.get(key)
            if not isinstance(val, str) or not SLUG_RE.match(val):
                raise LedgerError(f"{ctx}: {key} must be a kebab-case slug")
        require_str(rec, "source_version", ctx)
        dd = rec["defect_disposition"]
        stage = rec["response_stage"]
        if dd not in DEFECT_DISPOSITIONS or stage not in RESPONSE_STAGES:
            raise LedgerError(f"{ctx}: unknown disposition or stage")
        compat_row = next(
            r for r in src["compatibility_table"]
            if r["defect_disposition"] == dd
        )
        if stage not in compat_row["allowed_response_stages"]:
            raise LedgerError(
                f"{ctx}: {dd} + {stage} is an invalid combination"
            )
        ref = rec["affected_claim_ref"]
        if ref not in ids or ids[ref] != "claims":
            raise LedgerError(
                f"{ctx}: affected_claim_ref must name a claim record"
            )
        if rec["envelope_id"] == ENVELOPE_STUB_ID and \
                stage == "operationally-assured-in-envelope":
            raise LedgerError(
                f"{ctx}: the envelope stub can route, never assure — no "
                "operationally-assured stage may cite it"
            )
        for key in ("resolution_status", "blocking"):
            if key in rec:
                raise LedgerError(
                    f"{ctx}: {key} is generated, never hand-authored"
                )
        if not isinstance(rec["history"], list):
            raise LedgerError(f"{ctx}: history must be a list")
        for j, entry in enumerate(rec["history"]):
            hctx = f"{ctx}.history[{j}]"
            exact_keys(entry, ["field", "value", "date", "note"], hctx)
            if entry["field"] == "defect_disposition":
                if entry["value"] not in DEFECT_DISPOSITIONS:
                    raise LedgerError(f"{hctx}: unknown prior disposition")
            elif entry["field"] == "response_stage":
                if entry["value"] not in RESPONSE_STAGES:
                    raise LedgerError(f"{hctx}: unknown prior stage")
            else:
                raise LedgerError(f"{hctx}: field must name the disposition or "
                                  "stage axis")
            require_str(entry, "date", hctx)
            require_str(entry, "note", hctx)
        for key in ("evidence_notes", "residual_citations"):
            val = rec[key]
            if not isinstance(val, list) or \
                    any(not isinstance(s, str) or not s for s in val):
                raise LedgerError(f"{ctx}: {key} must be a list of strings")
        if "book2_crosswalk" in rec and rec["book2_crosswalk"] is not True:
            raise LedgerError(f"{ctx}: book2_crosswalk may only be true")
        _validate_controls(rec, compat_row, ctx)
        key_tuple = (rec["defect_id"], rec["affected_claim_ref"],
                     rec["consequence_id"], rec["scope_id"],
                     rec["envelope_id"], rec["source_version"])
        if key_tuple in seen_tuples:
            raise LedgerError(
                f"{ctx}: keying tuple duplicates {seen_tuples[key_tuple]} — "
                "one current disposition and stage per keyed row"
            )
        seen_tuples[key_tuple] = rec["id"]


RECEIPT_KEYS = [
    "id", "title", "defect_row_ref", "defect_id", "affected_claim_ref",
    "consequence_id", "defect_disposition", "response_stage", "claim_posture",
    "route_ref", "admissible_evidence", "assurance_ceiling", "what_failed",
    "hostile_witness", "why_it_failed", "response_change", "now_follows",
    "proof_ref", "negative_control_ref", "still_does_not_follow", "residuals",
    "scope_id", "source_version", "envelope_id", "owner_ref", "eligible_gate",
    "reader_mapping_ref",
]


def validate_receipts(src: dict, ids: dict, resolution: dict):
    defects_by_id = {r["id"]: r for r in src.get("defects", [])}
    claims_by_id = {r["id"]: r for r in src.get("claims", [])}
    seen_rows = set()
    for i, rec in enumerate(src.get("receipts", [])):
        ctx = f"receipts[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, RECEIPT_KEYS, ctx)
        for key in ("title", "admissible_evidence", "what_failed",
                    "hostile_witness", "why_it_failed", "response_change",
                    "now_follows", "still_does_not_follow", "eligible_gate"):
            require_str(rec, key, ctx)
        row = defects_by_id.get(rec["defect_row_ref"])
        if row is None:
            raise LedgerError(f"{ctx}: defect_row_ref names no defect row")
        if rec["defect_row_ref"] in seen_rows:
            raise LedgerError(f"{ctx}: one receipt max per defect row")
        seen_rows.add(rec["defect_row_ref"])
        for key in ("defect_id", "affected_claim_ref", "consequence_id",
                    "defect_disposition", "response_stage", "scope_id",
                    "source_version", "envelope_id"):
            if rec[key] != row[key]:
                raise LedgerError(
                    f"{ctx}: {key} must equal the defect row's — receipts "
                    "duplicate nothing loosely"
                )
        claim = claims_by_id[row["affected_claim_ref"]]
        if rec["claim_posture"] != claim["posture"]:
            raise LedgerError(f"{ctx}: claim_posture must equal the claim's")
        if rec["route_ref"] != claim["route_ref"]:
            raise LedgerError(f"{ctx}: route_ref must equal the claim's")
        if rec["assurance_ceiling"] != claim["posture"]:
            raise LedgerError(
                f"{ctx}: assurance_ceiling is the affected claim's posture — "
                "a receipt may not assert beyond it"
            )
        for key in ("proof_ref", "negative_control_ref", "reader_mapping_ref",
                    "owner_ref"):
            validate_reference(rec[key], f"{ctx}.{key}")
        residuals = rec["residuals"]
        if not isinstance(residuals, list) or not residuals or \
                any(not isinstance(s, str) or not s for s in residuals):
            raise LedgerError(f"{ctx}: residuals must be a non-empty list — a "
                              "receipt never implies it cured a wider defect")
        siblings = [d["id"] for d in src.get("defects", [])
                    if d["defect_id"] == row["defect_id"]
                    and d["id"] != row["id"]]
        named = [s for s in residuals if s in defects_by_id]
        for s in named:
            if s not in siblings:
                raise LedgerError(
                    f"{ctx}: residual {s} is not a sibling of this defect family"
                )
        if siblings and not named:
            raise LedgerError(
                f"{ctx}: this defect family has residual sibling rows "
                f"({', '.join(siblings)}) and the receipt must name one"
            )
        if not resolution[row["id"]]["candidate"]:
            raise LedgerError(
                f"{ctx}: receipt on a non-candidate row — resolution may not "
                "exceed the affected claim's ceiling or skip its controls"
            )


def compute_resolution(src: dict) -> dict:
    """Generated, never authored: resolution_status and blocking per defect."""
    compat = {r["defect_disposition"]: r for r in src["compatibility_table"]}
    claims_by_id = {r["id"]: r for r in src.get("claims", [])}
    receipts_by_row = {}
    for rcp in src.get("receipts", []):
        receipts_by_row.setdefault(rcp.get("defect_row_ref"), []).append(rcp)
    out = {}
    for rec in src.get("defects", []):
        row = compat[rec["defect_disposition"]]
        claim = claims_by_id.get(rec["affected_claim_ref"], {})
        stage_ok = rec["response_stage"] in IMPLEMENTED_STAGES and \
            rec["response_stage"] in row["allowed_response_stages"]
        required = CONTROL_REQUIREMENTS.get(rec["defect_disposition"])
        controls_ok = bool(required) and required in rec.get("controls", {})
        ceiling_ok = claim.get("posture") in ESTABLISHED_POSTURES
        if rec["defect_disposition"] == "remedied":
            ceiling_ok = (
                claim.get("posture") == "Evidenced"
                and claim.get("overlay") == "liveness"
                and claim.get("route_ref") == "FS-RTE-05"
                and rec["response_stage"] == "operationally-assured-in-envelope"
                and rec["envelope_id"] != ENVELOPE_STUB_ID
            )
        candidate = (row["resolution_eligible"] and stage_ok and controls_ok
                     and ceiling_ok)
        receipts = receipts_by_row.get(rec["id"], [])
        status = ("resolved-for-claim"
                  if candidate and len(receipts) == 1
                  else "unresolved-for-claim")
        blocking = (severity_class(rec) == "critical"
                    and status == "unresolved-for-claim")
        out[rec["id"]] = {"candidate": candidate, "resolution_status": status,
                          "blocking": blocking}
    return out


def collect_sibling_residuals():
    """Live-read the sibling reviewed JSONs' residual pools. Narrowness-impact
    rows never enter the pool: they are claim-impact anchors, not defects."""
    pool = set()
    ri = load_json(pathlib.Path(
        "new-book-plans/record-integrity-assurance-case.json"))
    pool |= {f"record-integrity-assurance-case#{c['id']}"
             for c in ri["claims"] if c.get("posture") != "current_verified"}
    pool |= {f"record-integrity-assurance-case#{d['id']}"
             for d in ri["defeaters"]}
    pool |= {f"record-integrity-assurance-case#limitations.{k}"
             for k in ri["limitations"]}
    rt = load_json(pathlib.Path("new-book-plans/record-integrity-red-team.json"))
    pool |= {f"record-integrity-red-team#{s['id']}" for s in rt["scenarios"]}
    pool |= {f"record-integrity-red-team#{o['id']}"
             for o in rt["observational_equivalence"]}
    pool |= {f"record-integrity-red-team#limits.{k}" for k in rt["limits"]}
    ta = load_json(pathlib.Path("new-book-plans/temporal-assurance-case.json"))
    pool |= {f"temporal-assurance-case#attacks.{a['id']}"
             for a in ta["attacks"]
             if a.get("posture") == "exposed_external_boundary"}
    pool |= {f"temporal-assurance-case#limits.{k}" for k in ta["limits"]}
    pe = load_json(pathlib.Path(
        "new-book-plans/placement-exhaustiveness-audit.json"))
    pool |= {f"placement-exhaustiveness-audit#limits.{k}" for k in pe["limits"]}
    am = load_json(pathlib.Path("new-book-plans/amendment-semantics-audit.json"))
    pool |= {f"amendment-semantics-audit#limits.{k}" for k in am["limits"]}
    asf = load_json(pathlib.Path(
        "new-book-plans/assertion-surface-contracts.json"))
    pool |= {f"assertion-surface-contracts#premises.{k}"
             for k, v in asf["premises"].items()
             if "deliberately_refused" in json.dumps(v.get("risk_dispositions"))}
    return pool


def validate_residual_coverage(src: dict):
    exclusions = src.get("residual_coverage_exclusions")
    if not isinstance(exclusions, list):
        raise LedgerError("residual_coverage_exclusions must be a list")
    excluded = set()
    for i, row in enumerate(exclusions):
        ctx = f"residual_coverage_exclusions[{i}]"
        exact_keys(row, ["source_file", "token", "reason"], ctx)
        for key in ("source_file", "token", "reason"):
            require_str(row, key, ctx)
        excluded.add(row["token"])
    cited = set()
    for rec in src.get("defects", []):
        cited |= set(rec.get("residual_citations", []))
    pool = collect_sibling_residuals()
    uncovered = pool - cited - excluded
    if uncovered:
        raise LedgerError(
            "sibling residuals uncovered by any defect row (cite or exclude "
            f"with a reason, in the same change): {sorted(uncovered)[:6]}"
        )
    stale = (cited | excluded) - pool
    if stale:
        raise LedgerError(
            f"residual citations or exclusions name tokens no sibling source "
            f"declares: {sorted(stale)[:6]}"
        )


def validate_deferred(src: dict):
    deferrals = src.get("deferred_populations")
    if not isinstance(deferrals, list):
        raise LedgerError("deferred_populations must be a list")
    by_type = {}
    for i, rec in enumerate(deferrals):
        ctx = f"deferred_populations[{i}]"
        exact_keys(rec, ["record_type", "owner_ref", "closure_condition",
                         "stage"], ctx)
        rt = rec["record_type"]
        if rt not in DEFERRABLE_ARRAYS:
            raise LedgerError(f"{ctx}: {rt!r} is not a deferrable record type")
        if rt in by_type:
            raise LedgerError(f"{ctx}: duplicate deferral for {rt}")
        validate_reference(rec["owner_ref"], f"{ctx}.owner_ref")
        require_str(rec, "closure_condition", ctx)
        require_str(rec, "stage", ctx)
        by_type[rt] = rec
    for array in DEFERRABLE_ARRAYS:
        rows = src.get(array)
        if not isinstance(rows, list):
            raise LedgerError(f"{array} must be present as a list")
        if not rows and array not in by_type:
            raise LedgerError(
                f"{array} is empty with no deferral record — an empty array "
                "needs an owner and a closure condition"
            )
        if rows and array in by_type:
            raise LedgerError(
                f"{array} is populated but still carries a deferral record"
            )


def collect_sibling_enums():
    """Live-read the six sibling reviewed JSONs and inventory their enums."""
    found = []
    for path in SIBLING_SOURCES:
        data = load_json(path)
        for key, val in data.items():
            if key.endswith("_meanings") and isinstance(val, dict):
                for value in val:
                    found.append((path.name, key, value))

        def walk(obj, field_path):
            if isinstance(obj, dict):
                for k, v in obj.items():
                    if isinstance(v, str) and k in ENUM_LEAF_KEYS:
                        found.append((path.name, k, v))
                    else:
                        walk(v, k)
            elif isinstance(obj, list):
                for item in obj:
                    walk(item, field_path)

        walk(data, "")
    return sorted(set(found))


def validate_enum_mapping(src: dict):
    mapping = src.get("enum_mapping")
    exclusions = src.get("enum_mapping_exclusions")
    if not isinstance(mapping, list) or not isinstance(exclusions, list):
        raise LedgerError("enum_mapping and enum_mapping_exclusions must be lists")
    mapped = set()
    for i, row in enumerate(mapping):
        ctx = f"enum_mapping[{i}]"
        exact_keys(row, ["source_file", "field", "value", "canonical", "note"],
                   ctx)
        for key in ("source_file", "field", "value", "canonical", "note"):
            require_str(row, key, ctx)
        mapped.add((row["source_file"], row["field"], row["value"]))
    excluded = set()
    for i, row in enumerate(exclusions):
        ctx = f"enum_mapping_exclusions[{i}]"
        exact_keys(row, ["source_file", "field", "value", "reason"], ctx)
        for key in ("source_file", "field", "value", "reason"):
            require_str(row, key, ctx)
        excluded.add((row["source_file"], row["field"], row["value"]))
    live = set(collect_sibling_enums())
    unmapped = live - mapped - excluded
    if unmapped:
        sample = sorted(unmapped)[:6]
        raise LedgerError(
            "reviewed enum values with no mapping row (map them mechanically in "
            f"the same change): {sample}"
        )
    stale = (mapped | excluded) - live
    if stale:
        raise LedgerError(
            f"enum mapping names values no sibling source declares: "
            f"{sorted(stale)[:6]}"
        )


def validate_stopping_rule(src: dict):
    rule = src.get("stopping_rule")
    if not isinstance(rule, dict):
        raise LedgerError("stopping_rule must be an object")
    exact_keys(
        rule,
        ["named_axes", "closure_conditions", "materiality_test", "boundary",
         "no_hiding_rule", "source_ref"],
        "stopping_rule",
    )
    axes_ids = {a["id"] for a in src.get("axes", [])}
    if set(rule["named_axes"]) != axes_ids:
        raise LedgerError(
            "stopping_rule.named_axes must name exactly the declared axes"
        )
    conds = rule["closure_conditions"]
    if not isinstance(conds, list) or len(conds) != 5:
        raise LedgerError(
            "closure_conditions must state the five ratified conditions of the "
            "versioned-closure rule"
        )
    for key in ("materiality_test", "boundary", "no_hiding_rule"):
        require_str(rule, key, "stopping_rule")
    if "not a timeless completeness theorem" not in rule["boundary"]:
        raise LedgerError(
            "stopping_rule.boundary must carry the ratified bound: versioned "
            "exhaustiveness, not a timeless completeness theorem"
        )
    validate_reference(rule["source_ref"], "stopping_rule.source_ref")


def collect_map_needles(src) -> set:
    """Every needle in the reviewed source that targets the coverage map."""
    needles = set()

    def walk(obj):
        if isinstance(obj, dict):
            for v in obj.values():
                walk(v)
        elif isinstance(obj, list):
            for v in obj:
                walk(v)
        elif isinstance(obj, str) and obj.startswith(str(COVERAGE_MAP) + "::"):
            needles.add(obj.split("::", 1)[1])

    walk(src)
    return needles


def render_coverage_region(src: dict) -> str:
    claims_by_id = {c["id"]: c for c in src["claims"]}
    lines = [
        "| Domain | Current Book 1 coverage | Ratified scope requirement | "
        "Status | Gap / author ruling required | Split claims (posture) |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for row in src["legacy_rows"]:
        splits = []
        for cid in row["split_claim_refs"]:
            claim = claims_by_id[cid]
            posture = claim["posture"]
            if posture == "Unestablished":
                posture += "/" + claim["unestablished_disposition"]
            splits.append(f"{cid} ({posture})")
        lines.append(
            "| " + " | ".join([
                row["domain_title"], row["legacy_coverage"],
                row["legacy_scope_requirement"], row["legacy_status_cell"],
                row["legacy_gap"], "; ".join(splits) or "—",
            ]) + " |"
        )
    return "\n".join(lines)


def validate_coverage_region(src: dict):
    """Guards on the generated region, run against the in-memory splice: no
    generated heading lines, and every ledger needle targeting the map must
    still occur exactly once in the spliced result."""
    body = render_coverage_region(src)
    for line in body.splitlines():
        if line.startswith("## ") or line.startswith("### "):
            raise LedgerError(
                "the generated coverage region may not emit a heading line"
            )
    map_path = ROOT / COVERAGE_MAP
    if not map_path.is_file():
        raise LedgerError(f"missing coverage map: {COVERAGE_MAP}")
    text = map_path.read_text(encoding="utf-8")
    if not REGION_RE.search(text):
        raise LedgerError(
            "coverage map has no generated region — add the BEGIN/END markers "
            "first"
        )
    spliced = splice_coverage(text, body)
    for needle in collect_map_needles(src):
        count = spliced.count(needle)
        if count != 1:
            raise LedgerError(
                f"after splicing, coverage-map needle must occur exactly once; "
                f"found {count}: {needle!r}"
            )
    return spliced


def splice_coverage(text: str, body: str) -> str:
    return REGION_RE.sub(lambda m: m.group(1) + "\n" + body + "\n" + m.group(3),
                         text, count=1)


REVIEW_EVENT_KEYS = [
    "id", "title", "reviewers", "protocol_ref", "independence",
    "received_window", "cutoff_date", "seeded_control_outcome",
    "planted_omission_outcome",
]


def validate_review_events(src: dict):
    """The stopping rule's named review event. independence: true is the R7
    admissibility test: named reviewer identities, a resolving protocol, and
    passed controls — the in-repo reviewer corpus can never supply these, and
    it is never admissible independent-review evidence for closure condition
    five, by doctrine and by this computation."""
    for i, rec in enumerate(src.get("review_events", [])):
        ctx = f"review_events[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, REVIEW_EVENT_KEYS, ctx)
        for key in ("title", "received_window", "cutoff_date",
                    "seeded_control_outcome", "planted_omission_outcome"):
            require_str(rec, key, ctx)
        if not isinstance(rec["independence"], bool):
            raise LedgerError(f"{ctx}: independence must be a boolean")
        reviewers = rec["reviewers"]
        if not isinstance(reviewers, list):
            raise LedgerError(f"{ctx}: reviewers must be a list")
        for j, rv in enumerate(reviewers):
            exact_keys(rv, ["identity", "discipline"], f"{ctx}.reviewers[{j}]")
            require_str(rv, "identity", f"{ctx}.reviewers[{j}]")
            require_str(rv, "discipline", f"{ctx}.reviewers[{j}]")
        if rec["independence"]:
            if not reviewers:
                raise LedgerError(
                    f"{ctx}: an independent event requires named reviewer "
                    "identities"
                )
            validate_reference(rec["protocol_ref"], f"{ctx}.protocol_ref")
            for key in ("seeded_control_outcome", "planted_omission_outcome"):
                if not rec[key].startswith("passed"):
                    raise LedgerError(
                        f"{ctx}: an independent event requires its {key} to "
                        "have passed — a review that misses the plant or "
                        "whose triage passes the seeds fails its control"
                    )
        else:
            require_str(rec, "protocol_ref", ctx)


PROPOSAL_KEYS = [
    "id", "title", "proposal", "source", "received_date", "triaged_date",
    "materiality_finding", "materiality_reason", "proposal_disposition",
    "reasons", "review_event_ref",
]
PROPOSAL_OPTIONAL = [
    "severity", "severity_owner", "independent_check", "created_record_refs",
    "routed_unestablished_disposition", "defect_row_ref",
]


def validate_proposals(src: dict, ids: dict):
    events_by_id = {r["id"]: r for r in src.get("review_events", [])}
    defect_ids = {r["id"] for r in src.get("defects", [])}
    for i, rec in enumerate(src.get("proposals", [])):
        ctx = f"proposals[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, PROPOSAL_KEYS, ctx, optional=PROPOSAL_OPTIONAL)
        for key in ("title", "proposal", "source", "received_date",
                    "triaged_date", "materiality_reason", "reasons"):
            require_str(rec, key, ctx)
        finding = rec["materiality_finding"]
        if finding not in ("material", "immaterial"):
            raise LedgerError(
                f"{ctx}: materiality_finding must be material or immaterial, "
                "judged under the stopping rule's materiality test"
            )
        disposition = rec["proposal_disposition"]
        if disposition not in PROPOSAL_DISPOSITIONS:
            raise LedgerError(f"{ctx}: unknown proposal_disposition")
        event = events_by_id.get(rec["review_event_ref"])
        if event is None:
            raise LedgerError(f"{ctx}: review_event_ref names no review event")
        if finding == "material":
            sev = rec.get("severity")
            if sev not in ("critical", "material"):
                raise LedgerError(
                    f"{ctx}: a material proposal carries a severity class of "
                    "critical or material under the candidate rubric"
                )
            validate_reference(rec.get("severity_owner", ""),
                              f"{ctx}.severity_owner")
            check = rec.get("independent_check")
            if not isinstance(check, str) or not check:
                raise LedgerError(
                    f"{ctx}: a material proposal requires its independent check"
                )
            if event["independence"]:
                validate_reference(check, f"{ctx}.independent_check")
            elif not check.startswith("none-recorded — "):
                raise LedgerError(
                    f"{ctx}: on a non-independent event the check is recorded "
                    "as absent with its reason, never fabricated"
                )
        else:
            for key in ("severity", "severity_owner", "independent_check"):
                if key in rec:
                    raise LedgerError(
                        f"{ctx}: {key} belongs only on material proposals — "
                        "minor is the editorial band inside the rubric prose, "
                        "not a materiality escape hatch"
                    )
            if disposition != "classified-out":
                raise LedgerError(
                    f"{ctx}: an immaterial proposal is classified out with "
                    "reasons"
                )
        if disposition == "added":
            refs = rec.get("created_record_refs")
            if not isinstance(refs, list) or not refs:
                raise LedgerError(
                    f"{ctx}: added means records were created — name them"
                )
            for j, ref in enumerate(refs):
                if ref in ids:
                    continue
                validate_reference(ref, f"{ctx}.created_record_refs[{j}]")
        elif "created_record_refs" in rec:
            raise LedgerError(
                f"{ctx}: created_record_refs belongs only on added proposals"
            )
        if disposition == "classified-out":
            routed = rec.get("routed_unestablished_disposition")
            low = rec["reasons"].lower()
            if routed is not None:
                if routed not in UNESTABLISHED_DISPOSITIONS:
                    raise LedgerError(
                        f"{ctx}: unknown routed unestablished disposition"
                    )
            elif "duplicate" not in low and "immaterial" not in low:
                raise LedgerError(
                    f"{ctx}: a classification that routes the item outward "
                    "carries the matching Unestablished disposition; only a "
                    "duplicate or immaterial classification carries none"
                )
        elif "routed_unestablished_disposition" in rec:
            raise LedgerError(
                f"{ctx}: routing dispositions belong only on classified-out "
                "proposals"
            )
        if disposition == "retained-limit":
            dref = rec.get("defect_row_ref")
            if dref not in defect_ids:
                raise LedgerError(
                    f"{ctx}: retention creates or joins a stable defect row — "
                    "the retained limit must link one"
                )
        elif "defect_row_ref" in rec:
            raise LedgerError(
                f"{ctx}: defect_row_ref belongs only on retained-limit "
                "proposals"
            )


def validate_severity_rubric(src: dict):
    rub = src.get("severity_rubric")
    if not isinstance(rub, dict):
        raise LedgerError("severity_rubric must be an object")
    exact_keys(rub, ["critical", "material", "minor", "materiality_ref",
                     "rubric_status"], "severity_rubric",
               optional=["confirmation_basis"])
    for key in ("critical", "material", "minor"):
        require_str(rub, key, "severity_rubric")
    if rub["materiality_ref"] != "stopping_rule.materiality_test":
        raise LedgerError(
            "severity_rubric.materiality_ref must bind the ratified "
            "materiality test by reference, never a paraphrase"
        )
    status = rub["rubric_status"]
    if status == RUBRIC_STATUS_CANDIDATE:
        if "confirmation_basis" in rub:
            raise LedgerError(
                "severity_rubric: a candidate rubric carries no confirmation "
                "basis"
            )
    elif status == RUBRIC_STATUS_CONFIRMED:
        require_str(rub, "confirmation_basis", "severity_rubric")
    else:
        raise LedgerError(
            f"severity_rubric.rubric_status must be exactly "
            f"{RUBRIC_STATUS_CANDIDATE!r} or {RUBRIC_STATUS_CONFIRMED!r} — "
            "confirmation is a recorded author act, never a rewording"
        )


def compute_gate_a_readiness(src: dict, resolution: dict):
    """The single computation the render and the closure validator both
    consume. Statuses echo the stopping rule's closure conditions by index so
    the two texts can never drift. No aggregate is ever derived from this."""
    conds = src["stopping_rule"]["closure_conditions"]
    deferred = sorted(d["record_type"] for d in src["deferred_populations"])
    blocking = sorted(rid for rid, r in resolution.items() if r["blocking"])
    independent = [e for e in src.get("review_events", [])
                   if e.get("independence") is True]
    rows = []
    if deferred:
        rows.append((conds[0], "unmet",
                     "record types remain deferred with owners: "
                     + ", ".join(deferred)))
    else:
        rows.append((conds[0], "met-in-form",
                     "every record type is populated or classified out; "
                     "material sufficiency stays a review question"))
    rows.append((conds[1], "form-only",
                 "the projections that exist regenerate — the check itself is "
                 "the proof — while the role matrix, dependency map, assurance "
                 "allocation, and reader ledger do not exist yet"))
    if blocking:
        rows.append((conds[2], "unmet",
                     "blocking defect rows exist: " + ", ".join(blocking)))
    else:
        rows.append((conds[2], "met-mechanically",
                     "no critical unresolved defect row blocks a claim"))
    rows.append((conds[3], "met-in-form",
                 "severity, consequence, owner, closure condition, and "
                 "public-claim limitation are validator-enforced on every "
                 "unresolved object; substance is reviewed, not proven"))
    if independent:
        rows.append((conds[4], "met-in-form",
                     "an independent review event exists; its proposals carry "
                     "their dispositions"))
    else:
        rows.append((conds[4], "unmet-external",
                     "no review event with independence true exists, and the "
                     "in-repo reviewer corpus is never admissible for this "
                     "condition"))
    preconditions = []
    if src["envelope"][0]["id"] == ENVELOPE_STUB_ID:
        preconditions.append(("the reference envelope", "unmet-external",
                              "still the explicit stub; the envelope item owns "
                              "its versioning"))
    if src["severity_rubric"]["rubric_status"] == RUBRIC_STATUS_CANDIDATE:
        preconditions.append(("the severity rubric", "unmet",
                              "candidate — author confirmation pending"))
    return rows, preconditions


def validate_closure_record(src: dict, readiness):
    if "closure_record" not in src:
        raise LedgerError(
            "closure_record must be present — null until the author closes"
        )
    rec = src["closure_record"]
    if rec is None:
        return
    ctx = "closure_record"
    exact_keys(
        rec,
        ["gate", "permitted_claim", "source_version", "envelope_ref",
         "candidate_ids", "review_cutoff", "review_event_ref",
         "assurance_record_refs", "residual_refs", "claim_limitations",
         "author_ratification_ref"],
        ctx,
    )
    if rec["envelope_ref"] == ENVELOPE_STUB_ID:
        raise LedgerError(
            f"{ctx}: a closure record may not cite the envelope stub — the "
            "stub can route, never assure"
        )
    events_by_id = {r["id"]: r for r in src.get("review_events", [])}
    event = events_by_id.get(rec["review_event_ref"])
    if event is None or event.get("independence") is not True:
        raise LedgerError(
            f"{ctx}: the closure record must name an independent review event"
        )
    rows, preconditions = readiness
    for name, status, reason in list(rows) + list(preconditions):
        if status not in READINESS_MET:
            raise LedgerError(
                f"{ctx}: a closure record may not exist while a closure "
                f"condition computes unmet — {name}: {reason}"
            )
    validate_reference(rec["author_ratification_ref"],
                      f"{ctx}.author_ratification_ref")
    if src["acceptance_gate"]["gate_a_status"] != "passed":
        raise LedgerError(
            f"{ctx}: a closure record requires gate_a_status passed, which "
            "this contract refuses — closing Gate A is a deliberate future "
            "amendment, never a latent flip"
        )


def validate_acceptance(src: dict):
    gate = src.get("acceptance_gate")
    if not isinstance(gate, dict):
        raise LedgerError("acceptance_gate must be an object")
    exact_keys(gate, ["verdict", "rollup_rule", "gate_a_status"],
               "acceptance_gate")
    if gate["verdict"] != VERDICT_LINE:
        raise LedgerError(
            "acceptance_gate.verdict must be the byte-exact verdict line"
        )
    require_str(gate, "rollup_rule", "acceptance_gate")
    if re.search(r"\d", gate["rollup_rule"]):
        raise LedgerError(
            "the rollup is non-numeric by rule — no digit may appear in it"
        )
    if gate["gate_a_status"] != "not-passed":
        raise LedgerError("gate_a_status must be not-passed until Gate A closes")


def validate(src: dict):
    validate_header(src)
    validate_bound_sources(src)
    validate_meanings(src)
    validate_axes(src)
    validate_compatibility(src)
    check_no_generic_disposition(src, "source")
    check_no_score_fields(src, "source")
    ids = validate_id_registry(src)
    routes_by_id = validate_routes(src)
    validate_domains(src, ids)
    validate_legacy_rows(src, ids)
    validate_claims(src, ids, routes_by_id)
    validate_bodies(src)
    validate_external_assumptions(src)
    validate_envelope(src)
    validate_defect_rows(src, ids)
    resolution = compute_resolution(src)
    validate_receipts(src, ids, resolution)
    validate_residual_coverage(src)
    validate_review_events(src)
    validate_proposals(src, ids)
    validate_severity_rubric(src)
    validate_deferred(src)
    validate_enum_mapping(src)
    validate_stopping_rule(src)
    validate_acceptance(src)
    readiness = compute_gate_a_readiness(src, resolution)
    validate_closure_record(src, readiness)
    validate_coverage_region(src)
    return resolution


# ── negative controls: sabotage first, trust after ───────────────────────────

def negative_controls(src: dict) -> int:
    """Each mutation must be rejected; a check never watched failing is not a
    check."""
    controls = []

    def control(name, mutate, expect=None):
        controls.append((name, mutate, expect))

    def first_claim(s):
        return s["claims"][0]

    control("generic disposition key is refused",
            lambda s: first_claim(s).update({"disposition": "open"}))
    control("numeric rollup is an aggregate score",
            lambda s: s["acceptance_gate"].update(
                {"rollup_rule": "9 of 10 rows established"}))
    control("score key is refused",
            lambda s: s["domains"][0].update({"score": "high"}))
    control("partial formalisation must be split",
            lambda s: first_claim(s).update({"status": "partial formalisation"}))
    control("a claim needs a recognised posture",
            lambda s: first_claim(s).update({"posture": "Probable"}))
    control("two postures is two records",
            lambda s: first_claim(s).update({"posture": "Derived; Specified"}))
    control("Derived requires executable evidence kind", _derived_bad_kind)
    control("liveness may not be Derived", _liveness_derived)
    control("feasibility may not appear at all", _feasibility_claim)
    control("established posture needs a built or available route",
            _established_on_unbuilt)
    control("a built route must keep its negative control",
            _route_without_control)
    control("deleted enum-mapping row fails closure",
            lambda s: s["enum_mapping"].pop(0))
    control("remedied + detected is invalid",
            _remedied_detected)
    control("hand-authored resolution_status is refused",
            lambda s: first_claim(s).update(
                {"resolution_status": "resolved-for-claim"}))
    control("empty array without deferral is refused", _drop_empty_deferral)
    control("the envelope stub can route, never assure",
            _stub_operationally_assured)
    control("a stale bound-source digest is caught",
            lambda s: s["bound_sources_sha256"].update(
                {"assurance_portfolio": "0" * 64}))
    control("a broken needle is caught",
            lambda s: s["domains"][0]["source_refs"].__setitem__(
                0, "TODO.md::negative-control-anchor-does-not-exist"))
    control("a Specified row needs its unimplemented marker",
            _specified_without_marker)
    control("an Unestablished row needs a named disposition",
            _unestablished_without_disposition)
    control("a domain layer must be the sentinel",
            lambda s: s["domains"][0].update({"layer": "constitutional-invariant"}))
    control("verdict line is byte-exact",
            lambda s: s["acceptance_gate"].update(
                {"verdict": VERDICT_LINE.lower()}))
    control("a receipt must bind a candidate row", _receipt_noncandidate,
            "non-candidate")
    control("elimination keeps its reintroduction control",
            _elimination_without_control)
    control("remedied cannot sit below its required stage",
            _remedied_wrong_stage)
    control("a receipt needs its reader-facing mapping",
            _receipt_without_reader_mapping)
    control("one keyed row per defect tuple", _duplicate_keying_tuple)
    control("hand-authored blocking is refused",
            lambda s: s["defects"][0].update({"blocking": False}))
    control("a receipt must name an existing defect row",
            lambda s: s["receipts"][0].update({"defect_row_ref": "FS-DFT-777"}))
    control("an unknown residual citation is stale",
            lambda s: s["defects"][0]["residual_citations"].append(
                "bogus-file#nope"))
    control("an uncovered sibling residual fails closure", _uncover_residual,
            "uncovered")
    control("a stale exclusion is refused",
            lambda s: s["residual_coverage_exclusions"].append(
                {"source_file": "x", "token": "bogus#token",
                 "reason": "control"}))
    control("a defect must affect a claim record",
            lambda s: s["defects"][0].update(
                {"affected_claim_ref": s["bodies"][0]["id"]}))
    control("a defect layer is never the domain sentinel",
            lambda s: s["defects"][0].update({"layer": DOMAIN_LAYER_SENTINEL}))
    control("the generated region may not duplicate a coverage-map needle",
            lambda s: s["legacy_rows"][0].update(
                {"legacy_gap": s["legacy_rows"][0]["legacy_gap"] +
                 " ## 3. Current coverage versus target scope"}),
            "exactly once")
    control("a closure record cannot exist while conditions compute unmet",
            _closure_while_unmet, "may not exist while")
    control("a closure record may not cite the envelope stub",
            _closure_env_stub, "never assure")
    control("a closure record requires an independent review event",
            _closure_non_independent_event, "independent review event")
    control("independence requires named reviewers and passed controls",
            _event_fake_independence, "independent event requires")
    control("an added proposal names resolvable created records",
            _proposal_added_unresolvable)
    control("a retained limit links its defect row",
            _proposal_retained_without_defect)
    control("outward classification carries its routing disposition",
            _proposal_routed_without_disposition)
    control("a material proposal carries severity, owner, and check",
            _proposal_material_missing_fields)
    control("severity belongs only on material proposals",
            _proposal_class_on_immaterial)
    control("a proposal must name its review event",
            _proposal_unknown_event)
    control("the rubric status is exact in both states",
            lambda s: s["severity_rubric"].update(
                {"rubric_status": "confirmed"}))
    control("a confirmed rubric records its basis",
            _confirmed_rubric_without_basis)

    passed = 0
    for entry in controls:
        name, mutate = entry[0], entry[1]
        expect = entry[2] if len(entry) > 2 else None
        mutant = copy.deepcopy(src)
        try:
            mutate(mutant)
            validate(mutant)
        except LedgerError as exc:
            if expect is not None and expect not in str(exc):
                raise LedgerError(
                    f"negative control failed for the wrong reason: {name} — "
                    f"expected {expect!r} in {exc}"
                )
            passed += 1
            continue
        raise LedgerError(f"negative control failed to fail: {name}")
    return passed


def _derived_bad_kind(s):
    for rec in s["claims"]:
        if rec["posture"] == "Derived":
            rec["evidence_kind"] = "inventory"
            return
    raise LedgerError("control setup: no Derived claim to mutate")


def _liveness_derived(s):
    for rec in s["claims"]:
        if rec["posture"] == "Derived":
            rec["overlay"] = "liveness"
            return
    raise LedgerError("control setup: no Derived claim to mutate")


def _feasibility_claim(s):
    rec = s["claims"][0]
    rec["overlay"] = "feasibility"


def _established_on_unbuilt(s):
    unbuilt = next(r["id"] for r in s["routes"]
                   if r["route_status"] == "unbuilt")
    for rec in s["claims"]:
        if rec["posture"] == "Derived":
            rec["route_ref"] = unbuilt
            return
    raise LedgerError("control setup: no Derived claim to mutate")


def _route_without_control(s):
    for rec in s["routes"]:
        if rec["route_status"] == "built":
            rec["negative_control"] = "not-yet-declared"
            return
    raise LedgerError("control setup: no built route to mutate")


def _specified_without_marker(s):
    for rec in s["claims"]:
        if rec["posture"] == "Specified":
            rec.pop("unimplemented_marker", None)
            return
    raise LedgerError("control setup: no Specified claim to mutate")


def _unestablished_without_disposition(s):
    for rec in s["claims"]:
        if rec["posture"] == "Unestablished":
            rec.pop("unestablished_disposition", None)
            return
    raise LedgerError("control setup: no Unestablished claim to mutate")


def _remedied_detected(s):
    for row in s["compatibility_table"]:
        if row["defect_disposition"] == "remedied":
            row["allowed_response_stages"] = ["detected"]
            return


def _stub_operationally_assured(s):
    # Minimally valid under the stage-2 rules except the envelope violation,
    # which the validator checks before the control requirements — the expect
    # string on this control pins the failure to the intended rule.
    s["defects"].append({
        "id": "FS-DFT-999", "defect_id": "FS-DFT-999", "title": "control",
        "applicability": "control", "layer": "constitutional-invariant",
        "status": "control", "severity": "material — control",
        "consequence": "control",
        "owner_ref": s["domains"][0]["source_refs"][0],
        "closure_condition": "control",
        "defect_disposition": "remedied",
        "response_stage": "operationally-assured-in-envelope",
        "affected_claim_ref": s["claims"][0]["id"],
        "consequence_id": "control", "scope_id": "control",
        "envelope_id": ENVELOPE_STUB_ID, "source_version": "control",
        "history": [], "evidence_notes": [], "residual_citations": [],
        "controls": {},
    })


def _drop_empty_deferral(s):
    for i, entry in enumerate(s["deferred_populations"]):
        if not s.get(entry["record_type"], []):
            s["deferred_populations"].pop(i)
            return
    raise LedgerError("control setup: no empty-array deferral to drop")


def _receipt_noncandidate(s):
    # Retarget the first receipt at an open-defect row with no family siblings,
    # keeping every cross-field equality intact so the candidate rule decides.
    target = next(
        d for d in s["defects"]
        if d["defect_disposition"] == "open-defect"
        and sum(1 for x in s["defects"] if x["defect_id"] == d["defect_id"]) == 1
    )
    claim = next(c for c in s["claims"] if c["id"] == target["affected_claim_ref"])
    rcp = s["receipts"][0]
    rcp["defect_row_ref"] = target["id"]
    for key in ("defect_id", "affected_claim_ref", "consequence_id",
                "defect_disposition", "response_stage", "scope_id",
                "source_version", "envelope_id"):
        rcp[key] = target[key]
    rcp["claim_posture"] = claim["posture"]
    rcp["route_ref"] = claim["route_ref"]
    rcp["assurance_ceiling"] = claim["posture"]
    rcp["residuals"] = ["none beyond the affected claim's own scope bound"]


def _elimination_without_control(s):
    for rec in s["defects"]:
        if rec["defect_disposition"] == "eliminated-structurally":
            rec["controls"] = {}
            return
    raise LedgerError("control setup: no eliminated row to mutate")


def _remedied_wrong_stage(s):
    for rec in s["defects"]:
        if rec["defect_disposition"] == "eliminated-structurally":
            rec["defect_disposition"] = "remedied"
            return
    raise LedgerError("control setup: no eliminated row to mutate")


def _receipt_without_reader_mapping(s):
    del s["receipts"][0]["reader_mapping_ref"]


def _duplicate_keying_tuple(s):
    twin = copy.deepcopy(s["defects"][0])
    twin["id"] = "FS-DFT-998"
    s["defects"].append(twin)


_CONTROL_NEEDLE = ("new-book-plans/full-society-boundary-decision.md::"
                   "## 4. Versioned closure")


def _strip_deferrals(s, *types):
    s["deferred_populations"] = [
        d for d in s["deferred_populations"] if d["record_type"] not in types
    ]


def _mk_event(s, independent=False):
    _strip_deferrals(s, "review_events")
    ev = {
        "id": "FS-REV-99", "title": "control fixture",
        "reviewers": [], "protocol_ref": "none — control fixture",
        "independence": independent, "received_window": "control",
        "cutoff_date": "control", "seeded_control_outcome": "not-run",
        "planted_omission_outcome": "not-run",
    }
    if independent:
        ev["reviewers"] = [{"identity": "control", "discipline": "control"}]
        ev["protocol_ref"] = _CONTROL_NEEDLE
        ev["seeded_control_outcome"] = "passed — control"
        ev["planted_omission_outcome"] = "passed — control"
    s["review_events"].append(ev)
    return ev


def _mk_proposal(s, **overrides):
    _mk_event(s)
    _strip_deferrals(s, "proposals")
    rec = {
        "id": "FS-PRO-99", "title": "control fixture", "proposal": "control",
        "source": "control", "received_date": "control",
        "triaged_date": "control", "materiality_finding": "immaterial",
        "materiality_reason": "control", "proposal_disposition":
        "classified-out", "reasons": "immaterial control fixture",
        "review_event_ref": "FS-REV-99",
    }
    rec.update(overrides)
    s["proposals"].append(rec)
    return rec


def _material_fields():
    return {
        "materiality_finding": "material", "severity": "material",
        "severity_owner": _CONTROL_NEEDLE,
        "independent_check": "none-recorded — control fixture on a "
                             "non-independent event",
        "reasons": "control",
    }


def _mk_closure(s):
    _mk_event(s, independent=True)
    s["closure_record"] = {
        "gate": "gate-a", "permitted_claim": "control",
        "source_version": s["source_version"], "envelope_ref": "FS-ENV-01",
        "candidate_ids": [], "review_cutoff": "control",
        "review_event_ref": "FS-REV-99", "assurance_record_refs": [],
        "residual_refs": [], "claim_limitations": "control",
        "author_ratification_ref": _CONTROL_NEEDLE,
    }
    return s["closure_record"]


def _closure_while_unmet(s):
    _mk_closure(s)


def _closure_env_stub(s):
    _mk_closure(s)["envelope_ref"] = ENVELOPE_STUB_ID


def _closure_non_independent_event(s):
    _mk_closure(s)
    s["review_events"][-1]["independence"] = False
    s["review_events"][-1]["protocol_ref"] = "none — control"


def _event_fake_independence(s):
    ev = _mk_event(s, independent=True)
    ev["reviewers"] = []


def _proposal_added_unresolvable(s):
    _mk_proposal(s, proposal_disposition="added",
                 created_record_refs=["bogus-file.md::no such anchor"],
                 **_material_fields())


def _proposal_retained_without_defect(s):
    _mk_proposal(s, proposal_disposition="retained-limit",
                 **_material_fields())


def _proposal_routed_without_disposition(s):
    _mk_proposal(s, reasons="routed to operations outside this volume")


def _proposal_material_missing_fields(s):
    _mk_proposal(s, materiality_finding="material", reasons="control")


def _proposal_class_on_immaterial(s):
    _mk_proposal(s, severity="minor")


def _proposal_unknown_event(s):
    _mk_proposal(s, review_event_ref="FS-REV-00")


def _confirmed_rubric_without_basis(s):
    s["severity_rubric"]["rubric_status"] = RUBRIC_STATUS_CONFIRMED
    s["severity_rubric"].pop("confirmation_basis", None)


def _uncover_residual(s):
    counts = {}
    for rec in s["defects"]:
        for token in rec["residual_citations"]:
            counts[token] = counts.get(token, 0) + 1
    for rec in s["defects"]:
        for token in list(rec["residual_citations"]):
            if counts.get(token) == 1:
                rec["residual_citations"].remove(token)
                return
    raise LedgerError("control setup: no singly-cited residual token")


# ── rendering ────────────────────────────────────────────────────────────────

def _bucket_cell(bucket):
    if "answer" in bucket:
        return bucket["answer"]
    if "routing_marker" in bucket:
        return f"*{bucket['routing_marker']}* — {bucket['note']}"
    u = bucket["unresolved"]
    return (f"**Unresolved** — severity: {u['severity']}; consequence: "
            f"{u['consequence']}; closure: {u['closure_condition']}; "
            f"public-claim limitation: {u['public_claim_limitation']}")


def render(src: dict, resolution: dict) -> str:
    out = []
    w = out.append
    blocked_by = {}
    for rec in src.get("defects", []):
        if resolution[rec["id"]]["blocking"]:
            blocked_by.setdefault(rec["affected_claim_ref"], []).append(rec["id"])
    w("<!-- SPDX-License-Identifier: CC-BY-4.0 -->")
    w("<!-- Generated by new-book-plans/13-full-society-ledger.py; do not edit. -->")
    w("")
    w("# Full-Society Domain-and-Layer Ledger — Generated Report")
    w("")
    w(f"**{src['acceptance_gate']['verdict']}**")
    w("")
    w(f"Reviewed source: `new-book-plans/full-society-ledger.json` "
      f"(source version `{src['source_version']}`; {STAGE_LABEL}). This report "
      "is a projection of the canonical source. Classification is routing, not "
      "assurance: every row establishes at most its own posture, and no count "
      "here is an assurance figure.")
    w("")
    w("## Declared axes and stopping rule")
    w("")
    for axis in src["axes"]:
        w(f"- **{axis['name']}** (`{axis['id']}`): {axis['values']} — "
          f"{axis['note']}")
    w("")
    rule = src["stopping_rule"]
    w("A version may close only when all of the following hold for the gate's "
      "permitted claim:")
    w("")
    for i, cond in enumerate(rule["closure_conditions"], 1):
        w(f"{i}. {cond}")
    w("")
    w(f"**Materiality:** {rule['materiality_test']}")
    w("")
    w(f"**Boundary:** {rule['boundary']}")
    w("")
    w(f"**No hiding:** {rule['no_hiding_rule']}")
    w("")
    w("## Gate A readiness (computed)")
    w("")
    w("Per-condition status, generated from the source and echoing the closure "
      "conditions above by index; no aggregate is derived from this list, and "
      "a closure record is refused while any row computes unmet. Closing "
      "Gate A is a deliberate future amendment with its own author "
      "ratification, never a latent flip.")
    w("")
    rows, preconditions = compute_gate_a_readiness(src, resolution)
    for name, status, reason in rows:
        w(f"- **{status}** — {name}: {reason}")
    for name, status, reason in preconditions:
        w(f"- **{status}** (precondition) — {name}: {reason}")
    w("")
    w("## The five layers")
    w("")
    w("The five routing dispositions are the reader-facing five layers "
      "(author-ratified 2026-08-09). One enum, one key: `scope_disposition`; "
      "a leaf record carries exactly one value and a domain record spans all "
      "five as its buckets.")
    w("")
    w("| Layer | Meaning |")
    w("| --- | --- |")
    for value in SCOPE_DISPOSITIONS:
        w(f"| `{value}` | {src['scope_disposition_meanings'][value]} |")
    w("")
    w("## Domains")
    w("")
    for rec in src["domains"]:
        w(f"### {rec['id']} — {rec['title']}")
        w("")
        w(f"- Status: {rec['status']}; applicability: {rec['applicability']}; "
          f"classes: {', '.join(rec['class_refs'])}")
        w(f"- Constitutional invariants: {_bucket_cell(rec['constitutional_invariants'])}")
        w(f"- Democratic / ordinary-law choices: {_bucket_cell(rec['ordinary_law_choices'])}")
        w(f"- Protected private/civic freedom: {_bucket_cell(rec['protected_private_civic'])}")
        w(f"- Book 2 operations: {_bucket_cell(rec['book2_operations'])}")
        w(f"- External assumptions: {_bucket_cell(rec['external_assumptions_note'])}")
        w(f"- Bodies: {', '.join(rec['bodies_refs']) if rec['bodies_refs'] else 'none named yet'}; "
          f"legacy rows: {', '.join(rec['legacy_row_refs']) if rec['legacy_row_refs'] else 'none'}")
        sa = rec["scenario_applicability"]
        w(f"- Scenario applicability: "
          f"{sa.get('answer', 'deferred — ' + sa.get('deferred_ref', ''))}")
        w(f"- Reader destination: {rec['reader_destination']}")
        w(f"- Severity if left open: {rec['severity']}; consequence: "
          f"{rec['consequence']}; closure: {rec['closure_condition']}")
        w("")
    w("## Legacy coverage rows and their splits")
    w("")
    w("Imported from the coverage map with wording frozen; each split claim "
      "carries exactly one posture per the ratified legend.")
    w("")
    w("| Row | Legacy status (frozen) | Split state | Claims |")
    w("| --- | --- | --- | --- |")
    for rec in src["legacy_rows"]:
        claims = ", ".join(rec["split_claim_refs"]) or "—"
        w(f"| {rec['id']} {rec['domain_title']} | {rec['legacy_status']} | "
          f"{rec['split_state']} | {claims} |")
    w("")
    w("## Claims (one posture each)")
    w("")
    w("| Claim | Layer | Posture | Route | Overlay | Scope bound | Blocked by |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["claims"]:
        posture = rec["posture"]
        if posture == "Unestablished":
            posture += f"/{rec['unestablished_disposition']}"
        if rec.get("evidence_kind"):
            posture += f" ({rec['evidence_kind']})"
        blockers = ", ".join(blocked_by.get(rec["id"], [])) or "—"
        w(f"| {rec['id']} {rec['title']} | {rec['layer']} | {posture} | "
          f"{rec['route_ref']} | {rec['overlay']} | {rec['scope_bound']} | "
          f"{blockers} |")
    w("")
    w("## Required bodies")
    w("")
    w("| Body | Constitutional job | May not do alone | Required check / remedy |")
    w("| --- | --- | --- | --- |")
    for rec in src["bodies"]:
        w(f"| {rec['id']} {rec['title']} | {rec['job']} | "
          f"{rec['may_not_do_alone']} | {rec['required_check']} |")
    w("")
    w("## Assurance routes")
    w("")
    w("| Route | Status | Warrants | Cannot warrant | Falsification | Negative control |")
    w("| --- | --- | --- | --- | --- | --- |")
    for rec in src["routes"]:
        w(f"| {rec['id']} {rec['title']} | {rec['route_status']} | "
          f"{rec['warrants']} | {rec['cannot_warrant']} | "
          f"{rec['falsification_condition']} | {rec['negative_control']} |")
    w("")
    w("## Enum mapping (maps, renames nothing)")
    w("")
    w("| Source | Field | Value | Canonical |")
    w("| --- | --- | --- | --- |")
    for row in src["enum_mapping"]:
        w(f"| {row['source_file']} | {row['field']} | `{row['value']}` | "
          f"{row['canonical']} |")
    w("")
    w("Deliberate exclusions (recorded, not silent):")
    w("")
    for row in src["enum_mapping_exclusions"]:
        w(f"- `{row['source_file']}` `{row['field']}` = `{row['value']}`: "
          f"{row['reason']}")
    w("")
    w("## Defect-disposition compatibility")
    w("")
    w("| Disposition | Allowed stages | Resolution-eligible | Requirement |")
    w("| --- | --- | --- | --- |")
    for row in src["compatibility_table"]:
        w(f"| `{row['defect_disposition']}` | "
          f"{', '.join(row['allowed_response_stages'])} | "
          f"{'yes' if row['resolution_eligible'] else 'no'} | "
          f"{row['resolution_requirement']} |")
    w("")
    w("## Defect rows (disposition and stage; resolution generated)")
    w("")
    w("The `:defect` markers in the pin files remain the complete list of "
      "book-declared, chapter-load-bearing flaws with flip tripwires; these "
      "rows are the wider engineering inventory, and they cite the markers "
      "where one exists. A resolved row resolves only its named consequence in "
      "its exact scope; resolution is claim-relative and asserts nothing "
      "beyond the affected claim's own posture. Rows are keyed by defect "
      "family, affected claim, consequence, scope, envelope, and source "
      "version; a residual sibling shares its family's defect_id.")
    w("")
    w("| Row | Family | Title | Affected claim | Disposition | Stage | "
      "Severity | Resolution (generated) | Blocking |")
    w("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for rec in src.get("defects", []):
        res = resolution[rec["id"]]
        w(f"| {rec['id']} | {rec['defect_id']} | {rec['title']} | "
          f"{rec['affected_claim_ref']} | {rec['defect_disposition']} | "
          f"{rec['response_stage']} | {severity_class(rec)} | "
          f"{res['resolution_status']} | "
          f"{'yes' if res['blocking'] else 'no'} |")
    w("")
    w("Residual citations bind every sibling residual pool to these rows "
      "under the live-read closure; narrowness-impact rows never enter the "
      "pool — they are claim-impact anchors, not defects.")
    w("")
    w("## Resolution receipts")
    w("")
    w(f"Every receipt records its eligible gate beside the ledger's standing "
      f"gate status ({src['acceptance_gate']['gate_a_status']}): a recorded "
      "gate is a binding, not a passage. A receipt exists only where the "
      "generated resolution permits one, and it never implies a narrower "
      "repair cured a wider defect.")
    w("")
    for rec in src.get("receipts", []):
        w(f"### {rec['id']} — {rec['title']}")
        w("")
        w(f"- Defect row: {rec['defect_row_ref']} (family {rec['defect_id']}); "
          f"claim {rec['affected_claim_ref']} at posture "
          f"{rec['claim_posture']} via {rec['route_ref']}; assurance ceiling "
          f"{rec['assurance_ceiling']}; eligible gate {rec['eligible_gate']} "
          f"(gate status: {src['acceptance_gate']['gate_a_status']})")
        w(f"- What failed: {rec['what_failed']}")
        w(f"- Hostile witness: {rec['hostile_witness']}")
        w(f"- Why it failed: {rec['why_it_failed']}")
        w(f"- The response: {rec['response_change']}")
        w(f"- What now follows: {rec['now_follows']}")
        w(f"- Proof: `{rec['proof_ref']}`; negative control: "
          f"`{rec['negative_control_ref']}`")
        w(f"- What still does not follow: {rec['still_does_not_follow']}")
        w(f"- Residuals: {'; '.join(rec['residuals'])}")
        w(f"- Reader mapping: `{rec['reader_mapping_ref']}`; admissible "
          f"evidence: {rec['admissible_evidence']}")
        w("")
    w("## Proposals and review events")
    w("")
    w("The review machinery stands ready and empty: proposal and review-event "
      "records are schema-enforced, and both populations stay deferred with "
      "owners until the independent scope review runs. The severity rubric's "
      f"status is {src['severity_rubric']['rubric_status']}, bound to the "
      "stopping rule's materiality test by reference. The in-repo reviewer "
      "corpus is never admissible independent-review evidence for the "
      "review-condition of closure: an independent event requires named "
      "reviewer identities, a resolving protocol, and passed seeded and "
      "planted-omission controls, none of which that corpus can supply. A "
      "reviewer compels a reasoned public disposition, not acceptance and not "
      "a veto; the severity owner and the closure record are author "
      "checkpoints.")
    w("")
    w("| Rubric class | Meaning |")
    w("| --- | --- |")
    for cls in ("critical", "material", "minor"):
        w(f"| {cls} | {src['severity_rubric'][cls]} |")
    w("")
    w("## External assumptions and the envelope")
    w("")
    for rec in src["external_assumptions"]:
        w(f"- **{rec['id']} {rec['title']}**: {rec['assumption']} Failure "
          f"consequence: {rec['failure_consequence']}")
    w("")
    stub = src["envelope"][0]
    w(f"**Envelope:** `{stub['id']}` is an explicit stub "
      f"({stub['status']}). {stub['note']}")
    w("")
    w("## Book 2 crosswalk (routed rows only)")
    w("")
    w("A collection-only projection: Book 2 remains inactive until Book 1 — "
      "First Edition actually ships, and this view carries routing and closure "
      "fields only. No operating owner, workforce, facility, capacity, service, "
      "or cost field appears here; those belong to Book 2's own responsibility "
      "view when it activates, generated from this same canonical source.")
    w("")
    w("| ID | Title | Routed as | Severity | Consequence | Closure condition |")
    w("| --- | --- | --- | --- | --- | --- |")
    for rec in src["claims"]:
        if rec["layer"] == "book-2-operation" or \
                rec.get("unestablished_disposition") == "routed-book-2":
            routed = rec["layer"]
            if rec.get("unestablished_disposition"):
                routed += f" ({rec['unestablished_disposition']})"
            w(f"| {rec['id']} | {rec['title']} | {routed} | {rec['severity']} | "
              f"{rec['consequence']} | {rec['closure_condition']} |")
    for rec in src.get("defects", []):
        if rec.get("book2_crosswalk"):
            w(f"| {rec['id']} | {rec['title']} | {rec['defect_disposition']} | "
              f"{severity_class(rec)} | {rec['consequence']} | "
              f"{rec['closure_condition']} |")
    w("")
    w("## Deferred populations and projections")
    w("")
    w("| Record type | Stage | Owner | Closure condition |")
    w("| --- | --- | --- | --- |")
    for rec in src["deferred_populations"]:
        w(f"| {rec['record_type']} | {rec['stage']} | `{rec['owner_ref']}` | "
          f"{rec['closure_condition']} |")
    w("")
    w("The coverage-map view, contract cards, role matrix, dependency map, "
      "assurance allocation, reader ledger, and Book 2 crosswalk are generated "
      "projections of this source; each arrives with its owning stage or "
      "tracker item, and none may be maintained by hand.")
    w("")
    w("## Conservative rollup")
    w("")
    w(src["acceptance_gate"]["rollup_rule"])
    w("")
    w("## Reproduce")
    w("")
    w("```bash")
    w("python3 new-book-plans/13-full-society-ledger.py --check")
    w("```")
    w("")
    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="validate and confirm the report is current")
    args = parser.parse_args()

    src = load_json(SOURCE)
    resolution = validate(src)
    controls = negative_controls(src)
    rendered = render(src, resolution)
    spliced_map = validate_coverage_region(src)

    out_path = ROOT / OUTPUT
    map_path = ROOT / COVERAGE_MAP
    if args.check:
        current = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
        if current != rendered:
            raise LedgerError(f"{OUTPUT} is STALE — rerun without --check")
        if map_path.read_text(encoding="utf-8") != spliced_map:
            raise LedgerError(
                f"{COVERAGE_MAP} generated region is STALE — rerun without "
                "--check"
            )
        print(f"{OUTPUT} and the coverage-map region are current; {controls} "
              "structural negative controls pass; enum-mapping and "
              "residual-coverage closures over the six reviewed sources hold; "
              "routing inventory only — nothing established beyond each row's "
              "own posture")
    else:
        out_path.write_text(rendered, encoding="utf-8")
        map_path.write_text(spliced_map, encoding="utf-8")
        print(f"wrote {OUTPUT} and the coverage-map region; {controls} "
              "structural negative controls pass")


if __name__ == "__main__":
    try:
        main()
    except LedgerError as exc:
        print(f"13-full-society-ledger: {exc}", file=sys.stderr)
        sys.exit(1)
