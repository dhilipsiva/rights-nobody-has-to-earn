// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact ordinary-KR records; agreement is not authentication or a clock.

use super::{Card, Dependency, Values};
use regex::Regex;
use std::collections::BTreeSet;

pub(super) const ATTESTERS: &[(&str, &str)] = &[
    ("$source", "ECSourceAuthority"),
    ("$review", "ECIndependentReviewAuthority"),
];
pub(super) const READERS: &[(&str, &str)] = &[
    ("$reader", "ECChallengeReaderAuthority"),
    ("$alternate", "ECAlternateReaderAuthority"),
    ("$auditor", "ECAuditReaderAuthority"),
];

pub(super) fn pattern() -> &'static Regex {
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\$[a-z][a-z0-9_]*").unwrap())
}

fn common(card: &Card) -> Vec<(&'static str, &'static str)> {
    vec![
        ("$subject", "Subject"),
        ("$case", "Case"),
        ("$revision", "Revision"),
        ("$version", "ConstitutionVersion"),
        ("$evidence_record", "EvidenceRecord"),
        ("$evidence_version", "EvidenceVersion"),
        ("$method", "MethodVersion"),
        ("$place", "Place"),
        ("$territory", "Territory"),
        ("$population", "AffectedPopulation"),
        ("$jurisdiction", "Jurisdiction"),
        ("$scope", "LegalScope"),
        ("$operator", "ResponsibleActor"),
        ("$reader", "ChallengeReader"),
        ("$alternate", "AlternateReader"),
        ("$auditor", "AuditReader"),
        ("$reason", "PublicReasons"),
        ("$challenge", "ChallengeRoute"),
        ("$correction", "CorrectionRoute"),
        ("$remedy", "RemedyRoute"),
        ("$window", "Window"),
        ("$start", "Start"),
        ("$end", "End"),
        ("$evaluation", "FreshEvaluation"),
        ("$revision", "SelectedRevision"),
        (
            if card.historical {
                "ECAuthenticatedPastDispositionNotContinuingAuthority"
            } else {
                "ECIndependentlyEstablishedCurrentWindow"
            },
            "CurrentDisposition",
        ),
        (
            if card.historical {
                "ECAuthenticationEvaluationWithinItsOwnRecordedWindowAndExactPastEventScope"
            } else {
                "ECStartBeforeEndAndEvaluationWithinOwnWindow"
            },
            "TemporalFinding",
        ),
        (
            if card.historical {
                "ECRetainOnlyExactHistoricalReplayIdentityNoCarryOfCurrentLegalForce"
            } else {
                "ECNoCarryAcrossVersionScopeSubjectOrWindow"
            },
            "Carry",
        ),
        (
            "ECPositiveConflictRecusalAndIndependenceReview",
            "Independence",
        ),
        (
            "ECPurposeLimitedNecessaryPrivateEvidenceAndIndependentAccess",
            "Privacy",
        ),
        (
            "ECAccessibleReasonsUncertaintyAndSourceLimitations",
            "Disclosure",
        ),
        (
            "ECReviewableRetentionDeletionCorrectionAndLawfulHistory",
            "RecordControl",
        ),
        (
            "ECPreserveUrgentProtectionAndRemedyDuringRecordFailure",
            "Continuity",
        ),
    ]
}

pub(super) fn fields(card: &Card) -> Vec<(String, String)> {
    let mut fields = common(card)
        .into_iter()
        .chain(card.fields.iter().copied())
        .map(|(value, scope)| (value.into(), scope.into()))
        .collect::<Vec<_>>();
    fields.extend(card.external_bindings.clone());
    fields.push((card.kind.into(), "Kind".into()));
    fields.push((card.class.into(), "DirectEffectClass".into()));
    fields.push((
        format!("{}TemporalContract", card.kind),
        "TemporalContractKind".into(),
    ));
    fields
}

pub(super) fn observe(actor: &str, record: &str, value: &str, scope: &str) -> String {
    format!("observe({actor}, {record}, {value}, EC{scope}Scope)")
}

pub(super) fn rule(atoms: &[String], head: &str) -> String {
    let text = format!("{} {head}", atoms.join(" "));
    let variables = pattern()
        .find_iter(&text)
        .map(|m| m.as_str())
        .collect::<BTreeSet<_>>();
    format!(
        "{}{} -> {head}.",
        variables
            .into_iter()
            .map(|v| format!("all {v}: "))
            .collect::<String>(),
        atoms.join(" & ")
    )
}

