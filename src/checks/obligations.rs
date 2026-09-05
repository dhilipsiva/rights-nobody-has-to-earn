// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native generator and verifier for the FS-CVF-016 obligation family.
//!
//! The suite keeps its two large reviewed inputs in an immutable, shareable
//! snapshot. Rendering, validation, and execution therefore reuse the same
//! constitution and ledger bytes. Executable cases call the in-process pin
//! runner with a fresh engine per case; no Python or `nibli-pin` child process
//! is involved in production operation.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::fmt::Write as _;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use regex::{Captures, Regex};

use crate::checks::ledger;
use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{self, LoadedSource, PinOptions, PreparedPinEngine};
use crate::scheduler::CancellationToken;

const CONSTITUTION_PATH: &str = "new-book-plans/constitution.nibli";
const LEDGER_PATH: &str = "new-book-plans/full-society-ledger.json";
const OBLIGATIONS_PINS_PATH: &str = "new-book-plans/obligations.pins.nibli";
const INDEPENDENCE_CF_PATH: &str =
    "new-book-plans/counterfactual/no-obligations-independent-source-review.nibli";
const SOURCE_CF_PATH: &str = "new-book-plans/counterfactual/no-obligations-source.nibli";
const READER_CF_PATH: &str = "new-book-plans/counterfactual/no-obligations-finding-reader.nibli";

const DERIVED_BEGIN: &str = "# <OBLIGATIONS-DERIVED-BEGIN>";
const DERIVED_END: &str = "# <OBLIGATIONS-DERIVED-END>";
const RULES_BEGIN: &str = "# <OBLIGATIONS-RULES-BEGIN>";
const RULES_END: &str = "# <OBLIGATIONS-RULES-END>";
const DERIVED_STATEMENT: &str = "derived_only(\"obliged\").";
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

const OBLIGATION_STATEMENT_SET_SHA256: &str =
    "f7e745bb141be4aba91902d78ec469977d3f8a72de46462084cde99e83320acc";
const PROSE_AGGREGATE_SHA256: &str =
    "bfb4789b9c24b236d95c26f50448221000491c0d8e1161cd8089bbca7db1ec26";
const DELIVERY_PROSE_AGGREGATE_SHA256: &str =
    "eeff5e40301bfaf85b9a1b5eb81de6821c790117aebe923f513d5bcabf0b34fd";
const ECONOMIC_PROSE_AGGREGATE_SHA256: &str =
    "e642322b7a787afb6adc7606c906b56e321385c5808a1418927e9239c6c2597d";

#[derive(Clone, Copy, Debug)]
struct Effect {
    number: u16,
    key: &'static str,
    title: &'static str,
    mode: &'static str,
    duty_class: &'static str,
    duty_kind: &'static str,
}

const EFFECTS: [Effect; 25] = [
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
];

#[derive(Clone, Copy)]
struct ProsePayload {
    id: &'static str,
    path: &'static str,
    length: usize,
    digest: &'static str,
}

const PROSE_PAYLOADS: [ProsePayload; 11] = [
    ProsePayload {
        id: "OBL-B1-01",
        path: "book-1/00-opening-note.md",
        length: 1143,
        digest: "33be8df380b9f1fc13c96d4b01b6fdba67edb2333f1b473824f5650272442a28",
    },
    ProsePayload {
        id: "OBL-B1-02",
        path: "book-1/00-opening-note.md",
        length: 549,
        digest: "3b14a4be32862226caf8241ab4f196ad5ef3996ecc123957c07cbb12263f497c",
    },
    ProsePayload {
        id: "OBL-B1-03",
        path: "book-1/00-opening-note.md",
        length: 651,
        digest: "c0820bfd382a08120066580a66972cf30ed28c733363e20096b486eb4fc49532",
    },
    ProsePayload {
        id: "OBL-B1-04",
        path: "book-1/00-opening-note.md",
        length: 711,
        digest: "4704ab855b2b4a562d0a37450de1966aca4cbedbf0548049cb47e4c62aec197e",
    },
    ProsePayload {
        id: "OBL-B1-05",
        path: "book-1/00-opening-note.md",
        length: 1264,
        digest: "eedf0fa0a4b2f0bd10be48f29132048e7b3b7922b76670db5a9a9408a315b6b2",
    },
    ProsePayload {
        id: "OBL-B1-06",
        path: "book-1/02-public-answerability.md",
        length: 956,
        digest: "8f5746aa21893ffe3b3f48431300c453b2bd054f8b34542ebf7438ddcc2a9cdf",
    },
    ProsePayload {
        id: "OBL-B1-07",
        path: "book-1/08-what-you-are-owed.md",
        length: 523,
        digest: "e69ccea1f8840f67e154e394c2c5f83d3c23cfd10d6efedff6cee9b514a87e79",
    },
    ProsePayload {
        id: "OBL-B1-13",
        path: "book-1/14-when-the-system-notices-it-broke.md",
        length: 1130,
        digest: "a0706ad6a5005e4828b5dc8f5cf5142f51ad9bf155cece94b3cdc389f196390d",
    },
    ProsePayload {
        id: "OBL-B1-14",
        path: "book-1/15-the-five-joints.md",
        length: 659,
        digest: "debd5f0e222b7102162f58e824fbd6d8c457949bb8bb3efc307221b16a3e813e",
    },
    ProsePayload {
        id: "OBL-B1-16",
        path: "book-1/method.md",
        length: 1537,
        digest: "2db17589f28d3de01df371766288e57ee8fbc4c23766395105d1221ca66e3df1",
    },
    ProsePayload {
        id: "OBL-B1-17",
        path: "book-1/method.md",
        length: 1861,
        digest: "9d144831cbd3eec27d92269d9d93d173247e9fee2aa36996a6f0b1a3ebcbfe5f",
    },
];

