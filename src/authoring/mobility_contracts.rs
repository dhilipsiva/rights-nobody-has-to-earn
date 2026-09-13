// SPDX-License-Identifier: MIT OR Apache-2.0

//! Semantic cards: each kind has a direct effect, not an omnibus group status.

use super::Card;

pub(super) const HARMS: &[&str] = &[
    "PermanentRelocation",
    "TitleExtinguishment",
    "IrreversibleTitleImpairment",
    "SovereigntyTransferOverCollectiveLand",
    "SacredSiteDestruction",
    "HazardousMaterialPlacement",
    "ComparableExistentialHarm",
];

pub(super) const BARRIERS: &[&str] = &[
    "MobilityStatusConditionedStandingFloorOrRemedy",
    "MobilityMissingRecordAdverseInference",
    "MobilityEnforcementEnrolmentThroughServices",
    "MobilityEnforcementCollectionThroughServices",
    "MobilityEnforcementTransmissionThroughServices",
    "MobilityRefoulementIncludingOnwardTransfer",
    "MobilityCollectiveExpulsion",
    "MobilityChildImmigrationDetention",
    "MobilityUnnecessaryOrUnreviewedAdultDetention",
    "MobilityDetentionWithoutMaximumOrRealAlternative",
    "MobilityStatelessnessCreationOrPerpetuation",
    "MobilityPunitiveNationalityDeprivation",
    "MobilityPushbackOrExternalisedDutyEvasion",
    "MobilityExclusionFromScarcityPopulation",
    "MobilityStatusBasedScarcityPriority",
    "PluralityForcedAssimilationOrSegregation",
    "PluralityCompelledBeliefLabourMarriageOrConformity",
    "PluralityBlockedExitDissentOrConfidentialHelp",
    "PluralityMembershipWorthRiskOrEligibilityReuse",
    "PluralityMembershipPoliticalWeight",
    "PluralityInternalLawLoweringIndividualRights",
    "PluralityNonmemberCommonServiceDenial",
    "PluralityCollectiveGuilt",
    "PluralityTitleLossByMembershipExit",
    "PluralityConservationAsDispossession",
    "ExternalExportOfDomesticProhibitedHarm",
];

