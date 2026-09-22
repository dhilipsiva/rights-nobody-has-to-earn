<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Chapter-by-chapter review — 21 September 2026

A fresh sequential read of all 34 ordered inputs at `eb4e4b05`: the epigraph,
the opening, chapters 1–31 and the optional method, in manifest order. I read
the same-day [final manuscript review](2026-09-21-final-manuscript-review.md)
only after forming these ratings; where they differ I say why. Beside the
reading, about thirty chapter claims were checked against the chapter pin
files, `tests/pins/suites.json`, the counterfactual inventory and the
constitution. This is an AI-assisted editorial judgment inside the project:
not a measurement, an independent endorsement, a reader study or evidence
that any institution operates. Ratings are out of 10 and concern each input's
work in this book — argument, precision and readability together.

## Overall: 8.5/10 as a manuscript

The proposal is distinctive and its discipline is real. The book separates
entitlement, duty, supplied finding, derived consequence and evidence of
receipt more consistently than any comparable text I know, and the separation
survives the hard cases: the child with one entry, scarcity, the discredited
discloser, the confined voter, the failed reviewer. Nearly every consequential
sentence I tested corresponds to an executed case (see *Verified as
described* below). The strongest chapters — 1, 5, 6, 21, 26, 28 — teach one
distinction through one person and stop.

What keeps it at 8.5 rather than 9 is editorial, plus two small precision
defects:

1. **Six chapters are catalogues rather than cases.** Chapters 3, 7, 11, 13,
   17 and 29 each contain long stretches that list what the rules require
   (nine duty kinds in a sentence, a thirteen-item public-result checklist,
   a statute-length animal-use standard) with no person to carry them. They
   are accurate projections of the source, but a reader retains a case and
   forgets a list.
2. **Part V is long and its last movement sprawls.** At 9,637 words it is
   about a sixth of the manuscript; the State section alone carries seven
   sub-arguments (territory, collective executive, president, residence,
   essentials, emergency, amendment), each sound, together fatiguing.
3. **Two experiments are described that do not execute as described**
   (chapter 27 with the method; chapter 16). Neither is a design defect;
   both are one-sentence fixes. Under this book's own standard — a described
   test must exist — they matter.

The same-day review rated the manuscript 9/10. The difference is weighting,
not disagreement about the facts: I count the catalogue passages more heavily
against readability, and I found the two description defects below that the
earlier read did not.

## Description defects found

### 1. Chapter 27 and the method describe a "plain label" experiment that no case runs — material, trivial to fix

Chapter 27 says: *"Another experiment replaces its protected condition with a
plain label. The tested entitlement then disappears and the hostile rule is
accepted."* The method repeats it: *"Another replaces the event-shaped
entitlement with a plain label; the tested entitlement disappears and the
hostile rule can load."*

No executed case replaces the entitlement with a label. The suite contains
exactly three edits to `entitled(every person, event { eats() })`:

- `amendments/AS-09` (and `AS-03`) **delete the line**; the entitlement
  disappears and `person($x) & ~eats($x) -> prisoner($x)` then loads
  (`prisoner(Cira)` TRUE). This is the result the two sentences describe.
- `amendments/AS-05` changes the grammar to `event { eats($x) }`; the
  entitlement disappears but the hostile rule is **still refused**. Chapter
  22 describes this case correctly as a grammar change that leaves the
  structural safeguard intact.

I also searched every pin file for an in-file `entitled(every person, <label>)`
assertion and found none. The described outcome is right; the described edit
is wrong. Fix: in both places say the experiment *deletes the entitlement
line* (or names AS-09), and keep AS-05 as chapter 22 has it. No rule or
expectation changes.

### 2. Chapter 16 states a State-entry counterfactual that runs only for the Court — minor

*"Removing State's public-body entry removes State's answerability without
removing its floor debts. The actual constitution supplies those entries; the
counterfactuals show why preserving them matters."* The executed
counterfactuals are `counterfactual/no-public-court` (removes `public(Court).`)
and `counterfactual/no-choose-boss` (removes `choose(Electorate, Boss).`).
Nothing removes `public(State).` The claim is true by inspection of the rules
(`authority` derives from `public`, `owe` reads only `person`), but it is
written as though a case shows it. Fix: say "Removing the Court's public-body
entry…" or add the State variant beside `no-public-court`.

### 3. Chapter 4's "changes what one floor debt promises" experiment — could not locate as described

