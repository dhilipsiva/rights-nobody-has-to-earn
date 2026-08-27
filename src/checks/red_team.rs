// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native bounded flat-snapshot record-integrity red-team.
//!
//! Structural validation and rendering operate on immutable caller-provided
//! inputs when available. Executable snapshots stay in memory and use the
//! native pin runner. Pin files with byte-identical prepared candidates are
//! grouped behind one [`PreparedPinEngine`], so the candidate is parsed once.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;
use crate::pin::{self, LoadedSource, PinOptions, PreparedPinEngine};

const DEFAULT_SOURCE: &str = "new-book-plans/record-integrity-red-team.json";
const DEFAULT_KB: &str = "new-book-plans/constitution.nibli";
const DEFAULT_LEDGER: &str = "new-book-plans/assertion-surface-contracts.json";
const DEFAULT_ASSURANCE: &str = "new-book-plans/record-integrity-assurance-case.json";
const DEFAULT_OUTPUT: &str = "new-book-plans/record-integrity-red-team.md";

const ROOT_KEYS: [&str; 19] = [
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "constitution_sha256",
    "assertion_surface_contracts_sha256",
    "record_integrity_assurance_case_sha256",
    "posture_meanings",
    "required_routes",
    "required_scenarios",
    "limits",
    "routes",
    "snapshots",
    "scenarios",
    "observational_equivalence",
    "temporal_handoff",
    "narrowness_impacts",
    "acceptance_result",
];

const POSTURE_KEYS: [&str; 3] = [
    "current_harm_reproduced",
    "flat_snapshot_boundary_confirmed",
    "negative_control_preserved",
];
const LIMIT_KEYS: [&str; 6] = [
    "flat_snapshot",
    "attribution",
    "temporal_coverage",
    "liveness",
    "scope",
    "no_new_gate",
];
const TEMPORAL_HANDOFF_KEYS: [&str; 4] = [
    "owner_ref",
    "owned_cases",
    "current_contract",
    "residual_boundary",
];
const ROUTE_KEYS: [&str; 15] = [
    "id",
    "title",
    "premises",
    "tested_delta_polarities",
    "assertion_harm",
    "withholding_deletion_harm",
    "claimant_public_power_polarity",
    "current_detectability",
    "safe_default",
    "authorised_disposition_boundary",
    "opposite_failure_test",
    "residual_limit",
    "owner_ref",
    "temporal_status",
    "scenario_refs",
];
const SNAPSHOT_KEYS: [&str; 4] = ["id", "description", "additions", "deletions"];
const SCENARIO_KEYS: [&str; 14] = [
    "id",
    "title",
    "kind",
    "result",
    "attribution",
    "route_refs",
    "state_refs",
    "queries",
    "comparisons",
    "preserved_invariants",
    "interpretation",
    "residual_limit",
    "authorised_disposition_boundary",
    "opposite_failure",
];
const QUERY_KEYS: [&str; 4] = ["state", "expression", "expected", "purpose"];
const SHORT_QUERY_KEYS: [&str; 2] = ["expression", "expected"];
const COMPARISON_KEYS: [&str; 6] = [
    "expression",
    "from_state",
    "from_expected",
    "to_state",
    "to_expected",
    "claim",
];
const INVARIANT_KEYS: [&str; 5] = ["expression", "from_state", "to_state", "expected", "claim"];
const OBSERVATIONAL_KEYS: [&str; 8] = [
    "id",
    "title",
    "route_ref",
    "world_descriptions",
    "snapshot_ref",
    "queries",
    "boundary",
    "prohibited_inference",
];
const NARROWNESS_KEYS: [&str; 5] = [
    "artifact_ref",
    "current_claim",
    "classification",
    "reason",
    "future_trigger",
];
const ACCEPTANCE_KEYS: [&str; 4] = ["result", "claim", "does_not_establish", "remaining_owner"];

const REQUIRED_ROUTE_IDS: [&str; 5] = ["RT-1", "RT-2", "RT-3", "RT-4", "RT-5"];
const REQUIRED_SCENARIO_IDS: [&str; 8] = [
    "RS-01", "RS-02", "RS-03", "RS-04", "RS-05", "RS-07", "RS-08", "RS-16",
];
const REQUIRED_OBSERVATIONAL_IDS: [&str; 4] = ["OE-1", "OE-2", "OE-3", "OE-4"];
const REQUIRED_PREMISES: [&str; 7] = [
    "at", "clear", "forgive", "free", "judge", "person", "rotten",
];
const REQUIRED_NARROWNESS_FILES: [&str; 12] = [
    "book-1/01-what-counts-as-evidence.md",
    "book-1/02-public-answerability.md",
    "book-1/03-who-holds-the-pen.md",
    "book-1/03-who-holds-the-pen.pins.nibli",
    "book-1/05-voiding.md",
    "book-1/06-clawback.md",
    "book-1/07-a-prisoner-is-a-person.md",
    "book-1/09-the-vote-conviction-does-not-take.md",
    "book-1/10-contribution.md",
    "book-1/13-the-one-thing-taken.md",
    "book-1/15-the-five-joints.md",
    "book-1/method.md",
];
const SCENARIO_KINDS: [&str; 5] = [
    "assertion",
    "disappearance",
    "two_entry_matrix",
    "companion_reuse",
    "negative_control",
];
const ATTRIBUTIONS: [&str; 5] = [
    "writer_and_authority_not_attributable_in_flat_snapshot",
    "constructed_source_delta_not_runtime_attribution",
    "writer_independence_not_represented_in_flat_snapshot",
    "purpose_and_case_not_represented_in_flat_snapshot",
    "raw_temporal_input_boundary",
];
const NARROWNESS_CLASSIFICATIONS: [&str; 2] = ["preserved_but_scoped", "revised_and_scoped"];
const EXPECTED: [&str; 2] = ["TRUE", "FALSE"];

const SEMANTIC_SENTINELS: [((&str, &str, &str), &str); 19] = [
    (("RS-01", "base", "prisoner(Adam)"), "TRUE"),
    (("RS-01", "free_adam", "prisoner(Adam)"), "FALSE"),
    (("RS-02", "base", "decide(Cira, Ballot)"), "FALSE"),
    (("RS-02", "adulthood_cira", "decide(Cira, Ballot)"), "TRUE"),
    (
        ("RS-03", "no_adulthood_hano", "decide(Hano, Ballot)"),
        "FALSE",
    ),
    (
        ("RS-04", "no_person_bela", "owe(State, Eats, Bela)"),
        "FALSE",
    ),
    (
        (
            "RS-04",
            "no_person_bela",
            "entitled(Bela, event { eats() })",
        ),
        "FALSE",
    ),
    (("RS-04", "no_person_bela", "false(Bela)"), "TRUE"),
    (
        ("RS-05", "carry_forge_marked", "false(Carry_Forge)"),
        "FALSE",
    ),
    (
        (
            "RS-05",
            "carry_forge_marked",
            "match(Carry_Forge, CarriedVoid)",
        ),
        "FALSE",
    ),
    (("RS-07", "vex_forgive_only", "clean(Vex)"), "FALSE"),
    (("RS-07", "vex_judgment_only", "clean(Vex)"), "FALSE"),
    (("RS-07", "vex_both", "clean(Vex)"), "TRUE"),
    (("RS-07", "vex_both", "permits(Review, Vex)"), "FALSE"),
    (("RS-08", "nia_precleared", "clean(Nia)"), "TRUE"),
    (
        ("RS-08", "nia_forgive_without_judgment", "clean(Nia)"),
        "FALSE",
    ),
    (("RS-16", "base", "permits(Appeals, Nia)"), "TRUE"),
    (
        ("RS-16", "nia_relief_clear_only", "permits(Appeals, Nia)"),
        "FALSE",
    ),
    (
        ("RS-16", "nia_relief_judgment_only", "permits(Appeals, Nia)"),
        "FALSE",
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Check,
    CheckExecute,
    Generate,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputSnapshot<'a> {
    pub(crate) source_json: Option<&'a str>,
    pub(crate) constitution: Option<&'a str>,
    pub(crate) assertion_ledger: Option<&'a str>,
    pub(crate) assurance_source: Option<&'a str>,
    pub(crate) generated_report: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RedTeamSource {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    evidence_role: String,
    constitution_sha256: String,
    assertion_surface_contracts_sha256: String,
    record_integrity_assurance_case_sha256: String,
    posture_meanings: BTreeMap<String, String>,
    required_routes: Vec<String>,
    required_scenarios: Vec<String>,
    limits: BTreeMap<String, String>,
    routes: Vec<RouteContract>,
    snapshots: Vec<Snapshot>,
    scenarios: Vec<Scenario>,
    observational_equivalence: Vec<ObservationalEquivalence>,
    temporal_handoff: TemporalHandoff,
    narrowness_impacts: Vec<NarrownessImpact>,
    acceptance_result: AcceptanceResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TemporalHandoff {
    owner_ref: String,
    owned_cases: Vec<String>,
    current_contract: String,
    residual_boundary: String,
}

/// A strongly typed JSON object whose source ordering is report-significant.
///
/// `BTreeMap` would reorder these authored premise keys during report rendering.
/// Keeping the entries as typed pairs preserves the reviewed JSON order without
/// exposing the production verifier to untyped `serde_json::Value` traversal.
#[derive(Clone, Debug)]
struct TestedDeltaPolarities(Vec<(String, Vec<String>)>);

impl TestedDeltaPolarities {
    fn iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.0.iter().map(|(premise, values)| (premise, values))
    }

    fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.iter().map(|(premise, _)| premise)
    }
}

impl<'a> IntoIterator for &'a TestedDeltaPolarities {
    type Item = (&'a String, &'a Vec<String>);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, Vec<String>)>,
        fn(&(String, Vec<String>)) -> (&String, &Vec<String>),
    >;

    fn into_iter(self) -> Self::IntoIter {
        fn pair_refs(pair: &(String, Vec<String>)) -> (&String, &Vec<String>) {
            (&pair.0, &pair.1)
        }

        self.0.iter().map(pair_refs)
    }
}

impl<'de> Deserialize<'de> for TestedDeltaPolarities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderedMapVisitor;

        impl<'de> Visitor<'de> for OrderedMapVisitor {
            type Value = TestedDeltaPolarities;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an object mapping premise names to delta polarities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                let mut seen = HashSet::new();
                while let Some((premise, polarities)) = map.next_entry::<String, Vec<String>>()? {
                    if !seen.insert(premise.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate tested-delta premise {premise:?}"
                        )));
                    }
                    entries.push((premise, polarities));
                }
                Ok(TestedDeltaPolarities(entries))
            }
        }

        deserializer.deserialize_map(OrderedMapVisitor)
    }
}

impl Serialize for TestedDeltaPolarities {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (premise, polarities) in &self.0 {
            map.serialize_entry(premise, polarities)?;
        }
        map.end()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RouteContract {
    id: String,
    title: String,
    premises: Vec<String>,
    tested_delta_polarities: TestedDeltaPolarities,
    assertion_harm: String,
    withholding_deletion_harm: String,
    claimant_public_power_polarity: String,
    current_detectability: String,
    safe_default: String,
    authorised_disposition_boundary: String,
    opposite_failure_test: String,
    residual_limit: String,
    owner_ref: String,
    temporal_status: String,
    scenario_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    id: String,
    description: String,
    additions: Vec<String>,
    deletions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScenarioQuery {
    state: String,
    expression: String,
    expected: String,
    purpose: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Comparison {
    expression: String,
    from_state: String,
    from_expected: String,
    to_state: String,
    to_expected: String,
    claim: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreservedInvariant {
    expression: String,
    from_state: String,
    to_state: String,
    expected: String,
    claim: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    title: String,
    kind: String,
    result: String,
    attribution: String,
    route_refs: Vec<String>,
    state_refs: Vec<String>,
    queries: Vec<ScenarioQuery>,
    comparisons: Vec<Comparison>,
    preserved_invariants: Vec<PreservedInvariant>,
    interpretation: String,
    residual_limit: String,
    authorised_disposition_boundary: String,
    opposite_failure: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ShortQuery {
    expression: String,
    expected: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ObservationalEquivalence {
    id: String,
    title: String,
    route_ref: String,
    world_descriptions: Vec<String>,
    snapshot_ref: String,
    queries: Vec<ShortQuery>,
    boundary: String,
    prohibited_inference: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NarrownessImpact {
    artifact_ref: String,
    current_claim: String,
    classification: String,
    reason: String,
    future_trigger: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceResult {
    result: String,
    claim: String,
    does_not_establish: Vec<String>,
    remaining_owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RedTeamError(String);

impl RedTeamError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for RedTeamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

type RedResult<T> = Result<T, RedTeamError>;
type QueryMap = BTreeMap<(String, String), String>;
type QueryVectors = BTreeMap<String, QueryMap>;

fn root_keys() -> BTreeSet<&'static str> {
    set(&ROOT_KEYS)
}

fn scenario_keys() -> BTreeSet<&'static str> {
    set(&SCENARIO_KEYS)
}

fn comparison_keys() -> BTreeSet<&'static str> {
    set(&COMPARISON_KEYS)
}

fn invariant_keys() -> BTreeSet<&'static str> {
    set(&INVARIANT_KEYS)
}

fn set(values: &[&'static str]) -> BTreeSet<&'static str> {
    values.iter().copied().collect()
}

fn id_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^[A-Z]{2}-[0-9]{1,2}$").expect("valid id regex"))
}

fn snapshot_id_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_]*$").expect("valid snapshot regex"))
}

fn relation_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_]*$").expect("valid relation regex"))
}

fn digest_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^[0-9a-f]{64}$").expect("valid digest regex"))
}

fn placeholder_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r"(?i)^(?:tbd|todo|unknown|n/?a|pending)$").expect("valid placeholder regex")
    })
}

fn atom_head_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"^([a-z][a-z0-9_]*)\(").expect("valid atom regex"))
}

fn exact_keys(value: &Map<String, Value>, expected: &BTreeSet<&str>, path: &str) -> RedResult<()> {
    let actual: BTreeSet<_> = value.keys().map(String::as_str).collect();
    let missing: Vec<_> = expected.difference(&actual).copied().collect();
    let extra: Vec<_> = actual.difference(expected).copied().collect();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }
    let mut details = Vec::new();
    if !missing.is_empty() {
        details.push(format!("missing {}", missing.join(", ")));
    }
    if !extra.is_empty() {
        details.push(format!("unknown {}", extra.join(", ")));
    }
    Err(RedTeamError::new(format!("{path}: {}", details.join("; "))))
}

fn object<'a>(value: &'a Value, path: &str) -> RedResult<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| RedTeamError::new(format!("{path}: expected an object")))
}

fn array<'a>(value: &'a Value, path: &str) -> RedResult<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| RedTeamError::new(format!("{path}: expected an array")))
}

fn text<'a>(value: &'a Value, path: &str) -> RedResult<&'a str> {
    let value = value
        .as_str()
        .ok_or_else(|| RedTeamError::new(format!("{path}: expected a string")))?;
    let normalized = value.trim();
    if normalized.is_empty() || placeholder_regex().is_match(normalized) {
        return Err(RedTeamError::new(format!(
            "{path}: requires reviewed, non-placeholder text"
        )));
    }
    Ok(value)
}

fn validate_text<'a>(raw: &'a str, path: &str) -> RedResult<&'a str> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}: non-empty text required"
        )));
    }
    if placeholder_regex().is_match(value) {
        return Err(RedTeamError::new(format!(
            "{path}: placeholder value rejected: {raw:?}"
        )));
    }
    Ok(value)
}

fn validate_text_list<'a>(
    values: &'a [String],
    path: &str,
    nonempty: bool,
    unique: bool,
) -> RedResult<Vec<&'a str>> {
    if nonempty && values.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}: non-empty list required"
        )));
    }
    let mut normalized = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        normalized.push(validate_text(value, &format!("{path}[{index}]"))?);
    }
    if unique && normalized.iter().copied().collect::<HashSet<_>>().len() != normalized.len() {
        return Err(RedTeamError::new(format!(
            "{path}: duplicate values rejected"
        )));
    }
    Ok(normalized)
}

fn validate_identifier_text<'a>(value: &'a str, path: &str, prefix: &str) -> RedResult<&'a str> {
    let identifier = validate_text(value, path)?;
    if !id_regex().is_match(identifier) || !identifier.starts_with(prefix) {
        return Err(RedTeamError::new(format!(
            "{path}: invalid stable identifier {identifier:?}"
        )));
    }
    Ok(identifier)
}

fn validate_expected_text<'a>(value: &'a str, path: &str) -> RedResult<&'a str> {
    let expected = validate_text(value, path)?;
    if !EXPECTED.contains(&expected) {
        return Err(RedTeamError::new(format!(
            "{path}: expected TRUE or FALSE, got {expected:?}"
        )));
    }
    Ok(expected)
}

fn text_list<'a>(
    value: &'a Value,
    path: &str,
    nonempty: bool,
    unique: bool,
) -> RedResult<Vec<&'a str>> {
    let values = array(value, path)?;
    if nonempty && values.is_empty() {
        return Err(RedTeamError::new(format!("{path}: must not be empty")));
    }
    let mut result = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        result.push(text(value, &format!("{path}[{index}]"))?);
    }
    if unique && result.iter().copied().collect::<HashSet<_>>().len() != result.len() {
        return Err(RedTeamError::new(format!(
            "{path}: duplicate values are not allowed"
        )));
    }
    Ok(result)
}

fn validate_identifier<'a>(value: &'a Value, path: &str, prefix: &str) -> RedResult<&'a str> {
    let identifier = text(value, path)?;
    if !id_regex().is_match(identifier) || !identifier.starts_with(prefix) {
        return Err(RedTeamError::new(format!(
            "{path}: invalid stable identifier {identifier:?}"
        )));
    }
    Ok(identifier)
}

