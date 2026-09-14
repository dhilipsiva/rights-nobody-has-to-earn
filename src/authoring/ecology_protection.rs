// SPDX-License-Identifier: MIT OR Apache-2.0

//! Protection, liability and continuity are separate from permission and guilt.

use super::Card;

pub(super) const FLOOR: &[&str] = &[
    "secure",
    "eats",
    "dwell",
    "healthy",
    "learn",
    "expresses",
    "believe",
    "meets",
];

pub(super) fn cards() -> Vec<Card> {
    vec![
        Card::new(
            "unequal-exposure",
            "ECUnequalEnvironmentalExposure",
            "Class1EnvironmentalEqualityProtection",
        )
        .fields(&[
            ("$exposure", "DistributionalExposureEvidence"),
            ("$ground", "EqualityGroundOrInteraction"),
            (
                "ECIndependentDirectIndirectSystemicOrIntersectingDisadvantageFinding",
                "ExposureDisadvantage",
            ),
            (
                "ECExistingSubstantiveEqualityAntiSubordinationAndAccommodationRoutes",
                "ExistingEqualityRoute",
            ),
            (
                "ECNoDiagnosticScoreReusedAsIndividualWorthRiskOrEligibility",
                "EqualityDataWall",
            ),
            (
                "ECAccessibleIndividualAndSystemicRemedyWithoutRetaliation",
                "EqualityProtection",
            ),
        ])
        .depends(&["environmental-claim"])
        .effect("obliged($reader, ECUseExistingSubstantiveEqualityProtectionAndRemedy, $record)")
        .effect("prevents($record, ECDiagnosticExposureAsIndividualWorthOrEligibility)"),
        Card::new(
            "immediate-protection",
            "ECImmediateNoFaultProtection",
            "Class6UrgentEnvironmentalContinuity",
        )
        .fields(&[
            ("$project", "ThreateningActivity"),
            ("$condition", "ThreatenedProtectedCondition"),
            (
                "ECCredibleActualOrThreatenedHarmNeedingPromptAction",
                "UrgentThreat",
            ),
            (
                "ECNecessaryProportionatePreventionCessationContainmentAndPublicRestoration",
                "ProtectiveResponse",
            ),
            (
                "ECNoWaitForFinalFaultCausationCostsOrControllerIdentity",
                "NoFaultTiming",
            ),
            (
                "ECHumanFloorAndAnimalCareContinuityDuringResponse",
                "ProtectiveContinuity",
            ),
            ("ECIndependentPromptReviewReasonsAndRemedy", "UrgentReview"),
            (
                "ECNoCriminalGuiltPunishmentCustodyOrGeneralEmergencyPower",
                "ProtectiveLimit",
            ),
        ])
        .duty("ECProvideImmediatePreventionContainmentAndRestorationWithoutAwaitingFault")
        .effect("obliged($alternate, ECPreserveUrgentHumanAndAnimalContinuity, $record)"),
        Card::new(
            "hazard-liability",
            "ECHazardousRestorationLiability",
            "Class4StrictRestorationCostLiability",
        )
        .fields(&[
            ("$activity", "HazardousActivity"),
            ("$subject", "LiableActor"),
            ("$harm", "CausallyConnectedHarm"),
            ("$costs", "ReasonableResponseCosts"),
            (
                "ECIndependentlyAdjudicatedInherentlyHazardousActivity",
                "InherentHazard",
            ),
            (
                "ECIndependentlyAdjudicatedCausalConnectionToHarm",
                "RestorationCausation",
            ),
            (
                "ECReasonableRestorationAndResponseCostsEstablished",
                "RestorationCosts",
            ),
            (
                "ECPermitCompetentCareAndNoIntentDoNotDefeatRestorationTier",
                "StrictTier",
            ),
            (
                "ECSeparateCulpabilityAndDueProcessBeforePunitiveConsequence",
                "PunitiveSeparation",
            ),
            (
                "ECHumanStandingFloorLibertyAndCollectiveTitleUnchanged",
                "LiabilityRightsWall",
            ),
        ])
        .extra("~($subject = $operator)")
        .extra("$operator = FSBOD_17")
        .default("$operator", "FSBOD_17")
        .effect("obliged($subject, ECPayAdjudicatedReasonableRestorationAndResponseCosts, $record)")
        .duty("ECEnforceRestorativeLiabilityWithoutInventingCulpability"),
        Card::new(
            "contribution-liability",
            "ECContributionOrControlLiability",
            "Class4AdjudicatedContributionLiability",
        )
        .fields(&[
            ("$activity", "HarmfulActivity"),
            ("$subject", "LiableActor"),
            ("$harm", "CausallyConnectedHarm"),
            ("$costs", "ReasonableResponseCosts"),
            ("$basis", "ContributionOrControlGround"),
            (
                "ECIndependentlyAdjudicatedCausallyMaterialContributionOrControl",
                "ContributionEvidence",
            ),
            (
                "ECIndividualizedReasonsNoticeHearingChallengeAndProportionateAllocation",
                "ContributionProcedure",
            ),
            (
                "ECInvestorWorkerFamilyMembershipAndOwnershipStatusAloneInsufficient",
                "StatusInsufficiency",
            ),
            (
                "ECNoCriminalPunishmentWithoutSeparateCulpabilityAndDueProcess",
                "PunitiveSeparation",
            ),
            (
                "ECHumanStandingFloorLibertyAndCollectiveTitleUnchanged",
                "LiabilityRightsWall",
            ),
        ])
        .extra("member($basis, ECContributionGround)")
        .extra("~($subject = $operator)")
        .extra("$operator = FSBOD_17")
        .default("$operator", "FSBOD_17")
        .default("$basis", "ECCausalMaterialControl")
        .effect("obliged($subject, ECProvideAdjudicatedContributionBasedRestoration, $record)")
        .duty("ECPreserveIndividualizedCausalReasonsAndIndependentRemedy"),
        Card::new(
            "orphan-restoration",
            "ECPublicRestorationContinuity",
            "Class6OrphanRestorationDuty",
        )
        .fields(&[
            ("$condition", "DamagedProtectedCondition"),
            ("$failure", "ResponsibleActorFailure"),
            (
                "ECPositiveUrgentNeedForPublicRestorationAndContinuity",
                "PublicRestorationNeed",
            ),
            (
                "ECNoWaitForIdentificationSolvencyInsuranceOrFinalRecovery",
                "RecoverySeparation",
            ),
            (
                "ECPreserveHumanFloorAnimalCareAndNecessaryPublicRepair",
                "OrphanContinuity",
            ),
            (
                "ECLaterCausalRecoverySeparatelyAdjudicatedWithoutSocializingPermission",
                "LaterRecovery",
            ),
        ])
        .extra("member($failure, ECResponsibleActorFailure)")
        .default("$failure", "ECResponsibleActorUnknown")
        .duty("ECProvidePublicRestorationAndContinuityBeforeCostRecovery"),
        Card::new(
            "human-reparation",
            "ECSeparateHumanEnvironmentalReparation",
            "Class4HumanHarmReparation",
        )
        .fields(&[
            ("$subject", "InjuredPresentPerson"),
            ("$liable_actor", "LiableActor"),
            ("$harm", "SeparatelyEstablishedHumanHarm"),
            ("$reparation", "ProportionateReparation"),
            (
                "ECIndependentHumanHarmCausationAndRemedyDecision",
                "ReparationDecision",
            ),
            (
                "ECExistingEconomicPropertyProcessAndLegitimateRelianceSafeguards",
                "EconomicSafeguards",
            ),
            (
                "ECNoPaymentExtinguishesEcologicalRestorationOrPurchasesFutureBreach",
                "SeparateRestoration",
            ),
            (
                "ECStoppingUnlawfulHarmNotAutomaticallyCompulsoryAcquisition",
                "AcquisitionBoundary",
            ),
        ])
        .extra("~($liable_actor = $review)")
        .extra("~($liable_actor = $reader)")
        .extra("~($liable_actor = $operator)")
        .extra("$operator = FSBOD_17")
        .default("$operator", "FSBOD_17")
        .effect("obliged($liable_actor, ECProvideSeparatelyAdjudicatedHumanReparation, $record)")
        .duty("ECPreserveEcologicalRepairAlongsideHumanReparation"),
        Card::new(
            "collective-reparation",
            "ECSeparateCollectiveEnvironmentalReparation",
            "Class4CollectiveHarmReparation",
        )
        .fields(&[
            ("$subject", "AffectedCollective"),
            ("$title", "CollectiveTitle"),
            ("$liable_actor", "LiableActor"),
            ("$harm", "SeparatelyEstablishedCollectiveHarm"),
            ("$reparation", "ProportionateCollectiveReparation"),
            (
                "ECIndependentCollectiveHarmAndRepresentativeMandateEvidence",
                "CollectiveReparationEvidence",
            ),
            (
                "ECProtectedTitleGovernmentCultureKnowledgeAndActualConsentBoundary",
                "CollectiveRightsBoundary",
            ),
            (
                "ECNoMoneyWaivesConsentTransfersTitleOrErasesEcologicalRestoration",
                "CollectiveNonSubstitution",
            ),
            (
                "ECAccessibleConsultationTimeAccommodationReasonsAndIndependentRemedy",
                "CollectiveProcedure",
            ),
        ])
        .extra("~($liable_actor = $review)")
        .extra("~($liable_actor = $reader)")
        .extra("~($liable_actor = $operator)")
        .extra("$operator = FSBOD_17")
        .default("$operator", "FSBOD_17")
        .effect(
            "obliged($liable_actor, ECProvideSeparatelyAdjudicatedCollectiveReparation, $record)",
        )
        .effect(
            "prevents($record, ECReparationAsConsentTitleTransferOrEcologicalRepairSubstitute)",
        ),
        Card::new(
            "dual-continuity",
            "ECDualFloorAndCeilingContinuity",
            "Class6LeastHarmDualContinuity",
        )
        .fields(&[
            ("$floor", "ExistingMaterialFloorItem"),
            ("$axis", "Axis"),
            ("$condition", "ProtectedCondition"),
            ("$route", "ImmediateLeastHarmRoute"),
            ("$transition", "SourceBoundTransition"),
            (
                "ECPositiveEvidenceNoPresentRouteMeetsBothFloorAndCeiling",
                "DualIncompatibilityEvidence",
            ),
            (
                "ECActualFeasibleAlternativesAndProcurementEffortExamined",
                "DualAlternatives",
            ),
            (
                "ECImmediateHumanContinuityByLeastHarmRoute",
                "ImmediateFloorContinuity",
            ),
            (
                "ECRecordEveryUnmetFloorPortionAsFailure",
                "FloorFailurePolarity",
            ),
            (
                "ECRecordEveryCeilingBreachAsBreach",
                "CeilingFailurePolarity",
            ),
            (
                "ECNoBudgetDelayMonopolyWithholdingOrProcurementRefusalAsImpossibility",
                "FalseScarcity",
            ),
            (
                "ECExistingPhysicalScarcityInterfaceAloneGovernsAllocation",
                "ScarcityAllocationBoundary",
            ),
            (
                "ECObtainAlternativesRepairHarmReassessAndPreventRepetition",
                "DualRepair",
            ),
            (
                "ECNoReproductiveFamilyMigrationCustodyOrWorthCoercion",
                "DualRightsWall",
            ),
        ])
        .extra("member($axis, ECAxis)")
        .extra("member($floor, ECExistingMaterialFloorItem)")
        .default("$floor", "\"eats\"")
        .default("$axis", "ECWaterAxis")
        .duty("ECPreserveImmediateFloorWhileObtainingAlternativesAndRepairingBreach")
        .effect("prevents($record, ECUnmetFloorRenamedSuccess)")
        .effect("prevents($record, ECCeilingBreachRenamedCompliance)"),
        Card::new(
            "non-evasion",
            "ECEcologicalNonEvasionProtection",
            "Class8CrossBoundaryEcologicalConstraint",
        )
        .fields(&[
            ("$activity", "CrossBoundaryActivity"),
            ("$controller", "MaterialController"),
            ("$origin", "OriginJurisdiction"),
            ("$destination", "DestinationJurisdiction"),
            (
                "ECCommonMinimaInterregionalEffectsAndPortabilityRemainBinding",
                "CommonCompetence",
            ),
            (
                "ECRegionalResidualPolicyAndStrongerCompatibleProtectionPreserved",
                "RegionalCompetence",
            ),
            (
                "ECNoTradeFinanceProcurementAffiliateSupplyChainOrFlagEscape",
                "EvasionMeans",
            ),
            (
                "ECNoConservationDispossessionAssimilationOrCollectiveConsentEvasion",
                "CollectiveBoundary",
            ),
            (
                "ECNoNewCustomsSanctionsTreatyTransferOrInternationalForcePower",
                "ExternalPowerBoundary",
            ),
            (
                "ECPublicReasonsIndependentCrossBoundaryChallengeAndRepair",
                "CrossBoundaryRemedy",
            ),
        ])
        .duty("ECPreventExternalizingProhibitedHarmAndPreserveCollectiveRights")
        .effect("prevents($activity, ECCorporatePropertyContractOrJurisdictionEscape)"),
    ]
}

pub(super) fn vocabulary_rules() -> Vec<String> {
    let mut rules = Vec::new();
    for floor in FLOOR {
        rules.push(format!("all $writer: all $record: observe($writer, $record, \"{floor}\", ECExistingMaterialFloorItemScope) -> member(\"{floor}\", ECExistingMaterialFloorItem)."));
    }
    for (scope, kind, values) in [
        (
            "ContributionOrControlGround",
            "ECContributionGround",
            &[
                "ECCausalContribution",
                "ECCausalDirection",
                "ECCausalAuthorization",
                "ECCausalMaterialControl",
                "ECKnowingCausalFacilitation",
                "ECCausalConcealment",
                "ECCausalEvasion",
            ][..],
        ),
        (
            "ResponsibleActorFailure",
            "ECResponsibleActorFailure",
            &[
                "ECResponsibleActorUnknown",
                "ECResponsibleActorAbsent",
                "ECResponsibleActorInsolvent",
                "ECResponsibleActorUnableToActInTime",
            ][..],
        ),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {kind})."));
        }
    }
    rules
}
