// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lawful evidence demands and accountable advocacy, never an operated audit.

use super::Card;

pub(super) fn cards() -> Vec<Card> {
    [
        ("guardian-information", "ECGuardianInformationFunction", "guardian-result", "FSBOD_22", "ECCommonsAndFutureConditionPurpose"),
        ("alternate-guardian-information", "ECAlternateGuardianInformationFunction", "guardian-alternate-function", "FSBOD_31", "ECCommonsAndFutureConditionPurpose"),
        ("animal-advocate-information", "ECAnimalAdvocateInformationFunction", "animal-advocate-result", "FSBOD_23", "ECPlausibleDirectAnimalProtectedInterestPurpose"),
        ("alternate-animal-information", "ECAlternateAnimalInformationFunction", "animal-alternate-function", "FSBOD_33", "ECPlausibleDirectAnimalProtectedInterestPurpose"),
    ].into_iter().map(|(id, kind, mandate, office, purpose)| {
        Card::new(id, kind, "Class6BoundedLawfulInstitutionalInformation")
            .fields(&[
                (purpose, "InstitutionalInformationPurpose"),
                ("$responsible_body", "ResponsibleAssessmentOrRecordBody"),
                ("$requested_information", "ExactLawfulInformationRequest"),
                ("$assessment", "RequestedAssessmentEvidenceUncertaintyAndAlternatives"),
                ("$lawful_basis", "InformationDisclosureLegalBasis"),
                ("$participation", "AccessibleParticipationAndObjectionRoute"),
                ("$execution_evidence", "DecisionReceiptAndExecutionEvidenceOrRecordedAbsence"),
                ("ECNecessaryProportionateMinimizedLawfulDisclosureAndIndependentAccess", "InformationNecessity"),
                ("ECPrivateEvidenceProtectedWithAccessibleReasonsAndLawfulSealedReview", "InformationPrivacy"),
                ("ECResponsibleBodyMustPublishAssessmentUncertaintyAlternativesAndReasons", "AssessmentPublication"),
                ("ECIndependentlyReviewableCommissionedEvidenceNotScientificOracle", "CommissionedEvidence"),
                ("ECSupportedWarningsRecommendationsDissentAndIndependentChallenges", "AdvocacyLimit"),
                ("ECExamineWhetherDecisionsWereReceivedAndActedOnWithoutAssumingSuccess", "ExecutionEvidenceBoundary"),
                ("ECMissingUnconfirmedOrFailedOperationRemainsUnestablishedOrFailure", "NoInventedOperation"),
                ("ECNoSearchEntrySeizureCompelledTestPersonalPenaltyOrTaxPower", "InformationPowerLimit"),
                ("ECNoPolicyVetoPermitOperationProsecutionOrFinalMeritsAuthority", "InstitutionalSeparation"),
                ("ECCorrectionIndependentChallengeContinuityAndOrdinaryCourtRemedy", "InformationRemedy"),
            ])
            .depends(&[mandate])
            .extra(&format!("$operator = {office}"))
            .extra("~($operator = $responsible_body)")
            .default("$operator", office)
            .effect("permits($operator, ECOnlyExactLawfulInformationAndReviewedEvidenceDemand, $record)")
            .effect("obliged($responsible_body, ECPublishLawfulAssessmentEvidenceUncertaintyAlternativesAndReasons, $record)")
            .duty("ECFacilitateParticipationPublishSupportedAdviceAndIndependentlyExamineResponseFailure")
    }).collect()
}
