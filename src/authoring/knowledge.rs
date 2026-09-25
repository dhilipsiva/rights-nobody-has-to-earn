// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of knowledge, communication, culture and the free social
//! field: what a public actor must establish before it may restrict, and what
//! it owes when it does not.

use super::procedural_load::{
    beneficial_kinds, fast_head, single_actor, PROMPT_REVIEW, REVIEW_SCOPE, WITHDRAWN,
};
use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "book-1/source/knowledge-source.json";
const BEGIN: &str = "# <KNOWLEDGE-AND-FREE-FIELD-RULES-BEGIN>";
const END: &str = "# <KNOWLEDGE-AND-FREE-FIELD-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const HOLDER_SEPARATION: &str = "~($holder = $actor)";
const AUTHORIZATION: &str = "ReviewedKnowledgeRestriction";
const ROLES: [(&str, &str); 3] = [
    ("$source", "KnowledgeSourceAuthority"),
    ("$evidence", "IndependentKnowledgeEvidenceAuthority"),
    ("$review", "IndependentKnowledgeReviewAuthority"),
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
    result.push([contract.kind.clone(), "KnowledgeRecordKindScope".into()]);
    result.extend(contract.fields.clone());
    result
}

/// Carried through to the authorization a dependent record rejoins, so an
/// enforcement cannot borrow a restriction reviewed for another matter.
const CARRIED: [&str; 8] = [
    "$matter",
    "$field",
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
            "authorized($reader, KnowledgeChallengeReaderAuthority, $record)",
            "authorized($alternate, KnowledgeAlternateReviewAuthority, $record)",
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
            "~related($record, KnowledgeRecordAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // A defect must not read its own consequence negatively, and a certified
    // nonresponse concerns a duty to act rather than a surviving record.
    if !matches!(contract.id.as_str(), "defect" | "nonresponse") {
        atoms.push("~contradict($record, KnowledgeRecordAuthorization)".into());
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
                format!("complete($authorization, {AUTHORIZATION}, $matter)"),
                "authority($actor, $restriction_kind, $authorization)".into(),
                "authorized($authorization_source, KnowledgeSourceAuthority, $authorization)"
                    .into(),
            ]
            .into_iter(),
        );
        // Rejoin the actual reviewed restriction, not a token naming one.
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

