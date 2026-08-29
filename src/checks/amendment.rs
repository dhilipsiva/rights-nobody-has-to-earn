// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native bounded amendment-semantics audit.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::checks::assertion_surface;
use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};

pub(crate) const STEP_NAME: &str = "amendment semantics";

const DEFAULT_SOURCE: &str = "new-book-plans/amendment-semantics-audit.json";
const DEFAULT_KB: &str = "new-book-plans/constitution.nibli";
const DEFAULT_LEDGER: &str = "new-book-plans/assertion-surface-contracts.json";
const DEFAULT_ASSURANCE: &str = "new-book-plans/record-integrity-assurance-case.json";
const DEFAULT_OUTPUT: &str = "new-book-plans/amendment-semantics-audit.md";
const REVIEWED_TIMEOUT_SECONDS: u64 = 60;
const SABOTAGE_FINAL_SUMMARY: &str = "nibli-pin: 1 FINDING(S) (exit 1)";
const SURFACE_SEAM_FRAGMENT: &str =
    "uses relations absent from engine inventory or alias contract: rich";

const REQUIRED_CASE_IDS: [&str; 9] = [
    "AS-01", "AS-02", "AS-03", "AS-04", "AS-05", "AS-06", "AS-07", "AS-08", "AS-09",
];
const LABEL_VERDICTS: [&str; 3] = ["blocked_label", "passing_label", "no_amendment_label"];
const EFFECT_KINDS: [&str; 5] = [
    "unchanged_control",
    "direct_deletion",
    "direct_replacement",
    "ground_label_addition",
    "vocabulary_widening",
];
const MATCH_POSTURES: [&str; 3] = ["true", "false", "not_applicable"];
const MUTATION_OPS: [&str; 3] = ["delete_exact", "replace_exact", "append_exact"];
const SURFACE_EXPECTATIONS: [&str; 2] = ["not_run", "reject_unreconciled_inventory_relation"];
const NARROWNESS_CLASSIFICATIONS: [&str; 2] = ["preserved_but_scoped", "revised_and_scoped"];
const REQUIRED_NARROWNESS_REFERENCES: [&str; 7] = [
    "book-1/01-what-counts-as-evidence.md::That is a check on declared names, not protection of their meaning.",
    "book-1/08-what-you-are-owed.md::At the floor layer, old hostile rules remain loadable. That is no longer the",
    "book-1/12-changing-the-rules.md::The recorded label verdict is real.",
    "book-1/15-the-five-joints.md::The consequential surfaces that remain are the constitutional source",
    "book-1/15-the-five-joints.md::narrow mechanism still blocks punishment for absence rather than the",
    "book-1/15-the-five-joints.md::That review is not constitutional approval",
    "book-1/method.md::the candidate edits are applied manually by the audit, not",
];

const FLOOR_LINE: &str = "entitled(every person, event { eats() }).\n";
const THREADED_FLOOR_LINE: &str = "entitled(every person, event { eats($x) }).\n";
const TARGETLESS_APPEND: &str = "\n# Amendment-semantics candidate additions (generated, not enacted).\nsuggest(Assembly, Amend_Targetless).\nratifies(Electorate, Amend_Targetless).\n";
const MISDIRECT_APPEND: &str = "\n# Amendment-semantics candidate additions (generated, not enacted).\nsuggest(Assembly, Amend_Misdirect).\nadjust(Amend_Misdirect, Art_Mint).\nratifies(Electorate, Amend_Misdirect).\n";
const SYNTAX_APPEND: &str = "\n# Amendment-semantics candidate additions (generated, not enacted).\nsuggest(Assembly, Amend_Syntax).\nadjust(Amend_Syntax, Art_Syntax).\nratifies(Electorate, Amend_Syntax).\n";
const ART_FLOOR_LABEL_APPEND: &str = "\n# Amendment-semantics candidate addition (generated, not enacted).\nadjust(Amend_Mint, Art_Floor).\n";
const ADMITS_ANCHOR: &str =
    "admits(\"deceive\").    admits(\"family\").     admits(\"forgive\").\n";
const ADMITS_RICH: &str =
    "admits(\"deceive\").    admits(\"family\").     admits(\"forgive\").\nadmits(\"rich\").\n";
const RICH_FACT_APPEND: &str = "\n# Amendment-semantics candidate facts (generated, not enacted).\npermanent(Art_Evidence).\nrich(Adam).\n";

#[derive(Clone, Copy)]
struct MutationShape {
    op: &'static str,
    before: &'static str,
    after: &'static str,
}

const EMPTY_SHAPE: &[MutationShape] = &[];
const AS03_SHAPE: &[MutationShape] = &[
    MutationShape {
        op: "delete_exact",
        before: FLOOR_LINE,
        after: "",
    },
    MutationShape {
        op: "append_exact",
        before: "",
        after: TARGETLESS_APPEND,
    },
];
const AS04_SHAPE: &[MutationShape] = &[
    MutationShape {
        op: "delete_exact",
        before: FLOOR_LINE,
        after: "",
    },
    MutationShape {
        op: "append_exact",
        before: "",
        after: MISDIRECT_APPEND,
    },
];
const AS05_SHAPE: &[MutationShape] = &[
    MutationShape {
        op: "replace_exact",
        before: FLOOR_LINE,
        after: THREADED_FLOOR_LINE,
    },
    MutationShape {
        op: "append_exact",
        before: "",
        after: SYNTAX_APPEND,
    },
];
const AS06_SHAPE: &[MutationShape] = &[MutationShape {
    op: "append_exact",
    before: "",
    after: ART_FLOOR_LABEL_APPEND,
}];
const AS08_SHAPE: &[MutationShape] = &[
    MutationShape {
        op: "replace_exact",
        before: ADMITS_ANCHOR,
        after: ADMITS_RICH,
    },
    MutationShape {
        op: "append_exact",
        before: "",
        after: RICH_FACT_APPEND,
    },
];
const AS09_SHAPE: &[MutationShape] = &[MutationShape {
    op: "delete_exact",
    before: FLOOR_LINE,
    after: "",
}];

fn required_mutation_shape(identifier: &str) -> &'static [MutationShape] {
    match identifier {
        "AS-03" => AS03_SHAPE,
        "AS-04" => AS04_SHAPE,
        "AS-05" => AS05_SHAPE,
        "AS-06" => AS06_SHAPE,
        "AS-08" => AS08_SHAPE,
        "AS-09" => AS09_SHAPE,
        _ => EMPTY_SHAPE,
    }
}

fn required_effect_kind(identifier: &str) -> &'static str {
    match identifier {
        "AS-03" | "AS-04" | "AS-09" => "direct_deletion",
        "AS-05" => "direct_replacement",
        "AS-06" => "ground_label_addition",
        "AS-08" => "vocabulary_widening",
        _ => "unchanged_control",
    }
}

fn required_label_manifest(
    identifier: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match identifier {
        "AS-01" => (
            "Amend_Floor",
            "Art_Floor",
            "blocked_label",
            "not_applicable",
        ),
        "AS-02" => ("Amend_Mint", "Art_Mint", "passing_label", "not_applicable"),
        "AS-03" => ("Amend_Targetless", "none", "passing_label", "false"),
        "AS-04" => ("Amend_Misdirect", "Art_Mint", "passing_label", "false"),
        "AS-05" => ("Amend_Syntax", "Art_Syntax", "passing_label", "false"),
        "AS-06" => (
            "Amend_Mint",
            "Art_Mint and Art_Floor",
            "blocked_label",
            "not_applicable",
        ),
        "AS-07" | "AS-08" => ("none", "none", "no_amendment_label", "not_applicable"),
        "AS-09" => ("Amend_Floor", "Art_Floor", "blocked_label", "true"),
        _ => ("", "", "", ""),
    }
}

fn semantic_sentinels() -> [(&'static str, &'static str, &'static str); 19] {
    [
        ("AS-01", "false(Amend_Floor)", "TRUE"),
        ("AS-01", "become(Amend_Floor, Law)", "FALSE"),
        ("AS-02", "false(Amend_Mint)", "FALSE"),
        ("AS-02", "become(Amend_Mint, Law)", "TRUE"),
        ("AS-03", "become(Amend_Targetless, Law)", "TRUE"),
        ("AS-03", "adjust(Amend_Targetless, $t)", "FALSE"),
        ("AS-03", "entitled(Adam, event { eats() })", "FALSE"),
        ("AS-04", "adjust(Amend_Misdirect, Art_Mint)", "TRUE"),
        ("AS-04", "adjust(Amend_Misdirect, Art_Floor)", "FALSE"),
        ("AS-04", "become(Amend_Misdirect, Law)", "TRUE"),
        ("AS-05", "entitled(Adam, event { eats() })", "FALSE"),
        ("AS-05", "eats(Adam)", "FALSE"),
        ("AS-06", "false(Amend_Mint)", "TRUE"),
        ("AS-06", "become(Amend_Mint, Law)", "FALSE"),
        ("AS-08", "permanent(Art_Evidence)", "TRUE"),
        ("AS-08", "rich(Adam)", "TRUE"),
        ("AS-09", "false(Amend_Floor)", "TRUE"),
        ("AS-09", "become(Amend_Floor, Law)", "FALSE"),
        ("AS-09", "entitled(Adam, event { eats() })", "FALSE"),
    ]
}

static VARIABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$[a-z][a-z0-9_]*").expect("valid regex"));
static SHA256: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("valid regex"));
static CASE_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^AS-[0-9]{2}$").expect("valid regex"));
static RELATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").expect("valid regex"));
static CONSTANT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][A-Za-z0-9_]*$").expect("valid regex"));
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:tbd|todo|unknown|n/?a|pending|placeholder)$").expect("valid regex")
});

