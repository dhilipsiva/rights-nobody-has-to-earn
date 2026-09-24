// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of the ordinary half of substantive equality: the
//! bounded positive measure and its continuation finding, the reasonable
//! accommodation with its equivalent alternative, the remedial proceeding
//! opened on a pattern, and the family's defect and nonresponse routes. The
//! baseline landed fifty-four person-held barriers and no interface under
//! them; this is the interface, so the equality chapter can show a measure
//! ending and an accommodation owed rather than only a barrier holding.
//!
//! Every record needs three mutually distinct attesters, a challenge reader
//! and an independent alternate, all held apart from the acting body and from
//! the person the record is about; a continuation rejoins the exact reviewed
//! measure by decision, domain, actor, version, period, jurisdiction, scope
//! and end, so a measure reviewed for one domain continues nothing in another.
//! Nothing here establishes that a barrier existed, that an adjustment was
//! provided, that a measure achieved anything, or that a proceeding was heard.

use super::procedural_load::{
    beneficial_kinds, fast_head, single_actor, PROMPT_REVIEW, REVIEW_SCOPE, WITHDRAWN,
};
use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "book-1/source/equality-source.json";
const BEGIN: &str = "# <SUBSTANTIVE-EQUALITY-ORDINARY-RULES-BEGIN>";
const END: &str = "# <SUBSTANTIVE-EQUALITY-ORDINARY-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const REQUESTER_SEPARATION: &str = "~($requester = $actor)";
const AUTHORIZATION: &str = "ReviewedPositiveMeasure";
const ROLES: [(&str, &str); 3] = [
    ("$source", "EqualitySourceAuthority"),
    ("$evidence", "IndependentEqualityEvidenceAuthority"),
    ("$review", "IndependentEqualityReviewAuthority"),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    common_fields: Vec<[String; 2]>,
    vocabularies: Vec<(String, String, Vec<String>)>,
    contracts: Vec<Contract>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    id: String,
    kind: String,
    fields: Vec<[String; 2]>,
    heads: Vec<String>,
    extra: Vec<String>,
    record_dependency: bool,
    bindings: BTreeMap<String, String>,
}

fn fields(source: &Source, contract: &Contract) -> Vec<[String; 2]> {
    let mut result = source.common_fields.clone();
    result.push([contract.kind.clone(), "EqualityRecordKindScope".into()]);
    result.extend(contract.fields.clone());
    result
}

/// Carried through to the role a dependent record rejoins, so a participation
/// or a best-interpretation route cannot borrow a role reviewed for another
/// decision, another domain or another actor.
const CARRIED: [&str; 8] = [
    "$decision",
    "$domain",
    "$actor",
    "$version",
    "$period",
    "$jurisdiction",
    "$scope",
    "$end",
];

