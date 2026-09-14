// SPDX-License-Identifier: MIT OR Apache-2.0

//! Divided appointments and finite externally certified collegial decisions.

use super::{Card, records};

pub(super) const CONTROL: &[(&str, &str)] = &[
    ("CurrentGovernment", "Direct"),
    ("CurrentGovernment", "DeFacto"),
    ("EitherChamber", "Direct"),
    ("EitherChamber", "DeFacto"),
    ("PartyCoalition", "Direct"),
    ("PartyCoalition", "DeFacto"),
    ("Industry", "Direct"),
    ("Industry", "DeFacto"),
    ("Profession", "Direct"),
    ("Profession", "DeFacto"),
    ("AdvocacyTendency", "Direct"),
    ("AdvocacyTendency", "DeFacto"),
    ("AppointingSource", "Direct"),
    ("AppointingSource", "DeFacto"),
];

fn appointment(id: &'static str, kind: &'static str, office: &'static str, animal: bool) -> Card {
    let mut card = Card::new(id, kind, "Class6IndependentAdvocateAppointment")
        .fields(&[
            ("$subject", "AppointedMember"),
            ("$office", "AppointedInstitution"),
            ("$seat", "AppointedSeat"),
            ("$selector_configuration", "DividedSelectorConfiguration"),
            ("$selector", "AppointingSource"),
            ("$qualification_authority", "QualificationAuthority"),
            ("$fallback_configuration", "FallbackConfiguration"),
            ("$term", "FiniteNonrenewableStaggeredTerm"),
            (
                "ECOpenAccessibleNominationsAndPublishedCriteria",
                "OpenNominations",
            ),
            (
                "ECMixedRelevantExpertiseLivedKnowledgeAndNoExclusiveProfession",
                "MixedComposition",
            ),
            (
                "ECCauseOnlyRemovalWithNoticeReasonsIndependentReviewAndContinuity",
                "CauseRemoval",
            ),
            (
                "ECProtectedPublicFundingDisclosureRecusalMinorityReasonsAndChallenge",
                "AppointmentAccountability",
            ),
            (
                "ECNoRegulatorScientistCourtAuditorOperatorOrOwnerFunctionFusion",
                "AdvocateSeparation",
            ),
            (
                "ECNoBudgetTaxPermitProgrammeProsecutionOrFinalVetoPower",
                "AdvocatePowerBoundary",
            ),
            (
                "ECPositiveCurrentAppointmentAndMandateFindingNotSelectionPermissionAlone",
                "ActualMandateFinding",
            ),
        ])
        .extra(&format!("$office = {office}"))
        .extra(&format!("$operator = {office}"))
        .default("$office", office)
        .default("$operator", office)
        .default("$qualification_authority", "FSBOD_24")
        .default("$selector", "FSBOD_01")
        .duty("ECPreserveDividedAppointmentIndependenceAndChallenge");
    for (source, mode) in CONTROL.iter().copied().chain(
        animal
            .then_some(("AnimalUseSector", "Direct"))
            .into_iter()
            .chain(animal.then_some(("AnimalUseSector", "DeFacto"))),
    ) {
        let scope = format!("Appointment{source}{mode}Control");
        for (actor, _) in records::ATTESTERS {
            card.extra.push(records::observe(
                actor,
                "$record",
                "ECIndependentlyExaminedNoMajorityControl",
                &scope,
            ));
        }
    }
    card
}

