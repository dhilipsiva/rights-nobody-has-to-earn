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

const PART_V: &str = "book-1/15-the-five-joints.md";

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
        .read(PART_V)
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
    let mut files: Vec<_> = std::fs::read_dir(context.path("book-1"))
        .expect("book-1")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.ends_with(".md")
                && name.starts_with(|c: char| c.is_ascii_digit())
                && name != "00-opening-note.md"
                && name != "15-the-five-joints.md"
        })
        .collect();
    files.sort();
    assert_eq!(files.len(), 14, "the derived chapter set changed");
    for name in files {
        let path = format!("book-1/{name}");
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
