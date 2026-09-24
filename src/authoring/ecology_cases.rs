// SPDX-License-Identifier: MIT OR Apache-2.0

//! Independent substantive records, not a reduced verification inventory.

use super::{Card, Values, records};

pub(super) struct Scenario {
    pub id: String,
    pub facts: String,
    pub pins: String,
}

pub(super) fn queries(card: &Card, values: &Values, expected: bool) -> String {
    records::heads(card)
        .iter()
        .map(|head| super::query(&records::ground(head, values), expected))
        .collect()
}

pub(super) fn conclusion(card: &Card, values: &Values, expected: bool) -> String {
    super::query(&records::ground(&records::heads(card)[0], values), expected)
}

#[derive(Default)]
pub(super) struct Stages {
    facts: std::collections::BTreeSet<String>,
    withheld: std::collections::BTreeSet<String>,
}

impl Stages {
    pub(super) fn add(&mut self, cards: &[Card], card: &Card, values: &Values) -> String {
        self.facts.extend(
            records::fixture(cards, card, values)
                .lines()
                .map(str::to_owned),
        );
        let mut activation = vec![records::ground(
            "authorized($review, ECIndependentReviewAuthority, $record).",
            values,
        )];
        if values.contains_key("$registry") {
            activation.push(records::ground(
                "authorized($registry_review, ECReplayReviewAuthority, $registry).",
                values,
            ));
        }
        for fact in &activation {
            assert!(
                self.facts.contains(fact),
                "stage must withhold an actual required credential"
            );
            self.withheld.insert(fact.clone());
        }
        activation
            .into_iter()
            .map(|fact| format!(":accept\n{fact}\n"))
            .collect()
    }

    pub(super) fn finish(self) -> String {
        self.facts
            .difference(&self.withheld)
            .map(|fact| format!("{fact}\n"))
            .collect()
    }
}

pub(super) fn join_parent(
    cards: &[Card],
    consumer: &Card,
    key: &str,
    values: &mut Values,
    parent_values: &Values,
) {
    let dependency = consumer
        .dependencies
        .iter()
        .find(|dependency| dependency.key == key)
        .unwrap();
    let parent = super::card(cards, dependency.id);
    for (variable, value) in parent_values {
        let target = records::dependency_variable(consumer, parent, dependency, variable);
        if target.starts_with('$') {
            values.insert(target, value.clone());
        } else {
            assert_eq!(&target, value, "constant parent binding");
        }
    }
}

