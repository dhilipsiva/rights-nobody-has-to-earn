<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Verification performance — integrated engine changes

Status: the complete normal verifier passed in **275.04 seconds
(4 minutes 35.04 seconds)**, meeting the five-minute TODO. The latest
local-binding and shared domain-rule changes are committed in Nibli `158e3b0`;
the runner-affinity and bundled-allocator changes accompany this note.
No constitutional source, pin expectation, case inventory or contradiction
check is changed.

The companion's previously pending work landed as `fa91a5f`. The adjacent
patch applies directly to that commit and contains only this experiment's
additional changes through `158e3b0`, including the local-binding and shared
domain-rule changes. No unrelated companion work was overwritten.

## Integrated changes

- Failed assertions run all ordinary guards on a detached candidate, then
  discard that candidate on error. Replaying its entire assertion registry
  before discarding it was redundant. Single, preassigned-ID, append-batch and
  fresh-batch ingress retain atomicity and live assertion-ID continuity.
- A query root already completed as a dependency reuses that completed
  extension. Dirty inserts are handled first; incomplete dependencies do not
  gain a completeness claim.
- A fully closed witness domain survives read-only queries. Mutation clears
  the reusable stamp; depth, proof/lookup mode and closure limits are checked.
  Incomplete closures are never reused as complete, and cancellation is still
  checked on the reuse path.
- Constant positions can index the first positive join. The existing
  round-local indexes are shared across rules; the ordinary tuple binder still
  checks every surviving candidate.
- Singleton batches move their owned buffer into assertion ingress and run
  structural preflight once, before allocating an ID. Multi-root statements
  retain whole-buffer preflight before any root is installed; replay retains
  its own validation.
- Domain planning compares a shared rule identity once instead of once per
  conclusion-index reference. Distinct allocations still receive full-value
  equality checking, with the same first-seen selection as before.
- A missing positive relation or constant-only match in any positive atom
  rejects a rule before building earlier joins. The indexes remain round-local
  and distinguish full from delta extensions; all survivors use the ordinary
  tuple binder.
- The immutable rule plan owns distinct dependency lists and negative-read
  membership. Reachability, completion checks and delta invalidation reuse
  those lists; no executable condition is removed or weakened.
- Rule compilation identifies condition-bound variables before building
  dependent-witness lists. It uses the same capped disjunction expansion and
  binder walk, without copying lists that the old path immediately discarded.
- Fixed-value positive checks run before the whole-body presence scan. A
  failed discriminator excludes a rule early; surviving candidates still
  receive every ordinary presence, arity and binding check.
- Already range-restricted projections specialize positive scalar equalities
  across positive atoms, negative atoms, heads and builtins. Every equality
  remains executable, including conflicting or negated equalities. Necessary
  prechecks prioritize the specialized fields; executable join order and the
  global equality-class refusal are unchanged.
- Extending the set of requested query roots moves already completed
  extensions into the freshly seeded model without repeating their joins.
  Dirty, incomplete or newly unseedable prior cones are not reused. Stored
  shape checks and out-of-cone inserts are re-read; cumulative tuple-budget
  boundaries fall back to the ordinary full evaluation order. Debug builds
  compare the completed results with a fresh evaluation of the whole union.
- Candidate narrowing borrows the knowledge base's existing immutable domain
  slice instead of cloning the whole domain in both existential and grouped
  event searches. Selected candidates still have owned output, and anchor
  selection, fallback domains, bindings and proof construction are unchanged.

Scoped-control retractions still execute the existing replay path. No pin
verdict is cached between verification runs. These are engine-internal,
within-run optimizations, not receipts, hashes or additional verification gates.

## Evidence so far

The production baseline passed 12,860 pins across 4,190 cases in 1,552.00
seconds with four workers and the release verifier built.

The manual development probe runs five representative live cases: arrest,
force abroad, a stateful withdrawal sequence, floor controls, and instrument
refusals. It executes their pins and complete contradiction reports.

