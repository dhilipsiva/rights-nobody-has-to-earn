// SPDX-License-Identifier: MIT OR Apache-2.0

//! Distinct direct effects, not one security or emergency certificate.

#[derive(Clone, Debug)]
pub(super) struct Card {
    pub id: &'static str,
    pub kind: &'static str,
    pub fields: Vec<(&'static str, &'static str)>,
    pub dependencies: Vec<&'static str>,
    pub duties: Vec<&'static str>,
    pub capability: Option<&'static str>,
    pub movement: bool,
    pub holding: bool,
    pub extra: Vec<String>,
    pub external_bindings: Vec<(String, String)>,
    pub external: Vec<super::external::Link>,
}

impl Card {
    pub(super) fn new(
        id: &'static str,
        kind: &'static str,
        fields: &[(&'static str, &'static str)],
    ) -> Self {
        Self {
            id,
            kind,
            fields: fields.to_vec(),
            dependencies: vec![],
            duties: vec![],
            capability: None,
            movement: false,
            holding: false,
            extra: vec![],
            external_bindings: vec![],
            external: vec![],
        }
    }
    fn depends(mut self, ids: &[&'static str]) -> Self {
        self.dependencies.extend_from_slice(ids);
        self
    }
    pub(super) fn duties(mut self, duties: &[&'static str]) -> Self {
        self.duties.extend_from_slice(duties);
        self
    }
    fn restricts(mut self, capability: &'static str, movement: bool, holding: bool) -> Self {
        self.capability = Some(capability);
        self.movement = movement;
        self.holding = holding;
        self
    }
}

pub(super) const FLOOR: &[&str] = &[
    "secure",
    "suffice",
    "eats",
    "dwell",
    "healthy",
    "learn",
    "expresses",
    "believe",
    "meets",
];

// These are capabilities being limited, not inputs to a further consequence.
// The compiled graph guard covers both the capability and loss projections.
pub(super) const CAPABILITY_LEAVES: &[&str] = &["travel", "lose"];
pub(super) const HUMAN_SUBJECT: &str =
    "PositiveHumanSubjectIdentificationNoCivilIdentityGuiltOrLawfulnessClaim";

pub(super) const CORE_BARS: &[&str] = &[
    "Torture",
    "CruelInhumanOrDegradingTreatment",
    "EnforcedDisappearance",
    "SecretOrUnacknowledgedDetention",
    "ExtrajudicialArbitraryOrSummaryKilling",
    "CollectivePunishmentOrReprisal",
    "IndefiniteDetentionWithoutChargeOrReview",
    "CoercedConfessionOrUseOfProhibitedEvidence",
    "HumanShields",
    "DeliberateAttackOnNonparticipants",
    "StarvationOrFloorDenialAsWeaponSanctionOrInducement",
    "ExperimentationWithoutFreeConsent",
    "IndiscriminateOrSuperfluousInjuryWeapons",
    "AutonomousHumanTargetingWithoutMeaningfulHumanControl",
    "AggressiveWar",
    "RefoulementIncludingOnwardTransfer",
    "CollectiveExpulsion",
    "DenialOfPromptIndependentJudicialDetentionReview",
    "DenialOfEffectiveRemedy",
    "PreventingAssemblyOrConstitutionalCourtFromSitting",
];

pub(super) const ASSESSMENTS: &[&str] = &[
    "Risk",
    "Threat",
    "Loyalty",
    "Dangerousness",
    "Clearance",
    "Watchlist",
];

pub(super) const PRIORITY_KEYS: &[&str] = &[
    "Nationality",
    "Citizenship",
    "ImmigrationStatus",
    "Documentation",
    "MannerOfEntry",
    "LengthOfPresence",
];

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, holder, function, limit) in [
        (
            "policing",
            "PSPolicingMandate",
            "$holder",
            "PreventRespondInvestigateAndProtectPeople",
            "NoChargeAdjudicationLongTermCustodyIntelligenceOrSelfAudit",
        ),
        (
            "prosecution",
            "PSProsecutionMandate",
            "$holder",
            "IndependentReasonedChargingDecision",
            "NoOwnInvestigationAdjudicationExecutionOrFinalReview",
        ),
        (
            "adjudication",
            "PSAdjudicationMandate",
            "FSBOD_17",
            "IndependentOrdinaryCourtAdjudication",
            "NoInvestigationProsecutionExecutionOrFinalSelfReview",
        ),
        (
            "custodial-execution",
            "PSCustodialExecutionMandate",
            "$holder",
            "ExecuteOnlyAnExactCurrentJudicialHoldingOrder",
            "NoGroundDecisionRenewalOrOwnReview",
        ),
        (
            "external-defence",
            "PSExternalDefenceMandate",
            "$holder",
            "ExternalDefenceUnderCivilianCommand",
            "NoDomesticPolicingSurveillanceDetentionArrestSearchOrInterrogation",
        ),
        (
            "security-intelligence",
            "PSSecurityIntelligenceMandate",
            "$holder",
            "NarrowIndividualExternalOrOrganisedThreatInformation",
            "NoArrestSearchDetentionChargeAdjudicationCustodyOrCanonicalPersonWriting",
        ),
    ] {
        cards.push(
            Card::new(
                id,
                kind,
                &[
                    (holder, "MandateHolder"),
                    ("Class6ProtectiveFunction", "ConstitutionalClass"),
                    (function, "Function"),
                    (limit, "NonDelegableLimit"),
                    (
                        "DemocraticAppointmentAndSpecifiedLegalMandate",
                        "Appointment",
                    ),
                    ("CivilianDemocraticallySourcedCommand", "Command"),
                    (
                        "NoServingArmedOrIntelligencePoliticalJudicialElectoralOrOversightSeat",
                        "OfficeIncompatibility",
                    ),
                    (
                        "NoSecondmentJointCommandSharedStaffOrReserveFusion",
                        "AntiFusion",
                    ),
                    (
                        "NoMilitaryJurisdictionOverCivilians",
                        "CivilianJurisdiction",
                    ),
                    (
                        "NoPrivateCoercionMercenaryParamilitaryOrSecretLegalPower",
                        "PublicMonopolyLimit",
                    ),
                    (
                        "IndependentAppointmentBudgetStaffAccessPublicationAndActionDuty",
                        "Oversight",
                    ),
                    ("NoMandateIsAnIndividualCoerciveOrder", "OrderBoundary"),
                    (
                        "NoSecretLawCourtDetentionSiteOrCourtInaccessibleEngagementRule",
                        "PublicLaw",
                    ),
                    (
                        "OnlyNarrowClassifiedAnnexWithFullIndependentOversightAccess",
                        "BudgetSecrecy",
                    ),
                ],
            )
            .duties(&["MaintainSeparatedProtectiveFunctionAndIndependentOversight"]),
        );
    }
    cards.extend([
        Card::new(
            "civil-assistance",
            "PSUnarmedCivilAssistance",
            &[
                (
                    "IndividuallyAuthorisedPublicUnarmedCivilAssistance",
                    "Assistance",
                ),
                (
                    "RescueTransportEngineeringMedicalOrDisasterSupport",
                    "Purpose",
                ),
                (
                    "NoArrestSearchSeizureDetentionInterrogationCrowdControlOrSurveillance",
                    "Limits",
                ),
            ],
        )
        .depends(&["external-defence"])
        .duties(&["ProvideOnlyTheSpecifiedUnarmedCivilAssistance"]),
        Card::new(
            "arrest",
            "PSIndividualArrestOrder",
            &[
                (
                    "IndividualArticulableContemporaneouslyRecordedGround",
                    "Ground",
                ),
                (
                    "IdentifiedActorAndAuthorAccessibleContemporaneousReasons",
                    "Notice",
                ),
                ("NotifyPersonOfSubjectsChoosingWithoutDelay", "ChosenPerson"),
                (
                    "ImmediateConfidentialCounselInterpreterAndAccommodation",
                    "Assistance",
                ),
                (
                    "PromptAutomaticIndependentJudicialReviewNotOnRequest",
                    "AutomaticReview",
                ),
                (
                    "KnownPlaceNamedHolderAndIndependentAccessToHoldingRecord",
                    "HoldingRecord",
                ),
            ],
        )
        .depends(&["policing"])
        .restricts("FreeMovement", true, true)
        .duties(&[
            "NotifyChosenPersonAndProvideImmediateCounsel",
            "ArrangeAutomaticIndependentJudicialDetentionReview",
        ]),
        Card::new(
            "pretrial-detention",
            "PSPretrialDetentionOrder",
            &[
                (
                    "IndividualNecessityNotCategoryStatusNationalityOrAllegationSeverity",
                    "Necessity",
                ),
                (
                    "LibertyDefaultRealAlternativeConsideredAndInsufficient",
                    "Alternatives",
                ),
                (
                    "PriorJudicialAuthorityPromptAutomaticAndRepeatedIndependentReview",
                    "JudicialReview",
                ),
                (
                    "NoIncommunicadoUnrecordedUndisclosedOrIndefiniteHolding",
                    "HoldingLimits",
                ),
                (
                    "CounselInterpreterAccommodationChosenPersonNotificationAndAccessibleReasons",
                    "Assistance",
                ),
            ],
        )
        .depends(&["adjudication", "custodial-execution"])
        .restricts("FreeMovement", true, true)
        .duties(&[
            "ArrangeAutomaticIndependentJudicialDetentionReview",
            "PreserveEveryRightNotIncidentToTheExactHoldingGround",
        ]),
        Card::new(
            "search",
            "PSIndividualSearchOrder",
            &[
                (
                    "IndividualRecordedGroundForExactPersonPlaceCommunicationOrBelonging",
                    "Ground",
                ),
                ("PriorIndependentJudicialAuthorisation", "PriorAuthority"),
                ("SpecifiedNecessarySearchScopeAndMeans", "Intrusion"),
                (
                    "IdentifiedActorAccessibleReasonsCounselInterpreterAndAccommodation",
                    "Assistance",
                ),
            ],
        )
        .depends(&["policing", "adjudication"])
        .restricts("SpecifiedPrivacy", false, false),
        Card::new(
            "immediate-danger-search",
            "PSImmediateDangerSearchException",
            &[
                (
                    "PositiveImmediateDangerEvidenceMakesPriorAuthorisationImpossible",
                    "Exception",
                ),
                ("NarrowNecessaryIntrusionOnlyToAvertThatDanger", "Intrusion"),
                (
                    "ContemporaneousReasonsImmediateReportAndPromptIndependentJudicialReview",
                    "Review",
                ),
                (
                    "IdentifiedActorCounselInterpreterAccommodationAndAffectedPersonNotice",
                    "Assistance",
                ),
            ],
        )
        .depends(&["policing"])
        .restricts("SpecifiedPrivacy", false, false)
        .duties(&["ReportImmediateDangerExceptionForIndependentJudicialReview"]),
        Card::new(
            "seizure",
            "PSIndividualSeizureOrder",
            &[
                (
                    "IndividualRecordedGroundAndPriorIndependentJudicialAuthority",
                    "Ground",
                ),
                (
                    "ExactPropertyScopeNecessaryProportionateAndProtectedEssentialsPreserved",
                    "Property",
                ),
                (
                    "IdentifiedActorAccessibleReasonsCounselInterpreterAndAccommodation",
                    "Assistance",
                ),
                (
                    "InventoryPreservationAccountingReturnAndCompensationRoute",
                    "CustodyOfProperty",
                ),
            ],
        )
        .depends(&["policing", "adjudication"])
        .restricts("SpecifiedPropertyUse", false, false)
        .duties(&["InventoryPreserveAccountForAndReturnSeizedProperty"]),
        Card::new(
            "force-test",
            "PSPublicActorForceNecessityFinding",
            &[
                (
                    "LegitimateSpecifiedProtectivePurposeAndIndividualRecordedGround",
                    "Purpose",
                ),
                (
                    "StrictNecessityNoReasonablyAvailableLessHarmfulMeans",
                    "Necessity",
                ),
                (
                    "MinimumSufficientAndNotDisproportionateToPreventedHarm",
                    "Proportionality",
                ),
                (
                    "WarningWhereFeasibleAndCeaseWhenPurposeAchievedOrUnattainable",
                    "WarningAndEnd",
                ),
                (
                    "ImmediateMedicalAidIncludingToPersonSubjectedToForce",
                    "Aid",
                ),
                (
                    "PublicActorBearsLawfulnessBurdenNoAdverseInferenceAgainstHarmedPerson",
                    "Burden",
                ),
                ("NoCategoricallyRefusedActUnderAnyCondition", "Means"),
            ],
        ),
        Card::new(
            "force",
            "PSNonLethalForceOrder",
            &[
                ("SpecifiedNonLethalMeansOnly", "Means"),
                (
                    "ExactIndividualForceFindingAndLawfulPolicingMandate",
                    "Binding",
                ),
            ],
        )
        .depends(&["policing", "force-test"])
        .restricts("SpecifiedPhysicalAutonomy", false, false)
        .duties(&[
            "CeaseUnneededForceAndProvideImmediateMedicalAid",
            "PreserveForceRecordAndSubmitToIndependentReview",
        ]),
        Card::new(
            "lethal-force",
            "PSLifeProtectingLethalForceOrder",
            &[
                ("StrictlyUnavoidableToProtectLife", "LethalNecessity"),
                (
                    "NeverPropertyEscapeAloneAssemblyOrderEnforcementOrPunishment",
                    "LethalLimits",
                ),
                (
                    "MeaningfulHumanControlAndIndividualAccountability",
                    "HumanControl",
                ),
            ],
        )
        .depends(&["policing", "force-test"])
        .restricts("SpecifiedPhysicalAutonomy", false, false)
        .duties(&["ProvideImmediateAidAndIndependentDeathOrSeriousInjuryInvestigation"]),
        Card::new(
            "quarantine",
            "PSIndividualQuarantineOrder",
            &[
                (
                    "SpecificCommunicableHazardEvidenceNotPersonalRiskScore",
                    "Ground",
                ),
                (
                    "IndividualNecessaryLeastRestrictiveHealthProtection",
                    "Necessity",
                ),
                (
                    "IndependentHealthEvidenceAndJudicialChallengeWithRepeatedReview",
                    "Review",
                ),
                (
                    "CareSupportAccessibleContactAndNoPunitiveConsequence",
                    "Continuity",
                ),
            ],
        )
        .depends(&["adjudication"])
        .restricts("FreeMovement", true, true),
        Card::new(
            "exclusion",
            "PSIndividualProtectiveExclusionOrder",
            &[
                (
                    "EvidencedSpecificHarmNotIdentityHouseholdOrDangerousnessLabel",
                    "Ground",
                ),
                (
                    "NecessaryProportionateLeastRestrictiveSpecifiedPlaceAndConduct",
                    "Necessity",
                ),
                (
                    "IndependentJudicialReviewSurvivorSupportAndMeaningfulDefence",
                    "Review",
                ),
                (
                    "NoPunishmentConvictionPlacementOrRecognitionEffect",
                    "Separation",
                ),
            ],
        )
        .depends(&["adjudication"])
        .restricts("FreeMovement", true, false),
        Card::new(
            "border-hold",
            "PSIndividualAdultBorderHold",
            &[
                (
                    "IndividualNecessaryAdultImmigrationGroundNotConvenienceDeterrenceOrSignal",
                    "Ground",
                ),
                (
                    "GeneralLegalAdulthoodIndependentlyEstablishedNoChildImmigrationHolding",
                    "Age",
                ),
                (
                    "PriorJudicialAuthorityFixedMaximumAndPeriodicIndependentReview",
                    "Review",
                ),
                (
                    "RealNonCustodialAlternativeShownInsufficient",
                    "Alternative",
                ),
                (
                    "AccessibleAdvocateInterpreterReasonsAndSuspensiveAppeal",
                    "DueProcess",
                ),
                (
                    "AbsoluteNonRefoulementNoCollectiveExpulsionAndFloorThroughout",
                    "RemovalLimits",
                ),
            ],
        )
        .depends(&["adjudication", "custodial-execution"])
        .restricts("FreeMovement", true, true),
        Card::new(
            "pre-expulsion-detention",
            "PSIndividualAdultPreExpulsionDetention",
            &[
                (
                    "IndividualNecessaryAdultGroundPendingSpecifiedRemovalDecision",
                    "Ground",
                ),
                (
                    "GeneralLegalAdulthoodIndependentlyEstablishedNoChildImmigrationHolding",
                    "Age",
                ),
                (
                    "PriorJudicialAuthorityFixedMaximumAndPeriodicIndependentReview",
                    "Review",
                ),
                (
                    "RealNonCustodialAlternativeShownInsufficient",
                    "Alternative",
                ),
                (
                    "FairAsylumDeterminationAdvocateInterpreterAndSuspensiveAppeal",
                    "DueProcess",
                ),
                (
                    "AbsoluteNonRefoulementNoCollectiveExpulsionAndFloorThroughout",
                    "RemovalLimits",
                ),
            ],
        )
        .depends(&["adjudication", "custodial-execution"])
        .restricts("FreeMovement", true, true),
        Card::new(
            "surveillance",
            "PSIndividualSurveillanceOrder",
            &[
                (
                    "IndividualSpecificSeriousExternalOrOrganisedThreatEvidence",
                    "Ground",
                ),
                (
                    "PriorIndividualJudicialAuthorisationExactTargetMeansDataScopeAndDuration",
                    "Authority",
                ),
                (
                    "LeastIntrusiveEffectiveMeansForTheIndividuallyEvidencedSeriousHarm",
                    "Intrusion",
                ),
                (
                    "NoBulkSuspicionlessCollectionUnsuspectedPersonRetentionOrPurchasedPartneredForeignUnlawfulData",
                    "Collection",
                ),
                ("FreshIndividualAuthorisationForEveryRenewal", "Renewal"),
                (
                    "MinimisationRetentionDeletionAndLaterNotification",
                    "Privacy",
                ),
                (
                    "SecretEvidenceNeverSoleOrDecisiveNoConsequenceWhereRequiredMaterialCannotBeDisclosed",
                    "Evidence",
                ),
                (
                    "NoPermanentlyUndisclosableAuthorisationNotifyWhenLawfulPurposeNoLongerDefeated",
                    "Notification",
                ),
                (
                    "NoCanonicalPersonRecordOrStandingFloorFranchiseLibertyRemedyAllocationUse",
                    "AssessmentWall",
                ),
            ],
        )
        .depends(&["security-intelligence", "adjudication"])
        .restricts("SpecifiedCommunicationPrivacy", false, false),
    ]);
    cards.extend(emergency_cards());
    let mut evacuation = cards
        .iter()
        .find(|card| card.id == "hazard-restriction")
        .unwrap()
        .clone();
    evacuation.id = "hazard-evacuation";
    evacuation.kind = "PSCollectiveLifeSavingEvacuationMeasure";
    evacuation.fields.extend([
        ("$collective_holder", "CollectiveHolder"),
        ("$collective_title", "CollectiveTitle"),
        ("$evacuation_operation", "EvacuationOperation"),
        ("$operation_revision", "OperationRevision"),
        (
            "TemporaryLifeSavingOnlyReturnRepairAndNoTitleTransferOrExtinguishment",
            "CollectiveLimit",
        ),
    ]);
    cards.push(evacuation);
    cards.extend(external_cards());
    // Every initial measure is usable under its initial declaration; only the
    // renewed route adds a separately current renewal dependency.
    for card in &mut cards {
        if card.id == "policing" {
            card.fields.extend([
                ("NonCoercivePresenceInformationAssistanceMediationAndReferralDefault", "OrdinaryAct"),
                ("PreventHarmNotOrderAsSuchAndNoImmigrationFloorCareLearningOrCollectiveStatusGate", "PolicingLimit"),
            ]);
        }
        if card.id == "transfer" {
            card.fields.push(("$transfer_kind", "TransferKind"));
            card.extra.extend([
                "member($transfer_kind, MPTransferKind)".into(),
                "~($transfer_kind = IndividualExpulsion)".into(),
            ]);
        }
        if card.id == "external-measure" {
            card.fields
                .push(("$external_instrument", "ExternalInstrument"));
        }
        if matches!(card.id, "force-abroad" | "immediate-defence") {
            card.fields.push(("$attack_mode", "AttackBasisMode"));
        }
        if super::temporal::is_measure(card.id) {
            card.dependencies.retain(|id| *id != "renewal");
        }
        if matches!(
            card.id,
            "alternate-authorization" | "substitute-review" | "alternate-ratification"
        ) {
            card.fields.extend([
                ("$target", "TargetRecord"),
                ("$target_revision", "TargetRevision"),
                ("$target_kind", "TargetKind"),
                (
                    "PredeclaredExactTargetFunctionAndIdenticalLimitsIndependentlyRechecked",
                    "PredeclarationBinding",
                ),
            ]);
        }
    }
    let mut substitute = cards
        .iter()
        .find(|c| c.id == "alternate-ratification")
        .unwrap()
        .clone();
    substitute.id = "substitute-ratification";
    substitute.kind = "PSFirstOpportunitySubstituteRatification";
    substitute
        .fields
        .retain(|(_, s)| !matches!(*s, "OrdinaryAuthorizer" | "PredeclaredAuthorizer"));
    substitute.fields.extend([
        ("$ordinary", "OrdinaryReviewer"),
        ("$substitute", "PredeclaredReviewer"),
    ]);
    substitute.duties = vec!["PublishExactFirstOpportunitySubstituteReviewRatification"];
    cards.push(substitute);
    for card in &mut cards {
        if card.id == "exit-settlement" {
            card.fields.extend([
                ("$territory", "ExitTerritory"),
                ("$settlement", "Settlement"),
                ("$settlement_revision", "SettlementRevision"),
                (
                    "IndependentlyEstablishedNoConsentRequiredCollectiveImpact",
                    "CollectiveImpactDisposition",
                ),
            ]);
        }
    }
    let mut collective = cards
        .iter()
        .find(|c| c.id == "exit-settlement")
        .unwrap()
        .clone();
    collective.id = "exit-with-consent";
    collective.kind = "PSRightsContinuousExitWithActualCollectiveConsent";
    collective
        .fields
        .retain(|(_, s)| *s != "CollectiveImpactDisposition");
    collective.fields.extend([
        (
            "IndependentlyEstablishedConsentRequiredCollectiveImpact",
            "CollectiveImpactDisposition",
        ),
        ("$collective_effect", "CollectiveEffect"),
        ("$collective_holder", "CollectiveHolder"),
        ("$collective_title", "CollectiveTitle"),
        (
            "$collective_consent_record",
            "ActualCollectiveConsentRecord",
        ),
    ]);
    collective
        .extra
        .push("member($collective_effect, MPConsentRequiredEffect)".into());
    cards.push(collective);
    cards.extend([
        Card::new(
            "case-remedy",
            "PSActualCourtBoundCaseRemedy",
            &[
                ("$target", "AffectedProtectiveAct"),
                (
                    "IndividualAndSystemicRemedyWithoutAnyThirdPartyRightsLoss",
                    "RemedyLimits",
                ),
                (
                    "NoNewCoercivePowerAndNoClaimOfPaidOrPerformedRemedy",
                    "RemedyBoundary",
                ),
            ],
        )
        .duties(&["ImplementExactIndependentCourtBoundProtectiveCaseRemedy"]),
        Card::new(
            "general-remedy",
            "PSActualConstitutionalCourtGeneralRemedy",
            &[
                ("$target", "AffectedProtectiveAct"),
                (
                    "GeneralInvalidationOnlyThroughActualConstitutionalCourtPower",
                    "Finality",
                ),
            ],
        )
        .duties(&["GiveEffectToActualConstitutionalInvalidationWithinItsScope"]),
        Card::new(
            "composition-remedy",
            "PSUninvolvedProtectiveCompositionRemedy",
            &[
                ("$target", "ChallengedCourtComposition"),
                ("ChallengedCourtCannotReviewOwnComposition", "Independence"),
            ],
        )
        .duties(&["GiveEffectToUninvolvedPanelCompositionReview"]),
    ]);
    for card in cards.iter_mut().filter(|card| card.capability.is_some()) {
        card.fields.push((HUMAN_SUBJECT, "HumanSubjectFinding"));
    }
    cards.iter_mut().find(|card| card.id == "security-intelligence").unwrap()
        .fields.push((
            "EmploymentVettingOnlyForSpecifiedNecessaryFunctionWithReasonsPossibleDisclosureIndependentChallengeAndRemedyNeverFloor",
            "VettingLimit",
        ));
    cards
}

