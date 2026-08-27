// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer};

use crate::cli::Error;
use crate::context::Context;

const MAX_AGE_DAYS: i64 = 400;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    #[serde(default)]
    spec: Option<RegistrySpec>,
    claims: Vec<Claim>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistrySpec {
    version: u64,
    licence: String,
    entry_fields: EntryFields,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryFields {
    id: String,
    claim: String,
    value: String,
    units: String,
    source: String,
    retrieved: String,
    method: String,
    fetch: String,
    notes: String,
    book: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    #[serde(default)]
    id: Presence<String>,
    #[serde(default)]
    claim: Presence<String>,
    #[serde(default)]
    value: Presence<ClaimValue>,
    #[serde(default)]
    units: Presence<String>,
    #[serde(default)]
    source: Presence<String>,
    #[serde(default)]
    retrieved: Presence<String>,
    #[serde(default)]
    method: Presence<String>,
    #[serde(default)]
    fetch: Presence<String>,
    #[serde(default)]
    notes: Presence<String>,
    #[serde(default)]
    book: Option<String>,
    #[serde(default)]
    observation_year: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClaimValue {
    Number(f64),
    Range([f64; 2]),
    Structured(BTreeMap<String, RegistryDatum>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RegistryDatum {
    Number(f64),
    List(Vec<RegistryDatum>),
    Object(BTreeMap<String, RegistryDatum>),
}

#[derive(Debug)]
enum Presence<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for Presence<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Presence<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T> Presence<T> {
    fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    fn is_null_or_missing(&self) -> bool {
        matches!(self, Self::Missing | Self::Null)
    }

    fn value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Report {
    claims: usize,
    fetchable: usize,
    problems: Vec<String>,
}

pub(crate) fn check(context: &Context) -> Result<String, Error> {
    let source = context.read("registry/claims.json")?;
    let report = evaluate(&source, utc_today_days()?)?;
    if !report.problems.is_empty() {
        return Err(Error::new(failure_message(&report)));
    }
    Ok(success_message(&report))
}

fn success_message(report: &Report) -> String {
    format!(
        "registry ok: {} claims ({} fetchable, {} pinned)",
        report.claims,
        report.fetchable,
        report.claims - report.fetchable
    )
}

fn failure_message(report: &Report) -> String {
    format!("registry check FAILED:\n  {}", report.problems.join("\n  "))
}

fn evaluate(source: &str, today_days: i64) -> Result<Report, Error> {
    let registry: Registry = serde_json::from_str(source)?;
    if let Some(spec) = &registry.spec {
        validate_spec(spec)?;
    }
    let claims = &registry.claims;
    let mut seen = HashSet::new();
    let mut problems = Vec::new();
    let mut fetchable = 0;

    for claim in claims {
        let identifier = match &claim.id {
            Presence::Value(value) => value.clone(),
            Presence::Null => "None".to_owned(),
            Presence::Missing => "<missing id>".to_owned(),
        };
        if !seen.insert(identifier.clone()) {
            problems.push(format!("{identifier}: duplicate id"));
        }
        for (key, missing) in [
            ("id", claim.id.is_missing()),
            ("claim", claim.claim.is_missing()),
            ("value", claim.value.is_missing()),
            ("units", claim.units.is_missing()),
            ("source", claim.source.is_missing()),
            ("retrieved", claim.retrieved.is_missing()),
            ("method", claim.method.is_missing()),
            ("fetch", claim.fetch.is_missing()),
            ("notes", claim.notes.is_missing()),
        ] {
            if missing {
                problems.push(format!("{identifier}: missing field '{key}'"));
            }
        }

        if let Some(value) = claim.value.value() {
            value.validate()?;
        }
        if claim.book.as_ref().is_some_and(String::is_empty) {
            return Err(Error::new(format!(
                "{identifier}: book must not be an empty string"
            )));
        }
        if claim.observation_year == Some(0) {
            return Err(Error::new(format!(
                "{identifier}: observation_year must be positive"
            )));
        }
        if claim.fetch.value().is_some_and(|value| !value.is_empty()) {
            fetchable += 1;
            if claim.value.is_null_or_missing()
                && claim
                    .retrieved
                    .value()
                    .is_some_and(|value| !value.is_empty())
            {
                problems.push(format!(
                    "{identifier}: fetchable entry has retrieved-date but null value \
— fetcher half-ran?"
                ));
            }
            if let Some(retrieved_text) = claim.retrieved.value().filter(|value| !value.is_empty())
            {
                let retrieved_days = parse_iso_date(retrieved_text).ok_or_else(|| {
                    Error::new(format!(
                        "{identifier}: invalid ISO retrieval date {retrieved_text}"
                    ))
                })?;
                if today_days - retrieved_days > MAX_AGE_DAYS {
                    problems.push(format!(
                        "{identifier}: stale — retrieved {retrieved_text}, over \
{MAX_AGE_DAYS} days; re-run: {}",
                        claim.fetch.value().expect("truthy fetch exists")
                    ));
                }
            }
        }
    }

    Ok(Report {
        claims: claims.len(),
        fetchable,
        problems,
    })
}

fn validate_spec(spec: &RegistrySpec) -> Result<(), Error> {
    if spec.version != 1 || spec.licence.is_empty() {
        return Err(Error::new("registry spec metadata is invalid"));
    }
    let fields = &spec.entry_fields;
    for (name, description) in [
        ("id", &fields.id),
        ("claim", &fields.claim),
        ("value", &fields.value),
        ("units", &fields.units),
        ("source", &fields.source),
        ("retrieved", &fields.retrieved),
        ("method", &fields.method),
        ("fetch", &fields.fetch),
        ("notes", &fields.notes),
        ("book", &fields.book),
    ] {
        if description.is_empty() {
            return Err(Error::new(format!(
                "registry spec entry_fields.{name} must not be empty"
            )));
        }
    }
    Ok(())
}

impl ClaimValue {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Number(value) if value.is_finite() => Ok(()),
            Self::Number(_) => Err(Error::new("registry value contains a non-finite number")),
            Self::Range(values) if values.iter().all(|value| value.is_finite()) => Ok(()),
            Self::Range(_) => Err(Error::new("registry range contains a non-finite number")),
            Self::Structured(values) => {
                if values.keys().any(|key| key.is_empty()) {
                    return Err(Error::new("registry value contains an empty metric name"));
                }
                values.values().try_for_each(RegistryDatum::validate)
            }
        }
    }
}

impl RegistryDatum {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Number(value) if value.is_finite() => Ok(()),
            Self::Number(_) => Err(Error::new("registry metric contains a non-finite number")),
            Self::List(values) => values.iter().try_for_each(Self::validate),
            Self::Object(values) => {
                if values.keys().any(|key| key.is_empty()) {
                    return Err(Error::new("registry metric contains an empty name"));
                }
                values.values().try_for_each(Self::validate)
            }
        }
    }
}

fn utc_today_days() -> Result<i64, Error> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::new(format!("system clock predates Unix epoch: {error}")))?;
    Ok((duration.as_secs() / 86_400) as i64)
}

