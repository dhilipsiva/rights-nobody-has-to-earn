// SPDX-License-Identifier: MIT OR Apache-2.0

//! Separate ordinary, enhanced, all-food and all-research admission tests.

use super::{Card, animals, records};

pub(super) const IMPACTS: &[&str] = &[
    "ECAnimalLethalUse",
    "ECAnimalInvasiveUse",
    "ECAnimalHighSeverityUse",
];
pub(super) const PURPOSES: &[&str] = &[
    "ECImmediateNecessaryNourishment",
    "ECSeriousHumanOrAnimalIllnessPreventionOrTreatment",
    "ECGraveSafetyOrPublicHealthProtection",
    "ECEvidenceBasedConservationOrRestoration",
    "ECWelfareCompatibleAssistanceOrWork",
    "ECQualifyingScientificResearch",
];
pub(super) const RESEARCH_PURPOSES: &[&str] = &[
    "ECReviewedHumanOrAnimalHealthAdvance",
    "ECReviewedSafetyAdvance",
    "ECReviewedEcologicalProtectionAdvance",
];

pub(super) fn rules() -> Vec<String> {
    let mut rules = Vec::new();
    for (values, scope, category) in [
        (IMPACTS, "EnhancedAnimalImpact", "ECEnhancedAnimalImpact"),
        (PURPOSES, "EnhancedAnimalPurpose", "ECSeriousAnimalPurpose"),
        (
            RESEARCH_PURPOSES,
            "AnimalResearchPurpose",
            "ECSeriousAnimalResearchPurpose",
        ),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {category})."));
        }
    }
    rules
}

