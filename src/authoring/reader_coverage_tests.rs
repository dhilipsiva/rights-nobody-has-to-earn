// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// Domains the ledger shows without ordinary operation. The set is empty as of
/// 2026-09-16: life course, family, care and reproduction was the last member,
/// and it left when the ordinary half of that baseline was implemented rather
/// than when a passage was written about it. The assertion stays as an empty
/// expectation, because that ordering is the rule — a domain shown only under
/// strain is a constitutional gap, and writing the passage first would
/// fictionalise the coverage.
const NO_ORDINARY_SHOWN: [&str; 0] = [];

/// Domains that meet the standard through a stated boundary rather than through
/// a passage in which something goes wrong. That is the weaker of the two
/// accepted forms. The set is empty as of 2026-09-16 and the assertion stays:
/// an empty expectation is what makes a domain landing without a failure
/// passage fail here rather than pass quietly.
const BOUNDED_BUT_NEVER_FAILING: [&str; 0] = [];

#[test]
fn every_passage_is_classified_and_every_basis_exists() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    validate(&context, &records).expect("the ledger classifies exactly the book's passages");
    assert_eq!(
        records.len(),
        headings(&context).expect("headings").len(),
        "the ledger must have one record per passage and no more"
    );
}

#[test]
fn the_thin_domains_are_the_ones_the_ledger_says_they_are() {
    let context = Context::discover().expect("repository");
    let rows = coverage(&context, &records(&context).expect("ledger source")).expect("coverage");
    // A domain is covered when it shows ordinary operation and either strain or
    // a stated boundary. Boundary is the weaker form and is counted separately.
    let uncovered: BTreeSet<_> = rows
        .iter()
        .filter(|(_, (works, trouble, bound))| *works > 0 && *trouble == 0 && *bound == 0)
        .map(|(domain, _)| domain.clone())
        .collect();
    assert!(
        uncovered.is_empty(),
        "these domains show ordinary operation and neither strain nor a stated \
         boundary: {uncovered:?}"
    );
    let no_ordinary: BTreeSet<_> = rows
        .iter()
        .filter(|(_, (works, _, _))| *works == 0)
        .map(|(domain, _)| domain.clone())
        .collect();
    assert_eq!(
        no_ordinary,
        NO_ORDINARY_SHOWN.iter().map(ToString::to_string).collect(),
        "the set of domains shown only under strain changed"
    );
    let bounded_only: BTreeSet<_> = rows
        .iter()
        .filter(|(_, (works, trouble, bound))| *works > 0 && *trouble == 0 && *bound > 0)
        .map(|(domain, _)| domain.clone())
        .collect();
    assert_eq!(
        bounded_only,
        BOUNDED_BUT_NEVER_FAILING
            .iter()
            .map(ToString::to_string)
            .collect(),
        "the set of domains carried by a boundary alone changed. Giving one a \
         passage where something goes wrong is progress and belongs out of this \
         list; a new arrival is a domain that landed without one."
    );
}

#[test]
fn no_domain_is_explained_only_through_custody() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let custodial = [
        "custody",
        "confin",
        "prison",
        "placement",
        "arrest",
        "detention",
        "release",
        "severity",
    ];
    let mut domains: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
    for record in &records {
        domains
            .entry(record.domain.clone())
            .or_default()
            .push(record);
    }
    for (domain, rows) in &domains {
        if domain.starts_with("Justice") || domain.starts_with("Public safety") {
            continue;
        }
        assert!(
            rows.iter().any(|record| {
                let text = format!("{} {}", record.setting, record.family).to_lowercase();
                !custodial.iter().any(|needle| text.contains(needle))
            }),
            "'{domain}' is explained only through custody, which the coverage \
             standard refuses for a non-justice domain"
        );
    }
}

#[test]
fn the_book_is_not_one_failure_first_formula() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let counted = |pattern: &str| {
        records
            .iter()
            .filter(|record| record.pattern == pattern)
            .count()
    };
    for pattern in PATTERNS {
        assert!(
            counted(pattern) > 0,
            "no passage follows the '{pattern}' pattern, so the book has \
             collapsed onto fewer shapes than the ruling names"
        );
    }
    let coercive = counted("coercive");
    let agency = counted("constructive") + counted("private-civic") + counted("democratic");
    assert!(
        coercive * 3 < records.len(),
        "{coercive} of {} passages are coercive. The refusal is one \
         failure-first formula for everything.",
        records.len()
    );
    assert!(
        agency > coercive * 2,
        "passages of provision, private life and democratic agency ({agency}) no \
         longer outweigh coercive ones ({coercive}) by the margin that keeps the \
         prisoner a stress test rather than the default inhabitant"
    );
}