const DELIVERY_PROSE_PAYLOADS: [ProsePayload; 8] = [
    ProsePayload {
        id: "DLV-B1-01",
        path: "book-1/08-what-you-are-owed.md",
        length: 5379,
        digest: "2b7b5991c69e6ee6f5c8b2c57f6f328eab4885c8fb40660d6734c9c4eb540f27",
    },
    ProsePayload {
        id: "DLV-B1-02",
        path: "book-1/08-what-you-are-owed.md",
        length: 2925,
        digest: "1c36283a3949033d8dbed4100fc58379db46caf86cb5f073a284852ee262f4e7",
    },
    ProsePayload {
        id: "DLV-B1-03",
        path: "book-1/08-what-you-are-owed.md",
        length: 1244,
        digest: "0eaa7c900e3945250dc357371537dc3eb085607192f1c0ee60f383c3c3cf1cfa",
    },
    ProsePayload {
        id: "DLV-B1-05",
        path: "book-1/13-the-one-thing-taken.md",
        length: 2759,
        digest: "78f6634729415c4b4f43d6050cdaf6fc76400405d132cbb89d53c31582799688",
    },
    ProsePayload {
        id: "DLV-B1-06",
        path: "book-1/14-when-the-system-notices-it-broke.md",
        length: 1819,
        digest: "fff58727f9a2ff4b242262dc6d0a51b1cf5b27b9e543a46c2668423a6a14b2af",
    },
    ProsePayload {
        id: "DLV-B1-08",
        path: "book-1/15-the-five-joints.md",
        length: 4099,
        digest: "7b8d518161c828b22269fafd950d8ecfe4699cf36814a58292384fb46eaec8ac",
    },
    ProsePayload {
        id: "DLV-B1-09",
        path: "book-1/method.md",
        length: 1097,
        digest: "0eb3513d83ca9a44b6a73ea539a6632032a78bec2834e104e582d4185cbfcd6e",
    },
    ProsePayload {
        id: "DLV-B1-10",
        path: "book-1/method.md",
        length: 2439,
        digest: "6c301a235d19332992cf2ad6643b8ae3f47c85488bd1be19cb5f9ac456d638cc",
    },
];

const ECONOMIC_PROSE_PAYLOADS: [ProsePayload; 3] = [
    ProsePayload {
        id: "ECON-B1-01",
        path: "book-1/08-what-you-are-owed.md",
        length: 5054,
        digest: "ea166a02e26f99691b151b730258aa65a8d763be6ce29f02a9a83f19b883b119",
    },
    ProsePayload {
        id: "ECON-B1-02",
        path: "book-1/13-the-one-thing-taken.md",
        length: 541,
        digest: "bee2c6504c7edd58c8ca2730bac7b4a18a1ac4d02b88cb2b6418f05bea6bfe3e",
    },
    ProsePayload {
        id: "ECON-B1-03",
        path: "book-1/14-when-the-system-notices-it-broke.md",
        length: 4411,
        digest: "115e59dbc15acfc05324bdd34478132cbfd9ed54df9fb4f1eef937adbdfd66d7",
    },
];

#[derive(Clone, Debug)]
pub(crate) struct SourceSnapshot {
    constitution: Arc<str>,
    protected_claim_refs: Arc<[String]>,
}

impl SourceSnapshot {
    pub(crate) fn from_sources(
        context: &Context,
        constitution: impl Into<Arc<str>>,
        ledger: impl Into<Arc<str>>,
    ) -> Result<Self, Error> {
        let constitution = constitution.into();
        let ledger = ledger.into();
        let protected_claim_refs = ledger::protected_claim_refs_from_source(context, &ledger)?;
        Ok(Self {
            constitution,
            protected_claim_refs: protected_claim_refs.into(),
        })
    }

    fn from_validated_ledger(
        constitution: impl Into<Arc<str>>,
        validated: &ledger::ValidatedLedger,
    ) -> Result<Self, Error> {
        Ok(Self {
            constitution: constitution.into(),
            protected_claim_refs: ledger::protected_claim_refs_from_validated(validated)?.into(),
        })
    }

