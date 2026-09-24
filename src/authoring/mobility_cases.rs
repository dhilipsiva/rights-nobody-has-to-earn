// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

fn without(facts: &str, record: &str, scope: &str) -> String {
    facts
        .lines()
        .filter(|line| {
            !(line.contains(&format!(", {record}, "))
                && line.ends_with(&format!(", MP{scope}Scope).")))
        })
        .map(|line| format!("{line}\n"))
        .collect()
}

fn counterfactual(export: &mut Export, authored: &[String], name: &str, removed: &str) {
    let edits = authored
        .iter()
        .filter(|r| r.contains(removed))
        .map(|r| Edit {
            before: r.clone(),
            after: r.replace(&format!(" & {removed}"), ""),
        })
        .collect::<Vec<_>>();
    assert!(!edits.is_empty(), "missing counterfactual {removed}");
    assert!(edits.iter().all(|e| e.before != e.after));
    export.bases.insert(
        name.into(),
        Base {
            base: Some("live".into()),
            edits,
            path: None,
        },
    );
}

fn completion(card: &Card, values: &Values, expected: bool) -> String {
    query(&ground(&heads(card)[0], values), expected)
}

pub(super) fn generate(
    context: &Context,
    export: &mut Export,
    cards: &[Card],
    authored: &[String],
) -> Result<(), Error> {
    for (name, atom) in [
        ("mobility-no-independent-review", INDEPENDENT),
        (
            "mobility-no-self-identification",
            "observe($holder, $self_record, $object, MPSelfIdentifiedCommunityScope)",
        ),
        (
            "mobility-no-consent",
            "complete($consent_record, PluralityActualConsent, $object)",
        ),
        (
            "mobility-no-project-binding",
            "observe($consent_source, $consent_record, $project_version, MPProjectVersionScope)",
        ),
        (
            "mobility-no-nonexistential-limit",
            "observe($review, $record, OtherMaterialNonexistentialEffect, MPEffectScope)",
        ),
    ] {
        counterfactual(export, authored, name, atom);
    }

    let beneficial = beneficial_kinds(context)?;
    for card in cards {
        let v = values(cards, card, "MPCase");
        let facts = fixture(cards, card, &v);
        add_case(
            context,
            export,
            &format!("{}/positive", card.id),
            "live",
            &facts,
            &queries(card, &v, true),
        )?;
        add_case(
            context,
            export,
            &format!("{}/withheld", card.id),
            "live",
            "",
            &queries(card, &v, false),
        )?;

        // Several genuinely isolated subjects per case amortize compilation.
        // No expectation is skipped; every common and kind-specific field is
        // removed in turn while all the others and real dependencies remain.
        for (batch, group) in fields(card).chunks(6).enumerate() {
            let mut all_facts = String::new();
            let mut pins = String::new();
            for (index, (_, scope)) in group.iter().enumerate() {
                let v = values(cards, card, &format!("MPOmit{batch}Row{index}"));
                all_facts += &without(&fixture(cards, card, &v), &v["$record"], scope);
                pins += &format!("# Withhold {scope}; every other field remains.\n");
                pins += &completion(card, &v, false);
            }
            add_case(
                context,
                export,
                &format!("{}/required-fields-{batch}", card.id),
                "live",
                &all_facts,
                &pins,
            )?;
        }
        let mut all_facts = String::new();
        let mut pins = String::new();
        for (i, (actor, role)) in ATTESTERS.iter().chain(READERS).enumerate() {
            let v = values(cards, card, &format!("MPUnauth{i}"));
            let facts = fixture(cards, card, &v).replace(
                &ground(&format!("authorized({actor}, {role}, $record).\n"), &v),
                "",
            );
            all_facts += &facts;
            pins += &completion(card, &v, false);
        }
        add_case(
            context,
            export,
            &format!("{}/writer-authority", card.id),
            "live",
            &all_facts,
            &pins,
        )?;

        let mut all_facts = String::new();
        let mut pins = String::new();
        let mut fusions = vec![
            ("$review", "$source"),
            ("$review", "$operator"),
            ("$reader", "$source"),
            ("$alternate", "$reader"),
            ("$auditor", "$operator"),
            ("$review", "$holder"),
            ("$reader", "$holder"),
            ("$alternate", "$holder"),
            ("$auditor", "$holder"),
        ];
        if card
            .fields
            .iter()
            .any(|(value, _)| *value == "$representative")
        {
            fusions.push(("$review", "$representative"));
        }
        if matches!(card.id, "private-access" | "nonresponse") {
            fusions.push(("$review", "$requester"));
        }
        for (i, (actor, other)) in fusions.iter().enumerate() {
            let mut v = values(cards, card, &format!("MPFusion{i}"));
            v.insert((*actor).into(), v[*other].clone());
            all_facts += &fixture(cards, card, &v);
            pins += &completion(card, &v, false);
        }
        add_case(
            context,
            export,
            &format!("{}/independence", card.id),
            "live",
            &all_facts,
            &pins,
        )?;
        let mut fused = v.clone();
        fused.insert("$review".into(), v["$source"].clone());
        add_case(
            context,
            export,
            &format!("{}/counterfactual-self-review", card.id),
            "mobility-no-independent-review",
            &fixture(cards, card, &fused),
            &completion(card, &fused, true),
        )?;

        // A contradictory authorized value blocks instead of offering a second
        // usable tuple. Unauthorized noise does not acquire that veto.
        let conflict = ground(
            "observe($review, $record, WrongVersion, MPSourceVersionScope).\n",
            &v,
        );
        add_case(
            context,
            export,
            &format!("{}/conflict-sequence", card.id),
            "live",
            &facts,
            &(completion(card, &v, true) + &conflict + &completion(card, &v, false)),
        )?;
        let noise = ground(
            "observe(UncredentialedWriter, $record, WrongVersion, MPSourceVersionScope).\n",
            &v,
        );
        add_case(
            context,
            export,
            &format!("{}/unauthorized-noise", card.id),
            "live",
            &(facts.clone() + &noise),
            &completion(card, &v, true),
        )?;
        let stale = facts.replace("CurrentReconciledFinding", "SupersededFinding");
        add_case(
            context,
            export,
            &format!("{}/stale", card.id),
            "live",
            &stale,
            &completion(card, &v, false),
        )?;

        let helped = beneficial.contains(card.kind);
        add_case(
            context,
            export,
            &format!("{}/single-actor", card.id),
            "live",
            &single_actor_facts(&facts, &v),
            &fast_queries(card, &v, helped),
        )?;
        if helped {
            add_case(
                context,
                export,
                &format!("{}/single-actor-withdrawn-on-review", card.id),
                "live",
                &(single_actor_facts(&facts, &v) + &withdrawal(&v)),
                &fast_queries(card, &v, false),
            )?;
        }
        for dep in &card.dependencies {
            let dep = self::card(cards, dep);
            let d = dependency_values(cards, card, dep, &v);
            let no = facts.replace(&own_facts(cards, dep, &d), "");
            assert_ne!(no, facts, "dependency omission {} -> {}", card.id, dep.id);
            add_case(
                context,
                export,
                &format!("{}/without-{}", card.id, dep.id),
                "live",
                &no,
                &completion(card, &v, false),
            )?;
        }
    }
    membership(context, export, cards)?;
    collective_effects(context, export, cards)?;
    defects(context, export, cards)?;
    boundaries(context, export, cards)?;
    rights_and_continuity(context, export, cards)?;
    Ok(())
}

