// SPDX-License-Identifier: MIT OR Apache-2.0

//! Claim-scoped resolution receipts: one per thread where the book identifies a
//! defect, claims a repair, or uses a failure as a witness.
//!
//! Each thread ends in exactly one of five states. Naming a limitation and
//! moving on is not one of them, which is why `resolved-for-claim` needs a rerun
//! of the former attack and every state needs what still does not follow.

use super::Export;
use crate::{cli::Error, context::Context};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt::Write as _;

const SOURCE: &str = "new-book-plans/resolution-receipts-source.json";
const REPORT: &str = "new-book-plans/resolution-receipts.md";

/// The only honest endings. `resolved-for-claim` closes the narrower claim its
/// rerun establishes and nothing wider; the other four do not close anything.
pub(crate) const STATES: [&str; 5] = [
    "resolved-for-claim",
    "operationally-unresolved",
    "externally-bounded",
    "irreducible-limitation",
    "open-defect",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    pub(crate) id: String,
    pub(crate) chapter: String,
    pub(crate) section: String,
    pub(crate) what_failed: String,
    pub(crate) what_changed: String,
    pub(crate) state: String,
    pub(crate) rerun: String,
    pub(crate) phrase: String,
    pub(crate) still_not_established: String,
    pub(crate) external_or_open: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    receipts: Vec<Receipt>,
}

pub(crate) fn receipts(context: &Context) -> Result<Vec<Receipt>, Error> {
    let source: Source = serde_json::from_str(&context.read(SOURCE)?)?;
    Ok(source.receipts)
}

pub(crate) fn validate(context: &Context, receipts: &[Receipt]) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    for receipt in receipts {
        if !seen.insert(&receipt.id) {
            return Err(Error::new(format!("duplicate receipt {}", receipt.id)));
        }
        if !STATES.contains(&receipt.state.as_str()) {
            return Err(Error::new(format!(
                "{}: {} is not one of the five endings",
                receipt.id, receipt.state
            )));
        }
        if !context.path(&receipt.rerun).exists() {
            return Err(Error::new(format!(
                "{}: rerun {} does not exist. A repair with nothing to rerun is a \
                 claim, not a receipt.",
                receipt.id, receipt.rerun
            )));
        }
        let prose = context.read(&receipt.chapter)?;
        if !prose.contains(&format!("## {}", receipt.section)) {
            return Err(Error::new(format!(
                "{}: {} has no section {:?}",
                receipt.id, receipt.chapter, receipt.section
            )));
        }
        let flat = prose.split_whitespace().collect::<Vec<_>>().join(" ");
        if !flat.contains(&receipt.phrase) {
            return Err(Error::new(format!(
                "{}: the book no longer says {:?}, so this receipt describes a \
                 thread that is not there",
                receipt.id, receipt.phrase
            )));
        }
        for (field, text) in [
            ("what_failed", &receipt.what_failed),
            ("what_changed", &receipt.what_changed),
            ("still_not_established", &receipt.still_not_established),
        ] {
            if text.len() < 40 {
                return Err(Error::new(format!(
                    "{}: {field} is too short to be a receipt",
                    receipt.id
                )));
            }
        }
    }
    // Every chapter that NARRATES a repair must carry a receipt. The detector is
    // the book's own idiom for telling one — "used to", "no longer", "the
    // repair", "it is resolved" — so a chapter cannot claim a repair in prose
    // and leave it without an ending.
    let narrates = regex::Regex::new(
        r"(?i)(used to (?:be|get|have|say|look)|no longer (?:has|have|is|does|destroys)|the repair|now closed|it is resolved|was wrong for|this design used to|earlier version of this design|the resolution was)",
    )
    .expect("repair pattern");
    let receipted: BTreeSet<&str> = receipts
        .iter()
        .map(|receipt| receipt.chapter.as_str())
        .collect();
    let mut chapters: Vec<String> = std::fs::read_dir(context.path("book-1"))?
        .filter_map(Result::ok)
        .map(|entry| format!("book-1/{}", entry.file_name().to_string_lossy()))
        .filter(|name| {
            name.ends_with(".md")
                && name["book-1/".len()..].starts_with(|c: char| c.is_ascii_digit())
                && !name.ends_with("00-opening-note.md")
        })
        .collect();
    chapters.sort();
    for chapter in chapters {
        let prose = context.read(&chapter)?;
        let flat = prose.split_whitespace().collect::<Vec<_>>().join(" ");
        if narrates.is_match(&flat) && !receipted.contains(chapter.as_str()) {
            return Err(Error::new(format!(
                "{chapter} narrates a repair and has no receipt. A thread the                  book tells has to end somewhere."
            )));
        }
    }
    // Disclosure is not closure, and a table that closed everything would be
    // the failure this item exists to prevent.
    if !receipts
        .iter()
        .any(|receipt| receipt.state != "resolved-for-claim")
    {
        return Err(Error::new(
            "every thread is resolved, which has never been true of this design",
        ));
    }
    Ok(())
}

fn render(receipts: &[Receipt]) -> String {
    let mut out = String::from(
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n\n\
         # Resolution receipts\n\n\
         Generated by `./generate.sh resolution-receipts` from\n\
         `resolution-receipts-source.json`. One receipt per thread where the book\n\
         identifies a defect, claims a repair, or uses a failure as a witness.\n\n\
         Each ends in exactly one of five states. **Naming a limitation and moving\n\
         on is not one of them.** `resolved-for-claim` closes only the narrower\n\
         claim its rerun establishes; the other four close nothing, and the report\n\
         gives disclosure no credit for closure.\n\n",
    );
    for state in STATES {
        let rows: Vec<_> = receipts
            .iter()
            .filter(|receipt| receipt.state == state)
            .collect();
        if rows.is_empty() {
            continue;
        }
        let _ = writeln!(out, "## {state}\n");
        for receipt in rows {
            let _ = writeln!(
                out,
                "### {} — {}\n\n\
                 *Told in* `{}`, \"{}\".\n\n\
                 **What failed.** {}\n\n\
                 **What changed.** {}\n\n\
                 **How the former attack is rerun.** `{}`\n\n\
                 **What still does not follow.** {}\n\n\
                 **What remains external or open.** {}\n",
                receipt.id,
                receipt.state,
                receipt.chapter.trim_start_matches("book-1/"),
                receipt.section,
                receipt.what_failed,
                receipt.what_changed,
                receipt.rerun,
                receipt.still_not_established,
                receipt.external_or_open
            );
        }
    }
    out
}

pub(crate) fn generate(context: &Context, _export: &mut Export) -> Result<(), Error> {
    let receipts = receipts(context)?;
    validate(context, &receipts)?;
    std::fs::write(context.path(REPORT), render(&receipts))?;
    println!("resolution receipts: {} threads", receipts.len());
    Ok(())
}

#[cfg(test)]
#[path = "resolution_receipts_tests.rs"]
mod tests;