    pub(crate) fn constitution(&self) -> &str {
        &self.constitution
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckReport {
    pub(crate) effects: usize,
    pub(crate) exact_statements: usize,
    pub(crate) main_pins: usize,
    pub(crate) independence_pins: usize,
    pub(crate) source_pins: usize,
    pub(crate) reader_pins: usize,
}

impl fmt::Display for CheckReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "obligations: PASS - {} effects, {} exact statements, {} main pins, \
             {}/{}/{} counterfactual pins, {} retained OBL-B1-v1, {} retained \
             DLV-B1-v1, and {} successor ECON-B1-v1 byte-exact prose payloads, \
             12 watched mutation seams, and one exact obliged consumer",
            self.effects,
            self.exact_statements,
            self.main_pins,
            self.independence_pins,
            self.source_pins,
            self.reader_pins,
            PROSE_PAYLOADS.len(),
            DELIVERY_PROSE_PAYLOADS.len(),
            ECONOMIC_PROSE_PAYLOADS.len(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutionReport {
    pub(crate) cases: usize,
    pub(crate) counterfactual_suites: usize,
    pub(crate) lines: Vec<String>,
}

/// Every byte needed by the executable obligations family after preflight.
///
/// Construction validates the live artifacts and captures the rendered cases;
/// execution therefore has no repository or working-directory dependency.
pub(crate) struct ExecutionPlan {
    tasks: Vec<ExecutionTask>,
    cases: usize,
    counterfactual_suites: usize,
}

impl fmt::Display for ExecutionReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.lines.join("\n"))
    }
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

fn ensure_execution_active(cancellation: Option<&CancellationToken>) -> ObligationResult<()> {
    if cancellation.is_some_and(CancellationToken::is_cancelled) {
        Err(obligation_error("obligations execution cancelled"))
    } else {
        Ok(())
    }
}

pub(crate) fn load_snapshot(context: &Context) -> Result<SourceSnapshot, Error> {
    let constitution = context.read(CONSTITUTION_PATH)?;
    let ledger = context.read(LEDGER_PATH)?;
    SourceSnapshot::from_sources(
        context,
        Arc::<str>::from(constitution),
        Arc::<str>::from(ledger),
    )
}

pub(crate) fn load_snapshot_with_ledger(
    context: &Context,
    validated: &ledger::ValidatedLedger,
) -> Result<SourceSnapshot, Error> {
    SourceSnapshot::from_validated_ledger(context.read(CONSTITUTION_PATH)?, validated)
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

fn statement_id(statement: &str) -> String {
    let without_period = statement.strip_suffix('.').unwrap_or(statement);
    let encoded = serde_json::to_string(&(without_period, 0)).expect("statement tuple serializes");
    sha256(encoded.as_bytes())
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
    // Kept as a separate certificate collision rule, not part of the origin
    // field loop. This sentinel is removed before generation below.
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
    atoms.extend([
        "observe($source, $record, $temporal_record, TemporalRecordScope)".to_owned(),
        "observe($record_review, $record, $temporal_record, TemporalRecordScope)".to_owned(),
        "observe($temporal, $temporal_record, $record, ObligationsRecordScope)".to_owned(),
        "observe($temporal_review, $temporal_record, $record, ObligationsRecordScope)".to_owned(),
        "observe($source, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)".to_owned(),
        "observe($record_review, $reconciliation, ObligationsRecordReconciled, ReconciliationStatusScope)".to_owned(),
        "observe($source, $reconciliation, $record, ObligationsRecordScope)".to_owned(),
        "observe($record_review, $reconciliation, $record, ObligationsRecordScope)".to_owned(),
        "observe($source, $reconciliation, $version, SourceVersionScope)".to_owned(),
        "observe($record_review, $reconciliation, $version, SourceVersionScope)".to_owned(),
        "~($source = $temporal)".to_owned(),
        "~($source = $temporal_review)".to_owned(),
        CURRENT_REVIEW_GUARD.to_owned(),
        "~($temporal = $temporal_review)".to_owned(),
        "~($temporal = $record_review)".to_owned(),
        "~($temporal_review = $record_review)".to_owned(),
    ]);
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

fn effect_conclusion_rules(effect: Effect) -> &'static [String] {
    &effect_rule_sets()[(effect.number - 198) as usize]
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

fn formal_statements() -> impl Iterator<Item = &'static str> {
    std::iter::once(DERIVED_STATEMENT).chain(formal_rules().iter().map(String::as_str))
}

fn effect_by_number(number: u16) -> Effect {
    EFFECTS[(number - 198) as usize]
}

fn legacy_reader(kind: &str) -> &'static str {
    match kind {
        "Placement" | "Isolation" => "Review",
        _ => "Appeals",
    }
}

fn enacted_statements(source: &str) -> ObligationResult<Vec<&str>> {
    let mut statements = Vec::new();
    statements.extend(
        block(source, DERIVED_BEGIN, DERIVED_END)?
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#')),
    );
    statements.extend(
        block(source, RULES_BEGIN, RULES_END)?
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#')),
    );
    Ok(statements)
}

fn body_relations(statement: &str) -> BTreeSet<&str> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN
        .get_or_init(|| Regex::new(r"\b([a-z][a-z0-9_]*)\(").expect("valid body relation regex"));
    let body = statement
        .split_once(" -> ")
        .map_or(statement, |parts| parts.0);
    pattern
        .captures_iter(body)
        .filter_map(|capture| capture.get(1).map(|matched| matched.as_str()))
        .collect()
}

fn consumer_rules(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && line.contains(" -> ")
                && body_relations(line).contains("obliged")
        })
        .collect()
}

fn validate_consumer_rules(rules: &[&str]) -> ObligationResult<()> {
    if rules != [typed_reader_bridge()] {
        return Err(obligation_error(
            "obliged consumers differ from the one family-owned typed reader bridge",
        ));
    }
    Ok(())
}

fn validate_consumer_allowlist_inner(source: &str, mutation_control: bool) -> ObligationResult<()> {
    let mut rules = consumer_rules(source);
    validate_consumer_rules(&rules)?;
    if mutation_control {
        let outside = "all $x: obliged(Review, $x) -> complete($x, OutsideObligationsReader).";
        rules.push(outside);
        if validate_consumer_rules(&rules).is_ok() {
            return Err(obligation_error(
                "outside obliged-reader mutation was not rejected",
            ));
        }
    }
    Ok(())
}

fn err_kinds(source: &str) -> BTreeSet<String> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(r"^err\([^,]+, ([A-Z][A-Za-z0-9_]*)\)\.$").expect("valid error-kind regex")
    });
    source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#') && line.contains(" -> err("))
        .filter_map(|line| line.split_once(" -> ").map(|parts| parts.1))
        .filter_map(|head| pattern.captures(head))
        .filter_map(|capture| capture.get(1).map(|matched| matched.as_str().to_owned()))
        .collect()
}

fn validate_route_exhaustiveness(source: &str) -> ObligationResult<()> {
    let actual = err_kinds(source);
    let expected: BTreeSet<String> = FINDING_KINDS
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect();
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).cloned().collect();
        let unexpected: Vec<_> = actual.difference(&expected).cloned().collect();
        return Err(obligation_error(format!(
            "finding kind inventory drifted: missing={missing:?} unexpected={unexpected:?}"
        )));
    }
    for kind in FINDING_KINDS {
        let reader = legacy_reader(kind);
        let mut found = false;
        for line in source.lines() {
            let Some((body, head)) = line.split_once(" -> ") else {
                continue;
            };
            if line.contains(" -> obliged(")
                && body.contains(&format!(", {kind})"))
                && head.starts_with(&format!("obliged({reader}, "))
            {
                found = true;
                break;
            }
        }
        if !found {
            return Err(obligation_error(format!(
                "finding kind lacks its exact {reader} reader route: {kind}"
            )));
        }
    }
    Ok(())
}

