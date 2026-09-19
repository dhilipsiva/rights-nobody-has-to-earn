// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reference integrity, made mechanical.
//!
//! Three things were hand-checked until 2026-09-16 and are checked here now:
//! that every reviewed `path::needle` reference resolves to a file in which
//! the needle occurs exactly once (the hazard CLAUDE.md warns about by name);
//! that the chapter manifest and the `book-1/` directory agree, with the
//! two-digit prefix equal to the manifest position; and that the navigation
//! the reader is handed — the opening note's annotated contents and every
//! relative link in the book and its appendix — actually resolves.
//!
//! The reference test is **asserted by membership against a dated baseline**:
//! measured on the unchanged tree at `9d0a4381` (paths as they stand after the
//! 2026-09-17 move), thirty-two references named
//! files that no longer exist (retired scripts, pre-suites counterfactual
//! names, verification receipts) and eleven needles no longer occurred in
//! their file. Those are recorded, not repaired, so the invariant a move must
//! keep is "the resolving set is unchanged modulo rewriting" — a new failure
//! and a silent repair both fail here.

use super::contents::{Contents, Role, Status};
use super::*;
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Where the reviewed JSON lives. Rewritten by the relocation tool when it moves.
const REVIEWED: &str = "book-1/source";

/// Files that reviewed references name and that the tree no longer carries,
/// measured 2026-09-16. Each is a historical pointer inside a frozen audit.
const KNOWN_MISSING: [&str; 32] = [
    "book-1/source/12-temporal-assurance.py",
    "book-1/source/13-full-society-ledger.py",
    "book-1/source/14-reader-evidence.py",
    "book-1/source/16-constitutional-closure.py",
    "book-1/source/7-assertion-surface.py",
    "book-1/source/9-record-integrity-red-team.py",
    "book-1/source/counterfactual/no-choose-boss.nibli",
    "book-1/source/counterfactual/no-dead-conjuncts.nibli",
    "book-1/source/counterfactual/no-person-line.nibli",
    "book-1/source/counterfactual/no-public-court.nibli",
    "book-1/source/counterfactual/undelivered-marker.nibli",
    "book-1/source/counterfactual/unguarded-pen.nibli",
    "book-1/source/verification-receipts/sha256-035ed0488cf074414f69f7255551323a43e8c0ec3926c53e9f49622b238f818d.json",
    "book-1/source/verification-receipts/sha256-05c2974b93429ef7e68d851195a8c70529f109b54fad6346a67751f590750e30.json",
    "book-1/source/verification-receipts/sha256-0f69b681f3041102069ea08527472991a472555155fb27ec74763166cd189227.json",
    "book-1/source/verification-receipts/sha256-1261ece1fe278e199ce9fb3567f7fc306875a656ba2bec5f3a521cc4cf08347a.json",
    "book-1/source/verification-receipts/sha256-2063c52ee41b6cc3cad30034e72c6029374b018ec7316cb474f64721df6af315.json",
    "book-1/source/verification-receipts/sha256-2c4f5879c901ca7f5ef3f4852893e91f1dc87ea26f0694efeebc10f70bcbd8ce.json",
    "book-1/source/verification-receipts/sha256-3849096fff1216c1cbd0914cd99ce01488501f126509b060438ec1139199a7b1.json",
    "book-1/source/verification-receipts/sha256-62edd4996c9928ce47a9f248f3ef19654996b8d655c99302e83f7eecffc4a297.json",
    "book-1/source/verification-receipts/sha256-99cba105a59e01b5335406904f72ac336d87f139b0e9acdf574075b95b23a0d0.json",
    "book-1/source/verification-receipts/sha256-a0ddc29b4bfeb0db302ef54276026946ff9518d6341772ef8dcd954339bb4f7c.json",
    "book-1/source/verification-receipts/sha256-a15e5a435c2b825622f4dd60ceaf21a9df4d11e3b3b27a4f7b04d0659b7473b8.json",
    "book-1/source/verification-receipts/sha256-aa0e91787c113de3bebcccd95a4083403d8caac86ae92b16a91631dcdf1d60db.json",
    "book-1/source/verification-receipts/sha256-b780c38fd3948f2eb3852f4fe4d2dbebc5674570a485f906dfd21c1c2871cb52.json",
    "book-1/source/verification-receipts/sha256-c9208b6db2b6eb865e6849267d511df1ed52a170d5eb840a5f4d34e71ee8d552.json",
    "book-1/source/verification-receipts/sha256-de0a932cff8e9e6147dcd529122119ad5d7b3272fab85f1a4854e5fa016deb25.json",
    "book-1/source/verification-receipts/sha256-e1184c495c3fb4628901e672e65b2d523b595c0b2e1230092f2cd06587c26e02.json",
    "book-1/source/verification-receipts/sha256-ea7a73c7741f1c318709ad06b0308cad122c103b64869fdc3bbc24696a6a4c75.json",
    "book-1/source/verification-receipts/sha256-f1f781ce58025d6226bf5dd8e7242924f1447f9bd75c9039030f649d9d97a334.json",
    "book-1/source/verification-receipts/sha256-fa2b552970c7e8bf7c2691395c325dbf4dcf4c0d834c5eb73d077797686ff74b.json",
    "book-1/source/verification-receipts/sha256-fd1b4841518380649b50080c1de34172d225a31c9f1e106bd2733495cd4dbbd1.json",
];

