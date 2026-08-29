// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native assertion-surface inventory, contract validator, and report renderer.
//!
//! The caller may provide the constitution and engine strata as one immutable
//! snapshot.  That is the fast path used by the native suite: the engine is not
//! run again and the constitution is not re-read.  With no snapshot, this
//! module reads the constitution once and obtains strata from the in-process
//! engine (or from `NIBLI_STRATA_FILE`, matching the legacy verifier hook).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};

use regex::Regex;
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{self, LoadedSource};

const DEFAULT_KB: &str = "new-book-plans/constitution.nibli";
const DEFAULT_CONTRACT: &str = "new-book-plans/assertion-surface-contracts.json";
const DEFAULT_OUTPUT: &str = "new-book-plans/assertion-surface-audit.md";

const DISPOSITIONS: [&str; 3] = ["deliberately_refused", "external", "patchable"];

const REQUIRED_TAG_BINDINGS: [(&str, &[&str]); 8] = [
    ("adulthood", &["at"]),
    ("amendment", &["adjust", "permanent", "ratifies", "suggest"]),
    ("epoch-carry", &["rotten"]),
    ("placement", &["at", "attack", "cruel", "injure", "put"]),
    ("public-body", &["public"]),
    ("release", &["free"]),
    ("roster-person", &["person"]),
    ("seating", &["choose"]),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Check,
    Fingerprints,
    Generate,
}

/// Immutable inputs already owned by the verifier orchestration layer.
///
/// Supplying only `constitution` still avoids a source re-read and lets this
/// module obtain strata from the in-process engine.  Supplying both fields
/// consumes an upstream strata result without starting or rebuilding anything.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputSnapshot<'a> {
    pub(crate) constitution: Option<&'a str>,
    pub(crate) strata_tsv: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Statement {
    text: String,
    line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Edge {
    dependency: String,
    negative: bool,
}

#[derive(Clone, Debug, Default)]
struct SourceInventory {
    admitted: BTreeSet<String>,
    derived_only: BTreeSet<String>,
    ground_asserted: BTreeSet<String>,
    producers: BTreeMap<String, Vec<Arc<str>>>,
    consumers: BTreeMap<String, Vec<(bool, Arc<str>)>>,
    rules: Vec<Arc<str>>,
    facts: Vec<Arc<str>>,
}

#[derive(Clone, Debug)]
struct Inventory {
    strata: BTreeMap<String, i64>,
    derived: BTreeSet<String>,
    edges: BTreeMap<String, Vec<Edge>>,
    admitted: BTreeSet<String>,
    derived_only: BTreeSet<String>,
    ground_asserted: BTreeSet<String>,
    rules_sha256: String,
    facts_sha256: String,
    route_fingerprints: BTreeMap<String, String>,
    statement_fingerprints: Arc<Vec<StatementFingerprint>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    spdx: String,
    schema_version: u64,
    #[serde(default)]
    aliases: BTreeMap<String, String>,
    cheapest_harm_metric: String,
    risk_disposition_meanings: BTreeMap<RiskDisposition, String>,
    required_semantic_tags: Vec<String>,
    additional_writable_channels: Vec<String>,
    rules_sha256: String,
    facts_sha256: String,
    route_fingerprints: BTreeMap<String, String>,
    #[serde(default)]
    reserved_retired_relations: BTreeMap<String, ReservedRetirement>,
    derived_relations: BTreeMap<String, DerivedContract>,
    premises: BTreeMap<String, PremiseContract>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum Classification {
    DerivedOnly,
    MixedBaseFact,
    PendingInterface,
}

impl Classification {
    fn as_str(self) -> &'static str {
        match self {
            Self::DerivedOnly => "derived_only",
            Self::MixedBaseFact => "mixed_base_fact",
            Self::PendingInterface => "pending_interface",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum RiskDisposition {
    DeliberatelyRefused,
    External,
    Patchable,
}

impl RiskDisposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::DeliberatelyRefused => "deliberately_refused",
            Self::External => "external",
            Self::Patchable => "patchable",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Operation {
    Assert,
    WithholdOrDelete,
}

impl Operation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Assert => "assert",
            Self::WithholdOrDelete => "withhold_or_delete",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedPosture {
    rule_produced: bool,
    admitted: bool,
    derived_only: bool,
    ground_asserted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DerivedContract {
    classification: Classification,
    contract_id: String,
    decision_ref: String,
    expected: ExpectedPosture,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReservedRetirement {
    retired_at: String,
    reason: String,
    must_not_become: String,
    source_ref: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HarmScenario {
    operation: Operation,
    operations: Vec<String>,
    effect: String,
    target: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PremiseContract {
    claimed_actor: String,
    tuple_claim: String,
    current_writer_authority: String,
    required_writer_authority: String,
    current_provenance: String,
    required_provenance: String,
    cheapest_harm: HarmScenario,
    withholding_deletion_harm: HarmScenario,
    current_challenge_route: String,
    required_challenge_route: String,
    risk_dispositions: Vec<RiskDisposition>,
    #[serde(default)]
    refused_alternative: Option<String>,
    owner_ref: String,
    tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RoutePayload {
    source_producers: Vec<String>,
    source_consumers: Vec<String>,
    engine_dependencies: Vec<String>,
    engine_readers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct StatementFingerprint {
    id: String,
    kind: String,
    occurrence: u64,
    statement: String,
    statement_sha256: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct FingerprintOutput<'a> {
    facts_sha256: &'a str,
    route_fingerprints: &'a BTreeMap<String, String>,
    rules_sha256: &'a str,
    statement_fingerprints: &'a [StatementFingerprint],
}

impl Inventory {
    fn writable(&self) -> BTreeSet<String> {
        self.admitted
            .difference(&self.derived_only)
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditError(String);

impl AuditError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

type AuditResult<T> = Result<T, AuditError>;

/// Run the native equivalent of the legacy script's three modes.
///
/// The returned text is the exact stdout line/body before the CLI's final
/// newline. `Generate` installs the report through `fs::write`, as the legacy
/// generator does; `Fingerprints` never validates the reviewed
/// contract; `Check` validates, runs controls, and compares report bytes.
pub(crate) fn run(
    context: &Context,
    mode: Mode,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run_inner(context, mode, snapshot)
        .map_err(|error| Error::new(format!("7-assertion-surface: {error}")))
}

pub(crate) fn check(
    context: &Context,
    strata_tsv: Option<&str>,
    constitution: Option<&str>,
) -> Result<String, Error> {
    run(
        context,
        Mode::Check,
        InputSnapshot {
            constitution,
            strata_tsv,
        },
    )
}

pub(crate) fn fingerprints(
    context: &Context,
    strata_tsv: Option<&str>,
    constitution: Option<&str>,
) -> Result<String, Error> {
    run(
        context,
        Mode::Fingerprints,
        InputSnapshot {
            constitution,
            strata_tsv,
        },
    )
}

pub(crate) fn generate(
    context: &Context,
    strata_tsv: Option<&str>,
    constitution: Option<&str>,
) -> Result<String, Error> {
    run(
        context,
        Mode::Generate,
        InputSnapshot {
            constitution,
            strata_tsv,
        },
    )
}

fn run_inner(context: &Context, mode: Mode, snapshot: InputSnapshot<'_>) -> AuditResult<String> {
    let contract_text = read_context(context, DEFAULT_CONTRACT, "contract")?;
    let (contract, contract_control_value) = load_contract(&contract_text, DEFAULT_CONTRACT)?;
    let source_owned;
    let source = if let Some(source) = snapshot.constitution {
        source
    } else {
        source_owned = read_context(context, DEFAULT_KB, "constitution")?;
        &source_owned
    };
    let strata_owned;
    let strata = if let Some(strata) = snapshot.strata_tsv {
        strata
    } else if let Some(cache) = std::env::var_os("NIBLI_STRATA_FILE") {
        let path = PathBuf::from(cache);
        strata_owned = std::fs::read_to_string(&path).map_err(|error| {
            AuditError::new(format!(
                "cannot read NIBLI_STRATA_FILE {}: {error}",
                path.display()
            ))
        })?;
        &strata_owned
    } else if snapshot.constitution.is_none()
        && let Some(pin_path) = std::env::var_os("NIBLI_PIN")
    {
        let pin_path = PathBuf::from(pin_path);
        if !pin_path.is_file() {
            return Err(AuditError::new(format!(
                "no nibli-pin at {} — build it release, or set NIBLI_PIN",
                pin_path.display()
            )));
        }
        let output = Command::new(&pin_path)
            .args(["--strata", "--kb", DEFAULT_KB])
            .current_dir(context.root())
            .output()
            .map_err(|error| {
                AuditError::new(format!("cannot run {}: {error}", pin_path.display()))
            })?;
        if !output.status.success() {
            return Err(AuditError::new(format!(
                "nibli-pin --strata failed:\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        strata_owned = String::from_utf8(output.stdout)
            .map_err(|error| AuditError::new(format!("nibli-pin output is not UTF-8: {error}")))?;
        &strata_owned
    } else {
        let output = pin::dump_strata(&[LoadedSource::new(DEFAULT_KB, source)]);
        if output.exit_code != pin::EXIT_OK {
            return Err(AuditError::new(format!(
                "nibli-pin --strata failed:\n{}{}",
                output.stdout, output.stderr
            )));
        }
        strata_owned = output.stdout;
        &strata_owned
    };

    let inventory = make_inventory(
        strata,
        source,
        &contract.aliases,
        mode == Mode::Fingerprints,
    )?;
    if mode == Mode::Fingerprints {
        return fingerprint_output(&inventory);
    }

    let mut references = ReferenceResolver::new(context);
    validate_contract_typed(&contract, &inventory, &mut references)?;
    let generated = render(&contract, &inventory)?;
    match mode {
        Mode::Fingerprints => unreachable!(),
        Mode::Check => {
            let controls =
                negative_controls(&contract_control_value, &inventory, source, &mut references)?;
            let current = read_context(context, DEFAULT_OUTPUT, "generated audit")?;
            if current != generated {
                return Err(AuditError::new(format!(
                    "{DEFAULT_OUTPUT} is STALE — rerun without --check"
                )));
            }
            Ok(format!(
                "{DEFAULT_OUTPUT} is current; {controls} negative controls pass"
            ))
        }
        Mode::Generate => {
            std::fs::write(context.path(DEFAULT_OUTPUT), generated).map_err(|error| {
                AuditError::new(format!(
                    "cannot write generated audit {DEFAULT_OUTPUT}: {error}"
                ))
            })?;
            Ok(format!("{DEFAULT_OUTPUT}: regenerated"))
        }
    }
}

fn read_context(context: &Context, relative: &str, kind: &str) -> AuditResult<String> {
    std::fs::read_to_string(context.path(relative))
        .map_err(|error| AuditError::new(format!("cannot read {kind} {relative}: {error}")))
}

fn is_artifact(name: &str) -> bool {
    name == "event" || name.starts_with("__abs_")
}

fn is_builtin(name: &str) -> bool {
    name == "equals"
}

fn name_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^[a-z_][a-z0-9_]*$").expect("valid name regex"))
}

fn call_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r"(?P<negative>~\s*)?(?P<name>[a-z_][a-z0-9_]*)\s*\(").expect("valid call regex")
    })
}

fn declaration_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r#"^(admits|derived_only)\("([a-z_][a-z0-9_]*)"\)$"#)
            .expect("valid declaration regex")
    })
}

fn floor_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(
            r"^entitled\(\s*every\s+([a-z_][a-z0-9_]*)\s*,\s*event\s*\{\s*([a-z_][a-z0-9_]*)\s*\([^)]*\)\s*\}\s*\)$",
        )
        .expect("valid floor regex")
    })
}

fn every_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"\bevery\s+([a-z_][a-z0-9_]*)\b").expect("valid every regex"))
}

fn placeholder_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r"(?i)^(?:tbd|todo|unknown|n/?a|pending)$").expect("valid placeholder regex")
    })
}

