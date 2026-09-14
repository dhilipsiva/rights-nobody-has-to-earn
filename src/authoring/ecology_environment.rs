// SPDX-License-Identifier: MIT OR Apache-2.0

//! Distinct human-right, scientific and commons effects.

use super::Card;

pub(super) const AXES: &[&str] = &[
    "ECClimateAxis",
    "ECWaterAxis",
    "ECAirAxis",
    "ECSoilAxis",
    "ECBiodiversityAxis",
    "ECHabitatAxis",
    "ECExtractionAxis",
    "ECWasteAxis",
];
pub(super) const INTERESTS: &[&str] = &[
    "ECCleanAirInterest",
    "ECSafeWaterInterest",
    "ECHealthySoilAndFoodSystemsInterest",
    "ECHazardousExposureProtectionInterest",
];
pub(super) const REACH: &[&str] = &[
    "ECPublicTierOrInstitution",
    "ECContractorOrDelegatedFunction",
    "ECCreatesOrControlsHarm",
    "ECAuthorizesHarm",
    "ECFinanceWithMaterialControl",
    "ECMaterialConcealmentOfHarm",
];
pub(super) const NONFUNGIBLE: &[&str] = &[
    "ECCeilingBreach",
    "ECIrreplaceableSystem",
    "ECUniqueSystem",
    "ECSacredSystem",
    "ECFunctionallyNonSubstitutableSystem",
    "ECLocalEnvironmentalOrHealthHarm",
    "ECEqualityOrCulturalHarm",
    "ECCollectiveTitleOrConsentHarm",
    "ECIrreversibleLoss",
];

pub(super) fn vocabulary_rules() -> Vec<String> {
    let mut rules = Vec::new();
    for (scope, vocabulary, values) in [
        ("Axis", "ECAxis", AXES),
        (
            "EnvironmentalInterest",
            "ECEnvironmentalInterest",
            INTERESTS,
        ),
        ("DirectReach", "ECDirectReach", REACH),
        ("NonfungibleGround", "ECNonfungibleGround", NONFUNGIBLE),
        (
            "ClaimGround",
            "ECEnvironmentalClaimGround",
            &[
                "ECActualEnvironmentalInterference",
                "ECCredibleThreatenedSeriousOrIrreversibleInterference",
            ][..],
        ),
        (
            "ThreatCategory",
            "ECSeriousOrIrreversibleThreat",
            &["ECSeriousHarm", "ECIrreversibleHarm"][..],
        ),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {vocabulary})."));
        }
    }
    rules
}