/// Postures no passage occupies. Each is a thing the constitution lets somebody
/// do that the book never shows anybody doing, so the list shrinking is
/// progress and the list growing is a passage that lost its subject.
const UNOCCUPIED_POSTURES: [&str; 0] = [];

/// Postures held by two passages or fewer. Thin is not absent, and a thin
/// posture is usually a missing interface rather than a missing paragraph — the
/// way to fill one is to land the rules that let somebody do the thing, not to
/// write a passage about it. Measured 2026-09-18, after the rebuild: `cares`
/// and `associates` left when the life-course and mobility families were
/// rendered, and `creates` is the one that remains.
/// Postures carried by two passages or fewer. The set is empty, and it reached
/// empty the way the ledger predicts: `creates` was the last one left, and what
/// moved it was a passage about why recognition exists at all — an interface
/// question rather than a missing paragraph, which is the pattern every thin
/// posture in this book has turned out to follow.
///
/// It stays asserted as an empty expectation so that a posture falling back to
/// two fails the check rather than passing quietly.
const THIN_POSTURES: [&str; 0] = [];

#[test]
fn the_thin_postures_are_the_ones_recorded() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let held = |posture: &str| {
        records
            .iter()
            .filter(|record| record.postures.iter().any(|entry| entry == posture))
            .count()
    };
    let thin: BTreeSet<String> = POSTURES
        .iter()
        .filter(|posture| {
            let count = held(posture);
            count > 0 && count <= 2
        })
        .map(|posture| (*posture).to_owned())
        .collect();
    assert_eq!(
        thin,
        THIN_POSTURES.iter().map(ToString::to_string).collect(),
        "the set of postures carried by two passages or fewer changed. A          posture leaving is progress; a posture arriving means the book stopped          showing somebody doing something it used to show."
    );
}

/// Every rule family the book renders must state a boundary somewhere it is
/// rendered. A family projected only in the affirmative reads as a promise, and
/// the whole register of this book is that a legal conclusion is not an event.
/// Boundary is read out of the prose rather than declared, so a family cannot
/// claim a disclosure its passages do not make.
#[test]
fn every_family_states_a_boundary_where_it_is_rendered() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let mut families: BTreeMap<String, bool> = BTreeMap::new();
    for record in &records {
        let stated = states_boundary(&context, record).expect("passage");
        let entry = families.entry(record.family.clone()).or_default();
        *entry = *entry || stated;
    }
    let silent: BTreeSet<&String> = families
        .iter()
        .filter(|(_, stated)| !**stated)
        .map(|(family, _)| family)
        .collect();
    assert!(
        silent.is_empty(),
        "these families are rendered without stating a boundary anywhere:          {silent:?}. A family projected only in the affirmative reads as a          promise."
    );
}

#[test]
fn the_postures_nobody_occupies_are_the_ones_recorded() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let held = |posture: &str| {
        records
            .iter()
            .filter(|record| record.postures.iter().any(|entry| entry == posture))
            .count()
    };
    let empty: BTreeSet<String> = POSTURES
        .iter()
        .filter(|posture| held(posture) == 0)
        .map(|posture| (*posture).to_owned())
        .collect();
    assert_eq!(
        empty,
        UNOCCUPIED_POSTURES
            .iter()
            .map(ToString::to_string)
            .collect(),
        "the set of postures no passage occupies changed. Filling one is what \
         the portfolio rebalance is for; a new one appearing means a passage \
         stopped showing somebody doing something."
    );
}

/// Rule families with a marked block in the constitution and no passage
/// tagged to them in the ledger. "Book 1 must project each landed family" is
/// the ruling; this is the census. Two on 2026-09-17, after the economy
/// chapters rendered ECONOMIC-CONSTITUTION, INCOME-SECURITY,
/// QUALIFICATIONS-COMPENSATION and PUBLIC-SCALE-VOCABULARY; each Phase B
/// chapter removes its family, and
/// the empty expectation then stays so the next family that lands without a
/// passage fails here instead of passing quietly.
const UNRENDERED_FAMILIES: [&str; 0] = [];

/// Ledger family tags that name no block: the kernel articles, the placement
/// rules, and the exempt Part V.
const LEDGER_ONLY_FAMILIES: [&str; 3] = ["ARTICLES", "PLACEMENT", "exempt"];

