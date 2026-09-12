// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::cli::Error;
use crate::context::Context;
use regex::{Captures, Regex};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fmt::Write as _;
use std::sync::{Arc, OnceLock};

const CONSTITUTION_PATH: &str = "new-book-plans/constitution.nibli";

const SOURCE_PATH: &str = "new-book-plans/obligations-source.json";

const RULES_BEGIN: &str = "# <OBLIGATIONS-RULES-BEGIN>";

const RULES_END: &str = "# <OBLIGATIONS-RULES-END>";

const CURRENT_REVIEW_GUARD: &str = "~($source = $record_review)";

const MAIN_HEADER: &str = "# Obligations and non-reciprocity family - executable coverage pins";

const INDEPENDENCE_HEADER: &str =
    "# Counterfactual: obligations source writer serves as record reviewer";

const SOURCE_HEADER: &str =
    "# Counterfactual: the obligation-origin materialization rule is removed";

const READER_HEADER: &str = "# Counterfactual: the typed finding-reader bridge is removed";

const MODES: [&str; 3] = [
    "PublicObligationBearerMode",
    "DelegatedObligationBearerMode",
    "PrivateObligationBearerMode",
];

const CLASSES: [&str; 4] = [
    "PersonDutyClass",
    "ClassNineCommonDutyClass",
    "RoleDutyClass",
    "VoluntaryDutyClass",
];

const FINDING_KINDS: [&str; 14] = [
    "Placement",
    "Isolation",
    "StatusConflict",
    "CarryOmission",
    "CarryForgery",
    "ClearOmission",
    "ClearForgery",
    "StandingOmission",
    "RecordDisappearance",
    "MaturityDispute",
    "OrderConflict",
    "TemporalRecord",
    "TemporalAuthority",
    "TemporalDispute",
];

#[derive(Clone, Copy, Debug)]
struct Effect {
    number: u16,
    key: &'static str,
    title: &'static str,
    mode: &'static str,
    duty_class: &'static str,
    duty_kind: &'static str,
}

const EFFECTS: [Effect; 29] = [
    Effect {
        number: 198,
        key: "public-respect-duty",
        title: "Public respect duty",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PublicRespectDutyKind",
    },
    Effect {
        number: 199,
        key: "public-protect-duty",
        title: "Public protection duty",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PublicProtectDutyKind",
    },
    Effect {
        number: 200,
        key: "public-fulfil-duty",
        title: "Public fulfilment duty",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PublicFulfilDutyKind",
    },
    Effect {
        number: 201,
        key: "public-continuity-duty",
        title: "Public continuity duty",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PublicContinuityDutyKind",
    },
    Effect {
        number: 202,
        key: "public-remedy-duty",
        title: "Public remedy duty",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PublicRemedyDutyKind",
    },
    Effect {
        number: 203,
        key: "delegation-without-discharge",
        title: "Delegation without public discharge",
        mode: MODES[1],
        duty_class: CLASSES[0],
        duty_kind: "DelegatedPublicFunctionDutyKind",
    },
    Effect {
        number: 204,
        key: "express-private-duty-prerequisite",
        title: "Express private-duty prerequisite",
        mode: MODES[2],
        duty_class: CLASSES[0],
        duty_kind: "ExpressPrivateDutyKind",
    },
    Effect {
        number: 205,
        key: "no-subject-matter-private-duty",
        title: "No private duty from subject matter alone",
        mode: MODES[2],
        duty_class: CLASSES[0],
        duty_kind: "PrivateDutyBoundaryKind",
    },
    Effect {
        number: 206,
        key: "person-duty-continuity-restoration",
        title: "Person-duty continuity and restoration",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "PersonDutyRemedyKind",
    },
    Effect {
        number: 207,
        key: "class9-common-cessation-restoration",
        title: "Class 9 common cessation and restoration",
        mode: MODES[0],
        duty_class: CLASSES[1],
        duty_kind: "ClassNineCommonDutyRemedyKind",
    },
    Effect {
        number: 208,
        key: "role-duty-correction-reassignment",
        title: "Role-duty correction and reassignment",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "RoleDutyRemedyKind",
    },
    Effect {
        number: 209,
        key: "voluntary-duty-bounded-cure",
        title: "Bounded voluntary-duty cure",
        mode: MODES[2],
        duty_class: CLASSES[3],
        duty_kind: "VoluntaryDutyRemedyKind",
    },
    Effect {
        number: 210,
        key: "source-specific-conflict-priority",
        title: "Source-specific obligation conflict priority",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "DutyConflictDispositionKind",
    },
    Effect {
        number: 211,
        key: "bounded-performance-excuse",
        title: "Bounded performance excuse",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "BoundedExcuseKind",
    },
    Effect {
        number: 212,
        key: "all-entitlement-nonreciprocity",
        title: "All-entitlement non-reciprocity",
        mode: MODES[0],
        duty_class: CLASSES[0],
        duty_kind: "NonreciprocityWallKind",
    },
    Effect {
        number: 213,
        key: "finding-reader-action-duty",
        title: "Finding reader and action duty",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingReaderActionDutyKind",
    },
    Effect {
        number: 214,
        key: "certified-positive-nonresponse",
        title: "Certified positive nonresponse",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingNonresponseKind",
    },
    Effect {
        number: 215,
        key: "finding-alternate-escalation",
        title: "Finding alternate escalation",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingAlternateEscalationKind",
    },
    Effect {
        number: 216,
        key: "finding-claimant-continuity",
        title: "Finding claimant continuity",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingClaimantContinuityKind",
    },
    Effect {
        number: 217,
        key: "individual-remedy-prior-decision-review",
        title: "Individual remedy and prior-decision review",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingIndividualRemedyKind",
    },
    Effect {
        number: 218,
        key: "finding-common-cause-investigation",
        title: "Finding common-cause investigation",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingCommonCauseKind",
    },
    Effect {
        number: 219,
        key: "responsible-control-correction",
        title: "Responsible-control correction",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingControlCorrectionKind",
    },
    Effect {
        number: 220,
        key: "affected-case-reaudit",
        title: "Affected-case re-audit",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingAffectedCaseReauditKind",
    },
    Effect {
        number: 221,
        key: "recurrence-verification",
        title: "Recurrence verification",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingRecurrenceVerificationKind",
    },
    Effect {
        number: 222,
        key: "systemic-work-no-individual-delay",
        title: "Systemic work cannot delay individual relief",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "FindingIndividualReliefNonDelayKind",
    },
    Effect {
        number: 387,
        key: "disclosure-duty",
        title: "Disclosure duty",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "DisclosureDutyKind",
    },
    Effect {
        number: 388,
        key: "disclosure-reader-action-duty",
        title: "Disclosure reader and action duty",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "DisclosureReaderActionDutyKind",
    },
    Effect {
        number: 389,
        key: "certified-positive-disclosure-nonresponse",
        title: "Certified positive disclosure nonresponse",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "DisclosureNonresponseKind",
    },
    Effect {
        number: 390,
        key: "disclosure-alternate-escalation",
        title: "Disclosure alternate escalation",
        mode: MODES[0],
        duty_class: CLASSES[2],
        duty_kind: "DisclosureAlternateEscalationKind",
    },
];

#[derive(Clone, Debug)]
pub(crate) struct SourceSnapshot {
    protected_claim_refs: Arc<[String]>,
}

fn source_snapshot(context: &Context) -> Result<SourceSnapshot, Error> {
    #[derive(serde::Deserialize)]
    struct Source {
        protected_claim_refs: Vec<String>,
    }
    let source: Source = serde_json::from_str(&context.read(SOURCE_PATH)?)?;
    Ok(SourceSnapshot {
        protected_claim_refs: source.protected_claim_refs.into(),
    })
}

#[derive(Clone, Debug)]
struct ObligationError(String);

impl fmt::Display for ObligationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

type ObligationResult<T> = Result<T, ObligationError>;

fn obligation_error(message: impl Into<String>) -> ObligationError {
    ObligationError(message.into())
}

fn obligations_error(error: ObligationError) -> Error {
    Error::new(format!("obligations: {error}"))
}

fn variable_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\$[a-z][a-z0-9_]*").expect("valid variable regex"))
}

fn quantifier_prefix() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"^(?:all \$[a-z][a-z0-9_]*: )+").expect("valid quantifier regex")
    })
}

fn block<'a>(text: &'a str, begin: &str, end: &str) -> ObligationResult<&'a str> {
    if text.matches(begin).count() != 1 || text.matches(end).count() != 1 {
        return Err(obligation_error(format!(
            "expected one ordered marker pair: {begin}, {end}"
        )));
    }
    let start = text.find(begin).expect("count proved marker exists") + begin.len();
    let relative_stop = text[start..].find(end).ok_or_else(|| {
        obligation_error(format!("expected one ordered marker pair: {begin}, {end}"))
    })?;
    Ok(&text[start..start + relative_stop])
}

