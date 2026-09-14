// SPDX-License-Identifier: MIT OR Apache-2.0

//! Named domain alternatives and actual upstream-proof refusal at every edge.

use super::{
    Card, animal_context, animal_use,
    cases::{self, Scenario},
    continuity, environment, records,
};

pub(super) fn boundaries(cards: &[Card]) -> Vec<Scenario> {
    let mut result = Vec::new();
    for (id, variable, choices) in [
        ("environmental-claim", "$interest", environment::INTERESTS),
        ("environmental-claim", "$reach", environment::REACH),
        ("science", "$axis", environment::AXES),
        ("nonfungibility", "$ground", environment::NONFUNGIBLE),
        (
            "guardian-alternate-function",
            "$failure",
            continuity::FAILURES,
        ),
        (
            "animal-alternate-function",
            "$failure",
            continuity::FAILURES,
        ),
        (
            "substitute-reviewer-function",
            "$failure",
            continuity::FAILURES,
        ),
        ("animal-enhanced-test", "$impact", animal_use::IMPACTS),
        ("animal-enhanced-test", "$purpose", animal_use::PURPOSES),
        (
            "animal-research-test",
            "$research_purpose",
            animal_use::RESEARCH_PURPOSES,
        ),
        ("animal-domestic", "$ownership", animal_context::OWNERSHIPS),
        ("animal-farmed", "$ownership", animal_context::OWNERSHIPS),
        ("animal-captive", "$ownership", animal_context::OWNERSHIPS),
        (
            "animal-wild-human-harm",
            "$ground",
            animal_context::HUMAN_HARMS,
        ),
        (
            "animal-grave-risk-test",
            "$ground",
            animal_context::GRAVE_RISKS,
        ),
        (
            "orphan-restoration",
            "$failure",
            &[
                "ECResponsibleActorUnknown",
                "ECResponsibleActorAbsent",
                "ECResponsibleActorInsolvent",
                "ECResponsibleActorUnableToActInTime",
            ][..],
        ),
        (
            "contribution-liability",
            "$basis",
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
    ] {
        let card = super::card(cards, id);
        for (value, expected) in choices
            .iter()
            .map(|value| (*value, true))
            .chain([("ECUnrecognizedStatusOrUnsupportedGround", false)])
        {
            let mut values = records::values(cards, card, "ECDomainChoice");
            assert!(values.contains_key(variable), "{id}: unknown {variable}");
            values.insert(variable.into(), value.into());
            result.push(Scenario {
                id: format!("domain/{id}-{}-{value}", variable.trim_start_matches('$')),
                facts: records::fixture(cards, card, &values),
                pins: cases::queries(card, &values, expected),
            });
        }
    }
    for card in cards {
        let values = records::values(cards, card, "ECActualUpstreamProof");
        let fixture = records::fixture(cards, card, &values);
        for dependency in &card.dependencies {
            let parent = super::card(cards, dependency.id);
            let credential = records::pattern().replace_all(
                "authorized($review, ECIndependentReviewAuthority, $record).\n",
                |m: &regex::Captures<'_>| {
                    records::dependency_variable(card, parent, dependency, &m[0])
                },
            );
            let fact = records::ground(&credential, &values);
            assert_eq!(
                fixture.matches(&fact).count(),
                1,
                "{} {}: actual unique upstream credential",
                card.id,
                dependency.key
            );
            result.push(Scenario {
                id: format!(
                    "upstream/{}/without-{}-actual-review",
                    card.id, dependency.key
                ),
                facts: fixture.replacen(&fact, "", 1),
                pins: cases::conclusion(card, &values, false),
            });
        }
    }
    result.extend(record_key_boundaries(cards));
    result
}

pub(super) fn record_key_boundaries(cards: &[Card]) -> Vec<Scenario> {
    cards
        .iter()
        .filter(|card| {
            !records::pattern()
                .find_iter(&records::heads(card)[0])
                .any(|variable| variable.as_str() == "$record")
        })
        .map(|card| {
            let good = records::values(cards, card, "ECExactEffect");
            let mut bad = good.clone();
            bad.insert("$record".into(), "ECUnqualifiedEffectRecord".into());
            let missing = if card.id == "animal-taxon-removal" {
                bad.insert("$operator".into(), "ECWrongTaxonRevisionActor".into());
                None
            } else {
                bad.insert("$binder".into(), "ECUncredentialedAmendmentBinder".into());
                Some(records::ground(
                    "authorized($binder, AmendmentSourceBindingAuthority, $amendment_record).\n",
                    &bad,
                ))
            };
            let mut invalid = records::fixture(cards, card, &bad);
            if let Some(missing) = missing {
                assert_eq!(invalid.matches(&missing).count(), 1);
                invalid = invalid.replacen(&missing, "", 1);
            }
            let mut pins = cases::queries(card, &good, true);
            // The shared taxon/candidate conclusion remains valid, but cannot lend
            // its authority to a different, unqualified finding's own duty.
            for effect in records::heads(card).iter().skip(1).filter(|effect| {
                records::pattern()
                    .find_iter(effect)
                    .any(|variable| variable.as_str() == "$record")
            }) {
                pins += &super::query(&records::ground(effect, &bad), false);
            }
            Scenario {
                id: format!("record-key/{}-no-borrowed-effect", card.id),
                facts: records::fixture(cards, card, &good) + &invalid,
                pins,
            }
        })
        .collect()
}
