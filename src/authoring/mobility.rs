// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit semantic authoring, never a verification-time generator or gate.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

#[path = "mobility_cases.rs"]
mod cases;
#[path = "mobility_contracts.rs"]
mod contracts;

const BEGIN: &str = "# <MOBILITY-PLURALITY-RULES-BEGIN>";
const END: &str = "# <MOBILITY-PLURALITY-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const SHARED: &[&str] = &[
    "Holder",
    "Object",
    "SourceVersion",
    "Epoch",
    "Jurisdiction",
    "LegalScope",
    "Window",
];
const ATTESTERS: &[(&str, &str)] = &[
    ("$source", "MPSourceAuthority"),
    ("$evidence", "MPEvidenceAuthority"),
    ("$review", "MPIndependentReviewAuthority"),
];
const READERS: &[(&str, &str)] = &[
    ("$reader", "MPChallengeReaderAuthority"),
    ("$alternate", "MPAlternateReviewAuthority"),
    ("$auditor", "MPAuditAuthority"),
];
type Values = BTreeMap<String, String>;

#[derive(Clone)]
struct Card {
    id: &'static str,
    kind: &'static str,
    fields: Vec<(&'static str, &'static str)>,
    dependencies: Vec<&'static str>,
    extra: Vec<String>,
    effects: Vec<String>,
}

