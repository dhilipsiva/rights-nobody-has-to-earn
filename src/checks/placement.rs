// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native placement-exhaustiveness audit.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::{canonical_json, sha256};
use crate::pin::{LoadedSource, PinOptions, PreparedPinEngine};

pub(crate) const STEP_NAME: &str = "placement exhaustiveness";

const DEFAULT_SOURCE: &str = "new-book-plans/placement-exhaustiveness-audit.json";
const DEFAULT_KB: &str = "new-book-plans/constitution.nibli";
const DEFAULT_OUTPUT: &str = "new-book-plans/placement-exhaustiveness-audit.md";
const STATUS: &str = "bounded_current_source_repository_assurance";
const EVIDENCE_ROLE: &str = "current_verified_narrowly";
const REVIEWED_TIMEOUT_SECONDS: u64 = 180;
// The isolated and combined matrix executions were compared for this exact
// source digest during the native migration. A changed constitution falls
// back to per-case isolation until its cross-case independence is recertified.
const COMBINED_MATRIX_CERTIFIED_KB_SHA256: &str =
    "4f09cdb7320c492eba55809df337eab4a4e3a464193b355781ddc9ea04115ace";
const TARGET_RELATIONS: [&str; 3] = ["fit", "dwell", "building"];
const AXIS_ORDER: [&str; 3] = ["severe", "family", "home"];
const SUBJECT_KINDS: [&str; 3] = ["confined", "registered_free", "registered_person"];
const LIMIT_KEYS: [&str; 9] = [
    "bounded_absence_meaning",
    "current_source",
    "runtime",
    "records_and_remedy",
    "housing_delivery",
    "future_delivery",
    "scope",
    "temporal_fixture",
    "trust_root",
];
const NARROWNESS_CLASSIFICATIONS: [&str; 2] = ["preserved", "revised_and_scoped"];

const HISTORICAL_DWELL_LINE: &str =
    "all $x: prisoner($x) & fit($x, Homestay) & ~at($x, PlacementHome) -> dwell($x).\n";
const HOMESTAY_LINE: &str =
    "all $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) -> building(Homestay, $x).\n";
const REVERSED_HOMESTAY_LINE: &str =
    "all $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) -> building(HighSec, $x).\n";
const DUPLICATE_APPEND: &str = "\n# Placement-exhaustiveness duplicate-destination mutation (generated, not enacted).\nall $x: prisoner($x) & at($x, PlacementHome) & fit($x, Homestay) -> building(HighSec, $x).\n";
const PAINTED_DELIVERY_APPEND: &str = "\n# Placement-exhaustiveness painted-delivery mutation (generated, not enacted).\nall $x: person($x) -> dwell($x).\n";

