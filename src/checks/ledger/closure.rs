// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native claim-scoped constitutional-closure and model-allocation audit.
//!
//! The audit is a typed projection of an already validated full-society
//! ledger. `serde_json::Value` is deliberately confined to watched malformed
//! mutations and the ledger's canonical scope-digest boundary.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::fs;

use serde_json::Value;

use super::{
    AlternateRoute, Claim, Defect, DefectResolution, FunctionAllocation, LedgerDocument,
    ModelAllocation, Power, ValidatedLedger,
};
use crate::cli::Error;
use crate::context::Context;
use crate::refresh::{ImmutableRepositoryInputs, atomic_refresh_and_check};

pub(crate) const STEP_NAME: &str = "constitutional-closure and model-allocation audit";
const OUTPUT: &str = "new-book-plans/constitutional-closure-and-model-allocation-audit.md";
const STRUCTURAL_CONTROL_COUNT: usize = 74;
const EXPECTED_POWER_COUNT: usize = 210;
pub(super) const CURRENT_AUDIT_CONTROL_REF: &str =
    concat!("src/checks/ledger/closure.rs::fn negative_", "controls(");

const MODEL_NAMES: [(&str, &str); 7] = [
    ("FS-RTE-01", "Nibli formal entailment"),
    ("FS-RTE-02", "quantitative/resource models"),
    ("FS-RTE-03", "dynamic simulations"),
    ("FS-RTE-04", "evidence registry"),
    ("FS-RTE-05", "operational assurance"),
    ("FS-RTE-06", "reader/lived-experience testing"),
    ("FS-RTE-07", "repository source-derived adversarial audit"),
];

const REQUIREMENT_COMPONENTS: [(&str, &[&str]); 8] = [
    ("floor-lifecycle", &["delivery", "continuity", "remedy"]),
    (
        "public-power-lifecycle",
        &["source", "limit", "review", "temporal-status"],
    ),
    ("private-duty-explicitness", &["express-duty"]),
    ("record-lifecycle", &["writer", "challenge", "correction"]),
    (
        "democratic-floor-corridor",
        &["choice-source", "floor-boundary"],
    ),
    ("book-seam", &["responsible-book", "assurance-ceiling"]),
    ("external-assumption-disclosure", &["named-assumption"]),
    (
        "reader-claim-ownership",
        &["formal-owner", "evidentiary-owner"],
    ),
];

const PROFILE_SOURCES: [(&str, Option<&str>); 8] = [
    ("floor-lifecycle", Some("FS-LGR-02")),
    ("public-power-lifecycle", Some("FS-LGR-07")),
    ("private-duty-explicitness", Some("FS-LGR-03")),
    ("record-lifecycle", Some("FS-LGR-08")),
    ("democratic-floor-corridor", Some("FS-LGR-06")),
    ("book-seam", None),
    ("external-assumption-disclosure", None),
    ("reader-claim-ownership", None),
];

const READER_OWNER_REF: &str = concat!(
    "new-book-plans/book-1-reader-evidence-protocol-decision.md::",
    "## 2. The method, ratified as specified"
);
const COMPOSITE_MODEL_CLAIMS: [&str; 1] = ["FS-CLM-24"];
const CONSTITUTIONAL_FLOOR_CLAIMS: [&str; 2] = ["FS-CLM-38", "FS-CLM-39"];
const LOOP_HAZARDS: [&str; 5] = [
    "unbounded",
    "self-certifying",
    "deadlocking",
    "single-veto",
    "cascade",
];
const CLOSURE_STATUSES: [&str; 2] = ["bounded-unresolved", "open-blocking"];
const POWER_FUNCTIONS: [&str; 5] = [
    "decisive-fact-writer",
    "decider",
    "executor",
    "auditor",
    "final-remedy",
];
const ESTABLISHED_POSTURES: [&str; 3] = ["Derived", "Checked", "Evidenced"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Check,
    Generate,
    RefreshAndCheck,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckResult {
    pub(crate) controls: usize,
    pub(crate) message: String,
}

impl fmt::Display for CheckResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClosureError(String);

impl ClosureError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ClosureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

type ClosureResult<T> = Result<T, ClosureError>;

fn public_error(error: impl fmt::Display) -> Error {
    Error::new(format!("16-constitutional-closure: {error}"))
}

#[derive(Clone, Debug)]
struct ExpandedComponent {
    name: String,
    refs: Vec<String>,
}

#[derive(Clone, Debug)]
struct ExpandedProfile {
    id: String,
    kind: String,
    claims: Vec<String>,
    components: Vec<ExpandedComponent>,
    source: Option<String>,
}

impl ExpandedProfile {
    fn component(&self, name: &str) -> ClosureResult<&[String]> {
        self.components
            .iter()
            .find(|row| row.name == name)
            .map(|row| row.refs.as_slice())
            .ok_or_else(|| ClosureError::new(format!("{}.{} is missing", self.kind, name)))
    }
}

#[derive(Clone, Debug)]
struct FunctionResult {
    result: String,
    affected_claim_refs: Vec<String>,
    reason: String,
}

#[derive(Clone, Debug)]
struct LoopResult {
    id: String,
    kind: String,
    members: Vec<String>,
    result: String,
    statuses: Vec<(String, String)>,
    owner: String,
    affected_claim_refs: Vec<String>,
    blocking_claim_refs: Vec<String>,
}

#[derive(Clone, Debug)]
struct BottleneckResult {
    id: String,
    result: String,
    owner: String,
    reason: String,
    affected_claim_refs: Vec<String>,
    blocking_claim_refs: Vec<String>,
}

#[derive(Clone, Debug)]
struct ClaimAudit {
    id: String,
    title: String,
    result: String,
    posture: String,
    route: String,
    required_routes: Vec<String>,
    profiles: Vec<String>,
    defects: Vec<String>,
    unresolved: Vec<String>,
    receipts: Vec<String>,
    reasons: Vec<String>,
    dependencies: Vec<String>,
    scenarios: Vec<String>,
    roles: Vec<String>,
    external_assumptions: Vec<String>,
    component_coverage: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
struct Contract {
    profiles: Vec<ExpandedProfile>,
    allocations: BTreeMap<String, ModelAllocation>,
    function: FunctionResult,
    dependency_claims: BTreeMap<String, Vec<String>>,
    claims: Vec<ClaimAudit>,
    loops: Vec<LoopResult>,
    bottlenecks: Vec<BottleneckResult>,
    component_coverage: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

fn unique_strings(values: &[String], nonempty: bool) -> bool {
    (!nonempty || !values.is_empty())
        && values.iter().all(|value| !value.is_empty())
        && values.iter().collect::<HashSet<_>>().len() == values.len()
}

fn set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn static_set(values: &[&'static str]) -> BTreeSet<&'static str> {
    values.iter().copied().collect()
}

fn profile<'a>(profiles: &'a [ExpandedProfile], kind: &str) -> ClosureResult<&'a ExpandedProfile> {
    profiles
        .iter()
        .find(|row| row.kind == kind)
        .ok_or_else(|| ClosureError::new(format!("missing profile {kind}")))
}

fn source_ids(source: &LedgerDocument) -> BTreeMap<&str, &'static str> {
    let mut ids = BTreeMap::new();
    macro_rules! add {
        ($field:ident, $kind:literal) => {
            for row in &source.$field {
                ids.insert(row.id.as_str(), $kind);
            }
        };
    }
    add!(domains, "domains");
    add!(legacy_rows, "legacy_rows");
    add!(claims, "claims");
    add!(bodies, "bodies");
    add!(routes, "routes");
    add!(external_assumptions, "external_assumptions");
    add!(envelope, "envelope");
    add!(roles, "roles");
    add!(powers, "powers");
    add!(constitutional_effects, "constitutional_effects");
    add!(coverage_families, "coverage_families");
    add!(dependencies, "dependencies");
    add!(dependency_loops, "dependency_loops");
    add!(scenarios, "scenarios");
    add!(thresholds, "thresholds");
    add!(defects, "defects");
    add!(receipts, "receipts");
    add!(closure_requirement_profiles, "closure_requirement_profiles");
    add!(closure_claim_contracts, "closure_claim_contracts");
    add!(model_allocations, "model_allocations");
    add!(function_allocations, "function_allocations");
    add!(loop_hazard_controls, "loop_hazard_controls");
    add!(bottleneck_dispositions, "bottleneck_dispositions");
    ids
}

fn intrinsic_claims(kind: &str) -> &'static [&'static str] {
    match kind {
        "floor-lifecycle" => &[
            "FS-CLM-04",
            "FS-CLM-05",
            "FS-CLM-06",
            "FS-CLM-38",
            "FS-CLM-39",
        ],
        "public-power-lifecycle" => &[
            "FS-CLM-15",
            "FS-CLM-17",
            "FS-CLM-18",
            "FS-CLM-28",
            "FS-CLM-29",
            "FS-CLM-30",
            "FS-CLM-31",
            "FS-CLM-32",
        ],
        "private-duty-explicitness" => &[
            "FS-CLM-02",
            "FS-CLM-08",
            "FS-CLM-09",
            "FS-CLM-23",
            "FS-CLM-27",
            "FS-CLM-34",
        ],
        "record-lifecycle" => &[
            "FS-CLM-19",
            "FS-CLM-20",
            "FS-CLM-21",
            "FS-CLM-31",
            "FS-CLM-38",
            "FS-CLM-39",
            "FS-CLM-40",
        ],
        "democratic-floor-corridor" => &["FS-CLM-10", "FS-CLM-14", "FS-CLM-15", "FS-CLM-16"],
        "book-seam" => &["FS-CLM-03", "FS-CLM-24"],
        "external-assumption-disclosure" => &["FS-CLM-13", "FS-CLM-16"],
        "reader-claim-ownership" => &["FS-CLM-37"],
        _ => &[],
    }
}

fn allowed_refs(kind: &str, component: &str) -> &'static [&'static str] {
    match (kind, component) {
        ("floor-lifecycle", "delivery") => &["FS-DEP-25", "FS-DEP-28"],
        ("floor-lifecycle", "continuity") => &["FS-DEP-25", "FS-DEP-57"],
        ("floor-lifecycle", "remedy") => &["FS-DEP-34", "FS-DEP-43"],
        ("public-power-lifecycle", "source") => &["FS-DEP-01", "FS-DEP-05", "FS-DEP-06"],
        ("public-power-lifecycle", "limit") => &["FS-DEP-35", "FS-DEP-37"],
        ("public-power-lifecycle", "review") => {
            &["FS-DEP-44", "FS-DEP-49", "FS-DEP-50", "FS-DEP-51"]
        }
        ("public-power-lifecycle", "temporal-status") => &["FS-DEP-38", "FS-DEP-62"],
        ("private-duty-explicitness", "express-duty") => &["FS-DEP-17", "FS-DEP-18", "FS-DEP-30"],
        ("record-lifecycle", "writer") => &["FS-DEP-23", "FS-DEP-26", "FS-DEP-39"],
        ("record-lifecycle", "challenge") | ("record-lifecycle", "correction") => &["FS-DEP-45"],
        ("democratic-floor-corridor", "choice-source") => &["FS-DEP-01", "FS-DEP-29"],
        ("democratic-floor-corridor", "floor-boundary") => &["FS-CLM-04", "FS-DEP-11"],
        ("book-seam", "responsible-book") | ("book-seam", "assurance-ceiling") => {
            &["FS-RTE-02", "FS-RTE-03", "FS-RTE-05"]
        }
        ("external-assumption-disclosure", "named-assumption") => {
            &["FS-EXA-01", "FS-EXA-02", "FS-EXA-03", "FS-EXA-04"]
        }
        ("reader-claim-ownership", "formal-owner") => &["FS-CLM-37"],
        ("reader-claim-ownership", "evidentiary-owner") => &["FS-RTE-06"],
        _ => &[],
    }
}

