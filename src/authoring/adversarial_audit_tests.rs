// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// The lenses the item declares. All fifteen, by name, so dropping one is a
/// failure rather than a smaller audit nobody noticed.
const DECLARED: [&str; 15] = [
    "care-life-course",
    "constitutional-law",
    "consumer-civil-justice",
    "data-ai-governance",
    "defence-external",
    "disability-accessibility",
    "ecology",
    "infrastructure",
    "labour-economy",
    "local-migration-collective",
    "media-science-culture",
    "policing-prison",
    "public-administration",
    "public-health",
    "quantitative-modelling",
];

#[test]
fn every_declared_lens_encodes_something_that_exists_and_finds_something() {
    let context = Context::discover().expect("repository");
    let lenses = lenses(&context).expect("audit source");
    validate(&context, &lenses).expect("every lens binds to a check and a finding");
    let present: BTreeSet<_> = lenses.iter().map(|lens| lens.id.clone()).collect();
    assert_eq!(
        present,
        DECLARED.iter().map(ToString::to_string).collect(),
        "the declared lens set changed; the item names fifteen"
    );
}

#[test]
fn the_audit_has_not_stopped_looking() {
    let context = Context::discover().expect("repository");
    let lenses = lenses(&context).expect("audit source");
    let open = lenses
        .iter()
        .flat_map(|lens| &lens.findings)
        .filter(|finding| finding.open)
        .count();
    assert!(
        open > 0,
        "no finding is open. Disclosure is not closure and an audit with nothing \
         open is an audit that stopped looking."
    );
    // Every kind the audit says it looks for is actually raised by somebody.
    for category in CATEGORIES {
        assert!(
            lenses
                .iter()
                .any(|lens| lens.findings.iter().any(|f| f.category == category)),
            "'{category}' is declared and never raised"
        );
    }
}
