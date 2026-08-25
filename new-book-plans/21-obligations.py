#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate and verify FS-CVF-016 obligations and non-reciprocity."""

from __future__ import annotations

import argparse
from concurrent.futures import FIRST_COMPLETED, ThreadPoolExecutor, wait
from dataclasses import dataclass
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
from typing import Iterable, Sequence

from verification_lock import (
    EX_TEMPFAIL,
    VerificationLock,
    VerificationLockBusy,
    VerificationLockError,
)


ROOT = pathlib.Path(__file__).resolve().parents[1]
CONSTITUTION = ROOT / "new-book-plans" / "constitution.nibli"
LEDGER = ROOT / "new-book-plans" / "full-society-ledger.json"
OBLIGATIONS_PINS = ROOT / "new-book-plans" / "obligations.pins.nibli"
COUNTERFACTUAL_DIR = ROOT / "new-book-plans" / "counterfactual"
INDEPENDENCE_CF = COUNTERFACTUAL_DIR / "no-obligations-independent-source-review.nibli"
SOURCE_CF = COUNTERFACTUAL_DIR / "no-obligations-source.nibli"
READER_CF = COUNTERFACTUAL_DIR / "no-obligations-finding-reader.nibli"
DEFAULT_PIN = ROOT.parent / "nibli" / "target" / "release" / "nibli-pin"
EXECUTE_WORKERS = 4

DERIVED_BEGIN = "# <OBLIGATIONS-DERIVED-BEGIN>"
DERIVED_END = "# <OBLIGATIONS-DERIVED-END>"
RULES_BEGIN = "# <OBLIGATIONS-RULES-BEGIN>"
RULES_END = "# <OBLIGATIONS-RULES-END>"
DERIVED_STATEMENT = 'derived_only("obliged").'
CURRENT_REVIEW_GUARD = "~($source = $record_review)"
VARIABLE_PATTERN = re.compile(r"\$[a-z][a-z0-9_]*")
QUANTIFIER_PREFIX = re.compile(r"^(?:all \$[a-z][a-z0-9_]*: )+")

MAIN_HEADER = "# Obligations and non-reciprocity family - executable coverage pins"
INDEPENDENCE_HEADER = "# Counterfactual: obligations source writer serves as record reviewer"
SOURCE_HEADER = "# Counterfactual: the obligation-origin materialization rule is removed"
READER_HEADER = "# Counterfactual: the typed finding-reader bridge is removed"

MODES = (
    "PublicObligationBearerMode",
    "DelegatedObligationBearerMode",
    "PrivateObligationBearerMode",
)
CLASSES = (
    "PersonDutyClass",
    "ClassNineCommonDutyClass",
    "RoleDutyClass",
    "VoluntaryDutyClass",
)
FINDING_KINDS = (
    "Placement",
    "Isolation",
    "StatusConflict",
    "CarryOmission",
    "CarryForgery",
    "ClearOmission",
    "ClearForgery",
    "StandingOmission",
    "RecordDisappearance",
    "MaturityDispute",
    "OrderConflict",
    "TemporalRecord",
    "TemporalAuthority",
    "TemporalDispute",
)
LEGACY_READER = {
    "Placement": "Review",
    "Isolation": "Review",
    **{kind: "Appeals" for kind in FINDING_KINDS[2:]},
}


@dataclass(frozen=True)
class Effect:
    number: int
    key: str
    title: str
    mode: str
    duty_class: str
    duty_kind: str


EFFECTS = (
    Effect(198, "public-respect-duty", "Public respect duty", MODES[0], CLASSES[0], "PublicRespectDutyKind"),
    Effect(199, "public-protect-duty", "Public protection duty", MODES[0], CLASSES[0], "PublicProtectDutyKind"),
    Effect(200, "public-fulfil-duty", "Public fulfilment duty", MODES[0], CLASSES[0], "PublicFulfilDutyKind"),
    Effect(201, "public-continuity-duty", "Public continuity duty", MODES[0], CLASSES[0], "PublicContinuityDutyKind"),
    Effect(202, "public-remedy-duty", "Public remedy duty", MODES[0], CLASSES[0], "PublicRemedyDutyKind"),
    Effect(203, "delegation-without-discharge", "Delegation without public discharge", MODES[1], CLASSES[0], "DelegatedPublicFunctionDutyKind"),
    Effect(204, "express-private-duty-prerequisite", "Express private-duty prerequisite", MODES[2], CLASSES[0], "ExpressPrivateDutyKind"),
    Effect(205, "no-subject-matter-private-duty", "No private duty from subject matter alone", MODES[2], CLASSES[0], "PrivateDutyBoundaryKind"),
    Effect(206, "person-duty-continuity-restoration", "Person-duty continuity and restoration", MODES[0], CLASSES[0], "PersonDutyRemedyKind"),
    Effect(207, "class9-common-cessation-restoration", "Class 9 common cessation and restoration", MODES[0], CLASSES[1], "ClassNineCommonDutyRemedyKind"),
    Effect(208, "role-duty-correction-reassignment", "Role-duty correction and reassignment", MODES[0], CLASSES[2], "RoleDutyRemedyKind"),
    Effect(209, "voluntary-duty-bounded-cure", "Bounded voluntary-duty cure", MODES[2], CLASSES[3], "VoluntaryDutyRemedyKind"),
    Effect(210, "source-specific-conflict-priority", "Source-specific obligation conflict priority", MODES[0], CLASSES[0], "DutyConflictDispositionKind"),
    Effect(211, "bounded-performance-excuse", "Bounded performance excuse", MODES[0], CLASSES[0], "BoundedExcuseKind"),
    Effect(212, "all-entitlement-nonreciprocity", "All-entitlement non-reciprocity", MODES[0], CLASSES[0], "NonreciprocityWallKind"),
    Effect(213, "finding-reader-action-duty", "Finding reader and action duty", MODES[0], CLASSES[2], "FindingReaderActionDutyKind"),
    Effect(214, "certified-positive-nonresponse", "Certified positive nonresponse", MODES[0], CLASSES[2], "FindingNonresponseKind"),
    Effect(215, "finding-alternate-escalation", "Finding alternate escalation", MODES[0], CLASSES[2], "FindingAlternateEscalationKind"),
    Effect(216, "finding-claimant-continuity", "Finding claimant continuity", MODES[0], CLASSES[2], "FindingClaimantContinuityKind"),
    Effect(217, "individual-remedy-prior-decision-review", "Individual remedy and prior-decision review", MODES[0], CLASSES[2], "FindingIndividualRemedyKind"),
    Effect(218, "finding-common-cause-investigation", "Finding common-cause investigation", MODES[0], CLASSES[2], "FindingCommonCauseKind"),
    Effect(219, "responsible-control-correction", "Responsible-control correction", MODES[0], CLASSES[2], "FindingControlCorrectionKind"),
    Effect(220, "affected-case-reaudit", "Affected-case re-audit", MODES[0], CLASSES[2], "FindingAffectedCaseReauditKind"),
    Effect(221, "recurrence-verification", "Recurrence verification", MODES[0], CLASSES[2], "FindingRecurrenceVerificationKind"),
    Effect(222, "systemic-work-no-individual-delay", "Systemic work cannot delay individual relief", MODES[0], CLASSES[2], "FindingIndividualReliefNonDelayKind"),
)
EFFECT_BY_NUMBER = {effect.number: effect for effect in EFFECTS}

# Reviewed after ``--fingerprints``. The digest covers the ordered SHA-256
# statement IDs: the derived-only declaration followed by every exact rule.
OBLIGATION_STATEMENT_SET_SHA256 = "f7e745bb141be4aba91902d78ec469977d3f8a72de46462084cde99e83320acc"