fn replace_block(text: &str, begin: &str, end: &str, payload: &str) -> ObligationResult<String> {
    let start = text
        .find(begin)
        .ok_or_else(|| obligation_error(format!("missing generated marker: {begin}")))?
        + begin.len();
    let stop = start
        + text[start..]
            .find(end)
            .ok_or_else(|| obligation_error(format!("missing generated marker: {end}")))?;
    Ok(format!(
        "{}\n{}\n{}",
        &text[..start],
        payload.trim_end(),
        &text[stop..]
    ))
}

fn ordered_variables(text: &str) -> Vec<&str> {
    let mut seen = HashSet::new();
    variable_pattern()
        .find_iter(text)
        .map(|matched| matched.as_str())
        .filter(|variable| seen.insert(*variable))
        .collect()
}

fn rule(atoms: &[String], head: &str) -> String {
    let body = atoms.join(" & ");
    let search = format!("{body} -> {head}");
    let mut quantified = String::new();
    for variable in ordered_variables(&search) {
        let _ = write!(quantified, "all {variable}: ");
    }
    format!("{quantified}{body} -> {head}.")
}

fn tri(atoms: &mut Vec<String>, subject: &str, value: &str, scope: &str) {
    atoms.push(format!("observe($source, {subject}, {value}, {scope})"));
    atoms.push(format!("observe($evidence, {subject}, {value}, {scope})"));
    atoms.push(format!("observe($review, {subject}, {value}, {scope})"));
}

const RAW_BINDINGS: [(&str, &str); 4] = [
    ("ObligationOriginScope", "ObligationOriginBinding"),
    ("SourceVersionScope", "ObligationVersionBinding"),
    ("JurisdictionScope", "ObligationJurisdictionBinding"),
    ("AuthorityScope", "ObligationScopeBinding"),
];

const ORIGIN_FIELD_SCOPES: [&str; 22] = [
    "DutyBearerScope",
    "DutyScope",
    "DutyStandardScope",
    "DutyBeneficiaryOrObjectScope",
    "DutyKindScope",
    "DutyFunctionOrCommitmentScope",
    "DutyBearerModeScope",
    "DutyClassScope",
    "DutyStartScope",
    "DutyEndScope",
    "ChallengeScope",
    "CorrectionScope",
    "RemedyScope",
    "DutyBreachScope",
    "DutyContinuityScope",
    "DutyPriorityScope",
    "DutyExcuseScope",
    "PublicPrincipalRetentionScope",
    "ExpressPrivateReachScope",
    "DutyNonWaiverScope",
    "FailurePolarityScope",
    "",
];

fn raw_current_atoms() -> Vec<String> {
    let mut atoms = vec![
        "authorized($source, ObligationsSourceAuthority, $record)".to_owned(),
        "authorized($temporal, ObligationsTemporalAuthority, $temporal_record)".to_owned(),
        "authorized($temporal_review, ObligationsTemporalReviewAuthority, $temporal_record)"
            .to_owned(),
        "authorized($record_review, ObligationsRecordReviewAuthority, $record)".to_owned(),
    ];
    for (authority, subject) in [
        ("$source", "$record"),
        ("$record_review", "$record"),
        ("$temporal", "$temporal_record"),
        ("$temporal_review", "$temporal_record"),
    ] {
        for (value, scope) in [
            ("Constitution_Obligations", "SourceFamilyScope"),
            ("$version", "SourceVersionScope"),
            ("$epoch", "SourceEpochScope"),
            ("$jurisdiction", "JurisdictionScope"),
            ("$legal_scope", "AuthorityScope"),
            ("$origin", "ObligationOriginScope"),
            ("$start", "DutyStartScope"),
            ("$end", "DutyEndScope"),
            ("ObligationsCurrentSelection", "EffectiveSelectionScope"),
            ("$reconciliation", "ReconciliationRecordScope"),
        ] {
            atoms.push(format!("observe({authority}, {subject}, {value}, {scope})"));
        }
    }
    atoms . extend (["observe($source, $record, $temporal_record, TemporalRecordScope)" . to_owned () , "observe($record_review, $record, $temporal_record, TemporalRecordScope)" . to_owned () , "observe($temporal, $temporal_record, $record, ObligationsRecordScope)" . to_owned () , "observe($temporal_review, $temporal_record, $record, ObligationsRecordScope)" . to_owned () , "observe($source, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)" . to_owned () , "observe($record_review, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)" . to_owned () , "observe($source, $reconciliation, $record, ObligationsRecordScope)" . to_owned () , "observe($record_review, $reconciliation, $record, ObligationsRecordScope)" . to_owned () , "observe($source, $reconciliation, $version, SourceVersionScope)" . to_owned () , "observe($record_review, $reconciliation, $version, SourceVersionScope)" . to_owned () , "~($source = $temporal)" . to_owned () , "~($source = $temporal_review)" . to_owned () , CURRENT_REVIEW_GUARD . to_owned () , "~($temporal = $temporal_review)" . to_owned () , "~($temporal = $record_review)" . to_owned () , "~($temporal_review = $record_review)" . to_owned () ,]) ;
    atoms.extend(
        RAW_BINDINGS
            .iter()
            .map(|(_, kind)| format!("~collide($record, {kind})")),
    );
    atoms
}

fn origin_contract_atoms() -> Vec<String> {
    let mut atoms = raw_current_atoms();
    atoms.extend([
        "authorized($evidence, ObligationsEvidenceAuthority, $record)".to_owned(),
        "authorized($review, IndependentObligationsReviewAuthority, $record)".to_owned(),
        "~($source = $evidence)".to_owned(),
        "~($source = $review)".to_owned(),
        "~($evidence = $review)".to_owned(),
    ]);
    for (value, scope) in [
        ("$record", "ObligationsRecordScope"),
        ("$version", "SourceVersionScope"),
        ("$epoch", "SourceEpochScope"),
        ("$jurisdiction", "JurisdictionScope"),
        ("$legal_scope", "AuthorityScope"),
        ("$temporal_record", "TemporalRecordScope"),
        ("$bearer", "DutyBearerScope"),
        ("$duty", "DutyScope"),
        ("$standard", "DutyStandardScope"),
        ("$target", "DutyBeneficiaryOrObjectScope"),
        ("$duty_kind", "DutyKindScope"),
        ("$function_or_commitment", "DutyFunctionOrCommitmentScope"),
        ("$bearer_mode", "DutyBearerModeScope"),
        ("$duty_class", "DutyClassScope"),
        ("$start", "DutyStartScope"),
        ("$end", "DutyEndScope"),
        ("$challenge_record", "ChallengeScope"),
        ("$correction_record", "CorrectionScope"),
        ("$remedy_record", "RemedyScope"),
        ("$breach_effect", "DutyBreachScope"),
        ("$continuity_effect", "DutyContinuityScope"),
        ("$priority_effect", "DutyPriorityScope"),
        ("$excuse_effect", "DutyExcuseScope"),
        ("$principal_retention", "PublicPrincipalRetentionScope"),
        ("$private_reach", "ExpressPrivateReachScope"),
        ("$non_waiver", "DutyNonWaiverScope"),
        ("$failure_polarity", "FailurePolarityScope"),
        ("$mode_certificate", "DutyBearerModeCertificateScope"),
        ("$class_certificate", "DutyClassCertificateScope"),
    ] {
        tri(&mut atoms, "$origin", value, scope);
    }
    atoms.push("~collide($origin, ObligationContractBinding)".to_owned());
    atoms
}

fn origin_join_atoms(effect: Effect) -> Vec<String> {
    let mut atoms = origin_contract_atoms();
    atoms.extend([
        "~collide($record, ObligationBearerModeBinding)".to_owned(),
        "~collide($record, ObligationClassBinding)".to_owned(),
    ]);
    tri(&mut atoms, "$origin", effect.mode, "DutyBearerModeScope");
    tri(&mut atoms, "$origin", effect.duty_class, "DutyClassScope");
    tri(&mut atoms, "$origin", effect.duty_kind, "DutyKindScope");
    tri(
        &mut atoms,
        "$record",
        &format!("FSCCE_{:03}", effect.number),
        "ObligationEffectScope",
    );
    tri(
        &mut atoms,
        "$effect_result",
        &format!("FSCCE_{:03}ObligationBranch", effect.number),
        "ObligationBranchScope",
    );
    tri(
        &mut atoms,
        "$effect_result",
        &format!("FSCCE_{:03}FailureWithholdsOnly", effect.number),
        "FailurePolarityScope",
    );
    if effect.mode == MODES[0] {
        atoms.push("public($bearer)".to_owned());
    }
    atoms
}

