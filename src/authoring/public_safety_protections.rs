// SPDX-License-Identifier: MIT OR Apache-2.0

//! Positive protected facts and duties, never an authorization to coerce.

use super::{
    cases::{Scenario, query},
    records,
};
use std::collections::{BTreeMap, BTreeSet};

type Values = BTreeMap<String, String>;

struct Fact {
    id: &'static str,
    kind: &'static str,
    fields: Vec<(&'static str, &'static str)>,
    premises: Vec<&'static str>,
    heads: Vec<&'static str>,
}

const WRITERS: &[(&str, &str)] = &[
    ("$source", "PSProtectedFactSourceAuthority"),
    ("$review", "PSIndependentProtectedFactReviewAuthority"),
];

fn facts() -> Vec<Fact> {
    vec![
        Fact {
            id: "protected-disclosure",
            kind: "PSProtectedPublicInterestDisclosureFinding",
            fields: vec![
                ("$recipient", "DisclosureRecipient"),
                ("$implicated", "ImplicatedPower"),
                ("$disclosure", "DisclosureRecord"),
                (
                    "ScopedPublicInterestDisclosureNotProofOfAllegedWrong",
                    "Finding",
                ),
                (
                    "ProtectDiscloserPreserveEvidenceAndIndependentlyAssessActualWrong",
                    "Purpose",
                ),
                (
                    "NoPriorInternalChannelWhereImplicatedNoRetaliationOrSecretPrivilege",
                    "DisclosureAccess",
                ),
                (
                    "IndependentRecipientWithEvidenceAccessAndProportionatePublicationDuty",
                    "RecipientMandate",
                ),
            ],
            premises: vec![
                "authorized($recipient, PSIndependentDisclosureRecipientAuthority, $record)",
                "~($recipient = $subject)",
                "~($recipient = $implicated)",
                "~($recipient = $source)",
                "~($recipient = $review)",
                "~($source = $implicated)",
                "~($review = $implicated)",
            ],
            heads: vec![
                "obliged($recipient, ProtectDisclosurePreserveAndAccessEvidenceWithProportionatePublication, $record)",
                "obliged(State, ProtectPublicInterestDiscloserAgainstRetaliationAndSecureRemedy, $subject)",
            ],
        },
        Fact {
            id: "family",
            kind: "PSPositivelyWitnessedFamilyRelationship",
            fields: vec![
                (
                    "ActualFamilyRelationshipNotInferenceFromResidenceMarriageOrDependency",
                    "Finding",
                ),
                (
                    "RelationshipRecognitionCorrectionAndCareOnlyNoWorthOrConfinementUse",
                    "Purpose",
                ),
                (
                    "NoCompulsoryDisclosureOrSingleApprovedFamilyForm",
                    "FamilyLiberty",
                ),
            ],
            premises: vec!["person($subject)"],
            heads: vec!["family($subject)"],
        },
        Fact {
            id: "parent",
            kind: "PSPositivelyWitnessedParentRelationship",
            fields: vec![
                ("$child", "Child"),
                (
                    "ActualParentRelationshipNoBiologicalMarriageOrHouseholdPresumption",
                    "Finding",
                ),
                (
                    "RelationshipRecognitionCorrectionChildProtectionAndCareOnly",
                    "Purpose",
                ),
                (
                    "NoCompulsoryDisclosureOrUnrelatedConfinementGround",
                    "FamilyLiberty",
                ),
            ],
            premises: vec![
                "person($subject)",
                "person($child)",
                "~($subject = $child)",
                "~($source = $child)",
                "~($review = $child)",
            ],
            heads: vec!["parent($subject, $child)"],
        },
        Fact {
            id: "actual-holding",
            kind: "PSPositiveActualPublicHoldingFinding",
            fields: vec![
                ("$holding_actor", "HoldingActor"),
                ("$place", "KnownOrIndependentlySoughtHoldingPlace"),
                ("$holding_kind", "HoldingKind"),
                ("$claimed_order", "ClaimedOrderOrNoOrderRecorded"),
                (
                    "PositiveCurrentPhysicalControlByOrForPublicPower",
                    "Finding",
                ),
                (
                    "SecureHumaneConditionsVoiceCounselIndependentAccessAndReview",
                    "Purpose",
                ),
                (
                    "NoLawfulOrderOfficialAcknowledgementConvictionOrRequestRequired",
                    "HoldingBoundary",
                ),
            ],
            premises: vec![
                "member($holding_kind, PSActualHoldingKind)",
                "~($source = $holding_actor)",
                "~($review = $holding_actor)",
            ],
            heads: vec![
                "person($subject)",
                "obliged($holding_actor, SecureShelterRecordedVoiceCareConfidentialCounselAndChosenNotification, $record)",
                "obliged(State, SecureAutomaticIndependentJudicialHoldingReviewAndKnownPlace, $record)",
                "obliged(State, PreserveIndependentAccessEvidenceReleaseRemedyAndContinuity, $record)",
            ],
        },
        Fact {
            id: "force-incident",
            kind: "PSPositiveProtectiveIncidentFinding",
            fields: vec![
                ("$deploying_body", "DeployingBody"),
                ("$commander", "ResponsibleCommand"),
                ("$incident", "IncidentKind"),
                ("$investigator", "IndependentInvestigationBody"),
                (
                    "PositiveProtectiveActIncidentNotFindingOfGuiltOrLawfulness",
                    "Finding",
                ),
                (
                    "InvestigateDeathSeriousInjuryTortureAndDisappearancePreserveEvidenceAndParticipation",
                    "Purpose",
                ),
                (
                    "StateBurdenNoOrdersDefenceIndividualAndCommandAccountability",
                    "Accountability",
                ),
                (
                    "IndependentAppointmentBudgetStaffAccessPublicationAndActionDuty",
                    "InvestigatorIndependence",
                ),
            ],
            premises: vec![
                "member($incident, PSProtectedIncidentKind)",
                "authorized($investigator, PSIndependentProtectiveInvestigationAuthority, $record)",
                "~($investigator = $deploying_body)",
                "~($investigator = $commander)",
                "~($investigator = $source)",
                "~($investigator = $review)",
                "~($source = $deploying_body)",
                "~($review = $deploying_body)",
                "~($source = $commander)",
                "~($review = $commander)",
            ],
            heads: vec![
                "obliged($investigator, IndependentlyInvestigateProtectiveIncidentAndCommandResponsibility, $record)",
                "obliged($deploying_body, PreserveEvidenceAndEnableIndependentAccessNoOrdersDefence, $record)",
                "obliged(State, SecureFamilyParticipationPrivacyPublicFindingsAndEffectiveRemedy, $record)",
            ],
        },
    ]
}

