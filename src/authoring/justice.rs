// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit case-process authoring. No court creation or execution semantics.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

#[path = "justice_cases.rs"]
mod cases;
#[path = "justice_contracts.rs"]
mod contracts;
#[cfg(test)]
#[path = "justice_tests.rs"]
mod tests;

const BEGIN: &str = "# <NON-CARCERAL-JUSTICE-RULES-BEGIN>";
const END: &str = "# <NON-CARCERAL-JUSTICE-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const ATTESTERS: &[(&str, &str)] = &[
    ("$source", "JSourceAuthority"),
    ("$evidence", "JEvidenceAuthority"),
    ("$review", "JIndependentReviewAuthority"),
];
const READERS: &[(&str, &str)] = &[
    ("$reader", "JChallengeReaderAuthority"),
    ("$alternate", "JAlternateReviewAuthority"),
    ("$auditor", "JAuditAuthority"),
];
const JOINS: &[&str] = &[
    "Subject",
    "OtherParty",
    "Case",
    "Domain",
    "SourceVersion",
    "Epoch",
    "Jurisdiction",
    "LegalScope",
    "Window",
    "SourceBoundEnd",
];
type Values = BTreeMap<String, String>;

#[derive(Clone)]
struct Card {
    id: &'static str,
    kind: &'static str,
    fields: Vec<(&'static str, &'static str)>,
    dependencies: Vec<&'static str>,
    effects: Vec<String>,
    extra: Vec<String>,
    court: Option<(usize, &'static str)>,
}

impl Card {
    fn new(
        id: &'static str,
        kind: &'static str,
        fields: &[(&'static str, &'static str)],
        effects: &[&str],
    ) -> Self {
        Self {
            id,
            kind,
            fields: fields.to_vec(),
            effects: effects.iter().map(|s| s.to_string()).collect(),
            dependencies: vec![],
            extra: vec![],
            court: None,
        }
    }
    fn depends(mut self, id: &'static str) -> Self {
        self.dependencies.push(id);
        self
    }
    fn extra(mut self, atom: &str) -> Self {
        self.extra.push(atom.into());
        self
    }
    fn court(mut self, number: usize, key: &'static str) -> Self {
        self.court = Some((number, key));
        self.fields.push(("$court_record", "CourtRecord"));
        self
    }
}

fn common() -> Vec<(&'static str, &'static str)> {
    vec![
        ("$subject", "Subject"),
        ("$other_party", "OtherParty"),
        ("$case", "Case"),
        ("$domain", "Domain"),
        ("$version", "SourceVersion"),
        ("$epoch", "Epoch"),
        ("$jurisdiction", "Jurisdiction"),
        ("$scope", "LegalScope"),
        ("$window", "Window"),
        ("$end", "SourceBoundEnd"),
        ("$basis", "EvidenceRecord"),
        ("$operator", "Operator"),
        ("$reader", "ChallengeReader"),
        ("$alternate", "AlternateReader"),
        ("$auditor", "AuditReader"),
        ("$challenge", "Challenge"),
        ("$correction", "Correction"),
        ("$remedy", "Remedy"),
        ("CurrentReconciledFinding", "CurrentDisposition"),
        ("FreshExactCaseSourceAndWindowNoAutomaticCarry", "Carry"),
        (
            "AffirmativeCaseBoundIndependenceRecusalAndConflictReview",
            "Independence",
        ),
        (
            "PublicAccessibleReasonsPrivatePurposeBoundEvidenceRetentionAndCorrection",
            "Privacy",
        ),
        (
            "StandingFloorLibertyVoiceEqualityAndEffectiveRemedyPreserved",
            "Corridor",
        ),
        (
            "PreserveRightsEvidenceAndInterimProtectionDuringChallengeCorrectionOrEnd",
            "Continuity",
        ),
    ]
}

fn fields(card: &Card) -> Vec<(&str, &str)> {
    let mut result = common();
    result.push((card.kind, "RecordKind"));
    result.extend(card.fields.iter().copied());
    result
}

fn card<'a>(cards: &'a [Card], id: &str) -> &'a Card {
    cards
        .iter()
        .find(|c| c.id == id)
        .expect("declared justice card")
}

