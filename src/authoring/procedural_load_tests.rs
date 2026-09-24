// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

fn inputs() -> (Context, Source, String) {
    let context = Context::discover().expect("repository");
    let source = source(&context).expect("procedural-load source");
    let constitution = context.read(CONSTITUTION).expect("constitution");
    (context, source, constitution)
}

#[test]
fn every_effect_is_classified_and_every_class_is_used() {
    let (_, source, constitution) = inputs();
    let analysis = analyse(&source, &constitution).expect("the classification covers the source");
    for class in source.classes.keys() {
        assert!(
            analysis.classes.values().any(|(c, _, _)| c == class),
            "no effect is classified {class}; a class nobody uses is not a finding"
        );
    }
}

#[test]
fn an_unclassified_record_kind_fails() {
    let (_, mut source, constitution) = inputs();
    let at = source
        .effects
        .iter()
        .position(|e| e.head == "complete")
        .expect("a record kind");
    let removed = source.effects.remove(at);
    let error = analyse(&source, &constitution)
        .err()
        .expect("a record kind nobody classified must fail");
    assert!(
        error.to_string().contains(&removed.key),
        "the failure names the record: {error}"
    );
}

#[test]
fn a_classification_naming_no_effect_fails() {
    let (_, mut source, constitution) = inputs();
    let mut stale = source.effects[0].clone();
    stale.key = String::from("NoSuchRecordKind");
    source.effects.push(stale);
    assert!(
        analyse(&source, &constitution).is_err(),
        "a classification that names nothing in the constitution must fail"
    );
}

#[test]
fn a_role_with_no_function_fails() {
    let (_, mut source, constitution) = inputs();
    source.functions.retain(|(pattern, _)| pattern != "Source");
    assert!(
        analyse(&source, &constitution).is_err(),
        "an authority the table cannot name must fail rather than vanish"
    );
}

/// The measurement has to find the duplication where it is known to exist:
/// an accommodation is attested by a source, an evidence attester and an
/// independent reviewer writing the same fields.
#[test]
fn a_repeated_attestation_is_found_where_it_exists() {
    let (_, source, constitution) = inputs();
    let analysis = analyse(&source, &constitution).expect("analysis");
    let key = (
        String::from("SUBSTANTIVE-EQUALITY-ORDINARY"),
        String::from("complete"),
        String::from("ReviewedReasonableAccommodation"),
    );
    let effect = &analysis.effects[&key];
    assert_eq!(analysis.classes[&key].0, "beneficial");
    assert!(
        effect.repeated.iter().any(|group| group.len() == 3),
        "the three matching attesters are not seen: {:?}",
        effect.repeated
    );
}

#[test]
fn the_report_is_current() {
    let (context, source, constitution) = inputs();
    let analysis = analyse(&source, &constitution).expect("analysis");
    assert_eq!(
        context.read(REPORT).expect("report"),
        render(&source, &analysis).expect("render"),
        "procedural-load.md is stale; run ./generate.sh procedural-load"
    );
}
