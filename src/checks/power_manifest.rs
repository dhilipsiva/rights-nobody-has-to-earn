// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::cli::Error;
use crate::context::Context;
use crate::digest::sha256;

pub(crate) const STEP_NAME: &str = "full-society power source manifest";

const SOURCE_PATH: &str = "new-book-plans/full-society-power-source-manifest.json";
const EXPECTED_MANIFEST_SHA256: &str =
    "2a664fa968423e1ffeec6036422600cc249aa7972258482978b921417ec5f67a";
const EXPECTED_SOURCE_COMMIT: &str = "36ed92c58877cffa5a11928ad200f0ca9a604820";
const STATUS: &str = concat!(
    "reviewed-inventory-input-not-law-not-operation-",
    "not-completeness-beyond-bound-version"
);
const REVIEWED_ROW_COUNT: usize = 237;

const ALLOWED_DISPOSITIONS: [&str; 4] = [
    "card-required",
    "power-contract-template",
    "existing-formal-crosswalk",
    "explicit-refusal-limit",
];

const SOURCE_FAMILIES: [(&str, &str); 8] = [
    (
        "new-book-plans/constitution.nibli",
        "current-formal-constitution",
    ),
    (
        "new-book-plans/book-1-state-form-and-political-membership-decision.md",
        "state-form-and-political-membership",
    ),
    (
        "new-book-plans/book-1-substantive-equality-and-anti-subordination-decision.md",
        "substantive-equality-and-anti-subordination",
    ),
    (
        "new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md",
        "economic-pluralism-and-protected-private-sphere",
    ),
    (
        "new-book-plans/book-1-family-dependency-reproduction-and-collective-plurality-decision.md",
        "family-dependency-reproduction-and-collective-plurality",
    ),
    (
        "new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md",
        "ecological-commons-and-non-human-animal",
    ),
    (
        "new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md",
        "public-safety-defence-emergency-and-external-power",
    ),
    ("new-book-plans/book-1-time-model-decision.md", "time-model"),
];

const EXPECTED_SOURCE_SHA256: [(&str, &str); 8] = [
    (
        "new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md",
        "d9bda040307eed017b55be751b2a99a73910568b0e8fbafb27c5c185c4f5b13c",
    ),
    (
        "new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md",
        "07b7d79df6069d71293c824a12f85f67aa487c9428b2f5253da6eb50e61ffa51",
    ),
    (
        "new-book-plans/book-1-family-dependency-reproduction-and-collective-plurality-decision.md",
        "6b42ef36e0ab54b8391f1d2bc174836a8f6f6130dfba4786c8e0483c10b34c59",
    ),
    (
        "new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md",
        "383790a454415a79f8769923b7cb56a2c7eecad8da4edee9fa010e5e54576a0c",
    ),
    (
        "new-book-plans/book-1-state-form-and-political-membership-decision.md",
        "ceed78c9df3ca1a60d993a3684058fb977042c7b2b8904d99ae697a583592f70",
    ),
    (
        "new-book-plans/book-1-substantive-equality-and-anti-subordination-decision.md",
        "c27e4175ccbd3034a04d9cebbd80d314400fc290cff07997c1e314d04db28979",
    ),
    (
        "new-book-plans/book-1-time-model-decision.md",
        "b164e60532a2179b12ff675050007ee88b8a17fe20a3d6b02e07fb74873d68f6",
    ),
    (
        "new-book-plans/constitution.nibli",
        "b4c0b0b6778c8c5ed414f9771b8f3004b20601f821f6f15c12c8c0d40bb50f62",
    ),
];

const EXPECTED_BY_DISPOSITION: [(&str, usize); 4] = [
    ("card-required", 209),
    ("power-contract-template", 1),
    ("existing-formal-crosswalk", 8),
    ("explicit-refusal-limit", 19),
];

const EXPECTED_BY_FAMILY: [(&str, usize); 8] = [
    ("current-formal-constitution", 8),
    ("ecological-commons-and-non-human-animal", 43),
    ("economic-pluralism-and-protected-private-sphere", 29),
    (
        "family-dependency-reproduction-and-collective-plurality",
        31,
    ),
    ("public-safety-defence-emergency-and-external-power", 64),
    ("state-form-and-political-membership", 51),
    ("substantive-equality-and-anti-subordination", 9),
    ("time-model", 2),
];

