// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

// Small DEVELOPMENT double for the external state machine, not a replacement
// constitution. The final integration test below loads the full actual source.
const MODEL: &str = r#"
admits("observe"). admits("authorized"). admits("person").
derived_only("complete").
all $writer: all $record: all $candidate: authorized($writer, AmendmentSourceBindingAuthority, $record) & observe($writer, $record, $candidate, AmendmentCandidateScope) -> complete($record, AmendmentCertifiedCandidate, $candidate).
all $writer: all $record: all $candidate: complete($record, AmendmentCertifiedCandidate, $candidate) & observe($writer, $record, $candidate, AmendmentPublishedCandidateScope) -> complete($record, AmendmentPublishedCandidate, $candidate).
all $writer: all $record: all $candidate: complete($record, AmendmentPublishedCandidate, $candidate) & observe($writer, $record, $candidate, AmendmentSelectedCandidateScope) -> complete($record, AmendmentEffectiveVersion, $candidate).
person(OriginalPerson).
"#;

fn version(id: &str, source: &str) -> Version {
    Version {
        id: id.into(),
        source: source.into(),
    }
}
fn host() -> Host {
    Host::new(
        version("EnactBase", MODEL),
        "EnactJurisdiction".into(),
        "EnactSlot".into(),
    )
    .unwrap()
}

fn review(base: Version, candidate: Version, record: &str, transition: &str) -> Review {
    Review {
        record: record.into(),
        transition: transition.into(),
        binder: "EnactBinder".into(),
        effect_review: "EnactEffects".into(),
        base: base.clone(),
        candidate: candidate.clone(),
        vocabulary: VocabularyReview::Unchanged,
        effects: vec![Effect {
            query: "person(OriginalPerson).".into(),
            before: true,
            after: true,
        }],
        evidence: format!(
            "authorized(EnactBinder, AmendmentSourceBindingAuthority, {record}).\nobserve(EnactBinder, {record}, {}, AmendmentBaseScope).\nobserve(EnactBinder, {record}, {}, AmendmentCandidateScope).\nobserve(EnactBinder, {record}, {transition}, AmendmentTransitionScope).\nobserve(EnactBinder, {record}, EnactEffects, AmendmentEffectReviewScope).\nobserve(EnactBinder, {record}, EnactJurisdiction, AmendmentJurisdictionScope).\nobserve(EnactBinder, {record}, UnchangedEvidenceVocabulary, AmendmentVocabularyDispositionScope).\n",
            base.id, candidate.id
        ),
    }
}

fn next(host: &Host) -> Review {
    review(
        host.effective.clone(),
        version(
            "EnactCandidate",
            &format!("{}\nperson(NewPerson).\n", host.effective.source),
        ),
        "EnactRecord",
        "EnactTransition",
    )
}

fn publication(review: &Review) -> String {
    format!(
        "observe(Publisher, {}, {}, AmendmentPublishedCandidateScope).",
        review.record, review.candidate.id
    )
}

fn selection(review: &Review, generation: u64) -> String {
    format!(
        "authorized(Selector, AmendmentEffectiveSelectionAuthority, {}).\nobserve(Selector, {}, {}, AmendmentSelectedCandidateScope).\nobserve(Selector, {}, EnactSlot, AmendmentEffectiveSlotScope).\nobserve(Selector, {}, EnactJurisdiction, AmendmentSelectedJurisdictionScope).\nobserve(Selector, {}, Generation{generation}, AmendmentSelectionGenerationScope).",
        review.record,
        review.record,
        review.candidate.id,
        review.record,
        review.record,
        review.record
    )
}

fn activate(host: &mut Host, review: Review) {
    let generation = host.generation + 1;
    host.certify(review.clone(), review.candidate.clone())
        .unwrap();
    host.publish(
        &review.transition,
        &review.candidate.source,
        &publication(&review),
    )
    .unwrap();
    host.select(
        &review.transition,
        "Selector",
        &selection(&review, generation),
    )
    .unwrap();
}

