// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod data;
mod pages;
mod reasoning;
mod state;
use data::*;
use dioxus::prelude::*;
use state::Preferences;

#[derive(Clone)]
pub struct PreferencesDirectory(pub std::path::PathBuf);

#[derive(Clone, Copy)]
pub struct Session {
    pub path: Signal<String>,
    pub preferences: Signal<Preferences>,
    pub ready: Signal<bool>,
    pub storage_error: Signal<String>,
    pub fragment: Signal<String>,
    pub history: Signal<Vec<String>>,
    pub future: Signal<Vec<String>>,
}
impl Session {
    pub fn save(mut self) {
        if let Err(e) = state::save(&self.preferences.peek()) {
            self.storage_error.set(e);
        }
    }
    pub fn visit(mut self, id: &str) {
        self.preferences.write().visit(id);
        self.save();
    }
    pub fn navigate(mut self, target: String) {
        let (path, fragment) = target.split_once('#').unwrap_or((&target, ""));
        if *self.path.peek() != path {
            let old = self.path.peek().clone();
            self.history.write().push(old);
            self.future.write().clear();
        }
        self.fragment.set(fragment.into());
        self.path.set(path.into());
    }
    pub fn back(mut self) {
        let target = self.history.write().pop();
        if let Some(target) = target {
            self.future.write().push(self.path.peek().clone());
            self.fragment.set(String::new());
            self.path.set(target);
        }
    }
    pub fn forward(mut self) {
        let target = self.future.write().pop();
        if let Some(target) = target {
            self.history.write().push(self.path.peek().clone());
            self.fragment.set(String::new());
            self.path.set(target);
        }
    }
}

#[component]
pub fn NavLink(
    to: String,
    children: Element,
    #[props(default = String::new())] class: String,
    #[props(default = String::new())] visit: String,
    #[props(default = String::new())] current: String,
) -> Element {
    let session = use_context::<Session>();
    let href = to.clone();
    rsx! { a { href, class, aria_current: if current.is_empty() { None } else { Some(current) },
        onclick: move |event| {
            if !visit.is_empty() { session.visit(&visit); }
            #[cfg(feature="desktop")]
            if to.starts_with(PREFIX) { event.prevent_default(); session.navigate(to.clone()); }
            #[cfg(not(feature="desktop"))] let _ = event;
        }, {children}
    } }
}

#[component]
pub fn Note(children: Element) -> Element {
    rsx! { div { class: "note", role: "note", span { class: "note-glyph", aria_hidden: "true", "⊥" } div { strong { "the honest part" } {children} } } }
}
#[component]
pub fn PageHeading(eyebrow: String, title: String, children: Element) -> Element {
    rsx! { header { class: "page-heading", span { class: "eyebrow", "// {eyebrow}" } h1 { "{title}" } {children} } }
}

