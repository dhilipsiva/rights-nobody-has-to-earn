// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[path = "../amendment_host.rs"]
mod amendment_host;

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = (|| -> Result<(), String> {
        if args.len() != 1 {
            return Err("usage: amendment-assurance <trusted-local-scenario.json>\nReplays supplied evidence in memory; does not authenticate or deploy anything.".into());
        }
        let bytes = std::fs::read(&args[0]).map_err(|e| e.to_string())?;
        let scenario = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let host = std::thread::Builder::new()
            .name("amendment-host".into())
            .stack_size(32 * 1024 * 1024)
            .spawn(move || amendment_host::replay(scenario))
            .map_err(|e| e.to_string())?
            .join()
            .map_err(|_| "amendment host worker failed".to_owned())??;
        println!(
            "In-memory checks passed: effective {}, {} transitions. No authentication, publication or deployment performed.",
            host.effective().id,
            host.history().len()
        );
        Ok(())
    })();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("amendment-assurance: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