fn parse_engine(
    text: &str,
) -> AuditResult<(
    BTreeMap<String, i64>,
    BTreeSet<String>,
    BTreeMap<String, Vec<Edge>>,
)> {
    let mut strata = BTreeMap::new();
    let mut derived = BTreeSet::new();
    let mut edges = BTreeMap::new();
    for raw_line in text.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let name = parts[0];
        if is_artifact(name) || is_builtin(name) {
            continue;
        }
        let stratum = parts[1]
            .parse::<i64>()
            .map_err(|_| AuditError::new(format!("invalid stratum row: {line}")))?;
        strata.insert(name.to_owned(), stratum);
        if parts[2] != "base" {
            derived.insert(name.to_owned());
        }
        let mut parsed = Vec::new();
        if let Some(raw_edges) = parts.get(3).filter(|value| !value.is_empty()) {
            for raw in raw_edges
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let Some(sign) = raw.as_bytes().first().copied() else {
                    continue;
                };
                if sign != b'+' && sign != b'-' {
                    return Err(AuditError::new(format!(
                        "invalid signed edge in strata row: {line}"
                    )));
                }
                let dependency = &raw[1..];
                if is_artifact(dependency) || is_builtin(dependency) {
                    continue;
                }
                parsed.push(Edge {
                    dependency: dependency.to_owned(),
                    negative: sign == b'-',
                });
            }
        }
        edges.insert(name.to_owned(), parsed);
    }
    if strata.is_empty() {
        return Err(AuditError::new(
            "nibli-pin --strata produced no relation rows",
        ));
    }
    Ok((strata, derived, edges))
}

fn lex_statements(source: &str) -> AuditResult<Vec<Statement>> {
    let mut statements = Vec::new();
    let mut buffer = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;
    let mut line = 1usize;
    let mut start_line = None;

    for character in source.chars() {
        if in_comment {
            if character == '\n' {
                in_comment = false;
                if !buffer.is_empty() {
                    buffer.push(' ');
                }
                line += 1;
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
            line += 1;
            escaped = false;
            continue;
        }
        if start_line.is_none() && !character.is_whitespace() {
            start_line = Some(line);
        }
        if character == '"' && !escaped {
            in_string = !in_string;
        }
        if character == '.' && !in_string {
            let normalized = buffer.split_whitespace().collect::<Vec<_>>().join(" ");
            if !normalized.is_empty() {
                statements.push(Statement {
                    text: normalized,
                    line: start_line.unwrap_or(line),
                });
            }
            buffer.clear();
            start_line = None;
            escaped = false;
            continue;
        }
        buffer.push(character);
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }
    let residue = buffer.trim();
    if in_string {
        return Err(AuditError::new(
            "unterminated string while reading constitution",
        ));
    }
    if !residue.is_empty() {
        let preview: String = residue.chars().take(80).collect();
        return Err(AuditError::new(format!(
            "unterminated active statement at line {}: {preview}",
            start_line.unwrap_or(line)
        )));
    }
    Ok(statements)
}

fn canonical(name: &str, aliases: &BTreeMap<String, String>) -> String {
    aliases
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_owned())
}

fn calls(text: &str, aliases: &BTreeMap<String, String>) -> Vec<(String, bool)> {
    call_regex()
        .captures_iter(text)
        .map(|captures| {
            (
                canonical(&captures["name"], aliases),
                captures.name("negative").is_some(),
            )
        })
        .collect()
}

fn parse_source(
    source: &str,
    known_relations: &BTreeSet<String>,
    expected_heads: &BTreeSet<String>,
    aliases: &BTreeMap<String, String>,
) -> AuditResult<SourceInventory> {
    let mut result = SourceInventory::default();
    let mut executable_seen = false;

    for statement in lex_statements(source)? {
        if let Some(declaration) = declaration_regex().captures(&statement.text) {
            let kind = &declaration[1];
            let name = canonical(&declaration[2], aliases);
            if executable_seen {
                return Err(AuditError::new(format!(
                    "late {kind} declaration for {name} at constitution line {}; declarations must precede executable content",
                    statement.line
                )));
            }
            let declarations = if kind == "admits" {
                &mut result.admitted
            } else {
                &mut result.derived_only
            };
            if !declarations.insert(name.clone()) {
                return Err(AuditError::new(format!(
                    "duplicate {kind} declaration for {name}"
                )));
            }
            continue;
        }

        executable_seen = true;
        // A long rule can be indexed under dozens of producer/consumer
        // relations. Share its bytes across those indexes; cloning the String
        // at every edge made the 5.6 MB constitution transiently occupy nearly
        // 3 GB during the assertion audit.
        let text: Arc<str> = Arc::from(statement.text);
        let mut producers = Vec::new();
        let mut consumers = Vec::new();
        let mut ground_names = Vec::new();
        let mut is_rule = false;
        let outer;

        if let Some((body, head)) = text.rsplit_once("->") {
            producers = calls(head, aliases)
                .into_iter()
                .map(|(name, _)| name)
                .collect();
            consumers = calls(body, aliases);
            is_rule = true;
            outer = Vec::new();
        } else if let Some(floor) = floor_regex().captures(&text) {
            producers = vec!["entitled".to_owned(), canonical(&floor[2], aliases)];
            consumers = vec![(canonical(&floor[1], aliases), false)];
            is_rule = true;
            outer = Vec::new();
        } else {
            outer = calls(&text, aliases);
            if let Some(universal) = every_regex().captures(&text).filter(|_| !outer.is_empty()) {
                producers = vec![outer[0].0.clone()];
                consumers = vec![(canonical(&universal[1], aliases), false)];
                is_rule = true;
            } else if !outer.is_empty() {
                ground_names = outer
                    .iter()
                    .filter(|(name, _)| !is_builtin(name))
                    .map(|(name, _)| name.clone())
                    .collect();
                result.ground_asserted.extend(ground_names.iter().cloned());
                result.facts.push(Arc::clone(&text));
            }
        }

        let mentioned: BTreeSet<_> = producers
            .iter()
            .chain(ground_names.iter())
            .chain(consumers.iter().map(|(name, _)| name))
            .cloned()
            .collect();
        let unknown: Vec<_> = mentioned
            .iter()
            .filter(|name| !known_relations.contains(*name) && !is_builtin(name))
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(AuditError::new(format!(
                "source statement at line {} uses relations absent from engine inventory or alias contract: {}",
                statement.line,
                unknown.join(", ")
            )));
        }
        if is_rule {
            if producers.is_empty() {
                return Err(AuditError::new(format!(
                    "rule at line {} has no parsed head: {text}",
                    statement.line
                )));
            }
            result.rules.push(Arc::clone(&text));
            for name in producers {
                result
                    .producers
                    .entry(name)
                    .or_default()
                    .push(Arc::clone(&text));
            }
            for (name, negative) in consumers {
                if !is_builtin(&name) {
                    result
                        .consumers
                        .entry(name)
                        .or_default()
                        .push((negative, Arc::clone(&text)));
                }
            }
        } else if outer.is_empty() {
            return Err(AuditError::new(format!(
                "unrecognized active statement at line {}: {text}",
                statement.line
            )));
        }
    }

    let source_heads: BTreeSet<_> = result.producers.keys().cloned().collect();
    if &source_heads != expected_heads {
        let missing: Vec<_> = expected_heads.difference(&source_heads).cloned().collect();
        let extra: Vec<_> = source_heads.difference(expected_heads).cloned().collect();
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!("engine-only heads: {}", missing.join(", ")));
        }
        if !extra.is_empty() {
            details.push(format!("source-only heads: {}", extra.join(", ")));
        }
        return Err(AuditError::new(format!(
            "authored-head reconciliation failed: {}",
            details.join("; ")
        )));
    }
    Ok(result)
}

