// SPDX-License-Identifier: MIT OR Apache-2.0

//! Substantive animal cases, including genuine stateful loss of permission.

use super::{
    Card, animals,
    cases::{self, Scenario, Stages},
    records,
};

pub(super) fn boundaries(cards: &[Card]) -> Vec<Scenario> {
    let coverage = super::card(cards, "animal-coverage");
    let urgent = super::card(cards, "animal-urgent-protection");
    let mut result = Vec::new();
    for taxon in animals::MINIMUM_TAXA {
        let mut values = records::values(cards, coverage, "ECMinimumAnimal");
        values.insert("$taxon".into(), (*taxon).into());
        let mut pins = cases::queries(coverage, &values, true);
        pins += &human_status_wall(&values["$subject"]);
        result.push(Scenario {
            id: format!("animal/minimum-{taxon}"),
            facts: records::fixture(cards, coverage, &values),
            pins,
        });
    }
    let mut uncertain = records::values(cards, coverage, "ECUncertainAnimal");
    uncertain.insert("$taxon".into(), "ECUnclassifiedPossibleAnimal".into());
    let mut rescue = records::values(cards, urgent, "ECUnregisteredAnimalRescue");
    rescue.insert("$subject".into(), uncertain["$subject"].clone());
    result.push(Scenario {
        id: "animal/uncertain-unregistered-interest-urgent-protection-without-classification"
            .into(),
        facts: records::fixture(cards, coverage, &uncertain)
            + &records::fixture(cards, urgent, &rescue),
        pins: cases::conclusion(coverage, &uncertain, false)
            + &cases::queries(urgent, &rescue, true)
            + &human_status_wall(&rescue["$subject"]),
    });
    let extension = super::card(cards, "animal-taxon-extension");
    let removal = super::card(cards, "animal-taxon-removal");
    let e = records::values(cards, extension, "ECExtendedTaxon");
    let mut animal = records::values(cards, coverage, "ECExtendedAnimal");
    for (to, from) in [
        ("$taxon", "$taxon"),
        ("$taxon_finding", "$record"),
        ("$taxon_source", "$source"),
        ("$taxon_review", "$review"),
        ("$version", "$version"),
    ] {
        animal.insert(to.into(), e[from].clone());
    }
    let mut removed = records::values(cards, removal, "ECRemovedTaxon");
    for (to, from) in [
        ("$taxon", "$taxon"),
        ("$version", "$version"),
        ("$prior_finding", "$record"),
        ("$prior_evidence", "$evidence_record"),
    ] {
        removed.insert(to.into(), e[from].clone());
    }
    rescue.insert("$subject".into(), animal["$subject"].clone());
    let mut stages = Stages::default();
    let activate = stages.add(cards, removal, &removed);
    result.push(Scenario {
        id: "animal/realistic-possibility-extension-rigorous-removal-and-urgent-continuity".into(),
        facts: records::fixture(cards, extension, &e)
            + &records::fixture(cards, coverage, &animal)
            + &records::fixture(cards, urgent, &rescue)
            + &stages.finish(),
        pins: cases::queries(extension, &e, true)
            + &cases::queries(coverage, &animal, true)
            + &activate
            + &cases::queries(removal, &removed, true)
            + &cases::conclusion(coverage, &animal, false)
            + &cases::queries(urgent, &rescue, true)
            + &human_status_wall(&animal["$subject"]),
    });
    let categorical = super::card(cards, "animal-categorical-bar");
    for ground in animals::CATEGORICAL_BARS {
        let id = match *ground {
            "ECUnrelievedSevereOrProlongedResearchProcedure"
            | "ECDispensableKillingOrSevereSufferingForCosmeticsOrMarketing" => {
                "animal-research-high-impact"
            }
            "ECDispensableKillingOrSevereSufferingForProfit" => "animal-food-high-impact",
            _ => "animal-ordinary-use",
        };
        let use_card = super::card(cards, id);
        let use_values = records::values(cards, use_card, "ECAnimalProhibitedUse");
        let mut bar = records::values(cards, categorical, "ECAnimalHardBar");
        for variable in ["$subject", "$case", "$controller", "$activity", "$version"] {
            bar.insert(variable.into(), use_values[variable].clone());
        }
        bar.insert("$ground".into(), (*ground).into());
        let mut stages = Stages::default();
        let activate = stages.add(cards, categorical, &bar);
        result.push(Scenario {
            id: format!("animal/categorical-{ground}"),
            facts: records::fixture(cards, use_card, &use_values) + &stages.finish(),
            pins: cases::queries(use_card, &use_values, true)
                + &activate
                + &cases::queries(categorical, &bar, true)
                + &cases::conclusion(use_card, &use_values, false)
                + &human_status_wall(&bar["$subject"]),
        });
    }
    result
}

fn human_status_wall(subject: &str) -> String {
    let mut pins = super::query(&format!("person({subject})"), false);
    for floor in super::protection::FLOOR {
        pins += &super::query(
            &format!("entitled({subject}, event {{ {floor}() }})"),
            false,
        );
    }
    for atom in [
        format!("authority({subject})"),
        format!("prisoner({subject})"),
        format!("punish(Court, {subject})"),
        format!("capture({subject})"),
        format!("restrain({subject})"),
    ] {
        pins += &super::query(&atom, false);
    }
    pins
}