// Counts are in ALLOWED_DISPOSITIONS order.
const EXPECTED_MATRIX: [(&str, [usize; 4]); 8] = [
    ("current-formal-constitution", [0, 0, 8, 0]),
    ("ecological-commons-and-non-human-animal", [40, 0, 0, 3]),
    (
        "economic-pluralism-and-protected-private-sphere",
        [28, 0, 0, 1],
    ),
    (
        "family-dependency-reproduction-and-collective-plurality",
        [31, 0, 0, 0],
    ),
    (
        "public-safety-defence-emergency-and-external-power",
        [50, 0, 0, 14],
    ),
    ("state-form-and-political-membership", [51, 0, 0, 0]),
    ("substantive-equality-and-anti-subordination", [9, 0, 0, 0]),
    ("time-model", [0, 1, 0, 1]),
];

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Manifest {
    spdx: String,
    schema_version: u64,
    title: String,
    status: String,
    source_commit: String,
    source_sha256: BTreeMap<String, String>,
    allowed_dispositions: Vec<String>,
    grain_rule_anchor: String,
    scope_note: String,
    row_count: usize,
    coverage_summary: CoverageSummary,
    rows: Vec<Row>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CoverageSummary {
    by_disposition: BTreeMap<String, usize>,
    by_source_family: BTreeMap<String, usize>,
    by_source_family_and_disposition: BTreeMap<String, BTreeMap<String, usize>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Row {
    provisional_key: String,
    title: String,
    disposition: String,
    source_anchor: String,
    source_path: String,
    source_needle: String,
    legal_effect_and_grain: String,
    source_family: String,
}

#[derive(Clone, Debug)]
struct SourceSnapshot {
    digest: String,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub(crate) reviewed_rows: usize,
    pub(crate) card_required: usize,
    pub(crate) contract_templates: usize,
    pub(crate) refusals_or_limits: usize,
    pub(crate) current_formal_crosswalks: usize,
    pub(crate) watched_mutations: usize,
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "full-society power source manifest is current: {} reviewed rows \
             ({} card-required powers, {} cross-power contract template, \
             {} refusal/limit, {} current-formal crosswalk); \
             {} watched-failing mutations pass; inventory only -- no law, \
             operation, FS-POW completion, or Gate A result",
            self.reviewed_rows,
            self.card_required,
            self.contract_templates,
            self.refusals_or_limits,
            self.current_formal_crosswalks,
            self.watched_mutations,
        )
    }
}

pub(crate) fn run(context: &Context) -> Result<(), Error> {
    println!("{}", check(context)?);
    Ok(())
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    check_root(context.root(), true)
}

fn check_root(root: &Path, check_git: bool) -> Result<Report, Error> {
    let manifest_path = root.join(SOURCE_PATH);
    let raw = std::fs::read(&manifest_path).map_err(|error| {
        manifest_error(format!("cannot load {}: {error}", manifest_path.display()))
    })?;
    if sha256(&raw) != EXPECTED_MANIFEST_SHA256 {
        return Err(manifest_error(
            "reviewed manifest digest differs from the checker-bound artifact",
        ));
    }
    let manifest: Manifest = serde_json::from_slice(&raw).map_err(|error| {
        manifest_error(format!("cannot load {}: {error}", manifest_path.display()))
    })?;
    let sources = load_source_snapshots(root)?;
    validate(root, &manifest, &sources, check_git)?;
    let watched_mutations = negative_controls(root, &manifest, &sources)?;

    Ok(Report {
        reviewed_rows: REVIEWED_ROW_COUNT,
        card_required: EXPECTED_BY_DISPOSITION[0].1,
        contract_templates: EXPECTED_BY_DISPOSITION[1].1,
        current_formal_crosswalks: EXPECTED_BY_DISPOSITION[2].1,
        refusals_or_limits: EXPECTED_BY_DISPOSITION[3].1,
        watched_mutations,
    })
}