pub(super) fn cards() -> Vec<Card> {
    let mut cards = vec![
        Card::new("environmental-claim", "ECEnvironmentalRightClaim", "Class3EnvironmentalRight")
            .fields(&[
                ("$interest", "EnvironmentalInterest"), ("$reach", "DirectReach"),
                ("$claim_ground", "ClaimGround"),
                ("ECPositivePresentHumanClaimantWithoutCivilIdentityGuiltOrPermissionCondition", "HumanClaimantFinding"),
                ("ECNoProprietaryInjuryOrOwnershipPrerequisite", "ClaimAccess"),
                ("ECComprehensibleTimelyInformation", "Information"),
                ("ECDisclosedUncertaintyAndSourceLimits", "Uncertainty"),
                ("ECAccessiblePriorParticipationWithExposedPeopleAndCollectives", "Participation"),
                ("ECPriorDirectIndirectCumulativeDistributionalCrossBoundaryLongLatencyAssessment", "Assessment"),
                ("ECReasonsAddressingAlternativesAndMaterialObjections", "ReasonsDuty"),
                ("ECProtectionFromReportingInvestigationAndChallengeRetaliation", "Retaliation"),
                ("ECIndependentInterimProtectionCorrectionAndEffectiveRemedy", "ProtectionRoute"),
                ("ECPublicContinuityWhenPrivateControllerOrProviderFails", "PrivateFailureContinuity"),
                ("ECNoNewFloorItemAndNoPersonalOutcomeCertification", "FloorBoundary"),
            ])
            .extra("member($interest, ECEnvironmentalInterest)")
            .extra("member($reach, ECDirectReach)")
            .extra("member($claim_ground, ECEnvironmentalClaimGround)")
            .default("$interest", "ECCleanAirInterest")
            .default("$reach", "ECCreatesOrControlsHarm")
            .default("$claim_ground", "ECCredibleThreatenedSeriousOrIrreversibleInterference")
            .duty("ECProtectIndependentEnvironmentalClaim")
            .effect("obliged($reader, ECReviewEnvironmentalInformationParticipationProtectionAndRemedy, $record)"),
        Card::new("science", "ECIndependentScientificFinding", "Class7ScientificEvidence")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$proposal", "ProposedStandard"),
                ("ECIndependentlyReviewedScientificEvidence", "ScientificReview"),
                ("ECAssumptionsAndUncertaintyDisclosed", "Assumptions"),
                ("ECCumulativeAndCompoundEffectsExamined", "CumulativeEffects"),
                ("ECDistributionalAndExposedPopulationEffectsExamined", "DistributionalEffects"),
                ("ECCrossBoundaryAndSupplyChainEffectsExamined", "CrossBoundaryEffects"),
                ("ECLongLatencyAndFullDurationEffectsExamined", "LongLatencyEffects"),
                ("ECLocalScientificAndIndigenousKnowledgeIndependentlyAssessed", "KnowledgeSources"),
                ("ECConflictsDissentAndCorrectionDisclosed", "DissentAndConflict"),
                ("ECScienceSuppliesEvidenceNotLawOrAnOperatedMeasurement", "ScienceLawSeparation"),
            ])
            .extra("member($axis, ECAxis)")
            .extra("$operator = FSBOD_28")
            .default("$axis", "ECClimateAxis")
            .default("$operator", "FSBOD_28")
            .duty("ECPreserveReviewedEvidenceUncertaintyAndCorrection"),
        Card::new("precaution", "ECProportionatePrecaution", "Class9Precaution")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$threat", "ThreatCategory"), ("$project", "ProposedActivity"),
                ("ECCrediblePlausibleThreatNotUncertaintyAsSafety", "ThreatEvidence"),
                ("ECProportionateToConsequenceReversibilityExposureAndControl", "Proportionality"),
                ("ECProponentDisclosesEvidenceAndMaterialLimits", "ProponentEvidence"),
                ("ECFeasibleLessHarmfulAlternativesIncludingNotProceeding", "Alternatives"),
                ("ECBurdenOnProposedAuthorityNotPersonalGuilt", "BurdenLimit"),
                ("ECNoZeroRiskRuleOrFloorWithholding", "OrdinaryLifeAndFloor"),
            ])
            .depends(&["science"])
            .extra("member($threat, ECSeriousOrIrreversibleThreat)")
            .default("$threat", "ECIrreversibleHarm")
            .duty("ECRequireEvidenceAlternativesAndProportionateInterimProtection"),
        Card::new("ceiling", "ECCurrentLegalCeiling", "Class9EcologicalCeiling")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$proposal", "ProposedStandard"), ("$standard", "EnactedStandard"),
                ("$standard_version", "StandardVersion"), ("$bill", "EnactingBill"),
                ("ECCeilingMinimumOrBudgetWithSeparateAxisAndUnit", "StandardForm"),
                ("ECPublishedDemocraticallyEnactedVersionNotScientificProposalAlone", "EnactmentFinding"),
                ("ECEqualOrStrongerProtectionThanReviewedScientificMinimum", "ScientificMinimum"),
                ("ECPublicAllocationRulesWithinEachSeparateEcologicalLimit", "Allocation"),
                ("ECMonitoringDisclosureCorrectionChallengeAndRepair", "MonitoringAndRepair"),
                ("ECNoCompensatingOneAxisWithAnother", "AxisSeparation"),
                ("ECQualitativeNonDestructionSurvivesMissingQuantification", "NonDestruction"),
                ("ECPublishedApplicableTerritoryPopulationAndPeriod", "Applicability"),
                ("ECNoBudgetEmergencyCorporateOrJurisdictionalDerogation", "NoDerogation"),
            ])
            .depends(&["science"])
            .extra("$operator = FSBOD_02")
            .default("$operator", "FSBOD_02")
            .duty("ECApplyOnlyExactEnactedEcologicalLimit")
            .effect("prevents($standard, ECOffsetBudgetEmergencyOrOtherAxisWaiver)"),
        Card::new("nonregression", "ECOutcomePreservingReplacement", "Class9OutcomeNonRegression")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$standard", "EnactedStandard"), ("$standard_version", "StandardVersion"),
                ("$prior_standard", "PriorStandard"), ("$prior_version", "PriorStandardVersion"),
                ("$comparison", "OutcomeComparisonEvidence"),
                ("ECIndependentEqualOrStrongerProtectedOutcomesFinding", "OutcomeNonRegression"),
                ("ECChangedScienceRecalibrationNotNumericFreezeOrConvenience", "Recalibration"),
                ("ECNoCostCompetitivenessPreferenceOrAdministrativeEaseException", "RegressionExcuses"),
                ("ECPublicReasonsParticipationAndIndependentChallenge", "ReplacementProcedure"),
            ])
            .depends(&["ceiling"])
            .duty("ECPreserveProtectedOutcomesDuringStandardReplacement"),
        Card::new("axis-compliance", "ECReviewedAxisCompliance", "Class9ExactAxisCompliance")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"), ("$project", "ProposedActivity"),
                ("$activity_version", "ActivityVersion"), ("$assessment", "AssessmentSet"),
                ("$standard", "EnactedStandard"), ("$standard_version", "StandardVersion"),
                ("ECPositiveIndependentComplianceWithThisAxisNotAnAggregatePass", "AxisCompliance"),
                ("ECDirectIndirectCumulativeCompoundAndDistributionalEffectsAssessed", "EffectsAssessment"),
                ("ECCrossBoundarySupplyChainLongLatencyAndFullDurationAssessed", "ExtendedEffects"),
                ("ECNoExternalizationJurisdictionShoppingOrCorporateFormEscape", "NonEvasion"),
                ("ECUncertaintyLimitsAffectedPeopleAndAlternativesPubliclyAddressed", "AssessmentReasons"),
            ])
            .depends(&["ceiling"])
            .duty("ECRetainExactAxisAssessmentAndContinuingCorrection"),
        Card::new("ceiling-review", "ECCeilingLegalityReview", "Class4IndependentLegalityReview")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$standard", "EnactedStandard"), ("$standard_version", "StandardVersion"),
                ("ECIndependentLegalityReviewOfExactCeilingAndRecord", "CeilingLegality"),
                ("ECCourtDoesNotChoosePolicyEnactLawOrClaimScientificTruth", "ReviewSeparation"),
                ("ECReasonsAccessibleInterimProtectionCorrectionAndRemedy", "LegalityRemedy"),
            ])
            .depends(&["ceiling"])
            .extra("$operator = FSBOD_17")
            .default("$operator", "FSBOD_17")
            .duty("ECGiveBoundedCeilingLegalityReasonsAndEffectiveRemedy"),
        Card::new("residual-restoration", "ECReplaceableResidualMeasure", "Class9RestorationSequence")
            .concludes("permits($operator, ECReplaceableResidualMeasure, $record)")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"), ("$project", "ProposedActivity"),
                ("$residual", "ResidualDamage"), ("$measure", "ResidualMeasure"),
                ("ECAvoidanceFirstIncludingNotProceeding", "Avoidance"),
                ("ECMinimizedAtSourceAfterAvoidance", "SourceMinimization"),
                ("ECNecessaryInPlaceRestorationBeforeResidualCompensation", "InPlaceRestoration"),
                ("ECIndependentlyEstablishedReplaceableResidualOnly", "Replaceability"),
                ("ECNoCeilingBreachIrreplaceableUniqueSacredOrLocalRightsLoss", "HardLimits"),
                ("ECAdditionalBeyondExistingDuties", "Additionality"),
                ("ECDurableForFullRelevantLossAndRiskDuration", "Durability"),
                ("ECFunctionalAndPlaceRelevantIndependentEquivalence", "Equivalence"),
                ("ECNoDoubleCountingOrDuplicateCredit", "DoubleCounting"),
                ("ECMonitorVerifyCorrectAndRepairFailedMeasure", "ResidualFailureRepair"),
                ("ECHumanAndCollectiveReparationRemainSeparate", "ReparationSeparation"),
            ])
            .extra("member($axis, ECAxis)")
            .extra("~oppose($condition, $project, $axis)")
            .default("$axis", "ECWaterAxis")
            .duty("ECExecuteOnlyBoundedResidualRestorationSequence"),
        Card::new("nonfungibility", "ECNonfungibleProtectionFinding", "Class9NonSubstitution")
            .fields(&[
                ("$axis", "Axis"), ("$condition", "ProtectedCondition"),
                ("$ground", "NonfungibleGround"), ("$project", "ProposedActivity"),
                ("ECIndependentEvidenceOfExactProtectedConditionAndGround", "ProtectedGroundEvidence"),
                ("ECOffsetCreditPaymentOrRemoteRestorationDoesNotExcuseGround", "NonSubstitution"),
                ("ECHumanCollectiveReparationSeparateFromEcologicalRepair", "ReparationSeparation"),
            ])
            .extra("member($axis, ECAxis)")
            .extra("member($ground, ECNonfungibleGround)")
            .default("$axis", "ECHabitatAxis")
            .default("$ground", "ECIrreplaceableSystem")
            .effect("oppose($condition, $project, $axis)")
            .effect("prevents($project, ECOffsetAsPermissionForNonfungibleHarm)")
            .duty("ECAvoidStopAndRepairNonfungibleHarm"),
    ];
    let mut authorization = Card::new(
        "high-consequence-basis",
        "ECHighConsequenceAuthorizationBasis",
        "Class9ReviewedHighConsequenceBasis",
    )
    .fields(&[
        ("$project", "ProposedActivity"),
        ("$activity_version", "ActivityVersion"),
        ("$assessment", "AssessmentSet"),
        ("$proponent", "ActivityProponent"),
        ("$mandate", "SpecificStatutoryMandate"),
        // Export the exact condition from each independently completed axis.
        // The execution rule reads these fields when applying non-substitution
        // bars; a variable occurring only in a negative guard is not a key.
        ("$climate_condition", "ClimateProtectedCondition"),
        ("$water_condition", "WaterProtectedCondition"),
        ("$air_condition", "AirProtectedCondition"),
        ("$soil_condition", "SoilProtectedCondition"),
        ("$biodiversity_condition", "BiodiversityProtectedCondition"),
        ("$habitat_condition", "HabitatProtectedCondition"),
        ("$extraction_condition", "ExtractionProtectedCondition"),
        ("$waste_condition", "WasteProtectedCondition"),
        (
            "ECLawSpecificallyDelegatesOnlyThisReviewedActivity",
            "StatutoryDelegation",
        ),
        (
            "ECNecessaryCompatiblePurposeAndLeastHarmfulFeasibleAlternative",
            "HighConsequenceNecessity",
        ),
        (
            "ECPriorAccessibleParticipationIncludingExposedHumansAnimalsAndCollectives",
            "HighConsequenceParticipation",
        ),
        (
            "ECPublicReasonsAddressMaterialObjectionsAndUncertainty",
            "HighConsequenceReasons",
        ),
        (
            "ECIndependentReviewInterimReliefAndCorrection",
            "HighConsequenceReview",
        ),
        (
            "ECNoFindingWaivesHumanFloorCollectiveConsentAnimalCoreOrOtherAxis",
            "HighConsequenceHardLimits",
        ),
        (
            "ECMonitoringRepairAndSourceBoundEndNotAnOperatedOutcome",
            "HighConsequenceEnd",
        ),
    ])
    .extra("$operator = FSBOD_29")
    .extra("~($operator = $proponent)")
    .default("$operator", "FSBOD_29")
    .duty("ECMonitorCorrectAndEndOnlyThisHighConsequenceAuthorization");
    for (axis, key) in AXES.iter().zip([
        "climate",
        "water",
        "air",
        "soil",
        "biodiversity",
        "habitat",
        "extraction",
        "waste",
    ]) {
        authorization = authorization.repeated("axis-compliance", key, &[("$axis", axis)]);
    }
    cards.push(authorization);
    let mut execution = Card::new(
        "high-consequence",
        "ECHighConsequenceExecution",
        "Class9BoundedHighConsequenceAct",
    )
    .concludes("permits($proponent, ECExactHighConsequenceActivity, $record)")
    .fields(&[
        ("$project", "ProposedActivity"),
        ("$activity_version", "ActivityVersion"),
        ("$proponent", "ActivityProponent"),
        ("$authorization_record", "AuthorizationRecord"),
        ("$authorization_version", "AuthorizationVersion"),
        (
            "ECExecutionWindowWithinExactAuthorizationScopeAndInterval",
            "AuthorizationContainment",
        ),
        (
            "ECNoGenericPermissionBeyondThisActivityAndCurrentEvaluation",
            "ExecutionLimit",
        ),
    ])
    .depends(&["high-consequence-basis"])
    .join("high-consequence-basis", "$record", "$authorization_record")
    .join(
        "high-consequence-basis",
        "$revision",
        "$authorization_version",
    )
    .extra("~oppose($authorization_record, $authorization_version, ECGuardianAutomaticStay)")
    .extra("~oppose($authorization_record, $authorization_version, ECJudicialInterimStay)")
    .extra("~oppose($authorization_record, $authorization_version, ECFinalJudicialProtection)")
    .duty("ECWithholdIrreversibleExecutionWhileExactCurrentStayApplies");
    for (axis, key) in AXES.iter().zip([
        "climate",
        "water",
        "air",
        "soil",
        "biodiversity",
        "habitat",
        "extraction",
        "waste",
    ]) {
        execution = execution.extra(&format!(
            "~oppose($high_consequence_basis_{key}_condition, $project, {axis})"
        ));
    }
    cards.push(execution);
    cards.push(
        Card::new(
            "reversible-continuity",
            "ECReversibleEssentialContinuity",
            "Class6BoundedReversibleContinuity",
        )
        .concludes("permits($operator, ECOnlyReviewedReversibleEssentialContinuity, $record)")
        .fields(&[
            ("$project", "ProposedActivity"),
            ("$authorization_record", "AuthorizationRecord"),
            ("$authorization_version", "AuthorizationVersion"),
            ("$operation", "ReversibleContinuityOperation"),
            (
                "ECIndependentFindingWhollyReversibleEssentialContinuityOnly",
                "ReversibilityFinding",
            ),
            (
                "ECNoIrreversiblePartCeilingBreachOrAnimalCoreWaiver",
                "ContinuityHardLimits",
            ),
            (
                "ECNecessaryLeastHarmFloorAndAnimalCareContinuity",
                "ContinuityNecessity",
            ),
            (
                "ECIndependentPromptReviewOwnEndAndEffectiveRemedy",
                "ContinuityReview",
            ),
            (
                "ECContinuityFindingCreatesNoEntryInspectionRequisitionOrHumanCoercionPower",
                "ContinuityAuthorityLimit",
            ),
        ])
        .duty("ECPreserveOnlyReviewedReversibleEssentialContinuityDuringStay"),
    );
    cards
}
