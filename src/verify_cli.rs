// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fmt;
use std::path::PathBuf;

use clap::{ArgGroup, Parser};

#[derive(Debug, Parser)]
#[command(
    name = "rights-verify",
    about = "Run the book's Nibli pins and contradiction checks"
)]
#[command(group(ArgGroup::new("mode").args(["only", "list"]).multiple(false)))]
pub(crate) struct Args {
    /// Run one pin file against the base selected by its suite.
    #[arg(long, value_name = "PINFILE")]
    pub(crate) only: Option<PathBuf>,
    /// List the substantive test cases without loading the engine.
    #[arg(long)]
    pub(crate) list: bool,
}

#[derive(Debug)]
pub(crate) struct Error {
    message: String,
    code: u8,
}

impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self::with_exit_code(message, 1)
    }

    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::with_exit_code(message, 2)
    }

    pub(crate) fn with_exit_code(message: impl Into<String>, code: u8) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }

    pub(crate) fn exit_code(&self) -> u8 {
        self.code
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::usage(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::usage(error.to_string())
    }
}

pub(crate) fn run() -> Result<(), Error> {
    let retired = [
        "--quick",
        "--table",
        "--fingerprints",
        "--refresh",
        "--emit-receipt",
        "--commit-gate",
        "--transition",
        "--wait-for-lock",
    ];
    for argument in std::env::args().skip(1) {
        if retired.contains(&argument.split('=').next().unwrap_or_default()) {
            return Err(Error::usage(format!(
                "{argument} was retired: ./verify.sh now runs all substantive pins and contradiction checks; use ./generate.sh for authoring"
            )));
        }
    }
    crate::execution::run(Args::parse())
}