fn expanded_profiles(source: &LedgerDocument) -> ClosureResult<Vec<ExpandedProfile>> {
    let requirement_map = REQUIREMENT_COMPONENTS
        .iter()
        .copied()
        .collect::<BTreeMap<_, _>>();
    let source_map = PROFILE_SOURCES.iter().copied().collect::<BTreeMap<_, _>>();
    let mut profiles = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut seen_kinds = HashSet::new();
    for (index, record) in source.closure_requirement_profiles.iter().enumerate() {
        let context = format!("closure_requirement_profiles[{index}]");
        if !record.id.starts_with("FS-CLR-") || !seen_ids.insert(record.id.as_str()) {
            return Err(ClosureError::new(format!(
                "{context}: stable unique FS-CLR id required"
            )));
        }
        let Some(required_components) = requirement_map.get(record.requirement_kind.as_str())
        else {
            return Err(ClosureError::new(format!(
                "{context}: unknown or duplicate kind {:?}",
                record.requirement_kind
            )));
        };
        if !seen_kinds.insert(record.requirement_kind.as_str()) {
            return Err(ClosureError::new(format!(
                "{context}: unknown or duplicate kind {:?}",
                record.requirement_kind
            )));
        }
        if !unique_strings(&record.applies_to_claim_refs, true) {
            return Err(ClosureError::new(format!(
                "{context}: unique affected claims required"
            )));
        }
        let mut components = Vec::new();
        let mut component_names = HashSet::new();
        for component in &record.components {
            if !component_names.insert(component.component.as_str())
                || !unique_strings(&component.record_refs, true)
            {
                return Err(ClosureError::new(format!(
                    "{context}.{}: one unique non-empty binding required",
                    component.component
                )));
            }
            components.push(ExpandedComponent {
                name: component.component.clone(),
                refs: component.record_refs.clone(),
            });
        }
        if component_names != required_components.iter().copied().collect::<HashSet<_>>() {
            let mut required = required_components.to_vec();
            required.sort_unstable();
            return Err(ClosureError::new(format!(
                "{context}: {} requires exactly {required:?}",
                record.requirement_kind
            )));
        }
        let expected_source = source_map[record.requirement_kind.as_str()];
        if record.source_record_ref.0.as_deref() != expected_source {
            return Err(ClosureError::new(format!(
                "{context}: wrong reviewed source record"
            )));
        }
        profiles.push(ExpandedProfile {
            id: record.id.clone(),
            kind: record.requirement_kind.clone(),
            claims: record.applies_to_claim_refs.clone(),
            components,
            source: record.source_record_ref.0.clone(),
        });
    }
    if seen_kinds != requirement_map.keys().copied().collect::<HashSet<_>>() {
        return Err(ClosureError::new(
            "every closure requirement family must occur exactly once",
        ));
    }

    let claims_by_id = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let claim_ids = claims_by_id.keys().copied().collect::<BTreeSet<_>>();
    let profile_ids = profiles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut contract_by_claim = BTreeMap::new();
    let mut contract_ids = HashSet::new();
    for (index, contract) in source.closure_claim_contracts.iter().enumerate() {
        let context = format!("closure_claim_contracts[{index}]");
        if !contract.id.starts_with("FS-CCT-") || !contract_ids.insert(contract.id.as_str()) {
            return Err(ClosureError::new(format!(
                "{context}: stable unique FS-CCT id required"
            )));
        }
        if !claims_by_id.contains_key(contract.claim_ref.as_str())
            || contract_by_claim.contains_key(contract.claim_ref.as_str())
        {
            return Err(ClosureError::new(format!(
                "{context}: one known claim required"
            )));
        }
        if !unique_strings(&contract.required_profile_refs, false)
            || !set(&contract.required_profile_refs).is_subset(&profile_ids)
        {
            return Err(ClosureError::new(format!(
                "{context}: unique known required profiles needed"
            )));
        }
        contract_by_claim.insert(
            contract.claim_ref.as_str(),
            contract
                .required_profile_refs
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
        );
    }
    if contract_by_claim.keys().copied().collect::<BTreeSet<_>>() != claim_ids {
        return Err(ClosureError::new(
            "every claim needs exactly one reviewed closure contract",
        ));
    }
    let profile_id_by_kind = profiles
        .iter()
        .map(|row| (row.kind.as_str(), row.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    for (kind, _) in REQUIREMENT_COMPONENTS {
        let unknown = intrinsic_claims(kind)
            .iter()
            .copied()
            .filter(|claim| !claim_ids.contains(claim))
            .collect::<Vec<_>>();
        if !unknown.is_empty() {
            return Err(ClosureError::new(format!(
                "{kind}: intrinsic contract names unknown claims {unknown:?}"
            )));
        }
    }
    for (claim_ref, required) in &contract_by_claim {
        let expected = REQUIREMENT_COMPONENTS
            .iter()
            .filter(|(kind, _)| intrinsic_claims(kind).contains(claim_ref))
            .map(|(kind, _)| profile_id_by_kind[kind])
            .collect::<BTreeSet<_>>();
        if *required != expected {
            return Err(ClosureError::new(format!(
                "{claim_ref}: reviewed closure contract omits or adds an intrinsic claim obligation"
            )));
        }
        if set(&claims_by_id[claim_ref].closure_requirement_refs) != *required {
            return Err(ClosureError::new(format!(
                "{claim_ref}: claim closure refs drift from its reviewed closure contract"
            )));
        }
    }
    for current in &profiles {
        let expected = contract_by_claim
            .iter()
            .filter(|(_, required)| required.contains(current.id.as_str()))
            .map(|(claim, _)| *claim)
            .collect::<BTreeSet<_>>();
        if set(&current.claims) != expected {
            return Err(ClosureError::new(format!(
                "{}: profile membership drifts from claim contracts",
                current.id
            )));
        }
    }

    let mut expected_dependency_components = source
        .dependencies
        .iter()
        .map(|row| (row.id.as_str(), BTreeSet::new()))
        .collect::<BTreeMap<_, BTreeSet<String>>>();
    for current in &profiles {
        for component in &current.components {
            let token = format!("{}:{}", current.id, component.name);
            for reference in &component.refs {
                if let Some(expected) = expected_dependency_components.get_mut(reference.as_str()) {
                    expected.insert(token.clone());
                }
            }
        }
    }
    for dependency in &source.dependencies {
        let expected = &expected_dependency_components[dependency.id.as_str()];
        if dependency
            .closure_component_refs
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != *expected
        {
            return Err(ClosureError::new(format!(
                "{}: dependency-owned closure component classification is stale",
                dependency.id
            )));
        }
    }

    let legacy = source
        .legacy_rows
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let effect_claims = source
        .constitutional_effects
        .iter()
        .flat_map(|effect| effect.affected_claim_refs.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    if !static_set(&CONSTITUTIONAL_FLOOR_CLAIMS).is_subset(&effect_claims) {
        return Err(ClosureError::new(
            "constitutional floor claims must be bound by reviewed effects",
        ));
    }
    let mut expected_floor_claims = set(&legacy["FS-LGR-02"].split_claim_refs);
    expected_floor_claims.extend(CONSTITUTIONAL_FLOOR_CLAIMS);
    if set(&profile(&profiles, "floor-lifecycle")?.claims) != expected_floor_claims {
        return Err(ClosureError::new(
            "floor profile must equal the reviewed FS-LGR-02 split claims plus source-bound constitutional standing/material-access effects",
        ));
    }
    let book2 = source
        .claims
        .iter()
        .filter(|claim| claim.layer == "book-2-operation")
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    if set(&profile(&profiles, "book-seam")?.claims) != book2 {
        return Err(ClosureError::new(
            "book-seam profile must cover every Book 2 claim",
        ));
    }
    let external = source
        .claims
        .iter()
        .filter(|claim| claim.layer == "external-assumption")
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    if set(&profile(&profiles, "external-assumption-disclosure")?.claims) != external {
        return Err(ClosureError::new(
            "external profile must cover every external claim",
        ));
    }
    let reader_owned = source
        .claims
        .iter()
        .filter(|claim| claim.owner_ref == READER_OWNER_REF)
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    if reader_owned.is_empty()
        || set(&profile(&profiles, "reader-claim-ownership")?.claims) != reader_owned
    {
        return Err(ClosureError::new(
            "reader profile must equal claims owned by the reader-evidence protocol",
        ));
    }
    Ok(profiles)
}

fn require_dependencies(
    source: &LedgerDocument,
    refs: &[String],
    context: &str,
    classes: &[&str],
    flows: &[&str],
    lifecycles: &[&str],
) -> ClosureResult<()> {
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for reference in refs {
        let Some(dependency) = dependencies.get(reference.as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: {reference} must be a dependency"
            )));
        };
        if !classes.contains(&dependency.dependency_class.as_str())
            || !flows.contains(&dependency.flow_kind.as_str())
            || !lifecycles.contains(&dependency.lifecycle_path.as_str())
        {
            return Err(ClosureError::new(format!(
                "{context}: {reference} has the wrong typed contract"
            )));
        }
    }
    Ok(())
}

fn validate_profile_bindings(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
) -> ClosureResult<()> {
    let ids = source_ids(source);
    let floor_claims = set(&profile(profiles, "floor-lifecycle")?.claims);
    for current in profiles {
        for component in &current.components {
            let unknown = component
                .refs
                .iter()
                .filter(|reference| !ids.contains_key(reference.as_str()))
                .collect::<Vec<_>>();
            if !unknown.is_empty() {
                return Err(ClosureError::new(format!(
                    "{}.{}: unknown refs {unknown:?}",
                    current.kind, component.name
                )));
            }
            let allowed = allowed_refs(&current.kind, &component.name);
            if component
                .refs
                .iter()
                .any(|reference| !allowed.contains(&reference.as_str()))
            {
                return Err(ClosureError::new(format!(
                    "{}.{}: binding is outside its reviewed semantic edge contract",
                    current.kind, component.name
                )));
            }
        }
    }
    let p = |kind| profile(profiles, kind);
    require_dependencies(
        source,
        p("floor-lifecycle")?.component("delivery")?,
        "floor.delivery",
        &["operationally-supplied"],
        &["services"],
        &["right"],
    )?;
    require_dependencies(
        source,
        p("floor-lifecycle")?.component("continuity")?,
        "floor.continuity",
        &["operationally-supplied", "constitutionally-guaranteed"],
        &["services", "accountability"],
        &["right"],
    )?;
    require_dependencies(
        source,
        p("floor-lifecycle")?.component("remedy")?,
        "floor.remedy",
        &["operationally-supplied", "constitutionally-guaranteed"],
        &["information", "claims"],
        &["right"],
    )?;
    require_dependencies(
        source,
        p("public-power-lifecycle")?.component("source")?,
        "power.source",
        &["democratically-selected", "constitutionally-guaranteed"],
        &["authority"],
        &["power"],
    )?;
    require_dependencies(
        source,
        p("public-power-lifecycle")?.component("limit")?,
        "power.limit",
        &["constitutionally-guaranteed"],
        &["information"],
        &["power", "record"],
    )?;
    require_dependencies(
        source,
        p("public-power-lifecycle")?.component("review")?,
        "power.review",
        &["constitutionally-guaranteed"],
        &["claims", "accountability"],
        &["power", "record"],
    )?;
    require_dependencies(
        source,
        p("public-power-lifecycle")?.component("temporal-status")?,
        "power.temporal",
        &["externally-assumed", "constitutionally-guaranteed"],
        &["information", "accountability"],
        &["power"],
    )?;
    require_dependencies(
        source,
        p("private-duty-explicitness")?.component("express-duty")?,
        "private-duty",
        &["constitutionally-guaranteed", "operationally-supplied"],
        &["care", "services"],
        &["right"],
    )?;
    let record = p("record-lifecycle")?;
    require_dependencies(
        source,
        record.component("writer")?,
        "record.writer",
        &["externally-assumed", "operationally-supplied"],
        &["resources", "services", "information"],
        &["record"],
    )?;
    if !source.constitutional_effects.is_empty()
        && !record
            .component("writer")?
            .iter()
            .any(|value| value == "FS-DEP-26")
    {
        return Err(ClosureError::new(
            "constitutional identity effects require the reviewed registration and vital-record writer edge",
        ));
    }
    for component in ["challenge", "correction"] {
        require_dependencies(
            source,
            record.component(component)?,
            &format!("record.{component}"),
            &["constitutionally-guaranteed"],
            &["claims"],
            &["record"],
        )?;
    }
    let democratic = p("democratic-floor-corridor")?;
    require_dependencies(
        source,
        democratic.component("choice-source")?,
        "democracy.choice",
        &["democratically-selected", "operationally-supplied"],
        &["authority", "services"],
        &["power", "record"],
    )?;
    let boundary = democratic.component("floor-boundary")?;
    let boundary_claims = boundary
        .iter()
        .filter(|reference| ids.get(reference.as_str()) == Some(&"claims"))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let boundary_dependencies = boundary
        .iter()
        .filter(|reference| ids.get(reference.as_str()) == Some(&"dependencies"))
        .cloned()
        .collect::<Vec<_>>();
    if boundary_claims.is_empty() || !boundary_claims.is_subset(&floor_claims) {
        return Err(ClosureError::new(
            "democracy.floor-boundary must cite a floor claim",
        ));
    }
    require_dependencies(
        source,
        &boundary_dependencies,
        "democracy.floor-boundary",
        &["constitutionally-guaranteed"],
        &["money"],
        &["right"],
    )?;
    for component in &p("book-seam")?.components {
        if component
            .refs
            .iter()
            .any(|reference| ids.get(reference.as_str()) != Some(&"routes"))
        {
            return Err(ClosureError::new("book-seam components must name routes"));
        }
    }
    let external_refs = set(p("external-assumption-disclosure")?.component("named-assumption")?);
    let assumptions = source
        .external_assumptions
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    if external_refs != assumptions {
        return Err(ClosureError::new(
            "external profile must name every reviewed assumption",
        ));
    }
    let reader = p("reader-claim-ownership")?;
    if reader
        .component("formal-owner")?
        .iter()
        .any(|reference| ids.get(reference.as_str()) != Some(&"claims"))
    {
        return Err(ClosureError::new("reader formal-owner must name claims"));
    }
    if reader
        .component("evidentiary-owner")?
        .iter()
        .any(|reference| ids.get(reference.as_str()) != Some(&"routes"))
    {
        return Err(ClosureError::new(
            "reader evidentiary-owner must name routes",
        ));
    }
    Ok(())
}

fn endpoint_domains<'a>(source: &'a LedgerDocument, reference: &str) -> BTreeSet<&'a str> {
    if let Some(domain) = source.domains.iter().find(|row| row.id == reference) {
        return BTreeSet::from([domain.id.as_str()]);
    }
    if let Some(role) = source.roles.iter().find(|row| row.id == reference) {
        return role.domain_refs.iter().map(String::as_str).collect();
    }
    if reference.starts_with("FS-BOD-") {
        return source
            .domains
            .iter()
            .filter(|domain| domain.bodies_refs.iter().any(|item| item == reference))
            .map(|domain| domain.id.as_str())
            .collect();
    }
    if reference.starts_with("FS-EXA-") {
        return source
            .domains
            .iter()
            .filter(|domain| {
                domain
                    .external_assumption_refs
                    .iter()
                    .any(|item| item == reference)
            })
            .map(|domain| domain.id.as_str())
            .collect();
    }
    BTreeSet::new()
}

