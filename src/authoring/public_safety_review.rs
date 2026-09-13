// SPDX-License-Identifier: MIT OR Apache-2.0

//! Independent findings, exact withdrawals, reachable readers and duties.
//! A missing original record cannot prevent a complaint or an abuse finding.

use super::{
    contracts::{self, Card},
    records,
};
use std::collections::BTreeMap;

pub(super) fn is_finding(id: &str) -> bool {
    matches!(id, "reviewed-defect" | "cessation" | "nonresponse")
}

pub(super) fn extend(cards: &mut Vec<Card>) {
    cards.push(Card::new(
        "reviewed-defect",
        "PSIndependentProtectiveDefectFinding",
        &[
            ("$target", "AffectedRecordOrUnrecordedAct"),
            ("$target_revision", "AffectedRevision"),
            ("$target_kind", "AffectedKind"),
            ("$target_source", "AffectedSource"),
            ("$target_window", "AffectedWindow"),
            ("$defect", "DefectKind"),
            (
                "PositiveIndependentEvidenceOfExactDefectNotMissingPersonInference",
                "Finding",
            ),
            (
                "OriginalActorRecordOrAcknowledgementNotConditionOfFindingOrRemedy",
                "UnrecordedAbuse",
            ),
            (
                "NoCoreAbuseImmunityAmnestyReposePardonOrSecretPrivilege",
                "Accountability",
            ),
            (
                "StopAffectedReliancePreserveEvidenceCareChallengeAndIndependentJudicialRemedy",
                "InterimDuty",
            ),
        ],
    ));
    cards
        .last_mut()
        .unwrap()
        .extra
        .push("member($defect, PSProtectiveDefectKind)".into());
    let cessation = cards
        .iter_mut()
        .find(|card| card.id == "cessation")
        .unwrap();
    cessation.fields.extend([
        ("$target_kind", "AffectedKind"),
        ("$target_source", "AffectedSource"),
        ("$target_window", "AffectedWindow"),
    ]);
    cards.push(Card::new(
        "nonresponse",
        "PSIndependentlyEvidencedProtectiveNonresponse",
        &[
            ("$target", "UnansweredRequestTarget"),
            ("$requester", "RequestInitiator"),
            ("$request", "ActualRequest"),
            ("$request_window", "ActualRequestWindow"),
            ("$deadline_finding", "ElapsedDeadlineFinding"),
            (
                "PositiveIndependentlyEstablishedDeadlinePassedWithoutRequiredAction",
                "Nonresponse",
            ),
            (
                "NoSilenceApprovalNoNewPowerNoIndefiniteHoldAndNoCareSuspension",
                "Limits",
            ),
            (
                "PredeclaredUninvolvedAlternateActualActionDutyAndCourtEscalation",
                "Alternate",
            ),
        ],
    ));
    cards.last_mut().unwrap().extra.extend([
        "challenge($requester, $reader, $target)".into(),
        "authorized($reader, PSChallengeReaderAuthority, $target)".into(),
        "observe($requester, $target, $case, PSRequestedCaseScope)".into(),
        "observe($requester, $target, $request, PSActualRequestScope)".into(),
        "~($requester = $reader)".into(),
        "~($requester = $alternate)".into(),
    ]);
    for card in cards
        .iter_mut()
        .filter(|card| matches!(card.id, "reviewed-defect" | "cessation"))
    {
        card.fields.extend([
            ("$target_holder", "AffectedHolder"),
            ("$target_operator", "AffectedExecutor"),
        ]);
        for (actor, _) in records::ATTESTERS.iter().chain(records::READERS) {
            for interested in ["$target_source", "$target_holder", "$target_operator"] {
                card.extra.push(format!("~({actor} = {interested})"));
            }
        }
    }
}

