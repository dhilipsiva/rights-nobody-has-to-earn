// SPDX-License-Identifier: MIT OR Apache-2.0
//! Static generation through Dioxus SSR, with matching Dioxus hydration markers.
use dioxus::prelude::*;
use rights_book_ui::{App, data::*};
use std::{fs, path::Path};
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn write(path: &Path, data: impl AsRef<[u8]>) -> std::io::Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, data)
}
fn companion_markdown(route: &str) -> String {
    let mut text = format!(
        "# {}\n\nBy dhilipsiva. Prose: CC BY 4.0.\n\nCanonical: {ORIGIN}{route}\n\n",
        route_title(route)
    );
    match route.strip_prefix(PREFIX).unwrap_or("") {
        "read/" => {
            for p in &book().pages {
                text += &format!("- [{}]({}index.md)", p.label, p.canonical);
                if !p.summary.is_empty() {
                    text += &format!(" — {}", p.summary);
                }
                text += "\n";
            }
        }
        "start/" => {
            let doc = start();
            for paragraph in &doc.panel {
                text += &format!("{paragraph}\n\n");
            }
            text += &format!("## A guided first run\n\n{}\n\n", doc.guided.note);
            for step in &doc.guided.steps {
                text += &format!("1. {} `? {}` {}\n", step.text, step.query, step.verdict);
            }
            text += &format!(
                "\n[Run Nell's fork live]({ORIGIN}{PREFIX}#fork={}) · [The tests these answers come from]({REPOSITORY}/blob/main/{})\n\n## Forks and joints\n\n",
                doc.guided.fork, doc.guided.pins
            );
            for paragraph in &doc.explainer.text {
                text += &format!("{paragraph}\n\n");
            }
            text += &format!(
                "- [Nell's fork]({ORIGIN}{PREFIX}#fork={})\n- [The measured joint]({ORIGIN}{PREFIX}#joint={})\n",
                doc.explainer.fork, doc.explainer.joint
            );
        }
        "second-engine/" => {
            let e = &assurance().engine;
            let agreement = if e.differences == 0 {
                "Every answer agrees with the verdict its pin records.".to_owned()
            } else {
                format!(
                    "{} answers differ from the verdict their pins record.",
                    e.differences
                )
            };
            text += &format!(
                "The constitution's rules were translated, statement by statement, into the input language of clingo {}, a separate answer set solver: {} clingo statements. The cases below were replayed through it, {} cases and {} queries, each answer compared with the verdict its pin records. {agreement}\n\n## Where it stops\n\n> {}\n\nFrom the method's section \u{201c}{}\u{201d}.\n\n## What the translation keeps\n\n",
                e.clingo,
                thousands(e.statements),
                e.cases.len(),
                thousands(e.queries),
                e.limit,
                e.method_section
            );
            for item in &e.keeps {
                text += &format!("- {item}\n");
            }
            text += "\n## What the comparison leaves out\n\n";
            for item in &e.leaves_out {
                text += &format!("- {item}\n");
            }
            text += "\n## The cases replayed\n\n| Case | Queries | Answers that differ |\n|---|---|---|\n";
            for case in &e.cases {
                text += &format!(
                    "| `{}` | {} | {} |\n",
                    case.id, case.queries, case.differences
                );
            }
            text += &format!(
                "\n[The report]({REPOSITORY}/blob/main/book-1/source/measurements/second-engine-report.md) · [The translator]({REPOSITORY}/blob/main/tools/second_engine.py)\n"
            );
        }
        "limits/" => {
            let l = &assurance().limits;
            text += "What the design, and the checks made on it, leave open, quoted from the records that hold each.\n\n## Open findings of the adversarial audit\n\n";
            for f in &l.findings {
                text += &format!(
                    "1. **{}.** {} What it withholds: {} ({})\n",
                    f.lens, f.finding, f.withholds, f.disposition
                );
            }
            text += &format!(
                "\n[The audit]({REPOSITORY}/blob/main/book-1/source/adversarial-audit.md)\n\n## Declared defects\n\n"
            );
            if l.defects.is_empty() {
                text +=
                    "No declared defect is active: no test expects a known defect to reproduce.\n";
            }
            for d in &l.defects {
                text += &format!("- `{}:{}` · {}\n", d.file, d.line, d.reason);
            }
            text += &format!(
                "\nRepaired defects are recorded in the [resolution receipts]({REPOSITORY}/blob/main/book-1/source/resolution-receipts.md), the [decision records]({REPOSITORY}/tree/main/book-1/appendix/decisions) and the repository's history.\n\n## What each chapter cannot settle\n\n"
            );
            for c in &l.chapters {
                text += &format!("### Chapter {}: {}\n\n", c.number, c.title);
                for paragraph in &c.paragraphs {
                    text += &format!("> {paragraph}\n>\n");
                }
                text += "\n";
            }
        }
        "search/" => {
            text += "Search runs locally in the interactive reader. Reading and browser Find remain available without JavaScript.\n";
        }
        "constitution/" => {
            let doc = articles();
            text += &format!("{}\n\n", doc.note);
            for (index, part) in doc.parts.iter().enumerate() {
                text += &format!("## {part}\n\n");
                for a in doc.articles.iter().filter(|a| a.part == index + 1) {
                    text += &format!("### Article {}. {}\n\n", a.number, a.title);
                    for (i, clause) in a.text.iter().enumerate() {
                        text += &format!("{}. {clause}\n", i + 1);
                    }
                    text += &format!(
                        "\nRule families: {}. Tested by: {}.\n\n",
                        a.families.join(", "),
                        a.pins.join("; ")
                    );
                }
            }
        }
        "cases/" => {
            use rights_book_ui::game_state::{chapters_of, game};
            let doc = chapter_cases();
            text += &format!("{}\n\n", doc.note);
            for page in book().pages.iter().filter(|p| p.number.is_some()) {
                let n = page.number.unwrap();
                let runs = |owner: &str, primary: u8| chapters_of(owner, primary).contains(&n);
                if !game().scenarios.iter().any(|f| runs(&f.id, f.chapter))
                    && !game()
                        .joints
                        .iter()
                        .any(|j| j.measured && runs(&j.id, j.chapter))
                    && !doc.moved.iter().any(|m| m.chapter == n)
                {
                    continue;
                }
                text += &format!("## {}\n\n", page.label);
                for f in game()
                    .scenarios
                    .iter()
                    .filter(|f| chapters_of(&f.id, f.chapter).contains(&n))
                {
                    text += &format!(
                        "- [{}]({ORIGIN}{PREFIX}#fork={}) · {}\n",
                        f.title, f.id, f.role
                    );
                }
                for j in game()
                    .joints
                    .iter()
                    .filter(|j| j.measured && chapters_of(&j.id, j.chapter).contains(&n))
                {
                    text += &format!(
                        "- [{}]({ORIGIN}{PREFIX}#joint={}) · compared live with “{}”\n",
                        j.title, j.id, j.off_label
                    );
                }
                for m in doc.moved.iter().filter(|m| m.chapter == n) {
                    text += &format!("\n### {}\n\n{}\n\n", m.title, m.text);
                    for link in &m.links {
                        text += &format!("- [{}]({})\n", link.label, repository_url(&link.path));
                    }
                }
                text += "\n";
            }
        }
        _ => {
            text += "Pick a person. Walk their forks.\n\nThe game starts the local engine automatically. Each move executes a complete isolated record against the pinned constitution. There are no embedded game verdicts. Gameplay requires JavaScript; the complete reader remains available without it.\n\nAuthored costs and discussion are identified separately from engine responses. Saved and shared history is replayed live before it contributes to the tally.\n\n";
            for f in &rights_book_ui::game_state::game().scenarios {
                text += &format!("## {}\n\n{}\n\n", f.title, f.role);
                for step in &f.steps {
                    text += &format!("- {}\n", step.label);
                    for query in &step.queries {
                        text += &format!("  - `? {}` {}\n", query.text, query.gloss);
                    }
                }
                text += &format!(
                    "\n[Read chapter {}]({ORIGIN}{}) · [Source]({REPOSITORY}/blob/main/{})\n\nAuthored cost: {}\n\n",
                    f.chapter,
                    chapter(f.chapter).path,
                    f.source,
                    f.cost.text
                );
            }
            text += &format!(
                "[Complete reader]({ORIGIN}{PREFIX}read/) · [Game data]({ORIGIN}{PREFIX}game.json) · [Versioned executable inputs]({ORIGIN}{PREFIX}cases.json)\n"
            );
        }
    }
    text
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist/rights-nobody-has-to-earn".into());
    let root = Path::new(&output);
    let hydration = dioxus_fullstack_core::HydrationContext::default()
        .serialized()
        .data;
    let all_routes = routes();
    for route in all_routes
        .iter()
        .cloned()
        .chain(std::iter::once(format!("{PREFIX}404/")))
    {
        let mut dom = VirtualDom::new(App);
        dom.provide_root_context(route.clone());
        dom.rebuild_in_place();
        let rendered = dioxus_ssr::pre_render(&dom);
        let page = book().pages.iter().find(|p| p.path == route);
        let title = format!("{} · {}", route_title(&route), book().title);
        let description = page.map(|p| p.description.clone()).unwrap_or_else(|| match route.strip_prefix(PREFIX).unwrap_or("") {
            "" => "Pick a person, walk their forks, and execute each record locally against the full constitutional design. Read the complete Book 1.",
            "map/" => "26 questions about personhood, the unconditional floor, ordinary life and public power, with a glossary and routes into Book 1.",
            "walkthrough/food-delivery/" => "Follow six delivery records. Explore why a receipt and an independent witness derive a conclusion, and what that conclusion cannot prove.",
            "about/" => "Authorship, sources, licensing, privacy and the limits of the executable Book 1 companion.",
            "read/" => "Read the complete Book 1 in manuscript order: epigraph, opening note, every chapter and Part opening case, the optional method, and a map, glossary and index.",
            "search/" => "Search the complete text of Book 1 locally on your device.",
            "constitution/" => "The constitution Book 1 describes, as numbered plain-language articles traced to the rules and tests that implement them.",
            "cases/" => "Each chapter's cases, run on your own device against the pinned constitution, and where the material the chapters no longer carry now lives.",
            "start/" => "What the companion is, what a fork and a joint are, and a guided first run of Nell's fork checked against Chapter 1's tests.",
            "second-engine/" => "The constitution's rules replayed in a second engine, clingo: what the translation keeps, what the comparison leaves out, every case and where the cross-check stops.",
            "limits/" => "What the design and its checks leave open: the adversarial audit's open findings, the declared defects and each chapter's account of what it cannot settle.", _ => "This page does not exist in Book 1.",
        }.into());
        let is_404 = route.ends_with("/404/");
        let noindex = route.ends_with("/search/") || is_404;
        let schema = if let Some(page) = page {
            serde_json::json!({"@context":"https://schema.org", "@type": if page.number.is_some() {"Chapter"} else {"CreativeWork"}, "name":page.title, "author":{"@type":"Person","name":book().author}, "url":page.canonical, "isPartOf":{"@type":"Book","name":book().title,"url":format!("{ORIGIN}{PREFIX}read/")}, "position":page.order+1, "license":"https://creativecommons.org/licenses/by/4.0/", "inLanguage": if page.stem == "epigraph" { vec!["ta","en"] } else {vec!["en"]}})
        } else {
            serde_json::json!({"@context":"https://schema.org","@type":"Book","name":book().title,"author":{"@type":"Person","name":book().author},"url":format!("{ORIGIN}{PREFIX}read/"),"license":"https://creativecommons.org/licenses/by/4.0/","inLanguage":"en"})
        };
        let robots = if noindex {
            "<meta name=\"robots\" content=\"noindex,follow\">"
        } else {
            ""
        };
        let html = format!(
            r#"<!doctype html>
<html lang="en" data-theme="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<link rel="icon" href="data:,"><title>{title}</title><meta name="description" content="{description}"><meta name="author" content="dhilipsiva">{robots}
<link rel="canonical" href="{ORIGIN}{route}"><meta property="og:type" content="book"><meta property="og:title" content="{title}"><meta property="og:description" content="{description}"><meta property="og:url" content="{ORIGIN}{route}">
<link rel="alternate" type="text/markdown" href="{route}index.md"><link rel="describedby" href="{PREFIX}llms.txt"><link rel="sitemap" type="application/xml" href="{PREFIX}sitemap.xml">
<link rel="stylesheet" href="{PREFIX}assets/fonts.css"><link rel="stylesheet" href="{PREFIX}assets/quine.css"><link rel="stylesheet" href="{PREFIX}assets/app.css">
<script type="application/ld+json">{schema}</script></head>
<body><div id="main">{rendered}</div><script src="{PREFIX}assets/platform.js"></script>
<script>window.hydrate_queue=[];window.initial_dioxus_hydration_data={hydration};</script>
<script type="module">import init from '{PREFIX}assets/app/rights_book_ui.js'; init().catch(error => console.error('Reader interactions could not load; static reading remains available.',error));</script>
</body></html>"#,
            title = escape(&title),
            description = escape(&description),
            schema = schema.to_string().replace('<', "\\u003c"),
            hydration = serde_json::to_string(&hydration)?
        );
        let relative = route.strip_prefix(PREFIX).unwrap();
        let dest = if is_404 {
            root.join("404.html")
        } else {
            root.join(relative).join("index.html")
        };
        write(&dest, html)?;
        if !is_404 {
            let md = page.map(|p| format!("{}\n---\n\nBy dhilipsiva. Prose: CC BY 4.0.\n\nCanonical: {}\n\nSource: {}\n", p.markdown, p.canonical, p.source)).unwrap_or_else(|| companion_markdown(&route));
            write(&root.join(relative).join("index.md"), md)?;
        }
    }
    let sitemap = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{}</urlset>\n",
        all_routes
            .iter()
            .filter(|r| !r.ends_with("/search/"))
            .map(|r| format!("<url><loc>{ORIGIN}{r}</loc></url>"))
            .collect::<String>()
    );
    write(&root.join("sitemap.xml"), sitemap)?;
    let content = serde_json::json!({"title":book().title,"author":book().author,"license":book().license,"word_count":book().word_count,
        "scope":"Current reading manuscript. Derived conclusions are not observed events. The opening note, Part V and method are non-derived channels.",
        "pages": book().pages.iter().map(|p| serde_json::json!({"title":p.title,"number":p.number,"part":p.part,"order":p.order,"canonical":p.canonical,"markdown":format!("{}index.md",p.canonical),"source":p.source,"sections":p.sections.iter().map(|s|serde_json::json!({"id":s.id,"title":s.title,"level":s.level,"url":format!("{}#{}",p.canonical,s.id)})).collect::<Vec<_>>()})).collect::<Vec<_>>()});
    write(
        &root.join("content.json"),
        serde_json::to_string_pretty(&content)?,
    )?;
    write(
        &root.join("cases.json"),
        include_str!("../../generated/cases.json"),
    )?;
    let mut index = format!(
        "# {}\n\n> Book 1 by dhilipsiva: a constitutional design, complete reader, and selected executable examples.\n\nProse is CC BY 4.0. Formal conclusions concern supplied records, not observed events. The canonical constitution and explicitly labelled isolated counterfactuals are distinct. This index follows the llms.txt discovery proposal; agent adoption is not guaranteed.\n\n## Reader\n\n",
        book().title
    );
    let mut full = index.clone();
    for page in &book().pages {
        index += &format!("- [{}]({}index.md)\n", page.label, page.canonical);
        full += &format!("\n---\n\nCitation: {}\n\n{}", page.canonical, page.markdown);
    }
    index += &format!(
        "\n## Companion and structured data\n\n- [Live execution game]({ORIGIN}{PREFIX}index.md)\n- [Each chapter's runnable cases]({ORIGIN}{PREFIX}cases/index.md)\n- [The constitution in plain language]({ORIGIN}{PREFIX}constitution/index.md)\n- [Authored game data]({ORIGIN}{PREFIX}game.json)\n- [Chapter and section index]({ORIGIN}{PREFIX}content.json)\n- [Versioned executable inputs; no answers]({ORIGIN}{PREFIX}cases.json)\n\n## Optional\n\n- [Full reader text]({ORIGIN}{PREFIX}llms-full.txt)\n"
    );
    write(&root.join("game.json"), include_str!("../../game.json"))?;
    let redirects = serde_json::json!({"version":1,"redirects":[
        {"from":format!("{PREFIX}map/"),"to":PREFIX,"status":301},
        {"from":format!("{PREFIX}walkthrough/food-delivery/"),"to":PREFIX,"status":301},
        {"from":format!("{PREFIX}about/"),"to":format!("{PREFIX}#dossier"),"status":301}
    ]});
    write(
        &root.join("redirects.json"),
        serde_json::to_string_pretty(&redirects)?,
    )?;
    write(&root.join("llms.txt"), index)?;
    write(&root.join("llms-full.txt"), full)?;
    println!(
        "Generated {} complete routes, Markdown counterparts, discovery files and 404.html in {}",
        all_routes.len(),
        root.display()
    );
    Ok(())
}
