// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{NavLink, Note, PageHeading, Session, data::*, reasoning::Reasoning};
use dioxus::prelude::*;
const FLOOR: [(&str, &str, &str); 8] = [
    ("Food", "Eats", "eats"),
    ("Shelter", "Dwell", "dwell"),
    ("Care", "Healthy", "healthy"),
    ("Learning", "Learn", "learn"),
    ("Safety", "Secure", "secure"),
    ("Expression", "Expresses", "expresses"),
    ("Belief", "Believe", "believe"),
    ("Company", "Meets", "meets"),
];
#[component]
pub fn Floor() -> Element {
    let mut evidence = use_signal(|| false);
    let mut result = use_signal(|| cases().cases[0].outcome.clone());
    let case_id = if evidence() {
        "floor-evidence"
    } else {
        "floor"
    };
    rsx! {
        section { class: "hero", div { class: "hero__grid" } div { class: "container hero__inner",
            div {
                span { class: "eyebrow", "// book 1 · a companion" }
                h1 { "What must a society provide for a person who can offer it " em { "nothing" } " in return?" }
                p { class: "hero__tagline", "Food, shelter, care, learning, safety, expression, belief and company form a floor — entitlements that neither private dependence nor public punishment may withdraw." }
                p { class: "hero__intro", "Being a person establishes standing in law and a claim to essentials. Employment, wealth, citizenship, family and good behaviour are not entrance requirements. Explore the design, read the book, and run selected examples on your device." }
                div { class: "actions",
                    NavLink { to: format!("{PREFIX}map/"), class: "q-btn q-btn--primary", "Start with a question →" }
                    NavLink { to: format!("{PREFIX}read/00-opening-note/"), class: "q-btn q-btn--secondary", "Opening note →" }
                }
                div { class: "measurements", span { strong { "chapters " } "31 + method" } span { strong { "words " } "{book().word_count}" } span { strong { "licence " } "CC BY 4.0" } }
            }
            Reasoning { key: "{case_id}", case_id, onresult: move |value| result.set(value) }
        } }
        section { class: "section--tight", id: "floor", div { class: "container",
            span { class: "eyebrow", "// the floor · nell" }
            h2 { class: "section-title", "Owed is not the same as delivered" }
            p { class: "section-intro", "Personhood establishes each duty. Delivery needs evidence. Nell’s starting record says only born(Nell): the whole floor is owed, and none is shown delivered." }
            p { NavLink { to: chapter(5).path.clone(), "Chapter 5 · Whether It Arrived →" } }
            div { class: "q-card",
                table { class: "floor-table", aria_label: "Nell’s entitlements and delivery conclusions",
                    thead { tr { th { scope: "col", "Essential" } th { scope: "col", class: "floor-pred", "In the rules" } th { scope: "col", "Owed" } th { scope: "col", "Delivered" } } }
                    tbody { for (name, item, pred) in FLOOR {
                        tr { td { "{name}" } td { class: "floor-pred", code { "owe(State, {item}, Nell)" } code { "{pred}(Nell)" } }
                            td { span { class: "q-badge", if result.read().verdicts.iter().any(|v|v.query==format!("owe(State, {item}, Nell).") && v.status=="TRUE") { "true" } else { "not derivable" } } }
                            td { class: if result.read().verdicts.iter().any(|v| v.query == format!("{pred}(Nell).") && v.status == "TRUE") { "true" } else { "false" },
                                if result.read().verdicts.iter().any(|v| v.query == format!("{pred}(Nell).") && v.status == "TRUE") { "true" }
                                else if pred == "believe" { "no route · by design" }
                                else { "not derivable" }
                            }
                        }
                    } }
                }
                div { class: "floor-footer",
                    button { class: "q-btn q-btn--secondary", aria_pressed: evidence(), onclick: move |_| {
                        let next = !evidence(); evidence.set(next);
                        result.set(cases().cases[usize::from(next)].outcome.clone());
                    }, if evidence() { "Remove the evidence ↻" } else { "Supply food evidence →" } }
                    p { class: "section-intro", if evidence() { "A receipt, an authorised witness, and the witness’s matching observation derive eats(Nell). Food evidence does not establish shelter. This conclusion does not establish that a child was fed." } else { "Supply a receipt and an independent witness for food. Only that delivery conclusion changes. The result is precomputed until you run this record locally." } }
                }
            }
            Note { "Nell is a test record. FALSE means not derivable from the supplied record; it does not establish real-world deprivation. TRUE does not establish real-world delivery. Derived is not observed." }
        } }
        section { class: "section--tight", div { class: "container two-column",
            div { span { class: "eyebrow", "// or by question" } h2 { class: "section-title", "Questions a person might actually ask" }
                p { class: "section-intro", "The reader’s map offers 26 ways into the book, from personhood and care to public power and its limits. Each question leads to the chapters that answer it." }
                NavLink { to: format!("{PREFIX}map/"), "All questions, by part →" }
            }
            div { class: "q-card pad",
                for index in [3_usize,0,4,24,19] { {
                    let q = &companion().questions[index];
                    rsx! { NavLink { to: question_path(q), visit: q.id.clone(), class: "question-row", span { "→" } span { "{q.text}" } } }
                } }
            }
        } }
    }
}
#[component]
pub fn Map() -> Element {
    let mut session = use_context::<Session>();
    let visited = session.preferences.read().visited.clone();
    let next = companion()
        .questions
        .iter()
        .find(|q| !visited.contains(&q.id));
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "reader’s map", title: "Every question, in the order the book asks them",
            p { "The chapters follow the order a person meets the design. A record is what the rules are told; a derived conclusion follows from that record and the rules; the floor is what every person is owed." }
        }
        div { class: "two-column", div { class: "stack",
            for part in 1..=4_u8 {
                section { class: "q-card question-group",
                    h2 { "{book().pages.iter().find(|p| p.part.is_some() && p.number == Some([1,8,16,24][part as usize-1])).unwrap().part.as_ref().unwrap()}" }
                    for q in companion().questions.iter().filter(|q| q.part == part) {
                        NavLink { to: question_path(q), visit: q.id.clone(), class: "question-row",
                            span { class: "mark", aria_label: if visited.contains(&q.id) { "Visited" } else { "Unvisited" }, if visited.contains(&q.id) { "✓" } else { "○" } }
                            span { "{q.text}" }
                            span { class: "refs", {q.chapters.iter().map(u8::to_string).collect::<Vec<_>>().join(" · ")} }
                        }
                    }
                }
            }
            div { class: "q-card pad", span { class: "eyebrow", "// part v · outside the graph" }
                p { "Why choose these rules? The author’s argument considers valuation, rotation, coercion, capture and the state." }
                NavLink { to: chapter(31).path.clone(), "The Five Joints →" }
            }
            Note { "A chapter’s presence means it addresses a subject. It does not establish that the institutions or services described operate in the world." }
        }
        aside { class: "stack side",
            div { class: "q-card pad marked", span { class: "eyebrow", "// your route" }
                p { class: "progress-count", "{visited.len()} " small { "of 26 questions visited" } }
                progress { value: visited.len().to_string(), max: "26", aria_label: "Questions visited" }
                p { "Progress is stored on this device. Visiting a question is not a test of understanding." }
                div { class: "actions",
                    if let Some(q) = next { NavLink { to: question_path(q), visit: q.id.clone(), class: "q-btn q-btn--primary", "Continue →" } }
                    button { class: "q-btn q-btn--ghost", onclick: move |_| { session.preferences.write().visited.clear(); session.save(); }, "Clear visits" }
                }
                p { NavLink { to: format!("{PREFIX}read/"), "Complete contents →" } }
            }
            section { class: "q-card pad glossary", h2 { class: "card-title", "Glossary" }
                for term in &companion().terms {
                    details { summary { "{term.name}" } p { "{term.definition}" }
                        NavLink { to: term.chapter.map(|n| chapter(n).path.clone()).unwrap_or_else(|| format!("{PREFIX}read/method/")), "Read in context →" }
                    }
                }
            }
        } }
    } }
}
#[component]
pub fn Walkthrough() -> Element {
    let mut session = use_context::<Session>();
    let step_index = session.preferences.read().step;
    let mut cf = use_signal(|| false);
    let mut answer = use_signal(|| None::<bool>);
    let step = &companion().steps[step_index];
    let case_id = if step_index == 4 && cf() {
        "delivery-counterfactual".into()
    } else {
        format!("delivery-{step_index}")
    };
    let mut go = move |index: usize| {
        cf.set(false);
        answer.set(None);
        session.preferences.write().step = index.min(5);
        session.visit("q5");
    };
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "chapter 5 · walkthrough", title: "Does a receipt prove I was fed?",
            p { "Follow six records through one delivery rule. The ordinary route needs a receipt and independent evidence about the same recipient, item and scope." }
        }
        div { class: "walk-grid",
            aside { nav { aria_label: "Walkthrough steps", ol { class: "step-list",
                for (i, item) in companion().steps.iter().enumerate() {
                    li { button { class: "step-link", aria_current: if i == step_index { Some("step") } else { None }, onclick: move |_| go(i),
                        span { class: "step-num", "{i+1}" } span { "{item.title}" }
                    } }
                }
            } }
                NavLink { to: chapter(5).path.clone(), "Read the chapter →" }
                p { button { class: "q-btn q-btn--ghost", onclick: move |_| go(0), "Reset walkthrough" } }
            }
            div { class: "stack",
                section { class: "q-card step-panel",
                    div { class: "pad",
                        span { class: "eyebrow", "// step {step_index+1} of 6" }
                        h2 { "{step.heading}" } p { "{step.lead}" }
                        div { class: "diagram", aria_label: "The distinction between an event and a derived conclusion", span { "event in the world" } span { "→ supplied report" } span { "→ rule" } span { "→ conclusion" } }
                        Reasoning { key: "{case_id}", case_id, onresult: move |_| {} }
                        p { "{step.note}" }
                        if step_index == 4 {
                            div { class: "cf-panel",
                                strong { if cf() { "Counterfactual: food independence condition removed" } else { "Canonical rule: witness must differ from provider" } }
                                p { "Only the declared food-rule edit is applied to an isolated copy. The shelter rule keeps its independence condition." }
                                button { class: "q-btn q-btn--secondary", aria_pressed: cf(), onclick: move |_| cf.toggle(), if cf() { "Restore the canonical rule ↻" } else { "Remove the food independence condition →" } }
                            }
                        }
                        details { summary { "Show the canonical food rule" }
                            pre { class: "rule-code", "all $p: all $item: all $src: all $w:\n  receives($p, $item, $src) &\n  authorized($w, DeliveryWitness, $p) &\n  observe($w, $item, $p, FoodScope) &\n  ~($w = $src) -> eats($p)." }
                        }
                        if step_index == 5 {
                            section { class: "self-check", h3 { class: "card-title", "Check your understanding" }
                                p { "Under the canonical rule, does food derive when the witness is also its provider?" }
                                div { class: "actions", button { class: "q-btn q-btn--secondary", aria_pressed: answer() == Some(true), onclick: move |_| answer.set(Some(true)), "Yes" } button { class: "q-btn q-btn--secondary", aria_pressed: answer() == Some(false), onclick: move |_| answer.set(Some(false)), "No" } }
                                if let Some(yes) = answer() { p { role: "status", if yes { "The answer is no. " } else { "Yes — that’s right: it does not. " } "The provider cannot supply the independent attestation. Step 5 lets you inspect both the canonical result and the isolated counterfactual." } }
                            }
                        }
                    }
                    div { class: "card-footer",
                        button { class: "q-btn q-btn--ghost", disabled: step_index == 0, onclick: move |_| go(step_index.saturating_sub(1)), "← Back" }
                        span { class: "eyebrow", "record {step_index+1} / 6" }
                        if step_index < 5 { button { class: "q-btn q-btn--primary", onclick: move |_| go(step_index+1), "Next →" } }
                        else { NavLink { to: format!("{PREFIX}map/"), class: "q-btn q-btn--primary", "Back to the map →" } }
                    }
                }
                Note { "Marisol names a formal test case. The rule checks that reports match; it cannot establish that any report is true. A derived conclusion is not an observed event." }
            }
        }
    } }
}
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
            p { NavLink { to: format!("{PREFIX}map/"), "Enter by question →" } }
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
#[component]
pub fn About() -> Element {
    let mut session = use_context::<Session>();
    rsx! { div { class: "container page",
        PageHeading { eyebrow: "about", title: "Plain answers about this book and companion", p { "A worked constitutional design, a complete reader, and selected executable examples. This companion cannot override or complete the formal specification." } }
        div { class: "two-column",
            div { class: "q-card pad faq",
                section { h2 { "Is this an existing constitution?" } p { "It is a design offered for discussion and criticism, not an account of a society already operating these arrangements. Book 1 describes the destination. Book 2, What It Would Take, owns operation and transition and remains inactive until Book 1’s release decision." } }
                section { h2 { "What has been checked here?" } p { "The labelled precomputed examples were executed against the full constitution when this UI was built. Run locally repeats one selected scenario on your device. Neither label asserts that every page, every pin, or all contradiction checks have just run." }
                    p { "The full book verifier is a separate tool. Its recorded measurements belong to the repository; they are not live status indicators for this site." }
                    a { href: format!("{REPOSITORY}/blob/main/README.md"), "Repository and verification instructions →" }
                }
                section { h2 { "Why doesn’t FALSE establish ‘no’?" } p { "FALSE means the positive statement is not derivable from the supplied rules and record. It does not establish real-world absence. TRUE is also a formal conclusion, not an observation. Refusal and incomplete execution are different outcomes and are displayed separately." } }
                section { h2 { "What leaves my device?" } p { "The web reader downloads pages, fonts and application assets from its host. Run locally additionally downloads the engine and full constitution. Reasoning, search terms, and saved progress are processed locally; this application sends none of them to a service. External source links open their respective sites. Hosting request logs are controlled by the website." } }
                section { h2 { "Who is speaking?" } p { "dhilipsiva. The opening note, Part V and the optional method carry the author’s argument. The derived chapters state consequences of the formal design. Names in formal cases are test records, not interviews or biographies." } }
                section { h2 { "How do I cite or reuse a passage?" } p { "Each chapter has a stable page URL, heading fragments, a visible Markdown link, and its manuscript source. Cite the title, author, chapter and section, and identify an adaptation. Prose is CC BY 4.0; the constitution is CC0; code is MIT OR Apache-2.0. The source registry carries its own licensing information." } }
                section { h2 { "Can agents read the book?" } p { "The agent index links to static Markdown and JSON. llms.txt follows a discovery proposal; adoption is not guaranteed. No chatbot, MCP server, or agent execution API is provided." }
                    div { class: "actions", a { href: public_url(format!("{PREFIX}llms.txt")), "llms.txt" } a { href: public_url(format!("{PREFIX}content.json")), "content.json" } a { href: public_url(format!("{PREFIX}cases.json")), "cases.json" } }
                }
                section { h2 { "How do I object or contribute?" } p { "Corrections, objections, evidence and proposed additions are welcome. A report does not require formal tooling." }
                    div { class: "actions", a { href: format!("{REPOSITORY}/blob/main/CONTRIBUTING.md"), "Contribution guide →" } a { href: format!("{REPOSITORY}/issues"), "Report a problem →" } }
                }
            }
            aside { class: "stack side",
                div { class: "q-card pad marked", h2 { class: "card-title", "Read the complete book" } p { "34 ordered inputs. The epigraph and method remain unnumbered." } NavLink { to: format!("{PREFIX}read/epigraph/"), class: "q-btn q-btn--primary", "Begin reading →" } }
                div { class: "q-card pad", h2 { class: "card-title", "Inspect the sources" }
                    ul { class: "source-list", for (path, label) in [("book-1/source/constitution.nibli", "Constitution"), ("book-1/contents.json", "Reading order"), ("tests/pins/suites.json", "Substantive case inventory"), ("engine.pin", "Pinned Nibli revision"), ("LICENSING.md", "Licence map")] {
                        li { a { href: format!("{REPOSITORY}/blob/main/{path}"), "{label}" } }
                    } }
                    p { class: "term-note", "Engine: {&cases().engine_revision[..12]}" }
                }
                div { class: "q-card pad", h2 { class: "card-title", "Type and licences" }
                    p { "QUINE: IBM Plex Sans, Mono and Serif; Space Grotesk. Tamil: Noto Serif Tamil. All fonts are bundled for offline use under SIL OFL 1.1." }
                    a { href: public_url(format!("{PREFIX}assets/fonts/README.md")), "Font sources and licence files →" }
                }
                div { class: "q-card pad", h2 { class: "card-title", "Saved on this device" } p { "Reset reading position, question visits, walkthrough position, and theme." }
                    button { class: "q-btn q-btn--secondary", onclick: move |_| { session.preferences.set(Default::default()); session.save(); }, "Reset saved preferences" }
                }
            }
        }
        Note { "An exposed failure still needs resolution. A constitutional duty still needs people and resources to fulfil it. Making commitments inspectable cannot make them just." }
    } }
}
