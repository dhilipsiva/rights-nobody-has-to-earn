// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reader-experience coverage ledger: one record per derived-chapter
//! section and per substantive Part V passage, projected into a report.
//!
//! It classifies what the reader actually meets — which domain, which rule
//! family, whether the passage shows ordinary operation or a protection, what
//! the person in it is doing, and how it ends. Its value is the gaps it
//! exposes, so the report prints those first and does not average them away.

use super::Export;
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const SOURCE: &str = "new-book-plans/reader-coverage-source.json";
const REPORT: &str = "new-book-plans/reader-coverage.md";

/// A chapter's text before its first heading.
pub(crate) const PREAMBLE: &str = "(preamble)";

/// Trajectories that show the design under strain rather than working.
const TROUBLE: [&str; 4] = ["contested", "fails", "continuity-remedy", "unresolved"];

/// A passage states a boundary when it says what it does NOT establish. This is
/// read out of the prose rather than declared in the source, so a record cannot
/// claim a disclosure the section does not make. Measured 2026-09-15: 55 of the
/// 86 passages match, which is why it discriminates rather than passing
/// everything.
fn boundary() -> Regex {
    Regex::new(
        r"(?i)\b(?:establish(?:es)? no|prove[sd]? no|do(?:es)? not (?:establish|prove|claim|show|mean|make|create|authorize|authorise|say|tell|reach|decide)|none of (?:this|these|it)|nothing (?:here|in this)|is not (?:evidence|proof|a claim)|remains? (?:an )?external assumption|neither [a-z ]{1,40} nor |no rule (?:in this|reads|converts)|cannot (?:prove|establish|show|tell))",
    )
    .expect("boundary pattern")
}

/// The text of one passage, from its heading to the next.
pub(crate) fn passage(context: &Context, chapter: &str, section: &str) -> Result<String, Error> {
    let source = context.read(chapter)?;
    if section == PREAMBLE {
        let end = source.find("\n## ").unwrap_or(source.len());
        return Ok(source[..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "));
    }
    let start = source
        .find(&format!("## {section}"))
        .ok_or_else(|| Error::new(format!("{chapter}: no passage {section:?}")))?;
    let rest = &source[start..];
    let end = rest[1..].find("\n## ").map_or(rest.len(), |at| at + 1);
    Ok(rest[..end].split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Ordinary operation, strain, and whether a boundary is stated.
pub(crate) fn states_boundary(context: &Context, record: &Record) -> Result<bool, Error> {
    Ok(boundary().is_match(&passage(context, &record.chapter, &record.section)?))
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub(crate) id: String,
    pub(crate) chapter: String,
    pub(crate) section: String,
    pub(crate) domain: String,
    pub(crate) family: String,
    pub(crate) function: String,
    pub(crate) setting: String,
    pub(crate) postures: Vec<String>,
    pub(crate) trajectory: String,
    pub(crate) pattern: String,
    pub(crate) basis: String,
}

/// Every posture the portfolio standard names. A posture nobody occupies is a
/// kind of person the book never shows doing that thing.
pub(crate) const POSTURES: [&str; 10] = [
    "chooses",
    "creates",
    "cares",
    "works",
    "associates",
    "requests",
    "receives",
    "challenges",
    "governs",
    "is acted upon",
];

/// The four chapter patterns the ruling names, plus the two this book's own
/// shape adds: the record chapters, which are about what may be written at all,
/// and Part V's argument, which is exempt from derivation.
pub(crate) const PATTERNS: [&str; 6] = [
    "constructive",
    "private-civic",
    "democratic",
    "coercive",
    "records",
    "argument",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    records: Vec<Record>,
}

pub(crate) fn records(context: &Context) -> Result<Vec<Record>, Error> {
    let source: Source = serde_json::from_str(&context.read(SOURCE)?)?;
    Ok(source.records)
}

/// Every `## ` heading of a derived chapter and of Part V, in file order. The
/// opening note is one of the three exempt elements and is deliberately absent.
pub(crate) fn headings(context: &Context) -> Result<Vec<(String, String)>, Error> {
    let mut rows = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(context.path("book-1"))?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.ends_with(".md")
                && name.starts_with(|c: char| c.is_ascii_digit())
                && name != "00-opening-note.md"
        })
        .collect();
    files.sort();
    for name in files {
        let path = format!("book-1/{name}");
        // Everything before the first heading is a passage too, and it is where
        // several chapters do their most substantive work.
        rows.push((path.clone(), PREAMBLE.to_owned()));
        for line in context.read(&path)?.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                rows.push((path.clone(), heading.trim().to_owned()));
            }
        }
    }
    Ok(rows)
}

