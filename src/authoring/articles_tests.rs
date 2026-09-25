// SPDX-License-Identifier: MIT OR Apache-2.0

//! The plain-language articles, traced to the rules (ruling D4, item 61).
//!
//! The companion publishes the constitution as numbered plain-language
//! articles (`ui/articles.json`). Most rules sit in generated blocks with no
//! article banner, so each article names the rule *families* that implement it
//! — a generated block's marker, or a hand-written `Article` banner — with the
//! contract or decision that governs them and the tests that check them. This
//! holds the map to account in both directions: every family that carries a
//! rule belongs to some article, and every article names families, records and
//! tests that exist. A declared non-formal part is allowed only with its reason.
//! The chapters cite the articles by number, and every cited number resolves.
//!
//! It checks the map, not the wording: whether an article's plain language
//! says what its families do is prose review.

use super::*;
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const ARTICLES: &str = "ui/articles.json";
const CITATION: &str = r"rights-nobody-has-to-earn/constitution/#article-(\d+)";

/// Every family the constitution declares, with the number of rules in each.
/// A generated block is `# <NAME-BEGIN>` … `# <NAME-END>`; a hand-written
/// section runs from a `# ─── Article …` banner to the next banner. A rule
/// belongs to its innermost block, or else to the section it sits in.
fn families(source: &str) -> BTreeMap<String, usize> {
    let begin = Regex::new(r"^# <([A-Z0-9-]+)-BEGIN>").unwrap();
    let end = Regex::new(r"^# <([A-Z0-9-]+)-END>").unwrap();
    let banner = Regex::new(r"^# ─── (.*)$").unwrap();
    let article = Regex::new(r"^(Article \w+)").unwrap();
    let mut stack: Vec<String> = Vec::new();
    let mut section: Option<String> = None;
    let mut found: BTreeMap<String, usize> = BTreeMap::new();
    for line in source.lines() {
        if let Some(c) = begin.captures(line) {
            stack.push(c[1].to_owned());
            found.entry(c[1].to_owned()).or_insert(0);
        }
        if let Some(c) = end.captures(line) {
            if stack.last().map(String::as_str) == Some(&c[1]) {
                stack.pop();
            }
        }
        if let Some(c) = banner.captures(line) {
            let title = c[1].trim().to_owned();
            let name = article
                .captures(&title)
                .map(|a| a[1].to_owned())
                .unwrap_or(title);
            found.entry(name.clone()).or_insert(0);
            section = Some(name);
        }
        if line.starts_with('#') || !line.contains("->") {
            continue;
        }
        let owner = stack.last().cloned().or_else(|| section.clone());
        if let Some(owner) = owner {
            *found.entry(owner).or_insert(0) += 1;
        }
    }
    found
}

/// What is wrong with the map, given the families the source declares, the
/// articles, the files that exist and the chapter numbers the manifest holds.
fn problems(
    declared: &BTreeMap<String, usize>,
    articles: &Value,
    exists: &dyn Fn(&str) -> bool,
    chapters: &BTreeSet<usize>,
) -> Vec<String> {
    let mut out = Vec::new();
    let list = articles["articles"].as_array().cloned().unwrap_or_default();
    let parts = articles["parts"].as_array().map(Vec::len).unwrap_or(0);
    let mut claimed: BTreeSet<String> = BTreeSet::new();
    for (index, article) in list.iter().enumerate() {
        let number = article["number"].as_u64().unwrap_or(0);
        let label = format!("Article {number}");
        if number != index as u64 + 1 {
            out.push(format!("{label}: articles must be numbered 1, 2, 3 … in order"));
        }
        let part = article["part"].as_u64().unwrap_or(0) as usize;
        if part == 0 || part > parts {
            out.push(format!("{label}: part {part} is not one of the {parts} parts"));
        }
        let strings = |key: &str| -> Vec<String> {
            article[key]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
                .unwrap_or_default()
        };
        let named = strings("families");
        let pins = strings("pins");
        let formal = article["formal"].as_str().unwrap_or("");
        match formal {
            "formal" | "partly formal" => {
                if named.is_empty() {
                    out.push(format!("{label}: names no rule family"));
                }
                if pins.is_empty() {
                    out.push(format!("{label}: names no test"));
                }
            }
            "non-formal" => {}
            other => out.push(format!("{label}: unknown formality '{other}'")),
        }
        if formal != "formal" && article["gap"].as_str().unwrap_or("").trim().is_empty() {
            out.push(format!("{label}: a non-formal part needs its reason in 'gap'"));
        }
        for family in &named {
            if !declared.contains_key(family) {
                out.push(format!("{label}: '{family}' is not a family the constitution declares"));
            }
            claimed.insert(family.clone());
        }
        for path in strings("records").iter().chain(pins.iter()) {
            if !exists(path) {
                out.push(format!("{label}: '{path}' does not exist"));
            }
        }
        for chapter in article["chapters"].as_array().cloned().unwrap_or_default() {
            let n = chapter.as_u64().unwrap_or(0) as usize;
            if !chapters.contains(&n) {
                out.push(format!("{label}: chapter {n} is not in the manifest"));
            }
        }
        if article["text"].as_array().map(Vec::is_empty).unwrap_or(true) {
            out.push(format!("{label}: has no text"));
        }
    }
    for (family, rules) in declared {
        if *rules > 0 && !claimed.contains(family) {
            out.push(format!("'{family}' carries {rules} rules and belongs to no article"));
        }
    }
    out
}

