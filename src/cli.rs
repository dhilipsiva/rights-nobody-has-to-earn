// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

use clap::{ArgGroup, Parser};

use crate::checks;
use crate::context::Context;
use crate::lock::VerificationLock;
use crate::receipt::{self, Transition};
use crate::suite::RunMode;

#[derive(Debug, Parser)]
#[command(name = "rights-verify", disable_help_subcommand = true)]
#[command(group(
    ArgGroup::new("mode")
        .args(["quick", "only", "table", "emit_receipt", "commit_gate"])
        .multiple(false)
))]
struct Args {
    #[arg(long)]
    quick: bool,

    #[arg(long, value_name = "PINFILE")]
    only: Option<PathBuf>,

    #[arg(long)]
    table: bool,

    #[arg(long, value_name = "PATH")]
    emit_receipt: Option<PathBuf>,

    #[arg(long, value_name = "RECEIPT")]
    commit_gate: Option<PathBuf>,

    #[arg(long, requires = "commit_gate", value_parser = ["audit", "closure", "tracker"])]
    transition: Option<String>,

    #[arg(long, value_name = "SECONDS")]
    wait_for_lock: Option<f64>,
}

pub(crate) fn run() -> Result<(), Error> {
    let args = Args::parse();
    let context = Context::discover()?;

    if args.table {
        return checks::claim_table::print(&context);
    }

    let _lock = VerificationLock::acquire(&context, "verify", args.wait_for_lock.unwrap_or(0.0))?;

    if let Some(output_directory) = args.emit_receipt {
        let mut output = std::io::stdout().lock();
        receipt::emit_receipt(&context, &output_directory, &mut output, |writer| {
            crate::suite::run(&context, RunMode::Full, writer)
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
            receipt::run_commit_gate(&context, &receipt_path, transition, &mut output, |writer| {
                crate::suite::run(&context, RunMode::Quick, writer)
            })?;
        drop(output);
        println!("{}", receipt::gate_success(transition, &tree));
        return Ok(());
    }

    if let Some(relative) = args.only {
        return run_only(&context, &relative);
    }

    crate::suite::run(
        &context,
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
}
