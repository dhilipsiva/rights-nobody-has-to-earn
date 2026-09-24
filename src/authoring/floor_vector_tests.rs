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
use std::collections::{BTreeMap, BTreeSet};

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
        .read("book-1/source/constitution.nibli")
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
fn floor_actualities_have_no_downstream_consumer() {
    let readers = |source: &[String]| {
        source
            .iter()
            .filter_map(|statement| {
                let (body, _) = split(statement)?;
                FLOOR
                    .iter()
                    .find(|item| {
                        Regex::new(&format!(r"(?:^|[^A-Za-z0-9_]){item}\("))
                            .unwrap()
                            .is_match(body)
                    })
                    .map(|item| ((*item).to_owned(), statement.clone()))
            })
            .collect::<Vec<_>>()
    };
    let source = statements();
    assert!(
        readers(&source).is_empty(),
        "a floor actuality has a downstream consumer: {:?}",
        readers(&source)
    );
    // The former absence alarm is a meaningful negative control. A supplied
    // receipt and its absence may neither become a penalty nor certify a breach.
    for item in FLOOR {
        for polarity in ["", "~"] {
            let mut changed = source.clone();
            changed.push(format!(
                "all $x: person($x) & {polarity}{item}($x) -> err($x, Isolation)."
            ));
            assert_eq!(readers(&changed).len(), 1, "{item} {polarity}");
        }
    }
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

/// No constitutional badge ranks a person, even as a Boolean. Keep the retired
/// relation unread and unproduced, and retain the check against counting acts.
#[test]
fn recognition_is_not_minted_or_read_and_contributions_are_not_counted() {
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
    // Removal is a policy decision. A leaf producer would reintroduce the
    // badge even without any consumer, so the absence of readers is insufficient.
    assert_eq!(minted, 0, "a rule reintroduced constitutional recognition");
    // Controls: each refused shape is expressible, so each refusal is a choice.
    for hostile in [
        "all $t: all $s: teaches($t, $s) & reward($t) -> lose(Points, $t).",
        "all $t: all $a: all $b: teaches($t, $a) & teaches($t, $b) -> reward($t).",
        "all $x: all $k: work($x, $k) & work($x, Census) -> reward($x).",
    ] {
        nibli_session::CoreSession::new()
            .compile_text(hostile)
            .expect("the refused shape is expressible");
        let (_, head) = split(hostile).expect("a rule");
        assert!(hostile.contains("reward("));
        assert!(head_relation(head) == "reward" || hostile.contains("& reward("));
    }
}

/// The determination/action boundary, where it is formal rather than editorial:
/// a duty is not an action and an interface is not a capacity, and the way this
/// design says so is that nothing reads them. `verify.sh` used to reject an
/// outside `obliged` consumer; that section retired on 2026-09-12, so the census
/// lives here with its controls.
const NO_READER: [&str; 8] = [
    "owe", "become", "lose", "insure", "provide", "grant", "reward", "prevents",
];

fn atom_arity(atom: &str) -> usize {
    atom.split_once('(')
        .map_or(0, |(_, args)| args.trim_end_matches(')').split(',').count())
}

/// The relations that record an alleged harm. Every one of them names the
/// alleged offender and the person harmed, and none of them names the writer,
/// so an entry here is a finding with no finder.
const ACCUSATION: [&str; 6] = ["attack", "cruel", "injure", "deceive", "capture", "rotten"];

/// Raw inputs whose use must not silently grow into a new adverse authority.
const ACCUSATION_READERS: [(&str, &str); 9] = [
    ("agree", "+capture"),
    ("correct", "+injure"),
    ("err", "+injure"),
    ("err", "~rotten"),
    ("match", "+injure"),
    ("match", "+rotten"),
    ("prisoner", "+injure"),
    ("prisoner", "+injure"),
    ("responsible", "+capture"),
];

/// Census the conclusions that read the raw allegation/examination vocabulary.
/// Raw deceit, attack and cruelty have no reader. The case-bound finding and prospective act still
/// require recorded examinations alongside their independently checked case
/// premises; this syntactic census is not a claim that capture alone suffices.
/// The red-team pins execute the actual effects and retain the other raw-harm
/// scenarios. A new reader still requires an explicit design decision.
#[test]
fn an_unsigned_accusation_reaches_exactly_the_measured_set() {
    let mut found: Vec<(String, String)> = Vec::new();
    for statement in statements() {
        let Some((body, head)) = split(&statement) else {
            continue;
        };
        let mut used = Vec::new();
        for relation in ACCUSATION {
            let read = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_])~?{relation}\(")).unwrap();
            if read.is_match(body) {
                let negated = Regex::new(&format!(r"~\s*{relation}\("))
                    .unwrap()
                    .is_match(body);
                used.push(format!("{}{relation}", if negated { "~" } else { "+" }));
            }
        }
        if !used.is_empty() {
            found.push((head_relation(head).to_owned(), used.join(",")));
        }
    }
    found.sort();
    let declared: Vec<(String, String)> = ACCUSATION_READERS
        .iter()
        .map(|(head, polarity)| ((*head).to_owned(), (*polarity).to_owned()))
        .collect();
    assert_eq!(
        found, declared,
        "the set of conclusions an accusation with no author can reach has \
         changed. Adding one is a design decision about a finding with no \
         finder, and it belongs in a ruling and in the red-team index before it \
         belongs in a rule"
    );

    // Sabotage: a new consequential reader must not pass quietly. This is the
    // shape that would — one line, one existing relation, a conclusion about a
    // person — and the census has to notice it.
    let hostile = "all $x: all $v: cruel($x, $v) -> lose(Points, $x).";
    nibli_session::CoreSession::new()
        .compile_text(hostile)
        .expect("the hostile rule is well-formed, which is why the census matters");
    let mut with_hostile = found.clone();
    with_hostile.push(("lose".to_owned(), "+cruel".to_owned()));
    with_hostile.sort();
    assert_ne!(
        with_hostile, declared,
        "the census would accept a new adverse reader of an unsigned accusation"
    );
}