- Unchanged engine: 143.49 seconds for the probe, including preparation.
- Detached-candidate discard and completed-root reuse: 44.70 seconds.
- Adding complete-witness reuse: 34.76 seconds. The eight-refusal instrument
  case took 0.018 seconds, compared with 28.574 seconds before the changes.
- The earlier constant-index candidate passed all 622 reasoner unit tests,
  including new work-count, atomicity, mutation, cancellation, mode and
  cache-versus-fresh regressions.
- The preceding discard-and-cache candidate passed all 12 differential tests
  in 98.85 seconds. These exercise Vampire, clingo, stratification rejection,
  retraction and materialization.
- Adding singleton/preflight changes passed 623 reasoner tests and all 12
  differential tests (90.82 seconds).
- Adding shared-identity filtering measured 31.96 seconds for the probe;
  constant prechecks reduced that to 28.68 seconds; planned dependency lists
  reduced it to **26.64 seconds**. That candidate passed 626 reasoner
  tests and five session fixture-batch tests. All 12 oracle/differential tests
  also passed (98.54 seconds).

The candidate retained at book commit `461f846` passed all **12,860 pins across
4,190 cases in
558.35 seconds (9 minutes 18.35 seconds)** with four workers. Contradiction
checks completed with no findings, and all nine existing defect pins still
reproduced. The release binary was built before the run; no other development
build or test was started during this measurement. This remains above the
five-minute target. That candidate passed 628 reasoner tests, all 12 oracle tests
(98.49 seconds), five session fixture-batch tests and 24 runner tests. Its
five-case probe took 23.46 seconds.

The scratch equality-specialization candidate passed 632 reasoner tests,
all 12 oracle/differential tests (94.27 seconds), five session fixture-batch
tests and 24 runner tests. Its five-case probe took 20.75 seconds. Its complete
run passed all **12,860 pins across 4,190 cases in 525.93 seconds (8 minutes
45.93 seconds)** with four workers, complete contradiction checks, no findings
and all nine existing defect pins reproduced. The release binary was built
before the run. This session started no other development build or test during
the measurement; other machine work was not controlled.

Integration mutation testing exposed a missing negative-only equality case,
not a failure of the unmutated implementation. The regression now checks
`~($kind = Other)` without a positive equality, and its denying counterpart.
The focused test passes. The first mutation run was stopped because its
reasoner-only baseline gave the wider per-mutation test set an inadequate
60-second floor; those timeouts are not credited as caught mutations. The
corrected run uses a 180-second floor and has caught the negative-only mutation.
The corrected broad sweep finished with 34 caught, six unbuildable, one missed
and two timed-out mutations. A new old-premise/new-delta regression catches
the missed full-versus-delta index mutation. Focused reruns also caught the
condition-binder mutation and the resumption-disabled mutation with explicit
assertion failures. The latter uses the existing in-cone/out-of-cone mechanism
test; the longer transitive-closure cost probe was interrupted rather than
credited as a catch. Combined, the broad sweep and focused follow-ups account
for all 43 mutations: 37 caught and six unbuildable, none unresolved. This is
not a claim that the broad sweep alone passed.

The first integration's reasoner suite passes all 633 tests. Native and
WebAssembly integration checks passed through `just ci-all`; the evidence
chapter also passed all 40 pins through normal `./verify.sh --only`.

The rebuilt normal `./verify.sh` then passed all **12,860 pins across 4,190
cases in 526.66 seconds (8 minutes 46.66 seconds)**, with complete contradiction
checks, no findings and all nine existing defect pins reproduced. This run
used the actual companion checkout and the book's dependency lockfile, four
workers and a prebuilt release binary. This session started no other development
build or test during the measurement; other machine work was not controlled.