/// What follows when one actor records a record: for a beneficial record its
/// duties and permissions and the prompt-review duty, never the completed
/// record; for anything else, nothing.
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
        heads.push(format!("complete($record, {}, $matter)", contract.kind));
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
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, KnowledgeSingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, KnowledgeRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, KnowledgeRecordAttester) & related($second, $record, KnowledgeRecordAttester) & member($scope, KnowledgeSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, KnowledgeRecordAmbiguity).".into());
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
    // Asking for review of a restriction, an access refusal or a correction
    // does not require the restricting actor's permission and does not itself
    // decide the request.
    rules.push("all $requester: all $reader: all $request: all $matter: challenge($requester, $reader, $request) & authorized($reader, KnowledgeChallengeReaderAuthority, $request) & observe($requester, $request, $matter, KnowledgeChallengeMatterScope) & ~($requester = $reader) -> obliged($reader, ReviewKnowledgeRestrictionAccessOrCorrection, $request).".into());
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
        .find(|c| c.id == "restriction")
        .expect("restriction contract");
    let mut other = bindings(source, authorization, "KnowledgeAuthorization");
    for name in CARRIED.into_iter().chain(["$restriction_kind"]) {
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
        &format!("knowledge/{id}"),
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
        "{BEGIN}\n# Supplied permissions, duties and reviewed findings. Nothing here certifies\n# truth, taste, belief, creativity, a relationship or personal fulfilment.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "book-1/source/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous knowledge block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing knowledge end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("knowledge end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("knowledge-no-independent-review", INDEPENDENT),
        (
            "knowledge-no-field-vocabulary",
            "member($field, KnowledgeFieldVocabulary)",
        ),
        (
            "knowledge-no-harm-vocabulary",
            "member($harm, KnowledgeHarmKindVocabulary)",
        ),
        ("knowledge-no-holder-separation", HOLDER_SEPARATION),
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
        let values = bindings(&source, contract, "KnowledgeCase");
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
                return Err(Error::new(format!("empty knowledge omission {scope}")));
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
            ("$reader", "KnowledgeChallengeReaderAuthority"),
            ("$alternate", "KnowledgeAlternateReviewAuthority"),
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
        add_case(
            context,
            export,
            &format!("{}/single-actor", contract.id),
            "live",
            &(dep.clone() + &single_actor_facts(&facts, &values)),
            &fast_queries(contract, &values, beneficial.contains(&contract.kind)),
        )?;
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
                        .chain(std::iter::once(&"UnapprovedKnowledgeKind".into()))
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
            "CurrentReconciledKnowledgeRecord",
            "SupersededKnowledgeRecord",
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
                    "knowledge-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            (
                "mismatched-source",
                "KnowledgeSourceVersionScope",
                "$version",
            ),
            ("period-drift", "KnowledgeEvidencePeriodScope", "$period"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentKnowledgeValue, {scope})."),
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
                ("wrong-matter", "$matter"),
                ("wrong-field", "$field"),
                ("wrong-version", "$version"),
                ("wrong-period", "$period"),
                ("wrong-restriction-kind", "$restriction_kind"),
            ] {
                let mut other = values.clone();
                other.insert(
                    variable.into(),
                    match variable {
                        "$field" => "ArtisticAndCulturalCreation".into(),
                        "$restriction_kind" => "ReviewedAccessCondition".into(),
                        _ => "UnrelatedKnowledgeValue".into(),
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

    // A restriction that is reviewed, then found defective, stops. Nothing it
    // touched ever reached the person's standing, floor, ballot or speech.
    let enforcement = get("enforcement");
    let values = bindings(source, enforcement, "KnowledgeSequence");
    let participant = values["$affected"].clone();
    let full = dependency(source, enforcement, &values)
        + &fixture(source, enforcement, &values)
        + &format!("person({participant}).\nat({participant}, GeneralAdulthood).\n");
    let defect = get("defect");
    let mut against_restriction = bindings(source, defect, "KnowledgeRestrictionDefect");
    against_restriction.insert("$target".into(), values["$authorization"].clone());
    against_restriction.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    for name in CARRIED {
        against_restriction.insert(name.into(), values[name].clone());
    }
    let restriction = get("restriction");
    let mut authorization = bindings(source, restriction, "KnowledgeAuthorization");
    for name in CARRIED.into_iter().chain(["$restriction_kind"]) {
        authorization.insert(name.into(), values[name].clone());
    }
    authorization.insert("$record".into(), values["$authorization"].clone());
    authorization.insert("$source".into(), values["$authorization_source"].clone());

    let mut steps = queries(restriction, &authorization, true);
    steps.push_str(&queries(enforcement, &values, true));
    steps.push_str(&fixture(source, defect, &against_restriction));
    steps.push_str(&queries(defect, &against_restriction, true));
    steps.push_str(&queries(restriction, &authorization, false));
    steps.push_str(&queries(enforcement, &values, false));
    for (atom, expected) in [
        (format!("person({participant})"), true),
        (format!("owe(State, Eats, {participant})"), true),
        (format!("decide({participant}, Ballot)"), true),
        (format!("expresses({participant})"), false),
        (format!("false({participant})"), false),
        (format!("prisoner({participant})"), false),
        (format!("lose(Points, {participant})"), false),
        (
            format!("prevents({participant}, CompelledConscience)"),
            true,
        ),
        (format!("prevents({participant}, CoercedAssociation)"), true),
        ("person(Hano)".into(), true),
        ("owe(State, Eats, Hano)".into(), true),
        ("decide(Hano, Ballot)".into(), true),
        ("authority(Tove)".into(), true),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "free-field/restriction-defect-and-rights-sequence",
        "live",
        &full,
        &steps,
    )?;

    // A defect about a different record leaves this one standing.
    let mut elsewhere = against_restriction.clone();
    elsewhere.insert("$target".into(), "DifferentKnowledgeRecord".into());
    let facts = full.clone()
        + &dependency(source, defect, &elsewhere)
        + &fixture(source, defect, &elsewhere);
    add_case(
        context,
        export,
        "free-field/unrelated-defect",
        "live",
        &facts,
        &(queries(enforcement, &values, true) + &queries(defect, &elsewhere, true)),
    )?;

    // A reviewed restriction for one field does not license enforcement in
    // another. The rejoin is what refuses it, not the label.
    let mut other_field = values.clone();
    other_field.insert("$field".into(), "ArtisticAndCulturalCreation".into());
    add_case(
        context,
        export,
        "free-field/restriction-does-not-travel-between-fields",
        "live",
        &(dependency(source, enforcement, &values) + &fixture(source, enforcement, &other_field)),
        &queries(enforcement, &other_field, false),
    )?;

    // Residual freedom: the duties land on the public actor, and nothing about
    // the activity becomes a permission to restrict it.
    let residual = get("residual");
    let residual_values = bindings(source, residual, "KnowledgeResidual");
    let holder = residual_values["$holder"].clone();
    let mut steps = queries(residual, &residual_values, true);
    for (atom, expected) in [
        (
            format!(
                "authority({}, ReviewedConductRestriction, {})",
                residual_values["$actor"], residual_values["$record"]
            ),
            false,
        ),
        (
            format!(
                "permits({}, EnforceReviewedKnowledgeRestriction, {})",
                residual_values["$actor"], residual_values["$record"]
            ),
            false,
        ),
        (
            format!(
                "complete({}, {AUTHORIZATION}, {})",
                residual_values["$record"], residual_values["$matter"]
            ),
            false,
        ),
        (format!("person({holder})"), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "free-field/residual-freedom-licenses-no-restriction",
        "live",
        &fixture(source, residual, &residual_values),
        &steps,
    )?;
    add_case(
        context,
        export,
        "free-field/counterfactual-holder-is-the-actor",
        "knowledge-no-holder-separation",
        &fixture(source, residual, &{
            let mut fused = residual_values.clone();
            fused.insert("$holder".into(), residual_values["$actor"].clone());
            fused
        }),
        &queries(
            residual,
            &{
                let mut fused = residual_values.clone();
                fused.insert("$holder".into(), residual_values["$actor"].clone());
                fused
            },
            true,
        ),
    )?;

    // A plurality finding is structural. It concludes nothing about content,
    // and nothing about a reader.
    let plurality = get("plurality");
    let plurality_values = bindings(source, plurality, "KnowledgePlurality");
    let mut steps = queries(plurality, &plurality_values, true);
    for (atom, expected) in [
        (
            format!(
                "complete({}, ReviewedKnowledgeRestriction, {})",
                plurality_values["$record"], plurality_values["$matter"]
            ),
            false,
        ),
        (
            format!(
                "authority({}, ReviewedConductRestriction, {})",
                plurality_values["$actor"], plurality_values["$record"]
            ),
            false,
        ),
        (format!("person({})", plurality_values["$service"]), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "free-field/concentration-finding-judges-no-content",
        "live",
        &fixture(source, plurality, &plurality_values),
        &steps,
    )?;

    // Asking for review creates the reader's duty; a certified nonresponse
    // moves it to the alternate without deciding the request either way.
    let nonresponse = get("nonresponse");
    let mut request = bindings(source, nonresponse, "KnowledgeRequest");
    request.insert("$target".into(), "KnowledgeRequestRecord".into());
    let requester = request["$requester"].clone();
    let reader = request["$reader"].clone();
    let facts = format!(
        "challenge({requester}, {reader}, KnowledgeRequestRecord).\nauthorized({reader}, KnowledgeChallengeReaderAuthority, KnowledgeRequestRecord).\nobserve({requester}, KnowledgeRequestRecord, KnowledgeRequestMatter, KnowledgeChallengeMatterScope).\n"
    );
    let mut steps = query(
        &format!(
            "obliged({reader}, ReviewKnowledgeRestrictionAccessOrCorrection, KnowledgeRequestRecord)"
        ),
        true,
    );
    steps.push_str(&queries(nonresponse, &request, false));
    steps.push_str(&fixture(source, nonresponse, &request));
    steps.push_str(&queries(nonresponse, &request, true));
    steps.push_str(&query(
        &format!(
            "obliged({reader}, ReviewKnowledgeRestrictionAccessOrCorrection, KnowledgeRequestRecord)"
        ),
        true,
    ));
    add_case(
        context,
        export,
        "free-field/request-and-certified-nonresponse",
        "live",
        &facts,
        &steps,
    )?;

    // An entry that merely names somebody's belief, opinion or club is inert.
    // The existing person-held barriers are untouched by this family.
    let note = concat!(
        "person(KnowledgeNoted).\n",
        "at(KnowledgeNoted, GeneralAdulthood).\n",
        "observe(KnowledgeNoteWriter, KnowledgeNoteRecord, KnowledgeNoted, KnowledgeObservedBeliefScope).\n",
        "observe(KnowledgeNoteWriter, KnowledgeNoteRecord, KnowledgeNoted, KnowledgeObservedAssociationScope).\n",
    );
    let mut control = String::new();
    for (atom, expected) in [
        ("person(KnowledgeNoted)", true),
        ("owe(State, Eats, KnowledgeNoted)", true),
        ("decide(KnowledgeNoted, Ballot)", true),
        ("expresses(KnowledgeNoted)", false),
        ("false(KnowledgeNoted)", false),
        ("prisoner(KnowledgeNoted)", false),
        ("lose(Points, KnowledgeNoted)", false),
        ("prevents(KnowledgeNoted, CompelledConscience)", true),
        ("prevents(KnowledgeNoted, CoercedAssociation)", true),
        ("prevents(KnowledgeNoted, InvoluntaryAssociation)", true),
        ("person(Hano)", true),
        ("decide(Hano, Ballot)", true),
    ] {
        control.push_str(&query(atom, expected));
    }
    add_case(
        context,
        export,
        "free-field/a-noted-belief-is-not-a-conclusion",
        "live",
        note,
        &control,
    )?;

    // Every head this family concludes is declared derived-only upstream, so a
    // forged entry is refused at assertion rather than believed.
    let forged = concat!(
        ":refuse reasoning /declared derived-only/\n",
        "authority(KnowledgeActor, ReviewedConductRestriction, KnowledgeForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "complete(KnowledgeForgedRecord, ReviewedKnowledgeRestriction, KnowledgeMatter).\n",
        ":refuse reasoning /declared derived-only/\n",
        "permits(KnowledgeActor, EnforceReviewedKnowledgeRestriction, KnowledgeForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "related(KnowledgeForgedRecord, KnowledgeRecordAmbiguity).\n",
        ":refuse reasoning /declared derived-only/\n",
        "contradict(KnowledgeForgedRecord, KnowledgeRecordAuthorization).\n",
    );
    add_case(
        context,
        export,
        "free-field/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    Ok(())
}
