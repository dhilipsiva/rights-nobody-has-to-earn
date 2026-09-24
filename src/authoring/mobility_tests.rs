// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn interested_holder_requester_and_representative_cannot_review_own_record() {
    let cards = contracts::cards();
    for (id, interested) in [
        ("membership", "$holder"),
        ("private-access", "$requester"),
        ("consent", "$representative"),
    ] {
        let c = card(&cards, id);
        let mut v = values(&cards, c, "MPInterested");
        v.insert("$review".into(), v[interested].clone());
        let session = nibli_session::CoreSession::new();
        session.assert_text("public(Court).").unwrap();
        let beneficial = super::super::procedural_load::beneficial_kinds(&Context::discover().unwrap()).unwrap();
        session.assert_text(&rules(&cards, &beneficial).join("\n")).unwrap();
        session.assert_text(&fixture(&cards, c, &v)).unwrap();
        let result = session
            .query_text(&format!("{}.", ground(&heads(c)[0], &v)))
            .unwrap();
        assert!(
            matches!(result, nibli_engine::EngineQueryResult::False),
            "{id}: {result:?}"
        );
    }
}

#[test]
fn scoped_authority_is_not_unary_permanent_answerability() {
    let session = nibli_session::CoreSession::new();
    session.assert_text("public(MPBody).\npublic(MPBody) -> authority(MPOperator, RightsBoundedInternalDecision, MPRecord).\n").unwrap();
    assert!(matches!(
        session
            .query_text("authority(MPOperator, RightsBoundedInternalDecision, MPRecord).")
            .unwrap(),
        nibli_engine::EngineQueryResult::True
    ));
    assert!(matches!(
        session.query_text("authority(MPOperator).").unwrap(),
        nibli_engine::EngineQueryResult::False
    ));
}

#[test]
fn cards_have_distinct_effects_and_no_personal_or_coercive_heads() {
    let cards = contracts::cards();
    assert_eq!(
        cards.iter().map(|c| c.id).collect::<BTreeSet<_>>().len(),
        cards.len()
    );
    assert_eq!(
        cards.iter().map(|c| c.kind).collect::<BTreeSet<_>>().len(),
        cards.len()
    );
    let forbidden = Regex::new(r"^(person|prisoner|free|travel|decide|reward|false|lose|eats|dwell|healthy|secure|suffice|believe|expresses|learn|meets)\(").unwrap();
    for c in &cards {
        assert!(heads(c).iter().all(|h| !forbidden.is_match(h)), "{}", c.id);
        assert_eq!(
            fields(c).len(),
            fields(c)
                .iter()
                .map(|(_, s)| s)
                .collect::<BTreeSet<_>>()
                .len()
        );
        for dep in &c.dependencies {
            assert_ne!(c.id, *dep);
            let _ = card(&cards, dep);
        }
        assert!(premises(&cards, c).contains(&INDEPENDENT.to_string()));
    }
}

#[test]
fn generation_is_explicit_idempotent_and_preserves_other_families() {
    let live = Context::discover().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let context = Context::from_test_root(dir.path().to_owned());
    std::fs::create_dir_all(context.path("book-1/source")).unwrap();
    let original = live.read("book-1/source/constitution.nibli").unwrap();
    std::fs::write(context.path("book-1/source/constitution.nibli"), &original).unwrap();
    // The generator reads the reviewed classification to find the effects
    // ruling D6 lets one actor give.
    std::fs::write(
        context.path("book-1/source/procedural-load-source.json"),
        live.read("book-1/source/procedural-load-source.json").unwrap(),
    )
    .unwrap();
    let mut first = Export::new();
    generate(&context, &mut first).unwrap();
    let generated = context.read("book-1/source/constitution.nibli").unwrap();
    let mut second = Export::new();
    generate(&context, &mut second).unwrap();
    assert_eq!(
        generated,
        context.read("book-1/source/constitution.nibli").unwrap()
    );
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    assert!(first.cases.iter().all(|c| c.scan));
    assert_eq!(
        original
            .split_once(BEGIN)
            .map_or(original.trim_end(), |(before, _)| before.trim_end()),
        generated.split_once(BEGIN).unwrap().0.trim_end()
    );
    for c in &first.cases {
        for path in &c.fixtures {
            for line in context
                .read(path)
                .unwrap()
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
            {
                assert!(line.ends_with('.'));
                assert!(!line.contains('$'), "ungrounded {line}");
                assert!(!line.contains("->"), "fixture must not change constitution");
            }
        }
    }
}

