// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;

pub(crate) const STEP_NAME: &str = "record-integrity assurance";

const DEFAULT_SOURCE: &str = "new-book-plans/record-integrity-assurance-case.json";
const DEFAULT_LEDGER: &str = "new-book-plans/assertion-surface-contracts.json";
const DEFAULT_OUTPUT: &str = "new-book-plans/record-integrity-assurance-case.md";

const POSTURES: [&str; 5] = [
    "book1_target_unimplemented",
    "book2_external_assumption",
    "current_verified",
    "external_verified",
    "refused_or_unprovable",
];
const EVIDENCE_ROLES: [&str; 4] = [
    "supports_current",
    "supports_external",
    "exposes_gap",
    "sets_boundary",
];
const EVIDENCE_KINDS: [&str; 7] = [
    "counterfactual",
    "decision",
    "formal",
    "generated",
    "operational",
    "prose",
    "reviewed",
];
const MANDATORY_DIMENSIONS: [&str; 22] = [
    "surface_completeness",
    "authorship",
    "authority",
    "permitted_basis",
    "provenance_authenticity",
    "visibility_privacy",
    "independent_witnessing",
    "separation_of_functions",
    "challenge",
    "append_only_correction_history",
    "retention",
    "deletion_control",
    "reconciliation",
    "external_assurance",
    "independent_recipient",
    "action_duty",
    "continuity_remedy",
    "omission_or_deletion_recovery",
    "temporal_status",
    "failure_polarity",
    "rule_integrity",
    "negative_premise_admissibility",
];
const REQUIRED_NARROWNESS_FILES: [&str; 8] = [
    "book-1/01-what-counts-as-evidence.md",
    "book-1/03-who-holds-the-pen.md",
    "book-1/05-voiding.md",
    "book-1/09-the-vote-conviction-does-not-take.md",
    "book-1/12-changing-the-rules.md",
    "book-1/14-when-the-system-notices-it-broke.md",
    "book-1/15-the-five-joints.md",
    "book-1/method.md",
];
const LIMITATION_KEYS: [&str; 8] = [
    "in_snapshot_absence",
    "t1_boundary",
    "monotone_derivation",
    "independence",
    "authenticity",
    "genesis",
    "classification_choice",
    "assurance_meta_root",
];

