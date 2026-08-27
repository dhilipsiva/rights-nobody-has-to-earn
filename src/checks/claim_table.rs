// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fs;

use crate::cli::Error;
use crate::context::Context;

const RIGHTS_FLOOR: &str = "new-book-plans/rights-floor.pins.nibli";
const BOILERPLATE: [&str; 3] = [
    "WHAT FALSE MEANS HERE",
    "fidelity pins",
    "KIND: CONTENT pin",
];

#[derive(Debug, Eq, PartialEq)]
struct Row {
    file: String,
    line: usize,
    query: String,
    verdict: String,
    claim: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct Extraction {
    rows: Vec<Row>,
    invalid: Vec<String>,
}

pub(crate) fn print(context: &Context) -> Result<(), Error> {
    let extraction = extract_repository(context)?;
    reject_invalid(&extraction)?;
    println!("{}", render_table(&extraction.rows));
    Ok(())
}

pub(crate) fn check(context: &Context) -> Result<String, Error> {
    let extraction = extract_repository(context)?;
    reject_invalid(&extraction)?;
    Ok(format!(
        "{} queries, every one reachable from a claim comment",
        extraction.rows.len()
    ))
}

fn extract_repository(context: &Context) -> Result<Extraction, Error> {
    let book_directory = context.path("book-1");
    let mut files = fs::read_dir(&book_directory)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".pins.nibli"))
    });
    files.sort();

    let mut sources = Vec::with_capacity(files.len() + 1);
    for path in files {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::new(format!("non-UTF-8 pin filename: {}", path.display())))?;
        sources.push((format!("book-1/{name}"), fs::read_to_string(path)?));
    }
    sources.push((RIGHTS_FLOOR.to_owned(), context.read(RIGHTS_FLOOR)?));
    Ok(extract(&sources))
}

fn extract(sources: &[(String, String)]) -> Extraction {
    let mut rows = Vec::new();
    let mut invalid = Vec::new();

    for (file, source) in sources {
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let Some(raw_query) = line.strip_prefix("? ") else {
                continue;
            };
            let query = raw_query.trim_end_matches('.').trim_end().to_owned();
            let verdict = lines
                .get(index + 1)
                .and_then(|next| next.strip_prefix("# =>"))
                .map_or_else(String::new, |value| value.trim().to_owned());
            let claim = claim_for(&lines, index);
            let valid = claim.as_ref().is_some_and(|value| {
                !value.is_empty() && !BOILERPLATE.iter().any(|boiler| value.contains(boiler))
            });
            if !valid {
                invalid.push(format!("{file}:{}: ? {query}.", index + 1));
            }
            rows.push(Row {
                file: file.clone(),
                line: index + 1,
                query,
                verdict,
                claim,
            });
        }
    }

    Extraction { rows, invalid }
}

fn claim_for(lines: &[&str], query_index: usize) -> Option<String> {
    let mut cursor = query_index.checked_sub(1)?;
    loop {
        let line = lines[cursor].trim();
        if line.starts_with("# =>") {
            if cursor == 0 {
                return None;
            }
            cursor -= 1;
            continue;
        }
        if line.starts_with('#') {
            let mut block = Vec::new();
            loop {
                let candidate = lines[cursor].trim();
                if !candidate.starts_with('#') || candidate.starts_with("# =>") {
                    break;
                }
                let text = candidate.trim_start_matches('#').trim();
                if !text.is_empty() {
                    block.push(text);
                }
                if cursor == 0 {
                    break;
                }
                cursor -= 1;
            }
            block.reverse();
            return Some(block.join(" "));
        }
        if cursor == 0 {
            return None;
        }
        cursor -= 1;
    }
}

fn reject_invalid(extraction: &Extraction) -> Result<(), Error> {
    if extraction.invalid.is_empty() {
        return Ok(());
    }
    Err(Error::new(format!(
        "queries with no reachable claim comment \
(write the claim as a # comment block above):\n  {}",
        extraction.invalid.join("\n  ")
    )))
}

