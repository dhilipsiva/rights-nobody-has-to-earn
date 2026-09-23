// SPDX-License-Identifier: MIT OR Apache-2.0
use nibli_session::CoreSession;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Scenario {
    pub id: String,
    pub counterfactual: Option<String>,
    pub record: Vec<String>,
    pub queries: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Edit {
    pub before: String,
    pub after: String,
}
#[derive(Deserialize)]
pub struct Inputs {
    pub constitution: String,
    pub counterfactuals: BTreeMap<String, Vec<Edit>>,
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
    pub counterfactual: Option<String>,
    pub complete: bool,
    pub verdicts: Vec<Verdict>,
}
fn lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect()
}
pub fn run(inputs: &Inputs, id: &str, cancellation: Arc<AtomicBool>) -> Result<Outcome, String> {
    let case = inputs
        .cases
        .iter()
        .find(|c| c.id == id)
        .ok_or("Unknown record")?;
    let mut source = inputs.constitution.clone();
    if let Some(cf) = &case.counterfactual {
        for edit in inputs
            .counterfactuals
            .get(cf)
            .ok_or("Unknown counterfactual")?
        {
            if source.matches(&edit.before).count() != 1 {
                return Err("Ambiguous counterfactual".into());
            }
            source = source.replacen(&edit.before, &edit.after, 1);
        }
    }
    let mut statements = lines(&source);
    statements.extend(case.record.iter().map(String::as_str));
    let (session, _) = CoreSession::from_text_batch_with_cancel(&statements, cancellation.clone())
        .map_err(|e| e.to_string())?;
    query(&session, case, &cancellation)
}
fn query(
    session: &CoreSession,
    case: &Scenario,
    cancellation: &AtomicBool,
) -> Result<Outcome, String> {
    session
        .kb()
        .prepare_materialization_plan()
        .map_err(|e| e.to_string())?;
    let mut verdicts = Vec::new();
    for text in &case.queries {
        if cancellation.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let (status, detail) = match session.compile_query_text(text) {
            Err(e) => ("REFUSED".to_string(), Some(e.to_string())),
            Ok(_) => match session.query_text(text) {
                Ok(result) => (
                    result.status_label().to_string(),
                    result.detail_label().map(str::to_string),
                ),
                Err(e) => ("INCOMPLETE".into(), Some(e.to_string())),
            },
        };
        verdicts.push(Verdict {
            query: text.clone(),
            status,
            detail,
        });
    }
    Ok(Outcome {
        id: case.id.clone(),
        counterfactual: case.counterfactual.clone(),
        complete: !verdicts.is_empty()
            && verdicts
                .iter()
                .all(|v| matches!(v.status.as_str(), "TRUE" | "FALSE" | "REFUSED")),
        verdicts,
    })
}
/// All constitution statements and declared transformations, never outcomes.
#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledInputs {
    pub base: Vec<nibli_types::logic::LogicBuffer>,
    pub edits: BTreeMap<String, Vec<CompiledEdit>>,
    pub cases: Vec<Scenario>,
    pub engine_revision: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledEdit {
    pub start: usize,
    pub count: usize,
    pub replacement: Vec<nibli_types::logic::LogicBuffer>,
}
pub fn compile(inputs: &Inputs) -> Result<CompiledInputs, String> {
    let core = CoreSession::new();
    let source = lines(&inputs.constitution);
    let compile_lines = |lines: Vec<&str>| {
        lines
            .iter()
            .map(|s| core.compile_text(s).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()
    };
    let base = compile_lines(source.clone())?;
    let mut edits = BTreeMap::new();
    for (id, changes) in &inputs.counterfactuals {
        let mut compiled = Vec::new();
        for edit in changes {
            let before = lines(&edit.before);
            if before.is_empty() {
                return Err("Empty counterfactual match".into());
            }
            let matches: Vec<_> = source
                .windows(before.len())
                .enumerate()
                .filter(|(_, window)| *window == before)
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err(format!("Ambiguous counterfactual: {id}"));
            }
            compiled.push(CompiledEdit {
                start: matches[0],
                count: before.len(),
                replacement: compile_lines(lines(&edit.after))?,
            });
        }
        compiled.sort_by_key(|e| std::cmp::Reverse(e.start));
        edits.insert(id.clone(), compiled);
    }
    Ok(CompiledInputs {
        base,
        edits,
        cases: inputs.cases.clone(),
        engine_revision: inputs.engine_revision.clone(),
    })
}
pub fn run_compiled(
    inputs: &CompiledInputs,
    id: &str,
    cancellation: Arc<AtomicBool>,
) -> Result<Outcome, String> {
    let case = inputs
        .cases
        .iter()
        .find(|c| c.id == id)
        .ok_or("Unknown record")?;
    if cancellation.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    // Immutable resource cache, isolated KB for EVERY record, including replays.
    let mut base = inputs.base.clone();
    if let Some(cf) = &case.counterfactual {
        for edit in inputs.edits.get(cf).ok_or("Unknown counterfactual")? {
            base.splice(
                edit.start..edit.start + edit.count,
                edit.replacement.clone(),
            );
        }
    }
    let compiler = CoreSession::new();
    let mut statements: Vec<_> = base
        .into_iter()
        .enumerate()
        .map(|(i, b)| (b, format!("constitution statement {}", i + 1)))
        .collect();
    for text in &case.record {
        statements.push((
            compiler.compile_text(text).map_err(|e| e.to_string())?,
            text.clone(),
        ));
    }
    let (kb, _) = nibli_reason::KnowledgeBase::from_compiled_batch_with_cancel(
        statements,
        cancellation.clone(),
    )
    .map_err(|e| e.to_string())?;
    query(&CoreSession::with_kb(kb), case, &cancellation)
}
#[wasm_bindgen]
pub struct Engine {
    inputs: CompiledInputs,
}
#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<Engine, JsError> {
        Ok(Self {
            inputs: postcard::from_bytes(bytes).map_err(|e| JsError::new(&e.to_string()))?,
        })
    }
    pub fn execute(&self, id: &str) -> Result<String, JsError> {
        let outcome = run_compiled(&self.inputs, id, Arc::new(AtomicBool::new(false)))
            .map_err(|e| JsError::new(&e))?;
        serde_json::to_string(&outcome).map_err(|e| JsError::new(&e.to_string()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_stops_construction() {
        let inputs = Inputs {
            constitution: "born(Nell).".into(),
            counterfactuals: BTreeMap::new(),
            engine_revision: "test".into(),
            cases: vec![Scenario {
                id: "test".into(),
                counterfactual: None,
                record: vec![],
                queries: vec!["born(Nell).".into()],
            }],
        };
        assert!(run(&inputs, "test", Arc::new(AtomicBool::new(true))).is_err());
        let compiled = compile(&inputs).unwrap();
        assert!(run_compiled(&compiled, "test", Arc::new(AtomicBool::new(true))).is_err());
    }
}
