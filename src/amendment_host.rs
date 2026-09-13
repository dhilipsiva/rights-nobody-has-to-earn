// SPDX-License-Identifier: MIT OR Apache-2.0

//! Single-process, in-memory enactment reference model over trusted input.
//! This is NOT a signature verifier, public registry or deployment service.
//! Nothing here writes source files or changes the book verification workflow.

use nibli_engine::EngineQueryResult;
use nibli_session::CoreSession;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub id: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub query: String,
    pub before: bool,
    pub after: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum VocabularyReview {
    Unchanged,
    Reviewed {
        admitted: Vec<String>,
        derived_only: Vec<String>,
    },
}

/// An adapter must authenticate this entire object, including both byte strings,
/// all effect expectations and the evidence's authority, before real-world use.
/// The local harness deliberately supplies it as a trust-root/test input.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub record: String,
    pub transition: String,
    pub binder: String,
    pub effect_review: String,
    pub base: Version,
    pub candidate: Version,
    pub vocabulary: VocabularyReview,
    pub effects: Vec<Effect>,
    pub evidence: String,
}

#[derive(Clone)]
struct Certified {
    review: Review,
    generation: u64,
    publication: Option<String>,
}

/// A session lease, not a reusable unguarded reasoner. Even rollback to identical
/// bytes must not revive a lease from an earlier selection generation.
#[derive(Clone, Debug)]
pub struct Lease {
    version: Version,
    generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Transition {
    pub id: String,
    pub record: String,
    pub predecessor: String,
    pub successor: String,
    pub generation: u64,
    pub evidence: String,
    pub review: Review,
}

pub struct Host {
    versions: BTreeMap<String, Version>,
    candidates: BTreeMap<String, Certified>,
    effective: Version,
    generation: u64,
    history: Vec<Transition>,
    jurisdiction: String,
    slot: String,
}

fn symbol(value: &str) -> Result<()> {
    if value
        .as_bytes()
        .first()
        .is_none_or(|b| !b.is_ascii_uppercase())
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(format!("invalid record identifier: {value:?}"));
    }
    Ok(())
}

fn evidence_lines(text: &str) -> Result<Vec<&str>> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| Regex::new(
        r"^(?:authorized\([A-Z][A-Za-z0-9_]*, [A-Z][A-Za-z0-9_]*, [A-Z][A-Za-z0-9_]*\)|observe\([A-Z][A-Za-z0-9_]*, [A-Z][A-Za-z0-9_]*, [A-Z][A-Za-z0-9_]*, [A-Z][A-Za-z0-9_]*\))\.$"
    ).unwrap());
    let lines: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect();
    if lines.iter().any(|s| !pattern.is_match(s)) {
        return Err("evidence must contain only ground authorized/3 and observe/4 facts".into());
    }
    Ok(lines)
}

fn fresh(source: &str, evidence: &str) -> Result<CoreSession> {
    let mut lines: Vec<_> = source
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect();
    lines.extend(evidence_lines(evidence)?);
    let (session, _) = CoreSession::from_text_batch(&lines).map_err(|e| e.to_string())?;
    scan(&session)?;
    Ok(session)
}

fn scan(session: &CoreSession) -> Result<()> {
    let scan = session.kb().check_contradictions_report();
    if !scan.violations.is_empty() || !scan.unresolved.is_empty() {
        return Err(format!(
            "contradiction check failed or incomplete: {scan:?}"
        ));
    }
    Ok(())
}

fn expect(session: &CoreSession, query: &str, expected: bool) -> Result<()> {
    let verdict = session.query_text(query).map_err(|e| e.to_string())?;
    compare_verdict(&verdict, query, expected)
}

fn compare_verdict(verdict: &EngineQueryResult, query: &str, expected: bool) -> Result<()> {
    if !verdict.is_definitive() || verdict.is_true() != expected {
        return Err(format!(
            "bounded check {query:?}: expected {expected}, got {}",
            verdict.status_label()
        ));
    }
    Ok(())
}

fn vocabulary(session: &CoreSession) -> (Vec<String>, Vec<String>) {
    (
        session.kb().admitted_relations(),
        session.kb().derived_only_relations(),
    )
}

impl Host {
    /// The initial effective source is supplied, not discovered or authenticated.
    pub fn new(initial: Version, jurisdiction: String, slot: String) -> Result<Self> {
        symbol(&initial.id)?;
        symbol(&jurisdiction)?;
        symbol(&slot)?;
        fresh(&initial.source, "")?;
        Ok(Self {
            versions: [(initial.id.clone(), initial.clone())].into(),
            candidates: BTreeMap::new(),
            effective: initial,
            generation: 0,
            history: Vec::new(),
            jurisdiction,
            slot,
        })
    }

