// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{
    NavLink, PageHeading, Session,
    data::*,
    game_state::{chapters_of, game},
};
use dioxus::prelude::*;
#[component]
pub fn Contents() -> Element {
    let session = use_context::<Session>();
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "the complete book", title: "The Rights Nobody Has to Earn", p { "By dhilipsiva. The epigraph, the opening note, every chapter and Part opening case, the unnumbered optional method, and a map, glossary and index. The manuscript is the source of every reader page." } }
        div { class: "two-column", div { class: "q-card pad",
            ol { class: "toc-list", for (i, page) in book().pages.iter().enumerate() {
                li {
                    if page.part.is_some() && (i == 0 || book().pages[i-1].part != page.part) { h2 { class: "toc-part", "{page.part.as_ref().unwrap()}" } }
                    NavLink { to: page.path.clone(), "{page.label}" }
                    if !page.summary.is_empty() { p { class: "toc-summary", "{page.summary}" } }
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
        PageHeading { eyebrow: "local full-text search", title: "Find a passage", p { "Search every reading input, including footnotes and Tamil. All search terms must occur in the chapter; matching ignores case. Your query stays on this device." } }
        div { style: "max-width:800px",
            label { r#for: "book-search", "Words or phrase" }
            input { id: "book-search", class: "search-input", r#type: "search", value: "{query}", placeholder: "Try: independent witness", oninput: move |event| query.set(event.value()), autocomplete: "off" }
            p { role: "status", aria_live: "polite", if query().trim().is_empty() { "Enter a word or phrase to search the book." } else { "{results.len()} reading inputs match." } }
            noscript { p { "Interactive search needs JavaScript. The complete contents and every chapter remain readable; use your browser’s Find command within a chapter." } }
            ul { class: "search-results", for (page, snippet) in results { li { NavLink { to: page.path.clone(), "{page.label}" } p { "{snippet}" } } } }
        }
    } }
}
/// The formal source also has hand-written sections headed "Article …"; name
/// them as the source's so they are not read as the plain-language numbers.
fn family_label(family: &str) -> String {
    if family.starts_with("Article ") {
        format!("the source's {family}")
    } else {
        family.to_owned()
    }
}
#[component]
pub fn Constitution() -> Element {
    let doc = articles();
    let blob = |path: &str| format!("{REPOSITORY}/blob/main/{path}");
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "the plain-language constitution", title: "{doc.title}", p { "{doc.note}" } }
        nav { aria_label: "Articles", class: "q-card pad",
            for (index, part) in doc.parts.iter().enumerate() {
                h2 { class: "toc-part", "{part}" }
                ol { class: "toc-list", start: "{doc.articles.iter().find(|a| a.part == index + 1).map(|a| a.number).unwrap_or(1)}",
                    for article in doc.articles.iter().filter(|a| a.part == index + 1) {
                        li { a { href: "#article-{article.number}", "Article {article.number}. {article.title}" } }
                    }
                }
            }
        }
        for (index, part) in doc.parts.iter().enumerate() {
            section { aria_label: "{part}",
                h2 { "{part}" }
                for article in doc.articles.iter().filter(|a| a.part == index + 1) {
                    article { id: "article-{article.number}", class: "q-card pad article",
                        h3 { "Article {article.number}. {article.title}" }
                        if article.core == "whole" { p { class: "eyebrow", "Protected core: beyond amendment." } }
                        else if let Some(scope) = article.core.strip_prefix("part: ") { p { class: "eyebrow", "Protected core in part: {scope}." } }
                        ol { for clause in article.text.iter() { li { "{clause}" } } }
                        if let Some(gap) = &article.gap { p { class: "note-text", strong { "Not formal in part. " } "{gap}" } }
                        details {
                            summary { "Where it comes from, and what it adds" }
                            p { "Argued in " for (i, n) in article.chapters.iter().enumerate() {
                                if i > 0 { ", " }
                                NavLink { to: chapter(*n).path.clone(), "Chapter {n}" }
                            } "." }
                            p { "Rule families in the " a { href: blob("book-1/source/constitution.nibli"), "formal source" } ": " for (i, family) in article.families.iter().enumerate() {
                                if i > 0 { ", " }
                                "{family_label(family)}"
                            } "." }
                            p { "Governed by: " for (i, record) in article.records.iter().enumerate() {
                                if i > 0 { "; " }
                                a { href: blob(record), "{record}" }
                            } "." }
                            p { "Tested by: " for (i, pin) in article.pins.iter().enumerate() {
                                if i > 0 { "; " }
                                a { href: blob(pin), "{pin}" }
                            } "." }
                            if !article.lineage.is_empty() {
                                p { "Compare, through the Constitute Project:" }
                                ul { for c in article.lineage.iter() {
                                    li { a { href: "{c.url}", "{c.constitution}, {c.provision}" } ": “{c.quote}”" }
                                } }
                            }
                            p { "{article.novelty}" }
                        }
                    }
                }
            }
        }
    } }
}
#[component]
pub fn Cases() -> Element {
    let doc = chapter_cases();
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "run the chapters", title: "{doc.title}", p { "{doc.note}" } }
        for page in book().pages.iter().filter(|p| p.number.is_some()) {{
            let n = page.number.unwrap();
            let forks: Vec<_> = game().scenarios.iter().filter(|f| chapters_of(&f.id, f.chapter).contains(&n)).collect();
            let joints: Vec<_> = game().joints.iter().filter(|j| j.measured && chapters_of(&j.id, j.chapter).contains(&n)).collect();
            let moved: Vec<_> = doc.moved.iter().filter(|m| m.chapter == n).collect();
            // Part V argues rather than states rules, so a chapter with nothing
            // to run or record is left out; every derived chapter has a case.
            if forks.is_empty() && joints.is_empty() && moved.is_empty() { return rsx! {}; }
            rsx! { section { id: "chapter-{n}", class: "q-card pad chapter-cases", "data-chapter": "{n}",
                h2 { NavLink { to: page.path.clone(), "{page.label}" } }
                if !forks.is_empty() { ul { class: "case-links", for f in forks {
                    li { a { href: "{PREFIX}#fork={f.id}", "data-run": "fork", "{f.title}" } " · {f.role}" }
                } } }
                if !joints.is_empty() { ul { class: "case-links", for j in joints {
                    li { a { href: "{PREFIX}#joint={j.id}", "data-run": "joint", "{j.title}" } " · compared live with “{j.off_label}”" }
                } } }
                for m in moved {
                    div { class: "moved",
                        h3 { "{m.title}" }
                        p { "{m.text}" }
                        ul { for link in m.links.iter() { li { a { href: repository_url(&link.path), "{link.label}" } } } }
                    }
                }
            } }
        }}
    } }
}
#[component]
pub fn Start() -> Element {
    let doc = start();
    let guided = &doc.guided;
    let explainer = &doc.explainer;
    let joint = game()
        .joints
        .iter()
        .find(|j| j.id == explainer.joint)
        .expect("explained joint");
    rsx! { div { class: "container page start-page",
        PageHeading { eyebrow: "a way in", title: "{doc.title}",
            for paragraph in doc.panel.iter() { p { "{paragraph}" } }
        }
        section { id: "guided-run", class: "q-card pad", aria_label: "A guided first run",
            h2 { "A guided first run" }
            p { "{guided.note}" }
            ol { class: "guided-steps",
                for step in guided.steps.iter() {
                    li { "data-query": "{step.query}",
                        p { "{step.text}" }
                        p { class: "guided-query", code { "? {step.query}" } " " span { class: "q-badge", "{step.verdict}" } }
                    }
                }
            }
            p { a { class: "q-btn q-btn--primary", href: "{PREFIX}#fork={guided.fork}", "Run Nell's fork live →" } " " a { href: "{REPOSITORY}/blob/main/{guided.pins}", "The tests these answers come from ↗" } }
        }
        section { id: "forks-and-joints", class: "q-card pad", aria_label: "Forks and joints",
            h2 { "Forks and joints" }
            for paragraph in explainer.text.iter() { p { "{paragraph}" } }
            ul { class: "case-links",
                li { a { href: "{PREFIX}#fork={explainer.fork}", "data-run": "fork", "Nell's fork: Be born with nobody" } }
                li { a { href: "{PREFIX}#joint={joint.id}", "data-run": "joint", "{joint.title}" } " · compared live with “{joint.off_label}”" }
            }
        }
        nav { class: "q-card pad", aria_label: "Where next",
            h2 { "Where next" }
            ul {
                li { NavLink { to: format!("{PREFIX}design/"), "The design in ten minutes, in the articles' words" } }
                li { NavLink { to: format!("{PREFIX}read/"), "The book map: every chapter with what it settles" } }
                li { NavLink { to: format!("{PREFIX}cases/"), "Each chapter's cases, ready to run" } }
                li { NavLink { to: format!("{PREFIX}constitution/"), "The constitution in numbered plain-language articles" } }
                li { a { href: "{PREFIX}#dossier", "The dossier of limits, costs and objections" } }
                li { NavLink { to: format!("{PREFIX}limits/"), "What the design and its checks leave open" } }
                li { NavLink { to: format!("{PREFIX}second-engine/"), "The rules replayed in a second engine" } }
            }
        }
    } }
}
/// A line whose `backticked` spans are code.
fn coded(text: &str) -> Element {
    let parts: Vec<(bool, String)> = text
        .split('`')
        .enumerate()
        .map(|(i, part)| (i % 2 == 1, part.to_owned()))
        .collect();
    rsx! {
        for (is_code, part) in parts.into_iter() {
            if is_code { code { "{part}" } } else { span { "{part}" } }
        }
    }
}
/// The reader address of a section of one book file, found by its heading.
fn section_link(file_stem: &str, heading: &str) -> Option<String> {
    let page = book().pages.iter().find(|p| p.stem == file_stem)?;
    let section = page.sections.iter().find(|s| s.title == heading)?;
    Some(format!("{}#{}", page.path, section.id))
}
#[component]
pub fn SecondEngine() -> Element {
    let engine = &assurance().engine;
    let agreement = if engine.differences == 0 {
        "Every answer agrees with the verdict its pin records.".to_owned()
    } else {
        format!(
            "{} answers differ from the verdict their pins record.",
            engine.differences
        )
    };
    let groups = engine
        .groups
        .iter()
        .map(|g| format!("{} {}", g.name, g.cases))
        .collect::<Vec<_>>()
        .join(", ");
    let method = section_link("method", &engine.method_section).unwrap_or_default();
    let cases = engine.cases.len();
    let statements = thousands(engine.statements);
    let queries = thousands(engine.queries);
    rsx! { div { class: "container page assurance-page",
        PageHeading { eyebrow: "what has been checked", title: "The second engine",
            p { "The constitution's rules were translated, statement by statement, into the input language of clingo {engine.clingo}, a separate answer set solver: {statements} clingo statements. The cases below were replayed through it, {cases} cases and {queries} queries, each answer compared with the verdict its pin records. {agreement}" }
        }
        section { id: "where-it-stops", class: "q-card pad", aria_label: "Where it stops",
            h2 { "Where it stops" }
            blockquote { p { "{engine.limit}" } }
            p { "From the method's section " a { href: "{method}", "{engine.method_section}" } ", which lists every check the project publishes and where each stops." }
        }
        section { id: "keeps", class: "q-card pad", aria_label: "What the translation keeps",
            h2 { "What the translation keeps" }
            ul { for item in engine.keeps.iter() { li { {coded(item)} } } }
        }
        section { id: "leaves-out", class: "q-card pad", aria_label: "What the comparison leaves out",
            h2 { "What the comparison leaves out" }
            ul { for item in engine.leaves_out.iter() { li { {coded(item)} } } }
        }
        section { id: "engine-cases", class: "q-card pad", aria_label: "The cases replayed",
            h2 { "The cases replayed" }
            p { "By group: {groups}." }
            details {
                summary { "Every case, with its queries and the answers that differ" }
                div { class: "table-scroll", role: "region", aria_label: "Cases replayed in the second engine", tabindex: "0",
                    table {
                        thead { tr { th { scope: "col", "Case" } th { scope: "col", "Queries" } th { scope: "col", "Answers that differ" } } }
                        tbody { for case in engine.cases.iter() { tr { "data-engine-case": "{case.id}", td { code { "{case.id}" } } td { "{case.queries}" } td { "{case.differences}" } } } }
                    }
                }
            }
            p { a { href: "{REPOSITORY}/blob/main/book-1/source/measurements/second-engine-report.md", "The report ↗" } " · " a { href: "{REPOSITORY}/blob/main/tools/second_engine.py", "The translator ↗" } " · " NavLink { to: format!("{PREFIX}limits/"), "Current limits" } }
        }
    } }
}
#[component]
pub fn Limits() -> Element {
    let limits = &assurance().limits;
    let engine_section = &assurance().engine.method_section;
    let method = section_link("method", engine_section).unwrap_or_default();
    rsx! { div { class: "container page assurance-page",
        PageHeading { eyebrow: "what is left open", title: "Current limits",
            p { "What the design, and the checks made on it, leave open, quoted from the records that hold each: the adversarial audit's open findings, the declared defects and each chapter's account of what it cannot settle. What was checked, and where each check stops, is in " NavLink { to: format!("{PREFIX}second-engine/"), "the second engine" } " and the method's section " a { href: "{method}", "{engine_section}" } "." }
        }
        section { id: "open-findings", class: "q-card pad", aria_label: "Open findings of the adversarial audit",
            h2 { "Open findings of the adversarial audit" }
            p { "The audit reads the source through the lenses of several disciplines. It is the project auditing its own repository, not an independent review. Each finding still open, with the claim it withholds:" }
            ol { class: "open-findings",
                for finding in limits.findings.iter() {
                    li { "data-finding": "{finding.lens}",
                        p { strong { "{finding.lens}. " } "{finding.finding}" }
                        p { class: "withholds", "What it withholds: {finding.withholds}" }
                        p { span { class: "q-badge", "{finding.disposition}" } }
                    }
                }
            }
            p { a { href: "{REPOSITORY}/blob/main/book-1/source/adversarial-audit.md", "The audit ↗" } }
        }
        section { id: "declared-defects", class: "q-card pad", aria_label: "Declared defects",
            h2 { "Declared defects" }
            if limits.defects.is_empty() {
                p { "No declared defect is active: no test expects a known defect to reproduce." }
            } else {
                ul { for defect in limits.defects.iter() { li { code { "{defect.file}:{defect.line}" } " · {defect.reason}" } } }
            }
            p { "Repaired defects are recorded outside the reading pages, which describe the current design: in the " a { href: "{REPOSITORY}/blob/main/book-1/source/resolution-receipts.md", "resolution receipts ↗" } ", the " a { href: "{REPOSITORY}/tree/main/book-1/appendix/decisions", "decision records ↗" } " and the " a { href: "{REPOSITORY}/commits/main", "repository's history ↗" } "." }
        }
        section { id: "chapter-limits", class: "q-card pad", aria_label: "What each chapter cannot settle",
            h2 { "What each chapter cannot settle" }
            p { "Each chapter that states its own limits does so in a section with that name, quoted here without its pointers onward." }
            for chapter in limits.chapters.iter() {{
                let stem = chapter.file.trim_start_matches("book-1/").trim_end_matches(".md").to_owned();
                let link = section_link(&stem, "What this cannot settle").unwrap_or_default();
                rsx! { article { class: "chapter-limit", id: "chapter-{chapter.number}", "data-chapter": "{chapter.number}",
                    h3 { a { href: "{link}", "Chapter {chapter.number}: {chapter.title}" } }
                    blockquote { for paragraph in chapter.paragraphs.iter() { p { "{paragraph}" } } }
                } }
            }}
        }
    } }
}
#[component]
pub fn Design() -> Element {
    let doc = design();
    rsx! { div { class: "container page design-page",
        PageHeading { eyebrow: "the whole design", title: "{doc.title}",
            p { "{doc.note}" }
        }
        for section in doc.sections.iter() {
            section { class: "q-card pad", aria_label: "{section.heading}",
                h2 { "{section.heading}" }
                if let Some(note) = &section.note {
                    p { "{note}" if let Some(number) = section.chapter { " Argued in " a { href: "{chapter(number).path}", "Chapter {number}" } "." } }
                }
                for group in section.groups.iter() {
                    if let Some(heading) = &group.heading { h3 { class: "design-commitment", "{heading}" } }
                    ul { class: "design-lines",
                        for line in group.lines.iter() {{
                            let chapter_path = chapter(line.chapter).path.clone();
                            rsx! { li { "data-article": "{line.article}", "data-chapter": "{line.chapter}",
                                span { "{line.text}" }
                                " "
                                span { class: "design-source",
                                    a { href: "{PREFIX}constitution/#article-{line.article}", "Article {line.article}" }
                                    " · "
                                    a { href: "{chapter_path}", "Chapter {line.chapter}" }
                                }
                            } }
                        }}
                    }
                }
            }
        }
        nav { class: "q-card pad", aria_label: "Where next",
            h2 { "Where next" }
            ul {
                li { NavLink { to: format!("{PREFIX}constitution/"), "Every article in full" } }
                li { NavLink { to: format!("{PREFIX}read/"), "The book map" } }
                li { NavLink { to: format!("{PREFIX}limits/"), "What the design and its checks leave open" } }
            }
        }
    } }
}
