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
  that tooling, and item 54 wrote Part I's sections and opening case. Until
  items 55–57 write the rest, the other Parts' chapters stay wholly derived. The existing appendix is a carried archive, not an extra
  channel for new reader-facing arguments.
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
- The [submission proposal](submission/README.md), and any proposal already
  sent to a publisher, describe the manuscript at `93fa5662`. Keep that commit
  identifiable; item 65 refreshes the submission when the revision is complete.
- External human review is optional, not a completion dependency (2026-08-15
  ruling); the plan's readers, experts, reproduction and red team are welcome
  under item 66. Never invent reader testing, expert endorsement, empirical
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
  amendable law bounded by the core. Item 58 defines the terms and argues the
  result.

## Ordered revision backlog

Work in this order: items 55–63 carry the rewrite under the ratified rulings;
item 64 finishes production, 65 closes the revision, and 66 is optional.
Numbering continues from items 01–33, which `CLAUDE.md` records; items 34–54
and 67–72 are complete and recorded there too. Chapter numbers refer to the
[current reading sequence](book-1/contents.json), which item 53 made the plan's
§4.2 table; the plan's own §14 punch list uses the earlier numbers, which
`tools/maps/2026-09-25-restructure.json` maps to these.

### Structure and rewrite

- [ ] **55. Rewrite Part II, and add "An Ordinary Week".**

  **Scope:** Part II, Chapters 7–15; plan §8.6 and §14; rulings D2, D3, D7 and
  D9.

  Rewrite each chapter in the shape Part I now has (item 54, recorded in
  `CLAUDE.md`). The derived account comes first: the situation as a pinned
  case, the rule and how it works, what it costs and who bears it, and at most
  one "What this cannot settle". One labelled argument section closes the
  chapter: the reason, the strongest alternative with its evidence, and what
  would change the choice. Documented cases appear only there and in the Part's
  opening case (D3), each registered and bound to its phrase; child sections
  follow D5; the line pointing to companion cases waits for item 63.

  Write the merged work and contribution chapter
  with five or fewer names, introducing and rejecting recognition once. Write
  the commons and animal chapters D7 creates, the animal chapter following
  ruling D9 and item 58. Add "An Ordinary Week": an adult who works, rents,
  visits a clinic, has a child in school, votes, disputes a landlord and is
  stopped by the police, showing which rules touch them, how lightly, and where
  the machinery stays out of sight. Its derived account has pinned cases, no
  timeline beyond them, no inner life and no composite biography. Write Part
  II's opening case, such as the Dutch childcare-benefits scandal or Robodebt.

  **Done when:** each chapter's derived sections pass the lint's thresholds with
  five or fewer case names and keep their pinned claims exact; each argument
  section states the reason, the strongest alternative and what would change
  the choice, with every source registered; and every claim in the new
  chapter's derived account traces to its pins.

- [ ] **56. Rewrite Part III.**

  **Scope:** Part III, Chapters 16–22; plan §14; rulings D2 and D3.

  Rewrite in item 55's shape. Keep the recall insight at the front of Chapter 16
  and move its counterfactual to the companion. Add a diagram of the bodies to
  Chapter 17. Halve Chapter 18's negation and move out its formal account of why
  an added rule does not repeal an existing one. Make Chapter 19 one of the
  strongest in the book. Move Chapter 22's source-editing tests and proposal
  names to the companion. The argument sections carry the precedents the plan
  lists, and Part III opens with a documented case such as *ADM Jabalpur*.

  **Done when:** item 55's conditions for chapters hold.

- [ ] **57. Rewrite Part IV.**

  **Scope:** Part IV, Chapters 23–28; plan §14; rulings D2, D3, D5 and D7.

  Rewrite in item 55's shape. Write the merged chapter on findings about people,
  with its reach stated first (item 41) and its cases cut to four. Keep the
  distinction that a disclosure against a private person opens no shield.
  Rebuild Chapter 25 on its principle, keeping its title and first line. Keep
  Chapter 26's severity table and cut its names to five. Keep Chapter 27's
  absolute prohibitions and add the cost of a lapse (item 39). Compare Chapter
  28 with national preventive mechanisms and human rights institutions in its
  argument section. Part IV opens with a documented case such as *Hussainara
  Khatoon*.

  **Done when:** item 55's conditions for chapters hold.

