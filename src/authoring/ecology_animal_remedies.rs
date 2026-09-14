// SPDX-License-Identifier: MIT OR Apache-2.0

//! Independent initiation and particular judicial orders, never animal guilt.

use super::Card;

pub(super) const REMEDIES: &[(&str, &str, &str, &str)] = &[
    (
        "animal-licence-order",
        "ECAnimalActivityLicenceOrder",
        "ECOnlyReviewSuspendOrRevokeExactAnimalActivityLicence",
        "ECActivityLicenceOnlyNoGeneralCivicOrEconomicDisqualification",
    ),
    (
        "animal-rescue-order",
        "ECAnimalRescueOrder",
        "ECProvideExactNecessaryAnimalRescue",
        "ECRescueAndNecessaryCareNotConditionedOnOwnerConsentOrPapers",
    ),
    (
        "animal-custody-order",
        "ECAnimalCustodyOrder",
        "ECArrangeExactRightsBoundedAnimalCustody",
        "ECAnimalCustodyByProtectedInterestsHumanDueProcessAndCareContinuityNotHumanDetention",
    ),
    (
        "animal-rehoming-order",
        "ECAnimalRehomingOrder",
        "ECArrangeExactSuitableAnimalRehoming",
        "ECSpeciesIndividualNeedsCareSuitabilityAndContinuityNotMarketPrice",
    ),
    (
        "animal-cessation-order",
        "ECAnimalCessationOrder",
        "ECStopExactUnlawfulAnimalActivity",
        "ECExactActivityAndScopeNoGeneralUnrelatedOccupationBan",
    ),
    (
        "animal-veterinary-order",
        "ECAnimalVeterinaryOrder",
        "ECProvideExactQualifiedVeterinaryTreatment",
        "ECQualifiedCurrentEvidenceHumaneTreatmentPainReliefAndContinuity",
    ),
    (
        "animal-rehabilitation-order",
        "ECAnimalRehabilitationOrder",
        "ECProvideExactAnimalRehabilitation",
        "ECSpeciesIndividualRecoveryNeedsQualifiedCareAndPlacementContinuity",
    ),
    (
        "animal-sanctuary-order",
        "ECAnimalSanctuaryOrder",
        "ECProvideExactSuitableAnimalSanctuary",
        "ECSpeciesAppropriateLongTermCareAndIndependentReviewNotIndefiniteWarehousing",
    ),
    (
        "animal-use-disqualification-order",
        "ECAnimalUseDisqualificationOrder",
        "ECApplyExactReviewableAnimalUseDisqualification",
        "ECIndividualizedAnimalUseRestrictionNotHumanStandingFloorVoteOrUnrelatedPrivateLifeLoss",
    ),
    (
        "animal-record-correction-order",
        "ECAnimalRecordCorrectionOrder",
        "ECCorrectExactAnimalRecordAndAffectedReliance",
        "ECReviewableCorrectionPreservesLawfulHistoryPrivacyAndCareWithoutSilentRenewal",
    ),
    (
        "animal-care-restitution-order",
        "ECAnimalCareRestitutionOrder",
        "ECPayExactAdjudicatedNecessaryAnimalCareCosts",
        "ECIndividuallyEstablishedCostsAndResponsibilityNoPaymentExtinguishesCareOrRescue",
    ),
    (
        "animal-structural-order",
        "ECAnimalStructuralRemedyOrder",
        "ECImplementExactAnimalProtectionStructuralRemedy",
        "ECSpecifiedSystemicFailureAndProportionateReviewableReformNoCourtProgrammeTakeover",
    ),
    (
        "animal-monitoring-order",
        "ECAnimalMonitoringOrder",
        "ECProvideExactPurposeLimitedAnimalProtectionMonitoring",
        "ECNecessaryScopedMonitoringIndependentReviewPrivacyAndOwnEndNoHumanScore",
    ),
    (
        "animal-habitat-order",
        "ECAnimalHabitatRestorationOrder",
        "ECRestoreExactAnimalHabitatConditions",
        "ECProtectedAnimalNeedsAndSeparateEcologicalStandardsNoOffsetForIrreplaceableLoss",
    ),
    (
        "animal-nonrepetition-order",
        "ECAnimalNonRepetitionOrder",
        "ECImplementExactAnimalHarmNonRepetitionMeasures",
        "ECEvidencedCausePreventionAndIndependentFollowUpNoSuccessfulNonRepetitionInference",
    ),
];

