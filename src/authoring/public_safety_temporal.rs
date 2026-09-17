// SPDX-License-Identifier: MIT OR Apache-2.0

//! Positive, exact-record temporal routes. No clock or absence-to-end rule.

use super::{contracts::Card, records};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct Variant {
    pub card: Card,
    pub authorization: &'static str,
    pub review: &'static str,
    pub window: &'static str,
    pub attack: Option<&'static str>,
    pub extra: Vec<String>,
    pub aliases: Vec<(String, String)>,
}

pub(super) fn is_measure(id: &str) -> bool {
    matches!(
        id,
        "acceleration"
            | "resource-redirection"
            | "requisition"
            | "hazard-restriction"
            | "hazard-evacuation"
    )
}

fn permits_alternate(id: &str) -> bool {
    matches!(id, "declaration" | "renewal" | "immediate-defence") || is_measure(id)
}

fn route_id(mode: &str, review: bool) -> Option<&'static str> {
    match (mode, review) {
        ("PSPendingAlternateRoute", false) => Some("alternate-authorization"),
        ("PSRatifiedAlternateRoute", false) => Some("alternate-ratification"),
        ("PSPendingSubstituteRoute", true) => Some("substitute-review"),
        ("PSRatifiedSubstituteRoute", true) => Some("substitute-ratification"),
        _ => None,
    }
}

impl Variant {
    fn alias(&mut self, left: String, right: String) {
        self.extra.push(format!("{left} = {right}"));
        self.aliases.push((left, right));
    }

    pub(super) fn label(&self) -> String {
        let mut label = format!("{}-{}-{}", self.authorization, self.review, self.window);
        if let Some(mode) = self.attack {
            label.push('-');
            label.push_str(mode);
        }
        label
    }

    pub(super) fn values(&self, cards: &[Card], prefix: &str) -> BTreeMap<String, String> {
        let mut values = records::values(cards, &self.card, prefix);
        values.insert("$authorization_mode".into(), self.authorization.into());
        values.insert("$review_mode".into(), self.review.into());
        values.insert("$window_mode".into(), self.window.into());
        if let Some(mode) = self.attack {
            values.insert("$attack_mode".into(), mode.into());
        }
        // Alias order is intentional: target/ordinary bindings precede the
        // selected acting authorizer and reviewer replacements.
        for (left, right) in &self.aliases {
            let value = values[right].clone();
            values.insert(left.clone(), value);
        }
        values
    }

    pub(super) fn premises(&self, cards: &[Card]) -> Vec<String> {
        let mut atoms = records::premises(cards, &self.card);
        atoms.extend([
            format!("$authorization_mode = {}", self.authorization),
            format!("$review_mode = {}", self.review),
            format!("$window_mode = {}", self.window),
        ]);
        atoms.extend(self.extra.clone());
        atoms
    }
}

