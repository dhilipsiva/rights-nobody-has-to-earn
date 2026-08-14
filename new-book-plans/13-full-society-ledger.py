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
compatibility table, and the mechanical enum mapping over the seven sibling
reviewed JSONs (which are read LIVE at --check, never digest-pinned, so a new
reviewed enum value fails here until its mapping row lands in the same change).
Stage 2 backfills every declared defect, repair narrative, open gap, and
sibling residual as keyed FS-DFT rows under a live-read citation closure,
computes resolution_status and blocking (never hand-authored), and mints
resolution receipts only where a row's generated resolution permits one.
Stage 3's first population is landed: the roles, life-course, scale, and
power-position matrix (FS-ROL) with domain, scale, and body-position closures
and recorded risk-based omissions; a role is never a floor-changing status.
Stage 3's second population, the functional-flow and dependency map (FS-DEP),
types every edge's endpoints, classes it by lawful source with the layer
derived, pins the three ratified lifecycle paths without flattening them,
requires a predeclared alternate route or its recorded absence, gives every
strongly connected region a declared classified loop witness, and records
the refused-flow walls; an edge is routing, never delivery.
Stage 3's third population, the whole-society scenario catalogue (FS-SCN),
records journeys, stress cases, the named collision axes, and the named
compound shocks as reviewed inventory: every domain reached, every critical
dependency edge stressed or its omission recorded, each record carrying an
owned ordinary, failure, and recovery route and the protected sphere tested
without a state-defined successful outcome; a route is routing, never
delivery, and no scenario claims execution — constitutional cases execute
only after the relevant author rulings and contract cards land.
Stage 4's machinery makes the future independent scope review admissible and
the Gate A closure refusable-until-green: proposal and review-event schemas
stand validated and empty, the severity rubric and the scope-review protocol
are author-confirmed with their bases recorded and the protocol's status line
live-checked against the reviewed source, an independent review
event is refused without a pre-registered commitment (plant, seed, and
protocol SHA-256 digests entered at commissioning; pre-images author-held
outside the repository; a commitment exists only against a confirmed
protocol), per-condition Gate A readiness is computed and
rendered, and a closure record is refused while any condition computes unmet —
closing Gate A is a deliberate future contract amendment, never a latent flip.
Deferred record types carry explicit deferral records with owners.

Usage:
  python3 new-book-plans/13-full-society-ledger.py            # regenerate MD
  python3 new-book-plans/13-full-society-ledger.py --check    # validate + fresh