fn validate(context: &Context, records: &[Record]) -> Result<(), Error> {
    let declared: Vec<(String, String)> = records
        .iter()
        .map(|record| (record.chapter.clone(), record.section.clone()))
        .collect();
    let present = headings(context)?;
    if declared != present {
        let missing: Vec<_> = present
            .iter()
            .filter(|row| !declared.contains(row))
            .take(4)
            .collect();
        let extra: Vec<_> = declared
            .iter()
            .filter(|row| !present.contains(row))
            .take(4)
            .collect();
        return Err(Error::new(format!(
            "the ledger and the book disagree. Unclassified passages: {missing:?}. \
             Records with no passage: {extra:?}"
        )));
    }
    let mut seen = BTreeSet::new();
    for record in records {
        if !seen.insert(&record.id) {
            return Err(Error::new(format!("duplicate record {}", record.id)));
        }
        if record.basis != "exempt-element" && !context.path(&record.basis).exists() {
            return Err(Error::new(format!(
                "{}: basis {} does not exist",
                record.id, record.basis
            )));
        }
        if !TROUBLE.contains(&record.trajectory.as_str()) && record.trajectory != "works" {
            return Err(Error::new(format!(
                "{}: unknown trajectory {}",
                record.id, record.trajectory
            )));
        }
        if record.postures.is_empty() {
            return Err(Error::new(format!("{}: no posture", record.id)));
        }
        for posture in &record.postures {
            if !POSTURES.contains(&posture.as_str()) {
                return Err(Error::new(format!(
                    "{}: unknown posture {posture}",
                    record.id
                )));
            }
        }
        if !PATTERNS.contains(&record.pattern.as_str()) {
            return Err(Error::new(format!(
                "{}: unknown pattern {}",
                record.id, record.pattern
            )));
        }
    }
    Ok(())
}