fn observe(actor: &str, record: &str, value: &str, scope: &str) -> String {
    format!("observe({actor}, {record}, {value}, J{scope}Scope)")
}

fn dep_var(id: &str, suffix: &str) -> String {
    format!("${}_{suffix}", id.replace('-', "_"))
}

fn joined<'a>(parent: &'a Card, dep: &'a Card) -> Vec<(&'a str, &'a str)> {
    let other = fields(dep);
    fields(parent)
        .into_iter()
        .filter(|(_, scope)| {
            JOINS.contains(scope)
                || (*scope != "CourtRecord"
                    && parent.fields.iter().any(|(_, s)| s == scope)
                    && other.iter().any(|(_, s)| s == scope))
        })
        .collect()
}

fn court_variable(variable: &str) -> String {
    match variable {
        "$subject" | "$case" | "$remedy" | "$version" | "$epoch" | "$jurisdiction" => {
            variable.into()
        }
        "$legal_scope" => "$scope".into(),
        "$invalidated_action" | "$composition_challenge" => "$target".into(),
        _ => format!("$court_{}", variable.trim_start_matches('$')),
    }
}

fn cards(context: &Context) -> Result<Vec<Card>, Error> {
    let mut cards = contracts::cards();
    for c in &mut cards {
        if let Some((number, key)) = c.court {
            for atom in super::state_form::court_consumer(context, number, key)? {
                c.extra.push(
                    variables()
                        .replace_all(&atom, |m: &regex::Captures<'_>| court_variable(&m[0]))
                        .into_owned(),
                );
            }
            for record in [
                "$court_record",
                "$court_result",
                "$court_temporal_record",
                "$court_reconciliation",
                "$court_result_reconciliation",
            ] {
                c.extra
                    .push(format!("~related({record}, JCourtBindingAmbiguity)"));
            }
            for actor in [
                "$court_source",
                "$court_evidence",
                "$court_review",
                "$court_executor",
            ] {
                for reviewer in ["$review", "$reader", "$alternate", "$auditor"] {
                    c.extra.push(format!("~({reviewer} = {actor})"));
                }
            }
        }
    }
    Ok(cards)
}

fn premises(cards: &[Card], card: &Card) -> Vec<String> {
    let mut atoms = ATTESTERS
        .iter()
        .chain(READERS)
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    let actors = [
        "$source",
        "$evidence",
        "$review",
        "$operator",
        "$reader",
        "$alternate",
        "$auditor",
    ];
    for (i, a) in actors.iter().enumerate() {
        for b in &actors[i + 1..] {
            atoms.push(format!("~({a} = {b})"));
        }
    }
    let functions = [
        "$investigator",
        "$prosecutor",
        "$defender",
        "$adjudicator",
        "$executor",
    ];
    let active = functions
        .iter()
        .filter(|f| card.fields.iter().any(|(v, _)| v == *f))
        .copied()
        .collect::<Vec<_>>();
    for (i, a) in active.iter().enumerate() {
        for b in &active[i + 1..] {
            atoms.push(format!("~({a} = {b})"));
        }
    }
    for subject in ["$subject", "$other_party"].iter().chain(active.iter()) {
        for reviewer in ["$review", "$reader", "$alternate", "$auditor"] {
            atoms.push(format!("~({reviewer} = {subject})"));
        }
    }
    atoms.push("~related($record, JRecordAmbiguity)".into());
    if !matches!(card.id, "defect" | "nonresponse" | "appeal") {
        atoms.push("~contradict($record, JEffectPermission)".into());
    }
    for (value, scope) in fields(card) {
        for (actor, _) in ATTESTERS {
            atoms.push(observe(actor, "$record", value, scope));
        }
    }
    atoms.push("member($domain, JJusticeDomain)".into());
    atoms.extend(card.extra.clone());
    for id in &card.dependencies {
        let dep = self::card(cards, id);
        let record = dep_var(id, "record");
        let source = dep_var(id, "source");
        atoms.push(format!("complete({record}, {}, $case)", dep.kind));
        atoms.push(format!("authorized({source}, JSourceAuthority, {record})"));
        for (value, scope) in joined(card, dep) {
            atoms.push(observe(&source, &record, value, scope));
        }
    }
    if matches!(card.id, "defect" | "appeal") {
        atoms.push("authorized($target_source, JSourceAuthority, $target)".into());
        atoms.push(observe(
            "$target_source",
            "$target",
            "$target_operator",
            "Operator",
        ));
        for reviewer in ["$review", "$reader", "$alternate", "$auditor"] {
            atoms.push(format!("~({reviewer} = $target_operator)"));
            for scope in [
                "Investigator",
                "Prosecutor",
                "Defender",
                "Adjudicator",
                "Executor",
            ] {
                atoms.push(format!(
                    "~{}",
                    observe("$target_source", "$target", reviewer, scope)
                ));
            }
        }
        for (value, scope) in common().into_iter().filter(|(_, s)| JOINS.contains(s)) {
            atoms.push(observe("$target_source", "$target", value, scope));
        }
    }
    atoms
}

fn heads(card: &Card) -> Vec<String> {
    let mut result = vec![format!("complete($record, {}, $case)", card.kind)];
    result.extend(card.effects.clone());
    result
}

fn variables() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\$[a-z][a-z0-9_]*").unwrap())
}

