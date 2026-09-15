// SPDX-License-Identifier: MIT OR Apache-2.0

//! The floor is a vector of separate protected conditions, never a total.
//!
//! Four properties, each measured against the enacted lines of the current
//! constitution and each paired with a control that must fail. They replace
//! pattern guards the 2026-09-12 verification decision retired with the old
//! `verify.sh` sections, and they are development tests: verification itself
//! stays pins and contradiction scans.

use super::*;
use regex::Regex;

/// The eight floor actualities, spelled as the constitution spells them.
const FLOOR: [&str; 8] = [
    "secure",
    "eats",
    "dwell",
    "healthy",
    "learn",
    "expresses",
    "believe",
    "meets",
];

/// Enacted lines only: comments carry prose about arithmetic and scores that
/// the rules deliberately do not contain.
fn statements() -> Vec<String> {
    Context::discover()
        .expect("repository")
        .read("new-book-plans/constitution.nibli")
        .expect("constitution")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn split(statement: &str) -> Option<(&str, &str)> {
    let at = statement.rfind("->")?;
    Some((&statement[..at], statement[at + 2..].trim()))
}

fn head_relation(head: &str) -> &str {
    head.split('(').next().unwrap_or_default().trim()
}

/// A bare number used as a term. Identifiers carrying digits — `FSPOW_061`,
/// `EconomicCurrentSelection_061` — are names, not quantities, and the lookaround
/// is what keeps this test about arithmetic rather than about spelling.
fn numeric_literal() -> Regex {
    Regex::new(r"(?:^|[^A-Za-z0-9_])\d+(?:\.\d+)?(?:[^A-Za-z0-9_]|$)").unwrap()
}

#[test]
fn nothing_in_the_enacted_lines_is_a_quantity() {
    let literal = numeric_literal();
    let offenders: Vec<_> = statements()
        .into_iter()
        .filter(|statement| literal.is_match(statement))
        .collect();
    assert!(
        offenders.is_empty(),
        "a numeric literal entered the enacted lines, so something can now be \
         weighted, totalled or compared: {:?}",
        &offenders[..offenders.len().min(3)]
    );
    // The control: the same scan over a line that does carry a quantity.
    assert!(literal.is_match("all $x: person($x) & sum($x, 2, 2) -> reward($x)."));
    assert!(!literal.is_match("observe($s, $r, FSPOW_061, PowerScope)."));
}

#[test]
fn no_relation_aggregates_scores_or_ranks() {
    let banned = [
        "sum",
        "product",
        "quotient",
        "difference",
        "count",
        "total",
        "score",
        "rank",
        "aggregate",
        "average",
        "index",
        "weight",
    ];
    let statements = statements();
    for name in banned {
        let used = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_]){name}\(")).unwrap();
        let offenders: Vec<_> = statements
            .iter()
            .filter(|statement| used.is_match(statement))
            .collect();
        assert!(
            offenders.is_empty(),
            "'{name}' is used as a relation, which is how heterogeneous floors, \
             liberties, commons and democratic choices collapse into one number: \
             {:?}",
            &offenders[..offenders.len().min(3)]
        );
    }
    let control = Regex::new(r"(?:^|[^A-Za-z0-9_])score\(").unwrap();
    assert!(control.is_match("all $x: person($x) -> score($x, Wellbeing)."));
}

#[test]
fn a_floor_actuality_is_read_only_under_negation_and_only_into_err() {
    let mut reads = Vec::new();
    for statement in statements() {
        let Some((body, head)) = split(&statement) else {
            continue;
        };
        for item in FLOOR {
            let positive = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_~]){item}\(")).unwrap();
            assert!(
                !positive.is_match(body),
                "'{item}' is read POSITIVELY by a rule, so having one floor \
                 condition can now do work elsewhere: {statement}"
            );
            if Regex::new(&format!(r"~{item}\(")).unwrap().is_match(body) {
                assert_eq!(
                    head_relation(head),
                    "err",
                    "'{item}' is read under negation into something other than \
                     `err` — noticed is the whole permission: {statement}"
                );
                reads.push(item);
            }
        }
    }
    // Article 6's isolation marker is the one read, and it is what keeps this
    // test from passing over a constitution that reads nothing at all.
    assert_eq!(
        reads,
        ["meets"],
        "the set of floor reads moved; re-read Article 6 and INVARIANT 1 before \
         changing this expectation"
    );
}

