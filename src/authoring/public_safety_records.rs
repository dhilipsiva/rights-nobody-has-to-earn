// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact record contracts. External attestations are not authenticated here.

use super::contracts::Card;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const ATTESTERS: &[(&str, &str)] = &[
    ("$source", "PSSourceAuthority"),
    ("$evidence", "PSEvidenceAuthority"),
    ("$authorizer", "PSIndependentAuthorizingAuthority"),
    ("$review", "PSIndependentReviewAuthority"),
];
pub(super) const READERS: &[(&str, &str)] = &[
    ("$reader", "PSChallengeReaderAuthority"),
    ("$alternate", "PSIndependentAlternateReaderAuthority"),
    ("$auditor", "PSIndependentAuditAuthority"),
];

pub(super) fn common() -> Vec<(&'static str, &'static str)> {
    vec![
        ("$subject", "CoreSubject"),
        ("$case", "CoreCase"),
        ("$version", "ConstitutionVersion"),
        ("$revision", "RecordRevision"),
        ("$epoch", "CoreEpoch"),
        ("$jurisdiction", "CoreJurisdiction"),
        ("$scope", "CoreLegalScope"),
        ("$window", "CoreWindow"),
        ("$start", "CoreStart"),
        ("$end", "CoreEnd"),
        ("$holder", "CoreHolder"),
        ("$operator", "CoreExecutor"),
        ("$authorization_mode", "AuthorizationMode"),
        ("$review_mode", "ReviewMode"),
        ("$window_mode", "WindowMode"),
        ("$ordinary_authorizer", "OrdinaryAuthorizingActor"),
        ("$ordinary_reviewer", "OrdinaryReviewingActor"),
        ("$reader", "CoreChallengeReader"),
        ("$alternate", "CoreAlternateReader"),
        ("$auditor", "CoreAuditReader"),
        ("$basis", "CoreEvidenceRecord"),
        ("$law", "CoreDemocraticLegalSource"),
        ("$challenge", "CoreChallenge"),
        ("$correction", "CoreCorrection"),
        ("$remedy", "CoreRemedy"),
        ("CurrentReconciledAuthority", "CoreCurrentDisposition"),
        ("$revision", "CoreCurrentSelectedRevision"),
        (
            "IndependentPositiveCurrentWindowStartBeforeEndFinding",
            "CoreTemporalFinding",
        ),
        (
            "NoAutomaticCarryAcrossRecordRevisionCaseSubjectScopeOrWindow",
            "CoreCarry",
        ),
        (
            "AffirmativeCaseBoundConflictRecusalAndIndependentReview",
            "CoreIndependence",
        ),
        (
            "PublicAccessibleReasonsNarrowPrivateEvidencePurposeRetentionAndCorrection",
            "CorePrivacy",
        ),
        (
            "EveryCategoricalBarAndNonDerogableProtectionPreserved",
            "CoreCorridor",
        ),
        (
            "CaseScopedPublicFunctionExecutorNoDelegatedPrivateCoercion",
            "CoreExecutorLimit",
        ),
        (
            "StandingFloorVoiceFranchiseCounselChallengeAndRemedyContinue",
            "CoreContinuity",
        ),
    ]
}

pub(super) fn fields(card: &Card) -> Vec<(&str, &str)> {
    let mut fields = common();
    fields.push(("$kind", "RecordKind"));
    fields.extend_from_slice(&card.fields);
    fields.extend(
        card.external_bindings
            .iter()
            .map(|(value, scope)| (value.as_str(), scope.as_str())),
    );
    fields
}

pub(super) fn observe(actor: &str, record: &str, value: &str, scope: &str) -> String {
    format!("observe({actor}, {record}, {value}, PS{scope}Scope)")
}

pub(super) fn variable_pattern() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\$[a-z][a-z0-9_]*").unwrap())
}