fn load_source_snapshots(root: &Path) -> Result<BTreeMap<String, SourceSnapshot>, Error> {
    let mut result = BTreeMap::new();
    for (relative, _) in EXPECTED_SOURCE_SHA256 {
        let bytes = std::fs::read(root.join(relative))
            .map_err(|_| manifest_error(format!("source digest mismatch: {relative}")))?;
        let digest = sha256(&bytes);
        let text = String::from_utf8(bytes)
            .map_err(|_| manifest_error(format!("source is not valid UTF-8: {relative}")))?;
        result.insert(relative.to_owned(), SourceSnapshot { digest, text });
    }
    Ok(result)
}

fn validate(
    root: &Path,
    source: &Manifest,
    sources: &BTreeMap<String, SourceSnapshot>,
    check_git: bool,
) -> Result<(), Error> {
    if source.spdx != "CC0-1.0" || source.schema_version != 1 {
        return Err(manifest_error(
            "manifest licence/schema must be CC0-1.0 / version 1",
        ));
    }
    if source.title != "Provisional FS-POW source-anchor manifest" {
        return Err(manifest_error("manifest title is not the reviewed title"));
    }
    if source.status != STATUS {
        return Err(manifest_error(
            "manifest status must preserve the non-law ceiling",
        ));
    }
    if source.source_commit != EXPECTED_SOURCE_COMMIT {
        return Err(manifest_error(
            "source_commit differs from the reviewed base",
        ));
    }
    if source.source_sha256 != expected_source_digests() {
        return Err(manifest_error(
            "source_sha256 differs from the reviewed source set",
        ));
    }
    if source.allowed_dispositions != expected_allowed_dispositions() {
        return Err(manifest_error(
            "allowed_dispositions must equal the closed vocabulary",
        ));
    }
    require_text(&source.grain_rule_anchor, "grain_rule_anchor")?;
    require_text(&source.scope_note, "scope_note")?;
    if [
        "creates no law",
        "Gate A result",
        "power-contract-template rows constrain cards",
    ]
    .iter()
    .any(|term| !source.scope_note.contains(term))
    {
        return Err(manifest_error("scope_note lost an inventory-only boundary"));
    }

    if source.rows.len() != REVIEWED_ROW_COUNT || source.row_count != REVIEWED_ROW_COUNT {
        return Err(manifest_error(
            "manifest must contain the reviewed 237-row population",
        ));
    }
    let summary = derived_summary(&source.rows);
    if source.coverage_summary != summary {
        return Err(manifest_error("coverage_summary is stale relative to rows"));
    }
    if summary.by_disposition != expected_disposition_counts() {
        return Err(manifest_error(
            "disposition totals differ from the reviewed population",
        ));
    }
    if summary.by_source_family != expected_family_counts() {
        return Err(manifest_error(
            "source-family totals differ from the reviewed population",
        ));
    }
    if summary.by_source_family_and_disposition != expected_matrix() {
        return Err(manifest_error(
            "family/disposition matrix differs from the reviewed population",
        ));
    }

    for (relative, expected) in EXPECTED_SOURCE_SHA256 {
        let snapshot = sources
            .get(relative)
            .ok_or_else(|| manifest_error(format!("source digest mismatch: {relative}")))?;
        if snapshot.digest != expected {
            return Err(manifest_error(format!(
                "source digest mismatch: {relative}"
            )));
        }
    }

    let mut keys = HashSet::new();
    let mut titles = HashSet::new();
    for (index, row) in source.rows.iter().enumerate() {
        let context = format!("rows[{index}]");
        let key = require_text(&row.provisional_key, &format!("{context}.provisional_key"))?;
        if !valid_provisional_key(key) {
            return Err(manifest_error(format!(
                "{context}: invalid provisional_key"
            )));
        }
        if !keys.insert(key.to_owned()) {
            return Err(manifest_error(format!(
                "{context}: duplicate provisional_key {key}"
            )));
        }
        let title = require_text(&row.title, &format!("{context}.title"))?;
        let normalized = normalize_title(title);
        if !titles.insert(normalized) {
            return Err(manifest_error(format!(
                "{context}: duplicate normalized title"
            )));
        }
        if !ALLOWED_DISPOSITIONS.contains(&row.disposition.as_str()) {
            return Err(manifest_error(format!(
                "{context}: unknown disposition {:?}",
                row.disposition
            )));
        }
        let relative = require_text(&row.source_path, &format!("{context}.source_path"))?;
        let expected_family = source_family(relative).ok_or_else(|| {
            manifest_error(format!(
                "{context}: source_path outside reviewed source set"
            ))
        })?;
        if row.source_family != expected_family {
            return Err(manifest_error(format!(
                "{context}: source_family mismatches source_path"
            )));
        }
        let is_constitution = relative == "new-book-plans/constitution.nibli";
        let is_crosswalk = row.disposition == "existing-formal-crosswalk";
        if is_constitution != is_crosswalk {
            return Err(manifest_error(format!(
                "{context}: current-formal crosswalk disposition is misclassified"
            )));
        }
        let needle = require_text(&row.source_needle, &format!("{context}.source_needle"))?;
        if row.source_anchor != format!("{relative}::{needle}") {
            return Err(manifest_error(format!(
                "{context}: source_anchor is not path::needle exact"
            )));
        }
        let snapshot = sources.get(relative).ok_or_else(|| {
            manifest_error(format!(
                "{context}: source_path outside reviewed source set"
            ))
        })?;
        if snapshot.text.matches(needle).count() != 1 {
            return Err(manifest_error(format!(
                "{context}: source needle must occur exactly once"
            )));
        }
        require_text(
            &row.legal_effect_and_grain,
            &format!("{context}.legal_effect_and_grain"),
        )?;
    }

    let (grain_path, grain_needle) = source
        .grain_rule_anchor
        .split_once("::")
        .ok_or_else(|| manifest_error("grain_rule_anchor must use path::needle"))?;
    if grain_path != "new-book-plans/book-1-constitutional-coverage-map.md" {
        return Err(manifest_error(
            "grain_rule_anchor must name the coverage map",
        ));
    }
    let grain_text = std::fs::read_to_string(root.join(grain_path)).map_err(|error| {
        manifest_error(format!(
            "cannot read grain-rule source {grain_path}: {error}"
        ))
    })?;
    if grain_text.matches(grain_needle).count() != 1 {
        return Err(manifest_error(
            "grain_rule_anchor must resolve exactly once",
        ));
    }

    if check_git {
        let status = Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "merge-base",
                "--is-ancestor",
                source.source_commit.as_str(),
                "HEAD",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| manifest_error(format!("cannot run git: {error}")))?;
        if !status.success() {
            return Err(manifest_error("source_commit must be an ancestor of HEAD"));
        }
    }

    Ok(())
}