#[test]
fn a_duty_is_not_an_action_because_nothing_reads_one() {
    let statements = statements();
    for relation in NO_READER {
        let read = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_])~?{relation}\(")).unwrap();
        for statement in &statements {
            let Some((body, _)) = split(statement) else {
                continue;
            };
            assert!(
                !read.is_match(body),
                "'{relation}' is now READ by a rule. It is a leaf on purpose: \
                 reading it is what turns a recorded determination into evidence \
                 that something happened: {statement}"
            );
        }
    }

    // `obliged` is the exception, and it is exactly one: the typed bridge that
    // reads the legacy two-place compatibility conclusion and concludes the
    // three-place duty. The three-place duty itself is read by nothing, which is
    // what keeps a duty from proving its own performance.
    let mut two_place_readers = Vec::new();
    let mut three_place_readers = Vec::new();
    for statement in &statements {
        let Some((body, head)) = split(statement) else {
            continue;
        };
        for found in Regex::new(r"obliged\([^)]*\)").unwrap().find_iter(body) {
            match atom_arity(found.as_str()) {
                2 => two_place_readers.push(head),
                _ => three_place_readers.push(head),
            }
        }
    }
    assert!(
        three_place_readers.is_empty(),
        "a typed duty is read by {three_place_readers:?}, so a duty can now \
         stand as evidence that its reader acted"
    );
    assert_eq!(
        two_place_readers.len(),
        1,
        "the legacy two-place duty has {} readers; exactly one allowlisted \
         bridge is the ruled design: {two_place_readers:?}",
        two_place_readers.len()
    );
    assert!(
        two_place_readers[0].starts_with("obliged("),
        "the one bridge must conclude a typed duty, not something else: {:?}",
        two_place_readers[0]
    );

    // Controls: every refused reader shape is expressible, so each absence is a
    // decision and not a limit of the language.
    for hostile in [
        "all $x: owe(State, Eats, $x) -> eats($x).",
        "all $r: all $x: obliged($r, ReviewPlacement, $x) -> free($x).",
        "all $x: lose(Points, $x) -> err($x, Recognition).",
        "all $x: prevents($x, CompelledConscience) -> reward($x).",
    ] {
        let session = nibli_session::CoreSession::new();
        session
            .compile_text(hostile)
            .expect("the refused reader shape is expressible");
        let (body, _) = split(hostile).expect("a rule");
        assert!(
            NO_READER.iter().any(|relation| {
                Regex::new(&format!(r"(?:^|[^A-Za-z0-9_])~?{relation}\("))
                    .unwrap()
                    .is_match(body)
            }) || Regex::new(r"obliged\([^)]*\)").unwrap().is_match(body),
            "control {hostile} does not exercise the census it is meant to trip"
        );
    }
}