fn validate_expected<'a>(value: &'a Value, path: &str) -> RedResult<&'a str> {
    let expected = text(value, path)?;
    if !EXPECTED.contains(&expected) {
        return Err(RedTeamError::new(format!(
            "{path}: expected TRUE or FALSE, got {expected:?}"
        )));
    }
    Ok(expected)
}

fn validate_ground_atom<'a>(value: &'a Value, path: &str) -> RedResult<(&'a str, &'a str)> {
    let atom = text(value, path)?;
    validate_ground_atom_text(atom, path)
}

fn validate_ground_atom_text<'a>(atom: &'a str, path: &str) -> RedResult<(&'a str, &'a str)> {
    if atom.contains(['\n', '\r']) {
        return Err(RedTeamError::new(format!(
            "{path}: ground atom must stay on one line"
        )));
    }
    let captures = atom_head_regex()
        .captures(atom)
        .ok_or_else(|| RedTeamError::new(format!("{path}: expected one relation atom")))?;
    if !atom
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_(),{} \t".contains(character))
    {
        return Err(RedTeamError::new(format!(
            "{path}: operator, directive, or statement injection rejected"
        )));
    }
    let mut stack = Vec::new();
    for (index, character) in atom.char_indices() {
        match character {
            '(' | '{' => stack.push(character),
            ')' | '}' => {
                let expected_open = if character == ')' { '(' } else { '{' };
                if stack.pop() != Some(expected_open) {
                    return Err(RedTeamError::new(format!("{path}: unbalanced delimiters")));
                }
                if stack.is_empty() && index + character.len_utf8() != atom.len() {
                    return Err(RedTeamError::new(format!(
                        "{path}: multiple atoms or trailing content rejected"
                    )));
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() || !atom.ends_with(')') {
        return Err(RedTeamError::new(format!(
            "{path}: unbalanced or incomplete ground atom"
        )));
    }
    Ok((captures.get(1).expect("head capture").as_str(), atom))
}

fn validate_expression<'a>(value: &'a Value, path: &str) -> RedResult<&'a str> {
    validate_ground_atom(value, path).map(|(_, atom)| atom)
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVisitor)
    }
}

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
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

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
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

    fn visit_map<A>(self, mut input: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = input.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, input.next_value::<UniqueValue>()?.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

fn load_json(text: &str, label: &str) -> RedResult<(RedTeamSource, Value)> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let value = UniqueValue::deserialize(&mut deserializer)
        .map_err(|error| RedTeamError::new(format!("cannot read {label}: {error}")))?
        .0;
    deserializer
        .end()
        .map_err(|error| RedTeamError::new(format!("cannot read {label}: {error}")))?;
    // Deserialize the typed document from the original bytes so the one
    // report-significant ordered object retains its reviewed source order.
    // The independently parsed `UniqueValue` remains the duplicate-key and
    // negative-control representation only.
    let source = serde_json::from_str(text)
        .map_err(|error| RedTeamError::new(format!("cannot read {label}: {error}")))?;
    Ok((source, value))
}

struct ReferenceResolver<'a> {
    context: &'a Context,
    root: PathBuf,
    files: HashMap<PathBuf, String>,
    validated: HashSet<String>,
}

impl<'a> ReferenceResolver<'a> {
    fn new(context: &'a Context) -> Self {
        Self {
            context,
            root: std::fs::canonicalize(context.root())
                .unwrap_or_else(|_| context.root().to_path_buf()),
            files: HashMap::new(),
            validated: HashSet::new(),
        }
    }

    fn validate(&mut self, value: &Value, path: &str) -> RedResult<String> {
        let reference = text(value, path)?.to_owned();
        self.validate_str(&reference, path)
    }

    fn validate_str(&mut self, value: &str, path: &str) -> RedResult<String> {
        let reference = validate_text(value, path)?.to_owned();
        if reference.matches("::").count() != 1 {
            return Err(RedTeamError::new(format!(
                "{path}: reference must be repo-local path::unique literal needle"
            )));
        }
        let (raw_file, needle) = reference
            .split_once("::")
            .expect("one separator was established");
        if raw_file.is_empty() || needle.is_empty() || raw_file.contains('\\') {
            return Err(RedTeamError::new(format!(
                "{path}: invalid reference path or empty needle"
            )));
        }
        let candidate = Path::new(raw_file);
        if candidate.is_absolute()
            || candidate
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err(RedTeamError::new(format!(
                "{path}: reference must stay inside the repository"
            )));
        }
        if self.validated.contains(&reference) {
            return Ok(reference);
        }
        let unresolved = self.context.path(candidate);
        if !unresolved.is_file() {
            return Err(RedTeamError::new(format!(
                "{path}: referenced file does not exist: {raw_file}"
            )));
        }
        let target = std::fs::canonicalize(&unresolved)
            .map_err(|_| RedTeamError::new(format!("path escapes repository: {raw_file}")))?;
        if !target.starts_with(&self.root) {
            return Err(RedTeamError::new(format!(
                "path escapes repository: {raw_file}"
            )));
        }
        if !self.files.contains_key(&target) {
            let body = std::fs::read_to_string(&target).map_err(|error| {
                RedTeamError::new(format!("{path}: cannot read {raw_file}: {error}"))
            })?;
            self.files.insert(target.clone(), body);
        }
        let count = self.files[&target].matches(needle).count();
        if count != 1 {
            return Err(RedTeamError::new(format!(
                "{path}: needle must occur exactly once in {raw_file}; found {count}"
            )));
        }
        self.validated.insert(reference.clone());
        Ok(reference)
    }
}

struct KbIndex<'a> {
    text: &'a str,
    lines: Vec<&'a str>,
    counts: HashMap<&'a str, usize>,
}

impl<'a> KbIndex<'a> {
    fn new(text: &'a str) -> Self {
        let lines: Vec<_> = text.lines().collect();
        let mut counts = HashMap::with_capacity(lines.len());
        for line in &lines {
            *counts.entry(*line).or_insert(0) += 1;
        }
        Self {
            text,
            lines,
            counts,
        }
    }

    fn count(&self, line: &str) -> usize {
        self.counts.get(line).copied().unwrap_or(0)
    }
}

fn snapshot_delta<'a>(
    snapshot: &'a Map<String, Value>,
    path: &str,
) -> RedResult<(Vec<&'a str>, Vec<&'a str>)> {
    let additions = text_list(
        &snapshot["additions"],
        &format!("{path}.additions"),
        false,
        true,
    )?;
    let deletions = text_list(
        &snapshot["deletions"],
        &format!("{path}.deletions"),
        false,
        true,
    )?;
    let addition_set: HashSet<_> = additions.iter().copied().collect();
    if deletions.iter().any(|value| addition_set.contains(value)) {
        return Err(RedTeamError::new(format!(
            "{path}: the same statement cannot be added and deleted"
        )));
    }
    let identifier = text(&snapshot["id"], &format!("{path}.id"))?;
    if identifier == "base" {
        if !additions.is_empty() || !deletions.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}: base snapshot must have no transformation"
            )));
        }
    } else if additions.is_empty() && deletions.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}: non-base snapshot transformation is a no-op"
        )));
    }
    Ok((additions, deletions))
}

fn validate_snapshot(
    index: &KbIndex<'_>,
    snapshot: &Map<String, Value>,
    path: &str,
) -> RedResult<()> {
    let (additions, deletions) = snapshot_delta(snapshot, path)?;
    for (item_index, statement) in additions.iter().enumerate() {
        let item_path = format!("{path}.additions[{item_index}]");
        let Some(atom) = statement.strip_suffix('.') else {
            return Err(RedTeamError::new(format!(
                "{item_path}: ground fact must end in '.'"
            )));
        };
        validate_ground_atom_text(atom, &item_path)?;
        let count = index.count(statement);
        if count != 0 {
            return Err(RedTeamError::new(format!(
                "{item_path}: addition is not exact and new; found {count} existing line(s)"
            )));
        }
    }
    for (item_index, statement) in deletions.iter().enumerate() {
        let item_path = format!("{path}.deletions[{item_index}]");
        let Some(atom) = statement.strip_suffix('.') else {
            return Err(RedTeamError::new(format!(
                "{item_path}: ground fact must end in '.'"
            )));
        };
        validate_ground_atom_text(atom, &item_path)?;
        let count = index.count(statement);
        if count != 1 {
            return Err(RedTeamError::new(format!(
                "{item_path}: deletion must match exactly once; found {count}"
            )));
        }
    }
    Ok(())
}

fn apply_snapshot(
    index: &KbIndex<'_>,
    snapshot: &Map<String, Value>,
    path: &str,
) -> RedResult<String> {
    validate_snapshot(index, snapshot, path)?;
    let (additions, deletions) = snapshot_delta(snapshot, path)?;
    let remove: HashSet<_> = deletions.into_iter().collect();
    let retained = index.lines.iter().filter(|line| !remove.contains(**line));
    let mut transformed = String::with_capacity(
        index.text.len() + additions.iter().map(|value| value.len() + 1).sum::<usize>() + 80,
    );
    for line in retained {
        transformed.push_str(line);
        transformed.push('\n');
    }
    if !additions.is_empty() {
        transformed.push_str("\n# Red-team snapshot additions (generated, not enacted).\n");
        for addition in additions {
            transformed.push_str(addition);
            transformed.push('\n');
        }
    }
    if text(&snapshot["id"], &format!("{path}.id"))? != "base" && transformed == index.text {
        return Err(RedTeamError::new(format!(
            "{path}: transformation produced byte-identical source"
        )));
    }
    Ok(transformed)
}

fn snapshot_delta_typed<'a>(
    snapshot: &'a Snapshot,
    path: &str,
) -> RedResult<(Vec<&'a str>, Vec<&'a str>)> {
    let additions = validate_text_list(
        &snapshot.additions,
        &format!("{path}.additions"),
        false,
        true,
    )?;
    let deletions = validate_text_list(
        &snapshot.deletions,
        &format!("{path}.deletions"),
        false,
        true,
    )?;
    let addition_set: HashSet<_> = additions.iter().copied().collect();
    if deletions.iter().any(|value| addition_set.contains(value)) {
        return Err(RedTeamError::new(format!(
            "{path}: the same statement cannot be added and deleted"
        )));
    }
    let identifier = validate_text(&snapshot.id, &format!("{path}.id"))?;
    if identifier == "base" {
        if !additions.is_empty() || !deletions.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}: base snapshot must have no transformation"
            )));
        }
    } else if additions.is_empty() && deletions.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}: non-base snapshot transformation is a no-op"
        )));
    }
    Ok((additions, deletions))
}

fn validate_snapshot_typed(index: &KbIndex<'_>, snapshot: &Snapshot, path: &str) -> RedResult<()> {
    let (additions, deletions) = snapshot_delta_typed(snapshot, path)?;
    for (item_index, statement) in additions.iter().enumerate() {
        let item_path = format!("{path}.additions[{item_index}]");
        let Some(atom) = statement.strip_suffix('.') else {
            return Err(RedTeamError::new(format!(
                "{item_path}: ground fact must end in '.'"
            )));
        };
        validate_ground_atom_text(atom, &item_path)?;
        let count = index.count(statement);
        if count != 0 {
            return Err(RedTeamError::new(format!(
                "{item_path}: addition is not exact and new; found {count} existing line(s)"
            )));
        }
    }
    for (item_index, statement) in deletions.iter().enumerate() {
        let item_path = format!("{path}.deletions[{item_index}]");
        let Some(atom) = statement.strip_suffix('.') else {
            return Err(RedTeamError::new(format!(
                "{item_path}: ground fact must end in '.'"
            )));
        };
        validate_ground_atom_text(atom, &item_path)?;
        let count = index.count(statement);
        if count != 1 {
            return Err(RedTeamError::new(format!(
                "{item_path}: deletion must match exactly once; found {count}"
            )));
        }
    }
    Ok(())
}

fn apply_snapshot_typed(index: &KbIndex<'_>, snapshot: &Snapshot, path: &str) -> RedResult<String> {
    validate_snapshot_typed(index, snapshot, path)?;
    let (additions, deletions) = snapshot_delta_typed(snapshot, path)?;
    let remove: HashSet<_> = deletions.into_iter().collect();
    let retained = index.lines.iter().filter(|line| !remove.contains(**line));
    let mut transformed = String::with_capacity(
        index.text.len() + additions.iter().map(|value| value.len() + 1).sum::<usize>() + 80,
    );
    for line in retained {
        transformed.push_str(line);
        transformed.push('\n');
    }
    if !additions.is_empty() {
        transformed.push_str("\n# Red-team snapshot additions (generated, not enacted).\n");
        for addition in additions {
            transformed.push_str(addition);
            transformed.push('\n');
        }
    }
    if snapshot.id != "base" && transformed == index.text {
        return Err(RedTeamError::new(format!(
            "{path}: transformation produced byte-identical source"
        )));
    }
    Ok(transformed)
}

fn scenario_query_map(
    scenario: &Map<String, Value>,
    path: &str,
    snapshot_ids: &BTreeSet<String>,
) -> RedResult<QueryMap> {
    let state_refs: BTreeSet<_> = text_list(
        &scenario["state_refs"],
        &format!("{path}.state_refs"),
        true,
        true,
    )?
    .into_iter()
    .map(str::to_owned)
    .collect();
    let unknown: Vec<_> = state_refs.difference(snapshot_ids).cloned().collect();
    if !unknown.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.state_refs: unknown snapshot(s): {}",
            unknown.join(", ")
        )));
    }
    if state_refs.len() < 2 {
        return Err(RedTeamError::new(format!(
            "{path}.state_refs: every scenario needs at least two states"
        )));
    }
    let queries = array(&scenario["queries"], &format!("{path}.queries"))?;
    let mut result = QueryMap::new();
    for (index, raw_query) in queries.iter().enumerate() {
        let query_path = format!("{path}.queries[{index}]");
        let query = object(raw_query, &query_path)?;
        exact_keys(query, &set(&QUERY_KEYS), &query_path)?;
        let state = text(&query["state"], &format!("{query_path}.state"))?;
        if !state_refs.contains(state) {
            return Err(RedTeamError::new(format!(
                "{query_path}.state: {state:?} is not in state_refs"
            )));
        }
        let expression =
            validate_expression(&query["expression"], &format!("{query_path}.expression"))?;
        let expected = validate_expected(&query["expected"], &format!("{query_path}.expected"))?;
        text(&query["purpose"], &format!("{query_path}.purpose"))?;
        let key = (state.to_owned(), expression.to_owned());
        if let Some(prior) = result.get(&key) {
            if prior != expected {
                return Err(RedTeamError::new(format!(
                    "{query_path}: conflicting expected result for {state}/{expression}"
                )));
            }
            return Err(RedTeamError::new(format!(
                "{query_path}: duplicate query in one scenario"
            )));
        }
        result.insert(key, expected.to_owned());
    }
    if result.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.queries: must not be empty"
        )));
    }
    for state in state_refs {
        if !result.keys().any(|(query_state, _)| query_state == &state) {
            return Err(RedTeamError::new(format!(
                "{path}.queries: state {state:?} has no executable query"
            )));
        }
    }
    Ok(result)
}

fn validate_comparisons(
    scenario: &Map<String, Value>,
    query_map: &QueryMap,
    path: &str,
) -> RedResult<()> {
    let comparisons = array(&scenario["comparisons"], &format!("{path}.comparisons"))?;
    let kind = scenario["kind"]
        .as_str()
        .expect("validated scenario kind is text");
    if kind == "negative_control" {
        if !comparisons.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.comparisons: negative control must not claim a flip"
            )));
        }
    } else if comparisons.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.comparisons: non-vacuous scenario needs a flip"
        )));
    }
    for (index, raw) in comparisons.iter().enumerate() {
        let item_path = format!("{path}.comparisons[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, &comparison_keys(), &item_path)?;
        let expression =
            validate_expression(&item["expression"], &format!("{item_path}.expression"))?;
        let from_state = text(&item["from_state"], &format!("{item_path}.from_state"))?;
        let to_state = text(&item["to_state"], &format!("{item_path}.to_state"))?;
        let from_expected = validate_expected(
            &item["from_expected"],
            &format!("{item_path}.from_expected"),
        )?;
        let to_expected =
            validate_expected(&item["to_expected"], &format!("{item_path}.to_expected"))?;
        text(&item["claim"], &format!("{item_path}.claim"))?;
        if from_state == to_state || from_expected == to_expected {
            return Err(RedTeamError::new(format!(
                "{item_path}: comparison must discriminate two states"
            )));
        }
        if query_map
            .get(&(from_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(from_expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: from result does not match a declared query"
            )));
        }
        if query_map
            .get(&(to_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(to_expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: to result does not match a declared query"
            )));
        }
    }

    let invariants = array(
        &scenario["preserved_invariants"],
        &format!("{path}.preserved_invariants"),
    )?;
    if invariants.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.preserved_invariants: positive control required"
        )));
    }
    for (index, raw) in invariants.iter().enumerate() {
        let item_path = format!("{path}.preserved_invariants[{index}]");
        let item = object(raw, &item_path)?;
        exact_keys(item, &invariant_keys(), &item_path)?;
        let expression =
            validate_expression(&item["expression"], &format!("{item_path}.expression"))?;
        let from_state = text(&item["from_state"], &format!("{item_path}.from_state"))?;
        let to_state = text(&item["to_state"], &format!("{item_path}.to_state"))?;
        let expected = validate_expected(&item["expected"], &format!("{item_path}.expected"))?;
        text(&item["claim"], &format!("{item_path}.claim"))?;
        if from_state == to_state {
            return Err(RedTeamError::new(format!(
                "{item_path}: invariant must span two states"
            )));
        }
        if query_map
            .get(&(from_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: from invariant lacks matching query"
            )));
        }
        if query_map
            .get(&(to_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: to invariant lacks matching query"
            )));
        }
    }
    Ok(())
}