fn premises(source: &Source, contract: &Contract) -> Vec<String> {
    let mut atoms = ROLES
        .iter()
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    atoms.extend(
        [
            "authorized($reader, EqualityChallengeReaderAuthority, $record)",
            "authorized($alternate, EqualityAlternateReviewAuthority, $record)",
            "~($source = $evidence)",
            INDEPENDENT,
            "~($evidence = $review)",
            "~($source = $actor)",
            "~($evidence = $actor)",
            "~($review = $actor)",
            "~($reader = $alternate)",
            "~($reader = $actor)",
            "~($alternate = $actor)",
            "~($reader = $source)",
            "~($reader = $evidence)",
            "~($reader = $review)",
            "~($alternate = $source)",
            "~($alternate = $evidence)",
            "~($alternate = $review)",
            "~related($record, EqualityRecordAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // A defect must not read its own consequence negatively, and a certified
    // nonresponse concerns a duty to act rather than a surviving record.
    if !matches!(contract.id.as_str(), "defect" | "nonresponse") {
        atoms.push("~contradict($record, EqualityRecordAuthorization)".into());
    }
    for [value, scope] in fields(source, contract) {
        for (actor, _) in ROLES {
            atoms.push(format!("observe({actor}, $record, {value}, {scope})"));
        }
        if let Some((_, vocabulary, _)) = source.vocabularies.iter().find(|(s, _, _)| s == &scope) {
            atoms.push(format!("member({value}, {vocabulary})"));
        }
    }
    atoms.extend(contract.extra.clone());
    if contract.record_dependency {
        atoms.extend(
            [
                format!("complete($authorization, {AUTHORIZATION}, $decision)"),
                "authority($actor, $measure_kind, $authorization)".into(),
                "authorized($authorization_source, EqualitySourceAuthority, $authorization)"
                    .into(),
            ]
            .into_iter(),
        );
        // Rejoin the actual reviewed role, not a token naming one.
        // Authorized conflicting values on that record block its completion.
        for [value, scope] in &source.common_fields {
            if CARRIED.contains(&value.as_str()) {
                atoms.push(format!(
                    "observe($authorization_source, $authorization, {value}, {scope})"
                ));
            }
        }
    }
    atoms
}

fn rule(atoms: &[String], head: &str) -> String {
    let text = format!("{} {head}", atoms.join(" "));
    let re = Regex::new(r"\$[a-z][a-z0-9_]*").unwrap();
    let variables = re
        .find_iter(&text)
        .map(|m| m.as_str())
        .collect::<BTreeSet<_>>();
    let quantifiers = variables
        .into_iter()
        .map(|v| format!("all {v}: "))
        .collect::<String>();
    format!("{quantifiers}{} -> {head}.", atoms.join(" & "))
}

fn fast_premises(source: &Source, contract: &Contract) -> Vec<String> {
    single_actor(&premises(source, contract), &["$evidence"], "$review")
}

/// The facts of a record the source alone attests: the other attesters'
/// authorisations and observations removed, the reviewer still named.
fn single_actor_facts(facts: &str, values: &BTreeMap<String, String>) -> String {
    let evidence = &values["$evidence"];
    let review = &values["$review"];
    let record = &values["$record"];
    facts
        .lines()
        .filter(|l| {
            !l.starts_with(&format!("authorized({evidence},"))
                && !l.starts_with(&format!("observe({evidence},"))
                && !l.starts_with(&format!("observe({review}, {record},"))
        })
        .map(|l| format!("{l}\n"))
        .collect()
}

/// What follows when one actor records a beneficial record: its duties and
/// permissions, and the prompt-review duty, but not the completed record.
fn fast_queries(contract: &Contract, values: &BTreeMap<String, String>, beneficial: bool) -> String {
    let mut steps: String = heads(contract)
        .iter()
        .map(|h| query(&ground(h, values), beneficial && fast_head(h)))
        .collect();
    steps.push_str(&query(
        &ground(&format!("obliged($review, {PROMPT_REVIEW}, $record)"), values),
        beneficial,
    ));
    steps
}

fn heads(contract: &Contract) -> Vec<String> {
    let mut heads = contract.heads.clone();
    if !heads.iter().any(|h| h.starts_with("complete(")) {
        heads.push(format!("complete($record, {}, $decision)", contract.kind));
    }
    heads
}

fn rules(source: &Source, beneficial: &BTreeSet<String>) -> Vec<String> {
    let mut rules = Vec::new();
    for (scope, vocabulary, values) in &source.vocabularies {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, {scope}) -> member({value}, {vocabulary})."));
        }
    }
    let scopes = source
        .contracts
        .iter()
        .flat_map(|c| fields(source, c))
        .map(|f| f[1].clone())
        .collect::<BTreeSet<_>>();
    for scope in scopes {
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, EqualitySingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, EqualityRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, EqualityRecordAttester) & related($second, $record, EqualityRecordAttester) & member($scope, EqualitySingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, EqualityRecordAmbiguity).".into());
    for contract in &source.contracts {
        for head in heads(contract) {
            rules.push(rule(&premises(source, contract), &head));
        }
    }
    // Ruling D6: help takes effect on one authorised actor. The source alone
    // records a beneficial record; its duties and permissions follow at once,
    // the named reviewer owes prompt review, and a defect found on review
    // withdraws them. The completed record stays behind full procedure, so
    // nothing adverse can be built on help one actor gave.
    for contract in source.contracts.iter().filter(|c| beneficial.contains(&c.kind)) {
        let fast = fast_premises(source, contract);
        for head in heads(contract).iter().filter(|h| fast_head(h)) {
            rules.push(rule(&fast, head));
        }
        rules.push(rule(&fast, &format!("obliged($review, {PROMPT_REVIEW}, $record)")));
    }
    // Asking for review of a role, a refused access or a correction does not
    // require the acting body's permission and does not itself decide the
    // request.
    rules.push("all $requester: all $reader: all $request: all $decision: challenge($requester, $reader, $request) & authorized($reader, EqualityChallengeReaderAuthority, $request) & observe($requester, $request, $decision, EqualityChallengeDecisionScope) & ~($requester = $reader) -> obliged($reader, ReviewTheEqualityRequestAccessOrCorrection, $request).".into());
    rules
}