PROSE_PAYLOADS = (
    ("OBL-B1-01", "book-1/00-opening-note.md", 1143, "33be8df380b9f1fc13c96d4b01b6fdba67edb2333f1b473824f5650272442a28"),
    ("OBL-B1-02", "book-1/00-opening-note.md", 549, "3b14a4be32862226caf8241ab4f196ad5ef3996ecc123957c07cbb12263f497c"),
    ("OBL-B1-03", "book-1/00-opening-note.md", 651, "c0820bfd382a08120066580a66972cf30ed28c733363e20096b486eb4fc49532"),
    ("OBL-B1-04", "book-1/00-opening-note.md", 711, "4704ab855b2b4a562d0a37450de1966aca4cbedbf0548049cb47e4c62aec197e"),
    ("OBL-B1-05", "book-1/00-opening-note.md", 1264, "eedf0fa0a4b2f0bd10be48f29132048e7b3b7922b76670db5a9a9408a315b6b2"),
    ("OBL-B1-06", "book-1/02-public-answerability.md", 956, "8f5746aa21893ffe3b3f48431300c453b2bd054f8b34542ebf7438ddcc2a9cdf"),
    ("OBL-B1-07", "book-1/08-what-you-are-owed.md", 523, "e69ccea1f8840f67e154e394c2c5f83d3c23cfd10d6efedff6cee9b514a87e79"),
    ("OBL-B1-08", "book-1/08-what-you-are-owed.md", 512, "c9e92c85171572c4de4c3a8178ad849e893f410712d08c121e1d5b448e818ba1"),
    ("OBL-B1-09", "book-1/08-what-you-are-owed.md", 4365, "4436ab741d4a7888651b47fafbe92f2aec195ccf434fc618d40bf1a734ab8edf"),
    ("OBL-B1-10", "book-1/08-what-you-are-owed.md", 74, "2911221a36b5c1ca35ce203772f22950ffa6d6c84603c3d2b98d46f3fd3923d4"),
    ("OBL-B1-11", "book-1/08-what-you-are-owed.md", 926, "7dbe4671999795f38c1cd44a979e73aafc188008e739d3351789e3b3aaa8688b"),
    ("OBL-B1-12", "book-1/14-when-the-system-notices-it-broke.md", 5442, "4a74e2b5df7d9efdb6fa86214c1c27c8a7a0dc8202ae509ddf5fbc90ff658f1a"),
    ("OBL-B1-13", "book-1/14-when-the-system-notices-it-broke.md", 1130, "a0706ad6a5005e4828b5dc8f5cf5142f51ad9bf155cece94b3cdc389f196390d"),
    ("OBL-B1-14", "book-1/15-the-five-joints.md", 659, "debd5f0e222b7102162f58e824fbd6d8c457949bb8bb3efc307221b16a3e813e"),
    ("OBL-B1-15", "book-1/15-the-five-joints.md", 2902, "741a94d4d5224d2b3b9bdfabdbad9cf6bd2fae5570351dfa839adb51370514a8"),
    ("OBL-B1-16", "book-1/method.md", 1537, "2db17589f28d3de01df371766288e57ee8fbc4c23766395105d1221ca66e3df1"),
    ("OBL-B1-17", "book-1/method.md", 1861, "9d144831cbd3eec27d92269d9d93d173247e9fee2aa36996a6f0b1a3ebcbfe5f"),
)
PROSE_AGGREGATE_SHA256 = "2975f698af4ea17c21b631dee8a42d4c9d69631b158db30dcf4177a178f40454"


def _block(text: str, begin: str, end: str) -> str:
    if text.count(begin) != 1 or text.count(end) != 1:
        raise RuntimeError(f"expected one ordered marker pair: {begin}, {end}")
    start = text.index(begin) + len(begin)
    stop = text.index(end, start)
    return text[start:stop]


def _replace_block(text: str, begin: str, end: str, payload: str) -> str:
    start = text.index(begin) + len(begin)
    stop = text.index(end, start)
    return text[:start] + "\n" + payload.rstrip() + "\n" + text[stop:]


def _statement_id(statement: str) -> str:
    encoded = json.dumps(
        [statement.removesuffix("."), 0],
        ensure_ascii=True,
        separators=(",", ":"),
    )
    return hashlib.sha256(encoded.encode("utf-8")).hexdigest()


def _rule(atoms: Iterable[str], head: str) -> str:
    body = " & ".join(atoms)
    variables = tuple(dict.fromkeys(VARIABLE_PATTERN.findall(body + " -> " + head)))
    quantified = "".join(f"all {variable}: " for variable in variables)
    return f"{quantified}{body} -> {head}."


def _tri(subject: str, value: str, scope: str) -> list[str]:
    return [
        f"observe($source, {subject}, {value}, {scope})",
        f"observe($evidence, {subject}, {value}, {scope})",
        f"observe($review, {subject}, {value}, {scope})",
    ]


RAW_BINDINGS = (
    ("ObligationOriginScope", "ObligationOriginBinding"),
    ("SourceVersionScope", "ObligationVersionBinding"),
    ("JurisdictionScope", "ObligationJurisdictionBinding"),
    ("AuthorityScope", "ObligationScopeBinding"),
)
ORIGIN_FIELD_SCOPES = (
    "DutyBearerScope",
    "DutyScope",
    "DutyStandardScope",
    "DutyBeneficiaryOrObjectScope",
    "DutyKindScope",
    "DutyFunctionOrCommitmentScope",
    "DutyBearerModeScope",
    "DutyClassScope",
    "DutyStartScope",
    "DutyEndScope",
    "ChallengeScope",
    "CorrectionScope",
    "RemedyScope",
    "DutyBreachScope",
    "DutyContinuityScope",
    "DutyPriorityScope",
    "DutyExcuseScope",
    "PublicPrincipalRetentionScope",
    "ExpressPrivateReachScope",
    "DutyNonWaiverScope",
    "FailurePolarityScope",
)


def _raw_current_atoms() -> list[str]:
    atoms = [
        "authorized($source, ObligationsSourceAuthority, $record)",
        "authorized($temporal, ObligationsTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, ObligationsTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, ObligationsRecordReviewAuthority, $record)",
    ]
    for authority, subject in (
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ):
        atoms.extend(
            (
                f"observe({authority}, {subject}, Constitution_Obligations, SourceFamilyScope)",
                f"observe({authority}, {subject}, $version, SourceVersionScope)",
                f"observe({authority}, {subject}, $epoch, SourceEpochScope)",
                f"observe({authority}, {subject}, $jurisdiction, JurisdictionScope)",
                f"observe({authority}, {subject}, $legal_scope, AuthorityScope)",
                f"observe({authority}, {subject}, $origin, ObligationOriginScope)",
                f"observe({authority}, {subject}, $start, DutyStartScope)",
                f"observe({authority}, {subject}, $end, DutyEndScope)",
                f"observe({authority}, {subject}, ObligationsCurrentSelection, EffectiveSelectionScope)",
                f"observe({authority}, {subject}, $reconciliation, ReconciliationRecordScope)",
            )
        )
    atoms.extend(
        (
            "observe($source, $record, $temporal_record, TemporalRecordScope)",
            "observe($record_review, $record, $temporal_record, TemporalRecordScope)",
            "observe($temporal, $temporal_record, $record, ObligationsRecordScope)",
            "observe($temporal_review, $temporal_record, $record, ObligationsRecordScope)",
            "observe($source, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)",
            "observe($record_review, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)",
            "observe($source, $reconciliation, $record, ObligationsRecordScope)",
            "observe($record_review, $reconciliation, $record, ObligationsRecordScope)",
            "observe($source, $reconciliation, $version, SourceVersionScope)",
            "observe($record_review, $reconciliation, $version, SourceVersionScope)",
            "~($source = $temporal)",
            "~($source = $temporal_review)",
            CURRENT_REVIEW_GUARD,
            "~($temporal = $temporal_review)",
            "~($temporal = $record_review)",
            "~($temporal_review = $record_review)",
        )
    )
    atoms.extend(f"~collide($record, {kind})" for _, kind in RAW_BINDINGS)
    return atoms


def _origin_contract_atoms() -> list[str]:
    atoms = [
        *_raw_current_atoms(),
        "authorized($evidence, ObligationsEvidenceAuthority, $record)",
        "authorized($review, IndependentObligationsReviewAuthority, $record)",
        "~($source = $evidence)",
        "~($source = $review)",
        "~($evidence = $review)",
    ]
    for value, scope in (
        ("$record", "ObligationsRecordScope"),
        ("$version", "SourceVersionScope"),
        ("$epoch", "SourceEpochScope"),
        ("$jurisdiction", "JurisdictionScope"),
        ("$legal_scope", "AuthorityScope"),
        ("$temporal_record", "TemporalRecordScope"),
        ("$bearer", "DutyBearerScope"),
        ("$duty", "DutyScope"),
        ("$standard", "DutyStandardScope"),
        ("$target", "DutyBeneficiaryOrObjectScope"),
        ("$duty_kind", "DutyKindScope"),
        ("$function_or_commitment", "DutyFunctionOrCommitmentScope"),
        ("$bearer_mode", "DutyBearerModeScope"),
        ("$duty_class", "DutyClassScope"),
        ("$start", "DutyStartScope"),
        ("$end", "DutyEndScope"),
        ("$challenge_record", "ChallengeScope"),
        ("$correction_record", "CorrectionScope"),
        ("$remedy_record", "RemedyScope"),
        ("$breach_effect", "DutyBreachScope"),
        ("$continuity_effect", "DutyContinuityScope"),
        ("$priority_effect", "DutyPriorityScope"),
        ("$excuse_effect", "DutyExcuseScope"),
        ("$principal_retention", "PublicPrincipalRetentionScope"),
        ("$private_reach", "ExpressPrivateReachScope"),
        ("$non_waiver", "DutyNonWaiverScope"),
        ("$failure_polarity", "FailurePolarityScope"),
        ("$mode_certificate", "DutyBearerModeCertificateScope"),
        ("$class_certificate", "DutyClassCertificateScope"),
    ):
        atoms.extend(_tri("$origin", value, scope))
    atoms.append("~collide($origin, ObligationContractBinding)")
    return atoms