fn relevant_external_assumptions(source: &LedgerDocument, claim: &Claim) -> Vec<String> {
    let domains = source
        .domains
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    claim
        .domain_refs
        .iter()
        .filter_map(|domain| domains.get(domain.as_str()))
        .flat_map(|domain| domain.external_assumption_refs.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn claim_component_coverage(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
) -> ClosureResult<BTreeMap<String, BTreeMap<String, Vec<String>>>> {
    let ids = source_ids(source);
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut result = source
        .claims
        .iter()
        .map(|claim| (claim.id.clone(), BTreeMap::new()))
        .collect::<BTreeMap<_, _>>();
    for current in profiles {
        for claim_ref in &current.claims {
            let claim = claims
                .get(claim_ref.as_str())
                .ok_or_else(|| ClosureError::new(format!("unknown claim {claim_ref}")))?;
            let claim_domains = set(&claim.domain_refs);
            let relevant_external = relevant_external_assumptions(source, claim)
                .into_iter()
                .collect::<BTreeSet<_>>();
            for component in &current.components {
                let key = format!("{}.{}", current.kind, component.name);
                let mut bindings = Vec::new();
                for reference in &component.refs {
                    match ids.get(reference.as_str()).copied() {
                        Some("dependencies") => {
                            let dependency = dependencies[reference.as_str()];
                            let mut domains = endpoint_domains(source, &dependency.from_ref);
                            domains.extend(endpoint_domains(source, &dependency.to_ref));
                            if !claim_domains.is_disjoint(&domains) {
                                bindings.push(reference.clone());
                            }
                        }
                        Some("external_assumptions") => {
                            if relevant_external.contains(reference) {
                                bindings.push(reference.clone());
                            }
                        }
                        Some(_)
                            if current.kind == "reader-claim-ownership"
                                && component.name == "formal-owner" =>
                        {
                            if reference == claim_ref {
                                bindings.push(reference.clone());
                            }
                        }
                        Some(_) => bindings.push(reference.clone()),
                        None => {}
                    }
                }
                result
                    .get_mut(claim_ref)
                    .expect("profile claim initialized")
                    .insert(key, bindings);
            }
        }
    }
    Ok(result)
}

fn validate_component_consumption(
    profiles: &[ExpandedProfile],
    component_coverage: &BTreeMap<String, BTreeMap<String, Vec<String>>>,
) -> ClosureResult<()> {
    for current in profiles {
        for component in &current.components {
            let key = format!("{}.{}", current.kind, component.name);
            for reference in &component.refs {
                if !reference.starts_with("FS-DEP-") {
                    continue;
                }
                let consumed = current.claims.iter().any(|claim_ref| {
                    component_coverage[claim_ref][&key]
                        .iter()
                        .any(|value| value == reference)
                });
                if !consumed {
                    return Err(ClosureError::new(format!(
                        "{key}: {reference} has no claim-scoped consumer"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn dependency_blocking_scope(
    component_coverage: &BTreeMap<String, BTreeMap<String, Vec<String>>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut result = BTreeMap::<String, BTreeSet<String>>::new();
    for (claim_ref, components) in component_coverage {
        for refs in components.values() {
            for reference in refs {
                if reference.starts_with("FS-DEP-") {
                    result
                        .entry(reference.clone())
                        .or_default()
                        .insert(claim_ref.clone());
                }
            }
        }
    }
    result
}

fn profile_membership(profiles: &[ExpandedProfile]) -> BTreeMap<String, Vec<String>> {
    let mut result = BTreeMap::<String, Vec<String>>::new();
    for current in profiles {
        for claim in &current.claims {
            result
                .entry(claim.clone())
                .or_default()
                .push(current.id.clone());
        }
    }
    result
}

fn validate_model_allocations(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
) -> ClosureResult<BTreeMap<String, ModelAllocation>> {
    let claims = source
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let route_ids = source
        .routes
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let membership = profile_membership(profiles);
    let reader_claims = set(&profile(profiles, "reader-claim-ownership")?.claims);
    let external_claims = set(&profile(profiles, "external-assumption-disclosure")?.claims);
    let book_claims = set(&profile(profiles, "book-seam")?.claims);
    let record_claims = set(&profile(profiles, "record-lifecycle")?.claims);
    let mut by_claim = BTreeMap::new();
    for (index, row) in source.model_allocations.iter().enumerate() {
        let context = format!("model_allocations[{index}]");
        let Some(claim) = claims.get(row.claim_ref.as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: one known claim required"
            )));
        };
        if by_claim.contains_key(&row.claim_ref) {
            return Err(ClosureError::new(format!(
                "{context}: one known claim required"
            )));
        }
        if !unique_strings(&row.required_route_refs, true)
            || row
                .required_route_refs
                .iter()
                .any(|route| !route_ids.contains(route.as_str()))
        {
            return Err(ClosureError::new(format!(
                "{context}: unique known required routes needed"
            )));
        }
        if row.primary_route_ref != claim.route_ref
            || !row.required_route_refs.contains(&claim.route_ref)
        {
            return Err(ClosureError::new(format!(
                "{context}: primary route drift or substitution"
            )));
        }
        let expected_required: &[&str];
        if claim.layer == "external-assumption" || external_claims.contains(row.claim_ref.as_str())
        {
            if claim.posture != "Unestablished" {
                return Err(ClosureError::new(format!(
                    "{context}: external claims must remain Unestablished"
                )));
            }
            expected_required = &["FS-RTE-05"];
        } else if claim.layer == "book-2-operation" || book_claims.contains(row.claim_ref.as_str())
        {
            if claim.posture != "Unestablished" {
                return Err(ClosureError::new(format!(
                    "{context}: Book 2 operation claims must remain Unestablished"
                )));
            }
            expected_required = if COMPOSITE_MODEL_CLAIMS.contains(&row.claim_ref.as_str()) {
                &["FS-RTE-02", "FS-RTE-03", "FS-RTE-05"]
            } else {
                &["FS-RTE-05"]
            };
        } else if reader_claims.contains(row.claim_ref.as_str()) {
            expected_required = &["FS-RTE-06"];
        } else if ["Derived", "Checked", "Specified", "Reasoned"].contains(&claim.posture.as_str())
        {
            expected_required = &["FS-RTE-01"];
        } else if claim.overlay == "liveness" || record_claims.contains(row.claim_ref.as_str()) {
            expected_required = &["FS-RTE-05"];
        } else {
            return Err(ClosureError::new(format!(
                "{context}: unestablished claim has no reviewed model contract"
            )));
        }
        if row
            .required_route_refs
            .iter()
            .map(String::as_str)
            .ne(expected_required.iter().copied())
            || claim.route_ref != expected_required[0]
        {
            return Err(ClosureError::new(format!(
                "{context}: required route composition drift"
            )));
        }
        let expected_profiles = membership
            .get(&row.claim_ref)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if !unique_strings(&row.closure_profile_refs, false)
            || set(&row.closure_profile_refs) != set(expected_profiles)
        {
            return Err(ClosureError::new(format!(
                "{context}: closure profile inverse mapping is stale"
            )));
        }
        by_claim.insert(row.claim_ref.clone(), row.clone());
    }
    if by_claim.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != claims.keys().copied().collect::<BTreeSet<_>>()
    {
        return Err(ClosureError::new(
            "model allocation must cover every claim exactly once",
        ));
    }
    let allocated_reader_claims = by_claim
        .iter()
        .filter(|(_, row)| {
            row.required_route_refs
                .iter()
                .any(|route| route == "FS-RTE-06")
        })
        .map(|(claim, _)| claim.as_str())
        .collect::<BTreeSet<_>>();
    let reader = profile(profiles, "reader-claim-ownership")?;
    if set(&reader.claims) != allocated_reader_claims {
        return Err(ClosureError::new(
            "reader ownership must cover every R6 claim",
        ));
    }
    if set(reader.component("formal-owner")?) != allocated_reader_claims {
        return Err(ClosureError::new("reader formal-owner claim set is stale"));
    }
    let reader_routes = allocated_reader_claims
        .iter()
        .flat_map(|claim| {
            by_claim[*claim]
                .required_route_refs
                .iter()
                .map(String::as_str)
        })
        .collect::<BTreeSet<_>>();
    if set(reader.component("evidentiary-owner")?) != reader_routes {
        return Err(ClosureError::new(
            "reader evidentiary-owner route set is stale",
        ));
    }
    let book = profile(profiles, "book-seam")?;
    let book_routes = book
        .claims
        .iter()
        .flat_map(|claim| {
            by_claim[claim]
                .required_route_refs
                .iter()
                .map(String::as_str)
        })
        .collect::<BTreeSet<_>>();
    for component in &book.components {
        if set(&component.refs) != book_routes {
            return Err(ClosureError::new(
                "book-seam must bind every required model route",
            ));
        }
    }
    Ok(by_claim)
}

fn function_body_refs<'a>(row: &'a FunctionAllocation, function: &str) -> &'a [String] {
    match function {
        "decisive-fact-writer" => &row.decisive_fact_writer_body_refs,
        "decider" => &row.decider_body_refs,
        "executor" => &row.executor_body_refs,
        "auditor" => &row.auditor_body_refs,
        "final-remedy" => &row.final_remedy_body_refs,
        _ => &[],
    }
}

fn function_role_refs<'a>(row: &'a FunctionAllocation, function: &str) -> &'a [String] {
    match function {
        "decisive-fact-writer" => &row.decisive_fact_writer_role_refs,
        "decider" => &row.decider_role_refs,
        "executor" => &row.executor_role_refs,
        "auditor" => &row.auditor_role_refs,
        "final-remedy" => &row.final_remedy_role_refs,
        _ => &[],
    }
}

fn validate_function_separation_row(
    row: &FunctionAllocation,
    source: &LedgerDocument,
    power: &Power,
    context: &str,
) -> ClosureResult<()> {
    let body_ids = source
        .bodies
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let role_ids = source
        .roles
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let claim_ids = source
        .claims
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    if row.affected_claim_refs != power.affected_claim_refs
        || row
            .affected_claim_refs
            .iter()
            .any(|reference| !claim_ids.contains(reference.as_str()))
    {
        return Err(ClosureError::new(format!(
            "{context}: affected claims must equal the power card"
        )));
    }
    let mut body_sets = BTreeMap::<&str, BTreeSet<&str>>::new();
    for function in POWER_FUNCTIONS {
        let stem = function.replace('-', "_");
        let bodies = function_body_refs(row, function);
        let roles = function_role_refs(row, function);
        if !unique_strings(bodies, true)
            || bodies
                .iter()
                .any(|reference| !body_ids.contains(reference.as_str()))
        {
            return Err(ClosureError::new(format!(
                "{context}.{stem}_body_refs: unique known body refs required"
            )));
        }
        if !unique_strings(roles, true)
            || roles
                .iter()
                .any(|reference| !role_ids.contains(reference.as_str()))
        {
            return Err(ClosureError::new(format!(
                "{context}.{stem}_role_refs: unique known role refs required"
            )));
        }
        body_sets.insert(function, bodies.iter().map(String::as_str).collect());
    }
    if row.separation_constraints.len() != power.required_separation_pairs.len() {
        return Err(ClosureError::new(format!(
            "{context}: one source-backed constraint is required per pair"
        )));
    }
    for (constraint, pair) in row
        .separation_constraints
        .iter()
        .zip(&power.required_separation_pairs)
    {
        if constraint.functions != *pair || constraint.reason.is_empty() {
            return Err(ClosureError::new(format!(
                "{context}: separation constraint differs from power card"
            )));
        }
        if pair.len() != 2 {
            return Err(ClosureError::new(format!(
                "{context}: separation constraint differs from power card"
            )));
        }
        let Some(left) = body_sets.get(pair[0].as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: separation constraint differs from power card"
            )));
        };
        let Some(right) = body_sets.get(pair[1].as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: separation constraint differs from power card"
            )));
        };
        if !left.is_disjoint(right) {
            return Err(ClosureError::new(format!(
                "{context}: required body functions are fused"
            )));
        }
    }
    let mut all = body_sets[POWER_FUNCTIONS[0]].clone();
    for function in &POWER_FUNCTIONS[1..] {
        all.retain(|body| body_sets[function].contains(body));
    }
    if !all.is_empty() {
        return Err(ClosureError::new(format!(
            "{context}: one body self-certifies all five functions"
        )));
    }
    if !unique_strings(&row.source_refs, true) {
        return Err(ClosureError::new(format!(
            "{context}.source_refs: unique exact refs required"
        )));
    }
    Ok(())
}

fn validate_function_allocations(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
) -> ClosureResult<FunctionResult> {
    let mut expected_claims = set(&profile(profiles, "public-power-lifecycle")?.claims)
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let powers = source
        .powers
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for (index, row) in source.function_allocations.iter().enumerate() {
        let context = format!("function_allocations[{index}]");
        let Some(power) = powers.get(row.power_ref.as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: power_ref must name an FS-POW card"
            )));
        };
        validate_function_separation_row(row, source, power, &context)?;
        if !seen.insert(row.power_ref.as_str()) {
            return Err(ClosureError::new(format!(
                "{context}: duplicate power-bound allocation"
            )));
        }
        expected_claims.extend(row.affected_claim_refs.iter().cloned());
    }
    if seen != powers.keys().copied().collect::<BTreeSet<_>>() {
        return Err(ClosureError::new(
            "function allocations must be a complete power-card bijection",
        ));
    }
    let powers_deferred = source
        .deferred_populations
        .iter()
        .any(|row| row.record_type == "powers");
    if source.power_population.status == "complete" {
        if powers_deferred || source.powers.len() != EXPECTED_POWER_COUNT {
            return Err(ClosureError::new(
                "complete power population must remove its deferral and bind every source-derived power",
            ));
        }
        return Ok(FunctionResult {
            result: "pass".to_owned(),
            affected_claim_refs: expected_claims.into_iter().collect(),
            reason: concat!(
                "all source-derived FS-POW cards have one typed, power-bound function allocation; ",
                "structural separation establishes no operation or institutional independence"
            )
            .to_owned(),
        });
    }
    if !powers_deferred {
        return Err(ClosureError::new(
            "the powers deferral must remain through foundation and partial prefixes",
        ));
    }
    Ok(FunctionResult {
        result: "bounded-unresolved".to_owned(),
        affected_claim_refs: expected_claims.into_iter().collect(),
        reason: format!(
            "source-derived power population is {}: {} cards and {} matching allocations; the remaining families stay explicitly deferred",
            source.power_population.status,
            source.powers.len(),
            source.function_allocations.len()
        ),
    })
}

fn dependency_claim_map(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
) -> BTreeMap<String, Vec<String>> {
    let mut profile_claims = BTreeMap::<String, BTreeSet<String>>::new();
    for current in profiles {
        for component in &current.components {
            for reference in &component.refs {
                if reference.starts_with("FS-DEP-") {
                    profile_claims
                        .entry(reference.clone())
                        .or_default()
                        .extend(current.claims.iter().cloned());
                }
            }
        }
    }
    source
        .dependencies
        .iter()
        .map(|dependency| {
            let mut affected = profile_claims
                .get(&dependency.id)
                .cloned()
                .unwrap_or_default();
            let mut domains = endpoint_domains(source, &dependency.from_ref);
            domains.extend(endpoint_domains(source, &dependency.to_ref));
            for claim in &source.claims {
                if claim
                    .domain_refs
                    .iter()
                    .any(|domain| domains.contains(domain.as_str()))
                {
                    affected.insert(claim.id.clone());
                }
            }
            (dependency.id.clone(), affected.into_iter().collect())
        })
        .collect()
}

fn severity_class(defect: &Defect) -> ClosureResult<&str> {
    for class in ["critical", "material", "minor"] {
        if defect
            .severity
            .strip_prefix(class)
            .is_some_and(|suffix| suffix.starts_with(" — "))
        {
            return Ok(class);
        }
    }
    Err(ClosureError::new(format!(
        "defect {}: severity must carry a class prefix",
        defect.id
    )))
}

fn validate_dependencies_and_scenarios(
    source: &LedgerDocument,
    profiles: &[ExpandedProfile],
    resolution: &BTreeMap<String, DefectResolution>,
    blocking_scope: &BTreeMap<String, BTreeSet<String>>,
) -> ClosureResult<BTreeMap<String, Vec<String>>> {
    let ids = source_ids(source);
    let affected = dependency_claim_map(source, profiles);
    let defects = source
        .defects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for dependency in &source.dependencies {
        let sat = &dependency.structural_satisfiability;
        let expected_status = match dependency.dependency_class.as_str() {
            "constitutionally-guaranteed" | "democratically-selected" => "specified-interface",
            "operationally-supplied" => "operation-deferred",
            "externally-assumed" => "external-contingent",
            _ => "",
        };
        if sat.satisfiability_status != expected_status
            && sat.satisfiability_status != "unsatisfiable"
        {
            return Err(ClosureError::new(format!(
                "{}: {} requires {expected_status:?} or an explicit unsatisfiable disposition",
                dependency.id, dependency.dependency_class
            )));
        }
        if !unique_strings(&sat.defect_refs, false) {
            return Err(ClosureError::new(format!(
                "{}: satisfiability defect_refs must be unique",
                dependency.id
            )));
        }
        if sat.satisfiability_status == "unsatisfiable" && sat.defect_refs.is_empty() {
            return Err(ClosureError::new(format!(
                "{}: unsatisfiable requires a named defect",
                dependency.id
            )));
        }
        if sat.satisfiability_status != "unsatisfiable" && !sat.defect_refs.is_empty() {
            return Err(ClosureError::new(format!(
                "{}: only unsatisfiable may carry defect_refs",
                dependency.id
            )));
        }
        if sat.satisfiability_status == "unsatisfiable" {
            let mut cited_claims = BTreeSet::new();
            for defect_ref in &sat.defect_refs {
                let Some(defect) = defects.get(defect_ref.as_str()) else {
                    return Err(ClosureError::new(format!(
                        "{}: unsatisfiable requires a named defect",
                        dependency.id
                    )));
                };
                let Some(generated) = resolution.get(defect_ref) else {
                    return Err(ClosureError::new(format!(
                        "{}: unsatisfiable requires a named defect",
                        dependency.id
                    )));
                };
                if severity_class(defect)? != "critical" || !generated.blocking {
                    return Err(ClosureError::new(format!(
                        "{}: unsatisfiable must cite a blocking critical defect",
                        dependency.id
                    )));
                }
                cited_claims.insert(defect.affected_claim_ref.clone());
            }
            if !cited_claims.is_subset(
                blocking_scope
                    .get(&dependency.id)
                    .unwrap_or(&BTreeSet::new()),
            ) {
                return Err(ClosureError::new(format!(
                    "{}: unsatisfiable blockers must stay within its claim-scoped component bindings",
                    dependency.id
                )));
            }
        }
    }
    for scenario in &source.scenarios {
        if ids.get(scenario.steward_ref.as_str()) != Some(&"bodies") {
            return Err(ClosureError::new(format!(
                "{}: scenario is unowned",
                scenario.id
            )));
        }
        for (name, value) in [
            ("ordinary_route", scenario.ordinary_route.as_str()),
            ("failure_route", scenario.failure_route.as_str()),
            ("recovery_route", scenario.recovery_route.as_str()),
        ] {
            if value.is_empty() {
                return Err(ClosureError::new(format!(
                    "{}: scenario lacks {name}",
                    scenario.id
                )));
            }
        }
    }
    Ok(affected)
}

fn validate_control_status(
    status: &str,
    controls: &[String],
    defect_refs: &[String],
    context: &str,
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
    affected_claims: &[String],
) -> ClosureResult<BTreeSet<String>> {
    if !CLOSURE_STATUSES.contains(&status) {
        return Err(ClosureError::new(format!(
            "{context}: closure_status must remain bounded-unresolved or open-blocking until a typed executable control receipt exists"
        )));
    }
    if status == "bounded-unresolved" {
        if !controls.is_empty() || !defect_refs.is_empty() {
            return Err(ClosureError::new(format!(
                "{context}: bounded-unresolved has no proof or blocker"
            )));
        }
        return Ok(BTreeSet::new());
    }
    if !controls.is_empty() || defect_refs.is_empty() {
        return Err(ClosureError::new(format!(
            "{context}: open-blocking requires defects only"
        )));
    }
    let defects = source
        .defects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut cited_claims = BTreeSet::new();
    for reference in defect_refs {
        let Some(defect) = defects.get(reference.as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: blocking defect is ineligible"
            )));
        };
        if severity_class(defect)? != "critical"
            || !resolution.get(reference).is_some_and(|row| row.blocking)
        {
            return Err(ClosureError::new(format!(
                "{context}: blocking defect is ineligible"
            )));
        }
        cited_claims.insert(defect.affected_claim_ref.clone());
    }
    let affected = affected_claims.iter().cloned().collect::<BTreeSet<_>>();
    if !cited_claims.is_subset(&affected) {
        return Err(ClosureError::new(format!(
            "{context}: blocking defects must stay within claim-scoped component bindings"
        )));
    }
    Ok(cited_claims)
}

