// SPDX-License-Identifier: MIT OR Apache-2.0

//! Content-addressed authoritative-verification receipts.
//!
//! The tracked receipt is intentionally compact.  Its expanded path manifest,
//! command record, and transcript live below the Git common directory.  New
//! receipts bind this executable (which embeds the Nibli engine) while the
//! source fields retain the exact Nibli checkout provenance used by the build.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::Error;
use crate::context::Context;
use crate::digest::{canonical_json, sha256};
use crate::report::TeeWriter;

const RECEIPT_DIRECTORY: &str = "new-book-plans/verification-receipts";
const LEDGER_PATH: &str = "new-book-plans/full-society-ledger.json";
const TODO_PATH: &str = "TODO.md";
const PROTOCOL_PATH: &str = "new-book-plans/full-society-scope-review-protocol.md";
const PROTOCOL_VERSION: u8 = 6;
const PROTOCOL_STATUS: &str =
    "repository-enforced 2026-08-27 -- receipt-aware mechanical-closure protocol v6";
const HISTORICAL_PROTOCOL_V5_VERSION: u8 = 5;
const HISTORICAL_PROTOCOL_V5_STATUS: &str =
    "repository-enforced 2026-08-23 -- receipt-aware mechanical-closure protocol v5";
const EVIDENCE_CEILING: &str = "Repository verification over the bound staged bytes only; no external truth, operation, delivery, liveness, feasibility, calibration, or institutional action follows.";
const EVIDENCE_SUBDIRECTORY: &str = "rights-verification/receipts";
const FULL_COMMAND: &str = "./verify.sh";
const NIBLI_DEPENDENCY_CRATES: &[&str] = &[
    "nibli-engine",
    "nibli-kr",
    "nibli-lexicon",
    "nibli-protocol",
    "nibli-reason",
    "nibli-render",
    "nibli-semantics",
    "nibli-session",
    "nibli-store",
    "nibli-types",
];

const LEGACY_CANDIDATE: &str = "e0e0ca1a09dc8bceaac95f29ab5f1afdc9795bb5";
const LEGACY_SOURCE_VERSION: &str = "fs-ledger-2026-08-21-state-form-prose-v1";
const LEGACY_AUDIT_ID: &str = "FS-SAU-34";
const LEGACY_TRANSCRIPT_SHA256: &str =
    "dc0eb1d869629a9093457fcc8a7c48d5a438777bae756e24a0447e4d60e1032f";
const LEGACY_REQUIRED_COMMANDS: [&str; 10] = [
    "python3 new-book-plans/14-reader-evidence.py --check",
    "python3 new-book-plans/14-reader-evidence.py --check --execute",
    "python3 new-book-plans/17-full-society-power-source-manifest.py --check",
    "python3 new-book-plans/13-full-society-ledger.py",
    "python3 new-book-plans/13-full-society-ledger.py --check",
    "python3 new-book-plans/16-constitutional-closure.py",
    "python3 new-book-plans/16-constitutional-closure.py --check",
    "./verify.sh --quick",
    "./verify.sh",
    "git diff --check",
];

const FORWARD_RECOVERY_AUDIT_ID: &str = "FS-SAU-42";
const FORWARD_RECOVERY_CLOSED_ANCHOR: &str = "6de1ddc9af5b7265157edc419889aa48e5010503";
const FORWARD_RECOVERY_CLOSED_SOURCE_VERSION: &str = "fs-ledger-2026-08-26-delivery-lifecycle-v1";
const FORWARD_RECOVERY_FIRST_CANDIDATE: &str = "fc22780d3e560c7bb0abd3aab56cfff401d18dc2";
const FORWARD_RECOVERY_FIRST_AUDIT: &str = "52336cc72b65bc0f8e73f035f38c3c44cea1951a";
const FORWARD_RECOVERY_FIRST_RECEIPT: &str = "new-book-plans/verification-receipts/sha256-62edd4996c9928ce47a9f248f3ef19654996b8d655c99302e83f7eecffc4a297.json";
const FORWARD_RECOVERY_FIRST_SOURCE_VERSION: &str = "fs-ledger-2026-08-27-native-verifier-v1";
const FORWARD_RECOVERY_FIRST_AUDIT_ID: &str = "FS-SAU-40";
const FORWARD_RECOVERY_SECOND_CANDIDATE: &str = "7b7852df158ced0a9d67088134474c91daa9355b";
const FORWARD_RECOVERY_SECOND_AUDIT: &str = "4612d080477efee5df33cc79e6cd953f035696b6";
const FORWARD_RECOVERY_SECOND_RECEIPT: &str = "new-book-plans/verification-receipts/sha256-2c4f5879c901ca7f5ef3f4852893e91f1dc87ea26f0694efeebc10f70bcbd8ce.json";
const FORWARD_RECOVERY_SECOND_SOURCE_VERSION: &str = "fs-ledger-2026-08-27-native-refresh-v1";
const FORWARD_RECOVERY_SECOND_AUDIT_ID: &str = "FS-SAU-41";

const GENERATED_PATHS: [&str; 11] = [
    "new-book-plans/3-spine.md",
    "new-book-plans/amendment-semantics-audit.md",
    "new-book-plans/assertion-surface-audit.md",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md",
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
    "new-book-plans/placement-exhaustiveness-audit.md",
    "new-book-plans/reader-evidence.md",
    "new-book-plans/record-integrity-assurance-case.md",
    "new-book-plans/record-integrity-red-team.md",
    "new-book-plans/temporal-assurance-case.md",
];
const ADMINISTRATIVE_PATHS: [&str; 5] = [
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    TODO_PATH,
    PROTOCOL_PATH,
];
const AUDIT_GENERATED_PATHS: [&str; 2] = [
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
];