def _origin_join_atoms(effect: Effect) -> list[str]:
    atoms = [
        *_origin_contract_atoms(),
        "~collide($record, ObligationBearerModeBinding)",
        "~collide($record, ObligationClassBinding)",
    ]
    atoms.extend(_tri("$origin", effect.mode, "DutyBearerModeScope"))
    atoms.extend(_tri("$origin", effect.duty_class, "DutyClassScope"))
    atoms.extend(_tri("$origin", effect.duty_kind, "DutyKindScope"))
    atoms.extend(_tri("$record", f"FSCCE_{effect.number:03d}", "ObligationEffectScope"))
    atoms.extend(_tri("$effect_result", f"FSCCE_{effect.number:03d}ObligationBranch", "ObligationBranchScope"))
    atoms.extend(_tri("$effect_result", f"FSCCE_{effect.number:03d}FailureWithholdsOnly", "FailurePolarityScope"))
    if effect.mode == MODES[0]:
        atoms.append("public($bearer)")
    return atoms


def _effect_extra_fields(effect: Effect, branch: str = "standard") -> list[tuple[str, str]]:
    fields: dict[int, list[tuple[str, str]]] = {
        198: [("RespectProtectedChoiceAndCondition", "PublicDutyEffectScope")],
        199: [("ProtectAgainstExpressCoveredInterference", "PublicDutyEffectScope")],
        200: [("FulfilPositiveEntitlement", "PublicDutyEffectScope")],
        201: [("ContinueProtectionAndProvision", "PublicDutyEffectScope")],
        202: [("ReviewAndRemedyPublicBreach", "PublicDutyEffectScope")],
        203: [
            ("$principal", "PublicPrincipalScope"),
            ("$function_or_commitment", "DelegatedPublicFunctionScope"),
            ("IdenticalPrincipalDutyAndFunction", "DelegatedDutyIdentityScope"),
            ("PrincipalDutyContinuityAndRemedyRemain", "PublicPrincipalRetentionScope"),
            ("DelegationCreatesNoPublicStatusOrAuthority", "DelegationBoundaryScope"),
        ],
        204: [
            ("ExpressPrivateDutyRecord", "PrivateDutySourceScope"),
            ("SubstantiveDutyAndClassCertificateRequired", "PrivateDutySubstanceScope"),
        ],
        205: [
            ("SubjectRelationshipDependencyOwnershipOrMarketAloneIsNoDuty", "PrivateDutyBoundaryScope"),
            ("NoHorizontalDutyWithoutExpressSubstantiveSource", "PrivateDutySubstanceScope"),
        ],
        206: [
            ("PersonContinuity", "PersonDutyContinuityScope"),
            ("PersonReasonsAndReview", "PersonDutyReviewScope"),
            ("PersonRestorationAndIndividualRemedy", "PersonDutyRemedyScope"),
        ],
        207: [
            ("CommonCessationProtectionAndRestoration", "CommonDutyRemedyScope"),
            ("CommonAccountingAndRecurrenceReview", "CommonDutyAccountingScope"),
            ("IndividualHarmsRemainSeparatelyRemediable", "CommonIndividualRemedyScope"),
        ],
        208: [
            ("RoleRecusalOrReassignmentAndCorrection", "RoleDutyCorrectionScope"),
            ("RoleAccountingAffectedCaseReviewAndContinuity", "RoleDutyReviewScope"),
            ("NoAutomaticPunishmentOrStatusLoss", "RoleDutyBoundaryScope"),
        ],
        209: [
            ("LawfulPerformanceUnwindingRestitutionCompensationOrExit", "VoluntaryDutyCureScope"),
            ("NoIndefinitePersonalServiceFloorLossOrNonwaivableWaiver", "VoluntaryDutyBoundaryScope"),
        ],
        210: [
            ("$conflict", "PositiveDutyConflictScope"),
            ("$preferred_duty", "PreferredDutyScope"),
            ("$preferred_standard", "PreferredDutyStandardScope"),
            ("$deferred_duty", "DeferredDutyScope"),
            ("$deferred_standard", "DeferredDutyStandardScope"),
            ("$deferred_origin", "DeferredDutyOriginScope"),
            ("$deferred_version", "DeferredSourceVersionScope"),
            ("$deferred_jurisdiction", "DeferredJurisdictionScope"),
            ("$deferred_legal_scope", "DeferredAuthorityScope"),
            ("DutyClassForbiddenAsPriority", "DutyPriorityBoundaryScope"),
            ("RightsAndContinuityFirst", "DutyPriorityBoundaryScope"),
            ("BothOriginalDutiesRemain", "DutyConflictRetentionScope"),
            ("DeferredPerformanceRecordedForRepair", "DutyConflictRepairScope"),
            (("SourceSpecificPairPriority" if branch == "priority" else "CertifiedTieOrMissingPriority"), "DutyPriorityDispositionScope"),
            (("NoLotteryOrRotation" if branch == "priority" else "IndivisibleMateriallyEqualChoiceOnly"), "DutyTieBoundaryScope"),
        ],
        211: [
            ("$impediment", "SourceEnumeratedImpedimentScope"),
            ("$performance_slice", "AffectedPerformanceSliceScope"),
            ("$onset", "ExcuseOnsetScope"),
            ("$excuse_end", "FiniteExcuseEndScope"),
            ("$excuse_review_event", "ExcuseReviewEventScope"),
            ("$notice", "ExcuseNoticeScope"),
            ("$alternate", "AlternateDutyBearerScope"),
            ("$alternate_duty", "AlternateDutyScope"),
            ("$alternate_standard", "AlternateDutyStandardScope"),
            ("OriginalDutyAccountabilityClaimBreachPrincipalAndRemediesRemain", "ExcuseRetentionScope"),
            ("SilenceConvenienceRefusalSelfCreatedOrConflictingEvidenceIsNoExcuse", "ExcuseBoundaryScope"),
        ],
        212: [
            ("$failure_kind", "DutyPerformanceOrFailureKindScope"),
            ("$protected_effect", "ProtectedEntitlementEffectScope"),
            ("NoDutyPerformanceOrFailureGate", "NonreciprocityScope"),
            ("SeparateConsequenceNeedsOwnSourceEvidenceProcessReviewAndRemedy", "SeparateConsequenceScope"),
        ],
        213: [
            ("$receipt", "FindingReceiptEvidenceScope"),
            ("$permitted_action", "FindingPermittedActionScope"),
            ("$reasons", "FindingReasonsScope"),
            ("$action_review", "FindingActionReviewScope"),
            ("$reader_duty", "FindingReaderDutyScope"),
            ("$reader_standard", "FindingReaderStandardScope"),
        ],
        214: [
            ("$nonresponse", "PositiveFindingNonresponseScope"),
            ("CertifiedPositiveNonresponse", "NonresponseDispositionScope"),
            ("SilenceIsNoAction", "NonresponseBoundaryScope"),
        ],
        215: [
            ("$nonresponse", "PositiveFindingNonresponseScope"),
            ("CertifiedPositiveNonresponse", "NonresponseDispositionScope"),
            ("SilenceIsNoAction", "NonresponseBoundaryScope"),
            ("$alternate", "AlternateFindingReaderScope"),
            ("$alternate_duty", "AlternateFindingDutyScope"),
            ("$alternate_standard", "AlternateFindingStandardScope"),
            ("PredeclaredAlternateEscalation", "FindingEscalationScope"),
        ],
        216: [
            ("$continuity_bearer", "FindingContinuityBearerScope"),
            ("$continuity_duty", "FindingContinuityDutyScope"),
            ("$continuity_standard", "FindingContinuityStandardScope"),
        ],
        217: [
            ("$remedy_bearer", "FindingRemedyBearerScope"),
            ("$individual_remedy_duty", "FindingIndividualRemedyScope"),
            ("$individual_remedy_standard", "FindingIndividualRemedyStandardScope"),
            ("$prior_review_bearer", "PriorDecisionReviewBearerScope"),
            ("$prior_review_duty", "PriorDecisionReviewDutyScope"),
            ("$prior_review_standard", "PriorDecisionReviewStandardScope"),
        ],
        218: [
            ("$investigation_bearer", "CommonCauseBearerScope"),
            ("$common_cause_duty", "CommonCauseInvestigationScope"),
            ("$common_cause_standard", "CommonCauseStandardScope"),
        ],
        219: [
            ("$correction_bearer", "ResponsibleControlBearerScope"),
            ("$control_correction_duty", "ResponsibleControlCorrectionScope"),
            ("$control_correction_standard", "ResponsibleControlStandardScope"),
        ],
        220: [
            ("$reaudit_bearer", "AffectedCaseReauditBearerScope"),
            ("$reaudit_duty", "AffectedCaseReauditScope"),
            ("$reaudit_standard", "AffectedCaseReauditStandardScope"),
        ],
        221: [
            ("$recurrence_bearer", "RecurrenceVerificationBearerScope"),
            ("$recurrence_duty", "RecurrenceVerificationScope"),
            ("$recurrence_standard", "RecurrenceVerificationStandardScope"),
        ],
        222: [
            ("$individual_relief_bearer", "IndividualReliefBearerScope"),
            ("$individual_relief_duty", "IndividualReliefDutyScope"),
            ("$individual_relief_standard", "IndividualReliefStandardScope"),
            ("SystemicWorkCannotDelayIndividualContinuityOrRemedy", "IndividualReliefNonDelayScope"),
        ],
    }
    return fields[effect.number]