fn rule(atoms: &[String], head: &str) -> String {
    let text = format!("{} {head}", atoms.join(" "));
    let names = variables()
        .find_iter(&text)
        .map(|m| m.as_str())
        .collect::<BTreeSet<_>>();
    format!(
        "{}{} -> {head}.",
        names
            .into_iter()
            .map(|v| format!("all {v}: "))
            .collect::<String>(),
        atoms.join(" & ")
    )
}

fn rules(cards: &[Card]) -> Vec<String> {
    let mut result = Vec::new();
    for (name, values) in contracts::vocabularies() {
        for value in values {
            result.push(format!("public(Court) -> member({value}, {name})."));
        }
    }
    for barrier in contracts::BARRIERS {
        result.push(format!(
            "all $person: person($person) -> prevents($person, {barrier})."
        ));
    }
    for duty in contracts::DUTIES {
        result.push(format!(
            "all $person: person($person) -> obliged(State, {duty}, $person)."
        ));
    }
    for duty in [
        "PreserveBodilyIntegrityHumaneConditionsAndConfidentialCommunication",
        "EnsureIndependentCustodialComplaintInspectionAndConfidentialCounsel",
        "ArrangeIndependentReleaseReviewAndDoNotDelayLawfulRelease",
        "PreserveCareHousingVoiceAndReintegrationAcrossRelease",
    ] {
        result.push(format!(
            "all $person: prisoner($person) -> obliged(State, {duty}, $person)."
        ));
    }
    for scope in cards
        .iter()
        .flat_map(fields)
        .map(|(_, s)| s)
        .collect::<BTreeSet<_>>()
    {
        result.push(format!(
            "public(Court) -> member(J{scope}Scope, JSingleValueScope)."
        ));
    }
    for (_, role) in ATTESTERS {
        result.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, JRecordAttester)."));
    }
    result.push("all $a: all $b: all $record: all $scope: all $x: all $y: related($a, $record, JRecordAttester) & related($b, $record, JRecordAttester) & member($scope, JSingleValueScope) & observe($a, $record, $x, $scope) & observe($b, $record, $y, $scope) & ~($x = $y) -> related($record, JRecordAmbiguity).".into());
    // The old court shell can retain a valid tuple alongside conflicting raw
    // metadata. Justice consumers must reject ambiguity, not pick that tuple.
    // These scopes are single-valued identity bindings, unlike the court's
    // intentionally multi-valued finance/interest classifications and
    // RemedyScope (the old shell uses it for route and substantive remedy).
    for scope in [
        "SourceFamilyScope",
        "SourceVersionScope",
        "SourceEpochScope",
        "TemporalRecordScope",
        "StateFormRecordScope",
        "PowerScope",
        "JurisdictionScope",
        "AuthorityScope",
        "HolderScope",
        "ReconciliationRecordScope",
        "ResultScope",
        "EndConditionScope",
        "CaseScope",
        "SubjectScope",
        "InvalidatedActionScope",
        "CompositionChallengeScope",
        "ConstitutionalCaseScope",
        "AlternatePanelScope",
    ] {
        result.push(format!(
            "public(Court) -> member({scope}, JCourtSingleBindingScope)."
        ));
    }
    for role in [
        "StateFormSourceAuthority",
        "StateFormTemporalAuthority",
        "StateFormTemporalReviewAuthority",
        "StateFormRecordReviewAuthority",
        "StateFormEvidenceAuthority",
        "IndependentStateFormReviewAuthority",
        "InstitutionalExecutionAuthority",
    ] {
        result.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, JCourtBindingWriter)."));
    }
    for scope in ["ResultScope", "ReconciliationRecordScope"] {
        result.push(format!("all $writer: all $record: all $next: related($writer, $record, JCourtBindingWriter) & observe($writer, $record, $next, {scope}) -> related($writer, $next, JCourtBindingWriter)."));
    }
    result.push("all $a: all $b: all $record: all $scope: all $x: all $y: related($a, $record, JCourtBindingWriter) & related($b, $record, JCourtBindingWriter) & member($scope, JCourtSingleBindingScope) & observe($a, $record, $x, $scope) & observe($b, $record, $y, $scope) & ~($x = $y) -> related($record, JCourtBindingAmbiguity).".into());
    for scope in ["Subject", "OtherParty"] {
        for choice in [
            "Withdrawal",
            "Refusal",
            "CoercedConsent",
            "ConflictingConsent",
        ] {
            result.push(format!("all $party: all $source: all $record: authorized($source, JSourceAuthority, $record) & observe($source, $record, $party, J{scope}Scope) & observe($party, $record, {choice}, JParticipantChoiceScope) -> related($record, JRestorationWithdrawn)."));
        }
    }
    let request = [
        "challenge($requester, $reader, $request)",
        "authorized($reader, JChallengeReaderAuthority, $request)",
        "~($requester = $reader)",
    ]
    .map(str::to_string);
    result.push(rule(
        &request,
        "obliged($reader, ReviewJusticeRequest, $request)",
    ));
    result.push(rule(
        &request,
        "obliged(State, PreserveJusticeAccessContinuityAndNonRetaliation, $request)",
    ));
    for card in cards {
        for head in heads(card) {
            result.push(rule(&premises(cards, card), &head));
        }
    }
    result
}