pub(super) fn guardian_sequences(cards: &[Card]) -> Vec<Scenario> {
    let execution = super::card(cards, "high-consequence");
    let stay = super::card(cards, "guardian-stay");
    let merits = super::card(cards, "guardian-merits");
    let reversible = super::card(cards, "reversible-continuity");
    let h = records::values(cards, execution, "ECStayExecution");
    let mut g = records::values(cards, stay, "ECStayGuardian");
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
        "$evidence_version",
    ] {
        g.insert(variable.into(), h[variable].clone());
    }
    let mut m = records::values(cards, merits, "ECStayMerits");
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
        "$evidence_version",
        "$ground",
    ] {
        m.insert(variable.into(), g[variable].clone());
    }
    let mut r = records::values(cards, reversible, "ECStayContinuity");
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
    ] {
        r.insert(variable.into(), h[variable].clone());
    }
    let initial = records::fixture(cards, execution, &h);
    let mut expiry_stages = Stages::default();
    let activate_guardian = expiry_stages.add(cards, stay, &g);
    let activate_continuity = expiry_stages.add(cards, reversible, &r);
    let mut pins = queries(execution, &h, true);
    // A real scoped assertion is retracted, not emulated by omitting it.
    let ended = records::ground(
        "observe($review, $record, ECEndedGuardianWindow, ECCurrentDispositionScope).\n",
        &g,
    );
    pins += &format!(":accept-scoped\n{ended}");
    pins += &activate_guardian;
    pins += &queries(stay, &g, true);
    pins += &conclusion(execution, &h, false);
    pins += &super::query(
        &format!(
            "complete({}, ECHighConsequenceAuthorizationBasis, {})",
            h["$authorization_record"], h["$subject"]
        ),
        true,
    );
    pins += &activate_continuity;
    pins += &queries(reversible, &r, true);
    pins += &ended;
    pins += &conclusion(stay, &g, false);
    pins += &queries(execution, &h, true);
    pins += &super::query(
        "permits(ECUnapprovedActor, ECExactHighConsequenceActivity, ECUnapprovedActivity)",
        false,
    );
    let expiry = Scenario {
        id: "guardian/stay-expiry-and-independent-continuity".into(),
        facts: initial.clone() + &expiry_stages.finish(),
        pins,
    };

    let mut stages = Stages::default();
    let mut pins = stages.add(cards, stay, &g) + &queries(stay, &g, true);
    pins += &stages.add(cards, merits, &m);
    pins += &queries(merits, &m, true);
    pins += &conclusion(stay, &g, false);
    for (index, mode) in [
        "same-key",
        "changed-ground",
        "changed-window",
        "new-authorization",
        "new-evidence",
    ]
    .iter()
    .enumerate()
    {
        let mut next = records::values(cards, stay, &format!("ECReplay{index}"));
        for variable in [
            "$subject",
            "$case",
            "$project",
            "$authorization_record",
            "$authorization_version",
            "$evidence_version",
            "$ground",
            "$window",
            "$start",
            "$end",
        ] {
            next.insert(variable.into(), g[variable].clone());
        }
        match *mode {
            "changed-ground" => {
                next.insert("$ground".into(), "ECSeriousHarmGround".into());
            }
            "changed-window" => {
                next.insert("$window".into(), "ECIllicitRenewalWindow".into());
            }
            "new-authorization" => {
                next.insert(
                    "$authorization_version".into(),
                    "ECMateriallyChangedAuthorization".into(),
                );
            }
            "new-evidence" => {
                next.insert("$evidence_version".into(), "ECMateriallyNewEvidence".into());
            }
            _ => {}
        }
        pins += &format!("# {mode}: no office-local replay identity.\n");
        pins += &stages.add(cards, stay, &next);
        pins += &conclusion(stay, &next, false);
    }
    for (index, (id, changed)) in [
        (
            "guardian-material-authorization-stay",
            "$authorization_version",
        ),
        ("guardian-material-evidence-stay", "$evidence_version"),
    ]
    .into_iter()
    .enumerate()
    {
        let renewal = super::card(cards, id);
        let mut next = records::values(cards, renewal, &format!("ECMaterialRenewal{index}"));
        join_parent(cards, renewal, "guardian-merits", &mut next, &m);
        for variable in [
            "$case",
            "$project",
            "$authorization_record",
            "$authorization_version",
            "$evidence_version",
        ] {
            next.insert(variable.into(), g[variable].clone());
        }
        next.insert(changed.into(), format!("ECSubstantivelyNewVersion{index}"));
        if changed == "$authorization_version" {
            next.insert(
                "$authorization_record".into(),
                "ECSubstantivelyNewAuthorizationRecord".into(),
            );
        }
        pins += &stages.add(cards, renewal, &next);
        pins += &queries(renewal, &next, true);
    }
    vec![
        expiry,
        Scenario {
            id: "guardian/finality-and-material-new-pair".into(),
            facts: initial + &stages.finish(),
            pins,
        },
    ]
}

fn observations(cards: &[Card], card: &Card) -> Vec<String> {
    records::base_premises(cards, card)
        .into_iter()
        .filter(|atom| atom.starts_with("observe(") && atom.split(", ").nth(1) == Some("$record"))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn copy_key(to: &mut Values, from: &Values) {
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
        "$evidence_version",
        "$ground",
    ] {
        if let Some(value) = from.get(variable) {
            to.insert(variable.into(), value.clone());
        }
    }
}