fn scenario_query_map_typed(
    scenario: &Scenario,
    path: &str,
    snapshot_ids: &BTreeSet<String>,
) -> RedResult<QueryMap> {
    let state_refs: BTreeSet<_> = validate_text_list(
        &scenario.state_refs,
        &format!("{path}.state_refs"),
        true,
        true,
    )?
    .into_iter()
    .map(str::to_owned)
    .collect();
    let unknown: Vec<_> = state_refs.difference(snapshot_ids).cloned().collect();
    if !unknown.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.state_refs: unknown snapshot(s): {}",
            unknown.join(", ")
        )));
    }
    if state_refs.len() < 2 {
        return Err(RedTeamError::new(format!(
            "{path}.state_refs: every scenario needs at least two states"
        )));
    }
    let mut result = QueryMap::new();
    for (index, query) in scenario.queries.iter().enumerate() {
        let query_path = format!("{path}.queries[{index}]");
        let state = validate_text(&query.state, &format!("{query_path}.state"))?;
        if !state_refs.contains(state) {
            return Err(RedTeamError::new(format!(
                "{query_path}.state: {state:?} is not in state_refs"
            )));
        }
        let expression = validate_ground_atom_text(
            validate_text(&query.expression, &format!("{query_path}.expression"))?,
            &format!("{query_path}.expression"),
        )?
        .1;
        let expected = validate_expected_text(&query.expected, &format!("{query_path}.expected"))?;
        validate_text(&query.purpose, &format!("{query_path}.purpose"))?;
        let key = (state.to_owned(), expression.to_owned());
        if let Some(prior) = result.get(&key) {
            if prior != expected {
                return Err(RedTeamError::new(format!(
                    "{query_path}: conflicting expected result for {state}/{expression}"
                )));
            }
            return Err(RedTeamError::new(format!(
                "{query_path}: duplicate query in one scenario"
            )));
        }
        result.insert(key, expected.to_owned());
    }
    if result.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.queries: must not be empty"
        )));
    }
    for state in state_refs {
        if !result.keys().any(|(query_state, _)| query_state == &state) {
            return Err(RedTeamError::new(format!(
                "{path}.queries: state {state:?} has no executable query"
            )));
        }
    }
    Ok(result)
}

fn validate_comparisons_typed(
    scenario: &Scenario,
    query_map: &QueryMap,
    path: &str,
) -> RedResult<()> {
    if scenario.kind == "negative_control" {
        if !scenario.comparisons.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.comparisons: negative control must not claim a flip"
            )));
        }
    } else if scenario.comparisons.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.comparisons: non-vacuous scenario needs a flip"
        )));
    }
    for (index, item) in scenario.comparisons.iter().enumerate() {
        let item_path = format!("{path}.comparisons[{index}]");
        let expression = validate_ground_atom_text(
            validate_text(&item.expression, &format!("{item_path}.expression"))?,
            &format!("{item_path}.expression"),
        )?
        .1;
        let from_state = validate_text(&item.from_state, &format!("{item_path}.from_state"))?;
        let to_state = validate_text(&item.to_state, &format!("{item_path}.to_state"))?;
        let from_expected =
            validate_expected_text(&item.from_expected, &format!("{item_path}.from_expected"))?;
        let to_expected =
            validate_expected_text(&item.to_expected, &format!("{item_path}.to_expected"))?;
        validate_text(&item.claim, &format!("{item_path}.claim"))?;
        if from_state == to_state || from_expected == to_expected {
            return Err(RedTeamError::new(format!(
                "{item_path}: comparison must discriminate two states"
            )));
        }
        if query_map
            .get(&(from_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(from_expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: from result does not match a declared query"
            )));
        }
        if query_map
            .get(&(to_state.to_owned(), expression.to_owned()))
            .map(String::as_str)
            != Some(to_expected)
        {
            return Err(RedTeamError::new(format!(
                "{item_path}: to result does not match a declared query"
            )));
        }
    }
    if scenario.preserved_invariants.is_empty() {
        return Err(RedTeamError::new(format!(
            "{path}.preserved_invariants: positive control required"
        )));
    }
    for (index, item) in scenario.preserved_invariants.iter().enumerate() {
        let item_path = format!("{path}.preserved_invariants[{index}]");
        let expression = validate_ground_atom_text(
            validate_text(&item.expression, &format!("{item_path}.expression"))?,
            &format!("{item_path}.expression"),
        )?
        .1;
        let from_state = validate_text(&item.from_state, &format!("{item_path}.from_state"))?;
        let to_state = validate_text(&item.to_state, &format!("{item_path}.to_state"))?;
        let expected = validate_expected_text(&item.expected, &format!("{item_path}.expected"))?;
        validate_text(&item.claim, &format!("{item_path}.claim"))?;
        if from_state == to_state {
            return Err(RedTeamError::new(format!(
                "{item_path}: invariant must span two states"
            )));
        }
        for (state, side) in [(from_state, "from"), (to_state, "to")] {
            if query_map
                .get(&(state.to_owned(), expression.to_owned()))
                .map(String::as_str)
                != Some(expected)
            {
                return Err(RedTeamError::new(format!(
                    "{item_path}: {side} invariant lacks matching query"
                )));
            }
        }
    }
    Ok(())
}

fn validate_source(
    source: &Value,
    kb: &KbIndex<'_>,
    kb_digest: &str,
    ledger_digest: &str,
    assurance_digest: &str,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<QueryVectors> {
    let source = object(source, "root")?;
    exact_keys(source, &root_keys(), "root")?;
    if source["spdx"].as_str() != Some("CC-BY-4.0") {
        return Err(RedTeamError::new("spdx: reviewed source must be CC-BY-4.0"));
    }
    if source["schema_version"].as_i64() != Some(2) {
        return Err(RedTeamError::new(
            "schema_version: only version 2 is supported",
        ));
    }
    text(&source["title"], "title")?;
    if source["status"].as_str() != Some("bounded_flat_snapshot_red_team_not_assurance") {
        return Err(RedTeamError::new(
            "status: this artifact must remain bounded flat-snapshot red-team evidence",
        ));
    }
    if source["evidence_role"].as_str() != Some("exposes_gap_and_tests_boundary") {
        return Err(RedTeamError::new(
            "evidence_role: mixed gap/boundary evidence may not be promoted to assurance",
        ));
    }
    for (key, actual) in [
        ("constitution_sha256", kb_digest),
        ("assertion_surface_contracts_sha256", ledger_digest),
        ("record_integrity_assurance_case_sha256", assurance_digest),
    ] {
        let declared = text(&source[key], key)?;
        if !digest_regex().is_match(declared) {
            return Err(RedTeamError::new(format!(
                "{key}: expected a lowercase SHA-256 digest"
            )));
        }
        if declared != actual {
            return Err(RedTeamError::new(format!(
                "{key}: stale; declared {declared}, actual {actual}"
            )));
        }
    }

    let postures = object(&source["posture_meanings"], "posture_meanings")?;
    exact_keys(postures, &set(&POSTURE_KEYS), "posture_meanings")?;
    for (key, value) in postures {
        text(value, &format!("posture_meanings.{key}"))?;
    }
    let limits = object(&source["limits"], "limits")?;
    exact_keys(limits, &set(&LIMIT_KEYS), "limits")?;
    for (key, value) in limits {
        text(value, &format!("limits.{key}"))?;
    }
    let handoff = object(&source["temporal_handoff"], "temporal_handoff")?;
    exact_keys(handoff, &set(&TEMPORAL_HANDOFF_KEYS), "temporal_handoff")?;
    references.validate(&handoff["owner_ref"], "temporal_handoff.owner_ref")?;
    let owned_cases: BTreeSet<_> = text_list(
        &handoff["owned_cases"],
        "temporal_handoff.owned_cases",
        true,
        true,
    )?
    .into_iter()
    .collect();
    if owned_cases != set(&["TA-02", "TA-03", "TA-04", "TA-08", "TA-25"]) {
        return Err(RedTeamError::new(
            "temporal_handoff.owned_cases: must name the exact delegated carry/status cases",
        ));
    }
    for field in ["current_contract", "residual_boundary"] {
        text(&handoff[field], &format!("temporal_handoff.{field}"))?;
    }

    let declared_routes: BTreeSet<_> =
        text_list(&source["required_routes"], "required_routes", true, true)?
            .into_iter()
            .collect();
    let declared_scenarios: BTreeSet<_> = text_list(
        &source["required_scenarios"],
        "required_scenarios",
        true,
        true,
    )?
    .into_iter()
    .collect();
    if declared_routes != set(&REQUIRED_ROUTE_IDS) {
        return Err(RedTeamError::new(format!(
            "required_routes: must name exactly {}",
            REQUIRED_ROUTE_IDS.join(", ")
        )));
    }
    if declared_scenarios != set(&REQUIRED_SCENARIO_IDS) {
        return Err(RedTeamError::new(format!(
            "required_scenarios: must name exactly {}",
            REQUIRED_SCENARIO_IDS.join(", ")
        )));
    }

    let mut routes: BTreeMap<String, &Map<String, Value>> = BTreeMap::new();
    let mut premise_coverage = BTreeSet::new();
    for (index, raw_route) in array(&source["routes"], "routes")?.iter().enumerate() {
        let path = format!("routes[{index}]");
        let route = object(raw_route, &path)?;
        exact_keys(route, &set(&ROUTE_KEYS), &path)?;
        let route_id = validate_identifier(&route["id"], &format!("{path}.id"), "RT-")?;
        if routes.contains_key(route_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {route_id}"
            )));
        }
        text(&route["title"], &format!("{path}.title"))?;
        let premises = text_list(&route["premises"], &format!("{path}.premises"), true, true)?;
        for premise in &premises {
            if !relation_regex().is_match(premise) {
                return Err(RedTeamError::new(format!(
                    "{path}.premises: invalid relation {premise:?}"
                )));
            }
            premise_coverage.insert((*premise).to_owned());
        }
        let tested = object(
            &route["tested_delta_polarities"],
            &format!("{path}.tested_delta_polarities"),
        )?;
        let tested_keys: BTreeSet<_> = tested.keys().map(String::as_str).collect();
        let premise_set: BTreeSet<_> = premises.iter().copied().collect();
        if tested_keys != premise_set {
            return Err(RedTeamError::new(format!(
                "{path}.tested_delta_polarities: must name every and only route premise"
            )));
        }
        for (premise, values) in tested {
            let polarities: BTreeSet<_> = text_list(
                values,
                &format!("{path}.tested_delta_polarities.{premise}"),
                true,
                true,
            )?
            .into_iter()
            .collect();
            if !polarities.is_subset(&set(&["addition", "deletion"])) {
                return Err(RedTeamError::new(format!(
                    "{path}.tested_delta_polarities.{premise}: unknown delta polarity"
                )));
            }
        }
        for field in [
            "assertion_harm",
            "withholding_deletion_harm",
            "claimant_public_power_polarity",
            "current_detectability",
            "safe_default",
            "authorised_disposition_boundary",
            "opposite_failure_test",
            "residual_limit",
            "temporal_status",
        ] {
            text(&route[field], &format!("{path}.{field}"))?;
        }
        references.validate(&route["owner_ref"], &format!("{path}.owner_ref"))?;
        let scenario_refs: BTreeSet<_> = text_list(
            &route["scenario_refs"],
            &format!("{path}.scenario_refs"),
            true,
            true,
        )?
        .into_iter()
        .collect();
        let unknown: Vec<_> = scenario_refs
            .difference(&set(&REQUIRED_SCENARIO_IDS))
            .copied()
            .collect();
        if !unknown.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.scenario_refs: unknown scenario(s): {}",
                unknown.join(", ")
            )));
        }
        routes.insert(route_id.to_owned(), route);
    }
    if routes.keys().map(String::as_str).collect::<BTreeSet<_>>() != set(&REQUIRED_ROUTE_IDS) {
        return Err(RedTeamError::new(
            "routes: missing or unexpected required route IDs",
        ));
    }
    let required_premises: BTreeSet<_> = REQUIRED_PREMISES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    if premise_coverage != required_premises {
        let missing: Vec<_> = required_premises
            .difference(&premise_coverage)
            .cloned()
            .collect();
        let extra: Vec<_> = premise_coverage
            .difference(&required_premises)
            .cloned()
            .collect();
        return Err(RedTeamError::new(format!(
            "routes: premise coverage mismatch; missing {}; extra {}",
            if missing.is_empty() {
                "none".to_owned()
            } else {
                missing.join(", ")
            },
            if extra.is_empty() {
                "none".to_owned()
            } else {
                extra.join(", ")
            },
        )));
    }

    let mut snapshots: BTreeMap<String, &Map<String, Value>> = BTreeMap::new();
    for (index, raw_snapshot) in array(&source["snapshots"], "snapshots")?.iter().enumerate() {
        let path = format!("snapshots[{index}]");
        let snapshot = object(raw_snapshot, &path)?;
        exact_keys(snapshot, &set(&SNAPSHOT_KEYS), &path)?;
        let snapshot_id = text(&snapshot["id"], &format!("{path}.id"))?;
        if !snapshot_id_regex().is_match(snapshot_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: invalid snapshot identifier"
            )));
        }
        if snapshots.contains_key(snapshot_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {snapshot_id}"
            )));
        }
        text(&snapshot["description"], &format!("{path}.description"))?;
        validate_snapshot(kb, snapshot, &path)?;
        snapshots.insert(snapshot_id.to_owned(), snapshot);
    }
    if !snapshots.contains_key("base") {
        return Err(RedTeamError::new("snapshots: base snapshot is required"));
    }

    let snapshot_ids: BTreeSet<_> = snapshots.keys().cloned().collect();
    let mut scenarios: BTreeMap<String, &Map<String, Value>> = BTreeMap::new();
    let mut query_vectors = QueryVectors::new();
    let mut scenario_to_routes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (index, raw_scenario) in array(&source["scenarios"], "scenarios")?.iter().enumerate() {
        let path = format!("scenarios[{index}]");
        let scenario = object(raw_scenario, &path)?;
        exact_keys(scenario, &scenario_keys(), &path)?;
        let scenario_id = validate_identifier(&scenario["id"], &format!("{path}.id"), "RS-")?;
        if scenarios.contains_key(scenario_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {scenario_id}"
            )));
        }
        text(&scenario["title"], &format!("{path}.title"))?;
        let kind = text(&scenario["kind"], &format!("{path}.kind"))?;
        if !SCENARIO_KINDS.contains(&kind) {
            return Err(RedTeamError::new(format!(
                "{path}.kind: unknown kind {kind:?}"
            )));
        }
        let result = text(&scenario["result"], &format!("{path}.result"))?;
        if !POSTURE_KEYS.contains(&result) {
            return Err(RedTeamError::new(format!(
                "{path}.result: unknown posture {result:?}"
            )));
        }
        if kind == "negative_control" && result != "negative_control_preserved" {
            return Err(RedTeamError::new(format!(
                "{path}.result: negative control must preserve its control"
            )));
        }
        if kind != "negative_control" && result == "negative_control_preserved" {
            return Err(RedTeamError::new(format!(
                "{path}.result: only negative control may use this posture"
            )));
        }
        let attribution = text(&scenario["attribution"], &format!("{path}.attribution"))?;
        if !ATTRIBUTIONS.contains(&attribution) {
            return Err(RedTeamError::new(format!(
                "{path}.attribution: attribution overclaim or unknown value"
            )));
        }
        if kind == "disappearance"
            && attribution != "constructed_source_delta_not_runtime_attribution"
        {
            return Err(RedTeamError::new(format!(
                "{path}.attribution: disappearance may not be attributed as live deletion or withholding"
            )));
        }
        let route_refs: BTreeSet<_> = text_list(
            &scenario["route_refs"],
            &format!("{path}.route_refs"),
            true,
            true,
        )?
        .into_iter()
        .map(str::to_owned)
        .collect();
        let route_ids: BTreeSet<_> = routes.keys().cloned().collect();
        let unknown: Vec<_> = route_refs.difference(&route_ids).cloned().collect();
        if !unknown.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.route_refs: unknown route(s): {}",
                unknown.join(", ")
            )));
        }
        let query_map = scenario_query_map(scenario, &path, &snapshot_ids)?;
        validate_comparisons(scenario, &query_map, &path)?;
        for field in [
            "interpretation",
            "residual_limit",
            "authorised_disposition_boundary",
            "opposite_failure",
        ] {
            text(&scenario[field], &format!("{path}.{field}"))?;
        }
        scenarios.insert(scenario_id.to_owned(), scenario);
        query_vectors.insert(scenario_id.to_owned(), query_map);
        scenario_to_routes.insert(scenario_id.to_owned(), route_refs);
    }
    if scenarios
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&REQUIRED_SCENARIO_IDS)
    {
        return Err(RedTeamError::new(
            "scenarios: missing or unexpected required scenario IDs",
        ));
    }

    for (route_id, route) in &routes {
        let declared: BTreeSet<_> = array(&route["scenario_refs"], "scenario_refs")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .expect("validated scenario reference")
                    .to_owned()
            })
            .collect();
        let actual: BTreeSet<_> = scenario_to_routes
            .iter()
            .filter(|(_, route_refs)| route_refs.contains(route_id))
            .map(|(scenario_id, _)| scenario_id.clone())
            .collect();
        if declared != actual {
            return Err(RedTeamError::new(format!(
                "routes[{route_id}].scenario_refs: does not reconcile with scenario route_refs"
            )));
        }
        let premises: BTreeSet<_> = text_list(&route["premises"], "premises", true, true)?
            .into_iter()
            .map(str::to_owned)
            .collect();
        let mut actual_polarities: BTreeMap<String, BTreeSet<String>> = premises
            .iter()
            .map(|premise| (premise.clone(), BTreeSet::new()))
            .collect();
        let route_states: BTreeSet<_> = actual
            .iter()
            .flat_map(|scenario_id| {
                array(&scenarios[scenario_id]["state_refs"], "state_refs")
                    .expect("validated state refs")
                    .iter()
                    .map(|value| value.as_str().expect("validated state").to_owned())
            })
            .collect();
        for state in route_states {
            let snapshot = snapshots[&state];
            for (field, polarity) in [("additions", "addition"), ("deletions", "deletion")] {
                for fact in array(&snapshot[field], field)? {
                    let fact = fact.as_str().expect("validated snapshot fact");
                    let relation = validate_ground_atom_text(
                        fact.strip_suffix('.').unwrap_or(fact),
                        &format!("snapshot {state}.{field}"),
                    )?
                    .0;
                    if let Some(values) = actual_polarities.get_mut(relation) {
                        values.insert(polarity.to_owned());
                    }
                }
            }
        }
        let declared_polarities: BTreeMap<_, _> =
            object(&route["tested_delta_polarities"], "tested_delta_polarities")?
                .iter()
                .map(|(premise, values)| {
                    (
                        premise.clone(),
                        values
                            .as_array()
                            .expect("validated polarities")
                            .iter()
                            .map(|value| value.as_str().expect("validated polarity").to_owned())
                            .collect(),
                    )
                })
                .collect();
        if declared_polarities != actual_polarities {
            return Err(RedTeamError::new(format!(
                "routes[{route_id}].tested_delta_polarities: does not reconcile with referenced snapshot deltas"
            )));
        }
    }

    for ((scenario_id, state, expression), expected) in SEMANTIC_SENTINELS {
        let actual = query_vectors
            .get(scenario_id)
            .and_then(|queries| queries.get(&(state.to_owned(), expression.to_owned())))
            .map(String::as_str);
        if actual != Some(expected) {
            return Err(RedTeamError::new(format!(
                "{scenario_id}: semantic sentinel {state}/{expression} must be {expected}, got {}",
                actual.unwrap_or("None")
            )));
        }
    }
    let states = |scenario_id: &str| -> BTreeSet<&str> {
        scenarios[scenario_id]["state_refs"]
            .as_array()
            .expect("validated state refs")
            .iter()
            .map(|value| value.as_str().expect("validated state"))
            .collect()
    };
    if states("RS-07") != set(&["base", "vex_forgive_only", "vex_judgment_only", "vex_both"]) {
        return Err(RedTeamError::new(
            "RS-07: correction matrix must contain neither, first-only, second-only, and both",
        ));
    }
    if states("RS-08") != set(&["base", "nia_precleared", "nia_forgive_without_judgment"]) {
        return Err(RedTeamError::new(
            "RS-08: pre-clear and generic-companion removal controls are mandatory",
        ));
    }
    if states("RS-16")
        != set(&[
            "base",
            "nia_relief_neither",
            "nia_relief_clear_only",
            "nia_relief_judgment_only",
        ])
    {
        return Err(RedTeamError::new(
            "RS-16: relief matrix must contain neither, clear-only, judgment-only, and both",
        ));
    }

    validate_observational(source, &routes, &snapshots)?;
    validate_narrowness(source, references)?;
    let acceptance = object(&source["acceptance_result"], "acceptance_result")?;
    exact_keys(acceptance, &set(&ACCEPTANCE_KEYS), "acceptance_result")?;
    if acceptance["result"].as_str() != Some("current_harm_reproduced") {
        return Err(RedTeamError::new(
            "acceptance_result.result: may not claim assurance",
        ));
    }
    text(&acceptance["claim"], "acceptance_result.claim")?;
    let residuals = text_list(
        &acceptance["does_not_establish"],
        "acceptance_result.does_not_establish",
        true,
        true,
    )?;
    let residual_text = residuals.join(" ").to_lowercase();
    for required in [
        "authorship",
        "deletion",
        "liveness",
        "recovery",
        "general",
        "deployment",
    ] {
        if !residual_text.contains(required) {
            return Err(RedTeamError::new(format!(
                "acceptance_result.does_not_establish: must retain {required:?} boundary"
            )));
        }
    }
    references.validate(
        &acceptance["remaining_owner"],
        "acceptance_result.remaining_owner",
    )?;
    Ok(query_vectors)
}