fn negative_controls(
    root: &Path,
    source: &Manifest,
    sources: &BTreeMap<String, SourceSnapshot>,
) -> Result<usize, Error> {
    let mut candidates = Vec::new();

    let mut candidate = source.clone();
    candidate.rows.pop();
    candidates.push(("row removed", candidate));

    let mut candidate = source.clone();
    candidate.row_count = 236;
    candidates.push(("reviewed row count changed", candidate));

    let mut candidate = source.clone();
    candidate
        .coverage_summary
        .by_disposition
        .insert("card-required".to_owned(), 208);
    candidates.push(("summary changed", candidate));

    let mut candidate = source.clone();
    candidate.rows[1].provisional_key = candidate.rows[0].provisional_key.clone();
    candidates.push(("duplicate key", candidate));

    let mut candidate = source.clone();
    candidate.rows[1].title = candidate.rows[0].title.clone();
    candidates.push(("duplicate title", candidate));

    let mut candidate = source.clone();
    candidate.rows[0].disposition = "passed".to_owned();
    candidates.push(("unknown disposition", candidate));

    let mut candidate = source.clone();
    candidate.rows[0].source_family = "time-model".to_owned();
    candidates.push(("source family drift", candidate));

    let mut candidate = source.clone();
    candidate.rows[0].source_anchor = "TODO.md::missing".to_owned();
    candidates.push(("source anchor drift", candidate));

    let mut candidate = source.clone();
    candidate.rows[0].source_needle = "definitely absent".to_owned();
    candidates.push(("missing source needle", candidate));

    let mut candidate = source.clone();
    if let Some((_, digest)) = candidate.source_sha256.iter_mut().next() {
        *digest = "0".repeat(64);
    }
    candidates.push(("source digest drift", candidate));

    let mut candidate = source.clone();
    candidate.scope_note = "Inventory.".to_owned();
    candidates.push(("ceiling removed", candidate));

    let mut candidate = source.clone();
    let template = candidate
        .rows
        .iter_mut()
        .find(|row| row.provisional_key == "time-power-specific-t3-contract")
        .ok_or_else(|| manifest_error("reviewed template row is missing"))?;
    template.disposition = "card-required".to_owned();
    candidates.push(("template promoted to power", candidate));

    let mut candidate = source.clone();
    candidate.status = "complete".to_owned();
    candidates.push(("status promoted", candidate));

    let mut candidate = source.clone();
    candidate.source_commit = "0".repeat(40);
    candidates.push(("source commit drift", candidate));

    for (name, candidate) in &candidates {
        if validate(root, candidate, sources, false).is_ok() {
            return Err(manifest_error(format!(
                "negative control did not fail: {name}"
            )));
        }
    }
    Ok(candidates.len())
}