type AuditResult<T> = Result<T, String>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewedSource {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    evidence_role: String,
    subprocess_timeout_seconds: u64,
    constitution_sha256: String,
    producer_fingerprints: BTreeMap<String, String>,
    destination_constants: Vec<String>,
    destination_constants_sha256: String,
    subject_contract: SubjectContract,
    axis_contract: AxisContract,
    limits: BTreeMap<String, String>,
    matrix: Vec<MatrixCase>,
    required_mutations: Vec<String>,
    mutations: Vec<MutationCase>,
    narrowness_impacts: Vec<NarrownessImpact>,
    acceptance_result: AcceptanceResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SubjectContract {
    states: Vec<String>,
    semantics: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AxisContract {
    order: Vec<String>,
    states: BTreeMap<String, Vec<String>>,
    semantics: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MatrixCase {
    id: String,
    subject_kind: String,
    axes: Axes,
    fit_homestay: String,
    dwell: String,
    destinations: Vec<String>,
    placement_err: String,
    interpretation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Axes {
    severe: String,
    family: String,
    home: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MutationCase {
    id: String,
    title: String,
    kind: String,
    mutations: Vec<SourceMutation>,
    mutation_sha256: String,
    expected_source_sha256: String,
    observations: Vec<Observation>,
    baseline_flips: Vec<BaselineFlip>,
    err_absence_case_refs: Vec<String>,
    alarm_setup_facts: BTreeMap<String, String>,
    interpretation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceMutation {
    op: String,
    before: String,
    after: String,
    before_sha256: String,
    after_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    expression: String,
    expected: String,
    purpose: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BaselineFlip {
    expression: String,
    baseline_expected: String,
    candidate_expected: String,
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
    does_not_establish: String,
    remaining_boundary: String,
}

#[derive(Clone, Debug, Default)]
struct SourceInventory {
    producers: BTreeMap<String, Vec<String>>,
    fingerprints: BTreeMap<String, String>,
    destinations: Vec<String>,
    destinations_sha256: String,
}

#[derive(Clone, Debug)]
struct Query {
    expression: String,
    expected: String,
    purpose: String,
}

#[derive(Clone, Debug)]
struct Snapshot {
    reviewed: ReviewedSource,
    source_relative: String,
    kb_relative: String,
    output_relative: String,
    output_path: PathBuf,
    input_identities: HashSet<FileIdentity>,
    kb_text: String,
    kb_digest: String,
    current_output: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ExecutionReport {
    pub(crate) base_runs: usize,
    pub(crate) base_pins: usize,
    pub(crate) composed_floor_runs: usize,
    pub(crate) composed_floor_pins: usize,
    pub(crate) candidate_runs: usize,
    pub(crate) candidate_pins: usize,
    pub(crate) sabotage_runs: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Report {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
    pub(crate) execution: Option<ExecutionReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenerationReport {
    pub(crate) output: String,
    pub(crate) structural_controls: usize,
}

impl fmt::Display for GenerationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: regenerated (structural generation; execution not requested); {} structural negative controls pass",
            self.output, self.structural_controls
        )
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is current; {} structural negative controls pass",
            self.output, self.structural_controls
        )?;
        match self.execution {
            None => formatter.write_str("; execution skipped"),
            Some(run) => write!(
                formatter,
                "; {} matrix / {} pins, {} direct composed floor probes / {} pins, \
                 {} mutation observation runs / {} pins, {} executable \
                 mutation-baseline sabotages, and 1 composed-standing-removal sabotage pass",
                run.base_runs,
                run.base_pins,
                run.composed_floor_runs,
                run.composed_floor_pins,
                run.candidate_runs,
                run.candidate_pins,
                run.sabotage_runs.saturating_sub(1),
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FingerprintReport(pub(crate) String);

#[derive(Serialize)]
struct FingerprintOutput {
    constitution_sha256: String,
    destination_constants: Vec<String>,
    destination_constants_sha256: String,
    mutations: BTreeMap<String, MutationFingerprint>,
    producer_fingerprints: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct MutationFingerprint {
    expected_source_sha256: String,
    fragments: Vec<FragmentFingerprint>,
    mutation_sha256: String,
}

#[derive(Serialize)]
struct FragmentFingerprint {
    after_sha256: String,
    before_sha256: String,
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_inner(context, false)
}

pub(crate) fn check_execute(context: &Context) -> Result<Report, Error> {
    check_inner(context, true)
}

pub(crate) fn generate(context: &Context) -> Result<GenerationReport, Error> {
    let snapshot = load_snapshot(context, false).map_err(placement_error)?;
    let inventory = validate_source(context, &snapshot).map_err(placement_error)?;
    let generated = render(&snapshot, &inventory);
    let structural_controls = negative_controls(context, &snapshot).map_err(placement_error)?;
    write_generated_output(
        &snapshot.output_path,
        generated.as_bytes(),
        &snapshot.input_identities,
    )
    .map_err(placement_error)?;
    Ok(GenerationReport {
        output: snapshot.output_relative,
        structural_controls,
    })
}

fn check_inner(context: &Context, execute: bool) -> Result<Report, Error> {
    let snapshot = load_snapshot(context, true).map_err(placement_error)?;
    let inventory = validate_source(context, &snapshot).map_err(placement_error)?;
    let generated = render(&snapshot, &inventory);
    if snapshot.current_output.as_deref() != Some(generated.as_bytes()) {
        return Err(placement_error(format!(
            "{} is STALE — rerun without --check",
            snapshot.output_relative
        )));
    }
    let structural_controls = negative_controls(context, &snapshot).map_err(placement_error)?;
    let execution = execute
        .then(|| execute_audit(&snapshot, &inventory))
        .transpose()
        .map_err(placement_error)?;
    Ok(Report {
        output: snapshot.output_relative,
        structural_controls,
        execution,
    })
}

pub(crate) fn fingerprints(context: &Context) -> Result<FingerprintReport, Error> {
    let snapshot = load_snapshot(context, false).map_err(placement_error)?;
    let inventory = source_inventory(&snapshot.kb_text).map_err(placement_error)?;
    let mut mutations = BTreeMap::new();
    for entry in &snapshot.reviewed.mutations {
        let candidate =
            apply_mutations(&snapshot.kb_text, &entry.mutations, false).map_err(placement_error)?;
        let normalized = entry
            .mutations
            .iter()
            .map(|item| {
                serde_json::json!({
                    "op": item.op,
                    "before": item.before,
                    "after": item.after,
                    "before_sha256": sha256(item.before.as_bytes()),
                    "after_sha256": sha256(item.after.as_bytes()),
                })
            })
            .collect::<Vec<_>>();
        mutations.insert(
            entry.id.clone(),
            MutationFingerprint {
                expected_source_sha256: sha256(candidate.as_bytes()),
                fragments: entry
                    .mutations
                    .iter()
                    .map(|item| FragmentFingerprint {
                        after_sha256: sha256(item.after.as_bytes()),
                        before_sha256: sha256(item.before.as_bytes()),
                    })
                    .collect(),
                mutation_sha256: sha256(canonical_json(&Value::Array(normalized))),
            },
        );
    }
    let output = FingerprintOutput {
        constitution_sha256: snapshot.kb_digest,
        destination_constants: inventory.destinations,
        destination_constants_sha256: inventory.destinations_sha256,
        mutations,
        producer_fingerprints: inventory.fingerprints,
    };
    let body = serde_json::to_string_pretty(&output)
        .map_err(|error| placement_error(error.to_string()))?;
    Ok(FingerprintReport(body))
}

fn placement_error(message: impl Into<String>) -> Error {
    Error::new(format!("11-placement-exhaustiveness: {}", message.into()))
}

fn load_snapshot(context: &Context, read_output: bool) -> AuditResult<Snapshot> {
    let source_path = context.root().join(DEFAULT_SOURCE);
    let kb_path = context.root().join(DEFAULT_KB);
    let output_path = context.root().join(DEFAULT_OUTPUT);
    validate_repo_path(context.root(), &source_path)?;
    validate_repo_path(context.root(), &kb_path)?;
    validate_repo_path(context.root(), &output_path)?;
    let (source_bytes, source_identity) = read_bound_file(&source_path, "placement audit source")?;
    let (kb_bytes, kb_identity) = read_bound_file(&kb_path, "constitution")?;
    let input_identities = require_distinct_identities(&[
        ("placement audit source", source_identity),
        ("constitution", kb_identity),
    ])?;
    validate_output_target(&output_path, &input_identities)?;
    if kb_bytes.contains(&b'\r') {
        return Err("constitution contains carriage-return bytes".into());
    }
    let kb_text = String::from_utf8(kb_bytes.clone())
        .map_err(|error| format!("constitution: invalid UTF-8: {error}"))?;
    parse_json_no_duplicates(&source_bytes)
        .map_err(|error| format!("cannot parse placement audit source: {error}"))?;
    let reviewed = serde_json::from_slice(&source_bytes)
        .map_err(|error| format!("invalid placement audit source: {error}"))?;
    let current_output = if read_output {
        Some(read_bound_file(&output_path, "generated placement report")?.0)
    } else {
        None
    };
    Ok(Snapshot {
        reviewed,
        source_relative: DEFAULT_SOURCE.into(),
        kb_relative: DEFAULT_KB.into(),
        output_relative: DEFAULT_OUTPUT.into(),
        output_path,
        input_identities,
        kb_digest: sha256(&kb_bytes),
        kb_text,
        current_output,
    })
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

fn validate_repo_path(root: &Path, path: &Path) -> AuditResult<()> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("path escapes repository: {}", path.display()))?;
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!("path escapes repository: {}", path.display()));
    }
    Ok(())
}

fn validate_source(context: &Context, snapshot: &Snapshot) -> AuditResult<SourceInventory> {
    let source = &snapshot.reviewed;
    nonempty(&source.spdx, "spdx")?;
    if source.spdx != "CC-BY-4.0" {
        return Err("spdx must be CC-BY-4.0".into());
    }
    if source.schema_version != 2 {
        return Err("schema_version must equal 2".into());
    }
    nonempty(&source.title, "title")?;
    if source.status != STATUS {
        return Err(format!("status must equal {STATUS}"));
    }
    if source.evidence_role != EVIDENCE_ROLE {
        return Err(format!("evidence_role must equal {EVIDENCE_ROLE}"));
    }
    if source.subprocess_timeout_seconds != REVIEWED_TIMEOUT_SECONDS {
        return Err(format!(
            "subprocess_timeout_seconds must equal {REVIEWED_TIMEOUT_SECONDS}"
        ));
    }
    require_sha(
        &source.constitution_sha256,
        "constitution_sha256",
        Some(&snapshot.kb_digest),
    )?;
    let inventory = source_inventory(&snapshot.kb_text)?;
    require_exact_map_keys(
        &source.producer_fingerprints,
        &TARGET_RELATIONS,
        "producer_fingerprints",
    )?;
    for relation in TARGET_RELATIONS {
        require_sha(
            &source.producer_fingerprints[relation],
            &format!("producer_fingerprints.{relation}"),
            Some(&inventory.fingerprints[relation]),
        )?;
    }
    unique_nonempty(&source.destination_constants, "destination_constants", true)?;
    if source.destination_constants != inventory.destinations {
        return Err(format!(
            "destination_constants: declared {:?}, discovered {:?}",
            source.destination_constants, inventory.destinations
        ));
    }
    require_sha(
        &source.destination_constants_sha256,
        "destination_constants_sha256",
        Some(&inventory.destinations_sha256),
    )?;
    validate_contracts(source)?;
    validate_limits(source)?;
    validate_matrix(source, &inventory)?;
    validate_mutations(source, &snapshot.kb_text, &inventory)?;
    validate_narrowness(context, source)?;
    validate_acceptance(source)?;
    Ok(inventory)
}

fn validate_contracts(source: &ReviewedSource) -> AuditResult<()> {
    let expected_subjects = SUBJECT_KINDS.map(str::to_owned).to_vec();
    if source.subject_contract.states != expected_subjects {
        return Err(format!(
            "subject_contract.states must equal {:?}",
            SUBJECT_KINDS
        ));
    }
    require_exact_map_keys(
        &source.subject_contract.semantics,
        &SUBJECT_KINDS,
        "subject_contract.semantics",
    )?;
    for kind in SUBJECT_KINDS {
        nonempty(
            &source.subject_contract.semantics[kind],
            &format!("subject_contract.semantics.{kind}"),
        )?;
    }
    if source.axis_contract.order != AXIS_ORDER.map(str::to_owned).to_vec() {
        return Err(format!("axis_contract.order must equal {AXIS_ORDER:?}"));
    }
    require_exact_map_keys(
        &source.axis_contract.states,
        &AXIS_ORDER,
        "axis_contract.states",
    )?;
    require_exact_map_keys(
        &source.axis_contract.semantics,
        &AXIS_ORDER,
        "axis_contract.semantics",
    )?;
    for axis in AXIS_ORDER {
        let required = axis_values(axis)
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<Vec<_>>();
        if source.axis_contract.states[axis] != required {
            return Err(format!(
                "axis_contract.states.{axis}: required state order changed"
            ));
        }
        nonempty(
            &source.axis_contract.semantics[axis],
            &format!("axis_contract.semantics.{axis}"),
        )?;
    }
    Ok(())
}

fn validate_limits(source: &ReviewedSource) -> AuditResult<()> {
    require_exact_map_keys(&source.limits, &LIMIT_KEYS, "limits")?;
    for key in LIMIT_KEYS {
        nonempty(&source.limits[key], &format!("limits.{key}"))?;
    }
    Ok(())
}

fn validate_matrix(source: &ReviewedSource, inventory: &SourceInventory) -> AuditResult<()> {
    let expected = all_case_keys();
    let mut seen = BTreeSet::new();
    let mut ids = HashSet::new();
    for (index, case) in source.matrix.iter().enumerate() {
        let path = format!("matrix[{index}]");
        nonempty(&case.id, &format!("{path}.id"))?;
        nonempty(&case.interpretation, &format!("{path}.interpretation"))?;
        if !ids.insert(case.id.clone()) {
            return Err(format!("{path}.id: duplicate {}", case.id));
        }
        if !SUBJECT_KINDS.contains(&case.subject_kind.as_str()) {
            return Err(format!(
                "{path}.subject_kind: unknown {:?}",
                case.subject_kind
            ));
        }
        validate_axes(&case.axes, &format!("{path}.axes"))?;
        let generated = case_id(&case.subject_kind, &case.axes);
        if case.id != generated {
            return Err(format!("{path}.id: expected generated id {generated:?}"));
        }
        let key = case_key(case);
        if !seen.insert(key.clone()) {
            return Err(format!("{path}: duplicate matrix tuple {key:?}"));
        }
        for (label, verdict) in [
            ("fit_homestay", &case.fit_homestay),
            ("dwell", &case.dwell),
            ("placement_err", &case.placement_err),
        ] {
            require_verdict(verdict, &format!("{path}.{label}"))?;
        }
        unique_nonempty(&case.destinations, &format!("{path}.destinations"), false)?;
        if case
            .destinations
            .iter()
            .any(|item| !inventory.destinations.contains(item))
        {
            return Err(format!("{path}.destinations: unknown destination"));
        }
        let (fit, dwell, destinations) = required_outcome(&case.subject_kind, &case.axes);
        if case.fit_homestay != fit || case.dwell != dwell || case.destinations != destinations {
            return Err(format!("{path}: current required outcome changed"));
        }
        if case.placement_err != "FALSE" {
            return Err(format!(
                "{path}.placement_err: current base matrix must remain FALSE"
            ));
        }
    }
    if seen != expected {
        return Err("matrix is not the exact Cartesian product".into());
    }
    Ok(())
}

fn validate_mutations(
    source: &ReviewedSource,
    kb_text: &str,
    inventory: &SourceInventory,
) -> AuditResult<()> {
    let required_ids = required_mutation_ids();
    if source.required_mutations != required_ids {
        return Err(format!("required_mutations must equal {required_ids:?}"));
    }
    let cases = source
        .matrix
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<HashMap<_, _>>();
    let base_queries = matrix_query_map(&source.matrix, inventory)?;
    let mut seen = HashSet::new();
    for (index, entry) in source.mutations.iter().enumerate() {
        let path = format!("mutations[{index}]");
        nonempty(&entry.id, &format!("{path}.id"))?;
        nonempty(&entry.title, &format!("{path}.title"))?;
        nonempty(&entry.interpretation, &format!("{path}.interpretation"))?;
        if !seen.insert(entry.id.clone()) {
            return Err(format!("{path}.id: duplicate {}", entry.id));
        }
        let (required_kind, shapes) = required_mutation_shape(&entry.id)?;
        if entry.kind != required_kind {
            return Err(format!("{path}.kind: expected {required_kind:?}"));
        }
        let actual_shapes = entry
            .mutations
            .iter()
            .map(|item| (item.op.as_str(), item.before.as_str(), item.after.as_str()))
            .collect::<Vec<_>>();
        if actual_shapes != shapes {
            return Err(format!("{path}.mutations: required exact shape changed"));
        }
        for (mutation_index, item) in entry.mutations.iter().enumerate() {
            require_sha(
                &item.before_sha256,
                &format!("{path}.mutations[{mutation_index}].before_sha256"),
                Some(&sha256(item.before.as_bytes())),
            )?;
            require_sha(
                &item.after_sha256,
                &format!("{path}.mutations[{mutation_index}].after_sha256"),
                Some(&sha256(item.after.as_bytes())),
            )?;
        }
        let payload = source_mutation_payload(&entry.mutations);
        require_sha(
            &entry.mutation_sha256,
            &format!("{path}.mutation_sha256"),
            Some(&sha256(canonical_json(&payload))),
        )?;
        let candidate = apply_mutations(kb_text, &entry.mutations, true)?;
        require_sha(
            &entry.expected_source_sha256,
            &format!("{path}.expected_source_sha256"),
            Some(&sha256(candidate.as_bytes())),
        )?;
        let mut observations = HashMap::new();
        for (observation_index, observation) in entry.observations.iter().enumerate() {
            nonempty(
                &observation.expression,
                &format!("{path}.observations[{observation_index}].expression"),
            )?;
            require_verdict(
                &observation.expected,
                &format!("{path}.observations[{observation_index}].expected"),
            )?;
            nonempty(
                &observation.purpose,
                &format!("{path}.observations[{observation_index}].purpose"),
            )?;
            if observations
                .insert(
                    observation.expression.as_str(),
                    observation.expected.as_str(),
                )
                .is_some()
            {
                return Err(format!("{path}.observations: duplicate expression"));
            }
        }
        if observations.is_empty() {
            return Err(format!("{path}.observations: must not be empty"));
        }
        let flips = entry
            .baseline_flips
            .iter()
            .map(|flip| {
                (
                    flip.expression.clone(),
                    flip.baseline_expected.clone(),
                    flip.candidate_expected.clone(),
                )
            })
            .collect::<Vec<_>>();
        let mut flip_ids = HashSet::new();
        for flip in &entry.baseline_flips {
            require_verdict(&flip.baseline_expected, "baseline_flips.baseline_expected")?;
            require_verdict(
                &flip.candidate_expected,
                "baseline_flips.candidate_expected",
            )?;
            if !flip_ids.insert(flip.expression.as_str()) {
                return Err(format!("{path}.baseline_flips: duplicate expression"));
            }
            if base_queries.get(&flip.expression) != Some(&flip.baseline_expected) {
                return Err(format!(
                    "{path}.baseline_flips: baseline expectation changed"
                ));
            }
            if flip.baseline_expected == flip.candidate_expected
                || observations.get(flip.expression.as_str())
                    != Some(&flip.candidate_expected.as_str())
            {
                return Err(format!(
                    "{path}.baseline_flips: candidate does not flip baseline"
                ));
            }
        }
        let (required_flips, required_refs) = required_mutation_contract(&entry.id)?;
        if flips != required_flips {
            return Err(format!(
                "{path}.baseline_flips: exact code-owned affected set changed"
            ));
        }
        if entry.err_absence_case_refs != required_refs {
            return Err(format!(
                "{path}.err_absence_case_refs: exact code-owned affected set changed"
            ));
        }
        unique_nonempty(&entry.err_absence_case_refs, "err_absence_case_refs", true)?;
        if entry
            .alarm_setup_facts
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != entry.err_absence_case_refs.iter().cloned().collect()
        {
            return Err(format!(
                "{path}.alarm_setup_facts: keys must equal err_absence_case_refs"
            ));
        }
        let required_destination = required_alarm_destination(&entry.id)?;
        for case_ref in &entry.err_absence_case_refs {
            let case = cases
                .get(case_ref.as_str())
                .ok_or_else(|| format!("{path}.err_absence_case_refs: unknown {case_ref}"))?;
            let subject = case_subject(&case.subject_kind, &case.axes);
            let err = format!("err({subject}, Placement)");
            if base_queries.get(&err).map(String::as_str) != Some("FALSE")
                || observations
                    .get(err.as_str())
                    .is_some_and(|value| *value != "FALSE")
            {
                return Err(format!("{path}: {err} must remain a FALSE baseline"));
            }
            let setup = &entry.alarm_setup_facts[case_ref];
            let arguments = parse_atom(setup, "put").ok_or_else(|| {
                format!(
                    "{path}.alarm_setup_facts.{case_ref}: expected put(actor, subject, destination)"
                )
            })?;
            if arguments.len() != 3
                || arguments[1] != subject
                || arguments[2] != required_destination
                || arguments.iter().any(|item| !valid_name(item))
            {
                return Err(format!(
                    "{path}.alarm_setup_facts.{case_ref}: invalid reviewed setup"
                ));
            }
            if observations
                .get(setup.as_str())
                .is_some_and(|value| *value != "TRUE")
            {
                return Err(format!(
                    "{path}.alarm_setup_facts.{case_ref}: authored observation must be TRUE"
                ));
            }
        }
    }
    if seen != required_ids.iter().cloned().collect() {
        return Err("mutations do not cover the exact required set".into());
    }
    Ok(())
}

fn validate_narrowness(context: &Context, source: &ReviewedSource) -> AuditResult<()> {
    if source.narrowness_impacts.is_empty() {
        return Err("narrowness_impacts must not be empty".into());
    }
    let mut references = HashSet::new();
    for (index, entry) in source.narrowness_impacts.iter().enumerate() {
        let path = format!("narrowness_impacts[{index}]");
        validate_reference(
            context,
            &entry.artifact_ref,
            &format!("{path}.artifact_ref"),
        )?;
        if !references.insert(entry.artifact_ref.as_str()) {
            return Err(format!("{path}.artifact_ref: duplicate"));
        }
        nonempty(&entry.current_claim, &format!("{path}.current_claim"))?;
        if !NARROWNESS_CLASSIFICATIONS.contains(&entry.classification.as_str()) {
            return Err(format!(
                "{path}.classification: unknown {}",
                entry.classification
            ));
        }
        nonempty(&entry.reason, &format!("{path}.reason"))?;
        nonempty(&entry.future_trigger, &format!("{path}.future_trigger"))?;
    }
    Ok(())
}

fn validate_acceptance(source: &ReviewedSource) -> AuditResult<()> {
    if source.acceptance_result.result != STATUS {
        return Err(format!("acceptance_result.result must equal {STATUS}"));
    }
    nonempty(&source.acceptance_result.claim, "acceptance_result.claim")?;
    nonempty(
        &source.acceptance_result.remaining_boundary,
        "acceptance_result.remaining_boundary",
    )?;
    let residual = source.acceptance_result.does_not_establish.to_lowercase();
    for term in [
        "runtime exclusivity",
        "authorship",
        "appeal",
        "remedy",
        "free-person housing delivery",
    ] {
        if !residual.contains(term) {
            return Err(format!(
                "acceptance_result.does_not_establish: missing {term:?} boundary"
            ));
        }
    }
    Ok(())
}

fn source_inventory(source: &str) -> AuditResult<SourceInventory> {
    let mut producers = TARGET_RELATIONS
        .into_iter()
        .map(|relation| (relation.to_owned(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    let mut destinations = BTreeSet::new();
    let head = Regex::new(r"^(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<args>.*)\)$")
        .expect("valid placement producer regex");
    for (statement, line) in lex_statements(source)? {
        let candidate = statement
            .rsplit_once("->")
            .map_or(statement.as_str(), |(_, value)| value.trim());
        let Some(captures) = head.captures(candidate) else {
            continue;
        };
        let relation = &captures["name"];
        let Some(entries) = producers.get_mut(relation) else {
            continue;
        };
        entries.push(statement.clone());
        if relation == "building" {
            let arguments = split_arguments(&captures["args"])?;
            if arguments.len() != 2 {
                return Err(format!(
                    "constitution line {line}: building head must have arity 2"
                ));
            }
            if !valid_name(&arguments[0]) || arguments[0].starts_with('$') {
                return Err(format!(
                    "constitution line {line}: building destination must be a reviewed literal constant"
                ));
            }
            destinations.insert(arguments[0].clone());
        }
    }
    let mut fingerprints = BTreeMap::new();
    for relation in TARGET_RELATIONS {
        let entries = producers
            .get_mut(relation)
            .expect("target relation inventory initialized");
        if entries.is_empty() {
            return Err(format!("constitution has no {relation} producer"));
        }
        entries.sort();
        if entries.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(format!(
                "constitution repeats an identical {relation} producer"
            ));
        }
        let value = serde_json::to_value(entries.as_slice()).map_err(|error| error.to_string())?;
        fingerprints.insert(relation.to_owned(), sha256(canonical_json(&value)));
    }
    let destinations = destinations.into_iter().collect::<Vec<_>>();
    let destinations_value =
        serde_json::to_value(&destinations).map_err(|error| error.to_string())?;
    let destinations_sha256 = sha256(canonical_json(&destinations_value));
    Ok(SourceInventory {
        producers,
        fingerprints,
        destinations,
        destinations_sha256,
    })
}

fn lex_statements(source: &str) -> AuditResult<Vec<(String, usize)>> {
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
                statements.push((normalized, start_line.unwrap_or(line)));
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
    if in_string {
        return Err("unterminated string in constitution".into());
    }
    let residue = buffer.trim();
    if !residue.is_empty() {
        return Err(format!(
            "unterminated active statement at line {}: {}",
            start_line.unwrap_or(line),
            residue.chars().take(80).collect::<String>()
        ));
    }
    Ok(statements)
}

fn split_arguments(value: &str) -> AuditResult<Vec<String>> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut parens = 0i32;
    let mut braces = 0i32;
    let mut brackets = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for character in value.chars() {
        if character == '"' && !escaped {
            in_string = !in_string;
        }
        if !in_string {
            match character {
                '(' => parens += 1,
                ')' => parens -= 1,
                '{' => braces += 1,
                '}' => braces -= 1,
                '[' => brackets += 1,
                ']' => brackets -= 1,
                ',' if parens == 0 && braces == 0 && brackets == 0 => {
                    let item = buffer.trim().to_owned();
                    if item.is_empty() {
                        return Err("empty argument".into());
                    }
                    result.push(item);
                    buffer.clear();
                    continue;
                }
                _ => {}
            }
            if parens < 0 || braces < 0 || brackets < 0 {
                return Err("unbalanced arguments".into());
            }
        }
        buffer.push(character);
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }
    if in_string || parens != 0 || braces != 0 || brackets != 0 {
        return Err("unbalanced arguments".into());
    }
    let item = buffer.trim().to_owned();
    if item.is_empty() {
        return Err("empty argument".into());
    }
    result.push(item);
    Ok(result)
}

fn axis_values(axis: &str) -> &'static [&'static str] {
    match axis {
        "severe" => &["not_derived", "derived"],
        "family" | "home" => &["absent", "present"],
        _ => &[],
    }
}

fn validate_axes(axes: &Axes, path: &str) -> AuditResult<()> {
    for (axis, value) in [
        ("severe", axes.severe.as_str()),
        ("family", axes.family.as_str()),
        ("home", axes.home.as_str()),
    ] {
        if !axis_values(axis).contains(&value) {
            return Err(format!("{path}.{axis}: invalid state {value:?}"));
        }
    }
    Ok(())
}

fn all_axes() -> Vec<Axes> {
    let mut result = Vec::with_capacity(8);
    for severe in axis_values("severe") {
        for family in axis_values("family") {
            for home in axis_values("home") {
                result.push(Axes {
                    severe: (*severe).into(),
                    family: (*family).into(),
                    home: (*home).into(),
                });
            }
        }
    }
    result
}

fn subject_prefix(kind: &str) -> &'static str {
    match kind {
        "confined" => "Confined",
        "registered_free" => "Free",
        "registered_person" => "Registered",
        _ => "Invalid",
    }
}

fn case_subject(kind: &str, axes: &Axes) -> String {
    format!(
        "{}_{}_{}_{}",
        subject_prefix(kind),
        if axes.severe == "derived" {
            "Severe"
        } else {
            "NotSevere"
        },
        if axes.family == "present" {
            "Family"
        } else {
            "NoFamily"
        },
        if axes.home == "present" {
            "Home"
        } else {
            "NoHome"
        },
    )
}

fn case_id(kind: &str, axes: &Axes) -> String {
    case_subject(kind, axes).to_lowercase().replace('_', "-")
}

fn case_key(case: &MatrixCase) -> String {
    format!(
        "{}|{}|{}|{}",
        case.subject_kind, case.axes.severe, case.axes.family, case.axes.home
    )
}

fn all_case_keys() -> BTreeSet<String> {
    SUBJECT_KINDS
        .into_iter()
        .flat_map(|kind| {
            all_axes()
                .into_iter()
                .map(move |axes| format!("{}|{}|{}|{}", kind, axes.severe, axes.family, axes.home))
        })
        .collect()
}

fn required_outcome(kind: &str, axes: &Axes) -> (&'static str, &'static str, Vec<String>) {
    if kind != "confined" {
        return ("FALSE", "FALSE", Vec::new());
    }
    if axes.severe == "derived" {
        return ("FALSE", "TRUE", vec!["HighSec".into()]);
    }
    if axes.home == "present" {
        return ("TRUE", "TRUE", vec!["Homestay".into()]);
    }
    ("TRUE", "TRUE", Vec::new())
}

fn case_queries(case: &MatrixCase, inventory: &SourceInventory) -> Vec<Query> {
    let subject = case_subject(&case.subject_kind, &case.axes);
    let lease = format!("Placement_Case_{subject}");
    let mut queries = vec![
        query(
            format!("person({subject})"),
            "TRUE",
            "Every generated subject has standing.",
        ),
        query(
            format!("prisoner({subject})"),
            if case.subject_kind == "confined" {
                "TRUE"
            } else {
                "FALSE"
            },
            "Confinement is the switch separating placement from the free mirror.",
        ),
        query(
            format!("correct({lease}, ActivePower)"),
            if case.subject_kind == "confined" {
                "TRUE"
            } else {
                "FALSE"
            },
            "Only confined rows receive the exact current case-bound custody lease.",
        ),
        query(
            format!("free({subject})"),
            if case.subject_kind == "registered_free" {
                "TRUE"
            } else {
                "FALSE"
            },
            "Affirmative freedom is distinguished from both confinement and personhood alone.",
        ),
        query(
            format!("severe({subject})"),
            if case.axes.severe == "derived" {
                "TRUE"
            } else {
                "FALSE"
            },
            "The generated severity setup reaches the declared axis state.",
        ),
        query(
            format!("family({subject})"),
            if case.axes.family == "present" {
                "TRUE"
            } else {
                "FALSE"
            },
            "The family axis matches the generated fixture record.",
        ),
        query(
            format!("at({subject}, PlacementHome)"),
            if case.axes.home == "present" {
                "TRUE"
            } else {
                "FALSE"
            },
            "The placement-home availability axis matches the generated fixture record.",
        ),
        query(
            format!("owe(State, Dwell, {subject})"),
            "TRUE",
            "The itemised shelter debt survives in confined and free rows.",
        ),
        query(
            format!("fit({subject}, Homestay)"),
            &case.fit_homestay,
            "Home-confinement eligibility matches the reviewed row.",
        ),
        query(
            format!("dwell({subject})"),
            &case.dwell,
            "Housing actuality matches the reviewed row.",
        ),
    ];
    for destination in &inventory.destinations {
        queries.push(query(
            format!("building({destination}, {subject})"),
            if case.destinations.contains(destination) {
                "TRUE"
            } else {
                "FALSE"
            },
            "Every discovered destination is queried, including every opposite outcome.",
        ));
    }
    queries.push(query(
        format!("err({subject}, Placement)"),
        &case.placement_err,
        "The current placement alarm remains silent in the accepted matrix.",
    ));
    queries
}

fn query(expression: String, expected: &str, purpose: &str) -> Query {
    Query {
        expression,
        expected: expected.into(),
        purpose: purpose.into(),
    }
}

fn matrix_query_map(
    cases: &[MatrixCase],
    inventory: &SourceInventory,
) -> AuditResult<HashMap<String, String>> {
    let mut result = HashMap::new();
    for case in cases {
        for query in case_queries(case, inventory) {
            if let Some(prior) = result.insert(query.expression.clone(), query.expected.clone()) {
                if prior != query.expected {
                    return Err(format!("matrix query conflict for {}", query.expression));
                }
            }
        }
    }
    Ok(result)
}

fn required_mutation_ids() -> Vec<String> {
    [
        "duplicate-destination",
        "historical-missing-dwell",
        "missing-required-destination",
        "opposite-destination",
        "painted-free-person-delivery",
    ]
    .map(str::to_owned)
    .to_vec()
}

fn required_mutation_shape(
    identifier: &str,
) -> AuditResult<(
    &'static str,
    Vec<(&'static str, &'static str, &'static str)>,
)> {
    match identifier {
        "duplicate-destination" => Ok((
            "duplicate_destination",
            vec![("append_exact", "", DUPLICATE_APPEND)],
        )),
        "historical-missing-dwell" => Ok((
            "historical_missing_dwell",
            vec![("delete_exact", HISTORICAL_DWELL_LINE, "")],
        )),
        "missing-required-destination" => Ok((
            "missing_required_destination",
            vec![("delete_exact", HOMESTAY_LINE, "")],
        )),
        "opposite-destination" => Ok((
            "opposite_destination",
            vec![("replace_exact", HOMESTAY_LINE, REVERSED_HOMESTAY_LINE)],
        )),
        "painted-free-person-delivery" => Ok((
            "painted_free_person_delivery",
            vec![("append_exact", "", PAINTED_DELIVERY_APPEND)],
        )),
        _ => Err(format!("unknown mutation {identifier:?}")),
    }
}

fn required_alarm_destination(identifier: &str) -> AuditResult<&'static str> {
    match identifier {
        "duplicate-destination" | "historical-missing-dwell" | "missing-required-destination" => {
            Ok("Homestay")
        }
        "opposite-destination" => Ok("HighSec"),
        "painted-free-person-delivery" => Ok("LowSec"),
        _ => Err(format!("unknown mutation {identifier:?}")),
    }
}

fn required_mutation_contract(
    identifier: &str,
) -> AuditResult<(Vec<(String, String, String)>, Vec<String>)> {
    match identifier {
        "duplicate-destination" => Ok((
            vec![(
                "building(HighSec, Confined_NotSevere_NoFamily_Home)".into(),
                "FALSE".into(),
                "TRUE".into(),
            )],
            vec!["confined-notsevere-nofamily-home".into()],
        )),
        "historical-missing-dwell" => Ok((
            vec![(
                "dwell(Confined_NotSevere_NoFamily_NoHome)".into(),
                "TRUE".into(),
                "FALSE".into(),
            )],
            vec!["confined-notsevere-nofamily-nohome".into()],
        )),
        "missing-required-destination" | "opposite-destination" => {
            let subjects = [
                "Confined_NotSevere_NoFamily_Home",
                "Confined_NotSevere_Family_Home",
            ];
            let mut flips = Vec::new();
            for subject in subjects {
                flips.push((
                    format!("building(Homestay, {subject})"),
                    "TRUE".into(),
                    "FALSE".into(),
                ));
                if identifier == "opposite-destination" {
                    flips.push((
                        format!("building(HighSec, {subject})"),
                        "FALSE".into(),
                        "TRUE".into(),
                    ));
                }
            }
            Ok((
                flips,
                vec![
                    "confined-notsevere-nofamily-home".into(),
                    "confined-notsevere-family-home".into(),
                ],
            ))
        }
        "painted-free-person-delivery" => {
            let mut flips = Vec::new();
            let mut refs = Vec::new();
            for kind in ["registered_free", "registered_person"] {
                for axes in all_axes() {
                    flips.push((
                        format!("dwell({})", case_subject(kind, &axes)),
                        "FALSE".into(),
                        "TRUE".into(),
                    ));
                    refs.push(case_id(kind, &axes));
                }
            }
            Ok((flips, refs))
        }
        _ => Err(format!(
            "no code-owned mutation contract for {identifier:?}"
        )),
    }
}

fn source_mutation_payload(mutations: &[SourceMutation]) -> Value {
    Value::Array(
        mutations
            .iter()
            .map(|item| {
                serde_json::json!({
                    "op": item.op,
                    "before": item.before,
                    "after": item.after,
                    "before_sha256": item.before_sha256,
                    "after_sha256": item.after_sha256,
                })
            })
            .collect(),
    )
}

fn apply_mutations(
    source: &str,
    mutations: &[SourceMutation],
    validate_hashes: bool,
) -> AuditResult<String> {
    let mut result = source.to_owned();
    for (index, mutation) in mutations.iter().enumerate() {
        if validate_hashes {
            require_sha(
                &mutation.before_sha256,
                &format!("mutation[{index}].before_sha256"),
                Some(&sha256(mutation.before.as_bytes())),
            )?;
            require_sha(
                &mutation.after_sha256,
                &format!("mutation[{index}].after_sha256"),
                Some(&sha256(mutation.after.as_bytes())),
            )?;
        }
        match mutation.op.as_str() {
            "append_exact" => {
                if !mutation.before.is_empty() {
                    return Err(format!("mutation[{index}]: append before must be empty"));
                }
                result.push_str(&mutation.after);
            }
            "delete_exact" | "replace_exact" => {
                if mutation.before.is_empty() {
                    return Err(format!("mutation[{index}]: exact mutation needs before"));
                }
                if result.matches(&mutation.before).count() != 1 {
                    return Err(format!(
                        "mutation[{index}]: before fragment must occur exactly once"
                    ));
                }
                if mutation.op == "delete_exact" && !mutation.after.is_empty() {
                    return Err(format!("mutation[{index}]: delete after must be empty"));
                }
                result = result.replacen(
                    &mutation.before,
                    if mutation.op == "delete_exact" {
                        ""
                    } else {
                        &mutation.after
                    },
                    1,
                );
            }
            _ => return Err(format!("mutation[{index}].op: unknown {:?}", mutation.op)),
        }
    }
    Ok(result)
}

fn parse_atom(value: &str, expected_relation: &str) -> Option<Vec<String>> {
    let open = value.find('(')?;
    if &value[..open] != expected_relation || !value.ends_with(')') {
        return None;
    }
    split_arguments(&value[open + 1..value.len() - 1]).ok()
}

fn valid_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn nonempty(value: &str, path: &str) -> AuditResult<()> {
    if value.trim().is_empty() {
        Err(format!("{path}: expected non-empty text"))
    } else {
        Ok(())
    }
}

fn unique_nonempty(values: &[String], path: &str, nonempty_list: bool) -> AuditResult<()> {
    if nonempty_list && values.is_empty() {
        return Err(format!("{path}: must not be empty"));
    }
    let mut seen = HashSet::new();
    for value in values {
        nonempty(value, path)?;
        if !seen.insert(value) {
            return Err(format!("{path}: duplicate values"));
        }
    }
    Ok(())
}

fn require_verdict(value: &str, path: &str) -> AuditResult<()> {
    if matches!(value, "TRUE" | "FALSE") {
        Ok(())
    } else {
        Err(format!("{path}: outcomes must be TRUE or FALSE"))
    }
}

fn require_sha(value: &str, path: &str, expected: Option<&str>) -> AuditResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{path}: expected lowercase SHA-256"));
    }
    if expected.is_some_and(|expected| value != expected) {
        return Err(format!(
            "{path}: stale; declared {value}, actual {}",
            expected.unwrap()
        ));
    }
    Ok(())
}

fn require_exact_map_keys<V>(
    value: &BTreeMap<String, V>,
    expected: &[&str],
    path: &str,
) -> AuditResult<()> {
    let actual = value.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("{path}: exact key contract changed"));
    }
    Ok(())
}

fn validate_reference(context: &Context, reference: &str, path: &str) -> AuditResult<()> {
    nonempty(reference, path)?;
    let (file, needle) = reference
        .split_once("::")
        .ok_or_else(|| format!("{path}: expected path::stable text"))?;
    let target = context.root().join(file.trim());
    validate_repo_path(context.root(), &target)?;
    let stable_text = needle.trim();
    if stable_text.is_empty() {
        return Err(format!("{path}: stable text must not be empty"));
    }
    let content = std::fs::read_to_string(&target)
        .map_err(|error| format!("{path}: cannot read {}: {error}", target.display()))?;
    let count = content.matches(stable_text).count();
    if count != 1 {
        return Err(format!(
            "{path}: stable text occurs {count} times in {}, expected exactly once",
            file.trim()
        ));
    }
    Ok(())
}

fn markdown(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn code(value: &str) -> String {
    let fence = if value.contains('`') { "``" } else { "`" };
    format!("{fence}{value}{fence}")
}

fn render(snapshot: &Snapshot, inventory: &SourceInventory) -> String {
    let source = &snapshot.reviewed;
    let mut cases = source.matrix.iter().collect::<Vec<_>>();
    cases.sort_by_key(|case| case.id.as_str());
    let mut mutations = source.mutations.iter().collect::<Vec<_>>();
    let required_ids = required_mutation_ids();
    mutations.sort_by_key(|entry| {
        required_ids
            .iter()
            .position(|identifier| identifier == &entry.id)
            .unwrap_or(usize::MAX)
    });
    let mut lines = vec![
        format!("<!-- SPDX-License-Identifier: {} -->", source.spdx),
        "<!-- Generated by the native rights-verify placement refresh; do not edit. -->".into(),
        String::new(),
        format!("# {}", source.title),
        String::new(),
        "## Verdict and scope".into(),
        String::new(),
        "**BOUNDED CURRENT-SOURCE REPOSITORY ASSURANCE — not a runtime placement guarantee or housing-delivery assurance.**".into(),
        String::new(),
        markdown(&source.acceptance_result.claim),
        String::new(),
        "The accepted matrix is exhaustive for the declared current axes and the exact".into(),
        "current producer surface. `FALSE` below means *not derivable from the supplied".into(),
        "bounded fixture*, not classical negation or an independently established fact.".into(),
        String::new(),
        "## Bound source manifest".into(),
        String::new(),
        format!("- Reviewed source: {}.", code(&snapshot.source_relative)),
        format!(
            "- Constitution: {} at SHA-256 {}.",
            code(&snapshot.kb_relative),
            code(&source.constitution_sha256)
        ),
        format!(
            "- Destination manifest: {}.",
            inventory
                .destinations
                .iter()
                .map(|value| code(value))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "- Destination-manifest SHA-256: {}.",
            code(&inventory.destinations_sha256)
        ),
        String::new(),
        "| produced relation | reviewed producer fingerprint | active producers |".into(),
        "| --- | --- | ---: |".into(),
    ];
    for relation in TARGET_RELATIONS {
        lines.push(format!(
            "| {} | {} | {} |",
            code(relation),
            code(&inventory.fingerprints[relation]),
            inventory.producers[relation].len()
        ));
    }
    lines.extend([
        String::new(),
        "### Active producer statements".into(),
        String::new(),
    ]);
    for relation in TARGET_RELATIONS {
        lines.extend([format!("#### {}", code(relation)), String::new()]);
        for rule in &inventory.producers[relation] {
            lines.push(format!("- {}", code(rule)));
        }
    }
    lines.extend([
        String::new(),
        "## Subject-status contract".into(),
        String::new(),
    ]);
    for kind in SUBJECT_KINDS {
        lines.push(format!(
            "- **{kind}:** {}",
            markdown(&source.subject_contract.semantics[kind])
        ));
    }
    lines.extend([String::new(), "## Axis contract".into(), String::new()]);
    for axis in AXIS_ORDER {
        lines.push(format!(
            "- **{axis}:** {}. {}",
            axis_values(axis)
                .iter()
                .map(|value| code(value))
                .collect::<Vec<_>>()
                .join(" / "),
            markdown(&source.axis_contract.semantics[axis])
        ));
    }
    for (kind, heading) in [
        ("confined", "Confined matrix"),
        ("registered_free", "Affirmatively free mirror"),
        ("registered_person", "Person-only mirror"),
    ] {
        lines.extend([
            String::new(),
            format!("## {heading}"),
            String::new(),
            "| case | severe | family | home | fit Homestay | dwell | destinations | placement err |".into(),
            "| --- | --- | --- | --- | --- | --- | --- | --- |".into(),
        ]);
        for case in cases.iter().filter(|case| case.subject_kind == kind) {
            let destinations = if case.destinations.is_empty() {
                "—".into()
            } else {
                case.destinations
                    .iter()
                    .map(|value| code(value))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                code(&case.id),
                code(&case.axes.severe),
                code(&case.axes.family),
                code(&case.axes.home),
                code(&case.fit_homestay),
                code(&case.dwell),
                destinations,
                code(&case.placement_err),
            ));
        }
        lines.extend([
            String::new(),
            "Every row also checks standing, affirmative freedom or confinement,".into(),
            "the presence or absence of an exact case-bound custody lease,".into(),
            "the itemised shelter debt, each axis result, and every".into(),
            "discovered non-selected destination.".into(),
            "A fresh one-pin probe checks the same subject's exact opaque shelter".into(),
            "entitlement against that row's full generated candidate, including".into(),
            "its actual standing route; it supplies no standing overlay and".into(),
            "extracts no floor rules.".into(),
        ]);
    }
    lines.extend([
        String::new(),
        "The two non-confined mirrors are current-source narrowness tripwires. They".into(),
        "record the present gap between entitlement, itemised debt, and actuality.".into(),
        "They are not a".into(),
        "permanent ban on a future valid free-person delivery route.".into(),
        String::new(),
        "## Executable source mutations".into(),
        String::new(),
        "Each candidate is an exact temporary source edit. Its harmful observations".into(),
        "must pass, while every listed baseline matrix expectation must fail. The".into(),
        "candidate also asks the current placement alarm about".into(),
        "every affected subject and requires it to remain silent.".into(),
        String::new(),
        "| mutation | kind | baseline flips | alarm-silence cases | candidate source SHA-256 |"
            .into(),
        "| --- | --- | ---: | ---: | --- |".into(),
    ]);
    for entry in &mutations {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code(&entry.id),
            code(&entry.kind),
            entry.baseline_flips.len(),
            entry.err_absence_case_refs.len(),
            code(&entry.expected_source_sha256),
        ));
    }
    for entry in &mutations {
        lines.extend([
            String::new(),
            format!("### {} — {}", entry.id, entry.title),
            String::new(),
            markdown(&entry.interpretation),
            String::new(),
            format!("- Mutation fingerprint: {}.", code(&entry.mutation_sha256)),
            "- Baseline acceptance flips:".into(),
        ]);
        for flip in &entry.baseline_flips {
            lines.push(format!(
                "  - {}: {} → {}.",
                code(&flip.expression),
                code(&flip.baseline_expected),
                code(&flip.candidate_expected)
            ));
        }
        lines.push("- Candidate observations:".into());
        for observation in &entry.observations {
            lines.push(format!(
                "  - {} = {}: {}",
                code(&observation.expression),
                code(&observation.expected),
                markdown(&observation.purpose)
            ));
        }
        lines.push(format!(
            "- Placement-alarm silence checked for: {}.",
            entry
                .err_absence_case_refs
                .iter()
                .map(|value| code(value))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        lines.push("- Positive placement-report probes:".into());
        for case_ref in &entry.err_absence_case_refs {
            lines.push(format!("  - {}.", code(&entry.alarm_setup_facts[case_ref])));
        }
    }
    lines.extend([String::new(), "## Limits".into(), String::new()]);
    let mut limit_keys = LIMIT_KEYS;
    limit_keys.sort();
    for key in limit_keys {
        lines.push(format!(
            "- **{}:** {}",
            title_case(&key.replace('_', " ")),
            markdown(&source.limits[key])
        ));
    }
    lines.extend([String::new(), "## Narrowness impacts".into(), String::new()]);
    for entry in &source.narrowness_impacts {
        lines.extend([
            format!("### {}", code(&entry.artifact_ref)),
            String::new(),
            format!("- **Current claim:** {}", markdown(&entry.current_claim)),
            format!("- **Classification:** {}.", code(&entry.classification)),
            format!("- **Reason:** {}", markdown(&entry.reason)),
            format!("- **Future trigger:** {}", markdown(&entry.future_trigger)),
            String::new(),
        ]);
    }
    lines.extend([
        "## Reproduce".into(),
        String::new(),
        "```bash".into(),
        "./verify.sh --refresh placement-exhaustiveness".into(),
        "./verify.sh --quick".into(),
        "./verify.sh".into(),
        "```".into(),
        String::new(),
        markdown(&source.acceptance_result.does_not_establish),
        String::new(),
        format!(
            "**Remaining boundary:** {}",
            markdown(&source.acceptance_result.remaining_boundary)
        ),
        String::new(),
    ]);
    lines.join("\n")
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn independently_observed(item: &str, evidence: &str, scope: &str) -> [String; 2] {
    [
        format!("observe(Chronicle, {item}, {evidence}, {scope})."),
        format!("observe(TemporalReview, {item}, {evidence}, {scope})."),
    ]
}

fn temporal_fixture_facts() -> Vec<String> {
    let mut facts = vec![
        "replace(Epoch_Current, Epoch_Previous, Chronicle).".into(),
        "list(Manifest_Previous, Epoch_Previous, ManifestOrder, Chronicle).".into(),
        "list(Manifest_Current, Epoch_Current, ManifestOrder, Chronicle).".into(),
        "passport(Binding_Chronicle, ChronicleLineage, Constitution_Temporal, Epoch_Current)."
            .into(),
    ];
    for (item, evidence, scope) in [
        (
            "Binding_Chronicle",
            "ChronicleLineage",
            "LineageFamilyScope",
        ),
        (
            "Binding_Chronicle",
            "Constitution_Temporal",
            "LineageVersionScope",
        ),
        ("Binding_Chronicle", "Epoch_Current", "LineageEpochScope"),
        ("Manifest_Previous", "Epoch_Previous", "ManifestScope"),
        ("Manifest_Current", "Epoch_Current", "ManifestScope"),
    ] {
        facts.extend(independently_observed(item, evidence, scope));
    }
    facts.push("list(Order_Court_A, Epoch_Current, Epoch_Review, EventSequence).".into());
    facts.extend(independently_observed(
        "Order_Court_A",
        "Epoch_Current",
        "EventStartScope",
    ));
    facts.extend(independently_observed(
        "Order_Court_A",
        "Epoch_Review",
        "EventEndScope",
    ));
    facts.push("list(Order_Log_Court_A, Epoch_Current, Epoch_Review, RecordSequence).".into());
    facts.extend(independently_observed(
        "Order_Log_Court_A",
        "Epoch_Current",
        "RecordStartScope",
    ));
    facts.extend(independently_observed(
        "Order_Log_Court_A",
        "Epoch_Review",
        "RecordEndScope",
    ));
    facts.push("date(Window_Custody, Epoch_Current, Epoch_Review, TimeService).".into());
    facts.extend(independently_observed(
        "Window_Custody",
        "Epoch_Current",
        "WindowStartScope",
    ));
    facts.extend(independently_observed(
        "Window_Custody",
        "Epoch_Review",
        "WindowEndScope",
    ));
    facts.push(
        "passport(Binding_Temporal, TemporalLeaseFamily, Constitution_Temporal, Epoch_Current)."
            .into(),
    );
    facts.extend(independently_observed(
        "Binding_Temporal",
        "TemporalLeaseFamily",
        "SourceFamilyScope",
    ));
    facts.extend(independently_observed(
        "Binding_Temporal",
        "Constitution_Temporal",
        "SourceVersionScope",
    ));
    facts.extend(independently_observed(
        "Binding_Temporal",
        "Epoch_Current",
        "SourceEpochScope",
    ));
    facts
}

fn prisoner_fixture_facts(subject: &str) -> Vec<String> {
    let victim = format!("Victim_{subject}");
    let case = format!("Placement_Case_{subject}");
    let mut facts = vec![
        format!("injure({subject}, {victim})."),
        format!("judge(Court, {subject})."),
        format!("cite(Court, {case}, {subject})."),
    ];
    facts.extend(independently_observed(&case, subject, "CaseScope"));
    facts.extend(independently_observed(&case, "Court", "HolderScope"));
    facts.extend(independently_observed(
        &case,
        "CourtJudgment",
        "JudgmentScope",
    ));
    facts.extend(independently_observed(&case, &victim, "InjuryVictimScope"));
    facts.push(format!("authorized({case}, ActiveCustody, {case})."));
    facts.extend(independently_observed(&case, "ActiveCustody", "PowerScope"));
    facts.extend(independently_observed(&case, &case, "CaseBindingScope"));
    facts.push(format!("limit({case}, {case}, Window_Custody)."));
    facts.extend(independently_observed(
        &case,
        "Window_Custody",
        "LimitScope",
    ));
    facts.push(format!("continue({case}, Epoch_Current)."));
    facts.extend(independently_observed(
        &case,
        "Epoch_Current",
        "RenewalScope",
    ));
    facts.push(format!("endorses(Electorate, {case})."));
    facts.push(format!("endorses(TemporalReview, {case})."));
    facts
}

fn matrix_fact_lines(cases: &[&MatrixCase]) -> Vec<String> {
    let mut facts = Vec::new();
    if cases.iter().any(|case| case.subject_kind == "confined") {
        facts.extend(temporal_fixture_facts());
    }
    for case in cases {
        let subject = case_subject(&case.subject_kind, &case.axes);
        let victim = format!("Victim_{subject}");
        match case.subject_kind.as_str() {
            "confined" => facts.extend(prisoner_fixture_facts(&subject)),
            "registered_free" => facts.push(format!("free({subject}).")),
            _ => facts.push(format!("person({subject}).")),
        }
        if case.axes.severe == "derived" {
            facts.push(format!("attack({subject}, {victim})."));
            facts.push(format!("cruel({subject}, {victim})."));
        }
        if case.axes.family == "present" {
            facts.push(format!("family({subject})."));
        }
        if case.axes.home == "present" {
            facts.push(format!("at({subject}, PlacementHome)."));
        }
    }
    facts
}

fn matrix_pin_lines(cases: &[&MatrixCase], inventory: &SourceInventory) -> Vec<String> {
    let count = cases
        .iter()
        .map(|case| case_queries(case, inventory).len())
        .sum::<usize>();
    let mut lines = vec![
        format!(":expect-pins {count}"),
        "# Generated placement-exhaustiveness matrix pins.".into(),
        "# These ephemeral pins are outside chapter pin-count reconciliation.".into(),
        String::new(),
    ];
    for case in cases {
        lines.push(format!("# {}", case.id));
        for query in case_queries(case, inventory) {
            lines.extend([
                format!("# {}", query.purpose),
                format!("? {}.", query.expression),
                format!("# => {}", query.expected),
                String::new(),
            ]);
        }
    }
    lines
}

fn composed_floor_pin_lines(case: &MatrixCase) -> Vec<String> {
    let subject = case_subject(&case.subject_kind, &case.axes);
    vec![
        ":expect-pins 1".into(),
        "# Full-source floor projection from the generated matrix subject's actual standing route."
            .into(),
        "# No standing overlay or extracted floor source is supplied.".into(),
        format!("? entitled({subject}, event {{ dwell() }})."),
        "# => TRUE".into(),
        String::new(),
    ]
}

fn validate_composed_floor_pin_lines(case: &MatrixCase, lines: &[String]) -> AuditResult<()> {
    let subject = case_subject(&case.subject_kind, &case.axes);
    let active = lines
        .iter()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::as_str)
        .collect::<Vec<_>>();
    let query = format!("? entitled({subject}, event {{ dwell() }}).");
    if active != [":expect-pins 1", query.as_str()]
        || lines
            .iter()
            .filter(|line| line.as_str() == "# => TRUE")
            .count()
            != 1
    {
        return Err(format!(
            "matrix.{}: composed floor probe must be query-only",
            case.id
        ));
    }
    Ok(())
}

fn mutation_observation_lines(
    entry: &MutationCase,
    cases: &HashMap<&str, &MatrixCase>,
) -> AuditResult<Vec<String>> {
    let mut queries = entry
        .observations
        .iter()
        .map(|item| Query {
            expression: item.expression.clone(),
            expected: item.expected.clone(),
            purpose: item.purpose.clone(),
        })
        .collect::<Vec<_>>();
    let mut known = queries
        .iter()
        .map(|query| query.expression.clone())
        .collect::<HashSet<_>>();
    for case_ref in &entry.err_absence_case_refs {
        let case = cases
            .get(case_ref.as_str())
            .ok_or_else(|| format!("unknown case {case_ref}"))?;
        let subject = case_subject(&case.subject_kind, &case.axes);
        let setup = entry.alarm_setup_facts[case_ref].clone();
        if known.insert(setup.clone()) {
            queries.push(query(
                setup,
                "TRUE",
                "A positive reviewed placement report makes the alarm-silence probe non-vacuous.",
            ));
        }
        let expression = format!("err({subject}, Placement)");
        if known.insert(expression.clone()) {
            queries.push(query(
                expression,
                "FALSE",
                "The constitutional placement alarm remains silent in this harmful candidate.",
            ));
        }
    }
    let mut lines = vec![
        format!(":expect-pins {}", queries.len()),
        format!("# Generated observations for mutation {}.", entry.id),
        "# The source change is constructed by this audit and is never enacted.".into(),
        String::new(),
    ];
    for query in queries {
        lines.extend([
            format!("# {}", query.purpose),
            format!("? {}.", query.expression),
            format!("# => {}", query.expected),
            String::new(),
        ]);
    }
    Ok(lines)
}

fn mutation_baseline_lines(entry: &MutationCase) -> Vec<String> {
    let mut lines = vec![
        format!(":expect-pins {}", entry.baseline_flips.len()),
        format!(
            "# Baseline acceptance expectations for mutation {}.",
            entry.id
        ),
        "# Every query must become a finding against the harmful candidate.".into(),
        String::new(),
    ];
    for flip in &entry.baseline_flips {
        lines.extend([
            format!("? {}.", flip.expression),
            format!("# => {}", flip.baseline_expected),
            String::new(),
        ]);
    }
    lines
}

fn active_mutation_lines(entry: &MutationCase) -> (Vec<&str>, Vec<&str>) {
    let mut deletions = Vec::new();
    let mut additions = Vec::new();
    for mutation in &entry.mutations {
        if matches!(mutation.op.as_str(), "delete_exact" | "replace_exact") {
            deletions.extend(
                mutation
                    .before
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#')),
            );
        }
        if matches!(mutation.op.as_str(), "append_exact" | "replace_exact") {
            additions.extend(
                mutation
                    .after
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#')),
            );
        }
    }
    (deletions, additions)
}

fn run_with_bound<T>(label: &str, function: impl FnOnce() -> AuditResult<T>) -> AuditResult<T> {
    let started = Instant::now();
    let result = function()?;
    if started.elapsed() > Duration::from_secs(REVIEWED_TIMEOUT_SECONDS) {
        return Err(format!(
            "{label}: in-process execution exceeded reviewed {REVIEWED_TIMEOUT_SECONDS}-second bound"
        ));
    }
    Ok(result)
}

fn execute_audit(snapshot: &Snapshot, inventory: &SourceInventory) -> AuditResult<ExecutionReport> {
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(4)
        .min(snapshot.reviewed.matrix.len())
        .max(1);
    let reports = std::thread::scope(|scope| {
        let handles = (0..workers)
            .map(|worker| scope.spawn(move || execute_worker(snapshot, inventory, worker, workers)))
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "placement execution worker panicked".to_owned())?
            })
            .collect::<AuditResult<Vec<_>>>()
    })?;
    Ok(reports
        .into_iter()
        .fold(ExecutionReport::default(), |mut total, report| {
            total.base_runs += report.base_runs;
            total.base_pins += report.base_pins;
            total.composed_floor_runs += report.composed_floor_runs;
            total.composed_floor_pins += report.composed_floor_pins;
            total.candidate_runs += report.candidate_runs;
            total.candidate_pins += report.candidate_pins;
            total.sabotage_runs += report.sabotage_runs;
            total
        }))
}

