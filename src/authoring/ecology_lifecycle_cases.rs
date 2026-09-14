// SPDX-License-Identifier: MIT OR Apache-2.0

//! Actual source-scoped withdrawal, independent readers and care continuity.

use super::{
    Card, Values,
    cases::{self, Scenario, Stages},
    lifecycle, records,
};

pub(super) fn affected_values(
    cards: &[Card],
    finding: &Card,
    target: &Card,
    values: &Values,
    prefix: &str,
) -> Values {
    let mut result = records::values(cards, finding, prefix);
    for variable in [
        "$subject",
        "$case",
        "$place",
        "$territory",
        "$population",
        "$jurisdiction",
        "$scope",
    ] {
        result.insert(variable.into(), values[variable].clone());
    }
    for (to, from) in [
        ("$target", "$record"),
        ("$target_revision", "$revision"),
        ("$target_version", "$version"),
        ("$target_source", "$source"),
        ("$target_review", "$review"),
        ("$target_operator", "$operator"),
        ("$target_window", "$window"),
        ("$target_end", "$end"),
        ("$target_evidence", "$evidence_record"),
        ("$target_evidence_version", "$evidence_version"),
    ] {
        result.insert(to.into(), values[from].clone());
    }
    result.insert("$target_kind".into(), target.kind.into());
    result
}

fn withdrawn(values: &Values, expected: bool) -> String {
    super::query(
        &records::ground("contradict($record, $revision, ECRecordReliance)", values),
        expected,
    )
}

fn urgent() -> (String, String) {
    let mut facts = String::new();
    let mut pins = String::new();
    for (kind, duty) in [
        (
            "ECUrgentPlausibleAnimalProtectionRequest",
            "ECProvidePromptSafeAnimalProtectionAndNecessaryCareBeforeRecordReconciliation",
        ),
        (
            "ECUrgentEnvironmentalProtectionRequest",
            "ECProvidePromptNoFaultEnvironmentalProtectionAndHumanAnimalContinuityBeforeRecordReconciliation",
        ),
    ] {
        let request = format!("{kind}Example");
        facts += &format!(
            "challenge(ECUrgentRequester, ECUrgentUnidentifiedSubject, {request}).\nobserve(ECUrgentRequester, {request}, {kind}, ECRequestKindScope).\n"
        );
        pins += &super::query(&format!("obliged(State, {duty}, {request})"), true);
        for head in [
            format!("permits(ECUrgentRequester, ECSearchOrSeizure, {request})"),
            format!("capture(ECUrgentUnidentifiedSubject)"),
            format!("prisoner(ECUrgentUnidentifiedSubject)"),
            format!("person(ECUrgentUnidentifiedSubject)"),
        ] {
            pins += &super::query(&head, false);
        }
    }
    (facts, pins)
}

