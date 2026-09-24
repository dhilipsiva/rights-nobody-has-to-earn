// SPDX-License-Identifier: MIT OR Apache-2.0

//! The procedural load: for every gated effect in the constitution, the roles
//! that must act before it takes effect, and the attestations that more than
//! one role repeats.
//!
//! Every conclusion that takes effect — a completed record, a permission, an
//! authority, a restraint, custody, standing, a delivery conclusion — is
//! classified in a reviewed source as beneficial, adverse, power over others,
//! oversight or institutional. The roles are read from the rules themselves, so
//! the table cannot drift from the source; a record kind nobody classified, a
//! classification naming no effect, and a role nobody mapped to a function each
//! fail. It is a list, never a score: nothing here totals or ranks a family.
//! Ruling D6 is applied from it.

use super::Export;
use crate::{cli::Error, context::Context};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

const SOURCE: &str = "book-1/source/procedural-load-source.json";
const REPORT: &str = "book-1/source/procedural-load.md";
const CONSTITUTION: &str = "book-1/source/constitution.nibli";

/// Heads that take effect. Duties (`obliged`) are owed rather than taking
/// effect, and barriers (`prevents`) read no authorised role, so neither is
/// here; the report says so.
const EFFECT_HEADS: [&str; 17] = [
    "complete",
    "permits",
    "authority",
    "restrain",
    "interrupt",
    "prisoner",
    "person",
    "eats",
    "dwell",
    "healthy",
    "secure",
    "meets",
    "insure",
    "grant",
    "provide",
    "decide",
    "agree",
];

/// Heads whose second place names what takes effect.
const KEYED: [&str; 7] = [
    "complete",
    "permits",
    "authority",
    "restrain",
    "interrupt",
    "provide",
    "grant",
];