"""

import argparse
import copy
import hashlib
import importlib.util
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE = pathlib.Path("new-book-plans/full-society-ledger.json")
OUTPUT = pathlib.Path("new-book-plans/full-society-ledger.md")
READER_OUTPUT = pathlib.Path("new-book-plans/full-society-reader-ledger.md")
POWER_SOURCE_MANIFEST = pathlib.Path(
    "new-book-plans/full-society-power-source-manifest.json"
)
POWER_SOURCE_FAMILY_ORDER = [
    "state-form-and-political-membership",
    "time-model",
    "substantive-equality-and-anti-subordination",
    "economic-pluralism-and-protected-private-sphere",
    "family-dependency-reproduction-and-collective-plurality",
    "ecological-commons-and-non-human-animal",
    "public-safety-defence-emergency-and-external-power",
    "current-formal-constitution",
]
POWER_FAMILY_CUMULATIVE_COUNTS = {
    "state-form-and-political-membership": {
        "powers": 51, "templates": 0, "refusals": 0, "crosswalks": 0,
    },
    "time-model": {
        "powers": 51, "templates": 1, "refusals": 1, "crosswalks": 0,
    },
    "substantive-equality-and-anti-subordination": {
        "powers": 60, "templates": 1, "refusals": 1, "crosswalks": 0,
    },
    "economic-pluralism-and-protected-private-sphere": {
        "powers": 88, "templates": 1, "refusals": 2, "crosswalks": 0,
    },
    "family-dependency-reproduction-and-collective-plurality": {
        "powers": 119, "templates": 1, "refusals": 2, "crosswalks": 0,
    },
    "ecological-commons-and-non-human-animal": {
        "powers": 159, "templates": 1, "refusals": 5, "crosswalks": 0,
    },
    "public-safety-defence-emergency-and-external-power": {
        "powers": 209, "templates": 1, "refusals": 19, "crosswalks": 0,
    },
    "current-formal-constitution": {
        "powers": 210, "templates": 1, "refusals": 19, "crosswalks": 8,
    },
}
POWER_FINAL_COUNTS = {
    "powers": 210,
    "templates": 1,
    "refusals": 19,
    "crosswalks": 8,
    "function_allocations": 210,
}
POWER_POPULATION_STATUSES = ["foundation", "partial", "complete"]
POWER_POSTURES = ["Specified", "Derived"]
POWER_PROFILE_ORDER = [
    "ordinary-public-power",
    "liberty-power-limit",
    "coercive-protective",
    "emergency",
    "commons-future-condition",
    "non-human-animal",
    "collective-authority-title-consent",
    "consequential-status-supported-decision",
    "economic-private-power-limit",
    "physical-scarcity",
    "consequential-record",
]
POWER_PROFILE_FIELDS = {
    "ordinary-public-power": [
        "office", "democratic_source", "trigger", "evidence", "scope",
        "conflict_rule", "non_delegable_limit", "review", "appeal",
        "end_condition", "temporal_status",
    ],
    "liberty-power-limit": [
        "protected_person", "prohibited_act", "direct_public_binding",
        "private_interference_duty", "explicit_private_binding",
        "narrow_exception_test", "evidence", "independent_reviewer",
        "public_reason", "review_or_end", "temporal_status", "remedy",
    ],
    "coercive-protective": [
        "named_instrument", "authorised_actor_and_mandate",
        "individualised_recorded_ground", "identification_and_reasons",
        "prior_authorisation_and_immediate_danger_exception",
        "necessity_least_harm_warning_cessation_aid",
        "public_actor_burden", "capability_removal_and_no_reader",
        "upstream_personhood_and_floor_refusals",
        "counsel_interpreter_accommodation", "third_party_notification",
        "prompt_automatic_review", "investigator_deployer_separation",
        "non_punitive_and_no_consequential_feed",
        "shelter_and_recorded_voice", "record_contract",
        "failure_polarity", "remedy", "audit",
        "source_bound_temporal_contract",
    ],
    "emergency": [
        "hazard_territory_affected_people", "separately_justified_powers",
        "narrowest_scope", "published_reasons_and_evidence",
        "cross_branch_authorisation", "alternate_authorising_route",
        "independent_substitute_reviewer", "ordinary_body_ratification",
        "legislature_and_court_notice", "declaration_temporal_contract",
        "renewal_temporal_contract", "exact_declaration_version_join",
        "judicial_review", "non_conferred_list", "non_derogable_core",
        "restoration", "post_hoc_audit", "lawful_and_unlawful_harm_remedy",
        "failure_polarity_and_frozen_record_limit",
    ],
    "commons-future-condition": [
        "protected_common_and_baseline", "public_and_private_duty",
        "qualitative_non_destruction_ceiling",
        "quantitative_standard_source_and_version",
        "independent_evidence_cumulative_effects_uncertainty_precaution",
        "functional_non_regression", "avoid_minimise_restore_hierarchy",
        "independent_initiators", "judicial_interim_relief",
        "guardian_only_pause", "guardian_appointment_and_conflict",
        "evidence_source_separation", "case_bound_replay_key",
        "alternate_advocate", "substitute_reviewer", "no_final_veto",
        "indigenous_title_and_consent", "prevention_and_cessation",
        "hazardous_activity_liability", "other_causation_and_control",
        "personal_culpability_due_process", "breach_and_interim_protection",
        "restoration", "public_accountability", "review",
        "temporal_status",
    ],
    "non-human-animal": [
        "protected_animal_or_category", "sentience_evidence_and_uncertainty",
        "responsible_actor", "welfare_minimum_and_prohibited_harm",
        "corridor_core", "permitted_use_category", "food_alternatives_test",
        "research_three_rs_and_categorical_ban", "enhanced_use_test",
        "species_appropriate_care", "independent_inspection_and_writer",
        "advocate_and_public_initiation", "challenge",
        "rescue_and_continuity", "licence_or_custody_correction", "remedy",
        "audit", "temporal_status", "commons_cross_reference",
    ],
    "collective-authority-title-consent": [
        "individual_and_collective_holders", "source_and_basis",
        "acceptance_or_decision_rule", "writer", "evidence",
        "permitted_legal_effect", "independent_reader",
        "privacy_and_dissent", "challenge_and_appeal", "correction",
        "carry", "end_or_review", "alternate_route", "continuity",
        "remedy", "failure_polarity", "constitutional_court_route",
    ],
    "consequential-status-supported-decision": [
        "holder", "source", "writer", "evidence_rule",
        "scope_and_permitted_effects", "voice_and_preferences",
        "required_support", "independent_reader_or_decider",
        "privacy_boundary", "challenge", "correction", "carry",
        "end_condition", "alternate_route", "continuity", "remedy",
        "failure_polarity",
    ],
    "economic-private-power-limit": [
        "holder_or_actor", "protected_permission",
        "public_or_private_trigger", "forbidden_waiver",
        "public_scale_finding_if_required", "evidence",
        "necessity_and_proportionality", "continuity", "challenge",
        "remedy", "temporal_status",
    ],
    "physical-scarcity": [
        "named_resource_and_population", "positive_scarcity_evidence",
        "alternatives_considered", "responsible_authority",
        "constitutional_minima_preserved", "no_reduced_redefinition",
        "every_shortfall_as_failure", "urgency_accessibility_irreversibility",
        "continuity_and_resource_benefit", "anti_proxy_test",
        "forbidden_priority_keys", "equal_claim_rotation_or_lottery",
        "interim_alternative", "review", "reassessment",
        "source_bound_end", "repair",
    ],
    "consequential-record": [
        "writer", "permitted_basis", "visibility_and_privacy", "challenge",
        "correction", "retention", "deletion_control", "external_assurance",
        "independent_recipient", "action_duty", "continuity_and_remedy",
        "temporal_status",
    ],
}

POWER_CONTRACT_KEYS = [
    "lawful_source", "trigger", "evidence_rule", "bounded_effect",
    "public_reasons", "conflict_rule", "non_delegable_limit",
    "independent_review", "appeal", "correction", "remedy",
    "end_condition", "temporal_status", "failure_polarity",
    "required_separation_pairs",
]
POWER_TEMPLATE_CONTRACT_KEYS = [
    "current_source", "scope", "review", "renewal_or_end", "challenge",
    "fail_closed_polarity", "frozen_record_limit", "book2_liveness_limit",
]
POWER_FUNCTIONS = [
    "decisive-fact-writer", "decider", "executor", "auditor",
    "final-remedy",
]
UNIVERSAL_SEPARATION_PAIRS = [
    ["decisive-fact-writer", "auditor"],
    ["executor", "auditor"],
    ["auditor", "final-remedy"],
]
CROSSWALK_POLICY = {
    "formal-electorate-seating-authority": "replace",
    "formal-public-body-authority": "replace",
    "formal-review-credential": "retire",
    "formal-tribunal-credential": "retire",
    "formal-appeals-expungement": "retire",
    "formal-appeals-relief": "replace",
    "formal-active-custody": "retain",
    "formal-amendment-label-result": "retire",
}
RETAINED_FORMAL_KEY = "formal-active-custody"
POWER_TEMPLATE_KEY = "time-power-specific-t3-contract"
POWER_BOOK2_OWNER = (
    "TODO.md::Every Book 1 domain card must nevertheless name its Book 2 "
    "operator/evidence owner"
)

POWER_SOURCE_BINDING = {
    "artifact_ref": str(POWER_SOURCE_MANIFEST),
    "artifact_sha256": (
        "4525badc3fbc4fdd8ab0a7bbf792e42409b740ea3ff90da036bd32531f358adf"
    ),
    "source_commit": "36ed92c58877cffa5a11928ad200f0ca9a604820",
    "inventory_status": (
        "reviewed-inventory-input-not-law-not-operation-"
        "not-completeness-beyond-bound-version"
    ),
    "row_count": 237,
    "disposition_counts": {
        "card-required": 209,
        "power-contract-template": 1,
        "existing-formal-crosswalk": 8,
        "explicit-refusal-limit": 19,
    },
    "power_population_status": "deferred-contract-cards-bodies-and-allocations",
    "known_allocation_gaps": [
        "appointments-qualification function and its nominee, selector, and qualification positions",
        "custodial execution function distinct from policing",
        "independent ecological science and assessment function",
        "ecological and animal regulation and inspection functions",
        "emergency alternate authoriser and independent substitute reviewer",
        "Guardian alternate advocate and substitute reviewer",
        "border and removal execution function",
    ],
    "owner_ref": "TODO.md::Maintain completed constitutional coverage rows",
    "closure_condition": (
        "complete per-instrument FS-POW contract cards, lawful body and role "
        "allocations, and power-bound decider, executor, auditor, and final-remedy "
        "separation rows for every card-required and retained formal entry"
    ),
    "scope_ceiling": (
        "Source-bound candidate census only: no row creates law, a complete "
        "contract card, a lawful holder, operation, assurance, FS-POW population "
        "completion, or Gate A passage. The cross-power temporal template "
        "creates no holder, power, or function allocation."
    ),
}

# The two rulings whose text defines the ledger's enums and stopping rule. An
# edit there must force a deliberate refresh here. The seven sibling reviewed
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

READER_EVIDENCE_SOURCE = pathlib.Path("new-book-plans/reader-evidence.json")
READER_EVIDENCE_VALIDATOR = pathlib.Path("new-book-plans/14-reader-evidence.py")

SIBLING_SOURCES = [
    pathlib.Path("new-book-plans/assertion-surface-contracts.json"),
    pathlib.Path("new-book-plans/record-integrity-assurance-case.json"),
    pathlib.Path("new-book-plans/record-integrity-red-team.json"),
    pathlib.Path("new-book-plans/amendment-semantics-audit.json"),
    pathlib.Path("new-book-plans/placement-exhaustiveness-audit.json"),
    pathlib.Path("new-book-plans/temporal-assurance-case.json"),
    READER_EVIDENCE_SOURCE,
]

# ID prefixes already in use across the seven sibling reviewed JSONs. A ledger ID
# may never collide with this space; the ledger's own IDs are FS-XXX-NN.
LIVE_SIBLING_PREFIXES = frozenset(
    ["AS", "OE", "RA", "RC", "RD", "RE", "RF", "RI", "RS", "RT", "TA", "TP"]
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
GATE_REFS = ["gate-a", "gate-b", "gate-c", "gate-d", "gate-e"]
GATE_APPLICABILITY_MEANINGS = {
    "gate-a": "versioned scope map and assurance test program",
    "gate-b": "Book 1 constitutional and social destination",
    "gate-c": "tested public Book 1 edition",
    "gate-d": "Book 2 operational model in a calibrated envelope",
    "gate-e": "integrated Book 1 and Book 2 pair",
}
# This is the validator-owned inverse of the reviewed per-defect field. The
# boundary decision makes applicability gate-relative; the exact groups below
# prevent a coordinated source-only edit from hiding a critical defect.
DEFECT_GATE_GROUPS = {
    tuple(GATE_REFS): frozenset({
        "FS-DFT-13", "FS-DFT-14", "FS-DFT-27", "FS-DFT-40",
    }),
    ("gate-c", "gate-d", "gate-e"): frozenset({"FS-DFT-20"}),
    ("gate-d", "gate-e"): frozenset({
        "FS-DFT-17", "FS-DFT-28", "FS-DFT-29",
        "FS-DFT-36", "FS-DFT-37", "FS-DFT-38",
    }),
    ("gate-b", "gate-c", "gate-d", "gate-e"): frozenset({
        *(f"FS-DFT-{number:02d}" for number in range(1, 42)),
    }) - frozenset({
        "FS-DFT-13", "FS-DFT-14", "FS-DFT-17", "FS-DFT-20",
        "FS-DFT-27", "FS-DFT-28", "FS-DFT-29", "FS-DFT-36",
        "FS-DFT-37", "FS-DFT-38", "FS-DFT-40",
    }),
}
READER_PROJECTION_POPULATIONS = (
    "axes", "compatibility_table", "enum_mapping",
    "enum_mapping_exclusions", "residual_coverage_exclusions",
    "domains", "legacy_rows", "claims", "bodies", "routes",
    "external_assumptions", "envelope", "roles", "role_omissions",
    "powers", "power_contract_templates", "power_refusals",
    "power_crosswalk_dispositions", "dependencies", "dependency_loops",
    "refused_flows",
    "scenarios", "scenario_omissions", "thresholds", "defects",
    "receipts", "proposals", "review_events", "deferred_populations",
    "closure_requirement_profiles", "closure_claim_contracts",
    "model_allocations", "function_allocations", "loop_hazard_controls",
    "bottleneck_dispositions",
)
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
    "evidence-pending",
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

# The roles matrix's closed vocabularies. A role records the standing of a
# person in a position and routes it against domains, scales, and the ratified
# bodies; it is never a floor-changing status, so every role's layer is pinned
# to the constitutional invariant of universal standing. Scales are the seven
# the tracker bullet names as test targets — a closed universe, so the
# validator requires each to be exercised. Anchors keep the formal honesty
# split: a derived constitution predicate, an asserted predicate (with its
# replace-card path recorded in prose), or ratified-but-unimplemented doctrine.
ROLE_KINDS = [
    "life-course",
    "care-and-dependency",
    "learning-and-culture",
    "economic",
    "civic-political",
    "membership-and-mobility",
    "justice-and-coercion",
    "cross-cutting",
]
ROLE_SCALES = [
    "individual",
    "household-association",
    "local",
    "regional",
    "national",
    "cross-jurisdictional",
    "intergenerational",
]
POWER_POSITIONS = ["affected", "checking"]
ROLE_ANCHORS = [
    "constitution-predicate-derived",
    "constitution-predicate-asserted",
    "ratified-doctrine-unimplemented",
]

# The dependency map's closed vocabularies. An edge records that a function
# depends on a flow — never that the flow arrives: no right is called
# delivered because an institution promised it, and no body is called
# functional because its name exists. The four-way class was committed in
# the deferral record's own closure condition; each class fixes the edge's
# layer. The three ratified lifecycle paths stay deliberately asymmetric —
# they are pinned by needle, never flattened into one stage vocabulary.
FLOW_KINDS = [
    "authority",
    "information",
    "care",
    "labour",
    "resources",
    "money",
    "claims",
    "services",
    "accountability",
]
DEPENDENCY_CLASSES = [
    "constitutionally-guaranteed",
    "democratically-selected",
    "operationally-supplied",
    "externally-assumed",
]
DEPENDENCY_CLASS_LAYER = {
    "constitutionally-guaranteed": "constitutional-invariant",
    "democratically-selected": "democratic-ordinary-law-choice",
    "operationally-supplied": "book-2-operation",
    "externally-assumed": "external-assumption",
}
LOOP_KINDS = ["service", "feedback", "fiscal", "ecological", "sequence"]
LIFECYCLE_PATHS = ["right", "power", "record", "outside-ratified-paths"]
LIFECYCLE_PATH_REFS = {
    "right": ("new-book-plans/book-1-constitutional-coverage-map.md::"
              "right  → duty → accessible delivery → recipient-side "
              "evidence of access/receipt"),
    "power": ("new-book-plans/book-1-constitutional-coverage-map.md::"
              "power  → lawful trigger → bounded act → public "
              "reason/evidence"),
    "record": ("new-book-plans/book-1-constitutional-coverage-map.md::"
               "record → authorised basis → limited visibility → challenge"),
}

# The scenario catalogue's closed vocabularies. A scenario is reviewed
# inventory — the assurance portfolio's kind I: citable as a reviewed threat
# model, never as proof or a counterexample harness. A record routes an owned
# ordinary, failure, and recovery path and never claims execution:
# constitutional cases execute only after the relevant author rulings and
# contract cards land, and the closure audit consumes this population. The
# collision axes and the compound shocks are the tracker mandate's named test
# targets — closed universes on the roles-scales precedent; a new axis or
# shock is a deliberate enum-and-meanings amendment, never a free string. A
# scenario citing the protected private/civic domain classifies which
# ratified protected-sphere tests it exercises; no ordinary route may pin
# love, belief, friendship, art, or fulfilment as a state-defined outcome —
# that refusal is reviewed prose discipline, stated in the render, not a
# pattern guard.
SCENARIO_KINDS = ["journey", "stress", "collision", "compound-shock"]
COLLISION_AXES = [
    "property-vs-floor",
    "speech-association-vs-private-harm",
    "majority-vs-minority",
    "parent-guardian-power-vs-child-standing",
    "employer-landlord-platform-power-vs-meaningful-exit",
    "emergency-vs-liberty",
    "present-allocation-vs-future-commons",
    "locality-vs-portability",
    "privacy-vs-public-accountability",
    "physical-scarcity-vs-equal-floor",
]
SHOCK_KINDS = [
    "pandemic",
    "famine",
    "infrastructure-failure",
    "displacement",
    "institutional-capture",
    "conflicting-jurisdictions",
]
PROTECTED_SPHERE_FORMS = [
    "freedom-without-permission",
    "non-recording-non-compulsion",
    "evidenced-harm-threshold",
    "recourse-against-interference",
]
SCENARIO_STATUS = "reviewed-inventory"
PROTECTED_SPHERE_DOMAIN = "FS-DOM-12"

# Stage marker: the reviewed source's status and the report's stage label move in
# lockstep with the content stages; bump both here and in the JSON together.
EXPECTED_STATUS = "stage_4_review_machinery"
STAGE_LABEL = "stage 4 machinery"

# Envelope contract: the array begins with the permanent FS-ENV-00 pre-envelope
# identity and may carry versioned successors. `calibrated` is refused outright
# in this contract — calibration is Book 2's Gate D work and becomes legal only
# through a deliberate future contract amendment, never a string flip. A
# structure-only envelope can route and be reviewed; it cannot assure.
ENVELOPE_STATUSES = ["stub", "versioned-structure", "calibrated"]
VALUE_STATUSES = ["declared-pending"]
LAWFUL_SOURCES = [
    "constitutional-minimum-or-ceiling",
    "democratic-policy-target",
    "scientific-safety-boundary",
    "operational-diagnostic",
]
# The lawful source fixes the threshold's layer; the mapping is closed.
LAWFUL_SOURCE_LAYER = {
    "constitutional-minimum-or-ceiling": "constitutional-invariant",
    "democratic-policy-target": "democratic-ordinary-law-choice",
    "scientific-safety-boundary": "external-assumption",
    "operational-diagnostic": "book-2-operation",
}
CRITERIA_SLUGS = ["adequacy", "accessibility-equality", "continuity",
                  "resilience", "sustainability", "resource", "safety"]
# Envelope-relative claims that MUST appear as dependents of some field: their
# establishment (never their norm content) varies with the envelope.
REQUIRED_DEPENDENTS = ("FS-CLM-06", "FS-CLM-20")

# The rubric's status string is exact in both states. It shipped as a
# candidate; the author confirmed it on 2026-08-09, and the confirmation basis
# is recorded on the rubric itself. An unconfirmed rubric computes as an unmet
# closure condition.
RUBRIC_STATUS_CANDIDATE = "candidate — author confirmation pending"
RUBRIC_STATUS_CONFIRMED = "author-confirmed 2026-08-09 — basis recorded"
READINESS_MET = {"met-mechanically", "met-in-form"}

# The scope-review protocol: R7's evidence contract and admissibility criteria
# live in one document, bound here by needle and by status line so neither can
# flip alone. It shipped as a candidate; the author confirmed it on 2026-08-09
# with the basis recorded on the binding and in the protocol's own
# confirmation record — the severity rubric's two-state history. The
# commitment record inside `review_protocol` stays null until the author
# commissions a review; plant and seed pre-images are constructed only then,
# held outside the repository, and only their SHA-256 digests ever enter this
# source. A commitment may exist only against a confirmed protocol, and so may
# the designation of the severity owner and independent checker — real
# recorded identities, distinct people, neither the pre-image custodian; a
# role-echo placeholder is a fabricated identity the mechanical check cannot
# see, so the substance stays reviewed.
PROTOCOL_DOC = pathlib.Path(
    "new-book-plans/full-society-scope-review-protocol.md")
PROTOCOL_STATUS_CANDIDATE = "candidate — author confirmation pending"
PROTOCOL_STATUS_CONFIRMED = "author-confirmed 2026-08-09 — basis recorded"
SHA256_HEX_RE = re.compile(r"^[0-9a-f]{64}$")
ISO_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")

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
] + DEFERRABLE_ARRAYS + [
    "power_contract_templates",
    "power_refusals",
    "power_crosswalk_dispositions",
    "closure_requirement_profiles",
    "closure_claim_contracts",
    "model_allocations",
    "function_allocations",
    "dependency_loops",
    "loop_hazard_controls",
    "bottleneck_dispositions",
]
ARRAY_RECORD_TYPES = {
    "domains": "domain",
    "legacy_rows": "legacy_row",
    "claims": "claim",
    "bodies": "body",
    "routes": "assurance_route",
    "external_assumptions": "external_assumption",
    "envelope": "envelope",
    "roles": "role",
    "powers": "power",
    "power_contract_templates": "power_contract_template",
    "power_refusals": "power_refusal",
    "power_crosswalk_dispositions": "power_crosswalk_disposition",
    "dependencies": "dependency",
    "scenarios": "scenario",
    "thresholds": "threshold",
    "defects": "defect",
    "receipts": "resolution_receipt",
    "proposals": "proposal",
    "review_events": "review_event",
    "closure_requirement_profiles": "closure_requirement_profile",
    "closure_claim_contracts": "closure_claim_contract",
    "model_allocations": "model_allocation",
    "function_allocations": "function_allocation",
    "dependency_loops": "dependency_loop",
    "loop_hazard_controls": "loop_hazard_control",
    "bottleneck_dispositions": "bottleneck_disposition",
}

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

# Reader evidence uses explicit field names instead of a generic `status`
# leaf. These consequential state fields join the live mapping closure so a
# state transition must update the canonical mapping in the same change.
READER_ENUM_LEAF_KEYS = frozenset([
    "threshold_status", "holdout_status", "result", "route_status",
    "evidence_contract_status", "negative_control_status",
    "pilot_status", "control_status", "admissibility",
    "protocol_validity", "attempt_status", "attempt_result",
    "void_reason_code", "binding_type", "scope", "core_failure_mode",
    "repetition_unit", "metric", "operator", "value_kind", "unit",
    "denominator", "adjudication", "impact", "missing", "ambiguous",
    "multiply_coded", "withdrawn", "excluded", "unclassified",
    "rounding", "coder_adjudication", "code",
])

FORBIDDEN_SCORE_KEYS = frozenset(
    ["score", "percentage", "total", "coverage_figure", "coverage_percent", "rank"]
)


class LedgerError(Exception):
    pass


_READER_EVIDENCE_CACHE = None


def load_validated_reader_evidence():
    """Load the live reader source and run its owning checker once.

    Script 14 owns pilot, threshold, ratification, holdout, and digest
    validity. Reusing its validator here keeps the cross-ledger posture
    transition exact without copying a second, drifting definition.
    """
    global _READER_EVIDENCE_CACHE
    if _READER_EVIDENCE_CACHE is not None:
        return _READER_EVIDENCE_CACHE
    reader = load_json(READER_EVIDENCE_SOURCE)
    validator_path = ROOT / READER_EVIDENCE_VALIDATOR
    spec = importlib.util.spec_from_file_location(
        "reader_evidence_validator", validator_path
    )
    if spec is None or spec.loader is None:
        raise LedgerError(
            f"cannot load reader-evidence validator: {READER_EVIDENCE_VALIDATOR}"
        )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    error_type = getattr(module, "ReaderEvidenceError", None)
    validate_reader = getattr(module, "validate", None)
    if error_type is None or not callable(validate_reader):
        raise LedgerError(
            "reader-evidence validator does not expose its validation contract"
        )
    try:
        validation = validate_reader(copy.deepcopy(reader))
    except error_type as exc:
        raise LedgerError(f"reader-evidence contract invalid: {exc}") from exc
    if (
            not isinstance(validation, tuple)
            or len(validation) != 2
            or any(type(value) is not bool for value in validation)
    ):
        raise LedgerError(
            "reader-evidence validator must return exactly two booleans: "
            "valid_pilot and valid_holdout_pass"
        )
    _, valid_holdout_pass = validation
    _READER_EVIDENCE_CACHE = (reader, valid_holdout_pass)
    return _READER_EVIDENCE_CACHE


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
    if type(src.get("schema_version")) is not int or src["schema_version"] != 3:
        raise LedgerError("schema_version must be the integer 3")
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



def validate_power_source_inventory(src: dict):
    binding = src.get("power_source_inventory")
    if binding != POWER_SOURCE_BINDING:
        raise LedgerError(
            "power_source_inventory must equal the checker-bound reviewed "
            "manifest contract"
        )
    if sha256(POWER_SOURCE_MANIFEST) != binding["artifact_sha256"]:
        raise LedgerError("power source manifest digest mismatch")
    manifest = load_json(POWER_SOURCE_MANIFEST)
    if manifest.get("source_commit") != binding["source_commit"]:
        raise LedgerError("power source manifest commit binding is stale")
    if manifest.get("status") != binding["inventory_status"]:
        raise LedgerError("power source manifest status binding is stale")
    if manifest.get("row_count") != binding["row_count"]:
        raise LedgerError("power source manifest row-count binding is stale")
    summary = manifest.get("coverage_summary", {}).get("by_disposition")
    if summary != binding["disposition_counts"]:
        raise LedgerError("power source manifest disposition binding is stale")
    deferrals = [
        row for row in src.get("deferred_populations", [])
        if row.get("record_type") == "powers"
    ]
    population = src.get("power_population", {})
    expected_deferrals = 0 if population.get("status") == "complete" else 1
    if len(deferrals) != expected_deferrals:
        raise LedgerError(
            "the powers deferral must remain through every partial prefix and "
            "disappear only with the complete source-derived population"
        )
    if deferrals:
        deferral = deferrals[0]
        if deferral.get("owner_ref") != binding["owner_ref"]:
            raise LedgerError(
                "powers deferral owner differs from the source inventory"
            )
        if deferral.get("closure_condition") != binding["closure_condition"]:
            raise LedgerError(
                "powers deferral closure differs from the source inventory"
            )
    if len(binding["known_allocation_gaps"]) != len(
            set(binding["known_allocation_gaps"])):
        raise LedgerError("known power-allocation gaps must be unique")
    validate_reference(binding["owner_ref"], "power_source_inventory.owner_ref")


def _power_manifest_rows():
    return load_json(POWER_SOURCE_MANIFEST)["rows"]


def _power_profiles(row: dict):
    key = row["provisional_key"]
    family = row["source_family"]
    profiles = {"ordinary-public-power"}
    if family == "substantive-equality-and-anti-subordination":
        profiles.add("liberty-power-limit")
        if any(token in key for token in (
                "diagnostic", "finding", "determination", "review")):
            profiles.add("consequential-record")
    elif family == "economic-pluralism-and-protected-private-sphere":
        profiles.add("economic-private-power-limit")
        if "scarcity" in key:
            profiles.add("physical-scarcity")
    elif family == "family-dependency-reproduction-and-collective-plurality":
        profiles.add("consequential-status-supported-decision")
        if any(token in key for token in (
                "collective", "minority", "indigenous", "title", "consent",
                "customary")):
            profiles.add("collective-authority-title-consent")
    elif family == "ecological-commons-and-non-human-animal":
        profiles.add("non-human-animal" if key.startswith("animal-")
                     else "commons-future-condition")
    elif family == "public-safety-defence-emergency-and-external-power":
        if key.startswith("emergency-"):
            profiles.add("emergency")
        if any(token in key for token in (
                "arrest", "detention", "search", "seizure", "force",
                "custod", "restriction", "polic", "protect", "border",
                "expulsion", "extradition", "transfer", "deployment",
                "defensive", "intelligence", "surveillance", "weapon")):
            profiles.add("coercive-protective")
        if "intelligence" in key or "record" in key:
            profiles.add("consequential-record")
    elif family == "state-form-and-political-membership":
        if any(token in key for token in (
                "certif", "appointment", "qualification", "membership",
                "election", "selection", "succession")):
            profiles.add("consequential-record")
    if key == RETAINED_FORMAL_KEY:
        profiles.add("coercive-protective")
    return [p for p in POWER_PROFILE_ORDER if p in profiles]


def _power_required_separations(profiles):
    pairs = copy.deepcopy(UNIVERSAL_SEPARATION_PAIRS)
    if "coercive-protective" in profiles:
        pairs.append(["decisive-fact-writer", "executor"])
    return pairs


def _typed_ref_list(value, expected_array, ids, context, allow_empty=False):
    if (not isinstance(value, list) or len(set(value)) != len(value)
            or (not value and not allow_empty)):
        suffix = "" if allow_empty else " non-empty"
        raise LedgerError(
            f"{context} must be a{suffix} duplicate-free list"
        )
    for ref in value:
        if ref not in ids or ids[ref] != expected_array:
            raise LedgerError(
                f"{context} must name {expected_array}, got {ref!r}"
            )


def _validate_source_refs(refs, context):
    if (not isinstance(refs, list) or not refs
            or len(set(refs)) != len(refs)):
        raise LedgerError(
            f"{context} must be a non-empty duplicate-free list"
        )
    for ref in refs:
        validate_reference(ref, context)


def _manifest_expectations(completed):
    manifest = _power_manifest_rows()
    rows = [
        row for family in completed for row in manifest
        if row["source_family"] == family
    ]
    powers = [r["provisional_key"] for r in rows
              if r["disposition"] == "card-required"]
    if "current-formal-constitution" in completed:
        powers.append(RETAINED_FORMAL_KEY)
    return {
        "powers": powers,
        "templates": [r["provisional_key"] for r in rows
                      if r["disposition"] == "power-contract-template"],
        "refusals": [r["provisional_key"] for r in rows
                     if r["disposition"] == "explicit-refusal-limit"],
        "crosswalks": [r["provisional_key"] for r in rows
                       if r["disposition"] == "existing-formal-crosswalk"],
    }


def validate_power_population(src: dict, ids: dict):
    population = src.get("power_population")
    if not isinstance(population, dict):
        raise LedgerError("power_population must be an object")
    exact_keys(
        population,
        ["status", "completed_source_families", "expected_final_counts",
         "resolved_allocation_gaps", "evidence_ceiling"],
        "power_population",
    )
    completed = population["completed_source_families"]
    if not isinstance(completed, list):
        raise LedgerError(
            "power_population.completed_source_families must be a list"
        )
    if completed != POWER_SOURCE_FAMILY_ORDER[:len(completed)]:
        raise LedgerError(
            "completed_source_families must be an exact checker-owned prefix"
        )
    expected_status = (
        "foundation" if not completed else
        "complete" if completed == POWER_SOURCE_FAMILY_ORDER else "partial"
    )
    if population["status"] != expected_status:
        raise LedgerError(
            f"power_population.status must be {expected_status!r} for its prefix"
        )
    if population["expected_final_counts"] != POWER_FINAL_COUNTS:
        raise LedgerError(
            "power_population.expected_final_counts must equal the "
            "checker-owned source-derived partition"
        )
    require_str(population, "evidence_ceiling", "power_population")
    if "no operation" not in population["evidence_ceiling"].lower():
        raise LedgerError(
            "power_population.evidence_ceiling must refuse operation claims"
        )

    gaps = population["resolved_allocation_gaps"]
    if not isinstance(gaps, list):
        raise LedgerError("resolved_allocation_gaps must be a list")
    expected_gaps = POWER_SOURCE_BINDING["known_allocation_gaps"]
    if [g.get("gap") for g in gaps] != expected_gaps:
        raise LedgerError(
            "resolved_allocation_gaps must resolve every checker-bound gap "
            "once and in canonical order"
        )
    for i, gap in enumerate(gaps):
        ctx = f"power_population.resolved_allocation_gaps[{i}]"
        exact_keys(gap, ["gap", "body_refs", "role_refs", "source_refs"], ctx)
        _typed_ref_list(gap["body_refs"], "bodies", ids, f"{ctx}.body_refs")
        _typed_ref_list(gap["role_refs"], "roles", ids, f"{ctx}.role_refs")
        _validate_source_refs(gap["source_refs"], f"{ctx}.source_refs")

    expectations = _manifest_expectations(completed)
    arrays = {
        "powers": src.get("powers", []),
        "templates": src.get("power_contract_templates", []),
        "refusals": src.get("power_refusals", []),
        "crosswalks": src.get("power_crosswalk_dispositions", []),
    }
    for name, records in arrays.items():
        keys = [r.get("manifest_key") for r in records]
        if keys != expectations[name]:
            raise LedgerError(
                f"{name} must contain every and only its completed-family "
                "manifest rows in canonical order"
            )
    expected_counts = ({"powers": 0, "templates": 0,
                        "refusals": 0, "crosswalks": 0}
                       if not completed else
                       POWER_FAMILY_CUMULATIVE_COUNTS[completed[-1]])
    actual_counts = {name: len(records) for name, records in arrays.items()}
    if actual_counts != expected_counts:
        raise LedgerError(
            f"power prefix counts {actual_counts} differ from "
            f"checker-owned {expected_counts}"
        )

    manifest = {r["provisional_key"]: r for r in _power_manifest_rows()}
    powers_by_manifest = {}
    for i, rec in enumerate(arrays["powers"]):
        ctx = f"powers[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + [
                "manifest_key", "source_family", "posture", "evidence_kind",
                "profiles", "domain_refs", "affected_claim_refs",
                "holder_body_refs", "holder_role_refs",
                "affected_role_refs", "checking_role_refs", "route_ref",
                "overlay", "public_claim_restriction",
                "structural_wall_refs", "related_power_refs",
                "enforcement_mechanism", "book2_owner_ref", "contract",
                "profile_contracts", "source_refs",
            ],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        row = manifest.get(rec["manifest_key"])
        if row is None:
            raise LedgerError(f"{ctx}: unknown manifest_key")
        if rec["source_family"] != row["source_family"]:
            raise LedgerError(f"{ctx}: source_family differs from manifest")
        retained = rec["manifest_key"] == RETAINED_FORMAL_KEY
        if row["disposition"] != (
                "existing-formal-crosswalk" if retained else "card-required"):
            raise LedgerError(f"{ctx}: manifest disposition cannot create a power")
        if rec["layer"] != "constitutional-invariant":
            raise LedgerError(f"{ctx}: power contracts are constitutional architecture")
        expected_posture = "Derived" if retained else "Specified"
        expected_evidence = "executable" if retained else "inventory"
        if rec["posture"] != expected_posture or rec["evidence_kind"] != expected_evidence:
            raise LedgerError(f"{ctx}: posture/evidence must follow source status")
        expected_status_value = (
            "implemented-narrow-derived" if retained
            else "ratified-unimplemented"
        )
        if rec["status"] != expected_status_value:
            raise LedgerError(f"{ctx}: status must be {expected_status_value}")
        profiles = _power_profiles(row)
        if rec["profiles"] != profiles:
            raise LedgerError(f"{ctx}: profiles differ from checker-owned classification")
        contracts = rec["profile_contracts"]
        if not isinstance(contracts, dict) or list(contracts) != profiles:
            raise LedgerError(f"{ctx}: profile_contracts must match profiles in order")
        for profile in profiles:
            block = contracts[profile]
            exact_keys(block, POWER_PROFILE_FIELDS[profile],
                       f"{ctx}.profile_contracts.{profile}")
            for field in POWER_PROFILE_FIELDS[profile]:
                value = require_str(block, field,
                                    f"{ctx}.profile_contracts.{profile}")
                if value.strip().lower() in {"n/a", "na", "tbd", "unknown"}:
                    raise LedgerError(f"{ctx}: blank substitute in profile contract")
        contract = rec["contract"]
        exact_keys(contract, POWER_CONTRACT_KEYS, f"{ctx}.contract")
        for field in POWER_CONTRACT_KEYS[:-1]:
            value = require_str(contract, field, f"{ctx}.contract")
            if value.strip().lower() in {"n/a", "na", "tbd", "unknown"}:
                raise LedgerError(f"{ctx}: blank substitute in contract")
        expected_pairs = _power_required_separations(profiles)
        if contract["required_separation_pairs"] != expected_pairs:
            raise LedgerError(f"{ctx}: required separation pairs are incomplete")
        _typed_ref_list(rec["domain_refs"], "domains", ids,
                        f"{ctx}.domain_refs")
        _typed_ref_list(rec["affected_claim_refs"], "claims", ids,
                        f"{ctx}.affected_claim_refs")
        _typed_ref_list(rec["holder_body_refs"], "bodies", ids,
                        f"{ctx}.holder_body_refs", allow_empty=True)
        _typed_ref_list(rec["holder_role_refs"], "roles", ids,
                        f"{ctx}.holder_role_refs")
        _typed_ref_list(rec["affected_role_refs"], "roles", ids,
                        f"{ctx}.affected_role_refs")
        _typed_ref_list(rec["checking_role_refs"], "roles", ids,
                        f"{ctx}.checking_role_refs")
        _typed_ref_list(rec["structural_wall_refs"], "power_refusals", ids,
                        f"{ctx}.structural_wall_refs", allow_empty=True)
        _typed_ref_list(rec["related_power_refs"], "powers", ids,
                        f"{ctx}.related_power_refs", allow_empty=True)
        if rec["route_ref"] not in ids or ids[rec["route_ref"]] != "routes":
            raise LedgerError(f"{ctx}: route_ref must name a route")
        if rec["overlay"] not in OVERLAYS:
            raise LedgerError(f"{ctx}: unknown overlay")
        require_str(rec, "public_claim_restriction", ctx)
        if "no operation" not in rec["public_claim_restriction"].lower():
            raise LedgerError(f"{ctx}: public claim restriction must refuse operation")
        require_str(rec, "enforcement_mechanism", ctx)
        if rec["book2_owner_ref"] != POWER_BOOK2_OWNER:
            raise LedgerError(f"{ctx}: Book 2 owner must remain checker-bound")
        validate_reference(rec["book2_owner_ref"], f"{ctx}.book2_owner_ref")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        if row["source_anchor"] not in rec["source_refs"]:
            raise LedgerError(f"{ctx}: source_refs must include manifest anchor")
        powers_by_manifest[rec["manifest_key"]] = rec

    for i, rec in enumerate(arrays["templates"]):
        ctx = f"power_contract_templates[{i}]"
        exact_keys(rec, COMMON_KEYS + ["manifest_key", "contract", "source_refs"], ctx)
        validate_common_record_fields(rec, ctx)
        row = manifest[rec["manifest_key"]]
        if rec["manifest_key"] != POWER_TEMPLATE_KEY:
            raise LedgerError(f"{ctx}: only the checker-bound time template is allowed")
        if rec["layer"] != "constitutional-invariant" or rec["status"] != "ratified-template":
            raise LedgerError(f"{ctx}: template layer/status mismatch")
        exact_keys(rec["contract"], POWER_TEMPLATE_CONTRACT_KEYS, f"{ctx}.contract")
        for field in POWER_TEMPLATE_CONTRACT_KEYS:
            require_str(rec["contract"], field, f"{ctx}.contract")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        if row["source_anchor"] not in rec["source_refs"]:
            raise LedgerError(f"{ctx}: template must include manifest anchor")

    for i, rec in enumerate(arrays["refusals"]):
        ctx = f"power_refusals[{i}]"
        exact_keys(
            rec,
            COMMON_KEYS + [
                "manifest_key", "source_family", "refusal", "scope",
                "protected_boundary", "permitted_residual",
                "non_authorisation", "affected_power_refs", "domain_refs",
                "affected_claim_refs", "affected_role_refs", "route_ref",
                "public_claim_restriction", "source_refs",
            ],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        row = manifest[rec["manifest_key"]]
        if row["disposition"] != "explicit-refusal-limit":
            raise LedgerError(f"{ctx}: only refusal manifest rows are allowed")
        if rec["source_family"] != row["source_family"]:
            raise LedgerError(f"{ctx}: source_family differs from manifest")
        for field in ("refusal", "scope", "protected_boundary",
                      "permitted_residual", "non_authorisation",
                      "public_claim_restriction"):
            require_str(rec, field, ctx)
        _typed_ref_list(rec["affected_power_refs"], "powers", ids,
                        f"{ctx}.affected_power_refs", allow_empty=True)
        _typed_ref_list(rec["domain_refs"], "domains", ids,
                        f"{ctx}.domain_refs")
        _typed_ref_list(rec["affected_claim_refs"], "claims", ids,
                        f"{ctx}.affected_claim_refs")
        _typed_ref_list(rec["affected_role_refs"], "roles", ids,
                        f"{ctx}.affected_role_refs")
        if rec["route_ref"] not in ids or ids[rec["route_ref"]] != "routes":
            raise LedgerError(f"{ctx}: route_ref must name a route")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        if row["source_anchor"] not in rec["source_refs"]:
            raise LedgerError(f"{ctx}: source_refs must include manifest anchor")

    for i, rec in enumerate(arrays["crosswalks"]):
        ctx = f"power_crosswalk_dispositions[{i}]"
        exact_keys(
            rec,
            COMMON_KEYS + [
                "manifest_key", "crosswalk_action", "target_power_refs",
                "current_effect", "retired_residual_effect", "non_extension",
                "transition_owner_ref", "source_refs",
            ],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        key = rec["manifest_key"]
        row = manifest[key]
        expected_action = CROSSWALK_POLICY[key]
        if rec["crosswalk_action"] != expected_action:
            raise LedgerError(f"{ctx}: crosswalk action violates checker-owned policy")
        _typed_ref_list(rec["target_power_refs"], "powers", ids,
                        f"{ctx}.target_power_refs", allow_empty=True)
        if expected_action == "retire" and rec["target_power_refs"]:
            raise LedgerError(f"{ctx}: retired effects have no target power")
        if expected_action != "retire" and not rec["target_power_refs"]:
            raise LedgerError(f"{ctx}: retain/replace needs a target power")
        if key == "formal-electorate-seating-authority":
            expected = [powers_by_manifest["gov-proportional-election-certification"]["id"]]
        elif key == "formal-public-body-authority":
            expected = [r["id"] for r in arrays["powers"]]
        elif key == "formal-appeals-relief":
            expected = [powers_by_manifest["gov-ordinary-court-relief"]["id"]]
        elif key == RETAINED_FORMAL_KEY:
            expected = [powers_by_manifest[RETAINED_FORMAL_KEY]["id"]]
        else:
            expected = []
        if rec["target_power_refs"] != expected:
            raise LedgerError(f"{ctx}: targets violate checker-owned crosswalk policy")
        for field in ("current_effect", "retired_residual_effect",
                      "non_extension"):
            require_str(rec, field, ctx)
        validate_reference(rec["transition_owner_ref"],
                           f"{ctx}.transition_owner_ref")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        if row["source_anchor"] not in rec["source_refs"]:
            raise LedgerError(f"{ctx}: source_refs must include manifest anchor")

    allocations = src.get("function_allocations", [])
    if len(allocations) != len(arrays["powers"]):
        raise LedgerError("every populated power requires exactly one FS-FAL")
    power_by_id = {r["id"]: r for r in arrays["powers"]}
    seen_power_refs = set()
    for i, rec in enumerate(allocations):
        ctx = f"function_allocations[{i}] ({rec.get('id', '?')})"
        function_fields = []
        for function in POWER_FUNCTIONS:
            stem = function.replace("-", "_")
            function_fields += [f"{stem}_body_refs", f"{stem}_role_refs"]
        exact_keys(
            rec,
            ["id", "power_ref", "affected_claim_refs"] + function_fields
            + ["separation_constraints", "source_refs"],
            ctx,
        )
        power = power_by_id.get(rec["power_ref"])
        if power is None or rec["power_ref"] in seen_power_refs:
            raise LedgerError(f"{ctx}: power_ref must be unique and name a power")
        seen_power_refs.add(rec["power_ref"])
        if rec["affected_claim_refs"] != power["affected_claim_refs"]:
            raise LedgerError(f"{ctx}: affected claims must equal the power card")
        bodies_by_function = {}
        for function in POWER_FUNCTIONS:
            stem = function.replace("-", "_")
            body_field = f"{stem}_body_refs"
            role_field = f"{stem}_role_refs"
            _typed_ref_list(rec[body_field], "bodies", ids,
                            f"{ctx}.{body_field}")
            _typed_ref_list(rec[role_field], "roles", ids,
                            f"{ctx}.{role_field}")
            bodies_by_function[function] = set(rec[body_field])
        constraints = rec["separation_constraints"]
        pairs = power["contract"]["required_separation_pairs"]
        if not isinstance(constraints, list) or len(constraints) != len(pairs):
            raise LedgerError(f"{ctx}: one constraint is required per pair")
        for j, (constraint, pair) in enumerate(zip(constraints, pairs)):
            cctx = f"{ctx}.separation_constraints[{j}]"
            exact_keys(constraint, ["functions", "reason", "source_ref"], cctx)
            if constraint["functions"] != pair:
                raise LedgerError(f"{cctx}: function pair differs from power contract")
            require_str(constraint, "reason", cctx)
            validate_reference(constraint["source_ref"], f"{cctx}.source_ref")
            if bodies_by_function[pair[0]] & bodies_by_function[pair[1]]:
                raise LedgerError(f"{cctx}: required body separation is fused")
        if set.intersection(*(bodies_by_function[f] for f in POWER_FUNCTIONS)):
            raise LedgerError(f"{ctx}: one body may not occupy all five functions")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
    if seen_power_refs != set(power_by_id):
        raise LedgerError("FS-FAL power references must be a complete bijection")

    if completed == POWER_SOURCE_FAMILY_ORDER:
        t3 = powers_by_manifest[RETAINED_FORMAL_KEY]
        custody = powers_by_manifest["protect-custodial-execution-mandate"]
        if t3["related_power_refs"] != [custody["id"]] or \
                custody["related_power_refs"] != [t3["id"]]:
            raise LedgerError("T3 Court authority and custody execution must be reciprocal but separate")
        if t3["holder_body_refs"] != ["FS-BOD-17"]:
            raise LedgerError("retained T3 authority is held only by the Court")
        if custody["holder_body_refs"] != ["FS-BOD-35"]:
            raise LedgerError("custodial execution is held only by the distinct executor")


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
        "envelope_status_meanings": ENVELOPE_STATUSES,
        "value_status_meanings": VALUE_STATUSES,
        "lawful_source_meanings": LAWFUL_SOURCES,
        "role_kind_meanings": ROLE_KINDS,
        "scale_meanings": ROLE_SCALES,
        "power_position_meanings": POWER_POSITIONS,
        "role_anchor_meanings": ROLE_ANCHORS,
        "flow_kind_meanings": FLOW_KINDS,
        "dependency_class_meanings": DEPENDENCY_CLASSES,
        "loop_kind_meanings": LOOP_KINDS,
        "lifecycle_path_meanings": LIFECYCLE_PATHS,
        "scenario_kind_meanings": SCENARIO_KINDS,
        "collision_axis_meanings": COLLISION_AXES,
        "shock_kind_meanings": SHOCK_KINDS,
        "protected_sphere_form_meanings": PROTECTED_SPHERE_FORMS,
        "gate_applicability_meanings": GATE_REFS,
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
    if src["gate_applicability_meanings"] != GATE_APPLICABILITY_MEANINGS:
        raise LedgerError(
            "gate_applicability_meanings must equal the ratified Gate A-E "
            "contract"
        )


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
            expected_type = ARRAY_RECORD_TYPES[array]
            if registry[prefix] != expected_type:
                raise LedgerError(
                    f"{array}: id {rid} prefix is registered for "
                    f"{registry[prefix]!r}, not {expected_type!r}"
                )
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
                "closure_requirement_refs",
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
                f"{ctx}: envelope_id names the pre-envelope identity "
                f"{ENVELOPE_STUB_ID} — Book 1 claims are not envelope-bound; "
                "only a functional or feasibility claim binds a versioned "
                "envelope, and Book 1 may carry none"
            )
        for ref in rec["domain_refs"]:
            if ref not in ids or ids[ref] != "domains":
                raise LedgerError(f"{ctx}: unknown domain {ref}")
        closure_refs = rec["closure_requirement_refs"]
        if (not isinstance(closure_refs, list)
                or len(closure_refs) != len(set(closure_refs))
                or any(not isinstance(ref, str) or not ref.startswith("FS-CLR-")
                       for ref in closure_refs)):
            raise LedgerError(
                f"{ctx}: closure_requirement_refs must be unique FS-CLR ids"
            )
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
                if route["route_status"] != "unbuilt":
                    raise LedgerError(
                        f"{ctx}: route-unbuilt requires an unbuilt route; "
                        f"{route['id']} is {route['route_status']}"
                    )
                # severity/consequence/owner/closure are already mandatory on
                # every record; the claim restriction is the extra field.
                if "restricted" not in rec["public_claim_restriction"].lower() \
                        and "no public" not in rec["public_claim_restriction"].lower():
                    raise LedgerError(
                        f"{ctx}: a route-unbuilt row must state its public-claim "
                        "restriction"
                    )
            if ud == "evidence-pending" and route["route_status"] not in (
                    "built", "available"):
                raise LedgerError(
                    f"{ctx}: evidence-pending requires a built or available "
                    f"route; {route['id']} is {route['route_status']}"
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


def validate_reader_evidence_alignment(src: dict, routes_by_id: dict):
    """Bind R6 and FS-CLM-37 to the live reviewed reader contract.

    R6 is external: it moves from unbuilt to available, never built. The
    claim is route-unbuilt while R6 is unbuilt, evidence-pending while R6
    is available without a matching valid pass, and Evidenced only for the
    exact pass accepted by script 14.
    """
    reader, valid_holdout_pass = load_validated_reader_evidence()
    reader_route = reader["route"]
    reader_claim = reader["claim"]
    route = routes_by_id.get("FS-RTE-06")
    if route is None:
        raise LedgerError("reader alignment: missing FS-RTE-06")
    if route["route_status"] == "built":
        raise LedgerError(
            "reader alignment: FS-RTE-06 is an external route and may never "
            "take the in-repository built status"
        )
    if route["status"] != route["route_status"]:
        raise LedgerError(
            "reader alignment: FS-RTE-06 status and route_status must agree"
        )
    if reader_route["route_id"] != route["id"]:
        raise LedgerError("reader alignment: reader source names the wrong route")
    if reader_route["route_status"] != route["route_status"]:
        raise LedgerError(
            "reader alignment: FS-RTE-06 route_status must match "
            "reader-evidence.json"
        )
    claim = next(
        (rec for rec in src.get("claims", []) if rec.get("id") == "FS-CLM-37"),
        None,
    )
    if claim is None:
        raise LedgerError("reader alignment: missing FS-CLM-37")
    if claim["route_ref"] != route["id"]:
        raise LedgerError("reader alignment: FS-CLM-37 must use FS-RTE-06")
    if reader_claim["claim_id"] != claim["id"]:
        raise LedgerError("reader alignment: reader source names the wrong claim")
    if valid_holdout_pass:
        expected = ("Evidenced", "none")
    elif route["route_status"] == "available":
        expected = ("Unestablished", "evidence-pending")
    else:
        expected = ("Unestablished", "route-unbuilt")
    reader_state = (reader_claim["posture"], reader_claim["disposition"])
    ledger_state = (
        claim["posture"],
        claim.get("unestablished_disposition", "none"),
    )
    if reader_state != expected:
        raise LedgerError(
            "reader alignment: reviewed reader claim contradicts its validated "
            "route and holdout state"
        )
    if ledger_state != expected:
        raise LedgerError(
            f"reader alignment: FS-CLM-37 must be {expected[0]}/{expected[1]} "
            "for the live reader-evidence state"
        )
    if (reader["result"] == "fail"
            and reader_route["route_status"] == "available"
            and ledger_state != ("Unestablished", "evidence-pending")):
        raise LedgerError(
            "reader alignment: a persisted failure on an available route "
            "requires Unestablished/evidence-pending; active holdout status "
            "may not rewrite it as not-run"
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


def validate_roles(src: dict, ids: dict):
    """The roles, life-course, scale, and power-position matrix.

    Three closures are mechanical: every domain cited by at least one role,
    every named scale exercised, and every required body carrying both an
    affected and a checking role position. Pairwise sufficiency (which
    role x scale x domain cells matter) is NOT mechanically decidable and
    stays a question for the independent scope review. An entry in
    role_omissions is a CLOSED classification decision with a risk-based
    reason (the residual_coverage_exclusions precedent), never
    unresolved-shape — "should be covered later" belongs in a defect row or a
    role's closure_condition, not here."""
    roles = src.get("roles", [])
    if not roles:
        if src.get("role_omissions"):
            raise LedgerError(
                "role_omissions may not exist while roles is deferred — an "
                "omission is a decision about a populated matrix"
            )
        return
    domain_ids = {r for r, a in ids.items() if a == "domains"}
    body_ids = {r for r, a in ids.items() if a == "bodies"}
    cited_domains, exercised_scales = set(), set()
    body_positions = {}
    roles_by_id = {}
    for i, rec in enumerate(roles):
        ctx = f"roles[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["role_kind", "domain_refs", "scales",
                           "power_positions", "formal_anchor",
                           "floor_invariance", "source_refs"],
            ctx,
            optional=["power_held"],
        )
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != "constitutional-invariant":
            raise LedgerError(
                f"{ctx}: a role is an application of universal standing and "
                "never a floor-changing status; its layer is "
                "constitutional-invariant — rule content lives on domains "
                "and claims, not on a role"
            )
        if rec["role_kind"] not in ROLE_KINDS:
            raise LedgerError(f"{ctx}: unknown role_kind {rec['role_kind']!r}")
        refs = rec["domain_refs"]
        if not isinstance(refs, list) or not refs \
                or len(set(refs)) != len(refs):
            raise LedgerError(
                f"{ctx}: domain_refs must be a non-empty duplicate-free list"
            )
        for ref in refs:
            if ref not in ids or ids[ref] != "domains":
                raise LedgerError(
                    f"{ctx}: domain_refs must name domains, got {ref!r}"
                )
        cited_domains |= set(refs)
        scales = rec["scales"]
        if (not isinstance(scales, list) or not scales
                or len(set(scales)) != len(scales)
                or any(sc not in ROLE_SCALES for sc in scales)):
            raise LedgerError(
                f"{ctx}: scales must be a non-empty duplicate-free subset of "
                f"{ROLE_SCALES}"
            )
        exercised_scales |= set(scales)
        pps = rec["power_positions"]
        if not isinstance(pps, list):
            raise LedgerError(f"{ctx}: power_positions must be a list")
        for j, pp in enumerate(pps):
            pctx = f"{ctx}.power_positions[{j}]"
            exact_keys(pp, ["body_ref", "position", "note"], pctx)
            if pp["body_ref"] not in ids or ids[pp["body_ref"]] != "bodies":
                raise LedgerError(
                    f"{pctx}: body_ref must name a ratified body, got "
                    f"{pp['body_ref']!r} — the FS-POW decomposition stays "
                    "with its own deferred population"
                )
            if pp["position"] not in POWER_POSITIONS:
                raise LedgerError(
                    f"{pctx}: position must be affected or checking"
                )
            require_str(pp, "note", pctx)
            body_positions.setdefault(pp["body_ref"], set()).add(
                pp["position"])
        ph = rec.get("power_held")
        if ph is not None:
            pctx = f"{ctx}.power_held"
            exact_keys(ph, ["power", "source_ref", "affected_role_refs",
                            "checking_refs"], pctx)
            require_str(ph, "power", pctx)
            validate_reference(ph["source_ref"], f"{pctx}.source_ref")
            ar = ph["affected_role_refs"]
            if not isinstance(ar, list) or not ar \
                    or len(set(ar)) != len(ar):
                raise LedgerError(
                    f"{pctx}: affected_role_refs must be a non-empty "
                    "duplicate-free list"
                )
            for ref in ar:
                if ref not in ids or ids[ref] != "roles":
                    raise LedgerError(
                        f"{pctx}: affected_role_refs must name roles, got "
                        f"{ref!r}"
                    )
                if ref == rec["id"]:
                    raise LedgerError(
                        f"{pctx}: a power's holder is not its own affected "
                        "position"
                    )
            cr = ph["checking_refs"]
            if not isinstance(cr, list) or not cr \
                    or len(set(cr)) != len(cr):
                raise LedgerError(
                    f"{pctx}: checking_refs must be a non-empty "
                    "duplicate-free list — an unchecked private power is "
                    "not recordable here"
                )
            for ref in cr:
                if ref not in ids or ids[ref] not in ("bodies", "roles"):
                    raise LedgerError(
                        f"{pctx}: checking_refs must name bodies or roles, "
                        f"got {ref!r}"
                    )
        fa = rec["formal_anchor"]
        exact_keys(fa, ["anchor", "refs"], f"{ctx}.formal_anchor")
        if fa["anchor"] not in ROLE_ANCHORS:
            raise LedgerError(
                f"{ctx}: unknown formal anchor {fa['anchor']!r}"
            )
        if not isinstance(fa["refs"], list) or not fa["refs"]:
            raise LedgerError(f"{ctx}: formal_anchor.refs must be non-empty")
        for j, ref in enumerate(fa["refs"]):
            validate_reference(ref, f"{ctx}.formal_anchor.refs[{j}]")
        if fa["anchor"].startswith("constitution-predicate"):
            if not any(r.split("::", 1)[0].endswith(".nibli")
                       for r in fa["refs"]):
                raise LedgerError(
                    f"{ctx}: a constitution-predicate anchor must cite the "
                    "constitution source itself (a .nibli needle), never "
                    "only prose"
                )
        require_str(rec, "floor_invariance", ctx)
        srcs = rec["source_refs"]
        if not isinstance(srcs, list) or not srcs:
            raise LedgerError(f"{ctx}: source_refs must be non-empty")
        for j, ref in enumerate(srcs):
            validate_reference(ref, f"{ctx}.source_refs[{j}]")
        roles_by_id[rec["id"]] = rec
    missing = sorted(domain_ids - cited_domains)
    if missing:
        raise LedgerError(
            "role/domain closure: each material domain needs reviewed role "
            f"applicability; no role cites: {missing}"
        )
    missing_scales = sorted(set(ROLE_SCALES) - exercised_scales)
    if missing_scales:
        raise LedgerError(
            "role/scale closure: every named scale must be exercised by at "
            f"least one role; unexercised: {missing_scales}"
        )
    for bid in sorted(body_ids):
        pos = body_positions.get(bid, set())
        if pos != set(POWER_POSITIONS):
            raise LedgerError(
                f"power-position closure: body {bid} needs both an affected "
                f"and a checking role position; has {sorted(pos) or 'neither'}"
            )
    om = src.get("role_omissions")
    if not isinstance(om, list) or not om:
        raise LedgerError(
            "role_omissions must be a non-empty list once roles populate — "
            "omitted combinations carry an explicit risk-based reason"
        )
    seen = set()
    for i, entry in enumerate(om):
        ctx = f"role_omissions[{i}]"
        axis = [k for k in ("omitted_role", "omitted_domain_ref",
                            "omitted_scale") if k in entry]
        if len(axis) != 1:
            raise LedgerError(
                f"{ctx}: exactly one of omitted_role / omitted_domain_ref / "
                "omitted_scale"
            )
        if axis[0] == "omitted_role":
            exact_keys(entry, ["omitted_role", "risk_reason", "source_ref"],
                       ctx)
            require_str(entry, "omitted_role", ctx)
            validate_reference(entry["source_ref"], f"{ctx}.source_ref")
            key = ("role", entry["omitted_role"])
        else:
            exact_keys(entry, ["role_ref", axis[0], "risk_reason"], ctx)
            role = roles_by_id.get(entry.get("role_ref"))
            if role is None:
                raise LedgerError(
                    f"{ctx}: unknown role {entry.get('role_ref')!r}"
                )
            val = entry[axis[0]]
            if axis[0] == "omitted_domain_ref":
                if val not in domain_ids:
                    raise LedgerError(f"{ctx}: unknown domain {val!r}")
                if val in role["domain_refs"]:
                    raise LedgerError(
                        f"{ctx}: stale omission — the role already carries "
                        f"{val}"
                    )
            else:
                if val not in ROLE_SCALES:
                    raise LedgerError(f"{ctx}: unknown scale {val!r}")
                if val in role["scales"]:
                    raise LedgerError(
                        f"{ctx}: stale omission — the role already carries "
                        f"{val}"
                    )
            key = (entry["role_ref"], val)
        if key in seen:
            raise LedgerError(f"{ctx}: duplicate omission {key}")
        seen.add(key)
        require_str(entry, "risk_reason", ctx)


