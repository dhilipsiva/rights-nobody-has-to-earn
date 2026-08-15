#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Migrate one exact full-society source-family prefix to schema v7.

This is a reviewed-source migration helper, not an assurance route.  Script 13
remains authoritative and must validate every emitted record.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib


ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT / "new-book-plans/full-society-ledger.json"
MANIFEST = ROOT / "new-book-plans/full-society-power-source-manifest.json"
VALIDATOR = ROOT / "new-book-plans/13-full-society-ledger.py"


def load_validator():
    spec = importlib.util.spec_from_file_location("full_society_ledger", VALIDATOR)
    if spec is None or spec.loader is None:
        raise SystemExit("cannot load script 13")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def term(text, source_ref, *, basis="source-specified", delegation=None):
    value = {
        "text": text,
        "basis": basis,
        "source_refs": [source_ref],
    }
    if basis == "bounded-delegation":
        value.update(delegation)
    return value


EFFECT_REPAIRS = {
    "gov-appointments-qualification-function": (
        "A dedicated appointments-qualification function held only by the "
        "card's named body and roles; audit cannot acquire it by implication."
    ),
    "gov-assembly-term-vacancy-election": (
        "One Assembly-specific term, vacancy, and early-election lifecycle "
        "choice inside the source bounds; no choice exists by silence."
    ),
    "gov-council-tenure-replacement": (
        "One Regions Council tenure, instruction, replacement, and vacancy "
        "lifecycle choice inside the source bounds; no choice exists by silence."
    ),
    "gov-executive-composition-replacement": (
        "One executive composition and succession lifecycle choice inside the "
        "source bounds; no choice exists by silence."
    ),
    "gov-president-selection-removal-mechanics": (
        "One finite presidential selection, alternate, and removal-confirmation "
        "choice inside the source bounds; no choice exists by silence."
    ),
    "eco-no-fault-protection-cessation": (
        "One immediate protective act while liability remains separately "
        "undecided; it establishes neither civil liability nor personal guilt."
    ),
}


def effect_grain(row):
    return EFFECT_REPAIRS.get(
        row["provisional_key"], row["legal_effect_and_grain"])


CORE_LEADS = {
    "lawful_source": "Authority exists only under the cited constitutional source",
    "trigger": "Legal effect begins only after the cited source premises are authenticated",
    "evidence_rule": "The decision record must establish the effect-specific premises and may not substitute labels or silence",
    "bounded_effect": "The holder may produce only the direct legal effect described by the source",
    "public_reasons": "Public reasons must connect the cited source, admitted facts, affected holder, and exact effect",
    "conflict_rule": "A conflicted holder withdraws and only a source-authorised alternate may proceed",
    "non_delegable_limit": "Delegation cannot enlarge the effect, evade a wall, or transfer final review to the holder",
    "independent_review": "Independent review reaches source, trigger, evidence, scope, reasons, execution, and temporal currency",
    "appeal": "An affected holder keeps the source-provided challenge route and interim protection while it is decided",
    "correction": "A superseded decision and each consequential record must be corrected without replay",
    "remedy": "Unlawful effect is withheld or stopped and attributable harm remains separately remediable",
    "end_condition": "Authority ends on the source-defined expiry, invalidation, supersession, or failed renewal",
    "temporal_status": "Currency depends on this power's own source record and never on the retained custody T3 record",
    "failure_polarity": "Missing, stale, conflicted, or unauthenticated positive premises withhold authority and create no opposite fact",
}


def delegated(row):
    text = (row["title"] + " " + row["legal_effect_and_grain"]).lower()
    return (
        row["provisional_key"] in {
            "gov-assembly-term-vacancy-election",
            "gov-council-tenure-replacement",
            "gov-executive-composition-replacement",
            "gov-president-selection-removal-mechanics",
        }
        or any(token in text for token in (
            "ordinary law", "ordinary-law", "legislative", "legislature",
            "democratic law", "enabling law", "regulation",
        ))
    )


def delegation_contract(row):
    family = row["source_family"]
    if family == "state-form-and-political-membership":
        owner = "People's Assembly or the constitutionally named joint chamber"
    elif family == "economic-pluralism-and-protected-private-sphere":
        owner = "People's Assembly acting through ordinary law"
    elif family == "family-dependency-reproduction-and-collective-plurality":
        owner = "People's Assembly acting inside the protected family and agency corridor"
    elif family == "ecological-commons-and-non-human-animal":
        owner = "People's Assembly or the narrowly named ecological or animal regulator"
    else:
        owner = "the source-named democratic authoriser"
    return {
        "choice_owner": owner,
        "bounds": effect_grain(row),
        "failure_default": (
            "No legal effect exists until the named owner makes and records a "
            "choice inside these bounds; silence supplies no authority."
        ),
    }


