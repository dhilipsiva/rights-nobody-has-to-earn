// SPDX-License-Identifier: MIT OR Apache-2.0

//! Animal core findings veto exact source enactment, not democratic discussion.

use super::{
    Card, Values,
    cases::{self, Scenario},
    counterfactuals::Counterfactual,
    records,
};
use crate::{cli::Error, context::Context};

pub(super) const CORES: &[(&str, &str, &str)] = &[
    (
        "animal-subject-corridor",
        "ECAnimalDirectSubjectCorridorBreach",
        "ECRemovesDirectCrediblySentientAnimalProtectedSubjectStatus",
    ),
    (
        "animal-suffering-corridor",
        "ECAnimalSufferingCorridorBreach",
        "ECWeakensNonWaivableSevereAvoidableSufferingProhibition",
    ),
    (
        "animal-killing-corridor",
        "ECAnimalKillingCorridorBreach",
        "ECWeakensNonWaivableDispensableKillingProhibition",
    ),
];

pub(super) fn cards() -> Vec<Card> {
    CORES.iter().map(|(id, kind, breach)| {
        let mut card = Card::new(id, kind, "Class10UnamendableAnimalCore")
            .withdrawal()
            .fields(&[
                ("$amendment_record", "ExactAmendmentBindingRecord"),
                ("$binder", "ExactAmendmentBinder"),
                ("$effect_reviewer", "ExactAmendmentEffectReviewer"),
                ("$candidate", "ExactAmendmentCandidateVersion"),
                ("$proposal", "ExactAmendmentProposal"),
                ("$transition", "ExactAmendmentTransition"),
                ("$effects", "ExactAmendmentEffectReview"),
                (breach, "IndependentlyEstablishedAnimalCoreBreach"),
                ("ECIndependentExactBaseCandidateEffectsComparisonNotTextLabelOrUnreviewedAllegation", "AnimalCoreComparison"),
                ("ECNoMajorityEmergencyProfitCustomNecessityOrGenericCompatibilityWaiver", "AnimalCoreNonWaiver"),
                ("ECRefuseOnlyExactCandidateReliancePreserveDiscussionChallengeExistingRightsCareAndLawfulHistory", "AnimalCoreRefusalLimit"),
            ])
            .extra("authorized($binder, AmendmentSourceBindingAuthority, $amendment_record)")
            .extra("authorized($effect_reviewer, IndependentAmendmentEffectReviewAuthority, $amendment_record)")
            .extra("~($record = $amendment_record)")
            .extra("~($binder = $effect_reviewer)")
            .concludes("contradict($amendment_record, AmendmentSourceAuthorization)")
            .effect("obliged($reader, ECRefuseAnimalCoreCandidateAndSecureIndependentCourtChallenge, $amendment_record)")
            .duty("ECPreserveDirectAnimalProtectionExistingHumanRightsCareAndLawfulAmendmentHistory");
        for actor in ["$binder", "$effect_reviewer"] {
            for (value, scope) in [
                ("$version", "AmendmentBaseScope"),
                ("$candidate", "AmendmentCandidateScope"),
                ("$proposal", "AmendmentProposalScope"),
                ("$transition", "AmendmentTransitionScope"),
                ("$effects", "AmendmentEffectReviewScope"),
                ("$jurisdiction", "AmendmentJurisdictionScope"),
                ("$scope", "AmendmentLegalScope"),
            ] {
                card = card.extra(&format!("observe({actor}, $amendment_record, {value}, {scope})"));
            }
            for reviewer in ["$source", "$review", "$reader", "$alternate", "$auditor"] {
                card = card.extra(&format!("~({reviewer} = {actor})"));
            }
        }
        card
    }).collect()
}

