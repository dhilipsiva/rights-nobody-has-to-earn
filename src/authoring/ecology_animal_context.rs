// SPDX-License-Identifier: MIT OR Apache-2.0

//! Same direct core in different settings; grave risk is not animal guilt.

use super::{Card, animals};

pub(super) const OWNERSHIPS: &[&str] = &[
    "ECPublicControl",
    "ECCooperativeControl",
    "ECHouseholdControl",
    "ECCustomaryControl",
    "ECNonprofitControl",
    "ECPrivateControl",
];
pub(super) const HUMAN_HARMS: &[&str] = &[
    "ECHumanPollution",
    "ECHumanInfrastructure",
    "ECHumanEntanglement",
    "ECHumanHabitatDestruction",
    "ECHumanIntroducedHazard",
    "ECUnlawfulCaptureOrTrade",
    "ECUnlawfulKilling",
    "ECHumanCreatedAnimalDependency",
];
pub(super) const GRAVE_RISKS: &[&str] = &[
    "ECGraveAnimalDiseaseThreat",
    "ECGraveDangerousAnimalThreat",
    "ECGraveHumanIntroducedPopulationHarm",
];

pub(super) fn rules() -> Vec<String> {
    let mut rules = Vec::new();
    for (values, scope, category) in [
        (OWNERSHIPS, "AnimalOwnershipForm", "ECAnimalControllerForm"),
        (HUMAN_HARMS, "WildHumanHarmGround", "ECWildHumanHarm"),
        (GRAVE_RISKS, "AnimalGraveRiskGround", "ECAnimalGraveRisk"),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {category})."));
        }
    }
    rules
}

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, context) in [
        (
            "animal-domestic",
            "ECDomesticAnimalProtection",
            "ECDomesticOrCompanionAnimal",
        ),
        (
            "animal-farmed",
            "ECFarmedAnimalProtection",
            "ECFarmedAnimal",
        ),
        (
            "animal-captive",
            "ECCaptiveWildAnimalProtection",
            "ECCaptiveDisplayOrTradedWildAnimal",
        ),
    ] {
        let mut card = animals::controlled(
            Card::new(id, kind, "Class10EqualDirectCoreAcrossAnimalContexts")
                .depends(&["animal-welfare"])
                .fields(&[
                    (context, "AnimalLivingContext"),
                    ("$ownership", "AnimalOwnershipForm"),
                    (
                        "ECSameDirectCoreRegardlessOfOwnershipFoodLabourLicenceOrConservationLabel",
                        "AnimalContextNoExemption",
                    ),
                ])
                .extra("member($ownership, ECAnimalControllerForm)")
                .default("$ownership", "ECPrivateControl"),
        );
        if id == "animal-captive" {
            card = card.fields(&[("ECSpeciesAppropriateHabitatEnrichmentMovementSocialMigrationAndReleaseConsideration", "CaptiveAnimalConditions"),
                ("ECConservationLabelDoesNotProveCaptivityOrBreedingNecessary", "CaptiveNecessityBoundary")]);
        }
        cards.push(card.effect("obliged($controller, ECRespectDirectAnimalCoreAndContextSpecificCareWithoutOwnershipExemption, $record)"));
    }
    cards.push(animals::controlled(Card::new("animal-wild-human-harm", "ECWildHumanCausedAnimalHarm", "Class10HumanResponsibilityForWildAnimalHarm")
        .depends(&["animal-coverage"])
        .fields(&[
            ("$ground", "WildHumanHarmGround"),
            ("ECCredibleEvidenceOfHumanCreatedControlledOrCausedAnimalHarm", "WildHumanCausation"),
            ("ECPrioritizeHumanCausePreventionRescueAndRepairWithSeparateHabitatAndEcosystemEffects", "WildHumanResponse"),
            ("ECNoNaturalPredatorOffenderOrUniversalWildIndividualRegistry", "WildHumanBoundary"),
        ])
        .extra("member($ground, ECWildHumanHarm)").default("$ground", "ECHumanEntanglement"))
        .duty("ECPrioritizePreventionRescueAndRepairOfHumanCausedWildAnimalHarm")
        .effect("obliged($controller, ECPreventAndRemedyEvidencedHumanCausedAnimalHarm, $record)"));
    cards.push(Card::new("animal-natural-process", "ECOrdinaryNaturalProcessBoundary", "Class10NoAnimalOffenceFromNature")
        .fields(&[
            ("$natural_event", "OrdinaryNaturalAnimalEvent"),
            ("ECIndependentEvidenceOfOrdinaryNaturalPredationIllnessHungerOrDeathNotHumanCreatedHarm", "NaturalEventFinding"),
            ("ECNoAnimalOffenderCriminalConvictionInferiorityOrPunishment", "NaturalNoOffender"),
            ("ECNoUniversalDutyToSuppressEcologicalProcessesOrRescueEveryWildIndividual", "NaturalNoUniversalRescue"),
            ("ECBoundedEvidenceBasedLeastHarmInterventionRemainsSeparatelyReviewable", "NaturalInterventionBoundary"),
        ])
        .effect("prevents($subject, ECAnimalGuiltOrPunishmentFromOrdinaryNature)")
        .effect("prevents($record, ECUniversalWildRescueOrPredationSuppressionInference)"));
    let wild = animals::controlled(Card::new("animal-wild-intervention-test", "ECBoundedWildAnimalInterventionTest", "Class10BoundedWildSufferingResponse")
        .depends(&["animal-coverage"])
        .fields(&[
            ("$suffering", "WildGraveSufferingEvidence"),
            ("$ecological_effect", "WildInterventionEcologicalEffectEvidence"),
            ("ECReviewedEvidenceShowsReductionOfGraveSufferingWithoutDisproportionateEcologicalHarm", "WildInterventionFinding"),
            ("ECBoundedLeastHarmMethodCurrentQualifiedReviewCareAndReassessment", "WildInterventionMethod"),
            ("ECNoNaturalGuiltUniversalRescueDutyOrWaiverOfAnimalAndEcologicalHardBars", "WildInterventionLimit"),
        ]));
    cards.push(wild);
    let control = animals::controlled(Card::new("animal-grave-risk-test", "ECGraveAnimalRiskResponseTest", "Class10NonlethalFirstGraveAnimalRiskManagement")
        .depends(&["animal-coverage"])
        .fields(&[
            ("$ground", "AnimalGraveRiskGround"), ("$risk", "AuthenticatedContestableGraveAnimalRiskEvidence"),
            ("$nonlethal_options", "FeasibleNonlethalAnimalControlEvidence"),
            ("ECActualGraveThreatEstablishedNotSpeciesPopulationPredatorOrMoralStatusLabel", "AnimalGraveRiskFinding"),
            ("ECHumanCausesAndPreventionAddressedFirst", "AnimalRiskHumanCausePriority"),
            ("ECFeasiblePreventionExclusionTreatmentVaccinationRelocationFertilityAndHabitatMeasuresEvaluatedBeforeKilling", "AnimalRiskNonlethalPriority"),
            ("ECLeastPainfulReliableMethodWithQualifiedOversight", "AnimalRiskMethod"),
            ("ECPublicAndWorkerSafetyIndigenousCollectiveRightsAndEcosystemIntegrityProtected", "AnimalRiskConcurrentRights"),
            ("ECPublicReasonsChallengeReassessmentRepairAndFreshOwnSourceBoundT3", "AnimalRiskReassessment"),
            ("ECNoAnimalConvictionPunishmentInferiorityInvasivenessAsMoralStatusOrSocialUsefulnessScore", "AnimalRiskNoGuilt"),
        ])
        .extra("member($ground, ECAnimalGraveRisk)").default("$ground", "ECGraveAnimalDiseaseThreat"));
    cards.push(control);
    for (id, kind, test, high, effect) in [
        (
            "animal-wild-low-impact",
            "ECLowImpactWildAnimalIntervention",
            "animal-wild-intervention-test",
            false,
            "permits($controller, ECOnlyBoundedLowImpactWildAnimalIntervention, $record)",
        ),
        (
            "animal-wild-high-impact",
            "ECEnhancedWildAnimalIntervention",
            "animal-wild-intervention-test",
            true,
            "permits($controller, ECOnlyBoundedEnhancedWildAnimalIntervention, $record)",
        ),
        (
            "animal-risk-low-impact",
            "ECLowImpactAnimalRiskResponse",
            "animal-grave-risk-test",
            false,
            "permits($controller, ECOnlyLowImpactNonlethalAnimalRiskResponse, $record)",
        ),
        (
            "animal-risk-high-impact",
            "ECEnhancedNonlethalAnimalRiskResponse",
            "animal-grave-risk-test",
            true,
            "permits($controller, ECOnlyEnhancedNonlethalAnimalRiskResponse, $record)",
        ),
        (
            "animal-risk-lethal",
            "ECLastResortLethalAnimalRiskResponse",
            "animal-grave-risk-test",
            true,
            "permits($controller, ECOnlyLastResortLeastPainfulLethalAnimalRiskResponse, $record)",
        ),
    ] {
        let mut card =
            Card::new(id, kind, "Class10ExactBoundedAnimalIntervention").depends(&[test]);
        if high {
            card = card.depends(&["animal-enhanced-test"]);
        } else {
            card = card.fields(&[(
                "ECPositivelyNonlethalNonInvasiveAndNonHighSeverity",
                "LowImpactAnimalUseFinding",
            )]);
        }
        if id == "animal-risk-lethal" {
            card = card
                .join("animal-enhanced-test", "$impact", "ECAnimalLethalUse")
                .fields(&[(
                    "ECFeasibleNonlethalRoutesUnavailableIneffectiveOrWouldCauseGreaterGraveHarm",
                    "AnimalLethalLastResortFinding",
                )]);
        } else if id == "animal-risk-high-impact" {
            card = card
                .fields(&[("$impact", "EnhancedAnimalImpact")])
                .extra("~($impact = ECAnimalLethalUse)");
        }
        cards.push(animals::permission(card, effect));
    }
    cards
}
