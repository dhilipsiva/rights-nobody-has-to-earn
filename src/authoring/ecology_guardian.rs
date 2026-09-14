// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared replay identity and separately evidenced judicial decisions.

use super::{Card, records};

// Four replay dimensions; the authorization version includes its record ID.
// Ordinary fields carry that identity, never an opaque office-local key.

pub(super) fn cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for (id, kind, qualification) in [
        (
            "present-person-commons-claim",
            "ECPresentPersonCommonsClaim",
            "ECPositivePresentPersonClaimant",
        ),
        (
            "association-commons-claim",
            "ECAssociationCommonsClaim",
            "ECIndependentlyQualifiedAssociationAndRepresentativeMandate",
        ),
        (
            "rights-advocate-commons-claim",
            "ECRightsAdvocateCommonsClaim",
            "ECCurrentOrdinaryRightsAdvocateMandate",
        ),
        (
            "guardian-commons-claim",
            "ECGuardianCommonsClaim",
            "ECCurrentCollegialGuardianDecision",
        ),
    ] {
        let mut card = Card::new(id, kind, "Class4IndependentCommonsInitiation")
            .fields(&[
                (qualification, "CommonsInitiatorFinding"),
                ("$project", "ChallengedActivity"),
                ("$claim", "CommonsOrFutureConditionClaim"),
                (
                    "ECIndependentInitiationWithoutJointClaimantOrProprietaryInjury",
                    "IndependentInitiation",
                ),
                ("ECNoInventedUnbornPreferences", "FuturePreferenceLimit"),
                (
                    "ECInformationParticipationReasonsInterimReliefRequestAndRemedy",
                    "CommonsClaimRoute",
                ),
                (
                    "ECOrdinaryRequestIsNotAnAutomaticStay",
                    "OrdinaryStayBoundary",
                ),
            ])
            .effect("obliged($reader, ECReceiveAndIndependentlyReviewCommonsClaim, $record)");
        if id == "guardian-commons-claim" {
            card = card.depends(&["guardian-result"]);
        }
        cards.push(card);
    }
    let stay = Card::new(
        "guardian-stay",
        "ECGuardianAutomaticStay",
        "Class4TemporaryIrreversiblePartPause",
    )
    .concludes("interrupt($operator, $record, ECGuardianAutomaticStay)")
    .fields(&[
        ("$project", "ChallengedActivity"),
        ("$authorization_record", "ChallengedAuthorizationRecord"),
        ("$authorization_version", "AuthorizationVersion"),
        ("$ground", "StayGround"),
        ("$irreversible_part", "IrreversiblePart"),
        (
            "ECPublishedCollegialEvidenceSupportedSeriousOrIrreversibleObjection",
            "SupportedObjection",
        ),
        (
            "ECExactChallengedRecordIdentificationNotItsAssumedLawfulness",
            "ChallengedRecordBoundary",
        ),
        (
            "ECOnlyIrreversiblePartPausedWithEssentialFloorAndAnimalCareContinuity",
            "PauseScope",
        ),
        (
            "ECExpeditedIndependentMeritsReviewAndSeparateSubstitute",
            "ExpeditedReview",
        ),
        (
            "ECGuardianOwnFreshT3NotCustodyT3OrSilentExtension",
            "GuardianT3",
        ),
        (
            "ECNoPolicyVetoFinalMeritsBudgetProgrammeOrScientificOracle",
            "GuardianPowerLimit",
        ),
    ])
    .depends(&["guardian-result"])
    .extra("$operator = FSBOD_22")
    .extra("member($ground, ECGuardianStayGround)")
    .default("$operator", "FSBOD_22")
    .default("$ground", "ECIrreversibleHarmGround")
    .extra("~end($record, ECFinalGuardianStayReplay)")
    .extra("~end($record, ECFinallyResolvedAuthorizationEvidencePair)")
    .effect("oppose($authorization_record, $authorization_version, ECGuardianAutomaticStay)")
    .duty("ECPauseOnlyIrreversiblePartForExactSharedInitialWindow");
    let mut initial = stay
        .clone()
        .fields(&[(
            "ECIndependentFirstCaseFindingNotAnOfficeLocalUnusedFlag",
            "FirstCaseFinding",
        )])
        .extra("~end($record, ECPreviouslyResolvedGuardianCase)");
    registry(
        &mut initial,
        "ECIndependentFirstCaseWithNoEarlierFinalDisposition",
    );
    cards.push(initial);
    for (id, kind, changed, finding) in [
        (
            "guardian-material-authorization-stay",
            "ECGuardianMaterialAuthorizationStay",
            "$authorization_version",
            "ECIndependentlyEstablishedMaterialAuthorizationChange",
        ),
        (
            "guardian-material-evidence-stay",
            "ECGuardianMaterialEvidenceStay",
            "$evidence_version",
            "ECIndependentlyEstablishedMaterialEvidenceChange",
        ),
    ] {
        let mut renewal = stay.clone();
        renewal.id = id;
        renewal.kind = kind;
        renewal = renewal
            .fields(&[
                ("$prior_final_record", "PriorFinalDispositionRecord"),
                ("$prior_authorization_record", "PriorAuthorizationRecord"),
                ("$prior_authorization_version", "PriorAuthorizationVersion"),
                ("$prior_evidence_version", "PriorEvidenceVersion"),
                ("$prior_ground", "PriorStayGround"),
                ("$material_comparison", "MaterialNoveltyComparisonEvidence"),
                (finding, "MaterialNoveltyFinding"),
                (
                    "ECIndependentOldAndNewSubstantiveComparisonNotRelabeling",
                    "MaterialComparison",
                ),
                (
                    "ECNoNoveltyFromOfficeGroundWindowReviewerOrVersionLabelAlone",
                    "NoveltyLimit",
                ),
            ])
            .depends(&["guardian-merits"])
            .join("guardian-merits", "$record", "$prior_final_record")
            .join(
                "guardian-merits",
                "$authorization_record",
                "$prior_authorization_record",
            )
            .join(
                "guardian-merits",
                "$authorization_version",
                "$prior_authorization_version",
            )
            .join(
                "guardian-merits",
                "$evidence_version",
                "$prior_evidence_version",
            )
            .join("guardian-merits", "$ground", "$prior_ground")
            .extra(if changed == "$authorization_version" {
                "(~($authorization_record = $prior_authorization_record) | ~($authorization_version = $prior_authorization_version))"
            } else { "~($evidence_version = $prior_evidence_version)" });
        registry(&mut renewal, finding);
        cards.push(renewal);
    }
    cards.push(
        Card::new(
            "guardian-merits",
            "ECFinalGuardianMerits",
            "Class4FinalIndependentStayDisposition",
        )
        .fields(&[
            ("$project", "ChallengedActivity"),
            ("$authorization_record", "ChallengedAuthorizationRecord"),
            ("$authorization_version", "AuthorizationVersion"),
            ("$ground", "StayGround"),
            ("$outcome", "FinalMeritsOutcome"),
            (
                "$responsible_controller",
                "AdjudicatedResponsibleController",
            ),
            (
                "ECOutcomeDistinguishesProtectionDutiesFromRestrictionOfTheExactChallengedAct",
                "FinalOutcomeScope",
            ),
            (
                "ECFinalIndependentMeritsNotInitiatorAdviceOrInterimOrder",
                "FinalDisposition",
            ),
            (
                "ECSharedCaseAuthorizationEvidenceGroundClosureAcrossEveryOfficeAndReviewer",
                "SharedFinality",
            ),
            (
                "ECNoRestartByGroundOfficeLabelReviewerOrWindowChange",
                "NoReplayEvasion",
            ),
            (
                "ECOnlyMateriallyNewAuthorizationOrEvidenceMayOpenAnotherPair",
                "MaterialNovelty",
            ),
            (
                "ECAccessibleFinalReasonsCorrectionRemedyAndLawfulHistory",
                "FinalRemedy",
            ),
        ])
        .extra("$operator = FSBOD_17")
        .extra("member($ground, ECGuardianStayGround)")
        .extra("member($outcome, ECGuardianMeritsOutcome)")
        .default("$operator", "FSBOD_17")
        .default("$ground", "ECIrreversibleHarmGround")
        .default("$outcome", "ECMeritsRequireProtection")
        .effect("end($record, ECFinalGuardianStayDisposition)")
        .duty("ECPublishFinalDispositionAndSharedReplayClosureWithoutClaimingExecution"),
    );
    let mut substitute = cards
        .iter()
        .find(|card| card.id == "guardian-merits")
        .unwrap()
        .clone();
    substitute.id = "substitute-guardian-merits";
    substitute.kind = "ECSubstituteFinalGuardianMerits";
    for atom in &mut substitute.extra {
        if atom == "$operator = FSBOD_17" {
            *atom = "$operator = FSBOD_32".into();
        }
    }
    substitute = substitute.default("$operator", "FSBOD_32")
        .depends(&["substitute-reviewer-function"])
        .fields(&[
            ("$substitute_reviewer", "AssignedIndependentReviewer"),
            ("$initiating_advocate", "InitiatingAdvocate"),
            ("ECSeparateAssignedJudgeUsesActualOrdinaryCourtSafeguardsWithoutUnavailableReviewersApproval", "SubstituteJudicialBasis"),
            ("ECNoGuardianAlternateAdvocacyOrRegulatoryFunctionInFinalMerits", "SubstituteMeritsSeparation"),
        ]);
    cards.push(substitute);
    for (original, id, kind) in [
        (
            "guardian-stay",
            "alternate-guardian-stay",
            "ECAlternateGuardianAutomaticStay",
        ),
        (
            "guardian-material-authorization-stay",
            "alternate-material-authorization-stay",
            "ECAlternateMaterialAuthorizationStay",
        ),
        (
            "guardian-material-evidence-stay",
            "alternate-material-evidence-stay",
            "ECAlternateMaterialEvidenceStay",
        ),
    ] {
        let mut alternate = cards
            .iter()
            .find(|card| card.id == original)
            .unwrap()
            .clone();
        alternate.id = id;
        alternate.kind = kind;
        for dependency in &mut alternate.dependencies {
            if dependency.id == "guardian-result" {
                dependency.id = "guardian-alternate-function";
                dependency.key = "guardian-alternate-function";
            }
        }
        for atom in &mut alternate.extra {
            if atom == "$operator = FSBOD_22" {
                *atom = "$operator = FSBOD_31".into();
            }
        }
        alternate = alternate.default("$operator", "FSBOD_31").fields(&[(
            "ECPredeclaredAlternateActsOnlyInUnavailableGuardianPlaceWithSharedReplayLimits",
            "AlternateStayBoundary",
        )]);
        cards.push(alternate);
    }
    for (original, id, kind) in [
        (
            "guardian-material-authorization-stay",
            "guardian-authorization-after-substitute",
            "ECGuardianNewAuthorizationAfterSubstitute",
        ),
        (
            "guardian-material-evidence-stay",
            "guardian-evidence-after-substitute",
            "ECGuardianNewEvidenceAfterSubstitute",
        ),
        (
            "alternate-material-authorization-stay",
            "alternate-authorization-after-substitute",
            "ECAlternateNewAuthorizationAfterSubstitute",
        ),
        (
            "alternate-material-evidence-stay",
            "alternate-evidence-after-substitute",
            "ECAlternateNewEvidenceAfterSubstitute",
        ),
    ] {
        let mut renewal = cards
            .iter()
            .find(|card| card.id == original)
            .unwrap()
            .clone();
        renewal.id = id;
        renewal.kind = kind;
        for dependency in &mut renewal.dependencies {
            if dependency.id == "guardian-merits" {
                dependency.id = "substitute-guardian-merits";
                dependency.key = "substitute-guardian-merits";
            }
        }
        renewal.dependency_bindings = renewal
            .dependency_bindings
            .into_iter()
            .map(|((key, variable), value)| {
                (
                    (
                        if key == "guardian-merits" {
                            "substitute-guardian-merits".into()
                        } else {
                            key
                        },
                        variable,
                    ),
                    value,
                )
            })
            .collect();
        cards.push(renewal);
    }
    cards
}

