// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// Domains the ledger currently shows with ordinary operation and no credible
/// failure, abuse or boundary. Closing one is the point of the portfolio
/// rebalance; until then the list is asserted so it cannot grow unnoticed.
const NO_STRAIN_SHOWN: [&str; 7] = [
    "Borders, migration, asylum and expulsion",
    "Collective and plurality rights",
    "Defence and armed force",
    "Ecology, future generations and commons",
    "Emergency and resilience",
    "Knowledge, communication and culture",
    "Non-human animals",
];

/// Domains shown only under strain, with no ordinary operation anywhere.
const NO_ORDINARY_SHOWN: [&str; 1] = ["Life course, family, care and reproduction"];

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
    let rows = coverage(&records(&context).expect("ledger source"));
    let no_strain: BTreeSet<_> = rows
        .iter()
        .filter(|(_, (_, trouble))| *trouble == 0)
        .map(|(domain, _)| domain.clone())
        .collect();
    let no_ordinary: BTreeSet<_> = rows
        .iter()
        .filter(|(_, (works, _))| *works == 0)
        .map(|(domain, _)| domain.clone())
        .collect();
    assert_eq!(
        no_strain,
        NO_STRAIN_SHOWN.iter().map(ToString::to_string).collect(),
        "the set of domains shown only working changed. Closing one is progress \
         and belongs in this list; a new one appearing is a domain that landed \
         without its failure case."
    );
    assert_eq!(
        no_ordinary,
        NO_ORDINARY_SHOWN.iter().map(ToString::to_string).collect(),
        "the set of domains shown only under strain changed"
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
