# Repository Guidelines

## Authority, Scope & Structure

`CLAUDE.md` is authoritative. Before work, read it, `TODO.md`, and `tmp.txt` (draft context only). `book-1/` contains numbered chapter/`.pins.nibli` pairs. Except for its labelled opening note, Part V, and method, prose must derive from `new-book-plans/constitution.nibli`; keep it jargon-free. Roadmaps, scaling, and implementation belong in book 2, which stays inactive. Preserve legacy `book.md`/`manifesto.md` until the TODO harvest.

`new-book-plans/` owns the constitution, generated spine/audits, reviewed JSON contracts, bounded record-snapshot red-team, and counterfactuals. `registry/` holds claims, snapshots, checks, and fetchers. Keep `book-1/epigraph.md` and `method.md` unnumbered.

## Validation & Development Commands

Run from the repository root:

```bash
./verify.sh --quick   # schema/freshness; skips executable suites
./verify.sh           # authoritative suite; required before every commit
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
python3 new-book-plans/13-full-society-ledger.py --check
python3 registry/check.py
```

Use release `nibli-pin --kb` at or after `4cb02aade43b394374c40e661907ad66df3af3fe`, never `nibli-host`. Omit `--check` only to regenerate. Edit reviewed JSON, never generated reports or spine blocks. After a rule/fact change, run `7-assertion-surface.py --fingerprints`, review, then copy candidate digests. Refresh reviewed digests in this order: assertion ledger (7), assurance source (8), red-team source (9), amendment and placement sources (10/11), then temporal source (12). The full-society ledger (13) sits off that chain — it digest-binds only the assurance-portfolio and full-society-boundary decisions and re-reads the sibling reviewed JSONs live at `--check` — so refresh it when either bound decision changes, and expect it to fail when a sibling adds a reviewed enum value with no mapping row. Generate reports 9 and 12 before rendering report 8 because its reviewed references name those outputs; then generate/check reports 8, 10, and 11. Evidence roles may not relabel a gap as assurance. After every constitution edit, comments included, regenerate counterfactuals and run the full verifier.

Script 10 manually applies candidates and does not prove enactment. Script 11 rejects conflicts with the current routing matrix when run but adds no runtime placement alarm or housing-delivery evidence. Script 12 proves bounded supplied-record safety, not outside clock, publication, storage, or institutional liveness.

## Editing, Testing & Naming

Match Markdown hierarchy and `NN-kebab-case.md`/`.pins.nibli` pairs. Write controls as `:accept-scoped`; use `:accept` only when the accepted statement is a later premise. State the rule producing a count, not a counted design claim. Add a primary source and URL with every statistic or named study. Python uses four spaces; new code needs `SPDX-License-Identifier: MIT OR Apache-2.0`.

## Commits, Pull Requests & Licensing

Make one chapter or section change per content commit. Use `<area>: <outcome>` subjects and explain why in a ~72-column body. Close TODOs separately with `Tracker: <what landed> (<content SHA>)`. Pull requests summarize the claim, validation, regenerated artifacts, and tracker item; screenshots are only for rendered visual changes.

Read `LICENSING.md` before adding files. New prose is CC-BY-4.0, code is MIT OR Apache-2.0, registry claims are CC0, and data snapshots can carry upstream terms. Legacy pre-decision material remains CC0 under the root `LICENSE`.
