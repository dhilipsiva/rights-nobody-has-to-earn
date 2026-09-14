// SPDX-License-Identifier: MIT OR Apache-2.0

//! Read the actual retained economic contract and its ordinary positive facts.

use super::{Values, records};
use crate::{cli::Error, context::Context};

fn head(number: usize) -> Result<String, Error> {
    if !matches!(number, 81 | 82) {
        return Err(Error::new("undeclared ecological economic interface"));
    }
    Ok(format!("authority(FSBOD_09, FSPOW_{number:03}, $record)"))
}

pub(super) fn consumer(context: &Context, number: usize) -> Result<Vec<String>, Error> {
    let head = head(number)?;
    let source = context.read("new-book-plans/constitution.nibli")?;
    let suffix = format!(" -> {head}.");
    let rules: Vec<_> = source
        .lines()
        .filter_map(|line| line.strip_suffix(&suffix))
        .collect();
    if rules.len() != 1 {
        return Err(Error::new(format!(
            "expected one actual {head} source, found {}",
            rules.len()
        )));
    }
    let quantifiers = regex::Regex::new(r"^(?:all \$[a-z][a-z0-9_]*: )+").unwrap();
    let body = quantifiers.replace(rules[0], "");
    if body == rules[0] {
        return Err(Error::new(
            "economic source has an ambiguous quantifier boundary",
        ));
    }
    let mut atoms: Vec<_> = body.split(" & ").map(str::to_owned).collect();
    atoms.push(head);
    for actor in ["$source", "$review"] {
        for (variable, scope) in [
            ("$resource", "ScarceResourceScope"),
            ("$population", "ScarcityPopulationScope"),
            (
                "$alternatives_or_allocation",
                if number == 81 {
                    "ScarcityAlternativesRecordScope"
                } else {
                    "ScarcityAllocationRecordScope"
                },
            ),
        ] {
            atoms.push(format!("observe({actor}, $record, {variable}, {scope})"));
        }
    }
    Ok(atoms)
}

fn positive(context: &Context, number: usize) -> Result<(String, Values), Error> {
    head(number)?;
    let source = context.read(&format!(
        "new-book-plans/economic-power-{number:03}.pins.nibli"
    ))?;
    let mut lines = Vec::new();
    let mut reached_query = false;
    for line in source.lines().map(str::trim) {
        if line.starts_with('?') {
            reached_query = true;
            break;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with(":expect-pins ") {
            continue;
        }
        if !line.starts_with("authorized(") && !line.starts_with("observe(") {
            return Err(Error::new(
                "economic positive source contains an undeclared statement",
            ));
        }
        lines.push(line.to_owned());
    }
    if !reached_query {
        return Err(Error::new("economic positive source has no query boundary"));
    }
    let record = format!("EconRecord{number:03}Live");
    let mut values = Values::new();
    values.insert("$record".into(), record.clone());
    for (variable, role) in [
        ("$source", "EconomicSourceAuthority"),
        ("$evidence", "EconomicEvidenceAuthority"),
        ("$review", "EconomicIndependentReviewAuthority"),
        ("$auditor", "EconomicAuditAuthority"),
        ("$final_review", "EconomicFinalReviewAuthority"),
        ("$executor", "EconomicExecutionAuthority"),
    ] {
        let suffix = format!(", {role}, {record}).");
        let actors: std::collections::BTreeSet<_> = lines
            .iter()
            .filter_map(|line| line.strip_prefix("authorized(")?.strip_suffix(&suffix))
            .collect();
        if actors.len() != 1 {
            return Err(Error::new(format!(
                "economic {number} positive has ambiguous {role}"
            )));
        }
        values.insert(variable.into(), (*actors.first().unwrap()).into());
    }
    let prefix = format!("observe({}, {record}, ", values["$source"]);
    for (variable, scope) in [
        ("$result", "ResultScope"),
        ("$version", "SourceVersionScope"),
        ("$epoch", "SourceEpochScope"),
        ("$temporal_record", "TemporalRecordScope"),
        ("$case", "EconomicCaseScope"),
        ("$jurisdiction", "JurisdictionScope"),
        ("$legal_scope", "AuthorityScope"),
        ("$end", "EndConditionScope"),
        ("$resource", "ScarceResourceScope"),
        ("$population", "ScarcityPopulationScope"),
        (
            "$alternatives_or_allocation",
            if number == 81 {
                "ScarcityAlternativesRecordScope"
            } else {
                "ScarcityAllocationRecordScope"
            },
        ),
    ] {
        let suffix = format!(", {scope}).");
        let observed: std::collections::BTreeSet<_> = lines
            .iter()
            .filter_map(|line| line.strip_prefix(&prefix)?.strip_suffix(&suffix))
            .collect();
        if observed.len() != 1 {
            return Err(Error::new(format!(
                "economic {number} positive has ambiguous {scope}"
            )));
        }
        values.insert(variable.into(), (*observed.first().unwrap()).into());
    }
    let facts = lines.join("\n") + "\n";
    let opaque = regex::Regex::new(&format!(
        r"\bEcon[A-Za-z0-9_]*{number:03}Live[A-Za-z0-9_]*\b"
    ))
    .unwrap();
    for name in opaque
        .find_iter(&facts)
        .map(|m| m.as_str())
        .collect::<std::collections::BTreeSet<_>>()
    {
        if !values.values().any(|value| value == name) {
            values.insert(format!("$fixture_{}", values.len()), name.into());
        }
    }
    for variable in records::pattern().find_iter(&consumer(context, number)?.join(" ")) {
        if !values.contains_key(variable.as_str()) {
            return Err(Error::new(format!(
                "economic source binding missing {}",
                variable.as_str()
            )));
        }
    }
    Ok((facts, values))
}

pub(super) fn example(
    context: &Context,
    number: usize,
    prefix: &str,
    bindings: &[(&str, &str)],
) -> Result<(String, Values), Error> {
    let (facts, originals) = positive(context, number)?;
    let mut values: Values = originals
        .keys()
        .map(|variable| {
            (
                variable.clone(),
                format!(
                    "{prefix}{}",
                    variable.trim_start_matches('$').replace('_', "")
                ),
            )
        })
        .collect();
    for (variable, value) in bindings {
        if !values.contains_key(*variable) {
            return Err(Error::new(format!("unknown economic binding {variable}")));
        }
        values.insert((*variable).into(), (*value).into());
    }
    let mut renames = Values::new();
    for (variable, original) in originals {
        let value = values[&variable].clone();
        if let Some(old) = renames.insert(original, value.clone()) {
            if old != value {
                return Err(Error::new("conflicting economic fixture identity bindings"));
            }
        }
    }
    let token = regex::Regex::new(r"\b[A-Za-z][A-Za-z0-9_]*\b").unwrap();
    let facts = token
        .replace_all(&facts, |m: &regex::Captures<'_>| {
            renames.get(&m[0]).cloned().unwrap_or_else(|| m[0].into())
        })
        .into_owned();
    Ok((facts, values))
}
