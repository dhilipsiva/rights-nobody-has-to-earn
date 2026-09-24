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
- **Unverified:** the plan's citations, which it says were written from memory
  (item 46).

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
  each derived chapter and measures the length rule by section; until item 53
  builds that tooling, the opening, Part V and the method remain the only
  non-derived text. The existing appendix is a carried archive, not an extra
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

Work in this order. Items 34–52 measure, repair and prepare; items 53–63 carry
the structure and the rewrite under the ratified rulings; item 64 finishes
production, 65 closes the revision, and 66 is optional. Numbering continues from
items 01–33, which `CLAUDE.md` records. Chapter numbers refer to the [current
reading sequence](book-1/contents.json), which the plan also uses, until item 53
changes it.

### Resolve the design questions, source and pins first

- [ ] **40. Correct the scarcity account: the manager does not attest the
  comparison.**

  **Scope:** Chapter 6 and the
  [scarcity and conflict contract](book-1/appendix/contracts/scarcity-and-conflict-contract.md);
  plan §7.6.

  The plan's premise is not true of the source. Every allocation field,
  including the comparison outcome, must be attested identically by a source,
  an evidence and a review writer, each distinct from the manager
  (`src/authoring/scarcity.rs:79-84`), and anyone may open review of a
  finding, allocation or shortfall without the manager's permission (`:193`).
  Chapter 6's "The manager's stated decision… Its reasons…" invites the
  misreading. Check whether a claimant's counter-evidence can reach the
  comparison itself or only review, and repair if it cannot.

  **Done when:** Chapter 6 shows who attests the comparison and how a
  claimant contests it, the contract records the disproved premise, and any
  added route is pinned.

