// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[path = "../authoring.rs"]
mod authoring;
#[path = "../verify_cli.rs"]
mod cli;
#[path = "../context.rs"]
mod context;
#[path = "../execution.rs"]
mod execution;
#[path = "../pin.rs"]
mod pin;
#[path = "../runner_pool.rs"]
mod scheduler;

fn main() -> std::process::ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = if args.len() == 1 {
        context::Context::discover().and_then(|context| authoring::run(&context, &args[0]))
    } else {
        Err(cli::Error::usage(
            "usage: ./generate.sh state-form|obligations|integrity|statistics|spine",
        ))
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("generate: {error}");
            std::process::ExitCode::from(error.exit_code())
        }
    }
}