fn validate_contract_shape(rules: &[String]) -> ObligationResult<()> {
    for token in [
        "DutyBearerModeCertificateScope",
        "DutyClassCertificateScope",
        "ObligationClassBinding",
        "ObligationOriginBinding",
        "ObligationVersionBinding",
        "ObligationJurisdictionBinding",
        "ObligationScopeBinding",
        "ObligationPriorityBinding",
        "ObligationExcuseBinding",
        "DutyClassForbiddenAsPriority",
        "RightsAndContinuityFirst",
        "CertifiedTieOrMissingPriority",
        "OriginalDutyAccountabilityClaimBreachPrincipalAndRemediesRemain",
        "NoDutyPerformanceOrFailureGate",
        "SystemicWorkCannotDelayIndividualContinuityOrRemedy",
    ] {
        if !rules.iter().any(|rule| rule.contains(token)) {
            return Err(obligation_error(format!(
                "obligations contract omits {token}"
            )));
        }
    }
    if rules.iter().any(|rule| rule.contains("admits(")) {
        return Err(obligation_error(
            "FS-CVF-016 may not add an admitted relation",
        ));
    }
    for effect in EFFECTS {
        let mut owned = Vec::new();
        let effect_rules: &[String] = if effect.number == 213 {
            owned.push(typed_reader_bridge().to_owned());
            &owned
        } else {
            effect_conclusion_rules(effect)
        };
        let needle = format!("FSCCE_{:03}", effect.number);
        if effect_rules.is_empty() || effect_rules.iter().any(|rule| !rule.contains(&needle)) {
            return Err(obligation_error(format!(
                "FS-CCE-{:03} source-bound conclusion drifted",
                effect.number
            )));
        }
    }
    Ok(())
}

fn validate_formal_surface(source: &str) -> ObligationResult<Vec<&str>> {
    let actual = enacted_statements(source)?;
    let expected: Vec<_> = formal_statements().collect();
    if actual != expected {
        return Err(obligation_error(
            "FS-CVF-016 source block differs from its generator",
        ));
    }
    let actual_ids: Vec<_> = actual
        .iter()
        .map(|statement| statement_id(statement))
        .collect();
    if sha256(actual_ids.join("\n").as_bytes()) != OBLIGATION_STATEMENT_SET_SHA256 {
        return Err(obligation_error("FS-CVF-016 exact statement IDs changed"));
    }
    validate_contract_shape(formal_rules())?;
    validate_route_exhaustiveness(source)?;
    validate_consumer_allowlist_inner(source, true)?;
    Ok(actual)
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

const WATCHED_MUTATION_CASES: [(&str, &[&str]); 12] = [
    (
        "raw-currentness-rejoin",
        &["origin omission SourceVersionScope"],
    ),
    ("duty-origin-binding", &["origin omission DutyScope"]),
    (
        "principal-non-transfer",
        &["FS-CCE-203 omission DelegatedDutyIdentityScope"],
    ),
    (
        "express-private-source",
        &["FS-CCE-204 omission PrivateDutySourceScope"],
    ),
    ("class-exclusivity", &["FS-CCE-198 conflicting duty class"]),
    (
        "entitlement-wall",
        &["FS-CCE-212 omission NonreciprocityScope"],
    ),
    (
        "rights-first-conflict",
        &["FS-CCE-210 omission DutyPriorityBoundaryScope"],
    ),
    (
        "tie-repair",
        &["FS-CCE-210 tie omission DutyPriorityDispositionScope"],
    ),
    (
        "excuse-independence-origin",
        &[
            "FS-CCE-211 self-certified excuse refusal",
            "FS-CCE-211 omission ExcuseRetentionScope",
        ],
    ),
    ("finding-reader", &["finding-reader counterfactual"]),
    (
        "nonresponse-alternate",
        &["FS-CCE-215 omission nonresponse alternate"],
    ),
    (
        "systemic-individual-separation",
        &["FS-CCE-222 omission IndividualReliefNonDelayScope"],
    ),
];

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
            format!("FS-CCE-212 protects the ledger-derived route for {claim_ref}."),
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

fn validate_watched_mutation_cases(cases: &[PinCase]) -> ObligationResult<()> {
    if WATCHED_MUTATION_CASES.len() != 12 {
        return Err(obligation_error(
            "obligations watched-mutation inventory must contain 12 seams",
        ));
    }
    let mut labels: HashSet<&str> = cases.iter().map(|case| case.label.as_str()).collect();
    labels.insert("finding-reader counterfactual");
    let required: BTreeSet<&str> = WATCHED_MUTATION_CASES
        .iter()
        .flat_map(|(_, labels)| labels.iter().copied())
        .collect();
    let missing: Vec<_> = required
        .into_iter()
        .filter(|label| !labels.contains(label))
        .collect();
    if !missing.is_empty() {
        return Err(obligation_error(format!(
            "obligations watched-mutation cases missing: {missing:?}"
        )));
    }
    Ok(())
}

fn append_pin_case(lines: &mut Vec<String>, case: &PinCase) {
    append_facts(lines, &case.facts);
    for query in &case.queries {
        append_query(lines, query);
    }
}

fn render_obligations_pins(cases: &[PinCase]) -> ObligationResult<String> {
    validate_watched_mutation_cases(cases)?;
    let mut lines = vec![
        "# Supplied records establish bounded legal effects only.".to_owned(),
        "# They prove no receipt, action, delivery, remedy, recurrence control, or institutional liveness."
            .to_owned(),
        String::new(),
    ];
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

#[derive(Clone, Debug)]
struct RenderedArtifact {
    path: &'static str,
    text: String,
}

fn rendered_artifacts(
    snapshot: &SourceSnapshot,
    cases: &[PinCase],
) -> ObligationResult<Vec<RenderedArtifact>> {
    Ok(vec![
        RenderedArtifact {
            path: OBLIGATIONS_PINS_PATH,
            text: render_obligations_pins(cases)?,
        },
        RenderedArtifact {
            path: INDEPENDENCE_CF_PATH,
            text: render_independence_counterfactual(snapshot.constitution())?,
        },
        RenderedArtifact {
            path: "new-book-plans/counterfactual/no-obligations-independent-source-review.pins.nibli",
            text: render_independence_pins()?,
        },
        RenderedArtifact {
            path: SOURCE_CF_PATH,
            text: render_source_counterfactual(snapshot.constitution())?,
        },
        RenderedArtifact {
            path: "new-book-plans/counterfactual/no-obligations-source.pins.nibli",
            text: render_source_pins()?,
        },
        RenderedArtifact {
            path: READER_CF_PATH,
            text: render_reader_counterfactual(snapshot.constitution())?,
        },
        RenderedArtifact {
            path: "new-book-plans/counterfactual/no-obligations-finding-reader.pins.nibli",
            text: render_reader_pins()?,
        },
    ])
}

fn render_formal_block() -> String {
    let mut lines = vec![
        "# Generated and exact-owned by new-book-plans/21-obligations.py.".to_owned(),
        "# Every downstream effect repeats the raw current source and origin join.".to_owned(),
        "# Formal conclusions prove no receipt, action, delivery, remedy, or liveness.".to_owned(),
    ];
    lines.extend(formal_rules().iter().cloned());
    lines.join("\n")
}

fn pin_count(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("? ")).count()
}

fn artifact_counts(artifacts: &[RenderedArtifact]) -> (usize, usize, usize, usize) {
    (
        pin_count(&artifacts[0].text),
        pin_count(&artifacts[2].text),
        pin_count(&artifacts[4].text),
        pin_count(&artifacts[6].text),
    )
}

fn render_pin_case(case: &PinCase) -> ObligationResult<String> {
    let mut lines = Vec::new();
    append_pin_case(&mut lines, case);
    finalize_pins(&format!("{MAIN_HEADER}: fresh case {}", case.label), &lines)
}

fn find_payload(path: &Path, data: &[u8], binding: ProsePayload) -> ObligationResult<Vec<u8>> {
    let mut starts = vec![0];
    starts.extend(
        data.iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'\n')
            .map(|(index, _)| index + 1),
    );
    let mut found = None;
    let mut matches = 0;
    for start in starts {
        let Some(payload) = data.get(start..start.saturating_add(binding.length)) else {
            continue;
        };
        if sha256(payload) == binding.digest {
            matches += 1;
            found = Some(payload.to_vec());
        }
    }
    if matches != 1 {
        return Err(obligation_error(format!(
            "approved prose payload {} occurs {matches} times in {}",
            binding.digest,
            path.display()
        )));
    }
    Ok(found.expect("one payload was counted"))
}

