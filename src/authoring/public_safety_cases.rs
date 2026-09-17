// SPDX-License-Identifier: MIT OR Apache-2.0

//! Substantive scenarios shared by authoring export and development tests.

pub(super) struct Scenario {
    pub id: String,
    pub facts: String,
    pub pins: String,
}

impl Scenario {
    pub fn new(id: impl Into<String>, facts: String, steps: String) -> Self {
        let count = steps.lines().filter(|line| line.starts_with("? ")).count();
        assert!(count > 0);
        Self {
            id: id.into(),
            facts,
            pins: format!(":expect-pins {count}\n{steps}"),
        }
    }
}

pub(super) fn query(atom: &str, expected: bool) -> String {
    format!(
        "? {atom}.\n# => {}\n",
        if expected { "TRUE" } else { "FALSE" }
    )
}

pub(super) fn core(cards: &[super::contracts::Card]) -> Vec<Scenario> {
    use super::{records, temporal};
    let mut cases = Vec::new();
    for card in cards {
        let values = records::values(cards, card, "PSCore");
        let facts = records::fixture(cards, card, &values);
        let complete = records::ground(
            &format!("complete($record, {}, $subject)", card.kind),
            &values,
        );
        let mut positive = query(&complete, true);
        for duty in &card.duties {
            positive += &query(
                &records::ground(&format!("obliged($operator, {duty}, $record)"), &values),
                true,
            );
        }
        if let Some(capability) = card.capability {
            let identification = records::human_subject_premises(card)
                .iter()
                .filter(|atom| atom.starts_with("observe(") || atom.starts_with("authorized("))
                .map(|atom| format!("{}.\n", records::ground(atom, &values)))
                .collect::<String>();
            let identified = query(&records::ground("person($subject)", &values), true)
                + &query(
                    &records::ground("owe(State, Eats, $subject)", &values),
                    true,
                )
                + &query(&complete, false)
                + &query(&records::ground("prisoner($subject)", &values), false)
                + &query(&records::ground("capture($subject)", &values), false);
            cases.push(Scenario::new(
                format!("core/{}/identified-human-without-order", card.id),
                identification.clone(),
                identified,
            ));
            if card.id == "arrest" {
                let uncredentialed = identification
                    .lines()
                    .filter(|line| !line.contains(", PSEvidenceAuthority,"))
                    .map(|line| format!("{line}\n"))
                    .collect::<String>();
                assert!(uncredentialed != identification);
                cases.push(Scenario::new(
                    "core/arrest/uncredentialed-human-identification",
                    uncredentialed,
                    query(&records::ground("person($subject)", &values), false),
                ));
            }
            for atom in [
                format!("restrain($subject, $record, {capability})"),
                format!("lose({capability}, $subject, $record)"),
                "person($subject)".into(),
            ] {
                positive += &query(&records::ground(&atom, &values), true);
            }
            positive += &query(
                &records::ground("travel($subject)", &values),
                !card.movement,
            );
            for atom in [
                "prisoner($subject)",
                "severe($subject)",
                "capture($subject)",
                "reward($subject)",
                "free($subject)",
            ] {
                positive += &query(&records::ground(atom, &values), false);
            }
            if card.holding {
                // Legal permission alone does not prove physical holding or
                // recipient-side delivery. Test this before any holding report.
                positive += &query(&records::ground("dwell($subject)", &values), false);
                positive += &query(&records::ground("expresses($subject)", &values), false);
            }
        }
        cases.push(Scenario::new(
            format!("core/{}/positive", card.id),
            facts.clone(),
            positive,
        ));
        if card
            .fields
            .iter()
            .any(|(holder, scope)| *scope == "MandateHolder" && !holder.starts_with('$'))
        {
            let mut wrong_holder = values.clone();
            wrong_holder.insert("$holder".into(), "SelfAppointedProtectiveCourt".into());
            cases.push(Scenario::new(
                format!("core/{}/wrong-fixed-holder", card.id),
                records::fixture(cards, card, &wrong_holder),
                query(&complete, false),
            ));
        }
        for (actor, role) in records::ATTESTERS.iter().chain(records::READERS) {
            let absent =
                records::ground(&format!("authorized({actor}, {role}, $record)."), &values);
            let missing = facts
                .lines()
                .filter(|line| *line != absent)
                .map(|line| format!("{line}\n"))
                .collect::<String>();
            assert!(missing != facts, "{} {role}", card.id);
            cases.push(Scenario::new(
                format!("core/{}/without-{role}", card.id),
                missing,
                query(&complete, false),
            ));
        }
        // Common fields share one authoring template; exercise all of them on
        // each distinct kind of producer, not a schema or freshness audit.
        let mut scopes = card
            .fields
            .iter()
            .map(|(_, scope)| *scope)
            .collect::<Vec<_>>();
        if matches!(
            card.id,
            "policing" | "arrest" | "declaration" | "reviewed-defect"
        ) {
            scopes.extend(records::common().iter().map(|(_, scope)| *scope));
        }
        scopes.sort();
        scopes.dedup();
        for scope in scopes {
            let missing = facts
                .lines()
                .filter(|line| {
                    !(line.starts_with(&format!(
                        "observe({}, {},",
                        values["$source"], values["$record"]
                    )) && line.ends_with(&format!(", PS{scope}Scope).")))
                })
                .map(|line| format!("{line}\n"))
                .collect::<String>();
            assert!(missing != facts, "{} {scope}", card.id);
            cases.push(Scenario::new(
                format!("core/{}/without-{scope}", card.id),
                missing,
                query(&complete, false),
            ));
        }
        let conflict = records::ground(
            "observe($source, $record, WrongRevision, PSRecordRevisionScope).\n",
            &values,
        );
        cases.push(Scenario::new(
            format!("core/{}/conflicting-current-record", card.id),
            facts.clone(),
            query(&complete, true) + &conflict + &query(&complete, false),
        ));
        let noise = records::ground(
            "observe(UncredentialedWriter, $record, WrongRevision, PSRecordRevisionScope).\n",
            &values,
        );
        cases.push(Scenario::new(
            format!("core/{}/unauthorized-noise", card.id),
            facts.clone() + &noise,
            query(&complete, true),
        ));
        let stale = facts
            .lines()
            .map(|line| {
                if line.contains(&format!(", {},", values["$record"]))
                    && line.ends_with(", PSCoreCurrentDispositionScope).")
                {
                    line.replace("CurrentReconciledAuthority", "SupersededAuthority") + "\n"
                } else {
                    format!("{line}\n")
                }
            })
            .collect::<String>();
        assert!(stale != facts);
        cases.push(Scenario::new(
            format!("core/{}/stale", card.id),
            stale,
            query(&complete, false),
        ));
        for variant in temporal::variants(card) {
            if variant.authorization == "PSOrdinaryRoute"
                && variant.review == "PSOrdinaryRoute"
                && variant.window != "PSRenewedDeclarationWindow"
            {
                continue;
            }
            let values = variant.values(cards, "PSCurrentVariant");
            let facts = temporal::fixture(cards, &variant, &values);
            let complete = records::ground(
                &format!("complete($record, {}, $subject)", card.kind),
                &values,
            );
            for evaluation in ["positive", "frozen-record-fresh-evaluation"] {
                cases.push(Scenario::new(
                    format!("core/{}/{}/{evaluation}", card.id, variant.label()),
                    facts.clone(),
                    query(&complete, true),
                ));
            }
            for id in variant.card.dependencies.iter().filter(|id| {
                matches!(
                    **id,
                    "renewal"
                        | "alternate-authorization"
                        | "alternate-ratification"
                        | "substitute-review"
                        | "substitute-ratification"
                )
            }) {
                let source = &values[&records::dependency_variable(id, "$source")];
                let record = &values[&records::dependency_variable(id, "$record")];
                let missing = facts
                    .lines()
                    .filter(|line| {
                        *line != format!("authorized({source}, PSSourceAuthority, {record}).")
                    })
                    .map(|line| format!("{line}\n"))
                    .collect::<String>();
                assert!(missing != facts);
                cases.push(Scenario::new(
                    format!("core/{}/{}/without-{id}", card.id, variant.label()),
                    missing,
                    query(&complete, false),
                ));
            }
        }
    }
    cases
}

