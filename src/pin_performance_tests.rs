// SPDX-License-Identifier: MIT OR Apache-2.0

//! Manual development measurements; never part of routine verification.

use super::*;

thread_local! {
    static TRACE_STEPS: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn trace_step(kind: &str, text: &str, elapsed: Duration) {
    if TRACE_STEPS.get() && elapsed >= Duration::from_millis(20) {
        let text = text.chars().take(180).collect::<String>();
        eprintln!("PROFILE {kind} {:.3}s {text}", elapsed.as_secs_f64());
    }
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

            for (name, fixture, pins) in [
                ("arrest", Some("tests/pins/public-safety/core/arrest/positive/fixture.nibli"), "tests/pins/public-safety/core/arrest/positive/expect.pins.nibli"),
                ("force-abroad", Some("tests/pins/public-safety/core/force-abroad/positive/fixture.nibli"), "tests/pins/public-safety/core/force-abroad/positive/expect.pins.nibli"),
                ("withdrawal", Some("tests/pins/public-safety/review/reviewed-defect/standing-without-separate-entry/fixture.nibli"), "tests/pins/public-safety/review/reviewed-defect/standing-without-separate-entry/expect.pins.nibli"),
                ("floor-controls", None, "new-book-plans/rights-floor.pins.nibli"),
                ("instrument-firewall", None, "tests/pins/public-safety/firewalls/arrest/expect.pins.nibli"),
            ] {
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
        })
        .unwrap()
        .join()
        .unwrap();
}