    pub fn effective(&self) -> &Version {
        &self.effective
    }
    pub fn history(&self) -> &[Transition] {
        &self.history
    }
    pub fn version(&self, id: &str) -> Option<&Version> {
        self.versions.get(id)
    }

    fn validate_review(&self, review: &Review, submitted: &Version) -> Result<()> {
        for value in [
            &review.record,
            &review.transition,
            &review.binder,
            &review.effect_review,
            &review.base.id,
            &review.candidate.id,
        ] {
            symbol(value)?;
        }
        if review.base != self.effective {
            return Err("stale or byte-mismatched effective base".into());
        }
        if &review.candidate != submitted {
            return Err("candidate differs from reviewed bytes or version".into());
        }
        if review.candidate.id == review.base.id {
            return Err("successor needs a fresh version occurrence".into());
        }
        if review.effects.is_empty() {
            return Err("bounded effect review is empty".into());
        }
        let base = fresh(&review.base.source, &review.evidence)?;
        // Interpret permission under the effective BASE, never the candidate's
        // self-authored rules. The candidate cannot authorize its own adoption.
        expect(
            &base,
            &format!(
                "complete({}, AmendmentCertifiedCandidate, {}).",
                review.record, review.candidate.id
            ),
            true,
        )?;
        expect(
            &base,
            &format!(
                "authorized({}, AmendmentSourceBindingAuthority, {}).",
                review.binder, review.record
            ),
            true,
        )?;
        for (value, scope) in [
            (review.base.id.as_str(), "AmendmentBaseScope"),
            (review.candidate.id.as_str(), "AmendmentCandidateScope"),
            (review.transition.as_str(), "AmendmentTransitionScope"),
            (review.effect_review.as_str(), "AmendmentEffectReviewScope"),
            (self.jurisdiction.as_str(), "AmendmentJurisdictionScope"),
            (
                match review.vocabulary {
                    VocabularyReview::Unchanged => "UnchangedEvidenceVocabulary",
                    _ => "ExplicitlyReviewedVocabularyChange",
                },
                "AmendmentVocabularyDispositionScope",
            ),
        ] {
            expect(
                &base,
                &format!(
                    "observe({}, {}, {value}, {scope}).",
                    review.binder, review.record
                ),
                true,
            )?;
        }
        let candidate = fresh(&review.candidate.source, "")?;
        let actual = vocabulary(&candidate);
        let expected = match &review.vocabulary {
            VocabularyReview::Unchanged => vocabulary(&base),
            VocabularyReview::Reviewed {
                admitted,
                derived_only,
            } => {
                let normalize = |names: &[String]| -> Result<Vec<String>> {
                    let unique: BTreeSet<_> = names.iter().cloned().collect();
                    if unique.len() != names.len() {
                        return Err("duplicate reviewed vocabulary name".into());
                    }
                    Ok(unique.into_iter().collect())
                };
                (normalize(admitted)?, normalize(derived_only)?)
            }
        };
        if actual != expected {
            return Err(
                "candidate admission or derived-only vocabulary differs from review".into(),
            );
        }
        // Review fixture facts never enter effect queries. Effects describe the
        // source itself, not a source plus a conveniently chosen permission record.
        let source_base = fresh(&review.base.source, "")?;
        for effect in &review.effects {
            expect(&source_base, &effect.query, effect.before)?;
            expect(&candidate, &effect.query, effect.after)?;
        }
        Ok(())
    }

    pub fn certify(&mut self, review: Review, submitted: Version) -> Result<()> {
        if self.candidates.contains_key(&review.transition)
            || self.history.iter().any(|h| h.id == review.transition)
        {
            return Err("replayed or divergent transition identifier".into());
        }
        if self.versions.contains_key(&submitted.id)
            || self
                .candidates
                .values()
                .any(|c| c.review.candidate.id == submitted.id || c.review.record == review.record)
            || self.history.iter().any(|h| h.record == review.record)
        {
            return Err("reused version or certificate record identifier".into());
        }
        self.validate_review(&review, &submitted)?;
        self.candidates.insert(
            review.transition.clone(),
            Certified {
                review,
                generation: self.generation,
                publication: None,
            },
        );
        Ok(())
    }

    /// Models checked publication evidence; it performs no public publication.
    pub fn publish(&mut self, transition: &str, bytes: &str, evidence: &str) -> Result<()> {
        let item = self.candidates.get(transition).ok_or("unknown candidate")?;
        if item.publication.is_some() {
            return Err("publication already recorded".into());
        }
        if bytes != item.review.candidate.source {
            return Err("publication byte mismatch".into());
        }
        if item.review.base != self.effective || item.generation != self.generation {
            return Err("stale base at publication".into());
        }
        let combined = format!("{}\n{evidence}", item.review.evidence);
        let session = fresh(&self.effective.source, &combined)?;
        expect(
            &session,
            &format!(
                "complete({}, AmendmentPublishedCandidate, {}).",
                item.review.record, item.review.candidate.id
            ),
            true,
        )?;
        self.candidates.get_mut(transition).unwrap().publication = Some(combined);
        Ok(())
    }