fn fields(fact: &Fact) -> Vec<(&str, &str)> {
    let mut result = vec![
        (fact.kind, "Kind"),
        ("$subject", "Subject"),
        ("$case", "Case"),
        ("$version", "ConstitutionVersion"),
        ("$revision", "RecordRevision"),
        ("$epoch", "Epoch"),
        ("$jurisdiction", "Jurisdiction"),
        ("$scope", "Scope"),
        ("$window", "Window"),
        ("$basis", "PositiveEvidence"),
        ("$reader", "IndependentChallengeReader"),
        ("$alternate", "IndependentAlternateReader"),
        ("$correction", "CorrectionRoute"),
        ("$remedy", "RemedyRoute"),
        ("$revision", "CurrentSelectedRevision"),
        (
            "PositiveExternallyEstablishedCurrentFactNotAnInstitutionalAct",
            "CurrentDisposition",
        ),
        (
            "PurposeBoundMinimalPrivateEvidenceAccessibleChallengeCorrectionRetentionAndErasure",
            "Privacy",
        ),
        (
            "NoUnrelatedPersonScoreOrEligibilityUseAndNoAdverseInferenceFromAbsence",
            "RecordBoundary",
        ),
        (
            "ServeAndProtectFirstWhileIdentityOrRecordIsReconciled",
            "Continuity",
        ),
    ];
    result.extend(fact.fields.iter().copied());
    result
}

fn observe(actor: &str, value: &str, scope: &str) -> String {
    format!("observe({actor}, $record, {value}, PSFact{scope}Scope)")
}

