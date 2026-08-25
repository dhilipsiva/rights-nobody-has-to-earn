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
Stage 4 implements the repository source-derived adversarial audit and keeps
Gate A closure refusable-until-green. Append-only FS-SAU rows bind the
validator-derived semantic scope digest, exact protocol bytes, declared
criteria, checker entry points, command chain, Gate-A-applicable findings,
result, policy basis, and evidence ceiling. Historical rows may retain their
recorded author basis. External commissions, proposals, and terminal events
remain optional evidence with their strict existing admissibility checks; empty
optional populations never block a gate. Per-condition Gate A readiness is
computed and rendered. A closure record is refused while any condition computes
unmet. Gate A passes only through a mechanical closure commit whose semantic
source exactly matches an immutable, fully verified candidate; no human act is
a completion dependency.
Deferred record types carry explicit deferral records with owners.

Usage:
  python3 new-book-plans/13-full-society-ledger.py            # regenerate MD
  python3 new-book-plans/13-full-society-ledger.py --check    # validate + fresh
  python3 new-book-plans/13-full-society-ledger.py \
      --refresh-and-check                                      # atomic refresh
"""

import argparse
import contextlib
import copy
import hashlib
import json
import math
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import types

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

LEGACY_POWER_CONTRACT_KEYS = [
    "lawful_source", "trigger", "evidence_rule", "bounded_effect",
    "public_reasons", "conflict_rule", "non_delegable_limit",
    "independent_review", "appeal", "correction", "remedy",
    "end_condition", "temporal_status", "failure_polarity",
    "required_separation_pairs",
]
POWER_CONTRACT_TERM_KEYS = LEGACY_POWER_CONTRACT_KEYS[:-1]
POWER_TEMPLATE_CONTRACT_KEYS = [
    "current_source", "scope", "review", "renewal_or_end", "challenge",
    "fail_closed_polarity", "frozen_record_limit", "book2_liveness_limit",
]
TERM_BASES = {"source-specified", "current-derived", "bounded-delegation"}
COVERAGE_SOURCE_FAMILY_ORDER = list(POWER_SOURCE_FAMILY_ORDER)
COVERAGE_POPULATION_STATUSES = ["foundation", "partial", "complete"]
COVERAGE_FAMILY_STATES = [
    "planned", "coverage-ready", "formalized", "prose-landed",
]
COVERAGE_DEFERRAL_TYPE = "coverage-contracts"
COVERAGE_EVIDENCE_CEILING = (
    "Coverage contracts and pre-drafting checks only; no operation, delivery, "
    "feasibility, liveness, reader response, external truth, or calibration follows."
)
CONSTITUTIONAL_EFFECT_COUNT = 222
UNIVERSAL_STANDING_STATEMENT_IDS = (
    "b214c5369507e0eb6eb829be92667f5a72b10cff32e966cb6374f4ef4087d8b9",
    "ac2a478457f4f7822d0ee1e62da4e08625aacdaf43c65a4b2b179dbdd784c9cc",
    "127ec74be3d2ec4289977fc4af0144f9ed47e88bde9ca8c3b150f179f8317d28",
    "d2f8035ff865e7e74254cbbd6daaa6314d93419151cf50b019bdf052db07b9e5",
    "ff116c9b7a723dda22c72a1a220f5b340b836ca44e7e4b3ba7d6d43c210fc7f6",
    "249713ae1af1dec1cb17dff9e4c631584d1843826761f1e65454adfcc6388f96",
)
EFFECT_CONTRACT_TERM_KEYS = [
    "lawful_source", "trigger", "evidence_rule", "bounded_effect",
    "forbidden_preconditions", "immediate_access", "record_boundary",
    "challenge_correction", "independent_check", "failure_default",
    "end_or_retention",
]
EFFECT_PROFILE_FIELDS = {
    "standing-status": [
        "holder", "forbidden_preconditions", "immediate_access_route",
        "challenge_correction", "independent_check",
    ],
    "material-floor-access": [
        "holder", "duty_bearer", "minimum", "accessibility", "breach",
        "interim_continuity", "remedy", "appeal", "audit",
    ],
    "fair-process-repair": [
        "trigger", "evidence", "accessible_notice", "advocate", "hearing",
        "appeal", "repair", "end_or_review", "temporal_status",
    ],
    "consequential-record": POWER_PROFILE_FIELDS["consequential-record"],
}
EFFECT_PROFILE_FIELDS.update({
    "source-bound-obligation": [
        "bearer", "bearer_mode", "beneficiary_or_object", "duty_kind",
        "function_or_commitment", "exact_origin", "source", "version",
        "jurisdiction", "legal_scope", "end", "public_principal_retention",
        "express_private_reach", "priority", "excuse", "breach",
        "continuity", "remedy", "non_waiver", "failure_polarity",
    ],
    "finding-action-accountability": [
        "finding_kind", "subject_and_case", "independent_recipient",
        "receipt_evidence", "permitted_actions", "reasons_and_review",
        "positive_nonresponse", "alternate_escalation", "continuity",
        "individual_remedy", "prior_decision_review", "common_cause",
        "responsible_control_correction", "re_audit", "recurrence",
        "individual_relief_non_delay",
    ],
    # These adapters deliberately reuse the power-card field sets.  They expose
    # the same contract vocabulary to constitutional effects without creating a
    # second hand-maintained definition that could drift.
    "liberty-power-limit-adapter":
        POWER_PROFILE_FIELDS["liberty-power-limit"],
    "economic-private-power-limit-adapter":
        POWER_PROFILE_FIELDS["economic-private-power-limit"],
    "class9-common-adapter":
        POWER_PROFILE_FIELDS["commons-future-condition"],
})
EFFECT_POLICY = {
    "universal-standing-root": ("class-01", ["class-07"],
                                 ["standing-status"], ["FS-CLM-38"]),
    "serve-before-reconciliation": ("class-02", ["class-01", "class-04"],
                                     ["material-floor-access", "fair-process-repair"],
                                     ["FS-CLM-39"]),
    "service-enforcement-firewall": ("class-07", ["class-01", "class-02"],
                                     ["consequential-record"], ["FS-CLM-39"]),
    "age-status-continuity": ("class-01", ["class-07", "class-04"],
                              ["standing-status", "consequential-record"],
                              ["FS-CLM-38", "FS-CLM-40"]),
    "identity-link-correction": ("class-07", ["class-01", "class-04"],
                                 ["consequential-record", "fair-process-repair"],
                                 ["FS-CLM-40"]),
    "supported-agency-standing": ("class-01", ["class-04", "class-07"],
                                  ["standing-status", "fair-process-repair"],
                                  ["FS-CLM-38"]),
    "death-status-boundary": ("class-07", ["class-01", "class-04"],
                              ["consequential-record"], ["FS-CLM-40"]),
    "provenance-accountability-retention": (
        "class-07", ["class-01", "class-04"],
        ["consequential-record", "fair-process-repair"], ["FS-CLM-40"]),
}

LIBERTY_ECOLOGY_STATEMENT_IDS = (
    "e8806d1c361e53118c77059fd70b3f50b55f909569a37c0ba89318ee186d887d",
    "3c0d6902dd8159f13e4c194c306da4ef304e13b73e46f1f6e5943ede137f2f03",
    "1899560cc11fc0bebe9b537d56c3fe365f5f3753ee97c3b20c4d18901b2faaac",
    "dcbf1b46c9c998ece28eebb23c0308c640efd0150ef085b12b04bd5ea720c6ed",
    "ae4ad530d6e286fa6f5ac7ac85078a4aeec0204ca683e3f1438f24a82f990726",
    "c58c81261c3a3382d5fddfbe15f7a830528e795cc1e8e1298a5acf252e99465b",
    "c711f18d494ec9234619bcdcff81be30b651780edc9dd15c93e5cc8682893e06",
    "e22df599d7ef00867dd58c878532e67a37416a49bda108edd1c8105354c8a578",
    "a17f74f2915cbf73dbc30185ced6e9ef5ef5134a7acf8b7b2b21b10c75019ce5",
    "281b248e80d5267e646bb1adb65de445bf9906d4161c1f0ebf82d78627537760",
    "3eaa16e0aee9148de09368aaa43ab7fe44cf0d1b18ff385686132e5b72dd10a6",
    "d41a5e1cf93d5d9112a0e3b81065126ce4cc17f06727df30e553558caf9ec4d2",
    "5fcc67708d2421f1b5043d958e327e24d6574fb95ccee5417900a944786e2ad6",
    "18c3ef4c28e7b9007b3fb463c7cdba6a9c40c2c1a60fa2c3a2175dc76e389e0c",
    "1d20381a8fd57fffd1c3e9a48b089d117a8e88c0dc2136d606e6eda268b377e6",
    "59ea8b0b53c4cbb3269a43d778e350354e68111f248defa9af57729833594562",
    "82fac77aa24990426e484304a34225c1881314c8b342478a91f2be629db94d93",
    "9945160424467c1c71a5963d1a009e7b0863942ddbe70cac08aa38ab063bfd7f",
    "e837440e83758b3a842b0193ef8bea4c6155142da093c011c703c8eefee4ecf6",
    "19b7563ff4f72e5a4e61b22a4dc45ce551497bc6e785e0bb039d3660e70dd3ef",
    "2f0e914d8ab1f9fa646629b9772b555e660ba0efe8ab86d857940bebe3bd6d1c",
    "87eb7ccc714f8bce83c703dc52faa31be5186ba812872d49a439237b73f1fbc7",
    "aed363e71c26f1d87e8ca08ddd4274157f8726a2972ef8b1ad5e005786695592",
    "67dcfadf4aa344ad7b9c2190d8efe53047823d54194d835984a8f689da71117d",
    "3b80c16bc175fd59206ac12c2789240219c5425cbb34cb3e9d5072df6fa346bc",
    "4a44120c2a8201750020f49c03ca98cedf24aa7bfad91d03b453696aa90c40f5",
    "7bd1313869000b528ba50a5063e98195f3cfcbb19c6121e71d0eb39d7e7c1729",
    "003d340ad3a8ce35a1e731c689e3a624790fdf656ee872fcada2b8577687ad72",
    "e5124e15be05e0c717689be90e2cf2cdefff3cb16023d03de13269c49b2f4815",
    "5c4535432b17d25a2f561b45858dad7806614ef9cdd90caa3b3ee0b06d908191",
    "45fff99561db38974a68ab71abc3af09d28857871c37cbb95e7871b3914eaf3e",
    "476aa5ca7e2e4fcb3889a3a482a8f54fd34ee25582c58845ca35983b8e09e54a",
    "2e599bcc35c8ebaf97d8f773b06bd12489069fb00a5fb32f19d220313c2fd030",
    "c2543cd256e7a00962f3a42a8194621f9d00cd7199c6c6169d2717291ba72701",
    "276f33d61f64d8ea8526372eda981b2f83ba8aece503c38844e6b3b91c4a1537",
    "ca8ff91470872b41ca22c94c4c32e73de2dd0947525a321e6c76ba46f5fd3d71",
    "dd02be8c23c0f02f2c2df1060c471fa4342f690f9b1d6676cfdbbcb36ee187d3",
    "5234bd6eebc9e87361926046fe35b04e9bc64782a18a28eed1bb3bc560cc7b61",
    "02820bda0cbc8f0fdf1b1bdcf218a88fd0e9ca228bfab8acb8bbaa83c84ff313",
)
EQUALITY_STATEMENT_IDS = (
    "785bdec1dac6890ad0d9abf75d896d0c30935aff75254de8194fc0ab7765c72c",
    "1e8cd72378d2f4eb3fbe18b51b905428102cf085c6b515dbea75f0de6544ea9c",
    "f003dc353d3b29ed7f7dbc6f091f2f34f4c908b12b3b9f30e14294a02a9de818",
    "202373ba4ce9c4fdfe0b4cc9de2cd442f2f2e31011ba61084538cb61c6f3722c",
    "8e8ae8aaee6447d0d9317c9fccc11c532734f0573c4c03d27a0cd8a22e80e9f7",
    "17e1d36abe0636c759bbfcf430f2becb89fdb8420bff8455b238a4f0dbce79b5",
    "884608dd758f6cb991268bcdec22380bc680a32dd97af6081aae59a399c9a611",
    "e62f850f59d5e115f1a476e13fdfd431e5a838064f8c96da191c207393a471ef",
    "55603d936de1a45957058ef47fdbbef4426886f77a88cf6bc50b37fea607e9f8",
    "af65ef4a6778aa49138b41101a1a6835e65948d8771108574c210fdc2f1abb21",
    "9e7d5f63e0f6af43e4a81057cf762f2485d613f3b7bda1b3d2f4fc420420b996",
    "c467464c89b1e50e17acff0c993c38992398e4bfed86f438cf61c03daef31337",
    "e919f4f7ed14f8139a406a39d77c71e1aeeb4503a05b37547bd2cb2a8cf5305c",
    "3a04ecb9237ea09ab6824a0de6b6841862ab0e30312b12eef127df756a547eb2",
    "b8c849194e86ac82da9dead8f95d61e3730df9cce9f82e867fc1d0b48c338c99",
    "81376bef276f0c2845532f20d25420f72b950c4a3a6839feac6c2360f8f2c08b",
    "b1a878aecd175021333ec36b9ce62f7c2aae1213879590b75115316012ba0a5f",
    "a1443514da8922d9c299daafd04c46ada6ef24110363af81df884c899399afca",
    "7d18c550d4d404b086ba502aca0c156910f8c88a3d57a75e1c6db5789d0dbf60",
    "12560623054d6d0629b1244f8b5e44a6a1c95fd57ffaa9aad96c514fa1914293",
    "8bc2374a7e6f7def90a8c9c9e5e32ac4d3dfd778723f5fe37deefdd1d77a91e8",
    "9042b3a1d76ba82fc52095e469cd762d8e6bee3d0853723d484dfbd79b678195",
    "5176f7794546335ca513e19757728a7fdf61f9b74ddfb62123f2e37626ce9529",
    "6e612ef33471db7a5e205915e474f221806ede17ed587c5b5ea25dabce271dad",
    "ea6073e25c5aafe7e6302599f62d0ca9b90012ecf96027d2a0b19ec216aa74bd",
    "618eaf9d9e0ac8e4733c0c1a333d8ec24c3fc97eae5c71492ce84112dc248731",
    "6a2998193851fc56dad54d380735bc5f6adc7e07283e347463e8d1fa1446d933",
    "9a187aecb889d21bce360986b6e3eb878fea361beb3b4cd5a161e6713db6fd26",
    "3b2c40f8eccf7dec9e1909d8260df9f0751ac899342da26df4017cb8453e295c",
    "5a26bd1ffbcddcf86bad8601a850ab8a872e3b6517195f1d696e06a5ed9992ac",
    "416882245f01afd29f67f9d51b794a63fe9148cb4b7e5e370df3c4c810d9a277",
    "92578c4d3d7e0e03c2201564acc5fe7978e98c52b78e566040ec4b56fe263894",
    "d1cadc8edfa8266a8d148fdb1dd007b7aafca032bc27604a8a761ef785adc421",
    "76243300ddd45079cb8960f4ef101dc0d5be51335f0adf3b56cd0c0cbc6d7247",
    "1b16feb27ef2d960096a049ff453f4a6b737e21b53ffb9ba4715fdfe151a4fae",
    "30662c71136d8534623e83da6b0304e447b1b9d5c9061cbea5b14f1a1dd80670",
    "f3188eeaeb88e2adfc067971d910b78d23e5c16262ae522d15b403bef61b42bc",
    "72dd9459072f34c05ee980dec76239e3caf2355cc2c5aa016db8eb15a36103f5",
    "2dea63f8495bba363a1441ee71eecc5a5c98fe7b92454b06f208b26680978cd7",
    "fa12d58d8b8b6da30fffdf5d82e1762fc89042f742c5036f1c2741990bd1eede",
    "57e14a3b6ce5555e863963de0682c419c184ec816f5ce0cca8be1c9c39cd67b9",
    "00720752bc16c55aec549a57a48e505c3f5055de3c6d019a7338d8f0abf36e52",
    "a6e9673b829bfa6e2d5985328bb49e6669274302cbd839485a8e9f6546a92df2",
    "79eaa86280e9069ae70daeb315fef3d3f52a6bfd69819070d08ce3804fd11601",
    "a843992ae640893d171cda7484a0fd9d7b9b178f718b001c247f35d417637109",
    "6fe12c1bc0d0ba03316cace645eba543a99cdc8e263f0aa9836bb3b9e2db53b2",
    "213a603242de02bb37af704298b344b986736e8941527cdf82675298118cf43f",
    "a87f6cc7920d941766e4546cf288ce0326de046f866bb258904a97e172a4ac0f",
    "188f7adb2dd576cdb8f48cfb8f3a3b8f7986c387d8e5632b4056227d24cfdbc9",
    "c4d69a121246f7173c172590d1c85ab786c49fa3d58ea44606bb51be6e47a771",
    "f216a7da904f738721c44413204d38b5799eb5acfe2f3c77d345995a21c71de9",
    "f5335bbe370872ebca3c0311896fd113cd6faabecb82fa2ee55f85ee49e35e7a",
    "5a06084bddd736f543fd903f3c2683abfcc82b51c602d800e3e8aaa8c45c22fd",
    "a73809db9653992a22efe63774c3bd25d9340f7db6f10d60094d9f77656d05bc",
)
EFFECT_PROFILE_FIELDS.update({
 "material-floor-inventory":["inventory_boundary","recipient_evidence","continuity","breach_remedy","ecology_non_substitution"],
 "liberty-power-limit":["holder","duty_bearer","protected_choice","lawful_limit","notice_reasons","challenge_review","remedy","failure_default"],
 "environmental-right":["holder","duty_bearer","information","assessment","participation","reasons","review_interim_protection","correction_remedy","equality_route"],
 "commons-future-condition":["protected_condition","evidence_writer","lawful_enactment","version_temporal_status","precaution","non_regression","avoid_minimize_restore","nonfungibility","future_capability","failure_transition"],
 "equality-status":["holder","open_grounds","forms","anti_subordination","no_score","failure_default"],
 "equality-reach":["public_reach","delegated_reach","named_private_reach","protected_association","public_continuity","remedy_route"],
 "equality-proof":["claimant_showing","respondent_burden","evidence","necessity","least_discriminatory_means","core_boundary","review"],
 "equality-access":["proactive_modes","individual_accommodation","institution_wide_burden","equivalent_route","continuity","remedy"],
 "equality-diagnostics":["purpose_limit","aggregation","privacy","independent_governance","record_firewall","no_individual_inference","challenge"],
 "equality-positive-measure":["objective","evidence","eligibility","political_boundary","temporal_source","alternate_review","silence_default","end"],
 "equality-repair":["historical_barrier","provenance","initiation","function_separation","individual_remedy","systemic_remedy","non_repetition","non_administration"],
 "equality-federal":["common_baseline","local_variation","portability","equalisation","cross_boundary_duty","dispute_forum","review"],
 "equality-distinction":["existing_distinction","classification","justification_test","narrowing","review","failure_default"],
})
EFFECT_POLICY.update({
 "material-floor-inventory":("class-02",["class-01","class-04"],["material-floor-inventory"],["FS-CLM-04","FS-CLM-05"]),
 "dual-floor-ceiling-continuity":("class-09",["class-02","class-04"],["material-floor-inventory","commons-future-condition"],["FS-CLM-04","FS-CLM-33"]),
 "privacy-power-limit":("class-01",["class-04","class-07"],["liberty-power-limit"],["FS-CLM-08"]),
 "bodily-autonomy-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-08"]),
 "movement-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-08","FS-CLM-25"]),
 "due-process-power-limit":("class-04",["class-01","class-07"],["liberty-power-limit","fair-process-repair"],["FS-CLM-08"]),
 "association-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-08"]),
 "expression-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-07","FS-CLM-08"]),
 "conscience-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-07","FS-CLM-08"]),
 "family-intimate-power-limit":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-08","FS-CLM-09"]),
 "equal-civic-status-limit":("class-03",["class-01","class-04"],["liberty-power-limit"],["FS-CLM-02","FS-CLM-08"]),
 "residual-private-civic-freedom":("class-01",["class-04"],["liberty-power-limit"],["FS-CLM-08","FS-CLM-35"]),
 "environmental-conditions-right":("class-01",["class-09","class-04"],["environmental-right"],["FS-CLM-33"]),
 "environmental-information-right":("class-07",["class-01","class-09"],["environmental-right"],["FS-CLM-33"]),
 "environmental-assessment-participation":("class-04",["class-01","class-09"],["environmental-right"],["FS-CLM-33"]),
 "environmental-reasons-review-remedy":("class-04",["class-01","class-09"],["environmental-right","fair-process-repair"],["FS-CLM-33"]),
 "environmental-public-private-reach":("class-06",["class-01","class-09"],["environmental-right"],["FS-CLM-33"]),
 "environmental-equality-no-score":("class-03",["class-01","class-09"],["environmental-right"],["FS-CLM-02","FS-CLM-33"]),
 "class9-commons-future-capability":("class-09",["class-04"],["commons-future-condition"],["FS-CLM-33"]),
 "class9-multiaxis-ceilings":("class-09",["class-07"],["commons-future-condition"],["FS-CLM-33"]),
 "class9-science-law-versioning":("class-09",["class-04","class-07"],["commons-future-condition"],["FS-CLM-33"]),
 "class9-precaution-nonregression":("class-09",["class-04"],["commons-future-condition"],["FS-CLM-33"]),
 "class9-restoration-nonfungibility":("class-09",["class-04"],["commons-future-condition"],["FS-CLM-33"]),
 "class9-initiation-interim-route":("class-04",["class-09"],["commons-future-condition","fair-process-repair"],["FS-CLM-33"]),
 "class9-liability-public-restoration":("class-04",["class-09"],["commons-future-condition"],["FS-CLM-33"]),
})
FAMILY_LIFE_STATEMENT_IDS = (
    "e8f3f9a55910078c217a576873a31959a6e07ae1b3decef810a72be220cc62c2",
    "5129eb8a60a3f3ef9a128a574c98d0471cf385bf43e78d5b2acc031f6bc87b72",
    "88baf3078c9fdfa78eaf47e2d40cc52fd5902127d603a9347e092c9e204596ab",
    "5a592f54c1934583209adade7e5c47c84977c9ef9e93cb5387f0864c5e76bb80",
    "2cda01b0a75cd82fece42f47c3dfbcb4a51e52052304bf8e8525ff18d13a10bd",
    "6222ad0e9dc0cc62205dcb0d2d767cb305433c7a99e5ee74ef1ac64272b64e21",
    "ee582628ff0e120197a68fb83372ac3d7e15927029edb758e9c5e5436adbaf72",
    "755aac1943c7e272735e5a020e481c1e55bd7a3768ffc701b923afd01d787c6d",
    "0eafff34bac9e74358fce1c7358ce386c0900e01178aa8cba94bdbafb2697745",
    "57d1d4da7cb147b95649643132bcf0f906f722d937e9d29f4d74297121b63a7b",
    "343b94128ba0173b9bdaabdf9a644e2feb07d8422619a07405a6a0118165d90f",
    "14650a506928b0b16ec5ea9bcbb4685754e02e7a6a618bd560b2c6bbc3c9517e",
    "8c83fb6c3bb811111965ab64a1c653ce40e152e3191aa4e8228ee6123ae78830",
    "4c04c9759d241cedca39f1c427be85a4ea66deb572bed609240a0fd750a1675f",
    "55e873fdcb71734240f25df22fccbdd03854e5952078d42f16265fde8c5c5a7b",
    "8b23b441f9573b03b6d56dd61088e62a8d7752d70a3637453f07770c75a61ea1",
    "f80baa6092daa9cb3554a9aceec870e0e4565bd0a397ecc3f36030da56296822",
    "952dedef4813b77bea3c70514925c37abf9e9f86c1175f321cf5c3d3314f4934",
    "1b5fd23cc81527fe82a3faeb98a5454916a9cdf7847ef833f7d674f76d4ed14e",
    "7f3eb374791107bfc9765d507dd5f7f08d792764787973188804d7025f43e126",
    "fb67870aa7ce7953515be2ae3cfee4a9585d8fffafe4615539a71e13348e99e1",
    "be4e46480e11620d7a3d958d9901cebbdf341a30680a55588a1e1d47651c3581",
    "c5060cbd8610281e2baab6c68759d2cb8f1572dd010cecdddc47b027fc123bc2",
    "2e55bd161fa4fcc699f994a0a264808c5c4e9fa420f46111b4708a01a76d14e7",
    "435e1552319b7365a9a652ea824ccd264888614d854c9fba382604997a6ad5c8",
    "91202dc2d73e1d9c4bc04c35da2b21d0740e74441db40f02e649bfcc6d05d0f1",
    "5ea2d46f0f5eeb7e776f830768d835214fef3a7bbed35373ce3c710a4c389723",
    "3ad6fa672b13e1ae21683a3677f7e04270d8e33276a2c0fd1414856c303d2eae",
    "1944a262e89c9d46cec531c5ef6d25ae5a525d26e7be6635760a27ef6581cf4a",
    "d0707fb82447b30aaec3917d8b65254f284ebfda955ba1f2077e29b7bc75fe6c",
    "0c3fa65fc20a46607b12fee64e9acd95d26182c7fc85d57da89d2f3cd53ae989",
    "b2018d3e2234110ee1a296500892157396e2d252e178f7c7a0cfd5b66745eb55",
    "fe8adc61b1371180becfffc35ea94a0983e7b02d0a774b23856b14d0b57d9f78",
    "deb5f0553b68a973e3bb95ee37d9bb32054c89cf36cb5768bddc5a57c86c5aac",
    "a4119d9144253acf689e5b0e0235c9338a0b30d8ebf8af7aff0d22543ea4c242",
    "921b398dcda9165dbd4405dc7f8bcb4d471dc4ee354ae6c7ba8412e3f2169ed4",
    "4d72c2e9fb9c30352625b3f31705dc58d0f2f41975acdd3e196792f6f3adf457",
    "1499f5f7ffffb760ef564f49974a9bb0c3bd452e67213a97cd52e267435b67c6",
    "f5eadbde5d458e261c50c4fbfec169425b52e80af6d27b6e60953e9502488839",
    "043903199b01d8b4dc5a7112653bce1c95afee85f0c6afa1c529a1f35a611f4c",
    "35c3436674ce9ce147da083dc3c94848a349801910136a7044c0a2b45da3a0c8",
    "7bb6fb8f582fee934c78d8b824847d9a5837dcc5c7aef0d441bce02175ca8137",
    "cda3c9641c0cb6801e11e0df801d4d44d898557ee822d46711772ffdc7751585",
    "f65a0700d95980aaf37230427a95f89610b092ed7987743e24abd0f74dc220c7",
    "e8e73e6adb3626239048ebcdb3dfd71c3e6ae43f5e3812262693d6c6cca1d0e3",
    "e5bf20230976d88ca43b890ecf56129f0ad621d79f7299396de824e2c5dd59cf",
    "0d81686ed7d6a34b0ab880b8a06ac5a366fe93bce6c6292dd6c787b87e866f14",
    "ac73f77e7d94d75597cf7f4823eaf895dda33e774e80aff3a8e07e9595f3a1ca",
    "368f70cf31b7bfbc163afd85d0479532d8c19bfc598f8913ab7f8bf4cd15f5c4",
    "8afda15302416a57d998174c206e8900ab450e551c894f15bfa3f9221cea8876",
    "12dbb80d4eb1df8d9df5f642fea834e035956d8e6ff2ad5b50af6bdbd82e7492",
    "882205333c7683ed797f0e4c7a578314b99a3747e0dc933450ddb00d66f5253d",
    "c78a3751e1e4660059b3b8deede0eadeb70850b347d89a26ee20b0775ae417ef",
    "a417716811ebf6ce88a3af2351c067d72df01d31640ce08e29f7b954e3e92bf4",
    "dd8ad3e16fd0f77ed4addd33701476a9fd3e1e25423551803b5b13f11a3993ff",
    "2103dc700db77c440729f0f07203971e06d1475636f52324f0ee982deb80b191",
    "382a370839189f30b519fbd5635957c26cd3218d2370de7bb5845930a0ce4573",
    "7697279e76071345623383d3104f44ad1922ee8d557170246ca641b3c35272af",
    "cd0f7c79776568307fce92951913e0f1e179bba8422ba354527111aeb1b2843e",
    "ba15a93a34b93b6a6da6bd26b59cdc7e546a33830dd095fe2e6bac51dd6d1ed1",
    "69063539c6b34e6581fdd3e7ef5640e4d0269ee51e6c47de2aec0903ae48f0f7",
    "829209c29f2f675aed8e18c41b9d0bbdceb55525b0d72c3a627a0695c6584d78",
    "176d95b9d17ab8162b069416ccac3f1359ec2f4d52d2bc71b5616ad9c3c09ca2",
    "f8364272969dbab65076fc81d50479499cc8096d1195b293db04c9c429596a5b",
    "1b78021815f1d01c4e3548f24121344f2ff5f5c5002a6bf032c3344747750191",
    "77414f305f4136c3a3e1e8b3e4568d7a7bbbc424514291f1bcdb1c8e7e81c2dc",
    "76f8213025e48ff0c41f3d73830ba55f9785c06292306357a7b38c04b455bcb8",
    "95bdd88597392a3397c63edb544adbc42cd2f027edd4312fb9cfdaf5d6796146",
    "29c63ba8f82704541d1923e65ec99cb8008feddbdbc6392b8131c6578a0358a1",
    "92e2afbf87e10332d2ab22baac6e54b86bde6852362fe73577e643518755288c",
    "de6a38f4bd5507edfec702b5a4a8a750cd08d5b7ec4c0aa781769279156c3357",
    "3057a0bcf4950e47e45052f15471ff5952968ef0e136d9e0c180e30dc3081dae",
    "bfe0ec45878e798f4a039515aac93e6720e1e6b5b7751bca1f30f8f9fe20a7f0",
    "3fe175a6e045360d3fc038dc00c5a5e3e6973a49d91f4e9ef8bf0dc42d2f00ae",
    "2b22d90165787f41773900ac3f2cacb02f9d7423ab6440037b5c84f247f1d74b",
    "76e1aad0278f64ec852dd1869e78817cf701249fdc0b58c99cb6b13d2cbef564",
    "a71b10ddf7247ae9f0f6861bb7be33642c81317ebd9b48c08d9660b57616e13c",
    "009bfa4015e6b9210388b1f8e2b0900e22a5a4dc1cda0dbb0c70a059b4a21c76",
    "a5a2885106a8694107be7fce7dc000652ef549f9defe22d5963ddf1703e9d88b",
    "3746c436b7a19ca62f03739bdf2d89d5841b0d0ebc11089ae2ded53623df4a59",
    "7cea2f1e26be2c5ac60bdae406af83b48d215b0fe9b58044f8a6dbd67e581f6b",
    "f66c4cbf332dbffac01bdce52bd1e564b4cd625ce8aea485ea5b06440922be17",
    "44aefb3c5782ca27b89e9873b20d7b547640558d61cff7428286b811cb579424",
    "18950bea0c53b32e2838c160e6299f12a0f1b54bf9a67c9c542fe6de924e12a9",
    "96b637666f5a5023960f0ecfd7b2cb3d34dd250f28596ff45a6bdcc1459e0a7a",
    "5a7b8c9bb254e2b41940afca6a8d2207dcb4a64b78c236352e2bff4d25f17c79",
    "901acd613093def9264ba16180d0283e1ac266abcec586e4c87f2234ba35198b",
    "4c4f8bffab48c1c19f3246e7c1beb827bb03212be79d05d2e1417d169fe8aa94",
    "83081144c80b98525f9fb27edc8963f08fed1ba18f8356c2390d8d1accf29d79",
    "e398042e5cf92c3ce33e90224e1f723a1b820bf438cf111e13cf6fd29e7fda16",
    "7ed255fdb4ae5e735675fa2050af6e30e183bb090756f8b076cd213fe1f5a9b5",
    "db982287a643bdafa8e22c08001c8a0395c4acb36a28ea30350db2affc88766a",
    "53ced65571062765d4d8be277e42b861de47328f7421d79be7008ab92d5a92ec",
    "0f32e588feb70f998148c4fc2a6eb962af4dadddfa5576e8f725307b2b80ad50",
    "2b79043eae1f5007ea3c9b7b9c652d567639358674fe0ac2a01ce24a4cc37c8f",
    "4e7fd6b92a1eaa0d1a758d42571512fbf66878c62687dd0ca87160ab0007ec28",
    "cb5b401e0f5dd441402ae4aa85019cf4550fba8e31bac5dd62fb0aaaa23e8de4",
    "8384b87729d80eb2e0847f5ea3f77586481dc6b402929dc119fd873a79a5a3e4",
    "f2d5a719063df809f883e96b4887741d34cee6401fd0db049d6f9b148e7cafbf",
    "926b91ae27ccd3cca3a69178c5cd2be5b43b2d5c0afd8c0e92543ce5d11a0b18",
    "f6ee09f80e686d53fcc3208b200a8b304d2b21ee8adb17739300719c989c9654",
    "dddf12ed0bc5bb51df37a9ea3a2210a808a0d3c58905649d37c3ffca7a614b9e",
    "5c22d2103e6c4985286f7cfb6a1842d6b069dcbedab2e31e517be901cdf5dd1e",
    "7b8fc77f744a8e70f3861a93242fdb7b61fac6f761efcc5f4634cb3d663ce0d5",
    "292c8a62459c28c41f85694da794949bdcb2e504e858ef3b9a2bcbab815a7820",
    "e013739134e4f56f523f345acc55eaf1062375ce206acecbfb71a4ba66fc0eb9",
    "43d732f59dccc99956f90768c786e8360b4ad576315c8961f997ae492fdd21ee",
    "ddc69629f233c47226a2fdaabf3fddd9f87422774ee091867c7697dac54775f8",
    "6acabcad51f49ac4c4830ba4f87e9045016e734cd2e9a0c4fabe4713e2f0703e",
    "0957a81913a5ad8f196ee49228605791725d49c73c051def22fd561131378a40",
    "6667d4327b986d21b2f8052d78c0423ee91b9ade2d2a71795e1d09706fd7e242",
    "da52b4660572a128c251627b162dcaf189e8a7d8ee561c2182a194e95ac097cb",
    "4792050f74237b00bc2efe91b707648fe94cf992b207c4ad28601071f84e16e4",
    "91045ceb687dcb8b83a51c5e2fdbbd616fd28a7e48b6faf4109db6ec70b2d433",
    "bacd9a85252ea6f62bcbe231f98298820691388bdb74b8854a1500abce1ad712",
    "441650fa4e2c402f764ed204257184e145d5a610ec04776273b1b0d6584f9246",
    "fca936ecef43e293b4b12cdc153eabf69ed4f0bbb0c5496148fc157961eac7d4",
    "aecff9973397e90085e1d46e99453afd68e589fb871a101b29d677038756b4a4",
    "2fce6a2dec1e71f648f8431174ec7e9e967d5143fc59988f3a8d802970e39b55",
    "a257377f0815f731af633b8b14e0d65160a9e58ae775ecc06242905e1f48acd3",
    "20d669ea6e281e90347c9e73101c99323d6b0f237053f3b36179b678941c626d",
    "467b5efac21df8fbc13279dd00df7da232fb5ca90acbd28766093b005bed3f7d",
    "89f8e1d545993e90e2d4974ce9a4346b7ce54dfed11e4cee541cd65394898e4d",
    "aae1b4a188a7a7b89a719e3d47a4360bd2650ee7a8f687927d9632b0b4c5b3b1",
)
EFFECT_PROFILE_FIELDS.update({'child-rights': ['holder',
                  'information',
                  'voice',
                  'best_interests',
                  'representation',
                  'remedy',
                  'failure_default'],
 'adulthood-status': ['common_tier',
                      'acquisition',
                      'evidence',
                      'alternative_evidence',
                      'continuity',
                      'early_authority',
                      'failure_default'],
 'family-module': ['module',
                   'entry',
                   'disclosure',
                   'exit',
                   'private_boundary',
                   'public_control',
                   'failure_default'],
 'parentage-origin': ['source',
                      'responsibility',
                      'plurality',
                      'origin_record',
                      'child_voice',
                      'continuity',
                      'failure_default'],
 'care-dependency': ['holder',
                     'public_duty',
                     'private_duty',
                     'continuity',
                     'relinquishment',
                     'recognition_boundary',
                     'remedy'],
 'domestic-protection': ['evidence_trigger',
                         'immediate_protection',
                         'targeting',
                         'separation',
                         'review',
                         'reunification',
                         'failure_default'],
 'supported-agency': ['legal_capacity',
                      'chosen_support',
                      'communication',
                      'best_interpretation',
                      'review',
                      'revocation',
                      'failure_default'],
 'reproductive-autonomy': ['holder',
                           'consent',
                           'confidentiality',
                           'third_party_boundary',
                           'pregnancy_authority',
                           'assisted_reproduction',
                           'continuity'],
 'ageing-end-life': ['holder',
                     'community_living',
                     'current_decision',
                     'prior_direction',
                     'treatment_refusal',
                     'independent_review',
                     'failure_default'],
 'succession': ['source',
                'beneficiary',
                'intestacy',
                'plurality',
                'public_status_boundary',
                'floor_continuity',
                'conflict_review'],
 'family-remedy': ['trigger',
                   'standing',
                   'interim_protection',
                   'correction',
                   'compensation',
                   'non_repetition',
                   'no_operation']})

EQUALITY_EFFECT_POLICY = {
 "substantive-equality-status":("class-01",["class-03"],["equality-status"],["FS-CLM-02"]),
 "anti-subordination-baseline":("class-01",["class-03","class-04"],["equality-status"],["FS-CLM-02"]),
 "open-protected-grounds":("class-01",["class-07"],["equality-status"],["FS-CLM-02"]),
 "direct-discrimination":("class-01",["class-04"],["equality-status","equality-proof"],["FS-CLM-02"]),
 "indirect-discrimination":("class-01",["class-04"],["equality-status","equality-proof"],["FS-CLM-02"]),
 "systemic-discrimination":("class-01",["class-04","class-07"],["equality-status","equality-diagnostics"],["FS-CLM-02"]),
 "multiple-discrimination":("class-01",["class-04"],["equality-status"],["FS-CLM-02"]),
 "intersectional-discrimination":("class-01",["class-04"],["equality-status"],["FS-CLM-02"]),
 "associative-discrimination":("class-01",["class-03"],["equality-status"],["FS-CLM-02"]),
 "accommodation-denial":("class-04",["class-01","class-02"],["equality-access","equality-proof"],["FS-CLM-02","FS-CLM-05"]),
 "segregation-exclusion":("class-03",["class-01","class-02"],["equality-status","equality-reach"],["FS-CLM-02","FS-CLM-08"]),
 "equality-harassment":("class-03",["class-01","class-04"],["equality-status","equality-proof"],["FS-CLM-02","FS-CLM-08"]),
 "equality-retaliation":("class-04",["class-01","class-03"],["equality-status","equality-repair"],["FS-CLM-02","FS-CLM-08"]),
 "public-delegated-equality-reach":("class-06",["class-01","class-04"],["equality-reach"],["FS-CLM-02"]),
 "named-private-equality-reach":("class-03",["class-01","class-06"],["equality-reach"],["FS-CLM-02","FS-CLM-08"]),
 "public-equality-protection-duty":("class-06",["class-01","class-04"],["equality-reach","equality-repair"],["FS-CLM-02"]),
 "voluntary-association-boundary":("class-03",["class-01"],["equality-reach"],["FS-CLM-02","FS-CLM-08"]),
 "expressive-role-boundary":("class-03",["class-01","class-04"],["equality-reach","equality-proof"],["FS-CLM-02","FS-CLM-08"]),
 "direct-distinction-test":("class-04",["class-01"],["equality-proof"],["FS-CLM-02"]),
 "indirect-distinction-test":("class-04",["class-01"],["equality-proof"],["FS-CLM-02"]),
 "equality-core-nonbalancing":("class-01",["class-02","class-03","class-04","class-05"],["equality-proof"],["FS-CLM-02","FS-CLM-05","FS-CLM-08","FS-CLM-26"]),
 "remedial-burden-shift":("class-04",["class-01","class-07"],["equality-proof"],["FS-CLM-02"]),
 "respondent-record-evidence":("class-07",["class-04"],["equality-proof","equality-diagnostics"],["FS-CLM-02"]),
 "proactive-accessibility":("class-02",["class-01","class-06"],["equality-access"],["FS-CLM-02","FS-CLM-05"]),
 "individual-accommodation":("class-04",["class-01","class-02"],["equality-access"],["FS-CLM-02","FS-CLM-05"]),
 "institution-wide-undue-burden":("class-04",["class-01","class-02","class-06"],["equality-access","equality-proof"],["FS-CLM-02","FS-CLM-05"]),
 "equivalent-public-route":("class-06",["class-01","class-02"],["equality-access","equality-reach"],["FS-CLM-02","FS-CLM-05"]),
 "equality-diagnostics-data-wall":("class-07",["class-01","class-04"],["equality-diagnostics"],["FS-CLM-02"]),
 "aggregate-pattern-presumption":("class-07",["class-04"],["equality-diagnostics","equality-proof"],["FS-CLM-02"]),
 "diagnostic-participation-noninference":("class-07",["class-01"],["equality-diagnostics"],["FS-CLM-02"]),
 "positive-measure-trigger":("class-01",["class-04","class-05"],["equality-positive-measure"],["FS-CLM-02"]),
 "positive-measure-contract":("class-05",["class-01","class-04","class-07"],["equality-positive-measure"],["FS-CLM-02","FS-CLM-26"]),
 "political-equality-measure-boundary":("class-05",["class-01"],["equality-positive-measure"],["FS-CLM-02","FS-CLM-26"]),
 "positive-measure-eligibility-firewall":("class-07",["class-01","class-05"],["equality-positive-measure","equality-diagnostics"],["FS-CLM-02"]),
 "positive-measure-temporal-end":("class-04",["class-01","class-05","class-07"],["equality-positive-measure"],["FS-CLM-02"]),
 "historical-barrier-repair":("class-04",["class-01","class-05"],["equality-repair"],["FS-CLM-02"]),
 "historical-record-disposition":("class-07",["class-01","class-04"],["equality-repair","equality-diagnostics"],["FS-CLM-02"]),
 "distributed-equality-initiation":("class-04",["class-01","class-06"],["equality-repair"],["FS-CLM-02"]),
 "equality-function-separation":("class-06",["class-04","class-07"],["equality-repair"],["FS-CLM-02"]),
 "individual-equality-remedies":("class-04",["class-01"],["equality-repair"],["FS-CLM-02"]),
 "systemic-equality-remedy":("class-04",["class-01","class-06"],["equality-repair"],["FS-CLM-02"]),
 "court-nonadministration":("class-04",["class-06"],["equality-repair"],["FS-CLM-02"]),
 "federal-equality-portability":("class-08",["class-01","class-06"],["equality-federal"],["FS-CLM-02","FS-CLM-26"]),
 "cross-boundary-equality-corridor":("class-08",["class-01","class-04","class-05"],["equality-federal"],["FS-CLM-02","FS-CLM-26"]),
 "family-distinction-narrowing":("class-04",["class-01","class-03"],["equality-distinction"],["FS-CLM-02"]),
 "home-distinction-narrowing":("class-04",["class-01","class-03"],["equality-distinction"],["FS-CLM-02"]),
 "maturity-distinction-narrowing":("class-04",["class-01","class-07"],["equality-distinction"],["FS-CLM-02"]),
 "conviction-distinction-narrowing":("class-04",["class-01","class-07"],["equality-distinction"],["FS-CLM-02"]),
 "custody-distinction-narrowing":("class-04",["class-01","class-03"],["equality-distinction"],["FS-CLM-02"]),
 "official-status-distinction-narrowing":("class-04",["class-01","class-06"],["equality-distinction"],["FS-CLM-02"]),
 "placement-distinction-narrowing":("class-04",["class-01","class-08"],["equality-distinction"],["FS-CLM-02"]),
 "voiding-distinction-narrowing":("class-04",["class-01","class-07"],["equality-distinction"],["FS-CLM-02"]),
 "recognition-nonoperation-retained":("class-07",["class-01"],["equality-distinction"],["FS-CLM-02"]),
 "qualification-distinction-narrowing":("class-04",["class-01","class-06"],["equality-distinction"],["FS-CLM-02"]),
}
EFFECT_POLICY.update(EQUALITY_EFFECT_POLICY)
FAMILY_LIFE_EFFECT_POLICY = {'child-independent-rights': ('class-01', ['class-02', 'class-04'], ['child-rights'], ['FS-CLM-09']),
 'child-accessible-information-help': ('class-02', ['class-01', 'class-04'], ['child-rights'], ['FS-CLM-09']),
 'child-voice-no-age-floor': ('class-01', ['class-04'], ['child-rights'], ['FS-CLM-09']),
 'child-view-weight-reasons': ('class-04', ['class-01', 'class-07'], ['child-rights'], ['FS-CLM-09']),
 'child-best-interests-record': ('class-04', ['class-01', 'class-07'], ['child-rights'], ['FS-CLM-09']),
 'child-individual-assessment': ('class-04', ['class-01'], ['child-rights'], ['FS-CLM-09']),
 'child-fiduciary-authority-limit': ('class-06', ['class-01', 'class-04'], ['child-rights'], ['FS-CLM-09']),
 'uniform-general-adulthood': ('class-01', ['class-05', 'class-08'], ['adulthood-status'], ['FS-CLM-10']),
 'automatic-adulthood-acquisition': ('class-01',
                                     ['class-04', 'class-07'],
                                     ['adulthood-status'],
                                     ['FS-CLM-10']),
 'adulthood-record-evidence-only': ('class-07',
                                    ['class-01'],
                                    ['adulthood-status'],
                                    ['FS-CLM-10', 'FS-CLM-19']),
 'adulthood-alternative-evidence': ('class-04',
                                    ['class-01', 'class-07'],
                                    ['adulthood-status'],
                                    ['FS-CLM-10']),
 'established-adult-continuity': ('class-01', ['class-04', 'class-07'], ['adulthood-status'], ['FS-CLM-10']),
 'adulthood-nonretrogression': ('class-01', ['class-04', 'class-07'], ['adulthood-status'], ['FS-CLM-10']),
 'provisional-adult-expiring-opportunity': ('class-05',
                                            ['class-01', 'class-04'],
                                            ['adulthood-status'],
                                            ['FS-CLM-10']),
 'early-authority-decision-specific': ('class-04',
                                       ['class-01'],
                                       ['adulthood-status'],
                                       ['FS-CLM-09', 'FS-CLM-10']),
 'early-authority-political-boundary': ('class-05',
                                        ['class-01', 'class-04'],
                                        ['adulthood-status'],
                                        ['FS-CLM-10']),
 'family-form-equal-respect': ('class-01', ['class-03'], ['family-module'], ['FS-CLM-09']),
 'family-module-separation': ('class-07',
                              ['class-01', 'class-03'],
                              ['family-module'],
                              ['FS-CLM-09', 'FS-CLM-19']),
 'family-module-entry-consent': ('class-03', ['class-01', 'class-04'], ['family-module'], ['FS-CLM-09']),
 'family-obligation-disclosure': ('class-07', ['class-03', 'class-04'], ['family-module'], ['FS-CLM-09']),
 'family-exit-winding-up': ('class-04', ['class-01', 'class-03'], ['family-module'], ['FS-CLM-09']),
 'family-private-waiver-limit': ('class-01',
                                 ['class-02', 'class-03', 'class-04'],
                                 ['family-module'],
                                 ['FS-CLM-09']),
 'marriage-no-constitutional-priority': ('class-03',
                                         ['class-01', 'class-04'],
                                         ['family-module'],
                                         ['FS-CLM-09']),
 'civil-family-public-control': ('class-06', ['class-03', 'class-04'], ['family-module'], ['FS-CLM-09']),
 'parentage-responsibility': ('class-06', ['class-01', 'class-04'], ['parentage-origin'], ['FS-CLM-09']),
 'parentage-source-plurality': ('class-07', ['class-01'], ['parentage-origin'], ['FS-CLM-09']),
 'plural-parentage': ('class-01', ['class-04', 'class-07'], ['parentage-origin'], ['FS-CLM-09']),
 'parentage-duty-allocation-continuity': ('class-02',
                                          ['class-01', 'class-04'],
                                          ['parentage-origin'],
                                          ['FS-CLM-09']),
 'adoption-consent-voice': ('class-04', ['class-01', 'class-07'], ['parentage-origin'], ['FS-CLM-09']),
 'parentage-anti-sale-status-gap': ('class-03',
                                    ['class-01', 'class-02'],
                                    ['parentage-origin'],
                                    ['FS-CLM-09']),
 'origin-record-integrity-access': ('class-07',
                                    ['class-01', 'class-04'],
                                    ['parentage-origin'],
                                    ['FS-CLM-09', 'FS-CLM-19']),
 'origin-access-no-contact-effect': ('class-07',
                                     ['class-01', 'class-03'],
                                     ['parentage-origin'],
                                     ['FS-CLM-09', 'FS-CLM-19']),
 'custody-parentage-continuity': ('class-01', ['class-04', 'class-08'], ['parentage-origin'], ['FS-CLM-09']),
 'public-first-care-continuity': ('class-02',
                                  ['class-01', 'class-06'],
                                  ['care-dependency'],
                                  ['FS-CLM-05', 'FS-CLM-09']),
 'assumed-care-duty-bounds': ('class-03', ['class-02', 'class-04'], ['care-dependency'], ['FS-CLM-09']),
 'kinship-no-personal-care-debt': ('class-03', ['class-01', 'class-02'], ['care-dependency'], ['FS-CLM-09']),
 'dependant-independent-rights': ('class-01', ['class-02', 'class-04'], ['care-dependency'], ['FS-CLM-09']),
 'caregiver-independent-rights': ('class-01', ['class-02', 'class-04'], ['care-dependency'], ['FS-CLM-09']),
 'caregiver-safe-relinquishment': ('class-04', ['class-01', 'class-02'], ['care-dependency'], ['FS-CLM-09']),
 'care-work-actual-function': ('class-07', ['class-02', 'class-03'], ['care-dependency'], ['FS-CLM-09']),
 'care-status-nondelivery': ('class-07', ['class-02'], ['care-dependency'], ['FS-CLM-09']),
 'household-nonprescription': ('class-01', ['class-03'], ['domestic-protection'], ['FS-CLM-09']),
 'status-no-harm-presumption': ('class-04', ['class-01', 'class-03'], ['domestic-protection'], ['FS-CLM-09']),
 'domestic-protection-evidence-trigger': ('class-04',
                                          ['class-01', 'class-07'],
                                          ['domestic-protection'],
                                          ['FS-CLM-09']),
 'confidential-help-no-offence-proof': ('class-02',
                                        ['class-01', 'class-04'],
                                        ['domestic-protection'],
                                        ['FS-CLM-09']),
 'targeted-protective-action': ('class-04', ['class-01', 'class-07'], ['domestic-protection'], ['FS-CLM-09']),
 'domestic-emergency-temporal-limit': ('class-04',
                                       ['class-01', 'class-07'],
                                       ['domestic-protection'],
                                       ['FS-CLM-09']),
 'child-separation-last-resort': ('class-04',
                                  ['class-01', 'class-02'],
                                  ['domestic-protection'],
                                  ['FS-CLM-09']),
 'support-before-removal': ('class-02', ['class-01', 'class-04'], ['domestic-protection'], ['FS-CLM-09']),
 'separation-review-reunification': ('class-04',
                                     ['class-01', 'class-07'],
                                     ['domestic-protection'],
                                     ['FS-CLM-09']),
 'child-activity-labour-safety': ('class-03',
                                  ['class-01', 'class-02'],
                                  ['domestic-protection'],
                                  ['FS-CLM-09']),
 'adult-legal-capacity': ('class-01', ['class-04'], ['supported-agency'], ['FS-CLM-09']),
 'plenary-guardianship-refusal': ('class-01', ['class-04', 'class-06'], ['supported-agency'], ['FS-CLM-09']),
 'chosen-support-control': ('class-01', ['class-03', 'class-04'], ['supported-agency'], ['FS-CLM-09']),
 'support-not-substitution': ('class-04', ['class-01', 'class-03'], ['supported-agency'], ['FS-CLM-09']),
 'support-refusal-continuity': ('class-02', ['class-01', 'class-04'], ['supported-agency'], ['FS-CLM-09']),
 'bounded-best-interpretation': ('class-04', ['class-01', 'class-07'], ['supported-agency'], ['FS-CLM-09']),
 'emergency-agency-noncarry': ('class-04', ['class-01', 'class-07'], ['supported-agency'], ['FS-CLM-09']),
 'agency-record-scope-revocation': ('class-07',
                                    ['class-01', 'class-04'],
                                    ['supported-agency'],
                                    ['FS-CLM-09', 'FS-CLM-19']),
 'reproductive-service-access': ('class-02',
                                 ['class-01', 'class-03'],
                                 ['reproductive-autonomy'],
                                 ['FS-CLM-09']),
 'reproductive-free-informed-choice': ('class-01',
                                       ['class-03', 'class-04'],
                                       ['reproductive-autonomy'],
                                       ['FS-CLM-09']),
 'reproductive-coercion-prohibition': ('class-01', ['class-04'], ['reproductive-autonomy'], ['FS-CLM-09']),
 'third-party-reproductive-veto-refusal': ('class-01',
                                           ['class-03', 'class-06'],
                                           ['reproductive-autonomy'],
                                           ['FS-CLM-09']),
 'pregnancy-continuation-termination-authority': ('class-01',
                                                  ['class-04'],
                                                  ['reproductive-autonomy'],
                                                  ['FS-CLM-09']),
 'fetal-override-refusal': ('class-01', ['class-04'], ['reproductive-autonomy'], ['FS-CLM-09']),
 'pregnancy-outcome-nonpunishment': ('class-04',
                                     ['class-01', 'class-07'],
                                     ['reproductive-autonomy'],
                                     ['FS-CLM-09']),
 'confidential-child-health-route': ('class-02',
                                     ['class-01', 'class-04'],
                                     ['reproductive-autonomy'],
                                     ['FS-CLM-09']),
 'conscientious-refusal-continuity': ('class-02',
                                      ['class-01', 'class-06'],
                                      ['reproductive-autonomy'],
                                      ['FS-CLM-09']),
 'assisted-reproduction-consent': ('class-04',
                                   ['class-01', 'class-07'],
                                   ['reproductive-autonomy'],
                                   ['FS-CLM-09']),
 'gestational-body-control-refusal': ('class-01',
                                      ['class-03', 'class-04'],
                                      ['reproductive-autonomy'],
                                      ['FS-CLM-09']),
 'gestational-anti-exploitation': ('class-03',
                                   ['class-01', 'class-04'],
                                   ['reproductive-autonomy'],
                                   ['FS-CLM-09']),
 'parentage-at-birth-continuity': ('class-02',
                                   ['class-01', 'class-07'],
                                   ['reproductive-autonomy'],
                                   ['FS-CLM-09']),
 'ageing-rights-continuity': ('class-01', ['class-02', 'class-04'], ['ageing-end-life'], ['FS-CLM-09']),
 'community-living-support': ('class-02',
                              ['class-01', 'class-04'],
                              ['ageing-end-life'],
                              ['FS-CLM-05', 'FS-CLM-09']),
 'placement-consent-narrow-authority': ('class-04',
                                        ['class-01', 'class-07'],
                                        ['ageing-end-life'],
                                        ['FS-CLM-09']),
 'current-treatment-decision': ('class-01', ['class-04'], ['ageing-end-life'], ['FS-CLM-08', 'FS-CLM-09']),
 'treatment-refusal-life-sustaining': ('class-01',
                                       ['class-04'],
                                       ['ageing-end-life'],
                                       ['FS-CLM-08', 'FS-CLM-09']),
 'treatment-authority-order': ('class-04', ['class-01', 'class-07'], ['ageing-end-life'], ['FS-CLM-09']),
 'treatment-dispute-review-continuity': ('class-04',
                                         ['class-01', 'class-02'],
                                         ['ageing-end-life'],
                                         ['FS-CLM-09']),
 'assisted-dying-democratic-boundary': ('class-05',
                                        ['class-01', 'class-04'],
                                        ['ageing-end-life'],
                                        ['FS-CLM-09']),
 'assisted-dying-safeguards': ('class-04',
                               ['class-01', 'class-05', 'class-07'],
                               ['ageing-end-life'],
                               ['FS-CLM-09']),
 'assisted-dying-representative-refusal': ('class-01', ['class-04'], ['ageing-end-life'], ['FS-CLM-09']),
 'assisted-dying-fresh-temporal': ('class-07',
                                   ['class-01', 'class-04'],
                                   ['ageing-end-life'],
                                   ['FS-CLM-09', 'FS-CLM-19']),
 'death-external-finding': ('class-07',
                            ['class-01', 'class-04'],
                            ['ageing-end-life'],
                            ['FS-CLM-09', 'FS-CLM-19']),
 'remains-directions-boundary': ('class-07',
                                 ['class-01', 'class-03'],
                                 ['ageing-end-life'],
                                 ['FS-CLM-09', 'FS-CLM-19']),
 'will-led-succession': ('class-03', ['class-01', 'class-04'], ['succession'], ['FS-CLM-09', 'FS-CLM-23']),
 'noninheritability-public-status': ('class-05',
                                     ['class-01', 'class-03', 'class-06'],
                                     ['succession'],
                                     ['FS-CLM-09', 'FS-CLM-23']),
 'functional-intestacy': ('class-03', ['class-01', 'class-04'], ['succession'], ['FS-CLM-09', 'FS-CLM-23']),
 'plural-child-succession-equality': ('class-01',
                                      ['class-03', 'class-04'],
                                      ['succession'],
                                      ['FS-CLM-09', 'FS-CLM-23']),
 'collective-title-succession-limit': ('class-09',
                                       ['class-03', 'class-08'],
                                       ['succession'],
                                       ['FS-CLM-09', 'FS-CLM-23']),
 'succession-floor-housing-continuity': ('class-02',
                                         ['class-03', 'class-04'],
                                         ['succession'],
                                         ['FS-CLM-05', 'FS-CLM-09']),
 'beneficiary-care-conflict': ('class-04',
                               ['class-03', 'class-06'],
                               ['succession'],
                               ['FS-CLM-09', 'FS-CLM-23']),
 'family-common-minimum': ('class-08', ['class-01', 'class-04'], ['family-remedy'], ['FS-CLM-09']),
 'family-function-separation': ('class-06', ['class-04', 'class-07'], ['family-remedy'], ['FS-CLM-09']),
 'family-remedy-set': ('class-04', ['class-01', 'class-02'], ['family-remedy'], ['FS-CLM-09']),
 'family-remedy-nonharm': ('class-04',
                           ['class-01', 'class-02', 'class-07'],
                           ['family-remedy'],
                           ['FS-CLM-09']),
 'missing-kinship-no-independence': ('class-07',
                                     ['class-04', 'class-06'],
                                     ['family-remedy'],
                                     ['FS-CLM-09', 'FS-CLM-19']),
 'family-status-no-confinement': ('class-04', ['class-01', 'class-08'], ['family-remedy'], ['FS-CLM-09']),
 'legacy-status-nonproof': ('class-07',
                            ['class-01', 'class-04'],
                            ['family-remedy'],
                            ['FS-CLM-09', 'FS-CLM-10']),
 'home-concept-separation': ('class-07',
                             ['class-01', 'class-03'],
                             ['family-remedy'],
                             ['FS-CLM-09', 'FS-CLM-19']),
 't3-life-course-nonborrowing': ('class-07', ['class-04'], ['family-remedy'], ['FS-CLM-09', 'FS-CLM-19']),
 'book2-life-course-boundary': ('class-10',
                                ['class-01', 'class-02', 'class-04'],
                                ['family-remedy'],
                                ['FS-CLM-09'])}
EFFECT_POLICY.update(FAMILY_LIFE_EFFECT_POLICY)
# Delivery and receipt: one direct effect per routed floor item plus the three
# structural guards. `believe` and `expresses` are deliberately unrouted and
# carry no effect here — a refusal has no counterfactual, so the reasons live in
# the ratified decision rather than in a card that could not be formalized.
DELIVERY_EFFECT_POLICY = {
 'food-delivery-receipt': ('class-02', ['class-07'], ['material-floor-access'], ['FS-CLM-05']),
 'shelter-delivery-receipt': ('class-02', ['class-07'], ['material-floor-access'], ['FS-CLM-05']),
 'care-delivery-receipt': ('class-02', ['class-07'], ['material-floor-access'], ['FS-CLM-05']),
 'security-delivery-receipt': ('class-02', ['class-07'], ['material-floor-access'], ['FS-CLM-05']),
 'company-delivery-receipt': ('class-02', ['class-07'], ['material-floor-access'], ['FS-CLM-05']),
 'provider-may-not-certify-receipt': ('class-07', ['class-02'], ['material-floor-access'], ['FS-CLM-05']),
 'authorised-independent-delivery-writer': ('class-07', ['class-06'], ['material-floor-access'], ['FS-CLM-05']),
 'arrival-not-roster-conditioned': ('class-02', ['class-01'], ['material-floor-access'], ['FS-CLM-05']),
}
EFFECT_POLICY.update(DELIVERY_EFFECT_POLICY)
DELIVERY_STATEMENT_IDS = (
    "1895923b6ccf7fa49418d3b44a7eeaceeaa287db2371586052782d74bc337d7c",
    "ba4be9a5c3e7dc0ff6a5abef5813871b2494511fca4c841ef381e218538952a9",
    "7ed9f9790f518441114bf12d10f649ad0c1d3d00fc122da404301c49b6535726",
    "ac57535a9a230c5ecb0d1a0862e61a8eb1a4e5ce71801d5e4e907972517e9e72",
    "2bdefdb9e029bc4ad762ea2508072c8914adfdaed92eeaf818fd6b400a9e95b4",
    "fa4ba5b070f5187db305b21ee2759a4efdcd186874df846808670c1dfc2827a3",
)
OBLIGATION_EFFECT_POLICY = {
    "public-respect-duty": (
        "class-01", ["class-04", "class-06"],
        ["source-bound-obligation", "liberty-power-limit-adapter"],
        ["FS-CLM-08"],
    ),
    "public-protect-duty": (
        "class-01", ["class-04", "class-06"],
        ["source-bound-obligation", "liberty-power-limit-adapter",
         "economic-private-power-limit-adapter"],
        ["FS-CLM-08"],
    ),
    "public-fulfil-duty": (
        "class-02", ["class-01", "class-06"],
        ["source-bound-obligation", "material-floor-access"],
        ["FS-CLM-04", "FS-CLM-05"],
    ),
    "public-continuity-duty": (
        "class-02", ["class-04", "class-06"],
        ["source-bound-obligation", "material-floor-access"],
        ["FS-CLM-05"],
    ),
    "public-remedy-duty": (
        "class-04", ["class-02", "class-06", "class-07"],
        ["source-bound-obligation", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-05", "FS-CLM-12", "FS-CLM-21"],
    ),
    "delegation-without-discharge": (
        "class-06", ["class-04", "class-07"],
        ["source-bound-obligation"], ["FS-CLM-18"],
    ),
    "express-private-duty-prerequisite": (
        "class-06", ["class-03", "class-07"],
        ["source-bound-obligation",
         "economic-private-power-limit-adapter"],
        ["FS-CLM-02", "FS-CLM-23"],
    ),
    "no-subject-matter-private-duty": (
        "class-01", ["class-03", "class-07"],
        ["source-bound-obligation", "liberty-power-limit-adapter",
         "economic-private-power-limit-adapter"],
        ["FS-CLM-08", "FS-CLM-35"],
    ),
    "person-duty-continuity-restoration": (
        "class-04", ["class-01", "class-02"],
        ["source-bound-obligation", "standing-status",
         "fair-process-repair"],
        ["FS-CLM-08", "FS-CLM-12"],
    ),
    "class9-common-cessation-restoration": (
        "class-09", ["class-04", "class-07"],
        ["source-bound-obligation", "class9-common-adapter",
         "fair-process-repair"],
        ["FS-CLM-33"],
    ),
    "role-duty-correction-reassignment": (
        "class-06", ["class-04", "class-07"],
        ["source-bound-obligation", "fair-process-repair"],
        ["FS-CLM-12", "FS-CLM-18"],
    ),
    "voluntary-duty-bounded-cure": (
        "class-03", ["class-01", "class-04"],
        ["source-bound-obligation", "liberty-power-limit-adapter",
         "fair-process-repair"],
        ["FS-CLM-23", "FS-CLM-35"],
    ),
    "source-specific-conflict-priority": (
        "class-04", ["class-02", "class-06", "class-07"],
        ["source-bound-obligation", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-12", "FS-CLM-18", "FS-CLM-21"],
    ),
    "bounded-performance-excuse": (
        "class-04", ["class-02", "class-06", "class-07"],
        ["source-bound-obligation", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-05", "FS-CLM-18", "FS-CLM-21"],
    ),
    "all-entitlement-nonreciprocity": (
        "class-01", ["class-02", "class-04", "class-05", "class-07"],
        ["source-bound-obligation", "standing-status",
         "material-floor-access", "fair-process-repair"],
        ["FS-CLM-01", "FS-CLM-02", "FS-CLM-04", "FS-CLM-05",
         "FS-CLM-07", "FS-CLM-08", "FS-CLM-12", "FS-CLM-14",
         "FS-CLM-25", "FS-CLM-35", "FS-CLM-38", "FS-CLM-39",
         "FS-CLM-40"],
    ),
    "finding-reader-action-duty": (
        "class-07", ["class-04", "class-06"],
        ["source-bound-obligation", "finding-action-accountability",
         "consequential-record"],
        ["FS-CLM-12", "FS-CLM-18", "FS-CLM-21"],
    ),
    "certified-positive-nonresponse": (
        "class-07", ["class-04", "class-06"],
        ["finding-action-accountability", "consequential-record",
         "fair-process-repair"],
        ["FS-CLM-12", "FS-CLM-18", "FS-CLM-21"],
    ),
    "finding-alternate-escalation": (
        "class-04", ["class-06", "class-07"],
        ["finding-action-accountability", "consequential-record",
         "fair-process-repair"],
        ["FS-CLM-12", "FS-CLM-18", "FS-CLM-21"],
    ),
    "finding-claimant-continuity": (
        "class-02", ["class-04", "class-06"],
        ["finding-action-accountability", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-05", "FS-CLM-12", "FS-CLM-18"],
    ),
    "individual-remedy-prior-decision-review": (
        "class-04", ["class-02", "class-07"],
        ["finding-action-accountability", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-05", "FS-CLM-12", "FS-CLM-21"],
    ),
    "finding-common-cause-investigation": (
        "class-07", ["class-04", "class-06"],
        ["finding-action-accountability", "consequential-record"],
        ["FS-CLM-18", "FS-CLM-21"],
    ),
    "responsible-control-correction": (
        "class-06", ["class-04", "class-07"],
        ["finding-action-accountability", "consequential-record"],
        ["FS-CLM-18", "FS-CLM-21"],
    ),
    "affected-case-reaudit": (
        "class-04", ["class-06", "class-07"],
        ["finding-action-accountability", "consequential-record",
         "fair-process-repair"],
        ["FS-CLM-12", "FS-CLM-21"],
    ),
    "recurrence-verification": (
        "class-07", ["class-04", "class-06"],
        ["finding-action-accountability", "consequential-record"],
        ["FS-CLM-18", "FS-CLM-21"],
    ),
    "systemic-work-no-individual-delay": (
        "class-04", ["class-02", "class-07"],
        ["finding-action-accountability", "material-floor-access",
         "fair-process-repair"],
        ["FS-CLM-05", "FS-CLM-12"],
    ),
}
EFFECT_POLICY.update(OBLIGATION_EFFECT_POLICY)
OBLIGATION_STATEMENT_IDS = (
    "f0d0ce8655703693fde3767766dc4a39833ae09a28af931021aa11232df973d9",
    "448d3158b1db6ea918562b83ec63b94a5ed24ab134d84f3dc0b8b4085ec16b0f",
    "9983a6c7539476584a4f04b76a88442f0d3a8d5d28b8bda06664232f7c74ae53",
    "829039c29ed95cf8bd8cea469132590a406460739bff6218e45b97b5f0661d29",
    "18389b45c29446c47c97ff5135e0cc88971767883d09b8ea0bfc56982076db5b",
    "e8799cbf3c1ca66cd8043936067964b295d65c79f4ad2e2d06741fdc1915b816",
    "73572016106e12be7f8918b31e8913ef2f7788a2c59c62428ad50206afccecd9",
    "d878d467c4d6585dfa12822b1c7494464ca6c68fb7f6e43036efe88dfaa60a97",
    "74ef4636ed8ad19ee8457dfe4903339c0a3901168baca230f82782ec6f7edd13",
    "034c0ac812d042ebe82bc81b6cc653314a20b1d0056bb64a88ce1fb195d5676f",
    "dc730383e97e128b43a51b145923360b27ee6146561bb245ee4a069ace6a918e",
    "99e417fe3a815330a7fdc8cd2c1693b11d26711edd10a8c5f586d0b4a733c1c5",
    "5125c5a6f2da30e7cb31de558d6a94e19c213323a5d42ca7d8e6fe371a05e7e3",
    "3e4a7ba64c86bb91f816bb609d2e09c11a3d00e9b4dd363e5a25c47d34f644d0",
    "fbc43f870605a2032f535973de5717fb53f1ad79e6affd5431260e8e390d1b78",
    "18710ae5f6ff5fdbd6d0e4dd1a79f8001b2dceb5103cea3fdc11473035bac443",
    "cce5947e058ed5606bc26cbd35be81b8a2ab6280e9210f83b2d1b1294def292b",
    "1026f634c4e7ae87842c12c80b5081a5f0830702b07d15155b228cafd65fa6ac",
    "664809f0e06730a9f2fb09b0ddaa6b18f86ccfbe880459bcfc857fa602e330ed",
    "dd48912f5352c5d47adfaf385c754d5a30c2a9da2a38b88a4eac917a437e194c",
    "6a53c03370820d0ef24f2f16d81278e25585d5a93990dd97b938aadfe84c61a9",
    "656bdcd3434fdcaf5df40db8e64adc8da3bc5e4939c731cce7c2a97b4d98659e",
    "ac6d68c1d82c8f7ede4b6971ca74119e5b39e50760caaaec5ac98b0f338deadb",
    "94a616ea27220e892a1646377b0fbae100e8d56ab15044b77ef166dacb020458",
    "240451537a59da8d5b5d16a504907f297a255ca33ab9c1a63af342aa2434ffd6",
    "aa75c1d9bd05e0c193eb2306c9fb797cfbb9b2d9dc51c3e1e2dbab910114aac1",
    "55413ee0c157549516306909d5d52906949e84b52c158bd7835a2f9d8e53f08c",
    "295bd2f86bf25231760183e9424b5044b3dda0b0757d7d990ff2349b3c1aa3d1",
    "9fe16e65f3050f82bd5826c821aeb0a3df7a4a3ff33fd5e023d70855993af138",
    "39e9aad756b8fdcb420de4b4a9d91af329d567f5a1b35161e85c89a7cc1a59db",
    "dfe6c82de9ba40eb7c1760d93f5ca4bf4ca3333053962f2437ec60a012e35482",
    "4426d96f3bbadb1321f69b960b9c93740a0c6ef9882c80846e35816c9c1ac380",
    "9dc34ae2b615f42dc02369257bb33cfcbcbb4999d1b2a3386798801ac74d3c5e",
    "0caa47e65769bbe10bd2c1800d2203bcd27bdfc180016629df595197040becf6",
    "ded0200f0d64683b654c00ca92e7e031a0ca8e0062903afa5b5f17c76120c4c4",
    "fa0da042ebab7240406178a578daa2acb525d9bf80b3689f3a7e9063cfc4f670",
    "a5d5666a0810562fad5d5bda3b836a6426b17a6bf62a7f72a88509efdddf1464",
    "770ae39e7a0c8a7927916b813a005ab2366c0d1b1dd552921d914b1ecd5e2f32",
    "9fed9a3c1fd125c64dae265e6b940fab2f1d695fabccb06acdb965e2a4cbd42b",
    "6e1dbab4ed716f0256d46490d8afb6f70136bae0764a6d2fd7b2151f8a79dcb7",
    "52c621f94f80d3b1c8772c7660a6b0e052523d4e0866a33fe6662bf308ccc525",
    "6807252ebbdd796e1ed5c5fe5d10968d9a980274c2dc2245f1ab70c20f944114",
    "17d6194e80cc02606bb5bd7a23ab33b84b94e90bffe395964769f3c72424300e",
    "3ee32c9351000c902c10c5caae7dced3803f3e9028bb9e0da34ad0177d2e7be7",
    "264e86c1cbadda6b2ccd0995e1fa03d5ee618e2b9d8b8d9ee4d22bf4134cb4f9",
    "b09f863f0125cd90dc4c36c2cda6d76272d6ee13c899e31fb2fc5d664fd4798f",
    "593116b6e8eef7729e559b3bc540ce6edf69a7ea9cbd2ece665a7e519c353e67",
    "c6ae8c6ac675aead3412b7458cd39d432b1cbcb96ef9c0767565827c96c664ed",
    "03a9af217e297855d18c6a8cc4693fcf4f95849b86e4347eb7eb4fe270b3d105",
    "5c50c4abd0237ea663dcbb8a5961658d9534caa8a9d7d8617a84e0b490bf9e70",
    "56c4a5564e62ebedb17bda6cf85f6940c2e7d623c5eadd5e17141fa729ddab14",
    "54b6bb06880aa77a612112a383c93676e1dd280118114d3856d1bd973e032afc",
    "2ec72b2255795a7b4845d4da99abf4a852dc08b98ac5cdc41176433eb124dc1d",
    "5b779fb293202c4299376ef803e5e6ec3dd9ec120eaf27ea890f771dda6361ef",
    "17b34583b5e02ee5d1655a821a902511a1a3e8eb4857f2d570a65f375412d425",
    "381b6b23dcceb2e10bea6632a92250a9aff927bbbc42791b0438426279d777eb",
    "0b2bd3568fb96093c539ac16c5ddfae8d4de6edb64c106f54f7a29931d369d0e",
    "453a16ae9a65e395f58874a6b08ac2c85f0f1f90d6b8721e098cc3489950829d",
    "7b56916c6c4fb9a6f8038db1cd49874a2489c323672d9e38e967d1cd6d7d1db1",
    "41d50e605d1e39a9a4be00ef09181d18428e23f366383d192e993529c91c220f",
    "05311ed06e15d0c4a1cdafc3362c7c8b1fbdbf2d16069de92a44b4d658e04973",
    "5e2e474f573c02b52dad3f10f42950e2e245c24023516388d2af55246b3d92f1",
    "0290cb7f3f7fb768aadb5c292e0933ca1e52c36b22e0b6857dfa3267402c1e18",
    "781848de083a3a0de49ae49fe884599a92d017768b070dca780917f6a60916c7",
    "f34f2075d6c2528c0829c48bb9fc4043a0bfdbb26acf722d1517b67c57cf5e40",
)
STATE_FORM_STATEMENT_IDS = (
    "7a34ee5c097abec3395b3144cbae0bff7413ece16c79b7541c997636cb0be120",
    "cbf6c93b1b009e2e03f6cabac00c4776b0381cc4e6f93f2fb833d1ca9e72fc97",
    "0b381309d82872f11307b73446c944d57b6be83f02a89dd9f23c40c7241349f9",
    "5df1de18eb3cb76f38b635c81fa1add72983aabe2cd26e7d5a848f6aebb8be72",
    "a1ce338f549d85b1bde69bae312bd5e2928f4207b6f3f58cf72163675c3c300e",
    "b7c10d5ca1eaf0192a9323e982780b196116b581aeab79b36d5ed43fd8d44fd1",
    "2105fce9f4488eeaa93913628f63dc5345033d23186644be83a4f5d577f23867",
    "47017762332d02f73157f85782a28ef139acff3a542f50f3d0200894ac415998",
    "5cc3830db7eb847fce8dc6926d3c2313f1253d09d5b780047c503eb632834a21",
    "d6a27797099ff55a70016ac62f486b6b04059284e70f160b777405f3da5b13c3",
    "301eb1908e99947db1e826623623c0bb5b3d3489b42872fac7acfaf5d8b9ba95",
    "c973da590eddfcbb752f7bbe93d4a76461ca8161b92945b7fdffffeed8071e1a",
    "c89d69111546e028229cde242febb661987612e2170d6c9268528571d14e4621",
    "37f23e62b26e0f89ba1dce1983251a197b2f5cfa24e3c71a1293994664e2bd23",
    "91d0dcf7e3bd8eae6ef2f690f14068c940c32d4c1f417b5661f9bb995b226665",
    "6a359753af497812fba0683c77b2e8b07d52e40937906d770d619508c9eda68a",
    "b321bb7e9e538d824c7e194aff69ebc3e62902c6c4580b0bc8d0da4eb1a186dd",
    "03dafcc9d115af36d25587fa94cb0f7f32aa62c76a4c5bdf07b27505fa27da82",
    "d4490fd4ae7bacec9c6bfb7dbaf248840d7ce4245af66c37bc803a5b7d71228e",
    "b5eda4abccaf4e656c599fdf2320ca172351ed7957df79a66d1c31c809afe024",
    "4311524ec5cd8207454a76f48241ad24447e509f90a1a5662d84cc2d80be9897",
    "3be9cdfa59790c6093ce997acd88527514ad18443997105c179ca92dbb8b06a8",
    "ffe7421ac9d93a9fea22903d5145b215009aa0bc112ceac0a2ac2f6a6323dcc3",
    "e11c660cb453f49c4462dd1e3520375992c02c64b76a28e77e5536ae495bad1b",
    "558110170f915b56acf28880339fa6d7d6811f84803870901d6d3dc7ab2ae61f",
    "2df2234cba02115e9a19fe52d0d5c80af904ca78bff8f2d67e95d7457ddc5838",
    "45d684532d8076cc742bd857b39bbb79ed3b2105ab021004b8e5919a0e10b367",
    "2dd3546dcf2aef0762ec5805820760a051c8c579ee4b0eda08c0d9e7f3226cf4",
    "b5870cab343940e4e9f33871c98b4c24fc7f1324afadc64ecf3d1106bd52d05c",
    "1b53adbdcf7a20ccba6f231f8a3b877aba0b750f329e818d6a9bb2773b9959c7",
    "020976a0c155fd00ca02a0d9eb9a5d8ed5a876be70e2e99434f84b84b63d22ce",
    "5cec2d2fe2d937352709d0d499bbdec51ff1f3e8ca64594285e4ffb1b19f4cf8",
    "0fb318876a97134050201121d98ee04c4dfc10ff661f22ff7481d01b8b2f6b33",
    "c04e3151686f44a0ee06c2a9557850d5791e1d82803d53fdcd3e2d0d3a3d0b5d",
    "850dcf4f1ae0e454beb9e99a6c3cc62a25a297879ea9560559acd5f8b8863b45",
    "05ef13f79aadb0bbf80b22f8b1e98b6efa5413b6658dbfb39760f850dba68989",
    "4ebbddb8b1cb7f7bae5fa3548f3d8c7b17854243d50332dc67dd7990eb1f7523",
    "c7904e0bad5db7105c3d64b635484b30f0f5e70e63bd9bcaf4ebf8e2a99a25fe",
    "d1d37a665a68b054798556c1b52e7bc3bed6516f0fb1b379ef006ca0d7652e39",
    "acda35ac4d21ec1f3f5bc181f9ba11cde6e18a5fbc6c0d2c4ed1de9102cfe080",
    "133550566d4a774f521d07884566f1243cc621c16825c108e648260ee19d5893",
    "e3389dd76f686ddaca60510dd099a73fcf286c5d305e6f7268522ffc150e2273",
    "516edc2a5fdc40f10562e892805aebb8c0c314bf282d4c6a325f9b9399aaf1a5",
    "f9dc52af720f84aa828f72bc8e8d882d7247d08905b35241fce4a3dbcffd5fde",
    "057890160bfdae20a7add0e2ec8b581874af772caf32db68b8a64ea79c2879d8",
    "75c286f547bc4050c6b07ffb23d1a9ad2993419dc62cdf02429bcd5b6a1d1937",
    "fd6fbd1fa094e23e99acd76a7a7f4fec86741cddbac89fcdfb733bccc70786f5",
    "5616f6a3969473715bd4d36d7d4317583eee5926d5159149dc2094f087eede16",
    "3776ffdaccdabfb9a1ecb45fdc0c129e4b3e746472868e0736dd65f6754ca929",
    "56d26f1a5d15c0c4a2ad93d5c237ca088530190ea382d538475b449c01f2f51b",
    "9efe739f709bfb56fbd8d9fe7c9800cf6237268242a61ba45dcd22a954004f75",
    "cca9d169b46a6f56831712e59c91845cd1e30f5d20f191b898533fd35c104e35",
    "4ef6fd700e7b049f0162997d0fa81c08c4e3de9d7aa5a120c4a504bbd97de47e",
    "3c884b360dbfd362eab19099ba6ad0c96d5290b4249a8417d847c411d3e6487b",
    "579db7615d1c40d9b973ee18e1245fd3b670b8511922f2860deefb6a40c8ae44",
    "0826ae8b88cef9e722c84461d0b47359c46d19241ab992ab888f41fd2d16baa8",
    "cd1663ceb0e0941c0ad1fe8287728a2d41365d04bf52da5f253e78d251d35955",
    "7896685fcdbc53550f6e96ac823671faf380f09d8c415b1662929f0f72a2f2e8",
    "6dc367e0683e9615e4aeb6c43e24a1a918176f5989b8dd8d2f94e413a51624f8",
    "89b1cd919147ae1f974fa388388251bf17280866a0d20afb52ccba267298d41f",
    "4ba8ac8a542834db88083ecf8ce2fd9e4053a7c175b6ebc0e40caf030cb579f1",
    "58f2f552f74de7b3b85f26c1b45cfb6b9087b6aef1a5049355ba9a5948b01aae",
    "f3ad283c813a7bd13c2f5f602e97ebfa669a065c98948afe75e99f9df87065c6",
    "e8faeade1cf500b7c4a41cddd5f9f9c4c039ac703309f078fe7995b8edd1156a",
    "9593427b2a36c9a79ba6402540223783e27cc3a12f514732bda74e81e6e13eac",
    "d851961eb17aaf6b254131d169feb47ab7a8c3fd200a00646dd278b96cff7633",
    "5798d2880d76b6e611dc6c756cabe514b389f6bfb83d3b0a033b67315356134d",
    "b13053e7c2792b5aa9254b3c9107983b6d33e744fb105ed2617024e764810159",
    "753e10550dbff81259f52c3ae0f6c6fd2ba8295a07cea017e1a900222eba9ac3",
    "52eccc856429fe6df17c872125977f2651ac933e876211233628d54abe433582",
    "6e8f8960300de2c3330656e54dcbfa813b1355b453b1f8c4c4363b5361ccf799",
    "c6620cc09fd432bc3a504c20a232a4ca347f7667f78a04dfd60e43768e80303b",
    "a2772a9887a071a933d079df7623605a8653c4eb7ff2b68699f99db2c8a4c4a7",
    "99e4b7d6ec9dc2f422b53f3faad5760de9b568b1634872db37f8c61f5669796b",
    "fc6d53363e7b2050728743bdecbe6a0961716cb939d61133b53107523791caee",
    "0ffd8237e5daa8fa9bcf7b0cd65abe8fb0918c9e0eb3ad245c1c960cc5483cc1",
    "082e8815bbed9d15ff0cfd83524252e8fce389741a5b82dc03ebcdf6dd09d1d0",
    "4b29da3e725e989939a01ec1b8a607f250ae40bab74f53d5ccd1358d7e29358a",
    "96f48e563b5ff8e6aace61f5e75fef0d45212eddcbe247ad1668c512de70fcf8",
    "957c082d4b60607e363ed2abf56bc372824937e2f3c65f58b9428f3f29754c62",
    "267a973bb694aa9d8f8314b24cb68e31703ac1d4cfab989365fba0a86cd9d1ba",
    "1a3cf10104175d69077ecbadd180d533849efe45f088888cb13dc1a3d10771bc",
    "5353e2d61da3c99b95a1d6c547e3e536922fbfdd35a1a613001cd482ded0b091",
    "3cd028eb4acc0117d766afa4c589cb0578518b1a30c24cfda2ad6110b7add689",
    "0da79dacb556baeea204782dd8f1a08a4b1c49c5ffbd1a66d5354ca2edee695b",
    "7b6b4d386950167d47c7cf5ad0eb1b234b686df86a54da635adef33ac7707ed3",
    "78e52857b51bf81f6d140f12d476aaf68bd9266be7e252a3525c0dfa5dcf0159",
    "12226377175bab15cd721b3d742da0f4da18f3834e418d470a402a19c48be7ee",
    "3cffbd750f70b341de6d5e144469e34f40c3c62f1fd5db19989a6d2663a547d3",
    "01d93956d24218e23bb53956e0bd33e94f40a7f80cd75ee46c2621df8b70f56f",
    "a02945b86ec64bfb4f4faaf612a7ab5836b214b6a70922ce6be1d4823ad088b2",
    "bef0a27648a863170be59b38a237110332137bed687e09959a6200cf4ad49132",
    "92e0ae8a42d85de3b7937bff65fa856f577976f86d3c29d24ccba878fbb8ac50",
    "3e64535fdc74e63b8b97b8081c655f41bd4c5dec5309c340326a28d070746481",
    "9fc55227548e3bf6e3204356ca8c3078b5c42c1a961de3d5859b23c9f9b80ca5",
    "284f703c78e697e1e0d18ddb9048da821c348ee27c936638afffecce66a5c076",
    "e6a42adc787a8445fc3b1c7a4f541d36ae69d5cf7b0e39b92c5f7bf604c0e6a7",
    "6541898250d7dce40a58b0fe0c2a16b24b8e72f960c7b72a60a6ac921a9787ab",
    "ed51b86d772ca1181a67c37dce7db0cb3918e8442c83862f20e419ac45beabc6",
    "fce6d56b7cbba7ebb5da79bdf602da7c0ed39825add2f72fae4e6bad57af7240",
    "a926def6bbae6b914571eedd610234e69c05d8e99b33e698a612def82762dae5",
    "c5de098b16aece0821132d254a47de2f21b82c2c81286f31aae4a4f77feb0f05",
    "cce90789328ea1de568e45d6eaa0e9aa7ce72939598c15e3c249d02be5af8f8d",
    "cfb40a88aa07f5268e4e427621582be47cde76af83a6e1f651f22ebb19dc5a4e",
    "e37dd2d8b5390d9e58c50724b644f2899fec592e3e8316000e86e61606e18551",
    "a551b3a66d75332be5eed3658b411145ff4e030b83257582b45edca8698535ba",
    "bc67afc2bd7145b3ff0fcb72ca59923ba7a9e4bc7792c0e17d1e5e76200d464a",
    "9e449d65b0ee849d12d3f2bed9b37173f5289652606287c5afc9b1d72c94a117",
    "cd0c0850bcce2c413bb77bcb8d920ec98ff693b91da6ebd5dde7ad1cb7742781",
    "9faceca870ca64dac1682e9c7acc5cb3326c56d26e3715b40d1862881226bcca",
    "e5afca140e69f71d3ed6bd10e7c4aa21e674c9968d020879b7aff98fc2479a06",
    "0defac9594f57f6fd96fae1440d687f32a9a7220b38917fb89ca5bc2530c2a84",
    "f7c0a0c1cb272ff40bf66af86e5fb197e37eb6dcebf62313c9e5cecbfd04bad4",
    "8fc4e69b5915c16070e460cbebb327188f00e64e54c384fe480691fb39f5f177",
    "51325d0040858a6c444b7146f1c4dd3b30a05c5ae5a44e48ea7e8f03ecf21f91",
    "54be7ad4ff624e0291f85f423886874c54c71d3b835b20da121529656ecb08d0",
    "433e7a84b63e526dfa8ce0abcd65c7cbecf17205cd1d98767e0cbbe0809e3541",
    "64b6be1473e2113877bf5aebd9912b9d0844f65fa79de35d4d5bba4f7808dd7a",
    "ff2fb20915766413de18581630c8125401720512a5c0c9cdc933c84f157c703c",
    "e388ffacdc1b15dc1951920c249d4a511d83592c71cac0abb2c3f63dc7b6e041",
    "16e97cb5fb888eb7796e1e29c7224ea4ddf8cc9e8710b706e7f37fd2ac88bc59",
    "10f4006804318fee8288ff717a04e12ddf4409b14540c75e2f8c50cc855522ab",
    "350086119a346c090900c1ccf742bbb0e47f0cf73a7d947edd4007155b938a3f",
    "16e0b4d8a92438ce26657dbdd11a86bde4f24e60abdc0aa83721ff9bf6b24132",
    "e8a463dfadfb5af539d3a6b091bf7d4c53a28d06cd85f30cf156009f397aa37a",
    "51f42a0f72f96b0edecad60cc3de05fe2d34838ba3934361042d69f8e1d6dc81",
    "cdd535a5cee42f87cee45a9f4f1b3ef071c13e8a834e9efc028512020c1de50b",
    "81fe0d3f7eb366b67e375ef49ef95afd843fdf9e2aa1fc3fa273165afcee2cef",
    "b5303148ac0ce3c9b481602d85e519d75be536672000227ab615101f8533ed86",
    "84c83a3485f4d704601aed8ecb6b59166322968e5b705e62ea2bf3740715ef79",
    "24a489f5cca970368ad37c56238cc883bb42e610f34cb4da21ab0af3e92b8138",
    "a3337ba557b24f4e37b3ac844123d2815491269cfa7bf9b3ffa7e850af0d5b5e",
    "36b28737c6f7f727b34ec40a9033ef9b710f600c636955c40e2a3f72c705ac0e",
    "0452220339dfbc1d724210e87d3842bbae0ddd2ff1e6df500bec6c4557c7eaa5",
    "f784f404808ba30e36c8a43167d031c43efbb6ba141716a665bf440ffa154a28",
    "3956793c5525d451323437108356878c767b426047f5cabab4adafe8dcc5009f",
    "9833b7764fac31247062ef00144c0ce302311e51c7069725fe1263fa152de751",
    "192fa339b8db7a29082415491be713a36127dd8428f7a7db2447580acd5831f5",
    "e755b6b76876d289933be1ced905757a75c9dac3ba37ec1567027f10015b3114",
    "0c403ba71581236e01ee8c8efe2c196c16ecc7ea8ce511227e05c49405434e4d",
    "f1b1bf3165dfee038e53fe1760b57672b9755bf8cf55bef45cc9776772dbffcd",
    "90299b0277dd6c29e59e3e9cf0705616950fc4a485b66a2e574be854b41f2936",
    "3fc366948a942e64eab81e7843db3e2ca77276fa54c49b5c39cc5568ba3ce882",
    "bd3a2112df4860bc25e5106447a00681242b53d5f5455f95c91509a2e8154f7e",
    "c7a8c628a4033c5bf1373935aa68c5ba4fd3d400b30a72527b33e8f642dfc8de",
    "f3ff53d0f6b082221138b02948442cf17c61b2f778ca75942e644b0559e36705",
    "65897ebb7a978d07460dd74a3e559b486a8e32be6824eadb45387398a6e31645",
    "b2a411702add4d4a697be5806a5ddf02c8d828f494a93ecc6f925c2adf3f9c22",
    "e960d4603dd0d8d853279f4f4d0843f5bd2969abc47e3013f32401fe742cf9a5",
    "83b39542cae3643819c2abe12f895e198e734c4d0f905dea1d59d3a8f809cc5e",
    "f0c4ea007833b5e5cd43f925938f22d5fbd5300ef6e2f8fd1beec1a0dd824563",
    "51e28be0ff227fec9ed8ee33abf829665207eb5495d55f465b5aaced082e0d15",
    "7f641eb86eca3e65e6a7f72730e041790661ccd8b744a1fabede6955fc036e86",
    "0964a596e29c5a91030ac1cc8f4d0c38bebe75c8b49d558a6ee65ef2ba8ebba2",
    "dbd91524c04c1c4702546af0e03e8cc260b7bfdd17f8afde6296fdc90171474a",
    "02c5f5de64ad6b9f3721d68c244cbdc9e35c341636e65e383fdc62ca87af5321",
    "6b67e4c6e9727384e05f21b753482cfc22237b054fba619419db0eb8c7915ad8",
    "ee4710ca537e769d4a711d868fcf4f4c1830f6748442a6887938eb76e3eeadfb",
    "9580b4648de0b051edc2d04a4254783fbffd8995277093c1f1a2e39194d9a1af",
    "5a1bb23d40a9ece21a90fac0bb8d69c3f925209559a08c3b4b323db44f3647b7",
    "a9ae37eded85be9070a5984e9985d4a4caba8574c6d5e14a17fc2092a9e936be",
    "7a0afa7281b735b60732103ddbfd6818e2ffff8399b1bcb9ca4d3ba60459bc86",
    "195a27f3b6eb6546258f5af859542fc6e3c887290f2f56d6d5531f3635e14262",
    "b3c2a02c0a08b899a639675327c09028ff84e402fdeb4dcb01842360fabf1453",
    "eecfbd887694ff4d9c6a4bd7053d2065994fbbc16d6be6384acd7c7bea97e8b0",
    "c10ce0a52b259c6404a357e8a1cb4c5b9f1a76a0ad3608e6e0aa4f8f0f02a769",
    "b11119304134396a1905fb9a2af814ea3295438c61830c0a7fd752eea042799a",
    "17636da5289bd66b5ef3fd810b99127eee1d4160871e7a7e56181367a3d0711f",
    "4beaa588bb33fb557e0fbaecbcef41b2c940f1d438d3e4d416390958942cd0f5",
    "3e49cac9109d4e8c1d2a0c7f89e0ded1cc4e30bda5186da0328ac9920bb28df1",
    "2fa7b8d7899694c982fe001ce5bf9dec4c5c9256aa6162da6eb684d448b3f7d7",
    "15c9edf969351c6730cf1358fc679451cbefae2e05e81a73d63e2e6694b1c867",
    "450f00386b09feda9d1ee4849d3abd3e1dd8ffbfa5199686a0d687601038b5c0",
    "8e707ddb31a5d023c0da42466a31c04974246612ee02050a565f8441a44974d3",
    "c9299e72da9cd7d2ba6d64744d8b9e755570b78dedd096ba9f6027031a3deca7",
    "e6585dac77c5333c237d93bca90ee8fd214e30ad242803c2861f4cc588fd2338",
    "0e3c1a3f47230bc4db1dda1576fef71499d3e2330c038ac8f045d69f0d0cfdf6",
    "e14440888c0e3bbca886c12d227e7104b767557e4ad0bcdcb25faaf406d8a5fe",
    "18c234247a3b43707c53f398a64d2a8f0fe10ee532bc8dbbb2474e02dd446582",
    "460870ffa2ba316109a01dc87cb32ed888d97f090d228186beba9502116d6e82",
    "e09f2a737625390107e6709060aaadad515f8c53a8c3e449a3dfc474ed546a3b",
    "07d61e65f8bfbac38c7899d2c014f2bb8930747b3998fcebfb2e2f9c6074e04f",
    "949a457fdb152c110df3ab99f99c09e08a557c19d22b8b34b5ce4fcc9f1cd6d3",
    "7c2732069daf6b1fb7abd013480274977b3950a62782c45c413694082d64ef21",
    "c06c3ba3ac9c4a85cc5311649c094a126158301c67afca1800faf749c7233a57",
    "f3ed700f74607a585a63d43796c1d32f90dce533b1f29f54f96b4fe3ea2cd7a9",
    "5ffb33c9cde32cb9e3f983ea969e4926cad9b88abfa7758ae83a4dcf7b63467b",
    "48904081b766912b6f0d8bb56c8f68f101e75cbcc1760a4e710b33f72b7223f4",
    "fdccd06a69ca78d6d252e1f3fd5f9aa7f91c17590238abbf279ed7da15a599c8",
    "43ba973254aa21c0fbd8deaedfd16dcac0042f5a2b07800eb932603b5eda80b5",
    "85740b73fd3ce206fcefdbb08b855ed2d409d49f4b2a26df2f91cab343d21443",
    "ee8d64bac85f66cf079680bad271f4596a24e5a022ff6c77fed8c0ffda1c0cfd",
    "0e5e4b4c53c71db426ed1ebbfa82bc54c82aeb6c6f2049d50fb96d6e3bea0c22",
    "cca51a2111abdc152426b7d722e2c6225044d9af4715e99df1d2ccbfd916f763",
    "82c7a9c6c5be7c8a404414aae19d1376c33ef2ae9c68819afb422be9f7ac4e6d",
    "1c1ddd1d3c6e3af9b1215a739147df9b032f7ec01fed9cd04821933cccfe3118",
    "39c24c3bdf730c86a8a48bb33f975a35d51601ba2442949eccf71e9f5140031a",
    "968a3936ef23525dfc75c08a91cdd12b758246a7e5b4e2068ac91847ba0d38cb",
    "527654c3bb138a63ae2b223877b5ad8dc0051b6c3210302eb1782f7949a3fc5a",
    "242a8a269a5b1345b66b975e4f7f30a4bc0bbb4401e6976c3e8bec92d131203b",
    "8b0b8c4916f70bf48ebe92784830220a5024aa71484d5d01ce23e1f7047715b9",
    "466a38dadb128b94d0d0f2fe0749768372da148a636500625b15ad527e5817ac",
    "3c3be5b750e5f9a0bbab16bc00f067dcf0a9d7006ef711b13ccf34d9ffc4df6a",
    "c3bb42b4742672b5a24ea03bd13de8043df7a61f4cb37abdc4cfb26ebfc5cc58",
    "a2e3741a85d50ebc070ed19ffdade6a26899bc8710ea17e7dba02e402ee9a5cb",
    "1f0a9a5080f02544a5e6aac0007f373d7ac28d1066be1db350c7e735683ab468",
    "14635ac45d4e7a84f9b91a4e3cb32f0fdf700a7c3dd8b833c45f1b250f91b7d3",
    "d0d2b7b91088f3a9bc04ea593a9b93de689375427bfa20852fab426f97f8d2fb",
    "3cffc3e473b31ef9eb7821ae893489860baebc3ec8af58be1ac25ecc6c456121",
    "aaba18bd63479f33c82cbbfbd6cd985ec1a62a13d9fe4520c7955b8d0ec22365",
    "d42313e739b3d479f81c871fd5b01f1f1bcd654743aecdf6bdfc4a750ea4f4a5",
    "1af8454c86e8d67e1f7da651c639c289a14c913b4d8291d1fdcfb70dce417834",
    "0dba95ad1a4367917452681009ee3e06fee9308fd01bdad7f6b371e565344b7a",
    "d892fdb38fb69c1ff92ba9828a02b5aed593015ebeaf027f7583a79cc408f018",
    "4e3eb27a123197df05ad200ae04f16ac02a7085edc9ff6be1cc64854d20666fb",
    "04e6db8444ae710d32907edbf9d3fe0a4a94848bb88bdab5d43aa0917b07a325",
    "6b77b98bbf941ff53b28341d4498de1821257cc07286aaf6d868917e6ef672a0",
    "45b1452476ac37177616519bc3df48d554d7a09ab225febec5436bb612198c66",
    "d5f7c5cf9d41cbb2e281a48493f491cdbe5cc78c8d95b1507101c8c01fc40949",
    "71f590774f510033fff1e0df5d503a40ecfb3a4169538bce7a69c9bf07c2731a",
    "3ffee9f823ccef59d875ece3e1f97183b0222d02723089aa5e08726a112c5786",
    "a0f19b62aeb2a8b10725f947e1ce12ffdace6462b0471b102da80884ffa24921",
    "7b765c609b4596c4a609be6e833ce46506d5419dc8242d2a8bdd1bf68f51fc61",
    "af2acd9933eb00ede16a799439eb4f512ec78dacf756f5174deb3cbd17ee4084",
    "e88797836a086677448ffd78fed18cf1c7670faab63df43a63f22bb5f8f97796",
    "38570856d781dde14ea1f3cf24a437a4cf9695c83401cc410ba6a2386d01dd98",
    "4ae8cba04a8435ef8c3fc56b8f3d5b77408740ed4541e283af4f6066a833e832",
    "0a7c6056469eed45063737f482ac0a4fdeda8dec8608982c735282c87fd9a8f6",
    "094fdb3424c36dedcb9e32a4c6950f0e2c6db35aebdaf7678034eacad2a69288",
    "b0085c4d8c1ef6d978b45e764f08660f83e58137f3eb84b5ffe65b41e06bb26a",
    "12a94d890ba82a4efd0d9b847e8dfab2befdbeaebbcd41f77ef5fad5868619a6",
    "32e00208ff691b8726627f102aeef3962515dc7cb9a2b1e0686422ba4fc951da",
    "b87e810a015e3dcafdc8346d65c8e3fa32cbd1f741641aa375b5854b752c51e5",
    "23afacc041496b02fae01ddf851f32d5680a450fc18eeb5fe36df758eeafcc17",
    "d844abcf68074ba3d0fe4a85fff5cff583e4cb59fe193cfc1c64e67c2b82f13b",
    "65b359ed43c9f43e9a1c10d8b6a7bc4c89435656e41ce1f91f033934162f5978",
    "7c1b78f953eb3758a33f5518ddb820b255f7b52a34c9e5196d091bad8e1f27b7",
    "266b8f319c428e8c6b08d7cfffc9225fedc9d639363e4a662828748faf9958b2",
    "c320f448ace552a61198578514dd3136c62eb24cd7fbe49c75259e7040633342",
    "4ad1cdb63e299d9ad0b96a255393e12bd8a031c4af15012c0a35f9b41162cfe6",
    "5021e8b7259ebeaa4355216467ace37b8c49a0e25005dc51da978a9f05e05cc3",
    "bf7b0bc652e86120fd3c054581f8b687e36f72eafddf7d105b86028f7f4c6bc1",
    "6358de559cfe9f3ae7a10f67686904c8cda8df0b4f418c2e3c5925e89ca80dd4",
    "cb15b4b63703cf75504698bf8e256c88e6ef2571af9a9a0906e2431b24e3163c",
    "336d6aa50d747e1ddc547faac9e288afa62c7f3897c2371b458697299da79087",
    "5205806d59331920e58b4cb87a2472fa0f44755458f7b473c7dc56612b6d07ca",
    "e09aaafd44367e85523f590b8c7bcbf2a7cb28b1cb2dab9cbdd842237a9a6acb",
    "1457d144e0b14ca35f717f434c7d4c22bbc73142fef9c9625c8d9fdf8e897f92",
    "8d443d516fee61a84ab72d0ca06680e034114ecd18715148c484e194666bb947",
    "6a982d963df881790e6eeab842a326b2bd6dee3e4b014badc51c87026be671b6",
    "d177c63608303b4ab72366641a1606817d017f20c4b4e17dd6fa690b3b49a65d",
    "d1f138614beaa8fa535c8f4e68fabfe08111ed26765e1773e0fbef9ee406625e",
    "b08be2b93635aa77c98b2bf37c6e872de68cc877c58adeddfc280e520a8ebb70",
    "c9ad473839203c290db224d5b729bce5132362909af476b2e1681c855d047f19",
    "03f7c39be1da61367cc249f84a3143cd2e771561f5f9e4be55f740bd8a7f37d4",
    "62fa15581db12fba63a8e56da4ec1a7e1223d07caacbc48ba264d09b23d36f5a",
    "6c6a44f30a8bd886183b852bc2d1ce3aac475ab7e8b9f1004a82ffec06166afd",
    "56f1b58f8ab7960b9926c163639d0840aaed8a505fafa62faa0b475e01ee2bb0",
    "c99dd45cafdb4e8f6653578916c989bcb6f9f17345f8cf202170363af9c228d4",
    "ff3e27d4d34781267fe81a4cca5c4726575c0d87f12b4eb55c220d59f2a90c9c",
    "50d6ae9adad65f1b0742d6d5770beff0ba12f5ebb284340c115860269c72a179",
    "d834c4682f6d424d9294201a7ade6d4a5bc4d344f1e5368529587f29b7cd71e8",
    "bb4cf648f2adb5f492a70a352d97304123f1ef14b8e3fab0197a37afd617770c",
    "46f2cdfa9f0d4f5049dab61a4f981dd2341997ec9051ea0309ef37a2ab043d08",
    "3cd6da30a961de626b46301719a85fa398d99cce4c31bd1cf1509e4f809d1326",
    "1ff7019a82d6d0139a20b83aafe5362016d2586a7292c2aebb860b7957504ad7",
    "1545c940a4661be967a851ff8bbf779aa9e309f229eee8d9727b77079dd13bd5",
    "3d3979db4d30eb1fda839c5dc8028ed3157857ee48059d0aeb72eba080722a17",
    "2fc97abb5a3237386d31ddbe51b34e7eea9ec8b8ef3453894ae8c66923b4d7c5",
    "5bbb2459a2d73437afafa71ccf729b6ced5de87a26203f51f73e7fc3b74860fa",
    "fbed463327dd6c7c1e87d4e72954f4d1e010a201ee9b45d414b26942eb5eac47",
    "dc9d8f83eb2025a83ee61cc2ed8d9c422d1aa631bdc0478ba0a7f5e441afd204",
    "a7e46755ab833078c8eaf54b7a2fa886607b282155cb4b3c5bef96f1daa4824a",
    "4d6af92950a9dd514e11e59e61b4952ec8a4a246385bc4b3ce7f65d3fa5f8d6d",
)
STATE_FORM_PIN_GROUP_REF = (
    "new-book-plans/state-form.pins.nibli::"
    "# State-form and political-membership family - executable coverage pins"
)
STATE_FORM_COUNTERFACTUAL_REF = (
    "new-book-plans/counterfactual/"
    "no-state-form-independent-current-review.pins.nibli::"
    "# Counterfactual: state-form source writer serves as temporal reviewer"
)
STATE_FORM_PROSE_REFS = (
    "book-1/00-opening-note.md::separates permanent public answerability from current office and current",
    "book-1/00-opening-note.md::shows why conviction and custody remove neither franchise nor candidacy,",
    "book-1/00-opening-note.md::**Current lawful authority:** a source-bound conclusion",
    "book-1/00-opening-note.md::**The federal constitutional bodies:** the People's Assembly, Regions",
    "book-1/00-opening-note.md::**The older assembly and approving-public entries:** produce a proposal and",
    "book-1/00-opening-note.md::**The State and the constitutional bodies:** the older `State` entry owes the",
    "book-1/00-opening-note.md::**Democracy, government, and administration:** Chapters",
    "book-1/00-opening-note.md::**Locality, membership, mobility, migration, and external relations:**",
    "book-1/01-what-counts-as-evidence.md::This is a real gap and it is stated here rather than buried, because it is",
    "book-1/02-public-answerability.md::Do not read those older routes as the map of the federal government added later.",
    "book-1/03-who-holds-the-pen.md::The later state-form rules close a different boundary without turning it into",
    "book-1/09-the-vote-conviction-does-not-take.md::Here, conviction and custody take neither franchise nor candidacy.",
    "book-1/09-the-vote-conviction-does-not-take.md::## The home custody does not move",
    "book-1/09-the-vote-conviction-does-not-take.md::Esa can challenge the missing or disputed age evidence.",
    "book-1/09-the-vote-conviction-does-not-take.md::## And the same silence as before",
    "book-1/12-changing-the-rules.md::A society that cannot change its rules is not stable, it is brittle. This record",
    "book-1/12-changing-the-rules.md::What the older route proposes is a trade.",
    "book-1/12-changing-the-rules.md::## Why this is thinner than it looks",
    "book-1/12-changing-the-rules.md::**Nothing happens when something receives the older law label.**",
    "book-1/method.md::The state-form and political-membership family completes the existing",
)
STATE_FORM_PART_V_REFS = (
    "book-1/15-the-five-joints.md::What does this design do about rotation? It now has a constitutional answer for",
    "book-1/15-the-five-joints.md::does not evade the state, it *specifies* one. It now specifies more than a",
    "book-1/15-the-five-joints.md::The binding layer is now wider. It names the tiers, bodies, holders,",
    "book-1/15-the-five-joints.md::**Survives, narrowed.** Under these rules, the floor is owed without an earned",
)
STATE_FORM_PROSE_DISPOSITION = (
    "Coverage, formal rules, executable tests, counterfactuals, and approved "
    "SF-B1-v1 prose are complete; these remain supplied-record constitutional "
    "declarations and authority only, and establish no authentication, operation, "
    "staffing, election, delivery, feasibility, liveness, calibration, outside "
    "time, or institutional action."
)
FLOOR_ENTITLEMENT_LINES=(
 "entitled(every person, event { secure() }).","entitled(every person, event { eats() }).",
 "entitled(every person, event { dwell() }).","entitled(every person, event { healthy() }).",
 "entitled(every person, event { learn() }).","entitled(every person, event { expresses() }).",
 "entitled(every person, event { believe() }).","entitled(every person, event { meets() }).",
)
MATERIAL_FLOOR_EFFECT_KEYS={"material-floor-inventory","dual-floor-ceiling-continuity"}
LIBERTY_EFFECT_KEYS={"privacy-power-limit","bodily-autonomy-power-limit","movement-power-limit","due-process-power-limit","association-power-limit","expression-power-limit","conscience-power-limit","family-intimate-power-limit","equal-civic-status-limit","residual-private-civic-freedom"}
ENVIRONMENTAL_EFFECT_KEYS={"environmental-conditions-right","environmental-information-right","environmental-assessment-participation","environmental-reasons-review-remedy","environmental-public-private-reach","environmental-equality-no-score"}
CLASS9_EFFECT_KEYS={"class9-commons-future-capability","class9-multiaxis-ceilings","class9-science-law-versioning","class9-precaution-nonregression","class9-restoration-nonfungibility","class9-initiation-interim-route","class9-liability-public-restoration"}
EQUALITY_EFFECT_KEYS=set(EQUALITY_EFFECT_POLICY)
FAMILY_LIFE_EFFECT_KEYS=set(FAMILY_LIFE_EFFECT_POLICY)
DELIVERY_EFFECT_KEYS=set(DELIVERY_EFFECT_POLICY)
OBLIGATION_EFFECT_KEYS=set(OBLIGATION_EFFECT_POLICY)
OBLIGATION_FINDING_EFFECT_KEYS=set(list(OBLIGATION_EFFECT_POLICY)[15:])

POWER_CLASS_IDS = [f"class-{i:02d}" for i in range(1, 11)]
CARD_V7_EXTRA_KEYS = [
    "manifest_key", "source_family", "posture", "evidence_kind",
    "primary_class_ref", "secondary_class_refs", "profiles",
    "domain_refs", "affected_claim_refs", "holder_body_refs",
    "holder_role_refs", "affected_role_refs", "checking_role_refs",
    "route_ref", "overlay", "public_claim_restriction",
    "structural_wall_refs", "related_power_refs", "enforcement_mechanism",
    "book2_owner_ref", "contract_terms", "profile_terms",
    "required_separation_pairs", "permitted_inputs", "prohibited_inputs",
    "permitted_downstream_effects", "evidence_authority",
    "negative_test", "counterfactual", "part_v_status",
    "book2_handoff", "source_refs",
]
CARD_LEGACY_EXTRA_KEYS = [
    "manifest_key", "source_family", "posture", "evidence_kind",
    "profiles", "domain_refs", "affected_claim_refs",
    "holder_body_refs", "holder_role_refs", "affected_role_refs",
    "checking_role_refs", "route_ref", "overlay",
    "public_claim_restriction", "structural_wall_refs",
    "related_power_refs", "enforcement_mechanism", "book2_owner_ref",
    "contract", "profile_contracts", "source_refs",
]
POWER_FUNCTIONS = [
    "decisive-fact-writer", "decider", "executor", "auditor",
    "final-remedy",
]
STATE_FORM_SEMANTIC_MAPPINGS = {
    "FS-POW-004": (
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-02", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-02", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-011": (
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-03",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-015": (
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-016": (
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-017": (
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-018": (
        (("FS-BOD-05",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-05",), ("FS-ROL-26",)),
        (("FS-BOD-05",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-022": (
        (("FS-BOD-17",), ("FS-ROL-27",)),
        (("FS-BOD-17",), ("FS-ROL-27",)),
        (("FS-BOD-17",), ("FS-ROL-27",)),
        (("FS-BOD-17",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-17",), ("FS-ROL-27",)),
    ),
    "FS-POW-023": (
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-25",), ("FS-ROL-46",)),
    ),
    "FS-POW-024": (
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-025": (
        (("FS-BOD-25",), ("FS-ROL-46",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
        (("FS-BOD-25",), ("FS-ROL-46",)),
        (("FS-BOD-25",), ("FS-ROL-46",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-25",), ("FS-ROL-46",)),
    ),
    "FS-POW-028": (
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-24",), ("FS-ROL-45",)),
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-05", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-029": (
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-26",)),
        (("FS-BOD-05", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-030": (
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-24",), ("FS-ROL-45",)),
        (("FS-BOD-17",), ("FS-ROL-27",)),
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-031": (
        (("FS-BOD-02", "FS-BOD-06"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-06",), ("FS-ROL-27",)),
        (("FS-BOD-06",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-032": (
        (("FS-BOD-03", "FS-BOD-21"), ("FS-ROL-26",)),
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-03",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-033": (
        (("FS-BOD-02", "FS-BOD-04"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-04",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-034": (
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-26"), ("FS-ROL-26", "FS-ROL-47")),
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-02", "FS-BOD-03"), ("FS-ROL-26",)),
        (("FS-BOD-26",), ("FS-ROL-47",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-035": (
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-24",), ("FS-ROL-45",)),
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-44",)),
        (("FS-BOD-24",), ("FS-ROL-45",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-037": (
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-25", "FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-06", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-25", "FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-038": (
        (("FS-BOD-01",), ("FS-ROL-25",)),
        (("FS-BOD-06",), ("FS-ROL-27",)),
        (("FS-BOD-01",), ("FS-ROL-25",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-039": (
        (("FS-BOD-01",), ("FS-ROL-25",)),
        (("FS-BOD-06",), ("FS-ROL-27",)),
        (("FS-BOD-18", "FS-BOD-01"), ("FS-ROL-27", "FS-ROL-25")),
        (("FS-BOD-05",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-040": (
        (("FS-BOD-01",), ("FS-ROL-25",)),
        (("FS-BOD-06",), ("FS-ROL-27",)),
        (("FS-BOD-01",), ("FS-ROL-25",)),
        (("FS-BOD-21",), ("FS-ROL-26",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-043": (
        (("FS-BOD-07",), ("FS-ROL-27",)),
        (("FS-BOD-02",), ("FS-ROL-26",)),
        (("FS-BOD-07",), ("FS-ROL-27",)),
        (("FS-BOD-07",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-044": (
        (("FS-BOD-05", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-02", "FS-BOD-03", "FS-BOD-06", "FS-BOD-21", "FS-BOD-24"), ("FS-ROL-26", "FS-ROL-27", "FS-ROL-45")),
        (("FS-BOD-05", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-05", "FS-BOD-21"), ("FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
    "FS-POW-045": (
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-25", "FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-04", "FS-BOD-06"), ("FS-ROL-27",)),
        (("FS-BOD-01", "FS-BOD-02", "FS-BOD-03", "FS-BOD-21"), ("FS-ROL-25", "FS-ROL-26", "FS-ROL-27")),
        (("FS-BOD-04",), ("FS-ROL-27",)),
        (("FS-BOD-19",), ("FS-ROL-27",)),
        (("FS-BOD-18",), ("FS-ROL-27",)),
    ),
}
STATE_FORM_DELEGATED_CONFIG_POWER_IDS = (
    "FS-POW-028", "FS-POW-029", "FS-POW-030", "FS-POW-031",
    "FS-POW-032", "FS-POW-033", "FS-POW-034", "FS-POW-035",
    "FS-POW-044",
)
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
        "e8dfa12813ce1bafc970e2350598482cdf5c159f6305846bc01063b6399ff3e7"
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
    "power_population_status": "complete-source-derived-contract-cards-and-allocations",
    "known_allocation_gaps": [
        "appointments-qualification function and its nominee, selector, and qualification positions",
        "custodial execution function distinct from policing",
        "independent ecological science and assessment function",
        "ecological and animal regulation and inspection functions",
        "emergency alternate authoriser and independent substitute reviewer",
        "Guardian alternate advocate and substitute reviewer",
        "border and removal execution function",
    ],
    "owner_ref": "new-book-plans/book-1-constitutional-coverage-map.md::Maintain completed constitutional coverage rows",
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
    (): frozenset({"FS-DFT-20"}),
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
    "power_crosswalk_dispositions", "constitutional_effects",
    "coverage_families",
    "dependencies", "dependency_loops",
    "refused_flows",
    "scenarios", "scenario_omissions", "thresholds", "defects",
    "receipts", "review_commissions", "proposals", "review_events",
    "scope_audits", "deferred_populations",
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

# ── the bodies specification ─────────────────────────────────────────────────
# A body card is a constitutional contract, never an operating description. The
# seven status senses exist because the state-form ruling refused to let one
# word carry them: universal standing is not membership, membership is not the
# ballot, the ballot is not candidacy, holding office is not holding power, and
# none of them is the answerability that survives removal. Collapsing any two
# was the specific failure the ruling named, so the schema keeps them apart by
# construction and the checker refuses a card that fuses them.
BODY_KINDS = [
    "universal-holder",
    "representative-chamber",
    "executive",
    "formal-continuity-office",
    "court",
    "independent-office",
    "administration",
    "predeclared-alternate",
    "predeclared-substitute-reviewer",
]
BODY_STATUS_SENSES = [
    "universal_human_standing",
    "political_membership",
    "franchise",
    "candidacy",
    "current_office",
    "current_lawful_power",
    "permanent_historical_public_answerability",
]
# The office contract the ruling requires of every office, in its own order.
# `anti_capture` is the eleventh: the ruling demands that direct AND de facto
# appointment control be observable, which the other ten do not reach.
BODY_OFFICE_TERMS = [
    "democratic_source",
    "jurisdiction",
    "ordinary_function",
    "delegation_boundary",
    "conflict_and_recusal",
    "appointment",
    "removal",
    "succession",
    "temporal_status",
    "public_reason_duty",
    "anti_capture",
]
ACCOUNTABILITY_ROUTE_TYPES = [
    "challenge", "review", "audit", "political-accountability",
]
ADVERSE_DETERMINATION_KINDS = ["none-by-design", "enumerated"]
CUSTODY_T3_RELATIONS = ["not-reusable", "retained-application"]
# Custody is the one ratified case-bound temporal contract. Every other power
# needs its own; the ruling says so in five separate decisions. Only the
# custodial executor may declare that it applies the retained record.
CUSTODY_T3_APPLICANT = "FS-BOD-35"
CUSTODY_T3_SOURCE_MARKERS = (
    "book-1-time-model-decision.md",
    "temporal-assurance-case.json",
    "temporal-assurance-case.md",
)
# The five mechanics the state-form ruling says block formal enactment until a
# body card fixes them. A card fixes one by recording its bounded delegation,
# never by inventing the number the ruling reserved.
BODY_DELEGATED_MECHANICS_REQUIRED = {
    "FS-BOD-02", "FS-BOD-03", "FS-BOD-04", "FS-BOD-05",
    "FS-BOD-17", "FS-BOD-18", "FS-BOD-19", "FS-BOD-24", "FS-BOD-25",
}
# The ruling's own words: do not relabel a current narrow fixture as a ratified
# institution. These are the exact equations it refused.
BODY_FIXTURE_RELABELS = (
    "convocation is the executive council",
    "convocation as the executive council",
    "court is the constitutional court",
    "current court as the constitutional court",
    "state is a completed federal",
    "state as a completed federal government",
    "assembly constant is the people's assembly",
    "electorate constant is the electorate",
)
# Routing prose drifts into arrival prose one verb at a time, and the ledger
# routes rather than delivers. A card states what is owed and what withholds a
# conclusion; it never states that anyone acted.
BODY_ARRIVAL_PHRASES = (
    "is delivered", "are delivered", "was delivered", "were delivered",
    "the remedy arrives", "the remedy reaches", "the election occurs",
    "the election happens", "the election takes place", "the body acts",
    "will act", "guarantees delivery", "actually arrives",
)
BODY_FEASIBILITY_TOKENS = (
    "feasible", "feasibility", "affordable", "affordability",
    "cost-effective", "capacity to deliver",
)
BODY_AGGREGATE_RE = re.compile(r"\b\d+\s*%|\b\d+\s+(?:of|out\s+of)\s+\d+\b")
BODY_STANDING_RE = re.compile(r"\bstanding\b", re.IGNORECASE)
# The three office senses the state-form ruling renamed away from "standing".
BODY_OFFICE_SENSES = (
    "current_office",
    "current_lawful_power",
    "permanent_historical_public_answerability",
)

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
EXPECTED_STATUS = "stage_4_repository_audit_complete"
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

# The scope-audit protocol is repository-enforced and byte-bound to each audit.
# External human review remains admissible optional input, but no project gate
# or release depends on recruiting or obtaining an act from another person.
PROTOCOL_DOC = pathlib.Path(
    "new-book-plans/full-society-scope-review-protocol.md")
PROTOCOL_STATUS_CANDIDATE = "candidate — author confirmation pending"
PROTOCOL_STATUS_CONFIRMED = (
    "repository-enforced 2026-08-23 -- receipt-aware mechanical-closure protocol v5")
SCOPE_AUDIT_POLICY_BASIS = (
    "new-book-plans/full-society-scope-review-protocol.md::"
    "## 5. Mechanical Gate A closure")
SHA256_HEX_RE = re.compile(r"^[0-9a-f]{64}$")
GIT_COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
ISO_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
UTC_INSTANT_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\dZ$")
REVIEW_CRITERIA = (
    "declared-rights", "declared-liberties", "declared-powers",
    "declared-duties", "protected-private-boundaries",
    "cross-domain-dependencies", "ordinary-life-account",
    "failure-and-recovery-paths", "adequacy", "accessibility-equality",
    "continuity", "resilience", "sustainability", "safety", "resource",
)
SCOPE_AUDIT_METHOD = "repository-source-derived-adversarial-audit"
SCOPE_AUDIT_RESULT = "passed-with-recorded-limits"
SCOPE_AUDIT_CONTROL_REFS = (
    "new-book-plans/13-full-society-ledger.py::"
    "def negative_controls(src: dict) "
    "-> int:",
    "new-book-plans/16-constitutional-closure.py::"
    "def negative_controls(source):",
)
SCOPE_AUDIT_V2_PENDING_COMMANDS = (
    "python3 new-book-plans/13-full-society-ledger.py --refresh-and-check",
    "python3 new-book-plans/16-constitutional-closure.py --refresh-and-check",
    "./verify.sh --emit-receipt new-book-plans/verification-receipts",
)
VERIFICATION_RECEIPT_REF_RE = re.compile(
    r"^new-book-plans/verification-receipts/"
    r"sha256-[0-9a-f]{64}\.json$")
SCOPE_AUDIT_EVIDENCE_CEILING = (
    "Checked repository structure and watched-failing mutations over the "
    "declared axes only; no independent-human warrant, reader response, "
    "external truth, operation, delivery, feasibility, liveness, calibration, "
    "timeless completeness, or authentication of the audit's own trust root "
    "follows.")
REVIEW_PACKET_PATHS = (
    "new-book-plans/full-society-ledger.json",
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
    "new-book-plans/book-1-constitutional-coverage-map.md",
    "new-book-plans/full-society-boundary-decision.md",
    "new-book-plans/book-1-assurance-portfolio-decision.md",
    "new-book-plans/full-society-scope-review-protocol.md",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md",
)
REVIEWER_CONSENT = "consented-to-the-commissioned-review"
REVIEWER_CONFLICT_CLEAR = (
    "no-reviewed-artifact-authorship-or-generation; not-custodian-darshu-"
    "or-dhanush; no-declared-conflict")
REVIEWER_COMPENSATION_CLEAR = "not-findings-contingent"
PROPOSAL_CLASSIFICATIONS = (
    "material-omission", "retained-limit", "duplicate", "immaterial",
    *UNESTABLISHED_DISPOSITIONS,
)
GATE_A_PERMITTED_CLAIM = (
    "The project has a versioned, reviewable scope map and assurance program.")
VERDICT_NOT_PASSED = (
    "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN "
    "POSTURE; GATE A NOT PASSED")
VERDICT_PASSED = (
    "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN "
    "POSTURE; GATE A PASSED")
REQUIRED_VERIFY_COMMANDS = (
    "python3 new-book-plans/14-reader-evidence.py --check",
    "python3 new-book-plans/14-reader-evidence.py --check --execute",
    "python3 new-book-plans/17-full-society-power-source-manifest.py --check",
    "python3 new-book-plans/13-full-society-ledger.py",
    "python3 new-book-plans/13-full-society-ledger.py --check",
    "python3 new-book-plans/16-constitutional-closure.py",
    "python3 new-book-plans/16-constitutional-closure.py --check",
    "./verify.sh --quick", "./verify.sh", "git diff --check",
)
LEGACY_V1_CLOSURE_CANDIDATE = (
    "e0e0ca1a09dc8bceaac95f29ab5f1afdc9795bb5")
LEGACY_V1_CLOSURE_SOURCE = "fs-ledger-2026-08-21-state-form-prose-v1"
LEGACY_V1_CLOSURE_AUDIT = "FS-SAU-34"
LEGACY_V1_CLOSURE_TRANSCRIPT = (
    "dc0eb1d869629a9093457fcc8a7c48d5a438777bae756e24a0447e4d60e1032f")
GATE_A_ASSURANCE_REFS = tuple(
    f'new-book-plans/full-society-ledger.json::"id": "FS-RTE-{n:02d}"'
    for n in range(1, 8)
) + (
    "new-book-plans/full-society-ledger.md::# Full-Society Domain-and-Layer Ledger — Generated Report",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md::# Constitutional-closure and model-allocation audit",
)

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
    "constitutional_effects",
    "coverage_families",
    "closure_requirement_profiles",
    "closure_claim_contracts",
    "model_allocations",
    "function_allocations",
    "dependency_loops",
    "loop_hazard_controls",
    "bottleneck_dispositions",
    "review_commissions",
    "proposals",
    "review_events",
    "scope_audits",
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
    "constitutional_effects": "constitutional_effect",
    "coverage_families": "coverage_family",
    "dependencies": "dependency",
    "scenarios": "scenario",
    "thresholds": "threshold",
    "defects": "defect",
    "receipts": "resolution_receipt",
    "proposals": "proposal",
    "review_events": "review_event",
    "review_commissions": "review_commission",
    "scope_audits": "scope_audit",
    "closure_requirement_profiles": "closure_requirement_profile",
    "closure_claim_contracts": "closure_claim_contract",
    "model_allocations": "model_allocation",
    "function_allocations": "function_allocation",
    "dependency_loops": "dependency_loop",
    "loop_hazard_controls": "loop_hazard_control",
    "bottleneck_dispositions": "bottleneck_disposition",
}

ENVELOPE_STUB_ID = "FS-ENV-00"

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


class ImmutableRepositoryInputs:
    """One-invocation cache for external repository inputs.

    The reviewed-source mutants are deliberately absent from this cache:
    every watched mutation still receives its own deep copy and traverses the
    full validator. Only bytes outside those in-memory mutants are cached. A
    successful check re-reads every cached path and HEAD once, so concurrent
    input drift cannot be accepted through a stale cache entry.
    """

    def __init__(self, root: pathlib.Path):
        self.root = root.resolve()
        self._bytes = {}
        self._text = {}
        self._json = {}
        self._digests = {}
        self._needle_counts = {}
        self._initial_reads = {}
        self._rehashes = {}
        self._metadata = {}
        self._head = self._git_head()

    def path(self, value: pathlib.Path) -> pathlib.Path:
        value = pathlib.Path(value)
        return (value if value.is_absolute() else self.root / value).resolve()

    @staticmethod
    def _stat_signature(path: pathlib.Path) -> tuple:
        info = path.stat()
        return (
            info.st_dev, info.st_ino, stat.S_IMODE(info.st_mode),
            info.st_size, info.st_mtime_ns, info.st_ctime_ns,
        )

    def read_bytes(self, value: pathlib.Path) -> bytes:
        path = self.path(value)
        if path not in self._bytes:
            try:
                before = self._stat_signature(path)
                payload = path.read_bytes()
                after = self._stat_signature(path)
            except OSError as exc:
                raise LedgerError(
                    f"cannot read immutable input {path}: {exc}"
                ) from exc
            if before != after:
                raise LedgerError(
                    f"immutable input drifted during initial read: {path}")
            self._bytes[path] = payload
            self._metadata[path] = after
            self._initial_reads[path] = self._initial_reads.get(path, 0) + 1
        return self._bytes[path]

    def read_text(self, value: pathlib.Path) -> str:
        path = self.path(value)
        if path not in self._text:
            try:
                self._text[path] = self.read_bytes(path).decode("utf-8")
            except UnicodeDecodeError as exc:
                raise LedgerError(
                    f"immutable input is not UTF-8: {path}") from exc
        return self._text[path]

    def load_json(self, value: pathlib.Path):
        path = self.path(value)
        if path not in self._json:
            try:
                self._json[path] = json.loads(self.read_text(path))
            except json.JSONDecodeError as exc:
                try:
                    label = path.relative_to(self.root)
                except ValueError:
                    label = path
                raise LedgerError(f"{label} is not valid JSON: {exc}") from exc
        return self._json[path]

    def sha256(self, value: pathlib.Path) -> str:
        path = self.path(value)
        if path not in self._digests:
            self._digests[path] = hashlib.sha256(
                self.read_bytes(path)).hexdigest()
        return self._digests[path]

    def needle_count(self, value: pathlib.Path, needle: str) -> int:
        path = self.path(value)
        key = (path, needle)
        if key not in self._needle_counts:
            self._needle_counts[key] = self.read_text(path).count(needle)
        return self._needle_counts[key]

    def adopt_bytes(self, value: pathlib.Path, payload: bytes):
        """Bind bytes read before this snapshot without reading them again."""
        path = self.path(value)
        if not isinstance(payload, bytes):
            raise LedgerError("adopted immutable input must be bytes")
        if path in self._bytes:
            if self._bytes[path] != payload:
                raise LedgerError(
                    f"conflicting immutable input bytes adopted for {path}")
            return
        try:
            metadata = self._stat_signature(path)
        except OSError as exc:
            raise LedgerError(
                f"cannot stat adopted immutable input {path}: {exc}") from exc
        self._bytes[path] = payload
        self._metadata[path] = metadata
        self._initial_reads[path] = 1

    def mode(self, value: pathlib.Path) -> int:
        """Return the snapshotted permission bits for an already-read path."""
        path = self.path(value)
        if path not in self._metadata:
            self.read_bytes(path)
        return self._metadata[path][2]

    def advance_replacement(self, value: pathlib.Path, payload: bytes):
        """Advance one intentionally replaced output before the final rehash."""
        path = self.path(value)
        if not isinstance(payload, bytes):
            raise LedgerError("replacement expectation must be bytes")
        self._bytes[path] = payload
        self._metadata[path] = self._stat_signature(path)
        self._initial_reads.setdefault(path, 1)
        self._text.pop(path, None)
        self._json.pop(path, None)
        self._digests.pop(path, None)
        for key in [
                key for key in self._needle_counts if key[0] == path
        ]:
            self._needle_counts.pop(key, None)

    def _git_head(self) -> str:
        proc = subprocess.run(
            ["git", "rev-parse", "--verify", "HEAD"],
            cwd=self.root,
            check=False,
            capture_output=True,
            text=True,
        )
        if (
                proc.returncode != 0
                or not GIT_COMMIT_RE.fullmatch(proc.stdout.strip())):
            raise LedgerError("cannot snapshot the current Git HEAD")
        return proc.stdout.strip()

    def assert_metadata_unchanged(self):
        """Cheap pre-replace guard; final success still requires byte rehash."""
        for path, expected in self._metadata.items():
            try:
                current = self._stat_signature(path)
            except OSError as exc:
                raise LedgerError(
                    f"immutable input disappeared before refresh: {path}"
                ) from exc
            if current != expected:
                raise LedgerError(
                    f"immutable input metadata drifted before refresh: {path}")
        if self._git_head() != self._head:
            raise LedgerError("Git HEAD changed before refresh")

    def assert_unchanged(self):
        for path, expected in self._bytes.items():
            try:
                before = self._stat_signature(path)
                current = path.read_bytes()
                after = self._stat_signature(path)
            except OSError as exc:
                raise LedgerError(
                    f"immutable input disappeared during validation: {path}"
                ) from exc
            self._rehashes[path] = self._rehashes.get(path, 0) + 1
            if before != after:
                raise LedgerError(
                    f"immutable input drifted during final rehash: {path}")
            if current != expected:
                raise LedgerError(
                    f"immutable input drifted during validation: {path}"
                )
            if after != self._metadata[path]:
                raise LedgerError(
                    f"immutable input metadata drifted during validation: {path}")
        if self._git_head() != self._head:
            raise LedgerError("Git HEAD changed during validation")
        if any(count != 1 for count in self._initial_reads.values()):
            raise LedgerError(
                "immutable input cache performed a duplicate disk read")
        if any(self._rehashes.get(path) != 1 for path in self._bytes):
            raise LedgerError(
                "an immutable input was not rehashed exactly once")


_IMMUTABLE_INPUTS = None
_GIT_SOURCE_CACHE = {}
_VERIFICATION_RECEIPT_CACHE = {}
_CACHED_SOURCE_MODULE_BYTES = {}


def install_immutable_input_snapshot(snapshot):
    """Install a fresh snapshot and clear process-local external caches."""
    global _IMMUTABLE_INPUTS, _READER_EVIDENCE_CACHE
    global _ASSERTION_STATEMENT_FINGERPRINT_CACHE, _PRIOR_REVIEW_STATE
    global _GIT_SOURCE_CACHE, _VERIFICATION_RECEIPT_CACHE
    _IMMUTABLE_INPUTS = snapshot
    _READER_EVIDENCE_CACHE = None
    _ASSERTION_STATEMENT_FINGERPRINT_CACHE = None
    _PRIOR_REVIEW_STATE = None
    _GIT_SOURCE_CACHE = {}
    _VERIFICATION_RECEIPT_CACHE = {}


def _input_bytes(path: pathlib.Path) -> bytes:
    if _IMMUTABLE_INPUTS is not None:
        return _IMMUTABLE_INPUTS.read_bytes(path)
    target = path if pathlib.Path(path).is_absolute() else ROOT / path
    return target.read_bytes()


def _input_text(path: pathlib.Path) -> str:
    if _IMMUTABLE_INPUTS is not None:
        return _IMMUTABLE_INPUTS.read_text(path)
    target = path if pathlib.Path(path).is_absolute() else ROOT / path
    return target.read_text(encoding="utf-8")


def _input_needle_count(path: pathlib.Path, needle: str) -> int:
    if _IMMUTABLE_INPUTS is not None:
        return _IMMUTABLE_INPUTS.needle_count(path, needle)
    return _input_text(path).count(needle)


def load_cached_source_module(module_name: str, path: pathlib.Path):
    """Execute exactly the source bytes cached and rehashed by this run."""
    target = pathlib.Path(path)
    target = (
        target if target.is_absolute() else ROOT / target
    ).resolve()
    payload = _input_bytes(target)
    digest = hashlib.sha256(payload).hexdigest()
    previous = sys.modules.get(module_name)
    if (
            previous is not None
            and getattr(previous, "__cached_source_path__", None)
            == str(target)
            and getattr(previous, "__cached_source_sha256__", None) == digest
    ):
        _CACHED_SOURCE_MODULE_BYTES[target] = payload
        return previous
    try:
        code = compile(payload, str(target), "exec", dont_inherit=True)
    except (SyntaxError, ValueError) as exc:
        raise LedgerError(
            f"cannot compile cached module source {target}: {exc}") from exc
    module = types.ModuleType(module_name)
    module.__file__ = str(target)
    module.__package__ = module_name.rpartition(".")[0]
    module.__cached_source_path__ = str(target)
    module.__cached_source_sha256__ = digest
    sys.modules[module_name] = module
    try:
        exec(code, module.__dict__)
    except BaseException:
        if previous is None:
            sys.modules.pop(module_name, None)
        else:
            sys.modules[module_name] = previous
        raise
    _CACHED_SOURCE_MODULE_BYTES[target] = payload
    return module


_SNAPSHOT_PATH_BASE = type(pathlib.Path())


class _SnapshotBackedPath(_SNAPSHOT_PATH_BASE):
    """Path whose content reads participate in the installed snapshot."""

    def read_bytes(self) -> bytes:
        return _input_bytes(pathlib.Path(str(self)))

    def read_text(self, encoding=None, errors=None) -> str:
        return self.read_bytes().decode(
            encoding or "utf-8", errors or "strict")


def _bind_reader_evidence_repository_reads(module):
    """Route Script 14's live repository reads through this invocation cache."""
    if getattr(module, "__immutable_repository_reads_bound__", False):
        return
    original_resolve = getattr(module, "resolve", None)
    if not callable(original_resolve):
        raise LedgerError(
            "reader-evidence validator exposes no repository path resolver")

    def snapshot_resolve(value):
        return _SnapshotBackedPath(original_resolve(value))

    module.resolve = snapshot_resolve
    module.__immutable_repository_reads_bound__ = True