fn route_payload(
    relation: &str,
    source: &SourceInventory,
    edges: &BTreeMap<String, Vec<Edge>>,
) -> RoutePayload {
    let mut readers = Vec::new();
    for (head, dependencies) in edges {
        for edge in dependencies {
            if edge.dependency == relation {
                readers.push(format!("{}{}", if edge.negative { '-' } else { '+' }, head));
            }
        }
    }
    readers.sort();

    let mut producers = source.producers.get(relation).cloned().unwrap_or_default();
    producers.sort();
    let source_consumers: BTreeSet<_> = source
        .consumers
        .get(relation)
        .into_iter()
        .flatten()
        .map(|(negative, rule)| format!("{}\t{rule}", if *negative { '-' } else { '+' }))
        .collect();
    let mut dependencies: Vec<_> = edges
        .get(relation)
        .into_iter()
        .flatten()
        .map(|edge| {
            format!(
                "{}{}",
                if edge.negative { '-' } else { '+' },
                edge.dependency
            )
        })
        .collect();
    dependencies.sort();

    RoutePayload {
        source_producers: producers
            .into_iter()
            .map(|value| value.to_string())
            .collect(),
        source_consumers: source_consumers.into_iter().collect(),
        engine_dependencies: dependencies,
        engine_readers: readers,
    }
}

fn statement_fingerprint_records(source: &str) -> AuditResult<Vec<StatementFingerprint>> {
    let mut occurrences: HashMap<String, u64> = HashMap::new();
    let mut records = Vec::new();
    for statement in lex_statements(source)? {
        let occurrence = *occurrences.get(&statement.text).unwrap_or(&0);
        occurrences.insert(statement.text.clone(), occurrence + 1);
        let id = sha256_serializable(&(statement.text.as_str(), occurrence));
        let statement_sha256 = sha256_serializable(&statement.text);
        let kind = if statement.text.trim_start().starts_with("all ")
            || statement.text.trim_start().starts_with("any ")
        {
            "rule"
        } else {
            "fact"
        };
        records.push(StatementFingerprint {
            id,
            kind: kind.to_owned(),
            occurrence,
            statement: statement.text,
            statement_sha256,
        });
    }
    Ok(records)
}

fn make_inventory(
    strata_text: &str,
    source_text: &str,
    contract_aliases: &BTreeMap<String, String>,
    include_statement_fingerprints: bool,
) -> AuditResult<Inventory> {
    let (strata, derived, edges) = parse_engine(strata_text)?;
    let mut aliases = BTreeMap::new();
    for (key, value) in contract_aliases {
        if !name_regex().is_match(key) || !name_regex().is_match(value) {
            return Err(AuditError::new(
                "aliases must map relation names to relation names",
            ));
        }
        aliases.insert(key.clone(), value.clone());
    }
    let alias_targets: BTreeSet<_> = aliases.values().cloned().collect();
    let missing_targets: Vec<_> = alias_targets
        .difference(&strata.keys().cloned().collect())
        .cloned()
        .collect();
    if !missing_targets.is_empty() {
        return Err(AuditError::new(format!(
            "aliases target relations absent from the engine inventory: {}",
            missing_targets.join(", ")
        )));
    }
    let alias_names: BTreeSet<_> = aliases.keys().cloned().collect();
    let strata_names: BTreeSet<_> = strata.keys().cloned().collect();
    let conflicts: Vec<_> = alias_names.intersection(&strata_names).cloned().collect();
    if !conflicts.is_empty() {
        return Err(AuditError::new(format!(
            "alias names collide with canonical engine relations: {}",
            conflicts.join(", ")
        )));
    }
    let source = parse_source(source_text, &strata_names, &derived, &aliases)?;
    let writable: BTreeSet<_> = source
        .admitted
        .difference(&source.derived_only)
        .cloned()
        .collect();
    let relations: BTreeSet<_> = derived.union(&writable).cloned().collect();
    let route_fingerprints = relations
        .iter()
        .map(|relation| {
            (
                relation.clone(),
                sha256_serializable(&route_payload(relation, &source, &edges)),
            )
        })
        .collect();
    let mut rules = source.rules.clone();
    rules.sort();
    let mut facts = source.facts.clone();
    facts.sort();
    Ok(Inventory {
        strata,
        derived,
        edges,
        admitted: source.admitted,
        derived_only: source.derived_only,
        ground_asserted: source.ground_asserted,
        rules_sha256: sha256_serializable(
            &rules
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        ),
        facts_sha256: sha256_serializable(
            &facts
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
        ),
        route_fingerprints,
        statement_fingerprints: Arc::new(if include_statement_fingerprints {
            statement_fingerprint_records(source_text)?
        } else {
            Vec::new()
        }),
    })
}

fn canonical_json(value: &Value, output: &mut String) {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => {
            output
                .push_str(&serde_json::to_string(value).expect("string serialization cannot fail"));
        }
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                canonical_json(value, output);
            }
            output.push(']');
        }
        Value::Object(values) => {
            output.push('{');
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output
                    .push_str(&serde_json::to_string(key).expect("key serialization cannot fail"));
                output.push(':');
                canonical_json(&values[key], output);
            }
            output.push('}');
        }
    }
}

fn sha256_json(value: &Value) -> String {
    let mut encoded = String::new();
    canonical_json(value, &mut encoded);
    sha256(encoded)
}

fn sha256_serializable(value: &impl Serialize) -> String {
    let value = serde_json::to_value(value).expect("typed canonical projection serializes");
    sha256_json(&value)
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, map.next_value::<UniqueValue>()?.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

fn load_contract(text: &str, path: &str) -> AuditResult<(Contract, Value)> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = UniqueValue::deserialize(&mut deserializer)
        .map_err(|error| AuditError::new(format!("cannot read contract {path}: {error}")))?
        .0;
    deserializer
        .end()
        .map_err(|error| AuditError::new(format!("cannot read contract {path}: {error}")))?;
    let contract = serde_json::from_str(text)
        .map_err(|error| AuditError::new(format!("cannot read contract {path}: {error}")))?;
    Ok((contract, value))
}

fn contract_from_value(value: &Value) -> AuditResult<Contract> {
    serde_json::from_value(value.clone())
        .map_err(|error| AuditError::new(format!("contract schema is invalid: {error}")))
}

fn validate_text<'a>(raw: &'a str, path: &str) -> AuditResult<&'a str> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(AuditError::new(format!("{path} must be non-empty text")));
    }
    if placeholder_regex().is_match(value) {
        return Err(AuditError::new(format!(
            "{path} contains placeholder value {raw:?}"
        )));
    }
    Ok(value)
}

struct ReferenceResolver<'a> {
    context: &'a Context,
    repository_root: PathBuf,
    files: HashMap<PathBuf, String>,
    validated: HashSet<String>,
}

impl<'a> ReferenceResolver<'a> {
    fn new(context: &'a Context) -> Self {
        let repository_root =
            std::fs::canonicalize(context.root()).unwrap_or_else(|_| context.root().to_path_buf());
        Self {
            context,
            repository_root,
            files: HashMap::new(),
            validated: HashSet::new(),
        }
    }

    fn validate(&mut self, value: &str, path: &str) -> AuditResult<String> {
        let reference = validate_text(value, path)?.to_owned();
        let Some((raw_file, needle)) = reference.split_once("::") else {
            return Err(AuditError::new(format!(
                "{path} must use path::stable text, not a line number"
            )));
        };
        if raw_file.trim().is_empty() || needle.trim().is_empty() {
            return Err(AuditError::new(format!(
                "{path} must name both a file and non-empty stable text"
            )));
        }
        if self.validated.contains(&reference) {
            return Ok(reference);
        }

        let raw_path = Path::new(raw_file);
        let unresolved = if raw_path.is_absolute() {
            raw_path.to_path_buf()
        } else {
            self.context.path(raw_path)
        };
        if !unresolved.is_file() {
            return Err(AuditError::new(format!(
                "{path} references missing file {raw_file}"
            )));
        }
        let target = std::fs::canonicalize(&unresolved)
            .map_err(|_| AuditError::new(format!("{path} references missing file {raw_file}")))?;
        if !target.starts_with(&self.repository_root) {
            return Err(AuditError::new(format!(
                "{path} must reference a repository-local file"
            )));
        }
        if !self.files.contains_key(&target) {
            let content = std::fs::read_to_string(&target).map_err(|error| {
                AuditError::new(format!("{path} cannot read {raw_file}: {error}"))
            })?;
            self.files.insert(target.clone(), content);
        }
        let occurrences = self.files[&target].matches(needle).count();
        if occurrences == 0 {
            return Err(AuditError::new(format!(
                "{path} reference text is stale in {raw_file}: {needle:?}"
            )));
        }
        if occurrences != 1 {
            return Err(AuditError::new(format!(
                "{path} reference text must identify one location in {raw_file}; matched {occurrences}"
            )));
        }
        self.validated.insert(reference.clone());
        Ok(reference)
    }
}