/// The record every public power reads for the current constitutional version.
/// Reading it does not make an effect inherit its class.
const GENERIC: [&str; 1] = ["StateFormCurrent"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Source {
    pub(crate) classes: BTreeMap<String, String>,
    pub(crate) functions: Vec<(String, String)>,
    pub(crate) reasons: BTreeMap<String, String>,
    pub(crate) effects: Vec<Classified>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Classified {
    pub(crate) family: String,
    pub(crate) head: String,
    pub(crate) key: String,
    pub(crate) class: String,
    pub(crate) because: String,
}

/// Family, head relation, and what takes effect (a record kind, an action, or
/// empty for a head with no such place).
pub(crate) type Key = (String, String, String);

/// What the rules concluding one effect require, across every rule for it.
#[derive(Default)]
pub(crate) struct Effect {
    pub(crate) rules: usize,
    pub(crate) min_roles: usize,
    pub(crate) max_roles: usize,
    pub(crate) authorities: BTreeSet<String>,
    /// Groups of authorities whose holders write exactly the same fields onto
    /// the effect's own record: the same attestation made more than once.
    pub(crate) repeated: BTreeSet<Vec<String>>,
    pub(crate) reads: BTreeSet<String>,
}

pub(crate) struct Analysis {
    pub(crate) effects: BTreeMap<Key, Effect>,
    /// Class, reason key, and whether the class was inherited from a record.
    pub(crate) classes: BTreeMap<Key, (String, String, bool)>,
    pub(crate) duties: usize,
    pub(crate) barriers: usize,
}

pub(crate) fn source(context: &Context) -> Result<Source, Error> {
    Ok(serde_json::from_str(&context.read(SOURCE)?)?)
}

fn statements(text: &str) -> Vec<(String, String)> {
    let begin = Regex::new(r"^#\s*<([A-Z-]+)-RULES-BEGIN>$").expect("begin marker");
    let end = Regex::new(r"^#\s*<([A-Z-]+)-RULES-END>$").expect("end marker");
    let mut open = String::from("ARTICLES");
    let mut rows = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(caught) = begin.captures(line) {
            open = caught[1].to_owned();
            continue;
        }
        if end.is_match(line) {
            open = String::from("ARTICLES");
            continue;
        }
        if !line.is_empty() && !line.starts_with('#') {
            rows.push((open.clone(), line.to_owned()));
        }
    }
    rows
}

/// Read every gated effect and what its rules require.
pub(crate) fn measure(constitution: &str) -> (BTreeMap<Key, Effect>, usize, usize) {
    let authorized = Regex::new(r"authorized\((\$?\w+), (\w+), (\$?\w+)\)").expect("authorized");
    let observe =
        Regex::new(r"(?:^|[^~\w])observe\((\$\w+), (\$\w+), [^,()]+, (\w+)\)").expect("observe");
    let complete = Regex::new(r"(?:^|[^~\w])complete\(\$?\w+, (\w+), ").expect("complete");
    let mut effects: BTreeMap<Key, Effect> = BTreeMap::new();
    let (mut duties, mut barriers) = (0, 0);
    for (family, statement) in statements(constitution) {
        let Some(at) = statement.rfind("->") else {
            continue;
        };
        let body = &statement[..at];
        let head = statement[at + 2..].trim().trim_end_matches('.');
        let Some(open) = head.find('(') else {
            continue;
        };
        let relation = &head[..open];
        if relation == "obliged" {
            duties += 1;
            continue;
        }
        if relation == "prevents" {
            barriers += 1;
            continue;
        }
        if !EFFECT_HEADS.contains(&relation) || !body.contains("authorized(") {
            continue;
        }
        let args: Vec<&str> = head[open + 1..head.len() - 1]
            .split(',')
            .map(str::trim)
            .collect();
        if relation == "complete" && args.len() != 3 {
            continue;
        }
        // A restraint or a stay names its record second and what it restricts
        // third; every other keyed head names what takes effect second.
        let (key, record) = match relation {
            "complete" => (args[1].to_owned(), Some(args[0])),
            "restrain" | "interrupt" if args.len() == 3 => (args[2].to_owned(), Some(args[1])),
            "restrain" | "interrupt" => (String::new(), None),
            _ if KEYED.contains(&relation) && args.len() > 1 => {
                (args[1].to_owned(), args.last().copied())
            }
            _ => (String::new(), None),
        };
        let mut roles: BTreeMap<&str, &str> = BTreeMap::new();
        for caught in authorized.captures_iter(body) {
            let target = caught.get(3).expect("target").as_str();
            if record.is_none_or(|r| target == r) {
                roles.insert(
                    caught.get(1).expect("holder").as_str(),
                    caught.get(2).expect("authority").as_str(),
                );
            }
        }
        if roles.is_empty() {
            for caught in authorized.captures_iter(body) {
                roles.insert(
                    caught.get(1).expect("holder").as_str(),
                    caught.get(2).expect("authority").as_str(),
                );
            }
        }
        let mut repeated = BTreeSet::new();
        if let Some(record) = record {
            let mut writes: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
            for caught in observe.captures_iter(body) {
                if caught.get(2).expect("record").as_str() == record {
                    writes
                        .entry(caught.get(1).expect("writer").as_str())
                        .or_default()
                        .insert(caught.get(3).expect("scope").as_str());
                }
            }
            let mut groups: BTreeMap<&BTreeSet<&str>, Vec<String>> = BTreeMap::new();
            for (holder, authority) in &roles {
                if let Some(fields) = writes.get(holder) {
                    groups
                        .entry(fields)
                        .or_default()
                        .push((*authority).to_owned());
                }
            }
            for (_, mut group) in groups {
                if group.len() > 1 {
                    group.sort();
                    repeated.insert(group);
                }
            }
        }
        let effect = effects.entry((family, relation.to_owned(), key)).or_default();
        effect.min_roles = if effect.rules == 0 {
            roles.len()
        } else {
            effect.min_roles.min(roles.len())
        };
        effect.max_roles = effect.max_roles.max(roles.len());
        effect.rules += 1;
        effect
            .authorities
            .extend(roles.values().map(|a| (*a).to_owned()));
        effect.repeated.extend(repeated);
        effect.reads.extend(
            complete
                .captures_iter(body)
                .map(|c| c.get(1).expect("kind").as_str().to_owned()),
        );
    }
    (effects, duties, barriers)
}

pub(crate) fn function<'a>(source: &'a Source, authority: &str) -> Option<&'a str> {
    source
        .functions
        .iter()
        .find(|(pattern, _)| authority.contains(pattern.as_str()))
        .map(|(_, function)| function.as_str())
}

fn label((_, head, key): &Key) -> String {
    if head == "complete" {
        key.clone()
    } else if key.is_empty() {
        format!("`{head}`")
    } else {
        format!("`{head}` {key}")
    }
}