/// Every family the constitution carries as a marked block, read from the
/// source so a family that lands without a passage fails rather than passes.
fn constitutional_families(context: &Context) -> BTreeSet<String> {
    let marker = Regex::new(r"^# <([A-Z][A-Z0-9-]*)-BEGIN>$").expect("marker");
    context
        .read("book-1/source/constitution.nibli")
        .expect("constitution")
        .lines()
        .filter_map(|line| marker.captures(line.trim()).map(|c| c[1].to_owned()))
        .map(|name| {
            let name = name
                .trim_end_matches("-RULES")
                .trim_end_matches("-ADMISSIONS")
                .trim_end_matches("-DERIVED")
                .trim_end_matches("-FACTS")
                .to_owned();
            // The economic family's derived-only roster is marked ECONOMIC.
            if name == "ECONOMIC" { "ECONOMIC-CONSTITUTION".to_owned() } else { name }
        })
        // The temporal articles are kernel, tagged ARTICLES in the ledger.
        .filter(|name| !name.starts_with("T1") && !name.starts_with("T2") && !name.starts_with("T3"))
        .collect()
}

#[test]
fn every_constitutional_family_is_projected_by_a_passage() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    let blocks = constitutional_families(&context);
    assert!(blocks.len() > 15, "the block census collapsed to {}", blocks.len());
    let tagged: BTreeSet<String> = records.iter().map(|r| r.family.clone()).collect();
    let unrendered: BTreeSet<String> = blocks.difference(&tagged).cloned().collect();
    assert_eq!(
        unrendered,
        UNRENDERED_FAMILIES.iter().map(ToString::to_string).collect(),
        "the set of rule families Book 1 does not project changed. A new arrival is a \
         family that landed without a passage; a departure belongs out of UNRENDERED_FAMILIES."
    );
    let orphan: Vec<&String> = tagged
        .iter()
        .filter(|f| !LEDGER_ONLY_FAMILIES.contains(&f.as_str()) && !blocks.contains(*f))
        .collect();
    assert!(orphan.is_empty(), "passages tagged to a family no block carries: {orphan:?}");
}

/// Derived chapters that do NOT carry the recurring `## The child with nobody`
/// section, each with the reason it does not. The list is asserted by
/// membership rather than counted, so a chapter landing without a slot and
/// without a reason fails here.
///
/// Ruling D5 keeps the section where the one-entry record produces a different
/// or instructive result. Chapter 1 is the child: the whole chapter is the case,
/// and a section inside it would be a section about its own subject. Chapter 9
/// already runs the no-age-premise case with Cira's work entry, and chapter 24
/// follows Cira's separate claim beside the teacher's finding. The rest follow
/// for the child exactly as for anyone, or need a record the child does not
/// have.
const CHILD_SLOT_EXEMPT: [(&str, &str); 20] = [
    ("01-the-child-with-nobody.md", "the chapter is the case, and its encounters reach standing as the birth does"),
    ("03-what-you-are-owed.md", "the floor follows for the child as for every person; chapter 1 runs it"),
    ("06-who-owes-and-what-follows.md", "the bystander's entry about the child is already the chapter's own case"),
    ("07-an-ordinary-week.md", "the week follows an adult with a full record; the one-line record holds none of its entries"),
    ("08-what-nobody-has-to-ask-permission-for.md", "the liberties follow from personhood alone, as for anyone"),
    ("09-work-pay-and-contribution.md", "no contribution yields no supplement, as for anyone; Cira carries the child exhibit"),
    ("10-what-money-cannot-buy.md", "the economic prohibitions hold for any person; no property adds nothing"),
    ("11-the-same-route-for-everyone.md", "the anti-substitution barriers hold for every person alike"),
    ("12-a-place-in-which-life-remains-possible.md", "the environmental claim needs no owner or spokesperson for anyone"),
    ("13-creatures-without-a-ballot.md", "an animal's protection reads no human record at all"),
    ("15-arriving-and-belonging.md", "the newcomer is the chapter's person with little on record"),
    ("16-answerability-and-authority.md", "answerability concerns public bodies; the child holds none"),
    ("17-how-public-power-is-built.md", "the institutions' mandates run to everyone alike"),
    ("19-what-may-be-kept-about-you.md", "a birth-only record holds no file to keep, use or correct"),
    ("20-a-crisis-does-not-suspend-the-republic.md", "an emergency leaves the child's floor as it leaves anyone's"),
    ("22-changing-the-rules.md", "the amendment rules protect the floor for everyone alike"),
    ("23-the-shield.md", "the shield needs an exposure this child has not made"),
    ("24-findings-about-people.md", "no finding reaches a birth entry, which holds no authority to sign; Cira's claim is tested here"),
    ("26-where-people-are-put.md", "placement needs a custody case the child's record does not have"),
    ("27-the-one-thing-taken.md", "confinement needs a custody case; chapter 25 runs the child beside the prisoner"),
];