The completed-cone extension candidate passed the unchanged **12,860 pins
across 4,190 cases in 457.07 seconds (7 minutes 37.07 seconds)** with four
workers and a prebuilt release binary. Contradiction checks completed with no
findings, and all nine existing defect pins still reproduced. This was a
scratch-binary run; it is not the normal-checkout measurement above. This
session started no other development build or test during the measurement;
other machine work was not controlled.

The extension candidate passed 636 reasoner tests and all 12 oracle/differential
tests (94.13 seconds). Its five-case probe took 20.64 seconds; arrest pins took
0.132 seconds, force-abroad pins 0.154 seconds, and withdrawal pins 0.321 seconds.
Model loading and the three actual scoped retractions still dominate that probe.
The final companion suite passes 637 reasoner tests after a defensive-helper
regression was added. The normal runner passes 24 development tests, with its
two manual probes ignored. Native and WebAssembly integration checks passed
through `just ci-all`, including another complete oracle/differential pass
(96.53 seconds).

The extension's broad mutation sweep finished with nine caught, two unbuildable
and two missed mutations. The new regression directly supplies dirty,
incomplete and newly unseedable prior results to the extension helper. Both
previously missed guard mutations are caught by assertion failures in a focused
rerun using the same configured test packages. Combined: 11 caught, two
unbuildable, none unresolved. The broad sweep alone did not pass; no timeout
is credited as a catch.

The rebuilt normal `./verify.sh` against Nibli `3f64f0b` then passed all
**12,860 pins across 4,190 cases in 448.90 seconds (7 minutes 28.90 seconds)**.
Contradiction checks completed with no findings, and all nine existing defect
pins still reproduced. This used the actual companion checkout and the book's
dependency lockfile, four workers and a prebuilt release binary. This session
started no other development build or test during the measurement; other
machine work was not controlled. The five-minute target is still not met.

The domain-borrowing candidate passed all 637 reasoner tests and the normal
runner's 24 development tests (two manual probes ignored). Native and
WebAssembly checks passed through `just ci-all`, including all 12
oracle/differential tests (90.59 seconds). Its targeted mutation sweep finished
with five caught and two unbuildable mutations, none unresolved.

In a fresh paired arrest-case diagnostic, the first query took 0.152 seconds
before borrowing and 0.073 seconds afterwards. Both runs performed 5,598
witness-activation checks and stored evidence for 264 activations; the change
does not bypass those proofs. Construction times were 2.735 and 2.740 seconds,
so no loading improvement is credited to this change. The single-query
improvement is not a whole-book timing claim.

The rebuilt normal `./verify.sh` against the domain-borrowing change, now Nibli
`8e72f6c`, passed all **12,860 pins across 4,190 cases in 416.94 seconds
(6 minutes 56.94 seconds)**. Contradiction checks completed with no findings,
and all nine known-defect pins still reproduced. Four workers, the book's
dependency lockfile and a prebuilt release binary were used. This session
started no other development build or test during the measurement; other
machine work was not controlled. The five-minute target remains open.

The external timing wrapper reported 1,553.90 seconds of user CPU time,
39.40 seconds of system CPU time, 380% CPU utilization and peak resident
memory of 18,797,032 KiB, with no swaps or major page faults. Its 418.41-second
wall time includes the incremental build wrapper; the verifier's own measured
time is the 416.94 seconds above.

The preceding candidate retained at book commit `9d8fb07` passed the same full
inventory in **661.85 seconds (11 minutes 1.85 seconds)**, also with four
workers, complete contradiction checks, no findings, nine reproduced defect
pins and no overlapping development build or test.

A complete four-worker run of the **earlier** candidate retained at book commit
`f5924d5` passed all
12,860 pins across 4,190 cases in **1,099.00 seconds (18 minutes 19 seconds)**.
Contradiction checks completed with no findings, and all nine existing defect
pins still reproduced. The release binary was built before the run. Brief
development-test builds also ran after the five-minute target had already been
exceeded; this is the observed run time, not an isolated benchmark.

