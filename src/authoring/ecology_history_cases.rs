// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real prior court composition, archival finality, and fresh material renewal.

use super::{
    Card, Values,
    cases::{self, Scenario, Stages},
    lifecycle_cases, records,
};

fn key(to: &mut Values, from: &Values) {
    for variable in [
        "$subject",
        "$case",
        "$project",
        "$authorization_record",
        "$authorization_version",
        "$evidence_version",
        "$ground",
        "$version",
        "$place",
        "$territory",
        "$population",
        "$jurisdiction",
        "$scope",
    ] {
        to.insert(variable.into(), from[variable].clone());
    }
}

fn history_values(cards: &[Card], history: &Card, merits: &Card, m: &Values) -> Values {
    let mut h = records::values(cards, history, "ECArchivedFinality");
    key(&mut h, m);
    for (to, from) in [
        ("$merits_record", "$record"),
        ("$merits_revision", "$revision"),
        ("$merits_source", "$source"),
        ("$merits_review", "$review"),
        ("$merits_operator", "$operator"),
        ("$merits_version", "$version"),
        ("$merits_window", "$window"),
        ("$merits_start", "$start"),
        ("$merits_end", "$end"),
        ("$merits_evaluation", "$evaluation"),
        ("$outcome", "$outcome"),
        ("$court_record", "$external_court_record"),
        ("$court_source", "$external_court_source"),
        ("$court_evidence", "$external_court_evidence"),
        ("$court_review", "$external_court_review"),
    ] {
        h.insert(to.into(), m[from].clone());
    }
    h.insert("$merits_kind".into(), merits.kind.into());
    h
}

pub(super) fn boundaries(cards: &[Card]) -> Vec<Scenario> {
    let history = super::card(cards, "guardian-final-history");
    let mut result = Vec::new();
    for merits_id in ["guardian-merits", "substitute-guardian-merits"] {
        let merits = super::card(cards, merits_id);
        for lifecycle_id in ["ecological-record-end", "ecological-record-correction"] {
            let mut m = records::values(cards, merits, "ECHistoricalCourt");
            m.insert("$outcome".into(), "ECMeritsRestrictChallengedAct".into());
            let h = history_values(cards, history, merits, &m);
            let finding = super::card(cards, lifecycle_id);
            let f =
                lifecycle_cases::affected_values(cards, finding, merits, &m, "ECEndedCurrentCourt");
            let mut stages = Stages::default();
            let activate_history = stages.add(cards, history, &h);
            let activate_end = stages.add(cards, finding, &f);
            let restriction = records::ground(
                "oppose($authorization_record, $authorization_version, ECFinalJudicialProtection)",
                &m,
            );
            let mut pins = cases::queries(merits, &m, true)
                + &super::query(&restriction, true)
                + &cases::conclusion(history, &h, false)
                + &activate_history
                + &cases::queries(history, &h, true)
                + &activate_end
                + &cases::queries(finding, &f, true)
                + &cases::conclusion(merits, &m, false)
                + &cases::queries(history, &h, true)
                + &super::query(&restriction, false);
            for id in ["guardian-stay", "alternate-guardian-stay"] {
                let stay = super::card(cards, id);
                let mut s =
                    records::values(cards, stay, &format!("ECPastReplay{}", id.replace('-', "")));
                key(&mut s, &m);
                pins += &stages.add(cards, stay, &s);
                pins += &cases::conclusion(stay, &s, false);
                pins += &super::query(
                    &records::ground("end($record, ECFinalGuardianStayReplay)", &s),
                    true,
                );
            }
            for (index, id) in [
                "guardian-authorization-after-history",
                "guardian-evidence-after-history",
                "alternate-authorization-after-history",
                "alternate-evidence-after-history",
            ]
            .iter()
            .enumerate()
            {
                let renewal = super::card(cards, id);
                let mut next =
                    records::values(cards, renewal, &format!("ECFreshHistoricalRenewal{index}"));
                key(&mut next, &m);
                cases::join_parent(cards, renewal, "guardian-final-history", &mut next, &h);
                // Each independent renewal in this case has its own truly new
                // authorization/evidence identity, not a fork of one window.
                if id.contains("authorization") {
                    next.insert(
                        "$authorization_record".into(),
                        format!("ECMateriallyNewAuthorization{index}"),
                    );
                    next.insert(
                        "$authorization_version".into(),
                        format!("ECMateriallyNewAuthorizationVersion{index}"),
                    );
                } else {
                    next.insert(
                        "$evidence_version".into(),
                        format!("ECMateriallyNewEvidenceVersion{index}"),
                    );
                }
                pins += &stages.add(cards, renewal, &next);
                pins += &cases::queries(renewal, &next, true);
                pins += &cases::conclusion(merits, &m, false);
            }
            result.push(Scenario {
                id: format!(
                    "guardian/history/{merits_id}-{lifecycle_id}-does-not-reopen-resolved-key"
                ),
                facts: records::fixture(cards, merits, &m) + &stages.finish(),
                pins,
            });
        }
    }
    let merits = super::card(cards, "guardian-merits");
    let m = records::values(cards, merits, "ECArchiveEvidenceCourt");
    let h = history_values(cards, history, merits, &m);
    let base = records::fixture(cards, history, &h);
    // This current-authority query is deliberately FALSE: the historical
    // certificate authenticates a past proof, not a current court mandate.
    result.push(Scenario {
        id: "guardian/history/historical-proof-does-not-grant-current-court-authority".into(),
        facts: base.clone(),
        pins: cases::queries(history, &h, true) + &cases::conclusion(merits, &m, false),
    });
    for (index, atom) in records::base_premises(cards, history)
        .into_iter()
        .filter(|atom| atom.starts_with("authorized(") || atom.starts_with("observe("))
        .enumerate()
    {
        let fact = format!("{}.\n", records::ground(&atom, &h));
        assert_eq!(base.matches(&fact).count(), 1);
        result.push(Scenario {
            id: format!("guardian/history/missing-original-or-archive-witness-{index}"),
            facts: base.replacen(&fact, "", 1),
            pins: cases::conclusion(history, &h, false),
        });
    }
    result
}
