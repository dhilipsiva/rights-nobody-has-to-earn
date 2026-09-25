// SPDX-License-Identifier: MIT OR Apache-2.0

//! The chapter manifest — the one statement of Book 1's reading order.
//!
//! The order is editorial, ruled 2026-09-16 (*engines before breaks*), and was
//! never computed: the stratification in `3-spine.md` is the derivation record
//! and implies no sequence. `book-1/contents.json` records the order, the parts,
//! and the rule; the filename prefix is a projection of it, and
//! `reference_integrity_tests` require prefix == position == reader-facing
//! number so the lexicographic runtime sort stays valid wherever it is
//! convenient. A `planned` entry reserves a number for a chapter not yet
//! written and has no file, so the whole final table can be ruled once and
//! each chapter can fill its slot without renumbering the rest.
//!
//! Each Part also carries its opening case (ruling D3, 2026-09-24): a short,
//! labelled, documented case that heads the Part. An opener is exempt text,
//! unnumbered and outside every derived-chapter check, and it is `planned`
//! until it is written. Part V is the last Part and holds the exempt
//! chapters, the synthesis and its companion (ruling D7).

use crate::{cli::Error, context::Context};
use serde::Deserialize;
use std::fmt::Write as _;

pub(crate) const MANIFEST: &str = "book-1/contents.json";
pub(crate) const BEGIN: &str = "<!-- BEGIN GENERATED: contents -->";
pub(crate) const END: &str = "<!-- END GENERATED: contents -->";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Contents {
    /// The ordering rule, verbatim. Absent until the tree actually follows it:
    /// stating a rule the order violates would be a claim, not a record.
    #[serde(default)]
    pub(crate) rule: Option<String>,
    pub(crate) front: Vec<String>,
    pub(crate) parts: Vec<Part>,
    pub(crate) back: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Part {
    pub(crate) title: String,
    pub(crate) opener: Opener,
    pub(crate) chapters: Vec<Chapter>,
}

/// A Part's labelled opening case. Its file, once landed, is named
/// `part-N-<slug>.md` for the Part's position N, so no numbered-chapter check
/// can mistake it for a chapter.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Opener {
    #[serde(default)]
    pub(crate) file: Option<String>,
    #[serde(default)]
    pub(crate) title: Option<String>,
    pub(crate) status: Status,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Chapter {
    pub(crate) number: usize,
    #[serde(default)]
    pub(crate) file: Option<String>,
    pub(crate) title: String,
    pub(crate) role: Role,
    #[serde(default)]
    pub(crate) group: Option<Group>,
    pub(crate) status: Status,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Role {
    Derived,
    Exempt,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Group {
    Engine,
    Break,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Status {
    Landed,
    Planned,
}

impl Chapter {
    /// The chapter's path from the repository root, if it has landed.
    pub(crate) fn path(&self) -> Option<String> {
        self.file.as_ref().map(|file| format!("book-1/{file}"))
    }

    pub(crate) fn prefix(&self) -> String {
        format!("{:02}-", self.number)
    }
}

impl Contents {
    pub(crate) fn load(context: &Context) -> Result<Self, Error> {
        let contents: Self = serde_json::from_str(&context.read(MANIFEST)?)?;
        contents.validate()?;
        Ok(contents)
    }

    /// Shape only: what the manifest says about itself. Whether the directory
    /// agrees is `reference_integrity_tests`' question.
    fn validate(&self) -> Result<(), Error> {
        let mut expected = 1;
        for chapter in self.chapters() {
            if chapter.number != expected {
                return Err(Error::new(format!(
                    "{MANIFEST}: chapter numbers must run 1, 2, 3… in order; found {} where {expected} was expected",
                    chapter.number
                )));
            }
            expected += 1;
            match (chapter.status, &chapter.file) {
                (Status::Landed, Some(file)) => {
                    if !file.starts_with(&chapter.prefix()) || !file.ends_with(".md") {
                        return Err(Error::new(format!(
                            "{MANIFEST}: chapter {} is {file}, which does not carry its own number as a two-digit prefix",
                            chapter.number
                        )));
                    }
                }
                (Status::Landed, None) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: chapter {} is landed but names no file",
                        chapter.number
                    )));
                }
                (Status::Planned, Some(_)) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: chapter {} is planned and must not name a file — a placeholder chapter would fictionalise coverage",
                        chapter.number
                    )));
                }
                (Status::Planned, None) => {}
            }
            match (chapter.role, chapter.group) {
                (Role::Derived, None) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: derived chapter {} must say whether it is an engine or a break",
                        chapter.number
                    )));
                }
                (Role::Exempt, Some(_)) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: exempt chapter {} carries a group; only derived chapters do",
                        chapter.number
                    )));
                }
                _ => {}
            }
        }
        if expected == 1 {
            return Err(Error::new(format!("{MANIFEST}: no chapters")));
        }
        for (index, part) in self.parts.iter().enumerate() {
            let opener = &part.opener;
            match (opener.status, &opener.file, &opener.title) {
                (Status::Landed, Some(file), Some(_)) => {
                    let prefix = format!("part-{}-", index + 1);
                    if !file.starts_with(&prefix) || !file.ends_with(".md") {
                        return Err(Error::new(format!(
                            "{MANIFEST}: Part {}'s opener is {file}; an opener is named {prefix}<slug>.md",
                            index + 1
                        )));
                    }
                }
                (Status::Landed, _, _) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: Part {}'s opener is landed but lacks a file or a title",
                        index + 1
                    )));
                }
                (Status::Planned, Some(_), _) => {
                    return Err(Error::new(format!(
                        "{MANIFEST}: Part {}'s opener is planned and must not name a file",
                        index + 1
                    )));
                }
                (Status::Planned, None, _) => {}
            }
        }
        let last = self.parts.last().expect("parts");
        if last.chapters.iter().any(|c| c.role == Role::Derived)
            || self.parts[..self.parts.len() - 1]
                .iter()
                .any(|part| part.chapters.iter().any(|c| c.role == Role::Exempt))
        {
            return Err(Error::new(format!(
                "{MANIFEST}: the exempt chapters form the last Part, Part V, and only it"
            )));
        }
        if !self.front.iter().any(|f| f == "00-opening-note.md") {
            return Err(Error::new(format!(
                "{MANIFEST}: the front matter must carry the opening note"
            )));
        }
        Ok(())
    }

    pub(crate) fn chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.parts.iter().flat_map(|part| part.chapters.iter())
    }

    /// Paths of the landed derived chapters, in reading order.
    pub(crate) fn derived(&self) -> Vec<String> {
        self.chapters()
            .filter(|c| c.role == Role::Derived)
            .filter_map(Chapter::path)
            .collect()
    }

    /// Paths of every landed numbered chapter — derived and exempt — in reading
    /// order. The opening note is front matter and is not among them.
    pub(crate) fn numbered(&self) -> Vec<String> {
        self.chapters().filter_map(Chapter::path).collect()
    }

    /// Part V's landed chapters, in reading order: the exempt chapters, which
    /// are the last Part. At least one has landed; the synthesis is first.
    pub(crate) fn part_v(&self) -> Result<Vec<String>, Error> {
        let exempt: Vec<String> = self
            .chapters()
            .filter(|c| c.role == Role::Exempt)
            .filter_map(Chapter::path)
            .collect();
        if exempt.is_empty() {
            return Err(Error::new(format!(
                "{MANIFEST}: Part V has no landed chapter"
            )));
        }
        Ok(exempt)
    }

    /// Landed Part openers, in reading order.
    pub(crate) fn openers(&self) -> Vec<String> {
        self.parts
            .iter()
            .filter_map(|part| part.opener.file.as_ref())
            .map(|file| format!("book-1/{file}"))
            .collect()
    }

    pub(crate) fn planned(&self) -> Vec<&Chapter> {
        self.chapters()
            .filter(|c| c.status == Status::Planned)
            .collect()
    }

    pub(crate) fn part_of(&self, path: &str) -> Option<&Part> {
        self.parts
            .iter()
            .find(|part| part.chapters.iter().any(|c| c.path().as_deref() == Some(path)))
    }

    /// The generated block `./generate.sh spine` writes into `3-spine.md`.
    pub(crate) fn render(&self) -> String {
        let mut out = String::new();
        match &self.rule {
            Some(rule) => {
                let _ = writeln!(out, "Reading order, from `{MANIFEST}`. The rule:\n\n> {rule}\n");
            }
            None => {
                let _ = writeln!(
                    out,
                    "Reading order, from `{MANIFEST}`. No ordering rule is recorded yet; the sequence is the one the filename prefixes carry.\n"
                );
            }
        }
        let _ = writeln!(
            out,
            "Front matter: {}.\n",
            self.front
                .iter()
                .map(|f| format!("`{f}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(out, "| # | Chapter | File | Role | Group | Status |");
        let _ = writeln!(out, "|---|---|---|---|---|---|");
        for (index, part) in self.parts.iter().enumerate() {
            let _ = writeln!(
                out,
                "| **Part {} — {}** | | | | | |",
                roman(index + 1),
                part.title
            );
            let _ = writeln!(
                out,
                "| — | Opening case{} | {} | exempt | — | {} |",
                part.opener
                    .title
                    .as_ref()
                    .map(|t| format!(": {t}"))
                    .unwrap_or_default(),
                part.opener
                    .file
                    .as_ref()
                    .map(|f| format!("`{f}`"))
                    .unwrap_or_else(|| "—".to_owned()),
                match part.opener.status {
                    Status::Landed => "landed",
                    Status::Planned => "planned",
                },
            );
            for chapter in &part.chapters {
                let _ = writeln!(
                    out,
                    "| {:02} | {} | {} | {} | {} | {} |",
                    chapter.number,
                    chapter.title,
                    chapter
                        .file
                        .as_ref()
                        .map(|f| format!("`{f}`"))
                        .unwrap_or_else(|| "—".to_owned()),
                    match chapter.role {
                        Role::Derived => "derived",
                        Role::Exempt => "exempt",
                    },
                    match chapter.group {
                        Some(Group::Engine) => "engine",
                        Some(Group::Break) => "break",
                        None => "—",
                    },
                    match chapter.status {
                        Status::Landed => "landed",
                        Status::Planned => "planned",
                    },
                );
            }
        }
        let _ = writeln!(
            out,
            "\nBack matter: {}.",
            self.back
                .iter()
                .map(|f| format!("`{f}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        out.trim_end().to_owned()
    }
}

/// The heading that opens a derived chapter's argument section (ruling D2,
/// 2026-09-24): `## Argument: <title>`. The label is part of the heading, so
/// every rendering — Markdown, the assembled book, a table of contents — shows
/// the reader where the chapter stops stating consequences and starts arguing.
pub(crate) const ARGUMENT: &str = "## Argument: ";

/// A derived chapter split at its argument section: the derived text, which
/// every derived-chapter check reads, and the argued text, which is empty when
/// the chapter has no argument section.
pub(crate) fn split_argument(text: &str) -> Result<(&str, &str), String> {
    let starts: Vec<usize> = text
        .match_indices('\n')
        .map(|(at, _)| at + 1)
        .chain(std::iter::once(0))
        .filter(|&at| text[at..].starts_with(ARGUMENT))
        .collect();
    match starts.as_slice() {
        [] => Ok((text, "")),
        [at] => {
            let argued = &text[*at..];
            if argued.lines().skip(1).any(|line| line.starts_with("## ")) {
                return Err(
                    "the argument section must be the chapter's last `## ` section".to_owned(),
                );
            }
            Ok((&text[..*at], argued))
        }
        _ => Err(format!(
            "a derived chapter carries one argument section, and this one has {}",
            starts.len()
        )),
    }
}

fn roman(n: usize) -> String {
    const NUMERALS: [(usize, &str); 7] = [
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
        (0, ""),
        (0, ""),
    ];
    let mut rest = n;
    let mut out = String::new();
    for (value, numeral) in NUMERALS {
        if value == 0 {
            break;
        }
        while rest >= value {
            out.push_str(numeral);
            rest -= value;
        }
    }
    out
}

/// Replace the body between a BEGIN/END marker pair, keeping both markers.
pub(crate) fn replace_region(source: &str, begin: &str, end: &str, body: &str) -> Option<String> {
    let start = source.find(begin)?;
    let content_start = start + begin.len();
    let end_offset = source[content_start..].find(end)?;
    let stop = content_start + end_offset;
    Some(format!(
        "{}{}\n{}\n{}",
        &source[..start],
        begin,
        body,
        &source[stop..]
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn roman_numerals_cover_the_parts_a_book_can_have() {
        assert_eq!(super::roman(1), "I");
        assert_eq!(super::roman(4), "IV");
        assert_eq!(super::roman(5), "V");
        assert_eq!(super::roman(9), "IX");
    }

    #[test]
    fn an_argument_section_is_labelled_single_and_last() {
        use super::split_argument;
        let plain = "# A\n\nDerived.\n\n## Rule\n\nMore.\n";
        assert_eq!(split_argument(plain), Ok((plain, "")));
        let argued = "# A\n\nDerived.\n\n## Argument: why\n\nIn 1996, I argue.\n\n### A case\n\nMore.\n";
        let (derived, argument) = split_argument(argued).expect("one last argument section");
        assert!(derived.ends_with("Derived.\n\n") && !derived.contains("1996"));
        assert!(argument.starts_with("## Argument: why") && argument.contains("1996"));
        // Sabotage: a derived section after the argument, and a second argument.
        assert!(split_argument("# A\n\n## Argument: why\n\nI argue.\n\n## Rule\n\nMore.\n").is_err());
        assert!(split_argument("# A\n\n## Argument: one\n\n## Argument: two\n").is_err());
        // An unlabelled heading is derived text, whatever it says.
        assert_eq!(split_argument("# A\n\n## Why this rule\n\nIt.\n").map(|(_, a)| a), Ok(""));
    }
}