/// Classify every measured effect, and refuse anything the source does not
/// account for.
pub(crate) fn analyse(source: &Source, constitution: &str) -> Result<Analysis, Error> {
    let (effects, duties, barriers) = measure(constitution);
    let mut explicit: BTreeMap<Key, &Classified> = BTreeMap::new();
    for entry in &source.effects {
        if !source.classes.contains_key(&entry.class) {
            return Err(Error::new(format!(
                "{} {}: unknown class {}",
                entry.family, entry.key, entry.class
            )));
        }
        if !source.reasons.contains_key(&entry.because) {
            return Err(Error::new(format!(
                "{} {}: unknown reason {}",
                entry.family, entry.key, entry.because
            )));
        }
        let key = (entry.family.clone(), entry.head.clone(), entry.key.clone());
        if !effects.contains_key(&key) {
            return Err(Error::new(format!(
                "the classification of {} {} names no effect in the constitution",
                entry.family,
                label(&key)
            )));
        }
        if explicit.insert(key, entry).is_some() {
            return Err(Error::new(format!(
                "{} {} is classified twice",
                entry.family, entry.key
            )));
        }
    }
    let mut kinds: BTreeMap<&str, (&str, &str)> = BTreeMap::new();
    for (key, entry) in &explicit {
        if key.1 == "complete" {
            kinds.insert(key.2.as_str(), (entry.class.as_str(), entry.because.as_str()));
        }
    }
    let mut classes = BTreeMap::new();
    let mut unclassified = Vec::new();
    for (key, effect) in &effects {
        if let Some(entry) = explicit.get(key) {
            classes.insert(
                key.clone(),
                (entry.class.clone(), entry.because.clone(), false),
            );
            continue;
        }
        if key.1 == "complete" {
            unclassified.push(format!("{} {}", key.0, key.2));
            continue;
        }
        let specific: Vec<&String> = effect
            .reads
            .iter()
            .filter(|kind| !GENERIC.contains(&kind.as_str()))
            .collect();
        let inherited = if effect.reads.contains(&key.2) {
            kinds.get(key.2.as_str())
        } else if specific.len() == 1 {
            kinds.get(specific[0].as_str())
        } else {
            None
        };
        match inherited {
            Some((class, because)) => {
                classes.insert(key.clone(), ((*class).into(), (*because).into(), true));
            }
            None => unclassified.push(format!("{} {}", key.0, label(key))),
        }
    }
    if !unclassified.is_empty() {
        return Err(Error::new(format!(
            "unclassified effects need a class in {SOURCE}: {}",
            unclassified.join(", ")
        )));
    }
    let unmapped: BTreeSet<&String> = effects
        .values()
        .flat_map(|e| e.authorities.iter())
        .filter(|a| function(source, a).is_none())
        .collect();
    if !unmapped.is_empty() {
        return Err(Error::new(format!(
            "these roles have no function in {SOURCE}: {unmapped:?}"
        )));
    }
    Ok(Analysis {
        effects,
        classes,
        duties,
        barriers,
    })
}

/// Every completed record an effect depends on, directly or through another.
fn prerequisites(analysis: &Analysis, reads: &BTreeSet<String>) -> BTreeSet<String> {
    let records: BTreeMap<&str, &Effect> = analysis
        .effects
        .iter()
        .filter(|(key, _)| key.1 == "complete")
        .map(|(key, effect)| (key.2.as_str(), effect))
        .collect();
    let mut seen = BTreeSet::new();
    let mut todo: Vec<String> = reads.iter().cloned().collect();
    while let Some(kind) = todo.pop() {
        if seen.insert(kind.clone()) {
            if let Some(effect) = records.get(kind.as_str()) {
                todo.extend(effect.reads.iter().cloned());
            }
        }
    }
    seen
}

fn functions(source: &Source, authorities: &BTreeSet<String>) -> String {
    let set: BTreeSet<&str> = authorities
        .iter()
        .filter_map(|a| function(source, a))
        .collect();
    set.into_iter().collect::<Vec<_>>().join(", ")
}