def _prime_repository_reference_inputs(value):
    """Bind every extant repository path named by a path::anchor value."""
    if isinstance(value, dict):
        for child in value.values():
            _prime_repository_reference_inputs(child)
        return
    if isinstance(value, list):
        for child in value:
            _prime_repository_reference_inputs(child)
        return
    if not isinstance(value, str) or "::" not in value:
        return
    relative = value.split("::", 1)[0]
    if (
            not re.fullmatch(r"[A-Za-z0-9_.][A-Za-z0-9_./-]*", relative)
            or pathlib.PurePosixPath(relative).is_absolute()
            or ".." in pathlib.PurePosixPath(relative).parts
    ):
        return
    target = (ROOT / relative).resolve()
    try:
        target.relative_to(ROOT.resolve())
    except ValueError:
        return
    if target.is_file():
        _input_bytes(target)


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
    _prime_repository_reference_inputs(reader)
    validator_path = ROOT / READER_EVIDENCE_VALIDATOR
    lock_path = ROOT / "new-book-plans/verification_lock.py"
    if lock_path.is_file():
        load_cached_source_module("verification_lock", lock_path)
    module = load_cached_source_module(
        "reader_evidence_validator", validator_path)
    _bind_reader_evidence_repository_reads(module)
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
    if _IMMUTABLE_INPUTS is not None:
        return _IMMUTABLE_INPUTS.sha256(path)
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def load_json(path: pathlib.Path):
    if _IMMUTABLE_INPUTS is not None:
        return _IMMUTABLE_INPUTS.load_json(path)
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
    count = _input_needle_count(target, needle)
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
    if type(src.get("schema_version")) is not int or src["schema_version"] != 7:
        raise LedgerError("schema_version must be the integer 7")
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
    profiles = set()
    if family == "state-form-and-political-membership":
        profiles.add("ordinary-public-power")
        if any(token in key for token in (
                "certif", "appointment", "qualification", "membership",
                "election", "selection", "succession", "record")):
            profiles.add("consequential-record")
    elif family == "substantive-equality-and-anti-subordination":
        profiles.add("liberty-power-limit")
        if any(token in key for token in (
                "diagnostic", "finding", "determination", "review", "record")):
            profiles.add("consequential-record")
    elif family == "economic-pluralism-and-protected-private-sphere":
        profiles.add("economic-private-power-limit")
        if "scarcity" in key:
            profiles.add("physical-scarcity")
    elif family == "family-dependency-reproduction-and-collective-plurality":
        profiles.add("consequential-status-supported-decision")
        if any(token in key for token in (
                "collective", "minority", "indigenous", "title", "consent",
                "customary", "self-government")):
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
        else:
            profiles.add("ordinary-public-power")
        if "intelligence" in key or "record" in key:
            profiles.add("consequential-record")
    elif key == RETAINED_FORMAL_KEY:
        profiles.update(("ordinary-public-power", "coercive-protective"))
    return [p for p in POWER_PROFILE_ORDER if p in profiles]


