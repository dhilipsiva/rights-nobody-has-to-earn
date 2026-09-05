# The Rights Nobody Has to Earn

*A formally audited design for a society, worked out to the point where it
catches its own failures.*

A worked design for a society in which a short list of basic things — safety, food,
shelter, care, learning, speech, belief, company — is owed to every person with no
qualifying condition. You do not have to work, contribute, belong, register or behave.

The principal product is the exact, versioned, formally audited constitutional
specification: its executable constitution, reviewed scope and assurance
contracts, tests and counterfactuals, generated projections, and receipt-bound
audit and closure. Book 1 is its reader-facing derivation. Its chapter order is
computed from the constitution rather than chosen, and—apart from three labelled
exceptions—nothing goes into it that the constitution does not derive. Book 2
will later describe evidence-bound operation and transition within a declared,
versioned reference envelope, including its local starting-state conditions,
without changing the audited destination.

## What is here

| | |
|---|---|
| `new-book-plans/constitution.nibli` | the executable constitution, in [nibli](https://github.com/dhilipsiva/nibli) KR |
| `new-book-plans/full-society-ledger.json` | the reviewed canonical cross-domain routing source owning stable domain, claim, body, route, external-assumption and envelope IDs; its coverage, allocation, reader-navigation and Book 2 routing reports are projections, while sibling assurance cases retain their own reviewed sources |
| `new-book-plans/counterfactual/` | reviewed one-change constitution variants, so deletion, replacement and addition consequences are executed rather than argued |
| `book-1/` | the reader-facing derived chapters, each with a sidecar of pinned queries against the constitution |
| `new-book-plans/3-spine.md` | the computed chapter order — generated, not hand-written |
| `new-book-plans/assertion-surface-audit.md` | the generated inventory of derived relations and writable-premise risks |
| `new-book-plans/assertion-surface-contracts.json` | the reviewed authority, provenance, harm, challenge and risk contracts behind that audit |
| `new-book-plans/record-integrity-assurance-case.md` | the generated current/target argument for positive writes, effective absences, authorship, correction, witnessing, reconciliation, challenge and recovery |
| `new-book-plans/record-integrity-assurance-case.json` | the reviewed claims, evidence, premise coverage, defeaters, defaults and Book 2 assumptions behind that case |
| `new-book-plans/record-integrity-red-team.md` | the generated, executable flat-snapshot audit of release, adulthood, roster, relief and forgiveness harms, including full-source floor entitlement checks |
| `new-book-plans/record-integrity-red-team.json` | the reviewed route postures, snapshot deltas, expected results, limits and narrowness impacts behind that audit |
| `new-book-plans/amendment-semantics-audit.md` | the generated, executable audit separating Article 9's declared labels from candidate-source effects |
| `new-book-plans/amendment-semantics-audit.json` | the reviewed exact mutations, expected verdicts, limits and affected claims behind that audit |
| `new-book-plans/placement-exhaustiveness-audit.md` | the generated, executable severity/family/home placement matrix and mutation audit |
| `new-book-plans/placement-exhaustiveness-audit.json` | the reviewed axes, routes, source manifest, harmful mutations, limits and affected claims behind that audit |
| `new-book-plans/temporal-assurance-case.md` | the generated staged T1/T2/T3 transition, order, renewal and residual-liveness assurance record |
| `new-book-plans/temporal-assurance-case.json` | the reviewed temporal inputs, source/effect bindings, attacks, fresh-process pairs and narrowness ledger |
| `new-book-plans/full-society-ledger.md` | the generated full-society domain-and-layer routing report: declared axes, the five layers, domains, split coverage claims, bodies, routes, enum mapping and the stopping rule |
| `verify.sh` | the one check: a tiny build shim for the single native `rights-verify` binary |

```bash
./verify.sh          # everything, including pins, executable audits and counterfactuals
./verify.sh --quick  # schema/freshness checks; skips the executable suites
./verify.sh --emit-receipt new-book-plans/verification-receipts
./verify.sh --wait-for-lock 300 --emit-receipt new-book-plans/verification-receipts
./verify.sh --commit-gate <receipt> --transition audit|closure|tracker
./verify.sh --fingerprints assertion-surface
./verify.sh --fingerprints state-form
./verify.sh --fingerprints obligations
./verify.sh --fingerprints full-society-ledger
./verify.sh --refresh assertion-surface
./verify.sh --refresh record-integrity-assurance
./verify.sh --refresh record-integrity-red-team
./verify.sh --refresh amendment-semantics
./verify.sh --refresh placement-exhaustiveness
./verify.sh --refresh temporal-assurance
./verify.sh --refresh reader-evidence
./verify.sh --refresh state-form
./verify.sh --refresh obligations
./verify.sh --refresh full-society-ledger
./verify.sh --refresh constitutional-closure
```

`verify.sh` performs Cargo's incremental freshness check and then replaces
itself with `target/release/rights-verify`. All schema checks, report freshness,
negative controls, Nibli executions, locking, receipts, and commit gates run in
that one Rust process. Native fingerprint and governed refresh modes use the
same binary; numbered Python verifier files remain historical parity references,
not workflow subprocesses.

Under protocol v6 (2026-08-27), any semantic, executable, verifier, fixture,
engine-binding, or generated-artifact change requires one fully staged authoritative receipt.
A following audit, closure, or tracker-only commit uses `--commit-gate` only
when the receipt's heavyweight dependency manifest is byte-identical and the
transition-specific structural validator passes. Missing local evidence, a
merge, an intervening unclassified commit, or any unexpected path, mode,
engine, environment, or input change fails closed; the gate never launches a
silent full run.

The author-ratified 2026-08-30 workflow uses two speeds without creating two
standards. A coherent constitutional or assurance batch is decomposed into
small, uncommitted authoring/review slices. Quick, focused, fingerprint, and
governed refresh/check modes provide feedback while those bytes change.
`Drafted — not audited` is only a workflow state and carries no claim warrant.
Once the complete batch freezes, one fully staged full receipt gates its single
candidate commit, followed immediately by the mandatory audit and closure
successors. An optional immediate tracker successor may delete a completed item.
Intermediate semantic commits cannot share that receipt, and unrelated work is
never bundled merely to save a full run.

One exact `FS-SAU-42` forward recovery is defined by the scope-review protocol
for two already-published v5 audit epochs whose closure successors were
omitted. It accepts only the named commits, receipts, closed anchor, committed
bytes, and digest-bound local evidence; it neither searches history nor
establishes a reusable compatibility path.

Heavyweight verifier entry points share one Git-common-directory kernel lock.
Contention exits 75 with sanitised owner details unless `--wait-for-lock
SECONDS` supplies an explicit bounded wait. Native ledger and closure checks
and refreshes preserve the atomic immutable-input/final-reread contract.
`RIGHTS_VERIFY_JOBS=1` through `4` selects the sole heavyweight execution-lane
capacity; legacy per-family worker variables fail closed. Full execution first
runs reader evidence in canonical position, then validates a typed dependency
graph containing only the preflight-byte-captured live-pin, obligations, and
state-form plans. Their complete repository input bytes include an isolated
live-pin shell-precondition tree. Red-team, temporal, amendment, placement, and
ordinary counterfactual execution remain serial afterward in canonical order. The graph
buffers unchanged canonical output, stops new launches at failure, cancels
higher canonical work, and joins or reaps all started work. This is not a total
OS-thread limit: every active family has one joinable wrapper, and nested family
workers consume the declared weighted lanes. Wrappers, heavyweight lane workers,
memory-owning worker states, and managed child processes have separately watched
bounds; one lane is the current per-file fail-fast semantic and failure-selection
reference. At capacities three and four, live pins use all but one lane while
captured obligations and state-form work advance in the remaining lane.
Receipts bind the effective lane/allocation metadata, full-suite execution
start and finish, and the digest of local expanded evidence containing elapsed
time; per-family and per-job timings remain unbound local diagnostics. State-form
keeps its reviewed 64 main and 17 counterfactual shards; its full-graph plan
uses one lane while focused execution may use the configured capacity.

The binary links the Nibli engine crates from the adjacent source checkout, and
receipt emission binds that source revision and the exact `rights-verify` bytes.
For standalone manual queries, use release `nibli-pin` at or after engine commit
`4cb02aade43b394374c40e661907ad66df3af3fe`, never `nibli-host`.

It exits non-zero on the first failure and names the claim that stopped being true.
That includes a new or reclassified rule head, a changed admission or ground-fact
snapshot, an unreviewed producer/consumer route in the assertion surface, or a
premise that has drifted out of the record-integrity case. The bounded red-team
reproduces selected current harms in constructed snapshots; it does not attribute
forgery, withholding or deletion. The assurance case's current verdict remains
deliberately **not established**: verification proves consequences from supplied
snapshots, not that a deployed record is authentic, complete, append-only or live,
and not that the checker authenticates its own source or toolchain. The amendment
audit manually applies exact candidate mutations and proves their bounded
consequences. It does not show that `become` enacted them, that a declared target is
true, or that a source transition was authorised. The placement audit generates the
current routing combinations for confined, affirmatively free, and person-only
subjects. When the full verifier runs, it rejects missing, conflicting, reversed, or
roster-only non-carceral housing outcomes. Its mutation probes positively establish a
placement report before checking alarm silence. A cold one-pin probe also derives each
row's opaque shelter entitlement from that row's complete standing route, without a
standing overlay. The audit adds no runtime placement alarm and does not prove that
housing or a reported placement exists in the world.

A second book—how different current societies could reach and operate the audited
destination within declared, versioned reference envelopes, organisationally and
technically—is planned and not started.

## Licence

Deliberately mixed: prose CC-BY-4.0, code MIT OR Apache-2.0, data CC0, and everything
committed before that decision irrevocably CC0 under the root `LICENSE`. See
[`LICENSING.md`](LICENSING.md) before adding files.