fn rejected<T>(result: Result<T>, needle: &str) {
    match result {
        Ok(_) => panic!("unexpected acceptance; expected {needle}"),
        Err(error) => assert!(error.contains(needle), "{error} (expected {needle})"),
    }
}

#[test]
fn approval_publication_and_effective_selection_are_distinct() {
    let mut host = host();
    let review = next(&host);
    host.certify(review.clone(), review.candidate.clone())
        .unwrap();
    assert_eq!(host.effective.id, "EnactBase");
    rejected(
        host.select(&review.transition, "Selector", &selection(&review, 1)),
        "not published",
    );
    host.publish(
        &review.transition,
        &review.candidate.source,
        &publication(&review),
    )
    .unwrap();
    assert_eq!(host.effective.id, "EnactBase");
    host.select(&review.transition, "Selector", &selection(&review, 1))
        .unwrap();
    assert_eq!(host.effective(), &review.candidate);
    assert!(
        host.query(&host.lease(), &review.candidate, "person(NewPerson).")
            .unwrap()
    );
    assert_eq!(host.history()[0].review, review);
}

#[test]
fn stale_base_and_one_byte_substitutions_fail_without_state_changes() {
    let mut host = host();
    let review = next(&host);
    let mut submitted = review.candidate.clone();
    submitted.source.push('\n');
    rejected(host.certify(review.clone(), submitted), "reviewed bytes");
    let mut stale = review.clone();
    stale.base.source.push('\n');
    rejected(
        host.certify(stale.clone(), stale.candidate.clone()),
        "stale",
    );
    stale = review.clone();
    stale.base.id = "EarlierBase".into();
    rejected(
        host.certify(stale.clone(), stale.candidate.clone()),
        "stale",
    );
    assert!(host.candidates.is_empty());
    assert!(host.history().is_empty());
    assert_eq!(host.effective.id, "EnactBase");
}

#[test]
fn divergent_transition_or_version_id_cannot_replace_a_certified_candidate() {
    let mut host = host();
    let first = next(&host);
    host.certify(first.clone(), first.candidate.clone())
        .unwrap();
    let mut divergent = first.clone();
    divergent
        .candidate
        .source
        .push_str("\nperson(DivergentPerson).\n");
    rejected(
        host.certify(divergent.clone(), divergent.candidate.clone()),
        "replayed or divergent",
    );
    divergent.transition = "AnotherTransition".into();
    rejected(
        host.certify(divergent.clone(), divergent.candidate.clone()),
        "reused version",
    );
    assert_eq!(host.candidates[&first.transition].review, first);
}

#[test]
fn competing_successors_are_serialized_against_the_effective_predecessor() {
    let mut host = host();
    let first = next(&host);
    let second = review(
        host.effective.clone(),
        version(
            "SecondCandidate",
            &format!("{MODEL}\nperson(SecondPerson)."),
        ),
        "SecondRecord",
        "SecondTransition",
    );
    for r in [&first, &second] {
        host.certify(r.clone(), r.candidate.clone()).unwrap();
        host.publish(&r.transition, &r.candidate.source, &publication(r))
            .unwrap();
    }
    host.select(&first.transition, "Selector", &selection(&first, 1))
        .unwrap();
    rejected(
        host.select(&second.transition, "Selector", &selection(&second, 1)),
        "stale base",
    );
    rejected(
        host.certify(first.clone(), first.candidate.clone()),
        "replayed",
    );
    assert_eq!(host.history.len(), 1);
    assert_eq!(host.effective, first.candidate);
}

#[test]
fn semantic_before_and_after_expectations_are_both_executed() {
    for before_failure in [true, false] {
        let mut host = host();
        let mut review = next(&host);
        review.effects = vec![Effect {
            query: "person(NewPerson).".into(),
            before: before_failure,
            after: before_failure,
        }];
        rejected(
            host.certify(review.clone(), review.candidate.clone()),
            "bounded check",
        );
        assert!(host.candidates.is_empty());
    }
}

