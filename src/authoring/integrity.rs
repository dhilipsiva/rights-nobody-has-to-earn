// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit semantic authoring for the 2026-09-09 integrity rulings.
//! No findings, duties, or verification results are persisted by the verifier.

use super::{Base, Edit, Export};
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "new-book-plans/integrity-source.json";
const BEGIN: &str = "# <DEMOCRATIC-INTEGRITY-RULES-BEGIN>";
const END: &str = "# <DEMOCRATIC-INTEGRITY-RULES-END>";
const INDEPENDENT: &str = "~($source = $review)";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    vocabularies: Vec<(String, String, Vec<String>)>,
    contracts: Vec<Contract>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    id: String,
    kind: String,
    fields: Vec<[String; 2]>,
    extra: Vec<String>,
    heads: Vec<String>,
    bindings: BTreeMap<String, String>,
    dependency: Option<String>,
}

fn fields(contract: &Contract) -> Vec<[String; 2]> {
    let mut fields = [
        ("$target", "IntegrityTargetScope"),
        ("$version", "IntegritySourceVersionScope"),
        ("$jurisdiction", "IntegrityJurisdictionScope"),
        ("$legal_scope", "IntegrityLegalScope"),
        ("$end", "IntegritySourceBoundEndScope"),
        (
            "CurrentReconciledIntegrityRecord",
            "IntegrityCurrentDispositionScope",
        ),
        (
            "IndependentlyReviewedEvidence",
            "IntegrityEvidenceDispositionScope",
        ),
        ("$reader", "IntegrityReaderScope"),
        ("$alternate", "IntegrityIndependentAlternateScope"),
        ("$challenge", "IntegrityChallengeScope"),
        ("$correction", "IntegrityCorrectionScope"),
        ("$remedy", "IntegrityRemedyScope"),
        (
            "CourtOrIndependentSubstitutePanel",
            "IntegrityFinalReviewScope",
        ),
        (
            "NoPersonalStatusFloorBallotOrLibertyConsequence",
            "IntegrityCorridorScope",
        ),
        (
            "ProportionatePurposeLimitedPrivateRecord",
            "IntegrityPrivacyScope",
        ),
    ]
    .into_iter()
    .map(|(v, s)| [v.to_owned(), s.to_owned()])
    .collect::<Vec<_>>();
    fields.push([contract.kind.clone(), "IntegrityRecordKindScope".into()]);
    fields.extend(contract.fields.clone());
    fields
}

fn premises(source: &Source, contract: &Contract) -> Vec<String> {
    let actors = ["$source", "$evidence", "$review"];
    let mut atoms = vec![
        "authorized($source, ElectoralOrIntegritySourceAuthority, $record)".into(),
        "authorized($evidence, IndependentIntegrityEvidenceAuthority, $record)".into(),
        "authorized($review, IndependentIntegrityReviewAuthority, $record)".into(),
        "authorized($reader, IntegrityFindingReaderAuthority, $record)".into(),
        "authorized($alternate, IndependentIntegrityAlternateAuthority, $record)".into(),
        "~($source = $evidence)".into(),
        INDEPENDENT.into(),
        "~($evidence = $review)".into(),
        "~($reader = $alternate)".into(),
    ];
    for [value, scope] in fields(contract) {
        for actor in actors {
            atoms.push(format!("observe({actor}, $record, {value}, {scope})"));
        }
        if let Some((_, vocabulary, _)) = source.vocabularies.iter().find(|(s, _, _)| s == &scope) {
            atoms.push(format!("member({value}, {vocabulary})"));
        }
        if value.starts_with('$')
            && matches!(
                scope.as_str(),
                "IntegrityTargetScope"
                    | "IntegrityOperationScope"
                    | "IntegrityCandidateFieldingAssociationScope"
                    | "IntegrityElectedMemberOrGroupScope"
                    | "IntegrityOfficeBearerScope"
                    | "IntegrityRecipientScope"
            )
        {
            for actor in actors {
                atoms.push(format!("~({actor} = {value})"));
            }
        }
    }
    atoms.extend(contract.extra.clone());
    atoms
}

