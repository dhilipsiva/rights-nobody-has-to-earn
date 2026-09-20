// SPDX-License-Identifier: MIT OR Apache-2.0

//! Supplied water/river collision, with separate allocation and animal gates.
//! These records establish conditional duties and permissions, not measurements,
//! actual supplies, a judicial decision outside the fixture, or performed rescue.

use super::{
    Card, Values,
    cases::{self, Scenario},
    records,
};

fn replace(values: &mut Values, entries: &[(&str, &str)]) {
    for (variable, value) in entries {
        assert!(values.contains_key(*variable), "unknown {variable}");
        values.insert((*variable).into(), (*value).into());
    }
}

fn retain_human_floor() -> String {
    [
        ("person(RiverWaterClaimant)", true),
        ("owe(State, Healthy, RiverWaterClaimant)", true),
        ("decide(RiverWaterClaimant, Ballot)", true),
        ("healthy(RiverWaterClaimant)", false),
        ("false(RiverWaterClaimant)", false),
        ("prisoner(RiverWaterClaimant)", false),
    ]
    .into_iter()
    .map(|(atom, expected)| super::query(atom, expected))
    .collect()
}

fn without(facts: &str, values: &Values, premise: &str) -> String {
    let premise = records::ground(premise, values);
    assert_eq!(facts.matches(&premise).count(), 1, "exact removed premise");
    facts.replacen(&premise, "", 1)
}