/// Families whose every record completion carries the challenge reader and the
/// independent alternate. The two older families predate that convention and are
/// named below rather than quietly excused.
const CHALLENGE_SKELETON: [&str; 11] = [
    "DEMOCRATIC-INTEGRITY",
    "ECOLOGICAL-ANIMAL",
    "FAMILY-LIFE-ORDINARY",
    "KNOWLEDGE-AND-FREE-FIELD",
    "MOBILITY-PLURALITY",
    "NON-CARCERAL-JUSTICE",
    "OFFICIAL-STATISTICS",
    "PUBLIC-SAFETY",
    "RECORD-POWER",
    "SCARCITY-AND-CONFLICT",
    "SUBSTANTIVE-EQUALITY-ORDINARY",
];
const PREDATES_CHALLENGE_SKELETON: [&str; 2] = ["ECONOMIC-CONSTITUTION", "STATE-FORM"];

/// The generated block a statement sits in, or `ARTICLES` for the hand-written
/// spine. Blocks do not nest, so a single open marker is enough to track.
fn families() -> Vec<(String, String)> {
    let source = Context::discover()
        .expect("repository")
        .read("book-1/source/constitution.nibli")
        .expect("constitution");
    let begin = Regex::new(r"^#\s*<([A-Z-]+)-RULES-BEGIN>$").unwrap();
    let end = Regex::new(r"^#\s*<([A-Z-]+)-RULES-END>$").unwrap();
    let mut open = String::from("ARTICLES");
    let mut rows = Vec::new();
    for line in source.lines().map(str::trim) {
        if let Some(caught) = begin.captures(line) {
            open = caught[1].to_owned();
            continue;
        }
        if end.is_match(line) {
            open = String::from("ARTICLES");
            continue;
        }
        if !line.is_empty() && !line.starts_with('#') {
            rows.push((open.clone(), line.to_owned()));
        }
    }
    rows
}

/// A three-place record completion — the shape every family uses for "this
/// record is complete for this kind about this subject". The two-place
/// `complete` of the legacy transition articles is a different relation.
fn record_completions() -> Vec<(String, String, String)> {
    families()
        .into_iter()
        .filter_map(|(family, statement)| {
            let (body, head) = split(&statement)?;
            (head.starts_with("complete(") && atom_arity(head.trim_end_matches('.')) == 3)
                .then(|| (family, body.to_owned(), statement.clone()))
        })
        .collect()
}

#[test]
fn every_power_record_is_independently_reviewed() {
    let completions = record_completions();
    assert!(
        completions.len() > 500,
        "the census found {} record completions, which is too few to be the \
         whole population — check the block and arity filters",
        completions.len()
    );
    let distinct = Regex::new(r"~\(\$\w+ = \$\w+\)").unwrap();
    let reviewer = Regex::new(r"authorized\(\$\w+, \w*Review\w*Authority, ").unwrap();
    for (family, body, statement) in &completions {
        assert!(
            distinct.is_match(body),
            "a {family} record completes with nobody held apart from anybody: {statement}"
        );
        assert!(
            reviewer.is_match(body),
            "a {family} record completes without an independent review \
             authority: {statement}"
        );
    }
    // The control is the shape, not a name: a completion whose body carries
    // neither guard compiles, so their presence everywhere is a decision.
    let hostile = "all $r: all $k: all $s: observe($r, $k, $s, SomeScope) -> complete($r, $k, $s).";
    nibli_session::CoreSession::new()
        .compile_text(hostile)
        .expect("an unguarded completion is expressible");
    let (body, head) = split(hostile).expect("a rule");
    assert_eq!(atom_arity(head.trim_end_matches('.')), 3);
    assert!(!distinct.is_match(body) && !reviewer.is_match(body));
}

