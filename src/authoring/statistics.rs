// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of bounded statistical permissions and evidence routes.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "new-book-plans/statistics-source.json";
const BEGIN: &str = "# <OFFICIAL-STATISTICS-RULES-BEGIN>";
const END: &str = "# <OFFICIAL-STATISTICS-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const ROLES: [(&str, &str); 3] = [
    ("$source", "StatisticsSourceAuthority"),
    ("$evidence", "IndependentStatisticsMethodAuthority"),
    ("$review", "IndependentStatisticsReviewAuthority"),
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
    use_dependency: bool,
    bindings: BTreeMap<String, String>,
}

fn fields(source: &Source, contract: &Contract) -> Vec<[String; 2]> {
    let mut result = source.common_fields.clone();
    result.push([contract.kind.clone(), "StatisticsRecordKindScope".into()]);
    result.extend(contract.fields.clone());
    result
}

fn premises(source: &Source, contract: &Contract) -> Vec<String> {
    let mut atoms = ROLES
        .iter()
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    atoms.extend(
        [
            "authorized($reader, StatisticsChallengeReaderAuthority, $record)",
            "authorized($alternate, StatisticsAlternateReviewAuthority, $record)",
            "~($source = $evidence)",
            INDEPENDENT,
            "~($evidence = $review)",
            "~($source = $operator)",
            "~($evidence = $operator)",
            "~($review = $operator)",
            "~($reader = $alternate)",
            "~($reader = $operator)",
            "~($alternate = $operator)",
            "~($reader = $source)",
            "~($reader = $evidence)",
            "~($reader = $review)",
            "~($alternate = $source)",
            "~($alternate = $evidence)",
            "~($alternate = $review)",
            "~related($record, StatisticsRecordAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // Defects must not read their own consequence negatively. Nonresponse
    // concerns a duty to act, independently of the challenged use permission.
    if !matches!(contract.id.as_str(), "defect" | "rebuttal" | "nonresponse") {
        atoms.push("~contradict($record, StatisticsUseAuthorization)".into());
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
    if contract.use_dependency {
        atoms.extend(
            [
                "complete($authorization, ReviewedStatisticalUse, $dataset)",
                "authority($operator, $purpose, $authorization)",
                "authorized($authorization_source, StatisticsSourceAuthority, $authorization)",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        // Rejoin the actual authorization, not merely a token naming one.
        // Authorized conflicting values on that record block its completion.
        for [value, scope] in &source.common_fields {
            if [
                "$dataset",
                "$operator",
                "$purpose",
                "$version",
                "$method",
                "$period",
                "$jurisdiction",
                "$scope",
                "$end",
            ]
            .contains(&value.as_str())
            {
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
        heads.push(format!("complete($record, {}, $dataset)", contract.kind));
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
    let scopes = source
        .contracts
        .iter()
        .flat_map(|c| fields(source, c))
        .map(|f| f[1].clone())
        .collect::<BTreeSet<_>>();
    for scope in scopes {
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, StatisticsSingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, StatisticsRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, StatisticsRecordAttester) & related($second, $record, StatisticsRecordAttester) & member($scope, StatisticsSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, StatisticsRecordAmbiguity).".into());
    for contract in &source.contracts {
        for head in heads(contract) {
            rules.push(rule(&premises(source, contract), &head));
        }
    }
    // Asking for review does not require the operator's permission or a
    // surviving statistical use, and does not itself grant private access.
    rules.push("all $requester: all $reader: all $request: all $dataset: challenge($requester, $reader, $request) & authorized($reader, StatisticsChallengeReaderAuthority, $request) & observe($requester, $request, $dataset, StatisticsChallengeDatasetScope) & ~($requester = $reader) -> obliged($reader, ReviewStatisticsAccessCorrectionDeletionOrChallenge, $request).".into());
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
    if !contract.use_dependency {
        return premises(source, contract)
            .into_iter()
            .filter(|a| {
                a.contains("$target_source")
                    && (a.starts_with("authorized(") || a.starts_with("observe("))
            })
            .map(|a| format!("{}.\n", ground(&a, values)))
            .collect();
    }
    let authorization = source.contracts.iter().find(|c| c.id == "use").unwrap();
    let mut other = bindings(source, authorization, "StatisticsAuthorization");
    for [value, _] in &source.common_fields {
        if value.starts_with('$')
            && ![
                "$reader",
                "$alternate",
                "$challenge",
                "$correction",
                "$remedy",
            ]
            .contains(&value.as_str())
        {
            other.insert(value.clone(), values[value].clone());
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
        &format!("statistics/{id}"),
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
        "{BEGIN}\n# Supplied permissions and bounded findings, not population computation or action.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "new-book-plans/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous statistics block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing statistics end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("statistics end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("statistics-no-independent-review", INDEPENDENT),
        (
            "statistics-no-purpose-wall",
            "member($purpose, StatisticsPurposeVocabulary)",
        ),
        (
            "statistics-no-record-separation",
            "~($dataset = $constitutional_record)",
        ),
        (
            "statistics-no-remedial-limit",
            "member($proceeding_kind, StatisticsRemedialProceedingVocabulary)",
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
        let values = bindings(&source, contract, "StatisticsCase");
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
                return Err(Error::new(format!("empty statistics omission {scope}")));
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
            ("$reader", "StatisticsChallengeReaderAuthority"),
            ("$alternate", "StatisticsAlternateReviewAuthority"),
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
                        .chain(std::iter::once(&"UnapprovedStatisticsKind".into()))
                    {
                        let mut other = values.clone();
                        other.insert(value.clone(), option.clone());
                        // Rebuild this case's independent dependency for a
                        // lawful different purpose, not for a mismatched use.
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
            "CurrentReconciledStatisticsRecord",
            "SupersededStatisticsRecord",
        );
        emit(export, "stale", "live", &stale, &values, false)?;
        for (label, variable, replacement) in [
            ("self-review", "$review", "$source"),
            ("operator-review", "$review", "$operator"),
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
                    "statistics-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            (
                "mismatched-source",
                "StatisticsSourceVersionScope",
                "$version",
            ),
            ("method-drift", "StatisticsMethodVersionScope", "$method"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentStatisticsValue, {scope})."),
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
        if contract.use_dependency {
            add_case(
                context,
                export,
                &format!("{}/without-authorization", contract.id),
                "live",
                &facts,
                &expected,
            )?;
            for (label, variable) in [
                ("wrong-dataset", "$dataset"),
                ("wrong-version", "$version"),
                ("wrong-method", "$method"),
                ("wrong-period", "$period"),
                ("wrong-purpose", "$purpose"),
            ] {
                let mut other = values.clone();
                other.insert(
                    variable.into(),
                    if variable == "$purpose" {
                        "PublicPlanning".into()
                    } else {
                        "UnrelatedStatisticsValue".into()
                    },
                );
                if other[variable] == values[variable] {
                    other.insert(variable.into(), "EqualityDiagnostics".into());
                }
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
    let get = |id: &str| source.contracts.iter().find(|c| c.id == id).unwrap();
    let use_contract = get("use");
    let values = bindings(source, use_contract, "StatisticsUse");
    for variable in [
        "$constitutional_record",
        "$eligibility_record",
        "$enforcement_record",
    ] {
        let mut other = values.clone();
        other.insert(variable.into(), values["$dataset"].clone());
        let facts = fixture(source, use_contract, &other);
        add_case(
            context,
            export,
            &format!("data-wall/reuse-{}", variable.trim_start_matches('$')),
            "live",
            &facts,
            &queries(use_contract, &other, false),
        )?;
        if variable == "$constitutional_record" {
            add_case(
                context,
                export,
                "data-wall/counterfactual-record-reuse",
                "statistics-no-record-separation",
                &facts,
                &queries(use_contract, &other, true),
            )?;
        }
    }
    for purpose in [
        "IndividualStanding",
        "IndividualFloor",
        "IndividualSanction",
        "IndividualRisk",
        "IndividualEntitlement",
        "IndividualWorth",
        "IndividualGuilt",
        "IndividualPunishment",
        "IndividualEligibility",
        "IndividualPoliticalWeight",
    ] {
        let mut other = values.clone();
        other.insert("$purpose".into(), purpose.into());
        let facts = fixture(source, use_contract, &other);
        add_case(
            context,
            export,
            &format!("data-wall/{purpose}"),
            "live",
            &facts,
            &queries(use_contract, &other, false),
        )?;
        if purpose == "IndividualRisk" {
            add_case(
                context,
                export,
                "data-wall/counterfactual-individual-risk",
                "statistics-no-purpose-wall",
                &facts,
                &queries(use_contract, &other, true),
            )?;
        }
    }
    let pattern = get("pattern");
    let mut criminal = bindings(source, pattern, "StatisticsCriminal");
    criminal.insert("$proceeding_kind".into(), "CriminalBurdenReversal".into());
    let facts = dependency(source, pattern, &criminal) + &fixture(source, pattern, &criminal);
    for (label, base, expected) in [
        ("criminal-burden", "live", false),
        (
            "counterfactual-criminal-burden",
            "statistics-no-remedial-limit",
            true,
        ),
    ] {
        add_case(
            context,
            export,
            &format!("data-wall/{label}"),
            base,
            &facts,
            &queries(pattern, &criminal, expected),
        )?;
    }
    for id in ["use", "release", "pattern", "access"] {
        let contract = get(id);
        let values = bindings(source, contract, "StatisticsSequence");
        let full = dependency(source, contract, &values)
            + &fixture(source, contract, &values)
            + "person(StatisticsParticipant).\nat(StatisticsParticipant, GeneralAdulthood).\n";
        let finding = get(if id == "pattern" {
            "rebuttal"
        } else {
            "defect"
        });
        let mut defect = bindings(source, finding, "StatisticsDefect");
        defect.insert("$target".into(), values["$record"].clone());
        defect.insert("$target_source".into(), values["$source"].clone());
        for variable in [
            "$dataset",
            "$operator",
            "$purpose",
            "$version",
            "$method",
            "$period",
            "$jurisdiction",
            "$scope",
            "$end",
            "$proceeding",
        ] {
            if let Some(value) = values.get(variable) {
                defect.insert(variable.into(), value.clone());
            }
        }
        let mut steps = queries(contract, &values, true);
        steps.push_str(&fixture(source, finding, &defect));
        steps.push_str(&queries(contract, &values, false));
        steps.push_str(&queries(finding, &defect, true));
        for (q, expected) in [
            ("person(Hano)", true),
            ("owe(State, Eats, Hano)", true),
            ("decide(Hano, Ballot)", true),
            ("expresses(Hano)", true),
            ("authority(Boss)", true),
            ("false(StatisticsParticipant)", false),
            ("prisoner(StatisticsParticipant)", false),
            ("lose(Points, StatisticsParticipant)", false),
            ("person(StatisticsParticipant)", true),
            ("owe(State, Eats, StatisticsParticipant)", true),
            ("decide(StatisticsParticipant, Ballot)", true),
            ("person(StatisticsSequenceDataset)", false),
        ] {
            steps.push_str(&query(q, expected));
        }
        add_case(
            context,
            export,
            &format!("{id}/defect-and-rights-sequence"),
            "live",
            &full,
            &steps,
        )?;
        for variable in ["$version", "$method", "$period"] {
            let mut stale = defect.clone();
            stale.insert(variable.into(), "EarlierStatisticsValue".into());
            let facts = full.clone() + &fixture(source, finding, &stale);
            add_case(
                context,
                export,
                &format!("{id}/defect-wrong-{}", variable.trim_start_matches('$')),
                "live",
                &facts,
                &(queries(contract, &values, true) + &queries(finding, &stale, false)),
            )?;
        }
        if contract.use_dependency {
            let finding = get("defect");
            let mut withdrawn = bindings(source, finding, "StatisticsWithdrawn");
            withdrawn.insert("$target".into(), values["$authorization"].clone());
            withdrawn.insert(
                "$target_source".into(),
                values["$authorization_source"].clone(),
            );
            for variable in [
                "$dataset",
                "$operator",
                "$purpose",
                "$version",
                "$method",
                "$period",
                "$jurisdiction",
                "$scope",
                "$end",
            ] {
                withdrawn.insert(variable.into(), values[variable].clone());
            }
            let steps = queries(contract, &values, true)
                + &fixture(source, finding, &withdrawn)
                + &queries(finding, &withdrawn, true)
                + &queries(contract, &values, false);
            add_case(
                context,
                export,
                &format!("{id}/authorization-withdrawal-sequence"),
                "live",
                &full,
                &steps,
            )?;
        }
        defect.insert("$target".into(), "DifferentStatisticsUse".into());
        let facts =
            full + &dependency(source, finding, &defect) + &fixture(source, finding, &defect);
        add_case(
            context,
            export,
            &format!("{id}/unrelated-defect"),
            "live",
            &facts,
            &(queries(contract, &values, true) + &queries(finding, &defect, true)),
        )?;
    }
    let facts = "challenge(StatisticsRequester, StatisticsReader, StatisticsRequest).\nauthorized(StatisticsReader, StatisticsChallengeReaderAuthority, StatisticsRequest).\nobserve(StatisticsRequester, StatisticsRequest, StatisticsDataset, StatisticsChallengeDatasetScope).\n";
    let mut steps = query(
        "obliged(StatisticsReader, ReviewStatisticsAccessCorrectionDeletionOrChallenge, StatisticsRequest)",
        true,
    );
    steps.push_str(&query(
        "permits(StatisticsRequester, InspectOwnStatisticalEvidence, StatisticsRequest)",
        false,
    ));
    let mut nonresponse = bindings(source, get("nonresponse"), "StatisticsNonresponse");
    nonresponse.insert("$target".into(), "StatisticsRequest".into());
    nonresponse.insert("$requester".into(), "StatisticsRequester".into());
    nonresponse.insert("$reader".into(), "StatisticsReader".into());
    steps.push_str(&queries(get("nonresponse"), &nonresponse, false));
    steps.push_str(&fixture(source, get("nonresponse"), &nonresponse));
    steps.push_str(&queries(get("nonresponse"), &nonresponse, true));
    steps.push_str(&query("obliged(StatisticsReader, ReviewStatisticsAccessCorrectionDeletionOrChallenge, StatisticsRequest)", true));
    add_case(
        context,
        export,
        "challenge/nonresponse-sequence",
        "live",
        facts,
        &steps,
    )?;
    let mut control = String::new();
    for (q, expected) in [
        ("person(Hano)", true),
        ("owe(State, Eats, Hano)", true),
        ("decide(Hano, Ballot)", true),
        ("prevents(Hano, DiagnosticNonparticipationPenalty)", true),
    ] {
        control.push_str(&query(q, expected));
    }
    let data = "observe(StatisticsParticipantWriter, StatisticsNonresponseRecord, Hano, StatisticsNonparticipantHandleScope).\n";
    add_case(
        context,
        export,
        "data-wall/nonparticipation-is-not-a-penalty",
        "live",
        data,
        &control,
    )?;
    let forged = ":refuse reasoning /declared derived-only/\nauthority(StatisticsOperator, IndividualRisk, StatisticsForgedRecord).\n:refuse reasoning /declared derived-only/\ncomplete(StatisticsForgedRecord, RebuttableAggregateEqualityPresumption, StatisticsProceeding).\n:refuse reasoning /declared derived-only/\nrelated(StatisticsForgedRecord, StatisticsRecordAmbiguity).\n:refuse reasoning /declared derived-only/\ncontradict(StatisticsForgedRecord, StatisticsUseAuthorization).\n";
    add_case(
        context,
        export,
        "data-wall/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "statistics_tests.rs"]
mod tests;
