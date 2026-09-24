// SPDX-License-Identifier: MIT OR Apache-2.0

//! Supplied comparative decisions: the model qualifies the record; it does
//! not measure supply, compare human worth, conduct a lottery or deliver care.

use super::{CARRIED, Source, add_case, bindings, dependency, fixture, queries, query};
use crate::{
    authoring::{Case, Export},
    cli::Error,
    context::Context,
};

pub(super) fn generate(
    context: &Context,
    export: &mut Export,
    source: &Source,
) -> Result<(), Error> {
    // This is the complete original allocation fixture, retained as the
    // regression that revealed the missing comparative-reasons requirement.
    // Register it without rewriting it from the repaired contract.
    export.cases.push(Case {
        id: "scarcity/worked/no-comparative-reasons".into(),
        base: "live".into(),
        fixtures: vec!["tests/pins/scarcity/worked/no-comparative-reasons/fixture.nibli".into()],
        pins: vec!["tests/pins/scarcity/worked/no-comparative-reasons/expect.pins.nibli".into()],
        edits: vec![],
        scan: true,
    });
    let get = |id: &str| {
        source
            .contracts
            .iter()
            .find(|contract| contract.id == id)
            .expect("declared scarcity contract")
    };
    let allocation = get("allocation");
    let comparisons = &source
        .vocabularies
        .iter()
        .find(|(scope, _, _)| scope == "ScarcityComparisonOutcomeScope")
        .unwrap()
        .2;
    let methods = &source
        .vocabularies
        .iter()
        .find(|(scope, _, _)| scope == "ScarcityAllocationMethodScope")
        .unwrap()
        .2;

    // Every incompatible pair is a substantive refusal. A compatible pair is
    // still only one prerequisite among all finding and allocation conditions.
    for comparison in comparisons {
        for method in methods {
            let mut values = bindings(source, allocation, "ScarcityRoute");
            values.insert("$comparison_outcome".into(), comparison.clone());
            values.insert("$allocation_method".into(), method.clone());
            let facts =
                dependency(source, allocation, &values) + &fixture(source, allocation, &values);
            let expected = source
                .allocation_routes
                .contains(&[comparison.clone(), method.clone()]);
            add_case(
                context,
                export,
                &format!("worked/route-{comparison}-{method}"),
                "live",
                &facts,
                &queries(allocation, &values, expected),
            )?;
            if comparison == "NoUsableEqualShareAndMateriallyUnequalClaims"
                && method == "DisclosedLottery"
            {
                add_case(
                    context,
                    export,
                    "worked/counterfactual-lottery-among-unequal-claims",
                    "scarcity-no-allocation-route",
                    &facts,
                    &queries(allocation, &values, true),
                )?;
            }
        }
    }

    let mut values = bindings(source, allocation, "ScarcityWorked");
    for (variable, value) in [
        ("$resource", "IndivisibleEssentialPowerUnit"),
        ("$population", "WaitingAndContinuityClaimants"),
        ("$period", "ThisAllocationWindow"),
        ("$claims", "WaitingAndContinuityClaimsAfterAccommodation"),
        ("$mitigation_key", "ImminentIrreversibleHarm"),
        (
            "$allocation",
            "ContinueUnitForContinuityClaimDuringThisWindow",
        ),
        (
            "$reasons",
            "EarlierNeedAndLargerResourceSpecificBenefitFavourWaitingButImminentIrreversibleInterruptionAndContinuityJustifyContinuingThisWindow",
        ),
    ] {
        values.insert(variable.into(), value.into());
    }
    let people = concat!(
        "person(ScarcityWaitingClaimant).\n",
        "person(ScarcityContinuityClaimant).\n",
        "at(ScarcityWaitingClaimant, GeneralAdulthood).\n",
        "at(ScarcityContinuityClaimant, GeneralAdulthood).\n",
        "list(WaitingAndContinuityClaimsAfterAccommodation, WaitingClaim, ScarcityWaitingClaimant, ScarcityComparedClaim).\n",
        "list(WaitingAndContinuityClaimsAfterAccommodation, ContinuityClaim, ScarcityContinuityClaimant, ScarcityComparedClaim).\n",
    );
    let shortfall = get("shortfall");
    let mut unmet = bindings(source, shortfall, "ScarcityWaitingShortfall");
    for name in CARRIED {
        unmet.insert(name.into(), values[name].clone());
    }
    for name in ["$authorization", "$authorization_source"] {
        unmet.insert(name.into(), values[name].clone());
    }
    unmet.insert("$claim".into(), "WaitingClaim".into());
    unmet.insert(
        "$unmet".into(),
        "WaitingClaimEssentialProvisionDuringThisWindow".into(),
    );
    let facts = dependency(source, allocation, &values)
        + &fixture(source, allocation, &values)
        + &fixture(source, shortfall, &unmet)
        + people;
    let mut retained = String::new();
    for person in ["ScarcityWaitingClaimant", "ScarcityContinuityClaimant"] {
        for (atom, expected) in [
            (format!("person({person})"), true),
            (format!("owe(State, Healthy, {person})"), true),
            (format!("decide({person}, Ballot)"), true),
            (format!("healthy({person})"), false),
            (format!("false({person})"), false),
            (format!("prisoner({person})"), false),
        ] {
            retained.push_str(&query(&atom, expected));
        }
    }
    let mut ordinary =
        queries(allocation, &values, true) + &queries(shortfall, &unmet, true) + &retained;
    ordinary.push_str(&query(
        "observe(ScarcityWorkedSource, ScarcityWorkedRecord, ContinueUnitForContinuityClaimDuringThisWindow, ScarcitySpecifiedAllocationScope)",
        true,
    ));
    add_case(
        context,
        export,
        "worked/competing-claims",
        "live",
        &facts,
        &ordinary,
    )?;

    // Neither merely preferring another result nor asking for review is itself
    // a defect. A qualified finding about the comparison can defeat this exact
    // allocation while the genuine shortage and the shortfall duties remain.
    let reader = &values["$reader"];
    let request = format!(
        "challenge(ScarcityWaitingClaimant, {reader}, ScarcityWorkedRecord).\nobserve(ScarcityWaitingClaimant, ScarcityWorkedRecord, IndivisibleEssentialPowerUnit, ScarcityChallengeResourceScope).\n"
    );
    let mut steps = ordinary.clone() + &request;
    steps.push_str(&query(
        &format!(
            "obliged({reader}, ReviewScarcityFindingAllocationOrShortfall, ScarcityWorkedRecord)"
        ),
        true,
    ));
    // Evidence the claimant submits with the request must be weighed. An entry
    // somebody else writes on the claimant's behalf does not create that duty,
    // and weighing is owed without the allocation losing its authority.
    let weigh = format!(
        "obliged({reader}, WeighTheClaimantsSubmittedEvidenceAgainstTheComparison, ScarcityWorkedRecord)"
    );
    steps.push_str(&query(&weigh, false));
    steps.push_str("observe(ScarcityWorkedSource, ScarcityWorkedRecord, ScarcityWaitingClaimantsSubmittedEvidence, ScarcityChallengeEvidenceScope).\n");
    steps.push_str(&query(&weigh, false));
    steps.push_str("observe(ScarcityWaitingClaimant, ScarcityWorkedRecord, ScarcityWaitingClaimantsSubmittedEvidence, ScarcityChallengeEvidenceScope).\n");
    steps.push_str(&query(&weigh, true));
    steps.push_str(&queries(allocation, &values, true));
    let defect = get("defect");
    let mut defect_values = bindings(source, defect, "ScarcityComparisonDefect");
    for name in CARRIED {
        defect_values.insert(name.into(), values[name].clone());
    }
    defect_values.insert("$target".into(), values["$record"].clone());
    defect_values.insert("$target_source".into(), values["$source"].clone());
    defect_values.insert(
        "$defect".into(),
        "UnsupportedAllocationComparisonOrReasons".into(),
    );
    steps.push_str(&fixture(source, defect, &defect_values));
    steps.push_str(&queries(defect, &defect_values, true));
    steps.push_str(&queries(allocation, &values, false));
    steps.push_str(&queries(shortfall, &unmet, true));
    steps.push_str(&query(
        "complete(ScarcityWorkedAuthorization, ReviewedPhysicalScarcityFinding, IndivisibleEssentialPowerUnit)",
        true,
    ));
    steps.push_str(&retained);
    add_case(
        context,
        export,
        "worked/challenge-and-comparison-defect",
        "live",
        &facts,
        &steps,
    )?;

    for (label, variable, value) in [
        (
            "forbidden-worth-priority",
            "$mitigation_key",
            "GeneralizedLifespan",
        ),
        (
            "lottery-for-unequal-claims",
            "$allocation_method",
            "DisclosedLottery",
        ),
    ] {
        let mut other = values.clone();
        other.insert(variable.into(), value.into());
        add_case(
            context,
            export,
            &format!("worked/{label}"),
            "live",
            &(dependency(source, allocation, &other)
                + &fixture(source, allocation, &other)
                + people),
            &(queries(allocation, &other, false) + &retained),
        )?;
    }

    // The supplied lottery result can name either equal claimant. The model
    // accepts the qualified decision; it does not draw the winner itself.
    for (label, decision) in [
        ("waiting", "LotteryResultAssignsThisWindowToWaitingClaim"),
        (
            "continuity",
            "LotteryResultAssignsThisWindowToContinuityClaim",
        ),
    ] {
        let mut equal = values.clone();
        equal.insert(
            "$comparison_outcome".into(),
            "NoUsableEqualShareAndMateriallyEqualClaims".into(),
        );
        equal.insert("$allocation_method".into(), "DisclosedLottery".into());
        equal.insert("$allocation".into(), decision.into());
        equal.insert(
            "$reasons".into(),
            "EqualUrgencyAccessIrreversibleHarmContinuityAndResourceSpecificBenefitAfterAccommodationDisclosedLotteryForThisWindow".into(),
        );
        add_case(
            context,
            export,
            &format!("worked/materially-equal-lottery-{label}"),
            "live",
            &(dependency(source, allocation, &equal)
                + &fixture(source, allocation, &equal)
                + people),
            &(queries(allocation, &equal, true) + &retained),
        )?;
    }

    let false_scarcity = get("false-scarcity");
    let mut refusal = bindings(source, false_scarcity, "ScarcityProcurementRefusal");
    for name in CARRIED {
        refusal.insert(name.into(), values[name].clone());
    }
    refusal.insert("$target".into(), values["$authorization"].clone());
    refusal.insert(
        "$target_source".into(),
        values["$authorization_source"].clone(),
    );
    refusal.insert("$false_kind".into(), "RefusalToProcure".into());
    let mut steps = ordinary + &fixture(source, false_scarcity, &refusal);
    steps.push_str(&queries(false_scarcity, &refusal, true));
    steps.push_str(&queries(allocation, &values, false));
    steps.push_str(&queries(shortfall, &unmet, false));
    steps.push_str(&retained);
    add_case(
        context,
        export,
        "worked/procurement-refusal-is-not-scarcity",
        "live",
        &facts,
        &steps,
    )?;
    Ok(())
}