/// Files whose reviewed needles no longer occur in them, with how many such
/// needles each carries, measured 2026-09-16.
// The five stale method references were resolved with its 2026-09-19 rewrite.
const KNOWN_UNMATCHED: [(&str, usize); 3] = [
    ("book-1/source/constitution.nibli", 2),
    ("book-1/source/counterfactual/README.md", 1),
    ("book-1/source/full-society-scope-review-protocol.md", 2),
];

fn reference() -> Regex {
    Regex::new(r"^(?:book-1|tests|registry)/[^:\s]+(?:::.+)?$").unwrap()
}

fn collect(value: &Value, pattern: &Regex, into: &mut BTreeSet<(String, String)>) {
    match value {
        Value::String(text) => {
            if pattern.is_match(text) {
                let (path, needle) = match text.split_once("::") {
                    Some((path, needle)) => (path.to_owned(), needle.to_owned()),
                    None => (text.clone(), String::new()),
                };
                into.insert((path, needle));
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect(item, pattern, into)),
        Value::Object(map) => map.values().for_each(|item| collect(item, pattern, into)),
        _ => {}
    }
}

fn reviewed_references(context: &Context) -> BTreeSet<(String, String)> {
    let pattern = reference();
    let mut refs = BTreeSet::new();
    let mut files: Vec<_> = std::fs::read_dir(context.path(REVIEWED))
        .expect("reviewed directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no reviewed JSON under {REVIEWED}");
    for file in files {
        let value: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        collect(&value, &pattern, &mut refs);
    }
    refs
}

#[test]
fn every_reviewed_reference_resolves_exactly_once() {
    let context = Context::discover().expect("repository");
    let refs = reviewed_references(&context);
    assert!(refs.len() > 1000, "the reference census collapsed to {}", refs.len());
    let mut cache: BTreeMap<String, String> = BTreeMap::new();
    let mut missing = BTreeSet::new();
    let mut unmatched: BTreeMap<String, usize> = BTreeMap::new();
    for (path, needle) in &refs {
        if !context.path(path).is_file() {
            missing.insert(path.clone());
            continue;
        }
        if needle.is_empty() {
            continue;
        }
        let text = cache
            .entry(path.clone())
            .or_insert_with(|| context.read(path).expect("referenced file"));
        if text.matches(needle.as_str()).count() != 1 {
            *unmatched.entry(path.clone()).or_default() += 1;
        }
    }
    let expected_missing: BTreeSet<String> = KNOWN_MISSING
        .iter()
        .map(|p| (*p).to_owned())
        .collect();
    assert_eq!(
        missing, expected_missing,
        "the set of reviewed references naming a file the tree does not carry changed. \
         A new arrival is a reference a move or an edit broke; a departure belongs out of KNOWN_MISSING."
    );
    let expected_unmatched: BTreeMap<String, usize> = KNOWN_UNMATCHED
        .iter()
        .map(|(p, n)| ((*p).to_owned(), *n))
        .collect();
    assert_eq!(
        unmatched, expected_unmatched,
        "the needles that do not occur exactly once in their file changed. A new arrival is a \
         needle an edit broke — re-check the file before committing; a departure belongs out of KNOWN_UNMATCHED."
    );
}

#[test]
fn contents_manifest_matches_the_directory() {
    let context = Context::discover().expect("repository");
    let contents = Contents::load(&context).expect("manifest");
    let on_disk: BTreeSet<String> = std::fs::read_dir(context.path("book-1"))
        .expect("book-1")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".md") && name.starts_with(|c: char| c.is_ascii_digit()))
        .collect();
    let mut listed: BTreeSet<String> = contents
        .chapters()
        .filter_map(|c| c.file.clone())
        .collect();
    for front in &contents.front {
        if front.starts_with(|c: char| c.is_ascii_digit()) {
            listed.insert(front.clone());
        }
    }
    assert_eq!(
        on_disk, listed,
        "the numbered files in book-1/ and the manifest's landed entries differ"
    );
    for chapter in contents.chapters() {
        match chapter.status {
            Status::Planned => {
                let stray: Vec<_> = on_disk
                    .iter()
                    .filter(|name| name.starts_with(&chapter.prefix()))
                    .collect();
                assert!(
                    stray.is_empty(),
                    "chapter {} is planned but {stray:?} carries its number",
                    chapter.number
                );
            }
            Status::Landed => {
                let file = chapter.file.as_ref().unwrap();
                let pins = format!("book-1/{}.pins.nibli", file.trim_end_matches(".md"));
                match chapter.role {
                    Role::Derived => assert!(
                        context.path(&pins).is_file(),
                        "derived chapter {file} has no paired {pins}"
                    ),
                    Role::Exempt => assert!(
                        !context.path(&pins).is_file(),
                        "exempt chapter {file} must not carry a pins file"
                    ),
                }
                let title = context
                    .read(chapter.path().unwrap())
                    .expect("chapter")
                    .lines()
                    .find_map(|line| line.strip_prefix("# ").map(str::trim).map(str::to_owned))
                    .unwrap_or_default();
                assert_eq!(
                    title, chapter.title,
                    "{file}'s H1 and the manifest title differ"
                );
            }
        }
    }
    // Each derived chapter is one live, scanned case in the execution inventory,
    // identified by its path sans extension.
    let inventory: Value =
        serde_json::from_str(&context.read("tests/pins/suites.json").unwrap()).unwrap();
    let cases = inventory["cases"].as_array().unwrap();
    for path in contents.derived() {
        let id = path.trim_end_matches(".md");
        let matching: Vec<&Value> = cases.iter().filter(|c| c["id"] == id).collect();
        assert_eq!(matching.len(), 1, "{id} must have exactly one case in suites.json");
        let case = matching[0];
        assert_eq!(case["base"], "live", "{id} must run against the live constitution");
        assert_eq!(case["scan"], true, "{id} must be scanned for contradictions");
        assert_eq!(
            case["pins"],
            Value::Array(vec![Value::String(format!("{id}.pins.nibli"))]),
            "{id}'s case must run exactly its paired pins file"
        );
    }
}