pub(super) fn variants(card: &Card) -> Vec<Variant> {
    let authorizations: &[&str] = if card.id == "immediate-defence" {
        // This exception is only pre-opportunity. Actual Assembly ratification
        // afterwards uses the ordinary force-abroad contract, not a carry of
        // an immediate response whose exception has ended.
        &["PSOrdinaryRoute", "PSPendingAlternateRoute"]
    } else if permits_alternate(card.id) {
        &[
            "PSOrdinaryRoute",
            "PSPendingAlternateRoute",
            "PSRatifiedAlternateRoute",
        ]
    } else {
        &["PSOrdinaryRoute"]
    };
    let reviews: &[&str] = if card.id == "immediate-defence" {
        &["PSOrdinaryRoute", "PSPendingSubstituteRoute"]
    } else if permits_alternate(card.id) {
        &[
            "PSOrdinaryRoute",
            "PSPendingSubstituteRoute",
            "PSRatifiedSubstituteRoute",
        ]
    } else {
        &["PSOrdinaryRoute"]
    };
    let windows: &[&str] = if is_measure(card.id) {
        &["PSInitialDeclarationWindow", "PSRenewedDeclarationWindow"]
    } else {
        &["PSOwnCurrentWindow"]
    };
    let mut result = Vec::new();
    for authorization in authorizations {
        for review in reviews {
            for window in windows {
                let mut variant = Variant {
                    card: card.clone(),
                    authorization,
                    review,
                    window,
                    attack: None,
                    extra: vec![],
                    aliases: vec![],
                };
                if *window == "PSRenewedDeclarationWindow" {
                    variant.card.dependencies.push("renewal");
                }
                for (mode, reviewing) in [(*authorization, false), (*review, true)] {
                    let acting = if reviewing { "$review" } else { "$authorizer" };
                    let ordinary = if reviewing {
                        "$ordinary_reviewer"
                    } else {
                        "$ordinary_authorizer"
                    };
                    if let Some(id) = route_id(mode, reviewing) {
                        variant.card.dependencies.push(id);
                        for (parent, child) in [
                            ("$target", "$record"),
                            ("$target_revision", "$revision"),
                            ("$target_kind", "$kind"),
                            ("$ordinary", ordinary),
                        ] {
                            variant.alias(records::dependency_variable(id, parent), child.into());
                        }
                        variant.alias(
                            acting.into(),
                            records::dependency_variable(id, "$substitute"),
                        );
                    } else {
                        variant.alias(ordinary.into(), acting.into());
                    }
                }
                if is_measure(card.id) {
                    // A positive, independently witnessed containment finding
                    // binds this measure's own interval to the selected exact
                    // declaration/renewal interval. Opaque times are not clocks.
                    let id = if *window == "PSRenewedDeclarationWindow" {
                        "renewal"
                    } else {
                        "declaration"
                    };
                    for (variable, scope) in [
                        ("$window", "SelectedEnvelopeWindow"),
                        ("$start", "SelectedEnvelopeStart"),
                        ("$end", "SelectedEnvelopeEnd"),
                    ] {
                        for (actor, _) in records::ATTESTERS {
                            variant.extra.push(records::observe(
                                actor,
                                "$record",
                                &records::dependency_variable(id, variable),
                                scope,
                            ));
                        }
                    }
                    for (actor, _) in records::ATTESTERS {
                        variant.extra.push(records::observe(
                            actor,
                            "$record",
                            "OwnIntervalWithinExactSelectedEnvelopeAndScope",
                            "EnvelopeContainment",
                        ));
                    }
                }
                result.push(variant);
            }
        }
    }
    if let Some(mode) = default_attack_mode(card.id) {
        let mut attacks = Vec::new();
        for variant in result {
            let mut kinetic = variant.clone();
            kinetic.attack = Some(mode);
            kinetic.extra.push(format!("$attack_mode = {mode}"));
            attacks.push(kinetic);
            let mut cyber = variant;
            cyber.attack = Some("PSIndependentlyEvidencedCyberAttack");
            cyber
                .extra
                .push("$attack_mode = PSIndependentlyEvidencedCyberAttack".into());
            cyber.card.dependencies.push("cyber-attack-basis");
            attacks.push(cyber);
        }
        attacks
    } else {
        result
    }
}

pub(super) fn default_attack_mode(id: &str) -> Option<&'static str> {
    match id {
        "force-abroad" => Some("PSKineticOrLawfulCollectiveSecurityGround"),
        "immediate-defence" => Some("PSActualKineticArmedAttack"),
        _ => None,
    }
}

pub(super) fn fixture(
    cards: &[Card],
    variant: &Variant,
    values: &BTreeMap<String, String>,
) -> String {
    let mut fixture = records::fixture(cards, &variant.card, values);
    for atom in &variant.extra {
        if atom.starts_with("observe(") {
            fixture.push_str(&format!("{}.\n", records::ground(atom, values)));
        }
    }
    fixture
}