fn registry(card: &mut Card, novelty: &'static str) {
    card.fields.extend([
        ("$registry", "SharedReplayRegistry"),
        ("$registry_source", "RegistryWriter"),
        ("$registry_review", "RegistryReviewer"),
        ("$registry_revision", "RegistryRevision"),
    ]);
    for (actor, role) in [
        ("$registry_source", "ECReplayRegistryAuthority"),
        ("$registry_review", "ECReplayReviewAuthority"),
    ] {
        card.extra
            .push(format!("authorized({actor}, {role}, $registry)"));
        for (value, scope) in registry_fields(novelty) {
            card.extra
                .push(records::observe(actor, "$registry", value, scope));
        }
    }
    card.extra
        .push("~($registry_source = $registry_review)".into());
    for actor in ["$registry_source", "$registry_review"] {
        for other in ["$operator", "$source", "$review"] {
            card.extra.push(format!("~({actor} = {other})"));
        }
    }
    card.extra
        .push("~related($registry, ECRecordAmbiguity)".into());
    card.extra
        .push("~contradict($registry, $registry_revision, ECRecordReliance)".into());
    card.extra
        .push("~end($record, ECConflictingReplayRegistry)".into());
}

fn registry_fields(novelty: &'static str) -> Vec<(&'static str, &'static str)> {
    vec![
        ("$case", "Case"),
        ("$authorization_record", "ChallengedAuthorizationRecord"),
        ("$authorization_version", "AuthorizationVersion"),
        ("$evidence_version", "EvidenceVersion"),
        ("$ground", "StayGround"),
        ("$registry_revision", "Revision"),
        ("$registry_revision", "SelectedRevision"),
        (
            "ECCurrentSharedReplayHistoryForThisExactPair",
            "RegistryDisposition",
        ),
        ("$window", "Window"),
        ("$start", "Start"),
        ("$end", "End"),
        ("ECSharedGuardianStayRegistry", "RegistryKind"),
        (
            "ECOneInitialWindowForThisAuthorizationEvidencePair",
            "InitialWindowFinding",
        ),
        (
            "ECPositivelyUnresolvedPairUnusedBeforeThisInitialWindow",
            "ReplayEligibility",
        ),
        (novelty, "RegistryNovelty"),
    ]
}