fn emergency_cards() -> Vec<Card> {
    vec![
        Card::new(
            "declaration",
            "PSNonDerogatingEmergencyDeclaration",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                (
                    "NamedHazardTerritoryAffectedPeopleAndSeparatelyJustifiedPowers",
                    "Declaration",
                ),
                (
                    "NarrowestSufficientScopeWithPublishedReasonsAndEvidence",
                    "PublicBasis",
                ),
                (
                    "CrossBranchAuthorisationNoBodyDeclaresAndExercisesAlone",
                    "CrossBranch",
                ),
                (
                    "ImmediateAssemblyAndConstitutionalCourtNotification",
                    "Notification",
                ),
                ("CurrentSourceBoundDeclarationWindow", "OwnWindow"),
                (
                    "NoSuspensionDecreeElectionDelayMandateExtensionOrFloorReduction",
                    "NonDerogation",
                ),
                (
                    "ReviewDeclarationAndEachMeasureNowNotOnlyAfterEnd",
                    "Review",
                ),
                (
                    "RationingOnlyThroughUnchangedFSPOW081FindingAndFSPOW082PhysicalScarcityAllocation",
                    "ScarcityBoundary",
                ),
                (
                    "ProviderContinuityOnlyNoNamedWorkerConscriptionPenaltyOrFloorWithdrawal",
                    "CompulsoryContinuityBoundary",
                ),
                (
                    "PriceControlRemainsOrdinaryEconomicLawNoDeclarationOrExecutiveDecreePower",
                    "PriceBoundary",
                ),
            ],
        )
        .duties(&["NotifyAssemblyAndConstitutionalCourtAndPreserveImmediateChallenge"]),
        Card::new(
            "renewal",
            "PSFreshEmergencyRenewal",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                (
                    "FreshSeparatelyAuthorisedSourceBoundRenewalNoAutomaticCarry",
                    "Renewal",
                ),
                ("CurrentIndependentlyReviewedNeedAndNarrowestScope", "Need"),
                ("OwnStartWindowEndAndExactDeclarationRevision", "OwnWindow"),
                (
                    "PublishedReasonsEvidenceAndCrossBranchRatification",
                    "PublicBasis",
                ),
            ],
        )
        .depends(&["declaration"]),
        Card::new(
            "acceleration",
            "PSProceduralAccelerationMeasure",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                (
                    "OnlySpecifiedShortenedStepsSittingParticipationOrTemporarySubstitution",
                    "Measure",
                ),
                (
                    "EverySubstantiveProtectionEvidenceRuleAndAppealPreserved",
                    "Limits",
                ),
            ],
        )
        .depends(&["declaration", "renewal"])
        .duties(&["PerformOnlyRightsPreservingProceduralAcceleration"]),
        Card::new(
            "resource-redirection",
            "PSPublicResourceRedirectionMeasure",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                (
                    "SpecifiedPublicCapacityFacilitiesAndPersonnelForNamedHazard",
                    "Measure",
                ),
                (
                    "NoNamedWorkerConscriptionNoIndividualWorkRefusalPenalty",
                    "LabourLimit",
                ),
                (
                    "PreserveFloorEssentialContinuityAndRecordUnmetDutiesAsFailures",
                    "Continuity",
                ),
            ],
        )
        .depends(&["declaration", "renewal"])
        .duties(&["RedirectOnlySpecifiedPublicResourcesWithRightsContinuity"]),
        Card::new(
            "requisition",
            "PSCompensatedRequisitionMeasure",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                ("SpecifiedPropertyNecessaryForNamedHazard", "Measure"),
                (
                    "ReasonsInventoryReturnOrCompensationAndEffectiveChallenge",
                    "Property",
                ),
                (
                    "NoFloorReductionAndNoTransferOfPublicDutyToNamedWorker",
                    "Limits",
                ),
            ],
        )
        .depends(&["declaration", "renewal"])
        .restricts("SpecifiedPropertyUse", false, false)
        .duties(&["InventoryAccountForReturnOrCompensateRequisitionedProperty"]),
        Card::new(
            "hazard-restriction",
            "PSIndividualHazardRestrictionMeasure",
            &[
                ("$hazard", "Hazard"),
                ("$population", "AffectedPopulation"),
                (
                    "IndividuallyNecessaryReviewableMovementAssemblyOrActivityRestriction",
                    "Measure",
                ),
                (
                    "ExactHazardNarrowestSufficientScopeOrdinaryGroundsAndReview",
                    "Limits",
                ),
                (
                    "NoPunitiveConsequenceOrOrdinaryDetentionBypass",
                    "Separation",
                ),
            ],
        )
        .depends(&["declaration", "renewal", "adjudication"])
        .restricts("FreeMovement", true, false),
        Card::new(
            "alternate-authorization",
            "PSPredeclaredAlternateAuthorization",
            &[
                ("$ordinary", "OrdinaryAuthorizer"),
                ("$substitute", "PredeclaredAuthorizer"),
                ("$opportunity", "FirstOpportunity"),
                ("PositiveEvidenceOrdinaryBodyUnableToAct", "Unavailability"),
                (
                    "IndependentPredeclarationNamingThisActorForThisExactFunction",
                    "Predeclaration",
                ),
                (
                    "IdenticalSubstantiveEvidentialScopeAndTemporalLimits",
                    "Limits",
                ),
                (
                    "ImmediateSubmissionForOrdinaryRatification",
                    "RatificationDuty",
                ),
                (
                    "NoFirstOpportunityYetPositivelyAttested",
                    "OpportunityStatus",
                ),
            ],
        )
        .duties(&["SubmitAlternateAuthorisationAtFirstOrdinaryOpportunity"]),
        Card::new(
            "substitute-review",
            "PSPredeclaredIndependentSubstituteReview",
            &[
                ("$ordinary", "OrdinaryReviewer"),
                ("$substitute", "PredeclaredReviewer"),
                ("$opportunity", "FirstOpportunity"),
                (
                    "PositiveEvidenceOrdinaryReviewerUnableToAct",
                    "Unavailability",
                ),
                (
                    "IndependentPredeclarationNamingThisReviewerForThisExactFunction",
                    "Predeclaration",
                ),
                (
                    "IdenticalIndependenceEvidenceScopeAndTemporalLimits",
                    "Limits",
                ),
                (
                    "ImmediateOrdinaryReviewAtFirstOpportunityNoApprovalBySilence",
                    "RatificationDuty",
                ),
                (
                    "NoFirstOpportunityYetPositivelyAttested",
                    "OpportunityStatus",
                ),
            ],
        )
        .duties(&["SubmitSubstituteReviewAtFirstOrdinaryOpportunity"]),
        Card::new(
            "alternate-ratification",
            "PSFirstOpportunityAlternateRatification",
            &[
                ("$ordinary", "OrdinaryAuthorizer"),
                ("$substitute", "PredeclaredAuthorizer"),
                ("$opportunity", "FirstOpportunity"),
                (
                    "PositiveFirstOrdinaryOpportunityAndActualRatification",
                    "Ratification",
                ),
                (
                    "ExactAlternateRecordRevisionAndSameUnreducedLimits",
                    "Binding",
                ),
            ],
        )
        .duties(&["PublishExactFirstOpportunityRatificationAndCurrentWindow"]),
        Card::new(
            "cessation",
            "PSRecordedProtectiveAuthorityEnd",
            &[
                ("$target", "AffectedOrder"),
                ("$target_revision", "AffectedRevision"),
                (
                    "PositiveRecordedCessationNotInferredFromMissingAuthority",
                    "EndAct",
                ),
                (
                    "RestoreArrangementsAuditEveryMeasureAccountAndRemedy",
                    "AfterEnd",
                ),
                ("EssentialContinuitySurvivesAuthorityEnd", "Continuity"),
            ],
        )
        .duties(&["RestoreArrangementsPreserveEvidenceAuditAndProvideRemedyAfterEnd"]),
    ]
}