pub(super) fn scenarios(cards: &[Card]) -> Vec<super::cases::Scenario> {
    use super::cases::{Scenario, query};
    let mut cases = Vec::new();
    for id in ["declaration", "requisition"] {
        let card = cards.iter().find(|card| card.id == id).unwrap();
        for variant in variants(card) {
            let values = variant.values(cards, "PSTemporal");
            let facts = fixture(cards, &variant, &values);
            let atom = records::ground(
                &format!("complete($record, {}, $subject)", card.kind),
                &values,
            );
            let name = format!("temporal/{id}/{}", variant.label());
            for evaluation in ["positive", "frozen-record-fresh-evaluation"] {
                cases.push(Scenario::new(
                    format!("{name}/{evaluation}"),
                    facts.clone(),
                    query(&atom, true),
                ));
            }
            for (mode, review) in [(variant.authorization, false), (variant.review, true)] {
                if let Some(dep) = route_id(mode, review) {
                    let record = &values[&records::dependency_variable(dep, "$record")];
                    let scope = if dep.ends_with("ratification") {
                        "PSRatificationScope"
                    } else {
                        "PSOpportunityStatusScope"
                    };
                    let absent = facts
                        .lines()
                        .filter(|line| {
                            !(line.contains(&format!(", {record},"))
                                && line.ends_with(&format!(", {scope}).")))
                        })
                        .map(|line| format!("{line}\n"))
                        .collect::<String>();
                    assert!(absent != facts);
                    cases.push(Scenario::new(
                        format!("{name}/{dep}/missing-opportunity-or-ratification"),
                        absent,
                        query(&atom, false),
                    ));
                    let wrong = facts
                        .lines()
                        .map(|line| {
                            if line.contains(&format!(", {record},"))
                                && line.ends_with(", PSTargetRevisionScope).")
                            {
                                line.replace(&values["$revision"], "DifferentRevision") + "\n"
                            } else {
                                format!("{line}\n")
                            }
                        })
                        .collect::<String>();
                    assert!(wrong != facts);
                    cases.push(Scenario::new(
                        format!("{name}/{dep}/wrong-target-revision"),
                        wrong,
                        query(&atom, false),
                    ));
                    if !dep.ends_with("ratification") {
                        let expired = facts
                            .lines()
                            .map(|line| {
                                if line.contains(&format!(", {record},")) {
                                    line.replace(
                                        "NoFirstOpportunityYetPositivelyAttested",
                                        "FirstOpportunityPassedWithoutRatification",
                                    ) + "\n"
                                } else {
                                    format!("{line}\n")
                                }
                            })
                            .collect::<String>();
                        assert!(expired != facts);
                        cases.push(Scenario::new(
                            format!("{name}/{dep}/unratified-first-opportunity"),
                            expired,
                            query(&atom, false),
                        ));
                    }
                }
            }
            if variant.window == "PSRenewedDeclarationWindow" {
                let renewal = &values[&records::dependency_variable("renewal", "$record")];
                let absent = facts
                    .lines()
                    .filter(|line| {
                        !(line.contains(&format!(", {renewal},"))
                            && line.ends_with(", PSCoreCurrentDispositionScope)."))
                    })
                    .map(|line| format!("{line}\n"))
                    .collect::<String>();
                assert!(absent != facts);
                cases.push(Scenario::new(
                    format!("{name}/renewal-currentness-missing"),
                    absent,
                    query(&atom, false),
                ));
                let source = &values[&records::dependency_variable("renewal", "$source")];
                let conflicted = format!(
                    "{facts}observe({source}, {renewal}, WrongDeclarationRevision, PSDependencyDeclarationRevisionScope).\n"
                );
                cases.push(Scenario::new(
                    format!("{name}/renewal-borrows-another-declaration"),
                    conflicted,
                    query(&atom, false),
                ));
            }
        }
    }
    cases
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::Context,
        pin::{LoadedSource, PinOptions, PreparedPinEngine},
    };

    #[test]
    fn exact_temporal_routes_and_frozen_record_limit() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = super::super::cards(&context).unwrap();
                let candidate = super::super::render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("temporal candidate", &candidate)]);
                for case in scenarios(&cards) {
                    let out = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &case.pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(out.exit_code, 0, "{}: {out:?}", case.id);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
