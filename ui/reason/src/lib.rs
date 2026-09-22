// SPDX-License-Identifier: MIT OR Apache-2.0
use nibli_session::CoreSession;
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    pub text: String,
    pub expected: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub source: String,
    pub counterfactual: bool,
    pub record: Vec<String>,
    pub queries: Vec<Query>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edit {
    pub before: String,
    pub after: String,
}
#[derive(Deserialize)]
pub struct Inputs {
    pub constitution: String,
    pub edit: Edit,
    pub engine_revision: String,
    pub cases: Vec<Scenario>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Verdict {
    pub query: String,
    pub status: String,
    pub detail: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Outcome {
    pub id: String,
    pub counterfactual: bool,
    pub complete: bool,
    pub verdicts: Vec<Verdict>,
}

pub fn run(inputs: &Inputs, id: &str, cancellation: Arc<AtomicBool>) -> Result<Outcome, String> {
    let case = inputs
        .cases
        .iter()
        .find(|c| c.id == id)
        .ok_or("Unknown scenario")?;
    let mut source = inputs.constitution.clone();
    if case.counterfactual {
        if source.matches(&inputs.edit.before).count() != 1 {
            return Err("Counterfactual requires exactly one matching source rule".into());
        }
        source = source.replacen(&inputs.edit.before, &inputs.edit.after, 1);
    }
    // Every request starts from the full constitution and its own record. In
    // particular, a restored rule and a backwards step cannot retain old facts.
    let statements: Vec<&str> = source
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .chain(case.record.iter().map(String::as_str))
        .collect();
    let (session, _) = CoreSession::from_text_batch_with_cancel(&statements, cancellation.clone())
        .map_err(|e| format!("Source or record could not load: {e}"))?;
    session
        .kb()
        .prepare_materialization_plan()
        .map_err(|e| e.to_string())?;
    query(&session, case, &cancellation)
}

fn query(
    session: &CoreSession,
    case: &Scenario,
    cancellation: &AtomicBool,
) -> Result<Outcome, String> {
    let mut verdicts = Vec::new();
    for query in &case.queries {
        if cancellation.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let (status, detail) = match session.compile_query_text(&query.text) {
            Err(e) => ("REFUSED".to_string(), Some(e.to_string())),
            Ok(_) => match session.query_text(&query.text) {
                Ok(result) => (
                    result.status_label().to_string(),
                    result.detail_label().map(str::to_string),
                ),
                Err(e) => ("INCOMPLETE".to_string(), Some(e.to_string())),
            },
        };
        verdicts.push(Verdict {
            query: query.text.clone(),
            status,
            detail,
        });
    }
    let complete = verdicts
        .iter()
        .all(|v| matches!(v.status.as_str(), "TRUE" | "FALSE" | "REFUSED"));
    Ok(Outcome {
        id: case.id.clone(),
        counterfactual: case.counterfactual,
        complete,
        verdicts,
    })
}

/// The full constitution compiled by the pinned engine. No rule is filtered.
/// Flat LogicBuffers avoid reparsing enormous conjunction trees on the browser's
/// fixed call stack. Every runtime still builds a fresh KB and executes queries.
#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledInputs {
    pub base: Vec<nibli_types::logic::LogicBuffer>,
    pub edit_index: usize,
    pub replacement: nibli_types::logic::LogicBuffer,
    pub cases: Vec<Scenario>,
    pub engine_revision: String,
}
pub fn compile(inputs: &Inputs) -> Result<CompiledInputs, String> {
    let core = CoreSession::new();
    let lines: Vec<_> = inputs
        .constitution
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect();
    let matches: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, s)| **s == inputs.edit.before.trim())
        .map(|(i, _)| i)
        .collect();
    if matches.len() != 1 {
        return Err("Counterfactual rule must match exactly once".into());
    }
    let base = lines
        .iter()
        .map(|s| core.compile_text(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompiledInputs {
        base,
        edit_index: matches[0],
        replacement: core
            .compile_text(inputs.edit.after.trim())
            .map_err(|e| e.to_string())?,
        cases: inputs.cases.clone(),
        engine_revision: inputs.engine_revision.clone(),
    })
}
pub fn run_compiled(
    mut inputs: CompiledInputs,
    id: &str,
    cancellation: Arc<AtomicBool>,
) -> Result<Outcome, String> {
    let case = inputs
        .cases
        .iter()
        .find(|c| c.id == id)
        .ok_or("Unknown scenario")?;
    if cancellation.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    if case.counterfactual {
        inputs.base[inputs.edit_index] = inputs.replacement;
    }
    let compiler = CoreSession::new();
    let mut statements: Vec<_> = inputs
        .base
        .into_iter()
        .enumerate()
        .map(|(i, buf)| (buf, format!("constitution statement {}", i + 1)))
        .collect();
    for record in &case.record {
        statements.push((
            compiler.compile_text(record).map_err(|e| e.to_string())?,
            record.clone(),
        ));
    }
    let (kb, _) = nibli_reason::KnowledgeBase::from_compiled_batch_with_cancel(
        statements,
        cancellation.clone(),
    )
    .map_err(|e| e.to_string())?;
    let session = CoreSession::with_kb(kb);
    session
        .kb()
        .prepare_materialization_plan()
        .map_err(|e| e.to_string())?;
    query(&session, case, &cancellation)
}

#[wasm_bindgen]
pub fn execute(inputs: &[u8], id: &str) -> Result<String, JsError> {
    let data: CompiledInputs =
        postcard::from_bytes(inputs).map_err(|e| JsError::new(&e.to_string()))?;
    let outcome =
        run_compiled(data, id, Arc::new(AtomicBool::new(false))).map_err(|e| JsError::new(&e))?;
    serde_json::to_string(&outcome).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_stops_construction() {
        let inputs = Inputs {
            constitution: "born(Nell).".into(),
            edit: Edit {
                before: "unused".into(),
                after: "unused".into(),
            },
            engine_revision: "test".into(),
            cases: vec![Scenario {
                id: "test".into(),
                title: "test".into(),
                source: "test".into(),
                counterfactual: false,
                record: vec![],
                queries: vec![],
            }],
        };
        assert!(run(&inputs, "test", Arc::new(AtomicBool::new(true))).is_err());
    }
}