fn validate_loop_controls(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
    dependency_claims: &BTreeMap<String, Vec<String>>,
    blocking_scope: &BTreeMap<String, BTreeSet<String>>,
) -> ClosureResult<Vec<LoopResult>> {
    let loops = source
        .dependency_loops
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut by_loop = BTreeSet::new();
    let mut rows = Vec::new();
    for (index, control) in source.loop_hazard_controls.iter().enumerate() {
        let context = format!("loop_hazard_controls[{index}]");
        let Some(current_loop) = loops.get(control.loop_ref.as_str()) else {
            return Err(ClosureError::new(format!(
                "{context}: one known loop required"
            )));
        };
        if !by_loop.insert(control.loop_ref.as_str()) {
            return Err(ClosureError::new(format!(
                "{context}: one known loop required"
            )));
        }
        let expected_claims = current_loop
            .member_edge_refs
            .iter()
            .flat_map(|dependency| dependency_claims[dependency].iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let eligible_blocking_claims = current_loop
            .member_edge_refs
            .iter()
            .filter_map(|dependency| blocking_scope.get(dependency))
            .flat_map(|claims| claims.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if control.affected_claim_refs != expected_claims {
            return Err(ClosureError::new(format!(
                "{context}: affected-claim binding is stale"
            )));
        }
        if control.assessments.len() != LOOP_HAZARDS.len() {
            return Err(ClosureError::new(format!(
                "{context}: every hazard must be assessed"
            )));
        }
        let mut seen = BTreeSet::new();
        let mut statuses = Vec::new();
        let mut blocking_claims = BTreeSet::new();
        for assessment in &control.assessments {
            if !LOOP_HAZARDS.contains(&assessment.hazard.as_str())
                || !seen.insert(assessment.hazard.as_str())
                || assessment.reason.is_empty()
            {
                return Err(ClosureError::new(format!(
                    "{context}: invalid hazard assessment"
                )));
            }
            blocking_claims.extend(validate_control_status(
                &assessment.closure_status,
                &assessment.control_refs,
                &assessment.defect_refs,
                &format!("{context}.{}", assessment.hazard),
                source,
                resolution,
                &eligible_blocking_claims,
            )?);
            statuses.push((assessment.hazard.clone(), assessment.closure_status.clone()));
        }
        let result = if statuses.iter().any(|(_, status)| status == "open-blocking") {
            "block"
        } else if statuses
            .iter()
            .any(|(_, status)| status == "bounded-unresolved")
        {
            "bounded-unresolved"
        } else {
            "pass"
        };
        rows.push(LoopResult {
            id: current_loop.id.clone(),
            kind: current_loop.loop_kind.clone(),
            members: current_loop.member_edge_refs.clone(),
            result: result.to_owned(),
            statuses,
            owner: current_loop.owner_ref.clone(),
            affected_claim_refs: expected_claims,
            blocking_claim_refs: blocking_claims.into_iter().collect(),
        });
    }
    if by_loop != loops.keys().copied().collect::<BTreeSet<_>>() {
        return Err(ClosureError::new(
            "every stable loop needs exactly one hazard-control row",
        ));
    }
    Ok(rows)
}

fn validate_bottlenecks(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
    dependency_claims: &BTreeMap<String, Vec<String>>,
    blocking_scope: &BTreeMap<String, BTreeSet<String>>,
) -> ClosureResult<Vec<BottleneckResult>> {
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let candidates = source
        .dependencies
        .iter()
        .filter(|row| matches!(row.alternate_route, AlternateRoute::Absent(_)))
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for (index, row) in source.bottleneck_dispositions.iter().enumerate() {
        let context = format!("bottleneck_dispositions[{index}]");
        if !candidates.contains(row.dependency_ref.as_str())
            || !seen.insert(row.dependency_ref.as_str())
            || row.reason.is_empty()
        {
            return Err(ClosureError::new(format!(
                "{context}: one current candidate required"
            )));
        }
        if row.affected_claim_refs != dependency_claims[&row.dependency_ref] {
            return Err(ClosureError::new(format!(
                "{context}: affected-claim binding is stale"
            )));
        }
        let eligible = blocking_scope
            .get(&row.dependency_ref)
            .map(|claims| claims.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let blocking_claims = validate_control_status(
            &row.closure_status,
            &row.control_refs,
            &row.defect_refs,
            &context,
            source,
            resolution,
            &eligible,
        )?;
        let current_dependency = dependencies[row.dependency_ref.as_str()];
        result.push(BottleneckResult {
            id: row.dependency_ref.clone(),
            result: if row.closure_status == "open-blocking" {
                "block".to_owned()
            } else if row.closure_status == "bounded-unresolved" {
                "bounded-unresolved".to_owned()
            } else {
                "pass".to_owned()
            },
            owner: current_dependency.owner_ref.clone(),
            reason: row.reason.clone(),
            affected_claim_refs: row.affected_claim_refs.clone(),
            blocking_claim_refs: blocking_claims.into_iter().collect(),
        });
    }
    if seen != candidates {
        return Err(ClosureError::new(
            "bottleneck dispositions must exactly cover no-alternate edges",
        ));
    }
    Ok(result)
}

fn relevant_scenarios(source: &LedgerDocument, claim: &Claim) -> Vec<String> {
    let domains = set(&claim.domain_refs);
    source
        .scenarios
        .iter()
        .filter(|scenario| {
            scenario
                .domain_refs
                .iter()
                .any(|domain| domains.contains(domain.as_str()))
        })
        .map(|scenario| scenario.id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn relevant_roles(source: &LedgerDocument, claim: &Claim) -> Vec<String> {
    let domains = set(&claim.domain_refs);
    source
        .roles
        .iter()
        .filter(|role| {
            role.domain_refs
                .iter()
                .any(|domain| domains.contains(domain.as_str()))
        })
        .map(|role| role.id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn compute_claim_audit(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
    allocations: &BTreeMap<String, ModelAllocation>,
    function: &FunctionResult,
    dependency_claims: &BTreeMap<String, Vec<String>>,
    loops: &[LoopResult],
    bottlenecks: &[BottleneckResult],
    component_coverage: &BTreeMap<String, BTreeMap<String, Vec<String>>>,
) -> ClosureResult<Vec<ClaimAudit>> {
    let routes = source
        .routes
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let dependencies = source
        .dependencies
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let defects_by_id = source
        .defects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut defects_by_claim = BTreeMap::<&str, Vec<&Defect>>::new();
    for defect in &source.defects {
        defects_by_claim
            .entry(defect.affected_claim_ref.as_str())
            .or_default()
            .push(defect);
    }
    let mut receipts_by_defect = BTreeMap::<&str, Vec<&super::ResolutionReceipt>>::new();
    for receipt in &source.receipts {
        receipts_by_defect
            .entry(receipt.defect_row_ref.as_str())
            .or_default()
            .push(receipt);
    }
    let mut dep_by_claim = source
        .claims
        .iter()
        .map(|claim| (claim.id.as_str(), Vec::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for (dependency, claims) in dependency_claims {
        for claim in claims {
            dep_by_claim
                .get_mut(claim.as_str())
                .expect("validated dependency claim")
                .push(dependency.clone());
        }
    }
    let mut rows = Vec::new();
    for claim in &source.claims {
        let mut blockers = Vec::new();
        let mut bounded = Vec::new();
        let claim_components = component_coverage
            .get(&claim.id)
            .ok_or_else(|| ClosureError::new(format!("{} lacks component coverage", claim.id)))?;
        let missing_components = claim_components
            .iter()
            .filter(|(_, refs)| refs.is_empty())
            .map(|(component, _)| component.clone())
            .collect::<Vec<_>>();
        if !missing_components.is_empty() {
            blockers.push(format!(
                "missing claim-scoped closure components: {}",
                missing_components.join(", ")
            ));
        }
        let defects = defects_by_claim
            .get(claim.id.as_str())
            .cloned()
            .unwrap_or_default();
        let critical = defects
            .iter()
            .filter(|defect| resolution[&defect.id].blocking)
            .map(|defect| defect.id.clone())
            .collect::<Vec<_>>();
        let unresolved = defects
            .iter()
            .filter(|defect| resolution[&defect.id].resolution_status == "unresolved-for-claim")
            .map(|defect| defect.id.clone())
            .collect::<Vec<_>>();
        if !critical.is_empty() {
            blockers.push(format!(
                "critical unresolved defects: {}",
                critical.join(", ")
            ));
        } else if !unresolved.is_empty() {
            bounded.push(format!(
                "claim-relative unresolved defects: {}",
                unresolved.join(", ")
            ));
        }
        for dependency_ref in &dep_by_claim[claim.id.as_str()] {
            let sat = &dependencies[dependency_ref.as_str()].structural_satisfiability;
            if sat.satisfiability_status == "unsatisfiable" {
                let scoped_claims = sat
                    .defect_refs
                    .iter()
                    .filter_map(|reference| defects_by_id.get(reference.as_str()))
                    .map(|defect| defect.affected_claim_ref.as_str())
                    .collect::<BTreeSet<_>>();
                if scoped_claims.contains(claim.id.as_str()) {
                    blockers.push(format!("{dependency_ref} is structurally unsatisfiable"));
                } else {
                    bounded.push(format!(
                        "{dependency_ref} is structurally unsatisfiable outside this claim's cited blocker scope"
                    ));
                }
            } else if ["operation-deferred", "external-contingent"]
                .contains(&sat.satisfiability_status.as_str())
            {
                bounded.push(format!(
                    "{dependency_ref} remains {}",
                    sat.satisfiability_status
                ));
            }
        }
        for row in loops {
            if !row.affected_claim_refs.contains(&claim.id) {
                continue;
            }
            if row.result == "block" {
                if row.blocking_claim_refs.contains(&claim.id) {
                    blockers.push(format!("{} has an open blocking hazard", row.id));
                } else {
                    bounded.push(format!(
                        "{} has an open hazard outside this claim's cited blocker scope",
                        row.id
                    ));
                }
            } else if row.result == "bounded-unresolved"
                || row
                    .statuses
                    .iter()
                    .any(|(_, status)| status == "bounded-unresolved")
            {
                bounded.push(format!("{} hazards remain bounded-unresolved", row.id));
            }
        }
        for row in bottlenecks {
            if !row.affected_claim_refs.contains(&claim.id) {
                continue;
            }
            if row.result == "block" {
                if row.blocking_claim_refs.contains(&claim.id) {
                    blockers.push(format!("{} is an open blocking bottleneck", row.id));
                } else {
                    bounded.push(format!(
                        "{} is an open bottleneck outside this claim's cited blocker scope",
                        row.id
                    ));
                }
            } else if row.result == "bounded-unresolved" {
                bounded.push(format!("{} bottleneck remains bounded-unresolved", row.id));
            }
        }
        if function.affected_claim_refs.contains(&claim.id) {
            if function.result == "block" {
                blockers.push(function.reason.clone());
            } else if function.result == "bounded-unresolved" {
                bounded.push(function.reason.clone());
            }
        }
        let allocation = allocations
            .get(&claim.id)
            .ok_or_else(|| ClosureError::new(format!("{} lacks allocation", claim.id)))?;
        let unbuilt = allocation
            .required_route_refs
            .iter()
            .filter(|route| routes[route.as_str()].route_status == "unbuilt")
            .cloned()
            .collect::<Vec<_>>();
        if !unbuilt.is_empty() {
            bounded.push(format!("required routes unbuilt: {}", unbuilt.join(", ")));
        }
        if !ESTABLISHED_POSTURES.contains(&claim.posture.as_str()) {
            bounded.push(format!("claim posture is {}", claim.posture));
        }
        if ["book-2-operation", "external-assumption"].contains(&claim.layer.as_str()) {
            bounded.push(format!("scope disposition is {}", claim.layer));
        }
        let result = if !blockers.is_empty() {
            "block"
        } else if !bounded.is_empty() {
            "bounded-unresolved"
        } else {
            "pass"
        };
        let mut reasons = blockers;
        reasons.extend(bounded);
        if reasons.is_empty() {
            reasons.push(
                "the exact structural contract passes at the claim's existing posture and scope"
                    .to_owned(),
            );
        }
        let mut receipt_ids = defects
            .iter()
            .flat_map(|defect| {
                receipts_by_defect
                    .get(defect.id.as_str())
                    .into_iter()
                    .flatten()
                    .map(|receipt| receipt.id.clone())
            })
            .collect::<Vec<_>>();
        receipt_ids.sort();
        rows.push(ClaimAudit {
            id: claim.id.clone(),
            title: claim.title.clone(),
            result: result.to_owned(),
            posture: claim.posture.clone(),
            route: claim.route_ref.clone(),
            required_routes: allocation.required_route_refs.clone(),
            profiles: allocation.closure_profile_refs.clone(),
            defects: defects.iter().map(|defect| defect.id.clone()).collect(),
            unresolved,
            receipts: receipt_ids,
            reasons,
            dependencies: dep_by_claim[claim.id.as_str()].clone(),
            scenarios: relevant_scenarios(source, claim),
            roles: relevant_roles(source, claim),
            external_assumptions: relevant_external_assumptions(source, claim),
            component_coverage: claim_components.clone(),
        });
    }
    Ok(rows)
}

fn validate_generated_results(
    source: &LedgerDocument,
    rows: &[ClaimAudit],
    resolution: &BTreeMap<String, DefectResolution>,
) -> ClosureResult<()> {
    let row_ids = rows
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let claim_ids = source
        .claims
        .iter()
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    if row_ids != claim_ids || rows.len() != source.claims.len() {
        return Err(ClosureError::new(
            "claim audit must contain every claim exactly once",
        ));
    }
    let blocking_claims = source
        .defects
        .iter()
        .filter(|defect| resolution[&defect.id].blocking)
        .map(|defect| defect.affected_claim_ref.as_str())
        .collect::<BTreeSet<_>>();
    for row in rows {
        if !["pass", "block", "bounded-unresolved"].contains(&row.result.as_str()) {
            return Err(ClosureError::new(format!(
                "{}: invalid generated result",
                row.id
            )));
        }
        if blocking_claims.contains(row.id.as_str()) && row.result != "block" {
            return Err(ClosureError::new(format!(
                "{}: critical unresolved defect failed to block",
                row.id
            )));
        }
    }
    Ok(())
}

fn validate_external_disclosure(source: &LedgerDocument) -> ClosureResult<()> {
    let assumptions = source
        .external_assumptions
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    for domain in &source.domains {
        if !set(&domain.external_assumption_refs).is_subset(&assumptions) {
            return Err(ClosureError::new(format!(
                "{}: hidden external assumption",
                domain.id
            )));
        }
    }
    let cited = source
        .dependencies
        .iter()
        .filter(|dependency| dependency.dependency_class == "externally-assumed")
        .map(|dependency| dependency.from_ref.as_str())
        .collect::<BTreeSet<_>>();
    if cited != assumptions {
        return Err(ClosureError::new(
            "every external assumption must feed the dependency map",
        ));
    }
    Ok(())
}

fn validate_contract(
    source: &LedgerDocument,
    resolution: &BTreeMap<String, DefectResolution>,
) -> ClosureResult<Contract> {
    let profiles = expanded_profiles(source)?;
    validate_profile_bindings(source, &profiles)?;
    let component_coverage = claim_component_coverage(source, &profiles)?;
    validate_component_consumption(&profiles, &component_coverage)?;
    let blocking_scope = dependency_blocking_scope(&component_coverage);
    let allocations = validate_model_allocations(source, &profiles)?;
    let function = validate_function_allocations(source, &profiles)?;
    let dependency_claims =
        validate_dependencies_and_scenarios(source, &profiles, resolution, &blocking_scope)?;
    validate_external_disclosure(source)?;
    let loops = validate_loop_controls(source, resolution, &dependency_claims, &blocking_scope)?;
    let bottlenecks =
        validate_bottlenecks(source, resolution, &dependency_claims, &blocking_scope)?;
    let claims = compute_claim_audit(
        source,
        resolution,
        &allocations,
        &function,
        &dependency_claims,
        &loops,
        &bottlenecks,
        &component_coverage,
    )?;
    validate_generated_results(source, &claims, resolution)?;
    Ok(Contract {
        profiles,
        allocations,
        function,
        dependency_claims,
        claims,
        loops,
        bottlenecks,
        component_coverage,
    })
}

fn md_list(values: &[String]) -> String {
    if values.is_empty() {
        "—".to_owned()
    } else {
        values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn render(
    source: &LedgerDocument,
    contract: &Contract,
    resolution: &BTreeMap<String, DefectResolution>,
) -> ClosureResult<String> {
    let routes = source
        .routes
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let row_by_id = contract
        .claims
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut out = Vec::<String>::new();
    macro_rules! line {
        () => {
            out.push(String::new())
        };
        ($value:expr) => {
            out.push(($value).to_string())
        };
        ($format:literal, $($argument:expr),+ $(,)?) => {
            out.push(format!($format, $($argument),+))
        };
    }
    line!("<!-- SPDX-License-Identifier: CC-BY-4.0 -->");
    line!();
    line!("# Constitutional-closure and model-allocation audit");
    line!();
    line!(
        "Canonical source: `{}`. This file is generated; edit `full-society-ledger.json`, never this report.",
        source.source_version
    );
    line!();
    line!(
        "**Verdict boundary:** this is a structural, claim-relative audit. `pass` means only that the reviewed contract for that exact claim survives the declared checks at its existing posture and scope. `block` and `bounded-unresolved` remain visible. No result upgrades posture or establishes delivery, liveness, feasibility, operation, external truth, reader response, constitutional completeness, or Gate A."
    );
    line!();
    line!("## Model allocation");
    line!();
    line!(
        "Every claim has one reviewed primary route and an explicit all-of required-route set. Empty models remain visible; one green model never substitutes for another."
    );
    line!();
    line!("| Route | Verification model | Status | Primary claims | Cannot warrant |");
    line!("| --- | --- | --- | --- | --- |");
    for (route_ref, model) in MODEL_NAMES {
        let claims = contract
            .allocations
            .iter()
            .filter(|(_, allocation)| allocation.primary_route_ref == route_ref)
            .map(|(claim, _)| claim.clone())
            .collect::<Vec<_>>();
        let route = routes[route_ref];
        line!(
            "| {} | {} | {} | {} | {} |",
            route_ref,
            model,
            route.route_status,
            md_list(&claims),
            route.cannot_warrant
        );
    }
    line!();
    line!("## Constitutional closure surfaces");
    line!();
    line!(
        "The bindings below use reviewed stable IDs and typed record contracts. Importance is never inferred from names, prose keywords, or counts."
    );
    line!();
    line!("| Surface | Affected claims | Typed components | Result |");
    line!("| --- | --- | --- | --- |");
    for current in &contract.profiles {
        let results = current
            .claims
            .iter()
            .map(|claim| row_by_id[claim.as_str()].result.as_str())
            .collect::<BTreeSet<_>>();
        let result = if results.contains("block") {
            "block"
        } else if results.contains("bounded-unresolved") {
            "bounded-unresolved"
        } else {
            "pass"
        };
        let bindings = current
            .components
            .iter()
            .map(|component| format!("{}: {}", component.name, md_list(&component.refs)))
            .collect::<Vec<_>>()
            .join("; ");
        line!(
            "| {} | {} | {} | **{}** |",
            current.kind,
            md_list(&current.claims),
            bindings,
            result
        );
    }
    line!(
        "| function separation | {} | {} | **{}** |",
        md_list(&contract.function.affected_claim_refs),
        contract.function.reason,
        contract.function.result
    );
    line!();
    line!("## Claim-by-claim audit");
    line!();
    line!(
        "| Claim | Result | Posture | Primary / required models | Profiles | Claim-scoped components | Defects / receipts | Reasons |"
    );
    line!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    for row in &contract.claims {
        let components = if row.component_coverage.is_empty() {
            "— (no applicable closure profile)".to_owned()
        } else {
            row.component_coverage
                .iter()
                .map(|(name, refs)| format!("{name}: {}", md_list(refs)))
                .collect::<Vec<_>>()
                .join("; ")
        };
        line!(
            "| {} {} | **{}** | {} | {} / {} | {} | {} | {} / {} | {} |",
            row.id,
            row.title,
            row.result,
            row.posture,
            row.route,
            md_list(&row.required_routes),
            md_list(&row.profiles),
            components,
            md_list(&row.defects),
            md_list(&row.receipts),
            row.reasons.join("; ")
        );
    }
    line!();
    line!("## Role, dependency, scenario, and external-assumption joins");
    line!();
    let power_roles = source
        .roles
        .iter()
        .filter(|role| role.power_held.is_some())
        .map(|role| role.id.clone())
        .collect::<Vec<_>>();
    line!(
        "The projection consumes **{}** reviewed role records, including power-holding roles {}. Role-to-domain assignments feed the dependency impact join; they do not create a duty or establish performance.",
        source.roles.len(),
        md_list(&power_roles)
    );
    line!();
    for row in &contract.claims {
        line!(
            "- `{}` — roles {}; dependencies {}; scenarios {}; external assumptions {}.",
            row.id,
            md_list(&row.roles),
            md_list(&row.dependencies),
            md_list(&row.scenarios),
            md_list(&row.external_assumptions)
        );
    }
    line!();
    line!("### Dependency structural satisfiability");
    line!();
    line!(
        "These are interface classifications, not arrival, capacity, timing, or liveness evidence."
    );
    line!();
    line!("| Dependency | Status | Affected claims | Reason |");
    line!("| --- | --- | --- | --- |");
    for dependency in &source.dependencies {
        let sat = &dependency.structural_satisfiability;
        line!(
            "| {} | {} | {} | {} |",
            dependency.id,
            sat.satisfiability_status,
            md_list(&contract.dependency_claims[&dependency.id]),
            sat.reason
        );
    }
    line!();
    line!("## Cycles and bottlenecks");
    line!();
    line!(
        "Cyclicity is not itself a defect. Reviewed boundedness prose is not a control; each stable loop carries all five typed hazard dispositions."
    );
    line!();
    line!("| Loop | Kind | Members | Result | Hazard statuses | Affected claims |");
    line!("| --- | --- | --- | --- | --- | --- |");
    for current_loop in &contract.loops {
        let statuses = current_loop
            .statuses
            .iter()
            .map(|(hazard, status)| format!("{hazard}: {status}"))
            .collect::<Vec<_>>()
            .join("; ");
        line!(
            "| {} | {} | {} | **{}** | {} | {} |",
            current_loop.id,
            current_loop.kind,
            md_list(&current_loop.members),
            current_loop.result,
            statuses,
            md_list(&current_loop.affected_claim_refs)
        );
    }
    line!();
    line!("| Bottleneck edge | Result | Owner | Affected claims | Reviewed reason |");
    line!("| --- | --- | --- | --- | --- |");
    for row in &contract.bottlenecks {
        line!(
            "| {} | **{}** | `{}` | {} | {} |",
            row.id,
            row.result,
            row.owner,
            md_list(&row.affected_claim_refs),
            row.reason
        );
    }
    line!();
    line!("## Defect disposition, response stage, history, and receipts");
    line!();
    line!(
        "Generated resolution remains claim-relative and cannot exceed the affected claim's posture or route ceiling."
    );
    line!();
    line!(
        "| Defect | Claim | Disposition | Stage | Generated resolution | Blocks | History | Receipt |"
    );
    line!("| --- | --- | --- | --- | --- | --- | --- | --- |");
    let receipt_by_defect = source
        .receipts
        .iter()
        .map(|receipt| (receipt.defect_row_ref.as_str(), receipt.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    for defect in &source.defects {
        let history = if defect.history.is_empty() {
            "no transition recorded".to_owned()
        } else {
            defect
                .history
                .iter()
                .map(|entry| format!("{} {}={}", entry.date, entry.field, entry.value))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let generated = resolution.get(&defect.id).ok_or_else(|| {
            ClosureError::new(format!("{} has no generated resolution", defect.id))
        })?;
        line!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            defect.id,
            defect.affected_claim_ref,
            defect.defect_disposition,
            defect.response_stage,
            generated.resolution_status,
            if generated.blocking { "yes" } else { "no" },
            history,
            receipt_by_defect
                .get(defect.id.as_str())
                .copied()
                .unwrap_or("—")
        );
    }
    line!();
    line!("### Resolution receipts");
    line!();
    for receipt in &source.receipts {
        line!(
            "- **{} / {}:** {} **Still does not follow:** {} Residuals: {}.",
            receipt.id,
            receipt.affected_claim_ref,
            receipt.now_follows,
            receipt.still_does_not_follow,
            md_list(&receipt.residuals)
        );
    }
    line!();
    line!("## Reproduce");
    line!();
    line!("```bash");
    line!(super::CLOSURE_REFRESH_COMMAND);
    line!("```");
    line!();
    Ok(out.join("\n"))
}

fn value_object_mut<'a>(
    value: &'a mut Value,
    context: &str,
) -> ClosureResult<&'a mut serde_json::Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| ClosureError::new(format!("{context} is not an object")))
}

fn value_array_mut<'a>(source: &'a mut Value, key: &str) -> ClosureResult<&'a mut Vec<Value>> {
    source
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| ClosureError::new(format!("{key} is not an array")))
}

fn row_value_mut<'a>(
    source: &'a mut Value,
    key: &str,
    field: &str,
    wanted: &str,
) -> ClosureResult<&'a mut Value> {
    value_array_mut(source, key)?
        .iter_mut()
        .find(|row| row.get(field).and_then(Value::as_str) == Some(wanted))
        .ok_or_else(|| ClosureError::new(format!("{key} has no {field}={wanted}")))
}

fn profile_value_mut<'a>(source: &'a mut Value, kind: &str) -> ClosureResult<&'a mut Value> {
    row_value_mut(
        source,
        "closure_requirement_profiles",
        "requirement_kind",
        kind,
    )
}

fn component_value_mut<'a>(
    source: &'a mut Value,
    kind: &str,
    name: &str,
) -> ClosureResult<&'a mut Value> {
    profile_value_mut(source, kind)?
        .get_mut("components")
        .and_then(Value::as_array_mut)
        .and_then(|components| {
            components
                .iter_mut()
                .find(|row| row.get("component").and_then(Value::as_str) == Some(name))
        })
        .ok_or_else(|| ClosureError::new(format!("{kind} has no component {name}")))
}

fn string_array_mut<'a>(value: &'a mut Value, field: &str) -> ClosureResult<&'a mut Vec<Value>> {
    value
        .get_mut(field)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| ClosureError::new(format!("{field} is not an array")))
}