#[test]
fn consumer_dependencies_join_exact_identity_and_project_fields() {
    let cards = contracts::cards();
    for c in &cards {
        for id in &c.dependencies {
            let dep = card(&cards, id);
            let fields = joins(c, dep)
                .into_iter()
                .map(|(_, s)| s)
                .collect::<BTreeSet<_>>();
            for scope in SHARED {
                assert!(fields.contains(scope), "{} lacks {scope}", c.id);
            }
            if c.id == "consented-effect" {
                for scope in ["Project", "ProjectVersion", "Effect"] {
                    assert!(fields.contains(scope));
                }
            }
        }
    }
}

#[test]
fn actual_consumers_do_not_turn_group_or_mobility_findings_into_person_consequences() {
    let cards = contracts::cards();
    let beneficial = super::super::procedural_load::beneficial_kinds(&Context::discover().unwrap()).unwrap();
    let allowed = rules(&cards, &beneficial).into_iter().collect::<BTreeSet<_>>();
    let named =
        Regex::new(r"\b(MP[A-Z]\w*|Mobility\w*|Plurality\w*|ExternalArrangementCompatibility)\b")
            .unwrap();
    let generic =
        Regex::new(r"\b(complete|authority|permits|obliged|related)\(\$\w+,\s*\$\w+(,\s*\$\w+)?\)")
            .unwrap();
    let person_head = Regex::new(
        r"->\s*(person|false|prisoner|severe|free|travel|owe|entitled|decide|lose|reward)\(",
    )
    .unwrap();
    // A line naming a mobility or plurality constant is accepted when it is one
    // of this family's own rules, and otherwise only while it concludes nothing
    // about a person. Requiring rule IDENTITY was the older test, and it stopped
    // describing the property once other families began rejoining these scopes:
    // the public-safety border-hold rules read `MP*` record scopes and conclude a
    // record completion, and the ecological single-value machinery registers
    // `MP*` scopes it reuses. Both were reviewed, neither concludes a person
    // head, and neither is this test's subject. Its subject is the head.
    let accepts = |text: &str| {
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .all(|line| {
                if named.is_match(line) {
                    allowed.contains(line) || !person_head.is_match(line)
                } else {
                    !(generic.is_match(line) && person_head.is_match(line))
                }
            })
    };
    // The allow-list half is an escape hatch for a mobility rule that
    // deliberately concludes about a person. No rule in this family does, so it
    // never fires — assert that rather than leave it inert, because a disjunct
    // nobody checks is how a guard goes quietly vacuous.
    assert!(
        !allowed.iter().any(|rule| person_head.is_match(rule)),
        "a mobility rule now concludes a person head; review it, because this \
         hatch is then the only thing admitting it"
    );
    let source = Context::discover()
        .unwrap()
        .read("book-1/source/constitution.nibli")
        .unwrap();
    assert!(accepts(&source), "unreviewed mobility/plurality consumer");
    for hostile in [
        // Named finding constants, which the generic shape cannot see.
        "all $record: all $holder: complete($record, PluralityMembershipOccurrence, $holder) -> false($holder).",
        "all $record: all $holder: complete($record, PluralityActualConsent, $holder) -> prisoner($holder).",
        // No mobility token at all: caught by the generic all-variable shape.
        "all $record: all $kind: all $holder: complete($record, $kind, $holder) -> decide($holder, Ballot).",
        // The shape the narrowing must still bite on: another family rejoining a
        // mobility record scope and concluding about the person rather than the
        // record. This is exactly what the reviewed cross-family lines do not do.
        "all $writer: all $record: all $holder: observe($writer, $record, $holder, MPAccessScope) -> prisoner($holder).",
        "all $writer: all $record: all $holder: observe($writer, $record, $holder, MPAccommodationScope) -> lose(Points, $holder).",
    ] {
        nibli_session::CoreSession::new()
            .compile_text(hostile)
            .unwrap();
        assert!(!accepts(&format!("{source}\n{hostile}")));
    }
}