pub(super) fn rules() -> Vec<String> {
    vec![
        "all $initiator: all $subject: all $request: challenge($initiator, $subject, $request) & observe($initiator, $request, ECClaimPlausibleAnimalInterest, ECRequestKindScope) -> obliged(State, ECReceiveAnimalProtectionClaimAndProvideIndependentHelpWithoutOwnerVeto, $request).".into(),
    ]
}

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, qualification, dependency, office) in [
        (
            "animal-human-claim",
            "ECAffectedHumanAnimalProtectionClaim",
            "ECAffectedHumanIndependentInitiation",
            None,
            None,
        ),
        (
            "animal-supporter-claim",
            "ECChosenSupporterAnimalProtectionClaim",
            "ECActualChosenSupporterMandateAndAffectedHumanVoice",
            None,
            None,
        ),
        (
            "animal-association-claim",
            "ECAssociationAnimalProtectionClaim",
            "ECQualifiedAssociationAndCurrentRepresentativeMandate",
            None,
            None,
        ),
        (
            "animal-professional-claim",
            "ECProfessionalAnimalProtectionClaim",
            "ECCurrentVeterinarianOrAuthorizedProfessionalQualification",
            None,
            None,
        ),
        (
            "animal-regulator-claim",
            "ECRegulatorAnimalProtectionClaim",
            "ECIndependentPublicAnimalRegulatorMandate",
            None,
            Some("FSBOD_29"),
        ),
        (
            "animal-advocate-claim",
            "ECAnimalAdvocateProtectionClaim",
            "ECCurrentIndependentCollegialAnimalAdvocateDecision",
            Some("animal-advocate-result"),
            Some("FSBOD_23"),
        ),
        (
            "alternate-animal-claim",
            "ECAlternateAnimalAdvocateProtectionClaim",
            "ECCurrentPredeclaredIndependentAnimalAlternateFunction",
            Some("animal-alternate-function"),
            Some("FSBOD_33"),
        ),
    ] {
        let mut card = Card::new(id, kind, "Class4IndependentAnimalProtectionInitiation")
            .fields(&[
                ("$initiator", "AnimalProtectionInitiator"), ("$claim", "AnimalProtectionClaim"),
                (qualification, "AnimalInitiatorQualification"),
                ("ECAnimalDirectInterestAndAffectedHumanCollectiveRightsRemainDistinct", "AnimalRepresentationBoundary"),
                ("ECNoExclusiveOwnerStandingWaiverSettlementOfAnimalInterestsOrOwnerVeto", "AnimalClaimNoOwnerControl"),
                ("ECLawfulInformationInspectionEvidenceReviewInterimRescueCessationAndRemedyRequests", "AnimalClaimRoutes"),
                ("ECRequestNotAutomaticStayFinalMeritsPermitBudgetProgrammeCustodyOrProsecution", "AnimalClaimPowerBoundary"),
            ])
            .extra("~($initiator = $subject)")
            .effect("obliged($reader, ECIndependentlyReceiveAnimalProtectionClaimAndAvailableInterimRemedyRequest, $record)");
        if let Some(dependency) = dependency {
            card = card.depends(&[dependency]);
        }
        if let Some(office) = office {
            card = card
                .extra(&format!("$initiator = {office}"))
                .extra(&format!("$operator = {office}"))
                .default("$initiator", office)
                .default("$operator", office);
        } else if id != "animal-association-claim" {
            card = card.fields(&[(
                "ECIndependentlyIdentifiedPresentHumanInitiatorNotAnimalPersonhoodOrRegistryQualification",
                "AnimalClaimHumanInitiatorFinding",
            )]);
        }
        cards.push(card);
    }
    cards.push(
        Card::new(
            "animal-inspection",
            "ECIndependentAnimalInspection",
            "Class6BoundedAnimalWelfareInspection",
        )
        .fields(&[
            ("$inspector", "IndependentQualifiedAnimalInspector"),
            ("$controller", "InspectedAnimalController"),
            ("$premises", "ExactAnimalInspectionPremises"),
            ("$access_basis", "ActualLawfulAnimalInspectionAccessBasis"),
            (
                "ECIndependentExpertiseAndEvidenceBasedNecessaryProportionateInspection",
                "AnimalInspectionNecessity",
            ),
            (
                "ECLawfulBoundedAccessNoAutomaticPrivateEntrySearchSeizureOrHumanCoercionPower",
                "AnimalInspectionAccessBoundary",
            ),
            (
                "ECPurposeLimitedFindingsPrivacyReasonsChallengeCorrectionAndSourceBoundEnd",
                "AnimalInspectionAccountability",
            ),
            (
                "ECNoSelfCertificationProsecutionCustodyConvictionOrPerformedInspectionInference",
                "AnimalInspectionPowerBoundary",
            ),
        ])
        .extra("$operator = FSBOD_30")
        .default("$operator", "FSBOD_30")
        .extra("~($inspector = $controller)")
        .extra("~($source = $inspector)")
        .extra("~($review = $inspector)")
        .extra("~($review = $controller)")
        .extra("~($source = $controller)")
        .concludes("permits($inspector, ECOnlyExactLawfullyAccessibleAnimalInspection, $record)"),
    );
    cards.push(Card::new("animal-guardian-conflict", "ECIndependentGuardianAnimalConflictDecision", "Class4IndependentNonAggregatedProtectionDecision")
        .depends(&["guardian-result", "animal-advocate-result"])
        .fields(&[
            ("$guardian_position", "ActualGuardianEcologicalPosition"), ("$animal_position", "ActualAnimalAdvocatePosition"),
            ("$human_right", "ConflictHumanEnvironmentalRightAxis"), ("$human_floor", "ConflictHumanFloorAxis"),
            ("$commons", "ConflictClass9ConditionAxis"), ("$animal_interests", "ConflictClass10InterestsAxis"),
            ("$collective", "ConflictIndigenousCollectiveRightsAxis"), ("$alternatives", "ConflictAlternativesAxis"),
            ("$uncertainty", "ConflictUncertaintyAxis"), ("$reversibility", "ConflictReversibilityAxis"),
            ("$continuity", "ConflictContinuityAxis"), ("$decision", "ExactIndependentConflictDisposition"),
            ("ECApplyEveryHardProhibitionFirstThenGivePublicReasonsForLeastHarmCompatibleRoute", "ConflictDecisionRule"),
            ("ECNeitherOfficeHasPriorityAndNoUtilityWorthOrCompensationScalarReplacesAnAxis", "ConflictNoOfficeOrScorePriority"),
            ("ECSelectedRouteLabelDoesNotAuthorizeAnActWithoutItsSeparateAnimalEcologicalAndHumanRightsGates", "ConflictNoAuthorizationShortcut"),
        ])
        .extra("$operator = FSBOD_17").default("$operator", "FSBOD_17")
        .duty("ECPublishAndApplyIndependentHardBarFirstAxisSeparatedLeastHarmConflictDecision"));
    for (id, kind, action, boundary) in REMEDIES {
        let mut card = Card::new(id, kind, "Class4ExactAnimalRemedyOrder")
            .fields(&[
                ("$executor", "AnimalRemedyExecutor"), ("$controller", "AnimalRemedyAffectedController"),
                ("$order", "ExactAnimalRemedyJudicialOrder"), ("$direction", "ExactAnimalRemedyDirection"),
                (action, "AnimalRemedyAction"), (boundary, "AnimalRemedySpecificBoundary"),
                ("ECIndependentAdjudicatedAnimalInterestInjuryOrUrgentPlausibleProtectionBasis", "AnimalRemedyBasis"),
                ("ECNecessaryProportionateParticularOrderWithHumanRightsNoticeReviewAndAppeal", "AnimalRemedyProcedure"),
                ("ECAnimalCareAndAffectedHumanStandingFloorLibertyEqualVoiceAndCollectiveRightsPreserved", "AnimalRemedyContinuity"),
                ("ECPaymentDoesNotDischargeRescueCareRestorationOrNonRepetition", "AnimalRemedyNonSubstitution"),
                ("ECNoHumanPunishmentSearchEntrySeizureOrCustodyAuthorityWithoutSeparateLawfulJusticeGate", "AnimalRemedyHumanCoercionBoundary"),
                ("ECNoPerformedRescueTreatmentTransferRestorationOrDeliveryInferred", "AnimalRemedyNoExecutionInference"),
            ])
            .extra("$operator = FSBOD_17").default("$operator", "FSBOD_17")
            .extra("$executor = FSBOD_34").default("$executor", "FSBOD_34")
            .extra("~($operator = $executor)").extra("~($review = $executor)")
            .extra("~($source = $executor)").extra("~($review = $controller)").extra("~($source = $controller)")
            .effect(&format!("obliged($executor, {action}, $record)"));
        // All source-bound direction identities are also read by the actual
        // court's witnesses in the external connector.
        if *id == "animal-care-restitution-order" {
            card = card.fields(&[("$costs", "AdjudicatedNecessaryAnimalCareCosts"),
                ("ECSeparatelyEstablishedIndividualResponsibilityNotOwnerWorkerFamilyOrInvestorStatusAlone", "AnimalCareCostLiability")]);
        }
        cards.push(card);
    }
    cards.push(Card::new("animal-human-justice", "ECSeparateHumanJusticeForAnimalViolation", "Class4SeparateHumanCulpabilityAndFullJustice")
        .fields(&[
            ("$affected_human", "AffectedHumanJusticeSubject"), ("$violation", "ExactSeriousAnimalViolation"),
            ("$culpability", "SeparatelyAdjudicatedHumanCulpabilityEvidence"),
            ("ECIndividuallyEstablishedIntentionalOrRecklessSeriousViolation", "AnimalViolationCulpabilityFinding"),
            ("ECFullApplicableProofDefenceIndependentHearingAndAppealNoAggregateBurdenReversal", "AnimalViolationHumanProcedure"),
            ("ECSeparateAlreadyLawfulCoerciveMandateRequiredNoNewPowerFromAnimalFinding", "AnimalViolationMandateBoundary"),
            ("ECNoAnimalPunishmentAndNoHumanStandingFloorCoreLibertyDueProcessVoteOrCollectiveRightLoss", "AnimalViolationRightsWall"),
        ])
        .extra("~($subject = $affected_human)")
        .extra("$operator = FSBOD_17").default("$operator", "FSBOD_17")
        .duty("ECUseOnlySeparatelyLawfulHumanJusticeAfterIndividualSeriousCulpabilityAndFullProcedure"));
    cards
}