fn render_table(rows: &[Row]) -> String {
    let mut output = vec![
        "# Claim-to-query table".to_owned(),
        String::new(),
        "Extracted from the pin files by 6-claim-table.py; a 〃 claim continues the row above."
            .to_owned(),
    ];
    let mut last_file: Option<&str> = None;
    let mut last_claim: Option<&str> = None;

    for row in rows {
        if last_file != Some(row.file.as_str()) {
            output.extend([
                String::new(),
                format!("## {}", row.file),
                String::new(),
                "| claim (from the pin comment) | query | verdict |".to_owned(),
                "|---|---|---|".to_owned(),
            ]);
            last_file = Some(row.file.as_str());
            last_claim = None;
        }
        let claim = row.claim.as_deref().unwrap_or_default();
        let shown = if last_claim == Some(claim) {
            "〃".to_owned()
        } else {
            claim.replace('|', "\\|")
        };
        output.push(format!("| {shown} | `{}` | {} |", row.query, row.verdict));
        last_claim = Some(claim);
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{extract, reject_invalid, render_table};

    fn source(name: &str, body: &str) -> (String, String) {
        (name.to_owned(), body.to_owned())
    }

    #[test]
    fn table_matches_comment_inheritance_verdicts_and_ditto_rows() {
        let result = extract(&[
            source(
                "book-1/01-example.pins.nibli",
                "# First claim | with pipe\n# continued\nfact(A).\n? first(A).\n# => TRUE\n? second(A).\n# => FALSE\n",
            ),
            source(
                "new-book-plans/rights-floor.pins.nibli",
                "## Floor claim\n:accept-scoped\n? floor(A).\n",
            ),
        ]);
        assert!(result.invalid.is_empty());
        assert_eq!(
            render_table(&result.rows),
            "# Claim-to-query table\n\
\n\
Extracted from the pin files by 6-claim-table.py; a 〃 claim continues the row above.\n\
\n\
## book-1/01-example.pins.nibli\n\
\n\
| claim (from the pin comment) | query | verdict |\n\
|---|---|---|\n\
| First claim \\| with pipe continued | `first(A)` | TRUE |\n\
| 〃 | `second(A)` | FALSE |\n\
\n\
## new-book-plans/rights-floor.pins.nibli\n\
\n\
| claim (from the pin comment) | query | verdict |\n\
|---|---|---|\n\
| Floor claim | `floor(A)` |  |"
        );
    }

    #[test]
    fn verdict_lines_are_skipped_when_finding_the_prior_claim() {
        let result = extract(&[source(
            "book-1/example.pins.nibli",
            "# Shared claim\n? first(A).\n# => TRUE\n:accept-scoped\nfact(B).\n? second(B).\n",
        )]);
        assert!(result.invalid.is_empty());
        assert_eq!(result.rows[1].claim.as_deref(), Some("Shared claim"));
    }

    #[test]
    fn missing_and_boilerplate_claims_report_exact_locations() {
        let result = extract(&[source(
            "book-1/example.pins.nibli",
            "? missing(A).\n# KIND: CONTENT pin\n? boiler(B).\n",
        )]);
        let error = reject_invalid(&result).unwrap_err();
        assert_eq!(
            error.to_string(),
            "queries with no reachable claim comment \
(write the claim as a # comment block above):\n  \
book-1/example.pins.nibli:1: ? missing(A).\n  \
book-1/example.pins.nibli:3: ? boiler(B)."
        );
    }

    #[test]
    fn empty_comment_lines_do_not_add_spaces_to_claims() {
        let result = extract(&[source(
            "book-1/example.pins.nibli",
            "# first\n#\n### second\n? query(A).\n",
        )]);
        assert_eq!(result.rows[0].claim.as_deref(), Some("first second"));
    }

    #[test]
    fn entirely_empty_comment_block_is_not_a_claim() {
        let result = extract(&[source("book-1/example.pins.nibli", "#\n###\n? query(A).\n")]);
        assert_eq!(
            result.invalid,
            vec!["book-1/example.pins.nibli:3: ? query(A)."]
        );
    }
}