def _reach(adj, start):
    """Nodes reachable from start in >= 1 step (5-spine-gen's DFS idiom)."""
    seen, stack = set(), [start]
    while stack:
        n = stack.pop()
        for m in adj.get(n, ()):
            if m not in seen:
                seen.add(m)
                stack.append(m)
    return seen


def validate_dependencies(src: dict, ids: dict):
    """The functional-flow and cross-domain dependency map.

    An edge is routing, never delivery: it records that a function depends
    on a flow, its lawful source class, its owner, and what breaks when the
    flow stops. Mechanical closures: every domain an endpoint, every flow
    kind exercised, every external assumption feeding an externally-assumed
    edge, and — at SCC grain — every strongly connected region of the
    declared graph carrying at least one declared, classified, bounded,
    owned loop witness. Boundedness is reviewed prose, never a proven
    property. The constitutional-closure audit consumes typed hazard and
    bottleneck dispositions; an open row remains unresolved and cyclicity
    alone is never rejected. The grain is one edge per flow kind
    per ordered pair; an absent alternate route is a recorded single point
    of failure, never a silent one."""
    deps = src.get("dependencies", [])
    if not deps:
        for extra in ("dependency_loops", "refused_flows"):
            if src.get(extra):
                raise LedgerError(
                    f"{extra} may not exist while dependencies is deferred "
                    "— loops and walls are decisions about a populated map"
                )
        return
    for name, ref in LIFECYCLE_PATH_REFS.items():
        validate_reference(ref, f"lifecycle path `{name}`")
    domain_ids = {r for r, a in ids.items() if a == "domains"}
    exa_ids = {r for r, a in ids.items() if a == "external_assumptions"}
    edges, triples = {}, set()
    touched, exercised, cited_exas = set(), set(), set()
    for i, rec in enumerate(deps):
        ctx = f"dependencies[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["flow_kind", "dependency_class", "from_ref",
                           "to_ref", "steward_ref", "lifecycle_path",
                           "interim_continuity", "remedy_route",
                           "restoration", "systemic_correction",
                           "alternate_route", "source_refs",
                           "structural_satisfiability",
                           "closure_component_refs"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["steward_ref"] not in ids or ids[rec["steward_ref"]] != "bodies":
            raise LedgerError(
                f"{ctx}: steward_ref must name a required body — the "
                f"institution answerable for the flow; got "
                f"{rec['steward_ref']!r}"
            )
        if rec["flow_kind"] not in FLOW_KINDS:
            raise LedgerError(f"{ctx}: unknown flow_kind {rec['flow_kind']!r}")
        cls = rec["dependency_class"]
        if cls not in DEPENDENCY_CLASSES:
            raise LedgerError(f"{ctx}: unknown dependency_class {cls!r}")
        if rec["layer"] != DEPENDENCY_CLASS_LAYER[cls]:
            raise LedgerError(
                f"{ctx}: an edge's layer follows its dependency class — "
                f"{cls} fixes {DEPENDENCY_CLASS_LAYER[cls]!r}"
            )
        f, t = rec["from_ref"], rec["to_ref"]
        if ids.get(f) not in ("bodies", "roles", "domains",
                              "external_assumptions"):
            raise LedgerError(
                f"{ctx}: source must name a body, role, domain, or external "
                f"assumption, got {f!r}"
            )
        if ids.get(t) not in ("bodies", "roles", "domains"):
            raise LedgerError(
                f"{ctx}: destination must name a body, role, or domain — an "
                f"external assumption supplies, never receives; got {t!r}"
            )
        if f == t:
            raise LedgerError(
                f"{ctx}: an edge may not terminate on its own source — a "
                "genuine internal circle is drawn through the roles and "
                "bodies that carry it and declared as a loop"
            )
        if (cls == "externally-assumed") != (ids[f] == "external_assumptions"):
            if cls == "externally-assumed":
                raise LedgerError(
                    f"{ctx}: an externally-assumed edge must flow from a "
                    "named external assumption — nothing internal "
                    "manufactures the outside act"
                )
            raise LedgerError(
                f"{ctx}: only an externally-assumed edge may flow from an "
                "external assumption"
            )
        trip = (f, t, rec["flow_kind"])
        if trip in triples:
            raise LedgerError(
                f"{ctx}: duplicate dependency edge {trip} — the map's grain "
                "is one edge per flow kind per ordered pair; merge the prose"
            )
        triples.add(trip)
        if rec["lifecycle_path"] not in LIFECYCLE_PATHS:
            raise LedgerError(
                f"{ctx}: lifecycle_path must be one of {LIFECYCLE_PATHS} — "
                "the three ratified paths stay asymmetric and are never "
                "flattened"
            )
        for key in ("interim_continuity", "remedy_route", "restoration",
                    "systemic_correction"):
            require_str(rec, key, ctx)
        sat = rec["structural_satisfiability"]
        exact_keys(sat, ["satisfiability_status", "defect_refs", "reason"],
                   f"{ctx}.structural_satisfiability")
        require_str(sat, "reason", f"{ctx}.structural_satisfiability")
        expected_sat = {
            "constitutionally-guaranteed": "specified-interface",
            "democratically-selected": "specified-interface",
            "operationally-supplied": "operation-deferred",
            "externally-assumed": "external-contingent",
        }[cls]
        sat_disposition = sat["satisfiability_status"]
        if sat_disposition not in {expected_sat, "unsatisfiable"}:
            raise LedgerError(
                f"{ctx}: {cls} requires {expected_sat!r} or an explicit "
                "unsatisfiable disposition"
            )
        defect_refs = sat["defect_refs"]
        if (not isinstance(defect_refs, list)
                or len(set(defect_refs)) != len(defect_refs)):
            raise LedgerError(
                f"{ctx}: satisfiability defect_refs must be unique"
            )
        if sat_disposition == "unsatisfiable" and not defect_refs:
            raise LedgerError(f"{ctx}: unsatisfiable requires a named defect")
        if sat_disposition != "unsatisfiable" and defect_refs:
            raise LedgerError(
                f"{ctx}: only unsatisfiable may carry defect_refs"
            )
        for defect_ref in defect_refs:
            if ids.get(defect_ref) != "defects":
                raise LedgerError(
                    f"{ctx}: satisfiability defect must name an FS-DFT row"
                )
        closure_components = rec["closure_component_refs"]
        if (not isinstance(closure_components, list)
                or len(closure_components) != len(set(closure_components))
                or any(not isinstance(ref, str)
                       or not re.match(r"^FS-CLR-\d+:[a-z0-9-]+$", ref)
                       for ref in closure_components)):
            raise LedgerError(
                f"{ctx}: closure_component_refs must be unique typed tokens"
            )
        ar = rec["alternate_route"]
        if not isinstance(ar, dict):
            raise LedgerError(f"{ctx}: alternate_route must be an object")
        shapes = [k for k in ("route", "no_alternate_reason") if k in ar]
        if len(shapes) != 1:
            raise LedgerError(
                f"{ctx}: alternate_route carries exactly one of route / "
                f"no_alternate_reason, got {shapes} — an absent alternate "
                "is a recorded single point of failure, never a silent one"
            )
        if "route" in ar:
            exact_keys(ar, ["route", "source_ref"],
                       f"{ctx}.alternate_route", optional=["misuse_note"])
            require_str(ar, "route", f"{ctx}.alternate_route")
            validate_reference(ar["source_ref"],
                               f"{ctx}.alternate_route.source_ref")
            if "misuse_note" in ar:
                require_str(ar, "misuse_note", f"{ctx}.alternate_route")
        else:
            exact_keys(ar, ["no_alternate_reason"], f"{ctx}.alternate_route")
            require_str(ar, "no_alternate_reason", f"{ctx}.alternate_route")
        srcs = rec["source_refs"]
        if not isinstance(srcs, list) or not srcs:
            raise LedgerError(f"{ctx}: source_refs must be non-empty")
        for j, ref in enumerate(srcs):
            validate_reference(ref, f"{ctx}.source_refs[{j}]")
        touched |= {x for x in (f, t) if x in domain_ids}
        if ids[f] == "external_assumptions":
            cited_exas.add(f)
        exercised.add(rec["flow_kind"])
        edges[rec["id"]] = rec
    loops = src.get("dependency_loops")
    if not isinstance(loops, list) or not loops:
        raise LedgerError(
            "dependency_loops must be a non-empty list once dependencies "
            "populate — an acyclic whole-society map would misstate the "
            "ratified fiscal and accountability circles"
        )
    seen_loops = set()
    loop_nodesets = []
    for i, loop in enumerate(loops):
        ctx = f"dependency_loops[{i}] ({loop.get('id', '?')})"
        exact_keys(loop, ["id", "loop_kind", "member_edge_refs", "boundedness",
                          "steward_ref", "owner_ref"], ctx)
        if (loop["steward_ref"] not in ids
                or ids[loop["steward_ref"]] != "bodies"):
            raise LedgerError(
                f"{ctx}: steward_ref must name a required body, got "
                f"{loop['steward_ref']!r}"
            )
        if loop["loop_kind"] not in LOOP_KINDS:
            raise LedgerError(
                f"{ctx}: unknown loop_kind {loop['loop_kind']!r}"
            )
        mem = loop["member_edge_refs"]
        if (not isinstance(mem, list) or len(mem) < 2
                or len(set(mem)) != len(mem)):
            raise LedgerError(
                f"{ctx}: member_edge_refs must be a duplicate-free ordered "
                "list of at least two edges"
            )
        for m in mem:
            if m not in edges:
                raise LedgerError(
                    f"{ctx}: loop member must name a dependency edge, "
                    f"got {m!r}"
                )
        for j, m in enumerate(mem):
            nxt = edges[mem[(j + 1) % len(mem)]]
            if edges[m]["to_ref"] != nxt["from_ref"]:
                raise LedgerError(
                    f"{ctx}: loop members must chain into a cycle — "
                    f"{m} ends at {edges[m]['to_ref']!r}, "
                    f"{nxt['id']} starts at {nxt['from_ref']!r}"
                )
        key = frozenset(mem)
        if key in seen_loops:
            raise LedgerError(f"{ctx}: duplicate loop {sorted(key)}")
        seen_loops.add(key)
        require_str(loop, "boundedness", ctx)
        validate_reference(loop["owner_ref"], f"{ctx}.owner_ref")
        nodes = set()
        for m in mem:
            nodes.add(edges[m]["from_ref"])
            nodes.add(edges[m]["to_ref"])
        loop_nodesets.append(nodes)
    walls = src.get("refused_flows")
    if not isinstance(walls, list) or not walls:
        raise LedgerError(
            "refused_flows must be a non-empty list once dependencies "
            "populate — the ratified walls are part of the map"
        )
    seen_walls = set()
    for i, ent in enumerate(walls):
        ctx = f"refused_flows[{i}]"
        exact_keys(ent, ["refused_flow", "flow_kind", "refusal_reason",
                         "source_ref"], ctx)
        require_str(ent, "refused_flow", ctx)
        if ent["flow_kind"] not in FLOW_KINDS:
            raise LedgerError(
                f"{ctx}: unknown flow_kind {ent['flow_kind']!r}"
            )
        require_str(ent, "refusal_reason", ctx)
        validate_reference(ent["source_ref"], f"{ctx}.source_ref")
        if ent["refused_flow"] in seen_walls:
            raise LedgerError(
                f"{ctx}: duplicate refused flow {ent['refused_flow']!r}"
            )
        seen_walls.add(ent["refused_flow"])
    # closures — ORDER IS LOAD-BEARING for the negative controls:
    # domain -> flow kind -> external assumption -> cycle coverage
    missing = sorted(domain_ids - touched)
    if missing:
        raise LedgerError(
            "dependency/domain closure: every material domain participates "
            f"in the flow map; no edge touches: {missing}"
        )
    missing_kinds = sorted(set(FLOW_KINDS) - exercised)
    if missing_kinds:
        raise LedgerError(
            "flow-kind closure: every named flow kind must be exercised by "
            f"at least one edge; unexercised: {missing_kinds}"
        )
    missing_exas = sorted(exa_ids - cited_exas)
    if missing_exas:
        raise LedgerError(
            "external-assumption closure: every named external assumption "
            f"must feed at least one externally-assumed edge; uncited: "
            f"{missing_exas}"
        )
    adj = {}
    for e in edges.values():
        adj.setdefault(e["from_ref"], set()).add(e["to_ref"])
    reach = {n: _reach(adj, n) for n in adj}
    nodes = set(adj)
    for targets in adj.values():
        nodes |= targets
    assigned = set()
    for n in sorted(nodes):
        if n in assigned:
            continue
        scc = {m for m in nodes
               if m in reach.get(n, ()) and n in reach.get(m, ())}
        if n in reach.get(n, ()):
            scc.add(n)
        if len(scc) < 2:
            continue
        assigned |= scc
        if not any(ns <= scc for ns in loop_nodesets):
            raise LedgerError(
                "cycle closure: a strongly connected region has no "
                f"declared loop witness: {sorted(scc)} — a cyclic region "
                "is classified and bounded, never silent"
            )


