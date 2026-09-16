# TODO — Book 1: the child at the heart, engines before breaks

Created 2026-09-16 on the author's instruction, after the root tracker had been
retired earlier the same day: *"Just plan and let us create a list of ordered
TODO items in a TODO.md file and work on it one at a time."* This file is that
list. It is a work list, not a second statement of any gate or discipline: the
controlling text of every rule it names is `CLAUDE.md`, and the two decision
records that item 1 writes are the controlling records for the design below.

## How to work this file

- **One item at a time, in order.** Pick the first open item, implement it,
  run `cargo test --release --bin generate`, then `./verify.sh` (complete; run
  the two **sequentially, never beside each other** — a concurrent run starved
  the verifier and OOM-killed a test binary on 2026-09-16), commit, push,
  delete the item. Update an item if it is partly done.
- **Item 1 moves the design below into the two controlling records and cuts it
  from this file**, leaving pointers. Until then this file is the only in-repo
  copy of the approved plan; that duplication is transient by construction.
- **Rules that bind every item** (one line each; the rulings in `CLAUDE.md`
  control): verification is Nibli pins and contradiction scans only — add no
  hashing, receipt or freshness gate; no digit and no counted claim in a
  derived chapter — state the rule that produces the count; never route a
  judgment through the compute backend; every argument position in a new fact
  is a new constant unless it is an institution; a derived chapter derives from
  the constitution and carries a paired `.pins.nibli` registered in
  `tests/pins/suites.json`; the register is flat — no interior state, never a
  cast name in a passage, never warm the cast; never invent testimony,
  external evidence, reviews or operational success; no arrival verb, no
  guarantee-of-growth or operation claim in prose (register-audit every prose
  batch before committing); moved author-approved text moves verbatim, and
  substantive rewording needs the exact-version record; new prose lands as
  `session-drafted, author-approved under delegated approval (2026-09-16)`.
- **Never touched in content** by the relocation tool (asserted byte-identical
  after a move): the constitution, the eight rule-family `*-source.json`,
  every non-comment line of every `*.pins.nibli`, `4-strata.py`, `registry/`,
  `book.md`, `manifesto.md`, `1.md`, `2.md`, `tmp.txt`, `reviews/`,
  `new-reviwes/`.
- Commits end with the standing attribution lines and explain why in the body.

## Rulings to record (item 1 carries these until they land in `CLAUDE.md`)

- **R1 — The child is the heart.** Supersedes the 2026-08-17 through-line
  refusal in exactly this scope: the caregiverless child is the opening
  argument (exempt), the first derived chapter (*The Child With Nobody*, a
  record of one line, `born(Nell).`, introduced by fixture and never added to
  the cast), and the closing test section `## The child with nobody` of every
  engine chapter that can run its rule against a person with one entry
  (asserted by membership; exemptions carry reasons). "Most vulnerable" is a
  **test on absent entries**, never a status (D3). **Unchanged:** the flat
  register (no interior state — the affect warning is now printed rather than
  implied); no vulnerability score, ranking, status or targeting in any record
  (`vulnerable` stays unadmitted and is pinned as refused); the methodological
  claim stays Reasoned, in the exempt elements, beside its one executable
  instance (universal standing); the `dignity`/`safety` bans; the title ruling;
  the passage rule and OL-15-v1; the receipt gap for a recipient who cannot
  choose a witness stays open and is named, not closed. The 2026-08-17
  decision's §3–§5 are superseded; its §2 thesis-form constraints stand.
  Controlling record: `book-1/appendix/decisions/child-with-nobody-decision.md`.
