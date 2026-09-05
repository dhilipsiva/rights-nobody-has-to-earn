#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Render and verify the FS-CVF-003 state-form constitutional rule family."""

from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
import pathlib
import re
import sys
from typing import Iterable, Sequence

from verification_lock import (
    EX_TEMPFAIL,
    VerificationLock,
    VerificationLockBusy,
    VerificationLockError,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONSTITUTION = ROOT / "new-book-plans" / "constitution.nibli"
STATE_FORM_PINS = ROOT / "new-book-plans" / "state-form.pins.nibli"
STATE_FORM_COUNTERFACTUAL = (
    ROOT
    / "new-book-plans"
    / "counterfactual"
    / "no-state-form-independent-current-review.nibli"
)
STATE_FORM_COUNTERFACTUAL_PINS = STATE_FORM_COUNTERFACTUAL.with_suffix(
    ".pins.nibli"
)

STATE_FORM_PINS_HEADER = (
    "# State-form and political-membership family - executable coverage pins"
)
STATE_FORM_COUNTERFACTUAL_PINS_HEADER = (
    "# Counterfactual: state-form source writer serves as temporal reviewer"
)
CURRENT_REVIEW_GUARD = " & ~($source = $temporal_review)"

EXPECTED_GENERIC_MAIN_PIN_COUNT = 335
EXPECTED_ACCEPTANCE_PIN_COUNT = 56
EXPECTED_MAIN_PIN_COUNT = 391
EXPECTED_COUNTERFACTUAL_PIN_COUNT = 51
MAIN_SHARD_COUNT = 64
COUNTERFACTUAL_SHARD_COUNT = 17
DEFAULT_SHARD_PARTITION = "bytes"
SHARD_PARTITION_MODES = ("bytes", "count")
EXPECTED_MAIN_PINS_SHA256 = (
    "41c36aa72b5330bd515363bade95ff118492e60d3e8ba76735c6c3aa2bebfbc2"
)
EXPECTED_COUNTERFACTUAL_SHA256 = (
    "07c3e13151e7f8c358db80efd03cf8a7f636b5245c8c1cca95327d41dfb5d90b"
)
EXPECTED_COUNTERFACTUAL_PINS_SHA256 = (
    "4b4910d71aaa9baa8606900131b95606756bc56d7dd5fc69902bd1da1d351fd5"
)
EXPECTED_CONSTITUTION_SHA256 = (
    "235c4a5811f5da1ebd0aa75ecb6e7abb1e75065ba746b243a241c3a4b3e812df"
)

ACCEPTANCE_CASE_IDS = (
    "FSACC-001-prisoner-franchise-candidacy",
    "FSACC-002-custody-home-continuity",
    "FSACC-003-nonconventional-residence",
    "FSACC-004-claimant-chosen-multiple-residence",
    "FSACC-005-atomic-move-no-double-no-gap",
    "FSACC-006-adulthood-evidence-continuity",
    "FSACC-007-office-move-continuity",
    "FSACC-008-nonresident-rights",
    "FSACC-009-anti-capture-appointment",
    "FSACC-010-formation-failure",
    "FSACC-011-nonblocking-presidency",
    "FSACC-012-one-return-council",
    "FSACC-013-proportional-certified-assembly",
    "FSACC-014-budget-continuity",
    "FSACC-015-alternate-court-panel",
    "FSACC-016-rights-corridor",
    "FSACC-017-negotiated-secession",
    "FSACC-018-duplicate-submission",
    "FSACC-019-missing-conflicting-certificate",
)

BEGIN = "# <STATE-FORM-RULES-BEGIN>"
END = "# <STATE-FORM-RULES-END>"
RENDERER_UNLOCKED = True

EXPECTED_CARD_COUNT = 51
EXPECTED_RESULT_COUNT = 131
EXPECTED_AUTHORITY_COUNT = 142
EXPECTED_STATEMENT_COUNT = 274
EXPECTED_BAND_COUNTS = (
    (1, 25, 35, 36),
    (26, 35, 47, 49),
    (36, 36, 16, 16),
    (37, 51, 33, 41),
)

EXPECTED_RULE_BLOCK_SHA256 = "98ea81f52420e67d994ab32058280f3ae789855208750e2ae7ab1556005e4ab6"
DELEGATION_MARKER = (
    "# FS-CVF-003 executable coverage is delegated to "
    "new-book-plans/state-form.pins.nibli."
)
DELEGATED_PIN_PATHS = (
    "book-1/01-what-counts-as-evidence.pins.nibli",
    "book-1/02-public-answerability.pins.nibli",
    "book-1/03-who-holds-the-pen.pins.nibli",
    "book-1/09-the-vote-conviction-does-not-take.pins.nibli",
    "book-1/12-changing-the-rules.pins.nibli",
)


RAW_SPECS = """
001|02|CommonTierEnumeratedCompetence@CompetenceScope,NoResidualOrInherentCommonPower@ResidualPowerScope,CommonGuaranteesBoundCompetence@GuaranteeScope
002|21|RegionalResidualAuthority@CompetenceScope,ProtectedLocalCompetenceBoundary@LocalMinimumScope,CommonGuaranteesBoundCompetence@GuaranteeScope
003|02|ReviewableSubsidiarityDisplacement@DisplacementScope,CrossBoundaryEqualityOrCapacityShowing@SubsidiarityEvidenceScope,NarrowestNecessaryDisplacement@DisplacementLimitScope,NoGeneralPreemption@PreemptionScope
004|18|NoUncontestedCompetenceHolder@CompetenceConflictScope,ConstitutionalCourtInterimAllocation@InterimAllocationScope,PreserveFloorAndPreventIrreparableHarm@InterimPurposeScope,NoFinalCompetence@FinalityScope
005|02|OrdinaryLawWithinCompetence@LawmakingScope,EntrenchedDemocraticCorridor@DemocraticCorridorScope,RevenueSeparateAuthorization@RevenueBoundaryScope,SpendingSeparateAuthorization@SpendingBoundaryScope
006|02|RevenueAuthorizationOnly@RevenueScope,NoSpendingByRevenueAuthorization@SpendingBoundaryScope
007|02|AppropriationAuthorizationOnly@SpendingScope,NoDeliveryOrCapacityEffect@DeliveryBoundaryScope
008|02|LegislativePublicOversightOnly@OversightScope,NoLegislativeSelfAdjudication@SelfReviewScope,IndependentAuditPreserved@AuditScope,CourtReviewPreserved@ReviewScope
009|02|AmendmentInitiationOnly@InitiationScope,ApprovalSeparate@ApprovalScope,ReferendumSeparate@ReferendumScope,CompatibilityReviewSeparate@ReviewScope,EnactmentSeparate@EnactmentScope
010|06|$configuration@ElectoralConfigurationScope,$eligible_roster@EligibleRosterScope,$metric@ProportionalityMetricScope,$tolerance@ProportionalityToleranceScope,$district_magnitude@DistrictMagnitudeScope,$threshold@ElectoralThresholdScope,EqualBallots@BallotEqualityScope,ProportionalOutcome@OutcomeScope,GenuineOppositionRights@OppositionScope,IndependentElectionAdministration@AdministrationScope,ContestableCertification@ChallengeScope,NoWealthRecognitionContributionScoreOrServiceWeighting@WeightingScope
011|21|$regional_legislature@RegionalLegislatureScope,$delegation@DelegationScope,$eligible_roster@EligibleRosterScope,$metric@ProportionalityMetricScope,$tolerance@ProportionalityToleranceScope,ProportionalCouncilDelegation@DelegationResultScope,EqualRegionalAggregateWeight@RegionalWeightScope,NoUnelectedExecutiveAppointment@AppointmentBoundaryScope,DelegationLifecycleSeparate@LifecycleScope
012|03|OneTimeReasonedBillReturn@ReturnScope,SameRuleAssemblyRepassageEndsReturn@RepassageScope,NoExtraThresholdOrIndefiniteInterval@DelayScope,NoPermanentCouncilVeto@VetoScope
013|03|FederalSettlementConsentOnly@ConsentScope,RegionalCompetenceChange@RegionalCompetenceScope,RegionalBoundaryOrEqualisationSettlement@RegionalSettlementScope,NoGeneralOrdinaryLawVeto@VetoScope
014|04|$confidence_mandate@ConfidenceMandateScope,CollectiveExecutiveAuthority@ExecutiveScope,AssemblyAnswerability@AnswerabilityScope,NoDecreeDissolutionVetoTermExtensionOrStandingEmergency@ExecutiveLimitScope
015|02|$assembly_roster@AssemblyRosterScope,$certified_government@CertifiedGovernmentScope,$confidence_result@ConfidenceResultScope,AssemblyConfidenceCertification@ConfidenceCertificationScope,FormalAppointmentSeparate@AppointmentBoundaryScope
016|02|$removed_government@RemovedGovernmentScope,$successor@SuccessorScope,ConstructiveNoConfidenceSameDecision@ConstructiveReplacementScope,NoVacancyOnlyRemoval@VacancyScope
017|02+03|$assembly_roster@AssemblyRosterScope,$council_roster@CouncilRosterScope,$selection_configuration@PresidentialSelectionConfigurationScope,FiniteMajorityProducingJointSelection@JointSelectionScope,AtMostOnePresident@UniqueOfficeScope,FormalGovernmentAppointmentSeparate@AppointmentBoundaryScope
018|05|$certified_government@CertifiedGovernmentScope,$confidence_result@ConfidenceResultScope,NonDiscretionaryFormalAppointment@FormalAppointmentScope,NoDefeatOfCertifiedResult@AppointmentBoundaryScope
019|05|$law@PromulgatedLawScope,BoundedFormalPromulgation@PromulgationScope,NoPolicyVetoByRefusal@VetoScope,AlternateContinuityOnRefusal@ContinuityScope
020|05|$certificate@ReceivedCertificateScope,CertificationReceiptOnly@ReceiptScope,UnderlyingElectionResultNotCertifiedByReceipt@CertificationBoundaryScope
021|26|$alternate@PredeclaredAlternateScope,$formal_duty@BoundedFormalDutyScope,RefusalVacancyFailureOrRemovalContinuity@ContinuityScope,NoPolicyPowerTransfer@PolicyPowerScope,FiniteAlternateEnd@EndBoundaryScope
022|17|$case@CaseScope,$subject@SubjectScope,$remedy@RemedyScope,CaseSpecificReliefOnly@ReliefScope,NoGeneralInvalidation@FinalityScope,PublicReasonsAndEffectiveReview@ReviewScope
023|18|$case@CaseScope,$invalidated_action@InvalidatedActionScope,$remedy@RemedyScope,GenerallyApplicableConstitutionalInvalidation@InvalidationScope,NoOrdinaryPolicyOrGovernmentAdministration@AdjudicationLimitScope,PublicReasonsAndEffectiveRemedy@ReviewScope
024|18|$election_result@ElectionProcedureResultScope,$amendment_result@AmendmentProcedureResultScope,$secession_result@SecessionProcedureResultScope,ConstitutionalLegalityReviewOnly@ProcedureReviewScope,SeparatelyTypedUnderlyingResults@ResultSeparationScope
025|25|$composition_challenge@CompositionChallengeScope,$alternate_panel@AlternatePanelScope,$remedy@RemedyScope,UninvolvedAlternatePanel@PanelIndependenceScope,ChallengedCourtCannotDecideCompositionCase@SelfReviewScope,EffectiveCorrectionRoute@CorrectionScope
026|24|$qualification_referral@QualificationReferralScope,$candidate@CandidateScope,AppointmentsQualificationFunctionOnly@QualificationScope,NamedRecipientBodyOnly@RecipientScope,AuditAcquiresNoQualificationPowerByImplication@AuditBoundaryScope
027|24|$qualification_record@QualificationRecordScope,$adverse_decision@AdverseQualificationDecisionScope,ReasonedQualificationReview@QualificationReviewScope,PositiveAdverseDecisionRequired@AdverseDecisionScope,SilenceIsNeitherApprovalNorAdverseResult@SilenceScope
028|01+02+03+21|$seat_configuration@SeatConfigurationScope,$selector_configuration@SelectorConfigurationScope,$qualification_authority@QualificationAuthorityScope,$fallback_configuration@FallbackConfigurationScope,DividedAppointingSources@DividedSourceScope,OpenNominations@NominationScope,NoCoalitionMajorityAppointmentControl@AntiCaptureScope
029|02+03+21|$cause_record@RemovalCauseScope,$independent_factfinder@IndependentFactfinderScope,$confirmation@CrossBodyConfirmationScope,CauseOnlyRemoval@RemovalScope,FactfindingAndConfirmationSeparated@SeparationScope
030|01+02+03+21|$vacancy_record@VacancyRecordScope,$fallback_configuration@FallbackConfigurationScope,PredeclaredVacancyAndCaptureFallback@FallbackScope,MissingCapturedConflictedOrSilentSourceCannotVeto@VacancyVetoScope,TemporaryContinuityWithoutPermanentPower@ContinuityScope
031|02+06|$finite_term@FiniteTermScope,$vacancy_rule@VacancyRuleScope,$early_election_source@EarlyElectionSourceScope,$successor@SuccessorScope,AssemblyLifecycleSourceBound@LifecycleScope,NoLifecycleChoiceBySilence@SilenceScope
032|03+21|$finite_tenure@FiniteTenureScope,$instruction_rule@InstructionRuleScope,$replacement_rule@ReplacementRuleScope,$vacancy_rule@VacancyRuleScope,CouncilDelegationLifecycleSourceBound@LifecycleScope,NoLifecycleChoiceBySilence@SilenceScope
033|02+04|$composition@ExecutiveCompositionScope,$replacement@MemberReplacementScope,$incapacity_rule@CoordinatorIncapacityScope,$successor@SuccessorScope,ExecutiveLifecycleSourceBound@LifecycleScope,NoCoordinatorPolicyPowerExpansion@CoordinatorBoundaryScope
034|02+03+26|$selection_fallback@SelectionFallbackScope,$alternate@PredeclaredAlternateScope,$removal_factfinder@IndependentFactfinderScope,$removal_confirmation@CrossBodyConfirmationScope,FinitePresidentSelectionFallback@SelectionScope,NoIncumbentExtensionByFailure@IncumbentExtensionScope
035|01+02+03+21|$seat_configuration@SeatConfigurationScope,$selector_configuration@SelectorConfigurationScope,$fallback_configuration@FallbackConfigurationScope,CourtAndOversightAllocationSourceBound@SeatAllocationScope,NoSingleCoalitionMajorityControl@AntiCaptureScope,UninvolvedAlternateRoute@AlternateRouteScope
036|06|
037|01+02+03+21|$proposal@AmendmentProposalScope,$assembly_roster@AssemblyRosterScope,$referendum_roster@ReferendumRosterScope,$assembly_result@AssemblyResultScope,$referendum_result@ReferendumResultScope,$council_result@CouncilResultScope,$affected_region_result@AffectedRegionResultScope,FullAssemblyTwoThirds@AssemblyThresholdScope,NationalReferendumRequired@ReferendumScope,ConditionalCouncilAndAffectedRegionConsent@RegionalConsentScope
038|01|$proposal@ConstitutionalInitiativeProposalScope,$elector_submission@ElectorSubmissionScope,$docket@DocketScope,$assembly_vote@RecordedAssemblyVoteScope,InitiativeForcesDocketAndVote@InitiativeEffectScope,AmendmentThresholdsNotBypassed@ThresholdBoundaryScope
039|01|$proposal@OrdinaryInitiativeProposalScope,$compatibility_review@CompatibilityReviewScope,$referendum_roster@ReferendumRosterScope,$counterproposal@CounterproposalScope,$operative_result@OperativeResultScope,BoundedOrdinaryLawInitiative@InitiativeScope,SingleOperativeResult@UniqueOperativeResultScope
040|01|$office@DirectOfficeScope,$removed_incumbent@RemovedIncumbentScope,$successor@SuccessorScope,$ballot@RecallBallotScope,RemovalAndSuccessorOnSameBallot@ConstructiveRecallScope,AssembliesAndExecutiveCouncilExcluded@RecallBoundaryScope
041|04|$formation_failure@FormationFailureScope,$caretaker_mandate@CaretakerMandateScope,$caretaker_end@CaretakerEndScope,TemporaryCaretakerContinuity@ContinuityScope,PreserveFloorAndExistingLaw@CaretakerPurposeScope,NoOrdinaryOrIrreversiblePolicy@PolicyBoundaryScope
042|06|$caretaker_deadline@CaretakerDeadlineScope,$election_call@FreshElectionCallScope,SourceBoundFreshElectionCall@ElectionCallScope,NoOutsideClockAdvanceOrElectionOccurrence@OperationBoundaryScope
043|07|$budget_deadlock@BudgetDeadlockScope,$essential_spending_schedule@EssentialSpendingScheduleScope,$continuity_end@ContinuityEndScope,EnumeratedEssentialSpendingContinuity@BudgetContinuityScope,NoNewProgrammeOrPermanentSpending@SpendingBoundaryScope
044|05+21|$office@OfficeScope,$succession_certificate@SuccessionCertificateScope,$outgoing_holder@OutgoingHolderScope,$incoming_holder@IncomingHolderScope,OfficeSpecificCertifiedTransfer@SuccessionScope,OutgoingRefusalCannotExtendPower@IncumbentExtensionScope
045|01+02+03+21|$opening_referendum@OpeningReferendumScope,$federal_agreement@FederalAgreementScope,$rights_review@RightsAndMinorityReviewScope,$settlement@SecessionSettlementScope,$affected_population_roster@AffectedPopulationRosterScope,$ratification_result@RatificationResultScope,$collective_consent@CollectiveConsentScope,OpeningReferendumIsNotExit@ReferendumBoundaryScope,NoMilitaryOrWithholdingRoute@CoercionBoundaryScope
046|21|ProtectedLocalBudgetAuthority@LocalBudgetScope,LawfulLocalBudgetOnly@BudgetBoundaryScope,CommonGuaranteesAndReviewableSubsidiarity@GuaranteeScope
047|21|ProtectedLocalAdministrationAuthority@LocalAdministrationScope,AdministrativeConvenienceCannotRetractLocalMinimum@LocalMinimumScope
048|21|ProtectedLocalPublicSpaceManagement@PublicSpaceScope,StandingEqualityLibertiesFloorDueProcessAndCommonsBoundaries@GuaranteeScope
049|21|ProtectedLocalFacilityManagement@LocalFacilityScope,AssignedServiceAdministrationSeparate@ServiceBoundaryScope
050|21|ProtectedLocalAssignedServiceAdministration@AssignedServiceScope,NoDeliveryOrCapacityClaim@DeliveryBoundaryScope
051|24|$qualification_referral@QualificationReferralScope,$qualification_record@QualificationRecordScope,$inaction_remedy@InactionRemedyScope,IndependentRecipientDutyToAct@RecipientDutyScope,ContinuityAndRemedyForInaction@ContinuityScope,AuditAcquiresNoFunctionByImplication@AuditBoundaryScope
""".strip()


@dataclass(frozen=True)
class Field:
    value: str
    scope: str


@dataclass(frozen=True)
class Card:
    number: int
    holders: tuple[str, ...]
    fields: tuple[Field, ...]

    @property
    def power(self) -> str:
        return f"FSPOW_{self.number:03d}"


def parse_specs() -> tuple[Card, ...]:
    cards = []
    for raw in RAW_SPECS.splitlines():
        number_raw, holders_raw, fields_raw = raw.split("|", 2)
        fields = []
        if fields_raw:
            for item in fields_raw.split(","):
                value, scope = item.split("@", 1)
                fields.append(Field(value, scope))
        cards.append(
            Card(
                int(number_raw),
                tuple(f"FSBOD_{part}" for part in holders_raw.split("+")),
                tuple(fields),
            )
        )
    numbers = [card.number for card in cards]
    if numbers != list(range(1, 52)):
        raise RuntimeError(f"state-form card sequence is not 001..051: {numbers}")
    return tuple(cards)


CARDS = parse_specs()
ROLE_NAMES = ("admin", "assurer", "service", "executor")
RESULT_ACTORS = ("admin", "assurer", "service")


def quantified(names: Iterable[str]) -> str:
    return "".join(f"all DOLLAR{name}: " for name in names).replace("DOLLAR", "$")


def distinct(names: Sequence[str]) -> list[str]:
    return [
        f"~(DOLLAR{left} = DOLLAR{right})".replace("DOLLAR", "$")
        for index, left in enumerate(names)
        for right in names[index + 1 :]
    ]


def observed(actors: Iterable[str], subject: str, value: str, scope: str) -> list[str]:
    return [
        f"observe(DOLLAR{actor}, {subject}, {value}, {scope})".replace("DOLLAR", "$")
        for actor in actors
    ]


def variable_names(fields: Iterable[Field]) -> list[str]:
    return [field.value[1:] for field in fields if field.value.startswith("$")]


def current_rule_premises() -> list[str]:
    return [
        "authorized($source, StateFormSourceAuthority, $record)",
        "authorized($temporal, StateFormTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, StateFormTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, StateFormRecordReviewAuthority, $record)",
        "observe($source, $record, Constitution_StateForm, SourceFamilyScope)",
        "observe($record_review, $record, Constitution_StateForm, SourceFamilyScope)",
        "observe($temporal, $temporal_record, Constitution_StateForm, SourceFamilyScope)",
        "observe($temporal_review, $temporal_record, Constitution_StateForm, SourceFamilyScope)",
        "observe($source, $record, $version, SourceVersionScope)",
        "observe($record_review, $record, $version, SourceVersionScope)",
        "observe($temporal, $temporal_record, $version, SourceVersionScope)",
        "observe($temporal_review, $temporal_record, $version, SourceVersionScope)",
        "observe($source, $record, $temporal_record, TemporalRecordScope)",
        "observe($record_review, $record, $temporal_record, TemporalRecordScope)",
        "observe($temporal, $temporal_record, $record, StateFormRecordScope)",
        "observe($temporal_review, $temporal_record, $record, StateFormRecordScope)",
        "observe($source, $record, $power, PowerScope)",
        "observe($record_review, $record, $power, PowerScope)",
        "observe($temporal, $temporal_record, $power, PowerScope)",
        "observe($temporal_review, $temporal_record, $power, PowerScope)",
        "observe($source, $record, $jurisdiction, JurisdictionScope)",
        "observe($record_review, $record, $jurisdiction, JurisdictionScope)",
        "observe($temporal, $temporal_record, $jurisdiction, JurisdictionScope)",
        "observe($temporal_review, $temporal_record, $jurisdiction, JurisdictionScope)",
        "observe($source, $record, $legal_scope, AuthorityScope)",
        "observe($record_review, $record, $legal_scope, AuthorityScope)",
        "observe($temporal, $temporal_record, $legal_scope, AuthorityScope)",
        "observe($temporal_review, $temporal_record, $legal_scope, AuthorityScope)",
        "observe($source, $record, $epoch, SourceEpochScope)",
        "observe($record_review, $record, $epoch, SourceEpochScope)",
        "observe($temporal, $temporal_record, $epoch, SourceEpochScope)",
        "observe($temporal_review, $temporal_record, $epoch, SourceEpochScope)",
        "observe($source, $record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($record_review, $record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($temporal, $temporal_record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($temporal_review, $temporal_record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($source, $record, $reconciliation, ReconciliationRecordScope)",
        "observe($record_review, $record, $reconciliation, ReconciliationRecordScope)",
        "observe($temporal, $temporal_record, $reconciliation, ReconciliationRecordScope)",
        "observe($temporal_review, $temporal_record, $reconciliation, ReconciliationRecordScope)",
        "observe($source, $reconciliation, StateFormRecordReconciled, ReconciliationStatusScope)",
        "observe($record_review, $reconciliation, StateFormRecordReconciled, ReconciliationStatusScope)",
        "observe($source, $reconciliation, $record, StateFormRecordScope)",
        "observe($record_review, $reconciliation, $record, StateFormRecordScope)",
        "observe($source, $reconciliation, $version, SourceVersionScope)",
        "observe($record_review, $reconciliation, $version, SourceVersionScope)",
        "observe($source, $reconciliation, $power, PowerScope)",
        "observe($record_review, $reconciliation, $power, PowerScope)",
        "observe($source, $reconciliation, $jurisdiction, JurisdictionScope)",
        "observe($record_review, $reconciliation, $jurisdiction, JurisdictionScope)",
        "observe($source, $reconciliation, $legal_scope, AuthorityScope)",
        "observe($record_review, $reconciliation, $legal_scope, AuthorityScope)",
        *distinct(("source", "temporal", "temporal_review", "record_review")),
    ]


def current_rule() -> str:
    names = (
        "record",
        "source",
        "temporal",
        "temporal_review",
        "record_review",
        "version",
        "epoch",
        "temporal_record",
        "power",
        "jurisdiction",
        "legal_scope",
        "reconciliation",
    )
    return f"{quantified(names)}{' & '.join(current_rule_premises())} -> complete($record, StateFormCurrent, $temporal_record)."



# V2 is the only candidate architecture. Canonical output stays blocked until
# this branch matrix is complete, reviewed, and executable.
JURISDICTION_LABELS = (
    "",
    "CommonFederal",
    "SourceNamedRegional",
    "CrossTierCompetenceBoundary",
    "DisputedCompetenceCase",
    "CommonLawmaking",
    "CommonRevenue",
    "CommonAppropriation",
    "CommonOversight",
    "CommonAmendmentInitiation",
    "CommonAssemblyElection",
    "SourceNamedRegionalDelegation",
    "CommonBillReturn",
    "CommonAndAffectedRegionalSettlement",
    "CommonExecutiveMandate",
    "CommonConfidence",
    "CommonConstructiveReplacement",
    "CommonJointPresidentialSelection",
    "CommonFormalAppointment",
    "CommonPromulgation",
    "CommonCertificateReceipt",
    "CommonFormalContinuity",
    "ExactCase",
    "ExactConstitutionalCase",
    "ExactProcedureCase",
    "ExactCompositionCase",
    "CommonAppointmentQualificationFunction",
    "ExactAppointmentQualificationCase",
    "ExactIndependentAppointmentSeat",
    "ExactIndependentOfficeRemoval",
    "ExactAppointmentFallback",
    "CommonAssemblyLifecycle",
    "SourceNamedRegionalCouncilLifecycle",
    "CommonExecutiveLifecycle",
    "CommonPresidentialLifecycle",
    "ExactCourtOrOversightSeat",
    "PoliticalHomeMultiTier",
    "CommonAndAffectedRegionalAmendment",
    "CommonConstitutionalInitiativeDocket",
    "CommonOrdinaryLawInitiative",
    "ExactDirectOfficeRecall",
    "CommonCaretaker",
    "CommonFreshElectionCall",
    "TierAssignedFiscalContinuity",
    "ExactOfficeTransfer",
    "CommonAndAffectedRegionalSecession",
    "SourceNamedLocalBudget",
    "SourceNamedLocalAdministration",
    "SourceNamedLocalPublicSpace",
    "SourceNamedLocalFacility",
    "SourceNamedLocalAssignedService",
    "ExactQualificationReferral",
)


@dataclass(frozen=True)
class AuthorizationSpec:
    actor: str
    authority: str


@dataclass(frozen=True)
class ObservationSpec:
    actor: str
    subject: str
    value: str
    scope: str


@dataclass(frozen=True)
class RuleBranch:
    card: Card
    key: str
    fields: tuple[Field, ...]
    dynamic: bool
    dynamic_subtype: str
    authority_holders: tuple[str, ...]
    authorizations: tuple[AuthorizationSpec, ...]
    observations: tuple[ObservationSpec, ...]

    @property
    def marker(self) -> str:
        words = "".join(part.title() for part in self.key.split("_"))
        return f"{self.card.power}{words}Branch"

    @property
    def jurisdiction_kind(self) -> str:
        return f"{self.card.power}{JURISDICTION_LABELS[self.card.number]}JurisdictionKind"

    @property
    def legal_scope_kind(self) -> str:
        return f"{self.marker}AuthorityScopeKind"


@dataclass(frozen=True)
class DecisionInterface:
    identity: Field
    configurations: tuple[Field, ...]
    rosters: tuple[Field, ...]
    submissions: tuple[Field, ...]
    outcomes: tuple[Field, ...]

    @property
    def owned_terms(self) -> tuple[Field, ...]:
        return (
            self.identity,
            *self.configurations,
            *self.rosters,
            *self.submissions,
            *self.outcomes,
        )


@dataclass(frozen=True)
class CertificateLink:
    certificate: Field
    result: Field


@dataclass(frozen=True)
class DecisionLineage:
    kind: str
    rationale: str
    interfaces: tuple[DecisionInterface, ...]
    upstream_links: tuple[CertificateLink, ...]
    certificate_set: Field
    result_certificate: Field
    certified_result: Field


def _fields(spec: str) -> tuple[Field, ...]:
    if not spec:
        return ()
    parsed = []
    for item in spec.split(","):
        value, scope = item.split("@", 1)
        parsed.append(Field(value, scope))
    return tuple(parsed)


def _one_field(spec: str) -> Field:
    fields = _fields(spec)
    if len(fields) != 1:
        raise RuntimeError(f"expected one exact lineage field, found {spec!r}")
    return fields[0]


def _interface(
    identity: str,
    *,
    configurations: str,
    rosters: str = "",
    submissions: str = "",
    outcomes: str,
) -> DecisionInterface:
    return DecisionInterface(
        identity=_one_field(f"{identity}@DecisionInterfaceScope"),
        configurations=_fields(configurations),
        rosters=_fields(rosters),
        submissions=_fields(submissions),
        outcomes=_fields(outcomes),
    )


def _link(certificate: str, result: str) -> CertificateLink:
    return CertificateLink(
        certificate=_one_field(certificate),
        result=_one_field(result),
    )


def _lineage(
    *,
    kind: str,
    rationale: str,
    interfaces: tuple[DecisionInterface, ...],
    certificate_set: str,
    result_certificate: str,
    certified_result: str,
    upstream_links: tuple[CertificateLink, ...] = (),
) -> DecisionLineage:
    return DecisionLineage(
        kind=kind,
        rationale=rationale,
        interfaces=interfaces,
        upstream_links=upstream_links,
        certificate_set=_one_field(certificate_set),
        result_certificate=_one_field(result_certificate),
        certified_result=_one_field(certified_result),
    )


def _single_collective_lineage(
    *,
    rationale: str,
    interface_identity: str,
    configurations: str,
    rosters: str,
    submissions: str,
    outcomes: str,
    certificate_set: str,
    result_certificate: str,
    certified_result: str,
) -> DecisionLineage:
    return _lineage(
        kind="collective-result",
        rationale=rationale,
        interfaces=(
            _interface(
                interface_identity,
                configurations=configurations,
                rosters=rosters,
                submissions=submissions,
                outcomes=outcomes,
            ),
        ),
        certificate_set=certificate_set,
        result_certificate=result_certificate,
        certified_result=certified_result,
    )


def _branch(
    number: int,
    key: str,
    *,
    extra: str = "",
    dynamic: bool = False,
    dynamic_subtype: str = "collective",
    holders: tuple[str, ...] | None = None,
    inherit: bool = True,
    authorizations: tuple[AuthorizationSpec, ...] = (),
    observations: tuple[ObservationSpec, ...] = (),
) -> RuleBranch:
    card = CARDS[number - 1]
    return RuleBranch(
        card=card,
        key=key,
        fields=tuple(dict.fromkeys(((*card.fields,) if inherit else ()) + _fields(extra))),
        dynamic=dynamic,
        dynamic_subtype=dynamic_subtype if dynamic else "static",
        authority_holders=card.holders if holders is None else holders,
        authorizations=authorizations,
        observations=observations,
    )


BRANCHES_001_025 = (
    _branch(1, "enumerated_competence", extra="$enumerated_competence@EnumeratedCompetenceScope"),
    _branch(2, "positive_residual_classification", extra="$region@RegionScope,$residual_competence@ResidualCompetenceScope,PositiveResidualClassification@ResidualClassificationScope"),
    _branch(3, "reviewable_displacement", extra="$showing@SubsidiarityEvidenceScope,CrossBoundaryCommonEqualityOrCapacity@ShowingClassificationScope,PublicReasonsComplete@ReasonsScope"),
    _branch(4, "interim_allocation", extra="$interim_holder@InterimAllocationScope,$reviewable_order@ReviewableOrderScope"),
    _branch(5, "unused_council_return", extra="$bill@BillScope,$law@LawScope,NoCouncilReturnUsed@CouncilStageScope"),
    _branch(5, "same_rule_repassage", extra="$bill@BillScope,$law@LawScope,SameRuleRepassageComplete@CouncilStageScope"),
    _branch(6, "revenue_authorization", extra="$revenue_measure@RevenueMeasureScope"),
    _branch(7, "appropriation_authorization", extra="$appropriation@AppropriationMeasureScope"),
    _branch(8, "public_oversight", extra="$oversight_subject@OversightSubjectScope,$oversight_instrument@OversightInstrumentScope,PublicReasonsComplete@ReasonsScope"),
    _branch(9, "amendment_initiation", extra="$proposal@AmendmentProposalScope,$assembly_act@AssemblyActScope"),
    _branch(10, "assembly_election", dynamic=True, extra="$candidate_or_list@CandidateOrListScope,$submission_set@SubmissionSetScope,$tally_rule@TallyRuleScope,$result_certificate@ResultCertificateScope,$recount@RecountScope"),
    _branch(11, "regional_delegation", dynamic=True, extra="$result_certificate@ResultCertificateScope,$regional_result@RegionalDelegationResultScope"),
    _branch(12, "one_time_return", extra="$bill@BillScope,$first_passage@FirstPassageScope,$unused_return@UnusedReturnScope,$public_reasons@PublicReasonsScope"),
    _branch(13, "regional_competence_settlement", inherit=False, dynamic=True, extra="$settlement@RegionalSettlementScope,$council_roster@CouncilRosterScope,$consent_result@CouncilResultScope,FederalSettlementConsentOnly@ConsentScope,RegionalCompetenceSettlement@SettlementKindScope,FullAggregateRegionalWeight@RegionalWeightRuleScope,NoGeneralOrdinaryLawVeto@VetoScope"),
    _branch(13, "regional_boundary_settlement", inherit=False, dynamic=True, extra="$settlement@RegionalSettlementScope,$council_roster@CouncilRosterScope,$consent_result@CouncilResultScope,FederalSettlementConsentOnly@ConsentScope,RegionalBoundarySettlement@SettlementKindScope,FullAggregateRegionalWeight@RegionalWeightRuleScope,NoGeneralOrdinaryLawVeto@VetoScope"),
    _branch(13, "equalisation_settlement", inherit=False, dynamic=True, extra="$settlement@RegionalSettlementScope,$council_roster@CouncilRosterScope,$consent_result@CouncilResultScope,FederalSettlementConsentOnly@ConsentScope,EqualisationSettlement@SettlementKindScope,FullAggregateRegionalWeight@RegionalWeightRuleScope,NoGeneralOrdinaryLawVeto@VetoScope"),
    _branch(13, "other_directly_regional_settlement", inherit=False, dynamic=True, extra="$settlement@RegionalSettlementScope,$council_roster@CouncilRosterScope,$consent_result@CouncilResultScope,FederalSettlementConsentOnly@ConsentScope,OtherDirectlyRegionalSettlement@SettlementKindScope,FullAggregateRegionalWeight@RegionalWeightRuleScope,NoGeneralOrdinaryLawVeto@VetoScope"),
    _branch(14, "confidence_mandate", extra="$confidence_certificate@ConfidenceCertificationScope,FSPOW_015@SourcePowerScope"),
    _branch(14, "constructive_successor_mandate", extra="$confidence_certificate@ConfidenceCertificationScope,FSPOW_016@SourcePowerScope"),
    _branch(15, "confidence_certification", dynamic=True, extra="$government_proposal@GovernmentProposalScope,$result_certificate@ResultCertificateScope"),
    _branch(16, "constructive_replacement", dynamic=True, extra="$assembly_roster@AssemblyRosterScope,$confidence_result@ConfidenceResultScope,$result_certificate@ResultCertificateScope,SameUniqueDecisionRemovesAndCertifies@UniqueDecisionScope"),
    _branch(17, "joint_presidential_selection", dynamic=True, extra="$candidate@CandidateScope,$selection_result@SelectionResultScope,$result_certificate@ResultCertificateScope,BothChambersParticipate@JointParticipationScope"),
    _branch(18, "formal_government_appointment", dynamic=True, extra="$confidence_certificate@ConfidenceCertificationScope,FSPOW_015@SourcePowerScope,$confidence_result_lineage@ConfidenceResultLineageScope,$appointment_result@AppointmentResultScope"),
    _branch(19, "promulgation", extra="$enactment@ValidEnactmentScope"),
    _branch(20, "certificate_receipt", extra="$certificate_kind@CertificateKindScope,$certificate_source@CertificateSourceScope,$certificate_version@CertificateVersionScope,$certificate_lineage@CertificateLineageScope"),
    _branch(21, "refusal_trigger", extra="$trigger@ContinuityTriggerScope,PositiveRefusalTrigger@TriggerKindScope"),
    _branch(21, "vacancy_trigger", extra="$trigger@ContinuityTriggerScope,PositiveVacancyTrigger@TriggerKindScope"),
    _branch(21, "selection_failure_trigger", extra="$trigger@ContinuityTriggerScope,PositiveSelectionFailureTrigger@TriggerKindScope"),
    _branch(21, "removal_trigger", extra="$trigger@ContinuityTriggerScope,PositiveRemovalTrigger@TriggerKindScope"),
    _branch(22, "case_specific_relief", extra="$admitted_facts@AdmittedFactsScope,$reasoned_decision@ReasonedDecisionScope"),
    _branch(23, "constitutional_invalidation", extra="$admitted_facts@AdmittedFactsScope,$reasoned_decision@ReasonedDecisionScope,$constitutional_case@ConstitutionalCaseScope"),
    _branch(24, "election_procedure_review", inherit=False, extra="$election_result@ElectionProcedureResultScope,$procedure@ProcedureScope,ConstitutionalLegalityReviewOnly@ProcedureReviewScope,SeparatelyTypedUnderlyingResults@ResultSeparationScope,$reasoned_decision@ReasonedDecisionScope,$remedy@RemedyScope"),
    _branch(24, "amendment_procedure_review", inherit=False, extra="$amendment_result@AmendmentProcedureResultScope,$procedure@ProcedureScope,ConstitutionalLegalityReviewOnly@ProcedureReviewScope,SeparatelyTypedUnderlyingResults@ResultSeparationScope,$reasoned_decision@ReasonedDecisionScope,$remedy@RemedyScope"),
    _branch(24, "secession_procedure_review", inherit=False, extra="$secession_result@SecessionProcedureResultScope,$procedure@ProcedureScope,ConstitutionalLegalityReviewOnly@ProcedureReviewScope,SeparatelyTypedUnderlyingResults@ResultSeparationScope,$reasoned_decision@ReasonedDecisionScope,$remedy@RemedyScope"),
    _branch(25, "alternate_composition_panel", extra="$uninvolved_membership@UninvolvedMembershipScope,$uninvolved_act@UninvolvedActScope,$reasoned_decision@ReasonedDecisionScope"),
)



SELECTOR_BODIES = ("FSBOD_01", "FSBOD_02", "FSBOD_03", "FSBOD_21")
CONFIRMING_BODIES = ("FSBOD_02", "FSBOD_03", "FSBOD_21")
SELECTOR_KEY_BY_BODY = {
    "FSBOD_01": "people",
    "FSBOD_02": "assembly",
    "FSBOD_03": "council",
    "FSBOD_21": "regional_local",
}


BRANCHES_026_035 = (
    _branch(
        26,
        "qualification_function",
        extra="$referral@QualificationReferralScope,$nominee@NomineeScope,$office@TargetOfficeScope,$criteria@QualificationCriteriaScope,$evidence_record@AdmittedEvidenceScope,$reasons@PublicReasonsScope,DedicatedAppointmentsQualificationFunction@FunctionScope,AuditAcquiresNoQualificationFunction@AuditBoundaryScope",
    ),
    _branch(
        27,
        "reasoned_qualified_determination",
        inherit=False,
        extra="$decision_record@QualificationDecisionRecordScope,$nominee@NomineeScope,$office@TargetOfficeScope,$criteria@QualificationCriteriaScope,$evidence_record@AdmittedEvidenceScope,$reasons@PublicReasonsScope,ReasonedQualifiedDetermination@QualificationDispositionScope,$expiry@DecisionExpiryScope,SilenceIsNoDisposition@SilenceScope",
    ),
    _branch(
        27,
        "reasoned_adverse_qualification",
        inherit=False,
        extra="$decision_record@QualificationDecisionRecordScope,$nominee@NomineeScope,$office@TargetOfficeScope,$criteria@QualificationCriteriaScope,$evidence_record@AdmittedEvidenceScope,$reasons@PublicReasonsScope,ReasonedUnqualifiedDetermination@QualificationDispositionScope,$expiry@DecisionExpiryScope,SilenceIsNoDisposition@SilenceScope",
    ),
    *tuple(
        _branch(
            28,
            f"{SELECTOR_KEY_BY_BODY[holder]}_selector_configuration",
            inherit=False,
            holders=(holder,),
            extra=f"$office@OfficeScope,$institution@InstitutionScope,$seat@SeatScope,$selector_configuration@SelectorConfigurationScope,$qualification_authority@QualificationAuthorityScope,$fallback_configuration@FallbackConfigurationScope,{holder}@SelectedHolderScope,OpenNominationConfiguration@NominationScope,NoMajorityDirectOrDeFactoControl@AntiCaptureScope",
        )
        for holder in SELECTOR_BODIES
    ),
    *tuple(
        _branch(
            28,
            f"{SELECTOR_KEY_BY_BODY[holder]}_appointment_selection",
            inherit=False,
            dynamic=True,
            holders=(holder,),
            extra=f"$office@OfficeScope,$institution@InstitutionScope,$seat@SeatScope,$selector_configuration@SelectorConfigurationScope,$qualification_authority@QualificationAuthorityScope,$fallback_configuration@FallbackConfigurationScope,$term@FiniteNonrenewableStaggeredTermScope,$qualified_nominee@QualifiedNomineeScope,$selection_certificate@SelectionCertificateScope,{holder}@SelectedHolderScope,OpenNominationAndQualificationComplete@AppointmentDispositionScope,CompletedDividedSourceSelection@SelectionDispositionScope,NoMajorityDirectOrDeFactoControl@AntiCaptureScope",
        )
        for holder in SELECTOR_BODIES
    ),
    *tuple(
        _branch(
            29,
            f"{SELECTOR_KEY_BY_BODY[holder]}_removal_confirmation",
            dynamic=True,
            holders=(holder,),
            extra=f"$office@OfficeScope,$term@CurrentOfficeTermScope,$removal_target@RemovalTargetScope,$stated_cause@RemovalCauseScope,$admitted_evidence@AdmittedEvidenceScope,$independent_factfinder@IndependentFactfinderScope,{holder}@SelectedConfirmingInstitutionScope,DistinctCrossBodyConfirmation@ConfirmationScope",
        )
        for holder in CONFIRMING_BODIES
    ),
    *tuple(
        _branch(
            30,
            f"{SELECTOR_KEY_BY_BODY[holder]}_{trigger_key}_fallback_appointment",
            dynamic=True,
            holders=(holder,),
            extra=f"$trigger@FallbackTriggerScope,{trigger_value}@FallbackTriggerKindScope,$fallback_configuration@FallbackConfigurationScope,$qualified_nominee@QualifiedNomineeScope,$seat@SeatScope,$temporary_scope@TemporaryAuthorityScope,$start@StartConditionScope,{holder}@SelectedHolderScope,PredeclaredFallbackOnly@FallbackBoundaryScope,RestorationOrLawfulAppointmentEndsFallback@RestorationScope",
        )
        for trigger_key, trigger_value in (
            ("missing_source", "PositiveMissingSourceFinding"),
            ("captured_source", "PositiveCapturedSourceFinding"),
            ("conflicted_source", "PositiveConflictedSourceFinding"),
            ("qualifying_silence", "PositiveQualifyingSilenceFinding"),
        )
        for holder in SELECTOR_BODIES
    ),
    _branch(
        31,
        "assembly_term_and_vacancy",
        inherit=False,
        holders=("FSBOD_02",),
        extra="$assembly@AssemblyScope,$finite_term@FiniteTermScope,$current_term_record@CurrentTermRecordScope,$vacancy@PositiveVacancyScope,$fill_route@VacancyFillRouteScope,NoTermOrVacancyChoiceBySilence@SilenceScope,NoExecutiveDissolution@ExecutiveBoundaryScope,NoPolicyTransferByVacancy@VacancyBoundaryScope",
    ),
    _branch(
        31,
        "lawful_early_election",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$assembly@AssemblyScope,$early_election_trigger@PositiveEarlyElectionTriggerScope,$electoral_configuration@ElectoralConfigurationScope,NoExecutiveDissolution@ExecutiveBoundaryScope",
    ),
    _branch(
        32,
        "finite_delegation_tenure",
        inherit=False,
        holders=("FSBOD_03",),
        extra="$region@RegionScope,$regional_legislature@RegionalLegislatureScope,$delegation@DelegationScope,$finite_tenure@FiniteTenureScope,EqualRegionalAggregateWeight@RegionalWeightScope,NoExecutiveAppointment@AppointmentBoundaryScope",
    ),
    _branch(
        32,
        "instruction_scope",
        inherit=False,
        holders=("FSBOD_21",),
        extra="$region@RegionScope,$regional_legislature@RegionalLegislatureScope,$delegation@DelegationScope,$instruction_rule@InstructionRuleScope,EqualRegionalAggregateWeight@RegionalWeightScope",
    ),
    _branch(
        32,
        "proportional_replacement",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_21",),
        extra="$region@RegionScope,$regional_legislature@RegionalLegislatureScope,$delegation@DelegationScope,$replacement_rule@ReplacementRuleScope,$replacement_result@ReplacementResultScope,ProportionalLegislatureDelegation@DelegationRuleScope,EqualRegionalAggregateWeight@RegionalWeightScope,NoExecutiveAppointment@AppointmentBoundaryScope",
    ),
    _branch(
        32,
        "delegation_vacancy_fill",
        inherit=False,
        holders=("FSBOD_21",),
        extra="$region@RegionScope,$regional_legislature@RegionalLegislatureScope,$delegation@DelegationScope,$vacancy@PositiveVacancyScope,$fill_route@VacancyFillRouteScope,EqualRegionalAggregateWeight@RegionalWeightScope",
    ),
    _branch(
        33,
        "executive_composition",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_02",),
        extra="$composition@ExecutiveCompositionScope,$confidence_certificate@ConfidenceCertificationScope,$current_mandate@CurrentAssemblyMandateScope,FSPOW_015@SourcePowerScope,NoDirectPresidentialMandate@MandateBoundaryScope",
    ),
    _branch(
        33,
        "executive_member_replacement",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_04",),
        extra="$replacement@MemberReplacementScope,$successor_certificate@SuccessorCertificationScope,$current_mandate@CurrentAssemblyMandateScope,FSPOW_016@SourcePowerScope,NoReplacementWithoutSuccessor@ReplacementBoundaryScope",
    ),
    _branch(
        33,
        "coordinator_incapacity",
        inherit=False,
        holders=("FSBOD_04",),
        extra="$incapacity_record@PositiveCoordinatorIncapacityScope,$bounded_substitute@BoundedSubstituteScope,$current_mandate@CurrentAssemblyMandateScope,NoBroaderPolicyTransfer@CoordinatorBoundaryScope",
    ),
    _branch(
        33,
        "coordinator_power_boundary",
        inherit=False,
        holders=("FSBOD_04",),
        extra="$coordinator@CoordinatorScope,$current_mandate@CurrentAssemblyMandateScope,NoDecreeDissolutionVetoOrEmergency@CoordinatorBoundaryScope",
    ),
    _branch(
        34,
        "joint_presidential_selection",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_02", "FSBOD_03"),
        extra="$assembly_roster@AssemblyRosterScope,$council_roster@CouncilRosterScope,$candidate@CandidateScope,$selection_configuration@PresidentialSelectionConfigurationScope,$selection_result@SelectionResultScope,FiniteMajorityProducingJointBallot@SelectionRuleScope,AtMostOnePresident@UniqueOfficeScope,BothChambersParticipate@JointParticipationScope",
    ),
    _branch(
        34,
        "alternate_continuity",
        inherit=False,
        holders=("FSBOD_26",),
        extra="$alternate@PredeclaredAlternateScope,$trigger@ContinuityTriggerScope,$positive_trigger_kind@PositiveContinuityTriggerKindScope,$bounded_formal_duties@BoundedFormalDutyScope,NoPolicyPowerTransfer@PolicyPowerScope,$finite_end@EndBoundaryScope",
    ),
    _branch(
        34,
        "cause_only_removal",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_02", "FSBOD_03"),
        extra="$office@OfficeScope,$term@CurrentOfficeTermScope,$stated_cause@RemovalCauseScope,$admitted_evidence@AdmittedEvidenceScope,$independent_factfinder@IndependentFactfinderScope,$cross_body_confirmation@CrossBodyConfirmationScope,$removal_result@RemovalResultScope,FactfinderAndConfirmationDistinct@SeparationScope",
    ),
    *tuple(
        _branch(
            35,
            f"{SELECTOR_KEY_BY_BODY[holder]}_seat_allocation",
            inherit=False,
            holders=(holder,),
            extra=f"$institution@InstitutionScope,$seat@SeatScope,$selector_configuration@SelectorConfigurationScope,$qualification_authority@QualificationAuthorityScope,$term@FiniteNonrenewableStaggeredTermScope,$fallback_configuration@FallbackConfigurationScope,{holder}@SelectedHolderScope,NoMajorityDirectOrDeFactoControl@AntiCaptureScope,ExactSeatConfiguration@SeatAllocationScope",
        )
        for holder in SELECTOR_BODIES
    ),
)


MULTI_HOME_AUTHORIZATION = (
    AuthorizationSpec("$subject", "PoliticalHomeChoiceAuthority"),
)


BRANCHES_036 = (
    _branch(
        36,
        "ordinary_resident_membership",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$ordinary_residence_evidence@OrdinaryResidenceEvidenceScope,$local_home@LocalPoliticalHomeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,SingleQualifyingConnection@ConnectionDispositionScope,OrdinaryResidenceEstablished@ResidenceDispositionScope,PoliticalMembershipEstablished@MembershipDispositionScope,NoPropertyAddressCitizenshipStatusWealthWorkContributionOrWaitingGate@MembershipBoundaryScope",
    ),
    _branch(
        36,
        "accessible_nonconventional_residence",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$accessible_residence_evidence@ResidenceEvidenceScope,$local_home@LocalPoliticalHomeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,HomelessDisplacedRefugeeOrStatelessOrdinaryResidence@ResidenceDispositionScope,PoliticalMembershipEstablished@MembershipDispositionScope,NoConventionalAddressOrDocumentPerfectionGate@MembershipBoundaryScope",
    ),
    _branch(
        36,
        "multiple_residences_first_choice",
        inherit=False,
        holders=("FSBOD_06",),
        authorizations=MULTI_HOME_AUTHORIZATION,
        observations=(
            ObservationSpec("$subject", "$record", "$first_home", "ChoiceScope"),
        ),
        extra="$subject@SubjectScope,$first_home@FirstQualifyingConnectionScope,$second_home@SecondQualifyingConnectionScope,$first_home@LocalPoliticalHomeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,DistinctQualifyingConnections@ConnectionDispositionScope,ClaimantChosenOnePoliticalHome@HomeChoiceDispositionScope,NoPublicDistrictShopping@HomeChoiceBoundaryScope",
    ),
    _branch(
        36,
        "multiple_residences_second_choice",
        inherit=False,
        holders=("FSBOD_06",),
        authorizations=MULTI_HOME_AUTHORIZATION,
        observations=(
            ObservationSpec("$subject", "$record", "$second_home", "ChoiceScope"),
        ),
        extra="$subject@SubjectScope,$first_home@FirstQualifyingConnectionScope,$second_home@SecondQualifyingConnectionScope,$second_home@LocalPoliticalHomeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,DistinctQualifyingConnections@ConnectionDispositionScope,ClaimantChosenOnePoliticalHome@HomeChoiceDispositionScope,NoPublicDistrictShopping@HomeChoiceBoundaryScope",
    ),
    _branch(
        36,
        "compelled_placement_nonchange",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$last_home@PoliticalHomeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,$placement_record@CompelledPlacementRecordScope,PrisonDetentionInstitutionShelterEvictionPostingOrForcedDisplacement@CompelledPlacementScope,LastUncontestedHomeContinuity@PoliticalHomeDispositionScope,NoPoliticalHomeChange@HomeContinuityScope",
    ),
    _branch(
        36,
        "last_uncontested_home_during_dispute",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$last_home@PoliticalHomeScope,$home_dispute@PoliticalHomeDisputeScope,$regional_home@RegionalPoliticalHomeScope,$common_home@CommonPoliticalHomeScope,LastVoluntaryUncontestedHomeContinues@HomeContinuityScope,PositiveContinuitySource@ContinuityDispositionScope",
    ),
    _branch(
        36,
        "provisional_first_home",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$accessible_evidence@ResidenceEvidenceScope,$attested_cross_jurisdiction_record@CrossJurisdictionRecordScope,$provisional_home@PoliticalHomeScope,$omission_challenge@OmissionChallengeScope,$alternate_reviewer@AlternateReviewerScope,NoPreviousHomeAttestationComplete@PriorHomeDispositionScope,ProvisionalFirstHomeEffective@HomeDispositionScope",
    ),
    _branch(
        36,
        "atomic_home_transfer",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$old_home@OldPoliticalHomeScope,$new_home@NewPoliticalHomeScope,$qualifying_connection@QualifyingConnectionScope,$transfer_record@AtomicTransferRecordScope,OldHomeEndsWhenNewHomeBegins@AtomicTransferDispositionScope,OneTransitionNoGapOrOverlap@AtomicTransferBoundaryScope",
    ),
    _branch(
        36,
        "adult_resident_franchise",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$adult_status_record@GeneralAdulthoodRecordScope,$political_home@PoliticalHomeScope,$decision@DecisionScope,GeneralAdulthoodEstablished@AdulthoodDispositionScope,CurrentResidenceEstablished@ResidenceDispositionScope,EqualGeneralBallot@FranchiseDispositionScope,NoCustodyStatusDisqualification@FranchiseBoundaryScope,NoCitizenshipPropertyDurationOrHigherAgeGate@FranchiseBoundaryScope",
    ),
    _branch(
        36,
        "adult_resident_candidacy",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$adult_status_record@GeneralAdulthoodRecordScope,$political_home@PoliticalHomeScope,$office@OfficeScope,GeneralAdulthoodEstablished@AdulthoodDispositionScope,CurrentResidenceEstablished@ResidenceDispositionScope,CandidacyUsesSameAdultResidentBaseline@CandidacyDispositionScope,NoCustodyStatusDisqualification@CandidacyBoundaryScope,NoCitizenshipPropertyDurationOrHigherAgeGate@CandidacyBoundaryScope",
    ),
    _branch(
        36,
        "unique_accepted_submission",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$decision@DecisionScope,$accepted_submission@AcceptedSubmissionScope,$old_home@OldPoliticalHomeScope,$new_home@NewPoliticalHomeScope,AcceptedSubmissionRemainsOnlySubmission@SubmissionDispositionScope,NoSecondSubmissionAfterMove@NoDoubleSubmissionScope,NoRetroactiveBallotInvalidation@SubmissionContinuityScope",
    ),
    _branch(
        36,
        "established_adulthood_continuity",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$adult_status_record@GeneralAdulthoodRecordScope,$evidence_dispute@AdulthoodEvidenceDisputeScope,LastUncontestedAdultStatusContinues@AdulthoodContinuityScope,NoRetroactiveAdulthoodWithdrawal@AdulthoodBoundaryScope,AdulthoodRecordEvidenceNotSource@AdulthoodEvidenceScope",
    ),
    _branch(
        36,
        "provisional_adulthood_expiring_opportunity",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$credible_threshold_evidence@AdulthoodEvidenceScope,$expiring_opportunity@ExpiringPoliticalOpportunityScope,$provisional_adult_status@ProvisionalAdulthoodScope,$review_record@AdulthoodReviewRecordScope,$alternate_reviewer@AlternateReviewerScope,ProvisionalAdultStatusEffective@AdulthoodDispositionScope,PositiveContraryDeterminationRequiredToEnd@AdulthoodEndScope",
    ),
    _branch(
        36,
        "positive_nonresident_disposition",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$nonresident_record@NonresidentRecordScope,NonresidentStatusEstablished@ResidenceDispositionScope,NoGeneralGovernmentBallotOrCandidacy@NonresidentPoliticalBoundaryScope,UniversalStandingProtectionAndPetitionPreserved@RightsContinuityScope",
    ),
    _branch(
        36,
        "former_resident_return_without_ballot",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$prior_connection@PriorResidenceConnectionScope,$return_record@ReturnRecordScope,FormerResidenceEstablished@PriorResidenceDispositionScope,ReturnRightPreserved@ReturnDispositionScope,NoDiasporaBallot@ReturnBoundaryScope",
    ),
    _branch(
        36,
        "office_move_continuity",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$subject@SubjectScope,$office_or_candidacy@OfficeOrCandidacyScope,$old_home@OldPoliticalHomeScope,$new_home@NewPoliticalHomeScope,$current_mandate_or_candidacy@CurrentMandateOrCandidacyScope,$successor@SuccessorScope,LawfulCurrentOfficeOrCandidacyEstablished@OfficeDispositionScope,MoveCannotSilentlyEndOrExtendMandate@OfficeMoveBoundaryScope,ProspectiveContinuingResidenceConditionOnly@OfficeConditionScope,LawfulSuccessorAndContinuityRequired@OfficeContinuityScope",
    ),
)


BRANCHES_037_051 = (
    _branch(
        37,
        "ordinary_amendment",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01", "FSBOD_02"),
        extra="$proposal@AmendmentProposalScope,$assembly_roster@AssemblyRosterScope,$assembly_result@AssemblyResultScope,$referendum_roster@ReferendumRosterScope,$referendum_result@ReferendumResultScope,FullAssemblyTwoThirdsApproval@AssemblyThresholdScope,NationalAffirmativeExceedsNegative@ReferendumDispositionScope,NoDirectRegionalSettlementChange@RegionalSettlementDispositionScope,CompatibilityAndCorridorReviewComplete@ReviewDispositionScope,NoTurnoutQuorum@QuorumBoundaryScope,TieOrMissingCertificationWithholdsOnly@FailureBoundaryScope",
    ),
    _branch(
        37,
        "directly_regional_settlement_amendment",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01", "FSBOD_02", "FSBOD_03"),
        extra="$proposal@AmendmentProposalScope,$assembly_roster@AssemblyRosterScope,$assembly_result@AssemblyResultScope,$referendum_roster@ReferendumRosterScope,$referendum_result@ReferendumResultScope,$council_roster@CouncilRosterScope,$council_result@CouncilResultScope,FullAssemblyTwoThirdsApproval@AssemblyThresholdScope,NationalAffirmativeExceedsNegative@ReferendumDispositionScope,DirectRegionalSettlementOtherThanCompetenceOrBoundary@RegionalSettlementDispositionScope,CouncilMajorityOfFullAggregateRegionalWeight@CouncilDispositionScope,CompatibilityAndCorridorReviewComplete@ReviewDispositionScope,NoTurnoutQuorum@QuorumBoundaryScope,TieOrMissingCertificationWithholdsOnly@FailureBoundaryScope",
    ),
    _branch(
        37,
        "competence_boundary_amendment",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01", "FSBOD_02", "FSBOD_03", "FSBOD_21"),
        extra="$proposal@AmendmentProposalScope,$assembly_roster@AssemblyRosterScope,$assembly_result@AssemblyResultScope,$referendum_roster@ReferendumRosterScope,$referendum_result@ReferendumResultScope,$council_roster@CouncilRosterScope,$council_result@CouncilResultScope,$affected_region_roster@AffectedRegionRosterScope,$affected_region_results@AffectedRegionResultsScope,FullAssemblyTwoThirdsApproval@AssemblyThresholdScope,NationalAffirmativeExceedsNegative@ReferendumDispositionScope,CompetenceOrBoundaryChange@RegionalSettlementDispositionScope,CouncilMajorityOfFullAggregateRegionalWeight@CouncilDispositionScope,EveryAffectedRegionAffirmativeExceedsNegative@AffectedRegionDispositionScope,CompleteAffectedRegionRoster@AffectedRegionCompletenessScope,CompatibilityAndCorridorReviewComplete@ReviewDispositionScope,NoTurnoutQuorum@QuorumBoundaryScope,TieOrMissingCertificationWithholdsOnly@FailureBoundaryScope",
    ),
    _branch(
        38,
        "constitutional_initiative_docket",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$proposal@ConstitutionalInitiativeProposalScope,$elector_roster@ElectorRosterScope,$signature_submissions@SignatureSubmissionSetScope,$threshold_configuration@ConstitutionalAccessThresholdScope,$distribution_configuration@GeographicDistributionScope,$qualification_certificate@QualificationCertificateScope,$docket_target@DocketTargetScope,$assembly_vote_target@AssemblyVoteTargetScope,UniqueAuthenticatedSignatures@SignatureDispositionScope,MandatoryDocketAndVoteDutiesOnly@DocketDispositionScope,NoDocketOrVoteOccurrenceClaim@OperationBoundaryScope,NoAmendmentApprovalByDocket@ApprovalBoundaryScope",
    ),
    *tuple(
        _branch(
            39,
            key,
            inherit=False,
            dynamic=True,
            holders=("FSBOD_01",),
            extra=f"$proposal@OrdinaryInitiativeProposalScope,$counterproposal@CounterproposalScope,$compatibility_review@CompatibilityReviewScope,PositiveCompatibilityAndCorridorReviewPassed@CompatibilityReviewDispositionScope,$regional_competence_hearing@RegionalCompetenceHearingScope,$referendum_roster@ReferendumRosterScope,$initiative_result@InitiativeResultScope,$counterproposal_result@CounterproposalResultScope,$unique_choice_certificate@UniqueChoiceCertificateScope,{disposition}@OperativeChoiceDispositionScope,ExactlyOneOperativeChoice@UniqueOperativeResultScope,NoConstitutionalOrCompetenceChange@InitiativeBoundaryScope",
        )
        for key, disposition in (
            ("initiative_only_wins", "CertifiedInitiativeOnlyWinner"),
            ("counterproposal_only_wins", "CertifiedCounterproposalOnlyWinner"),
            ("both_pass_initiative_larger_share", "CertifiedInitiativeHigherAffirmativeShare"),
            ("both_pass_counterproposal_larger_share", "CertifiedCounterproposalHigherAffirmativeShare"),
        )
    ),
    _branch(
        39,
        "tie_preserves_current_law",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$proposal@OrdinaryInitiativeProposalScope,$current_law@CurrentLawScope,$current_law_version@CurrentLawVersionScope,$tie_failure_certificate@InitiativeFailureCertificateScope,PositiveCertifiedTie@InitiativeFailureDispositionScope,NoOperativeInitiativeChoice@UniqueOperativeResultScope,CurrentLawContinuityFromPositiveSource@ContinuityDispositionScope",
    ),
    _branch(
        39,
        "neither_passes_preserves_current_law",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$proposal@OrdinaryInitiativeProposalScope,$current_law@CurrentLawScope,$current_law_version@CurrentLawVersionScope,$neither_failure_certificate@InitiativeFailureCertificateScope,PositiveCertifiedNeitherPasses@InitiativeFailureDispositionScope,NoOperativeInitiativeChoice@UniqueOperativeResultScope,CurrentLawContinuityFromPositiveSource@ContinuityDispositionScope",
    ),
    _branch(
        40,
        "constructive_recall_success",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$target@RecallTargetScope,$office@DirectSingleHolderOfficeScope,$current_term@CurrentOfficeTermScope,$recall_proposal@RecallProposalScope,$successor_slate@SuccessorSlateScope,$elector_roster@ElectorRosterScope,$submissions@SubmissionSetScope,$removal_result@RemovalResultScope,$successor_result@SuccessorResultScope,RemovalSupportExceedsOpposition@RemovalDispositionScope,UniqueSuccessorCertified@SuccessorDispositionScope,AtomicRemovalAndSuccessor@RecallDispositionScope,AssembliesAndExecutiveCouncilExcluded@RecallBoundaryScope",
    ),
    _branch(
        40,
        "recall_failure_current_term_continuity",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$target@RecallTargetScope,$office@DirectSingleHolderOfficeScope,$current_term@CurrentOfficeTermScope,$recall_failure_certificate@RecallFailureCertificateScope,PositiveCertifiedRecallFailureTieOrNoSuccessor@RecallFailureDispositionScope,NoRecallEffect@RecallDispositionScope,CurrentTermContinuityFromPositiveSource@ContinuityDispositionScope,NoIncumbentExtension@IncumbentBoundaryScope",
    ),
    _branch(
        41,
        "last_lawful_government_caretaker",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_04",),
        extra="$formation_failure@CertifiedFormationFailureScope,$last_lawful_government@LastLawfulGovernmentScope,$current_mandate@CurrentMandateScope,$caretaker_scope@CaretakerScope,$start@StartConditionScope,$constitutional_deadline@ConstitutionalDeadlineScope,ExistingLawAndFloorOnly@CaretakerPurposeScope,NoAvoidableIrreversiblePolicy@CaretakerBoundaryScope",
    ),
    _branch(
        41,
        "nonpolitical_administrative_successor",
        inherit=False,
        holders=("FSBOD_04",),
        extra="$condition_record@CaretakerConditionScope,$positive_condition_kind@PositiveNonpoliticalCaretakerConditionKindScope,$succession_configuration@PredeclaredNonpoliticalSuccessionScope,$administrative_holder@AdministrativeHolderScope,$start@StartConditionScope,UnlawfulRemovedCapturedOrCollectivelyIncapacitated@CaretakerConditionDispositionScope,EssentialContinuityOnly@CaretakerPurposeScope,NoOrdinaryPolicy@CaretakerBoundaryScope",
    ),
    _branch(
        42,
        "fresh_election_call",
        inherit=False,
        holders=("FSBOD_06",),
        extra="$deadline_configuration@ConstitutionalDeadlineConfigurationScope,$deadline_record@DeadlineRecordScope,$formation_failure@FormationFailureScope,$caretaker_record@CaretakerRecordScope,PositiveDeadlinePassed@DeadlineDispositionScope,FreshElectionCallAuthorityOnly@ElectionCallDispositionScope,NoOutsideClockOrElectionOccurrence@OperationBoundaryScope",
    ),
    _branch(
        43,
        "essential_budget_continuity",
        inherit=False,
        holders=("FSBOD_07",),
        extra="$budget_deadlock@PositiveBudgetDeadlockScope,$last_lawful_appropriation@LastLawfulAppropriationScope,$essential_set@EnumeratedEssentialContinuitySetScope,$public_legal_basis@PublicLegalBasisScope,$continuity_limit@ContinuityLimitScope,TemporaryEssentialSpendingOnly@BudgetContinuityDispositionScope,NoNewProgrammeOrPermanentSpending@SpendingBoundaryScope,NoDeliveryOrCapacityEffect@DeliveryBoundaryScope",
    ),
    _branch(
        43,
        "valid_budget_ends_continuity",
        inherit=False,
        holders=("FSBOD_07",),
        extra="$valid_budget@PositiveValidBudgetScope,$ended_continuity_record@EndedContinuityRecordScope,ValidBudgetPositivelyEndsContinuity@BudgetEndDispositionScope,NoAuthorityPersistenceBySilence@ContinuityBoundaryScope",
    ),
    _branch(
        43,
        "continuity_limit_ends_authority",
        inherit=False,
        holders=("FSBOD_07",),
        extra="$continuity_limit_record@PositiveContinuityLimitRecordScope,$ended_continuity_record@EndedContinuityRecordScope,ContinuityLimitPositivelyEndsAuthority@BudgetEndDispositionScope,NoAuthorityPersistenceByMissingTime@ContinuityBoundaryScope",
    ),
    _branch(
        44,
        "common_office_transfer",
        inherit=False,
        dynamic=True,
        dynamic_subtype="certificate",
        holders=("FSBOD_05",),
        extra="$office@OfficeScope,$predecessor@PredecessorScope,$successor@SuccessorScope,$predecessor_mandate@CurrentPredecessorMandateScope,$successor_certificate@SuccessorCertificateScope,$effective_office_transfer@OfficeTransferRecordScope,FSBOD_05@SelectedExecutorScope,AtomicCurrentAuthorityTransfer@TransferDispositionScope,OutgoingRefusalCannotExtendMandate@IncumbentBoundaryScope",
    ),
    _branch(
        44,
        "regional_local_office_transfer",
        inherit=False,
        dynamic=True,
        dynamic_subtype="certificate",
        holders=("FSBOD_21",),
        extra="$office@OfficeScope,$predecessor@PredecessorScope,$successor@SuccessorScope,$predecessor_mandate@CurrentPredecessorMandateScope,$successor_certificate@SuccessorCertificateScope,$effective_office_transfer@OfficeTransferRecordScope,FSBOD_21@SelectedExecutorScope,AtomicCurrentAuthorityTransfer@TransferDispositionScope,OutgoingRefusalCannotExtendMandate@IncumbentBoundaryScope",
    ),
    _branch(
        45,
        "opening_referendum",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$proposal@SecessionProposalScope,$secession_configuration@SecessionConfigurationScope,$territory@TerritoryScope,$affected_population@AffectedPopulationScope,$opening_roster@OpeningReferendumRosterScope,$opening_submissions@UniqueOpeningSubmissionSetScope,$opening_result@OpeningReferendumResultScope,CompleteUniqueOpeningSubmissions@SubmissionDispositionScope,OpeningAffirmativeExceedsNegative@OpeningDispositionScope,NegotiationOpeningAuthorityOnly@StageDispositionScope,OpeningReferendumIsNotExit@ReferendumBoundaryScope,NoMilitaryOrWithholdingRoute@CoercionBoundaryScope",
    ),
    _branch(
        45,
        "completed_negotiation",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_02", "FSBOD_03"),
        extra="$proposal@SecessionProposalScope,$secession_configuration@SecessionConfigurationScope,$territory@TerritoryScope,$affected_population@AffectedPopulationScope,$opening_result@OpeningReferendumResultScope,OpeningAffirmativeExceedsNegative@OpeningDispositionScope,$federal_agreement@FederalAgreementScope,PositiveFederalAgreementComplete@AgreementDispositionScope,$rights_review@RightsAndMinorityReviewScope,PositiveRightsAndMinorityReviewPassed@RightsReviewDispositionScope,$settlement@CompleteSecessionSettlementScope,PositiveSettlementComplete@SettlementDispositionScope,NegotiatedAgreementStageAuthorityOnly@StageDispositionScope,NoExitBeforeFinalRatification@StageBoundaryScope,NoMilitaryOrWithholdingRoute@CoercionBoundaryScope",
    ),
    _branch(
        45,
        "final_exit_no_collective_impact",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01",),
        extra="$proposal@SecessionProposalScope,$secession_configuration@SecessionConfigurationScope,$territory@TerritoryScope,$affected_population@AffectedPopulationScope,$opening_result@OpeningReferendumResultScope,OpeningAffirmativeExceedsNegative@OpeningDispositionScope,$federal_agreement@FederalAgreementScope,PositiveFederalAgreementComplete@AgreementDispositionScope,$rights_review@RightsAndMinorityReviewScope,PositiveRightsAndMinorityReviewPassed@RightsReviewDispositionScope,$settlement@CompleteSecessionSettlementScope,PositiveSettlementComplete@SettlementDispositionScope,$final_roster@AffectedPopulationRosterScope,$final_submissions@UniqueFinalSubmissionSetScope,$final_result@FinalRatificationResultScope,CompleteUniqueFinalSubmissions@SubmissionDispositionScope,PositiveFinalAffectedPopulationRatificationPassed@RatificationDispositionScope,PositiveNoCollectiveTitleOrSovereigntyImpact@CollectiveImpactDispositionScope,AllRequiredSecessionStagesComplete@StageDispositionScope,InternalConstitutionalExitOnly@ExitDispositionScope,NoMilitaryOrWithholdingRoute@CoercionBoundaryScope",
    ),
    _branch(
        45,
        "final_exit_with_collective_consent",
        inherit=False,
        dynamic=True,
        holders=("FSBOD_01", "FSBOD_21"),
        extra="$proposal@SecessionProposalScope,$secession_configuration@SecessionConfigurationScope,$territory@TerritoryScope,$affected_population@AffectedPopulationScope,$opening_result@OpeningReferendumResultScope,OpeningAffirmativeExceedsNegative@OpeningDispositionScope,$federal_agreement@FederalAgreementScope,PositiveFederalAgreementComplete@AgreementDispositionScope,$rights_review@RightsAndMinorityReviewScope,PositiveRightsAndMinorityReviewPassed@RightsReviewDispositionScope,$settlement@CompleteSecessionSettlementScope,PositiveSettlementComplete@SettlementDispositionScope,$collective_impact@CollectiveTitleOrSovereigntyImpactScope,PositiveCollectiveTitleOrSovereigntyImpact@CollectiveImpactDispositionScope,$actual_collective_consent@ActualCollectiveConsentScope,PositiveActualCollectiveConsent@CollectiveConsentDispositionScope,$final_roster@AffectedPopulationRosterScope,$final_submissions@UniqueFinalSubmissionSetScope,$final_result@FinalRatificationResultScope,CompleteUniqueFinalSubmissions@SubmissionDispositionScope,PositiveFinalAffectedPopulationRatificationPassed@RatificationDispositionScope,AllRequiredSecessionStagesComplete@StageDispositionScope,InternalConstitutionalExitOnly@ExitDispositionScope,NoDeemedConsentOrWithholdingRoute@CoercionBoundaryScope",
    ),
    _branch(46, "protected_local_budget", extra="$locality@LocalityScope,$local_holder@LocalHolderScope,$budget_instrument@LocalBudgetInstrumentScope,$term@CurrentHolderTermScope,EqualisationPortabilityAndSubsidiarityBoundaries@LocalBudgetBoundaryScope"),
    _branch(47, "protected_local_administration", extra="$locality@LocalityScope,$local_holder@LocalHolderScope,$administrative_scope@LocalAdministrativeScope,$term@CurrentHolderTermScope,ConstitutionalLocalMinimum@LocalMinimumScope,NoPleasureRetraction@RetractionBoundaryScope"),
    _branch(48, "protected_local_public_space", extra="$locality@LocalityScope,$public_space@LocalPublicSpaceScope,$management_scope@ManagementScope,StandingEqualityLibertiesFloorDueProcessAndCommons@GuaranteeScope"),
    _branch(49, "protected_local_facility", extra="$locality@LocalityScope,$facility@LocalFacilityScope,$management_scope@ManagementScope,AssignedServiceAdministrationSeparate@ServiceBoundaryScope,NoOwnershipOperationCapacityOrDeliveryInference@DeliveryBoundaryScope"),
    _branch(
        50,
        "constitutionally_assigned_local_service",
        inherit=False,
        extra="$locality@LocalityScope,$service@AssignedServiceScope,$assignment_instrument@ConstitutionalAssignmentInstrumentScope,$resource_authority_interface@ResourceAuthorityInterfaceScope,AdministrationAuthorityOnly@ServiceDispositionScope,NoDeliveryOrCapacityClaim@DeliveryBoundaryScope",
    ),
    _branch(
        50,
        "regionally_assigned_local_service",
        inherit=False,
        extra="$locality@LocalityScope,$service@AssignedServiceScope,$assignment_instrument@RegionalAssignmentInstrumentScope,$resource_authority_interface@ResourceAuthorityInterfaceScope,AdministrationAuthorityOnly@ServiceDispositionScope,NoDeliveryOrCapacityClaim@DeliveryBoundaryScope",
    ),
    _branch(
        51,
        "referral_recipient_duty",
        inherit=False,
        extra="$referral@QualificationReferralScope,$nominee@NomineeScope,$office@TargetOfficeScope,$authorized_referrer@AuthorizedReferrerScope,$qualification_mandate@QualificationMandateScope,$criteria@QualificationCriteriaScope,$evidence_route@EvidenceRouteScope,$referral_window@ReferralWindowScope,IndependentRecipientDutyToAct@DutyDispositionScope,ReferralAloneIsNoDecision@ReferralBoundaryScope",
    ),
    _branch(
        51,
        "reasoned_disposition_completion",
        inherit=False,
        extra="$referral@QualificationReferralScope,$qualification_record@QualificationRecordScope,$reasoned_disposition@ReasonedQualificationDispositionScope,$positive_disposition_kind@PositiveQualifiedOrAdverseDispositionScope,$correction@CorrectionScope,$challenge@ChallengeScope,OrdinaryDutyCompletion@DutyDispositionScope,SilenceIsNoQualificationDisposition@SilenceBoundaryScope",
    ),
    _branch(
        51,
        "inaction_alternate_remedy",
        inherit=False,
        extra="$referral@QualificationReferralScope,$inaction_or_unavailability_record@PositiveInactionOrUnavailabilityScope,$positive_condition_kind@PositiveInactionOrUnavailabilityDispositionScope,$alternate@PredeclaredAlternateScope,$consequence@InactionConsequenceScope,$temporary_claimant_continuity@TemporaryClaimantContinuityScope,SilenceIsNoApprovalOrRejection@SilenceBoundaryScope",
    ),
)


ALL_BRANCHES = (
    *BRANCHES_001_025,
    *BRANCHES_026_035,
    *BRANCHES_036,
    *BRANCHES_037_051,
)

DECISION_LINEAGES: dict[tuple[int, str], DecisionLineage] = {
    (10, "assembly_election"): _single_collective_lineage(
        rationale="certified proportional Assembly result",
        interface_identity="$assembly_election_interface",
        configurations="$configuration@ElectoralConfigurationScope",
        rosters="$eligible_roster@EligibleRosterScope",
        submissions="$submission_set@SubmissionSetScope",
        outcomes="$result@ResultScope",
        certificate_set="$assembly_election_certificate_set@CertificateSetScope",
        result_certificate="$result_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (11, "regional_delegation"): _single_collective_lineage(
        rationale="certified regional delegation result",
        interface_identity="$regional_delegation_interface",
        configurations="$regional_delegation_configuration@RegionalDelegationConfigurationScope",
        rosters="$eligible_roster@EligibleRosterScope",
        submissions="$regional_delegation_submissions@RegionalDelegationSubmissionSetScope",
        outcomes="$regional_result@RegionalDelegationResultScope",
        certificate_set="$regional_delegation_certificate_set@CertificateSetScope",
        result_certificate="$result_certificate@ResultCertificateScope",
        certified_result="$regional_result@RegionalDelegationResultScope",
    ),
    **{
        (13, key): _single_collective_lineage(
            rationale="certified Regions Council settlement consent",
            interface_identity="$council_consent_interface",
            configurations="$council_consent_configuration@CouncilConsentConfigurationScope",
            rosters="$council_roster@CouncilRosterScope",
            submissions="$council_consent_submissions@CouncilConsentSubmissionSetScope",
            outcomes="$consent_result@CouncilResultScope",
            certificate_set="$council_consent_certificate_set@CertificateSetScope",
            result_certificate="$council_consent_certificate@ResultCertificateScope",
            certified_result="$consent_result@CouncilResultScope",
        )
        for key in (
            "regional_competence_settlement",
            "regional_boundary_settlement",
            "equalisation_settlement",
            "other_directly_regional_settlement",
        )
    },
    (15, "confidence_certification"): _single_collective_lineage(
        rationale="certified Assembly confidence result",
        interface_identity="$confidence_interface",
        configurations="$confidence_configuration@ConfidenceConfigurationScope",
        rosters="$assembly_roster@AssemblyRosterScope",
        submissions="$confidence_submissions@ConfidenceSubmissionSetScope",
        outcomes="$confidence_result@ConfidenceResultScope,$certified_government@CertifiedGovernmentScope",
        certificate_set="$confidence_certificate_set@CertificateSetScope",
        result_certificate="$result_certificate@ResultCertificateScope",
        certified_result="$confidence_result@ConfidenceResultScope",
    ),
    (16, "constructive_replacement"): _single_collective_lineage(
        rationale="single certified constructive-confidence result",
        interface_identity="$constructive_confidence_interface",
        configurations="$constructive_confidence_configuration@ConstructiveConfidenceConfigurationScope",
        rosters="$assembly_roster@AssemblyRosterScope",
        submissions="$constructive_confidence_submissions@ConstructiveConfidenceSubmissionSetScope",
        outcomes="$confidence_result@ConfidenceResultScope",
        certificate_set="$constructive_confidence_certificate_set@CertificateSetScope",
        result_certificate="$result_certificate@ResultCertificateScope",
        certified_result="$confidence_result@ConfidenceResultScope",
    ),
    (17, "joint_presidential_selection"): _lineage(
        kind="collective-result",
        rationale="joint Assembly and Council selection",
        interfaces=(
            _interface(
                "$assembly_presidential_selection_interface",
                configurations="$selection_configuration@PresidentialSelectionConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$assembly_selection_submissions@AssemblySelectionSubmissionSetScope",
                outcomes="$selection_result@SelectionResultScope",
            ),
            _interface(
                "$council_presidential_selection_interface",
                configurations="$selection_configuration@PresidentialSelectionConfigurationScope",
                rosters="$council_roster@CouncilRosterScope",
                submissions="$council_selection_submissions@CouncilSelectionSubmissionSetScope",
                outcomes="$council_selection_result@CouncilSelectionResultScope",
            ),
        ),
        certificate_set="$joint_selection_certificate_set@CertificateSetScope",
        result_certificate="$result_certificate@ResultCertificateScope",
        certified_result="$selection_result@SelectionResultScope",
    ),
    (18, "formal_government_appointment"): _lineage(
        kind="record-certificate-consumption",
        rationale="non-discretionary appointment reuses the confidence decision",
        interfaces=(
            _interface(
                "$upstream_confidence_interface",
                configurations="$confidence_configuration@ConfidenceConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$confidence_submissions@ConfidenceSubmissionSetScope",
                outcomes="$confidence_result@ConfidenceResultScope,$certified_government@CertifiedGovernmentScope,$confidence_result_lineage@ConfidenceResultLineageScope",
            ),
        ),
        upstream_links=(
            _link(
                "$confidence_certificate@ConfidenceCertificationScope",
                "$confidence_result@ConfidenceResultScope",
            ),
        ),
        certificate_set="$confidence_certificate_set@CertificateSetScope",
        result_certificate="$confidence_certificate@ConfidenceCertificationScope",
        certified_result="$appointment_result@AppointmentResultScope",
    ),
    **{
        (28, key): _single_collective_lineage(
            rationale="divided-source appointment selection",
            interface_identity=interface_identity,
            configurations="$selector_configuration@SelectorConfigurationScope",
            rosters=roster,
            submissions=submissions,
            outcomes="$result@ResultScope",
            certificate_set="$selection_certificate_set@CertificateSetScope",
            result_certificate="$selection_certificate@SelectionCertificateScope",
            certified_result="$result@ResultScope",
        )
        for key, interface_identity, roster, submissions in (
            ("people_appointment_selection", "$people_selector_interface", "$people_selector_roster@SelectorRosterScope", "$people_selector_submissions@SelectorSubmissionSetScope"),
            ("assembly_appointment_selection", "$assembly_selector_interface", "$assembly_selector_roster@SelectorRosterScope", "$assembly_selector_submissions@SelectorSubmissionSetScope"),
            ("council_appointment_selection", "$council_selector_interface", "$council_selector_roster@SelectorRosterScope", "$council_selector_submissions@SelectorSubmissionSetScope"),
            ("regional_local_appointment_selection", "$regional_local_selector_interface", "$regional_local_selector_roster@SelectorRosterScope", "$regional_local_selector_submissions@SelectorSubmissionSetScope"),
        )
    },
    **{
        (29, key): _single_collective_lineage(
            rationale="cause finding and cross-body removal confirmation",
            interface_identity=interface_identity,
            configurations="$removal_configuration@RemovalConfigurationScope",
            rosters=roster,
            submissions=submissions,
            outcomes="$result@ResultScope",
            certificate_set="$removal_certificate_set@CertificateSetScope",
            result_certificate="$removal_certificate@ResultCertificateScope",
            certified_result="$result@ResultScope",
        )
        for key, interface_identity, roster, submissions in (
            ("assembly_removal_confirmation", "$assembly_removal_interface", "$assembly_confirming_institution_roster@ConfirmingInstitutionRosterScope", "$assembly_confirmation_submissions@ConfirmationSubmissionSetScope"),
            ("council_removal_confirmation", "$council_removal_interface", "$council_confirming_institution_roster@ConfirmingInstitutionRosterScope", "$council_confirmation_submissions@ConfirmationSubmissionSetScope"),
            ("regional_local_removal_confirmation", "$regional_local_removal_interface", "$regional_local_confirming_institution_roster@ConfirmingInstitutionRosterScope", "$regional_local_confirmation_submissions@ConfirmationSubmissionSetScope"),
        )
    },
    **{
        (30, key): _single_collective_lineage(
            rationale="predeclared temporary fallback appointment",
            interface_identity=interface_identity,
            configurations="$fallback_configuration@FallbackConfigurationScope",
            rosters=roster,
            submissions=submissions,
            outcomes="$result@ResultScope",
            certificate_set="$fallback_appointment_certificate_set@CertificateSetScope",
            result_certificate="$fallback_appointment_certificate@ResultCertificateScope",
            certified_result="$result@ResultScope",
        )
        for key, interface_identity, roster, submissions in (
            ("people_missing_source_fallback_appointment", "$people_missing_source_fallback_interface", "$people_fallback_selector_roster@FallbackSelectorRosterScope", "$people_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("assembly_missing_source_fallback_appointment", "$assembly_missing_source_fallback_interface", "$assembly_fallback_selector_roster@FallbackSelectorRosterScope", "$assembly_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("council_missing_source_fallback_appointment", "$council_missing_source_fallback_interface", "$council_fallback_selector_roster@FallbackSelectorRosterScope", "$council_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("regional_local_missing_source_fallback_appointment", "$regional_local_missing_source_fallback_interface", "$regional_local_fallback_selector_roster@FallbackSelectorRosterScope", "$regional_local_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("people_captured_source_fallback_appointment", "$people_captured_source_fallback_interface", "$people_fallback_selector_roster@FallbackSelectorRosterScope", "$people_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("assembly_captured_source_fallback_appointment", "$assembly_captured_source_fallback_interface", "$assembly_fallback_selector_roster@FallbackSelectorRosterScope", "$assembly_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("council_captured_source_fallback_appointment", "$council_captured_source_fallback_interface", "$council_fallback_selector_roster@FallbackSelectorRosterScope", "$council_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("regional_local_captured_source_fallback_appointment", "$regional_local_captured_source_fallback_interface", "$regional_local_fallback_selector_roster@FallbackSelectorRosterScope", "$regional_local_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("people_conflicted_source_fallback_appointment", "$people_conflicted_source_fallback_interface", "$people_fallback_selector_roster@FallbackSelectorRosterScope", "$people_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("assembly_conflicted_source_fallback_appointment", "$assembly_conflicted_source_fallback_interface", "$assembly_fallback_selector_roster@FallbackSelectorRosterScope", "$assembly_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("council_conflicted_source_fallback_appointment", "$council_conflicted_source_fallback_interface", "$council_fallback_selector_roster@FallbackSelectorRosterScope", "$council_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("regional_local_conflicted_source_fallback_appointment", "$regional_local_conflicted_source_fallback_interface", "$regional_local_fallback_selector_roster@FallbackSelectorRosterScope", "$regional_local_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("people_qualifying_silence_fallback_appointment", "$people_qualifying_silence_fallback_interface", "$people_fallback_selector_roster@FallbackSelectorRosterScope", "$people_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("assembly_qualifying_silence_fallback_appointment", "$assembly_qualifying_silence_fallback_interface", "$assembly_fallback_selector_roster@FallbackSelectorRosterScope", "$assembly_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("council_qualifying_silence_fallback_appointment", "$council_qualifying_silence_fallback_interface", "$council_fallback_selector_roster@FallbackSelectorRosterScope", "$council_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
            ("regional_local_qualifying_silence_fallback_appointment", "$regional_local_qualifying_silence_fallback_interface", "$regional_local_fallback_selector_roster@FallbackSelectorRosterScope", "$regional_local_fallback_selection_submissions@FallbackSelectionSubmissionSetScope"),
        )
    },
    (32, "proportional_replacement"): _single_collective_lineage(
        rationale="proportional regional-legislature replacement",
        interface_identity="$regional_replacement_interface",
        configurations="$replacement_rule@ReplacementRuleScope",
        rosters="$regional_legislature_roster@RegionalLegislatureRosterScope",
        submissions="$replacement_submissions@ReplacementSubmissionSetScope",
        outcomes="$replacement_result@ReplacementResultScope",
        certificate_set="$replacement_certificate_set@CertificateSetScope",
        result_certificate="$replacement_certificate@ResultCertificateScope",
        certified_result="$replacement_result@ReplacementResultScope",
    ),
    (33, "executive_composition"): _lineage(
        kind="record-certificate-consumption",
        rationale="executive composition reuses the certified confidence mandate",
        interfaces=(
            _interface(
                "$executive_composition_confidence_interface",
                configurations="$confidence_configuration@ConfidenceConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$confidence_submissions@ConfidenceSubmissionSetScope",
                outcomes="$current_mandate@CurrentAssemblyMandateScope",
            ),
        ),
        upstream_links=(
            _link(
                "$confidence_certificate@ConfidenceCertificationScope",
                "$current_mandate@CurrentAssemblyMandateScope",
            ),
        ),
        certificate_set="$confidence_certificate_set@CertificateSetScope",
        result_certificate="$confidence_certificate@ConfidenceCertificationScope",
        certified_result="$composition@ExecutiveCompositionScope",
    ),
    (33, "executive_member_replacement"): _lineage(
        kind="record-certificate-consumption",
        rationale="member replacement reuses the constructive-confidence successor",
        interfaces=(
            _interface(
                "$executive_replacement_confidence_interface",
                configurations="$constructive_confidence_configuration@ConstructiveConfidenceConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$constructive_confidence_submissions@ConstructiveConfidenceSubmissionSetScope",
                outcomes="$current_mandate@CurrentAssemblyMandateScope",
            ),
        ),
        upstream_links=(
            _link(
                "$successor_certificate@SuccessorCertificationScope",
                "$current_mandate@CurrentAssemblyMandateScope",
            ),
        ),
        certificate_set="$successor_certificate_set@CertificateSetScope",
        result_certificate="$successor_certificate@SuccessorCertificationScope",
        certified_result="$replacement@MemberReplacementScope",
    ),
    (34, "joint_presidential_selection"): _lineage(
        kind="collective-result",
        rationale="joint presidential ballot",
        interfaces=(
            _interface(
                "$assembly_presidential_ballot_interface",
                configurations="$selection_configuration@PresidentialSelectionConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$assembly_selection_submissions@AssemblySelectionSubmissionSetScope",
                outcomes="$selection_result@SelectionResultScope",
            ),
            _interface(
                "$council_presidential_ballot_interface",
                configurations="$selection_configuration@PresidentialSelectionConfigurationScope",
                rosters="$council_roster@CouncilRosterScope",
                submissions="$council_selection_submissions@CouncilSelectionSubmissionSetScope",
                outcomes="$council_selection_result@CouncilSelectionResultScope",
            ),
        ),
        certificate_set="$joint_selection_certificate_set@CertificateSetScope",
        result_certificate="$selection_certificate@ResultCertificateScope",
        certified_result="$selection_result@SelectionResultScope",
    ),
    (34, "cause_only_removal"): _single_collective_lineage(
        rationale="cause finding and cross-body presidential removal",
        interface_identity="$presidential_removal_interface",
        configurations="$cause_removal_configuration@CauseRemovalConfigurationScope",
        rosters="$confirming_institution_roster@ConfirmingInstitutionRosterScope",
        submissions="$confirmation_submissions@ConfirmationSubmissionSetScope",
        outcomes="$removal_result@RemovalResultScope",
        certificate_set="$removal_certificate_set@CertificateSetScope",
        result_certificate="$removal_certificate@ResultCertificateScope",
        certified_result="$removal_result@RemovalResultScope",
    ),
    (37, "ordinary_amendment"): _lineage(
        kind="collective-result",
        rationale="Assembly and referendum amendment conjunction",
        interfaces=(
            _interface(
                "$assembly_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$assembly_submissions@AssemblySubmissionSetScope",
                outcomes="$assembly_result@AssemblyResultScope",
            ),
            _interface(
                "$referendum_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$referendum_roster@ReferendumRosterScope",
                submissions="$referendum_submissions@ReferendumSubmissionSetScope",
                outcomes="$referendum_result@ReferendumResultScope",
            ),
        ),
        certificate_set="$amendment_certificate_set@CertificateSetScope",
        result_certificate="$amendment_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (37, "directly_regional_settlement_amendment"): _lineage(
        kind="collective-result",
        rationale="Assembly referendum and Council amendment conjunction",
        interfaces=(
            _interface(
                "$assembly_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$assembly_submissions@AssemblySubmissionSetScope",
                outcomes="$assembly_result@AssemblyResultScope",
            ),
            _interface(
                "$referendum_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$referendum_roster@ReferendumRosterScope",
                submissions="$referendum_submissions@ReferendumSubmissionSetScope",
                outcomes="$referendum_result@ReferendumResultScope",
            ),
            _interface(
                "$council_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$council_roster@CouncilRosterScope",
                submissions="$council_submissions@CouncilSubmissionSetScope",
                outcomes="$council_result@CouncilResultScope",
            ),
        ),
        certificate_set="$amendment_certificate_set@CertificateSetScope",
        result_certificate="$amendment_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (37, "competence_boundary_amendment"): _lineage(
        kind="collective-result",
        rationale="all affected amendment decision interfaces",
        interfaces=(
            _interface(
                "$assembly_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$assembly_submissions@AssemblySubmissionSetScope",
                outcomes="$assembly_result@AssemblyResultScope",
            ),
            _interface(
                "$referendum_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$referendum_roster@ReferendumRosterScope",
                submissions="$referendum_submissions@ReferendumSubmissionSetScope",
                outcomes="$referendum_result@ReferendumResultScope",
            ),
            _interface(
                "$council_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$council_roster@CouncilRosterScope",
                submissions="$council_submissions@CouncilSubmissionSetScope",
                outcomes="$council_result@CouncilResultScope",
            ),
            _interface(
                "$affected_region_amendment_interface",
                configurations="$amendment_configuration@AmendmentConfigurationScope",
                rosters="$affected_region_roster@AffectedRegionRosterScope,$affected_region_elector_rosters@AffectedRegionElectorRosterScope",
                submissions="$affected_region_submissions@AffectedRegionSubmissionSetScope",
                outcomes="$affected_region_results@AffectedRegionResultsScope",
            ),
        ),
        certificate_set="$amendment_certificate_set@CertificateSetScope",
        result_certificate="$amendment_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (38, "constitutional_initiative_docket"): _single_collective_lineage(
        rationale="threshold and distribution qualification",
        interface_identity="$constitutional_initiative_interface",
        configurations="$threshold_configuration@ConstitutionalAccessThresholdScope,$distribution_configuration@GeographicDistributionScope",
        rosters="$elector_roster@ElectorRosterScope",
        submissions="$signature_submissions@SignatureSubmissionSetScope",
        outcomes="$result@ResultScope",
        certificate_set="$qualification_certificate_set@CertificateSetScope",
        result_certificate="$qualification_certificate@QualificationCertificateScope",
        certified_result="$result@ResultScope",
    ),
    **{
        (39, key): _single_collective_lineage(
            rationale="ordinary initiative operative choice",
            interface_identity=interface_identity,
            configurations="$initiative_referendum_configuration@InitiativeReferendumConfigurationScope",
            rosters="$referendum_roster@ReferendumRosterScope",
            submissions="$referendum_submissions@ReferendumSubmissionSetScope",
            outcomes="$initiative_result@InitiativeResultScope,$counterproposal_result@CounterproposalResultScope",
            certificate_set="$operative_choice_certificate_set@CertificateSetScope",
            result_certificate="$unique_choice_certificate@UniqueChoiceCertificateScope",
            certified_result="$result@ResultScope",
        )
        for key, interface_identity in (
            ("initiative_only_wins", "$initiative_only_interface"),
            ("counterproposal_only_wins", "$counterproposal_only_interface"),
            ("both_pass_initiative_larger_share", "$initiative_larger_share_interface"),
            ("both_pass_counterproposal_larger_share", "$counterproposal_larger_share_interface"),
        )
    },
    (39, "tie_preserves_current_law"): _single_collective_lineage(
        rationale="certified initiative tie failure",
        interface_identity="$initiative_tie_interface",
        configurations="$initiative_referendum_configuration@InitiativeReferendumConfigurationScope",
        rosters="$referendum_roster@ReferendumRosterScope",
        submissions="$referendum_submissions@ReferendumSubmissionSetScope",
        outcomes="$result@ResultScope",
        certificate_set="$tie_failure_certificate_set@CertificateSetScope",
        result_certificate="$tie_failure_certificate@InitiativeFailureCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (39, "neither_passes_preserves_current_law"): _single_collective_lineage(
        rationale="certified neither-passes failure",
        interface_identity="$initiative_neither_interface",
        configurations="$initiative_referendum_configuration@InitiativeReferendumConfigurationScope",
        rosters="$referendum_roster@ReferendumRosterScope",
        submissions="$referendum_submissions@ReferendumSubmissionSetScope",
        outcomes="$result@ResultScope",
        certificate_set="$neither_failure_certificate_set@CertificateSetScope",
        result_certificate="$neither_failure_certificate@InitiativeFailureCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (40, "constructive_recall_success"): _single_collective_lineage(
        rationale="atomic recall and successor result",
        interface_identity="$constructive_recall_interface",
        configurations="$recall_configuration@RecallConfigurationScope",
        rosters="$elector_roster@ElectorRosterScope",
        submissions="$submissions@SubmissionSetScope",
        outcomes="$removal_result@RemovalResultScope,$successor_result@SuccessorResultScope",
        certificate_set="$recall_certificate_set@CertificateSetScope",
        result_certificate="$recall_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (40, "recall_failure_current_term_continuity"): _single_collective_lineage(
        rationale="certified recall failure",
        interface_identity="$recall_failure_interface",
        configurations="$recall_configuration@RecallConfigurationScope",
        rosters="$elector_roster@ElectorRosterScope",
        submissions="$recall_submissions@RecallSubmissionSetScope",
        outcomes="$result@ResultScope",
        certificate_set="$recall_failure_certificate_set@CertificateSetScope",
        result_certificate="$recall_failure_certificate@RecallFailureCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (41, "last_lawful_government_caretaker"): _lineage(
        kind="record-certificate-consumption",
        rationale="formation certificate yields failure then bounded caretaker",
        interfaces=(
            _interface(
                "$government_formation_interface",
                configurations="$formation_configuration@FormationConfigurationScope",
                rosters="$assembly_roster@AssemblyRosterScope",
                submissions="$formation_submissions@FormationSubmissionSetScope",
                outcomes="$formation_failure@CertifiedFormationFailureScope",
            ),
        ),
        upstream_links=(
            _link(
                "$formation_failure_certificate@FormationFailureCertificateScope",
                "$formation_failure@CertifiedFormationFailureScope",
            ),
            _link(
                "$formation_failure@CertifiedFormationFailureScope",
                "LastLawfulGovernmentCaretaker@ResultScope",
            ),
        ),
        certificate_set="$formation_failure_certificate_set@CertificateSetScope",
        result_certificate="$formation_failure_certificate@FormationFailureCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (44, "common_office_transfer"): _lineage(
        kind="record-certificate-consumption",
        rationale="successor certificate effects common-office transfer",
        interfaces=(),
        certificate_set="$successor_certificate_set@CertificateSetScope",
        result_certificate="$successor_certificate@SuccessorCertificateScope",
        certified_result="$effective_office_transfer@OfficeTransferRecordScope",
    ),
    (44, "regional_local_office_transfer"): _lineage(
        kind="record-certificate-consumption",
        rationale="successor certificate effects regional or local transfer",
        interfaces=(),
        certificate_set="$successor_certificate_set@CertificateSetScope",
        result_certificate="$successor_certificate@SuccessorCertificateScope",
        certified_result="$effective_office_transfer@OfficeTransferRecordScope",
    ),
    (45, "opening_referendum"): _single_collective_lineage(
        rationale="opening referendum authorizes negotiation only",
        interface_identity="$opening_referendum_interface",
        configurations="$secession_configuration@SecessionConfigurationScope",
        rosters="$opening_roster@OpeningReferendumRosterScope",
        submissions="$opening_submissions@UniqueOpeningSubmissionSetScope",
        outcomes="$opening_result@OpeningReferendumResultScope",
        certificate_set="$opening_certificate_set@CertificateSetScope",
        result_certificate="$opening_certificate@ResultCertificateScope",
        certified_result="$opening_result@OpeningReferendumResultScope",
    ),
    (45, "completed_negotiation"): _lineage(
        kind="record-certificate-consumption",
        rationale="negotiation consumes certified opening agreement rights and settlement records",
        interfaces=(),
        upstream_links=(
            _link("$opening_result_certificate@ResultCertificateScope", "$opening_result@OpeningReferendumResultScope"),
            _link("$federal_agreement_certificate@ResultCertificateScope", "$federal_agreement@FederalAgreementScope"),
            _link("$rights_review_certificate@ResultCertificateScope", "$rights_review@RightsAndMinorityReviewScope"),
            _link("$settlement_certificate@ResultCertificateScope", "$settlement@CompleteSecessionSettlementScope"),
        ),
        certificate_set="$negotiated_agreement_certificate_set@CertificateSetScope",
        result_certificate="$negotiated_agreement_certificate@ResultCertificateScope",
        certified_result="$result@ResultScope",
    ),
    (45, "final_exit_no_collective_impact"): _lineage(
        kind="collective-result",
        rationale="opening record chain and final affected-population ratification",
        interfaces=(
            _interface(
                "$final_ratification_interface",
                configurations="$final_ratification_configuration@FinalRatificationConfigurationScope",
                rosters="$final_roster@AffectedPopulationRosterScope",
                submissions="$final_submissions@UniqueFinalSubmissionSetScope",
                outcomes="$final_result@FinalRatificationResultScope",
            ),
        ),
        upstream_links=(
            _link("$opening_result_certificate@ResultCertificateScope", "$opening_result@OpeningReferendumResultScope"),
            _link("$federal_agreement_certificate@ResultCertificateScope", "$federal_agreement@FederalAgreementScope"),
            _link("$rights_review_certificate@ResultCertificateScope", "$rights_review@RightsAndMinorityReviewScope"),
            _link("$settlement_certificate@ResultCertificateScope", "$settlement@CompleteSecessionSettlementScope"),
        ),
        certificate_set="$final_exit_certificate_set@CertificateSetScope",
        result_certificate="$final_exit_certificate@ResultCertificateScope",
        certified_result="$final_result@FinalRatificationResultScope",
    ),
    (45, "final_exit_with_collective_consent"): _lineage(
        kind="collective-result",
        rationale="final ratification plus lawful affected-collective consent",
        interfaces=(
            _interface(
                "$final_ratification_interface",
                configurations="$final_ratification_configuration@FinalRatificationConfigurationScope",
                rosters="$final_roster@AffectedPopulationRosterScope",
                submissions="$final_submissions@UniqueFinalSubmissionSetScope",
                outcomes="$final_result@FinalRatificationResultScope",
            ),
            _interface(
                "$collective_consent_interface",
                configurations="$collective_consent_configuration@CollectiveConsentConfigurationScope",
                rosters="$affected_collective_roster@AffectedCollectiveRosterScope",
                submissions="$collective_consent_submissions@CollectiveConsentSubmissionSetScope",
                outcomes="$actual_collective_consent@ActualCollectiveConsentScope",
            ),
        ),
        upstream_links=(
            _link("$opening_result_certificate@ResultCertificateScope", "$opening_result@OpeningReferendumResultScope"),
            _link("$federal_agreement_certificate@ResultCertificateScope", "$federal_agreement@FederalAgreementScope"),
            _link("$rights_review_certificate@ResultCertificateScope", "$rights_review@RightsAndMinorityReviewScope"),
            _link("$settlement_certificate@ResultCertificateScope", "$settlement@CompleteSecessionSettlementScope"),
        ),
        certificate_set="$final_exit_certificate_set@CertificateSetScope",
        result_certificate="$final_exit_certificate@ResultCertificateScope",
        certified_result="$final_result@FinalRatificationResultScope",
    ),
}

FROZEN_FS036_KEYS = (
    "ordinary_resident_membership",
    "accessible_nonconventional_residence",
    "multiple_residences_first_choice",
    "multiple_residences_second_choice",
    "compelled_placement_nonchange",
    "last_uncontested_home_during_dispute",
    "provisional_first_home",
    "atomic_home_transfer",
    "adult_resident_franchise",
    "adult_resident_candidacy",
    "unique_accepted_submission",
    "established_adulthood_continuity",
    "provisional_adulthood_expiring_opportunity",
    "positive_nonresident_disposition",
    "former_resident_return_without_ballot",
    "office_move_continuity",
)
FROZEN_HOLDER_MAP = {
    (32, "finite_delegation_tenure"): ("FSBOD_03",),
    (32, "instruction_scope"): ("FSBOD_21",),
    (32, "proportional_replacement"): ("FSBOD_21",),
    (32, "delegation_vacancy_fill"): ("FSBOD_21",),
    (33, "executive_composition"): ("FSBOD_02",),
    (33, "executive_member_replacement"): ("FSBOD_04",),
    (33, "coordinator_incapacity"): ("FSBOD_04",),
    (33, "coordinator_power_boundary"): ("FSBOD_04",),
    (44, "common_office_transfer"): ("FSBOD_05",),
    (44, "regional_local_office_transfer"): ("FSBOD_21",),
    (45, "opening_referendum"): ("FSBOD_01",),
    (45, "completed_negotiation"): ("FSBOD_02", "FSBOD_03"),
    (45, "final_exit_no_collective_impact"): ("FSBOD_01",),
    (45, "final_exit_with_collective_consent"): ("FSBOD_01", "FSBOD_21"),
}
EXPECTED_BRANCH_IR_SHA256 = "3624348425931f9acef19a77a1bb7c840f321d9b892d68fcf6f756c26b1b1522"

FROZEN_DECLARED_ROLE_COUNTS = {
    "roster": 30,
    "submission": 6,
    "outcome": 61,
}
FROZEN_DECLARED_ROLE_SHA256 = (
    "d87260a7fde70b35842c39edd825d6ced74b1346fbf51e60f11a9f3322800b66"
)
ADDITIONAL_DECLARED_OUTCOME_SCOPES = frozenset(
    {
        "ActualCollectiveConsentScope",
        "CertifiedFormationFailureScope",
        "CertifiedGovernmentScope",
        "CompleteSecessionSettlementScope",
        "CurrentAssemblyMandateScope",
        "CurrentMandateScope",
        "CurrentPredecessorMandateScope",
        "ExecutiveCompositionScope",
        "FederalAgreementScope",
        "MemberReplacementScope",
        "OfficeTransferRecordScope",
        "RightsAndMinorityReviewScope",
    }
)
DECLARED_ROLE_EXCEPTIONS: dict[tuple[int, str, str, Field], str] = {
    (
        41,
        "last_lawful_government_caretaker",
        "outcome",
        Field("$current_mandate", "CurrentMandateScope"),
    ): (
        "positive current-mandate premise carried by the exact certified result "
        "envelope, not a government-formation outcome"
    ),
    (
        44,
        "common_office_transfer",
        "outcome",
        Field("$predecessor_mandate", "CurrentPredecessorMandateScope"),
    ): (
        "positive current-predecessor premise, not the certified successor "
        "transfer outcome"
    ),
    (
        44,
        "regional_local_office_transfer",
        "outcome",
        Field("$predecessor_mandate", "CurrentPredecessorMandateScope"),
    ): (
        "positive current-predecessor premise, not the certified successor "
        "transfer outcome"
    ),
}


def _field_ir(field: Field) -> list[str]:
    return [field.value, field.scope]


def decision_lineage_ir(lineage: DecisionLineage) -> dict[str, object]:
    return {
        "kind": lineage.kind,
        "rationale": lineage.rationale,
        "interfaces": [
            {
                "identity": _field_ir(interface.identity),
                "configurations": [
                    _field_ir(field) for field in interface.configurations
                ],
                "rosters": [_field_ir(field) for field in interface.rosters],
                "submissions": [
                    _field_ir(field) for field in interface.submissions
                ],
                "outcomes": [_field_ir(field) for field in interface.outcomes],
            }
            for interface in lineage.interfaces
        ],
        "upstream_links": [
            {
                "certificate": _field_ir(link.certificate),
                "result": _field_ir(link.result),
            }
            for link in lineage.upstream_links
        ],
        "certificate_set": _field_ir(lineage.certificate_set),
        "result_certificate": _field_ir(lineage.result_certificate),
        "certified_result": _field_ir(lineage.certified_result),
    }


def branch_ir_payload(branch: RuleBranch) -> dict[str, object]:
    identity = (branch.card.number, branch.key)
    return {
        "card": branch.card.number,
        "key": branch.key,
        "fields": [[field.value, field.scope] for field in branch.fields],
        "dynamic": branch.dynamic,
        "dynamic_subtype": branch.dynamic_subtype,
        "authority_holders": list(branch.authority_holders),
        "authorizations": [
            [item.actor, item.authority] for item in branch.authorizations
        ],
        "observations": [
            [item.actor, item.subject, item.value, item.scope]
            for item in branch.observations
        ],
        "marker": branch.marker,
        "jurisdiction_kind": branch.jurisdiction_kind,
        "legal_scope_kind": branch.legal_scope_kind,
        "decision_lineage": (
            decision_lineage_ir(DECISION_LINEAGES[identity])
            if branch.dynamic else None
        ),
    }


def branch_ir_sha256(branches: Sequence[RuleBranch]) -> str:
    payload = [branch_ir_payload(branch) for branch in branches]
    encoded = json.dumps(
        payload,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _branch_lookup(number: int, key: str) -> RuleBranch:
    matches = [
        branch
        for branch in ALL_BRANCHES
        if branch.card.number == number and branch.key == key
    ]
    if len(matches) != 1:
        raise RuntimeError(
            f"expected one FSPOW_{number:03d}/{key} branch, found {len(matches)}"
        )
    return matches[0]


def lineage_fields(lineage: DecisionLineage) -> tuple[Field, ...]:
    fields: list[Field] = []
    for interface in lineage.interfaces:
        fields.extend(interface.owned_terms)
    for link in lineage.upstream_links:
        fields.extend((link.certificate, link.result))
    fields.extend(
        (
            lineage.certificate_set,
            lineage.result_certificate,
            lineage.certified_result,
        )
    )
    return tuple(dict.fromkeys(fields))


def _declared_branch_role(field: Field) -> str | None:
    if not field.value.startswith("$"):
        return None
    if "RosterScope" in field.scope:
        return "roster"
    if "SubmissionSetScope" in field.scope:
        return "submission"
    if (
        "Result" in field.scope and "Certificate" not in field.scope
    ) or field.scope in ADDITIONAL_DECLARED_OUTCOME_SCOPES:
        return "outcome"
    return None


def declared_branch_role_rows() -> tuple[tuple[int, str, str, str, str], ...]:
    rows: list[tuple[int, str, str, str, str]] = []
    for branch in ALL_BRANCHES:
        if not branch.dynamic:
            continue
        for field in branch.fields:
            role = _declared_branch_role(field)
            if role is not None:
                rows.append(
                    (
                        branch.card.number,
                        branch.key,
                        role,
                        field.value,
                        field.scope,
                    )
                )
    return tuple(rows)


def validate_declared_role_ownership() -> None:
    rows = declared_branch_role_rows()
    counts = {
        role: sum(row[2] == role for row in rows)
        for role in ("roster", "submission", "outcome")
    }
    if counts != FROZEN_DECLARED_ROLE_COUNTS:
        raise RuntimeError(f"branch-declared role census changed: {counts!r}")
    encoded = (
        json.dumps(rows, ensure_ascii=True, separators=(",", ":")) + "\n"
    ).encode("utf-8")
    actual_sha256 = hashlib.sha256(encoded).hexdigest()
    if actual_sha256 != FROZEN_DECLARED_ROLE_SHA256:
        raise RuntimeError(
            "branch-declared role surface changed: "
            f"expected {FROZEN_DECLARED_ROLE_SHA256}, found {actual_sha256}"
        )

    used_exceptions: set[tuple[int, str, str, Field]] = set()
    category_names = {
        "roster": "rosters",
        "submission": "submissions",
        "outcome": "outcomes",
    }
    for number, key, role, value, scope in rows:
        branch = _branch_lookup(number, key)
        lineage = DECISION_LINEAGES[(number, key)]
        field = Field(value, scope)
        owners = [
            f"interface:{interface.identity.value}"
            for interface in lineage.interfaces
            if field in getattr(interface, category_names[role])
        ]
        # Certificate and upstream links bind an interface-owned outcome; they
        # become its primary owner only when no decision interface owns it.
        if role == "outcome" and not owners and field == lineage.certified_result:
            owners.append("certified-result")
        if role == "outcome" and not owners:
            owners.extend(
                f"upstream:{link.certificate.value}"
                for link in lineage.upstream_links
                if field == link.result
            )

        exception = (number, key, role, field)
        if not owners and exception in DECLARED_ROLE_EXCEPTIONS:
            if not DECLARED_ROLE_EXCEPTIONS[exception]:
                raise RuntimeError(f"empty declared-role exception: {exception!r}")
            used_exceptions.add(exception)
            continue
        if len(owners) != 1:
            raise RuntimeError(
                f"FSPOW_{number:03d}/{key} must assign declared {role} "
                f"{field.value}@{field.scope} to one primary owner; "
                f"found {owners!r}"
            )
    if used_exceptions != set(DECLARED_ROLE_EXCEPTIONS):
        raise RuntimeError(
            "declared-role exceptions changed or became unnecessary: "
            f"used={sorted(map(repr, used_exceptions))!r}"
        )


def validate_decision_lineage_manifest() -> None:
    dynamic_identities = {
        (branch.card.number, branch.key)
        for branch in ALL_BRANCHES
        if branch.dynamic
    }
    if set(DECISION_LINEAGES) != dynamic_identities:
        raise RuntimeError(
            "decision-lineage manifest keys differ from the 57 dynamic identities"
        )
    validate_declared_role_ownership()
    kind_counts = {
        kind: sum(lineage.kind == kind for lineage in DECISION_LINEAGES.values())
        for kind in ("collective-result", "record-certificate-consumption")
    }
    if kind_counts != {
        "collective-result": 50,
        "record-certificate-consumption": 7,
    }:
        raise RuntimeError(f"decision-lineage kind census changed: {kind_counts!r}")
    record_identities = {
        identity
        for identity, lineage in DECISION_LINEAGES.items()
        if lineage.kind == "record-certificate-consumption"
    }
    if record_identities != {
        (18, "formal_government_appointment"),
        (33, "executive_composition"),
        (33, "executive_member_replacement"),
        (41, "last_lawful_government_caretaker"),
        (44, "common_office_transfer"),
        (44, "regional_local_office_transfer"),
        (45, "completed_negotiation"),
    }:
        raise RuntimeError("record/certificate-consumption classification changed")

    zero_voter_identities: set[tuple[int, str]] = set()
    for identity, lineage in DECISION_LINEAGES.items():
        if not lineage.rationale:
            raise RuntimeError(f"{identity!r} lacks a lineage rationale")
        interface_ids = [interface.identity for interface in lineage.interfaces]
        if len(set(interface_ids)) != len(interface_ids):
            raise RuntimeError(f"{identity!r} repeats an interface identity")
        roster_count = sum(len(interface.rosters) for interface in lineage.interfaces)
        submission_count = sum(
            len(interface.submissions) for interface in lineage.interfaces
        )
        if roster_count == 0 or submission_count == 0:
            zero_voter_identities.add(identity)
        for interface in lineage.interfaces:
            if (
                not interface.configurations
                or not interface.rosters
                or not interface.submissions
                or not interface.outcomes
            ):
                raise RuntimeError(f"{identity!r} has an incomplete decision interface")
            for category in (
                interface.configurations,
                interface.rosters,
                interface.submissions,
                interface.outcomes,
            ):
                if len(set(category)) != len(category):
                    raise RuntimeError(
                        f"{identity!r}/{interface.identity.value} repeats a term"
                    )
        for category_name in ("rosters", "submissions", "outcomes"):
            owned = [
                field
                for interface in lineage.interfaces
                for field in getattr(interface, category_name)
            ]
            if len(set(owned)) != len(owned):
                raise RuntimeError(
                    f"{identity!r} assigns one {category_name} term twice"
                )
        encoded = json.dumps(
            decision_lineage_ir(lineage),
            sort_keys=True,
            separators=(",", ":"),
        )
        if "$evidence_set" in encoded or "$decision_configuration" in encoded:
            raise RuntimeError(f"{identity!r} acquired a generic lineage fallback")
        if "$submission_set" in encoded and identity != (10, "assembly_election"):
            raise RuntimeError(f"{identity!r} acquired an unowned generic submission")
    if zero_voter_identities != {
        (44, "common_office_transfer"),
        (44, "regional_local_office_transfer"),
        (45, "completed_negotiation"),
    }:
        raise RuntimeError(
            f"empty-voter lineage set changed: {sorted(zero_voter_identities)!r}"
        )
    certified_government = Field(
        "$certified_government", "CertifiedGovernmentScope"
    )
    for identity in (
        (15, "confidence_certification"),
        (18, "formal_government_appointment"),
    ):
        lineage = DECISION_LINEAGES[identity]
        owners = [
            interface.identity.value
            for interface in lineage.interfaces
            if certified_government in interface.outcomes
        ]
        if len(owners) != 1:
            raise RuntimeError(
                f"{identity!r} must bind certified government to one exact "
                f"decision interface; found {owners!r}"
            )



    operative_keys = {
        "initiative_only_wins",
        "counterproposal_only_wins",
        "both_pass_initiative_larger_share",
        "both_pass_counterproposal_larger_share",
    }
    for branch in (item for item in ALL_BRANCHES if item.card.number == 39):
        has_positive_review = any(
            field.value == "PositiveCompatibilityAndCorridorReviewPassed"
            and field.scope == "CompatibilityReviewDispositionScope"
            for field in branch.fields
        )
        if has_positive_review != (branch.key in operative_keys):
            raise RuntimeError(
                f"{branch.marker} compatibility/corridor polarity changed"
            )


def validate_branch_inventory() -> None:
    validate_decision_lineage_manifest()
    if len(CARDS) != EXPECTED_CARD_COUNT:
        raise RuntimeError(f"expected {EXPECTED_CARD_COUNT} cards, found {len(CARDS)}")
    if [card.number for card in CARDS] != list(range(1, EXPECTED_CARD_COUNT + 1)):
        raise RuntimeError("state-form cards are not the exact ordered 001..051 set")
    if (
        len(JURISDICTION_LABELS) != EXPECTED_CARD_COUNT + 1
        or JURISDICTION_LABELS[0] != ""
        or len(set(JURISDICTION_LABELS[1:])) != EXPECTED_CARD_COUNT
    ):
        raise RuntimeError("jurisdiction labels are not one unique label per card")

    result_count = len(ALL_BRANCHES)
    authority_count = sum(
        len(branch.authority_holders) for branch in ALL_BRANCHES
    )
    if result_count != EXPECTED_RESULT_COUNT:
        raise RuntimeError(
            f"expected {EXPECTED_RESULT_COUNT} result branches, found {result_count}"
        )
    if authority_count != EXPECTED_AUTHORITY_COUNT:
        raise RuntimeError(
            f"expected {EXPECTED_AUTHORITY_COUNT} authority heads, found {authority_count}"
        )
    actual_band_counts = tuple(
        (
            low,
            high,
            sum(low <= branch.card.number <= high for branch in ALL_BRANCHES),
            sum(
                len(branch.authority_holders)
                for branch in ALL_BRANCHES
                if low <= branch.card.number <= high
            ),
        )
        for low, high, _, _ in EXPECTED_BAND_COUNTS
    )
    if actual_band_counts != EXPECTED_BAND_COUNTS:
        raise RuntimeError(
            f"state-form band census changed: {actual_band_counts!r}"
        )

    identities = [(branch.card.number, branch.key) for branch in ALL_BRANCHES]
    if identities != sorted(identities, key=lambda item: item[0]):
        raise RuntimeError("state-form branch cards are not in nondecreasing order")
    if len(set(identities)) != len(identities):
        raise RuntimeError("state-form branch identities are not unique")
    if {number for number, _ in identities} != set(range(1, 52)):
        raise RuntimeError("one or more state-form powers has no branch")
    markers = [branch.marker for branch in ALL_BRANCHES]
    scope_kinds = [branch.legal_scope_kind for branch in ALL_BRANCHES]
    if len(set(markers)) != len(markers):
        raise RuntimeError("state-form branch markers are not unique")
    if len(set(scope_kinds)) != len(scope_kinds):
        raise RuntimeError("state-form authority-scope kinds are not unique")

    for branch in ALL_BRANCHES:
        if not branch.authority_holders:
            raise RuntimeError(f"{branch.marker} has no direct-effect holder")
        if any(not holder.startswith("FSBOD_") for holder in branch.authority_holders):
            raise RuntimeError(f"{branch.marker} has a non-body authority holder")
        expected_subtype = (
            branch.dynamic_subtype if branch.dynamic else "static"
        )
        if expected_subtype not in {"static", "collective", "certificate"}:
            raise RuntimeError(f"{branch.marker} has an unsupported dynamic subtype")
        if branch.dynamic != (branch.dynamic_subtype != "static"):
            raise RuntimeError(f"{branch.marker} has inconsistent dynamic metadata")
        field_pairs = [(field.value, field.scope) for field in branch.fields]
        if len(set(field_pairs)) != len(field_pairs):
            raise RuntimeError(f"{branch.marker} repeats an exact field binding")
        if any(
            "Parameter_" in field.value
            or field.scope in {"SourceTransitionScope", "CertificateTransitionScope"}
            for field in branch.fields
        ):
            raise RuntimeError(f"{branch.marker} contains an opaque or source-transition field")

    for holder in SELECTOR_BODIES:
        selection = _branch_lookup(
            28,
            f"{SELECTOR_KEY_BY_BODY[holder]}_appointment_selection",
        )
        selection_pairs = {
            (field.value, field.scope) for field in selection.fields
        }
        required_configuration = {
            ("$selector_configuration", "SelectorConfigurationScope"),
            ("$qualification_authority", "QualificationAuthorityScope"),
            ("$fallback_configuration", "FallbackConfigurationScope"),
            (holder, "SelectedHolderScope"),
        }
        if not required_configuration <= selection_pairs:
            raise RuntimeError(
                f"{selection.marker} lost its source-bound selector interface"
            )


    fs036 = tuple(
        branch.key for branch in ALL_BRANCHES if branch.card.number == 36
    )
    if fs036 != FROZEN_FS036_KEYS:
        raise RuntimeError(f"FSPOW_036 branch set changed: {fs036!r}")
    choice_keys = {
        "multiple_residences_first_choice",
        "multiple_residences_second_choice",
    }
    for branch in (item for item in ALL_BRANCHES if item.card.number == 36):
        if branch.key in choice_keys:
            if branch.authorizations != MULTI_HOME_AUTHORIZATION:
                raise RuntimeError(f"{branch.marker} lost claimant choice authority")
            if len(branch.observations) != 1 or branch.observations[0].scope != "ChoiceScope":
                raise RuntimeError(f"{branch.marker} lost its exact claimant choice")
        elif branch.authorizations or branch.observations:
            raise RuntimeError(f"{branch.marker} gained an unreviewed special witness")
    home_membership_values = {
        field.value for field in _branch_lookup(36, "ordinary_resident_membership").fields
    }
    if any("Adult" in value for value in home_membership_values):
        raise RuntimeError("political home or membership was gated on adulthood")
    atomic_scopes = {
        field.scope for field in _branch_lookup(36, "atomic_home_transfer").fields
    }
    if {"PriorSubmissionScope", "EffectiveSubmissionScope"} & atomic_scopes:
        raise RuntimeError("atomic political-home transfer improperly reads a submission")

    for identity, holders in FROZEN_HOLDER_MAP.items():
        if _branch_lookup(*identity).authority_holders != holders:
            raise RuntimeError(f"{identity!r} holder mapping changed")
    fs045_keys = tuple(
        branch.key for branch in ALL_BRANCHES if branch.card.number == 45
    )
    if fs045_keys != (
        "opening_referendum",
        "completed_negotiation",
        "final_exit_no_collective_impact",
        "final_exit_with_collective_consent",
    ):
        raise RuntimeError(f"FSPOW_045 stage set changed: {fs045_keys!r}")
    required_fs045_values = {
        "opening_referendum": {
            "CompleteUniqueOpeningSubmissions",
            "OpeningAffirmativeExceedsNegative",
            "OpeningReferendumIsNotExit",
        },
        "completed_negotiation": {
            "OpeningAffirmativeExceedsNegative",
            "PositiveFederalAgreementComplete",
            "PositiveRightsAndMinorityReviewPassed",
            "PositiveSettlementComplete",
        },
        "final_exit_no_collective_impact": {
            "OpeningAffirmativeExceedsNegative",
            "PositiveFederalAgreementComplete",
            "PositiveRightsAndMinorityReviewPassed",
            "PositiveSettlementComplete",
            "PositiveFinalAffectedPopulationRatificationPassed",
            "PositiveNoCollectiveTitleOrSovereigntyImpact",
        },
        "final_exit_with_collective_consent": {
            "OpeningAffirmativeExceedsNegative",
            "PositiveFederalAgreementComplete",
            "PositiveRightsAndMinorityReviewPassed",
            "PositiveSettlementComplete",
            "PositiveCollectiveTitleOrSovereigntyImpact",
            "PositiveActualCollectiveConsent",
            "PositiveFinalAffectedPopulationRatificationPassed",
        },
    }
    for key, required in required_fs045_values.items():
        actual = {field.value for field in _branch_lookup(45, key).fields}
        missing = required - actual
        if missing:
            raise RuntimeError(f"FSPOW_045/{key} lacks {sorted(missing)!r}")
    if _branch_lookup(44, "common_office_transfer").dynamic_subtype != "certificate":
        raise RuntimeError("common succession must use the certificate pipeline")
    if _branch_lookup(44, "regional_local_office_transfer").dynamic_subtype != "certificate":
        raise RuntimeError("regional/local succession must use the certificate pipeline")
    if any(
        branch.dynamic_subtype == "certificate" and branch.card.number != 44
        for branch in ALL_BRANCHES
    ):
        raise RuntimeError("certificate subtype escaped the FSPOW_044 boundary")

    actual_ir_sha256 = branch_ir_sha256(ALL_BRANCHES)
    if EXPECTED_BRANCH_IR_SHA256 and actual_ir_sha256 != EXPECTED_BRANCH_IR_SHA256:
        raise RuntimeError(
            "state-form branch IR changed: "
            f"expected {EXPECTED_BRANCH_IR_SHA256}, found {actual_ir_sha256}"
        )

CURRENT_REJOIN_NAMES = (
    "record",
    "source",
    "temporal",
    "temporal_review",
    "record_review",
    "version",
    "epoch",
    "temporal_record",
    "jurisdiction",
    "legal_scope",
    "reconciliation",
)


def current_rejoin_premises(
    power: str,
    jurisdiction_kind: str,
    legal_scope_kind: str,
) -> list[str]:
    return [
        "authorized($source, StateFormSourceAuthority, $record)",
        "authorized($temporal, StateFormTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, StateFormTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, StateFormRecordReviewAuthority, $record)",
        "observe($source, $record, Constitution_StateForm, SourceFamilyScope)",
        "observe($record_review, $record, Constitution_StateForm, SourceFamilyScope)",
        "observe($temporal, $temporal_record, Constitution_StateForm, SourceFamilyScope)",
        "observe($temporal_review, $temporal_record, Constitution_StateForm, SourceFamilyScope)",
        "observe($source, $record, $version, SourceVersionScope)",
        "observe($record_review, $record, $version, SourceVersionScope)",
        "observe($temporal, $temporal_record, $version, SourceVersionScope)",
        "observe($temporal_review, $temporal_record, $version, SourceVersionScope)",
        "observe($source, $record, $temporal_record, TemporalRecordScope)",
        "observe($record_review, $record, $temporal_record, TemporalRecordScope)",
        "observe($temporal, $temporal_record, $record, StateFormRecordScope)",
        "observe($temporal_review, $temporal_record, $record, StateFormRecordScope)",
        f"observe($source, $record, {power}, PowerScope)",
        f"observe($record_review, $record, {power}, PowerScope)",
        f"observe($temporal, $temporal_record, {power}, PowerScope)",
        f"observe($temporal_review, $temporal_record, {power}, PowerScope)",
        "observe($source, $record, $jurisdiction, JurisdictionScope)",
        "observe($record_review, $record, $jurisdiction, JurisdictionScope)",
        "observe($temporal, $temporal_record, $jurisdiction, JurisdictionScope)",
        "observe($temporal_review, $temporal_record, $jurisdiction, JurisdictionScope)",
        f"observe($source, $record, {jurisdiction_kind}, JurisdictionKindScope)",
        f"observe($record_review, $record, {jurisdiction_kind}, JurisdictionKindScope)",
        f"observe($temporal, $temporal_record, {jurisdiction_kind}, JurisdictionKindScope)",
        f"observe($temporal_review, $temporal_record, {jurisdiction_kind}, JurisdictionKindScope)",
        "observe($source, $record, $legal_scope, AuthorityScope)",
        "observe($record_review, $record, $legal_scope, AuthorityScope)",
        "observe($temporal, $temporal_record, $legal_scope, AuthorityScope)",
        "observe($temporal_review, $temporal_record, $legal_scope, AuthorityScope)",
        f"observe($source, $record, {legal_scope_kind}, AuthorityScopeKindScope)",
        f"observe($record_review, $record, {legal_scope_kind}, AuthorityScopeKindScope)",
        f"observe($temporal, $temporal_record, {legal_scope_kind}, AuthorityScopeKindScope)",
        f"observe($temporal_review, $temporal_record, {legal_scope_kind}, AuthorityScopeKindScope)",
        "observe($source, $record, $epoch, SourceEpochScope)",
        "observe($record_review, $record, $epoch, SourceEpochScope)",
        "observe($temporal, $temporal_record, $epoch, SourceEpochScope)",
        "observe($temporal_review, $temporal_record, $epoch, SourceEpochScope)",
        "observe($source, $record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($record_review, $record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($temporal, $temporal_record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($temporal_review, $temporal_record, StateFormCurrentSelection, EffectiveSelectionScope)",
        "observe($source, $record, $reconciliation, ReconciliationRecordScope)",
        "observe($record_review, $record, $reconciliation, ReconciliationRecordScope)",
        "observe($temporal, $temporal_record, $reconciliation, ReconciliationRecordScope)",
        "observe($temporal_review, $temporal_record, $reconciliation, ReconciliationRecordScope)",
        "observe($source, $reconciliation, StateFormRecordReconciled, ReconciliationStatusScope)",
        "observe($record_review, $reconciliation, StateFormRecordReconciled, ReconciliationStatusScope)",
        "observe($source, $reconciliation, $record, StateFormRecordScope)",
        "observe($record_review, $reconciliation, $record, StateFormRecordScope)",
        "observe($source, $reconciliation, $version, SourceVersionScope)",
        "observe($record_review, $reconciliation, $version, SourceVersionScope)",
        f"observe($source, $reconciliation, {power}, PowerScope)",
        f"observe($record_review, $reconciliation, {power}, PowerScope)",
        "observe($source, $reconciliation, $jurisdiction, JurisdictionScope)",
        "observe($record_review, $reconciliation, $jurisdiction, JurisdictionScope)",
        f"observe($source, $reconciliation, {jurisdiction_kind}, JurisdictionKindScope)",
        f"observe($record_review, $reconciliation, {jurisdiction_kind}, JurisdictionKindScope)",
        "observe($source, $reconciliation, $legal_scope, AuthorityScope)",
        "observe($record_review, $reconciliation, $legal_scope, AuthorityScope)",
        f"observe($source, $reconciliation, {legal_scope_kind}, AuthorityScopeKindScope)",
        f"observe($record_review, $reconciliation, {legal_scope_kind}, AuthorityScopeKindScope)",
    ]


def _observed_fields(
    actors: Iterable[str],
    subject: str,
    fields: Iterable[Field],
) -> list[str]:
    premises: list[str] = []
    for field in fields:
        premises.extend(observed(actors, subject, field.value, field.scope))
    return premises


def decision_lineage_premises(branch: RuleBranch) -> list[str]:
    lineage = DECISION_LINEAGES[(branch.card.number, branch.key)]
    dynamic_actors = ("admin", "assurer", "service")
    all_actors = ("source", "evidence", *dynamic_actors, "review")
    outcome_actors = ("source", "evidence", "service", "review")
    certificate_actors = ("source", "evidence", "assurer", "service", "review")
    body = [
        "authorized($admin, DecisionAdministrationAuthority, $record)",
        "authorized($assurer, IndependentCompletenessAssuranceAuthority, $record)",
        "authorized($service, ResultServiceAuthority, $record)",
        *observed(dynamic_actors, "$record", "Constitution_StateForm", "SourceFamilyScope"),
        *observed(dynamic_actors, "$record", "$version", "SourceVersionScope"),
        *observed(dynamic_actors, "$record", "$epoch", "SourceEpochScope"),
        *observed(dynamic_actors, "$record", "$temporal_record", "TemporalRecordScope"),
        *observed(dynamic_actors, "$record", branch.card.power, "PowerScope"),
        *observed(dynamic_actors, "$record", "$jurisdiction", "JurisdictionScope"),
        *observed(dynamic_actors, "$record", branch.jurisdiction_kind, "JurisdictionKindScope"),
        *observed(dynamic_actors, "$record", "$legal_scope", "AuthorityScope"),
        *observed(dynamic_actors, "$record", branch.legal_scope_kind, "AuthorityScopeKindScope"),
        *observed(dynamic_actors, "$record", "$reconciliation", "ReconciliationRecordScope"),
        *observed(dynamic_actors, "$record", "$result", "ResultScope"),
        *observed(dynamic_actors, "$result", "$result_reconciliation", "ReconciliationRecordScope"),
    ]
    for interface in lineage.interfaces:
        body.extend(
            [
                *_observed_fields(all_actors, "$result", (interface.identity,)),
                *_observed_fields(
                    all_actors,
                    "$result",
                    (
                        *interface.configurations,
                        *interface.rosters,
                        *interface.submissions,
                    ),
                ),
                *_observed_fields(
                    outcome_actors,
                    "$result",
                    interface.outcomes,
                ),
            ]
        )
        for roster in interface.rosters:
            body.extend(
                observed(
                    ("assurer", "review"),
                    roster.value,
                    "CompleteAndNonzeroEligibleRoster",
                    "RosterCompletenessDispositionScope",
                )
            )
        for submission in interface.submissions:
            body.extend(
                observed(
                    ("assurer", "review"),
                    submission.value,
                    "CompleteUniqueSubmissionSet",
                    "SubmissionCompletenessDispositionScope",
                )
            )
        interface_terms = (
            *interface.configurations,
            *interface.rosters,
            *interface.submissions,
            *interface.outcomes,
        )
        body.extend(
            _observed_fields(
                ("service", "review"),
                interface.identity.value,
                interface_terms,
            )
        )
        body.extend(
            observed(
                ("service", "review"),
                lineage.result_certificate.value,
                interface.identity.value,
                "DecisionInterfaceScope",
            )
        )
        body.extend(
            _observed_fields(
                ("service", "review"),
                lineage.result_certificate.value,
                interface_terms,
            )
        )

    body.extend(
        [
            *_observed_fields(all_actors, "$result", (lineage.certificate_set,)),
            *observed(
                ("assurer", "review"),
                lineage.certificate_set.value,
                "CompleteUniqueCertificateSet",
                "CertificateCompletenessDispositionScope",
            ),
            *_observed_fields(
                certificate_actors,
                "$result",
                (lineage.result_certificate,),
            ),
            *_observed_fields(
                outcome_actors,
                "$result",
                (lineage.certified_result,),
            ),
            *observed(
                ("service", "review"),
                lineage.certificate_set.value,
                lineage.result_certificate.value,
                "ResultCertificateScope",
            ),
            *observed(
                ("service", "review"),
                lineage.result_certificate.value,
                lineage.certified_result.value,
                "ResultScope",
            ),
        ]
    )
    for link in lineage.upstream_links:
        body.extend(
            [
                *_observed_fields(
                    certificate_actors,
                    "$result",
                    (link.certificate,),
                ),
                *_observed_fields(outcome_actors, "$result", (link.result,)),
                *observed(
                    ("service", "review"),
                    link.certificate.value,
                    link.result.value,
                    "ResultScope",
                ),
                *_observed_fields(
                    ("service", "review"),
                    lineage.result_certificate.value,
                    (link.certificate, link.result),
                ),
            ]
        )
    body.extend(
        [
            *observed(("service", "review"), "$result", "UniqueCertifiedResult", "ResultDispositionScope"),
            *distinct(dynamic_actors),
            "~($review = $admin)",
            "~($review = $assurer)",
            "~($review = $service)",
        ]
    )
    return list(dict.fromkeys(body))


def branch_variable_names(
    branch: RuleBranch,
    *,
    authority_stage: bool,
) -> list[str]:
    names = [
        *CURRENT_REJOIN_NAMES,
        "result",
        "evidence",
        "review",
        "challenge_record",
        "correction_record",
        "remedy_record",
        "end",
        "result_reconciliation",
    ]
    if authority_stage:
        names.append("executor")
    if branch.dynamic:
        names.extend(
            (
                "admin",
                "assurer",
                "service",
            )
        )
        names.extend(
            variable_names(
                lineage_fields(DECISION_LINEAGES[(branch.card.number, branch.key)])
            )
        )
    names.extend(variable_names(branch.fields))
    for authorization in branch.authorizations:
        names.extend(
            re.findall(
                r"\$([A-Za-z_][A-Za-z0-9_]*)",
                f"{authorization.actor} {authorization.authority}",
            )
        )
    for observation in branch.observations:
        names.extend(
            re.findall(
                r"\$([A-Za-z_][A-Za-z0-9_]*)",
                " ".join(
                    (
                        observation.actor,
                        observation.subject,
                        observation.value,
                        observation.scope,
                    )
                ),
            )
        )
    return list(dict.fromkeys(names))


def result_raw_premises(branch: RuleBranch) -> list[str]:
    card = branch.card
    result_actors = ("source", "evidence", "review")
    body = [
        "complete($record, StateFormCurrent, $temporal_record)",
        *current_rejoin_premises(
            card.power,
            branch.jurisdiction_kind,
            branch.legal_scope_kind,
        ),
        "authorized($evidence, StateFormEvidenceAuthority, $record)",
        "authorized($review, IndependentStateFormReviewAuthority, $record)",
        *observed(("evidence", "review"), "$record", "Constitution_StateForm", "SourceFamilyScope"),
        *observed(("evidence", "review"), "$record", "$version", "SourceVersionScope"),
        *observed(("evidence", "review"), "$record", "$epoch", "SourceEpochScope"),
        *observed(("evidence", "review"), "$record", "$temporal_record", "TemporalRecordScope"),
        *observed(("evidence", "review"), "$record", card.power, "PowerScope"),
        *observed(("evidence", "review"), "$record", "$jurisdiction", "JurisdictionScope"),
        *observed(("evidence", "review"), "$record", branch.jurisdiction_kind, "JurisdictionKindScope"),
        *observed(("evidence", "review"), "$record", "$legal_scope", "AuthorityScope"),
        *observed(("evidence", "review"), "$record", branch.legal_scope_kind, "AuthorityScopeKindScope"),
        *observed(("evidence", "review"), "$record", "$reconciliation", "ReconciliationRecordScope"),
        *observed(result_actors, "$record", "$result", "ResultScope"),
        *observed(result_actors, "$result", branch.marker, "StateFormBranchScope"),
        *observed(result_actors, "$result", "$challenge_record", "ChallengeScope"),
        *observed(result_actors, "$result", "$correction_record", "CorrectionScope"),
        *observed(result_actors, "$result", "$remedy_record", "RemedyScope"),
        *observed(("source", "review"), "$result", "$end", "EndConditionScope"),
        "observe($temporal, $temporal_record, $end, EndConditionScope)",
        "observe($temporal_review, $temporal_record, $end, EndConditionScope)",
        *observed(("source", "review"), "$result", "IndependentReviewComplete", "ReviewDispositionScope"),
        *observed(("source", "review"), "$result", f"{card.power}FailureWithholdsOnly", "FailurePolarityScope"),
        *observed(("source", "review"), "$result", "$result_reconciliation", "ReconciliationRecordScope"),
        "observe($source, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)",
        "observe($review, $result_reconciliation, StateFormResultReconciled, ReconciliationStatusScope)",
        "observe($source, $result_reconciliation, $result, ResultScope)",
        "observe($review, $result_reconciliation, $result, ResultScope)",
        "observe($source, $result_reconciliation, $record, StateFormRecordScope)",
        "observe($review, $result_reconciliation, $record, StateFormRecordScope)",
        "observe($source, $result_reconciliation, $version, SourceVersionScope)",
        "observe($review, $result_reconciliation, $version, SourceVersionScope)",
        f"observe($source, $result_reconciliation, {card.power}, PowerScope)",
        f"observe($review, $result_reconciliation, {card.power}, PowerScope)",
        "observe($source, $result_reconciliation, $jurisdiction, JurisdictionScope)",
        "observe($review, $result_reconciliation, $jurisdiction, JurisdictionScope)",
        f"observe($source, $result_reconciliation, {branch.jurisdiction_kind}, JurisdictionKindScope)",
        f"observe($review, $result_reconciliation, {branch.jurisdiction_kind}, JurisdictionKindScope)",
        "observe($source, $result_reconciliation, $legal_scope, AuthorityScope)",
        "observe($review, $result_reconciliation, $legal_scope, AuthorityScope)",
        f"observe($source, $result_reconciliation, {branch.legal_scope_kind}, AuthorityScopeKindScope)",
        f"observe($review, $result_reconciliation, {branch.legal_scope_kind}, AuthorityScopeKindScope)",
        *distinct(("source", "evidence", "review")),
    ]
    for field in branch.fields:
        body.extend(observed(result_actors, "$result", field.value, field.scope))
    for authorization in branch.authorizations:
        body.append(
            f"authorized({authorization.actor}, {authorization.authority}, $record)"
        )
    for observation in branch.observations:
        body.append(
            "observe("
            f"{observation.actor}, {observation.subject}, "
            f"{observation.value}, {observation.scope}"
            ")"
        )
    if branch.dynamic:
        body.extend(decision_lineage_premises(branch))
    return list(dict.fromkeys(body))


def authority_raw_premises(branch: RuleBranch) -> list[str]:
    card = branch.card
    body = result_raw_premises(branch)
    body.extend(
        [
            "authorized($executor, InstitutionalExecutionAuthority, $record)",
            "observe($executor, $record, Constitution_StateForm, SourceFamilyScope)",
            "observe($executor, $record, $version, SourceVersionScope)",
            "observe($executor, $record, $epoch, SourceEpochScope)",
            "observe($executor, $record, $temporal_record, TemporalRecordScope)",
            f"observe($executor, $record, {card.power}, PowerScope)",
            "observe($executor, $record, $jurisdiction, JurisdictionScope)",
            f"observe($executor, $record, {branch.jurisdiction_kind}, JurisdictionKindScope)",
            "observe($executor, $record, $legal_scope, AuthorityScope)",
            f"observe($executor, $record, {branch.legal_scope_kind}, AuthorityScopeKindScope)",
            "observe($executor, $record, $reconciliation, ReconciliationRecordScope)",
            "observe($executor, $record, $result, ResultScope)",
            "observe($executor, $result, $end, EndConditionScope)",
            "observe($executor, $result, $result_reconciliation, ReconciliationRecordScope)",
            "~($executor = $source)",
            "~($executor = $evidence)",
            "~($executor = $review)",
        ]
    )
    for holder in branch.authority_holders:
        body.extend(observed(("source", "review", "executor"), "$record", holder, "HolderScope"))
    if branch.dynamic:
        body.extend(
            (
                "~($executor = $admin)",
                "~($executor = $assurer)",
                "~($executor = $service)",
            )
        )
    return list(dict.fromkeys(body))


def v2_rules_for_branch(branch: RuleBranch) -> list[str]:
    result_quantifier = quantified(
        branch_variable_names(branch, authority_stage=False)
    )
    result_body = " & ".join(result_raw_premises(branch))
    result_head = f"complete($result, {branch.card.power}, $record)"
    rules = [f"{result_quantifier}{result_body} -> {result_head}."]
    authority_quantifier = quantified(
        branch_variable_names(branch, authority_stage=True)
    )
    authority_body = " & ".join(authority_raw_premises(branch))
    rules.extend(
        f"{authority_quantifier}{result_head} & {authority_body} -> authority({holder}, {branch.card.power}, $record)."
        for holder in branch.authority_holders
    )
    return rules


def draft_v2_rule_block() -> list[str]:
    rules = [current_rule()]
    for branch in ALL_BRANCHES:
        rules.extend(v2_rules_for_branch(branch))
    return rules

@dataclass(frozen=True)
class ParsedCall:
    name: str
    args: tuple[str, ...]


@dataclass(frozen=True)
class ParsedRule:
    quantified: tuple[str, ...]
    body_calls: tuple[ParsedCall, ...]
    disequalities: tuple[tuple[str, str], ...]
    head: ParsedCall


ALLOWED_RELATION_ARITIES = {
    "authorized": 3,
    "observe": 4,
    "complete": 3,
    "authority": 3,
}
BANNED_RELATIONS = {
    "match",
    "collide",
    "public",
    "choose",
    "decide",
    "broken",
    "approves",
    "mature",
}


def _split_top_level(text: str, separator: str) -> list[str]:
    if not separator:
        raise ValueError("separator must not be empty")
    parts: list[str] = []
    start = 0
    depth = 0
    index = 0
    while index < len(text):
        char = text[index]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth < 0:
                raise RuntimeError(f"unbalanced closing parenthesis in {text!r}")
        elif depth == 0 and text.startswith(separator, index):
            parts.append(text[start:index])
            index += len(separator)
            start = index
            continue
        index += 1
    if depth != 0:
        raise RuntimeError(f"unbalanced parentheses in {text!r}")
    parts.append(text[start:])
    return parts


def _parse_call(text: str) -> ParsedCall:
    match = re.fullmatch(r"([A-Za-z_][A-Za-z0-9_]*)\((.*)\)", text)
    if match is None:
        raise RuntimeError(f"not a positive relation call: {text!r}")
    arguments = tuple(
        item.strip() for item in _split_top_level(match.group(2), ",")
    )
    if any(not item for item in arguments):
        raise RuntimeError(f"empty relation argument in {text!r}")
    return ParsedCall(match.group(1), arguments)


def _parse_rule(statement: str) -> ParsedRule:
    if not statement.endswith("."):
        raise RuntimeError("state-form statement lacks a final period")
    remainder = statement[:-1]
    quantified_names: list[str] = []
    while remainder.startswith("all "):
        match = re.match(
            r"all \$([A-Za-z_][A-Za-z0-9_]*): ",
            remainder,
        )
        if match is None:
            raise RuntimeError(f"malformed universal quantifier in {statement!r}")
        quantified_names.append(match.group(1))
        remainder = remainder[match.end():]
    if "any " in remainder or statement.startswith("any "):
        raise RuntimeError("state-form rules may use universal quantification only")
    if len(quantified_names) != len(set(quantified_names)):
        raise RuntimeError("state-form rule quantifies a variable more than once")
    implication = _split_top_level(remainder, " -> ")
    if len(implication) != 2:
        raise RuntimeError("state-form rule must have exactly one top-level implication")
    body_text, head_text = implication
    body_calls: list[ParsedCall] = []
    disequalities: list[tuple[str, str]] = []
    body_atoms = _split_top_level(body_text, " & ")
    if len(body_atoms) != len(set(body_atoms)):
        raise RuntimeError("state-form rule repeats an exact body atom")
    for atom in body_atoms:
        disequality = re.fullmatch(
            r"~\(\$([A-Za-z_][A-Za-z0-9_]*) = "
            r"\$([A-Za-z_][A-Za-z0-9_]*)\)",
            atom,
        )
        if disequality is not None:
            disequalities.append((disequality.group(1), disequality.group(2)))
            continue
        if atom.startswith("~"):
            raise RuntimeError("negative predicate premises are forbidden")
        body_calls.append(_parse_call(atom))
    head = _parse_call(head_text)
    used = set(
        re.findall(
            r"\$([A-Za-z_][A-Za-z0-9_]*)",
            remainder,
        )
    )
    if used != set(quantified_names):
        raise RuntimeError(
            "quantified/used variable mismatch: "
            f"quantified={sorted(quantified_names)!r}, used={sorted(used)!r}"
        )
    return ParsedRule(
        tuple(quantified_names),
        tuple(body_calls),
        tuple(disequalities),
        head,
    )


def _validate_call(call: ParsedCall, *, in_head: bool) -> None:
    if call.name in BANNED_RELATIONS:
        raise RuntimeError(f"legacy or diagnostic relation {call.name}/{len(call.args)} is forbidden")
    expected_arity = ALLOWED_RELATION_ARITIES.get(call.name)
    if expected_arity is None or len(call.args) != expected_arity:
        raise RuntimeError(f"unapproved relation signature {call.name}/{len(call.args)}")
    if call.name == "authority" and not in_head:
        raise RuntimeError("authority/3 may appear only as a direct-effect head")
    if call.name in {"authorized", "observe"} and in_head:
        raise RuntimeError(f"{call.name}/{len(call.args)} may not appear in a head")
    if any(re.fullmatch(r"[0-9]+", argument) for argument in call.args):
        raise RuntimeError("standalone numeric literals are forbidden")


def _require_call(
    rule: ParsedRule,
    name: str,
    args: tuple[str, ...],
    *,
    context: str,
) -> None:
    if ParsedCall(name, args) not in rule.body_calls:
        raise RuntimeError(f"{context} lacks {name}{args!r}")


def _normalized_disequality_pairs(
    atoms: Sequence[str],
) -> tuple[tuple[str, str], ...]:
    pairs: list[tuple[str, str]] = []
    for atom in atoms:
        match = re.fullmatch(
            r"~\(\$([A-Za-z_][A-Za-z0-9_]*) = "
            r"\$([A-Za-z_][A-Za-z0-9_]*)\)",
            atom,
        )
        if match is not None:
            pairs.append(tuple(sorted(match.groups())))
    return tuple(sorted(pairs))



def _validate_explicit_lineage_contract(
    branch: RuleBranch,
    result_raw: Sequence[str],
) -> None:
    if not branch.dynamic:
        return
    atoms = set(result_raw)
    expected = set(decision_lineage_premises(branch))
    missing = expected - atoms
    if missing:
        raise RuntimeError(
            f"{branch.marker} lost explicit decision lineage: {sorted(missing)!r}"
        )
    combined = "\n".join(result_raw)
    if "$evidence_set" in combined or "$decision_configuration" in combined:
        raise RuntimeError(f"{branch.marker} contains a generic lineage fallback")
    if "$submission_set" in combined and (
        branch.card.number,
        branch.key,
    ) != (10, "assembly_election"):
        raise RuntimeError(f"{branch.marker} contains an unowned submission set")
    declared = set(branch_variable_names(branch, authority_stage=False))
    used = set(re.findall(r"\$([A-Za-z_][A-Za-z0-9_]*)", combined))
    if declared != used:
        raise RuntimeError(
            f"{branch.marker} quantified/used variables differ: "
            f"unused={sorted(declared - used)!r}, "
            f"unquantified={sorted(used - declared)!r}"
        )
    if branch.card.number == 39 and branch.key in {
        "initiative_only_wins",
        "counterproposal_only_wins",
        "both_pass_initiative_larger_share",
        "both_pass_counterproposal_larger_share",
    }:
        for actor in ("source", "evidence", "review"):
            actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
            required = (
                f"observe({actor_term}, $result, "
                "PositiveCompatibilityAndCorridorReviewPassed, "
                "CompatibilityReviewDispositionScope)"
            )
            if required not in atoms:
                raise RuntimeError(
                    f"{branch.marker} lost positive compatibility/corridor review"
                )


def _expect_explicit_lineage_failure(
    branch: RuleBranch,
    result_raw: Sequence[str],
    *,
    label: str,
) -> None:
    try:
        _validate_explicit_lineage_contract(branch, result_raw)
    except RuntimeError:
        return
    raise RuntimeError(f"watched state-form mutation survived: {label}")


def validate_explicit_lineage_rule_seams() -> None:
    current_rejoin = current_rejoin_premises(
        "FSPOW_001",
        "FSPOW_001CommonFederalJurisdictionKind",
        "FSPOW_001ExampleBranchAuthorityScopeKind",
    )
    if any(atom.startswith("~(") for atom in current_rejoin):
        raise RuntimeError("current rejoin duplicated a current-role disequality")
    for branch in ALL_BRANCHES:
        result_raw = result_raw_premises(branch)
        authority_raw = authority_raw_premises(branch)
        if authority_raw[:len(result_raw)] != result_raw:
            raise RuntimeError(f"{branch.marker} authority raw prefix differs")
        if "complete($record, StateFormCurrent, $temporal_record)" not in result_raw:
            raise RuntimeError(f"{branch.marker} lost current-record consumption")
        if any("$executor" in atom for atom in result_raw):
            raise RuntimeError(f"{branch.marker} result depends on executor")
        for field in branch.fields:
            for actor in ("source", "evidence", "review"):
                actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                expected = (
                    f"observe({actor_term}, $result, {field.value}, {field.scope})"
                )
                if expected not in result_raw:
                    raise RuntimeError(
                        f"{branch.marker} lost exact field witness {expected}"
                    )
        for holder in branch.authority_holders:
            for actor in ("source", "review", "executor"):
                actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                expected = f"observe({actor_term}, $record, {holder}, HolderScope)"
                if expected not in authority_raw:
                    raise RuntimeError(
                        f"{branch.marker} lost holder witness {expected}"
                    )
        if branch.dynamic:
            _validate_explicit_lineage_contract(branch, result_raw)
        elif any(
            name in "\n".join(result_raw)
            for name in (
                "DecisionAdministrationAuthority",
                "IndependentCompletenessAssuranceAuthority",
                "ResultServiceAuthority",
            )
        ):
            raise RuntimeError(f"{branch.marker} static rule leaked dynamic roles")


def validate_explicit_lineage_self_controls() -> None:
    for branch in (item for item in ALL_BRANCHES if item.dynamic):
        lineage = DECISION_LINEAGES[(branch.card.number, branch.key)]
        baseline = result_raw_premises(branch)
        _validate_explicit_lineage_contract(branch, baseline)
        set_terms = [
            field
            for interface in lineage.interfaces
            for field in (*interface.rosters, *interface.submissions)
        ]
        set_terms.append(lineage.certificate_set)
        for field in set_terms:
            for actor in ("source", "evidence", "admin", "assurer", "service", "review"):
                actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                needle = (
                    f"observe({actor_term}, $result, {field.value}, {field.scope})"
                )
                if baseline.count(needle) != 1:
                    raise RuntimeError(
                        f"{branch.marker} set self-control drifted: {needle}"
                    )
                _expect_explicit_lineage_failure(
                    branch,
                    [atom for atom in baseline if atom != needle],
                    label=f"{branch.marker} removed {actor} witness for {field.value}",
                )
                rebound = [
                    atom.replace(field.value, "MismatchedDecisionSet")
                    if atom == needle else atom
                    for atom in baseline
                ]
                _expect_explicit_lineage_failure(
                    branch,
                    rebound,
                    label=f"{branch.marker} rebound {actor} witness for {field.value}",
                )
        for interface in lineage.interfaces:
            completeness_specs = [
                *(
                    (
                        roster,
                        "CompleteAndNonzeroEligibleRoster",
                        "RosterCompletenessDispositionScope",
                    )
                    for roster in interface.rosters
                ),
                *(
                    (
                        submission,
                        "CompleteUniqueSubmissionSet",
                        "SubmissionCompletenessDispositionScope",
                    )
                    for submission in interface.submissions
                ),
            ]
            for field, disposition, scope in completeness_specs:
                for actor in ("assurer", "review"):
                    actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                    needle = (
                        f"observe({actor_term}, {field.value}, "
                        f"{disposition}, {scope})"
                    )
                    if baseline.count(needle) != 1:
                        raise RuntimeError(
                            f"{branch.marker} completeness fixture drifted: {needle}"
                        )
                    _expect_explicit_lineage_failure(
                        branch,
                        [atom for atom in baseline if atom != needle],
                        label=(
                            f"{branch.marker} removed {actor} completeness "
                            f"for {field.value}"
                        ),
                    )
        for actor in ("assurer", "review"):
            actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
            needle = (
                f"observe({actor_term}, {lineage.certificate_set.value}, "
                "CompleteUniqueCertificateSet, "
                "CertificateCompletenessDispositionScope)"
            )
            if baseline.count(needle) != 1:
                raise RuntimeError(
                    f"{branch.marker} certificate completeness fixture drifted"
                )
            _expect_explicit_lineage_failure(
                branch,
                [atom for atom in baseline if atom != needle],
                label=f"{branch.marker} removed {actor} certificate completeness",
            )
        for interface in lineage.interfaces:
            for field in interface.outcomes:
                witnesses = (
                    *(
                        (actor, "$result", "result witness")
                        for actor in ("source", "evidence", "service", "review")
                    ),
                    *(
                        (actor, interface.identity.value, "interface link")
                        for actor in ("service", "review")
                    ),
                    *(
                        (
                            actor,
                            lineage.result_certificate.value,
                            "certificate link",
                        )
                        for actor in ("service", "review")
                    ),
                )
                for actor, subject, label in witnesses:
                    actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                    needle = f"observe({actor_term}, {subject}, {field.value}, {field.scope})"
                    if baseline.count(needle) != 1:
                        raise RuntimeError(
                            f"{branch.marker} outcome {label} fixture drifted: {needle}"
                        )
                    _expect_explicit_lineage_failure(
                        branch,
                        [atom for atom in baseline if atom != needle],
                        label=(
                            f"{branch.marker} removed {actor} outcome {label} "
                            f"for {field.value}"
                        ),
                    )
                    rebound_needle = (
                        f"observe({actor_term}, {subject}, "
                        f"CrossSwappedOutcome, {field.scope})"
                    )
                    _expect_explicit_lineage_failure(
                        branch,
                        [
                            rebound_needle
                            if atom == needle else atom
                            for atom in baseline
                        ],
                        label=(
                            f"{branch.marker} rebound {actor} outcome {label} "
                            f"for {field.value}"
                        ),
                    )
        for actor in ("service", "review"):
            actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
            links = (
                (
                    lineage.certificate_set.value,
                    lineage.result_certificate.value,
                    "ResultCertificateScope",
                    "certificate-set link",
                ),
                (
                    lineage.result_certificate.value,
                    lineage.certified_result.value,
                    "ResultScope",
                    "certified-result link",
                ),
            )
            for subject, value, scope, label in links:
                needle = f"observe({actor_term}, {subject}, {value}, {scope})"
                if baseline.count(needle) != 1:
                    raise RuntimeError(
                        f"{branch.marker} certificate-link fixture drifted: {needle}"
                    )
                _expect_explicit_lineage_failure(
                    branch,
                    [atom for atom in baseline if atom != needle],
                    label=f"{branch.marker} removed {actor} {label}",
                )
                _expect_explicit_lineage_failure(
                    branch,
                    [
                        atom.replace(value, "CrossSwappedDecisionResult")
                        if atom == needle else atom
                        for atom in baseline
                    ],
                    label=f"{branch.marker} cross-swapped {actor} {label}",
                )
        for link in lineage.upstream_links:
            for actor in ("service", "review"):
                actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                needle = (
                    f"observe({actor_term}, {link.certificate.value}, "
                    f"{link.result.value}, ResultScope)"
                )
                if baseline.count(needle) != 1:
                    raise RuntimeError(
                        f"{branch.marker} upstream-link fixture drifted: {needle}"
                    )
                _expect_explicit_lineage_failure(
                    branch,
                    [atom for atom in baseline if atom != needle],
                    label=f"{branch.marker} removed upstream certificate link",
                )
        if len(lineage.interfaces) > 1:
            first, second = lineage.interfaces[:2]
            needle = (
                f"observe($service, {lineage.result_certificate.value}, "
                f"{first.identity.value}, DecisionInterfaceScope)"
            )
            _expect_explicit_lineage_failure(
                branch,
                [
                    atom.replace(first.identity.value, second.identity.value)
                    if atom == needle else atom
                    for atom in baseline
                ],
                label=f"{branch.marker} cross-swapped decision stages",
            )
        if branch.card.number == 39 and branch.key in {
            "initiative_only_wins",
            "counterproposal_only_wins",
            "both_pass_initiative_larger_share",
            "both_pass_counterproposal_larger_share",
        }:
            needle = (
                "observe($source, $result, "
                "PositiveCompatibilityAndCorridorReviewPassed, "
                "CompatibilityReviewDispositionScope)"
            )
            _expect_explicit_lineage_failure(
                branch,
                [atom for atom in baseline if atom != needle],
                label=f"{branch.marker} omitted positive compatibility review",
            )
            _expect_explicit_lineage_failure(
                branch,
                [
                    atom.replace(
                        "PositiveCompatibilityAndCorridorReviewPassed",
                        "AdverseCompatibilityReview",
                    )
                    if atom == needle else atom
                    for atom in baseline
                ],
                label=f"{branch.marker} rebound compatibility review",
            )


def validate_rule_surface(statements: Sequence[str]) -> tuple[ParsedRule, ...]:
    if len(statements) != EXPECTED_STATEMENT_COUNT:
        raise RuntimeError(
            f"expected {EXPECTED_STATEMENT_COUNT} statements, found {len(statements)}"
        )
    if len(set(statements)) != len(statements):
        raise RuntimeError("state-form statements are not unique")
    if sum(
        statement.count("~($source = $temporal_review)")
        for statement in statements
    ) != 1:
        raise RuntimeError("independent-current-review guard must occur exactly once")
    if any("FALSE" in statement for statement in statements):
        raise RuntimeError("state-form rules may not derive or consume FALSE")
    parsed = tuple(_parse_rule(statement) for statement in statements)
    normalized_review_guards = [
        (index, tuple(sorted(pair)))
        for index, rule in enumerate(parsed)
        for pair in rule.disequalities
        if set(pair) == {"source", "temporal_review"}
    ]
    if normalized_review_guards != [
        (0, ("source", "temporal_review"))
    ]:
        raise RuntimeError(
            "source/temporal-review separation must occur once, "
            "in the shared current rule only"
        )
    expected_current_pairs = {
        ("source", "temporal"),
        ("source", "temporal_review"),
        ("record_review", "source"),
        ("temporal", "temporal_review"),
        ("record_review", "temporal"),
        ("record_review", "temporal_review"),
    }
    current_pairs = {
        tuple(sorted(pair)) for pair in parsed[0].disequalities
    }
    if current_pairs != expected_current_pairs:
        raise RuntimeError(
            f"shared current separation set changed: {sorted(current_pairs)!r}"
        )

    signatures: set[tuple[str, int]] = set()
    for rule in parsed:
        for call in rule.body_calls:
            _validate_call(call, in_head=False)
            signatures.add((call.name, len(call.args)))
        _validate_call(rule.head, in_head=True)
        signatures.add((rule.head.name, len(rule.head.args)))
    expected_signatures = {
        ("authorized", 3),
        ("observe", 4),
        ("complete", 3),
        ("authority", 3),
    }
    if signatures != expected_signatures:
        raise RuntimeError(f"state-form relation signatures changed: {sorted(signatures)!r}")

    if parsed[0].head != ParsedCall(
        "complete",
        ("$record", "StateFormCurrent", "$temporal_record"),
    ):
        raise RuntimeError("first statement is not the shared current declaration")
    cursor = 1
    for branch in ALL_BRANCHES:
        result = parsed[cursor]
        expected_result = (
            "$result",
            branch.card.power,
            "$record",
        )
        if result.head != ParsedCall("complete", expected_result):
            raise RuntimeError(f"{branch.marker} result head/order changed")
        for actor in ("source", "evidence", "review"):
            actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
            _require_call(
                result,
                "observe",
                (actor_term, "$result", branch.marker, "StateFormBranchScope"),
                context=branch.marker,
            )
        cursor += 1
        for holder in branch.authority_holders:
            authority_rule = parsed[cursor]
            if authority_rule.head != ParsedCall(
                "authority",
                (holder, branch.card.power, "$record"),
            ):
                raise RuntimeError(f"{branch.marker}/{holder} direct-effect head changed")
            _require_call(
                authority_rule,
                "complete",
                expected_result,
                context=f"{branch.marker}/{holder}",
            )
            for required_holder in branch.authority_holders:
                for actor in ("source", "review", "executor"):
                    actor_term = f"DOLLAR{actor}".replace("DOLLAR", "$")
                    _require_call(
                        authority_rule,
                        "observe",
                        (actor_term, "$record", required_holder, "HolderScope"),
                        context=f"{branch.marker}/{holder}",
                    )
            cursor += 1
    if cursor != len(parsed):
        raise RuntimeError("unaccounted state-form statements remain")
    return parsed


def validate_draft_rules() -> tuple[str, ...]:
    validate_branch_inventory()
    validate_explicit_lineage_rule_seams()
    validate_explicit_lineage_self_controls()
    statements = tuple(draft_v2_rule_block())
    validate_rule_surface(statements)
    actual_rule_sha256 = hashlib.sha256(
        ("\n".join(statements) + "\n").encode("utf-8")
    ).hexdigest()
    if actual_rule_sha256 != EXPECTED_RULE_BLOCK_SHA256:
        raise RuntimeError(
            "state-form exact rule block changed: "
            f"expected {EXPECTED_RULE_BLOCK_SHA256}, found {actual_rule_sha256}"
        )

    return statements

def validate_delegation_markers() -> None:
    actual_paths = tuple(
        sorted(
            path.relative_to(ROOT).as_posix()
            for path in (ROOT / "book-1").glob("*.pins.nibli")
            if DELEGATION_MARKER in path.read_text(encoding="utf-8")
        )
    )
    if actual_paths != DELEGATED_PIN_PATHS:
        raise RuntimeError(
            "state-form chapter-pin delegation markers changed: "
            f"{actual_paths!r}"
        )
    for relative_path in DELEGATED_PIN_PATHS:
        text = (ROOT / relative_path).read_text(encoding="utf-8")
        if text.count(DELEGATION_MARKER) != 1:
            raise RuntimeError(
                f"{relative_path} must contain one exact state-form delegation marker"
            )



def rule_block() -> list[str]:
    if not RENDERER_UNLOCKED:
        raise RuntimeError(
            "FS-CVF-003 renderer remains locked pending independent review"
        )
    return list(validate_draft_rules())


def rendered_block() -> str:
    comments = [
        BEGIN,
        "# [2026-08-21] FS-CVF-003. Supplied records only: these rules reuse",
        "# admitted authorized/3 and observe/4, and derived complete/3 and",
        "# authority/3. They add no relation name, arity, admission, or fact.",
        "# The conclusions are bounded legal declarations and authority only:",
        "# never authentication, computation, action, delivery, liveness,",
        "# feasibility, or outside time. A falsely supplied current or reconciled",
        "# attestation remains an external trust-root failure. No producer reads",
        "# a negative predicate, diagnostic conflict, or legacy conclusion.",
    ]
    return "\n".join([*comments, *rule_block(), END]) + "\n"


def extract_block(text: str) -> str:
    if (
        text.count(BEGIN + "\n") != 1
        or text.count(END + "\n") != 1
        or text.index(BEGIN) >= text.index(END)
    ):
        raise RuntimeError(
            "constitution must contain one ordered state-form marker pair"
        )
    pattern = re.compile(
        rf"^{re.escape(BEGIN)}\n.*?^{re.escape(END)}\n",
        re.MULTILINE | re.DOTALL,
    )
    found = pattern.findall(text)
    if len(found) != 1:
        raise RuntimeError(f"expected one state-form block, found {len(found)}")
    return found[0]


VARIABLE_PATTERN = re.compile(r"\$[a-z][a-z0-9_]*")


@dataclass(frozen=True)
class AtomSelector:
    exact: str | None = None
    all_of: tuple[str, ...] = ()
    any_of: tuple[str, ...] = ()

    def matches(self, atom: str) -> bool:
        if self.exact is not None and atom != self.exact:
            return False
        if any(needle not in atom for needle in self.all_of):
            return False
        if self.any_of and not any(needle in atom for needle in self.any_of):
            return False
        return True


@dataclass(frozen=True)
class Omission:
    selectors: tuple[AtomSelector, ...] = ()


@dataclass(frozen=True)
class GroundedFixture:
    facts: tuple[str, ...]
    mapping: tuple[tuple[str, str], ...]

    def term(self, variable: str) -> str:
        return dict(self.mapping)[variable]


def _raw_fixture_atoms(branch: RuleBranch) -> tuple[str, ...]:
    premises = (*current_rule_premises(), *authority_raw_premises(branch))
    return tuple(
        dict.fromkeys(
            atom.strip()
            for atom in premises
            if atom.strip().startswith(("authorized(", "observe("))
        )
    )


def _omit_fixture_atoms(
    atoms: Sequence[str],
    omission: Omission,
) -> tuple[str, ...]:
    removed: set[str] = set()
    for selector in omission.selectors:
        hits = {atom for atom in atoms if selector.matches(atom)}
        if not hits:
            raise RuntimeError(
                f"state-form omission selector matched no atom: {selector!r}"
            )
        if selector.exact is not None and len(hits) != 1:
            raise RuntimeError(
                "exact state-form omission selector is not unique: "
                f"{selector.exact!r}"
            )
        removed.update(hits)
    remaining = tuple(atom for atom in atoms if atom not in removed)
    for selector in omission.selectors:
        if any(selector.matches(atom) for atom in remaining):
            raise RuntimeError(
                f"state-form omission selector left a matching atom: {selector!r}"
            )
    return remaining


def _fixture_constant(prefix: str, variable: str) -> str:
    words = variable[1:].split("_")
    return prefix + "".join(word.title() for word in words)


def _ground_fixture(
    branch: RuleBranch,
    prefix: str,
    *,
    fused_current_review: bool = False,
    omission: Omission = Omission(),
    overrides: Sequence[tuple[str, str]] = (),
) -> GroundedFixture:
    atoms = _omit_fixture_atoms(_raw_fixture_atoms(branch), omission)
    variables = tuple(
        dict.fromkeys(VARIABLE_PATTERN.findall("\n".join(atoms)))
    )
    mapping = {
        variable: _fixture_constant(prefix, variable)
        for variable in variables
    }
    mapping["$power"] = branch.card.power
    mapping.update(dict(overrides))
    if fused_current_review:
        mapping["$temporal_review"] = mapping["$source"]

    def ground(atom: str) -> str:
        return VARIABLE_PATTERN.sub(
            lambda found: mapping[found.group(0)],
            atom,
        )

    facts = tuple(dict.fromkeys(ground(atom) for atom in atoms))
    if any("$" in fact for fact in facts):
        raise RuntimeError(f"ungrounded state-form fixture: {prefix}")
    if any(
        not fact.startswith(("authorized(", "observe("))
        for fact in facts
    ):
        raise RuntimeError(f"non-base state-form fixture fact: {prefix}")
    return GroundedFixture(facts=facts, mapping=tuple(mapping.items()))


def _branch_holder_rows() -> tuple[tuple[RuleBranch, str], ...]:
    return tuple(
        (branch, holder)
        for branch in ALL_BRANCHES
        for holder in branch.authority_holders
    )


def _canonical_power_rows() -> tuple[tuple[RuleBranch, str], ...]:
    rows = []
    for number in range(1, EXPECTED_CARD_COUNT + 1):
        branch = next(
            item for item in ALL_BRANCHES if item.card.number == number
        )
        rows.append((branch, branch.authority_holders[0]))
    return tuple(rows)


def _authority_query(
    branch: RuleBranch,
    holder: str,
    fixture: GroundedFixture,
) -> str:
    return (
        f"authority({holder}, {branch.card.power}, "
        f"{fixture.term('$record')})"
    )


def _complete_query(
    branch: RuleBranch,
    fixture: GroundedFixture,
    *,
    result: str | None = None,
) -> str:
    result_term = result if result is not None else fixture.term("$result")
    return (
        f"complete({result_term}, {branch.card.power}, "
        f"{fixture.term('$record')})"
    )


def _append_facts(lines: list[str], fixture: GroundedFixture) -> None:
    lines.extend(f"{fact}." for fact in fixture.facts)


def _append_query(
    lines: list[str],
    query: str,
    expected: bool,
) -> None:
    lines.append(f"? {query}.")
    lines.append(f"# => {'TRUE' if expected else 'FALSE'}")
    lines.append("")


def _append_fixture_query(
    lines: list[str],
    branch: RuleBranch,
    holder: str,
    fixture: GroundedFixture,
    expected: bool,
) -> None:
    _append_facts(lines, fixture)
    _append_query(lines, _authority_query(branch, holder, fixture), expected)


def _positive_fixture_registry() -> dict[
    tuple[int, str, str],
    tuple[RuleBranch, GroundedFixture],
]:
    registry = {}
    for index, (branch, holder) in enumerate(_branch_holder_rows(), 1):
        registry[(branch.card.number, branch.key, holder)] = (
            branch,
            _ground_fixture(branch, f"SFMainP{index:03d}"),
        )
    return registry


def _registry_fixture(
    registry: dict[
        tuple[int, str, str],
        tuple[RuleBranch, GroundedFixture],
    ],
    number: int,
    key: str,
    holder: str,
) -> tuple[RuleBranch, GroundedFixture]:
    try:
        return registry[(number, key, holder)]
    except KeyError as error:
        raise RuntimeError(
            f"unknown state-form positive fixture {number:03d}/{key}/{holder}"
        ) from error


def _append_existing_authority(
    lines: list[str],
    registry: dict[
        tuple[int, str, str],
        tuple[RuleBranch, GroundedFixture],
    ],
    number: int,
    key: str,
    holder: str,
) -> GroundedFixture:
    branch, fixture = _registry_fixture(
        registry,
        number,
        key,
        holder,
    )
    _append_query(lines, _authority_query(branch, holder, fixture), True)
    return fixture


def _append_omission_negative(
    lines: list[str],
    number: int,
    key: str,
    holder: str,
    prefix: str,
    omission: Omission,
) -> GroundedFixture:
    branch = _branch_lookup(number, key)
    fixture = _ground_fixture(
        branch,
        prefix,
        omission=omission,
    )
    _append_fixture_query(lines, branch, holder, fixture, False)
    return fixture


def render_state_form_pins() -> str:
    registry = _positive_fixture_registry()
    lines = [
        "# SPDX-License-Identifier: MIT OR Apache-2.0",
        STATE_FORM_PINS_HEADER,
        "#",
        "# These fixtures supply bounded source records. They do not prove that",
        "# any institution, roster, result, office, or remedy exists outside",
        "# this executable probe.",
        f":expect-pins {EXPECTED_MAIN_PIN_COUNT}",
        "",
        "# <STATE-FORM-GENERIC-POSITIVE-BEGIN>",
    ]
    generic_count = 0
    for index, (branch, holder) in enumerate(_branch_holder_rows(), 1):
        fixture = registry[(branch.card.number, branch.key, holder)][1]
        lines.append(
            f"# {branch.card.power}/{branch.key}/{holder} positive authority."
        )
        _append_fixture_query(lines, branch, holder, fixture, True)
        generic_count += 1
    lines.append("# <STATE-FORM-GENERIC-POSITIVE-END>")
    lines.append("")
    lines.append("# <STATE-FORM-MISSING-REVIEW-BEGIN>")
    missing_review = Omission(
        (
            AtomSelector(
                exact=(
                    "authorized($review, "
                    "IndependentStateFormReviewAuthority, $record)"
                )
            ),
        )
    )
    for index, (branch, holder) in enumerate(_branch_holder_rows(), 1):
        fixture = _ground_fixture(
            branch,
            f"SFMainN{index:03d}",
            omission=missing_review,
        )
        lines.append(
            f"# {branch.card.power}/{branch.key}/{holder} "
            "missing independent review."
        )
        _append_fixture_query(lines, branch, holder, fixture, False)
        generic_count += 1
    lines.append("# <STATE-FORM-MISSING-REVIEW-END>")
    lines.append("")
    lines.append("# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-BEGIN>")
    for branch, holder in _canonical_power_rows():
        fixture = _ground_fixture(
            branch,
            f"SFMainF{branch.card.number:03d}",
            fused_current_review=True,
        )
        lines.append(
            f"# FS-POW-{branch.card.number:03d} negative: fused "
            "source/current reviewer cannot derive authority."
        )
        _append_fixture_query(lines, branch, holder, fixture, False)
        generic_count += 1
    lines.append("# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-END>")
    lines.append("")
    if generic_count != EXPECTED_GENERIC_MAIN_PIN_COUNT:
        raise RuntimeError(
            f"expected {EXPECTED_GENERIC_MAIN_PIN_COUNT} generic pins, "
            f"found {generic_count}"
        )
    acceptance_lines, acceptance_count = _render_acceptance_cases(registry)
    if acceptance_count != EXPECTED_ACCEPTANCE_PIN_COUNT:
        raise RuntimeError(
            f"expected {EXPECTED_ACCEPTANCE_PIN_COUNT} acceptance pins, "
            f"found {acceptance_count}"
        )
    lines.extend(acceptance_lines)
    rendered = "\n".join(lines).rstrip() + "\n"
    if _query_count(rendered) != EXPECTED_MAIN_PIN_COUNT:
        raise RuntimeError("rendered state-form main pin count drifted")
    return rendered


def render_state_form_counterfactual() -> str:
    constitution_text = CONSTITUTION.read_text(encoding="utf-8")
    source_rule = current_rule()
    if constitution_text.count(source_rule) != 1:
        raise RuntimeError(
            "state-form current rule must occur once in the constitution"
        )
    if source_rule.count(CURRENT_REVIEW_GUARD) != 1:
        raise RuntimeError(
            "state-form current rule must contain one independent-review guard"
        )
    mutated_rule = source_rule.replace(CURRENT_REVIEW_GUARD, "", 1)
    if CURRENT_REVIEW_GUARD in mutated_rule:
        raise RuntimeError("state-form counterfactual guard removal failed")
    counterfactual = constitution_text.replace(source_rule, mutated_rule, 1)
    source_lines = constitution_text.splitlines()
    counterfactual_lines = counterfactual.splitlines()
    if len(source_lines) != len(counterfactual_lines):
        raise RuntimeError("state-form counterfactual changed line count")
    changed = [
        (old, new)
        for old, new in zip(source_lines, counterfactual_lines, strict=True)
        if old != new
    ]
    if changed != [(source_rule, mutated_rule)]:
        raise RuntimeError(
            "state-form counterfactual is not the exact one-line mutation"
        )
    return counterfactual


def render_state_form_counterfactual_pins() -> str:
    lines = [
        "# SPDX-License-Identifier: MIT OR Apache-2.0",
        STATE_FORM_COUNTERFACTUAL_PINS_HEADER,
        f":expect-pins {EXPECTED_COUNTERFACTUAL_PIN_COUNT}",
        "",
    ]
    count = 0
    for branch, holder in _canonical_power_rows():
        fixture = _ground_fixture(
            branch,
            f"SFMainF{branch.card.number:03d}",
            fused_current_review=True,
        )
        lines.append(
            f"# FS-POW-{branch.card.number:03d} counterfactual: removing "
            "the independent-current-review guard derives authority."
        )
        _append_fixture_query(lines, branch, holder, fixture, True)
        count += 1
    if count != EXPECTED_COUNTERFACTUAL_PIN_COUNT:
        raise RuntimeError(
            "rendered state-form counterfactual pin count drifted"
        )
    rendered = "\n".join(lines).rstrip() + "\n"
    if _query_count(rendered) != EXPECTED_COUNTERFACTUAL_PIN_COUNT:
        raise RuntimeError(
            "rendered state-form counterfactual query count drifted"
        )
    return rendered


def _query_count(text: str) -> int:
    return sum(line.startswith("? ") for line in text.splitlines())

def _render_acceptance_cases(
    registry: dict[
        tuple[int, str, str],
        tuple[RuleBranch, GroundedFixture],
    ],
) -> tuple[list[str], int]:
    lines = ["# <STATE-FORM-ACCEPTANCE-CASES-BEGIN>", ""]
    count = 0

    def header(case_id: str) -> None:
        lines.append(f"# {case_id}")

    def existing(
        number: int,
        key: str,
        holder: str,
    ) -> GroundedFixture:
        nonlocal count
        fixture = _append_existing_authority(
            lines,
            registry,
            number,
            key,
            holder,
        )
        count += 1
        return fixture

    def negative(
        number: int,
        key: str,
        holder: str,
        prefix: str,
        omission: Omission,
    ) -> GroundedFixture:
        nonlocal count
        fixture = _append_omission_negative(
            lines,
            number,
            key,
            holder,
            prefix,
            omission,
        )
        count += 1
        return fixture

    def query(query_text: str, expected: bool) -> None:
        nonlocal count
        _append_query(lines, query_text, expected)
        count += 1

    header(ACCEPTANCE_CASE_IDS[0])
    franchise_branch = _branch_lookup(
        36, "adult_resident_franchise"
    )
    franchise = _ground_fixture(
        franchise_branch,
        "SFAcc001Franchise",
        overrides=(("$subject", "Ruk"),),
    )
    candidacy_branch = _branch_lookup(
        36, "adult_resident_candidacy"
    )
    candidacy = _ground_fixture(
        candidacy_branch,
        "SFAcc001Candidacy",
        overrides=(("$subject", "Ruk"),),
    )
    _append_facts(lines, franchise)
    _append_facts(lines, candidacy)
    query("prisoner(Ruk)", True)
    query(
        _authority_query(franchise_branch, "FSBOD_06", franchise),
        True,
    )
    query(
        _authority_query(candidacy_branch, "FSBOD_06", candidacy),
        True,
    )

    header(ACCEPTANCE_CASE_IDS[1])
    custody_branch = _branch_lookup(
        36, "compelled_placement_nonchange"
    )
    custody = _ground_fixture(
        custody_branch,
        "SFAcc002Custody",
        overrides=(("$subject", "Ruk"),),
    )
    _append_facts(lines, custody)
    query("prisoner(Ruk)", True)
    query(_authority_query(custody_branch, "FSBOD_06", custody), True)

    header(ACCEPTANCE_CASE_IDS[2])
    existing(
        36,
        "accessible_nonconventional_residence",
        "FSBOD_06",
    )

    header(ACCEPTANCE_CASE_IDS[3])
    existing(36, "multiple_residences_first_choice", "FSBOD_06")
    existing(36, "multiple_residences_second_choice", "FSBOD_06")
    negative(
        36,
        "multiple_residences_first_choice",
        "FSBOD_06",
        "SFAcc004NoChoice",
        Omission(
            (
                AtomSelector(
                    exact=(
                        "authorized($subject, "
                        "PoliticalHomeChoiceAuthority, $record)"
                    )
                ),
            )
        ),
    )

    header(ACCEPTANCE_CASE_IDS[4])
    existing(36, "atomic_home_transfer", "FSBOD_06")
    existing(36, "unique_accepted_submission", "FSBOD_06")

    header(ACCEPTANCE_CASE_IDS[5])
    existing(36, "established_adulthood_continuity", "FSBOD_06")
    existing(
        36,
        "provisional_adulthood_expiring_opportunity",
        "FSBOD_06",
    )

    header(ACCEPTANCE_CASE_IDS[6])
    existing(36, "office_move_continuity", "FSBOD_06")

    header(ACCEPTANCE_CASE_IDS[7])
    nonresident_branch, nonresident = _registry_fixture(
        registry,
        36,
        "positive_nonresident_disposition",
        "FSBOD_06",
    )
    query(
        _authority_query(nonresident_branch, "FSBOD_06", nonresident),
        True,
    )
    query(_complete_query(nonresident_branch, nonresident), True)
    query(
        _complete_query(
            nonresident_branch,
            nonresident,
            result="SFAcc008FranchiseResult",
        ),
        False,
    )
    existing(
        36,
        "former_resident_return_without_ballot",
        "FSBOD_06",
    )

    header(ACCEPTANCE_CASE_IDS[8])
    existing(28, "assembly_appointment_selection", "FSBOD_02")
    existing(
        30,
        "assembly_captured_source_fallback_appointment",
        "FSBOD_02",
    )
    existing(35, "assembly_seat_allocation", "FSBOD_02")
    negative(
        28,
        "assembly_appointment_selection",
        "FSBOD_02",
        "SFAcc009NoAntiCapture",
        Omission((AtomSelector(all_of=("AntiCaptureScope",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[9])
    caretaker = existing(
        41,
        "last_lawful_government_caretaker",
        "FSBOD_04",
    )
    query(
        f"authority(FSBOD_04, FSPOW_014, {caretaker.term('$record')})",
        False,
    )
    existing(42, "fresh_election_call", "FSBOD_06")
    negative(
        42,
        "fresh_election_call",
        "FSBOD_06",
        "SFAcc010NoDeadline",
        Omission((AtomSelector(all_of=("PositiveDeadlinePassed",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[10])
    existing(18, "formal_government_appointment", "FSBOD_05")
    existing(19, "promulgation", "FSBOD_05")
    existing(20, "certificate_receipt", "FSBOD_05")
    existing(21, "refusal_trigger", "FSBOD_26")
    existing(44, "common_office_transfer", "FSBOD_05")

    header(ACCEPTANCE_CASE_IDS[11])
    existing(12, "one_time_return", "FSBOD_03")
    existing(5, "same_rule_repassage", "FSBOD_02")
    negative(
        12,
        "one_time_return",
        "FSBOD_03",
        "SFAcc012NoUnusedReturn",
        Omission((AtomSelector(all_of=("UnusedReturnScope",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[12])
    existing(10, "assembly_election", "FSBOD_06")
    negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc013NoProportionalOutcome",
        Omission((AtomSelector(all_of=("ProportionalOutcome",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[13])
    existing(43, "essential_budget_continuity", "FSBOD_07")
    existing(43, "valid_budget_ends_continuity", "FSBOD_07")
    existing(43, "continuity_limit_ends_authority", "FSBOD_07")

    header(ACCEPTANCE_CASE_IDS[14])
    existing(25, "alternate_composition_panel", "FSBOD_25")
    negative(
        25,
        "alternate_composition_panel",
        "FSBOD_25",
        "SFAcc015NoAlternatePanel",
        Omission((AtomSelector(all_of=("UninvolvedAlternatePanel",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[15])
    negative(
        5,
        "same_rule_repassage",
        "FSBOD_02",
        "SFAcc016NoLegislativeCorridor",
        Omission(
            (AtomSelector(all_of=("EntrenchedDemocraticCorridor",)),)
        ),
    )
    negative(
        37,
        "ordinary_amendment",
        "FSBOD_01",
        "SFAcc016NoAmendmentCorridor",
        Omission(
            (
                AtomSelector(
                    all_of=("CompatibilityAndCorridorReviewComplete",)
                ),
            )
        ),
    )
    negative(
        39,
        "initiative_only_wins",
        "FSBOD_01",
        "SFAcc016NoInitiativeCorridor",
        Omission(
            (
                AtomSelector(
                    all_of=(
                        "PositiveCompatibilityAndCorridorReviewPassed",
                    )
                ),
            )
        ),
    )
    negative(
        45,
        "final_exit_no_collective_impact",
        "FSBOD_01",
        "SFAcc016NoSecessionRightsReview",
        Omission(
            (
                AtomSelector(
                    all_of=("PositiveRightsAndMinorityReviewPassed",)
                ),
            )
        ),
    )

    header(ACCEPTANCE_CASE_IDS[16])
    existing(45, "opening_referendum", "FSBOD_01")
    existing(45, "completed_negotiation", "FSBOD_02")
    existing(45, "completed_negotiation", "FSBOD_03")
    existing(45, "final_exit_no_collective_impact", "FSBOD_01")
    existing(45, "final_exit_with_collective_consent", "FSBOD_01")
    existing(45, "final_exit_with_collective_consent", "FSBOD_21")
    negative(
        45,
        "final_exit_with_collective_consent",
        "FSBOD_01",
        "SFAcc017NoSettlement",
        Omission((AtomSelector(all_of=("PositiveSettlementComplete",)),)),
    )

    header(ACCEPTANCE_CASE_IDS[17])
    negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc018NoExactRoster",
        Omission(
            (
                AtomSelector(
                    all_of=(
                        "$eligible_roster",
                        "CompleteAndNonzeroEligibleRoster",
                    )
                ),
            )
        ),
    )
    negative(
        10,
        "assembly_election",
        "FSBOD_06",
        "SFAcc018NoExactSubmissions",
        Omission(
            (
                AtomSelector(
                    all_of=(
                        "$submission_set",
                        "CompleteUniqueSubmissionSet",
                    )
                ),
            )
        ),
    )

    header(ACCEPTANCE_CASE_IDS[18])
    lineage = DECISION_LINEAGES[(44, "common_office_transfer")]
    negative(
        44,
        "common_office_transfer",
        "FSBOD_05",
        "SFAcc019NoCertificateChain",
        Omission(
            (
                AtomSelector(
                    any_of=(
                        "$successor",
                        "SuccessorScope",
                        lineage.certificate_set.value,
                        lineage.certificate_set.scope,
                        lineage.result_certificate.value,
                        lineage.result_certificate.scope,
                        lineage.certified_result.value,
                        lineage.certified_result.scope,
                        "CompleteUniqueCertificateSet",
                    )
                ),
            )
        ),
    )
    negative(
        44,
        "common_office_transfer",
        "FSBOD_05",
        "SFAcc019NoCertificateCompleteness",
        Omission(
            (
                AtomSelector(
                    all_of=("CompleteUniqueCertificateSet",)
                ),
            )
        ),
    )

    lines.append("# <STATE-FORM-ACCEPTANCE-CASES-END>")
    lines.append("")
    if tuple(
        line[2:]
        for line in lines
        if line.startswith("# FSACC-")
    ) != ACCEPTANCE_CASE_IDS:
        raise RuntimeError("state-form acceptance case manifest drifted")
    return lines, count

def _sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _validate_expected_sha256(label: str, expected: str) -> None:
    if re.fullmatch(r"[0-9a-f]{64}", expected) is None:
        raise RuntimeError(f"{label} expected SHA-256 is not frozen")


def _validate_pin_surface(
    text: str,
    *,
    header: str,
    expected_count: int,
    allow_prisoner: bool,
) -> None:
    lines = text.splitlines()
    if not lines or lines[0] != (
        "# SPDX-License-Identifier: MIT OR Apache-2.0"
    ):
        raise RuntimeError("state-form pin SPDX header drifted")
    if lines.count(header) != 1:
        raise RuntimeError(f"state-form pin family header drifted: {header}")
    if lines.count(f":expect-pins {expected_count}") != 1:
        raise RuntimeError("state-form pin expectation directive drifted")
    if _query_count(text) != expected_count:
        raise RuntimeError(
            f"expected {expected_count} state-form queries, "
            f"found {_query_count(text)}"
        )
    for line in lines:
        if (
            not line
            or line.startswith("#")
            or line.startswith(":")
            or line.startswith("? ")
        ):
            continue
        if not line.endswith("."):
            raise RuntimeError(f"malformed state-form fixture fact: {line!r}")
        atom = line[:-1]
        if atom.startswith(("complete(", "authority(")):
            raise RuntimeError(
                "state-form fixtures must not assert derived declarations "
                "or authority"
            )
        if atom.startswith(("authorized(", "observe(")):
            continue
        if allow_prisoner and atom.startswith("prisoner("):
            continue
        raise RuntimeError(f"unexpected state-form fixture fact: {atom!r}")


def _validate_main_pin_manifest(text: str) -> None:
    for case_id in ACCEPTANCE_CASE_IDS:
        if text.count(f"# {case_id}\n") != 1:
            raise RuntimeError(
                f"state-form acceptance case header drifted: {case_id}"
            )
    for number in range(1, EXPECTED_CARD_COUNT + 1):
        comment = (
            f"# FS-POW-{number:03d} negative: fused source/current "
            "reviewer cannot derive authority.\n"
        )
        if text.count(comment) != 1:
            raise RuntimeError(
                f"state-form main negative comment drifted: FS-POW-{number:03d}"
            )
    marker_pairs = (
        (
            "# <STATE-FORM-GENERIC-POSITIVE-BEGIN>",
            "# <STATE-FORM-GENERIC-POSITIVE-END>",
        ),
        (
            "# <STATE-FORM-MISSING-REVIEW-BEGIN>",
            "# <STATE-FORM-MISSING-REVIEW-END>",
        ),
        (
            "# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-BEGIN>",
            "# <STATE-FORM-INDEPENDENT-CURRENT-REVIEW-END>",
        ),
        (
            "# <STATE-FORM-ACCEPTANCE-CASES-BEGIN>",
            "# <STATE-FORM-ACCEPTANCE-CASES-END>",
        ),
    )
    for begin, end in marker_pairs:
        if text.count(begin) != 1 or text.count(end) != 1:
            raise RuntimeError(
                f"state-form pin marker drifted: {begin} / {end}"
            )
        if text.index(begin) >= text.index(end):
            raise RuntimeError(
                f"state-form pin marker order drifted: {begin} / {end}"
            )


def _validate_counterfactual_pin_manifest(text: str) -> None:
    for number in range(1, EXPECTED_CARD_COUNT + 1):
        comment = (
            f"# FS-POW-{number:03d} counterfactual: removing the "
            "independent-current-review guard derives authority.\n"
        )
        if text.count(comment) != 1:
            raise RuntimeError(
                "state-form counterfactual comment drifted: "
                f"FS-POW-{number:03d}"
            )


_SHARD_FIXTURE_TERM_RE = re.compile(
    r"\bSF(?:Main|Acc)[A-Za-z0-9_]*\b"
)
_SHARD_RELATION_CALL_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?:[a-z][A-Za-z0-9_]*)\("
)


@dataclass(frozen=True)
class PinProjectionQuery:
    query_line: str
    expectation_line: str
    facts: tuple[str, ...]


@dataclass(frozen=True)
class RenderedPinShard:
    name: str
    text: str
    query_count: int
    fixture_fact_count: int
    relation_call_count: int
    projection_utf8_bytes: int
    utf8_bytes: int
    fixture_facts_sha256: str
    query_stream_sha256: str
    projection_sha256: str
    partition_strategy: str


def _canonical_pin_facts(text: str) -> tuple[str, ...]:
    facts = tuple(
        line
        for line in text.splitlines()
        if line
        and not line.startswith(("#", ":", "? "))
    )
    if len(facts) != len(set(facts)):
        raise RuntimeError("canonical state-form pins contain duplicate facts")
    return facts


def _canonical_pin_query_pairs(text: str) -> tuple[tuple[str, str], ...]:
    lines = text.splitlines()
    pairs: list[tuple[str, str]] = []
    for index, line in enumerate(lines):
        if not line.startswith("? "):
            continue
        if index + 1 >= len(lines):
            raise RuntimeError("state-form query lacks an expected verdict")
        expectation = lines[index + 1]
        if expectation not in ("# => TRUE", "# => FALSE"):
            raise RuntimeError(
                f"state-form query verdict drifted after {line!r}"
            )
        pairs.append((line, expectation))
    return tuple(pairs)


def _pin_query_stream_sha256(text: str) -> str:
    return _pin_pairs_sha256(_canonical_pin_query_pairs(text))


def _pin_pairs_sha256(
    pairs: Sequence[tuple[str, str]],
) -> str:
    serialized = "".join(
        f"{query}\n{expectation}\n"
        for query, expectation in pairs
    )
    return _sha256_text(serialized)


def _canonical_pin_query_blocks(
    text: str,
) -> tuple[PinProjectionQuery, ...]:
    facts = _canonical_pin_facts(text)
    fact_tokens = tuple(
        frozenset(_SHARD_FIXTURE_TERM_RE.findall(fact))
        for fact in facts
    )
    token_to_fact_indices: dict[str, list[int]] = {}
    for index, tokens in enumerate(fact_tokens):
        if not tokens:
            raise RuntimeError(
                f"state-form fact has no isolated fixture term: {facts[index]}"
            )
        for token in tokens:
            token_to_fact_indices.setdefault(token, []).append(index)

    blocks: list[PinProjectionQuery] = []
    used_fact_indices: set[int] = set()
    for query, expectation in _canonical_pin_query_pairs(text):
        pending = list(_SHARD_FIXTURE_TERM_RE.findall(query))
        if not pending:
            if query != "? prisoner(Ruk).":
                raise RuntimeError(
                    "state-form query has no isolated fixture term: "
                    f"{query}"
                )
            blocks.append(
                PinProjectionQuery(query, expectation, ())
            )
            continue
        known_tokens: set[str] = set()
        selected: set[int] = set()
        while pending:
            token = pending.pop()
            if token in known_tokens:
                continue
            known_tokens.add(token)
            for fact_index in token_to_fact_indices.get(token, ()):
                if fact_index in selected:
                    continue
                selected.add(fact_index)
                for linked in fact_tokens[fact_index]:
                    if linked not in known_tokens:
                        pending.append(linked)
        if not selected:
            raise RuntimeError(
                f"state-form query has no fixture fact closure: {query}"
            )
        used_fact_indices.update(selected)
        blocks.append(
            PinProjectionQuery(
                query,
                expectation,
                tuple(facts[index] for index in sorted(selected)),
            )
        )
    if used_fact_indices != set(range(len(facts))):
        unused = [
            facts[index]
            for index in sorted(set(range(len(facts))) - used_fact_indices)
        ]
        raise RuntimeError(
            "canonical state-form facts are not exercised by a query: "
            f"{unused[:3]!r}"
        )
    return tuple(blocks)


def _balanced_pin_slices(
    total: int,
    shard_count: int,
) -> tuple[tuple[int, int], ...]:
    if shard_count <= 0 or total < shard_count:
        raise RuntimeError(
            f"cannot divide {total} queries into {shard_count} shards"
        )
    quotient, remainder = divmod(total, shard_count)
    slices: list[tuple[int, int]] = []
    start = 0
    for index in range(shard_count):
        size = quotient + (1 if index < remainder else 0)
        slices.append((start, start + size))
        start += size
    if start != total:
        raise RuntimeError("state-form shard slice census drifted")
    return tuple(slices)


def _render_pin_projection(
    blocks: Sequence[PinProjectionQuery],
) -> tuple[str, tuple[str, ...], tuple[tuple[str, str], ...]]:
    if not blocks:
        raise RuntimeError("state-form shard projection must not be empty")
    lines: list[str] = []
    emitted_fact_set: set[str] = set()
    emitted_facts: list[str] = []
    pairs: list[tuple[str, str]] = []
    for block in blocks:
        for fact in block.facts:
            if fact in emitted_fact_set:
                continue
            emitted_fact_set.add(fact)
            emitted_facts.append(fact)
            lines.append(fact)
        lines.append(block.query_line)
        lines.append(block.expectation_line)
        lines.append("")
        pairs.append((block.query_line, block.expectation_line))
    return (
        "\n".join(lines).rstrip() + "\n",
        tuple(emitted_facts),
        tuple(pairs),
    )


def _projection_utf8_bytes(
    blocks: Sequence[PinProjectionQuery],
) -> int:
    projection, _, _ = _render_pin_projection(blocks)
    return len(projection.encode("utf-8"))


def _greedy_byte_slices(
    blocks: Sequence[PinProjectionQuery],
    capacity: int,
) -> list[tuple[int, int]]:
    slices: list[tuple[int, int]] = []
    start = 0
    while start < len(blocks):
        end = start
        emitted_facts: set[str] = set()
        current_bytes = 0
        while end < len(blocks):
            block = blocks[end]
            new_facts = tuple(
                fact for fact in block.facts if fact not in emitted_facts
            )
            increment = sum(
                len(f"{fact}\n".encode("utf-8")) for fact in new_facts
            )
            increment += len(f"{block.query_line}\n".encode("utf-8"))
            increment += len(
                f"{block.expectation_line}\n".encode("utf-8")
            )
            if end > start:
                increment += 1  # the exact blank line between query blocks
            if current_bytes + increment > capacity:
                break
            current_bytes += increment
            emitted_facts.update(new_facts)
            end += 1
        if end == start:
            raise RuntimeError(
                "state-form byte capacity cannot hold one query block"
            )
        slices.append((start, end))
        start = end
    return slices


def _byte_balanced_pin_slices(
    blocks: Sequence[PinProjectionQuery],
    shard_count: int,
) -> tuple[tuple[int, int], ...]:
    total = len(blocks)
    if shard_count <= 0 or total < shard_count:
        raise RuntimeError(
            f"cannot divide {total} queries into {shard_count} shards"
        )

    lower = max(
        _projection_utf8_bytes((block,)) for block in blocks
    )
    upper = _projection_utf8_bytes(blocks)
    while lower < upper:
        candidate = (lower + upper) // 2
        if len(_greedy_byte_slices(blocks, candidate)) <= shard_count:
            upper = candidate
        else:
            lower = candidate + 1
    capacity = lower
    slices = _greedy_byte_slices(blocks, capacity)

    # A capacity feasible with fewer than the requested number of shards is
    # also feasible with exactly that many: splitting a contiguous projection
    # cannot make either child larger than its parent. Split the largest
    # remaining projection at its best byte-balanced boundary.
    while len(slices) < shard_count:
        splittable = [
            (index, start, end)
            for index, (start, end) in enumerate(slices)
            if end - start > 1
        ]
        if not splittable:
            raise RuntimeError(
                "state-form byte partition cannot reach shard census"
            )
        selected_index, start, end = max(
            splittable,
            key=lambda item: (
                _projection_utf8_bytes(blocks[item[1]:item[2]]),
                item[2] - item[1],
                -item[1],
            ),
        )
        midpoint = min(
            range(start + 1, end),
            key=lambda split: (
                max(
                    _projection_utf8_bytes(blocks[start:split]),
                    _projection_utf8_bytes(blocks[split:end]),
                ),
                abs(
                    _projection_utf8_bytes(blocks[start:split])
                    - _projection_utf8_bytes(blocks[split:end])
                ),
                split,
            ),
        )
        slices[selected_index:selected_index + 1] = [
            (start, midpoint),
            (midpoint, end),
        ]

    result = tuple(slices)
    if result[0][0] != 0 or result[-1][1] != total:
        raise RuntimeError("state-form byte shard endpoints drifted")
    if any(
        left_end != right_start
        for (_, left_end), (right_start, _) in zip(
            result,
            result[1:],
            strict=False,
        )
    ):
        raise RuntimeError("state-form byte shard contiguity drifted")
    if max(
        _projection_utf8_bytes(blocks[start:end])
        for start, end in result
    ) > capacity:
        raise RuntimeError("state-form byte shard capacity drifted")
    return result


def _pin_slices(
    blocks: Sequence[PinProjectionQuery],
    shard_count: int,
    partition_mode: str,
) -> tuple[tuple[int, int], ...]:
    if partition_mode == "bytes":
        return _byte_balanced_pin_slices(blocks, shard_count)
    if partition_mode == "count":
        return _balanced_pin_slices(len(blocks), shard_count)
    raise RuntimeError(
        f"unknown state-form shard partition mode: {partition_mode!r}"
    )


def _render_pin_shards(
    canonical: str,
    *,
    family: str,
    shard_count: int,
    allow_prisoner: bool,
    partition_mode: str,
) -> tuple[RenderedPinShard, ...]:
    blocks = _canonical_pin_query_blocks(canonical)
    canonical_pairs = _canonical_pin_query_pairs(canonical)
    canonical_facts = set(_canonical_pin_facts(canonical))
    aggregate_sha256 = _sha256_text(canonical)
    stream_sha256 = _pin_query_stream_sha256(canonical)
    shards: list[RenderedPinShard] = []
    projected_pairs: list[tuple[str, str]] = []
    projected_facts: set[str] = set()
    for index, (start, end) in enumerate(
        _pin_slices(blocks, shard_count, partition_mode),
        1,
    ):
        selected = blocks[start:end]
        projection, emitted_facts, selected_pairs = (
            _render_pin_projection(selected)
        )
        header = (
            f"# State-form {family} execution shard "
            f"{index:02d} of {shard_count:02d}"
        )
        lines = [
            "# SPDX-License-Identifier: MIT OR Apache-2.0",
            header,
            "#",
            "# Ephemeral lossless projection of the canonical aggregate pins.",
            f"# Canonical aggregate SHA-256: {aggregate_sha256}",
            f"# Canonical query-stream SHA-256: {stream_sha256}",
            f"# Partition strategy: {partition_mode}",
            f":expect-pins {len(selected)}",
            "",
        ]
        lines.extend(projection.splitlines())
        rendered = "\n".join(lines).rstrip() + "\n"
        _validate_pin_surface(
            rendered,
            header=header,
            expected_count=len(selected),
            allow_prisoner=allow_prisoner,
        )
        shards.append(
            RenderedPinShard(
                f"{family}-{index:02d}.pins.nibli",
                rendered,
                len(selected),
                len(emitted_facts),
                len(_SHARD_RELATION_CALL_RE.findall(projection)),
                len(projection.encode("utf-8")),
                len(rendered.encode("utf-8")),
                _sha256_text(
                    "".join(f"{fact}\n" for fact in emitted_facts)
                ),
                _pin_pairs_sha256(selected_pairs),
                _sha256_text(projection),
                partition_mode,
            )
        )
        projected_facts.update(emitted_facts)
        projected_pairs.extend(selected_pairs)
    if tuple(projected_pairs) != canonical_pairs:
        raise RuntimeError(
            f"state-form {family} shard query stream is not lossless"
        )
    if projected_facts != canonical_facts:
        raise RuntimeError(
            f"state-form {family} shard fact union is not lossless"
        )
    return tuple(shards)


def render_state_form_shard_bundle(
    partition_mode: str = DEFAULT_SHARD_PARTITION,
) -> tuple[RenderedPinShard, ...]:
    main = render_state_form_pins()
    counterfactual = render_state_form_counterfactual_pins()
    shards = (
        *_render_pin_shards(
            main,
            family="main",
            shard_count=MAIN_SHARD_COUNT,
            allow_prisoner=False,
            partition_mode=partition_mode,
        ),
        *_render_pin_shards(
            counterfactual,
            family="counterfactual",
            shard_count=COUNTERFACTUAL_SHARD_COUNT,
            allow_prisoner=False,
            partition_mode=partition_mode,
        ),
    )
    expected_names = (
        *(f"main-{index:02d}.pins.nibli"
          for index in range(1, MAIN_SHARD_COUNT + 1)),
        *(f"counterfactual-{index:02d}.pins.nibli"
          for index in range(1, COUNTERFACTUAL_SHARD_COUNT + 1)),
    )
    if tuple(shard.name for shard in shards) != expected_names:
        raise RuntimeError("state-form shard path inventory drifted")
    return tuple(shards)


def render_state_form_shard_index(
    shards: Sequence[RenderedPinShard],
) -> str:
    main = render_state_form_pins()
    counterfactual = render_state_form_counterfactual_pins()
    partition_strategies = {
        shard.partition_strategy for shard in shards
    }
    if len(partition_strategies) != 1:
        raise RuntimeError("state-form shard partition strategy drifted")
    partition_strategy = partition_strategies.pop()
    payload = {
        "schema_version": "state-form-pin-shards-v2",
        "partition": {
            "strategy": partition_strategy,
            "contiguous_query_blocks": True,
            "byte_basis": (
                "exact rendered UTF-8 query projection with transitive "
                "fixture closure and per-shard fact deduplication"
            ),
            "main_shard_count": MAIN_SHARD_COUNT,
            "counterfactual_shard_count": COUNTERFACTUAL_SHARD_COUNT,
        },
        "canonical": {
            "main": {
                "path": str(STATE_FORM_PINS.relative_to(ROOT)),
                "sha256": EXPECTED_MAIN_PINS_SHA256,
                "query_count": EXPECTED_MAIN_PIN_COUNT,
                "fixture_fact_count": len(_canonical_pin_facts(main)),
                "relation_call_count": len(
                    _SHARD_RELATION_CALL_RE.findall(main)
                ),
                "utf8_bytes": len(main.encode("utf-8")),
                "query_stream_sha256": _pin_query_stream_sha256(main),
            },
            "counterfactual": {
                "path": str(
                    STATE_FORM_COUNTERFACTUAL_PINS.relative_to(ROOT)
                ),
                "sha256": EXPECTED_COUNTERFACTUAL_PINS_SHA256,
                "query_count": EXPECTED_COUNTERFACTUAL_PIN_COUNT,
                "fixture_fact_count": len(
                    _canonical_pin_facts(counterfactual)
                ),
                "relation_call_count": len(
                    _SHARD_RELATION_CALL_RE.findall(counterfactual)
                ),
                "utf8_bytes": len(counterfactual.encode("utf-8")),
                "query_stream_sha256": (
                    _pin_query_stream_sha256(counterfactual)
                ),
            },
        },
        "shards": [
            {
                "path": shard.name,
                "query_count": shard.query_count,
                "fixture_fact_count": shard.fixture_fact_count,
                "relation_call_count": shard.relation_call_count,
                "projection_utf8_bytes": shard.projection_utf8_bytes,
                "utf8_bytes": shard.utf8_bytes,
                "fixture_facts_sha256": shard.fixture_facts_sha256,
                "query_stream_sha256": shard.query_stream_sha256,
                "projection_sha256": shard.projection_sha256,
                "sha256": _sha256_text(shard.text),
            }
            for shard in shards
        ],
    }
    return json.dumps(
        payload,
        ensure_ascii=True,
        indent=2,
        sort_keys=True,
    ) + "\n"


def _validate_counterfactual_shape(
    source: str,
    counterfactual: str,
) -> None:
    source_rule = current_rule()
    mutated_rule = source_rule.replace(CURRENT_REVIEW_GUARD, "", 1)
    if source.count(source_rule) != 1:
        raise RuntimeError("source current rule occurrence drifted")
    if counterfactual.count(mutated_rule) != 1:
        raise RuntimeError("counterfactual current rule occurrence drifted")
    if counterfactual.count(source_rule) != 0:
        raise RuntimeError("counterfactual retained the source current rule")
    source_lines = source.splitlines()
    counterfactual_lines = counterfactual.splitlines()
    if len(source_lines) != len(counterfactual_lines):
        raise RuntimeError("counterfactual changed constitution line count")
    differences = [
        (old, new)
        for old, new in zip(source_lines, counterfactual_lines, strict=True)
        if old != new
    ]
    if differences != [(source_rule, mutated_rule)]:
        raise RuntimeError(
            "counterfactual differs outside the one current-review guard"
        )


def _validate_artifact_bytes(
    path: pathlib.Path,
    expected: str,
    expected_sha256: str,
) -> None:
    if not path.is_file():
        raise RuntimeError(f"missing state-form artifact: {path}")
    actual = path.read_text(encoding="utf-8")
    if actual != expected:
        raise RuntimeError(
            f"state-form artifact differs from checker-owned bytes: {path}"
        )
    _validate_expected_sha256(str(path), expected_sha256)
    actual_sha256 = _sha256_text(actual)
    if actual_sha256 != expected_sha256:
        raise RuntimeError(
            f"state-form artifact SHA-256 drifted for {path}: "
            f"expected {expected_sha256}, found {actual_sha256}"
        )


def write_state_form_artifacts() -> None:
    rendered = (
        (STATE_FORM_PINS, render_state_form_pins()),
        (STATE_FORM_COUNTERFACTUAL, render_state_form_counterfactual()),
        (
            STATE_FORM_COUNTERFACTUAL_PINS,
            render_state_form_counterfactual_pins(),
        ),
    )
    for path, text in rendered:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        print(f"wrote {path.relative_to(ROOT)}")


def write_state_form_shards(
    output_dir: pathlib.Path,
    partition_mode: str = DEFAULT_SHARD_PARTITION,
) -> None:
    check()
    shards = render_state_form_shard_bundle(partition_mode)
    index_text = render_state_form_shard_index(shards)
    if output_dir.exists() and not output_dir.is_dir():
        raise RuntimeError(
            f"state-form shard output is not a directory: {output_dir}"
        )
    output_dir.mkdir(parents=True, exist_ok=True)
    existing = tuple(output_dir.iterdir())
    if existing:
        raise RuntimeError(
            "state-form shard output directory must be empty: "
            f"{output_dir}"
        )
    for shard in shards:
        path = output_dir / shard.name
        path.write_text(shard.text, encoding="utf-8")
        if path.read_text(encoding="utf-8") != shard.text:
            raise RuntimeError(f"state-form shard write drifted: {path}")
    index_path = output_dir / "index.json"
    index_path.write_text(index_text, encoding="utf-8")
    if index_path.read_text(encoding="utf-8") != index_text:
        raise RuntimeError(
            f"state-form shard index write drifted: {index_path}"
        )
    print(
        "wrote state-form execution bundle: "
        f"{MAIN_SHARD_COUNT} main shards, "
        f"{COUNTERFACTUAL_SHARD_COUNT} counterfactual shards, "
        f"{EXPECTED_MAIN_PIN_COUNT + EXPECTED_COUNTERFACTUAL_PIN_COUNT} "
        "lossless queries"
    )


def validate_state_form_artifacts() -> None:
    main_pins = render_state_form_pins()
    counterfactual = render_state_form_counterfactual()
    counterfactual_pins = render_state_form_counterfactual_pins()
    _validate_pin_surface(
        main_pins,
        header=STATE_FORM_PINS_HEADER,
        expected_count=EXPECTED_MAIN_PIN_COUNT,
        allow_prisoner=False,
    )
    _validate_main_pin_manifest(main_pins)
    _validate_pin_surface(
        counterfactual_pins,
        header=STATE_FORM_COUNTERFACTUAL_PINS_HEADER,
        expected_count=EXPECTED_COUNTERFACTUAL_PIN_COUNT,
        allow_prisoner=False,
    )
    _validate_counterfactual_pin_manifest(counterfactual_pins)
    source = CONSTITUTION.read_text(encoding="utf-8")
    _validate_counterfactual_shape(source, counterfactual)
    _validate_artifact_bytes(
        STATE_FORM_PINS,
        main_pins,
        EXPECTED_MAIN_PINS_SHA256,
    )
    _validate_artifact_bytes(
        STATE_FORM_COUNTERFACTUAL,
        counterfactual,
        EXPECTED_COUNTERFACTUAL_SHA256,
    )
    _validate_artifact_bytes(
        STATE_FORM_COUNTERFACTUAL_PINS,
        counterfactual_pins,
        EXPECTED_COUNTERFACTUAL_PINS_SHA256,
    )
    shards = render_state_form_shard_bundle()
    if sum(
        shard.query_count
        for shard in shards
        if shard.name.startswith("main-")
    ) != EXPECTED_MAIN_PIN_COUNT:
        raise RuntimeError("state-form main shard count drifted")
    if sum(
        shard.query_count
        for shard in shards
        if shard.name.startswith("counterfactual-")
    ) != EXPECTED_COUNTERFACTUAL_PIN_COUNT:
        raise RuntimeError("state-form counterfactual shard count drifted")
    json.loads(render_state_form_shard_index(shards))

def check() -> None:
    statements = validate_draft_rules()
    validate_delegation_markers()
    constitution_text = CONSTITUTION.read_text(encoding="utf-8")
    constitution_sha256 = _sha256_text(constitution_text)
    if constitution_sha256 != EXPECTED_CONSTITUTION_SHA256:
        raise RuntimeError(
            "constitution SHA-256 drifted for state-form artifacts: "
            f"expected {EXPECTED_CONSTITUTION_SHA256}, found {constitution_sha256}"
        )
    if not RENDERER_UNLOCKED:
        if BEGIN in constitution_text or END in constitution_text:
            raise RuntimeError(
                "locked state-form renderer found an unexpected constitution block"
            )
        print(
            "state-form: PASS (renderer locked) — "
            f"{len(CARDS)} cards, {len(ALL_BRANCHES)} result declarations, "
            f"{EXPECTED_AUTHORITY_COUNT} authority heads, "
            f"{len(statements)} exact statements, "
            f"IR {EXPECTED_BRANCH_IR_SHA256}, rules {EXPECTED_RULE_BLOCK_SHA256}"
        )
        return
    actual = extract_block(constitution_text)
    if actual != rendered_block():
        raise RuntimeError(
            "constitution state-form block differs from checker-owned exact block"
        )
    validate_state_form_artifacts()
    print(
        f"state-form: PASS — {len(CARDS)} cards, "
        f"{len(statements)} exact statements, "
        f"{EXPECTED_MAIN_PIN_COUNT} main pins, "
        f"{EXPECTED_COUNTERFACTUAL_PIN_COUNT} counterfactual pins"
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--print-block", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--write-artifacts", action="store_true")
    parser.add_argument(
        "--write-shards",
        type=pathlib.Path,
        metavar="DIRECTORY",
    )
    parser.add_argument(
        "--shard-partition",
        choices=SHARD_PARTITION_MODES,
        default=DEFAULT_SHARD_PARTITION,
        help=(
            "partition execution shards by exact projection bytes "
            "(default) or by query count for benchmark fallback"
        ),
    )
    args = parser.parse_args(argv)
    selected = sum(
        (
            args.print_block,
            args.check,
            args.write_artifacts,
            args.write_shards is not None,
        )
    )
    if selected != 1:
        parser.error(
            "choose exactly one of --print-block, --check, "
            "--write-artifacts, or --write-shards DIRECTORY"
        )
    if args.print_block:
        sys.stdout.write(rendered_block())
        return 0
    try:
        with VerificationLock(
            "verify",
            source_digest=hashlib.sha256(
                CONSTITUTION.read_bytes()
            ).hexdigest(),
            root=ROOT,
        ):
            if args.check:
                check()
            elif args.write_artifacts:
                write_state_form_artifacts()
            else:
                write_state_form_shards(
                    args.write_shards,
                    partition_mode=args.shard_partition,
                )
    except VerificationLockBusy as exc:
        print(f"state-form verification lock: {exc}", file=sys.stderr)
        return EX_TEMPFAIL
    except VerificationLockError as exc:
        print(f"state-form verification lock: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
