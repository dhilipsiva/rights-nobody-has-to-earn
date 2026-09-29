<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1: revision toward the best book of its kind

Created 2026-09-24 at the author's request, "Create a TODO.md file", from the
[revision plan](new-reviwes/revision-plan-9.5.md) and its
[prose lint](tools/prose_lint.py). Another AI assistant wrote the plan
outside this repository, from the 170-page review PDF built at `93fa5662`. Its
target is 9.5, "the best book in its genre that a serious reader could pick
up", and it places the manuscript at about 6: ideas 8, honesty 9, argument 7,
evidence and scholarship 5, readability for the stated audience 3. The
[2026-09-21 review](reviews/2026-09-21-final-manuscript-review.md) gave 9/10.
Both are editorial judgments by AI assistants, not measurements or independent
endorsements, and closing items does not make either score true. The plan's
"Review N" references are to reviews pasted into that assistant's
conversation, not to the files in `new-reviwes/`.

**The plan is evidence to weigh, not an instruction.** Its premises about this
repository were checked against `93fa5662` on 2026-09-24.

- **Confirmed:** the floor's bearer has no jurisdiction or scope (item 36); a
  defendant can disclose after charge at almost no cost (item 37); the floor's
  names drift (item 36); "void" and "recognition loss" outlast the mechanisms
  they name (item 41); test-harness names appear in prose (item 51); no chapter
  shows the text of an article (D4); the coercion argument rests on forced
  relocation rather than confinement (item 47); web artefacts reach print
  (item 64).
- **Disproved:** that the scarcity manager writes the comparison it is then
  reviewed on (item 40).
- **Reproduced:** run over the Markdown, `prose_lint.py` matches the plan's
  per-chapter figures within rounding, and every ordered input fails its
  negation threshold today; `tools/prose_lint.py` now holds each to its figures.
- **Checked:** the plan's citations, which it says were written from memory,
  against primary sources (item 46); `registry/plan-sources-2026-09.md` records
  the one that failed and where the plan misdescribed a source.

Several of its central recommendations would supersede author-ratified rulings.
The author ruled on each of them on 2026-09-24; the answers are under *Rulings
ratified 2026-09-24* and recorded in `CLAUDE.md`.

A second list, *From 8/10 to 9.5+*, added as `TODO-9.5.md` in `d5c9ccd7`, was
merged into this tracker on 2026-09-29 at the author's request and weighed the
same way. Its premise check, and where each of its items went, open the
*Ordered revision backlog toward 9.5+*.

## Execution contract

- Each newly added item begins pending; a proposed alternative is not enacted
  by its inclusion in this tracker.
- Work one numbered item at a time, in order. Each item includes its necessary
  rules, pins, prose and checks. A question that would supersede an author
  ruling goes to the author rather than being decided on a guess. Keep later
  items open until their own completion criteria are met.
- Recheck each premise the plan supplies against the current chapter,
  constitution, substantive pins and controlling decision, and correct the plan
  where it is wrong. A suspected defect is not an executed failing case.
- The constitution and substantive pins own derived behavior. Change the
  source and pins first and the prose last. Update an owning JSON or generator
  when appropriate; do not patch only generated output. Maintain affected
  existing coverage records instead of creating parallel inventories.
- The author's standing delegated approval (2026-09-13) covers recommended
  changes and prose inside the current rulings, the 2026-09-24 rulings included;
  it does not supersede an author ruling. Record substantive policy
  supersessions in their existing controlling decisions and summarize them in
  CLAUDE.md; retain the actual approved text. A TODO recommendation alone does
  not silently supersede constitutional law.
- Preserve universal standing, unconditional floors, the ordinary-life balance,
  the chapter/pin relationship, and the Book 1/Book 2 boundary. Book 1 assigns
  constitutional duties and remedies; Book 2 owns operation and transition,
  including staffing, timings and capacity. Book 2 stays collection-only until
  Gate C. Preserve the legacy manuscripts.
- Preserve the unnumbered epigraph and method, editorial order and the
  majority-derived length rule. Ruling D2 adds one labelled argument section to
  each derived chapter and measures the length rule by section; item 53 built
  that tooling, items 54–57 wrote the sections and opening cases of Parts
  I–IV, and item 59 made Part V a synthesis with its own opening case. The
  existing appendix is a carried archive, not an extra channel for new
  reader-facing arguments.