#[allow(non_snake_case)]
pub fn App() -> Element {
    let initial = use_context::<String>();
    let path = use_signal(|| initial);
    let preferences = use_signal(Preferences::default);
    let ready = use_signal(|| false);
    let storage_error = use_signal(String::new);
    let fragment = use_signal(String::new);
    let history = use_signal(Vec::new);
    let future = use_signal(Vec::new);
    let mut session = Session {
        path,
        preferences,
        ready,
        storage_error,
        fragment,
        history,
        future,
    };
    use_context_provider(|| session);
    use_effect(move || {
        if !(session.ready)() {
            match state::load() {
                Ok(value) => session.preferences.set(value),
                Err(error) => session.storage_error.set(error),
            }
            session.ready.set(true);
        }
    });
    use_effect(move || {
        if !(session.ready)() {
            return;
        }
        let theme = session.preferences.read().theme.clone();
        let eval = document::eval("document.documentElement.dataset.theme = await dioxus.recv();");
        let _ = eval.send(theme);
    });
    let path = path();
    let section = path
        .strip_prefix(PREFIX)
        .unwrap_or("404")
        .split('/')
        .next()
        .unwrap_or("");
    let title = route_title(&path);
    #[cfg(feature = "desktop")]
    use_effect(move || {
        let path = (session.path)();
        let eval = document::eval(
            "const route=await dioxus.recv(); document.title=route.title; document.getElementById('main-content')?.focus({preventScroll:true}); if (!route.reader) window.scrollTo({top:0,behavior:'instant'});",
        );
        let _ = eval.send(serde_json::json!({"title":route_title(&path), "reader":book().pages.iter().any(|p|p.path==path)}));
    });
    rsx! {
        div { id: "book-app", "data-ready": if ready() { "true" } else { "false" },
            a { class: "skip-link", href: "#main-content", onclick: move |event| {
                #[cfg(feature="desktop")]
                { event.prevent_default(); let _ = document::eval("const main=document.getElementById('main-content');main?.focus({preventScroll:true});main?.scrollIntoView({behavior:'instant'});"); }
                #[cfg(not(feature="desktop"))] let _ = event;
            }, "Skip to content" }
            header { class: "book-bar",
                div { class: "container",
                    NavLink { to: PREFIX, class: "book-brand", "the rights nobody has to earn " span { "· book 1" } }
                    nav { class: "book-nav", aria_label: "Book",
                        if cfg!(feature="desktop") {
                            button { class: "q-btn q-btn--ghost", aria_label: "Book back", disabled: history.read().is_empty(), onclick: move |_| session.back(), "←" }
                            button { class: "q-btn q-btn--ghost", aria_label: "Book forward", disabled: future.read().is_empty(), onclick: move |_| session.forward(), "→" }
                        }
                        for (route, label) in [("", "floor"), ("map", "map"), ("walkthrough", "walkthrough"), ("read", "read"), ("search", "search"), ("about", "about")] {
                            NavLink { to: format!("{PREFIX}{}", match route { "" => "".into(), "walkthrough" => "walkthrough/food-delivery/".into(), _ => format!("{route}/") }), current: if section == route { "page" } else { "" }, "{label}" }
                        }
                        button { class: "q-btn q-btn--ghost theme-control", aria_label: "Switch colour theme", onclick: move |_| {
                            let theme = if session.preferences.peek().theme == "dark" { "light" } else { "dark" };
                            session.preferences.write().theme = theme.into(); session.save();
                        }, if preferences.read().theme == "dark" { "☼" } else { "☾" } }
                    }
                }
            }
            main { id: "main-content", tabindex: "-1", aria_label: "{title}",
                if path == PREFIX { pages::Floor {} }
                else if path == format!("{PREFIX}map/") { pages::Map {} }
                else if path == format!("{PREFIX}walkthrough/food-delivery/") { pages::Walkthrough {} }
                else if path == format!("{PREFIX}about/") { pages::About {} }
                else if path == format!("{PREFIX}read/") { pages::Contents {} }
                else if path == format!("{PREFIX}search/") { pages::Search {} }
                else if let Some(page) = book().pages.iter().find(|p| p.path == path) { pages::Reader { key: "{page.stem}", page: page.clone() } }
                else { div { class: "container page", PageHeading { eyebrow: "404", title: "Page not found", p { "This address does not identify a page in Book 1." } } NavLink { to: format!("{PREFIX}read/"), "Browse the contents →" } } }
            }
            div { class: "container",
                if !storage_error().is_empty() { p { class: "storage-notice", role: "status", "{storage_error}" } }
                div { class: "sources-strip",
                    span { "dhilipsiva · prose CC BY 4.0" }
                    a { href: public_url(format!("{path}index.md")), "Markdown" }
                    NavLink { to: format!("{PREFIX}about/"), "Sources & licences" }
                    a { href: public_url(format!("{PREFIX}llms.txt")), "Agent index" }
                }
            }
        }
    }
}