def _route_contract_atoms() -> list[str]:
    atoms = ["err($finding_subject, $finding_kind)"]
    for value, scope in (
        ("$finding_kind", "FindingKindScope"),
        ("$finding_subject", "FindingSubjectScope"),
        ("$subject", "FindingAffectedSubjectScope"),
        ("$case", "FindingCaseScope"),
        ("$reader", "FindingReaderScope"),
        ("$route", "FindingRouteScope"),
    ):
        atoms.extend(_tri("$route", value, scope))
    atoms.extend(("~($reader = $source)", "~($reader = $evidence)", "~($reader = $review)"))
    return atoms


def _effect_atoms(effect: Effect, branch: str = "standard") -> list[str]:
    atoms = _origin_join_atoms(effect)
    if effect.number >= 213:
        atoms.extend(_route_contract_atoms())
    for value, scope in _effect_extra_fields(effect, branch):
        atoms.extend(_tri("$effect_result", value, scope))
    if effect.number == 203:
        atoms.extend(("public($principal)", "~($bearer = $principal)"))
    if effect.number == 210:
        atoms.append("~collide($conflict, ObligationPriorityBinding)")
    if effect.number == 211:
        atoms.extend((
            "~($bearer = $source)", "~($bearer = $evidence)", "~($bearer = $review)",
            "~collide($effect_result, ObligationExcuseBinding)",
        ))
    return atoms


def _collision_rules() -> list[str]:
    rules: list[str] = []
    for scope, kind in RAW_BINDINGS:
        rules.append(_rule((
            "authorized($source, ObligationsSourceAuthority, $record)",
            f"observe($source, $record, $first, {scope})",
            f"observe($source, $record, $second, {scope})",
            "~($first = $second)",
        ), f"collide($record, {kind})"))
    for scope in ORIGIN_FIELD_SCOPES:
        rules.append(_rule((
            f"observe($source, $origin, $first, {scope})",
            f"observe($source, $origin, $second, {scope})",
            "~($first = $second)",
        ), "collide($origin, ObligationContractBinding)"))
    rules.extend((
        _rule((
            "authorized($source, ObligationsSourceAuthority, $record)",
            "observe($source, $first_origin, $first_certificate, DutyBearerModeCertificateScope)",
            "observe($source, $first_origin, $first_mode, DutyBearerModeScope)",
            "observe($source, $second_origin, $second_certificate, DutyBearerModeCertificateScope)",
            "observe($source, $second_origin, $second_mode, DutyBearerModeScope)",
            "~($first_mode = $second_mode)",
        ), "collide($record, ObligationBearerModeBinding)"),
        _rule((
            "authorized($source, ObligationsSourceAuthority, $record)",
            "observe($source, $first_origin, $first_certificate, DutyClassCertificateScope)",
            "observe($source, $first_origin, $first_class, DutyClassScope)",
            "observe($source, $second_origin, $second_certificate, DutyClassCertificateScope)",
            "observe($source, $second_origin, $second_class, DutyClassScope)",
            "~($first_class = $second_class)",
        ), "collide($record, ObligationClassBinding)"),
        _rule((
            "observe($source, $conflict, $first, DutyPriorityDispositionScope)",
            "observe($source, $conflict, $second, DutyPriorityDispositionScope)",
            "~($first = $second)",
        ), "collide($conflict, ObligationPriorityBinding)"),
        _rule((
            "observe($source, $effect_result, $first, ExcuseReviewEventScope)",
            "observe($source, $effect_result, $second, ExcuseReviewEventScope)",
            "~($first = $second)",
        ), "collide($effect_result, ObligationExcuseBinding)"),
    ))
    return rules


def _common_rules() -> list[str]:
    return _collision_rules()


def _finding_route_rules() -> list[str]:
    direct = [*_origin_join_atoms(EFFECT_BY_NUMBER[213])]
    direct.extend(atom.replace("$finding_kind", "TemporalRecord") for atom in _route_contract_atoms())
    direct.extend(_tri("$route", "TemporalRecord", "FindingKindScope"))
    direct.append("observe($source, $route, Appeals, FindingReaderScope)")
    return [_rule(direct, "obliged(Appeals, $subject)")]


def _effect_conclusion_rules(effect: Effect) -> list[str]:
    if effect.number == 213:
        return []
    atoms = _effect_atoms(effect, "priority" if effect.number == 210 else "standard")
    heads: dict[int, tuple[str, ...]] = {
        198: ("obliged($bearer, $duty, $standard)",),
        199: ("obliged($bearer, $duty, $standard)",),
        200: ("obliged($bearer, $duty, $standard)",),
        201: ("obliged($bearer, $duty, $standard)",),
        202: ("obliged($bearer, $duty, $standard)",),
        203: ("obliged($bearer, $duty, $standard)", "obliged($principal, $duty, $standard)"),
        204: ("obliged($bearer, $duty, $standard)",),
        205: ("prevents($bearer, SubjectMatterOnlyPrivateDutyInference)",),
        206: ("obliged($bearer, $duty, $standard)",),
        207: ("obliged($bearer, $duty, $standard)",),
        208: ("obliged($bearer, $duty, $standard)",),
        209: ("obliged($bearer, $duty, $standard)",),
        210: (
            "obliged($bearer, $preferred_duty, $preferred_standard)",
            "obliged($bearer, $deferred_duty, $deferred_standard)",
            "prevents($effect_result, PriorityDischargesOriginalDuty)",
        ),
        211: ("obliged($bearer, $duty, $standard)", "obliged($alternate, $alternate_duty, $alternate_standard)"),
        212: ("prevents($target, ObligationFailureEntitlementGate)",),
        214: ("prevents($nonresponse, SilenceAsFindingAction)",),
        215: ("obliged($alternate, $alternate_duty, $alternate_standard)",),
        216: ("obliged($continuity_bearer, $continuity_duty, $continuity_standard)",),
        217: (
            "obliged($remedy_bearer, $individual_remedy_duty, $individual_remedy_standard)",
            "obliged($prior_review_bearer, $prior_review_duty, $prior_review_standard)",
        ),
        218: ("obliged($investigation_bearer, $common_cause_duty, $common_cause_standard)",),
        219: ("obliged($correction_bearer, $control_correction_duty, $control_correction_standard)",),
        220: ("obliged($reaudit_bearer, $reaudit_duty, $reaudit_standard)",),
        221: ("obliged($recurrence_bearer, $recurrence_duty, $recurrence_standard)",),
        222: (
            "obliged($individual_relief_bearer, $individual_relief_duty, $individual_relief_standard)",
            "prevents($subject, SystemicWorkDelaysIndividualRelief)",
        ),
    }
    branches = ("priority", "tie") if effect.number == 210 else ("standard",)
    return [_rule(_effect_atoms(effect, branch), head) for branch in branches for head in heads[effect.number]]


def _typed_reader_bridge() -> str:
    effect = EFFECT_BY_NUMBER[213]
    return _rule([*_effect_atoms(effect), "obliged($reader, $subject)"], "obliged($reader, $reader_duty, $reader_standard)")


