// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, ValueEnum};

use crate::checks;
use crate::context::Context;
use crate::lock::VerificationLock;
use crate::receipt::{self, Transition};
use crate::suite::RunMode;

#[derive(Debug, Parser)]
#[command(name = "rights-verify", disable_help_subcommand = true)]
#[command(group(
    ArgGroup::new("mode")
        .args([
            "quick",
            "only",
            "table",
            "fingerprints",
            "refresh",
            "emit_receipt",
            "commit_gate",
        ])
        .multiple(false)
))]
struct Args {
    #[arg(long)]
    quick: bool,

    #[arg(long, value_name = "PINFILE")]
    only: Option<PathBuf>,

    #[arg(long)]
    table: bool,

    #[arg(long, value_enum, value_name = "ARTIFACT")]
    fingerprints: Option<FingerprintArtifact>,

    #[arg(long, value_enum, value_name = "ARTIFACT")]
    refresh: Option<RefreshArtifact>,

    #[arg(long, value_name = "PATH")]
    emit_receipt: Option<PathBuf>,

    #[arg(long, value_name = "RECEIPT")]
    commit_gate: Option<PathBuf>,

    #[arg(long, requires = "commit_gate", value_parser = ["audit", "closure", "tracker"])]
    transition: Option<String>,

    #[arg(long, value_name = "SECONDS")]
    wait_for_lock: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RefreshArtifact {
    #[value(name = "spine")]
    Spine,

    #[value(name = "assertion-surface", alias = "assertion")]
    AssertionSurface,

    #[value(name = "record-integrity-assurance", alias = "assurance")]
    RecordIntegrityAssurance,

    #[value(name = "record-integrity-red-team", alias = "red-team")]
    RecordIntegrityRedTeam,

    #[value(name = "amendment-semantics", alias = "amendment")]
    AmendmentSemantics,

    #[value(name = "placement-exhaustiveness", alias = "placement")]
    PlacementExhaustiveness,

    #[value(name = "temporal-assurance", alias = "temporal")]
    TemporalAssurance,

    #[value(name = "state-form", alias = "state")]
    StateForm,

    #[value(name = "obligations", alias = "obligation")]
    Obligations,

    #[value(name = "reader-evidence", alias = "reader")]
    ReaderEvidence,

    #[value(name = "full-society-ledger", alias = "ledger")]
    FullSocietyLedger,

    #[value(name = "constitutional-closure", alias = "closure")]
    ConstitutionalClosure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum FingerprintArtifact {
    #[value(name = "assertion-surface", alias = "assertion")]
    AssertionSurface,

    #[value(name = "amendment-semantics", alias = "amendment")]
    AmendmentSemantics,

    #[value(name = "placement-exhaustiveness", alias = "placement")]
    PlacementExhaustiveness,

    #[value(name = "temporal-assurance", alias = "temporal")]
    TemporalAssurance,

    #[value(name = "state-form", alias = "state")]
    StateForm,

    #[value(name = "obligations", alias = "obligation")]
    Obligations,

    #[value(name = "full-society-ledger", alias = "ledger")]
    FullSocietyLedger,
}

type ArtifactHandler = fn(&Context) -> Result<String, Error>;

struct RefreshHandlers {
    spine: ArtifactHandler,
    assertion_surface: ArtifactHandler,
    record_integrity_assurance: ArtifactHandler,
    record_integrity_red_team: ArtifactHandler,
    amendment_semantics: ArtifactHandler,
    placement_exhaustiveness: ArtifactHandler,
    temporal_assurance: ArtifactHandler,
    state_form: ArtifactHandler,
    obligations: ArtifactHandler,
    reader_evidence: ArtifactHandler,
    full_society_ledger: ArtifactHandler,
    constitutional_closure: ArtifactHandler,
}

struct FingerprintHandlers {
    assertion_surface: ArtifactHandler,
    amendment_semantics: ArtifactHandler,
    placement_exhaustiveness: ArtifactHandler,
    temporal_assurance: ArtifactHandler,
    state_form: ArtifactHandler,
    obligations: ArtifactHandler,
    full_society_ledger: ArtifactHandler,
}

const REFRESH_HANDLERS: RefreshHandlers = RefreshHandlers {
    spine: refresh_spine,
    assertion_surface: refresh_assertion_surface,
    record_integrity_assurance: refresh_record_integrity_assurance,
    record_integrity_red_team: refresh_record_integrity_red_team,
    amendment_semantics: refresh_amendment_semantics,
    placement_exhaustiveness: refresh_placement_exhaustiveness,
    temporal_assurance: refresh_temporal_assurance,
    state_form: refresh_state_form,
    obligations: refresh_obligations,
    reader_evidence: refresh_reader_evidence,
    full_society_ledger: refresh_full_society_ledger,
    constitutional_closure: refresh_constitutional_closure,
};

const FINGERPRINT_HANDLERS: FingerprintHandlers = FingerprintHandlers {
    assertion_surface: fingerprint_assertion_surface,
    amendment_semantics: fingerprint_amendment_semantics,
    placement_exhaustiveness: fingerprint_placement_exhaustiveness,
    temporal_assurance: fingerprint_temporal_assurance,
    state_form: fingerprint_state_form,
    obligations: fingerprint_obligations,
    full_society_ledger: fingerprint_full_society_ledger,
};

fn refresh_artifact_with(
    context: &Context,
    artifact: RefreshArtifact,
    handlers: &RefreshHandlers,
) -> Result<String, Error> {
    let handler = match artifact {
        RefreshArtifact::Spine => handlers.spine,
        RefreshArtifact::AssertionSurface => handlers.assertion_surface,
        RefreshArtifact::RecordIntegrityAssurance => handlers.record_integrity_assurance,
        RefreshArtifact::RecordIntegrityRedTeam => handlers.record_integrity_red_team,
        RefreshArtifact::AmendmentSemantics => handlers.amendment_semantics,
        RefreshArtifact::PlacementExhaustiveness => handlers.placement_exhaustiveness,
        RefreshArtifact::TemporalAssurance => handlers.temporal_assurance,
        RefreshArtifact::StateForm => handlers.state_form,
        RefreshArtifact::Obligations => handlers.obligations,
        RefreshArtifact::ReaderEvidence => handlers.reader_evidence,
        RefreshArtifact::FullSocietyLedger => handlers.full_society_ledger,
        RefreshArtifact::ConstitutionalClosure => handlers.constitutional_closure,
    };
    handler(context)
}

fn refresh_artifact(context: &Context, artifact: RefreshArtifact) -> Result<String, Error> {
    refresh_artifact_with(context, artifact, &REFRESH_HANDLERS)
}

fn fingerprint_artifact_with(
    context: &Context,
    artifact: FingerprintArtifact,
    handlers: &FingerprintHandlers,
) -> Result<String, Error> {
    let handler = match artifact {
        FingerprintArtifact::AssertionSurface => handlers.assertion_surface,
        FingerprintArtifact::AmendmentSemantics => handlers.amendment_semantics,
        FingerprintArtifact::PlacementExhaustiveness => handlers.placement_exhaustiveness,
        FingerprintArtifact::TemporalAssurance => handlers.temporal_assurance,
        FingerprintArtifact::StateForm => handlers.state_form,
        FingerprintArtifact::Obligations => handlers.obligations,
        FingerprintArtifact::FullSocietyLedger => handlers.full_society_ledger,
    };
    handler(context)
}

fn fingerprint_artifact(context: &Context, artifact: FingerprintArtifact) -> Result<String, Error> {
    fingerprint_artifact_with(context, artifact, &FINGERPRINT_HANDLERS)
}

fn refresh_spine(context: &Context) -> Result<String, Error> {
    checks::spine::run(
        context,
        Path::new("new-book-plans/constitution.nibli"),
        Path::new("new-book-plans/3-spine.md"),
        false,
    )
}

fn refresh_assertion_surface(context: &Context) -> Result<String, Error> {
    checks::assertion_surface::generate(context, None, None)
}

fn refresh_record_integrity_assurance(context: &Context) -> Result<String, Error> {
    checks::assurance::generate(context).map(|report| report.to_string())
}

fn refresh_record_integrity_red_team(context: &Context) -> Result<String, Error> {
    checks::red_team::generate(context, checks::red_team::InputSnapshot::default())
}

fn refresh_amendment_semantics(context: &Context) -> Result<String, Error> {
    checks::amendment::generate(context).map(|report| report.to_string())
}

fn refresh_placement_exhaustiveness(context: &Context) -> Result<String, Error> {
    checks::placement::generate(context).map(|report| report.to_string())
}

fn refresh_temporal_assurance(context: &Context) -> Result<String, Error> {
    checks::temporal::generate(context).map(|report| report.to_string())
}

fn refresh_state_form(context: &Context) -> Result<String, Error> {
    let snapshot = checks::state_form::load_snapshot(context)?;
    checks::state_form::write_artifacts(context, &snapshot).map(|messages| messages.join("\n"))
}

fn refresh_obligations(context: &Context) -> Result<String, Error> {
    let snapshot = checks::obligations::load_snapshot(context)?;
    checks::obligations::write_artifacts(context, &snapshot).map(|messages| messages.join("\n"))
}

fn refresh_reader_evidence(context: &Context) -> Result<String, Error> {
    checks::reader::generate(context, false, checks::reader::InputSnapshot::default())
}

fn refresh_full_society_ledger(context: &Context) -> Result<String, Error> {
    checks::ledger::run(context, checks::ledger::Mode::RefreshAndCheck)
        .map(|report| report.to_string())
}

fn refresh_constitutional_closure(context: &Context) -> Result<String, Error> {
    checks::ledger::closure::run(context, checks::ledger::closure::Mode::RefreshAndCheck)
        .map(|report| report.to_string())
}

fn fingerprint_assertion_surface(context: &Context) -> Result<String, Error> {
    checks::assertion_surface::fingerprints(context, None, None)
}

fn fingerprint_amendment_semantics(context: &Context) -> Result<String, Error> {
    checks::amendment::fingerprints(context).map(|report| report.to_string())
}

fn fingerprint_placement_exhaustiveness(context: &Context) -> Result<String, Error> {
    checks::placement::fingerprints(context).map(|report| report.0)
}

fn fingerprint_temporal_assurance(context: &Context) -> Result<String, Error> {
    checks::temporal::fingerprints(context).map(|report| report.to_string())
}

fn fingerprint_state_form(context: &Context) -> Result<String, Error> {
    let snapshot = checks::state_form::load_snapshot(context)?;
    checks::state_form::fingerprints(context, &snapshot)
}

fn fingerprint_obligations(context: &Context) -> Result<String, Error> {
    let snapshot = checks::obligations::load_snapshot(context)?;
    checks::obligations::fingerprints(context, &snapshot)
}

fn fingerprint_full_society_ledger(context: &Context) -> Result<String, Error> {
    checks::ledger::fingerprints(context)
}

fn write_output<W: std::io::Write>(mut writer: W, output: &str) -> Result<(), Error> {
    writer.write_all(output.as_bytes())?;
    if !output.ends_with('\n') {
        writer.write_all(b"\n")?;
    }
    Ok(())
}

pub(crate) fn run() -> Result<(), Error> {
    let args = Args::parse();
    let context = Context::discover()?;

    if args.table {
        return checks::claim_table::print(&context);
    }

    if let Some(artifact) = args.fingerprints {
        let output = fingerprint_artifact(&context, artifact)?;
        write_output(std::io::stdout().lock(), &output)?;
        return Ok(());
    }

    // Instrument the four gate-relevant modes with the observational run
    // recorder. Focused and refresh modes stay uninstrumented; observation
    // points elsewhere are inert without this initialisation.
    let run_label = if args.quick {
        Some(crate::diagnostics::RunLabel::Quick)
    } else if args.emit_receipt.is_some() {
        Some(crate::diagnostics::RunLabel::EmitReceipt)
    } else if args.commit_gate.is_some() {
        Some(crate::diagnostics::RunLabel::CommitGate)
    } else if args.only.is_none() && args.refresh.is_none() {
        Some(crate::diagnostics::RunLabel::Full)
    } else {
        None
    };
    if let Some(label) = run_label {
        crate::diagnostics::initialise(label, context.root());
    }

    // A lock-refused run deliberately records no diagnostics document: it ran
    // no phases, and replacing the previous run's document with an empty one
    // would destroy the estimate baseline. The refusal itself is already
    // reported by the lock's queued note and its exit-75 owner details.
    let lock_started = std::time::Instant::now();
    let _lock = VerificationLock::acquire(&context, "verify", args.wait_for_lock.unwrap_or(0.0))?;
    crate::diagnostics::note_lock_acquired(lock_started.elapsed());

    let heartbeat = crate::diagnostics::start_heartbeat();
    let result = run_selected_mode(&context, args);
    drop(heartbeat);
    crate::diagnostics::finish(result.is_err());
    result
}

fn run_selected_mode(context: &Context, args: Args) -> Result<(), Error> {
    if let Some(artifact) = args.refresh {
        let output = refresh_artifact(context, artifact)?;
        write_output(std::io::stdout().lock(), &output)?;
        return Ok(());
    }

    if let Some(output_directory) = args.emit_receipt {
        let mut output = std::io::stdout().lock();
        receipt::emit_receipt(context, &output_directory, &mut output, |writer| {
            crate::suite::run(context, RunMode::Full, writer)
        })?;
        return Ok(());
    }

    if let Some(receipt_path) = args.commit_gate {
        let transition = Transition::parse(
            args.transition
                .as_deref()
                .ok_or_else(|| Error::usage("--commit-gate requires --transition"))?,
        )?;
        let mut output = std::io::stdout().lock();
        let tree =
            receipt::run_commit_gate(context, &receipt_path, transition, &mut output, |writer| {
                crate::suite::run(context, RunMode::Quick, writer)
            })?;
        drop(output);
        println!("{}", receipt::gate_success(transition, &tree));
        return Ok(());
    }

    if let Some(relative) = args.only {
        return run_only(context, &relative);
    }

    crate::suite::run(
        context,
        if args.quick {
            RunMode::Quick
        } else {
            RunMode::Full
        },
        std::io::stdout().lock(),
    )
}

fn run_only(context: &Context, relative: &std::path::Path) -> Result<(), Error> {
    let absolute = context.path(relative);
    if !absolute.is_file() {
        return Err(Error::usage(format!(
            "no such pin file: {}",
            relative.display()
        )));
    }
    let output = checks::pins::run_only(context, relative)?;
    print!("{}", output.stdout);
    eprint!("{}", output.stderr);
    println!(
        "\n\x1b[33mpartial\x1b[0m one pin file against one knowledge base; run ./verify.sh before committing."
    );
    if output.exit_code == 0 {
        Ok(())
    } else {
        Err(Error::with_exit_code(
            format!("focused pin suite exited {}", output.exit_code),
            u8::try_from(output.exit_code).unwrap_or(1),
        ))
    }
}

#[derive(Debug)]
pub(crate) struct Error {
    message: String,
    exit_code: u8,
}

impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 1,
        }
    }

    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: 2,
        }
    }

    pub(crate) fn with_exit_code(message: impl Into<String>, exit_code: u8) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }

    pub(crate) fn exit_code(&self) -> u8 {
        self.exit_code
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removed_hidden_mode_is_rejected() {
        assert!(Args::try_parse_from(["rights-verify", "--native-self-test"]).is_err());
    }

    #[test]
    fn public_modes_remain_mutually_exclusive() {
        for arguments in [
            vec!["rights-verify", "--quick", "--table"],
            vec!["rights-verify", "--quick", "--only", "pins.nibli"],
            vec!["rights-verify", "--emit-receipt", "receipts", "--table"],
            vec![
                "rights-verify",
                "--fingerprints",
                "full-society-ledger",
                "--quick",
            ],
            vec![
                "rights-verify",
                "--fingerprints",
                "ledger",
                "--refresh",
                "full-society-ledger",
            ],
            vec![
                "rights-verify",
                "--refresh",
                "full-society-ledger",
                "--quick",
            ],
            vec![
                "rights-verify",
                "--refresh",
                "constitutional-closure",
                "--only",
                "pins.nibli",
            ],
            vec![
                "rights-verify",
                "--commit-gate",
                "receipt.json",
                "--transition",
                "audit",
                "--quick",
            ],
        ] {
            assert!(Args::try_parse_from(arguments).is_err());
        }
    }

    #[test]
    fn refresh_artifacts_have_stable_public_names_and_aliases() {
        for (name, expected) in [
            ("spine", RefreshArtifact::Spine),
            ("assertion-surface", RefreshArtifact::AssertionSurface),
            ("assertion", RefreshArtifact::AssertionSurface),
            (
                "record-integrity-assurance",
                RefreshArtifact::RecordIntegrityAssurance,
            ),
            ("assurance", RefreshArtifact::RecordIntegrityAssurance),
            (
                "record-integrity-red-team",
                RefreshArtifact::RecordIntegrityRedTeam,
            ),
            ("red-team", RefreshArtifact::RecordIntegrityRedTeam),
            ("amendment-semantics", RefreshArtifact::AmendmentSemantics),
            ("amendment", RefreshArtifact::AmendmentSemantics),
            (
                "placement-exhaustiveness",
                RefreshArtifact::PlacementExhaustiveness,
            ),
            ("placement", RefreshArtifact::PlacementExhaustiveness),
            ("temporal-assurance", RefreshArtifact::TemporalAssurance),
            ("temporal", RefreshArtifact::TemporalAssurance),
            ("state-form", RefreshArtifact::StateForm),
            ("state", RefreshArtifact::StateForm),
            ("obligations", RefreshArtifact::Obligations),
            ("obligation", RefreshArtifact::Obligations),
            ("reader-evidence", RefreshArtifact::ReaderEvidence),
            ("reader", RefreshArtifact::ReaderEvidence),
            ("full-society-ledger", RefreshArtifact::FullSocietyLedger),
            ("ledger", RefreshArtifact::FullSocietyLedger),
            (
                "constitutional-closure",
                RefreshArtifact::ConstitutionalClosure,
            ),
            ("closure", RefreshArtifact::ConstitutionalClosure),
        ] {
            let arguments = Args::try_parse_from(["rights-verify", "--refresh", name]).unwrap();
            assert_eq!(arguments.refresh, Some(expected));
        }
        assert!(Args::try_parse_from(["rights-verify", "--refresh", "unknown-artifact"]).is_err());
    }

    #[test]
    fn fingerprint_artifacts_have_stable_public_names_and_aliases() {
        for (name, expected) in [
            ("assertion-surface", FingerprintArtifact::AssertionSurface),
            ("assertion", FingerprintArtifact::AssertionSurface),
            (
                "amendment-semantics",
                FingerprintArtifact::AmendmentSemantics,
            ),
            ("amendment", FingerprintArtifact::AmendmentSemantics),
            (
                "placement-exhaustiveness",
                FingerprintArtifact::PlacementExhaustiveness,
            ),
            ("placement", FingerprintArtifact::PlacementExhaustiveness),
            ("temporal-assurance", FingerprintArtifact::TemporalAssurance),
            ("temporal", FingerprintArtifact::TemporalAssurance),
            ("state-form", FingerprintArtifact::StateForm),
            ("state", FingerprintArtifact::StateForm),
            ("obligations", FingerprintArtifact::Obligations),
            ("obligation", FingerprintArtifact::Obligations),
            (
                "full-society-ledger",
                FingerprintArtifact::FullSocietyLedger,
            ),
            ("ledger", FingerprintArtifact::FullSocietyLedger),
        ] {
            let arguments =
                Args::try_parse_from(["rights-verify", "--fingerprints", name]).unwrap();
            assert_eq!(arguments.fingerprints, Some(expected));
        }
        assert!(
            Args::try_parse_from(["rights-verify", "--fingerprints", "constitutional-closure"])
                .is_err()
        );
    }

    #[test]
    fn artifact_dispatch_is_typed_exhaustive_and_preserves_output() {
        let context = Context::discover().expect("repository context");
        let refresh_handlers = RefreshHandlers {
            spine: |_| Ok("spine".to_owned()),
            assertion_surface: |_| Ok("assertion-surface".to_owned()),
            record_integrity_assurance: |_| Ok("record-integrity-assurance".to_owned()),
            record_integrity_red_team: |_| Ok("record-integrity-red-team".to_owned()),
            amendment_semantics: |_| Ok("amendment-semantics".to_owned()),
            placement_exhaustiveness: |_| Ok("placement-exhaustiveness".to_owned()),
            temporal_assurance: |_| Ok("temporal-assurance".to_owned()),
            state_form: |_| Ok("state-form".to_owned()),
            obligations: |_| Ok("obligations".to_owned()),
            reader_evidence: |_| Ok("reader-evidence".to_owned()),
            full_society_ledger: |_| Ok("full-society-ledger".to_owned()),
            constitutional_closure: |_| Ok("constitutional-closure".to_owned()),
        };
        for (artifact, expected) in [
            (RefreshArtifact::Spine, "spine"),
            (RefreshArtifact::AssertionSurface, "assertion-surface"),
            (
                RefreshArtifact::RecordIntegrityAssurance,
                "record-integrity-assurance",
            ),
            (
                RefreshArtifact::RecordIntegrityRedTeam,
                "record-integrity-red-team",
            ),
            (RefreshArtifact::AmendmentSemantics, "amendment-semantics"),
            (
                RefreshArtifact::PlacementExhaustiveness,
                "placement-exhaustiveness",
            ),
            (RefreshArtifact::TemporalAssurance, "temporal-assurance"),
            (RefreshArtifact::StateForm, "state-form"),
            (RefreshArtifact::Obligations, "obligations"),
            (RefreshArtifact::ReaderEvidence, "reader-evidence"),
            (RefreshArtifact::FullSocietyLedger, "full-society-ledger"),
            (
                RefreshArtifact::ConstitutionalClosure,
                "constitutional-closure",
            ),
        ] {
            assert_eq!(
                refresh_artifact_with(&context, artifact, &refresh_handlers)
                    .expect("refresh dispatch"),
                expected
            );
        }

        let fingerprint_handlers = FingerprintHandlers {
            assertion_surface: |_| Ok("assertion-surface".to_owned()),
            amendment_semantics: |_| Ok("amendment-semantics".to_owned()),
            placement_exhaustiveness: |_| Ok("placement-exhaustiveness".to_owned()),
            temporal_assurance: |_| Ok("temporal-assurance".to_owned()),
            state_form: |_| Ok("state-form".to_owned()),
            obligations: |_| Ok("obligations".to_owned()),
            full_society_ledger: |_| Ok("full-society-ledger".to_owned()),
        };
        for (artifact, expected) in [
            (FingerprintArtifact::AssertionSurface, "assertion-surface"),
            (
                FingerprintArtifact::AmendmentSemantics,
                "amendment-semantics",
            ),
            (
                FingerprintArtifact::PlacementExhaustiveness,
                "placement-exhaustiveness",
            ),
            (FingerprintArtifact::TemporalAssurance, "temporal-assurance"),
            (FingerprintArtifact::StateForm, "state-form"),
            (FingerprintArtifact::Obligations, "obligations"),
            (
                FingerprintArtifact::FullSocietyLedger,
                "full-society-ledger",
            ),
        ] {
            assert_eq!(
                fingerprint_artifact_with(&context, artifact, &fingerprint_handlers)
                    .expect("fingerprint dispatch"),
                expected
            );
        }

        let mut stdout = Vec::new();
        write_output(&mut stdout, "fingerprints\n").expect("terminated output");
        write_output(&mut stdout, "refresh").expect("unterminated output");
        assert_eq!(stdout, b"fingerprints\nrefresh\n");
    }
}
