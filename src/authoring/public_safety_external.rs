// SPDX-License-Identifier: MIT OR Apache-2.0

//! Consume existing substantive families without replacing them with labels.

use super::{contracts::Card, records};
use crate::{cli::Error, context::Context};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
enum Family {
    Mobility(&'static str),
    Justice(&'static str),
    Exit(bool),
}

#[derive(Clone, Debug)]
pub(super) struct Link {
    context: Context,
    family: Family,
    pub prefix: String,
    pub mapping: BTreeMap<String, String>,
    pub defaults: BTreeMap<String, String>,
}

impl Link {
    fn example(
        &self,
        prefix: &str,
        bindings: &[(&str, &str)],
    ) -> Result<(String, BTreeMap<String, String>), Error> {
        match self.family {
            Family::Mobility(id) => {
                super::super::mobility::protective_example(id, prefix, bindings)
            }
            Family::Justice(id) => {
                super::super::justice::protective_example(&self.context, id, prefix, bindings)
            }
            Family::Exit(collective) => super::super::state_form::protective_exit_example(
                &self.context,
                collective,
                prefix,
                bindings,
            ),
        }
    }

    pub fn fixture(&self, values: &BTreeMap<String, String>) -> String {
        let bindings = self
            .mapping
            .iter()
            .map(|(from, to)| (from.clone(), records::ground(to, values)))
            .collect::<Vec<_>>();
        let borrowed = bindings
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect::<Vec<_>>();
        self.example(&format!("{}{}", values["$record"], self.prefix), &borrowed)
            .expect("validated external authoring interface")
            .0
    }
}

fn add(
    context: &Context,
    card: &mut Card,
    key: &str,
    family: Family,
    bindings: &[(&str, &str)],
) -> Result<(), Error> {
    let atoms = match family {
        Family::Mobility(id) => super::super::mobility::protective_consumer(id)?,
        Family::Justice(id) => super::super::justice::protective_consumer(context, id)?,
        Family::Exit(collective) => {
            super::super::state_form::protective_exit_consumer(context, collective)?
        }
    };
    let mut link = Link {
        context: context.clone(),
        family,
        prefix: key.into(),
        mapping: BTreeMap::new(),
        defaults: BTreeMap::new(),
    };
    let example_prefix = format!("PS{}{}", card.id.replace('-', ""), key);
    let (_, defaults) = link.example(&example_prefix, &[])?;
    for variable in defaults.keys() {
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
    for (variable, value) in defaults {
        let target = &link.mapping[&variable];
        // Preserve literal enum defaults, not the example's opaque identities.
        // Reusing its record/writer names would collide across sibling cases.
        if target.starts_with('$') && !value.starts_with(&example_prefix) {
            link.defaults.insert(target.clone(), value);
        }
    }
    for atom in atoms {
        card.extra.push(
            records::variable_pattern()
                .replace_all(&atom, |m: &regex::Captures<'_>| {
                    link.mapping
                        .get(&m[0])
                        .unwrap_or_else(|| panic!("unmapped {key} variable {}", &m[0]))
                        .clone()
                })
                .into_owned(),
        );
    }
    if matches!(family, Family::Exit(_)) {
        for variable in [
            "$record",
            "$result",
            "$temporal_record",
            "$reconciliation",
            "$result_reconciliation",
        ] {
            card.extra.push(format!(
                "~related({}, PSExternalBindingAmbiguity)",
                link.mapping[variable]
            ));
        }
        let holder_guard = if matches!(family, Family::Exit(true)) {
            "PSExitConsentUnexpectedHolder"
        } else {
            "PSExitNoImpactUnexpectedHolder"
        };
        card.extra.push(format!(
            "~related({}, {holder_guard})",
            link.mapping["$record"]
        ));
    }
    // Bind the external record and its identity/window in this record too.
    // Subject/case/project mappings below are joins, not free-standing tags.
    for variable in [
        "$record",
        "$source",
        "$version",
        "$epoch",
        "$jurisdiction",
        "$scope",
        "$legal_scope",
        "$window",
        "$end",
        "$holder",
        "$operator",
        "$subject",
        "$object",
        "$case",
        "$project",
        "$project_version",
        "$effect",
        "$proposal",
        "$territory",
        "$settlement",
        "$actual_collective_consent",
    ] {
        if let Some(value) = link.mapping.get(variable) {
            let name = variable
                .trim_start_matches('$')
                .split('_')
                .map(|part| {
                    let (first, rest) = part.split_at(1);
                    format!("{}{rest}", first.to_ascii_uppercase())
                })
                .collect::<String>();
            card.external_bindings
                .push((value.clone(), format!("External{key}{name}")));
        }
    }
    for reviewer in [
        "$authorizer",
        "$review",
        "$reader",
        "$alternate",
        "$auditor",
    ] {
        for actor in [
            "$source",
            "$evidence",
            "$operator",
            "$executor",
            "$court_source",
            "$court_evidence",
            "$court_review",
            "$court_executor",
        ] {
            if let Some(value) = link.mapping.get(actor) {
                card.extra.push(format!("~({reviewer} = {value})"));
            }
        }
    }
    card.external.push(link);
    Ok(())
}

pub(super) fn connect(context: &Context, cards: &mut [Card]) -> Result<(), Error> {
    for card in cards {
        if card.id == "hazard-evacuation" {
            add(
                context,
                card,
                "Evacuation",
                Family::Mobility("evacuation"),
                &[
                    ("$holder", "$collective_holder"),
                    ("$object", "$collective_title"),
                    ("$project", "$evacuation_operation"),
                    ("$project_version", "$operation_revision"),
                    ("$declaration", "$declaration_record"),
                    ("$declaration_version", "$declaration_revision"),
                    ("$hazard", "$hazard"),
                    ("$version", "$version"),
                    ("$epoch", "$epoch"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$scope", "$scope"),
                    ("$window", "$window"),
                ],
            )?;
        }
        if matches!(
            card.id,
            "border-hold" | "pre-expulsion-detention" | "expulsion"
        ) {
            add(
                context,
                card,
                "Asylum",
                Family::Mobility("asylum"),
                &[
                    ("$holder", "$subject"),
                    ("$object", "$case"),
                    ("$version", "$version"),
                    ("$epoch", "$epoch"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$scope", "$scope"),
                ],
            )?;
        }
        if matches!(card.id, "expulsion" | "transfer") {
            let transfer = if card.id == "expulsion" {
                "IndividualExpulsion"
            } else {
                "$transfer_kind"
            };
            add(
                context,
                card,
                "Removal",
                Family::Mobility("removal"),
                &[
                    ("$holder", "$subject"),
                    ("$object", "$case"),
                    ("$version", "$version"),
                    ("$epoch", "$epoch"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$scope", "$scope"),
                    ("$transfer", transfer),
                ],
            )?;
        }
        if card.id == "external-measure" {
            add(
                context,
                card,
                "Arrangement",
                Family::Mobility("external"),
                &[
                    ("$holder", "$subject"),
                    ("$object", "$case"),
                    ("$version", "$version"),
                    ("$epoch", "$epoch"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$scope", "$scope"),
                    ("$instrument", "$external_instrument"),
                ],
            )?;
        }
        if card.id == "pretrial-detention" {
            add(
                context,
                card,
                "Hearing",
                Family::Justice("hearing"),
                &[
                    ("$subject", "$subject"),
                    ("$case", "$case"),
                    ("$version", "$version"),
                    ("$epoch", "$epoch"),
                    ("$jurisdiction", "$jurisdiction"),
                    ("$scope", "$scope"),
                    ("$domain", "CriminalJustice"),
                ],
            )?;
        }
        if matches!(
            card.id,
            "case-remedy" | "general-remedy" | "composition-remedy"
        ) {
            let id = match card.id {
                "case-remedy" => "case-relief",
                "general-remedy" => "general-invalidation",
                _ => "composition-review",
            };
            let mut bindings = vec![
                ("$subject", "$subject"),
                ("$case", "$case"),
                ("$version", "$version"),
                ("$epoch", "$epoch"),
                ("$jurisdiction", "$jurisdiction"),
                ("$scope", "$scope"),
                ("$domain", "AdministrativeJustice"),
                ("$operator", "$operator"),
            ];
            if card.id == "case-remedy" {
                bindings.push(("$executor", "$operator"));
            } else {
                bindings.push(("$target", "$target"));
            }
            add(context, card, "Justice", Family::Justice(id), &bindings)?;
            if card.id == "case-remedy" {
                let link = card.external.last().unwrap();
                let court = &link.mapping["$court_record"];
                // The ordinary court's old case-relief interface does not name
                // an invalidated act. Require that court's own raw witnesses
                // to join the affected protective act to this exact case.
                for writer in ["$court_source", "$court_review"] {
                    card.extra.push(format!(
                        "observe({}, {court}, $target, PSAffectedProtectiveActScope)",
                        link.mapping[writer]
                    ));
                }
                card.extra
                    .push(format!("~related({court}, PSExternalBindingAmbiguity)"));
            }
        }
        if matches!(card.id, "exit-settlement" | "exit-with-consent") {
            let collective = card.id == "exit-with-consent";
            let mut bindings = vec![
                ("$proposal", "$case"),
                ("$affected_population", "$subject"),
                ("$territory", "$territory"),
                ("$settlement", "$settlement"),
                ("$version", "$version"),
                ("$epoch", "$epoch"),
                ("$jurisdiction", "$jurisdiction"),
                ("$legal_scope", "$scope"),
            ];
            if collective {
                bindings.push(("$actual_collective_consent", "$collective_consent_record"));
            }
            add(context, card, "Exit", Family::Exit(collective), &bindings)?;
            if collective {
                add(
                    context,
                    card,
                    "Consent",
                    Family::Mobility("consented-effect"),
                    &[
                        ("$holder", "$collective_holder"),
                        ("$object", "$collective_title"),
                        ("$project", "$settlement"),
                        ("$project_version", "$settlement_revision"),
                        ("$consent_record", "$collective_consent_record"),
                        ("$effect", "$collective_effect"),
                        ("$version", "$version"),
                        ("$epoch", "$epoch"),
                        ("$jurisdiction", "$jurisdiction"),
                        ("$scope", "$scope"),
                    ],
                )?;
            }
        }
    }
    Ok(())
}

/// Fail closed on contradictory authorized identities in old raw shells.
/// Their deliberately multivalued finance/interest categories and RemedyScope
/// are not identity conflicts and are intentionally absent from this list.
pub(super) fn rules() -> Vec<String> {
    let mut result = Vec::new();
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
        result.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, PSExternalBindingWriter)."));
    }
    for scope in ["ResultScope", "ReconciliationRecordScope"] {
        result.push(format!("all $writer: all $record: all $next: related($writer, $record, PSExternalBindingWriter) & observe($writer, $record, $next, {scope}) -> related($writer, $next, PSExternalBindingWriter)."));
    }
    for scope in [
        "SourceFamilyScope",
        "SourceVersionScope",
        "SourceEpochScope",
        "TemporalRecordScope",
        "StateFormRecordScope",
        "PowerScope",
        "JurisdictionScope",
        "AuthorityScope",
        "ReconciliationRecordScope",
        "ResultScope",
        "EndConditionScope",
        "SecessionProposalScope",
        "SecessionConfigurationScope",
        "TerritoryScope",
        "AffectedPopulationScope",
        "OpeningReferendumResultScope",
        "OpeningDispositionScope",
        "FederalAgreementScope",
        "AgreementDispositionScope",
        "RightsAndMinorityReviewScope",
        "RightsReviewDispositionScope",
        "CompleteSecessionSettlementScope",
        "SettlementDispositionScope",
        "CollectiveTitleOrSovereigntyImpactScope",
        "CollectiveImpactDispositionScope",
        "ActualCollectiveConsentScope",
        "CollectiveConsentDispositionScope",
        "AffectedPopulationRosterScope",
        "UniqueFinalSubmissionSetScope",
        "FinalRatificationResultScope",
        "SubmissionDispositionScope",
        "RatificationDispositionScope",
        "StageDispositionScope",
        "ExitDispositionScope",
        "CoercionBoundaryScope",
        "PSAffectedProtectiveActScope",
    ] {
        result.push(format!("all $a: all $b: all $record: all $left: all $right: related($a, $record, PSExternalBindingWriter) & related($b, $record, PSExternalBindingWriter) & observe($a, $record, $left, {scope}) & observe($b, $record, $right, {scope}) & ~($left = $right) -> related($record, PSExternalBindingAmbiguity)."));
    }
    // Final exit with collective consent deliberately has two authority
    // holders. Reject an unlisted holder, not that required multiplicity.
    for (kind, allowed) in [
        ("PSExitNoImpactUnexpectedHolder", &["FSBOD_01"][..]),
        (
            "PSExitConsentUnexpectedHolder",
            &["FSBOD_01", "FSBOD_21"][..],
        ),
    ] {
        let mut atoms = vec![
            "related($writer, $record, PSExternalBindingWriter)".into(),
            "observe($writer, $record, $holder, HolderScope)".into(),
        ];
        atoms.extend(
            allowed
                .iter()
                .map(|holder| format!("~($holder = {holder})")),
        );
        result.push(records::rule(&atoms, &format!("related($record, {kind})")));
    }
    result
}

pub(super) fn scenarios(cards: &[Card]) -> Result<Vec<super::cases::Scenario>, Error> {
    use super::cases::{Scenario, query};
    let mut cases = Vec::new();
    let completion = |card: &Card, values: &BTreeMap<String, String>, expected| {
        query(
            &records::ground(
                &format!("complete($record, {}, $subject)", card.kind),
                values,
            ),
            expected,
        )
    };
    for card in cards.iter().filter(|card| !card.external.is_empty()) {
        let values = records::values(cards, card, "PSLinked");
        let facts = records::fixture(cards, card, &values);
        cases.push(Scenario::new(
            format!("external/{}/positive", card.id),
            facts.clone(),
            completion(card, &values, true),
        ));
        for link in &card.external {
            let value = |key: &str| records::ground(&link.mapping[key], &values);
            let source = value("$source");
            let record = value("$record");
            let (role, scope) = match link.family {
                Family::Mobility(_) => ("MPSourceAuthority", "MPSourceVersionScope"),
                Family::Justice(_) => ("JSourceAuthority", "JSourceVersionScope"),
                Family::Exit(_) => ("StateFormSourceAuthority", "SourceVersionScope"),
            };
            let absent = format!("authorized({source}, {role}, {record}).");
            let missing = facts
                .lines()
                .filter(|line| *line != absent)
                .map(|line| format!("{line}\n"))
                .collect::<String>();
            assert!(
                missing != facts,
                "missing omission target {} {}",
                card.id,
                link.prefix
            );
            cases.push(Scenario::new(
                format!("external/{}/{}/missing-source", card.id, link.prefix),
                missing,
                completion(card, &values, false),
            ));
            let conflict = format!("observe({source}, {record}, OtherExternalVersion, {scope}).\n");
            let other = records::values(cards, card, "PSUnrelated");
            let siblings = facts.clone() + &records::fixture(cards, card, &other);
            let mut steps = completion(card, &values, true) + &completion(card, &other, true);
            steps += &conflict;
            steps += &completion(card, &values, false);
            steps += &completion(card, &other, true);
            cases.push(Scenario::new(
                format!("external/{}/{}/conflict-sequence", card.id, link.prefix),
                siblings,
                steps,
            ));
            let noise = format!(
                "observe(UncredentialedWriter, {record}, OtherExternalVersion, {scope}).\n"
            );
            cases.push(Scenario::new(
                format!("external/{}/{}/uncredentialed-noise", card.id, link.prefix),
                facts.clone() + &noise,
                completion(card, &values, true),
            ));
            let case_scope = match link.family {
                Family::Mobility(_) => "MPObjectScope",
                Family::Justice(_) => "JCaseScope",
                Family::Exit(_) => "SecessionProposalScope",
            };
            let case_record = if matches!(link.family, Family::Exit(_)) {
                value("$result")
            } else {
                record.clone()
            };
            let wrong_case = facts
                .lines()
                .map(|line| {
                    if line.contains(&format!(", {case_record},"))
                        && line.ends_with(&format!(", {case_scope})."))
                    {
                        line.replace(&values["$case"], "UnrelatedCase") + "\n"
                    } else {
                        format!("{line}\n")
                    }
                })
                .collect::<String>();
            // Consent's object is a collective title, not the exit case.
            if !matches!(
                link.family,
                Family::Mobility("consented-effect" | "evacuation")
            ) {
                assert!(wrong_case != facts, "{} {}", card.id, link.prefix);
                cases.push(Scenario::new(
                    format!("external/{}/{}/wrong-case", card.id, link.prefix),
                    wrong_case,
                    completion(card, &values, false),
                ));
            }
            if matches!(link.family, Family::Exit(_)) {
                let steps = query(&format!("authority(FSBOD_01, FSPOW_045, {record})"), true)
                    + &completion(card, &values, false);
                cases.push(Scenario::new(
                    format!("external/{}/old-valid-tuple-is-not-unambiguous", card.id),
                    facts.clone() + &conflict,
                    steps,
                ));
                for actor in ["$admin", "$assurer", "$service"] {
                    let writer = value(actor);
                    let result = value("$result");
                    let conflict = format!(
                        "observe({writer}, {result}, OtherExitProposal, SecessionProposalScope).\n"
                    );
                    cases.push(Scenario::new(
                        format!(
                            "external/{}/conflicting-{}-result",
                            card.id,
                            actor.trim_start_matches('$')
                        ),
                        facts.clone() + &conflict,
                        completion(card, &values, false),
                    ));
                }
            }
        }
        if card.id == "case-remedy" {
            let link = card
                .external
                .iter()
                .find(|link| link.prefix == "Justice")
                .unwrap();
            let court = records::ground(&link.mapping["$court_record"], &values);
            let source = records::ground(&link.mapping["$court_source"], &values);
            let missing = facts
                .lines()
                .filter(|line| !line.ends_with(", PSAffectedProtectiveActScope)."))
                .map(|line| format!("{line}\n"))
                .collect::<String>();
            assert!(missing != facts);
            cases.push(Scenario::new(
                "external/case-remedy/actual-court-act-binding-required",
                missing,
                completion(card, &values, false),
            ));
            let conflict = format!(
                "observe({source}, {court}, UnrelatedProtectiveAct, PSAffectedProtectiveActScope).\n"
            );
            cases.push(Scenario::new(
                "external/case-remedy/wrong-affected-act",
                facts.clone() + &conflict,
                completion(card, &values, false),
            ));
        }
        if card.id == "transfer" {
            for kind in [
                "Extradition",
                "Surrender",
                "Handover",
                "IndividualExpulsion",
                "UnreviewedTransfer",
            ] {
                let mut selected = values.clone();
                selected.insert("$transfer_kind".into(), kind.into());
                cases.push(Scenario::new(
                    format!("external/transfer/{kind}"),
                    records::fixture(cards, card, &selected),
                    completion(
                        card,
                        &selected,
                        matches!(kind, "Extradition" | "Surrender" | "Handover"),
                    ),
                ));
            }
        }
        if card.id == "exit-with-consent" {
            for effect in [
                "PermanentRelocation",
                "TitleExtinguishment",
                "IrreversibleTitleImpairment",
                "SovereigntyTransferOverCollectiveLand",
                "SacredSiteDestruction",
                "HazardousMaterialPlacement",
                "ComparableExistentialHarm",
                "OrdinaryConsultationOnly",
            ] {
                let mut selected = values.clone();
                selected.insert("$collective_effect".into(), effect.into());
                cases.push(Scenario::new(
                    format!("external/exit-with-consent/{effect}"),
                    records::fixture(cards, card, &selected),
                    completion(card, &selected, effect != "OrdinaryConsultationOnly"),
                ));
            }
            let link = card
                .external
                .iter()
                .find(|link| link.prefix == "Consent")
                .unwrap();
            let value = |key: &str| records::ground(&link.mapping[key], &values);
            let mut binding_values = [
                "$holder",
                "$object",
                "$version",
                "$epoch",
                "$jurisdiction",
                "$scope",
                "$window",
            ]
            .iter()
            .map(|key| ((*key).to_owned(), value(key)))
            .collect::<Vec<_>>();
            binding_values.extend([
                ("$target".into(), value("$consent_record")),
                ("$target_source".into(), value("$consent_source")),
                ("$defect".into(), "ConsentCoercionOrBreach".into()),
            ]);
            let borrowed = binding_values
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<Vec<_>>();
            let (defect, _) = super::super::mobility::protective_example(
                "defect",
                "PSActualConsentDefect",
                &borrowed,
            )?;
            let other = records::values(cards, card, "PSOtherExit");
            let steps = completion(card, &values, true)
                + &defect
                + &completion(card, &values, false)
                + &completion(card, &other, true);
            cases.push(Scenario::new(
                "external/exit-with-consent/reviewed-consent-withdrawal",
                facts.clone() + &records::fixture(cards, card, &other),
                steps,
            ));
        }
    }
    Ok(cases)
}