fn validate_observational(
    source: &Map<String, Value>,
    routes: &BTreeMap<String, &Map<String, Value>>,
    snapshots: &BTreeMap<String, &Map<String, Value>>,
) -> RedResult<()> {
    let mut observational = BTreeSet::new();
    for (index, raw_entry) in array(
        &source["observational_equivalence"],
        "observational_equivalence",
    )?
    .iter()
    .enumerate()
    {
        let path = format!("observational_equivalence[{index}]");
        let entry = object(raw_entry, &path)?;
        exact_keys(entry, &set(&OBSERVATIONAL_KEYS), &path)?;
        let entry_id = validate_identifier(&entry["id"], &format!("{path}.id"), "OE-")?;
        if !observational.insert(entry_id.to_owned()) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {entry_id}"
            )));
        }
        text(&entry["title"], &format!("{path}.title"))?;
        let route_ref = text(&entry["route_ref"], &format!("{path}.route_ref"))?;
        if !routes.contains_key(route_ref) {
            return Err(RedTeamError::new(format!(
                "{path}.route_ref: unknown route {route_ref}"
            )));
        }
        let worlds = text_list(
            &entry["world_descriptions"],
            &format!("{path}.world_descriptions"),
            true,
            true,
        )?;
        if worlds.len() < 2 {
            return Err(RedTeamError::new(format!(
                "{path}.world_descriptions: at least two worlds required"
            )));
        }
        let snapshot_ref = text(&entry["snapshot_ref"], &format!("{path}.snapshot_ref"))?;
        if !snapshots.contains_key(snapshot_ref) {
            return Err(RedTeamError::new(format!(
                "{path}.snapshot_ref: unknown snapshot {snapshot_ref}"
            )));
        }
        let queries = array(&entry["queries"], &format!("{path}.queries"))?;
        if queries.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.queries: identical executable vector required"
            )));
        }
        for (query_index, raw_query) in queries.iter().enumerate() {
            let query_path = format!("{path}.queries[{query_index}]");
            let query = object(raw_query, &query_path)?;
            exact_keys(query, &set(&SHORT_QUERY_KEYS), &query_path)?;
            validate_expression(&query["expression"], &format!("{query_path}.expression"))?;
            validate_expected(&query["expected"], &format!("{query_path}.expected"))?;
        }
        text(&entry["boundary"], &format!("{path}.boundary"))?;
        text(
            &entry["prohibited_inference"],
            &format!("{path}.prohibited_inference"),
        )?;
    }
    if observational
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&REQUIRED_OBSERVATIONAL_IDS)
    {
        return Err(RedTeamError::new(
            "observational_equivalence: required indistinguishability cases missing",
        ));
    }
    Ok(())
}

fn validate_narrowness(
    source: &Map<String, Value>,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<()> {
    let mut files = BTreeSet::new();
    let mut seen = HashSet::new();
    for (index, raw_entry) in array(&source["narrowness_impacts"], "narrowness_impacts")?
        .iter()
        .enumerate()
    {
        let path = format!("narrowness_impacts[{index}]");
        let entry = object(raw_entry, &path)?;
        exact_keys(entry, &set(&NARROWNESS_KEYS), &path)?;
        let reference =
            references.validate(&entry["artifact_ref"], &format!("{path}.artifact_ref"))?;
        if !seen.insert(reference.clone()) {
            return Err(RedTeamError::new(format!(
                "{path}.artifact_ref: duplicate reference"
            )));
        }
        files.insert(
            reference
                .split_once("::")
                .expect("validated reference has separator")
                .0
                .to_owned(),
        );
        if !NARROWNESS_CLASSIFICATIONS
            .contains(&entry["classification"].as_str().unwrap_or_default())
        {
            return Err(RedTeamError::new(format!(
                "{path}.classification: unknown narrowness disposition"
            )));
        }
        for field in ["current_claim", "reason", "future_trigger"] {
            text(&entry[field], &format!("{path}.{field}"))?;
        }
    }
    let required: BTreeSet<_> = REQUIRED_NARROWNESS_FILES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    if files != required {
        let missing: Vec<_> = required.difference(&files).cloned().collect();
        let extra: Vec<_> = files.difference(&required).cloned().collect();
        return Err(RedTeamError::new(format!(
            "narrowness_impacts: file coverage mismatch; missing {}; extra {}",
            if missing.is_empty() {
                "none".to_owned()
            } else {
                missing.join(", ")
            },
            if extra.is_empty() {
                "none".to_owned()
            } else {
                extra.join(", ")
            },
        )));
    }
    Ok(())
}

fn validate_observational_typed(
    source: &RedTeamSource,
    routes: &BTreeMap<String, &RouteContract>,
    snapshots: &BTreeMap<String, &Snapshot>,
) -> RedResult<()> {
    let mut observational = BTreeSet::new();
    for (index, entry) in source.observational_equivalence.iter().enumerate() {
        let path = format!("observational_equivalence[{index}]");
        let entry_id = validate_identifier_text(&entry.id, &format!("{path}.id"), "OE-")?;
        if !observational.insert(entry_id.to_owned()) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {entry_id}"
            )));
        }
        validate_text(&entry.title, &format!("{path}.title"))?;
        let route_ref = validate_text(&entry.route_ref, &format!("{path}.route_ref"))?;
        if !routes.contains_key(route_ref) {
            return Err(RedTeamError::new(format!(
                "{path}.route_ref: unknown route {route_ref}"
            )));
        }
        let worlds = validate_text_list(
            &entry.world_descriptions,
            &format!("{path}.world_descriptions"),
            true,
            true,
        )?;
        if worlds.len() < 2 {
            return Err(RedTeamError::new(format!(
                "{path}.world_descriptions: at least two worlds required"
            )));
        }
        let snapshot_ref = validate_text(&entry.snapshot_ref, &format!("{path}.snapshot_ref"))?;
        if !snapshots.contains_key(snapshot_ref) {
            return Err(RedTeamError::new(format!(
                "{path}.snapshot_ref: unknown snapshot {snapshot_ref}"
            )));
        }
        if entry.queries.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.queries: identical executable vector required"
            )));
        }
        for (query_index, query) in entry.queries.iter().enumerate() {
            let query_path = format!("{path}.queries[{query_index}]");
            validate_ground_atom_text(
                validate_text(&query.expression, &format!("{query_path}.expression"))?,
                &format!("{query_path}.expression"),
            )?;
            validate_expected_text(&query.expected, &format!("{query_path}.expected"))?;
        }
        validate_text(&entry.boundary, &format!("{path}.boundary"))?;
        validate_text(
            &entry.prohibited_inference,
            &format!("{path}.prohibited_inference"),
        )?;
    }
    if observational
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&REQUIRED_OBSERVATIONAL_IDS)
    {
        return Err(RedTeamError::new(
            "observational_equivalence: required indistinguishability cases missing",
        ));
    }
    Ok(())
}