fn rights_and_continuity(
    context: &Context,
    export: &mut Export,
    cards: &[Card],
) -> Result<(), Error> {
    let member = card(cards, "membership");
    let title = card(cards, "title");
    let internal = card(cards, "internal-law");
    let mut m = values(cards, member, "MPPersonMembership");
    m.insert("$holder".into(), "MPAdultMember".into());
    let mut t = values(cards, title, "MPSeparateTitle");
    t.insert("$holder".into(), m["$object"].clone());
    let mut i = values(cards, internal, "MPAutonomy");
    i.insert("$holder".into(), m["$object"].clone());
    let mut facts = "at(MPAdultMember, RepublicJurisdiction).\nat(MPAdultMember, GeneralAdulthood).\nat(MPNonmemberResident, RepublicJurisdiction).\nat(MPNonmemberResident, GeneralAdulthood).\n".to_owned();
    facts += &fixture(cards, member, &m);
    facts += &fixture(cards, title, &t);
    facts += &fixture(cards, internal, &i);
    let rights = |holder: &str| {
        let mut q = String::new();
        for atom in [
            format!("person({holder})"),
            format!("owe(State, Eats, {holder})"),
            format!("travel({holder})"),
            format!("decide({holder}, Ballot)"),
            format!("prevents({holder}, PluralityNonmemberCommonServiceDenial)"),
        ] {
            q += &query(&atom, true);
        }
        for atom in [
            format!("false({holder})"),
            format!("prisoner({holder})"),
            format!("reward({holder})"),
            format!("decide({holder}, AdditionalGeneralGovernmentBallot)"),
        ] {
            q += &query(&atom, false);
        }
        q
    };
    let mut pins = completion(member, &m, true)
        + &completion(title, &t, true)
        + &completion(internal, &i, true);
    pins += &rights("MPAdultMember");
    pins += &rights("MPNonmemberResident");
    pins += &ground(
        "observe($holder, $record, FreeMembershipExit, MPSelfExitScope).\n",
        &m,
    );
    pins += &completion(member, &m, false);
    pins += &completion(title, &t, true);
    pins += &rights("MPAdultMember");
    pins += &rights("MPNonmemberResident");
    add_case(
        context,
        export,
        "boundaries/member-exit-title-and-nonmember-rights",
        "live",
        &facts,
        &pins,
    )?;

    // Every reviewed compatibility can cease without affecting public
    // continuity. A corrected record is a fresh occurrence, not erased history.
    let c = card(cards, "external");
    let v = values(cards, c, "MPEnd");
    let end = ground(
        "observe($review, $record, EndedFinding, MPCurrentDispositionScope).\n",
        &v,
    );
    let mut corrected = values(cards, c, "MPCorrected");
    for (var, _) in common().into_iter().filter(|(_, s)| SHARED.contains(s)) {
        corrected.insert(var.into(), v[var].clone());
    }
    corrected.insert("$window".into(), "FreshCorrectedWindow".into());
    let mut pins = completion(c, &v, true);
    pins += &end;
    pins += &completion(c, &v, false);
    pins += &fixture(cards, c, &corrected);
    pins += &completion(c, &corrected, true);
    pins += &completion(c, &v, false);
    add_case(
        context,
        export,
        "external/end-and-fresh-correction",
        "live",
        &fixture(cards, c, &v),
        &pins,
    )?;

    // Review of a domestic violation is a positive, attributable finding;
    // neither unknown supply chains nor foreign cooperation is presumed.
    for (index, kind) in [
        "ExportedLabourExploitation",
        "ExportedEcologicalDamage",
        "ExportedRightsViolation",
    ]
    .iter()
    .enumerate()
    {
        let v = values(cards, c, &format!("MPExport{index}"));
        let def = card(cards, "defect");
        let mut d = values(cards, def, &format!("MPExportDefect{index}"));
        for (var, _) in common().into_iter().filter(|(_, s)| SHARED.contains(s)) {
            d.insert(var.into(), v[var].clone());
        }
        d.insert("$target".into(), v["$record"].clone());
        d.insert("$target_source".into(), v["$source"].clone());
        d.insert("$defect".into(), (*kind).into());
        let mut pins = completion(c, &v, true);
        pins += &fixture(cards, def, &d);
        pins += &queries(def, &d, true);
        pins += &completion(c, &v, false);
        add_case(
            context,
            export,
            &format!("external/{kind}-withdraws"),
            "live",
            &fixture(cards, c, &v),
            &pins,
        )?;
    }
    Ok(())
}