#[test]
fn no_floor_condition_substitutes_for_another() {
    for statement in statements() {
        let Some((body, head)) = split(&statement) else {
            continue;
        };
        let concluded = head_relation(head);
        if !FLOOR.contains(&concluded) {
            continue;
        }
        for item in FLOOR {
            if item == concluded {
                continue;
            }
            let mentioned = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_]){item}\(")).unwrap();
            assert!(
                !mentioned.is_match(body),
                "'{concluded}' is derived from '{item}', so abundance in one \
                 protected condition now compensates for another: {statement}"
            );
        }
    }
    // The control: exactly the shape this test exists to refuse, and it compiles,
    // so the refusal is a decision rather than a limit of the language.
    let hostile = "all $x: person($x) & eats($x) & dwell($x) -> healthy($x).";
    nibli_session::CoreSession::new()
        .compile_text(hostile)
        .expect("the substitution shape is expressible");
    let (body, head) = split(hostile).expect("a rule");
    assert_eq!(head_relation(head), "healthy");
    assert!(
        Regex::new(r"(?:^|[^A-Za-z0-9_])eats\(")
            .unwrap()
            .is_match(body)
    );
}

/// Recognition is the other half of "never a total": a floor may not be traded
/// and recognition may not be counted. These restate the three checks the old
/// `verify.sh` sections 4, 4a and 4b performed, for the same reason and with
/// the same negative controls.
#[test]
fn recognition_is_minted_never_read_and_never_counted() {
    let statements = statements();
    let mut minted = 0;
    for statement in &statements {
        let Some((body, head)) = split(statement) else {
            continue;
        };
        assert!(
            !Regex::new(r"(?:^|[^A-Za-z0-9_~])reward\(")
                .unwrap()
                .is_match(body),
            "something now READS recognition, so it can condition an outcome: {statement}"
        );
        if head_relation(head) == "reward" {
            minted += 1;
            let places = head
                .trim_end_matches('.')
                .trim_end_matches(')')
                .split_once('(')
                .map_or(0, |(_, args)| args.split(',').count());
            assert_eq!(
                places, 1,
                "recognition gained a second place, which is a slot for what it \
                 was for — and a provenance slot exists only to be read: {statement}"
            );
        }
        // Counted degree needs no arithmetic, only the same relation twice with
        // its objects held apart. That is the shape, so that is the check.
        for door in ["teaches", "work"] {
            let uses = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_~]){door}\("))
                .unwrap()
                .find_iter(body)
                .count();
            assert!(
                uses < 2,
                "'{door}' is joined with itself, which counts contributions \
                 without ever writing a number: {statement}"
            );
        }
    }
    // "Nothing reads recognition" passes just as cleanly against a constitution
    // that renamed recognition away, so assert it is still there and still
    // minted. Three doors mint it today.
    assert_eq!(
        minted, 3,
        "the number of minting rules moved; recognition's doors are a ruled \
         design decision, not an incidental count"
    );
    // Controls: each refused shape is expressible, so each refusal is a choice.
    for hostile in [
        "all $t: all $s: teaches($t, $s) & reward($t) -> lose(Points, $t).",
        "all $t: all $a: all $b: teaches($t, $a) & teaches($t, $b) -> reward($t).",
        "all $x: all $k: work($x, $k) & work($x, Census) -> reward($x).",
    ] {
        nibli_session::CoreSession::new()
            .compile_text(hostile)
            .expect("the refused shape is expressible");
    }
}