- [ ] **41. State the signing restriction's reach, and name it for what it
  does.**

  **Scope:** every chapter that says "void"; the
  [credibility decision](book-1/appendix/decisions/credibility-finding-decision.md)
  and the
  [recognition decision](book-1/appendix/decisions/recognition-purpose-decision.md);
  plan §6.2 and §7.6.

  A credibility finding now restricts permission to sign new adverse findings
  (item 19) and no longer reaches recognition, which the design removed
  (item 21). "Void" and "voiding", used more than fifty times, invite a far
  broader reading, and "recognition loss" still appears thirteen times,
  mostly in lists of what does not follow. Decide whether the restriction's
  purpose widens to every office that makes adverse findings, such as
  inspectors and investigators, or stays with credentialed examiners, and pin
  the answer. Rename the mechanism in prose (the plan suggests "signing
  restriction" or "restriction on adverse-finding authority"), keeping
  credibility distinct from standing and public answerability. Chapter 25
  takes a new title and keeps its file path. Remove the recognition residue
  except where the book explains why the design keeps no recognition status.

  **Done when:** pins show a restricted signer, an unrestricted one,
  restoration and an office outside the restriction; the reach is stated
  where the mechanism is introduced; and neither "void" nor "recognition
  loss" remains in prose in this sense.

- [ ] **42. Measure the procedural load.**

  **Scope:** every rule family and its generator; plan §7.5; ruling D6.

  Classify each consequential conclusion as beneficial (it gives, preserves
  or releases) or adverse (it restricts, takes, confines or finds against),
  and generate from the source the distinct roles each needs before it takes
  effect: the plan's "signature budget". Mark duplicated steps. Part V already
  says "More signatures have no value in themselves". The table goes in the
  method as a list, not a score. This item changes no rule; item 43 applies
  ruling D6 from its table.

  **Done when:** the generator and its table exist, the classification has
  been reviewed against the source, and each duplicated step is listed with
  its family.

- [ ] **43. Apply ruling D6: fast for help, slow for harm.**

  **Scope:** every family item 42 classifies, their contract cards, and
  `src/authoring/floor_vector_tests.rs`; ruling D6.

  Apply the ruling family by family, from item 42's table. An act that only
  gives or preserves something for its subject (immediate care continuity, an
  accommodation, interim protection) takes effect on one authorised actor, and
  prompt independent review can correct it. Any appointment giving someone power
  over another person, and every adverse act, keep full prior procedure;
  delivery evidence keeps its independent witness. Revise
  `every_power_record_is_independently_reviewed` into the ruled property with
  its sabotage controls, move the change into each affected contract card, and
  remove the duplicated steps item 42 found.

  **Done when:** pins show a beneficial act taking effect on one actor and being
  corrected on review, an appointment of power over another person and an
  adverse act each refused without full procedure, and the revised development
  check passing with its controls.

- [ ] **44. Split `secure` into bodily safety and material security.**

  **Scope:** Articles 1 and 1b, the delivery routes (`FS-CVF-015`), the coverage
  map, the glossary and every chapter that names the floor; ruling D8.

  Implement ruling D8. Bodily safety becomes a floor item guaranteed as
  protection, with no receipt route; material security keeps the current receipt
  route and its witness distinct from the source. Complete the coverage-map
  contract and the formal proof the kernel requires, give the new item its
  entitlement, debt and refusal pins, keep every existing refusal, and carry the
  two names through every chapter and the glossary. Care keeps `healthy`, and
  company keeps its route.

  **Done when:** all nine floor items have their entitlement, debt, refusals and
  delivery status pinned, bodily safety has no receipt route, and the prose uses
  the ruled names.

- [ ] **45. Pin the remaining stress tests.**

  **Scope:** plan §7.7. Items 37–39 carry the disclosure, non-delivery and
  lapse cases, and item 58 the cultural-practice case.

  Write pin sets for: two nominally separate appointment selectors controlled
  by one coalition; an emergency that makes a scheduled election physically
  impossible; scarcity with manipulated urgency evidence; an agency and its
  contractor blaming each other in plural provision; a rights-preserving
  amendment blocked by an expansive reading of the core; and a person never
  recorded, such as Ori, reached through outreach. Each states what the
  design does and where it stops. A confirmed defect gets its own item under
  the resolve-before-defending rule.

  **Done when:** each case runs in the suite, and each confirmed defect is
  repaired or has its own item.

### Evidence and lineage

- [ ] **46. Verify and register the plan's sources.**

  **Scope:** `registry/claims.json` and every source named in plan §5–§10.

  The plan says its citations were written from memory and must be checked.
  Check each against a primary source before use: for case law, the court, date,
  citation and the passage that decides the point the prose relies on; for books
  and articles, the edition and page. Register each with a locator a reader can
  follow. The registry's lessons apply: an identifier is not a source, and a
  court is not a judgment. Under rulings D2 and D3 these sources may support the
  opening, the Part openers, the argument sections and Part V, never a derived
  section. Record any source that fails, with the claim it would have supported.

  **Done when:** every source a later item uses has a registry entry with a
  working locator, `registry/check.py` and the claim-discipline tests pass,
  and failures are recorded rather than dropped.

- [ ] **47. Correct Part V's evidence and credit its lineage.**

  **Scope:** Part V and the opening; plan §9 and §9.1.

  The coercion argument opens with Tanzania's villagisation, which concerns
  forced relocation, not criminal confinement. Replace it with evidence on
  incarceration and its alternatives, such as the Nordic comparisons and the
  recidivism and employment research the plan lists. Tie the communes and
  cooperatives to the choices they bear on, valuation and rotation, and add
  republic-level evidence for the others. Credit the lineage the book builds
  on where the argument uses each idea: Shue on basic rights; the respect,
  protect and fulfil framework; Sen and Drèze on entitlements; Pettit on
  non-domination; Anderson; Rawls on the social minimum; Rahman; Raworth and
  Rockström; Ostrom; Nissenbaum; the 3Rs; the CRPD and the UNCRC; Kymlicka
  and Shachar; Crépeau and Hastie; the Quebec Reference. Each source passes
  item 46 first.

  **Done when:** each historical example supports a stated comparison, every
  credit is verified and registered, and the Part V figure bindings pass.

### The formal layer

- [ ] **48. Name the flow constraints and situate the engine.**

  **Scope:** the optional method, and Chapters 4 and 27, which demonstrate
  the stratification refusals; plan §10.1–§10.3 and §11.

  Six kinds of constraint already govern how facts flow into consequences:
  closed inputs (`admits`), conclusions that cannot be written directly
  (`derived_only`), purpose-bound reads (the `pay` and `promise` guards),
  endpoints nothing reads (the no-reader census), no punishment from absence
  (the floor firewall), and scope binding (case- and incident-bound
  findings). Name them, map each to the rules and development checks that
  enforce it, relate them to information-flow control, taint analysis and
  contextual integrity, and say that they check written rule forms and can
  miss a semantically equivalent attack. Move the stratification refusals out
  of chapter prose and describe them accurately: designed consequences of
  writing entitlements as events downstream of personhood, specific to how a
  rule is written. Say who builds and maintains Nibli, that the constitution,
  pins and engine share a maintainer, how it relates to Datalog with
  stratified negation and to earlier law-as-code work (Sergot et al., Catala,
  Rules as Code), and what is new. The method's sealed scope stands: no
  machine-rendered English, no proof traces, no compute backend, excerpts
  with pointers.

  **Done when:** every named constraint points to its enforcing check, the
  method's quoted rules pass the quotation test, and no chapter explains
  stratification.

- [ ] **49. Run mutation testing over the rules.**

  **Scope:** the constitution and the suites in `tests/pins/suites.json`;
  plan §10.4.

  Mutate rule conditions (drop a conjunct, flip a negation, swap a role) and
  record which mutants every pin still passes. A complete run takes about
  twenty minutes, so sample by family, run focused suites, and state the
  sampling. Each survivor is a finding: a missing pin, a deliberately dormant
  guard such as Article 4's signer checks, or a defect for its own item.
  Publish the survivors as a list with their dispositions. The assurance
  portfolio refuses aggregate scores and percentages, so there is no mutation
  score.

  **Done when:** the harness runs as a development tool, its sampling is
  documented, and every survivor has a disposition.

- [ ] **50. Reproduce the core in a second engine.**

  **Scope:** standing, the floor, delivery, custody and the shield; plan
  §10.5.

  Reimplement that subset in a mainstream engine such as Soufflé or clingo,
  compare verdicts on shared fixtures, and publish every difference with its
  cause. A reimplementation made inside this project is a cross-check, not an
  independent reproduction; say so, and welcome an outside one under item 66.

  **Done when:** the subset runs in the second engine, each difference is
  fixed or explained, and the method reports the result with its limits.

### Prose that does not wait for the structure

- [ ] **51. Settle the vocabulary and the cast, and apply them.**

  **Scope:** all ordered inputs, the cast and its pins, and the lint's lists;
  plan §6.2 and §6.4.

  Apply the plan's replacements in reader prose. "Derive" becomes "follows"
  or "the rules conclude"; "supplied" becomes "recorded" or "given"; a
  "qualified" or "effective" finding becomes one "properly made" or "in
  force"; "reader", for an institution, becomes "the responding office"; "pen"
  and "credential" become "authority to sign findings"; "lease" becomes
  "custody authorisation and its review date"; "window" becomes "period"; and
  "selected current record" becomes "the version in force". Carry mechanics
  move to the method. Harness names (Targ4, Nogra, Nogrb, Partnr, Sock, the
  Amend_ proposals and the rest) and institutional fixtures become
  descriptions in prose, while the pins keep them, as item 27 did. Names that
  prejudge the case, Sly, Boss and Rebel, and Vex, Hex, Rex, Lupo and Don
  after checking, change in the cast and pins as well, because neutrality is
  a reason of substance: take a census first and move every locator. Choose a
  stable recurring cast of fifteen or fewer whose facts do not change between
  chapters, without assembling separate cases into one invented life. When a
  case changes the facts, write "Suppose instead…" or use another person.

  **Done when:** the lint's lists match the chosen vocabulary and cast,
  renamed identifiers leave the suites, coverage, receipts and reference
  tests passing, and no qualification is lost.

- [ ] **52. State the general limits once, and keep only each chapter's own.**

  **Scope:** the opening and every chapter; plan §6.1 and §6.5.

  Write one opening page stating the general limits: the rules read records,
  not the world; a conclusion does not prove its inputs; an entitlement is not
  a delivery; and operation belongs to Book 2. Then remove restatements of
  them from the chapters, keeping each chapter's own limit (a false witness,
  an absent record, a stopped clock, an unperformed remedy) beside the claim
  it qualifies, as item 29 required. Replace the limitation headings with at
  most one short passage per chapter on what it cannot settle, and update the
  coverage entries they anchor. Lead with what is true, then what is not:
  "Nell is owed food; nobody has yet shown that food arrived." Where a
  chapter's limit is a liveness limit, point once across the seam to the
  second book. Reduce the child sections as ruling D5 directs, recording each
  removal in `CHILD_SLOT_EXEMPT` with its reason.

  **Done when:** each chapter's disclaimer and negation figures fall toward
  the lint's thresholds, every claim keeps the qualification it needs, and no
  sentence implies an arrival.