- **Current design throughout — author instruction, 2026-09-18.** Every
  reader-facing part, including the opening, Part V and optional method, must
  describe the design of the edition being read. Remove accounts of the book's
  own development, earlier rules, repairs, additions and before/after versions,
  even when accurate or instructive. Explain the current rule, its rationale,
  consequences and limits directly. Keep revision history in Git and repository
  decision records outside the reading sequence. This is a continuing authoring
  rule, not a one-time reduction of repetition. Historical evidence about the
  world, explicitly tested alternatives and temporal behavior within the current
  model remain admissible; they must not narrate superseded designs. Preserve
  immutable published editions; "current" means the design bound to that edition.
- The child test follows ruling D5: a section returns where the one-entry record
  produces a different or instructive result, and each removal is recorded in
  `CHILD_SLOT_EXEMPT` with its reason. Never delete the membership requirement.
- Preserve the deliberate flat register. Improve cases through selection,
  sequence and clear roles, without invented dialogue, biography, feelings,
  testimony or successful outside events. Normative argument and documented
  evidence stay in their authorised places: the opening, the Part openers, the
  argument sections and Part V (rulings D2 and D3). Derived sections stay
  derived.
- The plan's own warnings stand: remove disclaimers rather than adding them;
  fix negation, jargon, names and structure rather than shortening sentences
  that are already short; move each objection to its mechanism rather than
  adding an objections chapter; go deeper on fewer domains rather than adding
  new ones; and do not let a plain-language constitution become a second
  free-hand paraphrase.
- Use focused substantive checks during implementation, then the complete
  `./verify.sh` for item completion, plus relevant existing development checks
  when their machinery changes. Measure changed prose with
  `tools/prose_lint.py --check`, and record improvements with `--ratchet`; it is
  a development check and adds no gate to `./verify.sh`.
  Review prose consistency separately. Report actual commands, elapsed time,
  failures, contradictions and incomplete checks. Do not introduce receipts,
  hashes, freshness gates or administrative audits.
- Do not delete substantive regression coverage to obtain a pass. When policy
  intentionally changes, replace superseded expected behavior explicitly while
  retaining the old exploit or counterexample as a regression scenario.
- Commit and push each implemented item after its required checks under the
  standing repository instruction. Preserve unrelated work. Delete completed
  tracker items; the coherent commit and current source retain their result.
  The website rebuilds the book and its companion from `main` at each of its
  deploys, so leave `main` coherent after every item.
- The [submission proposal](submission/README.md) describes the revised
  manuscript at the commit item 65 records; any proposal already sent to a
  publisher describes `93fa5662`, which stays identifiable.
- External human review is optional, not a completion dependency (2026-08-15
  ruling); the plan's readers, experts, reproduction and red team remain
  welcome as optional evidence, though the author closed item 66 on
  2026-09-25. Never invent reader testing, expert endorsement, empirical
  evidence or operational success. Do not treat AI agreement as independent
  validation.

## Resolve before defending — author instruction, 2026-09-18

This rule governs every item and its completion criteria. Exposing, labeling or
explaining a defect is not a substitute for correcting the design.

1. Establish the actual failure and its cause against the current source. Try
   substantive repairs, including a different representation, narrower power,
   replacement mechanism, simplification or removal. Examine nonessential
   design choices rather than treating an inherited decision as an immovable
   constraint. Preserve the constitutional commitments stated above and record
   any policy supersession under the standing delegated approval.
2. Implement and verify a sound resolution when available. Test both the
   legitimate behavior that must remain and the harmful behavior that must stop,
   including downstream interactions. Rewrite the reader-facing account to
   describe the resulting current design; do not narrate the repair history.
3. Defend a remaining limitation only when resolution is shown to be impossible
   within explicitly stated, justified constraints. Record the alternatives
   examined, their results and the necessary constraint preventing resolution in
   the existing repository decision record. A failed encoding, inconvenient
   implementation, elapsed effort, tooling limit or lack of a discovered fix is
   not proof that the constitutional problem cannot be solved. State the scope
   of any impossibility result; do not infer universal impossibility from a
   finite search. Uncertainty or an execution blocker leaves the issue open.
4. An adequate fallback defense must explain why the constraint is necessary,
   why the chosen arrangement is preferable to available alternatives, who bears
   the remaining harm, what safeguards and remedies limit it, and what evidence
   would require reconsideration. Merely calling a cost a trade-off, disclosing
   it honestly, or showing that a pin reproduces it does not meet this standard.