def _power_primary_class(row: dict):
    key = row["provisional_key"]
    family = row["source_family"]
    if key == RETAINED_FORMAL_KEY:
        return "class-04"
    if family == "state-form-and-political-membership":
        if any(t in key for t in ("election", "elector", "amendment", "ballot")):
            return "class-05"
        if any(t in key for t in (
                "membership", "residence", "local", "regional",
                "portability", "secession", "competence")):
            return "class-08"
        return "class-06"
    if family == "substantive-equality-and-anti-subordination":
        return "class-01"
    if family == "economic-pluralism-and-protected-private-sphere":
        return "class-02" if "scarcity" in key else "class-03"
    if family == "family-dependency-reproduction-and-collective-plurality":
        if any(t in key for t in (
                "collective", "minority", "indigenous", "title",
                "consent", "customary", "self-government")):
            return "class-08"
        return "class-01" if "adulthood" in key else "class-04"
    if family == "ecological-commons-and-non-human-animal":
        return "class-10" if key.startswith("animal-") else "class-09"
    if family == "public-safety-defence-emergency-and-external-power":
        if "intelligence" in key or "surveillance" in key:
            return "class-07"
        if any(t in key for t in (
                "arrest", "detention", "custod", "force", "search",
                "seizure", "border", "expulsion", "extradition", "transfer")):
            return "class-04"
        return "class-06"
    raise LedgerError(f"no primary class for {key}")