fn shortest_path(
    inventory: &Inventory,
    source: &str,
    target: &str,
) -> Option<Vec<(String, String, bool)>> {
    if source == target {
        return Some(Vec::new());
    }
    let mut reverse: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    for (head, dependencies) in &inventory.edges {
        for edge in dependencies {
            reverse
                .entry(edge.dependency.clone())
                .or_default()
                .push((head.clone(), edge.negative));
        }
    }
    for values in reverse.values_mut() {
        values.sort();
    }
    let mut queue = VecDeque::from([(source.to_owned(), Vec::new())]);
    let mut seen = BTreeSet::from([source.to_owned()]);
    while let Some((current, path)) = queue.pop_front() {
        if let Some(next) = reverse.get(&current) {
            for (head, negative) in next {
                let mut route = path.clone();
                route.push((current.clone(), head.clone(), *negative));
                if head == target {
                    return Some(route);
                }
                if seen.insert(head.clone()) {
                    queue.push_back((head.clone(), route));
                }
            }
        }
    }
    None
}

fn validate_scenario(
    relation: &str,
    expected_operation: Operation,
    scenario: &HarmScenario,
    inventory: &Inventory,
    path: &str,
    references: &mut ReferenceResolver<'_>,
) -> AuditResult<()> {
    if scenario.operation != expected_operation {
        return Err(AuditError::new(format!(
            "{path}.operation must be {:?}",
            expected_operation.as_str()
        )));
    }
    if scenario.operations.is_empty() {
        return Err(AuditError::new(format!(
            "{path}.operations must be a non-empty list"
        )));
    }
    let mut normalized = Vec::new();
    for (index, value) in scenario.operations.iter().enumerate() {
        normalized.push(validate_text(
            value,
            &format!("{path}.operations[{index}]"),
        )?);
    }
    if normalized.iter().copied().collect::<HashSet<_>>().len() != normalized.len() {
        return Err(AuditError::new(format!(
            "{path}.operations contains duplicates"
        )));
    }
    let relation_call = Regex::new(&format!(r"\b{}\s*\(", regex::escape(relation)))
        .expect("escaped relation call regex is valid");
    if !normalized.iter().any(|value| relation_call.is_match(value)) {
        return Err(AuditError::new(format!(
            "{path}.operations must include an operation on {relation}(...)"
        )));
    }
    validate_text(&scenario.effect, &format!("{path}.effect"))?;
    let target = validate_text(&scenario.target, &format!("{path}.target"))?;
    if !inventory.derived.contains(target) {
        return Err(AuditError::new(format!(
            "{path}.target {target:?} is not a current derived relation"
        )));
    }
    if shortest_path(inventory, relation, target).is_none() {
        return Err(AuditError::new(format!(
            "{path}.target {target:?} has no engine dependency path from {relation:?}"
        )));
    }
    references.validate(&scenario.evidence_ref, &format!("{path}.evidence_ref"))?;
    Ok(())
}

fn validate_reserved_retirements(
    contract: &Contract,
    inventory: &Inventory,
    references: &mut ReferenceResolver<'_>,
) -> AuditResult<BTreeSet<String>> {
    for (relation, entry) in &contract.reserved_retired_relations {
        let path = format!("reserved_retired_relations.{relation}");
        validate_text(&entry.retired_at, &format!("{path}.retired_at"))?;
        validate_text(&entry.reason, &format!("{path}.reason"))?;
        validate_text(&entry.must_not_become, &format!("{path}.must_not_become"))?;
        references.validate(&entry.source_ref, &format!("{path}.source_ref"))?;
        if !inventory.derived_only.contains(relation) {
            return Err(AuditError::new(format!(
                "{path}: a reserved retirement must carry the derived_only declaration; omission from admits is the defect, not the fix"
            )));
        }
        if inventory.derived.contains(relation) {
            return Err(AuditError::new(format!(
                "{path}: a reserved retirement has no producer; this relation is derived, so it is live vocabulary and not retired"
            )));
        }
        if inventory.admitted.contains(relation) {
            return Err(AuditError::new(format!(
                "{path}: a reserved retirement may not be admitted; the admission is exactly the ground write it exists to refuse"
            )));
        }
        if inventory.ground_asserted.contains(relation) {
            return Err(AuditError::new(format!(
                "{path}: a reserved retirement carries a ground fact, so the write it claims to refuse has already happened"
            )));
        }
    }
    Ok(contract
        .reserved_retired_relations
        .keys()
        .cloned()
        .collect())
}

