// SPDX-License-Identifier: MIT OR Apache-2.0

//! Claim-type discipline, where it is mechanical.
//!
//! The derived chapters carry no figures at all — that is the counted-claims
//! rule. Part V does, as hand-written prose, and every one of them rests on a
//! registry entry. This binds the two together by name, so a case cannot be
//! argued in the book without a source in the registry, and a source cannot be
//! dropped from the registry while the book still leans on it.
//!
//! Ruled 2026-09-15 under delegated approval: Part V's figures **stay
//! hand-written**, and traceability is this binding rather than inline registry
//! ids. Inline ids would put machinery into an exempt element whose whole point
//! is that it reads as argument, and the handful of figures does not justify a
//! rendering step. Build that step only when a consumer needs it.

use super::*;
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeSet;

/// Part V, read from the manifest: the one landed exempt chapter.
fn part_v(context: &Context) -> String {
    contents::Contents::load(context)
        .expect("manifest")
        .part_v()
        .expect("Part V")
}

/// Each historical case Part V argues from, the registry entry it rests on, and
/// a figure or name that must still be in the prose. Both directions matter:
/// the entry must exist, and the prose must still be making the claim.
const TRACED: [(&str, &str, &str); 9] = [
    ("Owen's New Harmony", "harrison-1969-owen", "800 settlers"),
    ("the kibbutzim", "abramitzky-kibbutz", "270 communities"),
    (
        "Tanzanian villagization",
        "medina-2011-cybersyn",
        "13 million",
    ),
    ("Mondragon", "mondragon-2025", "1956"),
    ("Kerala's People's Plan", "kerala-peoples-plan", "1996"),
    ("Cybersyn", "medina-2011-cybersyn", "500 surplus telex"),
    ("the Swiss WIR", "stodder-2009-wir", "1934"),
    (
        "Jharkhand card cancellations",
        "dreze-2017-cancelled-cards",
        "a million cards",
    ),
    (
        "the democracy/wellbeing narrowing",
        "vdem-2026-democracy-happiness-partial",
        "0.5 to about 0.2",
    ),
];

fn registry(context: &Context) -> Value {
    serde_json::from_str(&context.read("registry/claims.json").expect("registry"))
        .expect("registry parses")
}

#[test]
fn every_part_v_figure_rests_on_a_registry_entry() {
    let context = Context::discover().expect("repository");
    // Match against whitespace-normalised prose: these phrases sit across line
    // wraps, and a binding that broke on rewrapping would be noise, not a check.
    let prose = context
        .read(part_v(&context))
        .expect("Part V")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let registry = registry(&context);
    let ids: Vec<&str> = registry["claims"]
        .as_array()
        .expect("claims")
        .iter()
        .filter_map(|claim| claim["id"].as_str())
        .collect();
    for (case, id, figure) in TRACED {
        assert!(
            ids.contains(&id),
            "Part V argues from {case} but the registry has no '{id}'. A figure \
             in an exempt element still needs a source."
        );
        assert!(
            prose.contains(figure),
            "the registry still carries '{id}' for {case}, but Part V no longer \
             says '{figure}'. Either the prose changed and this binding is \
             stale, or the entry is now unused."
        );
    }
    // The Tanzanian and Cybersyn rows share an entry deliberately; the rest are
    // distinct, so a copy-paste that collapsed two cases onto one source would
    // show up here.
    let distinct: BTreeSet<&str> = TRACED.iter().map(|(_, id, _)| *id).collect();
    assert_eq!(
        distinct.len(),
        TRACED.len() - 1,
        "the registry ids behind Part V's cases stopped being distinct"
    );
}

