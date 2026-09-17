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
    pub(crate) chapters: Vec<Chapter>,
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

    /// Part V: the landed exempt chapter. The book has exactly one.
    pub(crate) fn part_v(&self) -> Result<String, Error> {
        let exempt: Vec<String> = self
            .chapters()
            .filter(|c| c.role == Role::Exempt)
            .filter_map(Chapter::path)
            .collect();
        match exempt.as_slice() {
            [one] => Ok(one.clone()),
            _ => Err(Error::new(format!(
                "{MANIFEST}: expected exactly one landed exempt chapter (Part V), found {}",
                exempt.len()
            ))),
        }
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
}