fn effect_extra_fields(effect: Effect, branch: &str) -> Vec<(&'static str, &'static str)> {
    match effect.number {
        198 => vec![(
            "RespectProtectedChoiceAndCondition",
            "PublicDutyEffectScope",
        )],
        199 => vec![(
            "ProtectAgainstExpressCoveredInterference",
            "PublicDutyEffectScope",
        )],
        200 => vec![("FulfilPositiveEntitlement", "PublicDutyEffectScope")],
        201 => vec![("ContinueProtectionAndProvision", "PublicDutyEffectScope")],
        202 => vec![("ReviewAndRemedyPublicBreach", "PublicDutyEffectScope")],
        203 => vec![
            ("$principal", "PublicPrincipalScope"),
            ("$function_or_commitment", "DelegatedPublicFunctionScope"),
            (
                "IdenticalPrincipalDutyAndFunction",
                "DelegatedDutyIdentityScope",
            ),
            (
                "PrincipalDutyContinuityAndRemedyRemain",
                "PublicPrincipalRetentionScope",
            ),
            (
                "DelegationCreatesNoPublicStatusOrAuthority",
                "DelegationBoundaryScope",
            ),
        ],
        204 => vec![
            ("ExpressPrivateDutyRecord", "PrivateDutySourceScope"),
            (
                "SubstantiveDutyAndClassCertificateRequired",
                "PrivateDutySubstanceScope",
            ),
        ],
        205 => vec![
            (
                "SubjectRelationshipDependencyOwnershipOrMarketAloneIsNoDuty",
                "PrivateDutyBoundaryScope",
            ),
            (
                "NoHorizontalDutyWithoutExpressSubstantiveSource",
                "PrivateDutySubstanceScope",
            ),
        ],
        206 => vec![
            ("PersonContinuity", "PersonDutyContinuityScope"),
            ("PersonReasonsAndReview", "PersonDutyReviewScope"),
            (
                "PersonRestorationAndIndividualRemedy",
                "PersonDutyRemedyScope",
            ),
        ],
        207 => vec![
            (
                "CommonCessationProtectionAndRestoration",
                "CommonDutyRemedyScope",
            ),
            (
                "CommonAccountingAndRecurrenceReview",
                "CommonDutyAccountingScope",
            ),
            (
                "IndividualHarmsRemainSeparatelyRemediable",
                "CommonIndividualRemedyScope",
            ),
        ],
        208 => vec![
            (
                "RoleRecusalOrReassignmentAndCorrection",
                "RoleDutyCorrectionScope",
            ),
            (
                "RoleAccountingAffectedCaseReviewAndContinuity",
                "RoleDutyReviewScope",
            ),
            ("NoAutomaticPunishmentOrStatusLoss", "RoleDutyBoundaryScope"),
        ],
        209 => vec![
            (
                "LawfulPerformanceUnwindingRestitutionCompensationOrExit",
                "VoluntaryDutyCureScope",
            ),
            (
                "NoIndefinitePersonalServiceFloorLossOrNonwaivableWaiver",
                "VoluntaryDutyBoundaryScope",
            ),
        ],
        210 => vec![
            ("$conflict", "PositiveDutyConflictScope"),
            ("$preferred_duty", "PreferredDutyScope"),
            ("$preferred_standard", "PreferredDutyStandardScope"),
            ("$deferred_duty", "DeferredDutyScope"),
            ("$deferred_standard", "DeferredDutyStandardScope"),
            ("$deferred_origin", "DeferredDutyOriginScope"),
            ("$deferred_version", "DeferredSourceVersionScope"),
            ("$deferred_jurisdiction", "DeferredJurisdictionScope"),
            ("$deferred_legal_scope", "DeferredAuthorityScope"),
            ("DutyClassForbiddenAsPriority", "DutyPriorityBoundaryScope"),
            ("RightsAndContinuityFirst", "DutyPriorityBoundaryScope"),
            ("BothOriginalDutiesRemain", "DutyConflictRetentionScope"),
            (
                "DeferredPerformanceRecordedForRepair",
                "DutyConflictRepairScope",
            ),
            (
                if branch == "priority" {
                    "SourceSpecificPairPriority"
                } else {
                    "CertifiedTieOrMissingPriority"
                },
                "DutyPriorityDispositionScope",
            ),
            (
                if branch == "priority" {
                    "NoLotteryOrRotation"
                } else {
                    "IndivisibleMateriallyEqualChoiceOnly"
                },
                "DutyTieBoundaryScope",
            ),
        ],
        211 => vec![
            ("$impediment", "SourceEnumeratedImpedimentScope"),
            ("$performance_slice", "AffectedPerformanceSliceScope"),
            ("$onset", "ExcuseOnsetScope"),
            ("$excuse_end", "FiniteExcuseEndScope"),
            ("$excuse_review_event", "ExcuseReviewEventScope"),
            ("$notice", "ExcuseNoticeScope"),
            ("$alternate", "AlternateDutyBearerScope"),
            ("$alternate_duty", "AlternateDutyScope"),
            ("$alternate_standard", "AlternateDutyStandardScope"),
            (
                "OriginalDutyAccountabilityClaimBreachPrincipalAndRemediesRemain",
                "ExcuseRetentionScope",
            ),
            (
                "SilenceConvenienceRefusalSelfCreatedOrConflictingEvidenceIsNoExcuse",
                "ExcuseBoundaryScope",
            ),
        ],
        212 => vec![
            ("$failure_kind", "DutyPerformanceOrFailureKindScope"),
            ("$protected_effect", "ProtectedEntitlementEffectScope"),
            ("NoDutyPerformanceOrFailureGate", "NonreciprocityScope"),
            (
                "SeparateConsequenceNeedsOwnSourceEvidenceProcessReviewAndRemedy",
                "SeparateConsequenceScope",
            ),
        ],
        213 => vec![
            ("$receipt", "FindingReceiptEvidenceScope"),
            ("$permitted_action", "FindingPermittedActionScope"),
            ("$reasons", "FindingReasonsScope"),
            ("$action_review", "FindingActionReviewScope"),
            ("$reader_duty", "FindingReaderDutyScope"),
            ("$reader_standard", "FindingReaderStandardScope"),
        ],
        214 => vec![
            ("$nonresponse", "PositiveFindingNonresponseScope"),
            (
                "CertifiedPositiveNonresponse",
                "NonresponseDispositionScope",
            ),
            ("SilenceIsNoAction", "NonresponseBoundaryScope"),
        ],
        215 => vec![
            ("$nonresponse", "PositiveFindingNonresponseScope"),
            (
                "CertifiedPositiveNonresponse",
                "NonresponseDispositionScope",
            ),
            ("SilenceIsNoAction", "NonresponseBoundaryScope"),
            ("$alternate", "AlternateFindingReaderScope"),
            ("$alternate_duty", "AlternateFindingDutyScope"),
            ("$alternate_standard", "AlternateFindingStandardScope"),
            ("PredeclaredAlternateEscalation", "FindingEscalationScope"),
        ],
        216 => vec![
            ("$continuity_bearer", "FindingContinuityBearerScope"),
            ("$continuity_duty", "FindingContinuityDutyScope"),
            ("$continuity_standard", "FindingContinuityStandardScope"),
        ],
        217 => vec![
            ("$remedy_bearer", "FindingRemedyBearerScope"),
            ("$individual_remedy_duty", "FindingIndividualRemedyScope"),
            (
                "$individual_remedy_standard",
                "FindingIndividualRemedyStandardScope",
            ),
            ("$prior_review_bearer", "PriorDecisionReviewBearerScope"),
            ("$prior_review_duty", "PriorDecisionReviewDutyScope"),
            ("$prior_review_standard", "PriorDecisionReviewStandardScope"),
        ],
        218 => vec![
            ("$investigation_bearer", "CommonCauseBearerScope"),
            ("$common_cause_duty", "CommonCauseInvestigationScope"),
            ("$common_cause_standard", "CommonCauseStandardScope"),
        ],
        219 => vec![
            ("$correction_bearer", "ResponsibleControlBearerScope"),
            (
                "$control_correction_duty",
                "ResponsibleControlCorrectionScope",
            ),
            (
                "$control_correction_standard",
                "ResponsibleControlStandardScope",
            ),
        ],
        220 => vec![
            ("$reaudit_bearer", "AffectedCaseReauditBearerScope"),
            ("$reaudit_duty", "AffectedCaseReauditScope"),
            ("$reaudit_standard", "AffectedCaseReauditStandardScope"),
        ],
        221 => vec![
            ("$recurrence_bearer", "RecurrenceVerificationBearerScope"),
            ("$recurrence_duty", "RecurrenceVerificationScope"),
            (
                "$recurrence_standard",
                "RecurrenceVerificationStandardScope",
            ),
        ],
        222 => vec![
            ("$individual_relief_bearer", "IndividualReliefBearerScope"),
            ("$individual_relief_duty", "IndividualReliefDutyScope"),
            (
                "$individual_relief_standard",
                "IndividualReliefStandardScope",
            ),
            (
                "SystemicWorkCannotDelayIndividualContinuityOrRemedy",
                "IndividualReliefNonDelayScope",
            ),
        ],
        387 => vec![
            ("$disclosure_bearer", "DisclosureBearerScope"),
            ("$disclosure_duty", "DisclosureDutyScope"),
            ("$disclosure_standard", "DisclosureStandardScope"),
            ("$disclosure_kind", "DisclosureKindScope"),
        ],
        388 => vec![
            ("$disclosure_reader", "DisclosureReaderScope"),
            ("$disclosure_reader_duty", "DisclosureReaderDutyScope"),
            (
                "$disclosure_reader_standard",
                "DisclosureReaderStandardScope",
            ),
            ("$disclosure_receipt", "DisclosureReceiptEvidenceScope"),
        ],
        389 => vec![
            (
                "$disclosure_nonresponse",
                "PositiveDisclosureNonresponseScope",
            ),
            (
                "$disclosure_nonresponse_duty",
                "DisclosureNonresponseDutyScope",
            ),
            (
                "$disclosure_nonresponse_standard",
                "DisclosureNonresponseStandardScope",
            ),
        ],
        390 => vec![
            ("$disclosure_alternate", "DisclosureAlternateScope"),
            (
                "$disclosure_alternate_duty",
                "DisclosureEscalationDutyScope",
            ),
            (
                "$disclosure_alternate_standard",
                "DisclosureEscalationStandardScope",
            ),
        ],
        _ => unreachable!("known effect"),
    }
}

