// SPDX-License-Identifier: MIT OR Apache-2.0

//! Paired live refusals and explicit, uniquely matched source transformations.

use super::{Card, Values, cases, lifecycle_cases, records};
use crate::{authoring::Edit, cli::Error};
use std::collections::BTreeSet;

pub(super) struct Counterfactual {
    pub id: String,
    pub facts: String,
    pub live_pins: String,
    pub changed_pins: String,
    pub edits: Vec<Edit>,
}

fn primary_without(cards: &[Card], card: &Card, atom: &str) -> Result<Vec<Edit>, Error> {
    let suffix = format!(" -> {}.", records::heads(card)[0]);
    let own_kind = records::observe("$source", "$record", card.kind, "Kind");
    let rules = records::rules(cards);
    let matching: Vec<_> = rules
        .iter()
        .filter(|rule| rule.ends_with(&suffix) && rule.contains(&own_kind))
        .collect();
    if matching.len() != 1 {
        return Err(Error::new(format!(
            "{} counterfactual requires one exact primary rule; found {}",
            card.id,
            matching.len()
        )));
    }
    let before = matching[0];
    let needle = format!(" & {atom}");
    if before.matches(&needle).count() != 1 {
        return Err(Error::new(format!(
            "counterfactual guard is absent or ambiguous: {atom}"
        )));
    }
    Ok(vec![Edit {
        before: before.to_string(),
        after: before.replacen(&needle, "", 1),
    }])
}

fn changed_rules(cards: &[Card], changed: &[Card]) -> Result<Vec<Edit>, Error> {
    super::validate(changed)?;
    let before: BTreeSet<_> = records::rules(cards).into_iter().collect();
    let after: BTreeSet<_> = records::rules(changed).into_iter().collect();
    let removed: Vec<_> = before.difference(&after).collect();
    let added: Vec<_> = after.difference(&before).cloned().collect();
    if removed.is_empty() || added.is_empty() {
        return Err(Error::new(
            "ecological counterfactual did not replace a rule",
        ));
    }
    let edits: Vec<_> = removed
        .into_iter()
        .enumerate()
        .map(|(index, rule)| Edit {
            before: format!("{rule}\n"),
            // Anchor every edit in a real statement. The execution inventory
            // deliberately ignores comments, including authoring block tags.
            after: if index == 0 {
                format!("{}\n", added.join("\n"))
            } else {
                String::new()
            },
        })
        .collect();
    Ok(edits)
}

fn paired(
    id: &str,
    card: &Card,
    values: &Values,
    facts: String,
    edits: Vec<Edit>,
) -> Counterfactual {
    Counterfactual {
        id: id.into(),
        facts,
        live_pins: cases::conclusion(card, values, false),
        changed_pins: cases::conclusion(card, values, true),
        edits,
    }
}