#[test]
fn empty_or_malformed_review_cannot_authorize() {
    let mut host = host();
    let mut r = next(&host);
    r.effects.clear();
    rejected(host.certify(r.clone(), r.candidate.clone()), "empty");
    r = next(&host);
    r.record = "Injected). person(Attacker".into();
    rejected(host.certify(r.clone(), r.candidate.clone()), "identifier");
    r = next(&host);
    r.effects[0].query = "not a query >>>".into();
    assert!(host.certify(r.clone(), r.candidate.clone()).is_err());
}

#[test]
fn effect_check_rejects_nondefinitive_results() {
    use nibli_engine::{EngineResourceKind, EngineUnknownReason};
    for verdict in [
        EngineQueryResult::Unknown(EngineUnknownReason::IncompleteKnowledge),
        EngineQueryResult::ResourceExceeded(EngineResourceKind::Depth),
    ] {
        for expected in [true, false] {
            assert!(compare_verdict(&verdict, "person(OriginalPerson).", expected).is_err());
        }
    }
}

#[test]
fn contradictions_and_incomplete_scans_are_not_clean_results() {
    rejected(
        fresh(&format!("{MODEL}\n~person(OriginalPerson)."), ""),
        "contradiction check",
    );
    let session = fresh(MODEL, "").unwrap();
    session.kb().require_recovery("injected uncertainty".into());
    rejected(scan(&session), "incomplete");
}

#[test]
fn evidence_cannot_inject_rules_admissions_or_derived_certificates() {
    for injection in [
        "admits(\"rich\").",
        "person(OriginalPerson).",
        "complete(EnactRecord, AmendmentCertifiedCandidate, EnactCandidate).",
        "all $x: person($x) -> person(Attacker).",
        ":require false",
        "observe(A, B, C, D). person(Attacker).",
    ] {
        let mut host = host();
        let mut review = next(&host);
        review.evidence += &format!("\n{injection}");
        rejected(
            host.certify(review.clone(), review.candidate.clone()),
            "only ground",
        );
    }
}

#[test]
fn vocabulary_and_derived_only_changes_need_explicit_exact_review() {
    for changed in [
        MODEL.replace(
            "admits(\"person\").",
            "admits(\"person\"). admits(\"rich\").",
        ),
        MODEL.replace("derived_only(\"complete\").", ""),
        MODEL.replace(
            "admits(\"observe\"). admits(\"authorized\"). admits(\"person\").",
            "",
        ),
    ] {
        let mut host = host();
        let mut r = next(&host);
        r.candidate.source = changed;
        rejected(
            host.certify(r.clone(), r.candidate.clone()),
            "vocabulary differs",
        );
        let session = fresh(&r.candidate.source, "").unwrap();
        let (admitted, derived_only) = vocabulary(&session);
        r.vocabulary = VocabularyReview::Reviewed {
            admitted,
            derived_only,
        };
        r.evidence = r.evidence.replace(
            "UnchangedEvidenceVocabulary",
            "ExplicitlyReviewedVocabularyChange",
        );
        host.certify(r.clone(), r.candidate.clone()).unwrap();
    }
}

#[test]
fn changed_bytes_cannot_supply_their_own_authorization() {
    let mut host = host();
    let mut review = next(&host);
    review.evidence = review
        .evidence
        .lines()
        .filter(|l| !l.starts_with("authorized("))
        .map(|l| format!("{l}\n"))
        .collect();
    review.candidate.source.push_str("\nall $x: person($x) -> complete(EnactRecord, AmendmentCertifiedCandidate, EnactCandidate).\n");
    rejected(
        host.certify(review.clone(), review.candidate.clone()),
        "bounded check",
    );
}