fn bindings(source: &Source, contract: &Contract, prefix: &str) -> BTreeMap<String, String> {
    let text = format!(
        "{} {}",
        premises(source, contract).join(" "),
        heads(contract).join(" ")
    );
    let re = Regex::new(r"\$[a-z][a-z0-9_]*").unwrap();
    let mut values = BTreeMap::new();
    for v in re.find_iter(&text).map(|m| m.as_str()) {
        let suffix = v
            .trim_start_matches('$')
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
            })
            .collect::<String>();
        values.insert(v.to_owned(), format!("{prefix}{suffix}"));
    }
    for [value, scope] in fields(source, contract) {
        if value.starts_with('$') {
            if let Some((_, _, options)) = source.vocabularies.iter().find(|(s, _, _)| s == &scope)
            {
                values.insert(value, options[0].clone());
            }
        }
    }
    values.extend(contract.bindings.clone());
    values
}

fn ground(text: &str, values: &BTreeMap<String, String>) -> String {
    Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .replace_all(text, |c: &regex::Captures<'_>| values[&c[0]].clone())
        .into_owned()
}

fn fixture(source: &Source, contract: &Contract, values: &BTreeMap<String, String>) -> String {
    premises(source, contract)
        .into_iter()
        .filter(|a| a.starts_with("authorized(") || a.starts_with("observe("))
        .filter(|a| !a.contains("$authorization_source") && !a.contains("$target_source"))
        .map(|a| format!("{}.\n", ground(&a, values)))
        .collect()
}

fn dependency(source: &Source, contract: &Contract, values: &BTreeMap<String, String>) -> String {
    if !contract.record_dependency {
        return premises(source, contract)
            .into_iter()
            .filter(|a| {
                a.contains("$target_source")
                    && (a.starts_with("authorized(") || a.starts_with("observe("))
            })
            .map(|a| format!("{}.\n", ground(&a, values)))
            .collect();
    }
    let authorization = source
        .contracts
        .iter()
        .find(|c| c.id == "measure")
        .expect("role contract");
    let mut other = bindings(source, authorization, "EqualityAuthorization");
    for name in CARRIED.into_iter().chain(["$measure_kind"]) {
        if let Some(value) = values.get(name) {
            other.insert(name.into(), value.clone());
        }
    }
    other.insert("$record".into(), values["$authorization"].clone());
    other.insert("$source".into(), values["$authorization_source"].clone());
    fixture(source, authorization, &other)
}

fn query(atom: &str, expected: bool) -> String {
    format!(
        "? {atom}.\n# => {}\n",
        if expected { "TRUE" } else { "FALSE" }
    )
}

fn queries(contract: &Contract, values: &BTreeMap<String, String>, expected: bool) -> String {
    heads(contract)
        .iter()
        .map(|h| query(&ground(h, values), expected))
        .collect()
}

fn add_case(
    context: &Context,
    export: &mut Export,
    id: &str,
    base: &str,
    facts: &str,
    steps: &str,
) -> Result<(), Error> {
    let count = steps
        .lines()
        .filter(|l| l.starts_with('?') || l.starts_with(":refuse") || l.starts_with(":accept"))
        .count();
    let pins = format!(":expect-pins {count}\n\n{steps}");
    export.add_case(
        context,
        &format!("equality/{id}"),
        base,
        facts,
        &[("expect", &pins)],
        vec![],
        true,
    )
}

fn without_scope(facts: &str, scope: &str) -> String {
    facts
        .lines()
        .filter(|l| !l.ends_with(&format!(", {scope}).")))
        .map(|l| format!("{l}\n"))
        .collect()
}

pub(crate) fn generate(context: &Context, export: &mut Export) -> Result<(), Error> {
    let source: Source = serde_json::from_str(&context.read(SOURCE)?)?;
    let beneficial = beneficial_kinds(context)?;
    let authored = rules(&source, &beneficial);
    let block = format!(
        "{BEGIN}\n# Supplied measures, accommodations, proceedings and reviewed findings. Nothing here certifies\n# a protected ground, a need, a disadvantage or the truth of a pattern.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "book-1/source/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous equality block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing equality end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("equality end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("equality-no-independent-review", INDEPENDENT),
        (
            "equality-no-domain-vocabulary",
            "member($domain, EqualityDomainVocabulary)",
        ),
        (
            "equality-no-measure-vocabulary",
            "member($measure_kind, EqualityMeasureKindVocabulary)",
        ),
        ("equality-no-requester-separation", REQUESTER_SEPARATION),
    ] {
        let edits = authored
            .iter()
            .filter(|r| r.contains(removed))
            .map(|r| Edit {
                before: r.clone(),
                after: r.replace(&format!(" & {removed}"), ""),
            })
            .collect::<Vec<_>>();
        if edits.is_empty() {
            return Err(Error::new(format!(
                "missing counterfactual target {removed}"
            )));
        }
        export.bases.insert(
            name.into(),
            Base {
                base: Some("live".into()),
                edits,
                path: None,
            },
        );
    }
    for contract in &source.contracts {
        let values = bindings(&source, contract, "EqualityCase");
        let facts = fixture(&source, contract, &values);
        let dep = dependency(&source, contract, &values);
        let expected = queries(contract, &values, false);
        let emit = |export: &mut Export,
                    label: &str,
                    base: &str,
                    own: &str,
                    values: &BTreeMap<String, String>,
                    expected| {
            add_case(
                context,
                export,
                &format!("{}/{label}", contract.id),
                base,
                &(dep.clone() + own),
                &queries(contract, values, expected),
            )
        };
        emit(export, "positive", "live", &facts, &values, true)?;
        emit(export, "withheld", "live", "", &values, false)?;
        for [_, scope] in fields(&source, contract) {
            let reduced = without_scope(&facts, &scope);
            if reduced == facts {
                return Err(Error::new(format!("empty equality omission {scope}")));
            }
            emit(
                export,
                &format!("without-{scope}"),
                "live",
                &reduced,
                &values,
                false,
            )?;
        }
        for (actor, _) in ROLES.into_iter().chain([
            ("$reader", "EqualityChallengeReaderAuthority"),
            ("$alternate", "EqualityAlternateReviewAuthority"),
        ]) {
            let reduced = facts
                .lines()
                .filter(|l| !l.starts_with(&format!("authorized({},", values[actor])))
                .map(|l| format!("{l}\n"))
                .collect::<String>();
            let label = format!("unauthorized-{}", actor.trim_start_matches('$'));
            if actor == "$evidence" && beneficial.contains(&contract.kind) {
                // Without the evidence attester the full route fails, and the
                // single-actor route still gives the help it carries.
                add_case(
                    context,
                    export,
                    &format!("{}/{label}", contract.id),
                    "live",
                    &(dep.clone() + &reduced),
                    &fast_queries(contract, &values, true),
                )?;
                continue;
            }
            emit(export, &label, "live", &reduced, &values, false)?;
        }
        if !matches!(contract.id.as_str(), "defect" | "nonresponse") {
            add_case(
                context,
                export,
                &format!("{}/single-actor", contract.id),
                "live",
                &(dep.clone() + &single_actor_facts(&facts, &values)),
                &fast_queries(contract, &values, beneficial.contains(&contract.kind)),
            )?;
        }
        if beneficial.contains(&contract.kind) {
            // The named reviewer's withdrawal on review switches the help off.
            let withdrawn = format!(
                "observe({}, {}, {WITHDRAWN}, {REVIEW_SCOPE}).\n",
                values["$review"], values["$record"]
            );
            add_case(
                context,
                export,
                &format!("{}/single-actor-withdrawn-on-review", contract.id),
                "live",
                &(dep.clone() + &single_actor_facts(&facts, &values) + &withdrawn),
                &fast_queries(contract, &values, false),
            )?;
        }
        for [value, scope] in fields(&source, contract) {
            if value.starts_with('$') && !contract.bindings.contains_key(&value) {
                if let Some((_, _, options)) =
                    source.vocabularies.iter().find(|(s, _, _)| s == &scope)
                {
                    for option in options
                        .iter()
                        .chain(std::iter::once(&"UnapprovedEqualityKind".into()))
                    {
                        let mut other = values.clone();
                        other.insert(value.clone(), option.clone());
                        let full = dependency(&source, contract, &other)
                            + &fixture(&source, contract, &other);
                        add_case(
                            context,
                            export,
                            &format!("{}/kind-{scope}-{option}", contract.id),
                            "live",
                            &full,
                            &queries(contract, &other, options.contains(option)),
                        )?;
                    }
                }
            }
        }
        let stale = facts.replace(
            "CurrentReconciledEqualityRecord",
            "SupersededEqualityRecord",
        );
        emit(export, "stale", "live", &stale, &values, false)?;
        for (label, variable, replacement) in [
            ("self-review", "$review", "$source"),
            ("actor-review", "$review", "$actor"),
            ("fused-alternate", "$alternate", "$reader"),
            ("source-as-challenge-reader", "$reader", "$source"),
            ("reviewer-as-alternate", "$alternate", "$review"),
        ] {
            let mut other = values.clone();
            other.insert(variable.into(), values[replacement].clone());
            let own = fixture(&source, contract, &other);
            emit(export, label, "live", &own, &other, false)?;
            if label == "self-review" {
                emit(
                    export,
                    "counterfactual-self-review",
                    "equality-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            (
                "mismatched-source",
                "EqualitySourceVersionScope",
                "$version",
            ),
            ("period-drift", "EqualityEvidencePeriodScope", "$period"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentEqualityValue, {scope})."),
                &values,
            );
            emit(
                export,
                label,
                "live",
                &facts.replace(&old, &new),
                &values,
                false,
            )?;
            let conflict = facts.clone() + &new + "\n";
            emit(
                export,
                &format!("conflicting-{label}"),
                "live",
                &conflict,
                &values,
                false,
            )?;
        }
        if contract.record_dependency {
            add_case(
                context,
                export,
                &format!("{}/without-authorization", contract.id),
                "live",
                &facts,
                &expected,
            )?;
            for (label, variable) in [
                ("wrong-decision", "$decision"),
                ("wrong-domain", "$domain"),
                ("wrong-version", "$version"),
                ("wrong-period", "$period"),
                ("wrong-measure-kind", "$measure_kind"),
            ] {
                let mut other = values.clone();
                other.insert(
                    variable.into(),
                    match variable {
                        "$domain" => "Housing".into(),
                        "$measure_kind" => "BoundedQuota".into(),
                        _ => "UnrelatedEqualityValue".into(),
                    },
                );
                emit(
                    export,
                    label,
                    "live",
                    &fixture(&source, contract, &other),
                    &other,
                    false,
                )?;
            }
        }
    }
    integration_cases(context, export, &source)?;
    Ok(())
}


fn integration_cases(context: &Context, export: &mut Export, source: &Source) -> Result<(), Error> {
    let get = |id: &str| {
        source
            .contracts
            .iter()
            .find(|c| c.id == id)
            .expect("declared contract")
    };
    // A measure is reviewed, a continuation finding runs on it, and then the
    // measure is found defective. Both stop. Nothing that happened reached the
    // standing, floor or liberty of anybody: an equality record that fails
    // takes nothing from the people it was for.
    let continuation = get("continuation");
    let values = bindings(source, continuation, "EqualitySequence");
    let full = dependency(source, continuation, &values) + &fixture(source, continuation, &values);
    let defect = get("defect");
    let mut against_measure = bindings(source, defect, "EqualityMeasureDefect");
    against_measure.insert("$target".into(), values["$authorization"].clone());
    against_measure.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    for name in CARRIED {
        against_measure.insert(name.into(), values[name].clone());
    }
    let measure = get("measure");
    let mut authorization = bindings(source, measure, "EqualityAuthorization");
    for name in CARRIED.into_iter().chain(["$measure_kind"]) {
        authorization.insert(name.into(), values[name].clone());
    }
    authorization.insert("$record".into(), values["$authorization"].clone());
    authorization.insert("$source".into(), values["$authorization_source"].clone());
    let mut steps = queries(measure, &authorization, true);
    steps.push_str(&queries(continuation, &values, true));
    steps.push_str(&fixture(source, defect, &against_measure));
    steps.push_str(&queries(defect, &against_measure, true));
    steps.push_str(&queries(measure, &authorization, false));
    steps.push_str(&queries(continuation, &values, false));
    for (atom, expected) in [
        ("person(Cira)".to_owned(), true),
        ("owe(State, Eats, Cira)".to_owned(), true),
        ("prisoner(Cira)".to_owned(), false),
        ("false(Cira)".to_owned(), false),
        (format!("person({})", values["$actor"]), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "measure/defect-and-rights-sequence",
        "live",
        &full,
        &steps,
    )?;
    // A defect about a different record leaves this one standing.
    let mut elsewhere = against_measure.clone();
    elsewhere.insert("$target".into(), "DifferentEqualityRecord".into());
    let facts = full.clone()
        + &dependency(source, defect, &elsewhere)
        + &fixture(source, defect, &elsewhere);
    add_case(
        context,
        export,
        "measure/unrelated-defect",
        "live",
        &facts,
        &(queries(continuation, &values, true) + &queries(defect, &elsewhere, true)),
    )?;
    // A measure reviewed for one domain continues nothing in another. The
    // rejoin refuses it, not the label.
    let mut other_domain = values.clone();
    other_domain.insert("$domain".into(), "Housing".into());
    add_case(
        context,
        export,
        "measure/does-not-travel-between-domains",
        "live",
        &(dependency(source, continuation, &values)
            + &fixture(source, continuation, &other_domain)),
        &queries(continuation, &other_domain, false),
    )?;
    // A measure whose reviewed record has been superseded continues nothing: a
    // calendar neither ends nor renews it, and a finding on a stale record is
    // not a finding on the measure.
    let stale = dependency(source, continuation, &values)
        .replace("CurrentReconciledEqualityRecord", "SupersededEqualityRecord")
        + &fixture(source, continuation, &values);
    add_case(
        context,
        export,
        "measure/expired-continuation-stops",
        "live",
        &stale,
        &queries(continuation, &values, false),
    )?;
    // An accommodation completes for a person the roster never entered, and
    // hands back no status, no verdict and no reusable classification.
    let accommodation = get("accommodation");
    let accommodation_values = bindings(source, accommodation, "EqualityAccommodation");
    let requester = accommodation_values["$requester"].clone();
    let mut steps = queries(accommodation, &accommodation_values, true);
    for (atom, expected) in [
        (format!("person({requester})"), false),
        (format!("false({requester})"), false),
        (format!("prisoner({requester})"), false),
        (
            format!(
                "complete({}, {AUTHORIZATION}, {})",
                accommodation_values["$record"], accommodation_values["$decision"]
            ),
            false,
        ),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "accommodation/confers-no-status",
        "live",
        &fixture(source, accommodation, &accommodation_values),
        &steps,
    )?;
    // A remedial proceeding opened on an aggregate pattern is a presumption and
    // an audit, never a verdict: the claimant and everybody else keep what the
    // record cannot take.
    let proceeding = get("proceeding");
    let proceeding_values = bindings(source, proceeding, "EqualityProceeding");
    let claimant = proceeding_values["$claimant"].clone();
    let mut steps = queries(proceeding, &proceeding_values, true);
    for (atom, expected) in [
        (format!("prisoner({claimant})"), false),
        (format!("false({claimant})"), false),
        (format!("lose(Points, {claimant})"), false),
        ("prisoner(Cira)".to_owned(), false),
        ("false(Cira)".to_owned(), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "proceeding/presumption-from-pattern",
        "live",
        &fixture(source, proceeding, &proceeding_values),
        &steps,
    )?;
    // Asking for review creates the reader's duty; a certified nonresponse
    // moves it to the alternate without deciding the request either way.
    let nonresponse = get("nonresponse");
    let mut request = bindings(source, nonresponse, "EqualityRequest");
    request.insert("$target".into(), "EqualityRequestRecord".into());
    let requester = request["$requester"].clone();
    let reader = request["$reader"].clone();
    let facts = format!(
        "challenge({requester}, {reader}, EqualityRequestRecord).\nauthorized({reader}, EqualityChallengeReaderAuthority, EqualityRequestRecord).\nobserve({requester}, EqualityRequestRecord, EqualityRequestDecision, EqualityChallengeDecisionScope).\n"
    );
    let mut steps = query(
        &format!(
            "obliged({reader}, ReviewTheEqualityRequestAccessOrCorrection, EqualityRequestRecord)"
        ),
        true,
    );
    steps.push_str(&queries(nonresponse, &request, false));
    steps.push_str(&fixture(source, nonresponse, &request));
    steps.push_str(&queries(nonresponse, &request, true));
    steps.push_str(&query(
        &format!(
            "obliged({reader}, ReviewTheEqualityRequestAccessOrCorrection, EqualityRequestRecord)"
        ),
        true,
    ));
    add_case(
        context,
        export,
        "equality/request-and-certified-nonresponse",
        "live",
        &facts,
        &steps,
    )?;
    // Every head this family concludes is declared derived-only upstream, so a
    // forged entry is refused at assertion rather than believed.
    let forged = concat!(
        ":refuse reasoning /declared derived-only/\n",
        "permits(EqualityActor, ApplyTheBoundedPositiveMeasure, EqualityForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "complete(EqualityForgedRecord, ReviewedPositiveMeasure, EqualityDecision).\n",
        ":refuse reasoning /declared derived-only/\n",
        "permits(EqualityActor, ContinueTheBoundedPositiveMeasure, EqualityForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "related(EqualityForgedRecord, EqualityRecordAmbiguity).\n",
        ":refuse reasoning /declared derived-only/\n",
        "contradict(EqualityForgedRecord, EqualityRecordAuthorization).\n",
    );
    add_case(
        context,
        export,
        "equality/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    // Ruling D6: an accommodation the source alone recorded takes effect at
    // once — the provider owes the adjustment and the person may use it — and
    // the named reviewer owes prompt review. A defect found on that review
    // withdraws the help. The completed record never derived, so nothing else
    // was built on it.
    let accommodation = get("accommodation");
    let fast = bindings(source, accommodation, "EqualityFastAccommodation");
    let fast_facts = single_actor_facts(&fixture(source, accommodation, &fast), &fast);
    let mut fast_steps = fast_queries(accommodation, &fast, true);
    let mut on_review = bindings(source, get("defect"), "EqualityFastDefect");
    on_review.insert("$target".into(), fast["$record"].clone());
    on_review.insert("$target_source".into(), fast["$source"].clone());
    for name in CARRIED {
        on_review.insert(name.into(), fast[name].clone());
    }
    fast_steps.push_str(&dependency(source, get("defect"), &on_review));
    fast_steps.push_str(&fixture(source, get("defect"), &on_review));
    fast_steps.push_str(&queries(get("defect"), &on_review, true));
    fast_steps.push_str(&fast_queries(accommodation, &fast, false));
    add_case(
        context,
        export,
        "accommodation/single-actor-corrected-on-review",
        "live",
        &fast_facts,
        &fast_steps,
    )?;
    Ok(())
}