#[derive(Clone, Debug)]
pub(crate) struct Paths {
    pub(crate) source: PathBuf,
    pub(crate) kb: PathBuf,
    pub(crate) ledger: PathBuf,
    pub(crate) assurance: PathBuf,
    pub(crate) output: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            source: PathBuf::from(DEFAULT_SOURCE),
            kb: PathBuf::from(DEFAULT_KB),
            ledger: PathBuf::from(DEFAULT_LEDGER),
            assurance: PathBuf::from(DEFAULT_ASSURANCE),
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExecutionReport {
    pub(crate) cases: usize,
    pub(crate) pins: usize,
    pub(crate) sabotage_controls: usize,
    pub(crate) seam_controls: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
    pub(crate) execution: Option<ExecutionReport>,
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is current; {} structural negative controls pass",
            self.output, self.structural_controls
        )?;
        match &self.execution {
            Some(execution) => write!(
                formatter,
                "; {} isolated cases / {} pins execute; {} sabotage and {} assertion-surface seam pass",
                execution.cases,
                execution.pins,
                execution.sabotage_controls,
                execution.seam_controls
            ),
            None => formatter.write_str("; execution skipped"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenerationReport {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
    pub(crate) execution: Option<ExecutionReport>,
}

impl fmt::Display for GenerationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: regenerated", self.output)?;
        match &self.execution {
            Some(execution) => write!(
                formatter,
                " after {} isolated cases / {} pins",
                execution.cases, execution.pins
            )?,
            None => formatter.write_str(" (structural generation; execution not requested)")?,
        }
        write!(
            formatter,
            "; {} structural negative controls pass",
            self.structural_controls
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FingerprintReport(String);

impl fmt::Display for FingerprintReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentSource {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    evidence_role: String,
    subprocess_timeout_seconds: u64,
    constitution_sha256: String,
    assertion_surface_contracts_sha256: String,
    record_integrity_assurance_case_sha256: String,
    label_verdict_meanings: LabelVerdictMeanings,
    limits: AmendmentLimits,
    required_cases: Vec<String>,
    cases: Vec<AmendmentCase>,
    narrowness_impacts: Vec<NarrownessImpact>,
    acceptance_result: AcceptanceResult,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelVerdictMeanings {
    blocked_label: String,
    passing_label: String,
    no_amendment_label: String,
}

impl LabelVerdictMeanings {
    fn get(&self, verdict: &str) -> Option<&str> {
        match verdict {
            "blocked_label" => Some(&self.blocked_label),
            "passing_label" => Some(&self.passing_label),
            "no_amendment_label" => Some(&self.no_amendment_label),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentLimits {
    manual_delta: String,
    semantic_completeness: String,
    source_history: String,
    engine_scope: String,
    no_new_gate: String,
}

impl AmendmentLimits {
    fn in_source_order(&self) -> [(&'static str, &str); 5] {
        [
            ("manual_delta", &self.manual_delta),
            ("semantic_completeness", &self.semantic_completeness),
            ("source_history", &self.source_history),
            ("engine_scope", &self.engine_scope),
            ("no_new_gate", &self.no_new_gate),
        ]
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmendmentCase {
    id: String,
    title: String,
    declared_label: DeclaredLabel,
    source_effect: SourceEffect,
    mutations: Vec<Mutation>,
    mutation_sha256: String,
    expected_source_sha256: String,
    source_assertions: Vec<SourceAssertion>,
    steps: Vec<Step>,
    assertion_surface_expectation: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredLabel {
    amendment: String,
    declared_target: String,
    verdict: String,
    summary: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEffect {
    kind: String,
    summary: String,
    label_matches_effect: String,
    protected_consequence: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Mutation {
    op: String,
    before: String,
    after: String,
    before_sha256: String,
    after_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceAssertion {
    kind: String,
    relation: String,
    subject: String,
    expected: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Step {
    #[serde(rename = "query")]
    Query {
        expression: String,
        expected: String,
        purpose: String,
    },
    #[serde(rename = "accept")]
    Accept {
        directive: String,
        statement: String,
        error_pattern: String,
        purpose: String,
    },
    #[serde(rename = "refuse")]
    Refuse {
        directive: String,
        statement: String,
        error_pattern: String,
        purpose: String,
    },
}

impl Step {
    fn purpose(&self) -> &str {
        match self {
            Self::Query { purpose, .. }
            | Self::Accept { purpose, .. }
            | Self::Refuse { purpose, .. } => purpose,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NarrownessImpact {
    artifact_ref: String,
    current_claim: String,
    classification: String,
    reason: String,
    future_trigger: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceResult {
    result: String,
    claim: String,
    does_not_establish: Vec<String>,
    remaining_boundary: String,
}

/// Strict projection of the assertion-surface contract.
///
/// Amendment semantics only consumes the premise names, but enumerating every
/// root field keeps this dependency fail-closed when the reviewed ledger shape
/// changes. The unconsumed payloads are deliberately skipped without building
/// a second loosely typed JSON tree.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssertionLedger {
    #[serde(rename = "spdx")]
    _spdx: IgnoredAny,
    #[serde(rename = "schema_version")]
    _schema_version: IgnoredAny,
    #[serde(rename = "aliases")]
    _aliases: IgnoredAny,
    #[serde(rename = "cheapest_harm_metric")]
    _cheapest_harm_metric: IgnoredAny,
    #[serde(rename = "risk_disposition_meanings")]
    _risk_disposition_meanings: IgnoredAny,
    #[serde(rename = "required_semantic_tags")]
    _required_semantic_tags: IgnoredAny,
    #[serde(rename = "additional_writable_channels")]
    _additional_writable_channels: IgnoredAny,
    #[serde(rename = "rules_sha256")]
    _rules_sha256: IgnoredAny,
    #[serde(rename = "facts_sha256")]
    _facts_sha256: IgnoredAny,
    #[serde(rename = "route_fingerprints")]
    _route_fingerprints: IgnoredAny,
    #[serde(rename = "reserved_retired_relations")]
    _reserved_retired_relations: IgnoredAny,
    #[serde(rename = "derived_relations")]
    _derived_relations: IgnoredAny,
    premises: BTreeMap<String, IgnoredAny>,
}

struct Snapshot {
    reviewed: AmendmentSource,
    reviewed_raw: Value,
    kb_bytes: Arc<[u8]>,
    kb_text: Arc<str>,
    ledger: AssertionLedger,
    ledger_raw: Value,
    kb_digest: String,
    ledger_digest: String,
    assurance_digest: String,
    references: BTreeMap<String, Arc<str>>,
    source_relative: String,
    kb_relative: String,
    ledger_relative: String,
    assurance_relative: String,
    output_relative: String,
    output_path: PathBuf,
    input_identities: HashSet<FileIdentity>,
    current_output: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
struct Validated {
    candidates: BTreeMap<String, Arc<str>>,
}

type AuditResult<T> = Result<T, String>;

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_with_paths(context, &Paths::default(), false)
}

pub(crate) fn check_execute(context: &Context) -> Result<Report, Error> {
    check_with_paths(context, &Paths::default(), true)
}

pub(crate) fn check_with_paths(
    context: &Context,
    paths: &Paths,
    execute: bool,
) -> Result<Report, Error> {
    let snapshot = load_snapshot(context, paths, true).map_err(amendment_error)?;
    let validated = validate_source(&snapshot).map_err(amendment_error)?;
    let generated = render(&snapshot).map_err(amendment_error)?;
    let structural_controls = negative_controls(&snapshot).map_err(amendment_error)?;
    let execution = if execute {
        Some(execute_cases(context, &snapshot, &validated).map_err(amendment_error)?)
    } else {
        None
    };
    if snapshot.current_output.as_deref() != Some(generated.as_bytes()) {
        return Err(amendment_error(format!(
            "{} is STALE — rerun without --check",
            snapshot.output_relative
        )));
    }
    Ok(Report {
        output: snapshot.output_relative,
        structural_controls,
        execution,
    })
}

pub(crate) fn generate(context: &Context) -> Result<GenerationReport, Error> {
    generate_with_paths(context, &Paths::default(), false)
}

pub(crate) fn generate_execute(context: &Context) -> Result<GenerationReport, Error> {
    generate_with_paths(context, &Paths::default(), true)
}

pub(crate) fn generate_with_paths(
    context: &Context,
    paths: &Paths,
    execute: bool,
) -> Result<GenerationReport, Error> {
    let snapshot = load_snapshot(context, paths, false).map_err(amendment_error)?;
    let validated = validate_source(&snapshot).map_err(amendment_error)?;
    let generated = render(&snapshot).map_err(amendment_error)?;
    let structural_controls = negative_controls(&snapshot).map_err(amendment_error)?;
    let execution = if execute {
        Some(execute_cases(context, &snapshot, &validated).map_err(amendment_error)?)
    } else {
        None
    };
    write_generated_output(
        &snapshot.output_path,
        generated.as_bytes(),
        &snapshot.input_identities,
    )
    .map_err(amendment_error)?;
    Ok(GenerationReport {
        output: snapshot.output_relative,
        structural_controls,
        execution,
    })
}

pub(crate) fn fingerprints(context: &Context) -> Result<FingerprintReport, Error> {
    fingerprints_with_paths(context, &Paths::default())
}

pub(crate) fn fingerprints_with_paths(
    context: &Context,
    paths: &Paths,
) -> Result<FingerprintReport, Error> {
    let result = (|| -> AuditResult<FingerprintReport> {
        let snapshot = load_snapshot(context, paths, false)?;
        let validated = validate_source(&snapshot)?;
        let mut candidate_source_sha256 = Map::new();
        let mut mutation_sha256 = Map::new();
        for case in &snapshot.reviewed.cases {
            let identifier = &case.id;
            candidate_source_sha256.insert(
                identifier.to_owned(),
                Value::String(sha256(validated.candidates[identifier].as_bytes())),
            );
            mutation_sha256.insert(
                identifier.to_owned(),
                Value::String(case.mutation_sha256.clone()),
            );
        }
        let mut value = serde_json::json!({
            "assertion_surface_contracts_sha256": snapshot.ledger_digest,
            "candidate_source_sha256": candidate_source_sha256,
            "constitution_sha256": snapshot.kb_digest,
            "mutation_sha256": mutation_sha256,
            "record_integrity_assurance_case_sha256": snapshot.assurance_digest,
        });
        sort_json(&mut value);
        let body = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
        Ok(FingerprintReport(body))
    })();
    result.map_err(amendment_error)
}

fn load_snapshot(context: &Context, paths: &Paths, read_output: bool) -> AuditResult<Snapshot> {
    let source_path = resolve_path(context, &paths.source);
    let kb_path = resolve_path(context, &paths.kb);
    let ledger_path = resolve_path(context, &paths.ledger);
    let assurance_path = resolve_path(context, &paths.assurance);
    let output_path = resolve_path(context, &paths.output);

    let source_relative = repo_relative(context.root(), &source_path)?;
    let kb_relative = repo_relative(context.root(), &kb_path)?;
    let ledger_relative = repo_relative(context.root(), &ledger_path)?;
    let assurance_relative = repo_relative(context.root(), &assurance_path)?;
    let output_relative = repo_relative(context.root(), &output_path)?;
    if output_relative != DEFAULT_OUTPUT {
        return Err("--output is fixed to new-book-plans/amendment-semantics-audit.md".into());
    }

    let (source_bytes, source_identity) =
        read_bound_file(&source_path, "amendment-semantics source")?;
    let (kb_bytes, kb_identity) = read_bound_file(&kb_path, "constitution")?;
    let (ledger_bytes, ledger_identity) = read_bound_file(&ledger_path, "assertion ledger")?;
    let (assurance_bytes, assurance_identity) =
        read_bound_file(&assurance_path, "assurance source")?;
    let input_identities = require_distinct_identities(&[
        ("amendment-semantics source", source_identity),
        ("constitution", kb_identity),
        ("assertion ledger", ledger_identity),
        ("assurance source", assurance_identity),
    ])?;
    validate_output_target(&output_path, &input_identities)?;

    let reviewed_raw = parse_json_no_duplicates(&source_bytes)
        .map_err(|error| format!("cannot parse amendment-semantics source: {error}"))?;
    let reviewed = serde_json::from_slice::<AmendmentSource>(&source_bytes)
        .map_err(|error| format!("cannot parse amendment-semantics source: {error}"))?;
    let ledger_raw = parse_json_no_duplicates(&ledger_bytes)
        .map_err(|error| format!("cannot parse assertion ledger: {error}"))?;
    let ledger = serde_json::from_slice::<AssertionLedger>(&ledger_bytes)
        .map_err(|error| format!("cannot parse assertion ledger: {error}"))?;
    let kb_text = decode_constitution_bytes(&kb_bytes, "constitution")?;
    decode_utf8_exact(&assurance_bytes, "assurance source")?;

    let mut references = BTreeMap::new();
    for impact in &reviewed.narrowness_impacts {
        let reference = &impact.artifact_ref;
        if let Some((raw_file, _)) = reference.split_once("::") {
            if references.contains_key(raw_file) {
                continue;
            }
            let path = context.path(raw_file);
            if repo_relative(context.root(), &path).is_ok() && path.is_file() {
                let (body, _) = read_bound_file(&path, "narrowness reference")?;
                references.insert(
                    raw_file.to_owned(),
                    Arc::<str>::from(decode_utf8_exact(&body, "narrowness reference")?),
                );
            }
        }
    }
    let current_output = if read_output {
        Some(read_bound_file(&output_path, "generated report")?.0)
    } else {
        None
    };
    Ok(Snapshot {
        kb_digest: sha256(&kb_bytes),
        ledger_digest: sha256(&ledger_bytes),
        assurance_digest: sha256(&assurance_bytes),
        reviewed,
        reviewed_raw,
        kb_bytes: Arc::from(kb_bytes),
        kb_text: Arc::from(kb_text),
        ledger,
        ledger_raw,
        references,
        source_relative,
        kb_relative,
        ledger_relative,
        assurance_relative,
        output_relative,
        output_path,
        input_identities,
        current_output,
    })
}

fn resolve_path(context: &Context, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        context.path(path)
    }
}

fn repo_relative(root: &Path, path: &Path) -> AuditResult<String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    let resolved = if path.exists() {
        path.canonicalize()
            .map_err(|error| format!("cannot resolve path {}: {error}", path.display()))?
    } else {
        let parent = path
            .parent()
            .ok_or_else(|| format!("path escapes repository: {}", path.display()))?
            .canonicalize()
            .map_err(|error| format!("cannot resolve path {}: {error}", path.display()))?;
        parent.join(
            path.file_name()
                .ok_or_else(|| format!("path escapes repository: {}", path.display()))?,
        )
    };
    resolved
        .strip_prefix(&root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| format!("path escapes repository: {}", path.display()))
}

fn read_bound_file(path: &Path, label: &str) -> AuditResult<(Vec<u8>, FileIdentity)> {
    let mut stream = File::open(path)
        .map_err(|error| format!("cannot read {label} {}: {error}", path.display()))?;
    let before = stream
        .metadata()
        .map_err(|error| format!("cannot read {label} {}: {error}", path.display()))?;
    if !before.is_file() {
        return Err(format!(
            "{label} must be a regular file: {}",
            path.display()
        ));
    }
    let mut value = Vec::with_capacity(before.len() as usize);
    stream
        .read_to_end(&mut value)
        .map_err(|error| format!("cannot read {label} {}: {error}", path.display()))?;
    let after = stream
        .metadata()
        .map_err(|error| format!("cannot read {label} {}: {error}", path.display()))?;
    if metadata_state(&before) != metadata_state(&after) || value.len() as u64 != after.len() {
        return Err(format!("{label} changed while its bound bytes were read"));
    }
    Ok((value, file_identity(&after)))
}

#[cfg(unix)]
fn metadata_state(metadata: &std::fs::Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    use std::os::unix::fs::MetadataExt;
    (
        metadata.dev(),
        metadata.ino(),
        metadata.size(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

#[cfg(unix)]
fn file_identity(metadata: &std::fs::Metadata) -> FileIdentity {
    use std::os::unix::fs::MetadataExt;
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(not(unix))]
fn metadata_state(metadata: &std::fs::Metadata) -> (u64, Option<std::time::SystemTime>) {
    (metadata.len(), metadata.modified().ok())
}

#[cfg(not(unix))]
fn file_identity(metadata: &std::fs::Metadata) -> FileIdentity {
    FileIdentity {
        device: 0,
        inode: metadata.len(),
    }
}

fn require_distinct_identities(
    named: &[(&str, FileIdentity)],
) -> AuditResult<HashSet<FileIdentity>> {
    let mut seen = HashMap::new();
    for (label, identity) in named {
        if let Some(previous) = seen.insert(*identity, *label) {
            return Err(format!(
                "resolved input identity collision: {label} aliases {previous}"
            ));
        }
    }
    Ok(seen.into_keys().collect())
}

fn validate_output_target(path: &Path, inputs: &HashSet<FileIdentity>) -> AuditResult<()> {
    if path.is_symlink() {
        return Err("generated output may not be a symlink".into());
    }
    if !path.exists() {
        return Ok(());
    }
    let metadata = path.metadata().map_err(|error| {
        format!(
            "cannot inspect generated output {}: {error}",
            path.display()
        )
    })?;
    validate_output_details(&metadata, inputs)
}

fn validate_output_details(
    metadata: &std::fs::Metadata,
    inputs: &HashSet<FileIdentity>,
) -> AuditResult<()> {
    #[cfg(unix)]
    let links = {
        use std::os::unix::fs::MetadataExt;
        metadata.nlink()
    };
    #[cfg(not(unix))]
    let links = 1;
    validate_output_characteristics(metadata.is_file(), links, file_identity(metadata), inputs)
}

fn validate_output_characteristics(
    regular: bool,
    links: u64,
    identity: FileIdentity,
    inputs: &HashSet<FileIdentity>,
) -> AuditResult<()> {
    if !regular {
        return Err("generated output must be a regular file".into());
    }
    if links != 1 {
        return Err("generated output must have exactly one hard link".into());
    }
    if inputs.contains(&identity) {
        return Err("generated output identity collides with an input".into());
    }
    Ok(())
}

fn write_generated_output(
    path: &Path,
    value: &[u8],
    inputs: &HashSet<FileIdentity>,
) -> AuditResult<()> {
    validate_output_target(path, inputs)?;
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut stream = options
        .open(path)
        .map_err(|error| format!("cannot open generated output {}: {error}", path.display()))?;
    validate_output_details(
        &stream.metadata().map_err(|error| {
            format!(
                "cannot inspect generated output {}: {error}",
                path.display()
            )
        })?,
        inputs,
    )?;
    stream
        .seek(std::io::SeekFrom::Start(0))
        .and_then(|_| stream.set_len(0))
        .and_then(|_| stream.write_all(value))
        .and_then(|_| stream.flush())
        .map_err(|error| format!("cannot write generated output {}: {error}", path.display()))
}

fn decode_utf8_exact(value: &[u8], label: &str) -> AuditResult<String> {
    String::from_utf8(value.to_vec()).map_err(|error| format!("{label}: invalid UTF-8: {error}"))
}

fn decode_constitution_bytes(value: &[u8], label: &str) -> AuditResult<String> {
    if value.contains(&b'\r') {
        return Err(format!(
            "{label}: carriage-return bytes are forbidden; exact source requires LF"
        ));
    }
    decode_utf8_exact(value, label)
}

fn amendment_error(message: impl Into<String>) -> Error {
    Error::new(format!("10-amendment-semantics: {}", message.into()))
}

fn reviewed_text<'a>(value: &'a str, path: &str, allow_none_word: bool) -> AuditResult<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() || (!allow_none_word && PLACEHOLDER.is_match(trimmed)) {
        return Err(format!("{path}: requires reviewed, non-placeholder text"));
    }
    Ok(value)
}

fn reviewed_text_list(values: &[String], path: &str, nonempty: bool) -> AuditResult<Vec<String>> {
    if nonempty && values.is_empty() {
        return Err(format!("{path}: must not be empty"));
    }
    let mut result = Vec::with_capacity(values.len());
    let mut seen = HashSet::new();
    for (index, value) in values.iter().enumerate() {
        let item = reviewed_text(value, &format!("{path}[{index}]"), false)?.to_owned();
        if !seen.insert(item.clone()) {
            return Err(format!("{path}: duplicate values are not allowed"));
        }
        result.push(item);
    }
    Ok(result)
}

fn validate_sha<'a>(digest: &'a str, path: &str, expected: Option<&str>) -> AuditResult<&'a str> {
    reviewed_text(digest, path, false)?;
    if !SHA256.is_match(digest) {
        return Err(format!("{path}: expected lowercase SHA-256"));
    }
    if let Some(actual) = expected
        && digest != actual
    {
        return Err(format!("{path}: stale; declared {digest}, actual {actual}"));
    }
    Ok(digest)
}

fn sha256_json(value: &impl Serialize) -> AuditResult<String> {
    let mut canonical = serde_json::to_value(value).map_err(|error| error.to_string())?;
    sort_json(&mut canonical);
    serde_json::to_vec(&canonical)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| error.to_string())
}

fn sort_json(value: &mut Value) {
    match value {
        Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            for (_, child) in &mut entries {
                sort_json(child);
            }
            object.extend(entries);
        }
        Value::Array(array) => {
            for child in array {
                sort_json(child);
            }
        }
        _ => {}
    }
}

fn balanced_single_atom(atom: &str, path: &str) -> AuditResult<String> {
    reviewed_text(atom, path, false)?;
    if atom.contains(['\n', '\r'])
        || !atom
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "_$(),{} \t".contains(ch))
    {
        return Err(format!("{path}: expected one injection-free query atom"));
    }
    let without_variables = VARIABLE.replace_all(atom, "");
    if without_variables.contains('$') {
        return Err(format!("{path}: invalid query variable"));
    }
    let relation_end = atom.find('(').unwrap_or(0);
    if relation_end == 0 || !RELATION.is_match(&atom[..relation_end]) {
        return Err(format!("{path}: query must begin with a relation atom"));
    }
    let mut stack = Vec::new();
    for (index, character) in atom.char_indices() {
        match character {
            '(' | '{' => stack.push(character),
            ')' | '}' => {
                let expected = if character == ')' { '(' } else { '{' };
                if stack.pop() != Some(expected) {
                    return Err(format!("{path}: unbalanced query delimiters"));
                }
                if stack.is_empty() && index + character.len_utf8() != atom.len() {
                    return Err(format!("{path}: trailing or multiple query atoms"));
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() || !atom.ends_with(')') {
        return Err(format!("{path}: incomplete query atom"));
    }
    Ok(atom.to_owned())
}

fn single_statement(statement: &str, path: &str) -> AuditResult<String> {
    reviewed_text(statement, path, false)?;
    if statement.contains(['\n', '\r', '#', '?', ';']) {
        return Err(format!("{path}: statement injection rejected"));
    }
    if !statement.ends_with('.') || statement.matches('.').count() != 1 {
        return Err(format!(
            "{path}: expected exactly one dot-terminated statement"
        ));
    }
    let body = statement[..statement.len() - 1].trim();
    let relation_start = body
        .find('(')
        .map(|index| RELATION.is_match(body[..index].trim()))
        .unwrap_or(false);
    let universal_start = Regex::new(r"^all\s+\$[a-z][a-z0-9_]*\s*:")
        .expect("valid regex")
        .is_match(body);
    if body.starts_with(':') || (!relation_start && !universal_start) {
        return Err(format!(
            "{path}: statement must start with one relation atom or universal rule"
        ));
    }
    if !statement
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "_$(),{}:&~> .-".contains(ch))
    {
        return Err(format!("{path}: unsupported statement character"));
    }
    let mut stack = Vec::new();
    for character in body.chars() {
        match character {
            '(' | '{' => stack.push(character),
            ')' | '}' => {
                let expected = if character == ')' { '(' } else { '{' };
                if stack.pop() != Some(expected) {
                    return Err(format!("{path}: unbalanced statement delimiters"));
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err(format!("{path}: unbalanced statement delimiters"));
    }
    Ok(statement.to_owned())
}

fn apply_mutations(base: &str, mutations: &[Mutation], path: &str) -> AuditResult<String> {
    let mut current = base.to_owned();
    for (index, mutation) in mutations.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let operation = reviewed_text(&mutation.op, &format!("{item_path}.op"), false)?;
        if !MUTATION_OPS.contains(&operation) {
            return Err(format!(
                "{item_path}.op: unknown exact mutation {operation:?}"
            ));
        }
        let before = mutation.before.as_str();
        let after = mutation.after.as_str();
        validate_sha(
            &mutation.before_sha256,
            &format!("{item_path}.before_sha256"),
            Some(&sha256(before.as_bytes())),
        )?;
        validate_sha(
            &mutation.after_sha256,
            &format!("{item_path}.after_sha256"),
            Some(&sha256(after.as_bytes())),
        )?;
        if operation == "append_exact" {
            if !before.is_empty()
                || after.is_empty()
                || !after.starts_with('\n')
                || !after.ends_with('\n')
            {
                return Err(format!(
                    "{item_path}: append_exact needs empty before and newline-bounded after"
                ));
            }
            if current.contains(after) {
                return Err(format!("{item_path}: appended fragment already exists"));
            }
            current.push_str(after);
            continue;
        }
        if before.is_empty() || !before.ends_with('\n') {
            return Err(format!(
                "{item_path}: exact source fragment must end in newline"
            ));
        }
        let count = current.matches(before).count();
        if count != 1 {
            return Err(format!(
                "{item_path}: before fragment must match exactly once; found {count}"
            ));
        }
        if operation == "delete_exact" {
            if !after.is_empty() {
                return Err(format!("{item_path}: delete_exact requires empty after"));
            }
        } else if after.is_empty() || !after.ends_with('\n') || after == before {
            return Err(format!(
                "{item_path}: replace_exact needs a distinct newline-terminated after"
            ));
        }
        current = current.replacen(before, after, 1);
    }
    Ok(current)
}

fn mutations_from_value(value: &Value, path: &str) -> AuditResult<Vec<Mutation>> {
    serde_json::from_value(value.clone()).map_err(|error| format!("{path}: {error}"))
}

fn apply_mutations_value(base: &str, mutations: &Value, path: &str) -> AuditResult<String> {
    let mutations = mutations_from_value(mutations, path)?;
    apply_mutations(base, &mutations, path)
}

fn active_statements(source: &str) -> AuditResult<Vec<String>> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;
    for character in source.chars() {
        if in_comment {
            if character == '\n' {
                in_comment = false;
                if !buffer.is_empty() {
                    buffer.push(' ');
                }
            }
            continue;
        }
        if !in_string && character == '#' {
            in_comment = true;
            continue;
        }
        if character == '\n' {
            if !buffer.is_empty() {
                buffer.push(' ');
            }
            escaped = false;
            continue;
        }
        if character == '"' && !escaped {
            in_string = !in_string;
        }
        if character == '.' && !in_string {
            let normalized = buffer.split_whitespace().collect::<Vec<_>>().join(" ");
            if !normalized.is_empty() {
                result.push(normalized);
            }
            buffer.clear();
            escaped = false;
            continue;
        }
        buffer.push(character);
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }
    if in_string || !buffer.trim().is_empty() {
        return Err("candidate source contains an unterminated statement".into());
    }
    Ok(result)
}

fn validate_source_assertions(
    assertions: &[SourceAssertion],
    candidate: &str,
    path: &str,
) -> AuditResult<()> {
    let statements = active_statements(candidate)?;
    for (index, assertion) in assertions.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        if assertion.kind != "no_relation_first_argument_atom" {
            return Err(format!("{item_path}.kind: unsupported source assertion"));
        }
        let relation = reviewed_text(&assertion.relation, &format!("{item_path}.relation"), false)?;
        let subject = reviewed_text(&assertion.subject, &format!("{item_path}.subject"), false)?;
        if !RELATION.is_match(relation) || !CONSTANT.is_match(subject) {
            return Err(format!("{item_path}: invalid relation or subject"));
        }
        if assertion.expected != "absent" {
            return Err(format!("{item_path}.expected: must remain absent"));
        }
        let pattern = Regex::new(&format!(
            r"(?:^|[^A-Za-z0-9_]){}\s*\(\s*{}\s*,",
            regex::escape(relation),
            regex::escape(subject)
        ))
        .map_err(|error| error.to_string())?;
        let matches = statements
            .iter()
            .filter(|statement| pattern.is_match(statement))
            .collect::<Vec<_>>();
        if !matches.is_empty() {
            return Err(format!(
                "{item_path}: targetless subject has target atom(s): {matches:?}"
            ));
        }
    }
    Ok(())
}

fn assertions_from_value(value: &Value, path: &str) -> AuditResult<Vec<SourceAssertion>> {
    serde_json::from_value(value.clone()).map_err(|error| format!("{path}: {error}"))
}

fn validate_source_assertions_value(
    assertions: &Value,
    candidate: &str,
    path: &str,
) -> AuditResult<()> {
    let assertions = assertions_from_value(assertions, path)?;
    validate_source_assertions(&assertions, candidate, path)
}

fn validate_steps(steps: &[Step], path: &str) -> AuditResult<BTreeMap<String, Vec<String>>> {
    if steps.is_empty() {
        return Err(format!("{path}: every case needs an executable step"));
    }
    let mut queries = BTreeMap::<String, Vec<String>>::new();
    for (index, step) in steps.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        if let Step::Query {
            expression,
            expected,
            purpose,
        } = step
        {
            let expression = balanced_single_atom(expression, &format!("{item_path}.expression"))?;
            let expected = reviewed_text(expected, &format!("{item_path}.expected"), false)?;
            if !["TRUE", "FALSE"].contains(&expected) {
                return Err(format!("{item_path}.expected: expected TRUE or FALSE"));
            }
            let purpose = reviewed_text(purpose, &format!("{item_path}.purpose"), false)?;
            if purpose.contains(['\n', '\r']) {
                return Err(format!(
                    "{item_path}.purpose: pin comment must stay on one line"
                ));
            }
            queries
                .entry(expression)
                .or_default()
                .push(expected.to_owned());
            continue;
        }
        let (step_type, directive, statement, pattern, purpose) = match step {
            Step::Accept {
                directive,
                statement,
                error_pattern,
                purpose,
            } => ("accept", directive, statement, error_pattern, purpose),
            Step::Refuse {
                directive,
                statement,
                error_pattern,
                purpose,
            } => ("refuse", directive, statement, error_pattern, purpose),
            Step::Query { .. } => unreachable!("query handled above"),
        };
        reviewed_text(directive, &format!("{item_path}.directive"), false)?;
        if directive != step_type {
            return Err(format!("{item_path}.directive: must equal step type"));
        }
        single_statement(statement, &format!("{item_path}.statement"))?;
        let pattern = reviewed_text(pattern, &format!("{item_path}.error_pattern"), true)?;
        if step_type == "accept" && pattern != "none" {
            return Err(format!("{item_path}.error_pattern: accept must use none"));
        }
        if step_type == "refuse"
            && (pattern == "none"
                || pattern.contains(['/', '\n', '\r'])
                || !pattern
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "_' `>-".contains(ch)))
        {
            return Err(format!(
                "{item_path}.error_pattern: refuse needs a slash-free error fragment"
            ));
        }
        let purpose = reviewed_text(purpose, &format!("{item_path}.purpose"), false)?;
        if purpose.contains(['\n', '\r']) {
            return Err(format!(
                "{item_path}.purpose: pin comment must stay on one line"
            ));
        }
    }
    Ok(queries)
}

fn validate_source(snapshot: &Snapshot) -> AuditResult<Validated> {
    validate_source_typed(&snapshot.reviewed, &snapshot.ledger, snapshot)
}

fn validate_source_value(
    reviewed: &Value,
    ledger: &Value,
    snapshot: &Snapshot,
) -> AuditResult<Validated> {
    let reviewed = serde_json::from_value::<AmendmentSource>(reviewed.clone())
        .map_err(|error| format!("root: {error}"))?;
    let ledger = serde_json::from_value::<AssertionLedger>(ledger.clone())
        .map_err(|error| format!("assertion ledger: {error}"))?;
    validate_source_typed(&reviewed, &ledger, snapshot)
}

fn validate_source_typed(
    source: &AmendmentSource,
    ledger: &AssertionLedger,
    snapshot: &Snapshot,
) -> AuditResult<Validated> {
    if source.spdx != "CC-BY-4.0" {
        return Err("spdx: reviewed source must be CC-BY-4.0".into());
    }
    if source.schema_version != 1 {
        return Err("schema_version: only integer version 1 is supported".into());
    }
    reviewed_text(&source.title, "title", false)?;
    if source.status != "bounded_source_mutation_audit_not_amendment_assurance" {
        return Err("status: bounded non-assurance posture must remain explicit".into());
    }
    if source.evidence_role != "exposes_semantic_gap" {
        return Err("evidence_role: gap evidence may not become assurance".into());
    }
    if source.subprocess_timeout_seconds != REVIEWED_TIMEOUT_SECONDS {
        return Err("subprocess_timeout_seconds: must equal the reviewed 60-second bound".into());
    }
    validate_sha(
        &source.constitution_sha256,
        "constitution_sha256",
        Some(&snapshot.kb_digest),
    )?;
    validate_sha(
        &source.assertion_surface_contracts_sha256,
        "assertion_surface_contracts_sha256",
        Some(&snapshot.ledger_digest),
    )?;
    validate_sha(
        &source.record_integrity_assurance_case_sha256,
        "record_integrity_assurance_case_sha256",
        Some(&snapshot.assurance_digest),
    )?;

    for key in LABEL_VERDICTS {
        reviewed_text(
            source
                .label_verdict_meanings
                .get(key)
                .expect("every fixed verdict has one typed meaning"),
            &format!("label_verdict_meanings.{key}"),
            false,
        )?;
    }
    for (key, value) in source.limits.in_source_order() {
        reviewed_text(value, &format!("limits.{key}"), false)?;
    }

    let required = reviewed_text_list(&source.required_cases, "required_cases", true)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let expected_required = REQUIRED_CASE_IDS
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    if required != expected_required {
        return Err("required_cases: exact AS-01 through AS-09 set required".into());
    }

    let mut cases = BTreeMap::<String, &AmendmentCase>::new();
    let mut candidates = BTreeMap::<String, Arc<str>>::new();
    let mut query_vectors = BTreeMap::<String, BTreeMap<String, Vec<String>>>::new();
    for (index, case) in source.cases.iter().enumerate() {
        let path = format!("cases[{index}]");
        let case_id = reviewed_text(&case.id, &format!("{path}.id"), false)?;
        if !CASE_ID.is_match(case_id) || !REQUIRED_CASE_IDS.contains(&case_id) {
            return Err(format!("{path}.id: unexpected stable case ID"));
        }
        if cases.contains_key(case_id) {
            return Err(format!("{path}.id: duplicate {case_id}"));
        }
        reviewed_text(&case.title, &format!("{path}.title"), false)?;

        let label = &case.declared_label;
        reviewed_text(
            &label.amendment,
            &format!("{path}.declared_label.amendment"),
            true,
        )?;
        reviewed_text(
            &label.declared_target,
            &format!("{path}.declared_label.declared_target"),
            true,
        )?;
        reviewed_text(
            &label.summary,
            &format!("{path}.declared_label.summary"),
            false,
        )?;
        let verdict = reviewed_text(
            &label.verdict,
            &format!("{path}.declared_label.verdict"),
            false,
        )?;
        if !LABEL_VERDICTS.contains(&verdict) {
            return Err(format!("{path}.declared_label.verdict: unknown verdict"));
        }
        if verdict == "no_amendment_label" && label.amendment != "none" {
            return Err(format!(
                "{path}.declared_label: no-label case must name none"
            ));
        }
        if verdict != "no_amendment_label" && label.amendment == "none" {
            return Err(format!("{path}.declared_label: amendment name required"));
        }

        let effect = &case.source_effect;
        let effect_kind =
            reviewed_text(&effect.kind, &format!("{path}.source_effect.kind"), false)?;
        if !EFFECT_KINDS.contains(&effect_kind) {
            return Err(format!("{path}.source_effect.kind: unknown effect kind"));
        }
        let effect_match = reviewed_text(
            &effect.label_matches_effect,
            &format!("{path}.source_effect.label_matches_effect"),
            false,
        )?;
        if !MATCH_POSTURES.contains(&effect_match) {
            return Err(format!(
                "{path}.source_effect.label_matches_effect: invalid"
            ));
        }
        reviewed_text(
            &effect.summary,
            &format!("{path}.source_effect.summary"),
            false,
        )?;
        reviewed_text(
            &effect.protected_consequence,
            &format!("{path}.source_effect.protected_consequence"),
            false,
        )?;
        let manifest = required_label_manifest(case_id);
        let actual_manifest = (
            label.amendment.as_str(),
            label.declared_target.as_str(),
            verdict,
            effect_match,
        );
        if actual_manifest != manifest {
            return Err(format!(
                "{path}: reviewed amendment/target/verdict/effect-match manifest drifted"
            ));
        }

        let required_shape = required_mutation_shape(case_id);
        if case.mutations.len() != required_shape.len()
            || case
                .mutations
                .iter()
                .zip(required_shape)
                .any(|(mutation, required)| {
                    mutation.op != required.op
                        || mutation.before != required.before
                        || mutation.after != required.after
                })
        {
            return Err(format!(
                "{path}.mutations: reviewed exact operation shape drifted"
            ));
        }
        if effect_kind != required_effect_kind(case_id) {
            return Err(format!(
                "{path}.source_effect.kind: does not match the reviewed exact mutation shape"
            ));
        }
        let candidate = apply_mutations(
            &snapshot.kb_text,
            &case.mutations,
            &format!("{path}.mutations"),
        )?;
        let declared_mutation_digest = validate_sha(
            &case.mutation_sha256,
            &format!("{path}.mutation_sha256"),
            None,
        )?;
        let actual_mutation_digest = sha256_json(&case.mutations)?;
        if declared_mutation_digest != actual_mutation_digest {
            return Err(format!(
                "{path}.mutation_sha256: stale; declared {declared_mutation_digest}, actual {actual_mutation_digest}"
            ));
        }
        let expected_source = validate_sha(
            &case.expected_source_sha256,
            &format!("{path}.expected_source_sha256"),
            None,
        )?;
        let actual_source = sha256(candidate.as_bytes());
        if expected_source != actual_source {
            return Err(format!(
                "{path}.expected_source_sha256: stale; declared {expected_source}, actual {actual_source}"
            ));
        }
        if effect_kind == "unchanged_control" {
            if !case.mutations.is_empty() || candidate != snapshot.kb_text.as_ref() {
                return Err(format!("{path}: unchanged control must be byte-identical"));
            }
        } else if case.mutations.is_empty() || candidate == snapshot.kb_text.as_ref() {
            return Err(format!(
                "{path}: source-effect case requires a real exact delta"
            ));
        }
        validate_source_assertions(
            &case.source_assertions,
            &candidate,
            &format!("{path}.source_assertions"),
        )?;
        let case_queries = validate_steps(&case.steps, &format!("{path}.steps"))?;
        if verdict != "no_amendment_label" {
            let amendment = label.amendment.as_str();
            let expected_pair = if verdict == "blocked_label" {
                ("TRUE", "FALSE")
            } else {
                ("FALSE", "TRUE")
            };
            let actual_false = case_queries
                .get(&format!("false({amendment})"))
                .map(Vec::as_slice)
                .unwrap_or_default();
            let actual_become = case_queries
                .get(&format!("become({amendment}, Law)"))
                .map(Vec::as_slice)
                .unwrap_or_default();
            if !actual_false.iter().any(|item| item == expected_pair.0)
                || !actual_become.iter().any(|item| item == expected_pair.1)
            {
                return Err(format!(
                    "{path}: declared label verdict is not reconciled to false/become queries"
                ));
            }
        }
        let surface = reviewed_text(
            &case.assertion_surface_expectation,
            &format!("{path}.assertion_surface_expectation"),
            false,
        )?;
        if !SURFACE_EXPECTATIONS.contains(&surface) {
            return Err(format!("{path}.assertion_surface_expectation: invalid"));
        }
        let expected_surface = if case_id == "AS-08" {
            "reject_unreconciled_inventory_relation"
        } else {
            "not_run"
        };
        if surface != expected_surface {
            return Err(format!(
                "{path}.assertion_surface_expectation: only AS-08 may run the seam"
            ));
        }
        cases.insert(case_id.to_owned(), case);
        candidates.insert(case_id.to_owned(), Arc::from(candidate));
        query_vectors.insert(case_id.to_owned(), case_queries);
    }
    if cases.keys().cloned().collect::<BTreeSet<_>>() != expected_required {
        return Err("cases: exact AS-01 through AS-09 cases required".into());
    }
    for (case_id, expression, expected) in semantic_sentinels() {
        let actual = query_vectors[case_id]
            .get(expression)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if !actual.iter().any(|item| item == expected) {
            return Err(format!(
                "{case_id}: semantic sentinel {expression} must include {expected}"
            ));
        }
    }

    let vocabulary_cases = cases
        .iter()
        .filter(|(_, case)| case.source_effect.kind == "vocabulary_widening")
        .map(|(case_id, _)| case_id.as_str())
        .collect::<BTreeSet<_>>();
    if vocabulary_cases != BTreeSet::from(["AS-08"]) {
        return Err("source_effect.kind: AS-08 alone must be vocabulary_widening".into());
    }
    let targetless_assertions = &cases["AS-03"].source_assertions;
    if targetless_assertions.len() != 1
        || targetless_assertions[0].kind != "no_relation_first_argument_atom"
    {
        return Err("AS-03: structural targetlessness assertion is mandatory".into());
    }
    for (case_id, expected_match) in [("AS-03", "false"), ("AS-04", "false"), ("AS-09", "true")] {
        let case = cases[case_id];
        if case.source_effect.kind != "direct_deletion"
            || case.source_effect.label_matches_effect != expected_match
        {
            return Err(format!(
                "{case_id}: reviewed direct-deletion match posture must remain explicit"
            ));
        }
        if !case.steps.iter().any(|step| {
            matches!(
                step,
                Step::Accept { statement, .. }
                    if statement == "all $x: person($x) & ~eats($x) -> prisoner($x)."
            )
        }) {
            return Err(format!(
                "{case_id}: executable adverse-rule acceptance is mandatory"
            ));
        }
        let prisoner = query_vectors[case_id]
            .get("prisoner(Cira)")
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>();
        if !prisoner.contains("TRUE") || !prisoner.contains("FALSE") {
            return Err(format!(
                "{case_id}: adverse rule needs discriminating pre/post prisoner queries"
            ));
        }
        if !query_vectors[case_id]
            .get("entitled(Adam, event { eats() })")
            .is_some_and(|values| values.iter().any(|value| value == "FALSE"))
        {
            return Err(format!(
                "{case_id}: deleted floor needs an executable entitlement consequence"
            ));
        }
    }
    if cases["AS-05"].source_effect.kind != "direct_replacement" {
        return Err("AS-05: concealed exact replacement is mandatory".into());
    }
    if !cases["AS-05"].steps.iter().any(|step| {
        matches!(
            step,
            Step::Refuse { error_pattern, .. }
                if error_pattern.contains("'prisoner' -> 'eats'")
        )
    }) {
        return Err("AS-05: structural-wall refusal control is mandatory".into());
    }
    if !cases["AS-07"].steps.iter().any(|step| {
        matches!(
            step,
            Step::Refuse {
                statement,
                error_pattern,
                ..
            } if statement == "rich(Adam)." && error_pattern == "not admitted vocabulary"
        )
    }) {
        return Err("AS-07: exact closed-vocabulary refusal is mandatory".into());
    }
    if ledger.premises.contains_key("rich") {
        return Err(
            "AS-08: live ledger already contracts rich; reviewed seam no longer applies".into(),
        );
    }
    let as08_statements = active_statements(&candidates["AS-08"])?;
    if !as08_statements
        .iter()
        .any(|statement| statement == "admits(\"rich\")")
    {
        return Err("AS-08: exact candidate must directly admit rich".into());
    }
    if !as08_statements
        .iter()
        .any(|statement| statement == "permanent(Art_Evidence)")
    {
        return Err("AS-08: exact candidate must directly register Art_Evidence".into());
    }

    let as09_sequence = cases["AS-09"]
        .steps
        .iter()
        .map(|step| match step {
            Step::Query {
                expression,
                expected,
                ..
            } => ("query", expression.as_str(), expected.as_str()),
            Step::Accept {
                directive,
                statement,
                ..
            } => ("accept", statement.as_str(), directive.as_str()),
            Step::Refuse {
                directive,
                statement,
                ..
            } => ("refuse", statement.as_str(), directive.as_str()),
        })
        .collect::<Vec<_>>();
    if as09_sequence
        != vec![
            ("query", "false(Amend_Floor)", "TRUE"),
            ("query", "become(Amend_Floor, Law)", "FALSE"),
            ("query", "entitled(Adam, event { eats() })", "FALSE"),
            ("query", "prisoner(Cira)", "FALSE"),
            (
                "accept",
                "all $x: person($x) & ~eats($x) -> prisoner($x).",
                "accept",
            ),
            ("query", "prisoner(Cira)", "TRUE"),
        ]
    {
        return Err(
            "AS-09: blocked-label, deleted-floor, and pre/rule/post harm sequence drifted".into(),
        );
    }

    let mut seen_narrowness = BTreeSet::new();
    for (index, entry) in source.narrowness_impacts.iter().enumerate() {
        let path = format!("narrowness_impacts[{index}]");
        let reference = validate_reference(
            &entry.artifact_ref,
            &format!("{path}.artifact_ref"),
            &snapshot.references,
        )?;
        if !seen_narrowness.insert(reference.to_owned()) {
            return Err(format!("{path}.artifact_ref: duplicate reference"));
        }
        let classification = reviewed_text(
            &entry.classification,
            &format!("{path}.classification"),
            false,
        )?;
        if !NARROWNESS_CLASSIFICATIONS.contains(&classification) {
            return Err(format!("{path}.classification: unknown classification"));
        }
        reviewed_text(
            &entry.current_claim,
            &format!("{path}.current_claim"),
            false,
        )?;
        reviewed_text(&entry.reason, &format!("{path}.reason"), false)?;
        reviewed_text(
            &entry.future_trigger,
            &format!("{path}.future_trigger"),
            false,
        )?;
    }
    let required_references = REQUIRED_NARROWNESS_REFERENCES
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    if seen_narrowness != required_references {
        let missing = required_references
            .difference(&seen_narrowness)
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = seen_narrowness
            .difference(&required_references)
            .cloned()
            .collect::<Vec<_>>();
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!(
                "required standing claim omitted: {}",
                missing.join(", ")
            ));
        }
        if !unexpected.is_empty() {
            details.push(format!(
                "unreviewed standing claim added: {}",
                unexpected.join(", ")
            ));
        }
        return Err(format!("narrowness_impacts: {}", details.join("; ")));
    }

    let acceptance = &source.acceptance_result;
    if acceptance.result != "semantic_gap_reproduced" {
        return Err("acceptance_result.result: may not claim assurance".into());
    }
    reviewed_text(&acceptance.claim, "acceptance_result.claim", false)?;
    let residuals = reviewed_text_list(
        &acceptance.does_not_establish,
        "acceptance_result.does_not_establish",
        true,
    )?;
    let residual_text = residuals.join(" ").to_lowercase();
    for term in [
        "become",
        "semantic completeness",
        "source author",
        "withholding gate",
    ] {
        if !residual_text.contains(term) {
            return Err(format!(
                "acceptance_result.does_not_establish: missing {term:?} boundary"
            ));
        }
    }
    reviewed_text(
        &acceptance.remaining_boundary,
        "acceptance_result.remaining_boundary",
        false,
    )?;
    Ok(Validated { candidates })
}

fn validate_reference<'a>(
    reference: &'a str,
    path: &str,
    references: &BTreeMap<String, Arc<str>>,
) -> AuditResult<&'a str> {
    reviewed_text(reference, path, false)?;
    if reference.matches("::").count() != 1 {
        return Err(format!(
            "{path}: reference must use repo-local path::unique literal needle"
        ));
    }
    let (raw_file, needle) = reference
        .split_once("::")
        .ok_or_else(|| format!("{path}: invalid path or empty reference needle"))?;
    let candidate = Path::new(raw_file);
    if raw_file.is_empty()
        || needle.is_empty()
        || raw_file.contains('\\')
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(format!("{path}: invalid path or empty reference needle"));
    }
    let body = references
        .get(raw_file)
        .ok_or_else(|| format!("{path}: referenced file does not exist: {raw_file}"))?;
    let count = body.matches(needle).count();
    if count != 1 {
        return Err(format!(
            "{path}: needle must occur exactly once in {raw_file}; found {count}"
        ));
    }
    Ok(reference)
}

fn markdown(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn code(value: impl fmt::Display) -> String {
    let value = value.to_string();
    let fence = if value.contains('`') { "``" } else { "`" };
    format!("{fence}{value}{fence}")
}

fn title_key(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), characters.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render(snapshot: &Snapshot) -> AuditResult<String> {
    let source = &snapshot.reviewed;
    let acceptance = &source.acceptance_result;
    let mut lines = vec![
        format!("<!-- SPDX-License-Identifier: {} -->", source.spdx),
        "<!-- Generated by the native rights-verify amendment-semantics refresh; do not edit. -->"
            .into(),
        String::new(),
        format!("# {}", source.title),
        String::new(),
        "## Verdict and scope".into(),
        String::new(),
        "**SEMANTIC GAP REPRODUCED — bounded source-mutation evidence, not amendment assurance.**"
            .into(),
        String::new(),
        markdown(&acceptance.claim),
        String::new(),
        "Each candidate diff is authored directly by this audit. A TRUE".into(),
        "`become(_, Law)` result is a label verdict only: the current constitution".into(),
        "has no reader that applies source text. A `false(_)` result likewise does".into(),
        "not prove that an independently supplied source transition was prevented.".into(),
        String::new(),
        "## Label-verdict meanings".into(),
        String::new(),
        "| label verdict | exact meaning |".into(),
        "| --- | --- |".into(),
    ];
    for verdict in LABEL_VERDICTS {
        lines.push(format!(
            "| {} | {} |",
            code(verdict),
            markdown(
                source
                    .label_verdict_meanings
                    .get(verdict)
                    .expect("fixed typed label meaning")
            )
        ));
    }
    lines.extend([
        String::new(),
        "## Label verdicts and source effects".into(),
        String::new(),
        "These columns are intentionally separate. The first is derived from".into(),
        "Article 9's self-declared labels; the second is the exact candidate text".into(),
        "the harness independently constructs.".into(),
        "Declared-target match is the reviewed test author's classification; Article 9".into(),
        "does not derive it, and the harness is not a semantic oracle. It says only".into(),
        "whether the declared target accurately describes the independently applied".into(),
        "effect—not that the label verdict controlled a source transition.".into(),
        String::new(),
        "| case | declared label and verdict | exact source effect | declared target matches effect |".into(),
        "| --- | --- | --- | --- |".into(),
    ]);
    for case in &source.cases {
        let label = &case.declared_label;
        let effect = &case.source_effect;
        let label_text = format!(
            "{} → {}; {}",
            label.amendment, label.declared_target, label.verdict
        );
        lines.push(format!(
            "| {} {} | {} | {}: {} | {} |",
            code(&case.id),
            markdown(&case.title),
            markdown(&label_text),
            code(&effect.kind),
            markdown(&effect.summary),
            code(&effect.label_matches_effect),
        ));
    }

    lines.extend([String::new(), "## Limits".into(), String::new()]);
    for (key, value) in source.limits.in_source_order() {
        lines.push(format!("- **{}:** {}", title_key(key), markdown(value)));
    }
    lines.extend([
        String::new(),
        "## Exact mutation manifest".into(),
        String::new(),
        "Every fragment digest is checked before application; deletions and".into(),
        "replacements must match exactly once, and each final candidate digest".into(),
        "is reviewed in the JSON source.".into(),
        String::new(),
        "| case | operation | before SHA-256 | after SHA-256 | candidate SHA-256 |".into(),
        "| --- | --- | --- | --- | --- |".into(),
    ]);
    for case in &source.cases {
        if case.mutations.is_empty() {
            lines.push(format!(
                "| {} | byte-identical control | — | — | {} |",
                code(&case.id),
                code(&case.expected_source_sha256),
            ));
            continue;
        }
        for (index, mutation) in case.mutations.iter().enumerate() {
            let candidate = if index == 0 {
                case.expected_source_sha256.as_str()
            } else {
                "↳"
            };
            lines.push(format!(
                "| {} | {} | {} | {} | {} |",
                code(&case.id),
                code(&mutation.op),
                code(&mutation.before_sha256),
                code(&mutation.after_sha256),
                code(candidate),
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Executable cases".into(),
        String::new(),
        "Every ordinary and opaque food-entitlement verdict runs in one process".into(),
        "against the full exact candidate source. This keeps an amendment's".into(),
        "floor effect coupled to all temporal and non-temporal rules that coexist".into(),
        "with it in the same full-candidate process.".into(),
        String::new(),
    ]);
    for case in &source.cases {
        let label = &case.declared_label;
        let effect = &case.source_effect;
        lines.extend([
            format!("### {} — {}", case.id, case.title),
            String::new(),
            format!(
                "- **Label verdict:** {} — {}",
                code(&label.verdict),
                markdown(&label.summary)
            ),
            format!(
                "- **Source effect:** {} — {}",
                code(&effect.kind),
                markdown(&effect.summary)
            ),
            format!(
                "- **Protected/adverse consequence:** {}",
                markdown(&effect.protected_consequence)
            ),
            format!("- **Mutation contract:** {}", code(&case.mutation_sha256)),
            String::new(),
            "| check | expected | purpose |".into(),
            "| --- | --- | --- |".into(),
        ]);
        for step in &case.steps {
            let (check, expected) = match step {
                Step::Query {
                    expression,
                    expected,
                    ..
                } => (code(expression), format!("**{expected}**")),
                Step::Accept { statement, .. } => {
                    (format!("accept {}", code(statement)), "**ACCEPTED**".into())
                }
                Step::Refuse {
                    statement,
                    error_pattern,
                    ..
                } => (
                    format!("refuse {}", code(statement)),
                    format!("**REFUSED** ({})", code(error_pattern)),
                ),
            };
            lines.push(format!(
                "| {check} | {expected} | {} |",
                markdown(step.purpose())
            ));
        }
        for assertion in &case.source_assertions {
            lines.push(format!(
                "| structural: no {} atom whose first argument is {} | **ABSENT** | Targetlessness is checked across active facts, compounds, and rule statements, not inferred from a finite query list. |",
                code(&assertion.relation),
                code(&assertion.subject),
            ));
        }
        if case.assertion_surface_expectation != "not_run" {
            lines.push(
                "| live assertion-surface pipeline | **REJECTS** `rich` during source/inventory reconciliation | The engine accepts the widening, but `rich` is absent from the audit's engine inventory and alias contract; premise-card validation is not reached. |".into(),
            );
        }
        lines.push(String::new());
    }
    lines.extend([
        "## Narrowness impacts".into(),
        String::new(),
        "The audit changes no live constitutional rule. These reviewed entries".into(),
        "record every standing claim whose scope depends on the label/effect".into(),
        "boundary or on the current source remaining narrow.".into(),
        String::new(),
        "| artifact | current claim | classification | reason | future trigger |".into(),
        "| --- | --- | --- | --- | --- |".into(),
    ]);
    for entry in &source.narrowness_impacts {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code(&entry.artifact_ref),
            markdown(&entry.current_claim),
            code(&entry.classification),
            markdown(&entry.reason),
            markdown(&entry.future_trigger),
        ));
    }
    lines.push(String::new());
    lines.extend([
        "## Acceptance result".into(),
        String::new(),
        "**SEMANTIC GAP REPRODUCED.**".into(),
        String::new(),
        markdown(&acceptance.claim),
        String::new(),
        "This artifact does **not** establish:".into(),
        String::new(),
    ]);
    for value in &acceptance.does_not_establish {
        lines.push(format!("- {}", markdown(value)));
    }
    lines.extend([
        String::new(),
        format!(
            "Remaining boundary: {}",
            markdown(&acceptance.remaining_boundary)
        ),
        String::new(),
        "## Maintenance".into(),
        String::new(),
        format!(
            "- Reviewed source: {}.",
            code(&snapshot.source_relative)
        ),
        format!(
            "- Constitution: {}, SHA-256 {}.",
            code(&snapshot.kb_relative),
            code(&source.constitution_sha256)
        ),
        format!(
            "- Assertion ledger: {}, SHA-256 {}.",
            code(&snapshot.ledger_relative),
            code(&source.assertion_surface_contracts_sha256)
        ),
        format!(
            "- Assurance source: {}, SHA-256 {}.",
            code(&snapshot.assurance_relative),
            code(&source.record_integrity_assurance_case_sha256)
        ),
        format!(
            "- Reviewed subprocess timeout: {} seconds for every isolated case, sabotage, and live seam.",
            code(source.subprocess_timeout_seconds)
        ),
        "- Bound input bytes are read once, decoded strictly, and hashed without newline translation; constitution CR bytes are refused and candidates are written as exact UTF-8 bytes.".into(),
        "- Existing inputs must have distinct device/inode identities; the generated output must be a single-link regular file distinct from every input.".into(),
        "- Expected failures are narrow contracts: exit 1 plus one final Nibli finding summary for sabotage, and exit 1 plus one anchored assertion-generator error for the live seam.".into(),
        "- Timed-out subprocesses run in an isolated process group and the whole process tree is terminated before the harness fails.".into(),
        "- Regenerate: `./verify.sh --refresh amendment-semantics`.".into(),
        "- Fast structural/freshness check: `./verify.sh --quick`.".into(),
        "- Authoritative execution: `./verify.sh`.".into(),
        "- Each candidate and the executable sabotage run in fresh engine processes.".into(),
        String::new(),
    ]);
    Ok(lines.join("\n"))
}

fn pin_text(case: &AmendmentCase, scope: &str) -> AuditResult<String> {
    let mut lines = vec![
        format!(":expect-pins {}", case.steps.len()),
        format!(
            "# Generated {scope} amendment-semantics pins for {}.",
            case.id
        ),
        "# Candidate source deltas are authored by the audit, never enacted by become.".into(),
        String::new(),
    ];
    for step in &case.steps {
        lines.push(format!("# {}", step.purpose()));
        match step {
            Step::Query {
                expression,
                expected,
                ..
            } => lines.extend([
                format!("? {expression}."),
                format!("# => {expected}"),
                String::new(),
            ]),
            Step::Accept { statement, .. } => {
                lines.extend([":accept".into(), statement.clone(), String::new()])
            }
            Step::Refuse {
                statement,
                error_pattern,
                ..
            } => lines.extend([
                format!(":refuse reasoning /{}/", error_pattern),
                statement.clone(),
                String::new(),
            ]),
        }
    }
    Ok(lines.join("\n"))
}

fn parse_pass_count(output: &str, label: &str) -> AuditResult<usize> {
    static CLEAN_FILE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?im)^\s*[^\r\n]+:\s+[0-9]+\s+pins?,\s+0\s+findings?,\s+0\s+harness errors?\s*$",
        )
        .expect("valid regex")
    });
    static FORBIDDEN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?im)(?:FINDING|HARNESS ERROR|NO LONGER REPRODUCE|TRACEBACK|PANIC)")
            .expect("valid regex")
    });
    static PASS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?m)^nibli-pin:\s+PASS\s+[—-]\s+([0-9]+)\s+pins?\s*$").expect("valid regex")
    });
    let marker_surface = CLEAN_FILE.replace_all(output, "");
    if let Some(marker) = FORBIDDEN.find(&marker_surface) {
        return Err(format!(
            "{label}: failure marker appeared despite process success: {}",
            marker.as_str()
        ));
    }
    let matches = PASS.captures_iter(output).collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "{label}: expected exactly one anchored PASS summary; found {}\n{}",
            matches.len(),
            tail_lines(output, 12)
        ));
    }
    matches[0][1]
        .parse::<usize>()
        .map_err(|error| format!("{label}: invalid PASS count: {error}"))
}