pub(super) fn economic_boundary(
    context: &crate::context::Context,
    cards: &[super::contracts::Card],
) -> Result<Vec<Scenario>, crate::cli::Error> {
    use super::records;
    let declaration = cards.iter().find(|card| card.id == "declaration").unwrap();
    let values = records::values(cards, declaration, "PSEconomicBoundary");
    let declared = records::fixture(cards, declaration, &values);
    let current = records::ground(
        "complete($record, PSNonDerogatingEmergencyDeclaration, $subject)",
        &values,
    );
    let mut cases = Vec::new();
    for power in ["081", "082"] {
        // Reuse the actual economic family's first positive witnesses. No
        // declaration substitutes for that family's independent currentness.
        let source = context.read(&format!("book-1/source/economic-power-{power}.pins.nibli"))?;
        let mut facts = String::new();
        for line in source.lines().take_while(|line| !line.starts_with("? ")) {
            if line.starts_with("authorized(") || line.starts_with("observe(") {
                facts.push_str(line);
                facts.push('\n');
            } else if !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with(":expect-pins ")
            {
                return Err(crate::cli::Error::new(
                    "unrecognized economic positive fixture statement",
                ));
            }
        }
        let authority = format!("authority(FSBOD_09, FSPOW_{power}, EconRecord{power}Live)");
        if facts.is_empty() || !source.contains(&format!("? {authority}.\n# => TRUE")) {
            return Err(crate::cli::Error::new(
                "economic positive control no longer recognized",
            ));
        }
        cases.push(Scenario::new(
            format!("economic/{power}/declaration-is-not-scarcity-authority"),
            declared.clone(),
            query(&current, true)
                + &query(&authority, false)
                + &facts
                + &query(&authority, true)
                + &query(&current, true),
        ));
        let without_review = facts
            .lines()
            .filter(|line| {
                !(line.starts_with("authorized(")
                    && line.contains(", EconomicTemporalReviewAuthority,"))
            })
            .map(|line| format!("{line}\n"))
            .collect::<String>();
        assert!(without_review != facts);
        cases.push(Scenario::new(
            format!("economic/{power}/declaration-cannot-replace-scarcity-currentness"),
            declared.clone() + &without_review,
            query(&current, true) + &query(&authority, false),
        ));
    }
    Ok(cases)
}

