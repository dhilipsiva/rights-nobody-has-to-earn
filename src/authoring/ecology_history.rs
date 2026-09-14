// SPDX-License-Identifier: MIT OR Apache-2.0

//! Authenticated past disposition, separate from a court's current legal force.
//! The caller supplies and retains the full at-decision proof. This is neither
//! persisted inference nor a claim that Nibli authenticates or archives it.

use super::{Card, records};

pub(super) fn extend(cards: &mut Vec<Card>) {
    let mut history = Card::new("guardian-final-history", "ECHistoricalGuardianFinalDisposition", "Class7ExactHistoricalJudicialReplayEvidence")
        .historical()
        .fields(&[
            ("$project", "ChallengedActivity"),
            ("$authorization_record", "ChallengedAuthorizationRecord"),
            ("$authorization_version", "AuthorizationVersion"),
            ("$ground", "StayGround"),
            ("$outcome", "FinalMeritsOutcome"),
            ("$merits_record", "OriginalFinalDispositionRecord"),
            ("$merits_revision", "OriginalFinalDispositionRevision"),
            ("$merits_kind", "OriginalFinalDispositionKind"),
            ("$merits_source", "OriginalFinalDispositionWriter"),
            ("$merits_review", "OriginalFinalDispositionReviewer"),
            ("$merits_operator", "OriginalFinalDispositionCourt"),
            ("$merits_version", "OriginalFinalDispositionConstitutionVersion"),
            ("$merits_window", "OriginalFinalDispositionWindow"),
            ("$merits_start", "OriginalFinalDispositionStart"),
            ("$merits_end", "OriginalFinalDispositionEnd"),
            ("$merits_evaluation", "OriginalFinalDispositionEvaluation"),
            ("$court_record", "OriginalActualJudicialMandateRecord"),
            ("$court_source", "OriginalActualJudicialMandateSource"),
            ("$court_evidence", "OriginalActualJudicialMandateEvidence"),
            ("$court_review", "OriginalActualJudicialMandateReview"),
            ("$court_proof", "AuthenticatedFullAtDecisionJudicialProof"),
            ("$published_disposition", "AuthenticatedPublishedOriginalDisposition"),
            ("ECIndependentRetrospectiveAuthenticationOfFullOriginalCourtMandateProcessAndFinalDecisionAtItsExactSourceVersion", "HistoricalJudicialProofFinding"),
            ("ECAtDecisionOrdinaryOrSeparatelyAssignedSubstituteCourtSafeguardsNotAnInitiatorOrInterimCertificate", "HistoricalJudicialBoundary"),
            ("ECExactCaseAuthorizationEvidenceGroundAlreadyFinallyResolvedRegardlessOfLaterCurrentForce", "HistoricalFinalityFinding"),
            ("ECCurrentMeritsCorrectionExpiryOrOfficeChangeDoesNotEraseThatPastResolutionOrReopenItsKey", "HistoricalNoReplayReset"),
            ("ECLawfulHistoryRetainedAndIndependentlyCorrectableNoInferencePersistenceOrAutomaticAdmission", "HistoricalAdmissionBoundary"),
            ("ECHistoryAloneDoesNotEnactPermitContinueRestrictionAuthenticateOrOperateArchive", "HistoryNoCurrentPower"),
        ])
        .extra("authorized($source, ECReplayRegistryAuthority, $record)")
        .extra("authorized($review, ECReplayReviewAuthority, $record)")
        .extra("member($ground, ECGuardianStayGround)")
        .extra("member($outcome, ECGuardianMeritsOutcome)")
        .extra("(($merits_kind = ECFinalGuardianMerits & $merits_operator = FSBOD_17) | ($merits_kind = ECSubstituteFinalGuardianMerits & $merits_operator = FSBOD_32))")
        .default("$ground", "ECIrreversibleHarmGround")
        .default("$outcome", "ECMeritsRequireProtection")
        .default("$merits_kind", "ECFinalGuardianMerits")
        .default("$merits_operator", "FSBOD_17")
        .effect("end($record, ECHistoricalGuardianFinalDisposition)")
        .duty("ECRetainExactAuthenticatedFinalReplayHistoryWithoutContinuingExpiredAuthority");
    for actor in [
        "$source",
        "$review",
        "$reader",
        "$alternate",
        "$auditor",
        "$operator",
    ] {
        for interested in [
            "$merits_source",
            "$merits_review",
            "$merits_operator",
            "$court_source",
            "$court_evidence",
            "$court_review",
            "FSBOD_22",
            "FSBOD_31",
        ] {
            history = history.extra(&format!("~({actor} = {interested})"));
        }
    }
    // The historical certificate must join the original authenticated record's
    // actual keyed witnesses. Its legal force need not still be current today.
    // Full at-decision judicial validity is the separately authenticated input
    // above; it is not inferred from these identity observations alone.
    for (actor, role) in [
        ("$merits_source", "ECSourceAuthority"),
        ("$merits_review", "ECIndependentReviewAuthority"),
    ] {
        history = history.extra(&format!("authorized({actor}, {role}, $merits_record)"));
        for (value, scope) in [
            ("$subject", "Subject"),
            ("$case", "Case"),
            ("$project", "ChallengedActivity"),
            ("$authorization_record", "ChallengedAuthorizationRecord"),
            ("$authorization_version", "AuthorizationVersion"),
            ("$evidence_version", "EvidenceVersion"),
            ("$ground", "StayGround"),
            ("$outcome", "FinalMeritsOutcome"),
            ("$merits_revision", "Revision"),
            ("$merits_kind", "Kind"),
            ("$merits_operator", "ResponsibleActor"),
            ("$merits_version", "ConstitutionVersion"),
            ("$jurisdiction", "Jurisdiction"),
            ("$scope", "LegalScope"),
            ("$merits_window", "Window"),
            ("$merits_start", "Start"),
            ("$merits_end", "End"),
            ("$merits_evaluation", "FreshEvaluation"),
            (
                "ECFinalIndependentMeritsNotInitiatorAdviceOrInterimOrder",
                "FinalDisposition",
            ),
        ] {
            history = history.extra(&records::observe(actor, "$merits_record", value, scope));
        }
    }
    history = history.extra("~($merits_source = $merits_review)");
    cards.push(history);
    for (original, id, kind) in [
        (
            "guardian-material-authorization-stay",
            "guardian-authorization-after-history",
            "ECGuardianNewAuthorizationAfterHistoricalFinality",
        ),
        (
            "guardian-material-evidence-stay",
            "guardian-evidence-after-history",
            "ECGuardianNewEvidenceAfterHistoricalFinality",
        ),
        (
            "alternate-material-authorization-stay",
            "alternate-authorization-after-history",
            "ECAlternateNewAuthorizationAfterHistoricalFinality",
        ),
        (
            "alternate-material-evidence-stay",
            "alternate-evidence-after-history",
            "ECAlternateNewEvidenceAfterHistoricalFinality",
        ),
    ] {
        let mut renewal = cards
            .iter()
            .find(|card| card.id == original)
            .unwrap()
            .clone();
        renewal.id = id;
        renewal.kind = kind;
        renewal
            .dependencies
            .retain(|dependency| dependency.id != "guardian-merits");
        renewal
            .dependency_bindings
            .retain(|(key, _), _| key != "guardian-merits");
        renewal = renewal
            .fields(&[(
                "$prior_history_record",
                "PriorAuthenticatedFinalHistoryRecord",
            )])
            .depends(&["guardian-final-history"])
            .join("guardian-final-history", "$record", "$prior_history_record")
            .join(
                "guardian-final-history",
                "$merits_record",
                "$prior_final_record",
            )
            .join(
                "guardian-final-history",
                "$authorization_record",
                "$prior_authorization_record",
            )
            .join(
                "guardian-final-history",
                "$authorization_version",
                "$prior_authorization_version",
            )
            .join(
                "guardian-final-history",
                "$evidence_version",
                "$prior_evidence_version",
            )
            .join("guardian-final-history", "$ground", "$prior_ground");
        cards.push(renewal);
    }
}