*"A different experiment changes what one floor debt promises. The changed
rule still owes something to a person, but it no longer owes the selected
item."* I found no case or development test that rewrites an `owe(State, …)`
rule. If the sentence means `AS-05` — which changes the *entitlement's*
grammar, not a debt — it should say entitlement. Low confidence that this is
an error rather than my search missing a fixture; worth a look.

### 4. Cosmetic

`03-what-counts-as-evidence.md:140` carries a coverage-owner HTML comment in
the middle of a section. It is invisible when rendered, but the paragraph that
follows it reads as orphaned in the source and should probably be joined to
the one above.

## Verified as described

Each of these chapter statements was matched to an executed pin, fixture or
declared source edit at `eb4e4b05`. This is a sample, not a census.

- Ch. 2: a first-contact entry naming the Court derives no personhood
  (`person(Court)` FALSE after `at(Court, FirstContact)`).
- Ch. 3: Probe's attack on one person and cruelty against another, then
  cruelty against the same person, derive no severity.
- Ch. 4: the belief-absence credibility rule is now **refused** (the pins
  changed with the credibility rework; the prose follows them), the
  company-absence recognition-loss rule loads scoped, and positive compulsion
  loads scoped.
- Ch. 5 / method: Marisol's food derives only with the authorised distinct
  witness; `person(Marisol)` stays FALSE; shelter does not follow.
- Ch. 6: the budget-choice control (`scarcity/priority/false-scarcity-stops-the-allocation`)
  and the procurement-refusal case (`scarcity/worked/`) are two different
  executed cases, so the chapter's two descriptions are consistent.
- Ch. 9: Ansel's certificate derives with `person(Ansel)` FALSE; Brix's
  self-certification does not.
- Ch. 10: Yano's bare judgment-plus-examination earns nothing; adding the
  ground `Hunch` still earns nothing.
- Ch. 11: `CrossRegionalReachUnderRegionalTier` is refused by the tier
  vocabulary.
- Ch. 12: Quillon's `RetaliationForm` is in the vocabulary; the invented form
  is not.
- Ch. 15: the newcomer's inclusion in the scarcity population is a duty on
  State, with the exclusion and status-priority barriers.
- Ch. 16: `authority(Boss)` TRUE beside `broken(Boss)` TRUE; Rebel's shield
  holds; `no-choose-boss` removes it.
- Ch. 18: the ballot-for-non-prisoners rule loads and Hano keeps the ballot;
  the forged `person & decide` assertion is rejected whole.
- Ch. 19: Hano's food debt and ballot survive the record-power sequence.
- Ch. 21 / method: the bare request obliges the independent reader; the
  automatic self-bar rule is refused; the hearing-duty rule loads.
- Ch. 22: `Amend_Decoy` with a harmless target obtains no personal finding.
- Ch. 24: the Sly section's conjunct is the court's exclusion from deciding
  its own prosecution; the Rex sequence is as the shield decision records.
- Ch. 25: Targo's explanation duty derives without a personhood entry;
  Purga's restoration ends the finding while the case stays recorded.
- Ch. 26: `person(Fin)` FALSE and `lose(Points, Fin)` FALSE.
- Ch. 27: `no-person-line` drops the prisoner-to-person rule, supplies Zed's
  full qualified case, and the belief-absence confinement rule then loads.
- Ch. 28: the five-row severity table matches `custody/severity-boundary`
  (grave-only, cruel-only, two-victim, missing-insufficiency and
  supported-residence branches).
- Ch. 29: `year(Term_Ruk, Two)` loads after `admits("year")` and
  `prisoner(Ruk)` stays TRUE; `year(Term, Two)` without the admission is
  refused.
- Ch. 30: the `obliged(Review, $x) & ~capture(Review, $x) -> err($x, Duty)`
  extension loads and fires for Ruk.

## Ratings by input

