// SPDX-License-Identifier: MIT OR Apache-2.0

//! Claim-type discipline, where it is mechanical.
//!
//! The derived text carries no figures at all — that is the counted-claims
//! rule. The argued text does, as hand-written prose: Part V, and since ruling
//! D2 the one labelled argument section that may close each derived chapter.
//! Every figure there rests on a registry entry. This binds the two together by
//! name, so a case cannot be argued in the book without a source in the
//! registry, and a source cannot be dropped from the registry while the book
//! still leans on it.
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

/// The book's argued text, read from the manifest: Part V's chapters and every
/// derived chapter's argument section, whitespace-normalised so a binding does
/// not break when a line is rewrapped.
fn argued_text(context: &Context) -> String {
    let contents = contents::Contents::load(context).expect("manifest");
    let mut text = String::new();
    for path in contents.part_v().expect("Part V") {
        text.push_str(&context.read(&path).expect("Part V"));
        text.push('\n');
    }
    for path in contents.derived() {
        let chapter = context.read(&path).expect("chapter");
        let (_, argued) = contents::split_argument(&chapter)
            .unwrap_or_else(|problem| panic!("{path}: {problem}"));
        text.push_str(argued);
        text.push('\n');
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Each historical case Part V argues from, the registry entry it rests on, and
/// a figure or name that must still be in the prose. Both directions matter:
/// the entry must exist, and the prose must still be making the claim.
/// Part V's figures, each bound to the registry entry that carries it and to a
/// phrase that must still be in the prose. Both directions matter: a figure
/// cannot lose its source, and a source cannot be dropped while the book still
/// leans on it. One source may carry several figures — Tanzania's relocation
/// count and Mondragon's headcount are not one claim each — so the rows are
/// keyed by case, not by entry.
const TRACED: [(&str, &str, &str); 39] = [
    ("Owen's New Harmony", "harrison-1969-owen", "New Harmony"),
    ("the kibbutzim", "abramitzky-kibbutz", "270 communities"),
    (
        "the kibbutz salary reform",
        "abramitzky-kibbutz",
        "three in four kibbutzim",
    ),
    (
        "China losing the right to leave",
        "lin-1990-collectivization",
        "until 1958",
    ),
    (
        "the Norwegian incarceration study",
        "bhuller-2020-incarceration-recidivism-employment",
        "judge they happened",
    ),
    (
        "Nordic penal exceptionalism",
        "pratt-2008-scandinavian-exceptionalism",
        "low imprisonment",
    ),
    ("the new civil death", "chin-2012-new-civil-death", "civil death"),
    ("the Bihar undertrials", "hussainara-khatoon-1979", "three to ten years"),
    ("Robodebt", "robodebt-royal-commission-2023", "crude and cruel"),
    (
        "the Dutch childcare-benefits affair",
        "nl-ongekend-onrecht-2020",
        "childcare-benefits affair",
    ),
    ("the SyRI judgment", "syri-hague-district-court-2020", "SyRI"),
    (
        "the NJAC judgment",
        "scaora-v-union-of-india-njac-2015",
        "Judicial Appointments Commission",
    ),
    (
        "Switzerland's chambers",
        "swiss-constitution-1999-federal-council-council-of-states",
        "Council of States",
    ),
    ("Uruguay's colegiado", "uruguay-colegiado-1952", "1966"),
    ("New Zealand's resident vote", "nz-electoral-act-1993-s74", "New Zealand"),
    (
        "Chile's resident vote",
        "chile-constitution-art14-foreign-residents-2025",
        "ten uninterrupted years",
    ),
    (
        "the German budget ruling",
        "bverfg-2023-second-supplementary-budget-2021",
        "supplementary budget",
    ),
    (
        "Colombia's displacement judgment",
        "colombia-t025-2004-displacement",
        "unconstitutional state of affairs for internally displaced",
    ),
    ("ADM Jabalpur", "adm-jabalpur-1976", "habeas corpus"),
    ("Kesavananda Bharati", "kesavananda-bharati-1973", "basic structure"),
    ("Mondragon", "mondragon-2025", "1956"),
    (
        "Mondragon's reported pay scale",
        "mondragon-2021-pay-range",
        "six to one",
    ),
    (
        "the corporate pay comparison",
        "epi-2026-ceo-pay-ratio",
        "325-to-1",
    ),
    ("Kerala's People's Plan", "kerala-peoples-plan", "1996"),
    (
        "Kerala's trained facilitators",
        "kerala-peoples-plan",
        "100,000 people",
    ),
    ("Cybersyn", "medina-2011-cybersyn", "existing telex network"),
    ("Auroville's governance", "auroville-governance", "Auroville"),
    ("the Swiss WIR", "stodder-2009-wir", "1934"),
    (
        "Jharkhand card cancellations",
        "dreze-2017-cancelled-cards",
        "mass cancellation of ration cards",
    ),
    (
        "the Jharkhand experimental study",
        "muralidharan-2025-lost-access",
        "one and a half and two million",
    ),
    (
        "Santoshi Kumari's death",
        "santoshi-kumari-2017",
        "Santoshi Kumari",
    ),
    (
        "the democracy/wellbeing narrowing",
        "vdem-2026-democracy-happiness-partial",
        "0.5 to about 0.2",
    ),
    (
        "the regime-group comparison",
        "vdem-2026-regime-ladder-steps",
        "closed and electoral autocracies",
    ),
    (
        "the exploratory dispersion calculation",
        "vdem-2026-floor-claim-instrument-fragile",
        "negative after income",
    ),
    (
        "the causal-control qualification",
        "cinelli-forney-pearl-controls",
        "Cinelli, Forney and Pearl",
    ),
    (
        "the comparison of significance labels",
        "gelman-stern-2006-significance",
        "Gelman and Stern",
    ),
    (
        "the significance guidance",
        "asa-2016-p-values",
        "American Statistical Association",
    ),
    (
        "constructive replacement with a Chancellor",
        "german-basic-law-constructive-replacement",
        "Germany's Basic Law",
    ),
    (
        "a bounded emergency derogation framework",
        "iccpr-article-4-derogation",
        "International Covenant on Civil and Political Rights",
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
    let prose = argued_text(&context);
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
            "the registry still carries '{id}' for {case}, but no argued text \
             says '{figure}' — neither Part V nor an argument section. Either \
             the prose changed and this binding is stale, or the entry is now \
             unused."
        );
    }
    // One source may legitimately carry several of Part V's figures, so the
    // rows are keyed by case and figure rather than by entry. What must never
    // collapse is a row onto another row: that is a copy-paste, not a shared
    // source.
    let distinct: BTreeSet<(&str, &str)> =
        TRACED.iter().map(|(case, _, figure)| (*case, *figure)).collect();
    assert_eq!(
        distinct.len(),
        TRACED.len(),
        "two of Part V's traced figures collapsed onto the same case and phrase"
    );
}

/// A source the reader cannot open is not a source. Every registry entry Part V
/// argues from must carry a locator — a URL or a DOI — in its `source` field.
/// This is what makes "every named study has a usable primary-source URL" a
/// check rather than a promise; it says nothing about whether the locator
/// resolves today, which is a network fact no test here can establish.
#[test]
fn every_traced_source_carries_a_locator_a_reader_can_follow() {
    let context = Context::discover().expect("repository");
    let registry = registry(&context);
    let claims = registry["claims"].as_array().expect("claims");
    let ids: BTreeSet<&str> = TRACED.iter().map(|(_, id, _)| *id).collect();
    for id in ids {
        let entry = claims
            .iter()
            .find(|claim| claim["id"].as_str() == Some(id))
            .unwrap_or_else(|| panic!("Part V argues from '{id}' and the registry has no such entry"));
        let source = entry["source"].as_str().unwrap_or_default();
        assert!(
            source.contains("http") || source.contains("doi:"),
            "registry entry '{id}' backs a figure in Part V but gives the reader \
             no locator: its source field carries neither a URL nor a DOI, so \
             the only path to the evidence is knowing where to look."
        );
    }
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
    // An argument section (ruling D2) is argued text and may cite a year or a
    // figure with its registry source; the check reads the derived text before
    // it, which is every line up to the section's labelled heading.
    let files = contents::Contents::load(&context).expect("manifest").derived();
    assert!(!files.is_empty(), "the manifest lists no derived chapters");
    for path in files {
        let chapter = context.read(&path).expect("chapter");
        let (derived, _) = contents::split_argument(&chapter)
            .unwrap_or_else(|problem| panic!("{path}: {problem}"));
        for (index, line) in derived.lines().enumerate() {
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

/// Ruling D2 lets each derived chapter close with one labelled argument
/// section. Every derived chapter either has none or has exactly one, headed
/// `## Argument: …` and last, so the derived-text checks and the argued-text
/// bindings always read the text they are meant to.
#[test]
fn every_argument_section_is_labelled_single_and_last() {
    let context = Context::discover().expect("repository");
    for path in contents::Contents::load(&context).expect("manifest").derived() {
        let chapter = context.read(&path).expect("chapter");
        if let Err(problem) = contents::split_argument(&chapter) {
            panic!("{path}: {problem}");
        }
    }
    // Sabotage: a figure in derived text before the label still counts; the
    // same figure after the label does not.
    let (derived, argued) =
        contents::split_argument("# A\n\nIt reached 1996 homes.\n\n## Argument: why\n\nIn 1996, I argue.\n")
            .expect("well formed");
    assert!(derived.contains("1996") && argued.contains("1996"));
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
        // Pin directives and verdict comments can precede a quoted rule.
        // They are not part of its statement and must not hide it from this
        // check. Once a rule starts, retain its wrapped continuation lines.
        if pending.is_empty() && !line.trim_start().starts_with("all ") {
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