fn derived_summary(rows: &[Row]) -> CoverageSummary {
    let mut by_disposition = expected_zero_disposition_counts();
    let mut by_source_family = BTreeMap::new();
    let mut by_source_family_and_disposition = BTreeMap::new();
    for (_, family) in SOURCE_FAMILIES {
        by_source_family.entry(family.to_owned()).or_insert(0);
        by_source_family_and_disposition
            .entry(family.to_owned())
            .or_insert_with(expected_zero_disposition_counts);
    }
    for row in rows {
        if let Some(count) = by_disposition.get_mut(&row.disposition) {
            *count += 1;
        }
        if let Some(count) = by_source_family.get_mut(&row.source_family) {
            *count += 1;
        }
        if let Some(by_family) = by_source_family_and_disposition.get_mut(&row.source_family)
            && let Some(count) = by_family.get_mut(&row.disposition)
        {
            *count += 1;
        }
    }
    CoverageSummary {
        by_disposition,
        by_source_family,
        by_source_family_and_disposition,
    }
}

fn expected_allowed_dispositions() -> Vec<String> {
    ALLOWED_DISPOSITIONS
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn expected_zero_disposition_counts() -> BTreeMap<String, usize> {
    ALLOWED_DISPOSITIONS
        .into_iter()
        .map(|disposition| (disposition.to_owned(), 0))
        .collect()
}

fn expected_disposition_counts() -> BTreeMap<String, usize> {
    EXPECTED_BY_DISPOSITION
        .into_iter()
        .map(|(name, count)| (name.to_owned(), count))
        .collect()
}

fn expected_family_counts() -> BTreeMap<String, usize> {
    EXPECTED_BY_FAMILY
        .into_iter()
        .map(|(name, count)| (name.to_owned(), count))
        .collect()
}

fn expected_matrix() -> BTreeMap<String, BTreeMap<String, usize>> {
    EXPECTED_MATRIX
        .into_iter()
        .map(|(family, counts)| {
            let dispositions = ALLOWED_DISPOSITIONS
                .into_iter()
                .zip(counts)
                .map(|(disposition, count)| (disposition.to_owned(), count))
                .collect();
            (family.to_owned(), dispositions)
        })
        .collect()
}

fn expected_source_digests() -> BTreeMap<String, String> {
    EXPECTED_SOURCE_SHA256
        .into_iter()
        .map(|(path, digest)| (path.to_owned(), digest.to_owned()))
        .collect()
}

fn source_family(path: &str) -> Option<&'static str> {
    SOURCE_FAMILIES
        .iter()
        .find_map(|(candidate, family)| (*candidate == path).then_some(*family))
}