#[test]
fn the_challenge_and_alternate_skeleton_holds_where_it_was_adopted() {
    let challenge = Regex::new(r"authorized\(\$\w+, \w*(?:Challenge|Alternate)\w*, ").unwrap();
    let mut all = BTreeSet::new();
    let mut none = BTreeSet::new();
    let mut partial = BTreeSet::new();
    for family in record_completions()
        .iter()
        .map(|(family, _, _)| family.clone())
        .collect::<BTreeSet<_>>()
    {
        let rows: Vec<_> = record_completions()
            .into_iter()
            .filter(|(owner, _, _)| *owner == family)
            .collect();
        let with = rows
            .iter()
            .filter(|(_, body, _)| challenge.is_match(body))
            .count();
        match with {
            0 => none.insert(family),
            count if count == rows.len() => all.insert(family),
            _ => partial.insert(family),
        };
    }
    assert_eq!(
        all,
        CHALLENGE_SKELETON.iter().map(ToString::to_string).collect(),
        "a family gained or lost the challenge-reader and independent-alternate \
         skeleton; that is a design change, not a regeneration artifact"
    );
    assert_eq!(
        none,
        PREDATES_CHALLENGE_SKELETON
            .iter()
            .map(ToString::to_string)
            .collect(),
        "the families predating the challenge skeleton changed; retrofitting one \
         is welcome, but it is a ruled change and belongs in its contract card"
    );
    assert_eq!(
        partial,
        ["AMENDMENT-ENACTMENT".to_owned()].into_iter().collect(),
        "the partially adopted set moved; amendment enactment is the one family \
         where some completions carry the skeleton and some do not"
    );
}

/// Record kinds that deliberately carry no end condition, because what they
/// record has none: a current selection, a published version, an effective
/// version, an escalation, and three positive findings that something already
/// happened. A power that can be exercised is not on this list, and a new
/// entry here is a design decision that belongs in a contract card.
const ENDLESS_BY_DESIGN: [&str; 7] = [
    "StateFormCurrent",
    "AmendmentPublishedCandidate",
    "AmendmentEffectiveVersion",
    "AmendmentNonresponseEscalation",
    "PSProtectedPublicInterestDisclosureFinding",
    "PSPositiveActualPublicHoldingFinding",
    "PSPositiveProtectiveIncidentFinding",
];

fn completed_kind(head: &str) -> String {
    head.trim_end_matches('.')
        .trim_end_matches(')')
        .split_once('(')
        .map(|(_, args)| args.split(',').nth(1).unwrap_or_default().trim().to_owned())
        .unwrap_or_default()
}

#[test]
fn no_family_can_be_vetoed_by_withholding_evidence() {
    let reader = Regex::new(r"authorized\(\$\w+, \w*Challenge\w*, ").unwrap();
    let alternate = Regex::new(r"authorized\(\$\w+, \w*Alternate\w*, ").unwrap();
    let mut with_reader = 0;
    for (family, body, statement) in record_completions() {
        if !reader.is_match(&body) {
            continue;
        }
        with_reader += 1;
        assert!(
            alternate.is_match(&body),
            "a {family} record admits a challenge reader with no independent \
             alternate behind them, so whoever declines to read it holds a \
             veto: {statement}"
        );
    }
    assert!(
        with_reader > 300,
        "only {with_reader} completions carry a challenge reader, which is too \
         few to be the population this test is about"
    );
}

#[test]
fn every_exercisable_power_record_carries_a_source_bound_end() {
    let ends = Regex::new(r", \w*End\w*Scope\)").unwrap();
    let mut endless = BTreeSet::new();
    for (family, body, statement) in record_completions() {
        let (_, head) = split(&statement).expect("a rule");
        if ends.is_match(&body) {
            continue;
        }
        endless.insert(completed_kind(head));
        assert!(
            ENDLESS_BY_DESIGN.contains(&completed_kind(head).as_str()),
            "a {family} record confers something with no source-bound end, which \
             is unbounded delegation unless it is one of the kinds that has \
             nothing to end: {statement}"
        );
    }
    assert_eq!(
        endless,
        ENDLESS_BY_DESIGN.iter().map(ToString::to_string).collect(),
        "the set of record kinds without an end moved; adding one is a design \
         decision that belongs in a contract card, and removing one means this \
         list is now stale"
    );
}

