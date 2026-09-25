<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Reading Order, Names, and the Appendix

## Revision rulings D1, D4 and D7 — 2026-09-24

The author ruled on the revision tracker's reserved questions on 2026-09-24;
`CLAUDE.md` records all nine under *The revision rulings D1–D9*. Three bear on
this decision. Each is ratified but unimplemented until its TODO item lands.

**D1 — the subtitle, the reader and the promise (§2's names).** The title is
unchanged. Book 1's subtitle becomes *A worked design for a society, with its
formal claims made executable.*, superseding the subtitle in `CLAUDE.md`'s title
ruling. The primary reader is the serious non-specialist, with lawyers and
policymakers second. The first page and back cover carry: *"A constitution
designed from the person with nothing, argued in plain language, with every rule
published so you can test it."* Items 60 and 64.

**D4 — the constitution in plain language, in the companion (§3).** Numbered
plain-language articles are published in the companion, not printed in the book,
and chapters cite article numbers. The articles trace to the rule families,
contracts and pins that implement them. R4 is unchanged, and the formal source
stays outside the book's ordered inputs. The method stays the book's technical
appendix within its sealed scope. The articles include an interpretation
article. Its burden half (a restrictive conclusion needs complete positive
evidence, and absence never extends a power) traces to existing fail-closed
rules and their pins. Its protective-reading half (where a standard is open, the
reading more protective of the floor and of liberty prevails) is a declared
non-formal article, listed as a deliberate traceability gap. Items 61 and 62.

**D7 — the structure (§1 and §6).** The chapter table becomes the revision
plan's. Chapters 1 and 2, 9 and 10, and 23, 25 and 26 merge; Chapter 13 splits
into commons and animals; "An Ordinary Week" joins Part II and "Where This Could
Fail" joins Part V; and the map, glossary, roles and subject index move to the
back matter beside the method, so that at most five pages precede Chapter 1.
This supersedes §6's refusal to merge Voiding with Clawback: its mechanical
reason, the unscoped `:accept` closing chapter 25's pins, is met by ordering the
merged pins so that it comes last. The ordering rule of §1 stands. Item 53
carries the manifest, relocation and test changes the structure needs.

## The structure implemented — item 53, 2026-09-25

Ruling D7's table is the manifest. Chapters 1 and 2 are *The Child With Nobody*;
9 and 10 are *Work, Pay and Contribution*; 23, 25 and 26 are *Findings About
People*, which moves to Part IV beside the shield; Chapter 13 becomes *A Place in
Which Life Remains Possible* (commons and future conditions) and *Creatures
Without a Ballot* (animals). Chapter 3 is retitled *What the Record May Say* and
Chapter 16 *Answerability and Authority*, as the plan's table names them, and
Part V is *The argument*. Chapter 7, *An Ordinary Week*, and Chapter 30, *Where
This Could Fail*, are reserved as planned entries until items 55 and 59 write
them, so the reading sequence runs 1–6, 8–29 in the meantime. Every Part carries
a planned labelled opening case (ruling D3), named `part-N-<slug>.md` once it
lands, and Part V may hold more than one exempt chapter.

The opening note keeps the question, the thesis, the child and the prisoner,
the general limits and how to read the book, and Chapter 1 begins after about
1,200 words. The map, choices table, annotated contents, glossary, roles and
cases, subject index and diagrams moved unchanged, apart from their contents
entries, to `reference.md`, *Map, Glossary and Index*, after the method.

The merges kept every pin statement. `tools/relocate.py` gained `merges` and
`splits`: a destination is written by hand, the tool removes the retired sources
and rewrites references, sending a `::needle` reference to a split source to the
destination that holds it, and `check` holds each group to account — the
sources' pin statements at HEAD are exactly the destinations', the pin counts
agree, and every source paragraph no destination carries verbatim is listed for
review. Chapter labels are no longer rewritten inside footnote definitions,
where "chapter 6" cites another book. The findings pins run the pen strand,
then the limits strand, then the paired finding, whose closing unscoped
acceptance comes last as this decision's §6 required; the pen strand's
persisting clearance and custody facts precede the other two and change none of
their verdicts. The map is `tools/maps/2026-09-25-restructure.json`.

## The opening rewritten — item 60, 2026-09-25

Ruling D1's promise now opens the opening note, beneath its title, so it is the
first thing a reader meets after the epigraph; the subtitle and the back cover
wait for item 64. The note is written for the serious non-specialist. It opens
on Santoshi Kumari's case (ruling D3), with the dispute over her death intact,
then gives the question and thesis, the child and the prisoner, the three
distinctions stated once with the operation limit beside them, what the book
claims and does not, its contributions in outline with Part V named for the
rest, the channels in which the author speaks, a statement of AI assistance
naming what was drafted, written and checked with it and how, the shared
maintainer of the constitution, its tests and the engine, and how to read the
book. In the review PDF the epigraph and the note take four pages, and Chapter 1
begins after Part I's opening case, within the five pages ruling D7 allows
before it. The exact prose is `session-drafted, author-approved under delegated
approval (2026-09-13)`.

## Choices before the institutional detail — item 25, 2026-09-21

The exact current [opening note](../../00-opening-note.md), its argument map
and glossary changes are
`session-drafted, author-approved under delegated approval (2026-09-13)`.
This is an editorial application of the settled reading order. It changes no
constitutional commitment, source rule, pin, chapter number or manifest role.

The opening now introduces public responsibility with plural provision,
residence-based political membership, divided government, collective execution,
independent appointments, purpose-limited records, the provisional shield,
emergency limits and unamendable human and animal protections. It distinguishes
those commitments from chosen instruments and names decision delay and harder
attribution as costs requiring argument. An early direct link reaches Chapter 1
without requiring the map, glossary or formal method.

The optional map pairs the chapters explaining mechanisms with their actual
Part V arguments. A heading over the existing scarcity argument supplies a
direct destination; the argument itself is unchanged. The protected-core and
institutional-mechanism definitions explain why a constitutional rule can bind
current government while remaining lawfully revisable. Normative justification
stays in the exempt opening and Part V. The competence-certificate glossary
label distinguishes that specific record from certificates of political results
and amendment candidates elsewhere in the book. The existing qualifications
entry in `full-society-ledger.json` points to that label; no new coverage row
or changed formal expectation is introduced.

The fixed child-argument sequence remains: the structural absence of recorded
private support and public action; the methodological claim explicitly marked
as argument beside its bounded standing result; the prisoner pairing; the
warning against attributed feelings; and the book's title. The opening claims
no delivered service, universal proof, outside testimony or successful operation.
The chapter order, optional method, unnumbered epigraph, child slots and three
exempt prose channels remain. The historical ruling below retains its original
ratification and implementation language; current scope and verification follow
`CLAUDE.md` and `book-1/contents.json`.

> **Status: author-ratified 2026-09-16, ratified but unimplemented.** Four
> rulings and the design that executes them: the reading order is editorial
> and runs engines before breaks; both books are named; the planning record
> becomes Book 1's appendix under a narrow override; and `book-1/source/`
> ceases to exist (done 2026-09-17). Nothing here creates a chapter, a rule or a claim by
> itself; the tracker items in §7 implement it. Where this record and
> `CLAUDE.md` diverge, this record controls. The companion record is
> [`child-with-nobody-decision.md`](child-with-nobody-decision.md).

## 1. The reading order is editorial: engines before breaks

**What was measured.** Chapter order was never computed. `3-spine.md` says so
itself — the stratification block is generated; the chapter list is a hand
list "in an editorial reading order" — and the runtime order is the filename
prefix in five separate implementations. Four prose sites nevertheless
asserted a computed order (the opening note's first page, "Why the prisoner
appears so early", "Why the chapters have this order", and the method part's
"The order of the chapters, and the tools that got it wrong"), and so did
`CLAUDE.md` ("strictly computed, never chosen") and the 2026-08-17 decision
("substance primacy is not allocatable — chapter order is computed"). All are
superseded by this ruling; the prose sites are rewritten by the tracker item
that renumbers.

**The rule**, recorded verbatim in `book-1/contents.json`:

> The chapters run in the order a person meets the design, not the order its
> rules depend on one another: first who counts and what may be written about
> them, then what every person is owed and how it is meant to reach them, then
> the ordinary life the design leaves alone, then the public power that serves
> that life, and only at the end what the design does to a person and what it
> does when it catches itself failing.

Short name: *engines before breaks*. It is testable by a reader: every title
before Part IV names something a person has, does or is owed; every Part IV
title names something done to a person or noticed about a failure. The
stratification in `3-spine.md` remains the derivation record and stops
implying an order. Chapter numbers in rulings dated before 2026-09-16 are
pre-reorder and are not rewritten; the applied maps under `tools/maps/` are
the key between old and new numbers.

**The final table.** Four derived Parts, so "Part V" keeps its name. Files
renumber once to these numbers, reserving the slots of chapters not yet
written; each later chapter fills its slot with no further renumber. Word
budgets are ceilings under "content governs".

| # | Title | Families | Status |
|---|---|---|---|
| **Part I — Who counts, and what they are owed** | | | |
| 01 | The Child With Nobody | UNIVERSAL-STANDING (birth root), ARTICLES 1/1b, FAMILY-LIFE-COURSE/ORDINARY as case, DELIVERY-RECEIPT (the honest half) | NEW — the heart |
| 02 | Who Counts | UNIVERSAL-STANDING (all four roots, T1 continuity, `person` writable) | PROMOTED from 07 "The standing roots do not replace this line" |
| 03 | What Counts as Evidence | ARTICLES 0/0a, amendment vocabulary | KEPT (was 01) minus "Counting without ranking people", "Keeping, watching…" |
| 04 | What You Are Owed | ARTICLES 1/1b, OBLIGATIONS core | KEPT (was 08): preamble, firewall, floor-debt core; family/care ¶ moves out |
| 05 | Whether It Arrived | DELIVERY-RECEIPT | PROMOTED from 08 "Owed is not the same as delivered" + 11 "What that does not fix" |
| 06 | When There Is Genuinely Not Enough | SCARCITY-AND-CONFLICT | PROMOTED from 08 |
| 07 | Who Owes, and What Follows | OBLIGATIONS | PROMOTED from 08 "Owed by whom" (duty kinds, excuse, civic, voluntary) + 14's reader/action/non-response/alternate/continuity/remedy chain |
| **Part II — The life the design leaves alone** | | | |
| 08 | What Nobody Has to Ask Permission For | KNOWLEDGE-AND-FREE-FIELD, LIBERTY-ECOLOGY (liberties) | PROMOTED from 08 + new § "The things power may not prescribe" |
| 09 | Earning Above the Floor | ECONOMIC-CONSTITUTION (labour, plural floor, insolvency), INCOME-SECURITY, QUALIFICATIONS-COMPENSATION | NEW (absorbs 08:758-779, 10:95-114) |
| 10 | Contribution | ARTICLES 3/4, `reward` | KEPT (was 10) minus supplement/certificate ¶s |
| 11 | What Money Cannot Buy | ECONOMIC-CONSTITUTION (property, contract, enterprise, fiscal, monetary, federal, remedy, temporal), PUBLIC-SCALE-VOCABULARY | NEW (absorbs 08:781-800) |
| 12 | The Same Route for Everyone | SUBSTANTIVE-EQUALITY (+ statistics pattern route) | NEW (absorbs 08:637-661) — needs the equality ordinary half first |
| 13 | A Place in Which Life Remains Possible | ECOLOGICAL-ANIMAL, LIBERTY-ECOLOGY (environment/Class 9) | PROMOTED from 08 ×4 + 09 "Speaking for conditions…" + new § "Axes that do not trade" |
| 14 | Holding a Role in Somebody's Life | FAMILY-LIFE-COURSE, FAMILY-LIFE-ORDINARY | PROMOTED from 09 ×2 + 08 "Care is not a family invoice" |
| 15 | Arriving and Belonging | MOBILITY-PLURALITY | PROMOTED from 08 ×2 + 09 ×2 |
| **Part III — The public power that serves it** | | | |
| 16 | Public Answerability, and Why It Is Never Revoked | ARTICLES (`authority`), STATE-FORM, T1 | KEPT (was 02) |
| 17 | How Public Power Is Built | STATE-FORM, DEMOCRATIC-INTEGRITY | PROMOTED from 09 ×2 + new §§ from state-form cases (tiers, bodies, appointments, deadlock, secession, result contract) |
| 18 | The Vote Conviction Does Not Take | ARTICLES (`decide`), STATE-FORM result contract | KEPT (was 09) core five sections |
| 19 | What May Be Kept About You | RECORD-POWER, OFFICIAL-STATISTICS | PROMOTED from 01 ×2 |
| 20 | A Crisis Does Not Suspend the Republic | PUBLIC-SAFETY (emergency, defence, external, border) | PROMOTED from 09 ×4 |
| 21 | A Way to Be Heard | NON-CARCERAL-JUSTICE | PROMOTED from 08 |
| 22 | Changing the Rules | AMENDMENT-ENACTMENT, ARTICLES 9 | KEPT (was 12) |
| 23 | Who Holds the Pen | ARTICLES 8 (`permits`), T1 | KEPT (was 03) — the bridge; its either-alone pair is what the reader carries into Part IV |
| **Part IV — What the design does to a person, and how it catches itself** | | | |
| 24 | The Shield | ARTICLES 7 (`defend`) | KEPT (was 04), length kept |
| 25 | Voiding | ARTICLES 4 (`false`) | KEPT (was 05), length kept |
| 26 | Clawback | ARTICLES 4 (`lose`) | KEPT (was 06), length kept |
| 27 | A Prisoner Is a Person | ARTICLES 6/1 | KEPT (was 07), −standing section → 02 |
| 28 | Where People Are Put | PLACEMENT, ARTICLES 5 | KEPT (was 11), −"What that does not fix" → 05, +14's duplicate alarm section |
| 29 | The One Thing Taken | `travel`, T3, PUBLIC-SAFETY (force, holding), justice release | KEPT (was 13), length kept |
| 30 | When the System Notices It Broke | ARTICLES 8b, OBLIGATIONS bridge, KNOWLEDGE withdrawal | KEPT (was 14), −duty chain → 07 |
| **Part V — Outside the graph** | | | |
| 31 | The Five Joints | exempt | KEPT (was 15) |
| — | `method.md` | exempt, unnumbered | unchanged filename (sealed) |

Projected: engines about fifty thousand words; breaks about seventeen
thousand, a quarter of the derived total; derived roughly sixty-seven to
seventy thousand against about twenty-three thousand exempt. Every one of the
twenty-two block families is projected by a headed section in exactly one
home chapter (LIBERTY-ECOLOGY splits by its own two halves, 08 + 13;
PUBLIC-SAFETY and NON-CARCERAL-JUSTICE keep their coercive-instrument
sections in 29).

## 2. Names

Book 1 keeps ***The Rights Nobody Has to Earn*** and now speaks it. Book 2 is
***What It Would Take*** — the author's choice on 2026-09-16. The conditional
mood is the honesty the assurance portfolio requires (nothing is claimed
built or live), a stranger knows the idiom, and it pairs by contrast: the
rights cost the person nothing, the society costs something to build. The
proposed subtitle for its README is *the road to the society in* The Rights
Nobody Has to Earn.

**Where the names are spoken.** In every exempt element, the front matter,
both READMEs, both trackers and the appendix: `README.md`, `book-1/README.md`;
the opening note's first page (Book 2 named where the note says what this
book is not, Book 1's title spoken in the argument block); Part V's two
existing seam sentences; the method part's close ("They are the next book,
*What It Would Take*. This one, *The Rights Nobody Has to Earn*, was the
destination; that one is the road."); `book-2/TODO.md`. **Derived chapters
stay name-free** by the author's choice: a derived sentence traces to a rule
or a pin, and a title is neither. The former constraint that "book-1
references book-2 exactly once, in the method's closing lines" is retired —
it was already false, since Part V names Book 2 twice.

**Rejected titles, so they are not re-proposed:** *Getting There From Here*
(plain and seam-audible, but implies the author already knows the way);
*What It Costs* (Part V's own phrase and Book 2 owns costs, but it invites the
"this is a warning" misreading the first readers already made); *Building the
Floor* (pairs with the book's key noun, but "floor" is jargon to a stranger —
the same reason Book 1's title avoided it); anything saying "how it works" or
"how they arrive".

## 3. The appendix — a carried archive, not a fourth channel

The planning record moves under `book-1/appendix/`. The 2026-08-08
narrative-register ruling refuses "a fourth exempt element" by name, and this
record overrides that refusal **narrowly**: the appendix is a non-derived
element of the *directory*, not of the *book*. It carries no passages, joins
no register, is not an ordered input to any artifact (`combine.sh`, the pilot
builder), sits outside the edition boundary (E2/P1/D2) until a Gate B/C
decision binds it, is not classified in the coverage ledger, is **outside the
length-invariant measurement** — which measures the ordered inputs — and may
never be the sole support for a claim a chapter makes. The book's exempt
elements remain three. The method part's sealed scope is untouched; its "the
repository is the appendix" pointer now names the directory.

The length exposure that forced this: the decisions, contracts and briefs
alone are about a hundred and forty-four thousand words against about fifty
thousand derived. Counted on the exempt side they would break "majority
derived" three times over; carried as an archive they count for nothing.

## 4. `new-book-plans/` ceases to exist

The author's choice on 2026-09-16: everything moves under `book-1/`.

**Appendix.** `book-1/appendix/{README.md, decisions/, contracts/, briefs/,
maps/}`; the `book-1-` prefix is dropped on the move, the `-decision`,
`-contract` and `-brief` suffixes stay. Forty-five files: eighteen decisions
(including `full-society-boundary-decision.md`), twenty-one contracts, three
briefs, three maps (`constitutional-coverage-map`, `constitutional-taxonomy`,
`red-team-index`). Files keep their CC-BY-4.0 headers; `LICENSING.md` gains
one clause that the per-file header governs wherever the file lives.

**Source.** Everything else — the formal source `constitution.nibli`, every
`*-source.json`, the forty family `*.pins.nibli`, `counterfactual/`, the
generated reports and frozen audit pairs, `reader-evidence*` and the pilot
kit, the fifteen `*-reader-draft.md`, `4-strata.py` (moved, content
untouched — an exhibit, not a script to repair), `15-pilot-reader-artifacts.py`,
and the engine-measurement history — moves by **flat rename** to
`book-1/source/`: same basenames, same subdirectories, one prefix rewrite.
Sub-structuring `source/` is deliberately not done here. The method part's
sealed rule (d) is untouched: the KR lives beside the book's ordered inputs,
never inside them, and `book-1/source/` and `book-1/appendix/` are not
chapters — the digit-prefix filter and the manifest keep them out.

**What the relocation tool rewrites** (measured at `c4cb3582`):
`full-society-ledger.json`, 44,744 lines carrying the 43,259 needle paths
(prefix only; needle text unchanged); `full-society-power-source-manifest.json`
483; `assertion-surface-contracts.json` 179 and its audit 135; the other frozen
audit pairs; `tests/pins/suites.json` 140; the generators' path constants (94
lines across `src/authoring/*.rs`, 29 in `src/*.rs`); `CLAUDE.md` 61;
`README.md` 22; `book-2/TODO.md` 20; pins-file header comments; the three
reader-side sources' `basis` and `chapter` paths. `verify.sh`, `generate.sh`
and `combine.sh` carry no path. `reviews/` quotations stay as written. No Rust
code reads any appendix file, and the move is invisible to the order sites
(non-recursive, digit-prefixed). `full-society-ledger.json`'s
`bound_sources_sha256` for two decision files goes stale when the fold
rewrites their cross-references — dormant, nothing reads it.

## 5. The manifest, the relocation tool and the permanent checks

**`book-1/contents.json`** (`deny_unknown_fields`): `rule`, `front`, `parts`
of `{title, chapters: [{number, file | null, title, role: derived|exempt,
group: engine|break|null, status: landed|planned, families}]}`, `back`. A
`planned` entry has no file and reserves its number. Filenames keep the
two-digit prefix, and a test requires prefix == manifest position ==
reader-facing number, so the lexicographic runtime sort stays valid where it
is convenient and the Rust `read_dir` sites read the manifest instead —
killing the fixed count of fourteen and the two literal filenames in
`claim_discipline_tests.rs`.

**`src/authoring/contents.rs`** — `load`, `derived()`, `numbered()`,
`part_v()`, `planned()`. Consumers: `claim_discipline_tests.rs`,
`reader_coverage.rs::headings`, `resolution_receipts.rs`, `spine.rs` (a
second generated region rendering number, file, title, role, group and part,
planned entries marked), `combine.sh` (cap removed), the pilot builder's last
numbered input.

**`tools/relocate.py`** (Python 3 stdlib, MIT OR Apache-2.0): `plan` derives
a rename map from the manifest against the disk by slug; `apply [--dry-run]`
does the `git mv`s, a longest-first substring rewrite of every `from` path
over the text scope with a boundary-guarded basename pass, a **simultaneous**
regex rewrite of `Chapter N`/`chapter N`/`chapter-N` labels restricted to the
numbered chapters, `method.md` and pins-file comments (never `CLAUDE.md`,
`3-spine.md`'s superseded list, frozen audits, drafts or `reviews/`), then
repoints `reader-coverage-source.json` (`chapter`/`basis`, id prefixes, a
stable re-sort by new position), the receipts and audit sources, and the pins
headers' `--only` lines; `check` asserts that no `from` string survives in
scope, that the content-never-touched set is byte-identical to `HEAD`, that
every non-comment pins line is byte-identical, and that `git status` shows
exactly the planned renames. Applied maps are kept as
`tools/maps/<date>-<name>.json`.

**Content never touched** (asserted byte-identical after a move): the
constitution; the eight rule-family `*-source.json`; every non-comment line
of every `*.pins.nibli`; `4-strata.py`; `registry/`; `book.md`,
`manifesto.md`, `1.md`, `2.md`, `tmp.txt`, `reviews/`, `new-reviwes/`.

**`src/authoring/reference_integrity_tests.rs`**, permanent under
`cargo test --release --bin generate`: every reviewed reference of the form
`path::needle` names an existing file and its needle occurs exactly once,
with the baseline measured before anything moves and any pre-existing
failure allowlisted with its count so the migration invariant is "the
resolving set is unchanged modulo rewriting"; the manifest matches the
directory (landed entries have files with prefixes at their positions, derived
ones have sibling pins, exempt ones do not, planned ones have no file, each
derived chapter has exactly one live `scan: true` suites case, each pins
header's `--only` names its own path); derived chapters run engines before
breaks; the opening note's navigation matches the manifest and every relative
link and fragment in the ordered inputs and the appendix resolves; the spine's
generated contents block is current.

## 6. The engines, and the breaks read last

**Conventions for every chapter item.** A paired pins file with a CONTENT pin,
`:expect-pins`, a header listing its supporting `tests/pins/...` paths, fresh
constants in every argument position and `:accept-scoped` controls; a suites
case; one ledger row per `## ` with `family` single-valued (split a mixed
section rather than blur whose Book 2 boundary is stated); no digits; the
rule, never the count; a closing boundary section whose prose matches the
ledger's boundary regex; the child slot; the opening note's contents,
domain and glossary entries; regenerated `reader-coverage.md`. A promotion
moves author-approved text verbatim and removes it from the host in the same
commit, moving any resolution receipt's `chapter` with the narrative it
binds.

**Rendering a dormant family honestly** — INCOME-SECURITY,
QUALIFICATIONS-COMPENSATION, FAMILY-LIFE-ORDINARY, the delivery routes,
first-contact standing — copies the shape of chapter 8's "Owed is not the
same as delivered" and chapter 10's supplement paragraphs: ask the cast
question and answer with the record's own no; say what the no is; state the
rule's shape in the conditional and name what would complete it; show the
routes are not blind with a fresh-constant probe in the pin file; close on
the Book 2 boundary and the liveness bar. No sentence may say a supplement, a
wage, a certificate or a measure arrives.

**Interface first.** SUBSTANTIVE-EQUALITY is the shape FAMILY-LIFE-COURSE was
before its ordinary half landed — fifty-four person-held barriers and no
interface underneath — so its ordinary half (`equality-source.json`,
`./generate.sh equality`, block `SUBSTANTIVE-EQUALITY-ORDINARY`, a new block
name, since reusing an existing block name silently deletes it) lands before
the equality chapter narrates a measure ending or an accommodation owed.

**The breaks.** No merges: the only candidate, Voiding with Clawback, is
refused for a mechanical reason — chapter 5's closing exhibit is an unscoped
`:accept` premise, so a concatenated pin file would run the clawback pins
against a widened base. Trims are executed by the promotions that receive the
content. Must not be cut, because a ruling or a receipt names it: Ruk's
placement and severity; Lupo's void; Cira's clawback and `person(Fin)`; the
Zed composition; the either-alone pair Adam/Voss/Nia; Don, Sly, Kel, Rex and
Zeno; Vex penless and Ambi/Solo; the discriminator pair `obliged(Review, Ruk)`
against `obliged(Ruk, Review)`; every phrase bound by the twelve receipts.
Tests: coercive passages fewer than a quarter of engine passages and every
Part IV prefix after every engine prefix; every constitutional family read
from the block markers projected by a passage, with the unrendered set
asserted by membership and kept as an empty expectation once it empties;
every family stating a boundary where it is rendered.

## 7. Implementation

The tracker `TODO.md` carries the items in order: the manifest, tool and
checks; the move; the child's record; the chapter; the opening argument; the
renumber; one item per new or promoted chapter; the child slots; the close of
the rebalance. Each item is verified by `cargo test --release --bin generate`
and the complete `./verify.sh`, run sequentially, and after a move or a
renumber by the tool's `check`, the spine generator, the report generators
byte-stable on a second run, `combine.sh`, and a pilot build.

## 8. Ratification record

The author's instruction of 2026-09-16 asked for the merge, the recomputation
with more chapters and new titles and priority, the correction of the breaks-
before-engines imbalance, names used throughout, and the child at the heart.
Four questions were put and answered the same day: Book 2's title; the fate of
`book-1/source/` (moved under `book-1/source/`); where the names are spoken
(never in a derived chapter); and the appendix's relation to the book (a
carried archive outside the reading order and the length rule). The planning
session's measurements are recorded in §1, §3 and §4 with the revision they
were taken at.