def formal_rules() -> tuple[str, ...]:
    rules = [*_common_rules(), *_finding_route_rules()]
    for effect in EFFECTS:
        rules.extend(_effect_conclusion_rules(effect))
    rules.append(_typed_reader_bridge())
    if len(rules) != len(set(rules)):
        raise RuntimeError("generated obligations rules are not unique")
    return tuple(rules)


def formal_statements() -> tuple[str, ...]:
    return (DERIVED_STATEMENT, *formal_rules())


def _enacted_statements(source: str) -> tuple[str, ...]:
    derived = tuple(line for line in _block(source, DERIVED_BEGIN, DERIVED_END).splitlines() if line and not line.startswith("#"))
    rules = tuple(line for line in _block(source, RULES_BEGIN, RULES_END).splitlines() if line and not line.startswith("#"))
    return (*derived, *rules)


def _body_relations(statement: str) -> set[str]:
    body = statement.split(" -> ", 1)[0]
    return set(re.findall(r"\b([a-z][a-z0-9_]*)\(", body))


def _consumer_rules(source: str) -> tuple[str, ...]:
    return tuple(line for line in source.splitlines() if line and not line.startswith("#") and " -> " in line and "obliged" in _body_relations(line))


def validate_consumer_allowlist(source: str, *, mutation_control: bool = True) -> None:
    if _consumer_rules(source) != (_typed_reader_bridge(),):
        raise RuntimeError("obliged consumers differ from the one family-owned typed reader bridge")
    if mutation_control:
        mutated = source + "\nall $x: obliged(Review, $x) -> complete($x, OutsideObligationsReader).\n"
        try:
            validate_consumer_allowlist(mutated, mutation_control=False)
        except RuntimeError:
            pass
        else:
            raise RuntimeError("outside obliged-reader mutation was not rejected")


def _err_kinds(source: str) -> set[str]:
    kinds: set[str] = set()
    for line in source.splitlines():
        if not line or line.startswith("#") or " -> err(" not in line:
            continue
        match = re.fullmatch(r"err\([^,]+, ([A-Z][A-Za-z0-9_]*)\)\.", line.split(" -> ", 1)[1])
        if match:
            kinds.add(match.group(1))
    return kinds


def _validate_route_exhaustiveness(source: str) -> None:
    actual = _err_kinds(source)
    expected = set(FINDING_KINDS)
    if actual != expected:
        raise RuntimeError(f"finding kind inventory drifted: missing={sorted(expected-actual)} unexpected={sorted(actual-expected)}")
    for kind in FINDING_KINDS:
        reader = LEGACY_READER[kind]
        routes = [
            line for line in source.splitlines()
            if " -> obliged(" in line and f", {kind})" in line.split(" -> ", 1)[0]
        ]
        if not routes or not any(rule.split(" -> ", 1)[1].startswith(f"obliged({reader}, ") for rule in routes):
            raise RuntimeError(f"finding kind lacks its exact {reader} reader route: {kind}")


def _validate_contract_shape(rules: Sequence[str]) -> None:
    text = "\n".join(rules)
    for token in (
        "DutyBearerModeCertificateScope", "DutyClassCertificateScope", "ObligationClassBinding",
        "ObligationOriginBinding", "ObligationVersionBinding", "ObligationJurisdictionBinding",
        "ObligationScopeBinding", "ObligationPriorityBinding", "ObligationExcuseBinding",
        "DutyClassForbiddenAsPriority", "RightsAndContinuityFirst", "CertifiedTieOrMissingPriority",
        "OriginalDutyAccountabilityClaimBreachPrincipalAndRemediesRemain",
        "NoDutyPerformanceOrFailureGate", "SystemicWorkCannotDelayIndividualContinuityOrRemedy",
    ):
        if token not in text:
            raise RuntimeError(f"obligations contract omits {token}")
    if "admits(" in text:
        raise RuntimeError("FS-CVF-016 may not add an admitted relation")
    for effect in EFFECTS:
        effect_rules = [_typed_reader_bridge()] if effect.number == 213 else _effect_conclusion_rules(effect)
        if not effect_rules or any(f"FSCCE_{effect.number:03d}" not in rule for rule in effect_rules):
            raise RuntimeError(f"FS-CCE-{effect.number:03d} source-bound conclusion drifted")


def validate_formal_surface(source: str | None = None) -> tuple[str, ...]:
    text = CONSTITUTION.read_text(encoding="utf-8") if source is None else source
    actual = _enacted_statements(text)
    expected = formal_statements()
    if actual != expected:
        raise RuntimeError("FS-CVF-016 source block differs from its generator")
    actual_ids = tuple(_statement_id(statement) for statement in actual)
    statement_set_sha256 = hashlib.sha256("\n".join(actual_ids).encode()).hexdigest()
    if statement_set_sha256 != OBLIGATION_STATEMENT_SET_SHA256:
        raise RuntimeError("FS-CVF-016 exact statement IDs changed")
    _validate_contract_shape(formal_rules())
    _validate_route_exhaustiveness(text)
    validate_consumer_allowlist(text)
    return actual


def _find_payload(path: pathlib.Path, length: int, digest: str) -> bytes:
    data = path.read_bytes()
    starts = [0, *(index + 1 for index, byte in enumerate(data) if byte == 10)]
    matches = [data[start : start + length] for start in starts if start + length <= len(data) and hashlib.sha256(data[start : start + length]).hexdigest() == digest]
    if len(matches) != 1:
        raise RuntimeError(f"approved prose payload {digest} occurs {len(matches)} times in {path}")
    return matches[0]


def validate_prose_payloads() -> None:
    payloads = [_find_payload(ROOT / rel, length, digest) for _, rel, length, digest in PROSE_PAYLOADS]
    if hashlib.sha256(b"\n\n".join(payloads)).hexdigest() != PROSE_AGGREGATE_SHA256:
        raise RuntimeError("OBL-B1-v1 aggregate prose digest changed")


def _protected_claim_refs() -> tuple[str, ...]:
    source = json.loads(LEDGER.read_text(encoding="utf-8"))
    row = next(item for item in source["constitutional_effects"] if item["id"] == "FS-CCE-212")
    refs = tuple(row["affected_claim_refs"])
    if len(refs) != 13 or len(set(refs)) != len(refs):
        raise RuntimeError("FS-CCE-212 protected claim set is not the ledger-derived 13-claim set")
    return refs


@dataclass(frozen=True)
class Fixture:
    facts: tuple[str, ...]
    mapping: tuple[tuple[str, str], ...]

    def term(self, variable: str) -> str:
        return dict(self.mapping)[variable]


@dataclass(frozen=True)
class PinCase:
    label: str
    facts: tuple[str, ...]
    queries: tuple[tuple[str, str, bool], ...]


WATCHED_MUTATION_CASES = {
    "raw-currentness-rejoin": ("origin omission SourceVersionScope",),
    "duty-origin-binding": ("origin omission DutyScope",),
    "principal-non-transfer": ("FS-CCE-203 omission DelegatedDutyIdentityScope",),
    "express-private-source": ("FS-CCE-204 omission PrivateDutySourceScope",),
    "class-exclusivity": ("FS-CCE-198 conflicting duty class",),
    "entitlement-wall": ("FS-CCE-212 omission NonreciprocityScope",),
    "rights-first-conflict": ("FS-CCE-210 omission DutyPriorityBoundaryScope",),
    "tie-repair": ("FS-CCE-210 tie omission DutyPriorityDispositionScope",),
    "excuse-independence-origin": (
        "FS-CCE-211 self-certified excuse refusal",
        "FS-CCE-211 omission ExcuseRetentionScope",
    ),
    "finding-reader": ("finding-reader counterfactual",),
    "nonresponse-alternate": ("FS-CCE-215 omission nonresponse alternate",),
    "systemic-individual-separation": ("FS-CCE-222 omission IndividualReliefNonDelayScope",),
}


def _raw_atoms(statement: str) -> tuple[str, ...]:
    body = QUANTIFIER_PREFIX.sub("", statement.split(" -> ", 1)[0])
    return tuple(atom for atom in body.split(" & ") if atom.startswith(("authorized(", "observe(", "public(", "challenge(")))


def _constant(prefix: str, variable: str) -> str:
    return prefix + "".join(word.title() for word in variable[1:].split("_"))