fn validate_contract_typed(
    contract: &Contract,
    inventory: &Inventory,
    references: &mut ReferenceResolver<'_>,
) -> AuditResult<()> {
    if contract.schema_version != 1 {
        return Err(AuditError::new("schema_version must be 1"));
    }
    if contract.spdx != "CC-BY-4.0" {
        return Err(AuditError::new("contract spdx must be CC-BY-4.0"));
    }
    validate_text(&contract.cheapest_harm_metric, "cheapest_harm_metric")?;
    let meaning_keys: BTreeSet<_> = contract.risk_disposition_meanings.keys().copied().collect();
    let dispositions = BTreeSet::from([
        RiskDisposition::DeliberatelyRefused,
        RiskDisposition::External,
        RiskDisposition::Patchable,
    ]);
    if meaning_keys != dispositions {
        return Err(AuditError::new(
            "risk_disposition_meanings must define every allowed disposition exactly",
        ));
    }
    for (disposition, meaning) in &contract.risk_disposition_meanings {
        validate_text(
            meaning,
            &format!("risk_disposition_meanings.{}", disposition.as_str()),
        )?;
    }

    let derived_entries = &contract.derived_relations;
    let registered: BTreeSet<_> = derived_entries.keys().cloned().collect();
    if inventory.derived != registered {
        let missing: Vec<_> = inventory.derived.difference(&registered).cloned().collect();
        let extra: Vec<_> = registered.difference(&inventory.derived).cloned().collect();
        let mut detail = Vec::new();
        if !missing.is_empty() {
            detail.push(format!(
                "unclassified derived relations: {}",
                missing.join(", ")
            ));
        }
        if !extra.is_empty() {
            detail.push(format!("stale derived contracts: {}", extra.join(", ")));
        }
        return Err(AuditError::new(detail.join("; ")));
    }

    let mut ids = HashSet::new();
    for relation in &inventory.derived {
        let path = format!("derived_relations.{relation}");
        let entry = &derived_entries[relation];
        let contract_id = validate_text(&entry.contract_id, &format!("{path}.contract_id"))?;
        if !ids.insert(contract_id.to_owned()) {
            return Err(AuditError::new(format!(
                "duplicate contract_id {contract_id:?}"
            )));
        }
        references.validate(&entry.decision_ref, &format!("{path}.decision_ref"))?;
        let actual = posture_value(relation, inventory);
        if entry.expected.rule_produced != actual.rule_produced
            || entry.expected.admitted != actual.admitted
            || entry.expected.derived_only != actual.derived_only
            || entry.expected.ground_asserted != actual.ground_asserted
        {
            return Err(AuditError::new(format!(
                "{relation} posture changed: expected {}, actual {}",
                serde_json::to_string(&entry.expected).expect("posture serializes"),
                serde_json::to_string(&actual).expect("posture serializes")
            )));
        }
        if entry.classification == Classification::DerivedOnly && !actual.derived_only {
            return Err(AuditError::new(format!(
                "{relation} is classified derived_only without the declaration"
            )));
        }
        if entry.classification == Classification::MixedBaseFact
            && !(actual.admitted && !actual.derived_only)
        {
            return Err(AuditError::new(format!(
                "{relation} mixed contract does not match raw posture"
            )));
        }
        if entry.classification == Classification::PendingInterface
            && (actual.admitted || actual.derived_only)
        {
            return Err(AuditError::new(format!(
                "{relation} pending contract does not match raw posture"
            )));
        }
    }

    let reserved = validate_reserved_retirements(contract, inventory, references)?;
    let undeclared_guards: Vec<_> = inventory
        .derived_only
        .difference(&inventory.derived)
        .filter(|relation| !reserved.contains(*relation))
        .cloned()
        .collect();
    if !undeclared_guards.is_empty() {
        return Err(AuditError::new(format!(
            "derived_only declarations without a current producer: {}",
            undeclared_guards.join(", ")
        )));
    }
    if !contract.additional_writable_channels.is_empty() {
        return Err(AuditError::new(
            "additional_writable_channels must currently be []; add schema support and a named channel contract before using it",
        ));
    }
    let premise_entries = &contract.premises;
    let expected_premises = inventory.writable();
    let registered_premises: BTreeSet<_> = premise_entries.keys().cloned().collect();
    if expected_premises != registered_premises {
        let missing: Vec<_> = expected_premises
            .difference(&registered_premises)
            .cloned()
            .collect();
        let extra: Vec<_> = registered_premises
            .difference(&expected_premises)
            .cloned()
            .collect();
        let mut detail = Vec::new();
        if !missing.is_empty() {
            detail.push(format!(
                "unreviewed writable premises: {}",
                missing.join(", ")
            ));
        }
        if !extra.is_empty() {
            detail.push(format!(
                "stale/non-writable premise contracts: {}",
                extra.join(", ")
            ));
        }
        return Err(AuditError::new(detail.join("; ")));
    }

    let mut seen_tags = BTreeSet::new();
    for relation in &expected_premises {
        let path = format!("premises.{relation}");
        let entry = &premise_entries[relation];
        validate_text(&entry.claimed_actor, &format!("{path}.claimed_actor"))?;
        validate_text(&entry.tuple_claim, &format!("{path}.tuple_claim"))?;
        validate_text(
            &entry.current_writer_authority,
            &format!("{path}.current_writer_authority"),
        )?;
        validate_text(
            &entry.required_writer_authority,
            &format!("{path}.required_writer_authority"),
        )?;
        validate_text(
            &entry.current_provenance,
            &format!("{path}.current_provenance"),
        )?;
        validate_text(
            &entry.required_provenance,
            &format!("{path}.required_provenance"),
        )?;
        validate_text(
            &entry.current_challenge_route,
            &format!("{path}.current_challenge_route"),
        )?;
        validate_text(
            &entry.required_challenge_route,
            &format!("{path}.required_challenge_route"),
        )?;
        validate_scenario(
            relation,
            Operation::Assert,
            &entry.cheapest_harm,
            inventory,
            &format!("{path}.cheapest_harm"),
            references,
        )?;
        validate_scenario(
            relation,
            Operation::WithholdOrDelete,
            &entry.withholding_deletion_harm,
            inventory,
            &format!("{path}.withholding_deletion_harm"),
            references,
        )?;
        let row_dispositions = &entry.risk_dispositions;
        if row_dispositions.is_empty() {
            return Err(AuditError::new(format!(
                "{path}.risk_dispositions must be a non-empty list"
            )));
        }
        if row_dispositions
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
            != row_dispositions.len()
        {
            return Err(AuditError::new(format!(
                "{path}.risk_dispositions contains duplicates"
            )));
        }
        if row_dispositions.contains(&RiskDisposition::DeliberatelyRefused) {
            let refused = entry.refused_alternative.as_deref().ok_or_else(|| {
                AuditError::new(format!("{path}.refused_alternative must be non-empty text"))
            })?;
            validate_text(refused, &format!("{path}.refused_alternative"))?;
        } else if entry.refused_alternative.is_some() {
            return Err(AuditError::new(format!(
                "{path}.refused_alternative is present without deliberately_refused"
            )));
        }
        references.validate(&entry.owner_ref, &format!("{path}.owner_ref"))?;
        let tags = &entry.tags;
        if tags.is_empty() {
            return Err(AuditError::new(format!(
                "{path}.tags must be a non-empty list"
            )));
        }
        let mut normalized = Vec::new();
        for (index, tag) in tags.iter().enumerate() {
            normalized.push(validate_text(tag, &format!("{path}.tags[{index}]"))?);
        }
        if normalized.iter().copied().collect::<HashSet<_>>().len() != normalized.len() {
            return Err(AuditError::new(format!("{path}.tags contains duplicates")));
        }
        seen_tags.extend(normalized.into_iter().map(str::to_owned));
    }

    let required_tags_raw = &contract.required_semantic_tags;
    let mut required_tags = BTreeSet::new();
    for (index, value) in required_tags_raw.iter().enumerate() {
        required_tags
            .insert(validate_text(value, &format!("required_semantic_tags[{index}]"))?.to_owned());
    }
    if required_tags.len() != required_tags_raw.len() {
        return Err(AuditError::new(
            "required_semantic_tags contains duplicates",
        ));
    }
    let mandated_tags: BTreeSet<_> = REQUIRED_TAG_BINDINGS
        .iter()
        .map(|(tag, _)| (*tag).to_owned())
        .collect();
    if required_tags != mandated_tags {
        return Err(AuditError::new(
            "required_semantic_tags must name the mandated audit tags exactly",
        ));
    }
    let missing_tags: Vec<_> = required_tags.difference(&seen_tags).cloned().collect();
    if !missing_tags.is_empty() {
        return Err(AuditError::new(format!(
            "required semantic tags have no premise row: {}",
            missing_tags.join(", ")
        )));
    }
    for (tag, relations) in REQUIRED_TAG_BINDINGS {
        let mut missing = Vec::new();
        for relation in relations {
            let has_tag = premise_entries
                .get(*relation)
                .is_some_and(|entry| entry.tags.iter().any(|value| value == tag));
            if !has_tag {
                missing.push(*relation);
            }
        }
        if !missing.is_empty() {
            return Err(AuditError::new(format!(
                "semantic tag {tag:?} missing from required premise rows: {}",
                missing.join(", ")
            )));
        }
    }

    let expected_rules = validate_text(&contract.rules_sha256, "rules_sha256")?;
    if expected_rules != inventory.rules_sha256 {
        return Err(AuditError::new(format!(
            "authored-rule fingerprint changed: expected {expected_rules}, actual {}; review changed producers, consumers, constants, bindings, and polarity before updating",
            inventory.rules_sha256
        )));
    }
    let expected_facts = validate_text(&contract.facts_sha256, "facts_sha256")?;
    if expected_facts != inventory.facts_sha256 {
        return Err(AuditError::new(format!(
            "authored-fact fingerprint changed: expected {expected_facts}, actual {}; review every current-snapshot cheapest-harm scenario before updating",
            inventory.facts_sha256
        )));
    }
    let expected_routes = &contract.route_fingerprints;
    let expected_route_keys: BTreeSet<_> = expected_routes.keys().cloned().collect();
    let route_keys: BTreeSet<_> = inventory.route_fingerprints.keys().cloned().collect();
    if expected_route_keys != route_keys {
        return Err(AuditError::new(
            "route_fingerprints keys do not match the audited relation surface",
        ));
    }
    let drift: Vec<_> = inventory
        .route_fingerprints
        .iter()
        .filter(|(relation, actual)| expected_routes.get(*relation) != Some(*actual))
        .map(|(relation, _)| relation.clone())
        .collect();
    if !drift.is_empty() {
        return Err(AuditError::new(format!(
            "producer/consumer route changed without reviewed fingerprint update: {}",
            drift.join(", ")
        )));
    }
    Ok(())
}

/// Deserialize each deliberately malformed negative-control candidate through
/// the same strict schema before exercising semantic validation. Live
/// validation calls `validate_contract_typed` directly and never traverses a
/// loose JSON tree.
fn validate_contract(
    value: &Value,
    inventory: &Inventory,
    references: &mut ReferenceResolver<'_>,
) -> AuditResult<()> {
    let contract = contract_from_value(value)?;
    validate_contract_typed(&contract, inventory, references)
}

fn posture_value(relation: &str, inventory: &Inventory) -> ExpectedPosture {
    ExpectedPosture {
        rule_produced: true,
        admitted: inventory.admitted.contains(relation),
        derived_only: inventory.derived_only.contains(relation),
        ground_asserted: inventory.ground_asserted.contains(relation),
    }
}

fn path_text(inventory: &Inventory, source: &str, target: &str) -> String {
    let Some(path) = shortest_path(inventory, source, target) else {
        return "no route".to_owned();
    };
    if path.is_empty() {
        return format!("`{source}` (direct writable relation)");
    }
    let mut rendered = format!("`{source}`");
    for (_, head, negative) in path {
        let _ = write!(rendered, " {}`{head}`", if negative { "─| " } else { "→ " });
    }
    rendered
}

