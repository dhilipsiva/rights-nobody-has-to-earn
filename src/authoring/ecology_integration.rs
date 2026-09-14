// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ecology does not replace physical scarcity or collective-rights contracts.

use super::Card;

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, allocation) in [
        (
            "ecological-physical-scarcity",
            "ECActualPhysicalScarcityDuringDualFailure",
            false,
        ),
        (
            "ecological-scarcity-allocation",
            "ECExistingScarcityAllocationDuringDualFailure",
            true,
        ),
    ] {
        let mut card = Card::new(id, kind, "Class2ExistingScarcityInterfaceWithEcologicalFailure")
            .depends(&["dual-continuity"])
            .fields(&[
                ("$resource", "ExactScarceResource"), ("$population", "ExactScarcityPopulation"),
                ("$alternatives_or_allocation", "ActualScarcityAlternativesOrAllocationRecord"),
                ("ECActualPhysicalScarcitySourceRequiredEcologicalCollisionIsNotPhysicalScarcity", "DistinctScarcityKinds"),
                ("ECAllAffectedPeopleWithoutMigrationStatusWorthProductivityOrOwnershipPriority", "ScarcityPopulationEquality"),
                ("ECFloorShortfallAndEcologicalBreachRemainSeparateFailuresWithContinuityRepairAndReassessment", "ScarcityDualFailurePolarity"),
                ("ECNoBudgetDelayMonopolyWithholdingProcurementRefusalOrEcologicalLabelAsScarcityProof", "ScarcityFalseGrounds"),
                ("ECNoPermissionForNewIrreversibleActivityOrAnimalCoreWaiverFromScarcity", "ScarcityNoEcologicalWaiver"),
            ])
            .extra("$operator = FSBOD_09").default("$operator", "FSBOD_09")
            .duty("ECPreserveActualScarcitySafeguardsFloorContinuityAndSeparateEcologicalFailure");
        if allocation {
            card = card.concludes("permits($operator, ECOnlyExistingRightsPreservingPhysicalScarcityAllocation, $record)");
        }
        cards.push(card);
    }
    for (id, kind, consent) in [
        (
            "ecological-collective-consent",
            "ECActualCollectiveConsentForEcologicalEffect",
            true,
        ),
        (
            "ecological-collective-consultation",
            "ECActualPriorCollectiveEcologicalConsultation",
            false,
        ),
    ] {
        let mut card = Card::new(id, kind, "Class8SeparateCollectiveRightsDuringEcologicalProtection")
            .fields(&[
                ("$collective_holder", "AffectedCollectiveHolder"), ("$collective_title", "ExistingCollectiveTitle"),
                ("$project", "ExactEcologicalProject"), ("$authorization_version", "ExactEcologicalProjectVersion"),
                ("ECIndividualBodilyHousingChildAndPoliticalRightsRemainIndependentOfCollectiveDecision", "CollectiveIndividualRightsWall"),
                ("ECOwnershipCustomaryOrIndigenousTitleNotLostByEcologicalLimitOrAnimalProtection", "CollectiveNoDispossession"),
                ("ECMeaningfulIndigenousSubsistenceReligiousAndConscienceAccommodationWithoutSevereAvoidableSufferingWaiver", "CollectiveAccommodationBoundary"),
                ("ECNeitherConsentNorConsultationAuthorizesCeilingBreachAnimalCoreViolationOrUnrelatedCoercion", "CollectiveNoCoreWaiver"),
                ("ECNecessaryProcessNotSufficientActivityPermissionEnactmentOrPerformedOperation", "CollectiveNoExecutionInference"),
            ])
            .duty("ECRespectActualCollectiveProcessAndIndependentHumanAnimalAndEcologicalLimits");
        if consent {
            card = card.fields(&[("$effect", "IndependentlyClassifiedConsentRequiredEffect")]);
        } else {
            card = card.fields(&[(
                "OtherMaterialNonexistentialEffect",
                "IndependentlyClassifiedConsultationEffect",
            )]);
        }
        cards.push(card);
    }
    cards
}
