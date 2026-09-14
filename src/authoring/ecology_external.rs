// SPDX-License-Identifier: MIT OR Apache-2.0

//! Full existing sources, including their raw witnesses and current windows.

use super::{Card, Values, records};
use crate::{cli::Error, context::Context};

#[derive(Clone, Copy)]
enum Family {
    StateForm(usize),
    Justice,
    Mobility,
    Economic(usize),
}

impl Family {
    fn consumer(self, context: &Context, branch: &str) -> Result<Vec<String>, Error> {
        match self {
            Self::StateForm(number) => {
                super::super::state_form::ecological_consumer(context, number, branch)
            }
            Self::Justice => super::super::justice::protective_consumer(context, branch),
            Self::Mobility => super::super::mobility::protective_consumer(branch),
            Self::Economic(number) => super::economic::consumer(context, number),
        }
    }

    fn example(
        self,
        context: &Context,
        branch: &str,
        prefix: &str,
        bindings: &[(&str, &str)],
    ) -> Result<(String, Values), Error> {
        match self {
            Self::StateForm(number) => {
                super::super::state_form::court_example(context, number, branch, prefix, bindings)
            }
            Self::Justice => {
                super::super::justice::protective_example(context, branch, prefix, bindings)
            }
            Self::Mobility => super::super::mobility::protective_example(branch, prefix, bindings),
            Self::Economic(number) => super::economic::example(context, number, prefix, bindings),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Link {
    branches: Vec<&'static str>,
    selected: usize,
    mapping: Values,
    templates: Vec<String>,
    pub defaults: Values,
    pub alternatives: Option<(String, Vec<String>)>,
}

impl Link {
    pub fn fixture_variables(&self) -> impl Iterator<Item = &str> {
        self.mapping
            .values()
            .filter(|value| value.starts_with('$'))
            .map(String::as_str)
    }

    pub fn fixture(&self, values: &Values) -> String {
        records::ground(&self.templates[self.selected], values)
    }
}

fn add(
    context: &Context,
    card: &mut Card,
    key: &str,
    number: usize,
    branches: &[&'static str],
    bindings: &[(&str, &str)],
) -> Result<(), Error> {
    add_family(
        context,
        card,
        key,
        Family::StateForm(number),
        branches,
        bindings,
    )
}

fn add_family(
    context: &Context,
    card: &mut Card,
    key: &str,
    family: Family,
    branches: &[&'static str],
    bindings: &[(&str, &str)],
) -> Result<(), Error> {
    if branches.is_empty() {
        return Err(Error::new("empty ecological upstream alternatives"));
    }
    let prefix = format!("EC{}{}", card.id.replace('-', ""), key);
    let mut values = Values::new();
    let mut choices = Vec::new();
    let mut branch_variables = Vec::new();
    for branch in branches.iter().rev() {
        choices.push(family.consumer(context, branch)?);
        let branch_values = family.example(context, branch, &prefix, &[])?.1;
        branch_variables.push(branch_values.keys().cloned().collect::<Vec<_>>());
        values.extend(branch_values);
    }
    branch_variables.reverse();
    let mut link = Link {
        branches: branches.to_vec(),
        selected: 0,
        templates: Vec::new(),
        mapping: Values::new(),
        defaults: Values::new(),
        alternatives: None,
    };
    for variable in values.keys() {
        link.mapping.insert(
            variable.clone(),
            format!(
                "$external_{}_{}",
                key.to_lowercase(),
                variable.trim_start_matches('$')
            ),
        );
    }
    for (from, to) in bindings {
        if !link.mapping.contains_key(*from) {
            return Err(Error::new(format!(
                "{}: unknown {key} binding {from}",
                card.id
            )));
        }
        link.mapping.insert((*from).into(), (*to).into());
    }
    for (variable, value) in values {
        let target = &link.mapping[&variable];
        if target.starts_with('$') && !value.starts_with(&prefix) {
            link.defaults.insert(target.clone(), value);
        }
    }
    // Build genuine upstream fixtures once during explicit authoring. Rebind
    // every variable (including deep witnesses) without caching any verdict.
    let placeholder = regex::Regex::new(r"ECUpstreamPlaceholder[0-9]+\b").unwrap();
    for (branch, variables) in branches.iter().zip(branch_variables) {
        let mut replacements = Values::new();
        let owned: Vec<_> = variables
            .iter()
            .enumerate()
            .map(|(index, variable)| {
                let marker = format!("ECUpstreamPlaceholder{index}");
                replacements.insert(marker.clone(), link.mapping[variable].clone());
                (variable.as_str(), marker)
            })
            .collect();
        let bindings: Vec<_> = owned
            .iter()
            .map(|(variable, marker)| (*variable, marker.as_str()))
            .collect();
        let facts = family.example(context, branch, &prefix, &bindings)?.0;
        link.templates.push(
            placeholder
                .replace_all(&facts, |m: &regex::Captures<'_>| {
                    replacements[&m[0]].clone()
                })
                .into_owned(),
        );
    }
    let rename = |atom: &str| {
        records::pattern()
            .replace_all(atom, |m: &regex::Captures<'_>| {
                link.mapping
                    .get(&m[0])
                    .unwrap_or_else(|| panic!("unmapped {key} variable {}", &m[0]))
                    .clone()
            })
            .into_owned()
    };
    let mut mapped_choices: Vec<Vec<String>> = choices
        .iter()
        .map(|atoms| atoms.iter().map(|atom| rename(atom)).collect())
        .collect();
    for (original, mapped) in choices.iter().zip(&mut mapped_choices) {
        let present: std::collections::BTreeSet<_> = original
            .iter()
            .flat_map(|atom| records::pattern().find_iter(atom).map(|m| m.as_str()))
            .collect();
        for reviewer in ["$review", "$reader", "$alternate", "$auditor"] {
            for actor in [
                "$source",
                "$evidence",
                "$review",
                "$executor",
                "$admin",
                "$assurer",
                "$service",
                "$operator",
                "$adjudicator",
                "$reader",
                "$alternate",
                "$defender",
            ] {
                if present.contains(actor) {
                    mapped.push(format!("~({reviewer} = {})", link.mapping[actor]));
                }
            }
        }
    }
    if mapped_choices.len() == 1 {
        card.extra.extend(mapped_choices.pop().unwrap());
    } else {
        // A disjunction of full existing contracts, not their common subset.
        let alternatives: Vec<_> = mapped_choices
            .iter()
            .map(|atoms| atoms.join(" & "))
            .collect();
        let expression = format!(
            "({})",
            alternatives
                .iter()
                .map(|body| format!("({body})"))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        card.extra.push(expression.clone());
        link.alternatives = Some((expression, alternatives));
    }
    for variable in [
        "$record",
        "$result",
        "$temporal_record",
        "$reconciliation",
        "$result_reconciliation",
    ] {
        if let Some(value) = link.mapping.get(variable) {
            card.extra
                .push(format!("~related({value}, ECExternalIdentityAmbiguity)"));
        }
    }
    for variable in [
        "$record",
        "$result",
        "$source",
        "$evidence",
        "$review",
        "$executor",
        "$temporal_record",
        "$temporal",
        "$temporal_review",
        "$version",
        "$epoch",
        "$end",
        "$jurisdiction",
        "$legal_scope",
        "$case",
        "$subject",
        "$law",
        "$bill",
        "$office",
        "$institution",
        "$seat",
        "$qualified_nominee",
        "$nominee",
        "$selection_certificate",
        "$selector_configuration",
        "$qualification_authority",
        "$fallback_configuration",
        "$term",
        "$scope",
        "$window",
        "$holder",
        "$object",
        "$project",
        "$project_version",
        "$effect",
        "$actual_collective_consent",
        "$obligor",
        "$relief",
        "$adjudicator",
        "$resource",
        "$population",
        "$alternatives_or_allocation",
    ] {
        if let Some(value) = link.mapping.get(variable) {
            let suffix: String = variable
                .trim_start_matches('$')
                .split('_')
                .map(|part| {
                    let (first, rest) = part.split_at(1);
                    format!("{}{rest}", first.to_ascii_uppercase())
                })
                .collect();
            card.external_bindings
                .push((value.clone(), format!("External{key}{suffix}")));
        }
    }
    card.external.push(link);
    Ok(())
}

pub(super) fn connect(context: &Context, cards: &mut [Card]) -> Result<(), Error> {
    for card in cards {
        if matches!(
            card.id,
            "guardian-appointment"
                | "animal-advocate-appointment"
                | "guardian-alternate-appointment"
                | "animal-alternate-appointment"
                | "substitute-reviewer-appointment"
        ) {
            appointments(context, card)?;
        }
        if let Some((_, _, action, _)) = super::animal_remedies::REMEDIES
            .iter()
            .find(|(id, _, _, _)| *id == card.id)
        {
            add(
                context,
                card,
                "AnimalCourt",
                22,
                &["case_specific_relief"],
                &[
                    ("$record", "$order"),
                    ("$case", "$case"),
                    ("$subject", "$subject"),
                    ("$remedy", "$direction"),
                    ("$admitted_facts", "$evidence_record"),
                    ("$reasoned_decision", "$reason"),
                    ("$version", "$version"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$legal_scope", "$scope"),
                ],
            )?;
            let link = card.external.last().unwrap();
            for actor in ["$source", "$review"] {
                card.extra.push(records::observe(
                    &link.mapping[actor],
                    "$order",
                    action,
                    "CourtAnimalRemedyAction",
                ));
                card.extra.push(records::observe(
                    &link.mapping[actor],
                    "$order",
                    "$controller",
                    "CourtAnimalAffectedController",
                ));
            }
        }
        if card.id == "animal-human-justice" {
            for (key, branch) in [
                ("HumanRelief", "case-relief"),
                ("HumanAssistance", "assistance"),
                ("HumanDefence", "defence"),
                ("HumanAppeal", "appeal"),
            ] {
                add_family(
                    context,
                    card,
                    key,
                    Family::Justice,
                    &[branch],
                    &[
                        ("$subject", "$affected_human"),
                        ("$case", "$case"),
                        ("$version", "$version"),
                        ("$epoch", "$epoch"),
                        ("$jurisdiction", "$jurisdiction"),
                        ("$scope", "$scope"),
                        ("$domain", "CriminalJustice"),
                    ],
                )?;
            }
        }
        if matches!(
            card.id,
            "ecological-physical-scarcity" | "ecological-scarcity-allocation"
        ) {
            let number = if card.id == "ecological-physical-scarcity" {
                81
            } else {
                82
            };
            add_family(
                context,
                card,
                "Scarcity",
                Family::Economic(number),
                &["actual-scarcity"],
                &[
                    ("$case", "$case"),
                    ("$resource", "$resource"),
                    ("$population", "$population"),
                    ("$alternatives_or_allocation", "$alternatives_or_allocation"),
                    ("$version", "$version"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$legal_scope", "$scope"),
                ],
            )?;
        }
        if matches!(
            card.id,
            "ecological-collective-consent" | "ecological-collective-consultation"
        ) {
            let consent = card.id == "ecological-collective-consent";
            let mut bindings = vec![
                ("$holder", "$collective_holder"),
                ("$object", "$collective_title"),
                ("$project", "$project"),
                ("$project_version", "$authorization_version"),
                ("$version", "$version"),
                ("$jurisdiction", "$jurisdiction"),
                ("$scope", "$scope"),
            ];
            if consent {
                bindings.push(("$effect", "$effect"));
            }
            add_family(
                context,
                card,
                "Collective",
                Family::Mobility,
                &[if consent {
                    "consented-effect"
                } else {
                    "consultation"
                }],
                &bindings,
            )?;
        }
        match card.id {
            "ceiling" => add(
                context,
                card,
                "Enactment",
                5,
                &["unused_council_return", "same_rule_repassage"],
                &[
                    ("$law", "$standard"),
                    ("$bill", "$bill"),
                    ("$version", "$version"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$legal_scope", "$scope"),
                ],
            )?,
            "ceiling-review"
            | "hazard-liability"
            | "contribution-liability"
            | "human-reparation"
            | "collective-reparation"
            | "animal-guardian-conflict"
            | "guardian-merits"
            | "substitute-guardian-merits"
            | "person-judicial-interim"
            | "association-judicial-interim"
            | "rights-advocate-judicial-interim"
            | "guardian-judicial-interim" => add(
                context,
                card,
                "Court",
                22,
                &["case_specific_relief"],
                &[
                    ("$case", "$case"),
                    ("$subject", "$subject"),
                    (
                        "$remedy",
                        if card.id.ends_with("judicial-interim") {
                            "$judicial_direction"
                        } else {
                            "$remedy"
                        },
                    ),
                    ("$admitted_facts", "$evidence_record"),
                    ("$reasoned_decision", "$reason"),
                    ("$version", "$version"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$legal_scope", "$scope"),
                ],
            )?,
            "high-consequence-basis" => add(
                context,
                card,
                "Delegation",
                5,
                &["unused_council_return", "same_rule_repassage"],
                &[
                    ("$law", "$mandate"),
                    ("$version", "$version"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$legal_scope", "$scope"),
                ],
            )?,
            _ => {}
        }
    }
    Ok(())
}

fn appointments(context: &Context, card: &mut Card) -> Result<(), Error> {
    add(
        context,
        card,
        "Qualification",
        27,
        &["reasoned_qualified_determination"],
        &[
            ("$nominee", "$subject"),
            ("$office", "$office"),
            ("$version", "$version"),
            ("$jurisdiction", "$jurisdiction"),
            ("$legal_scope", "$scope"),
        ],
    )?;
    add(
        context,
        card,
        "Selection",
        28,
        &[
            "people_appointment_selection",
            "assembly_appointment_selection",
            "council_appointment_selection",
            "regional_local_appointment_selection",
        ],
        &[
            ("$qualified_nominee", "$subject"),
            ("$office", "$office"),
            ("$institution", "$office"),
            ("$seat", "$seat"),
            ("$selector_configuration", "$selector_configuration"),
            ("$qualification_authority", "$qualification_authority"),
            ("$fallback_configuration", "$fallback_configuration"),
            ("$term", "$term"),
            ("$version", "$version"),
            ("$jurisdiction", "$jurisdiction"),
            ("$legal_scope", "$scope"),
        ],
    )?;
    add(
        context,
        card,
        "Seat",
        35,
        &[
            "people_seat_allocation",
            "assembly_seat_allocation",
            "council_seat_allocation",
            "regional_local_seat_allocation",
        ],
        &[
            ("$institution", "$office"),
            ("$seat", "$seat"),
            ("$selector_configuration", "$selector_configuration"),
            ("$qualification_authority", "$qualification_authority"),
            ("$fallback_configuration", "$fallback_configuration"),
            ("$term", "$term"),
            ("$version", "$version"),
            ("$jurisdiction", "$jurisdiction"),
            ("$legal_scope", "$scope"),
        ],
    )?;
    for key in ["selection", "seat"] {
        for actor in ["source", "review"] {
            card.extra.push(format!("observe($external_{key}_{actor}, $external_{key}_record, $selector, SelectedHolderScope)"));
        }
    }
    for actor in ["source", "review", "evidence", "executor"] {
        for other in ["source", "review", "evidence", "executor"] {
            card.extra.push(format!(
                "~($external_qualification_{actor} = $external_selection_{other})"
            ));
        }
    }
    Ok(())
}

pub(super) fn rules() -> Vec<String> {
    let mut rules = Vec::new();
    for role in [
        "StateFormSourceAuthority",
        "StateFormTemporalAuthority",
        "StateFormTemporalReviewAuthority",
        "StateFormRecordReviewAuthority",
        "StateFormEvidenceAuthority",
        "IndependentStateFormReviewAuthority",
        "InstitutionalExecutionAuthority",
        "DecisionAdministrationAuthority",
        "IndependentCompletenessAssuranceAuthority",
        "ResultServiceAuthority",
    ] {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, ECExternalIdentityWriter)."));
    }
    for scope in ["ResultScope", "ReconciliationRecordScope"] {
        rules.push(format!("all $writer: all $record: all $next: related($writer, $record, ECExternalIdentityWriter) & observe($writer, $record, $next, {scope}) -> related($writer, $next, ECExternalIdentityWriter)."));
    }
    // Historical category lists and RemedyScope are deliberately multi-valued.
    // These are exact single-valued identities, not a blanket schema audit.
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
        "SelectedHolderScope",
        "ReconciliationRecordScope",
        "ResultScope",
        "EndConditionScope",
        "CaseScope",
        "SubjectScope",
        "LawScope",
        "BillScope",
        "OfficeScope",
        "TargetOfficeScope",
        "InstitutionScope",
        "SeatScope",
        "QualifiedNomineeScope",
        "NomineeScope",
        "SelectionCertificateScope",
        "SelectorConfigurationScope",
        "QualificationAuthorityScope",
        "FallbackConfigurationScope",
        "FiniteNonrenewableStaggeredTermScope",
        "QualificationDecisionRecordScope",
        "DecisionExpiryScope",
        "InvalidatedActionScope",
        "CompositionChallengeScope",
    ] {
        rules.push(format!("all $a: all $b: all $record: all $left: all $right: related($a, $record, ECExternalIdentityWriter) & related($b, $record, ECExternalIdentityWriter) & observe($a, $record, $left, {scope}) & observe($b, $record, $right, {scope}) & ~($left = $right) -> related($record, ECExternalIdentityAmbiguity)."));
    }
    rules
}
