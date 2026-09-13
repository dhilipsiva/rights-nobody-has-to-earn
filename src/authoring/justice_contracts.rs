// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Card;

pub(super) const DOMAINS: &[&str] = &[
    "CivilJustice",
    "AdministrativeJustice",
    "FamilyJustice",
    "LabourJustice",
    "ConsumerJustice",
    "ConstitutionalJustice",
    "CriminalJustice",
];
pub(super) const RELIEFS: &[&str] = &[
    "Restitution",
    "Reparation",
    "Compensation",
    "Cessation",
    "SpecifiedPerformance",
    "RightsBoundedProtectiveArrangement",
];
pub(super) const BARRIERS: &[&str] = &[
    "JusticeWealthDocumentationOrStatusConditionedAccess",
    "JusticeSupportConditionedOnReportingTestimonyReconciliationOrForgiveness",
    "JusticeSupportAsProofOfGuilt",
    "JusticeAggregateEvidenceReversingCriminalBurden",
    "JusticeRestorationRefusalAsAdversePersonFinding",
    "JusticePrivateEvidenceAsWorthRiskOrEligibilityScore",
    "JusticeCaseReliefAsGeneralInvalidation",
    "JusticeEnforcementTakingStandingFloorOrPoliticalVoice",
    "JusticeProcessRecordAsConvictionCustodyOrCoerciveAuthority",
    "JusticeCustodyAsBodilyAbuseIsolationOrLossOfCounsel",
    "JusticeReleaseConditionalOnDebtLabourOrForgiveness",
    "JusticeInstitutionalDelayExtendingCustody",
];
pub(super) const DUTIES: &[&str] = &[
    "ProvideAccessibleNoticeCounselHearingChallengeAndEffectiveRemedy",
    "ProvideIndependentSurvivorSupportWithoutVerdictOrCooperationCondition",
    "PreserveRightsAndNonRetaliationDuringJusticeProceedings",
];

pub(super) fn vocabularies() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("JJusticeDomain", DOMAINS.to_vec()),
        ("JReliefKind", RELIEFS.to_vec()),
        (
            "JChargeOutcome",
            vec!["ReasonedChargeDecision", "ReasonedNonChargeDecision"],
        ),
        (
            "JDefectKind",
            vec![
                "UnfairProcess",
                "InterestedReview",
                "WithheldDefenceEvidence",
                "CoercedRestoration",
                "DisproportionateRelief",
                "RightsViolatingEnforcement",
                "UnlawfulCustodialConditions",
                "DelayedLawfulRelease",
                "PrivacyBreach",
                "ExpiredOrSupersededFinding",
            ],
        ),
    ]
}