#[test]
fn every_derived_chapter_runs_the_child_or_says_why_not() {
    let context = Context::discover().expect("repository");
    let exempt: BTreeSet<&str> = CHILD_SLOT_EXEMPT.iter().map(|(file, _)| *file).collect();
    assert_eq!(
        exempt.len(),
        CHILD_SLOT_EXEMPT.len(),
        "a chapter is listed twice in the exemptions"
    );
    let contents = Contents::load(&context).expect("manifest");
    let mut missing = Vec::new();
    for chapter in contents.derived() {
        let file = chapter
            .rsplit('/')
            .next()
            .expect("chapter file name")
            .to_string();
        let prose = context.read(&chapter).expect("chapter");
        let runs = prose.contains("\n## The child with nobody\n");
        match (runs, exempt.contains(file.as_str())) {
            (false, false) => missing.push(format!("{file} has no child slot and no reason")),
            (true, true) => missing.push(format!("{file} is exempt and carries a slot anyway")),
            _ => {}
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// Every chapter that runs the child must load the one-line record it runs
/// against. A slot written against a record the case does not carry would pass
/// the prose check above and prove nothing.
#[test]
fn every_child_slot_loads_the_one_line_record() {
    const FIXTURE: &str = "tests/pins/records/child_with_nobody/fixture.nibli";
    let context = Context::discover().expect("repository");
    let suites = context.read("tests/pins/suites.json").expect("suites");
    let suites: serde_json::Value = serde_json::from_str(&suites).expect("suites json");
    let cases = suites["cases"].as_array().expect("cases");
    let contents = Contents::load(&context).expect("manifest");
    let mut missing = Vec::new();
    for chapter in contents.derived() {
        let prose = context.read(&chapter).expect("chapter");
        if !prose.contains("\n## The child with nobody\n") {
            continue;
        }
        let id = chapter.trim_end_matches(".md");
        let case = cases
            .iter()
            .find(|case| case["id"].as_str() == Some(id))
            .unwrap_or_else(|| panic!("{id} has no case"));
        let loads = case["fixtures"]
            .as_array()
            .expect("fixtures")
            .iter()
            .any(|fixture| fixture.as_str() == Some(FIXTURE));
        if !loads {
            missing.push(format!("{id} runs the child without loading the record"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// Argued text and derived text are classified apart: a derived passage never
/// takes the `argument` pattern or the exempt-element basis, and Part V's
/// passages, like any labelled argument section (ruling D2), always take both.
#[test]
fn argued_and_derived_passages_are_classified_apart() {
    let context = Context::discover().expect("repository");
    let records = records(&context).expect("ledger source");
    validate(&context, &records).expect("the ledger as it stands");
    // Sabotage both ways: a derived passage labelled argument, and a Part V
    // passage labelled as derived text.
    let mut derived_as_argument = records.clone();
    let row = derived_as_argument
        .iter_mut()
        .find(|record| record.pattern != "argument")
        .expect("a derived passage");
    row.pattern = "argument".to_owned();
    row.basis = "exempt-element".to_owned();
    assert!(validate(&context, &derived_as_argument).is_err());
    let mut argument_as_derived = records.clone();
    let row = argument_as_derived
        .iter_mut()
        .find(|record| record.pattern == "argument")
        .expect("an argued passage");
    row.pattern = "constructive".to_owned();
    assert!(validate(&context, &argument_as_derived).is_err());
}

/// Words a reader reads, HTML comments aside.
fn words(text: &str) -> usize {
    let comment = regex::Regex::new(r"(?s)<!--.*?-->").unwrap();
    comment.replace_all(text, " ").split_whitespace().count()
}

/// The length invariant, measured by section (ruling D2): the derived text of
/// the derived chapters outweighs everything argued or exempt combined — the
/// front and back matter, the Part openers, Part V and every argument section.
/// The appendix and the formal source are outside the ordered inputs and
/// outside the measurement.
#[test]
fn the_book_stays_majority_derived_by_section() {
    let context = Context::discover().expect("repository");
    let contents = Contents::load(&context).expect("manifest");
    let mut derived = 0;
    let mut other = 0;
    for path in contents.derived() {
        let chapter = context.read(&path).expect("chapter");
        let (flat, argued) = crate::authoring::contents::split_argument(&chapter).expect("well formed");
        derived += words(flat);
        other += words(argued);
    }
    let mut exempt: Vec<String> = contents
        .front
        .iter()
        .chain(contents.back.iter())
        .map(|name| format!("book-1/{name}"))
        .collect();
    exempt.extend(contents.part_v().expect("Part V"));
    exempt.extend(contents.openers());
    for path in exempt {
        other += words(&context.read(&path).expect("exempt input"));
    }
    assert!(derived > 0 && other > 0, "the measurement read nothing");
    assert!(
        derived > other,
        "the book is no longer majority-derived: {derived} derived words against \
         {other} argued or exempt"
    );
}