fn validate_sabotage_failure(returncode: u8, output: &str) -> AuditResult<()> {
    if returncode != 1 {
        return Err(format!(
            "executable inverted-verdict sabotage exited {returncode}, expected 1"
        ));
    }
    static CLEAN_HARNESS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b0\s+harness errors?\b").expect("valid regex"));
    static FORBIDDEN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(?:HARNESS|NO LONGER|TRACEBACK|PANIC)\b").expect("valid regex")
    });
    let marker_surface = CLEAN_HARNESS.replace_all(output, "");
    if let Some(marker) = FORBIDDEN.find(&marker_surface) {
        return Err(format!(
            "executable inverted-verdict sabotage emitted forbidden marker: {}",
            marker.as_str()
        ));
    }
    let lines = output
        .trim_end_matches(['\r', '\n'])
        .lines()
        .collect::<Vec<_>>();
    let matches = lines
        .iter()
        .filter(|line| **line == SABOTAGE_FINAL_SUMMARY)
        .count();
    if matches != 1 || lines.last().copied() != Some(SABOTAGE_FINAL_SUMMARY) {
        return Err(format!(
            "executable inverted-verdict sabotage requires exactly one final {SABOTAGE_FINAL_SUMMARY}"
        ));
    }
    Ok(())
}

fn validate_surface_seam_failure(
    returncode: u8,
    output: &str,
    expected_fragment: &str,
) -> AuditResult<()> {
    if returncode != 1 {
        return Err(format!(
            "AS-08 assertion-surface seam exited {returncode}, expected 1"
        ));
    }
    static FORBIDDEN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(?:TRACEBACK|PANIC)\b").expect("valid regex"));
    if let Some(marker) = FORBIDDEN.find(output) {
        return Err(format!(
            "AS-08 assertion-surface seam emitted forbidden marker: {}",
            marker.as_str()
        ));
    }
    let pattern = Regex::new(&format!(
        r"^7-assertion-surface:\s+[^\r\n]*{}[^\r\n]*$",
        regex::escape(expected_fragment)
    ))
    .map_err(|error| error.to_string())?;
    let lines = output
        .trim_end_matches(['\r', '\n'])
        .lines()
        .collect::<Vec<_>>();
    let matches = lines.iter().filter(|line| pattern.is_match(line)).count();
    if matches != 1 || lines.last().is_none_or(|line| !pattern.is_match(line)) {
        return Err(
            "AS-08 assertion-surface seam requires exactly one final anchored 7-assertion-surface error containing the reviewed fragment".into(),
        );
    }
    Ok(())
}