fn premises(fact: &Fact) -> Vec<String> {
    let mut atoms = WRITERS
        .iter()
        .map(|(actor, role)| format!("authorized({actor}, {role}, $record)"))
        .collect::<Vec<_>>();
    atoms.extend([
        "authorized($reader, PSProtectedFactChallengeReaderAuthority, $record)".into(),
        "authorized($alternate, PSProtectedFactAlternateReaderAuthority, $record)".into(),
        "~contradict($record, PSProtectedFactAmbiguity)".into(),
    ]);
    for (i, actor) in ["$source", "$review", "$reader", "$alternate"]
        .iter()
        .enumerate()
    {
        for other in ["$source", "$review", "$reader", "$alternate"]
            .iter()
            .skip(i + 1)
        {
            atoms.push(format!("~({actor} = {other})"));
        }
        atoms.push(format!("~({actor} = $subject)"));
        for (interested, scope) in &fact.fields {
            if matches!(
                *scope,
                "Child"
                    | "HoldingActor"
                    | "DeployingBody"
                    | "ResponsibleCommand"
                    | "ImplicatedPower"
            ) {
                atoms.push(format!("~({actor} = {interested})"));
            }
        }
    }
    for (value, scope) in fields(fact) {
        for (actor, _) in WRITERS {
            atoms.push(observe(actor, value, scope));
        }
    }
    atoms.extend(fact.premises.iter().map(|s| (*s).into()));
    atoms
}

pub(super) fn rules() -> Vec<String> {
    let facts = facts();
    let mut result = Vec::new();
    for (_, role) in WRITERS {
        result.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, PSProtectedFactWriter)."));
    }
    for scope in facts
        .iter()
        .flat_map(fields)
        .map(|(_, scope)| scope)
        .collect::<BTreeSet<_>>()
    {
        result.push(records::rule(
            &[
                "related($a, $record, PSProtectedFactWriter)".into(),
                "related($b, $record, PSProtectedFactWriter)".into(),
                observe("$a", "$left", scope),
                observe("$b", "$right", scope),
                "~($left = $right)".into(),
            ],
            "contradict($record, PSProtectedFactAmbiguity)",
        ));
    }
    for fact in &facts {
        let atoms = premises(fact);
        // Relationship recognition must not feed the shared completion
        // predicate. Its existing readers include unrelated correction and
        // constitutional-authority paths. Keep these projections direct.
        if !matches!(fact.id, "family" | "parent") {
            result.push(records::rule(
                &atoms,
                &format!("complete($record, {}, $subject)", fact.kind),
            ));
        }
        for head in &fact.heads {
            result.push(records::rule(&atoms, head));
        }
        for duty in [
            "PreserveMinimalPrivateProtectedFactEvidenceAndAccessibleCorrection",
            "SecureCareAndRightsContinuityThroughoutFactDispute",
        ] {
            result.push(records::rule(
                &atoms,
                &format!("obliged($reader, {duty}, $record)"),
            ));
        }
    }
    for kind in [
        "IndividualArrest",
        "PretrialDetention",
        "Quarantine",
        "BorderHold",
        "PreExpulsionDetention",
        "InternationalTransfer",
        "CustodialSentence",
        "OtherPhysicalHoldingByOrForPublicPower",
        "UnrecordedOrUnlawfulPhysicalHolding",
    ] {
        result.push(format!(
            "public(Court) -> member({kind}, PSActualHoldingKind)."
        ));
    }
    for kind in ["Death", "SeriousInjury", "Torture", "EnforcedDisappearance"] {
        result.push(format!(
            "public(Court) -> member({kind}, PSProtectedIncidentKind)."
        ));
    }
    // Like the existing custody protections these are required conditions in
    // the constitutional model, not a delivery receipt or proof of action.
    // Physical holding grants no permission and produces no punitive status.
    result.push("all $requester: all $reader: all $record: challenge($requester, $reader, $record) & authorized($reader, PSProtectedFactChallengeReaderAuthority, $record) & ~($requester = $reader) -> obliged($reader, IndependentlyReviewProtectedFactAndSecureCareBeforeReconciliation, $record).".into());
    result.push("all $discloser: all $record: all $recipient: all $implicated: observe($discloser, $record, $recipient, PSDisclosureRecipientScope) & observe($discloser, $record, $implicated, PSImplicatedPowerScope) & authorized($recipient, PSIndependentDisclosureRecipientAuthority, $record) & ~($recipient = $discloser) & ~($recipient = $implicated) -> obliged($recipient, ReceiveConfidentialDisclosureWithoutPriorInternalChannel, $record).".into());
    // A credible current independent holding report triggers protection and
    // fact-finding even if the full record is absent, disputed or unacknowledged.
    result.push("all $witness: all $record: all $subject: authorized($witness, PSIndependentHoldingReportAuthority, $record) & observe($witness, $record, $subject, PSHoldingReportSubjectScope) & observe($witness, $record, CredibleCurrentPhysicalHoldingReport, PSHoldingReportFindingScope) & ~($witness = $subject) -> obliged(State, SecureImmediateHumaneCareVoiceAndIndependentHoldingStatusReview, $subject).".into());
    result.push("all $witness: all $record: all $subject: authorized($witness, PSIndependentProtectedFactReviewAuthority, $record) & observe($witness, $record, PSPositiveActualPublicHoldingFinding, PSFactKindScope) & observe($witness, $record, $subject, PSFactSubjectScope) & observe($witness, $record, PositiveCurrentPhysicalControlByOrForPublicPower, PSFactFindingScope) & observe($witness, $record, PositiveExternallyEstablishedCurrentFactNotAnInstitutionalAct, PSFactCurrentDispositionScope) & ~($witness = $subject) -> obliged(State, SecureImmediateHumaneCareVoiceAndIndependentHoldingStatusReview, $subject).".into());
    for duty in [
        "RespectUnconditionalConscientiousObjectionIncludingSpecificOperationWeaponOrOrder",
        "ProvideGenuinelyEquivalentNonPunitiveCivilianAlternativeWithoutSincerityTest",
        "PreserveEveryCivilRightEmploymentEducationAndFloorWithoutRepeatedObjectionPunishment",
        "ServeProtectAndProvideProcessBeforeRecordReconciliationNoMissingEntryInference",
        "PreserveHumanitarianCareCivilianProtectionAndAccessRegardlessOfConflictStatus",
    ] {
        result.push(format!(
            "all $person: person($person) -> obliged(State, {duty}, $person)."
        ));
    }
    result.push("all $person: person($person) -> prevents($person, ConscientiousObjectionDenialOrSincerityTest).".into());
    result.push("all $member: all $function: authorized($member, PSProtectiveServingMember, $function) -> obliged($member, RefuseManifestlyUnlawfulOrder, $function).".into());
    result.push("all $member: all $function: authorized($member, PSProtectiveServingMember, $function) -> obliged(State, ProtectRefusalOfManifestlyUnlawfulOrder, $member).".into());
    result
}

