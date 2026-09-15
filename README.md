# The Rights Nobody Has to Earn

*A worked design for a society, with its formal claims made executable.*

A worked design for a society in which a short list of basic things — safety, food,
shelter, care, learning, speech, belief, company — is owed to every person with no
qualifying condition. You do not have to work, contribute, belong, register or behave.

The executable constitution is the formal source. Book 1 explains its consequences
in ordinary language; its chapter pins test the corresponding formal claims.
Checking whether the prose faithfully explains those rules is a separate review,
not something the verifier certifies. Book 2 is planned to describe operation and
transition within declared local conditions without silently changing that
destination.

## What is here

| Path | Purpose |
|---|---|
| `new-book-plans/constitution.nibli` | The executable constitution, in [Nibli](https://github.com/dhilipsiva/nibli). |
| `book-1/` | Reader-facing chapters and their companion `.pins.nibli` tests. |
| `tests/pins/suites.json` | The executable case inventory: live rules, fixture inputs, pin files, and explicit counterfactual edits. |
| `tests/pins/` | Record, amendment, placement, temporal, state-form, obligations, and other substantive examples. |
| `new-book-plans/state-form-source.json` | State-form authoring input. |
| `new-book-plans/obligations-source.json` | The protected claim names used by the obligations authoring tool. |
| `new-book-plans/integrity-source.json` | Democratic and administrative integrity findings, kinds, and legal consequences. |
| `new-book-plans/statistics-source.json` | Bounded statistical uses, privacy, aggregate equality evidence, challenge and correction. |
| `new-book-plans/book-1-amendment-enactment-contract.md` | Exact-change authority, publication, effective-version, conflict, replay and remedy boundaries. |
| `new-book-plans/book-1-mobility-and-plurality-contract.md` | Mobility protections, differentiated collective rights, exact consent/consultation and external no-evasion. |
| `new-book-plans/book-1-non-carceral-justice-contract.md` | Accessible justice, actual court-bound remedies, voluntary restoration and release continuity. |
| `new-book-plans/book-1-public-safety-contract.md` | Separated protective powers, non-derogating emergencies, humane holding, external limits and exact review. |
| `src/amendment_host.rs` | Separate trusted-input, in-memory enactment reference model; no real authentication or deployment. |
| `new-book-plans/3-spine.md` | The chapter-order projection generated from the engine's dependency layers. |
| `verify.sh` | Run the pins and contradiction checks. |
| `generate.sh` | Explicitly regenerate authored rule, fixture, pin, or spine outputs. |

## Verify

Clone the Nibli source repository beside this checkout, as required by the Cargo
path dependencies. From this repository's root:

```bash
./verify.sh
./verify.sh --list
./verify.sh --only book-1/08-what-you-are-owed.pins.nibli
```

The default command runs all substantive pins, including counterfactuals, and
checks the live constitution and ordinary scenario snapshots for contradictions.
Deliberately changed or inconsistent counterfactual worlds retain their explicit
pin expectations; they are not required to satisfy the live world's
contradiction-free condition. A failed pin, engine error, timeout, or incomplete
contradiction check fails verification. An expected `UNKNOWN` pin is different
from an incomplete contradiction check.

The suite inventory declares each case's base, extra facts, pin files, and any
source edits. Counterfactual edits are applied to the current source, in order;
a missing or ambiguous edit target is an error. Cases have isolated knowledge
bases, and scoped pin controls do not leak into later queries.

`verify.sh` incrementally builds and runs the release Rust verifier using the
adjacent Nibli engine source. `RIGHTS_VERIFY_JOBS=1` through `4` selects the worker
count; the default uses up to four available cores. Verification does not compute
source hashes, issue receipts, inspect Git history, enforce audit/closure commits,
or validate administrative reports. There is no separate quick assurance mode
and no cached verdict that lets a changed book skip its tests. The focused
`--only` command is for feedback on one pin file, not a whole-book pass.

On Linux GNU targets the verifier bundles jemalloc for its allocation-heavy
worker pool. It needs no system jemalloc installation or `LD_PRELOAD` setting;
the first build compiles the bundled C library using a C compiler and Make
(available in the Nibli development shell). Other targets retain Rust's default
allocator. The explicit authoring executable is unchanged.

Latest full run measured on 2026-09-15 with four workers and the release binary
already built: all 77,902 pins across 13,566 cases passed, with complete formal
contradiction scans and no findings, in 824.25 seconds (13m44.25s); nine
existing known-defect pins still reproduce. User CPU was 3,174.52 seconds,
system CPU 77.29 seconds, utilisation 394%, and peak resident memory
21,626,012 KiB, with no major page faults and no swaps. Other machine activity
was not controlled.

This **does not** meet the under-five-minute target. The earlier 275.04-second
(4m35.04s) result was measured on the 4,190-case inventory that preceded the
ecological family and is not a current timing. The
[performance notes](new-book-plans/nibli-performance-candidate.md) record where
the current run spends its time and which leads remain. Every timing here is an
observation, not a cached verification result or a gate on later edits.

## Author

Generation is separate from verification and happens only when requested:

```bash
./generate.sh state-form
./generate.sh obligations
./generate.sh integrity
./generate.sh statistics
./generate.sh amendment
./generate.sh mobility
./generate.sh justice
./generate.sh public-safety
./generate.sh spine
```

The state-form, obligations, integrity, statistics, amendment, mobility,
justice and public-safety commands
update their constitution rule blocks, companion pins, and executable fixtures
from their authoring inputs. Review those changes before running verification.
Mobility's direct-effect cards live in `src/authoring/mobility_contracts.rs`;
its record and case authoring code is beside them. It adds no operational
border, emergency, treaty, identity or population-scoring system.
Justice's direct-effect cards live in `src/authoring/justice_contracts.rs`.
Its court-dependent effects consume the actual state-form power and raw
witnesses; they create neither new court holders nor coercive instruments.
Public safety's cards and composition helpers live in
`src/authoring/public_safety*.rs`. They consume the actual justice, mobility,
consent and exit interfaces and keep legal permission separate from an
independently reported act. No command operates a force or authenticates evidence.
The spine command refreshes the chapter-order projection;
chapter placement and prose remain review work. Ordinary chapter pins and test
fixtures can be edited directly.

The amendment host has separate development tests:

```bash
cargo test --bin amendment-assurance
cargo test --release --bin amendment-assurance
```

The first runs the fast host controls. The release run also executes the full
constitution-to-certified-candidate-to-effective-query integration case; debug
inference makes that case unusually slow. Both use the same checks. These tests
are not added to `verify.sh`. For an explicit trusted local replay, run
`cargo run --release --bin amendment-assurance -- <scenario.json>`; its typed
input is defined by `Scenario`, `Event` and `Review` in `src/amendment_host.rs`.
It reads the input once and changes only in-memory state. It compares full
source bytes, never source hashes, and cannot authenticate the supplied evidence
or perform real publication/deployment. Book 2 owns those operations.

A passing suite establishes the tested consequences of supplied rules and records.
It does not authenticate outside records, prove institutional action or delivery,
validate every sentence of prose, or prove the absence of untested semantic bugs.
The record scenarios reproduce selected harms without identifying who forged,
withheld, or deleted a real record. Amendment cases manually apply source edits;
they do not show that an amendment label enacted those edits. Placement cases
test the declared routing combinations, not whether housing exists in the world.

## Licence

Deliberately mixed: prose CC-BY-4.0, code MIT OR Apache-2.0, data CC0, and everything
committed before that decision irrevocably CC0 under the root `LICENSE`. See
[`LICENSING.md`](LICENSING.md) before adding files.