5. If neither resolution nor an adequate defense is established, keep the item
   open and pursue redesign, removal or narrowing of the unjustified mechanism
   or claim. Do not transfer a constitutional defect to Book 2 to close it.

Completion must distinguish a verified repair, a finding disproved by current
evidence, a necessarily constrained and adequately defended limitation, and an
unresolved defect or blocker. Only the first three can close the relevant issue.
Reader-facing prose states present rules and justified limits; the attempted
solutions and development history stay outside the book's reading sequence.

## Rulings ratified 2026-09-24

The author ruled on every reserved question on 2026-09-24. `CLAUDE.md` records
the rulings under *The revision rulings D1–D9*, with what each supersedes, and
each controlling decision carries its dated section. They are ratified but
unimplemented until the items named here land.

- **D1. Subtitle, reader and promise.** The subtitle becomes *A worked design
  for a society, with its formal claims made executable.* The primary reader is
  the serious non-specialist, with lawyers and policymakers second. The first
  page and back cover carry the plan's promise: *"A constitution designed from
  the person with nothing, argued in plain language, with every rule published
  so you can test it."* Items 60 and 64.
- **D2. An argument section in every derived chapter.** Each derived chapter
  keeps its pinned, flat account, then ends with one labelled first-person
  argument section: the reason, the strongest alternative with its evidence, and
  what would change the choice. Part V becomes a synthesis. This supersedes the
  2026-08-02 voice boundary's limit of argument to three elements, the governing
  bar the reserved question named; the length and digit rules are measured by
  section. Items 53–59 and 63.
- **D3. Documented cases.** Registry-sourced documented cases may appear in the
  opening (Santoshi Kumari first, with the dispute over her death intact), in a
  short labelled case opening each Part, and in argument sections; never in a
  derived section, and never with an invented inner life. Items 46, 47, 54–57
  and 60.
- **D4. The constitution in the companion.** Numbered plain-language articles
  are published in the companion, not the book. Chapters cite article numbers,
  and each article traces to the rule families, contracts and pins that
  implement it. The method stays the book's technical appendix. An
  interpretation article is added: its burden half traces to existing
  fail-closed rules and their pins, and its protective-reading half is a
  declared non-formal article, listed as a deliberate traceability gap. Items 61
  and 62.
- **D5. The child returns where the result differs.** R1's criterion becomes
  "produces a different or instructive result". Chapter 1 stays the case, and
  each removal is recorded in `CHILD_SLOT_EXEMPT` with its reason. Items 52 and
  54–57.
- **D6. Fast for help, slow for harm.** An act that only gives or preserves
  something for its subject takes effect on one authorised actor, with prompt
  independent review able to correct it. Appointments giving power over another
  person, and every adverse act, keep full prior procedure; delivery evidence
  keeps its independent witness. Items 42 and 43.
- **D7. The structure.** The plan's table is adopted. Chapters 1 and 2, 9 and
  10, and 23, 25 and 26 merge; Chapter 13 splits; "An Ordinary Week" and "Where
  This Could Fail" are added; and the map, glossary, roles and subject index
  move to the back beside the method. The refusal to merge Voiding with Clawback
  is superseded. Item 53, then items 54–60.
- **D8. Nine floor items.** `secure` splits into bodily safety, with no receipt
  route, and material security, keeping the receipt route. Care keeps `healthy`;
  company is unchanged. Item 44.
- **D9. The animal core as enacted.** The core keeps direct subject status and
  the bans on severe avoidable suffering and on killing solely for a listed
  dispensable purpose. The alternative-sensitive food rule stays ordinary
  amendable law bounded by the core. Item 58 defined the terms and argued the
  result in Chapter 13.

## Ordered revision backlog toward 9.5+

Status 2026-09-29: items 01–65 and 67–80 are complete and recorded in
`CLAUDE.md`; optional item 66 is closed without outside validation. The
[2026-09-25 revision review](reviews/2026-09-25-revision-review.md) rates the
revised manuscript 8/10 and publisher readiness 7.5/10: claim discipline holds
across ~98,000 words, but density and repetition keep it below 9 — catalogue
chapters (6, 13, 17, 27), cost-paragraph/argument restatement at chapter
endings, and length for the D1 serious non-specialist reader. A fresh
chapter-by-chapter pass (2026-09-29) concurs at ~7.1/10 average, weakest at
Ch. 7 and Ch. 27. No remaining item is an error the pins contradict. Item
81 is optional; items 82–87, merged below, are companion, navigation and
contribution work, and item 82 corrects a stale example. Chapter numbers refer
to the [current reading sequence](book-1/contents.json).