fn values(fact: &Fact, prefix: &str) -> Values {
    let mut values = records::variable_pattern()
        .find_iter(&premises(fact).join(" "))
        .map(|m| {
            (
                m.as_str().into(),
                format!(
                    "{prefix}{}",
                    m.as_str().trim_start_matches('$').replace('_', "")
                ),
            )
        })
        .collect::<Values>();
    values.insert(
        "$holding_kind".into(),
        "UnrecordedOrUnlawfulPhysicalHolding".into(),
    );
    values.insert("$claimed_order".into(), "NoOrderRecorded".into());
    values.insert("$incident".into(), "Death".into());
    values
}

fn fixture(fact: &Fact, values: &Values) -> String {
    premises(fact)
        .iter()
        .filter(|atom| atom.starts_with("authorized(") || atom.starts_with("observe("))
        .map(|atom| format!("{}.\n", records::ground(atom, values)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn checks(fact: &Fact, values: &Values, expected: bool) -> String {
    let mut result = if matches!(fact.id, "family" | "parent") {
        String::new()
    } else {
        query(
            &records::ground(
                &format!("complete($record, {}, $subject)", fact.kind),
                values,
            ),
            expected,
        )
    };
    for head in &fact.heads {
        result += &query(&records::ground(head, values), expected);
        if fact.id == "actual-holding" && *head == "person($subject)" {
            // Preserve the original actuality queries. A report of physical
            // control establishes the need for care, not its performance.
            for actuality in ["dwell($subject)", "expresses($subject)"] {
                result += &query(&records::ground(actuality, values), false);
            }
        }
    }
    result
}

pub(super) fn scenarios() -> Vec<Scenario> {
    let mut result = Vec::new();
    for fact in facts() {
        let values = values(&fact, "PSProtected");
        let mut given = fixture(&fact, &values);
        if matches!(fact.id, "family" | "parent") {
            given += &format!("at({}, RepublicJurisdiction).\n", values["$subject"]);
        }
        if fact.id == "parent" {
            given += &format!("at({}, RepublicJurisdiction).\n", values["$child"]);
        }
        let positive = checks(&fact, &values, true);
        result.push(Scenario::new(
            format!("protection/{}/positive", fact.id),
            given.clone(),
            positive.clone(),
        ));
        for (actor, _) in WRITERS {
            for (_, scope) in fields(&fact) {
                let missing = given
                    .lines()
                    .filter(|line| {
                        !(line.starts_with(&format!(
                            "observe({}, {},",
                            values[*actor], values["$record"]
                        )) && line.ends_with(&format!(", PSFact{scope}Scope).")))
                    })
                    .map(|line| format!("{line}\n"))
                    .collect::<String>();
                assert!(missing != given, "{} {actor} {scope}", fact.id);
                result.push(Scenario::new(
                    format!(
                        "protection/{}/without-{}-{scope}",
                        fact.id,
                        actor.trim_start_matches('$')
                    ),
                    missing,
                    checks(&fact, &values, false),
                ));
            }
        }
        let conflict = records::ground(
            "observe($source, $record, SupersededOrCorrectedFinding, PSFactCurrentDispositionScope).\n",
            &values,
        );
        result.push(Scenario::new(
            format!("protection/{}/positive-conflicting-correction", fact.id),
            given.clone(),
            positive.clone() + &conflict + &checks(&fact, &values, false),
        ));
        let noise = records::ground(
            "observe(UncredentialedWriter, $record, SupersededOrCorrectedFinding, PSFactCurrentDispositionScope).\n",
            &values,
        );
        result.push(Scenario::new(
            format!("protection/{}/uncredentialed-noise", fact.id),
            given.clone() + &noise,
            positive,
        ));
        let fused = given.replace(&values["$review"], &values["$source"]);
        result.push(Scenario::new(
            format!("protection/{}/self-review", fact.id),
            fused,
            checks(&fact, &values, false),
        ));
        if fact.id == "actual-holding" {
            let mut pins = String::new();
            for atom in ["person($subject)", "travel($subject)"] {
                pins += &query(&records::ground(atom, &values), true);
            }
            for atom in [
                "dwell($subject)",
                "expresses($subject)",
                "prisoner($subject)",
                "severe($subject)",
                "capture($subject)",
                "reward($subject)",
                "restrain($subject)",
                "free($subject)",
                "complete($claimed_order, PSIndividualArrestOrder, $subject)",
            ] {
                pins += &query(&records::ground(atom, &values), false);
            }
            result.push(Scenario::new(
                "protection/actual-holding/no-order-no-conviction-no-delivery-claim",
                given.clone(),
                pins,
            ));
            let shelter = records::ground(
                "receives($subject, IndependentlyReceivedShelter, ShelterProvider).\nauthorized(ShelterWitness, DeliveryWitness, $subject).\nobserve(ShelterWitness, IndependentlyReceivedShelter, $subject, ShelterScope).\n",
                &values,
            );
            result.push(Scenario::new(
                "protection/actual-holding/independent-shelter-receipt",
                given + &shelter,
                query(&records::ground("dwell($subject)", &values), true)
                    + &query(&records::ground("prisoner($subject)", &values), false)
                    + &query(&records::ground("expresses($subject)", &values), false),
            ));
        }
        if fact.id == "force-incident" {
            for incident in ["Death", "SeriousInjury", "Torture", "EnforcedDisappearance"] {
                let mut selected = values.clone();
                selected.insert("$incident".into(), incident.into());
                result.push(Scenario::new(
                    format!("protection/force-incident/{incident}"),
                    fixture(&fact, &selected),
                    checks(&fact, &selected, true),
                ));
            }
            let fused = fixture(&fact, &values)
                .replace(&values["$investigator"], &values["$deploying_body"]);
            result.push(Scenario::new(
                "protection/force-incident/deployer-cannot-investigate-itself",
                fused,
                checks(&fact, &values, false),
            ));
        }
    }
    result.push(Scenario::new(
        "protection/no-presumed-relationships",
        String::new(),
        query("family(Bela)", false)
            + &query("parent(Bela, Jala)", false)
            + &query("family(Lalo)", true)
            + &query("parent(Dev, Esa)", true),
    ));
    result.push(Scenario::new("protection/unrecorded-holding-report", "authorized(IndependentWitness, PSIndependentHoldingReportAuthority, UnknownHolding).\nobserve(IndependentWitness, UnknownHolding, UnrecordedPerson, PSHoldingReportSubjectScope).\nobserve(IndependentWitness, UnknownHolding, CredibleCurrentPhysicalHoldingReport, PSHoldingReportFindingScope).\n".into(),
        query("obliged(State, SecureImmediateHumaneCareVoiceAndIndependentHoldingStatusReview, UnrecordedPerson)", true)
        + &query("complete(UnknownHolding, PSPositiveActualPublicHoldingFinding, UnrecordedPerson)", false)
        + &query("prisoner(UnrecordedPerson)", false)));
    result.push(Scenario::new("protection/unconditional-objection", "at(ConscientiousObjector, RepublicJurisdiction).\nauthorized(ServingObjector, PSProtectiveServingMember, ProtectiveFunction).\n".into(),
        query("prevents(ConscientiousObjector, ConscientiousObjectionDenialOrSincerityTest)", true)
        + &query("obliged(ServingObjector, RefuseManifestlyUnlawfulOrder, ProtectiveFunction)", true)
        + &query("obliged(State, ProtectRefusalOfManifestlyUnlawfulOrder, ServingObjector)", true)
        + &query("prisoner(ConscientiousObjector)", false)));
    result.push(Scenario::new("protection/disclosure-without-implicated-internal-channel", "observe(Discloser, SurveillanceDisclosure, IndependentRecipient, PSDisclosureRecipientScope).\nobserve(Discloser, SurveillanceDisclosure, ImplicatedIntelligenceBody, PSImplicatedPowerScope).\nauthorized(IndependentRecipient, PSIndependentDisclosureRecipientAuthority, SurveillanceDisclosure).\n".into(),
        query("obliged(IndependentRecipient, ReceiveConfidentialDisclosureWithoutPriorInternalChannel, SurveillanceDisclosure)", true)
        + &query("complete(SurveillanceDisclosure, PSProtectedPublicInterestDisclosureFinding, Discloser)", false)
        + &query("false(ImplicatedIntelligenceBody)", false)));
    result
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};

    #[test]
    fn protected_facts_require_positive_evidence_not_lawful_holding_or_family_presumptions() {
        std::thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
            let context = Context::discover().unwrap();
            let cards = super::super::cards(&context).unwrap();
            let candidate = super::super::render(
                &context.read("book-1/source/constitution.nibli").unwrap(), &cards
            ).unwrap();
            let engine = PreparedPinEngine::new(&[LoadedSource::new("protected fact candidate", &candidate)]);
            for case in super::scenarios() {
                let out = engine.run_case(&[LoadedSource::new(&case.id, &case.facts)], &[LoadedSource::new(&case.id, &case.pins)], PinOptions::default(), true);
                assert_eq!(out.exit_code, 0, "{}: {out:?}", case.id);
            }
            let pins = ":expect-pins 6\n:refuse reasoning /Unstratifiable/\nall $x: person($x) & ~family($x) -> prisoner($x).\n:refuse reasoning /Unstratifiable/\nall $x: all $y: person($x) & ~parent($y, $x) -> prisoner($x).\n? family(Bela).\n# => FALSE\n? parent(Bela, Jala).\n# => FALSE\n? family(Lalo).\n# => TRUE\n? parent(Dev, Esa).\n# => TRUE\n";
            let out = engine.run_case(&[], &[LoadedSource::new("family firewall and existing facts", pins)], PinOptions::default(), true);
            assert_eq!(out.exit_code, 0, "{out:?}");
        }).unwrap().join().unwrap();
    }
}
