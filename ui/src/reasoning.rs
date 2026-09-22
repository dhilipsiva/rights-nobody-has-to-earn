// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::data::{Outcome, cases};
use dioxus::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(serde::Deserialize)]
struct Response {
    outcome: Outcome,
    elapsed_ms: f64,
}

#[cfg(feature = "web")]
async fn execute(id: String, _: Arc<AtomicBool>) -> Result<Response, String> {
    let mut eval = document::eval(
        r#"
        const id = await dioxus.recv();
        try { dioxus.send({ok:await window.bookUI.run(id)}); }
        catch (error) { dioxus.send({error:String(error.message || error)}); }
    "#,
    );
    eval.send(id).map_err(|e| e.to_string())?;
    let result: serde_json::Value = eval.recv().await.map_err(|e| e.to_string())?;
    if let Some(error) = result.get("error") {
        return Err(error.as_str().unwrap_or("Engine failed").into());
    }
    serde_json::from_value(result["ok"].clone()).map_err(|e| e.to_string())
}
#[cfg(feature = "desktop")]
async fn execute(id: String, cancelled: Arc<AtomicBool>) -> Result<Response, String> {
    let (sender, receiver) = futures_channel::oneshot::channel();
    std::thread::Builder::new()
        .name("book-reasoning".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            use std::io::Read;
            let started = std::time::Instant::now();
            let result = (|| -> Result<Response, String> {
                let mut source = Vec::new();
                flate2::read::GzDecoder::new(
                    &include_bytes!("../generated/constitution.bin.gz")[..],
                )
                .read_to_end(&mut source)
                .map_err(|e| e.to_string())?;
                let inputs = postcard::from_bytes(&source).map_err(|e| e.to_string())?;
                let result = book_reason::run_compiled(inputs, &id, cancelled)?;
                Ok(Response {
                    outcome: serde_json::from_value(
                        serde_json::to_value(result).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?,
                    elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                })
            })();
            let _ = sender.send(result);
        })
        .map_err(|e| e.to_string())?;
    receiver.await.map_err(|e| e.to_string())?
}
#[cfg(not(any(feature = "web", feature = "desktop")))]
async fn execute(_: String, _: Arc<AtomicBool>) -> Result<Response, String> {
    Err("Static rendering".into())
}
fn cancel(flag: &AtomicBool) {
    flag.store(true, Ordering::Relaxed);
    #[cfg(feature = "web")]
    {
        let _ = document::eval("window.bookUI?.cancel();");
    }
}

#[component]
pub fn Reasoning(case_id: String, onresult: EventHandler<Outcome>) -> Element {
    let case = cases()
        .cases
        .iter()
        .find(|c| c.scenario.id == case_id)
        .expect("curated case")
        .clone();
    let mut outcome = use_signal(|| case.outcome.clone());
    let mut loading = use_signal(|| false);
    let mut live = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut generation = use_signal(|| 0_u64);
    let mut cancellation = use_signal(|| Arc::new(AtomicBool::new(false)));
    let selection = use_memo(use_reactive((&case_id,), |(id,)| id));
    use_effect(move || {
        let id = selection();
        cancel(&cancellation.peek());
        let next = *generation.peek() + 1;
        generation.set(next);
        loading.set(false);
        live.set(false);
        message.set(String::new());
        outcome.set(
            cases()
                .cases
                .iter()
                .find(|c| c.scenario.id == id)
                .expect("curated case")
                .outcome
                .clone(),
        );
    });
    use_drop(move || cancel(&cancellation.peek()));
    let id = case_id.clone();
    let mut run = move |_| {
        cancel(&cancellation.peek());
        let flag = Arc::new(AtomicBool::new(false));
        cancellation.set(flag.clone());
        generation += 1;
        let request = generation();
        loading.set(true);
        live.set(false);
        message.set(
            "Loading the local engine and full constitution. Reading remains available.".into(),
        );
        let id = id.clone();
        spawn(async move {
            let result = execute(id.clone(), flag.clone()).await;
            if flag.load(Ordering::Relaxed) || request != generation() || id != selection() {
                return;
            }
            loading.set(false);
            match result {
                Ok(result) if result.outcome.id == id && result.outcome.complete => {
                    message.set(format!("Executed locally in {:.2} s. Derived conclusions do not establish real-world delivery.", result.elapsed_ms / 1000.0));
                    live.set(true);
                    onresult.call(result.outcome.clone());
                    outcome.set(result.outcome);
                }
                Ok(_) => message.set("Execution was incomplete. The displayed examples remain precomputed; retry when resources are available.".into()),
                Err(error) => message.set(format!("Local execution failed: {error}. The displayed examples remain precomputed. You can retry.")),
            }
        });
    };
    let record = if case.scenario.record.is_empty() {
        "// No scenario entries supplied.".into()
    } else {
        case.scenario.record.join("\n")
    };
    let is_live = live() && outcome.read().id == case_id;
    let displayed = if is_live {
        outcome()
    } else {
        case.outcome.clone()
    };
    rsx! {
        div { class: "term", "data-case": "{case_id}",
            div { class: "term__bar",
                span { class: "term__dot", style: "background:var(--crimson-500)" }
                span { class: "term__dot", style: "background:var(--amber-500)" }
                span { class: "term__dot", style: "background:var(--phosphor-500)" }
                span { class: "term__title", "{case.scenario.id} · nibli" }
                span { class: if is_live { "status-label live" } else { "status-label" }, "data-result-kind": if is_live {"live"} else {"precomputed"}, if is_live { "live" } else { "precomputed" } }
            }
            div { class: "term__body",
                p { class: "term-note", if case.scenario.counterfactual { "// Counterfactual · isolated constitution copy" } else { "// Canonical constitution · isolated scenario record" } }
                pre { "{record}" }
                for verdict in displayed.verdicts.iter().take(if case_id.starts_with("floor") { 0 } else { 20 }) {
                    div { class: "query", code { "? {verdict.query}" } output { class: verdict.status.to_lowercase(), "{verdict.status}" } }
                }
                if case_id.starts_with("floor") {
                    for query in ["person(Nell).", "owe(State, Eats, Nell).", "eats(Nell).", "dwell(Nell).", "vulnerable(Nell)."] {
                        if let Some(verdict) = displayed.verdicts.iter().find(|v| v.query == query) {
                            div { class: "query", code { "? {verdict.query}" } output { class: verdict.status.to_lowercase(), "{verdict.status}" } }
                        }
                    }
                }
                details { summary { "What the answers mean" }
                    p { "TRUE: derivable. FALSE: not derivable from this record. REFUSED: the query cannot be admitted, such as an unknown predicate. An incomplete run supplies no FALSE verdict." }
                }
                div { class: "run-controls",
                    button { class: "q-btn q-btn--primary", disabled: loading(), onclick: move |e| run(e), if message().contains("failed") || message().contains("incomplete") { "Retry locally" } else { "Run locally" } }
                    if loading() { button { class: "q-btn q-btn--secondary", onclick: move |_| { cancel(&cancellation.peek()); generation += 1; loading.set(false); live.set(false); message.set("Cancelled. Precomputed examples remain available.".into()); }, "Cancel" } }
                }
                p { class: "run-message", role: "status", aria_live: "polite", if message().is_empty() { "Initial results were executed during the build. Run locally to check this scenario on your device." } else { "{message}" } }
            }
        }
    }
}
