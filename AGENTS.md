# Repository Guidelines

## Authority, Scope & Structure

`CLAUDE.md` describes the project. Read it before work. The root `TODO.md`
is the ordered revision backlog created on 2026-09-24 from the outside revision
plan in `new-reviwes/`; its predecessor, requested on 2026-09-18, is complete.
The author's rulings D1–D9 of 2026-09-24 on that backlog's reserved questions
are recorded in `CLAUDE.md` with the items implementing each: D5–D8 have
landed, D2 and D3 hold for Parts I–III, and the rest wait for their items. The
2026-09-16 rebuild's rulings, measurements and chapter table are recorded in
`CLAUDE.md` and `book-1/contents.json`. `book-2/TODO.md` is Book 2's tracker
and is collection-only until Book 1 ships at Gate C.
The constitution and substantive Nibli pins live under `book-1/source/`;
`book-1/` is their reader projection, with chapter/pin pairs. Its labelled
opening note (whose map, glossary and index sit at the back in `reference.md`),
Part V, and method are the non-derived exceptions; a derived chapter may close
with one argument section headed `## Argument:` (ruling D2), which the checks
read as argued text, and each Part may open with a labelled documented case
(ruling D3); those of Parts I–III are written and the others are reserved in
`contents.json`. Book 2 remains inactive until Gate C and owns operation and
transition. Preserve the legacy manuscripts until their TODO harvest. Keep
epigraph and method unnumbered.
`ui/` is the Book 1 reader and live companion, published at
https://dhilipsiva.dev/rights-nobody-has-to-earn/ by the separate website
repository from this repository's `main`; derived chapters never name it.

All reader-facing parts describe the current design of their edition, including
the opening, Part V and method. Keep the book's development and revision history
in Git and repository decision records, not in the reader narrative. Follow the
2026-09-18 current-design ruling in `CLAUDE.md` on every future revision.

Resolve design failures before defending them: try substantive repairs and
verify a sound resolution. A fallback defense requires demonstrated inability
to resolve the issue under named, justified constraints and an adequate account
of alternatives, costs and safeguards. Disclosure or a failed attempt is not
completion. Follow the 2026-09-18 resolve-before-defending ruling in `CLAUDE.md`.

## Verification — author decision, 2026-09-12

Verification means executing the substantive Nibli pins and checking the
loaded formal model for contradictions. Review Markdown consistency separately
when changing prose. This replaces every earlier receipt, fingerprint, audit,
closure-successor, staged-candidate, and multi-hour batch requirement.

```bash
./verify.sh                 # all substantive pins and contradiction checks
./verify.sh --list          # show the execution inventory
./verify.sh --only book-1/02-what-the-record-may-say.pins.nibli
./generate.sh state-form    # explicit authoring, separate from verification
./generate.sh obligations
./generate.sh integrity
./generate.sh statistics
./generate.sh amendment
./generate.sh mobility
./generate.sh justice
./generate.sh public-safety
./generate.sh ecology
./generate.sh spine
```

Use `RIGHTS_VERIFY_JOBS=1..4` for the fixed worker pool. Compilation is
incremental; no verification result is persisted or skipped. The performance
target is a complete run under five minutes with the release binary built.
Report measured timing rather than assuming the target has been met.

`tests/pins/suites.json` routes ordinary pin files to the actual constitution,
isolated fixtures, and explicit counterfactual transformations. Preserve every
substantive expectation, including refusal, scoped controls, defect markers,
stateful sequences, and existing trusted shell preconditions. A missing file,
ambiguous source edit, malformed pin, or incomplete contradiction check fails.
Only explicitly declared counterfactuals use a modified constitution.

Keep semantic authoring generators small and explicit. They may generate rules
and cases from their source JSON; verification checks the resulting Nibli.
Do not add hashes, provenance receipts, schema/freshness audits, Git-history
validators, or report gates. Tests of the runner belong in its development test
suite. The retained historical plans and decisions describe previous tooling;
they impose no retired verification workflow.

The amendment host is a separate, trusted-input, in-memory reference model.
Its byte comparisons and transition tests are development checks, not hashes,
authentication, publication, deployment, or extra routine-verification gates.

Make coherent, reviewable changes, run relevant focused checks while editing,
then the complete verifier. Ordinary commits need no receipt or administrative
successors. Delete completed tracker items when their work is complete.

## Editing, Testing & Naming

Match Markdown hierarchy and `NN-kebab-case.md`/`.pins.nibli` pairs. Write controls as `:accept-scoped`; use `:accept` only when the accepted statement is a later premise. State the rule producing a count, not a counted design claim. Add a primary source and URL with every statistic or named study. Python uses four spaces; new code needs `SPDX-License-Identifier: MIT OR Apache-2.0`.

## Commits, Pull Requests & Licensing

Author decision, 2026-09-13: automatically commit and push after completing
each TODO item's implementation and required checks; no separate reminder is
needed. Work on one item at a time, then repeat through the backlog. The author
pre-approves the agent's recommended suggestions, including proposed prose,
and requests no questions. This supersedes per-passage approval pauses: record
session-drafted prose as author-approved under this delegated approval, retain
the exact text, and preserve every substantive and verification constraint.
Do not invent personal testimony, empirical evidence, or successful external
operations. Record genuine blockers without claiming completion. Include only
the item's coherent changes, preserving unrelated work.

Make coherent commits including affected rules, tests, and prose. Use
`<area>: <outcome>` subjects and explain why in a ~72-column body. No receipt,
audit, closure, or separate tracker commit is required. Pull requests summarize
the claim and validation; screenshots are only for rendered visual changes.

Read `LICENSING.md` before adding files. New prose is CC-BY-4.0, code is MIT OR Apache-2.0, registry claims are CC0, and data snapshots can carry upstream terms. Legacy pre-decision material remains CC0 under the root `LICENSE`.
