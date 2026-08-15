#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate the claim-scoped constitutional-closure/model-allocation audit.

This is a projection of full-society-ledger.json, never a second reviewed
source. It reuses script 13's validator and generated defect resolution.
"""

from __future__ import annotations

import argparse
import copy
import importlib.util
import pathlib
import sys


ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "new-book-plans/full-society-ledger.json"
OUTPUT = ROOT / "new-book-plans/constitutional-closure-and-model-allocation-audit.md"
LEDGER_SCRIPT = ROOT / "new-book-plans/13-full-society-ledger.py"


class ClosureAuditError(ValueError):
    pass


def load_ledger_module():
    spec = importlib.util.spec_from_file_location("full_society_ledger", LEDGER_SCRIPT)
    if spec is None or spec.loader is None:
        raise ClosureAuditError("cannot load the canonical ledger validator")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


LEDGER = load_ledger_module()

MODEL_NAMES = {
    "FS-RTE-01": "Nibli formal entailment",
    "FS-RTE-02": "quantitative/resource models",
    "FS-RTE-03": "dynamic simulations",
    "FS-RTE-04": "evidence registry",
    "FS-RTE-05": "operational assurance",
    "FS-RTE-06": "reader/lived-experience testing",
    "FS-RTE-07": "repository source-derived adversarial audit",
}
REQUIREMENT_COMPONENTS = {
    "floor-lifecycle": {"delivery", "continuity", "remedy"},
    "public-power-lifecycle": {"source", "limit", "review", "temporal-status"},
    "private-duty-explicitness": {"express-duty"},
    "record-lifecycle": {"writer", "challenge", "correction"},
    "democratic-floor-corridor": {"choice-source", "floor-boundary"},
    "book-seam": {"responsible-book", "assurance-ceiling"},
    "external-assumption-disclosure": {"named-assumption"},
    "reader-claim-ownership": {"formal-owner", "evidentiary-owner"},
}
PROFILE_SOURCES = {
    "floor-lifecycle": "FS-LGR-02",
    "public-power-lifecycle": "FS-LGR-07",
    "private-duty-explicitness": "FS-LGR-03",
    "record-lifecycle": "FS-LGR-08",
    "democratic-floor-corridor": "FS-LGR-06",
    "book-seam": None,
    "external-assumption-disclosure": None,
    "reader-claim-ownership": None,
}
READER_OWNER_REF = (
    "new-book-plans/book-1-reader-evidence-protocol-decision.md::"
    "## 2. The method, ratified as specified"
)
INTRINSIC_PROFILE_CLAIMS = {
    "floor-lifecycle": {
        "FS-CLM-04", "FS-CLM-05", "FS-CLM-06",
    },
    "public-power-lifecycle": {
        "FS-CLM-15", "FS-CLM-17", "FS-CLM-18", "FS-CLM-28",
        "FS-CLM-29", "FS-CLM-30", "FS-CLM-31", "FS-CLM-32",
    },
    "private-duty-explicitness": {
        "FS-CLM-02", "FS-CLM-08", "FS-CLM-09",
        "FS-CLM-23", "FS-CLM-27", "FS-CLM-34",
    },
    "record-lifecycle": {
        "FS-CLM-19", "FS-CLM-20", "FS-CLM-21", "FS-CLM-31",
    },
    "democratic-floor-corridor": {
        "FS-CLM-10", "FS-CLM-14", "FS-CLM-15", "FS-CLM-16",
    },
    "book-seam": {"FS-CLM-03", "FS-CLM-24"},
    "external-assumption-disclosure": {"FS-CLM-13", "FS-CLM-16"},
    "reader-claim-ownership": {"FS-CLM-37"},
}
INTRINSIC_COMPONENT_ALLOWED_REFS = {
    ("floor-lifecycle", "delivery"): {"FS-DEP-25", "FS-DEP-28"},
    ("floor-lifecycle", "continuity"): {"FS-DEP-25", "FS-DEP-57"},
    ("floor-lifecycle", "remedy"): {"FS-DEP-34", "FS-DEP-43"},
    ("public-power-lifecycle", "source"): {"FS-DEP-01", "FS-DEP-05", "FS-DEP-06"},
    ("public-power-lifecycle", "limit"): {"FS-DEP-35", "FS-DEP-37"},
    ("public-power-lifecycle", "review"): {
        "FS-DEP-44", "FS-DEP-49", "FS-DEP-50", "FS-DEP-51",
    },
    ("public-power-lifecycle", "temporal-status"): {"FS-DEP-38", "FS-DEP-62"},
    ("private-duty-explicitness", "express-duty"): {
        "FS-DEP-17", "FS-DEP-18", "FS-DEP-30",
    },
    ("record-lifecycle", "writer"): {"FS-DEP-23", "FS-DEP-26", "FS-DEP-39"},
    ("record-lifecycle", "challenge"): {"FS-DEP-45"},
    ("record-lifecycle", "correction"): {"FS-DEP-45"},
    ("democratic-floor-corridor", "choice-source"): {"FS-DEP-01", "FS-DEP-29"},
    ("democratic-floor-corridor", "floor-boundary"): {"FS-CLM-04", "FS-DEP-11"},
    ("book-seam", "responsible-book"): {"FS-RTE-02", "FS-RTE-03", "FS-RTE-05"},
    ("book-seam", "assurance-ceiling"): {"FS-RTE-02", "FS-RTE-03", "FS-RTE-05"},
    ("external-assumption-disclosure", "named-assumption"): {
        "FS-EXA-01", "FS-EXA-02", "FS-EXA-03", "FS-EXA-04",
    },
    ("reader-claim-ownership", "formal-owner"): {"FS-CLM-37"},
    ("reader-claim-ownership", "evidentiary-owner"): {"FS-RTE-06"},
}
COMPOSITE_MODEL_CLAIMS = {"FS-CLM-24"}

LOOP_HAZARDS = ("unbounded", "self-certifying", "deadlocking", "single-veto", "cascade")
CLOSURE_STATUSES = {"bounded-unresolved", "open-blocking"}


def by_id(source, key):
    return {row["id"]: row for row in source.get(key, [])}


def unique_string_list(value, *, nonempty=True):
    return (
        isinstance(value, list)
        and (bool(value) or not nonempty)
        and all(isinstance(item, str) and item for item in value)
        and len(value) == len(set(value))
    )


def source_ids(source):
    result = {}
    for key in (
        "domains", "legacy_rows", "claims", "bodies", "routes",
        "external_assumptions", "envelope", "roles", "powers",
        "dependencies", "scenarios", "thresholds", "defects", "receipts",
        "closure_requirement_profiles", "closure_claim_contracts",
        "model_allocations", "function_allocations", "dependency_loops",
        "loop_hazard_controls",
        "bottleneck_dispositions",
    ):
        for row in source.get(key, []):
            result[row["id"]] = key
    return result


def expanded_profiles(source):
    raw = source.get("closure_requirement_profiles")
    if not isinstance(raw, list):
        raise ClosureAuditError("closure_requirement_profiles must be reviewed")
    out = {}
    seen_ids = set()
    for index, record in enumerate(raw):
        context = f"closure_requirement_profiles[{index}]"
        if not isinstance(record, dict) or set(record) != {
            "id", "requirement_kind", "applies_to_claim_refs", "components",
            "source_record_ref",
        }:
            raise ClosureAuditError(f"{context}: exact reviewed profile keys required")
        if (not record["id"].startswith("FS-CLR-")
                or record["id"] in seen_ids):
            raise ClosureAuditError(f"{context}: stable unique FS-CLR id required")
        seen_ids.add(record["id"])
        kind = record["requirement_kind"]
        if kind not in REQUIREMENT_COMPONENTS or kind in out:
            raise ClosureAuditError(f"{context}: unknown or duplicate kind {kind!r}")
        claims = record["applies_to_claim_refs"]
        if not unique_string_list(claims):
            raise ClosureAuditError(f"{context}: unique affected claims required")
        component_rows = record["components"]
        if not isinstance(component_rows, list):
            raise ClosureAuditError(f"{context}: components must be a list")
        components = {}
        for component in component_rows:
            if not isinstance(component, dict) or set(component) != {"component", "record_refs"}:
                raise ClosureAuditError(f"{context}: exact component keys required")
            name = component["component"]
            refs = component["record_refs"]
            if name in components or not unique_string_list(refs):
                raise ClosureAuditError(f"{context}.{name}: one unique non-empty binding required")
            components[name] = tuple(refs)
        if set(components) != REQUIREMENT_COMPONENTS[kind]:
            raise ClosureAuditError(
                f"{context}: {kind} requires exactly {sorted(REQUIREMENT_COMPONENTS[kind])}"
            )
        if record["source_record_ref"] != PROFILE_SOURCES[kind]:
            raise ClosureAuditError(f"{context}: wrong reviewed source record")
        out[kind] = {
            "id": record["id"], "claims": tuple(claims),
            "components": components, "source": record["source_record_ref"],
        }
    if set(out) != set(REQUIREMENT_COMPONENTS):
        raise ClosureAuditError("every closure requirement family must occur exactly once")

    claims_by_id = by_id(source, "claims")
    claim_ids = set(claims_by_id)
    profile_ids = {profile["id"] for profile in out.values()}
    contracts = source.get("closure_claim_contracts")
    if not isinstance(contracts, list):
        raise ClosureAuditError("closure_claim_contracts must be reviewed")
    contract_by_claim = {}
    contract_ids = set()
    for index, contract in enumerate(contracts):
        context = f"closure_claim_contracts[{index}]"
        if not isinstance(contract, dict) or set(contract) != {
            "id", "claim_ref", "required_profile_refs",
        }:
            raise ClosureAuditError(f"{context}: exact claim-contract keys required")
        if (not contract["id"].startswith("FS-CCT-")
                or contract["id"] in contract_ids):
            raise ClosureAuditError(f"{context}: stable unique FS-CCT id required")
        contract_ids.add(contract["id"])
        claim_ref = contract["claim_ref"]
        required = contract["required_profile_refs"]
        if claim_ref not in claim_ids or claim_ref in contract_by_claim:
            raise ClosureAuditError(f"{context}: one known claim required")
        if (not unique_string_list(required, nonempty=False)
                or not set(required) <= profile_ids):
            raise ClosureAuditError(f"{context}: unique known required profiles needed")
        contract_by_claim[claim_ref] = tuple(required)
    if set(contract_by_claim) != claim_ids:
        raise ClosureAuditError("every claim needs exactly one reviewed closure contract")

    profile_id_by_kind = {
        kind: profile["id"] for kind, profile in out.items()
    }
    for kind, claim_refs in INTRINSIC_PROFILE_CLAIMS.items():
        unknown = claim_refs - claim_ids
        if unknown:
            raise ClosureAuditError(
                f"{kind}: intrinsic contract names unknown claims {sorted(unknown)}"
            )
    for claim_ref, required in contract_by_claim.items():
        expected = {
            profile_id_by_kind[kind]
            for kind, claim_refs in INTRINSIC_PROFILE_CLAIMS.items()
            if claim_ref in claim_refs
        }
        if set(required) != expected:
            raise ClosureAuditError(
                f"{claim_ref}: reviewed closure contract omits or adds an "
                "intrinsic claim obligation"
            )
        if set(claims_by_id[claim_ref]["closure_requirement_refs"]) != set(required):
            raise ClosureAuditError(
                f"{claim_ref}: claim closure refs drift from its reviewed closure contract"
            )
    for profile in out.values():
        expected = {
            claim for claim, required in contract_by_claim.items()
            if profile["id"] in required
        }
        if set(profile["claims"]) != expected:
            raise ClosureAuditError(
                f"{profile['id']}: profile membership drifts from claim contracts"
            )

    expected_dependency_components = {
        dependency["id"]: set() for dependency in source["dependencies"]
    }
    for profile in out.values():
        for component, refs in profile["components"].items():
            token = f"{profile['id']}:{component}"
            for ref in refs:
                if ref in expected_dependency_components:
                    expected_dependency_components[ref].add(token)
    for dependency in source["dependencies"]:
        expected = expected_dependency_components[dependency["id"]]
        if set(dependency["closure_component_refs"]) != expected:
            raise ClosureAuditError(
                f"{dependency['id']}: dependency-owned closure component classification is stale"
            )

    legacy = by_id(source, "legacy_rows")
    if tuple(legacy["FS-LGR-02"]["split_claim_refs"]) != out["floor-lifecycle"]["claims"]:
        raise ClosureAuditError("floor profile must equal reviewed FS-LGR-02 split claims")
    book2 = {c["id"] for c in source["claims"] if c["layer"] == "book-2-operation"}
    if set(out["book-seam"]["claims"]) != book2:
        raise ClosureAuditError("book-seam profile must cover every Book 2 claim")
    external = {c["id"] for c in source["claims"] if c["layer"] == "external-assumption"}
    if set(out["external-assumption-disclosure"]["claims"]) != external:
        raise ClosureAuditError("external profile must cover every external claim")
    reader_owned = {
        c["id"] for c in source["claims"] if c["owner_ref"] == READER_OWNER_REF
    }
    if not reader_owned or set(out["reader-claim-ownership"]["claims"]) != reader_owned:
        raise ClosureAuditError(
            "reader profile must equal claims owned by the reader-evidence protocol"
        )
    return out

def require_dependencies(source, refs, context, classes, flows, lifecycles):
    dependencies = by_id(source, "dependencies")
    for ref in refs:
        dependency = dependencies.get(ref)
        if dependency is None:
            raise ClosureAuditError(f"{context}: {ref} must be a dependency")
        if (dependency["dependency_class"] not in classes
                or dependency["flow_kind"] not in flows
                or dependency["lifecycle_path"] not in lifecycles):
            raise ClosureAuditError(f"{context}: {ref} has the wrong typed contract")


def validate_profile_bindings(source, profiles):
    ids = source_ids(source)
    floor_claims = set(profiles["floor-lifecycle"]["claims"])
    for kind, profile in profiles.items():
        for component, refs in profile["components"].items():
            unknown = [ref for ref in refs if ref not in ids]
            if unknown:
                raise ClosureAuditError(f"{kind}.{component}: unknown refs {unknown}")
            allowed = INTRINSIC_COMPONENT_ALLOWED_REFS[(kind, component)]
            if not set(refs) <= allowed:
                raise ClosureAuditError(
                    f"{kind}.{component}: binding is outside its reviewed "
                    "semantic edge contract"
                )
    p = profiles
    require_dependencies(source, p["floor-lifecycle"]["components"]["delivery"],
                         "floor.delivery", {"operationally-supplied"}, {"services"}, {"right"})
    require_dependencies(source, p["floor-lifecycle"]["components"]["continuity"],
                         "floor.continuity", {"operationally-supplied", "constitutionally-guaranteed"},
                         {"services", "accountability"}, {"right"})
    require_dependencies(source, p["floor-lifecycle"]["components"]["remedy"],
                         "floor.remedy", {"operationally-supplied", "constitutionally-guaranteed"},
                         {"information", "claims"}, {"right"})
    require_dependencies(source, p["public-power-lifecycle"]["components"]["source"],
                         "power.source", {"democratically-selected", "constitutionally-guaranteed"},
                         {"authority"}, {"power"})
    require_dependencies(source, p["public-power-lifecycle"]["components"]["limit"],
                         "power.limit", {"constitutionally-guaranteed"}, {"information"},
                         {"power", "record"})
    require_dependencies(source, p["public-power-lifecycle"]["components"]["review"],
                         "power.review", {"constitutionally-guaranteed"}, {"claims", "accountability"},
                         {"power", "record"})
    require_dependencies(source, p["public-power-lifecycle"]["components"]["temporal-status"],
                         "power.temporal", {"externally-assumed", "constitutionally-guaranteed"},
                         {"information", "accountability"}, {"power"})
    require_dependencies(source, p["private-duty-explicitness"]["components"]["express-duty"],
                         "private-duty", {"constitutionally-guaranteed", "operationally-supplied"},
                         {"care", "services"}, {"right"})
    require_dependencies(source, p["record-lifecycle"]["components"]["writer"],
                         "record.writer", {"externally-assumed", "operationally-supplied"},
                         {"resources", "services", "information"}, {"record"})
    for component in ("challenge", "correction"):
        require_dependencies(source, p["record-lifecycle"]["components"][component],
                             f"record.{component}", {"constitutionally-guaranteed"}, {"claims"},
                             {"record"})
    require_dependencies(source, p["democratic-floor-corridor"]["components"]["choice-source"],
                         "democracy.choice", {"democratically-selected", "operationally-supplied"},
                         {"authority", "services"}, {"power", "record"})
    boundary = p["democratic-floor-corridor"]["components"]["floor-boundary"]
    claims = [ref for ref in boundary if ids[ref] == "claims"]
    dependencies = [ref for ref in boundary if ids[ref] == "dependencies"]
    if not claims or not set(claims) <= floor_claims:
        raise ClosureAuditError("democracy.floor-boundary must cite a floor claim")
    require_dependencies(source, dependencies, "democracy.floor-boundary",
                         {"constitutionally-guaranteed"}, {"money"}, {"right"})
    for component in p["book-seam"]["components"].values():
        if any(ids[ref] != "routes" for ref in component):
            raise ClosureAuditError("book-seam components must name routes")
    external_refs = set(p["external-assumption-disclosure"]["components"]["named-assumption"])
    if external_refs != set(by_id(source, "external_assumptions")):
        raise ClosureAuditError("external profile must name every reviewed assumption")
    reader = p["reader-claim-ownership"]["components"]
    if any(ids[ref] != "claims" for ref in reader["formal-owner"]):
        raise ClosureAuditError("reader formal-owner must name claims")
    if any(ids[ref] != "routes" for ref in reader["evidentiary-owner"]):
        raise ClosureAuditError("reader evidentiary-owner must name routes")



def claim_component_coverage(source, profiles):
    """Derive claim-scoped bindings from reviewed domains and typed records."""
    ids = source_ids(source)
    claims = by_id(source, "claims")
    dependencies = by_id(source, "dependencies")
    result = {claim_ref: {} for claim_ref in claims}
    for kind, profile in profiles.items():
        for claim_ref in profile["claims"]:
            claim_domains = set(claims[claim_ref]["domain_refs"])
            relevant_external = set(relevant_external_assumptions(
                source, claims[claim_ref]
            ))
            for component, refs in profile["components"].items():
                key = f"{kind}.{component}"
                bindings = []
                for ref in refs:
                    record_type = ids[ref]
                    if record_type == "dependencies":
                        dependency = dependencies[ref]
                        domains = (
                            endpoint_domains(source, dependency["from_ref"])
                            | endpoint_domains(source, dependency["to_ref"])
                        )
                        if claim_domains & domains:
                            bindings.append(ref)
                    elif record_type == "external_assumptions":
                        if ref in relevant_external:
                            bindings.append(ref)
                    elif (kind == "reader-claim-ownership"
                          and component == "formal-owner"):
                        if ref == claim_ref:
                            bindings.append(ref)
                    else:
                        bindings.append(ref)
                result[claim_ref][key] = tuple(bindings)
    return result


def validate_component_consumption(profiles, component_coverage):
    """Refuse reviewed dependency bindings that support no exact claim."""
    for kind, profile in profiles.items():
        for component, refs in profile["components"].items():
            key = f"{kind}.{component}"
            for ref in refs:
                if not ref.startswith("FS-DEP-"):
                    continue
                if not any(
                    ref in component_coverage[claim_ref].get(key, ())
                    for claim_ref in profile["claims"]
                ):
                    raise ClosureAuditError(
                        f"{key}: {ref} has no claim-scoped consumer"
                    )


def profile_membership(profiles):
    result = {}
    for kind, profile in profiles.items():
        for claim in profile["claims"]:
            result.setdefault(claim, []).append(profile["id"])
    return result


def validate_model_allocations(source, profiles):
    claims = by_id(source, "claims")
    routes = by_id(source, "routes")
    membership = profile_membership(profiles)
    rows = source.get("model_allocations")
    if not isinstance(rows, list):
        raise ClosureAuditError("model_allocations must be reviewed")
    by_claim = {}
    reader_claims = set(profiles["reader-claim-ownership"]["claims"])
    external_claims = set(profiles["external-assumption-disclosure"]["claims"])
    book_claims = set(profiles["book-seam"]["claims"])
    record_claims = set(profiles["record-lifecycle"]["claims"])
    for index, row in enumerate(rows):
        context = f"model_allocations[{index}]"
        if not isinstance(row, dict) or set(row) != {
            "id", "claim_ref", "primary_route_ref", "required_route_refs",
            "closure_profile_refs",
        }:
            raise ClosureAuditError(f"{context}: exact allocation keys required")
        claim_ref = row["claim_ref"]
        if claim_ref not in claims or claim_ref in by_claim:
            raise ClosureAuditError(f"{context}: one known claim required")
        required = row["required_route_refs"]
        if (not unique_string_list(required)
                or any(route not in routes for route in required)):
            raise ClosureAuditError(f"{context}: unique known required routes needed")
        claim = claims[claim_ref]
        if row["primary_route_ref"] != claim["route_ref"] or claim["route_ref"] not in required:
            raise ClosureAuditError(f"{context}: primary route drift or substitution")

        if claim["layer"] == "external-assumption" or claim_ref in external_claims:
            if claim["posture"] != "Unestablished":
                raise ClosureAuditError(
                    f"{context}: external claims must remain Unestablished"
                )
            expected_required = ["FS-RTE-05"]
        elif claim["layer"] == "book-2-operation" or claim_ref in book_claims:
            if claim["posture"] != "Unestablished":
                raise ClosureAuditError(
                    f"{context}: Book 2 operation claims must remain Unestablished"
                )
            expected_required = (
                ["FS-RTE-02", "FS-RTE-03", "FS-RTE-05"]
                if claim_ref in COMPOSITE_MODEL_CLAIMS
                else ["FS-RTE-05"]
            )
        elif claim_ref in reader_claims:
            expected_required = ["FS-RTE-06"]
        elif claim["posture"] in {"Derived", "Checked", "Specified", "Reasoned"}:
            expected_required = ["FS-RTE-01"]
        elif claim["overlay"] == "liveness" or claim_ref in record_claims:
            expected_required = ["FS-RTE-05"]
        else:
            raise ClosureAuditError(
                f"{context}: unestablished claim has no reviewed model contract"
            )
        if required != expected_required or claim["route_ref"] != expected_required[0]:
            raise ClosureAuditError(f"{context}: required route composition drift")
        profile_refs = row["closure_profile_refs"]
        if (not unique_string_list(profile_refs, nonempty=False)
                or set(profile_refs) != set(membership.get(claim_ref, []))):
            raise ClosureAuditError(f"{context}: closure profile inverse mapping is stale")
        by_claim[claim_ref] = row
    if set(by_claim) != set(claims):
        raise ClosureAuditError("model allocation must cover every claim exactly once")

    allocated_reader_claims = {
        claim for claim, row in by_claim.items()
        if "FS-RTE-06" in row["required_route_refs"]
    }
    reader = profiles["reader-claim-ownership"]
    if set(reader["claims"]) != allocated_reader_claims:
        raise ClosureAuditError("reader ownership must cover every R6 claim")
    if set(reader["components"]["formal-owner"]) != allocated_reader_claims:
        raise ClosureAuditError("reader formal-owner claim set is stale")
    reader_routes = {
        route for claim in allocated_reader_claims
        for route in by_claim[claim]["required_route_refs"]
    }
    if set(reader["components"]["evidentiary-owner"]) != reader_routes:
        raise ClosureAuditError("reader evidentiary-owner route set is stale")
    book = profiles["book-seam"]
    book_routes = {
        route for claim in book["claims"]
        for route in by_claim[claim]["required_route_refs"]
    }
    for component in book["components"].values():
        if set(component) != book_routes:
            raise ClosureAuditError("book-seam must bind every required model route")
    return by_claim

def validate_function_separation_row(
        row, bodies, roles, claims, powers, context):
    function_fields = []
    for function in LEDGER.POWER_FUNCTIONS:
        stem = function.replace("-", "_")
        function_fields.extend([
            f"{stem}_body_refs", f"{stem}_role_refs",
        ])
    keys = {
        "id", "power_ref", "affected_claim_refs", *function_fields,
        "separation_constraints", "source_refs",
    }
    if not isinstance(row, dict) or set(row) != keys:
        raise ClosureAuditError(f"{context}: exact function keys required")
    power = powers.get(row["power_ref"])
    if power is None:
        raise ClosureAuditError(f"{context}: power_ref must name an FS-POW card")
    affected = row["affected_claim_refs"]
    if affected != power["affected_claim_refs"] or any(
            ref not in claims for ref in affected):
        raise ClosureAuditError(
            f"{context}: affected claims must equal the power card"
        )
    body_sets = {}
    for function in LEDGER.POWER_FUNCTIONS:
        stem = function.replace("-", "_")
        body_refs = row[f"{stem}_body_refs"]
        role_refs = row[f"{stem}_role_refs"]
        if (not unique_string_list(body_refs)
                or any(ref not in bodies for ref in body_refs)):
            raise ClosureAuditError(
                f"{context}.{stem}_body_refs: unique known body refs required"
            )
        if (not unique_string_list(role_refs)
                or any(ref not in roles for ref in role_refs)):
            raise ClosureAuditError(
                f"{context}.{stem}_role_refs: unique known role refs required"
            )
        body_sets[function] = set(body_refs)
    constraints = row["separation_constraints"]
    expected_pairs = (
        power["required_separation_pairs"]
        if "required_separation_pairs" in power
        else power["contract"]["required_separation_pairs"]
    )
    if not isinstance(constraints, list) or len(constraints) != len(expected_pairs):
        raise ClosureAuditError(
            f"{context}: one source-backed constraint is required per pair"
        )
    for constraint, pair in zip(constraints, expected_pairs):
        if not isinstance(constraint, dict) or set(constraint) != {
                "functions", "reason", "source_ref"}:
            raise ClosureAuditError(
                f"{context}: exact separation-constraint keys required"
            )
        if constraint["functions"] != pair or not constraint["reason"]:
            raise ClosureAuditError(
                f"{context}: separation constraint differs from power card"
            )
        LEDGER.validate_reference(
            constraint["source_ref"], f"{context}.separation_constraints"
        )
        if body_sets[pair[0]] & body_sets[pair[1]]:
            raise ClosureAuditError(
                f"{context}: required body functions are fused"
            )
    if set.intersection(*(body_sets[f] for f in LEDGER.POWER_FUNCTIONS)):
        raise ClosureAuditError(
            f"{context}: one body self-certifies all five functions"
        )
    source_refs = row["source_refs"]
    if not unique_string_list(source_refs):
        raise ClosureAuditError(f"{context}.source_refs: unique exact refs required")
    for ref in source_refs:
        LEDGER.validate_reference(ref, f"{context}.source_refs")


def validate_function_allocations(source, profiles):
    allocations = source.get("function_allocations")
    if not isinstance(allocations, list):
        raise ClosureAuditError("function_allocations must be reviewed")
    expected_claims = set(profiles["public-power-lifecycle"]["claims"])
    powers = by_id(source, "powers")
    bodies = by_id(source, "bodies")
    roles = by_id(source, "roles")
    claims = by_id(source, "claims")
    seen = set()
    for index, row in enumerate(allocations):
        context = f"function_allocations[{index}]"
        validate_function_separation_row(
            row, bodies, roles, claims, powers, context
        )
        if row["power_ref"] in seen:
            raise ClosureAuditError(
                f"{context}: duplicate power-bound allocation"
            )
        seen.add(row["power_ref"])
        expected_claims.update(row["affected_claim_refs"])
    if seen != set(powers):
        raise ClosureAuditError(
            "function allocations must be a complete power-card bijection"
        )
    status = source["power_population"]["status"]
    powers_deferred = any(
        d["record_type"] == "powers"
        for d in source["deferred_populations"]
    )
    if status == "complete":
        if powers_deferred or len(powers) != LEDGER.POWER_FINAL_COUNTS["powers"]:
            raise ClosureAuditError(
                "complete power population must remove its deferral and bind "
                "every source-derived power"
            )
        return {
            "result": "pass",
            "affected_claim_refs": sorted(expected_claims),
            "reason": (
                "all source-derived FS-POW cards have one typed, power-bound "
                "function allocation; structural separation establishes no "
                "operation or institutional independence"
            ),
        }
    if not powers_deferred:
        raise ClosureAuditError(
            "the powers deferral must remain through foundation and partial prefixes"
        )
    return {
        "result": "bounded-unresolved",
        "affected_claim_refs": sorted(expected_claims),
        "reason": (
            f"source-derived power population is {status}: "
            f"{len(powers)} cards and {len(allocations)} matching allocations; "
            "the remaining families stay explicitly deferred"
        ),
    }


def endpoint_domains(source, ref):
    domains = by_id(source, "domains")
    roles = by_id(source, "roles")
    if ref in domains:
        return {ref}
    if ref in roles:
        return set(roles[ref]["domain_refs"])
    if ref.startswith("FS-BOD-"):
        return {d["id"] for d in source["domains"] if ref in d["bodies_refs"]}
    if ref.startswith("FS-EXA-"):
        return {d["id"] for d in source["domains"] if ref in d["external_assumption_refs"]}
    return set()


def dependency_claim_map(source, profiles):
    profile_claims = {}
    for profile in profiles.values():
        for component in profile["components"].values():
            for ref in component:
                if ref.startswith("FS-DEP-"):
                    profile_claims.setdefault(ref, set()).update(profile["claims"])
    result = {}
    for dependency in source["dependencies"]:
        affected = set(profile_claims.get(dependency["id"], ()))
        domains = endpoint_domains(source, dependency["from_ref"]) | endpoint_domains(
            source, dependency["to_ref"]
        )
        for claim in source["claims"]:
            if domains & set(claim["domain_refs"]):
                affected.add(claim["id"])
        result[dependency["id"]] = sorted(affected)
    return result


def dependency_blocking_scope(component_coverage):
    """Bind blocking propagation to the exact claim/component edge join."""
    result = {}
    for claim_ref, components in component_coverage.items():
        for refs in components.values():
            for ref in refs:
                if ref.startswith("FS-DEP-"):
                    result.setdefault(ref, set()).add(claim_ref)
    return result


def validate_dependencies_and_scenarios(
        source, profiles, resolution, blocking_scope):
    ids = source_ids(source)
    affected = dependency_claim_map(source, profiles)
    defects_by_id = by_id(source, "defects")
    for dependency in source["dependencies"]:
        sat = dependency["structural_satisfiability"]
        if sat["satisfiability_status"] == "unsatisfiable":
            cited_claims = set()
            for defect_ref in sat["defect_refs"]:
                defect = defects_by_id[defect_ref]
                if (LEDGER.severity_class(defect) != "critical"
                        or not resolution[defect_ref]["blocking"]):
                    raise ClosureAuditError(
                        f"{dependency['id']}: unsatisfiable must cite a blocking critical defect"
                    )
                cited_claims.add(defect["affected_claim_ref"])
            if not cited_claims <= blocking_scope.get(dependency["id"], set()):
                raise ClosureAuditError(
                    f"{dependency['id']}: unsatisfiable blockers must stay "
                    "within its claim-scoped component bindings"
                )
    for scenario in source["scenarios"]:
        if ids.get(scenario["steward_ref"]) != "bodies":
            raise ClosureAuditError(f"{scenario['id']}: scenario is unowned")
        for key in ("ordinary_route", "failure_route", "recovery_route"):
            if not scenario.get(key):
                raise ClosureAuditError(f"{scenario['id']}: scenario lacks {key}")
    return affected


def validate_control_status(row, context, source, resolution, affected_claims):
    status = row["closure_status"]
    controls = row["control_refs"]
    defects = row["defect_refs"]
    if status not in CLOSURE_STATUSES:
        raise ClosureAuditError(
            f"{context}: closure_status must remain bounded-unresolved or "
            "open-blocking until a typed executable control receipt exists"
        )
    if status == "bounded-unresolved":
        if controls or defects:
            raise ClosureAuditError(f"{context}: bounded-unresolved has no proof or blocker")
    else:
        if controls or not defects:
            raise ClosureAuditError(f"{context}: open-blocking requires defects only")
        defects_by_id = by_id(source, "defects")
        cited_claims = set()
        for ref in defects:
            if (ref not in defects_by_id
                    or LEDGER.severity_class(defects_by_id[ref]) != "critical"
                    or not resolution[ref]["blocking"]):
                raise ClosureAuditError(f"{context}: blocking defect is ineligible")
            cited_claims.add(defects_by_id[ref]["affected_claim_ref"])
        if not cited_claims <= set(affected_claims):
            raise ClosureAuditError(
                f"{context}: blocking defects must stay within claim-scoped "
                "component bindings"
            )
        return cited_claims
    return set()


def validate_loop_controls(
        source, profiles, resolution, dependency_claims, blocking_scope):
    loops = by_id(source, "dependency_loops")
    controls = source.get("loop_hazard_controls")
    if not isinstance(controls, list):
        raise ClosureAuditError("loop_hazard_controls must be reviewed")
    by_loop = {}
    result_rows = []
    for index, control in enumerate(controls):
        context = f"loop_hazard_controls[{index}]"
        if not isinstance(control, dict) or set(control) != {
            "id", "loop_ref", "affected_claim_refs", "assessments",
        }:
            raise ClosureAuditError(f"{context}: exact loop control keys required")
        loop_ref = control["loop_ref"]
        if loop_ref not in loops or loop_ref in by_loop:
            raise ClosureAuditError(f"{context}: one known loop required")
        loop = loops[loop_ref]
        expected_claims = sorted({
            claim for dep in loop["member_edge_refs"]
            for claim in dependency_claims[dep]
        })
        eligible_blocking_claims = sorted({
            claim for dep in loop["member_edge_refs"]
            for claim in blocking_scope.get(dep, set())
        })
        if control["affected_claim_refs"] != expected_claims:
            raise ClosureAuditError(f"{context}: affected-claim binding is stale")
        assessments = control["assessments"]
        if not isinstance(assessments, list) or len(assessments) != len(LOOP_HAZARDS):
            raise ClosureAuditError(f"{context}: every hazard must be assessed")
        statuses = []
        hazard_statuses = {}
        blocking_claims = set()
        seen = set()
        for assessment in assessments:
            if not isinstance(assessment, dict) or set(assessment) != {
                "hazard", "closure_status", "control_refs", "defect_refs", "reason",
            }:
                raise ClosureAuditError(f"{context}: exact hazard keys required")
            hazard = assessment["hazard"]
            if hazard not in LOOP_HAZARDS or hazard in seen or not assessment["reason"]:
                raise ClosureAuditError(f"{context}: invalid hazard assessment")
            seen.add(hazard)
            blocking_claims.update(validate_control_status(
                assessment, f"{context}.{hazard}", source, resolution,
                eligible_blocking_claims,
            ))
            statuses.append(assessment["closure_status"])
            hazard_statuses[hazard] = assessment["closure_status"]
        result = (
            "block" if "open-blocking" in statuses
            else "bounded-unresolved" if "bounded-unresolved" in statuses
            else "pass"
        )
        result_rows.append({
            "id": loop_ref, "kind": loop["loop_kind"],
            "members": loop["member_edge_refs"], "result": result,
            "statuses": hazard_statuses,
            "owner": loop["owner_ref"], "affected_claim_refs": expected_claims,
            "blocking_claim_refs": sorted(blocking_claims),
        })
        by_loop[loop_ref] = control
    if set(by_loop) != set(loops):
        raise ClosureAuditError("every stable loop needs exactly one hazard-control row")
    return result_rows


def validate_bottlenecks(
        source, resolution, dependency_claims, blocking_scope):
    dependencies = by_id(source, "dependencies")
    candidates = {
        row["id"] for row in source["dependencies"]
        if "no_alternate_reason" in row["alternate_route"]
    }
    rows = source.get("bottleneck_dispositions")
    if not isinstance(rows, list):
        raise ClosureAuditError("bottleneck_dispositions must be reviewed")
    seen = set()
    result = []
    for index, row in enumerate(rows):
        context = f"bottleneck_dispositions[{index}]"
        if not isinstance(row, dict) or set(row) != {
            "id", "dependency_ref", "affected_claim_refs", "closure_status",
            "control_refs", "defect_refs", "reason",
        }:
            raise ClosureAuditError(f"{context}: exact bottleneck keys required")
        ref = row["dependency_ref"]
        if ref not in candidates or ref in seen or not row["reason"]:
            raise ClosureAuditError(f"{context}: one current candidate required")
        if row["affected_claim_refs"] != dependency_claims[ref]:
            raise ClosureAuditError(f"{context}: affected-claim binding is stale")
        blocking_claims = validate_control_status(
            row, context, source, resolution,
            sorted(blocking_scope.get(ref, set())),
        )
        status = row["closure_status"]
        result.append({
            "id": ref,
            "result": "block" if status == "open-blocking" else (
                "bounded-unresolved" if status == "bounded-unresolved" else "pass"
            ),
            "owner": dependencies[ref]["owner_ref"], "reason": row["reason"],
            "affected_claim_refs": row["affected_claim_refs"],
            "blocking_claim_refs": sorted(blocking_claims),
        })
        seen.add(ref)
    if seen != candidates:
        raise ClosureAuditError("bottleneck dispositions must exactly cover no-alternate edges")
    return result


def relevant_scenarios(source, claim):
    domains = set(claim["domain_refs"])
    return sorted(s["id"] for s in source["scenarios"] if domains & set(s["domain_refs"]))


def relevant_external_assumptions(source, claim):
    domains = by_id(source, "domains")
    return sorted({
        ref for domain in claim["domain_refs"]
        for ref in domains[domain]["external_assumption_refs"]
    })


def relevant_roles(source, claim):
    domains = set(claim["domain_refs"])
    return sorted(r["id"] for r in source["roles"] if domains & set(r["domain_refs"]))


def compute_claim_audit(source, resolution, profiles, allocations, function_row,
                        dependency_claims, loops, bottlenecks,
                        component_coverage):
    routes = by_id(source, "routes")
    defects_by_claim = {}
    for defect in source["defects"]:
        defects_by_claim.setdefault(defect["affected_claim_ref"], []).append(defect)
    receipts_by_defect = {}
    for receipt in source["receipts"]:
        receipts_by_defect.setdefault(receipt["defect_row_ref"], []).append(receipt)
    dep_by_claim = {c["id"]: [] for c in source["claims"]}
    for dependency, claims in dependency_claims.items():
        for claim in claims:
            dep_by_claim[claim].append(dependency)
    rows = []
    for claim in source["claims"]:
        blockers = []
        bounded = []
        claim_components = component_coverage[claim["id"]]
        missing_components = sorted(
            component for component, refs in claim_components.items() if not refs
        )
        if missing_components:
            blockers.append(
                "missing claim-scoped closure components: "
                + ", ".join(missing_components)
            )
        defects = defects_by_claim.get(claim["id"], [])
        critical = [d["id"] for d in defects if resolution[d["id"]]["blocking"]]
        unresolved = [
            d["id"] for d in defects
            if resolution[d["id"]]["resolution_status"] == "unresolved-for-claim"
        ]
        if critical:
            blockers.append("critical unresolved defects: " + ", ".join(critical))
        elif unresolved:
            bounded.append("claim-relative unresolved defects: " + ", ".join(unresolved))
        for dependency_ref in dep_by_claim[claim["id"]]:
            sat = by_id(source, "dependencies")[dependency_ref]["structural_satisfiability"]
            status = sat["satisfiability_status"]
            if status == "unsatisfiable":
                defects_by_id = by_id(source, "defects")
                scoped_claims = {
                    defects_by_id[ref]["affected_claim_ref"]
                    for ref in sat["defect_refs"]
                }
                if claim["id"] in scoped_claims:
                    blockers.append(f"{dependency_ref} is structurally unsatisfiable")
                else:
                    bounded.append(
                        f"{dependency_ref} is structurally unsatisfiable outside "
                        "this claim's cited blocker scope"
                    )
            elif status in {"operation-deferred", "external-contingent"}:
                bounded.append(f"{dependency_ref} remains {status}")
        for row in loops:
            if claim["id"] in row["affected_claim_refs"]:
                if row["result"] == "block":
                    if claim["id"] in row["blocking_claim_refs"]:
                        blockers.append(f"{row['id']} has an open blocking hazard")
                    else:
                        bounded.append(
                            f"{row['id']} has an open hazard outside this "
                            "claim's cited blocker scope"
                        )
                elif (row["result"] == "bounded-unresolved"
                      or "bounded-unresolved" in row["statuses"].values()):
                    bounded.append(f"{row['id']} hazards remain bounded-unresolved")
        for row in bottlenecks:
            if claim["id"] in row["affected_claim_refs"]:
                if row["result"] == "block":
                    if claim["id"] in row["blocking_claim_refs"]:
                        blockers.append(f"{row['id']} is an open blocking bottleneck")
                    else:
                        bounded.append(
                            f"{row['id']} is an open bottleneck outside this "
                            "claim's cited blocker scope"
                        )
                elif row["result"] == "bounded-unresolved":
                    bounded.append(f"{row['id']} bottleneck remains bounded-unresolved")
        if claim["id"] in function_row["affected_claim_refs"]:
            if function_row["result"] == "block":
                blockers.append(function_row["reason"])
            elif function_row["result"] == "bounded-unresolved":
                bounded.append(function_row["reason"])
        allocation = allocations[claim["id"]]
        unbuilt = [r for r in allocation["required_route_refs"] if routes[r]["route_status"] == "unbuilt"]
        if unbuilt:
            bounded.append("required routes unbuilt: " + ", ".join(unbuilt))
        if claim["posture"] not in LEDGER.ESTABLISHED_POSTURES:
            bounded.append(f"claim posture is {claim['posture']}")
        if claim["layer"] in {"book-2-operation", "external-assumption"}:
            bounded.append(f"scope disposition is {claim['layer']}")
        result = "block" if blockers else ("bounded-unresolved" if bounded else "pass")
        reasons = blockers + bounded
        if not reasons:
            reasons = ["the exact structural contract passes at the claim's existing posture and scope"]
        rows.append({
            "id": claim["id"], "title": claim["title"], "result": result,
            "posture": claim["posture"], "route": claim["route_ref"],
            "required_routes": allocation["required_route_refs"],
            "profiles": allocation["closure_profile_refs"],
            "defects": [d["id"] for d in defects], "unresolved": unresolved,
            "receipts": sorted(r["id"] for d in defects for r in receipts_by_defect.get(d["id"], [])),
            "reasons": reasons, "dependencies": sorted(dep_by_claim[claim["id"]]),
            "scenarios": relevant_scenarios(source, claim),
            "roles": relevant_roles(source, claim),
            "external_assumptions": relevant_external_assumptions(source, claim),
            "component_coverage": claim_components,
        })
    return rows


def validate_generated_results(source, rows, resolution):
    if {r["id"] for r in rows} != set(by_id(source, "claims")) or len(rows) != len(source["claims"]):
        raise ClosureAuditError("claim audit must contain every claim exactly once")
    blocking_claims = {
        d["affected_claim_ref"] for d in source["defects"] if resolution[d["id"]]["blocking"]
    }
    for row in rows:
        if row["result"] not in {"pass", "block", "bounded-unresolved"}:
            raise ClosureAuditError(f"{row['id']}: invalid generated result")
        if row["id"] in blocking_claims and row["result"] != "block":
            raise ClosureAuditError(f"{row['id']}: critical unresolved defect failed to block")


def validate_external_disclosure(source):
    assumptions = set(by_id(source, "external_assumptions"))
    cited = set()
    for domain in source["domains"]:
        if not set(domain["external_assumption_refs"]) <= assumptions:
            raise ClosureAuditError(f"{domain['id']}: hidden external assumption")
    for dependency in source["dependencies"]:
        if dependency["dependency_class"] == "externally-assumed":
            cited.add(dependency["from_ref"])
    if cited != assumptions:
        raise ClosureAuditError("every external assumption must feed the dependency map")


def validate_contract(source, resolution):
    profiles = expanded_profiles(source)
    validate_profile_bindings(source, profiles)
    component_coverage = claim_component_coverage(source, profiles)
    validate_component_consumption(profiles, component_coverage)
    blocking_scope = dependency_blocking_scope(component_coverage)
    allocations = validate_model_allocations(source, profiles)
    function_row = validate_function_allocations(source, profiles)
    dependency_claims = validate_dependencies_and_scenarios(
        source, profiles, resolution, blocking_scope
    )
    validate_external_disclosure(source)
    loops = validate_loop_controls(
        source, profiles, resolution, dependency_claims, blocking_scope
    )
    bottlenecks = validate_bottlenecks(
        source, resolution, dependency_claims, blocking_scope
    )
    rows = compute_claim_audit(
        source, resolution, profiles, allocations, function_row,
        dependency_claims, loops, bottlenecks, component_coverage,
    )
    validate_generated_results(source, rows, resolution)
    return (
        profiles, allocations, function_row, dependency_claims, rows, loops,
        bottlenecks, component_coverage,
    )


def append_control_audit(changed, title):
    changed["closure_record"] = None
    changed["acceptance_gate"].update({
        "verdict": LEDGER.VERDICT_NOT_PASSED,
        "gate_a_status": "not-passed",
    })
    if not changed.get("scope_audits"):
        return
    audit = copy.deepcopy(changed["scope_audits"][-1])
    audit.update({"id": "FS-SAU-98", "title": title})
    changed["scope_audits"].append(audit)


def expect_failure(name, source, mutate, contains=None):
    changed = copy.deepcopy(source)
    append_control_audit(changed, "Closure-audit watched mutation")
    mutate(changed)
    if changed.get("scope_audits"):
        changed["scope_audits"][-1]["scope_sha256"] = \
            LEDGER.review_scope_digest(changed)
    try:
        resolution = LEDGER.validate(changed)
        validate_contract(changed, resolution)
    except (LEDGER.LedgerError, ClosureAuditError) as exc:
        if contains and contains not in str(exc):
            raise ClosureAuditError(f"control {name!r} failed for wrong reason: {exc}") from exc
        return
    raise ClosureAuditError(f"control {name!r} did not fail")


def profile(source, kind):
    return next(p for p in source["closure_requirement_profiles"] if p["requirement_kind"] == kind)


def component(source, kind, name):
    return next(c for c in profile(source, kind)["components"] if c["component"] == name)


def claim_contract(source, claim_ref):
    return next(
        contract for contract in source["closure_claim_contracts"]
        if contract["claim_ref"] == claim_ref
    )


def negative_controls(source):
    controls = []

    def add(name, mutate, contains=None):
        expect_failure(name, source, mutate, contains)
        controls.append(name)

    def substitute_floor_delivery_with_wrong_lifecycle(s):
        next(d for d in s["dependencies"] if d["id"] == "FS-DEP-25")[
            "lifecycle_path"
        ] = "record"

    def substitute_floor_delivery_with_same_signature(s):
        refs = component(s, "floor-lifecycle", "delivery")["record_refs"]
        refs[refs.index("FS-DEP-25")] = "FS-DEP-30"
        next(d for d in s["dependencies"] if d["id"] == "FS-DEP-25")[
            "closure_component_refs"
        ].remove("FS-CLR-01:delivery")
        next(d for d in s["dependencies"] if d["id"] == "FS-DEP-30")[
            "closure_component_refs"
        ].append("FS-CLR-01:delivery")

    def jointly_substitute_route(s, claim_ref, route_ref):
        claim = next(c for c in s["claims"] if c["id"] == claim_ref)
        claim["route_ref"] = route_ref
        allocation = next(
            a for a in s["model_allocations"] if a["claim_ref"] == claim_ref
        )
        allocation["primary_route_ref"] = route_ref
        allocation["required_route_refs"] = [route_ref]

    def remove_profile_projection(s, kind, claim_ref):
        row = profile(s, kind)
        profile_ref = row["id"]
        row["applies_to_claim_refs"].remove(claim_ref)
        next(c for c in s["claims"] if c["id"] == claim_ref)[
            "closure_requirement_refs"
        ].remove(profile_ref)
        next(a for a in s["model_allocations"] if a["claim_ref"] == claim_ref)[
            "closure_profile_refs"
        ].remove(profile_ref)

    def attach_external_claim_to_reader_route(s):
        jointly_substitute_route(s, "FS-CLM-13", "FS-RTE-06")
        reader = profile(s, "reader-claim-ownership")
        reader["applies_to_claim_refs"].append("FS-CLM-13")
        component(s, "reader-claim-ownership", "formal-owner")[
            "record_refs"
        ].append("FS-CLM-13")
        next(c for c in s["claims"] if c["id"] == "FS-CLM-13")[
            "closure_requirement_refs"
        ].append("FS-CLR-08")
        next(a for a in s["model_allocations"] if a["claim_ref"] == "FS-CLM-13")[
            "closure_profile_refs"
        ].append("FS-CLR-08")
        claim_contract(s, "FS-CLM-13")["required_profile_refs"].append(
            "FS-CLR-08"
        )

    def elevate_scoped_claim(s, claim_ref):
        claim = next(c for c in s["claims"] if c["id"] == claim_ref)
        claim["posture"] = "Derived"
        claim["evidence_kind"] = "executable"
        claim.pop("unestablished_disposition")
        claim["route_ref"] = "FS-RTE-01"
        allocation = next(
            a for a in s["model_allocations"] if a["claim_ref"] == claim_ref
        )
        allocation["primary_route_ref"] = "FS-RTE-01"
        allocation["required_route_refs"] = ["FS-RTE-01"]

    def refresh_dependency_projections(s):
        profiles = expanded_profiles(s)
        dependency_claims = dependency_claim_map(s, profiles)
        loops = by_id(s, "dependency_loops")
        for row in s["loop_hazard_controls"]:
            row["affected_claim_refs"] = sorted({
                claim_ref
                for dependency_ref in loops[row["loop_ref"]]["member_edge_refs"]
                for claim_ref in dependency_claims[dependency_ref]
            })
        for row in s["bottleneck_dispositions"]:
            row["affected_claim_refs"] = dependency_claims[row["dependency_ref"]]

    def reintroduce_dead_component(s, kind, component_name, dependency_ref):
        profile_row = profile(s, kind)
        component_row = component(s, kind, component_name)
        component_row["record_refs"].append(dependency_ref)
        next(d for d in s["dependencies"] if d["id"] == dependency_ref)[
            "closure_component_refs"
        ].append(f"{profile_row['id']}:{component_name}")
        refresh_dependency_projections(s)

    def fabricate_untyped_power_transition(s):
        s["deferred_populations"] = [
            d for d in s["deferred_populations"] if d["record_type"] != "powers"
        ]
        s["power_population"]["status"] = "complete"
        s["power_population"]["completed_source_families"] = list(
            LEDGER.POWER_SOURCE_FAMILY_ORDER
        )
        s["powers"] = [{"id": "FS-POW-99"}]
        s["function_allocations"] = [{
            "id": "FS-FAL-99", "scope_id": "arbitrary",
            "affected_claim_refs": list(
                profile(s, "public-power-lifecycle")["applies_to_claim_refs"]
            ),
            "decider_refs": ["FS-BOD-01"], "executor_refs": ["FS-BOD-02"],
            "auditor_refs": ["FS-BOD-03"], "final_remedy_refs": ["FS-BOD-04"],
            "source_refs": [
                "new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"
            ],
        }]

    for kind, name in (
        ("floor-lifecycle", "delivery"), ("floor-lifecycle", "continuity"),
        ("floor-lifecycle", "remedy"), ("public-power-lifecycle", "source"),
        ("public-power-lifecycle", "limit"), ("public-power-lifecycle", "review"),
        ("public-power-lifecycle", "temporal-status"),
        ("private-duty-explicitness", "express-duty"),
        ("record-lifecycle", "writer"), ("record-lifecycle", "challenge"),
        ("record-lifecycle", "correction"),
        ("democratic-floor-corridor", "floor-boundary"),
    ):
        add(f"{kind}.{name} removed", lambda s, k=kind, n=name: component(s, k, n).update({"record_refs": []}))
    add("same-type wrong-lifecycle floor delivery",
        substitute_floor_delivery_with_wrong_lifecycle,
        "wrong typed contract")
    add("same-signature edge cannot replace reviewed floor delivery",
        substitute_floor_delivery_with_same_signature,
        "semantic edge contract")
    add("unused continuity edge cannot be reintroduced",
        lambda s: reintroduce_dead_component(
            s, "floor-lifecycle", "continuity", "FS-DEP-57"
        ), "no claim-scoped consumer")
    add("unused record writer cannot be reintroduced",
        lambda s: reintroduce_dead_component(
            s, "record-lifecycle", "writer", "FS-DEP-26"
        ), "no claim-scoped consumer")
    add("unused floor-boundary edge cannot be reintroduced",
        lambda s: reintroduce_dead_component(
            s, "democratic-floor-corridor", "floor-boundary", "FS-DEP-11"
        ), "no claim-scoped consumer")
    add("reader claim coverage moved", lambda s: (
        profile(s, "reader-claim-ownership").update({"applies_to_claim_refs": ["FS-CLM-36"]}),
        next(a for a in s["model_allocations"] if a["claim_ref"] == "FS-CLM-37")["closure_profile_refs"].remove("FS-CLR-08"),
        next(a for a in s["model_allocations"] if a["claim_ref"] == "FS-CLM-36")["closure_profile_refs"].append("FS-CLR-08"),
    ), "profile membership drifts from claim contracts")
    add("reader owner binding moved",
        lambda s: component(s, "reader-claim-ownership", "formal-owner").update({"record_refs": ["FS-CLM-36"]}),
        "formal-owner")
    add("external assumption hidden", lambda s: s["domains"][0].update({"external_assumption_refs": ["FS-EXA-99"]}))
    add("formal claim route substituted", lambda s: next(c for c in s["claims"] if c["id"] == "FS-CLM-04").update({"route_ref": "FS-RTE-04"}), "primary route drift")
    add("allocation and claim jointly substitute model", lambda s: (
        next(c for c in s["claims"] if c["id"] == "FS-CLM-04").update({"route_ref": "FS-RTE-04"}),
        next(a for a in s["model_allocations"] if a["claim_ref"] == "FS-CLM-04").update({"primary_route_ref": "FS-RTE-04", "required_route_refs": ["FS-RTE-04"]}),
    ), "required route composition drift")
    add("external claim jointly substituted to reader model",
        lambda s: jointly_substitute_route(s, "FS-CLM-13", "FS-RTE-06"),
        "required route composition drift")
    add("fully coordinated external-to-reader substitution",
        attach_external_claim_to_reader_route,
        "intrinsic claim obligation")
    add("external claim cannot be elevated onto formal route",
        lambda s: elevate_scoped_claim(s, "FS-CLM-16"),
        "external claims must remain Unestablished")
    add("Book 2 operation cannot be elevated onto formal route",
        lambda s: elevate_scoped_claim(s, "FS-CLM-24"),
        "Book 2 operation claims must remain Unestablished")
    add("live-record claim jointly substituted to quantitative model",
        lambda s: jointly_substitute_route(s, "FS-CLM-20", "FS-RTE-02"),
        "required route composition drift")
    add("composite route omitted", lambda s: next(a for a in s["model_allocations"] if a["claim_ref"] == "FS-CLM-24").update({"required_route_refs": ["FS-RTE-02", "FS-RTE-05"]}), "composition drift")
    add("Book 2 composite cannot be erased through legacy-row drift",
        lambda s: (
            next(c for c in s["claims"] if c["id"] == "FS-CLM-24").update({
                "legacy_row_ref": "FS-LGR-10", "route_ref": "FS-RTE-05"
            }),
            next(a for a in s["model_allocations"]
                 if a["claim_ref"] == "FS-CLM-24").update({
                     "primary_route_ref": "FS-RTE-05",
                     "required_route_refs": ["FS-RTE-05"],
                 }),
        ),
        "required route composition drift")
    add("model allocation omitted", lambda s: s["model_allocations"].pop())
    add("dependency satisfiability omitted", lambda s: s["dependencies"][0].pop("structural_satisfiability"))
    add("dependency status mismatched", lambda s: s["dependencies"][0]["structural_satisfiability"].update({"satisfiability_status": "external-contingent"}))
    add("unsatisfiable dependency has no defect", lambda s: s["dependencies"][0]["structural_satisfiability"].update({"satisfiability_status": "unsatisfiable"}), "requires a named defect")
    add("scenario ordinary route removed", lambda s: s["scenarios"][0].update({"ordinary_route": ""}))
    add("scenario failure route removed", lambda s: s["scenarios"][0].update({"failure_route": ""}))
    add("scenario recovery route removed", lambda s: s["scenarios"][0].update({"recovery_route": ""}))
    add("loop id made stale", lambda s: s["dependency_loops"][0].update({"id": "FS-LOP-99"}))
    add("loop takes allocation prefix", lambda s: (
        s["dependency_loops"][0].update({"id": "FS-MAL-99"}),
        s["loop_hazard_controls"][0].update({"loop_ref": "FS-MAL-99"}),
    ), "not 'dependency_loop'")
    add("loop hazard row removed", lambda s: s["loop_hazard_controls"].pop())
    add("loop hazard removed", lambda s: s["loop_hazard_controls"][0]["assessments"].pop())
    add("loop affected claims stale", lambda s: s["loop_hazard_controls"][0].update({"affected_claim_refs": []}), "affected-claim binding")
    add("generic anchor cannot reject a loop hazard", lambda s:
        s["loop_hazard_controls"][0]["assessments"][0].update({
            "closure_status": "rejected-by-control",
            "control_refs": ["new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"],
        }), "typed executable control receipt")
    add("bottleneck row removed", lambda s: s["bottleneck_dispositions"].pop())
    add("bottleneck affected claims stale", lambda s: s["bottleneck_dispositions"][0].update({"affected_claim_refs": []}), "affected-claim binding")
    add("generic anchor cannot reject a bottleneck", lambda s:
        s["bottleneck_dispositions"][0].update({
            "closure_status": "rejected-by-control",
            "control_refs": ["new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"],
        }), "typed executable control receipt")
    add("unrelated critical defect cannot block loop hazard", lambda s:
        s["loop_hazard_controls"][0]["assessments"][0].update({
            "closure_status": "open-blocking",
            "defect_refs": ["FS-DFT-16"],
        }), "claim-scoped component bindings")
    add("unrelated critical defect cannot block bottleneck", lambda s:
        s["bottleneck_dispositions"][0].update({
            "closure_status": "open-blocking",
            "defect_refs": ["FS-DFT-16"],
        }), "claim-scoped component bindings")
    add("dependency blocker must bind the exact claim component", lambda s:
        next(d for d in s["dependencies"] if d["id"] == "FS-DEP-28")[
            "structural_satisfiability"
        ].update({
            "satisfiability_status": "unsatisfiable",
            "defect_refs": ["FS-DFT-16"],
        }), "claim-scoped component bindings")
    add("bottleneck blocker must bind the exact claim component", lambda s:
        next(b for b in s["bottleneck_dispositions"]
             if b["dependency_ref"] == "FS-DEP-26").update({
                 "closure_status": "open-blocking",
                 "defect_refs": ["FS-DFT-28"],
             }), "claim-scoped component bindings")
    add("function inventory removed", lambda s: s.pop("function_allocations"))
    add("id-only powers cannot make function separation pass",
        fabricate_untyped_power_transition,
        "powers must contain every and only")
    add("role allocation drift", lambda s: next(r for r in s["roles"] if r["id"] == "FS-ROL-06").update({"domain_refs": ["FS-DOM-02", "FS-DOM-04"]}), "affected-claim binding")
    add("private-duty claim removed from all projections",
        lambda s: (
            remove_profile_projection(
                s, "private-duty-explicitness", "FS-CLM-34"
            ),
            claim_contract(s, "FS-CLM-34")[
                "required_profile_refs"
            ].remove("FS-CLR-03"),
        ),
        "intrinsic claim obligation")
    add("public-power claim removed from all projections",
        lambda s: (
            remove_profile_projection(
                s, "public-power-lifecycle", "FS-CLM-32"
            ),
            claim_contract(s, "FS-CLM-32")[
                "required_profile_refs"
            ].remove("FS-CLR-02"),
        ),
        "intrinsic claim obligation")
    add("record claim removed from all projections",
        lambda s: (
            remove_profile_projection(
                s, "record-lifecycle", "FS-CLM-31"
            ),
            claim_contract(s, "FS-CLM-31")[
                "required_profile_refs"
            ].remove("FS-CLR-04"),
        ),
        "intrinsic claim obligation")
    add("democratic claim removed from all projections",
        lambda s: (
            remove_profile_projection(
                s, "democratic-floor-corridor", "FS-CLM-10"
            ),
            claim_contract(s, "FS-CLM-10")[
                "required_profile_refs"
            ].remove("FS-CLR-05"),
        ),
        "intrinsic claim obligation")
    add("unsatisfiable dependency cannot borrow narrow blocker", lambda s:
        next(d for d in s["dependencies"] if d["id"] == "FS-DEP-01")[
            "structural_satisfiability"
        ].update({
            "satisfiability_status": "unsatisfiable",
            "defect_refs": ["FS-DFT-16"],
        }), "claim-scoped component bindings")
    add("authored closure result", lambda s: s["claims"][0].update({"closure_result": "pass"}))
    add("known defect loses disposition", lambda s: s["defects"][0].pop("defect_disposition"))
    add("known defect loses response stage", lambda s: s["defects"][0].pop("response_stage"))
    add("resolution hand-authored", lambda s: s["defects"][0].update({"resolution_status": "resolved-for-claim"}))
    add("required control removed", lambda s: next(d for d in s["defects"] if d["id"] == "FS-DFT-03").update({"controls": {}}))
    add("receipt exceeds ceiling", lambda s: s["receipts"][0].update({"assurance_ceiling": "Evidenced"}))
    add("narrow receipt promoted wide", lambda s: s["receipts"][0].update({"residuals": ["FS-DFT-41"]}))

    def assert_floor_delivery_gap(changed, label):
        append_control_audit(changed, "Closure-audit semantic control")
        changed["scope_audits"][-1]["scope_sha256"] = \
            LEDGER.review_scope_digest(changed)
        LEDGER.validate(changed)
        profiles = expanded_profiles(changed)
        validate_profile_bindings(changed, profiles)
        coverage = claim_component_coverage(changed, profiles)
        for claim_ref in ("FS-CLM-05", "FS-CLM-06"):
            if coverage[claim_ref]["floor-lifecycle.delivery"]:
                raise ClosureAuditError(
                    f"{label}: {claim_ref} retained unrelated delivery coverage"
                )
        controls.append(label)

    removed_delivery = copy.deepcopy(source)
    component(removed_delivery, "floor-lifecycle", "delivery")[
        "record_refs"
    ].remove("FS-DEP-25")
    next(d for d in removed_delivery["dependencies"] if d["id"] == "FS-DEP-25")[
        "closure_component_refs"
    ].remove("FS-CLR-01:delivery")
    assert_floor_delivery_gap(
        removed_delivery,
        "floor delivery removal exposes each uncovered floor claim",
    )

    resolution = LEDGER.compute_resolution(source)
    contract = validate_contract(source, resolution)
    rows = copy.deepcopy(contract[4])
    next(row for row in rows if row["id"] == "FS-CLM-05")["result"] = "pass"
    try:
        validate_generated_results(source, rows, resolution)
    except ClosureAuditError:
        controls.append("critical unresolved defect blocks its claim")
    else:
        raise ClosureAuditError("critical-block control did not fail")

    reordered = copy.deepcopy(source)
    reordered["loop_hazard_controls"][0]["assessments"].reverse()
    reordered_contract = validate_contract(
        reordered, LEDGER.compute_resolution(reordered)
    )
    if reordered_contract[5][0]["result"] != "bounded-unresolved":
        raise ClosureAuditError("owned cyclic loop lost its conservative classification")
    controls.append("bounded owned loop is classified, not rejected merely for cyclicity")

    scoped_dependency = copy.deepcopy(source)
    next(d for d in scoped_dependency["dependencies"] if d["id"] == "FS-DEP-25")[
        "structural_satisfiability"
    ].update({
        "satisfiability_status": "unsatisfiable",
        "defect_refs": ["FS-DFT-16"],
    })
    scoped_contract = validate_contract(
        scoped_dependency, LEDGER.compute_resolution(scoped_dependency)
    )
    scoped_claim = next(row for row in scoped_contract[4] if row["id"] == "FS-CLM-05")
    if not any("FS-DEP-25 is structurally unsatisfiable" in reason
               for reason in scoped_claim["reasons"]):
        raise ClosureAuditError("scoped unsatisfiable dependency did not block its claim")
    non_scoped_claim = next(
        row for row in scoped_contract[4] if row["id"] == "FS-CLM-04"
    )
    if (non_scoped_claim["result"] == "block"
            or not any("outside this claim's cited blocker scope" in reason
                       for reason in non_scoped_claim["reasons"])):
        raise ClosureAuditError("unsatisfiable dependency widened its blocker")
    controls.append("scoped unsatisfiable dependency propagates its critical blocker")
    controls.append("unscoped dependency claims remain bounded, not blocked")

    scoped_bottleneck = copy.deepcopy(source)
    next(row for row in scoped_bottleneck["bottleneck_dispositions"]
         if row["dependency_ref"] == "FS-DEP-23").update({
             "closure_status": "open-blocking",
             "defect_refs": ["FS-DFT-28"],
         })
    bottleneck_contract = validate_contract(
        scoped_bottleneck, LEDGER.compute_resolution(scoped_bottleneck)
    )
    btl = next(row for row in bottleneck_contract[6] if row["id"] == "FS-DEP-23")
    if btl["blocking_claim_refs"] != ["FS-CLM-20"]:
        raise ClosureAuditError("scoped bottleneck widened its critical blocker")
    non_scoped_claim = next(
        row for row in bottleneck_contract[4] if row["id"] == "FS-CLM-19"
    )
    if (non_scoped_claim["result"] == "block"
            or not any("outside this claim's cited blocker scope" in reason
                       for reason in non_scoped_claim["reasons"])):
        raise ClosureAuditError("open bottleneck widened its blocker")
    controls.append("scoped open bottleneck blocks only its defect claim")
    controls.append("unscoped bottleneck claims remain bounded, not blocked")

    scoped_loop = copy.deepcopy(source)
    loop_row = next(
        row for row in scoped_loop["loop_hazard_controls"]
        if row["loop_ref"] == "FS-LOP-03"
    )
    loop_row["assessments"][0].update({
        "closure_status": "open-blocking",
        "defect_refs": ["FS-DFT-16"],
    })
    loop_contract = validate_contract(
        scoped_loop, LEDGER.compute_resolution(scoped_loop)
    )
    loop = next(row for row in loop_contract[5] if row["id"] == "FS-LOP-03")
    if loop["blocking_claim_refs"] != ["FS-CLM-05"]:
        raise ClosureAuditError("scoped loop widened its critical blocker")
    non_scoped_claim = next(
        row for row in loop_contract[4] if row["id"] == "FS-CLM-04"
    )
    if (non_scoped_claim["result"] == "block"
            or not any("outside this claim's cited blocker scope" in reason
                       for reason in non_scoped_claim["reasons"])):
        raise ClosureAuditError("open loop widened its blocker")
    controls.append("scoped open loop blocks only its defect claim")
    controls.append("unscoped loop claims remain bounded, not blocked")

    fused_power = {
        "id": "FS-POW-999",
        "affected_claim_refs": ["FS-CLM-15"],
        "contract": {"required_separation_pairs": []},
    }
    fused = {
        "id": "FS-FAL-999", "power_ref": "FS-POW-999",
        "affected_claim_refs": ["FS-CLM-15"],
        "decisive_fact_writer_body_refs": ["FS-BOD-02"],
        "decisive_fact_writer_role_refs": ["FS-ROL-27"],
        "decider_body_refs": ["FS-BOD-02"],
        "decider_role_refs": ["FS-ROL-27"],
        "executor_body_refs": ["FS-BOD-02"],
        "executor_role_refs": ["FS-ROL-27"],
        "auditor_body_refs": ["FS-BOD-02"],
        "auditor_role_refs": ["FS-ROL-27"],
        "final_remedy_body_refs": ["FS-BOD-02"],
        "final_remedy_role_refs": ["FS-ROL-27"],
        "separation_constraints": [],
        "source_refs": ["new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"],
    }
    try:
        validate_function_separation_row(
            fused, by_id(source, "bodies"), by_id(source, "roles"),
            by_id(source, "claims"), {fused_power["id"]: fused_power},
            "synthetic fused function allocation",
        )
    except ClosureAuditError as exc:
        if "self-certifies" not in str(exc):
            raise
        controls.append("body cannot decide execute audit and finally remedy itself")
    else:
        raise ClosureAuditError("self-certification control did not fail")
    return len(controls)


def md_list(values):
    return ", ".join(f"`{value}`" for value in values) if values else "—"


def render(source, profiles, allocations, function_row, dependency_claims,
           rows, loops, bottlenecks, component_coverage, resolution):
    out = []
    write = out.append
    routes = by_id(source, "routes")
    row_by_id = {row["id"]: row for row in rows}
    write("<!-- SPDX-License-Identifier: CC-BY-4.0 -->")
    write("")
    write("# Constitutional-closure and model-allocation audit")
    write("")
    write(f"Canonical source: `{source['source_version']}`. This file is generated; edit `full-society-ledger.json`, never this report.")
    write("")
    write("**Verdict boundary:** this is a structural, claim-relative audit. `pass` means only that the reviewed contract for that exact claim survives the declared checks at its existing posture and scope. `block` and `bounded-unresolved` remain visible. No result upgrades posture or establishes delivery, liveness, feasibility, operation, external truth, reader response, constitutional completeness, or Gate A.")
    write("")
    write("## Model allocation")
    write("")
    write("Every claim has one reviewed primary route and an explicit all-of required-route set. Empty models remain visible; one green model never substitutes for another.")
    write("")
    write("| Route | Verification model | Status | Primary claims | Cannot warrant |")
    write("| --- | --- | --- | --- | --- |")
    for route_ref, model in MODEL_NAMES.items():
        claims = sorted(c for c, a in allocations.items() if a["primary_route_ref"] == route_ref)
        route = routes[route_ref]
        write(f"| {route_ref} | {model} | {route['route_status']} | {md_list(claims)} | {route['cannot_warrant']} |")
    write("")
    write("## Constitutional closure surfaces")
    write("")
    write("The bindings below use reviewed stable IDs and typed record contracts. Importance is never inferred from names, prose keywords, or counts.")
    write("")
    write("| Surface | Affected claims | Typed components | Result |")
    write("| --- | --- | --- | --- |")
    for kind, profile_row in profiles.items():
        results = {row_by_id[c]["result"] for c in profile_row["claims"]}
        result = "block" if "block" in results else ("bounded-unresolved" if "bounded-unresolved" in results else "pass")
        bindings = "; ".join(f"{k}: {md_list(v)}" for k, v in profile_row["components"].items())
        write(f"| {kind} | {md_list(profile_row['claims'])} | {bindings} | **{result}** |")
    write(f"| function separation | {md_list(function_row['affected_claim_refs'])} | {function_row['reason']} | **{function_row['result']}** |")
    write("")
    write("## Claim-by-claim audit")
    write("")
    write("| Claim | Result | Posture | Primary / required models | Profiles | Claim-scoped components | Defects / receipts | Reasons |")
    write("| --- | --- | --- | --- | --- | --- | --- | --- |")
    for row in rows:
        components = "; ".join(
            f"{name}: {md_list(refs)}"
            for name, refs in sorted(row["component_coverage"].items())
        ) or "— (no applicable closure profile)"
        write(f"| {row['id']} {row['title']} | **{row['result']}** | {row['posture']} | {row['route']} / {md_list(row['required_routes'])} | {md_list(row['profiles'])} | {components} | {md_list(row['defects'])} / {md_list(row['receipts'])} | {'; '.join(row['reasons'])} |")
    write("")
    write("## Role, dependency, scenario, and external-assumption joins")
    write("")
    power_roles = [r["id"] for r in source["roles"] if "power_held" in r]
    write(f"The projection consumes **{len(source['roles'])}** reviewed role records, including power-holding roles {md_list(power_roles)}. Role-to-domain assignments feed the dependency impact join; they do not create a duty or establish performance.")
    write("")
    for row in rows:
        write(f"- `{row['id']}` — roles {md_list(row['roles'])}; dependencies {md_list(row['dependencies'])}; scenarios {md_list(row['scenarios'])}; external assumptions {md_list(row['external_assumptions'])}.")
    write("")
    write("### Dependency structural satisfiability")
    write("")
    write("These are interface classifications, not arrival, capacity, timing, or liveness evidence.")
    write("")
    write("| Dependency | Status | Affected claims | Reason |")
    write("| --- | --- | --- | --- |")
    for dependency in source["dependencies"]:
        sat = dependency["structural_satisfiability"]
        write(f"| {dependency['id']} | {sat['satisfiability_status']} | {md_list(dependency_claims[dependency['id']])} | {sat['reason']} |")
    write("")
    write("## Cycles and bottlenecks")
    write("")
    write("Cyclicity is not itself a defect. Reviewed boundedness prose is not a control; each stable loop carries all five typed hazard dispositions.")
    write("")
    write("| Loop | Kind | Members | Result | Hazard statuses | Affected claims |")
    write("| --- | --- | --- | --- | --- | --- |")
    for loop in loops:
        statuses = "; ".join(f"{h}: {s}" for h, s in loop["statuses"].items())
        write(f"| {loop['id']} | {loop['kind']} | {md_list(loop['members'])} | **{loop['result']}** | {statuses} | {md_list(loop['affected_claim_refs'])} |")
    write("")
    write("| Bottleneck edge | Result | Owner | Affected claims | Reviewed reason |")
    write("| --- | --- | --- | --- | --- |")
    for row in bottlenecks:
        write(f"| {row['id']} | **{row['result']}** | `{row['owner']}` | {md_list(row['affected_claim_refs'])} | {row['reason']} |")
    write("")
    write("## Defect disposition, response stage, history, and receipts")
    write("")
    write("Generated resolution remains claim-relative and cannot exceed the affected claim's posture or route ceiling.")
    write("")
    write("| Defect | Claim | Disposition | Stage | Generated resolution | Blocks | History | Receipt |")
    write("| --- | --- | --- | --- | --- | --- | --- | --- |")
    receipt_by_defect = {r["defect_row_ref"]: r["id"] for r in source["receipts"]}
    for defect in source["defects"]:
        history = "; ".join(f"{h['date']} {h['field']}={h['value']}" for h in defect["history"]) or "no transition recorded"
        generated = resolution[defect["id"]]
        write(f"| {defect['id']} | {defect['affected_claim_ref']} | {defect['defect_disposition']} | {defect['response_stage']} | {generated['resolution_status']} | {'yes' if generated['blocking'] else 'no'} | {history} | {receipt_by_defect.get(defect['id'], '—')} |")
    write("")
    write("### Resolution receipts")
    write("")
    for receipt in source["receipts"]:
        write(f"- **{receipt['id']} / {receipt['affected_claim_ref']}:** {receipt['now_follows']} **Still does not follow:** {receipt['still_does_not_follow']} Residuals: {md_list(receipt['residuals'])}.")
    write("")
    write("## Reproduce")
    write("")
    write("```bash")
    write("python3 new-book-plans/16-constitutional-closure.py --check")
    write("```")
    write("")
    return "\n".join(out)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    source = LEDGER.load_json(LEDGER.SOURCE)
    resolution = LEDGER.validate(source)
    contract = validate_contract(source, resolution)
    controls = negative_controls(source)
    rendered = render(source, *contract[:4], *contract[4:], resolution)
    gate_status = source["acceptance_gate"]["gate_a_status"]
    if args.check:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.exists() else ""
        if current != rendered:
            raise ClosureAuditError(f"{OUTPUT.relative_to(ROOT)} is STALE — rerun without --check")
        print(f"{OUTPUT.relative_to(ROOT)} is current; {controls} watched-failing structural controls pass; claim results are contract-only; Gate A {gate_status}")
    else:
        OUTPUT.write_text(rendered, encoding="utf-8")
        print(f"wrote {OUTPUT.relative_to(ROOT)}; {controls} watched-failing structural controls pass; Gate A {gate_status}")


if __name__ == "__main__":
    try:
        main()
    except (ClosureAuditError, LEDGER.LedgerError) as exc:
        print(f"16-constitutional-closure: {exc}", file=sys.stderr)
        sys.exit(1)
