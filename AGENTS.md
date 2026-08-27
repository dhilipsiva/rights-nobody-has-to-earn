# Repository Guidelines

## Authority, Scope & Structure

`CLAUDE.md` is authoritative. Before work, read it, `TODO.md`, and `tmp.txt` (draft context only). `book-1/` contains numbered chapter/`.pins.nibli` pairs. Except for its labelled opening note, Part V, and method, prose must derive from `new-book-plans/constitution.nibli`; keep it jargon-free. Roadmaps, scaling, and implementation belong in book 2, which stays inactive. Preserve legacy `book.md`/`manifesto.md` until the TODO harvest.

`new-book-plans/` owns the constitution, generated spine/audits, reviewed JSON contracts, bounded record-snapshot red-team, and counterfactuals. `registry/` holds claims, snapshots, checks, and fetchers. Keep `book-1/epigraph.md` and `method.md` unnumbered.

## Validation & Development Commands

Run from the repository root:

```bash
./verify.sh --quick   # schema/freshness; skips executable suites
./verify.sh           # authoritative full suite
./verify.sh --emit-receipt new-book-plans/verification-receipts
./verify.sh --wait-for-lock 300 --emit-receipt new-book-plans/verification-receipts
./verify.sh --commit-gate <receipt> --transition audit|closure|tracker
./verify.sh --only book-1/<NN-chapter>.pins.nibli   # one pin suite in a fresh engine
./verify.sh --table   # print the claim table
python3 new-book-plans/5-spine-gen.py new-book-plans/constitution.nibli new-book-plans/3-spine.md --check
python3 new-book-plans/7-assertion-surface.py --check
python3 new-book-plans/8-record-integrity-assurance.py --check
python3 new-book-plans/9-record-integrity-red-team.py --check
python3 new-book-plans/9-record-integrity-red-team.py --check --execute
python3 new-book-plans/10-amendment-semantics.py --check
python3 new-book-plans/10-amendment-semantics.py --check --execute
python3 new-book-plans/11-placement-exhaustiveness.py --check
python3 new-book-plans/11-placement-exhaustiveness.py --check --execute
python3 new-book-plans/12-temporal-assurance.py --check
python3 new-book-plans/12-temporal-assurance.py --check --execute
python3 new-book-plans/14-reader-evidence.py --check
python3 new-book-plans/14-reader-evidence.py --check --execute
python3 new-book-plans/reader-evidence-admission-gate.py --self-test
python3 new-book-plans/15-pilot-reader-artifacts.py --check
python3 new-book-plans/17-full-society-power-source-manifest.py --check
python3 new-book-plans/13-full-society-ledger.py --check
python3 new-book-plans/13-full-society-ledger.py --refresh-and-check
python3 new-book-plans/16-constitutional-closure.py --check
python3 new-book-plans/16-constitutional-closure.py --refresh-and-check
python3 registry/check.py
```

**Superseding commit rule, 2026-08-23.** A semantic, executable, verifier,
fixture, engine, or generated-artifact change requires one fully staged
`--emit-receipt` full run. Do not run quick immediately before that unchanged
full run. Only the exact following audit, closure, and tracker administrative
successors may use `--commit-gate`, and only while the receipt's heavyweight
dependency manifest is byte-identical and their transition-specific structural
validators pass. Missing local evidence or any unexpected delta fails closed
without silently launching a full run. Existing default, quick, only, and table
modes retain their meanings; quick and focused modes are not semantic gates.

Heavyweight verifier entry points share one Git-common-directory kernel lock.
Contention exits 75 with sanitised owner details unless `--wait-for-lock
SECONDS` supplies an explicit bounded wait. `verify.sh` incrementally builds and
executes one `rights-verify` Rust binary; it launches no Python verifier
subprocesses. Native ledger and closure checks preserve the immutable-input and
final-reread contract; scripts 13 and 16 remain standalone atomic generators.
The embedded Nibli engine is compiled from the adjacent source checkout and its
source revision plus exact verifier bytes are bound into new receipts.
State-form execution retains its reviewed 64 main and 17 counterfactual
byte-balanced shards under a bounded four-worker, canonical-output, fail-fast
scheduler inside the native process.

Use release `nibli-pin --kb` at or after `4cb02aade43b394374c40e661907ad66df3af3fe`, never `nibli-host`. Omit `--check` only to regenerate. Edit reviewed JSON, never generated reports or spine blocks. After a rule/fact change, run `7-assertion-surface.py --fingerprints`, review, then copy candidate digests. Refresh reviewed digests in this order: assertion ledger (7), assurance source (8), red-team source (9), amendment and placement sources (10/11), then temporal source (12). The full-society ledger (13) sits off that chain — it digest-binds only the assurance-portfolio and full-society-boundary decisions and re-reads the sibling reviewed JSONs live at `--check` — so refresh it when either bound decision changes, and expect it to fail when a sibling adds a reviewed enum value with no mapping row. The same run generates and freshness-checks `full-society-ledger.md` and `full-society-reader-ledger.md`; the latter is structural navigation only and supplies no R6 evidence, comprehension result, accessibility validation, reader-suitability claim, Gate C evidence, or route availability. Generate reports 9 and 12 before rendering report 8 because its reviewed references name those outputs; then generate/check reports 8, 10, and 11. Evidence roles may not relabel a gap as assurance. After every constitution edit, comments included, regenerate counterfactuals and run the full verifier. `new-book-plans/4-strata.py` is retained wrong on purpose as a method-part exhibit — do not repair it; `18-coverage-contract-migration.py` is a reviewed-source migration helper only, outside the verify chain, and script 13 must validate everything it emits.