fn repeated(source: &Source, effect: &Effect) -> String {
    effect
        .repeated
        .iter()
        .map(|group| {
            let mut names: Vec<&str> = group.iter().filter_map(|a| function(source, a)).collect();
            names.sort_unstable();
            names.join(" + ")
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("; ")
}

fn roles(effect: &Effect) -> String {
    if effect.min_roles == effect.max_roles {
        effect.max_roles.to_string()
    } else {
        format!("{}–{}", effect.min_roles, effect.max_roles)
    }
}

pub(crate) fn render(source: &Source, analysis: &Analysis) -> Result<String, Error> {
    let mut out = String::new();
    let records: BTreeMap<&str, &Effect> = analysis
        .effects
        .iter()
        .filter(|(key, _)| key.1 == "complete")
        .map(|(key, effect)| (key.2.as_str(), effect))
        .collect();
    let _ = writeln!(
        out,
        "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n\n<!-- Generated by ./generate.sh procedural-load from {SOURCE} and the constitution. Do not edit by hand. -->\n"
    );
    let _ = writeln!(out, "# The procedural load\n");
    let _ = writeln!(
        out,
        "What has to act before each effect in the constitution takes effect: every completed record, permission, authority, restraint, custody, standing and delivery conclusion that rests on an authorised role. The roles are read from the rules; the class of each effect is a reviewed judgment in the source. This is a list, not a score. It totals and ranks nothing, and a longer list is not a better or worse one.\n"
    );
    let _ = writeln!(
        out,
        "*Roles on its own record* counts the authorised roles that act on the effect's own record, with a range where its rules differ. *Repeated attestation* names roles that write exactly the same fields onto that record. *Roles across its prerequisites* counts the roles on every completed record the effect reads, directly or through another.\n"
    );
    let _ = writeln!(out, "## The classes\n");
    for (class, meaning) in &source.classes {
        let _ = writeln!(out, "- **{class}** — {meaning}");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Effects by class\n");
    let _ = writeln!(out, "| Class | Effects | Inherited from a record they read |");
    let _ = writeln!(out, "|---|---|---|");
    for class in source.classes.keys() {
        let all = analysis
            .classes
            .values()
            .filter(|(c, _, _)| c == class)
            .count();
        let inherited = analysis
            .classes
            .values()
            .filter(|(c, _, i)| c == class && *i)
            .count();
        let _ = writeln!(out, "| {class} | {all} | {inherited} |");
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Duplicated steps\n");
    let _ = writeln!(
        out,
        "Where more than one role writes exactly the same fields onto one record, the attestation is made more than once. Each line names a family, the roles that repeat it, and the effects affected, by class.\n"
    );
    let mut patterns: BTreeMap<(String, String), BTreeMap<String, usize>> = BTreeMap::new();
    for (key, effect) in &analysis.effects {
        if effect.repeated.is_empty() {
            continue;
        }
        let class = &analysis.classes[key].0;
        *patterns
            .entry((key.0.clone(), repeated(source, effect)))
            .or_default()
            .entry(class.clone())
            .or_default() += 1;
    }
    for ((family, pattern), by_class) in &patterns {
        let classes: Vec<String> = by_class
            .iter()
            .map(|(class, n)| format!("{class} {n}"))
            .collect();
        let _ = writeln!(
            out,
            "- **{family}** — {pattern}: {}",
            classes.join(", ")
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Beneficial effects\n");
    let _ = writeln!(
        out,
        "Ruling D6 lets an act that only gives or preserves something for its subject take effect on one authorised actor, with prompt independent review able to correct it. These are the effects the classification places in that class, with what each needs today.\n"
    );
    let _ = writeln!(
        out,
        "| Family | Effect | Roles on its own record | Functions | Repeated attestation |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    for (key, effect) in &analysis.effects {
        if analysis.classes[key].0 != "beneficial" {
            continue;
        }
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            key.0,
            label(key),
            roles(effect),
            functions(source, &effect.authorities),
            repeated(source, effect)
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Every effect\n");
    let families: BTreeSet<&String> = analysis.effects.keys().map(|k| &k.0).collect();
    for family in families {
        let _ = writeln!(out, "### {family}\n");
        let _ = writeln!(
            out,
            "| Effect | Class | Why | Roles on its own record | Repeated attestation | Roles across its prerequisites |"
        );
        let _ = writeln!(out, "|---|---|---|---|---|---|");
        for (key, effect) in analysis.effects.iter().filter(|(k, _)| &k.0 == family) {
            let (class, because, inherited) = &analysis.classes[key];
            let why = &source.reasons[because];
            let prerequisites = prerequisites(analysis, &effect.reads);
            let across: usize = prerequisites
                .iter()
                .filter_map(|kind| records.get(kind.as_str()))
                .map(|e| e.max_roles)
                .sum();
            let _ = writeln!(
                out,
                "| {} | {}{} | {} | {} | {} | {} |",
                label(key),
                class,
                if *inherited { " (inherited)" } else { "" },
                why,
                roles(effect),
                repeated(source, effect),
                if prerequisites.is_empty() {
                    String::from("—")
                } else {
                    format!("{across} across {} records", prerequisites.len())
                }
            );
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Outside this table\n");
    let _ = writeln!(
        out,
        "Duties are owed rather than taking effect, and barriers read no authorised role, so neither is listed: {} rules conclude a duty (`obliged`) and {} conclude a barrier (`prevents`). Where a duty accompanies a restriction, the restriction is measured through the conclusion that takes effect, such as the `interrupt` stays in the ecological family.",
        analysis.duties, analysis.barriers
    );
    Ok(out)
}

pub(crate) fn generate(context: &Context, _export: &mut Export) -> Result<(), Error> {
    let source = source(context)?;
    let analysis = analyse(&source, &context.read(CONSTITUTION)?)?;
    std::fs::write(context.path(REPORT), render(&source, &analysis)?)?;
    println!(
        "procedural load: {} effects classified",
        analysis.classes.len()
    );
    Ok(())
}

#[cfg(test)]
#[path = "procedural_load_tests.rs"]
mod tests;