At that stage the complete-verification target had **not** been achieved. The
probe's larger speedup did not describe the whole inventory. Both the scratch
and normal release verifier completed the unchanged full inventory, but neither
run met five minutes. The runner's development suite separately passed 24
tests, with its two manual tests ignored; formatting and diff checks passed.

A separate strict reasoner Clippy check did not pass: existing front-end and
reasoner warnings remain outside this patch's scope. Two warnings in new
candidate expressions were corrected. The required runtime Clippy check and
`just ci-all` subsequently passed; that does not erase this separate diagnostic
failure.

The latest preparation probe measured 0.449 seconds copying compiled
statements, 2.711 seconds constructing the ordered model, and 1.243 seconds
preparing the rule plan. Three scoped-control retractions still took
2.79–3.56 seconds each. These costs are not hidden by the query gains.

A temporary internal loading profile measured 25,641,517 variable-name copies
before the condition-variable change, versus 1,209,036 afterwards. The same
diagnostic measured model construction at 3.622 and 2.885 seconds respectively.
Those temporary timing hooks are not in the retained patch or the verifier.

An ordered-rule-insertion experiment was tested and discarded: its repeated
construction measurement (3.640 seconds) did not improve on the paired prior
candidate (3.589 seconds). No speedup is credited to it.

## Full-inventory profiling

An opt-in development profile on 2026-09-13 ran the normal execution path
against companion `8e72f6c`: all 12,860 pins across 4,190 cases passed, all
contradiction checks completed with no findings, and the nine known defects
reproduced. Its instrumented time was 407.57 seconds, not a replacement for
the normal release measurement of 416.94 seconds.

The profile measured 635.299 cumulative worker-seconds preparing case bases
and 929.568 executing cases. Parallel worker durations are not wall-clock
contributions. Within those totals:

| Phase | Cumulative worker-seconds |
| --- | ---: |
| Ordered model construction, 145 builds | 380.749 |
| Materialization planning, 145 builds | 164.702 |
| Compiled statement copies | 22.486 |
| Fixture loading | 237.857 |
| Queries | 514.506 |
| Real scoped retractions | 48.827 |
| Independent snapshots | 17.917 |
| Complete contradiction scans | 2.252 |

Preparation accounted for about 41% of the measured case work, queries 33%,
and fixtures 15%; contradiction scans were below 1%. Separately, a temporary
field-by-field loading probe measured a warmed live snapshot at 0.006 seconds,
so a broad copy-on-write rewrite is not justified by that measurement.
Neither diagnostic changes what normal verification executes or accepts.

## Local bindings, worker affinity and allocator (2026-09-14)

Negated event groups now borrow their clause's variable map with one local
shadowing binding. The resulting templates preserve lookup precedence,
Skolems, recursive descent, flavor and refusal behavior. A paired loading
probe measured ordered construction at 3.179 seconds before this change and
2.659 seconds afterwards. This is a loading measurement, not a whole-book
speedup claim.

The runner also groups cases with exactly the same noncanonical base and
ordered edits on one worker. Canonical cases remain independently scheduled;
fixtures and pin files still receive their original isolation. Grouped
execution preserves original-index failure priority, cancellation and output
order, including a late failure in an early group. Every case still runs.

Together these changes passed normal `./verify.sh` in **348.17 seconds
(5 minutes 48.17 seconds)**: all 12,860 pins across 4,190 cases, complete
contradiction checks with no findings, and nine reproduced known defects.
Four workers and a prebuilt release binary were used, with no other session
build or test overlapping. External wall time was 349.64 seconds, user CPU
1,303.22 seconds, system CPU 23.46 seconds, utilization 379%, and peak RSS
18,028,924 KiB; there were three major page faults and no swaps.