#[test]
fn the_derived_chapters_still_carry_no_figures() {
    let context = Context::discover().expect("repository");
    let number = Regex::new(r"(?:^|[^A-Za-z0-9_.\-])\d[\d,]*(?:\.\d+)?").unwrap();
    let reference = Regex::new(r"(?i)\b(?:chapters?|articles?)\s+\d+[a-z]?|\bv0\.\d+\b").unwrap();
    // This guards the DIGIT half of the counted-claims rule, which is exact and
    // currently at zero. It does not guard the spelled-out half, and pretending
    // otherwise would be worse than the gap: measured 2026-09-15, the derived
    // chapters contain 60 uses of a cardinal beside an inventory noun — "two
    // witnesses", "three routes", "one person's word" — and every one of them
    // states a RULE rather than counting the record's contents. The banned
    // shape is "four people have shelter", and the difference from "it takes two
    // auditors" is semantic. An allowlist of 60 sentences would rot faster than
    // the prose. The spelled-out half is prose review.
    // The derived set is the manifest's, not a count: `reference_integrity_tests`
    // proves the manifest and the directory agree, so a chapter on disk that the
    // manifest does not know fails there rather than here.
    let files = contents::Contents::load(&context).expect("manifest").derived();
    assert!(!files.is_empty(), "the manifest lists no derived chapters");
    for path in files {
        for (index, line) in context.read(&path).expect("chapter").lines().enumerate() {
            if line.starts_with("# ") || line.starts_with("{") {
                continue;
            }
            // A cross-reference is navigation, not a claim about the design.
            // `chapter 5` and `Article 1b` say where to look; they count nothing.
            let line = reference.replace_all(line, " ");
            assert!(
                !number.is_match(&line),
                "{path}:{}: a derived chapter carries a figure. Counts here have \
                 been wrong every time they were checked; state the rule that \
                 produces the count instead: {line}",
                index + 1
            );
        }
    }
}

/// Every rule the method part quotes has to be a rule some file actually holds.
///
/// The part opens on a ground rule — the reader sees rules "exactly as they are
/// written in the files" — and that promise is mechanical, so it is checked
/// rather than trusted. It was not holding: the four standing roots were printed
/// with every variable stripped (`all : born() & ~public() -> person().`), which
/// is not a rule, is not what the source says, and had survived because nothing
/// looked. Rules the part shows in order to say they were REFUSED count too, and
/// they are held by the pin files that run the refusal — so a quoted refusal
/// cannot drift away from the refusal anybody can execute.
///
/// Scope, stated because the gap matters: this guards quoted RULES — a statement
/// opening `all ` and carrying an arrow. The vocabulary listings are set in
/// columns to fit the page and are deliberately outside it, as are the engine's
/// error messages and the pin excerpts.
#[test]
fn every_rule_the_method_part_quotes_is_a_rule_some_file_holds() {
    let context = Context::discover().expect("repository");
    let method = context.read("book-1/method.md").expect("method part");

    let mut quoted: Vec<String> = Vec::new();
    let mut fenced = false;
    let mut pending = String::new();
    for line in method.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            pending.clear();
            continue;
        }
        if !fenced {
            continue;
        }
        if !pending.is_empty() {
            pending.push(' ');
        }
        pending.push_str(line.trim());
        if pending.trim_end().ends_with('.') {
            let statement = normalise(&pending);
            if statement.starts_with("all ") && statement.contains("->") {
                quoted.push(statement);
            }
            pending.clear();
        }
    }
    assert!(
        quoted.len() >= 6,
        "the method part quotes {} rules, which is fewer than it has ever shown — \
         the extractor has stopped seeing them",
        quoted.len()
    );

    let mut sources = Vec::new();
    nibli_files(&context.path("book-1"), &mut sources);
    nibli_files(&context.path("tests"), &mut sources);
    assert!(sources.len() > 40, "the rule sweep found only {} files", sources.len());
    let mut held: BTreeSet<String> = BTreeSet::new();
    for file in &sources {
        for line in std::fs::read_to_string(file).expect("nibli file").lines() {
            let line = line.trim();
            if line.starts_with('#') || !line.contains("->") {
                continue;
            }
            held.insert(normalise(line));
        }
    }

    let missing: Vec<&String> = quoted
        .iter()
        .filter(|rule| !held.contains(*rule))
        .collect();
    assert!(
        missing.is_empty(),
        "the method part promises rules exactly as the files write them, and these \
         are in no file: {missing:#?}"
    );
}

