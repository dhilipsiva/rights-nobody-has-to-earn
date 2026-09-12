// SPDX-License-Identifier: MIT OR Apache-2.0

// Shared pin machinery is also used by the explicit authoring executable.
#![allow(dead_code)]

#[path = "verify_cli.rs"]
mod cli;
mod context;
mod execution;
mod pin;
#[path = "runner_pool.rs"]
mod scheduler;

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
