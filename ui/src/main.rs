// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg_attr(
    all(feature = "desktop", target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use rights_book_ui::{App, data::PREFIX};
fn main() {
    #[cfg(feature = "web")]
    {
        let path = web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_else(|| PREFIX.into());
        dioxus::LaunchBuilder::web()
            .with_cfg(dioxus_web::Config::new().hydrate(true))
            .with_context(path)
            .launch(App);
    }
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new()
                    .with_window(
                        WindowBuilder::new()
                            .with_title("The Rights Nobody Has to Earn")
                            .with_inner_size(dioxus::desktop::LogicalSize::new(1180.0, 860.0)),
                    )
                    .with_custom_head(include_str!("../generated/desktop-head.html").into()),
            )
            .with_context(PREFIX.to_string())
            .launch(App);
    }
    #[cfg(not(any(feature = "web", feature = "desktop")))]
    {
        let _ = (App, PREFIX);
        eprintln!("Use --features web or --features desktop. See ui/README.md.");
    }
}