PROFILE_SECONDARY_CLASSES = {
    "ordinary-public-power": "class-06",
    "liberty-power-limit": "class-01",
    "coercive-protective": "class-04",
    "emergency": "class-06",
    "commons-future-condition": "class-09",
    "non-human-animal": "class-10",
    "collective-authority-title-consent": "class-08",
    "consequential-status-supported-decision": "class-04",
    "economic-private-power-limit": "class-03",
    "physical-scarcity": "class-02",
    "consequential-record": "class-07",
}


def _power_secondary_classes(row, profiles):
    primary = _power_primary_class(row)
    result = []
    for profile in profiles:
        class_ref = PROFILE_SECONDARY_CLASSES[profile]
        if class_ref != primary and class_ref not in result:
            result.append(class_ref)
    return result


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




_ASSERTION_STATEMENT_FINGERPRINT_FIELDS = (
    "id", "statement_sha256", "occurrence", "kind", "statement",
)
_ASSERTION_STATEMENT_FINGERPRINT_CACHE = None


def _prime_assertion_fingerprint_inputs():
    if _IMMUTABLE_INPUTS is None:
        return
    _input_bytes(pathlib.Path("new-book-plans/7-assertion-surface.py"))
    _input_bytes(pathlib.Path("new-book-plans/constitution.nibli"))
    assertion_contract = pathlib.Path(
        "new-book-plans/assertion-surface-contracts.json")
    contract = load_json(assertion_contract)
    _prime_repository_reference_inputs(contract)
    strata_path = os.environ.get("NIBLI_STRATA_FILE")
    if strata_path:
        _input_bytes(pathlib.Path(strata_path))
        return
    engine = (
        os.environ.get("NIBLI_PIN")
        or shutil.which("nibli-pin")
        or os.path.expanduser(
            "~/projects/dhilipsiva/nibli/target/release/nibli-pin")
    )
    engine_path = pathlib.Path(engine)
    if engine_path.is_file():
        _input_bytes(engine_path)


def _assertion_statement_fingerprints():
    global _ASSERTION_STATEMENT_FINGERPRINT_CACHE
    if _ASSERTION_STATEMENT_FINGERPRINT_CACHE is not None:
        return [
            dict(zip(_ASSERTION_STATEMENT_FINGERPRINT_FIELDS, values))
            for values in _ASSERTION_STATEMENT_FINGERPRINT_CACHE
        ]
    _prime_assertion_fingerprint_inputs()
    proc = subprocess.run(
        [sys.executable, str(ROOT / "new-book-plans/7-assertion-surface.py"),
         "--fingerprints"],
        cwd=ROOT, check=False, capture_output=True, text=True,
    )
    if proc.returncode != 0:
        raise LedgerError(
            "assertion statement fingerprints failed: " + proc.stderr.strip())
    try:
        payload = json.loads(proc.stdout)
        rows = payload["statement_fingerprints"]
    except (json.JSONDecodeError, KeyError, TypeError) as exc:
        raise LedgerError("assertion statement fingerprints are malformed") from exc
    required = set(_ASSERTION_STATEMENT_FINGERPRINT_FIELDS)
    if any(set(row) != required for row in rows):
        raise LedgerError("assertion statement fingerprint schema drifted")
    _ASSERTION_STATEMENT_FINGERPRINT_CACHE = tuple(
        tuple(row[field] for field in _ASSERTION_STATEMENT_FINGERPRINT_FIELDS)
        for row in rows
    )
    return [
        dict(zip(_ASSERTION_STATEMENT_FINGERPRINT_FIELDS, values))
        for values in _ASSERTION_STATEMENT_FINGERPRINT_CACHE
    ]


def validate_coverage_population(src: dict):
    population = src.get("coverage_population")
    if not isinstance(population, dict):
        raise LedgerError("coverage_population must be an object")
    exact_keys(
        population,
        ["status", "completed_source_families", "expected_final_card_count",
         "expected_constitutional_effect_count",
         "legacy_fields_permitted_until_complete", "evidence_ceiling"],
        "coverage_population",
    )
    completed = population["completed_source_families"]
    if (not isinstance(completed, list)
            or completed != COVERAGE_SOURCE_FAMILY_ORDER[:len(completed)]):
        raise LedgerError(
            "coverage_population.completed_source_families must be an exact prefix")
    expected_status = (
        "foundation" if not completed else
        "complete" if completed == COVERAGE_SOURCE_FAMILY_ORDER else "partial"
    )
    if population["status"] != expected_status:
        raise LedgerError(
            f"coverage_population.status must be {expected_status!r}")
    if population["expected_final_card_count"] != 210:
        raise LedgerError("coverage_population final card count must remain 210")
    if population["expected_constitutional_effect_count"] != CONSTITUTIONAL_EFFECT_COUNT:
        raise LedgerError("coverage_population constitutional-effect count drifted")
    if population["legacy_fields_permitted_until_complete"] is not True:
        raise LedgerError("transitional legacy permission must remain explicit")
    if population["evidence_ceiling"] != COVERAGE_EVIDENCE_CEILING:
        raise LedgerError("coverage_population evidence ceiling drifted")
    coverage_deferrals = [
        row for row in src["deferred_populations"]
        if row["record_type"] == COVERAGE_DEFERRAL_TYPE
    ]
    if completed == COVERAGE_SOURCE_FAMILY_ORDER:
        if coverage_deferrals:
            raise LedgerError(
                "complete coverage may not retain the coverage-contract deferral")
    elif len(coverage_deferrals) != 1:
        raise LedgerError(
            "partial coverage requires exactly one coverage-contract deferral")
    return completed


def _validate_artifact_paths(values, context, allow_empty=False):
    if (not isinstance(values, list) or len(values) != len(set(values))
            or (not values and not allow_empty)):
        raise LedgerError(f"{context} must be a duplicate-free artifact list")
    for value in values:
        if not isinstance(value, str) or not value:
            raise LedgerError(f"{context} contains a blank artifact")
        path = value.split("::", 1)[0]
        if not (ROOT / path).is_file():
            raise LedgerError(f"{context} names missing artifact {path}")


def validate_coverage_families(src: dict, ids: dict):
    completed = src["coverage_population"]["completed_source_families"]
    rows = src.get("coverage_families")
    if not isinstance(rows, list) or not rows:
        raise LedgerError("coverage_families must be a non-empty list")
    statements = _assertion_statement_fingerprints()
    statement_ids = [row["id"] for row in statements]
    assigned_statements = []
    assigned_cards = []
    assigned_templates = []
    assigned_refusals = []
    assigned_crosswalks = []
    assigned_effects = []
    for i, rec in enumerate(rows):
        ctx = f"coverage_families[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            ["id", "title", "state", "source_family_refs", "card_refs",
             "template_refs", "refusal_refs", "crosswalk_refs",
             "effect_refs", "formal_statement_refs", "pin_group_refs",
             "counterfactual_refs", "prose_refs", "part_v_refs",
             "blocked_before_drafting", "source_refs"],
            ctx,
        )
        if rec["state"] not in COVERAGE_FAMILY_STATES:
            raise LedgerError(f"{ctx}: invalid coverage-family state")
        for field, array_name in (
                ("card_refs", "powers"),
                ("template_refs", "power_contract_templates"),
                ("refusal_refs", "power_refusals"),
                ("crosswalk_refs", "power_crosswalk_dispositions"),
                ("effect_refs", "constitutional_effects")):
            _typed_ref_list(rec[field], array_name, ids, f"{ctx}.{field}",
                            allow_empty=True)
        families = rec["source_family_refs"]
        if (not isinstance(families, list)
                or len(families) != len(set(families))
                or any(f not in COVERAGE_SOURCE_FAMILY_ORDER for f in families)):
            raise LedgerError(f"{ctx}: source-family refs are invalid")
        formal_refs = rec["formal_statement_refs"]
        if (not isinstance(formal_refs, list)
                or len(formal_refs) != len(set(formal_refs))
                or any(ref not in statement_ids for ref in formal_refs)):
            raise LedgerError(f"{ctx}: formal statement refs are invalid")
        _validate_artifact_paths(
            rec["pin_group_refs"], f"{ctx}.pin_group_refs", allow_empty=True)
        _validate_artifact_paths(
            rec["counterfactual_refs"], f"{ctx}.counterfactual_refs",
            allow_empty=True)
        _validate_artifact_paths(
            rec["prose_refs"], f"{ctx}.prose_refs", allow_empty=True)
        _validate_artifact_paths(
            rec["part_v_refs"], f"{ctx}.part_v_refs", allow_empty=True)
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        require_str(rec, "blocked_before_drafting", ctx)
        if rec["state"] == "planned":
            if any(rec[field] for field in (
                    "card_refs", "template_refs", "refusal_refs",
                    "crosswalk_refs", "effect_refs", "formal_statement_refs",
                    "pin_group_refs", "counterfactual_refs",
                    "prose_refs", "part_v_refs")):
                raise LedgerError(
                    f"{ctx}: planned family cannot contain formal rules, pins, or prose")
        if rec["state"] in {"coverage-ready", "formalized", "prose-landed"}:
            missing = [
                family for family in families if family not in completed
            ]
            if missing:
                raise LedgerError(
                    f"{ctx}: family is coverage-ready before cards: {missing}")
        if rec["state"] in {"formalized", "prose-landed"}:
            if not formal_refs or not rec["pin_group_refs"] or not rec["counterfactual_refs"]:
                raise LedgerError(
                    f"{ctx}: formalized family requires statements, pins, and counterfactuals")
            for record_ref, array_name in (
                    *((ref, "powers") for ref in rec["card_refs"]),
                    *((ref, "constitutional_effects") for ref in rec["effect_refs"])):
                record = next(row for row in src[array_name]
                              if row["id"] == record_ref)
                for key in ("negative_test", "counterfactual"):
                    if record[key]["status"] != "executable":
                        raise LedgerError(
                            f"{ctx}: formalized coverage requires executable {key}")
                expected_part_v = (
                    "prose-landed" if rec["state"] == "prose-landed"
                    else "formalized-not-prose-landed")
                if array_name == "constitutional_effects" and record["part_v_status"] != expected_part_v:
                    raise LedgerError(
                        f"{ctx}: effect Part V status must match family state")
        elif formal_refs or rec["pin_group_refs"] or rec["prose_refs"]:
            raise LedgerError(
                f"{ctx}: formal rules, pins, or prose precede formalization")
        if rec["state"] == "prose-landed":
            if not rec["prose_refs"] or not rec["part_v_refs"]:
                raise LedgerError(
                    f"{ctx}: prose-landed needs chapter and Part V references")
        elif rec["part_v_refs"]:
            raise LedgerError(f"{ctx}: Part V prose precedes formalization")
        assigned_statements.extend(formal_refs)
        assigned_cards.extend(rec["card_refs"])
        assigned_templates.extend(rec["template_refs"])
        assigned_refusals.extend(rec["refusal_refs"])
        assigned_crosswalks.extend(rec["crosswalk_refs"])
        assigned_effects.extend(rec["effect_refs"])
        statement_policy = {
            "FS-CVF-003": (STATE_FORM_STATEMENT_IDS, "state-form"),
            "FS-CVF-011": (UNIVERSAL_STANDING_STATEMENT_IDS, "universal-standing"),
            "FS-CVF-012": (LIBERTY_ECOLOGY_STATEMENT_IDS, "liberty-and-ecology"),
            "FS-CVF-013": (EQUALITY_STATEMENT_IDS, "substantive-equality"),
            "FS-CVF-014": (FAMILY_LIFE_STATEMENT_IDS, "family-and-life-course"),
            "FS-CVF-015": (DELIVERY_STATEMENT_IDS, "delivery-and-receipt"),
            "FS-CVF-016": (OBLIGATION_STATEMENT_IDS,
                           "obligations-and-nonreciprocity"),
        }
        if rec["id"] in statement_policy:
            owned_ids,label=statement_policy[rec["id"]]
            expected=list(owned_ids) if rec["state"] in {"formalized","prose-landed"} else []
            if rec["formal_statement_refs"] != expected:
                raise LedgerError(f"{ctx}: {label} formal surface drifted")
    if len(assigned_statements) != len(set(assigned_statements)):
        raise LedgerError("a formal statement belongs to multiple coverage families")
    if (set(assigned_statements) != set(statement_ids)
            or len(assigned_statements) != len(statement_ids)):
        missing = sorted(set(statement_ids) - set(assigned_statements))
        extra = sorted(set(assigned_statements) - set(statement_ids))
        raise LedgerError(
            f"formal statement assignment is not exact; missing={missing[:3]} "
            f"extra={extra[:3]}")
    manifest_family = {
        row["provisional_key"]: row["source_family"]
        for row in _power_manifest_rows()
    }
    def converted_records(records):
        return [
            row["id"] for row in records
            if manifest_family[row["manifest_key"]] in completed
        ]
    for values, expected, name in (
            (assigned_cards, converted_records(src["powers"]), "cards"),
            (assigned_templates,
             converted_records(src["power_contract_templates"]), "templates"),
            (assigned_refusals,
             converted_records(src["power_refusals"]), "refusals"),
            (assigned_crosswalks,
             converted_records(src["power_crosswalk_dispositions"]),
             "crosswalks"),
            (assigned_effects, [row["id"] for row in src["constitutional_effects"]],
             "constitutional effects")):
        if sorted(values) != sorted(expected) or len(values) != len(set(values)):
            raise LedgerError(
                f"coverage families must partition current {name} exactly once")