fn aggregate_payload_digest(payloads: &[Vec<u8>]) -> String {
    let total_len =
        payloads.iter().map(Vec::len).sum::<usize>() + payloads.len().saturating_sub(1) * 2;
    let mut aggregate = Vec::with_capacity(total_len);
    for (index, payload) in payloads.iter().enumerate() {
        if index != 0 {
            aggregate.extend_from_slice(b"\n\n");
        }
        aggregate.extend_from_slice(payload);
    }
    sha256(aggregate)
}

fn validate_prose_payloads(
    context: &Context,
    cancellation: Option<&CancellationToken>,
) -> ObligationResult<()> {
    ensure_execution_active(cancellation)?;
    let mut files: BTreeMap<&str, Vec<u8>> = BTreeMap::new();
    for binding in PROSE_PAYLOADS
        .iter()
        .chain(DELIVERY_PROSE_PAYLOADS.iter())
        .chain(ECONOMIC_PROSE_PAYLOADS.iter())
    {
        ensure_execution_active(cancellation)?;
        if !files.contains_key(binding.path) {
            let path = context.path(binding.path);
            let bytes = std::fs::read(&path).map_err(|error| {
                obligation_error(format!("cannot read {}: {error}", path.display()))
            })?;
            files.insert(binding.path, bytes);
        }
    }
    ensure_execution_active(cancellation)?;
    let retained: Vec<Vec<u8>> = PROSE_PAYLOADS
        .iter()
        .map(|binding| {
            let path = context.path(binding.path);
            find_payload(&path, &files[binding.path], *binding)
        })
        .collect::<ObligationResult<_>>()?;
    if aggregate_payload_digest(&retained) != PROSE_AGGREGATE_SHA256 {
        return Err(obligation_error(
            "retained OBL-B1-v1 aggregate prose digest changed",
        ));
    }
    ensure_execution_active(cancellation)?;
    let delivery: Vec<Vec<u8>> = DELIVERY_PROSE_PAYLOADS
        .iter()
        .map(|binding| {
            let path = context.path(binding.path);
            find_payload(&path, &files[binding.path], *binding)
        })
        .collect::<ObligationResult<_>>()?;
    if aggregate_payload_digest(&delivery) != DELIVERY_PROSE_AGGREGATE_SHA256 {
        return Err(obligation_error(
            "retained DLV-B1-v1 aggregate prose digest changed",
        ));
    }
    ensure_execution_active(cancellation)?;
    let economic: Vec<Vec<u8>> = ECONOMIC_PROSE_PAYLOADS
        .iter()
        .map(|binding| {
            let path = context.path(binding.path);
            find_payload(&path, &files[binding.path], *binding)
        })
        .collect::<ObligationResult<_>>()?;
    if aggregate_payload_digest(&economic) != ECONOMIC_PROSE_AGGREGATE_SHA256 {
        return Err(obligation_error(
            "ECON-B1-v1 aggregate prose digest changed",
        ));
    }
    Ok(())
}

fn check_inner(
    context: &Context,
    snapshot: &SourceSnapshot,
    cancellation: Option<&CancellationToken>,
) -> ObligationResult<(CheckReport, Vec<PinCase>, Vec<RenderedArtifact>)> {
    ensure_execution_active(cancellation)?;
    let statements = validate_formal_surface(snapshot.constitution())?;
    ensure_execution_active(cancellation)?;
    validate_prose_payloads(context, cancellation)?;
    ensure_execution_active(cancellation)?;
    let cases = main_pin_cases(snapshot)?;
    ensure_execution_active(cancellation)?;
    let artifacts = rendered_artifacts(snapshot, &cases)?;
    for artifact in &artifacts {
        ensure_execution_active(cancellation)?;
        let path = context.path(artifact.path);
        let current = std::fs::read(&path).map_err(|_| {
            obligation_error(format!(
                "obligations artifact differs from renderer: {}",
                path.display()
            ))
        })?;
        if current != artifact.text.as_bytes() {
            return Err(obligation_error(format!(
                "obligations artifact differs from renderer: {}",
                path.display()
            )));
        }
    }
    let (main_pins, independence_pins, source_pins, reader_pins) = artifact_counts(&artifacts);
    Ok((
        CheckReport {
            effects: EFFECTS.len(),
            exact_statements: statements.len(),
            main_pins,
            independence_pins,
            source_pins,
            reader_pins,
        },
        cases,
        artifacts,
    ))
}

pub(crate) fn check(context: &Context, snapshot: &SourceSnapshot) -> Result<CheckReport, Error> {
    check_inner(context, snapshot, None)
        .map(|(report, _, _)| report)
        .map_err(obligations_error)
}