def validate_scenarios(src: dict, ids: dict):
    """The whole-society journeys, collisions, and stress-case catalogue.

    A scenario is reviewed inventory — kind I in the assurance portfolio:
    citable as a reviewed threat model, never as proof or a counterexample
    harness. Each record routes an owned ordinary, failure, and recovery
    path; a route is routing, never delivery, and no record claims
    execution — constitutional cases execute only after the relevant
    author rulings and contract cards land, and the closure audit consumes
    this population. Mechanical closures: no domain still defers scenario
    applicability, every domain is reached, every kind, named collision
    axis, and named compound shock is exercised, every ratified
    protected-sphere test is exercised against the protected domain, and
    every critical dependency edge is stressed or its omission recorded.
    Whether the routes would hold is NOT decided here — capacity and
    degradation are Book 2's tests, and sufficiency stays a question for
    the independent scope review."""
    scenarios = src.get("scenarios", [])
    if not scenarios:
        if src.get("scenario_omissions"):
            raise LedgerError(
                "scenario_omissions may not exist while scenarios is "
                "deferred — an omission is a decision about a populated "
                "catalogue"
            )
        return
    domain_ids = {r for r, a in ids.items() if a == "domains"}
    dep_ids = {r for r, a in ids.items() if a == "dependencies"}
    if PROTECTED_SPHERE_DOMAIN not in domain_ids:
        raise LedgerError(
            "the protected private/civic domain must exist before "
            "scenarios can classify against it"
        )
    witness_pool = None
    cited_domains, cited_deps = set(), set()
    kinds, axes, shocks, forms = set(), set(), set(), set()
    for i, rec in enumerate(scenarios):
        ctx = f"scenarios[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["scenario_kind", "domain_refs",
                           "dependency_refs", "steward_ref",
                           "ordinary_route", "failure_route",
                           "recovery_route", "source_refs"],
            ctx,
            optional=["collision_axis", "shock_kind",
                      "protected_sphere_forms", "bounded_witness_refs"],
        )
        validate_common_record_fields(rec, ctx)
        if rec["status"] != SCENARIO_STATUS:
            raise LedgerError(
                f"{ctx}: a scenario's status is the exact literal "
                f"{SCENARIO_STATUS!r} — the catalogue is reviewed "
                "inventory and no row may claim execution or assurance"
            )
        if rec["layer"] != "constitutional-invariant":
            raise LedgerError(
                f"{ctx}: a scenario states Book 1 invariant and failure "
                "behaviour — capacity and degradation are Book 2's tests; "
                "its layer is constitutional-invariant, and layer-specific "
                "rule content stays on domains, claims, and edges"
            )
        if rec["scenario_kind"] not in SCENARIO_KINDS:
            raise LedgerError(
                f"{ctx}: unknown scenario_kind {rec['scenario_kind']!r}"
            )
        kinds.add(rec["scenario_kind"])
        refs = rec["domain_refs"]
        if not isinstance(refs, list) or not refs \
                or len(set(refs)) != len(refs):
            raise LedgerError(
                f"{ctx}: domain_refs must be a non-empty duplicate-free "
                "list"
            )
        for ref in refs:
            if ref not in domain_ids:
                raise LedgerError(
                    f"{ctx}: domain_refs must name domains, got {ref!r}"
                )
        cited_domains |= set(refs)
        drefs = rec["dependency_refs"]
        if not isinstance(drefs, list) or len(set(drefs)) != len(drefs):
            raise LedgerError(
                f"{ctx}: dependency_refs must be a duplicate-free list"
            )
        for ref in drefs:
            if ref not in dep_ids:
                raise LedgerError(
                    f"{ctx}: dependency_refs must name dependency edges, "
                    f"got {ref!r}"
                )
        cited_deps |= set(drefs)
        if (rec["steward_ref"] not in ids
                or ids[rec["steward_ref"]] != "bodies"):
            raise LedgerError(
                f"{ctx}: steward_ref must name a required body — the "
                "institution answerable for the routes; got "
                f"{rec['steward_ref']!r}"
            )
        if rec["scenario_kind"] == "collision":
            if "collision_axis" not in rec:
                raise LedgerError(
                    f"{ctx}: a collision scenario names its axis — the "
                    "named axes are the closed test targets"
                )
            if rec["collision_axis"] not in COLLISION_AXES:
                raise LedgerError(
                    f"{ctx}: unknown collision_axis "
                    f"{rec['collision_axis']!r}"
                )
            axes.add(rec["collision_axis"])
        elif "collision_axis" in rec:
            raise LedgerError(
                f"{ctx}: a collision axis belongs only on a collision "
                "scenario"
            )
        if rec["scenario_kind"] == "compound-shock":
            if "shock_kind" not in rec:
                raise LedgerError(
                    f"{ctx}: a compound-shock scenario names its shock — "
                    "the named shocks are the closed test targets"
                )
            if rec["shock_kind"] not in SHOCK_KINDS:
                raise LedgerError(
                    f"{ctx}: unknown shock_kind {rec['shock_kind']!r}"
                )
            shocks.add(rec["shock_kind"])
        elif "shock_kind" in rec:
            raise LedgerError(
                f"{ctx}: a shock kind belongs only on a compound-shock "
                "scenario"
            )
        if PROTECTED_SPHERE_DOMAIN in refs:
            psf = rec.get("protected_sphere_forms")
            if psf is None:
                raise LedgerError(
                    f"{ctx}: a scenario citing {PROTECTED_SPHERE_DOMAIN} "
                    "classifies which protected-sphere tests it exercises"
                )
            if (not isinstance(psf, list) or not psf
                    or len(set(psf)) != len(psf)
                    or any(f not in PROTECTED_SPHERE_FORMS for f in psf)):
                raise LedgerError(
                    f"{ctx}: protected_sphere_forms must be a non-empty "
                    f"duplicate-free subset of {PROTECTED_SPHERE_FORMS}"
                )
            forms |= set(psf)
        elif "protected_sphere_forms" in rec:
            raise LedgerError(
                f"{ctx}: protected_sphere_forms belongs only on a "
                f"scenario citing {PROTECTED_SPHERE_DOMAIN}"
            )
        for key in ("ordinary_route", "failure_route", "recovery_route"):
            require_str(rec, key, ctx)
        bw = rec.get("bounded_witness_refs")
        if bw is not None:
            if not isinstance(bw, list) or not bw \
                    or len(set(bw)) != len(bw):
                raise LedgerError(
                    f"{ctx}: bounded_witness_refs, if present, must be a "
                    "non-empty duplicate-free list"
                )
            if witness_pool is None:
                witness_pool = collect_bounded_witnesses()
            for tok in bw:
                if tok not in witness_pool:
                    raise LedgerError(
                        f"{ctx}: {tok!r} names no case in the live "
                        "sibling witness pool — a witness is a real "
                        "sibling row, never a fabricated execution"
                    )
        srcs = rec["source_refs"]
        if not isinstance(srcs, list) or not srcs:
            raise LedgerError(f"{ctx}: source_refs must be non-empty")
        for j, ref in enumerate(srcs):
            rel = ref.split("::", 1)[0] if isinstance(ref, str) else ""
            if rel.startswith(("book-1/", "book-2/")) \
                    or rel in ("book.md", "manifesto.md"):
                raise LedgerError(
                    f"{ctx}.source_refs[{j}]: a book-prose passage may "
                    "never support a scenario row — cite the ruling, "
                    "coverage-map, or planning source "
                    "(narrative-register rule)"
                )
            validate_reference(ref, f"{ctx}.source_refs[{j}]")
    om = src.get("scenario_omissions")
    if not isinstance(om, list) or not om:
        raise LedgerError(
            "scenario_omissions must be a non-empty list once scenarios "
            "populate — a bounded catalogue records what it classifies out"
        )
    omitted_deps, seen = set(), set()
    for i, entry in enumerate(om):
        ctx = f"scenario_omissions[{i}]"
        axis_keys = [k for k in ("omitted_scenario",
                                 "omitted_dependency_ref") if k in entry]
        if len(axis_keys) != 1:
            raise LedgerError(
                f"{ctx}: exactly one of omitted_scenario / "
                "omitted_dependency_ref"
            )
        exact_keys(entry, [axis_keys[0], "risk_reason", "source_ref"], ctx)
        if axis_keys[0] == "omitted_dependency_ref":
            ref = entry["omitted_dependency_ref"]
            if ref not in dep_ids:
                raise LedgerError(f"{ctx}: unknown dependency {ref!r}")
            if ref in cited_deps:
                raise LedgerError(
                    f"{ctx}: stale omission — a scenario already "
                    f"stresses {ref}"
                )
            omitted_deps.add(ref)
            key = ("dependency", ref)
        else:
            require_str(entry, "omitted_scenario", ctx)
            key = ("scenario", entry["omitted_scenario"])
        if key in seen:
            raise LedgerError(f"{ctx}: duplicate omission {key}")
        seen.add(key)
        require_str(entry, "risk_reason", ctx)
        validate_reference(entry["source_ref"], f"{ctx}.source_ref")
    # closures — ORDER IS LOAD-BEARING for the negative controls:
    # applicability coupling -> domain -> kind -> collision axis ->
    # shock kind -> protected sphere -> critical dependency
    still_deferred = sorted(
        d["id"] for d in src.get("domains", [])
        if "deferred_ref" in d["scenario_applicability"]
    )
    if still_deferred:
        raise LedgerError(
            "scenario-applicability coupling: a populated catalogue "
            "leaves no domain deferring — still defers scenario "
            f"applicability: {still_deferred}"
        )
    missing_domains = sorted(domain_ids - cited_domains)
    if missing_domains:
        raise LedgerError(
            "scenario/domain closure: every domain needs at least one "
            f"whole-society scenario; no scenario reaches: "
            f"{missing_domains}"
        )
    missing_kinds = sorted(set(SCENARIO_KINDS) - kinds)
    if missing_kinds:
        raise LedgerError(
            "scenario-kind closure: every scenario kind must be "
            f"exercised by at least one record; missing kinds: "
            f"{missing_kinds}"
        )
    missing_axes = sorted(set(COLLISION_AXES) - axes)
    if missing_axes:
        raise LedgerError(
            "collision-axis closure: every named collision axis must be "
            f"tested by at least one collision scenario; untested: "
            f"{missing_axes}"
        )
    missing_shocks = sorted(set(SHOCK_KINDS) - shocks)
    if missing_shocks:
        raise LedgerError(
            "shock-kind closure: every named compound shock must be "
            f"carried by at least one compound-shock scenario; uncarried: "
            f"{missing_shocks}"
        )
    missing_forms = sorted(set(PROTECTED_SPHERE_FORMS) - forms)
    if missing_forms:
        raise LedgerError(
            "protected-sphere closure: each ratified protected-sphere "
            "test needs a scenario citing the protected domain; "
            f"unexercised forms: {missing_forms}"
        )
    crit = {d["id"] for d in src.get("dependencies", [])
            if d["severity"] == "critical"}
    uncovered = sorted(crit - cited_deps - omitted_deps)
    if uncovered:
        raise LedgerError(
            "critical-dependency closure: every critical dependency edge "
            "is stressed by a scenario or its omission is recorded; "
            f"unstressed: {uncovered}"
        )


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


