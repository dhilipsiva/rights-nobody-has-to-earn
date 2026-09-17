# Licensing

This repository is deliberately **mixed-licence**, because it holds four different
kinds of artifact and one licence cannot serve them all. Each artifact type is
licensed for what it is.

| Artifact | Licence | SPDX |
|---|---|---|
| Book prose (the new derived book) | Creative Commons Attribution 4.0 International | `CC-BY-4.0` |
| Code — scripts, harness, data fetchers | MIT **or** Apache-2.0, at your option | `MIT OR Apache-2.0` |
| Data — the claim registry and fetched figures | CC0 1.0 Universal (public domain) | `CC0-1.0` |
| The constitution (`book-1/source/constitution.nibli`) | CC0 1.0 Universal, as a pre-decision file under the root `LICENSE` | `CC0-1.0` |

## Already dedicated, and not reversible

`book.md`, `manifesto.md`, and everything committed under the former
`new-book-plans/` — now `book-1/source/` and `book-1/appendix/` — were
published under the repository's root `LICENSE` — **CC0 1.0**, an *irrevocable*
dedication to the public domain. That stands. Those files are in the public domain
and no later decision can withdraw them. The root `LICENSE` therefore stays in
place and continues to govern them.

The table above governs **new** material only. Mixed licensing across a repository
is ordinary; what matters is that each artifact says which terms apply to it.

## Why each choice

**Prose → CC-BY-4.0.** Openness is the goal: anyone may translate, serialize,
excerpt, or republish. But attribution is *required*, which CC0 does not require
and which matters more here than it would for code. The book stakes everything on
being trusted, and provenance travelling with the text is part of how that is
earned — an altered edition circulating without a credit line is the failure mode
worth spending a licence clause on. CC-BY is also drafted for creative works and
maintained internationally, which suits a book written for a global audience.

Note the cost, accepted deliberately: CC-BY is perpetual and irrevocable, so
exclusivity can never be sold to a trade publisher. The reach strategy is
serialization and open circulation, not exclusivity.

**Code → MIT OR Apache-2.0.** The Rust-ecosystem convention and what nibli already
uses, so the toolchain is consistent end to end. Apache-2.0 contributes an explicit
warranty disclaimer, a liability limit, a trademark reservation, and a patent
grant; MIT contributes brevity for anyone who finds Apache heavy. Software licences
belong on software — their language ("substantial portions of the Software", NOTICE
files, patent grants) is drafted for code and reads badly when pointed at prose,
which is exactly why the book is not licensed this way.

**Data → CC0.** Nobody should need permission to check the numbers. Raw facts are
largely uncopyrightable in most jurisdictions anyway; CC0 removes the doubt and
makes independent verification frictionless. Since reader-side verification of the
*data* is what substitutes for showing the formalism, any licence friction here
would work directly against the book's central promise.

## Applying this

Add the full licence text alongside each artifact as it lands, rather than
committing boilerplate up front for files that do not exist yet:

- Book prose: `LICENSE-CC-BY` in the book's directory, plus an SPDX header or a
  licence line in the front matter.
- Code: `LICENSE-MIT` and `LICENSE-APACHE`, mirroring nibli's layout, with
  `SPDX-License-Identifier: MIT OR Apache-2.0` at the top of each source file.
- Data: `LICENSE-CC0` in the registry directory.

Canonical texts: <https://creativecommons.org/licenses/by/4.0/legalcode>,
<https://opensource.org/license/mit>, <https://www.apache.org/licenses/LICENSE-2.0>,
<https://creativecommons.org/publicdomain/zero/1.0/legalcode>.

## Two things no licence here does

**Trademark is unaffected.** CC0 explicitly does not waive trademark rights, and
neither CC-BY nor MIT grants them; Apache-2.0 reserves them expressly. The title
remains available as an integrity tool independent of any of the above — worth
knowing, since it is the only lever that survives an irrevocable content licence.

**Moral rights vary by country.** CC0 waives them only "to the extent possible
under law", and in many civil-law jurisdictions (Germany and France among them)
attribution and integrity rights are inalienable and cannot be waived at all. So
the position on the already-CC0 files is not uniform internationally. This
generally cuts in the author's favour, but "public domain everywhere" is not
strictly true of them.

*None of the above is legal advice. If a commercial deal is ever on the table, get
this reviewed by a lawyer before signing anything.*

## A file's licence travels with it

The SPDX header at the top of a file governs it wherever it lives. Moving a file —
as the planning record moved from `new-book-plans/` to `book-1/appendix/` and
the formal source to `book-1/source/` on 2026-09-17 — changes nothing about its
licence, and a file without a header keeps the dedication the section above
records.