fn route_contract_atoms() -> Vec<String> {
    let mut atoms = vec!["err($finding_subject, $finding_kind)".to_owned()];
    for (value, scope) in [
        ("$finding_kind", "FindingKindScope"),
        ("$finding_subject", "FindingSubjectScope"),
        ("$subject", "FindingAffectedSubjectScope"),
        ("$case", "FindingCaseScope"),
        ("$reader", "FindingReaderScope"),
        ("$route", "FindingRouteScope"),
    ] {
        tri(&mut atoms, "$route", value, scope);
    }
    atoms.extend([
        "~($reader = $source)".to_owned(),
        "~($reader = $evidence)".to_owned(),
        "~($reader = $review)".to_owned(),
    ]);
    atoms
}

fn effect_atoms(effect: Effect, branch: &str) -> Vec<String> {
    let mut atoms = origin_join_atoms(effect);
    if effect.number >= 213 {
        atoms.extend(route_contract_atoms());
    }
    for (value, scope) in effect_extra_fields(effect, branch) {
        tri(&mut atoms, "$effect_result", value, scope);
    }
    if effect.number == 203 {
        atoms.extend([
            "public($principal)".to_owned(),
            "~($bearer = $principal)".to_owned(),
        ]);
    }
    if effect.number == 210 {
        atoms.push("~collide($conflict, ObligationPriorityBinding)".to_owned());
    }
    if effect.number == 211 {
        atoms.extend([
            "~($bearer = $source)".to_owned(),
            "~($bearer = $evidence)".to_owned(),
            "~($bearer = $review)".to_owned(),
            "~collide($effect_result, ObligationExcuseBinding)".to_owned(),
        ]);
    }
    atoms
}

fn collision_rules() -> Vec<String> {
    let mut rules = Vec::new();
    for (scope, kind) in RAW_BINDINGS {
        rules.push(rule(
            &[
                "authorized($source, ObligationsSourceAuthority, $record)".to_owned(),
                format!("observe($source, $record, $first, {scope})"),
                format!("observe($source, $record, $second, {scope})"),
                "~($first = $second)".to_owned(),
            ],
            &format!("collide($record, {kind})"),
        ));
    }
    for scope in ORIGIN_FIELD_SCOPES
        .into_iter()
        .filter(|scope| !scope.is_empty())
    {
        rules.push(rule(
            &[
                format!("observe($source, $origin, $first, {scope})"),
                format!("observe($source, $origin, $second, {scope})"),
                "~($first = $second)".to_owned(),
            ],
            "collide($origin, ObligationContractBinding)",
        ));
    }
    rules.push(rule(
        &[
            "authorized($source, ObligationsSourceAuthority, $record)".to_owned(),
            "observe($source, $first_origin, $first_certificate, DutyBearerModeCertificateScope)"
                .to_owned(),
            "observe($source, $first_origin, $first_mode, DutyBearerModeScope)".to_owned(),
            "observe($source, $second_origin, $second_certificate, DutyBearerModeCertificateScope)"
                .to_owned(),
            "observe($source, $second_origin, $second_mode, DutyBearerModeScope)".to_owned(),
            "~($first_mode = $second_mode)".to_owned(),
        ],
        "collide($record, ObligationBearerModeBinding)",
    ));
    rules.push(rule(
        &[
            "authorized($source, ObligationsSourceAuthority, $record)".to_owned(),
            "observe($source, $first_origin, $first_certificate, DutyClassCertificateScope)"
                .to_owned(),
            "observe($source, $first_origin, $first_class, DutyClassScope)".to_owned(),
            "observe($source, $second_origin, $second_certificate, DutyClassCertificateScope)"
                .to_owned(),
            "observe($source, $second_origin, $second_class, DutyClassScope)".to_owned(),
            "~($first_class = $second_class)".to_owned(),
        ],
        "collide($record, ObligationClassBinding)",
    ));
    rules.push(rule(
        &[
            "observe($source, $conflict, $first, DutyPriorityDispositionScope)".to_owned(),
            "observe($source, $conflict, $second, DutyPriorityDispositionScope)".to_owned(),
            "~($first = $second)".to_owned(),
        ],
        "collide($conflict, ObligationPriorityBinding)",
    ));
    rules.push(rule(
        &[
            "observe($source, $effect_result, $first, ExcuseReviewEventScope)".to_owned(),
            "observe($source, $effect_result, $second, ExcuseReviewEventScope)".to_owned(),
            "~($first = $second)".to_owned(),
        ],
        "collide($effect_result, ObligationExcuseBinding)",
    ));
    rules
}

fn finding_route_rules() -> &'static [String] {
    static RULES: OnceLock<Vec<String>> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut direct = origin_join_atoms(effect_by_number(213));
        direct.extend(
            route_contract_atoms()
                .into_iter()
                .map(|atom| atom.replace("$finding_kind", "TemporalRecord")),
        );
        tri(&mut direct, "$route", "TemporalRecord", "FindingKindScope");
        direct.push("observe($source, $route, Appeals, FindingReaderScope)".to_owned());
        vec![rule(&direct, "obliged(Appeals, $subject)")]
    })
}

fn effect_heads(effect: Effect) -> &'static [&'static str] {
    match effect.number {
        198..=202 | 204 | 206..=209 => &["obliged($bearer, $duty, $standard)"],
        203 => &[
            "obliged($bearer, $duty, $standard)",
            "obliged($principal, $duty, $standard)",
        ],
        205 => &["prevents($bearer, SubjectMatterOnlyPrivateDutyInference)"],
        210 => &[
            "obliged($bearer, $preferred_duty, $preferred_standard)",
            "obliged($bearer, $deferred_duty, $deferred_standard)",
            "prevents($effect_result, PriorityDischargesOriginalDuty)",
        ],
        211 => &[
            "obliged($bearer, $duty, $standard)",
            "obliged($alternate, $alternate_duty, $alternate_standard)",
        ],
        212 => &["prevents($target, ObligationFailureEntitlementGate)"],
        213 => &[],
        214 => &["prevents($nonresponse, SilenceAsFindingAction)"],
        215 => &["obliged($alternate, $alternate_duty, $alternate_standard)"],
        216 => &["obliged($continuity_bearer, $continuity_duty, $continuity_standard)"],
        217 => &[
            "obliged($remedy_bearer, $individual_remedy_duty, $individual_remedy_standard)",
            "obliged($prior_review_bearer, $prior_review_duty, $prior_review_standard)",
        ],
        218 => &["obliged($investigation_bearer, $common_cause_duty, $common_cause_standard)"],
        219 => {
            &["obliged($correction_bearer, $control_correction_duty, $control_correction_standard)"]
        }
        220 => &["obliged($reaudit_bearer, $reaudit_duty, $reaudit_standard)"],
        221 => &["obliged($recurrence_bearer, $recurrence_duty, $recurrence_standard)"],
        222 => &[
            "obliged($individual_relief_bearer, $individual_relief_duty, $individual_relief_standard)",
            "prevents($subject, SystemicWorkDelaysIndividualRelief)",
        ],
        387 => &["obliged($disclosure_bearer, $disclosure_duty, $disclosure_standard)"],
        388 => {
            &["obliged($disclosure_reader, $disclosure_reader_duty, $disclosure_reader_standard)"]
        }
        389 => &[
            "obliged($disclosure_nonresponse, $disclosure_nonresponse_duty, $disclosure_nonresponse_standard)",
            "prevents($disclosure_nonresponse, SilenceAsDisclosureAction)",
        ],
        390 => &[
            "obliged($disclosure_alternate, $disclosure_alternate_duty, $disclosure_alternate_standard)",
        ],
        _ => unreachable!("known effect"),
    }
}