fn execute_worker(
    snapshot: &Snapshot,
    inventory: &SourceInventory,
    worker: usize,
    workers: usize,
) -> AuditResult<ExecutionReport> {
    let prepared =
        PreparedPinEngine::new(&[LoadedSource::new(&snapshot.kb_relative, &snapshot.kb_text)]);
    let mut base_pins = 0usize;
    let mut floor_pins = 0usize;
    let mut base_runs = 0usize;
    if snapshot.kb_digest == COMBINED_MATRIX_CERTIFIED_KB_SHA256 {
        if worker == 0 {
            (base_pins, floor_pins) = run_combined_matrix(&prepared, snapshot, inventory)?;
            base_runs = snapshot.reviewed.matrix.len();
        }
    } else {
        for case in snapshot
            .reviewed
            .matrix
            .iter()
            .enumerate()
            .filter(|(index, _)| index % workers == worker)
            .map(|(_, case)| case)
        {
            let facts = matrix_fact_lines(&[case]);
            let additions = facts.iter().map(String::as_str).collect::<Vec<_>>();
            let matrix_text = matrix_pin_lines(&[case], inventory).join("\n");
            let floor_lines = composed_floor_pin_lines(case);
            validate_composed_floor_pin_lines(case, &floor_lines)?;
            let floor_text = floor_lines.join("\n");
            let result = run_with_bound(&format!("base matrix {}", case.id), || {
                Ok(prepared.run_patched_files(
                    &[],
                    &additions,
                    &[
                        LoadedSource::new("matrix.pins.nibli", &matrix_text),
                        LoadedSource::new("entitlement.pins.nibli", &floor_text),
                    ],
                    PinOptions::default(),
                ))
            })?;
            if result.exit_code != 0 || result.files.len() != 2 {
                return Err(format!(
                    "base matrix {} failed\n{}{}",
                    case.id, result.stdout, result.stderr
                ));
            }
            let expected = case_queries(case, inventory).len();
            if result.files[0].pins != expected
                || result.files[0].findings != 0
                || result.files[0].harness != 0
                || result.files[1].pins != 1
                || result.files[1].findings != 0
                || result.files[1].harness != 0
            {
                return Err(format!(
                    "base matrix {} returned unexpected pin counts",
                    case.id
                ));
            }
            base_pins += result.files[0].pins;
            floor_pins += result.files[1].pins;
            base_runs += 1;
        }
    }

    let cases = snapshot
        .reviewed
        .matrix
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<HashMap<_, _>>();
    if worker == 0 {
        let standing = cases["confined-notsevere-nofamily-home"];
        let subject = case_subject(&standing.subject_kind, &standing.axes);
        let omitted = format!("judge(Court, {subject}).");
        let standing_facts = matrix_fact_lines(&[standing]);
        if standing_facts
            .iter()
            .filter(|line| *line == &omitted)
            .count()
            != 1
        {
            return Err("composed-standing sabotage requires one exact Court judgment fact".into());
        }
        let standing_additions = standing_facts
            .iter()
            .filter(|line| *line != &omitted)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let standing_pin = composed_floor_pin_lines(standing).join("\n");
        let standing_result = run_with_bound("composed-standing-removal sabotage", || {
            Ok(prepared.run_patched_files(
                &[],
                &standing_additions,
                &[LoadedSource::new("entitlement.pins.nibli", &standing_pin)],
                PinOptions::default(),
            ))
        })?;
        require_findings(&standing_result, 1, "composed-standing-removal sabotage")?;
    }

    let mut candidate_pins = 0usize;
    let mut candidate_runs = 0usize;
    for entry in snapshot
        .reviewed
        .mutations
        .iter()
        .enumerate()
        .filter(|(index, _)| index % workers == worker)
        .map(|(_, entry)| entry)
    {
        let selected = entry
            .err_absence_case_refs
            .iter()
            .map(|identifier| cases[identifier.as_str()])
            .collect::<Vec<_>>();
        let facts = matrix_fact_lines(&selected);
        let (deletions, mut source_additions) = active_mutation_lines(entry);
        let mut owned_additions = facts;
        owned_additions.extend(
            entry
                .err_absence_case_refs
                .iter()
                .map(|case_ref| format!("{}.", entry.alarm_setup_facts[case_ref])),
        );
        let fact_additions = owned_additions
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        source_additions.extend(fact_additions);
        let observations = mutation_observation_lines(entry, &cases)?;
        let observation_count = observations
            .first()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or_else(|| format!("{} observation count is invalid", entry.id))?;
        let observation_text = observations.join("\n");
        let baseline_text = mutation_baseline_lines(entry).join("\n");
        let result = run_with_bound(&format!("{} candidate", entry.id), || {
            Ok(prepared.run_patched_files(
                &deletions,
                &source_additions,
                &[
                    LoadedSource::new("observations.pins.nibli", &observation_text),
                    LoadedSource::new("baseline-acceptance.pins.nibli", &baseline_text),
                ],
                PinOptions::default(),
            ))
        })?;
        if result.files.len() != 2
            || result.files[0].pins != observation_count
            || result.files[0].findings != 0
            || result.files[0].harness != 0
            || result.files[1].pins != entry.baseline_flips.len()
            || result.files[1].findings != entry.baseline_flips.len()
            || result.files[1].harness != 0
        {
            return Err(format!(
                "{} observations or baseline sabotage failed\n{}{}",
                entry.id, result.stdout, result.stderr
            ));
        }
        candidate_pins += observation_count;
        candidate_runs += 1;
    }
    Ok(ExecutionReport {
        base_runs,
        base_pins,
        composed_floor_runs: base_runs,
        composed_floor_pins: floor_pins,
        candidate_runs,
        candidate_pins,
        sabotage_runs: candidate_runs + usize::from(worker == 0),
    })
}

