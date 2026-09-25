// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of record power: what a holder must establish before it
//! keeps, watches, profiles or automates over somebody's record, and what it
//! owes while it has not.

use super::procedural_load::{
    beneficial_kinds, fast_head, single_actor, PROMPT_REVIEW, REVIEW_SCOPE, WITHDRAWN,
};
use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "book-1/source/record-power-source.json";
const BEGIN: &str = "# <RECORD-POWER-RULES-BEGIN>";
const END: &str = "# <RECORD-POWER-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const HUMAN_REVIEW: &str = "~($reviewer = $holder)";
const AUTHORIZATION: &str = "ReviewedRecordHolding";
const ROLES: [(&str, &str); 3] = [
    ("$source", "RecordSourceAuthority"),
    ("$evidence", "IndependentRecordEvidenceAuthority"),
    ("$review", "IndependentRecordReviewAuthority"),
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
    #[serde(default)]
    continuing_heads: Vec<String>,
    extra: Vec<String>,
    record_dependency: bool,
    bindings: BTreeMap<String, String>,
}

fn fields(source: &Source, contract: &Contract) -> Vec<[String; 2]> {
    let mut result = source.common_fields.clone();
    result.push([contract.kind.clone(), "RecordEntryKindScope".into()]);
    result.extend(contract.fields.clone());
    result
}