fn validate_narrowness_typed(
    source: &RedTeamSource,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<()> {
    let mut files = BTreeSet::new();
    let mut seen = HashSet::new();
    for (index, entry) in source.narrowness_impacts.iter().enumerate() {
        let path = format!("narrowness_impacts[{index}]");
        let reference =
            references.validate_str(&entry.artifact_ref, &format!("{path}.artifact_ref"))?;
        if !seen.insert(reference.clone()) {
            return Err(RedTeamError::new(format!(
                "{path}.artifact_ref: duplicate reference"
            )));
        }
        files.insert(
            reference
                .split_once("::")
                .expect("validated reference has separator")
                .0
                .to_owned(),
        );
        if !NARROWNESS_CLASSIFICATIONS.contains(&entry.classification.as_str()) {
            return Err(RedTeamError::new(format!(
                "{path}.classification: unknown narrowness disposition"
            )));
        }
        validate_text(&entry.current_claim, &format!("{path}.current_claim"))?;
        validate_text(&entry.reason, &format!("{path}.reason"))?;
        validate_text(&entry.future_trigger, &format!("{path}.future_trigger"))?;
    }
    let required: BTreeSet<_> = REQUIRED_NARROWNESS_FILES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    if files != required {
        let missing: Vec<_> = required.difference(&files).cloned().collect();
        let extra: Vec<_> = files.difference(&required).cloned().collect();
        return Err(RedTeamError::new(format!(
            "narrowness_impacts: file coverage mismatch; missing {}; extra {}",
            if missing.is_empty() {
                "none".to_owned()
            } else {
                missing.join(", ")
            },
            if extra.is_empty() {
                "none".to_owned()
            } else {
                extra.join(", ")
            },
        )));
    }
    Ok(())
}

fn validate_source_typed(
    source: &RedTeamSource,
    kb: &KbIndex<'_>,
    kb_digest: &str,
    ledger_digest: &str,
    assurance_digest: &str,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<QueryVectors> {
    if source.spdx != "CC-BY-4.0" {
        return Err(RedTeamError::new("spdx: reviewed source must be CC-BY-4.0"));
    }
    if source.schema_version != 2 {
        return Err(RedTeamError::new(
            "schema_version: only version 2 is supported",
        ));
    }
    validate_text(&source.title, "title")?;
    if source.status != "bounded_flat_snapshot_red_team_not_assurance" {
        return Err(RedTeamError::new(
            "status: this artifact must remain bounded flat-snapshot red-team evidence",
        ));
    }
    if source.evidence_role != "exposes_gap_and_tests_boundary" {
        return Err(RedTeamError::new(
            "evidence_role: mixed gap/boundary evidence may not be promoted to assurance",
        ));
    }
    for (key, declared, actual) in [
        (
            "constitution_sha256",
            &source.constitution_sha256,
            kb_digest,
        ),
        (
            "assertion_surface_contracts_sha256",
            &source.assertion_surface_contracts_sha256,
            ledger_digest,
        ),
        (
            "record_integrity_assurance_case_sha256",
            &source.record_integrity_assurance_case_sha256,
            assurance_digest,
        ),
    ] {
        validate_text(declared, key)?;
        if !digest_regex().is_match(declared) {
            return Err(RedTeamError::new(format!(
                "{key}: expected a lowercase SHA-256 digest"
            )));
        }
        if declared != actual {
            return Err(RedTeamError::new(format!(
                "{key}: stale; declared {declared}, actual {actual}"
            )));
        }
    }
    if source
        .posture_meanings
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&POSTURE_KEYS)
    {
        return Err(RedTeamError::new(
            "posture_meanings: exact reviewed keys required",
        ));
    }
    for (key, value) in &source.posture_meanings {
        validate_text(value, &format!("posture_meanings.{key}"))?;
    }
    if source
        .limits
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&LIMIT_KEYS)
    {
        return Err(RedTeamError::new("limits: exact reviewed keys required"));
    }
    for (key, value) in &source.limits {
        validate_text(value, &format!("limits.{key}"))?;
    }
    references.validate_str(
        &source.temporal_handoff.owner_ref,
        "temporal_handoff.owner_ref",
    )?;
    let owned_cases: BTreeSet<_> = validate_text_list(
        &source.temporal_handoff.owned_cases,
        "temporal_handoff.owned_cases",
        true,
        true,
    )?
    .into_iter()
    .collect();
    if owned_cases != set(&["TA-02", "TA-03", "TA-04", "TA-08", "TA-25"]) {
        return Err(RedTeamError::new(
            "temporal_handoff.owned_cases: must name the exact delegated carry/status cases",
        ));
    }
    validate_text(
        &source.temporal_handoff.current_contract,
        "temporal_handoff.current_contract",
    )?;
    validate_text(
        &source.temporal_handoff.residual_boundary,
        "temporal_handoff.residual_boundary",
    )?;

    let declared_routes: BTreeSet<_> =
        validate_text_list(&source.required_routes, "required_routes", true, true)?
            .into_iter()
            .collect();
    let declared_scenarios: BTreeSet<_> =
        validate_text_list(&source.required_scenarios, "required_scenarios", true, true)?
            .into_iter()
            .collect();
    if declared_routes != set(&REQUIRED_ROUTE_IDS) {
        return Err(RedTeamError::new(format!(
            "required_routes: must name exactly {}",
            REQUIRED_ROUTE_IDS.join(", ")
        )));
    }
    if declared_scenarios != set(&REQUIRED_SCENARIO_IDS) {
        return Err(RedTeamError::new(format!(
            "required_scenarios: must name exactly {}",
            REQUIRED_SCENARIO_IDS.join(", ")
        )));
    }

    let mut routes: BTreeMap<String, &RouteContract> = BTreeMap::new();
    let mut premise_coverage = BTreeSet::new();
    for (index, route) in source.routes.iter().enumerate() {
        let path = format!("routes[{index}]");
        let route_id = validate_identifier_text(&route.id, &format!("{path}.id"), "RT-")?;
        if routes.contains_key(route_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {route_id}"
            )));
        }
        validate_text(&route.title, &format!("{path}.title"))?;
        let premises =
            validate_text_list(&route.premises, &format!("{path}.premises"), true, true)?;
        for premise in &premises {
            if !relation_regex().is_match(premise) {
                return Err(RedTeamError::new(format!(
                    "{path}.premises: invalid relation {premise:?}"
                )));
            }
            premise_coverage.insert((*premise).to_owned());
        }
        let tested_keys: BTreeSet<_> = route
            .tested_delta_polarities
            .keys()
            .map(String::as_str)
            .collect();
        let premise_set: BTreeSet<_> = premises.iter().copied().collect();
        if tested_keys != premise_set {
            return Err(RedTeamError::new(format!(
                "{path}.tested_delta_polarities: must name every and only route premise"
            )));
        }
        for (premise, values) in &route.tested_delta_polarities {
            let polarities: BTreeSet<_> = validate_text_list(
                values,
                &format!("{path}.tested_delta_polarities.{premise}"),
                true,
                true,
            )?
            .into_iter()
            .collect();
            if !polarities.is_subset(&set(&["addition", "deletion"])) {
                return Err(RedTeamError::new(format!(
                    "{path}.tested_delta_polarities.{premise}: unknown delta polarity"
                )));
            }
        }
        for (field, value) in [
            ("assertion_harm", &route.assertion_harm),
            (
                "withholding_deletion_harm",
                &route.withholding_deletion_harm,
            ),
            (
                "claimant_public_power_polarity",
                &route.claimant_public_power_polarity,
            ),
            ("current_detectability", &route.current_detectability),
            ("safe_default", &route.safe_default),
            (
                "authorised_disposition_boundary",
                &route.authorised_disposition_boundary,
            ),
            ("opposite_failure_test", &route.opposite_failure_test),
            ("residual_limit", &route.residual_limit),
            ("temporal_status", &route.temporal_status),
        ] {
            validate_text(value, &format!("{path}.{field}"))?;
        }
        references.validate_str(&route.owner_ref, &format!("{path}.owner_ref"))?;
        let scenario_refs: BTreeSet<_> = validate_text_list(
            &route.scenario_refs,
            &format!("{path}.scenario_refs"),
            true,
            true,
        )?
        .into_iter()
        .collect();
        let unknown: Vec<_> = scenario_refs
            .difference(&set(&REQUIRED_SCENARIO_IDS))
            .copied()
            .collect();
        if !unknown.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.scenario_refs: unknown scenario(s): {}",
                unknown.join(", ")
            )));
        }
        routes.insert(route_id.to_owned(), route);
    }
    if routes.keys().map(String::as_str).collect::<BTreeSet<_>>() != set(&REQUIRED_ROUTE_IDS) {
        return Err(RedTeamError::new(
            "routes: missing or unexpected required route IDs",
        ));
    }
    let required_premises: BTreeSet<_> = REQUIRED_PREMISES
        .iter()
        .map(|value| (*value).to_owned())
        .collect();
    if premise_coverage != required_premises {
        let missing: Vec<_> = required_premises
            .difference(&premise_coverage)
            .cloned()
            .collect();
        let extra: Vec<_> = premise_coverage
            .difference(&required_premises)
            .cloned()
            .collect();
        return Err(RedTeamError::new(format!(
            "routes: premise coverage mismatch; missing {}; extra {}",
            if missing.is_empty() {
                "none".to_owned()
            } else {
                missing.join(", ")
            },
            if extra.is_empty() {
                "none".to_owned()
            } else {
                extra.join(", ")
            },
        )));
    }

    let mut snapshots: BTreeMap<String, &Snapshot> = BTreeMap::new();
    for (index, snapshot) in source.snapshots.iter().enumerate() {
        let path = format!("snapshots[{index}]");
        let snapshot_id = validate_text(&snapshot.id, &format!("{path}.id"))?;
        if !snapshot_id_regex().is_match(snapshot_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: invalid snapshot identifier"
            )));
        }
        if snapshots.contains_key(snapshot_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {snapshot_id}"
            )));
        }
        validate_text(&snapshot.description, &format!("{path}.description"))?;
        validate_snapshot_typed(kb, snapshot, &path)?;
        snapshots.insert(snapshot_id.to_owned(), snapshot);
    }
    if !snapshots.contains_key("base") {
        return Err(RedTeamError::new("snapshots: base snapshot is required"));
    }

    let snapshot_ids: BTreeSet<_> = snapshots.keys().cloned().collect();
    let mut scenarios: BTreeMap<String, &Scenario> = BTreeMap::new();
    let mut query_vectors = QueryVectors::new();
    let mut scenario_to_routes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (index, scenario) in source.scenarios.iter().enumerate() {
        let path = format!("scenarios[{index}]");
        let scenario_id = validate_identifier_text(&scenario.id, &format!("{path}.id"), "RS-")?;
        if scenarios.contains_key(scenario_id) {
            return Err(RedTeamError::new(format!(
                "{path}.id: duplicate {scenario_id}"
            )));
        }
        validate_text(&scenario.title, &format!("{path}.title"))?;
        let kind = validate_text(&scenario.kind, &format!("{path}.kind"))?;
        if !SCENARIO_KINDS.contains(&kind) {
            return Err(RedTeamError::new(format!(
                "{path}.kind: unknown kind {kind:?}"
            )));
        }
        let result = validate_text(&scenario.result, &format!("{path}.result"))?;
        if !POSTURE_KEYS.contains(&result) {
            return Err(RedTeamError::new(format!(
                "{path}.result: unknown posture {result:?}"
            )));
        }
        if kind == "negative_control" && result != "negative_control_preserved" {
            return Err(RedTeamError::new(format!(
                "{path}.result: negative control must preserve its control"
            )));
        }
        if kind != "negative_control" && result == "negative_control_preserved" {
            return Err(RedTeamError::new(format!(
                "{path}.result: only negative control may use this posture"
            )));
        }
        let attribution = validate_text(&scenario.attribution, &format!("{path}.attribution"))?;
        if !ATTRIBUTIONS.contains(&attribution) {
            return Err(RedTeamError::new(format!(
                "{path}.attribution: attribution overclaim or unknown value"
            )));
        }
        if kind == "disappearance"
            && attribution != "constructed_source_delta_not_runtime_attribution"
        {
            return Err(RedTeamError::new(format!(
                "{path}.attribution: disappearance may not be attributed as live deletion or withholding"
            )));
        }
        let route_refs: BTreeSet<_> = validate_text_list(
            &scenario.route_refs,
            &format!("{path}.route_refs"),
            true,
            true,
        )?
        .into_iter()
        .map(str::to_owned)
        .collect();
        let unknown: Vec<_> = route_refs
            .difference(&routes.keys().cloned().collect())
            .cloned()
            .collect();
        if !unknown.is_empty() {
            return Err(RedTeamError::new(format!(
                "{path}.route_refs: unknown route(s): {}",
                unknown.join(", ")
            )));
        }
        let query_map = scenario_query_map_typed(scenario, &path, &snapshot_ids)?;
        validate_comparisons_typed(scenario, &query_map, &path)?;
        for (field, value) in [
            ("interpretation", &scenario.interpretation),
            ("residual_limit", &scenario.residual_limit),
            (
                "authorised_disposition_boundary",
                &scenario.authorised_disposition_boundary,
            ),
            ("opposite_failure", &scenario.opposite_failure),
        ] {
            validate_text(value, &format!("{path}.{field}"))?;
        }
        scenarios.insert(scenario_id.to_owned(), scenario);
        query_vectors.insert(scenario_id.to_owned(), query_map);
        scenario_to_routes.insert(scenario_id.to_owned(), route_refs);
    }
    if scenarios
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != set(&REQUIRED_SCENARIO_IDS)
    {
        return Err(RedTeamError::new(
            "scenarios: missing or unexpected required scenario IDs",
        ));
    }

    for (route_id, route) in &routes {
        let declared: BTreeSet<_> = route.scenario_refs.iter().cloned().collect();
        let actual: BTreeSet<_> = scenario_to_routes
            .iter()
            .filter(|(_, route_refs)| route_refs.contains(route_id))
            .map(|(scenario_id, _)| scenario_id.clone())
            .collect();
        if declared != actual {
            return Err(RedTeamError::new(format!(
                "routes[{route_id}].scenario_refs: does not reconcile with scenario route_refs"
            )));
        }
        let premises: BTreeSet<_> = route.premises.iter().cloned().collect();
        let mut actual_polarities: BTreeMap<String, BTreeSet<String>> = premises
            .iter()
            .map(|premise| (premise.clone(), BTreeSet::new()))
            .collect();
        let route_states: BTreeSet<_> = actual
            .iter()
            .flat_map(|scenario_id| scenarios[scenario_id].state_refs.iter().cloned())
            .collect();
        for state in route_states {
            let snapshot = snapshots[&state];
            for (facts, polarity) in [
                (&snapshot.additions, "addition"),
                (&snapshot.deletions, "deletion"),
            ] {
                for fact in facts {
                    let relation = validate_ground_atom_text(
                        fact.strip_suffix('.').unwrap_or(fact),
                        &format!("snapshot {state}"),
                    )?
                    .0;
                    if let Some(values) = actual_polarities.get_mut(relation) {
                        values.insert(polarity.to_owned());
                    }
                }
            }
        }
        let declared_polarities: BTreeMap<_, _> = route
            .tested_delta_polarities
            .iter()
            .map(|(premise, values)| (premise.clone(), values.iter().cloned().collect()))
            .collect();
        if declared_polarities != actual_polarities {
            return Err(RedTeamError::new(format!(
                "routes[{route_id}].tested_delta_polarities: does not reconcile with referenced snapshot deltas"
            )));
        }
    }

    for ((scenario_id, state, expression), expected) in SEMANTIC_SENTINELS {
        let actual = query_vectors
            .get(scenario_id)
            .and_then(|queries| queries.get(&(state.to_owned(), expression.to_owned())))
            .map(String::as_str);
        if actual != Some(expected) {
            return Err(RedTeamError::new(format!(
                "{scenario_id}: semantic sentinel {state}/{expression} must be {expected}, got {}",
                actual.unwrap_or("None")
            )));
        }
    }
    let states = |scenario_id: &str| -> BTreeSet<&str> {
        scenarios[scenario_id]
            .state_refs
            .iter()
            .map(String::as_str)
            .collect()
    };
    if states("RS-07") != set(&["base", "vex_forgive_only", "vex_judgment_only", "vex_both"]) {
        return Err(RedTeamError::new(
            "RS-07: correction matrix must contain neither, first-only, second-only, and both",
        ));
    }
    if states("RS-08") != set(&["base", "nia_precleared", "nia_forgive_without_judgment"]) {
        return Err(RedTeamError::new(
            "RS-08: pre-clear and generic-companion removal controls are mandatory",
        ));
    }
    if states("RS-16")
        != set(&[
            "base",
            "nia_relief_neither",
            "nia_relief_clear_only",
            "nia_relief_judgment_only",
        ])
    {
        return Err(RedTeamError::new(
            "RS-16: relief matrix must contain neither, clear-only, judgment-only, and both",
        ));
    }

    validate_observational_typed(source, &routes, &snapshots)?;
    validate_narrowness_typed(source, references)?;
    if source.acceptance_result.result != "current_harm_reproduced" {
        return Err(RedTeamError::new(
            "acceptance_result.result: may not claim assurance",
        ));
    }
    validate_text(&source.acceptance_result.claim, "acceptance_result.claim")?;
    let residuals = validate_text_list(
        &source.acceptance_result.does_not_establish,
        "acceptance_result.does_not_establish",
        true,
        true,
    )?;
    let residual_text = residuals.join(" ").to_lowercase();
    for required in [
        "authorship",
        "deletion",
        "liveness",
        "recovery",
        "general",
        "deployment",
    ] {
        if !residual_text.contains(required) {
            return Err(RedTeamError::new(format!(
                "acceptance_result.does_not_establish: must retain {required:?} boundary"
            )));
        }
    }
    references.validate_str(
        &source.acceptance_result.remaining_owner,
        "acceptance_result.remaining_owner",
    )?;
    Ok(query_vectors)
}

fn collect_queries(
    source: &Map<String, Value>,
) -> RedResult<BTreeMap<String, BTreeMap<String, String>>> {
    let mut result: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut add = |state: &str, expression: &str, expected: &str, path: &str| -> RedResult<()> {
        let state_queries = result.entry(state.to_owned()).or_default();
        if let Some(prior) = state_queries.get(expression)
            && prior != expected
        {
            return Err(RedTeamError::new(format!(
                "{path}: global query conflict for {state}/{expression}: {prior} vs {expected}"
            )));
        }
        state_queries.insert(expression.to_owned(), expected.to_owned());
        Ok(())
    };
    for (scenario_index, raw_scenario) in
        array(&source["scenarios"], "scenarios")?.iter().enumerate()
    {
        let scenario = object(raw_scenario, &format!("scenarios[{scenario_index}]"))?;
        for (query_index, raw_query) in array(&scenario["queries"], "queries")?.iter().enumerate() {
            let query = object(raw_query, "query")?;
            add(
                query["state"].as_str().expect("validated query state"),
                query["expression"].as_str().expect("validated expression"),
                query["expected"]
                    .as_str()
                    .expect("validated expected result"),
                &format!("scenarios[{scenario_index}].queries[{query_index}]"),
            )?;
        }
    }
    for (entry_index, raw_entry) in array(
        &source["observational_equivalence"],
        "observational_equivalence",
    )?
    .iter()
    .enumerate()
    {
        let entry = object(
            raw_entry,
            &format!("observational_equivalence[{entry_index}]"),
        )?;
        let state = entry["snapshot_ref"]
            .as_str()
            .expect("validated snapshot reference");
        for (query_index, raw_query) in array(&entry["queries"], "queries")?.iter().enumerate() {
            let query = object(raw_query, "query")?;
            add(
                state,
                query["expression"].as_str().expect("validated expression"),
                query["expected"]
                    .as_str()
                    .expect("validated expected result"),
                &format!("observational_equivalence[{entry_index}].queries[{query_index}]"),
            )?;
        }
    }
    Ok(result)
}

fn collect_queries_typed(
    source: &RedTeamSource,
) -> RedResult<BTreeMap<String, BTreeMap<String, String>>> {
    let mut result: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut add = |state: &str, expression: &str, expected: &str, path: &str| -> RedResult<()> {
        let state_queries = result.entry(state.to_owned()).or_default();
        if let Some(prior) = state_queries.get(expression)
            && prior != expected
        {
            return Err(RedTeamError::new(format!(
                "{path}: global query conflict for {state}/{expression}: {prior} vs {expected}"
            )));
        }
        state_queries.insert(expression.to_owned(), expected.to_owned());
        Ok(())
    };
    for (scenario_index, scenario) in source.scenarios.iter().enumerate() {
        for (query_index, query) in scenario.queries.iter().enumerate() {
            add(
                &query.state,
                &query.expression,
                &query.expected,
                &format!("scenarios[{scenario_index}].queries[{query_index}]"),
            )?;
        }
    }
    for (entry_index, entry) in source.observational_equivalence.iter().enumerate() {
        for (query_index, query) in entry.queries.iter().enumerate() {
            add(
                &entry.snapshot_ref,
                &query.expression,
                &query.expected,
                &format!("observational_equivalence[{entry_index}].queries[{query_index}]"),
            )?;
        }
    }
    Ok(result)
}

#[derive(Clone, Debug)]
struct OwnedPin {
    name: String,
    source: String,
    expected_pins: usize,
    expect_finding: bool,
}

#[derive(Clone, Debug)]
struct CandidateGroup {
    digest: String,
    deletions: Vec<String>,
    additions: Vec<String>,
    pins: Vec<OwnedPin>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExecutionResult {
    snapshots: usize,
    pins: usize,
    controls: usize,
}

fn query_pin_source(state: &str, queries: &BTreeMap<String, String>) -> String {
    let mut lines = vec![
        format!(":expect-pins {}", queries.len()),
        format!("# Generated red-team queries for snapshot {state}."),
        "# This file is ephemeral and outside chapter pin reconciliation.".to_owned(),
        String::new(),
    ];
    for (expression, expected) in queries {
        lines.extend([
            format!("# Reviewed expected consequence in {state}."),
            format!("? {expression}."),
            format!("# => {expected}"),
            String::new(),
        ]);
    }
    lines.join("\n")
}

fn sabotage_pin_source() -> String {
    concat!(
        ":expect-pins 1\n",
        "# Executable negative control: Adam is a prisoner in the base.\n\n",
        "? prisoner(Adam).\n",
        "# => FALSE\n",
    )
    .to_owned()
}

fn add_candidate_group(
    groups: &mut Vec<CandidateGroup>,
    mut deletions: Vec<String>,
    additions: Vec<String>,
    pin: OwnedPin,
) {
    // Deletion order does not affect the transformed candidate; addition order
    // does. This key therefore groups exactly the snapshots that produce the
    // same line-oriented KB without retaining a full multi-megabyte copy.
    deletions.sort();
    if let Some(group) = groups
        .iter_mut()
        .find(|group| group.deletions == deletions && group.additions == additions)
    {
        group.pins.push(pin);
        return;
    }
    let key = serde_json::to_vec(&(&deletions, &additions))
        .expect("candidate delta key is JSON serializable");
    groups.push(CandidateGroup {
        digest: sha256(key),
        deletions,
        additions,
        pins: vec![pin],
    });
}

fn output_tail(output: &pin::RunOutput) -> String {
    let combined = format!("{}{}", output.stdout, output.stderr);
    let lines: Vec<_> = combined.lines().collect();
    lines[lines.len().saturating_sub(12)..].join("\n")
}

fn run_candidate_group(prepared: &PreparedPinEngine, group: CandidateGroup) -> RedResult<()> {
    let loaded: Vec<_> = group
        .pins
        .iter()
        .map(|pin| LoadedSource::new(&pin.name, &pin.source))
        .collect();
    let deletions: Vec<_> = group.deletions.iter().map(String::as_str).collect();
    let additions: Vec<_> = group.additions.iter().map(String::as_str).collect();
    let output = prepared.run_patched_files(&deletions, &additions, &loaded, PinOptions::default());
    if output.files.len() != group.pins.len() {
        return Err(RedTeamError::new(format!(
            "candidate {}: native pin runner returned {} file reports for {} pin files\n{}",
            &group.digest[..16],
            output.files.len(),
            group.pins.len(),
            output_tail(&output),
        )));
    }
    for (expected, actual) in group.pins.iter().zip(&output.files) {
        if actual.pins != expected.expected_pins {
            return Err(RedTeamError::new(format!(
                "{}: ran {} pins, expected {}",
                expected.name, actual.pins, expected.expected_pins
            )));
        }
        if actual.harness != 0 || actual.resolved != 0 || actual.defects != 0 {
            return Err(RedTeamError::new(format!(
                "{}: native pin harness did not complete cleanly\n{}",
                expected.name,
                output_tail(&output)
            )));
        }
        let wanted_findings = usize::from(expected.expect_finding);
        if actual.findings != wanted_findings {
            let message = if expected.expect_finding {
                "executable inverted-sentinel control did not fail as a finding"
            } else {
                "reviewed snapshot query produced a finding"
            };
            return Err(RedTeamError::new(format!(
                "{}: {message}\n{}",
                expected.name,
                output_tail(&output)
            )));
        }
    }
    Ok(())
}

fn execute_scenarios(source: &Map<String, Value>, kb: &KbIndex<'_>) -> RedResult<ExecutionResult> {
    let snapshots: BTreeMap<_, _> = array(&source["snapshots"], "snapshots")?
        .iter()
        .filter_map(|value| value.as_object())
        .map(|snapshot| {
            (
                snapshot["id"]
                    .as_str()
                    .expect("validated snapshot id")
                    .to_owned(),
                snapshot,
            )
        })
        .collect();
    let queries = collect_queries(source)?;
    let pins_run = queries.values().map(BTreeMap::len).sum();
    let mut groups = Vec::new();
    for (state, state_queries) in &queries {
        let snapshot = snapshots
            .get(state)
            .ok_or_else(|| RedTeamError::new(format!("missing executable snapshot {state}")))?;
        validate_snapshot(kb, snapshot, &format!("snapshot {state}"))?;
        let (additions, deletions) = snapshot_delta(snapshot, &format!("snapshot {state}"))?;
        add_candidate_group(
            &mut groups,
            deletions.into_iter().map(str::to_owned).collect(),
            additions.into_iter().map(str::to_owned).collect(),
            OwnedPin {
                name: format!("{state}.pins.nibli"),
                source: query_pin_source(state, state_queries),
                expected_pins: state_queries.len(),
                expect_finding: false,
            },
        );
    }
    let base = snapshots
        .get("base")
        .ok_or_else(|| RedTeamError::new("snapshots: base snapshot is required"))?;
    validate_snapshot(kb, base, "snapshot base")?;
    let (base_additions, base_deletions) = snapshot_delta(base, "snapshot base")?;
    add_candidate_group(
        &mut groups,
        base_deletions.into_iter().map(str::to_owned).collect(),
        base_additions.into_iter().map(str::to_owned).collect(),
        OwnedPin {
            name: "inverted-sentinel.pins.nibli".to_owned(),
            source: sabotage_pin_source(),
            expected_pins: 1,
            expect_finding: true,
        },
    );

    let requested_workers = match std::env::var("RED_TEAM_JOBS") {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|_| RedTeamError::new("RED_TEAM_JOBS must be a positive integer"))?,
        Err(std::env::VarError::NotPresent) => 4,
        Err(error) => {
            return Err(RedTeamError::new(format!(
                "RED_TEAM_JOBS must be a positive integer: {error}"
            )));
        }
    };
    if requested_workers == 0 {
        return Err(RedTeamError::new(
            "RED_TEAM_JOBS must be a positive integer",
        ));
    }
    // The prepared engine owns a non-Sync transactional knowledge base. A
    // single parse followed by isolated patches is materially faster than
    // reparsing one full constitution per worker, so RED_TEAM_JOBS remains an
    // input validation contract but does not multiply native engine instances.
    let _requested_workers = requested_workers.min(groups.len());
    let prepared = PreparedPinEngine::new(&[LoadedSource::new(DEFAULT_KB, kb.text)]);
    for group in groups {
        run_candidate_group(&prepared, group)?;
    }
    Ok(ExecutionResult {
        snapshots: queries.len(),
        pins: pins_run,
        controls: 1,
    })
}

