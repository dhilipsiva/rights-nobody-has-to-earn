// SPDX-License-Identifier: MIT OR Apache-2.0

//! Manual development measurements; never part of routine verification.

use super::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Mutex;

static FULL_PROFILE: AtomicBool = AtomicBool::new(false);
static FULL_RECORDS: Mutex<FullRecords> = Mutex::new(FullRecords {
    phases: BTreeMap::new(),
    cases: Vec::new(),
    slow_steps: Vec::new(),
});

#[derive(Default)]
struct FullRecords {
    phases: BTreeMap<String, (usize, Duration)>,
    cases: Vec<(String, Duration, Duration)>,
    slow_steps: Vec<(Duration, String, String)>,
}

impl FullRecords {
    fn step(&mut self, kind: &str, text: &str, elapsed: Duration, case: &str) {
        let key = if kind == "prepare" {
            format!("{kind}: {text}")
        } else {
            kind.to_owned()
        };
        let (count, total) = self.phases.entry(key).or_default();
        *count += 1;
        *total += elapsed;
        if kind != "prepare" && elapsed >= Duration::from_millis(20) {
            self.slow_steps.push((
                elapsed,
                format!("{case} / {kind}"),
                text.chars().take(120).collect(),
            ));
        }
    }
}

thread_local! {
    static TRACE_STEPS: Cell<bool> = const { Cell::new(false) };
    static PROFILE_CASE: RefCell<String> = const { RefCell::new(String::new()) };
}

pub(super) fn trace_step(kind: &str, text: &str, elapsed: Duration) {
    if FULL_PROFILE.load(Ordering::Relaxed) {
        PROFILE_CASE.with_borrow(|case| {
            FULL_RECORDS.lock().unwrap().step(kind, text, elapsed, case);
        });
    }
    if TRACE_STEPS.get() && elapsed >= Duration::from_millis(20) {
        let text = text.chars().take(180).collect::<String>();
        eprintln!("PROFILE {kind} {:.3}s {text}", elapsed.as_secs_f64());
    }
}

pub(crate) fn profile_case_started(id: &str) {
    if FULL_PROFILE.load(Ordering::Relaxed) {
        PROFILE_CASE.with_borrow_mut(|case| id.clone_into(case));
    }
}

pub(crate) fn profile_case_finished(id: &str, prepare: Duration, run: Duration) {
    if FULL_PROFILE.load(Ordering::Relaxed) {
        FULL_RECORDS
            .lock()
            .unwrap()
            .cases
            .push((id.to_owned(), prepare, run));
        PROFILE_CASE.with_borrow_mut(String::clear);
    }
}

pub(super) struct PhaseTimer(&'static str, &'static str, Instant);

impl PhaseTimer {
    pub(super) fn start(kind: &'static str, text: &'static str) -> Self {
        Self(kind, text, Instant::now())
    }
}

impl Drop for PhaseTimer {
    fn drop(&mut self) {
        trace_step(self.0, self.1, self.2.elapsed());
    }
}

#[test]
fn full_profile_accumulates_phases_without_conflating_preparation_stages() {
    let mut records = FullRecords::default();
    records.step("prepare", "copies", Duration::from_millis(2), "a");
    records.step("prepare", "copies", Duration::from_millis(3), "b");
    records.step("prepare", "model", Duration::from_millis(5), "a");
    records.step("query", "first", Duration::from_millis(30), "a");
    records.step("query", "second", Duration::from_millis(40), "b");
    assert_eq!(
        records.phases["prepare: copies"],
        (2, Duration::from_millis(5))
    );
    assert_eq!(
        records.phases["prepare: model"],
        (1, Duration::from_millis(5))
    );
    assert_eq!(records.phases["query"], (2, Duration::from_millis(70)));
    assert_eq!(records.slow_steps.len(), 2);
    assert_eq!(records.slow_steps[1].1, "b / query");
}