fn membership(context: &Context, export: &mut Export, cards: &[Card]) -> Result<(), Error> {
    let member = card(cards, "membership");
    let v = values(cards, member, "MPMember");
    let facts = fixture(cards, member, &v);
    let self_fact = ground(
        "observe($holder, $self_record, $object, MPSelfIdentifiedCommunityScope).\n",
        &v,
    );
    let no_self = facts.replace(&self_fact, "");
    for (label, base, expected) in [
        ("acceptance-alone", "live", false),
        (
            "counterfactual-no-self",
            "mobility-no-self-identification",
            true,
        ),
    ] {
        add_case(
            context,
            export,
            &format!("membership/{label}"),
            base,
            &no_self,
            &completion(member, &v, expected),
        )?;
    }
    add_case(
        context,
        export,
        "membership/self-alone",
        "live",
        &self_fact,
        &completion(member, &v, false),
    )?;
    let mut second = values(cards, member, "MPSecondMembership");
    second.insert("$holder".into(), v["$holder"].clone());
    let exit = ground(
        "observe($holder, $record, FreeMembershipExit, MPSelfExitScope).\n",
        &v,
    );
    let mut steps = completion(member, &v, true) + &completion(member, &second, true);
    steps += &exit;
    steps += &completion(member, &v, false);
    steps += &completion(member, &second, true);
    steps += &ground(
        "? obliged(State, RecordExitWithoutRightsOrTitlePenalty, $record).\n# => TRUE\n",
        &v,
    );
    let mut renewed = values(cards, member, "MPReentry");
    renewed.insert("$holder".into(), v["$holder"].clone());
    renewed.insert("$object".into(), v["$object"].clone());
    steps += &fixture(cards, member, &renewed);
    steps += &completion(member, &renewed, true);
    steps += &completion(member, &v, false);
    add_case(
        context,
        export,
        "membership/multiple-exit-reentry",
        "live",
        &(facts.clone() + &fixture(cards, member, &second)),
        &steps,
    )?;
    let other_exit = ground(
        "observe(SomeoneElse, $record, FreeMembershipExit, MPSelfExitScope).\n",
        &v,
    );
    add_case(
        context,
        export,
        "membership/other-person-cannot-exit-member",
        "live",
        &(facts + &other_exit),
        &completion(member, &v, true),
    )?;
    Ok(())
}