fn rule(atoms: &[String], head: &str) -> String {
    let joined = format!("{} {head}", atoms.join(" "));
    let regex = Regex::new(r"\$[a-z][a-z0-9_]*").unwrap();
    let variables = regex
        .find_iter(&joined)
        .map(|m| m.as_str())
        .collect::<BTreeSet<_>>();
    let quantifier = variables
        .into_iter()
        .map(|v| format!("all {v}: "))
        .collect::<String>();
    format!("{quantifier}{} -> {head}.", atoms.join(" & "))
}

fn heads(contract: &Contract) -> Vec<String> {
    let mut heads = contract.heads.clone();
    let complete = format!("complete($record, {}, $target)", contract.kind);
    if !heads.contains(&complete) {
        heads.push(complete);
    }
    heads
}

fn rules(source: &Source) -> Vec<String> {
    let mut rules = Vec::new();
    for (scope, vocabulary, members) in &source.vocabularies {
        for member in members {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {member}, {scope}) -> member({member}, {vocabulary})."));
        }
    }
    for contract in &source.contracts {
        let atoms = premises(source, contract);
        for head in heads(contract) {
            rules.push(rule(&atoms, &head));
        }
    }
    // Dilution invokes the existing substantive-equality review, not a new
    // metric or an inference about an individual's protected ground.
    let contract = source
        .contracts
        .iter()
        .find(|c| c.id == "district-finding")
        .unwrap();
    let mut atoms = premises(source, contract);
    atoms.push("$purpose = ProtectedGroundDilution".into());
    rules.push(rule(
        &atoms,
        "obliged($reader, ReferDistrictPlanToSubstantiveEqualityReview, $target)",
    ));
    let contract = source
        .contracts
        .iter()
        .find(|c| c.id == "office-finding")
        .unwrap();
    let mut atoms = premises(source, contract);
    atoms.push("$mode = FormerHolderCounterpartyDealing".into());
    rules.push(rule(
        &atoms,
        "prevents($target, FormerHolderCounterpartyRepresentation)",
    ));
    rules
}

fn grounding(source: &Source, contract: &Contract, prefix: &str) -> BTreeMap<String, String> {
    let text = format!(
        "{} {}",
        premises(source, contract).join(" "),
        heads(contract).join(" ")
    );
    let regex = Regex::new(r"\$[a-z][a-z0-9_]*").unwrap();
    let mut bindings = BTreeMap::new();
    for v in regex.find_iter(&text).map(|m| m.as_str()) {
        let suffix = v
            .trim_start_matches('$')
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
            })
            .collect::<String>();
        bindings.insert(v.to_owned(), format!("{prefix}{suffix}"));
    }
    for [value, scope] in fields(contract) {
        if value.starts_with('$') {
            if let Some((_, _, members)) = source.vocabularies.iter().find(|(s, _, _)| s == &scope)
            {
                bindings.insert(value, members[0].clone());
            }
        }
    }
    bindings.extend(contract.bindings.clone());
    bindings
}

fn ground(text: &str, bindings: &BTreeMap<String, String>) -> String {
    Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .replace_all(text, |c: &regex::Captures<'_>| {
            bindings.get(&c[0]).unwrap().clone()
        })
        .into_owned()
}

fn fixture(
    source: &Source,
    contract: &Contract,
    bindings: &BTreeMap<String, String>,
) -> Vec<String> {
    premises(source, contract)
        .into_iter()
        .filter(|a| a.starts_with("authorized(") || a.starts_with("observe("))
        .map(|a| format!("{}.", ground(&a, bindings)))
        .collect()
}

fn queries(contract: &Contract, bindings: &BTreeMap<String, String>, expected: bool) -> String {
    heads(contract)
        .into_iter()
        .map(|head| {
            format!(
                "? {}.\n# => {}\n",
                ground(&head, bindings),
                if expected { "TRUE" } else { "FALSE" }
            )
        })
        .collect()
}

fn dependency(context: &Context, contract: &Contract) -> Result<String, Error> {
    let Some(path) = &contract.dependency else {
        return Ok(String::new());
    };
    Ok(context
        .read(path)?
        .lines()
        .take_while(|l| !l.trim_start().starts_with('?'))
        .filter(|l| !l.trim_start().starts_with(['#', ':']) && !l.trim().is_empty())
        .map(|l| format!("{l}\n"))
        .collect())
}

