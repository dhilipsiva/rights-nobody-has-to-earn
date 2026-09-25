<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The Rights Nobody Has to Earn

*A worked design for a society, with its formal claims made executable.*

This directory is the reader-facing Book 1 projection of the principal formally
audited constitutional specification. It is not the specification itself and
cannot override, complete, or upgrade it. It contains the epigraph, opening note,
derived chapters in the ruled reading order, Part V, the method part, and the
map, glossary and index at the back. The
numbered chapters between the opening note and Part V are the derived
spine—their order is editorial, *engines before breaks*, ruled on 2026-09-16 and
recorded in `contents.json`, of which the filename prefix is a checked
projection; their claims derive from the constitution and are pinned by the
`*.pins.nibli` files beside them. Its companion volume is Book 2, *What
It Would Take*: this book is the destination, that one is the road. Three elements are exempt from the
derivation gate and labelled so in their own text: the opening note, Part V, and
the method part. The opening's map, glossary and index sit at the back in
`reference.md`. Ruling D2 lets a derived chapter close with one argument
section, headed `## Argument:`, which every derived-chapter check reads as
argued text; ruling D3 lets each Part open with a labelled documented case.
Parts I–IV have their opening cases and their chapters' argument sections;
Part V's opening case is reserved in `contents.json` until it is written.

A chapter lands with the rules it renders: its paired pins file, its case in
`../tests/pins/suites.json`, its rows in the coverage ledger and its entry in
the annotated contents in `reference.md`, verified by `../verify.sh` and the development
tests. The receipt, audit and closure chain that once gated this was retired
on 2026-09-12.

Three files are deliberately unnumbered — `epigraph.md`, `method.md` and
`reference.md`. The epigraph is a poem; the optional method quotes the
machinery kept outside the ordinary reader chapters; the reference file holds
the map, contents, glossary and index. All three remain in the manifest's
reading sequence.
Two subdirectories are
not chapters and are outside the reading order and the length measurement:
`appendix/` carries the planning record (decisions, contracts, briefs, maps —
the former `new-book-plans/book-1-*` files with that prefix dropped), and
`source/` carries the formal source and everything beside it (the rest of the
former `new-book-plans/`, moved by flat rename on 2026-09-17).
Run `../verify.sh` to check every pinned claim against the constitution.

## Read or assemble the book

The same ordered inputs are published at
[dhilipsiva.dev/rights-nobody-has-to-earn/read/](https://dhilipsiva.dev/rights-nobody-has-to-earn/read/),
each as a web page with a Markdown counterpart, beside the
[live companion](https://dhilipsiva.dev/rights-nobody-has-to-earn/) that
executes selected records in the browser. The site is rebuilt from this
repository's `main`, so it follows the current manuscript and does not
identify an immutable edition.

Start with [the epigraph](epigraph.md), then [the opening note](00-opening-note.md).
The annotated contents in the [map, glossary and index](reference.md) describe
the route through the book;
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

Add `--sample` to assemble chapters 1, 4, 8, 21 and 29 as
`book-1-sample.html`, `book-1-sample.epub` and `book-1-sample.pdf` instead.
The [publisher proposal](../submission/README.md) explains the selection.
The sample keeps the original chapter numbers; references outside the
selection open their public repository locations.

Add `--summary` instead to build the plain-language summary edition,
`book-1-summary.html`, `.epub` and `.pdf`: the constitution the book describes,
as the numbered articles of `../ui/articles.json`, with the chapters that argue
each. It is the same text the companion publishes at `constitution/`.

Three parts of the back matter are generated and must be regenerated when
their sources change: `tools/book_diagrams.py` draws the diagrams in
`diagrams/`, each shown in the back matter beside its prose equivalent;
`tools/bibliography.py` writes the works cited in `bibliography.md` from the
notes; and `tools/book_index.py` writes the index at the end of `reference.md`
from the text. The book-builder tests fail when the works cited or the index
fall behind.

The 2026-09-25 reading copies hold 40 ordered inputs: 97,988 Markdown words,
HTML comments aside, of which the derived chapters' derived text holds 49,906
and their argument sections 21,960; the works cited add 5,220. The
five-chapter sample holds 13,997. The full PDF has 256 pages, the sample 36
and the summary edition 16; pagination depends on the browser and typography
used to rebuild them. The proposal explains the sample's selection.

Every copy carries the subtitle on its cover and the promise on its back
cover, and the EPUB and PDF carry both in their metadata. The print layout
hides the notes' return links and prints the path of any link to a repository
file, which paper cannot follow. HTML provides a linked contents list,
keyboard skip link and labelled table regions. EPUB carries the chapter order and nested contents. PDF includes
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