pub(super) fn guardian_boundaries(cards: &[Card]) -> Vec<Scenario> {
    let stay = super::card(cards, "guardian-stay");
    let execution = super::card(cards, "high-consequence");
    let g = records::values(cards, stay, "ECCertificate");
    let mut facts: String = records::base_premises(cards, stay)
        .iter()
        .filter(|atom| atom.starts_with("observe(") || atom.starts_with("authorized("))
        .map(|atom| format!("{}.\n", records::ground(atom, &g)))
        .collect();
    facts += &records::ground(
        "authorized(ECCertificateIssuer, Certifier, $record).\nobserve(ECCertificateIssuer, ECGuardianAutomaticStay, $record, CertificationScope).\n",
        &g,
    );
    let mut result = vec![Scenario {
        id: "guardian/qualification-certificate-is-not-a-stay".into(),
        facts,
        pins: super::query(
            &records::ground(
                "grant(ECCertificateIssuer, ECGuardianAutomaticStay, $record)",
                &g,
            ),
            true,
        ) + &conclusion(stay, &g, false)
            + &super::query(
                &records::ground(
                    "oppose($authorization_record, $authorization_version, ECGuardianAutomaticStay)",
                    &g,
                ),
                false,
            ),
    }];
    for id in [
        "present-person-commons-claim",
        "association-commons-claim",
        "rights-advocate-commons-claim",
        "guardian-commons-claim",
    ] {
        let claim = super::card(cards, id);
        let h = records::values(cards, execution, "ECOrdinaryRequestExecution");
        let mut c = records::values(cards, claim, "ECOrdinaryRequest");
        copy_key(&mut c, &h);
        result.push(Scenario {
            id: format!("guardian/{id}-does-not-impose-automatic-stay"),
            facts: records::fixture(cards, execution, &h) + &records::fixture(cards, claim, &c),
            pins: queries(claim, &c, true) + &queries(execution, &h, true)
                + &super::query(&records::ground("oppose($authorization_record, $authorization_version, ECGuardianAutomaticStay)", &h), false),
        });
    }
    for merits_id in ["guardian-merits", "substitute-guardian-merits"] {
        let merits = super::card(cards, merits_id);
        let h = records::values(cards, execution, "ECExactExecution");
        let mut other = records::values(cards, execution, "ECOtherExecution");
        for variable in [
            "$subject",
            "$case",
            "$project",
            "$authorization_version",
            "$evidence_version",
        ] {
            other.insert(variable.into(), h[variable].clone());
        }
        assert_ne!(h["$authorization_record"], other["$authorization_record"]);
        let mut g = records::values(cards, stay, "ECExactStay");
        copy_key(&mut g, &h);
        let mut m = records::values(cards, merits, "ECExactMerits");
        copy_key(&mut m, &g);
        m.insert("$outcome".into(), "ECMeritsRestrictChallengedAct".into());
        let mut stages = Stages::default();
        let mut pins = stages.add(cards, stay, &g)
            + &queries(stay, &g, true)
            + &conclusion(execution, &h, false)
            + &queries(execution, &other, true);
        pins += &stages.add(cards, merits, &m);
        pins += &queries(merits, &m, true);
        pins += &conclusion(stay, &g, false);
        pins += &super::query(
            &records::ground(
                "oppose($authorization_record, $authorization_version, ECFinalJudicialProtection)",
                &h,
            ),
            true,
        );
        pins += &records::ground(
            "observe($review, $record, ECEndedGuardianWindow, ECCurrentDispositionScope).\n",
            &g,
        );
        pins += &conclusion(execution, &h, false);
        pins += &queries(execution, &other, true);
        for next_id in ["guardian-stay", "alternate-guardian-stay"] {
            let next_card = super::card(cards, next_id);
            let mut next = records::values(
                cards,
                next_card,
                &format!("ECNext{}", next_id.replace('-', "")),
            );
            copy_key(&mut next, &g);
            pins += &stages.add(cards, next_card, &next);
            pins += &conclusion(next_card, &next, false);
        }
        result.push(Scenario {
            id: format!(
                "guardian/{merits_id}-exact-record-final-restriction-and-cross-office-replay"
            ),
            facts: records::fixture(cards, execution, &h)
                + &records::fixture(cards, execution, &other)
                + &stages.finish(),
            pins,
        });
    }
    result
}