def _envelope_calibrated(src: dict, envelope_id) -> bool:
    for rec in src.get("envelope", []):
        if rec.get("id") == envelope_id:
            return rec.get("envelope_status") == "calibrated"
    return False


def envelope_ids(src: dict) -> set:
    return {rec.get("id") for rec in src.get("envelope", [])}


def validate_envelope(src: dict, ids: dict):
    env = src.get("envelope", [])
    if not env or env[0].get("id") != ENVELOPE_STUB_ID:
        raise LedgerError(
            f"the envelope array begins with the permanent {ENVELOPE_STUB_ID} "
            "pre-envelope identity"
        )
    claims_by_id = {c["id"]: c for c in src.get("claims", [])}
    for i, rec in enumerate(env):
        ctx = f"envelope[{i}] ({rec.get('id', '?')})"
        if rec.get("envelope_status") == "calibrated":
            raise LedgerError(
                f"{ctx}: calibration is a deliberate future contract "
                "amendment — this contract refuses a calibrated envelope, "
                "because calibration is Book 2's Gate D work and a string "
                "flip must never unlock what only values can"
            )
        if rec.get("envelope_status") not in ENVELOPE_STATUSES:
            raise LedgerError(f"{ctx}: unknown envelope_status")
        if rec["layer"] != "external-assumption":
            raise LedgerError(
                f"{ctx}: until calibrated, an envelope is an external "
                "assumption"
            )
        if i == 0:
            exact_keys(rec, COMMON_KEYS + ["envelope_status", "note"], ctx)
            validate_common_record_fields(rec, ctx)
            if rec["envelope_status"] != "stub":
                raise LedgerError(f"{ctx}: the first record is the stub")
            if rec["status"] != "pre-envelope-identity":
                raise LedgerError(
                    f"{ctx}: the stub's status is pre-envelope-identity — it "
                    "is retained forever as the keying identity for records "
                    "landed before the envelope was versioned"
                )
            require_str(rec, "note", ctx)
            continue
        exact_keys(rec, COMMON_KEYS + ["envelope_status", "envelope_version",
                                       "note", "fields"], ctx)
        validate_common_record_fields(rec, ctx)
        if rec["envelope_status"] != "versioned-structure":
            raise LedgerError(
                f"{ctx}: a successor record is versioned-structure in this "
                "contract"
            )
        require_str(rec, "envelope_version", ctx)
        require_str(rec, "note", ctx)
        fields = rec["fields"]
        if not isinstance(fields, list) or not fields:
            raise LedgerError(f"{ctx}: a versioned envelope declares its fields")
        seen = set()
        dependents_everywhere = set()
        for j, field in enumerate(fields):
            fctx = f"{ctx}.fields[{j}] ({field.get('id', '?')})"
            exact_keys(field, ["id", "definition", "value_status",
                               "book2_owner_ref", "dependents", "invariance"],
                       fctx)
            fid = field.get("id")
            if not isinstance(fid, str) or not SLUG_RE.match(fid):
                raise LedgerError(f"{fctx}: field id must be a kebab-case slug")
            if fid in seen:
                raise LedgerError(f"{fctx}: duplicate field id")
            seen.add(fid)
            require_str(field, "definition", fctx)
            if field["value_status"] not in VALUE_STATUSES:
                raise LedgerError(
                    f"{fctx}: value_status must be declared-pending — values "
                    "are Book 2's Gate D calibration, never Book 1 content"
                )
            validate_reference(field["book2_owner_ref"],
                              f"{fctx}.book2_owner_ref")
            deps = field["dependents"]
            if not isinstance(deps, list):
                raise LedgerError(f"{fctx}: dependents must be a list")
            inv = field.get("invariance")
            if not deps and (not isinstance(inv, str) or not inv):
                raise LedgerError(
                    f"{fctx}: a field lists dependents or an explicit "
                    "invariance statement"
                )
            if not isinstance(inv, str) or not inv:
                raise LedgerError(f"{fctx}: invariance must be stated")
            for dep in deps:
                claim = claims_by_id.get(dep)
                if claim is None:
                    raise LedgerError(f"{fctx}: dependent {dep} is no claim")
                if claim["layer"] == "constitutional-invariant" and \
                        claim["posture"] in ("Derived", "Checked"):
                    raise LedgerError(
                        f"{fctx}: norm-content invariance — an established "
                        "constitutional invariant may not depend on an "
                        f"envelope field ({dep})"
                    )
                dependents_everywhere.add(dep)
        for req in REQUIRED_DEPENDENTS:
            if req not in dependents_everywhere:
                raise LedgerError(
                    f"{ctx}: envelope-relative claim {req} must appear as a "
                    "dependent of some field — its establishment varies with "
                    "the envelope even though its norm content does not"
                )


def validate_functional_criteria(src: dict):
    block = src.get("functional_criteria")
    if not isinstance(block, dict):
        raise LedgerError("functional_criteria must be an object")
    exact_keys(block, ["criteria", "drift_note"], "functional_criteria")
    require_str(block, "drift_note", "functional_criteria")
    criteria = block["criteria"]
    if not isinstance(criteria, list):
        raise LedgerError("functional_criteria.criteria must be a list")
    seen = set()
    for i, rec in enumerate(criteria):
        ctx = f"functional_criteria[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, ["id", "name", "definition", "binding_refs",
                         "provenance"], ctx)
        require_str(rec, "name", ctx)
        require_str(rec, "definition", ctx)
        if rec.get("id") not in CRITERIA_SLUGS:
            raise LedgerError(f"{ctx}: unknown criterion slug")
        if rec["id"] in seen:
            raise LedgerError(f"{ctx}: duplicate criterion")
        seen.add(rec["id"])
        refs = rec["binding_refs"]
        if not isinstance(refs, list) or not refs:
            raise LedgerError(
                f"{ctx}: a criterion binds the rulings' actual sentences by "
                "needle, never a paraphrase"
            )
        for j, ref in enumerate(refs):
            validate_reference(ref, f"{ctx}.binding_refs[{j}]")
        prov = rec["provenance"]
        if not isinstance(prov, list) or not prov or \
                any(not isinstance(s, str) or not s for s in prov):
            raise LedgerError(
                f"{ctx}: provenance lists every ratified text naming this "
                "criterion"
            )
    if seen != set(CRITERIA_SLUGS):
        raise LedgerError(
            "functional_criteria must carry exactly the seven-member union: "
            + ", ".join(CRITERIA_SLUGS)
        )


def validate_thresholds(src: dict, ids: dict):
    for i, rec in enumerate(src.get("thresholds", [])):
        ctx = f"thresholds[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["criterion_ref", "domain_refs", "definition",
                           "binding_ref", "lawful_source",
                           "decision_owner_ref", "measurement_owner_ref",
                           "value_status"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec.get("criterion_ref") not in CRITERIA_SLUGS:
            raise LedgerError(f"{ctx}: unknown criterion")
        for ref in rec["domain_refs"]:
            if ref not in ids or ids[ref] != "domains":
                raise LedgerError(f"{ctx}: domain_refs must name domains")
        require_str(rec, "definition", ctx)
        if re.search(r"\d", rec["definition"]):
            raise LedgerError(
                f"{ctx}: no numeric value may appear in a Book 1 threshold — "
                "meanings, not measurements"
            )
        validate_reference(rec["binding_ref"], f"{ctx}.binding_ref")
        ls = rec.get("lawful_source")
        if ls not in LAWFUL_SOURCES:
            raise LedgerError(f"{ctx}: unknown lawful_source")
        if rec["layer"] != LAWFUL_SOURCE_LAYER[ls]:
            raise LedgerError(
                f"{ctx}: layer must follow the lawful source — {ls} fixes "
                f"{LAWFUL_SOURCE_LAYER[ls]}"
            )
        validate_reference(rec["decision_owner_ref"],
                          f"{ctx}.decision_owner_ref")
        validate_reference(rec["measurement_owner_ref"],
                          f"{ctx}.measurement_owner_ref")
        if rec["value_status"] not in VALUE_STATUSES:
            raise LedgerError(f"{ctx}: value_status must be declared-pending")


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


