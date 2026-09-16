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

/// The tracker's done-when, made mechanical: an open finding is closed,
/// narrowed, or carries a limitation and a gate consequence. Leaving it open
/// with neither is the disclosure-as-ending move the resolution receipts refuse,
/// and it now fails generation rather than reading as candour.
#[test]
fn every_open_finding_says_what_it_costs() {
    let context = Context::discover().expect("repository");
    let lenses = lenses(&context).expect("audit source");
    for lens in &lenses {
        for finding in &lens.findings {
            if finding.open {
                let disposition = finding
                    .disposition
                    .as_deref()
                    .unwrap_or_else(|| panic!("{}: open with no disposition", lens.id));
                assert!(
                    DISPOSITIONS.contains(&disposition),
                    "{}: {disposition} is not a declared disposition",
                    lens.id
                );
                let consequence = finding
                    .consequence
                    .as_deref()
                    .unwrap_or_else(|| panic!("{}: open with no consequence", lens.id));
                assert!(
                    consequence.len() > 40,
                    "{}: a consequence this short is not one",
                    lens.id
                );
            } else {
                assert!(
                    finding.disposition.is_none() && finding.consequence.is_none(),
                    "{}: a closed finding carries no disposition; if it still \
                     costs something it is not closed",
                    lens.id
                );
            }
        }
    }

    // Sabotage: an open finding with nothing attached must fail validation.
    let mut hostile = lenses.clone();
    let victim = hostile
        .iter_mut()
        .flat_map(|lens| &mut lens.findings)
        .find(|finding| finding.open)
        .expect("an open finding");
    victim.disposition = None;
    victim.consequence = None;
    assert!(
        validate(&context, &hostile).is_err(),
        "an open finding with no disposition passed validation"
    );
}
