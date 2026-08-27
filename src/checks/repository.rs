// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::cli::Error;
use crate::context::Context;

const KB: &str = "new-book-plans/constitution.nibli";
const ABSENT_READERS: [&str; 6] = ["owe", "become", "travel", "lose", "reward", "building"];
const FLOOR_RIGHTS: [&str; 8] = [
    "secure",
    "eats",
    "dwell",
    "healthy",
    "learn",
    "expresses",
    "believe",
    "meets",
];
const UNSCOPED_OK: [&str; 5] = [
    "01-what-counts-as-evidence",
    "05-voiding",
    "06-clawback",
    "09-the-vote-conviction-does-not-take",
    "14-when-the-system-notices-it-broke",
];

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Report {
    pub(crate) messages: Vec<String>,
}

pub(crate) fn check(context: &Context) -> Result<Report, Error> {
    let mut messages = check_prose(context)?;
    messages.extend(check_constitution(&context.read(KB)?)?);
    messages.extend(check_control_scopes(context)?);
    Ok(Report { messages })
}

fn check_prose(context: &Context) -> Result<Vec<String>, Error> {
    let files = numbered_chapters(context)?;
    let jargon = Regex::new(
        r"(?i)nibli|predicat|stratum|strata|stratif|compil|assert|rule head|quantif|derivation|knowledge base|first-order|negation|conjunct",
    )
    .map_err(|error| Error::new(format!("invalid jargon expression: {error}")))?;
    let counted = Regex::new(
        r"(?i)\b(eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|hundred)([- ][a-z]+)?\b|\beight\b|\bexactly one\b|\bone thing\b|\bsingle deprivation\b",
    )
    .map_err(|error| Error::new(format!("invalid counted-claim expression: {error}")))?;
    let duration = Regex::new(r"(?i)(thirty|forty|fifty|hundred) (year|second|minute|mile)")
        .map_err(|error| Error::new(format!("invalid duration expression: {error}")))?;

    let mut jargon_hits = Vec::new();
    let mut counted_hits = Vec::new();
    for relative in &files {
        let source = context.read(relative)?;
        for (index, line) in source.lines().enumerate() {
            if jargon.is_match(line) {
                jargon_hits.push(format!("{}:{}:{}", relative.display(), index + 1, line));
            }
            if counted.is_match(line)
                && !duration.is_match(line)
                && !(relative == Path::new("book-1/13-the-one-thing-taken.md")
                    && index == 0
                    && line == "# The One Thing Taken")
            {
                counted_hits.push(format!("{}:{}:{}", relative.display(), index + 1, line));
            }
        }
    }

    if !jargon_hits.is_empty() {
        return Err(Error::new(format!(
            "jargon in a derived chapter\n{}",
            jargon_hits.join("\n")
        )));
    }
    if !counted_hits.is_empty() {
        let tail = counted_hits
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(Error::new(format!(
            "counted claims in the prose: {} (the gate is zero)\n{tail}\n\
             State the rule that produces the count, do not count the instances.",
            counted_hits.len()
        )));
    }

    Ok(vec![
        format!("jargon sweep clean across {} chapters", files.len()),
        "counted claims at zero (hard gate; ch13's title is the allowlisted exception)".to_owned(),
    ])
}

fn numbered_chapters(context: &Context) -> Result<Vec<PathBuf>, Error> {
    let mut files = fs::read_dir(context.path("book-1"))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.as_bytes().first().is_some_and(u8::is_ascii_digit) && name.ends_with(".md")
            })
    });
    files.sort();
    files
        .into_iter()
        .map(|path| {
            path.strip_prefix(context.root())
                .map(Path::to_path_buf)
                .map_err(|_| Error::new(format!("chapter escaped repository: {}", path.display())))
        })
        .collect()
}