pub(super) fn common(cards: &[Card]) -> Vec<Scenario> {
    cards
        .iter()
        .flat_map(|card| common_for(cards, card))
        .collect()
}

pub(super) fn common_for(cards: &[Card], card: &Card) -> Vec<Scenario> {
    let mut cases = Vec::new();
    let values = records::values(cards, card, "ECPositive");
    let facts = records::fixture(cards, card, &values);
    cases.push(Scenario {
        id: format!("{}/positive", card.id),
        facts: facts.clone(),
        pins: queries(card, &values, true),
    });
    cases.push(Scenario {
        id: format!("{}/withheld", card.id),
        facts: String::new(),
        pins: queries(card, &values, false),
    });

    // Independent subjects share a test case only to amortize preparation.
    // Every writer/reader field is still withheld separately and queried.
    for (batch, fields) in observations(cards, card).chunks(8).enumerate() {
        let mut facts = String::new();
        let mut pins = String::new();
        for (index, observation) in fields.iter().enumerate() {
            let mut values = records::values(cards, card, "ECOmitShared");
            // Cases are isolated. Stable identities let unchanged evidence be
            // shared on disk and prepared once without borrowing any verdict.
            values.insert("$record".into(), format!("ECOmitF{index}Record"));
            let atom = format!("{}.\n", records::ground(observation, &values));
            let fixture = records::fixture(cards, card, &values);
            assert_eq!(fixture.matches(&atom).count(), 1, "{}: {atom}", card.id);
            facts += &fixture.replacen(&atom, "", 1);
            pins += &format!("# Withhold one required observation: {observation}.\n");
            pins += &conclusion(card, &values, false);
        }
        cases.push(Scenario {
            id: format!("{}/required-fields-{batch}", card.id),
            facts: deduplicate(&facts),
            pins,
        });
    }

    let mut facts = String::new();
    let mut pins = String::new();
    let credentials: std::collections::BTreeSet<_> = records::base_premises(cards, card)
        .into_iter()
        .filter(|atom| atom.starts_with("authorized(") && atom.ends_with(", $record)"))
        .collect();
    for (index, atom) in credentials.iter().enumerate() {
        let mut values = records::values(cards, card, "ECUnauthShared");
        values.insert("$record".into(), format!("ECUnauth{index}Record"));
        let credential = records::ground(&format!("{atom}.\n"), &values);
        let fixture = records::fixture(cards, card, &values);
        assert_eq!(fixture.matches(&credential).count(), 1);
        facts += &fixture.replacen(&credential, "", 1);
        pins += &conclusion(card, &values, false);
    }
    cases.push(Scenario {
        id: format!("{}/authorized-roles", card.id),
        facts: deduplicate(&facts),
        pins,
    });

    let independent =
        regex::Regex::new(r"^~\((\$[a-z][a-z0-9_]*) = (\$[a-z][a-z0-9_]*)\)$").unwrap();
    let pairs: Vec<_> = records::base_premises(cards, card)
        .iter()
        .filter_map(|atom| {
            independent
                .captures(atom)
                .map(|m| (m[1].to_owned(), m[2].to_owned()))
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for (batch, pairs) in pairs.chunks(4).enumerate() {
        let mut facts = String::new();
        let mut pins = String::new();
        for (index, (actor, other)) in pairs.iter().enumerate() {
            let mut values = records::values(cards, card, &format!("ECFusedR{index}"));
            values.insert(actor.clone(), values[other].clone());
            facts += &records::fixture(cards, card, &values);
            pins += &format!("# Fuse {actor} with {other}.\n");
            pins += &conclusion(card, &values, false);
        }
        cases.push(Scenario {
            id: format!("{}/independence-{batch}", card.id),
            facts,
            pins,
        });
    }

    for (batch, fields) in observations(cards, card).chunks(4).enumerate() {
        let mut facts = String::new();
        let mut pins = String::new();
        for (index, observation) in fields.iter().enumerate() {
            let mut values = records::values(cards, card, "ECConflictShared");
            values.insert("$record".into(), format!("ECConflictF{index}Record"));
            let original = observation.clone();
            let parts = original
                .strip_prefix("observe(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
                .split(", ")
                .collect::<Vec<_>>();
            assert_eq!(parts.len(), 4);
            let conflict = format!(
                "observe({}, {}, ECMismatchedValue, {})",
                parts[0], parts[1], parts[3]
            );
            let original = format!("{}.\n", records::ground(&original, &values));
            let conflict = format!("{}.\n", records::ground(&conflict, &values));
            let fixture = records::fixture(cards, card, &values);
            assert_eq!(fixture.matches(&original).count(), 1);
            facts += &fixture.replacen(&original, &conflict, 1);
            pins += &conclusion(card, &values, false);
            pins += &super::query(
                &format!("related({}, ECRecordAmbiguity)", values["$record"]),
                true,
            );
        }
        cases.push(Scenario {
            id: format!("{}/conflicting-fields-{batch}", card.id),
            facts: deduplicate(&facts),
            pins,
        });
    }

    let values = records::values(cards, card, "ECNoise");
    let mut facts = records::fixture(cards, card, &values);
    facts += &format!(
        "observe(ECUncredentialedNoise, {}, ECWrongRevision, ECRevisionScope).\n",
        values["$record"]
    );
    let mut pins = queries(card, &values, true);
    pins += &super::query(
        &format!("related({}, ECRecordAmbiguity)", values["$record"]),
        false,
    );
    cases.push(Scenario {
        id: format!("{}/uncredentialed-noise", card.id),
        facts,
        pins,
    });
    cases
}

/// Ruling D6: a beneficial ecological record the source alone attests takes
/// effect at once, the completed record does not derive, and the named
/// reviewer's withdrawal on review switches the effect off.
pub(super) fn single_actor(
    cards: &[Card],
    beneficial: &std::collections::BTreeSet<String>,
) -> Vec<Scenario> {
    use super::super::procedural_load::{fast_head, PROMPT_REVIEW, REVIEW_SCOPE, WITHDRAWN};
    let mut cases = Vec::new();
    for card in cards.iter().filter(|c| beneficial.contains(c.kind)) {
        let values = records::values(cards, card, "ECSingleActor");
        let review = format!("observe({}, {},", values["$review"], values["$record"]);
        let facts: String = records::fixture(cards, card, &values)
            .lines()
            .filter(|line| !line.starts_with(&review))
            .map(|line| format!("{line}\n"))
            .collect();
        let heads = records::heads(card);
        let duty = records::ground(&format!("obliged($review, {PROMPT_REVIEW}, $record)"), &values);
        let effects = |holds: bool| -> String {
            let mut pins = String::new();
            for head in heads.iter().skip(1).filter(|h| fast_head(h)) {
                pins += &super::query(&records::ground(head, &values), holds);
            }
            pins + &super::query(&duty, holds)
        };
        let mut pins = conclusion(card, &values, false);
        pins += &effects(true);
        pins += &format!(
            "observe({}, {}, {WITHDRAWN}, {REVIEW_SCOPE}).\n",
            values["$review"], values["$record"]
        );
        pins += &effects(false);
        cases.push(Scenario {
            id: format!("{}/single-actor-and-withdrawal-on-review", card.id),
            facts,
            pins,
        });
    }
    cases
}

fn deduplicate(facts: &str) -> String {
    let mut seen = std::collections::BTreeSet::new();
    facts
        .lines()
        .filter(|line| seen.insert(*line))
        .map(|line| format!("{line}\n"))
        .collect()
}
