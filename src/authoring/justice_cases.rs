// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

fn completion(card: &Card, v: &Values, expected: bool) -> String {
    query(&ground(&heads(card)[0], v), expected)
}

fn without(facts: &str, record: &str, scope: &str) -> String {
    facts
        .lines()
        .filter(|l| {
            !(l.contains(&format!(", {record}, ")) && l.ends_with(&format!(", J{scope}Scope).")))
        })
        .map(|l| format!("{l}\n"))
        .collect()
}

fn counterfactual(export: &mut Export, authored: &[String], name: &str, atom: &str) {
    let edits = authored
        .iter()
        .filter(|r| r.contains(atom))
        .map(|r| Edit {
            before: r.clone(),
            after: r.replace(&format!(" & {atom}"), ""),
        })
        .collect::<Vec<_>>();
    assert!(!edits.is_empty() && edits.iter().all(|e| e.before != e.after));
    export.bases.insert(
        name.into(),
        Base {
            base: Some("live".into()),
            edits,
            path: None,
        },
    );
}

pub(super) fn generate(
    context: &Context,
    export: &mut Export,
    cards: &[Card],
    authored: &[String],
) -> Result<(), Error> {
    for (name, atom) in [
        ("justice-no-independent-review", INDEPENDENT),
        (
            "justice-no-restorative-choice",
            "observe($subject, $record, ActualInformedConsent, JParticipantChoiceScope)",
        ),
        (
            "justice-no-case-relief",
            "complete($case_relief_record, JusticeCaseSpecificRelief, $case)",
        ),
    ] {
        counterfactual(export, authored, name, atom);
    }

    for c in cards {
        let v = values(cards, c, "JCase");
        let facts = fixture(context, cards, c, &v)?;
        add_case(
            context,
            export,
            &format!("{}/positive", c.id),
            "live",
            &facts,
            &queries(c, &v, true),
        )?;
        add_case(
            context,
            export,
            &format!("{}/withheld", c.id),
            "live",
            "",
            &queries(c, &v, false),
        )?;
        for (batch, group) in fields(c).chunks(6).enumerate() {
            let mut all = String::new();
            let mut pins = String::new();
            for (i, (_, scope)) in group.iter().enumerate() {
                let v = values(cards, c, &format!("JOmit{batch}Row{i}"));
                all += &without(&fixture(context, cards, c, &v)?, &v["$record"], scope);
                pins += &format!("# Withhold {scope}, preserving every other premise.\n");
                pins += &completion(c, &v, false);
            }
            add_case(
                context,
                export,
                &format!("{}/required-fields-{batch}", c.id),
                "live",
                &all,
                &pins,
            )?;
        }
        let mut all = String::new();
        let mut pins = String::new();
        for (i, (actor, role)) in ATTESTERS.iter().chain(READERS).enumerate() {
            let v = values(cards, c, &format!("JNoAuthority{i}"));
            all += &fixture(context, cards, c, &v)?.replace(
                &ground(&format!("authorized({actor}, {role}, $record).\n"), &v),
                "",
            );
            pins += &completion(c, &v, false);
        }
        add_case(
            context,
            export,
            &format!("{}/writer-authority", c.id),
            "live",
            &all,
            &pins,
        )?;
        let mut aliases = vec![
            ("$review", "$source"),
            ("$review", "$operator"),
            ("$reader", "$review"),
            ("$alternate", "$reader"),
            ("$auditor", "$operator"),
            ("$review", "$subject"),
            ("$review", "$other_party"),
            ("$reader", "$subject"),
            ("$alternate", "$other_party"),
        ];
        if v.contains_key("$prosecutor") {
            aliases.push(("$prosecutor", "$investigator"));
        }
        if v.contains_key("$adjudicator") {
            aliases.push(("$review", "$adjudicator"));
        }
        if v.contains_key("$requester") {
            aliases.push(("$review", "$requester"));
        }
        if v.contains_key("$target_operator") {
            aliases.push(("$review", "$target_operator"));
        }
        if c.court.is_some() {
            aliases.push(("$review", "$court_source"));
            aliases.push(("$alternate", "$court_executor"));
        }
        let mut all = String::new();
        let mut pins = String::new();
        for (i, (left, right)) in aliases.iter().enumerate() {
            let mut v = values(cards, c, &format!("JFused{i}"));
            v.insert((*left).into(), v[*right].clone());
            all += &fixture(context, cards, c, &v)?;
            pins += &completion(c, &v, false);
        }
        add_case(
            context,
            export,
            &format!("{}/independence", c.id),
            "live",
            &all,
            &pins,
        )?;
        let mut fused = v.clone();
        fused.insert("$review".into(), v["$source"].clone());
        add_case(
            context,
            export,
            &format!("{}/counterfactual-self-review", c.id),
            "justice-no-independent-review",
            &fixture(context, cards, c, &fused)?,
            &completion(c, &fused, true),
        )?;
        let conflict = format!(
            "{}.\n",
            ground(
                &observe("$review", "$record", "WrongVersion", "SourceVersion"),
                &v
            )
        );
        add_case(
            context,
            export,
            &format!("{}/conflict-sequence", c.id),
            "live",
            &facts,
            &(completion(c, &v, true) + &conflict + &completion(c, &v, false)),
        )?;
        let noise = format!(
            "{}.\n",
            ground(
                &observe(
                    "UncredentialedWriter",
                    "$record",
                    "WrongVersion",
                    "SourceVersion"
                ),
                &v
            )
        );
        add_case(
            context,
            export,
            &format!("{}/unauthorized-noise", c.id),
            "live",
            &(facts.clone() + &noise),
            &completion(c, &v, true),
        )?;
        add_case(
            context,
            export,
            &format!("{}/stale", c.id),
            "live",
            &facts.replace("CurrentReconciledFinding", "SupersededFinding"),
            &completion(c, &v, false),
        )?;
        for id in &c.dependencies {
            add_case(
                context,
                export,
                &format!("{}/without-{id}", c.id),
                "live",
                &without(&facts, &v[&dep_var(id, "record")], "RecordKind"),
                &completion(c, &v, false),
            )?;
        }
        if c.court.is_some() {
            let missing = facts
                .lines()
                .filter(|l| {
                    !l.starts_with(&format!(
                        "authorized({}, StateFormSourceAuthority,",
                        v["$court_source"]
                    ))
                })
                .map(|l| format!("{l}\n"))
                .collect::<String>();
            add_case(
                context,
                export,
                &format!("{}/without-actual-court-authority", c.id),
                "live",
                &missing,
                &completion(c, &v, false),
            )?;
            let mut sequence = completion(c, &v, true);
            sequence += &format!(
                "observe({}, {}, WrongCourtVersion, SourceVersionScope).\n",
                v["$court_record_review"], v["$court_record"]
            );
            sequence += &completion(c, &v, false);
            add_case(
                context,
                export,
                &format!("{}/court-conflict-sequence", c.id),
                "live",
                &facts,
                &sequence,
            )?;
        }
    }
    domains_and_restoration(context, export, cards)?;
    remedies_and_boundaries(context, export, cards, authored)
}