pub(super) fn cards() -> Vec<Card> {
    let enhanced = animals::controlled(Card::new("animal-enhanced-test", "ECEnhancedAnimalUseTest", "Class10EnhancedUseConditionsNotGeneralPermission")
        .depends(&["animal-welfare"])
        .fields(&[
            ("$impact", "EnhancedAnimalImpact"), ("$purpose", "EnhancedAnimalPurpose"),
            ("$necessity", "EnhancedAnimalNecessityEvidence"), ("$alternatives", "EnhancedAnimalAlternativesEvidence"),
            ("$prior_review", "EnhancedAnimalIndependentPriorReview"),
            ("ECSeriousCompatiblePurposeEstablishedNotALabel", "EnhancedAnimalPurposeFinding"),
            ("ECCredibleEvidenceEstablishesNecessityForThisExactPurpose", "EnhancedAnimalNecessityFinding"),
            ("ECNoSafeAccessibleEffectiveMateriallyLessHarmfulAlternativeReasonablyAvailable", "EnhancedAnimalAlternativesFinding"),
            ("ECNumberDurationPainDistressConfinementInjuryAndRiskAvoidedOrMinimizedByLeastHarmMethod", "EnhancedAnimalLeastHarm"),
            ("ECSpeciesAppropriateCareMonitoringHumaneEndpointsAftercareAndContinuitySecured", "EnhancedAnimalCare"),
            ("ECIndependentReviewPrecedesHighConsequenceUseWithReasonsChallengeReassessmentRemedyAndOwnEnd", "EnhancedAnimalPriorReviewFinding"),
            ("ECThisTestDoesNotAuthorizeFoodResearchOrAnyActivityByItself", "EnhancedTestBoundary"),
        ])
        .extra("member($impact, ECEnhancedAnimalImpact)")
        .extra("member($purpose, ECSeriousAnimalPurpose)")
        .default("$impact", "ECAnimalInvasiveUse")
        .default("$purpose", "ECSeriousHumanOrAnimalIllnessPreventionOrTreatment"))
        .duty("ECRequireAllEnhancedConditionsBeforeAnyLethalInvasiveOrHighSeverityUse");
    let general = animals::permission(
        Card::new(
            "animal-enhanced-nonfood-use",
            "ECEnhancedNonFoodNonResearchUse",
            "Class10ExactEnhancedNonFoodUse",
        )
        .depends(&["animal-enhanced-test"])
        .fields(&[
            (
                "ECPositivelyNonFoodNonResearchNonTestingNonEducationUse",
                "EnhancedNonFoodDomain",
            ),
            ("$purpose", "EnhancedAnimalPurpose"),
        ])
        .extra("~($purpose = ECImmediateNecessaryNourishment)")
        .extra("~($purpose = ECQualifyingScientificResearch)"),
        "permits($controller, ECOnlyEnhancedNonFoodNonResearchAnimalUse, $record)",
    );
    let euthanasia = animals::permission(
        Card::new(
            "animal-euthanasia",
            "ECQualifiedAnimalEuthanasia",
            "Class10SeriousPurposeLeastDistressingDeath",
        )
        .depends(&["animal-enhanced-test"])
        .join("animal-enhanced-test", "$impact", "ECAnimalLethalUse")
        .fields(&[
            ("$qualified_finding", "EuthanasiaQualifiedCurrentEvidence"),
            (
                "ECOtherwiseUnrelievableSufferingOrAnotherSeriousPurposePassingFullEnhancedTest",
                "EuthanasiaPurpose",
            ),
            (
                "ECLeastDistressingReliableMethodWithQualifiedCareAndContinuity",
                "EuthanasiaMethod",
            ),
            (
                "ECNotOwnerInconvenienceShelterDelayOrdinaryCostOrReducedCommercialValueAlone",
                "EuthanasiaInsufficientGrounds",
            ),
            (
                "ECNotFoodProductionOrResearchProcedureAuthorization",
                "EuthanasiaDomainBoundary",
            ),
        ]),
        "permits($controller, ECOnlyQualifiedLeastDistressingEuthanasia, $record)",
    );
    let food = animals::controlled(Card::new("animal-food-test", "ECStrictAllAnimalFoodTest", "Class10EveryControlledFoodProductionTest")
        .depends(&["animal-welfare"])
        .fields(&[
            ("$affected_people", "AffectedFoodRecipients"),
            ("$nutrition_alternatives", "ActualNutritionAndAccessibilityEvidence"),
            ("ECNoSafeAccessibleNutritionallyAdequateMateriallyLessHarmfulAlternativeReasonablyAvailable", "StrictAnimalFoodAlternativeFinding"),
            ("ECTasteHabitPrestigeProfitAdvertisingOrPriceAloneInsufficient", "FoodInsufficientGrounds"),
            ("ECCostAndGeographyOnlyBearOnRealAccessibilityWithoutWithholdingHumanFoodFloor", "FoodAccessibilityBoundary"),
            ("ECBreedingHousingHandlingTransportAndKillingRemainSubjectToCareStandardsIndependentInspectionAndRemedy", "FoodWelfareConditions"),
            ("ECEffectiveStunningOrIndependentlyEstablishedLeastSufferingMethodWhereDeathOccurs", "FoodDeathMethod"),
            ("ECAllControlledFoodProductionIncludingNonlethalAndNonSevereUses", "AllFoodScope"),
            ("ECFoodPurposeNeverAuthorizesAvoidableSuffering", "FoodCoreBoundary"),
        ]))
        .duty("ECApplyStrictAccessibleNutritionAlternativeTestToEveryControlledAnimalFoodUse");
    let mut research = animals::controlled(Card::new("animal-research-test", "ECStrictAllAnimalResearchTest", "Class10EveryResearchTestingAndEducationTest")
        .depends(&["animal-welfare"])
        .fields(&[
            ("$research_purpose", "AnimalResearchPurpose"),
            ("$investigator", "AnimalResearchInvestigator"), ("$funder", "AnimalResearchFunder"),
            ("$facility", "AnimalResearchFacility"), ("$scientific_reviewer", "AnimalResearchScientificReviewer"),
            ("$ethical_reviewer", "AnimalResearchEthicalReviewer"),
            ("$replacement_evidence", "SystematicAnimalReplacementEvidence"),
            ("$registration", "ProspectiveAnimalResearchRegistration"),
            ("ECReviewedEvidenceSupportsSeriousHealthSafetyOrEcologicalAdvanceNotPrestigeCuriosityOrMarketing", "AnimalResearchPurposeFinding"),
            ("ECReplacementFirstThenReductionAndRefinement", "AnimalResearchReplacementOrder"),
            ("ECNoScientificallyValidNonAnimalOrMateriallyLessHarmfulMethodCanAchievePurpose", "AnimalResearchAlternativesFinding"),
            ("ECMinimumAnimalsAndLeastHarmfulSpeciesAppropriateValidDesign", "AnimalResearchDesign"),
            ("ECPainPreventionReliefHumaneEndpointsCareAftercareRehabilitationOrRehoming", "AnimalResearchCare"),
            ("ECIndependentProjectScientificAndEthicalReviewBeforeUse", "AnimalResearchPriorReview"),
            ("ECPublicResultAndWelfareReportingIncludesNegativeAndInconclusiveResultsWithNarrowSafetyPrivacyLimits", "AnimalResearchAllResults"),
            ("ECInspectionChallengeCorrectionSuspensionRemedyAndNonRepetition", "AnimalResearchAccountability"),
            ("ECNoUnrelievedSevereOrProlongedProcedure", "AnimalResearchHardLimit"),
            ("ECAllResearchTestingAndEducationIncludingNonlethalAndNonSevereUse", "AllResearchScope"),
        ])
        .extra("member($research_purpose, ECSeriousAnimalResearchPurpose)")
        .default("$research_purpose", "ECReviewedHumanOrAnimalHealthAdvance"));
    for (actor, role) in [
        ("$scientific_reviewer", "ECAnimalScientificReviewAuthority"),
        ("$ethical_reviewer", "ECAnimalEthicalReviewAuthority"),
    ] {
        research = research
            .extra(&format!("authorized({actor}, {role}, $record)"))
            .extra(&records::observe(
                actor,
                "$record",
                "ECIndependentProjectReviewApproved",
                "AnimalResearchIndependentReview",
            ));
        for other in [
            "$investigator",
            "$funder",
            "$facility",
            "$review",
            "$source",
            "$operator",
            "$controller",
        ] {
            research = research.extra(&format!("~({actor} = {other})"));
        }
    }
    research = research
        .extra("~($scientific_reviewer = $ethical_reviewer)")
        .duty("ECPursueFullReplacementAsSoonAsScientificallyValidAlternativesPerformThePurpose");
    let mut cards = vec![enhanced, general, euthanasia, food, research];
    for (id, kind, test, high, effect) in [
        (
            "animal-food-low-impact",
            "ECStrictLowImpactAnimalFoodUse",
            "animal-food-test",
            false,
            "permits($controller, ECOnlyStrictLowImpactAnimalFoodProduction, $record)",
        ),
        (
            "animal-food-high-impact",
            "ECStrictEnhancedAnimalFoodUse",
            "animal-food-test",
            true,
            "permits($controller, ECOnlyStrictEnhancedAnimalFoodProduction, $record)",
        ),
        (
            "animal-research-low-impact",
            "ECStrictLowImpactAnimalResearchUse",
            "animal-research-test",
            false,
            "permits($controller, ECOnlyStrictLowImpactAnimalResearchTestingOrEducation, $record)",
        ),
        (
            "animal-research-high-impact",
            "ECStrictEnhancedAnimalResearchUse",
            "animal-research-test",
            true,
            "permits($controller, ECOnlyStrictEnhancedAnimalResearchTestingOrEducation, $record)",
        ),
    ] {
        let mut card = Card::new(id, kind, "Class10ExactDomainSpecificAnimalUse").depends(&[test]);
        if high {
            card = card.depends(&["animal-enhanced-test"]).join(
                "animal-enhanced-test",
                "$purpose",
                if test == "animal-food-test" {
                    "ECImmediateNecessaryNourishment"
                } else {
                    "ECQualifyingScientificResearch"
                },
            );
        } else {
            card = card.fields(&[(
                "ECPositivelyNonlethalNonInvasiveAndNonHighSeverity",
                "LowImpactAnimalUseFinding",
            )]);
        }
        cards.push(animals::permission(card, effect));
    }
    cards.push(Card::new("animal-food-continuity", "ECAnimalFoodFloorAndCareContinuity", "Class6LessHarmNutritionAndProtectedFoodContinuity")
        .fields(&[
            ("$affected_people", "AffectedFoodRecipients"),
            ("$supply_failure", "ActualNutritionAccessibilityOrTransitionFailure"),
            ("ECSafeAdequateAccessibleLessHarmNutritionMustBeMadeAvailable", "LessHarmNutritionDuty"),
            ("ECNoHumanFoodLossWhileSupplyWorkersCommunitiesOrLocalCapacityAreNotReady", "FoodFloorContinuity"),
            ("ECSourceBoundSupportWithWorkerIndigenousSubsistenceProducerAndRegionalParticipationEqualityAndDueProcess", "FoodTransitionProtection"),
            ("ECDelayAndUnderinvestmentRemainPublicFailureNotPermanentAvoidableHarmPermission", "FoodDelayFailure"),
            ("ECNoSupplyArrivalOperationalFeasibilityOrTransitionSuccessInferred", "FoodNoOperationInference"),
        ])
        .duty("ECProvideAccessibleLessHarmNutritionAndProtectHumanFoodAndAnimalCareContinuity")
        .effect("obliged($alternate, ECRespondToNutritionAndTransitionFailureWithoutPermanentHarmPermission, $record)"));
    cards.push(animals::controlled(Card::new("animal-work-continuity", "ECWorkingAnimalCareAndExit", "Class10WorkingAndAssistanceAnimalContinuity")
        .depends(&["animal-welfare"])
        .fields(&[
            ("ECSpeciesAndIndividualWorkloadLimitsRestNecessaryCareAndSafeConditions", "WorkingAnimalConditions"),
            ("ECExitFromHarmThroughResponsiblePublicRouteAndRetirementOrPlacementContinuity", "WorkingAnimalExit"),
            ("ECUsefulnessAffectionProfitOrAssistanceLabelDoesNotEstablishWelfareOrOverrideEnhancedUseTest", "WorkingAnimalBoundary"),
        ]))
        .effect("obliged($controller, ECObserveWorkLimitsCareAndProtectedExitRetirementOrPlacement, $record)"));
    cards
}
