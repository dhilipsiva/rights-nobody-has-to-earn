// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of physical scarcity and cross-domain conflict: what a
//! manager must establish before it calls a shortage physical and allocates
//! under it, and what stays owed to whoever went without.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[path = "scarcity_worked.rs"]
mod worked;

const SOURCE: &str = "book-1/source/scarcity-source.json";
const BEGIN: &str = "# <SCARCITY-AND-CONFLICT-RULES-BEGIN>";
const END: &str = "# <SCARCITY-AND-CONFLICT-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const MITIGATION_KEYS: &str = "member($mitigation_key, ScarcityMitigationKeyVocabulary)";
const AUTHORIZATION: &str = "ReviewedPhysicalScarcityFinding";
const ROLES: [(&str, &str); 3] = [
    ("$source", "ScarcitySourceAuthority"),
    ("$evidence", "IndependentScarcityEvidenceAuthority"),
    ("$review", "IndependentScarcityReviewAuthority"),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    common_fields: Vec<[String; 2]>,
    vocabularies: Vec<(String, String, Vec<String>)>,
    allocation_routes: Vec<[String; 2]>,
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
    result.push([contract.kind.clone(), "ScarcityRecordKindScope".into()]);
    result.extend(contract.fields.clone());
    result
}

/// Carried through to the finding a dependent record rejoins, so an allocation
/// or a recorded shortfall cannot borrow a shortage found for another resource,
/// another population or another manager.
const CARRIED: [&str; 8] = [
    "$resource",
    "$population",
    "$manager",
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
            "authorized($reader, ScarcityChallengeReaderAuthority, $record)",
            "authorized($alternate, ScarcityAlternateReviewAuthority, $record)",
            "~($source = $evidence)",
            INDEPENDENT,
            "~($evidence = $review)",
            "~($source = $manager)",
            "~($evidence = $manager)",
            "~($review = $manager)",
            "~($reader = $alternate)",
            "~($reader = $manager)",
            "~($alternate = $manager)",
            "~($reader = $source)",
            "~($reader = $evidence)",
            "~($reader = $review)",
            "~($alternate = $source)",
            "~($alternate = $evidence)",
            "~($alternate = $review)",
            "~related($record, ScarcityRecordAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // A defect must not read its own consequence negatively, and a certified
    // nonresponse concerns a duty to act rather than a surviving record.
    if !matches!(
        contract.id.as_str(),
        "defect" | "nonresponse" | "false-scarcity"
    ) {
        atoms.push("~contradict($record, ScarcityFindingAuthorization)".into());
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
                format!("complete($authorization, {AUTHORIZATION}, $resource)"),
                "authority($manager, AdministerPhysicalScarcity, $authorization)".into(),
                "authorized($authorization_source, ScarcitySourceAuthority, $authorization)".into(),
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

fn heads(contract: &Contract) -> Vec<String> {
    let mut heads = contract.heads.clone();
    if !heads.iter().any(|h| h.starts_with("complete(")) {
        heads.push(format!("complete($record, {}, $resource)", contract.kind));
    }
    heads
}

fn rules(source: &Source) -> Vec<String> {
    let mut rules = Vec::new();
    for (scope, vocabulary, values) in &source.vocabularies {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, {scope}) -> member({value}, {vocabulary})."));
        }
    }
    // The supplied comparison chooses the permissible procedure, not a person.
    // A usable equal share precedes priority; a lottery cannot decide unequal
    // claims. These memberships describe the closed constitutional routes.
    for [comparison, method] in &source.allocation_routes {
        rules.push(format!("all $writer: all $record: observe($writer, $record, {comparison}, ScarcityComparisonOutcomeScope) -> member({method}, {comparison})."));
    }
    let scopes = source
        .contracts
        .iter()
        .flat_map(|c| fields(source, c))
        .map(|f| f[1].clone())
        .collect::<BTreeSet<_>>();
    for scope in scopes {
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, ScarcitySingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, ScarcityRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, ScarcityRecordAttester) & related($second, $record, ScarcityRecordAttester) & member($scope, ScarcitySingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, ScarcityRecordAmbiguity).".into());
    for contract in &source.contracts {
        for head in heads(contract) {
            rules.push(rule(&premises(source, contract), &head));
        }
    }
    // Asking for review of a finding, an allocation or a shortfall does not
    // require the manager's permission and does not itself decide the request.
    rules.push("all $requester: all $reader: all $request: all $resource: challenge($requester, $reader, $request) & authorized($reader, ScarcityChallengeReaderAuthority, $request) & observe($requester, $request, $resource, ScarcityChallengeResourceScope) & ~($requester = $reader) -> obliged($reader, ReviewScarcityFindingAllocationOrShortfall, $request).".into());
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
    if contract.id == "allocation" {
        values.insert(
            "$comparison_outcome".into(),
            "NoUsableEqualShareAndMateriallyUnequalClaims".into(),
        );
        values.insert(
            "$allocation_method".into(),
            "ReviewedComparativePriority".into(),
        );
    }
    values
}