fn domains_and_restoration(
    context: &Context,
    export: &mut Export,
    cards: &[Card],
) -> Result<(), Error> {
    for id in ["access", "assistance", "hearing", "survivor"] {
        let c = card(cards, id);
        for domain in contracts::DOMAINS {
            let mut v = values(cards, c, "JDomain");
            v.insert("$domain".into(), (*domain).into());
            add_case(
                context,
                export,
                &format!("{id}/{domain}"),
                "live",
                &fixture(context, cards, c, &v)?,
                &queries(c, &v, true),
            )?;
        }
        let mut v = values(cards, c, "JUnknownDomain");
        v.insert("$domain".into(), "SecretCourtJustice".into());
        add_case(
            context,
            export,
            &format!("{id}/no-undeclared-domain"),
            "live",
            &fixture(context, cards, c, &v)?,
            &completion(c, &v, false),
        )?;
    }
    let c = card(cards, "prosecution");
    let mut v = values(cards, c, "JNoCharge");
    v.insert("$domain".into(), "CriminalJustice".into());
    v.insert("$outcome".into(), "ReasonedNonChargeDecision".into());
    add_case(
        context,
        export,
        "prosecution/reasoned-noncharge",
        "live",
        &fixture(context, cards, c, &v)?,
        &queries(c, &v, true),
    )?;
    let c = card(cards, "restoration");
    let v = values(cards, c, "JRestoration");
    let facts = fixture(context, cards, c, &v)?;
    for (party, label) in [("$subject", "claimant"), ("$other_party", "respondent")] {
        let missing = facts.replace(
            &ground(
                &format!(
                    "observe({party}, $record, ActualInformedConsent, JParticipantChoiceScope).\n"
                ),
                &v,
            ),
            "",
        );
        add_case(
            context,
            export,
            &format!("restoration/{label}-no-consent"),
            "live",
            &missing,
            &completion(c, &v, false),
        )?;
        if party == "$subject" {
            add_case(
                context,
                export,
                "restoration/counterfactual-no-consent",
                "justice-no-restorative-choice",
                &missing,
                &completion(c, &v, true),
            )?;
        }
        for choice in [
            "Withdrawal",
            "Refusal",
            "CoercedConsent",
            "ConflictingConsent",
        ] {
            let step = ground(
                &format!("observe({party}, $record, {choice}, JParticipantChoiceScope).\n"),
                &v,
            );
            let pins = completion(c, &v, true)
                + &step
                + &completion(c, &v, false)
                + &query(&format!("prisoner({})", v[party]), false)
                + &query(
                    &format!("forgive({}, {})", v["$subject"], v["$other_party"]),
                    false,
                );
            add_case(
                context,
                export,
                &format!("restoration/{label}-{choice}"),
                "live",
                &facts,
                &pins,
            )?;
        }
    }
    let outsider = ground(
        "observe(UnrelatedPerson, $record, Withdrawal, JParticipantChoiceScope).\n",
        &v,
    );
    add_case(
        context,
        export,
        "restoration/outsider-cannot-withdraw-participant-choice",
        "live",
        &(facts + &outsider),
        &completion(c, &v, true),
    )?;
    Ok(())
}

