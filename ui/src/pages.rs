// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{NavLink, PageHeading, Session, data::*};
use dioxus::prelude::*;
#[component]
pub fn Contents() -> Element {
    let session = use_context::<Session>();
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "the complete book", title: "The Rights Nobody Has to Earn", p { "By dhilipsiva. Epigraph, opening note, 31 chapters, and an unnumbered optional method. The manuscript is the source of every reader page." } }
        div { class: "two-column", div { class: "q-card pad",
            ol { class: "toc-list", for (i, page) in book().pages.iter().enumerate() {
                li {
                    if page.part.is_some() && (i == 0 || book().pages[i-1].part != page.part) { h2 { class: "toc-part", "{page.part.as_ref().unwrap()}" } }
                    NavLink { to: page.path.clone(), "{page.label}" }
                }
            } }
        } aside { class: "q-card pad marked side",
            h2 { class: "card-title", "Your reading place" }
            p { "Chapter and scroll position are saved locally after the page becomes interactive." }
            NavLink { to: session.preferences.read().resume(), class: "q-btn q-btn--primary", if session.preferences.read().last_read.is_some() { "Resume reading →" } else { "Begin with the epigraph →" } }
            p { NavLink { to: format!("{PREFIX}search/"), "Search the full text →" } }
            p { NavLink { to: PREFIX.to_string(), "Explore the companion →" } }
        } }
    } }
}
#[component]
pub fn Reader(page: Page) -> Element {
    let mut session = use_context::<Session>();
    let watch_page = use_memo(use_reactive((&page,), |(page,)| page));
    let mut bridge = use_signal(|| None::<dioxus::core::Task>);
    use_effect(move || {
        if !(session.ready)() {
            return;
        }
        let watch_page = watch_page();
        if let Some(task) = *bridge.peek() {
            task.cancel();
        }
        let fragment = (session.fragment)();
        let path = watch_page.path.clone();
        let position = session
            .preferences
            .peek()
            .positions
            .get(&path)
            .copied()
            .unwrap_or(0.0);
        {
            let mut prefs = session.preferences.write();
            prefs.last_read = Some(path.clone());
            for question in &companion().questions {
                if question
                    .chapters
                    .iter()
                    .any(|n| Some(*n) == watch_page.number)
                {
                    prefs.visit(&question.id);
                }
            }
        }
        session.save();
        let mut eval = document::eval(
            r#"
            const config = await dioxus.recv();
            await window.bookUI.reading(config, msg => dioxus.send(msg));
        "#,
        );
        let _ = eval.send(serde_json::json!({"path":path,"y":position,"desktop":cfg!(feature="desktop"),"fragment":fragment}));
        bridge.set(Some(spawn(async move {
            while let Ok(value) = eval.recv::<serde_json::Value>().await {
                if value["kind"] == "scroll" {
                    if let (Some(path), Some(y)) = (value["path"].as_str(), value["y"].as_f64()) {
                        session
                            .preferences
                            .write()
                            .positions
                            .insert(path.into(), y.max(0.0));
                        session.save();
                    }
                } else if value["kind"] == "navigate" {
                    if let Some(path) = value["path"].as_str() {
                        session.navigate(path.into());
                    }
                }
            }
        })));
    });
    use_drop(|| {
        let _ = document::eval("window.bookUI?.stopReading?.();");
    });
    rsx! { div { class: "container page reader-layout",
        aside { class: "reader-nav", details {
            summary { "In this chapter" }
            nav { aria_label: "Chapter sections", ul { for section in page.sections.iter().filter(|s| s.level > 1) { li { a { href: format!("#{}", section.id), "{section.title}" } } } } }
            p { NavLink { to: format!("{PREFIX}read/"), "All contents →" } }
        } }
        div { class: "reader-body",
            div { class: "reader-tools", NavLink { to: format!("{PREFIX}read/"), "← Contents" } a { href: public_url(format!("{}index.md",page.path)), "Markdown" } a { href: "{page.source}", "Manuscript source" } }
            div { dangerous_inner_html: page.html.clone() }
            nav { class: "chapter-navigation", aria_label: "Reading sequence",
                if let Some(previous) = &page.previous { NavLink { to: crate::data::page(previous).unwrap().path.clone(), "← {crate::data::page(previous).unwrap().label}" } }
                else { span {} }
                if let Some(next) = &page.next { NavLink { to: crate::data::page(next).unwrap().path.clone(), "{crate::data::page(next).unwrap().label} →" } }
                else { NavLink { to: format!("{PREFIX}read/"), "Return to contents →" } }
            }
        }
    } }
}
#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let results = search(&query());
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "local full-text search", title: "Find a passage", p { "Search all 34 reading inputs, including footnotes and Tamil. All search terms must occur in the chapter; matching ignores case. Your query stays on this device." } }
        div { style: "max-width:800px",
            label { r#for: "book-search", "Words or phrase" }
            input { id: "book-search", class: "search-input", r#type: "search", value: "{query}", placeholder: "Try: independent witness", oninput: move |event| query.set(event.value()), autocomplete: "off" }
            p { role: "status", aria_live: "polite", if query().trim().is_empty() { "Enter a word or phrase to search the book." } else { "{results.len()} reading inputs match." } }
            noscript { p { "Interactive search needs JavaScript. The complete contents and every chapter remain readable; use your browser’s Find command within a chapter." } }
            ul { class: "search-results", for (page, snippet) in results { li { NavLink { to: page.path.clone(), "{page.label}" } p { "{snippet}" } } } }
        }
    } }
}