| Input | Rating | Review |
|---|---:|---|
| Epigraph | 8.5 | Short, plain and relevant: a life spent finding the next meal is the condition the floor abolishes, and the closing question does not overreach. Two lines of the rendering are flat ("telling so many petty tales"), which is the price of plainness. |
| Opening note | 8 | The first 600 words are the best pitch in the book: the question, the proposal, the choices with their costs, and a link out. The remaining 3,300 words of map, glossary, roles, cases, subject index and diagrams are good reference material that a first-time reader meets before any case; the glossary entries for the shield and the pen are precise and dense. |
| 1. The Child With Nobody | 9 | One entry, everything owed, nothing delivered — the thesis in a page, with absences in a record kept apart from facts about a child. The Ori and deleted-rule section is the abstract part, and "internal breach markers" arrives eight chapters before markers are explained. |
| 2. Who Counts | 8.5 | Four routes, one floor, and a handle at an encounter that merges nobody's history. Short and clean; "Standing without a roster" restates chapter 1, and "executable ballot route" is engine talk. |
| 3. What Counts as Evidence | 7.5 | The closed vocabulary, reserved conclusions and the Probe severity example are exactly right. The rest is a tour — protected register, amendment, the shield, the broken court, kinship, adulthood — previewing chapters 22–25 at a detail the reader cannot yet use; "False or missing inputs" is a grab-bag. Cut each preview to a sentence and a link. |
| 4. What You Are Owed | 8.5 | Shelter is defined rather than named, the firewall's edge is stated with the chapter's own checks as evidence, and "Owed by whom" separates debt from receipt cleanly. One sentence lists nine duty kinds; see finding 3 on the debt-rewrite experiment. |
| 5. Whether It Arrived | 9.5 | Marisol's staged evidence and the five-row claim table are the book's clearest teaching device. The custody section is a detour, but it earns its place by showing a sentence supplies no receipt. |
| 6. When There Is Genuinely Not Enough | 9 | An indivisible unit, two protected claims, a table of grounds, a reviewed decision, a surviving shortfall and the false-scarcity control: the chapter shows judgment being bounded rather than asserting it. The nonresponse passage is dense but exact. |
| 7. Who Owes, and What Follows | 7.5 | The governing question is answered in the first section — the chain from shortfall to review to withdrawal — and then the chapter becomes an index: delegation, voluntary provision, duties-not-price, economic wrongs, justice review, protective power, environmental and animal findings. Two sections pre-empt chapters 29 and 13. Keep the chain and the five forms of responsibility; link the rest forward. |
| 8. What Nobody Has to Ask Permission For | 9 | Residual freedom written as a positive record with duties on the public body, restriction needing its own harm ground, enforcement bound to the exact restriction. Concise; the barrier list is short enough to read. |
| 9. Earning Above the Floor | 9 | Ansel, Coll and Marlo each get an issuer, an attester and a negative control, so the reader learns the shape once and sees it three times. "When the money runs out" slides back into listing. |
| 10. Contribution | 9 | Says why recognition exists and what it does not buy, on Bela and Cira, in 800 words. The Yano/Hunch and Dev examples do real work. |
| 11. What Money Cannot Buy | 8 | A compact survey of housing, contract, enterprise, private power, expiry, public money, cash and tiers, each paragraph correct and none developed. Harrow, the one named case, is a vocabulary probe — a permitted ground versus an invented threshold — so the chapter's only example does little. Give Harrow a finding-and-remedy sequence or accept that this is the survey chapter. |
| 12. The Same Route for Everyone | 8.5 | Clear on why equal access can require unequal resources, with the accommodation record, the positive-measure end and the defect sequence. "The design must meet its own test" is the right move. Lists again in the forms and duties sections. |
| 13. A Place in Which Life Remains Possible | 7.5 | The water-and-river case is among the best in the book, and the two-office conflict and the defective-record ending are strong. Between them sits a statute: presumptive sentience, the welfare baseline, food, research, prohibited harm, lethal control — 2,800 words, the longest derived chapter. Split the animal material into its own chapter or move the standards into a table. |
| 14. Holding a Role in Somebody's Life | 9 | Care is not authority, support is not substitution, and the reviewed appointment is a real answer to the child who cannot ask. "Adulthood and missing evidence" belongs in chapter 18, which repeats it. |
| 15. Arriving and Belonging | 8.5 | Three sequences with honest stops: arrival without papers, the conflicting-version determination that completes nothing, the consent record that withdraws a permission. The belonging section is a rights list. The cost of the version conflict — the claimant waits — is stated plainly, which is rarer than it should be. |
| 16. Public Answerability, and Why It Is Never Revoked | 8.5 | Boss and Rebel make the distinction between ending office and ending answerability immediate, and "What it costs" is candid about the growing exposure set and the forged seating. See finding 2. |
| 17. How Public Power Is Built | 8 | The ordinary-bill stages, the institution table, caretaker authority and "Leaving" are concrete and good. The thirteen-item public-result sentence and the four integrity subsections (money, districts, opposition, coordination) are rules without a case; trim them to what the chapter's cases use. |
| 18. The Vote Conviction Does Not Take | 9 | Hano and Bela on one boundary from two sides, the political home that custody cannot move, the forged entitlement rejected whole, and an ending — "an entitlement to food is not a meal" — that lands. |
| 19. What May Be Kept About You | 8.5 | Holding, watching and automating as separate permissions on the same record, then the defect that withdraws the permissions and leaves the duties. "Counting without ranking people" is clear. Some sentences enumerate eight things. |
| 20. A Crisis Does Not Suspend the Republic | 8.5 | Requisition and force abroad each get an authority, a withdrawal and the duties that survive it; the accounting duty falling with the measure is printed rather than smoothed. Treaties and exit are listed. The closing heading "Withdrawal and redress" does not describe the section under it. |
| 21. A Way to Be Heard | 9.5 | Nia and Ruk separate access, request, duty, suspension and relief with no wasted words, and the refused automatic-bar rule shows why the encoding matters. The best of the institutional chapters. |
| 22. Changing the Rules | 8.5 | The three-stage table (certified, published, effective) and the proposal-name section make exact-candidate authority intelligible without the host lesson; "What entrenchment requires" is an honest ending. Slightly abstract for lack of a person; Jala is a name on a docket. |
| 23. Who Holds the Pen | 8.5 | Gia, Wren, Boss and Vex isolate seating, recall and carried history; the shortcut counterfactual and the relief section each add a distinct point. Dense but every paragraph carries its own case; "What this rests on" drifts back to listing. |
| 24. The Shield | 9 | Don, Sly, Kel and Rex are four distinct conditions, and the qualification sequence — agreement alone, then authority, then fresh disclosures, then an actual conflict — is demanding but each step is tested. Costs stated. |
| 25. Voiding | 8.5 | Every condition of an adverse finding is isolated by its own case, which is the chapter's strength and its load: Bela, Targ4, Koa, Ambi, Dev, Lupo and Mira, Tyr, Vex and Wren, Partnr and Frisk, Purga, Targo. A small table of case → condition isolated would halve the effort. |
| 26. The Limits of a Finding | 9 | Short and exact: what the finding reaches, what survives, what ends it, and the real cost to the person awaiting a review nobody has performed. |
| 27. A Prisoner Is a Person | 8.5 | The central commitment tested at its least convenient point, with the equivalent-evidence child comparison correcting the easy reading. Rated down for finding 1: one of its three experiments is misdescribed. |
| 28. Where People Are Put | 9 | The five-row severity table is the best single device in Part IV, and the alarm, competing-record and no-home sections keep permission, report, fault and remedy apart. |
| 29. The One Thing Taken | 8 | The first half — one legal loss, current authority, intake, leaving — is strong. The second half (protective powers, force, intelligence, objection) is the public-safety contract projected as a catalogue; accurate, but a different chapter's subject and no case carries it. Move it or give it Hano. |
| 30. When the System Notices It Broke | 8.5 | Marker to duty to alternate to the failure of the alternate, with the honest terminus: a duty still owed and nobody who acted. "When the responsible office fails" is the heart. The record-and-time paragraph is a list. |
| 31. The Five Joints | 8.5 | Each joint gives the strongest objection, the mechanism, a credible alternative, who bears the cost and what would reopen it; the historical cases are bounded to what their sources support. At 9,637 words it is a sixth of the book, and the State section reads as seven briefs in a row. The kitchen paragraph and "What the argument asks" hold. |
| Optional method | 9 | Rules quoted as written, verdicts as pinned, the `permits(Appeals, ·)` lesson, the outcome table and the running instructions. Honest about what a pass includes. See finding 1 for the one misdescribed experiment. |

## Where to spend the next editorial pass, in order

1. Fix findings 1 and 2 (four sentences across chapters 16 and 27 and the
   method), and check finding 3.
2. Chapters 7 and 13: cut the previews from 7; split or tabulate the animal
   standards in 13. These two moves would raise the book's readability more
   than anything else.
3. Chapter 3: reduce the amendment, shield and broken-court previews to a
   sentence each.
4. Chapters 17 and 29: move the integrity subsections and the protective-power
   catalogue to where a case can carry them, or give them one.
5. Part V's State section: group the seven sub-arguments under two or three
   headings and cut the restatements of limits that the derived chapters
   already carry.
6. Chapter 25: a case table.

## What this review did not do

No complete verifier run; the checks above are focused reads of pin files
and the suite inventory, not executions. No rendering or accessibility
inspection. No fresh retrieval of Part V's external sources; item 31's
evidence review stands as the record for those. The ratings are editorial
judgments and a lower rating identifies no failed pin or established
contradiction.