pub(super) fn firewalls(cards: &[super::contracts::Card]) -> Vec<Scenario> {
    let mut cases = Vec::new();
    for card in cards.iter().filter(|card| card.capability.is_some()) {
        let mut pins = String::new();
        for floor in super::contracts::FLOOR {
            pins += &format!(
                ":refuse reasoning /Unstratifiable/\nall $subject: person($subject) & ~{floor}($subject) -> restrain($subject, {}ForbiddenOrder, {}).\n",
                card.kind,
                card.capability.unwrap()
            );
        }
        cases.push(Scenario {
            id: format!("firewalls/{}", card.id),
            facts: String::new(),
            pins: format!(":expect-pins {}\n{pins}", super::contracts::FLOOR.len()),
        });
    }
    let mut pins = String::new();
    for floor in super::contracts::FLOOR {
        pins += &format!(
            ":refuse reasoning /Unstratifiable/\nall $subject: person($subject) & ~{floor}($subject) -> prisoner($subject).\n"
        );
    }
    cases.push(Scenario {
        id: "firewalls/existing-custody".into(),
        facts: String::new(),
        pins: format!(":expect-pins {}\n{pins}", super::contracts::FLOOR.len()),
    });
    let mut pins = String::new();
    for premise in ["~family($subject)", "~parent($parent, $subject)"] {
        let quantifier = if premise.contains("$parent") {
            "all $parent: "
        } else {
            ""
        };
        pins += &format!(
            ":refuse reasoning /Unstratifiable/\nall $subject: {quantifier}person($subject) & {premise} -> prisoner($subject).\n"
        );
    }
    cases.push(Scenario {
        id: "firewalls/relationships".into(),
        facts: String::new(),
        pins: format!(":expect-pins 2\n{pins}"),
    });
    cases
}

