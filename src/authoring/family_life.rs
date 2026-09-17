// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authoring of the ordinary half of life course, family, care and
//! reproduction: the roles, support, participation, continuity, bodily care,
//! treatment order and own-record access that the baseline's barriers
//! presupposed and the formal source did not carry.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "book-1/source/family-life-source.json";
const BEGIN: &str = "# <FAMILY-LIFE-ORDINARY-RULES-BEGIN>";
const END: &str = "# <FAMILY-LIFE-ORDINARY-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";
const PARTICIPANT_SEPARATION: &str = "~($participant = $actor)";
const AUTHORIZATION: &str = "ReviewedLifeCourseRole";
const ROLES: [(&str, &str); 3] = [
    ("$source", "LifeCourseSourceAuthority"),
    ("$evidence", "IndependentLifeCourseEvidenceAuthority"),
    ("$review", "IndependentLifeCourseReviewAuthority"),
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
    result.push([contract.kind.clone(), "LifeCourseRecordKindScope".into()]);
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
            "authorized($reader, LifeCourseChallengeReaderAuthority, $record)",
            "authorized($alternate, LifeCourseAlternateReviewAuthority, $record)",
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
            "~related($record, LifeCourseRecordAmbiguity)",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    // A defect must not read its own consequence negatively, and a certified
    // nonresponse concerns a duty to act rather than a surviving record.
    if !matches!(contract.id.as_str(), "defect" | "nonresponse") {
        atoms.push("~contradict($record, LifeCourseRecordAuthorization)".into());
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
                "authority($actor, $role_kind, $authorization)".into(),
                "authorized($authorization_source, LifeCourseSourceAuthority, $authorization)"
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

fn heads(contract: &Contract) -> Vec<String> {
    let mut heads = contract.heads.clone();
    if !heads.iter().any(|h| h.starts_with("complete(")) {
        heads.push(format!("complete($record, {}, $decision)", contract.kind));
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
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, LifeCourseSingleValueScope)."));
    }
    for (_, role) in ROLES {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, LifeCourseRecordAttester)."));
    }
    rules.push("all $first: all $second: all $record: all $scope: all $a: all $b: related($first, $record, LifeCourseRecordAttester) & related($second, $record, LifeCourseRecordAttester) & member($scope, LifeCourseSingleValueScope) & observe($first, $record, $a, $scope) & observe($second, $record, $b, $scope) & ~($a = $b) -> related($record, LifeCourseRecordAmbiguity).".into());
    for contract in &source.contracts {
        for head in heads(contract) {
            rules.push(rule(&premises(source, contract), &head));
        }
    }
    // Asking for review of a role, a refused access or a correction does not
    // require the acting body's permission and does not itself decide the
    // request.
    rules.push("all $requester: all $reader: all $request: all $decision: challenge($requester, $reader, $request) & authorized($reader, LifeCourseChallengeReaderAuthority, $request) & observe($requester, $request, $decision, LifeCourseChallengeDecisionScope) & ~($requester = $reader) -> obliged($reader, ReviewTheLifeCourseRequestAccessOrCorrection, $request).".into());
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
        .find(|c| c.id == "role")
        .expect("role contract");
    let mut other = bindings(source, authorization, "LifeCourseAuthorization");
    for name in CARRIED.into_iter().chain(["$role_kind"]) {
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
        &format!("family-life/{id}"),
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
        "{BEGIN}\n# Supplied permissions, duties and reviewed findings. Nothing here certifies\n# truth, taste, belief, creativity, a relationship or personal fulfilment.\n{}\n{END}",
        authored.join("\n")
    );
    let path = "book-1/source/constitution.nibli";
    let old = context.read(path)?;
    let updated = if let Some((before, rest)) = old.split_once(BEGIN) {
        if old.matches(BEGIN).count() != 1 || old.matches(END).count() != 1 {
            return Err(Error::new("ambiguous family-life block"));
        }
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing family-life end marker"))?;
        format!("{before}{block}{after}")
    } else if old.contains(END) {
        return Err(Error::new("family-life end marker without beginning"));
    } else {
        format!("{}\n\n{block}\n", old.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    for (name, removed) in [
        ("family-life-no-independent-review", INDEPENDENT),
        (
            "family-life-no-domain-vocabulary",
            "member($domain, LifeCourseDomainVocabulary)",
        ),
        (
            "family-life-no-role-vocabulary",
            "member($role_kind, LifeCourseRoleKindVocabulary)",
        ),
        ("family-life-no-participant-separation", PARTICIPANT_SEPARATION),
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
        let values = bindings(&source, contract, "LifeCourseCase");
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
                return Err(Error::new(format!("empty family-life omission {scope}")));
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
            ("$reader", "LifeCourseChallengeReaderAuthority"),
            ("$alternate", "LifeCourseAlternateReviewAuthority"),
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
                        .chain(std::iter::once(&"UnapprovedLifeCourseKind".into()))
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
            "CurrentReconciledLifeCourseRecord",
            "SupersededLifeCourseRecord",
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
                    "family-life-no-independent-review",
                    &own,
                    &other,
                    true,
                )?;
            }
        }
        for (label, scope, variable) in [
            (
                "mismatched-source",
                "LifeCourseSourceVersionScope",
                "$version",
            ),
            ("period-drift", "LifeCourseEvidencePeriodScope", "$period"),
        ] {
            let old = ground(
                &format!("observe($review, $record, {variable}, {scope})."),
                &values,
            );
            let new = ground(
                &format!("observe($review, $record, DifferentLifeCourseValue, {scope})."),
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
                ("wrong-role-kind", "$role_kind"),
            ] {
                let mut other = values.clone();
                other.insert(
                    variable.into(),
                    match variable {
                        "$domain" => "ReproductiveAndBodilyCare".into(),
                        "$role_kind" => "PropertyAndSuccession".into(),
                        _ => "UnrelatedLifeCourseValue".into(),
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

    // A role is reviewed, a participation runs on it, and then the role is
    // found defective. Both stop. Nothing that happened reached the standing,
    // floor or ballot of the person the decision was about.
    let participation = get("participation");
    let values = bindings(source, participation, "LifeCourseSequence");
    let affected = values["$participant"].clone();
    let full = dependency(source, participation, &values)
        + &fixture(source, participation, &values)
        + &format!("person({affected}).\nat({affected}, GeneralAdulthood).\n");
    let defect = get("defect");
    let mut against_role = bindings(source, defect, "LifeCourseRoleDefect");
    against_role.insert("$target".into(), values["$authorization"].clone());
    against_role.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    for name in CARRIED {
        against_role.insert(name.into(), values[name].clone());
    }
    let role = get("role");
    let mut authorization = bindings(source, role, "LifeCourseAuthorization");
    for name in CARRIED.into_iter().chain(["$role_kind"]) {
        authorization.insert(name.into(), values[name].clone());
    }
    authorization.insert("$record".into(), values["$authorization"].clone());
    authorization.insert("$source".into(), values["$authorization_source"].clone());

    let mut steps = queries(role, &authorization, true);
    steps.push_str(&queries(participation, &values, true));
    steps.push_str(&fixture(source, defect, &against_role));
    steps.push_str(&queries(defect, &against_role, true));
    steps.push_str(&queries(role, &authorization, false));
    steps.push_str(&queries(participation, &values, false));
    for (atom, expected) in [
        (format!("person({affected})"), true),
        (format!("owe(State, Eats, {affected})"), true),
        (format!("decide({affected}, Ballot)"), true),
        (format!("false({affected})"), false),
        (format!("prisoner({affected})"), false),
        (format!("lose(Points, {affected})"), false),
        (format!("mature({affected})"), false),
        ("person(Cira)".into(), true),
        ("owe(State, Eats, Cira)".into(), true),
        ("prisoner(Cira)".into(), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "life-course/role-defect-and-rights-sequence",
        "live",
        &full,
        &steps,
    )?;

    // A defect about a different record leaves this one standing.
    let mut elsewhere = against_role.clone();
    elsewhere.insert("$target".into(), "DifferentLifeCourseRecord".into());
    let facts = full.clone()
        + &dependency(source, defect, &elsewhere)
        + &fixture(source, defect, &elsewhere);
    add_case(
        context,
        export,
        "life-course/unrelated-defect",
        "live",
        &facts,
        &(queries(participation, &values, true) + &queries(defect, &elsewhere, true)),
    )?;

    // A role reviewed for one decision domain licenses nothing in another. The
    // rejoin refuses it, not the label.
    let mut other_domain = values.clone();
    other_domain.insert("$domain".into(), "ReproductiveAndBodilyCare".into());
    add_case(
        context,
        export,
        "life-course/role-does-not-travel-between-domains",
        "live",
        &(dependency(source, participation, &values)
            + &fixture(source, participation, &other_domain)),
        &queries(participation, &other_domain, false),
    )?;

    // Public care continuity is immediate and non-delegable: it completes with
    // no family role in the record at all, and it confers no power over
    // anybody. This is the half the baseline's barriers presupposed.
    let continuity = get("continuity");
    let continuity_values = bindings(source, continuity, "LifeCourseContinuity");
    let mut steps = queries(continuity, &continuity_values, true);
    for (atom, expected) in [
        (
            format!(
                "complete({}, {AUTHORIZATION}, {})",
                continuity_values["$record"], continuity_values["$decision"]
            ),
            false,
        ),
        (
            format!(
                "authority({}, Parentage, {})",
                continuity_values["$actor"], continuity_values["$record"]
            ),
            false,
        ),
        (format!("person({})", continuity_values["$actor"]), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "life-course/continuity-needs-no-family-role",
        "live",
        &fixture(source, continuity, &continuity_values),
        &steps,
    )?;

    // Support assists; it never replaces the person's own decision, and it
    // confers no role over them.
    let support = get("support");
    let support_values = bindings(source, support, "LifeCourseSupport");
    let mut steps = queries(support, &support_values, true);
    for (atom, expected) in [
        (
            format!(
                "complete({}, {AUTHORIZATION}, {})",
                support_values["$record"], support_values["$decision"]
            ),
            false,
        ),
        (
            format!(
                "permits({}, ActOnTheBoundedBestInterpretation, {})",
                support_values["$supporter"], support_values["$record"]
            ),
            false,
        ),
        (format!("person({})", support_values["$supporter"]), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "life-course/support-is-not-substitution",
        "live",
        &fixture(source, support, &support_values),
        &steps,
    )?;

    // The bounded best-interpretation route needs positive evidence that
    // support was actually provided first. Remove that one attestation and the
    // route does not derive: the last resort cannot be reached by skipping the
    // step before it.
    let interpretation = get("interpretation");
    let interpretation_values = bindings(source, interpretation, "LifeCourseInterpretation");
    let own = fixture(source, interpretation, &interpretation_values);
    let without_support = without_scope(&own, "LifeCourseSupportFirstScope");
    if without_support == own {
        return Err(Error::new("support-first attestation is not in the fixture"));
    }
    add_case(
        context,
        export,
        "life-course/interpretation-without-support-first",
        "live",
        &(dependency(source, interpretation, &interpretation_values) + &without_support),
        &queries(interpretation, &interpretation_values, false),
    )?;

    // Own-record access is bound to the person the record is about and hands
    // back no contact right and no reusable classification.
    let origin = get("origin");
    let origin_values = bindings(source, origin, "LifeCourseOrigin");
    let subject = origin_values["$subject_of_the_record"].clone();
    let mut steps = queries(origin, &origin_values, true);
    for (atom, expected) in [
        (
            format!(
                "permits({}, AccessOwnOriginRecord, {})",
                origin_values["$actor"], origin_values["$record"]
            ),
            false,
        ),
        (
            format!(
                "complete({}, {AUTHORIZATION}, {})",
                origin_values["$record"], origin_values["$decision"]
            ),
            false,
        ),
        (format!("person({subject})"), false),
        (format!("false({subject})"), false),
    ] {
        steps.push_str(&query(&atom, expected));
    }
    add_case(
        context,
        export,
        "life-course/own-record-access-confers-no-relationship",
        "live",
        &fixture(source, origin, &origin_values),
        &steps,
    )?;

    // Asking for review creates the reader's duty; a certified nonresponse
    // moves it to the alternate without deciding the request either way.
    let nonresponse = get("nonresponse");
    let mut request = bindings(source, nonresponse, "LifeCourseRequest");
    request.insert("$target".into(), "LifeCourseRequestRecord".into());
    let requester = request["$requester"].clone();
    let reader = request["$reader"].clone();
    let facts = format!(
        "challenge({requester}, {reader}, LifeCourseRequestRecord).\nauthorized({reader}, LifeCourseChallengeReaderAuthority, LifeCourseRequestRecord).\nobserve({requester}, LifeCourseRequestRecord, LifeCourseRequestDecision, LifeCourseChallengeDecisionScope).\n"
    );
    let mut steps = query(
        &format!(
            "obliged({reader}, ReviewTheLifeCourseRequestAccessOrCorrection, LifeCourseRequestRecord)"
        ),
        true,
    );
    steps.push_str(&queries(nonresponse, &request, false));
    steps.push_str(&fixture(source, nonresponse, &request));
    steps.push_str(&queries(nonresponse, &request, true));
    steps.push_str(&query(
        &format!(
            "obliged({reader}, ReviewTheLifeCourseRequestAccessOrCorrection, LifeCourseRequestRecord)"
        ),
        true,
    ));
    add_case(
        context,
        export,
        "life-course/request-and-certified-nonresponse",
        "live",
        &facts,
        &steps,
    )?;

    // Every head this family concludes is declared derived-only upstream, so a
    // forged entry is refused at assertion rather than believed.
    let forged = concat!(
        ":refuse reasoning /declared derived-only/\n",
        "authority(LifeCourseActor, Parentage, LifeCourseForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "complete(LifeCourseForgedRecord, ReviewedLifeCourseRole, LifeCourseDecision).\n",
        ":refuse reasoning /declared derived-only/\n",
        "permits(LifeCourseActor, ActOnTheBoundedBestInterpretation, LifeCourseForgedRecord).\n",
        ":refuse reasoning /declared derived-only/\n",
        "related(LifeCourseForgedRecord, LifeCourseRecordAmbiguity).\n",
        ":refuse reasoning /declared derived-only/\n",
        "contradict(LifeCourseForgedRecord, LifeCourseRecordAuthorization).\n",
    );
    add_case(
        context,
        export,
        "life-course/forged-conclusions",
        "live",
        "",
        forged,
    )?;
    Ok(())
}