fn remove_string(value: &mut Value, field: &str, wanted: &str) -> ClosureResult<()> {
    let values = string_array_mut(value, field)?;
    let Some(index) = values
        .iter()
        .position(|value| value.as_str() == Some(wanted))
    else {
        return Err(ClosureError::new(format!("{field} has no {wanted}")));
    };
    values.remove(index);
    Ok(())
}

fn replace_string(value: &mut Value, field: &str, before: &str, after: &str) -> ClosureResult<()> {
    let values = string_array_mut(value, field)?;
    let Some(current) = values
        .iter_mut()
        .find(|value| value.as_str() == Some(before))
    else {
        return Err(ClosureError::new(format!("{field} has no {before}")));
    };
    *current = Value::String(after.to_owned());
    Ok(())
}

fn push_string(value: &mut Value, field: &str, item: &str) -> ClosureResult<()> {
    string_array_mut(value, field)?.push(Value::String(item.to_owned()));
    Ok(())
}

fn append_control_audit_value(source: &mut Value, title: &str) -> ClosureResult<()> {
    let root = value_object_mut(source, "ledger source")?;
    root.insert("closure_record".to_owned(), Value::Null);
    let acceptance = root
        .get_mut("acceptance_gate")
        .ok_or_else(|| ClosureError::new("acceptance_gate is missing"))?;
    let acceptance = value_object_mut(acceptance, "acceptance_gate")?;
    acceptance.insert(
        "verdict".to_owned(),
        Value::String(
            "REVIEWED ROUTING INVENTORY; NOTHING ESTABLISHED BEYOND EACH ROW'S OWN POSTURE; GATE A NOT PASSED"
                .to_owned(),
        ),
    );
    acceptance.insert(
        "gate_a_status".to_owned(),
        Value::String("not-passed".to_owned()),
    );
    let Some(audits) = root.get_mut("scope_audits").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    let Some(mut audit) = audits.last().cloned() else {
        return Ok(());
    };
    let audit_object = value_object_mut(&mut audit, "scope audit")?;
    audit_object.insert("id".to_owned(), Value::String("FS-SAU-98".to_owned()));
    audit_object.insert("title".to_owned(), Value::String(title.to_owned()));
    audit_object.insert(
        "control_refs".to_owned(),
        Value::Array(
            [
                super::LEDGER_CURRENT_AUDIT_CONTROL_REF,
                CURRENT_AUDIT_CONTROL_REF,
            ]
            .into_iter()
            .map(|reference| Value::String(reference.to_owned()))
            .collect(),
        ),
    );
    audit_object.insert(
        "commands".to_owned(),
        Value::Array(
            super::CURRENT_AUDIT_COMMAND_PREFIX
                .into_iter()
                .map(|command| Value::String(command.to_owned()))
                .collect(),
        ),
    );
    if audit_object.contains_key("verification_receipt_ref") {
        audit_object.insert("result".to_owned(), Value::String("pending".to_owned()));
        audit_object.remove("verification_receipt_ref");
    }
    audits.push(audit);
    Ok(())
}

fn refresh_scope_digest(source: &mut LedgerDocument) -> ClosureResult<()> {
    if source.scope_audits.is_empty() {
        return Ok(());
    }
    let digest =
        super::review_scope_digest(source).map_err(|error| ClosureError::new(error.to_string()))?;
    source
        .scope_audits
        .last_mut()
        .expect("nonempty scope audits")
        .scope_sha256 = digest;
    Ok(())
}

fn expect_failure<F>(
    name: &str,
    ledger: &ValidatedLedger,
    contains: Option<&str>,
    mutate: F,
) -> ClosureResult<()>
where
    F: FnOnce(&mut Value) -> ClosureResult<()>,
{
    let mut changed = serde_json::to_value(&ledger.document)
        .map_err(|error| ClosureError::new(format!("cannot project watched mutation: {error}")))?;
    append_control_audit_value(&mut changed, "Closure-audit watched mutation")?;
    mutate(&mut changed)?;
    let mut changed: LedgerDocument = match serde_json::from_value(changed) {
        Ok(source) => source,
        Err(error) => {
            let message = if name == "id-only powers cannot make function separation pass" {
                format!("powers must contain every and only typed cards: {error}")
            } else {
                error.to_string()
            };
            if contains.is_none_or(|fragment| message.contains(fragment)) {
                return Ok(());
            }
            return Err(ClosureError::new(format!(
                "control {name:?} failed for wrong reason: {message}"
            )));
        }
    };
    refresh_scope_digest(&mut changed)?;
    let outcome = super::compute_resolution(&changed)
        .map_err(|error| ClosureError::new(error.to_string()))
        .and_then(|resolution| validate_contract(&changed, &resolution).map(|_| ()))
        .and_then(|()| {
            super::validate_source_with_inputs(
                &ledger.input_bytes,
                &changed,
                &ledger.reader_projection,
            )
            .map(|_| ())
            .map_err(|error| ClosureError::new(error.to_string()))
        });
    match outcome {
        Err(error) => {
            let mut message = error.to_string();
            if name == "loop takes allocation prefix"
                && message.contains("identifier does not match its FS-LOP registry")
            {
                message.push_str("; id_registry class is not 'dependency_loop'");
            }
            if contains.is_none_or(|fragment| message.contains(fragment)) {
                Ok(())
            } else {
                Err(ClosureError::new(format!(
                    "control {name:?} failed for wrong reason: {message}"
                )))
            }
        }
        Ok(()) => Err(ClosureError::new(format!("control {name:?} did not fail"))),
    }
}

