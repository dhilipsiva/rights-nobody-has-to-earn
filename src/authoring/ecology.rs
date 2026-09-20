// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit environmental/animal contracts and ordinary Nibli cases.

use crate::{cli::Error, context::Context};
use std::collections::{BTreeMap, BTreeSet};

#[path = "ecology_animal_cases.rs"]
mod animal_cases;
#[path = "ecology_animal_context.rs"]
mod animal_context;
#[path = "ecology_animal_remedies.rs"]
mod animal_remedies;
#[path = "ecology_animal_use.rs"]
mod animal_use;
#[path = "ecology_animals.rs"]
mod animals;
#[path = "ecology_cases.rs"]
mod cases;
#[path = "ecology_continuity.rs"]
mod continuity;
#[path = "ecology_corridor.rs"]
mod corridor;
#[path = "ecology_counterfactuals.rs"]
mod counterfactuals;
#[path = "ecology_domain_cases.rs"]
mod domain_cases;
#[path = "ecology_economic.rs"]
mod economic;
#[path = "ecology_environment.rs"]
mod environment;
#[path = "ecology_external.rs"]
mod external;
#[path = "ecology_guardian.rs"]
mod guardian;
#[path = "ecology_history.rs"]
mod history;
#[path = "ecology_history_cases.rs"]
mod history_cases;
#[path = "ecology_information.rs"]
mod information;
#[path = "ecology_institutions.rs"]
mod institutions;
#[path = "ecology_integration.rs"]
mod integration;
#[path = "ecology_judicial.rs"]
mod judicial;
#[path = "ecology_lifecycle.rs"]
mod lifecycle;
#[path = "ecology_lifecycle_cases.rs"]
mod lifecycle_cases;
#[path = "ecology_protection.rs"]
mod protection;
#[path = "ecology_records.rs"]
mod records;
#[path = "ecology_scarcity_cases.rs"]
mod scarcity_cases;

const BEGIN: &str = "# <ECOLOGICAL-ANIMAL-RULES-BEGIN>";
const END: &str = "# <ECOLOGICAL-ANIMAL-RULES-END>";
type Values = BTreeMap<String, String>;

#[derive(Clone, Copy, Debug)]
struct Dependency {
    id: &'static str,
    key: &'static str,
}

#[derive(Clone, Debug)]
struct Card {
    id: &'static str,
    kind: &'static str,
    class: &'static str,
    fields: Vec<(&'static str, &'static str)>,
    dependencies: Vec<Dependency>,
    extra: Vec<String>,
    effects: Vec<String>,
    conclusion: Option<&'static str>,
    defaults: Values,
    dependency_bindings: BTreeMap<(String, String), String>,
    external: Vec<external::Link>,
    external_bindings: Vec<(String, String)>,
    // Findings that withdraw reliance must not negatively read their own result.
    withdrawal: bool,
    // A bounded historical disposition is not a continuing legal authority.
    historical: bool,
}

impl Card {
    fn new(id: &'static str, kind: &'static str, class: &'static str) -> Self {
        Self {
            id,
            kind,
            class,
            fields: Vec::new(),
            dependencies: Vec::new(),
            extra: Vec::new(),
            effects: Vec::new(),
            conclusion: None,
            defaults: Values::new(),
            dependency_bindings: BTreeMap::new(),
            external: Vec::new(),
            external_bindings: Vec::new(),
            withdrawal: false,
            historical: false,
        }
    }

    fn fields(mut self, fields: &[(&'static str, &'static str)]) -> Self {
        self.fields.extend_from_slice(fields);
        self
    }

