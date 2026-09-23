// SPDX-License-Identifier: MIT OR Apache-2.0
//! Development-only executable exercising the actual native WebView and thread adapter.
use dioxus::{
    desktop::{Config, WindowBuilder},
    prelude::*,
};
use rights_book_ui::{App, PreferencesDirectory, data::PREFIX, reasoning};
use std::path::PathBuf;
#[derive(Clone)]
struct Output(PathBuf);
#[derive(Clone)]
struct CheckMode {
    resume: bool,
    reader_only: bool,
}
#[allow(non_snake_case)]
fn Harness() -> Element {
    let output = use_context::<Output>().0;
    let mode = use_context::<CheckMode>();
    let (resume, reader_only) = (mode.resume, mode.reader_only);
    use_effect(move || {
        let output = output.clone();
        spawn(async move {
            let mut eval = document::eval(include_str!("../../tests/desktop.js"));
            let expectations: serde_json::Value =
                serde_json::from_str(include_str!("../../tests/expectations.json")).unwrap();
            let _ = eval.send(serde_json::json!({"expectations":expectations,"resume":resume,"reader_only":reader_only}));
            let result = loop {
                let message = eval.recv::<serde_json::Value>().await;
                if let Ok(value) = &message {
                    if value["kind"] == "execute" {
                        let result = reasoning::execute(
                            value["id"].as_str().unwrap().into(),
                            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                        )
                        .await;
                        let response = match result {
                            Ok(r) => {
                                serde_json::json!({"outcome":r.outcome,"elapsed_ms":r.elapsed_ms})
                            }
                            Err(e) => serde_json::json!({"error":e}),
                        };
                        let _ = eval.send(response);
                        continue;
                    }
                }
                break message;
            };
            let success = result.as_ref().is_ok_and(|r| r["ok"] == true);
            let json =
                result.unwrap_or_else(|e| serde_json::json!({"ok":false,"error":e.to_string()}));
            std::fs::write(&output, serde_json::to_vec_pretty(&json).unwrap())
                .expect("test output");
            println!(
                "Desktop check: ok={}, scenarios={}, output={}",
                success,
                json["scenarios"],
                output.display()
            );
            std::process::exit(if success { 0 } else { 1 });
        });
    });
    rsx! { App {} }
}
fn main() {
    let output = std::env::current_dir().unwrap().join(PathBuf::from(
        std::env::args().nth(1).expect("test result JSON path"),
    ));
    let prefs = output.with_extension("preferences");
    std::fs::create_dir_all(&prefs).unwrap();
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Book UI development check")
                        // WebKitGTK suspends scrolling in hidden windows. Linux
                        // CI supplies a virtual display; WebView2 can stay hidden.
                        .with_visible(!cfg!(target_os = "windows")),
                )
                .with_data_directory(prefs.join("webview"))
                .with_custom_head(include_str!("../../generated/desktop-head.html").into()),
        )
        .with_context(PREFIX.to_string())
        .with_context(Output(output))
        .with_context(PreferencesDirectory(prefs))
        .with_context(CheckMode {
            resume: std::env::args().any(|arg| arg == "--resume"),
            reader_only: std::env::args().any(|arg| arg == "--reader-only"),
        })
        .launch(Harness);
}
