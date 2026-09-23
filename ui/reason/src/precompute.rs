// SPDX-License-Identifier: MIT OR Apache-2.0
//! Input compiler and optional development comparison. No outcomes are published.
use book_reason::{Inputs, compile, run, run_compiled};
use std::{
    fs,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
fn main() {
    std::thread::Builder::new()
        .name("book-input-compiler".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            if let Err(error) = build() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
fn build() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let inputs: Inputs = serde_json::from_str(&fs::read_to_string(
        args.get(1).ok_or("inputs path required")?,
    )?)?;
    let started = Instant::now();
    let compiled = compile(&inputs)?;
    fs::write(
        args.get(2).ok_or("compiled path required")?,
        postcard::to_stdvec(&compiled)?,
    )?;
    eprintln!(
        "Compiled all {} constitution statements in {:.2}s",
        compiled.base.len(),
        started.elapsed().as_secs_f64()
    );
    if let Some(expectations) = args.get(3) {
        let expected: serde_json::Value = serde_json::from_str(&fs::read_to_string(expectations)?)?;
        let mut results = Vec::new();
        let mut errors = Vec::new();
        for case in &inputs.cases {
            let now = Instant::now();
            let result = run(&inputs, &case.id, Arc::new(AtomicBool::new(false)))?;
            let replay = run_compiled(&compiled, &case.id, Arc::new(AtomicBool::new(false)))?;
            if result != replay || !result.complete {
                errors.push(format!(
                    "{} source/compiled mismatch or incomplete",
                    case.id
                ));
            }
            for wanted in expected[&case.id]
                .as_array()
                .ok_or("missing expectations")?
            {
                let actual = result
                    .verdicts
                    .iter()
                    .find(|v| v.query == wanted["query"].as_str().unwrap())
                    .ok_or("missing query")?;
                if actual.status != wanted["status"] {
                    errors.push(format!(
                        "{}: {}: wanted {} got {}",
                        case.id, actual.query, wanted["status"], actual.status
                    ));
                }
            }
            eprintln!(
                "{}: {} queries in {:.2}s",
                case.id,
                result.verdicts.len(),
                now.elapsed().as_secs_f64()
            );
            results.push(result);
        }
        // Changing rule, person and evidence must not retain prior facts.
        for id in [
            "nell:1",
            "j-independence:modified",
            "newcomer:0",
            "nell:0",
            "newcomer:1",
            "j-independence:canonical",
            "nell:0",
        ] {
            let replay = run_compiled(&compiled, id, Arc::new(AtomicBool::new(false)))?;
            if &replay
                != results
                    .iter()
                    .find(|r| r.id == id)
                    .ok_or("missing isolation baseline")?
            {
                errors.push(format!("isolation: {id}"));
            }
        }
        fs::create_dir_all("artifacts")?;
        fs::write(
            "artifacts/source-execution.json",
            serde_json::to_string_pretty(
                &serde_json::json!({"seconds":started.elapsed().as_secs_f64(),"outcomes":results,"errors":errors}),
            )?,
        )?;
        if !errors.is_empty() {
            return Err(errors.join("\n").into());
        }
        eprintln!(
            "Compared {} source/compiled records and isolation sequences in {:.2}s",
            results.len(),
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