fn ground(text: &str, values: &Values) -> String {
    variables()
        .replace_all(text, |m: &regex::Captures<'_>| {
            values
                .get(&m[0])
                .unwrap_or_else(|| panic!("missing justice binding {}", &m[0]))
                .clone()
        })
        .into_owned()
}

fn values(cards: &[Card], card: &Card, prefix: &str) -> Values {
    let text = format!(
        "{} {}",
        premises(cards, card).join(" "),
        heads(card).join(" ")
    );
    let mut values = Values::new();
    for variable in variables().find_iter(&text).map(|m| m.as_str()) {
        let suffix = variable
            .trim_start_matches('$')
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
            })
            .collect::<String>();
        values.insert(variable.into(), format!("{prefix}{suffix}"));
    }
    for (variable, value) in [
        ("$domain", "CivilJustice"),
        ("$relief", "Restitution"),
        ("$defect", "UnfairProcess"),
        ("$outcome", "ReasonedChargeDecision"),
    ] {
        if values.contains_key(variable) {
            values.insert(variable.into(), value.into());
        }
    }
    if card.id == "enforcement" {
        values.insert("$executor".into(), values["$operator"].clone());
    }
    values
}

fn dep_values(cards: &[Card], parent: &Card, dep: &Card, input: &Values) -> Values {
    let mut out = values(
        cards,
        dep,
        &format!("{}Dep{}", input["$record"], dep.id.replace('-', "")),
    );
    out.insert("$record".into(), input[&dep_var(dep.id, "record")].clone());
    out.insert("$source".into(), input[&dep_var(dep.id, "source")].clone());
    for (value, scope) in joined(parent, dep) {
        let upstream = fields(dep)
            .into_iter()
            .find(|(_, s)| *s == scope)
            .unwrap()
            .0;
        if upstream.starts_with('$') {
            out.insert(upstream.into(), ground(value, input));
        }
    }
    out
}