- [ ] **58. Argue the largest commitments in proportion to their stakes.**

  **Scope:** the animal food rule and core, the entrenched commons, the
  unamendable core, residence-based voting, the collective executive and the
  equal-weight chamber, and retained confinement; plan §8.2; rulings D2 and D9.

  Ruling D9 keeps the animal core as enacted. Define "direct protection",
  "avoidable" and "dispensable" on that basis, from the source: dispensable
  killing is killing solely for a purpose on its closed list, and the
  alternative-sensitive food rule is ordinary amendable law bounded by the core.
  Say plainly what that means for farming, household production and subsistence;
  argue against welfare regimes, political theories of animal rights and
  humane-farming positions; work the jallikattu case (*Animal Welfare Board of
  India v A. Nagaraja*, 2014, and the 2023 Constitution Bench decision); and
  spell out transition duties to the food floor, to workers and to communities.

  Give the commons their own argument, intertemporal freedom after *Neubauer*,
  and confront the power entrenchment gives interpreters: the basic-structure
  doctrine, Article 79(3) of the Basic Law, the substitution doctrine, Roznai
  and Landau. Engage Waldron by name and say what stops interpreters from
  expanding the core. Engage the all-affected and all-subjected debate on
  voting, Switzerland and Uruguay's colegiado on the executive, and abolitionist
  arguments, with the incarceration evidence, on confinement. Each argument goes
  in the argument section of the chapter that states the rule (D2), and its
  sources are the ones item 46 registered, read with their notes.

  **Done when:** each commitment has its strongest alternative stated fairly,
  its cost and who bears it, and the evidence that would change the choice.