def validate_state_form_coverage_coupling(src: dict):
    family = next(
        (row for row in src["coverage_families"]
         if row["id"] == "FS-CVF-003"),
        None,
    )
    if family is None:
        raise LedgerError("FS-CVF-003 state-form coverage family is missing")
    expected_card_refs = [f"FS-POW-{number:03d}" for number in range(1, 52)]
    if family["card_refs"] != expected_card_refs:
        raise LedgerError(
            "FS-CVF-003 must own FS-POW-001..051 in exact order")
    powers_by_id = {row["id"]: row for row in src["powers"]}
    formalized = family["state"] in {"formalized", "prose-landed"}
    prose_landed = family["state"] == "prose-landed"
    expected_pin_refs = [STATE_FORM_PIN_GROUP_REF] if formalized else []
    expected_counterfactual_refs = (
        [STATE_FORM_COUNTERFACTUAL_REF] if formalized else [])
    expected_prose_refs = list(STATE_FORM_PROSE_REFS) if prose_landed else []
    expected_part_v_refs = list(STATE_FORM_PART_V_REFS) if prose_landed else []
    for field, expected in (
            ("pin_group_refs", expected_pin_refs),
            ("counterfactual_refs", expected_counterfactual_refs),
            ("prose_refs", expected_prose_refs),
            ("part_v_refs", expected_part_v_refs)):
        if family[field] != expected:
            raise LedgerError(
                f"FS-CVF-003 {field} must match the checker-owned references")
    if prose_landed and (
            family["blocked_before_drafting"]
            != STATE_FORM_PROSE_DISPOSITION):
        raise LedgerError(
            "FS-CVF-003 prose disposition must preserve the evidence ceiling")
    expected_test_status = "executable" if formalized else "planned"
    expected_part_v_status = (
        "prose-landed"
        if prose_landed
        else (
            "formalized-not-prose-landed"
            if formalized
            else "coverage-only-not-formalized"
        )
    )
    for power_ref in family["card_refs"]:
        power = powers_by_id[power_ref]
        if (
                power["status"] != "ratified-unimplemented"
                or power["posture"] != "Specified"):
            raise LedgerError(
                f"{power_ref}: formal coverage cannot claim operation")
        number = int(power_ref.rsplit("-", 1)[1])
        expected_tests = {
            "negative_test": {
                "assertion": (
                    "Fuse the state-form source writer with the temporal "
                    f"reviewer for {power['title']}; the current record and "
                    "its direct effect must remain unavailable."
                ),
                "executable_ref": (
                    "new-book-plans/state-form.pins.nibli::"
                    f"# FS-POW-{number:03d} negative: fused source/current "
                    "reviewer cannot derive authority."
                ) if formalized else None,
            },
            "counterfactual": {
                "assertion": (
                    "Remove the source-writer/temporal-reviewer disequality "
                    f"for {power['title']}; the fused-role record derives the "
                    "bounded authority that the current constitution "
                    "withholds."
                ),
                "executable_ref": (
                    "new-book-plans/counterfactual/"
                    "no-state-form-independent-current-review.pins.nibli::"
                    f"# FS-POW-{number:03d} counterfactual: removing the "
                    "independent-current-review guard derives authority."
                ) if formalized else None,
            },
        }
        for field, expected in expected_tests.items():
            if power[field]["status"] != expected_test_status:
                raise LedgerError(
                    f"{power_ref}: {field} status must follow FS-CVF-003 state")
            if formalized:
                for key in ("assertion", "executable_ref"):
                    if power[field][key] != expected[key]:
                        raise LedgerError(
                            f"{power_ref}: {field}.{key} must match the "
                            "checker-owned state-form anchor")
            elif power[field]["executable_ref"] is not None:
                raise LedgerError(
                    f"{power_ref}: planned {field} cannot cite an executable")
        if power["part_v_status"] != expected_part_v_status:
            raise LedgerError(
                f"{power_ref}: Part V status must follow FS-CVF-003 state")


def _power_claim_refs(row: dict):
    key = row["provisional_key"]
    family = row["source_family"]
    refs = []
    def add(*values):
        for value in values:
            if value not in refs:
                refs.append(value)
    if key == RETAINED_FORMAL_KEY:
        add("FS-CLM-11", "FS-CLM-17")
    elif family == "state-form-and-political-membership":
        if any(t in key for t in (
                "competence", "regional", "subsidiarity", "political-home",
                "secession", "local-", "portability", "membership")):
            add("FS-CLM-26")
        if any(t in key for t in (
                "assembly", "election", "elector", "council", "confidence",
                "president", "amendment", "initiative", "recall", "budget",
                "revenue", "spending", "secession", "promulgation")):
            add("FS-CLM-15")
        if any(t in key for t in (
                "court", "relief", "invalidation", "panel", "appointment",
                "qualification", "executive", "oversight", "office",
                "caretaker", "successor", "administration", "bill-return")):
            add("FS-CLM-18")
        if not refs:
            add("FS-CLM-18")
    elif family == "substantive-equality-and-anti-subordination":
        add("FS-CLM-02")
        if any(t in key for t in ("diagnostic", "systemic", "determination")):
            add("FS-CLM-08")
    elif family == "economic-pluralism-and-protected-private-sphere":
        add("FS-CLM-23")
        if any(t in key for t in (
                "licence", "credit", "insurance", "private", "occupational")):
            add("FS-CLM-35")
    elif family == "family-dependency-reproduction-and-collective-plurality":
        add("FS-CLM-09")
        if "adulthood" in key:
            add("FS-CLM-10")
        if key.startswith("collective-"):
            add("FS-CLM-27")
        if "recognition-module" in key:
            add("FS-CLM-22")
    elif family == "ecological-commons-and-non-human-animal":
        add("FS-CLM-34" if key.startswith("animal-") else "FS-CLM-33")
    elif family == "public-safety-defence-emergency-and-external-power":
        if key.startswith("emergency-"):
            add("FS-CLM-28")
        if any(t in key for t in (
                "protect-", "coercive-", "restriction-", "policing",
                "custod", "arrest", "detention", "search", "seizure",
                "force", "prosecution", "adjudication")):
            add("FS-CLM-29")
        if key.startswith("defence-") or key == "protect-defence-mandate":
            add("FS-CLM-30")
        if "intelligence" in key or "surveillance" in key:
            add("FS-CLM-31")
        if key.startswith("border-") or any(t in key for t in (
                "expulsion", "extradition", "transfer")):
            add("FS-CLM-32")
        if key.startswith("external-"):
            add("FS-CLM-30")
        if not refs:
            add("FS-CLM-29")
    if not refs:
        raise LedgerError(f"no direct-effect claim mapping for {key}")
    return refs


def _domains_for_claim_refs(src: dict, claim_refs: list):
    claims = {row["id"]: row for row in src["claims"]}
    result = []
    for claim_ref in claim_refs:
        for domain_ref in claims[claim_ref]["domain_refs"]:
            if domain_ref not in result:
                result.append(domain_ref)
    return result


def _validate_contract_term(term, field, card, context):
    if not isinstance(term, dict):
        raise LedgerError(f"{context} must be an object")
    basis = term.get("basis")
    keys = ["text", "basis", "source_refs"]
    if basis == "bounded-delegation":
        keys += ["choice_owner", "bounds", "failure_default"]
    exact_keys(term, keys, context)
    text = require_str(term, "text", context)
    lower = text.lower()
    blank_tokens = {"n/a", "na", "tbd", "unknown", "unresolved"}
    if (lower.strip() in blank_tokens
            or re.search(r"\b(?:tbd|unresolved)\b", lower)
            or "is fixed by the source-bound" in lower):
        raise LedgerError(f"{context}: unresolved or legacy generic prose")
    if text.strip() == card["applicability"].strip():
        raise LedgerError(f"{context}: applicability duplication is not a term")
    label = field.replace("_", " ").lower()
    if lower.startswith(label + " ") or lower.startswith(label + ":"):
        raise LedgerError(f"{context}: field-name echo is not a contract term")
    if basis not in TERM_BASES:
        raise LedgerError(f"{context}.basis is invalid")
    _validate_source_refs(term["source_refs"], f"{context}.source_refs")
    if not set(term["source_refs"]).issubset(set(card["source_refs"])):
        raise LedgerError(f"{context}: term source must be a card source")
    if basis == "bounded-delegation":
        for key in ("choice_owner", "bounds", "failure_default"):
            value = require_str(term, key, context)
            if value.strip().lower() in {"n/a", "tbd", "unknown", "unresolved"}:
                raise LedgerError(f"{context}: delegation must be decision-complete")
    return text


def _validate_card_test(value, kind, card, context):
    if not isinstance(value, dict):
        raise LedgerError(f"{context} must be an object")
    exact_keys(value,
               ["id", "status", "assertion", "source_refs", "executable_ref"],
               context)
    if value["id"] != f"{card['id']}-{kind.upper()}":
        raise LedgerError(f"{context}.id must be card-derived")
    if value["status"] not in {"planned", "executable"}:
        raise LedgerError(f"{context}.status must be planned or executable")
    require_str(value, "assertion", context)
    _validate_source_refs(value["source_refs"], f"{context}.source_refs")
    if not set(value["source_refs"]).issubset(set(card["source_refs"])):
        raise LedgerError(f"{context}: test source must be a card source")
    if value["status"] == "executable":
        validate_reference(require_str(value, "executable_ref", context),
                           f"{context}.executable_ref")
    elif value["executable_ref"] is not None:
        raise LedgerError(f"{context}: planned tests cannot claim execution")


def validate_constitutional_effects(src: dict, ids: dict):
    rows = src.get("constitutional_effects")
    if not isinstance(rows, list) or len(rows) != CONSTITUTIONAL_EFFECT_COUNT:
        raise LedgerError("constitutional_effects must contain the checker-owned effect population")
    if [row.get("effect_key") for row in rows] != list(EFFECT_POLICY):
        raise LedgerError("constitutional_effects must follow checker-owned effect order")
    constitution = _input_text(
        pathlib.Path("new-book-plans/constitution.nibli"))
    actual_floor=tuple(line.strip() for line in constitution.splitlines() if line.strip().startswith("entitled(every person, event {"))
    if actual_floor != FLOOR_ENTITLEMENT_LINES:
        raise LedgerError("the eight-item material-floor inventory changed or gained a ninth item")
    seen_text = set()
    for index, rec in enumerate(rows):
        ctx = f"constitutional_effects[{index}] ({rec.get('id', '?')})"
        exact_keys(rec, COMMON_KEYS + ["effect_key", "posture", "primary_class_ref", "secondary_class_refs", "profiles", "affected_claim_refs", "domain_refs", "holder_role_refs", "affected_role_refs", "checking_role_refs", "permitted_inputs", "prohibited_inputs", "permitted_downstream_effects", "contract_terms", "profile_terms", "evidence_authority", "negative_test", "counterfactual", "part_v_status", "book2_handoff", "source_refs"], ctx)
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != "constitutional-invariant" or rec["posture"] not in {"Specified", "Derived"}:
            raise LedgerError(f"{ctx}: effect layer or posture is invalid")
        primary, secondary, profiles, claim_refs = EFFECT_POLICY[rec["effect_key"]]
        if (rec["primary_class_ref"], rec["secondary_class_refs"], rec["profiles"], rec["affected_claim_refs"]) != (primary, secondary, profiles, claim_refs):
            raise LedgerError(f"{ctx}: checker-owned classification drifted")
        if rec["domain_refs"] != _domains_for_claim_refs(src, claim_refs):
            raise LedgerError(f"{ctx}: domains must derive from affected claims")
        for field, array_name in (("affected_claim_refs", "claims"), ("domain_refs", "domains"), ("holder_role_refs", "roles"), ("affected_role_refs", "roles"), ("checking_role_refs", "roles")):
            _typed_ref_list(rec[field], array_name, ids, f"{ctx}.{field}")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        for field in ("permitted_inputs", "prohibited_inputs", "permitted_downstream_effects"):
            values = rec[field]
            if not isinstance(values, list) or not values or len(values) != len(set(values)) or any(not isinstance(value, str) or not value.strip() for value in values):
                raise LedgerError(f"{ctx}.{field} must be non-empty and duplicate-free")
        exact_keys(rec["contract_terms"], EFFECT_CONTRACT_TERM_KEYS, f"{ctx}.contract_terms")
        for field in EFFECT_CONTRACT_TERM_KEYS:
            text = _validate_contract_term(rec["contract_terms"][field], field, rec, f"{ctx}.contract_terms.{field}")
            if text in seen_text:
                raise LedgerError(f"{ctx}: repeated generic effect-contract prose")
            seen_text.add(text)
        if not isinstance(rec["profile_terms"], dict) or list(rec["profile_terms"]) != profiles:
            raise LedgerError(f"{ctx}: profile_terms must match applicable profiles")
        for profile in profiles:
            exact_keys(rec["profile_terms"][profile], EFFECT_PROFILE_FIELDS[profile], f"{ctx}.profile_terms.{profile}")
            for field in EFFECT_PROFILE_FIELDS[profile]:
                text = _validate_contract_term(rec["profile_terms"][profile][field], field, rec, f"{ctx}.profile_terms.{profile}.{field}")
                if text in seen_text:
                    raise LedgerError(f"{ctx}: repeated generic effect-profile prose")
                seen_text.add(text)
        _validate_contract_term(rec["evidence_authority"], "evidence_authority", rec, f"{ctx}.evidence_authority")
        _validate_card_test(rec["negative_test"], "negative", rec, f"{ctx}.negative_test")
        _validate_card_test(rec["counterfactual"], "counterfactual", rec, f"{ctx}.counterfactual")
        if rec["part_v_status"] not in {"coverage-only-not-formalized", "formalized-not-prose-landed", "prose-landed"}:
            raise LedgerError(f"{ctx}: Part V status is invalid")
        prohibited=" ".join(rec["prohibited_inputs"]).lower(); key=rec["effect_key"]
        if key in set(list(EFFECT_POLICY)[:8]): required=("registry","citizenship","capacity","documentation","t3","encounter handle","civil identity")
        elif key in MATERIAL_FLOOR_EFFECT_KEYS: required=("ninth floor","environmental right","t3")
        elif key in LIBERTY_EFFECT_KEYS: required=("material floor","service delivery","t3")
        elif key in ENVIRONMENTAL_EFFECT_KEYS: required=("material floor","environmental worth score","t3")
        elif key in CLASS9_EFFECT_KEYS: required=("material floor","single scalar","t3")
        elif key in EQUALITY_EFFECT_KEYS: required=("person worth","risk score","entitlement score","criminal burden","bare record absence","t3")
        elif key in FAMILY_LIFE_EFFECT_KEYS: required=("omnibus family","mature predicate","missing kinship","recognition","confinement","t3")
        elif key in DELIVERY_EFFECT_KEYS: required=("provider self-certification","personal state","roster entry","confinement","t3")
        elif key == "all-entitlement-nonreciprocity": required=(
            "work", "payment", "identity", "compliance", "contribution",
            "score", "approval", "role duty", "civic duty", "private duty",
            "voluntary duty", "personhood", "entitlement or public debt",
            "political voice", "evidence and challenge", "continuity",
            "remedy", "restrictive power", "bare obliged", "t3",
        )
        elif key in OBLIGATION_FINDING_EFFECT_KEYS: required=(
            "wrong recipient", "silent nonresponse",
            "systemic work delays individual relief", "bare obliged", "t3",
        )
        elif key in OBLIGATION_EFFECT_KEYS: required=(
            "duty class as priority", "subject matter",
            "self-certified excuse", "bare obliged", "t3",
        )
        else: raise LedgerError(f"{ctx}: missing effect boundary policy")
        for token in required:
            if token not in prohibited: raise LedgerError(f"{ctx}: prohibited inputs omit {token}")
        if key in OBLIGATION_EFFECT_KEYS:
            role_refs = set(rec["holder_role_refs"] + rec["affected_role_refs"]
                            + rec["checking_role_refs"])
            permitted_roles = {"FS-ROL-01", "FS-ROL-34", "FS-ROL-35",
                               "FS-ROL-40"}
            if key == "class9-common-cessation-restoration":
                permitted_roles.add("FS-ROL-49")
                if "FS-ROL-49" not in rec["checking_role_refs"]:
                    raise LedgerError(
                        f"{ctx}: Class 9 evidence must name the ecological scientist")
            if not role_refs.issubset(permitted_roles):
                raise LedgerError(
                    f"{ctx}: obligations must reuse protected-person, claimant, "
                    "victim, and record-subject roles without an institutional proxy")
            if (key != "class9-common-cessation-restoration"
                    and "FS-ROL-49" in role_refs):
                raise LedgerError(
                    f"{ctx}: ecological scientist is restricted to FS-CCE-207")
        if "no operation" not in require_str(rec, "book2_handoff", ctx).lower():
            raise LedgerError(f"{ctx}: Book 2 handoff must refuse operation")


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
    coverage_completed = src["coverage_population"]["completed_source_families"]
    powers_by_manifest = {}
    term_texts = set()
    for i, rec in enumerate(arrays["powers"]):
        ctx = f"powers[{i}] ({rec.get('id', '?')})"
        row = manifest.get(rec["manifest_key"])
        if row is None:
            raise LedgerError(f"{ctx}: unknown manifest_key")
        converted = row["source_family"] in coverage_completed
        exact_keys(
            rec,
            COMMON_KEYS + (
                CARD_V7_EXTRA_KEYS if converted else CARD_LEGACY_EXTRA_KEYS
            ),
            ctx,
        )
        validate_common_record_fields(rec, ctx)
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
        profiles = _power_profiles(row) if converted else rec["profiles"]
        if converted and rec["profiles"] != profiles:
            raise LedgerError(f"{ctx}: profiles differ from checker-owned classification")
        if (not isinstance(profiles, list) or not profiles
                or any(profile not in POWER_PROFILE_ORDER for profile in profiles)):
            raise LedgerError(f"{ctx}: profiles are invalid")
        expected_pairs = _power_required_separations(profiles)
        if converted:
            primary = _power_primary_class(row)
            if rec["primary_class_ref"] != primary:
                raise LedgerError(f"{ctx}: primary class differs from direct effect")
            secondary = _power_secondary_classes(row, profiles)
            if rec["secondary_class_refs"] != secondary:
                raise LedgerError(f"{ctx}: secondary classes differ from profiles")
            if rec["affected_claim_refs"] != _power_claim_refs(row):
                raise LedgerError(f"{ctx}: claims differ from direct legal effect")
            expected_domains = _domains_for_claim_refs(
                src, rec["affected_claim_refs"])
            if rec["domain_refs"] != expected_domains:
                raise LedgerError(
                    f"{ctx}: domains must be the ordered union of claim domains")
            contract = rec["contract_terms"]
            if not isinstance(contract, dict):
                raise LedgerError(f"{ctx}.contract_terms must be an object")
            exact_keys(contract, POWER_CONTRACT_TERM_KEYS,
                       f"{ctx}.contract_terms")
            for field in POWER_CONTRACT_TERM_KEYS:
                text = _validate_contract_term(
                    contract[field], field, rec,
                    f"{ctx}.contract_terms.{field}")
                if text in term_texts:
                    raise LedgerError(
                        f"{ctx}: repeated generic contract prose is prohibited")
                term_texts.add(text)
            blocks = rec["profile_terms"]
            if not isinstance(blocks, dict) or list(blocks) != profiles:
                raise LedgerError(
                    f"{ctx}: profile_terms must match exact applicable profiles")
            for profile in profiles:
                block = blocks[profile]
                exact_keys(block, POWER_PROFILE_FIELDS[profile],
                           f"{ctx}.profile_terms.{profile}")
                for field in POWER_PROFILE_FIELDS[profile]:
                    text = _validate_contract_term(
                        block[field], field, rec,
                        f"{ctx}.profile_terms.{profile}.{field}")
                    if text in term_texts:
                        raise LedgerError(
                            f"{ctx}: repeated generic profile prose is prohibited")
                    term_texts.add(text)
            if rec["required_separation_pairs"] != expected_pairs:
                raise LedgerError(f"{ctx}: required separation pairs are incomplete")
            for field in (
                    "permitted_inputs", "prohibited_inputs",
                    "permitted_downstream_effects"):
                values = rec[field]
                if (not isinstance(values, list) or not values
                        or len(values) != len(set(values))
                        or any(not isinstance(v, str) or not v.strip()
                               for v in values)):
                    raise LedgerError(
                        f"{ctx}.{field} must be non-empty and duplicate-free")
            if (not retained and not any(
                    "formal-active-custody" in value
                    for value in rec["prohibited_inputs"])):
                raise LedgerError(
                    f"{ctx}: every other power must prohibit T3 borrowing")
            _validate_contract_term(
                rec["evidence_authority"], "evidence_authority", rec,
                f"{ctx}.evidence_authority")
            _validate_card_test(
                rec["negative_test"], "negative", rec,
                f"{ctx}.negative_test")
            _validate_card_test(
                rec["counterfactual"], "counterfactual", rec,
                f"{ctx}.counterfactual")
            if retained:
                expected_part_v = "implemented-current-formal"
            elif (
                    rec["source_family"]
                    == "state-form-and-political-membership"):
                state_form_family = next(
                    row for row in src["coverage_families"]
                    if row["id"] == "FS-CVF-003")
                expected_part_v = {
                    "planned": "coverage-only-not-formalized",
                    "coverage-ready": "coverage-only-not-formalized",
                    "formalized": "formalized-not-prose-landed",
                    "prose-landed": "prose-landed",
                }[state_form_family["state"]]
            else:
                expected_part_v = "coverage-only-not-formalized"
            if rec["part_v_status"] != expected_part_v:
                raise LedgerError(f"{ctx}: Part V status overclaims formalization")
            handoff = require_str(rec, "book2_handoff", ctx)
            if "no operation" not in handoff.lower():
                raise LedgerError(f"{ctx}: Book 2 handoff must refuse operation")
        else:
            contracts = rec["profile_contracts"]
            if not isinstance(contracts, dict) or list(contracts) != profiles:
                raise LedgerError(
                    f"{ctx}: legacy profile_contracts must match profiles")
            for profile in profiles:
                block = contracts[profile]
                exact_keys(block, POWER_PROFILE_FIELDS[profile],
                           f"{ctx}.profile_contracts.{profile}")
                for field in POWER_PROFILE_FIELDS[profile]:
                    value = require_str(
                        block, field, f"{ctx}.profile_contracts.{profile}")
                    if value.strip().lower() in {
                            "n/a", "na", "tbd", "unknown", "unresolved"}:
                        raise LedgerError(
                            f"{ctx}: blank substitute in legacy profile contract")
            contract = rec["contract"]
            exact_keys(contract, LEGACY_POWER_CONTRACT_KEYS,
                       f"{ctx}.contract")
            for field in LEGACY_POWER_CONTRACT_KEYS[:-1]:
                value = require_str(contract, field, f"{ctx}.contract")
                if value.strip().lower() in {
                        "n/a", "na", "tbd", "unknown", "unresolved"}:
                    raise LedgerError(
                        f"{ctx}: blank substitute in legacy contract")
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
        row = manifest[rec["manifest_key"]]
        converted = row["source_family"] in coverage_completed
        contract_key = "contract_terms" if converted else "contract"
        exact_keys(rec, COMMON_KEYS + ["manifest_key", contract_key, "source_refs"], ctx)
        validate_common_record_fields(rec, ctx)
        if rec["manifest_key"] != POWER_TEMPLATE_KEY:
            raise LedgerError(f"{ctx}: only the checker-bound time template is allowed")
        if rec["layer"] != "constitutional-invariant" or rec["status"] != "ratified-template":
            raise LedgerError(f"{ctx}: template layer/status mismatch")
        contract = rec[contract_key]
        exact_keys(contract, POWER_TEMPLATE_CONTRACT_KEYS,
                   f"{ctx}.{contract_key}")
        for field in POWER_TEMPLATE_CONTRACT_KEYS:
            if converted:
                _validate_contract_term(
                    contract[field], field, rec,
                    f"{ctx}.{contract_key}.{field}")
            else:
                require_str(contract, field, f"{ctx}.{contract_key}")
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
        pairs = (
            power["required_separation_pairs"]
            if "required_separation_pairs" in power
            else power["contract"]["required_separation_pairs"]
        )
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


def validate_state_form_semantic_mappings(src: dict):
    powers = {row["id"]: row for row in src["powers"]}
    allocations = {
        row["power_ref"]: row for row in src["function_allocations"]
    }
    fields = ("holder", *POWER_FUNCTIONS)
    for power_ref, expected_rows in STATE_FORM_SEMANTIC_MAPPINGS.items():
        power = powers[power_ref]
        allocation = allocations[power_ref]
        for field, (expected_bodies, expected_roles) in zip(
                fields, expected_rows):
            if field == "holder":
                body_refs = power["holder_body_refs"]
                role_refs = power["holder_role_refs"]
            else:
                stem = field.replace("-", "_")
                body_refs = allocation[f"{stem}_body_refs"]
                role_refs = allocation[f"{stem}_role_refs"]
            if (body_refs != list(expected_bodies)
                    or role_refs != list(expected_roles)):
                raise LedgerError(
                    f"{power_ref}: checker-owned {field} mapping drifted")
    for power_ref in STATE_FORM_DELEGATED_CONFIG_POWER_IDS:
        text = powers[power_ref]["contract_terms"]["lawful_source"]["text"]
        for needle in (
                "holder body and role arrays name only eligible constitutional "
                "participants or subjects",
                "exact current source-supplied"):
            if needle not in text:
                raise LedgerError(
                    f"{power_ref}: supplied-configuration boundary drifted")


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
            temporary_pending_audit = (
                array == "scope_audits"
                and isinstance(rid, str)
                and re.fullmatch(r"FS-SAU-[0-9]+-PENDING", rid)
                and rec.get("source_version")
                != LEGACY_V1_CLOSURE_SOURCE
                and rec.get("result") == "pending"
            )
            if (not isinstance(rid, str)
                    or not ID_RE.match(rid) and not temporary_pending_audit):
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


def _body_register_guard(text: str, context: str):
    """A body card routes and states duties; it never reports an arrival.

    Three registers are refused together because each converts a specification
    into a claim it cannot support: an arrival verb turns an owed duty into a
    delivered one, an aggregate figure turns a routing inventory into a score
    the ledger refuses to compute, and a feasibility word makes a Book 2 claim
    inside Book 1, where no route can establish it.
    """
    lower = text.lower()
    for phrase in BODY_ARRIVAL_PHRASES:
        if phrase in lower:
            raise LedgerError(
                f"{context}: arrival register is refused — {phrase!r} states "
                "that something happened; a card states what is owed and what "
                "withholds a conclusion"
            )
    if BODY_AGGREGATE_RE.search(text):
        raise LedgerError(
            f"{context}: aggregate figure is refused — the ledger produces no "
            "score, total, percentage, or coverage figure"
        )
    for token in BODY_FEASIBILITY_TOKENS:
        if token in lower:
            raise LedgerError(
                f"{context}: feasibility claim is refused — {token!r} belongs "
                "to Book 2; no Book 1 route can establish it"
            )
    for relabel in BODY_FIXTURE_RELABELS:
        if relabel in lower:
            raise LedgerError(
                f"{context}: may not relabel a current fixture as a ratified "
                "institution — the ruling refuses this equation by name"
            )


def _body_term(rec, value, field, ctx):
    text = _validate_contract_term(value, field, rec, ctx)
    _body_register_guard(text, ctx)
    return text


BODY_MAP_CELLS = ["job", "may_not_do_alone", "required_check"]


def _coverage_map_body_rows() -> dict:
    """The reviewed required-bodies table, keyed by normalised title.

    Section 5 is hand-authored and sits outside the generated region, so it is
    the source these three cells are copied from — which is exactly why they
    can drift from it silently.  The map writes `Provision / treasury` where
    the ledger writes `Provision and treasury`; that difference is deliberate
    and normalised away, nothing else is.
    """
    text = _input_text(COVERAGE_MAP)
    if "## 5. Required bodies" not in text:
        raise LedgerError(
            "coverage map has no required-bodies section to bind the body "
            "cells to"
        )
    section = text.split("## 5. Required bodies", 1)[1].split("\n## 6.", 1)[0]
    rows = {}
    for line in section.splitlines():
        if not line.startswith("| ") or line.startswith(("| ---", "| Body")):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
        if len(cells) != len(BODY_MAP_CELLS) + 1:
            continue
        rows[_normalise_body_title(cells[0])] = cells[1:]
    return rows


def _normalise_body_title(title: str) -> str:
    return title.replace(" / ", " and ").replace("/", " and ").strip().lower()


def validate_body_map_cells(src: dict):
    """A body's three reviewed cells are a COPY of the coverage map's row.

    They were copied once and then paraphrased in place, and the paraphrases
    lost ratified content: the Civic President's removal shed `and cross-body
    confirmation`, leaving one half of a two-stage requirement; the armed
    forces shed `armed, or maintained`; the Council's return shed `one-time`,
    which is the whole difference between a suspensive return and a veto.
    Every card cites that section as its source, so a drifted copy is a card
    misquoting the source it names.  Bodies added after the section was
    written have no row and are skipped, but the row population must be fully
    consumed — a parser that matches nothing must fail, not pass quietly.
    """
    rows = _coverage_map_body_rows()
    if not rows:
        raise LedgerError("required-bodies table parsed to no rows")
    unmatched = set(rows)
    for i, rec in enumerate(src.get("bodies", [])):
        row = rows.get(_normalise_body_title(rec.get("title", "")))
        if row is None:
            continue
        unmatched.discard(_normalise_body_title(rec["title"]))
        for cell, expected in zip(BODY_MAP_CELLS, row):
            if rec[cell].strip() != expected.strip():
                raise LedgerError(
                    f"bodies[{i}] ({rec['id']}).{cell} has drifted from the "
                    "coverage map row it cites; the card must quote its "
                    f"source, not paraphrase it\n  card: {rec[cell]}\n  "
                    f"map:  {expected}"
                )
    if unmatched:
        raise LedgerError(
            "every required-bodies row must bind a body card; unbound: "
            + ", ".join(sorted(unmatched))
        )


def validate_bodies(src: dict, ids: dict):
    """The bodies specification: one constitutional contract per body.

    The card says what a body ordinarily does, what it may not do alone, who
    checks it, which individualized adverse determinations it can make, and on
    what its authority depends. It does not say that the body exists, is
    staffed, or has ever acted: every record stays `ratified-unimplemented`,
    and the affected claim keeps its `Specified` posture. Appeal attaches to a
    NAMED determination, never to the body as such — that is the mechanical
    form of the ruling's refusal to recreate a universal right of appeal.
    """
    for i, rec in enumerate(src.get("bodies", [])):
        ctx = f"bodies[{i}] ({rec.get('id', '?')})"
        exact_keys(
            rec,
            COMMON_KEYS + ["job", "may_not_do_alone", "required_check",
                           "source_ref", "source_refs", "body_kind",
                           "status_senses", "office_contract",
                           "accountability_routes", "adverse_determinations",
                           "temporal_contract", "delegated_mechanics",
                           "book2_handoff"],
            ctx,
        )
        validate_common_record_fields(rec, ctx)
        if rec["layer"] != "constitutional-invariant":
            raise LedgerError(
                f"{ctx}: a required body is constitutional architecture; its "
                "layer is constitutional-invariant"
            )
        for key in ("job", "may_not_do_alone", "required_check",
                    "book2_handoff"):
            require_str(rec, key, ctx)
        validate_reference(rec["source_ref"], f"{ctx}.source_ref")
        _validate_source_refs(rec["source_refs"], f"{ctx}.source_refs")
        if rec["source_ref"] not in rec["source_refs"]:
            raise LedgerError(
                f"{ctx}: card sources must contain the rendered source"
            )
        if rec["body_kind"] not in BODY_KINDS:
            raise LedgerError(
                f"{ctx}: body_kind is invalid; declared kinds are "
                f"{', '.join(BODY_KINDS)}"
            )

        senses = rec["status_senses"]
        if not isinstance(senses, dict):
            raise LedgerError(f"{ctx}.status_senses must be an object")
        exact_keys(senses, BODY_STATUS_SENSES, f"{ctx}.status_senses")
        for sense in BODY_STATUS_SENSES:
            sctx = f"{ctx}.status_senses.{sense}"
            text = _body_term(rec, senses[sense], sense, sctx)
            if sense in BODY_OFFICE_SENSES and BODY_STANDING_RE.search(text):
                raise LedgerError(
                    f"{sctx}: standing is reserved for universal personhood; "
                    "the office sense is public or historical answerability"
                )

        office = rec["office_contract"]
        if not isinstance(office, dict):
            raise LedgerError(f"{ctx}.office_contract must be an object")
        exact_keys(office, BODY_OFFICE_TERMS, f"{ctx}.office_contract")
        for term in BODY_OFFICE_TERMS:
            octx = f"{ctx}.office_contract.{term}"
            text = _body_term(rec, office[term], term, octx)
            if term == "ordinary_function" and text.strip() == rec["job"].strip():
                raise LedgerError(
                    f"{octx}: the ordinary function must expand the job, not "
                    "copy it — the check and the day job are both shown"
                )

        routes = rec["accountability_routes"]
        if not isinstance(routes, list) or not routes:
            raise LedgerError(
                f"{ctx}.accountability_routes must name at least one typed route"
            )
        seen_types, external_checker = set(), False
        for j, route in enumerate(routes):
            rctx = f"{ctx}.accountability_routes[{j}]"
            exact_keys(route, ["route_type", "checker_body_refs",
                               "checker_role_refs", "term"], rctx)
            if route["route_type"] not in ACCOUNTABILITY_ROUTE_TYPES:
                raise LedgerError(
                    f"{rctx}.route_type must be one of "
                    f"{', '.join(ACCOUNTABILITY_ROUTE_TYPES)}"
                )
            if route["route_type"] in seen_types:
                raise LedgerError(f"{rctx}: route types are duplicate-free")
            seen_types.add(route["route_type"])
            _typed_ref_list(route["checker_body_refs"], "bodies", ids,
                            f"{rctx}.checker_body_refs", allow_empty=True)
            _typed_ref_list(route["checker_role_refs"], "roles", ids,
                            f"{rctx}.checker_role_refs", allow_empty=True)
            if rec["id"] in route["checker_body_refs"]:
                raise LedgerError(
                    f"{rctx}: a body may not check itself — separation is the "
                    "whole of the required check"
                )
            if route["checker_body_refs"]:
                external_checker = True
            _body_term(rec, route["term"], route["route_type"], f"{rctx}.term")
        if not external_checker:
            raise LedgerError(
                f"{ctx}.accountability_routes needs at least one checking body"
            )

        adverse = rec["adverse_determinations"]
        if not isinstance(adverse, dict):
            raise LedgerError(f"{ctx}.adverse_determinations must be an object")
        exact_keys(adverse, ["kind", "note", "items"],
                   f"{ctx}.adverse_determinations")
        if adverse["kind"] not in ADVERSE_DETERMINATION_KINDS:
            raise LedgerError(
                f"{ctx}.adverse_determinations.kind must be "
                f"{' or '.join(ADVERSE_DETERMINATION_KINDS)}"
            )
        _body_term(rec, adverse["note"], "note",
                   f"{ctx}.adverse_determinations.note")
        items = adverse["items"]
        if not isinstance(items, list):
            raise LedgerError(f"{ctx}.adverse_determinations.items must be a list")
        if adverse["kind"] == "none-by-design" and items:
            raise LedgerError(
                f"{ctx}.adverse_determinations: none-by-design lists no items"
            )
        if adverse["kind"] == "enumerated" and not items:
            raise LedgerError(
                f"{ctx}.adverse_determinations: enumerated needs at least one item"
            )
        for j, item in enumerate(items):
            ictx = f"{ctx}.adverse_determinations.items[{j}]"
            exact_keys(item, ["name", "subject", "appeal", "remedy"], ictx)
            require_str(item, "name", ictx)
            require_str(item, "subject", ictx)
            _body_term(rec, item["appeal"], "appeal", f"{ictx}.appeal")
            _body_term(rec, item["remedy"], "remedy", f"{ictx}.remedy")

        temporal = rec["temporal_contract"]
        if not isinstance(temporal, dict):
            raise LedgerError(f"{ctx}.temporal_contract must be an object")
        exact_keys(temporal, ["contract_kind", "custody_t3_relation", "term",
                              "failure_polarity", "expiry_default"],
                   f"{ctx}.temporal_contract")
        require_str(temporal, "contract_kind", f"{ctx}.temporal_contract")
        if temporal["custody_t3_relation"] not in CUSTODY_T3_RELATIONS:
            raise LedgerError(
                f"{ctx}.temporal_contract.custody_t3_relation must be "
                f"{' or '.join(CUSTODY_T3_RELATIONS)}"
            )
        if (temporal["custody_t3_relation"] == "retained-application"
                and rec["id"] != CUSTODY_T3_APPLICANT):
            raise LedgerError(
                f"{ctx}.temporal_contract: custody T3 is not reusable — only "
                f"{CUSTODY_T3_APPLICANT} applies the retained record"
            )
        for key in ("term", "failure_polarity", "expiry_default"):
            tctx = f"{ctx}.temporal_contract.{key}"
            _body_term(rec, temporal[key], key, tctx)
            if rec["id"] == CUSTODY_T3_APPLICANT:
                continue
            for ref in temporal[key]["source_refs"]:
                if any(marker in ref for marker in CUSTODY_T3_SOURCE_MARKERS):
                    raise LedgerError(
                        f"{tctx}: custody T3 is not reusable as an office term "
                        "or election clock; this body needs its own source"
                    )

        mechanics = rec["delegated_mechanics"]
        if not isinstance(mechanics, list):
            raise LedgerError(f"{ctx}.delegated_mechanics must be a list")
        if rec["id"] in BODY_DELEGATED_MECHANICS_REQUIRED and not mechanics:
            raise LedgerError(
                f"{ctx}: a blocked mechanic must be filled — the ruling holds "
                "formal enactment until this card fixes it"
            )
        for j, entry in enumerate(mechanics):
            mctx = f"{ctx}.delegated_mechanics[{j}]"
            if not isinstance(entry, dict) or \
                    entry.get("basis") != "bounded-delegation":
                raise LedgerError(
                    f"{mctx}: a delegated mechanic is a bounded delegation, so "
                    "it declares its choice owner, bounds, and failure default"
                )
            _body_term(rec, entry, "delegated mechanic", mctx)