def _ground_rule(statement: str, prefix: str, *, overrides: dict[str, str] | None = None, fused: tuple[str, str] | None = None, omit_scopes: tuple[str, ...] = ()) -> Fixture:
    variables = tuple(dict.fromkeys(VARIABLE_PATTERN.findall(statement)))
    mapping = {variable: _constant(prefix, variable) for variable in variables}
    mapping.update(overrides or {})
    if fused:
        mapping[fused[1]] = mapping[fused[0]]

    def ground(value: str) -> str:
        return VARIABLE_PATTERN.sub(lambda match: mapping[match.group(0)], value)

    facts = []
    for atom in _raw_atoms(statement):
        if any(atom.endswith(f", {scope})") for scope in omit_scopes):
            continue
        facts.append(ground(atom))
    return Fixture(tuple(dict.fromkeys(facts)), tuple(mapping.items()))


def _effect_rule(effect: Effect, branch: str = "standard") -> str:
    if effect.number == 213:
        return _typed_reader_bridge()
    candidates = _effect_conclusion_rules(effect)
    if effect.number == 210 and branch == "tie":
        return candidates[len(candidates) // 2]
    return candidates[0]


def _finding_overrides(kind: str) -> dict[str, str]:
    reader = LEGACY_READER[kind]
    subject = {"Isolation": "Adam", "MaturityDispute": "Hano"}.get(kind, f"{kind}Affected")
    finding_subject = "Order_Court_A" if kind == "OrderConflict" else subject
    return {
        "$bearer": "State", "$target": subject, "$subject": subject,
        "$finding_subject": finding_subject, "$finding_kind": kind, "$reader": reader,
        "$reader_duty": f"Read{kind}FindingDuty", "$reader_standard": f"Read{kind}FindingStandard",
    }


def _finding_fixture_facts(kind: str, subject: str, finding_subject: str) -> tuple[str, ...]:
    entry = f"{kind}Entry"
    if kind == "Placement":
        return (f"put(State, {finding_subject}, Homestay)",)
    if kind == "Isolation":
        return ()
    if kind == "StatusConflict":
        return (
            f"person({finding_subject})", f"rotten({finding_subject})",
            f"authorized({finding_subject}, VoidStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, VoidScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, VoidScope)",
            f"carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)",
            f"carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)",
            f"authorized({finding_subject}, ClearStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, ClearScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, ClearScope)",
            f"carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)",
            f"carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "CarryOmission":
        return (
            f"authorized({finding_subject}, VoidStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, VoidScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, VoidScope)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "CarryForgery":
        return (
            f"carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)",
            f"carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "ClearOmission":
        return (
            f"authorized({finding_subject}, ClearStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, ClearScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, ClearScope)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "ClearForgery":
        return (
            f"carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)",
            f"carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "StandingOmission":
        return (
            f"authorized({finding_subject}, StandingStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, StandingScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, StandingScope)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "RecordDisappearance":
        return (
            f"authorized({finding_subject}, PreservedStatus, Epoch_Previous)",
            f"observe(Chronicle, {finding_subject}, Epoch_Previous, PreservedScope)",
            f"observe(TemporalReview, {finding_subject}, Epoch_Previous, PreservedScope)",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "MaturityDispute":
        return (f"challenge({subject}, MaturityRecord, TemporalReview)",)
    if kind == "OrderConflict":
        return (
            "list(ObligationsOrderOpposite, Epoch_Review, Epoch_Current, EventSequence)",
            "observe(Chronicle, ObligationsOrderOpposite, Epoch_Review, EventStartScope)",
            "observe(TemporalReview, ObligationsOrderOpposite, Epoch_Review, EventStartScope)",
            "observe(Chronicle, ObligationsOrderOpposite, Epoch_Current, EventEndScope)",
            "observe(TemporalReview, ObligationsOrderOpposite, Epoch_Current, EventEndScope)",
            f"challenge({subject}, {finding_subject}, TemporalReview)",
        )
    if kind == "TemporalRecord":
        return (f"authorized({finding_subject}, ActiveCustody, {kind}Case)",)
    if kind == "TemporalAuthority":
        return (
            f"person({finding_subject})", f"person({kind}Victim)",
            f"injure({finding_subject}, {kind}Victim)", f"judge(Court, {finding_subject})",
            f"challenge({subject}, {entry}, TemporalReview)",
        )
    if kind == "TemporalDispute":
        return (
            f"challenge({subject}, {kind}Lease, TemporalReview)",
            f"authorized({kind}Lease, ActiveCustody, {kind}Case)",
            f"cite(Court, {kind}Case, {subject})",
        )
    raise RuntimeError(f"unsupported finding fixture: {kind}")


def _effect_fixture(effect: Effect, prefix: str, *, branch: str = "standard", fused: tuple[str, str] | None = None, omit_scopes: tuple[str, ...] = (), overrides: dict[str, str] | None = None) -> Fixture:
    bearer = _constant(prefix, "$bearer")
    base = {
        "$bearer": bearer,
        "$target": _constant(prefix, "$target"), "$principal": "State",
        "$bearer_mode": effect.mode,
        "$duty_class": effect.duty_class,
        "$duty_kind": effect.duty_kind,
    }
    if effect.number >= 213:
        base.update(_finding_overrides("Placement"))
        base["$reader_duty"] = _constant(prefix, "$reader_duty")
        base["$reader_standard"] = _constant(prefix, "$reader_standard")
    base.update(overrides or {})
    fixture = _ground_rule(_effect_rule(effect, branch), prefix, overrides=base, fused=fused, omit_scopes=omit_scopes)
    facts = list(fixture.facts)
    if effect.number >= 213:
        kind = base["$finding_kind"]
        facts.extend(_finding_fixture_facts(kind, base["$subject"], base["$finding_subject"]))
    return Fixture(tuple(dict.fromkeys(facts)), fixture.mapping)


def _effect_query(effect: Effect, fixture: Fixture) -> str:
    head = _effect_rule(effect).split(" -> ", 1)[1].removesuffix(".")
    return VARIABLE_PATTERN.sub(lambda match: dict(fixture.mapping)[match.group(0)], head)


def _append_facts(lines: list[str], facts: Iterable[str]) -> None:
    lines.extend(f"{fact}." for fact in facts)


def _append_query(lines: list[str], claim: str, query: str, expected: bool) -> None:
    lines.extend((f"# {claim}", f"? {query}.", f"# => {'TRUE' if expected else 'FALSE'}", ""))


def _finalize_pins(header: str, lines: list[str]) -> str:
    queries = [line[2:-1] for line in lines if line.startswith("? ")]
    if len(queries) != len(set(queries)):
        duplicates = sorted(query for query in set(queries) if queries.count(query) > 1)
        raise RuntimeError(f"generated obligations queries are not unique: {duplicates[:3]}")
    return "\n".join(["# SPDX-License-Identifier: MIT OR Apache-2.0", header, f":expect-pins {len(queries)}", "", *lines]).rstrip() + "\n"


def _conclusion_heads(effect: Effect) -> tuple[str, ...]:
    return tuple(dict.fromkeys(rule.split(" -> ", 1)[1].removesuffix(".") for rule in _effect_conclusion_rules(effect)))


def _main_pin_cases() -> tuple[PinCase, ...]:
    cases: list[PinCase] = []
    for effect in EFFECTS:
        fixture = _effect_fixture(effect, f"OblPositive{effect.number:03d}")
        effect_query = _effect_query(effect, fixture)
        queries = [(f"FS-CCE-{effect.number:03d} positive: {effect.title}.", effect_query, True)]
        if effect.number != 213:
            for index, head in enumerate(_conclusion_heads(effect), 1):
                grounded = VARIABLE_PATTERN.sub(lambda match: dict(fixture.mapping)[match.group(0)], head)
                if grounded == effect_query:
                    continue
                queries.append((f"FS-CCE-{effect.number:03d} legal conclusion {index} remains source-bound.", grounded, True))
        cases.append(PinCase(f"FS-CCE-{effect.number:03d} positive", fixture.facts, tuple(queries)))

    for effect in EFFECTS:
        fixture = _effect_fixture(effect, f"OblFused{effect.number:03d}", fused=("$source", "$record_review"))
        cases.append(PinCase(
            f"FS-CCE-{effect.number:03d} source-review independence",
            fixture.facts,
            ((f"FS-CCE-{effect.number:03d} withholds when source and record reviewer fuse.", _effect_query(effect, fixture), False),),
        ))

    omission_scopes = (
        "DutyBearerScope", "DutyScope", "DutyStandardScope", "DutyBeneficiaryOrObjectScope",
        "DutyKindScope", "DutyFunctionOrCommitmentScope", "DutyBearerModeScope", "DutyClassScope",
        "SourceVersionScope", "SourceEpochScope", "JurisdictionScope", "AuthorityScope",
        "DutyStartScope", "DutyEndScope", "ChallengeScope", "CorrectionScope", "RemedyScope",
        "DutyBreachScope", "DutyContinuityScope", "DutyPriorityScope", "DutyExcuseScope",
        "PublicPrincipalRetentionScope", "ExpressPrivateReachScope", "DutyNonWaiverScope", "FailurePolarityScope",
    )
    for index, scope in enumerate(omission_scopes, 1):
        fixture = _effect_fixture(EFFECT_BY_NUMBER[198], f"OblOmit{index:02d}", omit_scopes=(scope,))
        cases.append(PinCase(
            f"origin omission {scope}", fixture.facts,
            ((f"Omitting {scope} withholds the duty origin.", _effect_query(EFFECT_BY_NUMBER[198], fixture), False),),
        ))

    for effect_number, scope in (
        (203, "DelegatedDutyIdentityScope"), (204, "PrivateDutySourceScope"),
        (206, "PersonDutyRemedyScope"), (207, "CommonDutyRemedyScope"),
        (208, "RoleDutyCorrectionScope"), (209, "VoluntaryDutyBoundaryScope"),
        (210, "DutyPriorityBoundaryScope"), (211, "FiniteExcuseEndScope"),
        (213, "FindingReceiptEvidenceScope"), (214, "PositiveFindingNonresponseScope"),
        (215, "FindingEscalationScope"), (222, "IndividualReliefNonDelayScope"),
    ):
        effect = EFFECT_BY_NUMBER[effect_number]
        fixture = _effect_fixture(effect, f"OblSpecialOmit{effect_number}", omit_scopes=(scope,))
        cases.append(PinCase(
            f"FS-CCE-{effect_number:03d} omission {scope}", fixture.facts,
            ((f"FS-CCE-{effect_number:03d} omitting {scope} withholds the effect.", _effect_query(effect, fixture), False),),
        ))

    class_collision = _effect_fixture(EFFECT_BY_NUMBER[198], "OblClassCollision")
    class_collision_facts = (*class_collision.facts, (
        f"observe({class_collision.term('$source')}, {class_collision.term('$origin')}, "
        "ClassNineCommonDutyClass, DutyClassScope)"
    ))
    cases.append(PinCase(
        "FS-CCE-198 conflicting duty class", class_collision_facts,
        (("A conflicting class certificate cannot compose with the duty origin.", _effect_query(EFFECT_BY_NUMBER[198], class_collision), False),),
    ))

    for effect_number, scope, label in (
        (210, "DutyPriorityDispositionScope", "FS-CCE-210 tie omission DutyPriorityDispositionScope"),
        (211, "ExcuseRetentionScope", "FS-CCE-211 omission ExcuseRetentionScope"),
        (212, "NonreciprocityScope", "FS-CCE-212 omission NonreciprocityScope"),
    ):
        effect = EFFECT_BY_NUMBER[effect_number]
        branch = "tie" if effect_number == 210 else "standard"
        fixture = _effect_fixture(
            effect,
            f"OblWatchedOmit{effect_number}",
            branch=branch,
            omit_scopes=(scope,),
        )
        cases.append(PinCase(
            label, fixture.facts,
            ((f"FS-CCE-{effect_number:03d} omitting {scope} withholds the watched effect.", _effect_query(effect, fixture), False),),
        ))

    nonresponse_alternate = _effect_fixture(
        EFFECT_BY_NUMBER[215],
        "OblNonresponseAlternate",
        omit_scopes=("PositiveFindingNonresponseScope", "AlternateFindingReaderScope"),
    )
    cases.append(PinCase(
        "FS-CCE-215 omission nonresponse alternate", nonresponse_alternate.facts,
        (("Alternate escalation requires both certified nonresponse and its exact alternate.", _effect_query(EFFECT_BY_NUMBER[215], nonresponse_alternate), False),),
    ))

    tie = _effect_fixture(EFFECT_BY_NUMBER[210], "OblPriorityTie", branch="tie")
    cases.append(PinCase(
        "FS-CCE-210 tie", tie.facts,
        (("An independently certified tie preserves both duties and assigns review.", _effect_query(EFFECT_BY_NUMBER[210], tie), True),),
    ))

    class_priority = _effect_fixture(EFFECT_BY_NUMBER[210], "OblClassPriority")
    cases.append(PinCase(
        "FS-CCE-210 class priority refusal",
        tuple(fact.replace("SourceSpecificPairPriority", "DutyClassPriorityDisposition") for fact in class_priority.facts),
        (("Duty class cannot become a conflict priority key.", _effect_query(EFFECT_BY_NUMBER[210], class_priority), False),),
    ))

    self_excuse = _effect_fixture(EFFECT_BY_NUMBER[211], "OblSelfExcuse", fused=("$source", "$bearer"))
    cases.append(PinCase(
        "FS-CCE-211 self-certified excuse refusal", self_excuse.facts,
        (("A duty bearer cannot certify its own excuse.", _effect_query(EFFECT_BY_NUMBER[211], self_excuse), False),),
    ))

    for index, claim_ref in enumerate(_protected_claim_refs(), 1):
        fixture = _effect_fixture(EFFECT_BY_NUMBER[212], f"OblWall{index:02d}", overrides={"$protected_effect": claim_ref.replace("-", "_")})
        cases.append(PinCase(
            f"FS-CCE-212 {claim_ref}", fixture.facts,
            ((f"FS-CCE-212 protects the ledger-derived route for {claim_ref}.", _effect_query(EFFECT_BY_NUMBER[212], fixture), True),),
        ))

    for index, kind in enumerate(FINDING_KINDS, 1):
        overrides = _finding_overrides(kind)
        fixture = _effect_fixture(EFFECT_BY_NUMBER[213], f"OblFinding{index:02d}", overrides=overrides)
        typed = f"obliged({overrides['$reader']}, {overrides['$reader_duty']}, {overrides['$reader_standard']})"
        cases.append(PinCase(
            f"{kind} finding route", fixture.facts,
            (
                (f"The exact {kind} finding route reaches its typed reader duty.", typed, True),
                (f"The wrong recipient gains no {kind} reader duty.", f"obliged(Wrong{kind}Reader, {overrides['$reader_duty']}, {overrides['$reader_standard']})", False),
            ),
        ))

    return tuple(cases)


def _validate_watched_mutation_cases(cases: Sequence[PinCase]) -> None:
    if len(WATCHED_MUTATION_CASES) != 12:
        raise RuntimeError("obligations watched-mutation inventory must contain 12 seams")
    labels = {case.label for case in cases}
    labels.add("finding-reader counterfactual")
    required = {
        label
        for case_labels in WATCHED_MUTATION_CASES.values()
        for label in case_labels
    }
    missing = required - labels
    if missing:
        raise RuntimeError(f"obligations watched-mutation cases missing: {sorted(missing)}")


def _append_pin_case(lines: list[str], case: PinCase) -> None:
    _append_facts(lines, case.facts)
    for claim, query, expected in case.queries:
        _append_query(lines, claim, query, expected)


def render_obligations_pins() -> str:
    lines = [
        "# Supplied records establish bounded legal effects only.",
        "# They prove no receipt, action, delivery, remedy, recurrence control, or institutional liveness.",
        "",
    ]
    cases = _main_pin_cases()
    _validate_watched_mutation_cases(cases)
    for case in cases:
        _append_pin_case(lines, case)

    return _finalize_pins(MAIN_HEADER, lines)


def _counterfactual_source(remove_rule: str) -> str:
    source = CONSTITUTION.read_text(encoding="utf-8")
    if source.count(remove_rule) != 1:
        raise RuntimeError("counterfactual seam occurrence drifted")
    return source.replace(remove_rule + "\n", "", 1)


def render_independence_counterfactual() -> str:
    source = CONSTITUTION.read_text(encoding="utf-8")
    block = _block(source, RULES_BEGIN, RULES_END)
    count = block.count(CURRENT_REVIEW_GUARD)
    if count < len(EFFECTS):
        raise RuntimeError("current-source independence seams drifted")
    mutated = block.replace(" & " + CURRENT_REVIEW_GUARD, "")
    start = source.index(RULES_BEGIN) + len(RULES_BEGIN)
    stop = source.index(RULES_END, start)
    return source[:start] + mutated + source[stop:]


def render_source_counterfactual() -> str:
    source = CONSTITUTION.read_text(encoding="utf-8")
    rules = [*_finding_route_rules()]
    for effect in EFFECTS:
        rules.extend(_effect_conclusion_rules(effect))
    rules.append(_typed_reader_bridge())
    for rule in rules:
        if source.count(rule) != 1:
            raise RuntimeError("source-removal counterfactual seam occurrence drifted")
        source = source.replace(rule + "\n", "", 1)
    return source


def render_reader_counterfactual() -> str:
    return _counterfactual_source(_typed_reader_bridge())


def render_independence_pins() -> str:
    lines: list[str] = []
    for effect in EFFECTS:
        fixture = _effect_fixture(effect, f"OblIndependence{effect.number:03d}", fused=("$source", "$record_review"))
        _append_facts(lines, fixture.facts)
        _append_query(lines, f"FS-CCE-{effect.number:03d} widens under fused source/review.", _effect_query(effect, fixture), True)
    return _finalize_pins(INDEPENDENCE_HEADER, lines)


def render_source_pins() -> str:
    lines: list[str] = []
    for effect in EFFECTS:
        fixture = _effect_fixture(effect, f"OblSourceRemoval{effect.number:03d}")
        _append_facts(lines, fixture.facts)
        _append_query(lines, f"FS-CCE-{effect.number:03d} disappears with origin materialization.", _effect_query(effect, fixture), False)
    _append_query(lines, "Unrelated personhood remains outside the removed obligation rule.", "person(Adam)", True)
    return _finalize_pins(SOURCE_HEADER, lines)


def render_reader_pins() -> str:
    lines: list[str] = []
    for index, kind in enumerate(FINDING_KINDS, 1):
        overrides = _finding_overrides(kind)
        fixture = _effect_fixture(EFFECT_BY_NUMBER[213], f"OblReaderRemoval{index:02d}", overrides=overrides)
        _append_facts(lines, fixture.facts)
        _append_query(lines, f"The {kind} compatibility conclusion remains after reader ablation.", f"obliged({overrides['$reader']}, {overrides['$subject']})", True)
        _append_query(lines, f"The {kind} typed reader duty disappears under reader ablation.", f"obliged({overrides['$reader']}, {overrides['$reader_duty']}, {overrides['$reader_standard']})", False)
    return _finalize_pins(READER_HEADER, lines)


def _rendered_artifacts() -> tuple[tuple[pathlib.Path, str], ...]:
    return (
        (OBLIGATIONS_PINS, render_obligations_pins()),
        (INDEPENDENCE_CF, render_independence_counterfactual()),
        (INDEPENDENCE_CF.with_suffix(".pins.nibli"), render_independence_pins()),
        (SOURCE_CF, render_source_counterfactual()),
        (SOURCE_CF.with_suffix(".pins.nibli"), render_source_pins()),
        (READER_CF, render_reader_counterfactual()),
        (READER_CF.with_suffix(".pins.nibli"), render_reader_pins()),
    )


def _render_formal_block() -> str:
    return "\n".join((
        "# Generated and exact-owned by new-book-plans/21-obligations.py.",
        "# Every downstream effect repeats the raw current source and origin join.",
        "# Formal conclusions prove no receipt, action, delivery, remedy, or liveness.",
        *formal_rules(),
    ))


def write_artifacts() -> None:
    source = CONSTITUTION.read_text(encoding="utf-8")
    CONSTITUTION.write_text(_replace_block(source, RULES_BEGIN, RULES_END, _render_formal_block()), encoding="utf-8")
    for path, text in _rendered_artifacts():
        path.parent.mkdir(parents=True, exist_ok=True)
        expected = text.encode("utf-8")
        path.write_bytes(expected)
        if path.read_bytes() != expected:
            raise RuntimeError(f"obligations artifact write drifted: {path}")
        print(f"wrote {path.relative_to(ROOT)}")


def _artifact_counts() -> tuple[int, int, int, int]:
    return tuple(sum(line.startswith("? ") for line in text.splitlines()) for text in (
        render_obligations_pins(), render_independence_pins(), render_source_pins(), render_reader_pins(),
    ))


def _render_pin_case(case: PinCase) -> str:
    lines: list[str] = []
    _append_pin_case(lines, case)
    return _finalize_pins(f"{MAIN_HEADER}: fresh case {case.label}", lines)


def _run_pin_task(task: tuple[str, pathlib.Path, pathlib.Path, int], pin: pathlib.Path) -> tuple[str, str]:
    label, kb, _, timeout = task
    result = subprocess.run(
        [str(pin), "--kb", str(kb), "--allow-shell", str(task[2])],
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    output = (result.stdout + result.stderr).strip()
    if result.returncode != 0:
        raise RuntimeError(f"{label} failed with exit {result.returncode}\n{output}")
    return label, output


def execute() -> None:
    check()
    pin = pathlib.Path(os.environ.get("NIBLI_PIN", DEFAULT_PIN))
    if not pin.is_file() or not os.access(pin, os.X_OK):
        raise RuntimeError(f"release nibli-pin is not executable: {pin}")
    with tempfile.TemporaryDirectory(prefix="obligations-execute-") as temp_name:
        temp = pathlib.Path(temp_name)
        tasks: list[tuple[str, pathlib.Path, pathlib.Path, int]] = []
        for index, case in enumerate(_main_pin_cases(), 1):
            path = temp / f"obligations-case-{index:03d}.pins.nibli"
            path.write_text(_render_pin_case(case), encoding="utf-8")
            tasks.append((f"fresh case {index:03d}: {case.label}", CONSTITUTION, path, 300))
        for label, kb, pins in (
            ("independence counterfactual", INDEPENDENCE_CF, INDEPENDENCE_CF.with_suffix(".pins.nibli")),
            ("source-removal counterfactual", SOURCE_CF, SOURCE_CF.with_suffix(".pins.nibli")),
            ("finding-reader counterfactual", READER_CF, READER_CF.with_suffix(".pins.nibli")),
        ):
            tasks.append((label, kb, pins, 900))
        task_iter = iter(tasks)
        pool = ThreadPoolExecutor(max_workers=EXECUTE_WORKERS)
        active = {}

        def submit_next() -> bool:
            try:
                task = next(task_iter)
            except StopIteration:
                return False
            active[pool.submit(_run_pin_task, task, pin)] = task[0]
            return True

        for _ in range(EXECUTE_WORKERS):
            submit_next()
        try:
            while active:
                done, _ = wait(active, return_when=FIRST_COMPLETED)
                for future in done:
                    active.pop(future)
                    label, output = future.result()
                    summary = output.splitlines()[-1] if output else "PASS"
                    print(f"obligations execute: {label}: {summary}", flush=True)
                    submit_next()
        except BaseException:
            for future in active:
                future.cancel()
            pool.shutdown(wait=False, cancel_futures=True)
            raise
        else:
            pool.shutdown(wait=True)


def check() -> None:
    source = CONSTITUTION.read_text(encoding="utf-8")
    validate_formal_surface(source)
    validate_prose_payloads()
    for path, text in _rendered_artifacts():
        if not path.is_file() or path.read_bytes() != text.encode("utf-8"):
            raise RuntimeError(f"obligations artifact differs from renderer: {path}")
    main, independence, source_count, reader = _artifact_counts()
    print(
        f"obligations: PASS - 25 effects, {len(formal_statements())} exact statements, "
        f"{main} main pins, {independence}/{source_count}/{reader} counterfactual pins, "
        "17 byte-exact OBL-B1-v1 prose payloads, 12 watched mutation seams, "
        "and one exact obliged consumer"
    )


def fingerprints() -> None:
    for statement in formal_statements():
        print(_statement_id(statement))


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true")
    group.add_argument("--write-artifacts", action="store_true")
    group.add_argument("--fingerprints", action="store_true")
    group.add_argument("--check-consumers", action="store_true")
    group.add_argument("--execute", action="store_true")
    args = parser.parse_args(argv)
    try:
        with VerificationLock("verify", source_digest=hashlib.sha256(CONSTITUTION.read_bytes()).hexdigest(), root=ROOT):
            if args.write_artifacts:
                write_artifacts()
            elif args.fingerprints:
                fingerprints()
            elif args.check_consumers:
                source = CONSTITUTION.read_text(encoding="utf-8")
                validate_consumer_allowlist(source)
                print("obliged consumer allowlist: one family-owned typed reader bridge; outside-reader mutation rejected")
            elif args.execute:
                execute()
            else:
                check()
    except VerificationLockBusy as exc:
        print(f"obligations verification lock: {exc}", file=sys.stderr)
        return EX_TEMPFAIL
    except VerificationLockError as exc:
        print(f"obligations verification lock: {exc}", file=sys.stderr)
        return 2
    except (RuntimeError, ValueError, KeyError, subprocess.SubprocessError) as exc:
        print(f"obligations: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