fn check_constitution(source: &str) -> Result<Vec<String>, Error> {
    let rules = rules(source);
    let false_control = body_hits(&rules, "false");
    if false_control.is_empty() {
        return Err(Error::new(
            "absence check is broken\npositive control /false/ returned 0 lines; it should return 5",
        ));
    }
    let mut messages = vec![format!(
        "positive control: /false/ appears in {} rule bodies",
        false_control.len()
    )];

    for predicate in ABSENT_READERS {
        let hits = body_hits(&rules, predicate);
        if !hits.is_empty() {
            return Err(Error::new(format!(
                "{predicate} is now read by a rule\n{} — the prose saying nothing follows from it is now false",
                render_hits(&hits)
            )));
        }
        messages.push(format!("nothing reads {predicate}"));
    }

    if source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .any(has_numeric_literal)
    {
        return Err(Error::new(
            "a numeric literal appeared in an enacted line\nchapter 10 turns on there being no arithmetic",
        ));
    }
    messages.push("no numeric literals in the constitution".to_owned());

    let mut noticed = 0;
    for rule in &rules {
        for predicate in FLOOR_RIGHTS {
            if predicate_call_with_lower_boundary(rule.body, predicate) {
                if !rule.head.contains("err(") {
                    return Err(Error::new(format!(
                        "{predicate} is read into something other than err — INVARIANT 1\n{}: {}\n\
                         A floor right may be noticed (head err) and never acted on. See constitution :103.",
                        rule.line, rule.raw
                    )));
                }
                noticed += 1;
                break;
            }
        }
    }
    if noticed == 0 {
        return Err(Error::new(
            "the noticing check is vacuous\nno floor right is read into err at all, so the guard above proves nothing — Article 6's isolation marker should be one",
        ));
    }
    messages.push(format!(
        "floor rights reach only err ({noticed} such rule, none reaching a consequence)"
    ));

    let reward_mentions = active_lines(source)
        .filter(|(_, line)| predicate_call_with_identifier_boundary(line, "reward"))
        .count();
    if reward_mentions == 0 {
        return Err(Error::new(
            "the reward guards are vacuous\nno enacted line mentions reward at all — 'nothing reads reward' above is proving nothing",
        ));
    }
    let judge_arity = arity_two_hits(source, "judge");
    if judge_arity.is_empty() {
        return Err(Error::new(
            "the arity guard is broken\npositive control /judge(_, _)/ returned 0; the multi-sig rule carries two places",
        ));
    }
    let reward_arity = arity_two_hits(source, "reward");
    if !reward_arity.is_empty() {
        return Err(Error::new(format!(
            "reward has grown a second place — provenance on recognition is refused\n{}\n\
             See CLAUDE.md: refused on grounds that hold on both sides of the Article 4 fork.",
            render_hits(&reward_arity)
        )));
    }
    messages.push(format!(
        "reward is one place wide ({reward_mentions} enacted mentions; control saw {} two-place judge)",
        judge_arity.len()
    ));

    let judge_joins = self_join_hits(&rules, "judge");
    if judge_joins.is_empty() {
        return Err(Error::new(
            "the counting guard is broken\npositive control /judge/ returned 0; the multi-sig rule joins it twice",
        ));
    }
    messages.push(format!(
        "positive control: /judge/ is joined with itself in {} rule",
        judge_joins.len()
    ));
    for (label, needle) in [("teaches", "teaches"), ("work", "work(")] {
        let hits = self_join_hits(&rules, needle);
        if !hits.is_empty() {
            return Err(Error::new(format!(
                "{label} is now joined with itself — that is counted degree on the reward side\n{} — chapter 10 says this will not be built",
                render_hits(&hits)
            )));
        }
        messages.push(format!("no rule counts {label} entries"));
    }

    Ok(messages)
}

#[derive(Clone, Copy)]
struct Rule<'a> {
    line: usize,
    raw: &'a str,
    body: &'a str,
    head: &'a str,
}

fn rules(source: &str) -> Vec<Rule<'_>> {
    active_lines(source)
        .filter_map(|(line, raw)| {
            raw.split_once("->").map(|(body, head)| Rule {
                line,
                raw,
                body,
                head,
            })
        })
        .collect()
}

fn active_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.starts_with('#'))
        .map(|(index, line)| (index + 1, line))
}

fn body_hits<'a>(rules: &'a [Rule<'a>], predicate: &str) -> Vec<(usize, &'a str)> {
    rules
        .iter()
        .filter(|rule| predicate_call_with_identifier_boundary(rule.body, predicate))
        .map(|rule| (rule.line, rule.raw))
        .collect()
}