fn execute_scenarios_typed(source: &RedTeamSource, kb: &KbIndex<'_>) -> RedResult<ExecutionResult> {
    let snapshots: BTreeMap<_, _> = source
        .snapshots
        .iter()
        .map(|snapshot| (snapshot.id.clone(), snapshot))
        .collect();
    let queries = collect_queries_typed(source)?;
    let pins_run = queries.values().map(BTreeMap::len).sum();
    let mut groups = Vec::new();
    for (state, state_queries) in &queries {
        let snapshot = snapshots
            .get(state)
            .ok_or_else(|| RedTeamError::new(format!("missing executable snapshot {state}")))?;
        validate_snapshot_typed(kb, snapshot, &format!("snapshot {state}"))?;
        let (additions, deletions) = snapshot_delta_typed(snapshot, &format!("snapshot {state}"))?;
        add_candidate_group(
            &mut groups,
            deletions.into_iter().map(str::to_owned).collect(),
            additions.into_iter().map(str::to_owned).collect(),
            OwnedPin {
                name: format!("{state}.pins.nibli"),
                source: query_pin_source(state, state_queries),
                expected_pins: state_queries.len(),
                expect_finding: false,
            },
        );
    }
    let base = snapshots
        .get("base")
        .ok_or_else(|| RedTeamError::new("snapshots: base snapshot is required"))?;
    validate_snapshot_typed(kb, base, "snapshot base")?;
    let (base_additions, base_deletions) = snapshot_delta_typed(base, "snapshot base")?;
    add_candidate_group(
        &mut groups,
        base_deletions.into_iter().map(str::to_owned).collect(),
        base_additions.into_iter().map(str::to_owned).collect(),
        OwnedPin {
            name: "inverted-sentinel.pins.nibli".to_owned(),
            source: sabotage_pin_source(),
            expected_pins: 1,
            expect_finding: true,
        },
    );

    let requested_workers = match std::env::var("RED_TEAM_JOBS") {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|_| RedTeamError::new("RED_TEAM_JOBS must be a positive integer"))?,
        Err(std::env::VarError::NotPresent) => 4,
        Err(error) => {
            return Err(RedTeamError::new(format!(
                "RED_TEAM_JOBS must be a positive integer: {error}"
            )));
        }
    };
    if requested_workers == 0 {
        return Err(RedTeamError::new(
            "RED_TEAM_JOBS must be a positive integer",
        ));
    }
    let _requested_workers = requested_workers.min(groups.len());
    let prepared = PreparedPinEngine::new(&[LoadedSource::new(DEFAULT_KB, kb.text)]);
    for group in groups {
        run_candidate_group(&prepared, group)?;
    }
    Ok(ExecutionResult {
        snapshots: queries.len(),
        pins: pins_run,
        controls: 1,
    })
}

fn string_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => "None".to_owned(),
        Value::Bool(value) => if *value { "True" } else { "False" }.to_owned(),
        value => value.to_string(),
    }
}