pub(super) fn cases(cards: &[Card]) -> Result<Vec<Counterfactual>, Error> {
    let mut result = Vec::new();
    let residual = super::card(cards, "residual-restoration");
    let r = records::values(cards, residual, "ECCounterfactualResidual");
    let nonfungible = super::card(cards, "nonfungibility");
    let mut n = records::values(cards, nonfungible, "ECCounterfactualIrreplaceable");
    for variable in ["$axis", "$condition", "$project"] {
        n.insert(variable.into(), r[variable].clone());
    }
    result.push(paired(
        "irreplaceable-loss-cannot-be-bought-by-an-offset",
        residual,
        &r,
        records::fixture(cards, residual, &r) + &records::fixture(cards, nonfungible, &n),
        primary_without(cards, residual, "~oppose($condition, $project, $axis)")?,
    ));

    let basis = super::card(cards, "high-consequence-basis");
    let b = records::values(cards, basis, "ECCounterfactualAxis");
    let climate = basis
        .dependencies
        .iter()
        .find(|dependency| dependency.key == "climate")
        .unwrap();
    let parent = super::card(cards, climate.id);
    let credential = records::pattern().replace_all(
        "authorized($review, ECIndependentReviewAuthority, $record).\n",
        |m: &regex::Captures<'_>| records::dependency_variable(basis, parent, climate, &m[0]),
    );
    let credential = records::ground(&credential, &b);
    let facts = records::fixture(cards, basis, &b);
    if facts.matches(&credential).count() != 1 {
        return Err(Error::new("axis counterfactual needs exact climate review"));
    }
    let mut changed = cards.to_vec();
    let altered = changed.iter_mut().find(|card| card.id == basis.id).unwrap();
    altered
        .dependencies
        .retain(|dependency| dependency.key != "climate");
    altered
        .dependency_bindings
        .retain(|(key, _), _| key != "climate");
    result.push(paired(
        "other-axes-cannot-replace-missing-climate-compliance",
        basis,
        &b,
        facts.replacen(&credential, "", 1),
        changed_rules(cards, &changed)?,
    ));

    let claim = super::card(cards, "environmental-claim");
    let mut values = records::values(cards, claim, "ECCounterfactualIndependence");
    values.insert("$review".into(), values["$source"].clone());
    result.push(paired(
        "source-cannot-review-itself",
        claim,
        &values,
        records::fixture(cards, claim, &values),
        primary_without(cards, claim, "~($source = $review)")?,
    ));

    let values = records::values(cards, claim, "ECCounterfactualReader");
    let credential = "authorized($reader, ECChallengeReaderAuthority, $record)";
    let fact = records::ground(&format!("{credential}.\n"), &values);
    let fixture = records::fixture(cards, claim, &values);
    if fixture.matches(&fact).count() != 1 {
        return Err(Error::new("missing exact challenge-reader fixture"));
    }
    result.push(paired(
        "actual-independent-challenge-reader",
        claim,
        &values,
        fixture.replacen(&fact, "", 1),
        primary_without(cards, claim, credential)?,
    ));

    let ordinary = super::card(cards, "animal-ordinary-use");
    let u = records::values(cards, ordinary, "ECCounterfactualAnimalUse");
    let categorical = super::card(cards, "animal-categorical-bar");
    let mut bar = records::values(cards, categorical, "ECCounterfactualAnimalBar");
    for variable in ["$subject", "$case", "$controller", "$activity", "$version"] {
        bar.insert(variable.into(), u[variable].clone());
    }
    result.push(paired(
        "categorical-bar-overrides-complete-use",
        ordinary,
        &u,
        records::fixture(cards, ordinary, &u) + &records::fixture(cards, categorical, &bar),
        primary_without(cards, ordinary, "~oppose($subject, $activity, ECAnimalUse)")?,
    ));

    let old = records::values(cards, ordinary, "ECCounterfactualOldWindow");
    let ended = super::card(cards, "ecological-record-end");
    let e = lifecycle_cases::affected_values(cards, ended, ordinary, &old, "ECCounterfactualEnd");
    let mut renamed = records::values(cards, ordinary, "ECCounterfactualRenamed");
    for variable in ["$window", "$start", "$end"] {
        renamed.insert(variable.into(), old[variable].clone());
    }
    result.push(paired(
        "record-renaming-cannot-revive-ended-window",
        ordinary,
        &renamed,
        records::fixture(cards, ordinary, &old)
            + &records::fixture(cards, ended, &e)
            + &records::fixture(cards, ordinary, &renamed),
        primary_without(
            cards,
            ordinary,
            "~contradict($window, ECSourceWindowCurrentReliance)",
        )?,
    ));

    for (id, consumer, parent, scope) in [
        (
            "all-food-use-needs-accessible-nutrition-alternative-finding",
            "animal-food-low-impact",
            "animal-food-test",
            "StrictAnimalFoodAlternativeFinding",
        ),
        (
            "research-needs-valid-replacement-alternative-finding",
            "animal-research-low-impact",
            "animal-research-test",
            "AnimalResearchAlternativesFinding",
        ),
        (
            "enhanced-use-needs-less-harm-alternative-finding",
            "animal-enhanced-nonfood-use",
            "animal-enhanced-test",
            "EnhancedAnimalAlternativesFinding",
        ),
    ] {
        let consumer = super::card(cards, consumer);
        let values = records::values(cards, consumer, "ECCounterfactualAlternative");
        let fixture = records::fixture(cards, consumer, &values);
        let suffix = format!(", EC{scope}Scope).");
        let removed = fixture
            .lines()
            .filter(|line| line.ends_with(&suffix))
            .count();
        if removed != 2 {
            return Err(Error::new(format!(
                "expected two exact alternative witnesses, found {removed}"
            )));
        }
        let facts = fixture
            .lines()
            .filter(|line| !line.ends_with(&suffix))
            .map(|line| format!("{line}\n"))
            .collect();
        let mut changed = cards.to_vec();
        let parent = changed.iter_mut().find(|card| card.id == parent).unwrap();
        let length = parent.fields.len();
        parent.fields.retain(|(_, field)| *field != scope);
        if parent.fields.len() + 1 != length {
            return Err(Error::new("ambiguous alternative-finding field"));
        }
        result.push(paired(
            id,
            consumer,
            &values,
            facts,
            changed_rules(cards, &changed)?,
        ));
    }

    let stay = super::card(cards, "guardian-stay");
    let s = records::values(cards, stay, "ECCounterfactualRegistry");
    let credential = "authorized($registry_review, ECReplayReviewAuthority, $registry)";
    let fact = records::ground(&format!("{credential}.\n"), &s);
    let fixture = records::fixture(cards, stay, &s);
    if fixture.matches(&fact).count() != 1 {
        return Err(Error::new("missing actual replay-review credential"));
    }
    result.push(paired(
        "shared-replay-registry-needs-independent-review",
        stay,
        &s,
        fixture.replacen(&fact, "", 1),
        primary_without(cards, stay, credential)?,
    ));

    let history = super::card(cards, "guardian-final-history");
    let h = records::values(cards, history, "ECCounterfactualFinalHistory");
    let mut s = records::values(cards, stay, "ECCounterfactualReplay");
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
        "$evidence_version",
        "$ground",
    ] {
        s.insert(variable.into(), h[variable].clone());
    }
    let edits: Vec<_> = records::rules(cards)
        .into_iter()
        .filter(|rule| {
            rule.contains("complete($final_record, ECHistoricalGuardianFinalDisposition,")
                && rule.contains(" -> end($candidate,")
        })
        .map(|rule| Edit {
            before: rule,
            after: String::new(),
        })
        .collect();
    if edits.len() != 3 {
        return Err(Error::new(
            "expected exact tuple, pair and case historical replay producers",
        ));
    }
    result.push(paired(
        "historical-finality-prevents-same-key-replay",
        stay,
        &s,
        records::fixture(cards, stay, &s) + &records::fixture(cards, history, &h),
        edits,
    ));
    Ok(result)
}