fn generate_effect_conclusion_rules(effect: Effect) -> Vec<String> {
    if effect.number == 213 {
        return Vec::new();
    }
    let branches: &[&str] = if effect.number == 210 {
        &["priority", "tie"]
    } else {
        &["standard"]
    };
    let mut rules = Vec::new();
    for branch in branches {
        let atoms = effect_atoms(effect, branch);
        for head in effect_heads(effect) {
            rules.push(rule(&atoms, head));
        }
    }
    rules
}

fn effect_rule_sets() -> &'static [Vec<String>] {
    static RULES: OnceLock<Vec<Vec<String>>> = OnceLock::new();
    RULES.get_or_init(|| {
        EFFECTS
            .iter()
            .copied()
            .map(generate_effect_conclusion_rules)
            .collect()
    })
}

#[doc = " Index by position in `EFFECTS`, never by `number - 198`. The rule sets are"]
#[doc = " built by iterating `EFFECTS` in order, so the two coincide only while the"]
#[doc = " numbers stay contiguous from 198 — an undocumented invariant that the"]
#[doc = " disclosure effects (387+) break. Position is what the data actually means."]
fn effect_index(number: u16) -> usize {
    EFFECTS
        .iter()
        .position(|effect| effect.number == number)
        .expect("effect number is declared in EFFECTS")
}

fn effect_conclusion_rules(effect: Effect) -> &'static [String] {
    &effect_rule_sets()[effect_index(effect.number)]
}

fn typed_reader_bridge() -> &'static str {
    static RULE: OnceLock<String> = OnceLock::new();
    RULE.get_or_init(|| {
        let mut atoms = effect_atoms(effect_by_number(213), "standard");
        atoms.push("obliged($reader, $subject)".to_owned());
        rule(&atoms, "obliged($reader, $reader_duty, $reader_standard)")
    })
}

fn formal_rules() -> &'static [String] {
    static RULES: OnceLock<Vec<String>> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut rules = collision_rules();
        rules.extend(finding_route_rules().iter().cloned());
        for effect in EFFECTS {
            rules.extend(effect_conclusion_rules(effect).iter().cloned());
        }
        rules.push(typed_reader_bridge().to_owned());
        let unique: HashSet<_> = rules.iter().collect();
        assert_eq!(
            rules.len(),
            unique.len(),
            "generated obligations rules are not unique"
        );
        rules
    })
}

fn effect_by_number(number: u16) -> Effect {
    EFFECTS[effect_index(number)]
}

fn legacy_reader(kind: &str) -> &'static str {
    match kind {
        "Placement" | "Isolation" => "Review",
        _ => "Appeals",
    }
}

#[derive(Clone, Debug)]
struct Fixture {
    facts: Vec<String>,
    mapping: BTreeMap<String, String>,
}

impl Fixture {
    fn term(&self, variable: &str) -> &str {
        self.mapping
            .get(variable)
            .unwrap_or_else(|| panic!("generated fixture has no {variable}"))
    }
}

#[derive(Clone, Debug)]
struct PinQuery {
    claim: String,
    query: String,
    expected: bool,
}

#[derive(Clone, Debug)]
struct PinCase {
    label: String,
    facts: Vec<String>,
    queries: Vec<PinQuery>,
}

impl PinCase {
    fn one(
        label: impl Into<String>,
        facts: Vec<String>,
        claim: impl Into<String>,
        query: impl Into<String>,
        expected: bool,
    ) -> Self {
        Self {
            label: label.into(),
            facts,
            queries: vec![PinQuery {
                claim: claim.into(),
                query: query.into(),
                expected,
            }],
        }
    }
}

fn raw_atoms(statement: &str) -> Vec<String> {
    let body = statement
        .split_once(" -> ")
        .map_or(statement, |parts| parts.0);
    let body = quantifier_prefix().replace(body, "");
    body.split(" & ")
        .filter(|atom| {
            ["authorized(", "observe(", "public(", "challenge("]
                .iter()
                .any(|prefix| atom.starts_with(prefix))
        })
        .map(str::to_owned)
        .collect()
}

fn constant(prefix: &str, variable: &str) -> String {
    let mut result = prefix.to_owned();
    for word in variable.trim_start_matches('$').split('_') {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.extend(first.to_uppercase());
            result.extend(chars.flat_map(char::to_lowercase));
        }
    }
    result
}

fn ground_rule(
    statement: &str,
    prefix: &str,
    overrides: &BTreeMap<String, String>,
    fused: Option<(&str, &str)>,
    omit_scopes: &[&str],
) -> Fixture {
    let mut mapping: BTreeMap<String, String> = ordered_variables(statement)
        .into_iter()
        .map(|variable| (variable.to_owned(), constant(prefix, variable)))
        .collect();
    mapping.extend(
        overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    if let Some((first, second)) = fused {
        let value = mapping
            .get(first)
            .unwrap_or_else(|| panic!("generated fixture has no fused variable {first}"))
            .clone();
        mapping.insert(second.to_owned(), value);
    }
    let ground = |value: &str| {
        variable_pattern()
            .replace_all(value, |captures: &Captures<'_>| {
                mapping
                    .get(&captures[0])
                    .unwrap_or_else(|| panic!("generated fixture has no {}", &captures[0]))
                    .to_owned()
            })
            .into_owned()
    };
    let mut facts = Vec::new();
    let mut seen = HashSet::new();
    for atom in raw_atoms(statement) {
        if omit_scopes
            .iter()
            .any(|scope| atom.ends_with(&format!(", {scope})")))
        {
            continue;
        }
        let fact = ground(&atom);
        if seen.insert(fact.clone()) {
            facts.push(fact);
        }
    }
    Fixture { facts, mapping }
}

fn effect_rule(effect: Effect, branch: &str) -> &'static str {
    if effect.number == 213 {
        return typed_reader_bridge();
    }
    let candidates = effect_conclusion_rules(effect);
    if effect.number == 210 && branch == "tie" {
        &candidates[candidates.len() / 2]
    } else {
        &candidates[0]
    }
}

fn finding_overrides(kind: &str) -> BTreeMap<String, String> {
    let reader = legacy_reader(kind);
    let subject = match kind {
        "Isolation" => "Adam".to_owned(),
        "MaturityDispute" => "Hano".to_owned(),
        _ => format!("{kind}Affected"),
    };
    let finding_subject = if kind == "OrderConflict" {
        "Order_Court_A".to_owned()
    } else {
        subject.clone()
    };
    BTreeMap::from([
        ("$bearer".to_owned(), "State".to_owned()),
        ("$target".to_owned(), subject.clone()),
        ("$subject".to_owned(), subject),
        ("$finding_subject".to_owned(), finding_subject),
        ("$finding_kind".to_owned(), kind.to_owned()),
        ("$reader".to_owned(), reader.to_owned()),
        ("$reader_duty".to_owned(), format!("Read{kind}FindingDuty")),
        (
            "$reader_standard".to_owned(),
            format!("Read{kind}FindingStandard"),
        ),
    ])
}