#[test]
fn no_family_claims_a_rival_final_authority() {
    let observed = Regex::new(r"observe\([^,]+, [^,]+, (\w+), (\w*Final\w*Scope)\)").unwrap();
    let mut routes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, statement) in families() {
        for caught in observed.captures_iter(&statement) {
            routes
                .entry(caught[2].to_owned())
                .or_default()
                .insert(caught[1].to_owned());
        }
    }
    assert!(
        routes.len() > 10,
        "only {} final-route scopes found, which is too few to be the population",
        routes.len()
    );
    for (scope, values) in &routes {
        assert!(
            values.len() == 1 || scope == "FinalityScope" || scope.starts_with("ECFinal"),
            "'{scope}' admits more than one final route — {values:?} — which is \
             how two bodies come to hold the last word over the same thing"
        );
    }
    // `FinalityScope` and the economic merits scopes are outcome vocabularies
    // rather than routes: they say what a final decision may be, not who makes
    // it. Naming them here is the difference between an exception and a hole.
    for scope in ["FinalityScope"] {
        assert!(
            routes[scope].len() > 1,
            "'{scope}' is exempted as an outcome vocabulary, but it now carries \
             a single value and may be a route after all"
        );
    }
}

/// Duty names that would compel or certify a personal state. Conditions may be
/// secured; belief, eating, learning, treatment, a relationship and fulfilment
/// may not be demanded of anybody or certified about them.
const COMPELLED_STATE: [&str; 12] = [
    "Believe",
    "Eat",
    "Learn",
    "Accept",
    "Feel",
    "Trust",
    "Happy",
    "Wellbeing",
    "Fulfil",
    "Satisf",
    "Love",
    "Loyal",
];

/// The one duty naming compliance, and what it actually is: an institution
/// carrying out a court's final protective direction, not a person made to
/// comply with anything.
const INSTITUTIONAL_COMPLIANCE: &str = "ECComplyWithExactFinalProtectiveDirection";

#[test]
fn no_duty_compels_or_certifies_a_personal_state() {
    let duty = Regex::new(r"->\s*obliged\([^,]+,\s*(\w+),").unwrap();
    let mut names = BTreeSet::new();
    for statement in statements() {
        if let Some(caught) = duty.captures(&statement) {
            names.insert(caught[1].to_owned());
        }
    }
    assert!(
        names.len() > 250,
        "only {} duty names found, which is too few to be the population",
        names.len()
    );
    for name in &names {
        if name == INSTITUTIONAL_COMPLIANCE {
            continue;
        }
        for state in COMPELLED_STATE {
            assert!(
                !name.contains(state),
                "the duty '{name}' names a personal state. Conditions may be \
                 secured; believing, eating, learning, accepting treatment, a \
                 relationship and fulfilment may not be demanded or certified."
            );
        }
        assert!(
            !name.contains("Comply"),
            "'{name}' is a second compliance duty. The one that exists is an \
             institution carrying out a court's final direction; a duty to \
             comply is otherwise how a personal state gets demanded sideways."
        );
    }
    // Nothing is named for a psychological state either, so no service record
    // can be read as evidence of one. Only two such names are expressible at
    // all — the rest are refused a step earlier, by the corpus.
    for relation in ["happy", "deserve"] {
        let used = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_]){relation}\(")).unwrap();
        assert!(
            !statements().iter().any(|line| used.is_match(line)),
            "'{relation}' is used as a relation, which is how a formal proof or \
             a service record comes to look like evidence about how somebody feels"
        );
    }
    // Where the guard is, and where it is not. `happy` and `deserve` are corpus
    // names, so the checks above are what keeps them out. `trust`, `wellbeing`,
    // `satisfaction`, `compliance`, `motivation`, `attitude` and `loyalty` are
    // not corpus names: the closure refuses them before any rule here sees
    // them, and this test would pass over a design that had no such guard.
    let session = nibli_session::CoreSession::new();
    for expressible in [
        "all $x: person($x) -> obliged($x, BelieveTheOfficialAccount, Record).",
        "all $x: person($x) -> happy($x).",
    ] {
        session
            .compile_text(expressible)
            .expect("the refused shape is expressible, so refusing it is a choice");
    }
    for refused_upstream in [
        "all $x: person($x) -> trust($x, State).",
        "all $x: person($x) -> compliance($x).",
    ] {
        assert!(
            session.compile_text(refused_upstream).is_err(),
            "{refused_upstream} now compiles, so the corpus no longer refuses \
             it and this test has to carry the name itself"
        );
    }
}

