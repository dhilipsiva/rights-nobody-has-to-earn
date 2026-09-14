// SPDX-License-Identifier: MIT OR Apache-2.0

//! Actual court contracts, distinct interim relief and final protection.

use super::Card;

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, claim) in [
        (
            "person-judicial-interim",
            "ECPersonJudicialInterimRelief",
            "present-person-commons-claim",
        ),
        (
            "association-judicial-interim",
            "ECAssociationJudicialInterimRelief",
            "association-commons-claim",
        ),
        (
            "rights-advocate-judicial-interim",
            "ECRightsAdvocateJudicialInterimRelief",
            "rights-advocate-commons-claim",
        ),
        (
            "guardian-judicial-interim",
            "ECGuardianJudicialInterimRelief",
            "guardian-commons-claim",
        ),
    ] {
        cards.push(
            Card::new(id, kind, "Class4IndependentJudicialInterimProtection")
                .concludes("interrupt($operator, $record, ECJudicialInterimStay)")
                .fields(&[
                    ("$project", "ChallengedActivity"),
                    ("$authorization_record", "ChallengedAuthorizationRecord"),
                    ("$authorization_version", "AuthorizationVersion"),
                    ("$irreversible_part", "IrreversiblePart"),
                    ("$judicial_direction", "ExactJudicialInterimDirection"),
                    (
                        "ECIndependentCurrentEvidenceOfSeriousOrIrreversibleRiskAndNeed",
                        "InterimNeed",
                    ),
                    (
                        "ECActualPendingProceedingAndNoFinalDispositionOnThisPair",
                        "PendingMeritsFinding",
                    ),
                    (
                        "ECProportionateLeastRestrictiveAffectedPartWithReasonsAndHearing",
                        "InterimProportionality",
                    ),
                    (
                        "ECOwnFreshSourceBoundJudicialWindowNotGuardianClockOrSilentExtension",
                        "JudicialInterimTemporalContract",
                    ),
                    (
                        "ECIndependentEssentialFloorAnimalCareAndReversibleContinuity",
                        "InterimContinuity",
                    ),
                    (
                        "ECRequestAloneConfersNoAutomaticStayOrMeritsPower",
                        "RequestDecisionSeparation",
                    ),
                    (
                        "ECPromptIndependentReviewCorrectionRemedyAndEnd",
                        "InterimReview",
                    ),
                ])
                .depends(&[claim])
                .extra("$operator = FSBOD_17")
                .extra("~end($record, ECFinallyResolvedAuthorizationEvidencePair)")
                .default("$operator", "FSBOD_17")
                .effect(
                    "oppose($authorization_record, $authorization_version, ECJudicialInterimStay)",
                )
                .duty("ECApplyOnlyTheActualCurrentIndependentInterimDirection"),
        );
    }
    for (id, kind, merits, office) in [
        (
            "guardian-final-protection",
            "ECGuardianFinalJudicialProtection",
            "guardian-merits",
            "FSBOD_17",
        ),
        (
            "substitute-final-protection",
            "ECSubstituteFinalJudicialProtection",
            "substitute-guardian-merits",
            "FSBOD_32",
        ),
    ] {
        cards.push(Card::new(id, kind, "Class4CaseSpecificFinalProtectiveDirection")
            .concludes("interrupt($operator, $record, ECFinalJudicialProtection)")
            .fields(&[
                ("$project", "ChallengedActivity"),
                ("$authorization_record", "ChallengedAuthorizationRecord"),
                ("$authorization_version", "AuthorizationVersion"),
                ("$responsible_controller", "AdjudicatedResponsibleController"),
                ("$judicial_direction", "ExactFinalProtectiveDirection"),
                ("ECActualFinalDecisionRequiresThisCaseSpecificProtection", "FinalProtectionFinding"),
                ("ECOnlyAdjudicatedAffectedActNotUnrelatedReversibleFloorContinuity", "FinalProtectionLimit"),
                ("ECGuardianStayExpiryDoesNotEraseFinalJudicialProtection", "FinalOrderIndependence"),
                ("ECCurrentLegalForceCorrectionAppealAndSourceBoundCessationSeparatelyReviewed", "FinalOrderCurrentness"),
                ("ECNoPerformedRestorationNoGeneralInvalidationAndNoPunitiveFinding", "FinalOrderBoundary"),
            ])
            .depends(&[merits])
            .join(merits, "$outcome", "ECMeritsRestrictChallengedAct")
            .join(merits, "$remedy", "$judicial_direction")
            .extra(&format!("$operator = {office}"))
            .default("$operator", office)
            .effect("oppose($authorization_record, $authorization_version, ECFinalJudicialProtection)")
            .effect("obliged($responsible_controller, ECComplyWithExactFinalProtectiveDirection, $record)")
            .duty("ECSecureCaseSpecificProtectionContinuityAndEffectiveReviewWithoutClaimingExecution"));
    }
    cards
}
