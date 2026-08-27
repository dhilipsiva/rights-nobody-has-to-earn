// SPDX-License-Identifier: MIT OR Apache-2.0

// The binary retains regeneration, fingerprint, and historical-parity seams
// beside the one production verification path. They are exercised directly by
// focused tests and standalone maintenance tools rather than the CLI.
#![allow(dead_code)]

mod checks;
mod cli;
mod context;
mod digest;
mod lock;
mod pin;
mod process;
mod receipt;
mod refresh;
mod report;
mod scheduler;
mod suite;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rights-verify: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
