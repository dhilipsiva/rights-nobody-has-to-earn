<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Verification performance — engine candidate

Status: experimental, not integrated into the companion checkout or the
production verifier. The five-minute TODO remains open. No constitutional
source, pin expectation, case inventory or contradiction check is changed.

The companion checkout already has pending work. The adjacent patch contains
only this experiment's additional changes against that working tree, not those
pre-existing edits. It is not a standalone patch against the companion's HEAD.
The existing checkout and its index have been left untouched.

## Changes under test

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
- The final constant-index candidate passed all 622 reasoner unit tests,
  including new work-count, atomicity, mutation, cancellation, mode and
  cache-versus-fresh regressions.
- The preceding discard-and-cache candidate passed all 12 differential tests
  in 98.85 seconds. These exercise Vampire, clingo, stratification rejection,
  retraction and materialization. Rerun this suite for the final index change.

A complete four-worker run of this experimental release binary passed all
12,860 pins across 4,190 cases in **1,099.00 seconds (18 minutes 19 seconds)**.
Contradiction checks completed with no findings, and all nine existing defect
pins still reproduced. The release binary was built before the run. Brief
development-test builds also ran after the five-minute target had already been
exceeded; this is the observed run time, not an isolated benchmark.

The complete-verification target has **not** been achieved. The probe's larger
speedup does not describe the whole inventory, and the production verifier is
unchanged. The runner's development suite separately passed 24 tests, with its
two manual tests ignored; formatting and diff checks passed.

A follow-on scratch experiment removes duplicate assertion preflight and
singleton-buffer copies during loading. Its 623 reasoner unit tests pass, but
it is not part of this retained patch or the full-run binary above. It still
needs its own performance measurement and integration checks.

## Reproduction and integration boundary

The runner's ignored development probe is
`profile_live_preparation_snapshots_and_cases` in
`src/pin_performance_tests.rs`. `RIGHTS_PROFILE_CASE` optionally selects one of
its five named cases; an unknown selection fails rather than measuring zero
cases. It is never part of ordinary verification.

The current experiment lives at `/tmp/rights-nibli-perf.llSbQz`. The binary for
this retained patch is `rights-verify-c622` in that directory. Its probe
manifest uses the live book runner with copied companion crates. Run its
release binary from the book repository to execute the unchanged live
inventory. The normal `./verify.sh` still uses the unmodified companion
checkout.

Before integration, reconcile the companion's pending changes without
overwriting them, apply only the additional patch, run its relevant development
checks, and rebuild and run the book's complete verifier. Do not delete the
performance TODO without a passing complete run under five minutes.