pub(super) fn dependency_variable(
    consumer: &Card,
    parent: &Card,
    dependency: &Dependency,
    variable: &str,
) -> String {
    if let Some(value) = consumer
        .dependency_bindings
        .get(&(dependency.key.into(), variable.into()))
    {
        return value.clone();
    }
    if [
        "$subject",
        "$case",
        "$version",
        "$place",
        "$territory",
        "$population",
        "$jurisdiction",
        "$scope",
    ]
    .contains(&variable)
        || consumer
            .fields
            .iter()
            .any(|(value, scope)| *value == variable && parent.fields.contains(&(*value, *scope)))
    {
        variable.into()
    } else {
        format!(
            "${}_{}",
            dependency.key.replace('-', "_"),
            variable.trim_start_matches('$')
        )
    }
}

fn dependency_scope(id: &str, field: &str) -> String {
    let id = id
        .split('-')
        .map(|s| {
            let (first, rest) = s.split_at(1);
            format!("{}{rest}", first.to_ascii_uppercase())
        })
        .collect::<String>();
    format!("Dependency{id}{field}")
}

pub(super) fn base_premises(cards: &[Card], card: &Card) -> Vec<String> {
    let mut atoms = ATTESTERS
        .iter()
        .chain(READERS)
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    atoms.push("authorized($operator, ECCaseScopedActor, $record)".into());
    let independent: Vec<_> = ATTESTERS
        .iter()
        .chain(READERS)
        .map(|(actor, _)| *actor)
        .collect();
    for (index, actor) in independent.iter().enumerate() {
        for other in &independent[index + 1..] {
            atoms.push(format!("~({actor} = {other})"));
        }
        for other in ["$operator", "$subject"] {
            atoms.push(format!("~({actor} = {other})"));
        }
    }
    atoms.push("~related($record, ECRecordAmbiguity)".into());
    if !card.withdrawal {
        atoms.push("~contradict($record, $revision, ECRecordReliance)".into());
    }
    if !card.withdrawal && !card.historical {
        atoms.push("~contradict($window, ECSourceWindowCurrentReliance)".into());
    }
    for (value, scope) in fields(card) {
        for (actor, _) in ATTESTERS {
            atoms.push(observe(actor, "$record", &value, &scope));
        }
    }
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        for (variable, scope) in [
            ("$record", "Record"),
            ("$source", "Writer"),
            ("$review", "Reviewer"),
            ("$revision", "Revision"),
            ("$subject", "Subject"),
            ("$case", "Case"),
            ("$version", "ConstitutionVersion"),
            ("$place", "Place"),
            ("$territory", "Territory"),
            ("$population", "AffectedPopulation"),
            ("$jurisdiction", "Jurisdiction"),
            ("$scope", "LegalScope"),
            ("$operator", "Actor"),
            ("$evidence_record", "EvidenceRecord"),
            ("$evidence_version", "EvidenceVersion"),
            ("$method", "MethodVersion"),
            ("$window", "Window"),
            ("$start", "Start"),
            ("$end", "End"),
            ("$evaluation", "Evaluation"),
        ] {
            for (actor, _) in ATTESTERS {
                atoms.push(observe(
                    actor,
                    "$record",
                    &dependency_variable(card, parent, dependency, variable),
                    &dependency_scope(dependency.key, scope),
                ));
            }
        }
    }
    atoms.extend(card.extra.clone());
    atoms
}

pub(super) fn premises(cards: &[Card], card: &Card) -> Vec<String> {
    let mut atoms = base_premises(cards, card);
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        let rename = |text: &str| {
            pattern()
                .replace_all(text, |m: &regex::Captures<'_>| {
                    dependency_variable(card, parent, dependency, &m[0])
                })
                .into_owned()
        };
        atoms.push(rename(&heads(parent)[0]));
        // Read the actual derived parent, then rejoin all of its own scoped
        // fields and currentness. The parent rule still requires its entire
        // raw upstream proof. Repeating those foreign contracts at every
        // downstream edge expands the rule graph without adding a condition.
        // No derived outcome is admitted by the fixtures or this interface.
        let mut own = parent.clone();
        own.dependencies.clear();
        own.extra.clear();
        atoms.extend(base_premises(cards, &own).iter().map(|atom| rename(atom)));
    }
    atoms
}

pub(super) fn heads(card: &Card) -> Vec<String> {
    let mut heads = vec![
        card.conclusion
            .map(str::to_owned)
            .unwrap_or_else(|| format!("complete($record, {}, $subject)", card.kind)),
    ];
    heads.extend(card.effects.clone());
    heads
}