fn add_case(
    context: &Context,
    export: &mut Export,
    id: &str,
    base: &str,
    fixture: &str,
    queries: &str,
) -> Result<(), Error> {
    let count = queries.lines().filter(|l| l.starts_with('?')).count();
    let pins = format!(":expect-pins {count}\n\n{queries}");
    export.add_case(
        context,
        &format!("integrity/{id}"),
        base,
        fixture,
        &[("expect", &pins)],
        vec![],
        true,
    )
}

pub(crate) fn generate(context: &Context, export: &mut Export) -> Result<(), Error> {
    let source: Source = serde_json::from_str(&context.read(SOURCE)?)?;
    let rules = rules(&source);
    let block = format!(
        "{BEGIN}\n# Supplied, current, independent records only; no authentication, clock, or arrival.\n{}\n{END}",
        rules.join("\n")
    );
    let path = "new-book-plans/constitution.nibli";
    let constitution = context.read(path)?;
    let updated = if let Some((before, rest)) = constitution.split_once(BEGIN) {
        let (_, after) = rest
            .split_once(END)
            .ok_or_else(|| Error::new("missing integrity end marker"))?;
        format!("{before}{block}{after}")
    } else {
        format!("{}\n\n{block}\n", constitution.trim_end())
    };
    std::fs::write(context.path(path), updated)?;
    let edits = rules
        .iter()
        .filter(|r| r.contains(INDEPENDENT))
        .map(|r| Edit {
            before: r.clone(),
            after: r.replace(&format!(" & {INDEPENDENT}"), ""),
        })
        .collect();
    export.bases.insert(
        "integrity-no-independent-review".into(),
        Base {
            base: Some("live".into()),
            edits,
            path: None,
        },
    );

    for contract in &source.contracts {
        let bindings = grounding(&source, contract, "IntegrityCase");
        let facts = fixture(&source, contract, &bindings);
        let dependency = dependency(context, contract)?;
        let full = format!("{dependency}{}\n", facts.join("\n"));
        add_case(
            context,
            export,
            &format!("{}/positive", contract.id),
            "live",
            &full,
            &queries(contract, &bindings, true),
        )?;
        add_case(
            context,
            export,
            &format!("{}/withheld", contract.id),
            "live",
            &dependency,
            &queries(contract, &bindings, false),
        )?;
        for scope in fields(contract)
            .into_iter()
            .map(|[_, scope]| scope)
            .collect::<BTreeSet<_>>()
        {
            let remaining = facts
                .iter()
                .filter(|f| !f.ends_with(&format!(", {scope}).")))
                .cloned()
                .collect::<Vec<_>>();
            if remaining.len() == facts.len() {
                return Err(Error::new(format!("empty integrity omission {scope}")));
            }
            add_case(
                context,
                export,
                &format!("{}/without-{scope}", contract.id),
                "live",
                &format!("{dependency}{}\n", remaining.join("\n")),
                &queries(contract, &bindings, false),
            )?;
        }
        for [value, scope] in fields(contract) {
            if let Some((_, _, members)) = source.vocabularies.iter().find(|(s, _, _)| s == &scope)
            {
                if value.starts_with('$') {
                    for member in members
                        .iter()
                        .chain(std::iter::once(&"UnratifiedIntegrityKind".to_owned()))
                    {
                        let mut other = bindings.clone();
                        other.insert(value.clone(), member.clone());
                        add_case(
                            context,
                            export,
                            &format!("{}/kind-{scope}-{member}", contract.id),
                            "live",
                            &format!(
                                "{dependency}{}\n",
                                fixture(&source, contract, &other).join("\n")
                            ),
                            &queries(contract, &other, members.contains(member)),
                        )?;
                    }
                }
            }
        }
        let stale = full.replace(
            "CurrentReconciledIntegrityRecord, IntegrityCurrentDispositionScope",
            "SupersededIntegrityRecord, IntegrityCurrentDispositionScope",
        );
        add_case(
            context,
            export,
            &format!("{}/stale", contract.id),
            "live",
            &stale,
            &queries(contract, &bindings, false),
        )?;
        let mismatch = full.replace(
            &format!(
                "observe({}, {}, {}, IntegritySourceVersionScope)",
                bindings["$review"], bindings["$record"], bindings["$version"]
            ),
            &format!(
                "observe({}, {}, DifferentSourceVersion, IntegritySourceVersionScope)",
                bindings["$review"], bindings["$record"]
            ),
        );
        add_case(
            context,
            export,
            &format!("{}/mismatched-version", contract.id),
            "live",
            &mismatch,
            &queries(contract, &bindings, false),
        )?;
        let mut fused = bindings.clone();
        fused.insert("$review".into(), fused["$source"].clone());
        let full = format!(
            "{dependency}{}\n",
            fixture(&source, contract, &fused).join("\n")
        );
        for (label, base, expected) in [
            ("self-review", "live", false),
            (
                "counterfactual-self-review",
                "integrity-no-independent-review",
                true,
            ),
        ] {
            add_case(
                context,
                export,
                &format!("{}/{label}", contract.id),
                base,
                &full,
                &queries(contract, &fused, expected),
            )?;
        }
        if contract.dependency.is_some() {
            add_case(
                context,
                export,
                &format!("{}/no-public-scale-finding", contract.id),
                "live",
                &format!("{}\n", facts.join("\n")),
                &queries(contract, &bindings, false),
            )?;
            let mut other = bindings.clone();
            other.insert("$actor".into(), "UnrelatedOperation".into());
            // Do not write a matching economic identity: that finding remains
            // about the original operation and cannot be borrowed by this one.
            let raw = fixture(&source, contract, &other).join("\n");
            let raw = raw
                .lines()
                .filter(|line| !line.contains("EconomicSubjectScope"))
                .map(|l| format!("{l}\n"))
                .collect::<String>();
            add_case(
                context,
                export,
                &format!("{}/different-public-scale-actor", contract.id),
                "live",
                &format!("{dependency}{raw}"),
                &queries(contract, &other, false),
            )?;
        }
    }
    integration_cases(context, export, &source)?;
    Ok(())
}