fn readers_text(inventory: &Inventory, relation: &str) -> String {
    let mut readers = Vec::new();
    for (head, dependencies) in &inventory.edges {
        for edge in dependencies {
            if edge.dependency == relation {
                readers.push(format!(
                    "{} `{head}`",
                    if edge.negative {
                        "negative"
                    } else {
                        "positive"
                    }
                ));
            }
        }
    }
    readers.sort();
    if readers.is_empty() {
        "none".to_owned()
    } else {
        readers.join(", ")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn escape_table(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn push_lines<'a>(lines: &mut Vec<String>, values: impl IntoIterator<Item = &'a str>) {
    lines.extend(values.into_iter().map(str::to_owned));
}

fn render(contract: &Contract, inventory: &Inventory) -> AuditResult<String> {
    let derived_entries = &contract.derived_relations;
    let premise_entries = &contract.premises;
    let meanings = &contract.risk_disposition_meanings;
    let mut lines: Vec<String> = Vec::new();
    push_lines(
        &mut lines,
        [
            "<!-- SPDX-License-Identifier: CC-BY-4.0 -->",
            "<!-- Generated by the native rights-verify assertion-surface refresh; edit assertion-surface-contracts.json, not this file. -->",
            "",
            "# Assertion Surface and High-Consequence Premise Audit",
            "",
            "This technical artifact reconciles three independent facts: what the engine",
            "reports as rule-produced, what Article 0a admits as ground vocabulary, and",
            "what Article 0 reserves for rule conclusions. The reviewed contract ledger",
            "supplies authority, provenance, harm, challenge, and risk judgments that the",
            "engine cannot derive.",
            "",
            "Run `./verify.sh --quick` for the structural check and its negative controls.",
            "A changed rule, ground fact,",
            "constant, binding, polarity, declaration, or relation requires an explicit ledger",
            "review before this report can be regenerated.",
            "After that review, `./verify.sh --fingerprints assertion-surface` prints candidate digests; it never updates",
            "the contract ledger.",
            "Regenerate this report with `./verify.sh --refresh assertion-surface`.",
            "",
            "## Measurement contract",
            "",
        ],
    );
    lines.push(format!(
        "- Cheapest harm means: {}",
        contract.cheapest_harm_metric
    ));
    lines.push("- Risk dispositions mean:".to_owned());
    for disposition in DISPOSITIONS {
        lines.push(format!(
            "  - `{disposition}` — {}",
            meanings[&match disposition {
                "deliberately_refused" => RiskDisposition::DeliberatelyRefused,
                "external" => RiskDisposition::External,
                "patchable" => RiskDisposition::Patchable,
                _ => unreachable!(),
            }]
        ));
    }
    push_lines(
        &mut lines,
        [
            "- `→` is a positive dependency; `─|` is a negative dependency, where",
            "  asserting the premise can suppress the downstream conclusion.",
        ],
    );
    lines.push(format!(
        "- Authored-rule fingerprint: `{}`.",
        inventory.rules_sha256
    ));
    lines.push(format!(
        "- Authored-fact fingerprint: `{}`.",
        inventory.facts_sha256
    ));
    push_lines(
        &mut lines,
        [
            "- Rule-head writability remains open for every derived relation;",
            "  `derived_only` blocks ground assertions, not rules.",
            "",
            "## Derived-relation assertion posture",
            "",
            "| relation | stratum | admitted | derived-only | ground facts | classification | contract | direct readers |",
            "| --- | ---: | --- | --- | --- | --- | --- | --- |",
        ],
    );
    for relation in &inventory.derived {
        let entry = &derived_entries[relation];
        lines.push(format!(
            "| `{relation}` | {} | {} | {} | {} | `{}` | `{}` | {} |",
            inventory.strata[relation],
            yes_no(inventory.admitted.contains(relation)),
            yes_no(inventory.derived_only.contains(relation)),
            yes_no(inventory.ground_asserted.contains(relation)),
            entry.classification.as_str(),
            entry.contract_id,
            readers_text(inventory, relation),
        ));
    }

    push_lines(
        &mut lines,
        [
            "",
            "## Writable-premise index",
            "",
            "The effective ground-writable surface is the active `admits` roster minus",
            "any active `derived_only` override, plus explicitly registered exceptional",
            "channels. No exceptional channel exists in the current contract.",
            "",
            "| premise | tags | cheapest target | dispositions | direct readers | route fingerprint |",
            "| --- | --- | --- | --- | --- | --- |",
        ],
    );
    for relation in inventory.writable() {
        let entry = &premise_entries[&relation];
        let cheapest = &entry.cheapest_harm;
        let tags = entry.tags.join(", ");
        let dispositions = entry
            .risk_dispositions
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "| `{relation}` | {} | `{}` | {} | {} | `{}` |",
            escape_table(&tags),
            cheapest.target,
            escape_table(&dispositions),
            readers_text(inventory, &relation),
            &inventory.route_fingerprints[&relation][..16],
        ));
    }

    push_lines(&mut lines, ["", "## Premise contracts", ""]);
    for relation in inventory.writable() {
        let entry = &premise_entries[&relation];
        let cheapest = &entry.cheapest_harm;
        let withheld = &entry.withholding_deletion_harm;
        lines.push(format!("### `{relation}`"));
        lines.push(String::new());
        lines.push(format!("- **Tuple claim:** {}", entry.tuple_claim));
        lines.push(format!("- **Claimed actor:** {}", entry.claimed_actor));
        lines.push(format!(
            "- **Current writer/authority:** {}",
            entry.current_writer_authority
        ));
        lines.push(format!(
            "- **Required writer/authority:** {}",
            entry.required_writer_authority
        ));
        lines.push(format!(
            "- **Current provenance:** {}",
            entry.current_provenance
        ));
        lines.push(format!(
            "- **Required provenance:** {}",
            entry.required_provenance
        ));
        let cheapest_operations = &cheapest.operations;
        lines.push(format!(
            "- **Cheapest harmful {}:** {} operation(s): {}. {} Structural route: {}. Evidence: `{}`.",
            cheapest.operation.as_str(),
            cheapest_operations.len(),
            cheapest_operations.join("; "),
            cheapest.effect,
            path_text(inventory, &relation, &cheapest.target),
            cheapest.evidence_ref,
        ));
        let withheld_operations = &withheld.operations;
        lines.push(format!(
            "- **Withholding/deletion harm:** {} operation(s): {}. {} Structural route: {}. Evidence: `{}`.",
            withheld_operations.len(),
            withheld_operations.join("; "),
            withheld.effect,
            path_text(inventory, &relation, &withheld.target),
            withheld.evidence_ref,
        ));
        lines.push(format!(
            "- **Current challenge route:** {}",
            entry.current_challenge_route
        ));
        lines.push(format!(
            "- **Required challenge route:** {}",
            entry.required_challenge_route
        ));
        let dispositions = entry
            .risk_dispositions
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>();
        lines.push(format!(
            "- **Risk disposition:** {}.",
            dispositions.join(", ")
        ));
        if dispositions.contains(&"deliberately_refused") {
            lines.push(format!(
                "- **Refused alternative:** {}",
                entry
                    .refused_alternative
                    .as_deref()
                    .expect("validated deliberate refusal carries explanation")
            ));
        }
        lines.push(format!("- **Owner:** `{}`.", entry.owner_ref));
        lines.push(format!(
            "- **Reviewed route fingerprint:** `{}`.",
            inventory.route_fingerprints[&relation]
        ));
        lines.push(String::new());
    }
    push_lines(
        &mut lines,
        [
            "## Limits",
            "",
            "This audit proves inventory completeness and makes reviewed assumptions",
            "drift-sensitive. Dependency reach is structural; it does not authenticate a",
            "fact, prove a scenario's real-world truth, or establish that an external",
            "clock or record advances. The operation sets remain a reviewed threat-model",
            "inventory rather than executable proof. The bounded record-integrity red-team",
            "executes only its named flat-snapshot release, adulthood, roster, relief,",
            "and forgiveness subset; the temporal suite separately owns carry, order,",
            "and renewal cases. All other ledger scenarios remain reviewed-only. The",
            "generated assurance case owns the wider control argument and records that",
            "overall deployed-record assurance is not established despite the staged",
            "T1/T2/T3 repository result.",
            "",
        ],
    );
    Ok(lines.join("\n"))
}

fn fingerprint_output(inventory: &Inventory) -> AuditResult<String> {
    let output = FingerprintOutput {
        facts_sha256: &inventory.facts_sha256,
        route_fingerprints: &inventory.route_fingerprints,
        rules_sha256: &inventory.rules_sha256,
        statement_fingerprints: inventory.statement_fingerprints.as_ref(),
    };
    let output = serde_json::to_value(output)
        .map_err(|error| AuditError::new(format!("cannot serialize fingerprints: {error}")))?;
    let mut rendered = String::new();
    pretty_json_python(&output, 0, &mut rendered);
    Ok(rendered)
}

fn python_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{001f}' => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character if !character.is_ascii() => {
                let point = character as u32;
                if point <= 0xffff {
                    let _ = write!(output, "\\u{point:04x}");
                } else {
                    let adjusted = point - 0x1_0000;
                    let high = 0xd800 + (adjusted >> 10);
                    let low = 0xdc00 + (adjusted & 0x3ff);
                    let _ = write!(output, "\\u{high:04x}\\u{low:04x}");
                }
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn pretty_json_python(value: &Value, indent: usize, output: &mut String) {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => python_json_string(value, output),
        Value::Array(values) => {
            if values.is_empty() {
                output.push_str("[]");
                return;
            }
            output.push_str("[\n");
            for (index, value) in values.iter().enumerate() {
                output.push_str(&" ".repeat(indent + 2));
                pretty_json_python(value, indent + 2, output);
                if index + 1 != values.len() {
                    output.push(',');
                }
                output.push('\n');
            }
            output.push_str(&" ".repeat(indent));
            output.push(']');
        }
        Value::Object(values) => {
            if values.is_empty() {
                output.push_str("{}");
                return;
            }
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort();
            output.push_str("{\n");
            let length = keys.len();
            for (index, key) in keys.into_iter().enumerate() {
                output.push_str(&" ".repeat(indent + 2));
                python_json_string(key, output);
                output.push_str(": ");
                pretty_json_python(&values[key], indent + 2, output);
                if index + 1 != length {
                    output.push(',');
                }
                output.push('\n');
            }
            output.push_str(&" ".repeat(indent));
            output.push('}');
        }
    }
}

fn expect_failure<T>(label: &str, result: AuditResult<T>) -> AuditResult<()> {
    if result.is_err() {
        Ok(())
    } else {
        Err(AuditError::new(format!(
            "negative control did not fail: {label}"
        )))
    }
}