fn fixture(
    context: &Context,
    cards: &[Card],
    card: &Card,
    values: &Values,
) -> Result<String, Error> {
    let mut result = String::new();
    // Only our admitted raw facts; upstream witnesses come from their actual
    // authoring implementation below, never from asserting a completion.
    for atom in premises(cards, card).into_iter().collect::<BTreeSet<_>>() {
        let own_observation = atom.starts_with("observe(")
            && atom
                .rsplit_once(", ")
                .is_some_and(|(_, scope)| scope.starts_with('J') && scope.ends_with("Scope)"));
        if (atom.starts_with("authorized(") && atom.contains(", J")
            || own_observation
            || atom.starts_with("challenge("))
            && !card
                .dependencies
                .iter()
                .any(|id| atom.contains(&dep_var(id, "source")))
        {
            result += &format!("{}.\n", ground(&atom, values));
        }
    }
    if let Some((number, key)) = card.court {
        let atoms = super::state_form::court_consumer(context, number, key)?.join(" ");
        let names = variables()
            .find_iter(&atoms)
            .map(|m| m.as_str())
            .collect::<BTreeSet<_>>();
        let owned = names
            .into_iter()
            .filter_map(|v| {
                values
                    .get(&court_variable(v))
                    .map(|value| (v.to_owned(), value.clone()))
            })
            .collect::<Vec<_>>();
        let bindings = owned
            .iter()
            .map(|(v, value)| (v.as_str(), value.as_str()))
            .collect::<Vec<_>>();
        result += &super::state_form::court_example(
            context,
            number,
            key,
            &format!("{}Court", values["$record"]),
            &bindings,
        )?
        .0;
    }
    for id in &card.dependencies {
        let dep = self::card(cards, id);
        result += &fixture(context, cards, dep, &dep_values(cards, card, dep, values))?;
    }
    Ok(result)
}

fn query(atom: &str, expected: bool) -> String {
    format!(
        "? {atom}.\n# => {}\n",
        if expected { "TRUE" } else { "FALSE" }
    )
}
fn queries(card: &Card, values: &Values, expected: bool) -> String {
    heads(card)
        .iter()
        .map(|h| query(&ground(h, values), expected))
        .collect()
}

fn add_case(
    context: &Context,
    export: &mut Export,
    id: &str,
    base: &str,
    fixture: &str,
    steps: &str,
) -> Result<(), Error> {
    let count = steps
        .lines()
        .filter(|l| l.starts_with('?') || l.starts_with(":accept") || l.starts_with(":refuse"))
        .count();
    export.add_case(
        context,
        &format!("justice/{id}"),
        base,
        fixture,
        &[("expect", &format!(":expect-pins {count}\n\n{steps}"))],
        vec![],
        true,
    )
}

pub(crate) fn generate(context: &Context, export: &mut Export) -> Result<(), Error> {
    let cards = cards(context)?;
    let authored = rules(&cards);
    let block = format!(
        "{BEGIN}\n# Case-process findings and duties; no court holder, coercive power, verdict or actual remedy is created.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "new-book-plans/constitution.nibli";
    let old = context.read(path)?;
    let updated = match (old.matches(BEGIN).count(), old.matches(END).count()) {
        (0, 0) => format!("{}\n\n{block}\n", old.trim_end()),
        (1, 1) => {
            let (before, rest) = old.split_once(BEGIN).unwrap();
            let (_, after) = rest
                .split_once(END)
                .ok_or_else(|| Error::new("justice markers out of order"))?;
            format!("{before}{block}{after}")
        }
        _ => return Err(Error::new("ambiguous justice rule block")),
    };
    std::fs::write(context.path(path), updated)?;
    cases::generate(context, export, &cards, &authored)
}
