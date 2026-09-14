// SPDX-License-Identifier: MIT OR Apache-2.0

//! Direct animal interests, not human personhood or a welfare certificate.

use super::{Card, records};

pub(super) const MINIMUM_TAXA: &[&str] =
    &["ECVertebrate", "ECCephalopodMollusc", "ECDecapodCrustacean"];

pub(super) const CATEGORICAL_BARS: &[&str] = &[
    "ECAnimalFightingOrBaiting",
    "ECAnimalSexualUse",
    "ECDeliberateAnimalCruelty",
    "ECPunitiveAnimalTreatment",
    "ECDispensableKillingOrSevereSufferingForSpectacle",
    "ECDispensableKillingOrSevereSufferingForGambling",
    "ECDispensableKillingOrSevereSufferingForAmusement",
    "ECDispensableKillingOrSevereSufferingForSportOrTrophy",
    "ECDispensableKillingOrSevereSufferingForPrestige",
    "ECDispensableKillingOrSevereSufferingForFashion",
    "ECDispensableKillingOrSevereSufferingForCosmeticsOrMarketing",
    "ECDispensableKillingOrSevereSufferingForNovelty",
    "ECDispensableKillingOrSevereSufferingForConvenience",
    "ECDispensableKillingOrSevereSufferingForProfit",
    "ECAbandonmentOrDenialOfNecessaryAssumedCare",
    "ECExtremePredictablyHarmfulConfinement",
    "ECPredictablySeriouslyHarmfulBreedingOrGeneticSelection",
    "ECPainfulNonTherapeuticMutilationWithoutFullSeriousPurposeNecessityPainControlAndLeastHarm",
    "ECUnrelievedSevereOrProlongedResearchProcedure",
];

pub(super) fn rules() -> Vec<String> {
    let mut rules = vec![
        "derived_only(\"class\").".into(),
        "derived_only(\"narrow\").".into(),
    ];
    for (values, scope, category) in [
        (MINIMUM_TAXA, "AnimalTaxon", "ECMinimumProtectedAnimalTaxon"),
        (
            CATEGORICAL_BARS,
            "AnimalCategoricalGround",
            "ECCategoricalAnimalBar",
        ),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {category})."));
        }
    }
    rules
}

pub(super) fn controlled(mut card: Card) -> Card {
    card = card
        .fields(&[
            ("$controller", "AnimalController"),
            ("$activity", "AnimalActivity"),
            (
                "ECExactAnimalInterestNotOwnerPreferenceOrMarketValue",
                "DirectInterestBoundary",
            ),
        ])
        .extra("~($subject = $controller)");
    for (actor, _) in records::ATTESTERS.iter().chain(records::READERS) {
        card = card.extra(&format!("~({actor} = $controller)"));
    }
    card
}

pub(super) fn permission(card: Card, effect: &'static str) -> Card {
    controlled(card)
        .extra("~oppose($subject, $activity, ECAnimalUse)")
        .concludes(effect)
        .fields(&[
            ("ECNoOwnershipSectorLicenceContractTraditionReligionOrCultureExemption", "NoAnimalCoreExemption"),
            ("$welfare_arrangement", "CurrentSpeciesAppropriateWelfareArrangementEvidence"),
            ("ECIndependentReviewEstablishesCareHandlingContinuityAndWelfareConditionsSecuredForThisUse", "AnimalUseWelfareFinding"),
        ])
        .extra("$operator = FSBOD_29")
        .extra("~($operator = $controller)")
        .default("$operator", "FSBOD_29")
}