def validate_roles(src: dict, ids: dict):
    """The roles, life-course, scale, and power-position matrix.

    Three closures are mechanical: every domain cited by at least one role,
    every named scale exercised, and every required body carrying both an
    affected and a checking role position. Pairwise sufficiency (which
    role x scale x domain cells matter) is NOT mechanically established; the
    repository audit checks only the declared source-derived criteria. An entry in
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
    degradation are Book 2's tests, while the repository audit checks only
    declared structural coverage and watched failures."""
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
        if not isinstance(gate_refs, list):
            raise LedgerError(
                f"{ctx}: applicable_gate_refs must be a list"
            )
        expected_gates = expected_defect_gate_refs(rec["id"])
        if not gate_refs and expected_gates:
            raise LedgerError(
                f"{ctx}: applicable_gate_refs must be non-empty for this "
                "checker-owned gate-applicable defect"
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
        if rt not in set(DEFERRABLE_ARRAYS) | {COVERAGE_DEFERRAL_TYPE}:
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
        "| Domain | Historical frozen coverage | Ratified scope requirement | "
        "Current contract readiness | Historical gap / ruling record | "
        "Split claims (posture) | Direct-effect cards | "
        "Implementation and tests | Book 2 boundary |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for row in src["legacy_rows"]:
        splits = []
        for cid in row["split_claim_refs"]:
            claim = claims_by_id[cid]
            posture = claim["posture"]
            if posture == "Unestablished":
                posture += "/" + claim["unestablished_disposition"]
            splits.append(f"{cid} ({posture})")
        cards = [
            record for record in (
                list(src["powers"]) + list(src["constitutional_effects"]))
            if any(
                claim_ref in row["split_claim_refs"]
                for claim_ref in record["affected_claim_refs"]
            )
        ]
        ready = bool(cards) and all(
            "contract_terms" in record and "profile_terms" in record
            for record in cards
        )
        readiness = (
            "coverage-ready; not formalized or operational"
            if ready else
            "historical row has no direct-effect card in this claim split"
        )
        tests = "; ".join(
            f"{power['id']}: "
            f"{power['negative_test']['status']}/"
            f"{power['counterfactual']['status']}"
            for power in cards
        ) or "—"
        book2 = (
            "Every listed card carries a no-operation Book 2 handoff"
            if cards else
            "No card handoff in this historical row"
        )
        lines.append(
            "| " + " | ".join([
                row["domain_title"],
                "Historical: " + row["legacy_coverage"],
                row["legacy_scope_requirement"],
                readiness,
                "Historical: " + row["legacy_gap"],
                "; ".join(splits) or "—",
                "; ".join(power["id"] for power in cards) or "—",
                tests,
                book2,
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
    text = _input_text(map_path)
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


def _require_utc(value, context: str):
    if not isinstance(value, str) or not UTC_INSTANT_RE.fullmatch(value):
        raise LedgerError(
            f"{context} must be canonical UTC YYYY-MM-DDTHH:MM:SSZ")
    return value


def review_scope_digest(src: dict) -> str:
    """Digest only the semantic map and assurance program.

    Audit/review administration, deferrals, acceptance metadata, and closure
    are excluded. R7's audit-state fields are normalised because executing the
    checker changes route status, not the reviewed map.
    """
    excluded = {
        "review_protocol", "review_commissions", "proposals",
        "review_events", "scope_audits", "deferred_populations",
        "closure_record", "acceptance_gate",
    }
    payload = {k: copy.deepcopy(v) for k, v in src.items()
               if k not in excluded}
    for route in payload.get("routes", []):
        if route.get("id") == "FS-RTE-07":
            for key in ("status", "route_status", "consequence",
                        "closure_condition", "negative_control"):
                route.pop(key, None)
    return _canonical_digest(payload)


def _proposal_intake_digest(event_ref: str, proposals: list) -> str:
    fields = ("id", "title", "proposal", "source_kind", "source_identity",
              "received_at_utc", "review_event_ref")
    payload = {
        "review_event_ref": event_ref,
        "ordered_proposals": [
            {key: row[key] for key in fields} for row in proposals
        ],
    }
    return _canonical_digest(payload)


COMMISSION_KEYS = [
    "id", "title", "source_version", "scope_sha256", "protocol_sha256",
    "plant_commitment_sha256", "seed_commitment_sha256",
    "commissioned_at_utc", "received_window", "cutoff_at_utc",
    "custodian_identity", "reviewers", "criterion_coverage", "packet_paths",
]
REVIEWER_KEYS = [
    "identity", "discipline", "criterion_refs", "consent_attestation",
    "conflict_attestation", "compensation_attestation",
]


_PRIOR_REVIEW_STATE = None


def _prior_review_state():
    """Nearest visible first-parent review state.

    If the working source differs from HEAD, HEAD is the predecessor. On a
    clean tree, HEAD^ is the predecessor. This is a visible-Git-history
    append-only check, not resistance to rewritten history.
    """
    global _PRIOR_REVIEW_STATE
    if _PRIOR_REVIEW_STATE is not None:
        return _PRIOR_REVIEW_STATE
    changed = subprocess.run(
        ["git", "diff", "--quiet", "HEAD", "--", str(SOURCE)],
        cwd=ROOT, check=False,
    ).returncode
    revision = "HEAD" if changed == 1 else "HEAD^"
    proc = subprocess.run(
        ["git", "show", f"{revision}:{SOURCE.as_posix()}"],
        cwd=ROOT, check=False, capture_output=True, text=True,
    )
    if proc.returncode != 0:
        _PRIOR_REVIEW_STATE = {
            "review_commissions": [], "proposals": [], "review_events": [],
            "scope_audits": []}
    else:
        prior = json.loads(proc.stdout)
        _PRIOR_REVIEW_STATE = {
            key: prior.get(key, [])
            for key in ("review_commissions", "proposals", "review_events",
                        "scope_audits")
        }
    return _PRIOR_REVIEW_STATE


def validate_review_history(src: dict):
    prior = _prior_review_state()
    for key in ("review_commissions", "proposals", "review_events",
                "scope_audits"):
        previous = prior[key]
        current = src.get(key, [])
        if current[:len(previous)] != previous:
            raise LedgerError(
                f"{key}: visible first-parent history must remain an exact "
                "append-only prefix; failed and stale records cannot be "
                "deleted or rewritten"
            )


def validate_review_commissions(src: dict):
    rp = src["review_protocol"]
    designation = rp.get("designation")
    for i, rec in enumerate(src.get("review_commissions", [])):
        ctx = f"review_commissions[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, COMMISSION_KEYS, ctx)
        require_str(rec, "title", ctx)
        source_version = require_str(rec, "source_version", ctx)
        for key in ("scope_sha256", "protocol_sha256",
                    "plant_commitment_sha256", "seed_commitment_sha256"):
            value = require_str(rec, key, ctx)
            if not SHA256_HEX_RE.fullmatch(value):
                raise LedgerError(
                    f"{ctx}.{key} must be 64 lowercase hex characters")
        if rec["plant_commitment_sha256"] == rec["seed_commitment_sha256"]:
            raise LedgerError(
                f"{ctx}: plant and seed commitments must be distinct")
        commissioned = _require_utc(
            rec["commissioned_at_utc"], f"{ctx}.commissioned_at_utc")
        window = rec["received_window"]
        if not isinstance(window, dict):
            raise LedgerError(f"{ctx}.received_window must be an object")
        exact_keys(window, ["opens_at_utc", "closes_at_utc"],
                   f"{ctx}.received_window")
        opens = _require_utc(window["opens_at_utc"],
                             f"{ctx}.received_window.opens_at_utc")
        closes = _require_utc(window["closes_at_utc"],
                              f"{ctx}.received_window.closes_at_utc")
        cutoff = _require_utc(rec["cutoff_at_utc"],
                              f"{ctx}.cutoff_at_utc")
        if not commissioned < opens < closes <= cutoff:
            raise LedgerError(
                f"{ctx}: chronology must be commissioned < open < close <= cutoff")
        if designation is None:
            raise LedgerError(
                f"{ctx}: commissioning requires the named Darshu/Dhanush designation")
        custodian = require_str(rec, "custodian_identity", ctx)
        if custodian != designation["custodian"]:
            raise LedgerError(
                f"{ctx}: custodian_identity must equal the designated custodian")
        reviewers = rec["reviewers"]
        if not isinstance(reviewers, list) or not reviewers:
            raise LedgerError(f"{ctx}: reviewers must be a non-empty list")
        identities = []
        disciplines = set()
        covered = set()
        barred = {designation["severity_owner"],
                  designation["independent_checker"], custodian}
        for j, reviewer in enumerate(reviewers):
            rctx = f"{ctx}.reviewers[{j}]"
            if not isinstance(reviewer, dict):
                raise LedgerError(f"{rctx} must be an object")
            exact_keys(reviewer, REVIEWER_KEYS, rctx)
            identity = require_str(reviewer, "identity", rctx)
            discipline = require_str(reviewer, "discipline", rctx)
            if identity in barred:
                raise LedgerError(
                    f"{rctx}: reviewer conflicts with custodian, Darshu, or Dhanush")
            identities.append(identity)
            disciplines.add(discipline)
            refs = reviewer["criterion_refs"]
            if not isinstance(refs, list) or not refs:
                raise LedgerError(f"{rctx}.criterion_refs must be non-empty")
            canonical = [item for item in REVIEW_CRITERIA if item in refs]
            if refs != canonical or len(refs) != len(set(refs)):
                raise LedgerError(
                    f"{rctx}.criterion_refs must be unique and in canonical order")
            unknown = set(refs) - set(REVIEW_CRITERIA)
            if unknown:
                raise LedgerError(
                    f"{rctx}.criterion_refs contains unknown criteria: {sorted(unknown)}")
            covered.update(refs)
            if reviewer["consent_attestation"] != REVIEWER_CONSENT:
                raise LedgerError(f"{rctx}: reviewer consent attestation is not exact")
            if reviewer["conflict_attestation"] != REVIEWER_CONFLICT_CLEAR:
                raise LedgerError(f"{rctx}: reviewer conflict attestation is not exact")
            if reviewer["compensation_attestation"] != REVIEWER_COMPENSATION_CLEAR:
                raise LedgerError(
                    f"{rctx}: findings-contingent compensation is refused")
        if len(identities) != len(set(identities)):
            raise LedgerError(f"{ctx}: reviewer identities must be unique")
        if len(disciplines) < 2:
            raise LedgerError(
                f"{ctx}: the panel must contain at least two disciplines")
        if rec["criterion_coverage"] != list(REVIEW_CRITERIA):
            raise LedgerError(
                f"{ctx}.criterion_coverage must be the exact full criterion list")
        if covered != set(REVIEW_CRITERIA):
            raise LedgerError(
                f"{ctx}: reviewer criterion union must cover every criterion")
        if rec["packet_paths"] != list(REVIEW_PACKET_PATHS):
            raise LedgerError(
                f"{ctx}.packet_paths must be the exact ordered packet manifest")
        if source_version == src["source_version"]:
            if rec["scope_sha256"] != review_scope_digest(src):
                raise LedgerError(
                    f"{ctx}: current-source commission scope digest is stale")
            actual_protocol = hashlib.sha256(_input_bytes(PROTOCOL_DOC)).hexdigest()
            if rec["protocol_sha256"] != actual_protocol:
                raise LedgerError(
                    f"{ctx}: current-source commission protocol digest is stale")


REVIEW_EVENT_KEYS = [
    "id", "title", "commission_ref", "packet_commit_sha", "source_version",
    "scope_sha256", "protocol_sha256", "intake_receipt", "control_reveal",
    "seeded_control", "planted_control", "outcome_status", "outcome_reason",
]


def validate_review_events(src: dict):
    commissions = {row["id"]: row
                   for row in src.get("review_commissions", [])}
    for i, rec in enumerate(src.get("review_events", [])):
        ctx = f"review_events[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, REVIEW_EVENT_KEYS, ctx)
        require_str(rec, "title", ctx)
        commission = commissions.get(rec["commission_ref"])
        if commission is None:
            raise LedgerError(f"{ctx}: commission_ref names no commission")
        if not GIT_COMMIT_RE.fullmatch(require_str(rec, "packet_commit_sha", ctx)):
            raise LedgerError(
                f"{ctx}.packet_commit_sha must be a 40-character lowercase Git id")
        for key in ("source_version", "scope_sha256", "protocol_sha256"):
            if rec[key] != commission[key]:
                raise LedgerError(f"{ctx}.{key} must equal its commission")
        intake = rec["intake_receipt"]
        if not isinstance(intake, dict):
            raise LedgerError(f"{ctx}.intake_receipt must be an object")
        exact_keys(intake,
                   ["frozen_at_utc", "ordered_proposal_ids", "manifest_sha256"],
                   f"{ctx}.intake_receipt")
        _require_utc(intake["frozen_at_utc"],
                     f"{ctx}.intake_receipt.frozen_at_utc")
        if not isinstance(intake["ordered_proposal_ids"], list):
            raise LedgerError(
                f"{ctx}.intake_receipt.ordered_proposal_ids must be a list")
        if len(intake["ordered_proposal_ids"]) != \
                len(set(intake["ordered_proposal_ids"])):
            raise LedgerError(f"{ctx}: intake proposal ids must be unique")
        if not SHA256_HEX_RE.fullmatch(
                require_str(intake, "manifest_sha256", f"{ctx}.intake_receipt")):
            raise LedgerError(f"{ctx}: intake manifest digest is malformed")
        reveal = rec["control_reveal"]
        if not isinstance(reveal, dict):
            raise LedgerError(f"{ctx}.control_reveal must be an object")
        exact_keys(
            reveal,
            ["revealed_at_utc", "plant_preimage_sha256",
             "seed_preimage_sha256", "planted_proposal_ref", "seed_results",
             "plant_match_checked_by", "plant_match_reason"],
            f"{ctx}.control_reveal",
        )
        _require_utc(reveal["revealed_at_utc"],
                     f"{ctx}.control_reveal.revealed_at_utc")
        for key in ("plant_preimage_sha256", "seed_preimage_sha256"):
            if not SHA256_HEX_RE.fullmatch(require_str(
                    reveal, key, f"{ctx}.control_reveal")):
                raise LedgerError(f"{ctx}.control_reveal.{key} is malformed")
        planted = reveal["planted_proposal_ref"]
        if planted is not None and not isinstance(planted, str):
            raise LedgerError(
                f"{ctx}.control_reveal.planted_proposal_ref must be null or an id")
        if not isinstance(reveal["seed_results"], list):
            raise LedgerError(f"{ctx}.control_reveal.seed_results must be a list")
        require_str(reveal, "plant_match_checked_by", f"{ctx}.control_reveal")
        require_str(reveal, "plant_match_reason", f"{ctx}.control_reveal")
        for key in ("seeded_control", "planted_control"):
            control = rec[key]
            if not isinstance(control, dict):
                raise LedgerError(f"{ctx}.{key} must be an object")
            exact_keys(control, ["status", "reason"], f"{ctx}.{key}")
            if control["status"] not in ("passed", "failed"):
                raise LedgerError(f"{ctx}.{key}.status must be passed or failed")
            require_str(control, "reason", f"{ctx}.{key}")
        if rec["outcome_status"] not in ("passed", "failed"):
            raise LedgerError(f"{ctx}.outcome_status must be passed or failed")
        require_str(rec, "outcome_reason", ctx)


PROPOSAL_KEYS = [
    "id", "title", "proposal", "source_kind", "source_identity",
    "received_at_utc", "triaged_at_utc", "severity_owner_identity",
    "materiality_finding", "materiality_reason", "classification",
    "checked_at_utc", "independent_checker_identity", "check_finding",
    "check_reason", "proposal_disposition", "disposition_at_utc", "reasons",
    "review_event_ref", "control_kind",
]
PROPOSAL_OPTIONAL = [
    "severity", "created_record_refs", "routed_unestablished_disposition",
    "defect_row_ref", "retained_limit_binding",
]


def _expected_retained_binding(src: dict, defect: dict) -> dict:
    claim = next(row for row in src["claims"]
                 if row["id"] == defect["affected_claim_ref"])
    return {
        "severity": severity_class(defect),
        "consequence": defect["consequence"],
        "owner_ref": defect["owner_ref"],
        "closure_condition": defect["closure_condition"],
        "applicable_gate_refs": defect["applicable_gate_refs"],
        "public_claim_restriction": claim["public_claim_restriction"],
    }


def validate_proposals(src: dict, ids: dict):
    events = {row["id"]: row for row in src.get("review_events", [])}
    commissions = {row["id"]: row
                   for row in src.get("review_commissions", [])}
    defects = {row["id"]: row for row in src.get("defects", [])}
    designation = src["review_protocol"].get("designation")
    for i, rec in enumerate(src.get("proposals", [])):
        ctx = f"proposals[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, PROPOSAL_KEYS, ctx, optional=PROPOSAL_OPTIONAL)
        for key in ("title", "proposal", "source_identity",
                    "materiality_reason", "check_reason", "reasons"):
            require_str(rec, key, ctx)
        event = events.get(rec["review_event_ref"])
        if event is None:
            raise LedgerError(f"{ctx}: review_event_ref names no review event")
        commission = commissions[event["commission_ref"]]
        received = _require_utc(rec["received_at_utc"],
                                f"{ctx}.received_at_utc")
        triaged = _require_utc(rec["triaged_at_utc"],
                              f"{ctx}.triaged_at_utc")
        checked = _require_utc(rec["checked_at_utc"],
                              f"{ctx}.checked_at_utc")
        disposed = _require_utc(rec["disposition_at_utc"],
                               f"{ctx}.disposition_at_utc")
        window = commission["received_window"]
        if not window["opens_at_utc"] <= received <= window["closes_at_utc"]:
            raise LedgerError(f"{ctx}: proposal received outside its window")
        if not received <= triaged <= checked <= disposed:
            raise LedgerError(
                f"{ctx}: chronology must be received <= triaged <= checked <= disposed")
        if designation is None:
            raise LedgerError(f"{ctx}: Darshu/Dhanush designation is absent")
        if rec["severity_owner_identity"] != designation["severity_owner"]:
            raise LedgerError(f"{ctx}: every proposal requires Darshu triage")
        if rec["independent_checker_identity"] != \
                designation["independent_checker"]:
            raise LedgerError(f"{ctx}: every proposal requires Dhanush checking")
        if rec["check_finding"] not in ("confirmed", "corrected"):
            raise LedgerError(f"{ctx}.check_finding must be confirmed or corrected")
        if rec["source_kind"] not in ("reviewer", "seed"):
            raise LedgerError(f"{ctx}.source_kind must be reviewer or seed")
        reviewer_ids = {row["identity"] for row in commission["reviewers"]}
        if rec["source_kind"] == "reviewer" and \
                rec["source_identity"] not in reviewer_ids:
            raise LedgerError(f"{ctx}: reviewer source is not on the panel")
        if rec["source_kind"] == "seed" and \
                rec["source_identity"] != "committed-seed-control":
            raise LedgerError(f"{ctx}: seed source identity is not exact")
        if rec["control_kind"] not in ("none", "seed", "plant-match"):
            raise LedgerError(f"{ctx}: unknown control_kind")
        if rec["control_kind"] == "seed" and rec["source_kind"] != "seed":
            raise LedgerError(f"{ctx}: seed controls must record seeded origin")
        if rec["control_kind"] == "plant-match" and \
                rec["source_kind"] != "reviewer":
            raise LedgerError(
                f"{ctx}: a planted match must be independently reviewer-sourced")
        finding = rec["materiality_finding"]
        if finding not in ("material", "immaterial"):
            raise LedgerError(f"{ctx}: unknown materiality_finding")
        if finding == "material":
            if rec.get("severity") not in ("critical", "material"):
                raise LedgerError(
                    f"{ctx}: material proposals require critical or material severity")
        elif "severity" in rec:
            raise LedgerError(f"{ctx}: severity belongs only on material proposals")
        classification = rec["classification"]
        if classification not in PROPOSAL_CLASSIFICATIONS:
            raise LedgerError(f"{ctx}: unknown classification")
        disposition = rec["proposal_disposition"]
        if disposition not in PROPOSAL_DISPOSITIONS:
            raise LedgerError(f"{ctx}: unknown proposal_disposition")
        if disposition == "added":
            if finding != "material" or classification != "material-omission":
                raise LedgerError(
                    f"{ctx}: added requires material-omission classification")
            refs = rec.get("created_record_refs")
            if not isinstance(refs, list) or not refs:
                raise LedgerError(f"{ctx}: added must name created records")
            for j, ref in enumerate(refs):
                if ref not in ids:
                    validate_reference(ref, f"{ctx}.created_record_refs[{j}]")
        elif "created_record_refs" in rec:
            raise LedgerError(f"{ctx}: created_record_refs belongs only on added")
        if disposition == "classified-out":
            allowed = {"duplicate", "immaterial", *UNESTABLISHED_DISPOSITIONS}
            if classification not in allowed:
                raise LedgerError(
                    f"{ctx}: classified-out classification is not permitted")
            routed = rec.get("routed_unestablished_disposition")
            if classification in UNESTABLISHED_DISPOSITIONS:
                if routed != classification:
                    raise LedgerError(
                        f"{ctx}: classification-to-Unestablished mapping must be exact")
            elif routed is not None:
                raise LedgerError(
                    f"{ctx}: duplicate or immaterial carries no routed disposition")
        elif "routed_unestablished_disposition" in rec:
            raise LedgerError(
                f"{ctx}: routing disposition belongs only on classified-out")
        if disposition == "retained-limit":
            if finding != "material" or classification != "retained-limit":
                raise LedgerError(
                    f"{ctx}: retained-limit disposition requires matching classification")
            defect = defects.get(rec.get("defect_row_ref"))
            if defect is None:
                raise LedgerError(f"{ctx}: retained limit must link a defect row")
            expected = _expected_retained_binding(src, defect)
            if rec.get("retained_limit_binding") != expected:
                raise LedgerError(
                    f"{ctx}: retained-limit binding must match the defect and claim")
            if rec["severity"] != expected["severity"]:
                raise LedgerError(
                    f"{ctx}: retained-limit severity must match its defect")
        else:
            for key in ("defect_row_ref", "retained_limit_binding"):
                if key in rec:
                    raise LedgerError(
                        f"{ctx}: {key} belongs only on retained-limit")


def validate_review_event_completeness(src: dict):
    proposals_by_id = {row["id"]: row for row in src.get("proposals", [])}
    commissions = {row["id"]: row
                   for row in src.get("review_commissions", [])}
    designation = src["review_protocol"]["designation"]
    for rec in src.get("review_events", []):
        ctx = f"review_events ({rec['id']})"
        commission = commissions[rec["commission_ref"]]
        ordered_ids = rec["intake_receipt"]["ordered_proposal_ids"]
        actual_ids = [row["id"] for row in src.get("proposals", [])
                      if row["review_event_ref"] == rec["id"]]
        if ordered_ids != actual_ids:
            raise LedgerError(
                f"{ctx}: intake ordered proposal ids must equal the event proposal set")
        ordered = [proposals_by_id[pid] for pid in ordered_ids]
        expected_digest = _proposal_intake_digest(rec["id"], ordered)
        if rec["intake_receipt"]["manifest_sha256"] != expected_digest:
            raise LedgerError(f"{ctx}: intake manifest digest does not match")
        frozen = rec["intake_receipt"]["frozen_at_utc"]
        reveal = rec["control_reveal"]
        revealed = reveal["revealed_at_utc"]
        if not commission["received_window"]["closes_at_utc"] <= frozen <= revealed:
            raise LedgerError(
                f"{ctx}: intake freezes after window close and before reveal")
        if revealed < commission["cutoff_at_utc"]:
            raise LedgerError(f"{ctx}: controls may not reveal before cutoff")
        for row in ordered:
            if row["triaged_at_utc"] > revealed or row["checked_at_utc"] > revealed:
                raise LedgerError(
                    f"{ctx}: triage and Dhanush checking must finish before reveal")
        seed_rows = [row for row in ordered if row["control_kind"] == "seed"]
        results = reveal["seed_results"]
        result_refs = [row.get("proposal_ref") for row in results]
        if len(result_refs) != len(set(result_refs)):
            raise LedgerError(f"{ctx}: seed result proposal refs must be unique")
        if set(result_refs) != {row["id"] for row in seed_rows}:
            raise LedgerError(
                f"{ctx}: reveal must adjudicate every and only seeded proposals")
        expected_sides = {row.get("expected_materiality") for row in results}
        if expected_sides != {"material", "immaterial"}:
            raise LedgerError(
                f"{ctx}: seeds must cover both material and immaterial cases")
        seed_ok = (
            reveal["seed_preimage_sha256"] == commission["seed_commitment_sha256"]
        )
        for j, result in enumerate(results):
            rctx = f"{ctx}.control_reveal.seed_results[{j}]"
            exact_keys(
                result,
                ["proposal_ref", "expected_materiality", "expected_severity",
                 "expected_disposition", "verified_by", "verification_reason"],
                rctx,
            )
            row = proposals_by_id[result["proposal_ref"]]
            if result["expected_materiality"] not in ("material", "immaterial"):
                raise LedgerError(f"{rctx}: invalid expected materiality")
            if result["expected_materiality"] == "material":
                if result["expected_severity"] not in ("critical", "material"):
                    raise LedgerError(f"{rctx}: material seed needs expected severity")
            elif result["expected_severity"] is not None:
                raise LedgerError(f"{rctx}: immaterial seed severity must be null")
            if result["expected_disposition"] not in PROPOSAL_DISPOSITIONS:
                raise LedgerError(f"{rctx}: invalid expected disposition")
            if result["verified_by"] != designation["independent_checker"]:
                raise LedgerError(f"{rctx}: Dhanush must verify seed results")
            require_str(result, "verification_reason", rctx)
            if (row["materiality_finding"] != result["expected_materiality"]
                    or row.get("severity") != result["expected_severity"]
                    or row["proposal_disposition"] != result["expected_disposition"]):
                seed_ok = False
        plant_ref = reveal["planted_proposal_ref"]
        plant_row = proposals_by_id.get(plant_ref)
        plant_ok = (
            reveal["plant_preimage_sha256"] == commission["plant_commitment_sha256"]
            and plant_row is not None
            and plant_row["control_kind"] == "plant-match"
            and plant_row["source_kind"] == "reviewer"
            and reveal["plant_match_checked_by"] ==
            designation["independent_checker"]
        )
        expected_seed = "passed" if seed_ok else "failed"
        expected_plant = "passed" if plant_ok else "failed"
        if rec["seeded_control"]["status"] != expected_seed:
            raise LedgerError(
                f"{ctx}: seeded-control status must be derived from reveal and results")
        if rec["planted_control"]["status"] != expected_plant:
            raise LedgerError(
                f"{ctx}: planted-control status must be derived from reveal and match")
        expected_outcome = (
            "passed" if expected_seed == expected_plant == "passed" else "failed")
        if rec["outcome_status"] != expected_outcome:
            raise LedgerError(
                f"{ctx}: outcome_status must be derived from both controls")


SCOPE_AUDIT_KEYS = [
    "id", "title", "source_version", "scope_sha256", "protocol_sha256",
    "executed_at_utc", "method", "criterion_coverage", "control_refs",
    "commands", "finding_refs", "result", "evidence_ceiling",
]


def _validated_compact_verification_receipt(
        ref: str,
        context: str,
        expected_source_version: str,
        expected_audit_id: str,
):
    if not isinstance(ref, str) or not VERIFICATION_RECEIPT_REF_RE.fullmatch(ref):
        raise LedgerError(
            f"{context}: verification_receipt_ref must be a content-addressed "
            "tracked receipt path"
        )
    if ref in _VERIFICATION_RECEIPT_CACHE:
        cached = _VERIFICATION_RECEIPT_CACHE[ref]
        if cached.get("source_version") != expected_source_version:
            raise LedgerError(
                f"{context}: receipt source version must match the current "
                "ledger source")
        if cached.get("audit_id") != expected_audit_id:
            raise LedgerError(
                f"{context}: receipt audit id must match the current audit")
        return cached
    path = ROOT / ref
    receipt_bytes = _input_bytes(path)
    helper = ROOT / "new-book-plans/20-verification-receipt.py"
    if not helper.is_file():
        raise LedgerError(
            f"{context}: verification receipt validator is missing")
    module_name = "_rights_verification_receipt_for_ledger"
    try:
        load_cached_source_module(
            "verification_lock",
            ROOT / "new-book-plans/verification_lock.py",
        )
        module = load_cached_source_module(module_name, helper)
    except Exception as exc:
        raise LedgerError(
            f"{context}: cannot load verification receipt validator: {exc}"
        ) from exc
    validator = getattr(module, "load_and_validate_receipt", None)
    if not callable(validator):
        raise LedgerError(
            f"{context}: receipt validator exposes no portable loader")
    try:
        receipt = validator(
            path,
            require_local=False,
            check_environment=False,
            check_engine=False,
            root=ROOT,
            raw_bytes=receipt_bytes,
        )
    except Exception as exc:
        raise LedgerError(
            f"{context}: verification receipt is invalid: {exc}") from exc
    if not isinstance(receipt, dict):
        raise LedgerError(
            f"{context}: receipt validator returned no compact receipt")
    if receipt.get("schema_version") != 2:
        raise LedgerError(f"{context}: receipt schema must be v2")
    if receipt.get("protocol_version") != 5:
        raise LedgerError(f"{context}: receipt protocol must be v5")
    if receipt.get("status") != "all-passed":
        raise LedgerError(f"{context}: receipt status must be all-passed")
    if receipt.get("source_version") == LEGACY_V1_CLOSURE_SOURCE:
        raise LedgerError(f"{context}: a v2 receipt may not downgrade to v1")
    if receipt.get("source_version") != expected_source_version:
        raise LedgerError(
            f"{context}: receipt source version must match the current ledger source")
    if receipt.get("audit_id") != expected_audit_id:
        raise LedgerError(
            f"{context}: receipt audit id must match the current audit")
    _VERIFICATION_RECEIPT_CACHE[ref] = receipt
    return receipt


def _expected_current_scope_audit_commands(rec: dict, context: str) -> list:
    expected = list(SCOPE_AUDIT_V2_PENDING_COMMANDS)
    if rec["result"] == SCOPE_AUDIT_RESULT:
        receipt_ref = require_str(rec, "verification_receipt_ref", context)
        _validated_compact_verification_receipt(
            receipt_ref,
            context,
            rec["source_version"],
            rec["id"],
        )
        expected.append(
            f"./verify.sh --commit-gate {receipt_ref} --transition audit")
    elif "verification_receipt_ref" in rec:
        raise LedgerError(
            f"{context}: only a passing current audit may name a receipt")
    return expected


def _gate_a_audit_finding_refs(src: dict) -> list:
    return sorted(
        row["id"] for row in src["defects"]
        if "gate-a" in row["applicable_gate_refs"]
    )


def qualifying_scope_audits(src: dict) -> list:
    scope = review_scope_digest(src)
    protocol = hashlib.sha256(_input_bytes(PROTOCOL_DOC)).hexdigest()
    return [
        row for row in src.get("scope_audits", [])
        if row["source_version"] == src["source_version"]
        and row["scope_sha256"] == scope
        and row["protocol_sha256"] == protocol
        and row["result"] == SCOPE_AUDIT_RESULT
    ]


def validate_scope_audits(src: dict):
    audits = src.get("scope_audits")
    if not isinstance(audits, list):
        raise LedgerError("scope_audits must be an append-only list")
    seen = set()
    for i, rec in enumerate(audits):
        ctx = f"scope_audits[{i}] ({rec.get('id', '?')})"
        exact_keys(rec, SCOPE_AUDIT_KEYS, ctx,
                   optional=[
                       "author_basis", "policy_basis",
                       "verification_receipt_ref"])
        basis_keys = [key for key in ("author_basis", "policy_basis")
                      if key in rec]
        if len(basis_keys) != 1:
            raise LedgerError(
                f"{ctx}: exactly one historical author_basis or current "
                "policy_basis is required")
        require_str(rec, basis_keys[0], ctx)
        require_str(rec, "title", ctx)
        require_str(rec, "source_version", ctx)
        for key in ("scope_sha256", "protocol_sha256"):
            if not SHA256_HEX_RE.fullmatch(require_str(rec, key, ctx)):
                raise LedgerError(f"{ctx}.{key} must be a SHA-256 digest")
        _require_utc(rec["executed_at_utc"], f"{ctx}.executed_at_utc")
        if rec["method"] != SCOPE_AUDIT_METHOD:
            raise LedgerError(f"{ctx}.method must be the checker-owned method")
        for key in ("criterion_coverage", "control_refs", "commands",
                    "finding_refs"):
            value = rec[key]
            if (not isinstance(value, list)
                    or len(value) != len(set(value))
                    or any(not isinstance(item, str) or not item
                           for item in value)):
                raise LedgerError(f"{ctx}.{key} must be a unique string list")
        if rec["result"] not in {SCOPE_AUDIT_RESULT, "pending", "failed"}:
            raise LedgerError(f"{ctx}.result is not an audit result")
        require_str(rec, "evidence_ceiling", ctx)
        if rec["id"] in seen:
            raise LedgerError(f"{ctx}: duplicate audit id")
        seen.add(rec["id"])
    scope = review_scope_digest(src)
    protocol = hashlib.sha256(_input_bytes(PROTOCOL_DOC)).hexdigest()
    current = [
        rec for rec in audits
        if rec["source_version"] == src["source_version"]
        and rec["scope_sha256"] == scope
        and rec["protocol_sha256"] == protocol
    ]
    if not current:
        if src["coverage_population"]["status"] != "complete":
            return
        raise LedgerError(
            "scope_audits requires a current-source repository audit")
    for rec in current:
        ctx = f"scope_audits current ({rec['id']})"
        if rec["criterion_coverage"] != list(REVIEW_CRITERIA):
            raise LedgerError(f"{ctx}: criterion coverage must be exact")
        if rec["control_refs"] != list(SCOPE_AUDIT_CONTROL_REFS):
            raise LedgerError(f"{ctx}: control references must be exact")
        expected_commands = _expected_current_scope_audit_commands(rec, ctx)
        if rec["commands"] != expected_commands:
            raise LedgerError(f"{ctx}: command chain must be exact")
        if rec["finding_refs"] != _gate_a_audit_finding_refs(src):
            raise LedgerError(f"{ctx}: finding references must cover Gate A")
        if rec["evidence_ceiling"] != SCOPE_AUDIT_EVIDENCE_CEILING:
            raise LedgerError(f"{ctx}: evidence ceiling must be byte-exact")
        if "author_basis" in rec:
            raise LedgerError(
                f"{ctx}: a current audit may not depend on an author act")
        if rec.get("policy_basis") != SCOPE_AUDIT_POLICY_BASIS:
            raise LedgerError(f"{ctx}: policy basis must be checker-owned")
        validate_reference(rec["policy_basis"], f"{ctx}.policy_basis")


def qualifying_review_events(src: dict) -> list:
    scope = review_scope_digest(src)
    protocol = hashlib.sha256(_input_bytes(PROTOCOL_DOC)).hexdigest()
    return [
        row for row in src.get("review_events", [])
        if row["outcome_status"] == "passed"
        and row["source_version"] == src["source_version"]
        and row["scope_sha256"] == scope
        and row["protocol_sha256"] == protocol
    ]


R7_BUILT = {
    "status": "built",
    "route_status": "built",
    "consequence": (
        "repository structure, declared criteria, Gate-A defect coverage, and "
        "watched-failing mutations are reproducibly checked"),
    "closure_condition": (
        "the current-source repository audit is present and exact; external "
        "human review remains optional evidence and is never a project gate"),
}


def validate_review_route_state(src: dict):
    route = next(row for row in src["routes"] if row["id"] == "FS-RTE-07")
    for key, value in R7_BUILT.items():
        if route[key] != value:
            raise LedgerError(
                f"FS-RTE-07.{key} must equal the repository-audit state exactly")


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
    """Bind the amended confirmed protocol and Darshu/Dhanush designation.

    Commissions live in the append-only `review_commissions` population. The
    protocol record contains no singular mutable commitment slot.
    """
    if "review_protocol" not in src:
        raise LedgerError("review_protocol must be present")
    rp = src["review_protocol"]
    ctx = "review_protocol"
    if not isinstance(rp, dict):
        raise LedgerError(f"{ctx} must be an object")
    exact_keys(rp, ["protocol_ref", "protocol_status", "status_line_ref",
                    "policy_basis", "mode", "external_review_policy",
                    "designation"], ctx)
    status = require_str(rp, "protocol_status", ctx)
    if status != PROTOCOL_STATUS_CONFIRMED:
        raise LedgerError(
            f"{ctx}.protocol_status must be the repository-enforced status")
    if rp["policy_basis"] != SCOPE_AUDIT_POLICY_BASIS:
        raise LedgerError(f"{ctx}.policy_basis must be checker-owned")
    validate_reference(rp["policy_basis"], f"{ctx}.policy_basis")
    if rp["mode"] != "repository-adversarial-audit":
        raise LedgerError(f"{ctx}.mode must be repository-adversarial-audit")
    if rp["external_review_policy"] != "optional-non-gating":
        raise LedgerError(f"{ctx}.external_review_policy must be optional-non-gating")
    ref = require_str(rp, "protocol_ref", ctx)
    validate_reference(ref, f"{ctx}.protocol_ref")
    if ref.split("::", 1)[0] != str(PROTOCOL_DOC):
        raise LedgerError(f"{ctx}.protocol_ref must resolve into {PROTOCOL_DOC}")
    line_ref = require_str(rp, "status_line_ref", ctx)
    validate_reference(line_ref, f"{ctx}.status_line_ref")
    line_path, line_needle = line_ref.split("::", 1)
    if line_path != str(PROTOCOL_DOC) or line_needle != "Status: " + status:
        raise LedgerError(
            f"{ctx}: protocol status and document status line must match exactly")
    designation = rp["designation"]
    dctx = f"{ctx}.designation"
    if not isinstance(designation, dict):
        raise LedgerError(f"{dctx} must be an object")
    exact_keys(designation,
               ["severity_owner", "independent_checker", "custodian",
                "designated_date", "designation_status", "basis"], dctx)
    for key in ("severity_owner", "independent_checker", "custodian", "basis"):
        require_str(designation, key, dctx)
    if not ISO_DATE_RE.fullmatch(require_str(
            designation, "designated_date", dctx)):
        raise LedgerError(f"{dctx}.designated_date must be YYYY-MM-DD")
    if designation["designation_status"] != "retired-as-project-gate-dependency":
        raise LedgerError(
            f"{dctx}.designation_status must retire the external dependency")
    if designation["severity_owner"] == designation["independent_checker"]:
        raise LedgerError(f"{dctx}: Darshu and Dhanush must be distinct people")
    if designation["custodian"] in (
            designation["severity_owner"], designation["independent_checker"]):
        raise LedgerError(f"{dctx}: the pre-image custodian may not triage or check")


def _gate_a_condition_1_deferred(src: dict):
    """Return only the populations named by closure condition one.

    Optional external-review records are evidence inputs, not required scope
    populations. Empty optional arrays therefore cannot create a gate deferral.
    """
    populations = {
        "domains", "roles", "powers", "dependencies", "scenarios", "defects",
        COVERAGE_DEFERRAL_TYPE,
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
    audits = qualifying_scope_audits(src)
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
    if audits:
        rows.append((conds[4], "met-mechanically",
                     "a current-source repository adversarial audit covers the "
                     "declared criteria, exact checker controls, command chain, "
                     "and every Gate-A-applicable defect disposition"))
    else:
        rows.append((conds[4], "unmet",
                     "no current-source repository adversarial audit exists"))
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


def _gate_a_residual_ids(src: dict, resolution: dict) -> list:
    return sorted(
        row["id"] for row in src["defects"]
        if resolution[row["id"]]["resolution_status"] == "unresolved-for-claim"
        and severity_class(row) != "critical"
        and "gate-a" in row["applicable_gate_refs"]
    )


def _gate_a_claim_limitations(src: dict, residual_ids: list) -> list:
    defects = {row["id"]: row for row in src["defects"]}
    claims = {row["id"]: row for row in src["claims"]}
    return [
        {
            "defect_ref": rid,
            "affected_claim_ref": defects[rid]["affected_claim_ref"],
            "public_claim_restriction": claims[
                defects[rid]["affected_claim_ref"]]["public_claim_restriction"],
        }
        for rid in residual_ids
    ]


def _source_at_commit(commit_sha: str, context: str) -> dict:
    if commit_sha in _GIT_SOURCE_CACHE:
        return _GIT_SOURCE_CACHE[commit_sha]
    proc = subprocess.run(
        ["git", "show", f"{commit_sha}:{SOURCE.as_posix()}"],
        cwd=ROOT, check=False, capture_output=True, text=True,
    )
    if proc.returncode != 0:
        raise LedgerError(
            f"{context}: candidate commit does not contain the canonical source")
    try:
        candidate = json.loads(proc.stdout)
        _GIT_SOURCE_CACHE[commit_sha] = candidate
        return candidate
    except json.JSONDecodeError as exc:
        raise LedgerError(
            f"{context}: candidate canonical source is not valid JSON") from exc


def _validate_legacy_v1_closure_receipt(rec: dict, audit: dict):
    ctx = "closure_record"
    receipt = rec["verification_receipt"]
    rctx = f"{ctx}.verification_receipt"
    if not isinstance(receipt, dict):
        raise LedgerError(f"{rctx} must be an object")
    exact_keys(
        receipt,
        ["candidate_commit_sha", "verified_at_utc", "commands",
         "result", "transcript_sha256"],
        rctx,
    )
    allowlisted = (
        rec["candidate_commit_sha"] == LEGACY_V1_CLOSURE_CANDIDATE
        and rec["source_version"] == LEGACY_V1_CLOSURE_SOURCE
        and audit["id"] == LEGACY_V1_CLOSURE_AUDIT
        and receipt.get("transcript_sha256") == LEGACY_V1_CLOSURE_TRANSCRIPT
    )
    if not allowlisted:
        raise LedgerError(
            f"{rctx}: legacy v1 is accepted only for the exact state-form "
            "candidate, source, audit, and transcript allowlist")
    if receipt["candidate_commit_sha"] != rec["candidate_commit_sha"]:
        raise LedgerError(
            f"{rctx}: candidate commit must match closure candidate")
    _require_utc(receipt["verified_at_utc"], f"{rctx}.verified_at_utc")
    if receipt["commands"] != list(REQUIRED_VERIFY_COMMANDS):
        raise LedgerError(
            f"{rctx}.commands must equal the legacy required verifier chain")
    if receipt["result"] != "all-passed":
        raise LedgerError(f"{rctx}.result must be all-passed")
    if not SHA256_HEX_RE.fullmatch(require_str(
            receipt, "transcript_sha256", rctx)):
        raise LedgerError(f"{rctx}.transcript_sha256 is malformed")


def _validate_v2_closure_receipt(rec: dict, audit: dict):
    ctx = "closure_record"
    rctx = f"{ctx}.verification_receipt_ref"
    receipt_ref = require_str(rec, "verification_receipt_ref", ctx)
    if audit.get("verification_receipt_ref") != receipt_ref:
        raise LedgerError(
            f"{ctx}: audit and closure must bind the same verification receipt")
    receipt = _validated_compact_verification_receipt(
        receipt_ref,
        rctx,
        rec["source_version"],
        audit["id"],
    )
    module = sys.modules.get("_rights_verification_receipt_for_ledger")
    transition_validator = getattr(
        module, "validate_recorded_transition", None)
    if not callable(transition_validator):
        raise LedgerError(
            f"{rctx}: receipt validator exposes no recorded-transition check")
    try:
        semantic_candidate_sha = transition_validator(
            receipt,
            rec["candidate_commit_sha"],
            "audit",
            receipt_path=receipt_ref,
            root=ROOT,
        )
    except Exception as exc:
        raise LedgerError(
            f"{rctx}: recorded audit transition is invalid: {exc}") from exc
    if not GIT_COMMIT_RE.fullmatch(semantic_candidate_sha):
        raise LedgerError(
            f"{rctx}: recorded transition returned no semantic candidate")
    semantic_candidate = _source_at_commit(
        semantic_candidate_sha, f"{rctx}.semantic_candidate")
    if semantic_candidate.get("source_version") != rec["source_version"]:
        raise LedgerError(
            f"{rctx}: receipt-bound semantic source version does not match")


def validate_closure_record(src: dict, readiness, resolution: dict):
    if "closure_record" not in src:
        raise LedgerError(
            "closure_record must be present - null until mechanical closure")
    rec = src["closure_record"]
    if rec is None:
        return
    ctx = "closure_record"
    if src["source_version"] == LEGACY_V1_CLOSURE_SOURCE:
        receipt_field = "verification_receipt"
    else:
        receipt_field = "verification_receipt_ref"
    exact_keys(
        rec,
        ["gate", "permitted_claim", "candidate_commit_sha", "source_version",
         "scope_sha256", "envelope_ref", "audit_cutoff_at_utc",
         "scope_audit_ref", "assurance_record_refs", "residual_refs",
         "claim_limitations", receipt_field, "closure_policy_ref"],
        ctx,
    )
    if rec["gate"] != "gate-a":
        raise LedgerError(f"{ctx}.gate must be gate-a")
    if rec["permitted_claim"] != GATE_A_PERMITTED_CLAIM:
        raise LedgerError(f"{ctx}.permitted_claim must be byte-exact")
    if not GIT_COMMIT_RE.fullmatch(require_str(
            rec, "candidate_commit_sha", ctx)):
        raise LedgerError(f"{ctx}.candidate_commit_sha must be a Git id")
    if rec["source_version"] != src["source_version"]:
        raise LedgerError(f"{ctx}.source_version must match the closure source")
    if rec["scope_sha256"] != review_scope_digest(src):
        raise LedgerError(f"{ctx}.scope_sha256 must match the semantic scope")
    if rec["envelope_ref"] != "FS-ENV-01":
        raise LedgerError(f"{ctx}.envelope_ref must be FS-ENV-01")
    envelope = next(row for row in src["envelope"]
                    if row["id"] == rec["envelope_ref"])
    if envelope["envelope_status"] != "versioned-structure":
        raise LedgerError(
            f"{ctx}: envelope must remain versioned structure, never calibration")
    qualifying = {row["id"]: row for row in qualifying_scope_audits(src)}
    audit = qualifying.get(rec["scope_audit_ref"])
    if audit is None:
        raise LedgerError(
            f"{ctx}: scope_audit_ref must name a current-source repository audit")
    cutoff = _require_utc(rec["audit_cutoff_at_utc"],
                          f"{ctx}.audit_cutoff_at_utc")
    if cutoff != audit["executed_at_utc"]:
        raise LedgerError(f"{ctx}: audit cutoff must equal audit execution")
    rows, preconditions = readiness
    for name, status, reason in list(rows) + list(preconditions):
        if status not in READINESS_MET:
            raise LedgerError(
                f"{ctx}: a closure record may not exist while a closure "
                f"condition computes unmet - {name}: {reason}")
    if rec["assurance_record_refs"] != list(GATE_A_ASSURANCE_REFS):
        raise LedgerError(
            f"{ctx}.assurance_record_refs must equal the checker-derived set")
    for i, ref in enumerate(rec["assurance_record_refs"]):
        validate_reference(ref, f"{ctx}.assurance_record_refs[{i}]")
    residuals = _gate_a_residual_ids(src, resolution)
    if rec["residual_refs"] != residuals:
        raise LedgerError(f"{ctx}.residual_refs must equal the derived set")
    limitations = _gate_a_claim_limitations(src, residuals)
    if rec["claim_limitations"] != limitations:
        raise LedgerError(
            f"{ctx}.claim_limitations must bind every derived residual exactly")
    policy = require_str(rec, "closure_policy_ref", ctx)
    if policy != SCOPE_AUDIT_POLICY_BASIS:
        raise LedgerError(
            f"{ctx}.closure_policy_ref must equal the checker-owned policy")
    validate_reference(policy, f"{ctx}.closure_policy_ref")
    if receipt_field == "verification_receipt":
        _validate_legacy_v1_closure_receipt(rec, audit)
    else:
        _validate_v2_closure_receipt(rec, audit)
    if src["acceptance_gate"]["gate_a_status"] != "passed":
        raise LedgerError(
            f"{ctx}: a closure record requires gate_a_status passed")
    candidate = _source_at_commit(
        rec["candidate_commit_sha"], f"{ctx}.candidate_commit_sha")
    if candidate.get("closure_record") is not None:
        raise LedgerError(f"{ctx}: candidate closure_record must still be null")
    if candidate.get("acceptance_gate", {}).get("gate_a_status") != "not-passed":
        raise LedgerError(f"{ctx}: candidate Gate A status must still be not-passed")
    if candidate.get("source_version") != rec["source_version"]:
        raise LedgerError(f"{ctx}: candidate source version does not match")
    if review_scope_digest(candidate) != rec["scope_sha256"]:
        raise LedgerError(f"{ctx}: candidate semantic scope does not match")
    candidate_audits = {row["id"]: row
                        for row in candidate.get("scope_audits", [])}
    if candidate_audits.get(audit["id"]) != audit:
        raise LedgerError(
            f"{ctx}: candidate commit must contain the exact repository audit")
    candidate_frozen = copy.deepcopy(candidate)
    current_frozen = copy.deepcopy(src)
    for value in (candidate_frozen, current_frozen):
        value.pop("closure_record", None)
        value.pop("acceptance_gate", None)
    if candidate_frozen != current_frozen:
        raise LedgerError(
            f"{ctx}: closure source must exactly match the immutable candidate "
            "outside closure and acceptance metadata")

def validate_acceptance(src: dict):
    gate = src.get("acceptance_gate")
    if not isinstance(gate, dict):
        raise LedgerError("acceptance_gate must be an object")
    exact_keys(gate, ["verdict", "rollup_rule", "gate_a_status"],
               "acceptance_gate")
    closed = src.get("closure_record") is not None
    expected_verdict = VERDICT_PASSED if closed else VERDICT_NOT_PASSED
    if gate["verdict"] != expected_verdict:
        raise LedgerError(
            "acceptance_gate.verdict must be the byte-exact closure-derived "
            "verdict line"
        )
    require_str(gate, "rollup_rule", "acceptance_gate")
    if re.search(r"\d", gate["rollup_rule"]):
        raise LedgerError(
            "the rollup is non-numeric by rule — no digit may appear in it"
        )
    expected_status = "passed" if closed else "not-passed"
    if gate["gate_a_status"] != expected_status:
        raise LedgerError(
            "gate_a_status must be derived exactly from closure-record presence")


def validate(src: dict):
    validate_header(src)
    validate_bound_sources(src)
    validate_power_source_inventory(src)
    validate_coverage_population(src)
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
    # roles first: a body card's checking routes name roles, so a deferred or
    # emptied roles array must be diagnosed as that, not as a dangling body ref.
    validate_roles(src, ids)
    validate_bodies(src, ids)
    validate_body_map_cells(src)
    validate_power_population(src, ids)
    validate_state_form_semantic_mappings(src)
    validate_constitutional_effects(src, ids)
    validate_coverage_families(src, ids)
    validate_state_form_coverage_coupling(src)
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
    validate_scope_audits(src)
    validate_review_history(src)
    validate_review_commissions(src)
    validate_review_events(src)
    validate_proposals(src, ids)
    validate_review_event_completeness(src)
    validate_review_route_state(src)
    validate_severity_rubric(src)
    validate_deferred(src)
    validate_enum_mapping(src)
    validate_stopping_rule(src)
    validate_acceptance(src)
    readiness = compute_gate_a_readiness(src, resolution)
    validate_closure_record(src, readiness, resolution)
    validate_coverage_region(src)
    return resolution


# ── negative controls: sabotage first, trust after ───────────────────────────

def _transient_control_audit(src: dict, title: str) -> dict:
    """Build a synthetic current audit without reusing a bound receipt."""
    audit = copy.deepcopy(src["scope_audits"][-1])
    audit.update({
        "id": "FS-SAU-99",
        "title": title,
    })
    if "verification_receipt_ref" in audit:
        audit.update({
            "id": "FS-SAU-99",
            "result": "pending",
            "commands": list(SCOPE_AUDIT_V2_PENDING_COMMANDS),
        })
        audit.pop("verification_receipt_ref")
    return audit

def negative_controls(src: dict) -> int:
    """Each mutation must be rejected; a check never watched failing is not a
    check."""
    controls = []

    def control(name, mutate, expect=None, validator=None):
        controls.append((name, mutate, expect, validator))

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
    if src["power_population"]["status"] != "complete":
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
    else:
        control("complete power population cannot regain its deferral",
                lambda s: s["deferred_populations"].append({
                    "record_type": "powers",
                    "owner_ref": POWER_SOURCE_BINDING["owner_ref"],
                    "closure_condition": POWER_SOURCE_BINDING[
                        "closure_condition"],
                    "stage": "stage-3",
                }))
    if src["powers"]:
        control("a power grain cannot be bundled or duplicated",
                lambda s: s["powers"][0].update(
                    {"manifest_key": s["powers"][1]["manifest_key"]}))
        control("a power profile cannot be dropped",
                lambda s: s["powers"][0]["profiles"].pop())
        control("profile fields reject blank substitutes",
                _blank_first_power_profile)
        if src["coverage_population"]["status"] == "complete":
            control("coverage completion is an exact source-family prefix",
                    lambda s: s["coverage_population"][
                        "completed_source_families"].__setitem__(
                            0, "time-model"))
            control("complete coverage cannot regain its deferral",
                    _coverage_regain_deferral)
            control("contract prose cannot be repeated across cards",
                    _duplicate_contract_prose)
            control("every contract term keeps a source",
                    _contract_term_without_source)
            control("constitutional-effect count is checker-owned",
                    lambda s: s["constitutional_effects"].pop())
            control("constitutional-effect taxonomy is checker-owned",
                    lambda s: s["constitutional_effects"][0].update(
                        {"primary_class_ref": "class-02"}))
            control("constitutional-effect forbidden gates are explicit",
                    lambda s: s["constitutional_effects"][0][
                        "prohibited_inputs"].__setitem__(0, "other limits"))
            control("coverage family partitions every effect exactly once",
                    lambda s: s["coverage_families"][-1][
                        "effect_refs"].pop())
            control("universal-standing formal surface is checker-owned",
                    lambda s: next(row for row in s["coverage_families"] if row["id"]=="FS-CVF-011")["formal_statement_refs"].reverse())
            control("material-floor inventory cannot become an environmental floor",
                    lambda s: next(row for row in s["constitutional_effects"] if row["effect_key"]=="material-floor-inventory")["prohibited_inputs"].__setitem__(0,"Other boundaries only"))
            if any(row["effect_key"] in EQUALITY_EFFECT_KEYS
                   for row in src["constitutional_effects"]):
                control("equality effects cannot lose their closed profile",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "direct-discrimination")[
                                "profiles"].pop())
                control("equality effects cannot import person scoring",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "substantive-equality-status")[
                                "prohibited_inputs"].__setitem__(0, "Other limits"))
                control("equality domains derive from direct claims",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "proactive-accessibility").update(
                                {"domain_refs": ["FS-DOM-12"]}))
                control("constitutional effects receive no power allocation",
                        lambda s: s["function_allocations"][0].update(
                            {"power_ref": "FS-CCE-34"}))
                control("equality effects cannot borrow T3",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "custody-distinction-narrowing")[
                                "prohibited_inputs"].__setitem__(3, "Other limits"))
            if any(row["effect_key"] in FAMILY_LIFE_EFFECT_KEYS
                   for row in src["constitutional_effects"]):
                control("family effects cannot regain an omnibus status proof",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "legacy-status-nonproof")[
                                "prohibited_inputs"].__setitem__(0, "Other limits"))
                control("family status cannot become a confinement input",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "family-status-no-confinement")[
                                "prohibited_inputs"].__setitem__(3, "Other limits"))
                control("family effects receive no power allocation",
                        lambda s: s["function_allocations"][0].update(
                            {"power_ref": "FS-CCE-88"}))
            if any(row["effect_key"] in OBLIGATION_EFFECT_KEYS
                   for row in src["constitutional_effects"]):
                control("obligation origin profile cannot be dropped",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "public-respect-duty")[
                                "profiles"].pop(0))
                control("obligation adapters reuse every power-profile field",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "public-respect-duty")[
                                "profile_terms"][
                                    "liberty-power-limit-adapter"].popitem())
                control("nonreciprocity wall is explicit for every duty gate",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] ==
                                "all-entitlement-nonreciprocity")[
                                    "prohibited_inputs"].__setitem__(0,
                                        "Other limits"))
                control("finding routes require positive nonresponse",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] ==
                                "certified-positive-nonresponse")[
                                    "prohibited_inputs"].__setitem__(0,
                                        "Other limits"))
                control("ecological scientist is not an institutional proxy",
                        lambda s: next(row for row in s["constitutional_effects"]
                            if row["effect_key"] == "public-fulfil-duty")[
                                "checking_role_refs"].append("FS-ROL-49"))
            control("bounded delegation is decision-complete",
                    _incomplete_bounded_delegation)
            control("primary class follows the direct effect",
                    lambda s: s["powers"][0].update(
                        {"primary_class_ref": "class-10"}))
            control("domains derive only from affected claims",
                    lambda s: s["powers"][0].update(
                        {"domain_refs": ["FS-DOM-12"]}))
            control("planned tests cannot claim execution",
                    _planned_test_claims_execution)
            control("formalization cannot precede coverage metadata",
                    _premature_formalization)
            control("prose cannot precede formalization",
                    _premature_prose)
            control("state-form test status follows coverage-family state",
                    _break_state_form_test_status_coupling)
            control("state-form Part V status follows coverage-family state",
                    _break_state_form_part_v_coupling)
            control("state-form formal statement order is checker-owned",
                    _break_state_form_statement_ref)
            control("state-form pin group reference is checker-owned",
                    _break_state_form_pin_group_ref)
            control("state-form counterfactual reference is checker-owned",
                    _break_state_form_counterfactual_ref)
            control("state-form per-card negative anchor is checker-owned",
                    _break_state_form_power_negative_ref)
            control("state-form per-card counterfactual anchor is checker-owned",
                    _break_state_form_power_counterfactual_ref)
            control("formal statements are assigned exactly once",
                    lambda s: s["coverage_families"][0][
                        "formal_statement_refs"].pop())
            control("other powers cannot borrow the retained T3 record",
                    _remove_t3_borrowing_wall)
            control("Book 2 routing is not a constitutional coverage family",
                    lambda s: s["coverage_families"][2].update(
                        {"source_family_refs": ["book-2-operation"]}))
        control("unknown power holder body is refused",
                lambda s: s["powers"][0]["holder_body_refs"].__setitem__(
                    0, "FS-BOD-999"))
        control("state-form power holder semantics are checker-owned",
                lambda s: next(
                    row for row in s["powers"]
                    if row["id"] == "FS-POW-023"
                ).update({
                    "holder_body_refs": ["FS-BOD-02"],
                    "holder_role_refs": ["FS-ROL-26"],
                }))
        control("delegated state-form holders cannot choose mechanics",
                lambda s: next(
                    row for row in s["powers"]
                    if row["id"] == "FS-POW-028"
                )["contract_terms"]["lawful_source"].update({
                    "text": "The holder arrays choose the appointment."
                }))
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
        control("state-form function allocation semantics are checker-owned",
                lambda s: next(
                    row for row in s["function_allocations"]
                    if row["power_ref"] == "FS-POW-023"
                ).update({
                    "decider_body_refs": ["FS-BOD-02"],
                    "decider_role_refs": ["FS-ROL-26"],
                }))
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
    control("empty array without deferral is refused",
            lambda s: s.update({"thresholds": []}),
            "thresholds is empty with no deferral record",
            validate_deferred)
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
                {"verdict": (
                    VERDICT_PASSED if s.get("closure_record")
                    else VERDICT_NOT_PASSED).lower()}))
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
    control(
        "a closure record cannot bypass closure-derived acceptance",
        _closure_while_unmet,
        "derived exactly from closure-record presence",
    )
    control("a closure record gate is exact",
            _closure_wrong_gate, ".gate must be gate-a")
    control("a closure record claim is byte-exact",
            _closure_wrong_claim, "permitted_claim must be byte-exact")
    control("a closure candidate is an immutable Git id",
            _closure_bad_candidate, "must be a Git id")
    control("a closure source version is current",
            _closure_stale_source, "must match the closure source")
    control("a closure scope digest is current",
            _closure_stale_scope, "must match the semantic scope")
    control("a closure record must cite FS-ENV-01",
            _closure_env_stub, "must be FS-ENV-01")
    control("a closure record requires a current-source repository audit",
            _closure_unknown_audit, "current-source repository audit")
    current_audits = qualifying_scope_audits(src)
    detailed_closure_receipt = (
        src["source_version"] == LEGACY_V1_CLOSURE_SOURCE
        or any("verification_receipt_ref" in row for row in current_audits)
    )
    if detailed_closure_receipt:
        control("a closure cutoff must match its repository audit",
                _closure_wrong_cutoff, "must equal audit execution")
        control("closure assurance refs are checker-derived",
                _closure_wrong_assurance, "checker-derived set")
        control("closure residual refs are checker-derived",
                _closure_wrong_residuals, "derived set")
        control("closure claim limitations bind residuals exactly",
                _closure_wrong_limitations, "bind every derived residual")
        if src["source_version"] != LEGACY_V1_CLOSURE_SOURCE:
            control("closure verification uses a content-addressed v2 receipt",
                    _closure_bad_v2_receipt_ref, "content-addressed")
            control("audit and closure bind the same v2 receipt",
                    _closure_mismatched_v2_receipt_ref,
                    "same verification receipt")
            control("v2 closure refuses an inline v1 downgrade",
                    _closure_v1_downgrade, "missing keys")
        else:
            control("closure verification binds the candidate",
                    _closure_wrong_verifier_candidate,
                    "must match closure candidate")
            control("closure verification runs the exact command chain",
                    _closure_wrong_verifier_commands, "required verifier chain")
            control("closure verification result is exact",
                    _closure_wrong_verifier_result, "must be all-passed")
            control("closure verification transcript is digest-bound",
                    _closure_bad_transcript, "is malformed")
        control("closure policy is checker-owned",
                _closure_bad_policy, "must equal the checker-owned policy")
    control("R7 cannot be marked unbuilt after its checks land",
            _r7_unbuilt, "repository-audit state")
    control("R7 cannot be relabelled available",
            _r7_available, "repository-audit state")
    control("the current scope audit binds the source version",
            _scope_audit_stale_source, "current-source repository audit")
    control("the current scope audit binds the semantic scope digest",
            _scope_audit_stale_scope, "current-source repository audit")
    control("the current scope audit binds the protocol digest",
            _scope_audit_stale_protocol, "current-source repository audit")
    control("the current scope audit covers every criterion",
            _scope_audit_criterion_gap, "criterion coverage must be exact")
    control("the current scope audit binds exact checker controls",
            _scope_audit_control_drift, "control references must be exact")
    control("the current scope audit binds the command chain",
            _scope_audit_command_drift, "command chain must be exact")
    control("the current scope audit covers Gate A findings",
            _scope_audit_finding_gap, "finding references must cover Gate A")
    control("the current scope audit result token is exact",
            _scope_audit_result_drift, "is not an audit result")
    control("the current scope audit preserves its evidence ceiling",
            _scope_audit_ceiling_drift, "evidence ceiling must be byte-exact")
    control("the current scope audit binds the mechanical closure policy",
            _scope_audit_policy_drift, "policy basis must be checker-owned")
    control("the current scope audit cannot depend on an author act",
            _scope_audit_author_dependency,
            "current audit may not depend on an author act")
    control("a passed event outcome is derived, not prose",
            _event_passed_with_failed_seed, "outcome_status must be derived")
    control("a terminal event requires its commission",
            _event_without_commission, "names no commission")
    control("a current commission binds the semantic scope digest",
            _commission_stale_scope, "scope digest is stale")
    control("a current commission binds the exact protocol digest",
            _commission_stale_protocol, "protocol digest is stale")
    control("plant and seed commitments are distinct",
            _commission_duplicate_digests, "must be distinct")
    control("commission chronology is canonical UTC and ordered",
            _commission_postdates_window, "commissioned < open")
    control("commission windows are structured UTC",
            _commission_bad_window, "canonical UTC")
    control("the packet manifest is exact and ordered",
            _commission_bad_packet, "exact ordered packet manifest")
    control("the panel contains at least two disciplines",
            _commission_one_discipline, "at least two disciplines")
    control("reviewer criteria collectively cover the full rubric",
            _commission_criterion_gap, "criterion union")
    control("a reviewer cannot be Darshu, Dhanush, or custodian",
            _commission_reviewer_conflict, "reviewer conflicts")
    control("reviewer conflict attestations are exact",
            _commission_missing_conflict, "missing keys")
    control("findings-contingent reviewer compensation is refused",
            _commission_contingent_pay, "findings-contingent")
    control("the frozen intake equals the event proposal set",
            _event_intake_mismatch, "must equal the event proposal set")
    control("the frozen intake digest binds proposal payloads",
            _event_bad_intake_digest, "does not match")
    control("controls cannot reveal early",
            _event_early_reveal, "may not reveal before cutoff")
    control("every proposal is received inside the window",
            _proposal_outside_window, "outside its window")
    control("proposal chronology is ordered",
            _proposal_check_before_triage, "chronology must be")
    control("every proposal receives Darshu triage",
            _proposal_missing_darshu, "requires Darshu triage")
    control("every proposal receives Dhanush checking",
            _proposal_missing_dhanush, "requires Dhanush checking")
    control("classification maps exactly to its outward disposition",
            _proposal_bad_classification_mapping, "mapping must be exact")
    control("an added proposal names resolvable created records",
            _proposal_added_unresolvable, "reference target missing")
    control("a retained limit links its exact defect binding",
            _proposal_retained_without_defect, "must link a defect row")
    control("failed events with proposal intake cannot be deleted",
            _delete_failed_event, "names no review event")
    control("the rubric status is exact in both states",
            lambda s: s["severity_rubric"].update(
                {"rubric_status": "confirmed"}))
    control("a confirmed rubric records its basis",
            _confirmed_rubric_without_basis)
    control("review_protocol must be present",
            lambda s: s.pop("review_protocol"), "must be present")
    control("the amended protocol status is exact",
            lambda s: s["review_protocol"].update(
                {"protocol_status": "confirmed"}), "repository-enforced status")
    control("the amended protocol binds its mechanical policy",
            lambda s: s["review_protocol"].pop("policy_basis"),
            "missing keys")
    control("the protocol mode is repository-adversarial",
            lambda s: s["review_protocol"].update({"mode": "panel-review"}),
            "must be repository-adversarial-audit")
    control("external review is explicitly optional",
            lambda s: s["review_protocol"].update(
                {"external_review_policy": "required"}),
            "must be optional-non-gating")
    control("the legacy designation is retired as a gate dependency",
            lambda s: s["review_protocol"]["designation"].update(
                {"designation_status": "active"}),
            "must retire the external dependency")
    control("optional-review owner and checker remain distinct people",
            _designation_owner_is_checker, "distinct people")
    control("an optional-review custodian may not triage or check",
            _designation_owner_is_custodian, "may not triage or check")
    control("the protocol status line is live-checked",
            lambda s: s["review_protocol"].update(
                {"status_line_ref": _PROTOCOL_NEEDLE}),
            "status and document status line")
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
    control("a closure record's envelope must be the structural envelope",
            _closure_envelope_missing, "must be FS-ENV-01")
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

    # ── the bodies specification: sabotage first, trust after ────────────────
    # Each control below is the falsification condition for one clause of the
    # ratified mandate. A clause with no watched-failing control is a clause
    # the checker does not actually hold.
    control("a body separates all seven status senses",
            lambda s: s["bodies"][0]["status_senses"].pop("franchise"),
            "missing keys")
    control("a body carries the whole office contract",
            lambda s: s["bodies"][0]["office_contract"].pop("succession"),
            "missing keys")
    control("a body kind is a declared kind",
            lambda s: s["bodies"][0].update({"body_kind": "ministry"}),
            "body_kind is invalid")
    control("a body's rendered source is one of its card sources",
            _body_source_not_in_sources,
            "card sources must contain the rendered source")
    control("a body term source must be a card source",
            _body_term_foreign_source, "term source must be a card source")
    control("the office senses do not reuse the word standing",
            _body_office_sense_says_standing,
            "standing is reserved for universal personhood")
    control("the ordinary function expands the job rather than copying it",
            _body_ordinary_function_copies_job, "must expand the job")
    control("a body may not check itself",
            _body_checks_itself, "may not check itself")
    control("a body names at least one external checker",
            _body_without_external_checker, "needs at least one checking body")
    control("a body's accountability routes are typed and duplicate-free",
            _body_duplicate_route_type, "duplicate-free")
    control("an enumerated adverse determination carries an appeal",
            _body_adverse_without_appeal, "missing keys")
    control("an enumerated adverse determination carries a remedy",
            _body_adverse_blank_remedy, "non-empty string")
    control("a body claiming no adverse determination lists none",
            _body_none_by_design_with_items, "none-by-design lists no items")
    control("an enumerated adverse determination names one",
            _body_enumerated_without_items, "needs at least one item")
    control("only custodial execution applies the retained custody contract",
            _body_borrows_custody_t3, "custody T3 is not reusable")
    control("a body temporal contract may not cite the custody clock source",
            _body_cites_custody_source, "custody T3 is not reusable")
    control("a blocked mechanic is filled",
            _body_drops_blocked_mechanic, "a blocked mechanic must be filled")
    control("a delegated mechanic declares its choice owner and bounds",
            _body_mechanic_not_delegated, "bounded delegation")
    control("a body card may not assert an arrival",
            _body_asserts_arrival, "arrival register is refused")
    control("a body card may not carry an aggregate figure",
            _body_asserts_aggregate, "aggregate figure is refused")
    control("a body card may not carry a feasibility claim",
            _body_asserts_feasibility, "feasibility claim is refused")
    control("a body card may not relabel a current fixture",
            _body_relabels_fixture, "may not relabel a current fixture")
    control("the advocate declares its non-substitution boundary",
            _body_advocate_boundary_blank, "must be a non-empty string")
    control("a body cell may not paraphrase the map row it cites",
            _body_cell_paraphrases_map, "drifted from the coverage map row")
    control("every required-bodies row still binds a card",
            _body_unbinds_a_map_row, "every required-bodies row must bind")
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

    # A semantic mutant appends its own transient current audit so committed
    # audit history remains an exact prefix while readiness is exercised
    # against the mutant's semantic scope.

    gate_a_critical = copy.deepcopy(src)
    gate_a_critical["closure_record"] = None
    gate_a_critical["acceptance_gate"].update({
        "verdict": VERDICT_NOT_PASSED,
        "gate_a_status": "not-passed",
    })
    gate_a_row = next(
        row for row in gate_a_critical["defects"] if row["id"] == "FS-DFT-27"
    )
    gate_a_row["severity"] = (
        "critical — semantic control for a scope-map defect"
    )
    control_audit = _transient_control_audit(
        gate_a_critical, "Semantic-control current audit")
    control_audit["scope_sha256"] = review_scope_digest(gate_a_critical)
    gate_a_critical["scope_audits"].append(control_audit)
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
    executed = set()
    validation_entries = {}
    for entry in controls:
        name, mutate, expect, validator = entry
        if (src["coverage_population"]["status"] != "complete"
                and getattr(mutate, "__name__", "").startswith(
                    ("_closure", "_scope_audit"))):
            continue
        if name in executed:
            raise LedgerError(
                f"negative control registered or executed twice: {name}")
        executed.add(name)
        mutant = copy.deepcopy(src)
        is_closure_mutation = getattr(
            mutate, "__name__", "").startswith("_closure")
        if mutant.get("scope_audits") and not is_closure_mutation:
            control_audit = _transient_control_audit(
                mutant, "Watched-mutation current audit")
            mutant["scope_audits"].append(control_audit)
        if not is_closure_mutation:
            mutant["closure_record"] = None
            mutant["acceptance_gate"].update({
                "verdict": VERDICT_NOT_PASSED,
                "gate_a_status": "not-passed",
            })
        mutate(mutant)
        if (name != "the current scope audit binds the semantic scope digest"
                and mutant.get("scope_audits")):
            mutant["scope_audits"][-1]["scope_sha256"] = \
                review_scope_digest(mutant)
        validation_entries[name] = validation_entries.get(name, 0) + 1
        try:
            (validator or validate)(mutant)
        except LedgerError as exc:
            if expect is not None and expect not in str(exc):
                raise LedgerError(
                    f"negative control failed for the wrong reason: {name} — "
                    f"expected {expect!r} in {exc}"
                )
            passed += 1
            continue
        raise LedgerError(f"negative control failed to fail: {name}")
    if (
            passed != len(executed)
            or set(validation_entries) != executed
            or any(count != 1 for count in validation_entries.values())):
        raise LedgerError(
            "not every active watched control executed exactly once")
    return passed + 3