    /// Atomic only inside this exclusively borrowed Host, not across processes.
    /// Every check precedes the state change. Rollback uses this same path with
    /// new authority and a new occurrence ID, retaining all previous versions.
    pub fn select(&mut self, transition: &str, selector: &str, evidence: &str) -> Result<()> {
        symbol(selector)?;
        let item = self
            .candidates
            .get(transition)
            .ok_or("unknown or consumed candidate")?;
        if item.review.base != self.effective || item.generation != self.generation {
            return Err("stale base at effective selection".into());
        }
        let published = item
            .publication
            .as_ref()
            .ok_or("candidate is not published")?;
        let combined = format!("{published}\n{evidence}");
        let session = fresh(&self.effective.source, &combined)?;
        expect(
            &session,
            &format!(
                "complete({}, AmendmentEffectiveVersion, {}).",
                item.review.record, item.review.candidate.id
            ),
            true,
        )?;
        let next = self
            .generation
            .checked_add(1)
            .ok_or("selection generation exhausted")?;
        expect(
            &session,
            &format!(
                "authorized({selector}, AmendmentEffectiveSelectionAuthority, {}) & observe({selector}, {}, {}, AmendmentEffectiveSlotScope) & observe({selector}, {}, {}, AmendmentSelectedJurisdictionScope) & observe({selector}, {}, Generation{next}, AmendmentSelectionGenerationScope).",
                item.review.record,
                item.review.record,
                self.slot,
                item.review.record,
                self.jurisdiction,
                item.review.record
            ),
            true,
        )?;
        let chosen = item.review.candidate.clone();
        let history = Transition {
            id: transition.to_owned(),
            record: item.review.record.clone(),
            predecessor: self.effective.id.clone(),
            successor: chosen.id.clone(),
            generation: next,
            evidence: combined,
            review: item.review.clone(),
        };
        // Load the exact selected source afresh before committing the pointer.
        // A session carrying the old rules is never reused for the new version.
        fresh(&chosen.source, "")?;
        self.versions.insert(chosen.id.clone(), chosen.clone());
        self.effective = chosen;
        self.generation = next;
        self.history.push(history);
        self.candidates.remove(transition);
        Ok(())
    }

    pub fn lease(&self) -> Lease {
        Lease {
            version: self.effective.clone(),
            generation: self.generation,
        }
    }

    pub fn query(&self, lease: &Lease, supplied: &Version, query: &str) -> Result<bool> {
        if lease.generation != self.generation
            || lease.version != self.effective
            || supplied != &self.effective
        {
            return Err("query uses a stale lease or the wrong source bytes/version".into());
        }
        let session = fresh(&supplied.source, "")?;
        let verdict = session.query_text(query).map_err(|e| e.to_string())?;
        if !verdict.is_definitive() {
            return Err(format!(
                "query is not definitive: {}",
                verdict.status_label()
            ));
        }
        Ok(verdict.is_true())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub initial: Version,
    pub jurisdiction: String,
    pub slot: String,
    pub events: Vec<Event>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Event {
    Certify {
        review: Box<Review>,
        submitted: Version,
    },
    Publish {
        transition: String,
        source: String,
        evidence: String,
    },
    Select {
        transition: String,
        selector: String,
        evidence: String,
    },
    Query {
        version: Version,
        query: String,
        expected: bool,
    },
}

pub fn replay(scenario: Scenario) -> Result<Host> {
    let mut host = Host::new(scenario.initial, scenario.jurisdiction, scenario.slot)?;
    for (index, event) in scenario.events.into_iter().enumerate() {
        let result = match event {
            Event::Certify { review, submitted } => host.certify(*review, submitted),
            Event::Publish {
                transition,
                source,
                evidence,
            } => host.publish(&transition, &source, &evidence),
            Event::Select {
                transition,
                selector,
                evidence,
            } => host.select(&transition, &selector, &evidence),
            Event::Query {
                version,
                query,
                expected,
            } => host
                .query(&host.lease(), &version, &query)
                .and_then(|actual| {
                    if actual == expected {
                        Ok(())
                    } else {
                        Err("effective-source query mismatch".into())
                    }
                }),
        };
        result.map_err(|e| format!("event {}: {e}", index + 1))?;
    }
    Ok(host)
}

#[cfg(test)]
#[path = "amendment_host_tests.rs"]
mod tests;
