// SPDX-License-Identifier: MIT OR Apache-2.0

//! Independently evidenced failures and ends, not clocks or erased history.

use super::{Card, records};

pub(super) const DEFECTS: &[&str] = &[
    "ECWithheldDecisiveEvidence",
    "ECUnauthorizedRecordWriter",
    "ECFusedInterestedRecordReview",
    "ECConflictingDecisiveRecordIdentity",
    "ECWrongSubjectPlaceVersionOrJurisdiction",
    "ECUnsupportedCurrentnessOrSilentHarmfulRenewal",
    "ECFalseOrUnreviewedScientificFinding",
    "ECConcealedUncertaintyOrMaterialAlternative",
    "ECRecordFalsificationOrEvidenceDestruction",
    "ECUnlawfulPrivateEvidenceDisclosure",
    "ECBlockedIndependentAccessChallengeOrRemedy",
    "ECRetaliationForEnvironmentalOrAnimalClaim",
    "ECAnimalRecordUsedAsConsequentialHumanIdentity",
    "ECAnimalRecordUsedAsHumanWorth",
    "ECAnimalRecordUsedAsHumanPolitics",
    "ECAnimalRecordUsedAsHumanFamilyStatus",
    "ECAnimalRecordUsedAsHumanMigrationStatus",
    "ECAnimalRecordUsedAsHumanRisk",
    "ECAnimalRecordUsedAsHumanEntitlement",
    "ECRecordFailureUsedToWithholdUrgentHumanOrAnimalCare",
];

fn affected(card: Card) -> Card {
    let mut card = card.fields(&[
        ("$target", "AffectedRecord"),
        ("$target_revision", "AffectedRecordRevision"),
        ("$target_kind", "AffectedRecordKind"),
        ("$target_version", "AffectedConstitutionVersion"),
        ("$target_source", "AffectedRecordWriter"),
        ("$target_review", "AffectedRecordReviewer"),
        ("$target_operator", "AffectedResponsibleActor"),
        ("$target_window", "AffectedRecordWindow"),
        ("$target_end", "AffectedRecordEnd"),
        ("$target_evidence", "AffectedEvidenceRecord"),
        ("$target_evidence_version", "AffectedEvidenceVersion"),
        ("ECIndependentPositiveEvidenceNotMereAbsenceOrAnUnreviewedAllegation", "RecordFindingBasis"),
        ("ECOriginalWriterOrControllerAcknowledgementNotRequiredForFindingOrUrgentRemedy", "NoOriginalActorVeto"),
        ("ECOnlyExactAffectedRelianceWithdrawnNoSafetyGuiltExecutionOrRepairedOutcomeInference", "WithdrawalPolarity"),
        ("ECPreserveLawfulHistoryPrivateEvidenceReasonsChallengeAndIndependentRemedy", "RecordHistoryProtection"),
        ("ECNoLostHumanStandingFloorBallotLibertyCollectiveRightOrAnimalCare", "RecordFailureRightsWall"),
        ("ECNoCarryOrAutomaticHarmfulAuthorityFromCorrectionOrRenamedWindow", "RecordCorrectionLimit"),
    ])
    .withdrawal()
    .extra("~($record = $target)")
    .default("$target_kind", "ECOrdinaryLowHarmAnimalUse")
    .effect("obliged($reader, ECReadExactFindingStopAffectedRelianceAndObtainIndependentRemedy, $record)")
    .effect("obliged($alternate, ECPreserveUrgentHumanAndAnimalCareAndIndependentChallengeDuringRecordFailure, $record)")
    .effect("obliged($auditor, ECAuditAffectedRelianceLawfulHistoryAndNonRepetitionWithoutAssumingExecution, $record)");
    for actor in records::ATTESTERS
        .iter()
        .chain(records::READERS)
        .map(|(actor, _)| *actor)
        .chain(["$operator"])
    {
        for interested in ["$target_source", "$target_review", "$target_operator"] {
            card = card.extra(&format!("~({actor} = {interested})"));
        }
    }
    card
}