fn load(context: &Context) -> (BTreeMap<String, usize>, Value, BTreeSet<usize>) {
    let declared = families(&context.read("book-1/source/constitution.nibli").unwrap());
    let articles: Value = serde_json::from_str(&context.read(ARTICLES).unwrap()).unwrap();
    let contents = contents::Contents::load(context).expect("manifest");
    let chapters = contents
        .chapters()
        .filter(|c| c.file.is_some())
        .map(|c| c.number)
        .collect();
    (declared, articles, chapters)
}

#[test]
fn every_rule_family_belongs_to_an_article_and_every_article_is_traced() {
    let context = Context::discover().expect("repository");
    let (declared, articles, chapters) = load(&context);
    assert!(
        declared.values().filter(|n| **n > 0).count() > 20,
        "the family census found too few rule-bearing families; the parser broke"
    );
    let exists = |path: &str| context.path(path).exists();
    let found = problems(&declared, &articles, &exists, &chapters);
    assert!(found.is_empty(), "the article map is out of step:\n{}", found.join("\n"));
}

#[test]
fn the_article_map_fails_when_a_family_or_a_test_goes_missing() {
    let context = Context::discover().expect("repository");
    let (declared, articles, chapters) = load(&context);
    let exists = |path: &str| context.path(path).exists();
    // Sabotage: an article loses its families and tests, another names a family
    // that does not exist, and a rule-bearing family loses its only claim.
    let mut broken = articles.clone();
    broken["articles"][0]["families"] = Value::Array(vec![]);
    broken["articles"][0]["pins"] = Value::Array(vec![]);
    broken["articles"][1]["families"] =
        Value::Array(vec![Value::String("NO-SUCH-RULES".into())]);
    let found = problems(&declared, &broken, &exists, &chapters);
    assert!(found.iter().any(|p| p.contains("names no rule family")), "{found:?}");
    assert!(found.iter().any(|p| p.contains("names no test")), "{found:?}");
    assert!(found.iter().any(|p| p.contains("NO-SUCH-RULES")), "{found:?}");
    assert!(
        found.iter().any(|p| p.contains("belongs to no article")),
        "dropping a family's claim must leave it unclaimed: {found:?}"
    );
    // A non-formal part without its reason fails too.
    let mut silent = articles.clone();
    let last = silent["articles"].as_array().unwrap().len() - 1;
    silent["articles"][last]["gap"] = Value::Null;
    let found = problems(&declared, &silent, &exists, &chapters);
    assert!(found.iter().any(|p| p.contains("needs its reason")), "{found:?}");
}

#[test]
fn every_article_a_chapter_cites_resolves() {
    let context = Context::discover().expect("repository");
    let (_, articles, _) = load(&context);
    let numbers: BTreeSet<u64> = articles["articles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["number"].as_u64())
        .collect();
    let contents = contents::Contents::load(&context).expect("manifest");
    let citation = Regex::new(CITATION).unwrap();
    let mut cited = 0;
    let mut paths: Vec<String> = contents.chapters().filter_map(|c| c.path()).collect();
    paths.push("book-1/00-opening-note.md".into());
    paths.push("book-1/reference.md".into());
    for path in paths {
        let text = context.read(&path).unwrap();
        for c in citation.captures_iter(&text) {
            cited += 1;
            let n: u64 = c[1].parse().unwrap();
            assert!(numbers.contains(&n), "{path} cites Article {n}, which does not exist");
        }
    }
    assert!(cited > 0, "no chapter cites an article; the citation pattern broke");
}

#[test]
fn every_article_records_its_lineage() {
    let context = Context::discover().expect("repository");
    let (_, articles, _) = load(&context);
    for article in articles["articles"].as_array().unwrap() {
        let n = article["number"].as_u64().unwrap();
        assert!(
            !article["novelty"].as_str().unwrap_or("").trim().is_empty(),
            "Article {n} states no lineage or novelty"
        );
        for entry in article["lineage"].as_array().cloned().unwrap_or_default() {
            let url = entry["url"].as_str().unwrap_or("");
            assert!(
                url.starts_with("https://www.constituteproject.org/constitution/"),
                "Article {n} compares a provision without a Constitute Project locator: {url}"
            );
            assert!(
                !entry["quote"].as_str().unwrap_or("").trim().is_empty(),
                "Article {n} compares a provision without the words it quotes"
            );
        }
    }
}