fn rule_bodies(cards: &[Card], card: &Card) -> Vec<Vec<String>> {
    let mut alternatives = std::collections::BTreeMap::new();
    for link in &card.external {
        if let Some((expression, choices)) = &link.alternatives {
            alternatives.insert(expression.clone(), choices.clone());
        }
    }
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        let rename = |text: &str| {
            pattern()
                .replace_all(text, |m: &regex::Captures<'_>| {
                    dependency_variable(card, parent, dependency, &m[0])
                })
                .into_owned()
        };
        for link in &parent.external {
            if let Some((expression, choices)) = &link.alternatives {
                alternatives.insert(
                    rename(expression),
                    choices.iter().map(|choice| rename(choice)).collect(),
                );
            }
        }
    }
    let mut bodies = vec![Vec::new()];
    for atom in premises(cards, card) {
        let choices = alternatives
            .get(&atom)
            .cloned()
            .unwrap_or_else(|| vec![atom]);
        bodies = bodies
            .into_iter()
            .flat_map(|body| {
                choices.iter().map(move |choice| {
                    let mut branch = body.clone();
                    branch.push(choice.clone());
                    branch
                })
            })
            .collect();
    }
    bodies
}

pub(super) fn rules(cards: &[Card]) -> Vec<String> {
    let mut rules = vec!["derived_only(\"oppose\").".into()];
    rules.extend(super::environment::vocabulary_rules());
    rules.extend(super::protection::vocabulary_rules());
    rules.extend(super::continuity::vocabulary_rules());
    rules.extend(super::animals::rules());
    rules.extend(super::animal_use::rules());
    rules.extend(super::animal_context::rules());
    rules.extend(super::animal_remedies::rules());
    rules.extend(super::lifecycle::rules(cards));
    rules.extend(super::guardian::rules(cards));
    rules.extend(super::external::rules());
    let scopes = cards
        .iter()
        .flat_map(|card| {
            base_premises(cards, card).into_iter().filter_map(|atom| {
                atom.strip_prefix("observe(")?
                    .strip_suffix(')')?
                    .rsplit_once(", ")
                    .map(|(_, scope)| scope.to_owned())
            })
        })
        .collect::<BTreeSet<_>>();
    for scope in scopes {
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, ECSingleValueScope)."));
    }
    for (_, role) in ATTESTERS {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, ECRecordAttester)."));
    }
    for role in [
        "ECDecisionAdministrationAuthority",
        "ECCompletenessAssuranceAuthority",
        "ECResultServiceAuthority",
        "ECReplayRegistryAuthority",
        "ECReplayReviewAuthority",
        "ECAnimalScientificReviewAuthority",
        "ECAnimalEthicalReviewAuthority",
    ] {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, ECRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, ECRecordAttester) & related($second, $record, ECRecordAttester) & member($scope, ECSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, ECRecordAmbiguity).".into());
    for card in cards {
        let heads = heads(card);
        let record_keyed = pattern()
            .find_iter(&heads[0])
            .any(|variable| variable.as_str() == "$record");
        // Scope each full alternative to its own variables. Inactive branch
        // witnesses must not become unguarded universal domain requirements.
        for body in rule_bodies(cards, card) {
            rules.push(rule(&body, &heads[0]));
            // Taxon and amendment barriers intentionally have global target
            // keys. They cannot certify a different finding's own effects.
            // Keep those effects bound to the complete raw finding instead.
            if !record_keyed {
                for effect in &heads[1..] {
                    rules.push(rule(&body, effect));
                }
            }
        }
        if !record_keyed {
            continue;
        }
        // As in the existing public-safety contracts, each direct effect reads
        // the actual derived primary conclusion and rejoins its own record.
        // That conclusion still requires every raw upstream witness. Its
        // consumers need not repeat the full upstream proof in each head.
        let mut own = card.clone();
        own.dependencies.clear();
        own.extra.clear();
        let mut effect_body = base_premises(cards, &own);
        effect_body.push(heads[0].clone());
        for effect in &heads[1..] {
            rules.push(rule(&effect_body, effect));
        }
    }
    rules
}

