// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The one domain the ledger shows without ordinary operation, and why it is a
/// constitutional gap rather than a prose one: the family, dependency,
/// reproduction and care baseline landed 106 person-held barriers and no
/// ordinary-operation interface, so there is nothing derivable for a passage to
/// show working. Writing one anyway would be fictionalising coverage, which the
/// portfolio standard refuses by name.
const NO_ORDINARY_SHOWN: [&str; 1] = ["Life course, family, care and reproduction"];

/// Domains that meet the standard through a stated boundary rather than through
/// a passage in which something goes wrong. That is the weaker of the two
/// accepted forms, and where the portfolio rebalance has most to do.
const BOUNDED_BUT_NEVER_FAILING: [&str; 7] = [
    "Borders, migration, asylum and expulsion",
    "Collective and plurality rights",
    "Defence and armed force",
    "Ecology, future generations and commons",
    "Emergency and resilience",
    "Knowledge, communication and culture",
    "Non-human animals",
];

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