/// GitHub's heading slug, close enough for this book's headings: lowercase,
/// punctuation dropped, spaces to hyphens.
fn slug(heading: &str) -> String {
    let cleaned: String = heading
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect();
    cleaned.to_lowercase().split_whitespace().collect::<Vec<_>>().join("-")
}

fn headings_in(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter(|line| line.starts_with('#'))
        .map(|line| slug(line.trim_start_matches('#')))
        .collect()
}

fn markdown_files(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            let name = entry.file_name().unwrap().to_string_lossy().into_owned();
            // The formal source's own markdown is not the reader's navigation.
            if name != "source" {
                markdown_files(&entry, into);
            }
        } else if entry.extension().is_some_and(|ext| ext == "md") {
            into.push(entry);
        }
    }
}

#[test]
fn every_relative_link_in_the_book_and_its_appendix_resolves() {
    let context = Context::discover().expect("repository");
    let link = Regex::new(r"\]\(([^)\s]+)\)").unwrap();
    let mut files = Vec::new();
    markdown_files(&context.path("book-1"), &mut files);
    assert!(files.len() > 15, "the link sweep found only {} files", files.len());
    let mut broken = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        for capture in link.captures_iter(&text) {
            let target = &capture[1];
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let (path_part, fragment) = match target.split_once('#') {
                Some((p, f)) => (p, Some(f)),
                None => (target, None),
            };
            let resolved = if path_part.is_empty() {
                file.clone()
            } else {
                file.parent().unwrap().join(path_part)
            };
            if !resolved.exists() {
                broken.push(format!("{}: {target} (no such file)", file.display()));
                continue;
            }
            if let Some(fragment) = fragment {
                if resolved.extension().is_some_and(|ext| ext == "md") {
                    let headings = headings_in(&std::fs::read_to_string(&resolved).unwrap());
                    if !headings.contains(fragment) {
                        broken.push(format!("{}: {target} (no such heading)", file.display()));
                    }
                }
            }
        }
    }
    assert!(broken.is_empty(), "dangling links:\n{}", broken.join("\n"));
}