def _blank_first_power_profile(s):
    power = s["powers"][0]
    profile = power["profiles"][0]
    if "profile_terms" in power:
        field = next(iter(power["profile_terms"][profile]))
        power["profile_terms"][profile][field]["text"] = "N/A"
    else:
        field = next(iter(power["profile_contracts"][profile]))
        power["profile_contracts"][profile][field] = "N/A"


def _coverage_regain_deferral(s):
    s["deferred_populations"].append({
        "record_type": COVERAGE_DEFERRAL_TYPE,
        "owner_ref": _CONTROL_NEEDLE,
        "closure_condition": "control",
        "stage": "control",
    })


def _duplicate_contract_prose(s):
    s["powers"][1]["contract_terms"]["lawful_source"]["text"] = (
        s["powers"][0]["contract_terms"]["lawful_source"]["text"])


def _contract_term_without_source(s):
    s["powers"][0]["contract_terms"]["lawful_source"]["source_refs"] = []


def _incomplete_bounded_delegation(s):
    term = next(
        term for power in s["powers"]
        for term in power["contract_terms"].values()
        if term["basis"] == "bounded-delegation")
    term.pop("failure_default")


def _planned_test_claims_execution(s):
    power = next(
        row for row in s["powers"]
        if row["negative_test"]["status"] == "planned")
    power["negative_test"]["status"] = "executable"


def _premature_formalization(s):
    family = next(
        row for row in s["coverage_families"]
        if row["state"] == "coverage-ready")
    family["state"] = "formalized"


def _premature_prose(s):
    family = next(
        row for row in s["coverage_families"]
        if row["state"] == "coverage-ready")
    family["prose_refs"] = ["book-1/01-what-counts-as-evidence.md"]


def _break_state_form_test_status_coupling(s):
    family = next(
        row for row in s["coverage_families"]
        if row["id"] == "FS-CVF-003")
    power = next(
        row for row in s["powers"]
        if row["id"] == family["card_refs"][0])
    power["negative_test"]["status"] = (
        "planned"
        if power["negative_test"]["status"] == "executable"
        else "executable"
    )


def _break_state_form_part_v_coupling(s):
    family = next(
        row for row in s["coverage_families"]
        if row["id"] == "FS-CVF-003")
    power = next(
        row for row in s["powers"]
        if row["id"] == family["card_refs"][0])
    power["part_v_status"] = (
        "coverage-only-not-formalized"
        if power["part_v_status"] != "coverage-only-not-formalized"
        else "formalized-not-prose-landed"
    )


def _state_form_family(s):
    return next(
        row for row in s["coverage_families"]
        if row["id"] == "FS-CVF-003")


def _break_state_form_statement_ref(s):
    _state_form_family(s)["formal_statement_refs"].reverse()


def _break_state_form_pin_group_ref(s):
    replacement = next(
        row for row in s["coverage_families"]
        if row["id"] == "FS-CVF-011")["pin_group_refs"][0]
    _state_form_family(s)["pin_group_refs"][0] = replacement


def _break_state_form_counterfactual_ref(s):
    replacement = next(
        row for row in s["coverage_families"]
        if row["id"] == "FS-CVF-011")["counterfactual_refs"][0]
    _state_form_family(s)["counterfactual_refs"][0] = replacement


def _break_state_form_power_negative_ref(s):
    family = _state_form_family(s)
    first, second = (
        next(row for row in s["powers"] if row["id"] == power_ref)
        for power_ref in family["card_refs"][:2]
    )
    first["negative_test"]["executable_ref"] = (
        second["negative_test"]["executable_ref"])


def _break_state_form_power_counterfactual_ref(s):
    family = _state_form_family(s)
    first, second = (
        next(row for row in s["powers"] if row["id"] == power_ref)
        for power_ref in family["card_refs"][:2]
    )
    first["counterfactual"]["executable_ref"] = (
        second["counterfactual"]["executable_ref"])


def _remove_t3_borrowing_wall(s):
    power = next(
        row for row in s["powers"]
        if row["manifest_key"] != RETAINED_FORMAL_KEY)
    power["prohibited_inputs"] = [
        value for value in power["prohibited_inputs"]
        if "formal-active-custody" not in value
    ]


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


# ── bodies-specification control mutations ───────────────────────────────────
# A foreign but real reference, used to prove the term-source subset rule.
_FOREIGN_REF = (
    "new-book-plans/full-society-scope-review-protocol.md::"
    "## 5. Mechanical Gate A closure"
)
_CUSTODY_REF = (
    "new-book-plans/book-1-time-model-decision.md::# Book 1 Time-Model Decision"
)


def _first_body(s, predicate, what):
    for rec in s["bodies"]:
        if predicate(rec):
            return rec
    raise LedgerError(f"control setup: no body {what}")


def _a_body_term(rec):
    """One term object from a card, for the register controls."""
    return rec["status_senses"]["universal_human_standing"]


def _body_source_not_in_sources(s):
    rec = _first_body(s, lambda r: len(r["source_refs"]) > 1,
                      "carries more than one card source")
    rec["source_refs"] = [r for r in rec["source_refs"]
                          if r != rec["source_ref"]]


def _body_term_foreign_source(s):
    # A real, resolvable reference the CARD does not carry: only the subset
    # rule can object, which is exactly the rule under test.
    term = _a_body_term(s["bodies"][0])
    term["source_refs"] = term["source_refs"] + [_FOREIGN_REF]


def _body_office_sense_says_standing(s):
    sense = s["bodies"][0]["status_senses"]["current_office"]
    sense["text"] = sense["text"] + " This is the body's standing."


def _body_ordinary_function_copies_job(s):
    rec = s["bodies"][0]
    rec["office_contract"]["ordinary_function"]["text"] = rec["job"]


def _body_checks_itself(s):
    rec = s["bodies"][0]
    rec["accountability_routes"][0]["checker_body_refs"] = [rec["id"]]


def _body_without_external_checker(s):
    for route in s["bodies"][0]["accountability_routes"]:
        route["checker_body_refs"] = []


def _body_duplicate_route_type(s):
    routes = s["bodies"][0]["accountability_routes"]
    routes.append(copy.deepcopy(routes[0]))


def _enumerated_body(s):
    return _first_body(s,
                       lambda r: r["adverse_determinations"]["kind"] == "enumerated",
                       "enumerates an adverse determination")