fn collective_effects(context: &Context, export: &mut Export, cards: &[Card]) -> Result<(), Error> {
    let effect = card(cards, "consented-effect");
    let consent = card(cards, "consent");
    for harm in contracts::HARMS {
        let mut v = values(cards, effect, "MPEffect");
        v.insert("$effect".into(), (*harm).into());
        let full = fixture(cards, effect, &v);
        let c = dependency_values(cards, effect, consent, &v);
        let no = full.replace(&own_facts(cards, consent, &c), "");
        add_case(
            context,
            export,
            &format!("effect/{harm}"),
            "live",
            &full,
            &queries(effect, &v, true),
        )?;
        add_case(
            context,
            export,
            &format!("effect/{harm}-no-consent"),
            "live",
            &no,
            &queries(effect, &v, false),
        )?;
        // Leave the consent source's binding fields, but remove its review
        // writer: actual completion fails while the raw tuple still exists.
        let no_review = full.replace(
            &ground(
                "authorized($review, MPIndependentReviewAuthority, $record).\n",
                &c,
            ),
            "",
        );
        add_case(
            context,
            export,
            &format!("effect/{harm}-counterfactual"),
            "mobility-no-consent",
            &no_review,
            &queries(effect, &v, true),
        )?;
    }
    let v = values(cards, effect, "MPChangedProject");
    let c = dependency_values(cards, effect, consent, &v);
    let full = fixture(cards, effect, &v);
    let parent = own_facts(cards, effect, &v);
    let altered_parent = parent.replace(&v["$project_version"], "DifferentProjectVersion");
    let mismatch = full.replace(&parent, &altered_parent);
    for (id, base, expected) in [
        ("changed-project", "live", false),
        (
            "counterfactual-changed-project",
            "mobility-no-project-binding",
            true,
        ),
    ] {
        add_case(
            context,
            export,
            &format!("effect/{id}"),
            base,
            &mismatch,
            &completion(effect, &v, expected),
        )?;
    }
    let conflict = ground(
        "observe($source, $record, DifferentProjectVersion, MPProjectVersionScope).\n",
        &c,
    );
    add_case(
        context,
        export,
        "effect/project-splice",
        "live",
        &(mismatch + &conflict),
        &completion(effect, &v, false),
    )?;

    for (scope, replacement) in [
        ("ConsentResult", "Silence"),
        ("ConsentResult", "TiedResult"),
        ("ConsentResult", "ConflictingResults"),
        ("DecisionCompleteness", "EmptyRoster"),
        ("DecisionCompleteness", "MissingParticipation"),
        ("ConsentQuality", "CoercedConsent"),
    ] {
        let v = values(cards, consent, "MPBadConsent");
        let facts = fixture(cards, consent, &v);
        let actual = fields(consent)
            .into_iter()
            .find(|(_, s)| *s == scope)
            .unwrap()
            .0;
        add_case(
            context,
            export,
            &format!("consent/{replacement}"),
            "live",
            &facts.replace(actual, replacement),
            &completion(consent, &v, false),
        )?;
    }
    for actor in ["$administration", "$assurer", "$result_service"] {
        let mut v = values(cards, consent, "MPPipeline");
        v.insert(actor.into(), v["$operator"].clone());
        add_case(
            context,
            export,
            &format!("consent/fused-{}", actor.trim_start_matches('$')),
            "live",
            &fixture(cards, consent, &v),
            &completion(consent, &v, false),
        )?;
    }
    for (actor, role) in [
        ("$administration", "MPDecisionAdministrationAuthority"),
        ("$assurer", "MPCompletenessAssuranceAuthority"),
        ("$result_service", "MPResultServiceAuthority"),
    ] {
        let v = values(cards, consent, "MPMissingPipeline");
        let facts = fixture(cards, consent, &v).replace(
            &ground(&format!("authorized({actor}, {role}, $record).\n"), &v),
            "",
        );
        add_case(
            context,
            export,
            &format!("consent/unauthorized-{}", actor.trim_start_matches('$')),
            "live",
            &facts,
            &completion(consent, &v, false),
        )?;
    }
    let mut alternate = values(cards, consent, "MPAlternateAssurance");
    alternate.insert(
        "$assurer".into(),
        "PredeclaredIndependentAlternateAssurer".into(),
    );
    add_case(
        context,
        export,
        "consent/alternate-completeness-attestation",
        "live",
        &fixture(cards, consent, &alternate),
        &completion(consent, &alternate, true),
    )?;
    let consultation = card(cards, "consultation");
    let v = values(cards, consultation, "MPConsult");
    let full = fixture(cards, consultation, &v);
    let late = full.replace(
        "BeforeCommitmentGoodFaithAccessibleInformationAdequateTime",
        "NoticeAfterCommitment",
    );
    add_case(
        context,
        export,
        "consultation/after-commitment",
        "live",
        &late,
        &completion(consultation, &v, false),
    )?;
    // An independently conflicting existential classification cannot be read as
    // the source's consultation-only category. Remove only that review check
    // in the explicit counterfactual, not the ordinary ambiguity guard.
    let altered = full.replace(
        &ground(
            "observe($review, $record, OtherMaterialNonexistentialEffect, MPEffectScope).\n",
            &v,
        ),
        "",
    );
    for (id, base, expected) in [
        ("without-effect-review", "live", false),
        (
            "counterfactual-effect-review",
            "mobility-no-nonexistential-limit",
            true,
        ),
    ] {
        add_case(
            context,
            export,
            &format!("consultation/{id}"),
            base,
            &altered,
            &completion(consultation, &v, expected),
        )?;
    }
    let mut all_facts = String::new();
    let mut pins = String::new();
    for (i, harm) in contracts::HARMS.iter().enumerate() {
        let v = values(cards, consultation, &format!("MPNoRelabel{i}"));
        all_facts +=
            &fixture(cards, consultation, &v).replace("OtherMaterialNonexistentialEffect", harm);
        pins += &completion(consultation, &v, false);
    }
    add_case(
        context,
        export,
        "consultation/no-existential-relabel",
        "live",
        &all_facts,
        &pins,
    )?;
    Ok(())
}

