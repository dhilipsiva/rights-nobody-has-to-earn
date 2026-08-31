<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The Rights Nobody Has to Earn

This directory is the reader-facing Book 1 projection of the principal formally
audited constitutional specification. It is not the specification itself and
cannot override, complete, or upgrade it. It contains the epigraph, opening note,
derived chapters in computed order, Part V, and method part. The numbered
chapters between the opening note and Part V are the derived spine—their order
is computed from the dependency stratification of the constitution in
`../new-book-plans/constitution.nibli`, and their claims are pinned by the
`*.pins.nibli` files beside them. Exactly three elements are exempt from the
derivation gate and labelled so in their own text: the opening note, Part V, and
the method part.

Book prose may be drafted as one slice of a coherent assurance batch, but it is
not landed or audited on its own. The same frozen candidate must include the
formal source, governed contracts, tests, counterfactuals, projections, and every
other affected passage; it then receives the repository's full receipt, audit,
and closure chain.

Two files are deliberately unnumbered — `epigraph.md` and `method.md` —
because the prose sweeps in `../verify.sh` glob the numbered files only: the
epigraph is a poem, and the method part must quote the machinery the sweeps
forbid everywhere else. Do not renumber either; the naming is load-bearing.
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