fn finding_fixture_facts(
    kind: &str,
    subject: &str,
    finding_subject: &str,
) -> ObligationResult<Vec<String>> {
    let entry = format!("{kind}Entry");
    let facts = match kind {
        "Placement" => vec![format!("put(State, {finding_subject}, Homestay)")],
        "Isolation" => Vec::new(),
        "StatusConflict" => vec![
            format!("person({finding_subject})"),
            format!("rotten({finding_subject})"),
            format!("authorized({finding_subject}, VoidStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, VoidScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, VoidScope)"),
            format!(
                "carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)"
            ),
            format!(
                "carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)"
            ),
            format!("authorized({finding_subject}, ClearStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, ClearScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, ClearScope)"),
            format!(
                "carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)"
            ),
            format!(
                "carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)"
            ),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "CarryOmission" => vec![
            format!("authorized({finding_subject}, VoidStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, VoidScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, VoidScope)"),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "CarryForgery" => vec![
            format!(
                "carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)"
            ),
            format!(
                "carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, VoidCarry)"
            ),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "ClearOmission" => vec![
            format!("authorized({finding_subject}, ClearStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, ClearScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, ClearScope)"),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "ClearForgery" => vec![
            format!(
                "carries(Chronicle, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)"
            ),
            format!(
                "carries(TemporalReview, {finding_subject}, Epoch_Current, Epoch_Previous, ClearCarry)"
            ),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "StandingOmission" => vec![
            format!("authorized({finding_subject}, StandingStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, StandingScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, StandingScope)"),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "RecordDisappearance" => vec![
            format!("authorized({finding_subject}, PreservedStatus, Epoch_Previous)"),
            format!("observe(Chronicle, {finding_subject}, Epoch_Previous, PreservedScope)"),
            format!("observe(TemporalReview, {finding_subject}, Epoch_Previous, PreservedScope)"),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "MaturityDispute" => vec![format!(
            "challenge({subject}, MaturityRecord, TemporalReview)"
        )],
        "OrderConflict" => vec![
            "list(ObligationsOrderOpposite, Epoch_Review, Epoch_Current, EventSequence)".to_owned(),
            "observe(Chronicle, ObligationsOrderOpposite, Epoch_Review, EventStartScope)"
                .to_owned(),
            "observe(TemporalReview, ObligationsOrderOpposite, Epoch_Review, EventStartScope)"
                .to_owned(),
            "observe(Chronicle, ObligationsOrderOpposite, Epoch_Current, EventEndScope)".to_owned(),
            "observe(TemporalReview, ObligationsOrderOpposite, Epoch_Current, EventEndScope)"
                .to_owned(),
            format!("challenge({subject}, {finding_subject}, TemporalReview)"),
        ],
        "TemporalRecord" => vec![format!(
            "authorized({finding_subject}, ActiveCustody, {kind}Case)"
        )],
        "TemporalAuthority" => vec![
            format!("person({finding_subject})"),
            format!("person({kind}Victim)"),
            format!("injure({finding_subject}, {kind}Victim)"),
            format!("judge(Court, {finding_subject})"),
            format!("challenge({subject}, {entry}, TemporalReview)"),
        ],
        "TemporalDispute" => vec![
            format!("challenge({subject}, {kind}Lease, TemporalReview)"),
            format!("authorized({kind}Lease, ActiveCustody, {kind}Case)"),
            format!("cite(Court, {kind}Case, {subject})"),
        ],
        _ => {
            return Err(obligation_error(format!(
                "unsupported finding fixture: {kind}"
            )));
        }
    };
    Ok(facts)
}

fn effect_fixture(
    effect: Effect,
    prefix: &str,
    branch: &str,
    fused: Option<(&str, &str)>,
    omit_scopes: &[&str],
    overrides: &BTreeMap<String, String>,
) -> ObligationResult<Fixture> {
    let mut base = BTreeMap::from([
        ("$bearer".to_owned(), constant(prefix, "$bearer")),
        ("$target".to_owned(), constant(prefix, "$target")),
        ("$principal".to_owned(), "State".to_owned()),
        ("$bearer_mode".to_owned(), effect.mode.to_owned()),
        ("$duty_class".to_owned(), effect.duty_class.to_owned()),
        ("$duty_kind".to_owned(), effect.duty_kind.to_owned()),
    ]);
    if effect.number >= 213 {
        base.extend(finding_overrides("Placement"));
        base.insert("$reader_duty".to_owned(), constant(prefix, "$reader_duty"));
        base.insert(
            "$reader_standard".to_owned(),
            constant(prefix, "$reader_standard"),
        );
    }
    base.extend(
        overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    let mut fixture = ground_rule(
        effect_rule(effect, branch),
        prefix,
        &base,
        fused,
        omit_scopes,
    );
    if effect.number >= 213 {
        let kind = base
            .get("$finding_kind")
            .expect("finding fixture has a kind");
        let subject = base.get("$subject").expect("finding fixture has a subject");
        let finding_subject = base
            .get("$finding_subject")
            .expect("finding fixture has a finding subject");
        let mut seen: HashSet<String> = fixture.facts.iter().cloned().collect();
        for fact in finding_fixture_facts(kind, subject, finding_subject)? {
            if seen.insert(fact.clone()) {
                fixture.facts.push(fact);
            }
        }
    }
    Ok(fixture)
}

fn plain_effect_fixture(effect: Effect, prefix: &str) -> ObligationResult<Fixture> {
    effect_fixture(effect, prefix, "standard", None, &[], &BTreeMap::new())
}

fn effect_query(effect: Effect, fixture: &Fixture) -> String {
    let head = effect_rule(effect, "standard")
        .split_once(" -> ")
        .expect("generated rule has a head")
        .1
        .strip_suffix('.')
        .expect("generated rule ends in a period");
    variable_pattern()
        .replace_all(head, |captures: &Captures<'_>| {
            fixture
                .mapping
                .get(&captures[0])
                .unwrap_or_else(|| panic!("fixture has no query variable {}", &captures[0]))
                .to_owned()
        })
        .into_owned()
}

fn append_facts(lines: &mut Vec<String>, facts: &[String]) {
    lines.extend(facts.iter().map(|fact| format!("{fact}.")));
}

fn append_query(lines: &mut Vec<String>, query: &PinQuery) {
    lines.extend([
        format!("# {}", query.claim),
        format!("? {}.", query.query),
        format!("# => {}", if query.expected { "TRUE" } else { "FALSE" }),
        String::new(),
    ]);
}

fn finalize_pins(header: &str, lines: &[String]) -> ObligationResult<String> {
    let queries: Vec<&str> = lines
        .iter()
        .filter_map(|line| {
            line.strip_prefix("? ")
                .and_then(|query| query.strip_suffix('.'))
        })
        .collect();
    let unique: HashSet<_> = queries.iter().copied().collect();
    if queries.len() != unique.len() {
        let mut duplicates: Vec<_> = unique
            .into_iter()
            .filter(|query| {
                queries
                    .iter()
                    .filter(|candidate| **candidate == *query)
                    .count()
                    > 1
            })
            .collect();
        duplicates.sort_unstable();
        duplicates.truncate(3);
        return Err(obligation_error(format!(
            "generated obligations queries are not unique: {duplicates:?}"
        )));
    }
    let mut all = vec![
        "# SPDX-License-Identifier: MIT OR Apache-2.0".to_owned(),
        header.to_owned(),
        format!(":expect-pins {}", queries.len()),
        String::new(),
    ];
    all.extend(lines.iter().cloned());
    Ok(format!("{}\n", all.join("\n").trim_end()))
}

fn conclusion_heads(effect: Effect) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for generated in effect_conclusion_rules(effect) {
        let head = generated
            .split_once(" -> ")
            .expect("generated rule has a head")
            .1
            .strip_suffix('.')
            .expect("generated rule ends in a period")
            .to_owned();
        if seen.insert(head.clone()) {
            result.push(head);
        }
    }
    result
}

fn ground_text(text: &str, mapping: &BTreeMap<String, String>) -> String {
    variable_pattern()
        .replace_all(text, |captures: &Captures<'_>| {
            mapping
                .get(&captures[0])
                .unwrap_or_else(|| panic!("fixture has no variable {}", &captures[0]))
                .to_owned()
        })
        .into_owned()
}

fn main_pin_cases(snapshot: &SourceSnapshot) -> ObligationResult<Vec<PinCase>> {
    let mut cases = Vec::new();
    for effect in EFFECTS {
        let fixture = plain_effect_fixture(effect, &format!("OblPositive{:03}", effect.number))?;
        let effect_query = effect_query(effect, &fixture);
        let mut queries = vec![PinQuery {
            claim: format!("FS-CCE-{:03} positive: {}.", effect.number, effect.title),
            query: effect_query.clone(),
            expected: true,
        }];
        if effect.number != 213 {
            for (index, head) in conclusion_heads(effect).into_iter().enumerate() {
                let grounded = ground_text(&head, &fixture.mapping);
                if grounded != effect_query {
                    queries.push(PinQuery {
                        claim: format!(
                            "FS-CCE-{:03} legal conclusion {} remains source-bound.",
                            effect.number,
                            index + 1
                        ),
                        query: grounded,
                        expected: true,
                    });
                }
            }
        }
        cases.push(PinCase {
            label: format!("FS-CCE-{:03} positive", effect.number),
            facts: fixture.facts,
            queries,
        });
    }
    for effect in EFFECTS {
        let fixture = effect_fixture(
            effect,
            &format!("OblFused{:03}", effect.number),
            "standard",
            Some(("$source", "$record_review")),
            &[],
            &BTreeMap::new(),
        )?;
        let query = effect_query(effect, &fixture);
        cases.push(PinCase::one(
            format!("FS-CCE-{:03} source-review independence", effect.number),
            fixture.facts,
            format!(
                "FS-CCE-{:03} withholds when source and record reviewer fuse.",
                effect.number
            ),
            query,
            false,
        ));
    }
    let omission_scopes = [
        "DutyBearerScope",
        "DutyScope",
        "DutyStandardScope",
        "DutyBeneficiaryOrObjectScope",
        "DutyKindScope",
        "DutyFunctionOrCommitmentScope",
        "DutyBearerModeScope",
        "DutyClassScope",
        "SourceVersionScope",
        "SourceEpochScope",
        "JurisdictionScope",
        "AuthorityScope",
        "DutyStartScope",
        "DutyEndScope",
        "ChallengeScope",
        "CorrectionScope",
        "RemedyScope",
        "DutyBreachScope",
        "DutyContinuityScope",
        "DutyPriorityScope",
        "DutyExcuseScope",
        "PublicPrincipalRetentionScope",
        "ExpressPrivateReachScope",
        "DutyNonWaiverScope",
        "FailurePolarityScope",
    ];
    for (index, scope) in omission_scopes.iter().enumerate() {
        let effect = effect_by_number(198);
        let fixture = effect_fixture(
            effect,
            &format!("OblOmit{:02}", index + 1),
            "standard",
            None,
            &[*scope],
            &BTreeMap::new(),
        )?;
        let query = effect_query(effect, &fixture);
        cases.push(PinCase::one(
            format!("origin omission {scope}"),
            fixture.facts,
            format!("Omitting {scope} withholds the duty origin."),
            query,
            false,
        ));
    }
    for (effect_number, scope) in [
        (203, "DelegatedDutyIdentityScope"),
        (204, "PrivateDutySourceScope"),
        (206, "PersonDutyRemedyScope"),
        (207, "CommonDutyRemedyScope"),
        (208, "RoleDutyCorrectionScope"),
        (209, "VoluntaryDutyBoundaryScope"),
        (210, "DutyPriorityBoundaryScope"),
        (211, "FiniteExcuseEndScope"),
        (213, "FindingReceiptEvidenceScope"),
        (214, "PositiveFindingNonresponseScope"),
        (215, "FindingEscalationScope"),
        (222, "IndividualReliefNonDelayScope"),
        (389, "PositiveDisclosureNonresponseScope"),
        (390, "DisclosureEscalationDutyScope"),
    ] {
        let effect = effect_by_number(effect_number);
        let fixture = effect_fixture(
            effect,
            &format!("OblSpecialOmit{effect_number}"),
            "standard",
            None,
            &[scope],
            &BTreeMap::new(),
        )?;
        let query = effect_query(effect, &fixture);
        cases.push(PinCase::one(
            format!("FS-CCE-{effect_number:03} omission {scope}"),
            fixture.facts,
            format!("FS-CCE-{effect_number:03} omitting {scope} withholds the effect."),
            query,
            false,
        ));
    }
    let class_effect = effect_by_number(198);
    let mut class_collision = effect_fixture(
        class_effect,
        "OblClassCollision",
        "standard",
        None,
        &[],
        &BTreeMap::new(),
    )?;
    class_collision.facts.push(format!(
        "observe({}, {}, ClassNineCommonDutyClass, DutyClassScope)",
        class_collision.term("$source"),
        class_collision.term("$origin")
    ));
    let query = effect_query(class_effect, &class_collision);
    cases.push(PinCase::one(
        "FS-CCE-198 conflicting duty class",
        class_collision.facts,
        "A conflicting class certificate cannot compose with the duty origin.",
        query,
        false,
    ));
    for (effect_number, scope, label) in [
        (
            210,
            "DutyPriorityDispositionScope",
            "FS-CCE-210 tie omission DutyPriorityDispositionScope",
        ),
        (
            211,
            "ExcuseRetentionScope",
            "FS-CCE-211 omission ExcuseRetentionScope",
        ),
        (
            212,
            "NonreciprocityScope",
            "FS-CCE-212 omission NonreciprocityScope",
        ),
    ] {
        let effect = effect_by_number(effect_number);
        let branch = if effect_number == 210 {
            "tie"
        } else {
            "standard"
        };
        let fixture = effect_fixture(
            effect,
            &format!("OblWatchedOmit{effect_number}"),
            branch,
            None,
            &[scope],
            &BTreeMap::new(),
        )?;
        let query = effect_query(effect, &fixture);
        cases.push(PinCase::one(
            label,
            fixture.facts,
            format!("FS-CCE-{effect_number:03} omitting {scope} withholds the watched effect."),
            query,
            false,
        ));
    }
    let alternate_effect = effect_by_number(215);
    let nonresponse_alternate = effect_fixture(
        alternate_effect,
        "OblNonresponseAlternate",
        "standard",
        None,
        &[
            "PositiveFindingNonresponseScope",
            "AlternateFindingReaderScope",
        ],
        &BTreeMap::new(),
    )?;
    let query = effect_query(alternate_effect, &nonresponse_alternate);
    cases.push(PinCase::one(
        "FS-CCE-215 omission nonresponse alternate",
        nonresponse_alternate.facts,
        "Alternate escalation requires both certified nonresponse and its exact alternate.",
        query,
        false,
    ));
    let priority_effect = effect_by_number(210);
    let tie = effect_fixture(
        priority_effect,
        "OblPriorityTie",
        "tie",
        None,
        &[],
        &BTreeMap::new(),
    )?;
    let query = effect_query(priority_effect, &tie);
    cases.push(PinCase::one(
        "FS-CCE-210 tie",
        tie.facts,
        "An independently certified tie preserves both duties and assigns review.",
        query,
        true,
    ));
    let class_priority = plain_effect_fixture(priority_effect, "OblClassPriority")?;
    let query = effect_query(priority_effect, &class_priority);
    let class_priority_facts = class_priority
        .facts
        .iter()
        .map(|fact| fact.replace("SourceSpecificPairPriority", "DutyClassPriorityDisposition"))
        .collect();
    cases.push(PinCase::one(
        "FS-CCE-210 class priority refusal",
        class_priority_facts,
        "Duty class cannot become a conflict priority key.",
        query,
        false,
    ));
    let excuse_effect = effect_by_number(211);
    let self_excuse = effect_fixture(
        excuse_effect,
        "OblSelfExcuse",
        "standard",
        Some(("$source", "$bearer")),
        &[],
        &BTreeMap::new(),
    )?;
    let query = effect_query(excuse_effect, &self_excuse);
    cases.push(PinCase::one(
        "FS-CCE-211 self-certified excuse refusal",
        self_excuse.facts,
        "A duty bearer cannot certify its own excuse.",
        query,
        false,
    ));
    let wall_effect = effect_by_number(212);
    for (index, claim_ref) in snapshot.protected_claim_refs.iter().enumerate() {
        let overrides =
            BTreeMap::from([("$protected_effect".to_owned(), claim_ref.replace('-', "_"))]);
        let fixture = effect_fixture(
            wall_effect,
            &format!("OblWall{:02}", index + 1),
            "standard",
            None,
            &[],
            &overrides,
        )?;
        let query = effect_query(wall_effect, &fixture);
        cases.push(PinCase::one(
            format!("FS-CCE-212 {claim_ref}"),
            fixture.facts,
            format!("FS-CCE-212 protects the declared route for {claim_ref}."),
            query,
            true,
        ));
    }
    let reader_effect = effect_by_number(213);
    for (index, kind) in FINDING_KINDS.iter().enumerate() {
        let overrides = finding_overrides(kind);
        let fixture = effect_fixture(
            reader_effect,
            &format!("OblFinding{:02}", index + 1),
            "standard",
            None,
            &[],
            &overrides,
        )?;
        let reader = overrides.get("$reader").expect("reader override");
        let duty = overrides.get("$reader_duty").expect("reader duty override");
        let standard = overrides
            .get("$reader_standard")
            .expect("reader standard override");
        cases.push(PinCase {
            label: format!("{kind} finding route"),
            facts: fixture.facts,
            queries: vec![
                PinQuery {
                    claim: format!("The exact {kind} finding route reaches its typed reader duty."),
                    query: format!("obliged({reader}, {duty}, {standard})"),
                    expected: true,
                },
                PinQuery {
                    claim: format!("The wrong recipient gains no {kind} reader duty."),
                    query: format!("obliged(Wrong{kind}Reader, {duty}, {standard})"),
                    expected: false,
                },
            ],
        });
    }
    Ok(cases)
}

fn append_pin_case(lines: &mut Vec<String>, case: &PinCase) {
    append_facts(lines, &case.facts);
    for query in &case.queries {
        append_query(lines, query);
    }
}

fn render_obligations_pins(cases: &[PinCase]) -> ObligationResult<String> {
    let mut lines = vec ! ["# Supplied records establish bounded legal effects only." . to_owned () , "# They prove no receipt, action, delivery, remedy, recurrence control, or institutional liveness." . to_owned () , String :: new () ,] ;
    for case in cases {
        append_pin_case(&mut lines, case);
    }
    finalize_pins(MAIN_HEADER, &lines)
}

fn render_independence_counterfactual(source: &str) -> ObligationResult<String> {
    let rules = block(source, RULES_BEGIN, RULES_END)?;
    if rules.matches(CURRENT_REVIEW_GUARD).count() < EFFECTS.len() {
        return Err(obligation_error(
            "current-source independence seams drifted",
        ));
    }
    let mutated = rules.replace(&format!(" & {CURRENT_REVIEW_GUARD}"), "");
    let start = source.find(RULES_BEGIN).expect("validated marker exists") + RULES_BEGIN.len();
    let stop = start
        + source[start..]
            .find(RULES_END)
            .expect("validated ordered marker exists");
    Ok(format!(
        "{}{}{}",
        &source[..start],
        mutated,
        &source[stop..]
    ))
}

fn remove_rule_once(mut source: String, remove_rule: &str) -> ObligationResult<String> {
    if source.matches(remove_rule).count() != 1 {
        return Err(obligation_error("counterfactual seam occurrence drifted"));
    }
    let needle = format!("{remove_rule}\n");
    source = source.replacen(&needle, "", 1);
    Ok(source)
}

fn render_source_counterfactual(source: &str) -> ObligationResult<String> {
    let mut output = source.to_owned();
    let mut rules: Vec<&str> = finding_route_rules().iter().map(String::as_str).collect();
    for effect in EFFECTS {
        rules.extend(effect_conclusion_rules(effect).iter().map(String::as_str));
    }
    rules.push(typed_reader_bridge());
    for generated in rules {
        if output.matches(generated).count() != 1 {
            return Err(obligation_error(
                "source-removal counterfactual seam occurrence drifted",
            ));
        }
        output = output.replacen(&format!("{generated}\n"), "", 1);
    }
    Ok(output)
}

fn render_reader_counterfactual(source: &str) -> ObligationResult<String> {
    remove_rule_once(source.to_owned(), typed_reader_bridge())
}

fn render_independence_pins() -> ObligationResult<String> {
    let mut lines = Vec::new();
    for effect in EFFECTS {
        let fixture = effect_fixture(
            effect,
            &format!("OblIndependence{:03}", effect.number),
            "standard",
            Some(("$source", "$record_review")),
            &[],
            &BTreeMap::new(),
        )?;
        append_facts(&mut lines, &fixture.facts);
        append_query(
            &mut lines,
            &PinQuery {
                claim: format!(
                    "FS-CCE-{:03} widens under fused source/review.",
                    effect.number
                ),
                query: effect_query(effect, &fixture),
                expected: true,
            },
        );
    }
    finalize_pins(INDEPENDENCE_HEADER, &lines)
}

fn render_source_pins() -> ObligationResult<String> {
    let mut lines = Vec::new();
    for effect in EFFECTS {
        let fixture =
            plain_effect_fixture(effect, &format!("OblSourceRemoval{:03}", effect.number))?;
        append_facts(&mut lines, &fixture.facts);
        append_query(
            &mut lines,
            &PinQuery {
                claim: format!(
                    "FS-CCE-{:03} disappears with origin materialization.",
                    effect.number
                ),
                query: effect_query(effect, &fixture),
                expected: false,
            },
        );
    }
    append_query(
        &mut lines,
        &PinQuery {
            claim: "Unrelated personhood remains outside the removed obligation rule.".to_owned(),
            query: "person(Adam)".to_owned(),
            expected: true,
        },
    );
    finalize_pins(SOURCE_HEADER, &lines)
}

fn render_reader_pins() -> ObligationResult<String> {
    let mut lines = Vec::new();
    let effect = effect_by_number(213);
    for (index, kind) in FINDING_KINDS.iter().enumerate() {
        let overrides = finding_overrides(kind);
        let fixture = effect_fixture(
            effect,
            &format!("OblReaderRemoval{:02}", index + 1),
            "standard",
            None,
            &[],
            &overrides,
        )?;
        append_facts(&mut lines, &fixture.facts);
        let reader = &overrides["$reader"];
        let subject = &overrides["$subject"];
        let duty = &overrides["$reader_duty"];
        let standard = &overrides["$reader_standard"];
        append_query(
            &mut lines,
            &PinQuery {
                claim: format!(
                    "The {kind} compatibility conclusion remains after reader ablation."
                ),
                query: format!("obliged({reader}, {subject})"),
                expected: true,
            },
        );
        append_query(
            &mut lines,
            &PinQuery {
                claim: format!("The {kind} typed reader duty disappears under reader ablation."),
                query: format!("obliged({reader}, {duty}, {standard})"),
                expected: false,
            },
        );
    }
    finalize_pins(READER_HEADER, &lines)
}

fn render_formal_block() -> String {
    let mut lines = vec![
        "# Generated by ./generate.sh obligations.".to_owned(),
        "# Every downstream effect repeats the raw current source and origin join.".to_owned(),
        "# Formal conclusions prove no receipt, action, delivery, remedy, or liveness.".to_owned(),
    ];
    lines.extend(formal_rules().iter().cloned());
    lines.join("\n")
}

pub(crate) fn generate(
    context: &Context,
    export: &mut crate::authoring::Export,
) -> Result<(), Error> {
    let snapshot = source_snapshot(context)?;
    let cases = main_pin_cases(&snapshot).map_err(obligations_error)?;
    let pins = render_obligations_pins(&cases).map_err(obligations_error)?;
    let independence = render_independence_pins().map_err(obligations_error)?;
    let source = render_source_pins().map_err(obligations_error)?;
    let reader = render_reader_pins().map_err(obligations_error)?;
    let constitution = context.read(CONSTITUTION_PATH)?;
    let updated = replace_block(
        &constitution,
        RULES_BEGIN,
        RULES_END,
        &render_formal_block(),
    )
    .map_err(obligations_error)?;
    std::fs::write(context.path(CONSTITUTION_PATH), updated)?;
    for (path, content) in [
        ("new-book-plans/obligations.pins.nibli", pins),
        (
            "new-book-plans/counterfactual/no-obligations-independent-source-review.pins.nibli",
            independence,
        ),
        (
            "new-book-plans/counterfactual/no-obligations-source.pins.nibli",
            source,
        ),
        (
            "new-book-plans/counterfactual/no-obligations-finding-reader.pins.nibli",
            reader,
        ),
    ] {
        std::fs::write(context.path(path), content)?;
    }
    export_cases(context, export)
}

pub(crate) fn export_cases(
    context: &Context,
    export: &mut crate::authoring::Export,
) -> Result<(), Error> {
    use crate::authoring::{Base, Edit};
    let constitution = context.read(CONSTITUTION_PATH)?;
    let snapshot = source_snapshot(context)?;
    let cases = main_pin_cases(&snapshot).map_err(obligations_error)?;
    for (index, case) in cases.iter().enumerate() {
        let fixture = case
            .facts
            .iter()
            .map(|fact| format!("{fact}.\n"))
            .collect::<String>();
        let mut pins = format!(
            "# SPDX-License-Identifier: MIT OR Apache-2.0\n# {}\n:expect-pins {}\n",
            case.label,
            case.queries.len()
        );
        for query in &case.queries {
            let _ = writeln!(
                pins,
                "\n# {}\n? {}.\n# => {}",
                query.claim,
                query.query,
                if query.expected { "TRUE" } else { "FALSE" }
            );
        }
        export.add_case(
            context,
            &format!("obligations/case-{:03}", index + 1),
            "live",
            &fixture,
            &[("expect", &pins)],
            Vec::new(),
            true,
        )?;
    }
    let mut independence = Vec::new();
    for line in block(&constitution, RULES_BEGIN, RULES_END)
        .map_err(obligations_error)?
        .lines()
    {
        if line.contains(CURRENT_REVIEW_GUARD) {
            independence.push(Edit {
                before: format!("{line}\n"),
                after: format!(
                    "{}\n",
                    line.replace(&format!(" & {CURRENT_REVIEW_GUARD}"), "")
                ),
            });
        }
    }
    let mut removal = finding_route_rules()
        .iter()
        .map(|line| Edit {
            before: format!("{line}\n"),
            after: String::new(),
        })
        .collect::<Vec<_>>();
    for effect in EFFECTS {
        removal.extend(effect_conclusion_rules(effect).iter().map(|line| Edit {
            before: format!("{line}\n"),
            after: String::new(),
        }));
    }
    removal.push(Edit {
        before: format!("{}\n", typed_reader_bridge()),
        after: String::new(),
    });
    let reader = vec![Edit {
        before: format!("{}\n", typed_reader_bridge()),
        after: String::new(),
    }];
    for (id, edits, pins, expected) in [
        (
            "no-independent-review",
            independence,
            render_independence_pins(),
            render_independence_counterfactual(&constitution),
        ),
        (
            "no-source",
            removal,
            render_source_pins(),
            render_source_counterfactual(&constitution),
        ),
        (
            "no-reader",
            reader,
            render_reader_pins(),
            render_reader_counterfactual(&constitution),
        ),
    ] {
        let pins = pins.map_err(obligations_error)?;
        if crate::authoring::apply_edits(&constitution, &edits)?
            != expected.map_err(obligations_error)?
        {
            return Err(Error::new(format!(
                "obligations {id} extraction changes its source variant"
            )));
        }
        let base = format!("obligations-{id}");
        export.bases.insert(
            base.clone(),
            Base {
                base: Some("live".into()),
                edits,
                path: None,
            },
        );
        export.add_case(
            context,
            &format!("obligations/{id}"),
            &base,
            "",
            &[("expect", &pins)],
            Vec::new(),
            false,
        )?;
    }
    Ok(())
}