### Structure and rewrite

- [ ] **53. Restructure the manuscript and build the tooling the rulings need.**

  **Scope:** `book-1/contents.json`, `src/authoring/contents.rs`,
  `tools/relocate.py`, `tools/build_book.py`, the development tests, file names,
  pin pairs, suites, coverage, receipts and every locator; plan §4.2; rulings
  D2, D3, D5 and D7.

  Apply ruling D7's table. `tools/relocate.py` performs renames and renumbering
  only, so extend it to merges and splits, or script them under the same checks:
  nothing lost, every non-comment pins line accounted for, and no stale
  reference left. Merge chapter 25's pins so that its closing unscoped `:accept`
  comes last. Extend the manifest for a labelled case opening each Part (D3) and
  for Part V's two chapters, since `part_v()` expects exactly one. Move the
  opening's reference material to the back matter and update
  `opening_note_navigation_matches_the_manifest`, and update `SAMPLE_CHAPTERS`
  and the sample's cover.

  Mark argument sections (D2) so that the digit rule, the figure bindings,
  reader coverage and the length measure treat derived and argued text
  separately, with the majority-derived rule measured by section. Every derived
  chapter keeps its paired pins and live case, and engines still precede breaks.

  **Done when:** the manifest, directory and assembled copies agree, every
  reference resolves, the development tests pass with their section-aware
  checks, and the complete verifier passes.

