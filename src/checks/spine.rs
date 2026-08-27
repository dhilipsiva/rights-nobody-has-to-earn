// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use regex::Regex;

use crate::cli::Error;
use crate::context::Context;
use crate::pin::{self, LoadedSource};

const BEGIN: &str = "<!-- BEGIN GENERATED: stratification -->";
const END: &str = "<!-- END GENERATED: stratification -->";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Edge {
    predicate: String,
    negative: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Stratum {
    level: usize,
    base: bool,
    edges: Vec<Edge>,
}

type Strata = BTreeMap<String, Stratum>;

pub(crate) fn run(
    context: &Context,
    kb_relative: &Path,
    document_relative: &Path,
    check: bool,
) -> Result<String, Error> {
    let source = context.read(kb_relative)?;
    let kb_name = kb_relative.to_string_lossy();
    let strata = pin::dump_strata(&[LoadedSource::new(&kb_name, &source)]);
    if strata.exit_code != pin::EXIT_OK {
        return Err(Error::new(format!(
            "5-spine-gen: in-process strata failed:\n{}{}",
            strata.stdout, strata.stderr
        )));
    }
    if let Some(cache) = std::env::var_os("NIBLI_STRATA_CACHE_OUT") {
        std::fs::write(&cache, &strata.stdout).map_err(|error| {
            Error::new(format!(
                "5-spine-gen: cannot write strata cache {}: {error}",
                Path::new(&cache).display()
            ))
        })?;
    }
    run_with_strata(
        context,
        kb_relative,
        document_relative,
        &strata.stdout,
        check,
    )
}

pub(crate) fn run_with_strata(
    context: &Context,
    kb_relative: &Path,
    document_relative: &Path,
    strata_text: &str,
    check: bool,
) -> Result<String, Error> {
    let document = context.path(document_relative);
    let strata = parse_strata(&strata_text)?;
    let source = context.read(kb_relative)?;
    let body = render(&source, &strata)?;
    let old = std::fs::read_to_string(&document)?;
    let new = replace_region(&old, &body).ok_or_else(|| {
        Error::new(format!(
            "{}: no generated region — add the BEGIN/END markers first",
            document.display()
        ))
    })?;

    if check {
        if new != old {
            return Err(Error::new(format!(
                "{} is STALE — rerun without --check",
                document_relative.display()
            )));
        }
        return Ok(format!("{} is current", document_relative.display()));
    }

    if new == old {
        return Ok(format!("{}: already current", document_relative.display()));
    }
    std::fs::write(&document, new)?;
    Ok(format!("{}: regenerated", document_relative.display()))
}

fn parse_strata(text: &str) -> Result<Strata, Error> {
    let mut result = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let level = fields[1].parse::<usize>().map_err(|error| {
            Error::new(format!("invalid engine stratum {:?}: {error}", fields[1]))
        })?;
        let edges = fields
            .get(3)
            .into_iter()
            .flat_map(|field| field.split(','))
            .map(str::trim)
            .filter(|edge| !edge.is_empty())
            .map(|edge| {
                let (polarity, predicate) = edge.split_at(1);
                Edge {
                    predicate: predicate.to_owned(),
                    negative: polarity == "-",
                }
            })
            .collect();
        result.insert(
            fields[0].to_owned(),
            Stratum {
                level,
                base: fields[2] == "base",
                edges,
            },
        );
    }
    if result.is_empty() {
        return Err(Error::new(
            "5-spine-gen: nibli-pin --strata produced no rows",
        ));
    }
    Ok(result)
}

fn text_facts(source: &str) -> Result<(Vec<String>, String, usize), Error> {
    let floor_pattern =
        Regex::new(r"^\s*([a-z_]+)\(\s*every\s+([a-z_]+)\s*,\s*event\s*\{\s*([a-z_]+)\s*\(")
            .expect("constant floor regex is valid");
    let every_pattern =
        Regex::new(r"\bevery\s+[a-z_]+").expect("constant universal regex is valid");
    let mut floor = Vec::new();
    let mut domain = None;
    let mut rules = 0;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.contains("->") {
            rules += 1;
            continue;
        }
        let without_period = line.strip_suffix('.').unwrap_or(line);
        if let Some(captures) = floor_pattern.captures(without_period) {
            domain = Some(captures[2].to_owned());
            floor.push(captures[3].to_owned());
            rules += 1;
        } else if every_pattern.is_match(line) {
            rules += 1;
        }
    }
    let domain = domain.ok_or_else(|| Error::new("constitution has no floor domain"))?;
    if floor.is_empty() {
        return Err(Error::new("constitution has no floor rights"));
    }
    Ok((floor, domain, rules))
}