#[derive(Clone, Debug)]
pub(crate) struct Paths {
    pub(crate) source: PathBuf,
    pub(crate) ledger: PathBuf,
    pub(crate) output: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            source: PathBuf::from(DEFAULT_SOURCE),
            ledger: PathBuf::from(DEFAULT_LEDGER),
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub(crate) output: String,
    pub(crate) watched_mutations: usize,
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is current; {} negative controls pass",
            self.output, self.watched_mutations
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenerationReport {
    pub(crate) output: String,
}

impl fmt::Display for GenerationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: regenerated", self.output)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssuranceSource {
    spdx: String,
    schema_version: u64,
    assertion_surface_contracts_sha256: String,
    title: String,
    top_claim: TopClaim,
    status_meanings: BTreeMap<String, String>,
    required_dimensions: Vec<String>,
    limitations: BTreeMap<String, String>,
    boundary: Boundary,
    claims: Vec<Claim>,
    record_classes: Vec<RecordClass>,
    premise_classes: BTreeMap<String, String>,
    defeaters: Vec<Defeater>,
    fail_safe_defaults: Vec<FailSafeDefault>,
    narrowness_impacts: Vec<NarrownessImpact>,
    acceptance_gate: Vec<AcceptanceGate>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TopClaim {
    id: String,
    claim: String,
    argument: String,
    current_verdict: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    book1: Vec<String>,
    book2: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    id: String,
    title: String,
    claim: String,
    argument: String,
    posture: String,
    dimensions: Vec<String>,
    current_evidence: Vec<Evidence>,
    known_failure: String,
    target_contract: String,
    acceptance_evidence: Vec<String>,
    residual_assumption: String,
    owner_ref: String,
    temporal_status: String,
    book2_handoff: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    kind: String,
    role: String,
    supports: String,
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordClass {
    id: String,
    title: String,
    description: String,
    assurance_claims: Vec<String>,
    failure_posture: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defeater {
    id: String,
    title: String,
    attack: String,
    disposition: String,
    owner_claims: Vec<String>,
    failure_consequence: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FailSafeDefault {
    id: String,
    condition: String,
    required_default: String,
    rationale: String,
    owner_claims: Vec<String>,
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
struct AcceptanceGate {
    id: String,
    requirement: String,
    evidence_needed: String,
    owner_ref: String,
}

/// Strict root projection of the assertion-surface contract.
///
/// This assurance check needs the premise inventory only. All other fields are
/// still named explicitly so a reviewed schema change cannot be silently
/// ignored by serde.
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
    source: AssuranceSource,
    ledger: AssertionLedger,
    ledger_digest: String,
    references: BTreeMap<String, String>,
    source_relative: String,
    ledger_relative: String,
    output_relative: String,
    output_path: PathBuf,
    current_output: Option<String>,
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_with_paths(context, &Paths::default())
}

pub(crate) fn check_with_paths(context: &Context, paths: &Paths) -> Result<Report, Error> {
    let snapshot = load_snapshot(context, paths, true)?;
    let forbidden = forbidden_files(&snapshot);
    validate_source(
        &snapshot.source,
        &snapshot.ledger,
        &snapshot.ledger_digest,
        &snapshot.references,
        &forbidden,
    )?;
    let generated = render(
        &snapshot.source,
        &snapshot.ledger_digest,
        &snapshot.source_relative,
        &snapshot.ledger_relative,
    );
    let watched_mutations = negative_controls(
        &snapshot.source,
        &snapshot.ledger,
        &snapshot.ledger_digest,
        &snapshot.references,
        &forbidden,
    )?;
    if snapshot.current_output.as_deref() != Some(generated.as_str()) {
        return Err(assurance_error(format!(
            "{} is STALE — rerun without --check",
            snapshot.output_relative
        )));
    }
    Ok(Report {
        output: snapshot.output_relative,
        watched_mutations,
    })
}

pub(crate) fn generate(context: &Context) -> Result<GenerationReport, Error> {
    generate_with_paths(context, &Paths::default())
}

pub(crate) fn generate_with_paths(
    context: &Context,
    paths: &Paths,
) -> Result<GenerationReport, Error> {
    let snapshot = load_snapshot(context, paths, false)?;
    let forbidden = forbidden_files(&snapshot);
    validate_source(
        &snapshot.source,
        &snapshot.ledger,
        &snapshot.ledger_digest,
        &snapshot.references,
        &forbidden,
    )?;
    let generated = render(
        &snapshot.source,
        &snapshot.ledger_digest,
        &snapshot.source_relative,
        &snapshot.ledger_relative,
    );
    if let Some(parent) = snapshot.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&snapshot.output_path, generated.as_bytes())?;
    Ok(GenerationReport {
        output: snapshot.output_relative,
    })
}

fn load_snapshot(context: &Context, paths: &Paths, read_output: bool) -> Result<Snapshot, Error> {
    let source_path = resolve_path(context, &paths.source);
    let ledger_path = resolve_path(context, &paths.ledger);
    let output_path = resolve_path(context, &paths.output);
    let source_relative = repo_relative(context.root(), &source_path)?;
    let ledger_relative = repo_relative(context.root(), &ledger_path)?;
    let output_relative = repo_relative(context.root(), &output_path)?;
    if same_file_target(&output_path, &source_path) || same_file_target(&output_path, &ledger_path)
    {
        return Err(assurance_error(
            "output path must not overwrite the source or assertion ledger",
        ));
    }
    if output_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("md"))
    {
        return Err(assurance_error("output path must end in .md"));
    }

    let source_bytes = std::fs::read(&source_path).map_err(|error| {
        assurance_error(format!(
            "cannot read assurance source {}: {error}",
            source_path.display()
        ))
    })?;
    parse_json_no_duplicates(&source_bytes).map_err(|error| {
        assurance_error(format!(
            "cannot read assurance source {}: {error}",
            source_path.display()
        ))
    })?;
    let source: AssuranceSource = serde_json::from_slice(&source_bytes).map_err(|error| {
        assurance_error(format!(
            "cannot read assurance source {}: {error}",
            source_path.display()
        ))
    })?;

    let ledger_bytes = std::fs::read(&ledger_path).map_err(|error| {
        assurance_error(format!(
            "cannot read assertion ledger {}: {error}",
            ledger_path.display()
        ))
    })?;
    parse_json_no_duplicates(&ledger_bytes).map_err(|error| {
        assurance_error(format!(
            "cannot read assertion ledger {}: {error}",
            ledger_path.display()
        ))
    })?;
    let ledger = serde_json::from_slice::<AssertionLedger>(&ledger_bytes).map_err(|error| {
        assurance_error(format!(
            "cannot read assertion ledger {}: {error}",
            ledger_path.display()
        ))
    })?;
    let ledger_digest = sha256(&ledger_bytes);

    let mut reference_paths = collect_reference_paths(&source);
    reference_paths.insert(source_relative.clone());
    reference_paths.insert(output_relative.clone());
    reference_paths.insert("TODO.md".to_owned());
    let references = load_reference_snapshot(context.root(), &reference_paths)?;
    let current_output = if read_output {
        Some(std::fs::read_to_string(&output_path).map_err(|error| {
            assurance_error(format!(
                "cannot read generated report {}: {error}",
                output_path.display()
            ))
        })?)
    } else {
        None
    };

    Ok(Snapshot {
        source,
        ledger,
        ledger_digest,
        references,
        source_relative,
        ledger_relative,
        output_relative,
        output_path,
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

fn repo_relative(root: &Path, path: &Path) -> Result<String, Error> {
    let root = root.canonicalize()?;
    let resolved = canonicalize_allow_missing(path)?;
    resolved
        .strip_prefix(&root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            assurance_error(format!(
                "reference path escapes the repository: {}",
                path.display()
            ))
        })
}

fn canonicalize_allow_missing(path: &Path) -> Result<PathBuf, Error> {
    let normalized = lexical_normalize(path)?;
    let mut existing = normalized.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            assurance_error(format!(
                "reference path escapes the repository: {}",
                path.display()
            ))
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            assurance_error(format!(
                "reference path escapes the repository: {}",
                path.display()
            ))
        })?;
    }
    let mut resolved = existing.canonicalize()?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn lexical_normalize(path: &Path) -> Result<PathBuf, Error> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(assurance_error(format!(
                        "reference path escapes the repository: {}",
                        path.display()
                    )));
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn same_file_target(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn collect_reference_paths(source: &AssuranceSource) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for claim in &source.claims {
        add_reference_path(&mut paths, &claim.owner_ref);
        for evidence in &claim.current_evidence {
            add_reference_path(&mut paths, &evidence.reference);
        }
    }
    for impact in &source.narrowness_impacts {
        add_reference_path(&mut paths, &impact.artifact_ref);
    }
    for gate in &source.acceptance_gate {
        add_reference_path(&mut paths, &gate.owner_ref);
    }
    paths
}

fn add_reference_path(paths: &mut BTreeSet<String>, reference: &str) {
    if let Some((path, _)) = reference.split_once("::")
        && safe_reference_path(path)
    {
        paths.insert(path.to_owned());
    }
}

fn load_reference_snapshot(
    root: &Path,
    paths: &BTreeSet<String>,
) -> Result<BTreeMap<String, String>, Error> {
    let canonical_root = root.canonicalize()?;
    let mut result = BTreeMap::new();
    for relative in paths {
        if !safe_reference_path(relative) {
            continue;
        }
        let candidate = root.join(relative);
        let Ok(canonical) = candidate.canonicalize() else {
            continue;
        };
        if !canonical.starts_with(&canonical_root) || !canonical.is_file() {
            continue;
        }
        let body = std::fs::read_to_string(&canonical).map_err(|error| {
            assurance_error(format!("cannot read referenced file {relative}: {error}"))
        })?;
        result.insert(relative.clone(), body);
    }
    Ok(result)
}

fn forbidden_files(snapshot: &Snapshot) -> HashSet<String> {
    [
        DEFAULT_SOURCE.to_owned(),
        DEFAULT_OUTPUT.to_owned(),
        snapshot.source_relative.clone(),
        snapshot.output_relative.clone(),
    ]
    .into_iter()
    .collect()
}

fn validate_source(
    source: &AssuranceSource,
    ledger: &AssertionLedger,
    ledger_digest: &str,
    references: &BTreeMap<String, String>,
    forbidden_evidence_files: &HashSet<String>,
) -> Result<(), Error> {
    if reviewed_text(&source.spdx, "spdx")? != "CC-BY-4.0" {
        return Err(assurance_error(
            "spdx: assurance-case prose must be CC-BY-4.0",
        ));
    }
    if source.schema_version != 1 {
        return Err(assurance_error("schema_version: expected integer 1"));
    }
    let expected_digest = reviewed_text(
        &source.assertion_surface_contracts_sha256,
        "assertion_surface_contracts_sha256",
    )?;
    if !is_lower_hex(expected_digest, 64) {
        return Err(assurance_error(
            "assertion_surface_contracts_sha256: expected 64 lowercase hex characters",
        ));
    }
    if expected_digest != ledger_digest {
        return Err(assurance_error(format!(
            "assertion_surface_contracts_sha256: assertion ledger drifted; \
             review premise classifications and update digest to {ledger_digest}"
        )));
    }
    reviewed_text(&source.title, "title")?;
    validate_top_claim(&source.top_claim)?;

    let expected_postures = string_set(POSTURES);
    if source
        .status_meanings
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>()
        != expected_postures
    {
        return Err(assurance_error(
            "status_meanings: must use the exact reviewed posture vocabulary",
        ));
    }
    for (posture, meaning) in &source.status_meanings {
        reviewed_text(meaning, &format!("status_meanings.{posture}"))?;
    }

    validate_unique_texts(&source.required_dimensions, "required_dimensions", true)?;
    let dimension_set = source
        .required_dimensions
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mandatory = string_set(MANDATORY_DIMENSIONS);
    if dimension_set != mandatory {
        let missing = mandatory
            .difference(&dimension_set)
            .cloned()
            .collect::<Vec<_>>();
        let extra = dimension_set
            .difference(&mandatory)
            .cloned()
            .collect::<Vec<_>>();
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!("missing {}", missing.join(", ")));
        }
        if !extra.is_empty() {
            details.push(format!("not mandatory here {}", extra.join(", ")));
        }
        return Err(assurance_error(format!(
            "required_dimensions: {}",
            details.join("; ")
        )));
    }

    if source
        .limitations
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != LIMITATION_KEYS.into_iter().collect()
    {
        return Err(assurance_error(
            "limitations: must use the exact reviewed limitation schema",
        ));
    }
    for (key, limitation) in &source.limitations {
        reviewed_text(limitation, &format!("limitations.{key}"))?;
    }
    let absence = source.limitations["in_snapshot_absence"].to_lowercase();
    let missing_absence = [
        "in-snapshot",
        "cannot distinguish",
        "deleted",
        "never written",
    ]
    .into_iter()
    .filter(|phrase| !absence.contains(phrase))
    .collect::<Vec<_>>();
    if !missing_absence.is_empty() {
        return Err(assurance_error(format!(
            "limitations.in_snapshot_absence: must state that an in-snapshot rule \
             cannot distinguish a deleted entry from one never written; missing {}",
            missing_absence.join(", ")
        )));
    }