pub(super) fn candidate_key_atoms(ground: bool) -> Vec<String> {
    let mut fields = vec![
        ("$case", "Case"),
        ("$authorization_record", "ChallengedAuthorizationRecord"),
        ("$authorization_version", "AuthorizationVersion"),
        ("$evidence_version", "EvidenceVersion"),
    ];
    if ground {
        fields.push(("$ground", "StayGround"));
    }
    candidate_atoms(&fields)
}

fn candidate_atoms(fields: &[(&str, &str)]) -> Vec<String> {
    let mut atoms = vec![
        "authorized($candidate_source, ECSourceAuthority, $candidate)".into(),
        "authorized($candidate_review, ECIndependentReviewAuthority, $candidate)".into(),
        "~($candidate_source = $candidate_review)".into(),
    ];
    for actor in ["$candidate_source", "$candidate_review"] {
        for (value, scope) in fields {
            atoms.push(records::observe(actor, "$candidate", value, scope));
        }
    }
    atoms
}

pub(super) fn rules(cards: &[Card]) -> Vec<String> {
    let mut rules = vec![
        "derived_only(\"end\").".into(),
        "derived_only(\"interrupt\").".into(),
    ];
    for (scope, kind, values) in [
        (
            "StayGround",
            "ECGuardianStayGround",
            &["ECSeriousHarmGround", "ECIrreversibleHarmGround"][..],
        ),
        (
            "FinalMeritsOutcome",
            "ECGuardianMeritsOutcome",
            &[
                "ECMeritsRequireProtection",
                "ECMeritsUpholdAuthorization",
                "ECMeritsRestrictChallengedAct",
            ][..],
        ),
    ] {
        for value in values {
            rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, EC{scope}Scope) -> member({value}, {kind})."));
        }
    }
    rules.push("all $claimant: all $request: challenge($claimant, $request) & observe($claimant, $request, ECCommonsProtectionRequest, ECRequestKindScope) -> obliged(State, ECProvideCommonsInitiationReviewAndNonRetaliation, $request).".into());
    // Two independently corroborated active registry records cannot give the
    // same authorization/evidence pair different initial windows or grounds.
    for scope in ["Window", "Start", "End", "StayGround"] {
        let mut atoms = candidate_key_atoms(false);
        for side in ["a", "b"] {
            let record = format!("${side}_registry");
            for (suffix, role) in [
                ("source", "ECReplayRegistryAuthority"),
                ("review", "ECReplayReviewAuthority"),
            ] {
                let actor = format!("${side}_{suffix}");
                atoms.push(format!("authorized({actor}, {role}, {record})"));
                for (value, field) in [
                    ("$case", "Case"),
                    ("$authorization_record", "ChallengedAuthorizationRecord"),
                    ("$authorization_version", "AuthorizationVersion"),
                    ("$evidence_version", "EvidenceVersion"),
                    ("ECSharedGuardianStayRegistry", "RegistryKind"),
                    (
                        "ECPositivelyUnresolvedPairUnusedBeforeThisInitialWindow",
                        "ReplayEligibility",
                    ),
                ] {
                    atoms.push(records::observe(&actor, &record, value, field));
                }
                atoms.push(records::observe(
                    &actor,
                    &record,
                    &format!("${side}_value"),
                    scope,
                ));
            }
            atoms.push(format!("~(${side}_source = ${side}_review)"));
        }
        atoms.push("~($a_value = $b_value)".into());
        rules.push(records::rule(
            &atoms,
            "end($candidate, ECConflictingReplayRegistry)",
        ));
    }
    for final_card in cards.iter().filter(|card| {
        matches!(
            card.id,
            "guardian-merits" | "substitute-guardian-merits" | "guardian-final-history"
        )
    }) {
        let rename = |text: &str| {
            records::pattern()
                .replace_all(text, |m: &regex::Captures<'_>| {
                    if [
                        "$case",
                        "$authorization_record",
                        "$authorization_version",
                        "$evidence_version",
                        "$ground",
                    ]
                    .contains(&&m[0])
                    {
                        m[0].to_owned()
                    } else {
                        format!("$final_{}", m[0].trim_start_matches('$'))
                    }
                })
                .into_owned()
        };
        let mut final_atoms: Vec<_> = records::premises(cards, final_card)
            .iter()
            .map(|atom| rename(atom))
            .collect();
        final_atoms.push(rename(&records::heads(final_card)[0]));
        if !final_card.historical {
            let mut restriction = records::premises(cards, final_card);
            restriction.push(records::heads(final_card)[0].clone());
            restriction.push("$outcome = ECMeritsRestrictChallengedAct".into());
            rules.push(records::rule(
                &restriction,
                "oppose($authorization_record, $authorization_version, ECFinalJudicialProtection)",
            ));
        }
        for (mut atoms, marker) in [
            (candidate_key_atoms(true), "ECFinalGuardianStayReplay"),
            (
                candidate_key_atoms(false),
                "ECFinallyResolvedAuthorizationEvidencePair",
            ),
            (
                candidate_atoms(&[("$case", "Case")]),
                "ECPreviouslyResolvedGuardianCase",
            ),
        ] {
            atoms.extend(final_atoms.clone());
            rules.push(records::rule(&atoms, &format!("end($candidate, {marker})")));
        }
    }
    rules
}