#[derive(Clone)]
struct ExecutionGroup {
    digest: String,
    candidate: Arc<str>,
    cases: Vec<(String, String, usize)>,
}

fn executable_lines(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn line_patch(base: &str, candidate: &str) -> (Vec<String>, Vec<String>) {
    let base = executable_lines(base);
    let candidate = executable_lines(candidate);
    let mut candidate_counts = HashMap::<&str, usize>::new();
    for line in &candidate {
        *candidate_counts.entry(line).or_default() += 1;
    }
    let mut deletions = Vec::new();
    for line in &base {
        let count = candidate_counts.entry(line).or_default();
        if *count == 0 {
            deletions.push(line.clone());
        } else {
            *count -= 1;
        }
    }
    let mut base_counts = HashMap::<&str, usize>::new();
    for line in &base {
        *base_counts.entry(line).or_default() += 1;
    }
    let mut additions = Vec::new();
    for line in &candidate {
        let count = base_counts.entry(line).or_default();
        if *count == 0 {
            additions.push(line.clone());
        } else {
            *count -= 1;
        }
    }
    (deletions, additions)
}

fn execute_cases(
    context: &Context,
    snapshot: &Snapshot,
    validated: &Validated,
) -> AuditResult<ExecutionReport> {
    let cases = &snapshot.reviewed.cases;
    let _requested_jobs = match std::env::var("AMENDMENT_AUDIT_JOBS") {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| "AMENDMENT_AUDIT_JOBS must be a positive integer".to_owned())?,
        Err(std::env::VarError::NotPresent) => 4,
        Err(error) => return Err(format!("cannot read AMENDMENT_AUDIT_JOBS: {error}")),
    };
    let mut grouped = BTreeMap::<String, ExecutionGroup>::new();
    let mut total_pins = 0;
    for case in cases {
        let identifier = case.id.as_str();
        let digest = case.expected_source_sha256.as_str();
        let pins = case.steps.len();
        let pins_text = pin_text(case, "full-source")?;
        total_pins += pins;
        grouped
            .entry(digest.to_owned())
            .or_insert_with(|| ExecutionGroup {
                digest: digest.to_owned(),
                candidate: Arc::clone(&validated.candidates[identifier]),
                cases: Vec::new(),
            })
            .cases
            .push((identifier.to_owned(), pins_text, pins));
    }
    // A prepared engine is intentionally !Sync because its engine snapshot has
    // RefCell-backed caches. Reusing one base sequentially avoids seven full
    // constitution parses; measured worker-local bases cost more CPU without a
    // material wall-time win. Every grouped pin file still runs against a fresh
    // clone of its patched candidate.
    let prepared =
        PreparedPinEngine::new(&[LoadedSource::new(&snapshot.kb_relative, &snapshot.kb_text)]);
    let options = PinOptions {
        allow_shell: false,
        working_directory: Some(context.root()),
    };
    let mut pins = 0;
    for group in grouped.into_values() {
        let started = Instant::now();
        let (deletions, additions) = line_patch(&snapshot.kb_text, &group.candidate);
        let deletion_refs = deletions.iter().map(String::as_str).collect::<Vec<_>>();
        let addition_refs = additions.iter().map(String::as_str).collect::<Vec<_>>();
        let pin_files = group
            .cases
            .iter()
            .map(|(identifier, pins, _)| LoadedSource::new(identifier, pins))
            .collect::<Vec<_>>();
        let output =
            prepared.run_patched_files(&deletion_refs, &addition_refs, &pin_files, options);
        if output.exit_code != 0 {
            return Err(format!(
                "candidate group {} nibli-pin exited {}\n{}",
                group.digest,
                output.exit_code,
                tail_lines(&format!("{}{}", output.stdout, output.stderr), 16)
            ));
        }
        if output.files.len() != group.cases.len() {
            return Err(format!(
                "candidate group {} returned {} file results, expected {}",
                group.digest,
                output.files.len(),
                group.cases.len()
            ));
        }
        for ((identifier, _, expected), actual) in group.cases.iter().zip(&output.files) {
            if actual.pins != *expected
                || actual.findings != 0
                || actual.harness != 0
                || actual.resolved != 0
            {
                return Err(format!(
                    "{identifier}: full-source engine ran {} clean pins, expected {expected}",
                    actual.pins
                ));
            }
            pins += actual.pins;
        }
        validate_elapsed(
            &format!("candidate group {}", group.digest),
            started.elapsed(),
            Duration::from_secs(REVIEWED_TIMEOUT_SECONDS),
        )?;
    }
    if pins != total_pins {
        return Err(format!("engine ran {pins} pins, expected {total_pins}"));
    }

    let sabotage_started = Instant::now();
    let sabotage_text = ":expect-pins 1\n# Deliberately inverted: the ordinary label currently passes.\n\n? become(Amend_Mint, Law).\n# => FALSE\n";
    let sabotage_run = prepared.run_files(
        &[LoadedSource::new("inverted.pins.nibli", sabotage_text)],
        options,
    );
    validate_sabotage_failure(
        sabotage_run.exit_code,
        &format!("{}{}", sabotage_run.stdout, sabotage_run.stderr),
    )?;
    validate_elapsed(
        "executable inverted-verdict sabotage",
        sabotage_started.elapsed(),
        Duration::from_secs(REVIEWED_TIMEOUT_SECONDS),
    )?;
    let sabotage = 1;

    let seam_candidate = &validated.candidates["AS-08"];
    let seam_started = Instant::now();
    let (seam_deletions, seam_additions) = line_patch(&snapshot.kb_text, seam_candidate);
    let seam_deletion_refs = seam_deletions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let seam_addition_refs = seam_additions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let strata = prepared.dump_patched_strata(&seam_deletion_refs, &seam_addition_refs);
    if strata.exit_code != 0 {
        return Err(format!(
            "AS-08 assertion-surface strata failed: {}{}",
            strata.stdout, strata.stderr
        ));
    }
    let surface = assertion_surface::check(context, Some(&strata.stdout), Some(seam_candidate));
    let surface_output = match surface {
        Ok(_) => {
            return Err("AS-08 assertion-surface seam exited 0, expected 1".into());
        }
        Err(error) => format!("{error}\n"),
    };
    validate_surface_seam_failure(1, &surface_output, SURFACE_SEAM_FRAGMENT)?;
    validate_elapsed(
        "AS-08 assertion-surface seam",
        seam_started.elapsed(),
        Duration::from_secs(REVIEWED_TIMEOUT_SECONDS),
    )?;
    Ok(ExecutionReport {
        cases: cases.len(),
        pins,
        sabotage_controls: sabotage,
        seam_controls: 1,
    })
}

