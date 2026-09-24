<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Contributing to The Rights Nobody Has to Earn

Corrections, objections, better evidence and proposed additions are welcome.
You can [open an issue](https://github.com/dhilipsiva/rights-nobody-has-to-earn/issues)
with a file, passage and explanation, without knowing Nibli or running the
tools. For an edit, fork the repository, make a branch and open a
[pull request](https://github.com/dhilipsiva/rights-nobody-has-to-earn/pulls)
against `main`. Keep each proposal focused on one problem. The current
manuscript is readable at
[dhilipsiva.dev/rights-nobody-has-to-earn](https://dhilipsiva.dev/rights-nobody-has-to-earn/),
and each entry in its companion's dossier carries an objection link that
opens a prefilled issue.

Describe what is wrong, the proposed result, the evidence and the checks you
ran. Say plainly when a check was not run or failed. An issue can report an
unresolved problem; it does not need to claim a repair.

## Find the right file

| Change | Start here |
|---|---|
| Wording or navigation | The chapter in [book-1](book-1/README.md); [contents.json](book-1/contents.json) gives its order. |
| An empirical claim or citation | The passage and its footnote, [registry/claims.json](registry/claims.json), and the [registry guidance](registry/README.md). |
| A constitutional consequence | [constitution.nibli](book-1/source/constitution.nibli), the chapter's `.pins.nibli` file and its case in [suites.json](tests/pins/suites.json). |
| Generated rules or cases | The owning JSON under `book-1/source/` or generator under `src/authoring/`; the [authoring instructions](README.md#author) identify the commands. |
| A rendering problem | [build_book.py](tools/build_book.py), [book.css](tools/book_assets/book.css) and the affected source passage. |
| Operation or transition | [Book 2's tracker](book-2/TODO.md), which remains collection-only until Book 1's release gate. |
| The website or its companion | [ui](ui/README.md); the [published section](https://dhilipsiva.dev/rights-nobody-has-to-earn/) is rebuilt from `main` by the separate website repository. |

Read the current instructions at the start of [CLAUDE.md](CLAUDE.md) and in
[AGENTS.md](AGENTS.md). Historical decisions and scripts do not restore retired
verification requirements. Preserve the legacy manuscripts and carried archive.

## Keep a change faithful and checkable

**Prose.** Chapters 1–30 explain the formal design in ordinary language.
Change the rules and pins first if you want a different consequence. The
labelled opening, Part V and optional method can make their own arguments;
they still describe the current edition. Keep development history in Git and
decision records. Do not invent testimony, feelings for the test characters,
evidence or successful real-world operations. Preserve the child tests and
their documented exemptions, and keep the epigraph and method unnumbered.

If you rename a heading or a passage used as a reference, update its existing
entry in [reader coverage](book-1/source/reader-coverage-source.json) and any
affected locators. These records follow the text; they are not substitutes for
reading it. Prefer a precise claim and its limit to a stronger unsupported one.

**Evidence.** Give every statistic or named study a primary source and URL,
the particular version and page/table or other locator, units, population,
period and material limitations. Keep the prose, footnote, registry entry and
existing claim bindings consistent. Distinguish a study's finding from your
interpretation. A retrieval date records an actual check, not an assumption
that a source remains current. Data snapshots retain their upstream terms.

**Formal changes.** Include a case that demonstrates the failure, the proposed
repair, and controls for legitimate behavior that must remain. Edit an owning
generator or JSON when the output is generated, then run its explicit
`./generate.sh` command. Preserve refusals, stateful sequences, counterfactuals
and trusted shell preconditions. Use `:accept-scoped` for temporary controls;
use `:accept` only when a later query intentionally depends on that premise.
A passing defect expectation reproduces a defect; it does not resolve it.
Constitutional failures stay with Book 1 until repaired or adequately defended
under the existing resolve-before-defending rule.

## Checks

The [setup instructions](README.md#verify) establish the adjacent Nibli checkout.
From the repository root, use the affected companion file for focused feedback:

```bash
./verify.sh --only book-1/05-whether-it-arrived.pins.nibli
```

Review the prose separately. Existing development checks for coverage, links
and claim bindings can be run with:

```bash
RUST_MIN_STACK=67108864 cargo test --release --locked --bin generate reader_coverage
RUST_MIN_STACK=67108864 cargo test --release --locked --bin generate reference_integrity_tests
RUST_MIN_STACK=67108864 cargo test --release --locked --bin generate claim_discipline_tests
```

For machinery changes, run its relevant development tests. For changes to
layout or navigation, [rebuild the review copies](book-1/README.md#read-or-assemble-the-book)
and inspect the affected HTML, EPUB and PDF output. A Markdown check cannot
establish correct rendered layout.

Before an accepted revision is completed, run `./verify.sh` for all substantive
pins and contradiction checks; report its actual result and elapsed time.
The maintainer can perform that run for a contributor who cannot. A focused
pass is partial. Neither a full pass nor AI agreement establishes outside
truth, institutional performance or independent reader review.

## A small example

An illustrative proposal makes the evidence explicit in
[chapter 5](book-1/05-whether-it-arrived.md):

```diff
-from Provender. That alone produces no food-delivery conclusion. It then gives
+from Provender. A receipt alone produces no food-delivery conclusion. It then gives
```

A useful PR description would explain that “a receipt” names what is
insufficient, cite the Marisol sequence in the companion pins, and report the
focused command above. Read the surrounding paragraph to confirm that adding
the authorised independent witness still produces the food conclusion. State
whether the complete verifier was run. This example is a proposed wording
change, not a claim that a contribution has been submitted or accepted.

## Credit, licences and editorial responsibility

State the name or handle you want credited and identify what you contributed.
Accepted changes retain contributor attribution in Git; shared drafting can
also receive a co-author trailer or an agreed passage/chapter credit. Propose
that credit in the PR for a substantial co-authored addition. A correction does
not automatically make its contributor an author of the entire book. Disclose
material AI assistance and check the resulting claims yourself; AI output is
not independent evidence or endorsement.

Contribute material you have the right to offer under the applicable terms.
The [licence map](LICENSING.md) distinguishes new book prose (CC BY 4.0), code
(MIT OR Apache-2.0), the constitution and registry claims (CC0), upstream data
and fonts, and legacy material already dedicated to CC0. Preserve notices
and source attributions.
The [CC BY 4.0 terms](https://creativecommons.org/licenses/by/4.0/) allow reuse
and adaptation with attribution, a licence link and an indication of changes;
existing grants are not withdrawn by later editorial decisions.

The maintainer integrates accepted proposals into the maintained edition and
remains responsible for its coherence. Review can request changes or decline a
proposal. An independent fork or adaptation can make different choices under
the applicable licences. Retain the credits and change notices those terms
require. For clarity, identify your edition as independent, without implying
this project's endorsement. Repository inclusion does not promise publisher
acceptance or any particular publishing agreement.