/// Purpose-limited records: a contribution record is written by a scheme and a
/// promise of pay by a payer, and each is read by exactly the rules that
/// conclude what it is for — never under negation, never concluded, never by
/// anything else. The engine does not enforce this: both are base relations,
/// so a rule confining a person for lacking one has no negative cycle for the
/// stratifier to refuse, and `counterfactual/unguarded-contribution-reader`
/// and `counterfactual/unguarded-compensation-reader` show it deriving. This
/// test is the guard the 2026-09-05 ruling named, re-established as a
/// development test after the retired repository audit; those two copies are
/// its watched failing controls.
const PURPOSE_LIMITED: [(&str, &[&str]); 2] = [("pay", &["insure"]), ("promise", &["provide"])];

fn purpose_limited_violations(statements: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    for (relation, readers) in PURPOSE_LIMITED {
        let read = Regex::new(&format!(r"(?:^|[^A-Za-z0-9_])(~?){relation}\(")).unwrap();
        for statement in statements {
            let Some((body, head)) = split(statement) else {
                if statement.starts_with(&format!("{relation}(")) {
                    violations.push(format!(
                        "'{relation}' is asserted by the constitution itself: {statement}"
                    ));
                }
                continue;
            };
            if head_relation(head) == relation {
                violations.push(format!("'{relation}' is concluded by a rule; it is a record, never a conclusion: {statement}"));
            }
            for found in read.captures_iter(body) {
                if &found[1] == "~" {
                    violations.push(format!("'{relation}' is read under negation; its absence may never be a premise: {statement}"));
                } else if !readers.contains(&head_relation(head)) {
                    violations.push(format!("'{relation}' is read by a rule concluding '{}', outside its purpose: {statement}", head_relation(head)));
                }
            }
        }
    }
    violations
}

#[test]
fn a_purpose_limited_record_is_read_only_for_its_purpose() {
    let statements = statements();
    let violations = purpose_limited_violations(&statements);
    assert!(violations.is_empty(), "{}", violations.join("\n"));
    for relation in PURPOSE_LIMITED.map(|(relation, _)| relation) {
        assert!(
            statements
                .iter()
                .any(|s| split(s).is_some_and(|(body, _)| body.contains(&format!("{relation}(")))),
            "'{relation}' is no longer read at all; a guard over a vanished record passes for nothing"
        );
    }
    // The watched failing controls: the exact hostile readers the execution
    // inventory keeps as counterfactual bases, applied to the real statements.
    let context = Context::discover().expect("repository");
    let inventory: serde_json::Value =
        serde_json::from_str(&context.read("tests/pins/suites.json").unwrap()).unwrap();
    for (base, relation) in [
        ("counterfactual/unguarded-contribution-reader", "pay"),
        ("counterfactual/unguarded-compensation-reader", "promise"),
    ] {
        let edits = inventory["bases"][base]["edits"]
            .as_array()
            .expect("hostile base");
        let mut hostile = statements.clone();
        for edit in edits {
            let before = edit["before"].as_str().unwrap();
            let after = edit["after"].as_str().unwrap();
            for line in after.strip_prefix(before).unwrap_or(after).lines() {
                if !line.trim().is_empty() {
                    hostile.push(line.trim().to_owned());
                }
            }
        }
        let found = purpose_limited_violations(&hostile);
        assert!(
            found
                .iter()
                .any(|v| v.contains(&format!("'{relation}' is read under negation"))),
            "{base} no longer trips the guard; the control must fail"
        );
    }
}

