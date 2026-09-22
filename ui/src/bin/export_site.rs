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
        "map/" => {
            text += "Questions are routes into the manuscript, not separate constitutional rules. Visits are saved locally.\n\n";
            for q in &companion().questions {
                text += &format!("- [{}]({ORIGIN}{})\n", q.text, question_path(q));
            }
            text += "\n## Glossary\n\n";
            for term in &companion().terms {
                text += &format!("### {}\n\n{}\n\n", term.name, term.definition);
            }
        }
        "walkthrough/food-delivery/" => {
            text += "Derived is not observed. Initial results are precomputed against the full constitution, not a claim of a freshly verified book.\n\n";
            for (i, step) in companion().steps.iter().enumerate() {
                text += &format!(
                    "## {}. {}\n\n{}\n\n{}\n\n",
                    i + 1,
                    step.heading,
                    step.lead,
                    step.note
                );
                let case = &cases()
                    .cases
                    .iter()
                    .find(|c| c.scenario.id == format!("delivery-{i}"))
                    .unwrap();
                text += &format!("```nibli\n{}\n```\n\n", case.scenario.record.join("\n"));
                for v in &case.outcome.verdicts {
                    text += &format!("- `{}`: **{}** (precomputed)\n", v.query, v.status);
                }
                text += "\n";
            }
            text += &format!(
                "Counterfactual details: [cases.json]({ORIGIN}{PREFIX}cases.json). The isolated edit removes only the food independence condition.\n\nSelf-check: the canonical rule does not derive food from a provider witnessing its own delivery.\n"
            );
        }
        "read/" => {
            for p in &book().pages {
                text += &format!("- [{}]({}index.md)\n", p.label, p.canonical);
            }
        }
        "search/" => {
            text += "Search runs locally in the interactive reader. All search terms must occur in a chapter. Use the [complete contents](../read/) for static navigation. Search results are excluded from indexing.\n";
        }
        "about/" => {
            text += &format!(
                "A constitutional design offered for discussion and criticism, not an account of a society operating these arrangements. By dhilipsiva. Book 2 concerns operation and transition and stays inactive until Book 1’s release decision.\n\nThe full manuscript lives in [the repository]({REPOSITORY}/tree/main/book-1). This companion cannot override the formal source.\n\n## Reasoning and privacy\n\nPrecomputed examples are executed during the UI build. Run locally executes one example on your device using the full constitution. Neither label claims the full verifier or contradiction scanner has just run. FALSE means not derivable from the supplied record; TRUE does not establish an observed event. Refusal and incomplete execution are separate.\n\nThe host serves static files. Engine resources download only when requested. Reasoning, search and saved progress stay on this device. External source links open their respective hosts.\n\n## Authorship, licences and sources\n\nThe author speaks in the opening, Part V and method; the derived chapters project the constitution. Case names are formal test records. Prose: CC BY 4.0. Code: MIT OR Apache-2.0. Constitution: CC0. Fonts: SIL OFL 1.1. See the [licence map]({REPOSITORY}/blob/main/LICENSING.md), [font sources]({ORIGIN}{PREFIX}assets/fonts/README.md), and [contribution guide]({REPOSITORY}/blob/main/CONTRIBUTING.md).\n\nAgent discovery uses the [llms.txt proposal](https://llmstxt.org/), with no guaranteed adoption, MCP server, chatbot or execution API.\n"
            );
        }
        _ => {
            text += "What must a society provide for a person who can offer it nothing in return?\n\nFood, shelter, care, learning, safety, expression, belief and company form an unconditional floor. Being a person establishes standing and a claim to essentials. Employment, wealth, citizenship, family and good behaviour are not entrance requirements.\n\n## Owed is not delivered\n\nNell’s starting record is `born(Nell).` All eight duties follow; none of the delivery conclusions follow. A food receipt, an authorised independent witness and a matching food observation derive `eats(Nell)`. Removing those entries restores the original result. No other delivery result changes. Belief has no delivery route by design.\n\nNell is a test record. FALSE means not derivable, not real-world absence. TRUE is a derived conclusion, not an observation. These examples are precomputed until Run locally succeeds.\n\n";
            text += &format!(
                "[Read Book 1]({ORIGIN}{PREFIX}read/index.md) · [26 questions]({ORIGIN}{PREFIX}map/index.md) · [Delivery walkthrough]({ORIGIN}{PREFIX}walkthrough/food-delivery/index.md)\n"
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
            "" => "What must a society provide for someone who can offer it nothing in return? Read Book 1 and explore its executable constitutional design.",
            "map/" => "26 questions about personhood, the unconditional floor, ordinary life and public power, with a glossary and routes into Book 1.",
            "walkthrough/food-delivery/" => "Follow six delivery records. Explore why a receipt and an independent witness derive a conclusion, and what that conclusion cannot prove.",
            "about/" => "Authorship, sources, licensing, privacy and the limits of the executable Book 1 companion.",
            "read/" => "Read the complete Book 1 in manuscript order: epigraph, opening note, 31 chapters and optional method.",
            "search/" => "Search the complete text of Book 1 locally on your device.", _ => "This page does not exist in Book 1.",
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
        "\n## Companion and structured data\n\n- [Questions and glossary]({ORIGIN}{PREFIX}map/index.md)\n- [Delivery walkthrough]({ORIGIN}{PREFIX}walkthrough/food-delivery/index.md)\n- [Sources and licences]({ORIGIN}{PREFIX}about/index.md)\n- [Chapter and section index]({ORIGIN}{PREFIX}content.json)\n- [Structured cases]({ORIGIN}{PREFIX}cases.json)\n\n## Optional\n\n- [Full reader text]({ORIGIN}{PREFIX}llms-full.txt)\n"
    );
    write(&root.join("llms.txt"), index)?;
    write(&root.join("llms-full.txt"), full)?;
    println!(
        "Generated {} complete routes, Markdown counterparts, discovery files and 404.html in {}",
        all_routes.len(),
        root.display()
    );
    Ok(())
}