fn collegial(
    id: &'static str,
    kind: &'static str,
    appointment: &'static str,
    office: &'static str,
) -> Card {
    let mut card = Card::new(id, kind, "Class5BoundedCollegialResult")
        .fields(&[
            ("$office", "DecisionInstitution"),
            ("$roster", "ExactFiniteRoster"),
            ("$mandate_set", "CompleteCurrentMemberMandateSet"),
            ("$representative", "RepresentedMember"),
            ("$submissions", "UniqueCompleteSubmissionSet"),
            ("$decision_rule", "ApplicableDecisionRule"),
            ("$decision", "ExactCollegialDecision"),
            ("$certificate", "ResultCertificate"),
            ("$admin", "DecisionAdministrator"),
            ("$assurer", "CompletenessAssurer"),
            ("$service", "IndependentResultService"),
            (
                "ECNonemptyFiniteAuthenticRosterWithEveryCurrentMandateIndependentlyReviewed",
                "RosterFinding",
            ),
            (
                "ECUniqueCompleteAuthenticSubmissionsForThisExactDecision",
                "SubmissionFinding",
            ),
            (
                "ECApprovedNonTiedCurrentConflictFreeResultUnderDeclaredRule",
                "CollectiveResult",
            ),
            (
                "ECConflictedMembersExcludedAndQuorumPositivelyEstablished",
                "RecusalAndQuorum",
            ),
            (
                "ECNoCountProvesCompletenessNoGenericThresholdComputedHere",
                "FiniteBoundary",
            ),
            (
                "ECPublishedReasonsMinorityPositionsAndCorrection",
                "CollegialReasons",
            ),
            (
                "ECExactMandateAndTerritorialCoverageIndependentlyEstablished",
                "MandateCoverage",
            ),
        ])
        .depends(&[appointment])
        .separate_context(appointment)
        .join(appointment, "$subject", "$representative")
        .extra(&format!("$office = {office}"))
        .extra(&format!("$operator = {office}"))
        .default("$office", office)
        .default("$operator", office)
        .duty("ECPublishOnlyExactIndependentlyCertifiedCollegialResult");
    let services = [
        ("$admin", "ECDecisionAdministrationAuthority"),
        ("$assurer", "ECCompletenessAssuranceAuthority"),
        ("$service", "ECResultServiceAuthority"),
    ];
    let certified_fields = records::fields(&card);
    for (index, (actor, role)) in services.iter().enumerate() {
        card.extra
            .push(format!("authorized({actor}, {role}, $record)"));
        for (other, _) in &services[index + 1..] {
            card.extra.push(format!("~({actor} = {other})"));
        }
        for other in [
            "$source",
            "$review",
            "$operator",
            "$representative",
            "$reader",
            "$alternate",
            "$auditor",
        ] {
            card.extra.push(format!("~({actor} = {other})"));
        }
        for (value, scope) in &certified_fields {
            card.extra
                .push(records::observe(actor, "$record", value, scope));
        }
    }
    card
}

pub(super) fn cards() -> Vec<Card> {
    let mut reviewer = appointment(
        "substitute-reviewer-appointment",
        "ECSubstituteReviewerAppointment",
        "FSBOD_32",
        false,
    );
    reviewer.class = "Class6IndependentSubstituteReviewerAppointment";
    for (value, scope) in &mut reviewer.fields {
        if *scope == "AdvocateSeparation" {
            *value = "ECIndependentJudicialAssignmentNotAdvocacyRegulationScienceAuditOrOperation";
        }
        if *scope == "AdvocatePowerBoundary" {
            *value =
                "ECOnlyAssignedIndependentJudicialReviewNotPolicyVetoAdvocacyBudgetOrProgramme";
        }
    }
    vec![
        appointment(
            "guardian-appointment",
            "ECGuardianAppointment",
            "FSBOD_22",
            false,
        ),
        appointment(
            "animal-advocate-appointment",
            "ECAnimalAdvocateAppointment",
            "FSBOD_23",
            true,
        ),
        collegial(
            "guardian-result",
            "ECGuardianCollegialResult",
            "guardian-appointment",
            "FSBOD_22",
        ),
        collegial(
            "animal-advocate-result",
            "ECAnimalAdvocateCollegialResult",
            "animal-advocate-appointment",
            "FSBOD_23",
        ),
        appointment(
            "guardian-alternate-appointment",
            "ECGuardianAlternateAppointment",
            "FSBOD_31",
            false,
        ),
        appointment(
            "animal-alternate-appointment",
            "ECAnimalAlternateAppointment",
            "FSBOD_33",
            true,
        ),
        reviewer,
        collegial(
            "guardian-alternate-result",
            "ECGuardianAlternateCollegialResult",
            "guardian-alternate-appointment",
            "FSBOD_31",
        ),
        collegial(
            "animal-alternate-result",
            "ECAnimalAlternateCollegialResult",
            "animal-alternate-appointment",
            "FSBOD_33",
        ),
    ]
}
