// SPDX-License-Identifier: MIT OR Apache-2.0

//! Positive, predeclared functional substitutes; silence supplies no mandate.

use super::Card;

pub(super) const FAILURES: &[&str] = &[
    "ECInstitutionalSilence",
    "ECInstitutionalVacancy",
    "ECInstitutionalConflict",
    "ECInstitutionalCapture",
    "ECInstitutionalRemoval",
    "ECInstitutionalRefusal",
    "ECInstitutionalIncapacity",
];

pub(super) fn vocabulary_rules() -> Vec<String> {
    FAILURES.iter().map(|ground| format!(
        "all $writer: all $record: observe($writer, $record, {ground}, ECUnavailabilityGroundScope) -> member({ground}, ECInstitutionUnavailable)."
    )).collect()
}

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, dependency, office, function) in [
        (
            "guardian-alternate-function",
            "ECGuardianAlternateFunction",
            "guardian-alternate-result",
            "FSBOD_31",
            "ECGuardianAdvocacyOnly",
        ),
        (
            "animal-alternate-function",
            "ECAnimalAlternateFunction",
            "animal-alternate-result",
            "FSBOD_33",
            "ECAnimalAdvocacyOnly",
        ),
        (
            "substitute-reviewer-function",
            "ECSubstituteReviewerFunction",
            "substitute-reviewer-appointment",
            "FSBOD_32",
            "ECIndependentGuardianMeritsReviewOnly",
        ),
    ] {
        let mut card = Card::new(id, kind, "Class6PredeclaredIndependentFunctionalContinuity")
            .fields(&[
                ("$unavailable_actor", "UnavailableActor"),
                ("$original_mandate", "OriginalFunctionOrVacantMandate"),
                ("$failure", "UnavailabilityGround"),
                ("$failure_evidence", "IndependentUnavailabilityEvidence"),
                ("$predeclaration", "PredeclaredSubstituteDeclaration"),
                (function, "SubstitutedFunction"),
                (
                    "ECPositivelyEstablishedUnavailabilityNotSilenceAsApproval",
                    "PositiveUnavailability",
                ),
                (
                    "ECAuthenticDeclarationPredatesFailureAndNamesApplicableSubstitute",
                    "PredeclarationFinding",
                ),
                (
                    "ECActualCurrentMandateCoversThisCaseTerritoryAndFunction",
                    "SubstituteMandateCoverage",
                ),
                (
                    "ECSeparateOwnCurrentFindingNotAnInheritedClockOrAutomaticRenewal",
                    "SubstituteTemporalBoundary",
                ),
                (
                    "ECSharedCaseAuthorizationEvidenceGroundAndFinalityNotAnOfficeLocalReset",
                    "SharedReplayBoundary",
                ),
                (
                    "ECPublicReasonsIndependentChallengeCorrectionAndProtectedContinuity",
                    "SubstituteAccountability",
                ),
                (
                    "ECNoApprovalVetoIndefiniteHoldOrUnrelatedPowerFromInstitutionalFailure",
                    "NoFailureExpansion",
                ),
            ])
            .depends(&[dependency])
            .extra(&format!("$operator = {office}"))
            .extra("~($operator = $unavailable_actor)")
            .extra("member($failure, ECInstitutionUnavailable)")
            .default("$operator", office)
            .default("$failure", "ECInstitutionalConflict")
            .duty("ECPreserveOnlyThePositivelyAssignedIndependentSubstituteFunction");
        if id == "substitute-reviewer-function" {
            card = card
                .separate_context(dependency)
                .join(dependency, "$subject", "$substitute_reviewer")
                .fields(&[
                    ("$substitute_reviewer", "AssignedIndependentReviewer"),
                    ("$initiating_advocate", "InitiatingAdvocate"),
                    ("$original_reviewer_assignment", "OriginalExactCaseJudicialAssignment"),
                    (
                        "ECIndependentReviewConfirmsUnavailableActorHeldThisExactJudicialAssignment",
                        "OriginalAssignmentAndFailureFinding",
                    ),
                    (
                        "ECNoApprovalByUnavailableOriginalReviewerRequired",
                        "NoUnavailableReviewerVeto",
                    ),
                    (
                        "ECActualOrdinaryJudicialSafeguardsRemainRequiredForEachDisposition",
                        "JudicialContractBoundary",
                    ),
                    (
                        "ECReviewerIsNotGuardianAlternateAnimalAdvocateRegulatorOrOperator",
                        "SubstituteReviewerSeparation",
                    ),
                ])
                .extra("~($substitute_reviewer = $unavailable_actor)")
                .extra("~($substitute_reviewer = $initiating_advocate)")
                .extra("~($operator = $initiating_advocate)");
        } else {
            let original = if id == "guardian-alternate-function" {
                "FSBOD_22"
            } else {
                "FSBOD_23"
            };
            card = card
                .extra(&format!("$unavailable_actor = {original}"))
                .default("$unavailable_actor", original);
        }
        cards.push(card);
    }
    cards
}