pub(super) fn cards() -> Vec<Card> {
    vec![
        Card::new("animal-taxon-extension", "ECAnimalTaxonExtension", "Class10EvidenceResponsiveProtectedTaxon")
            .concludes("complete($record, ECAnimalTaxonExtension, $taxon)")
            .fields(&[
                ("$taxon", "AnimalTaxon"),
                ("ECIndependentPublicEvidenceEstablishesRealisticPossibilityOfSentience", "SentiencePossibilityFinding"),
                ("ECConclusiveProofNotRequiredAndUncertaintyRemainsVisible", "SentienceThreshold"),
                ("ECPublicIndependentEvidenceResponsiveReviewAndTaxonSpecificReasons", "TaxonProcedure"),
            ])
            .extra("$operator = FSBOD_28").default("$operator", "FSBOD_28")
            .duty("ECExtendThePresumptionOnCredibleRealisticPossibilityNotCertainty"),
        Card::new("animal-taxon-removal", "ECAnimalTaxonPresumptionRemoval", "Class10RigorousNonRegressiveTaxonRevision")
            .concludes("narrow($taxon, $version, ECAnimalTaxonPresumption)")
            .fields(&[
                ("$taxon", "AnimalTaxon"),
                ("$prior_finding", "PriorExactTaxonPresumptionRecord"),
                ("$prior_evidence", "PriorTaxonPresumptionEvidence"),
                ("ECIndependentComparisonOfPriorPresumptionAndCurrentTaxonSpecificEvidence", "TaxonRemovalComparison"),
                ("ECComparablyRigorousIndependentPublicEvidenceNotResidualUncertainty", "TaxonRemovalEvidence"),
                ("ECTransparentContestableNonRegressionReviewProtectsDirectCoreAndUrgentCare", "TaxonRemovalNonRegression"),
                ("ECRemovalDoesNotEraseLawfulHistoryOrProveAbsenceOfSuffering", "TaxonRemovalBoundary"),
            ])
            .extra("$operator = FSBOD_28").default("$operator", "FSBOD_28")
            .duty("ECApplyOnlyTheIndependentlyReviewedNonRegressiveTaxonRevision"),
        Card::new("animal-coverage", "ECCrediblySentientAnimalCoverage", "Class10DirectProtectedAnimalSubject")
            .fields(&[
                ("$taxon", "AnimalTaxon"),
                ("$taxon_finding", "TaxonPresumptionRecord"),
                ("$taxon_source", "TaxonPresumptionSource"),
                ("$taxon_review", "TaxonPresumptionIndependentReviewer"),
                ("ECIndependentSpeciesAndIndividualClassificationEvidence", "AnimalClassification"),
                ("ECDirectBodilyIntegrityAvoidablePainFearDistressInjuryDiseaseAndDeprivationProtection", "AnimalBodilyInterest"),
                ("ECContinuedLifeSeriousNonAbsoluteInterest", "AnimalLifeInterest"),
                ("ECSpeciesAppropriateDependencyConditionsAndHumaneHandlingTransportTreatmentAndDeath", "AnimalCareInterest"),
                ("ECNoPersonFloorBallotCandidacyOfficePropertyTitleContractCapacityOrPoliticalWeight", "AnimalNotHumanStatus"),
                ("ECNoOwnerWealthPriceAffectionProductivityRarityOrGlobalAnimalWorthScore", "AnimalNoWorthRanking"),
                ("ECPropertyMayAllocateCareCostsAndCustodyButCannotWaiveDirectInterests", "AnimalPropertyBoundary"),
            ])
            .extra("(member($taxon, ECMinimumProtectedAnimalTaxon) | (complete($taxon_finding, ECAnimalTaxonExtension, $taxon) & authorized($taxon_source, ECSourceAuthority, $taxon_finding) & authorized($taxon_review, ECIndependentReviewAuthority, $taxon_finding) & observe($taxon_source, $taxon_finding, $version, ECConstitutionVersionScope) & observe($taxon_review, $taxon_finding, $version, ECConstitutionVersionScope) & ~($taxon_source = $taxon_review)))")
            .extra("~narrow($taxon, $version, ECAnimalTaxonPresumption)")
            .default("$taxon", "ECVertebrate")
            .effect("class($subject, ECCrediblySentientProtectedAnimal, $record)")
            .effect("prevents($subject, ECAnimalInterestWaiverByOwnerOrContract)"),
        Card::new("animal-urgent-protection", "ECUrgentPlausibleAnimalProtection", "Class6UrgentDirectAnimalProtection")
            .fields(&[
                ("$threat", "UrgentAnimalThreat"),
                ("ECCredibleUrgentThreatWhereCoveredAnimalInterestPlausiblyExists", "PlausibleProtectedInterest"),
                ("ECProtectFirstReconcileLaterWithoutOwnerChipPurchaseCivilIdentityOrCompletedTaxonRecord", "UrgentNoPaperworkGate"),
                ("ECTemporaryLocalSubjectHandleNotUniversalWildRegistryOrHumanIdentity", "UrgentSubjectHandle"),
                ("ECNecessaryLeastHarmRescueVeterinaryCareShelterAndContinuityRoute", "UrgentAnimalResponse"),
                ("ECPromptIndependentReviewAndSeparateHumanDueProcessForCoerciveSteps", "UrgentReviewBoundary"),
                ("ECNeitherOwnerConsentNorFinalFaultDeterminationPrerequisite", "UrgentNoOwnerVeto"),
                ("ECNoSearchEntrySeizureHumanCustodyOrPunishmentPowerFromThisFinding", "UrgentNoCoerciveShortcut"),
            ])
            .duty("ECProvideUrgentAnimalProtectionAndNecessaryCareBeforeRecordReconciliation")
            .effect("obliged($alternate, ECMaintainUrgentAnimalCareWhenOriginalRouteFails, $record)"),
        controlled(Card::new("animal-welfare", "ECControlledAnimalWelfareDuties", "Class10EveryControllerCareDuties")
            .depends(&["animal-coverage"])
            .fields(&[
                ("$control", "ActualControlOrCreatedDependencyEvidence"),
                ("$needs", "SpeciesIndividualControlDurationAndNeedsEvidence"),
                ("ECNourishmentWaterShelterMovementSocialBehaviourRestNecessaryCareAndSafeEnvironment", "MinimumAnimalCareConditions"),
                ("ECHumaneHandlingTransportTreatmentAndLawfullyJustifiedDeath", "AnimalHandlingConditions"),
                ("ECNoAbandonmentOverworkExtremeConfinementHarmfulBreedingOrAvoidableInvasion", "AnimalAvoidableHarmLimit"),
                ("ECContinuousPublicRescueVeterinaryShelterRehabilitationAndPlacementRoute", "ControllerFailureRoute"),
                ("ECCareDutiesNotProofAnimalIsHappyHealthyFulfilledTameUsefulOrBehavingNormally", "NoWelfareOutcomeInference"),
            ]))
            .effect("obliged($controller, ECProvideSpeciesAppropriateAnimalCareForActualControlAndDependency, $record)")
            .duty("ECPreventInvestigateAndRemedyPrivateAnimalInterferenceAndFailedCare"),
        controlled(Card::new("animal-categorical-bar", "ECAnimalCategoricalRefusal", "Class10NonWaivableAnimalConductLimit")
            .depends(&["animal-coverage"])
            .fields(&[
                ("$ground", "AnimalCategoricalGround"),
                ("ECIndependentExactConductEvidenceEstablishesThisCategoricalGround", "CategoricalAnimalFinding"),
                ("ECNoBalancingOfProhibitedHarmAgainstProfitOwnershipCustomResearchOrEcologicalOffice", "AnimalHardBarPriority"),
                ("ECMeaningfulConsultationEqualityAccommodationAndHumanRightsRemainWithoutCoreWaiver", "AnimalBarHumanRights"),
            ])
            .extra("member($ground, ECCategoricalAnimalBar)")
            .default("$ground", "ECDeliberateAnimalCruelty"))
            .effect("oppose($subject, $activity, ECAnimalUse)")
            .effect("prevents($record, ECAuthorizationOfCategoricallyProhibitedAnimalConduct)")
            .duty("ECStopAndRemedyTheExactCategoricallyProhibitedAnimalConduct"),
        permission(Card::new("animal-ordinary-use", "ECOrdinaryLowHarmAnimalUse", "Class10OrdinaryCompatibleAnimalUse")
            .depends(&["animal-welfare"])
            .fields(&[
                ("ECPositivelyNonFoodNonResearchNonTestingNonEducationUse", "OrdinaryAnimalDomain"),
                ("ECPositivelyNonlethalNonInvasiveAndNonHighSeverity", "OrdinaryAnimalSeverity"),
                ("ECCompanionshipRescueSanctuaryObservationOrMutuallySafeCoexistenceWithWelfare", "OrdinaryAnimalPurpose"),
                ("ECNoEnhancedSeriousPurposeTestMerelyBecauseAnOrdinaryCompatibleUseExists", "OrdinaryAnimalThreshold"),
            ]), "permits($controller, ECOnlyOrdinaryLowHarmAnimalUse, $record)"),
    ]
}