fn defects(context: &Context, export: &mut Export, cards: &[Card]) -> Result<(), Error> {
    let effect = card(cards, "consented-effect");
    let consent = card(cards, "consent");
    let defect = card(cards, "defect");
    let v = values(cards, effect, "MPWithdrawal");
    let c = dependency_values(cards, effect, consent, &v);
    let mut d = values(cards, defect, "MPDefect");
    for (variable, scope) in common().into_iter().filter(|(_, s)| SHARED.contains(s)) {
        assert!(!scope.is_empty());
        d.insert(variable.into(), c[variable].clone());
    }
    d.insert("$target".into(), c["$record"].clone());
    d.insert("$target_source".into(), c["$source"].clone());
    d.insert("$defect".into(), "ConsentCoercionOrBreach".into());
    let other = values(cards, effect, "MPUnaffected");
    let mut steps = completion(effect, &v, true) + &completion(consent, &c, true);
    steps += &fixture(cards, defect, &d);
    steps += &queries(defect, &d, true);
    steps += &completion(consent, &c, false);
    steps += &completion(effect, &v, false);
    steps += &completion(effect, &other, true);
    add_case(
        context,
        export,
        "defect/downstream-withdrawal",
        "live",
        &(fixture(cards, effect, &v) + &fixture(cards, effect, &other)),
        &steps,
    )?;
    let mismatch = fixture(cards, defect, &d).replace(&d["$version"], "OtherSourceVersion");
    // Do not append mismatched raw target fields: that would introduce a real
    // target ambiguity rather than test an irrelevant independent finding.
    let mismatch = mismatch
        .lines()
        .filter(|line| !line.starts_with(&format!("observe({},", d["$target_source"])))
        .map(|l| format!("{l}\n"))
        .collect::<String>();
    add_case(
        context,
        export,
        "defect/wrong-version",
        "live",
        &(fixture(cards, effect, &v) + &mismatch),
        &(completion(defect, &d, false) + &completion(effect, &v, true)),
    )?;

    let nr = card(cards, "nonresponse");
    let n = values(cards, nr, "MPNonresponse");
    let request = ground(
        "challenge($requester, $reader, $target).\nauthorized($reader, MPChallengeReaderAuthority, $target).\nobserve($requester, $target, $object, MPRequestedObjectScope).\n",
        &n,
    );
    let primary = ground(
        "obliged($reader, ReviewMobilityPluralityRequest, $target)",
        &n,
    );
    let alternate = ground(
        "obliged($alternate, ReviewMobilityPluralityRequestAndSecureRemedy, $target)",
        &n,
    );
    let mut steps = query(&primary, true) + &query(&alternate, false);
    steps += &fixture(cards, nr, &n);
    steps += &query(&alternate, true);
    steps += &query(&ground("false($requester)", &n), false);
    steps += &query(
        &ground(
            "permits($requester, InspectExactPrivatePluralityEvidence, $target)",
            &n,
        ),
        false,
    );
    add_case(
        context,
        export,
        "nonresponse/request-before-permission",
        "live",
        &request,
        &steps,
    )?;
    Ok(())
}