/// The personal finding and the incident finding have different readers.
/// Personal findings close prospective signing and trigger reasons and review
/// duties. Only the subject/incident form affects
/// the protection attached to that disclosure: the one-place shield and, since
/// revision item 37, the two case-bound shield rules (one per temporal witness)
/// that the conviction route reads. Neither form supplies a floor,
/// personhood, ballot, appointment, or custody conclusion directly.
const VOIDING_READERS: [(usize, &str, &str); 8] = [
    (1, "agree", "~false"),
    (1, "agree", "~false"),
    (1, "obliged", "+false"),
    (1, "obliged", "+false"),
    (1, "obliged", "+false"),
    (2, "defend", "~false"),
    (2, "defend", "~false"),
    (2, "defend", "~false"),
];

#[test]
fn a_credibility_voiding_reaches_exactly_the_measured_set() {
    let read = Regex::new(r"(?:^|[^A-Za-z0-9_])(~?)\s*false\(([^()]*)\)").unwrap();
    let mut found: Vec<(usize, String, String)> = Vec::new();
    for statement in statements() {
        let Some((body, head)) = split(&statement) else {
            continue;
        };
        for capture in read.captures_iter(body) {
            let polarity = if capture.get(1).is_some_and(|m| m.as_str() == "~") {
                "~false"
            } else {
                "+false"
            };
            let arity = capture[2].split(',').count();
            found.push((arity, head_relation(head).to_owned(), polarity.to_owned()));
        }
    }
    found.sort();
    let declared: Vec<(usize, String, String)> = VOIDING_READERS
        .iter()
        .map(|(arity, head, polarity)| (*arity, (*head).to_owned(), (*polarity).to_owned()))
        .collect();
    assert_eq!(
        found, declared,
        "what a credibility voiding reaches has changed. The chapter states this \
         set as the whole consequence, so a new reader makes the book wrong \
         before it makes the design worse — and a reader on the floor, on \
         standing, on liberty or on the ballot is the carve-out the design \
         refuses"
    );

    // Sabotage: the shape that would quietly widen it. A rule taking a voided
    // person's floor is refused by the stratifier and so cannot be the control;
    // this one loads, which is exactly why the census has to see it.
    let hostile = "all $x: false($x) -> err($x, Recognition).";
    nibli_session::CoreSession::new()
        .compile_text(hostile)
        .expect("the hostile rule is well-formed, which is why the census matters");
    let mut with_hostile = found.clone();
    with_hostile.push((1, "err".to_owned(), "+false".to_owned()));
    with_hostile.sort();
    assert_ne!(
        with_hostile, declared,
        "the census would accept a new reader of a credibility voiding"
    );
}

/// Every personal or incident finding has an identifiable case and reads its
/// restoration. Neither a parent/child judgment nor an amendment label nor an
/// aggregate carried credential restriction is a separate personal-penalty route.
#[test]
fn a_recorded_expungement_reaches_every_voiding_that_turns_on_a_finding() {
    let mut reads_clean = Vec::new();
    let mut ignores_clean = Vec::new();
    for statement in statements() {
        let Some((body, head)) = split(&statement) else {
            continue;
        };
        if head_relation(head) != "false" {
            continue;
        }
        if body.contains("~clean(") {
            reads_clean.push(body.to_owned());
        } else {
            ignores_clean.push(body.to_owned());
        }
    }
    assert_eq!(
        reads_clean.len(),
        2,
        "the routes an expungement stops changed: {reads_clean:?}"
    );
    assert!(
        ignores_clean.is_empty(),
        "a finding route bypasses restoration: {ignores_clean:?}"
    );
    for body in reads_clean {
        assert!(
            body.contains("responsible($record, CredibilityFinding)")
                && body.contains("list($record, $subject, $incident, CredibilityFinding)")
                && body.contains("~clean($record, CredibilityFinding)"),
            "a personal or incident consequence lacks its completed case or ending: {body}"
        );
    }
}
