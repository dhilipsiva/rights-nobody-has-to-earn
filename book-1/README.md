<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The Rights Nobody Has to Earn

This directory is the reader-facing Book 1 projection of the principal formally
audited constitutional specification. It is not the specification itself and
cannot override, complete, or upgrade it. It contains the epigraph, opening note,
derived chapters in the ruled reading order, Part V, and method part. The
numbered chapters between the opening note and Part V are the derived
spine—their order is editorial, *engines before breaks*, ruled on 2026-09-16 and
recorded in `contents.json`, of which the filename prefix is a checked
projection; their claims derive from the constitution and are pinned by the
`*.pins.nibli` files beside them. Its companion volume is Book 2, *What
It Would Take*: this book is the destination, that one is the road. Exactly three elements are exempt from the
derivation gate and labelled so in their own text: the opening note, Part V, and
the method part.

A chapter lands with the rules it renders: its paired pins file, its case in
`../tests/pins/suites.json`, its rows in the coverage ledger and its entry in
the opening note's contents, verified by `../verify.sh` and the development
tests. The receipt, audit and closure chain that once gated this was retired
on 2026-09-12.

Two files are deliberately unnumbered — `epigraph.md` and `method.md`.
The epigraph is a poem; the optional method quotes the machinery kept outside
the ordinary reader chapters. Both remain in the manifest's reading sequence.
Two subdirectories are
not chapters and are outside the reading order and the length measurement:
`appendix/` carries the planning record (decisions, contracts, briefs, maps —
the former `new-book-plans/book-1-*` files with that prefix dropped), and
`source/` carries the formal source and everything beside it (the rest of the
former `new-book-plans/`, moved by flat rename on 2026-09-17).
Run `../verify.sh` to check every pinned claim against the constitution.

## Read or assemble the book

Start with [the epigraph](epigraph.md), then [the opening note](00-opening-note.md).
The opening's annotated contents describe the route through the book;
[contents.json](contents.json) supplies the same order to the assembler.

From the repository root, with Python and [uv](https://docs.astral.sh/uv/) installed:

```bash
uv run --with playwright==1.63.0 python -m playwright install chromium
uv run tools/build_book.py
```

The first command installs the browser used for PDF printing. Chromium also
needs its platform's browser libraries. An existing compatible Chromium can
be selected with `--browser-executable /path/to/chromium`.

The second command produces `book-1-review.html`, `book-1-review.epub` and
`book-1-review.pdf` in `output/book-1/`. Use `--no-pdf` to build HTML and EPUB
without a browser, or `--output-dir PATH` to choose another output directory.
The script declares its exact Python dependencies. Generated review copies
are ignored by Git; rebuilding reads the current ordered source files.

Add `--sample` to assemble chapters 1, 5, 8, 21 and 31 as
`book-1-sample.html`, `book-1-sample.epub` and `book-1-sample.pdf` instead.
The [publisher proposal](../submission/README.md) explains the selection.
The sample keeps the original chapter numbers; references outside the
selection open their public repository locations.

HTML provides a linked contents list, keyboard skip link and labelled table
regions. EPUB carries the chapter order and nested contents. PDF includes
page numbers, a linked contents list and bookmarks. All three embed the
Tamil font used by the epigraph. In the full copy, chapter links stay within the edition;
links to formal files and the claim registry open their repository locations.
Those repository links follow `main` and do not identify immutable source.

Inspect generated formats after layout or navigation changes. The assembler
checks local destinations and headings; it does not establish actual-user
accessibility, comprehension or publication readiness. The old pilot files
under `output/pdf/` and `source/15-pilot-reader-artifacts.py` are historical;
they are not the current reading edition or an additional verification gate.
Constitutional verification remains `../verify.sh`, with prose consistency
and rendering reviewed separately.

To suggest a correction or co-authored addition, follow the
[contribution guide](../CONTRIBUTING.md). It covers prose, evidence, formal
changes, credit and independent adaptations.

## Licence

The new reader prose is licensed under the Creative Commons
Attribution 4.0 International licence (CC-BY-4.0). The full text is in
[LICENSE-CC-BY](LICENSE-CC-BY), and assembled copies carry the declaration
in their front matter. The embedded Tamil font has its own
[SIL Open Font License](../tools/book_assets/OFL-NotoSerifTamil.txt).

The formal source, pin files, code, registry and carried historical material
retain their applicable terms; see [../LICENSING.md](../LICENSING.md) for the
repository-wide licence map. Moving a file here does not change its licence.