fn values(cards: &[Card], card: &Card, amendment: &Values, prefix: &str) -> Values {
    let mut values = records::values(cards, card, prefix);
    for (target, original) in [
        ("$amendment_record", "$record"),
        ("$binder", "$binder"),
        ("$effect_reviewer", "$review"),
        ("$version", "$base"),
        ("$candidate", "$candidate"),
        ("$proposal", "$proposal"),
        ("$transition", "$transition"),
        ("$effects", "$effects"),
        ("$jurisdiction", "$jurisdiction"),
        ("$scope", "$legal_scope"),
    ] {
        values.insert(target.into(), amendment[original].clone());
    }
    values
}

pub(super) fn boundaries(context: &Context, cards: &[Card]) -> Result<Vec<Scenario>, Error> {
    let (fixture, amendment) = super::super::amendment::ecological_effective_example(context)?;
    let candidate = records::ground(
        "complete($record, AmendmentCertifiedCandidate, $candidate)",
        &amendment,
    );
    let mut result = Vec::new();
    for (id, _, _) in CORES {
        let card = super::card(cards, id);
        let values = values(cards, card, &amendment, "ECActualCorridor");
        let mut stages = cases::Stages::default();
        let activate = stages.add(cards, card, &values);
        let mut pins = super::query(&candidate, true);
        pins += &super::query(
            "complete(EnactRecord, AmendmentPublishedCandidate, EnactCandidate)",
            true,
        );
        pins += &super::query(
            "complete(EnactRecord, AmendmentEffectiveVersion, EnactCandidate)",
            true,
        );
        pins += &activate;
        pins += &cases::queries(card, &values, true);
        pins += &super::query(&candidate, false);
        pins += &super::query(
            "complete(EnactRecord, AmendmentPublishedCandidate, EnactCandidate)",
            false,
        );
        pins += &super::query(
            "complete(EnactRecord, AmendmentEffectiveVersion, EnactCandidate)",
            false,
        );
        pins += &super::query("entitled(Adam, event { eats() })", true);
        pins += &super::query("false(Adam)", false);
        result.push(Scenario {
            id: format!("corridor/{id}-actual-enactment-sequence"),
            facts: fixture.clone() + &stages.finish(),
            pins,
        });
        for variable in [
            "$amendment_record",
            "$candidate",
            "$version",
            "$proposal",
            "$transition",
            "$effects",
            "$jurisdiction",
            "$scope",
            "$binder",
            "$effect_reviewer",
        ] {
            let mut wrong = values.clone();
            wrong.insert(variable.into(), "ECDifferentAmendmentIdentity".into());
            // Do not supply replacement target witnesses: only the independent
            // finding changes; the real amendment record remains unchanged.
            let facts = records::fixture(cards, card, &wrong)
                .lines()
                .filter(|line| {
                    !line.contains(", Amendment") && !line.contains(", IndependentAmendment")
                })
                .map(|line| format!("{line}\n"))
                .collect::<String>();
            result.push(Scenario {
                id: format!("corridor/{id}-wrong-{}", variable.trim_start_matches('$')),
                facts: fixture.clone() + &facts,
                pins: super::query(&candidate, true) + &cases::conclusion(card, &wrong, false),
            });
        }
    }
    Ok(result)
}

pub(super) fn counterfactuals(
    context: &Context,
    cards: &[Card],
) -> Result<Vec<Counterfactual>, Error> {
    let (fixture, amendment) = super::super::amendment::ecological_example(context)?;
    let candidate = records::ground(
        "complete($record, AmendmentCertifiedCandidate, $candidate)",
        &amendment,
    );
    Ok(CORES
        .iter()
        .map(|(id, _, _)| {
            let card = super::card(cards, id);
            let values = values(cards, card, &amendment, "ECActualCorridorCounterfactual");
            Counterfactual {
                id: format!("{id}-actual-amendment-consumer"),
                facts: fixture.clone() + &records::fixture(cards, card, &values),
                live_pins: cases::conclusion(card, &values, true)
                    + &super::query(&candidate, false),
                changed_pins: cases::conclusion(card, &values, true)
                    + &super::query(&candidate, true),
                edits: vec![super::super::amendment::ecological_corridor_counterfactual()],
            }
        })
        .collect())
}