def _body_adverse_without_appeal(s):
    _enumerated_body(s)["adverse_determinations"]["items"][0].pop("appeal")


def _body_adverse_blank_remedy(s):
    _enumerated_body(s)["adverse_determinations"]["items"][0]["remedy"]["text"] = ""


def _body_none_by_design_with_items(s):
    donor = _enumerated_body(s)["adverse_determinations"]["items"][0]
    target = _first_body(
        s, lambda r: r["adverse_determinations"]["kind"] == "none-by-design",
        "claims no adverse determination")
    target["adverse_determinations"]["items"] = [copy.deepcopy(donor)]


def _body_enumerated_without_items(s):
    _enumerated_body(s)["adverse_determinations"]["items"] = []


def _body_borrows_custody_t3(s):
    rec = _first_body(s, lambda r: r["id"] != CUSTODY_T3_APPLICANT,
                      "other than the custodial executor")
    rec["temporal_contract"]["custody_t3_relation"] = "retained-application"


def _body_cites_custody_source(s):
    rec = _first_body(s, lambda r: r["id"] != CUSTODY_T3_APPLICANT,
                      "other than the custodial executor")
    rec["source_refs"] = rec["source_refs"] + [_CUSTODY_REF]
    term = rec["temporal_contract"]["term"]
    term["source_refs"] = term["source_refs"] + [_CUSTODY_REF]


def _body_drops_blocked_mechanic(s):
    rec = _first_body(s, lambda r: r["id"] in BODY_DELEGATED_MECHANICS_REQUIRED,
                      "carries a blocked mechanic")
    rec["delegated_mechanics"] = []


def _body_mechanic_not_delegated(s):
    rec = _first_body(s, lambda r: bool(r["delegated_mechanics"]),
                      "carries a delegated mechanic")
    rec["delegated_mechanics"][0]["basis"] = "source-specified"


def _body_asserts_arrival(s):
    term = _a_body_term(s["bodies"][0])
    term["text"] = term["text"] + " The remedy is delivered."


def _body_asserts_aggregate(s):
    term = _a_body_term(s["bodies"][0])
    term["text"] = term["text"] + " 5 of 7 are established."


def _body_asserts_feasibility(s):
    term = _a_body_term(s["bodies"][0])
    term["text"] = term["text"] + " The design is feasible."


def _body_relabels_fixture(s):
    term = _a_body_term(s["bodies"][0])
    term["text"] = term["text"] + " Read Convocation as the Executive Council."


def _body_cell_paraphrases_map(s):
    # the exact edit that happened: a cell reworded to say almost the same
    rec = _first_body(s, lambda r: r["id"] == "FS-BOD-05",
                      "is the Civic President")
    rec["required_check"] = rec["required_check"].replace(
        " and cross-body confirmation", "")


def _body_unbinds_a_map_row(s):
    rec = _first_body(s, lambda r: r["id"] == "FS-BOD-05",
                      "is the Civic President")
    rec["title"] = "Ceremonial President"


def _body_advocate_boundary_blank(s):
    rec = _first_body(s, lambda r: r["id"] == "FS-BOD-20",
                      "is the rights advocate")
    rec["office_contract"]["delegation_boundary"]["text"] = " "


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


def _mk_commission(s):
    if s.get("review_commissions"):
        return s["review_commissions"][-1]
    split = len(REVIEW_CRITERIA) // 2
    reviewers = [
        {
            "identity": "control-reviewer-a",
            "discipline": "constitutional-law",
            "criterion_refs": list(REVIEW_CRITERIA[:split]),
            "consent_attestation": REVIEWER_CONSENT,
            "conflict_attestation": REVIEWER_CONFLICT_CLEAR,
            "compensation_attestation": REVIEWER_COMPENSATION_CLEAR,
        },
        {
            "identity": "control-reviewer-b",
            "discipline": "systems-safety",
            "criterion_refs": list(REVIEW_CRITERIA[split:]),
            "consent_attestation": REVIEWER_CONSENT,
            "conflict_attestation": REVIEWER_CONFLICT_CLEAR,
            "compensation_attestation": REVIEWER_COMPENSATION_CLEAR,
        },
    ]
    rec = {
        "id": "FS-COM-99",
        "title": "control commission",
        "source_version": s["source_version"],
        "scope_sha256": review_scope_digest(s),
        "protocol_sha256": hashlib.sha256(_input_bytes(PROTOCOL_DOC)).hexdigest(),
        "plant_commitment_sha256": "a" * 64,
        "seed_commitment_sha256": "b" * 64,
        "commissioned_at_utc": "2026-08-14T00:00:00Z",
        "received_window": {
            "opens_at_utc": "2026-08-15T00:00:00Z",
            "closes_at_utc": "2026-08-16T00:00:00Z",
        },
        "cutoff_at_utc": "2026-08-17T00:00:00Z",
        "custodian_identity": s["review_protocol"]["designation"]["custodian"],
        "reviewers": reviewers,
        "criterion_coverage": list(REVIEW_CRITERIA),
        "packet_paths": list(REVIEW_PACKET_PATHS),
    }
    s["review_commissions"].append(rec)
    return rec


def _base_proposal(s, *, pid, source_kind, source_identity, finding,
                   classification, disposition, control_kind,
                   severity=None):
    rec = {
        "id": pid,
        "title": f"control proposal {pid}",
        "proposal": f"control proposal payload {pid}",
        "source_kind": source_kind,
        "source_identity": source_identity,
        "received_at_utc": "2026-08-15T12:00:00Z",
        "triaged_at_utc": "2026-08-16T01:00:00Z",
        "severity_owner_identity": s["review_protocol"]["designation"][
            "severity_owner"],
        "materiality_finding": finding,
        "materiality_reason": "control materiality reason",
        "classification": classification,
        "checked_at_utc": "2026-08-16T02:00:00Z",
        "independent_checker_identity": s["review_protocol"]["designation"][
            "independent_checker"],
        "check_finding": "confirmed",
        "check_reason": "control checking reason",
        "proposal_disposition": disposition,
        "disposition_at_utc": "2026-08-16T03:00:00Z",
        "reasons": "control disposition reason",
        "review_event_ref": "FS-REV-99",
        "control_kind": control_kind,
    }
    if severity is not None:
        rec["severity"] = severity
    if classification in UNESTABLISHED_DISPOSITIONS:
        rec["routed_unestablished_disposition"] = classification
    return rec


def _mk_event(s, passed=False):
    commission = _mk_commission(s)
    defect = s["defects"][0]
    plant = _base_proposal(
        s, pid="FS-PRO-97", source_kind="reviewer",
        source_identity=commission["reviewers"][0]["identity"],
        finding="material", severity=severity_class(defect),
        classification="retained-limit", disposition="retained-limit",
        control_kind="plant-match")
    plant["defect_row_ref"] = defect["id"]
    plant["retained_limit_binding"] = _expected_retained_binding(s, defect)
    seed_material = _base_proposal(
        s, pid="FS-PRO-98", source_kind="seed",
        source_identity="committed-seed-control", finding="material",
        severity="material", classification="routed-book-2",
        disposition="classified-out", control_kind="seed")
    seed_immaterial = _base_proposal(
        s, pid="FS-PRO-99", source_kind="seed",
        source_identity="committed-seed-control", finding="immaterial",
        classification="immaterial", disposition="classified-out",
        control_kind="seed")
    s["proposals"].extend([plant, seed_material, seed_immaterial])
    seed_results = [
        {
            "proposal_ref": seed_material["id"],
            "expected_materiality": "material",
            "expected_severity": "material" if passed else "critical",
            "expected_disposition": "classified-out",
            "verified_by": s["review_protocol"]["designation"][
                "independent_checker"],
            "verification_reason": "control seed verification",
        },
        {
            "proposal_ref": seed_immaterial["id"],
            "expected_materiality": "immaterial",
            "expected_severity": None,
            "expected_disposition": "classified-out",
            "verified_by": s["review_protocol"]["designation"][
                "independent_checker"],
            "verification_reason": "control seed verification",
        },
    ]
    ev = {
        "id": "FS-REV-99",
        "title": "control terminal event",
        "commission_ref": commission["id"],
        "packet_commit_sha": "c" * 40,
        "source_version": commission["source_version"],
        "scope_sha256": commission["scope_sha256"],
        "protocol_sha256": commission["protocol_sha256"],
        "intake_receipt": {
            "frozen_at_utc": "2026-08-16T00:01:00Z",
            "ordered_proposal_ids": [row["id"] for row in s["proposals"]],
            "manifest_sha256": _proposal_intake_digest(
                "FS-REV-99", s["proposals"]),
        },
        "control_reveal": {
            "revealed_at_utc": "2026-08-17T00:00:00Z",
            "plant_preimage_sha256": commission["plant_commitment_sha256"],
            "seed_preimage_sha256": commission["seed_commitment_sha256"],
            "planted_proposal_ref": plant["id"],
            "seed_results": seed_results,
            "plant_match_checked_by": s["review_protocol"]["designation"][
                "independent_checker"],
            "plant_match_reason": "control plant match",
        },
        "seeded_control": {
            "status": "passed" if passed else "failed",
            "reason": "control seeded outcome",
        },
        "planted_control": {
            "status": "passed",
            "reason": "control planted outcome",
        },
        "outcome_status": "passed" if passed else "failed",
        "outcome_reason": "control terminal outcome",
    }
    s["review_events"].append(ev)
    return ev


def _mk_proposal(s, **overrides):
    _mk_event(s, passed=False)
    rec = s["proposals"][-1]
    rec.update(overrides)
    return rec



def _mk_closure(s):
    resolution = compute_resolution(s)
    residuals = _gate_a_residual_ids(s, resolution)
    audit = s["scope_audits"][-1]
    is_v2 = s["source_version"] != LEGACY_V1_CLOSURE_SOURCE
    receipt_ref = audit.get("verification_receipt_ref")
    if not is_v2:
        audit["result"] = SCOPE_AUDIT_RESULT
    audit["scope_sha256"] = review_scope_digest(s)
    prior = s.get("closure_record")
    candidate_sha = (
        prior.get("candidate_commit_sha") if isinstance(prior, dict)
        else subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    )
    s["closure_record"] = {
        "gate": "gate-a",
        "permitted_claim": GATE_A_PERMITTED_CLAIM,
        "candidate_commit_sha": candidate_sha,
        "source_version": s["source_version"],
        "scope_sha256": review_scope_digest(s),
        "envelope_ref": "FS-ENV-01",
        "audit_cutoff_at_utc": audit["executed_at_utc"],
        "scope_audit_ref": audit["id"],
        "assurance_record_refs": list(GATE_A_ASSURANCE_REFS),
        "residual_refs": residuals,
        "claim_limitations": _gate_a_claim_limitations(s, residuals),
        "closure_policy_ref": SCOPE_AUDIT_POLICY_BASIS,
    }
    if is_v2:
        s["closure_record"]["verification_receipt_ref"] = (
            receipt_ref
            or "new-book-plans/verification-receipts/sha256-"
            + "0" * 64 + ".json"
        )
    else:
        if isinstance(prior, dict) and isinstance(
                prior.get("verification_receipt"), dict):
            receipt = copy.deepcopy(prior["verification_receipt"])
        else:
            receipt = {
                "candidate_commit_sha": candidate_sha,
                "verified_at_utc": "2026-08-18T00:00:00Z",
                "commands": list(REQUIRED_VERIFY_COMMANDS),
                "result": "all-passed",
                "transcript_sha256": "e" * 64,
            }
        s["closure_record"]["verification_receipt"] = receipt
    s["acceptance_gate"]["gate_a_status"] = "passed"
    s["acceptance_gate"]["verdict"] = VERDICT_PASSED
    return s["closure_record"]


def _closure_while_unmet(s):
    _mk_closure(s)
    s["acceptance_gate"]["gate_a_status"] = "not-passed"


def _closure_wrong_gate(s):
    _mk_closure(s)["gate"] = "gate-b"


def _closure_wrong_claim(s):
    _mk_closure(s)["permitted_claim"] = "broader control claim"


def _closure_bad_candidate(s):
    _mk_closure(s)["candidate_commit_sha"] = "candidate"


def _closure_stale_source(s):
    _mk_closure(s)["source_version"] = "stale"


def _closure_stale_scope(s):
    _mk_closure(s)["scope_sha256"] = "0" * 64


def _closure_env_stub(s):
    _mk_closure(s)["envelope_ref"] = ENVELOPE_STUB_ID


def _closure_unknown_audit(s):
    _mk_closure(s)["scope_audit_ref"] = "FS-SAU-98"


def _closure_wrong_cutoff(s):
    _mk_closure(s)["audit_cutoff_at_utc"] = "2026-08-19T00:00:00Z"


def _closure_wrong_assurance(s):
    _mk_closure(s)["assurance_record_refs"].pop()


def _closure_wrong_residuals(s):
    _mk_closure(s)["residual_refs"].append("FS-DFT-999")


def _closure_wrong_limitations(s):
    rec = _mk_closure(s)
    rec["claim_limitations"].append({
        "defect_ref": "FS-DFT-999", "affected_claim_ref": "FS-CLM-01",
        "public_claim_restriction": "control",
    })


def _closure_wrong_verifier_candidate(s):
    _mk_closure(s)["verification_receipt"]["candidate_commit_sha"] = "f" * 40


def _closure_wrong_verifier_commands(s):
    _mk_closure(s)["verification_receipt"]["commands"].pop()


def _closure_wrong_verifier_result(s):
    _mk_closure(s)["verification_receipt"]["result"] = "mostly-passed"


def _closure_bad_transcript(s):
    _mk_closure(s)["verification_receipt"]["transcript_sha256"] = "no"


def _closure_bad_v2_receipt_ref(s):
    rec = _mk_closure(s)
    rec["verification_receipt_ref"] = "not-a-receipt"
    s["scope_audits"][-1]["verification_receipt_ref"] = "not-a-receipt"


def _closure_mismatched_v2_receipt_ref(s):
    _mk_closure(s)["verification_receipt_ref"] = (
        "new-book-plans/verification-receipts/sha256-" + "0" * 64 + ".json"
    )


def _closure_v1_downgrade(s):
    rec = _mk_closure(s)
    rec.pop("verification_receipt_ref")
    rec["verification_receipt"] = {
        "candidate_commit_sha": rec["candidate_commit_sha"],
        "verified_at_utc": "2026-08-23T00:00:00Z",
        "commands": [],
        "result": "all-passed",
        "transcript_sha256": "0" * 64,
    }


def _closure_bad_policy(s):
    _mk_closure(s)["closure_policy_ref"] = _CONTROL_NEEDLE


def _r7_unbuilt(s):
    route = next(row for row in s["routes"] if row["id"] == "FS-RTE-07")
    route.update({"status": "unbuilt", "route_status": "unbuilt"})


def _r7_available(s):
    route = next(row for row in s["routes"] if row["id"] == "FS-RTE-07")
    route.update({"status": "available", "route_status": "available"})


def _scope_audit_stale_source(s):
    s["source_version"] = "control-current-source"
    s["scope_audits"][-1]["source_version"] = "stale-source"


def _scope_audit_stale_scope(s):
    s["title"] += " control semantic drift"
    s["scope_audits"][-1]["scope_sha256"] = "0" * 64


def _scope_audit_stale_protocol(s):
    s["source_version"] = "control-current-source"
    s["scope_audits"][-1]["source_version"] = s["source_version"]
    s["scope_audits"][-1]["protocol_sha256"] = "0" * 64


def _scope_audit_criterion_gap(s):
    s["scope_audits"][-1]["criterion_coverage"].pop()


def _scope_audit_control_drift(s):
    s["scope_audits"][-1]["control_refs"].pop()


def _scope_audit_command_drift(s):
    s["scope_audits"][-1]["commands"].pop()


def _scope_audit_finding_gap(s):
    s["scope_audits"][-1]["finding_refs"].pop()


def _scope_audit_result_drift(s):
    s["scope_audits"][-1]["result"] = "passed"


def _scope_audit_ceiling_drift(s):
    s["scope_audits"][-1]["evidence_ceiling"] = "broader claim"


def _scope_audit_policy_drift(s):
    s["scope_audits"][-1]["policy_basis"] = _CONTROL_NEEDLE


def _scope_audit_author_dependency(s):
    audit = s["scope_audits"][-1]
    audit.pop("policy_basis")
    audit["author_basis"] = "control author dependency"


def _event_passed_with_failed_seed(s):
    event = _mk_event(s, passed=True)
    event["outcome_status"] = "failed"


def _event_without_commission(s):
    _mk_event(s, passed=True)
    s["review_commissions"] = []


def _commission_stale_scope(s):
    _mk_commission(s)["scope_sha256"] = "0" * 64


def _commission_stale_protocol(s):
    _mk_commission(s)["protocol_sha256"] = "0" * 64


def _commission_duplicate_digests(s):
    rec = _mk_commission(s)
    rec["seed_commitment_sha256"] = rec["plant_commitment_sha256"]


def _commission_postdates_window(s):
    _mk_commission(s)["commissioned_at_utc"] = "2026-08-15T01:00:00Z"


def _commission_bad_window(s):
    _mk_commission(s)["received_window"]["opens_at_utc"] = "tomorrow"


def _commission_bad_packet(s):
    _mk_commission(s)["packet_paths"].pop()


def _commission_one_discipline(s):
    rec = _mk_commission(s)
    rec["reviewers"][1]["discipline"] = rec["reviewers"][0]["discipline"]


def _commission_criterion_gap(s):
    _mk_commission(s)["reviewers"][1]["criterion_refs"].pop()


def _commission_reviewer_conflict(s):
    rec = _mk_commission(s)
    rec["reviewers"][0]["identity"] = s["review_protocol"]["designation"][
        "severity_owner"]


def _commission_missing_conflict(s):
    _mk_commission(s)["reviewers"][0].pop("conflict_attestation")


def _commission_contingent_pay(s):
    _mk_commission(s)["reviewers"][0]["compensation_attestation"] = \
        "findings-contingent"


def _event_intake_mismatch(s):
    _mk_event(s, passed=True)["intake_receipt"]["ordered_proposal_ids"].pop()


def _event_bad_intake_digest(s):
    _mk_event(s, passed=True)["intake_receipt"]["manifest_sha256"] = "0" * 64


def _event_early_reveal(s):
    _mk_event(s, passed=True)["control_reveal"]["revealed_at_utc"] = \
        "2026-08-16T02:30:00Z"


def _proposal_outside_window(s):
    _mk_proposal(s, received_at_utc="2026-08-18T00:00:00Z")


def _proposal_check_before_triage(s):
    _mk_proposal(s, checked_at_utc="2026-08-16T00:30:00Z")


def _proposal_missing_darshu(s):
    _mk_proposal(s, severity_owner_identity="someone-else")


def _proposal_missing_dhanush(s):
    _mk_proposal(s, independent_checker_identity="someone-else")


def _proposal_bad_classification_mapping(s):
    _mk_event(s, passed=False)
    row = s["proposals"][1]
    row["routed_unestablished_disposition"] = "external-assumption"


def _proposal_added_unresolvable(s):
    _mk_event(s, passed=False)
    row = s["proposals"][0]
    row["classification"] = "material-omission"
    row["proposal_disposition"] = "added"
    row.pop("defect_row_ref")
    row.pop("retained_limit_binding")
    row["created_record_refs"] = ["bogus-file.md::no such anchor"]


def _proposal_retained_without_defect(s):
    _mk_event(s, passed=False)
    s["proposals"][0].pop("defect_row_ref")


def _delete_failed_event(s):
    _mk_event(s, passed=False)
    s["review_events"] = []


def _mk_designation(s):
    return s["review_protocol"]["designation"]


def _designation_owner_is_checker(s):
    d = _mk_designation(s)
    d["independent_checker"] = d["severity_owner"]


def _designation_owner_is_custodian(s):
    d = _mk_designation(s)
    d["severity_owner"] = d["custodian"]


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
      "a closure record is refused while any row computes unmet. Gate A passes "
      "only when a mechanical closure record binds an immutable, fully verified "
      "candidate with no semantic drift.")
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
      "is tested only against the declared source-derived audit criteria; "
      "no full Cartesian product is attempted, and deliberately omitted "
      "candidates and "
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
    coverage = src["coverage_population"]
    w(f"Power population status: **{population['status']}**. Coverage-contract "
      f"status: **{coverage['status']}**. Completed coverage prefix: "
      f"{', '.join(coverage['completed_source_families']) or 'none'}.")
    w("")
    w(f"Current rows: {len(src['powers'])} FS-POW cards; "
      f"{len(src['power_contract_templates'])} FS-PCT templates; "
      f"{len(src['power_refusals'])} FS-PRF refusals/limits; "
      f"{len(src['power_crosswalk_dispositions'])} FS-PCD formal "
      f"dispositions; {len(src['function_allocations'])} FS-FAL allocations.")
    w("")
    w(f"Evidence ceiling: {coverage['evidence_ceiling']}")
    w("")
    w("| Power | Manifest grain | Class / profiles | Claims / domains | "
      "Contract readiness | Tests | Part V / Book 2 boundary |")
    w("| --- | --- | --- | --- | --- | --- | --- |")
    for rec in src["powers"]:
        summary = rec["contract_terms"]["bounded_effect"]["text"]
        w(f"| {rec['id']} {rec['title']} | {rec['manifest_key']} | "
          f"{rec['primary_class_ref']}; {', '.join(rec['profiles'])} | "
          f"{', '.join(rec['affected_claim_refs'])}; "
          f"{', '.join(rec['domain_refs'])} | coverage-ready — {summary} | "
          f"{rec['negative_test']['status']}/"
          f"{rec['counterfactual']['status']} | "
          f"{rec['part_v_status']}; {rec['book2_handoff']} |")
    if not src["powers"]:
        w("| — | no completed source family | — | — | — | — | — |")
    w("")
    w("Constitutional non-power effects:")
    w("")
    w("| Effect | Class / profiles | Claims / domains | Readiness | Boundary |")
    w("| --- | --- | --- | --- | --- |")
    for rec in src["constitutional_effects"]:
        w(f"| {rec['id']} {rec['title']} | {rec['primary_class_ref']}; "
          f"{', '.join(rec['profiles'])} | "
          f"{', '.join(rec['affected_claim_refs'])}; "
          f"{', '.join(rec['domain_refs'])} | {rec['part_v_status']}; "
          f"{rec['negative_test']['status']}/{rec['counterfactual']['status']} | "
          f"{rec['book2_handoff']} |")
    w("")
    w("Coverage-family drafting gate:")
    w("")
    w("| Family | State | Powers / effects | Formal statements | Drafting block |")
    w("| --- | --- | ---: | ---: | --- |")
    for family in src["coverage_families"]:
        w(f"| {family['id']} {family['title']} | {family['state']} | "
          f"{len(family['card_refs'])} / {len(family['effect_refs'])} | "
          f"{len(family['formal_statement_refs'])} | "
          f"{family['blocked_before_drafting']} |")
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
    w("| Body | Kind | Constitutional job | May not do alone | Required check / remedy |")
    w("| --- | --- | --- | --- | --- |")
    for rec in src["bodies"]:
        w(f"| {rec['id']} {rec['title']} | {rec['body_kind']} | "
          f"{rec['job']} | {rec['may_not_do_alone']} | "
          f"{rec['required_check']} |")
    w("")
    w("### Body contracts")
    w("")
    w("Each card separates the seven status senses the state-form ruling "
      "refused to let one word carry, states the office contract, names who "
      "checks the body, and lists the individualized adverse determinations "
      "it can make. A determination carries an appeal and a remedy; a body "
      "that makes none carries neither, which is how the ruling's refusal to "
      "recreate a universal right of appeal is held mechanically rather than "
      "promised. A reserved choice appears as a bounded delegation with its "
      "owner, bounds, and failure default — never as an invented number. "
      "Every card stays ratified-unimplemented: this is what a body is "
      "constitutionally obliged to do and what withholds its conclusions, "
      "not a record that any body exists, is staffed, is independent in "
      "fact, or has ever acted.")
    w("")
    for rec in src["bodies"]:
        w(f"#### {rec['id']} — {rec['title']} ({rec['body_kind']})")
        w("")
        w(f"- Applicability: {rec['applicability']}")
        w(f"- Status: {rec['status']}; severity {rec['severity']}; "
          f"consequence: {rec['consequence']}")
        w(f"- Owner: `{rec['owner_ref']}`; closure: {rec['closure_condition']}")
        w("- Status senses:")
        for sense in BODY_STATUS_SENSES:
            label = sense.replace("_", " ")
            w(f"    - *{label}* — {rec['status_senses'][sense]['text']}")
        w("- Office contract:")
        for term in BODY_OFFICE_TERMS:
            label = term.replace("_", " ")
            w(f"    - *{label}* — {rec['office_contract'][term]['text']}")
        w("- Accountability routes:")
        for route in rec["accountability_routes"]:
            checkers = ", ".join(route["checker_body_refs"]
                                 + route["checker_role_refs"]) or "none named"
            w(f"    - *{route['route_type']}* ({checkers}) — "
              f"{route['term']['text']}")
        adverse = rec["adverse_determinations"]
        w(f"- Individualized adverse determinations: {adverse['kind']} — "
          f"{adverse['note']['text']}")
        for item in adverse["items"]:
            w(f"    - **{item['name']}** against {item['subject']}. "
              f"Appeal: {item['appeal']['text']} "
              f"Remedy: {item['remedy']['text']}")
        temporal = rec["temporal_contract"]
        w(f"- Temporal contract ({temporal['contract_kind']}; custody T3 "
          f"{temporal['custody_t3_relation']}): {temporal['term']['text']}")
        w(f"    - Failure polarity: {temporal['failure_polarity']['text']}")
        w(f"    - Expiry default: {temporal['expiry_default']['text']}")
        if rec["delegated_mechanics"]:
            w("- Delegated mechanics (bounded, never an invented value):")
            for entry in rec["delegated_mechanics"]:
                w(f"    - {entry['text']} Choice owner: {entry['choice_owner']} "
                  f"Bounds: {entry['bounds']} "
                  f"Failure default: {entry['failure_default']}")
        w(f"- Book 2 handoff: {rec['book2_handoff']}")
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
    w("Historical lawful-allocation gaps resolved by the complete FS-POW population:")
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
    w("## Repository scope audit and optional external review")
    w("")
    rp = src["review_protocol"]
    qualifying = qualifying_review_events(src)
    audits = qualifying_scope_audits(src)
    w(
        "Gate A uses the closed, source-derived repository audit. "
        f"Scope audits: {len(src['scope_audits'])}; current qualifying audits: "
        f"{len(audits)}. The audit binds the semantic scope, protocol, declared "
        "criteria, checker controls, command chain, and Gate-A-applicable "
        "finding set. Its evidence ceiling expressly supplies no independent-"
        "human, reader-response, external-truth, operational, feasibility, "
        "liveness, calibration, or timeless-completeness warrant."
    )
    w("")
    w("| Audit | Source | Scope digest | Executed | Result | Findings |")
    w("| --- | --- | --- | --- | --- | --- |")
    for rec in src["scope_audits"]:
        w(
            f"| {rec['id']} | {rec['source_version']} | "
            f"`{rec['scope_sha256']}` | {rec['executed_at_utc']} | "
            f"{rec['result']} | {', '.join(rec['finding_refs']) or '-'} |"
        )
    w("")
    d = rp["designation"]
    w(
        f"The protocol is bound at `{rp['protocol_ref']}`, status "
        f"{rp['protocol_status']}; mode `{rp['mode']}`; external review policy "
        f"`{rp['external_review_policy']}`; semantic scope digest "
        f"`{review_scope_digest(src)}`. The historical Darshu/Dhanush/custodian "
        f"designation is `{d['designation_status']}` as a project-gate "
        "dependency."
    )
    w("")
    w(
        "External commissions, proposals, and terminal events remain append-only "
        "optional evidence. They do not control R7, Gate A, Gate C, Gate E, or "
        "publication. Their stricter chronology, conflicts, custody, controls, "
        "and public-disposition checks still apply if that optional route is used. "
        f"Commissions: {len(src['review_commissions'])}; proposals: "
        f"{len(src['proposals'])}; terminal events: {len(src['review_events'])}; "
        f"current-source qualifying optional events: {len(qualifying)}."
    )
    w("")
    if src["review_commissions"]:
        w("| Commission | Source | Scope digest | Window | Cutoff | Reviewers |")
        w("| --- | --- | --- | --- | --- | --- |")
        for rec in src["review_commissions"]:
            window = rec["received_window"]
            reviewers = "; ".join(
                f"{row['identity']} ({row['discipline']})"
                for row in rec["reviewers"]
            )
            w(
                f"| {rec['id']} | {rec['source_version']} | "
                f"`{rec['scope_sha256']}` | "
                f"{window['opens_at_utc']} to {window['closes_at_utc']} | "
                f"{rec['cutoff_at_utc']} | {reviewers} |"
            )
        w("")
    if src["review_events"]:
        w("| Event | Commission | Packet commit | Outcome | Current optional |")
        w("| --- | --- | --- | --- | --- |")
        qualifying_ids = {row["id"] for row in qualifying}
        for rec in src["review_events"]:
            w(
                f"| {rec['id']} | {rec['commission_ref']} | "
                f"`{rec['packet_commit_sha']}` | "
                f"{rec['outcome_status']} - {rec['outcome_reason']} | "
                f"{'yes' if rec['id'] in qualifying_ids else 'no'} |"
            )
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
    w(f"Coverage contracts: **{src['coverage_population']['status']}**. "
      "Coverage-ready means source-specific planning is complete; it does not "
      "mean formalized, prose-landed, implemented, or operational.")
    w("")
    w("Coverage-family drafting states:")
    w("")
    for family in src["coverage_families"]:
        w(f"- {family['id']} {family['title']}: {family['state']} — "
          f"{family['blocked_before_drafting']}")
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
        domain_effects = [
            row for row in src["constitutional_effects"]
            if domain_id in row["domain_refs"]
        ]
        w("Source-derived power cards:")
        w("")
        for power in domain_powers:
            w(f"- {power['id']} — {power['title']} "
              f"({power['manifest_key']}); class "
              f"{power['primary_class_ref']}; profiles "
              f"{', '.join(power['profiles'])}; claims "
              f"{', '.join(power['affected_claim_refs'])}; tests "
              f"{power['negative_test']['status']}/"
              f"{power['counterfactual']['status']}; Part V "
              f"{power['part_v_status']}. Contract: "
              f"{power['contract_terms']['bounded_effect']['text']} "
              f"Book 2 boundary: {power['book2_handoff']}")
        if not domain_powers:
            w("- None in the completed source-family prefix.")
        w("")
        w("Constitutional non-power effects:")
        w("")
        for effect in domain_effects:
            w(f"- {effect['id']}  {effect['title']}; class "
              f"{effect['primary_class_ref']}; claims "
              f"{', '.join(effect['affected_claim_refs'])}; tests "
              f"{effect['negative_test']['status']}/"
              f"{effect['counterfactual']['status']}; Part V "
              f"{effect['part_v_status']}. Boundary: "
              f"{effect['book2_handoff']}")
        if not domain_effects:
            w("- None.")
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


def parse_lock_wait_seconds(value: str) -> float:
    """Argparse type for an explicit finite, non-negative lock timeout."""
    try:
        seconds = float(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(
            "lock wait must be a number of seconds") from exc
    if not math.isfinite(seconds) or seconds < 0:
        raise argparse.ArgumentTypeError(
            "lock wait must be finite and non-negative")
    return seconds


@contextlib.contextmanager
def verification_refresh_lock(command_name: str, *, wait_seconds: float = 0.0):
    helper = ROOT / "new-book-plans/verification_lock.py"
    if not helper.is_file():
        raise LedgerError(
            "verification lock helper is missing; command refuses to run")
    try:
        module = load_cached_source_module("verification_lock", helper)
    except Exception as exc:
        raise LedgerError(
            f"cannot load the verification lock helper: {exc}") from exc
    lock_type = getattr(module, "VerificationLock", None)
    if lock_type is None:
        raise LedgerError(
            "verification lock helper exposes no VerificationLock")
    busy_type = getattr(module, "VerificationLockBusy", ())
    try:
        with lock_type(command_name, wait_seconds=wait_seconds):
            yield
    except busy_type as exc:
        print(
            f"verification lock busy: {exc}",
            file=sys.stderr,
        )
        raise SystemExit(getattr(module, "EX_TEMPFAIL", 75)) from exc


def atomic_refresh_and_check(outputs, snapshot):
    """Replace all outputs or restore their exact prior byte/mode set."""
    prepared = []
    replaced = set()
    seen = set()

    def write_temp(path, prefix, payload, mode):
        descriptor, raw_temp = tempfile.mkstemp(
            prefix=f".{path.name}.{prefix}-", dir=path.parent)
        temp_path = pathlib.Path(raw_temp)
        try:
            with os.fdopen(descriptor, "wb") as handle:
                handle.write(payload)
                handle.flush()
                os.fsync(handle.fileno())
            os.chmod(temp_path, mode)
            if temp_path.read_bytes() != payload:
                raise LedgerError(
                    f"temporary {prefix} byte check failed: {path}")
        except BaseException:
            try:
                os.close(descriptor)
            except OSError:
                pass
            try:
                temp_path.unlink()
            except FileNotFoundError:
                pass
            raise
        return temp_path

    def rollback():
        for entry in reversed(prepared):
            path = entry["path"]
            if path not in replaced:
                continue
            if entry["existed"]:
                os.replace(entry["backup"], path)
                entry["backup"] = None
            else:
                try:
                    path.unlink()
                except FileNotFoundError:
                    pass
        for entry in prepared:
            path = entry["path"]
            if entry["existed"]:
                if (
                        path.read_bytes() != entry["original"]
                        or stat.S_IMODE(path.stat().st_mode) != entry["mode"]
                ):
                    raise LedgerError(
                        f"atomic refresh rollback verification failed: {path}")
            elif path.exists():
                raise LedgerError(
                    f"atomic refresh rollback left a new output: {path}")

    try:
        for raw_path, text in outputs:
            path = pathlib.Path(raw_path).resolve()
            if path in seen:
                raise LedgerError(f"duplicate atomic output path: {path}")
            seen.add(path)
            payload = text.encode("utf-8")
            existed = path.exists()
            original = snapshot.read_bytes(path) if existed else None
            mode = snapshot.mode(path) if existed else 0o644
            entry = {
                "path": path,
                "temp": None,
                "payload": payload,
                "mode": mode,
                "existed": existed,
                "original": original,
                "backup": None,
            }
            prepared.append(entry)
            entry["temp"] = write_temp(path, "refresh", payload, mode)
            if existed:
                entry["backup"] = write_temp(
                    path, "backup", original, mode)

        snapshot.assert_metadata_unchanged()
        for entry in prepared:
            if not entry["existed"] and entry["path"].exists():
                raise LedgerError(
                    f"new output appeared before atomic refresh: {entry['path']}")
        for entry in prepared:
            path = entry["path"]
            replaced.add(path)
            os.replace(entry["temp"], path)
            entry["temp"] = None
            if path.read_bytes() != entry["payload"]:
                raise LedgerError(
                    f"refreshed output failed its byte check: {path}")
            if stat.S_IMODE(path.stat().st_mode) != entry["mode"]:
                raise LedgerError(
                    f"refreshed output mode drifted: {path}")
            snapshot.advance_replacement(path, entry["payload"])
        snapshot.assert_unchanged()
    except BaseException as exc:
        try:
            rollback()
        except Exception as rollback_exc:
            raise LedgerError(
                f"atomic refresh failed and rollback failed: {rollback_exc}"
            ) from rollback_exc
        if isinstance(exc, (KeyboardInterrupt, SystemExit, LedgerError)):
            raise
        raise LedgerError(f"atomic refresh failed: {exc}") from exc
    finally:
        for entry in prepared:
            for key in ("temp", "backup"):
                temp_path = entry[key]
                if temp_path is not None:
                    try:
                        temp_path.unlink()
                    except FileNotFoundError:
                        pass


def _run(*, snapshot, check: bool, refresh_and_check: bool):

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
    if check:
        current = _input_text(out_path) if out_path.exists() else ""
        if current != rendered:
            raise LedgerError(f"{OUTPUT} is STALE — rerun without --check")
        current_reader = (
            _input_text(reader_out_path) if reader_out_path.exists() else "")
        if current_reader != rendered_reader:
            raise LedgerError(
                f"{READER_OUTPUT} is STALE — rerun without --check"
            )
        if _input_text(map_path) != spliced_map:
            raise LedgerError(
                f"{COVERAGE_MAP} generated region is STALE — rerun without "
                "--check"
            )
        snapshot.assert_unchanged()
        print(f"{OUTPUT}, {READER_OUTPUT}, and the coverage-map region are "
              f"current; {controls} structural negative controls pass; "
              "enum-mapping and "
              "residual-coverage closures over the seven reviewed sources hold; "
              "routing inventory only — nothing established beyond each row's "
              "own posture")
    elif refresh_and_check:
        atomic_refresh_and_check(
            [
                (out_path, rendered),
                (reader_out_path, rendered_reader),
                (map_path, spliced_map),
            ],
            snapshot,
        )
        print(
            f"refreshed and checked {OUTPUT}, {READER_OUTPUT}, and the "
            f"coverage-map region; {controls} structural negative controls "
            "pass; enum-mapping and residual-coverage closures over the seven "
            "reviewed sources hold; routing inventory only — nothing "
            "established beyond each row's own posture"
        )
    else:
        out_path.write_text(rendered, encoding="utf-8")
        reader_out_path.write_text(rendered_reader, encoding="utf-8")
        map_path.write_text(spliced_map, encoding="utf-8")
        print(f"wrote {OUTPUT}, {READER_OUTPUT}, and the coverage-map region; "
              f"{controls} structural negative controls pass")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--check", action="store_true",
                       help="validate and confirm the report is current")
    modes.add_argument(
        "--refresh-and-check",
        action="store_true",
        help="atomically refresh outputs and confirm their bytes under the lock",
    )
    parser.add_argument(
        "--wait-for-lock",
        type=parse_lock_wait_seconds,
        default=None,
        metavar="SECONDS",
        help="wait up to SECONDS for the generation/refresh lock",
    )
    args = parser.parse_args()
    if args.check and args.wait_for_lock is not None:
        parser.error("--wait-for-lock is unavailable with lock-free --check")
    command_name = (
        "script-13-refresh" if args.refresh_and_check
        else "script-13-check" if args.check
        else "script-13-generate"
    )
    snapshot = ImmutableRepositoryInputs(ROOT)
    install_immutable_input_snapshot(snapshot)
    _input_bytes(pathlib.Path(__file__))
    lock_context = (
        contextlib.nullcontext() if args.check
        else verification_refresh_lock(
            command_name,
            wait_seconds=args.wait_for_lock or 0.0,
        )
    )
    with lock_context:
        _run(
            snapshot=snapshot,
            check=args.check,
            refresh_and_check=args.refresh_and_check,
        )


if __name__ == "__main__":
    try:
        main()
    except LedgerError as exc:
        print(f"13-full-society-ledger: {exc}", file=sys.stderr)
        sys.exit(1)