pub(super) fn composed_windows(cards: &[super::contracts::Card]) -> Vec<Scenario> {
    use super::{records, temporal};
    let mut cases = Vec::new();
    for id in ["force-abroad", "immediate-defence", "hazard-evacuation"] {
        let card = cards.iter().find(|card| card.id == id).unwrap();
        for variant in temporal::variants(card) {
            let values = variant.values(cards, "PSBoundedWindow");
            let facts = temporal::fixture(cards, &variant, &values);
            let atom = records::ground(
                &format!("complete($record, {}, $subject)", card.kind),
                &values,
            );
            cases.push(Scenario::new(
                format!("composed/{id}/{}/positive", variant.label()),
                facts.clone(),
                query(&atom, true),
            ));
            if id == "immediate-defence" {
                let passed = facts.replace(
                    "NoFirstOpportunityYetPositivelyAttested",
                    "FirstOpportunityPassedWithoutRatification",
                );
                assert!(passed != facts);
                cases.push(Scenario::new(
                    format!(
                        "composed/{id}/{}/unratified-after-first-opportunity",
                        variant.label()
                    ),
                    passed,
                    query(&atom, false)
                        + &query(&records::ground("free($subject)", &values), false),
                ));
            }
            let missing = if variant.attack == Some("PSIndependentlyEvidencedCyberAttack") {
                let source = &values["$cyber_attack_basis_source"];
                let record = &values["$cyber_attack_basis_record"];
                facts
                    .lines()
                    .filter(|line| {
                        *line != format!("authorized({source}, PSSourceAuthority, {record}).")
                    })
                    .map(|line| format!("{line}\n"))
                    .collect::<String>()
            } else if id == "hazard-evacuation" {
                let record = &values["$external_evacuation_record"];
                facts
                    .lines()
                    .filter(|line| {
                        !(line.contains(&format!(", {record},"))
                            && line.ends_with(", MPEmergencyDeclarationVersionScope)."))
                    })
                    .map(|line| format!("{line}\n"))
                    .collect::<String>()
            } else {
                continue;
            };
            assert!(missing != facts, "{id} {}", variant.label());
            cases.push(Scenario::new(
                format!("composed/{id}/{}/missing-actual-upstream", variant.label()),
                missing,
                query(&atom, false),
            ));
        }
    }
    cases
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};

    #[test]
    fn exact_external_consumers_reject_missing_mismatched_and_withdrawn_evidence() {
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
                let engine = PreparedPinEngine::new(&[LoadedSource::new(
                    "protective candidate",
                    &candidate,
                )]);
                for case in super::super::external::scenarios(&cards).unwrap() {
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

    #[test]
    fn reviewed_defects_stop_only_exact_reliance_and_have_actual_readers() {
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
                let engine = PreparedPinEngine::new(&[LoadedSource::new(
                    "protective candidate",
                    &candidate,
                )]);
                for case in super::super::review::scenarios(&cards) {
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

    #[test]
    fn cyber_and_collective_evacuation_rejoin_actual_authority_and_windows() {
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
                let engine = PreparedPinEngine::new(&[LoadedSource::new(
                    "protective candidate",
                    &candidate,
                )]);
                let mut cases = super::composed_windows(&cards);
                cases.extend(super::economic_boundary(&context, &cards).unwrap());
                for case in cases {
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

    #[test]
    fn standing_survives_withdrawal_without_an_additional_presence_entry() {
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
                let engine = PreparedPinEngine::new(&[LoadedSource::new(
                    "protective candidate",
                    &candidate,
                )]);
                for case in super::super::review::scenarios(&cards)
                    .into_iter()
                    .filter(|case| case.id.ends_with("/standing-without-separate-entry"))
                {
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

    #[test]
    fn every_direct_producer_and_movement_blocker_works_without_inventing_holding() {
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
                let engine = PreparedPinEngine::new(&[LoadedSource::new(
                    "protective candidate",
                    &candidate,
                )]);
                for case in super::core(&cards).into_iter().filter(|case| {
                    case.id.ends_with("/positive") && case.id.split('/').count() == 3
                }) {
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

    #[test]
    fn composed_firewalls_and_no_reader_capabilities() {
        std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
            let context = Context::discover().unwrap();
            let cards = super::super::cards(&context).unwrap();
            let candidate = super::super::render(&context.read("book-1/source/constitution.nibli").unwrap(), &cards).unwrap();
            let engine = PreparedPinEngine::new(&[LoadedSource::new("protective candidate", &candidate)]);
            for case in super::firewalls(&cards) {
                let out = engine.run_case(&[], &[LoadedSource::new(&case.id, &case.pins)], PinOptions::default(), true);
                assert_eq!(out.exit_code, 0, "{}: {out:?}", case.id);
            }
            let strata = engine.dump_strata();
            assert_eq!(strata.exit_code, 0, "{strata:?}");
            for row in strata.stdout.lines().filter(|row| !row.starts_with('#')) {
                for edge in row.split('\t').nth(3).unwrap_or_default().split(',') {
                    let predicate = edge.trim().trim_start_matches(['+', '-']);
                    assert!(!super::super::contracts::CAPABILITY_LEAVES.contains(&predicate), "unexpected capability reader: {row}");
                }
            }
            let mut counterfactual = candidate.clone();
            for rule in super::super::protections::rules().iter().filter(|rule| rule.ends_with("-> family($subject).") || rule.ends_with("-> parent($subject, $child).")) {
                assert_eq!(counterfactual.matches(rule).count(), 1);
                counterfactual = counterfactual.replacen(rule, "", 1);
            }
            let control = PreparedPinEngine::new(&[LoadedSource::new("explicit relationship counterfactual", &counterfactual)]);
            let pins = ":expect-pins 2\n:accept-scoped\nall $x: person($x) & ~family($x) -> prisoner($x).\n:accept-scoped\nall $x: all $y: person($x) & ~parent($y, $x) -> prisoner($x).\n";
            let out = control.run_case(&[], &[LoadedSource::new("relationship gap control", pins)], PinOptions::default(), true);
            assert_eq!(out.exit_code, 0, "{out:?}");
        }).unwrap().join().unwrap();
    }
}
