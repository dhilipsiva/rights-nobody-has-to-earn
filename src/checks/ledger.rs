// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native full-society ledger validator and report generator.
//!
//! The reviewed ledger is deliberately parsed into typed contracts. Dynamic
//! JSON is used only by the duplicate-key preflight and at canonical digest
//! boundaries; semantic validation never indexes a `serde_json::Value`.

pub(crate) mod closure;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

#[cfg(test)]
use std::path::PathBuf;

use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::Error;
use crate::context::Context;
use crate::digest::{canonical_json, sha256};
use crate::receipt::{self, ValidationOptions};
use crate::refresh::{ImmutableRepositoryInputs, atomic_refresh_and_check};

use super::reader;

pub(crate) const STEP_NAME: &str = "full-society domain-and-layer ledger";
const SOURCE: &str = "new-book-plans/full-society-ledger.json";
const OUTPUT: &str = "new-book-plans/full-society-ledger.md";
const READER_OUTPUT: &str = "new-book-plans/full-society-reader-ledger.md";
const COVERAGE_MAP: &str = "new-book-plans/book-1-constitutional-coverage-map.md";
const PROTOCOL_DOC: &str = "new-book-plans/full-society-scope-review-protocol.md";
const POWER_MANIFEST: &str = "new-book-plans/full-society-power-source-manifest.json";
const READER_SOURCE: &str = "new-book-plans/reader-evidence.json";
const READER_PROTOCOL_DECISION: &str = "new-book-plans/book-1-reader-evidence-protocol-decision.md";
const ASSERTION_SOURCE: &str = "new-book-plans/assertion-surface-contracts.json";
const ASSURANCE_SOURCE: &str = "new-book-plans/record-integrity-assurance-case.json";
const RED_TEAM_SOURCE: &str = "new-book-plans/record-integrity-red-team.json";
const AMENDMENT_SOURCE: &str = "new-book-plans/amendment-semantics-audit.json";
const PLACEMENT_SOURCE: &str = "new-book-plans/placement-exhaustiveness-audit.json";
const TEMPORAL_SOURCE: &str = "new-book-plans/temporal-assurance-case.json";
const ASSURANCE_DECISION: &str = "new-book-plans/book-1-assurance-portfolio-decision.md";
const BOUNDARY_DECISION: &str = "new-book-plans/full-society-boundary-decision.md";
const ECONOMIC_DECISION: &str =
    "new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md";

const STATIC_INPUTS: [&str; 15] = [
    SOURCE,
    ASSURANCE_DECISION,
    BOUNDARY_DECISION,
    POWER_MANIFEST,
    PROTOCOL_DOC,
    ASSERTION_SOURCE,
    ASSURANCE_SOURCE,
    RED_TEAM_SOURCE,
    AMENDMENT_SOURCE,
    PLACEMENT_SOURCE,
    TEMPORAL_SOURCE,
    READER_SOURCE,
    READER_PROTOCOL_DECISION,
    COVERAGE_MAP,
    ECONOMIC_DECISION,
];

const EXPECTED_SCHEMA_VERSION: u64 = 7;
const EXPECTED_STATUS: &str = "stage_4_repository_audit_complete";
const STAGE_LABEL: &str = "stage 4 machinery";
const STRUCTURAL_CONTROL_COUNT: usize = 274;
const EXPECTED_POWER_COUNT: usize = 210;
const EXPECTED_EFFECT_COUNT: usize = 367;
const EXPECTED_TEMPLATE_COUNT: usize = 1;
const EXPECTED_REFUSAL_COUNT: usize = 19;
const EXPECTED_CROSSWALK_COUNT: usize = 8;
const EXPECTED_ALLOCATION_COUNT: usize = 210;

const ECONOMIC_EFFECT_FIRST: usize = 223;
const ECONOMIC_EFFECT_TERM_PLACEMENTS: usize = 3_268;
const ECONOMIC_EFFECT_TERM_SCHEMA_KEYS: usize = 113;
const ECONOMIC_EFFECT_TERM_SCHEMA_SHA256: &str =
    "857219448f37f2e2862492a0c11b2d37397ba2b615c34fd41a05dc96ec959c39";
const ECONOMIC_EFFECT_COMPLETION_CEILING: &str = "All 145 person-held effects, their separately typed always-on duties, the three non-power carry result interfaces, executable tests, counterfactuals, and approved Book 1 prose are complete. They establish only source-bound legal effects over supplied records; no operation, calculation, delivery, remedy, institutional act, liveness, calibration, external truth, or feasibility is proved.";

const DEMOCRATIC_POLICY_BOUNDARY_TERM_KEYS: [&str; 11] = [
    "democratic_source",
    "policy_choice",
    "competence",
    "corridor",
    "equality_boundary",
    "floor_boundary",
    "commons_boundary",
    "public_reasons",
    "review",
    "temporal_status",
    "failure_default",
];

const ECONOMIC_COMMON_POWER_FIELDS: [(&str, &str, &str); 12] = [
    ("case", "$case", "EconomicCaseScope"),
    ("subject", "$subject", "EconomicSubjectScope"),
    ("function", "$function", "EconomicFunctionScope"),
    ("affected", "$affected", "EconomicAffectedPeopleScope"),
    (
        "alternate_record_reviewer",
        "$alternate_record_reviewer",
        "EconomicAlternateRecordReviewActorScope",
    ),
    (
        "alternate_temporal_reviewer",
        "$alternate_temporal_reviewer",
        "EconomicAlternateTemporalReviewActorScope",
    ),
    (
        "alternate_independent_reviewer",
        "$alternate_independent_reviewer",
        "EconomicAlternateIndependentReviewActorScope",
    ),
    (
        "alternate_audit_reviewer",
        "$alternate_audit_reviewer",
        "EconomicAlternateAuditActorScope",
    ),
    (
        "alternate_final_reviewer",
        "$alternate_final_reviewer",
        "EconomicAlternateFinalReviewActorScope",
    ),
    (
        "independent_reviewer",
        "$review",
        "EconomicIndependentReviewActorScope",
    ),
    ("audit_reviewer", "$auditor", "EconomicAuditActorScope"),
    (
        "final_reviewer",
        "$final_review",
        "EconomicFinalReviewActorScope",
    ),
];

const ECONOMIC_COMMON_POWER_REQUIREMENTS: [(&str, &str); 12] = [
    (
        "EconomicCardEvidenceAuthenticatedContestableAndPurposeBound",
        "EvidenceRuleScope",
    ),
    (
        "EconomicEffectNecessaryAndProportionateToItsSource",
        "NecessityAndProportionalityScope",
    ),
    (
        "EconomicPublicReasonsConnectSourceFactsHolderAndEffect",
        "PublicReasonsRequirementScope",
    ),
    (
        "EconomicDelegationCannotEnlargeEffectOrEvadeWall",
        "NonDelegableLimitScope",
    ),
    (
        "EconomicConflictedHolderWithdrawsForSourceAuthorizedAlternate",
        "ConflictRuleScope",
    ),
    (
        "EconomicIndependentChallengeKeepsInterimProtection",
        "ChallengeRequirementScope",
    ),
    (
        "EconomicCorrectionReconcilesConsequentialRecordsWithoutReplay",
        "CorrectionRequirementScope",
    ),
    (
        "EconomicUnlawfulEffectStopsAndAttributableHarmRemainsRemediable",
        "RemedyRequirementScope",
    ),
    (
        "EconomicProtectedContinuitySurvivesMissingOrDisputedAuthority",
        "ContinuityRequirementScope",
    ),
    (
        "EconomicFailureWithholdsAuthorityAndCreatesNoOppositeFact",
        "EconomicCardFailurePolarityScope",
    ),
    (
        "EconomicUnavailableReviewerTransfersToPredeclaredAlternate",
        "AlternateReviewContinuityScope",
    ),
    (
        "EconomicAlternateReviewCannotApproveOrExtendBySilence",
        "AlternateReviewLimitScope",
    ),
];

const ECONOMIC_CURRENT_COLLISION_SCOPES: [&str; 17] = [
    "SourceFamilyScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "PowerScope",
    "TemporalContractKindScope",
    "EffectiveSelectionScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "AuthorityScope",
    "AuthorityScopeKindScope",
    "EndConditionScope",
    "ReconciliationRecordScope",
    "TemporalRecordScope",
    "TemporalAuthorityActorScope",
    "TemporalReviewActorScope",
    "ResultScope",
];

const ECONOMIC_RECONCILIATION_COLLISION_SCOPES: [&str; 13] = [
    "ReconciliationStatusScope",
    "EconomicRecordScope",
    "ResultScope",
    "PowerScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "TemporalRecordScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "AuthorityScope",
    "AuthorityScopeKindScope",
    "EndConditionScope",
];

const ECONOMIC_RESULT_COLLISION_SCOPES: [&str; 23] = [
    "EconomicBranchScope",
    "HolderScope",
    "ChallengeScope",
    "CorrectionScope",
    "RemedyScope",
    "EndConditionScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "TemporalRecordScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "AuthorityScope",
    "AuthorityScopeKindScope",
    "ReconciliationRecordScope",
    "ReviewDispositionScope",
    "FailurePolarityScope",
    "EconomicSourceActorScope",
    "EconomicEvidenceActorScope",
    "EconomicIndependentReviewActorScope",
    "EconomicAuditActorScope",
    "EconomicFinalReviewActorScope",
    "EconomicExecutionActorScope",
];

const ECONOMIC_CARRY_CURRENT_COLLISION_SCOPES: [&str; 28] = [
    "SourceFamilyScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "PowerScope",
    "TemporalContractKindScope",
    "EffectiveSelectionScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "AuthorityScope",
    "AuthorityScopeKindScope",
    "EndConditionScope",
    "ReconciliationRecordScope",
    "TemporalRecordScope",
    "TemporalAuthorityActorScope",
    "TemporalReviewActorScope",
    "ResultScope",
    "PriorSourceEpochScope",
    "EconomicCarryKindScope",
    "EconomicCarryPredecessorRecordScope",
    "EconomicCarryPredecessorResultScope",
    "EconomicCarrySuccessorEventScope",
    "EconomicCarryLegalScope",
    "EconomicCarryLegalScopeKindScope",
    "EconomicSubjectScope",
    "EconomicBenefitScope",
    "EconomicTitleScope",
    "EconomicLiabilityScope",
];

const ECONOMIC_CARRY_RESULT_COLLISION_SCOPES: [&str; 37] = [
    "EconomicBranchScope",
    "HolderScope",
    "ChallengeScope",
    "CorrectionScope",
    "RemedyScope",
    "EndConditionScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "TemporalRecordScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "AuthorityScope",
    "AuthorityScopeKindScope",
    "ReconciliationRecordScope",
    "ReviewDispositionScope",
    "FailurePolarityScope",
    "EconomicSourceActorScope",
    "EconomicEvidenceActorScope",
    "EconomicIndependentReviewActorScope",
    "EconomicAuditActorScope",
    "EconomicFinalReviewActorScope",
    "EconomicExecutionActorScope",
    "PriorSourceEpochScope",
    "EconomicCarryKindScope",
    "EconomicCarryPredecessorRecordScope",
    "EconomicCarryPredecessorResultScope",
    "EconomicCarrySuccessorEventScope",
    "EconomicCarryFindingKindScope",
    "EconomicCarryRequirementScope",
    "EconomicCarryEffectLimitScope",
    "EconomicCarryLegalScope",
    "EconomicCarryLegalScopeKindScope",
    "EconomicSubjectScope",
    "EconomicBenefitScope",
    "EconomicTitleScope",
    "EconomicLiabilityScope",
];

const ECONOMIC_CARRY_RECONCILIATION_COLLISION_SCOPES: [&str; 21] = [
    "ReconciliationStatusScope",
    "EconomicRecordScope",
    "ResultScope",
    "EconomicCarryKindScope",
    "EconomicCarryPredecessorRecordScope",
    "EconomicCarryPredecessorResultScope",
    "EconomicCarrySuccessorEventScope",
    "SourceVersionScope",
    "SourceEpochScope",
    "PriorSourceEpochScope",
    "TemporalRecordScope",
    "EconomicCaseScope",
    "JurisdictionScope",
    "JurisdictionKindScope",
    "EconomicCarryLegalScope",
    "EconomicCarryLegalScopeKindScope",
    "EndConditionScope",
    "EconomicSubjectScope",
    "EconomicBenefitScope",
    "EconomicTitleScope",
    "EconomicLiabilityScope",
];

const ECONOMIC_ALTERNATE_REVIEW_ROUTES: [(&str, &str, &str, &str); 5] = [
    (
        "record-review",
        "$power_record_review",
        "$power_alternate_record_reviewer",
        "EconomicAlternateRecordReviewActorScope",
    ),
    (
        "temporal-review",
        "$power_temporal_review",
        "$power_alternate_temporal_reviewer",
        "EconomicAlternateTemporalReviewActorScope",
    ),
    (
        "independent-review",
        "$power_review",
        "$power_alternate_independent_reviewer",
        "EconomicAlternateIndependentReviewActorScope",
    ),
    (
        "audit-review",
        "$power_auditor",
        "$power_alternate_audit_reviewer",
        "EconomicAlternateAuditActorScope",
    ),
    (
        "final-review",
        "$power_final_review",
        "$power_alternate_final_reviewer",
        "EconomicAlternateFinalReviewActorScope",
    ),
];

#[derive(Clone, Copy)]
struct EconomicDutyBridgeSpec {
    power: usize,
    key: &'static str,
    bearer: &'static str,
    duty: &'static str,
    standard: &'static str,
    function: &'static str,
    mode: &'static str,
}

const ECONOMIC_DUTY_BRIDGES: [EconomicDutyBridgeSpec; 31] = [
    EconomicDutyBridgeSpec {
        power: 63,
        key: "knowledge-floor-access",
        bearer: "FSBOD_09",
        duty: "EstablishEffectiveKnowledgeAccessRouteDuty",
        standard: "MaterialFloorProtectingCompatibleAccessRouteStandard",
        function: "$protected_duty",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 63,
        key: "knowledge-commons-access",
        bearer: "FSBOD_09",
        duty: "EstablishEffectiveKnowledgeCommonsAccessRouteDuty",
        standard: "ProtectedCommonsCompatibleAccessRouteStandard",
        function: "$protected_duty",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-fair-access",
        bearer: "$subject",
        duty: "ProvidePublicScaleFunctionFairAccessDuty",
        standard: "AffectedFunctionFairAccessStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-continuity",
        bearer: "$subject",
        duty: "MaintainPublicScaleFunctionContinuityDuty",
        standard: "AffectedFunctionContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-reasons-transparency",
        bearer: "$subject",
        duty: "ProvidePublicScaleFunctionReasonsTransparencyDuty",
        standard: "AffectedFunctionPublicReasonsAndTransparencyStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-portability-interoperability",
        bearer: "$subject",
        duty: "ProvidePublicScaleFunctionPortabilityInteroperabilityDuty",
        standard: "AffectedFunctionDataServicePortabilityInteroperabilityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-audit",
        bearer: "$subject",
        duty: "UndergoPublicScaleFunctionAuditDuty",
        standard: "AffectedFunctionIndependentAuditStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-challenge-correction",
        bearer: "$subject",
        duty: "ProvidePublicScaleFunctionChallengeCorrectionDuty",
        standard: "AffectedFunctionAccessibleChallengeCorrectionStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 64,
        key: "public-scale-remedy",
        bearer: "$subject",
        duty: "ProvidePublicScaleFunctionRemedyDuty",
        standard: "AffectedFunctionEffectiveRemedyStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 65,
        key: "remedy-transition-065",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 66,
        key: "remedy-transition-066",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 67,
        key: "remedy-transition-067",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 68,
        key: "remedy-transition-068",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 69,
        key: "remedy-transition-069",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 70,
        key: "remedy-transition-070",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 71,
        key: "remedy-transition-071",
        bearer: "FSBOD_09",
        duty: "ProtectEconomicRemedyTransitionParticipantsDuty",
        standard: "WorkersUsersFloorRecipientsAndEssentialServiceContinuityStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 72,
        key: "tax-procedure-disclosure",
        bearer: "FSBOD_02",
        duty: "ProvideTaxReasonsPrivacyAuditChallengeDuty",
        standard: "LawfulAuthorityPublicPurposeEqualityPrivacyReasonsAuditChallengeStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 73,
        key: "appropriation-transparency",
        bearer: "FSBOD_02",
        duty: "PublishAppropriationReportAuditFiscalRiskDuty",
        standard: "LegislativeAuthorizationCompleteReportingAuditFiscalRiskStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 74,
        key: "spending-transparency",
        bearer: "FSBOD_07",
        duty: "PublishPublicSpendingReportAndAuditDuty",
        standard: "ExactAppropriationCompleteReportingAuditAndNoDeliveryInferenceStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 75,
        key: "guarantee-risk-disclosure",
        bearer: "FSBOD_07",
        duty: "DiscloseAuditPublicGuaranteeRiskDuty",
        standard: "LegislativeAuthorizationCompleteRiskDisclosureAuditStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 76,
        key: "borrowing-risk-disclosure",
        bearer: "FSBOD_07",
        duty: "DiscloseAuditPublicBorrowingRiskDuty",
        standard: "LegislativeAuthorizationCompleteRiskDisclosureAuditCreditorLimitStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 77,
        key: "monetary-reasons-review",
        bearer: "FSBOD_08",
        duty: "PublishMonetaryReasonsDistributionReviewAndAuditDuty",
        standard: "MandateBoundReasonsDistributionalReviewAuditAndCauseRemovalStandard",
        function: "$function",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 78,
        key: "public-financial-decision-process",
        bearer: "FSBOD_09",
        duty: "ProvideFinancialDecisionReasonsCorrectionChallengeDuty",
        standard: "LawfulCriteriaEqualityPurposeBoundDataCorrectionAndChallengeStandard",
        function: "$function",
        mode: "PublicObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 78,
        key: "private-financial-decision-process",
        bearer: "$subject",
        duty: "ProvidePrivateFinancialDecisionReasonsCorrectionChallengeDuty",
        standard: "ExpressPrivateFunctionLawfulCriteriaEqualityDataCorrectionChallengeStandard",
        function: "$function",
        mode: "PrivateObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 81,
        key: "scarcity-reassessment",
        bearer: "FSBOD_09",
        duty: "ObtainRecordFreshScarcityReassessmentEvidenceDuty",
        standard: "SourceBoundEndIndependentReviewAndNoClockAdvanceStandard",
        function: "$resource",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 82,
        key: "scarcity-unmet-floor-record",
        bearer: "FSBOD_09",
        duty: "RecordEveryUnmetFloorPortionAsFailureDuty",
        standard: "NoRationOrMitigationCanRenameNonDeliveryAsSuccessStandard",
        function: "$resource",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 82,
        key: "scarcity-interim-repair",
        bearer: "FSBOD_09",
        duty: "ProvideInterimScarcityAlternativesAccessAndRepairDuty",
        standard: "AlternativesAccessibilityCorrectionReplenishmentAndRepairStandard",
        function: "$resource",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 83,
        key: "public-minimum-service-arrangement",
        bearer: "$provider_or_parties",
        duty: "ArrangeNarrowMinimumServiceContinuityDuty",
        standard: "ProviderOrPartiesNoNamedWorkerConscriptionAndBargainingSubstituteStandard",
        function: "$minimum_service_order",
        mode: "PublicObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 83,
        key: "private-minimum-service-arrangement",
        bearer: "$provider_or_parties",
        duty: "ArrangeExpressPrivateMinimumServiceContinuityDuty",
        standard: "ExpressPrivateProviderNoNamedWorkerConscriptionAndBargainingSubstituteStandard",
        function: "$minimum_service_order",
        mode: "PrivateObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 86,
        key: "public-unit-maintenance",
        bearer: "FSBOD_08",
        duty: "MaintainPublicUnitOfAccountDuty",
        standard: "CommonTierPublicUnitWithoutExclusiveInstrumentOrOperationClaimStandard",
        function: "$unit",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
    EconomicDutyBridgeSpec {
        power: 87,
        key: "settlement-backbone-maintenance",
        bearer: "FSBOD_08",
        duty: "MaintainAccessibleNonDigitalSettlementBackboneDuty",
        standard: "AccessibleNonDigitalFloorContinuousNoClearingOrLivenessClaimStandard",
        function: "$settlement_route",
        mode: "EconomicPowerBoundObligationBearerMode",
    },
];

#[derive(Clone, Copy)]
struct EconomicDependencySpec {
    card: usize,
    prerequisite: usize,
    label: &'static str,
    shared_fields: &'static [&'static str],
}

const ECONOMIC_DEPENDENCIES: [EconomicDependencySpec; 13] = [
    EconomicDependencySpec {
        card: 65,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 66,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 67,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 68,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 69,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 70,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 71,
        prerequisite: 64,
        label: "private_power",
        shared_fields: &["case", "subject", "function", "affected"],
    },
    EconomicDependencySpec {
        card: 71,
        prerequisite: 62,
        label: "acquisition",
        shared_fields: &[
            "case",
            "authorizing_law",
            "property",
            "public_purpose",
            "compensation",
        ],
    },
    EconomicDependencySpec {
        card: 74,
        prerequisite: 73,
        label: "appropriation",
        shared_fields: &[
            "case",
            "subject",
            "function",
            "affected",
            "appropriation",
            "tier",
            "fiscal_amount",
        ],
    },
    EconomicDependencySpec {
        card: 82,
        prerequisite: 81,
        label: "scarcity",
        shared_fields: &[
            "case",
            "subject",
            "function",
            "affected",
            "resource",
            "population",
        ],
    },
    EconomicDependencySpec {
        card: 85,
        prerequisite: 72,
        label: "tax",
        shared_fields: &["tax_instrument", "tier"],
    },
    EconomicDependencySpec {
        card: 87,
        prerequisite: 86,
        label: "unit",
        shared_fields: &["unit", "tier"],
    },
    EconomicDependencySpec {
        card: 88,
        prerequisite: 87,
        label: "backbone",
        shared_fields: &["unit", "tier", "settlement_route"],
    },
];

#[derive(Clone, Copy)]
struct StateDependencySpec {
    card: usize,
    prerequisite: usize,
    label: &'static str,
    economic_field: &'static str,
    state_field_scope: &'static str,
    branch: &'static str,
}

const STATE_DEPENDENCIES: [StateDependencySpec; 8] = [
    StateDependencySpec {
        card: 62,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "authorizing_law",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
    StateDependencySpec {
        card: 72,
        prerequisite: 6,
        label: "revenue",
        economic_field: "tax_instrument",
        state_field_scope: "RevenueMeasureScope",
        branch: "FSPOW_006RevenueAuthorizationBranch",
    },
    StateDependencySpec {
        card: 73,
        prerequisite: 7,
        label: "appropriation",
        economic_field: "appropriation",
        state_field_scope: "AppropriationMeasureScope",
        branch: "FSPOW_007AppropriationAuthorizationBranch",
    },
    StateDependencySpec {
        card: 75,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "authorizing_law",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
    StateDependencySpec {
        card: 76,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "authorizing_law",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
    StateDependencySpec {
        card: 77,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "mandate",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
    StateDependencySpec {
        card: 84,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "authorizing_law",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
    StateDependencySpec {
        card: 88,
        prerequisite: 5,
        label: "ordinary_law",
        economic_field: "authorizing_law",
        state_field_scope: "LawScope",
        branch: "FSPOW_005UnusedCouncilReturnBranch",
    },
];

const ECONOMIC_ALWAYS_DUTY_BINDINGS: [(&str, &str, &str); 25] = [
    (
        "material-floor-finance",
        "FinanceSecureMaintainMaterialFloorDuty",
        "NondelegableNoncontributoryRealAccessStandard",
    ),
    (
        "material-floor-continuity",
        "ActivateImmediatePublicFloorContinuityDuty",
        "WithdrawalDisputeInaccessibilityInadequacyOrFailureStandard",
    ),
    (
        "decent-work-opportunity",
        "SupportDecentWorkTrainingOccupationalAccessDuty",
        "GenuineOpportunityAntiExclusionNoNamedJobStandard",
    ),
    (
        "labour-safety",
        "ProvideSafeHealthyWorkConditionsDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-terms",
        "ProvideFairRemunerationRecoveryRestPredictableTermsDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-equality",
        "ProvideLabourEqualityAccommodationPrivacyInspectionDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-collective",
        "ProtectAssociationBargainingCollectiveActionDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-participation",
        "ProvideHighConsequenceWorkplaceInformationParticipationDuty",
        "CompatibleLawStructureAndThresholdStandard",
    ),
    (
        "housing-continuity",
        "ProvideEvictionForeclosureHousingContinuityRouteDuty",
        "LegalityNoticeHearingProportionalityReviewAndRealContinuityStandard",
    ),
    (
        "public-facing-service-protection",
        "ProvidePublicFacingServiceProtectionDuty",
        "AccessibleTermsSafetyCorrectionCancellationAndCollectiveRedressStandard",
    ),
    (
        "essential-public-facing-continuity",
        "MaintainEssentialPublicFacingServiceContinuityDuty",
        "EssentialServiceContinuityWithoutRightsWaiverStandard",
    ),
    (
        "private-sphere-help-exit",
        "MaintainConfidentialPrivateSphereHelpAndExitRouteDuty",
        "IndependentStandingCapacityFloorClaimsConfidentialHelpAndFreeExitStandard",
    ),
    (
        "personal-insolvency-fresh-start",
        "ProvideAccessiblePersonalInsolvencyFreshStartDuty",
        "FloorEssentialsOrdinaryToolsAndIndividualizedFraudProcessStandard",
    ),
    (
        "failure-wage-pension-protection",
        "ProtectWagesPensionsAndEarnedBenefitsOnFailureDuty",
        "EffectivePriorityGuaranteeOrEquivalentEmptyEstateStandard",
    ),
    (
        "insolvency-evasion-prevention",
        "PreventInsolvencyEvasionAndLiabilityDumpingDuty",
        "InsiderPreferenceFraudulentTransferAssetStrippingAndRemediationDutyStandard",
    ),
    (
        "failure-essential-continuity",
        "MaintainEssentialServiceContinuityThroughFailureDuty",
        "PublicContinuityWithoutAuthorityExtensionStandard",
    ),
    (
        "economic-individual-remedy",
        "ProvideEconomicIndividualRemedyDuty",
        "AuthenticatedBreachCessationProvisionRecoveryCorrectionRestitutionAndProtectionStandard",
    ),
    (
        "economic-continuity-remedy",
        "ProvideEconomicContinuityRemedyDuty",
        "AuthenticatedBreachInterimContinuityAndNonRepetitionStandard",
    ),
    (
        "labour-safety-public",
        "ProvideSafeHealthyWorkConditionsDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-terms-public",
        "ProvideFairRemunerationRecoveryRestPredictableTermsDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-equality-public",
        "ProvideLabourEqualityAccommodationPrivacyInspectionDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-collective-public",
        "ProtectAssociationBargainingCollectiveActionDuty",
        "ActualControlDependencyIntegrationAndEconomicRealityStandard",
    ),
    (
        "labour-participation-public",
        "ProvideHighConsequenceWorkplaceInformationParticipationDuty",
        "CompatibleLawStructureAndThresholdStandard",
    ),
    (
        "public-facing-service-protection-public",
        "ProvidePublicFacingServiceProtectionDuty",
        "AccessibleTermsSafetyCorrectionCancellationAndCollectiveRedressStandard",
    ),
    (
        "essential-public-facing-continuity-public",
        "MaintainEssentialPublicFacingServiceContinuityDuty",
        "EssentialServiceContinuityWithoutRightsWaiverStandard",
    ),
];

fn economic_always_duty_effect(key: &str) -> Option<usize> {
    Some(match key {
        "material-floor-finance"
        | "decent-work-opportunity"
        | "private-sphere-help-exit"
        | "personal-insolvency-fresh-start" => 200,
        "material-floor-continuity" | "housing-continuity" | "failure-essential-continuity" => 201,
        "economic-individual-remedy" | "economic-continuity-remedy" => 202,
        "labour-safety"
        | "labour-terms"
        | "labour-equality"
        | "labour-collective"
        | "labour-participation"
        | "public-facing-service-protection"
        | "essential-public-facing-continuity" => 204,
        "failure-wage-pension-protection"
        | "insolvency-evasion-prevention"
        | "labour-safety-public"
        | "labour-terms-public"
        | "labour-equality-public"
        | "labour-collective-public"
        | "labour-participation-public"
        | "public-facing-service-protection-public"
        | "essential-public-facing-continuity-public" => 199,
        _ => return None,
    })
}

const ECONOMIC_ASSERTION_WALL_IDS: [&str; 8] = [
    "recognition-binary",
    "recognition-arity-one",
    "recognition-non-ranked",
    "recognition-unread",
    "derived-only-title",
    "derived-only-liability",
    "book2-model-not-derived",
    "book2-statistic-not-derived",
];

const ECONOMIC_ASSERTION_WALL_EVIDENCE: [(&str, &str, &str); 8] = [
    ("recognition-binary", "reward(Quin).", "TRUE"),
    (
        "recognition-arity-one",
        "reward(Quin, EconomicRecognitionArityTwoProbe).",
        "FALSE",
    ),
    (
        "recognition-non-ranked",
        "reward(EconomicRecognitionRankOne, Quin).",
        "FALSE",
    ),
    (
        "recognition-unread",
        "prevents(Quin, CompensationRecognitionCoupling).",
        "TRUE",
    ),
    (
        "derived-only-title",
        "complete(RawTitleCarryResult, EconomicTitleCarryResult, RawTitleCarryRecord).",
        "FALSE",
    ),
    (
        "derived-only-liability",
        "complete(RawLiabilityCarryResult, EconomicLiabilityCarryResult, RawLiabilityCarryRecord).",
        "FALSE",
    ),
    (
        "book2-model-not-derived",
        "complete(RawBook2ModelResult, Book2ModelDerivedConstitutionalFact, RawBook2ModelRecord).",
        "FALSE",
    ),
    (
        "book2-statistic-not-derived",
        "complete(RawBook2StatisticResult, Book2StatisticDerivedConstitutionalFact, RawBook2StatisticRecord).",
        "FALSE",
    ),
];

const ECONOMIC_ACCEPTANCE_CASES: [(&str, &str, usize); 24] = [
    (
        "EAC-001",
        "every lawful ownership form, including a cooperative or public enterprise",
        7,
    ),
    (
        "EAC-002",
        "a person with zero contribution history and a prisoner refusing work without",
        5,
    ),
    (
        "EAC-003",
        "lawful collective action, a narrow minimum-service order, and a refused",
        5,
    ),
    (
        "EAC-004",
        "valid and pretextual licensing, accessible alternative proof, and credential-",
        4,
    ),
    (
        "EAC-005",
        "compensation remaining independent of `reward`, `false`, and `lose`",
        5,
    ),
    (
        "EAC-006",
        "lawful inheritance, estate-bounded debt, anti-concentration taxation",
        7,
    ),
    (
        "EAC-007",
        "a particular luxury asset being reachable while adequate secure housing and",
        3,
    ),
    (
        "EAC-008",
        "enforceable voluntary contracts, adhesion under dependency, invalid rights",
        8,
    ),
    (
        "EAC-009",
        "harmless household pooling with an independent confidential exit route",
        4,
    ),
    (
        "EAC-010",
        "a faith or cultural body selecting a genuinely expressive role but receiving",
        5,
    ),
    (
        "EAC-011",
        "a platform, monopoly, mutual-aid provider, landlord, lender, insurer, and",
        20,
    ),
    (
        "EAC-012",
        "an expired occupational licence or knowledge-exclusivity term whose",
        8,
    ),
    (
        "EAC-013",
        "enterprise petitioning remaining lawful while enterprise-treasury electoral",
        8,
    ),
    (
        "EAC-014",
        "capacity-based taxation, protected floor assets, transparent borrowing",
        5,
    ),
    (
        "EAC-015",
        "public and complementary payment instruments, offline access, credit denial",
        6,
    ),
    (
        "EAC-016",
        "personal fresh start, individualized proven fraud, employer insolvency with",
        7,
    ),
    (
        "EAC-017",
        "genuine scarcity versus budgetary withholding, hoarding, monopoly, and",
        12,
    ),
    (
        "EAC-018",
        "a resource-specific benefit finding remaining valid while generalized",
        10,
    ),
    (
        "EAC-019",
        "a regional economic policy remaining valid, a stronger regional protection",
        4,
    ),
    (
        "EAC-020",
        "a structural remedy preserving workers and essential users",
        2,
    ),
    (
        "EAC-021",
        "every temporary economic power ending under its own temporal contract",
        28,
    ),
    (
        "EAC-022",
        "recognition staying binary, arity-one, non-ranked, and unread",
        4,
    ),
    (
        "EAC-023",
        "raw lawful-title or liability assertions being refused wherever",
        2,
    ),
    (
        "EAC-024",
        "a Book 2 model or statistic being refused as a Nibli-derived constitutional",
        2,
    ),
];

const EXPECTED_ECONOMIC_ACCEPTANCE_CASES_SHA256: &str =
    "7bf864de16f0a78667f3857cf8fa94d67d98e2ec7ca68557e8a2fd1ae444e11a";

const SOURCE_FAMILIES: [&str; 8] = [
    "state-form-and-political-membership",
    "time-model",
    "substantive-equality-and-anti-subordination",
    "economic-pluralism-and-protected-private-sphere",
    "family-dependency-reproduction-and-collective-plurality",
    "ecological-commons-and-non-human-animal",
    "public-safety-defence-emergency-and-external-power",
    "current-formal-constitution",
];

const SCOPE_DISPOSITIONS: [&str; 5] = [
    "constitutional-invariant",
    "democratic-ordinary-law-choice",
    "protected-private-civic-freedom",
    "book-2-operation",
    "external-assumption",
];
const POSTURES: [&str; 6] = [
    "Derived",
    "Checked",
    "Evidenced",
    "Specified",
    "Reasoned",
    "Unestablished",
];
const UNESTABLISHED_DISPOSITIONS: [&str; 7] = [
    "routed-book-2",
    "external-assumption",
    "route-unbuilt",
    "evidence-pending",
    "author-ruling-pending",
    "refused",
    "not-establishable",
];
const EVIDENCE_KINDS: [&str; 4] = ["executable", "pattern-guard", "freshness", "inventory"];
const OVERLAYS: [&str; 4] = ["safety", "liveness", "feasibility", "none"];
const ROUTE_STATUSES: [&str; 3] = ["built", "available", "unbuilt"];
const DEFECT_DISPOSITIONS: [&str; 7] = [
    "eliminated-structurally",
    "prevented",
    "protected-consequence-contained",
    "remedied",
    "externally-bounded-assumption",
    "irreducible-limitation",
    "open-defect",
];
const RESPONSE_STAGES: [&str; 4] = [
    "detected",
    "interface-specified",
    "implemented-in-assigned-route",
    "operationally-assured-in-envelope",
];
const RESOLUTION_STATUSES: [&str; 2] = ["resolved-for-claim", "unresolved-for-claim"];
const PROPOSAL_DISPOSITIONS: [&str; 3] = ["added", "classified-out", "retained-limit"];
const ROUTING_MARKERS: [&str; 4] = [
    "not-constitutionally-prescribed",
    "democratic-ordinary-law-choice",
    "book-2-operation",
    "external-assumption",
];
const ENVELOPE_STATUSES: [&str; 3] = ["stub", "versioned-structure", "calibrated"];
const VALUE_STATUSES: [&str; 1] = ["declared-pending"];
const LAWFUL_SOURCES: [&str; 4] = [
    "constitutional-minimum-or-ceiling",
    "democratic-policy-target",
    "scientific-safety-boundary",
    "operational-diagnostic",
];
const GATE_REFS: [&str; 5] = ["gate-a", "gate-b", "gate-c", "gate-d", "gate-e"];
const ROLE_KINDS: [&str; 8] = [
    "life-course",
    "care-and-dependency",
    "learning-and-culture",
    "economic",
    "civic-political",
    "membership-and-mobility",
    "justice-and-coercion",
    "cross-cutting",
];
const ROLE_SCALES: [&str; 7] = [
    "individual",
    "household-association",
    "local",
    "regional",
    "national",
    "cross-jurisdictional",
    "intergenerational",
];
const POWER_POSITIONS: [&str; 2] = ["affected", "checking"];
const ROLE_ANCHORS: [&str; 3] = [
    "constitution-predicate-derived",
    "constitution-predicate-asserted",
    "ratified-doctrine-unimplemented",
];
const BODY_KINDS: [&str; 9] = [
    "universal-holder",
    "representative-chamber",
    "executive",
    "formal-continuity-office",
    "court",
    "independent-office",
    "administration",
    "predeclared-alternate",
    "predeclared-substitute-reviewer",
];
const ACCOUNTABILITY_ROUTE_TYPES: [&str; 4] =
    ["challenge", "review", "audit", "political-accountability"];
const ADVERSE_DETERMINATION_KINDS: [&str; 2] = ["none-by-design", "enumerated"];
const CUSTODY_T3_RELATIONS: [&str; 2] = ["not-reusable", "retained-application"];
const CUSTODY_T3_APPLICANT: &str = "FS-BOD-35";
const CRITERIA_SLUGS: [&str; 7] = [
    "adequacy",
    "accessibility-equality",
    "continuity",
    "resilience",
    "sustainability",
    "resource",
    "safety",
];
const REVIEW_CRITERIA: [&str; 15] = [
    "declared-rights",
    "declared-liberties",
    "declared-powers",
    "declared-duties",
    "protected-private-boundaries",
    "cross-domain-dependencies",
    "ordinary-life-account",
    "failure-and-recovery-paths",
    "adequacy",
    "accessibility-equality",
    "continuity",
    "resilience",
    "sustainability",
    "safety",
    "resource",
];
const PROTOCOL_STATUS_CONFIRMED: &str =
    "repository-enforced 2026-08-27 -- receipt-aware mechanical-closure protocol v6";
const SCOPE_AUDIT_POLICY_BASIS: &str =
    "new-book-plans/full-society-scope-review-protocol.md::## 5. Mechanical Gate A closure";
const SCOPE_AUDIT_METHOD: &str = "repository-source-derived-adversarial-audit";
const SCOPE_AUDIT_RESULT: &str = "passed-with-recorded-limits";
const SCOPE_AUDIT_EVIDENCE_CEILING: &str = "Checked repository structure and watched-failing mutations over the declared axes only; no independent-human warrant, reader response, external truth, operation, delivery, feasibility, liveness, calibration, timeless completeness, or authentication of the audit's own trust root follows.";
const LEDGER_CURRENT_AUDIT_CONTROL_REF: &str =
    concat!("src/checks/ledger.rs::fn negative_", "controls(");
const LEDGER_REFRESH_COMMAND: &str = "./verify.sh --refresh full-society-ledger";
const CLOSURE_REFRESH_COMMAND: &str = "./verify.sh --refresh constitutional-closure";
const EMIT_RECEIPT_COMMAND: &str =
    "./verify.sh --emit-receipt new-book-plans/verification-receipts";
const CURRENT_AUDIT_COMMAND_PREFIX: [&str; 3] = [
    LEDGER_REFRESH_COMMAND,
    CLOSURE_REFRESH_COMMAND,
    EMIT_RECEIPT_COMMAND,
];
const GATE_A_ASSURANCE_REFS: [&str; 9] = [
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-01\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-02\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-03\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-04\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-05\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-06\"",
    "new-book-plans/full-society-ledger.json::\"id\": \"FS-RTE-07\"",
    "new-book-plans/full-society-ledger.md::# Full-Society Domain-and-Layer Ledger — Generated Report",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md::# Constitutional-closure and model-allocation audit",
];
const R7_CONSEQUENCE: &str = "repository structure, declared criteria, Gate-A defect coverage, and watched-failing mutations are reproducibly checked";
const R7_CLOSURE_CONDITION: &str = "the current-source repository audit is present and exact; external human review remains optional evidence and is never a project gate";
const REVIEW_PACKET_PATHS: [&str; 8] = [
    "new-book-plans/full-society-ledger.json",
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
    "new-book-plans/book-1-constitutional-coverage-map.md",
    "new-book-plans/full-society-boundary-decision.md",
    "new-book-plans/book-1-assurance-portfolio-decision.md",
    "new-book-plans/full-society-scope-review-protocol.md",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md",
];
const REVIEWER_CONSENT: &str = "consented-to-the-commissioned-review";
const REVIEWER_CONFLICT_CLEAR: &str = "no-reviewed-artifact-authorship-or-generation; not-custodian-darshu-or-dhanush; no-declared-conflict";
const REVIEWER_COMPENSATION_CLEAR: &str = "not-findings-contingent";
const RUBRIC_STATUS_CANDIDATE: &str = "candidate — author confirmation pending";
const RUBRIC_STATUS_CONFIRMED: &str = "author-confirmed 2026-08-09 — basis recorded";
const FLOW_KINDS: [&str; 9] = [
    "authority",
    "information",
    "care",
    "labour",
    "resources",
    "money",
    "claims",
    "services",
    "accountability",
];
const DEPENDENCY_CLASSES: [&str; 4] = [
    "constitutionally-guaranteed",
    "democratically-selected",
    "operationally-supplied",
    "externally-assumed",
];
const LOOP_KINDS: [&str; 5] = ["service", "feedback", "fiscal", "ecological", "sequence"];
const LIFECYCLE_PATHS: [&str; 4] = ["right", "power", "record", "outside-ratified-paths"];
const SCENARIO_KINDS: [&str; 4] = ["journey", "stress", "collision", "compound-shock"];
const COLLISION_AXES: [&str; 10] = [
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
];
const SHOCK_KINDS: [&str; 6] = [
    "pandemic",
    "famine",
    "infrastructure-failure",
    "displacement",
    "institutional-capture",
    "conflicting-jurisdictions",
];
const PROTECTED_SPHERE_FORMS: [&str; 4] = [
    "freedom-without-permission",
    "non-recording-non-compulsion",
    "evidenced-harm-threshold",
    "recourse-against-interference",
];
const ENVELOPE_STUB_ID: &str = "FS-ENV-00";
const IMPLEMENTED_STAGES: [&str; 2] = [
    "implemented-in-assigned-route",
    "operationally-assured-in-envelope",
];

#[derive(Debug)]
struct LedgerError(String);

type LedgerResult<T> = Result<T, LedgerError>;

impl LedgerError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for LedgerError {}

impl From<std::io::Error> for LedgerError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct RequiredNullable<T>(Option<T>);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Term {
    text: String,
    basis: String,
    source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    choice_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bounds: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure_default: Option<String>,
}

type TermSet = BTreeMap<String, Term>;
type ProfileTerms = BTreeMap<String, TermSet>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Axis {
    id: String,
    name: String,
    values: String,
    note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerBucket {
    answer: String,
    refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoutingBucket {
    routing_marker: String,
    note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnresolvedDetail {
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    public_claim_limitation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnresolvedBucket {
    unresolved: UnresolvedDetail,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum DomainBucket {
    Answer(AnswerBucket),
    Routing(RoutingBucket),
    Unresolved(UnresolvedBucket),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioAnswerApplicability {
    answer: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioDeferredApplicability {
    deferred_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ScenarioApplicability {
    Answer(ScenarioAnswerApplicability),
    Deferred(ScenarioDeferredApplicability),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Domain {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    constitutional_invariants: DomainBucket,
    ordinary_law_choices: DomainBucket,
    protected_private_civic: DomainBucket,
    book2_operations: DomainBucket,
    external_assumptions_note: DomainBucket,
    class_refs: Vec<String>,
    bodies_refs: Vec<String>,
    external_assumption_refs: Vec<String>,
    legacy_row_refs: Vec<String>,
    scenario_applicability: ScenarioApplicability,
    reader_destination: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyRow {
    id: String,
    domain_title: String,
    legacy_coverage: String,
    legacy_scope_requirement: String,
    legacy_status_cell: String,
    legacy_gap: String,
    legacy_status: String,
    domain_refs: Vec<String>,
    split_claim_refs: Vec<String>,
    split_state: String,
    source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unresolved: Option<UnresolvedDetail>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    id: String,
    title: String,
    claim: String,
    layer: String,
    domain_refs: Vec<String>,
    legacy_row_ref: RequiredNullable<String>,
    class_refs: Vec<String>,
    posture: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unimplemented_marker: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unestablished_disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mutation_ref: Option<String>,
    route_ref: String,
    overlay: String,
    scope_bound: String,
    evidence_notes: Vec<String>,
    public_claim_restriction: String,
    applicability: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    envelope_id: String,
    closure_requirement_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerPosition {
    body_ref: String,
    position: String,
    note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FormalAnchor {
    anchor: String,
    refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerHeld {
    power: String,
    source_ref: String,
    affected_role_refs: Vec<String>,
    checking_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Role {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    role_kind: String,
    domain_refs: Vec<String>,
    scales: Vec<String>,
    power_positions: Vec<PowerPosition>,
    formal_anchor: FormalAnchor,
    floor_invariance: String,
    source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    power_held: Option<PowerHeld>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmittedRole {
    omitted_role: String,
    risk_reason: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmittedScale {
    role_ref: String,
    omitted_scale: String,
    risk_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmittedDomain {
    role_ref: String,
    omitted_domain_ref: String,
    risk_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum RoleOmission {
    Role(OmittedRole),
    Scale(OmittedScale),
    Domain(OmittedDomain),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestBinding {
    id: String,
    status: String,
    assertion: String,
    source_refs: Vec<String>,
    executable_ref: RequiredNullable<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Power {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    manifest_key: String,
    source_family: String,
    posture: String,
    evidence_kind: String,
    profiles: Vec<String>,
    domain_refs: Vec<String>,
    affected_claim_refs: Vec<String>,
    holder_body_refs: Vec<String>,
    holder_role_refs: Vec<String>,
    affected_role_refs: Vec<String>,
    checking_role_refs: Vec<String>,
    route_ref: String,
    overlay: String,
    public_claim_restriction: String,
    structural_wall_refs: Vec<String>,
    related_power_refs: Vec<String>,
    enforcement_mechanism: String,
    book2_owner_ref: String,
    source_refs: Vec<String>,
    primary_class_ref: String,
    secondary_class_refs: Vec<String>,
    contract_terms: TermSet,
    profile_terms: ProfileTerms,
    required_separation_pairs: Vec<Vec<String>>,
    permitted_inputs: Vec<String>,
    prohibited_inputs: Vec<String>,
    permitted_downstream_effects: Vec<String>,
    evidence_authority: Term,
    negative_test: TestBinding,
    counterfactual: TestBinding,
    part_v_status: String,
    book2_handoff: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicPowerRuleField {
    name: String,
    value: String,
    scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicPowerRuleRequirement {
    value: String,
    scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicPowerRuleContract {
    power_ref: String,
    temporal_contract: String,
    jurisdiction_kind: String,
    authority_scope_kind: String,
    holder: String,
    fields: Vec<EconomicPowerRuleField>,
    requirements: Vec<EconomicPowerRuleRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicCarryRuleContract {
    carry_kind: String,
    record_kind: String,
    temporal_contract: String,
    current_kind: String,
    current_selection: String,
    result_kind: String,
    branch: String,
    finding_kind: String,
    jurisdiction_kind: String,
    legal_scope_kind: String,
    interest: EconomicPowerRuleField,
    requirement: EconomicPowerRuleRequirement,
    predecessor_record_scope: String,
    predecessor_result_scope: String,
    successor_event_scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicAcceptanceSupport {
    owner_kind: String,
    owner_id: String,
    polarity: String,
    formal_refs: Vec<String>,
    pin_ref: String,
    query: String,
    expected_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicAcceptanceMapping {
    variant_id: String,
    mapping_id: String,
    assertion: String,
    supports: Vec<EconomicAcceptanceSupport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EconomicAcceptanceCase {
    case_id: String,
    source_needle: String,
    mappings: Vec<EconomicAcceptanceMapping>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstitutionalEffect {
    id: String,
    title: String,
    effect_key: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    posture: String,
    primary_class_ref: String,
    secondary_class_refs: Vec<String>,
    profiles: Vec<String>,
    affected_claim_refs: Vec<String>,
    domain_refs: Vec<String>,
    holder_role_refs: Vec<String>,
    affected_role_refs: Vec<String>,
    checking_role_refs: Vec<String>,
    permitted_inputs: Vec<String>,
    prohibited_inputs: Vec<String>,
    permitted_downstream_effects: Vec<String>,
    contract_terms: TermSet,
    profile_terms: ProfileTerms,
    evidence_authority: Term,
    negative_test: TestBinding,
    counterfactual: TestBinding,
    part_v_status: String,
    book2_handoff: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerTemplate {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    manifest_key: String,
    source_refs: Vec<String>,
    contract_terms: TermSet,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerRefusal {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    manifest_key: String,
    source_family: String,
    refusal: String,
    scope: String,
    protected_boundary: String,
    permitted_residual: String,
    non_authorisation: String,
    affected_power_refs: Vec<String>,
    domain_refs: Vec<String>,
    affected_claim_refs: Vec<String>,
    affected_role_refs: Vec<String>,
    route_ref: String,
    public_claim_restriction: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerCrosswalk {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    manifest_key: String,
    crosswalk_action: String,
    target_power_refs: Vec<String>,
    current_effect: String,
    retired_residual_effect: String,
    non_extension: String,
    transition_owner_ref: String,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageFamily {
    id: String,
    title: String,
    state: String,
    source_family_refs: Vec<String>,
    card_refs: Vec<String>,
    template_refs: Vec<String>,
    refusal_refs: Vec<String>,
    crosswalk_refs: Vec<String>,
    formal_statement_refs: Vec<String>,
    pin_group_refs: Vec<String>,
    counterfactual_refs: Vec<String>,
    prose_refs: Vec<String>,
    part_v_refs: Vec<String>,
    blocked_before_drafting: String,
    source_refs: Vec<String>,
    effect_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeparationConstraint {
    functions: Vec<String>,
    reason: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionAllocation {
    id: String,
    power_ref: String,
    affected_claim_refs: Vec<String>,
    decisive_fact_writer_body_refs: Vec<String>,
    decisive_fact_writer_role_refs: Vec<String>,
    decider_body_refs: Vec<String>,
    decider_role_refs: Vec<String>,
    executor_body_refs: Vec<String>,
    executor_role_refs: Vec<String>,
    auditor_body_refs: Vec<String>,
    auditor_role_refs: Vec<String>,
    final_remedy_body_refs: Vec<String>,
    final_remedy_role_refs: Vec<String>,
    separation_constraints: Vec<SeparationConstraint>,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlternatePresent {
    route: String,
    source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    misuse_note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlternateAbsent {
    no_alternate_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum AlternateRoute {
    Present(AlternatePresent),
    Absent(AlternateAbsent),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuralSatisfiability {
    defect_refs: Vec<String>,
    reason: String,
    satisfiability_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dependency {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    flow_kind: String,
    dependency_class: String,
    from_ref: String,
    to_ref: String,
    steward_ref: String,
    lifecycle_path: String,
    interim_continuity: String,
    remedy_route: String,
    restoration: String,
    systemic_correction: String,
    alternate_route: AlternateRoute,
    source_refs: Vec<String>,
    structural_satisfiability: StructuralSatisfiability,
    closure_component_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyLoop {
    id: String,
    loop_kind: String,
    member_edge_refs: Vec<String>,
    boundedness: String,
    steward_ref: String,
    owner_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RefusedFlow {
    refused_flow: String,
    flow_kind: String,
    refusal_reason: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    scenario_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    collision_axis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shock_kind: Option<String>,
    domain_refs: Vec<String>,
    dependency_refs: Vec<String>,
    steward_ref: String,
    ordinary_route: String,
    failure_route: String,
    recovery_route: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bounded_witness_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    protected_sphere_forms: Option<Vec<String>>,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmittedScenario {
    omitted_scenario: String,
    risk_reason: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OmittedDependency {
    omitted_dependency_ref: String,
    risk_reason: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum ScenarioOmission {
    Scenario(OmittedScenario),
    Dependency(OmittedDependency),
}

impl ScenarioOmission {
    fn risk_reason(&self) -> &str {
        match self {
            Self::Scenario(value) => &value.risk_reason,
            Self::Dependency(value) => &value.risk_reason,
        }
    }

    fn source_ref(&self) -> &str {
        match self {
            Self::Scenario(value) => &value.source_ref,
            Self::Dependency(value) => &value.source_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    route_status: String,
    warrants: String,
    cannot_warrant: String,
    falsification_condition: String,
    negative_control: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalAssumption {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    assumption: String,
    failure_consequence: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeField {
    id: String,
    definition: String,
    value_status: String,
    book2_owner_ref: String,
    dependents: Vec<String>,
    invariance: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    envelope_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    envelope_version: Option<String>,
    note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<EnvelopeField>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Threshold {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    criterion_ref: String,
    domain_refs: Vec<String>,
    definition: String,
    binding_ref: String,
    lawful_source: String,
    decision_owner_ref: String,
    measurement_owner_ref: String,
    value_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefectControls {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reintroduction_control_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initiation_control_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    containment_control_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recovery_fields: Option<RecoveryFields>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryFields {
    actor: String,
    trigger: String,
    interim_continuity: String,
    restoration: String,
    challenge: String,
    recurrence_control: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefectHistory {
    pub(crate) field: String,
    pub(crate) value: String,
    pub(crate) date: String,
    pub(crate) note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defect {
    id: String,
    defect_id: String,
    title: String,
    applicability: String,
    layer: String,
    applicable_gate_refs: Vec<String>,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    defect_disposition: String,
    response_stage: String,
    affected_claim_ref: String,
    consequence_id: String,
    scope_id: String,
    envelope_id: String,
    source_version: String,
    pub(crate) history: Vec<DefectHistory>,
    evidence_notes: Vec<String>,
    residual_citations: Vec<String>,
    controls: DefectControls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    book2_crosswalk: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolutionReceipt {
    id: String,
    title: String,
    defect_row_ref: String,
    defect_id: String,
    affected_claim_ref: String,
    consequence_id: String,
    defect_disposition: String,
    response_stage: String,
    claim_posture: String,
    route_ref: String,
    admissible_evidence: String,
    assurance_ceiling: String,
    what_failed: String,
    hostile_witness: String,
    why_it_failed: String,
    response_change: String,
    now_follows: String,
    proof_ref: String,
    negative_control_ref: String,
    still_does_not_follow: String,
    residuals: Vec<String>,
    scope_id: String,
    source_version: String,
    envelope_id: String,
    owner_ref: String,
    eligible_gate: String,
    reader_mapping_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityRow {
    defect_disposition: String,
    allowed_response_stages: Vec<String>,
    resolution_eligible: bool,
    resolution_requirement: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumMapping {
    source_file: String,
    field: String,
    value: String,
    canonical: String,
    note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumExclusion {
    source_file: String,
    field: String,
    value: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResidualExclusion {
    source_file: String,
    token: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureComponent {
    component: String,
    record_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureRequirementProfile {
    id: String,
    requirement_kind: String,
    applies_to_claim_refs: Vec<String>,
    components: Vec<ClosureComponent>,
    source_record_ref: RequiredNullable<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureClaimContract {
    id: String,
    claim_ref: String,
    required_profile_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelAllocation {
    id: String,
    claim_ref: String,
    primary_route_ref: String,
    required_route_refs: Vec<String>,
    closure_profile_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HazardAssessment {
    hazard: String,
    control_refs: Vec<String>,
    defect_refs: Vec<String>,
    reason: String,
    closure_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoopHazardControl {
    id: String,
    loop_ref: String,
    affected_claim_refs: Vec<String>,
    assessments: Vec<HazardAssessment>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BottleneckDisposition {
    id: String,
    dependency_ref: String,
    affected_claim_refs: Vec<String>,
    control_refs: Vec<String>,
    defect_refs: Vec<String>,
    reason: String,
    closure_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeAudit {
    id: String,
    title: String,
    source_version: String,
    scope_sha256: String,
    protocol_sha256: String,
    executed_at_utc: String,
    method: String,
    criterion_coverage: Vec<String>,
    control_refs: Vec<String>,
    commands: Vec<String>,
    finding_refs: Vec<String>,
    result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    author_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_basis: Option<String>,
    evidence_ceiling: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verification_receipt_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimLimitation {
    defect_ref: String,
    affected_claim_ref: String,
    public_claim_restriction: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClosureProjection {
    pub(crate) gate: String,
    pub(crate) permitted_claim: String,
    pub(crate) candidate_commit_sha: String,
    pub(crate) source_version: String,
    pub(crate) scope_sha256: String,
    pub(crate) envelope_ref: String,
    pub(crate) audit_cutoff_at_utc: String,
    pub(crate) scope_audit_ref: String,
    pub(crate) assurance_record_refs: Vec<String>,
    pub(crate) residual_refs: Vec<String>,
    claim_limitations: Vec<ClaimLimitation>,
    pub(crate) closure_policy_ref: String,
    pub(crate) verification_receipt_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceGate {
    verdict: String,
    rollup_rule: String,
    gate_a_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoppingRule {
    named_axes: Vec<String>,
    closure_conditions: Vec<String>,
    materiality_test: String,
    boundary: String,
    no_hiding_rule: String,
    source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeverityRubric {
    critical: String,
    material: String,
    minor: String,
    materiality_ref: String,
    rubric_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    confirmation_basis: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionalCriterion {
    id: String,
    name: String,
    definition: String,
    binding_refs: Vec<String>,
    provenance: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionalCriteria {
    criteria: Vec<FunctionalCriterion>,
    drift_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DispositionCounts {
    #[serde(rename = "card-required")]
    card_required: usize,
    #[serde(rename = "power-contract-template")]
    power_contract_template: usize,
    #[serde(rename = "existing-formal-crosswalk")]
    existing_formal_crosswalk: usize,
    #[serde(rename = "explicit-refusal-limit")]
    explicit_refusal_limit: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerSourceInventory {
    artifact_ref: String,
    artifact_sha256: String,
    source_commit: String,
    inventory_status: String,
    row_count: usize,
    disposition_counts: DispositionCounts,
    power_population_status: String,
    known_allocation_gaps: Vec<String>,
    owner_ref: String,
    closure_condition: String,
    scope_ceiling: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalCounts {
    powers: usize,
    templates: usize,
    refusals: usize,
    crosswalks: usize,
    function_allocations: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolvedAllocationGap {
    gap: String,
    body_refs: Vec<String>,
    role_refs: Vec<String>,
    source_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerPopulation {
    status: String,
    completed_source_families: Vec<String>,
    expected_final_counts: FinalCounts,
    resolved_allocation_gaps: Vec<ResolvedAllocationGap>,
    evidence_ceiling: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoveragePopulation {
    status: String,
    completed_source_families: Vec<String>,
    expected_final_card_count: usize,
    legacy_fields_permitted_until_complete: bool,
    evidence_ceiling: String,
    expected_constitutional_effect_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewDesignation {
    severity_owner: String,
    independent_checker: String,
    custodian: String,
    designated_date: String,
    basis: String,
    designation_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewProtocol {
    protocol_ref: String,
    protocol_status: String,
    status_line_ref: String,
    policy_basis: String,
    designation: ReviewDesignation,
    mode: String,
    external_review_policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeferredPopulation {
    record_type: String,
    owner_ref: String,
    closure_condition: String,
    stage: String,
}

// These optional external-review populations are empty in the ratified source,
// but remain fully typed so a future populated transition cannot bypass schema
// validation before its semantic admissibility checks run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceivedWindow {
    opens_at_utc: String,
    closes_at_utc: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviewer {
    identity: String,
    discipline: String,
    criterion_refs: Vec<String>,
    consent_attestation: String,
    conflict_attestation: String,
    compensation_attestation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewCommission {
    id: String,
    title: String,
    source_version: String,
    scope_sha256: String,
    protocol_sha256: String,
    plant_commitment_sha256: String,
    seed_commitment_sha256: String,
    commissioned_at_utc: String,
    received_window: ReceivedWindow,
    cutoff_at_utc: String,
    custodian_identity: String,
    reviewers: Vec<Reviewer>,
    criterion_coverage: Vec<String>,
    packet_paths: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntakeReceipt {
    frozen_at_utc: String,
    ordered_proposal_ids: Vec<String>,
    manifest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedResult {
    proposal_ref: String,
    expected_materiality: String,
    expected_severity: RequiredNullable<String>,
    expected_disposition: String,
    verified_by: String,
    verification_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlReveal {
    revealed_at_utc: String,
    plant_preimage_sha256: String,
    seed_preimage_sha256: String,
    planted_proposal_ref: RequiredNullable<String>,
    seed_results: Vec<SeedResult>,
    plant_match_checked_by: String,
    plant_match_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewControl {
    status: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewEvent {
    id: String,
    title: String,
    commission_ref: String,
    packet_commit_sha: String,
    source_version: String,
    scope_sha256: String,
    protocol_sha256: String,
    intake_receipt: IntakeReceipt,
    control_reveal: ControlReveal,
    seeded_control: ReviewControl,
    planted_control: ReviewControl,
    outcome_status: String,
    outcome_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedLimitBinding {
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    applicable_gate_refs: Vec<String>,
    public_claim_restriction: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    id: String,
    title: String,
    proposal: String,
    source_kind: String,
    source_identity: String,
    received_at_utc: String,
    triaged_at_utc: String,
    severity_owner_identity: String,
    materiality_finding: String,
    materiality_reason: String,
    classification: String,
    checked_at_utc: String,
    independent_checker_identity: String,
    check_finding: String,
    check_reason: String,
    proposal_disposition: String,
    disposition_at_utc: String,
    reasons: String,
    review_event_ref: String,
    control_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_record_refs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    routed_unestablished_disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    defect_row_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retained_limit_binding: Option<RetainedLimitBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommonTermSet {
    universal_human_standing: Term,
    political_membership: Term,
    franchise: Term,
    candidacy: Term,
    current_office: Term,
    current_lawful_power: Term,
    permanent_historical_public_answerability: Term,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OfficeContract {
    democratic_source: Term,
    jurisdiction: Term,
    ordinary_function: Term,
    delegation_boundary: Term,
    conflict_and_recusal: Term,
    appointment: Term,
    removal: Term,
    succession: Term,
    temporal_status: Term,
    public_reason_duty: Term,
    anti_capture: Term,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountabilityRoute {
    route_type: String,
    checker_body_refs: Vec<String>,
    checker_role_refs: Vec<String>,
    term: Term,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdverseItem {
    name: String,
    subject: String,
    appeal: Term,
    remedy: Term,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdverseDeterminations {
    kind: String,
    note: Term,
    items: Vec<AdverseItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalContract {
    contract_kind: String,
    custody_t3_relation: String,
    term: Term,
    failure_polarity: Term,
    expiry_default: Term,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Body {
    id: String,
    title: String,
    applicability: String,
    layer: String,
    status: String,
    severity: String,
    consequence: String,
    owner_ref: String,
    closure_condition: String,
    job: String,
    may_not_do_alone: String,
    required_check: String,
    source_ref: String,
    source_refs: Vec<String>,
    body_kind: String,
    status_senses: CommonTermSet,
    office_contract: OfficeContract,
    accountability_routes: Vec<AccountabilityRoute>,
    adverse_determinations: AdverseDeterminations,
    temporal_contract: TemporalContract,
    delegated_mechanics: Vec<Term>,
    book2_handoff: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundSources {
    assurance_portfolio: String,
    full_society_boundary: String,
}

// Source-specific, strict projections of the sibling reviewed contracts. The
// ledger consumes only enum, residual, and bounded-witness fields; every other
// known top-level field is explicitly acknowledged as `IgnoredAny`, so a new
// sibling root field still fails closed instead of disappearing into a generic
// JSON tree.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssertionPremiseProjection {
    claimed_actor: IgnoredAny,
    tuple_claim: IgnoredAny,
    current_writer_authority: IgnoredAny,
    required_writer_authority: IgnoredAny,
    current_provenance: IgnoredAny,
    required_provenance: IgnoredAny,
    cheapest_harm: IgnoredAny,
    withholding_deletion_harm: IgnoredAny,
    current_challenge_route: IgnoredAny,
    required_challenge_route: IgnoredAny,
    risk_dispositions: Vec<String>,
    owner_ref: IgnoredAny,
    tags: IgnoredAny,
    refused_alternative: Option<IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssertionSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    aliases: IgnoredAny,
    cheapest_harm_metric: IgnoredAny,
    risk_disposition_meanings: BTreeMap<String, IgnoredAny>,
    required_semantic_tags: IgnoredAny,
    additional_writable_channels: IgnoredAny,
    rules_sha256: IgnoredAny,
    facts_sha256: IgnoredAny,
    route_fingerprints: IgnoredAny,
    reserved_retired_relations: IgnoredAny,
    derived_relations: IgnoredAny,
    premises: BTreeMap<String, AssertionPremiseProjection>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssuranceClaimProjection {
    id: String,
    title: IgnoredAny,
    claim: IgnoredAny,
    argument: IgnoredAny,
    posture: String,
    dimensions: IgnoredAny,
    current_evidence: IgnoredAny,
    known_failure: IgnoredAny,
    target_contract: IgnoredAny,
    acceptance_evidence: IgnoredAny,
    residual_assumption: IgnoredAny,
    owner_ref: IgnoredAny,
    temporal_status: IgnoredAny,
    book2_handoff: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssuranceDefeaterProjection {
    id: String,
    title: IgnoredAny,
    attack: IgnoredAny,
    disposition: String,
    owner_claims: IgnoredAny,
    failure_consequence: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssuranceSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    assertion_surface_contracts_sha256: IgnoredAny,
    title: IgnoredAny,
    top_claim: IgnoredAny,
    status_meanings: BTreeMap<String, IgnoredAny>,
    required_dimensions: IgnoredAny,
    limitations: BTreeMap<String, String>,
    boundary: IgnoredAny,
    claims: Vec<AssuranceClaimProjection>,
    record_classes: IgnoredAny,
    premise_classes: IgnoredAny,
    defeaters: Vec<AssuranceDefeaterProjection>,
    fail_safe_defaults: IgnoredAny,
    narrowness_impacts: IgnoredAny,
    acceptance_gate: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RedTeamScenarioProjection {
    id: String,
    title: IgnoredAny,
    kind: IgnoredAny,
    state_refs: IgnoredAny,
    route_refs: IgnoredAny,
    queries: IgnoredAny,
    comparisons: IgnoredAny,
    preserved_invariants: IgnoredAny,
    attribution: IgnoredAny,
    interpretation: IgnoredAny,
    result: IgnoredAny,
    opposite_failure: IgnoredAny,
    authorised_disposition_boundary: IgnoredAny,
    residual_limit: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RedTeamObservationProjection {
    id: String,
    title: IgnoredAny,
    snapshot_ref: IgnoredAny,
    route_ref: IgnoredAny,
    world_descriptions: IgnoredAny,
    queries: IgnoredAny,
    prohibited_inference: IgnoredAny,
    boundary: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RedTeamSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: String,
    evidence_role: IgnoredAny,
    constitution_sha256: IgnoredAny,
    assertion_surface_contracts_sha256: IgnoredAny,
    record_integrity_assurance_case_sha256: IgnoredAny,
    posture_meanings: BTreeMap<String, IgnoredAny>,
    required_routes: IgnoredAny,
    required_scenarios: IgnoredAny,
    limits: BTreeMap<String, String>,
    temporal_handoff: IgnoredAny,
    routes: IgnoredAny,
    snapshots: IgnoredAny,
    scenarios: Vec<RedTeamScenarioProjection>,
    observational_equivalence: Vec<RedTeamObservationProjection>,
    narrowness_impacts: IgnoredAny,
    acceptance_result: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentLabelProjection {
    verdict: String,
    amendment: IgnoredAny,
    declared_target: IgnoredAny,
    summary: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentCaseProjection {
    id: String,
    title: IgnoredAny,
    mutations: IgnoredAny,
    mutation_sha256: IgnoredAny,
    expected_source_sha256: IgnoredAny,
    steps: IgnoredAny,
    source_assertions: IgnoredAny,
    source_effect: IgnoredAny,
    assertion_surface_expectation: IgnoredAny,
    declared_label: AmendmentLabelProjection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: String,
    evidence_role: IgnoredAny,
    subprocess_timeout_seconds: IgnoredAny,
    constitution_sha256: IgnoredAny,
    assertion_surface_contracts_sha256: IgnoredAny,
    record_integrity_assurance_case_sha256: IgnoredAny,
    label_verdict_meanings: BTreeMap<String, IgnoredAny>,
    limits: BTreeMap<String, String>,
    required_cases: IgnoredAny,
    cases: Vec<AmendmentCaseProjection>,
    narrowness_impacts: IgnoredAny,
    acceptance_result: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementMutationProjection {
    id: String,
    title: IgnoredAny,
    kind: IgnoredAny,
    mutations: IgnoredAny,
    mutation_sha256: IgnoredAny,
    expected_source_sha256: IgnoredAny,
    alarm_setup_facts: IgnoredAny,
    observations: IgnoredAny,
    baseline_flips: IgnoredAny,
    err_absence_case_refs: IgnoredAny,
    interpretation: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: String,
    evidence_role: IgnoredAny,
    subprocess_timeout_seconds: IgnoredAny,
    constitution_sha256: IgnoredAny,
    producer_fingerprints: IgnoredAny,
    destination_constants: IgnoredAny,
    destination_constants_sha256: IgnoredAny,
    subject_contract: IgnoredAny,
    axis_contract: IgnoredAny,
    limits: BTreeMap<String, String>,
    matrix: IgnoredAny,
    required_mutations: IgnoredAny,
    mutations: Vec<PlacementMutationProjection>,
    narrowness_impacts: IgnoredAny,
    acceptance_result: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalCaseProjection {
    id: String,
    title: IgnoredAny,
    stage: IgnoredAny,
    process_role: IgnoredAny,
    description: IgnoredAny,
    additions: IgnoredAny,
    deletions: IgnoredAny,
    checks: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalAttackProjection {
    id: String,
    stage: IgnoredAny,
    case_refs: IgnoredAny,
    control: IgnoredAny,
    posture: String,
    boundary: IgnoredAny,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemporalSiblingProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: String,
    evidence_role: IgnoredAny,
    subprocess_timeout_seconds: IgnoredAny,
    constitution_sha256: IgnoredAny,
    bound_sources_sha256: IgnoredAny,
    marker_sha256: IgnoredAny,
    stage_source_sha256: IgnoredAny,
    source_effect_binding: IgnoredAny,
    pre_t3_custody_rule: IgnoredAny,
    temporal_input_contracts: IgnoredAny,
    stages: IgnoredAny,
    cases: Vec<TemporalCaseProjection>,
    fresh_process_pairs: IgnoredAny,
    attacks: Vec<TemporalAttackProjection>,
    narrowness_impacts: IgnoredAny,
    limits: BTreeMap<String, String>,
    acceptance_result: IgnoredAny,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LedgerDocument {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    evidence_role: String,
    source_version: String,
    bound_sources_sha256: BoundSources,
    axes: Vec<Axis>,
    scope_disposition_meanings: BTreeMap<String, String>,
    gate_applicability_meanings: BTreeMap<String, String>,
    routing_marker_meanings: BTreeMap<String, String>,
    posture_meanings: BTreeMap<String, String>,
    unestablished_disposition_meanings: BTreeMap<String, String>,
    evidence_kind_meanings: BTreeMap<String, String>,
    overlay_meanings: BTreeMap<String, String>,
    route_status_meanings: BTreeMap<String, String>,
    defect_disposition_meanings: BTreeMap<String, String>,
    response_stage_meanings: BTreeMap<String, String>,
    resolution_status_meanings: BTreeMap<String, String>,
    proposal_disposition_meanings: BTreeMap<String, String>,
    envelope_status_meanings: BTreeMap<String, String>,
    value_status_meanings: BTreeMap<String, String>,
    lawful_source_meanings: BTreeMap<String, String>,
    role_kind_meanings: BTreeMap<String, String>,
    scale_meanings: BTreeMap<String, String>,
    power_position_meanings: BTreeMap<String, String>,
    role_anchor_meanings: BTreeMap<String, String>,
    flow_kind_meanings: BTreeMap<String, String>,
    dependency_class_meanings: BTreeMap<String, String>,
    loop_kind_meanings: BTreeMap<String, String>,
    lifecycle_path_meanings: BTreeMap<String, String>,
    scenario_kind_meanings: BTreeMap<String, String>,
    collision_axis_meanings: BTreeMap<String, String>,
    shock_kind_meanings: BTreeMap<String, String>,
    protected_sphere_form_meanings: BTreeMap<String, String>,
    compatibility_table: Vec<CompatibilityRow>,
    enum_mapping: Vec<EnumMapping>,
    enum_mapping_exclusions: Vec<EnumExclusion>,
    residual_coverage_exclusions: Vec<ResidualExclusion>,
    id_registry: BTreeMap<String, String>,
    domains: Vec<Domain>,
    legacy_rows: Vec<LegacyRow>,
    claims: Vec<Claim>,
    bodies: Vec<Body>,
    routes: Vec<Route>,
    external_assumptions: Vec<ExternalAssumption>,
    envelope: Vec<Envelope>,
    roles: Vec<Role>,
    role_omissions: Vec<RoleOmission>,
    power_source_inventory: PowerSourceInventory,
    power_population: PowerPopulation,
    coverage_population: CoveragePopulation,
    powers: Vec<Power>,
    economic_power_rule_contracts: Vec<EconomicPowerRuleContract>,
    economic_carry_rule_contracts: Vec<EconomicCarryRuleContract>,
    economic_acceptance_cases: Vec<EconomicAcceptanceCase>,
    power_contract_templates: Vec<PowerTemplate>,
    power_refusals: Vec<PowerRefusal>,
    power_crosswalk_dispositions: Vec<PowerCrosswalk>,
    coverage_families: Vec<CoverageFamily>,
    dependencies: Vec<Dependency>,
    dependency_loops: Vec<DependencyLoop>,
    refused_flows: Vec<RefusedFlow>,
    scenarios: Vec<Scenario>,
    scenario_omissions: Vec<ScenarioOmission>,
    thresholds: Vec<Threshold>,
    defects: Vec<Defect>,
    receipts: Vec<ResolutionReceipt>,
    proposals: Vec<Proposal>,
    review_events: Vec<ReviewEvent>,
    review_protocol: ReviewProtocol,
    review_commissions: Vec<ReviewCommission>,
    deferred_populations: Vec<DeferredPopulation>,
    stopping_rule: StoppingRule,
    severity_rubric: SeverityRubric,
    functional_criteria: FunctionalCriteria,
    closure_record: RequiredNullable<ClosureProjection>,
    acceptance_gate: AcceptanceGate,
    closure_requirement_profiles: Vec<ClosureRequirementProfile>,
    closure_claim_contracts: Vec<ClosureClaimContract>,
    model_allocations: Vec<ModelAllocation>,
    function_allocations: Vec<FunctionAllocation>,
    loop_hazard_controls: Vec<LoopHazardControl>,
    bottleneck_dispositions: Vec<BottleneckDisposition>,
    scope_audits: Vec<ScopeAudit>,
    constitutional_effects: Vec<ConstitutionalEffect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DefectResolution {
    pub(crate) candidate: bool,
    pub(crate) resolution_status: &'static str,
    pub(crate) blocking: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Check,
    Generate,
    RefreshAndCheck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckResult {
    pub(crate) controls: usize,
    pub(crate) message: String,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ScopeFingerprintOutput {
    source_version: String,
    scope_sha256: String,
}

impl std::fmt::Display for CheckResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Clone, Debug)]
struct Loaded {
    source: LedgerDocument,
    source_bytes: Vec<u8>,
}

pub(crate) struct ValidatedLedger {
    document: LedgerDocument,
    source_bytes: Vec<u8>,
    resolutions: BTreeMap<String, DefectResolution>,
    input_bytes: BTreeMap<String, Vec<u8>>,
    sibling_projections: SiblingProjections,
    reader_projection: reader::ReaderLedgerProjection,
    immutable_snapshot: Option<Mutex<ImmutableRepositoryInputs>>,
}

impl ValidatedLedger {
    pub(crate) fn document(&self) -> &LedgerDocument {
        &self.document
    }

    pub(crate) fn source_bytes(&self) -> &[u8] {
        &self.source_bytes
    }

    pub(crate) fn source_version(&self) -> &str {
        &self.document.source_version
    }

    pub(crate) fn closure(&self) -> Option<&ClosureProjection> {
        self.document.closure_record.0.as_ref()
    }

    pub(crate) fn resolutions(&self) -> &BTreeMap<String, DefectResolution> {
        &self.resolutions
    }

    pub(crate) fn immutable_inputs(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.input_bytes
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
    }

    fn protected_claim_refs_inner(&self) -> LedgerResult<Vec<String>> {
        let effect = self
            .document
            .constitutional_effects
            .iter()
            .find(|row| row.id == "FS-CCE-212")
            .ok_or_else(|| LedgerError::new("full-society ledger has no FS-CCE-212 row"))?;
        unique_strings(
            &effect.affected_claim_refs,
            "FS-CCE-212 affected_claim_refs",
            false,
        )?;
        if effect.affected_claim_refs.len() != 13 {
            return Err(LedgerError::new(
                "FS-CCE-212 protected claim set is not the ledger-derived 13-claim set",
            ));
        }
        Ok(effect.affected_claim_refs.clone())
    }
}

fn ledger_error(message: impl Into<String>) -> Error {
    Error::new(format!("13-full-society-ledger: {}", message.into()))
}

fn nonempty(value: &str, context: &str) -> LedgerResult<()> {
    if value.trim().is_empty() {
        return Err(LedgerError::new(format!(
            "{context} must be a non-empty string"
        )));
    }
    Ok(())
}

fn unique_strings(values: &[String], context: &str, allow_empty: bool) -> LedgerResult<()> {
    if !allow_empty && values.is_empty() {
        return Err(LedgerError::new(format!("{context} must be non-empty")));
    }
    let mut seen = HashSet::new();
    for value in values {
        nonempty(value, context)?;
        if !seen.insert(value.as_str()) {
            return Err(LedgerError::new(format!(
                "{context} must contain unique values"
            )));
        }
    }
    Ok(())
}

fn exact_meanings<const N: usize>(
    actual: &BTreeMap<String, String>,
    expected: [&str; N],
    context: &str,
) -> LedgerResult<()> {
    let actual_keys = actual.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_keys = expected.into_iter().collect::<BTreeSet<_>>();
    if actual_keys != expected_keys {
        return Err(LedgerError::new(format!(
            "{context} must define exactly {:?}",
            expected_keys
        )));
    }
    for (key, meaning) in actual {
        nonempty(meaning, &format!("{context}.{key}"))?;
    }
    Ok(())
}

fn parse_source(bytes: &[u8]) -> LedgerResult<LedgerDocument> {
    // This preflight intentionally materialises generic JSON solely to make
    // duplicate keys observable before serde's typed deserializer runs.
    parse_json_no_duplicates(bytes)
        .map_err(|error| LedgerError::new(format!("{SOURCE} is not valid JSON: {error}")))?;
    serde_json::from_slice(bytes)
        .map_err(|error| LedgerError::new(format!("{SOURCE} typed contract invalid: {error}")))
}

fn load_source(context: &Context) -> LedgerResult<Loaded> {
    let source_bytes = fs::read(context.path(SOURCE))
        .map_err(|error| LedgerError::new(format!("missing reviewed source: {SOURCE}: {error}")))?;
    let source = parse_source(&source_bytes)?;
    Ok(Loaded {
        source,
        source_bytes,
    })
}

fn load_static_inputs(context: &Context) -> LedgerResult<BTreeMap<String, Vec<u8>>> {
    let mut inputs = STATIC_INPUTS
        .into_iter()
        .map(|path| {
            fs::read(context.path(path))
                .map(|bytes| (path.to_owned(), bytes))
                .map_err(|error| LedgerError::new(format!("cannot read input {path}: {error}")))
        })
        .collect::<LedgerResult<BTreeMap<_, _>>>()?;
    for path in [SOURCE, READER_SOURCE] {
        let bytes = inputs
            .get(path)
            .expect("static source input was loaded")
            .clone();
        prime_reference_inputs(context, &mut inputs, &bytes)?;
    }
    Ok(inputs)
}

fn load_static_inputs_snapshotted(
    context: &Context,
    snapshot: &mut ImmutableRepositoryInputs,
) -> LedgerResult<BTreeMap<String, Vec<u8>>> {
    let mut inputs = BTreeMap::new();
    for path in STATIC_INPUTS {
        let bytes = snapshot
            .read_bytes(&context.path(path))
            .map_err(|error| LedgerError::new(error.to_string()))?
            .to_vec();
        inputs.insert(path.to_owned(), bytes);
    }
    for path in [SOURCE, READER_SOURCE] {
        let bytes = inputs
            .get(path)
            .expect("static source input was loaded")
            .clone();
        prime_reference_inputs_snapshotted(context, snapshot, &mut inputs, &bytes)?;
    }
    Ok(inputs)
}

fn json_string_values(bytes: &[u8]) -> LedgerResult<Vec<String>> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'"' {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        let mut escaped = false;
        while index < bytes.len() {
            let byte = bytes[index];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                index += 1;
                result.push(
                    serde_json::from_slice::<String>(&bytes[start..index]).map_err(|error| {
                        LedgerError::new(format!("invalid JSON string encoding: {error}"))
                    })?,
                );
                break;
            }
            index += 1;
        }
        if index > bytes.len() || bytes.get(index.saturating_sub(1)) != Some(&b'"') {
            return Err(LedgerError::new("unterminated JSON string"));
        }
    }
    Ok(result)
}

fn reference_path_candidate(value: &str) -> Option<&str> {
    const RECEIPT_PREFIX: &str = "new-book-plans/verification-receipts/sha256-";
    let path = if let Some((path, _)) = value.split_once("::") {
        path
    } else if value
        .strip_prefix(RECEIPT_PREFIX)
        .and_then(|tail| tail.strip_suffix(".json"))
        .is_some_and(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        value
    } else {
        return None;
    };
    let mut bytes = path.bytes();
    let first = bytes.next()?;
    if !(first.is_ascii_alphanumeric() || matches!(first, b'_' | b'.'))
        || !bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
        || path.starts_with('/')
        || path.split('/').any(|part| part == "..")
    {
        return None;
    }
    Some(path)
}

fn prime_reference_inputs(
    context: &Context,
    inputs: &mut BTreeMap<String, Vec<u8>>,
    source_bytes: &[u8],
) -> LedgerResult<()> {
    for value in json_string_values(source_bytes)? {
        let Some(relative) = reference_path_candidate(&value) else {
            continue;
        };
        if inputs.contains_key(relative) {
            continue;
        }
        let path = context.path(relative);
        if path.is_file() {
            inputs.insert(relative.to_owned(), fs::read(path)?);
        }
    }
    Ok(())
}

fn prime_reference_inputs_snapshotted(
    context: &Context,
    snapshot: &mut ImmutableRepositoryInputs,
    inputs: &mut BTreeMap<String, Vec<u8>>,
    source_bytes: &[u8],
) -> LedgerResult<()> {
    for value in json_string_values(source_bytes)? {
        let Some(relative) = reference_path_candidate(&value) else {
            continue;
        };
        if inputs.contains_key(relative) {
            continue;
        }
        let path = context.path(relative);
        if path.is_file() {
            let bytes = snapshot
                .read_bytes(&path)
                .map_err(|error| LedgerError::new(error.to_string()))?
                .to_vec();
            inputs.insert(relative.to_owned(), bytes);
        }
    }
    Ok(())
}

fn validate_repository_reference(
    inputs: &BTreeMap<String, Vec<u8>>,
    reference: &str,
    context: &str,
) -> LedgerResult<()> {
    if reference.matches("::").count() != 1 {
        return Err(LedgerError::new(format!(
            "{context}: reference must be `path::needle`, got {reference:?}"
        )));
    }
    let (path, needle) = reference
        .split_once("::")
        .expect("exactly one reference delimiter was checked");
    if path.is_empty() || needle.is_empty() {
        return Err(LedgerError::new(format!(
            "{context}: empty path or needle in {reference:?}"
        )));
    }
    if path.starts_with('/') || path.contains("..") || path.contains('\\') {
        return Err(LedgerError::new(format!(
            "{context}: path must be repo-relative: {path:?}"
        )));
    }
    let target = inputs
        .get(path)
        .ok_or_else(|| LedgerError::new(format!("{context}: reference target missing: {path}")))?;
    let target = std::str::from_utf8(target).map_err(|error| {
        LedgerError::new(format!(
            "{context}: reference target is not UTF-8: {path}: {error}"
        ))
    })?;
    let count = target.matches(needle).count();
    if count != 1 {
        return Err(LedgerError::new(format!(
            "{context}: needle must occur exactly once in {path}; found {count}: {needle:?}"
        )));
    }
    Ok(())
}

fn validate_repository_reference_list(
    inputs: &BTreeMap<String, Vec<u8>>,
    references: &[String],
    context: &str,
) -> LedgerResult<()> {
    for (index, reference) in references.iter().enumerate() {
        validate_repository_reference(inputs, reference, &format!("{context}[{index}]"))?;
    }
    Ok(())
}

fn validate_term_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    term: &Term,
    context: &str,
) -> LedgerResult<()> {
    validate_repository_reference_list(inputs, &term.source_refs, &format!("{context}.source_refs"))
}

fn validate_term_set_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    terms: &TermSet,
    context: &str,
) -> LedgerResult<()> {
    for (name, term) in terms {
        validate_term_references(inputs, term, &format!("{context}.{name}"))?;
    }
    Ok(())
}

fn validate_profile_term_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    profiles: &ProfileTerms,
    context: &str,
) -> LedgerResult<()> {
    for (profile, terms) in profiles {
        validate_term_set_references(inputs, terms, &format!("{context}.{profile}"))?;
    }
    Ok(())
}

fn validate_test_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    test: &TestBinding,
    context: &str,
) -> LedgerResult<()> {
    validate_repository_reference_list(
        inputs,
        &test.source_refs,
        &format!("{context}.source_refs"),
    )?;
    if test.status == "executable" {
        let reference = test.executable_ref.0.as_deref().ok_or_else(|| {
            LedgerError::new(format!("{context}: executable test needs executable_ref"))
        })?;
        validate_repository_reference(inputs, reference, &format!("{context}.executable_ref"))?;
    }
    Ok(())
}

fn validate_domain_bucket_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    bucket: &DomainBucket,
    context: &str,
) -> LedgerResult<()> {
    match bucket {
        DomainBucket::Answer(answer) => {
            validate_repository_reference_list(inputs, &answer.refs, &format!("{context}.refs"))
        }
        DomainBucket::Routing(_) => Ok(()),
        DomainBucket::Unresolved(unresolved) => validate_repository_reference(
            inputs,
            &unresolved.unresolved.owner_ref,
            &format!("{context}.unresolved.owner_ref"),
        ),
    }
}

/// Dereference exactly the typed fields the ledger checker treats as `path::needle`
/// evidence. Other strings containing `::` (coverage artifact identifiers,
/// command text, and internal tokens) deliberately remain syntactic values.
fn validate_typed_repository_references(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    validate_repository_reference(
        inputs,
        &source.power_source_inventory.owner_ref,
        "power_source_inventory.owner_ref",
    )?;

    for domain in &source.domains {
        let context = format!("domains.{}", domain.id);
        validate_repository_reference(inputs, &domain.owner_ref, &format!("{context}.owner_ref"))?;
        for (name, bucket) in [
            (
                "constitutional_invariants",
                &domain.constitutional_invariants,
            ),
            ("ordinary_law_choices", &domain.ordinary_law_choices),
            ("protected_private_civic", &domain.protected_private_civic),
            ("book2_operations", &domain.book2_operations),
            (
                "external_assumptions_note",
                &domain.external_assumptions_note,
            ),
        ] {
            validate_domain_bucket_references(inputs, bucket, &format!("{context}.{name}"))?;
        }
        validate_repository_reference_list(
            inputs,
            &domain.source_refs,
            &format!("{context}.source_refs"),
        )?;
    }
    for row in &source.legacy_rows {
        validate_repository_reference(
            inputs,
            &row.source_ref,
            &format!("legacy_rows.{}.source_ref", row.id),
        )?;
    }
    for claim in &source.claims {
        validate_repository_reference(
            inputs,
            &claim.owner_ref,
            &format!("claims.{}.owner_ref", claim.id),
        )?;
    }

    for role in &source.roles {
        let context = format!("roles.{}", role.id);
        validate_repository_reference(inputs, &role.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference_list(
            inputs,
            &role.formal_anchor.refs,
            &format!("{context}.formal_anchor.refs"),
        )?;
        validate_repository_reference_list(
            inputs,
            &role.source_refs,
            &format!("{context}.source_refs"),
        )?;
        if let Some(power) = &role.power_held {
            validate_repository_reference(
                inputs,
                &power.source_ref,
                &format!("{context}.power_held.source_ref"),
            )?;
        }
    }
    for (index, omission) in source.role_omissions.iter().enumerate() {
        if let RoleOmission::Role(omission) = omission {
            validate_repository_reference(
                inputs,
                &omission.source_ref,
                &format!("role_omissions[{index}].source_ref"),
            )?;
        }
    }

    for body in &source.bodies {
        let context = format!("bodies.{}", body.id);
        validate_repository_reference(inputs, &body.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference(inputs, &body.source_ref, &format!("{context}.source_ref"))?;
        validate_repository_reference_list(
            inputs,
            &body.source_refs,
            &format!("{context}.source_refs"),
        )?;
        for (name, term) in [
            (
                "universal_human_standing",
                &body.status_senses.universal_human_standing,
            ),
            (
                "political_membership",
                &body.status_senses.political_membership,
            ),
            ("franchise", &body.status_senses.franchise),
            ("candidacy", &body.status_senses.candidacy),
            ("current_office", &body.status_senses.current_office),
            (
                "current_lawful_power",
                &body.status_senses.current_lawful_power,
            ),
            (
                "permanent_historical_public_answerability",
                &body.status_senses.permanent_historical_public_answerability,
            ),
        ] {
            validate_term_references(inputs, term, &format!("{context}.status_senses.{name}"))?;
        }
        for (name, term) in [
            ("democratic_source", &body.office_contract.democratic_source),
            ("jurisdiction", &body.office_contract.jurisdiction),
            ("ordinary_function", &body.office_contract.ordinary_function),
            (
                "delegation_boundary",
                &body.office_contract.delegation_boundary,
            ),
            (
                "conflict_and_recusal",
                &body.office_contract.conflict_and_recusal,
            ),
            ("appointment", &body.office_contract.appointment),
            ("removal", &body.office_contract.removal),
            ("succession", &body.office_contract.succession),
            ("temporal_status", &body.office_contract.temporal_status),
            (
                "public_reason_duty",
                &body.office_contract.public_reason_duty,
            ),
            ("anti_capture", &body.office_contract.anti_capture),
        ] {
            validate_term_references(inputs, term, &format!("{context}.office_contract.{name}"))?;
        }
        for (index, route) in body.accountability_routes.iter().enumerate() {
            validate_term_references(
                inputs,
                &route.term,
                &format!("{context}.accountability_routes[{index}].term"),
            )?;
        }
        validate_term_references(
            inputs,
            &body.adverse_determinations.note,
            &format!("{context}.adverse_determinations.note"),
        )?;
        for (index, item) in body.adverse_determinations.items.iter().enumerate() {
            validate_term_references(
                inputs,
                &item.appeal,
                &format!("{context}.adverse_determinations.items[{index}].appeal"),
            )?;
            validate_term_references(
                inputs,
                &item.remedy,
                &format!("{context}.adverse_determinations.items[{index}].remedy"),
            )?;
        }
        validate_term_references(
            inputs,
            &body.temporal_contract.term,
            &format!("{context}.temporal_contract.term"),
        )?;
        validate_term_references(
            inputs,
            &body.temporal_contract.failure_polarity,
            &format!("{context}.temporal_contract.failure_polarity"),
        )?;
        validate_term_references(
            inputs,
            &body.temporal_contract.expiry_default,
            &format!("{context}.temporal_contract.expiry_default"),
        )?;
        for (index, term) in body.delegated_mechanics.iter().enumerate() {
            validate_term_references(
                inputs,
                term,
                &format!("{context}.delegated_mechanics[{index}]"),
            )?;
        }
    }

    for power in &source.powers {
        let context = format!("powers.{}", power.id);
        validate_repository_reference(inputs, &power.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference(
            inputs,
            &power.book2_owner_ref,
            &format!("{context}.book2_owner_ref"),
        )?;
        validate_repository_reference_list(
            inputs,
            &power.source_refs,
            &format!("{context}.source_refs"),
        )?;
        validate_term_set_references(
            inputs,
            &power.contract_terms,
            &format!("{context}.contract_terms"),
        )?;
        validate_profile_term_references(
            inputs,
            &power.profile_terms,
            &format!("{context}.profile_terms"),
        )?;
        validate_term_references(
            inputs,
            &power.evidence_authority,
            &format!("{context}.evidence_authority"),
        )?;
        validate_test_references(
            inputs,
            &power.negative_test,
            &format!("{context}.negative_test"),
        )?;
        validate_test_references(
            inputs,
            &power.counterfactual,
            &format!("{context}.counterfactual"),
        )?;
    }
    for effect in &source.constitutional_effects {
        let context = format!("constitutional_effects.{}", effect.id);
        validate_repository_reference(inputs, &effect.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference_list(
            inputs,
            &effect.source_refs,
            &format!("{context}.source_refs"),
        )?;
        validate_term_set_references(
            inputs,
            &effect.contract_terms,
            &format!("{context}.contract_terms"),
        )?;
        validate_profile_term_references(
            inputs,
            &effect.profile_terms,
            &format!("{context}.profile_terms"),
        )?;
        validate_term_references(
            inputs,
            &effect.evidence_authority,
            &format!("{context}.evidence_authority"),
        )?;
        validate_test_references(
            inputs,
            &effect.negative_test,
            &format!("{context}.negative_test"),
        )?;
        validate_test_references(
            inputs,
            &effect.counterfactual,
            &format!("{context}.counterfactual"),
        )?;
    }
    for template in &source.power_contract_templates {
        let context = format!("power_contract_templates.{}", template.id);
        validate_repository_reference(
            inputs,
            &template.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference_list(
            inputs,
            &template.source_refs,
            &format!("{context}.source_refs"),
        )?;
        validate_term_set_references(
            inputs,
            &template.contract_terms,
            &format!("{context}.contract_terms"),
        )?;
    }
    for refusal in &source.power_refusals {
        let context = format!("power_refusals.{}", refusal.id);
        validate_repository_reference(inputs, &refusal.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference_list(
            inputs,
            &refusal.source_refs,
            &format!("{context}.source_refs"),
        )?;
    }
    for crosswalk in &source.power_crosswalk_dispositions {
        let context = format!("power_crosswalk_dispositions.{}", crosswalk.id);
        validate_repository_reference(
            inputs,
            &crosswalk.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference(
            inputs,
            &crosswalk.transition_owner_ref,
            &format!("{context}.transition_owner_ref"),
        )?;
        validate_repository_reference_list(
            inputs,
            &crosswalk.source_refs,
            &format!("{context}.source_refs"),
        )?;
    }
    for gap in &source.power_population.resolved_allocation_gaps {
        validate_repository_reference_list(
            inputs,
            &gap.source_refs,
            "power_population.resolved_allocation_gaps.source_refs",
        )?;
    }
    for family in &source.coverage_families {
        validate_repository_reference_list(
            inputs,
            &family.source_refs,
            &format!("coverage_families.{}.source_refs", family.id),
        )?;
    }
    for allocation in &source.function_allocations {
        let context = format!("function_allocations.{}", allocation.id);
        validate_repository_reference_list(
            inputs,
            &allocation.source_refs,
            &format!("{context}.source_refs"),
        )?;
        for (index, constraint) in allocation.separation_constraints.iter().enumerate() {
            validate_repository_reference(
                inputs,
                &constraint.source_ref,
                &format!("{context}.separation_constraints[{index}].source_ref"),
            )?;
        }
    }

    for dependency in &source.dependencies {
        let context = format!("dependencies.{}", dependency.id);
        validate_repository_reference(
            inputs,
            &dependency.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference_list(
            inputs,
            &dependency.source_refs,
            &format!("{context}.source_refs"),
        )?;
        if let AlternateRoute::Present(alternate) = &dependency.alternate_route {
            validate_repository_reference(
                inputs,
                &alternate.source_ref,
                &format!("{context}.alternate_route.source_ref"),
            )?;
        }
    }
    for loop_row in &source.dependency_loops {
        validate_repository_reference(
            inputs,
            &loop_row.owner_ref,
            &format!("dependency_loops.{}.owner_ref", loop_row.id),
        )?;
    }
    for (index, flow) in source.refused_flows.iter().enumerate() {
        validate_repository_reference(
            inputs,
            &flow.source_ref,
            &format!("refused_flows[{index}].source_ref"),
        )?;
    }
    for scenario in &source.scenarios {
        let context = format!("scenarios.{}", scenario.id);
        validate_repository_reference(
            inputs,
            &scenario.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference_list(
            inputs,
            &scenario.source_refs,
            &format!("{context}.source_refs"),
        )?;
    }
    for (index, omission) in source.scenario_omissions.iter().enumerate() {
        validate_repository_reference(
            inputs,
            omission.source_ref(),
            &format!("scenario_omissions[{index}].source_ref"),
        )?;
    }
    for route in &source.routes {
        let context = format!("routes.{}", route.id);
        validate_repository_reference(inputs, &route.owner_ref, &format!("{context}.owner_ref"))?;
        validate_repository_reference(inputs, &route.source_ref, &format!("{context}.source_ref"))?;
    }
    for assumption in &source.external_assumptions {
        let context = format!("external_assumptions.{}", assumption.id);
        validate_repository_reference(
            inputs,
            &assumption.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference(
            inputs,
            &assumption.source_ref,
            &format!("{context}.source_ref"),
        )?;
    }
    for envelope in &source.envelope {
        let context = format!("envelope.{}", envelope.id);
        validate_repository_reference(
            inputs,
            &envelope.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        if let Some(fields) = &envelope.fields {
            for (index, field) in fields.iter().enumerate() {
                validate_repository_reference(
                    inputs,
                    &field.book2_owner_ref,
                    &format!("{context}.fields[{index}].book2_owner_ref"),
                )?;
            }
        }
    }
    for criterion in &source.functional_criteria.criteria {
        validate_repository_reference_list(
            inputs,
            &criterion.binding_refs,
            &format!("functional_criteria.{}.binding_refs", criterion.id),
        )?;
    }
    for threshold in &source.thresholds {
        let context = format!("thresholds.{}", threshold.id);
        validate_repository_reference(
            inputs,
            &threshold.owner_ref,
            &format!("{context}.owner_ref"),
        )?;
        validate_repository_reference(
            inputs,
            &threshold.binding_ref,
            &format!("{context}.binding_ref"),
        )?;
        validate_repository_reference(
            inputs,
            &threshold.decision_owner_ref,
            &format!("{context}.decision_owner_ref"),
        )?;
        validate_repository_reference(
            inputs,
            &threshold.measurement_owner_ref,
            &format!("{context}.measurement_owner_ref"),
        )?;
    }
    for defect in &source.defects {
        let context = format!("defects.{}", defect.id);
        validate_repository_reference(inputs, &defect.owner_ref, &format!("{context}.owner_ref"))?;
        if let Some(reference) = &defect.controls.reintroduction_control_ref {
            validate_repository_reference(
                inputs,
                reference,
                &format!("{context}.controls.reintroduction_control_ref"),
            )?;
        }
        if let Some(reference) = &defect.controls.initiation_control_ref {
            validate_repository_reference(
                inputs,
                reference,
                &format!("{context}.controls.initiation_control_ref"),
            )?;
        }
        if let Some(references) = &defect.controls.containment_control_refs {
            validate_repository_reference_list(
                inputs,
                references,
                &format!("{context}.controls.containment_control_refs"),
            )?;
        }
        if let Some(fields) = &defect.controls.recovery_fields {
            validate_repository_reference(
                inputs,
                &fields.evidence_ref,
                &format!("{context}.controls.recovery_fields.evidence_ref"),
            )?;
        }
    }
    for receipt in &source.receipts {
        let context = format!("receipts.{}", receipt.id);
        for (name, reference) in [
            ("proof_ref", &receipt.proof_ref),
            ("negative_control_ref", &receipt.negative_control_ref),
            ("reader_mapping_ref", &receipt.reader_mapping_ref),
            ("owner_ref", &receipt.owner_ref),
        ] {
            validate_repository_reference(inputs, reference, &format!("{context}.{name}"))?;
        }
    }
    validate_repository_reference(
        inputs,
        &source.stopping_rule.source_ref,
        "stopping_rule.source_ref",
    )?;
    for row in &source.deferred_populations {
        validate_repository_reference(
            inputs,
            &row.owner_ref,
            &format!("deferred_populations.{}.owner_ref", row.record_type),
        )?;
    }
    validate_repository_reference(
        inputs,
        &source.review_protocol.policy_basis,
        "review_protocol.policy_basis",
    )?;
    validate_repository_reference(
        inputs,
        &source.review_protocol.protocol_ref,
        "review_protocol.protocol_ref",
    )?;
    validate_repository_reference(
        inputs,
        &source.review_protocol.status_line_ref,
        "review_protocol.status_line_ref",
    )?;
    for audit in &source.scope_audits {
        if let Some(reference) = &audit.policy_basis {
            validate_repository_reference(
                inputs,
                reference,
                &format!("scope_audits.{}.policy_basis", audit.id),
            )?;
        }
    }
    for proposal in &source.proposals {
        if let Some(binding) = &proposal.retained_limit_binding {
            validate_repository_reference(
                inputs,
                &binding.owner_ref,
                &format!("proposals.{}.retained_limit_binding.owner_ref", proposal.id),
            )?;
        }
    }
    if let Some(closure) = &source.closure_record.0 {
        validate_repository_reference_list(
            inputs,
            &closure.assurance_record_refs,
            "closure_record.assurance_record_refs",
        )?;
        validate_repository_reference(
            inputs,
            &closure.closure_policy_ref,
            "closure_record.closure_policy_ref",
        )?;
    }
    Ok(())
}

fn input_bytes<'a>(inputs: &'a BTreeMap<String, Vec<u8>>, path: &str) -> LedgerResult<&'a [u8]> {
    inputs
        .get(path)
        .map(Vec::as_slice)
        .ok_or_else(|| LedgerError::new(format!("immutable input was not loaded: {path}")))
}

fn parse_typed_input<T: for<'de> Deserialize<'de>>(
    inputs: &BTreeMap<String, Vec<u8>>,
    path: &str,
) -> LedgerResult<T> {
    let bytes = input_bytes(inputs, path)?;
    parse_json_no_duplicates(bytes)
        .map_err(|error| LedgerError::new(format!("{path} is not valid JSON: {error}")))?;
    serde_json::from_slice(bytes)
        .map_err(|error| LedgerError::new(format!("{path} typed projection invalid: {error}")))
}

fn load_reader_projection(
    context: &Context,
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<reader::ReaderLedgerProjection> {
    reader::load_validated_reader_evidence(
        context,
        reader::InputSnapshot {
            source_json: Some(input_bytes(inputs, READER_SOURCE)?),
            generated_report: None,
            protocol_decision: Some(input_bytes(inputs, READER_PROTOCOL_DECISION)?),
        },
    )
    .map_err(|error| LedgerError::new(format!("reader-evidence contract invalid: {error}")))
}

struct SiblingProjections {
    assertion: AssertionSiblingProjection,
    assurance: AssuranceSiblingProjection,
    red_team: RedTeamSiblingProjection,
    amendment: AmendmentSiblingProjection,
    placement: PlacementSiblingProjection,
    temporal: TemporalSiblingProjection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewHistoryProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: IgnoredAny,
    evidence_role: IgnoredAny,
    source_version: IgnoredAny,
    bound_sources_sha256: IgnoredAny,
    axes: IgnoredAny,
    scope_disposition_meanings: IgnoredAny,
    gate_applicability_meanings: IgnoredAny,
    routing_marker_meanings: IgnoredAny,
    posture_meanings: IgnoredAny,
    unestablished_disposition_meanings: IgnoredAny,
    evidence_kind_meanings: IgnoredAny,
    overlay_meanings: IgnoredAny,
    route_status_meanings: IgnoredAny,
    defect_disposition_meanings: IgnoredAny,
    response_stage_meanings: IgnoredAny,
    resolution_status_meanings: IgnoredAny,
    proposal_disposition_meanings: IgnoredAny,
    envelope_status_meanings: IgnoredAny,
    value_status_meanings: IgnoredAny,
    lawful_source_meanings: IgnoredAny,
    role_kind_meanings: IgnoredAny,
    scale_meanings: IgnoredAny,
    power_position_meanings: IgnoredAny,
    role_anchor_meanings: IgnoredAny,
    flow_kind_meanings: IgnoredAny,
    dependency_class_meanings: IgnoredAny,
    loop_kind_meanings: IgnoredAny,
    lifecycle_path_meanings: IgnoredAny,
    scenario_kind_meanings: IgnoredAny,
    collision_axis_meanings: IgnoredAny,
    shock_kind_meanings: IgnoredAny,
    protected_sphere_form_meanings: IgnoredAny,
    compatibility_table: IgnoredAny,
    enum_mapping: IgnoredAny,
    enum_mapping_exclusions: IgnoredAny,
    residual_coverage_exclusions: IgnoredAny,
    id_registry: IgnoredAny,
    domains: IgnoredAny,
    legacy_rows: IgnoredAny,
    claims: IgnoredAny,
    bodies: IgnoredAny,
    routes: IgnoredAny,
    external_assumptions: IgnoredAny,
    envelope: IgnoredAny,
    roles: IgnoredAny,
    role_omissions: IgnoredAny,
    power_source_inventory: IgnoredAny,
    power_population: IgnoredAny,
    coverage_population: IgnoredAny,
    powers: IgnoredAny,
    economic_power_rule_contracts: Option<IgnoredAny>,
    economic_carry_rule_contracts: Option<IgnoredAny>,
    economic_acceptance_cases: Option<IgnoredAny>,
    power_contract_templates: IgnoredAny,
    power_refusals: IgnoredAny,
    power_crosswalk_dispositions: IgnoredAny,
    coverage_families: IgnoredAny,
    dependencies: IgnoredAny,
    dependency_loops: IgnoredAny,
    refused_flows: IgnoredAny,
    scenarios: IgnoredAny,
    scenario_omissions: IgnoredAny,
    thresholds: IgnoredAny,
    defects: IgnoredAny,
    receipts: IgnoredAny,
    proposals: Vec<Proposal>,
    review_events: Vec<ReviewEvent>,
    review_protocol: IgnoredAny,
    review_commissions: Vec<ReviewCommission>,
    deferred_populations: IgnoredAny,
    stopping_rule: IgnoredAny,
    severity_rubric: IgnoredAny,
    functional_criteria: IgnoredAny,
    closure_record: IgnoredAny,
    acceptance_gate: IgnoredAny,
    closure_requirement_profiles: IgnoredAny,
    closure_claim_contracts: IgnoredAny,
    model_allocations: IgnoredAny,
    function_allocations: IgnoredAny,
    loop_hazard_controls: IgnoredAny,
    bottleneck_dispositions: IgnoredAny,
    scope_audits: Vec<ScopeAudit>,
    constitutional_effects: IgnoredAny,
}

#[derive(Clone, Default)]
struct ReviewHistoryState {
    review_commissions: Vec<ReviewCommission>,
    proposals: Vec<Proposal>,
    review_events: Vec<ReviewEvent>,
    scope_audits: Vec<ScopeAudit>,
}

impl From<ReviewHistoryProjection> for ReviewHistoryState {
    fn from(projection: ReviewHistoryProjection) -> Self {
        Self {
            review_commissions: projection.review_commissions,
            proposals: projection.proposals,
            review_events: projection.review_events,
            scope_audits: projection.scope_audits,
        }
    }
}

impl ReviewHistoryState {
    #[cfg(test)]
    fn from_document(source: &LedgerDocument) -> Self {
        Self {
            review_commissions: source.review_commissions.clone(),
            proposals: source.proposals.clone(),
            review_events: source.review_events.clone(),
            scope_audits: source.scope_audits.clone(),
        }
    }
}

fn prior_review_state(context: &Context) -> LedgerResult<ReviewHistoryState> {
    let changed = Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--", SOURCE])
        .current_dir(context.root())
        .status()
        .map_err(|error| LedgerError::new(format!("cannot inspect review history: {error}")))?;
    let revision = match changed.code() {
        Some(0) => "HEAD^",
        Some(1) => "HEAD",
        _ => {
            return Err(LedgerError::new(
                "cannot determine the visible first-parent review predecessor",
            ));
        }
    };
    let object = format!("{revision}:{SOURCE}");
    let shown = Command::new("git")
        .args(["show", &object])
        .current_dir(context.root())
        .output()
        .map_err(|error| LedgerError::new(format!("cannot inspect review history: {error}")))?;
    if !shown.status.success() {
        return Ok(ReviewHistoryState::default());
    }
    parse_json_no_duplicates(&shown.stdout).map_err(|error| {
        LedgerError::new(format!(
            "visible first-parent ledger source is not valid JSON: {error}"
        ))
    })?;
    serde_json::from_slice::<ReviewHistoryProjection>(&shown.stdout)
        .map(ReviewHistoryState::from)
        .map_err(|error| {
            LedgerError::new(format!(
                "visible first-parent ledger source has an invalid typed review-history contract: {error}"
            ))
        })
}

fn require_append_only_prefix<T: PartialEq>(
    name: &str,
    previous: &[T],
    current: &[T],
) -> LedgerResult<()> {
    if current.get(..previous.len()) != Some(previous) {
        return Err(LedgerError::new(format!(
            "{name}: visible first-parent history must remain an exact append-only prefix; failed and stale records cannot be deleted or rewritten"
        )));
    }
    Ok(())
}

fn validate_review_history_against(
    previous: &ReviewHistoryState,
    current: &LedgerDocument,
) -> LedgerResult<()> {
    require_append_only_prefix(
        "review_commissions",
        &previous.review_commissions,
        &current.review_commissions,
    )?;
    require_append_only_prefix("proposals", &previous.proposals, &current.proposals)?;
    require_append_only_prefix(
        "review_events",
        &previous.review_events,
        &current.review_events,
    )?;
    require_append_only_prefix(
        "scope_audits",
        &previous.scope_audits,
        &current.scope_audits,
    )
}

fn validate_review_history(context: &Context, source: &LedgerDocument) -> LedgerResult<()> {
    let previous = prior_review_state(context)?;
    validate_review_history_against(&previous, source)
}

impl SiblingProjections {
    fn parse(inputs: &BTreeMap<String, Vec<u8>>) -> LedgerResult<Self> {
        Ok(Self {
            assertion: parse_typed_input(inputs, ASSERTION_SOURCE)?,
            assurance: parse_typed_input(inputs, ASSURANCE_SOURCE)?,
            red_team: parse_typed_input(inputs, RED_TEAM_SOURCE)?,
            amendment: parse_typed_input(inputs, AMENDMENT_SOURCE)?,
            placement: parse_typed_input(inputs, PLACEMENT_SOURCE)?,
            temporal: parse_typed_input(inputs, TEMPORAL_SOURCE)?,
        })
    }

    fn enum_rows(&self) -> BTreeSet<(String, String, String)> {
        let mut rows = BTreeSet::new();
        let mut meanings = |file: &str, field: &str, values: &BTreeMap<String, IgnoredAny>| {
            rows.extend(
                values
                    .keys()
                    .map(|value| (file.to_owned(), field.to_owned(), value.clone())),
            );
        };
        meanings(
            "assertion-surface-contracts.json",
            "risk_disposition_meanings",
            &self.assertion.risk_disposition_meanings,
        );
        meanings(
            "record-integrity-assurance-case.json",
            "status_meanings",
            &self.assurance.status_meanings,
        );
        meanings(
            "record-integrity-red-team.json",
            "posture_meanings",
            &self.red_team.posture_meanings,
        );
        meanings(
            "amendment-semantics-audit.json",
            "label_verdict_meanings",
            &self.amendment.label_verdict_meanings,
        );
        drop(meanings);
        let mut leaf = |file: &str, field: &str, value: &str| {
            rows.insert((file.to_owned(), field.to_owned(), value.to_owned()));
        };
        for row in &self.assurance.claims {
            leaf(
                "record-integrity-assurance-case.json",
                "posture",
                &row.posture,
            );
        }
        for row in &self.assurance.defeaters {
            leaf(
                "record-integrity-assurance-case.json",
                "disposition",
                &row.disposition,
            );
        }
        leaf(
            "record-integrity-red-team.json",
            "status",
            &self.red_team.status,
        );
        leaf(
            "amendment-semantics-audit.json",
            "status",
            &self.amendment.status,
        );
        for row in &self.amendment.cases {
            leaf(
                "amendment-semantics-audit.json",
                "verdict",
                &row.declared_label.verdict,
            );
        }
        leaf(
            "placement-exhaustiveness-audit.json",
            "status",
            &self.placement.status,
        );
        leaf(
            "temporal-assurance-case.json",
            "status",
            &self.temporal.status,
        );
        for row in &self.temporal.attacks {
            leaf("temporal-assurance-case.json", "posture", &row.posture);
        }
        rows
    }

    fn residuals(&self) -> BTreeSet<String> {
        let mut pool = BTreeSet::new();
        pool.extend(
            self.assurance
                .claims
                .iter()
                .filter(|row| row.posture != "current_verified")
                .map(|row| format!("record-integrity-assurance-case#{}", row.id)),
        );
        pool.extend(
            self.assurance
                .defeaters
                .iter()
                .map(|row| format!("record-integrity-assurance-case#{}", row.id)),
        );
        pool.extend(
            self.assurance
                .limitations
                .keys()
                .map(|key| format!("record-integrity-assurance-case#limitations.{key}")),
        );
        pool.extend(
            self.red_team
                .scenarios
                .iter()
                .map(|row| format!("record-integrity-red-team#{}", row.id)),
        );
        pool.extend(
            self.red_team
                .observational_equivalence
                .iter()
                .map(|row| format!("record-integrity-red-team#{}", row.id)),
        );
        pool.extend(
            self.red_team
                .limits
                .keys()
                .map(|key| format!("record-integrity-red-team#limits.{key}")),
        );
        pool.extend(
            self.temporal
                .attacks
                .iter()
                .filter(|row| row.posture == "exposed_external_boundary")
                .map(|row| format!("temporal-assurance-case#attacks.{}", row.id)),
        );
        pool.extend(
            self.temporal
                .limits
                .keys()
                .map(|key| format!("temporal-assurance-case#limits.{key}")),
        );
        pool.extend(
            self.placement
                .limits
                .keys()
                .map(|key| format!("placement-exhaustiveness-audit#limits.{key}")),
        );
        pool.extend(
            self.amendment
                .limits
                .keys()
                .map(|key| format!("amendment-semantics-audit#limits.{key}")),
        );
        pool.extend(
            self.assertion
                .premises
                .iter()
                .filter(|(_, row)| {
                    row.risk_dispositions
                        .iter()
                        .any(|value| value == "deliberately_refused")
                })
                .map(|(key, _)| format!("assertion-surface-contracts#premises.{key}")),
        );
        pool
    }

    fn bounded_witnesses(&self) -> BTreeSet<String> {
        self.red_team
            .scenarios
            .iter()
            .map(|row| format!("record-integrity-red-team#{}", row.id))
            .chain(
                self.temporal
                    .cases
                    .iter()
                    .map(|row| format!("temporal-assurance-case#{}", row.id)),
            )
            .chain(
                self.amendment
                    .cases
                    .iter()
                    .map(|row| format!("amendment-semantics-audit#{}", row.id)),
            )
            .chain(
                self.placement
                    .mutations
                    .iter()
                    .map(|row| format!("placement-exhaustiveness-audit#{}", row.id)),
            )
            .collect()
    }
}

fn validate_sibling_closures(
    source: &LedgerDocument,
    siblings: &SiblingProjections,
    reader: &reader::ReaderLedgerProjection,
) -> LedgerResult<()> {
    let reviewed_files = [
        "assertion-surface-contracts.json",
        "record-integrity-assurance-case.json",
        "record-integrity-red-team.json",
        "amendment-semantics-audit.json",
        "placement-exhaustiveness-audit.json",
        "temporal-assurance-case.json",
        "reader-evidence.json",
    ];
    let mapped = source
        .enum_mapping
        .iter()
        .filter(|row| reviewed_files.contains(&row.source_file.as_str()))
        .map(|row| {
            nonempty(&row.source_file, "enum_mapping.source_file")?;
            nonempty(&row.field, "enum_mapping.field")?;
            nonempty(&row.value, "enum_mapping.value")?;
            nonempty(&row.canonical, "enum_mapping.canonical")?;
            nonempty(&row.note, "enum_mapping.note")?;
            Ok((
                row.source_file.clone(),
                row.field.clone(),
                row.value.clone(),
            ))
        })
        .collect::<LedgerResult<BTreeSet<_>>>()?;
    let excluded = source
        .enum_mapping_exclusions
        .iter()
        .filter(|row| reviewed_files.contains(&row.source_file.as_str()))
        .map(|row| {
            nonempty(&row.source_file, "enum_mapping_exclusions.source_file")?;
            nonempty(&row.field, "enum_mapping_exclusions.field")?;
            nonempty(&row.value, "enum_mapping_exclusions.value")?;
            nonempty(&row.reason, "enum_mapping_exclusions.reason")?;
            Ok((
                row.source_file.clone(),
                row.field.clone(),
                row.value.clone(),
            ))
        })
        .collect::<LedgerResult<BTreeSet<_>>>()?;
    let mut live = siblings.enum_rows();
    live.extend(reader.enum_inventory.iter().map(|entry| {
        (
            "reader-evidence.json".to_owned(),
            entry.field.to_owned(),
            entry.value.clone(),
        )
    }));
    let uncovered = live
        .difference(&mapped)
        .filter(|row| !excluded.contains(*row))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    if !uncovered.is_empty() {
        return Err(LedgerError::new(format!(
            "reviewed enum values with no mapping row (map them mechanically in the same change): {uncovered:?}"
        )));
    }
    let stale = mapped
        .union(&excluded)
        .filter(|row| !live.contains(*row))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    if !stale.is_empty() {
        return Err(LedgerError::new(format!(
            "enum mapping names values no sibling source declares: {stale:?}"
        )));
    }

    let pool = siblings.residuals();
    let cited = source
        .defects
        .iter()
        .flat_map(|row| row.residual_citations.iter().cloned())
        .collect::<BTreeSet<_>>();
    let excluded = source
        .residual_coverage_exclusions
        .iter()
        .map(|row| {
            nonempty(&row.source_file, "residual_coverage_exclusions.source_file")?;
            nonempty(&row.token, "residual_coverage_exclusions.token")?;
            nonempty(&row.reason, "residual_coverage_exclusions.reason")?;
            Ok(row.token.clone())
        })
        .collect::<LedgerResult<BTreeSet<_>>>()?;
    let uncovered = pool
        .difference(&cited)
        .filter(|token| !excluded.contains(*token))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    if !uncovered.is_empty() {
        return Err(LedgerError::new(format!(
            "sibling residuals uncovered by any defect row (cite or exclude with a reason, in the same change): {uncovered:?}"
        )));
    }
    let stale = cited
        .union(&excluded)
        .filter(|token| !pool.contains(*token))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    if !stale.is_empty() {
        return Err(LedgerError::new(format!(
            "residual citations or exclusions name tokens no sibling source declares: {stale:?}"
        )));
    }

    let witnesses = siblings.bounded_witnesses();
    for scenario in &source.scenarios {
        if let Some(references) = &scenario.bounded_witness_refs {
            if references
                .iter()
                .any(|reference| !witnesses.contains(reference))
            {
                return Err(LedgerError::new(format!(
                    "{}: bounded witness is not a live sibling case",
                    scenario.id
                )));
            }
        }
    }
    Ok(())
}

fn validate_header(source: &LedgerDocument) -> LedgerResult<()> {
    if source.spdx != "CC-BY-4.0" {
        return Err(LedgerError::new(
            "reviewed source must declare spdx CC-BY-4.0",
        ));
    }
    if source.schema_version != EXPECTED_SCHEMA_VERSION {
        return Err(LedgerError::new(format!(
            "schema_version must be the integer {EXPECTED_SCHEMA_VERSION}"
        )));
    }
    nonempty(&source.title, "header.title")?;
    nonempty(&source.source_version, "header.source_version")?;
    if source.status != EXPECTED_STATUS {
        return Err(LedgerError::new(format!(
            "status must be {EXPECTED_STATUS}"
        )));
    }
    if source.evidence_role != "reviewed_inventory_not_assurance" {
        return Err(LedgerError::new(
            "evidence_role must be reviewed_inventory_not_assurance",
        ));
    }
    Ok(())
}

fn validate_bound_sources(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    for (label, path, expected) in [
        (
            "assurance_portfolio",
            ASSURANCE_DECISION,
            &source.bound_sources_sha256.assurance_portfolio,
        ),
        (
            "full_society_boundary",
            BOUNDARY_DECISION,
            &source.bound_sources_sha256.full_society_boundary,
        ),
    ] {
        let body = input_bytes(inputs, path)?;
        let actual = sha256(body);
        if &actual != expected {
            return Err(LedgerError::new(format!(
                "bound source `{label}` ({path}) digest mismatch: reviewed {}… actual {}… — re-review the ruling change, then refresh without --check",
                &expected[..expected.len().min(12)],
                &actual[..12]
            )));
        }
    }
    Ok(())
}

fn validate_meanings(source: &LedgerDocument) -> LedgerResult<()> {
    exact_meanings(
        &source.scope_disposition_meanings,
        SCOPE_DISPOSITIONS,
        "scope_disposition_meanings",
    )?;
    exact_meanings(
        &source.gate_applicability_meanings,
        GATE_REFS,
        "gate_applicability_meanings",
    )?;
    exact_meanings(
        &source.routing_marker_meanings,
        ROUTING_MARKERS,
        "routing_marker_meanings",
    )?;
    exact_meanings(&source.posture_meanings, POSTURES, "posture_meanings")?;
    exact_meanings(
        &source.unestablished_disposition_meanings,
        UNESTABLISHED_DISPOSITIONS,
        "unestablished_disposition_meanings",
    )?;
    exact_meanings(
        &source.evidence_kind_meanings,
        EVIDENCE_KINDS,
        "evidence_kind_meanings",
    )?;
    exact_meanings(&source.overlay_meanings, OVERLAYS, "overlay_meanings")?;
    exact_meanings(
        &source.route_status_meanings,
        ROUTE_STATUSES,
        "route_status_meanings",
    )?;
    exact_meanings(
        &source.defect_disposition_meanings,
        DEFECT_DISPOSITIONS,
        "defect_disposition_meanings",
    )?;
    exact_meanings(
        &source.response_stage_meanings,
        RESPONSE_STAGES,
        "response_stage_meanings",
    )?;
    exact_meanings(
        &source.resolution_status_meanings,
        RESOLUTION_STATUSES,
        "resolution_status_meanings",
    )?;
    exact_meanings(
        &source.proposal_disposition_meanings,
        PROPOSAL_DISPOSITIONS,
        "proposal_disposition_meanings",
    )?;
    exact_meanings(
        &source.envelope_status_meanings,
        ENVELOPE_STATUSES,
        "envelope_status_meanings",
    )?;
    exact_meanings(
        &source.value_status_meanings,
        VALUE_STATUSES,
        "value_status_meanings",
    )?;
    exact_meanings(
        &source.lawful_source_meanings,
        LAWFUL_SOURCES,
        "lawful_source_meanings",
    )?;
    exact_meanings(&source.role_kind_meanings, ROLE_KINDS, "role_kind_meanings")?;
    exact_meanings(&source.scale_meanings, ROLE_SCALES, "scale_meanings")?;
    exact_meanings(
        &source.power_position_meanings,
        POWER_POSITIONS,
        "power_position_meanings",
    )?;
    exact_meanings(
        &source.role_anchor_meanings,
        ROLE_ANCHORS,
        "role_anchor_meanings",
    )?;
    exact_meanings(&source.flow_kind_meanings, FLOW_KINDS, "flow_kind_meanings")?;
    exact_meanings(
        &source.dependency_class_meanings,
        DEPENDENCY_CLASSES,
        "dependency_class_meanings",
    )?;
    exact_meanings(&source.loop_kind_meanings, LOOP_KINDS, "loop_kind_meanings")?;
    exact_meanings(
        &source.lifecycle_path_meanings,
        LIFECYCLE_PATHS,
        "lifecycle_path_meanings",
    )?;
    exact_meanings(
        &source.scenario_kind_meanings,
        SCENARIO_KINDS,
        "scenario_kind_meanings",
    )?;
    exact_meanings(
        &source.collision_axis_meanings,
        COLLISION_AXES,
        "collision_axis_meanings",
    )?;
    exact_meanings(
        &source.shock_kind_meanings,
        SHOCK_KINDS,
        "shock_kind_meanings",
    )?;
    exact_meanings(
        &source.protected_sphere_form_meanings,
        PROTECTED_SPHERE_FORMS,
        "protected_sphere_form_meanings",
    )?;
    Ok(())
}

fn validate_axes(source: &LedgerDocument) -> LedgerResult<()> {
    let required = [
        "legal-effect-class",
        "social-domain",
        "layer",
        "posture",
        "route",
        "overlay",
        "defect-disposition",
        "response-stage",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    for (index, axis) in source.axes.iter().enumerate() {
        nonempty(&axis.id, &format!("axes[{index}].id"))?;
        nonempty(&axis.name, &format!("axes[{index}].name"))?;
        nonempty(&axis.values, &format!("axes[{index}].values"))?;
        nonempty(&axis.note, &format!("axes[{index}].note"))?;
        if !ids.insert(axis.id.as_str()) {
            return Err(LedgerError::new(format!(
                "axes[{index}]: duplicate axis id {}",
                axis.id
            )));
        }
    }
    if !required.is_subset(&ids) {
        return Err(LedgerError::new(
            "axes must include every checker-owned stopping-rule axis",
        ));
    }
    if source.stopping_rule.named_axes
        != source
            .axes
            .iter()
            .map(|row| row.id.clone())
            .collect::<Vec<_>>()
    {
        return Err(LedgerError::new(
            "stopping_rule.named_axes must exactly follow the declared axes",
        ));
    }
    if source.stopping_rule.closure_conditions.len() != 5 {
        return Err(LedgerError::new(
            "closure_conditions must state the five ratified conditions",
        ));
    }
    Ok(())
}

fn validate_compatibility(source: &LedgerDocument) -> LedgerResult<()> {
    let expected = DEFECT_DISPOSITIONS.into_iter().collect::<BTreeSet<_>>();
    let mut covered = BTreeSet::new();
    for (index, row) in source.compatibility_table.iter().enumerate() {
        if !expected.contains(row.defect_disposition.as_str()) {
            return Err(LedgerError::new(format!(
                "compatibility_table[{index}]: unknown defect_disposition"
            )));
        }
        if !covered.insert(row.defect_disposition.as_str()) {
            return Err(LedgerError::new(
                "compatibility_table contains a duplicate disposition",
            ));
        }
        unique_strings(
            &row.allowed_response_stages,
            &format!("compatibility_table[{index}].allowed_response_stages"),
            false,
        )?;
        if row
            .allowed_response_stages
            .iter()
            .any(|stage| !RESPONSE_STAGES.contains(&stage.as_str()))
        {
            return Err(LedgerError::new(format!(
                "compatibility_table[{index}]: allowed_response_stages invalid"
            )));
        }
        nonempty(
            &row.resolution_requirement,
            &format!("compatibility_table[{index}].resolution_requirement"),
        )?;
        if [
            "externally-bounded-assumption",
            "irreducible-limitation",
            "open-defect",
        ]
        .contains(&row.defect_disposition.as_str())
            && row.resolution_eligible
        {
            return Err(LedgerError::new(
                "explicit non-resolution dispositions cannot be resolution-eligible",
            ));
        }
        if row.defect_disposition == "remedied"
            && row.allowed_response_stages != ["operationally-assured-in-envelope"]
        {
            return Err(LedgerError::new(
                "remedied is eligible only at operationally-assured-in-envelope",
            ));
        }
    }
    if covered != expected {
        return Err(LedgerError::new(
            "compatibility_table must cover every defect disposition",
        ));
    }
    Ok(())
}

fn validate_power_binding(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    const INVENTORY_STATUS: &str =
        "reviewed-inventory-input-not-law-not-operation-not-completeness-beyond-bound-version";
    const POPULATION_STATUS: &str = "complete-source-derived-contract-cards-and-allocations";
    const OWNER_REF: &str = "new-book-plans/book-1-constitutional-coverage-map.md::Maintain completed constitutional coverage rows";
    const CLOSURE_CONDITION: &str = "complete per-instrument FS-POW contract cards, lawful body and role allocations, and power-bound decider, executor, auditor, and final-remedy separation rows for every card-required and retained formal entry";
    const SCOPE_CEILING: &str = "Source-bound candidate census only: no row creates law, a complete contract card, a lawful holder, operation, assurance, FS-POW population completion, or Gate A passage. The cross-power temporal template creates no holder, power, or function allocation.";
    const EXPECTED_GAPS: [&str; 7] = [
        "appointments-qualification function and its nominee, selector, and qualification positions",
        "custodial execution function distinct from policing",
        "independent ecological science and assessment function",
        "ecological and animal regulation and inspection functions",
        "emergency alternate authoriser and independent substitute reviewer",
        "Guardian alternate advocate and substitute reviewer",
        "border and removal execution function",
    ];
    let binding = &source.power_source_inventory;
    if binding.artifact_ref != POWER_MANIFEST
        || binding.artifact_sha256
            != "2a664fa968423e1ffeec6036422600cc249aa7972258482978b921417ec5f67a"
        || binding.source_commit != "36ed92c58877cffa5a11928ad200f0ca9a604820"
        || binding.inventory_status != INVENTORY_STATUS
        || binding.row_count != 237
        || binding.disposition_counts.card_required != 209
        || binding.disposition_counts.power_contract_template != 1
        || binding.disposition_counts.existing_formal_crosswalk != 8
        || binding.disposition_counts.explicit_refusal_limit != 19
        || binding.power_population_status != POPULATION_STATUS
        || binding.known_allocation_gaps
            != EXPECTED_GAPS
                .iter()
                .map(|gap| (*gap).to_owned())
                .collect::<Vec<_>>()
        || binding.owner_ref != OWNER_REF
        || binding.closure_condition != CLOSURE_CONDITION
        || binding.scope_ceiling != SCOPE_CEILING
    {
        return Err(LedgerError::new(
            "power_source_inventory must equal the checker-bound reviewed manifest contract",
        ));
    }
    let manifest_bytes = input_bytes(inputs, POWER_MANIFEST)?;
    if sha256(&manifest_bytes) != binding.artifact_sha256 {
        return Err(LedgerError::new("power source manifest digest mismatch"));
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestSummary {
        spdx: String,
        schema_version: u64,
        title: String,
        status: String,
        source_commit: String,
        source_sha256: BTreeMap<String, String>,
        allowed_dispositions: Vec<String>,
        grain_rule_anchor: String,
        scope_note: String,
        row_count: usize,
        coverage_summary: ManifestCoverage,
        rows: Vec<ManifestRow>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestCoverage {
        by_disposition: BTreeMap<String, usize>,
        by_source_family: BTreeMap<String, usize>,
        by_source_family_and_disposition: BTreeMap<String, BTreeMap<String, usize>>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestRow {
        provisional_key: String,
        title: String,
        disposition: String,
        source_anchor: String,
        source_path: String,
        source_needle: String,
        legal_effect_and_grain: String,
        source_family: String,
    }
    parse_json_no_duplicates(manifest_bytes)
        .map_err(|error| LedgerError::new(format!("power source manifest invalid: {error}")))?;
    let manifest: ManifestSummary = serde_json::from_slice(manifest_bytes)
        .map_err(|error| LedgerError::new(format!("power source manifest invalid: {error}")))?;
    let _typed_contract_reads = (
        &manifest.spdx,
        manifest.schema_version,
        &manifest.title,
        &manifest.source_sha256,
        &manifest.allowed_dispositions,
        &manifest.grain_rule_anchor,
        &manifest.scope_note,
        &manifest.coverage_summary.by_source_family,
        &manifest.coverage_summary.by_source_family_and_disposition,
    );
    if manifest.source_commit != binding.source_commit
        || manifest.status != binding.inventory_status
        || manifest.row_count != binding.row_count
        || manifest.rows.len() != binding.row_count
    {
        return Err(LedgerError::new(
            "power source manifest identity binding is stale",
        ));
    }
    let expected_dispositions = BTreeMap::from([
        (
            "card-required".to_owned(),
            binding.disposition_counts.card_required,
        ),
        (
            "power-contract-template".to_owned(),
            binding.disposition_counts.power_contract_template,
        ),
        (
            "existing-formal-crosswalk".to_owned(),
            binding.disposition_counts.existing_formal_crosswalk,
        ),
        (
            "explicit-refusal-limit".to_owned(),
            binding.disposition_counts.explicit_refusal_limit,
        ),
    ]);
    if manifest.coverage_summary.by_disposition != expected_dispositions {
        return Err(LedgerError::new(
            "power source manifest disposition binding is stale",
        ));
    }
    let mut manifest_keys = HashSet::new();
    for row in &manifest.rows {
        nonempty(&row.provisional_key, "power manifest provisional_key")?;
        nonempty(&row.title, "power manifest title")?;
        nonempty(&row.source_anchor, "power manifest source_anchor")?;
        nonempty(&row.source_path, "power manifest source_path")?;
        nonempty(&row.source_needle, "power manifest source_needle")?;
        nonempty(
            &row.legal_effect_and_grain,
            "power manifest legal_effect_and_grain",
        )?;
        nonempty(&row.source_family, "power manifest source_family")?;
        if !manifest_keys.insert((row.provisional_key.as_str(), row.disposition.as_str())) {
            return Err(LedgerError::new("power manifest contains duplicate grains"));
        }
    }
    if source.power_population.status != "complete"
        || source.power_population.completed_source_families != SOURCE_FAMILIES.map(str::to_owned)
    {
        return Err(LedgerError::new(
            "power family completion must be the exact complete prefix",
        ));
    }
    if !source
        .power_population
        .evidence_ceiling
        .to_ascii_lowercase()
        .contains("no operation")
        || source
            .power_population
            .resolved_allocation_gaps
            .iter()
            .map(|gap| gap.gap.as_str())
            .ne(EXPECTED_GAPS)
    {
        return Err(LedgerError::new(
            "resolved power-allocation gaps and evidence ceiling must remain checker-owned",
        ));
    }
    let body_ids = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let role_ids = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    for (index, gap) in source
        .power_population
        .resolved_allocation_gaps
        .iter()
        .enumerate()
    {
        unique_strings(
            &gap.body_refs,
            &format!("power_population.resolved_allocation_gaps[{index}].body_refs"),
            false,
        )?;
        unique_strings(
            &gap.role_refs,
            &format!("power_population.resolved_allocation_gaps[{index}].role_refs"),
            false,
        )?;
        unique_strings(
            &gap.source_refs,
            &format!("power_population.resolved_allocation_gaps[{index}].source_refs"),
            false,
        )?;
        if gap
            .body_refs
            .iter()
            .any(|reference| !body_ids.contains(reference.as_str()))
            || gap
                .role_refs
                .iter()
                .any(|reference| !role_ids.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "power_population.resolved_allocation_gaps[{index}] names an unknown body or role"
            )));
        }
    }
    let counts = &source.power_population.expected_final_counts;
    if (
        counts.powers,
        counts.templates,
        counts.refusals,
        counts.crosswalks,
        counts.function_allocations,
    ) != (
        EXPECTED_POWER_COUNT,
        EXPECTED_TEMPLATE_COUNT,
        EXPECTED_REFUSAL_COUNT,
        EXPECTED_CROSSWALK_COUNT,
        EXPECTED_ALLOCATION_COUNT,
    ) {
        return Err(LedgerError::new("power final counts are checker-owned"));
    }
    if source.powers.len() != EXPECTED_POWER_COUNT
        || source.power_contract_templates.len() != EXPECTED_TEMPLATE_COUNT
        || source.power_refusals.len() != EXPECTED_REFUSAL_COUNT
        || source.power_crosswalk_dispositions.len() != EXPECTED_CROSSWALK_COUNT
        || source.function_allocations.len() != EXPECTED_ALLOCATION_COUNT
    {
        return Err(LedgerError::new(
            "completed power population counts do not match the contract",
        ));
    }
    if source.coverage_population.status != "complete"
        || source.coverage_population.completed_source_families
            != SOURCE_FAMILIES.map(str::to_owned)
        || source.coverage_population.expected_final_card_count != EXPECTED_POWER_COUNT
        || source
            .coverage_population
            .expected_constitutional_effect_count
            != EXPECTED_EFFECT_COUNT
        || !source
            .coverage_population
            .legacy_fields_permitted_until_complete
        || source.constitutional_effects.len() != EXPECTED_EFFECT_COUNT
    {
        return Err(LedgerError::new(
            "coverage population completion contract drifted",
        ));
    }
    if source
        .deferred_populations
        .iter()
        .any(|row| matches!(row.record_type.as_str(), "powers" | "coverage-contracts"))
    {
        return Err(LedgerError::new(
            "complete power and coverage populations may not retain their deferrals",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct PowerPolicyProjection<'a> {
    id: &'a str,
    manifest_key: &'a str,
    source_family: &'a str,
    primary_class_ref: &'a str,
    secondary_class_refs: &'a [String],
    profiles: &'a [String],
    affected_claim_refs: &'a [String],
    domain_refs: &'a [String],
}

#[derive(Serialize)]
struct EffectPolicyProjection<'a> {
    id: &'a str,
    effect_key: &'a str,
    primary_class_ref: &'a str,
    secondary_class_refs: &'a [String],
    profiles: &'a [String],
    affected_claim_refs: &'a [String],
    domain_refs: &'a [String],
}

#[derive(Serialize)]
struct CoveragePolicyProjection<'a> {
    id: &'a str,
    state: &'a str,
    source_family_refs: &'a [String],
    card_refs: &'a [String],
    template_refs: &'a [String],
    refusal_refs: &'a [String],
    crosswalk_refs: &'a [String],
    effect_refs: &'a [String],
    formal_statement_refs: &'a [String],
    pin_group_refs: &'a [String],
    counterfactual_refs: &'a [String],
    prose_refs: &'a [String],
    part_v_refs: &'a [String],
}

#[derive(Serialize)]
struct StateFormPolicyProjection<'a> {
    id: &'a str,
    holder_body_refs: &'a [String],
    holder_role_refs: &'a [String],
    decisive_fact_writer_body_refs: &'a [String],
    decisive_fact_writer_role_refs: &'a [String],
    decider_body_refs: &'a [String],
    decider_role_refs: &'a [String],
    executor_body_refs: &'a [String],
    executor_role_refs: &'a [String],
    auditor_body_refs: &'a [String],
    auditor_role_refs: &'a [String],
    final_remedy_body_refs: &'a [String],
    final_remedy_role_refs: &'a [String],
    negative_status: &'a str,
    negative_assertion: &'a str,
    negative_executable_ref: &'a Option<String>,
    counterfactual_status: &'a str,
    counterfactual_assertion: &'a str,
    counterfactual_executable_ref: &'a Option<String>,
    part_v_status: &'a str,
}

#[derive(Serialize)]
struct EconomicPowerPolicyProjection<'a> {
    id: &'a str,
    holder_body_refs: &'a [String],
    holder_role_refs: &'a [String],
    decisive_fact_writer_body_refs: &'a [String],
    decisive_fact_writer_role_refs: &'a [String],
    decider_body_refs: &'a [String],
    decider_role_refs: &'a [String],
    executor_body_refs: &'a [String],
    executor_role_refs: &'a [String],
    auditor_body_refs: &'a [String],
    auditor_role_refs: &'a [String],
    final_remedy_body_refs: &'a [String],
    final_remedy_role_refs: &'a [String],
    required_separation_pairs: &'a [Vec<String>],
    prohibited_inputs: &'a [String],
    negative_status: &'a str,
    negative_executable_ref: &'a Option<String>,
    counterfactual_status: &'a str,
    counterfactual_executable_ref: &'a Option<String>,
    part_v_status: &'a str,
}

fn typed_fingerprint<T: Serialize>(value: &T, context: &str) -> LedgerResult<String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| LedgerError::new(format!("{context} cannot be fingerprinted: {error}")))?;
    Ok(sha256(&bytes))
}

#[derive(Debug)]
struct EconomicRule<'a> {
    body: HashSet<&'a str>,
    head: &'a str,
}

fn economic_split_top_level<'a>(text: &'a str, separator: &str) -> LedgerResult<Vec<&'a str>> {
    if separator.is_empty() || !text.is_ascii() {
        return Err(LedgerError::new(
            "economic rule parser requires a nonempty separator and ASCII source",
        ));
    }
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0_usize;
    let mut depth = 0_usize;
    let mut index = 0_usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Err(LedgerError::new(format!(
                        "economic rule has an unmatched closing parenthesis: {text:?}"
                    )));
                }
                depth -= 1;
            }
            _ if depth == 0 && text[index..].starts_with(separator) => {
                parts.push(&text[start..index]);
                index += separator.len();
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    if depth != 0 {
        return Err(LedgerError::new(format!(
            "economic rule has unbalanced parentheses: {text:?}"
        )));
    }
    parts.push(&text[start..]);
    Ok(parts)
}

fn economic_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn economic_rule_remainder(statement: &str) -> LedgerResult<&str> {
    let mut remainder = statement
        .strip_suffix('.')
        .ok_or_else(|| LedgerError::new("economic rule statement lacks its final period"))?;
    while let Some(after_all) = remainder.strip_prefix("all $") {
        let (name, rest) = after_all.split_once(": ").ok_or_else(|| {
            LedgerError::new(format!(
                "economic rule has a malformed universal quantifier: {statement:?}"
            ))
        })?;
        if !economic_valid_identifier(name) {
            return Err(LedgerError::new(format!(
                "economic rule has an invalid quantified name: {name:?}"
            )));
        }
        remainder = rest;
    }
    Ok(remainder)
}

fn economic_quantified_names(statement: &str) -> LedgerResult<Vec<&str>> {
    let mut remainder = statement
        .strip_suffix('.')
        .ok_or_else(|| LedgerError::new("economic rule statement lacks its final period"))?;
    let mut names = Vec::new();
    while let Some(after_all) = remainder.strip_prefix("all $") {
        let (name, rest) = after_all.split_once(": ").ok_or_else(|| {
            LedgerError::new(format!(
                "economic rule has a malformed universal quantifier: {statement:?}"
            ))
        })?;
        if !economic_valid_identifier(name) || names.contains(&name) {
            return Err(LedgerError::new(format!(
                "economic rule has an invalid or duplicate quantified name: {name:?}"
            )));
        }
        names.push(name);
        remainder = rest;
    }
    Ok(names)
}

fn parse_economic_rule(statement: &str) -> LedgerResult<EconomicRule<'_>> {
    let remainder = economic_rule_remainder(statement)?;
    let implication = economic_split_top_level(remainder, " -> ")?;
    let [body, head] = implication.as_slice() else {
        return Err(LedgerError::new(format!(
            "economic rule needs exactly one top-level implication: {statement:?}"
        )));
    };
    let atoms = economic_split_top_level(body, " & ")?;
    if atoms.iter().any(|atom| atom.is_empty()) {
        return Err(LedgerError::new(
            "economic rule contains an empty body atom",
        ));
    }
    Ok(EconomicRule {
        body: atoms.into_iter().collect(),
        head,
    })
}

fn economic_call<'a>(text: &'a str, expected_name: &str) -> LedgerResult<Vec<&'a str>> {
    let open = text
        .find('(')
        .ok_or_else(|| LedgerError::new(format!("economic rule head is not a call: {text:?}")))?;
    if !text.ends_with(')')
        || &text[..open] != expected_name
        || !economic_valid_identifier(&text[..open])
    {
        return Err(LedgerError::new(format!(
            "economic rule head is not {expected_name}: {text:?}"
        )));
    }
    let arguments = economic_split_top_level(&text[open + 1..text.len() - 1], ",")?
        .into_iter()
        .map(str::trim)
        .collect::<Vec<_>>();
    if arguments.iter().any(|argument| argument.is_empty()) {
        return Err(LedgerError::new(format!(
            "economic rule head has an empty argument: {text:?}"
        )));
    }
    Ok(arguments)
}

fn economic_block(source: &str) -> LedgerResult<&str> {
    const BEGIN: &str = "# <ECONOMIC-CONSTITUTION-RULES-BEGIN>";
    const END: &str = "# <ECONOMIC-CONSTITUTION-RULES-END>";
    if source.matches(BEGIN).count() != 1 || source.matches(END).count() != 1 {
        return Err(LedgerError::new(
            "constitution needs exactly one economic-rule marker pair",
        ));
    }
    let start = source
        .find(BEGIN)
        .expect("the economic begin marker count was one");
    let end = source
        .find(END)
        .expect("the economic end marker count was one");
    if start >= end {
        return Err(LedgerError::new(
            "constitution economic-rule markers are reversed",
        ));
    }
    let block = &source[start..end + END.len()];
    if !block.is_ascii() {
        return Err(LedgerError::new(
            "constitution economic-rule block must remain ASCII Nibli source",
        ));
    }
    Ok(block)
}

fn economic_power_number(power_ref: &str) -> LedgerResult<usize> {
    power_ref
        .strip_prefix("FS-POW-")
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .filter(|number| (61..=88).contains(number))
        .ok_or_else(|| {
            LedgerError::new(format!(
                "economic rule contract has an invalid power_ref: {power_ref}"
            ))
        })
}

fn require_economic_atom(rule: &EconomicRule<'_>, atom: &str, context: &str) -> LedgerResult<()> {
    if !rule.body.contains(atom) {
        return Err(LedgerError::new(format!(
            "{context}: required rule-body atom is missing: {atom}"
        )));
    }
    Ok(())
}

fn require_economic_observation(
    rule: &EconomicRule<'_>,
    actor: &str,
    subject: &str,
    value: &str,
    scope: &str,
    context: &str,
) -> LedgerResult<()> {
    require_economic_atom(
        rule,
        &format!("observe({actor}, {subject}, {value}, {scope})"),
        context,
    )
}

fn economic_field_pairs(contract: &EconomicPowerRuleContract) -> Vec<(&str, &str, &str)> {
    ECONOMIC_COMMON_POWER_FIELDS
        .into_iter()
        .chain(contract.fields.iter().map(|field| {
            (
                field.name.as_str(),
                field.value.as_str(),
                field.scope.as_str(),
            )
        }))
        .collect()
}

fn economic_requirement_pairs(contract: &EconomicPowerRuleContract) -> Vec<(&str, &str)> {
    ECONOMIC_COMMON_POWER_REQUIREMENTS
        .into_iter()
        .chain(
            contract
                .requirements
                .iter()
                .map(|requirement| (requirement.value.as_str(), requirement.scope.as_str())),
        )
        .collect()
}

fn validate_economic_rule_contract_rows(
    contracts: &[EconomicPowerRuleContract],
    expected_power_refs: &[String],
) -> LedgerResult<()> {
    let actual_refs = contracts
        .iter()
        .map(|contract| contract.power_ref.as_str())
        .collect::<Vec<_>>();
    let expected_refs = expected_power_refs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let sequential_refs = (61..=88)
        .map(|number| format!("FS-POW-{number:03}"))
        .collect::<Vec<_>>();
    if actual_refs != expected_refs
        || actual_refs
            != sequential_refs
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
    {
        return Err(LedgerError::new(
            "economic rule contracts must follow the exact FS-CVF-006 FS-POW-061..088 order",
        ));
    }
    let common_names = ECONOMIC_COMMON_POWER_FIELDS
        .iter()
        .map(|(name, _, _)| *name)
        .collect::<HashSet<_>>();
    let common_pairs = ECONOMIC_COMMON_POWER_FIELDS
        .iter()
        .map(|(_, value, scope)| (*value, *scope))
        .collect::<HashSet<_>>();
    let common_requirements = ECONOMIC_COMMON_POWER_REQUIREMENTS
        .into_iter()
        .collect::<HashSet<_>>();
    for contract in contracts {
        let _ = economic_power_number(&contract.power_ref)?;
        for (name, value) in [
            ("temporal_contract", contract.temporal_contract.as_str()),
            ("jurisdiction_kind", contract.jurisdiction_kind.as_str()),
            (
                "authority_scope_kind",
                contract.authority_scope_kind.as_str(),
            ),
            ("holder", contract.holder.as_str()),
        ] {
            if !economic_valid_identifier(value) {
                return Err(LedgerError::new(format!(
                    "{}.{} must be an exact Nibli identifier",
                    contract.power_ref, name
                )));
            }
        }
        let mut field_names = HashSet::new();
        let mut field_pairs = HashSet::new();
        for field in &contract.fields {
            if !economic_valid_identifier(&field.name)
                || !(economic_valid_identifier(&field.value)
                    || field
                        .value
                        .strip_prefix('$')
                        .is_some_and(economic_valid_identifier))
                || !economic_valid_identifier(&field.scope)
                || common_names.contains(field.name.as_str())
                || common_pairs.contains(&(field.value.as_str(), field.scope.as_str()))
                || !field_names.insert(field.name.as_str())
                || !field_pairs.insert((field.value.as_str(), field.scope.as_str()))
            {
                return Err(LedgerError::new(format!(
                    "{}: card-specific economic fields must be unique, valid, and exclude common fields",
                    contract.power_ref
                )));
            }
        }
        let mut requirements = HashSet::new();
        for requirement in &contract.requirements {
            if !economic_valid_identifier(&requirement.value)
                || !economic_valid_identifier(&requirement.scope)
                || common_requirements
                    .contains(&(requirement.value.as_str(), requirement.scope.as_str()))
                || !requirements.insert((requirement.value.as_str(), requirement.scope.as_str()))
            {
                return Err(LedgerError::new(format!(
                    "{}: card-specific economic requirements must be unique, valid, and exclude common requirements",
                    contract.power_ref
                )));
            }
        }
    }
    Ok(())
}

fn validate_economic_carry_rule_contract_rows(
    contracts: &[EconomicCarryRuleContract],
) -> LedgerResult<()> {
    let expected = [
        (
            (
                "benefit",
                "EconomicBenefitCarryRecord",
                "EconomicBenefitCarryTemporalContract",
                "EconomicBenefitCarryCurrent",
                "EconomicBenefitCarryCurrentSelection",
                "EconomicBenefitCarryResult",
                "EconomicBenefitCarryBranch",
                "EconomicAdjudicatedBenefitFinding",
                "BenefitSourceCompetentJurisdiction",
                "AdjudicatedBenefitCarryScope",
            ),
            ("benefit", "$benefit", "EconomicBenefitScope"),
            (
                "AdjudicatedBenefitCarryWithinExactLegalSource",
                "EconomicCarryRequirementScope",
            ),
            (
                "EconomicCarryPredecessorRecordScope",
                "EconomicCarryPredecessorResultScope",
                "EconomicCarrySuccessorEventScope",
            ),
        ),
        (
            (
                "title",
                "EconomicTitleCarryRecord",
                "EconomicTitleCarryTemporalContract",
                "EconomicTitleCarryCurrent",
                "EconomicTitleCarryCurrentSelection",
                "EconomicTitleCarryResult",
                "EconomicTitleCarryBranch",
                "EconomicLawfulTitleFinding",
                "PropertySitusAndCompetentTierJurisdiction",
                "LawfulTitleCarryScope",
            ),
            ("title", "$title", "EconomicTitleScope"),
            (
                "AdjudicatedTitleCarryWithinExactLegalSource",
                "EconomicCarryRequirementScope",
            ),
            (
                "EconomicCarryPredecessorRecordScope",
                "EconomicCarryPredecessorResultScope",
                "EconomicCarrySuccessorEventScope",
            ),
        ),
        (
            (
                "liability",
                "EconomicLiabilityCarryRecord",
                "EconomicLiabilityCarryTemporalContract",
                "EconomicLiabilityCarryCurrent",
                "EconomicLiabilityCarryCurrentSelection",
                "EconomicLiabilityCarryResult",
                "EconomicLiabilityCarryBranch",
                "EconomicAdjudicatedLiabilityFinding",
                "LiabilityAdjudicationJurisdiction",
                "AdjudicatedLiabilityCarryScope",
            ),
            ("liability", "$liability", "EconomicLiabilityScope"),
            (
                "AdjudicatedLiabilityCarryWithinExactLegalSource",
                "EconomicCarryRequirementScope",
            ),
            (
                "EconomicCarryPredecessorRecordScope",
                "EconomicCarryPredecessorResultScope",
                "EconomicCarrySuccessorEventScope",
            ),
        ),
    ];
    if contracts.len() != expected.len() {
        return Err(LedgerError::new(format!(
            "economic carry rule contracts must contain exactly benefit, title, and liability; found {} rows",
            contracts.len()
        )));
    }
    for (contract, expected) in contracts.iter().zip(expected) {
        let actual = (
            (
                contract.carry_kind.as_str(),
                contract.record_kind.as_str(),
                contract.temporal_contract.as_str(),
                contract.current_kind.as_str(),
                contract.current_selection.as_str(),
                contract.result_kind.as_str(),
                contract.branch.as_str(),
                contract.finding_kind.as_str(),
                contract.jurisdiction_kind.as_str(),
                contract.legal_scope_kind.as_str(),
            ),
            (
                contract.interest.name.as_str(),
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            (
                contract.requirement.value.as_str(),
                contract.requirement.scope.as_str(),
            ),
            (
                contract.predecessor_record_scope.as_str(),
                contract.predecessor_result_scope.as_str(),
                contract.successor_event_scope.as_str(),
            ),
        );
        if actual != expected {
            return Err(LedgerError::new(format!(
                "economic {} carry rule contract differs from checker policy",
                contract.carry_kind
            )));
        }
    }
    Ok(())
}

fn economic_reference_parts<'a>(
    reference: &'a str,
    context: &str,
) -> LedgerResult<(&'a str, &'a str)> {
    if reference.matches("::").count() != 1 {
        return Err(LedgerError::new(format!(
            "{context}: reference must be `path::needle`"
        )));
    }
    reference
        .split_once("::")
        .ok_or_else(|| LedgerError::new(format!("{context}: malformed repository reference")))
}

fn economic_executable_pin_paths<'a>(
    inputs: &BTreeMap<String, Vec<u8>>,
    negative: &'a TestBinding,
    counterfactual: &'a TestBinding,
    context: &str,
) -> LedgerResult<BTreeSet<&'a str>> {
    let mut paths = BTreeSet::new();
    for (label, test) in [("negative", negative), ("counterfactual", counterfactual)] {
        if test.status != "executable" {
            continue;
        }
        let reference = test.executable_ref.0.as_deref().ok_or_else(|| {
            LedgerError::new(format!(
                "{context}.{label}: executable test has no executable_ref"
            ))
        })?;
        validate_repository_reference(
            inputs,
            reference,
            &format!("{context}.{label}.executable_ref"),
        )?;
        let (path, _) =
            economic_reference_parts(reference, &format!("{context}.{label}.executable_ref"))?;
        if !path.ends_with(".pins.nibli") {
            return Err(LedgerError::new(format!(
                "{context}.{label}: executable_ref is not a pin suite"
            )));
        }
        paths.insert(path);
    }
    if paths.is_empty() {
        return Err(LedgerError::new(format!(
            "{context}: acceptance owner has no executable pin suite"
        )));
    }
    Ok(paths)
}

fn economic_constitutional_effect_constant(
    inputs: &BTreeMap<String, Vec<u8>>,
    owner: &ConstitutionalEffect,
    context: &str,
) -> LedgerResult<String> {
    if owner.negative_test.status != "executable" {
        return Err(LedgerError::new(format!(
            "{context}: constitutional-effect owner has no executable negative test"
        )));
    }
    let reference = owner
        .negative_test
        .executable_ref
        .0
        .as_deref()
        .ok_or_else(|| {
            LedgerError::new(format!(
                "{context}: constitutional-effect owner negative test has no executable_ref"
            ))
        })?;
    validate_repository_reference(inputs, reference, context)?;
    let (path, needle) = economic_reference_parts(reference, context)?;
    let text = std::str::from_utf8(input_bytes(inputs, path)?).map_err(|error| {
        LedgerError::new(format!(
            "{context}: constitutional-effect pin suite is not UTF-8: {error}"
        ))
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    let marker_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            (*line == needle
                || line
                    .strip_prefix(needle)
                    .is_some_and(|suffix| suffix.starts_with(' ')))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let [marker_index] = marker_indexes.as_slice() else {
        return Err(LedgerError::new(format!(
            "{context}: constitutional-effect negative executable_ref is not a unique exact marker"
        )));
    };
    let query = lines
        .get(marker_index + 1)
        .and_then(|line| line.strip_prefix("? "))
        .and_then(|line| line.strip_suffix('.'))
        .ok_or_else(|| {
            LedgerError::new(format!(
                "{context}: constitutional-effect negative marker lacks its adjacent query"
            ))
        })?;
    let arguments = economic_call(query, "prevents")?;
    if arguments.len() != 2 {
        return Err(LedgerError::new(format!(
            "{context}: constitutional-effect negative query is not binary prevents"
        )));
    }
    Ok(arguments[1].to_owned())
}

fn economic_effect_title_key(title: &str) -> LedgerResult<String> {
    if !title.is_ascii() {
        return Err(LedgerError::new(
            "economic constitutional-effect title must be ASCII",
        ));
    }
    let mut key = String::new();
    let mut pending_separator = false;
    for byte in title.bytes() {
        if byte.is_ascii_alphanumeric() {
            if pending_separator && !key.is_empty() {
                key.push('-');
            }
            key.push((byte as char).to_ascii_lowercase());
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }
    if key.is_empty() {
        return Err(LedgerError::new(
            "economic constitutional-effect title has no key material",
        ));
    }
    Ok(key)
}

fn strip_economic_effect_identity_tag(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("For FS-CCE-") else {
        return text;
    };
    let Some((identity, suffix)) = rest.split_once("), ") else {
        return text;
    };
    if identity.split_once(" (").is_some_and(|(number, constant)| {
        number.len() == 3
            && number.bytes().all(|byte| byte.is_ascii_digit())
            && !constant.is_empty()
            && constant
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    }) {
        suffix
    } else {
        text
    }
}

fn normalize_economic_effect_term(text: &str) -> String {
    strip_economic_effect_identity_tag(text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_economic_effect_formal_binding(
    inputs: &BTreeMap<String, Vec<u8>>,
    effect: &ConstitutionalEffect,
    formal_constant: &str,
) -> LedgerResult<()> {
    let context = format!("{}.formal_binding", effect.id);
    let formal_rule = format!("all $x: person($x) -> prevents($x, {formal_constant}).");
    let constitution =
        std::str::from_utf8(input_bytes(inputs, "new-book-plans/constitution.nibli")?).map_err(
            |error| LedgerError::new(format!("{context}: constitution is not UTF-8: {error}")),
        )?;
    if constitution
        .lines()
        .filter(|line| *line == formal_rule)
        .count()
        != 1
    {
        return Err(LedgerError::new(format!(
            "{context}: exact direct-effect rule is not unique"
        )));
    }

    let reference = effect
        .negative_test
        .executable_ref
        .0
        .as_deref()
        .ok_or_else(|| LedgerError::new(format!("{context}: negative pin reference is missing")))?;
    let (path, negative_marker) = economic_reference_parts(reference, &context)?;
    let pin_text = std::str::from_utf8(input_bytes(inputs, path)?)
        .map_err(|error| LedgerError::new(format!("{context}: pin suite is not UTF-8: {error}")))?;
    let lines = pin_text.lines().collect::<Vec<_>>();
    let positive_marker = format!("# {} executable boundary: {formal_constant}", effect.id);
    let positive_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == positive_marker).then_some(index))
        .collect::<Vec<_>>();
    let [positive_index] = positive_indexes.as_slice() else {
        return Err(LedgerError::new(format!(
            "{context}: exact positive executable marker is not unique"
        )));
    };
    let positive_query = format!("? prevents(Adam, {formal_constant}).");
    if lines.get(positive_index + 1).copied() != Some(positive_query.as_str())
        || lines.get(positive_index + 2).copied() != Some("# => TRUE")
    {
        return Err(LedgerError::new(format!(
            "{context}: positive executable pin is not the exact formal effect"
        )));
    }

    let negative_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            (*line == negative_marker
                || line
                    .strip_prefix(negative_marker)
                    .is_some_and(|suffix| suffix.starts_with(' ')))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let [negative_index] = negative_indexes.as_slice() else {
        return Err(LedgerError::new(format!(
            "{context}: exact negative executable marker is not unique"
        )));
    };
    let negative_query = format!("? prevents(EconomicUnregisteredHandle, {formal_constant}).");
    if lines.get(negative_index + 1).copied() != Some(negative_query.as_str())
        || lines.get(negative_index + 2).copied() != Some("# => FALSE")
    {
        return Err(LedgerError::new(format!(
            "{context}: negative executable pin is not the exact formal effect"
        )));
    }
    Ok(())
}

fn validate_economic_effect_term_contracts(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    family: &CoverageFamily,
) -> LedgerResult<()> {
    let expected_refs = (ECONOMIC_EFFECT_FIRST..=EXPECTED_EFFECT_COUNT)
        .map(|number| format!("FS-CCE-{number:03}"))
        .collect::<Vec<_>>();
    if family.effect_refs != expected_refs {
        return Err(LedgerError::new(
            "FS-CVF-017 must contain the exact ordered economic effect census",
        ));
    }
    if family.blocked_before_drafting != ECONOMIC_EFFECT_COMPLETION_CEILING {
        return Err(LedgerError::new(
            "FS-CVF-017 supplied-record completion ceiling differs from checker policy",
        ));
    }

    let effects = source
        .constitutional_effects
        .iter()
        .map(|effect| (effect.id.as_str(), effect))
        .collect::<HashMap<_, _>>();
    let mut canonical_suffixes = BTreeMap::<String, String>::new();
    let mut normalized_full_terms = HashSet::<String>::new();
    let mut placements = 0_usize;
    for effect_ref in &family.effect_refs {
        let effect = effects.get(effect_ref.as_str()).ok_or_else(|| {
            LedgerError::new(format!(
                "{effect_ref}: economic constitutional effect is missing"
            ))
        })?;
        let formal_constant = economic_constitutional_effect_constant(
            inputs,
            effect,
            &format!("{effect_ref}.formal_constant"),
        )?;
        validate_economic_effect_formal_binding(inputs, effect, &formal_constant)?;
        if economic_effect_title_key(&effect.title)? != effect.effect_key {
            return Err(LedgerError::new(format!(
                "{effect_ref}: title and effect key differ"
            )));
        }
        let expected_applicability = format!(
            "Every person is protected against {} as the bounded direct constitutional effect named by {formal_constant}.",
            effect.title.to_ascii_lowercase()
        );
        if effect.applicability != expected_applicability
            || effect.permitted_downstream_effects.len() != 1
            || effect.permitted_downstream_effects.first() != Some(&effect.applicability)
        {
            return Err(LedgerError::new(format!(
                "{effect_ref}: applicability, formal constant, or singleton downstream effect differs"
            )));
        }
        let required_prefix = format!("{} ", effect.applicability);
        for (schema_key, term) in effect
            .contract_terms
            .iter()
            .map(|(name, term)| (format!("contract_terms.{name}"), term))
            .chain(effect.profile_terms.iter().flat_map(|(profile, terms)| {
                terms
                    .iter()
                    .map(move |(name, term)| (format!("profile_terms.{profile}.{name}"), term))
            }))
        {
            let suffix = term.text.strip_prefix(&required_prefix).ok_or_else(|| {
                LedgerError::new(format!(
                    "{effect_ref}.{schema_key}: term must begin with the exact substantive applicability and one space"
                ))
            })?;
            if suffix.is_empty() || suffix.trim() != suffix {
                return Err(LedgerError::new(format!(
                    "{effect_ref}.{schema_key}: canonical term suffix is empty or padded"
                )));
            }
            let canonical_suffix = suffix
                .replace(&formal_constant, "{constant}")
                .replace(&effect.title.to_ascii_lowercase(), "{title}");
            match canonical_suffixes.get(&schema_key) {
                Some(expected) if expected != &canonical_suffix => {
                    return Err(LedgerError::new(format!(
                        "{effect_ref}.{schema_key}: term suffix differs from its checker-owned schema"
                    )));
                }
                None => {
                    canonical_suffixes.insert(schema_key.clone(), canonical_suffix);
                }
                _ => {}
            }
            let normalized = normalize_economic_effect_term(&term.text);
            if !normalized_full_terms.insert(normalized) {
                return Err(LedgerError::new(format!(
                    "{effect_ref}.{schema_key}: normalized economic effect term is duplicated"
                )));
            }
            placements += 1;
        }
    }
    if placements != ECONOMIC_EFFECT_TERM_PLACEMENTS
        || normalized_full_terms.len() != ECONOMIC_EFFECT_TERM_PLACEMENTS
        || canonical_suffixes.len() != ECONOMIC_EFFECT_TERM_SCHEMA_KEYS
    {
        return Err(LedgerError::new(format!(
            "economic effect term census differs: {placements} placements, {} unique full terms, {} schema keys",
            normalized_full_terms.len(),
            canonical_suffixes.len()
        )));
    }
    let suffix_fingerprint =
        typed_fingerprint(&canonical_suffixes, "economic effect term suffix schemas")?;
    if suffix_fingerprint != ECONOMIC_EFFECT_TERM_SCHEMA_SHA256 {
        return Err(LedgerError::new(format!(
            "economic effect term suffix schemas differ from checker policy; candidate {suffix_fingerprint}"
        )));
    }
    Ok(())
}

fn validate_economic_acceptance_pin<'a>(
    inputs: &BTreeMap<String, Vec<u8>>,
    support: &'a EconomicAcceptanceSupport,
    context: &str,
) -> LedgerResult<&'a str> {
    validate_repository_reference(inputs, &support.pin_ref, &format!("{context}.pin_ref"))?;
    let (path, needle) = economic_reference_parts(&support.pin_ref, &format!("{context}.pin_ref"))?;
    if !path.ends_with(".pins.nibli") || !needle.starts_with("# ") {
        return Err(LedgerError::new(format!(
            "{context}.pin_ref must name an exact pin comment in a .pins.nibli file"
        )));
    }
    if support.query.trim() != support.query
        || !support.query.is_ascii()
        || support.query.contains(['\n', '\r'])
        || support.query.starts_with('?')
        || !support.query.ends_with('.')
        || support.query.contains(" -> ")
    {
        return Err(LedgerError::new(format!(
            "{context}.query must be one canonical ASCII Nibli query without its `?` prefix"
        )));
    }
    let query_call = support
        .query
        .strip_suffix('.')
        .expect("query period was checked");
    let predicate = query_call
        .split_once('(')
        .map(|(name, _)| name)
        .ok_or_else(|| LedgerError::new(format!("{context}.query is not a predicate call")))?;
    let _ = economic_call(query_call, predicate)?;
    if !matches!(support.expected_result.as_str(), "TRUE" | "FALSE")
        || (support.polarity == "derive") != (support.expected_result == "TRUE")
        || !matches!(support.polarity.as_str(), "derive" | "withhold")
    {
        return Err(LedgerError::new(format!(
            "{context}: derive must expect TRUE and withhold must expect FALSE"
        )));
    }
    let text = std::str::from_utf8(input_bytes(inputs, path)?).map_err(|error| {
        LedgerError::new(format!("{context}.pin_ref target is not UTF-8: {error}"))
    })?;
    let lines = text.lines().collect::<Vec<_>>();
    let marker_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == needle).then_some(index))
        .collect::<Vec<_>>();
    let [marker_index] = marker_indexes.as_slice() else {
        return Err(LedgerError::new(format!(
            "{context}.pin_ref needle must be one exact full comment line"
        )));
    };
    if lines.get(marker_index + 1).copied() != Some(&format!("? {}", support.query))
        || lines.get(marker_index + 2).copied()
            != Some(&format!("# => {}", support.expected_result))
    {
        return Err(LedgerError::new(format!(
            "{context}.pin_ref is not immediately followed by its exact query and expected result"
        )));
    }
    Ok(path)
}

fn economic_acceptance_fixture_facts(
    text: &str,
    number: usize,
    fixture: &str,
    selection: &str,
) -> BTreeSet<String> {
    let fixture_token = format!("{number:03}{fixture}");
    let normalized_selection = format!("EconomicAcceptanceSelection_{number:03}");
    text.lines()
        .filter(|line| {
            line.contains(&fixture_token)
                && !line.starts_with('#')
                && !line.starts_with('?')
                && line.ends_with('.')
        })
        .map(|line| {
            economic_normalize_acceptance_fixture(
                &line.replace(selection, &normalized_selection),
                fixture,
            )
        })
        .collect()
}

fn economic_normalize_acceptance_fixture(line: &str, fixture: &str) -> String {
    let mut normalized = String::with_capacity(line.len());
    let mut start = 0_usize;
    for (index, _) in line.match_indices(fixture) {
        normalized.push_str(&line[start..index]);
        if index >= 3
            && line.as_bytes()[index - 3..index]
                .iter()
                .all(u8::is_ascii_digit)
        {
            normalized.push_str("AcceptanceFixture");
        } else {
            normalized.push_str(fixture);
        }
        start = index + fixture.len();
    }
    normalized.push_str(&line[start..]);
    normalized
}

fn validate_economic_expired_selection_fixture(
    inputs: &BTreeMap<String, Vec<u8>>,
    pin_path: &str,
    number: usize,
    context: &str,
) -> LedgerResult<()> {
    let text = std::str::from_utf8(input_bytes(inputs, pin_path)?).map_err(|error| {
        LedgerError::new(format!("{context}: power pin suite is not UTF-8: {error}"))
    })?;
    let live_token = format!("{number:03}Live");
    let expired_token = format!("{number:03}ExpiredSelection");
    let current_selection = format!("EconomicCurrentSelection_{number:03}");
    let expired_selection = format!("EconomicExpiredSelection_{number:03}");
    let live_selection_count = text
        .lines()
        .filter(|line| line.contains(&live_token) && line.contains(&current_selection))
        .count();
    let expired_selection_count = text
        .lines()
        .filter(|line| line.contains(&expired_token) && line.contains(&expired_selection))
        .count();
    if live_selection_count == 0
        || expired_selection_count != live_selection_count
        || text
            .lines()
            .any(|line| line.contains(&expired_token) && line.contains(&current_selection))
        || economic_acceptance_fixture_facts(text, number, "Live", &current_selection)
            != economic_acceptance_fixture_facts(
                text,
                number,
                "ExpiredSelection",
                &expired_selection,
            )
    {
        return Err(LedgerError::new(format!(
            "{context}: expired-selection evidence must reproduce the complete live fixture with every exact current selection replaced"
        )));
    }
    Ok(())
}

fn validate_economic_missing_requirement_fixture(
    inputs: &BTreeMap<String, Vec<u8>>,
    pin_path: &str,
    number: usize,
    requirement: &EconomicPowerRuleRequirement,
    context: &str,
) -> LedgerResult<()> {
    let text = std::str::from_utf8(input_bytes(inputs, pin_path)?).map_err(|error| {
        LedgerError::new(format!("{context}: power pin suite is not UTF-8: {error}"))
    })?;
    let current_selection = format!("EconomicCurrentSelection_{number:03}");
    let live = economic_acceptance_fixture_facts(text, number, "Live", &current_selection);
    let missing = economic_acceptance_fixture_facts(
        text,
        number,
        "MissingCardSpecificRequirement",
        &current_selection,
    );
    let expected_omissions = ["Source", "Evidence", "Review"]
        .into_iter()
        .map(|actor| {
            format!(
                "observe(Econ{actor}{number:03}AcceptanceFixture, EconResult{number:03}AcceptanceFixture, {}, {}).",
                requirement.value, requirement.scope
            )
        })
        .collect::<BTreeSet<_>>();
    if missing.difference(&live).next().is_some()
        || live.difference(&missing).cloned().collect::<BTreeSet<_>>() != expected_omissions
    {
        return Err(LedgerError::new(format!(
            "{context}: missing-premise evidence must reproduce the complete live fixture minus only the first checker-owned card requirement"
        )));
    }
    Ok(())
}

fn validate_economic_classified_acceptance_fixture(
    inputs: &BTreeMap<String, Vec<u8>>,
    pin_path: &str,
    number: usize,
    fixture: &str,
    live_value_stem: &str,
    classified_value: &str,
    context: &str,
) -> LedgerResult<()> {
    let text = std::str::from_utf8(input_bytes(inputs, pin_path)?).map_err(|error| {
        LedgerError::new(format!("{context}: power pin suite is not UTF-8: {error}"))
    })?;
    let live_token = format!("{number:03}Live");
    let fixture_token = format!("{number:03}{fixture}");
    let live_value = format!("{live_value_stem}{number:03}Live");
    let normalized_live_value = format!("{live_value_stem}{number:03}AcceptanceFixture");
    let normalized_value = format!("EconomicAcceptanceClassifiedValue_{number:03}");
    let live_count = text
        .lines()
        .filter(|line| line.contains(&live_token) && line.contains(&live_value))
        .count();
    let classified_count = text
        .lines()
        .filter(|line| line.contains(&fixture_token) && line.contains(classified_value))
        .count();
    let current_selection = format!("EconomicCurrentSelection_{number:03}");
    let live_facts = economic_acceptance_fixture_facts(text, number, "Live", &current_selection)
        .into_iter()
        .map(|line| line.replace(&normalized_live_value, &normalized_value))
        .collect::<BTreeSet<_>>();
    let classified_facts =
        economic_acceptance_fixture_facts(text, number, fixture, &current_selection)
            .into_iter()
            .map(|line| line.replace(classified_value, &normalized_value))
            .collect::<BTreeSet<_>>();
    if live_count == 0
        || classified_count != live_count
        || text
            .lines()
            .any(|line| line.contains(&fixture_token) && line.contains(&live_value))
        || live_facts != classified_facts
    {
        return Err(LedgerError::new(format!(
            "{context}: classified acceptance evidence must reproduce the complete live fixture with only its exact supplied classification replaced"
        )));
    }
    Ok(())
}

fn validate_economic_acceptance_owner(
    source: &LedgerDocument,
    inputs: &BTreeMap<String, Vec<u8>>,
    economic_family: &CoverageFamily,
    support: &EconomicAcceptanceSupport,
    formal_needles: &[&str],
    pin_path: &str,
    context: &str,
) -> LedgerResult<()> {
    let (_, pin_needle) =
        economic_reference_parts(&support.pin_ref, &format!("{context}.pin_ref"))?;
    let query_call = support
        .query
        .strip_suffix('.')
        .expect("acceptance query was already validated");
    let predicate = query_call
        .split_once('(')
        .map(|(name, _)| name)
        .expect("acceptance query call was already validated");
    let arguments = economic_call(query_call, predicate)?;
    let formal_owner_match = match support.owner_kind.as_str() {
        "constitutional-effect" => {
            let owner = source
                .constitutional_effects
                .iter()
                .find(|row| row.id == support.owner_id)
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: unknown constitutional-effect owner {}",
                        support.owner_id
                    ))
                })?;
            let paths = economic_executable_pin_paths(
                inputs,
                &owner.negative_test,
                &owner.counterfactual,
                &format!("{context}.owner"),
            )?;
            let effect_constant = economic_constitutional_effect_constant(
                inputs,
                owner,
                &format!("{context}.owner.negative_test"),
            )?;
            let effect_number = owner
                .id
                .strip_prefix("FS-CCE-")
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: constitutional-effect owner has a malformed ID"
                    ))
                })?;
            let expected_header = if (223..=EXPECTED_EFFECT_COUNT).contains(&effect_number) {
                format!("# {} executable boundary: {effect_constant}", owner.id)
            } else {
                format!("# {} executable boundary:", owner.id)
            };
            if !paths.contains(pin_path)
                || pin_needle != expected_header
                || predicate != "prevents"
                || arguments.len() != 2
                || arguments[1] != effect_constant
                || support.polarity != "derive"
            {
                return Err(LedgerError::new(format!(
                    "{context}: constitutional-effect support is not the exact positive owner boundary"
                )));
            }
            let formal_rule = format!("all $x: person($x) -> prevents($x, {effect_constant}).");
            formal_needles.iter().any(|needle| *needle == formal_rule)
        }
        "power" => {
            if !economic_family.card_refs.contains(&support.owner_id) {
                return Err(LedgerError::new(format!(
                    "{context}: power owner is not an FS-CVF-006 card"
                )));
            }
            let owner = source
                .powers
                .iter()
                .find(|row| row.id == support.owner_id)
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: unknown power owner {}",
                        support.owner_id
                    ))
                })?;
            let number = economic_power_number(&owner.id)?;
            let power = format!("FSPOW_{number:03}");
            let expected_path = format!("new-book-plans/economic-power-{number:03}.pins.nibli");
            let paths = economic_executable_pin_paths(
                inputs,
                &owner.negative_test,
                &owner.counterfactual,
                &format!("{context}.owner"),
            )?;
            let live_header = format!(
                "# FS-POW-{number:03} acceptance live result: the exact source-bound result derives."
            );
            let live_query =
                format!("complete(EconResult{number:03}Live, {power}, EconRecord{number:03}Live).");
            let contract = source
                .economic_power_rule_contracts
                .iter()
                .find(|row| row.power_ref == owner.id)
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: power owner has no reviewed economic rule contract"
                    ))
                })?;
            let first_requirement = contract.requirements.first().ok_or_else(|| {
                LedgerError::new(format!(
                    "{context}: power owner has no card-specific acceptance requirement"
                ))
            })?;
            let missing_header = format!(
                "# FS-POW-{number:03} card-specific premise: omitting {}/{} withholds the result.",
                first_requirement.value, first_requirement.scope
            );
            let missing_query = format!(
                "complete(EconResult{number:03}MissingCardSpecificRequirement, {power}, EconRecord{number:03}MissingCardSpecificRequirement)."
            );
            let expired_header = format!(
                "# FS-POW-{number:03} acceptance expired selection: an externally supplied non-current selection withholds the result."
            );
            let expired_query = format!(
                "complete(EconResult{number:03}ExpiredSelection, {power}, EconRecord{number:03}ExpiredSelection)."
            );
            let anti_concentration_header = "# FS-POW-072 acceptance anti-concentration tax: the externally classified instrument remains a democratic calibration choice.";
            let anti_concentration_query = "complete(EconResult072AcceptanceAntiConcentrationTax, FSPOW_072, EconRecord072AcceptanceAntiConcentrationTax).";
            let luxury_collection_header = "# FS-POW-085 acceptance luxury collection: the externally classified above-floor collection remains bounded by floor-asset protections.";
            let luxury_collection_query = "complete(EconResult085AcceptanceLuxuryCollection, FSPOW_085, EconRecord085AcceptanceLuxuryCollection).";
            let evidence_kind = if pin_needle == live_header
                && support.query == live_query
                && support.polarity == "derive"
            {
                "live"
            } else if pin_needle == missing_header
                && support.query == missing_query
                && support.polarity == "withhold"
            {
                "missing-card-specific-requirement"
            } else if pin_needle == expired_header
                && support.query == expired_query
                && support.polarity == "withhold"
            {
                "expired-selection"
            } else if number == 72
                && pin_needle == anti_concentration_header
                && support.query == anti_concentration_query
                && support.polarity == "derive"
            {
                "anti-concentration-tax"
            } else if number == 85
                && pin_needle == luxury_collection_header
                && support.query == luxury_collection_query
                && support.polarity == "derive"
            {
                "luxury-collection"
            } else {
                return Err(LedgerError::new(format!(
                    "{context}: power support is not a checker-owned live, card-specific-premise, expired-selection, or classified acceptance fixture"
                )));
            };
            if pin_path != expected_path
                || !paths.contains(pin_path)
                || predicate != "complete"
                || arguments.len() != 3
                || !arguments.contains(&power.as_str())
            {
                return Err(LedgerError::new(format!(
                    "{context}: power support is not an exact {} shard/header/query",
                    owner.id
                )));
            }
            if evidence_kind == "expired-selection" {
                validate_economic_expired_selection_fixture(inputs, pin_path, number, context)?;
            } else if evidence_kind == "missing-card-specific-requirement" {
                validate_economic_missing_requirement_fixture(
                    inputs,
                    pin_path,
                    number,
                    first_requirement,
                    context,
                )?;
            } else if evidence_kind == "anti-concentration-tax" {
                validate_economic_classified_acceptance_fixture(
                    inputs,
                    pin_path,
                    number,
                    "AcceptanceAntiConcentrationTax",
                    "EconTaxInstrument",
                    "AntiConcentrationTaxInstrument",
                    context,
                )?;
            } else if evidence_kind == "luxury-collection" {
                validate_economic_classified_acceptance_fixture(
                    inputs,
                    pin_path,
                    number,
                    "AcceptanceLuxuryCollection",
                    "EconCollection",
                    "ParticularNonFloorLuxuryAssetCollection",
                    context,
                )?;
            }
            let formal_marker = format!("# {}: {}", owner.id, owner.title);
            formal_needles.iter().any(|needle| *needle == formal_marker)
        }
        "power-refusal" => {
            if !source
                .power_refusals
                .iter()
                .any(|row| row.id == support.owner_id)
                || !economic_family.refusal_refs.contains(&support.owner_id)
            {
                return Err(LedgerError::new(format!(
                    "{context}: power-refusal owner is not an FS-CVF-006 refusal"
                )));
            }
            let prefix = format!("# {} acceptance:", support.owner_id);
            if !pin_needle.starts_with(&prefix)
                || !matches!(predicate, "prevents" | "complete" | "authority")
            {
                return Err(LedgerError::new(format!(
                    "{context}: power-refusal support lacks its dedicated owner evidence"
                )));
            }
            formal_needles
                .iter()
                .any(|needle| needle.starts_with(&prefix))
        }
        "carry-contract" => {
            let owner = source
                .economic_carry_rule_contracts
                .iter()
                .find(|row| row.carry_kind == support.owner_id)
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: unknown carry-contract owner {}",
                        support.owner_id
                    ))
                })?;
            let header_prefix = format!("# economic-carry-{} positive result:", owner.carry_kind);
            if pin_path != "new-book-plans/economic-constitution.pins.nibli"
                || !pin_needle.starts_with(&header_prefix)
                || predicate != "complete"
                || arguments.len() != 3
                || arguments[1] != owner.result_kind
                || support.polarity != "derive"
            {
                return Err(LedgerError::new(format!(
                    "{context}: carry support is not an exact {} current/result query",
                    owner.carry_kind
                )));
            }
            let formal_marker = format!("# economic-carry-{}:", owner.carry_kind);
            formal_needles
                .iter()
                .any(|needle| needle.starts_with(&formal_marker))
        }
        "economic-duty" => {
            let always = ECONOMIC_ALWAYS_DUTY_BINDINGS
                .iter()
                .find(|(key, _, _)| *key == support.owner_id);
            let power = ECONOMIC_DUTY_BRIDGES
                .iter()
                .find(|row| row.key == support.owner_id);
            let (duty, standard, expected_path, formal_marker) = if let Some((
                key,
                duty,
                standard,
            )) = always
            {
                let effect = economic_always_duty_effect(key).ok_or_else(|| {
                        LedgerError::new(format!(
                            "{context}: checker-owned always duty {key} has no generic obligation effect"
                        ))
                    })?;
                (
                    *duty,
                    *standard,
                    "new-book-plans/economic-constitution.pins.nibli".to_owned(),
                    format!("# economic-duty-{key}: {duty} / {standard} / FS-CCE-{effect:03}"),
                )
            } else if let Some(spec) = power {
                (
                    spec.duty,
                    spec.standard,
                    format!("new-book-plans/economic-power-{:03}.pins.nibli", spec.power),
                    format!(
                        "# economic-duty-{}: {} / {} / FS-POW-{:03}",
                        spec.key, spec.duty, spec.standard, spec.power
                    ),
                )
            } else {
                return Err(LedgerError::new(format!(
                    "{context}: unknown checker-owned economic duty {}",
                    support.owner_id
                )));
            };
            let polarity = if support.polarity == "derive" {
                "positive"
            } else {
                "negative"
            };
            let header_prefix = format!("# {} {polarity}:", support.owner_id);
            if pin_path != expected_path
                || !pin_needle.starts_with(&header_prefix)
                || predicate != "obliged"
                || arguments.len() != 3
                || arguments[1] != duty
                || arguments[2] != standard
            {
                return Err(LedgerError::new(format!(
                    "{context}: duty support is not the exact owner duty/standard query"
                )));
            }
            formal_needles.iter().any(|needle| *needle == formal_marker)
        }
        "assertion-wall" => {
            let (_, expected_query, expected_result) = ECONOMIC_ASSERTION_WALL_EVIDENCE
                .iter()
                .find(|(owner_id, _, _)| *owner_id == support.owner_id)
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{context}: unknown checker-owned assertion wall {}",
                        support.owner_id
                    ))
                })?;
            let prefix = format!("# assertion-wall-{}:", support.owner_id);
            if pin_path != "new-book-plans/economic-constitution.pins.nibli"
                || !pin_needle.starts_with(&prefix)
                || support.query != *expected_query
                || support.expected_result != *expected_result
            {
                return Err(LedgerError::new(format!(
                    "{context}: assertion wall lacks its dedicated exact pin header"
                )));
            }
            formal_needles
                .iter()
                .any(|needle| needle.starts_with(&prefix))
        }
        _ => {
            return Err(LedgerError::new(format!(
                "{context}: unapproved acceptance owner kind {}",
                support.owner_kind
            )));
        }
    };
    if !formal_owner_match {
        return Err(LedgerError::new(format!(
            "{context}: no exact formal_ref belongs to declared owner {}",
            support.owner_id
        )));
    }
    Ok(())
}

fn validate_economic_assertion_wall_surface(constitution: &str) -> LedgerResult<()> {
    if constitution.matches("derived_only(\"reward\").").count() != 1
        || constitution.matches("derived_only(\"complete\").").count() != 1
    {
        return Err(LedgerError::new(
            "economic assertion walls require exact derived-only reward and complete declarations",
        ));
    }
    for owner_id in ECONOMIC_ASSERTION_WALL_IDS {
        let marker = format!("# assertion-wall-{owner_id}:");
        if constitution
            .lines()
            .filter(|line| line.starts_with(&marker))
            .count()
            != 1
        {
            return Err(LedgerError::new(format!(
                "assertion wall {owner_id} needs one exact formal marker"
            )));
        }
    }
    let active_lines = constitution
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>();
    let reward_rules = active_lines
        .iter()
        .copied()
        .filter(|line| line.contains("reward("))
        .collect::<Vec<_>>();
    if reward_rules.len() != 3 {
        return Err(LedgerError::new(format!(
            "recognition must remain exactly three derived leaf rules; found {} reward rules",
            reward_rules.len()
        )));
    }
    for statement in reward_rules {
        let (_, head) = statement
            .rsplit_once(" -> ")
            .ok_or_else(|| LedgerError::new("recognition occurs outside a rule conclusion"))?;
        let head = head
            .strip_suffix('.')
            .ok_or_else(|| LedgerError::new("recognition rule lacks its final period"))?;
        let arguments = economic_call(head, "reward")?;
        if arguments.len() != 1
            || !arguments[0].starts_with('$')
            || statement
                .split_once(" -> ")
                .is_some_and(|(body, _)| body.contains("reward("))
        {
            return Err(LedgerError::new(
                "recognition must remain arity-one, non-ranked, and unread",
            ));
        }
    }
    for forbidden in [
        "Book2ModelDerivedConstitutionalFact",
        "Book2StatisticDerivedConstitutionalFact",
    ] {
        if active_lines.iter().any(|line| line.contains(forbidden)) {
            return Err(LedgerError::new(format!(
                "raw Book 2 assertion wall has a formal producer for {forbidden}"
            )));
        }
    }
    Ok(())
}

fn validate_grounded_economic_duty_pin_order(
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<()> {
    for spec in ECONOMIC_DUTY_BRIDGES
        .iter()
        .filter(|spec| !spec.bearer.starts_with('$'))
    {
        let path = format!("new-book-plans/economic-power-{:03}.pins.nibli", spec.power);
        let pin = std::str::from_utf8(input_bytes(inputs, &path)?).map_err(|error| {
            LedgerError::new(format!("{path}: economic live pin is not UTF-8: {error}"))
        })?;
        let lines = pin.lines().collect::<Vec<_>>();
        let query = format!(
            "? obliged({}, {}, {}).",
            spec.bearer, spec.duty, spec.standard
        );
        let positive_marker = format!(
            "# {} positive: the exact completed card and reviewed duty selection compose.",
            spec.key
        );
        let negative_marker = format!("# {} negative:", spec.key);

        let positive_markers = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| (*line == positive_marker).then_some(index))
            .collect::<Vec<_>>();
        if positive_markers.len() != 1 {
            return Err(LedgerError::new(format!(
                "{path}: {} must have exactly one canonical positive duty marker; found {}",
                spec.key,
                positive_markers.len()
            )));
        }
        let positive_marker_index = positive_markers[0];
        if lines.get(positive_marker_index + 1) != Some(&query.as_str())
            || lines.get(positive_marker_index + 2) != Some(&"# => TRUE")
        {
            return Err(LedgerError::new(format!(
                "{path}: {} positive duty marker at line {} must be followed by its exact query and TRUE verdict",
                spec.key,
                positive_marker_index + 1
            )));
        }
        let positive_block_start = lines[..positive_marker_index]
            .iter()
            .rposition(|line| line.starts_with("# => "))
            .map_or(0, |index| index + 1);
        if !lines[positive_block_start..positive_marker_index]
            .iter()
            .any(|line| {
                let line = line.trim();
                !line.is_empty()
                    && !line.starts_with('#')
                    && !line.starts_with('?')
                    && !line.starts_with(':')
            })
        {
            return Err(LedgerError::new(format!(
                "{path}: {} positive duty marker at line {} has no preceding assertion block",
                spec.key,
                positive_marker_index + 1
            )));
        }

        let negative_markers = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| line.starts_with(&negative_marker).then_some(index))
            .collect::<Vec<_>>();
        if negative_markers.is_empty() {
            return Err(LedgerError::new(format!(
                "{path}: {} must have at least one grounded negative duty case",
                spec.key
            )));
        }
        let query_count = lines.iter().filter(|line| **line == query).count();
        let expected_query_count = negative_markers.len() + 1;
        if query_count != expected_query_count {
            return Err(LedgerError::new(format!(
                "{path}: {} has {query_count} exact grounded duty query occurrences but {expected_query_count} marked cases; duplicate or unmarked queries are forbidden",
                spec.key
            )));
        }
        for marker_index in &negative_markers {
            if lines.get(marker_index + 1) != Some(&query.as_str())
                || lines.get(marker_index + 2) != Some(&"# => FALSE")
            {
                return Err(LedgerError::new(format!(
                    "{path}: {} negative duty marker at line {} must be followed by its exact query and FALSE verdict",
                    spec.key,
                    marker_index + 1
                )));
            }
            if marker_index + 2 >= positive_block_start {
                return Err(LedgerError::new(format!(
                    "{path}: {} grounded negative duty case at line {} must precede the matching positive assertion block",
                    spec.key,
                    marker_index + 1
                )));
            }
        }
    }
    Ok(())
}

fn validate_economic_power_088_dependency_links(
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<()> {
    const PATH: &str = "new-book-plans/economic-power-088.pins.nibli";
    const SCENARIOS: [&str; 11] = [
        "Live",
        "FusedRecord",
        "WrongTemporal",
        "MissingFieldAuthorizingLaw",
        "MissingCardSpecificRequirement",
        "MismatchedEnd",
        "ExpiredSelection",
        "CrossDependencyEconomic087",
        "CrossDependencyState005",
        "AlternateIndependentReviewPositive",
        "AlternateIndependentReviewMismatchedOriginBearer",
    ];

    let pin = std::str::from_utf8(input_bytes(inputs, PATH)?)
        .map_err(|error| LedgerError::new(format!("{PATH}: live pin is not UTF-8: {error}")))?;
    let lines = pin.lines().collect::<Vec<_>>();
    let mut scenario_slots = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    let mut dependency_line_count = 0_usize;

    for (index, line) in lines.iter().enumerate() {
        if !line.contains("EconomicDependencyResultScope_087_086")
            && !line.contains("EconomicDependencyRecordScope_087_086")
        {
            continue;
        }
        dependency_line_count += 1;
        let statement = line.strip_suffix('.').ok_or_else(|| {
            LedgerError::new(format!(
                "{PATH}: P088 dependency link at line {} lacks its final period",
                index + 1
            ))
        })?;
        let arguments = economic_call(statement, "observe").map_err(|error| {
            LedgerError::new(format!(
                "{PATH}: malformed P088 dependency link at line {}: {error}",
                index + 1
            ))
        })?;
        let [actor, dependent_result, target, scope] = arguments.as_slice() else {
            return Err(LedgerError::new(format!(
                "{PATH}: P088 dependency link at line {} must have four observation arguments",
                index + 1
            )));
        };
        let scenario = dependent_result
            .strip_prefix("EconResult087For088")
            .filter(|suffix| !suffix.is_empty())
            .ok_or_else(|| {
                LedgerError::new(format!(
                    "{PATH}: P088 dependency link at line {} has a non-P088 dependent result: {dependent_result}",
                    index + 1
                ))
            })?;
        let actor_kind = ["Source", "Evidence", "Review"]
            .into_iter()
            .find(|kind| *actor == format!("Econ{kind}087For088{scenario}"))
            .ok_or_else(|| {
                LedgerError::new(format!(
                    "{PATH}: P088 dependency link at line {} has an actor outside its scenario: {actor}",
                    index + 1
                ))
            })?;
        let (target_kind, expected_target) = match *scope {
            "EconomicDependencyResultScope_087_086" => {
                ("result", format!("EconResult086For088{scenario}"))
            }
            "EconomicDependencyRecordScope_087_086" => {
                ("record", format!("EconRecord086For088{scenario}"))
            }
            _ => {
                return Err(LedgerError::new(format!(
                    "{PATH}: P088 dependency link at line {} has an unexpected scope: {scope}",
                    index + 1
                )));
            }
        };
        if *target != expected_target {
            return Err(LedgerError::new(format!(
                "{PATH}: P088 dependency link at line {} targets {target}; it must target its matching For088 prerequisite producer {expected_target}",
                index + 1
            )));
        }
        if !scenario_slots
            .entry(scenario.to_owned())
            .or_default()
            .insert((actor_kind.to_owned(), target_kind.to_owned()))
        {
            return Err(LedgerError::new(format!(
                "{PATH}: duplicate P088 {actor_kind}/{target_kind} dependency link for scenario {scenario}"
            )));
        }
    }

    if dependency_line_count != 66 {
        return Err(LedgerError::new(format!(
            "{PATH}: P088 must contain exactly 66 scoped 087-to-086 dependency observations; found {dependency_line_count}"
        )));
    }
    let expected_scenarios = SCENARIOS.into_iter().collect::<BTreeSet<_>>();
    let actual_scenarios = scenario_slots
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_scenarios != expected_scenarios {
        return Err(LedgerError::new(format!(
            "{PATH}: P088 dependency-link scenario census drifted: {actual_scenarios:?}"
        )));
    }
    for scenario in SCENARIOS {
        let slots = scenario_slots
            .get(scenario)
            .expect("the exact P088 scenario census was checked");
        if slots.len() != 6 {
            return Err(LedgerError::new(format!(
                "{PATH}: P088 scenario {scenario} must have all six source/evidence/review result/record links; found {}",
                slots.len()
            )));
        }
        for producer in [
            format!(
                "observe(EconSource086For088{scenario}, EconRecord086For088{scenario}, FSPOW_086, PowerScope)."
            ),
            format!(
                "observe(EconSource086For088{scenario}, EconRecord086For088{scenario}, EconResult086For088{scenario}, ResultScope)."
            ),
        ] {
            let producer_count = lines.iter().filter(|line| **line == producer).count();
            if producer_count != 1 {
                return Err(LedgerError::new(format!(
                    "{PATH}: P088 scenario {scenario} must identify its For088 prerequisite producer exactly once; found {producer_count} copies of {producer}"
                )));
            }
        }
    }

    const COUNTERFACTUAL_PATH: &str =
        "new-book-plans/counterfactual/no-economic-independent-current-review-088.pins.nibli";
    const COUNTERFACTUAL_SCENARIO: &str = "Counterfactual";
    let counterfactual =
        std::str::from_utf8(input_bytes(inputs, COUNTERFACTUAL_PATH)?).map_err(|error| {
            LedgerError::new(format!(
                "{COUNTERFACTUAL_PATH}: counterfactual pin is not UTF-8: {error}"
            ))
        })?;
    let counterfactual_lines = counterfactual.lines().collect::<Vec<_>>();
    let counterfactual_dependency_count = counterfactual_lines
        .iter()
        .filter(|line| {
            line.contains("EconomicDependencyResultScope_087_086")
                || line.contains("EconomicDependencyRecordScope_087_086")
        })
        .count();
    if counterfactual_dependency_count != 6 {
        return Err(LedgerError::new(format!(
            "{COUNTERFACTUAL_PATH}: P088 counterfactual must contain exactly six scoped 087-to-086 dependency observations; found {counterfactual_dependency_count}"
        )));
    }
    for actor_kind in ["Source", "Evidence", "Review"] {
        for (target_kind, scope) in [
            ("Result", "EconomicDependencyResultScope_087_086"),
            ("Record", "EconomicDependencyRecordScope_087_086"),
        ] {
            let expected = format!(
                "observe(Econ{actor_kind}087For088{COUNTERFACTUAL_SCENARIO}, EconResult087For088{COUNTERFACTUAL_SCENARIO}, Econ{target_kind}086For088{COUNTERFACTUAL_SCENARIO}, {scope})."
            );
            let count = counterfactual_lines
                .iter()
                .filter(|line| **line == expected)
                .count();
            if count != 1 {
                return Err(LedgerError::new(format!(
                    "{COUNTERFACTUAL_PATH}: P088 counterfactual must contain its matching For088 prerequisite link exactly once; found {count} copies of {expected}"
                )));
            }
        }
    }
    for producer in [
        "observe(EconSource086For088Counterfactual, EconRecord086For088Counterfactual, FSPOW_086, PowerScope).",
        "observe(EconSource086For088Counterfactual, EconRecord086For088Counterfactual, EconResult086For088Counterfactual, ResultScope).",
    ] {
        let producer_count = counterfactual_lines
            .iter()
            .filter(|line| **line == producer)
            .count();
        if producer_count != 1 {
            return Err(LedgerError::new(format!(
                "{COUNTERFACTUAL_PATH}: P088 counterfactual must identify its For088 prerequisite producer exactly once; found {producer_count} copies of {producer}"
            )));
        }
    }

    let expected_tier_observations = [
        "observe(EconSource088Counterfactual, EconRecord088Counterfactual, CommonTier, GovernmentTierScope).",
        "observe(EconTemporal088Counterfactual, EconTemporalRecord088Counterfactual, CommonTier, GovernmentTierScope).",
        "observe(EconTemporalReview088Counterfactual, EconTemporalRecord088Counterfactual, CommonTier, GovernmentTierScope).",
        "observe(EconSource088Counterfactual, EconResult088Counterfactual, CommonTier, GovernmentTierScope).",
        "observe(EconEvidence088Counterfactual, EconResult088Counterfactual, CommonTier, GovernmentTierScope).",
        "observe(EconReview088Counterfactual, EconResult088Counterfactual, CommonTier, GovernmentTierScope).",
    ];
    let p088_tier_prefixes = [
        "observe(EconSource088Counterfactual, EconRecord088Counterfactual, ",
        "observe(EconTemporal088Counterfactual, EconTemporalRecord088Counterfactual, ",
        "observe(EconTemporalReview088Counterfactual, EconTemporalRecord088Counterfactual, ",
        "observe(EconSource088Counterfactual, EconResult088Counterfactual, ",
        "observe(EconEvidence088Counterfactual, EconResult088Counterfactual, ",
        "observe(EconReview088Counterfactual, EconResult088Counterfactual, ",
    ];
    let p088_tier_count = counterfactual_lines
        .iter()
        .filter(|line| {
            line.ends_with(", GovernmentTierScope).")
                && p088_tier_prefixes
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
        })
        .count();
    if p088_tier_count != expected_tier_observations.len() {
        return Err(LedgerError::new(format!(
            "{COUNTERFACTUAL_PATH}: P088 counterfactual must contain exactly six shared-tier observations; found {p088_tier_count}"
        )));
    }
    for expected in expected_tier_observations {
        let count = counterfactual_lines
            .iter()
            .filter(|line| **line == expected)
            .count();
        if count != 1 {
            return Err(LedgerError::new(format!(
                "{COUNTERFACTUAL_PATH}: P088 counterfactual must bind its shared payment chain to CommonTier exactly once; found {count} copies of {expected}"
            )));
        }
    }
    Ok(())
}

fn validate_economic_acceptance_cases(
    source: &LedgerDocument,
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<()> {
    if source.economic_acceptance_cases.len() != ECONOMIC_ACCEPTANCE_CASES.len() {
        return Err(LedgerError::new(format!(
            "economic acceptance matrix must contain exactly 24 ordered cases; found {}",
            source.economic_acceptance_cases.len()
        )));
    }
    let decision =
        std::str::from_utf8(input_bytes(inputs, ECONOMIC_DECISION)?).map_err(|error| {
            LedgerError::new(format!("economic decision source is not UTF-8: {error}"))
        })?;
    let constitution =
        std::str::from_utf8(input_bytes(inputs, "new-book-plans/constitution.nibli")?)
            .map_err(|error| LedgerError::new(format!("constitution is not UTF-8: {error}")))?;
    validate_economic_assertion_wall_surface(constitution)?;
    let economic_family = source
        .coverage_families
        .iter()
        .find(|row| row.id == "FS-CVF-006")
        .ok_or_else(|| LedgerError::new("FS-CVF-006 economic coverage family is missing"))?;
    let mut variant_ids = HashSet::new();
    let mut mapping_ids = HashSet::new();
    let mut assertions = HashSet::new();
    let mut source_needles = HashSet::new();
    let mut mapping_count = 0_usize;
    let mut support_count = 0_usize;

    for (case, (expected_id, expected_needle, expected_count)) in source
        .economic_acceptance_cases
        .iter()
        .zip(ECONOMIC_ACCEPTANCE_CASES)
    {
        if case.case_id != expected_id
            || case.source_needle != expected_needle
            || case.mappings.len() != expected_count
            || decision.matches(&case.source_needle).count() != 1
            || !source_needles.insert(case.source_needle.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: acceptance case order, exact source needle, or variant count differs from checker policy",
                case.case_id
            )));
        }
        for (index, mapping) in case.mappings.iter().enumerate() {
            let context = format!("{}.mappings[{index}]", case.case_id);
            let expected_variant = format!("{}-V{:03}", case.case_id, index + 1);
            let expected_mapping = format!("{}-M{:03}", case.case_id, index + 1);
            if mapping.variant_id != expected_variant
                || mapping.mapping_id != expected_mapping
                || !variant_ids.insert(mapping.variant_id.as_str())
                || !mapping_ids.insert(mapping.mapping_id.as_str())
            {
                return Err(LedgerError::new(format!(
                    "{context}: variant and mapping IDs must be canonical and globally unique"
                )));
            }
            nonempty(&mapping.assertion, &format!("{context}.assertion"))?;
            if mapping.assertion.trim() != mapping.assertion
                || mapping.assertion.contains(['\n', '\r'])
                || !assertions.insert(mapping.assertion.as_str())
                || mapping.supports.is_empty()
            {
                return Err(LedgerError::new(format!(
                    "{context}: assertion must be one unique human-readable line with at least one support"
                )));
            }
            let mut support_bindings = HashSet::new();
            for (support_index, support) in mapping.supports.iter().enumerate() {
                let support_context = format!("{context}.supports[{support_index}]");
                let binding = typed_fingerprint(support, &format!("{support_context}.binding"))?;
                if !support_bindings.insert(binding) {
                    return Err(LedgerError::new(format!(
                        "{context}: duplicate exact support within one atomic variant"
                    )));
                }
                unique_strings(
                    &support.formal_refs,
                    &format!("{support_context}.formal_refs"),
                    false,
                )?;
                let mut formal_needles = Vec::new();
                for (formal_index, reference) in support.formal_refs.iter().enumerate() {
                    let formal_context = format!("{support_context}.formal_refs[{formal_index}]");
                    validate_repository_reference(inputs, reference, &formal_context)?;
                    let (path, needle) = economic_reference_parts(reference, &formal_context)?;
                    if path != "new-book-plans/constitution.nibli" {
                        return Err(LedgerError::new(format!(
                            "{formal_context}: formal acceptance supports must point to constitution.nibli"
                        )));
                    }
                    let formal =
                        std::str::from_utf8(input_bytes(inputs, path)?).map_err(|error| {
                            LedgerError::new(format!(
                                "{formal_context}: target is not UTF-8: {error}"
                            ))
                        })?;
                    if formal.lines().filter(|line| *line == needle).count() != 1 {
                        return Err(LedgerError::new(format!(
                            "{formal_context}: needle must be one exact full formal-source line"
                        )));
                    }
                    formal_needles.push(needle);
                }
                let pin_path = validate_economic_acceptance_pin(inputs, support, &support_context)?;
                validate_economic_acceptance_owner(
                    source,
                    inputs,
                    economic_family,
                    support,
                    &formal_needles,
                    pin_path,
                    &support_context,
                )?;
                support_count += 1;
            }
            mapping_count += 1;
        }
    }
    let expected_count = ECONOMIC_ACCEPTANCE_CASES
        .iter()
        .map(|(_, _, count)| count)
        .sum::<usize>();
    if mapping_count != 171 || mapping_count != expected_count {
        return Err(LedgerError::new(format!(
            "economic acceptance matrix must contain exactly 171 atomic mappings; found {mapping_count}"
        )));
    }
    if support_count < mapping_count {
        return Err(LedgerError::new(
            "economic acceptance matrix has fewer exact supports than atomic variants",
        ));
    }
    if typed_fingerprint(
        &source.economic_acceptance_cases,
        "economic acceptance cases",
    )? != EXPECTED_ECONOMIC_ACCEPTANCE_CASES_SHA256
    {
        return Err(LedgerError::new(
            "economic acceptance variants, owners, polarities, formal refs, or executable pins differ from checker policy",
        ));
    }
    Ok(())
}

fn economic_collision_kind(scope: &str) -> LedgerResult<String> {
    let stem = scope.strip_suffix("Scope").ok_or_else(|| {
        LedgerError::new(format!(
            "economic collision scope lacks Scope suffix: {scope}"
        ))
    })?;
    Ok(format!("Economic{stem}Binding"))
}

fn require_economic_collision_guards(
    rule: &EconomicRule<'_>,
    subject: &str,
    scopes: impl IntoIterator<Item = String>,
    context: &str,
) -> LedgerResult<()> {
    for scope in scopes.into_iter().collect::<BTreeSet<_>>() {
        let kind = economic_collision_kind(&scope)?;
        require_economic_atom(rule, &format!("~contradict({subject}, {kind})"), context)?;
    }
    Ok(())
}

fn economic_current_collision_scopes(fields: &[(&str, &str, &str)]) -> Vec<String> {
    ECONOMIC_CURRENT_COLLISION_SCOPES
        .into_iter()
        .chain(fields.iter().map(|(_, _, scope)| *scope))
        .map(str::to_owned)
        .collect()
}

fn economic_result_collision_scopes(
    fields: &[(&str, &str, &str)],
    requirements: &[(&str, &str)],
) -> Vec<String> {
    ECONOMIC_RESULT_COLLISION_SCOPES
        .into_iter()
        .chain(fields.iter().map(|(_, _, scope)| *scope))
        .chain(requirements.iter().map(|(_, scope)| *scope))
        .map(str::to_owned)
        .collect()
}

fn economic_reconciliation_collision_scopes() -> Vec<String> {
    ECONOMIC_RECONCILIATION_COLLISION_SCOPES
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn require_economic_pairwise_separation(
    rule: &EconomicRule<'_>,
    actors: &[&str],
    context: &str,
) -> LedgerResult<()> {
    for (index, left) in actors.iter().enumerate() {
        for right in &actors[index + 1..] {
            require_economic_atom(rule, &format!("~({left} = {right})"), context)?;
        }
    }
    Ok(())
}

fn economic_power_title(number: usize) -> LedgerResult<&'static str> {
    match number {
        61 => Ok("Mandatory occupational licence"),
        62 => Ok("Compulsory acquisition"),
        63 => Ok("Knowledge exclusivity and compulsory access licence"),
        64 => Ok("Public-scale private-power finding"),
        65 => Ok("Public-scale access mandate"),
        66 => Ok("Public-option remedy"),
        67 => Ok("Structural-separation remedy"),
        68 => Ok("Breakup remedy"),
        69 => Ok("Receivership"),
        70 => Ok("Licence withdrawal"),
        71 => Ok("Public acquisition remedy"),
        72 => Ok("Taxation"),
        73 => Ok("Legislative appropriation"),
        74 => Ok("Public spending"),
        75 => Ok("Public guarantee"),
        76 => Ok("Public borrowing"),
        77 => Ok("Monetary-policy authority"),
        78 => Ok("Credit and insurance decision"),
        79 => Ok("Insolvency restructuring"),
        80 => Ok("Temporary licence control"),
        81 => Ok("Physical-scarcity finding"),
        82 => Ok("Physical-scarcity allocation"),
        83 => Ok("Narrow compulsory service continuity"),
        84 => Ok("Ordinary-law price control"),
        85 => Ok("Tax collection"),
        86 => Ok("Public unit-of-account authority"),
        87 => Ok("Accessible settlement-backbone authority"),
        88 => Ok("Complementary payment-instrument regulation"),
        _ => Err(LedgerError::new(format!(
            "economic power has no checker-owned title: FS-POW-{number:03}"
        ))),
    }
}

fn economic_power_branch(number: usize) -> LedgerResult<String> {
    let title = economic_power_title(number)?;
    let compact = title
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    Ok(format!("FSPOW_{number:03}{compact}Branch"))
}

fn economic_alternate_review_duty(number: usize) -> LedgerResult<String> {
    let title = economic_power_title(number)?;
    let compact = title
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    Ok(format!("Review{compact}Duty"))
}

fn economic_duty_branch(key: &str) -> String {
    let mut suffix = String::new();
    for part in key.split('-') {
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            suffix.extend(first.to_uppercase());
            suffix.extend(characters);
        }
    }
    format!("EconomicDuty{suffix}Branch")
}

fn economic_contract_by_number(
    contracts: &[EconomicPowerRuleContract],
    number: usize,
) -> LedgerResult<&EconomicPowerRuleContract> {
    let power_ref = format!("FS-POW-{number:03}");
    contracts
        .iter()
        .find(|contract| contract.power_ref == power_ref)
        .ok_or_else(|| LedgerError::new(format!("economic contract missing {power_ref}")))
}

fn economic_named_field<'a>(
    contract: &'a EconomicPowerRuleContract,
    name: &str,
) -> LedgerResult<(&'a str, &'a str)> {
    if let Some((_, value, scope)) = ECONOMIC_COMMON_POWER_FIELDS
        .iter()
        .find(|(field_name, _, _)| *field_name == name)
    {
        return Ok((value, scope));
    }
    contract
        .fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| (field.value.as_str(), field.scope.as_str()))
        .ok_or_else(|| {
            LedgerError::new(format!(
                "{} has no checker-owned economic field named {name}",
                contract.power_ref
            ))
        })
}

fn economic_push_tri(atoms: &mut Vec<String>, subject: &str, value: &str, scope: &str) {
    for actor in ["$source", "$evidence", "$review"] {
        atoms.push(format!("observe({actor}, {subject}, {value}, {scope})"));
    }
}

fn economic_pairwise_atoms(actors: &[&str]) -> Vec<String> {
    let mut atoms = Vec::new();
    for (index, left) in actors.iter().enumerate() {
        for right in &actors[index + 1..] {
            atoms.push(format!("~({left} = {right})"));
        }
    }
    atoms
}

fn economic_collision_guard_atoms(
    subject: &str,
    scopes: impl IntoIterator<Item = String>,
) -> LedgerResult<Vec<String>> {
    scopes
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|scope| {
            Ok(format!(
                "~contradict({subject}, {})",
                economic_collision_kind(&scope)?
            ))
        })
        .collect()
}

fn validate_economic_exact_body(
    rule: &EconomicRule<'_>,
    expected: &[String],
    context: &str,
) -> LedgerResult<()> {
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set.len() != expected.len() {
        return Err(LedgerError::new(format!(
            "{context}: checker-owned body contract contains duplicate atoms"
        )));
    }
    let actual_set = rule
        .body
        .iter()
        .map(|atom| (*atom).to_owned())
        .collect::<BTreeSet<_>>();
    if actual_set == expected_set {
        return Ok(());
    }
    let missing = expected_set
        .difference(&actual_set)
        .take(3)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected = actual_set
        .difference(&expected_set)
        .take(3)
        .cloned()
        .collect::<Vec<_>>();
    Err(LedgerError::new(format!(
        "{context}: checker-owned rule body drifted ({} expected, {} actual; missing {missing:?}; unexpected {unexpected:?})",
        expected_set.len(),
        actual_set.len()
    )))
}

fn economic_insert_observation(
    atoms: &mut BTreeSet<String>,
    actor: &str,
    subject: &str,
    value: &str,
    scope: &str,
) {
    atoms.insert(format!("observe({actor}, {subject}, {value}, {scope})"));
}

fn economic_carry_current_atoms(contract: &EconomicCarryRuleContract) -> LedgerResult<Vec<String>> {
    let mut atoms = [
        "authorized($source, EconomicSourceAuthority, $record)",
        "authorized($record_review, EconomicRecordReviewAuthority, $record)",
        "authorized($temporal, EconomicTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, EconomicTemporalReviewAuthority, $temporal_record)",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    for (actor, subject) in [
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ] {
        for (value, scope) in [
            (
                "Constitution_EconomicPluralismAndPrivateSphere",
                "SourceFamilyScope",
            ),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$prior_epoch", "PriorSourceEpochScope"),
            (
                "$predecessor_record",
                contract.predecessor_record_scope.as_str(),
            ),
            (
                "$predecessor_result",
                contract.predecessor_result_scope.as_str(),
            ),
            ("$successor_event", contract.successor_event_scope.as_str()),
            (contract.record_kind.as_str(), "EconomicCarryKindScope"),
            (
                contract.temporal_contract.as_str(),
                "TemporalContractKindScope",
            ),
            (
                contract.current_selection.as_str(),
                "EffectiveSelectionScope",
            ),
            ("$case", "EconomicCaseScope"),
            ("$subject", "EconomicSubjectScope"),
            (
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "EconomicCarryLegalScope"),
            (
                contract.legal_scope_kind.as_str(),
                "EconomicCarryLegalScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
            ("$reconciliation", "ReconciliationRecordScope"),
            ("$result", "ResultScope"),
        ] {
            economic_insert_observation(&mut atoms, actor, subject, value, scope);
        }
    }
    atoms.extend(
        [
            "observe($source, $record, $temporal_record, TemporalRecordScope)".to_owned(),
            "observe($record_review, $record, $temporal_record, TemporalRecordScope)".to_owned(),
            "observe($source, $record, $temporal, TemporalAuthorityActorScope)".to_owned(),
            "observe($record_review, $record, $temporal, TemporalAuthorityActorScope)".to_owned(),
            "observe($source, $record, $temporal_review, TemporalReviewActorScope)".to_owned(),
            "observe($record_review, $record, $temporal_review, TemporalReviewActorScope)"
                .to_owned(),
            "observe($temporal, $temporal_record, $record, EconomicRecordScope)".to_owned(),
            "observe($temporal_review, $temporal_record, $record, EconomicRecordScope)"
                .to_owned(),
            format!(
                "carries($temporal, $predecessor_result, $epoch, $prior_epoch, {})",
                contract.temporal_contract
            ),
            format!(
                "carries($temporal_review, $predecessor_result, $epoch, $prior_epoch, {})",
                contract.temporal_contract
            ),
            format!(
                "observe($source, $reconciliation, Economic{}CarryRecordReconciled, ReconciliationStatusScope)",
                economic_title_case(&contract.carry_kind)
            ),
            format!(
                "observe($record_review, $reconciliation, Economic{}CarryRecordReconciled, ReconciliationStatusScope)",
                economic_title_case(&contract.carry_kind)
            ),
        ],
    );
    for actor in ["$source", "$record_review"] {
        for (value, scope) in [
            ("$record", "EconomicRecordScope"),
            ("$result", "ResultScope"),
            (contract.record_kind.as_str(), "EconomicCarryKindScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$prior_epoch", "PriorSourceEpochScope"),
            (
                "$predecessor_record",
                contract.predecessor_record_scope.as_str(),
            ),
            (
                "$predecessor_result",
                contract.predecessor_result_scope.as_str(),
            ),
            ("$successor_event", contract.successor_event_scope.as_str()),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$subject", "EconomicSubjectScope"),
            (
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "EconomicCarryLegalScope"),
            (
                contract.legal_scope_kind.as_str(),
                "EconomicCarryLegalScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
        ] {
            economic_insert_observation(&mut atoms, actor, "$reconciliation", value, scope);
        }
    }
    atoms.extend(economic_pairwise_atoms(&[
        "$source",
        "$record_review",
        "$temporal",
        "$temporal_review",
    ]));
    atoms.extend(
        [
            "~($record = $predecessor_record)",
            "~($result = $predecessor_result)",
            "~($epoch = $prior_epoch)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    for subject in ["$record", "$temporal_record"] {
        atoms.extend(economic_collision_guard_atoms(
            subject,
            ECONOMIC_CARRY_CURRENT_COLLISION_SCOPES.map(str::to_owned),
        )?);
    }
    atoms.extend(economic_collision_guard_atoms(
        "$reconciliation",
        ECONOMIC_CARRY_RECONCILIATION_COLLISION_SCOPES.map(str::to_owned),
    )?);
    Ok(atoms.into_iter().collect())
}

fn economic_carry_result_atoms(contract: &EconomicCarryRuleContract) -> LedgerResult<Vec<String>> {
    let mut atoms = economic_carry_current_atoms(contract)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    atoms.extend([
        format!(
            "complete($record, {}, $temporal_record)",
            contract.current_kind
        ),
        "authorized($evidence, EconomicEvidenceAuthority, $record)".to_owned(),
        "authorized($review, EconomicIndependentReviewAuthority, $record)".to_owned(),
    ]);
    for actor in ["$source", "$evidence", "$review"] {
        for (value, scope) in [
            (
                "Constitution_EconomicPluralismAndPrivateSphere",
                "SourceFamilyScope",
            ),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$prior_epoch", "PriorSourceEpochScope"),
            (
                "$predecessor_record",
                contract.predecessor_record_scope.as_str(),
            ),
            (
                "$predecessor_result",
                contract.predecessor_result_scope.as_str(),
            ),
            ("$successor_event", contract.successor_event_scope.as_str()),
            ("$temporal_record", "TemporalRecordScope"),
            (contract.record_kind.as_str(), "EconomicCarryKindScope"),
            ("$case", "EconomicCaseScope"),
            ("$subject", "EconomicSubjectScope"),
            (
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "EconomicCarryLegalScope"),
            (
                contract.legal_scope_kind.as_str(),
                "EconomicCarryLegalScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
            ("$reconciliation", "ReconciliationRecordScope"),
            ("$result", "ResultScope"),
        ] {
            economic_insert_observation(&mut atoms, actor, "$record", value, scope);
        }
        for (value, scope) in [
            (contract.branch.as_str(), "EconomicBranchScope"),
            (
                contract.finding_kind.as_str(),
                "EconomicCarryFindingKindScope",
            ),
            (
                contract.requirement.value.as_str(),
                contract.requirement.scope.as_str(),
            ),
            (contract.record_kind.as_str(), "EconomicCarryKindScope"),
            ("$source", "EconomicSourceActorScope"),
            ("$evidence", "EconomicEvidenceActorScope"),
            ("$review", "EconomicIndependentReviewActorScope"),
            ("$challenge_record", "ChallengeScope"),
            ("$correction_record", "CorrectionScope"),
            ("$remedy_record", "RemedyScope"),
            ("$end", "EndConditionScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$prior_epoch", "PriorSourceEpochScope"),
            (
                "$predecessor_record",
                contract.predecessor_record_scope.as_str(),
            ),
            (
                "$predecessor_result",
                contract.predecessor_result_scope.as_str(),
            ),
            ("$successor_event", contract.successor_event_scope.as_str()),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$subject", "EconomicSubjectScope"),
            (
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "EconomicCarryLegalScope"),
            (
                contract.legal_scope_kind.as_str(),
                "EconomicCarryLegalScopeKindScope",
            ),
            (
                "EconomicCarryCreatesNoBenefitTitleLiabilityOrAuthority",
                "EconomicCarryEffectLimitScope",
            ),
            ("IndependentReviewComplete", "ReviewDispositionScope"),
            ("EconomicCarryFailureWithholdsOnly", "FailurePolarityScope"),
        ] {
            economic_insert_observation(&mut atoms, actor, "$result", value, scope);
        }
    }
    atoms.extend(
        [
            "observe($source, $result, $result_reconciliation, ReconciliationRecordScope)"
                .to_owned(),
            "observe($review, $result, $result_reconciliation, ReconciliationRecordScope)"
                .to_owned(),
            format!(
                "observe($source, $result_reconciliation, Economic{}CarryResultReconciled, ReconciliationStatusScope)",
                economic_title_case(&contract.carry_kind)
            ),
            format!(
                "observe($review, $result_reconciliation, Economic{}CarryResultReconciled, ReconciliationStatusScope)",
                economic_title_case(&contract.carry_kind)
            ),
        ],
    );
    for actor in ["$source", "$review"] {
        for (value, scope) in [
            ("$record", "EconomicRecordScope"),
            ("$result", "ResultScope"),
            (contract.record_kind.as_str(), "EconomicCarryKindScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$prior_epoch", "PriorSourceEpochScope"),
            (
                "$predecessor_record",
                contract.predecessor_record_scope.as_str(),
            ),
            (
                "$predecessor_result",
                contract.predecessor_result_scope.as_str(),
            ),
            ("$successor_event", contract.successor_event_scope.as_str()),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$subject", "EconomicSubjectScope"),
            (
                contract.interest.value.as_str(),
                contract.interest.scope.as_str(),
            ),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "EconomicCarryLegalScope"),
            (
                contract.legal_scope_kind.as_str(),
                "EconomicCarryLegalScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
        ] {
            economic_insert_observation(&mut atoms, actor, "$result_reconciliation", value, scope);
        }
    }
    atoms.extend(economic_pairwise_atoms(&[
        "$source",
        "$record_review",
        "$temporal",
        "$temporal_review",
        "$evidence",
        "$review",
    ]));
    atoms.extend(economic_collision_guard_atoms(
        "$result",
        ECONOMIC_CARRY_RESULT_COLLISION_SCOPES.map(str::to_owned),
    )?);
    atoms.extend(economic_collision_guard_atoms(
        "$result_reconciliation",
        ECONOMIC_CARRY_RECONCILIATION_COLLISION_SCOPES.map(str::to_owned),
    )?);
    Ok(atoms.into_iter().collect())
}

fn economic_title_case(value: &str) -> String {
    let mut characters = value.chars();
    let mut result = String::new();
    if let Some(first) = characters.next() {
        result.extend(first.to_uppercase());
    }
    result.extend(characters);
    result
}

fn validate_economic_carry_rules(
    lines: &[&str],
    contracts: &[EconomicCarryRuleContract],
) -> LedgerResult<BTreeMap<String, usize>> {
    let mut heads = BTreeMap::new();
    for contract in contracts {
        let context = format!("economic-carry-{}", contract.carry_kind);
        let marker = format!("# {context}: ");
        let marker_index = economic_unique_marker_index(lines, &marker)?;
        let statements = lines
            .get(marker_index + 1..marker_index + 3)
            .ok_or_else(|| {
                LedgerError::new(format!(
                    "{context} marker is not followed by two one-line rules"
                ))
            })?;
        if statements
            .iter()
            .any(|line| line.is_empty() || line.starts_with('#'))
            || lines
                .get(marker_index + 3)
                .is_some_and(|line| !line.is_empty() && !line.starts_with('#'))
        {
            return Err(LedgerError::new(format!(
                "{context} must have exactly two adjacent one-line rules"
            )));
        }
        if statements
            .iter()
            .any(|statement| statement.contains("FSPOW_") || statement.contains(" -> authority("))
        {
            return Err(LedgerError::new(format!(
                "{context} cannot consume or conclude a power or authority"
            )));
        }
        let current = parse_economic_rule(statements[0])?;
        let result = parse_economic_rule(statements[1])?;
        let current_head = format!(
            "complete($record, {}, $temporal_record)",
            contract.current_kind
        );
        let result_head = format!("complete($result, {}, $record)", contract.result_kind);
        if current.head != current_head || result.head != result_head {
            return Err(LedgerError::new(format!(
                "{context}: carry current/result heads differ from checker policy"
            )));
        }
        let interest_name =
            contract.interest.value.strip_prefix('$').ok_or_else(|| {
                LedgerError::new(format!("{context}: interest is not a variable"))
            })?;
        let mut current_names = [
            "record",
            "source",
            "record_review",
            "temporal",
            "temporal_review",
            "temporal_record",
            "version",
            "epoch",
            "prior_epoch",
            "case",
            "subject",
            "jurisdiction",
            "legal_scope",
            "end",
            "reconciliation",
            "result",
            "predecessor_record",
            "predecessor_result",
            "successor_event",
        ]
        .to_vec();
        current_names.push(interest_name);
        let mut result_names = current_names.clone();
        result_names.extend([
            "evidence",
            "review",
            "challenge_record",
            "correction_record",
            "remedy_record",
            "result_reconciliation",
        ]);
        if economic_quantified_names(statements[0])? != current_names
            || economic_quantified_names(statements[1])? != result_names
        {
            return Err(LedgerError::new(format!(
                "{context}: quantified current/result bindings differ from checker policy"
            )));
        }
        validate_economic_exact_body(
            &current,
            &economic_carry_current_atoms(contract)?,
            &format!("{context}-current"),
        )?;
        validate_economic_exact_body(
            &result,
            &economic_carry_result_atoms(contract)?,
            &format!("{context}-result"),
        )?;
        if heads.insert(current_head, 0).is_some() || heads.insert(result_head, 0).is_some() {
            return Err(LedgerError::new(
                "economic carry contracts contain duplicate complete heads",
            ));
        }
    }
    if heads.len() != 6 {
        return Err(LedgerError::new(format!(
            "economic carry contracts must own exactly six complete heads; found {}",
            heads.len()
        )));
    }
    Ok(heads)
}

fn economic_duty_origin_atoms(spec: &EconomicDutyBridgeSpec) -> Vec<String> {
    let mut atoms = [
        "authorized($source, ObligationsSourceAuthority, $record)",
        "authorized($temporal, ObligationsTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, ObligationsTemporalReviewAuthority, $temporal_record)",
        "authorized($record_review, ObligationsRecordReviewAuthority, $record)",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for (actor, subject) in [
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ] {
        for (value, scope) in [
            ("Constitution_Obligations", "SourceFamilyScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$jurisdiction", "JurisdictionScope"),
            ("$legal_scope", "AuthorityScope"),
            ("$origin", "ObligationOriginScope"),
            ("$start", "DutyStartScope"),
            ("$end", "DutyEndScope"),
            ("ObligationsCurrentSelection", "EffectiveSelectionScope"),
            ("$reconciliation", "ReconciliationRecordScope"),
        ] {
            atoms.push(format!("observe({actor}, {subject}, {value}, {scope})"));
        }
    }
    atoms.extend(
        [
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
            "~($source = $record_review)",
            "~($temporal = $temporal_review)",
            "~($temporal = $record_review)",
            "~($temporal_review = $record_review)",
            "~collide($record, ObligationOriginBinding)",
            "~collide($record, ObligationVersionBinding)",
            "~collide($record, ObligationJurisdictionBinding)",
            "~collide($record, ObligationScopeBinding)",
            "authorized($evidence, ObligationsEvidenceAuthority, $record)",
            "authorized($review, IndependentObligationsReviewAuthority, $record)",
            "~($source = $evidence)",
            "~($source = $review)",
            "~($evidence = $review)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    for (value, scope) in [
        ("$record", "ObligationsRecordScope"),
        ("$version", "SourceVersionScope"),
        ("$epoch", "SourceEpochScope"),
        ("$jurisdiction", "JurisdictionScope"),
        ("$legal_scope", "AuthorityScope"),
        ("$temporal_record", "TemporalRecordScope"),
        (spec.bearer, "DutyBearerScope"),
        (spec.duty, "DutyScope"),
        (spec.standard, "DutyStandardScope"),
        ("$affected", "DutyBeneficiaryOrObjectScope"),
        (spec.duty, "DutyKindScope"),
        (spec.function, "DutyFunctionOrCommitmentScope"),
        (spec.mode, "DutyBearerModeScope"),
        ("RoleDutyClass", "DutyClassScope"),
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
        ("EconomicDutyFailureWithholdsOnly", "FailurePolarityScope"),
        ("$mode_certificate", "DutyBearerModeCertificateScope"),
        ("$class_certificate", "DutyClassCertificateScope"),
    ] {
        economic_push_tri(&mut atoms, "$origin", value, scope);
    }
    atoms.push("~collide($origin, ObligationContractBinding)".to_owned());
    economic_push_tri(
        &mut atoms,
        "$origin",
        &economic_duty_branch(spec.key),
        "EconomicDutyBranchScope",
    );
    economic_push_tri(
        &mut atoms,
        "$origin",
        spec.standard,
        "EconomicDutyStandardKindScope",
    );
    atoms.push("~contradict($origin, EconomicDutySelectionBinding)".to_owned());
    atoms
}

fn economic_duty_power_binding_atoms() -> Vec<String> {
    let mut atoms = Vec::new();
    for (value, scope) in [
        ("$power_record", "EconomicDutyPowerRecordScope"),
        ("$case", "EconomicDutyPowerCaseScope"),
        ("$function", "EconomicDutyPowerFunctionScope"),
        ("$affected", "EconomicDutyPowerAffectedScope"),
        ("$power_version", "EconomicDutyPowerVersionScope"),
        ("$power_epoch", "EconomicDutyPowerEpochScope"),
        (
            "$power_temporal_record",
            "EconomicDutyPowerTemporalRecordScope",
        ),
        ("$power_jurisdiction", "EconomicDutyPowerJurisdictionScope"),
        ("$power_legal_scope", "EconomicDutyPowerAuthorityScope"),
        ("$power_end", "EconomicDutyPowerEndScope"),
        ("$power_result", "EconomicDutyPowerResultScope"),
    ] {
        economic_push_tri(&mut atoms, "$origin", value, scope);
    }
    atoms
}

fn economic_duty_power_join_atoms(
    spec: &EconomicDutyBridgeSpec,
    contract: &EconomicPowerRuleContract,
) -> LedgerResult<Vec<String>> {
    let power = format!("FSPOW_{:03}", spec.power);
    let branch = economic_power_branch(spec.power)?;
    let mut atoms = vec![
        format!("complete($power_result, {power}, $power_record)"),
        "authorized($power_source, EconomicSourceAuthority, $power_record)".to_owned(),
        "authorized($power_evidence, EconomicEvidenceAuthority, $power_record)".to_owned(),
        "authorized($power_review, EconomicIndependentReviewAuthority, $power_record)".to_owned(),
    ];
    let mut joined_fields = BTreeSet::from(["subject", "function", "affected"]);
    for value in [spec.bearer, "$affected", spec.function] {
        if let Some(field) = value.strip_prefix('$')
            && economic_named_field(contract, field).is_ok()
        {
            joined_fields.insert(field);
        }
    }
    for actor in ["$power_source", "$power_evidence", "$power_review"] {
        for (value, scope, subject) in [
            (power.as_str(), "PowerScope", "$power_record"),
            ("$power_result", "ResultScope", "$power_record"),
            (branch.as_str(), "EconomicBranchScope", "$power_result"),
            (contract.holder.as_str(), "HolderScope", "$power_result"),
            ("$power_source", "EconomicSourceActorScope", "$power_result"),
            (
                "$power_evidence",
                "EconomicEvidenceActorScope",
                "$power_result",
            ),
            (
                "$power_review",
                "EconomicIndependentReviewActorScope",
                "$power_result",
            ),
            ("$power_version", "SourceVersionScope", "$power_result"),
            ("$power_epoch", "SourceEpochScope", "$power_result"),
            (
                "$power_temporal_record",
                "TemporalRecordScope",
                "$power_result",
            ),
            ("$case", "EconomicCaseScope", "$power_result"),
            ("$power_jurisdiction", "JurisdictionScope", "$power_result"),
            ("$power_legal_scope", "AuthorityScope", "$power_result"),
            ("$power_end", "EndConditionScope", "$power_result"),
        ] {
            atoms.push(format!("observe({actor}, {subject}, {value}, {scope})"));
        }
        for field in &joined_fields {
            let (value, scope) = economic_named_field(contract, field)?;
            atoms.push(format!("observe({actor}, $power_result, {value}, {scope})"));
        }
    }
    atoms.extend(economic_pairwise_atoms(&[
        "$power_source",
        "$power_evidence",
        "$power_review",
    ]));
    let fields = economic_field_pairs(contract);
    let requirements = economic_requirement_pairs(contract);
    atoms.extend(economic_collision_guard_atoms(
        "$power_record",
        economic_current_collision_scopes(&fields),
    )?);
    atoms.extend(economic_collision_guard_atoms(
        "$power_result",
        economic_result_collision_scopes(&fields, &requirements),
    )?);
    Ok(atoms)
}

fn economic_duty_expected_atoms(
    spec: &EconomicDutyBridgeSpec,
    contract: &EconomicPowerRuleContract,
) -> LedgerResult<Vec<String>> {
    let mut atoms = economic_duty_origin_atoms(spec);
    atoms.extend(economic_duty_power_binding_atoms());
    atoms.extend(economic_duty_power_join_atoms(spec, contract)?);
    Ok(atoms)
}

fn validate_economic_duty_bridge_rule(
    spec: &EconomicDutyBridgeSpec,
    contract: &EconomicPowerRuleContract,
    rule: &EconomicRule<'_>,
) -> LedgerResult<usize> {
    let expected_head = format!("obliged({}, {}, {})", spec.bearer, spec.duty, spec.standard);
    if rule.head != expected_head {
        return Err(LedgerError::new(format!(
            "economic duty bridge {} head drifted: {}",
            spec.key, rule.head
        )));
    }
    let expected = economic_duty_expected_atoms(spec, contract)?;
    validate_economic_exact_body(rule, &expected, spec.key)?;
    Ok(expected.len())
}

fn validate_economic_duty_bridges(
    lines: &[&str],
    contracts: &[EconomicPowerRuleContract],
) -> LedgerResult<usize> {
    let mut atom_count = 0_usize;
    let mut keys = HashSet::new();
    for spec in ECONOMIC_DUTY_BRIDGES {
        if !keys.insert(spec.key) {
            return Err(LedgerError::new(format!(
                "duplicate checker-owned economic duty bridge: {}",
                spec.key
            )));
        }
        let marker = format!(
            "# economic-duty-{}: {} / {} / FS-POW-{:03}",
            spec.key, spec.duty, spec.standard, spec.power
        );
        let index = economic_unique_marker_index(lines, &marker)?;
        let statement = lines.get(index + 1).ok_or_else(|| {
            LedgerError::new(format!("economic duty bridge {marker:?} has no rule"))
        })?;
        if statement.is_empty() || statement.starts_with('#') {
            return Err(LedgerError::new(format!(
                "economic duty bridge {} must be one adjacent rule",
                spec.key
            )));
        }
        let rule = parse_economic_rule(statement)?;
        let contract = economic_contract_by_number(contracts, spec.power)?;
        atom_count += validate_economic_duty_bridge_rule(&spec, contract, &rule)?;
    }
    if keys.len() != 31 {
        return Err(LedgerError::new(format!(
            "economic power-conditioned duty bridge census drifted: {}",
            keys.len()
        )));
    }
    let actual_power_conditioned = lines
        .iter()
        .filter(|line| {
            !line.starts_with('#')
                && line.contains("complete($power_result, FSPOW_")
                && line.contains(" -> obliged(")
        })
        .count();
    if actual_power_conditioned != ECONOMIC_DUTY_BRIDGES.len() {
        return Err(LedgerError::new(format!(
            "economic block must contain exactly {} non-alternate power-conditioned duty bridges; found {actual_power_conditioned}",
            ECONOMIC_DUTY_BRIDGES.len()
        )));
    }
    Ok(atom_count)
}

fn economic_dependency_expected_atoms(
    contracts: &[EconomicPowerRuleContract],
    contract: &EconomicPowerRuleContract,
    spec: &EconomicDependencySpec,
) -> LedgerResult<Vec<String>> {
    let prerequisite = economic_contract_by_number(contracts, spec.prerequisite)?;
    let result = format!("${}_result", spec.label);
    let record = format!("${}_record", spec.label);
    let source = format!("${}_source", spec.label);
    let evidence = format!("${}_evidence", spec.label);
    let review = format!("${}_review", spec.label);
    let power = format!("FSPOW_{:03}", spec.prerequisite);
    let branch = economic_power_branch(spec.prerequisite)?;
    let mut atoms = vec![
        format!("complete({result}, {power}, {record})"),
        format!("authorized({source}, EconomicSourceAuthority, {record})"),
        format!("authorized({evidence}, EconomicEvidenceAuthority, {record})"),
        format!("authorized({review}, EconomicIndependentReviewAuthority, {record})"),
    ];
    for actor in [&source, &evidence, &review] {
        atoms.extend([
            format!("observe({actor}, {record}, {power}, PowerScope)"),
            format!("observe({actor}, {record}, {result}, ResultScope)"),
            format!("observe({actor}, {result}, {branch}, EconomicBranchScope)"),
        ]);
        for field in spec.shared_fields {
            let (value, _) = economic_named_field(contract, field)?;
            let (_, scope) = economic_named_field(prerequisite, field)?;
            atoms.push(format!("observe({actor}, {result}, {value}, {scope})"));
        }
    }
    atoms.extend(economic_pairwise_atoms(&[&source, &evidence, &review]));
    for actor in ["$source", "$evidence", "$review"] {
        atoms.extend([
            format!(
                "observe({actor}, $result, {result}, EconomicDependencyResultScope_{:03}_{:03})",
                spec.card, spec.prerequisite
            ),
            format!(
                "observe({actor}, $result, {record}, EconomicDependencyRecordScope_{:03}_{:03})",
                spec.card, spec.prerequisite
            ),
        ]);
    }
    Ok(atoms)
}

fn state_dependency_expected_atoms(
    contract: &EconomicPowerRuleContract,
    spec: &StateDependencySpec,
) -> LedgerResult<Vec<String>> {
    let result = format!("$state_{}_result", spec.label);
    let record = format!("$state_{}_record", spec.label);
    let source = format!("$state_{}_source", spec.label);
    let evidence = format!("$state_{}_evidence", spec.label);
    let review = format!("$state_{}_review", spec.label);
    let power = format!("FSPOW_{:03}", spec.prerequisite);
    let (value, _) = economic_named_field(contract, spec.economic_field)?;
    let mut atoms = vec![
        format!("complete({result}, {power}, {record})"),
        format!("authorized({source}, StateFormSourceAuthority, {record})"),
        format!("authorized({evidence}, StateFormEvidenceAuthority, {record})"),
        format!("authorized({review}, IndependentStateFormReviewAuthority, {record})"),
    ];
    for actor in [&source, &evidence, &review] {
        atoms.extend([
            format!("observe({actor}, {record}, {power}, PowerScope)"),
            format!("observe({actor}, {record}, {result}, ResultScope)"),
            format!(
                "observe({actor}, {result}, {}, StateFormBranchScope)",
                spec.branch
            ),
            format!(
                "observe({actor}, {result}, {value}, {})",
                spec.state_field_scope
            ),
        ]);
    }
    atoms.extend(economic_pairwise_atoms(&[&source, &evidence, &review]));
    for actor in ["$source", "$evidence", "$review"] {
        atoms.extend([
            format!(
                "observe({actor}, $result, {result}, EconomicStateDependencyResultScope_{:03}_{:03})",
                spec.card, spec.prerequisite
            ),
            format!(
                "observe({actor}, $result, {record}, EconomicStateDependencyRecordScope_{:03}_{:03})",
                spec.card, spec.prerequisite
            ),
        ]);
    }
    Ok(atoms)
}

fn economic_expected_dependency_atoms(
    contracts: &[EconomicPowerRuleContract],
    contract: &EconomicPowerRuleContract,
    number: usize,
) -> LedgerResult<Vec<String>> {
    let mut atoms = Vec::new();
    for spec in ECONOMIC_DEPENDENCIES
        .iter()
        .filter(|spec| spec.card == number)
    {
        atoms.extend(economic_dependency_expected_atoms(
            contracts, contract, spec,
        )?);
    }
    for spec in STATE_DEPENDENCIES.iter().filter(|spec| spec.card == number) {
        atoms.extend(state_dependency_expected_atoms(contract, spec)?);
    }
    Ok(atoms)
}

fn is_economic_dependency_atom(atom: &str) -> bool {
    if atom.contains("EconomicDependencyResultScope_")
        || atom.contains("EconomicDependencyRecordScope_")
        || atom.contains("EconomicStateDependencyResultScope_")
        || atom.contains("EconomicStateDependencyRecordScope_")
    {
        return true;
    }
    ECONOMIC_DEPENDENCIES.iter().any(|spec| {
        ["result", "record", "source", "evidence", "review"]
            .into_iter()
            .any(|suffix| atom.contains(&format!("${}_{suffix}", spec.label)))
    }) || STATE_DEPENDENCIES.iter().any(|spec| {
        ["result", "record", "source", "evidence", "review"]
            .into_iter()
            .any(|suffix| atom.contains(&format!("$state_{}_{suffix}", spec.label)))
    })
}

fn validate_economic_dependency_joins(
    contracts: &[EconomicPowerRuleContract],
    contract: &EconomicPowerRuleContract,
    rule: &EconomicRule<'_>,
    number: usize,
) -> LedgerResult<usize> {
    let context = format!("{} dependency joins", contract.power_ref);
    let expected = economic_expected_dependency_atoms(contracts, contract, number)?;
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set.len() != expected.len() {
        return Err(LedgerError::new(format!(
            "{context}: checker-owned dependency contract contains duplicate atoms"
        )));
    }
    let actual = rule
        .body
        .iter()
        .filter(|atom| is_economic_dependency_atom(atom))
        .map(|atom| (*atom).to_owned())
        .collect::<BTreeSet<_>>();
    if actual != expected_set {
        let missing = expected_set
            .difference(&actual)
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = actual
            .difference(&expected_set)
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        return Err(LedgerError::new(format!(
            "{context}: checker-owned dependency joins drifted (missing {missing:?}; unexpected {unexpected:?})"
        )));
    }
    Ok(expected.len())
}

fn validate_economic_current_rule(
    contract: &EconomicPowerRuleContract,
    rule: &EconomicRule<'_>,
    number: usize,
) -> LedgerResult<()> {
    let context = format!("{} current rule", contract.power_ref);
    let power = format!("FSPOW_{number:03}");
    let current = format!("EconomicCurrent_{number:03}");
    let current_selection = format!("EconomicCurrentSelection_{number:03}");
    let expected_head = format!("complete($record, {current}, $temporal_record)");
    if rule.head != expected_head {
        return Err(LedgerError::new(format!(
            "{context}: head drifted: {}",
            rule.head
        )));
    }
    for (actor, subject) in [
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ] {
        for (value, scope) in [
            (
                "Constitution_EconomicPluralismAndPrivateSphere",
                "SourceFamilyScope",
            ),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            (power.as_str(), "PowerScope"),
            (
                contract.temporal_contract.as_str(),
                "TemporalContractKindScope",
            ),
            (current_selection.as_str(), "EffectiveSelectionScope"),
            ("$case", "EconomicCaseScope"),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "AuthorityScope"),
            (
                contract.authority_scope_kind.as_str(),
                "AuthorityScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
            ("$reconciliation", "ReconciliationRecordScope"),
        ] {
            require_economic_observation(rule, actor, subject, value, scope, &context)?;
        }
        for (_, value, scope) in economic_field_pairs(contract) {
            require_economic_observation(rule, actor, subject, value, scope, &context)?;
        }
    }
    for atom in [
        "authorized($source, EconomicSourceAuthority, $record)",
        "authorized($record_review, EconomicRecordReviewAuthority, $record)",
        "authorized($temporal, EconomicTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, EconomicTemporalReviewAuthority, $temporal_record)",
    ] {
        require_economic_atom(rule, atom, &context)?;
    }
    require_economic_pairwise_separation(
        rule,
        &[
            "$source",
            "$record_review",
            "$temporal",
            "$temporal_review",
            "$review",
            "$auditor",
            "$final_review",
            "$alternate_record_reviewer",
            "$alternate_temporal_reviewer",
            "$alternate_independent_reviewer",
            "$alternate_audit_reviewer",
            "$alternate_final_reviewer",
        ],
        &context,
    )?;
    let fields = economic_field_pairs(contract);
    require_economic_collision_guards(
        rule,
        "$record",
        economic_current_collision_scopes(&fields),
        &context,
    )?;
    require_economic_collision_guards(
        rule,
        "$temporal_record",
        economic_current_collision_scopes(&fields),
        &context,
    )?;
    require_economic_collision_guards(
        rule,
        "$reconciliation",
        economic_reconciliation_collision_scopes(),
        &context,
    )
}

fn validate_economic_result_rule(
    contracts: &[EconomicPowerRuleContract],
    contract: &EconomicPowerRuleContract,
    rule: &EconomicRule<'_>,
    number: usize,
) -> LedgerResult<()> {
    let context = format!("{} result rule", contract.power_ref);
    let power = format!("FSPOW_{number:03}");
    let current = format!("EconomicCurrent_{number:03}");
    let expected_head = format!("complete($result, {power}, $record)");
    if rule.head != expected_head {
        return Err(LedgerError::new(format!(
            "{context}: head drifted: {}",
            rule.head
        )));
    }
    require_economic_atom(
        rule,
        &format!("complete($record, {current}, $temporal_record)"),
        &context,
    )?;
    for atom in [
        "authorized($source, EconomicSourceAuthority, $record)",
        "authorized($record_review, EconomicRecordReviewAuthority, $record)",
        "authorized($temporal, EconomicTemporalAuthority, $temporal_record)",
        "authorized($temporal_review, EconomicTemporalReviewAuthority, $temporal_record)",
        "authorized($evidence, EconomicEvidenceAuthority, $record)",
        "authorized($review, EconomicIndependentReviewAuthority, $record)",
        "authorized($auditor, EconomicAuditAuthority, $record)",
        "authorized($final_review, EconomicFinalReviewAuthority, $record)",
        "authorized($executor, EconomicExecutionAuthority, $record)",
    ] {
        require_economic_atom(rule, atom, &context)?;
    }
    for (actor, subject) in [
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ] {
        for (value, scope) in [
            (
                "Constitution_EconomicPluralismAndPrivateSphere",
                "SourceFamilyScope",
            ),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            (power.as_str(), "PowerScope"),
            ("$case", "EconomicCaseScope"),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "AuthorityScope"),
            (
                contract.authority_scope_kind.as_str(),
                "AuthorityScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
        ] {
            require_economic_observation(rule, actor, subject, value, scope, &context)?;
        }
        for (_, value, scope) in economic_field_pairs(contract) {
            require_economic_observation(rule, actor, subject, value, scope, &context)?;
        }
    }
    for actor in ["$source", "$evidence", "$review"] {
        for (value, scope) in [
            (power.as_str(), "PowerScope"),
            ("$result", "ResultScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "AuthorityScope"),
            (
                contract.authority_scope_kind.as_str(),
                "AuthorityScopeKindScope",
            ),
            ("$reconciliation", "ReconciliationRecordScope"),
        ] {
            require_economic_observation(rule, actor, "$record", value, scope, &context)?;
        }
        for (value, scope) in [
            (contract.holder.as_str(), "HolderScope"),
            ("$source", "EconomicSourceActorScope"),
            ("$evidence", "EconomicEvidenceActorScope"),
            ("$review", "EconomicIndependentReviewActorScope"),
            ("$auditor", "EconomicAuditActorScope"),
            ("$final_review", "EconomicFinalReviewActorScope"),
            ("$executor", "EconomicExecutionActorScope"),
            ("$challenge_record", "ChallengeScope"),
            ("$correction_record", "CorrectionScope"),
            ("$remedy_record", "RemedyScope"),
            ("$end", "EndConditionScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "AuthorityScope"),
            (
                contract.authority_scope_kind.as_str(),
                "AuthorityScopeKindScope",
            ),
        ] {
            require_economic_observation(rule, actor, "$result", value, scope, &context)?;
        }
        for (_, value, scope) in economic_field_pairs(contract) {
            require_economic_observation(rule, actor, "$result", value, scope, &context)?;
        }
        for (value, scope) in economic_requirement_pairs(contract) {
            require_economic_observation(rule, actor, "$result", value, scope, &context)?;
        }
    }
    require_economic_pairwise_separation(
        rule,
        &[
            "$source",
            "$record_review",
            "$temporal",
            "$temporal_review",
            "$evidence",
            "$review",
            "$auditor",
            "$final_review",
            "$executor",
            "$alternate_record_reviewer",
            "$alternate_temporal_reviewer",
            "$alternate_independent_reviewer",
            "$alternate_audit_reviewer",
            "$alternate_final_reviewer",
        ],
        &context,
    )?;
    let fields = economic_field_pairs(contract);
    let requirements = economic_requirement_pairs(contract);
    for subject in ["$record", "$temporal_record"] {
        require_economic_collision_guards(
            rule,
            subject,
            economic_current_collision_scopes(&fields),
            &context,
        )?;
    }
    require_economic_collision_guards(
        rule,
        "$reconciliation",
        economic_reconciliation_collision_scopes(),
        &context,
    )?;
    require_economic_collision_guards(
        rule,
        "$result",
        economic_result_collision_scopes(&fields, &requirements),
        &context,
    )?;
    require_economic_collision_guards(
        rule,
        "$result_reconciliation",
        economic_reconciliation_collision_scopes(),
        &context,
    )?;
    validate_economic_dependency_joins(contracts, contract, rule, number)?;
    Ok(())
}

fn validate_economic_authority_rule(
    contract: &EconomicPowerRuleContract,
    rule: &EconomicRule<'_>,
    number: usize,
) -> LedgerResult<()> {
    let context = format!("{} authority rule", contract.power_ref);
    let power = format!("FSPOW_{number:03}");
    let expected_head = format!("authority({}, {power}, $record)", contract.holder);
    if rule.head != expected_head {
        return Err(LedgerError::new(format!(
            "{context}: head drifted: {}",
            rule.head
        )));
    }
    require_economic_atom(
        rule,
        &format!("complete($result, {power}, $record)"),
        &context,
    )?;
    for atom in [
        "authorized($source, EconomicSourceAuthority, $record)",
        "authorized($evidence, EconomicEvidenceAuthority, $record)",
        "authorized($review, EconomicIndependentReviewAuthority, $record)",
        "authorized($auditor, EconomicAuditAuthority, $record)",
        "authorized($final_review, EconomicFinalReviewAuthority, $record)",
        "authorized($executor, EconomicExecutionAuthority, $record)",
    ] {
        require_economic_atom(rule, atom, &context)?;
    }
    for actor in [
        "$source",
        "$evidence",
        "$review",
        "$auditor",
        "$final_review",
        "$executor",
    ] {
        require_economic_observation(rule, actor, "$record", &power, "PowerScope", &context)?;
        require_economic_observation(rule, actor, "$record", "$result", "ResultScope", &context)?;
        for (value, scope) in [
            (contract.holder.as_str(), "HolderScope"),
            ("$source", "EconomicSourceActorScope"),
            ("$evidence", "EconomicEvidenceActorScope"),
            ("$review", "EconomicIndependentReviewActorScope"),
            ("$auditor", "EconomicAuditActorScope"),
            ("$final_review", "EconomicFinalReviewActorScope"),
            ("$executor", "EconomicExecutionActorScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$temporal_record", "TemporalRecordScope"),
            ("$case", "EconomicCaseScope"),
            ("$jurisdiction", "JurisdictionScope"),
            (contract.jurisdiction_kind.as_str(), "JurisdictionKindScope"),
            ("$legal_scope", "AuthorityScope"),
            (
                contract.authority_scope_kind.as_str(),
                "AuthorityScopeKindScope",
            ),
            ("$end", "EndConditionScope"),
        ] {
            require_economic_observation(rule, actor, "$result", value, scope, &context)?;
        }
    }
    require_economic_pairwise_separation(
        rule,
        &[
            "$source",
            "$evidence",
            "$review",
            "$auditor",
            "$final_review",
            "$executor",
        ],
        &context,
    )?;
    let fields = economic_field_pairs(contract);
    let requirements = economic_requirement_pairs(contract);
    require_economic_collision_guards(
        rule,
        "$record",
        economic_current_collision_scopes(&fields),
        &context,
    )?;
    require_economic_collision_guards(
        rule,
        "$result",
        economic_result_collision_scopes(&fields, &requirements),
        &context,
    )
}

fn economic_unique_marker_index(lines: &[&str], prefix: &str) -> LedgerResult<usize> {
    let matches = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with(prefix).then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        _ => Err(LedgerError::new(format!(
            "economic rule block needs exactly one marker beginning {prefix:?}; found {}",
            matches.len()
        ))),
    }
}

fn validate_economic_alternate_review_rule(
    rule: &EconomicRule<'_>,
    number: usize,
    route: &str,
    unavailable_reviewer: &str,
    alternate_reviewer: &str,
    alternate_scope: &str,
) -> LedgerResult<()> {
    let alternate_duty = economic_alternate_review_duty(number)?;
    let alternate_standard = "CertifiedUnavailabilityNoSilenceNoExtensionAndSourceBoundEndStandard";
    if rule.head != format!("obliged({alternate_reviewer}, {alternate_duty}, {alternate_standard})")
    {
        return Err(LedgerError::new(format!(
            "alternate-review-{number:03}-{route} must derive only its typed obligation"
        )));
    }
    let context = format!("alternate-review-{number:03}-{route}");
    for actor in ["$source", "$evidence", "$review"] {
        for (value, scope) in [
            (alternate_reviewer, "DutyBearerScope"),
            (alternate_duty.as_str(), "DutyScope"),
            (alternate_standard, "DutyStandardScope"),
        ] {
            require_economic_observation(rule, actor, "$origin", value, scope, &context)?;
            let prefix = format!("observe({actor}, $origin, ");
            let suffix = format!(", {scope})");
            let selected = format!("{prefix}{value}{suffix}");
            if rule.body.iter().any(|atom| {
                atom.starts_with(&prefix) && atom.ends_with(&suffix) && *atom != selected
            }) {
                return Err(LedgerError::new(format!(
                    "{context}: {scope} must select only {value} for {actor}"
                )));
            }
        }
    }
    require_economic_atom(
        rule,
        &format!("complete($power_record, EconomicCurrent_{number:03}, $power_temporal_record)"),
        &context,
    )?;
    require_economic_atom(
        rule,
        "authorized($unavailability_source, EconomicReviewerUnavailabilityAuthority, $unavailability_record)",
        &context,
    )?;
    for (actor, subject) in [
        ("$power_source", "$power_record"),
        ("$power_record_review", "$power_record"),
        ("$power_temporal", "$power_temporal_record"),
        ("$power_temporal_review", "$power_temporal_record"),
    ] {
        require_economic_observation(
            rule,
            actor,
            subject,
            alternate_reviewer,
            alternate_scope,
            &context,
        )?;
    }
    for actor in [
        "$unavailability_source",
        "$unavailability_evidence",
        "$unavailability_review",
    ] {
        require_economic_observation(
            rule,
            actor,
            "$unavailability_record",
            unavailable_reviewer,
            "UnavailableReviewerScope",
            &context,
        )?;
        require_economic_observation(
            rule,
            actor,
            "$unavailability_record",
            alternate_reviewer,
            "PredeclaredAlternateScope",
            &context,
        )?;
    }
    require_economic_pairwise_separation(
        rule,
        &[
            "$unavailability_source",
            "$unavailability_evidence",
            "$unavailability_review",
            alternate_reviewer,
            unavailable_reviewer,
        ],
        &context,
    )
}

fn validate_economic_power_rule_surface(
    constitution_bytes: &[u8],
    contracts: &[EconomicPowerRuleContract],
    carry_contracts: &[EconomicCarryRuleContract],
) -> LedgerResult<()> {
    let constitution = std::str::from_utf8(constitution_bytes)
        .map_err(|error| LedgerError::new(format!("constitution is not UTF-8: {error}")))?;
    let block = economic_block(constitution)?;
    let lines = block.lines().collect::<Vec<_>>();
    validate_economic_duty_bridges(&lines, contracts)?;
    let mut expected_complete_heads = validate_economic_carry_rules(&lines, carry_contracts)?;
    let mut alternate_count = 0_usize;
    for contract in contracts {
        let number = economic_power_number(&contract.power_ref)?;
        let power = format!("FSPOW_{number:03}");
        let current = format!("EconomicCurrent_{number:03}");
        expected_complete_heads
            .insert(format!("complete($record, {current}, $temporal_record)"), 0);
        expected_complete_heads.insert(format!("complete($result, {power}, $record)"), 0);

        let card_marker = format!("# {}: ", contract.power_ref);
        let marker_index = economic_unique_marker_index(&lines, &card_marker)?;
        let card_lines = lines
            .get(marker_index + 1..marker_index + 4)
            .ok_or_else(|| {
                LedgerError::new(format!(
                    "{} marker is not followed by its three one-line rules",
                    contract.power_ref
                ))
            })?;
        if card_lines
            .iter()
            .any(|line| line.is_empty() || line.starts_with('#'))
        {
            return Err(LedgerError::new(format!(
                "{} must have exactly three adjacent one-line rules",
                contract.power_ref
            )));
        }
        let current_rule = parse_economic_rule(card_lines[0])?;
        let result_rule = parse_economic_rule(card_lines[1])?;
        let authority_rule = parse_economic_rule(card_lines[2])?;
        validate_economic_current_rule(contract, &current_rule, number)?;
        validate_economic_result_rule(contracts, contract, &result_rule, number)?;
        validate_economic_authority_rule(contract, &authority_rule, number)?;

        for (route, unavailable_reviewer, alternate_reviewer, alternate_scope) in
            ECONOMIC_ALTERNATE_REVIEW_ROUTES
        {
            let marker = format!("# alternate-review-{number:03}-{route}: ");
            let index = economic_unique_marker_index(&lines, &marker)?;
            let statement = lines.get(index + 1).ok_or_else(|| {
                LedgerError::new(format!("{marker} has no adjacent rule statement"))
            })?;
            let rule = parse_economic_rule(statement)?;
            validate_economic_alternate_review_rule(
                &rule,
                number,
                route,
                unavailable_reviewer,
                alternate_reviewer,
                alternate_scope,
            )?;
            alternate_count += 1;
        }
    }
    if expected_complete_heads.len() != 62 || alternate_count != 140 {
        return Err(LedgerError::new(format!(
            "economic rule contract census drifted: {} complete heads and {alternate_count} alternate branches",
            expected_complete_heads.len()
        )));
    }

    let mut actual_complete_count = 0_usize;
    for statement in lines
        .iter()
        .copied()
        .filter(|line| !line.starts_with('#') && line.contains(" -> "))
    {
        let rule = parse_economic_rule(statement)?;
        if rule.head.starts_with("complete(") {
            let arguments = economic_call(rule.head, "complete")?;
            if arguments.len() != 3 {
                return Err(LedgerError::new(format!(
                    "economic complete head has arity {} instead of 3",
                    arguments.len()
                )));
            }
            let count = expected_complete_heads.get_mut(rule.head).ok_or_else(|| {
                LedgerError::new(format!(
                    "economic block contains an unapproved complete head: {}",
                    rule.head
                ))
            })?;
            *count += 1;
            actual_complete_count += 1;
        }
    }
    if actual_complete_count != 62 || expected_complete_heads.iter().any(|(_, count)| *count != 1) {
        return Err(LedgerError::new(format!(
            "economic block must contain exactly 56 power-card and six non-power carry complete heads; found {actual_complete_count}"
        )));
    }
    let unavailability_authority = "authorized($unavailability_source, EconomicReviewerUnavailabilityAuthority, $unavailability_record)";
    if block.matches(unavailability_authority).count() != 140 {
        return Err(LedgerError::new(
            "economic block must contain exactly five typed alternate-review branches per card",
        ));
    }
    Ok(())
}

fn validate_coverage_state_semantics(source: &LedgerDocument) -> LedgerResult<()> {
    let powers = source
        .powers
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let effects = source
        .constitutional_effects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    for family in &source.coverage_families {
        if !matches!(family.state.as_str(), "formalized" | "prose-landed") {
            continue;
        }
        if family.formal_statement_refs.is_empty()
            || family.pin_group_refs.is_empty()
            || family.counterfactual_refs.is_empty()
        {
            return Err(LedgerError::new(format!(
                "{}: a formalized coverage family needs exact statements, pins, and counterfactuals",
                family.id
            )));
        }
        if family.state == "formalized"
            && (!family.prose_refs.is_empty() || !family.part_v_refs.is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: formalized-not-prose-landed coverage cannot carry prose anchors",
                family.id
            )));
        }
        if family.state == "prose-landed"
            && (family.prose_refs.is_empty() || family.part_v_refs.is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: prose-landed coverage needs numbered/method and Part V anchors",
                family.id
            )));
        }
        for power_ref in &family.card_refs {
            let power = powers.get(power_ref.as_str()).ok_or_else(|| {
                LedgerError::new(format!("{}: unknown power {power_ref}", family.id))
            })?;
            if power.negative_test.status != "executable"
                || power.negative_test.executable_ref.0.is_none()
                || power.counterfactual.status != "executable"
                || power.counterfactual.executable_ref.0.is_none()
            {
                return Err(LedgerError::new(format!(
                    "{}: {power_ref} test state does not follow its formalized coverage family",
                    family.id
                )));
            }
            let valid_part_v = if family.state == "formalized" {
                power.part_v_status == "formalized-not-prose-landed"
            } else {
                matches!(
                    power.part_v_status.as_str(),
                    "prose-landed" | "implemented-current-formal"
                )
            };
            if !valid_part_v {
                return Err(LedgerError::new(format!(
                    "{}: {power_ref} Part V state does not follow its coverage family",
                    family.id
                )));
            }
        }
        for effect_ref in &family.effect_refs {
            let effect = effects.get(effect_ref.as_str()).ok_or_else(|| {
                LedgerError::new(format!("{}: unknown effect {effect_ref}", family.id))
            })?;
            if effect.negative_test.status != "executable"
                || effect.negative_test.executable_ref.0.is_none()
                || effect.counterfactual.status != "executable"
                || effect.counterfactual.executable_ref.0.is_none()
            {
                return Err(LedgerError::new(format!(
                    "{}: {effect_ref} test state does not follow its formalized coverage family",
                    family.id
                )));
            }
            let expected_part_v = if family.state == "formalized" {
                "formalized-not-prose-landed"
            } else {
                "prose-landed"
            };
            if effect.part_v_status != expected_part_v {
                return Err(LedgerError::new(format!(
                    "{}: {effect_ref} Part V state does not follow its coverage family",
                    family.id
                )));
            }
        }
    }
    Ok(())
}

fn validate_power_effect_coverage_policy(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    validate_coverage_state_semantics(source)?;
    let power_policy = source
        .powers
        .iter()
        .map(|row| PowerPolicyProjection {
            id: &row.id,
            manifest_key: &row.manifest_key,
            source_family: &row.source_family,
            primary_class_ref: &row.primary_class_ref,
            secondary_class_refs: &row.secondary_class_refs,
            profiles: &row.profiles,
            affected_claim_refs: &row.affected_claim_refs,
            domain_refs: &row.domain_refs,
        })
        .collect::<Vec<_>>();
    if typed_fingerprint(&power_policy, "power classification")?
        != "f945c6380472fdceeae5d949ac3bd4efed13f6d383ce353d26ad7d32c3f9aa6f"
    {
        return Err(LedgerError::new(
            "power profiles, direct-effect classes, claims, or domains differ from checker policy",
        ));
    }
    for power_id in [
        "FS-POW-028",
        "FS-POW-029",
        "FS-POW-030",
        "FS-POW-031",
        "FS-POW-032",
        "FS-POW-033",
        "FS-POW-034",
        "FS-POW-035",
        "FS-POW-044",
    ] {
        let text = source
            .powers
            .iter()
            .find(|row| row.id == power_id)
            .and_then(|row| row.contract_terms.get("lawful_source"))
            .map(|term| term.text.as_str())
            .ok_or_else(|| LedgerError::new(format!("{power_id}: lawful_source is missing")))?;
        if !text.contains(
            "holder body and role arrays name only eligible constitutional participants or subjects",
        ) || !text.contains("exact current source-supplied")
        {
            return Err(LedgerError::new(format!(
                "{power_id}: supplied-configuration boundary drifted"
            )));
        }
    }
    let effect_policy = source
        .constitutional_effects
        .iter()
        .map(|row| EffectPolicyProjection {
            id: &row.id,
            effect_key: &row.effect_key,
            primary_class_ref: &row.primary_class_ref,
            secondary_class_refs: &row.secondary_class_refs,
            profiles: &row.profiles,
            affected_claim_refs: &row.affected_claim_refs,
            domain_refs: &row.domain_refs,
        })
        .collect::<Vec<_>>();
    if typed_fingerprint(&effect_policy, "constitutional-effect classification")?
        != "c6ff7408b058d9dc35b008b133aa7df42939a6a12ec7e822b1ea94735db48041"
    {
        return Err(LedgerError::new(
            "constitutional-effect taxonomy, profiles, claims, or domains differ from checker policy",
        ));
    }
    let coverage_policy = source
        .coverage_families
        .iter()
        .map(|row| CoveragePolicyProjection {
            id: &row.id,
            state: &row.state,
            source_family_refs: &row.source_family_refs,
            card_refs: &row.card_refs,
            template_refs: &row.template_refs,
            refusal_refs: &row.refusal_refs,
            crosswalk_refs: &row.crosswalk_refs,
            effect_refs: &row.effect_refs,
            formal_statement_refs: &row.formal_statement_refs,
            pin_group_refs: &row.pin_group_refs,
            counterfactual_refs: &row.counterfactual_refs,
            prose_refs: &row.prose_refs,
            part_v_refs: &row.part_v_refs,
        })
        .collect::<Vec<_>>();
    if typed_fingerprint(&coverage_policy, "coverage-family policy")?
        != "8770cb9ffe4addf0aca00f193472c0f9a25703dbf1852db0e012ceb85079ce8e"
    {
        return Err(LedgerError::new(
            "coverage-family state, partitions, or formal surfaces differ from checker policy",
        ));
    }

    let state_family = source
        .coverage_families
        .iter()
        .find(|row| row.id == "FS-CVF-003")
        .ok_or_else(|| LedgerError::new("FS-CVF-003 state-form coverage family is missing"))?;
    let allocations = source
        .function_allocations
        .iter()
        .map(|row| (row.power_ref.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut state_policy = Vec::new();
    for power in source
        .powers
        .iter()
        .filter(|row| state_family.card_refs.contains(&row.id))
    {
        let allocation = allocations.get(power.id.as_str()).ok_or_else(|| {
            LedgerError::new(format!(
                "{}: state-form function allocation missing",
                power.id
            ))
        })?;
        state_policy.push(StateFormPolicyProjection {
            id: &power.id,
            holder_body_refs: &power.holder_body_refs,
            holder_role_refs: &power.holder_role_refs,
            decisive_fact_writer_body_refs: &allocation.decisive_fact_writer_body_refs,
            decisive_fact_writer_role_refs: &allocation.decisive_fact_writer_role_refs,
            decider_body_refs: &allocation.decider_body_refs,
            decider_role_refs: &allocation.decider_role_refs,
            executor_body_refs: &allocation.executor_body_refs,
            executor_role_refs: &allocation.executor_role_refs,
            auditor_body_refs: &allocation.auditor_body_refs,
            auditor_role_refs: &allocation.auditor_role_refs,
            final_remedy_body_refs: &allocation.final_remedy_body_refs,
            final_remedy_role_refs: &allocation.final_remedy_role_refs,
            negative_status: &power.negative_test.status,
            negative_assertion: &power.negative_test.assertion,
            negative_executable_ref: &power.negative_test.executable_ref.0,
            counterfactual_status: &power.counterfactual.status,
            counterfactual_assertion: &power.counterfactual.assertion,
            counterfactual_executable_ref: &power.counterfactual.executable_ref.0,
            part_v_status: &power.part_v_status,
        });
    }
    if typed_fingerprint(&state_policy, "state-form semantic mapping")?
        != "49f5649a290b2838b8e9f7b1e1715740713800cb512afba588f8f44bb1fa5943"
    {
        return Err(LedgerError::new(
            "state-form holder, function, test, or Part V mapping differs from checker policy",
        ));
    }

    let economic_family = source
        .coverage_families
        .iter()
        .find(|row| row.id == "FS-CVF-006")
        .ok_or_else(|| LedgerError::new("FS-CVF-006 economic coverage family is missing"))?;
    validate_economic_rule_contract_rows(
        &source.economic_power_rule_contracts,
        &economic_family.card_refs,
    )?;
    if typed_fingerprint(
        &source.economic_power_rule_contracts,
        "economic power rule contracts",
    )? != "7a8f4297c0ee6c64fe8580a3d61970f0ad4b280da41566eaf3f2ba020680d19a"
    {
        return Err(LedgerError::new(
            "economic power fields, requirements, temporal contracts, jurisdictions, scopes, or holders differ from checker policy",
        ));
    }
    validate_economic_carry_rule_contract_rows(&source.economic_carry_rule_contracts)?;
    validate_grounded_economic_duty_pin_order(inputs)?;
    validate_economic_power_088_dependency_links(inputs)?;
    validate_economic_acceptance_cases(source, inputs)?;
    validate_economic_power_rule_surface(
        input_bytes(inputs, "new-book-plans/constitution.nibli")?,
        &source.economic_power_rule_contracts,
        &source.economic_carry_rule_contracts,
    )?;
    let mut economic_policy = Vec::new();
    for power_ref in &economic_family.card_refs {
        let power = source
            .powers
            .iter()
            .find(|row| row.id == *power_ref)
            .ok_or_else(|| LedgerError::new(format!("{power_ref}: economic power is missing")))?;
        let allocation = allocations.get(power.id.as_str()).ok_or_else(|| {
            LedgerError::new(format!(
                "{}: economic function allocation missing",
                power.id
            ))
        })?;
        if !power
            .prohibited_inputs
            .iter()
            .any(|value| value.contains("custody T3"))
        {
            return Err(LedgerError::new(format!(
                "{}: economic power can borrow the custody T3 record",
                power.id
            )));
        }
        economic_policy.push(EconomicPowerPolicyProjection {
            id: &power.id,
            holder_body_refs: &power.holder_body_refs,
            holder_role_refs: &power.holder_role_refs,
            decisive_fact_writer_body_refs: &allocation.decisive_fact_writer_body_refs,
            decisive_fact_writer_role_refs: &allocation.decisive_fact_writer_role_refs,
            decider_body_refs: &allocation.decider_body_refs,
            decider_role_refs: &allocation.decider_role_refs,
            executor_body_refs: &allocation.executor_body_refs,
            executor_role_refs: &allocation.executor_role_refs,
            auditor_body_refs: &allocation.auditor_body_refs,
            auditor_role_refs: &allocation.auditor_role_refs,
            final_remedy_body_refs: &allocation.final_remedy_body_refs,
            final_remedy_role_refs: &allocation.final_remedy_role_refs,
            required_separation_pairs: &power.required_separation_pairs,
            prohibited_inputs: &power.prohibited_inputs,
            negative_status: &power.negative_test.status,
            negative_executable_ref: &power.negative_test.executable_ref.0,
            counterfactual_status: &power.counterfactual.status,
            counterfactual_executable_ref: &power.counterfactual.executable_ref.0,
            part_v_status: &power.part_v_status,
        });
    }
    if typed_fingerprint(&economic_policy, "economic power semantic mapping")?
        != "99be72da9b4006c8efb9f7ebe8a7b7b4c227ce9b1462cdca65c466ae1d31842e"
    {
        return Err(LedgerError::new(
            "economic holder, function, temporal wall, test, or Part V mapping differs from checker policy",
        ));
    }

    let economic_effect_family = source
        .coverage_families
        .iter()
        .find(|row| row.id == "FS-CVF-017")
        .ok_or_else(|| LedgerError::new("FS-CVF-017 economic effect family is missing"))?;
    validate_economic_effect_term_contracts(inputs, source, economic_effect_family)?;
    for effect_ref in &economic_effect_family.effect_refs {
        let effect = source
            .constitutional_effects
            .iter()
            .find(|row| row.id == *effect_ref)
            .ok_or_else(|| LedgerError::new(format!("{effect_ref}: economic effect is missing")))?;
        let prohibited = effect.prohibited_inputs.join(" ").to_ascii_lowercase();
        for sealed in [
            "raw work",
            "reward",
            "home",
            "family",
            "public label",
            "custody t3",
            "book 2 model",
        ] {
            if !prohibited.contains(sealed) {
                return Err(LedgerError::new(format!(
                    "{effect_ref}: economic effect no longer seals {sealed}"
                )));
            }
        }
    }
    Ok(())
}

fn register_ids(source: &LedgerDocument) -> LedgerResult<BTreeSet<String>> {
    let expected_registry = BTreeMap::from([
        ("FS-DOM", "domain"),
        ("FS-LGR", "legacy_row"),
        ("FS-CLM", "claim"),
        ("FS-BOD", "body"),
        ("FS-RTE", "assurance_route"),
        ("FS-EXA", "external_assumption"),
        ("FS-ENV", "envelope"),
        ("FS-ROL", "role"),
        ("FS-POW", "power"),
        ("FS-PCT", "power_contract_template"),
        ("FS-PRF", "power_refusal"),
        ("FS-PCD", "power_crosswalk_disposition"),
        ("FS-DEP", "dependency"),
        ("FS-SCN", "scenario"),
        ("FS-THR", "threshold"),
        ("FS-DFT", "defect"),
        ("FS-RCP", "resolution_receipt"),
        ("FS-PRO", "proposal"),
        ("FS-REV", "review_event"),
        ("FS-CLR", "closure_requirement_profile"),
        ("FS-MAL", "model_allocation"),
        ("FS-FAL", "function_allocation"),
        ("FS-LOP", "dependency_loop"),
        ("FS-LHC", "loop_hazard_control"),
        ("FS-BTL", "bottleneck_disposition"),
        ("FS-CCT", "closure_claim_contract"),
        ("FS-COM", "review_commission"),
        ("FS-SAU", "scope_audit"),
        ("FS-CVF", "coverage_family"),
        ("FS-CCE", "constitutional_effect"),
    ])
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect::<BTreeMap<_, _>>();
    if source.id_registry != expected_registry {
        return Err(LedgerError::new(
            "id_registry must equal the closed checker-owned record registry",
        ));
    }
    let groups: [(&str, Vec<&str>); 30] = [
        (
            "FS-DOM",
            source.domains.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-LGR",
            source
                .legacy_rows
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-CLM",
            source.claims.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-BOD",
            source.bodies.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-RTE",
            source.routes.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-EXA",
            source
                .external_assumptions
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-ENV",
            source.envelope.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-ROL",
            source.roles.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-POW",
            source.powers.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-PCT",
            source
                .power_contract_templates
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-PRF",
            source
                .power_refusals
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-PCD",
            source
                .power_crosswalk_dispositions
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-DEP",
            source
                .dependencies
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-SCN",
            source.scenarios.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-THR",
            source
                .thresholds
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-DFT",
            source.defects.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-RCP",
            source.receipts.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-PRO",
            source.proposals.iter().map(|row| row.id.as_str()).collect(),
        ),
        (
            "FS-REV",
            source
                .review_events
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-CLR",
            source
                .closure_requirement_profiles
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-MAL",
            source
                .model_allocations
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-FAL",
            source
                .function_allocations
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-LOP",
            source
                .dependency_loops
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-LHC",
            source
                .loop_hazard_controls
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-BTL",
            source
                .bottleneck_dispositions
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-CCT",
            source
                .closure_claim_contracts
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-COM",
            source
                .review_commissions
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-SAU",
            source
                .scope_audits
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-CVF",
            source
                .coverage_families
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
        (
            "FS-CCE",
            source
                .constitutional_effects
                .iter()
                .map(|row| row.id.as_str())
                .collect(),
        ),
    ];
    let mut all = BTreeSet::new();
    for (prefix, ids) in groups {
        for id in ids {
            let suffix = id.strip_prefix(&format!("{prefix}-"));
            let numeric = suffix.is_some_and(|value| {
                (2..=3).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_digit())
            });
            let pending_audit = prefix == "FS-SAU"
                && suffix
                    .and_then(|value| value.strip_suffix("-PENDING"))
                    .is_some_and(|value| {
                        (2..=3).contains(&value.len())
                            && value.chars().all(|ch| ch.is_ascii_digit())
                    });
            if !numeric && !pending_audit {
                return Err(LedgerError::new(format!(
                    "{id}: identifier does not match its {prefix} registry"
                )));
            }
            if !all.insert(id.to_owned()) {
                return Err(LedgerError::new(format!("duplicate record id {id}")));
            }
        }
    }
    Ok(all)
}

fn validate_common_record(
    id: &str,
    title: &str,
    applicability: &str,
    status: &str,
    severity: &str,
    consequence: &str,
    owner_ref: &str,
    closure_condition: &str,
) -> LedgerResult<()> {
    for (field, value) in [
        ("id", id),
        ("title", title),
        ("applicability", applicability),
        ("status", status),
        ("severity", severity),
        ("consequence", consequence),
        ("owner_ref", owner_ref),
        ("closure_condition", closure_condition),
    ] {
        nonempty(value, &format!("{id}.{field}"))?;
    }
    if status
        .to_ascii_lowercase()
        .contains("partial formalisation")
    {
        return Err(LedgerError::new(format!(
            "{id}: `partial formalisation` is retired; split the row"
        )));
    }
    Ok(())
}

fn validate_unresolved_detail(value: &UnresolvedDetail, context: &str) -> LedgerResult<()> {
    for (field, text) in [
        ("severity", value.severity.as_str()),
        ("consequence", value.consequence.as_str()),
        ("owner_ref", value.owner_ref.as_str()),
        ("closure_condition", value.closure_condition.as_str()),
        (
            "public_claim_limitation",
            value.public_claim_limitation.as_str(),
        ),
    ] {
        nonempty(text, &format!("{context}.{field}"))?;
    }
    Ok(())
}

fn validate_domain_bucket(value: &DomainBucket, context: &str) -> LedgerResult<()> {
    match value {
        DomainBucket::Answer(value) => {
            nonempty(&value.answer, &format!("{context}.answer"))?;
            unique_strings(&value.refs, &format!("{context}.refs"), false)?;
        }
        DomainBucket::Routing(value) => {
            if !ROUTING_MARKERS.contains(&value.routing_marker.as_str()) {
                return Err(LedgerError::new(format!(
                    "{context}: routing_marker is outside its closed enum"
                )));
            }
            nonempty(&value.note, &format!("{context}.note"))?;
        }
        DomainBucket::Unresolved(value) => {
            validate_unresolved_detail(&value.unresolved, &format!("{context}.unresolved"))?;
        }
    }
    Ok(())
}

fn expected_defect_gate_refs(id: &str) -> LedgerResult<&'static [&'static str]> {
    const ALL: &[&str] = &["gate-a", "gate-b", "gate-c", "gate-d", "gate-e"];
    const BOOK_ONE_ONWARD: &[&str] = &["gate-b", "gate-c", "gate-d", "gate-e"];
    const BOOK_TWO: &[&str] = &["gate-d", "gate-e"];
    const NONE: &[&str] = &[];
    match id {
        "FS-DFT-13" | "FS-DFT-14" | "FS-DFT-27" | "FS-DFT-40" => Ok(ALL),
        "FS-DFT-20" => Ok(NONE),
        "FS-DFT-17" | "FS-DFT-28" | "FS-DFT-29" | "FS-DFT-36" | "FS-DFT-37" | "FS-DFT-38" => {
            Ok(BOOK_TWO)
        }
        _ => {
            let number = id
                .strip_prefix("FS-DFT-")
                .and_then(|value| value.parse::<u8>().ok());
            if number.is_some_and(|value| (1..=41).contains(&value)) {
                Ok(BOOK_ONE_ONWARD)
            } else {
                Err(LedgerError::new(format!(
                    "{id}: checker-owned gate-applicability contract has no match"
                )))
            }
        }
    }
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_role_records(source: &LedgerDocument) -> LedgerResult<()> {
    if source.roles.is_empty() {
        if !source.role_omissions.is_empty() {
            return Err(LedgerError::new(
                "role_omissions may not exist while roles is deferred",
            ));
        }
        return Ok(());
    }
    let domain_ids = source
        .domains
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let body_ids = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let role_ids = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let mut cited_domains = HashSet::new();
    let mut exercised_scales = HashSet::new();
    let mut body_positions: HashMap<&str, HashSet<&str>> = HashMap::new();
    for role in &source.roles {
        validate_common_record(
            &role.id,
            &role.title,
            &role.applicability,
            &role.status,
            &role.severity,
            &role.consequence,
            &role.owner_ref,
            &role.closure_condition,
        )?;
        if role.layer != "constitutional-invariant"
            || !ROLE_KINDS.contains(&role.role_kind.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: role layer or kind is invalid",
                role.id
            )));
        }
        unique_strings(
            &role.domain_refs,
            &format!("{}.domain_refs", role.id),
            false,
        )?;
        if role
            .domain_refs
            .iter()
            .any(|reference| !domain_ids.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: role names an unknown domain",
                role.id
            )));
        }
        cited_domains.extend(role.domain_refs.iter().map(String::as_str));
        unique_strings(&role.scales, &format!("{}.scales", role.id), false)?;
        if role
            .scales
            .iter()
            .any(|scale| !ROLE_SCALES.contains(&scale.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: scales are outside the closed enum",
                role.id
            )));
        }
        exercised_scales.extend(role.scales.iter().map(String::as_str));
        for position in &role.power_positions {
            if !body_ids.contains(position.body_ref.as_str())
                || !POWER_POSITIONS.contains(&position.position.as_str())
            {
                return Err(LedgerError::new(format!(
                    "{}: power position has an unknown body or position",
                    role.id
                )));
            }
            nonempty(&position.note, &format!("{}.power_positions.note", role.id))?;
            body_positions
                .entry(position.body_ref.as_str())
                .or_default()
                .insert(position.position.as_str());
        }
        if let Some(held) = &role.power_held {
            nonempty(&held.power, &format!("{}.power_held.power", role.id))?;
            unique_strings(
                &held.affected_role_refs,
                &format!("{}.power_held.affected_role_refs", role.id),
                false,
            )?;
            unique_strings(
                &held.checking_refs,
                &format!("{}.power_held.checking_refs", role.id),
                false,
            )?;
            if held
                .affected_role_refs
                .iter()
                .any(|reference| !role_ids.contains(reference.as_str()) || reference == &role.id)
                || held.checking_refs.iter().any(|reference| {
                    !role_ids.contains(reference.as_str()) && !body_ids.contains(reference.as_str())
                })
            {
                return Err(LedgerError::new(format!(
                    "{}: power_held has an invalid affected or checking reference",
                    role.id
                )));
            }
        }
        if !ROLE_ANCHORS.contains(&role.formal_anchor.anchor.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: formal anchor is outside its closed enum",
                role.id
            )));
        }
        unique_strings(
            &role.formal_anchor.refs,
            &format!("{}.formal_anchor.refs", role.id),
            false,
        )?;
        if role
            .formal_anchor
            .anchor
            .starts_with("constitution-predicate")
            && !role.formal_anchor.refs.iter().any(|reference| {
                reference
                    .split_once("::")
                    .is_some_and(|(path, _)| path.ends_with(".nibli"))
            })
        {
            return Err(LedgerError::new(format!(
                "{}: constitution-predicate anchor must cite a .nibli source",
                role.id
            )));
        }
        nonempty(
            &role.floor_invariance,
            &format!("{}.floor_invariance", role.id),
        )?;
        unique_strings(
            &role.source_refs,
            &format!("{}.source_refs", role.id),
            false,
        )?;
    }
    if cited_domains != domain_ids {
        return Err(LedgerError::new(
            "role/domain closure: every material domain must be cited",
        ));
    }
    if exercised_scales
        != ROLE_SCALES
            .iter()
            .copied()
            .collect::<HashSet<&'static str>>()
    {
        return Err(LedgerError::new(
            "role/scale closure: every named scale must be exercised",
        ));
    }
    let expected_positions = POWER_POSITIONS.iter().copied().collect::<HashSet<_>>();
    for body_id in &body_ids {
        if body_positions
            .get(body_id)
            .is_none_or(|positions| positions != &expected_positions)
        {
            return Err(LedgerError::new(format!(
                "power-position closure: body {body_id} needs affected and checking positions"
            )));
        }
    }
    if source.role_omissions.is_empty() {
        return Err(LedgerError::new(
            "role_omissions must be non-empty once roles populate",
        ));
    }
    let roles_by_id = source
        .roles
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut omission_keys = HashSet::new();
    for omission in &source.role_omissions {
        let key = match omission {
            RoleOmission::Role(value) => {
                nonempty(&value.omitted_role, "role_omissions.omitted_role")?;
                nonempty(&value.risk_reason, "role_omissions.risk_reason")?;
                ("role".to_owned(), value.omitted_role.clone())
            }
            RoleOmission::Scale(value) => {
                let role = roles_by_id
                    .get(value.role_ref.as_str())
                    .ok_or_else(|| LedgerError::new("role omission names an unknown role"))?;
                if !ROLE_SCALES.contains(&value.omitted_scale.as_str())
                    || role.scales.contains(&value.omitted_scale)
                {
                    return Err(LedgerError::new("role scale omission is unknown or stale"));
                }
                nonempty(&value.risk_reason, "role_omissions.risk_reason")?;
                (value.role_ref.clone(), value.omitted_scale.clone())
            }
            RoleOmission::Domain(value) => {
                let role = roles_by_id
                    .get(value.role_ref.as_str())
                    .ok_or_else(|| LedgerError::new("role omission names an unknown role"))?;
                if !domain_ids.contains(value.omitted_domain_ref.as_str())
                    || role.domain_refs.contains(&value.omitted_domain_ref)
                {
                    return Err(LedgerError::new("role domain omission is unknown or stale"));
                }
                nonempty(&value.risk_reason, "role_omissions.risk_reason")?;
                (value.role_ref.clone(), value.omitted_domain_ref.clone())
            }
        };
        if !omission_keys.insert(key) {
            return Err(LedgerError::new("duplicate role omission"));
        }
    }
    Ok(())
}

fn body_register_guard(text: &str, context: &str) -> LedgerResult<()> {
    const ARRIVAL_PHRASES: [&str; 13] = [
        "is delivered",
        "are delivered",
        "was delivered",
        "were delivered",
        "the remedy arrives",
        "the remedy reaches",
        "the election occurs",
        "the election happens",
        "the election takes place",
        "the body acts",
        "will act",
        "guarantees delivery",
        "actually arrives",
    ];
    const FEASIBILITY_TOKENS: [&str; 6] = [
        "feasible",
        "feasibility",
        "affordable",
        "affordability",
        "cost-effective",
        "capacity to deliver",
    ];
    const FIXTURE_RELABELS: [&str; 8] = [
        "convocation is the executive council",
        "convocation as the executive council",
        "court is the constitutional court",
        "current court as the constitutional court",
        "state is a completed federal",
        "state as a completed federal government",
        "assembly constant is the people's assembly",
        "electorate constant is the electorate",
    ];
    let lower = text.to_ascii_lowercase();
    if let Some(phrase) = ARRIVAL_PHRASES
        .iter()
        .find(|phrase| lower.contains(**phrase))
    {
        return Err(LedgerError::new(format!(
            "{context}: arrival register is refused ({phrase})"
        )));
    }
    let tokens = lower
        .split_whitespace()
        .map(|token| token.trim_matches(|character: char| !character.is_ascii_alphanumeric()))
        .collect::<Vec<_>>();
    let aggregate = lower
        .split_whitespace()
        .any(|token| token.contains('%') && token.bytes().any(|byte| byte.is_ascii_digit()))
        || tokens.windows(3).any(|window| {
            window[0].bytes().all(|byte| byte.is_ascii_digit())
                && window[1] == "of"
                && window[2].bytes().all(|byte| byte.is_ascii_digit())
        })
        || tokens.windows(4).any(|window| {
            window[0].bytes().all(|byte| byte.is_ascii_digit())
                && window[1] == "out"
                && window[2] == "of"
                && window[3].bytes().all(|byte| byte.is_ascii_digit())
        });
    if aggregate {
        return Err(LedgerError::new(format!(
            "{context}: aggregate figure is refused"
        )));
    }
    if let Some(token) = FEASIBILITY_TOKENS
        .iter()
        .find(|token| lower.contains(**token))
    {
        return Err(LedgerError::new(format!(
            "{context}: feasibility claim is refused ({token})"
        )));
    }
    if FIXTURE_RELABELS
        .iter()
        .any(|relabel| lower.contains(relabel))
    {
        return Err(LedgerError::new(format!(
            "{context}: may not relabel a current fixture"
        )));
    }
    Ok(())
}

fn validate_body_term(body: &Body, term: &Term, context: &str) -> LedgerResult<()> {
    if term
        .source_refs
        .iter()
        .any(|reference| !body.source_refs.contains(reference))
    {
        return Err(LedgerError::new(format!(
            "{context}: term source must be a card source"
        )));
    }
    body_register_guard(&term.text, context)
}

fn validate_body_records(source: &LedgerDocument) -> LedgerResult<()> {
    let body_ids = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let role_ids = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    const DELEGATED_REQUIRED: [&str; 9] = [
        "FS-BOD-02",
        "FS-BOD-03",
        "FS-BOD-04",
        "FS-BOD-05",
        "FS-BOD-17",
        "FS-BOD-18",
        "FS-BOD-19",
        "FS-BOD-24",
        "FS-BOD-25",
    ];
    for body in &source.bodies {
        validate_common_record(
            &body.id,
            &body.title,
            &body.applicability,
            &body.status,
            &body.severity,
            &body.consequence,
            &body.owner_ref,
            &body.closure_condition,
        )?;
        if body.layer != "constitutional-invariant"
            || !BODY_KINDS.contains(&body.body_kind.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: body layer or body_kind is invalid",
                body.id
            )));
        }
        for (field, value) in [
            ("job", body.job.as_str()),
            ("may_not_do_alone", body.may_not_do_alone.as_str()),
            ("required_check", body.required_check.as_str()),
            ("book2_handoff", body.book2_handoff.as_str()),
        ] {
            nonempty(value, &format!("{}.{field}", body.id))?;
        }
        unique_strings(
            &body.source_refs,
            &format!("{}.source_refs", body.id),
            false,
        )?;
        if !body.source_refs.contains(&body.source_ref) {
            return Err(LedgerError::new(format!(
                "{}: source_refs must contain the rendered source_ref",
                body.id
            )));
        }
        for (name, term) in [
            (
                "universal_human_standing",
                &body.status_senses.universal_human_standing,
            ),
            (
                "political_membership",
                &body.status_senses.political_membership,
            ),
            ("franchise", &body.status_senses.franchise),
            ("candidacy", &body.status_senses.candidacy),
            ("current_office", &body.status_senses.current_office),
            (
                "current_lawful_power",
                &body.status_senses.current_lawful_power,
            ),
            (
                "permanent_historical_public_answerability",
                &body.status_senses.permanent_historical_public_answerability,
            ),
        ] {
            validate_term(term, &format!("{}.status_senses.{name}", body.id))?;
            validate_body_term(body, term, &format!("{}.status_senses.{name}", body.id))?;
            if matches!(
                name,
                "current_office"
                    | "current_lawful_power"
                    | "permanent_historical_public_answerability"
            ) && term
                .text
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| word.eq_ignore_ascii_case("standing"))
            {
                return Err(LedgerError::new(format!(
                    "{}.status_senses.{name}: standing is reserved for universal personhood",
                    body.id
                )));
            }
        }
        for (name, term) in [
            ("democratic_source", &body.office_contract.democratic_source),
            ("jurisdiction", &body.office_contract.jurisdiction),
            ("ordinary_function", &body.office_contract.ordinary_function),
            (
                "delegation_boundary",
                &body.office_contract.delegation_boundary,
            ),
            (
                "conflict_and_recusal",
                &body.office_contract.conflict_and_recusal,
            ),
            ("appointment", &body.office_contract.appointment),
            ("removal", &body.office_contract.removal),
            ("succession", &body.office_contract.succession),
            ("temporal_status", &body.office_contract.temporal_status),
            (
                "public_reason_duty",
                &body.office_contract.public_reason_duty,
            ),
            ("anti_capture", &body.office_contract.anti_capture),
        ] {
            validate_term(term, &format!("{}.office_contract.{name}", body.id))?;
            validate_body_term(body, term, &format!("{}.office_contract.{name}", body.id))?;
        }
        if body.office_contract.ordinary_function.text.trim() == body.job.trim() {
            return Err(LedgerError::new(format!(
                "{}: ordinary_function must expand the rendered job",
                body.id
            )));
        }
        if body.accountability_routes.is_empty() {
            return Err(LedgerError::new(format!(
                "{}: at least one accountability route is required",
                body.id
            )));
        }
        let mut route_types = HashSet::new();
        let mut external_checker = false;
        for route in &body.accountability_routes {
            if !ACCOUNTABILITY_ROUTE_TYPES.contains(&route.route_type.as_str())
                || !route_types.insert(route.route_type.as_str())
            {
                return Err(LedgerError::new(format!(
                    "{}: accountability route type is invalid or duplicated",
                    body.id
                )));
            }
            unique_strings(
                &route.checker_body_refs,
                &format!("{}.accountability.checker_body_refs", body.id),
                true,
            )?;
            unique_strings(
                &route.checker_role_refs,
                &format!("{}.accountability.checker_role_refs", body.id),
                true,
            )?;
            if route
                .checker_body_refs
                .iter()
                .any(|reference| !body_ids.contains(reference.as_str()) || reference == &body.id)
                || route
                    .checker_role_refs
                    .iter()
                    .any(|reference| !role_ids.contains(reference.as_str()))
            {
                return Err(LedgerError::new(format!(
                    "{}: accountability route has invalid or self-checking references",
                    body.id
                )));
            }
            external_checker |= !route.checker_body_refs.is_empty();
            validate_term(&route.term, &format!("{}.accountability.term", body.id))?;
            validate_body_term(
                body,
                &route.term,
                &format!("{}.accountability.term", body.id),
            )?;
        }
        if !external_checker {
            return Err(LedgerError::new(format!(
                "{}: at least one accountability route needs a checking body",
                body.id
            )));
        }
        if !ADVERSE_DETERMINATION_KINDS.contains(&body.adverse_determinations.kind.as_str())
            || (body.adverse_determinations.kind == "none-by-design"
                && !body.adverse_determinations.items.is_empty())
            || (body.adverse_determinations.kind == "enumerated"
                && body.adverse_determinations.items.is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: adverse determination contract is invalid",
                body.id
            )));
        }
        validate_term(
            &body.adverse_determinations.note,
            &format!("{}.adverse_determinations.note", body.id),
        )?;
        validate_body_term(
            body,
            &body.adverse_determinations.note,
            &format!("{}.adverse_determinations.note", body.id),
        )?;
        for item in &body.adverse_determinations.items {
            nonempty(&item.name, &format!("{}.adverse_item.name", body.id))?;
            nonempty(&item.subject, &format!("{}.adverse_item.subject", body.id))?;
            validate_term(&item.appeal, &format!("{}.adverse_item.appeal", body.id))?;
            validate_term(&item.remedy, &format!("{}.adverse_item.remedy", body.id))?;
            validate_body_term(
                body,
                &item.appeal,
                &format!("{}.adverse_item.appeal", body.id),
            )?;
            validate_body_term(
                body,
                &item.remedy,
                &format!("{}.adverse_item.remedy", body.id),
            )?;
        }
        nonempty(
            &body.temporal_contract.contract_kind,
            &format!("{}.temporal_contract.contract_kind", body.id),
        )?;
        if !CUSTODY_T3_RELATIONS.contains(&body.temporal_contract.custody_t3_relation.as_str())
            || (body.temporal_contract.custody_t3_relation == "retained-application"
                && body.id != CUSTODY_T3_APPLICANT)
        {
            return Err(LedgerError::new(format!(
                "{}: custody T3 relation is invalid or reused",
                body.id
            )));
        }
        for (name, term) in [
            ("term", &body.temporal_contract.term),
            ("failure_polarity", &body.temporal_contract.failure_polarity),
            ("expiry_default", &body.temporal_contract.expiry_default),
        ] {
            validate_term(term, &format!("{}.temporal_contract.{name}", body.id))?;
            validate_body_term(body, term, &format!("{}.temporal_contract.{name}", body.id))?;
            if body.id != CUSTODY_T3_APPLICANT
                && term.source_refs.iter().any(|reference| {
                    [
                        "book-1-time-model-decision.md",
                        "temporal-assurance-case.json",
                        "temporal-assurance-case.md",
                    ]
                    .iter()
                    .any(|marker| reference.contains(marker))
                })
            {
                return Err(LedgerError::new(format!(
                    "{}.temporal_contract.{name}: custody T3 is not reusable",
                    body.id
                )));
            }
        }
        if DELEGATED_REQUIRED.contains(&body.id.as_str()) && body.delegated_mechanics.is_empty() {
            return Err(LedgerError::new(format!(
                "{}: required bounded delegated mechanics are missing",
                body.id
            )));
        }
        for (index, term) in body.delegated_mechanics.iter().enumerate() {
            validate_term(term, &format!("{}.delegated_mechanics[{index}]", body.id))?;
            validate_body_term(
                body,
                term,
                &format!("{}.delegated_mechanics[{index}]", body.id),
            )?;
            if term.basis != "bounded-delegation" {
                return Err(LedgerError::new(format!(
                    "{}: delegated mechanics must declare bounded-delegation",
                    body.id
                )));
            }
        }
    }
    Ok(())
}

fn normalise_body_title(title: &str) -> String {
    title
        .replace(" / ", " and ")
        .replace('/', " and ")
        .trim()
        .to_ascii_lowercase()
}

fn validate_body_map_cells(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    let map = std::str::from_utf8(input_bytes(inputs, COVERAGE_MAP)?)
        .map_err(|_| LedgerError::new("coverage map is not UTF-8"))?;
    let section = map
        .split_once("## 5. Required bodies")
        .and_then(|(_, tail)| tail.split_once("\n## 6."))
        .map(|(section, _)| section)
        .ok_or_else(|| LedgerError::new("coverage map has no required-bodies section"))?;
    let mut rows = BTreeMap::<String, [String; 3]>::new();
    for line in section.lines().filter(|line| {
        line.starts_with("| ") && !line.starts_with("| ---") && !line.starts_with("| Body")
    }) {
        let cells = line
            .trim()
            .trim_matches('|')
            .split(" | ")
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() == 4 {
            rows.insert(
                normalise_body_title(cells[0]),
                [cells[1].into(), cells[2].into(), cells[3].into()],
            );
        }
    }
    if rows.is_empty() {
        return Err(LedgerError::new(
            "coverage-map required-bodies table parsed to no rows",
        ));
    }
    let mut unmatched = rows.keys().cloned().collect::<BTreeSet<_>>();
    for body in &source.bodies {
        let key = normalise_body_title(&body.title);
        let Some(expected) = rows.get(&key) else {
            continue;
        };
        unmatched.remove(&key);
        for (field, actual, expected) in [
            ("job", body.job.as_str(), expected[0].as_str()),
            (
                "may_not_do_alone",
                body.may_not_do_alone.as_str(),
                expected[1].as_str(),
            ),
            (
                "required_check",
                body.required_check.as_str(),
                expected[2].as_str(),
            ),
        ] {
            if actual.trim() != expected.trim() {
                return Err(LedgerError::new(format!(
                    "{}.{} drifted from the coverage map row it cites",
                    body.id, field
                )));
            }
        }
    }
    if !unmatched.is_empty() {
        return Err(LedgerError::new(format!(
            "every required-bodies row must bind a body card: {unmatched:?}"
        )));
    }
    Ok(())
}

fn validate_envelope_criteria_thresholds(source: &LedgerDocument) -> LedgerResult<()> {
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    if source.envelope.is_empty() || source.envelope[0].id != ENVELOPE_STUB_ID {
        return Err(LedgerError::new(format!(
            "the envelope array must begin with {ENVELOPE_STUB_ID}"
        )));
    }
    for (index, envelope) in source.envelope.iter().enumerate() {
        validate_common_record(
            &envelope.id,
            &envelope.title,
            &envelope.applicability,
            &envelope.status,
            &envelope.severity,
            &envelope.consequence,
            &envelope.owner_ref,
            &envelope.closure_condition,
        )?;
        if !ENVELOPE_STATUSES.contains(&envelope.envelope_status.as_str())
            || envelope.envelope_status == "calibrated"
            || envelope.layer != "external-assumption"
        {
            return Err(LedgerError::new(format!(
                "{}: envelope status or layer is invalid for Book 1",
                envelope.id
            )));
        }
        nonempty(&envelope.note, &format!("{}.note", envelope.id))?;
        if index == 0 {
            if envelope.envelope_status != "stub"
                || envelope.status != "pre-envelope-identity"
                || envelope.envelope_version.is_some()
                || envelope.fields.is_some()
            {
                return Err(LedgerError::new(format!(
                    "{}: permanent stub shape or identity drifted",
                    envelope.id
                )));
            }
            continue;
        }
        if envelope.envelope_status != "versioned-structure"
            || envelope
                .envelope_version
                .as_deref()
                .is_none_or(|version| version.trim().is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: successor must be a named versioned structure",
                envelope.id
            )));
        }
        let fields = envelope
            .fields
            .as_ref()
            .filter(|values| !values.is_empty())
            .ok_or_else(|| {
                LedgerError::new(format!("{}: versioned envelope needs fields", envelope.id))
            })?;
        let mut ids = HashSet::new();
        let mut dependents = HashSet::new();
        for field in fields {
            if !is_slug(&field.id) || !ids.insert(field.id.as_str()) {
                return Err(LedgerError::new(format!(
                    "{}: envelope field id is invalid or duplicated",
                    envelope.id
                )));
            }
            nonempty(
                &field.definition,
                &format!("{}.fields.definition", envelope.id),
            )?;
            nonempty(
                &field.invariance,
                &format!("{}.fields.invariance", envelope.id),
            )?;
            if !VALUE_STATUSES.contains(&field.value_status.as_str()) {
                return Err(LedgerError::new(format!(
                    "{}: envelope field value_status is invalid",
                    envelope.id
                )));
            }
            unique_strings(
                &field.dependents,
                &format!("{}.fields.dependents", envelope.id),
                true,
            )?;
            for reference in &field.dependents {
                let claim = claims.get(reference.as_str()).ok_or_else(|| {
                    LedgerError::new(format!(
                        "{}: envelope dependent is not a claim",
                        envelope.id
                    ))
                })?;
                if claim.layer == "constitutional-invariant"
                    && matches!(claim.posture.as_str(), "Derived" | "Checked")
                {
                    return Err(LedgerError::new(format!(
                        "{}: established constitutional norm may not depend on envelope values",
                        envelope.id
                    )));
                }
                dependents.insert(reference.as_str());
            }
        }
        for required in ["FS-CLM-06", "FS-CLM-20"] {
            if !dependents.contains(required) {
                return Err(LedgerError::new(format!(
                    "{}: envelope-relative claim {required} is absent",
                    envelope.id
                )));
            }
        }
    }
    nonempty(
        &source.functional_criteria.drift_note,
        "functional_criteria.drift_note",
    )?;
    let mut criteria = HashSet::new();
    for criterion in &source.functional_criteria.criteria {
        if !CRITERIA_SLUGS.contains(&criterion.id.as_str())
            || !criteria.insert(criterion.id.as_str())
        {
            return Err(LedgerError::new(format!(
                "functional criterion {} is unknown or duplicated",
                criterion.id
            )));
        }
        nonempty(&criterion.name, &format!("{}.name", criterion.id))?;
        nonempty(
            &criterion.definition,
            &format!("{}.definition", criterion.id),
        )?;
        unique_strings(
            &criterion.binding_refs,
            &format!("{}.binding_refs", criterion.id),
            false,
        )?;
        unique_strings(
            &criterion.provenance,
            &format!("{}.provenance", criterion.id),
            false,
        )?;
    }
    if criteria != CRITERIA_SLUGS.iter().copied().collect::<HashSet<_>>() {
        return Err(LedgerError::new(
            "functional_criteria must carry exactly the seven-member union",
        ));
    }
    let domains = source
        .domains
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    for threshold in &source.thresholds {
        validate_common_record(
            &threshold.id,
            &threshold.title,
            &threshold.applicability,
            &threshold.status,
            &threshold.severity,
            &threshold.consequence,
            &threshold.owner_ref,
            &threshold.closure_condition,
        )?;
        if !CRITERIA_SLUGS.contains(&threshold.criterion_ref.as_str())
            || !LAWFUL_SOURCES.contains(&threshold.lawful_source.as_str())
            || !VALUE_STATUSES.contains(&threshold.value_status.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: threshold criterion, lawful source, or value status is invalid",
                threshold.id
            )));
        }
        let expected_layer = match threshold.lawful_source.as_str() {
            "constitutional-minimum-or-ceiling" => "constitutional-invariant",
            "democratic-policy-target" => "democratic-ordinary-law-choice",
            "scientific-safety-boundary" => "external-assumption",
            "operational-diagnostic" => "book-2-operation",
            _ => unreachable!("lawful source checked above"),
        };
        if threshold.layer != expected_layer {
            return Err(LedgerError::new(format!(
                "{}: layer does not follow its lawful source",
                threshold.id
            )));
        }
        unique_strings(
            &threshold.domain_refs,
            &format!("{}.domain_refs", threshold.id),
            true,
        )?;
        if threshold
            .domain_refs
            .iter()
            .any(|reference| !domains.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: threshold names an unknown domain",
                threshold.id
            )));
        }
        nonempty(
            &threshold.definition,
            &format!("{}.definition", threshold.id),
        )?;
        if threshold
            .definition
            .chars()
            .any(|character| character.is_ascii_digit())
        {
            return Err(LedgerError::new(format!(
                "{}: Book 1 threshold definitions may not contain numbers",
                threshold.id
            )));
        }
    }
    Ok(())
}

fn reachable<'a>(
    adjacency: &HashMap<&'a str, HashSet<&'a str>>,
    start: &'a str,
) -> HashSet<&'a str> {
    let mut seen = HashSet::new();
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        if let Some(next) = adjacency.get(node) {
            for target in next {
                if seen.insert(*target) {
                    stack.push(target);
                }
            }
        }
    }
    seen
}

fn valid_closure_component_token(value: &str) -> bool {
    let Some((requirement, component)) = value.split_once(':') else {
        return false;
    };
    requirement.strip_prefix("FS-CLR-").is_some_and(|number| {
        !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
    }) && is_slug(component)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_content_addressed_receipt_ref(value: &str) -> bool {
    const PREFIX: &str = "new-book-plans/verification-receipts/sha256-";
    value
        .strip_prefix(PREFIX)
        .and_then(|tail| tail.strip_suffix(".json"))
        .is_some_and(|digest| is_lower_hex(digest, 64))
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn is_utc_instant(value: &str) -> bool {
    if value.len() != 20
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || value.as_bytes()[10] != b'T'
        || value.as_bytes()[13] != b':'
        || value.as_bytes()[16] != b':'
        || value.as_bytes()[19] != b'Z'
        || !value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
    {
        return false;
    }
    value[11..13].parse::<u8>().is_ok_and(|hour| hour < 24)
        && value[14..16].parse::<u8>().is_ok_and(|minute| minute < 60)
        && value[17..19].parse::<u8>().is_ok_and(|second| second < 60)
}

fn validate_dependency_records(source: &LedgerDocument) -> LedgerResult<()> {
    if source.dependencies.is_empty() {
        if !source.dependency_loops.is_empty() || !source.refused_flows.is_empty() {
            return Err(LedgerError::new(
                "dependency loops and refused flows require a populated dependency map",
            ));
        }
        return Ok(());
    }
    let domains = source
        .domains
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let bodies = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let roles = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let assumptions = source
        .external_assumptions
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let defects = source
        .defects
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let mut edges = HashMap::new();
    let mut triples = HashSet::new();
    let mut touched_domains = HashSet::new();
    let mut exercised = HashSet::new();
    let mut cited_assumptions = HashSet::new();
    for dependency in &source.dependencies {
        validate_common_record(
            &dependency.id,
            &dependency.title,
            &dependency.applicability,
            &dependency.status,
            &dependency.severity,
            &dependency.consequence,
            &dependency.owner_ref,
            &dependency.closure_condition,
        )?;
        if !bodies.contains(dependency.steward_ref.as_str())
            || !FLOW_KINDS.contains(&dependency.flow_kind.as_str())
            || !DEPENDENCY_CLASSES.contains(&dependency.dependency_class.as_str())
            || !LIFECYCLE_PATHS.contains(&dependency.lifecycle_path.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: dependency steward or closed classification is invalid",
                dependency.id
            )));
        }
        let expected_layer = match dependency.dependency_class.as_str() {
            "constitutionally-guaranteed" => "constitutional-invariant",
            "democratically-selected" => "democratic-ordinary-law-choice",
            "operationally-supplied" => "book-2-operation",
            "externally-assumed" => "external-assumption",
            _ => unreachable!("dependency class checked above"),
        };
        if dependency.layer != expected_layer {
            return Err(LedgerError::new(format!(
                "{}: dependency layer does not follow its class",
                dependency.id
            )));
        }
        let source_ok = bodies.contains(dependency.from_ref.as_str())
            || roles.contains(dependency.from_ref.as_str())
            || domains.contains(dependency.from_ref.as_str())
            || assumptions.contains(dependency.from_ref.as_str());
        let target_ok = bodies.contains(dependency.to_ref.as_str())
            || roles.contains(dependency.to_ref.as_str())
            || domains.contains(dependency.to_ref.as_str());
        if !source_ok || !target_ok || dependency.from_ref == dependency.to_ref {
            return Err(LedgerError::new(format!(
                "{}: dependency endpoint is invalid",
                dependency.id
            )));
        }
        let external_source = assumptions.contains(dependency.from_ref.as_str());
        if (dependency.dependency_class == "externally-assumed") != external_source {
            return Err(LedgerError::new(format!(
                "{}: external-assumption polarity is inconsistent",
                dependency.id
            )));
        }
        if !triples.insert((
            dependency.from_ref.as_str(),
            dependency.to_ref.as_str(),
            dependency.flow_kind.as_str(),
        )) {
            return Err(LedgerError::new(format!(
                "{}: duplicate dependency grain",
                dependency.id
            )));
        }
        for (field, value) in [
            ("interim_continuity", dependency.interim_continuity.as_str()),
            ("remedy_route", dependency.remedy_route.as_str()),
            ("restoration", dependency.restoration.as_str()),
            (
                "systemic_correction",
                dependency.systemic_correction.as_str(),
            ),
        ] {
            nonempty(value, &format!("{}.{field}", dependency.id))?;
        }
        let expected_satisfiability = match dependency.dependency_class.as_str() {
            "constitutionally-guaranteed" | "democratically-selected" => "specified-interface",
            "operationally-supplied" => "operation-deferred",
            "externally-assumed" => "external-contingent",
            _ => unreachable!(),
        };
        let satisfiability = &dependency.structural_satisfiability;
        nonempty(
            &satisfiability.reason,
            &format!("{}.structural_satisfiability.reason", dependency.id),
        )?;
        unique_strings(
            &satisfiability.defect_refs,
            &format!("{}.structural_satisfiability.defect_refs", dependency.id),
            true,
        )?;
        if !matches!(
            satisfiability.satisfiability_status.as_str(),
            value if value == expected_satisfiability || value == "unsatisfiable"
        ) || (satisfiability.satisfiability_status == "unsatisfiable"
            && satisfiability.defect_refs.is_empty())
            || (satisfiability.satisfiability_status != "unsatisfiable"
                && !satisfiability.defect_refs.is_empty())
            || satisfiability
                .defect_refs
                .iter()
                .any(|reference| !defects.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: structural satisfiability contract is invalid",
                dependency.id
            )));
        }
        unique_strings(
            &dependency.closure_component_refs,
            &format!("{}.closure_component_refs", dependency.id),
            true,
        )?;
        if dependency
            .closure_component_refs
            .iter()
            .any(|value| !valid_closure_component_token(value))
        {
            return Err(LedgerError::new(format!(
                "{}: closure_component_refs must be typed FS-CLR tokens",
                dependency.id
            )));
        }
        match &dependency.alternate_route {
            AlternateRoute::Present(alternate) => {
                nonempty(
                    &alternate.route,
                    &format!("{}.alternate.route", dependency.id),
                )?;
                if alternate
                    .misuse_note
                    .as_deref()
                    .is_some_and(|value| value.trim().is_empty())
                {
                    return Err(LedgerError::new(format!(
                        "{}: alternate misuse_note may not be blank",
                        dependency.id
                    )));
                }
            }
            AlternateRoute::Absent(alternate) => nonempty(
                &alternate.no_alternate_reason,
                &format!("{}.alternate.no_alternate_reason", dependency.id),
            )?,
        }
        unique_strings(
            &dependency.source_refs,
            &format!("{}.source_refs", dependency.id),
            false,
        )?;
        for endpoint in [dependency.from_ref.as_str(), dependency.to_ref.as_str()] {
            if domains.contains(endpoint) {
                touched_domains.insert(endpoint);
            }
        }
        if external_source {
            cited_assumptions.insert(dependency.from_ref.as_str());
        }
        exercised.insert(dependency.flow_kind.as_str());
        edges.insert(dependency.id.as_str(), dependency);
    }
    if touched_domains != domains
        || exercised != FLOW_KINDS.iter().copied().collect::<HashSet<_>>()
        || cited_assumptions != assumptions
    {
        return Err(LedgerError::new(
            "dependency domain, flow-kind, or external-assumption closure failed",
        ));
    }
    if source.dependency_loops.is_empty() || source.refused_flows.is_empty() {
        return Err(LedgerError::new(
            "populated dependencies require declared loops and refused flows",
        ));
    }
    let mut loop_keys = HashSet::new();
    let mut loop_node_sets = Vec::new();
    for loop_row in &source.dependency_loops {
        if !bodies.contains(loop_row.steward_ref.as_str())
            || !LOOP_KINDS.contains(&loop_row.loop_kind.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: loop kind or steward is invalid",
                loop_row.id
            )));
        }
        unique_strings(
            &loop_row.member_edge_refs,
            &format!("{}.member_edge_refs", loop_row.id),
            false,
        )?;
        if loop_row.member_edge_refs.len() < 2
            || loop_row
                .member_edge_refs
                .iter()
                .any(|reference| !edges.contains_key(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: loop needs at least two known edges",
                loop_row.id
            )));
        }
        for index in 0..loop_row.member_edge_refs.len() {
            let current = edges[loop_row.member_edge_refs[index].as_str()];
            let next = edges
                [loop_row.member_edge_refs[(index + 1) % loop_row.member_edge_refs.len()].as_str()];
            if current.to_ref != next.from_ref {
                return Err(LedgerError::new(format!(
                    "{}: loop members do not chain into a cycle",
                    loop_row.id
                )));
            }
        }
        let key = loop_row
            .member_edge_refs
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if !loop_keys.insert(key) {
            return Err(LedgerError::new("duplicate dependency loop"));
        }
        nonempty(
            &loop_row.boundedness,
            &format!("{}.boundedness", loop_row.id),
        )?;
        let mut nodes = HashSet::new();
        for reference in &loop_row.member_edge_refs {
            let edge = edges[reference.as_str()];
            nodes.insert(edge.from_ref.as_str());
            nodes.insert(edge.to_ref.as_str());
        }
        loop_node_sets.push(nodes);
    }
    let mut walls = HashSet::new();
    for flow in &source.refused_flows {
        nonempty(&flow.refused_flow, "refused_flows.refused_flow")?;
        nonempty(&flow.refusal_reason, "refused_flows.refusal_reason")?;
        if !FLOW_KINDS.contains(&flow.flow_kind.as_str())
            || !walls.insert(flow.refused_flow.as_str())
        {
            return Err(LedgerError::new(
                "refused flow kind is invalid or refused_flow is duplicated",
            ));
        }
    }
    let mut adjacency: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut nodes = HashSet::new();
    for edge in edges.values() {
        adjacency
            .entry(edge.from_ref.as_str())
            .or_default()
            .insert(edge.to_ref.as_str());
        nodes.insert(edge.from_ref.as_str());
        nodes.insert(edge.to_ref.as_str());
    }
    let reach = nodes
        .iter()
        .map(|node| (*node, reachable(&adjacency, node)))
        .collect::<HashMap<_, _>>();
    let mut assigned = HashSet::new();
    for node in &nodes {
        if assigned.contains(node) {
            continue;
        }
        let component = nodes
            .iter()
            .copied()
            .filter(|candidate| {
                reach
                    .get(node)
                    .is_some_and(|targets| targets.contains(candidate))
                    && reach
                        .get(candidate)
                        .is_some_and(|targets| targets.contains(node))
            })
            .collect::<HashSet<_>>();
        if component.len() < 2 {
            continue;
        }
        assigned.extend(component.iter().copied());
        if !loop_node_sets
            .iter()
            .any(|witness| witness.is_subset(&component))
        {
            return Err(LedgerError::new(
                "cycle closure: a strongly connected region has no declared loop witness",
            ));
        }
    }
    Ok(())
}

fn validate_scenario_records(
    source: &LedgerDocument,
    siblings: &SiblingProjections,
) -> LedgerResult<()> {
    if source.scenarios.is_empty() {
        if !source.scenario_omissions.is_empty() {
            return Err(LedgerError::new(
                "scenario omissions require a populated scenario catalogue",
            ));
        }
        return Ok(());
    }
    let domains = source
        .domains
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let bodies = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    if !domains.contains("FS-DOM-12") {
        return Err(LedgerError::new(
            "the protected private/civic domain is missing",
        ));
    }
    let witness_pool = siblings.bounded_witnesses();
    let mut cited_domains = HashSet::new();
    let mut cited_dependencies = HashSet::new();
    let mut kinds = HashSet::new();
    let mut axes = HashSet::new();
    let mut shocks = HashSet::new();
    let mut forms = HashSet::new();
    for scenario in &source.scenarios {
        validate_common_record(
            &scenario.id,
            &scenario.title,
            &scenario.applicability,
            &scenario.status,
            &scenario.severity,
            &scenario.consequence,
            &scenario.owner_ref,
            &scenario.closure_condition,
        )?;
        if scenario.status != "reviewed-inventory"
            || scenario.layer != "constitutional-invariant"
            || !SCENARIO_KINDS.contains(&scenario.scenario_kind.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: scenario status, layer, or kind is invalid",
                scenario.id
            )));
        }
        kinds.insert(scenario.scenario_kind.as_str());
        unique_strings(
            &scenario.domain_refs,
            &format!("{}.domain_refs", scenario.id),
            false,
        )?;
        unique_strings(
            &scenario.dependency_refs,
            &format!("{}.dependency_refs", scenario.id),
            true,
        )?;
        if scenario
            .domain_refs
            .iter()
            .any(|reference| !domains.contains(reference.as_str()))
            || scenario
                .dependency_refs
                .iter()
                .any(|reference| !dependencies.contains(reference.as_str()))
            || !bodies.contains(scenario.steward_ref.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: scenario domain, dependency, or steward reference is invalid",
                scenario.id
            )));
        }
        cited_domains.extend(scenario.domain_refs.iter().map(String::as_str));
        cited_dependencies.extend(scenario.dependency_refs.iter().map(String::as_str));
        match scenario.scenario_kind.as_str() {
            "collision" => {
                let axis = scenario.collision_axis.as_deref().ok_or_else(|| {
                    LedgerError::new(format!("{}: collision axis is missing", scenario.id))
                })?;
                if !COLLISION_AXES.contains(&axis) || scenario.shock_kind.is_some() {
                    return Err(LedgerError::new(format!(
                        "{}: collision axis is invalid or shock leaked onto collision",
                        scenario.id
                    )));
                }
                axes.insert(axis);
            }
            "compound-shock" => {
                let shock = scenario.shock_kind.as_deref().ok_or_else(|| {
                    LedgerError::new(format!("{}: shock kind is missing", scenario.id))
                })?;
                if !SHOCK_KINDS.contains(&shock) || scenario.collision_axis.is_some() {
                    return Err(LedgerError::new(format!(
                        "{}: shock kind is invalid or collision leaked onto shock",
                        scenario.id
                    )));
                }
                shocks.insert(shock);
            }
            _ if scenario.collision_axis.is_some() || scenario.shock_kind.is_some() => {
                return Err(LedgerError::new(format!(
                    "{}: axis/shock fields belong only on matching scenario kinds",
                    scenario.id
                )));
            }
            _ => {}
        }
        let reaches_protected = scenario
            .domain_refs
            .iter()
            .any(|value| value == "FS-DOM-12");
        match (&scenario.protected_sphere_forms, reaches_protected) {
            (Some(values), true) => {
                unique_strings(
                    values,
                    &format!("{}.protected_sphere_forms", scenario.id),
                    false,
                )?;
                if values
                    .iter()
                    .any(|value| !PROTECTED_SPHERE_FORMS.contains(&value.as_str()))
                {
                    return Err(LedgerError::new(format!(
                        "{}: protected sphere form is outside the closed enum",
                        scenario.id
                    )));
                }
                forms.extend(values.iter().map(String::as_str));
            }
            (None, true) | (Some(_), false) => {
                return Err(LedgerError::new(format!(
                    "{}: protected-sphere classification polarity is invalid",
                    scenario.id
                )));
            }
            (None, false) => {}
        }
        for (field, value) in [
            ("ordinary_route", scenario.ordinary_route.as_str()),
            ("failure_route", scenario.failure_route.as_str()),
            ("recovery_route", scenario.recovery_route.as_str()),
        ] {
            nonempty(value, &format!("{}.{field}", scenario.id))?;
        }
        if let Some(witnesses) = &scenario.bounded_witness_refs {
            unique_strings(
                witnesses,
                &format!("{}.bounded_witness_refs", scenario.id),
                false,
            )?;
            if witnesses.iter().any(|value| !witness_pool.contains(value)) {
                return Err(LedgerError::new(format!(
                    "{}: bounded witness names no live sibling row",
                    scenario.id
                )));
            }
        }
        unique_strings(
            &scenario.source_refs,
            &format!("{}.source_refs", scenario.id),
            false,
        )?;
        if scenario.source_refs.iter().any(|reference| {
            reference.split_once("::").is_some_and(|(path, _)| {
                path.starts_with("book-1/")
                    || path.starts_with("book-2/")
                    || matches!(path, "book.md" | "manifesto.md")
            })
        }) {
            return Err(LedgerError::new(format!(
                "{}: book prose may not support a scenario row",
                scenario.id
            )));
        }
    }
    if source.scenario_omissions.is_empty() {
        return Err(LedgerError::new(
            "scenario_omissions must be non-empty once scenarios populate",
        ));
    }
    let mut omitted_dependencies = HashSet::new();
    let mut omission_keys = HashSet::new();
    for omission in &source.scenario_omissions {
        nonempty(omission.risk_reason(), "scenario_omissions.risk_reason")?;
        let key = match omission {
            ScenarioOmission::Scenario(value) => {
                nonempty(
                    &value.omitted_scenario,
                    "scenario_omissions.omitted_scenario",
                )?;
                ("scenario", value.omitted_scenario.as_str())
            }
            ScenarioOmission::Dependency(value) => {
                if !dependencies.contains(value.omitted_dependency_ref.as_str())
                    || cited_dependencies.contains(value.omitted_dependency_ref.as_str())
                {
                    return Err(LedgerError::new(
                        "scenario dependency omission is unknown or stale",
                    ));
                }
                omitted_dependencies.insert(value.omitted_dependency_ref.as_str());
                ("dependency", value.omitted_dependency_ref.as_str())
            }
        };
        if !omission_keys.insert(key) {
            return Err(LedgerError::new("duplicate scenario omission"));
        }
    }
    if source.domains.iter().any(|domain| {
        matches!(
            domain.scenario_applicability,
            ScenarioApplicability::Deferred(_)
        )
    }) || cited_domains != domains
        || kinds != SCENARIO_KINDS.iter().copied().collect::<HashSet<_>>()
        || axes != COLLISION_AXES.iter().copied().collect::<HashSet<_>>()
        || shocks != SHOCK_KINDS.iter().copied().collect::<HashSet<_>>()
        || forms
            != PROTECTED_SPHERE_FORMS
                .iter()
                .copied()
                .collect::<HashSet<_>>()
    {
        return Err(LedgerError::new(
            "scenario applicability, domain, kind, axis, shock, or protected-sphere closure failed",
        ));
    }
    let critical = source
        .dependencies
        .iter()
        .filter(|dependency| dependency.severity == "critical")
        .map(|dependency| dependency.id.as_str())
        .collect::<HashSet<_>>();
    if critical
        .difference(&cited_dependencies)
        .any(|reference| !omitted_dependencies.contains(reference))
    {
        return Err(LedgerError::new(
            "critical-dependency closure: an edge is neither stressed nor omitted",
        ));
    }
    Ok(())
}

fn validate_review_contract(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    let protocol = &source.review_protocol;
    if protocol.protocol_status != PROTOCOL_STATUS_CONFIRMED
        || protocol.policy_basis != SCOPE_AUDIT_POLICY_BASIS
        || protocol.mode != "repository-adversarial-audit"
        || protocol.external_review_policy != "optional-non-gating"
    {
        return Err(LedgerError::new(
            "review_protocol status, policy, mode, or external-review policy drifted",
        ));
    }
    let (protocol_path, _) = protocol.protocol_ref.split_once("::").ok_or_else(|| {
        LedgerError::new("review_protocol.protocol_ref must be a repository reference")
    })?;
    if protocol_path != PROTOCOL_DOC
        || protocol.status_line_ref
            != format!("{PROTOCOL_DOC}::Status: {}", protocol.protocol_status)
    {
        return Err(LedgerError::new(
            "review_protocol reference or live status-line binding drifted",
        ));
    }
    let designation = &protocol.designation;
    for (field, value) in [
        ("severity_owner", designation.severity_owner.as_str()),
        (
            "independent_checker",
            designation.independent_checker.as_str(),
        ),
        ("custodian", designation.custodian.as_str()),
        ("basis", designation.basis.as_str()),
    ] {
        nonempty(value, &format!("review_protocol.designation.{field}"))?;
    }
    if !is_iso_date(&designation.designated_date)
        || designation.designation_status != "retired-as-project-gate-dependency"
        || designation.severity_owner == designation.independent_checker
        || designation.custodian == designation.severity_owner
        || designation.custodian == designation.independent_checker
    {
        return Err(LedgerError::new(
            "review_protocol designation date, retirement, or role separation drifted",
        ));
    }
    let review_route = source
        .routes
        .iter()
        .find(|row| row.id == "FS-RTE-07")
        .ok_or_else(|| LedgerError::new("FS-RTE-07 repository-audit route is missing"))?;
    if review_route.status != "built"
        || review_route.route_status != "built"
        || review_route.consequence != R7_CONSEQUENCE
        || review_route.closure_condition != R7_CLOSURE_CONDITION
    {
        return Err(LedgerError::new(
            "FS-RTE-07 repository-audit state must equal the checker-owned state exactly",
        ));
    }
    for (field, value) in [
        (
            "materiality_test",
            source.stopping_rule.materiality_test.as_str(),
        ),
        ("boundary", source.stopping_rule.boundary.as_str()),
        (
            "no_hiding_rule",
            source.stopping_rule.no_hiding_rule.as_str(),
        ),
    ] {
        nonempty(value, &format!("stopping_rule.{field}"))?;
    }
    if !source
        .stopping_rule
        .boundary
        .contains("not a timeless completeness theorem")
    {
        return Err(LedgerError::new(
            "stopping_rule.boundary lost its versioned-exhaustiveness limit",
        ));
    }
    for (field, value) in [
        ("critical", source.severity_rubric.critical.as_str()),
        ("material", source.severity_rubric.material.as_str()),
        ("minor", source.severity_rubric.minor.as_str()),
    ] {
        nonempty(value, &format!("severity_rubric.{field}"))?;
    }
    if source.severity_rubric.materiality_ref != "stopping_rule.materiality_test"
        || match source.severity_rubric.rubric_status.as_str() {
            RUBRIC_STATUS_CANDIDATE => source.severity_rubric.confirmation_basis.is_some(),
            RUBRIC_STATUS_CONFIRMED => source
                .severity_rubric
                .confirmation_basis
                .as_deref()
                .is_none_or(|value| value.trim().is_empty()),
            _ => true,
        }
    {
        return Err(LedgerError::new(
            "severity rubric status, materiality binding, or confirmation basis drifted",
        ));
    }
    let mut deferrals = HashSet::new();
    const DEFERRABLE: [&str; 8] = [
        "roles",
        "powers",
        "dependencies",
        "scenarios",
        "thresholds",
        "defects",
        "receipts",
        "coverage-contracts",
    ];
    for deferral in &source.deferred_populations {
        if !DEFERRABLE.contains(&deferral.record_type.as_str())
            || !deferrals.insert(deferral.record_type.as_str())
        {
            return Err(LedgerError::new(
                "deferred population record_type is unknown or duplicated",
            ));
        }
        nonempty(
            &deferral.closure_condition,
            &format!(
                "deferred_populations.{}.closure_condition",
                deferral.record_type
            ),
        )?;
        nonempty(
            &deferral.stage,
            &format!("deferred_populations.{}.stage", deferral.record_type),
        )?;
    }
    for (record_type, populated) in [
        ("roles", !source.roles.is_empty()),
        ("powers", !source.powers.is_empty()),
        ("dependencies", !source.dependencies.is_empty()),
        ("scenarios", !source.scenarios.is_empty()),
        ("thresholds", !source.thresholds.is_empty()),
        ("defects", !source.defects.is_empty()),
        ("receipts", !source.receipts.is_empty()),
    ] {
        let staged_power_prefix = record_type == "powers"
            && populated
            && deferrals.contains(record_type)
            && source.power_population.status == "partial";
        if (!populated && !deferrals.contains(record_type))
            || (populated && deferrals.contains(record_type) && !staged_power_prefix)
        {
            return Err(LedgerError::new(format!(
                "{record_type}: populated/empty state disagrees with its deferral"
            )));
        }
    }
    let scope_digest = review_scope_digest(source)?;
    let protocol_digest = sha256(input_bytes(inputs, PROTOCOL_DOC)?);
    let mut audit_ids = HashSet::new();
    let mut current = Vec::new();
    for audit in &source.scope_audits {
        if !audit_ids.insert(audit.id.as_str())
            || !is_lower_hex(&audit.scope_sha256, 64)
            || !is_lower_hex(&audit.protocol_sha256, 64)
            || !is_utc_instant(&audit.executed_at_utc)
            || audit.method != SCOPE_AUDIT_METHOD
            || !matches!(
                audit.result.as_str(),
                SCOPE_AUDIT_RESULT | "pending" | "failed"
            )
            || audit.evidence_ceiling.trim().is_empty()
            || audit.author_basis.is_some() == audit.policy_basis.is_some()
        {
            return Err(LedgerError::new(format!(
                "{}: scope audit identity, digest, timestamp, method, result, or basis is invalid",
                audit.id
            )));
        }
        for (field, values) in [
            ("criterion_coverage", &audit.criterion_coverage),
            ("control_refs", &audit.control_refs),
            ("commands", &audit.commands),
            ("finding_refs", &audit.finding_refs),
        ] {
            unique_strings(values, &format!("{}.{}", audit.id, field), true)?;
        }
        if audit.source_version == source.source_version
            && audit.scope_sha256 == scope_digest
            && audit.protocol_sha256 == protocol_digest
        {
            current.push(audit);
        }
    }
    if current.is_empty() && source.coverage_population.status == "complete" {
        return Err(LedgerError::new(
            "scope_audits requires a current-source repository audit",
        ));
    }
    let finding_refs = source
        .defects
        .iter()
        .filter(|defect| {
            defect
                .applicable_gate_refs
                .iter()
                .any(|gate| gate == "gate-a")
        })
        .map(|defect| defect.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_controls = [
        LEDGER_CURRENT_AUDIT_CONTROL_REF,
        closure::CURRENT_AUDIT_CONTROL_REF,
    ];
    for audit in current {
        if audit
            .criterion_coverage
            .iter()
            .map(String::as_str)
            .ne(REVIEW_CRITERIA.iter().copied())
            || audit
                .control_refs
                .iter()
                .map(String::as_str)
                .ne(expected_controls)
            || audit
                .finding_refs
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                != finding_refs
            || audit.evidence_ceiling != SCOPE_AUDIT_EVIDENCE_CEILING
            || audit.author_basis.is_some()
            || audit.policy_basis.as_deref() != Some(SCOPE_AUDIT_POLICY_BASIS)
        {
            return Err(LedgerError::new(format!(
                "{}: current audit coverage, controls, findings, ceiling, or policy drifted",
                audit.id
            )));
        }
        let pending_commands = CURRENT_AUDIT_COMMAND_PREFIX;
        if audit.commands.len() < pending_commands.len()
            || audit.commands[..pending_commands.len()]
                .iter()
                .map(String::as_str)
                .ne(pending_commands)
        {
            return Err(LedgerError::new(format!(
                "{}: current audit command prefix drifted",
                audit.id
            )));
        }
        match audit.result.as_str() {
            SCOPE_AUDIT_RESULT => {
                let receipt_ref = audit.verification_receipt_ref.as_deref().ok_or_else(|| {
                    LedgerError::new(format!("{}: passing audit needs a receipt", audit.id))
                })?;
                if !is_content_addressed_receipt_ref(receipt_ref) {
                    return Err(LedgerError::new(format!(
                        "{}: verification_receipt_ref must be a content-addressed v2 receipt",
                        audit.id
                    )));
                }
                let expected_command =
                    format!("./verify.sh --commit-gate {receipt_ref} --transition audit");
                if audit.commands
                    != [
                        pending_commands[0].to_owned(),
                        pending_commands[1].to_owned(),
                        pending_commands[2].to_owned(),
                        expected_command,
                    ]
                {
                    return Err(LedgerError::new(format!(
                        "{}: passing audit command chain drifted",
                        audit.id
                    )));
                }
                input_bytes(inputs, receipt_ref)?;
            }
            _ if audit.verification_receipt_ref.is_some()
                || audit.commands.len() != pending_commands.len() =>
            {
                return Err(LedgerError::new(format!(
                    "{}: only a passing audit may name a receipt/commit gate",
                    audit.id
                )));
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct ProposalIntakeRow<'a> {
    id: &'a str,
    title: &'a str,
    proposal: &'a str,
    source_kind: &'a str,
    source_identity: &'a str,
    received_at_utc: &'a str,
    review_event_ref: &'a str,
}

#[derive(Serialize)]
struct ProposalIntakeProjection<'a> {
    review_event_ref: &'a str,
    ordered_proposals: Vec<ProposalIntakeRow<'a>>,
}

fn proposal_intake_digest(event_ref: &str, proposals: &[&Proposal]) -> LedgerResult<String> {
    typed_fingerprint(
        &ProposalIntakeProjection {
            review_event_ref: event_ref,
            ordered_proposals: proposals
                .iter()
                .map(|row| ProposalIntakeRow {
                    id: &row.id,
                    title: &row.title,
                    proposal: &row.proposal,
                    source_kind: &row.source_kind,
                    source_identity: &row.source_identity,
                    received_at_utc: &row.received_at_utc,
                    review_event_ref: &row.review_event_ref,
                })
                .collect(),
        },
        "review-event proposal intake",
    )
}

fn is_proposal_classification(value: &str) -> bool {
    matches!(
        value,
        "material-omission" | "retained-limit" | "duplicate" | "immaterial"
    ) || UNESTABLISHED_DISPOSITIONS.contains(&value)
}

fn expected_retained_binding(
    source: &LedgerDocument,
    defect: &Defect,
) -> LedgerResult<RetainedLimitBinding> {
    let claim = source
        .claims
        .iter()
        .find(|row| row.id == defect.affected_claim_ref)
        .ok_or_else(|| LedgerError::new("retained-limit defect names no claim"))?;
    Ok(RetainedLimitBinding {
        severity: severity_class(defect)?.to_owned(),
        consequence: defect.consequence.clone(),
        owner_ref: defect.owner_ref.clone(),
        closure_condition: defect.closure_condition.clone(),
        applicable_gate_refs: defect.applicable_gate_refs.clone(),
        public_claim_restriction: claim.public_claim_restriction.clone(),
    })
}

fn validate_optional_review_records(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    ids: &BTreeSet<String>,
) -> LedgerResult<()> {
    let designation = &source.review_protocol.designation;
    let scope_digest = review_scope_digest(source)?;
    let protocol_sha256 = protocol_digest(inputs)?;
    let barred_reviewers = [
        designation.severity_owner.as_str(),
        designation.independent_checker.as_str(),
        designation.custodian.as_str(),
    ];
    let mut commissions = HashMap::new();
    for commission in &source.review_commissions {
        let context = format!("review_commissions ({})", commission.id);
        nonempty(&commission.title, &format!("{context}.title"))?;
        nonempty(
            &commission.source_version,
            &format!("{context}.source_version"),
        )?;
        for (field, value) in [
            ("scope_sha256", commission.scope_sha256.as_str()),
            ("protocol_sha256", commission.protocol_sha256.as_str()),
            (
                "plant_commitment_sha256",
                commission.plant_commitment_sha256.as_str(),
            ),
            (
                "seed_commitment_sha256",
                commission.seed_commitment_sha256.as_str(),
            ),
        ] {
            if !is_lower_hex(value, 64) {
                return Err(LedgerError::new(format!(
                    "{context}.{field} must be 64 lowercase hex characters"
                )));
            }
        }
        if commission.plant_commitment_sha256 == commission.seed_commitment_sha256 {
            return Err(LedgerError::new(format!(
                "{context}: plant and seed commitments must be distinct"
            )));
        }
        for (field, value) in [
            (
                "commissioned_at_utc",
                commission.commissioned_at_utc.as_str(),
            ),
            (
                "received_window.opens_at_utc",
                commission.received_window.opens_at_utc.as_str(),
            ),
            (
                "received_window.closes_at_utc",
                commission.received_window.closes_at_utc.as_str(),
            ),
            ("cutoff_at_utc", commission.cutoff_at_utc.as_str()),
        ] {
            if !is_utc_instant(value) {
                return Err(LedgerError::new(format!(
                    "{context}.{field} must be canonical UTC"
                )));
            }
        }
        if !(commission.commissioned_at_utc < commission.received_window.opens_at_utc
            && commission.received_window.opens_at_utc < commission.received_window.closes_at_utc
            && commission.received_window.closes_at_utc <= commission.cutoff_at_utc)
        {
            return Err(LedgerError::new(format!(
                "{context}: chronology must be commissioned < open < close <= cutoff"
            )));
        }
        if commission.custodian_identity != designation.custodian {
            return Err(LedgerError::new(format!(
                "{context}: custodian_identity must equal the designated custodian"
            )));
        }
        if commission.reviewers.is_empty() {
            return Err(LedgerError::new(format!(
                "{context}: reviewers must be a non-empty list"
            )));
        }
        let mut identities = HashSet::new();
        let mut disciplines = HashSet::new();
        let mut covered = HashSet::new();
        for reviewer in &commission.reviewers {
            let reviewer_context = format!("{context}.reviewers ({})", reviewer.identity);
            nonempty(&reviewer.identity, &format!("{reviewer_context}.identity"))?;
            nonempty(
                &reviewer.discipline,
                &format!("{reviewer_context}.discipline"),
            )?;
            if barred_reviewers.contains(&reviewer.identity.as_str()) {
                return Err(LedgerError::new(format!(
                    "{reviewer_context}: reviewer conflicts with custodian, Darshu, or Dhanush"
                )));
            }
            if !identities.insert(reviewer.identity.as_str()) {
                return Err(LedgerError::new(format!(
                    "{context}: reviewer identities must be unique"
                )));
            }
            disciplines.insert(reviewer.discipline.as_str());
            unique_strings(
                &reviewer.criterion_refs,
                &format!("{reviewer_context}.criterion_refs"),
                false,
            )?;
            let canonical = REVIEW_CRITERIA
                .iter()
                .filter(|criterion| reviewer.criterion_refs.iter().any(|row| row == **criterion))
                .copied();
            if reviewer
                .criterion_refs
                .iter()
                .map(String::as_str)
                .ne(canonical)
            {
                return Err(LedgerError::new(format!(
                    "{reviewer_context}.criterion_refs must be unique and in canonical order"
                )));
            }
            covered.extend(reviewer.criterion_refs.iter().map(String::as_str));
            if reviewer.consent_attestation != REVIEWER_CONSENT {
                return Err(LedgerError::new(format!(
                    "{reviewer_context}: reviewer consent attestation is not exact"
                )));
            }
            if reviewer.conflict_attestation != REVIEWER_CONFLICT_CLEAR {
                return Err(LedgerError::new(format!(
                    "{reviewer_context}: reviewer conflict attestation is not exact"
                )));
            }
            if reviewer.compensation_attestation != REVIEWER_COMPENSATION_CLEAR {
                return Err(LedgerError::new(format!(
                    "{reviewer_context}: findings-contingent compensation is refused"
                )));
            }
        }
        if disciplines.len() < 2 {
            return Err(LedgerError::new(format!(
                "{context}: the panel must contain at least two disciplines"
            )));
        }
        if commission
            .criterion_coverage
            .iter()
            .map(String::as_str)
            .ne(REVIEW_CRITERIA)
            || covered != REVIEW_CRITERIA.iter().copied().collect::<HashSet<_>>()
        {
            return Err(LedgerError::new(format!(
                "{context}: reviewer criterion union and coverage must cover every criterion"
            )));
        }
        if commission
            .packet_paths
            .iter()
            .map(String::as_str)
            .ne(REVIEW_PACKET_PATHS)
        {
            return Err(LedgerError::new(format!(
                "{context}.packet_paths must be the exact ordered packet manifest"
            )));
        }
        if commission.source_version == source.source_version
            && (commission.scope_sha256 != scope_digest
                || commission.protocol_sha256 != protocol_sha256)
        {
            return Err(LedgerError::new(format!(
                "{context}: current-source commission scope or protocol digest is stale"
            )));
        }
        commissions.insert(commission.id.as_str(), commission);
    }

    let mut events = HashMap::new();
    for event in &source.review_events {
        let context = format!("review_events ({})", event.id);
        nonempty(&event.title, &format!("{context}.title"))?;
        let commission = commissions
            .get(event.commission_ref.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{context}: commission_ref names no commission"))
            })?;
        if !is_lower_hex(&event.packet_commit_sha, 40) {
            return Err(LedgerError::new(format!(
                "{context}.packet_commit_sha must be a 40-character lowercase Git id"
            )));
        }
        if event.source_version != commission.source_version
            || event.scope_sha256 != commission.scope_sha256
            || event.protocol_sha256 != commission.protocol_sha256
        {
            return Err(LedgerError::new(format!(
                "{context}: source version and digests must equal its commission"
            )));
        }
        if !is_utc_instant(&event.intake_receipt.frozen_at_utc)
            || !is_lower_hex(&event.intake_receipt.manifest_sha256, 64)
            || !is_utc_instant(&event.control_reveal.revealed_at_utc)
            || !is_lower_hex(&event.control_reveal.plant_preimage_sha256, 64)
            || !is_lower_hex(&event.control_reveal.seed_preimage_sha256, 64)
        {
            return Err(LedgerError::new(format!(
                "{context}: intake or reveal timestamp/digest is malformed"
            )));
        }
        unique_strings(
            &event.intake_receipt.ordered_proposal_ids,
            &format!("{context}.intake_receipt.ordered_proposal_ids"),
            true,
        )?;
        if let Some(reference) = &event.control_reveal.planted_proposal_ref.0 {
            nonempty(
                reference,
                &format!("{context}.control_reveal.planted_proposal_ref"),
            )?;
        }
        nonempty(
            &event.control_reveal.plant_match_checked_by,
            &format!("{context}.control_reveal.plant_match_checked_by"),
        )?;
        nonempty(
            &event.control_reveal.plant_match_reason,
            &format!("{context}.control_reveal.plant_match_reason"),
        )?;
        for (field, control) in [
            ("seeded_control", &event.seeded_control),
            ("planted_control", &event.planted_control),
        ] {
            if !matches!(control.status.as_str(), "passed" | "failed") {
                return Err(LedgerError::new(format!(
                    "{context}.{field}.status must be passed or failed"
                )));
            }
            nonempty(&control.reason, &format!("{context}.{field}.reason"))?;
        }
        if !matches!(event.outcome_status.as_str(), "passed" | "failed") {
            return Err(LedgerError::new(format!(
                "{context}.outcome_status must be passed or failed"
            )));
        }
        nonempty(&event.outcome_reason, &format!("{context}.outcome_reason"))?;
        events.insert(event.id.as_str(), (event, *commission));
    }

    let defects = source
        .defects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut proposals = HashMap::new();
    for proposal in &source.proposals {
        let context = format!("proposals ({})", proposal.id);
        for (field, value) in [
            ("title", proposal.title.as_str()),
            ("proposal", proposal.proposal.as_str()),
            ("source_identity", proposal.source_identity.as_str()),
            ("materiality_reason", proposal.materiality_reason.as_str()),
            ("check_reason", proposal.check_reason.as_str()),
            ("reasons", proposal.reasons.as_str()),
        ] {
            nonempty(value, &format!("{context}.{field}"))?;
        }
        let (_, commission) = events
            .get(proposal.review_event_ref.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{context}: review_event_ref names no review event"))
            })?;
        for (field, value) in [
            ("received_at_utc", proposal.received_at_utc.as_str()),
            ("triaged_at_utc", proposal.triaged_at_utc.as_str()),
            ("checked_at_utc", proposal.checked_at_utc.as_str()),
            ("disposition_at_utc", proposal.disposition_at_utc.as_str()),
        ] {
            if !is_utc_instant(value) {
                return Err(LedgerError::new(format!(
                    "{context}.{field} must be canonical UTC"
                )));
            }
        }
        if proposal.received_at_utc < commission.received_window.opens_at_utc
            || proposal.received_at_utc > commission.received_window.closes_at_utc
        {
            return Err(LedgerError::new(format!(
                "{context}: proposal received outside its window"
            )));
        }
        if !(proposal.received_at_utc <= proposal.triaged_at_utc
            && proposal.triaged_at_utc <= proposal.checked_at_utc
            && proposal.checked_at_utc <= proposal.disposition_at_utc)
        {
            return Err(LedgerError::new(format!(
                "{context}: chronology must be received <= triaged <= checked <= disposed"
            )));
        }
        if proposal.severity_owner_identity != designation.severity_owner {
            return Err(LedgerError::new(format!(
                "{context}: every proposal requires Darshu triage"
            )));
        }
        if proposal.independent_checker_identity != designation.independent_checker {
            return Err(LedgerError::new(format!(
                "{context}: every proposal requires Dhanush checking"
            )));
        }
        if !matches!(proposal.check_finding.as_str(), "confirmed" | "corrected") {
            return Err(LedgerError::new(format!(
                "{context}.check_finding must be confirmed or corrected"
            )));
        }
        if !matches!(proposal.source_kind.as_str(), "reviewer" | "seed") {
            return Err(LedgerError::new(format!(
                "{context}.source_kind must be reviewer or seed"
            )));
        }
        let reviewer_ids = commission
            .reviewers
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<HashSet<_>>();
        if (proposal.source_kind == "reviewer"
            && !reviewer_ids.contains(proposal.source_identity.as_str()))
            || (proposal.source_kind == "seed"
                && proposal.source_identity != "committed-seed-control")
        {
            return Err(LedgerError::new(format!(
                "{context}: proposal source identity is not admitted"
            )));
        }
        if !matches!(
            proposal.control_kind.as_str(),
            "none" | "seed" | "plant-match"
        ) || (proposal.control_kind == "seed" && proposal.source_kind != "seed")
            || (proposal.control_kind == "plant-match" && proposal.source_kind != "reviewer")
        {
            return Err(LedgerError::new(format!(
                "{context}: control kind and source kind disagree"
            )));
        }
        if !matches!(
            proposal.materiality_finding.as_str(),
            "material" | "immaterial"
        ) || (proposal.materiality_finding == "material"
            && !matches!(proposal.severity.as_deref(), Some("critical" | "material")))
            || (proposal.materiality_finding == "immaterial" && proposal.severity.is_some())
        {
            return Err(LedgerError::new(format!(
                "{context}: materiality and severity contract drifted"
            )));
        }
        if !is_proposal_classification(&proposal.classification)
            || !PROPOSAL_DISPOSITIONS.contains(&proposal.proposal_disposition.as_str())
        {
            return Err(LedgerError::new(format!(
                "{context}: unknown classification or proposal disposition"
            )));
        }
        match proposal.proposal_disposition.as_str() {
            "added" => {
                if proposal.materiality_finding != "material"
                    || proposal.classification != "material-omission"
                {
                    return Err(LedgerError::new(format!(
                        "{context}: added requires material-omission classification"
                    )));
                }
                let refs = proposal
                    .created_record_refs
                    .as_ref()
                    .filter(|refs| !refs.is_empty())
                    .ok_or_else(|| {
                        LedgerError::new(format!("{context}: added must name created records"))
                    })?;
                for reference in refs {
                    if !ids.contains(reference) {
                        validate_repository_reference(
                            inputs,
                            reference,
                            &format!("{context}.created_record_refs"),
                        )?;
                    }
                }
                if proposal.routed_unestablished_disposition.is_some()
                    || proposal.defect_row_ref.is_some()
                    || proposal.retained_limit_binding.is_some()
                {
                    return Err(LedgerError::new(format!(
                        "{context}: added carries disposition-only fields"
                    )));
                }
            }
            "classified-out" => {
                let classified = proposal.classification.as_str();
                if !(matches!(classified, "duplicate" | "immaterial")
                    || UNESTABLISHED_DISPOSITIONS.contains(&classified))
                    || (UNESTABLISHED_DISPOSITIONS.contains(&classified)
                        && proposal.routed_unestablished_disposition.as_deref() != Some(classified))
                    || (matches!(classified, "duplicate" | "immaterial")
                        && proposal.routed_unestablished_disposition.is_some())
                    || proposal.created_record_refs.is_some()
                    || proposal.defect_row_ref.is_some()
                    || proposal.retained_limit_binding.is_some()
                {
                    return Err(LedgerError::new(format!(
                        "{context}: classification-to-Unestablished mapping must be exact"
                    )));
                }
            }
            "retained-limit" => {
                if proposal.materiality_finding != "material"
                    || proposal.classification != "retained-limit"
                    || proposal.created_record_refs.is_some()
                    || proposal.routed_unestablished_disposition.is_some()
                {
                    return Err(LedgerError::new(format!(
                        "{context}: retained-limit disposition requires matching classification"
                    )));
                }
                let defect = proposal
                    .defect_row_ref
                    .as_deref()
                    .and_then(|reference| defects.get(reference).copied())
                    .ok_or_else(|| {
                        LedgerError::new(format!(
                            "{context}: retained limit must link a defect row"
                        ))
                    })?;
                let expected = expected_retained_binding(source, defect)?;
                if proposal.retained_limit_binding.as_ref() != Some(&expected)
                    || proposal.severity.as_deref() != Some(expected.severity.as_str())
                {
                    return Err(LedgerError::new(format!(
                        "{context}: retained-limit binding must match the defect and claim"
                    )));
                }
            }
            _ => unreachable!("proposal disposition was validated"),
        }
        proposals.insert(proposal.id.as_str(), proposal);
    }

    for (event, commission) in events.values() {
        let context = format!("review_events ({})", event.id);
        let ordered_ids = &event.intake_receipt.ordered_proposal_ids;
        let actual = source
            .proposals
            .iter()
            .filter(|row| row.review_event_ref == event.id)
            .collect::<Vec<_>>();
        if ordered_ids
            .iter()
            .map(String::as_str)
            .ne(actual.iter().map(|row| row.id.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{context}: intake ordered proposal ids must equal the event proposal set"
            )));
        }
        let ordered = ordered_ids
            .iter()
            .map(|id| {
                proposals.get(id.as_str()).copied().ok_or_else(|| {
                    LedgerError::new(format!("{context}: intake proposal id is unknown"))
                })
            })
            .collect::<LedgerResult<Vec<_>>>()?;
        if event.intake_receipt.manifest_sha256 != proposal_intake_digest(&event.id, &ordered)? {
            return Err(LedgerError::new(format!(
                "{context}: intake manifest digest does not match"
            )));
        }
        let frozen = &event.intake_receipt.frozen_at_utc;
        let reveal = &event.control_reveal;
        if commission.received_window.closes_at_utc > *frozen
            || *frozen > reveal.revealed_at_utc
            || reveal.revealed_at_utc < commission.cutoff_at_utc
        {
            return Err(LedgerError::new(format!(
                "{context}: controls may not reveal before cutoff and intake must freeze in order"
            )));
        }
        if ordered.iter().any(|row| {
            row.triaged_at_utc > reveal.revealed_at_utc
                || row.checked_at_utc > reveal.revealed_at_utc
        }) {
            return Err(LedgerError::new(format!(
                "{context}: triage and Dhanush checking must finish before reveal"
            )));
        }
        let seed_ids = ordered
            .iter()
            .filter(|row| row.control_kind == "seed")
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();
        let result_ids = reveal
            .seed_results
            .iter()
            .map(|row| row.proposal_ref.as_str())
            .collect::<HashSet<_>>();
        if result_ids.len() != reveal.seed_results.len() || result_ids != seed_ids {
            return Err(LedgerError::new(format!(
                "{context}: reveal must adjudicate every and only seeded proposals"
            )));
        }
        let expected_sides = reveal
            .seed_results
            .iter()
            .map(|row| row.expected_materiality.as_str())
            .collect::<HashSet<_>>();
        if expected_sides != HashSet::from(["material", "immaterial"]) {
            return Err(LedgerError::new(format!(
                "{context}: seeds must cover both material and immaterial cases"
            )));
        }
        let mut seed_ok = reveal.seed_preimage_sha256 == commission.seed_commitment_sha256;
        for result in &reveal.seed_results {
            let result_context = format!(
                "{context}.control_reveal.seed_results ({})",
                result.proposal_ref
            );
            let proposal = proposals
                .get(result.proposal_ref.as_str())
                .copied()
                .ok_or_else(|| LedgerError::new(format!("{result_context}: proposal missing")))?;
            if !matches!(
                result.expected_materiality.as_str(),
                "material" | "immaterial"
            ) || (result.expected_materiality == "material"
                && !matches!(
                    result.expected_severity.0.as_deref(),
                    Some("critical" | "material")
                ))
                || (result.expected_materiality == "immaterial"
                    && result.expected_severity.0.is_some())
                || !PROPOSAL_DISPOSITIONS.contains(&result.expected_disposition.as_str())
                || result.verified_by != designation.independent_checker
            {
                return Err(LedgerError::new(format!(
                    "{result_context}: seed expectation contract is invalid"
                )));
            }
            nonempty(
                &result.verification_reason,
                &format!("{result_context}.verification_reason"),
            )?;
            if proposal.materiality_finding != result.expected_materiality
                || proposal.severity != result.expected_severity.0
                || proposal.proposal_disposition != result.expected_disposition
            {
                seed_ok = false;
            }
        }
        let plant_ok = reveal.plant_preimage_sha256 == commission.plant_commitment_sha256
            && reveal
                .planted_proposal_ref
                .0
                .as_deref()
                .and_then(|reference| proposals.get(reference).copied())
                .is_some_and(|row| {
                    row.control_kind == "plant-match" && row.source_kind == "reviewer"
                })
            && reveal.plant_match_checked_by == designation.independent_checker;
        let expected_seed = if seed_ok { "passed" } else { "failed" };
        let expected_plant = if plant_ok { "passed" } else { "failed" };
        let expected_outcome = if seed_ok && plant_ok {
            "passed"
        } else {
            "failed"
        };
        if event.seeded_control.status != expected_seed
            || event.planted_control.status != expected_plant
            || event.outcome_status != expected_outcome
        {
            return Err(LedgerError::new(format!(
                "{context}: outcome_status must be derived from both controls"
            )));
        }
    }
    Ok(())
}

fn validate_current_audit_receipts(
    context: &Context,
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
) -> LedgerResult<()> {
    let scope_digest = review_scope_digest(source)?;
    let protocol_digest = sha256(input_bytes(inputs, PROTOCOL_DOC)?);
    for audit in source.scope_audits.iter().filter(|audit| {
        audit.source_version == source.source_version
            && audit.scope_sha256 == scope_digest
            && audit.protocol_sha256 == protocol_digest
            && audit.result == SCOPE_AUDIT_RESULT
    }) {
        let receipt_ref = audit.verification_receipt_ref.as_deref().ok_or_else(|| {
            LedgerError::new(format!("{}: passing audit needs a receipt", audit.id))
        })?;
        let validated_receipt = receipt::validate_receipt_bytes(
            context,
            &context.path(receipt_ref),
            input_bytes(inputs, receipt_ref)?,
            ValidationOptions {
                require_local: false,
                check_environment: false,
                check_engine: false,
                source_version: Some(&audit.source_version),
                audit_id: Some(&audit.id),
            },
        )
        .map_err(|error| {
            LedgerError::new(format!(
                "{}: verification receipt invalid: {error}",
                audit.id
            ))
        })?;
        if source
            .closure_record
            .0
            .as_ref()
            .is_some_and(|closure| closure.scope_audit_ref == audit.id)
        {
            let compact = validated_receipt.v2().map_err(|error| {
                LedgerError::new(format!(
                    "closure_record.verification_receipt_ref: receipt is not v2: {error}"
                ))
            })?;
            receipt::validate_recorded_transition(
                context,
                compact,
                &source
                    .closure_record
                    .0
                    .as_ref()
                    .expect("closure audit match requires a closure record")
                    .candidate_commit_sha,
                receipt::Transition::Audit,
                receipt_ref,
            )
            .map_err(|error| {
                LedgerError::new(format!(
                    "closure_record.verification_receipt_ref: recorded audit transition is invalid: {error}"
                ))
            })?;
        }
    }
    Ok(())
}

fn validate_records(source: &LedgerDocument, ids: &BTreeSet<String>) -> LedgerResult<()> {
    validate_role_records(source)?;
    validate_body_records(source)?;
    validate_envelope_criteria_thresholds(source)?;
    validate_dependency_records(source)?;
    let domains = source
        .domains
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let claims = source
        .claims
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let bodies = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let roles = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let routes = source
        .routes
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let legacy_rows = source
        .legacy_rows
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let external_assumptions = source
        .external_assumptions
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let envelope_ids = source
        .envelope
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let powers = source
        .powers
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let loops = source
        .dependency_loops
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();

    if source.routes.len() != 7 {
        return Err(LedgerError::new(
            "routes must carry exactly the seven ratified routes",
        ));
    }
    for route in &source.routes {
        validate_common_record(
            &route.id,
            &route.title,
            &route.applicability,
            &route.status,
            &route.severity,
            &route.consequence,
            &route.owner_ref,
            &route.closure_condition,
        )?;
        if !ROUTE_STATUSES.contains(&route.route_status.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: unknown route_status",
                route.id
            )));
        }
        nonempty(&route.warrants, &format!("{}.warrants", route.id))?;
        nonempty(
            &route.cannot_warrant,
            &format!("{}.cannot_warrant", route.id),
        )?;
        if matches!(route.route_status.as_str(), "built" | "available") {
            for (field, value) in [
                (
                    "falsification_condition",
                    route.falsification_condition.as_str(),
                ),
                ("negative_control", route.negative_control.as_str()),
            ] {
                if value.trim().is_empty()
                    || value.trim().to_ascii_lowercase().starts_with("not-yet")
                {
                    return Err(LedgerError::new(format!(
                        "{}: a {} route must declare its {field}",
                        route.id, route.route_status
                    )));
                }
            }
        }
    }
    for domain in &source.domains {
        validate_common_record(
            &domain.id,
            &domain.title,
            &domain.applicability,
            &domain.status,
            &domain.severity,
            &domain.consequence,
            &domain.owner_ref,
            &domain.closure_condition,
        )?;
        if domain.layer != "spans-all-layers" {
            return Err(LedgerError::new(format!(
                "{}: a domain must span all layers",
                domain.id
            )));
        }
        unique_strings(
            &domain.class_refs,
            &format!("{}.class_refs", domain.id),
            false,
        )?;
        if domain.class_refs.iter().any(|value| {
            value
                .strip_prefix("class-")
                .and_then(|number| number.parse::<u8>().ok())
                .is_none_or(|number| !(1..=10).contains(&number) || value.len() != 8)
        }) {
            return Err(LedgerError::new(format!(
                "{}: class_refs must name taxonomy classes class-01..class-10",
                domain.id
            )));
        }
        for (field, values, known) in [
            ("bodies_refs", &domain.bodies_refs, &bodies),
            (
                "external_assumption_refs",
                &domain.external_assumption_refs,
                &external_assumptions,
            ),
            ("legacy_row_refs", &domain.legacy_row_refs, &legacy_rows),
        ] {
            unique_strings(values, &format!("{}.{field}", domain.id), true)?;
            if values.iter().any(|value| !known.contains(value.as_str())) {
                return Err(LedgerError::new(format!(
                    "{}: {field} names an unknown id",
                    domain.id
                )));
            }
        }
        for (field, bucket) in [
            (
                "constitutional_invariants",
                &domain.constitutional_invariants,
            ),
            ("ordinary_law_choices", &domain.ordinary_law_choices),
            ("protected_private_civic", &domain.protected_private_civic),
            ("book2_operations", &domain.book2_operations),
            (
                "external_assumptions_note",
                &domain.external_assumptions_note,
            ),
        ] {
            validate_domain_bucket(bucket, &format!("{}.{field}", domain.id))?;
        }
        match &domain.scenario_applicability {
            ScenarioApplicability::Answer(value) => {
                nonempty(
                    &value.answer,
                    &format!("{}.scenario_applicability.answer", domain.id),
                )?;
            }
            ScenarioApplicability::Deferred(value) => {
                nonempty(
                    &value.deferred_ref,
                    &format!("{}.scenario_applicability.deferred_ref", domain.id),
                )?;
            }
        }
        nonempty(
            &domain.reader_destination,
            &format!("{}.reader_destination", domain.id),
        )?;
        unique_strings(
            &domain.source_refs,
            &format!("{}.source_refs", domain.id),
            false,
        )?;
    }
    if source.domains.len() < 12 {
        return Err(LedgerError::new(
            "domains must cover at least the twelve-cluster minimum",
        ));
    }
    for row in &source.legacy_rows {
        for (field, value) in [
            ("domain_title", row.domain_title.as_str()),
            ("legacy_coverage", row.legacy_coverage.as_str()),
            (
                "legacy_scope_requirement",
                row.legacy_scope_requirement.as_str(),
            ),
            ("legacy_status_cell", row.legacy_status_cell.as_str()),
            ("legacy_gap", row.legacy_gap.as_str()),
            ("legacy_status", row.legacy_status.as_str()),
            ("source_ref", row.source_ref.as_str()),
        ] {
            nonempty(value, &format!("{}.{field}", row.id))?;
        }
        for (field, value) in [
            ("legacy_coverage", row.legacy_coverage.as_str()),
            (
                "legacy_scope_requirement",
                row.legacy_scope_requirement.as_str(),
            ),
            ("legacy_status_cell", row.legacy_status_cell.as_str()),
            ("legacy_gap", row.legacy_gap.as_str()),
        ] {
            if value.contains('|') || value.contains('\n') {
                return Err(LedgerError::new(format!(
                    "{}: {field} may not contain a table pipe or newline",
                    row.id
                )));
            }
        }
        unique_strings(&row.domain_refs, &format!("{}.domain_refs", row.id), false)?;
        if row
            .domain_refs
            .iter()
            .any(|reference| !domains.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: domain_refs names an unknown domain",
                row.id
            )));
        }
        match row.split_state.as_str() {
            "split" => {
                unique_strings(
                    &row.split_claim_refs,
                    &format!("{}.split_claim_refs", row.id),
                    false,
                )?;
                if row.unresolved.is_some()
                    || row
                        .split_claim_refs
                        .iter()
                        .any(|reference| !claims.contains(reference.as_str()))
                {
                    return Err(LedgerError::new(format!(
                        "{}: split row has invalid claims or unresolved state",
                        row.id
                    )));
                }
            }
            "split-deferred" => {
                if !row.split_claim_refs.is_empty() || row.unresolved.is_none() {
                    return Err(LedgerError::new(format!(
                        "{}: split-deferred needs unresolved and no claims",
                        row.id
                    )));
                }
                validate_unresolved_detail(
                    row.unresolved.as_ref().expect("checked above"),
                    &format!("{}.unresolved", row.id),
                )?;
            }
            _ => {
                return Err(LedgerError::new(format!(
                    "{}: split_state must be split or split-deferred",
                    row.id
                )));
            }
        }
    }
    let routes_by_id = source
        .routes
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    for claim in &source.claims {
        validate_common_record(
            &claim.id,
            &claim.title,
            &claim.applicability,
            &claim.status,
            &claim.severity,
            &claim.consequence,
            &claim.owner_ref,
            &claim.closure_condition,
        )?;
        nonempty(&claim.claim, &format!("{}.claim", claim.id))?;
        nonempty(&claim.scope_bound, &format!("{}.scope_bound", claim.id))?;
        nonempty(
            &claim.public_claim_restriction,
            &format!("{}.public_claim_restriction", claim.id),
        )?;
        if !SCOPE_DISPOSITIONS.contains(&claim.layer.as_str())
            || !POSTURES.contains(&claim.posture.as_str())
            || !OVERLAYS.contains(&claim.overlay.as_str())
        {
            return Err(LedgerError::new(format!(
                "{}: claim layer, posture, or overlay is outside its closed enum",
                claim.id
            )));
        }
        unique_strings(
            &claim.domain_refs,
            &format!("{}.domain_refs", claim.id),
            false,
        )?;
        if claim
            .domain_refs
            .iter()
            .any(|item| !domains.contains(item.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: claim names an unknown domain",
                claim.id
            )));
        }
        if claim
            .legacy_row_ref
            .0
            .as_deref()
            .is_some_and(|reference| !legacy_rows.contains(reference))
        {
            return Err(LedgerError::new(format!(
                "{}: claim names an unknown legacy row",
                claim.id
            )));
        }
        unique_strings(
            &claim.closure_requirement_refs,
            &format!("{}.closure_requirement_refs", claim.id),
            true,
        )?;
        if claim
            .closure_requirement_refs
            .iter()
            .any(|reference| !reference.starts_with("FS-CLR-"))
        {
            return Err(LedgerError::new(format!(
                "{}: closure_requirement_refs must be FS-CLR ids",
                claim.id
            )));
        }
        if claim.envelope_id != ENVELOPE_STUB_ID {
            return Err(LedgerError::new(format!(
                "{}: Book 1 claims must name the pre-envelope identity {ENVELOPE_STUB_ID}",
                claim.id
            )));
        }
        if claim
            .evidence_notes
            .iter()
            .any(|note| note.trim().is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: evidence_notes must contain non-empty strings",
                claim.id
            )));
        }
        let route = routes_by_id.get(claim.route_ref.as_str()).ok_or_else(|| {
            LedgerError::new(format!("{}: claim names an unknown route", claim.id))
        })?;
        if claim.posture == "Unestablished" {
            let disposition = claim.unestablished_disposition.as_deref().ok_or_else(|| {
                LedgerError::new(format!(
                    "{}: Unestablished requires an explicit disposition",
                    claim.id
                ))
            })?;
            if !UNESTABLISHED_DISPOSITIONS.contains(&disposition) {
                return Err(LedgerError::new(format!(
                    "{}: unknown Unestablished disposition",
                    claim.id
                )));
            }
            match disposition {
                "route-unbuilt" => {
                    let restriction = claim.public_claim_restriction.to_ascii_lowercase();
                    if route.route_status != "unbuilt"
                        || (!restriction.contains("restricted")
                            && !restriction.contains("no public"))
                    {
                        return Err(LedgerError::new(format!(
                            "{}: route-unbuilt requires an unbuilt route and explicit public restriction",
                            claim.id
                        )));
                    }
                }
                "evidence-pending"
                    if !matches!(route.route_status.as_str(), "built" | "available") =>
                {
                    return Err(LedgerError::new(format!(
                        "{}: evidence-pending requires a built or available route",
                        claim.id
                    )));
                }
                _ => {}
            }
        } else if claim.unestablished_disposition.is_some() {
            return Err(LedgerError::new(format!(
                "{}: unestablished_disposition belongs only on Unestablished rows",
                claim.id
            )));
        }
        if ["Derived", "Checked", "Evidenced"].contains(&claim.posture.as_str())
            && route.route_status == "unbuilt"
        {
            return Err(LedgerError::new(format!(
                "{}: established posture needs a built or available route",
                claim.id
            )));
        }
        match claim.posture.as_str() {
            "Derived" => {
                if claim.evidence_kind.as_deref() != Some("executable") {
                    return Err(LedgerError::new(format!(
                        "{}: Derived requires executable evidence kind",
                        claim.id
                    )));
                }
                let notes = claim.evidence_notes.join(" ").to_ascii_lowercase();
                if notes.contains("mutation") && claim.mutation_ref.is_none() {
                    return Err(LedgerError::new(format!(
                        "{}: Derived over a mutation needs a mutation_ref",
                        claim.id
                    )));
                }
            }
            "Checked" => {
                if !matches!(
                    claim.evidence_kind.as_deref(),
                    Some("pattern-guard" | "freshness" | "inventory")
                ) {
                    return Err(LedgerError::new(format!(
                        "{}: Checked requires pattern-guard, freshness, or inventory",
                        claim.id
                    )));
                }
                if claim.claim.to_ascii_lowercase().contains("impossib") {
                    return Err(LedgerError::new(format!(
                        "{}: Checked may not be phrased as an impossibility",
                        claim.id
                    )));
                }
            }
            _ if claim.evidence_kind.is_some() => {
                return Err(LedgerError::new(format!(
                    "{}: evidence_kind belongs only on Derived or Checked rows",
                    claim.id
                )));
            }
            _ => {}
        }
        if claim.posture == "Specified" && claim.unimplemented_marker != Some(true) {
            return Err(LedgerError::new(format!(
                "{}: Specified needs its explicit unimplemented marker",
                claim.id
            )));
        }
        if claim.overlay == "liveness" && claim.posture != "Unestablished" {
            return Err(LedgerError::new(format!(
                "{}: liveness remains Unestablished until operational assurance exists",
                claim.id
            )));
        }
        if claim.overlay == "feasibility" {
            return Err(LedgerError::new(format!(
                "{}: feasibility claims belong outside Book 1",
                claim.id
            )));
        }
        if claim.mutation_ref.is_some() && claim.posture != "Derived" {
            return Err(LedgerError::new(format!(
                "{}: mutation_ref belongs only on Derived rows",
                claim.id
            )));
        }
    }
    for assumption in &source.external_assumptions {
        validate_common_record(
            &assumption.id,
            &assumption.title,
            &assumption.applicability,
            &assumption.status,
            &assumption.severity,
            &assumption.consequence,
            &assumption.owner_ref,
            &assumption.closure_condition,
        )?;
        if assumption.layer != "external-assumption" {
            return Err(LedgerError::new(format!(
                "{}: layer must be external-assumption",
                assumption.id
            )));
        }
        nonempty(
            &assumption.assumption,
            &format!("{}.assumption", assumption.id),
        )?;
        nonempty(
            &assumption.failure_consequence,
            &format!("{}.failure_consequence", assumption.id),
        )?;
    }
    if source.envelope.is_empty() || source.envelope[0].id != ENVELOPE_STUB_ID {
        return Err(LedgerError::new(format!(
            "the envelope array must begin with {ENVELOPE_STUB_ID}"
        )));
    }
    if source
        .envelope
        .iter()
        .any(|envelope| envelope.envelope_status == "calibrated")
    {
        return Err(LedgerError::new(
            "calibrated envelopes remain outside the current Book 1 contract",
        ));
    }
    if !envelope_ids.contains(ENVELOPE_STUB_ID) {
        return Err(LedgerError::new("the permanent envelope stub is missing"));
    }
    let mut manifest_keys = HashSet::new();
    let mut power_term_texts = HashSet::new();
    for power in &source.powers {
        if !manifest_keys.insert(power.manifest_key.as_str()) {
            return Err(LedgerError::new(
                "a power grain cannot be bundled or duplicated",
            ));
        }
        unique_strings(&power.profiles, &format!("{}.profiles", power.id), false)?;
        if power
            .profiles
            .iter()
            .any(|profile| !power.profile_terms.contains_key(profile))
            || power
                .profile_terms
                .keys()
                .any(|profile| !power.profiles.contains(profile))
        {
            return Err(LedgerError::new(format!(
                "{}: power profile declarations and terms differ",
                power.id
            )));
        }
        validate_term_set(
            &power.contract_terms,
            &format!("{}.contract_terms", power.id),
        )?;
        validate_profile_terms(&power.profile_terms, &format!("{}.profile_terms", power.id))?;
        validate_term(
            &power.evidence_authority,
            &format!("{}.evidence_authority", power.id),
        )?;
        validate_test_binding(&power.negative_test, &format!("{}.negative_test", power.id))?;
        validate_test_binding(
            &power.counterfactual,
            &format!("{}.counterfactual", power.id),
        )?;
        for (context, term) in power
            .contract_terms
            .iter()
            .map(|(name, term)| (format!("{}.contract_terms.{name}", power.id), term))
            .chain(power.profile_terms.iter().flat_map(|(profile, terms)| {
                terms.iter().map(move |(name, term)| {
                    (format!("{}.profile_terms.{profile}.{name}", power.id), term)
                })
            }))
            .chain(std::iter::once((
                format!("{}.evidence_authority", power.id),
                &power.evidence_authority,
            )))
        {
            if term
                .source_refs
                .iter()
                .any(|reference| !power.source_refs.contains(reference))
            {
                return Err(LedgerError::new(format!(
                    "{context}: term source must be a card source"
                )));
            }
            if !power_term_texts.insert(term.text.as_str()) {
                return Err(LedgerError::new(format!(
                    "{context}: repeated generic power-contract prose is prohibited"
                )));
            }
        }
        for (kind, test) in [
            ("negative", &power.negative_test),
            ("counterfactual", &power.counterfactual),
        ] {
            if test.id != format!("{}-{}", power.id, kind.to_ascii_uppercase())
                || test
                    .source_refs
                    .iter()
                    .any(|reference| !power.source_refs.contains(reference))
            {
                return Err(LedgerError::new(format!(
                    "{}.{} test identity or source binding drifted",
                    power.id, kind
                )));
            }
        }
        for (field, values) in [
            ("permitted_inputs", &power.permitted_inputs),
            ("prohibited_inputs", &power.prohibited_inputs),
            (
                "permitted_downstream_effects",
                &power.permitted_downstream_effects,
            ),
        ] {
            unique_strings(values, &format!("{}.{}", power.id, field), false)?;
        }
        if power.manifest_key != "formal-active-custody"
            && !power
                .prohibited_inputs
                .iter()
                .any(|value| value.contains("formal-active-custody"))
        {
            return Err(LedgerError::new(format!(
                "{}: every other power must prohibit T3 borrowing",
                power.id
            )));
        }
        for reference in &power.holder_body_refs {
            if !bodies.contains(reference.as_str()) {
                return Err(LedgerError::new(format!(
                    "{}: unknown power holder body {reference}",
                    power.id
                )));
            }
        }
        for reference in power
            .holder_role_refs
            .iter()
            .chain(power.affected_role_refs.iter())
            .chain(power.checking_role_refs.iter())
        {
            if !roles.contains(reference.as_str()) {
                return Err(LedgerError::new(format!(
                    "{}: unknown power role {reference}",
                    power.id
                )));
            }
        }
        if !routes.contains(power.route_ref.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: unknown power assurance route",
                power.id
            )));
        }
    }
    let powers_by_id = source
        .powers
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut allocated = HashSet::new();
    for allocation in &source.function_allocations {
        if !powers.contains(allocation.power_ref.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: function allocation names no power",
                allocation.id
            )));
        }
        if !allocated.insert(allocation.power_ref.as_str()) {
            return Err(LedgerError::new("one allocation cannot serve two powers"));
        }
        validate_allocation(allocation, &bodies, &roles)?;
        let power = powers_by_id[allocation.power_ref.as_str()];
        if allocation.affected_claim_refs != power.affected_claim_refs
            || allocation.separation_constraints.len() != power.required_separation_pairs.len()
        {
            return Err(LedgerError::new(format!(
                "{}: allocation claims or separation pairs differ from its power",
                allocation.id
            )));
        }
        let body_refs = |function: &str| -> &[String] {
            match function {
                "decisive-fact-writer" => &allocation.decisive_fact_writer_body_refs,
                "decider" => &allocation.decider_body_refs,
                "executor" => &allocation.executor_body_refs,
                "auditor" => &allocation.auditor_body_refs,
                "final-remedy" => &allocation.final_remedy_body_refs,
                _ => &[],
            }
        };
        for (constraint, pair) in allocation
            .separation_constraints
            .iter()
            .zip(&power.required_separation_pairs)
        {
            if constraint.functions != *pair
                || pair.len() != 2
                || body_refs(&pair[0])
                    .iter()
                    .any(|body| body_refs(&pair[1]).contains(body))
            {
                return Err(LedgerError::new(format!(
                    "{}: required function separation differs or is fused",
                    allocation.id
                )));
            }
        }
    }
    if allocated != powers {
        return Err(LedgerError::new("a power allocation cannot disappear"));
    }
    let retained = source
        .powers
        .iter()
        .find(|row| row.manifest_key == "formal-active-custody")
        .ok_or_else(|| LedgerError::new("retained formal T3 power is missing"))?;
    let custody = source
        .powers
        .iter()
        .find(|row| row.manifest_key == "protect-custodial-execution-mandate")
        .ok_or_else(|| LedgerError::new("custodial execution power is missing"))?;
    if retained.holder_body_refs != ["FS-BOD-17"]
        || custody.holder_body_refs != ["FS-BOD-35"]
        || retained.related_power_refs != [custody.id.as_str()]
        || custody.related_power_refs != [retained.id.as_str()]
    {
        return Err(LedgerError::new(
            "T3 Court authority and custody execution must remain reciprocal but separate",
        ));
    }
    let crosswalk_policy = BTreeMap::from([
        ("formal-electorate-seating-authority", "replace"),
        ("formal-public-body-authority", "replace"),
        ("formal-review-credential", "retire"),
        ("formal-tribunal-credential", "retire"),
        ("formal-appeals-expungement", "retire"),
        ("formal-appeals-relief", "replace"),
        ("formal-active-custody", "retain"),
        ("formal-amendment-label-result", "retire"),
    ]);
    if source.power_crosswalk_dispositions.iter().any(|row| {
        crosswalk_policy.get(row.manifest_key.as_str()).copied()
            != Some(row.crosswalk_action.as_str())
    }) {
        return Err(LedgerError::new(
            "formal crosswalk action violates checker-owned policy",
        ));
    }
    let mut effect_term_texts = HashSet::new();
    for (effect_index, effect) in source.constitutional_effects.iter().enumerate() {
        unique_strings(&effect.profiles, &format!("{}.profiles", effect.id), false)?;
        if effect
            .profiles
            .iter()
            .any(|profile| !effect.profile_terms.contains_key(profile))
            || effect
                .profile_terms
                .keys()
                .any(|profile| !effect.profiles.contains(profile))
        {
            return Err(LedgerError::new(format!(
                "{}: constitutional-effect profile declarations and terms differ",
                effect.id
            )));
        }
        validate_term_set(
            &effect.contract_terms,
            &format!("{}.contract_terms", effect.id),
        )?;
        validate_constitutional_effect_profile_terms(
            &effect.profile_terms,
            &format!("{}.profile_terms", effect.id),
        )?;
        validate_term(
            &effect.evidence_authority,
            &format!("{}.evidence_authority", effect.id),
        )?;
        validate_test_binding(
            &effect.negative_test,
            &format!("{}.negative_test", effect.id),
        )?;
        validate_test_binding(
            &effect.counterfactual,
            &format!("{}.counterfactual", effect.id),
        )?;
        for (context, term) in effect
            .contract_terms
            .iter()
            .map(|(name, term)| (format!("{}.contract_terms.{name}", effect.id), term))
            .chain(effect.profile_terms.iter().flat_map(|(profile, terms)| {
                terms.iter().map(move |(name, term)| {
                    (
                        format!("{}.profile_terms.{profile}.{name}", effect.id),
                        term,
                    )
                })
            }))
        {
            if term
                .source_refs
                .iter()
                .any(|reference| !effect.source_refs.contains(reference))
            {
                return Err(LedgerError::new(format!(
                    "{context}: term source must be a card source"
                )));
            }
            if !effect_term_texts.insert(term.text.as_str()) {
                return Err(LedgerError::new(format!(
                    "{context}: repeated generic effect-contract prose is prohibited"
                )));
            }
        }
        for (kind, test) in [
            ("negative", &effect.negative_test),
            ("counterfactual", &effect.counterfactual),
        ] {
            if test.id != format!("{}-{}", effect.id, kind.to_ascii_uppercase())
                || test
                    .source_refs
                    .iter()
                    .any(|reference| !effect.source_refs.contains(reference))
            {
                return Err(LedgerError::new(format!(
                    "{}.{} test identity or source binding drifted",
                    effect.id, kind
                )));
            }
        }
        for (field, values) in [
            ("permitted_inputs", &effect.permitted_inputs),
            ("prohibited_inputs", &effect.prohibited_inputs),
            (
                "permitted_downstream_effects",
                &effect.permitted_downstream_effects,
            ),
        ] {
            unique_strings(values, &format!("{}.{}", effect.id, field), false)?;
        }
        if effect
            .affected_claim_refs
            .iter()
            .any(|item| !claims.contains(item.as_str()))
            || effect
                .domain_refs
                .iter()
                .any(|item| !domains.contains(item.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: constitutional effect has dangling claim/domain references",
                effect.id
            )));
        }
        if effect
            .holder_role_refs
            .iter()
            .chain(effect.affected_role_refs.iter())
            .chain(effect.checking_role_refs.iter())
            .any(|reference| !roles.contains(reference.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: constitutional effect has a dangling role reference",
                effect.id
            )));
        }
        let prohibited = effect.prohibited_inputs.join(" ").to_ascii_lowercase();
        let required: &[&str] = if effect_index < 8 {
            &["registry"]
        } else {
            match effect.effect_key.as_str() {
                "material-floor-inventory" => &["ninth floor"],
                "substantive-equality-status" => &["person worth"],
                "custody-distinction-narrowing" => &["t3"],
                "legacy-status-nonproof" => &["omnibus family"],
                "family-status-no-confinement" => &["confinement"],
                "all-entitlement-nonreciprocity" => &["work"],
                "certified-positive-nonresponse" => &["wrong recipient"],
                _ => &[],
            }
        };
        if required.iter().any(|token| !prohibited.contains(token)) {
            return Err(LedgerError::new(format!(
                "{}: prohibited inputs omit a checker-required boundary",
                effect.id
            )));
        }
        if effect.effect_key == "public-fulfil-duty"
            && effect
                .checking_role_refs
                .iter()
                .any(|role| role == "FS-ROL-49")
        {
            return Err(LedgerError::new(format!(
                "{}: ecological scientist cannot stand in for an obligation institution",
                effect.id
            )));
        }
        for (adapter, power_profile) in [
            ("liberty-power-limit-adapter", "liberty-power-limit"),
            (
                "economic-private-power-limit-adapter",
                "economic-private-power-limit",
            ),
            ("class9-common-adapter", "commons-future-condition"),
        ] {
            let Some(adapter_terms) = effect.profile_terms.get(adapter) else {
                continue;
            };
            let expected = source
                .powers
                .iter()
                .find_map(|power| power.profile_terms.get(power_profile))
                .ok_or_else(|| {
                    LedgerError::new(format!(
                        "{}: no power profile supplies adapter {adapter}",
                        effect.id
                    ))
                })?;
            if adapter_terms.keys().ne(expected.keys()) {
                return Err(LedgerError::new(format!(
                    "{}: {adapter} must reuse every power-profile field",
                    effect.id
                )));
            }
        }
    }
    for dependency in &source.dependencies {
        if !FLOW_KINDS.contains(&dependency.flow_kind.as_str())
            || !DEPENDENCY_CLASSES.contains(&dependency.dependency_class.as_str())
            || !LIFECYCLE_PATHS.contains(&dependency.lifecycle_path.as_str())
            || dependency.from_ref == dependency.to_ref
        {
            return Err(LedgerError::new(format!(
                "{}: dependency classification or endpoint invalid",
                dependency.id
            )));
        }
        if !ids.contains(&dependency.from_ref) || !ids.contains(&dependency.to_ref) {
            return Err(LedgerError::new(format!(
                "{}: dependency endpoint is unknown",
                dependency.id
            )));
        }
    }
    for loop_row in &source.dependency_loops {
        if !LOOP_KINDS.contains(&loop_row.loop_kind.as_str())
            || loop_row.member_edge_refs.len() < 2
            || loop_row
                .member_edge_refs
                .iter()
                .any(|item| !dependencies.contains(item.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: dependency loop witness is invalid",
                loop_row.id
            )));
        }
    }
    for scenario in &source.scenarios {
        if !SCENARIO_KINDS.contains(&scenario.scenario_kind.as_str())
            || scenario
                .domain_refs
                .iter()
                .any(|item| !domains.contains(item.as_str()))
            || scenario
                .dependency_refs
                .iter()
                .any(|item| !dependencies.contains(item.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: scenario classification or reference is invalid",
                scenario.id
            )));
        }
        match scenario.scenario_kind.as_str() {
            "collision" if scenario.collision_axis.is_none() => {
                return Err(LedgerError::new(format!(
                    "{}: collision scenario needs a collision axis",
                    scenario.id
                )));
            }
            "compound-shock" if scenario.shock_kind.is_none() => {
                return Err(LedgerError::new(format!(
                    "{}: compound shock needs a shock kind",
                    scenario.id
                )));
            }
            _ => {}
        }
        if scenario
            .collision_axis
            .as_deref()
            .is_some_and(|axis| !COLLISION_AXES.contains(&axis))
            || scenario
                .shock_kind
                .as_deref()
                .is_some_and(|shock| !SHOCK_KINDS.contains(&shock))
            || scenario
                .protected_sphere_forms
                .as_ref()
                .is_some_and(|forms| {
                    forms
                        .iter()
                        .any(|form| !PROTECTED_SPHERE_FORMS.contains(&form.as_str()))
                })
        {
            return Err(LedgerError::new(format!(
                "{}: scenario closed enum drifted",
                scenario.id
            )));
        }
    }
    for control in &source.loop_hazard_controls {
        if !loops.contains(control.loop_ref.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: loop hazard names no loop",
                control.id
            )));
        }
    }
    let defects_by_id = source
        .defects
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    let compatibility = source
        .compatibility_table
        .iter()
        .map(|row| (row.defect_disposition.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut defect_keys = HashSet::new();
    for defect in &source.defects {
        validate_common_record(
            &defect.id,
            &defect.title,
            &defect.applicability,
            &defect.status,
            &defect.severity,
            &defect.consequence,
            &defect.owner_ref,
            &defect.closure_condition,
        )?;
        severity_class(defect)?;
        if !SCOPE_DISPOSITIONS.contains(&defect.layer.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: defect layer is outside the five dispositions",
                defect.id
            )));
        }
        if !defects_by_id.contains(defect.defect_id.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: defect_id must name the family's primary defect row",
                defect.id
            )));
        }
        if !is_slug(&defect.consequence_id) || !is_slug(&defect.scope_id) {
            return Err(LedgerError::new(format!(
                "{}: consequence_id and scope_id must be kebab-case slugs",
                defect.id
            )));
        }
        nonempty(
            &defect.source_version,
            &format!("{}.source_version", defect.id),
        )?;
        let compatibility_row = compatibility
            .get(defect.defect_disposition.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{}: unknown defect disposition", defect.id))
            })?;
        if !compatibility_row
            .allowed_response_stages
            .contains(&defect.response_stage)
        {
            return Err(LedgerError::new(format!(
                "{}: disposition and response stage are incompatible",
                defect.id
            )));
        }
        if !claims.contains(defect.affected_claim_ref.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: affected_claim_ref must name a claim",
                defect.id
            )));
        }
        if !envelope_ids.contains(defect.envelope_id.as_str()) {
            return Err(LedgerError::new(format!(
                "{}: envelope_id names no envelope",
                defect.id
            )));
        }
        if defect.response_stage == "operationally-assured-in-envelope"
            && !source.envelope.iter().any(|envelope| {
                envelope.id == defect.envelope_id && envelope.envelope_status == "calibrated"
            })
        {
            return Err(LedgerError::new(format!(
                "{}: operationally-assured requires a calibrated envelope",
                defect.id
            )));
        }
        let expected_gates = expected_defect_gate_refs(&defect.id)?;
        if defect
            .applicable_gate_refs
            .iter()
            .map(String::as_str)
            .ne(expected_gates.iter().copied())
        {
            return Err(LedgerError::new(format!(
                "{}: applicable_gate_refs differ from checker-owned classification",
                defect.id
            )));
        }
        for (index, history) in defect.history.iter().enumerate() {
            match history.field.as_str() {
                "defect_disposition" if DEFECT_DISPOSITIONS.contains(&history.value.as_str()) => {}
                "response_stage" if RESPONSE_STAGES.contains(&history.value.as_str()) => {}
                "defect_disposition" | "response_stage" => {
                    return Err(LedgerError::new(format!(
                        "{}.history[{index}]: prior value is outside its closed enum",
                        defect.id
                    )));
                }
                _ => {
                    return Err(LedgerError::new(format!(
                        "{}.history[{index}]: field must name disposition or stage",
                        defect.id
                    )));
                }
            }
            nonempty(
                &history.date,
                &format!("{}.history[{index}].date", defect.id),
            )?;
            nonempty(
                &history.note,
                &format!("{}.history[{index}].note", defect.id),
            )?;
        }
        for (field, values) in [
            ("evidence_notes", &defect.evidence_notes),
            ("residual_citations", &defect.residual_citations),
        ] {
            if values.iter().any(|value| value.trim().is_empty()) {
                return Err(LedgerError::new(format!(
                    "{}: {field} must contain non-empty strings",
                    defect.id
                )));
            }
        }
        if defect.book2_crosswalk.is_some_and(|value| !value) {
            return Err(LedgerError::new(format!(
                "{}: book2_crosswalk may only be true",
                defect.id
            )));
        }
        let controls = &defect.controls;
        let controls_count = usize::from(controls.reintroduction_control_ref.is_some())
            + usize::from(controls.initiation_control_ref.is_some())
            + usize::from(controls.containment_control_refs.is_some())
            + usize::from(controls.recovery_fields.is_some());
        let needs_controls = compatibility_row.resolution_eligible
            && IMPLEMENTED_STAGES.contains(&defect.response_stage.as_str());
        if !needs_controls && controls_count != 0 {
            return Err(LedgerError::new(format!(
                "{}: controls belong only on eligible implemented rows",
                defect.id
            )));
        }
        if needs_controls {
            let correct = match defect.defect_disposition.as_str() {
                "eliminated-structurally" => {
                    controls_count == 1 && controls.reintroduction_control_ref.is_some()
                }
                "prevented" => controls_count == 1 && controls.initiation_control_ref.is_some(),
                "protected-consequence-contained" => {
                    controls_count == 1
                        && controls
                            .containment_control_refs
                            .as_ref()
                            .is_some_and(|values| !values.is_empty())
                }
                "remedied" => controls_count == 1 && controls.recovery_fields.is_some(),
                _ => false,
            };
            if !correct {
                return Err(LedgerError::new(format!(
                    "{}: implemented disposition lacks its exact typed control",
                    defect.id
                )));
            }
        }
        for value in controls
            .reintroduction_control_ref
            .iter()
            .chain(controls.initiation_control_ref.iter())
            .chain(
                controls
                    .containment_control_refs
                    .iter()
                    .flat_map(|values| values.iter()),
            )
        {
            if value.starts_with("not-yet") {
                return Err(LedgerError::new(format!(
                    "{}: a not-yet control is not a control",
                    defect.id
                )));
            }
        }
        if let Some(recovery) = &controls.recovery_fields {
            for (field, value) in [
                ("actor", recovery.actor.as_str()),
                ("trigger", recovery.trigger.as_str()),
                ("interim_continuity", recovery.interim_continuity.as_str()),
                ("restoration", recovery.restoration.as_str()),
                ("challenge", recovery.challenge.as_str()),
                ("recurrence_control", recovery.recurrence_control.as_str()),
                ("evidence_ref", recovery.evidence_ref.as_str()),
            ] {
                nonempty(
                    value,
                    &format!("{}.controls.recovery_fields.{field}", defect.id),
                )?;
                if value.starts_with("not-yet") {
                    return Err(LedgerError::new(format!(
                        "{}: a not-yet recovery field is not a control",
                        defect.id
                    )));
                }
            }
        }
        let key = (
            defect.defect_id.as_str(),
            defect.affected_claim_ref.as_str(),
            defect.consequence_id.as_str(),
            defect.scope_id.as_str(),
            defect.envelope_id.as_str(),
            defect.source_version.as_str(),
        );
        if !defect_keys.insert(key) {
            return Err(LedgerError::new(format!(
                "{}: duplicate defect keying tuple",
                defect.id
            )));
        }
    }
    Ok(())
}

fn validate_term_set(terms: &TermSet, context: &str) -> LedgerResult<()> {
    if terms.is_empty() {
        return Err(LedgerError::new(format!("{context} must be non-empty")));
    }
    for (key, term) in terms {
        validate_term(term, &format!("{context}.{key}"))?;
    }
    Ok(())
}

fn validate_profile_terms(terms: &ProfileTerms, context: &str) -> LedgerResult<()> {
    if terms.is_empty() {
        return Err(LedgerError::new(format!("{context} must be non-empty")));
    }
    for (profile, values) in terms {
        validate_term_set(values, &format!("{context}.{profile}"))?;
    }
    Ok(())
}

fn validate_constitutional_effect_profile_terms(
    terms: &ProfileTerms,
    context: &str,
) -> LedgerResult<()> {
    validate_profile_terms(terms, context)?;
    let Some(boundary) = terms.get("democratic-policy-boundary") else {
        return Ok(());
    };
    let actual = boundary.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = DEMOCRATIC_POLICY_BOUNDARY_TERM_KEYS
        .into_iter()
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(LedgerError::new(format!(
            "{context}.democratic-policy-boundary must contain exactly: {}",
            DEMOCRATIC_POLICY_BOUNDARY_TERM_KEYS.join(", ")
        )));
    }
    Ok(())
}

fn validate_term(term: &Term, context: &str) -> LedgerResult<()> {
    nonempty(&term.text, &format!("{context}.text"))?;
    nonempty(&term.basis, &format!("{context}.basis"))?;
    let lower = term.text.trim().to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "n/a" | "na" | "tbd" | "unknown" | "unresolved"
    ) || lower
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| matches!(token, "tbd" | "unresolved"))
        || lower.contains("is fixed by the source-bound")
    {
        return Err(LedgerError::new(format!(
            "{context}: unresolved or legacy generic prose is not a contract term"
        )));
    }
    unique_strings(&term.source_refs, &format!("{context}.source_refs"), false)?;
    let delegated = [
        term.choice_owner.as_deref(),
        term.bounds.as_deref(),
        term.failure_default.as_deref(),
    ];
    if delegated.iter().any(Option::is_some) && delegated.iter().any(Option::is_none) {
        return Err(LedgerError::new(format!(
            "{context}: bounded delegation requires choice_owner, bounds, and failure_default together"
        )));
    }
    Ok(())
}

fn validate_test_binding(test: &TestBinding, context: &str) -> LedgerResult<()> {
    for (field, value) in [
        ("id", &test.id),
        ("status", &test.status),
        ("assertion", &test.assertion),
    ] {
        nonempty(value, &format!("{context}.{field}"))?;
    }
    if let Some(executable_ref) = &test.executable_ref.0 {
        nonempty(executable_ref, &format!("{context}.executable_ref"))?;
    }
    if !matches!(test.status.as_str(), "planned" | "executable")
        || (test.status == "executable") != test.executable_ref.0.is_some()
    {
        return Err(LedgerError::new(format!(
            "{context}: status and executable_ref must agree"
        )));
    }
    unique_strings(&test.source_refs, &format!("{context}.source_refs"), false)
}

fn validate_allocation(
    allocation: &FunctionAllocation,
    bodies: &HashSet<&str>,
    roles: &HashSet<&str>,
) -> LedgerResult<()> {
    let body_groups = [
        &allocation.decisive_fact_writer_body_refs,
        &allocation.decider_body_refs,
        &allocation.executor_body_refs,
        &allocation.auditor_body_refs,
        &allocation.final_remedy_body_refs,
    ];
    let role_groups = [
        &allocation.decisive_fact_writer_role_refs,
        &allocation.decider_role_refs,
        &allocation.executor_role_refs,
        &allocation.auditor_role_refs,
        &allocation.final_remedy_role_refs,
    ];
    if body_groups
        .iter()
        .flat_map(|values| values.iter())
        .any(|item| !bodies.contains(item.as_str()))
        || role_groups
            .iter()
            .flat_map(|values| values.iter())
            .any(|item| !roles.contains(item.as_str()))
    {
        return Err(LedgerError::new(format!(
            "{}: function allocation has an unknown body or role",
            allocation.id
        )));
    }
    for constraint in &allocation.separation_constraints {
        if constraint.functions.len() != 2 || constraint.functions[0] == constraint.functions[1] {
            return Err(LedgerError::new(format!(
                "{}: required function separation must name two distinct functions",
                allocation.id
            )));
        }
        nonempty(&constraint.reason, "separation constraint reason")?;
        nonempty(&constraint.source_ref, "separation constraint source_ref")?;
    }
    Ok(())
}

fn compute_resolution(source: &LedgerDocument) -> LedgerResult<BTreeMap<String, DefectResolution>> {
    let compatibility = source
        .compatibility_table
        .iter()
        .map(|row| (row.defect_disposition.as_str(), row))
        .collect::<HashMap<_, _>>();
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let routes = source
        .routes
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let receipt_counts = source
        .receipts
        .iter()
        .fold(HashMap::new(), |mut counts, receipt| {
            *counts
                .entry(receipt.defect_row_ref.as_str())
                .or_insert(0_usize) += 1;
            counts
        });
    let mut result = BTreeMap::new();
    for defect in &source.defects {
        if defect.defect_id.trim().is_empty()
            || !DEFECT_DISPOSITIONS.contains(&defect.defect_disposition.as_str())
            || !RESPONSE_STAGES.contains(&defect.response_stage.as_str())
            || defect
                .applicable_gate_refs
                .iter()
                .any(|gate| !GATE_REFS.contains(&gate.as_str()))
        {
            return Err(LedgerError::new(format!(
                "{}: defect classification is invalid",
                defect.id
            )));
        }
        let row = compatibility
            .get(defect.defect_disposition.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{}: defect has no compatibility row", defect.id))
            })?;
        if !row.allowed_response_stages.contains(&defect.response_stage) {
            return Err(LedgerError::new(format!(
                "{}: disposition and response stage are incompatible",
                defect.id
            )));
        }
        let claim = claims
            .get(defect.affected_claim_ref.as_str())
            .ok_or_else(|| LedgerError::new(format!("{}: affected claim is unknown", defect.id)))?;
        let _route = routes.get(claim.route_ref.as_str()).ok_or_else(|| {
            LedgerError::new(format!("{}: affected claim route is unknown", defect.id))
        })?;
        let implemented = [
            "implemented-in-assigned-route",
            "operationally-assured-in-envelope",
        ]
        .contains(&defect.response_stage.as_str());
        let controls_ok = match defect.defect_disposition.as_str() {
            "eliminated-structurally" => defect.controls.reintroduction_control_ref.is_some(),
            "prevented" => defect.controls.initiation_control_ref.is_some(),
            "protected-consequence-contained" => defect
                .controls
                .containment_control_refs
                .as_ref()
                .is_some_and(|values| !values.is_empty()),
            "remedied" => defect.controls.recovery_fields.is_some(),
            _ => false,
        };
        let ceiling_ok = ["Derived", "Checked", "Evidenced"].contains(&claim.posture.as_str());
        let candidate = row.resolution_eligible && implemented && controls_ok && ceiling_ok;
        let resolved = candidate && receipt_counts.get(defect.id.as_str()) == Some(&1);
        let blocking = defect.severity.starts_with("critical — ") && !resolved;
        result.insert(
            defect.id.clone(),
            DefectResolution {
                candidate,
                resolution_status: if resolved {
                    "resolved-for-claim"
                } else {
                    "unresolved-for-claim"
                },
                blocking,
            },
        );
    }
    Ok(result)
}

fn validate_receipts(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
) -> LedgerResult<()> {
    let defects = source
        .defects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut covered = HashSet::new();
    for receipt in &source.receipts {
        let defect = defects
            .get(receipt.defect_row_ref.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{}: receipt names no defect row", receipt.id))
            })?;
        let generated = resolution.get(&defect.id).expect("defects were resolved");
        let claim = claims
            .get(defect.affected_claim_ref.as_str())
            .ok_or_else(|| {
                LedgerError::new(format!("{}: receipt defect names no claim", receipt.id))
            })?;
        for (field, value) in [
            ("title", receipt.title.as_str()),
            ("admissible_evidence", receipt.admissible_evidence.as_str()),
            ("what_failed", receipt.what_failed.as_str()),
            ("hostile_witness", receipt.hostile_witness.as_str()),
            ("why_it_failed", receipt.why_it_failed.as_str()),
            ("response_change", receipt.response_change.as_str()),
            ("now_follows", receipt.now_follows.as_str()),
            (
                "still_does_not_follow",
                receipt.still_does_not_follow.as_str(),
            ),
            ("eligible_gate", receipt.eligible_gate.as_str()),
        ] {
            nonempty(value, &format!("{}.{field}", receipt.id))?;
        }
        if receipt.defect_id != defect.defect_id
            || receipt.affected_claim_ref != defect.affected_claim_ref
            || receipt.consequence_id != defect.consequence_id
            || receipt.defect_disposition != defect.defect_disposition
            || receipt.response_stage != defect.response_stage
            || receipt.scope_id != defect.scope_id
            || receipt.source_version != defect.source_version
            || receipt.envelope_id != defect.envelope_id
            || receipt.claim_posture != claim.posture
            || receipt.route_ref != claim.route_ref
            || receipt.assurance_ceiling != claim.posture
        {
            return Err(LedgerError::new(format!(
                "{}: receipt is not an exact projection of its resolved defect",
                receipt.id
            )));
        }
        if !covered.insert(defect.id.as_str()) {
            return Err(LedgerError::new(
                "a resolved defect may have only one receipt",
            ));
        }
        for (field, value) in [
            ("proof_ref", &receipt.proof_ref),
            ("negative_control_ref", &receipt.negative_control_ref),
            ("reader_mapping_ref", &receipt.reader_mapping_ref),
            ("owner_ref", &receipt.owner_ref),
        ] {
            nonempty(value, &format!("{}.{field}", receipt.id))?;
        }
        if receipt.residuals.is_empty()
            || receipt
                .residuals
                .iter()
                .any(|value| value.trim().is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: residuals must be a non-empty list",
                receipt.id
            )));
        }
        let siblings = source
            .defects
            .iter()
            .filter(|candidate| {
                candidate.defect_id == defect.defect_id && candidate.id != defect.id
            })
            .map(|candidate| candidate.id.as_str())
            .collect::<HashSet<_>>();
        let named_siblings = receipt
            .residuals
            .iter()
            .filter(|reference| defects.contains_key(reference.as_str()))
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if named_siblings
            .iter()
            .any(|reference| !siblings.contains(reference))
            || (!siblings.is_empty() && named_siblings.is_empty())
        {
            return Err(LedgerError::new(format!(
                "{}: named defect residuals must be siblings of the receipted row",
                receipt.id
            )));
        }
        if !generated.candidate {
            return Err(LedgerError::new(format!(
                "{}: receipt names a non-candidate defect",
                receipt.id
            )));
        }
    }
    let expected = resolution
        .iter()
        .filter_map(|(id, row)| {
            (row.resolution_status == "resolved-for-claim").then_some(id.as_str())
        })
        .collect::<HashSet<_>>();
    if covered != expected {
        return Err(LedgerError::new(
            "resolution receipts must cover exactly the generated eligible resolutions",
        ));
    }
    Ok(())
}

fn validate_acceptance_and_closure(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<()> {
    let closure = &source.closure_record.0;
    let closed = closure.is_some();
    let expected_verdict = if closed {
        "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A PASSED"
    } else {
        "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED"
    };
    if source.acceptance_gate.verdict != expected_verdict
        || source.acceptance_gate.gate_a_status != if closed { "passed" } else { "not-passed" }
        || source.acceptance_gate.rollup_rule.trim().is_empty()
        || source
            .acceptance_gate
            .rollup_rule
            .chars()
            .any(|character| character.is_ascii_digit())
    {
        return Err(LedgerError::new(
            "acceptance gate must be derived exactly from closure-record presence",
        ));
    }
    let Some(closure) = closure else {
        return Ok(());
    };
    if closure.gate != "gate-a"
        || closure.permitted_claim
            != "The project has a versioned, reviewable scope map and assurance program."
        || closure.source_version != source.source_version
        || closure.envelope_ref != "FS-ENV-01"
        || closure.candidate_commit_sha.len() != 40
        || !closure
            .candidate_commit_sha
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        || closure.scope_sha256.len() != 64
        || !closure
            .scope_sha256
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(LedgerError::new("closure record identity contract drifted"));
    }
    let envelope = source
        .envelope
        .iter()
        .find(|row| row.id == closure.envelope_ref)
        .ok_or_else(|| LedgerError::new("closure record envelope is unknown"))?;
    if envelope.envelope_status != "versioned-structure" {
        return Err(LedgerError::new(
            "closure record envelope must remain versioned structure",
        ));
    }
    let audit = source
        .scope_audits
        .iter()
        .find(|row| row.id == closure.scope_audit_ref)
        .ok_or_else(|| LedgerError::new("closure_record: scope audit is unknown"))?;
    if closure.scope_sha256 != review_scope_digest(source)?
        || audit.source_version != closure.source_version
        || audit.scope_sha256 != closure.scope_sha256
        || audit.protocol_sha256 != protocol_digest(inputs)?
        || audit.executed_at_utc != closure.audit_cutoff_at_utc
        || audit.result != "passed-with-recorded-limits"
        || audit.verification_receipt_ref.as_deref()
            != Some(closure.verification_receipt_ref.as_str())
    {
        return Err(LedgerError::new(
            "closure record must exactly bind its qualifying scope audit",
        ));
    }
    unique_strings(
        &closure.assurance_record_refs,
        "closure assurance refs",
        false,
    )?;
    if closure
        .assurance_record_refs
        .iter()
        .map(String::as_str)
        .ne(GATE_A_ASSURANCE_REFS)
    {
        return Err(LedgerError::new(
            "closure assurance_record_refs must equal the checker-derived set",
        ));
    }
    unique_strings(&closure.residual_refs, "closure residual refs", true)?;
    let mut expected_residuals = source
        .defects
        .iter()
        .filter_map(|defect| {
            let generated = resolution.get(&defect.id)?;
            (generated.resolution_status == "unresolved-for-claim"
                && !defect.severity.starts_with("critical — ")
                && defect
                    .applicable_gate_refs
                    .iter()
                    .any(|gate| gate == "gate-a"))
            .then_some(defect.id.clone())
        })
        .collect::<Vec<_>>();
    expected_residuals.sort();
    if closure.residual_refs != expected_residuals {
        return Err(LedgerError::new(
            "closure residual_refs must equal the checker-derived set",
        ));
    }
    let claims = source
        .claims
        .iter()
        .map(|claim| (claim.id.as_str(), claim))
        .collect::<HashMap<_, _>>();
    let defects = source
        .defects
        .iter()
        .map(|defect| (defect.id.as_str(), defect))
        .collect::<HashMap<_, _>>();
    let expected_limitations = expected_residuals
        .iter()
        .map(|id| {
            let defect = defects
                .get(id.as_str())
                .ok_or_else(|| LedgerError::new("derived residual defect is missing"))?;
            let claim = claims
                .get(defect.affected_claim_ref.as_str())
                .ok_or_else(|| LedgerError::new("derived residual claim is missing"))?;
            Ok(ClaimLimitation {
                defect_ref: id.clone(),
                affected_claim_ref: defect.affected_claim_ref.clone(),
                public_claim_restriction: claim.public_claim_restriction.clone(),
            })
        })
        .collect::<LedgerResult<Vec<_>>>()?;
    if closure.claim_limitations != expected_limitations {
        return Err(LedgerError::new(
            "closure claim_limitations must bind every derived residual exactly",
        ));
    }
    if closure.closure_policy_ref != SCOPE_AUDIT_POLICY_BASIS {
        return Err(LedgerError::new(
            "closure closure_policy_ref must equal the checker-owned policy",
        ));
    }
    if !is_content_addressed_receipt_ref(&closure.verification_receipt_ref) {
        return Err(LedgerError::new(
            "closure verification_receipt_ref must be a content-addressed v2 receipt",
        ));
    }
    if source.defects.iter().any(|defect| {
        resolution.get(&defect.id).is_some_and(|generated| {
            generated.blocking
                && defect
                    .applicable_gate_refs
                    .iter()
                    .any(|gate| gate == "gate-a")
        })
    }) {
        return Err(LedgerError::new(
            "closure record may not exist while a Gate-A condition computes unmet",
        ));
    }
    Ok(())
}

fn validate_reader_alignment(
    source: &LedgerDocument,
    projection: &reader::ReaderLedgerProjection,
) -> LedgerResult<()> {
    let route = source
        .routes
        .iter()
        .find(|row| row.id == "FS-RTE-06")
        .ok_or_else(|| LedgerError::new("reader alignment: FS-RTE-06 is missing"))?;
    let claim = source
        .claims
        .iter()
        .find(|row| row.id == "FS-CLM-37")
        .ok_or_else(|| LedgerError::new("reader alignment: FS-CLM-37 is missing"))?;
    reader::validate_reader_evidence_alignment(
        projection,
        reader::ReaderRouteAlignment {
            id: &route.id,
            status: &route.status,
            route_status: &route.route_status,
        },
        reader::ReaderClaimAlignment {
            id: &claim.id,
            route_ref: &claim.route_ref,
            posture: &claim.posture,
            unestablished_disposition: claim.unestablished_disposition.as_deref(),
        },
    )
    .map_err(|error| LedgerError::new(format!("reader-evidence alignment invalid: {error}")))
}

fn validate_source_with_projections(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    siblings: &SiblingProjections,
    reader_projection: &reader::ReaderLedgerProjection,
) -> LedgerResult<BTreeMap<String, DefectResolution>> {
    validate_header(source)?;
    validate_bound_sources(inputs, source)?;
    validate_typed_repository_references(inputs, source)?;
    validate_meanings(source)?;
    validate_axes(source)?;
    validate_compatibility(source)?;
    validate_power_binding(inputs, source)?;
    let ids = register_ids(source)?;
    validate_records(source, &ids)?;
    validate_power_effect_coverage_policy(inputs, source)?;
    validate_body_map_cells(inputs, source)?;
    validate_scenario_records(source, siblings)?;
    validate_sibling_closures(source, siblings, reader_projection)?;
    validate_review_contract(inputs, source)?;
    validate_optional_review_records(inputs, source, &ids)?;
    validate_reader_alignment(source, reader_projection)?;
    let resolution = compute_resolution(source)?;
    validate_receipts(source, &resolution)?;
    validate_acceptance_and_closure(source, &resolution, inputs)?;
    Ok(resolution)
}

fn validate_source_with_inputs(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    reader_projection: &reader::ReaderLedgerProjection,
) -> LedgerResult<BTreeMap<String, DefectResolution>> {
    let siblings = SiblingProjections::parse(inputs)?;
    validate_source_with_projections(inputs, source, &siblings, reader_projection)
}

fn validate_source(
    context: &Context,
    source: &LedgerDocument,
) -> LedgerResult<BTreeMap<String, DefectResolution>> {
    let inputs = load_static_inputs(context)?;
    let reader_projection = load_reader_projection(context, &inputs)?;
    let resolutions = validate_source_with_inputs(&inputs, source, &reader_projection)?;
    validate_review_history(context, source)?;
    validate_current_audit_receipts(context, &inputs, source)?;
    Ok(resolutions)
}

fn transient_control_audit(source: &LedgerDocument, title: &str) -> LedgerResult<ScopeAudit> {
    let mut audit = source
        .scope_audits
        .last()
        .cloned()
        .ok_or_else(|| LedgerError::new("control setup: source has no scope audit"))?;
    audit.id = "FS-SAU-99".into();
    audit.title = title.into();
    audit.control_refs = vec![
        LEDGER_CURRENT_AUDIT_CONTROL_REF.into(),
        closure::CURRENT_AUDIT_CONTROL_REF.into(),
    ];
    audit.commands = CURRENT_AUDIT_COMMAND_PREFIX
        .iter()
        .map(|command| (*command).to_owned())
        .collect();
    if audit.verification_receipt_ref.is_some() {
        audit.result = "pending".into();
        audit.verification_receipt_ref = None;
    }
    Ok(audit)
}

fn synthesize_control_closure(source: &mut LedgerDocument) -> LedgerResult<()> {
    if source.closure_record.0.is_some() {
        return Ok(());
    }

    // The governed checker builds an otherwise complete closure fixture before
    // applying every closure mutation. Semantic candidates correctly carry
    // `closure_record: null`, so native controls must do the same instead of
    // assuming that the production source is already closed.
    let resolution = compute_resolution(source)?;
    let mut residual_refs = source
        .defects
        .iter()
        .filter_map(|defect| {
            let generated = resolution.get(&defect.id)?;
            (generated.resolution_status == "unresolved-for-claim"
                && !defect.severity.starts_with("critical — ")
                && defect
                    .applicable_gate_refs
                    .iter()
                    .any(|gate| gate == "gate-a"))
            .then_some(defect.id.clone())
        })
        .collect::<Vec<_>>();
    residual_refs.sort();
    let claims = source
        .claims
        .iter()
        .map(|claim| (claim.id.as_str(), claim))
        .collect::<HashMap<_, _>>();
    let defects = source
        .defects
        .iter()
        .map(|defect| (defect.id.as_str(), defect))
        .collect::<HashMap<_, _>>();
    let claim_limitations = residual_refs
        .iter()
        .map(|id| {
            let defect = defects
                .get(id.as_str())
                .ok_or_else(|| LedgerError::new("control residual defect is missing"))?;
            let claim = claims
                .get(defect.affected_claim_ref.as_str())
                .ok_or_else(|| LedgerError::new("control residual claim is missing"))?;
            Ok(ClaimLimitation {
                defect_ref: id.clone(),
                affected_claim_ref: defect.affected_claim_ref.clone(),
                public_claim_restriction: claim.public_claim_restriction.clone(),
            })
        })
        .collect::<LedgerResult<Vec<_>>>()?;
    let scope_sha256 = review_scope_digest(source)?;
    let (audit_cutoff_at_utc, scope_audit_ref, verification_receipt_ref) = {
        let audit = source
            .scope_audits
            .last_mut()
            .ok_or_else(|| LedgerError::new("control setup: source has no scope audit"))?;
        audit.scope_sha256.clone_from(&scope_sha256);
        (
            audit.executed_at_utc.clone(),
            audit.id.clone(),
            audit.verification_receipt_ref.clone().unwrap_or_else(|| {
                format!(
                    "new-book-plans/verification-receipts/sha256-{}.json",
                    "0".repeat(64)
                )
            }),
        )
    };

    source.closure_record.0 = Some(ClosureProjection {
        gate: "gate-a".into(),
        permitted_claim: "The project has a versioned, reviewable scope map and assurance program."
            .into(),
        candidate_commit_sha: "0".repeat(40),
        source_version: source.source_version.clone(),
        scope_sha256,
        envelope_ref: "FS-ENV-01".into(),
        audit_cutoff_at_utc,
        scope_audit_ref,
        assurance_record_refs: GATE_A_ASSURANCE_REFS
            .iter()
            .map(|reference| (*reference).to_owned())
            .collect(),
        residual_refs,
        claim_limitations,
        closure_policy_ref: SCOPE_AUDIT_POLICY_BASIS.into(),
        verification_receipt_ref,
    });
    source.acceptance_gate.verdict = "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A PASSED".into();
    source.acceptance_gate.gate_a_status = "passed".into();
    Ok(())
}

fn run_typed_negative_control<F>(
    ledger: &ValidatedLedger,
    name: &str,
    closure_mutation: bool,
    preserve_scope_digest: bool,
    expected_error: Option<&str>,
    mutate: F,
) -> LedgerResult<()>
where
    F: FnOnce(&mut LedgerDocument) -> LedgerResult<()>,
{
    let mut mutant = ledger.document.clone();
    if closure_mutation {
        synthesize_control_closure(&mut mutant).map_err(|error| {
            LedgerError::new(format!("negative control setup failed: {name}: {error}"))
        })?;
    } else {
        mutant.scope_audits.push(transient_control_audit(
            &mutant,
            "Watched-mutation current audit",
        )?);
        mutant.closure_record.0 = None;
        mutant.acceptance_gate.verdict = "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED".into();
        mutant.acceptance_gate.gate_a_status = "not-passed".into();
    }
    mutate(&mut mutant).map_err(|error| {
        LedgerError::new(format!("negative control setup failed: {name}: {error}"))
    })?;
    if !closure_mutation && !preserve_scope_digest {
        let digest = review_scope_digest(&mutant)?;
        mutant
            .scope_audits
            .last_mut()
            .expect("transient audit was appended")
            .scope_sha256 = digest;
    }
    match validate_source_with_projections(
        &ledger.input_bytes,
        &mutant,
        &ledger.sibling_projections,
        &ledger.reader_projection,
    ) {
        Err(error)
            if expected_error.is_none_or(|expected| error.to_string().contains(expected))
                && (!preserve_scope_digest
                    || expected_error.is_some_and(|expected| {
                        expected.contains("current-source repository audit")
                    })
                    || !error
                        .to_string()
                        .contains("requires a current-source repository audit")) =>
        {
            Ok(())
        }
        Err(error) => Err(LedgerError::new(format!(
            "negative control failed for the wrong reason: {name} — expected {:?} in {error}",
            expected_error.unwrap_or("a semantic invariant before current-audit fallback")
        ))),
        Ok(_) => Err(LedgerError::new(format!(
            "negative control failed to fail: {name}"
        ))),
    }
}

fn run_shape_negative_control<F>(
    ledger: &ValidatedLedger,
    name: &str,
    expected_error: Option<&str>,
    closure_fixture: bool,
    mutate: F,
) -> LedgerResult<()>
where
    F: FnOnce(&mut Value) -> LedgerResult<()>,
{
    // Generic JSON is intentional here: these controls remove required fields
    // or insert forbidden fields and therefore cannot be represented by the
    // production typed contract whose rejection they exercise.
    let mut document = ledger.document.clone();
    if closure_fixture {
        synthesize_control_closure(&mut document).map_err(|error| {
            LedgerError::new(format!("negative control setup failed: {name}: {error}"))
        })?;
    }
    let mut value = serde_json::to_value(&document).map_err(|error| {
        LedgerError::new(format!(
            "negative control cannot serialize typed source: {error}"
        ))
    })?;
    mutate(&mut value).map_err(|error| {
        LedgerError::new(format!("negative control setup failed: {name}: {error}"))
    })?;
    let bytes = serde_json::to_vec(&value).map_err(|error| {
        LedgerError::new(format!(
            "negative control cannot serialize malformed source: {error}"
        ))
    })?;
    match parse_source(&bytes) {
        Err(error)
            if expected_error.is_none_or(|expected| error.to_string().contains(expected)) =>
        {
            Ok(())
        }
        Err(error) => Err(LedgerError::new(format!(
            "negative control failed for the wrong reason: {name} — expected {:?} in {error}",
            expected_error.expect("guarded above")
        ))),
        Ok(_) => Err(LedgerError::new(format!(
            "negative control failed to fail: {name}"
        ))),
    }
}

fn value_object_mut<'a>(
    value: &'a mut Value,
    context: &str,
) -> LedgerResult<&'a mut serde_json::Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| LedgerError::new(format!("control setup: {context} is not an object")))
}

fn value_array_mut<'a>(value: &'a mut Value, key: &str) -> LedgerResult<&'a mut Vec<Value>> {
    value_object_mut(value, "root")?
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| LedgerError::new(format!("control setup: {key} is not an array")))
}

fn negative_controls_claims_and_defects(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, true, None, $mutate)?;
            passed += 1;
        }};
        ($name:literal, $expected:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, true, Some($expected), $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! shape {
        ($name:literal, $mutate:expr) => {{
            run_shape_negative_control(ledger, $name, None, false, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! typed_rehash {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, false, None, $mutate)?;
            passed += 1;
        }};
        ($name:literal, $expected:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, false, Some($expected), $mutate)?;
            passed += 1;
        }};
    }

    shape!("generic disposition key is refused", |value| {
        let claim = value_array_mut(value, "claims")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no claim"))?;
        value_object_mut(claim, "claims[0]")?.insert("disposition".into(), "open".into());
        Ok(())
    });
    typed!("numeric rollup is an aggregate score", |source| {
        source.acceptance_gate.rollup_rule = "9 of 10 rows established".into();
        Ok(())
    });
    shape!("score key is refused", |value| {
        let domain = value_array_mut(value, "domains")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no domain"))?;
        value_object_mut(domain, "domains[0]")?.insert("score".into(), "high".into());
        Ok(())
    });
    typed!("partial formalisation must be split", |source| {
        source.claims[0].status = "partial formalisation".into();
        Ok(())
    });
    typed!("a claim needs a recognised posture", |source| {
        source.claims[0].posture = "Probable".into();
        Ok(())
    });
    typed!("two postures is two records", |source| {
        source.claims[0].posture = "Derived; Specified".into();
        Ok(())
    });
    typed!("Derived requires executable evidence kind", |source| {
        let claim = source
            .claims
            .iter_mut()
            .find(|claim| claim.posture == "Derived")
            .ok_or_else(|| LedgerError::new("control setup: no Derived claim"))?;
        claim.evidence_kind = Some("inventory".into());
        Ok(())
    });
    typed!("liveness may not be Derived", |source| {
        let claim = source
            .claims
            .iter_mut()
            .find(|claim| claim.posture == "Derived")
            .ok_or_else(|| LedgerError::new("control setup: no Derived claim"))?;
        claim.overlay = "liveness".into();
        Ok(())
    });
    typed!("feasibility may not appear at all", |source| {
        source.claims[0].overlay = "feasibility".into();
        Ok(())
    });
    typed!(
        "established posture needs a built or available route",
        |source| {
            let route = source
                .routes
                .iter()
                .find(|route| route.route_status == "unbuilt")
                .map(|route| route.id.clone())
                .ok_or_else(|| LedgerError::new("control setup: no unbuilt route"))?;
            let claim = source
                .claims
                .iter_mut()
                .find(|claim| claim.posture == "Derived")
                .ok_or_else(|| LedgerError::new("control setup: no Derived claim"))?;
            claim.route_ref = route;
            Ok(())
        }
    );
    typed!("a built route must keep its negative control", |source| {
        let route = source
            .routes
            .iter_mut()
            .find(|route| route.route_status == "built")
            .ok_or_else(|| LedgerError::new("control setup: no built route"))?;
        route.negative_control = "not-yet-declared".into();
        Ok(())
    });
    typed!("deleted enum-mapping row fails closure", |source| {
        source.enum_mapping.remove(0);
        Ok(())
    });
    typed!("remedied + detected is invalid", |source| {
        source
            .compatibility_table
            .iter_mut()
            .find(|row| row.defect_disposition == "remedied")
            .ok_or_else(|| LedgerError::new("control setup: no remedied compatibility row"))?
            .allowed_response_stages = vec!["detected".into()];
        Ok(())
    });
    shape!("hand-authored resolution_status is refused", |value| {
        let claim = value_array_mut(value, "claims")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no claim"))?;
        value_object_mut(claim, "claims[0]")?
            .insert("resolution_status".into(), "resolved-for-claim".into());
        Ok(())
    });
    typed!("empty array without deferral is refused", |source| {
        source.thresholds.clear();
        Ok(())
    });
    typed!("the envelope stub can route, never assure", |source| {
        let mut defect = source.defects[0].clone();
        defect.id = "FS-DFT-999".into();
        defect.defect_id = "FS-DFT-999".into();
        defect.title = "control".into();
        defect.applicability = "control".into();
        defect.layer = "constitutional-invariant".into();
        defect.status = "control".into();
        defect.severity = "material — control".into();
        defect.consequence = "control".into();
        defect.owner_ref = source.domains[0].source_refs[0].clone();
        defect.closure_condition = "control".into();
        defect.defect_disposition = "remedied".into();
        defect.response_stage = "operationally-assured-in-envelope".into();
        defect.affected_claim_ref = source.claims[0].id.clone();
        defect.consequence_id = "control".into();
        defect.scope_id = "control".into();
        defect.envelope_id = ENVELOPE_STUB_ID.into();
        defect.source_version = "control".into();
        defect.history.clear();
        defect.evidence_notes.clear();
        defect.residual_citations.clear();
        defect.controls = DefectControls {
            reintroduction_control_ref: None,
            initiation_control_ref: None,
            containment_control_refs: None,
            recovery_fields: None,
        };
        defect.applicable_gate_refs = vec!["gate-a".into()];
        source.defects.push(defect);
        Ok(())
    });
    typed!("a stale bound-source digest is caught", |source| {
        source.bound_sources_sha256.assurance_portfolio = "0".repeat(64);
        Ok(())
    });
    typed!("a broken needle is caught", |source| {
        source.domains[0].source_refs[0] = "TODO.md::negative-control-anchor-does-not-exist".into();
        Ok(())
    });
    typed!("a Specified row needs its unimplemented marker", |source| {
        source
            .claims
            .iter_mut()
            .find(|claim| claim.posture == "Specified")
            .ok_or_else(|| LedgerError::new("control setup: no Specified claim"))?
            .unimplemented_marker = None;
        Ok(())
    });
    typed!("an Unestablished row needs a named disposition", |source| {
        source
            .claims
            .iter_mut()
            .find(|claim| claim.posture == "Unestablished")
            .ok_or_else(|| LedgerError::new("control setup: no Unestablished claim"))?
            .unestablished_disposition = None;
        Ok(())
    });
    typed!("a domain layer must be the sentinel", |source| {
        source.domains[0].layer = "constitutional-invariant".into();
        Ok(())
    });
    typed!(
        "route-unbuilt requires an unbuilt route",
        "requires an unbuilt route",
        |source| {
            let built = source
                .routes
                .iter()
                .find(|route| route.route_status == "built")
                .map(|route| route.id.clone())
                .ok_or_else(|| LedgerError::new("control setup: no built route"))?;
            source
                .claims
                .iter_mut()
                .find(|claim| claim.unestablished_disposition.as_deref() == Some("route-unbuilt"))
                .ok_or_else(|| LedgerError::new("control setup: no route-unbuilt claim"))?
                .route_ref = built;
            Ok(())
        }
    );
    typed!(
        "evidence-pending requires a built or available route",
        "requires a built or available route",
        |source| {
            source
                .claims
                .iter_mut()
                .find(|claim| claim.unestablished_disposition.as_deref() == Some("route-unbuilt"))
                .ok_or_else(|| LedgerError::new("control setup: no route-unbuilt claim"))?
                .unestablished_disposition = Some("evidence-pending".into());
            Ok(())
        }
    );
    typed!("verdict line is byte-exact", |source| {
        source.acceptance_gate.verdict = source.acceptance_gate.verdict.to_ascii_lowercase();
        Ok(())
    });
    typed_rehash!(
        "a receipt must bind a candidate row",
        "non-candidate",
        |source| {
            let target = source
                .defects
                .iter()
                .find(|defect| {
                    defect.defect_disposition == "open-defect"
                        && source
                            .defects
                            .iter()
                            .filter(|other| other.defect_id == defect.defect_id)
                            .count()
                            == 1
                })
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: no singleton open defect"))?;
            let claim = source
                .claims
                .iter()
                .find(|claim| claim.id == target.affected_claim_ref)
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: target claim missing"))?;
            let receipt = &mut source.receipts[0];
            receipt.defect_row_ref = target.id;
            receipt.defect_id = target.defect_id;
            receipt.affected_claim_ref = target.affected_claim_ref;
            receipt.consequence_id = target.consequence_id;
            receipt.defect_disposition = target.defect_disposition;
            receipt.response_stage = target.response_stage;
            receipt.scope_id = target.scope_id;
            receipt.source_version = target.source_version;
            receipt.envelope_id = target.envelope_id;
            receipt.claim_posture = claim.posture.clone();
            receipt.route_ref = claim.route_ref;
            receipt.assurance_ceiling = claim.posture;
            receipt.residuals = vec!["none beyond the affected claim's own scope bound".into()];
            Ok(())
        }
    );
    typed!("elimination keeps its reintroduction control", |source| {
        let defect = source
            .defects
            .iter_mut()
            .find(|defect| defect.defect_disposition == "eliminated-structurally")
            .ok_or_else(|| LedgerError::new("control setup: no eliminated defect"))?;
        defect.controls = DefectControls {
            reintroduction_control_ref: None,
            initiation_control_ref: None,
            containment_control_refs: None,
            recovery_fields: None,
        };
        Ok(())
    });
    typed!("remedied cannot sit below its required stage", |source| {
        source
            .defects
            .iter_mut()
            .find(|defect| defect.defect_disposition == "eliminated-structurally")
            .ok_or_else(|| LedgerError::new("control setup: no eliminated defect"))?
            .defect_disposition = "remedied".into();
        Ok(())
    });
    shape!("a receipt needs its reader-facing mapping", |value| {
        let receipt = value_array_mut(value, "receipts")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no receipt"))?;
        value_object_mut(receipt, "receipts[0]")?.remove("reader_mapping_ref");
        Ok(())
    });
    typed!("one keyed row per defect tuple", |source| {
        let mut twin = source.defects[0].clone();
        twin.id = "FS-DFT-998".into();
        source.defects.push(twin);
        Ok(())
    });
    shape!("hand-authored blocking is refused", |value| {
        let defect = value_array_mut(value, "defects")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no defect"))?;
        value_object_mut(defect, "defects[0]")?.insert("blocking".into(), false.into());
        Ok(())
    });
    typed_rehash!("a receipt must name an existing defect row", |source| {
        source.receipts[0].defect_row_ref = "FS-DFT-777".into();
        Ok(())
    });
    typed!("an unknown residual citation is stale", |source| {
        source.defects[0]
            .residual_citations
            .push("bogus-file#nope".into());
        Ok(())
    });
    typed!(
        "an uncovered sibling residual fails closure",
        "uncovered",
        |source| {
            let mut counts = HashMap::<String, usize>::new();
            for token in source
                .defects
                .iter()
                .flat_map(|defect| defect.residual_citations.iter())
            {
                *counts.entry(token.clone()).or_default() += 1;
            }
            for defect in &mut source.defects {
                if let Some(index) = defect
                    .residual_citations
                    .iter()
                    .position(|token| counts.get(token) == Some(&1))
                {
                    defect.residual_citations.remove(index);
                    return Ok(());
                }
            }
            Err(LedgerError::new("control setup: no singly cited residual"))
        }
    );
    typed!("a stale exclusion is refused", |source| {
        source.residual_coverage_exclusions.push(ResidualExclusion {
            source_file: "x".into(),
            token: "bogus#token".into(),
            reason: "control".into(),
        });
        Ok(())
    });
    typed!("a defect must affect a claim record", |source| {
        source.defects[0].affected_claim_ref = source.bodies[0].id.clone();
        Ok(())
    });
    typed!("a defect layer is never the domain sentinel", |source| {
        source.defects[0].layer = "spans-all-layers".into();
        Ok(())
    });
    shape!("a defect declares gate applicability", |value| {
        let defect = value_array_mut(value, "defects")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no defect"))?;
        value_object_mut(defect, "defects[0]")?.remove("applicable_gate_refs");
        Ok(())
    });
    typed!("gate applicability is non-empty", |source| {
        source.defects[0].applicable_gate_refs.clear();
        Ok(())
    });
    typed!("gate applicability rejects unknown gates", |source| {
        source.defects[0].applicable_gate_refs = vec!["gate-z".into()];
        Ok(())
    });
    typed!("gate applicability rejects duplicates", |source| {
        source.defects[0].applicable_gate_refs = vec!["gate-a".into(), "gate-a".into()];
        Ok(())
    });
    typed!("gate applicability follows canonical order", |source| {
        source.defects[0].applicable_gate_refs = vec!["gate-b".into(), "gate-a".into()];
        Ok(())
    });
    typed!(
        "gate applicability cannot silently hide or widen a defect",
        |source| {
            source
                .defects
                .iter_mut()
                .find(|defect| defect.id == "FS-DFT-16")
                .ok_or_else(|| LedgerError::new("control setup: FS-DFT-16 missing"))?
                .applicable_gate_refs = GATE_REFS.iter().map(|gate| (*gate).into()).collect();
            Ok(())
        }
    );

    Ok(passed)
}

fn negative_controls_envelope_roles_bodies(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, true, None, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! closure {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, true, true, None, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! shape {
        ($name:literal, $mutate:expr) => {{
            run_shape_negative_control(ledger, $name, None, false, $mutate)?;
            passed += 1;
        }};
    }

    typed!("the rubric status is exact in both states", |source| {
        source.severity_rubric.rubric_status = "confirmed".into();
        Ok(())
    });
    typed!("a confirmed rubric records its basis", |source| {
        source.severity_rubric.rubric_status = RUBRIC_STATUS_CONFIRMED.into();
        source.severity_rubric.confirmation_basis = None;
        Ok(())
    });
    shape!("review_protocol must be present", |value| {
        value_object_mut(value, "root")?.remove("review_protocol");
        Ok(())
    });
    typed!("the amended protocol status is exact", |source| {
        source.review_protocol.protocol_status = "confirmed".into();
        Ok(())
    });
    shape!(
        "the amended protocol binds its mechanical policy",
        |value| {
            let protocol = value_object_mut(value, "root")?
                .get_mut("review_protocol")
                .ok_or_else(|| LedgerError::new("control setup: review_protocol missing"))?;
            value_object_mut(protocol, "review_protocol")?.remove("policy_basis");
            Ok(())
        }
    );
    typed!("the protocol mode is repository-adversarial", |source| {
        source.review_protocol.mode = "panel-review".into();
        Ok(())
    });
    typed!("external review is explicitly optional", |source| {
        source.review_protocol.external_review_policy = "required".into();
        Ok(())
    });
    typed!(
        "the legacy designation is retired as a gate dependency",
        |source| {
            source.review_protocol.designation.designation_status = "active".into();
            Ok(())
        }
    );
    typed!(
        "optional-review owner and checker remain distinct people",
        |source| {
            source.review_protocol.designation.independent_checker =
                source.review_protocol.designation.severity_owner.clone();
            Ok(())
        }
    );
    typed!(
        "an optional-review custodian may not triage or check",
        |source| {
            source.review_protocol.designation.severity_owner =
                source.review_protocol.designation.custodian.clone();
            Ok(())
        }
    );
    typed!("the protocol status line is live-checked", |source| {
        source.review_protocol.status_line_ref =
            "new-book-plans/full-society-scope-review-protocol.md::# Full-Society Scope-Review Protocol".into();
        Ok(())
    });
    typed!(
        "a calibrated envelope is refused in this contract",
        |source| {
            source.envelope[1].envelope_status = "calibrated".into();
            Ok(())
        }
    );
    typed!(
        "an established invariant may not depend on an envelope field",
        |source| {
            source.envelope[1].fields.as_mut().ok_or_else(|| {
                LedgerError::new("control setup: structural envelope has no fields")
            })?[0]
                .dependents
                .push("FS-CLM-01".into());
            Ok(())
        }
    );
    typed!(
        "envelope-relative claims must appear as dependents",
        |source| {
            for field in source.envelope[1].fields.as_mut().ok_or_else(|| {
                LedgerError::new("control setup: structural envelope has no fields")
            })? {
                field
                    .dependents
                    .retain(|reference| reference != "FS-CLM-06");
            }
            Ok(())
        }
    );
    typed!("a defect's envelope must exist", |source| {
        source.defects[0].envelope_id = "FS-ENV-77".into();
        Ok(())
    });
    typed!(
        "a structure-only envelope cannot carry operational assurance",
        |source| {
            let mut row = source.defects[0].clone();
            row.id = "FS-DFT-997".into();
            row.defect_id = "FS-DFT-997".into();
            row.defect_disposition = "remedied".into();
            row.response_stage = "operationally-assured-in-envelope".into();
            row.envelope_id = "FS-ENV-01".into();
            row.consequence_id = "control".into();
            row.scope_id = "control".into();
            row.controls = DefectControls {
                reintroduction_control_ref: None,
                initiation_control_ref: None,
                containment_control_refs: None,
                recovery_fields: None,
            };
            row.applicable_gate_refs = vec!["gate-a".into()];
            source.defects.push(row);
            Ok(())
        }
    );
    closure!(
        "a closure record's envelope must be the structural envelope",
        |source| {
            source
                .closure_record
                .0
                .as_mut()
                .ok_or_else(|| LedgerError::new("control setup: closure record missing"))?
                .envelope_ref = "FS-ENV-77".into();
            Ok(())
        }
    );
    typed!(
        "an envelope field states dependents or invariance",
        |source| {
            let field = &mut source.envelope[1].fields.as_mut().ok_or_else(|| {
                LedgerError::new("control setup: structural envelope has no fields")
            })?[0];
            field.dependents.clear();
            field.invariance.clear();
            Ok(())
        }
    );
    typed!("a threshold's lawful source is closed", |source| {
        source.thresholds[0].lawful_source = "vibes".into();
        Ok(())
    });
    shape!("a criterion carries its provenance", |value| {
        let root = value_object_mut(value, "root")?;
        let criterion = root
            .get_mut("functional_criteria")
            .and_then(Value::as_object_mut)
            .and_then(|criteria| criteria.get_mut("criteria"))
            .and_then(Value::as_array_mut)
            .and_then(|criteria| criteria.first_mut())
            .ok_or_else(|| LedgerError::new("control setup: no functional criterion"))?;
        value_object_mut(criterion, "functional criterion")?.remove("provenance");
        Ok(())
    });
    shape!(
        "a value-bearing key is refused on an envelope field",
        |value| {
            let root = value_object_mut(value, "root")?;
            let field = root
                .get_mut("envelope")
                .and_then(Value::as_array_mut)
                .and_then(|envelopes| envelopes.get_mut(1))
                .and_then(Value::as_object_mut)
                .and_then(|envelope| envelope.get_mut("fields"))
                .and_then(Value::as_array_mut)
                .and_then(|fields| fields.first_mut())
                .ok_or_else(|| LedgerError::new("control setup: no envelope field"))?;
            value_object_mut(field, "envelope field")?.insert("value".into(), "ten".into());
            Ok(())
        }
    );
    typed!("no numeric value in a Book 1 threshold", |source| {
        source.thresholds[0].definition.push_str(" 42");
        Ok(())
    });
    typed!("the criteria canon is the seven-member union", |source| {
        source.functional_criteria.criteria.remove(0);
        Ok(())
    });
    typed!("a populated record type sheds its deferral", |source| {
        source.deferred_populations.push(DeferredPopulation {
            record_type: "roles".into(),
            owner_ref: "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                .into(),
            closure_condition: "control".into(),
            stage: "stage-3".into(),
        });
        Ok(())
    });
    typed!("a role's domain ref must resolve", |source| {
        source.roles[0].domain_refs = vec!["FS-DOM-99".into()];
        Ok(())
    });
    typed!("a role may not cite a non-domain as its domain", |source| {
        source.roles[0].domain_refs = vec![source.bodies[0].id.clone()];
        Ok(())
    });
    typed!(
        "each material domain keeps a reviewed role citation",
        |source| {
            let mut touched = false;
            for role in &mut source.roles {
                if role
                    .domain_refs
                    .iter()
                    .any(|reference| reference == "FS-DOM-12")
                {
                    role.domain_refs
                        .retain(|reference| reference != "FS-DOM-12");
                    touched = true;
                }
                if role.domain_refs.is_empty() {
                    role.domain_refs.push("FS-DOM-01".into());
                }
            }
            if !touched {
                return Err(LedgerError::new("control setup: no role cites FS-DOM-12"));
            }
            Ok(())
        }
    );
    typed!("every named scale is exercised", |source| {
        let mut touched = false;
        for role in &mut source.roles {
            if role.scales.iter().any(|scale| scale == "intergenerational") {
                role.scales.retain(|scale| scale != "intergenerational");
                touched = true;
            }
            if role.scales.is_empty() {
                role.scales.push("individual".into());
            }
        }
        if !touched {
            return Err(LedgerError::new("control setup: scale is not exercised"));
        }
        Ok(())
    });
    typed!("a required body keeps both positions", |source| {
        let body_id = source.bodies[0].id.clone();
        let mut touched = false;
        for role in &mut source.roles {
            let before = role.power_positions.len();
            role.power_positions
                .retain(|position| position.body_ref != body_id);
            touched |= before != role.power_positions.len();
        }
        if !touched {
            return Err(LedgerError::new("control setup: no role cites first body"));
        }
        Ok(())
    });
    shape!("a body separates all seven status senses", |value| {
        let root = value_object_mut(value, "root")?;
        let senses = root
            .get_mut("bodies")
            .and_then(Value::as_array_mut)
            .and_then(|bodies| bodies.first_mut())
            .and_then(Value::as_object_mut)
            .and_then(|body| body.get_mut("status_senses"))
            .ok_or_else(|| LedgerError::new("control setup: no body senses"))?;
        value_object_mut(senses, "status_senses")?.remove("franchise");
        Ok(())
    });
    shape!("a body carries the whole office contract", |value| {
        let root = value_object_mut(value, "root")?;
        let office = root
            .get_mut("bodies")
            .and_then(Value::as_array_mut)
            .and_then(|bodies| bodies.first_mut())
            .and_then(Value::as_object_mut)
            .and_then(|body| body.get_mut("office_contract"))
            .ok_or_else(|| LedgerError::new("control setup: no office contract"))?;
        value_object_mut(office, "office_contract")?.remove("succession");
        Ok(())
    });
    typed!("a body kind is a declared kind", |source| {
        source.bodies[0].body_kind = "ministry".into();
        Ok(())
    });
    typed!(
        "a body's rendered source is one of its card sources",
        |source| {
            let body = source
                .bodies
                .iter_mut()
                .find(|body| body.source_refs.len() > 1)
                .ok_or_else(|| LedgerError::new("control setup: no multi-source body"))?;
            body.source_refs
                .retain(|reference| reference != &body.source_ref);
            Ok(())
        }
    );
    typed!("a body term source must be a card source", |source| {
        source.bodies[0]
            .status_senses
            .universal_human_standing
            .source_refs
            .push("new-book-plans/full-society-scope-review-protocol.md::## 5. Mechanical Gate A closure".into());
        Ok(())
    });
    typed!(
        "the office senses do not reuse the word standing",
        |source| {
            source.bodies[0]
                .status_senses
                .current_office
                .text
                .push_str(" This is the body's standing.");
            Ok(())
        }
    );
    typed!(
        "the ordinary function expands the job rather than copying it",
        |source| {
            source.bodies[0].office_contract.ordinary_function.text = source.bodies[0].job.clone();
            Ok(())
        }
    );
    typed!("a body may not check itself", |source| {
        source.bodies[0].accountability_routes[0].checker_body_refs =
            vec![source.bodies[0].id.clone()];
        Ok(())
    });
    typed!("a body names at least one external checker", |source| {
        for route in &mut source.bodies[0].accountability_routes {
            route.checker_body_refs.clear();
        }
        Ok(())
    });
    typed!(
        "a body's accountability routes are typed and duplicate-free",
        |source| {
            let duplicate = source.bodies[0].accountability_routes[0].clone();
            source.bodies[0].accountability_routes.push(duplicate);
            Ok(())
        }
    );
    shape!(
        "an enumerated adverse determination carries an appeal",
        |value| {
            let root = value_object_mut(value, "root")?;
            let item = root
                .get_mut("bodies")
                .and_then(Value::as_array_mut)
                .and_then(|bodies| {
                    bodies.iter_mut().find(|body| {
                        body.get("adverse_determinations")
                            .and_then(|value| value.get("kind"))
                            .and_then(Value::as_str)
                            == Some("enumerated")
                    })
                })
                .and_then(Value::as_object_mut)
                .and_then(|body| body.get_mut("adverse_determinations"))
                .and_then(Value::as_object_mut)
                .and_then(|adverse| adverse.get_mut("items"))
                .and_then(Value::as_array_mut)
                .and_then(|items| items.first_mut())
                .ok_or_else(|| LedgerError::new("control setup: no adverse item"))?;
            value_object_mut(item, "adverse item")?.remove("appeal");
            Ok(())
        }
    );
    typed!(
        "an enumerated adverse determination carries a remedy",
        |source| {
            source
                .bodies
                .iter_mut()
                .find(|body| body.adverse_determinations.kind == "enumerated")
                .ok_or_else(|| LedgerError::new("control setup: no enumerated body"))?
                .adverse_determinations
                .items[0]
                .remedy
                .text
                .clear();
            Ok(())
        }
    );
    typed!(
        "a body claiming no adverse determination lists none",
        |source| {
            let donor = source
                .bodies
                .iter()
                .find(|body| body.adverse_determinations.kind == "enumerated")
                .ok_or_else(|| LedgerError::new("control setup: no enumerated body"))?
                .adverse_determinations
                .items[0]
                .clone();
            source
                .bodies
                .iter_mut()
                .find(|body| body.adverse_determinations.kind == "none-by-design")
                .ok_or_else(|| LedgerError::new("control setup: no none-by-design body"))?
                .adverse_determinations
                .items = vec![donor];
            Ok(())
        }
    );
    typed!("an enumerated adverse determination names one", |source| {
        source
            .bodies
            .iter_mut()
            .find(|body| body.adverse_determinations.kind == "enumerated")
            .ok_or_else(|| LedgerError::new("control setup: no enumerated body"))?
            .adverse_determinations
            .items
            .clear();
        Ok(())
    });
    typed!(
        "only custodial execution applies the retained custody contract",
        |source| {
            source
                .bodies
                .iter_mut()
                .find(|body| body.id != CUSTODY_T3_APPLICANT)
                .ok_or_else(|| LedgerError::new("control setup: no non-custodial body"))?
                .temporal_contract
                .custody_t3_relation = "retained-application".into();
            Ok(())
        }
    );
    typed!(
        "a body temporal contract may not cite the custody clock source",
        |source| {
            let body = source
                .bodies
                .iter_mut()
                .find(|body| body.id != CUSTODY_T3_APPLICANT)
                .ok_or_else(|| LedgerError::new("control setup: no non-custodial body"))?;
            let reference =
                "new-book-plans/book-1-time-model-decision.md::# Book 1 Time-Model Decision"
                    .to_owned();
            body.source_refs.push(reference.clone());
            body.temporal_contract.term.source_refs.push(reference);
            Ok(())
        }
    );
    typed!("a blocked mechanic is filled", |source| {
        source
            .bodies
            .iter_mut()
            .find(|body| {
                [
                    "FS-BOD-02",
                    "FS-BOD-03",
                    "FS-BOD-04",
                    "FS-BOD-05",
                    "FS-BOD-17",
                    "FS-BOD-18",
                    "FS-BOD-19",
                    "FS-BOD-24",
                    "FS-BOD-25",
                ]
                .contains(&body.id.as_str())
            })
            .ok_or_else(|| LedgerError::new("control setup: no delegated body"))?
            .delegated_mechanics
            .clear();
        Ok(())
    });
    typed!(
        "a delegated mechanic declares its choice owner and bounds",
        |source| {
            source
                .bodies
                .iter_mut()
                .find(|body| !body.delegated_mechanics.is_empty())
                .ok_or_else(|| LedgerError::new("control setup: no delegated mechanic"))?
                .delegated_mechanics[0]
                .basis = "source-specified".into();
            Ok(())
        }
    );
    typed!("a body card may not assert an arrival", |source| {
        source.bodies[0]
            .status_senses
            .universal_human_standing
            .text
            .push_str(" The remedy is delivered.");
        Ok(())
    });
    typed!("a body card may not carry an aggregate figure", |source| {
        source.bodies[0]
            .status_senses
            .universal_human_standing
            .text
            .push_str(" 5 of 7 are established.");
        Ok(())
    });
    typed!("a body card may not carry a feasibility claim", |source| {
        source.bodies[0]
            .status_senses
            .universal_human_standing
            .text
            .push_str(" The design is feasible.");
        Ok(())
    });
    typed!("a body card may not relabel a current fixture", |source| {
        source.bodies[0]
            .status_senses
            .universal_human_standing
            .text
            .push_str(" Read Convocation as the Executive Council.");
        Ok(())
    });
    typed!(
        "the advocate declares its non-substitution boundary",
        |source| {
            source
                .bodies
                .iter_mut()
                .find(|body| body.id == "FS-BOD-20")
                .ok_or_else(|| LedgerError::new("control setup: FS-BOD-20 missing"))?
                .office_contract
                .delegation_boundary
                .text = " ".into();
            Ok(())
        }
    );
    typed!(
        "a body cell may not paraphrase the map row it cites",
        |source| {
            let body = source
                .bodies
                .iter_mut()
                .find(|body| body.id == "FS-BOD-05")
                .ok_or_else(|| LedgerError::new("control setup: FS-BOD-05 missing"))?;
            body.required_check = body
                .required_check
                .replace(" and cross-body confirmation", "");
            Ok(())
        }
    );
    typed!("every required-bodies row still binds a card", |source| {
        source
            .bodies
            .iter_mut()
            .find(|body| body.id == "FS-BOD-05")
            .ok_or_else(|| LedgerError::new("control setup: FS-BOD-05 missing"))?
            .title = "Ceremonial President".into();
        Ok(())
    });
    typed!("an omission carries its risk-based reason", |source| {
        let recorded = source
            .role_omissions
            .iter()
            .filter_map(|omission| match omission {
                RoleOmission::Scale(value) => {
                    Some((value.role_ref.clone(), value.omitted_scale.clone()))
                }
                _ => None,
            })
            .collect::<HashSet<_>>();
        for role in &source.roles {
            for scale in ROLE_SCALES {
                if !role.scales.iter().any(|actual| actual == scale)
                    && !recorded.contains(&(role.id.clone(), scale.into()))
                {
                    source
                        .role_omissions
                        .push(RoleOmission::Scale(OmittedScale {
                            role_ref: role.id.clone(),
                            omitted_scale: scale.into(),
                            risk_reason: String::new(),
                        }));
                    return Ok(());
                }
            }
        }
        Err(LedgerError::new(
            "control setup: no unrecorded role/scale omission",
        ))
    });
    typed!("an omission names a real role", |source| {
        source
            .role_omissions
            .push(RoleOmission::Scale(OmittedScale {
                role_ref: "FS-ROL-777".into(),
                omitted_scale: "individual".into(),
                risk_reason: "control".into(),
            }));
        Ok(())
    });
    typed!("a stale omission is refused", |source| {
        source
            .role_omissions
            .push(RoleOmission::Scale(OmittedScale {
                role_ref: source.roles[0].id.clone(),
                omitted_scale: source.roles[0].scales[0].clone(),
                risk_reason: "control".into(),
            }));
        Ok(())
    });
    typed!("a role's layer is universal standing", |source| {
        source.roles[0].layer = "book-2-operation".into();
        Ok(())
    });
    typed!("a role anchor is closed", |source| {
        source.roles[0].formal_anchor.anchor = "vibes".into();
        Ok(())
    });
    typed!(
        "a constitution-predicate anchor cites the constitution",
        |source| {
            let replacement = source.domains[0].source_refs[0].clone();
            source
                .roles
                .iter_mut()
                .find(|role| {
                    role.formal_anchor
                        .anchor
                        .starts_with("constitution-predicate")
                })
                .ok_or_else(|| LedgerError::new("control setup: no constitution-predicate role"))?
                .formal_anchor
                .refs = vec![replacement];
            Ok(())
        }
    );
    typed!("an unchecked private power is refused", |source| {
        source
            .roles
            .iter_mut()
            .find_map(|role| role.power_held.as_mut())
            .ok_or_else(|| LedgerError::new("control setup: no role holds a power"))?
            .checking_refs
            .clear();
        Ok(())
    });
    typed!("a private power's affected side names roles", |source| {
        let body = source.bodies[0].id.clone();
        source
            .roles
            .iter_mut()
            .find_map(|role| role.power_held.as_mut())
            .ok_or_else(|| LedgerError::new("control setup: no role holds a power"))?
            .affected_role_refs = vec![body];
        Ok(())
    });
    typed!("a duplicate role id is caught", |source| {
        source.roles.push(source.roles[0].clone());
        Ok(())
    });
    typed!("role meanings cannot drift", |source| {
        source.scale_meanings.remove("intergenerational");
        Ok(())
    });
    typed!(
        "role_omissions may not outlive a deferred roles array",
        |source| {
            source.roles.clear();
            Ok(())
        }
    );

    Ok(passed)
}

fn negative_controls_dependencies_and_scenarios(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, true, None, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! shape {
        ($name:literal, $mutate:expr) => {{
            run_shape_negative_control(ledger, $name, None, false, $mutate)?;
            passed += 1;
        }};
    }

    typed!("a dependency destination must resolve", |source| {
        source.dependencies[0].to_ref = "FS-DOM-99".into();
        Ok(())
    });
    typed!("a dependency endpoint type is closed", |source| {
        source.dependencies[0].from_ref = source.claims[0].id.clone();
        Ok(())
    });
    typed!(
        "an externally-assumed edge flows from a named assumption",
        |source| {
            source
                .dependencies
                .iter_mut()
                .find(|row| row.dependency_class == "externally-assumed")
                .ok_or_else(|| LedgerError::new("control setup: no externally-assumed edge"))?
                .from_ref = source.bodies[0].id.clone();
            Ok(())
        }
    );
    typed!(
        "an external assumption feeds only externally-assumed edges",
        |source| {
            let triples = source
                .dependencies
                .iter()
                .map(|row| {
                    (
                        row.from_ref.clone(),
                        row.to_ref.clone(),
                        row.flow_kind.clone(),
                    )
                })
                .collect::<HashSet<_>>();
            let row = source
                .dependencies
                .iter_mut()
                .find(|row| {
                    row.dependency_class != "externally-assumed"
                        && !triples.contains(&(
                            "FS-EXA-01".into(),
                            row.to_ref.clone(),
                            row.flow_kind.clone(),
                        ))
                })
                .ok_or_else(|| LedgerError::new("control setup: no collision-free non-EXA edge"))?;
            row.from_ref = "FS-EXA-01".into();
            Ok(())
        }
    );
    typed!("an edge's layer follows its class", |source| {
        let row = &mut source.dependencies[0];
        row.layer = if row.dependency_class == "operationally-supplied" {
            "constitutional-invariant"
        } else {
            "book-2-operation"
        }
        .into();
        Ok(())
    });
    typed!("each material domain joins the flow map", |source| {
        let dropped = source
            .dependencies
            .iter()
            .filter(|row| row.from_ref == "FS-DOM-12" || row.to_ref == "FS-DOM-12")
            .map(|row| row.id.clone())
            .collect::<HashSet<_>>();
        if dropped.is_empty() {
            return Err(LedgerError::new("control setup: no edge touches FS-DOM-12"));
        }
        source.dependencies.retain(|row| !dropped.contains(&row.id));
        source.dependency_loops.retain(|row| {
            !row.member_edge_refs
                .iter()
                .any(|reference| dropped.contains(reference))
        });
        if source.dependency_loops.is_empty() {
            return Err(LedgerError::new(
                "control setup: dropping FS-DOM-12 emptied loops",
            ));
        }
        Ok(())
    });
    typed!("every flow kind is exercised", |source| {
        let triples = source
            .dependencies
            .iter()
            .map(|row| {
                (
                    row.from_ref.clone(),
                    row.to_ref.clone(),
                    row.flow_kind.clone(),
                )
            })
            .collect::<HashSet<_>>();
        for kind in FLOW_KINDS {
            let movers = source
                .dependencies
                .iter()
                .filter(|row| row.flow_kind == kind)
                .map(|row| row.id.clone())
                .collect::<Vec<_>>();
            if movers.is_empty() {
                continue;
            }
            for target in FLOW_KINDS {
                if target == kind
                    || movers.iter().any(|id| {
                        let row = source
                            .dependencies
                            .iter()
                            .find(|candidate| candidate.id == *id)
                            .expect("selected edge exists");
                        triples.contains(&(row.from_ref.clone(), row.to_ref.clone(), target.into()))
                    })
                {
                    continue;
                }
                for row in &mut source.dependencies {
                    if movers.contains(&row.id) {
                        row.flow_kind = target.into();
                    }
                }
                return Ok(());
            }
        }
        Err(LedgerError::new("control setup: no collision-free relabel"))
    });
    typed!("every external assumption stays cited", |source| {
        let triples = source
            .dependencies
            .iter()
            .map(|row| {
                (
                    row.from_ref.clone(),
                    row.to_ref.clone(),
                    row.flow_kind.clone(),
                )
            })
            .collect::<HashSet<_>>();
        let movers = source
            .dependencies
            .iter()
            .filter(|row| row.from_ref == "FS-EXA-01")
            .map(|row| row.id.clone())
            .collect::<Vec<_>>();
        if movers.is_empty() {
            return Err(LedgerError::new(
                "control setup: no edge sourced at FS-EXA-01",
            ));
        }
        for target in ["FS-EXA-02", "FS-EXA-03", "FS-EXA-04"] {
            if movers.iter().any(|id| {
                let row = source
                    .dependencies
                    .iter()
                    .find(|candidate| candidate.id == *id)
                    .expect("selected edge exists");
                triples.contains(&(target.into(), row.to_ref.clone(), row.flow_kind.clone()))
            }) {
                continue;
            }
            for row in &mut source.dependencies {
                if movers.contains(&row.id) {
                    row.from_ref = target.into();
                }
            }
            return Ok(());
        }
        Err(LedgerError::new(
            "control setup: no collision-free EXA retarget",
        ))
    });
    typed!("a cycle needs a declared loop witness", |source| {
        let used = source
            .dependencies
            .iter()
            .flat_map(|row| [row.from_ref.as_str(), row.to_ref.as_str()])
            .collect::<HashSet<_>>();
        let free = source
            .roles
            .iter()
            .filter(|row| !used.contains(row.id.as_str()))
            .map(|row| row.id.clone())
            .collect::<Vec<_>>();
        if free.len() < 2 {
            return Err(LedgerError::new("control setup: no free endpoint pair"));
        }
        for (index, (from_ref, to_ref)) in [
            (free[0].clone(), free[1].clone()),
            (free[1].clone(), free[0].clone()),
        ]
        .into_iter()
        .enumerate()
        {
            let mut twin = source.dependencies[0].clone();
            twin.id = format!("FS-DEP-90{}", index + 1);
            twin.from_ref = from_ref;
            twin.to_ref = to_ref;
            twin.flow_kind = "services".into();
            twin.dependency_class = "operationally-supplied".into();
            twin.layer = "book-2-operation".into();
            twin.lifecycle_path = "outside-ratified-paths".into();
            twin.structural_satisfiability = StructuralSatisfiability {
                satisfiability_status: "operation-deferred".into(),
                defect_refs: Vec::new(),
                reason: "synthetic control edge".into(),
            };
            source.dependencies.push(twin);
        }
        Ok(())
    });
    typed!("a declared loop is a real cycle", |source| {
        for left in &source.dependencies {
            for right in &source.dependencies {
                if left.id != right.id
                    && left.to_ref != right.from_ref
                    && right.to_ref != left.from_ref
                {
                    source.dependency_loops[0].member_edge_refs =
                        vec![left.id.clone(), right.id.clone()];
                    return Ok(());
                }
            }
        }
        Err(LedgerError::new("control setup: no non-chaining edge pair"))
    });
    typed!("a loop member is a real edge", |source| {
        source.dependency_loops[0].member_edge_refs[0] = "FS-DEP-777".into();
        Ok(())
    });
    shape!("a refused flow cites its wall", |value| {
        let row = value_array_mut(value, "refused_flows")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no refused flow"))?;
        value_object_mut(row, "refused_flows[0]")?.remove("source_ref");
        Ok(())
    });
    typed!("a duplicate edge is one edge", |source| {
        let mut twin = source.dependencies[0].clone();
        twin.id = "FS-DEP-903".into();
        source.dependencies.push(twin);
        Ok(())
    });
    typed!("a duplicate refusal is caught", |source| {
        source.refused_flows.push(source.refused_flows[0].clone());
        Ok(())
    });
    typed!("a populated dependencies sheds its deferral", |source| {
        source.deferred_populations.push(DeferredPopulation {
            record_type: "dependencies".into(),
            owner_ref: "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                .into(),
            closure_condition: "control".into(),
            stage: "stage-3".into(),
        });
        Ok(())
    });
    typed!("loops and walls may not outlive a deferred map", |source| {
        source.dependencies.clear();
        Ok(())
    });
    typed!("dependency meanings cannot drift", |source| {
        source.flow_kind_meanings.remove("care");
        Ok(())
    });
    typed!(
        "a lifecycle path is ratified or recorded outside",
        |source| {
            source.dependencies[0].lifecycle_path = "delivery".into();
            Ok(())
        }
    );
    shape!("an absent alternate is recorded, never silent", |value| {
        let row = value_array_mut(value, "dependencies")?
            .first_mut()
            .ok_or_else(|| LedgerError::new("control setup: no dependency"))?;
        value_object_mut(row, "dependencies[0]")?
            .insert("alternate_route".into(), Value::Object(Default::default()));
        Ok(())
    });
    typed!("an edge may not feed itself", |source| {
        let row = source
            .dependencies
            .iter_mut()
            .find(|row| row.dependency_class != "externally-assumed")
            .ok_or_else(|| LedgerError::new("control setup: every edge is externally assumed"))?;
        row.to_ref = row.from_ref.clone();
        Ok(())
    });
    typed!("a populated scenarios sheds its deferral", |source| {
        source.deferred_populations.push(DeferredPopulation {
            record_type: "scenarios".into(),
            owner_ref: "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                .into(),
            closure_condition: "control".into(),
            stage: "stage-3".into(),
        });
        Ok(())
    });
    typed!("omissions may not outlive a deferred catalogue", |source| {
        source.scenarios.clear();
        Ok(())
    });
    typed!(
        "a populated catalogue flips every domain's applicability",
        |source| {
            source.domains[0].scenario_applicability =
                ScenarioApplicability::Deferred(ScenarioDeferredApplicability {
                    deferred_ref:
                        "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                            .into(),
                });
            Ok(())
        }
    );
    typed!("each domain keeps a whole-society scenario", |source| {
        let mut touched = false;
        for row in &mut source.scenarios {
            if row
                .domain_refs
                .iter()
                .any(|reference| reference == "FS-DOM-03")
            {
                row.domain_refs.retain(|reference| reference != "FS-DOM-03");
                touched = true;
                if row.domain_refs.is_empty() {
                    row.domain_refs.push("FS-DOM-01".into());
                }
            }
        }
        if !touched {
            return Err(LedgerError::new(
                "control setup: no scenario cites FS-DOM-03",
            ));
        }
        Ok(())
    });
    typed!("every scenario kind is exercised", |source| {
        let mut touched = false;
        for row in &mut source.scenarios {
            if row.scenario_kind == "stress" {
                row.scenario_kind = "journey".into();
                touched = true;
            }
        }
        if !touched {
            return Err(LedgerError::new("control setup: no stress scenario"));
        }
        Ok(())
    });
    typed!("every collision axis is tested", |source| {
        let mut touched = false;
        for row in &mut source.scenarios {
            if row.collision_axis.as_deref() == Some(COLLISION_AXES[0]) {
                row.collision_axis = Some(COLLISION_AXES[1].into());
                touched = true;
            }
        }
        if !touched {
            return Err(LedgerError::new(
                "control setup: first collision axis is unused",
            ));
        }
        Ok(())
    });
    typed!("every named shock is carried", |source| {
        let mut touched = false;
        for row in &mut source.scenarios {
            if row.shock_kind.as_deref() == Some(SHOCK_KINDS[0]) {
                row.shock_kind = Some(SHOCK_KINDS[1].into());
                touched = true;
            }
        }
        if !touched {
            return Err(LedgerError::new(
                "control setup: first shock kind is unused",
            ));
        }
        Ok(())
    });
    typed!("every protected-sphere test is exercised", |source| {
        let gone = PROTECTED_SPHERE_FORMS[0];
        let fill = PROTECTED_SPHERE_FORMS[3];
        let mut touched = false;
        for row in &mut source.scenarios {
            if let Some(forms) = &mut row.protected_sphere_forms
                && forms.iter().any(|form| form == gone)
            {
                forms.retain(|form| form != gone);
                touched = true;
                if forms.is_empty() {
                    forms.push(fill.into());
                }
            }
        }
        if !touched {
            return Err(LedgerError::new(
                "control setup: first protected form is unused",
            ));
        }
        Ok(())
    });
    typed!(
        "every critical edge is stressed or recorded omitted",
        |source| {
            let cited = source
                .scenarios
                .iter()
                .flat_map(|row| row.dependency_refs.iter().map(String::as_str))
                .collect::<HashSet<_>>();
            let target = source
                .dependencies
                .iter()
                .find(|row| row.severity == "critical" && cited.contains(row.id.as_str()))
                .map(|row| row.id.clone())
                .ok_or_else(|| LedgerError::new("control setup: no cited critical edge"))?;
            for row in &mut source.scenarios {
                row.dependency_refs.retain(|reference| reference != &target);
            }
            Ok(())
        }
    );
    typed!("a collision axis belongs only on a collision", |source| {
        let row = source
            .scenarios
            .iter_mut()
            .find(|row| !matches!(row.scenario_kind.as_str(), "collision" | "compound-shock"))
            .ok_or_else(|| LedgerError::new("control setup: no non-collision scenario"))?;
        row.collision_axis = Some(COLLISION_AXES[0].into());
        Ok(())
    });
    typed!("a collision scenario names its axis", |source| {
        source
            .scenarios
            .iter_mut()
            .find(|row| row.scenario_kind == "collision")
            .ok_or_else(|| LedgerError::new("control setup: no collision scenario"))?
            .collision_axis = None;
        Ok(())
    });
    typed!("a shock kind is closed", |source| {
        source
            .scenarios
            .iter_mut()
            .find(|row| row.scenario_kind == "compound-shock")
            .ok_or_else(|| LedgerError::new("control setup: no compound-shock scenario"))?
            .shock_kind = Some("asteroid".into());
        Ok(())
    });
    typed!("a scenario's dependency ref must resolve", |source| {
        source.scenarios[0].dependency_refs = vec![source.claims[0].id.clone()];
        Ok(())
    });
    typed!("a scenario's layer states Book 1 behaviour", |source| {
        source.scenarios[0].layer = "book-2-operation".into();
        Ok(())
    });
    typed!(
        "a scenario's status is the exact inventory literal",
        |source| {
            source.scenarios[0].status = "reviewed-routing".into();
            Ok(())
        }
    );
    typed!("scenario meanings cannot drift", |source| {
        source.collision_axis_meanings.remove("property-vs-floor");
        Ok(())
    });
    typed!("a stale scenario omission is refused", |source| {
        let reference = source
            .scenarios
            .iter()
            .find_map(|row| row.dependency_refs.first())
            .cloned()
            .ok_or_else(|| LedgerError::new("control setup: no scenario cites an edge"))?;
        source
            .scenario_omissions
            .push(ScenarioOmission::Dependency(OmittedDependency {
                omitted_dependency_ref: reference,
                risk_reason: "control".into(),
                source_ref:
                    "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                        .into(),
            }));
        Ok(())
    });
    typed!("a bounded witness is a real sibling case", |source| {
        source.scenarios[0].bounded_witness_refs =
            Some(vec!["record-integrity-red-team#RS-99".into()]);
        Ok(())
    });
    typed!("a duplicate scenario id is one scenario", |source| {
        source.scenarios.push(source.scenarios[0].clone());
        Ok(())
    });
    typed!("an omission needs a risk-based reason", |source| {
        match &mut source.scenario_omissions[0] {
            ScenarioOmission::Scenario(value) => value.risk_reason.clear(),
            ScenarioOmission::Dependency(value) => value.risk_reason.clear(),
        }
        Ok(())
    });
    typed!("a book passage may never support a scenario", |source| {
        source.scenarios[0].source_refs = vec!["book-1/rights-floor.md::the floor".into()];
        Ok(())
    });

    Ok(passed)
}

fn negative_controls_power_and_effects(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, true, None, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! shape {
        ($name:literal, $mutate:expr) => {{
            run_shape_negative_control(ledger, $name, None, false, $mutate)?;
            passed += 1;
        }};
    }

    shape!("power source inventory binding is required", |value| {
        value_object_mut(value, "root")?.remove("power_source_inventory");
        Ok(())
    });
    typed!("power source inventory digest is exact", |source| {
        source.power_source_inventory.artifact_sha256 = "0".repeat(64);
        Ok(())
    });
    typed!("power source inventory row count is exact", |source| {
        source.power_source_inventory.row_count = 236;
        Ok(())
    });
    typed!(
        "known power allocation gaps cannot disappear silently",
        |source| {
            source.power_source_inventory.known_allocation_gaps.pop();
            Ok(())
        }
    );
    typed!("power final counts are checker-owned", |source| {
        source.power_population.expected_final_counts.powers = 211;
        Ok(())
    });
    typed!("power family completion is an exact prefix", |source| {
        source.power_population.completed_source_families = vec!["time-model".into()];
        source.power_population.status = "partial".into();
        Ok(())
    });
    typed!(
        "resolved power-allocation gaps are append-only and exact",
        |source| {
            source.power_population.resolved_allocation_gaps.pop();
            Ok(())
        }
    );
    typed!(
        "complete power population cannot regain its deferral",
        |source| {
            source.deferred_populations.push(DeferredPopulation {
                record_type: "powers".into(),
                owner_ref: source.power_source_inventory.owner_ref.clone(),
                closure_condition: source.power_source_inventory.closure_condition.clone(),
                stage: "stage-3".into(),
            });
            Ok(())
        }
    );
    typed!("a power grain cannot be bundled or duplicated", |source| {
        source.powers[0].manifest_key = source.powers[1].manifest_key.clone();
        Ok(())
    });
    typed!("a power profile cannot be dropped", |source| {
        source.powers[0].profiles.pop();
        Ok(())
    });
    typed!("profile fields reject blank substitutes", |source| {
        let profile = source.powers[0]
            .profiles
            .first()
            .cloned()
            .ok_or_else(|| LedgerError::new("control setup: first power has no profile"))?;
        let terms = source.powers[0]
            .profile_terms
            .get_mut(&profile)
            .ok_or_else(|| LedgerError::new("control setup: first power profile has no terms"))?;
        terms
            .values_mut()
            .next()
            .ok_or_else(|| LedgerError::new("control setup: first power profile is empty"))?
            .text = "N/A".into();
        Ok(())
    });
    typed!(
        "coverage completion is an exact source-family prefix",
        |source| {
            source.coverage_population.completed_source_families[0] = "time-model".into();
            Ok(())
        }
    );
    typed!("complete coverage cannot regain its deferral", |source| {
        source.deferred_populations.push(DeferredPopulation {
            record_type: "coverage-contracts".into(),
            owner_ref: "new-book-plans/full-society-boundary-decision.md::## 4. Versioned closure"
                .into(),
            closure_condition: "control".into(),
            stage: "control".into(),
        });
        Ok(())
    });
    typed!("contract prose cannot be repeated across cards", |source| {
        let text = source.powers[0]
            .contract_terms
            .get("lawful_source")
            .ok_or_else(|| LedgerError::new("control setup: lawful_source term missing"))?
            .text
            .clone();
        source.powers[1]
            .contract_terms
            .get_mut("lawful_source")
            .ok_or_else(|| LedgerError::new("control setup: second lawful_source term missing"))?
            .text = text;
        Ok(())
    });
    typed!("every contract term keeps a source", |source| {
        source.powers[0]
            .contract_terms
            .get_mut("lawful_source")
            .ok_or_else(|| LedgerError::new("control setup: lawful_source term missing"))?
            .source_refs
            .clear();
        Ok(())
    });
    typed!("constitutional-effect count is checker-owned", |source| {
        source.constitutional_effects.pop();
        Ok(())
    });
    typed!(
        "constitutional-effect taxonomy is checker-owned",
        |source| {
            source.constitutional_effects[0].primary_class_ref = "class-02".into();
            Ok(())
        }
    );
    typed!(
        "constitutional-effect forbidden gates are explicit",
        |source| {
            source.constitutional_effects[0].prohibited_inputs[0] = "other limits".into();
            Ok(())
        }
    );
    typed!(
        "coverage family partitions every effect exactly once",
        |source| {
            source
                .coverage_families
                .last_mut()
                .ok_or_else(|| LedgerError::new("control setup: no coverage family"))?
                .effect_refs
                .pop();
            Ok(())
        }
    );
    typed!(
        "universal-standing formal surface is checker-owned",
        |source| {
            source
                .coverage_families
                .iter_mut()
                .find(|row| row.id == "FS-CVF-011")
                .ok_or_else(|| LedgerError::new("control setup: FS-CVF-011 missing"))?
                .formal_statement_refs
                .reverse();
            Ok(())
        }
    );
    typed!(
        "material-floor inventory cannot become an environmental floor",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "material-floor-inventory")
                .ok_or_else(|| LedgerError::new("control setup: material floor effect missing"))?
                .prohibited_inputs[0] = "Other boundaries only".into();
            Ok(())
        }
    );
    typed!(
        "equality effects cannot lose their closed profile",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "direct-discrimination")
                .ok_or_else(|| LedgerError::new("control setup: direct-discrimination missing"))?
                .profiles
                .pop();
            Ok(())
        }
    );
    typed!("equality effects cannot import person scoring", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.effect_key == "substantive-equality-status")
            .ok_or_else(|| LedgerError::new("control setup: substantive equality missing"))?
            .prohibited_inputs[0] = "Other limits".into();
        Ok(())
    });
    typed!("equality domains derive from direct claims", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.effect_key == "proactive-accessibility")
            .ok_or_else(|| LedgerError::new("control setup: proactive accessibility missing"))?
            .domain_refs = vec!["FS-DOM-12".into()];
        Ok(())
    });
    typed!(
        "constitutional effects receive no power allocation",
        |source| {
            source.function_allocations[0].power_ref = "FS-CCE-34".into();
            Ok(())
        }
    );
    typed!("equality effects cannot borrow T3", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.effect_key == "custody-distinction-narrowing")
            .ok_or_else(|| LedgerError::new("control setup: custody equality effect missing"))?
            .prohibited_inputs[3] = "Other limits".into();
        Ok(())
    });
    typed!(
        "family effects cannot regain an omnibus status proof",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "legacy-status-nonproof")
                .ok_or_else(|| LedgerError::new("control setup: legacy status effect missing"))?
                .prohibited_inputs[0] = "Other limits".into();
            Ok(())
        }
    );
    typed!(
        "family status cannot become a confinement input",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "family-status-no-confinement")
                .ok_or_else(|| {
                    LedgerError::new("control setup: family confinement effect missing")
                })?
                .prohibited_inputs[3] = "Other limits".into();
            Ok(())
        }
    );
    typed!("family effects receive no power allocation", |source| {
        source.function_allocations[0].power_ref = "FS-CCE-88".into();
        Ok(())
    });
    typed!("obligation origin profile cannot be dropped", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.effect_key == "public-respect-duty")
            .ok_or_else(|| LedgerError::new("control setup: public respect effect missing"))?
            .profiles
            .remove(0);
        Ok(())
    });
    typed!(
        "obligation adapters reuse every power-profile field",
        |source| {
            let terms = source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "public-respect-duty")
                .ok_or_else(|| LedgerError::new("control setup: public respect effect missing"))?
                .profile_terms
                .get_mut("liberty-power-limit-adapter")
                .ok_or_else(|| LedgerError::new("control setup: liberty adapter missing"))?;
            let key = terms
                .keys()
                .next_back()
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: liberty adapter empty"))?;
            terms.remove(&key);
            Ok(())
        }
    );
    typed!(
        "nonreciprocity wall is explicit for every duty gate",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "all-entitlement-nonreciprocity")
                .ok_or_else(|| LedgerError::new("control setup: nonreciprocity effect missing"))?
                .prohibited_inputs[0] = "Other limits".into();
            Ok(())
        }
    );
    typed!("finding routes require positive nonresponse", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.effect_key == "certified-positive-nonresponse")
            .ok_or_else(|| LedgerError::new("control setup: finding effect missing"))?
            .prohibited_inputs[0] = "Other limits".into();
        Ok(())
    });
    typed!(
        "ecological scientist is not an institutional proxy",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.effect_key == "public-fulfil-duty")
                .ok_or_else(|| LedgerError::new("control setup: public fulfil effect missing"))?
                .checking_role_refs
                .push("FS-ROL-49".into());
            Ok(())
        }
    );
    typed!("bounded delegation is decision-complete", |source| {
        let term = source
            .powers
            .iter_mut()
            .flat_map(|row| row.contract_terms.values_mut())
            .find(|term| term.basis == "bounded-delegation")
            .ok_or_else(|| LedgerError::new("control setup: no bounded delegation term"))?;
        term.failure_default = None;
        Ok(())
    });
    typed!("primary class follows the direct effect", |source| {
        source.powers[0].primary_class_ref = "class-10".into();
        Ok(())
    });
    typed!("domains derive only from affected claims", |source| {
        source.powers[0].domain_refs = vec!["FS-DOM-12".into()];
        Ok(())
    });
    typed!("planned tests cannot claim execution", |source| {
        source
            .powers
            .iter_mut()
            .find(|row| row.negative_test.status == "planned")
            .ok_or_else(|| LedgerError::new("control setup: no planned power test"))?
            .negative_test
            .status = "executable".into();
        Ok(())
    });
    typed!("formalization cannot precede coverage metadata", |source| {
        source
            .coverage_families
            .iter_mut()
            .find(|row| row.state == "coverage-ready")
            .ok_or_else(|| LedgerError::new("control setup: no coverage-ready family"))?
            .state = "formalized".into();
        Ok(())
    });
    typed!("prose cannot precede formalization", |source| {
        source
            .coverage_families
            .iter_mut()
            .find(|row| row.state == "coverage-ready")
            .ok_or_else(|| LedgerError::new("control setup: no coverage-ready family"))?
            .prose_refs = vec!["book-1/01-what-counts-as-evidence.md".into()];
        Ok(())
    });
    typed!(
        "state-form test status follows coverage-family state",
        |source| {
            let power_ref = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-003")
                .and_then(|row| row.card_refs.first())
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: state-form family empty"))?;
            let test = &mut source
                .powers
                .iter_mut()
                .find(|row| row.id == power_ref)
                .ok_or_else(|| LedgerError::new("control setup: state-form power missing"))?
                .negative_test;
            test.status = if test.status == "executable" {
                "planned"
            } else {
                "executable"
            }
            .into();
            Ok(())
        }
    );
    typed!(
        "state-form Part V status follows coverage-family state",
        |source| {
            let power_ref = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-003")
                .and_then(|row| row.card_refs.first())
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: state-form family empty"))?;
            let power = source
                .powers
                .iter_mut()
                .find(|row| row.id == power_ref)
                .ok_or_else(|| LedgerError::new("control setup: state-form power missing"))?;
            power.part_v_status = if power.part_v_status != "coverage-only-not-formalized" {
                "coverage-only-not-formalized"
            } else {
                "formalized-not-prose-landed"
            }
            .into();
            Ok(())
        }
    );
    typed!(
        "state-form formal statement order is checker-owned",
        |source| {
            source
                .coverage_families
                .iter_mut()
                .find(|row| row.id == "FS-CVF-003")
                .ok_or_else(|| LedgerError::new("control setup: FS-CVF-003 missing"))?
                .formal_statement_refs
                .reverse();
            Ok(())
        }
    );
    typed!(
        "state-form pin group reference is checker-owned",
        |source| {
            let replacement = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-011")
                .and_then(|row| row.pin_group_refs.first())
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: universal-standing pin missing"))?;
            source
                .coverage_families
                .iter_mut()
                .find(|row| row.id == "FS-CVF-003")
                .and_then(|row| row.pin_group_refs.first_mut())
                .ok_or_else(|| LedgerError::new("control setup: state-form pin missing"))?
                .clone_from(&replacement);
            Ok(())
        }
    );
    typed!(
        "state-form counterfactual reference is checker-owned",
        |source| {
            let replacement = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-011")
                .and_then(|row| row.counterfactual_refs.first())
                .cloned()
                .ok_or_else(|| {
                    LedgerError::new("control setup: universal counterfactual missing")
                })?;
            source
                .coverage_families
                .iter_mut()
                .find(|row| row.id == "FS-CVF-003")
                .and_then(|row| row.counterfactual_refs.first_mut())
                .ok_or_else(|| LedgerError::new("control setup: state counterfactual missing"))?
                .clone_from(&replacement);
            Ok(())
        }
    );
    typed!(
        "state-form per-card negative anchor is checker-owned",
        |source| {
            let refs = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-003")
                .ok_or_else(|| LedgerError::new("control setup: FS-CVF-003 missing"))?
                .card_refs[..2]
                .to_vec();
            let replacement = source
                .powers
                .iter()
                .find(|row| row.id == refs[1])
                .ok_or_else(|| LedgerError::new("control setup: second state power missing"))?
                .negative_test
                .executable_ref
                .0
                .clone();
            source
                .powers
                .iter_mut()
                .find(|row| row.id == refs[0])
                .ok_or_else(|| LedgerError::new("control setup: first state power missing"))?
                .negative_test
                .executable_ref = RequiredNullable(replacement);
            Ok(())
        }
    );
    typed!(
        "state-form per-card counterfactual anchor is checker-owned",
        |source| {
            let refs = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-003")
                .ok_or_else(|| LedgerError::new("control setup: FS-CVF-003 missing"))?
                .card_refs[..2]
                .to_vec();
            let replacement = source
                .powers
                .iter()
                .find(|row| row.id == refs[1])
                .ok_or_else(|| LedgerError::new("control setup: second state power missing"))?
                .counterfactual
                .executable_ref
                .0
                .clone();
            source
                .powers
                .iter_mut()
                .find(|row| row.id == refs[0])
                .ok_or_else(|| LedgerError::new("control setup: first state power missing"))?
                .counterfactual
                .executable_ref = RequiredNullable(replacement);
            Ok(())
        }
    );
    typed!(
        "economic test status follows coverage-family state",
        |source| {
            let power_ref = source
                .coverage_families
                .iter()
                .find(|row| row.id == "FS-CVF-006")
                .and_then(|row| row.card_refs.first())
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: economic family empty"))?;
            source
                .powers
                .iter_mut()
                .find(|row| row.id == power_ref)
                .ok_or_else(|| LedgerError::new("control setup: economic power missing"))?
                .negative_test
                .status = "planned".into();
            Ok(())
        }
    );
    typed!(
        "economic holder and function semantics are checker-owned",
        |source| {
            let power = source
                .powers
                .iter_mut()
                .find(|row| row.id == "FS-POW-061")
                .ok_or_else(|| LedgerError::new("control setup: FS-POW-061 missing"))?;
            power.holder_body_refs = vec!["FS-BOD-02".into()];
            power.holder_role_refs = vec!["FS-ROL-26".into()];
            Ok(())
        }
    );
    typed!("economic effects seal legacy and Book 2 inputs", |source| {
        source
            .constitutional_effects
            .iter_mut()
            .find(|row| row.id == "FS-CCE-223")
            .ok_or_else(|| LedgerError::new("control setup: FS-CCE-223 missing"))?
            .prohibited_inputs[0] = "Other limits".into();
        Ok(())
    });
    typed!(
        "economic effect terms require their substantive applicability prefix",
        |source| {
            let effect = source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.id == "FS-CCE-223")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-223 missing"))?;
            let prefix = format!("{} ", effect.applicability);
            let term = effect
                .contract_terms
                .get_mut("evidence_rule")
                .ok_or_else(|| LedgerError::new("control setup: evidence_rule missing"))?;
            term.text = term
                .text
                .strip_prefix(&prefix)
                .ok_or_else(|| LedgerError::new("control setup: applicability prefix missing"))?
                .to_owned();
            Ok(())
        }
    );
    typed!(
        "economic effect applicability cannot be swapped across effects",
        |source| {
            let swapped = source
                .constitutional_effects
                .iter()
                .find(|row| row.id == "FS-CCE-224")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-224 missing"))?
                .applicability
                .clone();
            let effect = source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.id == "FS-CCE-223")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-223 missing"))?;
            let prefix = format!("{} ", effect.applicability);
            let term = effect
                .contract_terms
                .get_mut("evidence_rule")
                .ok_or_else(|| LedgerError::new("control setup: evidence_rule missing"))?;
            let suffix = term
                .text
                .strip_prefix(&prefix)
                .ok_or_else(|| LedgerError::new("control setup: applicability prefix missing"))?;
            term.text = format!("{swapped} {suffix}");
            Ok(())
        }
    );
    typed!(
        "economic effect terms reject a bare identity tag",
        |source| {
            let effect = source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.id == "FS-CCE-223")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-223 missing"))?;
            let prefix = format!("{} ", effect.applicability);
            let term = effect
                .contract_terms
                .get_mut("evidence_rule")
                .ok_or_else(|| LedgerError::new("control setup: evidence_rule missing"))?;
            let suffix = term
                .text
                .strip_prefix(&prefix)
                .ok_or_else(|| LedgerError::new("control setup: applicability prefix missing"))?;
            term.text = format!("For FS-CCE-223 (EconomicFormSuppression), {suffix}");
            Ok(())
        }
    );
    typed!(
        "economic effect terms reject normalized duplicates",
        |source| {
            let effect = source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.id == "FS-CCE-223")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-223 missing"))?;
            let duplicate = effect
                .contract_terms
                .get("lawful_source")
                .ok_or_else(|| LedgerError::new("control setup: lawful_source missing"))?
                .text
                .replacen("Only the", "Only  the", 1);
            if duplicate
                == effect
                    .contract_terms
                    .get("lawful_source")
                    .expect("lawful_source checked above")
                    .text
            {
                return Err(LedgerError::new(
                    "control setup: lawful_source normalization token missing",
                ));
            }
            effect
                .contract_terms
                .get_mut("trigger")
                .ok_or_else(|| LedgerError::new("control setup: trigger missing"))?
                .text = duplicate;
            Ok(())
        }
    );
    typed!(
        "economic effect term suffixes cannot be invented",
        |source| {
            source
                .constitutional_effects
                .iter_mut()
                .find(|row| row.id == "FS-CCE-224")
                .ok_or_else(|| LedgerError::new("control setup: FS-CCE-224 missing"))?
                .contract_terms
                .get_mut("evidence_rule")
                .ok_or_else(|| LedgerError::new("control setup: evidence_rule missing"))?
                .text
                .push_str(" An invented extension follows.");
            Ok(())
        }
    );
    typed!(
        "economic effect completion cannot weaken its evidence ceiling",
        |source| {
            source
                .coverage_families
                .iter_mut()
                .find(|row| row.id == "FS-CVF-017")
                .ok_or_else(|| LedgerError::new("control setup: FS-CVF-017 missing"))?
                .blocked_before_drafting = "All economic effects are operationally assured.".into();
            Ok(())
        }
    );
    typed!("formal statements are assigned exactly once", |source| {
        source.coverage_families[0].formal_statement_refs.pop();
        Ok(())
    });
    typed!(
        "other powers cannot borrow the retained T3 record",
        |source| {
            let power = source
                .powers
                .iter_mut()
                .find(|row| row.manifest_key != "formal-active-custody")
                .ok_or_else(|| LedgerError::new("control setup: no non-retained power"))?;
            power
                .prohibited_inputs
                .retain(|value| !value.contains("formal-active-custody"));
            Ok(())
        }
    );
    typed!(
        "Book 2 routing is not a constitutional coverage family",
        |source| {
            source.coverage_families[2].source_family_refs = vec!["book-2-operation".into()];
            Ok(())
        }
    );
    typed!("unknown power holder body is refused", |source| {
        source.powers[0].holder_body_refs[0] = "FS-BOD-999".into();
        Ok(())
    });
    typed!(
        "state-form power holder semantics are checker-owned",
        |source| {
            let power = source
                .powers
                .iter_mut()
                .find(|row| row.id == "FS-POW-023")
                .ok_or_else(|| LedgerError::new("control setup: FS-POW-023 missing"))?;
            power.holder_body_refs = vec!["FS-BOD-02".into()];
            power.holder_role_refs = vec!["FS-ROL-26".into()];
            Ok(())
        }
    );
    typed!(
        "delegated state-form holders cannot choose mechanics",
        |source| {
            source
                .powers
                .iter_mut()
                .find(|row| row.id == "FS-POW-028")
                .ok_or_else(|| LedgerError::new("control setup: FS-POW-028 missing"))?
                .contract_terms
                .get_mut("lawful_source")
                .ok_or_else(|| LedgerError::new("control setup: lawful source term missing"))?
                .text = "The holder arrays choose the appointment.".into();
            Ok(())
        }
    );
    typed!("a power allocation cannot disappear", |source| {
        source.function_allocations.pop();
        Ok(())
    });
    typed!("one allocation cannot serve two powers", |source| {
        source.function_allocations[0].power_ref = source.function_allocations[1].power_ref.clone();
        Ok(())
    });
    typed!("required function separation cannot be fused", |source| {
        source.function_allocations[0].auditor_body_refs[0] =
            source.function_allocations[0].decisive_fact_writer_body_refs[0].clone();
        Ok(())
    });
    typed!(
        "state-form function allocation semantics are checker-owned",
        |source| {
            let allocation = source
                .function_allocations
                .iter_mut()
                .find(|row| row.power_ref == "FS-POW-023")
                .ok_or_else(|| LedgerError::new("control setup: FS-POW-023 allocation missing"))?;
            allocation.decider_body_refs = vec!["FS-BOD-02".into()];
            allocation.decider_role_refs = vec!["FS-ROL-26".into()];
            Ok(())
        }
    );
    typed!("a refusal cannot be promoted into a power", |source| {
        source.powers.push(source.powers[0].clone());
        Ok(())
    });
    typed!("formal crosswalk policy is checker-owned", |source| {
        source.power_crosswalk_dispositions[0].crosswalk_action = "retire".into();
        Ok(())
    });
    typed!(
        "T3 custody authority cannot merge with its executor",
        |source| {
            source
                .powers
                .iter_mut()
                .find(|row| row.manifest_key == "formal-active-custody")
                .ok_or_else(|| LedgerError::new("control setup: retained T3 power missing"))?
                .holder_body_refs = vec!["FS-BOD-35".into()];
            Ok(())
        }
    );

    Ok(passed)
}

fn closure_record_mut(source: &mut LedgerDocument) -> LedgerResult<&mut ClosureProjection> {
    source
        .closure_record
        .0
        .as_mut()
        .ok_or_else(|| LedgerError::new("control setup: closure record missing"))
}

fn current_scope_audit_mut(source: &mut LedgerDocument) -> LedgerResult<&mut ScopeAudit> {
    source
        .scope_audits
        .last_mut()
        .ok_or_else(|| LedgerError::new("control setup: current scope audit missing"))
}

fn run_coverage_projection_negative_control(ledger: &ValidatedLedger) -> LedgerResult<()> {
    let mut mutant = ledger.document.clone();
    mutant.scope_audits.push(transient_control_audit(
        &mutant,
        "Watched-mutation current audit",
    )?);
    mutant.closure_record.0 = None;
    mutant.acceptance_gate.verdict = "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED".into();
    mutant.acceptance_gate.gate_a_status = "not-passed".into();
    mutant.legacy_rows[0]
        .legacy_gap
        .push_str(" ## 3. Current coverage versus target scope");
    let digest = review_scope_digest(&mutant)?;
    current_scope_audit_mut(&mut mutant)?.scope_sha256 = digest;
    validate_source_with_projections(
        &ledger.input_bytes,
        &mutant,
        &ledger.sibling_projections,
        &ledger.reader_projection,
    )
    .map_err(|error| {
        LedgerError::new(format!(
            "negative control failed before coverage projection: {error}"
        ))
    })?;
    let body = render_coverage_region(&mutant)?;
    let source_bytes = serde_json::to_vec(&mutant).map_err(|error| {
        LedgerError::new(format!(
            "cannot serialize coverage projection mutant: {error}"
        ))
    })?;
    let current = std::str::from_utf8(input_bytes(&ledger.input_bytes, COVERAGE_MAP)?)
        .map_err(|_| LedgerError::new("coverage map is not UTF-8"))?;
    match splice_coverage_map(current, &body, &source_bytes) {
        Err(error) if error.to_string().contains("exactly once") => Ok(()),
        Err(error) => Err(LedgerError::new(format!(
            "coverage projection control failed for the wrong reason: {error}"
        ))),
        Ok(_) => Err(LedgerError::new(
            "negative control failed to fail: the generated region may not duplicate a coverage-map needle",
        )),
    }
}

fn negative_controls_closure_and_audit(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $preserve_scope:expr, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, $preserve_scope, None, $mutate)?;
            passed += 1;
        }};
    }
    macro_rules! closure {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, true, true, None, $mutate)?;
            passed += 1;
        }};
    }

    run_coverage_projection_negative_control(ledger)?;
    passed += 1;
    closure!(
        "a closure record cannot bypass closure-derived acceptance",
        |source| {
            source.acceptance_gate.gate_a_status = "not-passed".into();
            Ok(())
        }
    );
    closure!("a closure record gate is exact", |source| {
        closure_record_mut(source)?.gate = "gate-b".into();
        Ok(())
    });
    closure!("a closure record claim is byte-exact", |source| {
        closure_record_mut(source)?.permitted_claim = "broader control claim".into();
        Ok(())
    });
    closure!("a closure candidate is an immutable Git id", |source| {
        closure_record_mut(source)?.candidate_commit_sha = "candidate".into();
        Ok(())
    });
    closure!("a closure source version is current", |source| {
        closure_record_mut(source)?.source_version = "stale".into();
        Ok(())
    });
    closure!("a closure scope digest is current", |source| {
        closure_record_mut(source)?.scope_sha256 = "0".repeat(64);
        Ok(())
    });
    closure!("a closure record must cite FS-ENV-01", |source| {
        closure_record_mut(source)?.envelope_ref = "FS-ENV-02".into();
        Ok(())
    });
    closure!(
        "a closure record requires a current-source repository audit",
        |source| {
            closure_record_mut(source)?.scope_audit_ref = "FS-SAU-98".into();
            Ok(())
        }
    );
    closure!(
        "a closure cutoff must match its repository audit",
        |source| {
            closure_record_mut(source)?.audit_cutoff_at_utc = "2026-08-19T00:00:00Z".into();
            Ok(())
        }
    );
    closure!("closure assurance refs are checker-derived", |source| {
        closure_record_mut(source)?.assurance_record_refs.pop();
        Ok(())
    });
    closure!("closure residual refs are checker-derived", |source| {
        closure_record_mut(source)?
            .residual_refs
            .push("FS-DFT-999".into());
        Ok(())
    });
    closure!(
        "closure claim limitations bind residuals exactly",
        |source| {
            closure_record_mut(source)?
                .claim_limitations
                .push(ClaimLimitation {
                    defect_ref: "FS-DFT-999".into(),
                    affected_claim_ref: "FS-CLM-01".into(),
                    public_claim_restriction: "control".into(),
                });
            Ok(())
        }
    );
    closure!(
        "closure verification uses a content-addressed v2 receipt",
        |source| {
            closure_record_mut(source)?.verification_receipt_ref = "not-a-receipt".into();
            current_scope_audit_mut(source)?.verification_receipt_ref =
                Some("not-a-receipt".into());
            Ok(())
        }
    );
    closure!("audit and closure bind the same v2 receipt", |source| {
        closure_record_mut(source)?.verification_receipt_ref = format!(
            "new-book-plans/verification-receipts/sha256-{}.json",
            "0".repeat(64)
        );
        Ok(())
    });
    run_shape_negative_control(
        ledger,
        "v2 closure refuses an inline v1 downgrade",
        None,
        true,
        |value| {
            let closure = value_object_mut(value, "root")?
                .get_mut("closure_record")
                .ok_or_else(|| LedgerError::new("control setup: closure_record missing"))?;
            let closure = value_object_mut(closure, "closure_record")?;
            let candidate = closure
                .get("candidate_commit_sha")
                .cloned()
                .ok_or_else(|| LedgerError::new("control setup: candidate missing"))?;
            closure.remove("verification_receipt_ref");
            closure.insert(
                "verification_receipt".into(),
                serde_json::json!({
                    "candidate_commit_sha": candidate,
                    "verified_at_utc": "2026-08-23T00:00:00Z",
                    "commands": [],
                    "result": "all-passed",
                    "transcript_sha256": "0".repeat(64),
                }),
            );
            Ok(())
        },
    )?;
    passed += 1;
    closure!("closure policy is checker-owned", |source| {
        closure_record_mut(source)?.closure_policy_ref =
            "new-book-plans/full-society-scope-review-protocol.md::# Full-Society Scope-Review Protocol".into();
        Ok(())
    });
    typed!(
        "R7 cannot be marked unbuilt after its checks land",
        true,
        |source| {
            let route = source
                .routes
                .iter_mut()
                .find(|row| row.id == "FS-RTE-07")
                .ok_or_else(|| LedgerError::new("control setup: FS-RTE-07 missing"))?;
            route.status = "unbuilt".into();
            route.route_status = "unbuilt".into();
            Ok(())
        }
    );
    typed!("R7 cannot be relabelled available", true, |source| {
        let route = source
            .routes
            .iter_mut()
            .find(|row| row.id == "FS-RTE-07")
            .ok_or_else(|| LedgerError::new("control setup: FS-RTE-07 missing"))?;
        route.status = "available".into();
        route.route_status = "available".into();
        Ok(())
    });
    typed!(
        "the current scope audit binds the source version",
        false,
        |source| {
            source.source_version = "control-current-source".into();
            current_scope_audit_mut(source)?.source_version = "stale-source".into();
            Ok(())
        }
    );
    run_typed_negative_control(
        ledger,
        "the current scope audit binds the semantic scope digest",
        false,
        true,
        Some("requires a current-source repository audit"),
        |source| {
            source.title.push_str(" control semantic drift");
            current_scope_audit_mut(source)?.scope_sha256 = "0".repeat(64);
            Ok(())
        },
    )?;
    passed += 1;
    typed!(
        "the current scope audit binds the protocol digest",
        false,
        |source| {
            source.source_version = "control-current-source".into();
            let audit = current_scope_audit_mut(source)?;
            audit.source_version = "control-current-source".into();
            audit.protocol_sha256 = "0".repeat(64);
            Ok(())
        }
    );
    typed!(
        "the current scope audit covers every criterion",
        false,
        |source| {
            current_scope_audit_mut(source)?.criterion_coverage.pop();
            Ok(())
        }
    );
    typed!(
        "the current scope audit binds exact checker controls",
        false,
        |source| {
            current_scope_audit_mut(source)?.control_refs.pop();
            Ok(())
        }
    );
    typed!(
        "the current scope audit binds the command chain",
        false,
        |source| {
            current_scope_audit_mut(source)?.commands.pop();
            Ok(())
        }
    );
    typed!(
        "the current scope audit covers Gate A findings",
        false,
        |source| {
            current_scope_audit_mut(source)?.finding_refs.pop();
            Ok(())
        }
    );
    typed!(
        "the current scope audit result token is exact",
        false,
        |source| {
            current_scope_audit_mut(source)?.result = "passed".into();
            Ok(())
        }
    );
    typed!(
        "the current scope audit preserves its evidence ceiling",
        false,
        |source| {
            current_scope_audit_mut(source)?.evidence_ceiling = "broader claim".into();
            Ok(())
        }
    );
    typed!(
        "the current scope audit binds the mechanical closure policy",
        false,
        |source| {
            current_scope_audit_mut(source)?.policy_basis = Some(
            "new-book-plans/full-society-scope-review-protocol.md::# Full-Society Scope-Review Protocol".into(),
        );
            Ok(())
        }
    );
    typed!(
        "the current scope audit cannot depend on an author act",
        false,
        |source| {
            let audit = current_scope_audit_mut(source)?;
            audit.policy_basis = None;
            audit.author_basis = Some("control author dependency".into());
            Ok(())
        }
    );

    Ok(passed)
}

fn make_control_commission(
    source: &mut LedgerDocument,
    inputs: &BTreeMap<String, Vec<u8>>,
) -> LedgerResult<usize> {
    if !source.review_commissions.is_empty() {
        return Ok(source.review_commissions.len() - 1);
    }
    let split = REVIEW_CRITERIA.len() / 2;
    let reviewer = |identity: &str, discipline: &str, criteria: &[&str]| Reviewer {
        identity: identity.into(),
        discipline: discipline.into(),
        criterion_refs: criteria.iter().map(|value| (*value).into()).collect(),
        consent_attestation: REVIEWER_CONSENT.into(),
        conflict_attestation: REVIEWER_CONFLICT_CLEAR.into(),
        compensation_attestation: REVIEWER_COMPENSATION_CLEAR.into(),
    };
    source.review_commissions.push(ReviewCommission {
        id: "FS-COM-99".into(),
        title: "control commission".into(),
        source_version: source.source_version.clone(),
        scope_sha256: review_scope_digest(source)?,
        protocol_sha256: protocol_digest(inputs)?,
        plant_commitment_sha256: "a".repeat(64),
        seed_commitment_sha256: "b".repeat(64),
        commissioned_at_utc: "2026-08-14T00:00:00Z".into(),
        received_window: ReceivedWindow {
            opens_at_utc: "2026-08-15T00:00:00Z".into(),
            closes_at_utc: "2026-08-16T00:00:00Z".into(),
        },
        cutoff_at_utc: "2026-08-17T00:00:00Z".into(),
        custodian_identity: source.review_protocol.designation.custodian.clone(),
        reviewers: vec![
            reviewer(
                "control-reviewer-a",
                "constitutional-law",
                &REVIEW_CRITERIA[..split],
            ),
            reviewer(
                "control-reviewer-b",
                "systems-safety",
                &REVIEW_CRITERIA[split..],
            ),
        ],
        criterion_coverage: REVIEW_CRITERIA
            .iter()
            .map(|value| (*value).into())
            .collect(),
        packet_paths: REVIEW_PACKET_PATHS
            .iter()
            .map(|value| (*value).into())
            .collect(),
    });
    Ok(source.review_commissions.len() - 1)
}

struct ControlProposalSpec<'a> {
    id: &'a str,
    source_kind: &'a str,
    source_identity: &'a str,
    finding: &'a str,
    classification: &'a str,
    disposition: &'a str,
    control_kind: &'a str,
    severity: Option<&'a str>,
}

fn base_control_proposal(
    designation: &ReviewDesignation,
    spec: ControlProposalSpec<'_>,
) -> Proposal {
    let routed = UNESTABLISHED_DISPOSITIONS
        .contains(&spec.classification)
        .then(|| spec.classification.to_owned());
    Proposal {
        id: spec.id.into(),
        title: format!("control proposal {}", spec.id),
        proposal: format!("control proposal payload {}", spec.id),
        source_kind: spec.source_kind.into(),
        source_identity: spec.source_identity.into(),
        received_at_utc: "2026-08-15T12:00:00Z".into(),
        triaged_at_utc: "2026-08-16T01:00:00Z".into(),
        severity_owner_identity: designation.severity_owner.clone(),
        materiality_finding: spec.finding.into(),
        materiality_reason: "control materiality reason".into(),
        classification: spec.classification.into(),
        checked_at_utc: "2026-08-16T02:00:00Z".into(),
        independent_checker_identity: designation.independent_checker.clone(),
        check_finding: "confirmed".into(),
        check_reason: "control checking reason".into(),
        proposal_disposition: spec.disposition.into(),
        disposition_at_utc: "2026-08-16T03:00:00Z".into(),
        reasons: "control disposition reason".into(),
        review_event_ref: "FS-REV-99".into(),
        control_kind: spec.control_kind.into(),
        severity: spec.severity.map(str::to_owned),
        created_record_refs: None,
        routed_unestablished_disposition: routed,
        defect_row_ref: None,
        retained_limit_binding: None,
    }
}

fn make_control_event(
    source: &mut LedgerDocument,
    inputs: &BTreeMap<String, Vec<u8>>,
    passed: bool,
) -> LedgerResult<usize> {
    let commission_index = make_control_commission(source, inputs)?;
    let commission = source.review_commissions[commission_index].clone();
    let designation = source.review_protocol.designation.clone();
    let defect = source
        .defects
        .first()
        .cloned()
        .ok_or_else(|| LedgerError::new("control setup: no defect"))?;
    let mut plant = base_control_proposal(
        &designation,
        ControlProposalSpec {
            id: "FS-PRO-97",
            source_kind: "reviewer",
            source_identity: &commission.reviewers[0].identity,
            finding: "material",
            classification: "retained-limit",
            disposition: "retained-limit",
            control_kind: "plant-match",
            severity: Some(severity_class(&defect)?),
        },
    );
    plant.defect_row_ref = Some(defect.id.clone());
    plant.retained_limit_binding = Some(expected_retained_binding(source, &defect)?);
    let seed_material = base_control_proposal(
        &designation,
        ControlProposalSpec {
            id: "FS-PRO-98",
            source_kind: "seed",
            source_identity: "committed-seed-control",
            finding: "material",
            classification: "routed-book-2",
            disposition: "classified-out",
            control_kind: "seed",
            severity: Some("material"),
        },
    );
    let seed_immaterial = base_control_proposal(
        &designation,
        ControlProposalSpec {
            id: "FS-PRO-99",
            source_kind: "seed",
            source_identity: "committed-seed-control",
            finding: "immaterial",
            classification: "immaterial",
            disposition: "classified-out",
            control_kind: "seed",
            severity: None,
        },
    );
    source
        .proposals
        .extend([plant, seed_material, seed_immaterial]);
    let ordered = source.proposals.iter().collect::<Vec<_>>();
    let ordered_ids = ordered.iter().map(|row| row.id.clone()).collect();
    let manifest_sha256 = proposal_intake_digest("FS-REV-99", &ordered)?;
    let checker = designation.independent_checker.clone();
    source.review_events.push(ReviewEvent {
        id: "FS-REV-99".into(),
        title: "control terminal event".into(),
        commission_ref: commission.id.clone(),
        packet_commit_sha: "c".repeat(40),
        source_version: commission.source_version.clone(),
        scope_sha256: commission.scope_sha256.clone(),
        protocol_sha256: commission.protocol_sha256.clone(),
        intake_receipt: IntakeReceipt {
            frozen_at_utc: "2026-08-16T00:01:00Z".into(),
            ordered_proposal_ids: ordered_ids,
            manifest_sha256,
        },
        control_reveal: ControlReveal {
            revealed_at_utc: "2026-08-17T00:00:00Z".into(),
            plant_preimage_sha256: commission.plant_commitment_sha256,
            seed_preimage_sha256: commission.seed_commitment_sha256,
            planted_proposal_ref: RequiredNullable(Some("FS-PRO-97".into())),
            seed_results: vec![
                SeedResult {
                    proposal_ref: "FS-PRO-98".into(),
                    expected_materiality: "material".into(),
                    expected_severity: RequiredNullable(Some(
                        if passed { "material" } else { "critical" }.into(),
                    )),
                    expected_disposition: "classified-out".into(),
                    verified_by: checker.clone(),
                    verification_reason: "control seed verification".into(),
                },
                SeedResult {
                    proposal_ref: "FS-PRO-99".into(),
                    expected_materiality: "immaterial".into(),
                    expected_severity: RequiredNullable(None),
                    expected_disposition: "classified-out".into(),
                    verified_by: checker.clone(),
                    verification_reason: "control seed verification".into(),
                },
            ],
            plant_match_checked_by: checker,
            plant_match_reason: "control plant match".into(),
        },
        seeded_control: ReviewControl {
            status: if passed { "passed" } else { "failed" }.into(),
            reason: "control seeded outcome".into(),
        },
        planted_control: ReviewControl {
            status: "passed".into(),
            reason: "control planted outcome".into(),
        },
        outcome_status: if passed { "passed" } else { "failed" }.into(),
        outcome_reason: "control terminal outcome".into(),
    });
    Ok(source.review_events.len() - 1)
}

fn run_prepared_shape_negative_control<P, M>(
    ledger: &ValidatedLedger,
    name: &str,
    prepare: P,
    mutate: M,
) -> LedgerResult<()>
where
    P: FnOnce(&mut LedgerDocument, &BTreeMap<String, Vec<u8>>) -> LedgerResult<()>,
    M: FnOnce(&mut Value) -> LedgerResult<()>,
{
    let mut source = ledger.document.clone();
    source.scope_audits.push(transient_control_audit(
        &source,
        "Watched-mutation current audit",
    )?);
    source.closure_record.0 = None;
    source.acceptance_gate.verdict = "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED".into();
    source.acceptance_gate.gate_a_status = "not-passed".into();
    prepare(&mut source, &ledger.input_bytes)?;
    let digest = review_scope_digest(&source)?;
    current_scope_audit_mut(&mut source)?.scope_sha256 = digest;
    validate_source_with_projections(
        &ledger.input_bytes,
        &source,
        &ledger.sibling_projections,
        &ledger.reader_projection,
    )
    .map_err(|error| LedgerError::new(format!("control fixture invalid for {name}: {error}")))?;
    let mut value = serde_json::to_value(&source).map_err(|error| {
        LedgerError::new(format!(
            "cannot serialize control fixture for {name}: {error}"
        ))
    })?;
    mutate(&mut value)?;
    let bytes = serde_json::to_vec(&value).map_err(|error| {
        LedgerError::new(format!(
            "cannot serialize malformed control for {name}: {error}"
        ))
    })?;
    match parse_source(&bytes) {
        Err(_) => Ok(()),
        Ok(_) => Err(LedgerError::new(format!(
            "negative control failed to fail: {name}"
        ))),
    }
}

fn negative_controls_optional_review(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut passed = 0_usize;
    macro_rules! typed {
        ($name:literal, $mutate:expr) => {{
            run_typed_negative_control(ledger, $name, false, false, None, $mutate)?;
            passed += 1;
        }};
    }

    typed!("a passed event outcome is derived, not prose", |source| {
        let index = make_control_event(source, &ledger.input_bytes, true)?;
        source.review_events[index].outcome_status = "failed".into();
        Ok(())
    });
    typed!("a terminal event requires its commission", |source| {
        make_control_event(source, &ledger.input_bytes, true)?;
        source.review_commissions.clear();
        Ok(())
    });
    typed!(
        "a current commission binds the semantic scope digest",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].scope_sha256 = "0".repeat(64);
            Ok(())
        }
    );
    typed!(
        "a current commission binds the exact protocol digest",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].protocol_sha256 = "0".repeat(64);
            Ok(())
        }
    );
    typed!("plant and seed commitments are distinct", |source| {
        let index = make_control_commission(source, &ledger.input_bytes)?;
        source.review_commissions[index].seed_commitment_sha256 = source.review_commissions[index]
            .plant_commitment_sha256
            .clone();
        Ok(())
    });
    typed!(
        "commission chronology is canonical UTC and ordered",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].commissioned_at_utc = "2026-08-15T01:00:00Z".into();
            Ok(())
        }
    );
    typed!("commission windows are structured UTC", |source| {
        let index = make_control_commission(source, &ledger.input_bytes)?;
        source.review_commissions[index]
            .received_window
            .opens_at_utc = "tomorrow".into();
        Ok(())
    });
    typed!("the packet manifest is exact and ordered", |source| {
        let index = make_control_commission(source, &ledger.input_bytes)?;
        source.review_commissions[index].packet_paths.pop();
        Ok(())
    });
    typed!("the panel contains at least two disciplines", |source| {
        let index = make_control_commission(source, &ledger.input_bytes)?;
        let discipline = source.review_commissions[index].reviewers[0]
            .discipline
            .clone();
        source.review_commissions[index].reviewers[1].discipline = discipline;
        Ok(())
    });
    typed!(
        "reviewer criteria collectively cover the full rubric",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].reviewers[1]
                .criterion_refs
                .pop();
            Ok(())
        }
    );
    typed!(
        "a reviewer cannot be Darshu, Dhanush, or custodian",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].reviewers[0].identity =
                source.review_protocol.designation.severity_owner.clone();
            Ok(())
        }
    );
    run_prepared_shape_negative_control(
        ledger,
        "reviewer conflict attestations are exact",
        |source, inputs| {
            make_control_commission(source, inputs)?;
            Ok(())
        },
        |value| {
            let reviewer = value_array_mut(value, "review_commissions")?
                .first_mut()
                .and_then(Value::as_object_mut)
                .and_then(|commission| commission.get_mut("reviewers"))
                .and_then(Value::as_array_mut)
                .and_then(|reviewers| reviewers.first_mut())
                .ok_or_else(|| LedgerError::new("control setup: reviewer missing"))?;
            value_object_mut(reviewer, "reviewer")?.remove("conflict_attestation");
            Ok(())
        },
    )?;
    passed += 1;
    typed!(
        "findings-contingent reviewer compensation is refused",
        |source| {
            let index = make_control_commission(source, &ledger.input_bytes)?;
            source.review_commissions[index].reviewers[0].compensation_attestation =
                "findings-contingent".into();
            Ok(())
        }
    );
    typed!(
        "the frozen intake equals the event proposal set",
        |source| {
            let index = make_control_event(source, &ledger.input_bytes, true)?;
            source.review_events[index]
                .intake_receipt
                .ordered_proposal_ids
                .pop();
            Ok(())
        }
    );
    typed!(
        "the frozen intake digest binds proposal payloads",
        |source| {
            let index = make_control_event(source, &ledger.input_bytes, true)?;
            source.review_events[index].intake_receipt.manifest_sha256 = "0".repeat(64);
            Ok(())
        }
    );
    typed!("controls cannot reveal early", |source| {
        let index = make_control_event(source, &ledger.input_bytes, true)?;
        source.review_events[index].control_reveal.revealed_at_utc = "2026-08-16T02:30:00Z".into();
        Ok(())
    });
    typed!("every proposal is received inside the window", |source| {
        make_control_event(source, &ledger.input_bytes, false)?;
        source
            .proposals
            .last_mut()
            .expect("control proposals")
            .received_at_utc = "2026-08-18T00:00:00Z".into();
        Ok(())
    });
    typed!("proposal chronology is ordered", |source| {
        make_control_event(source, &ledger.input_bytes, false)?;
        source
            .proposals
            .last_mut()
            .expect("control proposals")
            .checked_at_utc = "2026-08-16T00:30:00Z".into();
        Ok(())
    });
    typed!("every proposal receives Darshu triage", |source| {
        make_control_event(source, &ledger.input_bytes, false)?;
        source
            .proposals
            .last_mut()
            .expect("control proposals")
            .severity_owner_identity = "someone-else".into();
        Ok(())
    });
    typed!("every proposal receives Dhanush checking", |source| {
        make_control_event(source, &ledger.input_bytes, false)?;
        source
            .proposals
            .last_mut()
            .expect("control proposals")
            .independent_checker_identity = "someone-else".into();
        Ok(())
    });
    typed!(
        "classification maps exactly to its outward disposition",
        |source| {
            make_control_event(source, &ledger.input_bytes, false)?;
            source.proposals[1].routed_unestablished_disposition =
                Some("external-assumption".into());
            Ok(())
        }
    );
    typed!(
        "an added proposal names resolvable created records",
        |source| {
            make_control_event(source, &ledger.input_bytes, false)?;
            let proposal = &mut source.proposals[0];
            proposal.classification = "material-omission".into();
            proposal.proposal_disposition = "added".into();
            proposal.defect_row_ref = None;
            proposal.retained_limit_binding = None;
            proposal.created_record_refs = Some(vec!["bogus-file.md::no such anchor".into()]);
            Ok(())
        }
    );
    typed!(
        "a retained limit links its exact defect binding",
        |source| {
            make_control_event(source, &ledger.input_bytes, false)?;
            source.proposals[0].defect_row_ref = None;
            Ok(())
        }
    );
    typed!(
        "failed events with proposal intake cannot be deleted",
        |source| {
            make_control_event(source, &ledger.input_bytes, false)?;
            source.review_events.clear();
            Ok(())
        }
    );

    Ok(passed)
}

fn gate_a_condition_one_deferred(source: &LedgerDocument) -> Vec<String> {
    const POPULATIONS: [&str; 7] = [
        "domains",
        "roles",
        "powers",
        "dependencies",
        "scenarios",
        "defects",
        "coverage-contracts",
    ];
    let mut result = source
        .deferred_populations
        .iter()
        .filter_map(|row| {
            POPULATIONS
                .contains(&row.record_type.as_str())
                .then(|| row.record_type.clone())
        })
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}

fn semantic_controls(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    let mut gate_a_critical = ledger.document.clone();
    gate_a_critical.closure_record.0 = None;
    gate_a_critical.acceptance_gate.verdict = "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED".into();
    gate_a_critical.acceptance_gate.gate_a_status = "not-passed".into();
    gate_a_critical
        .defects
        .iter_mut()
        .find(|row| row.id == "FS-DFT-27")
        .ok_or_else(|| LedgerError::new("semantic control setup: FS-DFT-27 missing"))?
        .severity = "critical — semantic control for a scope-map defect".into();
    let mut audit = transient_control_audit(&gate_a_critical, "Semantic-control current audit")?;
    audit.scope_sha256 = review_scope_digest(&gate_a_critical)?;
    gate_a_critical.scope_audits.push(audit);
    let resolution = validate_source_with_projections(
        &ledger.input_bytes,
        &gate_a_critical,
        &ledger.sibling_projections,
        &ledger.reader_projection,
    )?;
    let defect = gate_a_critical
        .defects
        .iter()
        .find(|row| row.id == "FS-DFT-27")
        .expect("semantic control retained FS-DFT-27");
    if !resolution.get("FS-DFT-27").is_some_and(|row| row.blocking)
        || !defect
            .applicable_gate_refs
            .iter()
            .any(|gate| gate == "gate-a")
    {
        return Err(LedgerError::new(
            "semantic control failed: a valid Gate-A-applicable critical defect must make condition three unmet",
        ));
    }

    let mut review_only = ledger.document.clone();
    review_only.deferred_populations.extend([
        DeferredPopulation {
            record_type: "proposals".into(),
            owner_ref: SCOPE_AUDIT_POLICY_BASIS.into(),
            closure_condition: "optional review proposal population".into(),
            stage: "optional".into(),
        },
        DeferredPopulation {
            record_type: "review_events".into(),
            owner_ref: SCOPE_AUDIT_POLICY_BASIS.into(),
            closure_condition: "optional review event population".into(),
            stage: "optional".into(),
        },
    ]);
    let before = gate_a_condition_one_deferred(&review_only);
    review_only
        .deferred_populations
        .retain(|row| !matches!(row.record_type.as_str(), "proposals" | "review_events"));
    if before != gate_a_condition_one_deferred(&review_only) {
        return Err(LedgerError::new(
            "semantic control failed: review outputs may not alter condition one",
        ));
    }

    let complete_reader = render_reader(&ledger.document, &ledger.resolutions)?;
    validate_reader_projection(&ledger.document, &complete_reader)?;
    let first_population = reader_population_lines(&ledger.document)?
        .into_iter()
        .next()
        .ok_or_else(|| LedgerError::new("semantic control setup: no reader population"))?;
    let incomplete_reader = complete_reader.replacen(&first_population, "", 1);
    if validate_reader_projection(&ledger.document, &incomplete_reader).is_ok() {
        return Err(LedgerError::new(
            "semantic control failed: reader projection may not omit a canonical population",
        ));
    }
    Ok(3)
}

fn negative_controls(ledger: &ValidatedLedger) -> LedgerResult<usize> {
    type ControlGroup = fn(&ValidatedLedger) -> LedgerResult<usize>;
    const GROUPS: [(&str, ControlGroup); 7] = [
        ("claims-and-defects", negative_controls_claims_and_defects),
        (
            "envelope-roles-bodies",
            negative_controls_envelope_roles_bodies,
        ),
        (
            "dependencies-and-scenarios",
            negative_controls_dependencies_and_scenarios,
        ),
        ("power-and-effects", negative_controls_power_and_effects),
        (
            "closure-and-scope-audit",
            negative_controls_closure_and_audit,
        ),
        ("optional-review", negative_controls_optional_review),
        ("semantic", semantic_controls),
    ];
    const WORKERS: usize = 4;
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    thread::scope(|scope| {
        for _ in 0..WORKERS.min(GROUPS.len()) {
            let sender = sender.clone();
            let next = &next;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some((_, group)) = GROUPS.get(index) else {
                        break;
                    };
                    if sender.send((index, group(ledger))).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
    });
    let mut results = (0..GROUPS.len()).map(|_| None).collect::<Vec<_>>();
    for (index, result) in receiver {
        results[index] = Some(result);
    }
    let mut count = 0_usize;
    for (index, result) in results.into_iter().enumerate() {
        let result = result.ok_or_else(|| {
            LedgerError::new(format!(
                "structural-control worker returned no result for {}",
                GROUPS[index].0
            ))
        })?;
        count += result.map_err(|error| {
            LedgerError::new(format!(
                "{} structural-control group failed: {error}",
                GROUPS[index].0
            ))
        })?;
    }
    if count != STRUCTURAL_CONTROL_COUNT {
        return Err(LedgerError::new(format!(
            "active structural-control count drifted: expected {STRUCTURAL_CONTROL_COUNT}, executed {count}"
        )));
    }
    Ok(count)
}

pub(crate) fn load_and_validate(context: &Context) -> Result<ValidatedLedger, Error> {
    let mut immutable_snapshot = ImmutableRepositoryInputs::new(context.root())?;
    let inputs = load_static_inputs_snapshotted(context, &mut immutable_snapshot)
        .map_err(|error| ledger_error(error.to_string()))?;
    let source_bytes = input_bytes(&inputs, SOURCE)
        .map_err(|error| ledger_error(error.to_string()))?
        .to_vec();
    let source = parse_source(&source_bytes).map_err(|error| ledger_error(error.to_string()))?;
    let sibling_projections =
        SiblingProjections::parse(&inputs).map_err(|error| ledger_error(error.to_string()))?;
    let reader_projection = load_reader_projection(context, &inputs)
        .map_err(|error| ledger_error(error.to_string()))?;
    let resolutions = validate_source_with_projections(
        &inputs,
        &source,
        &sibling_projections,
        &reader_projection,
    )
    .map_err(|error| ledger_error(error.to_string()))?;
    validate_review_history(context, &source).map_err(|error| ledger_error(error.to_string()))?;
    validate_current_audit_receipts(context, &inputs, &source)
        .map_err(|error| ledger_error(error.to_string()))?;
    Ok(ValidatedLedger {
        document: source,
        source_bytes,
        resolutions,
        input_bytes: inputs,
        sibling_projections,
        reader_projection,
        immutable_snapshot: Some(Mutex::new(immutable_snapshot)),
    })
}

pub(crate) fn closure_projection(context: &Context) -> Result<ClosureProjection, Error> {
    load_and_validate(context)?
        .closure()
        .cloned()
        .ok_or_else(|| ledger_error("closure_record is null"))
}

pub(crate) fn protected_claim_refs(context: &Context) -> Result<Vec<String>, Error> {
    load_and_validate(context)?
        .protected_claim_refs_inner()
        .map_err(|error| ledger_error(error.to_string()))
}

pub(crate) fn protected_claim_refs_from_validated(
    ledger: &ValidatedLedger,
) -> Result<Vec<String>, Error> {
    ledger
        .protected_claim_refs_inner()
        .map_err(|error| ledger_error(error.to_string()))
}

pub(crate) fn protected_claim_refs_from_source(
    context: &Context,
    source: &str,
) -> Result<Vec<String>, Error> {
    let document =
        parse_source(source.as_bytes()).map_err(|error| ledger_error(error.to_string()))?;
    let input_bytes =
        load_static_inputs(context).map_err(|error| ledger_error(error.to_string()))?;
    let sibling_projections =
        SiblingProjections::parse(&input_bytes).map_err(|error| ledger_error(error.to_string()))?;
    let reader_projection = load_reader_projection(context, &input_bytes)
        .map_err(|error| ledger_error(error.to_string()))?;
    let resolutions = validate_source_with_projections(
        &input_bytes,
        &document,
        &sibling_projections,
        &reader_projection,
    )
    .map_err(|error| ledger_error(error.to_string()))?;
    validate_current_audit_receipts(context, &input_bytes, &document)
        .map_err(|error| ledger_error(error.to_string()))?;
    ValidatedLedger {
        document,
        source_bytes: source.as_bytes().to_vec(),
        resolutions,
        input_bytes,
        sibling_projections,
        reader_projection,
        immutable_snapshot: None,
    }
    .protected_claim_refs_inner()
    .map_err(|error| ledger_error(error.to_string()))
}

fn canonical_digest<T: Serialize>(value: &T) -> LedgerResult<String> {
    let value = serde_json::to_value(value)
        .map_err(|error| LedgerError::new(format!("cannot canonicalize JSON: {error}")))?;
    Ok(sha256(canonical_json(&value)))
}

fn population_line<T: Serialize>(
    name: &str,
    rows: &[T],
    identities: Vec<&str>,
) -> LedgerResult<String> {
    let identity_text = if identities.is_empty() {
        if rows.is_empty() {
            "empty".to_owned()
        } else {
            "unkeyed rows; digest is authoritative".to_owned()
        }
    } else {
        identities.join(", ")
    };
    Ok(format!(
        "| `{name}` | {} | `{}` | {identity_text} |",
        rows.len(),
        canonical_digest(&rows)?
    ))
}

fn reader_population_lines(source: &LedgerDocument) -> LedgerResult<Vec<String>> {
    macro_rules! keyed {
        ($field:ident) => {
            population_line(
                stringify!($field),
                &source.$field,
                source.$field.iter().map(|row| row.id.as_str()).collect(),
            )?
        };
    }
    macro_rules! unkeyed {
        ($field:ident) => {
            population_line(stringify!($field), &source.$field, Vec::new())?
        };
    }
    Ok(vec![
        keyed!(axes),
        unkeyed!(compatibility_table),
        unkeyed!(enum_mapping),
        unkeyed!(enum_mapping_exclusions),
        unkeyed!(residual_coverage_exclusions),
        keyed!(domains),
        keyed!(legacy_rows),
        keyed!(claims),
        keyed!(bodies),
        keyed!(routes),
        keyed!(external_assumptions),
        keyed!(envelope),
        keyed!(roles),
        unkeyed!(role_omissions),
        keyed!(powers),
        unkeyed!(economic_power_rule_contracts),
        unkeyed!(economic_carry_rule_contracts),
        unkeyed!(economic_acceptance_cases),
        keyed!(power_contract_templates),
        keyed!(power_refusals),
        keyed!(power_crosswalk_dispositions),
        keyed!(constitutional_effects),
        keyed!(coverage_families),
        keyed!(dependencies),
        keyed!(dependency_loops),
        unkeyed!(refused_flows),
        keyed!(scenarios),
        unkeyed!(scenario_omissions),
        keyed!(thresholds),
        keyed!(defects),
        keyed!(receipts),
        keyed!(review_commissions),
        keyed!(proposals),
        keyed!(review_events),
        keyed!(scope_audits),
        unkeyed!(deferred_populations),
        keyed!(closure_requirement_profiles),
        keyed!(closure_claim_contracts),
        keyed!(model_allocations),
        keyed!(function_allocations),
        keyed!(loop_hazard_controls),
        keyed!(bottleneck_dispositions),
    ])
}

fn bucket_cell(bucket: &DomainBucket) -> String {
    match bucket {
        DomainBucket::Answer(value) => value.answer.clone(),
        DomainBucket::Routing(value) => {
            format!("*{}* — {}", value.routing_marker, value.note)
        }
        DomainBucket::Unresolved(value) => format!(
            "**Unresolved** — severity: {}; consequence: {}; closure: {}; public-claim limitation: {}",
            value.unresolved.severity,
            value.unresolved.consequence,
            value.unresolved.closure_condition,
            value.unresolved.public_claim_limitation
        ),
    }
}

fn markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\n', '\r'], " ")
}

fn severity_class(defect: &Defect) -> LedgerResult<&str> {
    for class in ["critical", "material", "minor"] {
        if defect
            .severity
            .strip_prefix(class)
            .is_some_and(|suffix| suffix.starts_with(" — "))
        {
            return Ok(class);
        }
    }
    Err(LedgerError::new(format!(
        "defect {}: severity must carry a class prefix (critical / material / minor) followed by ' — ' and prose",
        defect.id
    )))
}

/// This is intentionally the canonical-JSON boundary: the semantic document
/// has already passed strict typed validation, and `Value` is used only to
/// construct the checker-defined digest projection without duplicating the
/// entire source schema as a second Rust type.
fn review_scope_digest(source: &LedgerDocument) -> LedgerResult<String> {
    let mut value = serde_json::to_value(source)
        .map_err(|error| LedgerError::new(format!("cannot serialize review scope: {error}")))?;
    let root = value
        .as_object_mut()
        .ok_or_else(|| LedgerError::new("serialized ledger root is not an object"))?;
    for key in [
        "review_protocol",
        "review_commissions",
        "proposals",
        "review_events",
        "scope_audits",
        "deferred_populations",
        "closure_record",
        "acceptance_gate",
    ] {
        root.remove(key);
    }
    let routes = root
        .get_mut("routes")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| LedgerError::new("serialized review scope has no routes array"))?;
    let review_route = routes
        .iter_mut()
        .find(|route| route.get("id").and_then(Value::as_str) == Some("FS-RTE-07"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| LedgerError::new("serialized review scope has no FS-RTE-07 route"))?;
    for key in [
        "status",
        "route_status",
        "consequence",
        "closure_condition",
        "negative_control",
    ] {
        review_route.remove(key);
    }
    Ok(sha256(canonical_json(&value)))
}

/// Print the candidate semantic-scope digest without requiring the current
/// audit row to have caught up with the reviewed source.
///
/// The source still crosses the duplicate-key preflight and exact typed ledger
/// contract. Deliberately stop before sibling, history, and current-audit
/// validation so this read-only mode remains usable while preparing a new
/// source-version audit record.
pub(crate) fn fingerprints(context: &Context) -> Result<String, Error> {
    let loaded = load_source(context).map_err(|error| ledger_error(error.to_string()))?;
    let output = ScopeFingerprintOutput {
        source_version: loaded.source.source_version.clone(),
        scope_sha256: review_scope_digest(&loaded.source)
            .map_err(|error| ledger_error(error.to_string()))?,
    };
    serde_json::to_string_pretty(&output)
        .map_err(|error| ledger_error(format!("cannot serialize scope fingerprint: {error}")))
}

fn protocol_digest(inputs: &BTreeMap<String, Vec<u8>>) -> LedgerResult<String> {
    Ok(sha256(input_bytes(inputs, PROTOCOL_DOC)?))
}

fn qualifying_scope_audits<'a>(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &'a LedgerDocument,
) -> LedgerResult<Vec<&'a ScopeAudit>> {
    let scope = review_scope_digest(source)?;
    let protocol = protocol_digest(inputs)?;
    Ok(source
        .scope_audits
        .iter()
        .filter(|row| {
            row.source_version == source.source_version
                && row.scope_sha256 == scope
                && row.protocol_sha256 == protocol
                && row.result == "passed-with-recorded-limits"
        })
        .collect())
}

fn qualifying_review_events<'a>(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &'a LedgerDocument,
) -> LedgerResult<Vec<&'a ReviewEvent>> {
    let scope = review_scope_digest(source)?;
    let protocol = protocol_digest(inputs)?;
    Ok(source
        .review_events
        .iter()
        .filter(|row| {
            row.outcome_status == "passed"
                && row.source_version == source.source_version
                && row.scope_sha256 == scope
                && row.protocol_sha256 == protocol
        })
        .collect())
}

type ReadinessRow = (String, &'static str, String);

fn compute_gate_a_readiness(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
) -> LedgerResult<(Vec<ReadinessRow>, Vec<ReadinessRow>)> {
    let conditions = &source.stopping_rule.closure_conditions;
    if conditions.len() < 5 {
        return Err(LedgerError::new(
            "stopping_rule must provide five Gate A closure conditions",
        ));
    }
    let gated_populations = [
        "domains",
        "roles",
        "powers",
        "dependencies",
        "scenarios",
        "defects",
        "source-specific-power-contracts",
    ];
    let mut deferred = source
        .deferred_populations
        .iter()
        .filter(|row| gated_populations.contains(&row.record_type.as_str()))
        .map(|row| row.record_type.as_str())
        .collect::<Vec<_>>();
    deferred.sort_unstable();
    let mut blocking = source
        .defects
        .iter()
        .filter(|row| {
            row.applicable_gate_refs.iter().any(|gate| gate == "gate-a")
                && resolution
                    .get(&row.id)
                    .is_some_and(|generated| generated.blocking)
        })
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>();
    blocking.sort_unstable();
    let audits = qualifying_scope_audits(inputs, source)?;
    let mut rows = Vec::with_capacity(5);
    if deferred.is_empty() {
        rows.push((
            conditions[0].clone(),
            "met-in-form",
            "every record type is populated or classified out; material sufficiency stays a review question".into(),
        ));
    } else {
        rows.push((
            conditions[0].clone(),
            "unmet",
            format!(
                "record types remain deferred with owners: {}",
                deferred.join(", ")
            ),
        ));
    }
    rows.push((
        conditions[1].clone(),
        "met-in-form",
        "the coverage, role, dependency, assurance-allocation, structural-reader, and Book 2 projections regenerate from the canonical source; projection freshness establishes no reader evidence or operational result".into(),
    ));
    if blocking.is_empty() {
        rows.push((
            conditions[2].clone(),
            "met-mechanically",
            "no critical unresolved defect row is applicable to Gate A's map-and-test-program claim; later-gate claim blockers remain visible and unresolved".into(),
        ));
    } else {
        rows.push((
            conditions[2].clone(),
            "unmet",
            format!(
                "critical unresolved defects applicable to Gate A's map-and-test-program claim exist: {}",
                blocking.join(", ")
            ),
        ));
    }
    rows.push((
        conditions[3].clone(),
        "met-in-form",
        "severity, consequence, owner, closure condition, and public-claim limitation are validator-enforced on every unresolved object; substance is reviewed, not proven".into(),
    ));
    rows.push(if audits.is_empty() {
        (
            conditions[4].clone(),
            "unmet",
            "no current-source repository adversarial audit exists".into(),
        )
    } else {
        (
            conditions[4].clone(),
            "met-mechanically",
            "a current-source repository adversarial audit covers the declared criteria, exact checker controls, command chain, and every Gate-A-applicable defect disposition".into(),
        )
    });

    let mut preconditions = Vec::new();
    if source
        .envelope
        .iter()
        .skip(1)
        .any(|row| row.envelope_status == "versioned-structure")
    {
        preconditions.push((
            "the reference envelope".into(),
            "met-in-form",
            "versioned in structure and reviewable; this satisfies Gate A's envelope precondition. Calibration and values remain Book 2 Gate D work, and operational assurance and remedied resolution still require them".into(),
        ));
    } else {
        preconditions.push((
            "the reference envelope".into(),
            "unmet-external",
            "still the explicit stub; Gate A requires a non-stub, versioned-structure envelope"
                .into(),
        ));
    }
    if source.severity_rubric.rubric_status == "candidate" {
        preconditions.push((
            "the severity rubric".into(),
            "unmet",
            "candidate — author confirmation pending".into(),
        ));
    }
    Ok((rows, preconditions))
}

fn body_status_senses(body: &Body) -> [(&'static str, &Term); 7] {
    [
        (
            "universal_human_standing",
            &body.status_senses.universal_human_standing,
        ),
        (
            "political_membership",
            &body.status_senses.political_membership,
        ),
        ("franchise", &body.status_senses.franchise),
        ("candidacy", &body.status_senses.candidacy),
        ("current_office", &body.status_senses.current_office),
        (
            "current_lawful_power",
            &body.status_senses.current_lawful_power,
        ),
        (
            "permanent_historical_public_answerability",
            &body.status_senses.permanent_historical_public_answerability,
        ),
    ]
}

fn body_office_terms(body: &Body) -> [(&'static str, &Term); 11] {
    [
        ("democratic_source", &body.office_contract.democratic_source),
        ("jurisdiction", &body.office_contract.jurisdiction),
        ("ordinary_function", &body.office_contract.ordinary_function),
        (
            "delegation_boundary",
            &body.office_contract.delegation_boundary,
        ),
        (
            "conflict_and_recusal",
            &body.office_contract.conflict_and_recusal,
        ),
        ("appointment", &body.office_contract.appointment),
        ("removal", &body.office_contract.removal),
        ("succession", &body.office_contract.succession),
        ("temporal_status", &body.office_contract.temporal_status),
        (
            "public_reason_duty",
            &body.office_contract.public_reason_duty,
        ),
        ("anti_capture", &body.office_contract.anti_capture),
    ]
}

fn render_report(
    inputs: &BTreeMap<String, Vec<u8>>,
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
) -> LedgerResult<String> {
    let mut output = Vec::<String>::new();
    macro_rules! w {
        () => {
            output.push(String::new())
        };
        ($format:literal, $($argument:expr),+ $(,)?) => {
            output.push(format!($format, $($argument),+))
        };
        ($value:expr) => {
            output.push(($value).to_string())
        };
    }

    let mut blocked_by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for defect in &source.defects {
        if resolution
            .get(&defect.id)
            .is_some_and(|generated| generated.blocking)
        {
            blocked_by
                .entry(defect.affected_claim_ref.as_str())
                .or_default()
                .push(defect.id.as_str());
        }
    }

    w!("<!-- SPDX-License-Identifier: CC-BY-4.0 -->");
    w!("<!-- Generated by the native rights-verify ledger refresh; do not edit. -->");
    w!();
    w!("# Full-Society Domain-and-Layer Ledger — Generated Report");
    w!();
    w!("**{}**", source.acceptance_gate.verdict);
    w!();
    w!(
        "Reviewed source: `new-book-plans/full-society-ledger.json` (source version `{}`; {}). This report is a projection of the canonical source. Classification is routing, not assurance: every row establishes at most its own posture, and no count here is an assurance figure.",
        source.source_version,
        STAGE_LABEL
    );
    w!();
    w!("## Declared axes and stopping rule");
    w!();
    for axis in &source.axes {
        w!(
            "- **{}** (`{}`): {} — {}",
            axis.name,
            axis.id,
            axis.values,
            axis.note
        );
    }
    w!();
    let rule = &source.stopping_rule;
    w!("A version may close only when all of the following hold for the gate's permitted claim:");
    w!();
    for (index, condition) in rule.closure_conditions.iter().enumerate() {
        w!("{}. {}", index + 1, condition);
    }
    w!();
    w!("**Materiality:** {}", rule.materiality_test);
    w!();
    w!("**Boundary:** {}", rule.boundary);
    w!();
    w!("**No hiding:** {}", rule.no_hiding_rule);
    w!();
    w!("## Gate A readiness (computed)");
    w!();
    w!(
        "Per-condition status, generated from the source and echoing the closure conditions above by index; no aggregate is derived from this list, and a closure record is refused while any row computes unmet. Gate A passes only when a mechanical closure record binds an immutable, fully verified candidate with no semantic drift."
    );
    w!();
    let (readiness, preconditions) = compute_gate_a_readiness(inputs, source, resolution)?;
    for (name, status, reason) in readiness {
        w!("- **{}** — {}: {}", status, name, reason);
    }
    for (name, status, reason) in preconditions {
        w!("- **{}** (precondition) — {}: {}", status, name, reason);
    }
    w!();
    w!("## The five layers");
    w!();
    w!(
        "The five routing dispositions are the reader-facing five layers (author-ratified 2026-08-09). One enum, one key: `scope_disposition`; a leaf record carries exactly one value and a domain record spans all five as its buckets."
    );
    w!();
    w!("| Layer | Meaning |");
    w!("| --- | --- |");
    for value in SCOPE_DISPOSITIONS {
        w!(
            "| `{}` | {} |",
            value,
            source.scope_disposition_meanings[value]
        );
    }
    w!();
    w!("## Domains");
    w!();
    for row in &source.domains {
        w!("### {} — {}", row.id, row.title);
        w!();
        w!(
            "- Status: {}; applicability: {}; classes: {}",
            row.status,
            row.applicability,
            row.class_refs.join(", ")
        );
        w!(
            "- Constitutional invariants: {}",
            bucket_cell(&row.constitutional_invariants)
        );
        w!(
            "- Democratic / ordinary-law choices: {}",
            bucket_cell(&row.ordinary_law_choices)
        );
        w!(
            "- Protected private/civic freedom: {}",
            bucket_cell(&row.protected_private_civic)
        );
        w!(
            "- Book 2 operations: {}",
            bucket_cell(&row.book2_operations)
        );
        w!(
            "- External assumptions: {}",
            bucket_cell(&row.external_assumptions_note)
        );
        w!(
            "- Bodies: {}; legacy rows: {}",
            if row.bodies_refs.is_empty() {
                "none named yet".into()
            } else {
                row.bodies_refs.join(", ")
            },
            if row.legacy_row_refs.is_empty() {
                "none".into()
            } else {
                row.legacy_row_refs.join(", ")
            }
        );
        let scenario_applicability = match &row.scenario_applicability {
            ScenarioApplicability::Answer(value) => value.answer.as_str(),
            ScenarioApplicability::Deferred(value) => {
                w!(
                    "- Scenario applicability: deferred — {}",
                    value.deferred_ref
                );
                ""
            }
        };
        if !scenario_applicability.is_empty() {
            w!("- Scenario applicability: {}", scenario_applicability);
        }
        w!("- Reader destination: {}", row.reader_destination);
        w!(
            "- Severity if left open: {}; consequence: {}; closure: {}",
            row.severity,
            row.consequence,
            row.closure_condition
        );
        w!();
    }
    w!("## Roles, life-course stages, scales, and power positions");
    w!();
    w!(
        "Each role records the standing of a person in a position — life-course stages are roles of a kind — and routes it against domains, scales, and the ratified bodies. A role is never a floor-changing status: one person occupies many roles and none buys a higher floor or a lower one, which is why every role's layer is the constitutional invariant of universal standing; rule content stays on domains and claims. Axis coverage is mechanical — every domain cited, every named scale exercised, every required body carrying both an affected and a checking role position, every recorded private power naming its affected counter-roles and its checkers — while pairwise sufficiency is tested only against the declared source-derived audit criteria; no full Cartesian product is attempted, and deliberately omitted candidates and combinations are recorded below with risk-based reasons. The FS-POW decomposition of each power is staged below by exact source-family prefix and remains deferred until the complete population. Formal anchors stay honest: a derived constitution predicate, an asserted predicate with its replace-card path, or ratified-but-unimplemented doctrine."
    );
    w!();
    w!("| Role | Kind | Domains | Scales | Affected by | Checks | Anchor |");
    w!("| --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.roles {
        let mut affected = Vec::new();
        let mut checks = Vec::new();
        for position in &row.power_positions {
            if position.position == "affected" {
                affected.push(position.body_ref.as_str());
            } else {
                checks.push(position.body_ref.as_str());
            }
        }
        w!(
            "| {} {} | {} | {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.role_kind,
            row.domain_refs.join(", "),
            row.scales.join(", "),
            if affected.is_empty() {
                "—".into()
            } else {
                affected.join(", ")
            },
            if checks.is_empty() {
                "—".into()
            } else {
                checks.join(", ")
            },
            row.formal_anchor.anchor
        );
    }
    w!();
    w!(
        "Recorded private and delegated powers (the holder's own record names who stands under the power and who checks it):"
    );
    w!();
    for row in &source.roles {
        if let Some(power) = &row.power_held {
            w!(
                "- `{}` holds: {} Affected: {}; checked by: {}.",
                row.id,
                power.power,
                power.affected_role_refs.join(", "),
                power.checking_refs.join(", ")
            );
        }
    }
    w!();
    w!("Deliberately omitted candidates and combinations (recorded, not silent):");
    w!();
    for omission in &source.role_omissions {
        match omission {
            RoleOmission::Role(row) => {
                w!("- No role for {}: {}", row.omitted_role, row.risk_reason);
            }
            RoleOmission::Scale(row) => {
                w!(
                    "- `{}` omits `{}`: {}",
                    row.role_ref,
                    row.omitted_scale,
                    row.risk_reason
                );
            }
            RoleOmission::Domain(row) => {
                w!(
                    "- `{}` omits `{}`: {}",
                    row.role_ref,
                    row.omitted_domain_ref,
                    row.risk_reason
                );
            }
        }
    }
    w!();
    w!("## Source-derived power contracts and function allocations");
    w!();
    let population = &source.power_population;
    let coverage = &source.coverage_population;
    w!(
        "Power population status: **{}**. Coverage-contract status: **{}**. Completed coverage prefix: {}.",
        population.status,
        coverage.status,
        if coverage.completed_source_families.is_empty() {
            "none".into()
        } else {
            coverage.completed_source_families.join(", ")
        }
    );
    w!();
    w!(
        "Current rows: {} FS-POW cards; {} FS-PCT templates; {} FS-PRF refusals/limits; {} FS-PCD formal dispositions; {} FS-FAL allocations.",
        source.powers.len(),
        source.power_contract_templates.len(),
        source.power_refusals.len(),
        source.power_crosswalk_dispositions.len(),
        source.function_allocations.len()
    );
    w!();
    w!("Evidence ceiling: {}", coverage.evidence_ceiling);
    w!();
    w!(
        "| Power | Manifest grain | Class / profiles | Claims / domains | Contract readiness | Tests | Part V / Book 2 boundary |"
    );
    w!("| --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.powers {
        let summary = row
            .contract_terms
            .get("bounded_effect")
            .ok_or_else(|| LedgerError::new(format!("{} lacks bounded_effect", row.id)))?;
        w!(
            "| {} {} | {} | {}; {} | {}; {} | coverage-ready — {} | {}/{} | {}; {} |",
            row.id,
            row.title,
            row.manifest_key,
            row.primary_class_ref,
            row.profiles.join(", "),
            row.affected_claim_refs.join(", "),
            row.domain_refs.join(", "),
            summary.text,
            row.negative_test.status,
            row.counterfactual.status,
            row.part_v_status,
            row.book2_handoff
        );
    }
    if source.powers.is_empty() {
        w!("| — | no completed source family | — | — | — | — | — |");
    }
    w!();
    w!("Constitutional non-power effects:");
    w!();
    w!("| Effect | Class / profiles | Claims / domains | Readiness | Boundary |");
    w!("| --- | --- | --- | --- | --- |");
    for row in &source.constitutional_effects {
        w!(
            "| {} {} | {}; {} | {}; {} | {}; {}/{} | {} |",
            row.id,
            row.title,
            row.primary_class_ref,
            row.profiles.join(", "),
            row.affected_claim_refs.join(", "),
            row.domain_refs.join(", "),
            row.part_v_status,
            row.negative_test.status,
            row.counterfactual.status,
            row.book2_handoff
        );
    }
    w!();
    w!("Economic non-power carry contracts:");
    w!();
    w!(
        "| Carry | Current → result | Temporal / predecessor contract | Bounded interest / requirement |"
    );
    w!("| --- | --- | --- | --- |");
    for row in &source.economic_carry_rule_contracts {
        w!(
            "| {} | `{}` → `{}` | `{}`; `{}` / `{}` / `{}` | `{}` @ `{}`; `{}` @ `{}` |",
            row.carry_kind,
            row.current_kind,
            row.result_kind,
            row.temporal_contract,
            row.predecessor_record_scope,
            row.predecessor_result_scope,
            row.successor_event_scope,
            row.interest.value,
            row.interest.scope,
            row.requirement.value,
            row.requirement.scope
        );
    }
    w!();
    w!("### Reviewed §15 economic acceptance matrix");
    w!();
    w!(
        "This is a digest-bound structural and executable mapping of the 24 reviewed acceptance cases. A TRUE or FALSE pin result establishes only the exact repository query under the current source and engine; it does not establish external truth, delivery, institutional operation, liveness, feasibility, reader comprehension, or any Book 2 model. Composite variants retain every separately owned support instead of treating one pin as proof of the whole assertion."
    );
    w!();
    w!("| Case / reviewed source grain | Atomic variant | Exact owned executable supports |");
    w!("| --- | --- | --- |");
    for case in &source.economic_acceptance_cases {
        for mapping in &case.mappings {
            let supports = mapping
                .supports
                .iter()
                .map(|support| {
                    format!(
                        "`{}:{}` {} — `{}` → `{}` = **{}**",
                        support.owner_kind,
                        support.owner_id,
                        support.polarity,
                        support.pin_ref,
                        support.query,
                        support.expected_result
                    )
                })
                .collect::<Vec<_>>()
                .join("<br>");
            w!(
                "| {} — {} | `{}` / `{}` — {} | {} |",
                case.case_id,
                markdown_table_cell(&case.source_needle),
                mapping.variant_id,
                mapping.mapping_id,
                markdown_table_cell(&mapping.assertion),
                markdown_table_cell(&supports)
            );
        }
    }
    w!();
    w!("Coverage-family drafting gate:");
    w!();
    w!("| Family | State | Powers / effects | Formal statements | Drafting block |");
    w!("| --- | --- | ---: | ---: | --- |");
    for row in &source.coverage_families {
        w!(
            "| {} {} | {} | {} / {} | {} | {} |",
            row.id,
            row.title,
            row.state,
            row.card_refs.len(),
            row.effect_refs.len(),
            row.formal_statement_refs.len(),
            row.blocked_before_drafting
        );
    }
    w!();
    w!("Contract templates:");
    w!();
    for row in &source.power_contract_templates {
        w!(
            "- `{}` / `{}`: {}. {}",
            row.id,
            row.manifest_key,
            row.title,
            row.closure_condition
        );
    }
    if source.power_contract_templates.is_empty() {
        w!("- None in the completed prefix.");
    }
    w!();
    w!("Refusals and formal transitions:");
    w!();
    for row in &source.power_refusals {
        w!(
            "- `{}` / `{}`: {} Non-authorisation: {}",
            row.id,
            row.manifest_key,
            row.refusal,
            row.non_authorisation
        );
    }
    for row in &source.power_crosswalk_dispositions {
        w!(
            "- `{}` / `{}`: `{}` → {}.",
            row.id,
            row.manifest_key,
            row.crosswalk_action,
            if row.target_power_refs.is_empty() {
                "no successor".into()
            } else {
                row.target_power_refs.join(", ")
            }
        );
    }
    if source.power_refusals.is_empty() && source.power_crosswalk_dispositions.is_empty() {
        w!("- None in the completed prefix.");
    }
    w!();
    w!(
        "Function allocations (body functions; role references identify the corresponding position classes, not proof of staffing or independence):"
    );
    w!();
    for row in &source.function_allocations {
        w!(
            "- `{}` → `{}`; writer {}; decider {}; executor {}; auditor {}; final remedy {}.",
            row.id,
            row.power_ref,
            row.decisive_fact_writer_body_refs.join(", "),
            row.decider_body_refs.join(", "),
            row.executor_body_refs.join(", "),
            row.auditor_body_refs.join(", "),
            row.final_remedy_body_refs.join(", ")
        );
    }
    if source.function_allocations.is_empty() {
        w!("- None in the completed prefix.");
    }
    w!();
    w!("## Functional flows and cross-domain dependencies");
    w!();
    w!(
        "Each edge records that a function depends on a flow — its lawful source class, its owner, and what breaks when the flow stops. An edge never records that the flow arrives: no right is called delivered because an institution promised it, and no body is called functional because its name exists. The four-way class is routing, not assurance — constitutionally-guaranteed names the lawful source of the obligation, never a delivery status, and an externally-assumed edge names a premise nothing internal manufactures. The mechanical cycle check establishes exactly one thing: every strongly connected region of the declared graph carries at least one declared, classified, owner-named loop witness with a recorded boundedness statement. Boundedness is reviewed prose, not a proven property. The closure audit publishes self-certifying, deadlocking, single-veto, unbounded, bottleneck, and cascade hazards as bounded-unresolved or scoped blocking; it admits no rejected-by-control result until a route-bound executable-control receipt schema lands. Alternate routes are predeclared with their doctrine needle or their absence is recorded as a named single point of failure. Refused flows are walls, not edges: doctrine forbids them, and drawing one as a dependency would be the defect."
    );
    w!();
    w!("| Edge | Flow | Class | Source → Destination | Path | Alternate | Owner |");
    w!("| --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.dependencies {
        let alternate = match &row.alternate_route {
            AlternateRoute::Present(_) => "declared",
            AlternateRoute::Absent(_) => "none — recorded",
        };
        w!(
            "| {} {} | {} | {} | {} → {} | {} | {} | {} |",
            row.id,
            row.title,
            row.flow_kind,
            row.dependency_class,
            row.from_ref,
            row.to_ref,
            row.lifecycle_path,
            alternate,
            row.steward_ref
        );
    }
    w!();
    w!(
        "Per-edge routing (absence, continuity, remedy, restoration, correction — routing statements, never delivery):"
    );
    w!();
    for row in &source.dependencies {
        w!(
            "- `{}` absence: {} Continuity: {} Remedy: {} Restoration: {} Correction: {}",
            row.id,
            row.consequence,
            row.interim_continuity,
            row.remedy_route,
            row.restoration,
            row.systemic_correction
        );
    }
    w!();
    w!("Single points of failure (no alternate route, recorded):");
    w!();
    for row in &source.dependencies {
        if let AlternateRoute::Absent(alternate) = &row.alternate_route {
            w!(
                "- `{}` {}: {}",
                row.id,
                row.title,
                alternate.no_alternate_reason
            );
        }
    }
    w!();
    w!("Declared loops (classified, bounded, owned):");
    w!();
    for row in &source.dependency_loops {
        w!(
            "- `{}` {} loop (steward {}): {} — bounded: {}",
            row.id,
            row.loop_kind,
            row.steward_ref,
            row.member_edge_refs.join(" → "),
            row.boundedness
        );
    }
    w!();
    w!("Refused flows (walls, not edges):");
    w!();
    for row in &source.refused_flows {
        w!(
            "- {} [{}]: {}",
            row.refused_flow,
            row.flow_kind,
            row.refusal_reason
        );
    }
    w!();
    w!("## Whole-society journeys, collisions, and stress cases");
    w!();
    w!(
        "The scenario catalogue is reviewed inventory — a reviewed threat model, never proof and never a counterexample harness. Each record routes an owned ordinary, failure, and recovery path: the failure route carries interim continuity, the recovery route carries remedy and restoration together, and a route is routing, never delivery. Nothing here executed — constitutional cases execute only after the relevant author rulings and contract cards land, and the closure audit consumes this population. The kinds, collision axes, compound shocks, and protected-sphere tests are closed vocabularies; every domain is reached and every critical dependency edge is stressed or its omission recorded. Shock records state Book 1 invariant and failure behaviour only — capacity and degradation are Book 2's tests. Protected-sphere scenarios test freedom without permission, non-recording and non-compulsion, the narrow evidenced-harm threshold, and recourse against interference — never a state-defined successful life outcome. A bounded witness names a sibling case and establishes only what that artifact's own posture states."
    );
    w!();
    w!("| Scenario | Kind | Domains | Edges | Steward | Axis / shock |");
    w!("| --- | --- | --- | --- | --- | --- |");
    for row in &source.scenarios {
        let axis = row
            .collision_axis
            .as_deref()
            .or(row.shock_kind.as_deref())
            .unwrap_or("—");
        w!(
            "| {} {} | {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.scenario_kind,
            row.domain_refs.join(", "),
            if row.dependency_refs.is_empty() {
                "—".into()
            } else {
                row.dependency_refs.join(", ")
            },
            row.steward_ref,
            axis
        );
    }
    w!();
    w!("Per-scenario routes (routing statements, never delivery or execution):");
    w!();
    for row in &source.scenarios {
        w!(
            "- `{}` ordinary: {} Failure: {} Recovery: {}",
            row.id,
            row.ordinary_route,
            row.failure_route,
            row.recovery_route
        );
        if let Some(forms) = &row.protected_sphere_forms {
            if !forms.is_empty() {
                w!("  - protected-sphere tests: {}", forms.join(", "));
            }
        }
        if let Some(witnesses) = &row.bounded_witness_refs {
            if !witnesses.is_empty() {
                w!("  - bounded sibling witnesses: {}", witnesses.join(", "));
            }
        }
    }
    w!();
    w!("Deliberately omitted scenario candidates (recorded, not silent):");
    w!();
    for row in &source.scenario_omissions {
        match row {
            ScenarioOmission::Scenario(value) => {
                w!("- {}: {}", value.omitted_scenario, value.risk_reason);
            }
            ScenarioOmission::Dependency(value) => {
                w!(
                    "- dependency {}: {}",
                    value.omitted_dependency_ref,
                    value.risk_reason
                );
            }
        }
    }
    w!();
    w!("## Legacy coverage rows and their splits");
    w!();
    w!(
        "Imported from the coverage map with wording frozen; each split claim carries exactly one posture per the ratified legend."
    );
    w!();
    w!("| Row | Legacy status (frozen) | Split state | Claims |");
    w!("| --- | --- | --- | --- |");
    for row in &source.legacy_rows {
        w!(
            "| {} {} | {} | {} | {} |",
            row.id,
            row.domain_title,
            row.legacy_status,
            row.split_state,
            if row.split_claim_refs.is_empty() {
                "—".into()
            } else {
                row.split_claim_refs.join(", ")
            }
        );
    }
    w!();
    w!("## Claims (one posture each)");
    w!();
    w!("| Claim | Layer | Posture | Route | Overlay | Scope bound | Blocked by |");
    w!("| --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.claims {
        let mut posture = row.posture.clone();
        if row.posture == "Unestablished" {
            if let Some(disposition) = &row.unestablished_disposition {
                posture.push('/');
                posture.push_str(disposition);
            }
        }
        if let Some(kind) = &row.evidence_kind {
            posture.push_str(" (");
            posture.push_str(kind);
            posture.push(')');
        }
        let blockers = blocked_by
            .get(row.id.as_str())
            .map(|ids| ids.join(", "))
            .unwrap_or_else(|| "—".into());
        w!(
            "| {} {} | {} | {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.layer,
            posture,
            row.route_ref,
            row.overlay,
            row.scope_bound,
            blockers
        );
    }
    w!();
    w!("## Required bodies");
    w!();
    w!("| Body | Kind | Constitutional job | May not do alone | Required check / remedy |");
    w!("| --- | --- | --- | --- | --- |");
    for row in &source.bodies {
        w!(
            "| {} {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.body_kind,
            row.job,
            row.may_not_do_alone,
            row.required_check
        );
    }
    w!();
    w!("### Body contracts");
    w!();
    w!(
        "Each card separates the seven status senses the state-form ruling refused to let one word carry, states the office contract, names who checks the body, and lists the individualized adverse determinations it can make. A determination carries an appeal and a remedy; a body that makes none carries neither, which is how the ruling's refusal to recreate a universal right of appeal is held mechanically rather than promised. A reserved choice appears as a bounded delegation with its owner, bounds, and failure default — never as an invented number. Every card stays ratified-unimplemented: this is what a body is constitutionally obliged to do and what withholds its conclusions, not a record that any body exists, is staffed, is independent in fact, or has ever acted."
    );
    w!();
    for row in &source.bodies {
        w!("#### {} — {} ({})", row.id, row.title, row.body_kind);
        w!();
        w!("- Applicability: {}", row.applicability);
        w!(
            "- Status: {}; severity {}; consequence: {}",
            row.status,
            row.severity,
            row.consequence
        );
        w!(
            "- Owner: `{}`; closure: {}",
            row.owner_ref,
            row.closure_condition
        );
        w!("- Status senses:");
        for (name, term) in body_status_senses(row) {
            w!("    - *{}* — {}", name.replace('_', " "), term.text);
        }
        w!("- Office contract:");
        for (name, term) in body_office_terms(row) {
            w!("    - *{}* — {}", name.replace('_', " "), term.text);
        }
        w!("- Accountability routes:");
        for route in &row.accountability_routes {
            let checkers = route
                .checker_body_refs
                .iter()
                .chain(&route.checker_role_refs)
                .map(String::as_str)
                .collect::<Vec<_>>();
            w!(
                "    - *{}* ({}) — {}",
                route.route_type,
                if checkers.is_empty() {
                    "none named".into()
                } else {
                    checkers.join(", ")
                },
                route.term.text
            );
        }
        let adverse = &row.adverse_determinations;
        w!(
            "- Individualized adverse determinations: {} — {}",
            adverse.kind,
            adverse.note.text
        );
        for item in &adverse.items {
            w!(
                "    - **{}** against {}. Appeal: {} Remedy: {}",
                item.name,
                item.subject,
                item.appeal.text,
                item.remedy.text
            );
        }
        let temporal = &row.temporal_contract;
        w!(
            "- Temporal contract ({}; custody T3 {}): {}",
            temporal.contract_kind,
            temporal.custody_t3_relation,
            temporal.term.text
        );
        w!("    - Failure polarity: {}", temporal.failure_polarity.text);
        w!("    - Expiry default: {}", temporal.expiry_default.text);
        if !row.delegated_mechanics.is_empty() {
            w!("- Delegated mechanics (bounded, never an invented value):");
            for term in &row.delegated_mechanics {
                w!(
                    "    - {} Choice owner: {} Bounds: {} Failure default: {}",
                    term.text,
                    term.choice_owner.as_deref().unwrap_or(""),
                    term.bounds.as_deref().unwrap_or(""),
                    term.failure_default.as_deref().unwrap_or("")
                );
            }
        }
        w!("- Book 2 handoff: {}", row.book2_handoff);
        w!();
    }
    let inventory = &source.power_source_inventory;
    w!("## Public-power source inventory");
    w!();
    w!(&inventory.scope_ceiling);
    w!();
    w!(
        "The reviewed manifest `{}` binds {} source-identified entries: {} require contract cards, {} are refusals or limits, and {} crosswalk narrow current formal fixtures.",
        inventory.artifact_ref,
        inventory.row_count,
        inventory.disposition_counts.card_required,
        inventory.disposition_counts.explicit_refusal_limit,
        inventory.disposition_counts.existing_formal_crosswalk
    );
    w!();
    w!("Historical lawful-allocation gaps resolved by the complete FS-POW population:");
    w!();
    for gap in &inventory.known_allocation_gaps {
        w!("- {}", gap);
    }
    w!();
    w!("Closure: {}", inventory.closure_condition);
    w!();
    w!("## Assurance routes");
    w!();
    w!("| Route | Status | Warrants | Cannot warrant | Falsification | Negative control |");
    w!("| --- | --- | --- | --- | --- | --- |");
    for row in &source.routes {
        w!(
            "| {} {} | {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.route_status,
            row.warrants,
            row.cannot_warrant,
            row.falsification_condition,
            row.negative_control
        );
    }
    w!();
    w!("## Enum mapping (maps, renames nothing)");
    w!();
    w!("| Source | Field | Value | Canonical |");
    w!("| --- | --- | --- | --- |");
    for row in &source.enum_mapping {
        w!(
            "| {} | {} | `{}` | {} |",
            row.source_file,
            row.field,
            row.value,
            row.canonical
        );
    }
    w!();
    w!("Deliberate exclusions (recorded, not silent):");
    w!();
    for row in &source.enum_mapping_exclusions {
        w!(
            "- `{}` `{}` = `{}`: {}",
            row.source_file,
            row.field,
            row.value,
            row.reason
        );
    }
    w!();
    w!("## Defect-disposition compatibility");
    w!();
    w!("| Disposition | Allowed stages | Resolution-eligible | Requirement |");
    w!("| --- | --- | --- | --- |");
    for row in &source.compatibility_table {
        w!(
            "| `{}` | {} | {} | {} |",
            row.defect_disposition,
            row.allowed_response_stages.join(", "),
            if row.resolution_eligible { "yes" } else { "no" },
            row.resolution_requirement
        );
    }
    w!();
    w!("## Defect rows (disposition and stage; resolution generated)");
    w!();
    w!(
        "The `:defect` markers in the pin files remain the complete list of book-declared, chapter-load-bearing flaws with flip tripwires; these rows are the wider engineering inventory, and they cite the markers where one exists. A resolved row resolves only its named consequence in its exact scope; resolution is claim-relative and asserts nothing beyond the affected claim's own posture. Rows are keyed by defect family, affected claim, consequence, scope, envelope, and source version; a residual sibling shares its family's defect_id."
    );
    w!();
    w!(
        "| Row | Family | Title | Affected claim | Disposition | Stage | Severity | Resolution (generated) | Blocking |"
    );
    w!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.defects {
        let generated = resolution
            .get(&row.id)
            .ok_or_else(|| LedgerError::new(format!("{} has no generated resolution", row.id)))?;
        w!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.id,
            row.defect_id,
            row.title,
            row.affected_claim_ref,
            row.defect_disposition,
            row.response_stage,
            severity_class(row)?,
            generated.resolution_status,
            if generated.blocking { "yes" } else { "no" }
        );
    }
    w!();
    w!(
        "Residual citations bind every sibling residual pool to these rows under the live-read closure; narrowness-impact rows never enter the pool — they are claim-impact anchors, not defects."
    );
    w!();
    w!("## Resolution receipts");
    w!();
    w!(
        "Every receipt records its eligible gate beside the ledger's standing gate status ({}): a recorded gate is a binding, not a passage. A receipt exists only where the generated resolution permits one, and it never implies a narrower repair cured a wider defect.",
        source.acceptance_gate.gate_a_status
    );
    w!();
    for row in &source.receipts {
        w!("### {} — {}", row.id, row.title);
        w!();
        w!(
            "- Defect row: {} (family {}); claim {} at posture {} via {}; assurance ceiling {}; eligible gate {} (gate status: {})",
            row.defect_row_ref,
            row.defect_id,
            row.affected_claim_ref,
            row.claim_posture,
            row.route_ref,
            row.assurance_ceiling,
            row.eligible_gate,
            source.acceptance_gate.gate_a_status
        );
        w!("- What failed: {}", row.what_failed);
        w!("- Hostile witness: {}", row.hostile_witness);
        w!("- Why it failed: {}", row.why_it_failed);
        w!("- The response: {}", row.response_change);
        w!("- What now follows: {}", row.now_follows);
        w!(
            "- Proof: `{}`; negative control: `{}`",
            row.proof_ref,
            row.negative_control_ref
        );
        w!(
            "- What still does not follow: {}",
            row.still_does_not_follow
        );
        w!("- Residuals: {}", row.residuals.join("; "));
        w!(
            "- Reader mapping: `{}`; admissible evidence: {}",
            row.reader_mapping_ref,
            row.admissible_evidence
        );
        w!();
    }
    w!("## Repository scope audit and optional external review");
    w!();
    let protocol = &source.review_protocol;
    let qualifying_events = qualifying_review_events(inputs, source)?;
    let qualifying_audits = qualifying_scope_audits(inputs, source)?;
    w!(
        "Gate A uses the closed, source-derived repository audit. Scope audits: {}; current qualifying audits: {}. The audit binds the semantic scope, protocol, declared criteria, checker controls, command chain, and Gate-A-applicable finding set. Its evidence ceiling expressly supplies no independent-human, reader-response, external-truth, operational, feasibility, liveness, calibration, or timeless-completeness warrant.",
        source.scope_audits.len(),
        qualifying_audits.len()
    );
    w!();
    w!("| Audit | Source | Scope digest | Executed | Result | Findings |");
    w!("| --- | --- | --- | --- | --- | --- |");
    for row in &source.scope_audits {
        w!(
            "| {} | {} | `{}` | {} | {} | {} |",
            row.id,
            row.source_version,
            row.scope_sha256,
            row.executed_at_utc,
            row.result,
            if row.finding_refs.is_empty() {
                "-".into()
            } else {
                row.finding_refs.join(", ")
            }
        );
    }
    w!();
    let designation = &protocol.designation;
    w!(
        "The protocol is bound at `{}`, status {}; mode `{}`; external review policy `{}`; semantic scope digest `{}`. The historical Darshu/Dhanush/custodian designation is `{}` as a project-gate dependency.",
        protocol.protocol_ref,
        protocol.protocol_status,
        protocol.mode,
        protocol.external_review_policy,
        review_scope_digest(source)?,
        designation.designation_status
    );
    w!();
    w!(
        "External commissions, proposals, and terminal events remain append-only optional evidence. They do not control R7, Gate A, Gate C, Gate E, or publication. Their stricter chronology, conflicts, custody, controls, and public-disposition checks still apply if that optional route is used. Commissions: {}; proposals: {}; terminal events: {}; current-source qualifying optional events: {}.",
        source.review_commissions.len(),
        source.proposals.len(),
        source.review_events.len(),
        qualifying_events.len()
    );
    w!();
    if !source.review_commissions.is_empty() {
        w!("| Commission | Source | Scope digest | Window | Cutoff | Reviewers |");
        w!("| --- | --- | --- | --- | --- | --- |");
        for row in &source.review_commissions {
            let reviewers = row
                .reviewers
                .iter()
                .map(|reviewer| format!("{} ({})", reviewer.identity, reviewer.discipline))
                .collect::<Vec<_>>()
                .join("; ");
            w!(
                "| {} | {} | `{}` | {} to {} | {} | {} |",
                row.id,
                row.source_version,
                row.scope_sha256,
                row.received_window.opens_at_utc,
                row.received_window.closes_at_utc,
                row.cutoff_at_utc,
                reviewers
            );
        }
        w!();
    }
    if !source.review_events.is_empty() {
        let qualifying_ids = qualifying_events
            .iter()
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();
        w!("| Event | Commission | Packet commit | Outcome | Current optional |");
        w!("| --- | --- | --- | --- | --- |");
        for row in &source.review_events {
            w!(
                "| {} | {} | `{}` | {} - {} | {} |",
                row.id,
                row.commission_ref,
                row.packet_commit_sha,
                row.outcome_status,
                row.outcome_reason,
                if qualifying_ids.contains(row.id.as_str()) {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        w!();
    }
    w!("| Rubric class | Meaning |");
    w!("| --- | --- |");
    for (class, meaning) in [
        ("critical", &source.severity_rubric.critical),
        ("material", &source.severity_rubric.material),
        ("minor", &source.severity_rubric.minor),
    ] {
        w!("| {} | {} |", class, meaning);
    }
    w!();
    w!("## External assumptions and the envelope");
    w!();
    for row in &source.external_assumptions {
        w!(
            "- **{} {}**: {} Failure consequence: {}",
            row.id,
            row.title,
            row.assumption,
            row.failure_consequence
        );
    }
    w!();
    w!("## The reference envelope (structure)");
    w!();
    for row in &source.envelope {
        w!("- **{}** ({}): {}", row.id, row.envelope_status, row.note);
    }
    w!();
    if let Some(successor) = source.envelope.iter().skip(1).next() {
        w!(
            "Version `{}`. No value enters Book 1: every field's value status names Book 2's Gate D calibration as owner, and this contract refuses a calibrated envelope outright — calibration is a deliberate future contract amendment. This versioned structure satisfies only Gate A's envelope precondition; operation and remedy still require calibration.",
            successor.envelope_version.as_deref().unwrap_or("")
        );
        w!();
        w!("| Field | Definition | Value status | Dependents | Invariance |");
        w!("| --- | --- | --- | --- | --- |");
        for field in successor.fields.as_deref().unwrap_or(&[]) {
            w!(
                "| {} | {} | {} | {} | {} |",
                field.id,
                field.definition,
                field.value_status,
                if field.dependents.is_empty() {
                    "—".into()
                } else {
                    field.dependents.join(", ")
                },
                field.invariance
            );
        }
        w!();
    }
    let criteria = &source.functional_criteria;
    w!("## Functional criteria (the meanings of functional)");
    w!();
    w!(&criteria.drift_note);
    w!();
    w!("| Criterion | Definition | Provenance |");
    w!("| --- | --- | --- |");
    for row in &criteria.criteria {
        w!(
            "| {} | {} | {} |",
            row.name,
            row.definition,
            row.provenance.join("; ")
        );
    }
    w!();
    w!("## Thresholds (meanings, not measurements)");
    w!();
    w!(
        "Each threshold binds a ratified sentence by needle and classifies its lawful source; its layer follows that source, its decision owner is separated from its measurement owner, and no numeric value appears — values arrive with their classified lawful source, never here."
    );
    w!();
    w!("| Threshold | Criterion | Domains | Lawful source | Layer | Definition |");
    w!("| --- | --- | --- | --- | --- | --- |");
    for row in &source.thresholds {
        w!(
            "| {} {} | {} | {} | {} | {} | {} |",
            row.id,
            row.title,
            row.criterion_ref,
            row.domain_refs.join(", "),
            row.lawful_source,
            row.layer,
            row.definition
        );
    }
    w!();
    w!("## Book 2 crosswalk (routed rows only)");
    w!();
    w!(
        "A collection-only projection: Book 2 remains inactive until Book 1 — First Edition actually ships, and this view carries routing and closure fields only. No operating owner, workforce, facility, capacity, service, or cost field appears here; those belong to Book 2's own responsibility view when it activates, generated from this same canonical source."
    );
    w!();
    w!("| ID | Title | Routed as | Owner | Severity | Consequence | Closure condition |");
    w!("| --- | --- | --- | --- | --- | --- | --- |");
    for row in &source.claims {
        if row.layer == "book-2-operation"
            || row.unestablished_disposition.as_deref() == Some("routed-book-2")
        {
            let mut routed = row.layer.clone();
            if let Some(disposition) = &row.unestablished_disposition {
                routed.push_str(" (");
                routed.push_str(disposition);
                routed.push(')');
            }
            w!(
                "| {} | {} | {} | `{}` | {} | {} | {} |",
                row.id,
                row.title,
                routed,
                row.owner_ref,
                row.severity,
                row.consequence,
                row.closure_condition
            );
        }
    }
    for row in &source.defects {
        if row.book2_crosswalk == Some(true) {
            w!(
                "| {} | {} | {} | `{}` | {} | {} | {} |",
                row.id,
                row.title,
                row.defect_disposition,
                row.owner_ref,
                severity_class(row)?,
                row.consequence,
                row.closure_condition
            );
        }
    }
    for row in &source.powers {
        w!(
            "| {} | {} | power operation/evidence handoff | `{}` | {} | {} | {} |",
            row.id,
            row.title,
            row.book2_owner_ref,
            row.severity,
            row.consequence,
            row.closure_condition
        );
    }
    w!();
    w!("## Deferred populations and projections");
    w!();
    w!("| Record type | Stage | Owner | Closure condition |");
    w!("| --- | --- | --- | --- |");
    for row in &source.deferred_populations {
        w!(
            "| {} | {} | `{}` | {} |",
            row.record_type,
            row.stage,
            row.owner_ref,
            row.closure_condition
        );
    }
    w!();
    w!(
        "The coverage-map view, the role matrix, the dependency map, the scenario catalogue, the Book 2 crosswalk, and the assurance allocation now regenerate from the canonical source. The structural reader ledger also regenerates from that source; it is navigation only and supplies no R6 evidence, comprehension result, accessibility validation, reader-suitability claim, Gate C evidence, or route availability. No one projection substitutes for another."
    );
    w!();
    w!("## Conservative rollup");
    w!();
    w!(&source.acceptance_gate.rollup_rule);
    w!();
    w!("## Reproduce");
    w!();
    w!("```bash");
    w!(LEDGER_REFRESH_COMMAND);
    w!("```");
    w!();
    Ok(output.join("\n"))
}

fn render_coverage_region(source: &LedgerDocument) -> LedgerResult<String> {
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();
    let mut lines = vec![
        "| Domain | Historical frozen coverage | Ratified scope requirement | Current contract readiness | Historical gap / ruling record | Split claims (posture) | Direct-effect cards | Implementation and tests | Book 2 boundary |".to_owned(),
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_owned(),
    ];
    for legacy in &source.legacy_rows {
        let splits = legacy
            .split_claim_refs
            .iter()
            .map(|claim_ref| {
                let claim = claims.get(claim_ref.as_str()).ok_or_else(|| {
                    LedgerError::new(format!(
                        "{} names missing split claim {}",
                        legacy.id, claim_ref
                    ))
                })?;
                let mut posture = claim.posture.clone();
                if claim.posture == "Unestablished" {
                    posture.push('/');
                    posture.push_str(claim.unestablished_disposition.as_deref().ok_or_else(
                        || {
                            LedgerError::new(format!(
                                "{} is Unestablished without a disposition",
                                claim.id
                            ))
                        },
                    )?);
                }
                Ok(format!("{} ({posture})", claim.id))
            })
            .collect::<LedgerResult<Vec<_>>>()?;

        struct Card<'a> {
            id: &'a str,
            affected_claim_refs: &'a [String],
            negative_status: &'a str,
            counterfactual_status: &'a str,
        }
        let cards = source
            .powers
            .iter()
            .map(|row| Card {
                id: &row.id,
                affected_claim_refs: &row.affected_claim_refs,
                negative_status: &row.negative_test.status,
                counterfactual_status: &row.counterfactual.status,
            })
            .chain(source.constitutional_effects.iter().map(|row| Card {
                id: &row.id,
                affected_claim_refs: &row.affected_claim_refs,
                negative_status: &row.negative_test.status,
                counterfactual_status: &row.counterfactual.status,
            }))
            .filter(|card| {
                card.affected_claim_refs
                    .iter()
                    .any(|claim| legacy.split_claim_refs.contains(claim))
            })
            .collect::<Vec<_>>();
        let readiness = if cards.is_empty() {
            "historical row has no direct-effect card in this claim split"
        } else {
            // Strict typed deserialization makes contract_terms/profile_terms
            // mandatory on both card variants, so nonempty cards are ready.
            "coverage-ready; not formalized or operational"
        };
        let tests = if cards.is_empty() {
            "—".to_owned()
        } else {
            cards
                .iter()
                .map(|card| {
                    format!(
                        "{}: {}/{}",
                        card.id, card.negative_status, card.counterfactual_status
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let ids = if cards.is_empty() {
            "—".to_owned()
        } else {
            cards
                .iter()
                .map(|card| card.id)
                .collect::<Vec<_>>()
                .join("; ")
        };
        let book2 = if cards.is_empty() {
            "No card handoff in this historical row"
        } else {
            "Every listed card carries a no-operation Book 2 handoff"
        };
        lines.push(format!(
            "| {} | Historical: {} | {} | {} | Historical: {} | {} | {} | {} | {} |",
            legacy.domain_title,
            legacy.legacy_coverage,
            legacy.legacy_scope_requirement,
            readiness,
            legacy.legacy_gap,
            if splits.is_empty() {
                "—".into()
            } else {
                splits.join("; ")
            },
            ids,
            tests,
            book2
        ));
    }
    Ok(lines.join("\n"))
}

fn coverage_map_needles(source_bytes: &[u8]) -> LedgerResult<BTreeSet<String>> {
    const PREFIX: &str = "new-book-plans/book-1-constitutional-coverage-map.md::";
    let text = std::str::from_utf8(source_bytes)
        .map_err(|_| LedgerError::new("reviewed source is not UTF-8"))?;
    let mut rest = text;
    let mut needles = BTreeSet::new();
    while let Some(offset) = rest.find(PREFIX) {
        let start = offset;
        let mut escaped = false;
        let mut end = None;
        for (index, byte) in rest[start..].bytes().enumerate() {
            if index == 0 {
                continue;
            }
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                end = Some(start + index);
                break;
            }
        }
        let end = end.ok_or_else(|| {
            LedgerError::new("unterminated coverage-map reference in reviewed source")
        })?;
        let encoded = format!("\"{}\"", &rest[start..end]);
        let reference: String = serde_json::from_str(&encoded).map_err(|error| {
            LedgerError::new(format!("invalid coverage-map reference encoding: {error}"))
        })?;
        let needle = reference
            .strip_prefix(PREFIX)
            .expect("searched coverage-map prefix");
        needles.insert(needle.to_owned());
        rest = &rest[end + 1..];
    }
    Ok(needles)
}

fn splice_coverage_map(current: &str, body: &str, source_bytes: &[u8]) -> LedgerResult<String> {
    const BEGIN: &str = "<!-- BEGIN GENERATED: full-society-coverage -->";
    const END: &str = "<!-- END GENERATED: full-society-coverage -->";
    if current.matches(BEGIN).count() != 1 || current.matches(END).count() != 1 {
        return Err(LedgerError::new(
            "coverage map has no unique generated region — add the BEGIN/END markers first",
        ));
    }
    if body
        .lines()
        .any(|line| line.starts_with("## ") || line.starts_with("### "))
    {
        return Err(LedgerError::new(
            "the generated coverage region may not emit a heading line",
        ));
    }
    let begin = current.find(BEGIN).expect("unique begin marker") + BEGIN.len();
    let end = current.find(END).expect("unique end marker");
    if begin > end {
        return Err(LedgerError::new(
            "coverage map generated-region markers are reversed",
        ));
    }
    let spliced = format!("{}\n{}\n{}", &current[..begin], body, &current[end..]);
    for needle in coverage_map_needles(source_bytes)? {
        let count = spliced.matches(&needle).count();
        if count != 1 {
            return Err(LedgerError::new(format!(
                "after splicing, coverage-map needle must occur exactly once; found {count}: {needle:?}"
            )));
        }
    }
    Ok(spliced)
}

fn validate_reader_projection(source: &LedgerDocument, rendered: &str) -> LedgerResult<()> {
    const CEILING: &str = "**STRUCTURAL READER NAVIGATION ONLY.** This projection supplies no R6 evidence, comprehension result, accessibility validation, reader-suitability claim, Gate C evidence, or route availability.";
    if rendered.matches(CEILING).count() != 1 {
        return Err(LedgerError::new(
            "reader projection must carry the exact no-evidence ceiling once",
        ));
    }
    let source_line = format!(
        "Canonical source SHA-256: `{}`. Every canonical list population is bound below; this digest also binds the non-list contract fields.",
        canonical_digest(source)?
    );
    if rendered.matches(&source_line).count() != 1 {
        return Err(LedgerError::new(
            "reader projection must bind the exact canonical source once",
        ));
    }
    for line in reader_population_lines(source)? {
        if rendered.matches(&line).count() != 1 {
            let population = line.split('|').nth(1).unwrap_or("unknown").trim();
            return Err(LedgerError::new(format!(
                "reader projection population closure is missing or duplicated: {population}"
            )));
        }
    }
    Ok(())
}

fn render_reader(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
) -> LedgerResult<String> {
    let mut output = Vec::new();
    let mut line = |value: String| output.push(value);
    let mut blocked_by: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for defect in &source.defects {
        if resolution
            .get(&defect.id)
            .is_some_and(|generated| generated.blocking)
        {
            blocked_by
                .entry(defect.affected_claim_ref.as_str())
                .or_default()
                .push(defect.id.as_str());
        }
    }
    let claims_by_id = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<HashMap<_, _>>();

    line("<!-- SPDX-License-Identifier: CC-BY-4.0 -->".into());
    line("<!-- Generated by the native rights-verify ledger refresh; do not edit. -->".into());
    line(String::new());
    line("# Full-Society Structural Reader Ledger — Generated Projection".into());
    line(String::new());
    line("**STRUCTURAL READER NAVIGATION ONLY.** This projection supplies no R6 evidence, comprehension result, accessibility validation, reader-suitability claim, Gate C evidence, or route availability.".into());
    line(String::new());
    line(format!(
        "Canonical source version: `{}`. Gate verdict: **{}**",
        source.source_version, source.acceptance_gate.verdict
    ));
    line(String::new());
    line(format!(
        "Coverage contracts: **{}**. Coverage-ready means source-specific planning is complete; it does not mean formalized, prose-landed, implemented, or operational.",
        source.coverage_population.status
    ));
    line(String::new());
    line("Coverage-family drafting states:".into());
    line(String::new());
    for family in &source.coverage_families {
        line(format!(
            "- {} {}: {} — {}",
            family.id, family.title, family.state, family.blocked_before_drafting
        ));
    }
    line(String::new());
    line(format!(
        "Canonical source SHA-256: `{}`. Every canonical list population is bound below; this digest also binds the non-list contract fields.",
        canonical_digest(source)?
    ));
    line(String::new());
    line("## Projection population closure".into());
    line(String::new());
    line("| Canonical population | Rows | Canonical SHA-256 | Stable identities |".into());
    line("| --- | ---: | --- | --- |".into());
    for row in reader_population_lines(source)? {
        line(row);
    }
    line(String::new());
    line("## Five-layer key".into());
    line(String::new());
    for layer in SCOPE_DISPOSITIONS {
        line(format!(
            "- `{layer}`: {}",
            source.scope_disposition_meanings[layer]
        ));
    }
    line(String::new());
    line("## Domain navigation".into());
    line(String::new());
    for domain in &source.domains {
        let domain_claims = source
            .claims
            .iter()
            .filter(|claim| claim.domain_refs.contains(&domain.id))
            .collect::<Vec<_>>();
        let claim_ids = domain_claims
            .iter()
            .map(|claim| claim.id.as_str())
            .collect::<HashSet<_>>();
        let domain_scenarios = source
            .scenarios
            .iter()
            .filter(|scenario| scenario.domain_refs.contains(&domain.id))
            .collect::<Vec<_>>();
        let domain_defects = source
            .defects
            .iter()
            .filter(|defect| claim_ids.contains(defect.affected_claim_ref.as_str()))
            .collect::<Vec<_>>();
        line(format!("### {} — {}", domain.id, domain.title));
        line(String::new());
        line(format!(
            "**Reader destination:** {}",
            domain.reader_destination
        ));
        line(String::new());
        line("Layer dispositions:".into());
        line(String::new());
        for (layer, bucket) in SCOPE_DISPOSITIONS.into_iter().zip([
            &domain.constitutional_invariants,
            &domain.ordinary_law_choices,
            &domain.protected_private_civic,
            &domain.book2_operations,
            &domain.external_assumptions_note,
        ]) {
            line(format!("- `{layer}`: {}", bucket_cell(bucket)));
        }
        line(String::new());
        line("Claims:".into());
        line(String::new());
        for claim in &domain_claims {
            let mut disposition = claim.posture.clone();
            if let Some(value) = &claim.unestablished_disposition {
                disposition.push_str(" / ");
                disposition.push_str(value);
            }
            let blockers = blocked_by
                .get(claim.id.as_str())
                .map(|values| values.join(", "))
                .unwrap_or_else(|| "none".to_owned());
            line(format!(
                "- **{} — {}**: {} Posture: `{disposition}`; route: `{}`; overlay: `{}`; blocking defect rows: {blockers}. Scope: {} Public limit: {}",
                claim.id,
                claim.title,
                claim.claim,
                claim.route_ref,
                claim.overlay,
                claim.scope_bound,
                claim.public_claim_restriction
            ));
        }
        if domain_claims.is_empty() {
            line("- None.".into());
        }
        line(String::new());
        let domain_powers = source
            .powers
            .iter()
            .filter(|power| power.domain_refs.contains(&domain.id))
            .collect::<Vec<_>>();
        let domain_effects = source
            .constitutional_effects
            .iter()
            .filter(|effect| effect.domain_refs.contains(&domain.id))
            .collect::<Vec<_>>();
        line("Source-derived power cards:".into());
        line(String::new());
        for power in &domain_powers {
            let bounded = power
                .contract_terms
                .get("bounded_effect")
                .ok_or_else(|| LedgerError::new(format!("{} lacks bounded_effect", power.id)))?;
            line(format!(
                "- {} — {} ({}); class {}; profiles {}; claims {}; tests {}/{}; Part V {}. Contract: {} Book 2 boundary: {}",
                power.id,
                power.title,
                power.manifest_key,
                power.primary_class_ref,
                power.profiles.join(", "),
                power.affected_claim_refs.join(", "),
                power.negative_test.status,
                power.counterfactual.status,
                power.part_v_status,
                bounded.text,
                power.book2_handoff
            ));
        }
        if domain_powers.is_empty() {
            line("- None in the completed source-family prefix.".into());
        }
        line(String::new());
        line("Constitutional non-power effects:".into());
        line(String::new());
        for effect in &domain_effects {
            line(format!(
                "- {} \u{14} {}; class {}; claims {}; tests {}/{}; Part V {}. Boundary: {}",
                effect.id,
                effect.title,
                effect.primary_class_ref,
                effect.affected_claim_refs.join(", "),
                effect.negative_test.status,
                effect.counterfactual.status,
                effect.part_v_status,
                effect.book2_handoff
            ));
        }
        if domain_effects.is_empty() {
            line("- None.".into());
        }
        line(String::new());
        line("Ordinary, failure, and recovery routing:".into());
        line(String::new());
        for scenario in &domain_scenarios {
            line(format!(
                "- **{} — {}** (`{}`): ordinary — {}; failure — {}; recovery — {}",
                scenario.id,
                scenario.title,
                scenario.scenario_kind,
                scenario.ordinary_route,
                scenario.failure_route,
                scenario.recovery_route
            ));
        }
        if domain_scenarios.is_empty() {
            line("- None recorded.".into());
        }
        line(String::new());
        line("Open and bounded defect consequences:".into());
        line(String::new());
        for defect in &domain_defects {
            let generated = &resolution[&defect.id];
            line(format!(
                "- **{} — {}**: severity {}; consequence: {}; closure: {}; applicable gates: {}; generated resolution: `{}`; blocking for `{}`: `{}`.",
                defect.id,
                defect.title,
                defect.severity,
                defect.consequence,
                defect.closure_condition,
                defect.applicable_gate_refs.join(", "),
                generated.resolution_status,
                defect.affected_claim_ref,
                generated.blocking
            ));
        }
        if domain_defects.is_empty() {
            line("- None recorded.".into());
        }
        line(String::new());
    }
    line("## Bounded repair mappings".into());
    line(String::new());
    line("These are receipt-to-reader mapping references for eligible repairs. They do not establish that a reader understood or could access them.".into());
    line(String::new());
    for receipt in &source.receipts {
        let claim = claims_by_id
            .get(receipt.affected_claim_ref.as_str())
            .ok_or_else(|| LedgerError::new(format!("{} names no claim", receipt.id)))?;
        line(format!(
            "- `{}` → `{}` (`{}`); ceiling: `{}`; still does not follow: {}",
            receipt.id,
            claim.id,
            receipt.reader_mapping_ref,
            receipt.assurance_ceiling,
            receipt.still_does_not_follow
        ));
    }
    if source.receipts.is_empty() {
        line("- None.".into());
    }
    line(String::new());
    line("## Reproduce".into());
    line(String::new());
    line("```bash".into());
    line(LEDGER_REFRESH_COMMAND.into());
    line("```".into());
    line(String::new());
    Ok(output.join("\n"))
}

struct RenderedOutputs {
    report: String,
    reader: String,
    coverage_map: String,
}

fn render_outputs(
    ledger: &ValidatedLedger,
    current_coverage_map: &str,
) -> LedgerResult<RenderedOutputs> {
    let report = render_report(&ledger.input_bytes, &ledger.document, &ledger.resolutions)?;
    let reader = render_reader(&ledger.document, &ledger.resolutions)?;
    validate_reader_projection(&ledger.document, &reader)?;
    let coverage_region = render_coverage_region(&ledger.document)?;
    let coverage_map =
        splice_coverage_map(current_coverage_map, &coverage_region, &ledger.source_bytes)?;
    Ok(RenderedOutputs {
        report,
        reader,
        coverage_map,
    })
}

fn read_utf8(path: &Path, context: &str) -> LedgerResult<String> {
    let bytes = fs::read(path)
        .map_err(|error| LedgerError::new(format!("cannot read {context}: {error}")))?;
    String::from_utf8(bytes).map_err(|_| LedgerError::new(format!("{context} is not valid UTF-8")))
}

pub(crate) fn check_validated(
    context: &Context,
    ledger: &ValidatedLedger,
) -> Result<CheckResult, Error> {
    let controls = negative_controls(ledger).map_err(|error| ledger_error(error.to_string()))?;
    let snapshot = ledger.immutable_snapshot.as_ref().ok_or_else(|| {
        ledger_error("normal check requires its original immutable-input snapshot")
    })?;
    let mut snapshot = snapshot
        .lock()
        .map_err(|_| ledger_error("immutable-input snapshot lock is poisoned"))?;
    let current_map = snapshot
        .read_text(&context.path(COVERAGE_MAP))
        .map_err(|error| ledger_error(error.to_string()))?;
    let rendered =
        render_outputs(ledger, &current_map).map_err(|error| ledger_error(error.to_string()))?;
    let output_path = context.path(OUTPUT);
    let current_report = if output_path.exists() {
        snapshot
            .read_bytes(&output_path)
            .map_err(|error| ledger_error(error.to_string()))?
            .to_vec()
    } else {
        Vec::new()
    };
    if current_report != rendered.report.as_bytes() {
        return Err(ledger_error(format!(
            "{OUTPUT} is STALE — rerun without --check"
        )));
    }
    let reader_output_path = context.path(READER_OUTPUT);
    let current_reader = if reader_output_path.exists() {
        snapshot
            .read_bytes(&reader_output_path)
            .map_err(|error| ledger_error(error.to_string()))?
            .to_vec()
    } else {
        Vec::new()
    };
    if current_reader != rendered.reader.as_bytes() {
        return Err(ledger_error(format!(
            "{READER_OUTPUT} is STALE — rerun without --check"
        )));
    }
    if current_map.as_bytes() != rendered.coverage_map.as_bytes() {
        return Err(ledger_error(format!(
            "{COVERAGE_MAP} generated region is STALE — rerun without --check"
        )));
    }
    snapshot
        .assert_unchanged()
        .map_err(|error| ledger_error(error.to_string()))?;
    Ok(CheckResult {
        controls,
        message: format!(
            "{OUTPUT}, {READER_OUTPUT}, and the coverage-map region are current; {controls} structural negative controls pass; enum-mapping and residual-coverage closures over the seven reviewed sources hold; routing inventory only — nothing established beyond each row's own posture"
        ),
    })
}

pub(crate) fn check(context: &Context) -> Result<CheckResult, Error> {
    let ledger = load_and_validate(context)?;
    check_validated(context, &ledger)
}

pub(crate) fn run_validated(
    context: &Context,
    mode: Mode,
    ledger: &ValidatedLedger,
) -> Result<CheckResult, Error> {
    if mode == Mode::Check {
        return check_validated(context, ledger);
    }

    let controls = negative_controls(ledger).map_err(|error| ledger_error(error.to_string()))?;

    if mode == Mode::RefreshAndCheck {
        let mut snapshot = ImmutableRepositoryInputs::new(context.root())?;
        for (path, bytes) in ledger.immutable_inputs() {
            snapshot.adopt_bytes(&context.path(path), bytes)?;
        }
        let current_map = snapshot.read_text(&context.path(COVERAGE_MAP))?;
        let rendered = render_outputs(ledger, &current_map)
            .map_err(|error| ledger_error(error.to_string()))?;
        atomic_refresh_and_check(
            &[
                (context.path(OUTPUT), rendered.report.into_bytes()),
                (context.path(READER_OUTPUT), rendered.reader.into_bytes()),
                (
                    context.path(COVERAGE_MAP),
                    rendered.coverage_map.into_bytes(),
                ),
            ],
            &mut snapshot,
        )?;
        return Ok(CheckResult {
            controls,
            message: format!(
                "refreshed and checked {OUTPUT}, {READER_OUTPUT}, and the coverage-map region; {controls} structural negative controls pass; enum-mapping and residual-coverage closures over the seven reviewed sources hold; routing inventory only — nothing established beyond each row's own posture"
            ),
        });
    }

    let current_map = read_utf8(&context.path(COVERAGE_MAP), COVERAGE_MAP)
        .map_err(|error| ledger_error(error.to_string()))?;
    let rendered =
        render_outputs(ledger, &current_map).map_err(|error| ledger_error(error.to_string()))?;
    fs::write(context.path(OUTPUT), rendered.report)?;
    fs::write(context.path(READER_OUTPUT), rendered.reader)?;
    fs::write(context.path(COVERAGE_MAP), rendered.coverage_map)?;
    Ok(CheckResult {
        controls,
        message: format!(
            "wrote {OUTPUT}, {READER_OUTPUT}, and the coverage-map region; {controls} structural negative controls pass"
        ),
    })
}

pub(crate) fn run(context: &Context, mode: Mode) -> Result<CheckResult, Error> {
    let ledger = load_and_validate(context)?;
    run_validated(context, mode, &ledger)
}

struct JsonSeed;

impl<'de> DeserializeSeed<'de> for JsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonSeed.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(JsonSeed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, map.next_value_seed(JsonSeed)?);
        }
        Ok(Value::Object(values))
    }
}

fn parse_json_no_duplicates(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = JsonSeed.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::from_test_root(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
    }

    fn reviewed_economic_carry_contracts() -> Vec<EconomicCarryRuleContract> {
        let context = context();
        let bytes = fs::read(context.path(SOURCE)).expect("reviewed ledger source");
        let value = parse_json_no_duplicates(&bytes).expect("unique-key ledger JSON");
        serde_json::from_value(
            value
                .get("economic_carry_rule_contracts")
                .expect("economic carry contracts")
                .clone(),
        )
        .expect("typed economic carry contracts")
    }

    fn reviewed_economic_acceptance_cases() -> Vec<EconomicAcceptanceCase> {
        let context = context();
        let bytes = fs::read(context.path(SOURCE)).expect("reviewed ledger source");
        let value = parse_json_no_duplicates(&bytes).expect("unique-key ledger JSON");
        serde_json::from_value(
            value
                .get("economic_acceptance_cases")
                .expect("economic acceptance cases")
                .clone(),
        )
        .expect("typed economic acceptance cases")
    }

    fn move_grounded_positive_block_before_first_negative(pin: &str, key: &str) -> String {
        let mut lines = pin.lines().map(str::to_owned).collect::<Vec<_>>();
        let positive_marker = format!(
            "# {key} positive: the exact completed card and reviewed duty selection compose."
        );
        let positive_marker_index = lines
            .iter()
            .position(|line| line == &positive_marker)
            .expect("positive duty marker");
        let positive_block_start = lines[..positive_marker_index]
            .iter()
            .rposition(|line| line.starts_with("# => "))
            .map_or(0, |index| index + 1);
        let positive_block = lines
            .drain(positive_block_start..positive_marker_index + 3)
            .collect::<Vec<_>>();

        let negative_marker = format!("# {key} negative:");
        let negative_marker_index = lines
            .iter()
            .position(|line| line.starts_with(&negative_marker))
            .expect("negative duty marker");
        let negative_block_start = lines[..negative_marker_index]
            .iter()
            .rposition(|line| line.starts_with("# => "))
            .map_or(0, |index| index + 1);
        lines.splice(negative_block_start..negative_block_start, positive_block);

        let mut mutant = lines.join("\n");
        if pin.ends_with('\n') {
            mutant.push('\n');
        }
        mutant
    }

    #[test]
    fn grounded_economic_duty_pin_order_is_checker_owned() {
        let grounded = ECONOMIC_DUTY_BRIDGES
            .iter()
            .filter(|spec| !spec.bearer.starts_with('$'))
            .collect::<Vec<_>>();
        assert_eq!(grounded.len(), 21);
        assert_eq!(
            grounded
                .iter()
                .map(|spec| spec.power)
                .collect::<HashSet<_>>()
                .len(),
            19
        );

        let inputs = load_static_inputs(&context()).expect("static economic live pins");
        validate_grounded_economic_duty_pin_order(&inputs)
            .expect("grounded negative cases precede their positive assertion blocks");
    }

    #[test]
    fn grounded_economic_duty_pin_order_rejects_a_reordered_positive_block() {
        let path = "new-book-plans/economic-power-063.pins.nibli";
        let mut inputs = load_static_inputs(&context()).expect("static economic live pins");
        let pin = std::str::from_utf8(input_bytes(&inputs, path).expect("power 063 live pin"))
            .expect("UTF-8 power 063 live pin")
            .to_owned();
        let mutant =
            move_grounded_positive_block_before_first_negative(&pin, "knowledge-floor-access");
        inputs.insert(path.to_owned(), mutant.into_bytes());

        let error = validate_grounded_economic_duty_pin_order(&inputs)
            .expect_err("a positive fixture before its matching negatives must fail");
        let message = error.to_string();
        assert!(message.contains(path));
        assert!(message.contains("knowledge-floor-access"));
        assert!(message.contains("must precede the matching positive assertion block"));
    }

    #[test]
    fn grounded_economic_duty_pin_order_rejects_an_unmarked_duplicate_query() {
        let path = "new-book-plans/economic-power-063.pins.nibli";
        let mut inputs = load_static_inputs(&context()).expect("static economic live pins");
        let pin = std::str::from_utf8(input_bytes(&inputs, path).expect("power 063 live pin"))
            .expect("UTF-8 power 063 live pin")
            .to_owned();
        let spec = ECONOMIC_DUTY_BRIDGES
            .iter()
            .find(|spec| spec.key == "knowledge-floor-access")
            .expect("knowledge floor duty bridge");
        let query = format!(
            "? obliged({}, {}, {}).",
            spec.bearer, spec.duty, spec.standard
        );
        let mutant = pin.replacen(&query, &format!("{query}\n{query}"), 1);
        assert_ne!(mutant, pin);
        inputs.insert(path.to_owned(), mutant.into_bytes());

        let error = validate_grounded_economic_duty_pin_order(&inputs)
            .expect_err("an unmarked duplicate grounded query must fail");
        let message = error.to_string();
        assert!(message.contains(path));
        assert!(message.contains("knowledge-floor-access"));
        assert!(message.contains("duplicate or unmarked queries are forbidden"));
    }

    #[test]
    fn economic_power_088_dependency_links_are_checker_owned() {
        let inputs = load_static_inputs(&context()).expect("static economic live pins");
        validate_economic_power_088_dependency_links(&inputs)
            .expect("live and counterfactual P088 links target matching For088 producers");
    }

    #[test]
    fn economic_power_088_dependency_links_reject_stale_nested_targets() {
        let path = "new-book-plans/economic-power-088.pins.nibli";
        let mut inputs = load_static_inputs(&context()).expect("static economic live pins");
        let pin = std::str::from_utf8(input_bytes(&inputs, path).expect("power 088 live pin"))
            .expect("UTF-8 power 088 live pin")
            .to_owned();

        for (current, stale) in [
            (
                "EconResult086For088Live, EconomicDependencyResultScope_087_086",
                "EconResult086For087Live, EconomicDependencyResultScope_087_086",
            ),
            (
                "EconRecord086For088Live, EconomicDependencyRecordScope_087_086",
                "EconRecord086For087Live, EconomicDependencyRecordScope_087_086",
            ),
        ] {
            let mutant = pin.replacen(current, stale, 1);
            assert_ne!(mutant, pin, "watched dependency target must exist");
            inputs.insert(path.to_owned(), mutant.into_bytes());

            let error = validate_economic_power_088_dependency_links(&inputs)
                .expect_err("a stale For087 prerequisite target must fail");
            let message = error.to_string();
            assert!(message.contains(path));
            assert!(message.contains(stale.split(',').next().expect("stale target")));
            assert!(message.contains("matching For088 prerequisite producer"));
        }
    }

    #[test]
    fn economic_power_088_counterfactual_rejects_stale_nested_targets() {
        let path =
            "new-book-plans/counterfactual/no-economic-independent-current-review-088.pins.nibli";
        let mut inputs = load_static_inputs(&context()).expect("static economic pins");
        let pin =
            std::str::from_utf8(input_bytes(&inputs, path).expect("power 088 counterfactual"))
                .expect("UTF-8 power 088 counterfactual")
                .to_owned();
        let mutant = pin.replacen(
            "EconResult086For088Counterfactual, EconomicDependencyResultScope_087_086",
            "EconResult086For087Counterfactual, EconomicDependencyResultScope_087_086",
            1,
        );
        assert_ne!(mutant, pin, "watched counterfactual target must exist");
        inputs.insert(path.to_owned(), mutant.into_bytes());

        let error = validate_economic_power_088_dependency_links(&inputs)
            .expect_err("a stale counterfactual For087 prerequisite target must fail");
        let message = error.to_string();
        assert!(message.contains(path));
        assert!(message.contains("matching For088 prerequisite link"));
    }

    #[test]
    fn economic_power_088_counterfactual_rejects_a_private_tier_token() {
        let path =
            "new-book-plans/counterfactual/no-economic-independent-current-review-088.pins.nibli";
        let mut inputs = load_static_inputs(&context()).expect("static economic pins");
        let pin =
            std::str::from_utf8(input_bytes(&inputs, path).expect("power 088 counterfactual"))
                .expect("UTF-8 power 088 counterfactual")
                .to_owned();
        let mutant = pin.replacen(
            "EconSource088Counterfactual, EconRecord088Counterfactual, CommonTier, GovernmentTierScope",
            "EconSource088Counterfactual, EconRecord088Counterfactual, EconTier088Counterfactual, GovernmentTierScope",
            1,
        );
        assert_ne!(mutant, pin, "watched counterfactual tier must exist");
        inputs.insert(path.to_owned(), mutant.into_bytes());

        let error = validate_economic_power_088_dependency_links(&inputs)
            .expect_err("a private counterfactual tier token must fail");
        let message = error.to_string();
        assert!(message.contains(path));
        assert!(message.contains("shared payment chain to CommonTier"));
    }

    #[test]
    fn economic_acceptance_cases_are_structurally_owner_bound() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static acceptance inputs");
        validate_economic_acceptance_cases(&loaded.source, &inputs)
            .expect("reviewed acceptance cases and owner evidence");
        assert_eq!(
            typed_fingerprint(
                &loaded.source.economic_acceptance_cases,
                "economic acceptance cases"
            )
            .expect("acceptance fingerprint"),
            EXPECTED_ECONOMIC_ACCEPTANCE_CASES_SHA256
        );
    }

    #[test]
    fn economic_acceptance_power_fixture_semantics_are_checker_owned() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static acceptance inputs");
        for number in 61..=88 {
            let contract = loaded
                .source
                .economic_power_rule_contracts
                .iter()
                .find(|row| row.power_ref == format!("FS-POW-{number:03}"))
                .expect("economic power contract");
            validate_economic_missing_requirement_fixture(
                &inputs,
                &format!("new-book-plans/economic-power-{number:03}.pins.nibli"),
                number,
                contract
                    .requirements
                    .first()
                    .expect("card-specific requirement"),
                &format!("FS-POW-{number:03} missing requirement"),
            )
            .expect("missing requirement differs only by its exact required observations");
            validate_economic_expired_selection_fixture(
                &inputs,
                &format!("new-book-plans/economic-power-{number:03}.pins.nibli"),
                number,
                &format!("FS-POW-{number:03} expired selection"),
            )
            .expect("expired selection differs only in its supplied currentness");
        }
        validate_economic_classified_acceptance_fixture(
            &inputs,
            "new-book-plans/economic-power-072.pins.nibli",
            72,
            "AcceptanceAntiConcentrationTax",
            "EconTaxInstrument",
            "AntiConcentrationTaxInstrument",
            "FS-POW-072 acceptance classification",
        )
        .expect("anti-concentration tax differs only in its supplied instrument classification");
        validate_economic_classified_acceptance_fixture(
            &inputs,
            "new-book-plans/economic-power-085.pins.nibli",
            85,
            "AcceptanceLuxuryCollection",
            "EconCollection",
            "ParticularNonFloorLuxuryAssetCollection",
            "FS-POW-085 acceptance classification",
        )
        .expect("luxury collection differs only in its supplied collection classification");
    }

    fn assert_economic_acceptance_digest_changed(
        mutant: &Vec<EconomicAcceptanceCase>,
        context: &str,
    ) {
        assert_ne!(
            typed_fingerprint(mutant, context).expect("mutant acceptance fingerprint"),
            EXPECTED_ECONOMIC_ACCEPTANCE_CASES_SHA256,
            "acceptance mutation escaped its typed digest: {context}"
        );
    }

    #[test]
    fn economic_acceptance_digest_watches_every_required_field_and_omission() {
        let cases = reviewed_economic_acceptance_cases();
        assert_eq!(
            typed_fingerprint(&cases, "reviewed acceptance cases").expect("acceptance digest"),
            EXPECTED_ECONOMIC_ACCEPTANCE_CASES_SHA256
        );
        let mut controls = 0_usize;
        for case_index in 0..cases.len() {
            let mut mutant = cases.clone();
            mutant.remove(case_index);
            assert_economic_acceptance_digest_changed(&mutant, "case omission");
            controls += 1;

            for field in 0..2 {
                let mut mutant = cases.clone();
                match field {
                    0 => mutant[case_index].case_id.push_str("Mutation"),
                    1 => mutant[case_index].source_needle.push_str(" mutation"),
                    _ => unreachable!(),
                }
                assert_economic_acceptance_digest_changed(&mutant, "case field mutation");
                controls += 1;
            }

            for mapping_index in 0..cases[case_index].mappings.len() {
                let mut mutant = cases.clone();
                mutant[case_index].mappings.remove(mapping_index);
                assert_economic_acceptance_digest_changed(&mutant, "mapping omission");
                controls += 1;

                for field in 0..3 {
                    let mut mutant = cases.clone();
                    let mapping = &mut mutant[case_index].mappings[mapping_index];
                    match field {
                        0 => mapping.variant_id.push_str("Mutation"),
                        1 => mapping.mapping_id.push_str("Mutation"),
                        2 => mapping.assertion.push_str(" mutation"),
                        _ => unreachable!(),
                    }
                    assert_economic_acceptance_digest_changed(&mutant, "mapping field mutation");
                    controls += 1;
                }

                for support_index in 0..cases[case_index].mappings[mapping_index].supports.len() {
                    let mut mutant = cases.clone();
                    mutant[case_index].mappings[mapping_index]
                        .supports
                        .remove(support_index);
                    assert_economic_acceptance_digest_changed(&mutant, "support omission");
                    controls += 1;

                    for field in 0..6 {
                        let mut mutant = cases.clone();
                        let support =
                            &mut mutant[case_index].mappings[mapping_index].supports[support_index];
                        match field {
                            0 => support.owner_kind.push_str("Mutation"),
                            1 => support.owner_id.push_str("Mutation"),
                            2 => support.polarity.push_str("Mutation"),
                            3 => support.pin_ref.push_str("Mutation"),
                            4 => support.query.push_str("Mutation"),
                            5 => support.expected_result.push_str("Mutation"),
                            _ => unreachable!(),
                        }
                        assert_economic_acceptance_digest_changed(
                            &mutant,
                            "support field mutation",
                        );
                        controls += 1;
                    }

                    for formal_index in 0..cases[case_index].mappings[mapping_index].supports
                        [support_index]
                        .formal_refs
                        .len()
                    {
                        let mut mutant = cases.clone();
                        mutant[case_index].mappings[mapping_index].supports[support_index]
                            .formal_refs
                            .remove(formal_index);
                        assert_economic_acceptance_digest_changed(
                            &mutant,
                            "formal reference omission",
                        );
                        controls += 1;

                        let mut mutant = cases.clone();
                        mutant[case_index].mappings[mapping_index].supports[support_index]
                            .formal_refs[formal_index]
                            .push_str("Mutation");
                        assert_economic_acceptance_digest_changed(
                            &mutant,
                            "formal reference mutation",
                        );
                        controls += 1;
                    }
                }
            }
        }
        assert_eq!(controls, 3_636);
    }

    #[test]
    fn economic_acceptance_owner_evidence_rejects_every_cross_owner_substitution() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static acceptance inputs");
        let family = loaded
            .source
            .coverage_families
            .iter()
            .find(|row| row.id == "FS-CVF-006")
            .expect("economic coverage family");
        let supports = loaded
            .source
            .economic_acceptance_cases
            .iter()
            .flat_map(|case| &case.mappings)
            .flat_map(|mapping| &mapping.supports)
            .collect::<Vec<_>>();
        assert_eq!(supports.len(), 320);
        let mut controls = 0_usize;
        for (index, support) in supports.iter().enumerate() {
            let donor = supports
                .iter()
                .copied()
                .find(|candidate| {
                    candidate.owner_kind == support.owner_kind
                        && candidate.owner_id != support.owner_id
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} needs a distinct same-kind substitution donor",
                        support.owner_kind
                    )
                });
            let formal_needles = support
                .formal_refs
                .iter()
                .map(|reference| {
                    economic_reference_parts(reference, "owner substitution formal ref")
                        .expect("formal reference")
                        .1
                })
                .collect::<Vec<_>>();
            let pin_path =
                economic_reference_parts(&support.pin_ref, "owner substitution pin reference")
                    .expect("pin reference")
                    .0;
            let mut owner_substitution = (*support).clone();
            owner_substitution.owner_id.clone_from(&donor.owner_id);
            assert!(
                validate_economic_acceptance_owner(
                    &loaded.source,
                    &inputs,
                    family,
                    &owner_substitution,
                    &formal_needles,
                    pin_path,
                    &format!("owner substitution {index}"),
                )
                .is_err(),
                "support {index} accepted another owner's ID"
            );
            controls += 1;

            let mut evidence_substitution = (*support).clone();
            evidence_substitution.polarity.clone_from(&donor.polarity);
            evidence_substitution
                .formal_refs
                .clone_from(&donor.formal_refs);
            evidence_substitution.pin_ref.clone_from(&donor.pin_ref);
            evidence_substitution.query.clone_from(&donor.query);
            evidence_substitution
                .expected_result
                .clone_from(&donor.expected_result);
            let donor_pin_path = validate_economic_acceptance_pin(
                &inputs,
                &evidence_substitution,
                &format!("evidence substitution {index}"),
            )
            .expect("donor evidence remains an exact pin tuple");
            let donor_formal_needles = evidence_substitution
                .formal_refs
                .iter()
                .map(|reference| {
                    economic_reference_parts(reference, "evidence substitution formal ref")
                        .expect("formal reference")
                        .1
                })
                .collect::<Vec<_>>();
            assert!(
                validate_economic_acceptance_owner(
                    &loaded.source,
                    &inputs,
                    family,
                    &evidence_substitution,
                    &donor_formal_needles,
                    donor_pin_path,
                    &format!("evidence substitution {index}"),
                )
                .is_err(),
                "support {index} accepted another owner's formal/pin/query tuple"
            );
            controls += 1;
        }
        assert_eq!(controls, 640);
    }

    #[test]
    fn current_source_deserializes_into_the_typed_contract() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        assert_eq!(loaded.source.schema_version, EXPECTED_SCHEMA_VERSION);
        assert_eq!(loaded.source.powers.len(), EXPECTED_POWER_COUNT);
    }

    #[test]
    fn current_audit_contract_is_native_without_rewriting_historical_rows() {
        assert_eq!(
            [
                LEDGER_CURRENT_AUDIT_CONTROL_REF,
                closure::CURRENT_AUDIT_CONTROL_REF,
            ],
            [
                concat!("src/checks/ledger.rs::fn negative_", "controls("),
                concat!("src/checks/ledger/closure.rs::fn negative_", "controls("),
            ]
        );
        assert_eq!(
            CURRENT_AUDIT_COMMAND_PREFIX,
            [
                "./verify.sh --refresh full-society-ledger",
                "./verify.sh --refresh constitutional-closure",
                "./verify.sh --emit-receipt new-book-plans/verification-receipts",
            ]
        );

        let context = context();
        let mut candidate = load_source(&context).expect("typed ledger source").source;
        assert!(candidate.scope_audits.iter().any(|audit| {
            audit
                .commands
                .iter()
                .any(|command| command.starts_with("python3 "))
        }));
        let mut inputs = load_static_inputs(&context).expect("static inputs");
        for reference in [
            LEDGER_CURRENT_AUDIT_CONTROL_REF,
            closure::CURRENT_AUDIT_CONTROL_REF,
        ] {
            let path = reference.split_once("::").expect("path::needle").0;
            inputs
                .entry(path.to_owned())
                .or_insert_with(|| fs::read(context.path(path)).expect("Rust control source"));
            validate_repository_reference(&inputs, reference, "native current audit control")
                .expect("unique Rust control symbol");
        }
        candidate.source_version = "native-current-audit-test-v1".to_owned();
        let mut audit = transient_control_audit(&candidate, "Native current audit")
            .expect("native current audit fixture");
        audit.source_version.clone_from(&candidate.source_version);
        audit.scope_sha256 = review_scope_digest(&candidate).expect("scope digest");
        audit.protocol_sha256 = protocol_digest(&inputs).expect("protocol digest");
        candidate.scope_audits.push(audit);

        validate_review_contract(&inputs, &candidate)
            .expect("native current audit with immutable Python-backed history");
    }

    #[test]
    fn active_ledger_renderers_reproduce_through_the_native_refresh_mode() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static inputs");
        let resolution = compute_resolution(&loaded.source).expect("defect resolution");
        let report =
            render_report(&inputs, &loaded.source, &resolution).expect("main ledger report");
        let reader = render_reader(&loaded.source, &resolution).expect("reader ledger report");

        for rendered in [&report, &reader] {
            assert!(rendered.contains(LEDGER_REFRESH_COMMAND));
            assert!(!rendered.contains("python3 new-book-plans/13-full-society-ledger.py"));
            assert!(!rendered.contains("Generated by new-book-plans/13-full-society-ledger.py"));
        }
    }

    #[test]
    fn fingerprints_compute_before_current_audit_validation() {
        let loaded = load_source(&context()).expect("typed ledger source");
        let mut candidate = loaded.source;
        candidate.source_version.push_str("-candidate");
        candidate
            .scope_audits
            .last_mut()
            .expect("current scope audit")
            .scope_sha256 = "0".repeat(64);
        let expected = ScopeFingerprintOutput {
            source_version: candidate.source_version.clone(),
            scope_sha256: review_scope_digest(&candidate).expect("candidate scope digest"),
        };

        let temporary = tempfile::tempdir().expect("temporary fingerprint repository");
        let source_path = temporary.path().join(SOURCE);
        fs::create_dir_all(source_path.parent().expect("source parent"))
            .expect("create source parent");
        fs::write(
            source_path,
            serde_json::to_vec_pretty(&candidate).expect("serialize candidate source"),
        )
        .expect("write candidate source");
        let candidate_context = Context::from_test_root(temporary.path().to_path_buf());
        let actual: ScopeFingerprintOutput = serde_json::from_str(
            &fingerprints(&candidate_context).expect("candidate fingerprints"),
        )
        .expect("typed fingerprint output");
        assert_eq!(actual, expected);
    }

    #[test]
    fn current_source_passes_core_validation_and_has_consistent_gate_state() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        match ledger.closure() {
            Some(projection) => {
                assert_eq!(projection.gate, "gate-a");
                assert_eq!(projection.residual_refs.len(), 4);
                assert_eq!(ledger.document.acceptance_gate.gate_a_status, "passed");
            }
            None => {
                assert_eq!(ledger.document.acceptance_gate.gate_a_status, "not-passed");
                assert!(matches!(
                    ledger
                        .document
                        .scope_audits
                        .last()
                        .map(|audit| audit.result.as_str()),
                    Some("pending" | SCOPE_AUDIT_RESULT)
                ));
            }
        }
    }

    #[test]
    fn current_closure_receipt_binds_the_recorded_audit_transition() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static inputs");
        validate_current_audit_receipts(&context, &inputs, &loaded.source)
            .expect("receipt-bound recorded audit transition");
    }

    #[test]
    fn closure_receipt_rejects_an_unrecorded_audit_successor_when_closed() {
        let context = context();
        let mut loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static inputs");
        if loaded.source.closure_record.0.is_none() {
            assert_eq!(loaded.source.acceptance_gate.gate_a_status, "not-passed");
            validate_current_audit_receipts(&context, &inputs, &loaded.source)
                .expect("an open candidate has no recorded closure transition to replay");
            return;
        }
        let head = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(context.root())
            .output()
            .expect("Git HEAD");
        assert!(head.status.success());
        loaded
            .source
            .closure_record
            .0
            .as_mut()
            .expect("closure record")
            .candidate_commit_sha = String::from_utf8(head.stdout)
            .expect("UTF-8 Git HEAD")
            .trim()
            .to_owned();
        let error = validate_current_audit_receipts(&context, &inputs, &loaded.source)
            .expect_err("unrecorded audit successor must fail");
        assert!(
            error
                .to_string()
                .contains("recorded audit transition is invalid")
        );
    }

    #[test]
    fn current_review_history_preserves_the_visible_first_parent_prefix() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        validate_review_history(&context, &loaded.source)
            .expect("visible first-parent review prefix");
    }

    #[test]
    fn review_history_projection_knows_current_economic_roots() {
        let context = context();
        let bytes = fs::read(context.path(SOURCE)).expect("reviewed ledger source");
        let current: ReviewHistoryProjection =
            serde_json::from_slice(&bytes).expect("current review-history projection");
        assert!(current.economic_power_rule_contracts.is_some());
        assert!(current.economic_carry_rule_contracts.is_some());
        assert!(current.economic_acceptance_cases.is_some());

        for root in [
            "economic_power_rule_contracts",
            "economic_carry_rule_contracts",
            "economic_acceptance_cases",
        ] {
            let mut historical: Value =
                serde_json::from_slice(&bytes).expect("reviewed ledger JSON");
            historical
                .as_object_mut()
                .expect("ledger object")
                .remove(root);
            serde_json::from_value::<ReviewHistoryProjection>(historical)
                .expect("historical review projection may predate economic roots");
        }

        let mut unknown: Value = serde_json::from_slice(&bytes).expect("reviewed ledger JSON");
        unknown
            .as_object_mut()
            .expect("ledger object")
            .insert("unexpected_review_history_root".to_owned(), true.into());
        let error = serde_json::from_value::<ReviewHistoryProjection>(unknown)
            .err()
            .expect("unknown review-history root must fail");
        assert!(error.to_string().contains("unexpected_review_history_root"));
    }

    #[test]
    fn loaded_ledger_retains_every_consumed_input_for_a_final_rehash() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        ledger
            .immutable_snapshot
            .as_ref()
            .expect("immutable snapshot")
            .lock()
            .expect("snapshot lock")
            .assert_unchanged()
            .expect("final immutable-input rehash");
    }

    #[test]
    fn every_review_history_population_is_prefix_protected() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let inputs = load_static_inputs(&context).expect("static inputs");
        let mut current = loaded.source;
        make_control_event(&mut current, &inputs, true).expect("populated review fixture");
        let previous = ReviewHistoryState::from_document(&current);
        validate_review_history_against(&previous, &current).expect("unchanged history");

        let commission = current
            .review_commissions
            .pop()
            .expect("control commission");
        assert!(
            validate_review_history_against(&previous, &current)
                .expect_err("commission deletion must fail")
                .to_string()
                .starts_with("review_commissions:")
        );
        current.review_commissions.push(commission);

        let proposal_title = current.proposals[0].title.clone();
        current.proposals[0].title.push_str(" rewritten");
        assert!(
            validate_review_history_against(&previous, &current)
                .expect_err("proposal rewrite must fail")
                .to_string()
                .starts_with("proposals:")
        );
        current.proposals[0].title = proposal_title;

        let event = current.review_events.pop().expect("control review event");
        assert!(
            validate_review_history_against(&previous, &current)
                .expect_err("event deletion must fail")
                .to_string()
                .starts_with("review_events:")
        );
        current.review_events.push(event);

        let audit_title = current.scope_audits[0].title.clone();
        current.scope_audits[0].title.push_str(" rewritten");
        assert!(
            validate_review_history_against(&previous, &current)
                .expect_err("scope-audit rewrite must fail")
                .to_string()
                .starts_with("scope_audits:")
        );
        current.scope_audits[0].title = audit_title;

        current
            .scope_audits
            .push(current.scope_audits.last().expect("scope audit").clone());
        validate_review_history_against(&previous, &current).expect("append-only successor");
    }

    #[test]
    fn reader_report_matches_the_python_generated_fixture_byte_for_byte() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let resolution = validate_source(&context, &loaded.source).expect("valid source");
        let rendered = render_reader(&loaded.source, &resolution).expect("reader report");
        let expected = context.read(READER_OUTPUT).expect("reader fixture");
        if rendered.as_bytes() != expected.as_bytes() {
            let mismatch = rendered
                .bytes()
                .zip(expected.bytes())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or_else(|| rendered.len().min(expected.len()));
            let start = mismatch.saturating_sub(100);
            let actual_end = (mismatch + 200).min(rendered.len());
            let expected_end = (mismatch + 200).min(expected.len());
            panic!(
                "reader report mismatch at byte {mismatch}; lengths {} != {}; actual {:?}; expected {:?}",
                rendered.len(),
                expected.len(),
                &rendered[start..actual_end],
                &expected[start..expected_end],
            );
        }
    }

    fn assert_exact(label: &str, actual: &str, expected: &str) {
        if actual.as_bytes() == expected.as_bytes() {
            return;
        }
        let mismatch = actual
            .bytes()
            .zip(expected.bytes())
            .position(|(actual, expected)| actual != expected)
            .unwrap_or_else(|| actual.len().min(expected.len()));
        let start = mismatch.saturating_sub(100);
        let actual_end = (mismatch + 200).min(actual.len());
        let expected_end = (mismatch + 200).min(expected.len());
        panic!(
            "{label} mismatch at byte {mismatch}; lengths {} != {}; actual {:?}; expected {:?}",
            actual.len(),
            expected.len(),
            &actual[start..actual_end],
            &expected[start..expected_end],
        );
    }

    #[test]
    fn main_report_matches_the_python_generated_fixture_byte_for_byte() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let resolution = validate_source(&context, &loaded.source).expect("valid source");
        let inputs = load_static_inputs(&context).expect("static inputs");
        let rendered = render_report(&inputs, &loaded.source, &resolution).expect("main report");
        let expected = context.read(OUTPUT).expect("main fixture");
        assert_exact("main report", &rendered, &expected);
    }

    #[test]
    fn coverage_region_matches_the_python_generated_fixture_byte_for_byte() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let current = context.read(COVERAGE_MAP).expect("coverage map");
        let region = render_coverage_region(&loaded.source).expect("coverage region");
        let spliced =
            splice_coverage_map(&current, &region, &loaded.source_bytes).expect("coverage splice");
        assert_exact("coverage map", &spliced, &current);
    }

    #[test]
    fn duplicate_keys_fail_before_typed_deserialization() {
        let error = parse_source(br#"{"spdx":"CC-BY-4.0","spdx":"CC0-1.0"}"#)
            .expect_err("duplicate key must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: spdx")
        );
    }

    #[test]
    fn democratic_policy_boundary_requires_its_exact_term_key_set() {
        let term = Term {
            text: "Reviewed boundary term".into(),
            basis: "constitutional-source".into(),
            source_refs: vec!["new-book-plans/constitution.nibli::source".into()],
            choice_owner: None,
            bounds: None,
            failure_default: None,
        };
        let boundary = DEMOCRATIC_POLICY_BOUNDARY_TERM_KEYS
            .into_iter()
            .map(|key| (key.to_owned(), term.clone()))
            .collect();
        let mut profiles = ProfileTerms::from([("democratic-policy-boundary".into(), boundary)]);

        validate_constitutional_effect_profile_terms(&profiles, "FS-CCE-TEST.profile_terms")
            .expect("the exact key set must pass");

        profiles
            .get_mut("democratic-policy-boundary")
            .expect("boundary profile")
            .remove("review");
        let error =
            validate_constitutional_effect_profile_terms(&profiles, "FS-CCE-TEST.profile_terms")
                .expect_err("a missing key must fail");
        assert!(error.to_string().contains("must contain exactly"));

        profiles
            .get_mut("democratic-policy-boundary")
            .expect("boundary profile")
            .insert("review".into(), term.clone());
        profiles
            .get_mut("democratic-policy-boundary")
            .expect("boundary profile")
            .insert("unexpected".into(), term);
        let error =
            validate_constitutional_effect_profile_terms(&profiles, "FS-CCE-TEST.profile_terms")
                .expect_err("an extra key must fail");
        assert!(error.to_string().contains("must contain exactly"));
    }

    #[test]
    fn economic_carry_contract_fields_are_checker_owned() {
        let contracts = reviewed_economic_carry_contracts();
        validate_economic_carry_rule_contract_rows(&contracts)
            .expect("reviewed economic carry contracts");
        let mut controls = 0_usize;
        for index in 0..contracts.len() {
            for field in 0..18 {
                let mut mutant = contracts.clone();
                let row = &mut mutant[index];
                match field {
                    0 => row.carry_kind.push_str("Mutation"),
                    1 => row.record_kind.push_str("Mutation"),
                    2 => row.temporal_contract.push_str("Mutation"),
                    3 => row.current_kind.push_str("Mutation"),
                    4 => row.current_selection.push_str("Mutation"),
                    5 => row.result_kind.push_str("Mutation"),
                    6 => row.branch.push_str("Mutation"),
                    7 => row.finding_kind.push_str("Mutation"),
                    8 => row.jurisdiction_kind.push_str("Mutation"),
                    9 => row.legal_scope_kind.push_str("Mutation"),
                    10 => row.interest.name.push_str("Mutation"),
                    11 => row.interest.value.push_str("Mutation"),
                    12 => row.interest.scope.push_str("Mutation"),
                    13 => row.requirement.value.push_str("Mutation"),
                    14 => row.requirement.scope.push_str("Mutation"),
                    15 => row.predecessor_record_scope.push_str("Mutation"),
                    16 => row.predecessor_result_scope.push_str("Mutation"),
                    17 => row.successor_event_scope.push_str("Mutation"),
                    _ => unreachable!(),
                }
                assert!(
                    validate_economic_carry_rule_contract_rows(&mutant).is_err(),
                    "carry row {index} field {field} escaped checker policy"
                );
                controls += 1;
            }
        }
        assert_eq!(controls, 54);
    }

    #[test]
    fn economic_carry_rule_atoms_are_watched_exhaustively() {
        let contracts = reviewed_economic_carry_contracts();
        validate_economic_carry_rule_contract_rows(&contracts)
            .expect("reviewed economic carry contracts");
        let context = context();
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        validate_economic_carry_rules(&lines, &contracts).expect("economic carry surface");
        let mut controls = 0_usize;
        for contract in &contracts {
            let marker = format!("# economic-carry-{}: ", contract.carry_kind);
            let marker_index = economic_unique_marker_index(&lines, &marker).expect("carry marker");
            for (statement, expected) in [
                (
                    lines[marker_index + 1],
                    economic_carry_current_atoms(contract).expect("current atoms"),
                ),
                (
                    lines[marker_index + 2],
                    economic_carry_result_atoms(contract).expect("result atoms"),
                ),
            ] {
                let rule = parse_economic_rule(statement).expect("carry rule");
                validate_economic_exact_body(&rule, &expected, "baseline carry rule")
                    .expect("exact carry body");
                for atom in &expected {
                    let mut omission = EconomicRule {
                        body: rule.body.clone(),
                        head: rule.head,
                    };
                    assert!(omission.body.remove(atom.as_str()), "{atom}");
                    assert!(
                        validate_economic_exact_body(&omission, &expected, "carry omission")
                            .is_err(),
                        "carry omission escaped: {atom}"
                    );
                    controls += 1;

                    let mut substitution = EconomicRule {
                        body: rule.body.clone(),
                        head: rule.head,
                    };
                    assert!(substitution.body.remove(atom.as_str()), "{atom}");
                    substitution
                        .body
                        .insert("~contradict($mutation, EconomicMutationBinding)");
                    assert!(
                        validate_economic_exact_body(
                            &substitution,
                            &expected,
                            "carry substitution",
                        )
                        .is_err(),
                        "carry substitution escaped: {atom}"
                    );
                    controls += 1;
                }
            }
        }
        assert_eq!(controls, 4_008);
    }

    #[test]
    fn economic_power_rule_surface_matches_reviewed_contracts() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let family = loaded
            .source
            .coverage_families
            .iter()
            .find(|row| row.id == "FS-CVF-006")
            .expect("economic coverage family");
        validate_economic_rule_contract_rows(
            &loaded.source.economic_power_rule_contracts,
            &family.card_refs,
        )
        .expect("reviewed economic rule contracts");
        assert_eq!(
            typed_fingerprint(
                &loaded.source.economic_power_rule_contracts,
                "economic power rule contracts",
            )
            .expect("economic contract fingerprint"),
            "7a8f4297c0ee6c64fe8580a3d61970f0ad4b280da41566eaf3f2ba020680d19a"
        );
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        validate_economic_power_rule_surface(
            constitution.as_bytes(),
            &loaded.source.economic_power_rule_contracts,
            &loaded.source.economic_carry_rule_contracts,
        )
        .expect("economic rule surface");
    }

    #[test]
    fn economic_power_duty_bridge_atom_mutations_are_watched() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        let mut omissions = 0_usize;
        let mut substitutions = 0_usize;
        let mut head_controls = 0_usize;
        for spec in &ECONOMIC_DUTY_BRIDGES {
            let marker = format!(
                "# economic-duty-{}: {} / {} / FS-POW-{:03}",
                spec.key, spec.duty, spec.standard, spec.power
            );
            let index = economic_unique_marker_index(&lines, &marker).expect("duty marker");
            let rule = parse_economic_rule(lines[index + 1]).expect("duty rule");
            let contract = economic_contract_by_number(
                &loaded.source.economic_power_rule_contracts,
                spec.power,
            )
            .expect("duty power contract");
            let expected = economic_duty_expected_atoms(spec, contract).expect("duty atoms");
            validate_economic_duty_bridge_rule(spec, contract, &rule)
                .expect("unmodified duty bridge");
            for atom in expected {
                let mut mutant = EconomicRule {
                    body: rule.body.clone(),
                    head: rule.head,
                };
                assert!(mutant.body.remove(atom.as_str()), "{}: {atom}", spec.key);
                assert!(
                    validate_economic_duty_bridge_rule(spec, contract, &mutant).is_err(),
                    "{} did not watch {atom}",
                    spec.key
                );
                omissions += 1;

                let dollar = atom.find('$').expect("checker-owned duty atom variable");
                let substitute = format!("{}$mutated_{}", &atom[..dollar], &atom[dollar + 1..]);
                let mut mutant = EconomicRule {
                    body: rule.body.clone(),
                    head: rule.head,
                };
                assert!(mutant.body.remove(atom.as_str()), "{}: {atom}", spec.key);
                assert!(mutant.body.insert(substitute.as_str()), "{substitute}");
                assert!(
                    validate_economic_duty_bridge_rule(spec, contract, &mutant).is_err(),
                    "{} admitted substitution {substitute} for {atom}",
                    spec.key
                );
                substitutions += 1;
            }
            let mutant = EconomicRule {
                body: rule.body.clone(),
                head: "obliged($bearer, $duty, $standard)",
            };
            assert!(
                validate_economic_duty_bridge_rule(spec, contract, &mutant).is_err(),
                "{} did not watch its exact head",
                spec.key
            );
            head_controls += 1;
        }
        assert_eq!(omissions, 10_488);
        assert_eq!(substitutions, 10_488);
        assert_eq!(head_controls, 31);
    }

    #[test]
    fn economic_dependency_join_atom_mutations_are_watched() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let contracts = &loaded.source.economic_power_rule_contracts;
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        let mut omissions = 0_usize;
        let mut substitutions = 0_usize;
        for contract in contracts {
            let number = economic_power_number(&contract.power_ref).expect("power number");
            let marker = format!("# {}: ", contract.power_ref);
            let index = economic_unique_marker_index(&lines, &marker).expect("card marker");
            let rule = parse_economic_rule(lines[index + 2]).expect("result rule");
            let expected = economic_expected_dependency_atoms(contracts, contract, number)
                .expect("dependency atoms");
            validate_economic_dependency_joins(contracts, contract, &rule, number)
                .expect("unmodified dependency joins");
            for atom in expected {
                let mut mutant = EconomicRule {
                    body: rule.body.clone(),
                    head: rule.head,
                };
                assert!(
                    mutant.body.remove(atom.as_str()),
                    "{}: {atom}",
                    contract.power_ref
                );
                assert!(
                    validate_economic_dependency_joins(contracts, contract, &mutant, number)
                        .is_err(),
                    "{} did not watch {atom}",
                    contract.power_ref
                );
                omissions += 1;

                let dollar = atom
                    .find('$')
                    .expect("checker-owned dependency atom variable");
                let substitute = format!("{}$mutated_{}", &atom[..dollar], &atom[dollar + 1..]);
                let mut mutant = EconomicRule {
                    body: rule.body.clone(),
                    head: rule.head,
                };
                assert!(
                    mutant.body.remove(atom.as_str()),
                    "{}: {atom}",
                    contract.power_ref
                );
                assert!(mutant.body.insert(substitute.as_str()), "{substitute}");
                assert!(
                    validate_economic_dependency_joins(contracts, contract, &mutant, number)
                        .is_err(),
                    "{} admitted substitution {substitute} for {atom}",
                    contract.power_ref
                );
                substitutions += 1;
            }
        }
        assert_eq!(
            ECONOMIC_DEPENDENCIES
                .iter()
                .filter(|spec| spec.card == 71)
                .count(),
            2,
            "FS-POW-071 must retain both independent economic dependencies"
        );
        assert_eq!(
            ECONOMIC_DEPENDENCIES
                .iter()
                .filter(|spec| spec.card == 88)
                .count(),
            1,
            "FS-POW-088 must retain its settlement-backbone dependency"
        );
        assert_eq!(
            STATE_DEPENDENCIES
                .iter()
                .filter(|spec| spec.card == 88)
                .count(),
            1,
            "FS-POW-088 must independently retain its ordinary-law dependency"
        );
        assert_eq!(omissions, 645);
        assert_eq!(substitutions, 645);
    }

    #[test]
    fn economic_alternate_origin_selection_mutations_are_watched() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        let mut controls = 0_usize;
        for contract in &loaded.source.economic_power_rule_contracts {
            let number = economic_power_number(&contract.power_ref).expect("power number");
            let alternate_duty =
                economic_alternate_review_duty(number).expect("alternate duty constant");
            let alternate_standard =
                "CertifiedUnavailabilityNoSilenceNoExtensionAndSourceBoundEndStandard";
            for (route, unavailable_reviewer, alternate_reviewer, alternate_scope) in
                ECONOMIC_ALTERNATE_REVIEW_ROUTES
            {
                let marker = format!("# alternate-review-{number:03}-{route}: ");
                let index =
                    economic_unique_marker_index(&lines, &marker).expect("alternate marker");
                let rule = parse_economic_rule(lines[index + 1]).expect("alternate rule");
                validate_economic_alternate_review_rule(
                    &rule,
                    number,
                    route,
                    unavailable_reviewer,
                    alternate_reviewer,
                    alternate_scope,
                )
                .expect("unmodified alternate rule");
                for actor in ["$source", "$evidence", "$review"] {
                    for (value, scope) in [
                        (alternate_reviewer, "DutyBearerScope"),
                        (alternate_duty.as_str(), "DutyScope"),
                        (alternate_standard, "DutyStandardScope"),
                    ] {
                        let atom = format!("observe({actor}, $origin, {value}, {scope})");
                        let mut mutant = EconomicRule {
                            body: rule.body.clone(),
                            head: rule.head,
                        };
                        assert!(mutant.body.remove(atom.as_str()), "{marker}: {atom}");
                        assert!(
                            validate_economic_alternate_review_rule(
                                &mutant,
                                number,
                                route,
                                unavailable_reviewer,
                                alternate_reviewer,
                                alternate_scope,
                            )
                            .is_err(),
                            "{marker} did not watch {atom}"
                        );
                        controls += 1;
                    }
                    for (value, scope) in [
                        ("$alternate_reviewer", "DutyBearerScope"),
                        ("$duty", "DutyScope"),
                        ("$standard", "DutyStandardScope"),
                    ] {
                        let atom = format!("observe({actor}, $origin, {value}, {scope})");
                        let mut mutant = EconomicRule {
                            body: rule.body.clone(),
                            head: rule.head,
                        };
                        assert!(mutant.body.insert(atom.as_str()), "{marker}: {atom}");
                        assert!(
                            validate_economic_alternate_review_rule(
                                &mutant,
                                number,
                                route,
                                unavailable_reviewer,
                                alternate_reviewer,
                                alternate_scope,
                            )
                            .is_err(),
                            "{marker} admitted generic origin selection {atom}"
                        );
                        controls += 1;
                    }
                }
                let mutant = EconomicRule {
                    body: rule.body.clone(),
                    head: "obliged($alternate_reviewer, $duty, $standard)",
                };
                assert!(
                    validate_economic_alternate_review_rule(
                        &mutant,
                        number,
                        route,
                        unavailable_reviewer,
                        alternate_reviewer,
                        alternate_scope,
                    )
                    .is_err(),
                    "{marker} did not watch its exact head"
                );
                controls += 1;
            }
        }
        assert_eq!(controls, 2_660);
    }

    #[test]
    fn economic_power_rule_field_and_requirement_omissions_are_watched() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        let mut controls = 0_usize;
        for contract in &loaded.source.economic_power_rule_contracts {
            let number = economic_power_number(&contract.power_ref).expect("power number");
            let marker = format!("# {}: ", contract.power_ref);
            let index = economic_unique_marker_index(&lines, &marker).expect("card marker");
            let current = parse_economic_rule(lines[index + 1]).expect("current rule");
            let result = parse_economic_rule(lines[index + 2]).expect("result rule");
            for (actor, subject) in [
                ("$source", "$record"),
                ("$record_review", "$record"),
                ("$temporal", "$temporal_record"),
                ("$temporal_review", "$temporal_record"),
            ] {
                for (_, value, scope) in economic_field_pairs(contract) {
                    let atom = format!("observe({actor}, {subject}, {value}, {scope})");
                    let mut mutant = EconomicRule {
                        body: current.body.clone(),
                        head: current.head,
                    };
                    assert!(mutant.body.remove(atom.as_str()), "{atom}");
                    assert!(
                        validate_economic_current_rule(contract, &mutant, number).is_err(),
                        "{} current rule did not watch {atom}",
                        contract.power_ref
                    );
                    controls += 1;

                    let mut mutant = EconomicRule {
                        body: result.body.clone(),
                        head: result.head,
                    };
                    assert!(mutant.body.remove(atom.as_str()), "{atom}");
                    assert!(
                        validate_economic_result_rule(
                            &loaded.source.economic_power_rule_contracts,
                            contract,
                            &mutant,
                            number,
                        )
                        .is_err(),
                        "{} result rule did not watch {atom}",
                        contract.power_ref
                    );
                    controls += 1;
                }
            }
            for actor in ["$source", "$evidence", "$review"] {
                for (_, value, scope) in economic_field_pairs(contract) {
                    let atom = format!("observe({actor}, $result, {value}, {scope})");
                    let mut mutant = EconomicRule {
                        body: result.body.clone(),
                        head: result.head,
                    };
                    assert!(mutant.body.remove(atom.as_str()), "{atom}");
                    assert!(
                        validate_economic_result_rule(
                            &loaded.source.economic_power_rule_contracts,
                            contract,
                            &mutant,
                            number,
                        )
                        .is_err(),
                        "{} result rule did not watch {atom}",
                        contract.power_ref
                    );
                    controls += 1;
                }
                for (value, scope) in economic_requirement_pairs(contract) {
                    let atom = format!("observe({actor}, $result, {value}, {scope})");
                    let mut mutant = EconomicRule {
                        body: result.body.clone(),
                        head: result.head,
                    };
                    assert!(mutant.body.remove(atom.as_str()), "{atom}");
                    assert!(
                        validate_economic_result_rule(
                            &loaded.source.economic_power_rule_contracts,
                            contract,
                            &mutant,
                            number,
                        )
                        .is_err(),
                        "{} result rule did not watch {atom}",
                        contract.power_ref
                    );
                    controls += 1;
                }
            }
        }
        assert_eq!(controls, 6_036);
    }

    #[test]
    fn economic_alternate_review_cannot_complete_a_power() {
        let context = context();
        let loaded = load_source(&context).expect("typed ledger source");
        let constitution = context
            .read("new-book-plans/constitution.nibli")
            .expect("constitution");
        let block = economic_block(&constitution).expect("economic block");
        let lines = block.lines().collect::<Vec<_>>();
        let marker = "# alternate-review-061-record-review: ";
        let index = economic_unique_marker_index(&lines, marker).expect("alternate marker");
        let statement = lines[index + 1];
        let rule = parse_economic_rule(statement).expect("alternate rule");
        let replacement = statement.replace(
            &format!(" -> {}.", rule.head),
            " -> complete($power_alternate_record_reviewer, FSPOW_061, $power_record).",
        );
        assert_ne!(statement, replacement);
        let mutant = constitution.replacen(statement, &replacement, 1);
        validate_economic_power_rule_surface(
            mutant.as_bytes(),
            &loaded.source.economic_power_rule_contracts,
            &loaded.source.economic_carry_rule_contracts,
        )
        .expect_err("an alternate-review branch cannot complete a power");
    }

    #[test]
    fn scenario_applicability_rejects_a_nested_extra_field() {
        let error = serde_json::from_str::<ScenarioApplicability>(
            r#"{"answer":"all declared scenarios","unexpected":"not reviewed"}"#,
        )
        .expect_err("scenario applicability payload extras must fail");
        assert!(
            error.to_string().contains("did not match any variant"),
            "unexpected strict-union error: {error}"
        );
    }

    #[test]
    fn scenario_applicability_rejects_answer_and_deferred_ref_together() {
        let error = serde_json::from_str::<ScenarioApplicability>(
            r#"{"answer":"all declared scenarios","deferred_ref":"decision.md::scope"}"#,
        )
        .expect_err("scenario applicability must select exactly one payload");
        assert!(
            error.to_string().contains("did not match any variant"),
            "unexpected strict-union error: {error}"
        );
    }

    #[test]
    fn typed_claim_and_defect_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count =
            negative_controls_claims_and_defects(&ledger).expect("claim and defect controls");
        assert_eq!(count, 42);
    }

    #[test]
    fn envelope_role_and_body_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count = negative_controls_envelope_roles_bodies(&ledger)
            .expect("envelope, role, and body controls");
        assert_eq!(count, 65);
    }

    #[test]
    fn dependency_and_scenario_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count = negative_controls_dependencies_and_scenarios(&ledger)
            .expect("dependency and scenario controls");
        assert_eq!(count, 41);
    }

    #[test]
    fn power_and_effect_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count = negative_controls_power_and_effects(&ledger)
            .expect("power and constitutional-effect controls");
        assert_eq!(count, 69);
    }

    #[test]
    fn closure_and_scope_audit_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count =
            negative_controls_closure_and_audit(&ledger).expect("closure and scope-audit controls");
        assert_eq!(count, 30);
    }

    #[test]
    fn optional_review_mutations_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        let count = negative_controls_optional_review(&ledger).expect("optional-review controls");
        assert_eq!(count, 24);
    }

    #[test]
    fn semantic_controls_are_watched() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        assert_eq!(semantic_controls(&ledger).expect("semantic controls"), 3);
    }

    #[test]
    fn all_structural_controls_execute_exactly_once() {
        let ledger = load_and_validate(&context()).expect("validated ledger");
        assert_eq!(
            negative_controls(&ledger).expect("all structural controls"),
            STRUCTURAL_CONTROL_COUNT
        );
    }
}
