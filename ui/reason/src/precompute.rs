// SPDX-License-Identifier: MIT OR Apache-2.0
use book_reason::{Inputs, compile, run, run_compiled};
use std::{
    fs,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
fn main() {
    // The source emitter traverses very long conjunctions. Windows' default
    // main-thread stack is too small; use the same explicit budget everywhere.
    std::thread::Builder::new()
        .name("book-precompute".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            if let Err(error) = precompute() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        })
        .expect("precompute thread")
        .join()
        .expect("precompute execution");
}
fn precompute() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let inputs: Inputs = serde_json::from_str(&fs::read_to_string(
        args.get(1).ok_or("inputs path required")?,
    )?)?;
    let started = Instant::now();
    let compiled = compile(&inputs)?;
    let encoded = postcard::to_stdvec(&compiled)?;
    fs::write(
        args.get(3).ok_or("compiled resource path required")?,
        &encoded,
    )?;
    eprintln!(
        "Compiled all {} constitution statements into {} bytes",
        compiled.base.len(),
        encoded.len()
    );
    let mut cases = Vec::new();
    // Run again in reverse order: restoration/removal must produce the same results.
    for case in &inputs.cases {
        let result = run(&inputs, &case.id, Arc::new(AtomicBool::new(false)))?;
        for (query, verdict) in case.queries.iter().zip(&result.verdicts) {
            if query.expected != verdict.status {
                return Err(format!(
                    "{}: {}: expected {}, got {}",
                    case.id, query.text, query.expected, verdict.status
                )
                .into());
            }
        }
        if !result.complete {
            return Err(format!("Incomplete scenario {}", case.id).into());
        }
        eprintln!("{}: {} queries", case.id, result.verdicts.len());
        cases.push(serde_json::json!({"scenario":case, "outcome":result}));
    }
    for row in cases.iter().rev() {
        let result = run_compiled(
            compiled.clone(),
            row["scenario"]["id"].as_str().unwrap(),
            Arc::new(AtomicBool::new(false)),
        )?;
        if serde_json::to_value(result)? != row["outcome"] {
            return Err("Compiled/full-source execution differs".into());
        }
    }
    let output = serde_json::json!({"label":"precomputed", "engine_revision":inputs.engine_revision,
        "constitution_source":"https://github.com/dhilipsiva/rights-nobody-has-to-earn/blob/main/book-1/source/constitution.nibli",
        "counterfactual_edit":inputs.edit,
        "scope":"Curated examples executed against the full constitution. No full pin suite or contradiction scan is claimed. Derived conclusions are not observations.",
        "license":"CC0-1.0", "cases":cases});
    fs::write(
        args.get(2).ok_or("output path required")?,
        serde_json::to_string_pretty(&output)? + "\n",
    )?;
    eprintln!(
        "Precomputed and replayed {} scenarios in {:.2}s",
        cases.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
