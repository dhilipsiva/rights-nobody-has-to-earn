// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit protective-power rules and substantive Nibli scenarios.

#[path = "public_safety_cases.rs"]
mod cases;
#[path = "public_safety_contracts.rs"]
mod contracts;
#[path = "public_safety_external.rs"]
mod external;
#[path = "public_safety_protections.rs"]
mod protections;
#[path = "public_safety_records.rs"]
mod records;
#[path = "public_safety_review.rs"]
mod review;
#[path = "public_safety_temporal.rs"]
mod temporal;

fn cards(context: &crate::context::Context) -> Result<Vec<contracts::Card>, crate::cli::Error> {
    let mut cards = contracts::cards();
    review::extend(&mut cards);
    external::connect(context, &mut cards)?;
    Ok(cards)
}

fn rules(cards: &[contracts::Card]) -> Vec<String> {
    let mut rules = records::rules(cards);
    rules.extend(protections::rules());
    rules.extend(review::rules(cards));
    rules
}

const BEGIN: &str = "# <PUBLIC-SAFETY-RULES-BEGIN>";
const END: &str = "# <PUBLIC-SAFETY-RULES-END>";
const OLD_TRAVEL: &str = "all $anyone: person($anyone) & ~prisoner($anyone) -> travel($anyone).";
const PROTECTED_TRAVEL: &str =
    "all $anyone: person($anyone) & ~prisoner($anyone) & ~restrain($anyone) -> travel($anyone).";

fn render(source: &str, cards: &[contracts::Card]) -> Result<String, crate::cli::Error> {
    use crate::cli::Error;
    let source = match (
        source.matches(OLD_TRAVEL).count(),
        source.matches(PROTECTED_TRAVEL).count(),
    ) {
        (1, 0) => source.replacen(OLD_TRAVEL, PROTECTED_TRAVEL, 1),
        (0, 1) => source.into(),
        _ => {
            return Err(Error::new(
                "public safety needs exactly one recognized movement rule",
            ));
        }
    };
    let block = format!(
        "{BEGIN}\n# Separated protective powers and safeguards. Supplied findings do not authenticate evidence, advance time or perform an act.\nderived_only(\"restrain\").\n{}\n{END}",
        rules(cards).join("\n")
    );
    match (source.matches(BEGIN).count(), source.matches(END).count()) {
        (0, 0) => Ok(format!("{}\n\n{block}\n", source.trim_end())),
        (1, 1) => {
            let (before, rest) = source.split_once(BEGIN).unwrap();
            let (_, after) = rest
                .split_once(END)
                .ok_or_else(|| Error::new("public-safety markers out of order"))?;
            Ok(format!("{before}{block}{after}"))
        }
        _ => Err(Error::new("ambiguous public-safety rule block")),
    }
}