#[test]
#[ignore = "manual full-inventory profile, not a verification gate"]
fn profile_complete_verification() {
    *FULL_RECORDS.lock().unwrap() = FullRecords::default();
    assert!(!FULL_PROFILE.swap(true, Ordering::Relaxed));
    let result = crate::execution::run(crate::cli::Args {
        only: None,
        list: false,
    });
    FULL_PROFILE.store(false, Ordering::Relaxed);
    let mut records = std::mem::take(&mut *FULL_RECORDS.lock().unwrap());
    eprintln!("PROFILE cumulative worker seconds, not wall-clock contributions:");
    for (phase, (count, elapsed)) in &records.phases {
        eprintln!(
            "PROFILE phase {phase}: {count} calls {:.3}s",
            elapsed.as_secs_f64()
        );
    }
    let preparation: Duration = records.cases.iter().map(|row| row.1).sum();
    let execution: Duration = records.cases.iter().map(|row| row.2).sum();
    eprintln!(
        "PROFILE {} cases: preparation={:.3}s execution={:.3}s",
        records.cases.len(),
        preparation.as_secs_f64(),
        execution.as_secs_f64()
    );
    records
        .cases
        .sort_by_key(|row| std::cmp::Reverse(row.1 + row.2));
    for (id, prepare, run) in records.cases.iter().take(25) {
        eprintln!(
            "PROFILE case {id}: prepare={:.3}s run={:.3}s",
            prepare.as_secs_f64(),
            run.as_secs_f64()
        );
    }
    records
        .slow_steps
        .sort_by_key(|row| std::cmp::Reverse(row.0));
    for (elapsed, label, text) in records.slow_steps.iter().take(25) {
        eprintln!("PROFILE step {label}: {:.3}s {text}", elapsed.as_secs_f64());
    }
    assert!(!records.cases.is_empty(), "profile ran no cases");
    result.unwrap();
}

#[test]
#[ignore = "manual measurement of the current full constitution, not a verification gate"]
fn profile_live_preparation_snapshots_and_cases() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            TRACE_STEPS.set(true);
            let context = crate::context::Context::discover().unwrap();
            let source = context.read("new-book-plans/constitution.nibli").unwrap();
            let started = Instant::now();
            let compiled = CompiledSource::new(&source);
            eprintln!("PROFILE compile-source {:.3}s", started.elapsed().as_secs_f64());
            let started = Instant::now();
            let prepared = PreparedPinEngine::new_cached(
                &[LoadedSource::new("live constitution", &source)],
                Arc::new(AtomicBool::new(false)),
                &compiled,
            );
            assert!(prepared.base.harness.is_empty());
            eprintln!("PROFILE prepare-base {:.3}s", started.elapsed().as_secs_f64());
            let started = Instant::now();
            let scan = prepared.base.engine.kb().check_contradictions_report();
            assert!(scan.is_clean(), "{scan:?}");
            eprintln!("PROFILE base-scan {:.3}s", started.elapsed().as_secs_f64());

            let mut measured_cases = 0;
            for (name, fixture, pins) in [
                ("arrest", Some("tests/pins/public-safety/core/arrest/positive/fixture.nibli"), "tests/pins/public-safety/core/arrest/positive/expect.pins.nibli"),
                ("force-abroad", Some("tests/pins/public-safety/core/force-abroad/positive/fixture.nibli"), "tests/pins/public-safety/core/force-abroad/positive/expect.pins.nibli"),
                ("withdrawal", Some("tests/pins/public-safety/review/reviewed-defect/standing-without-separate-entry/fixture.nibli"), "tests/pins/public-safety/review/reviewed-defect/standing-without-separate-entry/expect.pins.nibli"),
                ("floor-controls", None, "new-book-plans/rights-floor.pins.nibli"),
                ("instrument-firewall", None, "tests/pins/public-safety/firewalls/arrest/expect.pins.nibli"),
            ] {
                if std::env::var("RIGHTS_PROFILE_CASE")
                    .is_ok_and(|selected| selected != name)
                {
                    continue;
                }
                measured_cases += 1;
                let fixture = fixture.map(|path| context.read(path).unwrap()).unwrap_or_default();
                let pins = context.read(pins).unwrap();
                let started = Instant::now();
                prepared.base.engine.kb().with_assumptions(&[], |kb| {
                    let snapshot = started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    let statements = fixture.lines().map(str::trim)
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                        .map(|text| (prepared.base.engine.compile_text(text).unwrap(), text.to_owned()))
                        .collect();
                    let compile = started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    kb.assert_compiled_batch(statements).unwrap();
                    let assert = started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    let report = run_file_with_engine(
                        &LoadedSource::new(name, &pins), &prepared.base.engine, kb,
                        PinOptions::default(),
                    );
                    let query = started.elapsed().as_secs_f64();
                    assert!(report.harness.is_empty() && report.findings.is_empty() && report.resolved.is_empty(), "{}", finish_file_reports(&[LoadedSource::new(name, &pins)], vec![report]).stdout);
                    let started = Instant::now();
                    let scan = kb.check_contradictions_report();
                    assert!(scan.is_clean(), "{scan:?}");
                    eprintln!("PROFILE {name}: snapshot={snapshot:.3}s compile-fixture={compile:.3}s assert-fixture={assert:.3}s pins={query:.3}s scan={:.3}s", started.elapsed().as_secs_f64());
                }).unwrap();
            }
            assert!(measured_cases > 0, "RIGHTS_PROFILE_CASE matched no measurement case");
        })
        .unwrap()
        .join()
        .unwrap();
}
