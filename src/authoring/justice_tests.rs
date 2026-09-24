// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn actual_cases_preserve_judicial_and_person_boundaries() {
    // Debug builds need the same roomy stack as the existing host integration
    // tests when parsing the complete constitutional rule expressions.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run_actual_cases)
        .unwrap()
        .join()
        .unwrap();
}

fn run_actual_cases() {
    use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};
    let context = Context::discover().unwrap();
    let source = context.read("book-1/source/constitution.nibli").unwrap();
    let inventory: Export =
        serde_json::from_str(&context.read("tests/pins/suites.json").unwrap()).unwrap();
    let mut engines = BTreeMap::new();
    for id in [
        "case-relief/positive",
        "case-relief/without-actual-court-authority",
        "case-relief/court-conflict-sequence",
        "case-relief/independence",
        "composition-review/positive",
        "general-invalidation/positive",
        "enforcement/positive",
        "enforcement/wrong-case",
        "enforcement/wrong-executor",
        "defect/independence",
        "defect/no-adjudicator-self-review",
        "defect/no-executor-self-review",
        "appeal/independence",
        "enforcement/counterfactual-borrow-other-case",
        "enforcement/counterfactual-no-relief",
        "enforcement/no-general-invalidation",
        "enforcement/no-custodial-remedy-kind",
        "defect/downstream-withdrawal-and-fresh-correction",
        "defect/wrong-version-cannot-withdraw-real-relief",
        "restoration/claimant-Withdrawal",
        "restoration/respondent-CoercedConsent",
        "restoration/counterfactual-no-consent",
        "nonresponse/independence",
        "boundaries/no-person-consequences-or-forged-orders",
    ] {
        let name = format!("justice/{id}");
        let case = inventory
            .cases
            .iter()
            .find(|c| c.id == name)
            .unwrap_or_else(|| panic!("missing case {name}"));
        let engine = engines.entry(case.base.clone()).or_insert_with(|| {
            let text =
                super::super::apply_edits(&source, &inventory.bases[&case.base].edits).unwrap();
            PreparedPinEngine::new(&[LoadedSource::new("actual constitution", &text)])
        });
        let fixtures = case
            .fixtures
            .iter()
            .map(|p| context.read(p).unwrap())
            .collect::<Vec<_>>();
        let pins = case
            .pins
            .iter()
            .map(|p| context.read(p).unwrap())
            .collect::<Vec<_>>();
        let fixture_sources = case
            .fixtures
            .iter()
            .zip(&fixtures)
            .map(|(p, s)| LoadedSource::new(p, s))
            .collect::<Vec<_>>();
        let pin_sources = case
            .pins
            .iter()
            .zip(&pins)
            .map(|(p, s)| LoadedSource::new(p, s))
            .collect::<Vec<_>>();
        let output = engine.run_case(&fixture_sources, &pin_sources, PinOptions::default(), true);
        assert_eq!(output.exit_code, 0, "{name}: {output:?}");
    }
}

#[test]
fn cards_are_scoped_and_do_not_create_courts_people_or_coercion() {
    let context = Context::discover().unwrap();
    let cards = cards(&context).unwrap();
    let forbidden = Regex::new(r"^(authority|person|prisoner|free|travel|decide|reward|false|lose|eats|dwell|healthy|secure|believe|expresses|learn|meets)\(").unwrap();
    for c in &cards {
        assert!(heads(c).iter().all(|h| !forbidden.is_match(h)), "{}", c.id);
        assert_eq!(
            fields(c).len(),
            fields(c)
                .iter()
                .map(|(_, s)| s)
                .collect::<BTreeSet<_>>()
                .len(),
            "{}",
            c.id
        );
        assert!(premises(&cards, c).contains(&INDEPENDENT.into()));
        if c.court.is_some() {
            assert!(
                premises(&cards, c)
                    .iter()
                    .any(|a| a.starts_with("authority(FSBOD_"))
            );
        }
    }
    assert!(
        !fields(card(&cards, "hearing"))
            .iter()
            .any(|(_, s)| *s == "Prosecutor")
    );
}

#[test]
fn generation_is_idempotent_and_preserves_other_families() {
    let live = Context::discover().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let context = Context::from_test_root(directory.path().to_owned());
    std::fs::create_dir_all(context.path("book-1/source")).unwrap();
    for path in [
        "book-1/source/constitution.nibli",
        "book-1/source/state-form-source.json",
        "book-1/source/procedural-load-source.json",
    ] {
        std::fs::copy(live.path(path), context.path(path)).unwrap();
    }
    let before = context.read("book-1/source/constitution.nibli").unwrap();
    let mut first = Export::new();
    generate(&context, &mut first).unwrap();
    let generated = context.read("book-1/source/constitution.nibli").unwrap();
    let mut second = Export::new();
    generate(&context, &mut second).unwrap();
    assert_eq!(
        generated,
        context.read("book-1/source/constitution.nibli").unwrap()
    );
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    assert_eq!(
        before
            .split_once(BEGIN)
            .map_or(before.trim_end(), |(b, _)| b.trim_end()),
        generated.split_once(BEGIN).unwrap().0.trim_end()
    );
    for c in &first.cases {
        assert!(c.scan);
        for path in &c.fixtures {
            for line in context
                .read(path)
                .unwrap()
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
            {
                assert!(
                    !line.contains("->") && !line.contains('$') && line.ends_with('.'),
                    "{line}"
                );
            }
        }
    }
}

#[test]
fn case_relief_consumes_the_complete_existing_court_contract() {
    let context = Context::discover().unwrap();
    let cards = cards(&context).unwrap();
    let atoms = premises(&cards, card(&cards, "case-relief"));
    for atom in
        super::super::state_form::court_consumer(&context, 22, "case_specific_relief").unwrap()
    {
        let renamed = variables()
            .replace_all(&atom, |m: &regex::Captures<'_>| court_variable(&m[0]))
            .into_owned();
        assert!(atoms.contains(&renamed), "{renamed}");
    }
    for scope in [
        "Subject",
        "Case",
        "SourceVersion",
        "Epoch",
        "Jurisdiction",
        "LegalScope",
    ] {
        assert!(
            joined(card(&cards, "enforcement"), card(&cards, "case-relief"))
                .iter()
                .any(|(_, s)| *s == scope)
        );
    }
}
