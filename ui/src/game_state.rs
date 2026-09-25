// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::data::{Outcome, cases};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
#[derive(Clone, Deserialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub role: String,
}
#[derive(Clone, Deserialize)]
pub struct Lens {
    pub id: String,
    pub glyph: String,
    pub label: String,
}
#[derive(Clone, Deserialize)]
pub struct Query {
    pub text: String,
    pub label: String,
    pub kind: String,
}
#[derive(Clone, Deserialize)]
pub struct Move {
    pub id: String,
    #[serde(rename = "move")]
    pub label: String,
    pub queries: Vec<Query>,
}
#[derive(Clone, Deserialize)]
pub struct Cost {
    pub title: String,
    pub text: String,
}
#[derive(Clone, Deserialize)]
pub struct Fork {
    pub id: String,
    pub person: String,
    pub chapter: u8,
    pub title: String,
    pub role: String,
    pub lens: String,
    pub source: String,
    pub steps: Vec<Move>,
    pub cost: Cost,
    pub faults: Vec<String>,
}
#[derive(Clone, Deserialize)]
pub struct Joint {
    pub id: String,
    pub chapter: u8,
    pub title: String,
    pub initial: bool,
    #[serde(rename = "offLabel")]
    pub off_label: String,
    pub measured: bool,
    pub source: String,
    pub queries: Vec<Query>,
    pub cost: Cost,
    pub faults: Vec<String>,
}
#[derive(Clone, Deserialize)]
pub struct Fault {
    pub id: String,
    pub title: String,
    pub text: String,
    pub book: String,
    pub kind: String,
}
#[derive(Deserialize)]
pub struct GameData {
    pub people: Vec<Person>,
    pub lenses: Vec<Lens>,
    pub scenarios: Vec<Fork>,
    pub joints: Vec<Joint>,
    pub faults: Vec<Fault>,
}
pub fn game() -> &'static GameData {
    static G: OnceLock<GameData> = OnceLock::new();
    G.get_or_init(|| serde_json::from_str(include_str!("../game.json")).expect("authored game"))
}
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct History {
    pub version: u8,
    pub completed: BTreeSet<String>,
    pub joints: BTreeMap<String, bool>,
    pub measured: BTreeSet<String>,
}
impl History {
    pub fn fresh() -> Self {
        Self {
            version: 2,
            ..Default::default()
        }
    }
    pub fn sanitize(mut self) -> Self {
        if self.version != 2 {
            return Self::fresh();
        }
        self.completed
            .retain(|id| game().scenarios.iter().any(|s| s.id == *id));
        self.measured
            .retain(|id| game().joints.iter().any(|j| j.id == *id && j.measured));
        self.joints
            .retain(|id, _| game().joints.iter().any(|j| j.id == *id));
        self
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Loading,
    Ready,
    Running,
    Complete,
    Cancelled,
    Failed,
    Incomplete,
}
impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
        }
    }
}
#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub background: bool,
    pub owner: String,
}
#[derive(Clone)]
pub struct Play {
    pub person: String,
    pub selection: String,
    pub selection_id: u64,
    pub request: u64,
    pub phase: Phase,
    pub loaded: bool,
    pub active: bool,
    pub background: bool,
    pub message: String,
    pub storage: String,
    pub history: History,
    pub displayed: Vec<Outcome>,
    pub receipts: BTreeMap<String, Vec<Outcome>>,
    pub pending: Option<Job>,
    pub restore: Vec<String>,
    pub restore_paused: bool,
    pub restoring: BTreeMap<String, Vec<Outcome>>,
}
impl Default for Play {
    fn default() -> Self {
        Self {
            person: "Nell".into(),
            selection: "nell".into(),
            selection_id: 0,
            request: 0,
            phase: Phase::Loading,
            loaded: false,
            active: false,
            background: false,
            message: "Starting the local engine and loading the full constitution…".into(),
            storage: String::new(),
            history: History::fresh(),
            displayed: vec![],
            receipts: BTreeMap::new(),
            pending: None,
            restore: vec![],
            restore_paused: false,
            restoring: BTreeMap::new(),
        }
    }
}
pub fn ids(owner: &str) -> Vec<String> {
    if let Some(s) = game().scenarios.iter().find(|s| s.id == owner) {
        return s.steps.iter().map(|s| s.id.clone()).collect();
    }
    if game().joints.iter().any(|j| j.id == owner && j.measured) {
        return vec![format!("{owner}:canonical"), format!("{owner}:modified")];
    }
    vec![]
}
/// The chapters a fork or measured joint runs: its own, and every chapter whose
/// pins its records are drawn from or refer to.
pub fn chapters_of(owner: &str, primary: u8) -> BTreeSet<u8> {
    let mut found = BTreeSet::from([primary]);
    for id in ids(owner) {
        let Some(case) = cases().cases.iter().find(|c| c.id == id) else {
            continue;
        };
        for source in &case.sources {
            let Some(rest) = source.path.strip_prefix("book-1/") else {
                continue;
            };
            if rest.as_bytes().get(2) == Some(&b'-') {
                if let Some(Ok(n)) = rest.get(..2).map(str::parse::<u8>) {
                    found.insert(n);
                }
            }
        }
    }
    found
}
pub fn valid(outcome: &Outcome, id: &str) -> bool {
    let Some(case) = cases().cases.iter().find(|c| c.id == id) else {
        return false;
    };
    outcome.id == id
        && outcome.complete
        && outcome.counterfactual == case.counterfactual
        && outcome.verdicts.len() == case.queries.len()
        && outcome.verdicts.iter().zip(&case.queries).all(|(v, q)| {
            v.query == *q && matches!(v.status.as_str(), "TRUE" | "FALSE" | "REFUSED")
        })
}
pub fn changed(results: &[Outcome]) -> bool {
    results.len() == 2
        && results[0]
            .verdicts
            .iter()
            .zip(&results[1].verdicts)
            .any(|(a, b)| {
                a.query == b.query
                    && a.status != b.status
                    && matches!(a.status.as_str(), "TRUE" | "FALSE")
                    && matches!(b.status.as_str(), "TRUE" | "FALSE")
            })
}
pub fn tag(query: &Query, status: &str) -> Option<&'static str> {
    match (query.kind.as_str(), status) {
        ("protection", "TRUE") | ("restraint", "FALSE") | ("vocabulary", "REFUSED") => Some("held"),
        ("gap", "FALSE") | ("duty_limit", "TRUE") => Some("limited"),
        ("standing_gap", "FALSE") => Some("fault"),
        _ => None,
    }
}
impl Play {
    pub fn select(&mut self, id: &str) {
        self.request += 1;
        self.selection_id += 1;
        self.active = false;
        self.pending = None;
        self.selection = id.into();
        self.displayed.clear();
        self.phase = if self.loaded {
            Phase::Ready
        } else {
            Phase::Loading
        };
        self.message = if self.loaded {
            "Choose a move to execute its complete record.".into()
        } else {
            "Loading the local engine…".into()
        };
    }
    pub fn job(&self) -> Option<Job> {
        if !self.loaded || self.active {
            return None;
        }
        if let Some(job) = &self.pending {
            return Some(job.clone());
        }
        if self.restore_paused {
            return None;
        }
        self.restore.first().and_then(|owner| {
            let count = self.restoring.get(owner).map_or(0, Vec::len);
            ids(owner).get(count).map(|id| Job {
                id: id.clone(),
                owner: owner.clone(),
                background: true,
            })
        })
    }
    pub fn accept(&mut self, job: &Job, outcome: Outcome) {
        let results = if job.background {
            self.restoring.entry(job.owner.clone()).or_default()
        } else {
            &mut self.displayed
        };
        results.push(outcome);
        if results.len() == ids(&job.owner).len() {
            let results = results.clone();
            if job.owner.starts_with("j-") {
                if changed(&results) {
                    self.history.measured.insert(job.owner.clone());
                } else {
                    self.history.measured.remove(&job.owner);
                }
            } else {
                self.history.completed.insert(job.owner.clone());
            }
            self.receipts.insert(job.owner.clone(), results);
            self.restore.retain(|s| s != &job.owner);
            self.restoring.remove(&job.owner);
        } else if !job.background && job.owner.starts_with("j-") {
            self.pending = Some(Job {
                id: format!("{}:modified", job.owner),
                ..job.clone()
            });
        }
    }
    pub fn tally(&self) -> [usize; 3] {
        let mut tally = [0; 3];
        for (id, results) in &self.receipts {
            if let Some(f) = game().scenarios.iter().find(|s| s.id == *id) {
                for (step, result) in f.steps.iter().zip(results) {
                    for query in &step.queries {
                        if let Some(v) = result.verdicts.iter().find(|v| v.query == query.text) {
                            if let Some(t) = tag(query, &v.status) {
                                tally[match t {
                                    "held" => 0,
                                    "limited" => 1,
                                    _ => 2,
                                }] += 1;
                            }
                        }
                    }
                }
            } else if changed(results) {
                tally[2] += 1;
            }
        }
        tally
    }
    pub fn unlocked(&self, id: &str) -> bool {
        game()
            .scenarios
            .iter()
            .any(|s| s.faults.iter().any(|f| f == id) && self.receipts.contains_key(&s.id))
            || game().joints.iter().any(|j| {
                j.faults.iter().any(|f| f == id)
                    && self.receipts.get(&j.id).is_some_and(|r| changed(r))
            })
    }
}
#[cfg(feature = "web")]
pub fn load_history() -> Result<History, String> {
    let w = web_sys::window().ok_or("Window unavailable")?;
    if let Ok(hash) = w.location().hash() {
        if let Some(encoded) = hash.strip_prefix("#game=") {
            let text = decode_url(encoded)?;
            return serde_json::from_str::<History>(&text)
                .map(History::sanitize)
                .map_err(|_| "Shared history is invalid; no results were accepted.".into());
        }
    }
    let storage = w
        .local_storage()
        .ok()
        .flatten()
        .ok_or("Game storage is unavailable. This visit uses memory.")?;
    let text = storage
        .get_item("b1game:v2")
        .map_err(|_| "Cannot read game storage")?;
    Ok(text
        .and_then(|s| serde_json::from_str::<History>(&s).ok())
        .unwrap_or_else(History::fresh)
        .sanitize())
}
#[cfg(feature = "desktop")]
fn game_path() -> Result<std::path::PathBuf, String> {
    if let Some(d) = dioxus::prelude::try_consume_context::<crate::PreferencesDirectory>() {
        return Ok(d.0.join("b1game-v2.json"));
    }
    directories::ProjectDirs::from("dev", "dhilipsiva", "rights-book")
        .map(|d| d.data_local_dir().join("b1game-v2.json"))
        .ok_or("Application data unavailable".into())
}
#[cfg(feature = "desktop")]
pub fn load_history() -> Result<History, String> {
    match std::fs::read_to_string(game_path()?) {
        Ok(s) => Ok(serde_json::from_str::<History>(&s)
            .unwrap_or_else(|_| History::fresh())
            .sanitize()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(History::fresh()),
        Err(e) => Err(e.to_string()),
    }
}
#[cfg(feature = "web")]
pub fn save_history(history: &History) -> Result<(), String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or("Game storage is unavailable. This visit uses memory.")?
        .set_item(
            "b1game:v2",
            &serde_json::to_string(history).map_err(|e| e.to_string())?,
        )
        .map_err(|_| "Game progress could not be saved; this visit uses memory.".into())
}
#[cfg(feature = "desktop")]
pub fn save_history(history: &History) -> Result<(), String> {
    let path = game_path()?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(
        path,
        serde_json::to_vec(history).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
#[cfg(not(any(feature = "web", feature = "desktop")))]
pub fn load_history() -> Result<History, String> {
    Ok(History::fresh())
}
#[cfg(not(any(feature = "web", feature = "desktop")))]
pub fn save_history(_: &History) -> Result<(), String> {
    Ok(())
}
pub fn encode_url(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
pub fn decode_url(text: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3).ok_or("Invalid shared link")?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid shared link")?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "Invalid shared link".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn response(id: &str) -> Outcome {
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../tests/expectations.json")).unwrap();
        let case = cases().cases.iter().find(|c| c.id == id).unwrap();
        Outcome {
            id: id.into(),
            counterfactual: case.counterfactual.clone(),
            complete: true,
            verdicts: case
                .queries
                .iter()
                .map(|q| crate::data::Verdict {
                    query: q.clone(),
                    status: expected[id]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|v| v["query"] == *q)
                        .and_then(|v| v["status"].as_str())
                        .unwrap_or("FALSE")
                        .into(),
                    detail: None,
                })
                .collect(),
        }
    }
    #[test]
    fn history_never_accepts_totals_or_verdicts() {
        assert!(serde_json::from_str::<History>(r#"{"version":2,"held":99}"#).is_err());
        assert!(serde_json::from_str::<History>(r#"{"version":2,"outcomes":[]}"#).is_err());
        let h = History {
            version: 1,
            completed: BTreeSet::from(["nell".into()]),
            ..Default::default()
        }
        .sanitize();
        assert!(h.completed.is_empty());
        let mut play = Play::default();
        play.history.completed.insert("nell".into());
        assert_eq!(play.tally(), [0, 0, 0]);
        assert!(!play.unlocked("delivery"));
    }
    #[test]
    fn nell_counts_actual_results_once_and_selection_clears_them() {
        let mut play = Play::default();
        for id in ids("nell") {
            play.accept(
                &Job {
                    id: id.clone(),
                    owner: "nell".into(),
                    background: false,
                },
                response(&id),
            );
        }
        assert_eq!(play.tally(), [3, 2, 0]);
        play.select("nell");
        assert!(play.displayed.is_empty());
        for id in ids("nell") {
            play.accept(
                &Job {
                    id: id.clone(),
                    owner: "nell".into(),
                    background: false,
                },
                response(&id),
            );
        }
        assert_eq!(play.tally(), [3, 2, 0]);
        play.select("silence");
        assert!(play.displayed.is_empty());
    }
    #[test]
    fn incomplete_missing_reordered_and_stale_responses_are_not_complete() {
        let mut o = response("nell:0");
        assert!(valid(&o, "nell:0"));
        assert!(!valid(&o, "nell:1"));
        o.complete = false;
        assert!(!valid(&o, "nell:0"));
        o.complete = true;
        o.verdicts.pop();
        assert!(!valid(&o, "nell:0"));
        o = response("nell:0");
        o.verdicts.swap(0, 1);
        assert!(!valid(&o, "nell:0"));
        o = response("nell:0");
        o.verdicts[0].status = "INCOMPLETE".into();
        assert!(!valid(&o, "nell:0"));
        o = response("nell:0");
        o.verdicts[0].status = "FALSE".into();
        assert!(
            valid(&o, "nell:0"),
            "Unexpected definitive answers stay actual answers"
        );
    }
    #[test]
    fn joint_fault_needs_two_live_definitive_results() {
        let a = response("j-independence:canonical");
        let b = response("j-independence:modified");
        assert!(!changed(std::slice::from_ref(&a)));
        assert!(changed(&[a.clone(), b]));
        assert!(!changed(&[a.clone(), a]));
    }
    #[test]
    fn foreground_jobs_take_priority_and_saved_progress_starts_without_receipts() {
        let mut p = Play::default();
        p.loaded = true;
        p.restore = vec!["nell".into()];
        assert!(p.job().unwrap().background);
        p.pending = Some(Job {
            id: "silence:0".into(),
            owner: "silence".into(),
            background: false,
        });
        assert!(!p.job().unwrap().background);
        assert_eq!(p.tally(), [0, 0, 0]);
        p.active = true;
        assert!(p.job().is_none());
        assert_eq!(
            decode_url(&encode_url("தமிழ் / #game={}")).unwrap(),
            "தமிழ் / #game={}"
        );
    }
}