    fn depends(mut self, ids: &[&'static str]) -> Self {
        self.dependencies
            .extend(ids.iter().map(|id| Dependency { id, key: id }));
        self
    }

    fn repeated(mut self, id: &'static str, key: &'static str, bindings: &[(&str, &str)]) -> Self {
        self.dependencies.push(Dependency { id, key });
        for (variable, value) in bindings {
            self.dependency_bindings
                .insert((key.into(), (*variable).into()), (*value).into());
        }
        self
    }

    fn extra(mut self, atom: &str) -> Self {
        self.extra.push(atom.into());
        self
    }

    fn effect(mut self, atom: &str) -> Self {
        self.effects.push(atom.into());
        self
    }

    fn concludes(mut self, atom: &'static str) -> Self {
        self.conclusion = Some(atom);
        self
    }

    fn withdrawal(mut self) -> Self {
        self.withdrawal = true;
        self
    }

    fn historical(mut self) -> Self {
        self.historical = true;
        self
    }

    fn duty(self, duty: &str) -> Self {
        self.effect(&format!("obliged($operator, {duty}, $record)"))
    }

    fn default(mut self, variable: &str, value: &str) -> Self {
        self.defaults.insert(variable.into(), value.into());
        self
    }

    fn join(mut self, dependency: &str, variable: &str, value: &str) -> Self {
        self.dependency_bindings
            .insert((dependency.into(), variable.into()), value.into());
        self
    }

    fn separate_context(mut self, dependency: &str) -> Self {
        for variable in [
            "$subject",
            "$case",
            "$place",
            "$territory",
            "$population",
            "$scope",
        ] {
            self = self.join(
                dependency,
                variable,
                &format!(
                    "${}_{}",
                    dependency.replace('-', "_"),
                    variable.trim_start_matches('$')
                ),
            );
        }
        self
    }
}

fn cards(context: &Context) -> Result<Vec<Card>, Error> {
    let mut cards = environment::cards();
    cards.extend(protection::cards());
    cards.extend(institutions::cards());
    cards.extend(continuity::cards());
    cards.extend(guardian::cards());
    cards.extend(judicial::cards());
    cards.extend(information::cards());
    cards.extend(animals::cards());
    cards.extend(animal_use::cards());
    cards.extend(animal_context::cards());
    cards.extend(animal_remedies::cards());
    cards.extend(integration::cards());
    cards.extend(lifecycle::cards());
    cards.extend(corridor::cards());
    history::extend(&mut cards);
    external::connect(context, &mut cards)?;
    Ok(cards)
}

fn card<'a>(cards: &'a [Card], id: &str) -> &'a Card {
    cards
        .iter()
        .find(|card| card.id == id)
        .expect("declared ecological dependency")
}

fn validate(cards: &[Card]) -> Result<(), Error> {
    let ids: BTreeSet<_> = cards.iter().map(|card| card.id).collect();
    let kinds: BTreeSet<_> = cards.iter().map(|card| card.kind).collect();
    if ids.len() != cards.len() || kinds.len() != cards.len() {
        return Err(Error::new("duplicate ecological card or effect kind"));
    }
    for card in cards {
        let fields = records::fields(card);
        let scopes: BTreeSet<_> = fields.iter().map(|(_, scope)| scope).collect();
        if scopes.len() != fields.len() {
            return Err(Error::new(format!(
                "{} repeats a single-valued scope",
                card.id
            )));
        }
        let dependency_keys: BTreeSet<_> = card
            .dependencies
            .iter()
            .map(|dependency| dependency.key)
            .collect();
        if dependency_keys.len() != card.dependencies.len() {
            return Err(Error::new(format!(
                "{} repeats a dependency identity",
                card.id
            )));
        }
        for (key, _) in card.dependency_bindings.keys() {
            if !dependency_keys.contains(key.as_str()) {
                return Err(Error::new(format!(
                    "{} joins an absent dependency {key}",
                    card.id
                )));
            }
        }
        let mut pending: Vec<_> = card
            .dependencies
            .iter()
            .map(|dependency| dependency.id)
            .collect();
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if id == card.id || !ids.contains(id) {
                return Err(Error::new(format!(
                    "{} has a cyclic or absent dependency {id}",
                    card.id
                )));
            }
            if seen.insert(id) {
                pending.extend(
                    self::card(cards, id)
                        .dependencies
                        .iter()
                        .map(|dependency| dependency.id),
                );
            }
        }
        let body = records::premises(cards, card);
        let positive: BTreeSet<_> = body
            .iter()
            .filter(|atom| !atom.starts_with('~'))
            .flat_map(|atom| {
                records::pattern()
                    .find_iter(atom)
                    .map(|m| m.as_str().to_owned())
            })
            .collect();
        for atom in body.iter().filter(|atom| atom.starts_with('~')) {
            for variable in records::pattern().find_iter(atom).map(|m| m.as_str()) {
                if !positive.contains(variable) {
                    return Err(Error::new(format!(
                        "{} has an unbound negative-guard variable {variable}",
                        card.id
                    )));
                }
            }
        }
    }
    Ok(())
}