fn run_combined_matrix(
    prepared: &PreparedPinEngine,
    snapshot: &Snapshot,
    inventory: &SourceInventory,
) -> AuditResult<(usize, usize)> {
    let cases = snapshot.reviewed.matrix.iter().collect::<Vec<_>>();
    let facts = matrix_fact_lines(&cases);
    let additions = facts.iter().map(String::as_str).collect::<Vec<_>>();
    let matrix_text = matrix_pin_lines(&cases, inventory).join("\n");
    let mut floor_storage = Vec::with_capacity(cases.len());
    for case in &cases {
        let lines = composed_floor_pin_lines(case);
        validate_composed_floor_pin_lines(case, &lines)?;
        floor_storage.push((
            format!("{}.entitlement.pins.nibli", case.id),
            lines.join("\n"),
        ));
    }
    let mut pin_files = Vec::with_capacity(cases.len() + 1);
    pin_files.push(LoadedSource::new("matrix.pins.nibli", &matrix_text));
    pin_files.extend(
        floor_storage
            .iter()
            .map(|(name, body)| LoadedSource::new(name, body)),
    );
    let result = run_with_bound("certified combined placement matrix", || {
        Ok(prepared.run_patched_files(&[], &additions, &pin_files, PinOptions::default()))
    })?;
    let expected_matrix = cases
        .iter()
        .map(|case| case_queries(case, inventory).len())
        .sum::<usize>();
    if result.exit_code != 0
        || result.files.len() != cases.len() + 1
        || result.files[0].pins != expected_matrix
        || result.files[0].findings != 0
        || result.files[0].harness != 0
        || result.files[1..]
            .iter()
            .any(|file| file.pins != 1 || file.findings != 0 || file.harness != 0)
    {
        return Err(format!(
            "certified combined placement matrix failed\n{}{}",
            result.stdout, result.stderr
        ));
    }
    Ok((expected_matrix, cases.len()))
}