def profile_semantics(field):
    label = field.replace("_", " ")
    if any(token in field for token in (
            "evidence", "writer", "source", "basis", "record")):
        lead = "admits only authenticated and contestable evidence for"
    elif any(token in field for token in (
            "review", "audit", "reader", "challenge", "appeal")):
        lead = "keeps an independent challenge or checking route for"
    elif any(token in field for token in (
            "remedy", "correction", "restoration", "repair", "relief")):
        lead = "requires cessation, correction, and effect-specific repair for"
    elif any(token in field for token in (
            "end", "temporal", "renewal", "reassessment", "retention")):
        lead = "ends or renews only on a current source-bound record for"
    elif any(token in field for token in (
            "privacy", "visibility", "deletion")):
        lead = "limits visibility and consequential reuse to what is necessary for"
    elif any(token in field for token in (
            "continuity", "carry", "rescue", "interim")):
        lead = "preserves the protected condition while deciding"
    elif any(token in field for token in (
            "actor", "office", "holder", "authority", "responsible")):
        lead = "identifies the source-authorised holder and excludes inherent authority over"
    elif any(token in field for token in (
            "conflict", "alternate", "separation", "substitute")):
        lead = "prevents conflicted self-authorisation and names the bounded alternate for"
    else:
        lead = "fixes the admissible constitutional boundary for"
    return label, lead


def migrate_power(module, power, row):
    source_ref = row["source_anchor"]
    grain = effect_grain(row)
    title = row["title"]
    retained = row["provisional_key"] == module.RETAINED_FORMAL_KEY
    profiles = module._power_profiles(row)
    contract_terms = {}
    for field in module.POWER_CONTRACT_TERM_KEYS:
        basis = "current-derived" if retained else "source-specified"
        extra = None
        if not retained and delegated(row) and field in {"trigger", "bounded_effect"}:
            basis = "bounded-delegation"
            extra = delegation_contract(row)
        contract_terms[field] = term(
            f"For {title}, {CORE_LEADS[field].lower()}: {grain}",
            source_ref,
            basis=basis,
            delegation=extra,
        )
    profile_terms = {}
    for profile in profiles:
        profile_terms[profile] = {}
        for field in module.POWER_PROFILE_FIELDS[profile]:
            label, lead = profile_semantics(field)
            profile_terms[profile][field] = term(
                f"Within the {profile} profile for {title}, the {label} term {lead}: {grain}",
                source_ref,
                basis="current-derived" if retained else "source-specified",
            )

    old_contract = power.pop("contract")
    power.pop("profile_contracts")
    power["profiles"] = profiles
    power["primary_class_ref"] = module._power_primary_class(row)
    power["secondary_class_refs"] = module._power_secondary_classes(row, profiles)
    power["affected_claim_refs"] = module._power_claim_refs(row)
    power["domain_refs"] = None  # filled from canonical claims below
    power["contract_terms"] = contract_terms
    power["profile_terms"] = profile_terms
    power["required_separation_pairs"] = old_contract["required_separation_pairs"]
    expected_pairs = module._power_required_separations(profiles)
    power["required_separation_pairs"] = expected_pairs
    power["permitted_inputs"] = [
        f"Authenticated current source record for {title}",
        f"Contestable evidence of the specific premises for {grain}",
        f"A holder-and-scope record limited to {power['manifest_key']}",
    ]
    power["prohibited_inputs"] = [
        f"Labels, silence, status, or institutional name alone for {title}",
        f"A stale, superseded, conflicted, or unauthenticated record for {title}",
        f"The formal-active-custody T3 record borrowed for {power['manifest_key']}",
    ]
    power["permitted_downstream_effects"] = [grain]
    power["evidence_authority"] = term(
        f"For {title}, only the cited source and its authenticated premises may establish this bounded legal effect: {grain}",
        source_ref,
        basis="current-derived" if retained else "source-specified",
    )
    executable = retained
    power["negative_test"] = {
        "id": f"{power['id']}-NEGATIVE",
        "status": "executable" if executable else "planned",
        "assertion": (
            f"Withhold one required premise for {title}; the direct effect must not arise and no opposite fact may be manufactured."
        ),
        "source_refs": [source_ref],
        "executable_ref": (
            "new-book-plans/12-temporal-assurance.py::"
            "def negative_controls(reviewed: Mapping[str, object], constitution: str, dependencies: Mapping[str, bytes]) -> int:"
            if executable else None
        ),
    }
    power["counterfactual"] = {
        "id": f"{power['id']}-COUNTERFACTUAL",
        "status": "executable" if executable else "planned",
        "assertion": (
            f"Replace the source, holder, scope, or temporal record for {title}; the original direct effect must remain unavailable."
        ),
        "source_refs": [source_ref],
        "executable_ref": (
            "new-book-plans/temporal-assurance-case.json::"
            '"case_bound_rule_fragment":'
            if executable else None
        ),
    }
    power["part_v_status"] = (
        "implemented-current-formal" if retained
        else "coverage-only-not-formalized"
    )
    power["book2_handoff"] = (
        f"Book 2 may design staffing, capacity, workflow, and evidence custody for {title}; "
        "no operation, delivery, feasibility, liveness, or calibration follows from this card."
    )