fn normalise(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn nibli_files(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            nibli_files(&entry, into);
        } else if entry.extension().is_some_and(|ext| ext == "nibli") {
            into.push(entry);
        }
    }
}

/// A boundary may say what it does not establish. It may not say it establishes
/// the opposite.
///
/// "This record does not establish that anybody was paid" and "this record
/// proves nobody was paid" are different claims, and only the first is true:
/// the record is silent, and nothing follows from silence. The second is the
/// closed-world fallacy this design refuses everywhere else — an absent fact
/// read as a negative finding — and it had reached four chapters' boundary
/// sections, which are precisely the places written to be scrupulous.
///
/// The shape this catches is `proves` followed by a CLAUSE beginning with a
/// negative. "proves nothing about the service" is a noun phrase and is fine;
/// "proves nothing was procured" is a negative finding and is not. The same for
/// "proves no fund exists" against "proves no civil identity".
#[test]
fn no_boundary_claims_a_negative_it_cannot_establish() {
    let context = Context::discover().expect("repository");
    // `proves nobody|nothing <word>`, where the word is not "about".
    let clause = Regex::new(r"(?i)\bproves\s+(?:that\s+)?(?:nobody|nothing)\s+(\w+)").unwrap();
    // `proves no <noun> <copula-or-event-verb>` — the verb is what makes it a clause.
    let copula = Regex::new(
        r"(?i)\bproves\s+(?:that\s+)?no\s+\w+\s+(?:was|were|is|are|exists|existed|moved|happened|occurred|arrived|reached)\b",
    )
    .unwrap();

    // A sentence that DENIES the proving is not making the claim. "Neither
    // device proves that nobody judges a person elsewhere" is correct and says
    // the opposite of "this record proves nobody was paid". The denial is
    // stripped before matching, on the same principle as the repair detector:
    // a rule beats a per-chapter allowlist, which would rot.
    let denies = Regex::new(
        r"(?i)\b(?:neither\s+\w+\s+proves|does not prove|do not prove|cannot prove|never proves)\b",
    )
    .unwrap();

    let offend = |text: &str| -> Option<String> {
        let owned = denies.replace_all(text, "denies").into_owned();
        let flat: &str = &owned;
        for caught in clause.captures_iter(flat) {
            if !caught[1].eq_ignore_ascii_case("about") {
                return Some(caught[0].to_owned());
            }
        }
        copula.find(flat).map(|m| m.as_str().to_owned())
    };

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(context.path("book-1"))
        .expect("book-1")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort();
    assert!(files.len() > 30, "the sweep found only {} inputs", files.len());

    let mut found = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("input");
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if let Some(hit) = offend(&flat) {
            found.push(format!("{}: {hit}", file.display()));
        }
    }
    assert!(
        found.is_empty(),
        "a boundary claims a negative finding where the record is silent. \
         Absence of evidence is not evidence of absence, and this design says so \
         everywhere else: {found:#?}"
    );

    // Sabotage: the exact sentences this was written for must still be caught.
    for hostile in [
        "a compensation concluded here proves nobody was paid",
        "the guarantee route proves no fund exists",
        "and proves that nothing was procured, delivered, restored or repaired",
        "A completed accommodation record proves no adjustment was provided",
    ] {
        assert!(
            offend(hostile).is_some(),
            "the detector stopped catching {hostile:?}"
        );
    }
    // And the legitimate noun-phrase forms must stay allowed.
    for fine in [
        "it proves nothing about the service that carried it",
        "the custody entry alone proves no home",
        "the temporary encounter handle proves no civil identity",
        "Neither device proves that nobody judges a person elsewhere",
        "this record does not prove that anybody was paid",
    ] {
        assert!(offend(fine).is_none(), "the detector now rejects {fine:?}");
    }
}