### The merged list, *From 8/10 to 9.5+*

The list was added as `TODO-9.5.md` in `d5c9ccd7`, where git keeps it. It
rates the book and its companion 8/10 and names five weaknesses: the learning
curve, overloaded chapters, one maintainer checking their own work, a companion
that leaves a newcomer lost, and the distance between the prose and the formal
language. Its premises were checked against `d5c9ccd7` on 2026-09-29.

- **Confirmed:** Chapters 13, 27, 29, 17 and 22 run to 485, 381, 625, 396 and
  271 lines and cover what the list says. The companion's home opens on "Pick
  a person. Walk their forks." under "one person, four lenses" and defines
  neither a fork nor a joint; its dossier shows 21 entries in one grid, each
  marked only as authored discussion or measured joint; and its contents page
  lists the book's pages by Part with no summaries. Every derived chapter's
  argument section says what would change the choice and who bears its cost,
  and nothing gathers them.
- **Already done:** `tools/second_engine.py` replays 129 cases and 1,673
  queries in clingo, each answer agreeing with its pin (item 50); it is a
  cross-check made within the project, not the independent implementation the
  list describes. The method's "What has been checked, and where each check
  stops" lists the pins, the shape checks, mutation testing, the second
  engine, the articles' trace, the audit's open findings and the declared
  defects (item 62), and its first section teaches the notation. Every
  companion verdict shows its query under "Query and engine detail", beside a
  label and a fixed explanation of TRUE, FALSE and refusal, and every fork
  step carries its query in `ui/game.json`, so showing queries needs no engine
  work or query registry. `CONTRIBUTING.md` exists (item 17).
- **Disproved:** no "How a bill becomes law" diagram exists. The back matter's
  five are the duty chain, delivery evidence, the placement ceiling, the
  democratic corridor and the six ways a fact is kept from a consequence.
- **Found while checking:** `CONTRIBUTING.md`'s worked example names Chapter
  4's pin file and chapter by their names before the restructure and quotes a
  sentence item 51 rewrote; `tools/relocate.py` does not rewrite that file and
  no check reads it (item 82).