fn render(source: &str, cards: &[Card]) -> Result<String, Error> {
    validate(cards)?;
    let block = format!(
        "{BEGIN}\n# Exact supplied findings. No measurement, authentication, clock, institution or performed remedy is inferred.\n{}\n{END}",
        records::rules(cards).join("\n")
    );
    match (source.matches(BEGIN).count(), source.matches(END).count()) {
        (0, 0) => Ok(format!("{}\n\n{block}\n", source.trim_end())),
        (1, 1) => {
            let (before, rest) = source.split_once(BEGIN).unwrap();
            let (_, after) = rest
                .split_once(END)
                .ok_or_else(|| Error::new("ecological markers out of order"))?;
            Ok(format!("{before}{block}{after}"))
        }
        _ => Err(Error::new("ambiguous ecological rule block")),
    }
}

fn query(atom: &str, expected: bool) -> String {
    format!(
        "? {atom}.\n# => {}\n",
        if expected { "TRUE" } else { "FALSE" }
    )
}

fn add_case(
    context: &Context,
    export: &mut super::Export,
    id: &str,
    fixture: &str,
    pins: &str,
) -> Result<(), Error> {
    let count = pin_count(pins);
    export.add_case(
        context,
        &format!("ecology/{id}"),
        "live",
        fixture,
        &[(
            "expect",
            &format!("# SPDX-License-Identifier: MIT OR Apache-2.0\n:expect-pins {count}\n{pins}"),
        )],
        vec![],
        true,
    )
}

fn pin_count(pins: &str) -> usize {
    pins.lines()
        .filter(|line| {
            line.starts_with('?') || line.starts_with(":accept") || line.starts_with(":refuse")
        })
        .count()
}