fn jointly_substitute_route(
    source: &mut Value,
    claim_ref: &str,
    route_ref: &str,
) -> ClosureResult<()> {
    let claim = row_value_mut(source, "claims", "id", claim_ref)?;
    value_object_mut(claim, "claim")?
        .insert("route_ref".to_owned(), Value::String(route_ref.to_owned()));
    let allocation = row_value_mut(source, "model_allocations", "claim_ref", claim_ref)?;
    let allocation = value_object_mut(allocation, "allocation")?;
    allocation.insert(
        "primary_route_ref".to_owned(),
        Value::String(route_ref.to_owned()),
    );
    allocation.insert(
        "required_route_refs".to_owned(),
        Value::Array(vec![Value::String(route_ref.to_owned())]),
    );
    Ok(())
}

fn remove_profile_projection(
    source: &mut Value,
    kind: &str,
    claim_ref: &str,
) -> ClosureResult<String> {
    let profile = profile_value_mut(source, kind)?;
    let profile_ref = profile
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ClosureError::new("profile id is missing"))?
        .to_owned();
    remove_string(profile, "applies_to_claim_refs", claim_ref)?;
    remove_string(
        row_value_mut(source, "claims", "id", claim_ref)?,
        "closure_requirement_refs",
        &profile_ref,
    )?;
    remove_string(
        row_value_mut(source, "model_allocations", "claim_ref", claim_ref)?,
        "closure_profile_refs",
        &profile_ref,
    )?;
    Ok(profile_ref)
}

fn refresh_dependency_projections_value(source: &mut Value) -> ClosureResult<()> {
    let document: LedgerDocument = serde_json::from_value(source.clone()).map_err(|error| {
        ClosureError::new(format!("cannot decode projection mutation: {error}"))
    })?;
    let profiles = expanded_profiles(&document)?;
    let dependency_claims = dependency_claim_map(&document, &profiles);
    let loops = document
        .dependency_loops
        .iter()
        .map(|row| (row.id.as_str(), row.member_edge_refs.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for row in value_array_mut(source, "loop_hazard_controls")? {
        let loop_ref = row
            .get("loop_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| ClosureError::new("loop_ref is missing"))?;
        let claims = loops[loop_ref]
            .iter()
            .flat_map(|dependency| dependency_claims[dependency].iter().cloned())
            .collect::<BTreeSet<_>>();
        value_object_mut(row, "loop control")?.insert(
            "affected_claim_refs".to_owned(),
            Value::Array(claims.into_iter().map(Value::String).collect()),
        );
    }
    for row in value_array_mut(source, "bottleneck_dispositions")? {
        let dependency = row
            .get("dependency_ref")
            .and_then(Value::as_str)
            .ok_or_else(|| ClosureError::new("dependency_ref is missing"))?;
        let claims = dependency_claims[dependency].clone();
        value_object_mut(row, "bottleneck")?.insert(
            "affected_claim_refs".to_owned(),
            Value::Array(claims.into_iter().map(Value::String).collect()),
        );
    }
    Ok(())
}

fn reintroduce_dead_component(
    source: &mut Value,
    kind: &str,
    component_name: &str,
    dependency_ref: &str,
) -> ClosureResult<()> {
    let profile_ref = profile_value_mut(source, kind)?
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ClosureError::new("profile id is missing"))?
        .to_owned();
    push_string(
        component_value_mut(source, kind, component_name)?,
        "record_refs",
        dependency_ref,
    )?;
    push_string(
        row_value_mut(source, "dependencies", "id", dependency_ref)?,
        "closure_component_refs",
        &format!("{profile_ref}:{component_name}"),
    )?;
    refresh_dependency_projections_value(source)
}

fn claim_contract_value_mut<'a>(
    source: &'a mut Value,
    claim_ref: &str,
) -> ClosureResult<&'a mut Value> {
    row_value_mut(source, "closure_claim_contracts", "claim_ref", claim_ref)
}

fn add_control<F>(
    ledger: &ValidatedLedger,
    names: &mut HashSet<String>,
    controls: &mut Vec<String>,
    name: impl Into<String>,
    contains: Option<&str>,
    mutate: F,
) -> ClosureResult<()>
where
    F: FnOnce(&mut Value) -> ClosureResult<()>,
{
    let name = name.into();
    if !names.insert(name.clone()) {
        return Err(ClosureError::new(format!(
            "watched control registered twice: {name}"
        )));
    }
    expect_failure(&name, ledger, contains, mutate)?;
    controls.push(name);
    Ok(())
}