/// When testing one vocabulary value, supply a compatible counterpart so the
/// positive control exercises that value. Incompatible pairs are tested
/// independently; an unknown value is deliberately left unsupported.
fn match_allocation_route(source: &Source, values: &mut BTreeMap<String, String>, changed: &str) {
    let (index, counterpart) = match changed {
        "$comparison_outcome" => (0, "$allocation_method"),
        "$allocation_method" => (1, "$comparison_outcome"),
        _ => return,
    };
    if let Some(route) = source
        .allocation_routes
        .iter()
        .find(|route| route[index] == values[changed])
    {
        values.insert(counterpart.into(), route[1 - index].clone());
    }
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
        .find(|c| c.id == "finding")
        .expect("finding contract");
    let mut other = bindings(source, authorization, "ScarcityAuthorization");
    for name in CARRIED {
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
        &format!("scarcity/{id}"),
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
    let authored = rules(&source);
    let block = format!(
        "{BEGIN}\n# Supplied findings, allocations and recorded shortfalls. A reduced ration is\n# never the minimum, and no priority key ranks a person.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "book-1/source/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous scarcity block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing scarcity end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("scarcity end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("scarcity-no-independent-review", INDEPENDENT),
        (
            "scarcity-no-ground-vocabulary",
            "member($ground, ScarcityGroundVocabulary)",
        ),
        (
            "scarcity-no-conflict-vocabulary",
            "member($conflict_kind, ScarcityConflictKindVocabulary)",
        ),
        ("scarcity-no-mitigation-keys", MITIGATION_KEYS),
        (
            "scarcity-no-allocation-route",
            "member($allocation_method, $comparison_outcome)",
        ),
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
        let values = bindings(&source, contract, "ScarcityCase");
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
                return Err(Error::new(format!("empty scarcity omission {scope}")));
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
            ("$reader", "ScarcityChallengeReaderAuthority"),
            ("$alternate", "ScarcityAlternateReviewAuthority"),
        ]) {
            let reduced = facts
                .lines()
                .filter(|l| !l.starts_with(&format!("authorized({},", values[actor])))
                .map(|l| format!("{l}\n"))
                .collect::<String>();
            emit(
                export,
                &format!("unauthorized-{}", actor.trim_start_matches('$')),
                "live",
                &reduced,
                &values,
                false,
            )?;
        }
        for [value, scope] in fields(&source, contract) {
            if value.starts_with('$') && !contract.bindings.contains_key(&value) {
                if let Some((_, _, options)) =
                    source.vocabularies.iter().find(|(s, _, _)| s == &scope)
                {
                    for option in options
                        .iter()
                        .chain(std::iter::once(&"UnapprovedScarcityKind".into()))
                    {
                        let mut other = values.clone();
                        other.insert(value.clone(), option.clone());
                        match_allocation_route(&source, &mut other, &value);
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
            "CurrentReconciledScarcityRecord",
            "SupersededScarcityRecord",
        );
        emit(export, "stale", "live", &stale, &values, false)?;
        for (label, variable, replacement) in [
            ("self-review", "$review", "$source"),
            ("manager-review", "$review", "$manager"),
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
                    "scarcity-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            (
                "mismatched-source",
                "ScarcitySourceVersionScope",
                "$version",
            ),
            ("period-drift", "ScarcityEvidencePeriodScope", "$period"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentScarcityValue, {scope})."),
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
                ("wrong-resource", "$resource"),
                ("wrong-population", "$population"),
                ("wrong-version", "$version"),
                ("wrong-period", "$period"),
                ("wrong-manager", "$manager"),
            ] {
                let mut other = values.clone();
                other.insert(variable.into(), "UnrelatedScarcityValue".into());
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
    worked::generate(context, export, &source)?;
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
    let finding = get("finding");
    let allocation = get("allocation");
    let shortfall = get("shortfall");

    // Every forbidden priority key, by name. A key outside the mitigation
    // vocabulary allocates nothing, and the counterfactual pairs the first one
    // with what its removal would permit.
    let values = bindings(source, allocation, "ScarcityKeyed");
    let forbidden = source
        .vocabularies
        .iter()
        .find(|(scope, _, _)| scope == "ForbiddenPriorityKeyScope")
        .expect("forbidden keys")
        .2
        .clone();
    for key in &forbidden {
        let mut other = values.clone();
        other.insert("$mitigation_key".into(), key.clone());
        let facts = dependency(source, allocation, &other) + &fixture(source, allocation, &other);
        add_case(
            context,
            export,
            &format!("priority/forbidden-{key}"),
            "live",
            &facts,
            &queries(allocation, &other, false),
        )?;
        if key == "Wealth" {
            add_case(
                context,
                export,
                "priority/counterfactual-forbidden-wealth",
                "scarcity-no-mitigation-keys",
                &facts,
                &queries(allocation, &other, true),
            )?;
        }
    }

    // A finding, the allocation it licenses and a recorded shortfall all stand;
    // then the shortage is independently established to have been a choice, and
    // all three stop while the floor debt and the ballot do not move.
    let values = bindings(source, allocation, "ScarcitySequence");
    let mut unmet = bindings(source, shortfall, "ScarcityUnmet");
    for name in CARRIED {
        unmet.insert(name.into(), values[name].clone());
    }
    unmet.insert("$authorization".into(), values["$authorization"].clone());
    unmet.insert(
        "$authorization_source".into(),
        values["$authorization_source"].clone(),
    );
    let mut authorization = bindings(source, finding, "ScarcityAuthorization");
    for name in CARRIED {
        authorization.insert(name.into(), values[name].clone());
    }
    authorization.insert("$record".into(), values["$authorization"].clone());
    authorization.insert("$source".into(), values["$authorization_source"].clone());

    let full = dependency(source, allocation, &values)
        + &fixture(source, allocation, &values)
        + &fixture(source, shortfall, &unmet)
        + "person(ScarcityClaimant).\nat(ScarcityClaimant, GeneralAdulthood).\n";
    let false_scarcity = get("false-scarcity");
    let mut choice = bindings(source, false_scarcity, "ScarcityChoice");
    choice.insert("$target".into(), values["$authorization"].clone());
    choice.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    for name in CARRIED {
        choice.insert(name.into(), values[name].clone());
    }

    let mut steps = queries(finding, &authorization, true);
    steps.push_str(&queries(allocation, &values, true));
    steps.push_str(&queries(shortfall, &unmet, true));
    steps.push_str(&fixture(source, false_scarcity, &choice));
    steps.push_str(&queries(false_scarcity, &choice, true));
    steps.push_str(&queries(finding, &authorization, false));
    steps.push_str(&queries(allocation, &values, false));
    steps.push_str(&queries(shortfall, &unmet, false));
    for (atom, expected) in [
        ("person(ScarcityClaimant)", true),
        ("owe(State, Eats, ScarcityClaimant)", true),
        ("decide(ScarcityClaimant, Ballot)", true),
        ("false(ScarcityClaimant)", false),
        ("prisoner(ScarcityClaimant)", false),
        ("lose(Points, ScarcityClaimant)", false),
        ("person(Hano)", true),
        ("owe(State, Eats, Hano)", true),
    ] {
        steps.push_str(&query(atom, expected));
    }
    add_case(
        context,
        export,
        "priority/false-scarcity-stops-the-allocation",
        "live",
        &full,
        &steps,
    )?;

    // Each refused ground by name: a shortage that is really a budget choice, a
    // price, a delay, withholding, a monopoly, a provider failure or a refusal
    // to procure is a failure, and naming it stops the finding it targets.
    for kind in &source
        .vocabularies
        .iter()
        .find(|(scope, _, _)| scope == "FalseScarcityKindScope")
        .expect("false-scarcity kinds")
        .2
        .clone()
    {
        let mut other = choice.clone();
        other.insert("$false_kind".into(), kind.clone());
        add_case(
            context,
            export,
            &format!("priority/not-a-shortage-{kind}"),
            "live",
            &(full.clone() + &fixture(source, false_scarcity, &other)),
            &(queries(false_scarcity, &other, true) + &queries(finding, &authorization, false)),
        )?;
    }

    // A finding for one resource or population licenses no allocation for
    // another. The rejoin refuses it, not the label.
    for (label, variable) in [
        ("another-resource", "$resource"),
        ("another-population", "$population"),
    ] {
        let mut other = values.clone();
        other.insert(variable.into(), format!("Other{label}").replace('-', ""));
        add_case(
            context,
            export,
            &format!("priority/finding-does-not-cover-{label}"),
            "live",
            &(dependency(source, allocation, &values) + &fixture(source, allocation, &other)),
            &queries(allocation, &other, false),
        )?;
    }

    // Each named cross-domain conflict resolves on its own stated typed rule,
    // and an unnamed one resolves nothing.
    let conflict = get("conflict");
    let base = bindings(source, conflict, "ScarcityConflict");
    for kind in &source
        .vocabularies
        .iter()
        .find(|(scope, _, _)| scope == "ScarcityConflictKindScope")
        .expect("conflict kinds")
        .2
        .clone()
    {
        let mut other = base.clone();
        other.insert("$conflict_kind".into(), kind.clone());
        add_case(
            context,
            export,
            &format!("conflict/{kind}"),
            "live",
            &fixture(source, conflict, &other),
            &queries(conflict, &other, true),
        )?;
    }

    // Asking for review creates the reader's duty; a certified nonresponse
    // moves it to the alternate without deciding the request.
    let nonresponse = get("nonresponse");
    let mut request = bindings(source, nonresponse, "ScarcityRequest");
    request.insert("$target".into(), "ScarcityRequestRecord".into());
    let requester = request["$requester"].clone();
    let reader = request["$reader"].clone();
    let facts = format!(
        "challenge({requester}, {reader}, ScarcityRequestRecord).\nauthorized({reader}, ScarcityChallengeReaderAuthority, ScarcityRequestRecord).\nobserve({requester}, ScarcityRequestRecord, ScarcityRequestResource, ScarcityChallengeResourceScope).\n"
    );
    let mut steps = query(
        &format!(
            "obliged({reader}, ReviewScarcityFindingAllocationOrShortfall, ScarcityRequestRecord)"
        ),
        true,
    );
    steps.push_str(&queries(nonresponse, &request, false));
    steps.push_str(&fixture(source, nonresponse, &request));
    steps.push_str(&queries(nonresponse, &request, true));
    add_case(
        context,
        export,
        "priority/request-and-certified-nonresponse",
        "live",
        &facts,
        &steps,
    )?;

    // Every head this family concludes is declared derived-only upstream.
    let forged = concat!(
        ":refuse reasoning /declared derived-only/\n",
        "authority(ScarcityManager, AdministerPhysicalScarcity, ScarcityForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "complete(ScarcityForgedRecord, ReviewedPhysicalScarcityFinding, ScarcityResource).\n",
        ":refuse reasoning /declared derived-only/\n",
        "permits(ScarcityManager, AllocateUnderPhysicalScarcity, ScarcityForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "contradict(ScarcityForgedRecord, ScarcityFindingAuthorization).\n",
    );
    add_case(
        context,
        export,
        "priority/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    Ok(())
}