fn external_cards() -> Vec<Card> {
    vec![
        Card::new("expulsion", "PSIndividualExpulsionOrder", &[
            ("IndividualJudicialDecisionAccessibleReasonsAndSuspensiveAppeal", "Decision"),
            ("ConnectionResidenceFamilyLifeAndAffectedChildPositionConsidered", "IndividualCircumstances"),
            ("NoRefoulementDirectOrOnwardNoCollectiveExpulsionNoStatelessness", "AbsoluteLimits"),
            ("FairAsylumDeterminationAdvocateInterpreterAndActualEffectiveAppeal", "Process"),
            ("NoPushbackExternalisedDutyEvasionOrJurisdictionShopping", "NoEvasion"),
        ]).depends(&["adjudication"]).restricts("FreeMovement", true, false),
        Card::new("transfer", "PSIndividualInternationalTransferOrder", &[
            ("IndividualJudicialDecisionAndSuspensiveAppeal", "Decision"),
            ("NoRefoulementCategoricalHarmOrConstitutionallyDeficientForeignProceeding", "AbsoluteLimits"),
            ("OffenceHereNoPoliticalCharacterisationOrIdentityBeliefAssociationExpressionPunishment", "Offence"),
            ("DiplomaticAssuranceWeakEvidenceNeverCureForBar", "Assurance"),
            ("HandingOverRemainsRepublicCoerciveActWithOrdinaryGroundsAndForceLimits", "ContinuingResponsibility"),
        ]).depends(&["adjudication", "custodial-execution"]).restricts("FreeMovement", true, true),
        Card::new("legal-assistance", "PSBoundedMutualLegalAssistance", &[
            ("IndividualLawfulPurposeScopeAndIndependentJudicialReview", "Purpose"),
            ("NoObtainingLaunderingReceivingOrTransmittingMaterialForForbiddenAct", "NoEvasion"),
            ("PrivacyPurposeMinimisationRetentionCorrectionAndEffectiveRemedy", "Privacy"),
        ]).depends(&["adjudication"]).duties(&["ProvideOnlyLawfulPurposeBoundInternationalLegalAssistance"]),
        Card::new("defence-structure", "PSAssemblyBoundedDefenceStructure", &[
            ("ActualAssemblyCeilingOnSizeArmamentAndOrganisation", "Ceiling"),
            ("OrdinaryLegislativeAppropriationAndFullIndependentAuditAccess", "Funding"),
            ("NoOutsideFundingArmingMaintainingOrDelegatedPrivateCoercion", "Limits"),
        ]).depends(&["external-defence"]).duties(&["KeepDefenceWithinAssemblyCeilingsAppropriationsAndAudit"]),
        Card::new("force-abroad", "PSAssemblyAuthorisedForceAbroad", &[
            ("DefenceAgainstActualOrImminentAttackOrLawfulCollectiveSecurityHumanitarianMandate", "Ground"),
            ("PriorActualAssemblyAuthorisation", "Assembly"),
            ("PublishedObjectiveLegalBasisScopeGeographyMeansDurationAndReporting", "PublicBasis"),
            ("NoAggressiveSecretWarOrDomesticMandate", "Limits"),
            ("OrdinaryForceNecessityHumanitarianDutiesAndCategoricalBarsIntact", "ForceLimits"),
        ]).depends(&["external-defence", "defence-structure", "force-test"]).restricts("SpecifiedPhysicalAutonomy", false, false),
        Card::new("immediate-defence", "PSImmediateResponseToActualArmedAttack", &[
            ("PositiveActualAttackImmediateDefensiveNecessity", "Ground"),
            ("PriorAssemblyActionImpossibleBeforeImmediateResponse", "Exception"),
            ("ImmediateSubmissionForRatificationCurrentPreOpportunityWindowOnly", "Ratification"),
            ("NoFirstOpportunityYetPositivelyAttested", "OpportunityStatus"),
            ("PublishedLegalBasisNoAggressionSameUnreducedForceAndHumanitarianLimits", "Limits"),
        ]).depends(&["external-defence", "defence-structure", "force-test"]).restricts("SpecifiedPhysicalAutonomy", false, false)
            .duties(&["SubmitImmediateDefensiveResponseForAssemblyRatificationWithoutDelay"]),
        Card::new("cyber-attack-basis", "PSIndependentlyEvidencedCyberAttackEquivalence", &[
            ("ScaleAndEffectsEquivalentToKineticArmedAttack", "Threshold"),
            ("EvidencedStatedIndependentlyAssessableAttribution", "Attribution"),
            ("NoCivilianLifeCriticalInfrastructureAttackOrSeparateResponsePower", "Limits"),
        ]).duties(&["SubmitAttackEquivalenceAndAttributionToOrdinaryAuthorisationReview"]),
        Card::new("arms-transfer", "PSReviewedArmsTransfer", &[
            ("EvidencedCapabilityAndRecipientUseTestAgainstEveryCategoricalBar", "Use"),
            ("IndependentReviewAndPublicReportingWithNarrowProtectedAnnex", "Review"),
            ("SuspendOnCredibleMisuseEvidence", "Misuse"),
            ("NoRecruitmentEnlistmentConscriptionOrHostilityUseBelowGeneralAdulthood", "Children"),
        ]).duties(&["MonitorAndSuspendArmsTransferOnCredibleMisuseEvidence"]),
        Card::new("treaty-entry", "PSConstitutionallyBoundedTreatyRatification", &[
            ("ExecutiveNegotiationActualAssemblyRatification", "Ratification"),
            ("RegionsCouncilAndEveryDirectlyAffectedRegionConsentWhereCompetenceOrBoundaryTouched", "RegionalConsent"),
            ("NoProvisionalApplicationPreemptingRatification", "ProvisionalApplication"),
            ("StandingFloorEqualityDueProcessCoreLibertiesCommonsAnimalCoreAndProtectiveCorridorIntact", "Supremacy"),
            ("RatifiersReviewersAndCourtsMustExamineActualEffect", "ReviewDuty"),
            ("NoEngineReadingOrCertificationOfActualTreatyTextEffect", "SemanticBoundary"),
        ]).duties(&["ApplyTreatySupremacyLimitWithoutTreatingDeclaredLabelAsActualEffect"]),
        Card::new("treaty-withdrawal", "PSConstitutionallyBoundedTreatyWithdrawal", &[
            ("SameExecutiveAssemblyAndApplicableRegionalConsentRouteAsEntry", "Withdrawal"),
            ("RightsContinuityNoPreemptiveProvisionalWithdrawalAndActualEffectReview", "Limits"),
            ("NoEngineReadingOrCertificationOfActualTreatyTextEffect", "SemanticBoundary"),
        ]).duties(&["ApplySameDemocraticConsentAndRightsLimitsToTreatyWithdrawal"]),
        Card::new("external-measure", "PSBoundedExternalTradeOrSanctionsMeasure", &[
            ("OrdinaryDemocraticLawExactMeasureAndIndependentReview", "Source"),
            ("NoPopulationFloorDenialOrHumanitarianAccessRestriction", "Limits"),
            ("NoTradeProcurementInvestmentAffiliateSupplyChainFlagArbitrationOrExportedEnforcementEvasion", "NoEvasion"),
        ]).duties(&["ApplyOrdinaryRightsBoundedExternalMeasuresWithoutEvasion"]),
        Card::new("exit-settlement", "PSRightsContinuousExternalExitSettlement", &[
            ("ActualCurrentInternalExitRouteFederalAgreementRightsReviewAndFinalPopulationRatification", "InternalRoute"),
            ("BordersAssetsDebtsCommonsFloorsAndMinorityProtections", "ExistingContents"),
            ("NationalityStatelessnessNonmovingResidentsFamilyKinCultureRecordsDefenceSecurityAndTreatySuccession", "AdditionalContents"),
            ("ActualPriorInformedCollectiveConsentWhereTitleSacredSiteOrCollectiveSovereigntyAffected", "CollectiveConsent"),
            ("NoMilitaryInstrumentForTerritorialIntegrityOrDefenceOverLawfullyExitedTerritory", "DefenceLimit"),
            ("NoWeaponisedTradeBorderPaymentsOrServicesAgainstLawfulExit", "Levers"),
            ("ForeignRecognitionCooperationAndReadmissionAreExternalAssumptions", "ExternalBoundary"),
        ]).duties(&["PreserveEveryPersonAndCollectiveRightAcrossLawfulExit"]),
        Card::new("common-competence", "PSExpressBoundedCommonCompetence", &[
            ("RepresentationTreatiesDefenceForceAbroadBordersAsylumRemovalLegalAssistanceTradeSanctionsArmsIntelligenceMinimaAndCrossBoundaryHazards", "Enumeration"),
            ("NoGeneralSecurityForeignAffairsPolicingOrEmergencyPower", "NoResidualCommonPower"),
            ("RegionalLocalPolicingCivilProtectionAndResidualCompetencePreserved", "Subsidiarity"),
            ("StrongerCompatibleSubnationalProtectionPermittedDisplacementOnlyAsJustified", "SubnationalProtection"),
        ]).duties(&["KeepCommonPowerEnumeratedAndPreserveRegionalResidualCompetence"]),
    ]
}