fn tail_lines(value: &str, count: usize) -> String {
    let lines = value.lines().collect::<Vec<_>>();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

fn expect_failure(
    label: &str,
    result: AuditResult<impl Sized>,
    contains: Option<&str>,
) -> AuditResult<()> {
    match result {
        Err(error) if contains.is_none_or(|needle| error.contains(needle)) => Ok(()),
        Err(error) => Err(format!(
            "negative control failed for the wrong reason: {label}: {error}"
        )),
        Ok(_) => Err(format!("negative control did not fail: {label}")),
    }
}

fn root_mut(value: &mut Value) -> AuditResult<&mut Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| "negative-control root is not an object".into())
}

fn case_mut<'a>(value: &'a mut Value, identifier: &str) -> AuditResult<&'a mut Map<String, Value>> {
    let cases = root_mut(value)?
        .get_mut("cases")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "negative-control cases are not an array".to_owned())?;
    let mut matches = cases
        .iter_mut()
        .filter(|case| case.get("id").and_then(Value::as_str) == Some(identifier));
    let result = matches
        .next()
        .ok_or_else(|| format!("negative-control lookup: expected one {identifier}; found 0"))?;
    if matches.next().is_some() {
        return Err(format!(
            "negative-control lookup: expected one {identifier}; found more than one"
        ));
    }
    result
        .as_object_mut()
        .ok_or_else(|| format!("negative-control case {identifier} is not an object"))
}

