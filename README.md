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

Latest full run measured on 2026-09-12 with four workers and the release binary
already built:
all 6,305 pins across 1,160 cases passed, with clean formal contradiction scans,
in 746.17 seconds (12m26s), including the integrity and statistics cases.
An earlier same-day run of the same formal inputs took 458.52 seconds (7m39s).
The under-five-minute full-run target is not yet met; the remaining cost is
primarily Nibli loading and inference. These are timing observations, not cached
verification results or gates on later edits.

## Author

Generation is separate from verification and happens only when requested:

```bash
./generate.sh state-form
./generate.sh obligations
./generate.sh integrity
./generate.sh statistics
./generate.sh spine
```

The state-form, obligations and integrity commands update their constitution
rule blocks, companion pins, and executable fixtures from their authoring
inputs. Review those changes before
running verification. The spine command refreshes the chapter-order projection;
chapter placement and prose remain review work. Ordinary chapter pins and test
fixtures can be edited directly.

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