fn predicate_call_with_identifier_boundary(text: &str, predicate: &str) -> bool {
    call_with_boundary(text, predicate, |byte| {
        !(byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

fn predicate_call_with_lower_boundary(text: &str, predicate: &str) -> bool {
    call_with_boundary(text, predicate, |byte| !byte.is_ascii_lowercase())
}

fn call_with_boundary(text: &str, predicate: &str, valid_left: impl Fn(u8) -> bool) -> bool {
    let needle = format!("{predicate}(");
    text.match_indices(&needle)
        .any(|(index, _)| index == 0 || valid_left(text.as_bytes()[index - 1]))
}

fn has_numeric_literal(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let left_ok =
            start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let right_ok =
            index == bytes.len() || !(bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_');
        if left_ok && right_ok {
            return true;
        }
    }
    false
}

fn arity_two_hits<'a>(source: &'a str, predicate: &str) -> Vec<(usize, &'a str)> {
    active_lines(source)
        .filter(|(_, line)| {
            let needle = format!("{predicate}(");
            line.match_indices(&needle).any(|(index, _)| {
                let boundary = index == 0
                    || !(line.as_bytes()[index - 1].is_ascii_lowercase()
                        || line.as_bytes()[index - 1] == b'_');
                boundary
                    && line[index + needle.len()..]
                        .split(')')
                        .next()
                        .is_some_and(|arguments| arguments.contains(','))
            })
        })
        .collect()
}

fn self_join_hits<'a>(rules: &'a [Rule<'a>], needle: &str) -> Vec<(usize, &'a str)> {
    rules
        .iter()
        .filter(|rule| rule.body.match_indices(needle).count() >= 2)
        .map(|rule| (rule.line, rule.raw))
        .collect()
}

fn render_hits(hits: &[(usize, &str)]) -> String {
    hits.iter()
        .map(|(line, text)| format!("{line}: {text}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn check_control_scopes(context: &Context) -> Result<Vec<String>, Error> {
    let mut paths = Vec::new();
    collect_pin_files(&context.path("new-book-plans"), false, &mut paths)?;
    collect_pin_files(
        &context.path("new-book-plans/counterfactual"),
        false,
        &mut paths,
    )?;
    collect_pin_files(&context.path("book-1"), false, &mut paths)?;
    paths.sort();

    let mut allowlisted = 0;
    let mut bad = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path)?;
        let count = source.lines().filter(|line| *line == ":accept").count();
        if count == 0 {
            continue;
        }
        let base = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".pins.nibli"))
            .ok_or_else(|| Error::new(format!("invalid pin filename: {}", path.display())))?;
        if UNSCOPED_OK.contains(&base) {
            allowlisted += 1;
        } else {
            let relative = path.strip_prefix(context.root()).unwrap_or(&path);
            bad.push(format!("{}: {count} unscoped", relative.display()));
        }
    }
    if allowlisted != UNSCOPED_OK.len() {
        return Err(Error::new(format!(
            "the control-scope guard is broken\nexpected every allowlisted file to carry a plain :accept; matched {allowlisted} of {}. Either the syntax moved or a premise site was converted.",
            UNSCOPED_OK.len()
        )));
    }
    if !bad.is_empty() {
        return Err(Error::new(format!(
            "a control leaves its statement in the KB — every pin below it runs against a widened base\n{}\n\
             Write these :accept-scoped, or allowlist the file above if the statement is a premise a later query needs.",
            bad.join("\n")
        )));
    }
    Ok(vec![
        "every control is written :accept-scoped (the engine, not this check, puts the base back)"
            .to_owned(),
    ])
}

fn collect_pin_files(
    directory: &Path,
    recursive: bool,
    output: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if recursive && path.is_dir() {
            collect_pin_files(&path, true, output)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".pins.nibli"))
        {
            output.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        arity_two_hits, check_constitution, has_numeric_literal,
        predicate_call_with_identifier_boundary, self_join_hits,
    };
    use crate::context::Context;

    #[test]
    fn identifier_boundaries_match_the_shell_guards() {
        assert!(predicate_call_with_identifier_boundary(
            "reward(A)",
            "reward"
        ));
        assert!(predicate_call_with_identifier_boundary(
            "~reward(A)",
            "reward"
        ));
        assert!(!predicate_call_with_identifier_boundary(
            "prereward(A)",
            "reward"
        ));
        assert!(!predicate_call_with_identifier_boundary(
            "x_reward(A)",
            "reward"
        ));
    }

    #[test]
    fn numeric_guard_ignores_digits_inside_identifiers() {
        assert!(!has_numeric_literal("source(T3LifeCourseNonborrowing)."));
        assert!(has_numeric_literal("score(Person, 3)."));
        assert!(has_numeric_literal("3 -> score(Person)."));
    }

    #[test]
    fn arity_and_self_join_scans_keep_line_locations() {
        let source = "judge(A, B) -> ok(A).\nteaches(A, B) & teaches(C, B) -> reward(B).\n";
        assert_eq!(arity_two_hits(source, "judge")[0].0, 1);
        let rules = super::rules(source);
        assert_eq!(self_join_hits(&rules, "teaches")[0].0, 2);
    }

    #[test]
    fn representative_clean_constitution_reports_all_guard_groups() {
        let source = "false(A) -> err(A, Missing).\n\
                      prisoner(A) & ~eats(A) -> err(A, Isolation).\n\
                      judge(A, B) & judge(C, B) -> signed(B).\n\
                      taught(A) -> reward(A).\n";
        let messages = check_constitution(source).unwrap();
        assert!(
            messages
                .iter()
                .any(|message| message == "nothing reads reward")
        );
        assert!(
            messages
                .iter()
                .any(|message| message.starts_with("reward is one place wide"))
        );
        assert!(
            messages
                .iter()
                .any(|message| message == "no rule counts teaches entries")
        );
    }

    #[test]
    fn current_repository_passes_native_guards() {
        let context = Context::discover().unwrap();
        let report = super::check(&context).unwrap();
        assert!(report.messages.len() >= 15);
    }
}