fn mutation_mut<'a>(
    case: &'a mut Map<String, Value>,
    operation: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> AuditResult<&'a mut Map<String, Value>> {
    let mutations = case
        .get_mut("mutations")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "negative-control mutations are not an array".to_owned())?;
    let mut matches = mutations.iter_mut().filter(|mutation| {
        mutation.get("op").and_then(Value::as_str) == Some(operation)
            && before
                .is_none_or(|value| mutation.get("before").and_then(Value::as_str) == Some(value))
            && after
                .is_none_or(|value| mutation.get("after").and_then(Value::as_str) == Some(value))
    });
    let result = matches.next().ok_or_else(|| {
        "negative-control lookup: expected one semantic mutation; found 0".to_owned()
    })?;
    if matches.next().is_some() {
        return Err(
            "negative-control lookup: expected one semantic mutation; found more than one".into(),
        );
    }
    result
        .as_object_mut()
        .ok_or_else(|| "negative-control mutation is not an object".into())
}

fn query_mut<'a>(
    case: &'a mut Map<String, Value>,
    expression: &str,
    expected: Option<&str>,
) -> AuditResult<&'a mut Map<String, Value>> {
    let steps = case
        .get_mut("steps")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "negative-control steps are not an array".to_owned())?;
    let mut matches = steps.iter_mut().filter(|step| {
        step.get("type").and_then(Value::as_str) == Some("query")
            && step.get("expression").and_then(Value::as_str) == Some(expression)
            && expected
                .is_none_or(|value| step.get("expected").and_then(Value::as_str) == Some(value))
    });
    let result = matches.next().ok_or_else(|| {
        format!("negative-control lookup: expected one query {expression:?}; found 0")
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "negative-control lookup: expected one query {expression:?}; found more than one"
        ));
    }
    result
        .as_object_mut()
        .ok_or_else(|| "negative-control query is not an object".into())
}

fn control_mut<'a>(
    case: &'a mut Map<String, Value>,
    step_type: &str,
    statement: &str,
) -> AuditResult<&'a mut Map<String, Value>> {
    let steps = case
        .get_mut("steps")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "negative-control steps are not an array".to_owned())?;
    let mut matches = steps.iter_mut().filter(|step| {
        step.get("type").and_then(Value::as_str) == Some(step_type)
            && step.get("statement").and_then(Value::as_str) == Some(statement)
    });
    let result = matches.next().ok_or_else(|| {
        format!("negative-control lookup: expected one {step_type} statement; found 0")
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "negative-control lookup: expected one {step_type} statement; found more than one"
        ));
    }
    result
        .as_object_mut()
        .ok_or_else(|| "negative-control statement is not an object".into())
}