fn remedies_and_boundaries(
    context: &Context,
    export: &mut Export,
    cards: &[Card],
    authored: &[String],
) -> Result<(), Error> {
    let c = card(cards, "enforcement");
    let v = values(cards, c, "JEnforce");
    let facts = fixture(context, cards, c, &v)?;
    let target = &v["$case_relief_record"];
    add_case(
        context,
        export,
        "enforcement/counterfactual-no-relief",
        "justice-no-case-relief",
        &without(&facts, target, "RecordKind"),
        &completion(c, &v, true),
    )?;
    let dep = card(cards, "case-relief");
    let original = dep_values(cards, c, dep, &v);
    let old_facts = fixture(context, cards, dep, &original)?;
    let mut wrong = original.clone();
    wrong.insert("$case".into(), "DifferentCase".into());
    let mismatched = facts
        .strip_suffix(&old_facts)
        .expect("dependency appended last")
        .to_string()
        + &fixture(context, cards, dep, &wrong)?;
    add_case(
        context,
        export,
        "enforcement/wrong-case",
        "live",
        &mismatched,
        &(completion(dep, &wrong, true) + &completion(c, &v, false)),
    )?;
    let mut wrong_executor = original.clone();
    wrong_executor.insert("$executor".into(), "DifferentLawfulExecutor".into());
    let other_executor_facts = facts.strip_suffix(&old_facts).unwrap().to_string()
        + &fixture(context, cards, dep, &wrong_executor)?;
    add_case(
        context,
        export,
        "enforcement/wrong-executor",
        "live",
        &other_executor_facts,
        &(completion(dep, &wrong_executor, true) + &completion(c, &v, false)),
    )?;
    let completion_atom = "complete($case_relief_record, JusticeCaseSpecificRelief, $case)";
    let raw_join = "observe($case_relief_source, $case_relief_record, $case, JCaseScope)";
    let edits = authored
        .iter()
        .filter(|r| r.contains(raw_join))
        .map(|r| Edit {
            before: r.clone(),
            after: r.replace(&format!(" & {raw_join}"), "").replace(
                completion_atom,
                "complete($case_relief_record, JusticeCaseSpecificRelief, DifferentCase)",
            ),
        })
        .collect();
    export.bases.insert(
        "justice-borrow-other-case".into(),
        Base {
            base: Some("live".into()),
            edits,
            path: None,
        },
    );
    add_case(
        context,
        export,
        "enforcement/counterfactual-borrow-other-case",
        "justice-borrow-other-case",
        &mismatched,
        &completion(c, &v, true),
    )?;
    for relief in contracts::RELIEFS {
        let mut v = values(cards, c, "JReliefKind");
        v.insert("$relief".into(), (*relief).into());
        add_case(
            context,
            export,
            &format!("enforcement/{relief}"),
            "live",
            &fixture(context, cards, c, &v)?,
            &queries(c, &v, true),
        )?;
    }
    let mut invalid = v.clone();
    invalid.insert("$relief".into(), "Imprisonment".into());
    add_case(
        context,
        export,
        "enforcement/no-custodial-remedy-kind",
        "live",
        &fixture(context, cards, c, &invalid)?,
        &completion(c, &invalid, false),
    )?;
    let pins = completion(c, &v, true)
        + &query(
            &format!(
                "authority(FSBOD_18, FSPOW_023, {})",
                v["$case_relief_record"]
            ),
            false,
        )
        + &query(
            &format!(
                "complete({}, JusticeConstitutionalInvalidation, {})",
                v["$record"], v["$case"]
            ),
            false,
        );
    add_case(
        context,
        export,
        "enforcement/no-general-invalidation",
        "live",
        &facts,
        &pins,
    )?;
    let d = card(cards, "defect");
    let mut dv = values(cards, d, "JDefect");
    for (value, scope) in common().into_iter().filter(|(_, s)| JOINS.contains(s)) {
        if value.starts_with('$') {
            let upstream = fields(dep)
                .into_iter()
                .find(|(_, s)| *s == scope)
                .unwrap()
                .0;
            dv.insert(value.into(), ground(upstream, &original));
        }
    }
    dv.insert("$target".into(), target.clone());
    dv.insert("$target_source".into(), original["$source"].clone());
    dv.insert("$target_operator".into(), original["$operator"].clone());
    for (variable, label) in [("$adjudicator", "adjudicator"), ("$executor", "executor")] {
        let mut interested = dv.clone();
        interested.insert("$review".into(), original[variable].clone());
        add_case(
            context,
            export,
            &format!("defect/no-{label}-self-review"),
            "live",
            &(facts.clone() + &fixture(context, cards, d, &interested)?),
            &(completion(d, &interested, false) + &completion(c, &v, true)),
        )?;
    }
    let mut sequence = completion(c, &v, true)
        + &fixture(context, cards, d, &dv)?
        + &queries(d, &dv, true)
        + &completion(c, &v, false);
    let fresh = values(cards, c, "JFresh");
    sequence += &(fixture(context, cards, c, &fresh)? + &queries(c, &fresh, true));
    add_case(
        context,
        export,
        "defect/downstream-withdrawal-and-fresh-correction",
        "live",
        &facts,
        &sequence,
    )?;
    let mut wrong = dv.clone();
    wrong.insert("$version".into(), "WrongDefectVersion".into());
    let wrong_facts = fixture(context, cards, d, &wrong)?;
    let wrong_facts = wrong_facts
        .lines()
        .filter(|l| !l.starts_with(&format!("observe({}, {},", dv["$target_source"], target)))
        .map(|l| format!("{l}\n"))
        .collect::<String>();
    add_case(
        context,
        export,
        "defect/wrong-version-cannot-withdraw-real-relief",
        "live",
        &(facts + &wrong_facts),
        &(completion(d, &wrong, false) + &completion(c, &v, true)),
    )?;
    let mut fixture_text = "person(JUnrecordedPerson).\n".to_string();
    let mut steps = String::new();
    for duty in [
        "PreserveBodilyIntegrityHumaneConditionsAndConfidentialCommunication",
        "EnsureIndependentCustodialComplaintInspectionAndConfidentialCounsel",
        "ArrangeIndependentReleaseReviewAndDoNotDelayLawfulRelease",
        "PreserveCareHousingVoiceAndReintegrationAcrossRelease",
    ] {
        steps += &query(&format!("obliged(State, {duty}, Hano)"), true);
    }
    for duty in contracts::DUTIES {
        steps += &query(&format!("obliged(State, {duty}, JUnrecordedPerson)"), true);
    }
    for barrier in contracts::BARRIERS {
        steps += &query(&format!("prevents(JUnrecordedPerson, {barrier})"), true);
    }
    for c in cards {
        let mut v = values(cards, c, &format!("JBoundary{}", c.id.replace('-', "")));
        v.insert("$subject".into(), "JUnrecordedPerson".into());
        fixture_text += &fixture(context, cards, c, &v)?;
        steps += &completion(c, &v, true);
    }
    for head in [
        "prisoner(JUnrecordedPerson)",
        "free(JUnrecordedPerson)",
        "authority(JUnrecordedPerson)",
        "reward(JUnrecordedPerson, Trust)",
        "decide(JUnrecordedPerson, Ballot)",
        "false(JUnrecordedPerson)",
    ] {
        steps += &query(head, false);
    }
    for floor in [
        "eats",
        "dwell",
        "healthy",
        "secure",
        "learn",
        "believe",
        "expresses",
        "meets",
    ] {
        steps += &query(&format!("{floor}(JUnrecordedPerson)"), false);
    }
    steps += "\n:refuse reasoning /declared derived-only/\ncomplete(ForgedRecord, JusticeCaseSpecificRelief, ForgedCase).\n\n:refuse reasoning /declared derived-only/\npermits(ForgedOperator, ImplementSpecifiedNoncoerciveRemedy, ForgedRecord).\n";
    add_case(
        context,
        export,
        "boundaries/no-person-consequences-or-forged-orders",
        "live",
        &fixture_text,
        &steps,
    )?;
    let request = "challenge(AnyRequester, IndependentJusticeReader, BareJusticeRequest).\nauthorized(IndependentJusticeReader, JChallengeReaderAuthority, BareJusticeRequest).\n";
    add_case(
        context,
        export,
        "appeal/request-without-operator-or-previous-permission",
        "live",
        request,
        &query(
            "obliged(IndependentJusticeReader, ReviewJusticeRequest, BareJusticeRequest)",
            true,
        ),
    )?;
    let c = card(cards, "nonresponse");
    let v = values(cards, c, "JSilence");
    let facts = fixture(context, cards, c, &v)?;
    add_case(
        context,
        export,
        "nonresponse/silence-without-positive-elapsed-evidence",
        "live",
        &without(&facts, &v["$record"], "Nonresponse"),
        &queries(c, &v, false),
    )?;
    Ok(())
}