Validate and render reader evidence with script 14 before checking the
full-society ledger when its reviewed source changes. `--check` validates the
reviewed state and digest contract; `--check --execute` also exercises the
deterministic evaluator, derived end-to-end boundary fixtures, and the
watched-failing mutations required by the populated stage. The checker enforces
global pilot/holdout identifier uniqueness, one freshness record per run,
payload-bound custody, embedded frozen ratification, canonical UTC chronology,
candidate ancestry, and the root `history_transition` contract. That contract
binds `previous_source_commit`, `previous_source_sha256`,
`previous_history_head_sha256`, and `history_head_sha256` to the nearest
earlier normal first-parent commit that changed `reader-evidence.json`;
preserves prior attempt prefixes and terminal attempts; and permits one pilot-
or holdout-domain step only. Each successor pre-registration binds
`predecessor_attempt_sha256` and `prior_history_head_sha256` to its frozen
predecessor. These are artifact checks over visible normal first-parent Git
history only: they neither prove resistance to rewritten Git history nor attest
to external truth.

Run script 17 before script 13. It checks the source-digest-bound
full-society power census: 237 reviewed source entries, comprising 209
card-required powers, 1 cross-power contract template, 19 explicit
refusals or limits, and 8 narrow current-
formal crosswalks. The census is an inventory prerequisite only. It creates no
FS-POW contract card, lawful holder, operation, assurance, or Gate A result.
Script 13 now checks the completed source-derived population: 210 actual
legal-effect cards, 1 cross-power temporal template, 19 refusal or limit rows,
8 formal dispositions, and 210 power-bound allocations. Completion removes
only the powers deferral. Gate A condition five is met by the current-source repository adversarial
audit. Gate A closes through a checker-derived record bound to an immutable
verified candidate; no human act is required. External human review is optional
evidence and no project gate or publication depends on it.

Run script 16 after script 13. It computes the claim-scoped constitutional-
closure and model-allocation projection from the reviewed canonical source and
checks `constitutional-closure-and-model-allocation-audit.md` plus its watched-
failing mutations. Its `pass`, `block`, and `bounded-unresolved` results are
structural artifact results only. A `pass` means that the reviewed structural
contract for that claim passes the declared audit; it upgrades no claim posture
and establishes no delivery, liveness, feasibility, operation, or Gate A
closure. Both quick and full verification run this structural check. Edit the
reviewed canonical source, never the generated audit.

Run the fixed admission gate's `--self-test` in both quick and full verification.
That builds and tests one component only; it does not make R6 built or available.
Every active completed attempt must store `gate_admission_receipt`, and only the
exact output of the digest-bound gate with `decision=admit` may establish
FS-CLM-37. The current dormant source supplies no pilot-derived taxonomy label
or threshold value.

Script 15 checks deterministic HTML/EPUB generation, semantic source headings,
and local body-link targets for the draft pilot snapshot. It does not render or
validate PDF, perform a human screen-reader smoke test, freeze an instrument or
snapshot, run a pilot, create reader evidence, or make R6 built or available.
The public pilot kit contains templates only; the runnable instrument, rubric,
seeded-control preimage, identities, and raw responses remain private.

Script 10 manually applies candidates and does not prove enactment. Script 11 rejects conflicts with the current routing matrix when run but adds no runtime placement alarm or housing-delivery evidence. Script 12 proves bounded supplied-record safety, not outside clock, publication, storage, or institutional liveness.

## Editing, Testing & Naming

Match Markdown hierarchy and `NN-kebab-case.md`/`.pins.nibli` pairs. Write controls as `:accept-scoped`; use `:accept` only when the accepted statement is a later premise. State the rule producing a count, not a counted design claim. Add a primary source and URL with every statistic or named study. Python uses four spaces; new code needs `SPDX-License-Identifier: MIT OR Apache-2.0`.

## Commits, Pull Requests & Licensing

Make one chapter or section change per content commit. Use `<area>: <outcome>` subjects and explain why in a ~72-column body. Close TODOs separately with `Tracker: <what landed> (<content SHA>)`. Pull requests summarize the claim, validation, regenerated artifacts, and tracker item; screenshots are only for rendered visual changes.

Read `LICENSING.md` before adding files. New prose is CC-BY-4.0, code is MIT OR Apache-2.0, registry claims are CC0, and data snapshots can carry upstream terms. Legacy pre-decision material remains CC0 under the root `LICENSE`.