fn parse_iso_date(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = parse_digits(&bytes[0..4])? as i64;
    let month = parse_digits(&bytes[5..7])? as u32;
    let day = parse_digits(&bytes[8..10])? as u32;
    if year == 0 || !(1..=12).contains(&month) {
        return None;
    }
    let maximum_day = match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=maximum_day).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn parse_digits(value: &[u8]) -> Option<u32> {
    value.iter().try_fold(0_u32, |number, digit| {
        digit
            .is_ascii_digit()
            .then_some(number * 10 + u32::from(*digit - b'0'))
    })
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::{
        check, days_from_civil, evaluate, failure_message, parse_iso_date, success_message,
    };
    use crate::context::Context;

    #[test]
    fn live_registry_matches_the_typed_schema() {
        let context = Context::discover().expect("discover repository");
        let message = check(&context).expect("live registry check");
        assert!(message.starts_with("registry ok:"));
    }

    #[test]
    fn valid_registry_matches_success_counts() {
        let report = evaluate(
            r#"{"claims":[
                {"id":"live","claim":"c","value":1,"units":"u","source":"s",
                 "retrieved":"2026-08-27","method":"m","fetch":"fetch.py","notes":"n"},
                {"id":"paper","claim":"c","value":2,"units":"u","source":"s",
                 "retrieved":"2000-01-01","method":"m","fetch":null,"notes":"n"}
            ]}"#,
            days_from_civil(2026, 8, 27),
        )
        .unwrap();
        assert_eq!(report.claims, 2);
        assert_eq!(report.fetchable, 1);
        assert!(report.problems.is_empty());
        assert_eq!(
            success_message(&report),
            "registry ok: 2 claims (1 fetchable, 1 pinned)"
        );
    }

    #[test]
    fn failures_preserve_python_check_order_and_wording() {
        let report = evaluate(
            r#"{"claims":[
                {"id":"dup","fetch":"refresh.py","retrieved":"2025-01-01","value":null},
                {"id":"dup","claim":"c","value":1,"units":"u","source":"s",
                 "retrieved":null,"method":"m","fetch":null,"notes":"n"}
            ]}"#,
            days_from_civil(2026, 8, 27),
        )
        .unwrap();
        assert_eq!(
            report.problems,
            vec![
                "dup: missing field 'claim'",
                "dup: missing field 'units'",
                "dup: missing field 'source'",
                "dup: missing field 'method'",
                "dup: missing field 'notes'",
                "dup: fetchable entry has retrieved-date but null value — fetcher half-ran?",
                "dup: stale — retrieved 2025-01-01, over 400 days; re-run: refresh.py",
                "dup: duplicate id",
            ]
        );
        assert_eq!(
            failure_message(&report),
            "registry check FAILED:\n  dup: missing field 'claim'\n  \
dup: missing field 'units'\n  dup: missing field 'source'\n  \
dup: missing field 'method'\n  dup: missing field 'notes'\n  \
dup: fetchable entry has retrieved-date but null value — fetcher half-ran?\n  \
dup: stale — retrieved 2025-01-01, over 400 days; re-run: refresh.py\n  \
dup: duplicate id"
        );
    }

    #[test]
    fn missing_or_null_retrieval_date_is_not_an_extra_failure() {
        let report = evaluate(
            r#"{"claims":[
                {"id":"missing","claim":"c","value":1,"units":"u","source":"s",
                 "method":"m","fetch":"refresh.py","notes":"n"},
                {"id":"null","claim":"c","value":1,"units":"u","source":"s",
                 "retrieved":null,"method":"m","fetch":"refresh.py","notes":"n"}
            ]}"#,
            days_from_civil(2026, 8, 27),
        )
        .unwrap();
        assert_eq!(report.problems, vec!["missing: missing field 'retrieved'"]);
    }

    #[test]
    fn staleness_is_strictly_more_than_four_hundred_days() {
        let today = days_from_civil(2026, 8, 27);
        let exact = days_from_civil(2025, 7, 23);
        assert_eq!(today - exact, 400);
        let source = |retrieved: &str| {
            format!(
                "{{\"claims\":[{{\"id\":\"x\",\"claim\":\"c\",\"value\":1,\"units\":\"u\",\"source\":\"s\",\"retrieved\":\"{retrieved}\",\"method\":\"m\",\"fetch\":\"f\",\"notes\":\"n\"}}]}}"
            )
        };
        assert!(
            evaluate(&source("2025-07-23"), today)
                .unwrap()
                .problems
                .is_empty()
        );
        assert_eq!(
            evaluate(&source("2025-07-22"), today)
                .unwrap()
                .problems
                .len(),
            1
        );
    }

    #[test]
    fn iso_date_parser_validates_leap_days_and_epoch() {
        assert_eq!(parse_iso_date("1970-01-01"), Some(0));
        assert!(parse_iso_date("2024-02-29").is_some());
        assert_eq!(parse_iso_date("2025-02-29"), None);
    }

    #[test]
    fn typed_claim_contract_rejects_unknown_fields() {
        let error = evaluate(
            r#"{"claims":[{"id":"x","claim":"c","value":1,"units":"u",
                "source":"s","retrieved":null,"method":"m","fetch":null,
                "notes":"n","surprise":true}]}"#,
            days_from_civil(2026, 8, 27),
        )
        .expect_err("unknown field must fail");
        assert!(error.to_string().contains("unknown field `surprise`"));
    }
}