    validate_unique_texts(&source.boundary.book1, "boundary.book1", true)?;
    validate_unique_texts(&source.boundary.book2, "boundary.book2", true)?;
    let book2_text = source.boundary.book2.join(" ").to_lowercase();
    let handoffs = [
        ("storage", &["storage"][..]),
        ("identity", &["identity"][..]),
        (
            "cryptographic",
            &["cryptograph", "signature", "credential", "key lifecycle"][..],
        ),
        ("operational", &["operational", "operate", "operation"][..]),
    ];
    let missing_handoffs = handoffs
        .into_iter()
        .filter_map(|(label, alternatives)| {
            (!alternatives
                .iter()
                .any(|marker| book2_text.contains(marker)))
            .then_some(label)
        })
        .collect::<Vec<_>>();
    if !missing_handoffs.is_empty() {
        return Err(assurance_error(format!(
            "boundary.book2: must hand off storage, identity, cryptographic, and \
             operational mechanisms; missing {}",
            missing_handoffs.join(", ")
        )));
    }

    if source.claims.is_empty() {
        return Err(assurance_error("claims: must not be empty"));
    }
    for (index, claim) in source.claims.iter().enumerate() {
        validate_claim_shape(claim, index, references, forbidden_evidence_files)?;
    }
    let mut claim_ids = vec![source.top_claim.id.clone()];
    claim_ids.extend(source.claims.iter().map(|claim| claim.id.clone()));
    let duplicates = duplicates(&claim_ids);
    if !duplicates.is_empty() {
        return Err(assurance_error(format!(
            "claims: duplicate claim ID(s): {}",
            duplicates.join(", ")
        )));
    }
    let known_claims = claim_ids.iter().cloned().collect::<BTreeSet<_>>();
    let required_claims = required_ids("RI", 1..=13);
    let missing_claims = required_claims
        .difference(&known_claims)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_claims.is_empty() {
        return Err(assurance_error(format!(
            "claims: missing required claim ID(s): {}",
            missing_claims.join(", ")
        )));
    }
    let claims_by_id = source
        .claims
        .iter()
        .map(|claim| (claim.id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    if claims_by_id["RI-11"].posture != "refused_or_unprovable" {
        return Err(assurance_error(
            "claims: reviewed impossibility boundary must remain refused: RI-11",
        ));
    }
    if !["book2_external_assumption", "external_verified"]
        .contains(&claims_by_id["RI-10"].posture.as_str())
    {
        return Err(assurance_error(
            "claims: reviewed external control must remain external: RI-10",
        ));
    }
    let mut covered_dimensions = BTreeSet::new();
    for (index, claim) in source.claims.iter().enumerate() {
        covered_dimensions.extend(claim.dimensions.iter().cloned());
        for (evidence_index, evidence) in claim.current_evidence.iter().enumerate() {
            if evidence.supports != claim.id {
                return Err(assurance_error(format!(
                    "claims[{index}].current_evidence[{evidence_index}].supports: \
                     must name its owning claim {}",
                    claim.id
                )));
            }
        }
        validate_posture(claim, index)?;
    }
    let uncovered = mandatory
        .difference(&covered_dimensions)
        .cloned()
        .collect::<Vec<_>>();
    if !uncovered.is_empty() {
        return Err(assurance_error(format!(
            "claims: no assurance claim covers required dimension(s): {}",
            uncovered.join(", ")
        )));
    }
    let non_refused = source
        .claims
        .iter()
        .filter(|claim| claim.posture != "refused_or_unprovable")
        .collect::<Vec<_>>();
    let computed_verdict = if !non_refused.is_empty()
        && non_refused.iter().all(|claim| {
            ["current_verified", "external_verified"].contains(&claim.posture.as_str())
        }) {
        "established"
    } else {
        "not_established"
    };
    if source.top_claim.current_verdict != computed_verdict {
        return Err(assurance_error(format!(
            "top_claim.current_verdict: source says {}, but claim postures require {computed_verdict}",
            source.top_claim.current_verdict
        )));
    }

    let record_ids = validate_record_classes(&source.record_classes, &claims_by_id)?;
    let required_records = required_ids("RC", 1..=5);
    let missing_records = required_records
        .difference(&record_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_records.is_empty() {
        return Err(assurance_error(format!(
            "record_classes: missing required ID(s): {}",
            missing_records.join(", ")
        )));
    }
    let temporal = claims_by_id["RI-7"];
    if !temporal.current_evidence.iter().any(|evidence| {
        evidence
            .reference
            .starts_with("new-book-plans/book-1-time-model-decision.md::")
    }) {
        return Err(assurance_error(
            "claims: RI-7 must cite the ratified time-model decision",
        ));
    }
    if temporal.owner_ref
        != "new-book-plans/book-1-time-model-decision.md::## 7. Formal implementation and verification gate"
    {
        return Err(assurance_error(
            "claims: RI-7 must remain owned by the ratified time-model gate",
        ));
    }

    validate_premise_classes(&source.premise_classes, ledger, &record_ids)?;
    validate_defeaters(&source.defeaters, &known_claims)?;
    validate_fail_safe_defaults(&source.fail_safe_defaults, &known_claims)?;
    validate_narrowness(&source.narrowness_impacts, references)?;
    validate_acceptance_gate(&source.acceptance_gate, references)?;
    Ok(())
}

fn validate_top_claim(top: &TopClaim) -> Result<(), Error> {
    validate_id(&top.id, "top_claim.id", "RI")?;
    reviewed_text(&top.claim, "top_claim.claim")?;
    reviewed_text(&top.argument, "top_claim.argument")?;
    if !["established", "not_established"].contains(&top.current_verdict.as_str()) {
        return Err(assurance_error(
            "top_claim.current_verdict: expected established or not_established",
        ));
    }
    if top.id != "RI-0" {
        return Err(assurance_error("top_claim.id: expected RI-0"));
    }
    Ok(())
}

fn validate_claim_shape(
    claim: &Claim,
    index: usize,
    references: &BTreeMap<String, String>,
    forbidden_evidence_files: &HashSet<String>,
) -> Result<(), Error> {
    let path = format!("claims[{index}]");
    validate_id(&claim.id, &format!("{path}.id"), "RI")?;
    for (name, value) in [
        ("title", &claim.title),
        ("claim", &claim.claim),
        ("argument", &claim.argument),
        ("known_failure", &claim.known_failure),
        ("target_contract", &claim.target_contract),
        ("residual_assumption", &claim.residual_assumption),
        ("temporal_status", &claim.temporal_status),
        ("book2_handoff", &claim.book2_handoff),
    ] {
        reviewed_text(value, &format!("{path}.{name}"))?;
    }
    if !POSTURES.contains(&claim.posture.as_str()) {
        return Err(assurance_error(format!(
            "{path}.posture: expected one of {}",
            POSTURES.join(", ")
        )));
    }
    validate_unique_texts(&claim.dimensions, &format!("{path}.dimensions"), true)?;
    let unknown_dimensions = claim
        .dimensions
        .iter()
        .filter(|dimension| !MANDATORY_DIMENSIONS.contains(&dimension.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();
    if !unknown_dimensions.is_empty() {
        return Err(assurance_error(format!(
            "{path}.dimensions: unknown dimension(s): {}",
            unknown_dimensions
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    if claim.current_evidence.is_empty() {
        return Err(assurance_error(format!(
            "{path}.current_evidence: must not be empty"
        )));
    }
    for (evidence_index, evidence) in claim.current_evidence.iter().enumerate() {
        let evidence_path = format!("{path}.current_evidence[{evidence_index}]");
        reviewed_text(&evidence.kind, &format!("{evidence_path}.kind"))?;
        if !EVIDENCE_KINDS.contains(&evidence.kind.as_str()) {
            return Err(assurance_error(format!(
                "{evidence_path}.kind: expected one of {}",
                EVIDENCE_KINDS.join(", ")
            )));
        }
        reviewed_text(&evidence.role, &format!("{evidence_path}.role"))?;
        if !EVIDENCE_ROLES.contains(&evidence.role.as_str()) {
            return Err(assurance_error(format!(
                "{evidence_path}.role: expected one of {}",
                EVIDENCE_ROLES.join(", ")
            )));
        }
        validate_id(
            &evidence.supports,
            &format!("{evidence_path}.supports"),
            "RI",
        )?;
        validate_reference(
            &evidence.reference,
            &format!("{evidence_path}.ref"),
            references,
        )?;
        let evidence_file = evidence
            .reference
            .split_once("::")
            .map_or("", |pair| pair.0);
        if forbidden_evidence_files.contains(evidence_file) {
            return Err(assurance_error(format!(
                "{evidence_path}.ref: the assurance source or its generated \
                 report cannot serve as current evidence for itself"
            )));
        }
    }
    validate_unique_texts(
        &claim.acceptance_evidence,
        &format!("{path}.acceptance_evidence"),
        false,
    )?;
    validate_reference(&claim.owner_ref, &format!("{path}.owner_ref"), references)?;
    Ok(())
}

fn validate_posture(claim: &Claim, index: usize) -> Result<(), Error> {
    let path = format!("claims[{index}]");
    let roles = claim
        .current_evidence
        .iter()
        .map(|evidence| evidence.role.as_str())
        .collect::<HashSet<_>>();
    match claim.posture.as_str() {
        "current_verified" => {
            if !roles.contains("supports_current") {
                return Err(assurance_error(format!(
                    "{path}: current_verified requires supports_current evidence"
                )));
            }
            if roles.contains("exposes_gap") {
                return Err(assurance_error(format!(
                    "{path}: current_verified cannot rely on evidence that exposes a gap"
                )));
            }
            if roles.contains("supports_external") {
                return Err(assurance_error(format!(
                    "{path}: current_verified cannot substitute deployed evidence \
                     for a repository invariant"
                )));
            }
            if !claim.acceptance_evidence.is_empty() {
                return Err(assurance_error(format!(
                    "{path}: current_verified cannot retain unmet acceptance evidence"
                )));
            }
        }
        "external_verified" => {
            if claim.id != "RI-10" {
                return Err(assurance_error(format!(
                    "{path}: {} is not a reviewed schema-v1 external claim",
                    claim.id
                )));
            }
            let operational = claim.current_evidence.iter().any(|evidence| {
                evidence.role == "supports_external" && evidence.kind == "operational"
            });
            if !operational {
                return Err(assurance_error(format!(
                    "{path}: external_verified requires reviewed operational evidence"
                )));
            }
            if roles.contains("exposes_gap") || roles.contains("supports_current") {
                return Err(assurance_error(format!(
                    "{path}: external_verified cannot rely on gap evidence or a \
                     repository-only invariant"
                )));
            }
            if !claim.acceptance_evidence.is_empty() {
                return Err(assurance_error(format!(
                    "{path}: external_verified cannot retain unmet acceptance evidence"
                )));
            }
            if !claim.owner_ref.starts_with("book-2/") {
                return Err(assurance_error(format!(
                    "{path}: external_verified owner must be in book-2/"
                )));
            }
        }
        "book1_target_unimplemented" => {
            if claim.acceptance_evidence.is_empty() {
                return Err(assurance_error(format!(
                    "{path}: book1_target_unimplemented requires acceptance evidence"
                )));
            }
            if claim.target_contract.trim().is_empty() {
                return Err(assurance_error(format!(
                    "{path}: book1_target_unimplemented requires a target contract"
                )));
            }
            if !roles.contains("exposes_gap") && !roles.contains("sets_boundary") {
                return Err(assurance_error(format!(
                    "{path}: book1_target_unimplemented requires gap or boundary evidence"
                )));
            }
        }
        "book2_external_assumption" => {
            if claim.id != "RI-10" {
                return Err(assurance_error(format!(
                    "{path}: {} is not a reviewed schema-v1 external claim",
                    claim.id
                )));
            }
            if claim.book2_handoff.trim().is_empty() {
                return Err(assurance_error(format!(
                    "{path}: book2_external_assumption requires a Book 2 handoff"
                )));
            }
            if claim.residual_assumption.trim().is_empty() {
                return Err(assurance_error(format!(
                    "{path}: book2_external_assumption requires a residual assumption"
                )));
            }
            if claim.acceptance_evidence.is_empty() {
                return Err(assurance_error(format!(
                    "{path}: book2_external_assumption requires acceptance evidence"
                )));
            }
            if !roles.contains("sets_boundary")
                || roles.contains("supports_current")
                || roles.contains("supports_external")
            {
                return Err(assurance_error(format!(
                    "{path}: book2_external_assumption requires boundary evidence, \
                     not evidence claiming the external service is current"
                )));
            }
            if !claim.owner_ref.starts_with("book-2/") {
                return Err(assurance_error(format!(
                    "{path}: book2_external_assumption owner must be in book-2/"
                )));
            }
        }
        "refused_or_unprovable" => {
            if claim.id != "RI-11" {
                return Err(assurance_error(format!(
                    "{path}: {} is not a reviewed schema-v1 refusal",
                    claim.id
                )));
            }
            if !explicit_impossibility(&claim.claim) {
                return Err(assurance_error(format!(
                    "{path}: refused_or_unprovable must state the impossibility \
                     explicitly in its claim"
                )));
            }
            if !claim.acceptance_evidence.is_empty() {
                return Err(assurance_error(format!(
                    "{path}: refused_or_unprovable cannot retain an implementation gate"
                )));
            }
            if !roles.contains("supports_current")
                && !roles.contains("supports_external")
                && !roles.contains("sets_boundary")
            {
                return Err(assurance_error(format!(
                    "{path}: refused_or_unprovable requires evidence for its boundary"
                )));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_record_classes(
    classes: &[RecordClass],
    claims_by_id: &BTreeMap<&str, &Claim>,
) -> Result<BTreeSet<String>, Error> {
    if classes.is_empty() {
        return Err(assurance_error("record_classes: must not be empty"));
    }
    let mut identifiers = BTreeSet::new();
    for (index, class) in classes.iter().enumerate() {
        let path = format!("record_classes[{index}]");
        validate_id(&class.id, &format!("{path}.id"), "RC")?;
        if !identifiers.insert(class.id.clone()) {
            return Err(assurance_error(format!(
                "{path}.id: duplicate record-class ID {}",
                class.id
            )));
        }
        reviewed_text(&class.title, &format!("{path}.title"))?;
        reviewed_text(&class.description, &format!("{path}.description"))?;
        validate_unique_texts(
            &class.assurance_claims,
            &format!("{path}.assurance_claims"),
            true,
        )?;
        let unknown = class
            .assurance_claims
            .iter()
            .filter(|identifier| !claims_by_id.contains_key(identifier.as_str()))
            .cloned()
            .collect::<BTreeSet<_>>();
        if !unknown.is_empty() {
            return Err(assurance_error(format!(
                "{path}.assurance_claims: unknown claim ID(s): {}",
                unknown.into_iter().collect::<Vec<_>>().join(", ")
            )));
        }
        let covered = class
            .assurance_claims
            .iter()
            .flat_map(|identifier| claims_by_id[identifier.as_str()].dimensions.iter().cloned())
            .collect::<BTreeSet<_>>();
        let missing = string_set(MANDATORY_DIMENSIONS)
            .difference(&covered)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(assurance_error(format!(
                "{path}.assurance_claims: record class bypasses mandatory \
                 assurance dimension(s): {}",
                missing.join(", ")
            )));
        }
        reviewed_text(&class.failure_posture, &format!("{path}.failure_posture"))?;
    }
    Ok(identifiers)
}

fn validate_premise_classes(
    mapping: &BTreeMap<String, String>,
    ledger: &AssertionLedger,
    record_ids: &BTreeSet<String>,
) -> Result<(), Error> {
    let ledger_keys = ledger.premises.keys().cloned().collect::<BTreeSet<_>>();
    let mapping_keys = mapping.keys().cloned().collect::<BTreeSet<_>>();
    if ledger_keys != mapping_keys {
        let missing = ledger_keys
            .difference(&mapping_keys)
            .cloned()
            .collect::<Vec<_>>();
        let extra = mapping_keys
            .difference(&ledger_keys)
            .cloned()
            .collect::<Vec<_>>();
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!("uncovered premise(s): {}", missing.join(", ")));
        }
        if !extra.is_empty() {
            details.push(format!("unknown premise(s): {}", extra.join(", ")));
        }
        return Err(assurance_error(format!(
            "premise_classes: {}",
            details.join("; ")
        )));
    }
    for (premise, class_id) in mapping {
        validate_id(class_id, &format!("premise_classes.{premise}"), "RC")?;
        if !record_ids.contains(class_id) {
            return Err(assurance_error(format!(
                "premise_classes.{premise}: unknown record-class ID {class_id}"
            )));
        }
    }
    Ok(())
}

fn validate_defeaters(entries: &[Defeater], known_claims: &BTreeSet<String>) -> Result<(), Error> {
    if entries.is_empty() {
        return Err(assurance_error("defeaters: must not be empty"));
    }
    let mut identifiers = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let path = format!("defeaters[{index}]");
        validate_id(&entry.id, &format!("{path}.id"), "RD")?;
        if !identifiers.insert(entry.id.clone()) {
            return Err(assurance_error(format!(
                "{path}.id: duplicate defeater ID {}",
                entry.id
            )));
        }
        for (key, value) in [
            ("title", &entry.title),
            ("attack", &entry.attack),
            ("disposition", &entry.disposition),
            ("failure_consequence", &entry.failure_consequence),
        ] {
            reviewed_text(value, &format!("{path}.{key}"))?;
        }
        if !POSTURES.contains(&entry.disposition.as_str()) {
            return Err(assurance_error(format!(
                "{path}.disposition: expected one of {}",
                POSTURES.join(", ")
            )));
        }
        validate_claim_references(
            &entry.owner_claims,
            &format!("{path}.owner_claims"),
            known_claims,
        )?;
    }
    require_ids("defeaters", &identifiers, &required_ids("RD", 1..=15))
}

fn validate_fail_safe_defaults(
    entries: &[FailSafeDefault],
    known_claims: &BTreeSet<String>,
) -> Result<(), Error> {
    if entries.is_empty() {
        return Err(assurance_error("fail_safe_defaults: must not be empty"));
    }
    let mut identifiers = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let path = format!("fail_safe_defaults[{index}]");
        validate_id(&entry.id, &format!("{path}.id"), "RF")?;
        if !identifiers.insert(entry.id.clone()) {
            return Err(assurance_error(format!(
                "{path}.id: duplicate fail-safe ID {}",
                entry.id
            )));
        }
        for (key, value) in [
            ("condition", &entry.condition),
            ("required_default", &entry.required_default),
            ("rationale", &entry.rationale),
        ] {
            reviewed_text(value, &format!("{path}.{key}"))?;
        }
        validate_claim_references(
            &entry.owner_claims,
            &format!("{path}.owner_claims"),
            known_claims,
        )?;
    }
    require_ids(
        "fail_safe_defaults",
        &identifiers,
        &required_ids("RF", 1..=6),
    )
}

fn validate_narrowness(
    entries: &[NarrownessImpact],
    references: &BTreeMap<String, String>,
) -> Result<(), Error> {
    if entries.is_empty() {
        return Err(assurance_error("narrowness_impacts: must not be empty"));
    }
    let mut seen = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let path = format!("narrowness_impacts[{index}]");
        validate_reference(
            &entry.artifact_ref,
            &format!("{path}.artifact_ref"),
            references,
        )?;
        if !seen.insert(entry.artifact_ref.clone()) {
            return Err(assurance_error(format!(
                "{path}.artifact_ref: duplicate narrowness artifact"
            )));
        }
        for (key, value) in [
            ("current_claim", &entry.current_claim),
            ("classification", &entry.classification),
            ("reason", &entry.reason),
            ("future_trigger", &entry.future_trigger),
        ] {
            reviewed_text(value, &format!("{path}.{key}"))?;
        }
        if !["preserved", "revised", "retired"].contains(&entry.classification.as_str()) {
            return Err(assurance_error(format!(
                "{path}.classification: expected preserved, revised, or retired"
            )));
        }
    }
    let covered = seen
        .iter()
        .filter_map(|reference| reference.split_once("::").map(|pair| pair.0.to_owned()))
        .collect::<BTreeSet<_>>();
    let required = string_set(REQUIRED_NARROWNESS_FILES);
    let missing = required.difference(&covered).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(assurance_error(format!(
            "narrowness_impacts: missing standing artifact(s): {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn validate_acceptance_gate(
    entries: &[AcceptanceGate],
    references: &BTreeMap<String, String>,
) -> Result<(), Error> {
    if entries.is_empty() {
        return Err(assurance_error("acceptance_gate: must not be empty"));
    }
    let mut identifiers = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let path = format!("acceptance_gate[{index}]");
        validate_id(&entry.id, &format!("{path}.id"), "RA")?;
        if !identifiers.insert(entry.id.clone()) {
            return Err(assurance_error(format!(
                "{path}.id: duplicate acceptance-gate ID {}",
                entry.id
            )));
        }
        reviewed_text(&entry.requirement, &format!("{path}.requirement"))?;
        reviewed_text(&entry.evidence_needed, &format!("{path}.evidence_needed"))?;
        validate_reference(&entry.owner_ref, &format!("{path}.owner_ref"), references)?;
    }
    require_ids("acceptance_gate", &identifiers, &required_ids("RA", 1..=8))
}

fn validate_claim_references(
    references: &[String],
    path: &str,
    known_claims: &BTreeSet<String>,
) -> Result<(), Error> {
    validate_unique_texts(references, path, true)?;
    let unknown = references
        .iter()
        .filter(|reference| !known_claims.contains(*reference))
        .cloned()
        .collect::<BTreeSet<_>>();
    if !unknown.is_empty() {
        return Err(assurance_error(format!(
            "{path}: unknown claim ID(s): {}",
            unknown.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(())
}

fn validate_reference(
    reference: &str,
    path: &str,
    references: &BTreeMap<String, String>,
) -> Result<(), Error> {
    reviewed_text(reference, path)?;
    if reference.matches("::").count() != 1 {
        return Err(assurance_error(format!(
            "{path}: reference must be repo-local path::unique literal needle"
        )));
    }
    let (raw_file, needle) = reference.split_once("::").expect("count checked");
    if raw_file.is_empty() || needle.is_empty() {
        return Err(assurance_error(format!(
            "{path}: reference path and needle must both be non-empty"
        )));
    }
    if raw_file.contains('\\') {
        return Err(assurance_error(format!(
            "{path}: reference paths must use forward slashes"
        )));
    }
    if !safe_reference_path(raw_file) {
        return Err(assurance_error(format!(
            "{path}: reference path must stay inside the repository"
        )));
    }
    let Some(body) = references.get(raw_file) else {
        return Err(assurance_error(format!(
            "{path}: referenced file does not exist: {raw_file}"
        )));
    };
    let count = body.matches(needle).count();
    if count != 1 {
        return Err(assurance_error(format!(
            "{path}: needle must occur exactly once in {raw_file}; found {count}"
        )));
    }
    Ok(())
}

fn safe_reference_path(path: &str) -> bool {
    let candidate = Path::new(path);
    !candidate.is_absolute()
        && !path.contains('\\')
        && candidate
            .components()
            .all(|component| matches!(component, Component::CurDir | Component::Normal(_)))
}

fn validate_unique_texts(values: &[String], path: &str, nonempty: bool) -> Result<(), Error> {
    if nonempty && values.is_empty() {
        return Err(assurance_error(format!("{path}: must not be empty")));
    }
    let mut unique = HashSet::new();
    for (index, value) in values.iter().enumerate() {
        reviewed_text(value, &format!("{path}[{index}]"))?;
        if !unique.insert(value) {
            return Err(assurance_error(format!(
                "{path}: duplicate values are not allowed"
            )));
        }
    }
    Ok(())
}

fn reviewed_text<'a>(value: &'a str, path: &str) -> Result<&'a str, Error> {
    let stripped = value.trim();
    let lower = stripped.to_lowercase();
    if stripped.is_empty()
        || ["tbd", "todo", "unknown", "n/a", "na", "pending"].contains(&lower.as_str())
    {
        return Err(assurance_error(format!(
            "{path}: requires reviewed, non-placeholder text"
        )));
    }
    Ok(value)
}

fn validate_id(value: &str, path: &str, family: &str) -> Result<(), Error> {
    reviewed_text(value, path)?;
    let Some(number) = value.strip_prefix(&format!("{family}-")) else {
        return Err(invalid_id(path, value, family));
    };
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_id(path, value, family));
    }
    Ok(())
}

fn invalid_id(path: &str, value: &str, family: &str) -> Error {
    let label = match family {
        "RI" => "RI-N claim",
        "RC" => "RC-N record-class",
        "RD" => "RD-N defeater",
        "RF" => "RF-N fail-safe",
        "RA" => "RA-N acceptance-gate",
        _ => family,
    };
    assurance_error(format!(
        "{path}: {value:?} must be a stable {label} identifier"
    ))
}

fn explicit_impossibility(claim: &str) -> bool {
    let lower = claim.to_lowercase();
    [
        "cannot",
        "can not",
        "unprovable",
        "not provable",
        "impossible",
        "does not distinguish",
        "no in-snapshot",
        "no internal",
        "no rule",
        "refused",
    ]
    .into_iter()
    .any(|marker| lower.contains(marker))
}

fn is_lower_hex(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn string_set<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
    values.into_iter().map(str::to_owned).collect()
}

fn required_ids(family: &str, range: impl IntoIterator<Item = usize>) -> BTreeSet<String> {
    range
        .into_iter()
        .map(|number| format!("{family}-{number}"))
        .collect()
}

fn require_ids(
    label: &str,
    actual: &BTreeSet<String>,
    required: &BTreeSet<String>,
) -> Result<(), Error> {
    let missing = required.difference(actual).cloned().collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(assurance_error(format!(
            "{label}: missing required ID(s): {}",
            missing.join(", ")
        )))
    }
}

fn duplicates(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            duplicates.insert(value.clone());
        }
    }
    duplicates.into_iter().collect()
}

fn render(
    source: &AssuranceSource,
    ledger_digest: &str,
    source_path: &str,
    ledger_path: &str,
) -> String {
    let classes = source
        .record_classes
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut lines = vec![
        format!("<!-- SPDX-License-Identifier: {} -->", source.spdx),
        "<!-- Generated by new-book-plans/8-record-integrity-assurance.py; do not edit. -->"
            .to_owned(),
        String::new(),
        format!("# {}", source.title),
        String::new(),
        "## Verdict and scope".to_owned(),
        String::new(),
        format!(
            "**{} — {}**",
            source
                .top_claim
                .current_verdict
                .replace('_', " ")
                .to_uppercase(),
            source.top_claim.claim
        ),
        String::new(),
        markdown(&source.top_claim.argument),
        String::new(),
        "The verdict becomes **ESTABLISHED** only when every non-refused control is".to_owned(),
        "`current_verified` or `external_verified`. A refusal can mark an honest".to_owned(),
        "model boundary; it cannot".to_owned(),
        "silently satisfy a control that the design claims to provide.".to_owned(),
        String::new(),
        "| posture | meaning |".to_owned(),
        "| --- | --- |".to_owned(),
    ];
    for posture in POSTURES {
        lines.push(format!(
            "| {} | {} |",
            code(posture),
            markdown(&source.status_meanings[posture])
        ));
    }
    lines.extend([
        String::new(),
        "## Limitations and Book 1/Book 2 boundary".to_owned(),
        String::new(),
        "These are load-bearing limitations, not implementation notes:".to_owned(),
        String::new(),
    ]);
    for key in LIMITATION_KEYS {
        lines.push(format!(
            "- **{}:** {}",
            title_words(&key.replace('_', " ")),
            markdown(&source.limitations[key])
        ));
    }
    lines.extend([String::new(), "### Book 1 owns".to_owned(), String::new()]);
    lines.extend(bullets(&source.boundary.book1));
    lines.extend([String::new(), "### Book 2 owns".to_owned(), String::new()]);
    lines.extend(bullets(&source.boundary.book2));
    lines.extend([
        String::new(),
        "## Claim summary".to_owned(),
        String::new(),
        "| claim | title | posture | assurance dimensions |".to_owned(),
        "| --- | --- | --- | --- |".to_owned(),
    ]);
    for claim in &source.claims {
        let dimensions = claim
            .dimensions
            .iter()
            .map(|item| code(item))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "| {} | {} | {} | {dimensions} |",
            code(&claim.id),
            markdown(&claim.title),
            code(&claim.posture)
        ));
    }
    lines.extend([String::new(), "## Claim details".to_owned(), String::new()]);
    for claim in &source.claims {
        lines.extend([
            format!("### {} — {}", claim.id, claim.title),
            String::new(),
            format!("**Claim.** {}", markdown(&claim.claim)),
            String::new(),
            format!("**Argument.** {}", markdown(&claim.argument)),
            String::new(),
            format!("- **Posture:** {}", code(&claim.posture)),
            format!(
                "- **Current failure:** {}",
                fallback_markdown(&claim.known_failure, "None recorded.")
            ),
            format!(
                "- **Target contract:** {}",
                fallback_markdown(&claim.target_contract, "None.")
            ),
            format!(
                "- **Residual assumption:** {}",
                fallback_markdown(&claim.residual_assumption, "None.")
            ),
            format!(
                "- **Temporal status:** {}",
                fallback_markdown(&claim.temporal_status, "None.")
            ),
            format!(
                "- **Book 2 handoff:** {}",
                fallback_markdown(&claim.book2_handoff, "None.")
            ),
            format!("- **Owner:** {}", code(&claim.owner_ref)),
            String::new(),
            "**Current evidence**".to_owned(),
            String::new(),
            "| kind | evidence role | supports | reference |".to_owned(),
            "| --- | --- | --- | --- |".to_owned(),
        ]);
        for evidence in &claim.current_evidence {
            lines.push(format!(
                "| {} | {} | {} | {} |",
                markdown(&evidence.kind),
                code(&evidence.role),
                code(&evidence.supports),
                code(&evidence.reference)
            ));
        }
        lines.extend([
            String::new(),
            "**Acceptance evidence still required**".to_owned(),
            String::new(),
        ]);
        if claim.acceptance_evidence.is_empty() {
            lines.push("None.".to_owned());
        } else {
            lines.extend(bullets(&claim.acceptance_evidence));
        }
        lines.push(String::new());
    }
    lines.extend([
        "## Record classes and premise coverage".to_owned(),
        String::new(),
        "Every writable premise in the reviewed assertion-surface ledger belongs".to_owned(),
        "to exactly one record class here. Classification does not authenticate an".to_owned(),
        "entry or make the chosen classes exhaustive; it selects the reviewed".to_owned(),
        "assurance argument that must govern it.".to_owned(),
        String::new(),
    ]);
    for class in &source.record_classes {
        let owners = class
            .assurance_claims
            .iter()
            .map(|item| code(item))
            .collect::<Vec<_>>()
            .join(", ");
        lines.extend([
            format!("### {} — {}", class.id, class.title),
            String::new(),
            markdown(&class.description),
            String::new(),
            format!("- **Assurance claims:** {owners}"),
            format!(
                "- **Failure posture:** {}",
                markdown(&class.failure_posture)
            ),
            String::new(),
        ]);
    }
    lines.extend([
        "| writable premise | record class | class title |".to_owned(),
        "| --- | --- | --- |".to_owned(),
    ]);
    for (premise, class_id) in &source.premise_classes {
        lines.push(format!(
            "| {} | {} | {} |",
            code(premise),
            code(class_id),
            markdown(&classes[class_id.as_str()].title)
        ));
    }
    lines.extend([String::new(), "## Defeaters".to_owned(), String::new()]);
    for entry in &source.defeaters {
        lines.extend([
            format!("### {} — {}", entry.id, entry.title),
            String::new(),
            format!("- **Attack:** {}", markdown(&entry.attack)),
            format!("- **Disposition:** {}", markdown(&entry.disposition)),
            format!(
                "- **Owned by:** {}",
                entry
                    .owner_claims
                    .iter()
                    .map(|item| code(item))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "- **If unresolved:** {}",
                markdown(&entry.failure_consequence)
            ),
            String::new(),
        ]);
    }
    lines.extend(["## Fail-safe defaults".to_owned(), String::new()]);
    for entry in &source.fail_safe_defaults {
        lines.extend([
            format!("### {}", entry.id),
            String::new(),
            format!("- **Condition:** {}", markdown(&entry.condition)),
            format!(
                "- **Required default:** {}",
                markdown(&entry.required_default)
            ),
            format!("- **Reason:** {}", markdown(&entry.rationale)),
            format!(
                "- **Owned by:** {}",
                entry
                    .owner_claims
                    .iter()
                    .map(|item| code(item))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            String::new(),
        ]);
    }
    lines.extend([
        "## Narrowness impacts".to_owned(),
        String::new(),
        "These standing claims must be re-reviewed when the named trigger occurs,".to_owned(),
        "even if their own numbered chapter derivations do not change.".to_owned(),
        String::new(),
        "| artifact | current claim | classification | reason | future trigger |".to_owned(),
        "| --- | --- | --- | --- | --- |".to_owned(),
    ]);
    for entry in &source.narrowness_impacts {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            code(&entry.artifact_ref),
            markdown(&entry.current_claim),
            markdown(&entry.classification),
            markdown(&entry.reason),
            markdown(&entry.future_trigger)
        ));
    }
    lines.extend([
        String::new(),
        "## Acceptance gate".to_owned(),
        String::new(),
        "The top verdict stays **NOT ESTABLISHED** until every applicable item has".to_owned(),
        "reviewed evidence and the claim postures are updated without weakening a".to_owned(),
        "refusal or moving a Book 2 assumption into the constitutional kernel.".to_owned(),
        String::new(),
        "| gate | requirement | evidence needed | owner |".to_owned(),
        "| --- | --- | --- | --- |".to_owned(),
    ]);
    for entry in &source.acceptance_gate {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            code(&entry.id),
            markdown(&entry.requirement),
            markdown(&entry.evidence_needed),
            code(&entry.owner_ref)
        ));
    }
    lines.extend([
        String::new(),
        "## Maintenance and limits".to_owned(),
        String::new(),
        format!("- Source: {}.", code(source_path)),
        format!(
            "- Assertion ledger: {}, exact SHA-256 {}.",
            code(ledger_path),
            code(ledger_digest)
        ),
        "- Regenerate after reviewing the JSON source; never hand-edit this report.".to_owned(),
        "- Run `python3 new-book-plans/8-record-integrity-assurance.py --check`.".to_owned(),
        "- The checker proves schema coverage, traceability, ledger coupling, and".to_owned(),
        "  report freshness. It does not prove real authorship, witness independence,".to_owned(),
        "  storage integrity, clock progress, omission recovery, or deletion recovery.".to_owned(),
        "- The bounded report at `new-book-plans/record-integrity-red-team.md`".to_owned(),
        "  executes selected release, adulthood, roster, relief, and forgiveness harms,".to_owned(),
        "  plus a negative control proving that bare `rotten` is inert. Those cases".to_owned(),
        "  expose flat-snapshot gaps and one input boundary; they do not establish".to_owned(),
        "  authorship, runtime attribution, recovery, liveness, or operational integrity,"
            .to_owned(),
        "  and they do not duplicate the staged temporal assurance harness.".to_owned(),
        String::new(),
    ]);
    lines.join("\n")
}

fn markdown(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

fn fallback_markdown(value: &str, fallback: &str) -> String {
    let rendered = markdown(value);
    if rendered.is_empty() {
        fallback.to_owned()
    } else {
        rendered
    }
}

fn code(value: &str) -> String {
    let fence = if value.contains('`') { "``" } else { "`" };
    format!("{fence}{value}{fence}")
}

fn bullets(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("- {}", markdown(value)))
        .collect()
}

fn title_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first
                    .to_uppercase()
                    .chain(characters.flat_map(char::to_lowercase))
                    .collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn negative_controls(
    source: &AssuranceSource,
    ledger: &AssertionLedger,
    ledger_digest: &str,
    references: &BTreeMap<String, String>,
    forbidden: &HashSet<String>,
) -> Result<usize, Error> {
    let mut controls = 0;
    let validate = |candidate: &AssuranceSource| {
        validate_source(candidate, ledger, ledger_digest, references, forbidden)
    };

    let mut changed = source.clone();
    changed.assertion_surface_contracts_sha256 = "0".repeat(64);
    expect_failure("assertion-ledger drift", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    let first = changed
        .premise_classes
        .keys()
        .next()
        .cloned()
        .expect("reviewed premises");
    changed.premise_classes.remove(&first);
    expect_failure("uncovered writable premise", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed
        .required_dimensions
        .retain(|dimension| dimension != "retention");
    expect_failure("missing lifecycle dimension", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed
        .record_classes
        .iter_mut()
        .find(|class| class.id == "RC-1")
        .expect("reviewed RC-1")
        .assurance_claims
        .retain(|claim| claim != "RI-5");
    expect_failure(
        "record class bypasses a mandatory dimension",
        validate(&changed),
    )?;
    controls += 1;

    let mut changed = source.clone();
    changed.claims.push(changed.claims[0].clone());
    expect_failure("duplicate claim ID", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    claim_mut(&mut changed, "RI-1").owner_ref =
        "TODO.md::negative-control-anchor-does-not-exist".to_owned();
    expect_failure("dangling evidence/owner reference", validate(&changed))?;
    controls += 1;

    let mut candidate = claim(source, "RI-1").clone();
    candidate.posture = "current_verified".to_owned();
    candidate.acceptance_evidence.clear();
    for evidence in &mut candidate.current_evidence {
        evidence.role = "exposes_gap".to_owned();
    }
    expect_failure(
        "gap evidence promoted to current",
        validate_posture(&candidate, 0),
    )?;
    controls += 1;

    let mut candidate = claim(source, "RI-10").clone();
    candidate.posture = "book2_external_assumption".to_owned();
    candidate.book2_handoff.clear();
    candidate.acceptance_evidence = vec!["Independent deployment evidence.".to_owned()];
    for evidence in &mut candidate.current_evidence {
        evidence.role = "sets_boundary".to_owned();
    }
    expect_failure(
        "external assumption without Book 2 handoff",
        validate_posture(&candidate, 0),
    )?;
    controls += 1;

    let mut candidate = claim(source, "RI-10").clone();
    candidate.posture = "external_verified".to_owned();
    candidate.acceptance_evidence.clear();
    for evidence in &mut candidate.current_evidence {
        evidence.role = "sets_boundary".to_owned();
    }
    expect_failure(
        "external claim promoted without operational evidence",
        validate_posture(&candidate, 0),
    )?;
    controls += 1;
    for evidence in &mut candidate.current_evidence {
        evidence.role = "supports_external".to_owned();
        evidence.kind = "operational".to_owned();
    }
    validate_posture(&candidate, 0)?;

    let mut candidate = claim(source, "RI-1").clone();
    candidate.posture = "refused_or_unprovable".to_owned();
    candidate.acceptance_evidence.clear();
    for evidence in &mut candidate.current_evidence {
        evidence.role = "sets_boundary".to_owned();
    }
    expect_failure(
        "implementable control disposition-washed as a refusal",
        validate_posture(&candidate, 0),
    )?;
    controls += 1;

    let mut changed = source.clone();
    changed.limitations.remove("in_snapshot_absence");
    expect_failure("missing deletion limitation", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.record_classes[0]
        .assurance_claims
        .push("RI-C-DOES-NOT-EXIST".to_owned());
    expect_failure("dangling claim reference", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    let first = changed
        .premise_classes
        .keys()
        .next()
        .cloned()
        .expect("reviewed premises");
    changed
        .premise_classes
        .insert(first, "RI-R-DOES-NOT-EXIST".to_owned());
    expect_failure("dangling record-class reference", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.top_claim.current_verdict = if changed.top_claim.current_verdict == "not_established" {
        "established".to_owned()
    } else {
        "not_established".to_owned()
    };
    expect_failure("top verdict inconsistent with postures", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.schema_version = 999;
    expect_failure("unknown schema version", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    claim_mut(&mut changed, "RI-1").title = "pending".to_owned();
    expect_failure("blank or placeholder claim prose", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.defeaters[0].disposition = "banana".to_owned();
    expect_failure("unknown defeater disposition", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    claim_mut(&mut changed, "RI-1").current_evidence[0].reference = format!(
        "{}::\"schema_version\": 1",
        forbidden
            .iter()
            .find(|path| path.ends_with("record-integrity-assurance-case.json"))
            .map(String::as_str)
            .unwrap_or(DEFAULT_SOURCE)
    );
    expect_failure("assurance case used as self-evidence", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.claims.retain(|claim| claim.id != "RI-12");
    for class in &mut changed.record_classes {
        class.assurance_claims.retain(|claim| claim != "RI-12");
    }
    expect_failure("required claim deleted", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.defeaters.retain(|entry| entry.id != "RD-1");
    expect_failure("required defeater deleted", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed
        .fail_safe_defaults
        .retain(|entry| entry.id != "RF-1");
    expect_failure("required fail-safe deleted", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.acceptance_gate.retain(|entry| entry.id != "RA-1");
    expect_failure("required acceptance gate deleted", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    changed.narrowness_impacts.retain(|entry| {
        entry.artifact_ref
            != "book-1/01-what-counts-as-evidence.md::Neither mechanism can find an encounter nobody reports, detect a"
    });
    expect_failure("standing narrowness review deleted", validate(&changed))?;
    controls += 1;

    let mut changed = source.clone();
    claim_mut(&mut changed, "RI-7").owner_ref = claim(source, "RI-1").owner_ref.clone();
    expect_failure("T3 transition owner bypassed", validate(&changed))?;
    controls += 1;

    expect_failure(
        "duplicate JSON object key",
        parse_json_no_duplicates(br#"{"premise_classes": {}, "premise_classes": {}}"#)
            .map(|_| ())
            .map_err(|error| assurance_error(error.to_string())),
    )?;
    controls += 1;

    Ok(controls)
}

fn claim<'a>(source: &'a AssuranceSource, identifier: &str) -> &'a Claim {
    source
        .claims
        .iter()
        .find(|claim| claim.id == identifier)
        .expect("reviewed claim exists")
}

fn claim_mut<'a>(source: &'a mut AssuranceSource, identifier: &str) -> &'a mut Claim {
    source
        .claims
        .iter_mut()
        .find(|claim| claim.id == identifier)
        .expect("reviewed claim exists")
}

fn expect_failure<T>(label: &str, result: Result<T, Error>) -> Result<(), Error> {
    if result.is_err() {
        Ok(())
    } else {
        Err(assurance_error(format!(
            "negative control did not fail: {label}"
        )))
    }
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

    fn visit_map<A>(self, mut mapping: A) -> Result<Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = mapping.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            values.insert(key, mapping.next_value_seed(JsonSeed)?);
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

fn assurance_error(message: impl Into<String>) -> Error {
    Error::new(format!("8-record-integrity-assurance: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context::discover().expect("discover repository")
    }

    fn snapshot() -> Snapshot {
        load_snapshot(&context(), &Paths::default(), true).expect("load live snapshot")
    }

    #[test]
    fn current_live_inputs_and_generated_report_pass() {
        let report = check(&context()).expect("live assurance check");
        assert_eq!(report.watched_mutations, 25);
        assert_eq!(report.output, DEFAULT_OUTPUT);
    }

    #[test]
    fn success_text_matches_python_exactly() {
        assert_eq!(
            Report {
                output: DEFAULT_OUTPUT.to_owned(),
                watched_mutations: 25,
            }
            .to_string(),
            "new-book-plans/record-integrity-assurance-case.md is current; 25 negative controls pass"
        );
    }

    #[test]
    fn renderer_matches_committed_report_byte_for_byte() {
        let snapshot = snapshot();
        let rendered = render(
            &snapshot.source,
            &snapshot.ledger_digest,
            &snapshot.source_relative,
            &snapshot.ledger_relative,
        );
        assert_eq!(rendered, snapshot.current_output.expect("current output"));
    }

    #[test]
    fn all_twenty_five_watched_mutations_fail() {
        let snapshot = snapshot();
        assert_eq!(
            negative_controls(
                &snapshot.source,
                &snapshot.ledger,
                &snapshot.ledger_digest,
                &snapshot.references,
                &forbidden_files(&snapshot),
            )
            .expect("negative controls"),
            25
        );
    }

    #[test]
    fn changed_ledger_digest_is_rejected() {
        let snapshot = snapshot();
        let error = validate_source(
            &snapshot.source,
            &snapshot.ledger,
            &"0".repeat(64),
            &snapshot.references,
            &forbidden_files(&snapshot),
        )
        .expect_err("ledger drift must fail");
        assert!(error.to_string().contains("assertion ledger drifted"));
    }

    #[test]
    fn missing_mandatory_dimension_is_rejected() {
        let snapshot = snapshot();
        let mut source = snapshot.source.clone();
        source
            .required_dimensions
            .retain(|dimension| dimension != "retention");
        let error = validate_source(
            &source,
            &snapshot.ledger,
            &snapshot.ledger_digest,
            &snapshot.references,
            &forbidden_files(&snapshot),
        )
        .expect_err("dimension deletion must fail");
        assert!(error.to_string().contains("missing retention"));
    }

    #[test]
    fn duplicate_json_keys_are_rejected_at_every_depth() {
        let error = parse_json_no_duplicates(br#"{"outer":{"value":1,"value":2}}"#)
            .expect_err("nested duplicate must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate JSON object key: value")
        );
    }

    #[test]
    fn assertion_ledger_projection_rejects_unknown_root_fields() {
        let raw = std::fs::read(context().path(DEFAULT_LEDGER)).expect("read assertion ledger");
        let mut ledger = parse_json_no_duplicates(&raw).expect("parse assertion ledger");
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
    fn generation_to_temporary_report_is_byte_exact() {
        let temporary = tempfile::tempdir_in(context().root()).expect("temporary directory");
        let relative = temporary
            .path()
            .strip_prefix(context().root())
            .expect("repo relative")
            .join("assurance.md");
        let paths = Paths {
            output: relative.clone(),
            ..Paths::default()
        };
        let report = generate_with_paths(&context(), &paths).expect("generate report");
        assert_eq!(report.output, relative.to_string_lossy());
        let generated = std::fs::read(context().path(&relative)).expect("generated bytes");
        let committed = std::fs::read(context().path(DEFAULT_OUTPUT)).expect("committed bytes");
        assert_eq!(generated, committed);
    }
}