pub(crate) fn check_and_prepare_execution(
    context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<(CheckReport, ExecutionPlan), Error> {
    let (report, cases, artifacts) =
        check_inner(context, snapshot, None).map_err(obligations_error)?;
    let tasks = execution_tasks(snapshot, &cases, &artifacts, None).map_err(obligations_error)?;
    Ok((
        report,
        ExecutionPlan {
            tasks,
            cases: cases.len(),
            counterfactual_suites: 3,
        },
    ))
}

pub(crate) fn check_consumers(
    _context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<String, Error> {
    validate_consumer_allowlist_inner(snapshot.constitution(), true).map_err(obligations_error)?;
    Ok(
        "obliged consumer allowlist: one family-owned typed reader bridge; outside-reader mutation rejected"
            .to_owned(),
    )
}

pub(crate) fn fingerprints(
    _context: &Context,
    _snapshot: &SourceSnapshot,
) -> Result<String, Error> {
    let mut output = String::new();
    for statement in formal_statements() {
        let _ = writeln!(output, "{}", statement_id(statement));
    }
    Ok(output)
}

pub(crate) fn write_artifacts(
    context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<Vec<String>, Error> {
    let result = (|| -> ObligationResult<Vec<String>> {
        let updated_constitution = replace_block(
            snapshot.constitution(),
            RULES_BEGIN,
            RULES_END,
            &render_formal_block(),
        )?;
        std::fs::write(
            context.path(CONSTITUTION_PATH),
            updated_constitution.as_bytes(),
        )
        .map_err(|error| obligation_error(error.to_string()))?;
        let updated = SourceSnapshot {
            constitution: Arc::from(updated_constitution),
            protected_claim_refs: Arc::clone(&snapshot.protected_claim_refs),
        };
        let cases = main_pin_cases(&updated)?;
        let artifacts = rendered_artifacts(&updated, &cases)?;
        let mut messages = Vec::new();
        for artifact in artifacts {
            let path = context.path(artifact.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| obligation_error(error.to_string()))?;
            }
            std::fs::write(&path, artifact.text.as_bytes())
                .map_err(|error| obligation_error(error.to_string()))?;
            let installed =
                std::fs::read(&path).map_err(|error| obligation_error(error.to_string()))?;
            if installed != artifact.text.as_bytes() {
                return Err(obligation_error(format!(
                    "obligations artifact write drifted: {}",
                    path.display()
                )));
            }
            messages.push(format!("wrote {}", artifact.path));
        }
        Ok(messages)
    })();
    result.map_err(obligations_error)
}

#[derive(Clone)]
struct ExecutionTask {
    suite_label: String,
    kb_name: String,
    kb: Arc<str>,
    pin_files: Vec<ExecutionPin>,
}

#[derive(Clone)]
struct ExecutionPin {
    label: String,
    pin_name: String,
    pins: Arc<str>,
    pin_count: usize,
}

fn execution_tasks(
    snapshot: &SourceSnapshot,
    cases: &[PinCase],
    artifacts: &[RenderedArtifact],
    cancellation: Option<&CancellationToken>,
) -> ObligationResult<Vec<ExecutionTask>> {
    ensure_execution_active(cancellation)?;
    let mut main_pins = Vec::with_capacity(cases.len());
    for (index, case) in cases.iter().enumerate() {
        ensure_execution_active(cancellation)?;
        let pin_name = format!("obligations-case-{:03}.pins.nibli", index + 1);
        main_pins.push(ExecutionPin {
            label: format!("fresh case {:03}: {}", index + 1, case.label),
            pin_name,
            pins: Arc::from(render_pin_case(case)?),
            pin_count: case.queries.len(),
        });
    }
    let mut tasks = vec![ExecutionTask {
        suite_label: "fresh obligations cases".to_owned(),
        kb_name: CONSTITUTION_PATH.to_owned(),
        kb: Arc::clone(&snapshot.constitution),
        pin_files: main_pins,
    }];
    for (label, kb_index, pins_index) in [
        ("independence counterfactual", 1, 2),
        ("source-removal counterfactual", 3, 4),
        ("finding-reader counterfactual", 5, 6),
    ] {
        ensure_execution_active(cancellation)?;
        tasks.push(ExecutionTask {
            suite_label: label.to_owned(),
            kb_name: artifacts[kb_index].path.to_owned(),
            kb: Arc::from(artifacts[kb_index].text.clone()),
            pin_files: vec![ExecutionPin {
                label: label.to_owned(),
                pin_name: artifacts[pins_index].path.to_owned(),
                pins: Arc::from(artifacts[pins_index].text.clone()),
                pin_count: pin_count(&artifacts[pins_index].text),
            }],
        });
    }
    Ok(tasks)
}

fn run_execution_task(
    task: &ExecutionTask,
    cancellation: Option<&CancellationToken>,
) -> ObligationResult<Vec<String>> {
    ensure_execution_active(cancellation)?;
    let pin_files: Vec<_> = task
        .pin_files
        .iter()
        .map(|file| LoadedSource::new(&file.pin_name, &file.pins))
        .collect();
    let prepared = cancellation.map_or_else(
        || PreparedPinEngine::new(&[LoadedSource::new(&task.kb_name, &task.kb)]),
        |cancellation| {
            PreparedPinEngine::new_cancellable(
                &[LoadedSource::new(&task.kb_name, &task.kb)],
                cancellation.flag(),
            )
        },
    );
    let output = prepared.run_files(
        &pin_files,
        PinOptions {
            allow_shell: false,
            working_directory: None,
            cancellation,
        },
    );
    ensure_execution_active(cancellation)?;
    let combined = format!("{}{}", output.stdout, output.stderr);
    if output.exit_code != pin::EXIT_OK {
        return Err(obligation_error(format!(
            "{} failed with exit {}\n{}",
            task.suite_label,
            output.exit_code,
            combined.trim()
        )));
    }
    if output.files.len() != task.pin_files.len() {
        return Err(obligation_error(format!(
            "{} returned {} file reports for {} pin files",
            task.suite_label,
            output.files.len(),
            task.pin_files.len()
        )));
    }
    task.pin_files
        .iter()
        .zip(&output.files)
        .map(|(file, actual)| {
            if actual.display_name != file.pin_name
                || actual.pins != file.pin_count
                || actual.defects != 0
                || actual.findings != 0
                || actual.resolved != 0
                || actual.harness != 0
            {
                return Err(obligation_error(format!(
                    "{} produced an unexpected report for {}: {:?}",
                    task.suite_label, file.pin_name, actual
                )));
            }
            Ok(format!(
                "obligations execute: {}: nibli-pin: PASS — {} pins",
                file.label, actual.pins
            ))
        })
        .collect()
}

pub(crate) fn execute(
    context: &Context,
    snapshot: &SourceSnapshot,
) -> Result<ExecutionReport, Error> {
    let (_, plan) = check_and_prepare_execution(context, snapshot)?;
    execute_plan_inner(&plan, None)
}

pub(crate) fn execute_with_cancellation(
    context: &Context,
    snapshot: &SourceSnapshot,
    cancellation: CancellationToken,
) -> Result<ExecutionReport, Error> {
    ensure_execution_active(Some(&cancellation)).map_err(obligations_error)?;
    let (_, cases, artifacts) =
        check_inner(context, snapshot, Some(&cancellation)).map_err(obligations_error)?;
    let tasks = execution_tasks(snapshot, &cases, &artifacts, Some(&cancellation))
        .map_err(obligations_error)?;
    let plan = ExecutionPlan {
        tasks,
        cases: cases.len(),
        counterfactual_suites: 3,
    };
    execute_plan_inner(&plan, Some(&cancellation))
}

pub(crate) fn execute_plan_with_cancellation(
    plan: &ExecutionPlan,
    cancellation: CancellationToken,
) -> Result<ExecutionReport, Error> {
    execute_plan_inner(plan, Some(&cancellation))
}

fn execute_plan_inner(
    plan: &ExecutionPlan,
    cancellation: Option<&CancellationToken>,
) -> Result<ExecutionReport, Error> {
    ensure_execution_active(cancellation).map_err(obligations_error)?;
    let mut results = Vec::with_capacity(plan.cases + plan.counterfactual_suites);
    for task in &plan.tasks {
        ensure_execution_active(cancellation).map_err(obligations_error)?;
        results.extend(run_execution_task(task, cancellation).map_err(obligations_error)?);
    }
    Ok(ExecutionReport {
        cases: plan.cases,
        counterfactual_suites: plan.counterfactual_suites,
        lines: results,
    })
}

#[cfg(test)]
pub(crate) fn synthetic_execution_plan_for_suite() -> ExecutionPlan {
    ExecutionPlan {
        tasks: vec![ExecutionTask {
            suite_label: "captured obligations fixture".to_owned(),
            kb_name: "captured-obligations.nibli".to_owned(),
            kb: Arc::from("person(Ara).\n"),
            pin_files: vec![ExecutionPin {
                label: "captured case".to_owned(),
                pin_name: "captured-obligations.pins.nibli".to_owned(),
                pins: Arc::from("? person(Ara).\n# => TRUE\n:expect-pins 1\n"),
                pin_count: 1,
            }],
        }],
        cases: 1,
        counterfactual_suites: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::Instant;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    fn snapshot(context: &Context) -> SourceSnapshot {
        load_snapshot(context).expect("load obligations sources")
    }

    #[test]
    fn pre_cancelled_execution_returns_before_reading_empty_context() {
        let temporary = tempfile::tempdir().expect("temporary empty context");
        let context = Context::from_test_root(temporary.path().to_path_buf());
        let snapshot = SourceSnapshot {
            constitution: Arc::from(""),
            protected_claim_refs: Arc::from(Vec::<String>::new()),
        };
        let cancellation = CancellationToken::new();
        assert!(cancellation.cancel());

        let error = execute_with_cancellation(&context, &snapshot, cancellation)
            .expect_err("pre-cancelled execution must stop before any context read");
        assert_eq!(
            error.to_string(),
            "obligations: obligations execution cancelled"
        );
    }

    #[test]
    fn captured_execution_plan_runs_without_a_repository_context() {
        let plan = synthetic_execution_plan_for_suite();

        let report = execute_plan_with_cancellation(&plan, CancellationToken::new())
            .expect("captured obligations plan");
        assert_eq!(report.cases, 1);
        assert_eq!(report.counterfactual_suites, 0);
        assert_eq!(
            report.lines,
            ["obligations execute: captured case: nibli-pin: PASS — 1 pins"]
        );
    }

    #[test]
    fn formal_surface_and_statement_digest_match_the_live_constitution() {
        let context = context();
        let snapshot = snapshot(&context);
        let actual = enacted_statements(snapshot.constitution()).expect("extract formal surface");
        let expected: Vec<_> = formal_statements().collect();
        if actual != expected {
            let index = actual
                .iter()
                .zip(&expected)
                .position(|(left, right)| left != right)
                .unwrap_or_else(|| actual.len().min(expected.len()));
            eprintln!(
                "first formal difference at {index}; actual statements={}, expected statements={}",
                actual.len(),
                expected.len()
            );
            if let (Some(left), Some(right)) = (actual.get(index), expected.get(index)) {
                let byte = left
                    .bytes()
                    .zip(right.bytes())
                    .position(|(a, b)| a != b)
                    .unwrap_or_else(|| left.len().min(right.len()));
                eprintln!(
                    "byte {byte}; actual len={} expected len={}\nactual: {:?}\nexpected: {:?}",
                    left.len(),
                    right.len(),
                    &left[byte.saturating_sub(100)..left.len().min(byte + 300)],
                    &right[byte.saturating_sub(100)..right.len().min(byte + 300)]
                );
            }
        }
        let statements = validate_formal_surface(snapshot.constitution())
            .expect("validate generated formal surface");
        assert_eq!(actual, expected);
        assert_eq!(formal_rules().len(), 64);
        assert_eq!(statements.len(), 65);
        assert_eq!(statement_id(statements[0]), statement_id(DERIVED_STATEMENT));
        let replaced = replace_block(
            snapshot.constitution(),
            RULES_BEGIN,
            RULES_END,
            &render_formal_block(),
        )
        .expect("render formal block into constitution");
        assert_eq!(replaced, snapshot.constitution());
    }

    #[test]
    fn every_rendered_artifact_is_byte_identical_to_the_live_artifact() {
        let context = context();
        let snapshot = snapshot(&context);
        let cases = main_pin_cases(&snapshot).expect("build pin cases");
        let artifacts = rendered_artifacts(&snapshot, &cases).expect("render artifacts");
        assert_eq!(artifacts.len(), 7);
        for artifact in artifacts {
            let actual = std::fs::read(context.path(artifact.path)).expect("read live artifact");
            assert_eq!(
                actual,
                artifact.text.as_bytes(),
                "artifact differs: {}",
                artifact.path
            );
        }
    }

    #[test]
    fn case_and_pin_counts_preserve_the_execution_contract() {
        let context = context();
        let snapshot = snapshot(&context);
        let cases = main_pin_cases(&snapshot).expect("build pin cases");
        validate_watched_mutation_cases(&cases).expect("watched cases");
        assert_eq!(cases.len(), 122);
        assert_eq!(
            cases.iter().map(|case| case.queries.len()).sum::<usize>(),
            142
        );
        let artifacts = rendered_artifacts(&snapshot, &cases).expect("render artifacts");
        assert_eq!(artifact_counts(&artifacts), (142, 25, 26, 28));
        let tasks = execution_tasks(&snapshot, &cases, &artifacts, None).expect("execution tasks");
        assert_eq!(tasks.len(), 4);
        assert_eq!(tasks[0].pin_files.len(), 122);
        assert!(tasks[1..].iter().all(|task| task.pin_files.len() == 1));
        assert!(Arc::ptr_eq(&tasks[0].kb, &snapshot.constitution));
    }

    #[test]
    fn check_and_consumer_messages_match_the_python_interface() {
        let context = context();
        let snapshot = snapshot(&context);
        let report = check(&context, &snapshot).expect("check obligations");
        assert_eq!(
            report.to_string(),
            "obligations: PASS - 25 effects, 65 exact statements, 142 main pins, \
             25/26/28 counterfactual pins, 11 retained OBL-B1-v1, 8 retained \
             DLV-B1-v1, and 3 successor ECON-B1-v1 byte-exact prose payloads, \
             12 watched mutation seams, and one exact obliged consumer"
        );
        assert_eq!(
            check_consumers(&context, &snapshot).expect("check consumers"),
            "obliged consumer allowlist: one family-owned typed reader bridge; \
             outside-reader mutation rejected"
        );

        let mut mutated = snapshot.constitution().to_owned();
        mutated
            .push_str("\nall $x: obliged(Review, $x) -> complete($x, OutsideObligationsReader).\n");
        let error = validate_consumer_allowlist_inner(&mutated, false)
            .expect_err("outside reader must fail");
        assert_eq!(
            error.to_string(),
            "obliged consumers differ from the one family-owned typed reader bridge"
        );
    }

    #[test]
    fn fingerprints_have_the_reviewed_count_and_aggregate() {
        let context = context();
        let snapshot = snapshot(&context);
        let output = fingerprints(&context, &snapshot).expect("fingerprints");
        assert_eq!(output.lines().count(), 65);
        assert_eq!(
            sha256(output.trim_end().as_bytes()),
            OBLIGATION_STATEMENT_SET_SHA256
        );
    }

    #[test]
    #[ignore = "loads the 5.6 MB live constitution into two independent engines"]
    fn live_first_case_matches_the_sibling_pin_runner_byte_for_byte() {
        let context = context();
        let snapshot = snapshot(&context);
        let sibling = context.path("../nibli/target/release/nibli-pin");
        if !sibling.is_file() {
            eprintln!("skipping because {} is not built", sibling.display());
            return;
        }
        let cases = main_pin_cases(&snapshot).expect("build pin cases");
        let pins = render_pin_case(&cases[0]).expect("render first case");
        let temporary = tempfile::tempdir().expect("temporary directory");
        let pin_path = temporary.path().join("obligations-case-001.pins.nibli");
        std::fs::write(&pin_path, &pins).expect("write pin file");

        let started = Instant::now();
        let sibling_output = Command::new(&sibling)
            .args(["--kb", CONSTITUTION_PATH, "--allow-shell"])
            .arg(&pin_path)
            .current_dir(context.root())
            .output()
            .expect("run sibling pin runner");
        let in_process = pin::run_pin_files(
            &[LoadedSource::new(
                CONSTITUTION_PATH,
                snapshot.constitution(),
            )],
            &[LoadedSource::new("obligations-case-001.pins.nibli", &pins)],
            PinOptions {
                allow_shell: true,
                working_directory: Some(context.root()),
                cancellation: None,
            },
        );
        eprintln!("live parity completed in {:.3?}", started.elapsed());

        assert_eq!(
            sibling_output.status.code(),
            Some(i32::from(in_process.exit_code))
        );
        assert_eq!(sibling_output.stdout, in_process.stdout.as_bytes());
        assert_eq!(sibling_output.stderr, in_process.stderr.as_bytes());
        assert_eq!(in_process.pins, cases[0].queries.len());
    }

    #[test]
    #[ignore = "manual Rust-only live performance check"]
    fn live_first_case_runs_in_process_with_the_expected_report() {
        let context = context();
        let snapshot = snapshot(&context);
        let cases = main_pin_cases(&snapshot).expect("build pin cases");
        let pins = render_pin_case(&cases[0]).expect("render first case");
        let started = Instant::now();
        let output = pin::run_pin_files(
            &[LoadedSource::new(
                CONSTITUTION_PATH,
                snapshot.constitution(),
            )],
            &[LoadedSource::new("obligations-case-001.pins.nibli", &pins)],
            PinOptions {
                allow_shell: true,
                working_directory: Some(context.root()),
                cancellation: None,
            },
        );
        eprintln!("Rust-only live case completed in {:.3?}", started.elapsed());

        assert_eq!(
            output.exit_code,
            pin::EXIT_OK,
            "{}{}",
            output.stdout,
            output.stderr
        );
        assert_eq!(output.pins, cases[0].queries.len());
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].pins, cases[0].queries.len());
    }
}