pub(super) fn cards() -> Vec<Card> {
    vec![
        affected(Card::new("ecological-record-defect", "ECIndependentEcologicalAnimalRecordDefect", "Class7IndependentRecordDefect"))
            .fields(&[("$defect", "EcologicalRecordDefectKind")])
            .extra("member($defect, ECEcologicalAnimalRecordDefectKind)")
            .default("$defect", "ECWithheldDecisiveEvidence")
            .effect("err($record, ECReviewedEcologicalAnimalRecordDefect)"),
        affected(Card::new("ecological-record-end", "ECIndependentEcologicalAnimalEndFinding", "Class7SourceBoundRecordEnd"))
            .historical()
            .fields(&[
                ("$elapsed_evidence", "IndependentElapsedEndEvidence"),
                ("ECIndependentlyEstablishedOwnSourceBoundEndReachedNotCustodyClockOrAssumedTime", "RecordEndFinding"),
                ("ECOrdinaryEndIsNotItselfMisconductOrProofTheActivityActuallyStopped", "RecordEndPolarity"),
            ]),
        affected(Card::new("ecological-record-correction", "ECIndependentEcologicalAnimalCorrection", "Class7ExactCorrectionWithoutRenewal"))
            .fields(&[
                ("$replacement", "ProposedReplacementRecord"),
                ("$replacement_revision", "ProposedReplacementRevision"),
                ("$comparison", "IndependentCorrectionComparisonEvidence"),
                ("ECReasonedExactCorrectionWithIndependentAccessAndPublishedRelianceConsequences", "CorrectionFinding"),
                ("ECReplacementHasNoForceUntilItsOwnCompleteFreshSubstantiveContractDerives", "ReplacementNonEnactment"),
                ("ECNewRecordIdentityKeepsSupersededEvidenceAndFinalHistoryDistinguishable", "CorrectionIdentity"),
            ])
            .extra("~($target = $replacement)")
            .extra("~($record = $replacement)")
            .effect("prevents($replacement, ECCorrectionAsAutomaticRenewedAuthority)"),
        Card::new("ecological-record-nonresponse", "ECIndependentEcologicalAnimalNonresponse", "Class7PositiveNonresponseAndAlternateDuty")
            .fields(&[
                ("$target", "UnansweredRecordOrUnrecordedAct"),
                ("$requester", "IndependentRequestInitiator"),
                ("$request", "ActualUnansweredRequest"),
                ("$request_window", "ActualRequestWindow"),
                ("$elapsed_evidence", "IndependentRequestDeadlineEvidence"),
                ("ECPositiveIndependentFindingDeadlinePassedWithoutRequiredResponse", "NonresponseFinding"),
                ("ECPredeclaredUninvolvedAlternateAccessActionAndCourtEscalation", "NonresponseAlternateRoute"),
                ("ECNoSilenceApprovalAutomaticStayNewAuthorityIndefiniteHoldOrCareSuspension", "NonresponseLimits"),
            ])
            .extra("challenge($requester, $target, $request)")
            .extra("authorized($reader, ECChallengeReaderAuthority, $target)")
            .extra("observe($requester, $request, $target, ECAffectedRecordScope)")
            .extra("observe($requester, $request, $case, ECCaseScope)")
            .extra("~($requester = $reader)")
            .extra("~($requester = $alternate)")
            .effect("err($record, ECEcologicalAnimalRecordNonresponse)")
            .effect("obliged($alternate, ECIndependentlyActOnUnansweredRequestSecureCareAndEscalateCourtRemedy, $target)"),
    ]
}

pub(super) fn target_join() -> Vec<String> {
    let mut atoms = vec!["authorized($target_source, ECSourceAuthority, $target)".into()];
    for (value, scope) in [
        ("$subject", "Subject"),
        ("$case", "Case"),
        ("$target_revision", "Revision"),
        ("$target_kind", "Kind"),
        ("$target_version", "ConstitutionVersion"),
        ("$place", "Place"),
        ("$territory", "Territory"),
        ("$population", "AffectedPopulation"),
        ("$jurisdiction", "Jurisdiction"),
        ("$scope", "LegalScope"),
        ("$target_operator", "ResponsibleActor"),
        ("$target_window", "Window"),
        ("$target_end", "End"),
        ("$target_evidence", "EvidenceRecord"),
        ("$target_evidence_version", "EvidenceVersion"),
    ] {
        atoms.push(records::observe("$target_source", "$target", value, scope));
    }
    atoms
}

pub(super) fn rules(cards: &[Card]) -> Vec<String> {
    let mut rules = Vec::new();
    for defect in DEFECTS {
        rules.push(format!("all $writer: all $record: observe($writer, $record, {defect}, ECEcologicalRecordDefectKindScope) -> member({defect}, ECEcologicalAnimalRecordDefectKind)."));
    }
    for card in cards
        .iter()
        .filter(|card| !card.withdrawal && !card.historical)
    {
        rules.push(format!("all $writer: all $record: observe($writer, $record, {}, ECAffectedRecordKindScope) -> member({}, ECWithdrawableEcologicalAnimalKind).", card.kind, card.kind));
    }
    for id in [
        "ecological-record-defect",
        "ecological-record-end",
        "ecological-record-correction",
    ] {
        let Some(card) = cards.iter().find(|card| card.id == id) else {
            continue;
        };
        // Read the actual raw finding, not its generic completion head: the
        // withdrawal must be below consumers that negatively test reliance.
        // Finding/remedy duties themselves do not await the original record.
        let mut atoms = records::premises(cards, card);
        atoms.push("member($target_kind, ECWithdrawableEcologicalAnimalKind)".into());
        atoms.extend(target_join());
        rules.push(records::rule(
            &atoms,
            "contradict($target, $target_revision, ECRecordReliance)",
        ));
        if id == "ecological-record-end" {
            rules.push(records::rule(
                &atoms,
                "contradict($target_window, ECSourceWindowCurrentReliance)",
            ));
        }
    }
    for (kind, duty) in [
        (
            "ECUrgentPlausibleAnimalProtectionRequest",
            "ECProvidePromptSafeAnimalProtectionAndNecessaryCareBeforeRecordReconciliation",
        ),
        (
            "ECUrgentEnvironmentalProtectionRequest",
            "ECProvidePromptNoFaultEnvironmentalProtectionAndHumanAnimalContinuityBeforeRecordReconciliation",
        ),
    ] {
        rules.push(format!("all $requester: all $subject: all $request: challenge($requester, $subject, $request) & observe($requester, $request, {kind}, ECRequestKindScope) -> obliged(State, {duty}, $request)."));
    }
    rules.push("all $requester: all $reader: all $request: challenge($requester, $reader, $request) & authorized($reader, ECChallengeReaderAuthority, $request) & ~($requester = $reader) -> obliged($reader, ECReceiveIndependentRecordChallengeWithoutOriginalRecordOrActorApproval, $request).".into());
    rules
}