fn require_text<'a>(value: &'a str, context: &str) -> Result<&'a str, Error> {
    if value.trim().is_empty() {
        return Err(manifest_error(format!(
            "{context}: non-empty text required"
        )));
    }
    Ok(value)
}

fn valid_provisional_key(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn normalize_title(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            part.chars()
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn manifest_error(message: impl Into<String>) -> Error {
    Error::new(format!("power manifest error: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    fn fixture() -> (Manifest, BTreeMap<String, SourceSnapshot>) {
        let raw = std::fs::read(root().join(SOURCE_PATH)).expect("read reviewed manifest");
        let source = serde_json::from_slice(&raw).expect("parse reviewed manifest");
        let sources = load_source_snapshots(root()).expect("load reviewed sources");
        (source, sources)
    }

    #[test]
    fn reviewed_manifest_and_git_history_pass() {
        let report = check_root(root(), true).expect("reviewed manifest must pass");
        assert_eq!(report.reviewed_rows, 237);
        assert_eq!(report.watched_mutations, 14);
    }

    #[test]
    fn report_text_is_byte_for_byte_python_parity() {
        let report = Report {
            reviewed_rows: 237,
            card_required: 209,
            contract_templates: 1,
            refusals_or_limits: 19,
            current_formal_crosswalks: 8,
            watched_mutations: 14,
        };
        assert_eq!(
            report.to_string(),
            "full-society power source manifest is current: 237 reviewed rows \
             (209 card-required powers, 1 cross-power contract template, \
             19 refusal/limit, 8 current-formal crosswalk); \
             14 watched-failing mutations pass; inventory only -- no law, \
             operation, FS-POW completion, or Gate A result"
        );
    }

    #[test]
    fn all_fourteen_python_negative_controls_fail() {
        let (source, sources) = fixture();
        assert_eq!(
            negative_controls(root(), &source, &sources).expect("controls must be watched"),
            14
        );
    }

    #[test]
    fn derived_summary_matches_reviewed_three_way_census() {
        let (source, _) = fixture();
        let summary = derived_summary(&source.rows);
        assert_eq!(summary, source.coverage_summary);
        assert_eq!(summary.by_disposition, expected_disposition_counts());
        assert_eq!(summary.by_source_family, expected_family_counts());
        assert_eq!(summary.by_source_family_and_disposition, expected_matrix());
    }

    #[test]
    fn duplicate_titles_are_case_and_whitespace_normalized() {
        let (mut source, sources) = fixture();
        source.rows[1].title = format!("  {}  ", source.rows[0].title.to_uppercase());
        let error = validate(root(), &source, &sources, false)
            .expect_err("normalized duplicate title must fail");
        assert!(error.to_string().contains("duplicate normalized title"));
    }

    #[test]
    fn missing_needle_fails_even_with_a_matching_anchor() {
        let (mut source, sources) = fixture();
        source.rows[0].source_needle = "definitely absent".to_owned();
        source.rows[0].source_anchor = format!(
            "{}::{}",
            source.rows[0].source_path, source.rows[0].source_needle
        );
        let error = validate(root(), &source, &sources, false)
            .expect_err("missing source needle must fail");
        assert!(error.to_string().contains("must occur exactly once"));
    }

    #[test]
    fn exact_schema_rejects_an_extra_top_level_key() {
        let raw = std::fs::read(root().join(SOURCE_PATH)).expect("read reviewed manifest");
        let mut value: serde_json::Value = serde_json::from_slice(&raw).expect("parse JSON");
        value
            .as_object_mut()
            .expect("manifest object")
            .insert("unreviewed".to_owned(), serde_json::Value::Bool(true));
        let error = serde_json::from_value::<Manifest>(value)
            .expect_err("unreviewed top-level key must fail");
        assert!(error.to_string().contains("unknown field `unreviewed`"));
    }

    #[test]
    fn provisional_keys_match_the_python_ascii_grammar() {
        for valid in ["a", "a1", "time-power-specific-t3-contract"] {
            assert!(valid_provisional_key(valid), "{valid}");
        }
        for invalid in ["", "-a", "a-", "a--b", "Upper", "é", "white space"] {
            assert!(!valid_provisional_key(invalid), "{invalid}");
        }
    }
}