/// Carried through to the holding a dependent record rejoins, so a processing
/// or automated-support record cannot borrow a holding reviewed for somebody
/// else, another domain or another purpose.
const CARRIED: [&str; 9] = [
    "$subject",
    "$domain",
    "$holder",
    "$purpose",
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
            "authorized($reader, RecordChallengeReaderAuthority, $record)",
            "authorized($alternate, RecordAlternateReviewAuthority, $record)",
            "~($source = $evidence)",
            INDEPENDENT,
            "~($evidence = $review)",
            "~($source = $holder)",
            "~($evidence = $holder)",
            "~($review = $holder)",
            "~($reader = $alternate)",
            "~($reader = $holder)",
            "~($alternate = $holder)",
            "~($reader = $source)",
            "~($reader = $evidence)",
            "~($reader = $review)",
            "~($alternate = $source)",
            "~($alternate = $evidence)",
            "~($alternate = $review)",
            "~related($record, RecordEntryAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // A defect must not read its own consequence negatively, and a certified
    // nonresponse concerns a duty to act rather than a surviving record.
    if !matches!(contract.id.as_str(), "defect" | "nonresponse") {
        atoms.push("~contradict($record, RecordUseAuthorization)".into());
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
                format!("complete($authorization, {AUTHORIZATION}, $subject)"),
                "authority($holder, $purpose, $authorization)".into(),
                "authorized($authorization_source, RecordSourceAuthority, $authorization)".into(),
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

// Duties attach to the fully specified undertaking, including the exact raw
// holding to which a dependent instrument refers. They do not authorize use.
// Withdrawal or conflicting later entries cannot discharge those duties or
// the subject's contest right. The original positive fields and separations
// still bind the beneficiary, responsible actor and independent reviewer.
fn continuing_premises(source: &Source, contract: &Contract) -> Vec<String> {
    premises(source, contract)
        .into_iter()
        .filter(|atom| {
            !matches!(
                atom.as_str(),
                "~related($record, RecordEntryAmbiguity)"
                    | "~contradict($record, RecordUseAuthorization)"
                    | "complete($authorization, ReviewedRecordHolding, $subject)"
                    | "authority($holder, $purpose, $authorization)"
            )
        })
        .collect()
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
        heads.push(format!("complete($record, {}, $subject)", contract.kind));
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
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, RecordSingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, RecordEntryAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, RecordEntryAttester) & related($second, $record, RecordEntryAttester) & member($scope, RecordSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, RecordEntryAmbiguity).".into());
    for contract in &source.contracts {
        for head in heads(contract) {
            let atoms = if contract.continuing_heads.contains(&head) {
                continuing_premises(source, contract)
            } else {
                premises(source, contract)
            };
            rules.push(rule(&atoms, &head));
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
    // Asking to see, correct or object does not require the holder's permission
    // and does not itself decide the request.
    rules.push("all $requester: all $reader: all $request: all $subject: challenge($requester, $reader, $request) & authorized($reader, RecordChallengeReaderAuthority, $request) & observe($requester, $request, $subject, RecordChallengeSubjectScope) & ~($requester = $reader) -> obliged($reader, ReviewRecordAccessCorrectionOrObjection, $request).".into());
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
        .find(|c| c.id == "holding")
        .expect("holding contract");
    let mut other = bindings(source, authorization, "RecordAuthorization");
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
    queries_with_continuity(contract, values, expected, expected)
}

fn queries_with_continuity(
    contract: &Contract,
    values: &BTreeMap<String, String>,
    power: bool,
    protection: bool,
) -> String {
    heads(contract)
        .iter()
        .map(|h| {
            query(
                &ground(h, values),
                if contract.continuing_heads.contains(h) {
                    protection
                } else {
                    power
                },
            )
        })
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
        &format!("record-power/{id}"),
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
        "{BEGIN}\n# Supplied permissions, duties and reviewed findings over supplied record\n# entries. No computed output is an oracle and no entry rates a person.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "book-1/source/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous record-power block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing record-power end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("record-power end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("record-power-no-independent-review", INDEPENDENT),
        (
            "record-power-no-domain-vocabulary",
            "member($domain, RecordDomainVocabulary)",
        ),
        (
            "record-power-no-purpose-vocabulary",
            "member($purpose, RecordPurposeVocabulary)",
        ),
        ("record-power-no-human-review", HUMAN_REVIEW),
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
        let values = bindings(&source, contract, "RecordCase");
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
                return Err(Error::new(format!("empty record-power omission {scope}")));
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
            ("$reader", "RecordChallengeReaderAuthority"),
            ("$alternate", "RecordAlternateReviewAuthority"),
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
                        .chain(std::iter::once(&"UnapprovedRecordKind".into()))
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
        let stale = facts.replace("CurrentReconciledRecordEntry", "SupersededRecordEntry");
        emit(export, "stale", "live", &stale, &values, false)?;
        for (label, variable, replacement) in [
            ("self-review", "$review", "$source"),
            ("holder-review", "$review", "$holder"),
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
                    "record-power-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            ("mismatched-source", "RecordSourceVersionScope", "$version"),
            ("period-drift", "RecordEvidencePeriodScope", "$period"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentRecordValue, {scope})."),
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
            add_case(
                context,
                export,
                &format!("{}/conflicting-{label}", contract.id),
                "live",
                &(dep.clone() + &conflict),
                &queries_with_continuity(contract, &values, false, true),
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
                ("wrong-subject", "$subject"),
                ("wrong-domain", "$domain"),
                ("wrong-version", "$version"),
                ("wrong-period", "$period"),
                ("wrong-purpose", "$purpose"),
            ] {
                let mut other = values.clone();
                other.insert(
                    variable.into(),
                    match variable {
                        "$domain" => "HousingRecord".into(),
                        "$purpose" => "IndependentOversightOfTheHolder".into(),
                        _ => "UnrelatedRecordValue".into(),
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

    // A holding, the processing it licenses and the automated support that
    // reads it all stand. One reviewed defect withdraws the holding and its
    // dependent permissions while preserving the duties and subject protection.
    // The opaque subject handle gains no personal status from this sequence.
    let holding = get("holding");
    let processing = get("processing");
    let automated = get("automated");
    let values = bindings(source, processing, "RecordSequence");
    let mut supported = bindings(source, automated, "RecordSupport");
    for name in CARRIED {
        supported.insert(name.into(), values[name].clone());
    }
    supported.insert("$authorization".into(), values["$authorization"].clone());
    supported.insert(
        "$authorization_source".into(),
        values["$authorization_source"].clone(),
    );
    let mut authorization = bindings(source, holding, "RecordAuthorization");
    for name in CARRIED {
        authorization.insert(name.into(), values[name].clone());
    }
    authorization.insert("$record".into(), values["$authorization"].clone());
    authorization.insert("$source".into(), values["$authorization_source"].clone());

    let full = dependency(source, processing, &values)
        + &fixture(source, processing, &values)
        + &fixture(source, automated, &supported);
    let defect = get("defect");
    let mut against_holding = bindings(source, defect, "RecordHoldingDefect");
    against_holding.insert("$target".into(), values["$authorization"].clone());
    against_holding.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    for name in CARRIED {
        against_holding.insert(name.into(), values[name].clone());
    }

    let subject = values["$subject"].clone();
    let mut steps = queries(holding, &authorization, true);
    steps.push_str(&queries(processing, &values, true));
    steps.push_str(&queries(automated, &supported, true));
    steps.push_str(&fixture(source, defect, &against_holding));
    steps.push_str(&queries(defect, &against_holding, true));
    steps.push_str(&queries_with_continuity(
        holding,
        &authorization,
        false,
        true,
    ));
    steps.push_str(&queries_with_continuity(processing, &values, false, true));
    steps.push_str(&queries_with_continuity(automated, &supported, false, true));
    for (atom, expected) in [
        (format!("person({subject})"), false),
        (format!("false({subject})"), false),
        (format!("prisoner({subject})"), false),
        (format!("lose(Points, {subject})"), false),
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
        "power/holding-defect-and-rights-sequence",
        "live",
        &full,
        &steps,
    )?;

    // A defect against the dependent record itself also leaves its duties and
    // contest right in force; the parent remains authorized in this sequence.
    for (contract, affected) in [(processing, &values), (automated, &supported)] {
        let mut finding = bindings(source, defect, "RecordOwnDefect");
        finding.insert("$target".into(), affected["$record"].clone());
        finding.insert("$target_source".into(), affected["$source"].clone());
        for name in CARRIED {
            finding.insert(name.into(), affected[name].clone());
        }
        let sequence = queries(contract, affected, true)
            + &fixture(source, defect, &finding)
            + &queries(defect, &finding, true)
            + &queries_with_continuity(contract, affected, false, true)
            + &queries(holding, &authorization, true);
        add_case(
            context,
            export,
            &format!("power/{}-own-defect-retains-protection", contract.id),
            "live",
            &full,
            &sequence,
        )?;
    }

    // A defect about a different holding leaves this one standing.
    let mut elsewhere = against_holding.clone();
    elsewhere.insert("$target".into(), "DifferentRecordHolding".into());
    let facts = full.clone()
        + &dependency(source, defect, &elsewhere)
        + &fixture(source, defect, &elsewhere);
    add_case(
        context,
        export,
        "power/unrelated-defect",
        "live",
        &facts,
        &(queries(processing, &values, true) + &queries(defect, &elsewhere, true)),
    )?;

    // A holding reviewed for one person, domain or purpose licenses no
    // processing about another. The rejoin refuses it, not the label.
    for (label, variable, replacement) in [
        ("another-subject", "$subject", "AnotherRecordSubject"),
        ("another-domain", "$domain", "PolicingRecord"),
        (
            "another-purpose",
            "$purpose",
            "IndependentOversightOfTheHolder",
        ),
    ] {
        let mut other = values.clone();
        if other[variable] == replacement {
            continue;
        }
        other.insert(variable.into(), replacement.into());
        add_case(
            context,
            export,
            &format!("power/holding-does-not-cover-{label}"),
            "live",
            &(dependency(source, processing, &values) + &fixture(source, processing, &other)),
            &queries(processing, &other, false),
        )?;
    }

    // The automated route is support, never the decision. Take away the human
    // and independent review, or let the reviewer be the holder, and nothing
    // follows; the counterfactual shows that this is what refused it.
    let mut fused = supported.clone();
    fused.insert("$reviewer".into(), supported["$holder"].clone());
    let fused_facts =
        dependency(source, automated, &supported) + &fixture(source, automated, &fused);
    add_case(
        context,
        export,
        "power/automated-reviewer-cannot-be-the-holder",
        "live",
        &fused_facts,
        &queries(automated, &fused, false),
    )?;
    add_case(
        context,
        export,
        "power/counterfactual-automated-reviewer-is-the-holder",
        "record-power-no-human-review",
        &fused_facts,
        &queries(automated, &fused, true),
    )?;

    // Asking to see or correct creates the reader's duty; a certified
    // nonresponse moves it to the alternate without deciding the request.
    let nonresponse = get("nonresponse");
    let mut request = bindings(source, nonresponse, "RecordRequest");
    request.insert("$target".into(), "RecordRequestEntry".into());
    let requester = request["$requester"].clone();
    let reader = request["$reader"].clone();
    let facts = format!(
        "challenge({requester}, {reader}, RecordRequestEntry).\nauthorized({reader}, RecordChallengeReaderAuthority, RecordRequestEntry).\nobserve({requester}, RecordRequestEntry, RecordRequestSubject, RecordChallengeSubjectScope).\n"
    );
    let mut steps = query(
        &format!("obliged({reader}, ReviewRecordAccessCorrectionOrObjection, RecordRequestEntry)"),
        true,
    );
    steps.push_str(&query(
        &format!("permits({requester}, InspectOwnRecord, RecordRequestEntry)"),
        false,
    ));
    steps.push_str(&queries(nonresponse, &request, false));
    steps.push_str(&fixture(source, nonresponse, &request));
    steps.push_str(&queries(nonresponse, &request, true));
    steps.push_str(&query(
        &format!("obliged({reader}, ReviewRecordAccessCorrectionOrObjection, RecordRequestEntry)"),
        true,
    ));
    add_case(
        context,
        export,
        "power/request-and-certified-nonresponse",
        "live",
        &facts,
        &steps,
    )?;

    // An entry that names somebody is inert about them. The legacy writable
    // surface is untouched: a raw report still decides nothing on its own.
    let note = concat!(
        "person(RecordNoted).\n",
        "at(RecordNoted, GeneralAdulthood).\n",
        "observe(RecordNoteWriter, RecordNoteEntry, RecordNoted, RecordSubjectScope).\n",
        "observe(RecordNoteWriter, RecordNoteEntry, Profiling, RecordProcessingKindScope).\n",
        "rotten(RecordNoted).\n",
    );
    let mut control = String::new();
    for (atom, expected) in [
        ("person(RecordNoted)", true),
        ("owe(State, Eats, RecordNoted)", true),
        ("decide(RecordNoted, Ballot)", true),
        ("false(RecordNoted)", false),
        ("prisoner(RecordNoted)", false),
        ("lose(Points, RecordNoted)", false),
        ("severe(RecordNoted)", false),
        ("prevents(RecordNoted, CompelledConscience)", true),
    ] {
        control.push_str(&query(atom, expected));
    }
    add_case(
        context,
        export,
        "power/a-noted-entry-is-not-a-conclusion",
        "live",
        note,
        &control,
    )?;

    // Every head this family concludes is declared derived-only upstream, so a
    // forged entry is refused at assertion rather than believed.
    let forged = concat!(
        ":refuse reasoning /declared derived-only/\n",
        "authority(RecordHolder, DeliverTheServiceRecorded, RecordForgedEntry).\n",
        ":refuse reasoning /declared derived-only/\n",
        "complete(RecordForgedEntry, ReviewedRecordHolding, RecordSubject).\n",
        ":refuse reasoning /declared derived-only/\n",
        "permits(RecordHolder, TargetedSurveillance, RecordForgedEntry).\n",
        ":refuse reasoning /declared derived-only/\n",
        "related(RecordForgedEntry, RecordEntryAmbiguity).\n",
        ":refuse reasoning /declared derived-only/\n",
        "contradict(RecordForgedEntry, RecordUseAuthorization).\n",
    );
    add_case(
        context,
        export,
        "power/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    Ok(())
}