fn require_findings(
    result: &crate::pin::RunOutput,
    expected: usize,
    label: &str,
) -> AuditResult<()> {
    if result.exit_code != crate::pin::EXIT_FINDING
        || result.findings.len() != expected
        || !result.harness.is_empty()
        || !result.resolved.is_empty()
    {
        return Err(format!(
            "{label}: expected exactly {expected} findings\n{}{}",
            result.stdout, result.stderr
        ));
    }
    Ok(())
}

fn negative_controls(context: &Context, snapshot: &Snapshot) -> AuditResult<usize> {
    let validate = |reviewed: ReviewedSource| -> AuditResult<()> {
        let mut changed = snapshot.clone();
        changed.reviewed = reviewed;
        validate_source(context, &changed).map(|_| ())
    };
    let mut controls = 0usize;
    macro_rules! fails {
        ($label:expr, $body:expr) => {{
            expect_failure($label, $body)?;
            controls += 1;
        }};
    }

    let mut changed = snapshot.reviewed.clone();
    changed.constitution_sha256 = "0".repeat(64);
    fails!("constitution digest drift", validate(changed));

    let encoded = serde_json::to_string(&snapshot.reviewed).map_err(|error| error.to_string())?;
    let boolean_schema = encoded.replacen("\"schema_version\":2", "\"schema_version\":true", 1);
    fails!(
        "boolean schema version",
        serde_json::from_str::<ReviewedSource>(&boolean_schema)
            .map(|_| ())
            .map_err(|error| error.to_string())
    );
    let floating_timeout = encoded.replacen(
        "\"subprocess_timeout_seconds\":180",
        "\"subprocess_timeout_seconds\":180.0",
        1,
    );
    fails!(
        "floating subprocess timeout",
        serde_json::from_str::<ReviewedSource>(&floating_timeout)
            .map(|_| ())
            .map_err(|error| error.to_string())
    );

    let mut changed = snapshot.reviewed.clone();
    changed
        .producer_fingerprints
        .insert("building".into(), "0".repeat(64));
    fails!("building producer drift", validate(changed));

    let fact_source = format!(
        "{}\n# Ground-producer discovery controls (temporary, not enacted).\nfit(GroundPlacementProbe, Homestay).\ndwell(GroundPlacementProbe).\nbuilding(MedSec, GroundPlacementProbe).\n",
        snapshot.kb_text
    );
    let fact_inventory = source_inventory(&fact_source)?;
    for relation in TARGET_RELATIONS {
        fails!(
            &format!("ground {relation} producer discovered"),
            require_sha(
                &snapshot.reviewed.producer_fingerprints[relation],
                &format!("ground-control producer_fingerprints.{relation}"),
                Some(&fact_inventory.fingerprints[relation]),
            )
        );
    }
    fails!(
        "ground building destination discovered",
        if snapshot.reviewed.destination_constants != fact_inventory.destinations {
            Err("ground building destination changed the manifest".into())
        } else {
            Ok(())
        }
    );
    let mut changed = snapshot.reviewed.clone();
    changed.destination_constants.pop();
    fails!("hidden destination", validate(changed));
    let mut widened = snapshot.reviewed.destination_constants.clone();
    widened.push("MedSec".into());
    fails!(
        "new destination discovered",
        if widened != snapshot.reviewed.destination_constants {
            Err("new destination changed the manifest".into())
        } else {
            Ok(())
        }
    );

    let mut changed = snapshot.reviewed.clone();
    changed.matrix.pop();
    fails!("missing Cartesian row", validate(changed));
    for subject_kind in ["registered_free", "registered_person"] {
        let mut changed = snapshot.reviewed.clone();
        let case = changed
            .matrix
            .iter_mut()
            .find(|case| case.subject_kind == subject_kind)
            .ok_or_else(|| "negative-control subject case missing".to_owned())?;
        case.dwell = "TRUE".into();
        fails!(
            &format!("painted {subject_kind} delivery accepted"),
            validate(changed)
        );
    }
    let mut changed = snapshot.reviewed.clone();
    changed.mutations.pop();
    fails!("missing mutation class", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    changed.mutations[0].mutation_sha256 = "0".repeat(64);
    fails!("mutation digest drift", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    changed.mutations[0].baseline_flips[0].candidate_expected = changed.mutations[0].baseline_flips
        [0]
    .baseline_expected
    .clone();
    fails!("mutation no longer flips baseline", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    mutation_mut(&mut changed, "painted-free-person-delivery")?
        .baseline_flips
        .pop();
    fails!("painted-delivery row pruned", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    let missing = mutation_mut(&mut changed, "missing-required-destination")?;
    let removed = missing
        .err_absence_case_refs
        .pop()
        .ok_or_else(|| "negative-control case ref missing".to_owned())?;
    missing.alarm_setup_facts.remove(&removed);
    fails!("missing-destination case pruned", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    let first = &mut changed.mutations[0];
    let case_ref = first.err_absence_case_refs[0].clone();
    first.alarm_setup_facts.remove(&case_ref);
    fails!("vacuous placement-alarm setup", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    let first = &mut changed.mutations[0];
    let case_ref = first.err_absence_case_refs[0].clone();
    first.alarm_setup_facts.insert(
        case_ref,
        "put(State, Confined_NotSevere_NoFamily_Home, HighSec)".into(),
    );
    fails!("downgraded placement-alarm setup", validate(changed));

    let mut changed = snapshot.reviewed.clone();
    changed.acceptance_result.does_not_establish = "Everything is assured.".into();
    fails!("assurance overclaim", validate(changed));
    let probe = snapshot
        .reviewed
        .matrix
        .first()
        .ok_or_else(|| "matrix is empty".to_owned())?;
    let mut overlay = composed_floor_pin_lines(probe);
    overlay.insert(
        3,
        format!(
            "person({}).",
            case_subject(&probe.subject_kind, &probe.axes)
        ),
    );
    fails!(
        "composed floor standing overlay",
        validate_composed_floor_pin_lines(probe, &overlay)
    );
    fails!(
        "duplicate JSON key",
        parse_json_no_duplicates(br#"{"status":"bounded","status":"assured"}"#)
            .map(|_| ())
            .map_err(|error| error.to_string())
    );
    if controls != 23 {
        return Err(format!(
            "internal control-count drift: {controls}, expected 23"
        ));
    }
    Ok(controls)
}

fn expect_failure<T>(label: &str, result: AuditResult<T>) -> AuditResult<()> {
    if result.is_err() {
        Ok(())
    } else {
        Err(format!("structural negative control did not fail: {label}"))
    }
}

fn mutation_mut<'a>(
    source: &'a mut ReviewedSource,
    identifier: &str,
) -> AuditResult<&'a mut MutationCase> {
    source
        .mutations
        .iter_mut()
        .find(|entry| entry.id == identifier)
        .ok_or_else(|| format!("mutation {identifier:?} missing"))
}

struct JsonSeed;

impl<'de> DeserializeSeed<'de> for JsonSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
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
        Ok(Value::String(value.into()))
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

    fn generation_fixture() -> (tempfile::TempDir, Context) {
        let live = context();
        let temporary = tempfile::tempdir().expect("temporary placement repository");
        for relative in [DEFAULT_SOURCE, DEFAULT_KB] {
            let target = temporary.path().join(relative);
            std::fs::create_dir_all(target.parent().expect("fixture parent"))
                .expect("create fixture parent");
            std::fs::copy(live.path(relative), &target).expect("copy placement input");
        }
        let source = load_snapshot(&live, false).expect("live placement snapshot");
        for impact in &source.reviewed.narrowness_impacts {
            let relative = impact
                .artifact_ref
                .split_once("::")
                .expect("narrowness path::needle")
                .0;
            let target = temporary.path().join(relative);
            std::fs::create_dir_all(target.parent().expect("narrowness fixture parent"))
                .expect("create narrowness fixture parent");
            std::fs::copy(live.path(relative), target).expect("copy narrowness fixture");
        }
        let isolated = Context::from_test_root(temporary.path().to_path_buf());
        (temporary, isolated)
    }

    #[test]
    fn live_structural_check_and_report_are_exact() {
        let report = check(&context()).expect("live placement check");
        assert_eq!(report.structural_controls, 23);
        assert_eq!(report.execution, None);
        assert_eq!(
            report.to_string(),
            "new-book-plans/placement-exhaustiveness-audit.md is current; 23 structural negative controls pass; execution skipped"
        );
    }

    #[test]
    fn duplicate_keys_are_rejected_before_typed_deserialization() {
        let error = parse_json_no_duplicates(br#"{"outer":{"x":1,"x":2}}"#)
            .expect_err("duplicate key must fail");
        assert!(error.to_string().contains("duplicate JSON object key: x"));
    }

    #[test]
    fn fingerprint_shape_is_stable() {
        let report = fingerprints(&context()).expect("fingerprints");
        let parsed: Value = serde_json::from_str(&report.0).expect("fingerprint JSON");
        assert_eq!(parsed["mutations"].as_object().map(Map::len), Some(5));
    }

    #[test]
    fn native_generation_installs_the_exact_typed_projection() {
        let (_temporary, isolated) = generation_fixture();
        let report = generate(&isolated).expect("native placement generation");
        assert_eq!(report.structural_controls, 23);
        assert_eq!(
            report.to_string(),
            "new-book-plans/placement-exhaustiveness-audit.md: regenerated (structural generation; execution not requested); 23 structural negative controls pass"
        );
        let generated = std::fs::read_to_string(isolated.path(DEFAULT_OUTPUT))
            .expect("generated placement report");
        assert!(generated.contains(
            "<!-- Generated by the native rights-verify placement refresh; do not edit. -->"
        ));
        assert!(generated.contains(
            "./verify.sh --refresh placement-exhaustiveness\n./verify.sh --quick\n./verify.sh"
        ));
        assert!(!generated.contains(concat!("11-placement-", "exhaustiveness.py")));
        check(&isolated).expect("generated projection passes the typed check path");
    }

    #[cfg(unix)]
    #[test]
    fn native_generation_rejects_a_symlink_output_without_touching_its_target() {
        use std::os::unix::fs::symlink;

        let (_temporary, isolated) = generation_fixture();
        let protected = isolated.path("protected.md");
        std::fs::write(&protected, b"protected\n").expect("write protected target");
        symlink(&protected, isolated.path(DEFAULT_OUTPUT)).expect("install output symlink");
        let error = generate(&isolated).expect_err("symlink output must fail");
        assert!(
            error
                .to_string()
                .contains("generated output may not be a symlink")
        );
        assert_eq!(
            std::fs::read(&protected).expect("read protected target"),
            b"protected\n"
        );
    }

    #[test]
    #[ignore = "full in-process placement mutation execution"]
    fn live_execution_passes() {
        let report = check_execute(&context()).expect("live placement execution");
        assert_eq!(report.execution.expect("execution").base_runs, 24);
    }
}