fn defect_kinds() -> Vec<String> {
    let mut kinds = contracts::CORE_BARS
        .iter()
        .map(|name| format!("PSProhibited{name}"))
        .collect::<Vec<_>>();
    kinds.extend(
        [
            "PSUnlawfulProtectiveMandate",
            "PSFusedProtectiveFunctions",
            "PSServingArmedOrIntelligenceIncompatibleOffice",
            "PSMilitaryCivilianJurisdiction",
            "PSCoerciveUnarmedAssistance",
            "PSPrivateDelegatedCoercion",
            "PSSecretLawCourtOrCourtInaccessibleEngagementRule",
            "PSClassifiedBudgetBeyondNarrowFullyAuditableAnnex",
            "PSOrderMaintenanceWithoutIndividualHarmGround",
            "PSImmigrationFloorCareLearningOrCollectiveStatusPolicingGate",
            "PSEmergencyRationingOutsidePhysicalScarcityContract",
            "PSCompulsoryNamedWorkerService",
            "PSUnilateralEmergencyPriceDecree",
            "PSMissingIndividualGroundOrAutomaticJudicialReview",
            "PSUnnecessaryOrDisproportionateForce",
            "PSUnlawfulLethalForce",
            "PSDeployingBodySelfInvestigation",
            "PSImmunityForCoreAbuse",
            "PSUnrecordedCoercion",
            "PSEvidenceDestruction",
            "PSEvidenceFalsification",
            "PSIntimidatedWitness",
            "PSConscientiousObjectionPenaltyOrSincerityTest",
            "PSRetaliationForRefusalOfUnlawfulOrder",
            "PSRetaliationForProtectedDisclosure",
            "PSCompulsoryPriorImplicatedInternalChannel",
            "PSChildImmigrationDetention",
            "PSIndefiniteOrUnnecessaryAdultImmigrationDetention",
            "PSUnlawfulEnrolmentForEnforcement",
            "PSUnlawfulCollectionForEnforcement",
            "PSUnlawfulTransmissionForEnforcement",
            "PSUnrecordedPersonAdverseInference",
            "PSExclusionOfArrivalsFromScarcityPopulation",
            "PSBulkOrSuspicionlessCollection",
            "PSPurchasedUnlawfullyCollectableData",
            "PSSoleOrDecisiveSecretEvidence",
            "PSDisproportionateOrNeedlesslyIntrusiveSurveillance",
            "PSUnsuspectedPersonDataRetention",
            "PSPermanentlyUndisclosableSurveillanceAuthorisation",
            "PSUndisclosableMaterialUsedForConsequence",
            "PSNonFunctionSpecificEmploymentVetting",
            "PSEmergencyDerogationOrDecree",
            "PSElectionDelayOrMandateExtension",
            "PSUnratifiedAlternateAfterFirstOpportunity",
            "PSUnratifiedSubstituteAfterFirstOpportunity",
            "PSCredibleArmsMisuse",
            "PSChildRecruitmentOrHostilityUse",
            "PSCivilianLifeCriticalCyberAttack",
            "PSUnratifiedForceAbroad",
            "PSSecretWar",
            "PSTreatyActualEffectCorridorViolation",
            "PSMissingRequiredRegionalConsent",
            "PSProvisionalRatificationPreemption",
            "PSExternalRightsEvasion",
            "PSLawfulExitMilitaryCoercion",
            "PSWeaponisedExitServiceTradeOrBorderLever",
            "PSMissingActualCollectiveConsent",
            "PSWithheldHumaneHoldingConditionsOrIndependentAccess",
        ]
        .map(str::to_owned),
    );
    for assessment in contracts::ASSESSMENTS {
        for effect in [
            "CanonicalPersonRecord",
            "Standing",
            "Personhood",
            "Floor",
            "Franchise",
            "Candidacy",
            "Liberty",
            "Remedy",
            "Allocation",
        ] {
            kinds.push(format!("PS{assessment}AssessmentUsedFor{effect}"));
        }
    }
    for key in contracts::PRIORITY_KEYS {
        kinds.push(format!("PS{key}ScarcityPriority"));
    }
    kinds
}