fn render(source: &str, graph: &Strata) -> Result<String, Error> {
    let (floor, domain, rules) = text_facts(source)?;
    let real: Strata = graph
        .iter()
        .filter(|(name, _)| !is_artifact(name))
        .map(|(name, row)| (name.clone(), row.clone()))
        .collect();
    let base: BTreeSet<_> = real
        .iter()
        .filter(|(_, row)| row.base)
        .map(|(name, _)| name.clone())
        .collect();
    let mut by_level: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (name, row) in &real {
        by_level.entry(row.level).or_default().push(name.clone());
    }
    let evidence: Vec<_> = base
        .iter()
        .filter(|name| name.as_str() != "equals")
        .cloned()
        .collect();
    let max_level = real
        .values()
        .map(|row| row.level)
        .max()
        .ok_or_else(|| Error::new("engine graph has no non-artifact predicates"))?;
    let floor_level = real
        .get(&floor[0])
        .ok_or_else(|| Error::new(format!("floor predicate {} absent from strata", floor[0])))?
        .level;

    let mut output = vec![
        "| measurement | predicates | derived | rules | strata |".to_owned(),
        "|---|---|---|---|---|".to_owned(),
        format!(
            "| computed from the constitution | **{}** | **{}** | **{rules}** | **{}** |",
            real.len(),
            real.len() - base.len(),
            max_level + 1
        ),
        String::new(),
        format!(
            "The floor is **{}** rights — `{}` — each derived from `{domain}`, which is why they sit at stratum {floor_level} rather than 0. That is the firewall: being inside the `{domain}` cone is what makes a punishing rule a negative cycle.",
            floor.len(),
            floor.join("`, `")
        ),
        String::new(),
        "| Stratum | Predicates |".to_owned(),
        "|---|---|".to_owned(),
    ];
    for (level, names) in by_level {
        let marked = names
            .into_iter()
            .map(|name| {
                if floor.contains(&name) {
                    format!("**{name}**")
                } else if !base.contains(&name) && cone_monotone(&name, graph, &mut HashSet::new())
                {
                    format!("`{name}` *(monotone cone)*")
                } else {
                    format!("`{name}`")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        output.push(format!("| **{level}** | {marked} |"));
    }
    output.extend([
        String::new(),
        format!(
            "Evidence predicates ({}), the complete list of what the world may report: `{}`.",
            evidence.len(),
            evidence.join("`, `")
        ),
        String::new(),
        "Strata, base/derived and edge polarity are the engine's, via `nibli-pin --strata`. Two filters are this document's choice and are named so they are visible: the compiler artifacts `event` and `__abs_<hash>` are dropped, and `equals` — which exists because `~($a = $b)` is a real negative edge — counts as a predicate but is not evidence, since nobody writes it.".to_owned(),
    ]);
    Ok(output.join("\n"))
}

fn cone_monotone(predicate: &str, graph: &Strata, seen: &mut HashSet<String>) -> bool {
    if !seen.insert(predicate.to_owned()) {
        return true;
    }
    graph.get(predicate).is_none_or(|row| {
        row.edges
            .iter()
            .all(|edge| !edge.negative && cone_monotone(&edge.predicate, graph, seen))
    })
}

fn is_artifact(name: &str) -> bool {
    name == "event" || name.starts_with("__abs_")
}

fn replace_region(source: &str, body: &str) -> Option<String> {
    let begin = source.find(BEGIN)?;
    let content_start = begin + BEGIN.len();
    let end_offset = source[content_start..].find(END)?;
    let end = content_start + end_offset;
    Some(format!(
        "{}{}\n{}\n{}",
        &source[..begin],
        BEGIN,
        body,
        &source[end..]
    ))
}

#[cfg(test)]
mod tests {
    use super::{BEGIN, END, parse_strata, replace_region, text_facts};

    #[test]
    fn parses_engine_tsv_and_edge_polarity() {
        let graph =
            parse_strata("# header\neats\t2\tderived\t+person,-equals\nperson\t0\tbase\t\n")
                .unwrap();
        assert_eq!(graph["eats"].level, 2);
        assert!(!graph["eats"].base);
        assert!(!graph["eats"].edges[0].negative);
        assert!(graph["eats"].edges[1].negative);
    }

    #[test]
    fn counts_arrow_bare_every_and_floor_rules() {
        let source = "entitled(every person, event { eats() }).\n\
                      owe(State, Provision, every person).\n\
                      all $x: person($x) -> known($x).\n";
        let (floor, domain, rules) = text_facts(source).unwrap();
        assert_eq!(floor, ["eats"]);
        assert_eq!(domain, "person");
        assert_eq!(rules, 3);
    }

    #[test]
    fn replaces_only_generated_region() {
        let source = format!("before\n{BEGIN}\nold\n{END}\nafter\n");
        let replaced = replace_region(&source, "new").unwrap();
        assert_eq!(replaced, format!("before\n{BEGIN}\nnew\n{END}\nafter\n"));
    }
}