pub(super) fn cards() -> Vec<Card> {
    vec![
        Card::new(
            "access",
            "JusticeAccessibleProcess",
            &[
                ("TimelyUnderstandableNoticeAndPreparationTime", "Notice"),
                (
                    "AccessibleInformationInterpreterAccommodationAndNonDigitalRoute",
                    "Access",
                ),
                (
                    "IndependentCounselOrAdvocateWithoutWealthDocumentationOrStatusBar",
                    "Assistance",
                ),
                ("EffectiveHearingChallengePublicReasonsAndRemedy", "Process"),
            ],
            &["obliged($operator, SupplyAccessibleJusticeProcess, $record)"],
        ),
        Card::new(
            "assistance",
            "JusticeIndependentAssistance",
            &[
                ("$defender", "Defender"),
                (
                    "ChosenOrIndependentlyAppointedCompetentCounselAdvocate",
                    "Representation",
                ),
                (
                    "ConfidentialCommunicationAdequateTimeFacilitiesAndEffectiveResources",
                    "Facilities",
                ),
                (
                    "ChildIndependentVoiceAndSeparateAssistanceForConflictingInterests",
                    "Children",
                ),
                (
                    "SupportedAgencyNoRepresentativeCreatedStandingOrBlanketControl",
                    "Agency",
                ),
            ],
            &["obliged($operator, MakeIndependentJusticeAssistanceAvailable, $record)"],
        ),
        Card::new(
            "investigation",
            "JusticeInvestigationProcedure",
            &[
                ("$investigator", "Investigator"),
                ("$prosecutor", "Prosecutor"),
                (
                    "CaseBoundLawfulMandateGroundReasonsAndIndependentComplaint",
                    "Mandate",
                ),
                (
                    "PreserveInculpatoryAndExculpatoryEvidenceAndProvenance",
                    "EvidenceHandling",
                ),
                (
                    "LeastIntrusiveLawfulMeansNoCoerciveAuthorityFromThisFinding",
                    "Means",
                ),
                (
                    "NoChargingAdjudicatingExecutingOrFinalSelfReview",
                    "Separation",
                ),
                (
                    "FairDisclosureWithIndependentlyReviewedNecessaryPrivacyProtection",
                    "Disclosure",
                ),
            ],
            &["obliged($investigator, PreserveAndDiscloseLawfulCaseEvidence, $record)"],
        ),
        Card::new(
            "prosecution",
            "JusticeChargingProcedure",
            &[
                ("$investigator", "Investigator"),
                ("$prosecutor", "Prosecutor"),
                ("$adjudicator", "Adjudicator"),
                ("$executor", "Executor"),
                ("$outcome", "ChargeOutcome"),
                (
                    "IndependentLawfulChargingFunctionWithAccessibleReasonsAndReview",
                    "Mandate",
                ),
                (
                    "ApplicableIndividualProofNoAggregateBurdenReversalOrSupportAsGuilt",
                    "Proof",
                ),
                (
                    "NoInvestigationAdjudicationExecutionOrFinalSelfReviewFusion",
                    "Separation",
                ),
            ],
            &["obliged($prosecutor, GiveReasonedReviewableChargingDisposition, $record)"],
        )
        .extra("member($outcome, JChargeOutcome)"),
        Card::new(
            "defence",
            "JusticeConfidentialDefence",
            &[
                ("$investigator", "Investigator"),
                ("$prosecutor", "Prosecutor"),
                ("$defender", "Defender"),
                ("$adjudicator", "Adjudicator"),
                (
                    "IndependentLoyalRepresentationNotControlledByAccuserOrExecutive",
                    "Mandate",
                ),
                (
                    "UsableCaseEvidenceExculpatoryDisclosureAdequatePreparationAndChallenge",
                    "Facilities",
                ),
                (
                    "ConfidentialCounselInterpreterAccommodationAndChosenSupport",
                    "Communication",
                ),
                (
                    "PresumptionOfInnocenceSilenceNotGuiltNoCompelledSelfIncrimination",
                    "Proof",
                ),
            ],
            &["obliged($operator, PreserveEffectiveIndependentDefence, $record)"],
        ),
        Card::new(
            "hearing",
            "JusticeFairHearing",
            &[
                ("$adjudicator", "Adjudicator"),
                (
                    "EffectiveNoticePreparationEvidenceAccessAndFairOpportunityToAnswer",
                    "Hearing",
                ),
                (
                    "IndependentImpartialCompetentDecisionMakerWithinExistingMandate",
                    "DecisionMaker",
                ),
                (
                    "DomainApplicableProofIndividualReasonsNoCriminalBurdenReversal",
                    "Proof",
                ),
                (
                    "NecessaryReviewedPrivacyProtectionPreservesEffectiveDefence",
                    "Disclosure",
                ),
                (
                    "AccessibleReasonedDecisionEffectiveIndependentAppealAndInterimProtection",
                    "Appeal",
                ),
                (
                    "NoInvestigationProsecutionExecutionOrFinalReviewOfOwnCase",
                    "FunctionScope",
                ),
            ],
            &["obliged($operator, ProvideFairReasonedAndContestableHearing, $record)"],
        ),
        Card::new(
            "survivor",
            "JusticeSurvivorProtection",
            &[
                (
                    "IndependentConfidentialAdviceSupportAndNonCoerciveSafetyPlanning",
                    "Support",
                ),
                (
                    "NoConvictionReportingTestimonyReconciliationOrForgivenessCondition",
                    "Access",
                ),
                (
                    "ChildAgencyConflictFreeSupportPrivacyAndNonRetaliation",
                    "Protection",
                ),
                (
                    "NoPredeterminedGuiltNoDefenceDenialNoUnprovidedCoerciveInstrument",
                    "Limits",
                ),
            ],
            &["obliged($operator, ProvideIndependentSurvivorProtection, $record)"],
        ),
        Card::new(
            "restoration",
            "JusticeVoluntaryRestoration",
            &[
                ("$agreement", "Agreement"),
                ("$participants", "ParticipantRecord"),
                (
                    "CompleteExternallyAssuredAffectedParticipantRecordAllActualConsents",
                    "Completeness",
                ),
                (
                    "FreeInformedRevocableChoiceIndependentAdviceAndNoCoercion",
                    "Consent",
                ),
                (
                    "LawfulSpecificAgreementNoThirdPartyWaiverNoPublicAccountabilityErasure",
                    "AgreementLimits",
                ),
                (
                    "NoForgivenessDutyNoRefusalPenaltyNoAutomaticCriminalOrReleaseEffect",
                    "Refusal",
                ),
            ],
            &["permits($operator, FacilitateSpecifiedVoluntaryRestoration, $record)"],
        )
        .extra("observe($subject, $record, ActualInformedConsent, JParticipantChoiceScope)")
        .extra("observe($other_party, $record, ActualInformedConsent, JParticipantChoiceScope)")
        .extra("~related($record, JRestorationWithdrawn)"),
        Card::new(
            "case-relief",
            "JusticeCaseSpecificRelief",
            &[
                ("$relief", "ReliefKind"),
                ("$obligor", "Obligor"),
                ("$adjudicator", "Adjudicator"),
                ("$executor", "Executor"),
                (
                    "IndividuallyEstablishedWrongLiabilityAndEffectiveRemedyBasis",
                    "ReliefBasis",
                ),
                (
                    "NecessaryProportionateRightsPreservingReliefWithinJudicialScope",
                    "ReliefLimits",
                ),
                (
                    "NoGeneralInvalidationNoNewCoercivePowerNoFindingOfDelivery",
                    "Limits",
                ),
            ],
            &["permits($operator, RelyOnSpecifiedCaseRelief, $record)"],
        )
        .depends("hearing")
        .court(22, "case_specific_relief")
        .extra("member($relief, JReliefKind)"),
        Card::new(
            "general-invalidation",
            "JusticeConstitutionalInvalidation",
            &[
                ("$target", "ChallengedAction"),
                (
                    "ConstitutionalQuestionLawfulGeneralEffectAndPublicReasons",
                    "Ground",
                ),
                ("NoOrdinaryPolicyOrGovernmentAdministration", "Limits"),
                (
                    "EffectiveRemedyIndependentChallengeAndLawfulFutureCorrection",
                    "Review",
                ),
            ],
            &["permits($operator, RelyOnSpecifiedConstitutionalInvalidation, $record)"],
        )
        .court(23, "constitutional_invalidation"),
        Card::new(
            "composition-review",
            "JusticeUninvolvedCompositionReview",
            &[
                ("$target", "CompositionChallenge"),
                (
                    "PredeclaredUninvolvedPanelNoChallengedMemberDecidesOwnComposition",
                    "Panel",
                ),
                (
                    "PublicReasonsEffectiveCorrectionAndRightsContinuity",
                    "Review",
                ),
            ],
            &["obliged($operator, GiveUninvolvedCompositionReviewAndCorrection, $record)"],
        )
        .court(25, "alternate_composition_panel"),
        Card::new(
            "enforcement",
            "JusticeNoncoerciveEnforcement",
            &[
                ("$relief", "ReliefKind"),
                ("$obligor", "Obligor"),
                ("$adjudicator", "Adjudicator"),
                ("$executor", "Executor"),
                (
                    "ExactLawfulOrderNamedExecutorMeansObjectAndSourceBoundEnd",
                    "ExecutionScope",
                ),
                (
                    "NoticeOpportunityToComplyContestAndIndependentStayReview",
                    "EnforcementProcess",
                ),
                (
                    "LeastCoerciveProportionateMeansPreserveEssentialsThirdPartiesAndRights",
                    "EnforcementLimits",
                ),
                (
                    "NonCoerciveImplementationOnlyNoArrestSearchSeizureOrCustodyAuthority",
                    "Means",
                ),
            ],
            &["permits($operator, ImplementSpecifiedNoncoerciveRemedy, $record)"],
        )
        .depends("case-relief")
        .extra("member($relief, JReliefKind)")
        .extra("observe($source, $record, $operator, JExecutorScope)"),
        Card::new(
            "appeal",
            "JusticeIndependentAppeal",
            &[
                ("$target", "ChallengedRecord"),
                (
                    "AccessibleIndependentReviewErrorsEvidenceRightsAndEffectiveInterimRelief",
                    "Appeal",
                ),
                (
                    "NoOriginalActorPermissionNoRetaliationNoLossOfServicesOrVoice",
                    "Access",
                ),
            ],
            &["obliged($reader, HearJusticeAppealAndGiveEffectiveRelief, $record)"],
        ),
        Card::new(
            "conditions",
            "JusticeCustodialSafeguards",
            &[
                (
                    "BodilyIntegrityHumaneConditionsNoTortureCrueltyOrAbuse",
                    "Integrity",
                ),
                (
                    "ConfidentialCounselFamilyCommunicationIndependentComplaintAndInspection",
                    "Communication",
                ),
                (
                    "HealthHousingEducationBeliefVoiceAndAccommodationContinue",
                    "Conditions",
                ),
                (
                    "NoNewDeprivationNoCustodyExtensionNoConditionsAsPunishment",
                    "Limits",
                ),
            ],
            &["obliged($operator, MaintainRightsRespectingCustodialConditions, $record)"],
        ),
        Card::new(
            "release-review",
            "JusticeReleaseReview",
            &[
                ("$custody_case", "CustodyCase"),
                (
                    "CurrentLawfulGroundEndNecessityProportionalityAndAlternativesExamined",
                    "Review",
                ),
                (
                    "IndependentAccessibleReleaseChallengeCounselAndReasonedDecision",
                    "Process",
                ),
                (
                    "NoAutomaticRenewalNoProcedureDelayBeyondLawfulEnd",
                    "Limits",
                ),
                (
                    "ImmediateLawfulReleaseAndContinuityWithoutDebtWorkOrForgivenessTest",
                    "ContinuityPlan",
                ),
            ],
            &["obliged($operator, ActOnIndependentLawfulReleaseReview, $record)"],
        ),
        Card::new(
            "continuity",
            "JusticeReleaseContinuity",
            &[
                ("$custody_case", "CustodyCase"),
                (
                    "HousingCareDocumentsEducationWorkAccessAndCommunityParticipation",
                    "ContinuityPlan",
                ),
                (
                    "NoDebtLabourObedienceRecordPerfectionOrForgivenessCondition",
                    "Access",
                ),
                (
                    "PrivateNonStigmatizingSupportNoGeneralSupervisionOrRiskStatus",
                    "PrivacyLimits",
                ),
                (
                    "ServicesAndAgencyContinueWithoutProofOfInstitutionalSuccess",
                    "Limits",
                ),
            ],
            &["obliged($operator, PreservePostReleaseContinuityAndReintegration, $record)"],
        ),
        Card::new(
            "defect",
            "JusticeReviewedDefect",
            &[
                ("$target", "AffectedRecord"),
                ("$defect", "DefectKind"),
                (
                    "PositiveSpecificEvidenceIndependentFindingAndProportionateAffectedScope",
                    "DefectBasis",
                ),
                (
                    "PreserveEvidenceAndRightsCorrectRemedyReauditAndPreventRecurrence",
                    "Response",
                ),
            ],
            &[
                "contradict($target, JEffectPermission)",
                "obliged($operator, PreserveCorrectAndRemedyJusticeDefect, $record)",
            ],
        )
        .extra("member($defect, JDefectKind)"),
        Card::new(
            "nonresponse",
            "JusticeEstablishedNonresponse",
            &[
                ("$requester", "Requester"),
                ("$target", "ReviewRequest"),
                (
                    "PositiveNoticeReceiptOpportunityAndSourceBoundDeadlineElapsed",
                    "Nonresponse",
                ),
                (
                    "IndependentAlternateUnderSameLimitsNoSilenceAsApproval",
                    "AlternateDuty",
                ),
            ],
            &["obliged($alternate, PerformUnfulfilledJusticeReview, $record)"],
        )
        .extra("challenge($requester, $reader, $target)")
        .extra("authorized($reader, JChallengeReaderAuthority, $target)")
        .extra("~($requester = $reader)")
        .extra("~($requester = $alternate)")
        .extra("~($requester = $review)"),
    ]
}