fn integration_cases(context: &Context, export: &mut Export, source: &Source) -> Result<(), Error> {
    let (office, mapping, authority) = super::state_form::integrity_fixture(
        context,
        36,
        "adult_resident_candidacy",
        "IntegrityOffice",
    )?;
    let outcome = |query: &str, expected: bool| {
        format!(
            "? {query}.\n# => {}\n",
            if expected { "TRUE" } else { "FALSE" }
        )
    };
    for scope in [
        "PoliticalFinancePayerScope",
        "PoliticalFinanceControllingPayerScope",
    ] {
        let reduced = office
            .lines()
            .filter(|l| !l.ends_with(&format!(", {scope}).")))
            .map(|l| format!("{l}\n"))
            .collect::<String>();
        add_case(
            context,
            export,
            &format!("payer-identity/without-{scope}"),
            "live",
            &reduced,
            &outcome(&authority, false),
        )?;
    }
    let review = &mapping["$review"];
    let result = &mapping["$result"];
    let controller = &mapping["$controlling_payer"];
    let mismatched = office.replace(&format!("observe({review}, {result}, {controller}, PoliticalFinanceControllingPayerScope)"), &format!("observe({review}, {result}, DifferentController, PoliticalFinanceControllingPayerScope)"));
    add_case(
        context,
        export,
        "payer-identity/mismatched-controller",
        "live",
        &mismatched,
        &outcome(&authority, false),
    )?;

    for id in ["money-finding", "office-finding", "opposition-withholding"] {
        let contract = source.contracts.iter().find(|c| c.id == id).unwrap();
        let mut bindings = grounding(source, contract, "IntegrityAdverse");
        bindings.insert("$target".into(), mapping["$record"].clone());
        let facts = fixture(source, contract, &bindings).join("\n");
        let tag = format!(
            "contradict({}, DemocraticIntegrityAuthorization)",
            mapping["$record"]
        );
        let mut pins = outcome(&authority, true) + &outcome(&tag, false);
        pins.push_str(&format!("{facts}\n"));
        pins.push_str(&outcome(&tag, true));
        pins.push_str(&outcome(&authority, false));
        pins.push_str(&outcome("authority(Boss)", true));
        pins.push_str(&outcome("person(Hano)", true));
        pins.push_str(&outcome("decide(Hano, Ballot)", true));
        pins.push_str(&outcome("owe(State, Eats, Hano)", true));
        add_case(
            context,
            export,
            &format!("{id}/act-not-person-sequence"),
            "live",
            &office,
            &pins,
        )?;
        bindings.insert("$target".into(), "DifferentOfficeAct".into());
        add_case(
            context,
            export,
            &format!("{id}/different-act"),
            "live",
            &format!(
                "{office}{}\n",
                fixture(source, contract, &bindings).join("\n")
            ),
            &outcome(&authority, true),
        )?;
    }
    for (lawful_id, finding_id) in [
        ("district-plan", "district-finding"),
        ("party-selection", "party-finding"),
    ] {
        let lawful = source.contracts.iter().find(|c| c.id == lawful_id).unwrap();
        let finding = source
            .contracts
            .iter()
            .find(|c| c.id == finding_id)
            .unwrap();
        let bindings = grounding(source, lawful, "IntegrityLawful");
        let base = fixture(source, lawful, &bindings).join("\n") + "\n";
        let mut adverse = grounding(source, finding, "IntegrityAdverse");
        adverse.insert("$target".into(), bindings["$target"].clone());
        if let Some(association) = bindings.get("$association") {
            adverse.insert("$association".into(), association.clone());
        }
        let mut pins = queries(lawful, &bindings, true);
        pins.push_str(&format!(
            "{}\n",
            fixture(source, finding, &adverse).join("\n")
        ));
        pins.push_str(&queries(lawful, &bindings, false));
        pins.push_str(&queries(finding, &adverse, true));
        pins.push_str(&outcome(
            "authority(FSBOD_06, ProscribeParty, IntegrityLawfulTarget)",
            false,
        ));
        add_case(
            context,
            export,
            &format!("{lawful_id}/adverse-finding-sequence"),
            "live",
            &base,
            &pins,
        )?;
    }
    let district = source
        .contracts
        .iter()
        .find(|c| c.id == "district-finding")
        .unwrap();
    let mut bindings = grounding(source, district, "EqualityDistrict");
    bindings.insert("$purpose".into(), "ProtectedGroundDilution".into());
    let equality = ground(
        "obliged($reader, ReferDistrictPlanToSubstantiveEqualityReview, $target)",
        &bindings,
    );
    add_case(
        context,
        export,
        "district-finding/equality-review-route",
        "live",
        &(fixture(source, district, &bindings).join("\n") + "\n"),
        &outcome(&equality, true),
    )?;
    let (election, mapping, authority) = super::state_form::integrity_fixture(
        context,
        10,
        "assembly_election",
        "IntegrityElection",
    )?;
    bindings.insert("$target".into(), mapping["$district_plan"].clone());
    let mut pins = outcome(&authority, true);
    pins.push_str(&(fixture(source, district, &bindings).join("\n") + "\n"));
    pins.push_str(&outcome(&authority, false));
    add_case(
        context,
        export,
        "district-finding/electoral-result-sequence",
        "live",
        &election,
        &pins,
    )?;
    let office = source
        .contracts
        .iter()
        .find(|c| c.id == "office-finding")
        .unwrap();
    for holder in ["GoverningOfficeHolder", "OppositionOfficeHolder"] {
        let mut bindings = grounding(source, office, holder);
        bindings.insert("$office".into(), holder.into());
        add_case(
            context,
            export,
            &format!("office-finding/equal-treatment-{holder}"),
            "live",
            &(fixture(source, office, &bindings).join("\n") + "\n"),
            &queries(office, &bindings, true),
        )?;
    }
    let mut former = grounding(source, office, "FormerHolderCase");
    former.insert("$mode".into(), "FormerHolderCounterpartyDealing".into());
    let refusal = ground(
        "prevents($target, FormerHolderCounterpartyRepresentation)",
        &former,
    );
    add_case(
        context,
        export,
        "office-finding/former-holder-representation",
        "live",
        &(fixture(source, office, &former).join("\n") + "\n"),
        &outcome(&refusal, true),
    )?;
    let (franchise, mapping, authority) = super::state_form::integrity_fixture(
        context,
        36,
        "adult_resident_franchise",
        "IntegrityFranchise",
    )?;
    let finance = source
        .contracts
        .iter()
        .find(|c| c.id == "money-finding")
        .unwrap();
    let mut bindings = grounding(source, finance, "MisappliedFinance");
    bindings.insert("$target".into(), mapping["$record"].clone());
    let facts = franchise + &fixture(source, finance, &bindings).join("\n") + "\n";
    add_case(
        context,
        export,
        "money-finding/franchise-does-not-consume-finding",
        "live",
        &facts,
        &outcome(&authority, true),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_side_rules_never_classify_people_or_create_a_truth_judge() {
        let context = Context::discover().unwrap();
        let source: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
        let contract = source
            .contracts
            .iter()
            .find(|c| c.id == "coordinated-operation")
            .unwrap();
        let check = |contract: &Contract| -> bool {
            let text = format!(
                "{} {}",
                premises(&source, contract).join(" "),
                heads(contract).join(" ")
            );
            let forbidden = Regex::new(r"(?i)\b(person|belief|influence|susceptibility|exposure|loyalty|score|truth|false|prisoner|decide|owe|pay|promise|reward)\s*\(|\$(person|belief|susceptibility|exposure|loyalty|score)\b").unwrap();
            !forbidden.is_match(&text)
                && contract.fields.iter().all(|f| {
                    matches!(
                        f[1].as_str(),
                        "IntegrityOperationScope"
                            | "IntegrityOperationKindScope"
                            | "IntegrityOperationControllerScope"
                            | "IntegrityOperationFunctionScope"
                            | "IntegrityPublicScaleRecordScope"
                            | "IntegrityFindingDispositionScope"
                            | "IntegrityEvidenceLimitScope"
                            | "IntegrityConsequenceLimitScope"
                    )
                })
                && contract
                    .fields
                    .iter()
                    .any(|f| f[1] == "IntegrityOperationKindScope")
                && contract
                    .extra
                    .iter()
                    .any(|a| a == "authority(FSBOD_09, FSPOW_064, $economic_record)")
        };
        assert!(check(contract));
        let mut poisoned: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
        let poisoned = poisoned
            .contracts
            .iter_mut()
            .find(|c| c.id == "coordinated-operation")
            .unwrap();
        poisoned.extra.push("person($actor)".into());
        assert!(!check(poisoned), "watched person-side premise must fail");
        poisoned.extra.pop();
        poisoned.heads.push("false($actor)".into());
        assert!(
            !check(poisoned),
            "watched person-side consequence must fail"
        );
        poisoned.heads.pop();
        poisoned.fields.push([
            "$actor".into(),
            "IntegrityPersonalSusceptibilityScope".into(),
        ]);
        assert!(
            !check(poisoned),
            "person-side vocabulary must fail even with a neutral variable name"
        );
    }

    #[test]
    fn live_operation_consumers_reject_the_watched_person_side_fixture() {
        let context = Context::discover().unwrap();
        let source: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
        let allowed = rules(&source).into_iter().collect::<BTreeSet<_>>();
        let watched = Regex::new(r"\b(IntegrityOperation\w*|CoordinatedOperationFinding|DiscloseCoordinationAndFunding|AttributeCoordinatedAdvocacy|ProvideCalibratedAccessAuditChallengeAndRemedy)\b").unwrap();
        let check = |text: &str| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.starts_with('#') && watched.is_match(line))
                .all(|line| allowed.contains(line))
        };
        let constitution = context.read("new-book-plans/constitution.nibli").unwrap();
        assert!(
            check(&constitution),
            "an unreviewed operation consumer entered the constitution"
        );
        let hostile = context
            .read("tests/fixtures/integrity-person-side.nibli")
            .unwrap();
        assert!(
            !check(&format!("{constitution}\n{hostile}")),
            "watched fixture must trip the operation-only guard"
        );
        for line in hostile
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        {
            nibli_session::CoreSession::new()
                .compile_text(line)
                .expect("fixture must be real corpus-valid Nibli");
        }
    }
}
