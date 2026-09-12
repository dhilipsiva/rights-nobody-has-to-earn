<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Executable book cases

`suites.json` is the inventory used by `./verify.sh`. It lists all substantive
pin cases, including chapter claims, supplied-record scenarios, and intentional
counterfactuals. Verification runs their pins and the applicable contradiction
checks; prose consistency remains a separate review.

## Running

Run these commands from the repository root:

```bash
./verify.sh
./verify.sh --list
./verify.sh --only tests/pins/records/free_adam/expect.pins.nibli
```

`--only` selects the listed pin file wherever the inventory uses it, with each
matching case's base and fixtures. It reports `PARTIAL`: it is feedback on that
selection, not a whole-book pass. Use `--list` to find executable pin paths.

The runner reads the selected base sources, fixtures, and pin files before
starting workers. The result describes those captured inputs, not later edits.
This is a per-file startup capture, not an atomic filesystem snapshot; rerun
after changing inputs. Explicit shell preconditions are an exception: they run
in the repository working directory and can read its live files.

Compiled bases and rule plans are reused only inside that process. There is no
source-hash calculation, persistent verdict cache, receipt, Git-history gate,
or generated-report freshness requirement. Cargo's ordinary incremental build
cache is separate from the test results. `RIGHTS_VERIFY_JOBS=1` through `4`
controls the fixed worker pool.

## Editing the inventory

All file paths are repository-relative. A base has exactly one of `path` (a
Nibli source file) or `base` (another named base), plus optional `edits`.
The `live` base is `new-book-plans/constitution.nibli`; use the full live source
unless a case deliberately tests a named source variant.

A case looks like this existing record example:

```json
{
  "id": "records/free_adam",
  "base": "live",
  "fixtures": ["tests/pins/records/free_adam/fixture.nibli"],
  "pins": ["tests/pins/records/free_adam/expect.pins.nibli"],
  "scan": true
}
```

Case IDs must be unique and `pins` must be nonempty. `fixtures` and `edits`
default to empty arrays, `scan` defaults to `true`, and `allow_shell` defaults
to `false`. Unknown fields, missing inputs, unknown bases, and cyclic base
chains are errors.

Fixtures contain ordinary, period-terminated Nibli statements, not queries or
pin directives. They are loaded before each pin file in that case. Every pin
file starts with isolated state; assertions within a file retain their original
sequential meaning. Use `:accept-scoped` for temporary controls, and `:accept`
only when later pins intentionally depend on the accepted statement. Keep
`:expect-pins` accurate, and preserve `:defect` annotations when a test records
a known flaw. Do not replace an unexpected verdict with `UNKNOWN` merely to
make a test pass.

`allow_shell: true` explicitly permits that case's `:require` commands. These
execute real shell commands with the verifier user's permissions; reserve them
for reviewed repository preconditions, and do not enable them for untrusted
pin files. Most cases need no shell commands.

### Source edits

An edit is `{ "before": "...", "after": "..." }`. Edits apply in order:
parent-base edits first, then child-base edits, then case edits. A nonempty
`before` must match exactly one contiguous block of source statement lines;
blank lines, comment-only lines, and leading/trailing line whitespace are
ignored. Statement content remains exact, so changing a rule requires reviewing
its affected counterfactual edits. An empty `after` deletes the matched block;
an empty `before` appends the supplied statements. No complete counterfactual
constitution file needs to be copied or refreshed.

### Contradiction boundaries

Keep `scan: true` for the live constitution and ordinary scenario cases. The
runner scans the prepared base and the completed scenario state when fixtures
or pins add statements. Temporary scoped controls are restored before that
final scan; this is not a scan of every possible intermediate state.

Use `scan: false` only for a deliberately altered or inconsistent test world
whose explicit pins describe the intended consequences. Document that purpose
beside the pins; disabling scanning is not a repair for a live contradiction.
Scanner findings and incomplete checks both fail verification. An explicitly
expected `UNKNOWN` query does not make an incomplete scan acceptable. The
constitution's ordinary `false` and `contradict` predicates are not themselves
scanner findings.

A clean scan describes the represented formal model and constraints. It is not
a proof that the English book is consistent, the model covers every situation,
or outside records and institutions behave as described.

## Authoring and aggregate files

```bash
./generate.sh state-form
./generate.sh obligations
./generate.sh spine
```

State-form and obligations generation explicitly updates their constitution
blocks, aggregate pin files, extracted executable cases, and inventory entries.
It preserves unrelated inventory entries and their settings. Review the result
before verifying. Pending authoring-source edits are not enacted by `verify.sh`.
Spine generation updates the chapter-order projection only.

Some original aggregate pin files remain under `new-book-plans/` for authoring
and inspection. Their executable examples are split into isolated case assets
here, including state-form, obligations, and economic direct-effect cases.
The inventory, not a filename suffix or directory scan, determines what runs.
Editing an aggregate that is not listed in the inventory does not change the
executed cases: regenerate its supported family or edit the actual case assets.
Chapter files and many counterfactual pin files are still listed directly and
execute from their original paths.

New `.nibli` and `.pins.nibli` assets should carry
`# SPDX-License-Identifier: MIT OR Apache-2.0`, unless an existing source has
an explicit licence that must be preserved.