- **Would supersede a ruling, so not enacted by inclusion:** new chapters
  split from 13 and 27 (D7's table); Nibli boxes inside derived chapters,
  which stay jargon-free and already point to their runnable cases (item 63);
  a history appendix for Part V's cases (D3 names where documented cases may
  appear, and the method's scope is sealed); a summary of about 1,000 words in
  the opening note (D7 allows at most five pages before Chapter 1, and the
  note already runs to about 1,300 words); and repaired defects listed in
  reader-facing pages (current design throughout). Each goes to the author if
  still wanted; the items below meet the need inside the rulings.
- **Restated:** its caps of 350 lines per chapter and a 10% cut in words
  become the lint's word counts with no percentage target, since cuts follow
  chapter purpose and repetition (item 14); its tests on newcomers become
  optional evidence under the 2026-08-15 ruling, not completion conditions;
  and its phases and week estimates give way to this tracker's order.

Where each of its items went:

| Merged list | Here |
|---|---|
| 1. Split Chapters 13, 27 and 29 | 73, 75 and 80 (done) |
| 2. An onboarding path in the companion | 83 |
| 3. Bridge the prose and the formal language | 77 (done), 84 |
| 4. Independent validation | 81, 85 |
| 5. Compress Chapters 17 and 22 | 73 and 77 (done) |
| 6. The design in ten minutes | 86 |
| 7. Pins a non-programmer can read | 77 (done), 84 |
| 8. What would change my mind | 87 |
| 9. Costs and who bears them | 87 |
| 10. A second-engine page | 85 |
| 11. A known-defects page | 85 |
| 12. How to contribute | 82 |
| 13. Part V's historical cases | 80 (done) |
| 14. A book map in the companion | 83 |
| 15. Forks and joints explained | 83 |

### 81. Independent validation — OPTIONAL, needs new author decision

Item 66 is closed (2026-09-25) with no outside expert, reader,
reproduction, or red-team evidence, and none is claimed. The 9.5 plan §12
requires it, but seeking it again needs a new decision per the closure
record. Do not treat AI agreement (including this tracker or any review in
`reviews/`) as validation. If the author reopens: comparative-constitutional
lawyer, economic/social-rights scholar, formal-methods specialist, animal
law scholar, frontline practitioner, three finishing lay readers with
confusion logs, plus independent core-subset reproduction (e.g. Soufflé or
clingo) and a published red-team challenge. Until then this item stays
unopened and blocks no submission.

From the merged list: its second engine is built (item 50), and the method
reports it as a cross-check within the project, so its request to build one
is met. What it adds here is an outside reimplementation and a red team run by
someone else, both already named above.

### 82. Keep the contribution guide runnable

Why: `CONTRIBUTING.md` (item 17) covers issues, pull requests, the file map,
checks, credit and AI assistance, but its worked example still names
`book-1/05-whether-it-arrived.pins.nibli` and `book-1/05-whether-it-arrived.md`,
which item 53 renumbered `04-`, and quotes a Chapter 4 sentence naming a
fixture that item 51 took out of the prose. The text scope of
`tools/relocate.py` lists `CLAUDE.md`, `README.md`, `AGENTS.md`, `LICENSING.md`
and `TODO.md` but not this file, and no check reads it. From the merged list's
item 12.
Do: rest the example on a sentence Chapter 4 now carries and on its current
pin file, and rerun its command; add `CONTRIBUTING.md` to the relocation
tool's text scope and to an existing link check, so the next move rewrites it
or fails; point to `tools/second_engine.py` and its report for anyone
reproducing the core elsewhere, which is what the list's "how to build a
second engine" can mean now that both exist; and link the guide from the
companion.
Close when: the example's focused run passes; the extended check fails on a
planted stale link and passes on the file; the companion links the guide; and
the affected development tests pass.

### 83. The companion's front door

Why: the companion's home opens on "Pick a person. Walk their forks." and
defines neither a fork, a joint nor a lens; its dossier shows its entries in
one grid; and its contents page lists the book's pages without the one-line
summaries that `reference.md`'s annotated contents already carry. From the
merged list's items 2, 14 and 15.
Do: a "Start here" panel of at most 200 words on the home, saying what the
book is, what a fork is (one person's record walked a move at a time, each
move run live), what a joint is (the constitution compared with one declared
change), what the four lenses are, and where to begin: Nell's fork *Be born
with nobody*. A guided first run of that fork says only what Chapter 1's pins
return: Nell is a person, she is owed each of the nine floor items, the
barriers that protect a child hold for her, and no delivery follows from her
one entry. The list's draft, which said she is owed "everything" and that
nothing else follows, is not used. A page explains forks and joints with that
fork and the measured joint *Delete one rule; the child is owed nothing*. The
dossier groups its entries by what they discuss (limits, costs, objections)
and keeps each entry's kind. The contents page becomes a book map, giving each
chapter the summary `reference.md` holds, read from that file rather than
restated. The home links each.
Close when: the companion's input and browser tests cover the panel, the
guided run, the explainer and the map, with a sabotage control that fails when
a guided step states a conclusion its pin does not return; the static exporter
renders the new routes; and the web build type-checks for wasm32 and the
desktop build inside `ui/shell.nix`. How a newcomer reads it is optional
evidence under the 2026-08-15 ruling, recorded with its provenance if
gathered, never a completion condition.

### 84. The query behind each conclusion

Why: each companion verdict shows its query and a short label such as "Food
provision · Marisol", but no sentence says what the query asks; and the
chapter pin files, where a reader checking a chapter reads its tests, comment
their queries unevenly. From the merged list's items 3 and 7.
Do: give each query in `ui/game.json` a one-sentence gloss saying what it
asks, never what it proves ("Asks whether the rules conclude that the State
owes Nell food"), shown beside the verdict, whose existing explanation keeps
FALSE as not derivable from the record rather than untrue. Give each query
block in `book-1/*.pins.nibli` a plain comment saying what it tests and why,
where one is missing; generated cases keep what their generators write. The
list's "Verify this" boxes of Nibli queries inside every derived chapter would
put formal notation in derived text, which stays jargon-free and already
points to its cases (item 63), so they need an author ruling if still wanted.
Close when: a companion input test requires a gloss for every query, with a
sabotage control; a comparison with `HEAD` shows every active pin statement
and expectation unchanged; the chapter pin files pass focused runs, then the
complete `./verify.sh`.

### 85. Checks and limits where a reader can find them

Why: the method lists every check and where it stops (item 62), and the
second engine's report and the adversarial audit sit in the repository, but
the companion shows none of them. From the merged list's items 4, 10 and 11.
Do: a companion page generated from
`book-1/source/measurements/second-engine-results.json` and its report,
showing what the translation keeps and drops, the cases replayed and each
answer's agreement with its pin, with the method's limit that a misreading
shared by both engines passes both. A page of current limits quotes, with
links, the audit's open findings and the claim each withholds, the declared
defects (none is active) and each chapter's "What this cannot settle" where it
has one. Repaired defects stay in the repository's records — git, the decision
records and `book-1/source/resolution-receipts.md` — which the page may link,
since reader-facing pages describe the current design. The red team and the
outside reimplementation the list asks for are item 81's.
Close when: both pages are generated from their sources; a companion input
test fails when a source changes and its page does not; and the static
exporter renders them.

### 86. The design in ten minutes

Why: the opening states the promise, the thesis and the choices in outline,
but a newcomer cannot see the whole design without reading about 98,000
words. From the merged list's item 6.
Do: a summary of at most 1,000 words whose every line comes from the
plain-language article that states it and links that article and the chapter
that argues it: the nine floor items owed on personhood alone, the routes into
personhood, the one thing a sentence takes, the four emergency powers, the
protected core as Chapter 22's closed list gives it, and Part V's five
commitments. The list's draft named only "the bans on torture and collective
expulsion", dropping the ban on sending anyone into persecution and the other
categorical refusals on force, so its wording is not used. Publish it in the
companion, linked from the home, and at the front of the summary edition
(`tools/build_book.py --summary`). The opening note keeps its length, since D7
allows at most five pages before Chapter 1, and the summary must not become a
second free-hand paraphrase of the constitution.
Close when: a companion input test ties every line to an article and a
chapter that exist, with a sabotage control; the summary edition rebuilds with
it; and a prose consistency review passes.

### 87. Reconsideration conditions and costs, collected

Why: each derived chapter's argument section says what would change its
author's mind and who bears the choice's cost, and nothing gathers them. From
the merged list's items 8 and 9.
Do: after item 74 fixes where each chapter keeps its cost accounting,
generate two indexes from the argument sections, as the works cited and the
index are generated: the conditions grouped by theme and the costs grouped by
who bears them, each entry quoted and linked to its section. Most sections say
"I would reconsider"; Chapter 8 says "I would reopen" and Chapter 24 "I would
move toward … on evidence", so extraction reads each section's closing
conditions rather than one phrase and fails when a derived chapter's section
yields none. Place them in the reference's back matter and the companion; the
opening note may point to them in one sentence. Quoting keeps them indexes
rather than new argument or a second paraphrase.
Close when: the generator's `--check` passes and fails on a chapter planted
without a condition; the book-builder tests pass; and prose lint `--check`
shows no regression.

## Licence

This new planning text is licensed under CC-BY-4.0. See the full
[licence text](book-1/LICENSE-CC-BY) and the repository's
[licensing policy](LICENSING.md).

## Submissions — after every other item

Author's note, 2026-09-25. Once every other item in this tracker is complete,
submit to these presses in parallel:

- [MIT Press — Direct to Open](https://direct.mit.edu/books/pages/direct-to-open)
- [punctum books](https://punctumbooks.com/)
- [UCL Press](https://uclpress.co.uk/)
- [University of Westminster Press](https://uwestminsterpress.co.uk/)
- [Polity](https://www.politybooks.com/)
- [Pluto Press](https://www.plutobooks.com/)
- [Verso](https://www.versobooks.com/en-gb)

Start from the publisher-neutral proposal in `submission/README.md`, updated to
the finished text, and adapt it to each press's current official submission
requirements. No press is contacted before then.

On 2026-09-25 the author chose all seven, knowing that four presses' own
policies stand in the way, and then decided not to approach UCL Press:
- Verso's opposition to generative AI;
- UCL Press's AI policy and its charge;
- punctum's window, open only from May to July;
- Westminster's CC BY-NC-ND book licence.

[`submission/presses.md`](submission/presses.md) records each press's
requirements with their sources. [`submission/drafts/`](submission/drafts/README.md)
holds a message ready for each of the other six; the messages to Verso,
punctum and Westminster raise the conflict first. The author sends each one after filling the gaps marked
`[AUTHOR: …]`. Nothing has been sent.