/// Per domain: passages showing ordinary operation, passages showing strain,
/// and passages stating a boundary.
pub(crate) fn coverage(
    context: &Context,
    records: &[Record],
) -> Result<BTreeMap<String, (usize, usize, usize)>, Error> {
    let mut rows: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for record in records {
        let entry = rows.entry(record.domain.clone()).or_default();
        if record.trajectory == "works" {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
        if states_boundary(context, record)? {
            entry.2 += 1;
        }
    }
    Ok(rows)
}

fn render(context: &Context, records: &[Record]) -> Result<String, Error> {
    let mut out = String::from(
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n\n\
         # Reader-experience coverage\n\n\
         Generated by `./generate.sh reader-coverage` from\n\
         `reader-coverage-source.json`. One row per derived-chapter section and\n\
         per Part V passage. The opening note is an exempt element and is not\n\
         classified here.\n\n\
         A domain needs ordinary operation **and** a credible failure, abuse or\n\
         boundary. Whether a passage states a boundary is read out of its own\n\
         prose — what it says it does not establish — rather than declared here,\n\
         so no row can claim a disclosure its section does not make.\n\n\
         This is a map of what a reader meets, not a claim about whether they\n\
         understand it. No reader has been asked anything; R6 is unbuilt, and\n\
         nothing here is reader evidence.\n\n\
         ## Where a domain is thin\n\n\
         These have ordinary operation or strain, and not the other.\n\n\
         | Domain | Ordinary | Strain | Boundary | Missing |\n| --- | ---: | ---: | ---: | --- |\n",
    );
    let rows = coverage(context, records)?;
    let mut thin = 0;
    for (domain, (works, trouble, bound)) in &rows {
        let missing = match (works, trouble, bound) {
            (0, _, _) => "ordinary operation",
            (_, 0, 0) => "a failure, abuse or boundary",
            _ => continue,
        };
        thin += 1;
        let _ = writeln!(
            out,
            "| {domain} | {works} | {trouble} | {bound} | {missing} |"
        );
    }
    if thin == 0 {
        out.push_str("| — | | | | none |\n");
    }
    out.push_str(
        "\n## Shown working, bounded, but never failing\n\n\
         These meet the standard through a stated boundary rather than through a\n\
         passage in which something goes wrong. That is the weaker of the two\n\
         forms, and it is where the portfolio rebalance has most to do.\n\n\
         | Domain | Ordinary | Strain | Boundary |\n| --- | ---: | ---: | ---: |\n",
    );
    for (domain, (works, trouble, bound)) in &rows {
        if *trouble == 0 && *bound > 0 {
            let _ = writeln!(out, "| {domain} | {works} | {trouble} | {bound} |");
        }
    }
    out.push_str(
        "\n## Postures\n\n\
         A posture nobody occupies is a kind of person the book never shows\n\
         doing that thing, and a posture carried by one or two passages is\n\
         nearly that. Every posture is occupied; `cares` and `creates` are\n\
         carried in single figures, which is where the rebalance has most to do.\n\n\
         | Posture | Passages |\n| --- | ---: |\n",
    );
    for posture in POSTURES {
        let count = records
            .iter()
            .filter(|record| record.postures.iter().any(|held| held == posture))
            .count();
        let _ = writeln!(out, "| {posture} | {count} |");
    }
    out.push_str(
        "\n## Chapter patterns\n\n\
         The refusal here is one failure-first formula for everything. A\n\
         chapter being of one pattern is the design — each has a subject — so\n\
         what matters is the book's shape, not the chapter's.\n\n\
         | Pattern | Passages |\n| --- | ---: |\n",
    );
    for pattern in PATTERNS {
        let count = records
            .iter()
            .filter(|record| record.pattern == pattern)
            .count();
        let _ = writeln!(out, "| {pattern} | {count} |");
    }
    out.push_str(
        "\nWhether a passage follows its pattern's own arc — seeks, responds,\n\
         receipt, challenge, continuity, boundary — is prose review and is not\n\
         checked here.\n",
    );
    out.push_str("\n## Every domain\n\n| Domain | Ordinary | Strain | Boundary |\n| --- | ---: | ---: | ---: |\n");
    for (domain, (works, trouble, bound)) in &rows {
        let _ = writeln!(out, "| {domain} | {works} | {trouble} | {bound} |");
    }
    out.push_str("\n## Every passage\n\n| ID | Chapter | Section | Domain | Family | Function | Setting | Posture | Trajectory | Boundary | Pattern | Basis |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for record in records {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
            record.id,
            record.chapter.trim_start_matches("book-1/"),
            record.section,
            record.domain,
            record.family,
            record.function,
            record.setting,
            record.postures.join(", "),
            record.trajectory,
            if states_boundary(context, record)? {
                "yes"
            } else {
                "no"
            },
            record.pattern,
            record.basis
        );
    }
    Ok(out)
}

pub(crate) fn generate(context: &Context, _export: &mut Export) -> Result<(), Error> {
    let records = records(context)?;
    validate(context, &records)?;
    std::fs::write(context.path(REPORT), render(context, &records)?)?;
    println!("reader coverage: {} passages classified", records.len());
    Ok(())
}

#[cfg(test)]
#[path = "reader_coverage_tests.rs"]
mod tests;