#[test]
fn publication_mismatch_and_wrong_selection_slot_or_generation_preserve_state() {
    let mut host = host();
    let r = next(&host);
    host.certify(r.clone(), r.candidate.clone()).unwrap();
    rejected(
        host.publish(&r.transition, MODEL, &publication(&r)),
        "byte mismatch",
    );
    assert!(host.candidates[&r.transition].publication.is_none());
    host.publish(&r.transition, &r.candidate.source, &publication(&r))
        .unwrap();
    for bad in [
        selection(&r, 0),
        selection(&r, 1).replace("EnactSlot", "WrongSlot"),
        selection(&r, 1).replace("EnactJurisdiction", "WrongJurisdiction"),
    ] {
        rejected(
            host.select(&r.transition, "Selector", &bad),
            "bounded check",
        );
        assert_eq!(host.effective.id, "EnactBase");
        assert!(host.history.is_empty());
    }
}

#[test]
fn rollback_needs_fresh_authority_and_does_not_revive_old_query_leases() {
    let mut host = host();
    let original = host.effective.clone();
    let old_lease = host.lease();
    let first = next(&host);
    activate(&mut host, first.clone());
    let old_new_lease = host.lease();
    rejected(
        host.query(&old_lease, &host.effective, "person(OriginalPerson)."),
        "stale lease",
    );
    let r = review(
        host.effective.clone(),
        version("RollbackOccurrence", &original.source),
        "RollbackRecord",
        "RollbackTransition",
    );
    activate(&mut host, r.clone());
    assert_eq!(host.effective.source, original.source);
    assert_ne!(host.effective.id, original.id);
    assert_eq!(host.version(&original.id), Some(&original));
    assert_eq!(host.version(&first.candidate.id), Some(&first.candidate));
    assert_eq!(host.history.len(), 2);
    assert_eq!(host.history[1].predecessor, first.candidate.id);
    for lease in [&old_lease, &old_new_lease] {
        rejected(
            host.query(lease, &host.effective, "person(OriginalPerson)."),
            "stale lease",
        );
    }
    assert!(
        !host
            .query(&host.lease(), &host.effective, "person(NewPerson).")
            .unwrap()
    );
    let mut forged = host.effective.clone();
    forged.source.push('\n');
    rejected(
        host.query(&host.lease(), &forged, "person(OriginalPerson)."),
        "wrong source",
    );
    rejected(
        host.query(&host.lease(), &original, "person(OriginalPerson)."),
        "wrong source",
    );
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "full-source execution: cargo test --release --bin amendment-assurance"
)]
fn actual_constitution_result_to_exact_source_to_fresh_query() {
    // The debug compiler's full-source traversal exceeds libtest's small
    // default stack. Exercise the same bounded worker stack as the host CLI.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(actual_constitution_transition)
        .unwrap()
        .join()
        .unwrap();
}

fn actual_constitution_transition() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap();
    let source = read("new-book-plans/constitution.nibli");
    let evidence = read("tests/pins/amendment-enactment/certified/fixture.nibli");
    let pub_evidence = read("tests/pins/amendment-enactment/published/fixture.nibli");
    let selected = read("tests/pins/amendment-enactment/effective/fixture.nibli")
        .replace("EnactGeneration", "Generation1");
    let mut host = Host::new(
        version("EnactBase", &source),
        "EnactJurisdiction".into(),
        "EnactSlot".into(),
    )
    .unwrap();
    let mut r = review(
        host.effective.clone(),
        version(
            "EnactCandidate",
            &format!("{source}\nperson(EnactNewPerson).\n"),
        ),
        "EnactRecord",
        "EnactTransition",
    );
    r.evidence = evidence;
    r.effects = vec![
        Effect {
            query: "person(EnactNewPerson).".into(),
            before: false,
            after: true,
        },
        Effect {
            query: "entitled(Adam, event { eats() }).".into(),
            before: true,
            after: true,
        },
    ];
    host.certify(r.clone(), r.candidate.clone()).unwrap();
    host.publish(&r.transition, &r.candidate.source, &pub_evidence)
        .unwrap();
    host.select(&r.transition, "EnactSelector", &selected)
        .unwrap();
    assert!(
        host.query(&host.lease(), &r.candidate, "person(EnactNewPerson).")
            .unwrap()
    );
}
