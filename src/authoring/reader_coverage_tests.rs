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

/// Postures held by two passages or fewer. Thin is not absent, and the two
/// thinnest are the ones the portfolio standard cares most about: somebody
/// making something, and somebody caring for somebody.
const THIN_POSTURES: [&str; 3] = ["associates", "cares", "creates"];

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
/// the ruling; this is the census. Three on 2026-09-17, after the economy
/// chapter rendered ECONOMIC-CONSTITUTION, INCOME-SECURITY and
/// QUALIFICATIONS-COMPENSATION; each Phase B chapter removes its family, and
/// the empty expectation then stays so the next family that lands without a
/// passage fails here instead of passing quietly.
const UNRENDERED_FAMILIES: [&str; 3] = [
    "LIBERTY-ECOLOGY",
    "PUBLIC-SCALE-VOCABULARY",
    "SUBSTANTIVE-EQUALITY",
];

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
