// SPDX-License-Identifier: MIT OR Apache-2.0

//! The source-derived multidisciplinary adversarial audit.
//!
//! Fifteen declared lenses, each bound to the checks and cases that encode it
//! and to what it currently finds. A lens that names nothing executable is not
//! a lens, so every `encoded_by` path must exist; a lens that finds nothing is
//! not an audit, so every lens must carry at least one finding.
//!
//! This is a repository audit over the source. It warrants no independent human
//! review, no reader response, no external truth, and no operation.

use super::Export;
use crate::{cli::Error, context::Context};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const SOURCE: &str = "new-book-plans/adversarial-audit-source.json";
const REPORT: &str = "new-book-plans/adversarial-audit.md";

/// The finding kinds the audit is required to look for.
pub(crate) const CATEGORIES: [&str; 7] = [
    "omitted-domain",
    "unowned-dependency",
    "hidden-liveness-assumption",
    "private-power-blind-spot",
    "impossible-operation-overclaim",
    "totalising-rule",
    "narrative-distortion",
];

/// What an open finding does to the claims that depend on it. Disclosure is not
/// one of the values: naming a limitation and moving on is what the resolution
/// receipts refuse, and the same refusal belongs here.
pub(crate) const DISPOSITIONS: [&str; 4] = [
    // The route that would establish it is not built, so no claim may take an
    // established posture through it. Carries the claim restriction with it.
    "route-unbuilt",
    // Real, scoped, and it limits what may be said rather than blocking a gate.
    "public-claim-limited",
    // Waits on a decision that is not a session's to make.
    "author-ruling-pending",
    // Blocks the gate whose permitted claim it touches.
    "blocks-gate",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Finding {
    pub(crate) finding: String,
    pub(crate) category: String,
    pub(crate) open: bool,
    /// Required on an open finding, refused on a closed one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<String>,
    /// What this costs in what the project may claim or do. Required with a
    /// disposition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) consequence: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Lens {
    pub(crate) id: String,
    pub(crate) lens: String,
    pub(crate) encoded_by: Vec<String>,
    pub(crate) findings: Vec<Finding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    lenses: Vec<Lens>,
}

pub(crate) fn lenses(context: &Context) -> Result<Vec<Lens>, Error> {
    let source: Source = serde_json::from_str(&context.read(SOURCE)?)?;
    Ok(source.lenses)
}

pub(crate) fn validate(context: &Context, lenses: &[Lens]) -> Result<(), Error> {
    let mut seen = BTreeSet::new();
    for lens in lenses {
        if !seen.insert(&lens.id) {
            return Err(Error::new(format!("duplicate lens {}", lens.id)));
        }
        if lens.encoded_by.is_empty() {
            return Err(Error::new(format!(
                "{}: a lens that names nothing executable is not a lens",
                lens.id
            )));
        }
        for path in &lens.encoded_by {
            if !context.path(path).exists() {
                return Err(Error::new(format!(
                    "{}: {path} does not exist, so this lens encodes nothing",
                    lens.id
                )));
            }
        }
        if lens.findings.is_empty() {
            return Err(Error::new(format!(
                "{}: a lens that finds nothing is not an audit",
                lens.id
            )));
        }
        for finding in &lens.findings {
            if !CATEGORIES.contains(&finding.category.as_str()) {
                return Err(Error::new(format!(
                    "{}: {} is not one of the declared finding kinds",
                    lens.id, finding.category
                )));
            }
            if finding.finding.len() < 40 {
                return Err(Error::new(format!(
                    "{}: a finding this short says nothing",
                    lens.id
                )));
            }
            // An open finding has to say what it does to the project. This is
            // the tracker's done-when made mechanical: closed, narrowed, or
            // carrying a limitation and a consequence.
            match (finding.open, &finding.disposition, &finding.consequence) {
                (true, None, _) | (true, _, None) => {
                    return Err(Error::new(format!(
                        "{}: an open finding needs a disposition and a \
                         consequence. Leaving it open with neither is the \
                         disclosure-as-ending move: '{}'",
                        lens.id, finding.finding
                    )));
                }
                (true, Some(disposition), _) if !DISPOSITIONS.contains(&disposition.as_str()) => {
                    return Err(Error::new(format!(
                        "{}: {disposition} is not a declared disposition",
                        lens.id
                    )));
                }
                (false, Some(_), _) | (false, _, Some(_)) => {
                    return Err(Error::new(format!(
                        "{}: a closed finding carries no disposition; if it \
                         still costs something it is not closed",
                        lens.id
                    )));
                }
                _ => {}
            }
        }
    }
    // Every declared finding kind has to be exercised, or the audit is looking
    // for fewer things than it says it looks for.
    for category in CATEGORIES {
        if !lenses
            .iter()
            .any(|lens| lens.findings.iter().any(|f| f.category == category))
        {
            return Err(Error::new(format!(
                "no lens raised a '{category}' finding, so that kind is declared \
                 and not looked for"
            )));
        }
    }
    // An audit with nothing open is an audit that stopped looking.
    if !lenses
        .iter()
        .any(|lens| lens.findings.iter().any(|finding| finding.open))
    {
        return Err(Error::new(
            "no finding is open, which has never been true of this design",
        ));
    }
    Ok(())
}

fn render(lenses: &[Lens]) -> String {
    let open: Vec<_> = lenses
        .iter()
        .flat_map(|lens| {
            lens.findings
                .iter()
                .filter(|finding| finding.open)
                .map(move |finding| (lens, finding))
        })
        .collect();
    let mut out = String::from(
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n\n\
         # Multidisciplinary adversarial audit\n\n\
         Generated by `./generate.sh adversarial-audit` from\n\
         `adversarial-audit-source.json`. Fifteen declared lenses, each bound to\n\
         the checks and cases that encode it and to what it finds. A lens naming\n\
         nothing executable fails; a lens finding nothing fails; a declared\n\
         finding kind nobody raises fails.\n\n\
         This is a repository audit over the source. It warrants no independent\n\
         human review, no reader response, no external truth and no operation.\n\
         External multidisciplinary and lived-experience submissions remain\n\
         welcome optional evidence; none is required for completion.\n\n\
         ## Open findings\n\n\
         Every one carries a disposition and what it costs. `route-unbuilt` means\n\
         the route that would establish the affected claim is neither built nor\n\
         available; `public-claim-limited` means the finding bounds what may be\n\
         said rather than blocking a gate; `author-ruling-pending` means the\n\
         decision is not a session's to make; `blocks-gate` means what it says.\n\
         Disclosure is not a disposition.\n\n\
         | Lens | Kind | Finding | Disposition | Consequence |\n| --- | --- | --- | --- | --- |\n",
    );
    for (lens, finding) in &open {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            lens.lens,
            finding.category,
            finding.finding,
            finding.disposition.as_deref().unwrap_or(""),
            finding.consequence.as_deref().unwrap_or("")
        );
    }
    out.push_str("\n## Findings by kind\n\n| Kind | Total | Open |\n| --- | ---: | ---: |\n");
    let mut totals: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for lens in lenses {
        for finding in &lens.findings {
            let entry = totals.entry(finding.category.as_str()).or_default();
            entry.0 += 1;
            if finding.open {
                entry.1 += 1;
            }
        }
    }
    for (category, (total, open)) in &totals {
        let _ = writeln!(out, "| {category} | {total} | {open} |");
    }
    out.push_str("\n## Every lens\n");
    for lens in lenses {
        let _ = writeln!(out, "\n### {}\n", lens.lens);
        out.push_str("Encoded by:\n\n");
        for path in &lens.encoded_by {
            let _ = writeln!(out, "- `{path}`");
        }
        out.push('\n');
        for finding in &lens.findings {
            let _ = writeln!(
                out,
                "- **{}** ({}) — {}",
                if finding.open { "open" } else { "held" },
                finding.category,
                finding.finding
            );
        }
    }
    out
}

pub(crate) fn generate(context: &Context, _export: &mut Export) -> Result<(), Error> {
    let lenses = lenses(context)?;
    validate(context, &lenses)?;
    std::fs::write(context.path(REPORT), render(&lenses))?;
    let open = lenses
        .iter()
        .flat_map(|lens| &lens.findings)
        .filter(|finding| finding.open)
        .count();
    println!(
        "adversarial audit: {} lenses, {} findings, {open} open",
        lenses.len(),
        lenses.iter().map(|lens| lens.findings.len()).sum::<usize>()
    );
    Ok(())
}

#[cfg(test)]
#[path = "adversarial_audit_tests.rs"]
mod tests;