The reasoner passed 640 tests and `just ci-all`, including the native, oracle,
Lean, WASI and browser/V8 checks. The configured changed-code mutation sweep
finished with eight caught, three unviable and three timed-out mutations
(exit 3). Those timeout logs had failed assertions before later tests hung.
A supplemental focused replay of all five `PatternVariables` mutations
finished with four caught and one unviable (exit 0), resolving the timeout
cases. This does not turn the original broad run into a timeout-free pass or
change the companion's mutation baseline.

The runner passed 29 development tests; three manual probes were ignored.
An optional strict rights Clippy check failed on existing warnings in unchanged
pin/child-process code and implicit generator targets. No unrelated warning
cleanup or suppression was added, and a baseline Clippy rerun was not made.

A separate allocator experiment ran the same prebuilt binary with system
jemalloc 5.3.0 preloaded. It passed the unchanged full inventory in **283.31
seconds (4 minutes 43.31 seconds)**, with complete contradiction checks, no
findings and all nine known defects reproduced. External wall time was
283.49 seconds, user CPU 1,027.29 seconds, system CPU 57.69 seconds, utilization
382%, peak RSS 12,714,692 KiB, and no major page faults or swaps. No other
session build or test overlapped; other machine work was not controlled.
The initial preload attempt failed before any pin because its C++ runtime
library was not found; the measured run used an explicit library search path.
This experiment alone does not establish normal-workflow performance.

