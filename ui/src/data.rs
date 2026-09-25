// SPDX-License-Identifier: MIT OR Apache-2.0
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
pub const PREFIX: &str = "/rights-nobody-has-to-earn/";
pub const ORIGIN: &str = "https://dhilipsiva.dev";
pub const REPOSITORY: &str = "https://github.com/dhilipsiva/rights-nobody-has-to-earn";
/// Public downloads open on the hosted site from desktop, not its asset server.
pub fn public_url(path: impl AsRef<str>) -> String {
    #[cfg(feature = "desktop")]
    {
        format!("{ORIGIN}{}", path.as_ref())
    }
    #[cfg(not(feature = "desktop"))]
    {
        path.as_ref().into()
    }
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub level: u8,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Page {
    pub stem: String,
    pub title: String,
    pub label: String,
    pub number: Option<u8>,
    pub part: Option<String>,
    pub order: usize,
    pub path: String,
    pub canonical: String,
    pub source: String,
    pub description: String,
    pub html: String,
    pub text: String,
    pub markdown: String,
    pub sections: Vec<Section>,
    pub previous: Option<String>,
    pub next: Option<String>,
}
#[derive(Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub license: String,
    pub word_count: usize,
    pub pages: Vec<Page>,
}
#[derive(Clone, Deserialize)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub chapters: Vec<u8>,
    pub part: u8,
}
#[derive(Clone, Deserialize)]
pub struct Term {
    pub name: String,
    pub definition: String,
    pub chapter: Option<u8>,
}
#[derive(Deserialize)]
pub struct Companion {
    pub questions: Vec<Question>,
    pub terms: Vec<Term>,
}
#[derive(Clone, Deserialize)]
pub struct Comparison {
    pub constitution: String,
    pub provision: String,
    pub url: String,
    pub quote: String,
}
#[derive(Clone, Deserialize)]
pub struct Article {
    pub number: u8,
    pub part: usize,
    pub title: String,
    pub core: String,
    pub text: Vec<String>,
    pub families: Vec<String>,
    pub records: Vec<String>,
    pub pins: Vec<String>,
    pub chapters: Vec<u8>,
    pub formal: String,
    pub gap: Option<String>,
    pub lineage: Vec<Comparison>,
    pub novelty: String,
}
#[derive(Deserialize)]
pub struct Articles {
    pub title: String,
    pub note: String,
    pub parts: Vec<String>,
    pub articles: Vec<Article>,
}
#[derive(Clone, Deserialize)]
pub struct Link {
    pub label: String,
    pub path: String,
}
#[derive(Clone, Deserialize)]
pub struct Moved {
    pub chapter: u8,
    pub title: String,
    pub text: String,
    pub links: Vec<Link>,
}
#[derive(Deserialize)]
pub struct ChapterCases {
    pub title: String,
    pub note: String,
    pub moved: Vec<Moved>,
}
/// A repository link for a file, a directory or a heading in a Markdown file.
pub fn repository_url(path: &str) -> String {
    let (file, anchor) = path.split_once('#').unwrap_or((path, ""));
    let kind = if file
        .rsplit('/')
        .next()
        .is_some_and(|name| name.contains('.'))
    {
        "blob"
    } else {
        "tree"
    };
    let anchor = if anchor.is_empty() {
        String::new()
    } else {
        format!("#{anchor}")
    };
    format!("{REPOSITORY}/{kind}/main/{file}{anchor}")
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Verdict {
    pub query: String,
    pub status: String,
    pub detail: Option<String>,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Outcome {
    pub id: String,
    pub counterfactual: Option<String>,
    pub complete: bool,
    pub verdicts: Vec<Verdict>,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub counterfactual: Option<String>,
    pub record: Vec<String>,
    pub queries: Vec<String>,
    pub sources: Vec<Source>,
}
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Source {
    pub path: String,
    pub through_line: Option<usize>,
}
#[derive(Deserialize)]
pub struct Cases {
    pub engine_revision: String,
    pub cases: Vec<Scenario>,
}
pub fn book() -> &'static Book {
    static B: OnceLock<Book> = OnceLock::new();
    B.get_or_init(|| {
        serde_json::from_str(include_str!("../generated/book.json")).expect("assembled book")
    })
}
pub fn companion() -> &'static Companion {
    static C: OnceLock<Companion> = OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_str(include_str!("../companion.json")).expect("companion copy")
    })
}
pub fn articles() -> &'static Articles {
    static A: OnceLock<Articles> = OnceLock::new();
    A.get_or_init(|| {
        serde_json::from_str(include_str!("../articles.json")).expect("plain-language articles")
    })
}
pub fn chapter_cases() -> &'static ChapterCases {
    static C: OnceLock<ChapterCases> = OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_str(include_str!("../chapter-cases.json")).expect("chapter cases")
    })
}
pub fn cases() -> &'static Cases {
    static C: OnceLock<Cases> = OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_str(include_str!("../generated/cases.json")).expect("input-only scenarios")
    })
}
pub fn chapter(number: u8) -> &'static Page {
    book()
        .pages
        .iter()
        .find(|p| p.number == Some(number))
        .expect("chapter")
}
pub fn page(stem: &str) -> Option<&'static Page> {
    book().pages.iter().find(|p| p.stem == stem)
}
pub fn question_path(q: &Question) -> String {
    chapter(q.chapters[0]).path.clone()
}
pub fn route_title(path: &str) -> String {
    if let Some(p) = book().pages.iter().find(|p| p.path == path) {
        return p.label.clone();
    }
    match path.strip_prefix(PREFIX).unwrap_or("404") {
        "" => "Pick a person. Walk their forks.",
        "map/" => "Reader’s map",
        "walkthrough/food-delivery/" => "Does a receipt prove I was fed?",
        "about/" => "About the book and companion",
        "read/" => "Read Book 1",
        "search/" => "Search Book 1",
        "constitution/" => "The constitution in plain language",
        "cases/" => "Run the chapters",
        _ => "Page not found",
    }
    .into()
}
pub fn routes() -> Vec<String> {
    ["", "read/", "search/", "constitution/", "cases/"]
        .iter()
        .map(|p| format!("{PREFIX}{p}"))
        .chain(book().pages.iter().map(|p| p.path.clone()))
        .collect()
}
pub fn search(query: &str) -> Vec<(&'static Page, String)> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return vec![];
    }
    let words: Vec<_> = needle.split_whitespace().collect();
    book()
        .pages
        .iter()
        .filter_map(|page| {
            let lower = page.text.to_lowercase();
            if !words.iter().all(|word| lower.contains(word)) {
                return None;
            }
            // Map byte positions through characters, so Tamil and case expansion never
            // become invalid UTF-8 boundaries in a snippet.
            let chars: Vec<char> = page.text.chars().collect();
            let first = lower.find(words[0]).unwrap_or(0);
            let position = lower[..first].chars().count().min(chars.len());
            let start = position.saturating_sub(70);
            let end = (position + 230).min(chars.len());
            Some((
                page,
                format!(
                    "{}{}{}",
                    if start > 0 { "…" } else { "" },
                    chars[start..end].iter().collect::<String>(),
                    if end < chars.len() { "…" } else { "" }
                ),
            ))
        })
        .collect()
}
