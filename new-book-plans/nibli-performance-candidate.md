<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Verification performance — integrated engine changes

Status: integrated as Nibli `fdd9de1` and `3f64f0b`. Engine integration checks
passed, and the rebuilt normal verifier passed the complete book in
**448.90 seconds (7 minutes 28.90 seconds)**.
The five-minute TODO remains open. No constitutional source, pin expectation,
case inventory or contradiction check is changed.

The companion's previously pending work landed as `fa91a5f`. The adjacent
patch applies directly to that commit and contains only this experiment's
additional changes, now committed as `fdd9de1` and `3f64f0b`. The companion was
clean before the patch was applied; no unrelated companion work was overwritten.

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

The complete-verification target has **not** been achieved. The probe's larger
speedup does not describe the whole inventory. Both the scratch and normal
release verifier completed the unchanged full inventory, but neither run met
five minutes. The runner's development suite separately passed 24 tests, with its
two manual tests ignored; formatting and diff checks passed.

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

## Reproduction and integration boundary

The runner's ignored development probe is
`profile_live_preparation_snapshots_and_cases` in
`src/pin_performance_tests.rs`. `RIGHTS_PROFILE_CASE` optionally selects one of
its five named cases; an unknown selection fails rather than measuring zero
cases. It is never part of ordinary verification.

The scratch workspace is `/tmp/rights-nibli-perf.llSbQz`. Its probe manifest
uses the live book runner with copied companion crates. The measured release
binary is `probe-target/release/rights-verify`. The separately retained
`rights-verify-c622`, `rights-verify-c626` and `rights-verify-c628` are earlier full-run candidates,
not the current patch. Scratch source can contain subsequent untested
experiments; the adjacent patch is the retained, measured candidate.
Run an experimental binary from the book repository to execute the unchanged
live inventory. Normal `./verify.sh` still uses the companion checkout.

The additional patch is committed in the companion checkout as `fdd9de1` and
`3f64f0b`. Native, WebAssembly and mutation checks are complete. The normal book
verifier passed the full inventory against `3f64f0b` in 448.90 seconds.
The performance TODO remains open until a complete run passes under five minutes.