pub(super) fn boundaries(cards: &[Card]) -> Vec<Scenario> {
    let target = super::card(cards, "animal-ordinary-use");
    let values = records::values(cards, target, "ECLifecycleTarget");
    let other = records::values(cards, target, "ECLifecycleUnaffected");
    let base = records::fixture(cards, target, &values);
    let (urgent_facts, urgent_pins) = urgent();
    let mut result = vec![Scenario {
        id: "record/urgent-care-with-no-completed-record-owner-or-taxonomy".into(),
        facts: urgent_facts.clone(),
        pins: urgent_pins.clone() + &cases::conclusion(target, &values, false),
    }];
    for id in [
        "ecological-record-defect",
        "ecological-record-end",
        "ecological-record-correction",
    ] {
        let finding = super::card(cards, id);
        let finding_values = affected_values(cards, finding, target, &values, "ECLifecycleFinding");
        let mut stages = Stages::default();
        let activate = stages.add(cards, finding, &finding_values);
        let mut pins = cases::conclusion(target, &values, true)
            + &cases::conclusion(finding, &finding_values, false)
            + &withdrawn(&values, false)
            + &activate
            + &cases::queries(finding, &finding_values, true)
            + &withdrawn(&values, true)
            + &cases::conclusion(target, &values, false)
            + &cases::queries(target, &other, true)
            + &urgent_pins;
        if id == "ecological-record-end" {
            pins += &super::query(
                &records::ground(
                    "err($record, ECReviewedEcologicalAnimalRecordDefect)",
                    &finding_values,
                ),
                false,
            );
        }
        result.push(Scenario {
            id: format!("record/{id}/exact-stateful-withdrawal-and-unaffected-care"),
            facts: base.clone()
                + &records::fixture(cards, target, &other)
                + &urgent_facts
                + &stages.finish(),
            pins,
        });
        result.push(Scenario {
            id: format!("record/{id}/finding-and-remedy-without-original-record"),
            facts: records::fixture(cards, finding, &finding_values),
            pins: cases::queries(finding, &finding_values, true)
                + &withdrawn(&values, false)
                + &cases::conclusion(target, &values, false),
        });
    }
    let finding = super::card(cards, "ecological-record-defect");
    let finding_values = affected_values(cards, finding, target, &values, "ECLifecycleDefect");
    for defect in lifecycle::DEFECTS {
        let mut changed = finding_values.clone();
        changed.insert("$defect".into(), (*defect).into());
        result.push(Scenario {
            id: format!("record/defect/{defect}"),
            facts: base.clone() + &records::fixture(cards, finding, &changed),
            pins: cases::queries(finding, &changed, true)
                + &withdrawn(&values, true)
                + &cases::conclusion(target, &values, false),
        });
    }
    let variables: std::collections::BTreeSet<_> = lifecycle::target_join()
        .iter()
        .flat_map(|atom| {
            records::pattern()
                .find_iter(atom)
                .map(|m| m.as_str().to_owned())
        })
        .collect();
    for variable in variables {
        let mut changed = finding_values.clone();
        changed.insert(variable.clone(), "ECWrongAffectedIdentity".into());
        result.push(Scenario {
            id: format!(
                "record/withdrawal-wrong-{}",
                variable.trim_start_matches('$')
            ),
            facts: base.clone() + &records::fixture(cards, finding, &changed),
            pins: cases::queries(finding, &changed, true)
                + &withdrawn(&values, false)
                + &cases::queries(target, &values, true),
        });
    }
    let correction = super::card(cards, "ecological-record-correction");
    let fresh = records::values(cards, target, "ECFreshCorrectedAuthority");
    let mut corrected =
        affected_values(cards, correction, target, &values, "ECCorrectionNoRenewal");
    corrected.insert("$replacement".into(), fresh["$record"].clone());
    corrected.insert("$replacement_revision".into(), fresh["$revision"].clone());
    let mut stages = Stages::default();
    let activate_fresh = stages.add(cards, target, &fresh);
    result.push(Scenario {
        id: "record/correction-is-not-a-replacement-permit".into(),
        facts: base + &records::fixture(cards, correction, &corrected) + &stages.finish(),
        pins: cases::queries(correction, &corrected, true)
            + &withdrawn(&values, true)
            + &cases::conclusion(target, &values, false)
            + &cases::conclusion(target, &fresh, false)
            + &activate_fresh
            + &cases::queries(target, &fresh, true)
            + &cases::conclusion(target, &values, false),
    });
    let nonresponse = super::card(cards, "ecological-record-nonresponse");
    let n = records::values(cards, nonresponse, "ECUnansweredRecord");
    let mut stages = Stages::default();
    let activate = stages.add(cards, nonresponse, &n);
    result.push(Scenario {
        id: "record/nonresponse-requires-independent-deadline-finding".into(),
        facts: stages.finish(),
        pins: cases::queries(nonresponse, &n, false)
            + &activate
            + &cases::queries(nonresponse, &n, true)
            + &super::query(
                &records::ground("interrupt($operator, $record, ECGuardianAutomaticStay)", &n),
                false,
            )
            + &super::query(
                &records::ground(
                    "permits($operator, ECExactHighConsequenceActivity, $record)",
                    &n,
                ),
                false,
            ),
    });
    let ended = super::card(cards, "ecological-record-end");
    let e = affected_values(cards, ended, target, &values, "ECRecordedPastEnd");
    let mut renamed = records::values(cards, target, "ECRenamedExpiredAuthority");
    for variable in ["$window", "$start", "$end"] {
        renamed.insert(variable.into(), values[variable].clone());
    }
    let fresh = records::values(cards, target, "ECGenuinelyFreshWindow");
    let mut stages = Stages::default();
    let activate = stages.add(cards, ended, &e);
    result.push(Scenario {
        id: "record/ended-window-cannot-be-carried-into-renamed-authority".into(),
        facts: records::fixture(cards, target, &values)
            + &records::fixture(cards, target, &renamed)
            + &records::fixture(cards, target, &fresh)
            + &stages.finish(),
        pins: cases::queries(target, &values, true)
            + &cases::queries(target, &renamed, true)
            + &activate
            + &cases::queries(ended, &e, true)
            + &cases::conclusion(target, &values, false)
            + &cases::conclusion(target, &renamed, false)
            + &cases::queries(target, &fresh, true),
    });
    result
}