pub(super) fn rule(atoms: &[String], head: &str) -> String {
    let whole = format!("{} {head}", atoms.join(" "));
    let variables = variable_pattern()
        .find_iter(&whole)
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

pub(super) fn dependency_variable(id: &str, variable: &str) -> String {
    // Shared constitutional context is not a shared record revision or window.
    if matches!(
        variable,
        "$case" | "$version" | "$epoch" | "$jurisdiction" | "$scope"
    ) {
        variable.into()
    } else {
        format!(
            "${}_{}",
            id.replace('-', "_"),
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

pub(super) fn base_premises(card: &Card) -> Vec<String> {
    let mut atoms = ATTESTERS
        .iter()
        .chain(READERS)
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    atoms.push("authorized($operator, PSCaseScopedExecutor, $record)".into());
    let sources = ATTESTERS
        .iter()
        .chain(READERS)
        .map(|(a, _)| *a)
        .collect::<Vec<_>>();
    for (i, a) in sources.iter().enumerate() {
        for b in &sources[i + 1..] {
            atoms.push(format!("~({a} = {b})"));
        }
        for other in ["$subject", "$holder", "$operator"] {
            atoms.push(format!("~({a} = {other})"));
        }
    }
    if super::review::is_finding(card.id) {
        // A finding that withdraws reliance cannot negatively read the same
        // predicate that carries its withdrawal. Both ambiguity markers are
        // independently produced from raw witnesses below, not from each other.
        atoms.push("~related($record, PSFindingAmbiguity)".into());
    } else {
        atoms.push("~contradict($record, PSBindingConflict)".into());
        atoms.push("~contradict($record, PSEffectReliance)".into());
    }
    atoms.push(format!("$kind = {}", card.kind));
    for (holder, scope) in &card.fields {
        if *scope == "MandateHolder" && !holder.starts_with('$') {
            atoms.push(format!("$holder = {holder}"));
        }
    }
    for (person, scope) in &card.fields {
        if matches!(
            *scope,
            "PredeclaredAuthorizer"
                | "PredeclaredReviewer"
                | "OrdinaryAuthorizer"
                | "OrdinaryReviewer"
        ) {
            for (actor, _) in ATTESTERS.iter().chain(READERS) {
                atoms.push(format!("~({actor} = {person})"));
            }
        }
    }
    for (value, scope) in fields(card) {
        for (actor, _) in ATTESTERS {
            atoms.push(observe(actor, "$record", value, scope));
        }
    }
    for id in &card.dependencies {
        for (variable, scope) in [
            ("$record", "Record"),
            ("$revision", "Revision"),
            ("$subject", "Subject"),
            ("$holder", "Holder"),
            ("$operator", "Executor"),
            ("$source", "Source"),
            ("$window", "Window"),
            ("$start", "Start"),
            ("$end", "End"),
        ] {
            for (actor, _) in ATTESTERS {
                atoms.push(observe(
                    actor,
                    "$record",
                    &dependency_variable(id, variable),
                    &dependency_scope(id, scope),
                ));
            }
        }
    }
    atoms.extend(card.extra.clone());
    atoms
}

pub(super) fn premises(cards: &[Card], card: &Card) -> Vec<String> {
    let mut atoms = base_premises(card);
    for id in &card.dependencies {
        let parent = cards
            .iter()
            .find(|c| c.id == *id)
            .expect("declared protective dependency");
        let record = dependency_variable(id, "$record");
        let subject = dependency_variable(id, "$subject");
        atoms.push(format!("complete({record}, {}, {subject})", parent.kind));
        // Rejoin the complete raw parent contract. Completion alone may not
        // lend a version, holder, source, case or window to another order.
        for atom in base_premises(parent) {
            atoms.push(
                variable_pattern()
                    .replace_all(&atom, |m: &regex::Captures<'_>| {
                        dependency_variable(id, &m[0])
                    })
                    .into_owned(),
            );
        }
        for ancestor in &parent.dependencies {
            if card.dependencies.contains(ancestor) {
                for variable in [
                    "$record",
                    "$revision",
                    "$subject",
                    "$holder",
                    "$operator",
                    "$source",
                    "$window",
                    "$start",
                    "$end",
                ] {
                    let direct = dependency_variable(ancestor, variable);
                    let indirect = dependency_variable(id, &direct);
                    atoms.push(format!("{direct} = {indirect}"));
                }
            }
        }
        // Independent authorization/review cannot be supplied by the parent
        // power's holder, executor, decisive source or evidence writer.
        for reviewer in [
            "$authorizer",
            "$review",
            "$reader",
            "$alternate",
            "$auditor",
        ] {
            for actor in ["$holder", "$operator", "$source", "$evidence"] {
                atoms.push(format!(
                    "~({reviewer} = {})",
                    dependency_variable(id, actor)
                ));
            }
        }
        if parent.capability.is_some() || matches!(parent.id, "force-test" | "cyber-attack-basis") {
            let parent_subject = dependency_variable(id, "$subject");
            atoms.push(format!("$subject = {parent_subject}"));
        }
        for (value, scope) in &card.fields {
            if matches!(*scope, "Hazard" | "AffectedPopulation")
                && parent.fields.iter().any(|(_, s)| s == scope)
            {
                let parent_value = dependency_variable(id, value);
                atoms.push(format!("{value} = {parent_value}"));
            }
        }
    }
    atoms
}

pub(super) fn conflict_rules(cards: &[Card]) -> Vec<String> {
    let mut scopes = cards
        .iter()
        .flat_map(fields)
        .map(|(_, s)| s.to_string())
        .collect::<BTreeSet<_>>();
    for card in cards {
        for id in &card.dependencies {
            for field in [
                "Record", "Revision", "Subject", "Holder", "Executor", "Source", "Window", "Start",
                "End",
            ] {
                scopes.insert(dependency_scope(id, field));
            }
        }
    }
    let mut rules = ATTESTERS.iter().map(|(_, role)| format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, PSBindingWriter).")).collect::<Vec<_>>();
    for scope in scopes {
        let atoms = [
            "related($a, $record, PSBindingWriter)".into(),
            "related($b, $record, PSBindingWriter)".into(),
            observe("$a", "$record", "$left", &scope),
            observe("$b", "$record", "$right", &scope),
            "~($left = $right)".into(),
        ];
        rules.push(rule(&atoms, "contradict($record, PSBindingConflict)"));
        rules.push(rule(&atoms, "related($record, PSFindingAmbiguity)"));
    }
    rules
}

pub(super) fn human_subject_premises(card: &Card) -> Vec<String> {
    assert!(card.capability.is_some());
    let mut atoms = vec![
        "authorized($source, PSSourceAuthority, $record)".into(),
        "authorized($evidence, PSEvidenceAuthority, $record)".into(),
        "~($source = $evidence)".into(),
    ];
    for (value, scope) in [
        ("$subject", "CoreSubject"),
        (card.kind, "RecordKind"),
        (super::contracts::HUMAN_SUBJECT, "HumanSubjectFinding"),
    ] {
        for actor in ["$source", "$evidence"] {
            atoms.push(observe(actor, "$record", value, scope));
        }
    }
    atoms
}

pub(super) fn rules(cards: &[Card]) -> Vec<String> {
    let variants = cards
        .iter()
        .flat_map(super::temporal::variants)
        .collect::<Vec<_>>();
    let mut result = conflict_rules(&variants.iter().map(|v| v.card.clone()).collect::<Vec<_>>());
    result.extend(super::external::rules());
    for scope in [
        "SelectedEnvelopeWindow",
        "SelectedEnvelopeStart",
        "SelectedEnvelopeEnd",
        "EnvelopeContainment",
    ] {
        result.push(rule(
            &[
                "related($a, $record, PSBindingWriter)".into(),
                "related($b, $record, PSBindingWriter)".into(),
                observe("$a", "$record", "$left", scope),
                observe("$b", "$record", "$right", scope),
                "~($left = $right)".into(),
            ],
            "contradict($record, PSBindingConflict)",
        ));
    }
    for variant in &variants {
        let card = &variant.card;
        let atoms = variant.premises(cards);
        result.push(rule(
            &atoms,
            &format!("complete($record, {}, $subject)", card.kind),
        ));
    }
    for card in cards {
        // A record's direct effects consume its already established full
        // conclusion and rejoin its unambiguous own tuple. They do not need
        // another copy of every upstream court and temporal proof for every
        // output and route. Dependent powers still use premises() above, with
        // the complete raw parent contracts; nothing is replaced by a label.
        let mut own = card.clone();
        own.dependencies.clear();
        own.extra.clear();
        let mut effects = base_premises(&own);
        effects.push(format!("complete($record, {}, $subject)", card.kind));
        // The exact recorded instrument identifies who continues to owe its
        // protective duties. Invalidating it or a parent permission cannot
        // waive those duties. This body creates no completed authority or act.
        let continuing = base_premises(&own)
            .into_iter()
            .filter(|atom| {
                !matches!(
                    atom.as_str(),
                    "~contradict($record, PSBindingConflict)"
                        | "~contradict($record, PSEffectReliance)"
                        | "~related($record, PSFindingAmbiguity)"
                )
            })
            .collect::<Vec<_>>();
        for duty in &card.duties {
            result.push(rule(
                &continuing,
                &format!("obliged($operator, {duty}, $record)"),
            ));
        }
        if let Some(capability) = card.capability {
            // A positively identified human does not cease to be a person when
            // the power's currentness, grounds or lawful exercise fails.
            // This consumes independent raw human-subject evidence, not a
            // valid order, civil identity, conviction or physical-holding claim.
            result.push(rule(&human_subject_premises(card), "person($subject)"));
            result.push(rule(
                &effects,
                &format!("restrain($subject, $record, {capability})"),
            ));
            let exact = vec![
                format!("complete($record, {}, $subject)", card.kind),
                format!("restrain($subject, $record, {capability})"),
            ];
            result.push(rule(&exact, "person($subject)"));
            result.push(rule(
                &exact,
                &format!("lose({capability}, $subject, $record)"),
            ));
            if card.movement {
                result.push(rule(&exact, "restrain($subject)"));
            }
        }
    }
    result
}

pub(super) fn values(cards: &[Card], card: &Card, prefix: &str) -> BTreeMap<String, String> {
    values_with_bindings(cards, card, prefix, &BTreeMap::new())
}

fn values_with_bindings(
    cards: &[Card],
    card: &Card,
    root_prefix: &str,
    bindings: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let prefix = format!("{root_prefix}{}", card.id.replace('-', ""));
    let all = premises(cards, card).join(" ");
    let mut values = variable_pattern()
        .find_iter(&all)
        .map(|m| {
            (
                m.as_str().into(),
                format!(
                    "{prefix}{}",
                    m.as_str()
                        .trim_start_matches('$')
                        .split('_')
                        .map(|s| {
                            let (first, rest) = s.split_at(1);
                            format!("{}{rest}", first.to_ascii_uppercase())
                        })
                        .collect::<String>()
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    values.insert("$kind".into(), card.kind.into());
    values.insert("$authorization_mode".into(), "PSOrdinaryRoute".into());
    values.insert("$review_mode".into(), "PSOrdinaryRoute".into());
    values.insert(
        "$window_mode".into(),
        if super::temporal::is_measure(card.id) {
            "PSInitialDeclarationWindow"
        } else {
            "PSOwnCurrentWindow"
        }
        .into(),
    );
    values.insert("$ordinary_authorizer".into(), values["$authorizer"].clone());
    values.insert("$ordinary_reviewer".into(), values["$review"].clone());
    for link in &card.external {
        values.extend(link.defaults.clone());
    }
    super::review::default_values(card, &mut values);
    if let Some(mode) = super::temporal::default_attack_mode(card.id) {
        values.insert("$attack_mode".into(), mode.into());
    }
    values.extend(bindings.clone());
    if card.id == "transfer" && !bindings.contains_key("$transfer_kind") {
        values.insert("$transfer_kind".into(), "Extradition".into());
    }
    if let Some((holder, _)) = card
        .fields
        .iter()
        .find(|(v, s)| *s == "MandateHolder" && !v.starts_with('$'))
    {
        values.insert("$holder".into(), (*holder).into());
    }
    for id in &card.dependencies {
        let parent = cards.iter().find(|c| c.id == *id).unwrap();
        let mut inherited = BTreeMap::new();
        for variable in ["$case", "$version", "$epoch", "$jurisdiction", "$scope"] {
            inherited.insert(variable.into(), values[variable].clone());
        }
        if parent.capability.is_some() || matches!(parent.id, "force-test" | "cyber-attack-basis") {
            inherited.insert("$subject".into(), values["$subject"].clone());
        }
        for (v, s) in &card.fields {
            if matches!(*s, "Hazard" | "AffectedPopulation")
                && parent.fields.iter().any(|(_, ps)| ps == s)
            {
                inherited.insert((*v).into(), values[*v].clone());
            }
        }
        let upstream = values_with_bindings(cards, parent, root_prefix, &inherited);
        for (k, v) in upstream {
            values.insert(dependency_variable(id, &k), v);
        }
    }
    if super::temporal::is_measure(card.id) {
        for variable in ["$window", "$start", "$end"] {
            let target = dependency_variable("declaration", variable);
            assert!(values.contains_key(&target));
        }
    }
    values
}

pub(super) fn ground(text: &str, values: &BTreeMap<String, String>) -> String {
    variable_pattern()
        .replace_all(text, |m: &regex::Captures<'_>| {
            values
                .get(&m[0])
                .unwrap_or_else(|| panic!("unbound authoring variable {}", &m[0]))
                .clone()
        })
        .into_owned()
}

pub(super) fn fixture(cards: &[Card], card: &Card, values: &BTreeMap<String, String>) -> String {
    let mut facts = premises(cards, card)
        .iter()
        .filter(|a| {
            a.starts_with("authorized(") || a.starts_with("observe(") || a.starts_with("challenge(")
        })
        .map(|a| format!("{}.\n", ground(a, values)))
        .collect::<BTreeSet<_>>();
    if card.capability.is_some() {
        // Synthetic people have independent standing evidence. Withdrawing an
        // order must not erase their only supplied personhood premise.
        facts.insert(format!(
            "at({}, RepublicJurisdiction).\n",
            values["$subject"]
        ));
    }
    for id in &card.dependencies {
        let parent = cards.iter().find(|c| c.id == *id).unwrap();
        let upstream = self::values(cards, parent, "Temporary")
            .keys()
            .map(|k| (k.clone(), values[&dependency_variable(id, k)].clone()))
            .collect();
        facts.insert(fixture(cards, parent, &upstream));
    }
    if super::temporal::is_measure(card.id) {
        let id = if values["$window_mode"] == "PSRenewedDeclarationWindow" {
            "renewal"
        } else {
            "declaration"
        };
        for (variable, scope) in [
            ("$window", "SelectedEnvelopeWindow"),
            ("$start", "SelectedEnvelopeStart"),
            ("$end", "SelectedEnvelopeEnd"),
        ] {
            for (actor, _) in ATTESTERS {
                facts.insert(format!(
                    "{}.\n",
                    ground(
                        &observe(actor, "$record", &dependency_variable(id, variable), scope),
                        values
                    )
                ));
            }
        }
        for (actor, _) in ATTESTERS {
            facts.insert(format!(
                "{}.\n",
                ground(
                    &observe(
                        actor,
                        "$record",
                        "OwnIntervalWithinExactSelectedEnvelopeAndScope",
                        "EnvelopeContainment"
                    ),
                    values
                )
            ));
        }
    }
    for link in &card.external {
        facts.insert(link.fixture(values));
    }
    facts.into_iter().collect()
}