def refusal_claims(module, row, refusal, powers):
    by_id = {power["id"]: power for power in powers}
    result = []
    for power_ref in refusal["affected_power_refs"]:
        for claim_ref in by_id[power_ref]["affected_claim_refs"]:
            if claim_ref not in result:
                result.append(claim_ref)
    if result:
        return result
    if row["source_family"] == "time-model":
        return ["FS-CLM-19"]
    return module._power_claim_refs(row)


def main():
    module = load_validator()
    parser = argparse.ArgumentParser()
    parser.add_argument("family", choices=module.COVERAGE_SOURCE_FAMILY_ORDER)
    args = parser.parse_args()
    source = json.loads(SOURCE.read_text(encoding="utf-8"))
    completed = source["coverage_population"]["completed_source_families"]
    expected = module.COVERAGE_SOURCE_FAMILY_ORDER[len(completed)]
    if args.family != expected:
        raise SystemExit(f"next family must be {expected!r}, got {args.family!r}")
    manifest_rows = json.loads(MANIFEST.read_text(encoding="utf-8"))["rows"]
    manifest = {row["provisional_key"]: row for row in manifest_rows}
    claims = {row["id"]: row for row in source["claims"]}

    for power in source["powers"]:
        row = manifest[power["manifest_key"]]
        if row["source_family"] != args.family:
            continue
        migrate_power(module, power, row)
        domains = []
        for claim_ref in power["affected_claim_refs"]:
            for domain_ref in claims[claim_ref]["domain_refs"]:
                if domain_ref not in domains:
                    domains.append(domain_ref)
        power["domain_refs"] = domains

    powers_by_id = {power["id"]: power for power in source["powers"]}
    for allocation in source["function_allocations"]:
        power = powers_by_id[allocation["power_ref"]]
        row = manifest[power["manifest_key"]]
        if row["source_family"] == args.family:
            allocation["affected_claim_refs"] = list(
                power["affected_claim_refs"])

    if args.family == "time-model":
        template = source["power_contract_templates"][0]
        row = manifest[template["manifest_key"]]
        old = template.pop("contract")
        template["contract_terms"] = {
            field: term(
                f"For the power-specific T3 template, {field.replace('_', ' ')} is limited to the cited temporal rule: {text}",
                row["source_anchor"],
            )
            for field, text in old.items()
        }

    for refusal in source["power_refusals"]:
        row = manifest[refusal["manifest_key"]]
        if row["source_family"] != args.family:
            continue
        refusal["affected_claim_refs"] = refusal_claims(
            module, row, refusal, source["powers"])
        domains = []
        for claim_ref in refusal["affected_claim_refs"]:
            for domain_ref in claims[claim_ref]["domain_refs"]:
                if domain_ref not in domains:
                    domains.append(domain_ref)
        refusal["domain_refs"] = domains

    family_row = next(
        row for row in source["coverage_families"]
        if row["source_family_refs"] == [args.family]
    )
    family_row["state"] = "coverage-ready"
    family_row["card_refs"] = [
        power["id"] for power in source["powers"]
        if manifest[power["manifest_key"]]["source_family"] == args.family
        and power["manifest_key"] != module.RETAINED_FORMAL_KEY
    ]
    family_row["refusal_refs"] = [
        row["id"] for row in source["power_refusals"]
        if manifest[row["manifest_key"]]["source_family"] == args.family
    ]
    family_row["crosswalk_refs"] = [
        row["id"] for row in source["power_crosswalk_dispositions"]
        if manifest[row["manifest_key"]]["source_family"] == args.family
    ]

    temporal_family = next(
        row for row in source["coverage_families"]
        if row["id"] == "FS-CVF-002"
    )
    if args.family == "time-model":
        temporal_family["template_refs"] = [
            source["power_contract_templates"][0]["id"]
        ]
    if args.family == "current-formal-constitution":
        temporal_family["card_refs"] = [
            next(power["id"] for power in source["powers"]
                 if power["manifest_key"] == module.RETAINED_FORMAL_KEY)
        ]

    completed.append(args.family)
    source["coverage_population"]["status"] = (
        "complete" if completed == module.COVERAGE_SOURCE_FAMILY_ORDER
        else "partial"
    )
    source["source_version"] = (
        "fs-ledger-2026-08-15-coverage-contracts-v7-"
        + args.family
    )
    if completed == module.COVERAGE_SOURCE_FAMILY_ORDER:
        source["deferred_populations"] = [
            row for row in source["deferred_populations"]
            if row["record_type"] != module.COVERAGE_DEFERRAL_TYPE
        ]
    SOURCE.write_text(
        json.dumps(source, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
