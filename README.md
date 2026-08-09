# The Rights Nobody Has to Earn

*A design for a society worked out to the point where it catches its own failures.*

A worked design for a society in which a short list of basic things — safety, food,
shelter, care, learning, speech, belief, company — is owed to every person with no
qualifying condition. You do not have to work, contribute, belong, register or behave.

The design is written as a formal constitution, and the book's chapter order is computed
from it rather than chosen. Nothing goes into the book that the constitution does not
derive. That is also how the book finds its own defects, and it reports them: most of
what is owed never actually arrives, and the protection that does exist covers
imprisonment and stops there.

## What is here

| | |
|---|---|
| `book-1/` | the chapters, each with a sidecar of pinned queries against the constitution |
| `new-book-plans/constitution.nibli` | the constitution, in [nibli](https://github.com/dhilipsiva/nibli) KR |
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
| `new-book-plans/full-society-ledger.json` | the reviewed canonical full-society source owning stable domain, claim, body, route, external-assumption and envelope IDs; every coverage, assurance, reader and Book 2 view is a projection of it |
| `new-book-plans/counterfactual/` | reviewed one-change constitution variants, so deletion, replacement and addition consequences are executed rather than argued |
| `verify.sh` | the one check |

```bash
./verify.sh          # everything, including pins, executable audits and counterfactuals
./verify.sh --quick  # schema/freshness checks; skips the executable suites
python3 new-book-plans/9-record-integrity-red-team.py --check --execute
python3 new-book-plans/10-amendment-semantics.py --check --execute
python3 new-book-plans/11-placement-exhaustiveness.py --check --execute
python3 new-book-plans/12-temporal-assurance.py --check --execute
```

Use release `nibli-pin` at or after engine commit
`4cb02aade43b394374c40e661907ad66df3af3fe`; earlier builds may fail to complete
opaque entitlement checks whose ground subject is itself derived.

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

A second book — how you would actually build this, organisationally and technically — is
planned and not started.

## Licence

Deliberately mixed: prose CC-BY-4.0, code MIT OR Apache-2.0, data CC0, and everything
committed before that decision irrevocably CC0 under the root `LICENSE`. See
[`LICENSING.md`](LICENSING.md) before adding files.