fn receipt_error(message: impl Into<String>) -> Error {
    Error::new(format!("verification receipt: {}", message.into()))
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub(crate) struct Sha256(String);

impl Sha256 {
    fn of(bytes: impl AsRef<[u8]>) -> Self {
        Self(sha256(bytes))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Sha256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if is_lower_hex(&value, 64) {
            Ok(Self(value))
        } else {
            Err(de::Error::custom("must be lowercase SHA-256"))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub(crate) struct GitSha(String);

impl GitSha {
    fn parse(value: impl Into<String>, context: &str) -> Result<Self, Error> {
        let value = value.into();
        if is_lower_hex(&value, 40) {
            Ok(Self(value))
        } else {
            Err(receipt_error(format!("{context} must be a Git SHA-1")))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GitSha {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if is_lower_hex(&value, 40) {
            Ok(Self(value))
        } else {
            Err(de::Error::custom("must be a lowercase Git SHA-1"))
        }
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(transparent)]
struct UtcTimestamp(String);

impl UtcTimestamp {
    fn now() -> Result<Self, Error> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| receipt_error(format!("system clock predates Unix epoch: {error}")))?
            .as_secs() as i64;
        let days = seconds.div_euclid(86_400);
        let within = seconds.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        Ok(Self(format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
            within / 3_600,
            (within % 3_600) / 60,
            within % 60
        )))
    }
}

impl<'de> Deserialize<'de> for UtcTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if valid_utc_seconds(&value) {
            Ok(Self(value))
        } else {
            Err(de::Error::custom("must be canonical valid UTC seconds"))
        }
    }
}

fn valid_utc_seconds(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let number = |start: usize, end: usize| -> Option<u32> {
        let slice = &bytes[start..end];
        slice.iter().all(u8::is_ascii_digit).then(|| {
            slice
                .iter()
                .fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
        })
    };
    let Some(year) = number(0, 4) else {
        return false;
    };
    let Some(month) = number(5, 7) else {
        return false;
    };
    let Some(day) = number(8, 10) else {
        return false;
    };
    let Some(hour) = number(11, 13) else {
        return false;
    };
    let Some(minute) = number(14, 16) else {
        return false;
    };
    let Some(second) = number(17, 19) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    (1..=12).contains(&month)
        && day >= 1
        && day <= month_days[(month - 1) as usize]
        && hour < 24
        && minute < 60
        && second < 60
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct RequiredNullable<T>(Option<T>);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManifestClass {
    Source,
    VerifierInput,
    Fixture,
    GeneratedArtifact,
    Administrative,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GitObjectKind {
    Blob,
    Commit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: GitObjectKind,
    object: GitSha,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassifiedManifestEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: GitObjectKind,
    object: GitSha,
    class: ManifestClass,
}

impl ClassifiedManifestEntry {
    fn raw(&self) -> ManifestEntry {
        ManifestEntry {
            path: self.path.clone(),
            mode: self.mode.clone(),
            kind: self.kind.clone(),
            object: self.object.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassManifest {
    count: usize,
    sha256: Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassManifests {
    source: ClassManifest,
    #[serde(rename = "verifier-input")]
    verifier_input: ClassManifest,
    fixture: ClassManifest,
    #[serde(rename = "generated-artifact")]
    generated_artifact: ClassManifest,
    administrative: ClassManifest,
}

impl ClassManifests {
    fn total(&self) -> usize {
        self.source.count
            + self.verifier_input.count
            + self.fixture.count
            + self.generated_artifact.count
            + self.administrative.count
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompactCandidate {
    parent_commit_sha: GitSha,
    tree_sha: GitSha,
    path_manifest_sha256: Sha256,
    path_count: usize,
    class_manifests: ClassManifests,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpandedCandidate {
    parent_commit_sha: GitSha,
    tree_sha: GitSha,
    path_manifest_sha256: Sha256,
    path_manifest: Vec<ClassifiedManifestEntry>,
    class_manifests: ClassManifests,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationResult {
    command: String,
    exit_code: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompactVerification {
    command_sha256: Sha256,
    transcript_sha256: Sha256,
    started_at_utc: UtcTimestamp,
    finished_at_utc: UtcTimestamp,
    results: Vec<VerificationResult>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandRecord {
    display: String,
    argv_sha256: Sha256,
    started_at_utc: UtcTimestamp,
    finished_at_utc: UtcTimestamp,
    elapsed_milliseconds: u128,
    exit_code: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpandedVerification {
    commands: Vec<CommandRecord>,
    result: String,
    transcript_sha256: Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EngineIdentity {
    binary_basename: String,
    binary_path_sha256: Sha256,
    binary_sha256: Sha256,
    binary_size: usize,
    source_override: bool,
    source_available: bool,
    source_commit_sha: RequiredNullable<GitSha>,
    source_dirty: RequiredNullable<bool>,
    source_status_sha256: RequiredNullable<Sha256>,
    source_diff_sha256: RequiredNullable<Sha256>,
    source_untracked_count: RequiredNullable<usize>,
    source_untracked_sha256: RequiredNullable<Sha256>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformIdentity {
    system: String,
    release: String,
    machine: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvironmentDetails {
    allowlisted_values: BTreeMap<String, String>,
    hashed_values: BTreeMap<String, Sha256>,
    platform: PlatformIdentity,
    tools: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpandedEnvironment {
    details: EnvironmentDetails,
    sha256: Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompactEnvironment {
    sha256: Sha256,
    fields: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalEvidence {
    expanded_manifest_sha256: Sha256,
    transcript_sha256: Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactReceipt {
    spdx: String,
    schema_version: u8,
    protocol_version: u8,
    protocol_status: String,
    receipt_id: Sha256,
    status: String,
    source_version: String,
    audit_id: String,
    candidate: CompactCandidate,
    verification: CompactVerification,
    engine: EngineIdentity,
    environment: CompactEnvironment,
    local_evidence: LocalEvidence,
    evidence_ceiling: String,
}

#[derive(Serialize)]
struct ReceiptDigestView<'a> {
    spdx: &'a str,
    schema_version: u8,
    protocol_version: u8,
    protocol_status: &'a str,
    status: &'a str,
    source_version: &'a str,
    audit_id: &'a str,
    candidate: &'a CompactCandidate,
    verification: &'a CompactVerification,
    engine: &'a EngineIdentity,
    environment: &'a CompactEnvironment,
    local_evidence: &'a LocalEvidence,
    evidence_ceiling: &'a str,
}

impl CompactReceipt {
    fn digest(&self) -> Result<Sha256, Error> {
        canonical_digest(&ReceiptDigestView {
            spdx: &self.spdx,
            schema_version: self.schema_version,
            protocol_version: self.protocol_version,
            protocol_status: &self.protocol_status,
            status: &self.status,
            source_version: &self.source_version,
            audit_id: &self.audit_id,
            candidate: &self.candidate,
            verification: &self.verification,
            engine: &self.engine,
            environment: &self.environment,
            local_evidence: &self.local_evidence,
            evidence_ceiling: &self.evidence_ceiling,
        })
    }

    pub(crate) fn receipt_id(&self) -> &str {
        self.receipt_id.as_str()
    }

    pub(crate) fn source_version(&self) -> &str {
        &self.source_version
    }

    pub(crate) fn audit_id(&self) -> &str {
        &self.audit_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpandedReceipt {
    schema_version: u8,
    protocol_version: u8,
    protocol_status: String,
    source_version: String,
    audit_id: String,
    candidate: ExpandedCandidate,
    verification: ExpandedVerification,
    engine: EngineIdentity,
    environment: ExpandedEnvironment,
    evidence_ceiling: String,
}

#[derive(Debug, Deserialize)]
struct SchemaProbe {
    schema_version: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
// Schema v1 predates the exact-key v2 contract. The frozen Python validator
// intentionally ignored extra legacy metadata (including `verified_at_utc`),
// so adding `deny_unknown_fields` here would break the one allowlisted tuple.
struct LegacyReceipt {
    #[allow(dead_code)]
    schema_version: Option<u8>,
    candidate_commit_sha: String,
    commands: Vec<String>,
    result: String,
    transcript_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValidatedReceipt {
    Legacy,
    V2(Box<CompactReceipt>),
}

impl ValidatedReceipt {
    pub(crate) fn schema_version(&self) -> u8 {
        match self {
            Self::Legacy => 1,
            Self::V2(_) => 2,
        }
    }

    pub(crate) fn v2(&self) -> Result<&CompactReceipt, Error> {
        match self {
            Self::V2(receipt) => Ok(receipt),
            Self::Legacy => Err(receipt_error(
                "administrative commit gates require receipt schema v2",
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidationOptions<'a> {
    pub(crate) require_local: bool,
    pub(crate) check_environment: bool,
    pub(crate) check_engine: bool,
    pub(crate) source_version: Option<&'a str>,
    pub(crate) audit_id: Option<&'a str>,
}

impl Default for ValidationOptions<'_> {
    fn default() -> Self {
        Self {
            require_local: true,
            check_environment: true,
            check_engine: true,
            source_version: None,
            audit_id: None,
        }
    }
}

fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    // `Value` is deliberately confined to this serialization boundary.  No
    // production semantic validation indexes or traverses dynamic JSON.
    let value = serde_json::to_value(value)?;
    Ok(canonical_json(&value))
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<Sha256, Error> {
    Ok(Sha256::of(canonical(value)?))
}

fn pretty<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    // Python's reference renderer sorts every object recursively.  Reparse the
    // typed serialization only at this deterministic rendering boundary.
    let mut value = serde_json::to_value(value)?;
    sort_json_keys(&mut value);
    let mut bytes = serde_json::to_vec_pretty(&value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sort_json_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => values.iter_mut().for_each(sort_json_keys),
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                sort_json_keys(value);
            }
            let old = std::mem::take(values);
            let sorted = old.into_iter().collect::<BTreeMap<_, _>>();
            values.extend(sorted);
        }
        _ => {}
    }
}

pub(crate) fn classify_path(path: &str) -> ManifestClass {
    if ADMINISTRATIVE_PATHS.contains(&path)
        || path.starts_with("new-book-plans/verification-receipts/")
    {
        ManifestClass::Administrative
    } else if GENERATED_PATHS.contains(&path) {
        ManifestClass::GeneratedArtifact
    } else if path == "Cargo.toml"
        || path == "Cargo.lock"
        || path.starts_with("src/")
        || path == "build.rs"
        || path == "verify.sh"
        || path.ends_with(".sh")
        || path == "registry/check.py"
        || (path.starts_with("new-book-plans/") && path.ends_with(".py"))
    {
        ManifestClass::VerifierInput
    } else if path.ends_with(".pins.nibli")
        || path.contains("/counterfactual/")
        || path.contains("/fixtures/")
        || path.contains("/testdata/")
    {
        ManifestClass::Fixture
    } else {
        ManifestClass::Source
    }
}

fn git(root: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<Output, Error> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| receipt_error(format!("cannot run git: {error}")))
}

fn git_checked(
    root: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<Vec<u8>, Error> {
    let output = git(root, args)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Err(receipt_error(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("git exited {}", output.status)
        }))
    }
}

fn git_text(
    root: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<String, Error> {
    let bytes = git_checked(root, args)?;
    String::from_utf8(bytes)
        .map(|value| value.trim().to_owned())
        .map_err(|error| receipt_error(format!("git output is not UTF-8: {error}")))
}

fn parse_manifest(raw: &[u8], index: bool) -> Result<Vec<ManifestEntry>, Error> {
    let mut entries = Vec::new();
    for record in raw
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| receipt_error("malformed Git path manifest"))?;
        let left = std::str::from_utf8(&record[..tab])
            .map_err(|_| receipt_error("Git path manifest metadata is not UTF-8"))?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|_| receipt_error("Git path manifest path is not UTF-8"))?
            .to_owned();
        let fields = left.split(' ').collect::<Vec<_>>();
        let (mode, kind, object) = if index {
            if fields.len() != 3 || fields[2] != "0" {
                return Err(receipt_error("index contains a non-zero merge stage"));
            }
            (
                fields[0],
                if fields[0] == "160000" {
                    GitObjectKind::Commit
                } else {
                    GitObjectKind::Blob
                },
                fields[1],
            )
        } else {
            if fields.len() != 3 {
                return Err(receipt_error("malformed Git tree manifest"));
            }
            let kind = match fields[1] {
                "blob" => GitObjectKind::Blob,
                "commit" => GitObjectKind::Commit,
                other => {
                    return Err(receipt_error(format!(
                        "unsupported recursive Git object type: {other}"
                    )));
                }
            };
            (fields[0], kind, fields[2])
        };
        entries.push(ManifestEntry {
            path,
            mode: mode.to_owned(),
            kind,
            object: GitSha::parse(object, "manifest object")?,
        });
    }
    entries.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    if entries.windows(2).any(|pair| pair[0].path == pair[1].path) {
        return Err(receipt_error("Git path manifest contains duplicate paths"));
    }
    Ok(entries)
}

fn index_manifest(root: &Path) -> Result<Vec<ManifestEntry>, Error> {
    parse_manifest(&git_checked(root, ["ls-files", "-s", "-z"])?, true)
}

fn tree_manifest(root: &Path, treeish: &str) -> Result<Vec<ManifestEntry>, Error> {
    parse_manifest(
        &git_checked(root, ["ls-tree", "-r", "-z", "--full-tree", treeish])?,
        false,
    )
}

fn manifest_map(manifest: &[ManifestEntry]) -> BTreeMap<&str, &ManifestEntry> {
    manifest
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect()
}

fn blob(root: &Path, object: &GitSha) -> Result<Vec<u8>, Error> {
    git_checked(root, ["cat-file", "blob", object.as_str()])
}

fn blob_at(root: &Path, manifest: &[ManifestEntry], path: &str) -> Result<Vec<u8>, Error> {
    let entry = manifest_map(manifest)
        .get(path)
        .copied()
        .filter(|entry| entry.kind == GitObjectKind::Blob)
        .ok_or_else(|| receipt_error(format!("required blob is absent: {path}")))?;
    blob(root, &entry.object)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StagedCandidate {
    parent: GitSha,
    tree: GitSha,
    manifest: Vec<ManifestEntry>,
}

fn fully_staged_candidate(root: &Path) -> Result<StagedCandidate, Error> {
    if !git_checked(root, ["ls-files", "-u", "-z"])?.is_empty() {
        return Err(receipt_error("the Git index contains unresolved stages"));
    }
    let unstaged = git(root, ["diff", "--quiet", "--"])?;
    match unstaged.status.code() {
        Some(0) => {}
        Some(1) => {
            return Err(receipt_error(
                "receipt emission requires no unstaged tracked changes",
            ));
        }
        _ => return Err(receipt_error("cannot inspect unstaged changes")),
    }
    let untracked = git_checked(root, ["ls-files", "--others", "--exclude-standard", "-z"])?;
    if !untracked.is_empty() {
        let names = untracked
            .split(|byte| *byte == 0)
            .filter(|value| !value.is_empty())
            .take(5)
            .map(|value| {
                let path = Path::new(OsStr::from_bytes(value));
                path.file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        return Err(receipt_error(format!(
            "receipt emission requires no non-ignored untracked files: {}",
            names.join(", ")
        )));
    }
    let staged = git(root, ["diff", "--cached", "--quiet", "HEAD", "--"])?;
    match staged.status.code() {
        Some(0) => {
            return Err(receipt_error(
                "receipt emission requires a staged candidate",
            ));
        }
        Some(1) => {}
        _ => return Err(receipt_error("cannot inspect staged changes")),
    }
    let check = git(root, ["diff", "--cached", "--check"])?;
    if !check.status.success() {
        return Err(receipt_error(format!(
            "staged candidate fails git diff --cached --check: {}",
            String::from_utf8_lossy(&check.stdout).trim()
        )));
    }
    Ok(StagedCandidate {
        parent: GitSha::parse(git_text(root, ["rev-parse", "HEAD"])?, "candidate parent")?,
        tree: GitSha::parse(git_text(root, ["write-tree"])?, "candidate tree")?,
        manifest: index_manifest(root)?,
    })
}

fn classified_manifest(
    manifest: &[ManifestEntry],
) -> Result<(Vec<ClassifiedManifestEntry>, ClassManifests), Error> {
    let expanded = manifest
        .iter()
        .map(|entry| ClassifiedManifestEntry {
            path: entry.path.clone(),
            mode: entry.mode.clone(),
            kind: entry.kind.clone(),
            object: entry.object.clone(),
            class: classify_path(&entry.path),
        })
        .collect::<Vec<_>>();
    let row = |class| -> Result<ClassManifest, Error> {
        let rows = expanded
            .iter()
            .filter(|entry| entry.class == class)
            .map(ClassifiedManifestEntry::raw)
            .collect::<Vec<_>>();
        Ok(ClassManifest {
            count: rows.len(),
            sha256: canonical_digest(&rows)?,
        })
    };
    let classes = ClassManifests {
        source: row(ManifestClass::Source)?,
        verifier_input: row(ManifestClass::VerifierInput)?,
        fixture: row(ManifestClass::Fixture)?,
        generated_artifact: row(ManifestClass::GeneratedArtifact)?,
        administrative: row(ManifestClass::Administrative)?,
    };
    Ok((expanded, classes))
}

fn git_common_directory(root: &Path) -> Result<PathBuf, Error> {
    let path = git_text(
        root,
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    fs::canonicalize(&path).map_err(|error| {
        receipt_error(format!(
            "cannot resolve Git common directory {path}: {error}"
        ))
    })
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct UntrackedSourceEntry {
    path: String,
    kind: String,
    mode: u32,
    sha256: Sha256,
}

#[derive(Clone, Debug)]
struct MappedExecutableIdentity {
    basename: String,
    path_sha256: Sha256,
    sha256: Sha256,
    size: usize,
}

fn mapped_executable_identity() -> Result<MappedExecutableIdentity, Error> {
    static IDENTITY: OnceLock<Result<MappedExecutableIdentity, String>> = OnceLock::new();
    IDENTITY
        .get_or_init(|| {
            let display = std::env::current_exe()
                .map_err(|error| format!("cannot locate rights-verify: {error}"))?;
            let body = fs::read("/proc/self/exe")
                .map_err(|error| format!("cannot read mapped rights-verify: {error}"))?;
            if body.is_empty() {
                return Err("mapped rights-verify executable is empty".to_owned());
            }
            let basename = display
                .file_name()
                .and_then(OsStr::to_str)
                .filter(|name| !name.is_empty())
                .ok_or_else(|| "rights-verify executable basename is invalid".to_owned())?
                .to_owned();
            Ok(MappedExecutableIdentity {
                basename,
                path_sha256: Sha256::of(display.as_os_str().as_bytes()),
                sha256: Sha256::of(&body),
                size: body.len(),
            })
        })
        .clone()
        .map_err(receipt_error)
}

fn bind_provenance(binding: &mut Vec<u8>, value: &[u8]) {
    binding.extend_from_slice(&(value.len() as u64).to_be_bytes());
    binding.extend_from_slice(value);
}

fn engine_identity() -> Result<EngineIdentity, Error> {
    let binary = mapped_executable_identity()?;

    // The embedded engine was compiled from this path dependency. A runtime
    // NIBLI_SRC override cannot alter those embedded bytes and must not be
    // represented as the binary's source provenance.
    let source_override = false;
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../nibli");
    let source = fs::canonicalize(&source).unwrap_or(source);
    let probe = git(&source, ["rev-parse", "HEAD"])?;
    let source_available = probe.status.success();
    let (
        source_commit_sha,
        source_dirty,
        source_status_sha256,
        source_diff_sha256,
        source_untracked_count,
        source_untracked_sha256,
    ) = if source_available {
        let commit = String::from_utf8(probe.stdout)
            .map_err(|_| receipt_error("Nibli source commit is not ASCII"))?
            .trim()
            .to_owned();
        let status = git_checked(
            &source,
            ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?;
        let diff = git_checked(&source, ["diff", "--binary", "HEAD", "--"])?;
        let untracked = git_checked(
            &source,
            ["ls-files", "--others", "--exclude-standard", "-z"],
        )?;
        let mut rows = Vec::new();
        for raw in untracked
            .split(|byte| *byte == 0)
            .filter(|raw| !raw.is_empty())
        {
            let relative = std::str::from_utf8(raw)
                .map_err(|_| receipt_error("untracked Nibli source path is not UTF-8"))?;
            let item = source.join(relative);
            let metadata = fs::symlink_metadata(&item)?;
            let (body, kind) = if metadata.file_type().is_symlink() {
                (
                    fs::read_link(&item)?.as_os_str().as_bytes().to_vec(),
                    "symlink",
                )
            } else if metadata.is_file() {
                (fs::read(&item)?, "file")
            } else {
                return Err(receipt_error("unsupported untracked Nibli source entry"));
            };
            rows.push((
                UntrackedSourceEntry {
                    path: relative.to_owned(),
                    kind: kind.to_owned(),
                    mode: metadata.permissions().mode() & 0o177777,
                    sha256: Sha256::of(&body),
                },
                body,
            ));
        }
        rows.sort_by(|left, right| left.0.path.as_bytes().cmp(right.0.path.as_bytes()));
        let typed_rows = rows.into_iter().map(|(row, _)| row).collect::<Vec<_>>();
        (
            RequiredNullable(Some(GitSha::parse(commit, "Nibli source commit")?)),
            RequiredNullable(Some(!status.is_empty())),
            RequiredNullable(Some(Sha256::of(&status))),
            RequiredNullable(Some(Sha256::of(&diff))),
            RequiredNullable(Some(typed_rows.len())),
            RequiredNullable(Some(canonical_digest(&typed_rows)?)),
        )
    } else {
        return Err(receipt_error(
            "native verifier requires its Git-bound Nibli source checkout",
        ));
    };
    Ok(EngineIdentity {
        binary_basename: binary.basename,
        binary_path_sha256: binary.path_sha256,
        binary_sha256: binary.sha256,
        binary_size: binary.size,
        source_override,
        source_available,
        source_commit_sha,
        source_dirty,
        source_status_sha256,
        source_diff_sha256,
        source_untracked_count,
        source_untracked_sha256,
    })
}

fn first_line(root: &Path, program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).current_dir(root).output() {
        Ok(output) => String::from_utf8_lossy(&[output.stdout, output.stderr].concat())
            .lines()
            .next()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("exit-{}", output.status.code().unwrap_or(-1))),
        Err(error) => format!("error-{error}"),
    }
}

fn uname(root: &Path, flag: &str) -> String {
    first_line(root, "uname", &[flag])
}

fn sanitized_environment(root: &Path) -> Result<ExpandedEnvironment, Error> {
    let mut allowlisted_values = BTreeMap::new();
    for key in [
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TZ",
        "PYTHONHASHSEED",
        "SOURCE_DATE_EPOCH",
        "STATE_FORM_MAX_PARALLEL",
        "RIGHTS_VERIFY_JOBS",
        "TEMPORAL_ASSURANCE_JOBS",
        "AMENDMENT_AUDIT_JOBS",
        "RED_TEAM_JOBS",
    ] {
        if let Ok(value) = std::env::var(key) {
            allowlisted_values.insert(key.to_owned(), value);
        }
    }
    let mut hashed_values = BTreeMap::new();
    for key in [
        "PATH",
        "NIBLI_PIN",
        "NIBLI_SRC",
        "NIBLI_STRATA_FILE",
        "NIBLI_STRATA_CACHE_OUT",
    ] {
        if let Some(value) = std::env::var_os(key) {
            hashed_values.insert(key.to_owned(), Sha256::of(value.as_bytes()));
        }
    }
    let details = EnvironmentDetails {
        allowlisted_values,
        hashed_values,
        platform: PlatformIdentity {
            system: uname(root, "-s"),
            release: uname(root, "-r"),
            machine: uname(root, "-m"),
        },
        tools: BTreeMap::from([
            ("bash".to_owned(), first_line(root, "bash", &["--version"])),
            ("git".to_owned(), first_line(root, "git", &["--version"])),
            (
                "rights-verify".to_owned(),
                format!("rights-verify {}", env!("CARGO_PKG_VERSION")),
            ),
        ]),
    };
    Ok(ExpandedEnvironment {
        sha256: canonical_digest(&details)?,
        details,
    })
}

struct DuplicateKeySeed;

impl<'de> DeserializeSeed<'de> for DuplicateKeySeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateKeyVisitor)
    }
}

struct DuplicateKeyVisitor;

impl<'de> Visitor<'de> for DuplicateKeyVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("valid JSON without duplicate object keys")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
            }
            map.next_value_seed(DuplicateKeySeed)?;
        }
        Ok(())
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element_seed(DuplicateKeySeed)?.is_some() {}
        Ok(())
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }
}

fn reject_duplicate_keys(bytes: &[u8], context: &str) -> Result<(), Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    DuplicateKeySeed
        .deserialize(&mut deserializer)
        .map_err(|error| receipt_error(format!("{context}: {error}")))?;
    deserializer
        .end()
        .map_err(|error| receipt_error(format!("{context}: {error}")))
}

fn parse_typed<T: for<'de> Deserialize<'de>>(bytes: &[u8], context: &str) -> Result<T, Error> {
    reject_duplicate_keys(bytes, context)?;
    serde_json::from_slice(bytes).map_err(|error| receipt_error(format!("{context}: {error}")))
}

fn validate_compact(receipt: &CompactReceipt, path: &Path) -> Result<(), Error> {
    validate_compact_protocol(
        receipt,
        path,
        PROTOCOL_VERSION,
        PROTOCOL_STATUS,
        "current protocol v6",
    )
}

fn validate_compact_protocol(
    receipt: &CompactReceipt,
    path: &Path,
    protocol_version: u8,
    protocol_status: &str,
    protocol_label: &str,
) -> Result<(), Error> {
    if receipt.spdx != "CC0-1.0" || receipt.schema_version != 2 {
        return Err(receipt_error(
            "receipt licence/schema must be CC0-1.0 / version 2",
        ));
    }
    if receipt.protocol_version != protocol_version || receipt.protocol_status != protocol_status {
        return Err(receipt_error(format!(
            "receipt does not bind {protocol_label}"
        )));
    }
    if receipt.status != "all-passed" {
        return Err(receipt_error("receipt is not passing"));
    }
    if receipt.digest()? != receipt.receipt_id {
        return Err(receipt_error(
            "receipt self digest does not match receipt_id",
        ));
    }
    let expected_name = format!("sha256-{}.json", receipt.receipt_id.as_str());
    if path.file_name().and_then(OsStr::to_str) != Some(expected_name.as_str()) {
        return Err(receipt_error(
            "receipt filename does not match its self digest",
        ));
    }
    if receipt.source_version.is_empty() {
        return Err(receipt_error(
            "receipt source_version must be a nonempty string",
        ));
    }
    if !canonical_audit_id(&receipt.audit_id) {
        return Err(receipt_error(
            "receipt audit_id must be a canonical FS-SAU id",
        ));
    }
    if receipt.evidence_ceiling != EVIDENCE_CEILING {
        return Err(receipt_error("receipt evidence ceiling is not byte-exact"));
    }
    if receipt.candidate.path_count == 0 {
        return Err(receipt_error("candidate path count is invalid"));
    }
    if receipt.candidate.class_manifests.total() != receipt.candidate.path_count {
        return Err(receipt_error(
            "candidate class counts do not cover the path manifest",
        ));
    }
    let expected_command_digest = canonical_digest(&[FULL_COMMAND])?;
    if receipt.verification.command_sha256 != expected_command_digest {
        return Err(receipt_error(
            "receipt command digest is not the full verifier",
        ));
    }
    if receipt.verification.results
        != [VerificationResult {
            command: FULL_COMMAND.to_owned(),
            exit_code: 0,
        }]
    {
        return Err(receipt_error(
            "receipt does not bind one passing full verifier",
        ));
    }
    if receipt.verification.transcript_sha256 != receipt.local_evidence.transcript_sha256 {
        return Err(receipt_error("receipt transcript bindings disagree"));
    }
    if receipt.verification.finished_at_utc < receipt.verification.started_at_utc {
        return Err(receipt_error("verification finish predates its start"));
    }
    if receipt.environment.fields.iter().any(String::is_empty)
        || !strictly_sorted_unique(&receipt.environment.fields)
    {
        return Err(receipt_error(
            "receipt environment fields must be sorted and unique",
        ));
    }
    if receipt.engine.binary_basename.is_empty()
        || receipt.engine.binary_size == 0
        || nullable_engine_fields_consistent(&receipt.engine).is_err()
    {
        return Err(receipt_error("receipt engine identity is invalid"));
    }
    Ok(())
}

fn canonical_audit_id(value: &str) -> bool {
    value.strip_prefix("FS-SAU-").is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn nullable_engine_fields_consistent(engine: &EngineIdentity) -> Result<(), ()> {
    let present = [
        engine.source_commit_sha.0.is_some(),
        engine.source_dirty.0.is_some(),
        engine.source_status_sha256.0.is_some(),
        engine.source_diff_sha256.0.is_some(),
        engine.source_untracked_count.0.is_some(),
        engine.source_untracked_sha256.0.is_some(),
    ];
    if engine.source_available {
        present
            .into_iter()
            .all(|value| value)
            .then_some(())
            .ok_or(())
    } else {
        present
            .into_iter()
            .all(|value| !value)
            .then_some(())
            .ok_or(())
    }
}

fn validate_legacy(
    bytes: &[u8],
    source_version: Option<&str>,
    audit_id: Option<&str>,
) -> Result<ValidatedReceipt, Error> {
    if (source_version, audit_id) != (Some(LEGACY_SOURCE_VERSION), Some(LEGACY_AUDIT_ID)) {
        return Err(receipt_error(
            "legacy receipt is outside the one exact v1 allowlist",
        ));
    }
    let receipt: LegacyReceipt = parse_typed(bytes, "cannot read legacy receipt")?;
    if receipt.candidate_commit_sha != LEGACY_CANDIDATE {
        return Err(receipt_error("legacy candidate is not allowlisted"));
    }
    if receipt.transcript_sha256 != LEGACY_TRANSCRIPT_SHA256 {
        return Err(receipt_error("legacy transcript digest is not allowlisted"));
    }
    if receipt.result != "all-passed" {
        return Err(receipt_error("legacy receipt did not pass"));
    }
    if receipt.commands != LEGACY_REQUIRED_COMMANDS {
        return Err(receipt_error(
            "legacy receipt command list is not the preserved full run",
        ));
    }
    Ok(ValidatedReceipt::Legacy)
}

pub(crate) fn load_and_validate(
    context: &Context,
    path: &Path,
    options: ValidationOptions<'_>,
) -> Result<ValidatedReceipt, Error> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        context.path(path)
    };
    let bytes = fs::read(&absolute).map_err(|error| {
        receipt_error(format!(
            "cannot read receipt {}: {error}",
            absolute.display()
        ))
    })?;
    validate_receipt_bytes(context, &absolute, &bytes, options)
}

pub(crate) fn validate_receipt_bytes(
    context: &Context,
    nominal_path: &Path,
    bytes: &[u8],
    options: ValidationOptions<'_>,
) -> Result<ValidatedReceipt, Error> {
    let check_engine = options.check_engine;
    let runtime = (options.check_environment || options.check_engine)
        .then(|| runtime_bindings(context.root()))
        .transpose()?;
    let validated = validate_receipt_bytes_with_runtime(
        context.root(),
        nominal_path,
        bytes,
        options,
        runtime.as_ref().map(|runtime| &runtime.environment),
        runtime.as_ref().map(|runtime| &runtime.engine),
    )?;
    if check_engine && let (Some(runtime), Ok(receipt)) = (runtime.as_ref(), validated.v2()) {
        let manifest = tree_manifest(context.root(), receipt.candidate.tree_sha.as_str())?;
        verify_compiled_candidate(context.root(), &manifest, runtime)?;
    }
    Ok(validated)
}

fn validate_receipt_bytes_with_runtime(
    root: &Path,
    nominal_path: &Path,
    bytes: &[u8],
    options: ValidationOptions<'_>,
    environment: Option<&ExpandedEnvironment>,
    engine: Option<&EngineIdentity>,
) -> Result<ValidatedReceipt, Error> {
    reject_duplicate_keys(bytes, "cannot read receipt")?;
    let probe: SchemaProbe = serde_json::from_slice(bytes).map_err(|error| {
        receipt_error(format!(
            "cannot read receipt {}: {error}",
            nominal_path.display()
        ))
    })?;
    match probe.schema_version {
        None | Some(1) => validate_legacy(bytes, options.source_version, options.audit_id),
        Some(2) => {
            let receipt: CompactReceipt = serde_json::from_slice(bytes).map_err(|error| {
                receipt_error(format!(
                    "cannot read receipt {}: {error}",
                    nominal_path.display()
                ))
            })?;
            validate_compact(&receipt, nominal_path)?;
            if receipt.source_version == LEGACY_SOURCE_VERSION {
                return Err(receipt_error(
                    "the legacy source may not be relabelled as receipt v2",
                ));
            }
            if options.require_local {
                validate_local_evidence(root, &receipt)?;
            }
            if options.check_environment
                && environment
                    .ok_or_else(|| receipt_error("environment probe is unavailable"))?
                    .sha256
                    != receipt.environment.sha256
            {
                return Err(receipt_error("sanitized environment drifted from receipt"));
            }
            if options.check_engine
                && engine.ok_or_else(|| receipt_error("engine probe is unavailable"))?
                    != &receipt.engine
            {
                return Err(receipt_error(
                    "rights-verify binary or Nibli source identity drifted from receipt",
                ));
            }
            Ok(ValidatedReceipt::V2(Box::new(receipt)))
        }
        Some(_) => Err(receipt_error(
            "unknown receipt schema; downgrade is forbidden",
        )),
    }
}

fn load_and_validate_with_runtime(
    root: &Path,
    path: &Path,
    options: ValidationOptions<'_>,
    runtime: &RuntimeBindings,
) -> Result<ValidatedReceipt, Error> {
    let check_engine = options.check_engine;
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let bytes = fs::read(&absolute).map_err(|error| {
        receipt_error(format!(
            "cannot read receipt {}: {error}",
            absolute.display()
        ))
    })?;
    let validated = validate_receipt_bytes_with_runtime(
        root,
        &absolute,
        &bytes,
        options,
        Some(&runtime.environment),
        Some(&runtime.engine),
    )?;
    if check_engine && let Ok(receipt) = validated.v2() {
        let manifest = tree_manifest(root, receipt.candidate.tree_sha.as_str())?;
        verify_compiled_candidate(root, &manifest, runtime)?;
    }
    Ok(validated)
}

fn validate_local_evidence(root: &Path, receipt: &CompactReceipt) -> Result<(), Error> {
    let evidence = git_common_directory(root)?
        .join(EVIDENCE_SUBDIRECTORY)
        .join(format!("sha256-{}", receipt.receipt_id.as_str()));
    let expanded_path = evidence.join("expanded-manifest.json");
    let transcript_path = evidence.join("transcript.log");
    let results_path = evidence.join("command-results.json");
    if !expanded_path.is_file() || !transcript_path.is_file() || !results_path.is_file() {
        return Err(receipt_error("local receipt evidence is missing"));
    }
    let expanded_bytes = fs::read(&expanded_path)?;
    let transcript = fs::read(&transcript_path)?;
    let results_bytes = fs::read(&results_path)?;
    if Sha256::of(&expanded_bytes) != receipt.local_evidence.expanded_manifest_sha256 {
        return Err(receipt_error("expanded manifest digest mismatch"));
    }
    if Sha256::of(&transcript) != receipt.verification.transcript_sha256 {
        return Err(receipt_error("transcript digest mismatch"));
    }
    let expanded: ExpandedReceipt = parse_typed(&expanded_bytes, "expanded manifest is invalid")?;
    if expanded.schema_version != 2
        || expanded.protocol_version != receipt.protocol_version
        || expanded.protocol_status != receipt.protocol_status
        || expanded.source_version != receipt.source_version
        || expanded.audit_id != receipt.audit_id
        || expanded.evidence_ceiling != receipt.evidence_ceiling
    {
        return Err(receipt_error(
            "expanded and compact protocol bindings disagree",
        ));
    }
    if pretty(&expanded.verification)? != results_bytes {
        return Err(receipt_error(
            "command-results evidence is not the exact manifest record",
        ));
    }
    if expanded.verification.result != "all-passed"
        || expanded.verification.commands.len() != 1
        || expanded.verification.commands[0].argv_sha256 != receipt.verification.command_sha256
    {
        return Err(receipt_error(
            "expanded and compact command bindings disagree",
        ));
    }
    let raw_manifest = expanded
        .candidate
        .path_manifest
        .iter()
        .map(ClassifiedManifestEntry::raw)
        .collect::<Vec<_>>();
    if canonical_digest(&raw_manifest)? != receipt.candidate.path_manifest_sha256
        || raw_manifest.len() != receipt.candidate.path_count
    {
        return Err(receipt_error(
            "expanded and compact path manifests disagree",
        ));
    }
    let (classified, classes) = classified_manifest(&raw_manifest)?;
    if expanded.candidate.path_manifest != classified {
        return Err(receipt_error(
            "expanded path classifications are not deterministic",
        ));
    }
    if expanded.candidate.parent_commit_sha != receipt.candidate.parent_commit_sha
        || expanded.candidate.tree_sha != receipt.candidate.tree_sha
        || expanded.candidate.path_manifest_sha256 != receipt.candidate.path_manifest_sha256
        || expanded.candidate.class_manifests != classes
        || receipt.candidate.class_manifests != classes
    {
        return Err(receipt_error(
            "expanded and compact candidate bindings disagree",
        ));
    }
    if tree_manifest(root, receipt.candidate.tree_sha.as_str())? != raw_manifest {
        return Err(receipt_error(
            "candidate Git tree no longer matches path manifest",
        ));
    }
    if expanded.engine != receipt.engine {
        return Err(receipt_error(
            "expanded and compact engine identities disagree",
        ));
    }
    if expanded.environment.sha256 != receipt.environment.sha256 {
        return Err(receipt_error(
            "expanded and compact environment bindings disagree",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct JsonSpan {
    start: usize,
    end: usize,
}

fn skip_ws(bytes: &[u8], position: &mut usize) {
    while bytes
        .get(*position)
        .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
    {
        *position += 1;
    }
}

fn scan_string(bytes: &[u8], position: &mut usize) -> Result<JsonSpan, Error> {
    skip_ws(bytes, position);
    let start = *position;
    if bytes.get(*position) != Some(&b'"') {
        return Err(receipt_error("JSON string expected"));
    }
    *position += 1;
    let mut escaped = false;
    while let Some(byte) = bytes.get(*position).copied() {
        *position += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Ok(JsonSpan {
                start,
                end: *position,
            });
        }
    }
    Err(receipt_error("unterminated JSON string"))
}

fn scan_value(bytes: &[u8], position: &mut usize) -> Result<JsonSpan, Error> {
    skip_ws(bytes, position);
    let start = *position;
    match bytes.get(*position).copied() {
        Some(b'"') => {
            scan_string(bytes, position)?;
        }
        Some(b'{') => {
            *position += 1;
            skip_ws(bytes, position);
            if bytes.get(*position) == Some(&b'}') {
                *position += 1;
            } else {
                loop {
                    scan_string(bytes, position)?;
                    skip_ws(bytes, position);
                    if bytes.get(*position) != Some(&b':') {
                        return Err(receipt_error("JSON object colon expected"));
                    }
                    *position += 1;
                    scan_value(bytes, position)?;
                    skip_ws(bytes, position);
                    match bytes.get(*position) {
                        Some(b',') => *position += 1,
                        Some(b'}') => {
                            *position += 1;
                            break;
                        }
                        _ => return Err(receipt_error("JSON object delimiter expected")),
                    }
                }
            }
        }
        Some(b'[') => {
            *position += 1;
            skip_ws(bytes, position);
            if bytes.get(*position) == Some(&b']') {
                *position += 1;
            } else {
                loop {
                    scan_value(bytes, position)?;
                    skip_ws(bytes, position);
                    match bytes.get(*position) {
                        Some(b',') => *position += 1,
                        Some(b']') => {
                            *position += 1;
                            break;
                        }
                        _ => return Err(receipt_error("JSON array delimiter expected")),
                    }
                }
            }
        }
        Some(_) => {
            while bytes.get(*position).is_some_and(|byte| {
                !matches!(byte, b' ' | b'\n' | b'\r' | b'\t' | b',' | b']' | b'}')
            }) {
                *position += 1;
            }
            if *position == start {
                return Err(receipt_error("JSON value expected"));
            }
        }
        None => return Err(receipt_error("JSON value expected")),
    }
    Ok(JsonSpan {
        start,
        end: *position,
    })
}

fn object_fields(bytes: &[u8]) -> Result<BTreeMap<String, JsonSpan>, Error> {
    reject_duplicate_keys(bytes, "JSON object is invalid")?;
    let mut position = 0;
    skip_ws(bytes, &mut position);
    if bytes.get(position) != Some(&b'{') {
        return Err(receipt_error("JSON object expected"));
    }
    position += 1;
    let mut fields = BTreeMap::new();
    skip_ws(bytes, &mut position);
    if bytes.get(position) == Some(&b'}') {
        position += 1;
    } else {
        loop {
            let key_span = scan_string(bytes, &mut position)?;
            let key: String = serde_json::from_slice(&bytes[key_span.start..key_span.end])?;
            skip_ws(bytes, &mut position);
            if bytes.get(position) != Some(&b':') {
                return Err(receipt_error("JSON object colon expected"));
            }
            position += 1;
            let value = scan_value(bytes, &mut position)?;
            fields.insert(key, value);
            skip_ws(bytes, &mut position);
            match bytes.get(position) {
                Some(b',') => position += 1,
                Some(b'}') => {
                    position += 1;
                    break;
                }
                _ => return Err(receipt_error("JSON object delimiter expected")),
            }
        }
    }
    skip_ws(bytes, &mut position);
    if position != bytes.len() {
        return Err(receipt_error("trailing bytes after JSON object"));
    }
    Ok(fields)
}

fn array_elements(bytes: &[u8]) -> Result<Vec<JsonSpan>, Error> {
    let mut position = 0;
    skip_ws(bytes, &mut position);
    if bytes.get(position) != Some(&b'[') {
        return Err(receipt_error("JSON array expected"));
    }
    position += 1;
    let mut elements = Vec::new();
    skip_ws(bytes, &mut position);
    if bytes.get(position) == Some(&b']') {
        position += 1;
    } else {
        loop {
            elements.push(scan_value(bytes, &mut position)?);
            skip_ws(bytes, &mut position);
            match bytes.get(position) {
                Some(b',') => position += 1,
                Some(b']') => {
                    position += 1;
                    break;
                }
                _ => return Err(receipt_error("JSON array delimiter expected")),
            }
        }
    }
    skip_ws(bytes, &mut position);
    if position != bytes.len() {
        return Err(receipt_error("trailing bytes after JSON array"));
    }
    Ok(elements)
}

fn field_bytes<'a>(
    bytes: &'a [u8],
    fields: &BTreeMap<String, JsonSpan>,
    name: &str,
) -> Result<&'a [u8], Error> {
    let span = fields
        .get(name)
        .ok_or_else(|| receipt_error(format!("required JSON field is absent: {name}")))?;
    Ok(&bytes[span.start..span.end])
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingAudit {
    id: String,
    title: String,
    source_version: String,
    scope_sha256: Sha256,
    protocol_sha256: Sha256,
    executed_at_utc: UtcTimestamp,
    method: String,
    criterion_coverage: Vec<String>,
    control_refs: Vec<String>,
    commands: Vec<String>,
    finding_refs: Vec<String>,
    result: String,
    policy_basis: String,
    evidence_ceiling: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PassingAudit {
    id: String,
    title: String,
    source_version: String,
    scope_sha256: Sha256,
    protocol_sha256: Sha256,
    executed_at_utc: UtcTimestamp,
    method: String,
    criterion_coverage: Vec<String>,
    control_refs: Vec<String>,
    commands: Vec<String>,
    finding_refs: Vec<String>,
    result: String,
    policy_basis: String,
    evidence_ceiling: String,
    verification_receipt_ref: String,
}

fn pending_audit_id(value: &str) -> Option<&str> {
    let id = value.strip_suffix("-PENDING")?;
    canonical_audit_id(id).then_some(id)
}

fn ledger_receipt_context(
    root: &Path,
    manifest: &[ManifestEntry],
) -> Result<(String, String), Error> {
    let bytes = blob_at(root, manifest, LEDGER_PATH)?;
    let projection = ledger_closure_projection(&bytes)?;
    let fields = object_fields(&bytes)
        .map_err(|_| receipt_error("staged full-society ledger is not valid UTF-8 JSON"))?;
    let source_version = projection.source_version;
    if source_version.is_empty() {
        return Err(receipt_error(
            "staged full-society ledger has no source_version",
        ));
    }
    let audits = field_bytes(&bytes, &fields, "scope_audits")?;
    let rows = array_elements(audits)?;
    let last = rows
        .last()
        .ok_or_else(|| receipt_error("staged ledger has no pending scope audit"))?;
    let pending: PendingAudit = parse_typed(
        &audits[last.start..last.end],
        "staged ledger pending audit is malformed",
    )?;
    let audit_id = pending_audit_id(&pending.id)
        .ok_or_else(|| {
            receipt_error("staged ledger must end in one current-source pending FS-SAU audit")
        })?
        .to_owned();
    if pending.result != "pending" || pending.source_version != source_version {
        return Err(receipt_error(
            "staged ledger must end in one current-source pending FS-SAU audit",
        ));
    }
    Ok((source_version, audit_id))
}

fn check_protocol(root: &Path, manifest: &[ManifestEntry]) -> Result<(), Error> {
    let bytes = blob_at(root, manifest, PROTOCOL_PATH)?;
    let body = std::str::from_utf8(&bytes)
        .map_err(|_| receipt_error("scope-review protocol is not UTF-8"))?;
    if body.contains(PROTOCOL_STATUS) {
        Ok(())
    } else {
        Err(receipt_error(
            "staged candidate does not publish protocol-v6 status",
        ))
    }
}

fn safe_output_directory(root: &Path, path: &Path) -> Result<PathBuf, Error> {
    let candidate = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let expected = root.join(RECEIPT_DIRECTORY);
    if candidate != expected {
        return Err(receipt_error(format!(
            "receipt output must be {RECEIPT_DIRECTORY}"
        )));
    }
    if let Ok(metadata) = fs::symlink_metadata(&candidate)
        && metadata.file_type().is_symlink()
    {
        return Err(receipt_error(
            "receipt output directory may not be a symbolic link",
        ));
    }
    let parent = fs::canonicalize(
        candidate
            .parent()
            .ok_or_else(|| receipt_error("receipt output has no parent"))?,
    )?;
    let canonical_root = fs::canonicalize(root)?;
    if !parent.starts_with(&canonical_root) {
        return Err(receipt_error("receipt output escapes the repository"));
    }
    Ok(candidate)
}

fn random_suffix() -> Result<String, Error> {
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random)
        .map_err(|error| receipt_error(format!("cannot create temporary name: {error}")))?;
    Ok(random.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn atomic_write(path: &Path, body: &[u8], mode: u32) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| receipt_error("atomic output has no parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("receipt"),
        std::process::id(),
        random_suffix()?
    ));
    let result = (|| {
        let mut handle = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&temporary)?;
        handle.write_all(body)?;
        handle.sync_all()?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(mode))?;
        fs::rename(&temporary, path)?;
        Ok::<(), Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn engine_stable(before: &EngineIdentity, after: &EngineIdentity) -> bool {
    // Unlike the former external nibli-pin, the running executable cannot be
    // rebuilt beneath an emission and still describe the code that just ran.
    // Both the embedded binary and its Nibli provenance must remain identical.
    before == after
}

fn authoritative_binary_name(name: &str) -> bool {
    name == "rights-verify" || (cfg!(test) && name.starts_with("rights_verify-"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeBindings {
    engine: EngineIdentity,
    environment: ExpandedEnvironment,
    compiled_verifier_inputs_sha256: Sha256,
}

fn runtime_bindings(root: &Path) -> Result<RuntimeBindings, Error> {
    let engine = engine_identity()?;
    let compiled_commit = env!("RIGHTS_VERIFY_COMPILED_NIBLI_COMMIT");
    if engine.source_commit_sha.0.as_ref().map(GitSha::as_str) != Some(compiled_commit) {
        return Err(receipt_error(
            "Nibli source revision differs from the revision embedded at compile time; rebuild rights-verify",
        ));
    }
    let compiled_nibli_inputs =
        Sha256(env!("RIGHTS_VERIFY_COMPILED_NIBLI_INPUTS_SHA256").to_owned());
    if nibli_dependency_input_digest()? != compiled_nibli_inputs {
        return Err(receipt_error(
            "Nibli dependency inputs differ from the inputs embedded at compile time; rebuild rights-verify",
        ));
    }
    Ok(RuntimeBindings {
        engine,
        environment: sanitized_environment(root)?,
        compiled_verifier_inputs_sha256: Sha256(
            env!("RIGHTS_VERIFY_COMPILED_INPUTS_SHA256").to_owned(),
        ),
    })
}

fn nibli_dependency_input_digest() -> Result<Sha256, Error> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../nibli");
    let source = fs::canonicalize(&source).map_err(|error| {
        receipt_error(format!(
            "cannot resolve Nibli dependency source {}: {error}",
            source.display()
        ))
    })?;
    let mut paths = Vec::new();
    for name in ["Cargo.toml", "Cargo.lock"] {
        add_dependency_input_if_present(&source, Path::new(name), &mut paths)?;
    }
    for crate_name in NIBLI_DEPENDENCY_CRATES {
        let crate_root = Path::new(crate_name);
        add_dependency_input_if_present(&source, &crate_root.join("Cargo.toml"), &mut paths)?;
        add_dependency_input_if_present(&source, &crate_root.join("build.rs"), &mut paths)?;
        let source_directory = crate_root.join("src");
        if source.join(&source_directory).is_dir() {
            collect_dependency_inputs(&source, &source_directory, &mut paths)?;
        }
    }
    paths.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });

    let mut binding = Vec::new();
    for relative in paths {
        let path = source.join(&relative);
        let metadata = fs::symlink_metadata(&path)?;
        let body = if metadata.file_type().is_symlink() {
            fs::read_link(&path)?.as_os_str().as_bytes().to_vec()
        } else {
            fs::read(&path)?
        };
        bind_provenance(&mut binding, relative.as_os_str().as_bytes());
        bind_provenance(&mut binding, &body);
    }
    Ok(Sha256::of(binding))
}

fn add_dependency_input_if_present(
    root: &Path,
    relative: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    match fs::symlink_metadata(root.join(relative)) {
        Ok(metadata) if metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            output.push(relative.to_path_buf());
            Ok(())
        }
        Ok(_) => Err(receipt_error(format!(
            "unsupported Nibli dependency input: {}",
            relative.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn collect_dependency_inputs(
    root: &Path,
    relative_directory: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    let directory = root.join(relative_directory);
    let mut entries = fs::read_dir(&directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| {
        left.as_os_str()
            .as_bytes()
            .cmp(right.as_os_str().as_bytes())
    });
    for path in entries {
        let metadata = fs::symlink_metadata(&path)?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| receipt_error("Nibli dependency input escaped its source root"))?
            .to_path_buf();
        if metadata.file_type().is_dir() {
            collect_dependency_inputs(root, &relative, output)?;
        } else if metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            output.push(relative);
        } else {
            return Err(receipt_error(format!(
                "unsupported Nibli dependency input: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn verifier_build_input(path: &str) -> bool {
    path == "Cargo.toml" || path == "Cargo.lock" || path == "build.rs" || path.starts_with("src/")
}

fn verifier_build_input_digest(root: &Path, manifest: &[ManifestEntry]) -> Result<Sha256, Error> {
    let mut binding = Vec::new();
    for entry in manifest
        .iter()
        .filter(|entry| verifier_build_input(&entry.path))
    {
        if entry.kind != GitObjectKind::Blob {
            return Err(receipt_error(format!(
                "verifier build input is not a blob: {}",
                entry.path
            )));
        }
        bind_provenance(&mut binding, entry.path.as_bytes());
        bind_provenance(&mut binding, &blob(root, &entry.object)?);
    }
    Ok(Sha256::of(binding))
}

fn verify_compiled_candidate(
    root: &Path,
    manifest: &[ManifestEntry],
    runtime: &RuntimeBindings,
) -> Result<(), Error> {
    if verifier_build_input_digest(root, manifest)? != runtime.compiled_verifier_inputs_sha256 {
        return Err(receipt_error(
            "staged Cargo/src verifier inputs differ from the inputs embedded at compile time; rebuild rights-verify",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EmittedReceipt {
    pub(crate) path: PathBuf,
    pub(crate) receipt_id: String,
}

/// Run the native full verifier once and emit a schema-v2 receipt.
///
/// The caller owns the heavyweight kernel lock. `run_full` must invoke the
/// in-process full suite; this function tees its exact bytes to `output` and
/// the locally retained transcript without launching `verify.sh` recursively.
pub(crate) fn emit_receipt<W, F>(
    context: &Context,
    output_directory: &Path,
    output: &mut W,
    run_full: F,
) -> Result<EmittedReceipt, Error>
where
    W: Write,
    F: FnOnce(&mut dyn Write) -> Result<(), Error>,
{
    emit_receipt_with_runtime(
        context.root(),
        output_directory,
        output,
        run_full,
        runtime_bindings,
    )
}

fn emit_receipt_with_runtime<W, F, P>(
    root: &Path,
    output_directory: &Path,
    output: &mut W,
    run_full: F,
    mut probe_runtime: P,
) -> Result<EmittedReceipt, Error>
where
    W: Write,
    F: FnOnce(&mut dyn Write) -> Result<(), Error>,
    P: FnMut(&Path) -> Result<RuntimeBindings, Error>,
{
    let output_directory = safe_output_directory(root, output_directory)?;
    let before = fully_staged_candidate(root)?;
    let (expanded_manifest, class_manifests) = classified_manifest(&before.manifest)?;
    let manifest_sha = canonical_digest(&before.manifest)?;
    let (source_version, audit_id) = ledger_receipt_context(root, &before.manifest)?;
    if source_version == LEGACY_SOURCE_VERSION {
        return Err(receipt_error(
            "the allowlisted legacy source may not be relabelled as receipt v2",
        ));
    }
    check_protocol(root, &before.manifest)?;
    let initial_runtime = probe_runtime(root)?;
    verify_compiled_candidate(root, &before.manifest, &initial_runtime)?;
    let environment = initial_runtime.environment;
    let initial_engine = initial_runtime.engine;
    if !initial_engine.source_available || initial_engine.source_commit_sha.0.is_none() {
        return Err(receipt_error(
            "native receipt emission requires an available Git-bound Nibli source checkout",
        ));
    }
    if !authoritative_binary_name(&initial_engine.binary_basename) {
        return Err(receipt_error(
            "authoritative receipt engine must be the rights-verify executable",
        ));
    }

    let started = UtcTimestamp::now()?;
    let monotonic = Instant::now();
    let mut transcript = Vec::new();
    let run_result = {
        let mut tee = TeeWriter::new(&mut *output, &mut transcript);
        let result = run_full(&mut tee);
        let flush = tee.flush().map_err(Error::from);
        result.and(flush)
    };
    let elapsed_milliseconds = monotonic.elapsed().as_millis();
    let finished = UtcTimestamp::now()?;
    if let Err(run_error) = run_result {
        let failed = git_common_directory(root)?
            .join(EVIDENCE_SUBDIRECTORY)
            .join(format!(
                "failed-{}-{}",
                before.tree.as_str(),
                random_suffix()?
            ));
        fs::create_dir_all(&failed)?;
        fs::set_permissions(&failed, fs::Permissions::from_mode(0o700))?;
        atomic_write(&failed.join("transcript.log"), &transcript, 0o600)?;
        return Err(receipt_error(format!(
            "authoritative verification failed; diagnostic transcript retained under the Git common directory: {run_error}"
        )));
    }
    let after = fully_staged_candidate(root)?;
    if after != before {
        return Err(receipt_error(
            "repository inputs drifted during verification",
        ));
    }
    let final_runtime = probe_runtime(root)?;
    verify_compiled_candidate(root, &after.manifest, &final_runtime)?;
    let engine = final_runtime.engine;
    if !engine_stable(&initial_engine, &engine) {
        return Err(receipt_error(
            "rights-verify binary or Nibli source drifted during verification",
        ));
    }
    let final_environment = final_runtime.environment;
    if final_environment != environment {
        return Err(receipt_error("sanitized verification environment drifted"));
    }
    let transcript_sha = Sha256::of(&transcript);
    let command_digest = canonical_digest(&[FULL_COMMAND])?;
    let command = CommandRecord {
        display: FULL_COMMAND.to_owned(),
        argv_sha256: command_digest.clone(),
        started_at_utc: started.clone(),
        finished_at_utc: finished.clone(),
        elapsed_milliseconds,
        exit_code: 0,
    };
    let expanded = ExpandedReceipt {
        schema_version: 2,
        protocol_version: PROTOCOL_VERSION,
        protocol_status: PROTOCOL_STATUS.to_owned(),
        source_version: source_version.clone(),
        audit_id: audit_id.clone(),
        candidate: ExpandedCandidate {
            parent_commit_sha: before.parent.clone(),
            tree_sha: before.tree.clone(),
            path_manifest_sha256: manifest_sha.clone(),
            path_manifest: expanded_manifest,
            class_manifests: class_manifests.clone(),
        },
        verification: ExpandedVerification {
            commands: vec![command.clone()],
            result: "all-passed".to_owned(),
            transcript_sha256: transcript_sha.clone(),
        },
        engine: engine.clone(),
        environment: environment.clone(),
        evidence_ceiling: EVIDENCE_CEILING.to_owned(),
    };
    let expanded_bytes = pretty(&expanded)?;
    let mut fields = environment
        .details
        .allowlisted_values
        .keys()
        .chain(environment.details.hashed_values.keys())
        .cloned()
        .collect::<Vec<_>>();
    fields.sort();
    fields.dedup();
    let mut compact = CompactReceipt {
        spdx: "CC0-1.0".to_owned(),
        schema_version: 2,
        protocol_version: PROTOCOL_VERSION,
        protocol_status: PROTOCOL_STATUS.to_owned(),
        receipt_id: Sha256("0".repeat(64)),
        status: "all-passed".to_owned(),
        source_version,
        audit_id,
        candidate: CompactCandidate {
            parent_commit_sha: before.parent,
            tree_sha: before.tree,
            path_manifest_sha256: manifest_sha,
            path_count: before.manifest.len(),
            class_manifests,
        },
        verification: CompactVerification {
            command_sha256: command_digest,
            transcript_sha256: transcript_sha.clone(),
            started_at_utc: started,
            finished_at_utc: finished,
            results: vec![VerificationResult {
                command: FULL_COMMAND.to_owned(),
                exit_code: 0,
            }],
        },
        engine,
        environment: CompactEnvironment {
            sha256: environment.sha256,
            fields,
        },
        local_evidence: LocalEvidence {
            expanded_manifest_sha256: Sha256::of(&expanded_bytes),
            transcript_sha256: transcript_sha,
        },
        evidence_ceiling: EVIDENCE_CEILING.to_owned(),
    };
    compact.receipt_id = compact.digest()?;
    let receipt_id = compact.receipt_id.as_str().to_owned();
    let evidence_directory = git_common_directory(root)?
        .join(EVIDENCE_SUBDIRECTORY)
        .join(format!("sha256-{receipt_id}"));
    if evidence_directory.exists() {
        return Err(receipt_error("local evidence directory already exists"));
    }
    fs::create_dir_all(&evidence_directory)?;
    fs::set_permissions(&evidence_directory, fs::Permissions::from_mode(0o700))?;
    atomic_write(
        &evidence_directory.join("expanded-manifest.json"),
        &expanded_bytes,
        0o600,
    )?;
    atomic_write(
        &evidence_directory.join("transcript.log"),
        &transcript,
        0o600,
    )?;
    atomic_write(
        &evidence_directory.join("command-results.json"),
        &pretty(&expanded.verification)?,
        0o600,
    )?;

    fs::create_dir_all(&output_directory)?;
    let receipt_path = output_directory.join(format!("sha256-{receipt_id}.json"));
    let receipt_bytes = pretty(&compact)?;
    if receipt_path.exists() {
        if fs::read(&receipt_path)? != receipt_bytes {
            return Err(receipt_error("content-addressed receipt path collision"));
        }
    } else {
        atomic_write(&receipt_path, &receipt_bytes, 0o644)?;
    }
    let relative = receipt_path
        .strip_prefix(root)
        .map_err(|_| receipt_error("receipt output escapes the repository"))?;
    writeln!(output, "{}", relative.display())?;
    Ok(EmittedReceipt {
        path: receipt_path,
        receipt_id,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Transition {
    Audit,
    Closure,
    Tracker,
}

impl Transition {
    pub(crate) fn parse(value: &str) -> Result<Self, Error> {
        match value {
            "audit" => Ok(Self::Audit),
            "closure" => Ok(Self::Closure),
            "tracker" => Ok(Self::Tracker),
            _ => Err(receipt_error(format!("unknown transition: {value}"))),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Audit => "audit",
            Self::Closure => "closure",
            Self::Tracker => "tracker",
        }
    }
}

#[derive(Clone, Debug)]
struct PathChange {
    old: Option<ManifestEntry>,
    new: Option<ManifestEntry>,
}

fn changed_paths(old: &[ManifestEntry], new: &[ManifestEntry]) -> BTreeMap<String, PathChange> {
    let old = old
        .iter()
        .map(|entry| (entry.path.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    let new = new
        .iter()
        .map(|entry| (entry.path.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    old.keys()
        .chain(new.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|path| {
            let old_entry = old.get(path);
            let new_entry = new.get(path);
            (old_entry != new_entry).then(|| {
                (
                    path.clone(),
                    PathChange {
                        old: old_entry.cloned(),
                        new: new_entry.cloned(),
                    },
                )
            })
        })
        .collect()
}

fn validate_modes(
    changes: &BTreeMap<String, PathChange>,
    receipt_path: Option<&str>,
) -> Result<(), Error> {
    for (path, change) in changes {
        match (&change.old, &change.new) {
            (None, Some(new)) if Some(path.as_str()) == receipt_path && new.mode == "100644" => {}
            (Some(old), Some(new)) if old.mode == new.mode && old.kind == new.kind => {}
            (None, _) => {
                return Err(receipt_error(format!(
                    "unauthorised added path or mode: {path}"
                )));
            }
            _ => {
                return Err(receipt_error(format!(
                    "mode, type, or deletion is not administrative: {path}"
                )));
            }
        }
    }
    Ok(())
}

fn receipt_relative(root: &Path, path: &Path) -> Result<String, Error> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = fs::canonicalize(&absolute)?;
    let root = fs::canonicalize(root)?;
    let relative = resolved
        .strip_prefix(root)
        .map_err(|_| receipt_error("receipt must be inside the repository"))?
        .to_string_lossy()
        .replace('\\', "/");
    if !relative.starts_with("new-book-plans/verification-receipts/") {
        return Err(receipt_error(
            "receipt must be in the tracked receipt directory",
        ));
    }
    Ok(relative)
}

fn passing_audit(
    pending: &PendingAudit,
    receipt: &CompactReceipt,
    receipt_path: &str,
) -> Result<PassingAudit, Error> {
    let id = pending_audit_id(&pending.id).ok_or_else(|| {
        receipt_error("verified candidate must end in one exact protocol-v6 pending audit")
    })?;
    if pending.result != "pending" {
        return Err(receipt_error(
            "candidate audit is not the next pending FS-SAU row",
        ));
    }
    if id != receipt.audit_id {
        return Err(receipt_error("pending audit id does not match the receipt"));
    }
    if pending.source_version != receipt.source_version {
        return Err(receipt_error(
            "pending audit source does not match the receipt",
        ));
    }
    if pending.title.matches("pending").count() != 1 {
        return Err(receipt_error(
            "pending audit title must identify its pending state",
        ));
    }
    if pending.commands.is_empty() {
        return Err(receipt_error("pending audit command chain is missing"));
    }
    let mut commands = pending.commands.clone();
    commands.push(format!(
        "./verify.sh --commit-gate {receipt_path} --transition audit"
    ));
    Ok(PassingAudit {
        id: id.to_owned(),
        title: pending.title.replacen("pending", "passing", 1),
        source_version: pending.source_version.clone(),
        scope_sha256: pending.scope_sha256.clone(),
        protocol_sha256: pending.protocol_sha256.clone(),
        executed_at_utc: receipt.verification.finished_at_utc.clone(),
        method: pending.method.clone(),
        criterion_coverage: pending.criterion_coverage.clone(),
        control_refs: pending.control_refs.clone(),
        commands,
        finding_refs: pending.finding_refs.clone(),
        result: "passed-with-recorded-limits".to_owned(),
        policy_basis: pending.policy_basis.clone(),
        evidence_ceiling: pending.evidence_ceiling.clone(),
        verification_receipt_ref: receipt_path.to_owned(),
    })
}

fn top_level_equal_except(old: &[u8], new: &[u8], exceptions: &[&str]) -> Result<bool, Error> {
    let old_fields = object_fields(old)?;
    let new_fields = object_fields(new)?;
    let exceptions = exceptions.iter().copied().collect::<BTreeSet<_>>();
    let old_names = old_fields
        .keys()
        .filter(|name| !exceptions.contains(name.as_str()))
        .collect::<Vec<_>>();
    let new_names = new_fields
        .keys()
        .filter(|name| !exceptions.contains(name.as_str()))
        .collect::<Vec<_>>();
    if old_names != new_names {
        return Ok(false);
    }
    Ok(old_names.into_iter().all(|name| {
        let old_span = old_fields[name];
        let new_span = new_fields[name];
        old[old_span.start..old_span.end] == new[new_span.start..new_span.end]
    }))
}

fn scope_audit_elements(ledger: &[u8]) -> Result<(&[u8], Vec<JsonSpan>), Error> {
    let fields = object_fields(ledger)?;
    let audits = field_bytes(ledger, &fields, "scope_audits")?;
    let elements = array_elements(audits)?;
    Ok((audits, elements))
}

fn exact_prefix(old: &[u8], old_rows: &[JsonSpan], new: &[u8], new_rows: &[JsonSpan]) -> bool {
    old_rows.len() <= new_rows.len()
        && old_rows
            .iter()
            .zip(new_rows)
            .all(|(left, right)| old[left.start..left.end] == new[right.start..right.end])
}

fn validate_audit_transition(
    root: &Path,
    old_manifest: &[ManifestEntry],
    new_manifest: &[ManifestEntry],
    receipt: &CompactReceipt,
    receipt_path: &str,
) -> Result<(), Error> {
    let changes = changed_paths(old_manifest, new_manifest);
    let allowed = BTreeSet::from([
        LEDGER_PATH,
        receipt_path,
        AUDIT_GENERATED_PATHS[0],
        AUDIT_GENERATED_PATHS[1],
    ]);
    if changes.keys().map(String::as_str).collect::<BTreeSet<_>>() != allowed {
        return Err(receipt_error(
            "audit transition must change exactly the audit source, receipt, and deterministic ledger projections",
        ));
    }
    validate_modes(&changes, Some(receipt_path))?;
    if blob_at(root, new_manifest, receipt_path)? != pretty(receipt)? {
        return Err(receipt_error(
            "tracked receipt bytes differ from validated receipt",
        ));
    }
    let old = blob_at(root, old_manifest, LEDGER_PATH)?;
    let new = blob_at(root, new_manifest, LEDGER_PATH)?;
    if !top_level_equal_except(&old, &new, &["scope_audits"])? {
        return Err(receipt_error(
            "audit transition changed non-audit ledger values",
        ));
    }
    let old_projection = ledger_closure_projection(&old)?;
    ledger_closure_projection(&new)?;
    let source_version = old_projection.source_version;
    if source_version != receipt.source_version {
        return Err(receipt_error(
            "candidate ledger source does not match the receipt",
        ));
    }
    let (old_audits, old_rows) = scope_audit_elements(&old)?;
    let (new_audits, new_rows) = scope_audit_elements(&new)?;
    if old_rows.is_empty()
        || new_rows.len() != old_rows.len() + 1
        || !exact_prefix(old_audits, &old_rows, new_audits, &new_rows)
    {
        return Err(receipt_error(
            "audit history must be exact-prefix append-only",
        ));
    }
    let pending_span = old_rows.last().expect("nonempty checked");
    let pending: PendingAudit = parse_typed(
        &old_audits[pending_span.start..pending_span.end],
        "verified candidate pending audit is malformed",
    )?;
    let passing_span = new_rows.last().expect("new audit exists");
    let passing: PassingAudit = parse_typed(
        &new_audits[passing_span.start..passing_span.end],
        "passing audit is malformed",
    )?;
    if passing != passing_audit(&pending, receipt, receipt_path)? {
        return Err(receipt_error(
            "passing audit is not the exact pending-row/receipt derivation",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimLimitation {
    defect_ref: String,
    affected_claim_ref: String,
    public_claim_restriction: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureRecord {
    gate: String,
    permitted_claim: String,
    candidate_commit_sha: GitSha,
    source_version: String,
    scope_sha256: Sha256,
    envelope_ref: String,
    audit_cutoff_at_utc: UtcTimestamp,
    scope_audit_ref: String,
    assurance_record_refs: Vec<String>,
    residual_refs: Vec<String>,
    claim_limitations: Vec<ClaimLimitation>,
    verification_receipt_ref: String,
    closure_policy_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptanceGate {
    verdict: String,
    rollup_rule: String,
    gate_a_status: String,
}

/// Receipt-owned strict projection of the reviewed ledger root.
///
/// `checks::ledger` already depends on this module for receipt validation, so
/// importing its full `LedgerDocument` here would create a dependency cycle.
/// Every legitimate root key is therefore listed explicitly. Unconsumed
/// payloads use `IgnoredAny`, while the three values used by receipt transitions
/// retain their concrete types. A new ledger root field fails closed here until
/// it is reviewed and added to both strict schemas.
#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerClosureProjection {
    spdx: IgnoredAny,
    schema_version: IgnoredAny,
    title: IgnoredAny,
    status: IgnoredAny,
    evidence_role: IgnoredAny,
    source_version: String,
    bound_sources_sha256: IgnoredAny,
    axes: IgnoredAny,
    scope_disposition_meanings: IgnoredAny,
    gate_applicability_meanings: IgnoredAny,
    routing_marker_meanings: IgnoredAny,
    posture_meanings: IgnoredAny,
    unestablished_disposition_meanings: IgnoredAny,
    evidence_kind_meanings: IgnoredAny,
    overlay_meanings: IgnoredAny,
    route_status_meanings: IgnoredAny,
    defect_disposition_meanings: IgnoredAny,
    response_stage_meanings: IgnoredAny,
    resolution_status_meanings: IgnoredAny,
    proposal_disposition_meanings: IgnoredAny,
    envelope_status_meanings: IgnoredAny,
    value_status_meanings: IgnoredAny,
    lawful_source_meanings: IgnoredAny,
    role_kind_meanings: IgnoredAny,
    scale_meanings: IgnoredAny,
    power_position_meanings: IgnoredAny,
    role_anchor_meanings: IgnoredAny,
    flow_kind_meanings: IgnoredAny,
    dependency_class_meanings: IgnoredAny,
    loop_kind_meanings: IgnoredAny,
    lifecycle_path_meanings: IgnoredAny,
    scenario_kind_meanings: IgnoredAny,
    collision_axis_meanings: IgnoredAny,
    shock_kind_meanings: IgnoredAny,
    protected_sphere_form_meanings: IgnoredAny,
    compatibility_table: IgnoredAny,
    enum_mapping: IgnoredAny,
    enum_mapping_exclusions: IgnoredAny,
    residual_coverage_exclusions: IgnoredAny,
    id_registry: IgnoredAny,
    domains: IgnoredAny,
    legacy_rows: IgnoredAny,
    claims: IgnoredAny,
    bodies: IgnoredAny,
    routes: IgnoredAny,
    external_assumptions: IgnoredAny,
    envelope: IgnoredAny,
    roles: IgnoredAny,
    role_omissions: IgnoredAny,
    power_source_inventory: IgnoredAny,
    power_population: IgnoredAny,
    coverage_population: IgnoredAny,
    powers: IgnoredAny,
    power_contract_templates: IgnoredAny,
    power_refusals: IgnoredAny,
    power_crosswalk_dispositions: IgnoredAny,
    coverage_families: IgnoredAny,
    dependencies: IgnoredAny,
    dependency_loops: IgnoredAny,
    refused_flows: IgnoredAny,
    scenarios: IgnoredAny,
    scenario_omissions: IgnoredAny,
    thresholds: IgnoredAny,
    defects: IgnoredAny,
    receipts: IgnoredAny,
    proposals: IgnoredAny,
    review_events: IgnoredAny,
    review_protocol: IgnoredAny,
    review_commissions: IgnoredAny,
    deferred_populations: IgnoredAny,
    stopping_rule: IgnoredAny,
    severity_rubric: IgnoredAny,
    functional_criteria: IgnoredAny,
    closure_record: RequiredNullable<ClosureRecord>,
    acceptance_gate: AcceptanceGate,
    closure_requirement_profiles: IgnoredAny,
    closure_claim_contracts: IgnoredAny,
    model_allocations: IgnoredAny,
    function_allocations: IgnoredAny,
    loop_hazard_controls: IgnoredAny,
    bottleneck_dispositions: IgnoredAny,
    scope_audits: IgnoredAny,
    constitutional_effects: IgnoredAny,
}

fn ledger_closure_projection(bytes: &[u8]) -> Result<LedgerClosureProjection, Error> {
    parse_typed(bytes, "full-society ledger closure projection is malformed")
}

#[derive(Clone, Copy)]
struct ForwardRecoveryEpoch {
    candidate: &'static str,
    audit: &'static str,
    receipt_path: &'static str,
    source_version: &'static str,
    audit_id: &'static str,
    expected_parent: &'static str,
}

const FORWARD_RECOVERY_EPOCHS: [ForwardRecoveryEpoch; 2] = [
    ForwardRecoveryEpoch {
        candidate: FORWARD_RECOVERY_FIRST_CANDIDATE,
        audit: FORWARD_RECOVERY_FIRST_AUDIT,
        receipt_path: FORWARD_RECOVERY_FIRST_RECEIPT,
        source_version: FORWARD_RECOVERY_FIRST_SOURCE_VERSION,
        audit_id: FORWARD_RECOVERY_FIRST_AUDIT_ID,
        expected_parent: FORWARD_RECOVERY_CLOSED_ANCHOR,
    },
    ForwardRecoveryEpoch {
        candidate: FORWARD_RECOVERY_SECOND_CANDIDATE,
        audit: FORWARD_RECOVERY_SECOND_AUDIT,
        receipt_path: FORWARD_RECOVERY_SECOND_RECEIPT,
        source_version: FORWARD_RECOVERY_SECOND_SOURCE_VERSION,
        audit_id: FORWARD_RECOVERY_SECOND_AUDIT_ID,
        expected_parent: FORWARD_RECOVERY_FIRST_AUDIT,
    },
];

fn require_open_ledger(
    root: &Path,
    manifest: &[ManifestEntry],
    source_version: &str,
    context: &str,
) -> Result<(), Error> {
    let ledger = ledger_closure_projection(&blob_at(root, manifest, LEDGER_PATH)?)?;
    if ledger.source_version != source_version
        || ledger.closure_record.0.is_some()
        || ledger.acceptance_gate.gate_a_status != "not-passed"
    {
        return Err(receipt_error(format!(
            "{context} is not the exact open Gate A source"
        )));
    }
    Ok(())
}

fn historical_v5_receipt_at(
    root: &Path,
    manifest: &[ManifestEntry],
    epoch: ForwardRecoveryEpoch,
) -> Result<CompactReceipt, Error> {
    let bytes = blob_at(root, manifest, epoch.receipt_path)?;
    let receipt: CompactReceipt =
        parse_typed(&bytes, "forward-recovery historical receipt is malformed")?;
    validate_compact_protocol(
        &receipt,
        Path::new(epoch.receipt_path),
        HISTORICAL_PROTOCOL_V5_VERSION,
        HISTORICAL_PROTOCOL_V5_STATUS,
        "historical protocol v5",
    )?;
    if receipt.source_version != epoch.source_version || receipt.audit_id != epoch.audit_id {
        return Err(receipt_error(
            "forward-recovery historical receipt source/audit binding drifted",
        ));
    }
    validate_local_evidence(root, &receipt)?;
    Ok(receipt)
}

fn validate_forward_recovery_epoch(root: &Path, epoch: ForwardRecoveryEpoch) -> Result<(), Error> {
    let candidate = GitSha::parse(epoch.candidate, "forward-recovery candidate")?;
    let audit = GitSha::parse(epoch.audit, "forward-recovery audit")?;
    let expected_parent = GitSha::parse(epoch.expected_parent, "forward-recovery predecessor")?;
    let candidate_manifest = tree_manifest(root, candidate.as_str())?;
    let audit_manifest = tree_manifest(root, audit.as_str())?;
    let receipt = historical_v5_receipt_at(root, &audit_manifest, epoch)?;

    if receipt.candidate.parent_commit_sha != expected_parent {
        return Err(receipt_error(
            "forward-recovery historical receipt predecessor drifted",
        ));
    }
    candidate_commit(root, &receipt, &candidate)?;
    require_single_parent(root, &audit, &candidate)?;
    require_open_ledger(
        root,
        &candidate_manifest,
        epoch.source_version,
        "forward-recovery candidate ledger",
    )?;
    require_open_ledger(
        root,
        &audit_manifest,
        epoch.source_version,
        "forward-recovery audit ledger",
    )?;
    validate_audit_transition(
        root,
        &candidate_manifest,
        &audit_manifest,
        &receipt,
        epoch.receipt_path,
    )
}

fn forward_recovery_closed_ledger(
    root: &Path,
    receipt: &CompactReceipt,
) -> Result<LedgerClosureProjection, Error> {
    if receipt.protocol_version != PROTOCOL_VERSION
        || receipt.protocol_status != PROTOCOL_STATUS
        || receipt.audit_id != FORWARD_RECOVERY_AUDIT_ID
        || receipt.candidate.parent_commit_sha.as_str() != FORWARD_RECOVERY_SECOND_AUDIT
    {
        return Err(receipt_error(
            "receipt predecessor is not a closed Gate A source",
        ));
    }

    let anchor_manifest = tree_manifest(root, FORWARD_RECOVERY_CLOSED_ANCHOR)?;
    let anchor = ledger_closure_projection(&blob_at(root, &anchor_manifest, LEDGER_PATH)?)?;
    if anchor.source_version != FORWARD_RECOVERY_CLOSED_SOURCE_VERSION
        || anchor.closure_record.0.is_none()
        || anchor.acceptance_gate.gate_a_status != "passed"
    {
        return Err(receipt_error(
            "forward-recovery closed Gate A anchor drifted",
        ));
    }
    for epoch in FORWARD_RECOVERY_EPOCHS {
        validate_forward_recovery_epoch(root, epoch)?;
    }
    Ok(anchor)
}

fn prior_closed_ledger(
    root: &Path,
    receipt: &CompactReceipt,
) -> Result<LedgerClosureProjection, Error> {
    let prior_manifest = tree_manifest(root, receipt.candidate.parent_commit_sha.as_str())?;
    let bytes = blob_at(root, &prior_manifest, LEDGER_PATH)?;
    let prior = ledger_closure_projection(&bytes)?;
    if prior.closure_record.0.is_some() && prior.acceptance_gate.gate_a_status == "passed" {
        return Ok(prior);
    }
    forward_recovery_closed_ledger(root, receipt)
}

fn validate_closure_transition(
    root: &Path,
    old_manifest: &[ManifestEntry],
    new_manifest: &[ManifestEntry],
    receipt: &CompactReceipt,
    receipt_path: &str,
    audit_commit: &GitSha,
) -> Result<(), Error> {
    let changes = changed_paths(old_manifest, new_manifest);
    let allowed = BTreeSet::from([
        LEDGER_PATH,
        AUDIT_GENERATED_PATHS[0],
        AUDIT_GENERATED_PATHS[1],
    ]);
    if changes.keys().map(String::as_str).collect::<BTreeSet<_>>() != allowed {
        return Err(receipt_error(
            "closure transition must change exactly closure source and deterministic ledger projections",
        ));
    }
    validate_modes(&changes, None)?;
    let old_bytes = blob_at(root, old_manifest, LEDGER_PATH)?;
    let new_bytes = blob_at(root, new_manifest, LEDGER_PATH)?;
    if !top_level_equal_except(
        &old_bytes,
        &new_bytes,
        &["closure_record", "acceptance_gate"],
    )? {
        return Err(receipt_error(
            "closure transition changed non-closure ledger values",
        ));
    }
    let old = ledger_closure_projection(&old_bytes)?;
    let new = ledger_closure_projection(&new_bytes)?;
    if old.source_version != receipt.source_version {
        return Err(receipt_error("closure source does not match the receipt"));
    }
    let (audits, rows) = scope_audit_elements(&old_bytes)?;
    if rows.len() < 2 {
        return Err(receipt_error(
            "closure predecessor has no passing current audit",
        ));
    }
    let pending_row = &rows[rows.len() - 2];
    let passing_row = &rows[rows.len() - 1];
    let pending: PendingAudit = parse_typed(
        &audits[pending_row.start..pending_row.end],
        "closure pending audit is malformed",
    )?;
    let passing: PassingAudit = parse_typed(
        &audits[passing_row.start..passing_row.end],
        "closure passing audit is malformed",
    )?;
    let expected_passing = passing_audit(&pending, receipt, receipt_path)?;
    if passing != expected_passing {
        return Err(receipt_error(
            "closure predecessor audit is not receipt-derived",
        ));
    }
    let prior = prior_closed_ledger(root, receipt)?;
    let prior_closure = prior
        .closure_record
        .0
        .ok_or_else(|| receipt_error("receipt predecessor closure is absent"))?;
    if old.closure_record.0.is_some() {
        return Err(receipt_error(
            "closure transition did not start from null closure",
        ));
    }
    if old.acceptance_gate.gate_a_status != "not-passed" {
        return Err(receipt_error(
            "closure transition did not start from exact open Gate A",
        ));
    }
    let expected_gate = AcceptanceGate {
        verdict: prior.acceptance_gate.verdict,
        rollup_rule: old.acceptance_gate.rollup_rule.clone(),
        gate_a_status: "passed".to_owned(),
    };
    if new.acceptance_gate != expected_gate {
        return Err(receipt_error(
            "acceptance metadata is not the exact derived closure",
        ));
    }
    let expected_closure = ClosureRecord {
        gate: prior_closure.gate,
        permitted_claim: prior_closure.permitted_claim,
        candidate_commit_sha: audit_commit.clone(),
        source_version: receipt.source_version.clone(),
        scope_sha256: passing.scope_sha256,
        envelope_ref: prior_closure.envelope_ref,
        audit_cutoff_at_utc: passing.executed_at_utc,
        scope_audit_ref: passing.id,
        assurance_record_refs: prior_closure.assurance_record_refs,
        residual_refs: prior_closure.residual_refs,
        claim_limitations: prior_closure.claim_limitations,
        verification_receipt_ref: receipt_path.to_owned(),
        closure_policy_ref: passing.policy_basis,
    };
    if new.closure_record.0.as_ref() != Some(&expected_closure) {
        return Err(receipt_error(
            "closure record is not predecessor/audit-derived",
        ));
    }
    let closure = new.closure_record.0.expect("compared as present");
    if closure.residual_refs.iter().any(String::is_empty)
        || closure.residual_refs.iter().collect::<BTreeSet<_>>().len()
            != closure.residual_refs.len()
        || closure.claim_limitations.len() != closure.residual_refs.len()
        || closure
            .claim_limitations
            .iter()
            .zip(&closure.residual_refs)
            .any(|(limitation, residual)| {
                limitation.defect_ref != *residual
                    || limitation.affected_claim_ref.is_empty()
                    || limitation.public_claim_restriction.is_empty()
            })
    {
        return Err(receipt_error(
            "closure residual and limitation sets are malformed",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ActiveReference {
    source: String,
    pointer: String,
    target: String,
    needle: String,
    count: usize,
}

fn walk_json_strings(
    bytes: &[u8],
    pointer: &str,
    strings: &mut Vec<(String, String)>,
) -> Result<(), Error> {
    let mut position = 0;
    skip_ws(bytes, &mut position);
    match bytes.get(position) {
        Some(b'"') => {
            let span = scan_string(bytes, &mut position)?;
            skip_ws(bytes, &mut position);
            if position != bytes.len() {
                return Err(receipt_error("trailing bytes after JSON string"));
            }
            let value: String = serde_json::from_slice(&bytes[span.start..span.end])?;
            strings.push((pointer.to_owned(), value));
        }
        Some(b'[') => {
            for (index, span) in array_elements(bytes)?.into_iter().enumerate() {
                walk_json_strings(
                    &bytes[span.start..span.end],
                    &format!("{pointer}/{index}"),
                    strings,
                )?;
            }
        }
        Some(b'{') => {
            for (key, span) in object_fields(bytes)? {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                walk_json_strings(
                    &bytes[span.start..span.end],
                    &format!("{pointer}/{escaped}"),
                    strings,
                )?;
            }
        }
        Some(_) => {
            // The enclosing duplicate-key preflight has already validated this
            // primitive. It contains no string reference.
        }
        None => return Err(receipt_error("empty JSON value")),
    }
    Ok(())
}

fn safe_reference_path(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphanumeric() || matches!(first, b'_' | b'.'))
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-' | b'/'))
}

fn active_reference_projection(
    root: &Path,
    manifest: &[ManifestEntry],
) -> Result<Vec<ActiveReference>, Error> {
    let paths = manifest_map(manifest);
    let mut projection = Vec::new();
    let mut target_cache = BTreeMap::<String, String>::new();
    for entry in manifest {
        if !entry.path.ends_with(".json")
            || entry
                .path
                .starts_with("new-book-plans/verification-receipts/")
            || entry.kind != GitObjectKind::Blob
        {
            continue;
        }
        let source = blob(root, &entry.object)?;
        if serde_json::from_slice::<serde::de::IgnoredAny>(&source).is_err() {
            continue;
        }
        reject_duplicate_keys(&source, "active-reference JSON")?;
        let mut strings = Vec::new();
        if walk_json_strings(&source, "", &mut strings).is_err() {
            continue;
        }
        for (pointer, value) in strings {
            if value.matches("::").count() != 1 {
                continue;
            }
            let (target, needle) = value.split_once("::").expect("one separator checked");
            if target.is_empty()
                || needle.is_empty()
                || !safe_reference_path(target)
                || !(target.contains('/')
                    || target
                        .rsplit('/')
                        .next()
                        .is_some_and(|name| name.contains('.'))
                    || paths.contains_key(target))
            {
                continue;
            }
            let target_entry = paths
                .get(target)
                .copied()
                .filter(|entry| entry.kind == GitObjectKind::Blob)
                .ok_or_else(|| {
                    receipt_error(format!(
                        "active reference target is absent or not a blob: {target}"
                    ))
                })?;
            if !target_cache.contains_key(target) {
                let body = String::from_utf8(blob(root, &target_entry.object)?).map_err(|_| {
                    receipt_error(format!("reference target is not UTF-8: {target}"))
                })?;
                target_cache.insert(target.to_owned(), body);
            }
            let count = target_cache[target].matches(needle).count();
            if count != 1 {
                return Err(receipt_error(format!(
                    "active reference must occur exactly once; found {count}: {target}::{needle}"
                )));
            }
            projection.push(ActiveReference {
                source: entry.path.clone(),
                pointer,
                target: target.to_owned(),
                needle: needle.to_owned(),
                count,
            });
        }
    }
    Ok(projection)
}

fn line_spans(body: &str) -> Vec<(usize, usize, &str)> {
    let mut spans = Vec::new();
    let mut start = 0;
    for line in body.split_inclusive('\n') {
        let end = start + line.len();
        spans.push((start, end, line));
        start = end;
    }
    if start < body.len() {
        spans.push((start, body.len(), &body[start..]));
    }
    spans
}

fn markdown_header(line: &str) -> bool {
    let hashes = line.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ')
}

fn unchecked_todo_blocks(body: &str) -> Vec<(usize, usize)> {
    let lines = line_spans(body);
    let starts = lines
        .iter()
        .enumerate()
        .filter_map(|(index, (_, _, line))| line.starts_with("- [ ] ").then_some(index))
        .collect::<Vec<_>>();
    starts
        .iter()
        .enumerate()
        .map(|(position, start_line)| {
            let next_item = starts.get(position + 1).copied().unwrap_or(lines.len());
            let end_line = (*start_line + 1..next_item)
                .find(|index| markdown_header(lines[*index].2))
                .unwrap_or(next_item);
            let start = lines[*start_line].0;
            let end = lines.get(end_line).map_or(body.len(), |line| line.0);
            (start, end)
        })
        .collect()
}

fn validate_tracker_transition(
    root: &Path,
    old_manifest: &[ManifestEntry],
    new_manifest: &[ManifestEntry],
) -> Result<(), Error> {
    let changes = changed_paths(old_manifest, new_manifest);
    if changes.keys().map(String::as_str).collect::<Vec<_>>() != [TODO_PATH] {
        return Err(receipt_error("tracker transition may change TODO.md only"));
    }
    validate_modes(&changes, None)?;
    let old = String::from_utf8(blob_at(root, old_manifest, TODO_PATH)?)
        .map_err(|_| receipt_error("TODO.md must remain UTF-8"))?;
    let new = String::from_utf8(blob_at(root, new_manifest, TODO_PATH)?)
        .map_err(|_| receipt_error("TODO.md must remain UTF-8"))?;
    let blocks = unchecked_todo_blocks(&old);
    let (start, end) = blocks
        .first()
        .copied()
        .ok_or_else(|| receipt_error("tracker predecessor has no unchecked TODO block"))?;
    if format!("{}{}", &old[..start], &old[end..]) != new {
        return Err(receipt_error(
            "tracker transition must delete exactly the first whole top-level unchecked TODO block without replacement or reordering",
        ));
    }
    if active_reference_projection(root, old_manifest)?
        != active_reference_projection(root, new_manifest)?
    {
        return Err(receipt_error(
            "tracker transition changed an active path::needle projection",
        ));
    }
    Ok(())
}

fn parents(root: &Path, commit: &GitSha) -> Result<Vec<GitSha>, Error> {
    let text = git_text(root, ["rev-list", "--parents", "-n", "1", commit.as_str()])?;
    let mut fields = text.split_whitespace();
    if fields.next() != Some(commit.as_str()) {
        return Err(receipt_error(format!(
            "cannot inspect commit ancestry: {}",
            commit.as_str()
        )));
    }
    fields
        .map(|field| GitSha::parse(field, "commit parent"))
        .collect()
}

fn require_single_parent(root: &Path, commit: &GitSha, expected: &GitSha) -> Result<(), Error> {
    if parents(root, commit)? == [expected.clone()] {
        Ok(())
    } else {
        Err(receipt_error(
            "merge or intervening commit invalidates receipt reuse",
        ))
    }
}

fn candidate_commit(root: &Path, receipt: &CompactReceipt, commit: &GitSha) -> Result<(), Error> {
    let tree = GitSha::parse(
        git_text(
            root,
            ["rev-parse", &format!("{}^{{tree}}", commit.as_str())],
        )?,
        "candidate commit tree",
    )?;
    if tree != receipt.candidate.tree_sha {
        return Err(receipt_error("candidate commit tree differs from receipt"));
    }
    require_single_parent(root, commit, &receipt.candidate.parent_commit_sha)?;
    let manifest = tree_manifest(root, commit.as_str())?;
    if canonical_digest(&manifest)? != receipt.candidate.path_manifest_sha256
        || manifest.len() != receipt.candidate.path_count
    {
        return Err(receipt_error(
            "candidate commit path manifest differs from receipt",
        ));
    }
    Ok(())
}

pub(crate) fn resolve_candidate_commit(
    context: &Context,
    receipt: &CompactReceipt,
    tip: &str,
) -> Result<GitSha, Error> {
    resolve_candidate_commit_at(context.root(), receipt, tip)
}

fn resolve_candidate_commit_at(
    root: &Path,
    receipt: &CompactReceipt,
    tip: &str,
) -> Result<GitSha, Error> {
    let commits = git_text(root, ["rev-list", "--first-parent", tip])?;
    let mut matches = Vec::new();
    for value in commits.lines() {
        let commit = GitSha::parse(value, "first-parent commit")?;
        let tree = GitSha::parse(
            git_text(
                root,
                ["rev-parse", &format!("{}^{{tree}}", commit.as_str())],
            )?,
            "first-parent tree",
        )?;
        if tree != receipt.candidate.tree_sha {
            continue;
        }
        if parents(root, &commit)? == [receipt.candidate.parent_commit_sha.clone()] {
            matches.push(commit);
        }
    }
    if matches.len() == 1 {
        Ok(matches.remove(0))
    } else {
        Err(receipt_error(
            "receipt candidate does not resolve to one normal first-parent commit",
        ))
    }
}

pub(crate) fn validate_recorded_transition(
    context: &Context,
    receipt: &CompactReceipt,
    successor: &str,
    transition: Transition,
    receipt_path: &str,
) -> Result<GitSha, Error> {
    validate_recorded_transition_at(context.root(), receipt, successor, transition, receipt_path)
}

fn validate_recorded_transition_at(
    root: &Path,
    receipt: &CompactReceipt,
    successor: &str,
    transition: Transition,
    receipt_path: &str,
) -> Result<GitSha, Error> {
    let successor = GitSha::parse(successor, "recorded successor")?;
    let successor_parents = parents(root, &successor)?;
    if successor_parents.len() != 1 {
        return Err(receipt_error(
            "recorded administrative successor must not be a merge",
        ));
    }
    let predecessor = successor_parents[0].clone();
    match transition {
        Transition::Audit => {
            candidate_commit(root, receipt, &predecessor)?;
            validate_audit_transition(
                root,
                &tree_manifest(root, predecessor.as_str())?,
                &tree_manifest(root, successor.as_str())?,
                receipt,
                receipt_path,
            )?;
        }
        Transition::Closure => {
            validate_recorded_transition_at(
                root,
                receipt,
                predecessor.as_str(),
                Transition::Audit,
                receipt_path,
            )?;
            validate_closure_transition(
                root,
                &tree_manifest(root, predecessor.as_str())?,
                &tree_manifest(root, successor.as_str())?,
                receipt,
                receipt_path,
                &predecessor,
            )?;
        }
        Transition::Tracker => {
            return Err(receipt_error("unknown recorded transition: tracker"));
        }
    }
    Ok(predecessor)
}

pub(crate) fn validate_commit_gate(
    context: &Context,
    receipt_path: &Path,
    transition: Transition,
) -> Result<GitSha, Error> {
    let validated = load_and_validate(context, receipt_path, ValidationOptions::default())?;
    let receipt = validated.v2()?;
    validate_commit_gate_for_receipt(context.root(), receipt_path, transition, receipt)
}

fn validate_commit_gate_for_receipt(
    root: &Path,
    receipt_path: &Path,
    transition: Transition,
    receipt: &CompactReceipt,
) -> Result<GitSha, Error> {
    let relative_receipt = receipt_relative(root, receipt_path)?;
    let staged = fully_staged_candidate(root)?;
    let head = GitSha::parse(git_text(root, ["rev-parse", "HEAD"])?, "HEAD")?;
    match transition {
        Transition::Audit => {
            candidate_commit(root, receipt, &head)?;
            validate_audit_transition(
                root,
                &tree_manifest(root, head.as_str())?,
                &staged.manifest,
                receipt,
                &relative_receipt,
            )?;
        }
        Transition::Closure => {
            let audit_parents = parents(root, &head)?;
            if audit_parents.len() != 1 {
                return Err(receipt_error("audit successor must not be a merge"));
            }
            let candidate = &audit_parents[0];
            candidate_commit(root, receipt, candidate)?;
            validate_audit_transition(
                root,
                &tree_manifest(root, candidate.as_str())?,
                &tree_manifest(root, head.as_str())?,
                receipt,
                &relative_receipt,
            )?;
            validate_closure_transition(
                root,
                &tree_manifest(root, head.as_str())?,
                &staged.manifest,
                receipt,
                &relative_receipt,
                &head,
            )?;
        }
        Transition::Tracker => {
            let closure_parents = parents(root, &head)?;
            if closure_parents.len() != 1 {
                return Err(receipt_error("closure successor must not be a merge"));
            }
            let audit = &closure_parents[0];
            let audit_parents = parents(root, audit)?;
            if audit_parents.len() != 1 {
                return Err(receipt_error("audit successor must not be a merge"));
            }
            let candidate = &audit_parents[0];
            candidate_commit(root, receipt, candidate)?;
            validate_audit_transition(
                root,
                &tree_manifest(root, candidate.as_str())?,
                &tree_manifest(root, audit.as_str())?,
                receipt,
                &relative_receipt,
            )?;
            validate_closure_transition(
                root,
                &tree_manifest(root, audit.as_str())?,
                &tree_manifest(root, head.as_str())?,
                receipt,
                &relative_receipt,
                audit,
            )?;
            validate_tracker_transition(
                root,
                &tree_manifest(root, head.as_str())?,
                &staged.manifest,
            )?;
        }
    }
    Ok(staged.tree)
}

/// Validate an administrative successor, run the native quick structural
/// suite in process, and revalidate all staged, environment, and engine bytes.
/// The caller owns the heavyweight verifier lock.
pub(crate) fn run_commit_gate<W, F>(
    context: &Context,
    receipt_path: &Path,
    transition: Transition,
    output: &mut W,
    run_quick: F,
) -> Result<GitSha, Error>
where
    W: Write,
    F: FnOnce(&mut dyn Write) -> Result<(), Error>,
{
    run_commit_gate_with_runtime(
        context.root(),
        receipt_path,
        transition,
        output,
        run_quick,
        runtime_bindings,
    )
}

fn run_commit_gate_with_runtime<W, F, P>(
    root: &Path,
    receipt_path: &Path,
    transition: Transition,
    output: &mut W,
    run_quick: F,
    mut probe_runtime: P,
) -> Result<GitSha, Error>
where
    W: Write,
    F: FnOnce(&mut dyn Write) -> Result<(), Error>,
    P: FnMut(&Path) -> Result<RuntimeBindings, Error>,
{
    let before = fully_staged_candidate(root)?;
    let initial_runtime = probe_runtime(root)?;
    let validated = load_and_validate_with_runtime(
        root,
        receipt_path,
        ValidationOptions::default(),
        &initial_runtime,
    )?;
    let tree = validate_commit_gate_for_receipt(root, receipt_path, transition, validated.v2()?)?;
    run_quick(output).map_err(|error| {
        receipt_error(format!(
            "structural quick verification failed; no full fallback was run: {error}"
        ))
    })?;
    output.flush()?;
    if fully_staged_candidate(root)? != before {
        return Err(receipt_error(
            "staged repository inputs drifted during structural verification",
        ));
    }
    let final_runtime = probe_runtime(root)?;
    load_and_validate_with_runtime(
        root,
        receipt_path,
        ValidationOptions::default(),
        &final_runtime,
    )?;
    Ok(tree)
}

pub(crate) fn validation_success(receipt: &ValidatedReceipt) -> String {
    format!(
        "verification receipt schema v{}: ok",
        receipt.schema_version()
    )
}

pub(crate) fn gate_success(transition: Transition, tree: &GitSha) -> String {
    format!(
        "verification commit gate {}: ok ({})",
        transition.as_str(),
        tree.as_str()
    )
}

const RECEIPT_REPOSITORY_SELF_TEST_SCENARIOS: usize = 14;
const RECEIPT_PROTOCOL_SELF_TEST_SCENARIOS: usize = 20;
const SELF_TEST_SOURCE_VERSION: &str = "fs-ledger-native-receipt-self-test-v1";
const SELF_TEST_AUDIT_ID: &str = "FS-SAU-43";

// The disposable repository exercises receipt transitions rather than the
// ledger checker, but receipt validation now requires the ledger's exact root
// shape. Keep its unconsumed fields explicit here so the fixture cannot mask a
// production schema addition. Unit values are sufficient because the receipt
// projection deliberately ignores these field payloads after key validation.
const SELF_TEST_IGNORED_LEDGER_ROOT_FIELDS: &[&str] = &[
    "spdx",
    "schema_version",
    "title",
    "status",
    "evidence_role",
    "bound_sources_sha256",
    "axes",
    "scope_disposition_meanings",
    "gate_applicability_meanings",
    "routing_marker_meanings",
    "posture_meanings",
    "unestablished_disposition_meanings",
    "evidence_kind_meanings",
    "overlay_meanings",
    "route_status_meanings",
    "defect_disposition_meanings",
    "response_stage_meanings",
    "resolution_status_meanings",
    "proposal_disposition_meanings",
    "envelope_status_meanings",
    "value_status_meanings",
    "lawful_source_meanings",
    "role_kind_meanings",
    "scale_meanings",
    "power_position_meanings",
    "role_anchor_meanings",
    "flow_kind_meanings",
    "dependency_class_meanings",
    "loop_kind_meanings",
    "lifecycle_path_meanings",
    "scenario_kind_meanings",
    "collision_axis_meanings",
    "shock_kind_meanings",
    "protected_sphere_form_meanings",
    "compatibility_table",
    "enum_mapping",
    "enum_mapping_exclusions",
    "residual_coverage_exclusions",
    "id_registry",
    "domains",
    "legacy_rows",
    "claims",
    "bodies",
    "routes",
    "external_assumptions",
    "envelope",
    "roles",
    "role_omissions",
    "power_source_inventory",
    "power_population",
    "coverage_population",
    "powers",
    "power_contract_templates",
    "power_refusals",
    "power_crosswalk_dispositions",
    "coverage_families",
    "dependencies",
    "dependency_loops",
    "refused_flows",
    "scenarios",
    "scenario_omissions",
    "thresholds",
    "defects",
    "receipts",
    "proposals",
    "review_events",
    "review_protocol",
    "review_commissions",
    "deferred_populations",
    "stopping_rule",
    "severity_rubric",
    "functional_criteria",
    "closure_requirement_profiles",
    "closure_claim_contracts",
    "model_allocations",
    "function_allocations",
    "loop_hazard_controls",
    "bottleneck_dispositions",
    "constitutional_effects",
];

fn self_test_ignored_ledger_root_fields() -> BTreeMap<&'static str, ()> {
    SELF_TEST_IGNORED_LEDGER_ROOT_FIELDS
        .iter()
        .copied()
        .map(|name| (name, ()))
        .collect()
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum SelfTestAudit {
    Pending(PendingAudit),
    Passing(PassingAudit),
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct SelfTestLedger {
    source_version: String,
    scope_audits: Vec<SelfTestAudit>,
    closure_record: RequiredNullable<ClosureRecord>,
    acceptance_gate: AcceptanceGate,
    #[serde(flatten)]
    ignored_root_fields: BTreeMap<&'static str, ()>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct SelfTestLegacyReceipt {
    candidate_commit_sha: String,
    commands: Vec<String>,
    result: String,
    transcript_sha256: String,
}

struct ReceiptSelfTestFixture {
    _temporary: tempfile::TempDir,
    root: PathBuf,
    runtime: RuntimeBindings,
    pending: PendingAudit,
    ledger: SelfTestLedger,
}

fn receipt_self_test_error(number: usize, name: &str, message: impl Into<String>) -> Error {
    Error::new(format!(
        "verification receipt self-test scenario {number:02} ({name}): {}",
        message.into()
    ))
}

fn receipt_self_test_require(
    condition: bool,
    number: usize,
    name: &str,
    message: &'static str,
) -> Result<(), Error> {
    if condition {
        Ok(())
    } else {
        Err(receipt_self_test_error(number, name, message))
    }
}

fn receipt_self_test_git(
    root: &Path,
    arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), Error> {
    git_checked(root, arguments).map(|_| ())
}

fn receipt_self_test_write(path: &Path, body: impl AsRef<[u8]>) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)?;
    Ok(())
}

fn receipt_self_test_runtime() -> Result<RuntimeBindings, Error> {
    let details = EnvironmentDetails {
        allowlisted_values: BTreeMap::from([("LANG".to_owned(), "C".to_owned())]),
        hashed_values: BTreeMap::new(),
        platform: PlatformIdentity {
            system: "Linux".to_owned(),
            release: "native-self-test".to_owned(),
            machine: "test-machine".to_owned(),
        },
        tools: BTreeMap::from([
            ("bash".to_owned(), "self-test bash".to_owned()),
            ("git".to_owned(), "self-test git".to_owned()),
            (
                "rights-verify".to_owned(),
                "rights-verify native-self-test".to_owned(),
            ),
        ]),
    };
    Ok(RuntimeBindings {
        engine: EngineIdentity {
            binary_basename: "rights-verify".to_owned(),
            binary_path_sha256: Sha256("a".repeat(64)),
            binary_sha256: Sha256("b".repeat(64)),
            binary_size: 128,
            source_override: false,
            source_available: true,
            source_commit_sha: RequiredNullable(Some(GitSha("c".repeat(40)))),
            source_dirty: RequiredNullable(Some(false)),
            source_status_sha256: RequiredNullable(Some(Sha256("d".repeat(64)))),
            source_diff_sha256: RequiredNullable(Some(Sha256("e".repeat(64)))),
            source_untracked_count: RequiredNullable(Some(0)),
            source_untracked_sha256: RequiredNullable(Some(Sha256("f".repeat(64)))),
        },
        environment: ExpandedEnvironment {
            sha256: canonical_digest(&details)?,
            details,
        },
        compiled_verifier_inputs_sha256: Sha256("0".repeat(64)),
    })
}

impl ReceiptSelfTestFixture {
    fn initialise() -> Result<Self, Error> {
        let temporary = tempfile::Builder::new()
            .prefix("rights-receipt-self-test-")
            .tempdir()?;
        let root = temporary.path().join("repo");
        fs::create_dir_all(&root)?;
        receipt_self_test_git(&root, ["init", "--quiet"])?;
        receipt_self_test_git(&root, ["config", "user.name", "Receipt Tests"])?;
        receipt_self_test_git(
            &root,
            ["config", "user.email", "receipt-tests@example.invalid"],
        )?;
        receipt_self_test_write(&root.join("verify.sh"), "#!/bin/sh\nexit 0\n")?;
        fs::set_permissions(root.join("verify.sh"), fs::Permissions::from_mode(0o755))?;
        receipt_self_test_write(
            &root.join(PROTOCOL_PATH),
            format!("Status: {PROTOCOL_STATUS}\n\nPolicy\n"),
        )?;
        let todo = concat!(
            "# TODO\n\n",
            "- [ ] **Temporary verifier item.**\n",
            "  Remove this exact block after closure.\n\n",
            "- [ ] **Specify obligations without reciprocal bargains.**\n",
            "  This remains unfinished.\n",
        )
        .to_owned();
        receipt_self_test_write(&root.join(TODO_PATH), &todo)?;
        let prior_closure = ClosureRecord {
            gate: "gate-a".to_owned(),
            permitted_claim: "prior structural claim".to_owned(),
            candidate_commit_sha: GitSha("0".repeat(40)),
            source_version: "fs-ledger-prior-closed-v1".to_owned(),
            scope_sha256: Sha256("0".repeat(64)),
            envelope_ref: "FS-ENV-01".to_owned(),
            audit_cutoff_at_utc: UtcTimestamp("2026-08-20T00:00:00Z".to_owned()),
            scope_audit_ref: "FS-SAU-42".to_owned(),
            assurance_record_refs: vec!["FS-ASR-01".to_owned()],
            residual_refs: vec!["FS-DEF-01".to_owned()],
            claim_limitations: vec![ClaimLimitation {
                defect_ref: "FS-DEF-01".to_owned(),
                affected_claim_ref: "FS-CLM-01".to_owned(),
                public_claim_restriction: "No operational claim.".to_owned(),
            }],
            verification_receipt_ref: "new-book-plans/verification-receipts/prior.json".to_owned(),
            closure_policy_ref: "protocol::Policy".to_owned(),
        };
        let prior = SelfTestLedger {
            source_version: "fs-ledger-prior-closed-v1".to_owned(),
            scope_audits: Vec::new(),
            closure_record: RequiredNullable(Some(prior_closure)),
            acceptance_gate: AcceptanceGate {
                verdict: "prior structural verdict".to_owned(),
                rollup_rule: "all conditions".to_owned(),
                gate_a_status: "passed".to_owned(),
            },
            ignored_root_fields: self_test_ignored_ledger_root_fields(),
        };
        receipt_self_test_write(&root.join(LEDGER_PATH), pretty(&prior)?)?;
        receipt_self_test_write(&root.join("candidate.txt"), "base\n")?;
        receipt_self_test_write(
            &root.join("new-book-plans/testdata/fixture.txt"),
            "fixture\n",
        )?;
        receipt_self_test_write(
            &root.join("verification-shard-runner.sh"),
            "#!/bin/sh\nexit 0\n",
        )?;
        fs::set_permissions(
            root.join("verification-shard-runner.sh"),
            fs::Permissions::from_mode(0o755),
        )?;
        receipt_self_test_write(
            &root.join("new-book-plans/verification_lock_client.py"),
            "# native compatibility fixture\n",
        )?;
        for path in AUDIT_GENERATED_PATHS {
            receipt_self_test_write(&root.join(path), "base projection\n")?;
        }
        receipt_self_test_write(
            &root.join("new-book-plans/constitutional-closure-and-model-allocation-audit.md"),
            "base closure projection\n",
        )?;
        receipt_self_test_git(&root, ["add", "."])?;
        receipt_self_test_git(&root, ["commit", "--quiet", "-m", "base"])?;

        let pending = PendingAudit {
            id: format!("{SELF_TEST_AUDIT_ID}-PENDING"),
            title: "Receipt-aware repository audit pending".to_owned(),
            source_version: SELF_TEST_SOURCE_VERSION.to_owned(),
            scope_sha256: Sha256("1".repeat(64)),
            protocol_sha256: Sha256("2".repeat(64)),
            executed_at_utc: UtcTimestamp("2026-08-23T00:00:00Z".to_owned()),
            method: "repository adversarial audit".to_owned(),
            criterion_coverage: vec!["semantic scope".to_owned()],
            control_refs: vec!["CTRL-01".to_owned()],
            commands: vec!["rights-verify --native-self-test receipt-protocol".to_owned()],
            finding_refs: vec!["FS-DEF-01".to_owned()],
            result: "pending".to_owned(),
            policy_basis: "new-book-plans/full-society-scope-review-protocol.md::Policy".to_owned(),
            evidence_ceiling: "Repository structure only.".to_owned(),
        };
        let ledger = SelfTestLedger {
            source_version: SELF_TEST_SOURCE_VERSION.to_owned(),
            scope_audits: vec![SelfTestAudit::Pending(pending.clone())],
            closure_record: RequiredNullable(None),
            acceptance_gate: AcceptanceGate {
                verdict: "pending structural verdict".to_owned(),
                rollup_rule: "all conditions".to_owned(),
                gate_a_status: "not-passed".to_owned(),
            },
            ignored_root_fields: self_test_ignored_ledger_root_fields(),
        };
        let mut fixture = Self {
            _temporary: temporary,
            root,
            runtime: receipt_self_test_runtime()?,
            pending,
            ledger,
        };
        fixture.write_ledger()?;
        receipt_self_test_write(&fixture.root.join("candidate.txt"), "candidate\n")?;
        receipt_self_test_git(&fixture.root, ["add", LEDGER_PATH, "candidate.txt"])?;
        fixture.runtime.compiled_verifier_inputs_sha256 =
            verifier_build_input_digest(&fixture.root, &index_manifest(&fixture.root)?)?;
        Ok(fixture)
    }

    fn write_ledger(&mut self) -> Result<(), Error> {
        receipt_self_test_write(&self.root.join(LEDGER_PATH), pretty(&self.ledger)?)
    }

    fn restore_candidate_worktree(&self) -> Result<(), Error> {
        receipt_self_test_git(&self.root, ["restore", "candidate.txt"])
    }

    fn stage_projections(&self, label: &str) -> Result<(), Error> {
        for path in AUDIT_GENERATED_PATHS {
            receipt_self_test_write(&self.root.join(path), format!("{label} projection\n"))?;
        }
        receipt_self_test_git(&self.root, ["add", AUDIT_GENERATED_PATHS[0]])?;
        receipt_self_test_git(&self.root, ["add", AUDIT_GENERATED_PATHS[1]])
    }

    fn stage_passing_audit(
        &mut self,
        receipt_path: &Path,
        receipt: &CompactReceipt,
        passing: PassingAudit,
    ) -> Result<String, Error> {
        let relative = receipt_path
            .strip_prefix(&self.root)
            .map_err(|_| receipt_error("self-test receipt escaped repository"))?
            .to_string_lossy()
            .replace('\\', "/");
        self.ledger.scope_audits = vec![
            SelfTestAudit::Pending(self.pending.clone()),
            SelfTestAudit::Passing(passing),
        ];
        self.write_ledger()?;
        self.stage_projections("audit")?;
        receipt_self_test_git(&self.root, ["add", LEDGER_PATH, &relative])?;
        let expected = passing_audit(&self.pending, receipt, &relative)?;
        if !matches!(self.ledger.scope_audits.last(), Some(SelfTestAudit::Passing(row)) if row == &expected)
        {
            return Err(receipt_error("self-test passing audit is not canonical"));
        }
        Ok(relative)
    }
}

fn run_receipt_repository_self_tests() -> Result<usize, Error> {
    let mut fixture = ReceiptSelfTestFixture::initialise().map_err(|error| {
        receipt_self_test_error(7, "dirty and untracked inputs fail", error.to_string())
    })?;
    let root = fixture.root.clone();

    receipt_self_test_write(&root.join(TODO_PATH), "dirty\n")?;
    receipt_self_test_require(
        fully_staged_candidate(&root).is_err(),
        7,
        "dirty and untracked inputs fail",
        "unstaged tracked input was accepted",
    )?;
    receipt_self_test_git(&root, ["restore", TODO_PATH])?;
    receipt_self_test_write(&root.join("untracked.txt"), "untracked\n")?;
    receipt_self_test_require(
        fully_staged_candidate(&root).is_err(),
        7,
        "dirty and untracked inputs fail",
        "untracked input was accepted",
    )?;
    fs::remove_file(root.join("untracked.txt"))?;

    let runtime = fixture.runtime.clone();
    let drift_path = root.join("candidate.txt");
    let drift_result = emit_receipt_with_runtime(
        &root,
        Path::new(RECEIPT_DIRECTORY),
        &mut Vec::new(),
        move |_writer| {
            receipt_self_test_write(&drift_path, "drift\n")?;
            Ok(())
        },
        move |_root| Ok(runtime.clone()),
    );
    receipt_self_test_require(
        drift_result.is_err(),
        15,
        "emission rejects staged input drift",
        "emission accepted repository drift",
    )?;
    fixture.restore_candidate_worktree()?;

    let stale_verifier_ran = Cell::new(false);
    let mut stale_runtime = fixture.runtime.clone();
    stale_runtime.compiled_verifier_inputs_sha256 = Sha256("9".repeat(64));
    let stale_result = emit_receipt_with_runtime(
        &root,
        Path::new(RECEIPT_DIRECTORY),
        &mut Vec::new(),
        |_writer| {
            stale_verifier_ran.set(true);
            Ok(())
        },
        move |_root| Ok(stale_runtime.clone()),
    );
    receipt_self_test_require(
        stale_result.is_err() && !stale_verifier_ran.get(),
        18,
        "final native executable bytes bind",
        "stale compiled verifier inputs reached authoritative execution",
    )?;

    let verifier_ran = Cell::new(false);
    let missing_result = emit_receipt_with_runtime(
        &root,
        Path::new(RECEIPT_DIRECTORY),
        &mut Vec::new(),
        |_writer| {
            verifier_ran.set(true);
            Ok(())
        },
        |_root| {
            Err(receipt_error(
                "rights-verify executable is missing or empty",
            ))
        },
    );
    receipt_self_test_require(
        missing_result.is_err() && !verifier_ran.get(),
        19,
        "missing authoritative executable fails closed",
        "emission ran without an authoritative executable identity",
    )?;

    let runtime = fixture.runtime.clone();
    let mut emission_output = Vec::new();
    let emitted = emit_receipt_with_runtime(
        &root,
        Path::new(RECEIPT_DIRECTORY),
        &mut emission_output,
        |writer| {
            writeln!(writer, "native verifier PASS")?;
            Ok(())
        },
        move |_root| Ok(runtime.clone()),
    )
    .map_err(|error| {
        receipt_self_test_error(
            8,
            "receipt integrity and retained evidence",
            error.to_string(),
        )
    })?;
    let receipt_bytes = fs::read(&emitted.path)?;
    let validated = validate_receipt_bytes_with_runtime(
        &root,
        &emitted.path,
        &receipt_bytes,
        ValidationOptions::default(),
        Some(&fixture.runtime.environment),
        Some(&fixture.runtime.engine),
    )
    .map_err(|error| {
        receipt_self_test_error(
            8,
            "receipt integrity and retained evidence",
            error.to_string(),
        )
    })?;
    let compact = validated.v2()?.clone();
    receipt_self_test_require(
        compact.audit_id == SELF_TEST_AUDIT_ID
            && emission_output.starts_with(b"native verifier PASS\n"),
        8,
        "receipt integrity and retained evidence",
        "valid native receipt did not round-trip",
    )?;

    fs::remove_file(&emitted.path)?;
    let cached = validate_receipt_bytes_with_runtime(
        &root,
        &emitted.path,
        &receipt_bytes,
        ValidationOptions {
            require_local: false,
            check_environment: false,
            check_engine: false,
            source_version: None,
            audit_id: None,
        },
        None,
        None,
    )?;
    receipt_self_test_require(
        cached.v2()?.receipt_id() == compact.receipt_id(),
        8,
        "receipt integrity and retained evidence",
        "cached compact receipt did not validate without local evidence",
    )?;
    fs::write(&emitted.path, &receipt_bytes)?;

    let wrong_name = emitted
        .path
        .with_file_name(format!("sha256-{}.json", "0".repeat(64)));
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            &wrong_name,
            &receipt_bytes,
            ValidationOptions::default(),
            Some(&fixture.runtime.environment),
            Some(&fixture.runtime.engine),
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "wrong content-addressed filename was accepted",
    )?;
    let mut mutated_receipt = compact.clone();
    mutated_receipt.evidence_ceiling.push_str(" mutated");
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            &emitted.path,
            &pretty(&mutated_receipt)?,
            ValidationOptions::default(),
            Some(&fixture.runtime.environment),
            Some(&fixture.runtime.engine),
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "receipt mutation preserved validity",
    )?;

    let evidence = git_common_directory(&root)?
        .join(EVIDENCE_SUBDIRECTORY)
        .join(format!("sha256-{}", compact.receipt_id()));
    let transcript_path = evidence.join("transcript.log");
    let transcript = fs::read(&transcript_path)?;
    fs::remove_file(&transcript_path)?;
    let missing_evidence = validate_receipt_bytes_with_runtime(
        &root,
        &emitted.path,
        &receipt_bytes,
        ValidationOptions::default(),
        Some(&fixture.runtime.environment),
        Some(&fixture.runtime.engine),
    );
    fs::write(&transcript_path, transcript)?;
    receipt_self_test_require(
        missing_evidence.is_err(),
        8,
        "receipt integrity and retained evidence",
        "missing transcript evidence was accepted",
    )?;

    let mut engine_drift = fixture.runtime.engine.clone();
    engine_drift.binary_sha256 = Sha256("0".repeat(64));
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            &emitted.path,
            &receipt_bytes,
            ValidationOptions::default(),
            Some(&fixture.runtime.environment),
            Some(&engine_drift),
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "engine byte drift was accepted",
    )?;
    let mut source_drift = fixture.runtime.engine.clone();
    source_drift.source_commit_sha = RequiredNullable(Some(GitSha("1".repeat(40))));
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            &emitted.path,
            &receipt_bytes,
            ValidationOptions::default(),
            Some(&fixture.runtime.environment),
            Some(&source_drift),
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "engine source drift was accepted",
    )?;
    let mut environment_drift = fixture.runtime.environment.clone();
    environment_drift.sha256 = Sha256("0".repeat(64));
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            &emitted.path,
            &receipt_bytes,
            ValidationOptions::default(),
            Some(&environment_drift),
            Some(&fixture.runtime.engine),
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "sanitized environment drift was accepted",
    )?;

    let legacy = SelfTestLegacyReceipt {
        candidate_commit_sha: LEGACY_CANDIDATE.to_owned(),
        commands: LEGACY_REQUIRED_COMMANDS.map(str::to_owned).to_vec(),
        result: "all-passed".to_owned(),
        transcript_sha256: LEGACY_TRANSCRIPT_SHA256.to_owned(),
    };
    let legacy_options = ValidationOptions {
        require_local: false,
        check_environment: false,
        check_engine: false,
        source_version: Some(LEGACY_SOURCE_VERSION),
        audit_id: Some(LEGACY_AUDIT_ID),
    };
    receipt_self_test_require(
        matches!(
            validate_receipt_bytes_with_runtime(
                &root,
                Path::new("legacy-v1.json"),
                &pretty(&legacy)?,
                legacy_options.clone(),
                None,
                None,
            ),
            Ok(ValidatedReceipt::Legacy)
        ),
        8,
        "receipt integrity and retained evidence",
        "exact legacy allowlist receipt was rejected",
    )?;
    for commands in [
        legacy.commands[..legacy.commands.len() - 1].to_vec(),
        {
            let mut commands = legacy.commands.clone();
            commands.push("unexpected command".to_owned());
            commands
        },
        legacy.commands.iter().rev().cloned().collect(),
    ] {
        let invalid = SelfTestLegacyReceipt {
            commands,
            ..legacy.clone()
        };
        receipt_self_test_require(
            validate_receipt_bytes_with_runtime(
                &root,
                Path::new("legacy-v1.json"),
                &pretty(&invalid)?,
                legacy_options.clone(),
                None,
                None,
            )
            .is_err(),
            8,
            "receipt integrity and retained evidence",
            "invalid legacy command sequence was accepted",
        )?;
    }
    let wrong_legacy_source = ValidationOptions {
        source_version: Some(SELF_TEST_SOURCE_VERSION),
        ..legacy_options
    };
    receipt_self_test_require(
        validate_receipt_bytes_with_runtime(
            &root,
            Path::new("legacy-v1.json"),
            &pretty(&legacy)?,
            wrong_legacy_source,
            None,
            None,
        )
        .is_err(),
        8,
        "receipt integrity and retained evidence",
        "legacy receipt was accepted outside its exact source allowlist",
    )?;

    receipt_self_test_require(
        strictly_sorted_unique(&compact.environment.fields)
            && compact.environment.fields == ["LANG".to_owned()],
        9,
        "environment fields are globally sorted",
        "compact environment field names are not sorted and unique",
    )?;
    receipt_self_test_require(
        compact.engine == fixture.runtime.engine
            && compact.engine.binary_sha256 == fixture.runtime.engine.binary_sha256
            && compact.engine.binary_size == fixture.runtime.engine.binary_size,
        18,
        "final native executable bytes bind",
        "emitted receipt did not bind the authoritative executable identity",
    )?;

    receipt_self_test_git(&root, ["commit", "--quiet", "-m", "candidate"])?;
    let relative = emitted
        .path
        .strip_prefix(&root)
        .map_err(|_| receipt_error("self-test receipt escaped repository"))?
        .to_string_lossy()
        .replace('\\', "/");
    let passing = passing_audit(&fixture.pending, &compact, &relative)?;
    fixture.stage_passing_audit(&emitted.path, &compact, passing.clone())?;

    receipt_self_test_write(&root.join("unexpected.txt"), "not administrative\n")?;
    receipt_self_test_git(&root, ["add", "unexpected.txt"])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        10,
        "audit rejects unclassified paths",
        "audit transition accepted an unclassified path",
    )?;
    receipt_self_test_git(&root, ["restore", "--staged", "unexpected.txt"])?;
    fs::remove_file(root.join("unexpected.txt"))?;

    receipt_self_test_write(&root.join("verify.sh"), "#!/bin/sh\nexit 1\n")?;
    receipt_self_test_git(&root, ["add", "verify.sh"])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        11,
        "audit rejects verifier and fixture mutations",
        "audit transition accepted verifier mutation",
    )?;
    receipt_self_test_git(&root, ["restore", "--staged", "verify.sh"])?;
    receipt_self_test_git(&root, ["restore", "verify.sh"])?;
    let fixture_path = "new-book-plans/testdata/fixture.txt";
    receipt_self_test_write(&root.join(fixture_path), "mutated fixture\n")?;
    receipt_self_test_git(&root, ["add", fixture_path])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        11,
        "audit rejects verifier and fixture mutations",
        "audit transition accepted fixture mutation",
    )?;
    receipt_self_test_git(&root, ["restore", "--staged", fixture_path])?;
    receipt_self_test_git(&root, ["restore", fixture_path])?;

    let mut semantic_mutation = passing.clone();
    semantic_mutation.title.push_str(" mutated");
    fixture.ledger.scope_audits = vec![
        SelfTestAudit::Pending(fixture.pending.clone()),
        SelfTestAudit::Passing(semantic_mutation),
    ];
    fixture.write_ledger()?;
    receipt_self_test_git(&root, ["add", LEDGER_PATH])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        13,
        "strict audit rejects semantic mutation",
        "non-derived passing audit was accepted",
    )?;
    fixture.ledger.scope_audits = vec![
        SelfTestAudit::Pending(fixture.pending.clone()),
        SelfTestAudit::Passing(passing.clone()),
    ];
    fixture.write_ledger()?;
    receipt_self_test_git(&root, ["add", LEDGER_PATH])?;

    let missing_projection = AUDIT_GENERATED_PATHS[0];
    receipt_self_test_git(&root, ["restore", "--staged", missing_projection])?;
    receipt_self_test_git(&root, ["restore", missing_projection])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        14,
        "audit requires exact projection set",
        "audit transition accepted a missing generated projection",
    )?;
    receipt_self_test_write(&root.join(missing_projection), "audit projection\n")?;
    receipt_self_test_git(&root, ["add", missing_projection])?;
    let unexpected_projection =
        "new-book-plans/constitutional-closure-and-model-allocation-audit.md";
    receipt_self_test_write(
        &root.join(unexpected_projection),
        "unexpected audit projection\n",
    )?;
    receipt_self_test_git(&root, ["add", unexpected_projection])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Audit, &compact)
            .is_err(),
        14,
        "audit requires exact projection set",
        "audit transition accepted an extra generated projection",
    )?;
    receipt_self_test_git(&root, ["restore", "--staged", unexpected_projection])?;
    receipt_self_test_git(&root, ["restore", unexpected_projection])?;

    let quick_drift_path = root.join("candidate.txt");
    let runtime = fixture.runtime.clone();
    let quick_input_drift = run_commit_gate_with_runtime(
        &root,
        &emitted.path,
        Transition::Audit,
        &mut Vec::new(),
        move |_writer| {
            receipt_self_test_write(&quick_drift_path, "quick drift\n")?;
            Ok(())
        },
        move |_root| Ok(runtime.clone()),
    );
    receipt_self_test_require(
        quick_input_drift.is_err(),
        16,
        "commit gate rejects quick input drift",
        "commit gate accepted repository drift during quick verification",
    )?;
    fixture.restore_candidate_worktree()?;

    let probes = Cell::new(0_usize);
    let stable_runtime = fixture.runtime.clone();
    let mut drifted_runtime = fixture.runtime.clone();
    drifted_runtime.engine.binary_sha256 = Sha256("9".repeat(64));
    let quick_engine_drift = run_commit_gate_with_runtime(
        &root,
        &emitted.path,
        Transition::Audit,
        &mut Vec::new(),
        |writer| {
            writeln!(writer, "native quick PASS")?;
            Ok(())
        },
        |_root| {
            let call = probes.get();
            probes.set(call + 1);
            Ok(if call == 0 {
                stable_runtime.clone()
            } else {
                drifted_runtime.clone()
            })
        },
    );
    receipt_self_test_require(
        quick_engine_drift.is_err() && probes.get() == 2,
        17,
        "commit gate rejects post-quick engine drift",
        "commit gate accepted authoritative executable drift",
    )?;

    let runtime = fixture.runtime.clone();
    let mut quick_output = Vec::new();
    run_commit_gate_with_runtime(
        &root,
        &emitted.path,
        Transition::Audit,
        &mut quick_output,
        |writer| {
            writeln!(writer, "native quick PASS")?;
            Ok(())
        },
        move |_root| Ok(runtime.clone()),
    )
    .map_err(|error| {
        receipt_self_test_error(
            12,
            "strict audit closure tracker transitions",
            error.to_string(),
        )
    })?;
    receipt_self_test_require(
        quick_output == b"native quick PASS\n",
        12,
        "strict audit closure tracker transitions",
        "quick verifier transcript was not exact",
    )?;
    receipt_self_test_git(&root, ["commit", "--quiet", "-m", "audit"])?;
    let audit_commit = GitSha::parse(git_text(&root, ["rev-parse", "HEAD"])?, "audit")?;
    let candidate_commit = validate_recorded_transition_at(
        &root,
        &compact,
        audit_commit.as_str(),
        Transition::Audit,
        &relative,
    )?;
    receipt_self_test_require(
        resolve_candidate_commit_at(&root, &compact, "HEAD")? == candidate_commit,
        12,
        "strict audit closure tracker transitions",
        "recorded audit did not resolve its unique candidate",
    )?;

    fixture.ledger.closure_record = RequiredNullable(Some(ClosureRecord {
        gate: "gate-a".to_owned(),
        permitted_claim: "prior structural claim".to_owned(),
        candidate_commit_sha: audit_commit.clone(),
        source_version: SELF_TEST_SOURCE_VERSION.to_owned(),
        scope_sha256: passing.scope_sha256.clone(),
        envelope_ref: "FS-ENV-01".to_owned(),
        audit_cutoff_at_utc: passing.executed_at_utc.clone(),
        scope_audit_ref: passing.id.clone(),
        assurance_record_refs: vec!["FS-ASR-01".to_owned()],
        residual_refs: vec!["FS-DEF-01".to_owned()],
        claim_limitations: vec![ClaimLimitation {
            defect_ref: "FS-DEF-01".to_owned(),
            affected_claim_ref: "FS-CLM-01".to_owned(),
            public_claim_restriction: "No operational claim.".to_owned(),
        }],
        verification_receipt_ref: relative.clone(),
        closure_policy_ref: passing.policy_basis.clone(),
    }));
    fixture.ledger.acceptance_gate = AcceptanceGate {
        verdict: "prior structural verdict".to_owned(),
        rollup_rule: "all conditions".to_owned(),
        gate_a_status: "passed".to_owned(),
    };
    fixture.write_ledger()?;
    fixture.stage_projections("closure")?;
    receipt_self_test_git(&root, ["add", LEDGER_PATH])?;
    receipt_self_test_write(
        &root.join(unexpected_projection),
        "unexpected closure projection\n",
    )?;
    receipt_self_test_git(&root, ["add", unexpected_projection])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Closure, &compact)
            .is_err(),
        12,
        "strict audit closure tracker transitions",
        "closure transition accepted an unrelated generated projection",
    )?;
    receipt_self_test_git(&root, ["restore", "--staged", unexpected_projection])?;
    receipt_self_test_git(&root, ["restore", unexpected_projection])?;
    let runtime = fixture.runtime.clone();
    run_commit_gate_with_runtime(
        &root,
        &emitted.path,
        Transition::Closure,
        &mut Vec::new(),
        |_writer| Ok(()),
        move |_root| Ok(runtime.clone()),
    )?;
    receipt_self_test_git(&root, ["commit", "--quiet", "-m", "closure"])?;
    let closure_commit = GitSha::parse(git_text(&root, ["rev-parse", "HEAD"])?, "closure")?;
    receipt_self_test_require(
        validate_recorded_transition_at(
            &root,
            &compact,
            closure_commit.as_str(),
            Transition::Closure,
            &relative,
        )? == audit_commit,
        12,
        "strict audit closure tracker transitions",
        "recorded closure ancestry was not exact",
    )?;

    let remaining_todo = concat!(
        "# TODO\n\n",
        "- [ ] **Specify obligations without reciprocal bargains.**\n",
        "  This remains unfinished.\n",
    );
    receipt_self_test_write(&root.join(TODO_PATH), remaining_todo)?;
    receipt_self_test_git(&root, ["add", TODO_PATH])?;
    let runtime = fixture.runtime.clone();
    let tracker_tree = run_commit_gate_with_runtime(
        &root,
        &emitted.path,
        Transition::Tracker,
        &mut Vec::new(),
        |_writer| Ok(()),
        move |_root| Ok(runtime.clone()),
    )?;
    receipt_self_test_require(
        gate_success(Transition::Tracker, &tracker_tree)
            .starts_with("verification commit gate tracker: ok ("),
        12,
        "strict audit closure tracker transitions",
        "tracker success diagnostic changed",
    )?;
    let wrong_block = concat!(
        "# TODO\n\n",
        "- [ ] **Temporary verifier item.**\n",
        "  Remove this exact block after closure.\n",
    );
    receipt_self_test_write(&root.join(TODO_PATH), wrong_block)?;
    receipt_self_test_git(&root, ["add", TODO_PATH])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Tracker, &compact)
            .is_err(),
        12,
        "strict audit closure tracker transitions",
        "tracker accepted deletion of the wrong TODO block",
    )?;
    let duplicate_needle = format!(
        "{remaining_todo}\n- [ ] **Specify obligations again.**\n  Duplicate Specify obligations needle.\n"
    );
    receipt_self_test_write(&root.join(TODO_PATH), duplicate_needle)?;
    receipt_self_test_git(&root, ["add", TODO_PATH])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Tracker, &compact)
            .is_err(),
        12,
        "strict audit closure tracker transitions",
        "tracker accepted duplicate active reference needle",
    )?;
    receipt_self_test_write(
        &root.join(TODO_PATH),
        remaining_todo.replace("This remains unfinished.", "This was silently rewritten."),
    )?;
    receipt_self_test_git(&root, ["add", TODO_PATH])?;
    receipt_self_test_require(
        validate_commit_gate_for_receipt(&root, &emitted.path, Transition::Tracker, &compact)
            .is_err(),
        12,
        "strict audit closure tracker transitions",
        "tracker accepted rewrite outside the deleted block",
    )?;

    receipt_self_test_require(
        classify_path("verification-shard-runner.sh") == ManifestClass::VerifierInput
            && classify_path("new-book-plans/verification_lock_client.py")
                == ManifestClass::VerifierInput
            && classify_path("Cargo.toml") == ManifestClass::VerifierInput
            && classify_path("Cargo.lock") == ManifestClass::VerifierInput
            && classify_path("src/receipt.rs") == ManifestClass::VerifierInput
            && classify_path("book-1/01-opening.pins.nibli") == ManifestClass::Fixture
            && classify_path("TODO.md") == ManifestClass::Administrative,
        20,
        "native verifier input classification",
        "native verifier or compatibility input was misclassified",
    )?;
    Ok(RECEIPT_REPOSITORY_SELF_TEST_SCENARIOS)
}

pub(crate) fn self_test() -> Result<String, Error> {
    let lock_count = crate::lock::receipt_protocol_self_test()?;
    let receipt_count = run_receipt_repository_self_tests()?;
    if lock_count + receipt_count != RECEIPT_PROTOCOL_SELF_TEST_SCENARIOS {
        return Err(receipt_error("native receipt self-test count drifted"));
    }
    if !valid_utc_seconds("2026-08-27T00:00:00Z")
        || valid_utc_seconds("2026-02-29T00:00:00Z")
        || !valid_utc_seconds("2028-02-29T23:59:59Z")
    {
        return Err(receipt_error("native receipt UTC self-test failed"));
    }
    receipt_self_test_require(
        fixture_free_self_test_typed_boundaries(),
        20,
        "native verifier input classification",
        "typed JSON boundary smoke controls failed",
    )?;
    Ok("verification-receipt protocol self-test: PASS".to_owned())
}

fn fixture_free_self_test_typed_boundaries() -> bool {
    let todo = "# TODO\n\n- [ ] first\n  body\n\n- [ ] second\n";
    unchecked_todo_blocks(todo) == [(8, 28), (28, todo.len())]
        && reject_duplicate_keys(br#"{"one":{"two":[1,true,null]}}"#, "self-test").is_ok()
        && reject_duplicate_keys(br#"{"one":1,"one":2}"#, "self-test").is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_test_ledger_root(value: &mut serde_json::Value) {
        let root = value
            .as_object_mut()
            .expect("test ledger root is an object");
        root.remove("owner_ref");
        for name in SELF_TEST_IGNORED_LEDGER_ROOT_FIELDS {
            root.insert((*name).to_owned(), serde_json::Value::Null);
        }
    }

    fn test_git(root: &Path, arguments: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn tracked_schema_v2_receipts_keep_exact_self_digest_compatibility() {
        let context = Context::discover().unwrap();
        let directory = context.path(RECEIPT_DIRECTORY);
        let mut paths = fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(OsStr::to_str) == Some("json"))
            .collect::<Vec<_>>();
        paths.sort();
        assert!(!paths.is_empty());
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let receipt: CompactReceipt = parse_typed(&bytes, "tracked compact receipt").unwrap();
            let (version, status, label) =
                if receipt.protocol_version == HISTORICAL_PROTOCOL_V5_VERSION {
                    (
                        HISTORICAL_PROTOCOL_V5_VERSION,
                        HISTORICAL_PROTOCOL_V5_STATUS,
                        "historical protocol v5",
                    )
                } else {
                    (PROTOCOL_VERSION, PROTOCOL_STATUS, "current protocol v6")
                };
            validate_compact_protocol(&receipt, &path, version, status, label)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            validate_local_evidence(context.root(), &receipt)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert_eq!(
                path.file_name().and_then(OsStr::to_str).unwrap(),
                format!("sha256-{}.json", receipt.receipt_id())
            );
            if receipt.protocol_version == HISTORICAL_PROTOCOL_V5_VERSION {
                let error = validate_receipt_bytes(
                    &context,
                    &path,
                    &bytes,
                    ValidationOptions {
                        require_local: true,
                        check_environment: false,
                        check_engine: false,
                        source_version: None,
                        audit_id: None,
                    },
                )
                .expect_err("historical v5 must not enter the current validator");
                assert!(error.to_string().contains("current protocol v6"));
            }
        }
    }

    fn forward_recovery_probe(context: &Context) -> CompactReceipt {
        let manifest = tree_manifest(context.root(), FORWARD_RECOVERY_SECOND_AUDIT).unwrap();
        let mut receipt =
            historical_v5_receipt_at(context.root(), &manifest, FORWARD_RECOVERY_EPOCHS[1])
                .unwrap();
        // Production reaches the recovery helper only after validating the
        // new compact receipt. This probe supplies those already-established
        // fields while retaining the historical receipt's irrelevant payload.
        receipt.protocol_version = PROTOCOL_VERSION;
        receipt.protocol_status = PROTOCOL_STATUS.to_owned();
        receipt.audit_id = FORWARD_RECOVERY_AUDIT_ID.to_owned();
        receipt.candidate.parent_commit_sha =
            GitSha::parse(FORWARD_RECOVERY_SECOND_AUDIT, "recovery parent").unwrap();
        receipt
    }

    #[test]
    fn protocol_v6_forward_recovery_accepts_only_the_published_chain() {
        let context = Context::discover().unwrap();
        let receipt = forward_recovery_probe(&context);
        let anchor = forward_recovery_closed_ledger(context.root(), &receipt)
            .expect("published two-epoch recovery chain");
        assert_eq!(
            anchor.source_version,
            FORWARD_RECOVERY_CLOSED_SOURCE_VERSION
        );
        assert!(anchor.closure_record.0.is_some());
        assert_eq!(anchor.acceptance_gate.gate_a_status, "passed");

        let mut wrong_audit = receipt.clone();
        wrong_audit.audit_id = "FS-SAU-43".to_owned();
        assert!(forward_recovery_closed_ledger(context.root(), &wrong_audit).is_err());

        let mut wrong_parent = receipt;
        wrong_parent.candidate.parent_commit_sha =
            GitSha::parse(FORWARD_RECOVERY_FIRST_AUDIT, "wrong recovery parent").unwrap();
        assert!(forward_recovery_closed_ledger(context.root(), &wrong_parent).is_err());
    }

    #[test]
    fn historical_v5_receipts_are_strict_and_recovery_only() {
        let context = Context::discover().unwrap();
        let epoch = FORWARD_RECOVERY_EPOCHS[0];
        let manifest = tree_manifest(context.root(), epoch.audit).unwrap();
        let receipt = historical_v5_receipt_at(context.root(), &manifest, epoch)
            .expect("exact historical v5 receipt");
        assert_eq!(receipt.source_version, epoch.source_version);
        assert_eq!(receipt.audit_id, epoch.audit_id);

        let wrong_audit = ForwardRecoveryEpoch {
            audit_id: FORWARD_RECOVERY_SECOND_AUDIT_ID,
            ..epoch
        };
        let error = historical_v5_receipt_at(context.root(), &manifest, wrong_audit)
            .expect_err("historical audit relabelling must fail");
        assert!(error.to_string().contains("source/audit binding drifted"));

        let bytes = blob_at(context.root(), &manifest, epoch.receipt_path).unwrap();
        let error = validate_receipt_bytes(
            &context,
            Path::new(epoch.receipt_path),
            &bytes,
            ValidationOptions {
                require_local: false,
                check_environment: false,
                check_engine: false,
                source_version: Some(epoch.source_version),
                audit_id: Some(epoch.audit_id),
            },
        )
        .expect_err("v5 receipt must remain outside normal current validation");
        assert!(error.to_string().contains("current protocol v6"));
    }

    #[test]
    fn exact_legacy_v1_allowlist_is_preserved() {
        let context = Context::discover().unwrap();
        let body = serde_json::to_vec(&serde_json::json!({
            "candidate_commit_sha": LEGACY_CANDIDATE,
            "verified_at_utc": "2026-08-22T06:26:52Z",
            "commands": LEGACY_REQUIRED_COMMANDS,
            "result": "all-passed",
            "transcript_sha256": LEGACY_TRANSCRIPT_SHA256,
        }))
        .unwrap();
        let nominal = context.path("legacy-v1.json");
        let loaded = validate_receipt_bytes(
            &context,
            &nominal,
            &body,
            ValidationOptions {
                require_local: false,
                check_environment: false,
                check_engine: false,
                source_version: Some(LEGACY_SOURCE_VERSION),
                audit_id: Some(LEGACY_AUDIT_ID),
            },
        )
        .unwrap();
        assert_eq!(
            validation_success(&loaded),
            "verification receipt schema v1: ok"
        );

        let error = validate_receipt_bytes(
            &context,
            &nominal,
            &body,
            ValidationOptions {
                require_local: false,
                check_environment: false,
                check_engine: false,
                source_version: Some("wrong"),
                audit_id: Some(LEGACY_AUDIT_ID),
            },
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("outside the one exact v1 allowlist")
        );
    }

    #[test]
    fn legacy_extras_remain_v1_only_compatibility() {
        let context = Context::discover().unwrap();
        let legacy = serde_json::json!({
            "schema_version": 1,
            "candidate_commit_sha": LEGACY_CANDIDATE,
            "verified_at_utc": "2026-08-22T06:26:52Z",
            "commands": LEGACY_REQUIRED_COMMANDS,
            "result": "all-passed",
            "transcript_sha256": LEGACY_TRANSCRIPT_SHA256,
            "preserved_legacy_metadata": {"ignored_by_v1": true},
        });
        assert!(matches!(
            validate_receipt_bytes(
                &context,
                &context.path("legacy-v1.json"),
                &serde_json::to_vec(&legacy).unwrap(),
                ValidationOptions {
                    require_local: false,
                    check_environment: false,
                    check_engine: false,
                    source_version: Some(LEGACY_SOURCE_VERSION),
                    audit_id: Some(LEGACY_AUDIT_ID),
                },
            )
            .unwrap(),
            ValidatedReceipt::Legacy
        ));

        let receipt_path = fs::read_dir(context.path(RECEIPT_DIRECTORY))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().and_then(OsStr::to_str) == Some("json"))
            .unwrap();
        let mut compact: serde_json::Value =
            serde_json::from_slice(&fs::read(receipt_path).unwrap()).unwrap();
        compact
            .as_object_mut()
            .unwrap()
            .insert("preserved_legacy_metadata".to_owned(), true.into());
        let error = parse_typed::<CompactReceipt>(
            &serde_json::to_vec(&compact).unwrap(),
            "schema-v2 extra-key test",
        )
        .unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn ledger_closure_projection_is_root_exact() {
        let context = Context::discover().unwrap();
        let bytes = fs::read(context.path(LEDGER_PATH)).unwrap();
        let projection = ledger_closure_projection(&bytes).unwrap();
        assert!(!projection.source_version.is_empty());

        let mut extra_root: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        extra_root
            .as_object_mut()
            .unwrap()
            .insert("unexpected_receipt_root".to_owned(), true.into());
        let error = ledger_closure_projection(&serde_json::to_vec(&extra_root).unwrap())
            .err()
            .expect("unknown ledger root key must fail");
        assert!(error.to_string().contains("unexpected_receipt_root"));

        let mut missing_root: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        missing_root.as_object_mut().unwrap().remove("axes");
        let error = ledger_closure_projection(&serde_json::to_vec(&missing_root).unwrap())
            .err()
            .expect("missing ledger root key must fail");
        assert!(error.to_string().contains("missing field `axes`"));

        let mut extra_gate: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        extra_gate["acceptance_gate"]
            .as_object_mut()
            .unwrap()
            .insert("unexpected_gate_field".to_owned(), true.into());
        let error = ledger_closure_projection(&serde_json::to_vec(&extra_gate).unwrap())
            .err()
            .expect("unknown acceptance-gate key must fail");
        assert!(error.to_string().contains("unexpected_gate_field"));
    }

    #[test]
    fn native_inputs_are_verifier_classified() {
        for path in [
            "Cargo.toml",
            "Cargo.lock",
            "src/main.rs",
            "src/checks/temporal.rs",
            "verification-shard-runner.sh",
            "new-book-plans/verification_lock_client.py",
        ] {
            assert_eq!(classify_path(path), ManifestClass::VerifierInput, "{path}");
        }
        assert_eq!(
            classify_path("new-book-plans/testdata/example.json"),
            ManifestClass::Fixture
        );
        assert_eq!(classify_path("TODO.md"), ManifestClass::Administrative);
    }

    #[test]
    fn scanners_preserve_tracker_and_reference_contracts() {
        let body = "# TODO\n\n- [ ] first\n  body\n\n- [ ] second\n  tail\n";
        let blocks = unchecked_todo_blocks(body);
        assert_eq!(&body[blocks[0].0..blocks[0].1], "- [ ] first\n  body\n\n");
        assert_eq!(&body[blocks[1].0..blocks[1].1], "- [ ] second\n  tail\n");

        let json = br#"{"z":["no",{"ref":"TODO.md::first"}],"a":"x"}"#;
        reject_duplicate_keys(json, "test").unwrap();
        let mut strings = Vec::new();
        walk_json_strings(json, "", &mut strings).unwrap();
        assert_eq!(
            strings,
            [
                ("/a".to_owned(), "x".to_owned()),
                ("/z/0".to_owned(), "no".to_owned()),
                ("/z/1/ref".to_owned(), "TODO.md::first".to_owned()),
            ]
        );
    }

    #[test]
    fn typed_boundaries_reject_duplicates_unknown_keys_and_bad_time() {
        assert!(reject_duplicate_keys(br#"{"x":{"a":1,"a":2}}"#, "test").is_err());
        assert!(!valid_utc_seconds("2026-02-29T00:00:00Z"));
        assert!(valid_utc_seconds("2028-02-29T23:59:59Z"));

        let row = br#"{
            "id":"FS-SAU-1-PENDING","title":"audit pending","source_version":"v",
            "scope_sha256":"1111111111111111111111111111111111111111111111111111111111111111",
            "protocol_sha256":"2222222222222222222222222222222222222222222222222222222222222222",
            "executed_at_utc":"2026-08-27T00:00:00Z","method":"m",
            "criterion_coverage":[],"control_refs":[],"commands":["x"],
            "finding_refs":[],"result":"pending","policy_basis":"p",
            "evidence_ceiling":"e","unexpected":true
        }"#;
        assert!(parse_typed::<PendingAudit>(row, "test").is_err());
    }

    #[test]
    fn pure_protocol_self_test_is_exact() {
        assert_eq!(
            self_test().unwrap(),
            "verification-receipt protocol self-test: PASS"
        );
    }

    #[test]
    fn compile_time_nibli_provenance_matches_current_checkout() {
        let engine = engine_identity().unwrap();
        assert_eq!(
            engine.source_commit_sha.0.as_ref().map(GitSha::as_str),
            Some(env!("RIGHTS_VERIFY_COMPILED_NIBLI_COMMIT"))
        );
        assert_eq!(
            nibli_dependency_input_digest().unwrap(),
            Sha256(env!("RIGHTS_VERIFY_COMPILED_NIBLI_INPUTS_SHA256").to_owned())
        );
    }

    #[test]
    fn executable_identity_hashes_the_mapped_image() {
        let identity = mapped_executable_identity().unwrap();
        assert_eq!(
            identity.sha256,
            Sha256::of(fs::read("/proc/self/exe").unwrap())
        );
        assert_eq!(
            identity.size,
            fs::metadata("/proc/self/exe").unwrap().len() as usize
        );
    }

    #[test]
    fn native_emission_retains_and_revalidates_exact_local_evidence() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        test_git(root, &["init", "-q"]);
        test_git(root, &["config", "user.name", "Receipt Tests"]);
        test_git(
            root,
            &["config", "user.email", "receipt-tests@example.invalid"],
        );
        fs::create_dir_all(root.join("new-book-plans")).unwrap();
        fs::write(root.join("verify.sh"), "#!/bin/sh\nexit 0\n").unwrap();
        fs::write(
            root.join(PROTOCOL_PATH),
            format!("Status: {PROTOCOL_STATUS}\n"),
        )
        .unwrap();
        fs::write(
            root.join(TODO_PATH),
            "# TODO\n\n- [ ] Temporary first item.\n  Delete this block.\n\n- [ ] Second retained item.\n  Keep this block.\n",
        )
        .unwrap();
        for path in AUDIT_GENERATED_PATHS {
            fs::write(root.join(path), "base projection\n").unwrap();
        }
        let mut prior_ledger = serde_json::json!({
            "source_version": "base",
            "scope_audits": [],
            "closure_record": {
                "gate": "gate-a",
                "permitted_claim": "prior structural claim",
                "candidate_commit_sha": "0000000000000000000000000000000000000000",
                "source_version": "base",
                "scope_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                "envelope_ref": "FS-ENV-01",
                "audit_cutoff_at_utc": "2026-08-20T00:00:00Z",
                "scope_audit_ref": "FS-SAU-42",
                "assurance_record_refs": ["FS-ASR-01"],
                "residual_refs": ["FS-DEF-01"],
                "claim_limitations": [{
                    "defect_ref": "FS-DEF-01",
                    "affected_claim_ref": "FS-CLM-01",
                    "public_claim_restriction": "No operational claim."
                }],
                "verification_receipt_ref": "new-book-plans/verification-receipts/prior.json",
                "closure_policy_ref": "protocol::Policy"
            },
            "acceptance_gate": {
                "verdict": "prior structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "passed"
            },
            "owner_ref": "TODO.md::Second retained item"
        });
        complete_test_ledger_root(&mut prior_ledger);
        fs::write(root.join(LEDGER_PATH), pretty(&prior_ledger).unwrap()).unwrap();
        test_git(root, &["add", "."]);
        test_git(root, &["commit", "-qm", "base"]);

        let pending = PendingAudit {
            id: "FS-SAU-43-PENDING".to_owned(),
            title: "Receipt-aware repository audit pending".to_owned(),
            source_version: "fs-ledger-test-native-v1".to_owned(),
            scope_sha256: Sha256("1".repeat(64)),
            protocol_sha256: Sha256("2".repeat(64)),
            executed_at_utc: UtcTimestamp("2026-08-27T00:00:00Z".to_owned()),
            method: "repository adversarial audit".to_owned(),
            criterion_coverage: vec!["semantic scope".to_owned()],
            control_refs: vec!["CTRL-01".to_owned()],
            commands: vec!["rights-verify --native-self-test".to_owned()],
            finding_refs: vec!["FS-DEF-01".to_owned()],
            result: "pending".to_owned(),
            policy_basis: "protocol::Policy".to_owned(),
            evidence_ceiling: "Repository structure only.".to_owned(),
        };
        let mut ledger = serde_json::json!({
            "source_version": "fs-ledger-test-native-v1",
            "scope_audits": [pending.clone()],
            "closure_record": null,
            "acceptance_gate": {
                "verdict": "pending structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "not-passed"
            },
            "owner_ref": "TODO.md::Second retained item"
        });
        complete_test_ledger_root(&mut ledger);
        fs::write(root.join(LEDGER_PATH), pretty(&ledger).unwrap()).unwrap();
        test_git(root, &["add", LEDGER_PATH]);

        let context = Context::from_test_root(root.to_path_buf());
        let mut runtime = receipt_self_test_runtime().unwrap();
        runtime.compiled_verifier_inputs_sha256 =
            verifier_build_input_digest(root, &index_manifest(root).unwrap()).unwrap();
        let mut output = Vec::new();
        let emitted = emit_receipt_with_runtime(
            root,
            Path::new(RECEIPT_DIRECTORY),
            &mut output,
            |writer| {
                writeln!(writer, "native verifier PASS")?;
                Ok(())
            },
            |_| Ok(runtime.clone()),
        )
        .unwrap();
        assert!(emitted.path.is_file());
        assert!(
            String::from_utf8(output)
                .unwrap()
                .starts_with("native verifier PASS\n")
        );
        let loaded = load_and_validate_with_runtime(
            root,
            &emitted.path,
            ValidationOptions::default(),
            &runtime,
        )
        .unwrap();
        assert_eq!(loaded.v2().unwrap().receipt_id(), emitted.receipt_id);
        assert_eq!(
            loaded.v2().unwrap().engine.binary_basename,
            runtime.engine.binary_basename
        );

        let compact = loaded.v2().unwrap().clone();
        let relative_receipt = emitted
            .path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        test_git(root, &["commit", "-qm", "candidate"]);
        let candidate_sha =
            GitSha::parse(git_text(root, ["rev-parse", "HEAD"]).unwrap(), "candidate").unwrap();

        let passing = passing_audit(&pending, &compact, &relative_receipt).unwrap();
        let mut audit_ledger = serde_json::json!({
            "source_version": "fs-ledger-test-native-v1",
            "scope_audits": [pending.clone(), passing.clone()],
            "closure_record": null,
            "acceptance_gate": {
                "verdict": "pending structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "not-passed"
            },
            "owner_ref": "TODO.md::Second retained item"
        });
        complete_test_ledger_root(&mut audit_ledger);
        fs::write(root.join(LEDGER_PATH), pretty(&audit_ledger).unwrap()).unwrap();
        for path in AUDIT_GENERATED_PATHS {
            fs::write(root.join(path), "audit projection\n").unwrap();
        }
        test_git(
            root,
            &[
                "add",
                LEDGER_PATH,
                AUDIT_GENERATED_PATHS[0],
                AUDIT_GENERATED_PATHS[1],
                &relative_receipt,
            ],
        );
        let mut quick_output = Vec::new();
        run_commit_gate_with_runtime(
            root,
            &emitted.path,
            Transition::Audit,
            &mut quick_output,
            |writer| {
                writeln!(writer, "native quick PASS")?;
                Ok(())
            },
            |_| Ok(runtime.clone()),
        )
        .unwrap();
        assert_eq!(quick_output, b"native quick PASS\n");
        test_git(root, &["commit", "-qm", "audit"]);
        let audit_sha =
            GitSha::parse(git_text(root, ["rev-parse", "HEAD"]).unwrap(), "audit").unwrap();
        assert_eq!(
            validate_recorded_transition(
                &context,
                &compact,
                audit_sha.as_str(),
                Transition::Audit,
                &relative_receipt,
            )
            .unwrap(),
            candidate_sha
        );
        assert_eq!(
            resolve_candidate_commit(&context, &compact, "HEAD").unwrap(),
            candidate_sha
        );

        let closure = ClosureRecord {
            gate: "gate-a".to_owned(),
            permitted_claim: "prior structural claim".to_owned(),
            candidate_commit_sha: audit_sha.clone(),
            source_version: compact.source_version.clone(),
            scope_sha256: passing.scope_sha256.clone(),
            envelope_ref: "FS-ENV-01".to_owned(),
            audit_cutoff_at_utc: passing.executed_at_utc.clone(),
            scope_audit_ref: passing.id.clone(),
            assurance_record_refs: vec!["FS-ASR-01".to_owned()],
            residual_refs: vec!["FS-DEF-01".to_owned()],
            claim_limitations: vec![ClaimLimitation {
                defect_ref: "FS-DEF-01".to_owned(),
                affected_claim_ref: "FS-CLM-01".to_owned(),
                public_claim_restriction: "No operational claim.".to_owned(),
            }],
            verification_receipt_ref: relative_receipt.clone(),
            closure_policy_ref: passing.policy_basis.clone(),
        };
        let mut closure_ledger = serde_json::json!({
            "source_version": "fs-ledger-test-native-v1",
            "scope_audits": [pending.clone(), passing.clone()],
            "closure_record": closure,
            "acceptance_gate": {
                "verdict": "prior structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "passed"
            },
            "owner_ref": "TODO.md::Second retained item"
        });
        complete_test_ledger_root(&mut closure_ledger);
        let mut wrong_closure = closure_ledger.clone();
        wrong_closure["closure_record"]["candidate_commit_sha"] =
            serde_json::Value::String("f".repeat(40));
        fs::write(root.join(LEDGER_PATH), pretty(&wrong_closure).unwrap()).unwrap();
        for path in AUDIT_GENERATED_PATHS {
            fs::write(root.join(path), "closure projection\n").unwrap();
        }
        test_git(
            root,
            &[
                "add",
                LEDGER_PATH,
                AUDIT_GENERATED_PATHS[0],
                AUDIT_GENERATED_PATHS[1],
            ],
        );
        let loaded = load_and_validate_with_runtime(
            root,
            &emitted.path,
            ValidationOptions::default(),
            &runtime,
        )
        .unwrap();
        assert!(
            validate_commit_gate_for_receipt(
                root,
                &emitted.path,
                Transition::Closure,
                loaded.v2().unwrap(),
            )
            .is_err(),
            "wrong closure ancestry must fail"
        );
        fs::write(root.join(LEDGER_PATH), pretty(&closure_ledger).unwrap()).unwrap();
        test_git(root, &["add", LEDGER_PATH]);
        validate_commit_gate_for_receipt(
            root,
            &emitted.path,
            Transition::Closure,
            loaded.v2().unwrap(),
        )
        .unwrap();
        test_git(root, &["commit", "-qm", "closure"]);
        let closure_sha =
            GitSha::parse(git_text(root, ["rev-parse", "HEAD"]).unwrap(), "closure").unwrap();
        assert_eq!(
            validate_recorded_transition(
                &context,
                &compact,
                closure_sha.as_str(),
                Transition::Closure,
                &relative_receipt,
            )
            .unwrap(),
            audit_sha
        );

        fs::write(
            root.join(TODO_PATH),
            "# TODO\n\n- [ ] Second retained item.\n  Keep this block.\n",
        )
        .unwrap();
        test_git(root, &["add", TODO_PATH]);
        let tracker_tree = validate_commit_gate_for_receipt(
            root,
            &emitted.path,
            Transition::Tracker,
            loaded.v2().unwrap(),
        )
        .unwrap();
        assert_eq!(
            gate_success(Transition::Tracker, &tracker_tree),
            format!(
                "verification commit gate tracker: ok ({})",
                tracker_tree.as_str()
            )
        );
    }
}