#[test]
fn opening_note_navigation_matches_the_manifest() {
    let context = Context::discover().expect("repository");
    let contents = Contents::load(&context).expect("manifest");
    let note = context.read("book-1/00-opening-note.md").unwrap();
    let start = note.find("## Annotated contents").expect("annotated contents");
    let end = note[start..]
        .find("\n## ")
        .map(|offset| start + offset)
        .unwrap_or(note.len());
    let section = &note[start..end];
    let part_heading = Regex::new(r"(?m)^### Part [IVX]+ — (.+)$").unwrap();
    let listed_parts: Vec<String> = part_heading
        .captures_iter(section)
        .map(|c| c[1].trim().to_owned())
        .collect();
    let manifest_parts: Vec<String> = contents.parts.iter().map(|p| p.title.clone()).collect();
    assert_eq!(
        listed_parts, manifest_parts,
        "the opening note's part headings and the manifest's parts differ"
    );
    let link = Regex::new(r"\]\(([0-9]{2}-[^)#]+\.md)\)").unwrap();
    let mut per_part: Vec<Vec<String>> = Vec::new();
    for piece in part_heading.split(section).skip(1) {
        per_part.push(
            link.captures_iter(piece)
                .map(|c| c[1].to_owned())
                .collect::<Vec<_>>(),
        );
    }
    for (part, listed) in contents.parts.iter().zip(per_part.iter()) {
        let landed: Vec<String> = part.chapters.iter().filter_map(|c| c.file.clone()).collect();
        assert_eq!(
            listed, &landed,
            "under \"Part … — {}\" the opening note lists chapters in a different order or set than the manifest",
            part.title
        );
    }
}

/// The ruled order, once the tree follows it: every break comes after every
/// engine, and the exempt chapter comes last. Added with the reorder of
/// 2026-09-17, when the manifest first carried the rule.
#[test]
fn derived_chapters_run_engines_before_breaks() {
    let context = Context::discover().expect("repository");
    let contents = Contents::load(&context).expect("manifest");
    assert!(
        contents.rule.is_some(),
        "the manifest carries no ordering rule; the reorder is ruled and the rule is recorded there"
    );
    let mut seen_break = false;
    let mut seen_exempt = false;
    for chapter in contents.chapters() {
        match (chapter.role, chapter.group) {
            (Role::Derived, Some(super::contents::Group::Engine)) => {
                assert!(!seen_break, "chapter {} is an engine after a break", chapter.number);
                assert!(!seen_exempt, "chapter {} follows the exempt chapter", chapter.number);
            }
            (Role::Derived, Some(super::contents::Group::Break)) => {
                assert!(!seen_exempt, "chapter {} follows the exempt chapter", chapter.number);
                seen_break = true;
            }
            (Role::Exempt, _) => seen_exempt = true,
            (Role::Derived, None) => unreachable!("validated at load"),
        }
    }
    assert!(seen_break && seen_exempt, "the manifest has no breaks or no exempt chapter");
}

#[test]
fn spine_contents_block_is_current() {
    let context = Context::discover().expect("repository");
    let contents = Contents::load(&context).expect("manifest");
    let spine = context.read("book-1/source/3-spine.md").unwrap();
    let begin = spine
        .find(super::contents::BEGIN)
        .expect("3-spine.md has the contents markers");
    let after = begin + super::contents::BEGIN.len();
    let end = after + spine[after..].find(super::contents::END).expect("end marker");
    let current = spine[after..end].trim();
    assert_eq!(
        current,
        contents.render(),
        "3-spine.md's reading-order block is stale; run ./generate.sh spine"
    );
}