The verifier now selects `tikv-jemallocator` on Linux GNU targets; other targets
and the separate authoring executable retain their default allocator. The
bundled library removes the experimental system-library/preload setup. The
[upstream integration instructions](https://github.com/tikv/jemallocator)
describe the `GlobalAlloc` wrapper. Cargo resolves wrapper 0.7.0 to published
sys crate 0.7.1, bundling jemalloc 5.3.1; the dependency lock records the actual
packages used. The published source was inspected, rather than assuming its
repository tag represents the package; upstream separately records the
[sys release/repository mismatch](https://github.com/tikv/jemallocator/issues/169).

The first debug build failed because Nix's fortified C headers reject `-O0`
under the allocator's `-Werror` configuration probe. Optimizing only
`tikv-jemalloc-sys` in the development profile preserves that hardening and
lets the normal debug test command build. All 29 allocator-backed runner
tests then passed. The Rust test profile inherits this
[package-specific development setting](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides).

Normal `./verify.sh` with the bundled allocator then passed all **12,860 pins
across 4,190 cases in 310.79 seconds (5 minutes 10.79 seconds)**. Contradiction
checks completed with no findings, and all nine known defects reproduced.
The release binary was prebuilt, four workers were used, and `LD_PRELOAD`,
`MALLOC_CONF` and `_RJEM_MALLOC_CONF` were unset. No other session build or test
overlapped. External wall time was 311.05 seconds, user CPU 1,144.34 seconds,
system CPU 47.77 seconds, utilization 383%, peak RSS 13,587,420 KiB, with no
major page faults or swaps. The runtime dependency list contains libc and
libgcc, not a system jemalloc or C++ runtime. The normal run improved, but
remained 10.79 seconds above target; the experimental preload timing is not
substituted for it.

## Sharing domain-planning inputs (2026-09-14)

The dependency graph and activation schedule now consume the same borrowed
list of semantically distinct domain rules. Previously each independently
performed the same identity comparisons. The earlier materialization-rule
iterator remains separate: it has a different template scope and ordering.
Graph edges, stratification, activation guards and mutation invalidation are
unchanged; the list exists only while preparing that model.

A regression first observed four identity comparisons, then passed with the
required two after the change. New semantic regressions exercise quoted-body
opacity, adding and retracting a generating rule, invalid graphs and synthetic
domain-node collisions. All 642 reasoner tests and all 29 runner tests passed.
The paired release probe measured domain planning at 0.788 seconds before
and 0.717 seconds after the change. Other preparation phases also varied;
this isolated observation is not a claimed full-book improvement.

A supplemental mutation run explicitly included `domain.rs`, which the
companion's usual mutation configuration excludes. All 16 mutations of
`domain_dependency_graph` were caught, with no missed or timed-out cases
(exit 0). This includes whole-graph replacements and guard mutations; the
tool did not generate a mutation of the new list-construction expression.

The current engine also passed `just ci-all`, including native tests, all
12 oracle/differential cases (110.36 seconds), Lean proofs, WASI host checks
and the browser-class V8 determinism and lifecycle tests. These development
checks are not added to routine book verification.

An additional empty-template fast path was tried and removed. It did not
show a clear further benefit; a quoted-body fixture also disproved the
assumption that absence of surface `some` syntax means no individual
templates. The current candidate retains complete domain-graph preparation.

The rebuilt normal `./verify.sh` then passed all **12,860 pins across 4,190
cases in 275.04 seconds (4 minutes 35.04 seconds)**. Complete contradiction
checks found no contradictions and all nine known-defect expectations still
reproduced (exit 0). Four workers and a prebuilt release binary were used,
with `LD_PRELOAD`, `MALLOC_CONF` and `_RJEM_MALLOC_CONF` unset. No other build
or test from this session overlapped; other machine activity was not controlled.
External wall time was 275.29 seconds, user CPU 1,004.99 seconds, system CPU
42.17 seconds, utilization 380%, peak RSS 13,447,508 KiB, one major page fault
and no swaps. This measured normal-workflow run meets the five-minute target;
the difference from earlier runs is not attributed solely to one optimization
or offered as a runtime guarantee on every machine.

The timed command, from the rights repository, was:

```bash
nix develop /home/dhilipsiva/projects/dhilipsiva/nibli \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes --command env \
  -u LD_PRELOAD -u MALLOC_CONF -u _RJEM_MALLOC_CONF \
  RIGHTS_VERIFY_JOBS=4 /usr/bin/time -v ./verify.sh
```

## Reproduction and integration boundary

The runner's ignored development probe is
`profile_live_preparation_snapshots_and_cases` in
`src/pin_performance_tests.rs`. `RIGHTS_PROFILE_CASE` optionally selects one of
its five named cases; an unknown selection fails rather than measuring zero
cases. It is never part of ordinary verification.

The additional ignored test `profile_complete_verification` runs the actual
full-inventory execution path and aggregates preparation, fixture, snapshot,
query, retraction and scan timings. Run it alone, with `--ignored --exact
--nocapture --test-threads=1`, as
`pin::performance_tests::profile_complete_verification` in the release
`rights-verify` test binary. All profiling hooks are compiled only for tests;
there is no production profiling flag, persisted verdict or extra gate.

Earlier experiments used `/tmp/rights-nibli-perf.llSbQz`; that temporary
workspace was cleared before the September 14 resumption. It is not a current
reproduction path. The adjacent cumulative patch retains the engine changes
against `fa91a5f`; normal `./verify.sh` builds against the actual adjacent
companion checkout and executes the live book inventory. Build the release
verifier before timing it, and run no other development build or test during
the measurement. No allocator preload is needed for the integrated binary.

The additional patch is committed in the companion checkout through `158e3b0`.
Native, WebAssembly and mutation checks are recorded above. The normal book
verifier passed the full
inventory with the grouped runner and bundled allocator in 275.04 seconds.
The completed performance tracker item was removed; no subsequent pin is
skipped because of this result.

## The expanded inventory — measured 2026-09-15

The ecological family took the inventory from 4,190 cases to 13,566 and from
12,860 pins to 77,902. The 275.04-second result above describes the smaller
inventory and is **not** a current timing. Measured at book `3e199dc1` against
companion `979fe8b`, four workers, release binary prebuilt, no other session
build or test overlapping and other machine activity uncontrolled:

> All **77,902 pins across 13,566 cases** passed with complete contradiction
> checks and no findings in **824.25 seconds (13 minutes 44.25 seconds)**. Nine
> existing known-defect expectations still reproduce. External wall time
> 13:44.57, user CPU 3,174.52 s, system CPU 77.29 s, 394% utilisation, peak
> resident memory 21,626,012 KiB, zero major page faults and zero swaps.

The five-minute target needs about 1,200 cumulative worker-seconds; the run
spends about 3,143. What follows is where they go and what each lead is worth,
so the next attempt starts from measurements instead of guesses.

### Where the time goes

`profile_complete_verification` ran the ordinary execution path over the same
inventory (instrumented time 810.18 seconds, not a substitute for the release
measurement above). Parallel worker durations are not wall-clock contributions.

| Phase | Calls | Cumulative worker-seconds |
| --- | ---: | ---: |
| Queries | 77,517 | 1,652.473 |
| Fixture loading | 13,596 | 624.157 |
| Ordered model construction | 131 | 356.649 |
| Materialization planning | 131 | 139.377 |
| Independent snapshots | 13,382 | 68.449 |
| Real scoped retractions | 15 | 55.746 |
| Compiled statement copies | 131 | 22.687 |
| Pin-file assertions | 554 | 17.472 |
| Complete contradiction scans | 13,693 | 4.977 |

Preparation accounted for 591.211 seconds and case execution for 2,551.456.
Contradiction scans remain below one per cent and are not a lead.

### Preparation is already minimal

The inventory declares 127 distinct non-canonical preparation keys — a base
name plus its ordered edits — and the four workers each build the canonical
live base once. That is exactly the 131 observed builds, so no base is built
twice and no scheduling change can remove one. The uncomfortable part is the
ratio: those 127 variant models serve **216 cases**, 1.6% of the inventory, for
496 of the 591 preparation seconds. A cheaper way to obtain a one-statement
variant of an already-built model is a preparation-side lead in its own right;
rule retraction is not it, at 3.1–4.2 seconds per retracted rule.

### The fixed per-snapshot cost, and why warming does not remove it

The first query inside a fresh snapshot costs 37–48 ms **whatever it asks**.
Measured with `RIGHTS_PROFILE_CONE=1` on the live base:

| Query | Cold | Same query again |
| --- | ---: | ---: |
| `person(Adam).` | 0.0475 s | 0.0001 s |
| `prisoner(Adam).` | 0.0435 s | 0.0001 s |
| `complete(ECOmitF0Record, ECEnvironmentalRightClaim, ECOmitSharedV26).` | 0.0400 s | 0.0001 s |
| `complete(PSCorearrestRecord, PSIndividualArrestOrder, PSCorearrestSubject).` | 0.0377 s | 0.0000 s |
| `interrupt(FSBOD_22, ECStayGuardianV164, ECGuardianAutomaticStay).` | 0.0384 s | 0.0000 s |
| `false(Bela).` | 0.0374 s | 0.0000 s |
| `reward(Esa).` | 0.0358 s | 0.0001 s |

Temporary engine instrumentation (since reverted; the companion checkout is
unchanged) split that cost. `prepare_query_domain` took 28–39 ms, the
stratum-ordered saturation 10–15 ms, and `seed_edb` 0.3–1.2 ms over the base's
534 seeded tuples. Inside the domain closure the work is one round that derives
**256 witness activations** from 8 domain rules over 32 guard-restricted
candidates, and a second round that adds nothing; the candidate set comes from a
completed unary guard that no case fixture touches, so every case derives the
same 256 activations. Hoisting the closure's per-round rebuild of the 8,245-member
sweep list changed nothing measurable, which is how the 256 activations were
identified as the actual cost.

Two resets make that work per-snapshot rather than per-run. `Clone for
KnowledgeBaseInner` sets `materialized` to `None` and `query_domain` to its
default, so warming the base does not carry into a snapshot — measured directly:
after running ecology, public-safety and arrest pins against the base itself, a
no-fixture ecology case inside a snapshot still cost 0.042 s. And
`invalidate_materialization_for_insert` clears the closure stamp on every
insert, while `observe` is read under negation 124 times in the constitution, so
an ecology fixture also drops the saturation outright through `Fate::Drop`
rather than growing it.

**Lead, sized.** Carrying the saturation and the witness closure across a
snapshot, with a domain dependency cone so an insert outside it keeps the
stamp, is worth roughly 35 ms × 13,566 ≈ **475 worker-seconds, about 15%**. It
is an engine change in `nibli-reason`, not a runner change, and it needs
`ensure_domain_members_cached` to re-derive the closure's own members so a
cloned member cache and cloned activations cannot disagree.

### Case shapes

`profile_live_preparation_snapshots_and_cases` now measures two ecology cases
and two deliberately degenerate ones beside the original five. `pins-again` is
the same pin file re-run inside the same snapshot: whatever it costs is what the
first pass had to build.

| Case | snapshot | compile-fixture | assert-fixture | pins | pins-again |
| --- | ---: | ---: | ---: | ---: | ---: |
| arrest | 0.001 s | 0.005 s | 0.046 s | 0.066 s | 0.000 s |
| force-abroad | 0.002 s | 0.016 s | 0.132 s | 0.080 s | 0.000 s |
| withdrawal | 0.002 s | 0.012 s | 0.079 s | 0.179 s | 0.111 s |
| floor-controls | 0.002 s | 0.000 s | 0.002 s | 10.739 s | 9.989 s |
| instrument-firewall | 0.003 s | 0.000 s | 0.003 s | 0.014 s | 0.025 s |
| ecology-required-fields | 0.001 s | 0.007 s | 0.055 s | 0.053 s | 0.000 s |
| ecology-conflicting-fields | 0.002 s | 0.067 s | 0.494 s | 0.247 s | 0.001 s |
| ecology-no-facts | 0.002 s | 0.000 s | 0.003 s | 0.042 s | 0.000 s |
| ecology-shared-only | 0.002 s | 0.007 s | 0.049 s | 0.052 s | 0.000 s |

`ecology-no-facts` runs the required-fields pins with no fixture at all. Every
pin there is `FALSE` with or without the fixture, so the case still passes, and
its 0.042 s is the fixed cost with the case's own evidence removed: about 80% of
what the fully loaded case pays.

### Leads that were measured and are worth less than they look

**Caching compiled fixture statements.** The run asserts 6,965,211 fixture
statements, of which 1,256,800 are textually distinct, so a text-keyed cache
would remove about four fifths of the compilations. But compilation is only
about a tenth of fixture time — 0.007 s of 0.062 s for an 841-statement ecology
case — so the whole lead is worth roughly 60 worker-seconds against a
six-figure buffer cache.

**Hoisting statements common to a batch.** Consecutive cases sharing their first
fixture are already grouped: 8,085 of the 13,566 cases sit in 389 such batches,
which is why 6.97 M statements are asserted rather than the 21.39 M a naive run
would. Hoisting what every member of a batch shares is worth **nothing**: the
intersection over each of the 389 batches is empty, by construction, because
each member withholds a different required field. Pairwise overlap inside a
batch is high (80 of 88 statements for two required-fields siblings) and the
whole-batch intersection is zero.

**The heavy tail.** 116 `new-book-plans/economic-power-*` cases carry 10,000 to
21,000 assertions inside the pin file itself and run 6–12 seconds each; the
`:accept-scoped` controls in `rights-floor` and chapter 8 spend 3.1–4.2 seconds
per retracted rule, 55.7 seconds over 15 retractions. Neither is reachable from
the runner: the assertions and the retractions are what those cases test.

### What this does not establish

These are timings of one machine on one day, not a guarantee. No verdict is
cached, no pin is skipped, no expectation was weakened and the case inventory is
unchanged. The companion checkout was restored to `979fe8b` after the temporary
instrumentation; none of it is retained in either repository.