fn impact_mut<'a>(
    value: &'a mut Value,
    artifact_ref: &str,
) -> AuditResult<&'a mut Map<String, Value>> {
    let impacts = root_mut(value)?
        .get_mut("narrowness_impacts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "negative-control narrowness impacts are not an array".to_owned())?;
    let mut matches = impacts
        .iter_mut()
        .filter(|impact| impact.get("artifact_ref").and_then(Value::as_str) == Some(artifact_ref));
    let result = matches.next().ok_or_else(|| {
        format!("negative-control lookup: expected one narrowness impact {artifact_ref:?}; found 0")
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "negative-control lookup: expected one narrowness impact {artifact_ref:?}; found more than one"
        ));
    }
    result
        .as_object_mut()
        .ok_or_else(|| "negative-control narrowness impact is not an object".into())
}

fn validate_elapsed(label: &str, elapsed: Duration, bound: Duration) -> AuditResult<()> {
    if elapsed > bound {
        Err(format!(
            "{label}: in-process execution exceeded reviewed {}-second bound",
            bound.as_secs()
        ))
    } else {
        Ok(())
    }
}

fn negative_controls(snapshot: &Snapshot) -> AuditResult<usize> {
    let mut controls = 0;
    macro_rules! fails {
        ($label:expr, $result:expr) => {{
            expect_failure($label, $result, None)?;
            controls += 1;
        }};
        ($label:expr, $result:expr, $contains:expr) => {{
            expect_failure($label, $result, Some($contains))?;
            controls += 1;
        }};
    }
    let validate =
        |candidate: &Value| validate_source_value(candidate, &snapshot.ledger_raw, snapshot);

    for (key, label) in [
        ("constitution_sha256", "constitution digest drift"),
        (
            "assertion_surface_contracts_sha256",
            "assertion-ledger digest drift",
        ),
        (
            "record_integrity_assurance_case_sha256",
            "assurance-source digest drift",
        ),
    ] {
        let mut changed = snapshot.reviewed_raw.clone();
        root_mut(&mut changed)?.insert(key.into(), Value::String("0".repeat(64)));
        fails!(label, validate(&changed));
    }

    let crlf = snapshot
        .kb_bytes
        .iter()
        .fold(Vec::new(), |mut bytes, byte| {
            if *byte == b'\n' {
                bytes.push(b'\r');
            }
            bytes.push(*byte);
            bytes
        });
    if crlf.as_slice() == snapshot.kb_bytes.as_ref() {
        return Err("CRLF negative control requires at least one LF byte".into());
    }
    fails!(
        "CRLF constitution rejected before newline normalization",
        decode_constitution_bytes(&crlf, "CRLF control"),
        "carriage-return bytes are forbidden"
    );

    let hardlink_parent = snapshot
        .output_path
        .parent()
        .ok_or_else(|| "generated output has no parent".to_owned())?;
    let hardlink_temp = tempfile::Builder::new()
        .prefix(".amendment-hardlink-control-")
        .tempdir_in(hardlink_parent)
        .map_err(|error| format!("cannot create hardlink control: {error}"))?;
    let first = hardlink_temp.path().join("first.json");
    let alias = hardlink_temp.path().join("alias.json");
    std::fs::write(&first, b"{}\n")
        .map_err(|error| format!("cannot write hardlink control: {error}"))?;
    std::fs::hard_link(&first, &alias).map_err(|error| {
        format!("hardlink negative control could not create its alias: {error}")
    })?;
    let (_, first_identity) = read_bound_file(&first, "hardlink control first")?;
    let (_, alias_identity) = read_bound_file(&alias, "hardlink control alias")?;
    fails!(
        "hardlinked input identities collide",
        require_distinct_identities(&[("first", first_identity), ("alias", alias_identity)])
    );
    fails!(
        "hardlinked generated output rejected",
        validate_output_target(&alias, &HashSet::new()),
        "exactly one hard link"
    );
    fails!(
        "generated output identity collides with input",
        validate_output_characteristics(true, 1, first_identity, &HashSet::from([first_identity])),
        "identity collides with an input"
    );

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?.insert("schema_version".into(), Value::Bool(true));
    fails!("boolean schema version", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?
        .get_mut("cases")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "cases missing".to_owned())?
        .retain(|case| case.get("id").and_then(Value::as_str) != Some("AS-01"));
    fails!("required case deleted", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let duplicate = case_mut(&mut changed, "AS-01")?.clone();
    root_mut(&mut changed)?["cases"]
        .as_array_mut()
        .ok_or_else(|| "cases missing".to_owned())?
        .push(Value::Object(duplicate));
    fails!("duplicate case ID", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-01")?.insert("unexpected".into(), Value::String("field".into()));
    fails!("unknown case field", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    query_mut(case_mut(&mut changed, "AS-01")?, "false(Amend_Floor)", None)?
        .insert("expected".into(), Value::String("FALSE".into()));
    fails!("reversed semantic sentinel", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    mutation_mut(
        case_mut(&mut changed, "AS-03")?,
        "delete_exact",
        Some(FLOOR_LINE),
        None,
    )?
    .insert("before_sha256".into(), Value::String("0".repeat(64)));
    fails!("exact fragment digest drift", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-03")?.insert(
        "expected_source_sha256".into(),
        Value::String("0".repeat(64)),
    );
    fails!("candidate source digest drift", validate(&changed));

    let baseline_as03 = case_mut(&mut snapshot.reviewed_raw.clone(), "AS-03")?.clone();
    let mut missing = mutation_mut(
        &mut baseline_as03.clone(),
        "delete_exact",
        Some(FLOOR_LINE),
        None,
    )?
    .clone();
    missing.insert(
        "before".into(),
        Value::String("missing exact line\n".into()),
    );
    missing.insert(
        "before_sha256".into(),
        Value::String(sha256(b"missing exact line\n")),
    );
    fails!(
        "exact deletion with zero matches",
        apply_mutations_value(
            &snapshot.kb_text,
            &Value::Array(vec![Value::Object(missing)]),
            "zero-match control"
        )
    );

    let mut baseline_as03_for_append = baseline_as03.clone();
    let mut unknown = mutation_mut(
        &mut baseline_as03_for_append,
        "append_exact",
        None,
        Some(TARGETLESS_APPEND),
    )?
    .clone();
    unknown.insert("op".into(), Value::String("rewrite_semantically".into()));
    fails!(
        "unknown mutation operation",
        apply_mutations_value(
            &snapshot.kb_text,
            &Value::Array(vec![Value::Object(unknown)]),
            "unknown-op control"
        )
    );

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-03")?.insert("source_assertions".into(), Value::Array(Vec::new()));
    fails!("targetlessness assertion removed", validate(&changed));

    let targetless_assertions = snapshot.reviewed_raw["cases"]
        .as_array()
        .and_then(|cases| cases.iter().find(|case| case["id"] == "AS-03"))
        .map(|case| case["source_assertions"].clone())
        .ok_or_else(|| "AS-03 assertions missing".to_owned())?;
    fails!(
        "targetless atom hidden in a compound fact",
        validate_source_assertions_value(
            &targetless_assertions,
            &format!(
                "{}\nperson(Cira) & adjust (Amend_Targetless, Art_Mint).\n",
                snapshot.kb_text
            ),
            "compound-target control"
        )
    );
    fails!(
        "targetless atom derived in a rule head",
        validate_source_assertions_value(
            &targetless_assertions,
            &format!(
                "{}\nall $x: person($x) -> adjust (Amend_Targetless, Art_Mint).\n",
                snapshot.kb_text
            ),
            "rule-target control"
        )
    );

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-04")?["source_effect"]
        .as_object_mut()
        .ok_or_else(|| "source_effect missing".to_owned())?
        .remove("label_matches_effect");
    fails!("label and source effect collapsed", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let label = case_mut(&mut changed, "AS-09")?["declared_label"]
        .as_object_mut()
        .ok_or_else(|| "declared_label missing".to_owned())?;
    label.insert("amendment".into(), Value::String("none".into()));
    label.insert("declared_target".into(), Value::String("none".into()));
    label.insert("verdict".into(), Value::String("no_amendment_label".into()));
    fails!("AS-09 relabelled as no-amendment", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let label = case_mut(&mut changed, "AS-08")?["declared_label"]
        .as_object_mut()
        .ok_or_else(|| "declared_label missing".to_owned())?;
    label.insert("amendment".into(), Value::String("Amend_Evidence".into()));
    label.insert(
        "declared_target".into(),
        Value::String("Art_Evidence".into()),
    );
    label.insert("verdict".into(), Value::String("blocked_label".into()));
    fails!("AS-08 relabelled as blocked", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-08")?["source_effect"]
        .as_object_mut()
        .ok_or_else(|| "source_effect missing".to_owned())?
        .insert("label_matches_effect".into(), Value::String("true".into()));
    fails!("AS-08 declared-target match promoted", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-05")?["steps"]
        .as_array_mut()
        .ok_or_else(|| "steps missing".to_owned())?
        .retain(|step| {
            !(step["type"] == "refuse" && step["error_pattern"] == "'prisoner' -> 'eats'")
        });
    fails!(
        "concealed structural-wall control removed",
        validate(&changed)
    );

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-08")?.insert(
        "assertion_surface_expectation".into(),
        Value::String("not_run".into()),
    );
    fails!("assertion-surface seam disabled", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-01")?.insert(
        "assertion_surface_expectation".into(),
        Value::String("reject_unreconciled_inventory_relation".into()),
    );
    fails!(
        "non-AS-08 assertion-surface seam enabled",
        validate(&changed)
    );

    let mut changed = snapshot.reviewed_raw.clone();
    case_mut(&mut changed, "AS-06")?["source_effect"]
        .as_object_mut()
        .ok_or_else(|| "source_effect missing".to_owned())?
        .insert("kind".into(), Value::String("vocabulary_widening".into()));
    fails!(
        "non-AS-08 vocabulary-widening classification",
        validate(&changed)
    );

    let mut changed = snapshot.reviewed_raw.clone();
    mutation_mut(
        case_mut(&mut changed, "AS-06")?,
        "append_exact",
        None,
        Some(ART_FLOOR_LABEL_APPEND),
    )?
    .insert(
        "after".into(),
        Value::String(ART_FLOOR_LABEL_APPEND.replace("Art_Floor", "Art_Mint")),
    );
    fails!("Art_Floor append operation shape drift", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    mutation_mut(
        case_mut(&mut changed, "AS-08")?,
        "replace_exact",
        Some(ADMITS_ANCHOR),
        Some(ADMITS_RICH),
    )?
    .insert(
        "after".into(),
        Value::String(format!("{ADMITS_ANCHOR}admits(\"wealthy\").\n")),
    );
    fails!("exact admits-rich widening drift", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    query_mut(case_mut(&mut changed, "AS-01")?, "false(Amend_Floor)", None)?.insert(
        "expression".into(),
        Value::String("false(Amend_Floor). ? become(Amend_Floor, Law)".into()),
    );
    fails!("query injection", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    query_mut(
        case_mut(&mut changed, "AS-03")?,
        "adjust(Amend_Targetless, $t)",
        None,
    )?
    .insert(
        "expression".into(),
        Value::String("adjust(Amend_Targetless, $T)".into()),
    );
    fails!("invalid query variable", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let purpose = query_mut(case_mut(&mut changed, "AS-01")?, "false(Amend_Floor)", None)?
        .get_mut("purpose")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "purpose missing".to_owned())?
        .to_owned();
    query_mut(case_mut(&mut changed, "AS-01")?, "false(Amend_Floor)", None)?.insert(
        "purpose".into(),
        Value::String(format!("{purpose}\n? rich(Adam).")),
    );
    fails!("pin-comment injection", validate(&changed));

    let adverse = "all $x: person($x) & ~eats($x) -> prisoner($x).";
    let mut changed = snapshot.reviewed_raw.clone();
    control_mut(case_mut(&mut changed, "AS-03")?, "accept", adverse)?.insert(
        "statement".into(),
        Value::String(format!("{adverse} rich(Adam).")),
    );
    fails!("statement injection", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    control_mut(case_mut(&mut changed, "AS-07")?, "refuse", "rich(Adam).")?
        .insert("statement".into(), Value::String(":accept.".into()));
    fails!("directive-shaped statement", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    control_mut(case_mut(&mut changed, "AS-07")?, "refuse", "rich(Adam).")?.insert(
        "statement".into(),
        Value::String("All $x: person($x) -> rich($x).".into()),
    );
    fails!("invalid statement start", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let steps = case_mut(&mut changed, "AS-09")?
        .get_mut("steps")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "AS-09 steps missing".to_owned())?;
    let pre = steps
        .iter()
        .position(|step| {
            step["type"] == "query"
                && step["expression"] == "prisoner(Cira)"
                && step["expected"] == "FALSE"
        })
        .ok_or_else(|| "AS-09 pre-harm step missing".to_owned())?;
    let rule = steps
        .iter()
        .position(|step| step["type"] == "accept" && step["statement"] == adverse)
        .ok_or_else(|| "AS-09 adverse step missing".to_owned())?;
    steps.swap(pre, rule);
    fails!("AS-09 pre/rule/post sequence reversed", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?.insert(
        "status".into(),
        Value::String("semantic_entrenchment_established".into()),
    );
    fails!("assurance promotion", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["limits"]
        .as_object_mut()
        .ok_or_else(|| "limits missing".to_owned())?
        .remove("no_new_gate");
    fails!("no-new-gate boundary removed", validate(&changed));

    for invalid_timeout in [Value::from(0), Value::Bool(true), Value::from(61)] {
        let mut changed = snapshot.reviewed_raw.clone();
        root_mut(&mut changed)?.insert("subprocess_timeout_seconds".into(), invalid_timeout);
        fails!("invalid subprocess timeout", validate(&changed));
    }

    fails!(
        "in-process timeout is named and fail-closed",
        validate_elapsed(
            "timeout control",
            Duration::from_secs(2),
            Duration::from_secs(1)
        ),
        "timeout control: in-process execution exceeded reviewed 1-second bound"
    );

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["acceptance_result"]["does_not_establish"]
        .as_array_mut()
        .ok_or_else(|| "does_not_establish missing".to_owned())?
        .retain(|value| {
            !value
                .as_str()
                .is_some_and(|text| text.contains("withholding gate"))
        });
    fails!("withholding boundary removed", validate(&changed));

    let mut changed_ledger = snapshot.ledger_raw.clone();
    root_mut(&mut changed_ledger)?["premises"]
        .as_object_mut()
        .ok_or_else(|| "ledger premises missing".to_owned())?
        .insert("rich".into(), Value::Object(Map::new()));
    fails!(
        "vocabulary seam silently contracted",
        validate_source_value(&snapshot.reviewed_raw, &changed_ledger, snapshot)
    );

    let mut duplicate_case = baseline_as03;
    let duplicate_mutation =
        mutation_mut(&mut duplicate_case, "delete_exact", Some(FLOOR_LINE), None)?.clone();
    fails!(
        "exact deletion with duplicate matches",
        apply_mutations_value(
            &format!("{}{FLOOR_LINE}", snapshot.kb_text),
            &Value::Array(vec![Value::Object(duplicate_mutation)]),
            "duplicate-match control"
        )
    );

    for marker in [
        "FINDING",
        "HARNESS ERROR",
        "NO LONGER REPRODUCE",
        "Traceback",
        "panic",
    ] {
        fails!(
            "successful pin output containing forbidden marker",
            parse_pass_count(
                &format!("nibli-pin: PASS — 1 pin\n{marker}\n"),
                "parser control"
            )
        );
    }
    fails!(
        "multiple PASS summaries",
        parse_pass_count(
            "nibli-pin: PASS — 1 pin\nnibli-pin: PASS — 1 pin\n",
            "parser control"
        )
    );

    let valid_sabotage =
        format!("FINDINGS (1) — a pinned property regressed:\n{SABOTAGE_FINAL_SUMMARY}\n");
    for returncode in [0, 2] {
        fails!(
            "sabotage marker paired with wrong rc",
            validate_sabotage_failure(returncode, &valid_sabotage)
        );
    }
    fails!(
        "duplicate sabotage final summaries",
        validate_sabotage_failure(1, &format!("{valid_sabotage}{SABOTAGE_FINAL_SUMMARY}\n"))
    );
    fails!(
        "sabotage final summary followed by output",
        validate_sabotage_failure(1, &format!("{valid_sabotage}unexpected trailing line\n"))
    );
    for marker in ["HARNESS ERROR", "NO LONGER REPRODUCE", "Traceback", "panic"] {
        fails!(
            "sabotage output contains forbidden marker",
            validate_sabotage_failure(1, &format!("{marker}\n{valid_sabotage}"))
        );
    }

    let valid_seam =
        format!("7-assertion-surface: source statement uses {SURFACE_SEAM_FRAGMENT}\n");
    for returncode in [0, 2] {
        fails!(
            "assertion seam marker paired with wrong rc",
            validate_surface_seam_failure(returncode, &valid_seam, SURFACE_SEAM_FRAGMENT)
        );
    }
    fails!(
        "unanchored assertion seam error",
        validate_surface_seam_failure(1, &format!("prefix {valid_seam}"), SURFACE_SEAM_FRAGMENT)
    );
    fails!(
        "duplicate assertion seam errors",
        validate_surface_seam_failure(
            1,
            &format!("{valid_seam}{valid_seam}"),
            SURFACE_SEAM_FRAGMENT
        )
    );
    for marker in ["Traceback", "panic"] {
        fails!(
            "assertion seam output contains forbidden marker",
            validate_surface_seam_failure(
                1,
                &format!("{marker}\n{valid_seam}"),
                SURFACE_SEAM_FRAGMENT
            )
        );
    }

    let chapter_one = REQUIRED_NARROWNESS_REFERENCES[0];
    let chapter_eight = REQUIRED_NARROWNESS_REFERENCES[1];
    let method = REQUIRED_NARROWNESS_REFERENCES[6];
    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["narrowness_impacts"]
        .as_array_mut()
        .ok_or_else(|| "narrowness impacts missing".to_owned())?
        .retain(|impact| impact["artifact_ref"] != chapter_one);
    fails!("required narrowness impact deleted", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["narrowness_impacts"]
        .as_array_mut()
        .ok_or_else(|| "narrowness impacts missing".to_owned())?
        .retain(|impact| impact["artifact_ref"] != chapter_eight);
    fails!("Chapter 8 narrowness impact deleted", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    let duplicate = impact_mut(&mut changed, chapter_one)?.clone();
    root_mut(&mut changed)?["narrowness_impacts"]
        .as_array_mut()
        .ok_or_else(|| "narrowness impacts missing".to_owned())?
        .push(Value::Object(duplicate));
    fails!(
        "duplicate narrowness artifact reference",
        validate(&changed)
    );

    let mut changed = snapshot.reviewed_raw.clone();
    impact_mut(&mut changed, chapter_one)?
        .insert("classification".into(), Value::String("unchanged".into()));
    fails!("invalid narrowness classification", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    impact_mut(&mut changed, chapter_one)?.insert(
        "artifact_ref".into(),
        Value::String(
            "book-1/01-what-counts-as-evidence.md::THIS REVIEWED ANCHOR DOES NOT EXIST".into(),
        ),
    );
    fails!("stale narrowness anchor", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    impact_mut(&mut changed, chapter_one)?.insert(
        "artifact_ref".into(),
        Value::String("book-1/01-what-counts-as-evidence.md::the".into()),
    );
    fails!("ambiguous narrowness anchor", validate(&changed));

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["cases"]
        .as_array_mut()
        .ok_or_else(|| "cases missing".to_owned())?
        .reverse();
    query_mut(
        case_mut(&mut changed, "AS-03")?,
        "adjust(Amend_Targetless, $t)",
        None,
    )?
    .insert("expected".into(), Value::String("TRUE".into()));
    fails!(
        "reordered cases retain semantic controls",
        validate(&changed)
    );

    let mut changed = snapshot.reviewed_raw.clone();
    root_mut(&mut changed)?["narrowness_impacts"]
        .as_array_mut()
        .ok_or_else(|| "narrowness impacts missing".to_owned())?
        .reverse();
    impact_mut(&mut changed, method)?
        .insert("classification".into(), Value::String("unknown".into()));
    fails!(
        "reordered narrowness entries retain semantic controls",
        validate(&changed)
    );

    fails!(
        "duplicate JSON object key",
        parse_json_no_duplicates(br#"{"status":"bounded","status":"assured"}"#)
            .map_err(|error| error.to_string())
    );
    if controls != 74 {
        return Err(format!(
            "structural negative-control inventory drifted: {controls}, expected 74"
        ));
    }
    Ok(controls)
}

struct JsonSeed;

impl<'de> DeserializeSeed<'de> for JsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E>
    where
        E: de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonSeed.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(JsonSeed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, map.next_value_seed(JsonSeed)?);
        }
        Ok(Value::Object(values))
    }
}

fn parse_json_no_duplicates(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = JsonSeed.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    #[test]
    fn live_validation_and_report_are_byte_exact() {
        let snapshot = load_snapshot(&context(), &Paths::default(), true).expect("snapshot");
        let validated = validate_source(&snapshot).expect("validate live source");
        assert_eq!(validated.candidates.len(), 9);
        let rendered = render(&snapshot).expect("render");
        assert!(
            rendered.contains("Generated by the native rights-verify amendment-semantics refresh")
        );
        assert!(rendered.contains("`./verify.sh --refresh amendment-semantics`"));
        assert!(rendered.contains("`./verify.sh --quick`"));
        assert!(rendered.contains("Authoritative execution: `./verify.sh`"));
        assert!(!rendered.contains("python3 "));
        assert_eq!(
            rendered.as_bytes(),
            snapshot.current_output.as_deref().expect("current report")
        );
    }

    #[test]
    fn exact_success_text_matches_python() {
        let report = check(&context()).expect("live amendment check");
        assert_eq!(report.structural_controls, 74);
        assert_eq!(
            report.to_string(),
            "new-book-plans/amendment-semantics-audit.md is current; 74 structural negative controls pass; execution skipped"
        );
    }

    #[test]
    fn duplicate_json_keys_are_rejected() {
        let error = parse_json_no_duplicates(br#"{"status":"bounded","status":"assured"}"#)
            .expect_err("duplicate key must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: status")
        );
    }

    #[test]
    fn assertion_ledger_projection_rejects_unknown_root_fields() {
        let snapshot = load_snapshot(&context(), &Paths::default(), false).expect("snapshot");
        let mut ledger = snapshot.ledger_raw.clone();
        ledger
            .as_object_mut()
            .expect("ledger object")
            .insert("unreviewed_root".to_owned(), Value::Bool(true));
        let error = match serde_json::from_value::<AssertionLedger>(ledger) {
            Err(error) => error,
            Ok(_) => panic!("unknown ledger root must fail"),
        };
        assert!(
            error
                .to_string()
                .contains("unknown field `unreviewed_root`")
        );
    }

    #[test]
    fn all_structural_controls_are_live_and_counted() {
        let snapshot = load_snapshot(&context(), &Paths::default(), false).expect("snapshot");
        assert_eq!(negative_controls(&snapshot).expect("negative controls"), 74);
    }

    #[test]
    fn representative_semantic_mutations_fail_closed() {
        let snapshot = load_snapshot(&context(), &Paths::default(), false).expect("snapshot");

        let mut reversed = snapshot.reviewed_raw.clone();
        query_mut(
            case_mut(&mut reversed, "AS-01").expect("AS-01"),
            "false(Amend_Floor)",
            None,
        )
        .expect("query")
        .insert("expected".into(), Value::String("FALSE".into()));
        assert!(validate_source_value(&reversed, &snapshot.ledger_raw, &snapshot).is_err());

        let mut widened = snapshot.reviewed_raw.clone();
        case_mut(&mut widened, "AS-08").expect("AS-08").insert(
            "assertion_surface_expectation".into(),
            Value::String("not_run".into()),
        );
        assert!(validate_source_value(&widened, &snapshot.ledger_raw, &snapshot).is_err());

        let assertions = snapshot.reviewed_raw["cases"]
            .as_array()
            .and_then(|cases| cases.iter().find(|case| case["id"] == "AS-03"))
            .map(|case| &case["source_assertions"])
            .expect("AS-03 assertions");
        assert!(
            validate_source_assertions_value(
                assertions,
                &format!(
                    "{}\nall $x: person($x) -> adjust (Amend_Targetless, Art_Mint).\n",
                    snapshot.kb_text
                ),
                "derived target mutation",
            )
            .is_err()
        );
    }

    #[test]
    fn candidate_groups_collapse_to_seven_line_patches() {
        let snapshot = load_snapshot(&context(), &Paths::default(), false).expect("snapshot");
        let validated = validate_source(&snapshot).expect("validate");
        let unique = validated
            .candidates
            .values()
            .map(|candidate| sha256(candidate.as_bytes()))
            .collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), 7);
        let (deletions, additions) = line_patch(&snapshot.kb_text, &validated.candidates["AS-08"]);
        assert!(deletions.is_empty());
        assert!(additions.iter().any(|line| line == "admits(\"rich\")."));
        assert!(additions.iter().any(|line| line == "rich(Adam)."));
    }

    #[test]
    fn native_fingerprint_extension_is_deterministic_and_bound() {
        let first = fingerprints(&context()).expect("fingerprints").to_string();
        let second = fingerprints(&context()).expect("fingerprints").to_string();
        assert_eq!(first, second);
        let value: Value = serde_json::from_str(&first).expect("fingerprint JSON");
        assert_eq!(
            value["candidate_source_sha256"]
                .as_object()
                .expect("candidate map")
                .len(),
            9
        );
        assert_eq!(
            value["constitution_sha256"]
                .as_str()
                .expect("constitution digest")
                .len(),
            64
        );
    }

    #[test]
    fn generation_and_execution_messages_match_python() {
        let skipped = GenerationReport {
            output: DEFAULT_OUTPUT.into(),
            structural_controls: 74,
            execution: None,
        };
        assert_eq!(
            skipped.to_string(),
            "new-book-plans/amendment-semantics-audit.md: regenerated (structural generation; execution not requested); 74 structural negative controls pass"
        );
        let executed = Report {
            output: DEFAULT_OUTPUT.into(),
            structural_controls: 74,
            execution: Some(ExecutionReport {
                cases: 9,
                pins: 44,
                sabotage_controls: 1,
                seam_controls: 1,
            }),
        };
        assert_eq!(
            executed.to_string(),
            "new-book-plans/amendment-semantics-audit.md is current; 74 structural negative controls pass; 9 isolated cases / 44 pins execute; 1 sabotage and 1 assertion-surface seam pass"
        );
    }

    #[test]
    #[ignore = "full release-engine execution; run explicitly for parity"]
    fn live_execute_matches_python_counts_and_message() {
        let report = check_execute(&context()).expect("live amendment execution");
        assert_eq!(
            report.execution,
            Some(ExecutionReport {
                cases: 9,
                pins: 44,
                sabotage_controls: 1,
                seam_controls: 1,
            })
        );
        assert_eq!(
            report.to_string(),
            "new-book-plans/amendment-semantics-audit.md is current; 74 structural negative controls pass; 9 isolated cases / 44 pins execute; 1 sabotage and 1 assertion-surface seam pass"
        );
    }
}