def expected_defect_gate_refs(defect_id: str):
    matches = [
        list(gates) for gates, defect_ids in DEFECT_GATE_GROUPS.items()
        if defect_id in defect_ids
    ]
    if len(matches) != 1:
        raise LedgerError(
            f"{defect_id}: checker-owned gate-applicability contract has "
            f"{len(matches)} matches; a deliberate contract update is required"
        )
    return matches[0]


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
                           "evidence_notes", "residual_citations", "controls",
                           "applicable_gate_refs"],
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
        if rec["envelope_id"] not in envelope_ids(src):
            raise LedgerError(
                f"{ctx}: envelope_id names no envelope record"
            )
        if stage == "operationally-assured-in-envelope" and \
                not _envelope_calibrated(src, rec["envelope_id"]):
            raise LedgerError(
                f"{ctx}: the operationally-assured stage requires a "
                f"calibrated envelope; {rec['envelope_id']} can route, never "
                "assure — the stub and a structure-only envelope alike"
            )
        for key in ("resolution_status", "blocking"):
            if key in rec:
                raise LedgerError(
                    f"{ctx}: {key} is generated, never hand-authored"
                )
        gate_refs = rec["applicable_gate_refs"]
        if not isinstance(gate_refs, list) or not gate_refs:
            raise LedgerError(
                f"{ctx}: applicable_gate_refs must be a non-empty list"
            )
        if len(gate_refs) != len(set(gate_refs)):
            raise LedgerError(
                f"{ctx}: applicable_gate_refs contains duplicates"
            )
        unknown_gates = sorted(set(gate_refs) - set(GATE_REFS))
        if unknown_gates:
            raise LedgerError(
                f"{ctx}: unknown applicable gate refs {unknown_gates}"
            )
        canonical_gates = [gate for gate in GATE_REFS if gate in gate_refs]
        if gate_refs != canonical_gates:
            raise LedgerError(
                f"{ctx}: applicable_gate_refs must follow canonical gate order"
            )
        expected_gates = expected_defect_gate_refs(rec["id"])
        if gate_refs != expected_gates:
            raise LedgerError(
                f"{ctx}: applicable_gate_refs must equal the checker-owned "
                f"gate-applicability contract {expected_gates}; classification "
                "cannot hide a critical defect"
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
                and _envelope_calibrated(src, rec["envelope_id"])
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


def collect_bounded_witnesses():
    """Live-read the sibling case inventories with stable row ids. A witness
    token records that a bounded sibling case already exercises a slice of a
    scenario's subject matter; it establishes only what that artifact's own
    posture states — never that this scenario executed — and never upgrades
    this catalogue's inventory kind. Live-read means a renamed sibling case
    fails --check, the house drift rule."""
    pool = set()
    rt = load_json(pathlib.Path("new-book-plans/record-integrity-red-team.json"))
    pool |= {f"record-integrity-red-team#{s['id']}" for s in rt["scenarios"]}
    ta = load_json(pathlib.Path("new-book-plans/temporal-assurance-case.json"))
    pool |= {f"temporal-assurance-case#{c['id']}" for c in ta["cases"]}
    am = load_json(pathlib.Path("new-book-plans/amendment-semantics-audit.json"))
    pool |= {f"amendment-semantics-audit#{c['id']}" for c in am["cases"]}
    pe = load_json(pathlib.Path(
        "new-book-plans/placement-exhaustiveness-audit.json"))
    pool |= {f"placement-exhaustiveness-audit#{m['id']}"
             for m in pe["mutations"]}
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
        staged_power_prefix = (
            array == "powers" and rows and array in by_type
            and src["power_population"]["status"] == "partial"
        )
        if rows and array in by_type and not staged_power_prefix:
            raise LedgerError(
                f"{array} is populated but still carries a deferral record"
            )


def collect_sibling_enums():
    """Live-read the seven sibling reviewed JSONs and inventory their enums."""
    found = []
    for path in SIBLING_SOURCES:
        data = load_json(path)
        leaf_keys = ENUM_LEAF_KEYS
        if path == READER_EVIDENCE_SOURCE:
            leaf_keys = leaf_keys | READER_ENUM_LEAF_KEYS
        for key, val in data.items():
            if key.endswith("_meanings") and isinstance(val, dict):
                for value in val:
                    found.append((path.name, key, value))

        def walk(obj, field_path):
            if isinstance(obj, dict):
                for k, v in obj.items():
                    if isinstance(v, str) and k in leaf_keys:
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
        "Status | Gap / author ruling required | Split claims (posture) | "
        "Source-derived power cards |",
        "| --- | --- | --- | --- | --- | --- | --- |",
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
                "; ".join(
                    power["id"] for power in src["powers"]
                    if set(row["domain_refs"]) & set(power["domain_refs"])
                ) or "—",
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
            if rec["protocol_ref"].split("::", 1)[0] != str(PROTOCOL_DOC):
                raise LedgerError(
                    f"{ctx}: an independent event must run under the "
                    f"scope-review protocol ({PROTOCOL_DOC})"
                )
            commitment = (src.get("review_protocol") or {}).get(
                "review_commitment")
            if not commitment:
                raise LedgerError(
                    f"{ctx}: an independent event requires a pre-registered "
                    "review commitment — plant and seed digests enter this "
                    "source at commissioning, before the packet is assembled"
                )
            open_date = rec["received_window"].split(" ", 1)[0]
            if not ISO_DATE_RE.match(open_date):
                raise LedgerError(
                    f"{ctx}: an independent event's received_window opens "
                    "with an ISO date (YYYY-MM-DD …) so the commitment's "
                    "precedence is checkable"
                )
            if commitment.get("committed_date", "") > open_date:
                raise LedgerError(
                    f"{ctx}: the review commitment must precede the received "
                    "window — a commitment that postdates its event is no "
                    "commitment"
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


def validate_review_protocol(src: dict):
    """The scope-review protocol binding. The protocol document carries R7's
    evidence contract and admissibility criteria, author-confirmed with the
    basis recorded; the commitment record stays null and the designation
    absent until the author commissions a review. Pre-images are author-held
    outside the repository — only their SHA-256 digests ever enter this
    source, and an independent review event is refused without them
    (validate_review_events). The designation names the severity owner and
    independent checker: real recorded identities, distinct people, neither
    the pre-image custodian — the mechanical checks are exact-string form;
    the substance (one person under two spellings, or a label standing in
    for a person) is reviewed, not computed."""
    if "review_protocol" not in src:
        raise LedgerError(
            "review_protocol must be present — the scope-review protocol "
            "binding, null commitment until the author commissions a review"
        )
    rp = src["review_protocol"]
    ctx = "review_protocol"
    if not isinstance(rp, dict):
        raise LedgerError(f"{ctx} must be an object")
    exact_keys(rp, ["protocol_ref", "protocol_status", "status_line_ref",
                    "review_commitment"], ctx,
               optional=["confirmation_basis", "designation"])
    status = require_str(rp, "protocol_status", ctx)
    if status == PROTOCOL_STATUS_CANDIDATE:
        if "confirmation_basis" in rp:
            raise LedgerError(
                f"{ctx}: a candidate protocol carries no confirmation basis"
            )
    elif status == PROTOCOL_STATUS_CONFIRMED:
        require_str(rp, "confirmation_basis", ctx)
    else:
        raise LedgerError(
            f"{ctx}: protocol_status must be exactly "
            f"{PROTOCOL_STATUS_CANDIDATE!r} or "
            f"{PROTOCOL_STATUS_CONFIRMED!r} — confirmation is a recorded "
            "author act, never a rewording"
        )
    if rp["review_commitment"] is not None and \
            status != PROTOCOL_STATUS_CONFIRMED:
        raise LedgerError(
            f"{ctx}: a commitment may exist only against a confirmed "
            "protocol — confirmation precedes commissioning"
        )
    designation = rp.get("designation")
    if designation is not None:
        if status != PROTOCOL_STATUS_CONFIRMED:
            raise LedgerError(
                f"{ctx}: a designation may exist only against a confirmed "
                "protocol — confirmation precedes commissioning"
            )
        dctx = f"{ctx}.designation"
        if not isinstance(designation, dict):
            raise LedgerError(f"{dctx} must be an object")
        exact_keys(designation,
                   ["severity_owner", "independent_checker", "custodian",
                    "designated_date", "basis"], dctx)
        for key in ("severity_owner", "independent_checker", "custodian",
                    "basis"):
            require_str(designation, key, dctx)
        ddate = require_str(designation, "designated_date", dctx)
        if not ISO_DATE_RE.match(ddate):
            raise LedgerError(
                f"{dctx}.designated_date must be an ISO date (YYYY-MM-DD)"
            )
        if designation["severity_owner"] == \
                designation["independent_checker"]:
            raise LedgerError(
                f"{dctx}: the severity owner and the independent checker "
                "must be distinct people"
            )
        for key in ("severity_owner", "independent_checker"):
            if designation[key] == designation["custodian"]:
                raise LedgerError(
                    f"{dctx}: the pre-image custodian may not triage — "
                    f"{key} equals the custodian identity"
                )
    ref = require_str(rp, "protocol_ref", ctx)
    validate_reference(ref, f"{ctx}.protocol_ref")
    if ref.split("::", 1)[0] != str(PROTOCOL_DOC):
        raise LedgerError(
            f"{ctx}.protocol_ref must resolve into the scope-review protocol "
            f"document ({PROTOCOL_DOC})"
        )
    line_ref = require_str(rp, "status_line_ref", ctx)
    validate_reference(line_ref, f"{ctx}.status_line_ref")
    line_path, line_needle = line_ref.split("::", 1)
    if line_path != str(PROTOCOL_DOC):
        raise LedgerError(
            f"{ctx}.status_line_ref must resolve into the scope-review "
            f"protocol document ({PROTOCOL_DOC})"
        )
    if line_needle != "Status: " + status:
        raise LedgerError(
            f"{ctx}: the protocol document's status line must record the "
            "recorded status — the document and this source flip together or "
            "not at all"
        )
    commitment = rp["review_commitment"]
    if commitment is None:
        return
    cctx = f"{ctx}.review_commitment"
    if not isinstance(commitment, dict):
        raise LedgerError(f"{cctx} must be null or an object")
    exact_keys(commitment,
               ["plant_commitment_sha256", "seed_commitment_sha256",
                "protocol_sha256", "committed_date", "custody"], cctx)
    for key in ("plant_commitment_sha256", "seed_commitment_sha256",
                "protocol_sha256"):
        val = require_str(commitment, key, cctx)
        if not SHA256_HEX_RE.match(val):
            raise LedgerError(
                f"{cctx}.{key} must be 64 lowercase hex characters — the "
                "SHA-256 digest of an author-held pre-image or of the "
                "protocol document"
            )
    if commitment["plant_commitment_sha256"] == \
            commitment["seed_commitment_sha256"]:
        raise LedgerError(
            f"{cctx}: the plant and seed commitments must be distinct "
            "pre-images"
        )
    actual = hashlib.sha256((ROOT / PROTOCOL_DOC).read_bytes()).hexdigest()
    if commitment["protocol_sha256"] != actual:
        raise LedgerError(
            f"{cctx}.protocol_sha256 does not match the protocol document's "
            "current bytes — a commitment binds the exact protocol text; "
            "re-commission after any protocol edit"
        )
    date = require_str(commitment, "committed_date", cctx)
    if not ISO_DATE_RE.match(date):
        raise LedgerError(
            f"{cctx}.committed_date must be an ISO date (YYYY-MM-DD)"
        )
    custody = require_str(commitment, "custody", cctx)
    if "outside the repo" not in custody:
        raise LedgerError(
            f"{cctx}.custody must state that pre-images stay outside the "
            "repository — only digests ever enter this source"
        )


def _gate_a_condition_1_deferred(src: dict):
    """Return only the populations named by closure condition one.

    Proposals and review events are outputs of condition five's review.
    Counting their pre-review deferrals here would make commissioning that
    review circular.
    """
    populations = {
        "domains", "roles", "powers", "dependencies", "scenarios", "defects",
    }
    return sorted(
        row["record_type"] for row in src["deferred_populations"]
        if row["record_type"] in populations
    )


def compute_gate_a_readiness(src: dict, resolution: dict):
    """The single computation the render and the closure validator both
    consume. Statuses echo the stopping rule's closure conditions by index so
    the two texts can never drift. No aggregate is ever derived from this."""
    conds = src["stopping_rule"]["closure_conditions"]
    deferred = _gate_a_condition_1_deferred(src)
    defects_by_id = {row["id"]: row for row in src["defects"]}
    blocking = sorted(
        rid for rid, row in resolution.items()
        if row["blocking"]
        and "gate-a" in defects_by_id[rid]["applicable_gate_refs"]
    )
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
    rows.append((conds[1], "met-in-form",
                 "the coverage, role, dependency, assurance-allocation, "
                 "structural-reader, and Book 2 projections regenerate from "
                 "the canonical source; projection freshness establishes no "
                 "reader evidence or operational result"))
    if blocking:
        rows.append((conds[2], "unmet",
                     "critical unresolved defects applicable to Gate A's "
                     "map-and-test-program claim exist: "
                     + ", ".join(blocking)))
    else:
        rows.append((conds[2], "met-mechanically",
                     "no critical unresolved defect row is applicable to Gate "
                     "A's map-and-test-program claim; later-gate claim blockers "
                     "remain visible and unresolved"))
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
    successor = next(
        (r for r in src["envelope"][1:]
         if r.get("envelope_status") == "versioned-structure"), None)
    if successor is None:
        preconditions.append(("the reference envelope", "unmet-external",
                              "still the explicit stub; Gate A requires a non-stub, "
                              "versioned-structure envelope"))
    else:
        preconditions.append(("the reference envelope", "met-in-form",
                              "versioned in structure and reviewable; this satisfies "
                              "Gate A's envelope precondition. Calibration and "
                              "values remain Book 2 Gate D work, and operational "
                              "assurance and remedied resolution still require them"))
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
    if rec["envelope_ref"] not in envelope_ids(src):
        raise LedgerError(f"{ctx}: envelope_ref names no envelope record")
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
    validate_power_source_inventory(src)
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
    validate_reader_evidence_alignment(src, routes_by_id)
    validate_bodies(src)
    validate_roles(src, ids)
    validate_power_population(src, ids)
    validate_dependencies(src, ids)
    validate_scenarios(src, ids)
    validate_external_assumptions(src)
    validate_envelope(src, ids)
    validate_functional_criteria(src)
    validate_thresholds(src, ids)
    validate_defect_rows(src, ids)
    resolution = compute_resolution(src)
    validate_receipts(src, ids, resolution)
    validate_residual_coverage(src)
    validate_review_protocol(src)
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

    control("power source inventory binding is required",
            lambda s: s.pop("power_source_inventory"))
    control("power source inventory digest is exact",
            lambda s: s["power_source_inventory"].update(
                {"artifact_sha256": "0" * 64}))
    control("power source inventory row count is exact",
            lambda s: s["power_source_inventory"].update({"row_count": 236}))
    control("known power allocation gaps cannot disappear silently",
            lambda s: s["power_source_inventory"][
                "known_allocation_gaps"].pop())
    control("power final counts are checker-owned",
            lambda s: s["power_population"]["expected_final_counts"].update(
                {"powers": 211}))
    control("power family completion is an exact prefix",
            lambda s: s["power_population"].update(
                {"completed_source_families": ["time-model"],
                 "status": "partial"}))
    control("resolved power-allocation gaps are append-only and exact",
            lambda s: s["power_population"][
                "resolved_allocation_gaps"].pop())
    control("power population cannot claim complete early",
            lambda s: s["power_population"].update({"status": "complete"}))
    control("powers deferral cannot disappear during a partial prefix",
            lambda s: s["deferred_populations"].remove(next(
                row for row in s["deferred_populations"]
                if row["record_type"] == "powers")))
    control("powers deferral closure is exact",
            lambda s: next(
                row for row in s["deferred_populations"]
                if row["record_type"] == "powers"
            ).update({"closure_condition": "cards later"}))
    if src["powers"]:
        control("a power grain cannot be bundled or duplicated",
                lambda s: s["powers"][0].update(
                    {"manifest_key": s["powers"][1]["manifest_key"]}))
        control("a power profile cannot be dropped",
                lambda s: s["powers"][0]["profiles"].pop())
        control("profile fields reject blank substitutes",
                lambda s: s["powers"][0]["profile_contracts"][
                    s["powers"][0]["profiles"][0]].update({"office": "N/A"}))
        control("unknown power holder body is refused",
                lambda s: s["powers"][0]["holder_body_refs"].__setitem__(
                    0, "FS-BOD-999"))
        control("a power allocation cannot disappear",
                lambda s: s["function_allocations"].pop())
        control("one allocation cannot serve two powers",
                lambda s: s["function_allocations"][0].update(
                    {"power_ref": s["function_allocations"][1]["power_ref"]}))
        control("required function separation cannot be fused",
                lambda s: s["function_allocations"][0][
                    "auditor_body_refs"].__setitem__(
                        0, s["function_allocations"][0][
                            "decisive_fact_writer_body_refs"][0]))
    if src["power_refusals"]:
        control("a refusal cannot be promoted into a power",
                lambda s: s["powers"].append(copy.deepcopy(s["powers"][0])))
    if src["power_crosswalk_dispositions"]:
        control("formal crosswalk policy is checker-owned",
                lambda s: s["power_crosswalk_dispositions"][0].update(
                    {"crosswalk_action": "retire"}))
        control("T3 custody authority cannot merge with its executor",
                lambda s: next(
                    row for row in s["powers"]
                    if row["manifest_key"] == RETAINED_FORMAL_KEY
                ).update({"holder_body_refs": ["FS-BOD-35"]}))
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
    control("route-unbuilt requires an unbuilt route",
            _route_unbuilt_on_built, "requires an unbuilt route")
    control("evidence-pending requires a built or available route",
            _evidence_pending_on_unbuilt, "requires a built or available route")
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
    control("a defect declares gate applicability",
            lambda s: s["defects"][0].pop("applicable_gate_refs"),
            "missing keys")
    control("gate applicability is non-empty",
            lambda s: s["defects"][0].update({"applicable_gate_refs": []}),
            "non-empty")
    control("gate applicability rejects unknown gates",
            lambda s: s["defects"][0].update(
                {"applicable_gate_refs": ["gate-z"]}),
            "unknown applicable gate")
    control("gate applicability rejects duplicates",
            lambda s: s["defects"][0].update(
                {"applicable_gate_refs": ["gate-a", "gate-a"]}),
            "duplicates")
    control("gate applicability follows canonical order",
            lambda s: s["defects"][0].update(
                {"applicable_gate_refs": ["gate-b", "gate-a"]}),
            "canonical gate order")
    control("gate applicability cannot silently hide or widen a defect",
            lambda s: next(
                row for row in s["defects"] if row["id"] == "FS-DFT-16"
            ).update({"applicable_gate_refs": list(GATE_REFS)}),
            "checker-owned gate-applicability contract")
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
    control("review_protocol must be present",
            lambda s: s.pop("review_protocol"),
            "must be present")
    control("the protocol status is exact in both states",
            lambda s: s["review_protocol"].update(
                {"protocol_status": "confirmed"}),
            "recorded author act")
    control("a confirmed protocol records its basis",
            lambda s: s["review_protocol"].pop("confirmation_basis"),
            "confirmation_basis")
    control("a candidate protocol carries no basis",
            lambda s: s["review_protocol"].update(
                {"protocol_status": PROTOCOL_STATUS_CANDIDATE}),
            "carries no confirmation basis")
    control("a commitment requires a confirmed protocol",
            _commitment_on_candidate_protocol,
            "confirmed protocol")
    control("owner and checker are distinct people",
            _designation_owner_is_checker, "distinct people")
    control("the custodian may not triage as severity owner",
            _designation_owner_is_custodian, "may not triage")
    control("the custodian may not triage as independent checker",
            _designation_checker_is_custodian, "may not triage")
    control("a designation requires a confirmed protocol",
            _designation_on_candidate_protocol, "confirmed protocol")
    control("a designation date is an ISO date",
            lambda s: _mk_designation(s).update(
                {"designated_date": "someday"}),
            "YYYY-MM-DD")
    control("a designation carries exactly its declared keys",
            lambda s: _mk_designation(s).update({"note": "extra"}),
            "unexpected keys")
    control("the protocol's status line is live-checked",
            lambda s: s["review_protocol"].update(
                {"status_line_ref": str(PROTOCOL_DOC) +
                 "::Status: author-confirmed — control"}),
            "exactly once")
    control("the status line must record the recorded status",
            lambda s: s["review_protocol"].update(
                {"status_line_ref": _PROTOCOL_NEEDLE}),
            "status line must record")
    control("an independent event requires a pre-registered commitment",
            _event_without_commitment, "pre-registered")
    control("a commitment digest is 64 lowercase hex",
            lambda s: _mk_commitment(s).update(
                {"plant_commitment_sha256": "A" * 64}),
            "lowercase hex")
    control("plant and seed commitments are distinct",
            lambda s: _mk_commitment(s).update(
                {"seed_commitment_sha256": "a" * 64}),
            "distinct")
    control("a commitment binds the protocol text by digest",
            lambda s: _mk_commitment(s).update(
                {"protocol_sha256": "0" * 64}),
            "re-commission")
    control("a commitment date is an ISO date",
            lambda s: _mk_commitment(s).update(
                {"committed_date": "soon"}),
            "YYYY-MM-DD")
    control("a commitment may not postdate its review window",
            _commitment_postdates_window, "precede")
    control("an independent window opens with its date",
            _window_without_date, "opens with")
    control("custody stays outside the repository",
            lambda s: _mk_commitment(s).update(
                {"custody": "kept in-repo for convenience"}),
            "outside the repo")
    control("an independent event runs under the scope-review protocol",
            _event_wrong_protocol, "scope-review protocol")
    control("a commitment carries exactly its declared keys",
            lambda s: _mk_commitment(s).update({"note": "extra"}),
            "unexpected keys")
    control("a calibrated envelope is refused in this contract",
            lambda s: s["envelope"][1].update(
                {"envelope_status": "calibrated"}),
            "future contract amendment")
    control("an established invariant may not depend on an envelope field",
            lambda s: s["envelope"][1]["fields"][0]["dependents"].append(
                "FS-CLM-01"),
            "norm-content")
    control("envelope-relative claims must appear as dependents",
            _drop_required_dependent, "envelope-relative")
    control("a defect's envelope must exist",
            lambda s: s["defects"][0].update({"envelope_id": "FS-ENV-77"}),
            "names no envelope record")
    control("a structure-only envelope cannot carry operational assurance",
            _structure_operationally_assured, "calibrated")
    control("a closure record's envelope must exist",
            _closure_envelope_missing, "names no envelope record")
    control("an envelope field states dependents or invariance",
            lambda s: s["envelope"][1]["fields"][0].update(
                {"dependents": [], "invariance": ""}),
            "dependents or an explicit")
    control("a threshold's lawful source is closed",
            lambda s: s["thresholds"][0].update({"lawful_source": "vibes"}))
    control("a criterion carries its provenance",
            lambda s: s["functional_criteria"]["criteria"][0].pop(
                "provenance"))
    control("a value-bearing key is refused on an envelope field",
            lambda s: s["envelope"][1]["fields"][0].update({"value": "ten"}))
    control("no numeric value in a Book 1 threshold",
            lambda s: s["thresholds"][0].update(
                {"definition": s["thresholds"][0]["definition"] + " 42"}),
            "numeric")
    control("the criteria canon is the seven-member union",
            lambda s: s["functional_criteria"]["criteria"].pop(0),
            "seven-member")
    control("a populated record type sheds its deferral",
            lambda s: s["deferred_populations"].append(
                {"record_type": "roles", "owner_ref": _CONTROL_NEEDLE,
                 "closure_condition": "control", "stage": "stage-3"}),
            "still carries a deferral record")
    control("a role's domain ref must resolve",
            lambda s: s["roles"][0].update({"domain_refs": ["FS-DOM-99"]}),
            "must name domains")
    control("a role may not cite a non-domain as its domain",
            lambda s: s["roles"][0].update(
                {"domain_refs": [s["bodies"][0]["id"]]}),
            "must name domains")
    control("each material domain keeps a reviewed role citation",
            _uncite_domain, "no role cites")
    control("every named scale is exercised",
            _unexercise_scale, "unexercised")
    control("a required body keeps both positions",
            _strip_body_positions, "both an affected and a checking")
    control("an omission carries its risk-based reason",
            _omission_empty_reason, "risk_reason")
    control("an omission names a real role",
            lambda s: s["role_omissions"].append(
                {"role_ref": "FS-ROL-777", "omitted_scale": "individual",
                 "risk_reason": "control"}),
            "unknown role")
    control("a stale omission is refused",
            _stale_omission, "stale omission")
    control("a role's layer is universal standing",
            lambda s: s["roles"][0].update({"layer": "book-2-operation"}),
            "never a floor-changing status")
    control("a role anchor is closed",
            lambda s: s["roles"][0]["formal_anchor"].update(
                {"anchor": "vibes"}),
            "unknown formal anchor")
    control("a constitution-predicate anchor cites the constitution",
            _anchor_without_nibli, "never only prose")
    control("an unchecked private power is refused",
            _power_held_unchecked, "unchecked private power")
    control("a private power's affected side names roles",
            _power_held_bad_affected, "must name roles")
    control("a duplicate role id is caught",
            lambda s: s["roles"].append(copy.deepcopy(s["roles"][0])),
            "duplicate id")
    control("role meanings cannot drift",
            lambda s: s["scale_meanings"].pop("intergenerational"),
            "must define exactly")
    control("role_omissions may not outlive a deferred roles array",
            lambda s: s.update({"roles": []}),
            "while roles is deferred")
    control("a dependency destination must resolve",
            lambda s: s["dependencies"][0].update({"to_ref": "FS-DOM-99"}),
            "destination must name a body, role, or domain")
    control("a dependency endpoint type is closed",
            lambda s: s["dependencies"][0].update(
                {"from_ref": s["claims"][0]["id"]}),
            "source must name a body, role, domain, or external")
    control("an externally-assumed edge flows from a named assumption",
            _dep_exa_without_terminal,
            "must flow from a named external assumption")
    control("an external assumption feeds only externally-assumed edges",
            _dep_exa_on_wrong_class,
            "only an externally-assumed edge may flow")
    control("an edge's layer follows its class",
            _dep_layer_mismatch, "layer follows its dependency class")
    control("each material domain joins the flow map",
            _dep_uncover_domain, "no edge touches")
    control("every flow kind is exercised",
            _dep_unexercise_flow, "unexercised")
    control("every external assumption stays cited",
            _dep_uncite_exa, "uncited")
    control("a cycle needs a declared loop witness",
            _dep_undeclared_cycle, "no declared loop witness")
    control("a declared loop is a real cycle",
            _dep_loop_not_cycle, "must chain into a cycle")
    control("a loop member is a real edge",
            lambda s: s["dependency_loops"][0]["member_edge_refs"
                                               ].__setitem__(0, "FS-DEP-777"),
            "loop member must name a dependency edge")
    control("a refused flow cites its wall",
            lambda s: s["refused_flows"][0].pop("source_ref"),
            "missing keys")
    control("a duplicate edge is one edge",
            _dep_duplicate_edge, "duplicate dependency edge")
    control("a duplicate refusal is caught",
            lambda s: s["refused_flows"].append(dict(s["refused_flows"][0])),
            "duplicate refused flow")
    control("a populated dependencies sheds its deferral",
            lambda s: s["deferred_populations"].append(
                {"record_type": "dependencies", "owner_ref": _CONTROL_NEEDLE,
                 "closure_condition": "control", "stage": "stage-3"}),
            "still carries a deferral record")
    control("loops and walls may not outlive a deferred map",
            lambda s: s.update({"dependencies": []}),
            "while dependencies is deferred")
    control("dependency meanings cannot drift",
            lambda s: s["flow_kind_meanings"].pop("care"),
            "must define exactly")
    control("a lifecycle path is ratified or recorded outside",
            lambda s: s["dependencies"][0].update(
                {"lifecycle_path": "delivery"}),
            "lifecycle_path must be one of")
    control("an absent alternate is recorded, never silent",
            lambda s: s["dependencies"][0].update({"alternate_route": {}}),
            "exactly one of route / no_alternate_reason")
    control("an edge may not feed itself",
            _dep_self_edge, "may not terminate on its own source")
    control("a populated scenarios sheds its deferral",
            lambda s: s["deferred_populations"].append(
                {"record_type": "scenarios", "owner_ref": _CONTROL_NEEDLE,
                 "closure_condition": "control", "stage": "stage-3"}),
            "still carries a deferral record")
    control("omissions may not outlive a deferred catalogue",
            lambda s: s.update({"scenarios": []}),
            "while scenarios is deferred")
    control("a populated catalogue flips every domain's applicability",
            lambda s: s["domains"][0].update({"scenario_applicability": {
                "deferred_ref": _CONTROL_NEEDLE}}),
            "still defers scenario applicability")
    control("each domain keeps a whole-society scenario",
            _scn_unreach_domain, "no scenario reaches")
    control("every scenario kind is exercised",
            _scn_unexercise_kind, "missing kinds")
    control("every collision axis is tested",
            _scn_untest_axis, "untested")
    control("every named shock is carried",
            _scn_uncarry_shock, "uncarried")
    control("every protected-sphere test is exercised",
            _scn_unexercise_form, "unexercised forms")
    control("every critical edge is stressed or recorded omitted",
            _scn_unstress_edge, "unstressed")
    control("a collision axis belongs only on a collision",
            _scn_axis_on_noncollision,
            "belongs only on a collision scenario")
    control("a collision scenario names its axis",
            _scn_collision_without_axis, "names its axis")
    control("a shock kind is closed",
            _scn_bad_shock_kind, "unknown shock_kind")
    control("a scenario's dependency ref must resolve",
            lambda s: s["scenarios"][0].update(
                {"dependency_refs": [s["claims"][0]["id"]]}),
            "dependency_refs must name dependency edges")
    control("a scenario's layer states Book 1 behaviour",
            lambda s: s["scenarios"][0].update({"layer": "book-2-operation"}),
            "capacity and degradation are Book 2's tests")
    control("a scenario's status is the exact inventory literal",
            lambda s: s["scenarios"][0].update({"status": "reviewed-routing"}),
            "reviewed-inventory")
    control("scenario meanings cannot drift",
            lambda s: s["collision_axis_meanings"].pop("property-vs-floor"),
            "must define exactly")
    control("a stale scenario omission is refused",
            _scn_stale_omission, "stale omission")
    control("a bounded witness is a real sibling case",
            lambda s: s["scenarios"][0].update(
                {"bounded_witness_refs": ["record-integrity-red-team#RS-99"]}),
            "names no case in the live sibling witness pool")
    control("a duplicate scenario id is one scenario",
            _scn_duplicate_id, "duplicate id")
    control("an omission needs a risk-based reason",
            lambda s: s["scenario_omissions"][0].update({"risk_reason": ""}),
            "must be a non-empty string")
    control("a book passage may never support a scenario",
            lambda s: s["scenarios"][0].update(
                {"source_refs": ["book-1/rights-floor.md::the floor"]}),
            "narrative-register rule")

    # Positive regression: Gate A closure-envelope validation accepts the
    # non-stub versioned structure without pretending it is calibrated. The
    # full source still refuses gate_a_status=passed until a future author
    # amendment and all independent readiness conditions are met.
    closure_ready = copy.deepcopy(src)
    _mk_closure(closure_ready)
    closure_ready["acceptance_gate"]["gate_a_status"] = "passed"
    validate_closure_record(
        closure_ready,
        ([("control", "met-in-form", "control")],
         [("the reference envelope", "met-in-form", "control")]),
    )

    gate_a_critical = copy.deepcopy(src)
    gate_a_row = next(
        row for row in gate_a_critical["defects"] if row["id"] == "FS-DFT-27"
    )
    gate_a_row["severity"] = (
        "critical — semantic control for a scope-map defect"
    )
    validate(gate_a_critical)
    gate_a_rows, _ = compute_gate_a_readiness(
        gate_a_critical, compute_resolution(gate_a_critical)
    )
    if gate_a_rows[2][1] != "unmet" or \
            "FS-DFT-27" not in gate_a_rows[2][2]:
        raise LedgerError(
            "semantic control failed: a valid Gate-A-applicable critical "
            "defect must make condition three unmet"
        )

    review_only = copy.deepcopy(src)
    before = _gate_a_condition_1_deferred(review_only)
    review_only["deferred_populations"] = [
        row for row in review_only["deferred_populations"]
        if row["record_type"] not in {"proposals", "review_events"}
    ]
    after = _gate_a_condition_1_deferred(review_only)
    if before != after:
        raise LedgerError(
            "semantic control failed: review outputs may not alter condition one"
        )

    complete_reader = render_reader(src, compute_resolution(src))
    validate_reader_projection(src, complete_reader)
    first_population = READER_PROJECTION_POPULATIONS[0]
    reader_marker = _reader_population_line(src, first_population)
    incomplete_reader = complete_reader.replace(reader_marker, "", 1)
    try:
        validate_reader_projection(src, incomplete_reader)
    except LedgerError:
        pass
    else:
        raise LedgerError(
            "semantic control failed: reader projection may not omit a "
            "canonical population"
        )

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
    return passed + 3


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


def _route_unbuilt_on_built(s):
    built = next(r["id"] for r in s["routes"]
                 if r["route_status"] == "built")
    for rec in s["claims"]:
        if rec.get("unestablished_disposition") == "route-unbuilt":
            rec["route_ref"] = built
            return
    raise LedgerError("control setup: no route-unbuilt claim to mutate")


def _evidence_pending_on_unbuilt(s):
    for rec in s["claims"]:
        if rec.get("unestablished_disposition") == "route-unbuilt":
            rec["unestablished_disposition"] = "evidence-pending"
            return
    raise LedgerError("control setup: no route-unbuilt claim to mutate")


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


def _dep_exa_without_terminal(s):
    for rec in s["dependencies"]:
        if rec["dependency_class"] == "externally-assumed":
            rec["from_ref"] = s["bodies"][0]["id"]
            return
    raise LedgerError("control setup: no externally-assumed edge")


def _dep_exa_on_wrong_class(s):
    triples = {(r["from_ref"], r["to_ref"], r["flow_kind"])
               for r in s["dependencies"]}
    for rec in s["dependencies"]:
        if rec["dependency_class"] == "externally-assumed":
            continue
        if ("FS-EXA-01", rec["to_ref"], rec["flow_kind"]) in triples:
            continue
        rec["from_ref"] = "FS-EXA-01"
        return
    raise LedgerError("control setup: no collision-free non-EXA edge")


def _dep_layer_mismatch(s):
    rec = s["dependencies"][0]
    if rec["dependency_class"] == "operationally-supplied":
        rec["layer"] = "constitutional-invariant"
    else:
        rec["layer"] = "book-2-operation"


def _dep_uncover_domain(s):
    kept, dropped = [], set()
    for rec in s["dependencies"]:
        if "FS-DOM-12" in (rec["from_ref"], rec["to_ref"]):
            dropped.add(rec["id"])
        else:
            kept.append(rec)
    if not dropped:
        raise LedgerError("control setup: no edge touches FS-DOM-12")
    s["dependencies"] = kept
    s["dependency_loops"] = [
        loop for loop in s["dependency_loops"]
        if not dropped & set(loop["member_edge_refs"])
    ]
    if not s["dependency_loops"]:
        raise LedgerError("control setup: dropping FS-DOM-12 emptied loops")


def _dep_unexercise_flow(s):
    triples = {(r["from_ref"], r["to_ref"], r["flow_kind"])
               for r in s["dependencies"]}
    for kind in FLOW_KINDS:
        movers = [r for r in s["dependencies"] if r["flow_kind"] == kind]
        if not movers:
            continue
        for target in FLOW_KINDS:
            if target == kind:
                continue
            if any((r["from_ref"], r["to_ref"], target) in triples
                   for r in movers):
                continue
            for r in movers:
                r["flow_kind"] = target
            return
    raise LedgerError("control setup: no collision-free relabel")


def _dep_uncite_exa(s):
    triples = {(r["from_ref"], r["to_ref"], r["flow_kind"])
               for r in s["dependencies"]}
    movers = [r for r in s["dependencies"] if r["from_ref"] == "FS-EXA-01"]
    if not movers:
        raise LedgerError("control setup: no edge sourced at FS-EXA-01")
    for target in ("FS-EXA-02", "FS-EXA-03", "FS-EXA-04"):
        if any((target, r["to_ref"], r["flow_kind"]) in triples
               for r in movers):
            continue
        for r in movers:
            r["from_ref"] = target
        return
    raise LedgerError("control setup: no collision-free EXA retarget")


def _dep_undeclared_cycle(s):
    # Two roles that are no edge's endpoint guarantee the synthetic
    # two-cycle forms its own strongly connected region with no witness;
    # a free body pair could already sit inside a witnessed region.
    used = set()
    for r in s["dependencies"]:
        used.add(r["from_ref"])
        used.add(r["to_ref"])
    free = [r["id"] for r in s["roles"] if r["id"] not in used]
    if len(free) < 2:
        raise LedgerError("control setup: no free endpoint pair")
    pair = (free[0], free[1])
    for i, (f, t) in enumerate([pair, pair[::-1]]):
        twin = copy.deepcopy(s["dependencies"][0])
        twin.update({
            "id": f"FS-DEP-90{i + 1}", "from_ref": f, "to_ref": t,
            "flow_kind": "services",
            "dependency_class": "operationally-supplied",
            "layer": "book-2-operation",
            "lifecycle_path": "outside-ratified-paths",
            "structural_satisfiability": {
                "satisfiability_status": "operation-deferred",
                "defect_refs": [],
                "reason": "synthetic control edge",
            },
        })
        s["dependencies"].append(twin)


def _dep_loop_not_cycle(s):
    edges = {r["id"]: r for r in s["dependencies"]}
    ids = list(edges)
    for a in ids:
        for b in ids:
            if a == b:
                continue
            if (edges[a]["to_ref"] != edges[b]["from_ref"]
                    and edges[b]["to_ref"] != edges[a]["from_ref"]):
                s["dependency_loops"][0]["member_edge_refs"] = [a, b]
                return
    raise LedgerError("control setup: no non-chaining edge pair")


def _dep_duplicate_edge(s):
    twin = copy.deepcopy(s["dependencies"][0])
    twin["id"] = "FS-DEP-903"
    s["dependencies"].append(twin)


def _dep_self_edge(s):
    for rec in s["dependencies"]:
        if rec["dependency_class"] != "externally-assumed":
            rec["to_ref"] = rec["from_ref"]
            return
    raise LedgerError("control setup: every edge is externally assumed")


def _uncite_domain(s):
    # The closure guarantees FS-DOM-12 is cited; strip it everywhere. A role
    # left with no domains gets one so only the closure — not the non-empty
    # rule — decides the verdict.
    touched = False
    for rec in s["roles"]:
        if "FS-DOM-12" in rec["domain_refs"]:
            rec["domain_refs"] = [r for r in rec["domain_refs"]
                                  if r != "FS-DOM-12"]
            touched = True
        if not rec["domain_refs"]:
            rec["domain_refs"] = ["FS-DOM-01"]
    if not touched:
        raise LedgerError("control setup: no role cites FS-DOM-12")


def _unexercise_scale(s):
    touched = False
    for rec in s["roles"]:
        if "intergenerational" in rec["scales"]:
            rec["scales"] = [sc for sc in rec["scales"]
                             if sc != "intergenerational"]
            touched = True
        if not rec["scales"]:
            rec["scales"] = ["individual"]
    if not touched:
        raise LedgerError("control setup: no role exercises intergenerational")


def _strip_body_positions(s):
    bid = s["bodies"][0]["id"]
    touched = False
    for rec in s["roles"]:
        kept = [pp for pp in rec["power_positions"]
                if pp["body_ref"] != bid]
        if len(kept) != len(rec["power_positions"]):
            rec["power_positions"] = kept
            touched = True
    if not touched:
        raise LedgerError(f"control setup: no role positions cite {bid}")


def _omission_empty_reason(s):
    recorded = {(e.get("role_ref"), e.get("omitted_scale"))
                for e in s["role_omissions"] if "omitted_scale" in e}
    for rec in s["roles"]:
        for sc in ROLE_SCALES:
            if sc not in rec["scales"] \
                    and (rec["id"], sc) not in recorded:
                s["role_omissions"].append(
                    {"role_ref": rec["id"], "omitted_scale": sc,
                     "risk_reason": ""})
                return
    raise LedgerError("control setup: no unrecorded omitted scale pair")


def _stale_omission(s):
    rec = s["roles"][0]
    s["role_omissions"].append(
        {"role_ref": rec["id"], "omitted_scale": rec["scales"][0],
         "risk_reason": "control"})


def _anchor_without_nibli(s):
    for rec in s["roles"]:
        if rec["formal_anchor"]["anchor"].startswith("constitution-predicate"):
            rec["formal_anchor"]["refs"] = [
                s["domains"][0]["source_refs"][0]]
            return
    raise LedgerError("control setup: no constitution-predicate role anchor")


def _power_held_unchecked(s):
    for rec in s["roles"]:
        if rec.get("power_held"):
            rec["power_held"]["checking_refs"] = []
            return
    raise LedgerError("control setup: no role holds a power")


def _power_held_bad_affected(s):
    for rec in s["roles"]:
        if rec.get("power_held"):
            rec["power_held"]["affected_role_refs"] = [s["bodies"][0]["id"]]
            return
    raise LedgerError("control setup: no role holds a power")


_CONTROL_NEEDLE = ("new-book-plans/full-society-boundary-decision.md::"
                   "## 4. Versioned closure")
_PROTOCOL_NEEDLE = ("new-book-plans/full-society-scope-review-protocol.md::"
                    "# Full-Society Scope-Review Protocol")


def _strip_deferrals(s, *types):
    s["deferred_populations"] = [
        d for d in s["deferred_populations"] if d["record_type"] not in types
    ]


def _mk_commitment(s):
    s["review_protocol"]["review_commitment"] = {
        "plant_commitment_sha256": "a" * 64,
        "seed_commitment_sha256": "b" * 64,
        "protocol_sha256": hashlib.sha256(
            (ROOT / PROTOCOL_DOC).read_bytes()).hexdigest(),
        "committed_date": "2026-08-09",
        "custody": "control fixture — pre-images author-held outside the "
                   "repository",
    }
    return s["review_protocol"]["review_commitment"]


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
        ev["protocol_ref"] = _PROTOCOL_NEEDLE
        ev["received_window"] = "2026-09-01 to 2026-10-01"
        ev["seeded_control_outcome"] = "passed — control"
        ev["planted_omission_outcome"] = "passed — control"
        _mk_commitment(s)
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


def _event_without_commitment(s):
    _mk_event(s, independent=True)
    s["review_protocol"]["review_commitment"] = None


def _commitment_postdates_window(s):
    _mk_event(s, independent=True)
    s["review_protocol"]["review_commitment"]["committed_date"] = "2027-01-01"


def _window_without_date(s):
    ev = _mk_event(s, independent=True)
    ev["received_window"] = "control"


def _event_wrong_protocol(s):
    ev = _mk_event(s, independent=True)
    ev["protocol_ref"] = _CONTROL_NEEDLE


def _commitment_on_candidate_protocol(s):
    rp = s["review_protocol"]
    rp["protocol_status"] = PROTOCOL_STATUS_CANDIDATE
    rp.pop("confirmation_basis", None)
    _mk_commitment(s)


def _mk_designation(s):
    s["review_protocol"]["designation"] = {
        "severity_owner": "control-owner — control discipline",
        "independent_checker": "control-checker — control discipline",
        "custodian": "control-custodian — pre-image custody",
        "designated_date": "2026-08-09",
        "basis": "control fixture",
    }
    return s["review_protocol"]["designation"]


def _designation_owner_is_checker(s):
    d = _mk_designation(s)
    d["independent_checker"] = d["severity_owner"]


def _designation_owner_is_custodian(s):
    d = _mk_designation(s)
    d["severity_owner"] = d["custodian"]


def _designation_checker_is_custodian(s):
    d = _mk_designation(s)
    d["independent_checker"] = d["custodian"]


def _designation_on_candidate_protocol(s):
    rp = s["review_protocol"]
    rp["protocol_status"] = PROTOCOL_STATUS_CANDIDATE
    rp.pop("confirmation_basis", None)
    _mk_designation(s)


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


def _drop_required_dependent(s):
    for field in s["envelope"][1]["fields"]:
        if "FS-CLM-06" in field["dependents"]:
            field["dependents"].remove("FS-CLM-06")


def _structure_operationally_assured(s):
    row = copy.deepcopy(s["defects"][0])
    row.update({
        "id": "FS-DFT-997", "defect_id": "FS-DFT-997",
        "defect_disposition": "remedied",
        "response_stage": "operationally-assured-in-envelope",
        "envelope_id": "FS-ENV-01", "consequence_id": "control",
        "scope_id": "control", "controls": {},
    })
    s["defects"].append(row)


def _closure_envelope_missing(s):
    _mk_closure(s)["envelope_ref"] = "FS-ENV-77"


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


def _scn_unreach_domain(s):
    # Strip FS-DOM-03 from every scenario, back-filling FS-DOM-01 when a
    # list would empty so only the domain closure — not the non-empty
    # rule — decides the verdict. FS-DOM-03 is not the protected domain,
    # so no protected_sphere_forms XOR side effect fires.
    touched = False
    for rec in s["scenarios"]:
        if "FS-DOM-03" in rec["domain_refs"]:
            rec["domain_refs"] = [r for r in rec["domain_refs"]
                                  if r != "FS-DOM-03"]
            touched = True
            if not rec["domain_refs"]:
                rec["domain_refs"] = ["FS-DOM-01"]
    if not touched:
        raise LedgerError("control setup: no scenario cites FS-DOM-03")


def _scn_unexercise_kind(s):
    # Relabel every stress record to journey — neither kind carries an
    # axis or shock, so no XOR side effect fires.
    touched = False
    for rec in s["scenarios"]:
        if rec["scenario_kind"] == "stress":
            rec["scenario_kind"] = "journey"
            touched = True
    if not touched:
        raise LedgerError("control setup: no stress scenario")


def _scn_untest_axis(s):
    touched = False
    for rec in s["scenarios"]:
        if rec.get("collision_axis") == COLLISION_AXES[0]:
            rec["collision_axis"] = COLLISION_AXES[1]
            touched = True
    if not touched:
        raise LedgerError(
            f"control setup: no collision carries {COLLISION_AXES[0]}"
        )


def _scn_uncarry_shock(s):
    touched = False
    for rec in s["scenarios"]:
        if rec.get("shock_kind") == SHOCK_KINDS[0]:
            rec["shock_kind"] = SHOCK_KINDS[1]
            touched = True
    if not touched:
        raise LedgerError(
            f"control setup: no compound shock carries {SHOCK_KINDS[0]}"
        )


def _scn_unexercise_form(s):
    # Remove one protected-sphere form from every carrier, substituting
    # another when a list would empty so only the form closure decides.
    gone, fill = PROTECTED_SPHERE_FORMS[0], PROTECTED_SPHERE_FORMS[3]
    touched = False
    for rec in s["scenarios"]:
        psf = rec.get("protected_sphere_forms")
        if psf and gone in psf:
            psf.remove(gone)
            touched = True
            if not psf:
                psf.append(fill)
    if not touched:
        raise LedgerError(f"control setup: no scenario exercises {gone}")


def _scn_unstress_edge(s):
    cited = set()
    for rec in s["scenarios"]:
        cited |= set(rec["dependency_refs"])
    target = None
    for dep in s["dependencies"]:
        if dep["severity"] == "critical" and dep["id"] in cited:
            target = dep["id"]
            break
    if target is None:
        raise LedgerError("control setup: no cited critical edge")
    for rec in s["scenarios"]:
        rec["dependency_refs"] = [r for r in rec["dependency_refs"]
                                  if r != target]


def _scn_axis_on_noncollision(s):
    for rec in s["scenarios"]:
        if rec["scenario_kind"] not in ("collision", "compound-shock"):
            rec["collision_axis"] = COLLISION_AXES[0]
            return
    raise LedgerError("control setup: no non-collision scenario")


def _scn_collision_without_axis(s):
    for rec in s["scenarios"]:
        if rec["scenario_kind"] == "collision":
            del rec["collision_axis"]
            return
    raise LedgerError("control setup: no collision scenario")


def _scn_bad_shock_kind(s):
    for rec in s["scenarios"]:
        if rec["scenario_kind"] == "compound-shock":
            rec["shock_kind"] = "asteroid"
            return
    raise LedgerError("control setup: no compound-shock scenario")


def _scn_stale_omission(s):
    for rec in s["scenarios"]:
        if rec["dependency_refs"]:
            s["scenario_omissions"].append(
                {"omitted_dependency_ref": rec["dependency_refs"][0],
                 "risk_reason": "control", "source_ref": _CONTROL_NEEDLE})
            return
    raise LedgerError("control setup: no scenario cites an edge")


def _scn_duplicate_id(s):
    s["scenarios"].append(copy.deepcopy(s["scenarios"][0]))


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
    w("## Roles, life-course stages, scales, and power positions")
    w("")
    w("Each role records the standing of a person in a position — life-course "
      "stages are roles of a kind — and routes it against domains, scales, "
      "and the ratified bodies. A role is never a floor-changing status: one "
      "person occupies many roles and none buys a higher floor or a lower "
      "one, which is why every role's layer is the constitutional invariant "
      "of universal standing; rule content stays on domains and claims. Axis "
      "coverage is mechanical — every domain cited, every named scale "
      "exercised, every required body carrying both an affected and a "
      "checking role position, every recorded private power naming its "
      "affected counter-roles and its checkers — while pairwise sufficiency "
      "stays a question for the independent scope review; no full Cartesian "
      "product is attempted, and deliberately omitted candidates and "
      "combinations are recorded below with risk-based reasons. The FS-POW "
      "decomposition of each power is staged below by exact source-family "
      "prefix and remains deferred until the complete population. "
      "Formal anchors stay honest: a derived constitution predicate, an "
      "asserted predicate with its replace-card path, or "
      "ratified-but-unimplemented doctrine.")
    w("")
    w("| Role | Kind | Domains | Scales | Affected by | Checks | Anchor |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["roles"]:
        affected, checks = [], []
        for pp in rec["power_positions"]:
            (affected if pp["position"] == "affected" else checks).append(
                pp["body_ref"])
        w(f"| {rec['id']} {rec['title']} | {rec['role_kind']} | "
          f"{', '.join(rec['domain_refs'])} | {', '.join(rec['scales'])} | "
          f"{', '.join(affected) or '—'} | {', '.join(checks) or '—'} | "
          f"{rec['formal_anchor']['anchor']} |")
    w("")
    w("Recorded private and delegated powers (the holder's own record names "
      "who stands under the power and who checks it):")
    w("")
    for rec in src["roles"]:
        ph = rec.get("power_held")
        if ph is None:
            continue
        w(f"- `{rec['id']}` holds: {ph['power']} Affected: "
          f"{', '.join(ph['affected_role_refs'])}; checked by: "
          f"{', '.join(ph['checking_refs'])}.")
    w("")
    w("Deliberately omitted candidates and combinations (recorded, not "
      "silent):")
    w("")
    for entry in src["role_omissions"]:
        if "omitted_role" in entry:
            w(f"- No role for {entry['omitted_role']}: "
              f"{entry['risk_reason']}")
        else:
            what = entry.get("omitted_domain_ref") or entry["omitted_scale"]
            w(f"- `{entry['role_ref']}` omits `{what}`: "
              f"{entry['risk_reason']}")
    w("")
    w("## Source-derived power contracts and function allocations")
    w("")
    population = src["power_population"]
    completed = population["completed_source_families"]
    w(f"Population status: **{population['status']}**. Completed source-family "
      f"prefix: {', '.join(completed) if completed else 'none — foundation only'}.")
    w("")
    w(f"Current rows: {len(src['powers'])} FS-POW cards; "
      f"{len(src['power_contract_templates'])} FS-PCT templates; "
      f"{len(src['power_refusals'])} FS-PRF refusals/limits; "
      f"{len(src['power_crosswalk_dispositions'])} FS-PCD formal "
      f"dispositions; {len(src['function_allocations'])} FS-FAL allocations.")
    w("")
    w(f"Evidence ceiling: {population['evidence_ceiling']}")
    w("")
    w("| Power | Manifest grain | Profiles | Holder bodies | Holder roles | "
      "Claims | Book 2 owner |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["powers"]:
        w(f"| {rec['id']} {rec['title']} | `{rec['manifest_key']}` | "
          f"{', '.join(rec['profiles'])} | "
          f"{', '.join(rec['holder_body_refs']) or 'role-held'} | "
          f"{', '.join(rec['holder_role_refs'])} | "
          f"{', '.join(rec['affected_claim_refs'])} | "
          f"`{rec['book2_owner_ref']}` |")
    if not src["powers"]:
        w("| — | no completed source family | — | — | — | — | — |")
    w("")
    w("Contract templates:")
    w("")
    for rec in src["power_contract_templates"]:
        w(f"- `{rec['id']}` / `{rec['manifest_key']}`: {rec['title']}. "
          f"{rec['closure_condition']}")
    if not src["power_contract_templates"]:
        w("- None in the completed prefix.")
    w("")
    w("Refusals and formal transitions:")
    w("")
    for rec in src["power_refusals"]:
        w(f"- `{rec['id']}` / `{rec['manifest_key']}`: {rec['refusal']} "
          f"Non-authorisation: {rec['non_authorisation']}")
    for rec in src["power_crosswalk_dispositions"]:
        w(f"- `{rec['id']}` / `{rec['manifest_key']}`: "
          f"`{rec['crosswalk_action']}` → "
          f"{', '.join(rec['target_power_refs']) or 'no successor'}.")
    if not src["power_refusals"] and not src["power_crosswalk_dispositions"]:
        w("- None in the completed prefix.")
    w("")
    w("Function allocations (body functions; role references identify the "
      "corresponding position classes, not proof of staffing or independence):")
    w("")
    for rec in src["function_allocations"]:
        w(f"- `{rec['id']}` → `{rec['power_ref']}`; writer "
          f"{', '.join(rec['decisive_fact_writer_body_refs'])}; decider "
          f"{', '.join(rec['decider_body_refs'])}; executor "
          f"{', '.join(rec['executor_body_refs'])}; auditor "
          f"{', '.join(rec['auditor_body_refs'])}; final remedy "
          f"{', '.join(rec['final_remedy_body_refs'])}.")
    if not src["function_allocations"]:
        w("- None in the completed prefix.")
    w("")
    w("## Functional flows and cross-domain dependencies")
    w("")
    w("Each edge records that a function depends on a flow — its lawful "
      "source class, its owner, and what breaks when the flow stops. An "
      "edge never records that the flow arrives: no right is called "
      "delivered because an institution promised it, and no body is called "
      "functional because its name exists. The four-way class is routing, "
      "not assurance — constitutionally-guaranteed names the lawful source "
      "of the obligation, never a delivery status, and an "
      "externally-assumed edge names a premise nothing internal "
      "manufactures. The mechanical cycle check establishes exactly one "
      "thing: every strongly connected region of the declared graph "
      "carries at least one declared, classified, owner-named loop witness "
      "with a recorded boundedness statement. Boundedness is reviewed "
      "prose, not a proven property. The closure audit publishes "
      "self-certifying, deadlocking, single-veto, unbounded, bottleneck, "
      "and cascade hazards as bounded-unresolved or scoped blocking; it "
      "admits no rejected-by-control result until a route-bound executable-"
      "control receipt schema lands. Alternate routes are predeclared with "
      "their doctrine "
      "needle or their absence is recorded as a named single point of "
      "failure. Refused flows are walls, not edges: doctrine forbids them, "
      "and drawing one as a dependency would be the defect.")
    w("")
    w("| Edge | Flow | Class | Source → Destination | Path | Alternate | "
      "Owner |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["dependencies"]:
        alt = ("declared" if "route" in rec["alternate_route"]
               else "none — recorded")
        w(f"| {rec['id']} {rec['title']} | {rec['flow_kind']} | "
          f"{rec['dependency_class']} | {rec['from_ref']} → "
          f"{rec['to_ref']} | {rec['lifecycle_path']} | {alt} | "
          f"{rec['steward_ref']} |")
    w("")
    w("Per-edge routing (absence, continuity, remedy, restoration, "
      "correction — routing statements, never delivery):")
    w("")
    for rec in src["dependencies"]:
        w(f"- `{rec['id']}` absence: {rec['consequence']} Continuity: "
          f"{rec['interim_continuity']} Remedy: {rec['remedy_route']} "
          f"Restoration: {rec['restoration']} Correction: "
          f"{rec['systemic_correction']}")
    w("")
    w("Single points of failure (no alternate route, recorded):")
    w("")
    for rec in src["dependencies"]:
        ar = rec["alternate_route"]
        if "no_alternate_reason" in ar:
            w(f"- `{rec['id']}` {rec['title']}: "
              f"{ar['no_alternate_reason']}")
    w("")
    w("Declared loops (classified, bounded, owned):")
    w("")
    for loop in src["dependency_loops"]:
        chain = " → ".join(loop["member_edge_refs"])
        w(f"- `{loop['id']}` {loop['loop_kind']} loop "
          f"(steward {loop['steward_ref']}): "
          f"{chain} — bounded: {loop['boundedness']}")
    w("")
    w("Refused flows (walls, not edges):")
    w("")
    for ent in src["refused_flows"]:
        w(f"- {ent['refused_flow']} [{ent['flow_kind']}]: "
          f"{ent['refusal_reason']}")
    w("")
    w("## Whole-society journeys, collisions, and stress cases")
    w("")
    w("The scenario catalogue is reviewed inventory — a reviewed threat "
      "model, never proof and never a counterexample harness. Each record "
      "routes an owned ordinary, failure, and recovery path: the failure "
      "route carries interim continuity, the recovery route carries remedy "
      "and restoration together, and a route is routing, never delivery. "
      "Nothing here executed — constitutional cases execute only after the "
      "relevant author rulings and contract cards land, and the closure "
      "audit consumes this population. The kinds, collision axes, compound "
      "shocks, and protected-sphere tests are closed vocabularies; every "
      "domain is reached and every critical dependency edge is stressed or "
      "its omission recorded. Shock records state Book 1 invariant and "
      "failure behaviour only — capacity and degradation are Book 2's "
      "tests. Protected-sphere scenarios test freedom without permission, "
      "non-recording and non-compulsion, the narrow evidenced-harm "
      "threshold, and recourse against interference — never a "
      "state-defined successful life outcome. A bounded witness names a "
      "sibling case and establishes only what that artifact's own posture "
      "states.")
    w("")
    w("| Scenario | Kind | Domains | Edges | Steward | Axis / shock |")
    w("| --- | --- | --- | --- | --- | --- |")
    for rec in src["scenarios"]:
        axis = (rec.get("collision_axis") or rec.get("shock_kind") or "—")
        edges_cell = ", ".join(rec["dependency_refs"]) or "—"
        w(f"| {rec['id']} {rec['title']} | {rec['scenario_kind']} | "
          f"{', '.join(rec['domain_refs'])} | {edges_cell} | "
          f"{rec['steward_ref']} | {axis} |")
    w("")
    w("Per-scenario routes (routing statements, never delivery or "
      "execution):")
    w("")
    for rec in src["scenarios"]:
        w(f"- `{rec['id']}` ordinary: {rec['ordinary_route']} Failure: "
          f"{rec['failure_route']} Recovery: {rec['recovery_route']}")
        psf = rec.get("protected_sphere_forms")
        if psf:
            w(f"  - protected-sphere tests: {', '.join(psf)}")
        bw = rec.get("bounded_witness_refs")
        if bw:
            w(f"  - bounded sibling witnesses: {', '.join(bw)}")
    w("")
    w("Deliberately omitted scenario candidates (recorded, not silent):")
    w("")
    for ent in src["scenario_omissions"]:
        label = ent.get("omitted_scenario") \
            or f"`{ent['omitted_dependency_ref']}`"
        w(f"- {label}: {ent['risk_reason']}")
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
    power_inventory = src["power_source_inventory"]
    w("## Public-power source inventory")
    w("")
    w(power_inventory["scope_ceiling"])
    w("")
    w(f"The reviewed manifest `{power_inventory['artifact_ref']}` binds "
      f"{power_inventory['row_count']} source-identified entries: "
      f"{power_inventory['disposition_counts']['card-required']} require "
      "contract cards, "
      f"{power_inventory['disposition_counts']['explicit-refusal-limit']} are "
      "refusals or limits, and "
      f"{power_inventory['disposition_counts']['existing-formal-crosswalk']} "
      "crosswalk narrow current formal fixtures.")
    w("")
    w("Known lawful-allocation gaps that keep FS-POW deferred:")
    w("")
    for gap in power_inventory["known_allocation_gaps"]:
        w(f"- {gap}")
    w("")
    w(f"Closure: {power_inventory['closure_condition']}")
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
      "a veto; "
      + ("the protocol confirmation, "
         if src["review_protocol"]["protocol_status"]
         == PROTOCOL_STATUS_CANDIDATE else "")
      + "the review commissioning and its commitment, "
      + ("the severity owner, "
         if src["review_protocol"].get("designation") is None else "")
      + "and the closure record are author "
      "checkpoints.")
    w("")
    rp = src["review_protocol"]
    line = (f"The scope-review protocol is bound at `{rp['protocol_ref']}`, "
            f"status {rp['protocol_status']}; its status line is live-checked "
            "against this source. ")
    if rp["review_commitment"] is None:
        line += ("No review has been commissioned: the commitment record is "
                 "null, plant and seed pre-images are constructed only at "
                 "commissioning and stay author-held outside the repository, "
                 "and an independent review event without a pre-registered "
                 "commitment is refused.")
    else:
        c = rp["review_commitment"]
        line += (f"A commitment was published on {c['committed_date']}: "
                 f"plant `{c['plant_commitment_sha256']}`, seeds "
                 f"`{c['seed_commitment_sha256']}`, protocol text "
                 f"`{c['protocol_sha256']}`; {c['custody']}.")
    if rp.get("designation") is None:
        line += (" No severity owner or independent checker is designated: "
                 "the designation record is schema-enforced behind its "
                 "absence — real recorded identities, distinct people, "
                 "neither the pre-image custodian — and remains an author "
                 "checkpoint.")
    else:
        d = rp["designation"]
        line += (f" Designated on {d['designated_date']}: severity owner "
                 f"{d['severity_owner']}; independent checker "
                 f"{d['independent_checker']}; custodian {d['custodian']}.")
    w(line)
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
    w("## The reference envelope (structure)")
    w("")
    for rec in src["envelope"]:
        w(f"- **{rec['id']}** ({rec['envelope_status']}): {rec['note']}")
    w("")
    successor = next((r for r in src["envelope"][1:]), None)
    if successor is not None:
        w(f"Version `{successor['envelope_version']}`. No value enters Book 1: "
          "every field's value status names Book 2's Gate D calibration as "
          "owner, and this contract refuses a calibrated envelope outright — "
          "calibration is a deliberate future contract amendment. This "
          "versioned structure satisfies only Gate A's envelope precondition; "
          "operation and remedy still require calibration.")
        w("")
        w("| Field | Definition | Value status | Dependents | Invariance |")
        w("| --- | --- | --- | --- | --- |")
        for field in successor["fields"]:
            deps = ", ".join(field["dependents"]) or "—"
            w(f"| {field['id']} | {field['definition']} | "
              f"{field['value_status']} | {deps} | {field['invariance']} |")
        w("")
    crit = src["functional_criteria"]
    w("## Functional criteria (the meanings of functional)")
    w("")
    w(crit["drift_note"])
    w("")
    w("| Criterion | Definition | Provenance |")
    w("| --- | --- | --- |")
    for rec in crit["criteria"]:
        w(f"| {rec['name']} | {rec['definition']} | "
          f"{'; '.join(rec['provenance'])} |")
    w("")
    w("## Thresholds (meanings, not measurements)")
    w("")
    w("Each threshold binds a ratified sentence by needle and classifies its "
      "lawful source; its layer follows that source, its decision owner is "
      "separated from its measurement owner, and no numeric value appears — "
      "values arrive with their classified lawful source, never here.")
    w("")
    w("| Threshold | Criterion | Domains | Lawful source | Layer | "
      "Definition |")
    w("| --- | --- | --- | --- | --- | --- |")
    for rec in src.get("thresholds", []):
        w(f"| {rec['id']} {rec['title']} | {rec['criterion_ref']} | "
          f"{', '.join(rec['domain_refs'])} | {rec['lawful_source']} | "
          f"{rec['layer']} | {rec['definition']} |")
    w("")
    w("## Book 2 crosswalk (routed rows only)")
    w("")
    w("A collection-only projection: Book 2 remains inactive until Book 1 — "
      "First Edition actually ships, and this view carries routing and closure "
      "fields only. No operating owner, workforce, facility, capacity, service, "
      "or cost field appears here; those belong to Book 2's own responsibility "
      "view when it activates, generated from this same canonical source.")
    w("")
    w("| ID | Title | Routed as | Owner | Severity | Consequence | "
      "Closure condition |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["claims"]:
        if rec["layer"] == "book-2-operation" or \
                rec.get("unestablished_disposition") == "routed-book-2":
            routed = rec["layer"]
            if rec.get("unestablished_disposition"):
                routed += f" ({rec['unestablished_disposition']})"
            w(f"| {rec['id']} | {rec['title']} | {routed} | "
              f"`{rec['owner_ref']}` | {rec['severity']} | "
              f"{rec['consequence']} | {rec['closure_condition']} |")
    for rec in src.get("defects", []):
        if rec.get("book2_crosswalk"):
            w(f"| {rec['id']} | {rec['title']} | {rec['defect_disposition']} | "
              f"`{rec['owner_ref']}` | {severity_class(rec)} | "
              f"{rec['consequence']} | {rec['closure_condition']} |")
    for rec in src["powers"]:
        w(f"| {rec['id']} | {rec['title']} | power operation/evidence handoff | "
          f"`{rec['book2_owner_ref']}` | {rec['severity']} | "
          f"{rec['consequence']} | {rec['closure_condition']} |")
    w("")
    w("## Deferred populations and projections")
    w("")
    w("| Record type | Stage | Owner | Closure condition |")
    w("| --- | --- | --- | --- |")
    for rec in src["deferred_populations"]:
        w(f"| {rec['record_type']} | {rec['stage']} | `{rec['owner_ref']}` | "
          f"{rec['closure_condition']} |")
    w("")
    w("The coverage-map view, the role matrix, the dependency map, the "
      "scenario catalogue, the Book 2 crosswalk, and the assurance allocation "
      "now regenerate from the canonical source. The structural reader ledger "
      "also regenerates from that source; it is navigation only and supplies "
      "no R6 evidence, comprehension result, accessibility validation, reader-"
      "suitability claim, Gate C evidence, or route availability. No one "
      "projection substitutes for another.")
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






def _canonical_digest(value) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _reader_population_line(src: dict, population: str) -> str:
    rows = src[population]
    identities = [
        row["id"] for row in rows
        if isinstance(row, dict) and isinstance(row.get("id"), str)
    ]
    identity_text = ", ".join(identities) if identities else (
        "unkeyed rows; digest is authoritative" if rows else "empty"
    )
    return (
        f"| `{population}` | {len(rows)} | `{_canonical_digest(rows)}` | "
        f"{identity_text} |"
    )


def reader_projection_population_lines(src: dict):
    actual = {
        key for key, value in src.items() if isinstance(value, list)
    }
    declared = set(READER_PROJECTION_POPULATIONS)
    if actual != declared:
        raise LedgerError(
            "reader projection population contract is stale: "
            f"missing {sorted(actual - declared)}, "
            f"extra {sorted(declared - actual)}"
        )
    return [
        _reader_population_line(src, population)
        for population in READER_PROJECTION_POPULATIONS
    ]


def validate_reader_projection(src: dict, rendered: str):
    ceiling = (
        "**STRUCTURAL READER NAVIGATION ONLY.** This projection supplies no R6 "
        "evidence, comprehension result, accessibility validation, reader-"
        "suitability claim, Gate C evidence, or route availability."
    )
    if rendered.count(ceiling) != 1:
        raise LedgerError(
            "reader projection must carry the exact no-evidence ceiling once"
        )
    source_line = (
        f"Canonical source SHA-256: `{_canonical_digest(src)}`. "
        "Every canonical list population is bound below; this digest also "
        "binds the non-list contract fields."
    )
    if rendered.count(source_line) != 1:
        raise LedgerError(
            "reader projection must bind the exact canonical source once"
        )
    for line in reader_projection_population_lines(src):
        if rendered.count(line) != 1:
            raise LedgerError(
                "reader projection population closure is missing or duplicated: "
                + line.split("|")[1].strip()
            )


def render_reader(src: dict, resolution: dict) -> str:
    """Render a structural, reader-oriented projection of the same source."""
    out = []
    w = out.append
    blocked_by = {}
    for row in src["defects"]:
        if resolution[row["id"]]["blocking"]:
            blocked_by.setdefault(
                row["affected_claim_ref"], []
            ).append(row["id"])
    claims_by_id = {row["id"]: row for row in src["claims"]}

    w("<!-- SPDX-License-Identifier: CC-BY-4.0 -->")
    w("<!-- Generated by new-book-plans/13-full-society-ledger.py; "
      "do not edit. -->")
    w("")
    w("# Full-Society Structural Reader Ledger — Generated Projection")
    w("")
    w("**STRUCTURAL READER NAVIGATION ONLY.** This projection supplies no R6 "
      "evidence, comprehension result, accessibility validation, reader-"
      "suitability claim, Gate C evidence, or route availability.")
    w("")
    w(f"Canonical source version: `{src['source_version']}`. "
      f"Gate verdict: **{src['acceptance_gate']['verdict']}**")
    w("")
    w(f"Canonical source SHA-256: `{_canonical_digest(src)}`. "
      "Every canonical list population is bound below; this digest also "
      "binds the non-list contract fields.")
    w("")
    w("## Projection population closure")
    w("")
    w("| Canonical population | Rows | Canonical SHA-256 | Stable identities |")
    w("| --- | ---: | --- | --- |")
    for line in reader_projection_population_lines(src):
        w(line)
    w("")
    w("## Five-layer key")
    w("")
    for layer in SCOPE_DISPOSITIONS:
        w(f"- `{layer}`: {src['scope_disposition_meanings'][layer]}")
    w("")
    w("## Domain navigation")
    w("")
    for domain in src["domains"]:
        domain_id = domain["id"]
        domain_claims = [
            row for row in src["claims"] if domain_id in row["domain_refs"]
        ]
        claim_ids = {row["id"] for row in domain_claims}
        domain_scenarios = [
            row for row in src["scenarios"]
            if domain_id in row["domain_refs"]
        ]
        domain_defects = [
            row for row in src["defects"]
            if row["affected_claim_ref"] in claim_ids
        ]
        w(f"### {domain_id} — {domain['title']}")
        w("")
        w(f"**Reader destination:** {domain['reader_destination']}")
        w("")
        w("Layer dispositions:")
        w("")
        for bucket_key, layer in zip(DOMAIN_BUCKETS, SCOPE_DISPOSITIONS):
            w(f"- `{layer}`: {_bucket_cell(domain[bucket_key])}")
        w("")
        w("Claims:")
        w("")
        for claim in domain_claims:
            disposition = claim["posture"]
            if claim.get("unestablished_disposition"):
                disposition += f" / {claim['unestablished_disposition']}"
            blockers = ", ".join(
                blocked_by.get(claim["id"], [])
            ) or "none"
            w(f"- **{claim['id']} — {claim['title']}**: {claim['claim']} "
              f"Posture: `{disposition}`; route: `{claim['route_ref']}`; "
              f"overlay: `{claim['overlay']}`; blocking defect rows: "
              f"{blockers}. Scope: {claim['scope_bound']} Public limit: "
              f"{claim['public_claim_restriction']}")
        if not domain_claims:
            w("- None.")
        w("")
        domain_powers = [
            row for row in src["powers"] if domain_id in row["domain_refs"]
        ]
        w("Source-derived power cards:")
        w("")
        for power in domain_powers:
            w(f"- `{power['id']}` — {power['title']} "
              f"(`{power['manifest_key']}`); holder bodies: "
              f"{', '.join(power['holder_body_refs']) or 'role-held'}; "
              f"Book 2 owner: `{power['book2_owner_ref']}`.")
        if not domain_powers:
            w("- None in the completed source-family prefix.")
        w("")
        w("Ordinary, failure, and recovery routing:")
        w("")
        for scenario in domain_scenarios:
            w(f"- **{scenario['id']} — {scenario['title']}** "
              f"(`{scenario['scenario_kind']}`): ordinary — "
              f"{scenario['ordinary_route']}; failure — "
              f"{scenario['failure_route']}; recovery — "
              f"{scenario['recovery_route']}")
        if not domain_scenarios:
            w("- None recorded.")
        w("")
        w("Open and bounded defect consequences:")
        w("")
        for defect in domain_defects:
            generated = resolution[defect["id"]]
            w(f"- **{defect['id']} — {defect['title']}**: severity "
              f"{defect['severity']}; consequence: {defect['consequence']}; "
              f"closure: {defect['closure_condition']}; applicable gates: "
              f"{', '.join(defect['applicable_gate_refs'])}; generated "
              f"resolution: `{generated['resolution_status']}`; blocking "
              f"for `{defect['affected_claim_ref']}`: "
              f"`{str(generated['blocking']).lower()}`.")
        if not domain_defects:
            w("- None recorded.")
        w("")
    w("## Bounded repair mappings")
    w("")
    w("These are receipt-to-reader mapping references for eligible repairs. "
      "They do not establish that a reader understood or could access them.")
    w("")
    for receipt in src["receipts"]:
        claim = claims_by_id[receipt["affected_claim_ref"]]
        w(f"- `{receipt['id']}` → `{claim['id']}` "
          f"(`{receipt['reader_mapping_ref']}`); ceiling: "
          f"`{receipt['assurance_ceiling']}`; still does not follow: "
          f"{receipt['still_does_not_follow']}")
    if not src["receipts"]:
        w("- None.")
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
    rendered_reader = render_reader(src, resolution)
    validate_reader_projection(src, rendered_reader)
    spliced_map = validate_coverage_region(src)

    out_path = ROOT / OUTPUT
    reader_out_path = ROOT / READER_OUTPUT
    map_path = ROOT / COVERAGE_MAP
    if args.check:
        current = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
        if current != rendered:
            raise LedgerError(f"{OUTPUT} is STALE — rerun without --check")
        current_reader = reader_out_path.read_text(
            encoding="utf-8"
        ) if reader_out_path.exists() else ""
        if current_reader != rendered_reader:
            raise LedgerError(
                f"{READER_OUTPUT} is STALE — rerun without --check"
            )
        if map_path.read_text(encoding="utf-8") != spliced_map:
            raise LedgerError(
                f"{COVERAGE_MAP} generated region is STALE — rerun without "
                "--check"
            )
        print(f"{OUTPUT}, {READER_OUTPUT}, and the coverage-map region are "
              f"current; {controls} structural negative controls pass; "
              "enum-mapping and "
              "residual-coverage closures over the seven reviewed sources hold; "
              "routing inventory only — nothing established beyond each row's "
              "own posture")
    else:
        out_path.write_text(rendered, encoding="utf-8")
        reader_out_path.write_text(rendered_reader, encoding="utf-8")
        map_path.write_text(spliced_map, encoding="utf-8")
        print(f"wrote {OUTPUT}, {READER_OUTPUT}, and the coverage-map region; "
              f"{controls} structural negative controls pass")


if __name__ == "__main__":
    try:
        main()
    except LedgerError as exc:
        print(f"13-full-society-ledger: {exc}", file=sys.stderr)
        sys.exit(1)