- [ ] **59. Make Part V a synthesis, and rank where the design could fail.**

  **Scope:** Part V; plan §8.1 and §8.4; rulings D2, D3 and D7.

  With each chapter carrying its own argument (D2), rewrite Part V as synthesis:
  how the choices fit together, where they trade off, and the commitments
  beneath them. State the contributions: keeping entitlement, duty, evidence and
  delivery apart as a principle of design; constraints on how facts flow into
  consequences (item 48); testing by paired edge cases; and the specific
  mechanisms, namely the one thing taken, answerability that survives recall,
  severity as a ceiling rather than a selector, false-scarcity controls, silence
  never counting as approval, and the firewall between services and enforcement.
  Answer the objection that designing from the extreme case burdens ordinary
  life, using the ordinary-week chapter. Add "Where This Could Fail": at-scale
  non-performance, open standards read narrowly, captured review, slow and
  costly procedure, input integrity, and fiscal feasibility (Book 2's), each
  with the evidence that would show it failing, what the design does now and
  what a fix would take. The ranking is argued, not scored. Part V takes its
  opening case as D3 directs.

  **Done when:** no argument depends on a passage moved elsewhere, each ranked
  limitation has its falsifier, and the chapter replaces distributed
  disclaimers rather than adding to them.

- [ ] **60. Rewrite the opening.**

  **Scope:** the opening note and the front matter; plan §4.2, §8.5 and §11;
  rulings D1, D3 and D7.

  Keep what precedes Chapter 1 to five pages or fewer. Open with Santoshi
  Kumari's case, the dispute over her death intact (D3). Then give the question
  and thesis, Nell and the prisoner, the three distinctions stated once (record
  and world, entitlement and delivery, conclusion and event), the general limits
  (item 52), what the book claims and what it does not, its contributions, and
  how to read it. The first page carries ruling D1's promise, and the opening is
  written for the serious non-specialist. Make the statement of AI assistance
  specific: which parts were drafted, edited, coded or tested with it, and how
  the outputs were checked. Disclose that the constitution, pins and engine
  share a maintainer. The reference material is at the back (item 53).

  **Done when:** Chapter 1 begins within five pages, the opening reads without
  the glossary, and it promises neither operation nor proof.

### Appendices, companion and production

- [ ] **61. Publish the constitution in plain language in the companion.**

  **Scope:** plain-language articles in the companion (`ui/`), an article map to
  rule families, contracts and pins, a traceability check, and the chapters'
  article citations; plan §5; ruling D4.

  Write numbered plain-language articles, organised by the book's Parts, with
  the protected core marked. Map each article to the rule families, contracts
  and pins that implement it; map by family, because most rules sit in generated
  blocks with no article banner. Generate the articles' skeleton from that map
  and edit by hand, never with machine-rendered English. Add the interpretation
  article as ruled: its burden half (a restrictive conclusion needs complete
  positive evidence, and absence never extends a power) traces to the existing
  fail-closed rules and their pins, and its protective-reading half is a
  declared non-formal article, listed as a deliberate gap. Add a development
  check that every article has at least one family and one pin and every rule
  family belongs to an article. Publish the articles in the companion, and cite
  article numbers from the chapters. Compare each article with existing
  constitutions through the Constitute Project and record its lineage and
  novelty. A lawyer's conformance review of a sample is welcome under item 66,
  not required.

  **Done when:** no rule family lacks an article and no article lacks a test, or
  each gap is listed with its reason; the companion publishes the articles; and
  every article number cited in a chapter resolves.

- [ ] **62. Finish the method as the book's technical appendix.**

  **Scope:** `book-1/method.md`; plan §4.1 and §10; ruling D4.

  The method keeps its name and place as the book's technical appendix (D4).
  Bring together the flow constraints (item 48), the signature budgets (item
  42), the surviving mutants (item 49), the reproduction (item 50), coverage by
  article as lists (item 61), the adversarial audit's open findings, and the
  state of declared defects: none is active, and the method says so. Keep its
  sealed scope and the plan's length of about 3,000 words.

  **Done when:** every published assurance is a list with its route and
  limits, no aggregate score appears, and the quoted rules pass the quotation
  test.

- [ ] **63. Map every chapter to runnable companion cases.**

  **Scope:** `ui/case-map.json`, `ui/game.json` and each chapter's pointer;
  plan §10.6; ruling D2.

  Close each chapter's derived account with a line pointing to the companion
  cases it can run, as ruling D2's chapter shape sets, and move there what left
  the prose (counterfactuals, harness cases, carry mechanics, byte-level
  amendment checks) with its explanation. The companion executes records live
  and packages no expected answers; keep it so. The website picks up the change
  at its next deploy from `main`.

  **Done when:** every pointer opens a case that runs, the companion's tests
  pass, and nothing that left the prose is lost.

- [ ] **64. Production and accessibility.**

  **Scope:** `tools/build_book.py`, the opening's navigation text, the READMEs,
  the proposal and new diagrams; plan §13; ruling D1.

  Print ruling D1's subtitle, *A worked design for a society, with its formal
  claims made executable.*, on the cover and in the EPUB and PDF metadata, the
  READMEs and the proposal, and its promise on the back cover. Remove web
  artefacts from print: the closing "Continue to Chapter 1, or return to the
  contents." at the end of `book-1/reference.md`, which repeats the opening's
  "Begin with Chapter 1"; the "↩︎" returns the footnote renderer adds; and
  phrasing that fails on paper. Make cross-references numeric. Draw the duty chain, delivery
  evidence, the placement ceiling, the democratic corridor and the flow
  constraints as diagrams, each with its prose equivalent. Add a bibliography
  and an index. Publish a plain-language summary edition, since the constitution
  asks plain language of public bodies.

  **Done when:** rebuilt HTML, EPUB and PDF pass the renderer tests, EPUBCheck
  and inspection, with no web artefact in print.

### Completion

- [ ] **65. Read the revision fresh, verify it, and refresh the submission.**

  **Scope:** all ordered inputs, `reviews/`, `submission/README.md`, the
  reading copies and `CLAUDE.md`.

  Read every ordered input in sequence and record a fresh assessment, as items
  20 and 33 did, without protecting either review's diagnosis. Run the
  complete verifier and the development checks, rebuild and inspect every
  format, and remeasure with the lint. Refresh the proposal and sample to the
  revised text; a proposal already sent describes `93fa5662`, which stays
  identifiable. Record the revision's rulings and measurements in
  `CLAUDE.md`.

  **Done when:** the assessment, verification, measurements and submission
  all describe the same commit.

- [ ] **66. Optional: independent validation.**

  **Scope:** plan §8.3, §10.5 and §12. Never a completion condition
  (2026-08-15 ruling).

  Welcome evidence: a comparative constitutional lawyer, a scholar of
  economic and social rights, a formal-methods specialist, an animal-law or
  ethics scholar, frontline practitioners, and three lay readers with a
  confusion log and recall questions; an opponent test for each contested
  choice; an independent reproduction; a public red-team challenge ("write a
  rule that harms Nell or a prisoner and still passes the suite"); and a
  peer-reviewed paper on the flow constraints. Recruit early, because their
  calendars are the longest dependency. Anything published or sent needs the
  author's go-ahead. Record any such evidence under the existing provenance,
  custody and disposition rules for outside review and reader evidence; never
  invent a reader, endorsement or result, and do not count AI reviews as
  validation.

  **Done when:** the author closes it. It never blocks another item.

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
