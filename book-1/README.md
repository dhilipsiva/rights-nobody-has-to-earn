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

Two files are deliberately unnumbered — `epigraph.md` and `method.md` —
because the prose sweeps in `../verify.sh` glob the numbered files only: the
epigraph is a poem, and the method part must quote the machinery the sweeps
forbid everywhere else. Do not renumber either; the naming is load-bearing. Two subdirectories are
not chapters and are outside the reading order and the length measurement:
`appendix/` carries the planning record (decisions, contracts, briefs, maps —
the former `new-book-plans/book-1-*` files with that prefix dropped), and
`source/` carries the formal source and everything beside it (the rest of the
former `new-book-plans/`, moved by flat rename on 2026-09-17).
Run `../verify.sh` to check every pinned claim against the constitution.

## Licence

All prose in this directory is licensed under the Creative Commons
Attribution 4.0 International licence (CC-BY-4.0). The full text is in
[LICENSE-CC-BY](LICENSE-CC-BY). The licence declaration lives here rather
than in the chapters because the chapters are reader-facing prose; this
file is the front matter the repository carries until the book has its own.

The pin files beside the chapters are part of the repository's verification
harness; see [../LICENSING.md](../LICENSING.md) for the repository-wide
licence map.