pub(crate) fn generate(context: &Context, export: &mut super::Export) -> Result<(), Error> {
    let cards = cards(context)?;
    let path = "book-1/source/constitution.nibli";
    let output = render(&context.read(path)?, &cards)?;
    for card in &cards {
        let mut common = cases::common_for(&cards, card);
        for (family, width) in (1..=8)
            .rev()
            .map(|n| ("required-fields-", n))
            .chain((1..=4).rev().map(|n| ("independence-", n)))
            .chain((1..=4).rev().map(|n| ("conflicting-fields-", n * 2)))
        {
            let prefix = format!("{}/{family}", card.id);
            let matches = |case: &&cases::Scenario| {
                case.id.starts_with(&prefix) && pin_count(&case.pins) == width
            };
            let grouped: Vec<_> = common.iter().filter(matches).collect();
            let Some(first) = grouped.first() else {
                continue;
            };
            let mut shared: BTreeSet<_> = first.facts.lines().map(str::to_owned).collect();
            for case in &grouped[1..] {
                let present: BTreeSet<_> = case.facts.lines().collect();
                shared.retain(|fact| present.contains(fact.as_str()));
            }
            if shared.is_empty() {
                continue;
            }
            let path = format!(
                "tests/pins/ecology/shared/{}-{}-{width}.nibli",
                card.id,
                family.trim_end_matches('-'),
            );
            std::fs::create_dir_all(context.path("tests/pins/ecology/shared"))?;
            let contents = format!(
                "# SPDX-License-Identifier: CC0-1.0\n{}\n",
                shared.iter().cloned().collect::<Vec<_>>().join("\n")
            );
            std::fs::write(context.path(&path), contents)?;
            for case in common.iter().filter(matches) {
                let remainder: String = case
                    .facts
                    .lines()
                    .filter(|line| !shared.contains(*line))
                    .map(|line| format!("{line}\n"))
                    .collect();
                add_case(context, export, &case.id, &remainder, &case.pins)?;
                export
                    .cases
                    .last_mut()
                    .unwrap()
                    .fixtures
                    .insert(0, path.clone());
            }
            common.retain(|case| !matches(&case));
        }
        for case in common {
            add_case(context, export, &case.id, &case.facts, &case.pins)?;
        }
    }
    for case in cases::guardian_sequences(&cards)
        .into_iter()
        .chain(cases::guardian_boundaries(&cards))
        .chain(animal_cases::boundaries(&cards))
        .chain(lifecycle_cases::boundaries(&cards))
        .chain(history_cases::boundaries(&cards))
        .chain(corridor::boundaries(context, &cards)?)
        .chain(domain_cases::boundaries(&cards))
        .chain(scarcity_cases::boundaries(&cards))
    {
        add_case(context, export, &case.id, &case.facts, &case.pins)?;
    }
    for case in counterfactuals::cases(&cards)?
        .into_iter()
        .chain(corridor::counterfactuals(context, &cards)?)
    {
        add_case(
            context,
            export,
            &format!("counterfactual/{}/live-refusal", case.id),
            &case.facts,
            &case.live_pins,
        )?;
        export.add_case(
            context,
            &format!("ecology/counterfactual/{}/changed-source", case.id),
            "live",
            &case.facts,
            &[(
                "expect",
                &format!(
                    "# SPDX-License-Identifier: MIT OR Apache-2.0\n:expect-pins {}\n{}",
                    pin_count(&case.changed_pins),
                    case.changed_pins
                ),
            )],
            case.edits,
            true,
        )?;
    }
    std::fs::write(context.path(path), output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};

    #[test]
    fn replay_tuple_preserves_case_authorization_evidence_and_ground() {
        let final_atoms: Vec<_> = [
            ("$case", "Case"),
            ("$authorization_record", "ChallengedAuthorizationRecord"),
            ("$authorization_version", "AuthorizationVersion"),
            ("$evidence_version", "EvidenceVersion"),
            ("$ground", "StayGround"),
        ]
        .into_iter()
        .map(|(value, scope)| records::observe("ECTupleWriter", "ECTupleFinalRecord", value, scope))
        .collect();
        let mut source =
            "admits(\"observe\"). admits(\"authorized\"). derived_only(\"end\").\n".to_owned();
        for (ground, marker) in [
            (true, "ECFinalGuardianStayReplay"),
            (false, "ECFinallyResolvedAuthorizationEvidencePair"),
        ] {
            let mut atoms = final_atoms.clone();
            atoms.extend(guardian::candidate_key_atoms(ground));
            source += &format!(
                "{}\n",
                records::rule(&atoms, &format!("end($candidate, {marker})"))
            );
        }
        let engine = PreparedPinEngine::new(&[LoadedSource::new("tuple-development", &source)]);
        let values: Values = [
            ("$case", "ECTupleCase"),
            ("$authorization_record", "ECTupleAuthorizationRecord"),
            ("$authorization_version", "ECTupleAuthorization"),
            ("$evidence_version", "ECTupleEvidence"),
            ("$ground", "ECTupleGround"),
        ]
        .into_iter()
        .map(|(a, b)| (a.into(), b.into()))
        .collect();
        let mut facts: String = final_atoms
            .iter()
            .map(|atom| format!("{}.\n", records::ground(atom, &values)))
            .collect();
        let mut pins = String::new();
        for (index, variable) in [
            "unchanged",
            "$case",
            "$authorization_record",
            "$authorization_version",
            "$evidence_version",
            "$ground",
        ]
        .iter()
        .enumerate()
        {
            let mut changed = values.clone();
            if *variable != "unchanged" {
                changed.insert((*variable).into(), "ECTupleOtherValue".into());
            }
            for name in ["candidate", "candidate_source", "candidate_review"] {
                changed.insert(
                    format!("${name}"),
                    format!("ECTuple{index}{}", name.replace('_', "")),
                );
            }
            for atom in guardian::candidate_key_atoms(true)
                .into_iter()
                .filter(|atom| !atom.starts_with('~'))
            {
                facts += &format!("{}.\n", records::ground(&atom, &changed));
            }
            pins += &query(
                &format!("end({}, ECFinalGuardianStayReplay)", changed["$candidate"]),
                *variable == "unchanged",
            );
            pins += &query(
                &format!(
                    "end({}, ECFinallyResolvedAuthorizationEvidencePair)",
                    changed["$candidate"]
                ),
                matches!(*variable, "unchanged" | "$ground"),
            );
        }
        let result = engine.run_case(
            &[LoadedSource::new("tuple", &facts)],
            &[LoadedSource::new(
                "tuple",
                &format!(":expect-pins 12\n{pins}"),
            )],
            PinOptions::default(),
            true,
        );
        assert_eq!(result.exit_code, 0, "{}", result.stderr);
    }

    fn initial_cards() -> Vec<Card> {
        environment::cards()
            .into_iter()
            .filter(|card| {
                matches!(
                    card.id,
                    "environmental-claim"
                        | "science"
                        | "precaution"
                        | "nonfungibility"
                        | "residual-restoration"
                )
            })
            .collect()
    }

    #[test]
    fn ecological_fixture_only_variables_survive_nested_dependencies() {
        let mut cards = cards(&Context::discover().unwrap()).unwrap();
        cards.push(
            Card::new("fixture-child", "ECFixtureChild", "ECDevelopmentOnly")
                .depends(&["ecological-physical-scarcity"]),
        );
        cards.push(
            Card::new(
                "fixture-grandchild",
                "ECFixtureGrandchild",
                "ECDevelopmentOnly",
            )
            .depends(&["fixture-child"]),
        );
        let parent = card(&cards, "ecological-physical-scarcity");
        let body = records::premises(&cards, parent).join(" ");
        let fixture_variable = parent.external[0]
            .fixture_variables()
            .find(|variable| !body.contains(variable))
            .expect("the actual scarcity fixture has deeper evidence than its consumer");
        let parent_values = records::values(&cards, parent, "ECFixtureParent");
        let other_values = records::values(&cards, parent, "ECFixtureOther");
        assert_ne!(
            parent_values[fixture_variable],
            other_values[fixture_variable]
        );
        let parent_fixture = records::fixture(&cards, parent, &parent_values);
        let mut child_values =
            records::values(&cards, card(&cards, "fixture-child"), "ECFixtureChild");
        cases::join_parent(
            &cards,
            card(&cards, "fixture-child"),
            "ecological-physical-scarcity",
            &mut child_values,
            &parent_values,
        );
        let grandchild = card(&cards, "fixture-grandchild");
        let mut grandchild_values = records::values(&cards, grandchild, "ECFixtureGrandchild");
        cases::join_parent(
            &cards,
            grandchild,
            "fixture-child",
            &mut grandchild_values,
            &child_values,
        );
        let actual = records::fixture(&cards, grandchild, &grandchild_values);
        assert!(!actual.contains('$'));
        let actual: BTreeSet<_> = actual.lines().collect();
        assert!(parent_fixture.lines().all(|line| actual.contains(line)));
        for card in &cards {
            let values = records::values(&cards, card, "ECFixtureWhole");
            assert!(
                !records::fixture(&cards, card, &values).contains('$'),
                "{}",
                card.id
            );
        }
    }

    #[test]
    fn ecological_candidate_loads_as_a_single_batch() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let canonical = context.read("book-1/source/constitution.nibli").unwrap();
                let compiler = nibli_session::CoreSession::new();
                // Fail immediately on a malformed or unstratified candidate,
                // before the pin harness's per-statement diagnostic fallback.
                let source = render(&canonical, &cards).unwrap();
                let statements = source
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with(['#', ':', '?']))
                    .map(|line| (compiler.compile_text(line).unwrap(), line.to_owned()))
                    .collect();
                nibli_reason::KnowledgeBase::from_compiled_batch(statements).unwrap_or_else(
                    |error| panic!("complete ecological candidate failed: {error}"),
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ecological_statements_use_the_actual_corpus_signatures() {
        let cards = cards(&Context::discover().unwrap()).unwrap();
        let compiler = nibli_session::CoreSession::new();
        for (index, statement) in records::rules(&cards).iter().enumerate() {
            compiler.compile_text(statement).unwrap_or_else(|error| {
                panic!(
                    "ecological statement {} does not compile: {error}",
                    index + 1
                )
            });
        }
    }

    #[test]
    fn negative_guards_require_positive_record_bindings() {
        let mut cards = initial_cards();
        cards[0]
            .extra
            .push("~oppose($unbound_condition, $record, ECClimateAxis)".into());
        let error = validate(&cards).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unbound negative-guard variable $unbound_condition")
        );
    }

    #[test]
    fn ecological_cards_have_distinct_effects_and_stable_rendering() {
        let cards = cards(&Context::discover().unwrap()).unwrap();
        validate(&cards).unwrap();
        let source = Context::discover()
            .unwrap()
            .read("book-1/source/constitution.nibli")
            .unwrap();
        let first = render(&source, &cards).unwrap();
        let floor =
            regex::Regex::new(r"(?m)^entitled\(every person, event \{ ([a-z]+)\(\) \}\)\.$")
                .unwrap();
        let actual: BTreeSet<_> = floor
            .captures_iter(&source)
            .map(|m| m[1].to_owned())
            .collect();
        assert_eq!(
            actual,
            protection::FLOOR
                .iter()
                .map(|name| (*name).to_owned())
                .collect()
        );
        assert_eq!(first, render(&first, &cards).unwrap());
        assert!(render(&format!("{source}\n{BEGIN}"), &cards).is_err());
        assert!(!records::rules(&cards).iter().any(|rule| {
            ["person(", "reward(", "healthy(", "prisoner("]
                .iter()
                .any(|name| rule.split(" -> ").nth(1).unwrap_or("").starts_with(name))
        }));
        assert!(
            cards
                .iter()
                .all(|card| !card.extra.iter().any(|atom| atom.starts_with("person("))),
            "A present-human source finding must not turn broad standing into a completion premise"
        );
    }

    #[test]
    fn environmental_findings_need_complete_independent_scoped_records() {
        let cards = initial_cards();
        let source = format!(
            "admits(\"observe\"). admits(\"authorized\"). admits(\"person\").\nderived_only(\"complete\"). derived_only(\"prevents\"). derived_only(\"obliged\"). derived_only(\"permits\"). derived_only(\"member\").\n{}\n",
            records::rules(&cards).join("\n")
        );
        let engine = PreparedPinEngine::new(&[LoadedSource::new("development-contracts", &source)]);
        for card in &cards {
            let values = records::values(&cards, card, "ECDev");
            let fixture = records::fixture(&cards, card, &values);
            let completion = records::ground(&records::heads(card)[0], &values);
            for (label, facts, expected) in [
                ("positive", fixture.clone(), true),
                ("absent", String::new(), false),
                (
                    "independent reader missing",
                    fixture.replace(
                        &format!(
                            "authorized({}, ECIndependentReviewAuthority, {}).\n",
                            values["$review"], values["$record"]
                        ),
                        "",
                    ),
                    false,
                ),
            ] {
                let pins = format!(":expect-pins 1\n{}", query(&completion, expected));
                let result = engine.run_case(
                    &[LoadedSource::new(label, &facts)],
                    &[LoadedSource::new(card.id, &pins)],
                    PinOptions::default(),
                    true,
                );
                assert_eq!(
                    result.exit_code, 0,
                    "{} {label}: {}",
                    card.id, result.stderr
                );
            }
        }
    }

    #[test]
    fn every_environmental_scope_and_independence_guard_is_exercised() {
        let cards = initial_cards();
        let source = format!(
            "admits(\"observe\"). admits(\"authorized\"). admits(\"person\").\nderived_only(\"complete\"). derived_only(\"prevents\"). derived_only(\"obliged\"). derived_only(\"permits\"). derived_only(\"member\").\n{}\n",
            records::rules(&cards).join("\n")
        );
        let engine = PreparedPinEngine::new(&[LoadedSource::new("development-contracts", &source)]);
        for case in cases::common(&cards) {
            let count = case
                .pins
                .lines()
                .filter(|line| line.starts_with('?'))
                .count();
            let pins = format!(":expect-pins {count}\n{}", case.pins);
            let result = engine.run_case(
                &[LoadedSource::new(&case.id, &case.facts)],
                &[LoadedSource::new(&case.id, &pins)],
                PinOptions::default(),
                true,
            );
            assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
        }
    }

    #[test]
    fn initial_ecological_findings_compose_with_the_actual_constitution() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(initial_ecological_composition)
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ecological_record_lifecycle_composes_with_actual_sources() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in lifecycle_cases::boundaries(&cards) {
                    eprintln!("ecological record lifecycle: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn guardian_history_survives_current_merits_end_without_carrying_authority() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in history_cases::boundaries(&cards) {
                    eprintln!("ecological historical finality: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn global_findings_cannot_lend_effects_to_unqualified_records() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("actual-constitution", &source)]);
                let cases = domain_cases::record_key_boundaries(&cards);
                assert_eq!(cases.len(), 4);
                for case in cases {
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ecological_domain_alternatives_and_actual_upstream_refusals() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in domain_cases::boundaries(&cards) {
                    eprintln!("ecological domain: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn animal_corridor_blocks_actual_amendment_enactment_only() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in corridor::boundaries(&context, &cards).unwrap() {
                    eprintln!("animal corridor: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ecological_counterfactuals_change_only_explicit_actual_source_rules() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let live =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in counterfactuals::cases(&cards)
                    .unwrap()
                    .into_iter()
                    .chain(corridor::counterfactuals(&context, &cards).unwrap())
                {
                    eprintln!("ecological source counterfactual: {}", case.id);
                    let authored = super::super::apply_edits(&source, &case.edits).unwrap();
                    let edits = case
                        .edits
                        .iter()
                        .map(|edit| crate::execution::Edit {
                            before: edit.before.clone(),
                            after: edit.after.clone(),
                        })
                        .collect::<Vec<_>>();
                    let changed = crate::execution::apply_edits(&source, &edits, &case.id).unwrap();
                    let statements = |text: &str| {
                        text.lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty() && !line.starts_with('#'))
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        statements(&authored),
                        statements(&changed),
                        "{}: authoring/execution edit agreement",
                        case.id
                    );
                    let engine = PreparedPinEngine::new(&[LoadedSource::new(
                        "explicit-source-counterfactual",
                        &changed,
                    )]);
                    for (engine, pins) in [(&live, &case.live_pins), (&engine, &case.changed_pins)]
                    {
                        let pins = format!(":expect-pins {}\n{pins}", pin_count(pins));
                        let result = engine.run_case(
                            &[LoadedSource::new(&case.id, &case.facts)],
                            &[LoadedSource::new(&case.id, &pins)],
                            PinOptions::default(),
                            true,
                        );
                        assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    #[ignore = "manual full common-case audit; routine verification executes the exported cases"]
    fn all_ecological_common_cases_compose_with_actual_sources() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                let mut total_cases = 0;
                let mut total_pins = 0;
                for card in &cards {
                    let started = std::time::Instant::now();
                    let cases = cases::common_for(&cards, card);
                    let count = cases.len();
                    let pins: usize = cases.iter().map(|case| pin_count(&case.pins)).sum();
                    let bytes: usize = cases.iter().map(|case| case.facts.len()).sum();
                    eprintln!(
                        "ecological common {}: {count} cases, {pins} pins, {bytes} fixture bytes",
                        card.id
                    );
                    for case in cases {
                        let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                        let result = engine.run_case(
                            &[LoadedSource::new(&case.id, &case.facts)],
                            &[LoadedSource::new(&case.id, &pins)],
                            PinOptions::default(),
                            true,
                        );
                        assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                    }
                    eprintln!(
                        "ecological common {} passed in {:.2}s",
                        card.id,
                        started.elapsed().as_secs_f64()
                    );
                    total_cases += count;
                    total_pins += pins;
                }
                eprintln!(
                    "all ecological common cases passed: {total_cases} cases, {total_pins} pins"
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn guardian_stay_refusals_match_fresh_composition() {
        std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
            let context = Context::discover().unwrap();
            let cards = cards(&context).unwrap();
            let source = render(&context.read("book-1/source/constitution.nibli").unwrap(), &cards).unwrap();
            let engine = PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
            let execution = card(&cards, "high-consequence");
            let stay = card(&cards, "guardian-stay");
            let h = records::values(&cards, execution, "ECFreshExecution");
            let mut g = records::values(&cards, stay, "ECFreshGuardian");
            for variable in ["$subject", "$case", "$project", "$authorization_record", "$authorization_version", "$evidence_version"] {
                g.insert(variable.into(), h[variable].clone());
            }
            let facts = records::fixture(&cards, execution, &h) + &records::fixture(&cards, stay, &g);
            let ended = records::ground("observe($review, $record, ECEndedGuardianWindow, ECCurrentDispositionScope).\n", &g);
            for (label, fixtures, active) in [("active", facts.clone(), true), ("ended", facts + &ended, false)] {
                let pins = format!(":expect-pins 2\n{}{}", query(&records::ground(&records::heads(stay)[0], &g), active),
                    query(&records::ground(&records::heads(execution)[0], &h), !active));
                let result = engine.run_case(&[LoadedSource::new(label, &fixtures)], &[LoadedSource::new(label, &pins)], PinOptions::default(), true);
                assert_eq!(result.exit_code, 0, "{label}: {}", result.stderr);
            }
        }).unwrap().join().unwrap();
    }

    #[test]
    fn guardian_stay_sequence_uses_the_actual_loaded_sources() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in cases::guardian_sequences(&cards) {
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn guardian_boundaries_compose_with_actual_sources() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in cases::guardian_boundaries(&cards) {
                    eprintln!("ecological guardian boundary: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn animal_boundaries_compose_with_actual_sources() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let context = Context::discover().unwrap();
                let cards = cards(&context).unwrap();
                let source = render(
                    &context.read("book-1/source/constitution.nibli").unwrap(),
                    &cards,
                )
                .unwrap();
                let engine =
                    PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
                for case in animal_cases::boundaries(&cards) {
                    eprintln!("ecological animal boundary: {}", case.id);
                    let pins = format!(":expect-pins {}\n{}", pin_count(&case.pins), case.pins);
                    let result = engine.run_case(
                        &[LoadedSource::new(&case.id, &case.facts)],
                        &[LoadedSource::new(&case.id, &pins)],
                        PinOptions::default(),
                        true,
                    );
                    assert_eq!(result.exit_code, 0, "{}: {}", case.id, result.stderr);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    fn initial_ecological_composition() {
        let context = Context::discover().unwrap();
        eprintln!("ecological composition: preparing card sources");
        let cards = cards(&context).unwrap();
        eprintln!("ecological composition: rendering {} cards", cards.len());
        let source = render(
            &context.read("book-1/source/constitution.nibli").unwrap(),
            &cards,
        )
        .unwrap();
        eprintln!("ecological composition: loading candidate model");
        let engine = PreparedPinEngine::new(&[LoadedSource::new("live-plus-candidate", &source)]);
        for card in &cards {
            eprintln!("ecological live composition: {}", card.id);
            let values = records::values(&cards, card, "ECComposition");
            let fixture = records::fixture(&cards, card, &values);
            let pins = format!(
                ":expect-pins {}\n{}",
                records::heads(card).len(),
                cases::queries(card, &values, true)
            );
            let result = engine.run_case(
                &[LoadedSource::new(card.id, &fixture)],
                &[LoadedSource::new(card.id, &pins)],
                PinOptions::default(),
                true,
            );
            assert_eq!(result.exit_code, 0, "{}: {}", card.id, result.stderr);
        }
    }
}