pub(super) fn cards() -> Vec<Card> {
    vec![
        Card::new(
            "asylum",
            "MobilityFairDetermination",
            &[
                (
                    "AccessibleInformationAdvocateInterpreterHearingReasons",
                    "Process",
                ),
                ("EffectiveIndependentSuspensiveAppeal", "Appeal"),
                (
                    "NewcomerMigrantRefugeeOrStatelessClaimNotDiscretionaryFavour",
                    "Access",
                ),
                (
                    "IndividualDeterminationWithoutStatusConditionedServices",
                    "Decision",
                ),
            ],
            &[],
            &["obliged($operator, ProvideFairAsylumDetermination, $record)"],
        ),
        Card::new(
            "removal",
            "MobilityRemovalCompatibility",
            &[
                ("$transfer", "TransferKind"),
                ("IndividualJudicialDecisionAndDisclosedReasons", "Decision"),
                ("AccessibleAdvocateInterpreterAndHearing", "Process"),
                ("EffectiveIndependentSuspensiveAppeal", "Appeal"),
                (
                    "ConnectionsFamilyResidenceAndEachChildConsidered",
                    "PersonalConnections",
                ),
                (
                    "FullOnwardChainReviewedNoRefoulementOrCategoricalAbuse",
                    "TransferSafety",
                ),
                (
                    "DomesticOffenceNoPoliticalOrIdentityBeliefExpressionPretext",
                    "TransferGround",
                ),
                (
                    "ConstitutionalProceedingProtectionsEstablished",
                    "ForeignProcess",
                ),
                (
                    "AssuranceNotSubstituteForEvidenceOrExceptionToBar",
                    "DiplomaticAssurance",
                ),
                (
                    "NoStatelessnessNoCollectiveExpulsionNoDutyEvasion",
                    "RemovalLimits",
                ),
                (
                    "CompatibilityOnlySeparateCoerciveAuthorityRequired",
                    "NonExecution",
                ),
            ],
            &[],
            &["obliged($reader, PreserveSuspensiveRemovalReview, $record)"],
        )
        .extra("member($transfer, MPTransferKind)"),
        Card::new(
            "nationality",
            "MobilityNationalityRecognition",
            &[
                ("BornInRepublicOtherwiseStateless", "NationalityBasis"),
                (
                    "AccessibleAlternativeEvidenceNotConstitutiveRegistration",
                    "NationalityEvidence",
                ),
            ],
            &[],
            &["obliged($operator, RecognizeOtherwiseStatelessChildNationality, $record)"],
        ),
        Card::new(
            "identity",
            "PluralityBoundedCommunityFinding",
            &[
                ("$group_kind", "CommunityKind"),
                (
                    "IndependentEvidenceOfSpecifiedCommunityNotStateInventedIdentity",
                    "CommunityBasis",
                ),
            ],
            &[],
            &[],
        )
        .extra("member($group_kind, MPCommunityKind)"),
        Card::new(
            "membership",
            "PluralityMembershipOccurrence",
            &[
                ("$self_record", "SelfIdentificationRecord"),
                (
                    "KnowableLawfulAcceptanceWithContinuityAndNondiscrimination",
                    "Acceptance",
                ),
                (
                    "MultipleMembershipFreeExitPrivateProceduralChallenge",
                    "MembershipLimits",
                ),
            ],
            &[],
            &["obliged($operator, RespectPrivateMultipleMembershipAndExit, $record)"],
        )
        .extra("observe($holder, $self_record, $object, MPSelfIdentifiedCommunityScope)")
        .extra("~related($holder, $record, MPExitedMembership)"),
        Card::new(
            "culture",
            "PluralityCulturalProtection",
            &[
                (
                    "LanguageCultureBeliefEducationMediaAssociationInstitutions",
                    "CulturalCapacities",
                ),
                (
                    "AccessiblePublicInformationServicesAndEffectiveParticipation",
                    "CulturalAccess",
                ),
                (
                    "VoluntaryPracticeNoAssimilationSegregationOrForcedConformity",
                    "CulturalLimits",
                ),
                (
                    "CrossBoundaryCultureKinAndGenerationalContinuity",
                    "CulturalContinuity",
                ),
            ],
            &["identity"],
            &["obliged($operator, ProtectPluralCultureLanguageEducationAndParticipation, $record)"],
        ),
        Card::new(
            "indigenous-government",
            "PluralityIndigenousSelfGovernment",
            &[
                ("IndigenousPeople", "CommunityKind"),
                ("EstablishedInternalAndLocalCompetence", "GovernmentScope"),
                (
                    "InstitutionsOfOwnChoosingWithinIndividualRights",
                    "GovernmentBasis",
                ),
            ],
            &["identity"],
            &["authority($operator, IndigenousInternalAndLocalGovernment, $record)"],
        ),
        Card::new(
            "minority-territory",
            "PluralityMinorityTerritorialGovernment",
            &[
                ("MinorityCommunity", "CommunityKind"),
                (
                    "IndependentlyEstablishedHistoricalOrTerritorialBasis",
                    "TerritorialBasis",
                ),
                (
                    "FederalSubsidiarityEqualityAndResidenceDemocraticRoute",
                    "TerritorialAuthority",
                ),
                (
                    "NoCulturalLabelOnlyOrUnboundedJurisdiction",
                    "GovernmentScope",
                ),
            ],
            &["identity"],
            &["authority($operator, BoundedMinorityTerritorialGovernment, $record)"],
        ),
        Card::new(
            "title",
            "PluralityProtectedTitle",
            &[
                (
                    "IndependentlyEstablishedCollectiveLandWaterResourceOrCulturalTitle",
                    "TitleBasis",
                ),
                (
                    "HoldUseGovernProtectAndTransmitWithinRightsAndCommons",
                    "TitleCapacities",
                ),
                (
                    "NoTitleLossByExitEvacuationOrEcologicalPretext",
                    "TitleContinuity",
                ),
            ],
            &["identity"],
            &["obliged($operator, RespectAndPreserveCollectiveTitle, $record)"],
        ),
        Card::new(
            "internal-law",
            "PluralityInternalDecision",
            &[
                ("LawfulInternalSelectionOrCustomaryRule", "InternalDecision"),
                (
                    "GenuinelyInternalScopeNoGovernmentOverUnconsentingNeighbours",
                    "InternalScope",
                ),
                (
                    "IndividualVoiceDissentReasonsAppealAndChildRights",
                    "IndividualVoice",
                ),
                (
                    "NonmemberHousingPropertyCommonServicesAndEqualBallot",
                    "NonmemberRights",
                ),
                (
                    "NoSecondGeneralElectorateOrUnilateralSecession",
                    "PoliticalLimits",
                ),
                (
                    "VoluntaryPracticeConfidentialHelpAndExit",
                    "InternalLiberty",
                ),
            ],
            &["identity"],
            &["authority($operator, RightsBoundedInternalDecision, $record)"],
        ),
        Card::new(
            "representation",
            "PluralityLawfulRepresentation",
            &[
                ("$representative", "Representative"),
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                (
                    "CollectiveChosenLawfulProcessNotConvenientStateSpokesperson",
                    "RepresentativeSource",
                ),
                (
                    "ScopedMandateConflictDisclosureAndIndependentChallenge",
                    "RepresentativeLimits",
                ),
            ],
            &["identity"],
            &[],
        )
        .extra("~($representative = $source)")
        .extra("~($representative = $evidence)")
        .extra("~($representative = $review)"),
        Card::new(
            "consent",
            "PluralityActualConsent",
            &[
                ("$representative", "Representative"),
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                ("$effect", "Effect"),
                ("$administration", "DecisionAdministration"),
                ("$assurer", "CompletenessAssurer"),
                ("$result_service", "ResultService"),
                ("$certificate", "ConsentCertificate"),
                ("$roster", "DecisionRoster"),
                ("$decision_rule", "CollectiveDecisionRule"),
                (
                    "AuthenticatedClassifiedSubmissionsUnderLawfulCollectiveRule",
                    "SubmissionBasis",
                ),
                (
                    "IndependentlyCompleteNonemptyRosterAndParticipation",
                    "DecisionCompleteness",
                ),
                (
                    "AffirmativeActualConsentNoTieConflictSilenceOrCoercion",
                    "ConsentResult",
                ),
                (
                    "FreePriorInformedAccessibleInformationTimeAndIndependentAdvice",
                    "ConsentQuality",
                ),
                (
                    "PublicScopePrivateIdentityChallengeCorrectionAndMaterialBreachRemedy",
                    "ConsentProcess",
                ),
                (
                    "AllAffectedHoldersTitlesAndEffectsDeclaredForExactScope",
                    "EffectCompleteness",
                ),
            ],
            &["title", "representation"],
            &[],
        )
        .extra("member($effect, MPConsentRequiredEffect)"),
        Card::new(
            "consented-effect",
            "PluralityConsentedEffectCompatibility",
            &[
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                ("$effect", "Effect"),
                (
                    "IndependentlyClassifiedExistentialEffectNotSelfLabelled",
                    "EffectClassification",
                ),
                (
                    "IndividualBodilyHousingChildRightsEcologicalCeilingsAndAnimalCore",
                    "EffectLimits",
                ),
                (
                    "NecessaryConsentNotSufficientCoerciveOrEnactmentAuthority",
                    "NonExecution",
                ),
            ],
            &["consent"],
            &["permits($operator, RelyOnConsentForExactCollectiveEffect, $record)"],
        )
        .extra("member($effect, MPConsentRequiredEffect)"),
        Card::new(
            "consultation",
            "PluralityPriorConsultation",
            &[
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                ("OtherMaterialNonexistentialEffect", "Effect"),
                (
                    "IndependentAllEffectsReviewNoConsentRequiredHarm",
                    "EffectClassification",
                ),
                (
                    "BeforeCommitmentGoodFaithAccessibleInformationAdequateTime",
                    "ConsultationTiming",
                ),
                (
                    "LawfulRepresentativesAndAffectedMembersParticipate",
                    "ConsultationParticipation",
                ),
                (
                    "AlternativesAccommodationMitigationBenefitsAndBurdensReviewed",
                    "Accommodation",
                ),
                (
                    "PublicReasonedResponseChallengeAndCorrection",
                    "ConsultationResponse",
                ),
                (
                    "NoBlanketVetoNoSubstituteForRequiredConsent",
                    "ConsultationLimits",
                ),
            ],
            &["title", "representation"],
            &[],
        ),
        Card::new(
            "consulted-effect",
            "PluralityConsultedEffectCompatibility",
            &[
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                ("OtherMaterialNonexistentialEffect", "Effect"),
                (
                    "IndividualRightsCommonsCeilingsAndAnimalProtectionIntact",
                    "EffectLimits",
                ),
            ],
            &["consultation"],
            &["permits($operator, RelyOnConsultationForExactCollectiveEffect, $record)"],
        ),
        Card::new(
            "evacuation",
            "PluralityEvacuationCompatibility",
            &[
                ("$project", "Project"),
                ("$project_version", "ProjectVersion"),
                ("$declaration", "EmergencyDeclaration"),
                ("$declaration_version", "EmergencyDeclarationVersion"),
                ("$hazard", "NamedHazard"),
                (
                    "TemporaryImminentLifeSavingNecessaryLeastRestrictive",
                    "EvacuationBasis",
                ),
                (
                    "SeparateMeasureJustificationPublishedReasonsAndCrossBranchAuthorization",
                    "EmergencyBasis",
                ),
                (
                    "CurrentMeasureWindowRejoinsExactDeclarationVersion",
                    "EmergencyTime",
                ),
                (
                    "IndependentJudicialReviewRestorationAndPostHocAudit",
                    "EmergencyReview",
                ),
                (
                    "SafeContinuityReturnRepairAndNoTitleTransferOrExtinguishment",
                    "EvacuationContinuity",
                ),
                (
                    "CompatibilityOnlyNoEmergencyDeclarationOrForcedMovementAuthority",
                    "NonExecution",
                ),
            ],
            &["title"],
            &["obliged($operator, PreserveTitleSafeContinuityReturnAndRepair, $record)"],
        ),
        Card::new(
            "private-access",
            "PluralityPrivateEvidenceAccess",
            &[
                ("$requester", "AccessRequester"),
                (
                    "CaseBoundEntitlementMinimizedPrivateEvidenceNoIdentityPublication",
                    "AccessBasis",
                ),
                (
                    "InspectChallengeOrCorrectExactMembershipTitleOrConsentRecord",
                    "AccessPurpose",
                ),
            ],
            &[],
            &["permits($requester, InspectExactPrivatePluralityEvidence, $record)"],
        ),
        Card::new(
            "remedy",
            "PluralityRightsBoundedRemedy",
            &[
                ("$relief", "ReliefKind"),
                (
                    "IndependentlyEstablishedContinuingWrongAndAffectedClaim",
                    "RemedyBasis",
                ),
                (
                    "IndividualVoiceNoCollectiveGuiltOtherRightsAndValidTitlePreserved",
                    "RemedyLimits",
                ),
                (
                    "ImmediateContinuityCessationProtectionAndNonRetaliation",
                    "RemedyContinuity",
                ),
                (
                    "RelatedCaseReviewAndBoundedStructuralNonRepetition",
                    "RemedySystemic",
                ),
            ],
            &[],
            &["obliged($operator, $relief, $record)"],
        )
        .extra("member($relief, MPReliefKind)"),
        Card::new(
            "external",
            "ExternalArrangementCompatibility",
            &[
                ("$instrument", "ExternalInstrument"),
                (
                    "ExactAttributablePublicPrivateDelegatedOrControlledArrangement",
                    "ExternalAttribution",
                ),
                (
                    "IndependentlyReviewedDomesticLabourEcologyAndRightsLimits",
                    "DomesticBaseline",
                ),
                (
                    "FullRelevantSupplyChainAndControlEvidenceNotCorporateLabel",
                    "ExternalEvidence",
                ),
                (
                    "NoExportOfProhibitedHarmNoForumFlagOrJurisdictionEvasion",
                    "ExternalCompliance",
                ),
                (
                    "ForeignCooperationAndEvidenceRemainNamedExternalAssumptions",
                    "ExternalAssumptions",
                ),
            ],
            &[],
            &["permits($operator, RelyOnExactExternalCompatibilityFinding, $record)"],
        )
        .extra("member($instrument, MPExternalInstrument)"),
        Card::new(
            "defect",
            "MobilityPluralityReviewedDefect",
            &[
                ("$target", "AffectedRecord"),
                ("$defect", "DefectKind"),
                (
                    "PositiveIndependentlyEstablishedScopedDefectNotMereAbsence",
                    "DefectBasis",
                ),
            ],
            &[],
            &[
                "contradict($target, MPEffectPermission)",
                "obliged($reader, PreserveCorrectAndRemedyMobilityPluralityRecord, $target)",
                "obliged($auditor, AuditRelatedCasesAndNonRepetition, $target)",
            ],
        )
        .extra("member($defect, MPDefectKind)"),
        Card::new(
            "nonresponse",
            "MobilityPluralityNonresponse",
            &[
                ("$target", "AffectedRequest"),
                ("$deadline", "ResponseDeadline"),
                (
                    "PositiveNoticeRealOpportunityAndSourceBoundDeadlineElapsed",
                    "NonresponseBasis",
                ),
                (
                    "IndependentAlternateForUnavailableConflictedOrNonrespondingReader",
                    "AlternateBasis",
                ),
            ],
            &[],
            &["obliged($alternate, ReviewMobilityPluralityRequestAndSecureRemedy, $target)"],
        ),
    ]
}