fn boundaries(context: &Context, export: &mut Export, cards: &[Card]) -> Result<(), Error> {
    let mut facts =
        "at(MPNewcomer, EffectiveControl).\nat(MPUnaccompanied, RepublicJurisdiction).\n"
            .to_owned();
    let mut pins = String::new();
    for holder in ["MPNewcomer", "MPUnaccompanied"] {
        pins += &query(&format!("person({holder})"), true);
        pins += &query(&format!("owe(State, Eats, {holder})"), true);
        pins += &query(&format!("travel({holder})"), true);
        pins += &query(&format!("decide({holder}, Ballot)"), false);
        for duty in [
            "ServeFirstNoImmigrationDocumentationOrEnforcementCondition",
            "KeepStandingFloorLibertyLanguageProcessAndRemedyContinuous",
            "IncludeEveryoneUnderJurisdictionOrEffectiveControlInScarcity",
        ] {
            pins += &query(&format!("obliged(State, {duty}, {holder})"), true);
        }
        for barrier in contracts::BARRIERS {
            pins += &query(&format!("prevents({holder}, {barrier})"), true);
        }
    }
    for (index, id) in [
        "membership",
        "internal-law",
        "title",
        "indigenous-government",
        "evacuation",
        "removal",
    ]
    .iter()
    .enumerate()
    {
        let c = card(cards, id);
        let v = values(cards, c, &format!("MPBoundary{index}"));
        facts += &fixture(cards, c, &v);
        pins += &completion(c, &v, true);
        for atom in [
            "person($holder)",
            "decide($holder, Ballot)",
            "prisoner($holder)",
            "false($holder)",
            "reward($holder)",
            "lose(Points, $holder)",
            "free($holder)",
        ] {
            pins += &query(&ground(atom, &v), false);
        }
    }
    for statement in [
        "complete(Forged, PluralityActualConsent, Land)",
        "authority(Forged, RightsBoundedInternalDecision, Record)",
        "permits(Forged, RelyOnConsentForExactCollectiveEffect, Record)",
        "obliged(State, ServeFirstNoImmigrationDocumentationOrEnforcementCondition, Nobody)",
    ] {
        pins += &format!(":refuse reasoning /declared derived-only/\n{statement}.\n");
    }
    add_case(
        context,
        export,
        "boundaries/standing-rights-and-no-coercive-consequence",
        "live",
        &facts,
        &pins,
    )?;

    // All lawful kinds exercise the same exact-source contract. Bad kinds do
    // not get to invent an unreviewed purpose by spelling a fresh constant.
    for (id, variable, vocabulary) in [
        ("identity", "$group_kind", "MPCommunityKind"),
        ("removal", "$transfer", "MPTransferKind"),
        ("remedy", "$relief", "MPReliefKind"),
        ("external", "$instrument", "MPExternalInstrument"),
        ("defect", "$defect", "MPDefectKind"),
    ] {
        let c = card(cards, id);
        let options = vocabularies()
            .into_iter()
            .find(|(name, _)| *name == vocabulary)
            .unwrap()
            .1;
        for (batch, group) in options.chunks(5).enumerate() {
            let mut facts = String::new();
            let mut pins = String::new();
            for (i, option) in group.iter().enumerate() {
                let mut v = values(cards, c, &format!("MPKind{batch}Case{i}"));
                v.insert(variable.into(), (*option).into());
                facts += &fixture(cards, c, &v);
                pins += &queries(c, &v, true);
            }
            add_case(
                context,
                export,
                &format!("{id}/kinds-{batch}"),
                "live",
                &facts,
                &pins,
            )?;
        }
        let mut v = values(cards, c, "MPUnknownKind");
        v.insert(variable.into(), "UnapprovedKind".into());
        add_case(
            context,
            export,
            &format!("{id}/unknown-kind"),
            "live",
            &fixture(cards, c, &v),
            &completion(c, &v, false),
        )?;
    }
    Ok(())
}