- **R2 — Names (author's choice, 2026-09-16).** Book 1 keeps *The Rights
  Nobody Has to Earn* and now speaks it. Book 2 is ***What It Would Take***.
  Both names are used in every exempt element, the front matter, the READMEs,
  both trackers and the appendix. Derived chapters stay name-free (author's
  choice): a derived sentence traces to a rule or a pin, and a title is
  neither. The "book-1 references book-2 exactly once" constraint is retired
  (it was already false: Part V names Book 2 twice and the method once).
- **R3 — The reading order is editorial: engines before breaks.** Rule (D1):
  *the chapters run in the order a person meets the design, not the order its
  rules depend on one another.* `book-1/contents.json` records it and tests
  enforce it; the stratification in `3-spine.md` remains the derivation record
  and stops implying an order. Supersedes "chapter order is strictly computed,
  never chosen" and the 2026-08-17 "substance primacy is not allocatable";
  chapter numbers in rulings dated before 2026-09-16 are pre-reorder and are
  not rewritten (the maps under `tools/maps/` are the key). Controlling record:
  `book-1/appendix/decisions/reading-order-and-appendix-decision.md`.
- **R4 — The appendix is a carried archive, not a fourth channel.** The
  planning record moves under `book-1/appendix/`. The 2026-08-08 refusal of "a
  fourth exempt element" is overridden **narrowly**: the appendix is a
  non-derived element of the *directory*, not of the *book* — it carries no
  passages, joins no register, is not an ordered input to any artifact
  (`combine.sh`, the pilot builder), sits outside the edition boundary
  (E2/P1/D2) until a Gate B/C decision binds it, is not classified in the
  coverage ledger, is **outside the length-invariant measurement** (which
  measures the ordered inputs), and may never be the sole support for a claim a
  chapter makes. The book's exempt elements remain three; the method part's
  sealed scope is untouched and its "the repository is the appendix" pointer
  now names the directory.
- **R5 — `new-book-plans/` ceases to exist (author's choice, 2026-09-16).**
  The 45 human-facing planning documents move to `book-1/appendix/` (D6);
  everything else moves by flat rename to `book-1/source/` — the formal source
  (`constitution.nibli`), every `*-source.json`, the 40 family `*.pins.nibli`,
  `counterfactual/`, the generated reports and frozen audit pairs,
  `reader-evidence*` and the pilot kit, the 15 `*-reader-draft.md`,
  `4-strata.py` (moved, content untouched — an exhibit, not a script to
  repair), `15-pilot-reader-artifacts.py`, and the engine-measurement history.
  The method part's sealed rule (d) is untouched: the KR lives beside the
  book's ordered inputs, never inside them, and `book-1/source/` and
  `book-1/appendix/` are not chapters (the digit-prefix filter and the manifest
  keep them out). Every `new-book-plans/` path in code, scripts, suites,
  CLAUDE.md, READMEs, trackers, pins headers and reviewed JSON is rewritten by
  the relocation tool in item 3; `reviews/` quotations are not.
- **R6 — Re-measurement the 2026-08-17 ruling required.** Custody remains the
  deepest derivation chain; family/life-course is now the widest family (the
  person-held barrier block plus ten ordinary contracts) and its heads are read
  by no rule; in the shipped cast the floor actualities still derive only
  through confinement. The "hardest stress test" superlative is retired in
  favour of the pair (child and prisoner fail in opposite directions), which
  needs no winner. Method recorded, never a number.
- **R7 — Two counts corrected.** "Fourteen derived chapters" (CLAUDE.md, the
  digit-gate test) becomes manifest-derived; "references book-2 exactly once"
  is replaced by R2.

## Design carried here until item 1 records it

### D1 — The reading order and the final chapter table

**Rule (recorded verbatim in the manifest's `rule` field and in R3):**

> The chapters run in the order a person meets the design, not the order its
> rules depend on one another: first who counts and what may be written about
> them, then what every person is owed and how it is meant to reach them, then
> the ordinary life the design leaves alone, then the public power that serves
> that life, and only at the end what the design does to a person and what it
> does when it catches itself failing.

Short name: *engines before breaks*. Testable: every title before Part IV names
something a person has, does or is owed; every Part IV title names something
done to a person or noticed about a failure.

Four derived Parts so **"Part V" keeps its name** (a proper noun in
`claim_discipline_tests::PART_V`, the ledger's `argument` pattern, and scores
of rulings). Files renumber **once** to these numbers (item 7), reserving the
slots of chapters not yet written; each later chapter fills its slot with no
further renumber. Word budgets are ceilings under "content governs".

| # | Title | Families | Status |
|---|---|---|---|
| **Part I — Who counts, and what they are owed** | | | |
| 01 | The Child With Nobody | UNIVERSAL-STANDING (birth root), ARTICLES 1/1b, FAMILY-LIFE-COURSE/ORDINARY as case, DELIVERY-RECEIPT (the honest half) | NEW — heart (D3) |
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
| 12 | The Same Route for Everyone | SUBSTANTIVE-EQUALITY (+ statistics pattern route) | NEW (absorbs 08:637-661) — needs item 10 first |
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

Projected: engines ≈ 50,000 words; breaks ≈ 17,300 (≈ 25 % of derived);
derived ≈ 67–70k against ≈ 23k exempt. Every one of the 22 block families is
projected by a headed section in exactly one home chapter (LIBERTY-ECOLOGY
splits by its own two halves, 08 + 13; PUBLIC-SAFETY and NON-CARCERAL-JUSTICE
keep their coercive-instrument sections in 29).

### D2 — The manifest, the relocation tool, and the permanent checks

**`book-1/contents.json`** (new, `deny_unknown_fields`): `rule`, `front:
["epigraph.md","00-opening-note.md"]`, `parts: [{title, chapters: [{number,
file | null, title, role: derived|exempt, group: engine|break|null, status:
landed|planned, families: [...]}]}]`, `back: ["method.md"]`. A `planned` entry
has no file and reserves its number. Filenames keep the two-digit prefix and a
test requires **prefix == manifest position == reader-facing number**, so the
lexicographic runtime sort stays valid where convenient and the three Rust
`read_dir` sites read the manifest instead (killing the fixed count of 14 and
the two literal filenames in `claim_discipline_tests.rs`).

**`src/authoring/contents.rs`** — `load`, `derived()`, `numbered()`,
`part_v()`, `planned()`. Consumers: `claim_discipline_tests.rs` (`PART_V` and
the `read_dir`/`14` block), `reader_coverage.rs::headings`,
`resolution_receipts.rs` chapter list, `spine.rs` (second generated region
`<!-- BEGIN GENERATED: contents -->` rendering number/file/title/role/group/part,
planned entries marked), `combine.sh` (cap removed: `grep -E '/[0-9]{2}-' |
grep -v '/00-'`), `15-pilot-reader-artifacts.py:871` (last numbered input
instead of the literal).

**`tools/relocate.py`** (Python 3 stdlib, MIT OR Apache-2.0): `plan` derives a
rename map from the manifest vs disk by slug; `apply [--dry-run]` does the
`git mv`s, then a longest-first substring rewrite of every `from` path over the
text scope (with a basename pass guarded by a boundary character), then a
**simultaneous** regex rewrite of `Chapter N`/`chapter N`/`chapter-N` labels
restricted to `book-1/[0-9][0-9]-*.md`, `method.md` and pins-file comments
(never CLAUDE.md, `3-spine.md`'s superseded list, frozen audits, drafts,
`reviews/`), then repoints `reader-coverage-source.json` (`chapter`/`basis`,
id prefixes, stable re-sort by new position), the receipts and audit sources,
and the pins headers' `--only` lines; `check` asserts no `from` string survives
in scope, the never-touch set is byte-identical to `HEAD`, every non-comment
pins line is byte-identical, and `git status` shows exactly the planned
renames. Applied maps are kept as `tools/maps/<date>-<name>.json` — the
permanent key from pre-reorder chapter numbers in dated rulings.

Text scope: `CLAUDE.md`, `README.md`, `AGENTS.md`, `LICENSING.md`,
`book-1/**/*.md`, every `*.pins.nibli` (comment lines only), `book-2/*.md`,
`tests/pins/suites.json`, the planning/source `*.md`, `*.json` minus the eight
rule-family sources, `*.py` minus `4-strata.py`, `counterfactual/README.md`,
`reader-evidence-pilot/*.md`, `src/**/*.rs`, `verify.sh`, `generate.sh`,
`combine.sh`. The reference regex is `^(book-1|tests|registry)/[^:\s]+(::.+)?$`
once item 3 has landed (`new-book-plans` is accepted only in the item-2
baseline).

**`src/authoring/reference_integrity_tests.rs`** (permanent, under `cargo test
--release --bin generate`): `every_reviewed_reference_resolves_exactly_once`
(walk every string in every reviewed `*.json`; a value matching the reference
regex must name an existing file and its needle must occur exactly once;
dedupe `(path, needle)` pairs; **baseline first** — any reference failing on
the unchanged tree goes into a dated allowlist constant with its count, so the
migration invariant is "the resolving set is unchanged modulo rewriting");
`contents_manifest_matches_the_directory` (digit-prefixed `.md` set == landed
entries; prefixes == positions; every `derived` landed file has a sibling
`.pins.nibli`, exempt ones do not; planned entries have **no** file;
`suites.json` has exactly one live `scan: true` case per derived chapter whose
`id` is the path sans extension; each pins header's `--only` names its own
path); `derived_chapters_run_engines_before_breaks` (no `break` before an
`engine`; exempt only at the end; added with item 7);
`opening_note_navigation_matches_the_manifest` (the `### Part …` headings and
link order under them equal the manifest's landed entries; every relative link
and `#fragment` in the ordered inputs and the appendix resolves);
`spine_contents_block_is_current`.

### D3 — The heart: Nell, the definition, the recurring section

**The structural definition of "most vulnerable" — a test on absent entries,
checkable by query, never a field:**

| Clause | What is absent in the record | Where |
|---|---|---|
| No private route to the floor | no `parent/2`, `family/1`, `home/1`, `married/2`, `sibling/2`; no life-course role record naming a Household, Caregiving or MaterialSupport role for them | admits roster; FAMILY-LIFE-ORDINARY role vocabulary |
| Cannot secure help, challenge or remedy alone, and nobody is chosen to | no ChosenSupporter/IndependentAdvocate support record; no `challenge/3` they authored | FAMILY-LIFE-ORDINARY; `challenge` at stratum 0 |
| Not yet acted upon by public power | no `judge/2`, `put/2`, `capture/2`, `restrain`, arrest order, active custody | Article 6, T3, PUBLIC-SAFETY |
| Presence not guaranteed | only `born/1` or `at/2` can enter them; if neither is written, `person` does not derive | the universal-standing rules (`all $child: born($child) & ~public($child) -> person($child).` and the three `at` roots) |

The design's answer: **the public duty reads none of those premises** — every
debt and every barrier has `person($x)` as its whole body. Why it is not a
status: `vulnerable(Nell).` is unadmitted vocabulary and is pinned as refused
(the `rich(Adam).`/`dangerous(Adam).` shape in chapter 1's suite); the test is
computed from silence and reads into nothing; it captures a set (the
unconscious adult nobody has come for, the person whose only supporter is the
one accused of failing them, the unaccompanied newcomer under control — already
pinned at `tests/pins/mobility/boundaries/standing-rights-and-no-coercive-consequence`
— the person whose language nobody present speaks) and excludes, by the same
test, the prisoner and any child with a recorded parent. That partition is the
pair.

**The cast constant** is `Nell` (unused anywhere; `Ori` is the never-entered
child, queried and never written). Nell enters by **fixture only**:
`tests/pins/records/child_with_nobody/fixture.nibli` = `born(Nell).` with its
own scenario case, listed in `suites.json` for the heart chapter's case and for
every slot chapter's case. Never added to the constitution's cast (the Zed
precedent: a cast entry enters every roster query in every case). No `born(...)`
fact is exercised anywhere in the repository today; this is the first exercise
of the birth root.

**Chapter 01 — *The Child With Nobody*** (`01-the-child-with-nobody.md` +
`.pins.nibli`; ceiling ~3,000 words; the record is one line and the density
belongs to the pins). Register rule for every sentence about Nell: an entry
present, an entry absent, a conclusion that derives, one that does not, or a
rule stated — no predicate of state (alone, hungry, safe, afraid). "Nobody"
always means record absence. It stands first honestly because its positive
derivations rest on `{born, public}` → `person` → the rules whose whole body is
`person`; everything that reads `prisoner`, adulthood evidence or the
two-witness machinery is kept out and lands in the slots.

- *Preamble — one entry.* The line, the absences, the rule that reads it.
  Pins: `person(Nell)` TRUE; `family/home/prisoner(Nell)` FALSE with
  what-would-flip-it comments. Derived-chapter wording: "The record for this
  chapter holds one line about Nell: that Nell was born. No entry names a
  parent, a family, a home, a household, a supporter, a case, an order, or a
  teacher. Nothing in the record says that anyone has done anything to Nell or
  for Nell. This chapter asks what the rules make of that record."
- *Everything owed.* Eight `owe(State, K, Nell)` and eight `entitled(Nell,
  event { … })` TRUE — enumerated, not sampled.
- *No one is presumed to provide.* Every FAMILY-LIFE barrier TRUE for Nell
  (PublicFirstCareContinuity, KinshipNoPersonalCareDebt, ChildIndependentRights,
  ChildVoiceNoAgeFloor, ChildSeparationLastResort, SupportBeforeRemoval,
  FamilyStatusNoConfinement, HouseholdNonprescription, CareStatusNondelivery,
  Book2LifeCourseBoundary, …); `:refuse` `vulnerable(Nell).`; the
  absence-to-confinement refusal pointed to, not duplicated (cite
  `counterfactual/no-family-confinement-wall.pins.nibli` in the header).
- *The duty that needs no family in the record.* Prose; header cites
  `tests/pins/family-life/life-course/continuity-needs-no-family-role`; the
  executable pin lands in 04's slot. Says the family ships dormant and proves
  no care continued.
- *What does not follow.* `eats/dwell/healthy/secure/meets(Nell)` FALSE with
  the rule shape stated (a record that something reached Nell plus an
  independent witness who was not the giver) and the Yenna mirror named
  (`tests/pins/delivery/received-outside-custody`); `learn(Nell)` FALSE. "The
  record does not say that Nell ate. It does not say that Nell did not eat. It
  says nothing, and the rules derive nothing from nothing."
- *The child the record never entered.* `person/owe/prevents(…Ori…)` FALSE;
  counterfactual base `counterfactual/no-birth-standing` (delete the birth
  rule; mirror of `no-first-contact-standing`) with a
  `counterfactual/no-birth-standing.pins.nibli`: Nell owed nothing, barred from
  nothing, no alarm notices.
- *What this chapter does not establish.* No birth observed, no registry, no
  witness authorised, nothing arrived, no service, no continuity performed;
  the argument for why this case protects everyone is the author's, in the
  opening note.

**The recurring section** — `## The child with nobody`, heading identical
wherever it appears (the ledger keys on the exact heading), 60–140 words in the
form "For the child with nobody, this rule [does X / reaches nothing]. [The
derivation or the absence.] [The entry that would have to exist to change
it.]"; pins in the chapter's own pin file against the fixture. Which chapters
(final numbers): 03 applies (the one entry is on the list; `vulnerable` refused
here too); 16 applies (verify a public-body fact for the owing body first);
25 applies (two-body voiding of Nell leaves all eight `owe` TRUE); 27 applies
(the pair executed: Zed `dwell`/`expresses` TRUE through confinement, Nell both
FALSE, both owed); 04 applies (a witnessed receipt probe derives `eats(Nell)`;
the open gap — who authorises the witness for a person who cannot choose one —
named, not closed; continuity duty pinned here); 18 applies via voice, not vote
(`decide(Nell, Ballot)` FALSE; `ChildVoiceNoAgeFloor` TRUE); 28 applies
(`dwell(Nell)` FALSE; nothing places Nell); 22 applies (`permanent(Art_Floor)`,
`permanent(Art_Person)`); 29 applies (`travel(Nell)` TRUE, `restrain(Nell)`
FALSE); 30 applies (`err(Nell, Isolation)` FALSE; new case on base
`counterfactual/undelivered-marker` with the fixture: `err(Nell, Undelivered)`
TRUE — the alarm that would notice the child fires on everyone). No slot, with
reasons: 10 (Cira covers it), 23, 24, 26. Every NEW/PROMOTED chapter carries its
slot from the day it lands. Part V restates the pair as argument.

Ledger: one row per slot section, `setting: "the child with nobody: one birth
entry and nothing else"`, postures `["receives", "is acted upon"]` only,
trajectory per slot, `basis` = the chapter pin file. Tests in
`reader_coverage_tests.rs`: `CHILD_SLOT_CHAPTERS` and `CHILD_SLOT_EXEMPT`
(with reason strings) asserted by membership; every slot chapter's suites case
lists the fixture; **no domain's coverage may come from child-slot rows alone**
(the anti-monoculture guard gains the child form beside its custody form).

### D4 — The opening argument and the names

**Placement.** Keep `00-opening-note.md:3-16`; insert the argument block after
line 16 so the book opens on it; amend line 18's back-reference. Order inside
the block: the definition → the methodological claim, labelled as argument →
the pair → the affect warning → the title. Delete `### Why the prisoner
appears so early` (156-175) and replace with `### The child and the prisoner`
(Reader's-Map register, no argument; keeps the sense of 166-170); item 7 later
rewrites 22-24 and 729-741 to the ordering rule. Candidate wording (exempt,
first person; lands under delegated approval at item 6; may not claim an
arrival, a guarantee of growth, or an operation):

> I should say what I mean by the most vulnerable person, because the phrase
> usually hides a judgement and I want it to hide nothing. I mean a test on
> the record, not a description of anyone. First, every private route to what
> a person needs is absent: no parent, no relative, no household, no
> association or charity that the rules could presume is feeding or housing or
> watching over them. Second, they cannot get help, make a challenge, or be
> heard on their own, and nobody has been chosen to do it for them. Third,
> public power has not acted on them — nobody has arrested, placed, assessed
> or registered them — so nothing guarantees that they appear in the record at
> all. A newborn with nobody meets every part of that test at once, which is
> why this book returns to that child in nearly every chapter. But so does an
> unconscious adult nobody has come for, a person whose only supporter is the
> one accused of failing them, and a stranger under this society's control
> whose language nobody present speaks. The test names which entries are
> missing. It never names what somebody is.
>
> The design's answer to the test is the shortest sentence in the book: the
> public duty does not read any of those entries. What is owed follows from
> being a person, and being a person follows from a birth, a first contact,
> being within the society's reach, or being under its control — nothing more.
> There is no field in the record for *vulnerable*. A rule that tried to read
> one would be refused for using a word the record does not admit, and the
> chapter on evidence shows it being refused. That is deliberate. A society
> that first has to decide who is vulnerable has built the instrument for
> deciding who is not.
>
> Now the argument, and I mark it as argument because no machine checked it.
> If the person the record knows least about is protected by rules that read
> nothing but personhood, then everyone about whom the record knows more is
> protected by the same rules, because the extra entries are not conditions. I
> believe that building against this case surfaces defects that would
> otherwise reach people who are easier to overlook. The one instance I can
> point to is executable: the standing rules were built for the person who
> cannot produce a record, and the same rules turned out to cover the
> undocumented adult, the person present without papers, and the person under
> this society's control. The general claim is mine. The instance is the
> machine's.
>
> The prisoner is this book's other test, and the two fail in opposite
> directions. Public power has acted on the prisoner, so the record is full — a
> case, a judgment, witnesses, a custody authority with a window. It has not
> acted on the child, so the record is nearly empty. In the record supplied
> with this book the prisoner is the only kind of person for whom shelter and
> a recorded voice actually derive, and they derive through confinement. For
> the child, nothing arrives at all. If you want to know whether a rule
> protects people or merely describes them, run it against both. Every chapter
> that follows does.
>
> You will notice that this book never tells you what the child feels. That is
> not coldness and it is not an oversight. The record holds one line about the
> child, and a book that added fear or comfort to it would be writing exactly
> the kind of entry the record refuses to hold. If you find yourself supplying
> the feeling, notice that it is you supplying it — and then ask, of every rule
> in the book, the only question the book can answer: what does this do for a
> person about whom it knows nothing except that they exist?
>
> That is why the book is called *The Rights Nobody Has to Earn*. The word
> doing the work is *nobody*: the child has earned nothing, can earn nothing,
> and is owed everything the rules owe anyone. The rest of the title — worked
> out to the point where it catches its own failures — is the other half of
> the same page: the design can say with total precision what the child is
> owed, and cannot say that any of it arrived.

Also in the opening note: glossary entry "the child with nobody: the book's
recurring test record — one birth entry and no other entry about the person; a
record shape, not a status or a finding about anyone"; Person entry gains "and
derives it from a birth or an encounter"; named-cases index gains Nell and Ori;
domain map gains chapter 1 under personhood/life course and body/care; `:172`
"through the prisoner framing above" → "through the child and the prisoner".

**Names — where spoken.** `README.md:1,9-14` and `book-1/README.md:3` name
both; the opening note's first page names Book 2 ("…not a plan for getting to
one from here — that plan is the second book, *What It Would Take*") and speaks
Book 1's title in the block above; `15-the-five-joints.md:556` and `:836` name
Book 2; `method.md:942` → "They are the next book, *What It Would Take*. This
one, *The Rights Nobody Has to Earn*, was the destination; that one is the
road."; `book-2/TODO.md:1` gains the title and a dated line. Derived chapters
stay name-free (R2). Rejected Book 2 titles, so they are not re-proposed:
*Getting There From Here* (implies the author knows the way); *What It Costs*
(invites the "warning" misreading the first readers already made); *Building
the Floor* ("floor" is jargon to a stranger, the reason Book 1's title avoided
it); anything saying "how it works" or "how they arrive".

### D5 — The new and promoted chapters

**Conventions for every chapter item:** paired `book-1/NN-slug.pins.nibli`
(CONTENT pin, `:expect-pins`, header listing supporting `tests/pins/...` paths
as `08-what-you-are-owed.pins.nibli:1-90` does, fresh constants in every
argument position, controls `:accept-scoped`); registered in `suites.json`
`{"id":"book-1/NN-slug","base":"live","fixtures":[...],"pins":[...],"scan":true}`;
one ledger row per `## ` with `family` single-valued (split a mixed section
rather than blur whose Book 2 boundary is being stated); no digits; rule-not-
count; a closing boundary section whose prose matches the ledger's boundary
regex ("establishes no", "does not prove"); the child slot; the opening note's
contents/domain/glossary entries; regenerated `reader-coverage.md`. A
promotion **moves author-approved text verbatim** (delegated approval covers
relocation; substantive rewording needs the exact-version record) and removes
it from the host in the same commit, moving any resolution receipt's `chapter`
with the narrative it binds (the repair-narrated-⇒-receipt detector).

**Rendering a dormant family honestly** (INCOME-SECURITY,
QUALIFICATIONS-COMPENSATION, FAMILY-LIFE-ORDINARY, the delivery routes,
first-contact standing) — copy `08:440-502` and `10:95-114`: ask the cast
question and answer with the record's own no; say what the no is (a statement
about what the supplied record can establish, not a finding about anyone);
state the rule's shape in the conditional and name what would complete it;
show the routes are not blind with a fresh-constant probe in the pin file
("the rulebook knows how it could count; the record reports none"); close on
the Book 2 boundary and the liveness bar ("No sentence in Book 1 may say a
supplement arrives"). Pin-file pattern: "ships dormant" FALSE block over the
cast, probe blocks TRUE, conjunct-stripping FALSEs, floor/recognition
invariance.

**09 — Earning Above the Floor.** Preamble: work is not a condition
(`WorkChoiceRefusalExitDenial`, `WorkConstitutionalGate`,
`FloorContributionCondition`; `economic-constitution.pins.nibli`,
`counterfactual/no-economic-work-freedom`, `no-economic-floor-gate`). *Whose
work it is* (worker status follows control, custody labour; FS-CCE-236..245,
labour duties). *Acting together* (posture `associates`: collective action,
no individual conscription, blanket strike ban refused; FS-POW-083). *A
licence needs a reason, a certificate opens no door* (FS-POW-061/070/080;
`grant/3` read by nothing; `counterfactual/no-certificate-rule`). *Pay is a
record with the kind named in it* (`promise/3` → `provide/3`, payer ≠
attester, the coupling barriers, restitution on a court fraud finding;
`qualifications-compensation.pins.nibli`; moved prose 08:770-779, 10:107-114).
*A supplement above the floor* — dormant (`pay/4` + `insure/3`, adjudicator ≠
carrier, `PublicGuarantee`, purpose-reuse and score-reuse barriers;
`income-security.pins.nibli`, `unguarded-contribution-reader`; moved prose
08:758-768, 10:95-105). *When the money runs out* (insolvency block;
FS-POW-079/069). Boundary: rates, budgets, actuarial assumptions, wage levels,
scheme administration, certification bodies, adequacy, funding, payment and
arrival are outside the record. No new cases required; optional composition
case `tests/pins/composition/economic-and-floor-joined`.

**11 — What Money Cannot Buy.** Preamble: no form of provision is privileged
and price cannot gate the floor (`EconomicFormSuppression`,
`OwnershipFormGuaranteed*`, `PaymentInstrumentAsDelivery`; FS-CCE-223..228).
*Owning without owning the floor* (conditional possession/use/transfer/
inheritance, tenure plurality, eviction continuity, acquisition FS-POW-062/071,
estate debt, knowledge exclusivity FS-POW-063). *A promise is not a trap*
(posture `chooses`: consent, corridor waivers, enterprise capacities, limited
liability, treasury electoral spending → cross-ref 18). *When a private power
owes public duties* — PUBLIC-SCALE-VOCABULARY (eight named grounds, five
function classes, tier pairings, FS-POW-064 finding → 065–071 remedies,
expiry; `public-scale-vocabulary.pins.nibli`; moved prose 08:781-800). *Public
money* (fiscal block; FS-POW-072..076, 085). *Money you can hold in your hand*
(non-digital backbone FS-POW-086..088, monetary office 077, credit 078). *Which
tier decides* (federal block). *When an economic power ends, and what does not*
— the family's strain passage (`counterfactual/no-economic-independent-current-review-0NN`,
carry contracts). Boundary: the record values no property, calculates no tax,
clears no market, measures no dominance, authenticates no financial record; the
engine can say which named ground a finding claims and refuse one that claims
none, and cannot tell whether a network effect exists.

**12 — The Same Route for Everyone.** Preamble (the floor says what nobody may
be left without; equality asks whether a rule marks some people for a worse
route). *The forms it takes* (direct/indirect/systemic/multiple/intersectional/
associative, accommodation denial, segregation, harassment, retaliation). *Who
is bound, and where private life stays private*. *What a distinction has to
prove* (`EqualityRemedialBurdenRefusal`; `statistics/data-wall/criminal-burden`).
*Accessibility is not a ninth floor item* (moved 08:646-653;
`knowledge/accessibility/positive`). *Patterns without verdicts*
(`statistics/pattern`, `rebuttal`, `nonparticipation-is-not-a-penalty`; moved
08:655-661). *A measure with an end* — strain passage. *Repair, and who may
ask*. *The old distinctions under the new test* (family/home/maturity/
conviction/custody/official-status substitutions — forward references to
Part IV). Boundary: not a population-statistics, identity-authentication or
institutional-liveness system. **Interface first:** SUBSTANTIVE-EQUALITY is
exactly the shape FAMILY-LIFE-COURSE was before 2026-09-16 — 54 person-held
barriers and no ordinary half — so item 10 builds `equality-source.json` +
`./generate.sh equality` (block `SUBSTANTIVE-EQUALITY-ORDINARY`, a **new**
block name — reusing an existing block name silently deletes it) with at
least `equality/measure/{positive, without-EndScope,
expired-continuation-stops}`, `equality/accommodation/{positive,
request-and-certified-nonresponse}`, `equality/proceeding/presumption-from-pattern`
(joins `statistics/pattern`), a defect route; until it lands the two sections
would be barrier plus stated boundary, tagged `contested`, never narrating a
measure ending.

**Sections rendered per family** (one family per section so each gets its
tag): liberties → 08 "The things power may not prescribe" (FS-CCE-09..18, the
ten liberties in the LIBERTY-ECOLOGY block; person-held limits on power, not
delivery; public institutions answer private interference, no general
horizontal effect); environment/Class 9 → 13 "Axes that do not trade"
(FS-CCE-19..33); public-scale → 11; income/qualifications → 09; state-form
structure → 17.

### D6 — The fold: `new-book-plans/` → `book-1/appendix/` + `book-1/source/`

**Appendix.** `book-1/appendix/{README.md, decisions/, contracts/, briefs/,
maps/}`; the `book-1-` prefix is dropped on the move, the
`-decision`/`-contract`/`-brief` suffixes stay. **45 files** — 18 decisions
(incl. `full-society-boundary-decision.md`), 21 contracts, 3 briefs, 3 maps
(`constitutional-coverage-map`, `constitutional-taxonomy`, `red-team-index`).
Files keep their CC-BY-4.0 headers; `LICENSING.md` gains one clause that the
per-file header governs wherever the file lives. `appendix/README.md` carries
R4 and the fold map.

**Source.** Everything else in `new-book-plans/` (156 entries today, minus the
45) moves by **flat rename** to `book-1/source/` — same basenames, same
`counterfactual/` and `reader-evidence-pilot/` subdirectories — so the rewrite
is one prefix and no basename changes. Sub-structuring `source/` is
deliberately not done here; it can be a later item with the same tool. Then
`new-book-plans/` is empty and removed.

**What the tool rewrites (measured at `c4cb3582`):** `full-society-ledger.json`
44,744 lines carrying the 43,259 needle paths (prefix only; needle text
unchanged); `full-society-power-source-manifest.json` 483;
`assertion-surface-contracts.json` 179 and its audit 135; the other frozen
audit pairs (~120); `tests/pins/suites.json` 140 (bases, fixtures, pins);
`src/authoring.rs` 21 and the generators' `SOURCE`/`REPORT`/constitution path
constants (94 lines across `src/authoring/*.rs`, 29 in `src/*.rs`); CLAUDE.md
61; `README.md` 22; `book-2/TODO.md` 20; pins-file header comments; the three
reader-side sources' `basis`/`chapter` paths; `verify.sh`/`generate.sh`/
`combine.sh` carry no path (checked). `reviews/deepseek.md`'s 12 mentions are a
quotation and stay. No Rust code reads any appendix file; the move is
invisible to the five order sites (non-recursive, digit-prefixed).
`full-society-ledger.json`'s `bound_sources_sha256` for two decision files goes
stale when the fold rewrites their cross-references — dormant (nothing reads
it), one sentence in CLAUDE.md's historical paragraph says so.

### D7 — The breaks, read last

No merges: the only candidate (25+26) is refused for a mechanical reason —
chapter 5's closing exhibit is an unscoped `:accept` premise, so a concatenated
pin file would run the clawback pins against a widened base, the vacuous-green
trap already recorded. Trims are executed by the Phase B moves (27 −standing,
28 −"What that does not fix" +14's duplicate alarm section, 30 −duty chain).
**Must not be cut, because a ruling or receipt names it:** Ruk's placement and
severity; Lupo's void; Cira's clawback and `person(Fin)`; the Zed composition;
the either-alone pair Adam/Voss/Nia; Don/Sly/Kel/Rex/Zeno; Vex penless and
Ambi/Solo; the discriminator pair `obliged(Review, Ruk)`/`obliged(Ruk, Review)`;
every phrase bound by the twelve receipts. Exempt framing (delegated approval):
opening note 729-741 → the ordering rule and "why the child appears first and
the prisoner last"; Part V's coercion joint gains one sentence that the breaks
are read last. Tests: `breaks_come_after_engines` (coercive passages fewer than
a quarter of engine passages; Part IV prefixes sort after every engine prefix),
`every_constitutional_family_is_projected_by_a_passage` (families read from
the block markers; `UNRENDERED_FAMILIES: [&str; 6]` shrinks to empty and the
empty expectation stays), `every_family_states_a_boundary_where_it_is_rendered`.

## Items

### Phase A — the heart and the order

1. **Record the rulings, and name both books.** CLAUDE.md: R1–R7 as bullets
   (the child bullet after the 2026-08-17 one, with an inline supersession
   there; dated supersessions on "strictly computed" and on "exactly once";
   Files-section entries for `book-1/appendix/`, `book-1/source/`,
   `book-1/contents.json`, `tools/`);
   `book-1/appendix/decisions/child-with-nobody-decision.md` (D3 + the
   definition + the re-measurement method) and
   `book-1/appendix/decisions/reading-order-and-appendix-decision.md` (D1
   table, D2, D6, D7, the rejected Book 2 titles) — the directory's first two
   files; supersession notes in the 2026-08-17 decision (§3–§5) and the
   narrative-register decision ("No fourth exempt element"); `README.md`,
   `book-1/README.md`, `book-2/TODO.md:1` name both books and correct
   "exactly once"; then cut the design sections above from this file, leaving
   pointers. *Done when:* `cargo test` green; `./verify.sh` green (no formal
   change); the needles that point into CLAUDE.md still resolve exactly once
   (hand-checked until item 2).
2. **Give the book a contents manifest, a relocation tool, and
   reference-integrity tests.** `book-1/contents.json` in the *current* order
   with roles/groups and no `rule` field; `src/authoring/contents.rs`; the three
   Rust sites, `combine.sh`, the pilot line switched to the manifest;
   `tools/relocate.py`; `reference_integrity_tests.rs` with the needle baseline
   measured **before** anything moves. Nothing moves. *Done when:* `cargo test`
   green with the fixed 14 and the two literal filenames gone; `./verify.sh`
   green; `bash combine.sh` and a pilot build produce the same ordered inputs
   as before.
3. **Move `new-book-plans/` under `book-1/`: the planning record to
   `book-1/appendix/`, everything else to `book-1/source/`.** One map
   (`tools/maps/2026-09-16-fold.json`: the 45 appendix moves with their
   renamed basenames, plus the flat `new-book-plans/` → `book-1/source/`
   prefix for the rest) → `apply` → `check`; `appendix/README.md` (R4 and the
   map); LICENSING clause; CLAUDE.md Files section rewritten (`new-book-plans/`
   entry retired; `book-1/source/` and `book-1/appendix/` described), the
   "no fourth exempt element" and `bound_sources_sha256` annotations;
   `book-1/README.md` gains the two subdirectories in its conventions;
   `book-2/TODO.md` cites repointed; the method part's pointer sentence names
   `book-1/appendix/` and `book-1/source/` (a pointer, within sealed rule (d)).
   *Done when:* `check` passes (every `new-book-plans/` string gone outside
   `reviews/` and git history; constitution, rule-family sources, pins
   non-comment lines and `4-strata.py` byte-identical; `git status` exactly the
   planned renames; `new-book-plans/` absent); `cargo test` (needle set
   unchanged modulo rewrite); `./verify.sh` (the runner's constitution path and
   every suites base/fixture/pin path resolve); `./generate.sh spine` and the
   three report generators byte-stable on a second run; `bash combine.sh` and a
   pilot build unchanged.
4. **Run the constitution against a person with one entry.**
   `tests/pins/records/child_with_nobody/{fixture,expect}.nibli`
   (`born(Nell).`; person TRUE, eight owe, eight entitled, every barrier,
   actualities FALSE, `vulnerable(Nell).` refused, Ori FALSE); base
   `counterfactual/no-birth-standing` + its pins file; the Nell case on
   `counterfactual/undelivered-marker`; `suites.json` entries. No prose. *Done
   when:* `./verify.sh --only` each, then full; `:expect-pins` floors set.
5. **Write the heart chapter — *The Child With Nobody* — as chapter 1.**
   Insert at 01 (manifest edit → `plan` → `apply` shifts the fourteen by one;
   map `2026-09-16-heart.json`); chapter prose (D3 shape) + paired pins file +
   suites case listing the fixture + ledger rows + opening-note contents/
   glossary/index/domain entries; `claim_discipline` and coverage green. *Done
   when:* `check`; `cargo test`; `./verify.sh`; `reader-coverage.md`
   regenerated; the chapter contains no digit and no predicate of state.
6. **Open the book on the argument.** Opening note: the argument block (D4),
   `### The child and the prisoner` replacing "Why the prisoner appears so
   early", line 18 back-reference, `:172`; Book 2 named on the first page;
   Part V `:556`/`:836` and `method.md:942` seam sentences; recorded as
   `session-drafted, author-approved under delegated approval (2026-09-16)`.
   *Done when:* navigation test green; `cargo test`; `./verify.sh`; no arrival
   verb, no guarantee-of-growth claim (register audit before commit).
7. **Rule the reading order and renumber once to the final table.** Manifest
   gets `rule` and the D1 table with `planned` entries; `plan` → map
   `2026-09-16-reorder.json` → `apply` → `check`;
   `derived_chapters_run_engines_before_breaks` added; `3-spine.md` header
   corrected, generated contents block, old hand list re-headed "superseded
   2026-09-16 — pre-reorder numbers"; opening note 22-24 and 729-741 rewritten
   to the rule; `method.md:221-253` rewritten (the stratification is the
   derivation record; the reading order is ruled); `book-1/README.md:8-11`,
   `README.md:44`; the three forward cross-references the validator reports
   are reworded; regenerated reports. *Done when:* `check`; `cargo test` (incl.
   `spine_contents_block_is_current`); `./verify.sh`; `bash combine.sh` yields
   the landed chapters in manifest order.

### Phase B — the engines (one chapter per item; unrendered mass first, then a receiving chapter before its host is cut)

Each Phase B item: chapter + pins + suites + ledger rows + child slot +
opening-note entries + host cut + `UNRENDERED_FAMILIES`/`THIN_POSTURES`
membership updated + regenerated reports + `cargo test` + `./verify.sh`.

8. **09 — Earning Above the Floor** (NEW; cuts 08:758-779, 10:95-114; closes
   INCOME-SECURITY, QUALIFICATIONS-COMPENSATION, half of ECONOMIC).
9. **11 — What Money Cannot Buy** (NEW; cuts 08:781-800; closes ECONOMIC,
   PUBLIC-SCALE-VOCABULARY).
10. **Implement the substantive-equality ordinary half** (`equality-source.json`,
    `src/authoring/equality.rs` from `family_life.rs` by substitution, block
    `SUBSTANTIVE-EQUALITY-ORDINARY`, contract card, the cases in D5;
    `CHALLENGE_SKELETON` gains the family). Interface first, passage second.
11. **12 — The Same Route for Everyone** (NEW; cuts 08:637-661; closes
    SUBSTANTIVE-EQUALITY).
12. **08 — What Nobody Has to Ask Permission For** (PROMOTED + liberties
    section; half of LIBERTY-ECOLOGY).
13. **13 — A Place in Which Life Remains Possible** (PROMOTED + Class 9
    section; closes LIBERTY-ECOLOGY).
14. **05 — Whether It Arrived** (PROMOTED from 08 and 11).
15. **06 — When There Is Genuinely Not Enough** (PROMOTED).
16. **07 — Who Owes, and What Follows** (PROMOTED from 08 and 14; receipt
    `unread-duty` moves only if its phrase moves).
17. **15 — Arriving and Belonging** (PROMOTED).
18. **14 — Holding a Role in Somebody's Life** (PROMOTED).
19. **17 — How Public Power Is Built** (PROMOTED + new state-form sections).
20. **19 — What May Be Kept About You** (PROMOTED from 01).
21. **20 — A Crisis Does Not Suspend the Republic** (PROMOTED).
22. **21 — A Way to Be Heard** (PROMOTED).
23. **02 — Who Counts** (PROMOTED from 07; opens by generalising chapter 1's
    birth root to the other three; may add a `records/first_contact_*` case).
24. **The child slot in every kept engine chapter** (03, 04, 16, 18, 22; the
    exempt list 10/23 with reasons) and the membership tests
    (`CHILD_SLOT_CHAPTERS`, fixture listing, anti-monoculture child form).

### Phase C — the breaks, read last

25. **The child slot in the break chapters** (25, 27, 28, 29, 30 incl. the
    Undelivered case; 24/26 exempt with reasons); Part V's coercion joint
    sentence and the pair in its family section.
26. **Close the rebalance.** `breaks_come_after_engines` coercive-share bound
    and `every_family_states_a_boundary_where_it_is_rendered` landed;
    `UNRENDERED_FAMILIES` empty; `THIN_POSTURES` re-censused; length invariant
    re-measured (`wc -w`, appendix and source excluded) and recorded; R6
    re-measurement recorded; CLAUDE.md implementation-supersession notes on
    R1/R3/R4/R5; no `planned` entry left in the manifest.

## Verification for every item

- `cargo test --release --bin generate`, then `./verify.sh` (complete; ~15
  minutes; nine known-defect pins still reproduce), sequentially. After a
  renumber or move additionally `python3 tools/relocate.py check <map>`,
  `./generate.sh spine`, the three report generators byte-stable on a second
  run, `bash combine.sh`, and a pilot build into its temp directory.
- Prose items get a register audit before commit: no arrival verbs, no
  guarantee-of-growth or operation claims, no interior state, no digits, no
  cast name in a passage.