fn markdown_value(value: &Value) -> String {
    string_value(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn code_value(value: &Value) -> String {
    let value = string_value(value);
    let fence = if value.contains('`') { "``" } else { "`" };
    format!("{fence}{value}{fence}")
}

fn markdown_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn code_text(value: &str) -> String {
    let fence = if value.contains('`') { "``" } else { "`" };
    format!("{fence}{value}{fence}")
}

fn title_case_key(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(characters).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_lines<'a>(lines: &mut Vec<String>, values: impl IntoIterator<Item = &'a str>) {
    lines.extend(values.into_iter().map(str::to_owned));
}

fn render(source: &Value) -> RedResult<String> {
    let source = object(source, "root")?;
    let postures = object(&source["posture_meanings"], "posture_meanings")?;
    let limits = object(&source["limits"], "limits")?;
    let acceptance = object(&source["acceptance_result"], "acceptance_result")?;
    let mut lines = Vec::new();
    lines.push(format!(
        "<!-- SPDX-License-Identifier: {} -->",
        source["spdx"].as_str().expect("validated SPDX")
    ));
    push_lines(
        &mut lines,
        [
            "<!-- Generated by the native rights-verify record-integrity-red-team refresh; do not edit. -->",
            "",
        ],
    );
    lines.push(format!(
        "# {}",
        source["title"].as_str().expect("validated title")
    ));
    push_lines(
        &mut lines,
        [
            "",
            "## Verdict and scope",
            "",
            "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED — bounded gap and boundary evidence, not record-integrity assurance.**",
            "",
        ],
    );
    lines.push(markdown_value(&acceptance["claim"]));
    push_lines(
        &mut lines,
        [
            "",
            "A green executable run means the release engine produced every reviewed",
            "consequence for the constructed snapshots. It does not authenticate those",
            "snapshots, attribute a write or absence, supersede the implemented T1/T3",
            "assurance case, or prove that an institution acts on a finding.",
            "",
            "| posture | meaning |",
            "| --- | --- |",
        ],
    );
    let mut posture_names = POSTURE_KEYS;
    posture_names.sort();
    for posture in posture_names {
        lines.push(format!(
            "| {} | {} |",
            code_value(&Value::String(posture.to_owned())),
            markdown_value(&postures[posture])
        ));
    }
    push_lines(&mut lines, ["", "## Limits", ""]);
    for key in LIMIT_KEYS {
        lines.push(format!(
            "- **{}:** {}",
            title_case_key(key),
            markdown_value(&limits[key])
        ));
    }

    push_lines(&mut lines, ["", "## Route postures", ""]);
    for raw_route in array(&source["routes"], "routes")? {
        let route = raw_route.as_object().expect("validated route");
        let tested = route["tested_delta_polarities"]
            .as_object()
            .expect("validated delta coverage");
        let delta_coverage = tested
            .iter()
            .map(|(premise, polarities)| {
                format!(
                    "{}: {}",
                    code_value(&Value::String(premise.clone())),
                    polarities
                        .as_array()
                        .expect("validated polarities")
                        .iter()
                        .map(string_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!(
            "### {} — {}",
            string_value(&route["id"]),
            string_value(&route["title"])
        ));
        lines.push(String::new());
        lines.push(format!(
            "- **Writable premise(s):** {}",
            route["premises"]
                .as_array()
                .expect("validated premises")
                .iter()
                .map(code_value)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(format!("- **Executed delta coverage:** {delta_coverage}"));
        for (label, field) in [
            ("Assertion harm", "assertion_harm"),
            ("Withholding/deletion harm", "withholding_deletion_harm"),
            (
                "Claimant/public-power polarity",
                "claimant_public_power_polarity",
            ),
            ("Current detectability", "current_detectability"),
            ("Safe default", "safe_default"),
            (
                "Authorised-disposition boundary",
                "authorised_disposition_boundary",
            ),
            ("Opposite-failure test", "opposite_failure_test"),
            ("Temporal status", "temporal_status"),
            ("Residual limit", "residual_limit"),
        ] {
            lines.push(format!("- **{label}:** {}", markdown_value(&route[field])));
        }
        lines.push(format!(
            "- **Assurance or repair owner:** {}",
            code_value(&route["owner_ref"])
        ));
        lines.push(format!(
            "- **Executable scenarios:** {}",
            route["scenario_refs"]
                .as_array()
                .expect("validated scenario refs")
                .iter()
                .map(code_value)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(String::new());
    }

    push_lines(
        &mut lines,
        [
            "## Executed snapshot manifest",
            "",
            "Each authored delta below is validated as exactly one ground atom. The",
            "full verifier executes the resulting ephemeral snapshot independently.",
            "",
            "| state | exact additions | exact deletions |",
            "| --- | --- | --- |",
        ],
    );
    for raw_snapshot in array(&source["snapshots"], "snapshots")? {
        let snapshot = raw_snapshot.as_object().expect("validated snapshot");
        let additions = snapshot["additions"]
            .as_array()
            .expect("validated additions")
            .iter()
            .map(code_value)
            .collect::<Vec<_>>()
            .join("<br>");
        let deletions = snapshot["deletions"]
            .as_array()
            .expect("validated deletions")
            .iter()
            .map(code_value)
            .collect::<Vec<_>>()
            .join("<br>");
        lines.push(format!(
            "| {} | {} | {} |",
            code_value(&snapshot["id"]),
            if additions.is_empty() {
                "â€”"
            } else {
                &additions
            },
            if deletions.is_empty() {
                "â€”"
            } else {
                &deletions
            },
        ));
    }

    push_lines(
        &mut lines,
        [
            "",
            "## Executable scenario summary",
            "",
            "| scenario | route(s) | kind | result | attribution limit |",
            "| --- | --- | --- | --- | --- |",
        ],
    );
    for raw_scenario in array(&source["scenarios"], "scenarios")? {
        let scenario = raw_scenario.as_object().expect("validated scenario");
        lines.push(format!(
            "| {} {} | {} | {} | {} | {} |",
            code_value(&scenario["id"]),
            markdown_value(&scenario["title"]),
            scenario["route_refs"]
                .as_array()
                .expect("validated route refs")
                .iter()
                .map(code_value)
                .collect::<Vec<_>>()
                .join(", "),
            code_value(&scenario["kind"]),
            code_value(&scenario["result"]),
            code_value(&scenario["attribution"]),
        ));
    }

    push_lines(&mut lines, ["", "## Executable scenario details", ""]);
    for raw_scenario in array(&source["scenarios"], "scenarios")? {
        let scenario = raw_scenario.as_object().expect("validated scenario");
        lines.push(format!(
            "### {} — {}",
            string_value(&scenario["id"]),
            string_value(&scenario["title"])
        ));
        lines.push(String::new());
        lines.push(format!(
            "**{}**",
            string_value(&scenario["result"])
                .replace('_', " ")
                .to_uppercase()
        ));
        lines.push(String::new());
        lines.push(format!(
            "- **Attribution:** {}",
            code_value(&scenario["attribution"])
        ));
        lines.push(format!(
            "- **States:** {}",
            scenario["state_refs"]
                .as_array()
                .expect("validated states")
                .iter()
                .map(code_value)
                .collect::<Vec<_>>()
                .join(", ")
        ));
        for (label, field) in [
            ("Interpretation", "interpretation"),
            (
                "Authorised-disposition boundary",
                "authorised_disposition_boundary",
            ),
            ("Opposite failure", "opposite_failure"),
            ("Residual limit", "residual_limit"),
        ] {
            lines.push(format!(
                "- **{label}:** {}",
                markdown_value(&scenario[field])
            ));
        }
        push_lines(
            &mut lines,
            [
                "",
                "| state | query | expected | purpose |",
                "| --- | --- | --- | --- |",
            ],
        );
        for raw_query in scenario["queries"].as_array().expect("validated queries") {
            let query = raw_query.as_object().expect("validated query");
            lines.push(format!(
                "| {} | {} | **{}** | {} |",
                code_value(&query["state"]),
                code_value(&query["expression"]),
                string_value(&query["expected"]),
                markdown_value(&query["purpose"]),
            ));
        }
        let comparisons = scenario["comparisons"]
            .as_array()
            .expect("validated comparisons");
        if !comparisons.is_empty() {
            push_lines(&mut lines, ["", "**Discriminating flips**", ""]);
            for raw_item in comparisons {
                let item = raw_item.as_object().expect("validated comparison");
                lines.push(format!(
                    "- {}: {} {} → {} {} — {}",
                    code_value(&item["expression"]),
                    code_value(&item["from_state"]),
                    string_value(&item["from_expected"]),
                    code_value(&item["to_state"]),
                    string_value(&item["to_expected"]),
                    markdown_value(&item["claim"]),
                ));
            }
        }
        push_lines(&mut lines, ["", "**Preserved controls**", ""]);
        for raw_item in scenario["preserved_invariants"]
            .as_array()
            .expect("validated invariants")
        {
            let item = raw_item.as_object().expect("validated invariant");
            lines.push(format!(
                "- {} stays **{}** from {} to {} — {}",
                code_value(&item["expression"]),
                string_value(&item["expected"]),
                code_value(&item["from_state"]),
                code_value(&item["to_state"]),
                markdown_value(&item["claim"]),
            ));
        }
        lines.push(String::new());
    }

    push_lines(
        &mut lines,
        [
            "## Flat-snapshot indistinguishability boundary",
            "",
            "Each case deliberately maps multiple real-world descriptions to one",
            "identical snapshot and query vector outside the currently witnessed",
            "temporal scopes. No extra fact identifies which world occurred.",
            "",
        ],
    );
    for raw_entry in array(
        &source["observational_equivalence"],
        "observational_equivalence",
    )? {
        let entry = raw_entry
            .as_object()
            .expect("validated observational entry");
        lines.push(format!(
            "### {} — {}",
            string_value(&entry["id"]),
            string_value(&entry["title"])
        ));
        lines.push(String::new());
        lines.push(format!(
            "- **One snapshot:** {}",
            code_value(&entry["snapshot_ref"])
        ));
        lines.push(format!(
            "- **Worlds with the same record:** {}",
            entry["world_descriptions"]
                .as_array()
                .expect("validated worlds")
                .iter()
                .map(markdown_value)
                .collect::<Vec<_>>()
                .join("; ")
        ));
        lines.push(format!(
            "- **Boundary:** {}",
            markdown_value(&entry["boundary"])
        ));
        lines.push(format!(
            "- **Prohibited inference:** {}",
            markdown_value(&entry["prohibited_inference"])
        ));
        push_lines(
            &mut lines,
            [
                "",
                "| query | expected in every described world |",
                "| --- | --- |",
            ],
        );
        for raw_query in entry["queries"].as_array().expect("validated queries") {
            let query = raw_query.as_object().expect("validated query");
            lines.push(format!(
                "| {} | **{}** |",
                code_value(&query["expression"]),
                string_value(&query["expected"])
            ));
        }
        lines.push(String::new());
    }

    let handoff = source["temporal_handoff"]
        .as_object()
        .expect("validated temporal handoff");
    push_lines(&mut lines, ["## Temporal assurance handoff", ""]);
    lines.push(markdown_value(&handoff["current_contract"]));
    lines.push(String::new());
    lines.push(format!(
        "- **Owner:** {}",
        code_value(&handoff["owner_ref"])
    ));
    lines.push(format!(
        "- **Owned executable cases:** {}",
        handoff["owned_cases"]
            .as_array()
            .expect("validated cases")
            .iter()
            .map(code_value)
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(format!(
        "- **Residual boundary:** {}",
        markdown_value(&handoff["residual_boundary"])
    ));

    push_lines(
        &mut lines,
        [
            "",
            "## Narrowness impacts",
            "",
            "This red-team changes no formal rule. It exposed several prose claims as",
            "too broad; those claims were revised and scoped in the same change. Other",
            "standing claims remain true only with the limits recorded here.",
            "",
            "| artifact | current claim | classification | reason | future trigger |",
            "| --- | --- | --- | --- | --- |",
        ],
    );
    for raw_entry in array(&source["narrowness_impacts"], "narrowness_impacts")? {
        let entry = raw_entry.as_object().expect("validated narrowness entry");
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code_value(&entry["artifact_ref"]),
            markdown_value(&entry["current_claim"]),
            code_value(&entry["classification"]),
            markdown_value(&entry["reason"]),
            markdown_value(&entry["future_trigger"]),
        ));
    }

    push_lines(
        &mut lines,
        [
            "",
            "## Acceptance result",
            "",
            "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED.**",
            "",
        ],
    );
    lines.push(markdown_value(&acceptance["claim"]));
    push_lines(
        &mut lines,
        ["", "This artifact does **not** establish:", ""],
    );
    for value in acceptance["does_not_establish"]
        .as_array()
        .expect("validated residuals")
    {
        lines.push(format!("- {}", markdown_value(value)));
    }
    lines.push(String::new());
    lines.push(format!(
        "Remaining gap owner: {}.",
        code_value(&acceptance["remaining_owner"])
    ));
    push_lines(&mut lines, ["", "## Maintenance", ""]);
    lines.push(format!(
        "- Reviewed source: {}.",
        code_value(&Value::String(DEFAULT_SOURCE.to_owned()))
    ));
    lines.push(format!(
        "- Constitution: {}, SHA-256 {}.",
        code_value(&Value::String(DEFAULT_KB.to_owned())),
        code_value(&source["constitution_sha256"])
    ));
    lines.push(format!(
        "- Assertion ledger: {}, SHA-256 {}.",
        code_value(&Value::String(DEFAULT_LEDGER.to_owned())),
        code_value(&source["assertion_surface_contracts_sha256"])
    ));
    lines.push(format!(
        "- Assurance source: {}, SHA-256 {}.",
        code_value(&Value::String(DEFAULT_ASSURANCE.to_owned())),
        code_value(&source["record_integrity_assurance_case_sha256"])
    ));
    push_lines(
        &mut lines,
        [
            "- Regenerate only through `./verify.sh --refresh record-integrity-red-team`.",
            "- Fast freshness/schema check: `./verify.sh --quick`.",
            "- Authoritative executable check: `./verify.sh`.",
            "- The executable snapshots are temporary and remain outside chapter `:expect-pins` reconciliation.",
            "",
        ],
    );
    Ok(lines.join("\n"))
}

fn render_typed(source: &RedTeamSource) -> String {
    let acceptance = &source.acceptance_result;
    let mut lines = Vec::new();
    lines.push(format!("<!-- SPDX-License-Identifier: {} -->", source.spdx));
    push_lines(
        &mut lines,
        [
            "<!-- Generated by the native rights-verify record-integrity-red-team refresh; do not edit. -->",
            "",
        ],
    );
    lines.push(format!("# {}", source.title));
    push_lines(
        &mut lines,
        [
            "",
            "## Verdict and scope",
            "",
            "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED — bounded gap and boundary evidence, not record-integrity assurance.**",
            "",
        ],
    );
    lines.push(markdown_text(&acceptance.claim));
    push_lines(
        &mut lines,
        [
            "",
            "A green executable run means the release engine produced every reviewed",
            "consequence for the constructed snapshots. It does not authenticate those",
            "snapshots, attribute a write or absence, supersede the implemented T1/T3",
            "assurance case, or prove that an institution acts on a finding.",
            "",
            "| posture | meaning |",
            "| --- | --- |",
        ],
    );
    let mut posture_names = POSTURE_KEYS;
    posture_names.sort();
    for posture in posture_names {
        lines.push(format!(
            "| {} | {} |",
            code_text(posture),
            markdown_text(&source.posture_meanings[posture])
        ));
    }
    push_lines(&mut lines, ["", "## Limits", ""]);
    for key in LIMIT_KEYS {
        lines.push(format!(
            "- **{}:** {}",
            title_case_key(key),
            markdown_text(&source.limits[key])
        ));
    }

    push_lines(&mut lines, ["", "## Route postures", ""]);
    for route in &source.routes {
        let delta_coverage = route
            .tested_delta_polarities
            .iter()
            .map(|(premise, polarities)| {
                format!("{}: {}", code_text(premise), polarities.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!("### {} — {}", route.id, route.title));
        lines.push(String::new());
        lines.push(format!(
            "- **Writable premise(s):** {}",
            route
                .premises
                .iter()
                .map(|value| code_text(value))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(format!("- **Executed delta coverage:** {delta_coverage}"));
        for (label, value) in [
            ("Assertion harm", &route.assertion_harm),
            (
                "Withholding/deletion harm",
                &route.withholding_deletion_harm,
            ),
            (
                "Claimant/public-power polarity",
                &route.claimant_public_power_polarity,
            ),
            ("Current detectability", &route.current_detectability),
            ("Safe default", &route.safe_default),
            (
                "Authorised-disposition boundary",
                &route.authorised_disposition_boundary,
            ),
            ("Opposite-failure test", &route.opposite_failure_test),
            ("Temporal status", &route.temporal_status),
            ("Residual limit", &route.residual_limit),
        ] {
            lines.push(format!("- **{label}:** {}", markdown_text(value)));
        }
        lines.push(format!(
            "- **Assurance or repair owner:** {}",
            code_text(&route.owner_ref)
        ));
        lines.push(format!(
            "- **Executable scenarios:** {}",
            route
                .scenario_refs
                .iter()
                .map(|value| code_text(value))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push(String::new());
    }

    push_lines(
        &mut lines,
        [
            "## Executed snapshot manifest",
            "",
            "Each authored delta below is validated as exactly one ground atom. The",
            "full verifier executes the resulting ephemeral snapshot independently.",
            "",
            "| state | exact additions | exact deletions |",
            "| --- | --- | --- |",
        ],
    );
    for snapshot in &source.snapshots {
        let additions = snapshot
            .additions
            .iter()
            .map(|value| code_text(value))
            .collect::<Vec<_>>()
            .join("<br>");
        let deletions = snapshot
            .deletions
            .iter()
            .map(|value| code_text(value))
            .collect::<Vec<_>>()
            .join("<br>");
        lines.push(format!(
            "| {} | {} | {} |",
            code_text(&snapshot.id),
            if additions.is_empty() {
                "â€”"
            } else {
                &additions
            },
            if deletions.is_empty() {
                "â€”"
            } else {
                &deletions
            },
        ));
    }

    push_lines(
        &mut lines,
        [
            "",
            "## Executable scenario summary",
            "",
            "| scenario | route(s) | kind | result | attribution limit |",
            "| --- | --- | --- | --- | --- |",
        ],
    );
    for scenario in &source.scenarios {
        lines.push(format!(
            "| {} {} | {} | {} | {} | {} |",
            code_text(&scenario.id),
            markdown_text(&scenario.title),
            scenario
                .route_refs
                .iter()
                .map(|value| code_text(value))
                .collect::<Vec<_>>()
                .join(", "),
            code_text(&scenario.kind),
            code_text(&scenario.result),
            code_text(&scenario.attribution),
        ));
    }

    push_lines(&mut lines, ["", "## Executable scenario details", ""]);
    for scenario in &source.scenarios {
        lines.push(format!("### {} — {}", scenario.id, scenario.title));
        lines.push(String::new());
        lines.push(format!(
            "**{}**",
            scenario.result.replace('_', " ").to_uppercase()
        ));
        lines.push(String::new());
        lines.push(format!(
            "- **Attribution:** {}",
            code_text(&scenario.attribution)
        ));
        lines.push(format!(
            "- **States:** {}",
            scenario
                .state_refs
                .iter()
                .map(|value| code_text(value))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        for (label, value) in [
            ("Interpretation", &scenario.interpretation),
            (
                "Authorised-disposition boundary",
                &scenario.authorised_disposition_boundary,
            ),
            ("Opposite failure", &scenario.opposite_failure),
            ("Residual limit", &scenario.residual_limit),
        ] {
            lines.push(format!("- **{label}:** {}", markdown_text(value)));
        }
        push_lines(
            &mut lines,
            [
                "",
                "| state | query | expected | purpose |",
                "| --- | --- | --- | --- |",
            ],
        );
        for query in &scenario.queries {
            lines.push(format!(
                "| {} | {} | **{}** | {} |",
                code_text(&query.state),
                code_text(&query.expression),
                query.expected,
                markdown_text(&query.purpose),
            ));
        }
        if !scenario.comparisons.is_empty() {
            push_lines(&mut lines, ["", "**Discriminating flips**", ""]);
            for item in &scenario.comparisons {
                lines.push(format!(
                    "- {}: {} {} → {} {} — {}",
                    code_text(&item.expression),
                    code_text(&item.from_state),
                    item.from_expected,
                    code_text(&item.to_state),
                    item.to_expected,
                    markdown_text(&item.claim),
                ));
            }
        }
        push_lines(&mut lines, ["", "**Preserved controls**", ""]);
        for item in &scenario.preserved_invariants {
            lines.push(format!(
                "- {} stays **{}** from {} to {} — {}",
                code_text(&item.expression),
                item.expected,
                code_text(&item.from_state),
                code_text(&item.to_state),
                markdown_text(&item.claim),
            ));
        }
        lines.push(String::new());
    }

    push_lines(
        &mut lines,
        [
            "## Flat-snapshot indistinguishability boundary",
            "",
            "Each case deliberately maps multiple real-world descriptions to one",
            "identical snapshot and query vector outside the currently witnessed",
            "temporal scopes. No extra fact identifies which world occurred.",
            "",
        ],
    );
    for entry in &source.observational_equivalence {
        lines.push(format!("### {} — {}", entry.id, entry.title));
        lines.push(String::new());
        lines.push(format!(
            "- **One snapshot:** {}",
            code_text(&entry.snapshot_ref)
        ));
        lines.push(format!(
            "- **Worlds with the same record:** {}",
            entry
                .world_descriptions
                .iter()
                .map(|value| markdown_text(value))
                .collect::<Vec<_>>()
                .join("; ")
        ));
        lines.push(format!(
            "- **Boundary:** {}",
            markdown_text(&entry.boundary)
        ));
        lines.push(format!(
            "- **Prohibited inference:** {}",
            markdown_text(&entry.prohibited_inference)
        ));
        push_lines(
            &mut lines,
            [
                "",
                "| query | expected in every described world |",
                "| --- | --- |",
            ],
        );
        for query in &entry.queries {
            lines.push(format!(
                "| {} | **{}** |",
                code_text(&query.expression),
                query.expected
            ));
        }
        lines.push(String::new());
    }

    let handoff = &source.temporal_handoff;
    push_lines(&mut lines, ["## Temporal assurance handoff", ""]);
    lines.push(markdown_text(&handoff.current_contract));
    lines.push(String::new());
    lines.push(format!("- **Owner:** {}", code_text(&handoff.owner_ref)));
    lines.push(format!(
        "- **Owned executable cases:** {}",
        handoff
            .owned_cases
            .iter()
            .map(|value| code_text(value))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(format!(
        "- **Residual boundary:** {}",
        markdown_text(&handoff.residual_boundary)
    ));

    push_lines(
        &mut lines,
        [
            "",
            "## Narrowness impacts",
            "",
            "This red-team changes no formal rule. It exposed several prose claims as",
            "too broad; those claims were revised and scoped in the same change. Other",
            "standing claims remain true only with the limits recorded here.",
            "",
            "| artifact | current claim | classification | reason | future trigger |",
            "| --- | --- | --- | --- | --- |",
        ],
    );
    for entry in &source.narrowness_impacts {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code_text(&entry.artifact_ref),
            markdown_text(&entry.current_claim),
            code_text(&entry.classification),
            markdown_text(&entry.reason),
            markdown_text(&entry.future_trigger),
        ));
    }

    push_lines(
        &mut lines,
        [
            "",
            "## Acceptance result",
            "",
            "**CURRENT FLAT-SNAPSHOT HARMS REPRODUCED.**",
            "",
        ],
    );
    lines.push(markdown_text(&acceptance.claim));
    push_lines(
        &mut lines,
        ["", "This artifact does **not** establish:", ""],
    );
    for value in &acceptance.does_not_establish {
        lines.push(format!("- {}", markdown_text(value)));
    }
    lines.push(String::new());
    lines.push(format!(
        "Remaining gap owner: {}.",
        code_text(&acceptance.remaining_owner)
    ));
    push_lines(&mut lines, ["", "## Maintenance", ""]);
    lines.push(format!("- Reviewed source: {}.", code_text(DEFAULT_SOURCE)));
    lines.push(format!(
        "- Constitution: {}, SHA-256 {}.",
        code_text(DEFAULT_KB),
        code_text(&source.constitution_sha256)
    ));
    lines.push(format!(
        "- Assertion ledger: {}, SHA-256 {}.",
        code_text(DEFAULT_LEDGER),
        code_text(&source.assertion_surface_contracts_sha256)
    ));
    lines.push(format!(
        "- Assurance source: {}, SHA-256 {}.",
        code_text(DEFAULT_ASSURANCE),
        code_text(&source.record_integrity_assurance_case_sha256)
    ));
    push_lines(
        &mut lines,
        [
            "- Regenerate only through `./verify.sh --refresh record-integrity-red-team`.",
            "- Fast freshness/schema check: `./verify.sh --quick`.",
            "- Authoritative executable check: `./verify.sh`.",
            "- The executable snapshots are temporary and remain outside chapter `:expect-pins` reconciliation.",
            "",
        ],
    );
    lines.join("\n")
}

fn expect_failure<T>(label: &str, result: RedResult<T>) -> RedResult<()> {
    if result.is_err() {
        Ok(())
    } else {
        Err(RedTeamError::new(format!(
            "negative control did not fail: {label}"
        )))
    }
}

fn root_mut(value: &mut Value) -> &mut Map<String, Value> {
    value.as_object_mut().expect("cloned source is an object")
}

fn collection_mut<'a>(value: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
    root_mut(value)[key]
        .as_array_mut()
        .expect("validated collection is an array")
}

fn row_mut<'a>(
    value: &'a mut Value,
    collection: &str,
    identifier: &str,
) -> &'a mut Map<String, Value> {
    collection_mut(value, collection)
        .iter_mut()
        .find_map(|row| {
            let row = row.as_object_mut()?;
            (row.get("id").and_then(Value::as_str) == Some(identifier)).then_some(row)
        })
        .expect("governed row exists")
}

fn validate_candidate(
    candidate: &Value,
    kb: &KbIndex<'_>,
    kb_digest: &str,
    ledger_digest: &str,
    assurance_digest: &str,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<()> {
    let candidate: RedTeamSource = serde_json::from_value(candidate.clone())
        .map_err(|error| RedTeamError::new(format!("candidate schema is invalid: {error}")))?;
    validate_source_typed(
        &candidate,
        kb,
        kb_digest,
        ledger_digest,
        assurance_digest,
        references,
    )
    .map(|_| ())
}

fn negative_controls(
    source: &Value,
    kb: &KbIndex<'_>,
    kb_digest: &str,
    ledger_digest: &str,
    assurance_digest: &str,
    references: &mut ReferenceResolver<'_>,
) -> RedResult<usize> {
    let mut controls = 0usize;
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
        let mut changed = source.clone();
        root_mut(&mut changed).insert(key.to_owned(), Value::String("0".repeat(64)));
        expect_failure(
            label,
            validate_candidate(
                &changed,
                kb,
                kb_digest,
                ledger_digest,
                assurance_digest,
                references,
            ),
        )?;
        controls += 1;
    }

    let mut changed = source.clone();
    root_mut(&mut changed).insert("schema_version".to_owned(), Value::Bool(true));
    expect_failure(
        "boolean schema version",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "routes").remove(0);
    expect_failure(
        "required route deleted",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "scenarios").remove(0);
    expect_failure(
        "required scenario deleted",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    let duplicate = collection_mut(&mut changed, "routes")[0].clone();
    collection_mut(&mut changed, "routes").push(duplicate);
    expect_failure(
        "duplicate route ID",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "scenarios")[0]
        .as_object_mut()
        .expect("scenario is object")
        .insert(
            "route_refs".to_owned(),
            Value::Array(vec![Value::String("RT-99".to_owned())]),
        );
    expect_failure(
        "dangling route reference",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "routes")[0]
        .as_object_mut()
        .expect("route is object")
        .remove("authorised_disposition_boundary");
    expect_failure(
        "missing authorised-disposition boundary",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "routes")[0]
        .as_object_mut()
        .expect("route is object")
        .insert(
            "premises".to_owned(),
            Value::Array(vec![
                Value::String("free".to_owned()),
                Value::String("invented_relation".to_owned()),
            ]),
        );
    expect_failure(
        "required premise coverage drift",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "routes")[0]
        .as_object_mut()
        .expect("route is object")
        .get_mut("tested_delta_polarities")
        .and_then(Value::as_object_mut)
        .expect("polarities object")
        .insert(
            "free".to_owned(),
            Value::Array(vec![Value::String("deletion".to_owned())]),
        );
    expect_failure(
        "declared delta coverage drift",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "snapshots", "free_adam")["additions"]
        .as_array_mut()
        .expect("additions array")
        .push(Value::String("person(Adam).".to_owned()));
    expect_failure(
        "no-op existing addition",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "snapshots", "free_adam").insert(
        "additions".to_owned(),
        Value::Array(vec![Value::String(
            "free(Probe). person(Probe_Injected).".to_owned(),
        )]),
    );
    expect_failure(
        "multiple statements hidden in one addition",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "snapshots", "no_adulthood_hano").insert(
        "deletions".to_owned(),
        Value::Array(vec![Value::String(
            "at(NeverThere, GeneralAdulthood).".to_owned(),
        )]),
    );
    expect_failure(
        "deletion with zero exact matches",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let duplicate_base = format!("{}at(Hano, GeneralAdulthood).\n", kb.text);
    let duplicate_index = KbIndex::new(&duplicate_base);
    let source_object = source.as_object().expect("source object");
    let deletion_snapshot = source_object["snapshots"]
        .as_array()
        .expect("snapshots array")
        .iter()
        .find_map(|value| {
            let snapshot = value.as_object()?;
            (snapshot["id"].as_str() == Some("no_adulthood_hano")).then_some(snapshot)
        })
        .expect("deletion snapshot");
    expect_failure(
        "deletion with multiple exact matches",
        apply_snapshot(
            &duplicate_index,
            deletion_snapshot,
            "duplicate deletion control",
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    let scenario = row_mut(&mut changed, "scenarios", "RS-01");
    let query = scenario["queries"]
        .as_array_mut()
        .expect("queries array")
        .iter_mut()
        .find_map(|value| {
            let query = value.as_object_mut()?;
            (query["state"].as_str() == Some("base")
                && query["expression"].as_str() == Some("prisoner(Adam)"))
            .then_some(query)
        })
        .expect("sentinel query");
    query.insert("expected".to_owned(), Value::String("FALSE".to_owned()));
    expect_failure(
        "reversed expected verdict",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "scenarios", "RS-01")["queries"]
        .as_array_mut()
        .expect("queries array")[0]
        .as_object_mut()
        .expect("query object")
        .insert(
            "expression".to_owned(),
            Value::String("free(Adam). ? person(Eve)".to_owned()),
        );
    expect_failure(
        "query statement injection",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "scenarios", "RS-01")
        .insert("comparisons".to_owned(), Value::Array(Vec::new()));
    expect_failure(
        "missing discriminating flip",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    let comparison = row_mut(&mut changed, "scenarios", "RS-01")["comparisons"]
        .as_array_mut()
        .expect("comparisons array")[0]
        .as_object_mut()
        .expect("comparison object");
    let from_expected = comparison["from_expected"].clone();
    comparison.insert("to_expected".to_owned(), from_expected);
    expect_failure(
        "non-discriminating comparison",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "scenarios", "RS-01")
        .insert("preserved_invariants".to_owned(), Value::Array(Vec::new()));
    expect_failure(
        "missing preserved positive control",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "scenarios")[0]
        .as_object_mut()
        .expect("scenario object")
        .remove("opposite_failure");
    expect_failure(
        "missing opposite-failure analysis",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    let states = row_mut(&mut changed, "scenarios", "RS-07")["state_refs"]
        .as_array_mut()
        .expect("state refs");
    let position = states
        .iter()
        .position(|value| value.as_str() == Some("vex_judgment_only"))
        .expect("matrix state");
    states.remove(position);
    expect_failure(
        "incomplete two-entry matrix",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    row_mut(&mut changed, "scenarios", "RS-03").insert(
        "attribution".to_owned(),
        Value::String("deletion_proved".to_owned()),
    );
    expect_failure(
        "disappearance falsely attributed",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    for (key, value, label) in [
        (
            "status",
            "general_temporal_assurance",
            "bounded red-team promoted to general assurance",
        ),
        (
            "evidence_role",
            "supports_current",
            "gap evidence promoted to assurance",
        ),
    ] {
        let mut changed = source.clone();
        root_mut(&mut changed).insert(key.to_owned(), Value::String(value.to_owned()));
        expect_failure(
            label,
            validate_candidate(
                &changed,
                kb,
                kb_digest,
                ledger_digest,
                assurance_digest,
                references,
            ),
        )?;
        controls += 1;
    }

    let mut changed = source.clone();
    collection_mut(&mut changed, "narrowness_impacts").remove(0);
    expect_failure(
        "standing narrowness claim omitted",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "narrowness_impacts")[0]
        .as_object_mut()
        .expect("narrowness row")
        .insert(
            "classification".to_owned(),
            Value::String("preserved_unqualified".to_owned()),
        );
    expect_failure(
        "unknown narrowness disposition",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    let residuals = root_mut(&mut changed)["acceptance_result"]
        .as_object_mut()
        .expect("acceptance object")["does_not_establish"]
        .as_array_mut()
        .expect("residuals array");
    residuals.retain(|value| !value.as_str().unwrap_or_default().contains("recovery"));
    expect_failure(
        "residual recovery boundary erased",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    collection_mut(&mut changed, "routes")[0]
        .as_object_mut()
        .expect("route object")
        .insert(
            "owner_ref".to_owned(),
            Value::String("TODO.md::heading that does not exist".to_owned()),
        );
    expect_failure(
        "dangling route owner",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    root_mut(&mut changed)["temporal_handoff"]
        .as_object_mut()
        .expect("handoff object")
        .insert(
            "owned_cases".to_owned(),
            Value::Array(vec![Value::String("TA-02".to_owned())]),
        );
    expect_failure(
        "delegated temporal coverage erased",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    let mut changed = source.clone();
    root_mut(&mut changed)["temporal_handoff"]
        .as_object_mut()
        .expect("handoff object")
        .insert(
            "owner_ref".to_owned(),
            Value::String(
                "new-book-plans/12-temporal-assurance.py::heading that does not exist".to_owned(),
            ),
        );
    expect_failure(
        "dangling temporal-assurance owner",
        validate_candidate(
            &changed,
            kb,
            kb_digest,
            ledger_digest,
            assurance_digest,
            references,
        ),
    )?;
    controls += 1;

    expect_failure(
        "duplicate JSON object key",
        load_json(
            r#"{"status": "bounded", "status": "assured"}"#,
            "negative-control source",
        ),
    )?;
    controls += 1;
    Ok(controls)
}

pub(crate) fn run(
    context: &Context,
    mode: Mode,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run_inner(context, mode, snapshot)
        .map_err(|error| Error::new(format!("9-record-integrity-red-team: {error}")))
}

pub(crate) fn check(
    context: &Context,
    execute: bool,
    snapshot: InputSnapshot<'_>,
) -> Result<String, Error> {
    run(
        context,
        if execute {
            Mode::CheckExecute
        } else {
            Mode::Check
        },
        snapshot,
    )
}

pub(crate) fn generate(context: &Context, snapshot: InputSnapshot<'_>) -> Result<String, Error> {
    run(context, Mode::Generate, snapshot)
}

fn read_context(context: &Context, path: &str, label: &str) -> RedResult<String> {
    std::fs::read_to_string(context.path(path)).map_err(|error| {
        RedTeamError::new(format!(
            "cannot read {label} {}: {error}",
            context.path(path).display()
        ))
    })
}

fn run_inner(context: &Context, mode: Mode, snapshot: InputSnapshot<'_>) -> RedResult<String> {
    let source_owned;
    let source_text = if let Some(value) = snapshot.source_json {
        value
    } else {
        source_owned = read_context(context, DEFAULT_SOURCE, "red-team source")?;
        &source_owned
    };
    let kb_owned;
    let kb_text = if let Some(value) = snapshot.constitution {
        value
    } else {
        kb_owned = read_context(context, DEFAULT_KB, "constitution")?;
        &kb_owned
    };
    let ledger_owned;
    let ledger_text = if let Some(value) = snapshot.assertion_ledger {
        value
    } else {
        ledger_owned = read_context(context, DEFAULT_LEDGER, "assertion ledger")?;
        &ledger_owned
    };
    let assurance_owned;
    let assurance_text = if let Some(value) = snapshot.assurance_source {
        value
    } else {
        assurance_owned = read_context(context, DEFAULT_ASSURANCE, "assurance source")?;
        &assurance_owned
    };

    let (source, source_control_value) =
        load_json(source_text, &format!("red-team source {DEFAULT_SOURCE}"))?;
    let kb_digest = sha256(kb_text.as_bytes());
    let ledger_digest = sha256(ledger_text.as_bytes());
    let assurance_digest = sha256(assurance_text.as_bytes());
    let kb = KbIndex::new(kb_text);
    let mut references = ReferenceResolver::new(context);
    validate_source_typed(
        &source,
        &kb,
        &kb_digest,
        &ledger_digest,
        &assurance_digest,
        &mut references,
    )?;
    let generated = render_typed(&source);
    let controls = negative_controls(
        &source_control_value,
        &kb,
        &kb_digest,
        &ledger_digest,
        &assurance_digest,
        &mut references,
    )?;

    let should_execute = mode != Mode::Check;
    let execution = if should_execute {
        execute_scenarios_typed(&source, &kb)?
    } else {
        ExecutionResult {
            snapshots: 0,
            pins: 0,
            controls: 0,
        }
    };

    match mode {
        Mode::Check | Mode::CheckExecute => {
            let report_owned;
            let current = if let Some(value) = snapshot.generated_report {
                value
            } else {
                report_owned = read_context(context, DEFAULT_OUTPUT, "generated report")?;
                &report_owned
            };
            if current != generated {
                return Err(RedTeamError::new(format!(
                    "{DEFAULT_OUTPUT} is STALE — rerun without --check"
                )));
            }
            let suffix = if should_execute {
                format!(
                    "; {} snapshots / {} pins execute; {} executable sabotage passes",
                    execution.snapshots, execution.pins, execution.controls
                )
            } else {
                "; execution skipped".to_owned()
            };
            Ok(format!(
                "{DEFAULT_OUTPUT} is current; {controls} structural negative controls pass{suffix}"
            ))
        }
        Mode::Generate => {
            let output = context.path(DEFAULT_OUTPUT);
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    RedTeamError::new(format!("cannot create {}: {error}", parent.display()))
                })?;
            }
            std::fs::write(&output, generated).map_err(|error| {
                RedTeamError::new(format!("cannot write {}: {error}", output.display()))
            })?;
            Ok(format!(
                "{DEFAULT_OUTPUT}: regenerated after {} snapshots / {} pins; {controls} structural and {} executable negative controls pass",
                execution.snapshots, execution.pins, execution.controls
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    #[test]
    fn duplicate_json_keys_are_rejected_at_every_depth() {
        let error = load_json(
            r#"{"outer":{"value":1,"value":2}}"#,
            "duplicate-key fixture",
        )
        .expect_err("nested duplicate key must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: value")
        );
    }

    #[test]
    fn ground_atoms_allow_event_terms_and_reject_statement_injection() {
        assert_eq!(
            validate_ground_atom_text("entitled(Bela, event { eats() })", "fixture")
                .expect("event term is a ground atom"),
            ("entitled", "entitled(Bela, event { eats() })")
        );
        for invalid in [
            "free(Adam). ? person(Eve)",
            "free(Adam) trailing",
            "free((Adam)",
            "Free(Adam)",
            "free(Adam)\nperson(Eve)",
        ] {
            assert!(
                validate_ground_atom_text(invalid, "fixture").is_err(),
                "accepted invalid atom {invalid:?}"
            );
        }
    }

    #[test]
    fn snapshot_transformations_are_exact_and_byte_stable() {
        let base = "admits(\"person\").\nperson(Adam).\nperson(Bela).\n";
        let snapshot = json!({
            "id": "swap",
            "description": "replace one exact fact",
            "additions": ["person(Eve)."],
            "deletions": ["person(Bela)."]
        });
        assert_eq!(
            apply_snapshot(
                &KbIndex::new(base),
                snapshot.as_object().expect("snapshot object"),
                "snapshot swap"
            )
            .expect("snapshot applies"),
            concat!(
                "admits(\"person\").\n",
                "person(Adam).\n",
                "\n",
                "# Red-team snapshot additions (generated, not enacted).\n",
                "person(Eve).\n",
            )
        );

        let existing_addition = json!({
            "id": "bad",
            "description": "existing addition",
            "additions": ["person(Adam)."],
            "deletions": []
        });
        assert!(
            apply_snapshot(
                &KbIndex::new(base),
                existing_addition.as_object().expect("snapshot object"),
                "snapshot bad"
            )
            .expect_err("existing addition must fail")
            .to_string()
            .contains("addition is not exact and new")
        );

        let duplicate_base = "person(Adam).\nperson(Adam).\n";
        let ambiguous_deletion = json!({
            "id": "bad",
            "description": "ambiguous deletion",
            "additions": [],
            "deletions": ["person(Adam)."]
        });
        assert!(
            apply_snapshot(
                &KbIndex::new(duplicate_base),
                ambiguous_deletion.as_object().expect("snapshot object"),
                "snapshot bad"
            )
            .expect_err("non-exact deletion must fail")
            .to_string()
            .contains("deletion must match exactly once")
        );
    }

    #[test]
    fn grouped_native_execution_runs_queries_and_inverted_sabotage() {
        let kb_text = concat!(
            "admits(\"person\").\n",
            "admits(\"prisoner\").\n",
            "person(Adam).\n",
            "prisoner(Adam).\n",
        );
        let source = json!({
            "snapshots": [
                {
                    "id": "base",
                    "description": "unchanged base",
                    "additions": [],
                    "deletions": []
                },
                {
                    "id": "eve",
                    "description": "one added person",
                    "additions": ["person(Eve)."],
                    "deletions": []
                }
            ],
            "scenarios": [{
                "queries": [
                    {"state": "base", "expression": "person(Adam)", "expected": "TRUE"},
                    {"state": "base", "expression": "prisoner(Adam)", "expected": "TRUE"},
                    {"state": "eve", "expression": "person(Eve)", "expected": "TRUE"}
                ]
            }],
            "observational_equivalence": []
        });
        assert_eq!(
            execute_scenarios(
                source.as_object().expect("source object"),
                &KbIndex::new(kb_text)
            )
            .expect("native execution succeeds"),
            ExecutionResult {
                snapshots: 2,
                pins: 3,
                controls: 1,
            }
        );
    }

    #[test]
    fn live_inputs_match_python_report_stdout_and_all_controls() {
        let context = context();
        let source_text = context.read(DEFAULT_SOURCE).expect("reviewed source");
        let kb_text = context.read(DEFAULT_KB).expect("constitution");
        let ledger_text = context.read(DEFAULT_LEDGER).expect("assertion ledger");
        let assurance_text = context.read(DEFAULT_ASSURANCE).expect("assurance source");
        let report = context.read(DEFAULT_OUTPUT).expect("reviewed report");
        let (source, control_value) =
            load_json(&source_text, DEFAULT_SOURCE).expect("source parses");
        let kb = KbIndex::new(&kb_text);
        let kb_digest = sha256(kb_text.as_bytes());
        let ledger_digest = sha256(ledger_text.as_bytes());
        let assurance_digest = sha256(assurance_text.as_bytes());
        let mut references = ReferenceResolver::new(&context);

        let query_vectors = validate_source_typed(
            &source,
            &kb,
            &kb_digest,
            &ledger_digest,
            &assurance_digest,
            &mut references,
        )
        .expect("live source validates");
        assert_eq!(query_vectors.len(), 8);
        let executable = collect_queries_typed(&source).expect("queries reconcile");
        assert_eq!(executable.len(), 15);
        assert_eq!(executable.values().map(BTreeMap::len).sum::<usize>(), 108);
        let rendered = render_typed(&source);
        assert!(
            rendered.contains(
                "Generated by the native rights-verify record-integrity-red-team refresh"
            )
        );
        assert!(rendered.contains("`./verify.sh --refresh record-integrity-red-team`"));
        assert!(rendered.contains("`./verify.sh --quick`"));
        assert!(rendered.contains("Authoritative executable check: `./verify.sh`"));
        assert!(!rendered.contains("python3 "));
        if rendered != report {
            let first = rendered
                .bytes()
                .zip(report.bytes())
                .position(|(left, right)| left != right)
                .unwrap_or_else(|| rendered.len().min(report.len()));
            let start = first.saturating_sub(100);
            let rendered_end = (first + 200).min(rendered.len());
            let report_end = (first + 200).min(report.len());
            panic!(
                "native report differs at byte {first}:\nrendered={:?}\nreviewed={:?}",
                &rendered[start..rendered_end],
                &report[start..report_end]
            );
        }
        assert_eq!(
            negative_controls(
                &control_value,
                &kb,
                &kb_digest,
                &ledger_digest,
                &assurance_digest,
                &mut references,
            )
            .expect("all watched mutations fail"),
            32
        );

        let status = check(
            &context,
            false,
            InputSnapshot {
                source_json: Some(&source_text),
                constitution: Some(&kb_text),
                assertion_ledger: Some(&ledger_text),
                assurance_source: Some(&assurance_text),
                generated_report: Some(&report),
            },
        )
        .expect("cached public API succeeds");
        assert_eq!(
            status,
            "new-book-plans/record-integrity-red-team.md is current; 32 structural negative controls pass; execution skipped"
        );
    }

    #[test]
    #[ignore = "release-sized engine parity; run explicitly with cargo test --release -- --ignored"]
    fn live_native_execution_matches_python_success_contract() {
        let context = context();
        let source_text = context.read(DEFAULT_SOURCE).expect("reviewed source");
        let kb_text = context.read(DEFAULT_KB).expect("constitution");
        let ledger_text = context.read(DEFAULT_LEDGER).expect("assertion ledger");
        let assurance_text = context.read(DEFAULT_ASSURANCE).expect("assurance source");
        let report = context.read(DEFAULT_OUTPUT).expect("reviewed report");
        let status = check(
            &context,
            true,
            InputSnapshot {
                source_json: Some(&source_text),
                constitution: Some(&kb_text),
                assertion_ledger: Some(&ledger_text),
                assurance_source: Some(&assurance_text),
                generated_report: Some(&report),
            },
        )
        .expect("live native execution succeeds");
        assert_eq!(
            status,
            "new-book-plans/record-integrity-red-team.md is current; 32 structural negative controls pass; 15 snapshots / 108 pins execute; 1 executable sabotage passes"
        );
    }
}