impl Card {
    fn new(
        id: &'static str,
        kind: &'static str,
        fields: &[(&'static str, &'static str)],
        dependencies: &[&'static str],
        effects: &[&str],
    ) -> Self {
        Self {
            id,
            kind,
            fields: fields.to_vec(),
            dependencies: dependencies.to_vec(),
            extra: vec![],
            effects: effects.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn extra(mut self, atom: &str) -> Self {
        self.extra.push(atom.into());
        self
    }
}

fn common() -> Vec<(&'static str, &'static str)> {
    vec![
        ("$holder", "Holder"),
        ("$object", "Object"),
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
        ("FreshExactSourceAndWindowNoAutomaticCarry", "Carry"),
        (
            "AffirmativeCaseBoundIndependenceAndConflictReview",
            "Independence",
        ),
        (
            "MinimalPrivatePurposeBoundEvidenceNoWorthRiskOrEligibilityReuse",
            "Privacy",
        ),
        (
            "IndividualStandingFloorLibertyEqualityVoiceBallotAndRemedyIntact",
            "Corridor",
        ),
        (
            "IndependentConstitutionalCourtReviewOrLawfulSubstitute",
            "FinalReview",
        ),
        (
            "PreserveRecordProvenanceAndRightsDuringChallengeCorrectionOrEnd",
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
    cards.iter().find(|c| c.id == id).expect("declared card")
}

fn record_var(id: &str) -> String {
    format!("${}_record", id.replace('-', "_"))
}

fn observation(actor: &str, record: &str, value: &str, scope: &str) -> String {
    format!("observe({actor}, {record}, {value}, MP{scope}Scope)")
}

fn joins<'a>(consumer: &'a Card, dependency: &'a Card) -> Vec<(&'a str, &'a str)> {
    let upstream = fields(dependency);
    fields(consumer)
        .into_iter()
        .filter(|(_, scope)| {
            SHARED.contains(scope)
                || (consumer.fields.iter().any(|(_, s)| s == scope)
                    && upstream.iter().any(|(_, s)| s == scope))
        })
        .collect()
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
    for (i, left) in actors.iter().enumerate() {
        for right in &actors[i + 1..] {
            atoms.push(format!("~({left} = {right})"));
        }
    }
    // Being distinct from the operator is not enough: the interested holder
    // (or a named representative/requester) cannot independently review itself.
    let mut interested = vec!["$holder"];
    if card
        .fields
        .iter()
        .any(|(value, _)| *value == "$representative")
    {
        interested.push("$representative");
    }
    if matches!(card.id, "private-access" | "nonresponse") {
        interested.push("$requester");
    }
    for subject in interested {
        for reviewer in ["$review", "$reader", "$alternate", "$auditor"] {
            atoms.push(format!("~({reviewer} = {subject})"));
        }
    }
    atoms.push("~related($record, MPRecordAmbiguity)".into());
    if !matches!(card.id, "defect" | "nonresponse") {
        atoms.push("~contradict($record, MPEffectPermission)".into());
    }
    for (value, scope) in fields(card) {
        for (actor, _) in ATTESTERS {
            atoms.push(observation(actor, "$record", value, scope));
        }
    }
    atoms.extend(card.extra.clone());
    for dep in &card.dependencies {
        let dep = self::card(cards, dep);
        let record = record_var(dep.id);
        let source = format!("${}_source", dep.id.replace('-', "_"));
        atoms.push(format!("complete({record}, {}, $object)", dep.kind));
        atoms.push(format!("authorized({source}, MPSourceAuthority, {record})"));
        for (value, scope) in joins(card, dep) {
            atoms.push(observation(&source, &record, value, scope));
        }
    }
    if card.id == "consent" {
        let roles = [
            ("$administration", "MPDecisionAdministrationAuthority"),
            ("$assurer", "MPCompletenessAssuranceAuthority"),
            ("$result_service", "MPResultServiceAuthority"),
        ];
        for (actor, role) in roles {
            atoms.push(format!("authorized({actor}, {role}, $record)"));
            for (value, scope) in fields(card) {
                atoms.push(observation(actor, "$record", value, scope));
            }
            for other in actors.iter().chain(["$representative"].iter()) {
                atoms.push(format!("~({actor} = {other})"));
            }
        }
        atoms.extend(
            [
                "~($administration = $assurer)",
                "~($administration = $result_service)",
                "~($assurer = $result_service)",
            ]
            .map(str::to_string),
        );
    }
    if card.id == "defect" {
        // Rejoin raw target identity, never its completion: otherwise the
        // permission -> defect -> no-permission dependency is a negative cycle.
        atoms.push("authorized($target_source, MPSourceAuthority, $target)".into());
        for (value, scope) in common().into_iter().filter(|(_, s)| SHARED.contains(s)) {
            atoms.push(observation("$target_source", "$target", value, scope));
        }
    }
    if card.id == "nonresponse" {
        atoms.extend(
            [
                "challenge($requester, $reader, $target)",
                "authorized($reader, MPChallengeReaderAuthority, $target)",
                "observe($requester, $target, $object, MPRequestedObjectScope)",
                "~($requester = $reader)",
                "~($requester = $alternate)",
            ]
            .map(str::to_string),
        );
    }
    atoms
}

fn heads(card: &Card) -> Vec<String> {
    let mut result = vec![format!("complete($record, {}, $object)", card.kind)];
    result.extend(card.effects.clone());
    result
}

fn rule(atoms: &[String], head: &str) -> String {
    let text = format!("{} {head}", atoms.join(" "));
    let re = Regex::new(r"\$[a-z][a-z0-9_]*").unwrap();
    let variables = re
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

fn vocabularies() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("MPConsentRequiredEffect", contracts::HARMS.to_vec()),
        (
            "MPCommunityKind",
            vec!["IndigenousPeople", "MinorityCommunity"],
        ),
        (
            "MPTransferKind",
            vec![
                "IndividualExpulsion",
                "Extradition",
                "Surrender",
                "Handover",
            ],
        ),
        (
            "MPReliefKind",
            vec![
                "CollectiveRestitution",
                "TitleRestoration",
                "IndividualRelief",
                "CompensationAndReparation",
                "BoundedStructuralNonRepetition",
            ],
        ),
        (
            "MPExternalInstrument",
            vec![
                "PublicProcurement",
                "PublicInvestment",
                "ExternalTrade",
                "CorporateForm",
                "Contracting",
                "Affiliate",
                "SupplyChain",
                "FlagOfConvenience",
                "ArbitrationForum",
                "ExportedEnforcement",
                "JurisdictionShopping",
            ],
        ),
        (
            "MPDefectKind",
            vec![
                "MembershipDispute",
                "UnlawfulAcceptance",
                "ForcedAssimilation",
                "InternalRightsViolation",
                "NonmemberExclusion",
                "UnlawfulDispossession",
                "ConsentCoercionOrBreach",
                "MissingActualConsent",
                "ConsultationAfterCommitment",
                "RefoulementOrCategoricalTransferBar",
                "UnlawfulImmigrationDetention",
                "EnforcementDataReuse",
                "ExportedLabourExploitation",
                "ExportedEcologicalDamage",
                "ExportedRightsViolation",
                "ExpiredOrSupersededEvidence",
                "PrivacyBreach",
                "ReaderConflict",
            ],
        ),
    ]
}

fn rules(cards: &[Card]) -> Vec<String> {
    let mut result = contracts::BARRIERS
        .iter()
        .map(|kind| format!("all $holder: person($holder) -> prevents($holder, {kind})."))
        .collect::<Vec<_>>();
    for duty in [
        "ServeFirstNoImmigrationDocumentationOrEnforcementCondition",
        "KeepStandingFloorLibertyLanguageProcessAndRemedyContinuous",
        "IncludeEveryoneUnderJurisdictionOrEffectiveControlInScarcity",
    ] {
        result.push(format!(
            "all $holder: person($holder) -> obliged(State, {duty}, $holder)."
        ));
    }
    // Community labels are bounded findings, never human personhood.
    for (vocabulary, values) in vocabularies() {
        for value in values {
            result.push(format!("public(Court) -> member({value}, {vocabulary})."));
        }
    }
    for scope in cards
        .iter()
        .flat_map(fields)
        .map(|(_, scope)| scope)
        .collect::<BTreeSet<_>>()
    {
        result.push(format!(
            "public(Court) -> member(MP{scope}Scope, MPSingleValueScope)."
        ));
    }
    for role in ATTESTERS.iter().map(|(_, r)| *r).chain([
        "MPDecisionAdministrationAuthority",
        "MPCompletenessAssuranceAuthority",
        "MPResultServiceAuthority",
    ]) {
        result.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, MPRecordAttester)."));
    }
    result.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, MPRecordAttester) & related($second, $record, MPRecordAttester) & member($scope, MPSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, MPRecordAmbiguity).".into());
    result.push("all $holder: all $record: observe($holder, $record, FreeMembershipExit, MPSelfExitScope) -> related($holder, $record, MPExitedMembership).".into());
    result.push("all $holder: all $record: observe($holder, $record, FreeMembershipExit, MPSelfExitScope) -> obliged(State, RecordExitWithoutRightsOrTitlePenalty, $record).".into());
    result.push("all $requester: all $reader: all $request: all $object: challenge($requester, $reader, $request) & authorized($reader, MPChallengeReaderAuthority, $request) & observe($requester, $request, $object, MPRequestedObjectScope) & ~($requester = $reader) -> obliged($reader, ReviewMobilityPluralityRequest, $request).".into());
    result.push("all $requester: all $reader: all $request: all $object: challenge($requester, $reader, $request) & authorized($reader, MPChallengeReaderAuthority, $request) & observe($requester, $request, $object, MPRequestedObjectScope) & ~($requester = $reader) -> obliged(State, PreserveContinuityPrivacyAndNonRetaliationDuringReview, $request).".into());
    for card in cards {
        for head in heads(card) {
            result.push(rule(&premises(cards, card), &head));
        }
    }
    result
}

fn ground(text: &str, values: &Values) -> String {
    Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .replace_all(text, |c: &regex::Captures<'_>| {
            values
                .get(&c[0])
                .unwrap_or_else(|| panic!("missing binding {}", &c[0]))
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
    for variable in Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .find_iter(&text)
        .map(|m| m.as_str())
    {
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
    for (variable, default) in [
        ("$group_kind", "IndigenousPeople"),
        ("$effect", contracts::HARMS[0]),
        ("$transfer", "IndividualExpulsion"),
        ("$relief", "CollectiveRestitution"),
        ("$instrument", "PublicProcurement"),
        ("$defect", "MembershipDispute"),
    ] {
        if values.contains_key(variable) {
            values.insert(variable.into(), default.into());
        }
    }
    values
}

fn own_facts(cards: &[Card], card: &Card, values: &Values) -> String {
    let dependency_sources = card
        .dependencies
        .iter()
        .map(|id| format!("${}_source", id.replace('-', "_")))
        .collect::<Vec<_>>();
    premises(cards, card)
        .into_iter()
        .filter(|a| {
            (a.starts_with("authorized(")
                || a.starts_with("observe(")
                || a.starts_with("challenge("))
                && !dependency_sources.iter().any(|s| a.contains(s))
                && !a.contains("$target_source")
        })
        .map(|a| format!("{}.\n", ground(&a, values)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn dependency_values(cards: &[Card], parent: &Card, dep: &Card, input: &Values) -> Values {
    let mut output = values(
        cards,
        dep,
        &format!("{}Dep{}", input["$record"], dep.id.replace('-', "")),
    );
    output.insert("$record".into(), input[&record_var(dep.id)].clone());
    output.insert(
        "$source".into(),
        input[&format!("${}_source", dep.id.replace('-', "_"))].clone(),
    );
    for (value, scope) in joins(parent, dep) {
        let upstream = fields(dep)
            .into_iter()
            .find(|(_, s)| *s == scope)
            .unwrap()
            .0;
        if upstream.starts_with('$') {
            output.insert(upstream.into(), ground(value, input));
        }
    }
    output
}

fn fixture(cards: &[Card], card: &Card, values: &Values) -> String {
    let mut result = own_facts(cards, card, values);
    for id in &card.dependencies {
        let dep = self::card(cards, id);
        result += &fixture(cards, dep, &dependency_values(cards, card, dep, values));
    }
    if card.id == "defect" {
        for atom in premises(cards, card)
            .into_iter()
            .filter(|a| a.contains("$target_source"))
        {
            result += &format!("{}.\n", ground(&atom, values));
        }
    }
    result
}

fn query(atom: &str, expected: bool) -> String {
    format!(
        "? {atom}.\n# => {}\n",
        if expected { "TRUE" } else { "FALSE" }
    )
}

/// Authoring seam for an exact protective-power consumer. Retains the actual
/// mobility/plurality premises; no compatibility tag is asserted as evidence.
pub(crate) fn protective_consumer(id: &str) -> Result<Vec<String>, Error> {
    if !matches!(
        id,
        "asylum" | "removal" | "consented-effect" | "evacuation" | "external" | "defect"
    ) {
        return Err(Error::new("undeclared protective mobility interface"));
    }
    let cards = contracts::cards();
    let card = self::card(&cards, id);
    let mut atoms = premises(&cards, card);
    atoms.push(format!("complete($record, {}, $object)", card.kind));
    Ok(atoms)
}

pub(crate) fn protective_example(
    id: &str,
    prefix: &str,
    bindings: &[(&str, &str)],
) -> Result<(String, Values), Error> {
    protective_consumer(id)?;
    let cards = contracts::cards();
    let card = self::card(&cards, id);
    let mut values = self::values(&cards, card, prefix);
    for (variable, value) in bindings {
        if !values.contains_key(*variable) {
            return Err(Error::new(format!("unknown mobility binding {variable}")));
        }
        values.insert((*variable).into(), (*value).into());
    }
    Ok((fixture(&cards, card, &values), values))
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
        &format!("mobility/{id}"),
        base,
        fixture,
        &[("expect", &format!(":expect-pins {count}\n\n{steps}"))],
        vec![],
        true,
    )
}

pub(crate) fn generate(context: &Context, export: &mut Export) -> Result<(), Error> {
    let cards = contracts::cards();
    let authored = rules(&cards);
    let block = format!(
        "{BEGIN}\n# Opaque, bounded supplied findings; no identity, tally, clock or institutional act is computed.\n{}\n{END}",
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
                .ok_or_else(|| Error::new("mobility markers out of order"))?;
            format!("{before}{block}{after}")
        }
        _ => return Err(Error::new("ambiguous mobility rule block")),
    };
    std::fs::write(context.path(path), updated)?;
    cases::generate(context, export, &cards, &authored)
}

#[cfg(test)]
#[path = "mobility_tests.rs"]
mod tests;