fn nested_object_mut<'a>(
    value: &'a mut Value,
    first: &str,
    second: &str,
) -> &'a mut Map<String, Value> {
    value
        .as_object_mut()
        .expect("cloned contract is an object")
        .get_mut(first)
        .and_then(Value::as_object_mut)
        .expect("validated contract collection is an object")
        .get_mut(second)
        .and_then(Value::as_object_mut)
        .expect("validated contract row is an object")
}

fn negative_controls(
    contract: &Value,
    inventory: &Inventory,
    source_text: &str,
    references: &mut ReferenceResolver<'_>,
) -> AuditResult<usize> {
    let mut controls = 0usize;
    let contract_object = contract
        .as_object()
        .expect("validated contract remains an object");
    let derived_entries = contract_object["derived_relations"]
        .as_object()
        .expect("validated derived entries remain an object");
    let premise_entries = contract_object["premises"]
        .as_object()
        .expect("validated premise entries remain an object");
    let sorted_derived: BTreeMap<_, _> = derived_entries.iter().collect();
    let guarded_relation = sorted_derived.iter().find_map(|(relation, entry)| {
        (entry["classification"].as_str() == Some("derived_only")).then_some((*relation).clone())
    });
    let pending_relation = sorted_derived.iter().find_map(|(relation, entry)| {
        (entry["classification"].as_str() == Some("pending_interface"))
            .then_some((*relation).clone())
    });
    let premise_relation = inventory
        .writable()
        .into_iter()
        .next()
        .ok_or_else(|| AuditError::new("negative controls need a writable premise"))?;
    let route_relation = inventory
        .route_fingerprints
        .keys()
        .next()
        .cloned()
        .ok_or_else(|| AuditError::new("negative controls need an audited relation"))?;
    let sorted_premises: BTreeMap<_, _> = premise_entries.iter().collect();
    let refused_relation = sorted_premises.iter().find_map(|(relation, entry)| {
        entry["risk_dispositions"]
            .as_array()
            .is_some_and(|values| {
                values
                    .iter()
                    .any(|value| value.as_str() == Some("deliberately_refused"))
            })
            .then_some((*relation).clone())
    });
    let mut unregistered_head = "audit_probe_result".to_owned();
    while inventory.strata.contains_key(&unregistered_head) {
        unregistered_head.push_str("_next");
    }
    let mut unregistered_ground = "audit_probe_ground".to_owned();
    while inventory.admitted.contains(&unregistered_ground) {
        unregistered_ground.push_str("_next");
    }

    let mut changed = inventory.clone();
    changed.derived.insert(unregistered_head.clone());
    changed.strata.insert(unregistered_head.clone(), 0);
    changed.edges.insert(unregistered_head, Vec::new());
    expect_failure(
        "new unclassified rule head",
        validate_contract(contract, &changed, references),
    )?;
    controls += 1;

    if let Some(relation) = guarded_relation {
        let mut changed = inventory.clone();
        if !changed.admitted.remove(&relation) {
            changed.admitted.insert(relation);
        }
        expect_failure(
            "raw admits drift under derived_only",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;
    }

    if let Some(relation) = pending_relation {
        let mut changed = inventory.clone();
        changed.admitted.insert(relation);
        expect_failure(
            "pending relation admitted",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;
    }

    let reserved_relation = contract_object
        .get("reserved_retired_relations")
        .and_then(Value::as_object)
        .and_then(|entries| entries.keys().min().cloned());
    if let Some(relation) = reserved_relation {
        let mut changed = inventory.clone();
        changed.derived.insert(relation.clone());
        expect_failure(
            "a reserved retirement acquired a producer",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;

        let mut changed = inventory.clone();
        changed.admitted.insert(relation.clone());
        expect_failure(
            "a reserved retirement was re-admitted",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;

        let mut changed = inventory.clone();
        changed.ground_asserted.insert(relation.clone());
        expect_failure(
            "a reserved retirement carries a ground fact",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;

        let mut changed = inventory.clone();
        changed.derived_only.remove(&relation);
        expect_failure(
            "a reserved retirement lost its derived_only declaration",
            validate_contract(contract, &changed, references),
        )?;
        controls += 1;
    }

    let mut changed = inventory.clone();
    changed.admitted.insert(unregistered_ground);
    expect_failure(
        "new ground-only admitted relation",
        validate_contract(contract, &changed, references),
    )?;
    controls += 1;

    let mut changed = inventory.clone();
    changed.admitted.remove(&premise_relation);
    expect_failure(
        "admission removed",
        validate_contract(contract, &changed, references),
    )?;
    controls += 1;

    let mut changed = inventory.clone();
    changed
        .route_fingerprints
        .insert(route_relation, "0".repeat(64));
    expect_failure(
        "producer/consumer route drift",
        validate_contract(contract, &changed, references),
    )?;
    controls += 1;

    let mut changed = inventory.clone();
    changed.facts_sha256 = "0".repeat(64);
    expect_failure(
        "current-snapshot fact drift",
        validate_contract(contract, &changed, references),
    )?;
    controls += 1;

    let mut changed_contract = contract.clone();
    nested_object_mut(&mut changed_contract, "premises", &premise_relation)
        .remove("required_provenance");
    expect_failure(
        "missing semantic field",
        validate_contract(&changed_contract, inventory, references),
    )?;
    controls += 1;

    let mut changed_contract = contract.clone();
    nested_object_mut(&mut changed_contract, "premises", &premise_relation)
        .get_mut("cheapest_harm")
        .and_then(Value::as_object_mut)
        .expect("validated scenario is an object")
        .insert(
            "operation".to_owned(),
            Value::String("withhold_or_delete".to_owned()),
        );
    expect_failure(
        "scenario operation swapped",
        validate_contract(&changed_contract, inventory, references),
    )?;
    controls += 1;

    let mut changed_contract = contract.clone();
    nested_object_mut(&mut changed_contract, "premises", &premise_relation)
        .get_mut("cheapest_harm")
        .and_then(Value::as_object_mut)
        .expect("validated scenario is an object")
        .insert(
            "evidence_ref".to_owned(),
            Value::String("TODO.md::".to_owned()),
        );
    expect_failure(
        "empty reference anchor",
        validate_contract(&changed_contract, inventory, references),
    )?;
    controls += 1;

    let mut changed_contract = contract.clone();
    nested_object_mut(&mut changed_contract, "premises", &premise_relation)
        .get_mut("cheapest_harm")
        .and_then(Value::as_object_mut)
        .expect("validated scenario is an object")
        .insert(
            "operations".to_owned(),
            Value::Array(vec![Value::String("assert `unrelated(Fresh)`".to_owned())]),
        );
    expect_failure(
        "scenario omits audited relation",
        validate_contract(&changed_contract, inventory, references),
    )?;
    controls += 1;

    let mut changed_contract = contract.clone();
    let scenario = nested_object_mut(&mut changed_contract, "premises", &premise_relation)
        .get_mut("cheapest_harm")
        .and_then(Value::as_object_mut)
        .expect("validated scenario is an object");
    let operation = scenario["operations"]
        .as_array()
        .and_then(|values| values.first())
        .cloned()
        .expect("validated operations are non-empty");
    scenario.insert(
        "operations".to_owned(),
        Value::Array(vec![operation.clone(), operation]),
    );
    expect_failure(
        "duplicate scenario operation",
        validate_contract(&changed_contract, inventory, references),
    )?;
    controls += 1;

    if let Some(relation) = refused_relation {
        let mut changed_contract = contract.clone();
        nested_object_mut(&mut changed_contract, "premises", &relation)
            .remove("refused_alternative");
        expect_failure(
            "unexplained deliberate refusal",
            validate_contract(&changed_contract, inventory, references),
        )?;
        controls += 1;
    }

    let baseline: Vec<_> = lex_statements(source_text)?
        .into_iter()
        .map(|statement| statement.text)
        .collect();
    let commented_source = format!(
        "{source_text}\n# admits(\"phantom\"). derived_only(\"phantom\").\n# all $x: person($x) -> phantom($x).\n"
    );
    let commented: Vec<_> = lex_statements(&commented_source)?
        .into_iter()
        .map(|statement| statement.text)
        .collect();
    if commented != baseline {
        return Err(AuditError::new(
            "negative control failed: commented pseudo-statements were active",
        ));
    }
    controls += 1;

    let sample = "admits(\"alpha\"). admits(\"beta\"). fact(One). derived_only(\"late\").";
    let sample_statements = lex_statements(sample)?;
    let names: Vec<_> = sample_statements[..2]
        .iter()
        .map(|statement| {
            declaration_regex()
                .captures(&statement.text)
                .expect("sample declarations match")[2]
                .to_owned()
        })
        .collect();
    if names != ["alpha", "beta"] {
        return Err(AuditError::new(
            "negative control failed: same-line declarations were missed",
        ));
    }
    controls += 1;

    let compound = parse_source(
        "alpha(One) & beta(Two).",
        &BTreeSet::from(["alpha".to_owned(), "beta".to_owned()]),
        &BTreeSet::new(),
        &BTreeMap::new(),
    )?;
    if compound.ground_asserted != BTreeSet::from(["alpha".to_owned(), "beta".to_owned()]) {
        return Err(AuditError::new(
            "negative control failed: compound ground facts were missed",
        ));
    }
    controls += 1;

    expect_failure(
        "late declaration",
        parse_source(
            sample,
            &BTreeSet::from(["fact".to_owned()]),
            &BTreeSet::new(),
            &BTreeMap::new(),
        ),
    )?;
    controls += 1;

    expect_failure(
        "unrecognized active statement",
        parse_source(
            "opaque directive.",
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeMap::new(),
        ),
    )?;
    controls += 1;
    Ok(controls)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_STRATA: &str = concat!(
        "# nibli-strata v1\n",
        "alpha\t0\tbase\t\n",
        "omega\t1\tderived\t+alpha\n",
    );
    const SMALL_SOURCE: &str = concat!(
        "admits(\"alpha\"). derived_only(\"omega\"). ",
        "alpha(One). all $x: alpha($x) -> omega($x).",
    );
    const SMALL_FINGERPRINTS: &str = r#"{
  "facts_sha256": "fd76dbee488973087e96e44655ceb8f8dd9365059697d53219f8a7a6a17d9e0d",
  "route_fingerprints": {
    "alpha": "84a106f92072930e929e7828157ed673acb2d8390879446efba3a0ba9994af6f",
    "omega": "e5b1feb9d1336e2ce29514cab9a44f56cd732bd920497e4387b73b0909df7c44"
  },
  "rules_sha256": "002279814f58417bd6ab5170c6f0bff76c3b92fe60d672cc8acfbe470dc5a800",
  "statement_fingerprints": [
    {
      "id": "c36d87598dc5bbdda237c44ff8532500d23fcfceeb30f4b874f8ad97d4a88d89",
      "kind": "fact",
      "occurrence": 0,
      "statement": "admits(\"alpha\")",
      "statement_sha256": "a904752f330fb728a6e70dcaa5613f9bd6cefe48739b653d4ab22116291de55a"
    },
    {
      "id": "ea4e0f1e5a4597ffd758d1b8753240f403ecec5fb72a8288ebc9da636b1693c6",
      "kind": "fact",
      "occurrence": 0,
      "statement": "derived_only(\"omega\")",
      "statement_sha256": "ccbf3d4f13a5607592fba90753b3ce94be344587e91e4d155b9726f051a41264"
    },
    {
      "id": "1212f8bb3ffdcce3ce020948dcbe75c779d315964c4f6794142ba3d442b58454",
      "kind": "fact",
      "occurrence": 0,
      "statement": "alpha(One)",
      "statement_sha256": "316fa07f88a455add518b7f8ff49c92dd355d176756b878eeb229e2cff4d0602"
    },
    {
      "id": "728448cb9d4ba0f0b8f61994b9c87588a2260b828bf9a5a9d1403a94111921b3",
      "kind": "rule",
      "occurrence": 0,
      "statement": "all $x: alpha($x) -> omega($x)",
      "statement_sha256": "c61f5c6eafafcdf82667ad156be5fdab4bdca82adad37ec7f3a8119af7ce38bd"
    }
  ]
}"#;

    fn empty_alias_contract() -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn live_strata(context: &Context, source: &str) -> String {
        if let Some(path) = std::env::var_os("NIBLI_STRATA_FILE") {
            return std::fs::read_to_string(path).expect("cached live strata");
        }
        let pin_path = std::env::var_os("NIBLI_PIN")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| {
                    PathBuf::from(home).join("projects/dhilipsiva/nibli/target/release/nibli-pin")
                })
            });
        if let Some(pin_path) = pin_path.filter(|path| path.is_file()) {
            let output = Command::new(pin_path)
                .args(["--strata", "--kb", DEFAULT_KB])
                .current_dir(context.root())
                .output()
                .expect("release nibli-pin starts");
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return String::from_utf8(output.stdout).expect("strata are UTF-8");
        }
        let output = pin::dump_strata(&[LoadedSource::new(DEFAULT_KB, source)]);
        assert_eq!(output.exit_code, pin::EXIT_OK, "{}", output.stderr);
        output.stdout
    }

    #[test]
    fn lexer_ignores_comments_and_preserves_quoted_hashes() {
        let statements =
            lex_statements("# admits(\"phantom\").\nadmits(\"alpha\"). fact(\"a#b.c\").\n")
                .expect("fixture lexes");
        assert_eq!(
            statements,
            [
                Statement {
                    text: "admits(\"alpha\")".to_owned(),
                    line: 2,
                },
                Statement {
                    text: "fact(\"a#b.c\")".to_owned(),
                    line: 2,
                },
            ]
        );
        assert!(lex_statements("fact(\"open).").is_err());
        assert!(lex_statements("fact(One)").is_err());
    }

    #[test]
    fn strata_parser_preserves_polarity_and_rejects_unsigned_edges() {
        let (strata, derived, edges) = parse_engine(concat!(
            "event\t0\tbase\t\n",
            "equals\t0\tbase\t\n",
            "alpha\t0\tbase\t\n",
            "omega\t1\tderived\t+alpha,-shadow,+event\n",
            "shadow\t0\tbase\t\n",
        ))
        .expect("fixture parses");
        assert_eq!(strata.len(), 3);
        assert_eq!(derived, BTreeSet::from(["omega".to_owned()]));
        assert_eq!(
            edges["omega"],
            [
                Edge {
                    dependency: "alpha".to_owned(),
                    negative: false,
                },
                Edge {
                    dependency: "shadow".to_owned(),
                    negative: true,
                },
            ]
        );
        assert!(parse_engine("alpha\t0\tbase\tomega\n").is_err());
    }

    #[test]
    fn duplicate_json_keys_are_rejected_at_every_depth() {
        let error = load_contract(r#"{"aliases": {}, "nested": {"x": 1, "x": 2}}"#, "fixture")
            .expect_err("duplicate must fail");
        assert!(error.to_string().contains("duplicate JSON object key: x"));
    }

    #[test]
    fn fingerprint_json_uses_python_ascii_escaping() {
        let mut rendered = String::new();
        pretty_json_python(
            &Value::Object(Map::from_iter([(
                "z".to_owned(),
                Value::String("é😀".to_owned()),
            )])),
            0,
            &mut rendered,
        );
        assert_eq!(rendered, "{\n  \"z\": \"\\u00e9\\ud83d\\ude00\"\n}");
    }

    #[test]
    fn fingerprint_stream_is_byte_identical_to_python_fixture() {
        let inventory = make_inventory(SMALL_STRATA, SMALL_SOURCE, &empty_alias_contract(), true)
            .expect("fixture inventory builds");
        assert_eq!(
            fingerprint_output(&inventory).expect("fingerprints render"),
            SMALL_FINGERPRINTS
        );
    }

    #[test]
    fn source_parser_rejects_late_declarations_and_unknown_directives() {
        assert!(
            parse_source(
                "alpha(One). admits(\"alpha\").",
                &BTreeSet::from(["alpha".to_owned()]),
                &BTreeSet::new(),
                &BTreeMap::new(),
            )
            .expect_err("late declaration fails")
            .to_string()
            .contains("late admits declaration")
        );
        assert!(
            parse_source(
                "opaque directive.",
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeMap::new(),
            )
            .expect_err("directive fails")
            .to_string()
            .contains("unrecognized active statement")
        );
    }

    #[test]
    fn live_inputs_match_reviewed_digests_report_and_all_controls() {
        let context = Context::discover().expect("repository context");
        let source = context.read(DEFAULT_KB).expect("constitution");
        let strata = live_strata(&context, &source);
        let contract_text = context.read(DEFAULT_CONTRACT).expect("contract");
        let (contract, control_value) =
            load_contract(&contract_text, DEFAULT_CONTRACT).expect("contract parses");
        let inventory =
            make_inventory(&strata, &source, &contract.aliases, true).expect("inventory");
        assert_eq!(inventory.derived.len(), 39);
        assert_eq!(inventory.writable().len(), 43);
        assert_eq!(inventory.route_fingerprints.len(), 81);
        assert_eq!(inventory.statement_fingerprints.len(), 1_419);
        let fingerprint_bytes = format!(
            "{}\n",
            fingerprint_output(&inventory).expect("live fingerprints render")
        );
        assert_eq!(
            sha256(fingerprint_bytes),
            "6694b910c38afe2797d9d3189e6d5f98e01a14f127b18a5e26adb15c499ee13b",
            "native --fingerprints bytes differ from Python"
        );

        let mut references = ReferenceResolver::new(&context);
        validate_contract_typed(&contract, &inventory, &mut references)
            .expect("live contract validates");
        let rendered = render(&contract, &inventory).expect("report renders");
        assert!(
            rendered.contains("Generated by the native rights-verify assertion-surface refresh")
        );
        assert!(rendered.contains("`./verify.sh --quick`"));
        assert!(rendered.contains("`./verify.sh --fingerprints assertion-surface`"));
        assert!(rendered.contains("`./verify.sh --refresh assertion-surface`"));
        assert!(!rendered.contains("python3 "));
        assert_eq!(
            rendered,
            context.read(DEFAULT_OUTPUT).expect("reviewed report"),
            "native report bytes differ from the governed Python artifact"
        );
        assert_eq!(
            negative_controls(&control_value, &inventory, &source, &mut references)
                .expect("negative controls pass"),
            22
        );

        let status =
            check(&context, Some(&strata), Some(&source)).expect("cached public API succeeds");
        assert_eq!(
            status,
            "new-book-plans/assertion-surface-audit.md is current; 22 negative controls pass"
        );
    }
}