/// Ruling D6: a claim, an interim protection or a continuity record that
/// only gives its subject something takes effect on the source alone. Its
/// duties follow at once from every alternative body, the named reviewer owes
/// prompt review and may withdraw them, and the completed record — which the
/// Guardian's stay, merits and every adverse route read — stays behind full
/// procedure.
pub(super) fn single_actor_rules(cards: &[Card], beneficial: &BTreeSet<String>) -> Vec<String> {
    use super::super::procedural_load::{fast_head, single_actor, PROMPT_REVIEW};
    let mut rules = Vec::new();
    for card in cards.iter().filter(|c| beneficial.contains(c.kind)) {
        let heads = heads(card);
        for body in rule_bodies(cards, card) {
            let fast = single_actor(&body, &[], "$review");
            for effect in heads.iter().skip(1).filter(|h| fast_head(h)) {
                rules.push(rule(&fast, effect));
            }
            rules.push(rule(&fast, &format!("obliged($review, {PROMPT_REVIEW}, $record)")));
        }
    }
    rules
}

pub(super) fn ground(text: &str, values: &Values) -> String {
    pattern()
        .replace_all(text, |m: &regex::Captures<'_>| {
            values
                .get(&m[0])
                .unwrap_or_else(|| panic!("unbound ecological variable {}", &m[0]))
                .clone()
        })
        .into_owned()
}

fn fixture_variables(cards: &[Card], card: &Card) -> BTreeSet<String> {
    let text = format!(
        "{} {}",
        premises(cards, card).join(" "),
        heads(card).join(" ")
    );
    let mut variables: BTreeSet<String> = pattern()
        .find_iter(&text)
        .map(|m| m.as_str().into())
        .collect();
    // An actual upstream positive may include deep evidence which its public
    // consumer need not repeat. Give that evidence case-scoped identities too;
    // fixture bookkeeping must not add new constitutional premises.
    for link in &card.external {
        variables.extend(link.fixture_variables().map(str::to_owned));
    }
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        for variable in fixture_variables(cards, parent) {
            let target = dependency_variable(card, parent, dependency, &variable);
            if target.starts_with('$') {
                variables.insert(target);
            }
        }
    }
    variables
}

pub(super) fn values(cards: &[Card], card: &Card, prefix: &str) -> Values {
    let mut values: Values = fixture_variables(cards, card)
        .into_iter()
        .enumerate()
        // Fixture identities carry no legal meaning. Enumerate the exact
        // variable set instead of repeating deep dependency paths in every
        // fact. This is injective within the case; all explicit bindings and
        // legal constants below remain unchanged. It computes no fingerprint.
        .map(|(index, variable)| (variable, format!("{prefix}V{index}")))
        .collect();
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        for (variable, value) in values_for_defaults(cards, parent) {
            let target = dependency_variable(card, parent, dependency, &variable);
            if target.starts_with('$') {
                values.insert(target, value);
            }
        }
    }
    values.extend(card.defaults.clone());
    for link in &card.external {
        values.extend(link.defaults.clone());
    }
    values
}

fn values_for_defaults(cards: &[Card], card: &Card) -> Values {
    let mut defaults = Values::new();
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        for (variable, value) in values_for_defaults(cards, parent) {
            let target = dependency_variable(card, parent, dependency, &variable);
            if target.starts_with('$') {
                defaults.insert(target, value);
            }
        }
    }
    defaults.extend(card.defaults.clone());
    for link in &card.external {
        defaults.extend(link.defaults.clone());
    }
    defaults
}

pub(super) fn fixture(cards: &[Card], card: &Card, values: &Values) -> String {
    let mut facts = Vec::new();
    for link in &card.external {
        facts.extend(link.fixture(values).lines().map(str::to_owned));
    }
    for dependency in &card.dependencies {
        let parent = super::card(cards, dependency.id);
        let mut upstream = self::values(
            cards,
            parent,
            &format!("{}{}", values["$record"], dependency.key.replace('-', "")),
        );
        for (variable, value) in &mut upstream {
            let target = dependency_variable(card, parent, dependency, variable);
            if target.starts_with('$') {
                if let Some(shared) = values.get(&target) {
                    *value = shared.clone();
                }
            } else {
                *value = target;
            }
        }
        facts.extend(fixture(cards, parent, &upstream).lines().map(str::to_owned));
    }
    for atom in base_premises(cards, card) {
        if ["observe(", "authorized(", "person(", "challenge("]
            .iter()
            .any(|name| atom.starts_with(name))
        {
            facts.push(format!("{}.", ground(&atom, values)));
        }
    }
    let mut seen = BTreeSet::new();
    facts
        .into_iter()
        .filter(|fact| seen.insert(fact.clone()))
        .map(|fact| format!("{fact}\n"))
        .collect()
}