fn target_join() -> Vec<String> {
    let mut atoms = vec!["authorized($target_source, PSSourceAuthority, $target)".into()];
    for (variable, scope) in [
        ("$subject", "CoreSubject"),
        ("$case", "CoreCase"),
        ("$version", "ConstitutionVersion"),
        ("$target_revision", "RecordRevision"),
        ("$epoch", "CoreEpoch"),
        ("$jurisdiction", "CoreJurisdiction"),
        ("$scope", "CoreLegalScope"),
        ("$target_kind", "RecordKind"),
        ("$target_window", "CoreWindow"),
        ("$target_holder", "CoreHolder"),
        ("$target_operator", "CoreExecutor"),
    ] {
        atoms.push(records::observe(
            "$target_source",
            "$target",
            variable,
            scope,
        ));
    }
    atoms
}

pub(super) fn rules(cards: &[Card]) -> Vec<String> {
    let mut result = Vec::new();
    for kind in defect_kinds() {
        result.push(format!(
            "public(Court) -> member({kind}, PSProtectiveDefectKind)."
        ));
        result.push(format!(
            "all $person: person($person) -> prevents($person, {kind})."
        ));
    }
    for card in cards.iter().filter(|card| !is_finding(card.id)) {
        result.push(format!(
            "public(Court) -> member({}, PSWithdrawableProtectiveKind).",
            card.kind
        ));
    }
    for id in ["reviewed-defect", "cessation"] {
        let card = cards.iter().find(|card| card.id == id).unwrap();
        let atoms = records::premises(cards, card);
        let mut withdrawal = atoms.clone();
        withdrawal.push("member($target_kind, PSWithdrawableProtectiveKind)".into());
        withdrawal.extend(target_join());
        result.push(records::rule(
            &withdrawal,
            "contradict($target, PSEffectReliance)",
        ));
        let mut reading = atoms;
        if id == "reviewed-defect" {
            result.push(records::rule(
                &reading,
                "err($record, PSReviewedProtectiveDefect)",
            ));
            reading.push("err($record, PSReviewedProtectiveDefect)".into());
        } else {
            // An ordinary recorded end is not itself a violation. Its reader
            // consumes the exact end record, not a fabricated error marker.
            reading.push(format!("complete($record, {}, $subject)", card.kind));
        }
        for duty in [
            "ReadExactProtectiveFindingAndStopAffectedReliance",
            "PreserveEvidenceSecureCareAndObtainActualIndependentCourtRemedy",
            "ReviewConnectedCasesAndBoundedStructuralNonRepetitionWithoutThirdPartyRightsLoss",
        ] {
            result.push(records::rule(
                &reading,
                &format!("obliged($reader, {duty}, $record)"),
            ));
        }
    }
    let card = cards.iter().find(|card| card.id == "nonresponse").unwrap();
    let atoms = records::premises(cards, card);
    result.push(records::rule(
        &atoms,
        "err($record, PSProtectiveNonresponse)",
    ));
    let mut reading = atoms;
    reading.push("err($record, PSProtectiveNonresponse)".into());
    result.push(records::rule(&reading, "obliged($alternate, ReviewProtectiveRequestSecureCareAndEscalateActualCourtRemedy, $target)"));
    result.push("all $requester: all $reader: all $target: challenge($requester, $reader, $target) & authorized($reader, PSChallengeReaderAuthority, $target) & ~($requester = $reader) -> obliged($reader, ReceiveAndIndependentlyReviewProtectiveRequestWithoutOfficialRecordPrecondition, $target).".into());
    result
}

pub(super) fn default_values(card: &Card, values: &mut BTreeMap<String, String>) {
    if card.id == "reviewed-defect" {
        values.insert("$defect".into(), "PSUnrecordedCoercion".into());
    }
}