pub(crate) fn generate(
    context: &crate::context::Context,
    export: &mut super::Export,
) -> Result<(), crate::cli::Error> {
    let cards = cards(context)?;
    let path = "new-book-plans/constitution.nibli";
    let rendered = render(&context.read(path)?, &cards)?;
    let mut scenarios = cases::core(&cards);
    scenarios.extend(external::scenarios(&cards)?);
    scenarios.extend(protections::scenarios());
    scenarios.extend(review::scenarios(&cards));
    scenarios.extend(cases::composed_windows(&cards));
    scenarios.extend(cases::economic_boundary(context, &cards)?);
    scenarios.extend(temporal::scenarios(&cards));
    scenarios.extend(cases::firewalls(&cards));
    let ids = scenarios
        .iter()
        .map(|case| &case.id)
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != scenarios.len() {
        return Err(crate::cli::Error::new("duplicate public-safety scenario"));
    }
    std::fs::write(context.path(path), rendered)?;
    for case in scenarios {
        export.add_case(
            context,
            &format!("public-safety/{}", case.id),
            "live",
            &case.facts,
            &[("expect", &case.pins)],
            vec![],
            true,
        )?;
    }
    let relationship_edits = protections::rules()
        .into_iter()
        .filter(|rule| {
            rule.ends_with("-> family($subject).") || rule.ends_with("-> parent($subject, $child).")
        })
        .map(|before| super::Edit {
            before,
            after: String::new(),
        })
        .collect::<Vec<_>>();
    if relationship_edits.len() != 2 {
        return Err(crate::cli::Error::new(
            "expected two positive relationship projections",
        ));
    }
    export.bases.insert(
        "without-positive-relationships".into(),
        super::Base {
            base: Some("live".into()),
            edits: relationship_edits,
            ..super::Base::default()
        },
    );
    export.add_case(context, "public-safety/counterfactual/relationship-gap", "without-positive-relationships", "", &[("expect", ":expect-pins 5\n:accept-scoped\nall $x: person($x) & ~family($x) -> prisoner($x).\n:accept-scoped\nall $x: all $y: person($x) & ~parent($y, $x) -> prisoner($x).\n# Retained pre-repair non-floor control: parentage otherwise joins credibility to personhood.\n:accept-scoped\nall $x: person($x) & ~false($x) -> prisoner($x).\n? family(Lalo).\n# => TRUE\n? parent(Dev, Esa).\n# => TRUE\n")], vec![], true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::context::Context;

    #[test]
    fn source_generation_is_idempotent_and_rejects_ambiguous_movement() {
        let context = Context::discover().unwrap();
        let cards = super::cards(&context).unwrap();
        let source = context.read("new-book-plans/constitution.nibli").unwrap();
        let first = super::render(&source, &cards).unwrap();
        let second = super::render(&first, &cards).unwrap();
        assert!(first == second);
        assert_eq!(first.matches(super::BEGIN).count(), 1);
        assert_eq!(first.matches(super::PROTECTED_TRAVEL).count(), 1);
        assert!(super::render(&format!("{source}\n{}", super::OLD_TRAVEL), &cards).is_err());
    }

    #[test]
    fn exported_inventory_repeats_without_altering_unrelated_source() {
        let live = Context::discover().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let context = Context::from_test_root(directory.path().to_owned());
        std::fs::create_dir_all(context.path("new-book-plans")).unwrap();
        for path in [
            "new-book-plans/constitution.nibli",
            "new-book-plans/state-form-source.json",
            "new-book-plans/economic-power-081.pins.nibli",
            "new-book-plans/economic-power-082.pins.nibli",
        ] {
            std::fs::copy(live.path(path), context.path(path)).unwrap();
        }
        let mut first = super::super::Export::new();
        super::generate(&context, &mut first).unwrap();
        let source = context.read("new-book-plans/constitution.nibli").unwrap();
        let mut second = super::super::Export::new();
        super::generate(&context, &mut second).unwrap();
        assert!(context.read("new-book-plans/constitution.nibli").unwrap() == source);
        assert!(serde_json::to_value(&first).unwrap() == serde_json::to_value(&second).unwrap());
        assert!(first.cases.iter().all(|case| case.scan));
        assert!(
            first
                .cases
                .iter()
                .any(|case| case.base == "without-positive-relationships")
        );
        for case in &first.cases {
            for path in case.fixtures.iter().chain(&case.pins) {
                assert!(context.path(path).is_file(), "{path}");
            }
        }
        eprintln!(
            "exported {} public-safety cases; {} source bytes",
            first.cases.len(),
            source.len()
        );
    }

    #[test]
    fn direct_effect_contracts_keep_force_test_distinct_from_an_order() {
        let cards = super::contracts::cards();
        let by_id = cards
            .iter()
            .map(|c| (c.id, c))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(by_id.len(), cards.len());
        assert_eq!(
            cards
                .iter()
                .map(|c| c.kind)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            cards.len()
        );
        for card in &cards {
            assert_eq!(
                card.fields.len(),
                card.fields
                    .iter()
                    .map(|(_, s)| s)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                "{}",
                card.id
            );
            for dep in &card.dependencies {
                assert!(by_id.contains_key(dep), "{} lacks {dep}", card.id);
                assert_ne!(card.id, *dep);
            }
            if card.holding {
                assert!(card.movement, "{}", card.id);
            }
        }
        assert!(by_id["force-test"].capability.is_none());
        assert!(by_id["lethal-force"].dependencies.contains(&"force-test"));
        assert!(!by_id["lethal-force"].dependencies.contains(&"force"));
        assert!(
            by_id["force"]
                .fields
                .contains(&("SpecifiedNonLethalMeansOnly", "Means"))
        );
    }

    #[test]
    fn examples_supply_witnesses_without_asserting_derived_authority() {
        let context = Context::discover().unwrap();
        let cards = super::cards(&context).unwrap();
        for card in &cards {
            let values = super::records::values(&cards, card, "PSExample");
            let fixture = super::records::fixture(&cards, card, &values);
            assert!(
                fixture.lines().all(|line| line.is_empty()
                    || line.starts_with("authorized(")
                    || line.starts_with("at(")
                    || line.starts_with("challenge(")
                    || line.starts_with("observe(")),
                "{}",
                card.id
            );
        }
    }
}