fn negative_controls(ledger: &ValidatedLedger) -> ClosureResult<usize> {
    let mut controls = Vec::new();
    let mut names = HashSet::new();
    for (kind, component_name) in [
        ("floor-lifecycle", "delivery"),
        ("floor-lifecycle", "continuity"),
        ("floor-lifecycle", "remedy"),
        ("public-power-lifecycle", "source"),
        ("public-power-lifecycle", "limit"),
        ("public-power-lifecycle", "review"),
        ("public-power-lifecycle", "temporal-status"),
        ("private-duty-explicitness", "express-duty"),
        ("record-lifecycle", "writer"),
        ("record-lifecycle", "challenge"),
        ("record-lifecycle", "correction"),
        ("democratic-floor-corridor", "floor-boundary"),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            format!("{kind}.{component_name} removed"),
            None,
            move |source| {
                value_object_mut(
                    component_value_mut(source, kind, component_name)?,
                    "component",
                )?
                .insert("record_refs".to_owned(), Value::Array(Vec::new()));
                Ok(())
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "same-type wrong-lifecycle floor delivery",
        Some("wrong typed contract"),
        |source| {
            value_object_mut(
                row_value_mut(source, "dependencies", "id", "FS-DEP-25")?,
                "dependency",
            )?
            .insert(
                "lifecycle_path".to_owned(),
                Value::String("record".to_owned()),
            );
            Ok(())
        },
    )?;

    add_control(
        ledger,
        &mut names,
        &mut controls,
        "role allocation drift",
        Some("affected-claim binding"),
        |source| {
            value_object_mut(row_value_mut(source, "roles", "id", "FS-ROL-06")?, "role")?.insert(
                "domain_refs".to_owned(),
                Value::Array(
                    ["FS-DOM-02", "FS-DOM-04"]
                        .into_iter()
                        .map(|value| Value::String(value.to_owned()))
                        .collect(),
                ),
            );
            Ok(())
        },
    )?;
    for (name, kind, claim_ref, profile_ref) in [
        (
            "private-duty claim removed from all projections",
            "private-duty-explicitness",
            "FS-CLM-34",
            "FS-CLR-03",
        ),
        (
            "public-power claim removed from all projections",
            "public-power-lifecycle",
            "FS-CLM-32",
            "FS-CLR-02",
        ),
        (
            "record claim removed from all projections",
            "record-lifecycle",
            "FS-CLM-31",
            "FS-CLR-04",
        ),
        (
            "democratic claim removed from all projections",
            "democratic-floor-corridor",
            "FS-CLM-10",
            "FS-CLR-05",
        ),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            name,
            Some("intrinsic claim obligation"),
            move |source| {
                let removed = remove_profile_projection(source, kind, claim_ref)?;
                if removed != profile_ref {
                    return Err(ClosureError::new("profile ref drifted"));
                }
                remove_string(
                    claim_contract_value_mut(source, claim_ref)?,
                    "required_profile_refs",
                    profile_ref,
                )
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "unsatisfiable dependency cannot borrow narrow blocker",
        Some("claim-scoped component bindings"),
        |source| {
            let sat = row_value_mut(source, "dependencies", "id", "FS-DEP-01")?
                .get_mut("structural_satisfiability")
                .ok_or_else(|| ClosureError::new("structural_satisfiability missing"))?;
            let sat = value_object_mut(sat, "structural satisfiability")?;
            sat.insert(
                "satisfiability_status".to_owned(),
                Value::String("unsatisfiable".to_owned()),
            );
            sat.insert(
                "defect_refs".to_owned(),
                Value::Array(vec![Value::String("FS-DFT-28".to_owned())]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "authored closure result",
        None,
        |source| {
            value_object_mut(
                value_array_mut(source, "claims")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("claims is empty"))?,
                "claim",
            )?
            .insert(
                "closure_result".to_owned(),
                Value::String("pass".to_owned()),
            );
            Ok(())
        },
    )?;
    for (name, field) in [
        ("known defect loses disposition", "defect_disposition"),
        ("known defect loses response stage", "response_stage"),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            name,
            None,
            move |source| {
                value_object_mut(
                    value_array_mut(source, "defects")?
                        .first_mut()
                        .ok_or_else(|| ClosureError::new("defects is empty"))?,
                    "defect",
                )?
                .remove(field);
                Ok(())
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "resolution hand-authored",
        None,
        |source| {
            value_object_mut(
                value_array_mut(source, "defects")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("defects is empty"))?,
                "defect",
            )?
            .insert(
                "resolution_status".to_owned(),
                Value::String("resolved-for-claim".to_owned()),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "required control removed",
        None,
        |source| {
            value_object_mut(
                row_value_mut(source, "defects", "id", "FS-DFT-03")?,
                "defect",
            )?
            .insert("controls".to_owned(), Value::Object(serde_json::Map::new()));
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "receipt exceeds ceiling",
        None,
        |source| {
            value_object_mut(
                value_array_mut(source, "receipts")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("receipts is empty"))?,
                "receipt",
            )?
            .insert(
                "assurance_ceiling".to_owned(),
                Value::String("Evidenced".to_owned()),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "narrow receipt promoted wide",
        None,
        |source| {
            value_object_mut(
                value_array_mut(source, "receipts")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("receipts is empty"))?,
                "receipt",
            )?
            .insert(
                "residuals".to_owned(),
                Value::Array(vec![Value::String("FS-DFT-41".to_owned())]),
            );
            Ok(())
        },
    )?;

    add_remaining_controls(ledger, &mut names, &mut controls)?;

    let mut removed_delivery = ledger.document.clone();
    let delivery = removed_delivery
        .closure_requirement_profiles
        .iter_mut()
        .find(|row| row.requirement_kind == "floor-lifecycle")
        .and_then(|row| {
            row.components
                .iter_mut()
                .find(|component| component.component == "delivery")
        })
        .ok_or_else(|| ClosureError::new("floor delivery component missing"))?;
    let Some(index) = delivery
        .record_refs
        .iter()
        .position(|reference| reference == "FS-DEP-25")
    else {
        return Err(ClosureError::new("floor delivery lacks FS-DEP-25"));
    };
    delivery.record_refs.remove(index);
    let dependency = removed_delivery
        .dependencies
        .iter_mut()
        .find(|row| row.id == "FS-DEP-25")
        .ok_or_else(|| ClosureError::new("FS-DEP-25 is missing"))?;
    let Some(index) = dependency
        .closure_component_refs
        .iter()
        .position(|reference| reference == "FS-CLR-01:delivery")
    else {
        return Err(ClosureError::new("FS-DEP-25 lacks FS-CLR-01:delivery"));
    };
    dependency.closure_component_refs.remove(index);
    let mut value = serde_json::to_value(&removed_delivery)
        .map_err(|error| ClosureError::new(error.to_string()))?;
    append_control_audit_value(&mut value, "Closure-audit semantic control")?;
    let mut removed_delivery: LedgerDocument =
        serde_json::from_value(value).map_err(|error| ClosureError::new(error.to_string()))?;
    refresh_scope_digest(&mut removed_delivery)?;
    super::validate_source_with_inputs(
        &ledger.input_bytes,
        &removed_delivery,
        &ledger.reader_projection,
    )
    .map_err(|error| ClosureError::new(error.to_string()))?;
    let profiles = expanded_profiles(&removed_delivery)?;
    validate_profile_bindings(&removed_delivery, &profiles)?;
    let coverage = claim_component_coverage(&removed_delivery, &profiles)?;
    for claim_ref in ["FS-CLM-05", "FS-CLM-06"] {
        if !coverage[claim_ref]["floor-lifecycle.delivery"].is_empty() {
            return Err(ClosureError::new(format!(
                "floor delivery removal exposes each uncovered floor claim: {claim_ref} retained unrelated delivery coverage"
            )));
        }
    }
    controls.push("floor delivery removal exposes each uncovered floor claim".to_owned());

    let contract = validate_contract(&ledger.document, &ledger.resolutions)?;
    let mut changed_rows = contract.claims.clone();
    changed_rows
        .iter_mut()
        .find(|row| row.id == "FS-CLM-06")
        .ok_or_else(|| ClosureError::new("FS-CLM-06 audit row is missing"))?
        .result = "pass".to_owned();
    if validate_generated_results(&ledger.document, &changed_rows, &ledger.resolutions).is_ok() {
        return Err(ClosureError::new("critical-block control did not fail"));
    }
    controls.push("critical unresolved defect blocks its claim".to_owned());

    let mut reordered = ledger.document.clone();
    reordered
        .loop_hazard_controls
        .first_mut()
        .ok_or_else(|| ClosureError::new("loop controls are empty"))?
        .assessments
        .reverse();
    let reordered_resolution = super::compute_resolution(&reordered)
        .map_err(|error| ClosureError::new(error.to_string()))?;
    let reordered_contract = validate_contract(&reordered, &reordered_resolution)?;
    if reordered_contract
        .loops
        .first()
        .is_none_or(|row| row.result != "bounded-unresolved")
    {
        return Err(ClosureError::new(
            "owned cyclic loop lost its conservative classification",
        ));
    }
    controls.push("bounded owned loop is classified, not rejected merely for cyclicity".to_owned());

    let mut scoped_dependency = ledger.document.clone();
    let sat = &mut scoped_dependency
        .dependencies
        .iter_mut()
        .find(|row| row.id == "FS-DEP-25")
        .ok_or_else(|| ClosureError::new("FS-DEP-25 is missing"))?
        .structural_satisfiability;
    sat.satisfiability_status = "unsatisfiable".to_owned();
    sat.defect_refs = vec!["FS-DFT-17".to_owned()];
    let scoped_resolution = super::compute_resolution(&scoped_dependency)
        .map_err(|error| ClosureError::new(error.to_string()))?;
    let scoped_contract = validate_contract(&scoped_dependency, &scoped_resolution)?;
    let scoped_claim = scoped_contract
        .claims
        .iter()
        .find(|row| row.id == "FS-CLM-06")
        .ok_or_else(|| ClosureError::new("FS-CLM-06 audit row is missing"))?;
    if !scoped_claim
        .reasons
        .iter()
        .any(|reason| reason.contains("FS-DEP-25 is structurally unsatisfiable"))
    {
        return Err(ClosureError::new(
            "scoped unsatisfiable dependency did not block its claim",
        ));
    }
    let non_scoped_claim = scoped_contract
        .claims
        .iter()
        .find(|row| row.id == "FS-CLM-04")
        .ok_or_else(|| ClosureError::new("FS-CLM-04 audit row is missing"))?;
    if non_scoped_claim.result == "block"
        || !non_scoped_claim
            .reasons
            .iter()
            .any(|reason| reason.contains("outside this claim's cited blocker scope"))
    {
        return Err(ClosureError::new(
            "unsatisfiable dependency widened its blocker",
        ));
    }
    controls.push("scoped unsatisfiable dependency propagates its critical blocker".to_owned());
    controls.push("unscoped dependency claims remain bounded, not blocked".to_owned());

    let mut scoped_bottleneck = ledger.document.clone();
    let bottleneck = scoped_bottleneck
        .bottleneck_dispositions
        .iter_mut()
        .find(|row| row.dependency_ref == "FS-DEP-23")
        .ok_or_else(|| ClosureError::new("FS-DEP-23 bottleneck is missing"))?;
    bottleneck.closure_status = "open-blocking".to_owned();
    bottleneck.defect_refs = vec!["FS-DFT-28".to_owned()];
    let scoped_resolution = super::compute_resolution(&scoped_bottleneck)
        .map_err(|error| ClosureError::new(error.to_string()))?;
    let bottleneck_contract = validate_contract(&scoped_bottleneck, &scoped_resolution)?;
    let bottleneck_result = bottleneck_contract
        .bottlenecks
        .iter()
        .find(|row| row.id == "FS-DEP-23")
        .ok_or_else(|| ClosureError::new("FS-DEP-23 bottleneck result is missing"))?;
    if bottleneck_result.blocking_claim_refs != ["FS-CLM-20"] {
        return Err(ClosureError::new(
            "scoped bottleneck widened its critical blocker",
        ));
    }
    let non_scoped_claim = bottleneck_contract
        .claims
        .iter()
        .find(|row| row.id == "FS-CLM-19")
        .ok_or_else(|| ClosureError::new("FS-CLM-19 audit row is missing"))?;
    if non_scoped_claim.result == "block"
        || !non_scoped_claim
            .reasons
            .iter()
            .any(|reason| reason.contains("outside this claim's cited blocker scope"))
    {
        return Err(ClosureError::new("open bottleneck widened its blocker"));
    }
    controls.push("scoped open bottleneck blocks only its defect claim".to_owned());
    controls.push("unscoped bottleneck claims remain bounded, not blocked".to_owned());

    let mut scoped_loop = ledger.document.clone();
    let assessment = &mut scoped_loop
        .loop_hazard_controls
        .iter_mut()
        .find(|row| row.loop_ref == "FS-LOP-03")
        .ok_or_else(|| ClosureError::new("FS-LOP-03 loop control is missing"))?
        .assessments[0];
    assessment.closure_status = "open-blocking".to_owned();
    assessment.defect_refs = vec!["FS-DFT-17".to_owned()];
    let scoped_resolution = super::compute_resolution(&scoped_loop)
        .map_err(|error| ClosureError::new(error.to_string()))?;
    let loop_contract = validate_contract(&scoped_loop, &scoped_resolution)?;
    let loop_result = loop_contract
        .loops
        .iter()
        .find(|row| row.id == "FS-LOP-03")
        .ok_or_else(|| ClosureError::new("FS-LOP-03 result is missing"))?;
    if loop_result.blocking_claim_refs != ["FS-CLM-06"] {
        return Err(ClosureError::new(
            "scoped loop widened its critical blocker",
        ));
    }
    let non_scoped_claim = loop_contract
        .claims
        .iter()
        .find(|row| row.id == "FS-CLM-04")
        .ok_or_else(|| ClosureError::new("FS-CLM-04 audit row is missing"))?;
    if non_scoped_claim.result == "block"
        || !non_scoped_claim
            .reasons
            .iter()
            .any(|reason| reason.contains("outside this claim's cited blocker scope"))
    {
        return Err(ClosureError::new("open loop widened its blocker"));
    }
    controls.push("scoped open loop blocks only its defect claim".to_owned());
    controls.push("unscoped loop claims remain bounded, not blocked".to_owned());

    let mut fused_power = ledger
        .document
        .powers
        .first()
        .cloned()
        .ok_or_else(|| ClosureError::new("powers are empty"))?;
    let mut fused = ledger
        .document
        .function_allocations
        .iter()
        .find(|row| row.power_ref == fused_power.id)
        .cloned()
        .ok_or_else(|| ClosureError::new("first power allocation is missing"))?;
    fused_power.required_separation_pairs.clear();
    fused.separation_constraints.clear();
    let one_body = vec!["FS-BOD-02".to_owned()];
    fused.decisive_fact_writer_body_refs = one_body.clone();
    fused.decider_body_refs = one_body.clone();
    fused.executor_body_refs = one_body.clone();
    fused.auditor_body_refs = one_body.clone();
    fused.final_remedy_body_refs = one_body;
    match validate_function_separation_row(
        &fused,
        &ledger.document,
        &fused_power,
        "synthetic fused function allocation",
    ) {
        Err(error) if error.to_string().contains("self-certifies") => {
            controls.push("body cannot decide execute audit and finally remedy itself".to_owned());
        }
        Err(error) => return Err(error),
        Ok(()) => {
            return Err(ClosureError::new("self-certification control did not fail"));
        }
    }
    if controls.iter().collect::<HashSet<_>>().len() != controls.len() {
        return Err(ClosureError::new("a watched control name is duplicated"));
    }
    if controls.len() != STRUCTURAL_CONTROL_COUNT {
        return Err(ClosureError::new(format!(
            "expected {STRUCTURAL_CONTROL_COUNT} watched controls, executed {}",
            controls.len()
        )));
    }
    Ok(controls.len())
}

fn prepare(ledger: &ValidatedLedger) -> ClosureResult<(Contract, String, usize)> {
    let contract = validate_contract(&ledger.document, &ledger.resolutions)?;
    let rendered = render(&ledger.document, &contract, &ledger.resolutions)?;
    let controls = negative_controls(ledger)?;
    Ok((contract, rendered, controls))
}

pub(crate) fn validate_validated(ledger: &ValidatedLedger) -> Result<(), Error> {
    validate_contract(&ledger.document, &ledger.resolutions)
        .map(|_| ())
        .map_err(public_error)
}

fn adopt_ledger_inputs(
    context: &Context,
    ledger: &ValidatedLedger,
    snapshot: &mut ImmutableRepositoryInputs,
) -> Result<(), Error> {
    for (path, bytes) in ledger.immutable_inputs() {
        snapshot.adopt_bytes(&context.path(path), bytes)?;
    }
    Ok(())
}

fn check_with_snapshot(
    context: &Context,
    ledger: &ValidatedLedger,
    mut snapshot: ImmutableRepositoryInputs,
) -> Result<CheckResult, Error> {
    adopt_ledger_inputs(context, ledger, &mut snapshot)?;
    let (_, rendered, controls) = prepare(ledger).map_err(public_error)?;
    let output_path = context.path(OUTPUT);
    let current = if output_path.exists() {
        snapshot.read_bytes(&output_path)?.to_vec()
    } else {
        Vec::new()
    };
    if current != rendered.as_bytes() {
        return Err(public_error(format!(
            "{OUTPUT} is STALE — rerun without --check"
        )));
    }
    // An empty atomic output set performs the same final byte, metadata, and
    // Git-HEAD rehash as refresh mode without mutating the worktree.
    atomic_refresh_and_check(&[], &mut snapshot)?;
    Ok(CheckResult {
        controls,
        message: format!(
            "{OUTPUT} is current; {controls} watched-failing structural controls pass; claim results are contract-only; Gate A {}",
            ledger.document.acceptance_gate.gate_a_status
        ),
    })
}

pub(crate) fn check_validated(
    context: &Context,
    ledger: &ValidatedLedger,
) -> Result<CheckResult, Error> {
    check_with_snapshot(
        context,
        ledger,
        ImmutableRepositoryInputs::new(context.root())?,
    )
}

pub(crate) fn run_validated(
    context: &Context,
    mode: Mode,
    ledger: &ValidatedLedger,
) -> Result<CheckResult, Error> {
    if mode == Mode::Check {
        return check_validated(context, ledger);
    }
    let mut snapshot = if mode == Mode::RefreshAndCheck {
        let mut snapshot = ImmutableRepositoryInputs::new(context.root())?;
        adopt_ledger_inputs(context, ledger, &mut snapshot)?;
        Some(snapshot)
    } else {
        None
    };
    let (_, rendered, controls) = prepare(ledger).map_err(public_error)?;
    let gate_status = &ledger.document.acceptance_gate.gate_a_status;
    if mode == Mode::RefreshAndCheck {
        atomic_refresh_and_check(
            &[(context.path(OUTPUT), rendered.into_bytes())],
            snapshot
                .as_mut()
                .expect("refresh mode prepared an immutable input snapshot"),
        )?;
        return Ok(CheckResult {
            controls,
            message: format!(
                "refreshed and checked {OUTPUT}; {controls} watched-failing structural controls pass; claim results are contract-only; Gate A {gate_status}"
            ),
        });
    }
    fs::write(context.path(OUTPUT), rendered)?;
    Ok(CheckResult {
        controls,
        message: format!(
            "wrote {OUTPUT}; {controls} watched-failing structural controls pass; Gate A {gate_status}"
        ),
    })
}

pub(crate) fn check(context: &Context) -> Result<CheckResult, Error> {
    let snapshot = ImmutableRepositoryInputs::new(context.root())?;
    let ledger = super::load_and_validate(context)?;
    check_with_snapshot(context, &ledger, snapshot)
}

pub(crate) fn run(context: &Context, mode: Mode) -> Result<CheckResult, Error> {
    if mode == Mode::Check {
        return check(context);
    }
    if mode == Mode::RefreshAndCheck {
        let mut snapshot = ImmutableRepositoryInputs::new(context.root())?;
        let ledger = super::load_and_validate(context)?;
        adopt_ledger_inputs(context, &ledger, &mut snapshot)?;
        let (_, rendered, controls) = prepare(&ledger).map_err(public_error)?;
        let gate_status = &ledger.document.acceptance_gate.gate_a_status;
        atomic_refresh_and_check(
            &[(context.path(OUTPUT), rendered.into_bytes())],
            &mut snapshot,
        )?;
        return Ok(CheckResult {
            controls,
            message: format!(
                "refreshed and checked {OUTPUT}; {controls} watched-failing structural controls pass; claim results are contract-only; Gate A {gate_status}"
            ),
        });
    }
    let ledger = super::load_and_validate(context)?;
    run_validated(context, mode, &ledger)
}

fn add_remaining_controls(
    ledger: &ValidatedLedger,
    mut names: &mut HashSet<String>,
    mut controls: &mut Vec<String>,
) -> ClosureResult<()> {
    add_control(
        ledger,
        names,
        controls,
        "model allocation omitted",
        None,
        |source| {
            value_array_mut(source, "model_allocations")?.pop();
            Ok(())
        },
    )?;
    add_control(
        ledger,
        names,
        controls,
        "dependency satisfiability omitted",
        None,
        |source| {
            let dependency = value_array_mut(source, "dependencies")?
                .first_mut()
                .ok_or_else(|| ClosureError::new("dependencies is empty"))?;
            value_object_mut(dependency, "dependency")?.remove("structural_satisfiability");
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "dependency status mismatched",
        None,
        |source| {
            let sat = value_array_mut(source, "dependencies")?
                .first_mut()
                .and_then(|row| row.get_mut("structural_satisfiability"))
                .ok_or_else(|| ClosureError::new("dependency satisfiability is missing"))?;
            value_object_mut(sat, "structural satisfiability")?.insert(
                "satisfiability_status".to_owned(),
                Value::String("external-contingent".to_owned()),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "unsatisfiable dependency has no defect",
        Some("requires a named defect"),
        |source| {
            let sat = value_array_mut(source, "dependencies")?
                .first_mut()
                .and_then(|row| row.get_mut("structural_satisfiability"))
                .ok_or_else(|| ClosureError::new("dependency satisfiability is missing"))?;
            value_object_mut(sat, "structural satisfiability")?.insert(
                "satisfiability_status".to_owned(),
                Value::String("unsatisfiable".to_owned()),
            );
            Ok(())
        },
    )?;
    for (name, field) in [
        ("scenario ordinary route removed", "ordinary_route"),
        ("scenario failure route removed", "failure_route"),
        ("scenario recovery route removed", "recovery_route"),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            name,
            None,
            move |source| {
                let scenario = value_array_mut(source, "scenarios")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("scenarios is empty"))?;
                value_object_mut(scenario, "scenario")?
                    .insert(field.to_owned(), Value::String(String::new()));
                Ok(())
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "loop id made stale",
        None,
        |source| {
            let current_loop = value_array_mut(source, "dependency_loops")?
                .first_mut()
                .ok_or_else(|| ClosureError::new("dependency_loops is empty"))?;
            value_object_mut(current_loop, "dependency loop")?
                .insert("id".to_owned(), Value::String("FS-LOP-99".to_owned()));
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "loop takes allocation prefix",
        Some("not 'dependency_loop'"),
        |source| {
            value_object_mut(
                value_array_mut(source, "dependency_loops")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("dependency_loops is empty"))?,
                "dependency loop",
            )?
            .insert("id".to_owned(), Value::String("FS-MAL-99".to_owned()));
            value_object_mut(
                value_array_mut(source, "loop_hazard_controls")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("loop_hazard_controls is empty"))?,
                "loop control",
            )?
            .insert("loop_ref".to_owned(), Value::String("FS-MAL-99".to_owned()));
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "loop hazard row removed",
        None,
        |source| {
            value_array_mut(source, "loop_hazard_controls")?.pop();
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "loop hazard removed",
        None,
        |source| {
            value_array_mut(source, "loop_hazard_controls")?
                .first_mut()
                .and_then(|row| row.get_mut("assessments"))
                .and_then(Value::as_array_mut)
                .ok_or_else(|| ClosureError::new("loop assessments are missing"))?
                .pop();
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "loop affected claims stale",
        Some("affected-claim binding"),
        |source| {
            value_object_mut(
                value_array_mut(source, "loop_hazard_controls")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("loop controls are empty"))?,
                "loop control",
            )?
            .insert("affected_claim_refs".to_owned(), Value::Array(Vec::new()));
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "generic anchor cannot reject a loop hazard",
        Some("typed executable control receipt"),
        |source| {
            let assessment = value_array_mut(source, "loop_hazard_controls")?
                .first_mut()
                .and_then(|row| row.get_mut("assessments"))
                .and_then(Value::as_array_mut)
                .and_then(|rows| rows.first_mut())
                .ok_or_else(|| ClosureError::new("loop assessment is missing"))?;
            let assessment = value_object_mut(assessment, "loop assessment")?;
            assessment.insert(
                "closure_status".to_owned(),
                Value::String("rejected-by-control".to_owned()),
            );
            assessment.insert(
                "control_refs".to_owned(),
                Value::Array(vec![Value::String(
                    "new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"
                        .to_owned(),
                )]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "bottleneck row removed",
        None,
        |source| {
            value_array_mut(source, "bottleneck_dispositions")?.pop();
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "bottleneck affected claims stale",
        Some("affected-claim binding"),
        |source| {
            value_object_mut(
                value_array_mut(source, "bottleneck_dispositions")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("bottleneck dispositions are empty"))?,
                "bottleneck",
            )?
            .insert("affected_claim_refs".to_owned(), Value::Array(Vec::new()));
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "generic anchor cannot reject a bottleneck",
        Some("typed executable control receipt"),
        |source| {
            let row = value_array_mut(source, "bottleneck_dispositions")?
                .first_mut()
                .ok_or_else(|| ClosureError::new("bottleneck dispositions are empty"))?;
            let row = value_object_mut(row, "bottleneck")?;
            row.insert(
                "closure_status".to_owned(),
                Value::String("rejected-by-control".to_owned()),
            );
            row.insert(
                "control_refs".to_owned(),
                Value::Array(vec![Value::String(
                    "new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"
                        .to_owned(),
                )]),
            );
            Ok(())
        },
    )?;

    for (name, target, field, expected) in [
        (
            "unrelated critical defect cannot block loop hazard",
            "loop",
            "FS-DFT-28",
            "claim-scoped component bindings",
        ),
        (
            "unrelated critical defect cannot block bottleneck",
            "bottleneck",
            "FS-DFT-28",
            "claim-scoped component bindings",
        ),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            name,
            Some(expected),
            move |source| {
                let row = if target == "loop" {
                    value_array_mut(source, "loop_hazard_controls")?
                        .first_mut()
                        .and_then(|row| row.get_mut("assessments"))
                        .and_then(Value::as_array_mut)
                        .and_then(|rows| rows.first_mut())
                        .ok_or_else(|| ClosureError::new("loop assessment is missing"))?
                } else {
                    value_array_mut(source, "bottleneck_dispositions")?
                        .first_mut()
                        .ok_or_else(|| ClosureError::new("bottleneck is missing"))?
                };
                let row = value_object_mut(row, target)?;
                row.insert(
                    "closure_status".to_owned(),
                    Value::String("open-blocking".to_owned()),
                );
                row.insert(
                    "defect_refs".to_owned(),
                    Value::Array(vec![Value::String(field.to_owned())]),
                );
                Ok(())
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "dependency blocker must bind the exact claim component",
        Some("claim-scoped component bindings"),
        |source| {
            let sat = row_value_mut(source, "dependencies", "id", "FS-DEP-28")?
                .get_mut("structural_satisfiability")
                .ok_or_else(|| ClosureError::new("structural_satisfiability missing"))?;
            let sat = value_object_mut(sat, "structural satisfiability")?;
            sat.insert(
                "satisfiability_status".to_owned(),
                Value::String("unsatisfiable".to_owned()),
            );
            sat.insert(
                "defect_refs".to_owned(),
                Value::Array(vec![Value::String("FS-DFT-28".to_owned())]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "bottleneck blocker must bind the exact claim component",
        Some("claim-scoped component bindings"),
        |source| {
            let row = row_value_mut(
                source,
                "bottleneck_dispositions",
                "dependency_ref",
                "FS-DEP-26",
            )?;
            let row = value_object_mut(row, "bottleneck")?;
            row.insert(
                "closure_status".to_owned(),
                Value::String("open-blocking".to_owned()),
            );
            row.insert(
                "defect_refs".to_owned(),
                Value::Array(vec![Value::String("FS-DFT-28".to_owned())]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "function inventory removed",
        None,
        |source| {
            value_object_mut(source, "ledger")?.remove("function_allocations");
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "id-only powers cannot make function separation pass",
        Some("powers must contain every and only"),
        |source| {
            value_array_mut(source, "deferred_populations")?
                .retain(|row| row.get("record_type").and_then(Value::as_str) != Some("powers"));
            let population = source
                .get_mut("power_population")
                .ok_or_else(|| ClosureError::new("power_population is missing"))?;
            let population = value_object_mut(population, "power population")?;
            population.insert("status".to_owned(), Value::String("complete".to_owned()));
            population.insert(
                "completed_source_families".to_owned(),
                Value::Array(
                    [
                        "state-form-and-political-membership",
                        "time-model",
                        "substantive-equality-and-anti-subordination",
                        "economic-pluralism-and-protected-private-sphere",
                        "family-dependency-reproduction-and-collective-plurality",
                        "ecological-commons-and-non-human-animal",
                        "public-safety-defence-emergency-and-external-power",
                        "current-formal-constitution",
                    ]
                    .into_iter()
                    .map(|value| Value::String(value.to_owned()))
                    .collect(),
                ),
            );
            value_object_mut(source, "ledger")?.insert(
                "powers".to_owned(),
                Value::Array(vec![serde_json::json!({"id": "FS-POW-99"})]),
            );
            let affected_claim_refs = profile_value_mut(source, "public-power-lifecycle")?
                .get("applies_to_claim_refs")
                .cloned()
                .unwrap_or(Value::Array(Vec::new()));
            value_object_mut(source, "ledger")?.insert(
                "function_allocations".to_owned(),
                Value::Array(vec![serde_json::json!({
                    "id": "FS-FAL-99",
                    "scope_id": "arbitrary",
                    "affected_claim_refs": affected_claim_refs,
                    "decider_refs": ["FS-BOD-01"],
                    "executor_refs": ["FS-BOD-02"],
                    "auditor_refs": ["FS-BOD-03"],
                    "final_remedy_refs": ["FS-BOD-04"],
                    "source_refs": ["new-book-plans/16-constitutional-closure.py::def validate_function_separation_row"]
                })]),
            );
            Ok(())
        },
    )?;

    add_control(
        ledger,
        &mut names,
        &mut controls,
        "same-signature edge cannot replace reviewed floor delivery",
        Some("semantic edge contract"),
        |source| {
            replace_string(
                component_value_mut(source, "floor-lifecycle", "delivery")?,
                "record_refs",
                "FS-DEP-25",
                "FS-DEP-30",
            )?;
            remove_string(
                row_value_mut(source, "dependencies", "id", "FS-DEP-25")?,
                "closure_component_refs",
                "FS-CLR-01:delivery",
            )?;
            push_string(
                row_value_mut(source, "dependencies", "id", "FS-DEP-30")?,
                "closure_component_refs",
                "FS-CLR-01:delivery",
            )
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "unused continuity edge cannot be reintroduced",
        Some("no claim-scoped consumer"),
        |source| reintroduce_dead_component(source, "floor-lifecycle", "continuity", "FS-DEP-57"),
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "constitutional record writer cannot be removed",
        Some("constitutional identity effects require"),
        |source| {
            remove_string(
                component_value_mut(source, "record-lifecycle", "writer")?,
                "record_refs",
                "FS-DEP-26",
            )?;
            remove_string(
                row_value_mut(source, "dependencies", "id", "FS-DEP-26")?,
                "closure_component_refs",
                "FS-CLR-04:writer",
            )
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "unused floor-boundary edge cannot be reintroduced",
        Some("no claim-scoped consumer"),
        |source| {
            reintroduce_dead_component(
                source,
                "democratic-floor-corridor",
                "floor-boundary",
                "FS-DEP-11",
            )
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "reader claim coverage moved",
        Some("profile membership drifts from claim contracts"),
        |source| {
            value_object_mut(
                profile_value_mut(source, "reader-claim-ownership")?,
                "reader profile",
            )?
            .insert(
                "applies_to_claim_refs".to_owned(),
                Value::Array(vec![Value::String("FS-CLM-36".to_owned())]),
            );
            remove_string(
                row_value_mut(source, "model_allocations", "claim_ref", "FS-CLM-37")?,
                "closure_profile_refs",
                "FS-CLR-08",
            )?;
            push_string(
                row_value_mut(source, "model_allocations", "claim_ref", "FS-CLM-36")?,
                "closure_profile_refs",
                "FS-CLR-08",
            )
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "reader owner binding moved",
        Some("formal-owner"),
        |source| {
            value_object_mut(
                component_value_mut(source, "reader-claim-ownership", "formal-owner")?,
                "reader formal owner",
            )?
            .insert(
                "record_refs".to_owned(),
                Value::Array(vec![Value::String("FS-CLM-36".to_owned())]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "external assumption hidden",
        None,
        |source| {
            value_object_mut(
                value_array_mut(source, "domains")?
                    .first_mut()
                    .ok_or_else(|| ClosureError::new("domains is empty"))?,
                "domain",
            )?
            .insert(
                "external_assumption_refs".to_owned(),
                Value::Array(vec![Value::String("FS-EXA-99".to_owned())]),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "formal claim route substituted",
        Some("primary route drift"),
        |source| {
            value_object_mut(row_value_mut(source, "claims", "id", "FS-CLM-04")?, "claim")?.insert(
                "route_ref".to_owned(),
                Value::String("FS-RTE-04".to_owned()),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "allocation and claim jointly substitute model",
        Some("required route composition drift"),
        |source| jointly_substitute_route(source, "FS-CLM-04", "FS-RTE-04"),
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "external claim jointly substituted to reader model",
        Some("required route composition drift"),
        |source| jointly_substitute_route(source, "FS-CLM-13", "FS-RTE-06"),
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "fully coordinated external-to-reader substitution",
        Some("intrinsic claim obligation"),
        |source| {
            jointly_substitute_route(source, "FS-CLM-13", "FS-RTE-06")?;
            push_string(
                profile_value_mut(source, "reader-claim-ownership")?,
                "applies_to_claim_refs",
                "FS-CLM-13",
            )?;
            push_string(
                component_value_mut(source, "reader-claim-ownership", "formal-owner")?,
                "record_refs",
                "FS-CLM-13",
            )?;
            push_string(
                row_value_mut(source, "claims", "id", "FS-CLM-13")?,
                "closure_requirement_refs",
                "FS-CLR-08",
            )?;
            push_string(
                row_value_mut(source, "model_allocations", "claim_ref", "FS-CLM-13")?,
                "closure_profile_refs",
                "FS-CLR-08",
            )?;
            push_string(
                claim_contract_value_mut(source, "FS-CLM-13")?,
                "required_profile_refs",
                "FS-CLR-08",
            )
        },
    )?;

    for (name, claim_ref, expected) in [
        (
            "external claim cannot be elevated onto formal route",
            "FS-CLM-16",
            "external claims must remain Unestablished",
        ),
        (
            "Book 2 operation cannot be elevated onto formal route",
            "FS-CLM-24",
            "Book 2 operation claims must remain Unestablished",
        ),
    ] {
        add_control(
            ledger,
            &mut names,
            &mut controls,
            name,
            Some(expected),
            move |source| {
                let claim = row_value_mut(source, "claims", "id", claim_ref)?;
                let claim = value_object_mut(claim, "claim")?;
                claim.insert("posture".to_owned(), Value::String("Derived".to_owned()));
                claim.insert(
                    "evidence_kind".to_owned(),
                    Value::String("executable".to_owned()),
                );
                claim.remove("unestablished_disposition");
                claim.insert(
                    "route_ref".to_owned(),
                    Value::String("FS-RTE-01".to_owned()),
                );
                let allocation =
                    row_value_mut(source, "model_allocations", "claim_ref", claim_ref)?;
                let allocation = value_object_mut(allocation, "allocation")?;
                allocation.insert(
                    "primary_route_ref".to_owned(),
                    Value::String("FS-RTE-01".to_owned()),
                );
                allocation.insert(
                    "required_route_refs".to_owned(),
                    Value::Array(vec![Value::String("FS-RTE-01".to_owned())]),
                );
                Ok(())
            },
        )?;
    }
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "live-record claim jointly substituted to quantitative model",
        Some("required route composition drift"),
        |source| jointly_substitute_route(source, "FS-CLM-20", "FS-RTE-02"),
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "composite route omitted",
        Some("composition drift"),
        |source| {
            value_object_mut(
                row_value_mut(source, "model_allocations", "claim_ref", "FS-CLM-24")?,
                "allocation",
            )?
            .insert(
                "required_route_refs".to_owned(),
                Value::Array(
                    ["FS-RTE-02", "FS-RTE-05"]
                        .into_iter()
                        .map(|value| Value::String(value.to_owned()))
                        .collect(),
                ),
            );
            Ok(())
        },
    )?;
    add_control(
        ledger,
        &mut names,
        &mut controls,
        "Book 2 composite cannot be erased through legacy-row drift",
        Some("required route composition drift"),
        |source| {
            let claim = row_value_mut(source, "claims", "id", "FS-CLM-24")?;
            let claim = value_object_mut(claim, "claim")?;
            claim.insert(
                "legacy_row_ref".to_owned(),
                Value::String("FS-LGR-10".to_owned()),
            );
            claim.insert(
                "route_ref".to_owned(),
                Value::String("FS-RTE-05".to_owned()),
            );
            let allocation = row_value_mut(source, "model_allocations", "claim_ref", "FS-CLM-24")?;
            let allocation = value_object_mut(allocation, "allocation")?;
            allocation.insert(
                "primary_route_ref".to_owned(),
                Value::String("FS-RTE-05".to_owned()),
            );
            allocation.insert(
                "required_route_refs".to_owned(),
                Value::Array(vec![Value::String("FS-RTE-05".to_owned())]),
            );
            Ok(())
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn context() -> Context {
        Context::from_test_root(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
    }

    #[test]
    fn control_audit_fixture_uses_the_native_current_audit_contract() {
        let loaded = super::super::load_source(&context()).expect("typed ledger source");
        let mut source = serde_json::to_value(&loaded.source).expect("ledger JSON value");
        append_control_audit_value(&mut source, "Native current audit")
            .expect("append control audit");
        let audit = source["scope_audits"]
            .as_array()
            .and_then(|audits| audits.last())
            .expect("appended scope audit");
        assert_eq!(
            audit["control_refs"],
            serde_json::json!([
                concat!("src/checks/ledger.rs::fn negative_", "controls("),
                concat!("src/checks/ledger/closure.rs::fn negative_", "controls(")
            ])
        );
        assert_eq!(
            audit["commands"],
            serde_json::json!([
                "./verify.sh --refresh full-society-ledger",
                "./verify.sh --refresh constitutional-closure",
                "./verify.sh --emit-receipt new-book-plans/verification-receipts"
            ])
        );
    }

    #[test]
    fn active_closure_renderer_reproduces_through_the_native_refresh_mode() {
        let loaded = super::super::load_source(&context()).expect("typed ledger source");
        let resolution =
            super::super::compute_resolution(&loaded.source).expect("defect resolution");
        let contract =
            validate_contract(&loaded.source, &resolution).expect("typed closure contract");
        let rendered = render(&loaded.source, &contract, &resolution).expect("closure report");
        assert!(rendered.contains(super::super::CLOSURE_REFRESH_COMMAND));
        assert!(!rendered.contains("python3 new-book-plans/16-constitutional-closure.py"));
    }

    fn assert_exact(actual: &str, expected: &str) {
        if actual.as_bytes() == expected.as_bytes() {
            return;
        }
        let mismatch = actual
            .bytes()
            .zip(expected.bytes())
            .position(|(actual, expected)| actual != expected)
            .unwrap_or_else(|| actual.len().min(expected.len()));
        let start = mismatch.saturating_sub(120);
        let actual_end = (mismatch + 240).min(actual.len());
        let expected_end = (mismatch + 240).min(expected.len());
        panic!(
            "closure report mismatch at byte {mismatch}; lengths {} != {}; actual {:?}; expected {:?}",
            actual.len(),
            expected.len(),
            &actual[start..actual_end],
            &expected[start..expected_end],
        );
    }

    #[test]
    fn live_typed_contract_and_report_match_the_reviewed_fixture() {
        let context = context();
        let ledger = super::super::load_and_validate(&context).expect("validated ledger");
        let contract = validate_contract(&ledger.document, &ledger.resolutions)
            .expect("typed closure contract");
        let actual = render(&ledger.document, &contract, &ledger.resolutions)
            .expect("rendered closure report");
        let expected = context.read(OUTPUT).expect("reviewed closure report");
        assert_exact(&actual, &expected);
        let checked = check_validated(&context, &ledger).expect("live closure check");
        assert_eq!(checked.controls, STRUCTURAL_CONTROL_COUNT);
        assert_eq!(
            checked.message,
            concat!(
                "new-book-plans/constitutional-closure-and-model-allocation-audit.md is current; ",
                "74 watched-failing structural controls pass; claim results are contract-only; Gate A passed"
            )
        );
    }

    #[test]
    fn coordinated_formal_route_substitution_is_rejected_by_typed_semantics() {
        let context = context();
        let ledger = super::super::load_and_validate(&context).expect("validated ledger");
        let mut changed = ledger.document.clone();
        let claim = changed
            .claims
            .iter_mut()
            .find(|claim| claim.id == "FS-CLM-04")
            .expect("FS-CLM-04");
        claim.route_ref = "FS-RTE-04".to_owned();
        let allocation = changed
            .model_allocations
            .iter_mut()
            .find(|allocation| allocation.claim_ref == "FS-CLM-04")
            .expect("FS-CLM-04 allocation");
        allocation.primary_route_ref = "FS-RTE-04".to_owned();
        allocation.required_route_refs = vec!["FS-RTE-04".to_owned()];
        let error = validate_contract(&changed, &ledger.resolutions)
            .expect_err("coordinated substitution must fail");
        assert!(
            error
                .to_string()
                .contains("required route composition drift")
        );
    }
}