pub(super) fn scenarios(cards: &[Card]) -> Vec<super::cases::Scenario> {
    use super::cases::{Scenario, query};
    let mut result = Vec::new();
    let target = cards.iter().find(|card| card.id == "arrest").unwrap();
    let target_values = records::values(cards, target, "PSTarget");
    let target_facts = records::fixture(cards, target, &target_values);
    let other_values = records::values(cards, target, "PSUnaffectedTarget");
    let other_facts = records::fixture(cards, target, &other_values);
    let complete = |values: &BTreeMap<String, String>, expected| {
        query(
            &records::ground(
                "complete($record, PSIndividualArrestOrder, $subject)",
                values,
            ),
            expected,
        )
    };
    for id in ["reviewed-defect", "cessation"] {
        let card = cards.iter().find(|card| card.id == id).unwrap();
        let mut values = records::values(cards, card, "PSReview");
        for variable in [
            "$subject",
            "$case",
            "$version",
            "$epoch",
            "$jurisdiction",
            "$scope",
        ] {
            values.insert(variable.into(), target_values[variable].clone());
        }
        for (from, to) in [
            ("$record", "$target"),
            ("$revision", "$target_revision"),
            ("$kind", "$target_kind"),
            ("$source", "$target_source"),
            ("$window", "$target_window"),
            ("$holder", "$target_holder"),
            ("$operator", "$target_operator"),
        ] {
            values.insert(to.into(), target_values[from].clone());
        }
        let finding = records::fixture(cards, card, &values);
        let mut steps = complete(&target_values, true) + &complete(&other_values, true) + &finding;
        steps += &complete(&target_values, false);
        steps += &complete(&other_values, true);
        steps += &query(
            &records::ground("contradict($target, PSEffectReliance)", &values),
            true,
        );
        steps += &query(
            &records::ground(
                "obliged($reader, ReadExactProtectiveFindingAndStopAffectedReliance, $record)",
                &values,
            ),
            true,
        );
        steps += &query(&records::ground("free($subject)", &values), false);
        if id == "cessation" {
            steps += &query(
                &records::ground("err($record, PSRecordedProtectiveCessation)", &values),
                false,
            );
        }
        steps += &query(&records::ground("person($subject)", &values), true);
        steps += &query(
            &records::ground("owe(State, Eats, $subject)", &values),
            true,
        );
        result.push(Scenario::new(
            format!("review/{id}/exact-withdrawal-and-reader"),
            target_facts.clone() + &other_facts,
            steps.clone(),
        ));
        let without_separate_entry = (target_facts.clone() + &other_facts)
            .lines()
            .filter(|line| !line.starts_with("at("))
            .map(|line| format!("{line}\n"))
            .collect::<String>();
        result.push(Scenario::new(
            format!("review/{id}/standing-without-separate-entry"),
            without_separate_entry,
            steps,
        ));
        for key in [
            "$target",
            "$target_revision",
            "$case",
            "$version",
            "$subject",
            "$target_window",
            "$target_kind",
            "$target_source",
            "$target_holder",
            "$target_operator",
        ] {
            let mut mismatch = values.clone();
            mismatch.insert(key.into(), "UnrelatedFindingIdentity".into());
            result.push(Scenario::new(
                format!("review/{id}/wrong-{}", key.trim_start_matches('$')),
                target_facts.clone() + &records::fixture(cards, card, &mismatch),
                complete(&target_values, true),
            ));
        }
        let missing = finding
            .lines()
            .filter(|line| {
                !line.starts_with(&format!(
                    "observe({}, {},",
                    values["$review"], values["$record"]
                ))
            })
            .map(|line| format!("{line}\n"))
            .collect::<String>();
        assert!(missing != finding);
        result.push(Scenario::new(
            format!("review/{id}/missing-independent-review"),
            target_facts.clone() + &missing,
            complete(&target_values, true),
        ));
        for interested in ["$target_source", "$target_holder", "$target_operator"] {
            let compromised = finding.replace(&values["$review"], &values[interested]);
            result.push(Scenario::new(
                format!(
                    "review/{id}/interested-{}-reviewer",
                    interested.trim_start_matches('$')
                ),
                target_facts.clone() + &compromised,
                complete(&target_values, true),
            ));
        }
        if id == "reviewed-defect" {
            for defect in defect_kinds() {
                let mut selected = values.clone();
                selected.insert("$defect".into(), defect.clone());
                result.push(Scenario::new(
                    format!("review/defect/{defect}"),
                    target_facts.clone() + &records::fixture(cards, card, &selected),
                    complete(&target_values, false)
                        + &query(
                            &records::ground("err($record, PSReviewedProtectiveDefect)", &selected),
                            true,
                        ),
                ));
            }
            result.push(Scenario::new("review/unrecorded-abuse-is-still-a-finding", finding,
                query(&records::ground("err($record, PSReviewedProtectiveDefect)", &values), true)
                + &query(&records::ground("obliged($reader, PreserveEvidenceSecureCareAndObtainActualIndependentCourtRemedy, $record)", &values), true)
                + &query(&records::ground("contradict($target, PSEffectReliance)", &values), false)));
        }
    }
    let card = cards.iter().find(|card| card.id == "nonresponse").unwrap();
    let values = records::values(cards, card, "PSNonresponse");
    let request = records::ground(
        "challenge($requester, $reader, $target).\nauthorized($reader, PSChallengeReaderAuthority, $target).\nobserve($requester, $target, $case, PSRequestedCaseScope).\nobserve($requester, $target, $request, PSActualRequestScope).\n",
        &values,
    );
    let primary = records::ground(
        "obliged($reader, ReceiveAndIndependentlyReviewProtectiveRequestWithoutOfficialRecordPrecondition, $target)",
        &values,
    );
    let alternate = records::ground(
        "obliged($alternate, ReviewProtectiveRequestSecureCareAndEscalateActualCourtRemedy, $target)",
        &values,
    );
    result.push(Scenario::new(
        "review/open-request-and-positive-nonresponse",
        request,
        query(&primary, true)
            + &query(&alternate, false)
            + &records::fixture(cards, card, &values)
            + &query(&alternate, true)
            + &query(&records::ground("restrain($requester)", &values), false),
    ));
    for (child_id, parent_id, defect) in [
        ("arrest", "policing", "PSFusedProtectiveFunctions"),
        (
            "surveillance",
            "security-intelligence",
            "PSServingArmedOrIntelligenceIncompatibleOffice",
        ),
        (
            "requisition",
            "declaration",
            "PSEmergencyDerogationOrDecree",
        ),
        (
            "force-abroad",
            "defence-structure",
            "PSPrivateDelegatedCoercion",
        ),
    ] {
        let child = cards.iter().find(|card| card.id == child_id).unwrap();
        let parent = cards.iter().find(|card| card.id == parent_id).unwrap();
        let child_values = records::values(cards, child, "PSMandateConflict");
        let parent_values = records::values(cards, parent, "UnusedTemplate")
            .keys()
            .map(|key| {
                (
                    key.clone(),
                    child_values[&records::dependency_variable(parent_id, key)].clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let finding = cards
            .iter()
            .find(|card| card.id == "reviewed-defect")
            .unwrap();
        let mut finding_values = records::values(cards, finding, "PSMandateDefect");
        for key in [
            "$subject",
            "$case",
            "$version",
            "$epoch",
            "$jurisdiction",
            "$scope",
        ] {
            finding_values.insert(key.into(), parent_values[key].clone());
        }
        for (from, to) in [
            ("$record", "$target"),
            ("$revision", "$target_revision"),
            ("$kind", "$target_kind"),
            ("$source", "$target_source"),
            ("$window", "$target_window"),
            ("$holder", "$target_holder"),
            ("$operator", "$target_operator"),
        ] {
            finding_values.insert(to.into(), parent_values[from].clone());
        }
        finding_values.insert("$defect".into(), defect.into());
        let completion = |card: &Card, values: &BTreeMap<String, String>, expected| {
            query(
                &records::ground(
                    &format!("complete($record, {}, $subject)", card.kind),
                    values,
                ),
                expected,
            )
        };
        let steps = completion(parent, &parent_values, true)
            + &completion(child, &child_values, true)
            + &records::fixture(cards, finding, &finding_values)
            + &completion(parent, &parent_values, false)
            + &completion(child, &child_values, false)
            + &query(&records::ground("person($subject)", &child_values), true)
            + &query(
                &records::ground("owe(State, Eats, $subject)", &child_values),
                true,
            )
            + &query(&records::ground("prisoner($subject)", &child_values), false);
        result.push(Scenario::new(
            format!("review/{parent_id}-conflict-withdraws-{child_id}"),
            records::fixture(cards, child, &child_values),
            steps,
        ));
    }
    result
}