pub(super) fn boundaries(cards: &[Card]) -> Vec<Scenario> {
    let mut result = Vec::new();
    let dual = super::card(cards, "dual-continuity");
    let mut water = records::values(cards, dual, "RiverWater");
    replace(
        &mut water,
        &[
            ("$record", "RiverWaterContinuityRecord"),
            ("$subject", "ExistingEssentialWaterService"),
            ("$case", "WaterAndRiverConflict"),
            ("$floor", "\"healthy\""),
            ("$axis", "ECWaterAxis"),
            ("$condition", "RiverMinimumFlow"),
            ("$route", "LeastHarmExistingSupplyForImmediateHealth"),
            ("$transition", "ObtainAlternativeSupplyBeforeSourceBoundEnd"),
        ],
    );
    let people = "person(RiverWaterClaimant).\nat(RiverWaterClaimant, GeneralAdulthood).\n";
    let water_facts = records::fixture(cards, dual, &water) + people;
    let floor = retain_human_floor();

    let allocation = super::card(cards, "ecological-scarcity-allocation");
    let mut scarce = records::values(cards, allocation, "RiverAllocation");
    replace(
        &mut scarce,
        &[
            ("$record", "RiverWaterAllocationRecord"),
            ("$resource", "CurrentSafeWaterSupply"),
            (
                "$alternatives_or_allocation",
                "ReviewedCurrentWaterAllocation",
            ),
        ],
    );
    cases::join_parent(cards, allocation, "dual-continuity", &mut scarce, &water);
    result.push(Scenario {
        id: "scarcity/water-floor-and-river-limit".into(),
        facts: water_facts.clone(),
        pins: cases::queries(dual, &water, true)
            + &cases::queries(allocation, &scarce, false)
            + &floor,
    });
    // The actual economic finding and allocation proof, with all their
    // premises, are supplied by the existing external connector here.
    result.push(Scenario {
        id: "scarcity/water-separate-physical-allocation".into(),
        facts: records::fixture(cards, allocation, &scarce) + people,
        pins: cases::queries(dual, &water, true)
            + &cases::queries(allocation, &scarce, true)
            + &floor,
    });
    for (id, premise) in [
        (
            "water-no-positive-incompatibility",
            "observe($source, $record, ECPositiveEvidenceNoPresentRouteMeetsBothFloorAndCeiling, ECDualIncompatibilityEvidenceScope).\n",
        ),
        (
            "water-procurement-not-examined",
            "observe($source, $record, ECActualFeasibleAlternativesAndProcurementEffortExamined, ECDualAlternativesScope).\n",
        ),
    ] {
        result.push(Scenario {
            id: format!("scarcity/{id}"),
            facts: without(&water_facts, &water, premise),
            pins: cases::queries(dual, &water, false) + &floor,
        });
    }

    let rescue = super::card(cards, "animal-enhanced-nonfood-use");
    let mut fish = records::values(cards, rescue, "RiverFish");
    replace(
        &mut fish,
        &[
            ("$record", "RiverFishRescueRecord"),
            ("$subject", "FishExposedByRiverWithdrawal"),
            ("$case", "WaterAndRiverConflict"),
            ("$controller", "RiverFishCareController"),
            ("$purpose", "ECEvidenceBasedConservationOrRestoration"),
        ],
    );
    let conflict = super::card(cards, "animal-guardian-conflict");
    let mut decision = records::values(cards, conflict, "RiverConflict");
    replace(
        &mut decision,
        &[
            ("$record", "RiverIndependentConflictRecord"),
            ("$case", "WaterAndRiverConflict"),
            (
                "$guardian_position",
                "ProtectRiverMinimumFlowAndFutureConditions",
            ),
            (
                "$animal_position",
                "ProtectIndividualFishLifeCareAndLeastHarmRescue",
            ),
            ("$human_right", "PresentSafeWaterEnvironmentalClaim"),
            ("$human_floor", "ImmediateHealthContinuityClaim"),
            ("$commons", "RiverMinimumFlowAndLivingSystemCondition"),
            (
                "$animal_interests",
                "IndividualFishLifeBodilyIntegrityAndCare",
            ),
            ("$collective", "AffectedCollectiveRightsRemainIndependent"),
            (
                "$alternatives",
                "ActualAlternateSupplyAndLessHarmRescueExamined",
            ),
            (
                "$uncertainty",
                "DisclosedLimitsOfFlowAndInterventionEvidence",
            ),
            (
                "$reversibility",
                "InterimSupplyAndNoNewIrreversibleActivity",
            ),
            ("$continuity", "HumanWaterAndAnimalCareBothRemainOwed"),
            (
                "$decision",
                "LeastHarmInterimWaterAndSeparatelyQualifiedFishRescue",
            ),
        ],
    );
    let conflict_facts = records::fixture(cards, conflict, &decision) + &water_facts;
    result.push(Scenario {
        id: "scarcity/water-animal-conflict-is-not-use-permission".into(),
        facts: conflict_facts.clone(),
        pins: cases::queries(conflict, &decision, true)
            + &cases::queries(dual, &water, true)
            + &cases::queries(rescue, &fish, false)
            + &cases::queries(allocation, &scarce, false)
            + &floor,
    });
    let fish_facts = records::fixture(cards, rescue, &fish);
    result.push(Scenario {
        id: "scarcity/water-separately-reviewed-animal-rescue".into(),
        facts: conflict_facts.clone() + &fish_facts,
        pins: cases::queries(conflict, &decision, true)
            + &cases::queries(rescue, &fish, true)
            + &floor,
    });
    let enhanced = super::card(cards, "animal-enhanced-test");
    let dependency = rescue
        .dependencies
        .iter()
        .find(|d| d.id == enhanced.id)
        .unwrap();
    let prior_review = records::pattern().replace_all(
        "observe($source, $record, ECIndependentReviewPrecedesHighConsequenceUseWithReasonsChallengeReassessmentRemedyAndOwnEnd, ECEnhancedAnimalPriorReviewFindingScope).\n",
        |m: &regex::Captures<'_>| records::dependency_variable(rescue, enhanced, dependency, &m[0]),
    );
    result.push(Scenario {
        id: "scarcity/water-rescue-without-separate-prior-review".into(),
        facts: conflict_facts + &without(&fish_facts, &fish, &prior_review),
        pins: cases::queries(conflict, &decision, true)
            + &cases::queries(rescue, &fish, false)
            + &floor,
    });
    result
}
