// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::data::Outcome;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(serde::Deserialize)]
pub struct Response {
    pub outcome: Outcome,
    pub elapsed_ms: f64,
}
#[cfg(feature = "web")]
async fn bridge(id: Option<String>) -> Result<serde_json::Value, String> {
    use dioxus::prelude::*;
    let mut eval = document::eval(
        r#"
        const id = await dioxus.recv();
        try { dioxus.send({ok: id === null ? await window.bookUI.start() : await window.bookUI.run(id)}); }
        catch (error) { dioxus.send({error: String(error.message || error)}); }
    "#,
    );
    eval.send(id).map_err(|e| e.to_string())?;
    let result: serde_json::Value = eval.recv().await.map_err(|e| e.to_string())?;
    if let Some(error) = result.get("error") {
        return Err(error.as_str().unwrap_or("Engine failed").into());
    }
    Ok(result["ok"].clone())
}
#[cfg(feature = "desktop")]
fn resources() -> Result<Arc<book_reason::CompiledInputs>, String> {
    use std::{io::Read, sync::OnceLock};
    static CACHE: OnceLock<Result<Arc<book_reason::CompiledInputs>, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut bytes = Vec::new();
            flate2::read::GzDecoder::new(&include_bytes!("../generated/constitution.bin.gz")[..])
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            postcard::from_bytes(&bytes)
                .map(Arc::new)
                .map_err(|e| e.to_string())
        })
        .clone()
}
#[cfg(feature = "desktop")]
async fn native(id: Option<String>, flag: Arc<AtomicBool>) -> Result<Option<Response>, String> {
    let (tx, rx) = futures_channel::oneshot::channel();
    std::thread::Builder::new()
        .name("book-engine".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let now = std::time::Instant::now();
            let result = (|| {
                let inputs = resources()?;
                if flag.load(Ordering::Relaxed) {
                    return Err("Cancelled".into());
                }
                id.map(|id| {
                    let outcome = book_reason::run_compiled(&inputs, &id, flag)?;
                    Ok(Response {
                        outcome: serde_json::from_value(
                            serde_json::to_value(outcome).map_err(|e| e.to_string())?,
                        )
                        .map_err(|e| e.to_string())?,
                        elapsed_ms: now.elapsed().as_secs_f64() * 1000.0,
                    })
                })
                .transpose()
            })();
            let _ = tx.send(result);
        })
        .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}
pub async fn initialize(flag: Arc<AtomicBool>) -> Result<(), String> {
    #[cfg(feature = "web")]
    {
        let _ = flag;
        bridge(None).await?;
    }
    #[cfg(feature = "desktop")]
    {
        native(None, flag).await?;
    }
    #[cfg(not(any(feature = "web", feature = "desktop")))]
    let _ = flag;
    Ok(())
}
pub async fn execute(id: String, flag: Arc<AtomicBool>) -> Result<Response, String> {
    #[cfg(feature = "web")]
    {
        let _ = flag;
        serde_json::from_value(bridge(Some(id)).await?).map_err(|e| e.to_string())
    }
    #[cfg(feature = "desktop")]
    {
        native(Some(id), flag).await?.ok_or("No result".into())
    }
    #[cfg(not(any(feature = "web", feature = "desktop")))]
    {
        let _ = (id, flag);
        Err("Static rendering".into())
    }
}
pub fn cancel(flag: &AtomicBool) {
    flag.store(true, Ordering::Relaxed);
    #[cfg(feature = "web")]
    {
        let _ = dioxus::prelude::document::eval("window.bookUI?.cancel();");
    }
}