- [ ] **54. Rewrite Part I.**

  **Scope:** current Chapters 1–7; plan §4.3, §8.6 and §14; rulings D2, D3 and
  D5.

  Rewrite each chapter in the shape ruling D2 sets. The derived account comes
  first: the situation as a pinned case, the rule and how it works, what it
  costs and who bears it, what the chapter cannot settle, and a line pointing to
  its companion cases. One labelled argument section closes the chapter: the
  reason, the strongest alternative with its evidence, and what would change the
  choice. Documented cases and precedents appear only there and in the Part's
  opening case (D3), and child sections follow D5.

  Take the P1 chapters first, following the plan's punch list. In the child
  chapter, keep the four ways a person enters the record and move the birth-rule
  counterfactual to the companion. Cut Chapter 3's cases to four and move out
  its vocabulary-change and carry mechanics. Make Chapter 5 the showcase for
  recipient-side evidence, keep its claims table and move its custody material
  to Part IV. Bring the allocation literature and the triage cases to Chapter
  6's argument, and the tiered bearer and systemic remedy to Chapter 7. Add the
  walkthrough of a claim that fails: one claim from first contact through
  assistance, provision, evidence and dispute to failed responses from both
  reviewer and alternate, and on to remedy, with no timings or staffing, which
  belong to Book 2. Write Part I's opening case, distinct from the book's.

  **Done when:** each chapter's derived sections pass the lint's thresholds with
  five or fewer case names and keep their pinned claims exact, and each argument
  section states the reason, the strongest alternative and what would change the
  choice, with every source registered.

- [ ] **55. Rewrite Part II, and add "An Ordinary Week".**

  **Scope:** current Chapters 8–15; plan §8.6 and §14; rulings D2, D3, D7 and
  D9.

  Rewrite in item 54's shape. Write the merged work and contribution chapter
  with five or fewer names, introducing and rejecting recognition once. Write
  the commons and animal chapters D7 creates, the animal chapter following
  ruling D9 and item 58. Add "An Ordinary Week": an adult who works, rents,
  visits a clinic, has a child in school, votes, disputes a landlord and is
  stopped by the police, showing which rules touch them, how lightly, and where
  the machinery stays out of sight. Its derived account has pinned cases, no
  timeline beyond them, no inner life and no composite biography. Write Part
  II's opening case, such as the Dutch childcare-benefits scandal or Robodebt.

  **Done when:** the conditions of item 54 hold, and every claim in the new
  chapter's derived account traces to its pins.

- [ ] **56. Rewrite Part III.**

  **Scope:** current Chapters 16–22; plan §14; rulings D2 and D3.

  Rewrite in item 54's shape. Keep the recall insight at the front of Chapter 16
  and move its counterfactual to the companion. Add a diagram of the bodies to
  Chapter 17. Halve Chapter 18's negation and move out its formal account of why
  an added rule does not repeal an existing one. Make Chapter 19 one of the
  strongest in the book. Move Chapter 22's source-editing tests and proposal
  names to the companion. The argument sections carry the precedents the plan
  lists, and Part III opens with a documented case such as *ADM Jabalpur*.

  **Done when:** the conditions of item 54 hold.

- [ ] **57. Rewrite Part IV.**

  **Scope:** current Chapters 23–30; plan §14; rulings D2, D3, D5 and D7.

  Rewrite in item 54's shape. Write the merged chapter on findings about people,
  with its reach stated first (item 41) and its cases cut to four. Keep the
  distinction that a disclosure against a private person opens no shield.
  Rebuild Chapter 27 on its principle, keeping its title and first line. Keep
  Chapter 28's severity table and cut its names to five. Keep Chapter 29's
  absolute prohibitions and add the cost of a lapse (item 39). Compare Chapter
  30 with national preventive mechanisms and human rights institutions in its
  argument section. Part IV opens with a documented case such as *Hussainara
  Khatoon*.

  **Done when:** the conditions of item 54 hold.

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
  sources pass item 46 first.

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
  share a maintainer. The reference material moves to the back under item 53.

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
  contents." (`book-1/00-opening-note.md:447`), which repeats "Begin with
  Chapter 1" (`:84`); the "↩︎" returns the footnote renderer adds; and phrasing
  that fails on paper, such as "the choices below lead directly to them"
  (`:86-87`). Make cross-references numeric. Draw the duty chain, delivery
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
