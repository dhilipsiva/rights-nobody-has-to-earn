# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Its sections are ordered: what the repository is, how work lands, the
author-ratified rulings, the files, and the long-settled design decisions.
The rulings section is the largest and is grouped by kind, not by date; each
ruling names its own controlling record under `book-1/appendix/` (the
planning record, moved there from the former `new-book-plans/` on 2026-09-17,
with the `book-1-` prefix dropped from each name), which is
authoritative where this summary and that record ever diverge.

## What This Repository Is

A constitutional design expressed in Nibli, with Book 1 as its reader-facing
projection. The formal source is `book-1/source/constitution.nibli`; its
substantive behavior is tested by Nibli pins and counterfactuals. Except for
the labelled opening note, Part V's argument and evidence, and the optional
method part, Book 1's claims derive from that constitution. The reader chapters
remain jargon-free. Generated reports and prose do not override the formal source.

**Ratified 2026-09-24, not yet implemented:** each derived chapter will also
close with one labelled argument section, and documented cases will also appear
in the opening and at the head of each Part (D2 and D3 of *The revision rulings
D1–D9* below). Until those items land, the paragraph above describes the
edition.

Book 2 owns operation and transition within a declared reference envelope:
staffing, costs, resources, technology, workflows, capacity, and empirical
feasibility. Its tracker is collection-only until Book 1 ships at Gate C.
Preserve `book.md` and `manifesto.md` until their legacy harvest is complete.

## Current design throughout — author decision, 2026-09-18

The reader-facing book always describes the current design of the edition being
read. This applies to every chapter, the opening note, Part V and the optional
method. Do not narrate how the book or constitution developed: no earlier-version
comparisons, accounts of repairs or additions, or before/after design histories,
even when accurate or instructive. Explain the current rule, its rationale,
consequences and remaining limits directly. This instruction supersedes earlier
permission to retain development history for its explanatory value and governs
future revisions as well as the current edit.

Git and repository decision records retain the development history outside the
reading sequence. Historical evidence about the world, explicitly tested
alternatives, and temporal behavior within the current model remain admissible;
they must not become accounts of superseded designs. Do not conceal a present
defect or present a historical result as a current test. Preserve immutable
published editions: "current" means the design bound to that edition.

## Resolve design failures before defending them — author decision, 2026-09-18

When a failure can be corrected in the design, correct it. Confirm the failure,
try substantive repairs and alternative representations, and consider narrowing,
replacing or removing the faulty mechanism. Reconsider nonessential inherited
choices under the standing delegated approval while preserving constitutional
commitments and recording substantive supersessions. Implement and verify a
sound resolution before revising the reader-facing account of the current design.

An adequately defended limitation is a fallback only when resolution is shown
to be impossible within named, justified constraints. Record alternatives,
results and the necessary constraint in the existing repository decision record.
A failed encoding, tooling limit, implementation cost or unsuccessful search is
not proof of impossibility; uncertainty or blocked execution leaves the issue
open. Bound any impossibility claim to what the evidence actually establishes.

A fallback defense must establish why the constraint is necessary, why the
chosen arrangement is preferable to available alternatives, who bears its cost,
the safeguards and remedies, and what would require reconsideration. Disclosure,
an asserted trade-off or a passing known-defect pin is insufficient. If neither
repair nor adequate defense is established, keep the work open and pursue
redesign, removal or narrowing; do not relabel a constitutional defect as Book 2
operation. This supersedes earlier permission to close work merely by exposing
or acknowledging a design failure. The book states the resulting present design
and justified limits; attempted repairs and development history stay in the repo.

## Verification and authoring — author decision, 2026-09-12

The author clarified that verification means **Nibli pins and contradiction
checking**, with prose consistency reviewed separately. This decision replaces
the receipt-bound verification and commit protocols, including all mandatory
audit/closure successors, fingerprint chains, staged-candidate manifests,
four-hour authoring-slice records, and generated-report freshness gates.

`./verify.sh` builds the native runner incrementally and executes all
substantive tests. `--list` shows the inventory; `--only <pin-file>` is focused
feedback and is labelled partial. The former quick/full distinction, receipts,
commit gates, and fingerprint modes are retired. No old gate must approve its
own removal, and no small change requires administrative audit commits.

Keep chapter/floor/domain pins, generated record/temporal/amendment/placement
cases, state-form and obligations cases, and substantive counterfactuals.
Preserve exact expected verdicts and stateful assertion order. Isolate unrelated
scenarios; never make the test faster by testing a reduced constitution or
discarding expectations. Counterfactuals apply explicit edits to the current
source instead of maintaining whole constitution copies.

The runner reports contradiction findings and incomplete checks separately.
A clean result describes the loaded formal model and represented constraints,
not a universal proof about prose or real-world feasibility. Ordinary
constitutional `false` and `contradict` predicates are not scanner findings.
Prose changes receive a separate consistency review.

`RIGHTS_VERIFY_JOBS=1..4` controls the fixed worker pool. Reuse compiled bases
and immutable rule plans within each process; do not persist verdicts, hash
input trees, or skip tests based on earlier results. The target is a complete
run under five minutes with the release binary already built. Report actual
measurements and any remaining bottleneck honestly.

Useful authoring generators remain behind `./generate.sh adversarial-audit`,
`./generate.sh state-form`,
`./generate.sh obligations`, `./generate.sh integrity`,
`./generate.sh statistics`, `./generate.sh amendment`, `./generate.sh mobility`,
`./generate.sh justice`, `./generate.sh knowledge`,
`./generate.sh procedural-load`, `./generate.sh reader-coverage`,
`./generate.sh record-power`,
`./generate.sh resolution-receipts`, `./generate.sh scarcity`,
`./generate.sh public-safety`, `./generate.sh ecology`, and
`./generate.sh spine`. They write semantic
rules and tests explicitly; normal verification reads the resulting Nibli.
Pending JSON authoring edits are not enacted automatically by verification.

Make coherent edits and ordinary commits after relevant checks. Tracker items
can close when the work is complete; there is no receipt, audit, or closure
successor prerequisite. Preserve unrelated work already in the tree.

The rulings below retain constitutional design history and substantive
decisions. Descriptions of historical audit scripts, hashes, receipts, generated
assurance reports, and their former gates are historical only and do not
reinstate the retired tooling or workflow.

## Author-Ratified Rulings

Grouped by kind. Dates in each heading are the ratification dates; a ruling
that was later implemented or narrowed carries that supersession inline.

### Item 51 — the vocabulary and the cast settled, 2026-09-25

The plan's replacements now hold in the reader prose of the opening, Chapters
1–30 and Part V. "Derive" became "follows" or "the rules conclude";
"supplied" became "recorded" or "given"; a "qualified" finding became one
"properly made"; an "effective" finding or version became one "in force",
while an effective remedy, alternative or control keeps its word; "reader",
for an institution, became "the responding office"; "pen" and "credential"
became "the authority to sign findings"; "lease" and "window" became the
custody authorisation and its review date or period; and "selected current
record" became "the version in force". Carrying a status between versions of
the record is now described plainly, with its mechanics in the method's new
"Versions of the record", which quotes the lineage-collision and standing
rules verbatim. Harness and fixture names in prose (Targ4, Nogra, Nogrb,
Partnr, Sock, the Amend_ proposals, SchemeM, PublicGuarantee, HighSec,
Homestay, Provender, Ledgerwitness and the rest) and institutional constants
(Review, Appeals, Convocation, Electorate, Chronicle, TemporalReview) became
descriptions, and the pins keep them. The opening's glossary defines
"properly made", "in force", "the version in force" and the custody
authorisation. Where a chapter's test adds a fact about a cast member, the
prose says "Suppose…". One Chapter 14 sentence that denied the case
participant's personhood was corrected: since item 72 the first-contact entry
makes the participant a person.

The names that prejudged their cases are neutral everywhere the current
design is stated or tested: Boss is Tove, Rebel Iris, Sly Lior, Vex Mael, Hex
Saba, Rex Dara, Lupo Edo, Don Faro, and the victim fixture Scapegoat is
Tamsin — in the constitution's facts, pins, cases, generators, reviewed JSON,
generated reports, the red-team index and the companion's data, with eleven
paths moved (`records/mael_*`, the `party-*-dara` shield-independence cases
and `counterfactual/no-choose-tove`). Dated records keep the names they were
written with: this file, the tracker, the appendix decisions and briefs, the
reviews, the reader drafts and the retired receipts. The companion's scenario
keys (`rex`, `sly-selfcert`) stay, so shared links keep working. Regenerating
the integrity, knowledge, record-power and statistics families reproduces the
renamed output byte for byte.

The recurring cast is fifteen people whose facts stay the same wherever they
appear: Nell, Hano, Ruk, Bela, Cira, Marisol, Esa, Adam, Ivo, Kel, Gia, Wren,
Iris, Tove and Mael. Every other name appears only in its home chapter; the
opening's index names everyone with that chapter. `tools/prose_lint.py` now
holds `RECURRING` and `HOME`, counts a name outside its home chapter as a
stray held at zero in every chapter, and counts "pen" and "credential" as
terms of art; Brix, Dunya, Zeno and Voss, which no ordered input uses, left
its list. Its baseline ratchets down: every chapter's terms of art are within
the plan's five per 1,000 words (from as high as 27.6), no chapter carries a
banned term or a harness name, disclaimers fell in most chapters, and
negation held or fell. Distinct names still exceed five in Chapters 3, 10,
16, 24, 25, 28 and 29, which the restructure's merges and rewrites address.

Twelve changed sentences and headings moved their reviewed references in the
full-society ledger, the record-integrity case, the receipts source and the
reader-coverage source; the coverage report (243 passages) and receipts are
regenerated. The exact prose is `session-drafted, author-approved under
delegated approval (2026-09-13)`. The rule count is unchanged at 7,558. All
141 authoring development tests pass, with five declared ignored, the fifteen
prose-lint unit tests pass, and the prose check passes. `RIGHTS_VERIFY_JOBS=4
./verify.sh` passes 90,009 pins across 16,354 cases with complete
contradiction checks and no findings in 1,201.00s.

### Item 50 — the core replayed in a second engine, 2026-09-25

`tools/second_engine.py` translates the constitution statement by statement
into clingo's input language (9,575 clingo statements after a disjunctive
body becomes one rule per disjunct) and replays selected cases through clingo
5.8.2 from PyPI: constants become quoted strings, `$variables` clingo
variables, `event { P() }` a constant, `~` negation as failure and the
equalities `=` and `!=`; `entitled(every person, …)` becomes a rule from
`person`; `admits` and `derived_only` are dropped because clingo refuses no
input. Each case is grounded once, and every fact and accepted rule its
fixtures and pins assert is guarded by an external atom of its own, switched
on at its position, so each query sees exactly what was asserted before it.
Refusals, scoped acceptances, shell checks and contradiction scans are not
compared, and only live-source cases run.

The comparison covers standing, the floor, delivery, custody and the shield:
the chapter pins for Chapters 1, 2, 4, 5, 24 and 27–29, the floor, standing
and delivery source pins, every live custody, placement and delivery case,
and the six stress cases — 129 cases and 1,673 queries.
Every answer agrees with the verdict its pin records. The one class of
difference found on the way was the translator's own: declaring an asserted
fact itself external fails silently in clingo when a rule can also derive
the atom, so a person asserted in a custody case and also derivable from
custody went missing; guarding each assertion with its own external atom
fixed it, and the run was repeated from the start. The results and report are
in `book-1/source/measurements/`, and the method's "Testing the tests" states
the result with its limits. A reimplementation made inside this project is a
cross-check, not an independent reproduction: the same author chose the
translation, and a misreading shared by both engines would pass both; an
outside reimplementation remains welcome under item 66. No rule, pin or case
changes. The authoring development tests pass (141, five declared ignored).

### Item 49 — mutation testing over the rules, 2026-09-25

`tools/mutation.py` changes one rule of the live constitution in one way —
drops a conjunct, flips a negation, or swaps two roles inside one atom — and
runs the change, as an in-memory edit, against the cases whose pins name the
rule's head: cases on the live source and cases on a temporal stage or
counterfactual base whose edits leave the rule intact. It generates only safe
mutants and skips swaps that change nothing. The recorded run sampled three
rules per family with seed 49 and at most twenty-four cases per mutant, half
expecting the head TRUE and half FALSE, ranked by how many of the rule's own
constants each case mentions; a mutant that exhausts 300 seconds or 16 GB is a
resource result. Results, hand-written dispositions and the generated
survivors list live in `book-1/source/measurements/`, outside the top level
the reviewed-reference test reads, because case identifiers look like paths.
No score is kept: the assurance portfolio refuses aggregate scores.

Of seventy-eight mutants, forty-six were killed at once, one was refused and
thirty-one survived. New pins kill thirteen of the survivors, the wider case
selection kills one more, and one exhausts memory rather than surviving. The
pins cover a reader requesting its
own review (Chapter 22), a witness who observed nothing (delivery), the
legacy family, marriage and sibling barriers holding for persons only and
following the sibling entry's own subject (equality source pins), a condition
record bound twice (the custody condition findings), a public body present in
the jurisdiction (universal standing), each timing witness's missing standing
and preserved carry on its own (TA-10, TA-05), a challenge naming no
conflicting claim (TA-13), a challenge naming no custody authorisation
(TA-41), and two new independence cases giving two roles one holder — a
knowledge reader who is the actor, and a statistics source reviewing itself.
Earlier sampling passes added the heard-teaching route (Chapter 5), a
tribunal credential that does not follow (Chapter 23), a failure pattern split
across tiers (Chapter 7), each dating witness alone (Chapter 24), each
delivery route's provider witnessing itself (delivery) and a generated
amendment case holding uncredentialed consent noise. The missing universal
kinship barrier the pass surfaced was examined and is deliberate: the child
record's pins already state that the kinship barrier forbids substituting a
recorded marriage or sibling status, so it has no subject without one.

The sixteen survivors left each carry a disposition. Thirteen drop or negate
one writer's copy of a field its record's other writers still record, or
leave a single-actor route asking less than the full one. No generated case
sees either, because generated omission cases remove a field from every
writer at once, so four development tests on the written rules reject them,
each sabotage-tested:
`record_attesters_record_every_field_in_the_uniform_families`, widened from
seven template families to ten by comparing only the `$source`, `$evidence`
and `$review` attesters (the four families whose sources alone record some
fields are asserted by membership);
`each_field_is_attested_the_same_way_across_its_family`, holding every family
to one attestation pattern per field with five deliberate two-pattern fields
listed; `single_actor_routes_ask_their_source_for_the_full_record`, which
holds each ruling-D6 route to the full route's source fields and its review
duty to the route's own; and `both_timing_witnesses_record_every_field`. One
survivor is subsumed: dropping the record-path closure's guard only makes an
entry on a cycle precede itself, and every reader either requires the entry
free of an order collision or needs the same event-path order. Two are
missing pins in generated families: the ecology appointment's sixteen
variants, of which the cases exercise one, and the economic generator's
alternate-review routes, pinned one kind per power.

The harness itself changed twice on evidence: a case's own refusal pins print
stratification errors, so only a mutant line that failed to load now counts
as refused and memory exhaustion is checked first, which moved one T1 mutant
from refused to resource; and the selection now reaches derived bases, which
is what let the temporal alarms be observed at all. The rule count is
unchanged at 7,558. All 141 authoring development tests pass, with five
declared ignored, and the prose check passes. `RIGHTS_VERIFY_JOBS=4
./verify.sh` passes 90,009 pins across 16,354 cases with complete
contradiction checks and no findings in 1,212.23s.

### Item 48 — the flow constraints named and the engine situated, 2026-09-25

The method gains "Six ways a fact is kept from a consequence", a table of the
constraints the design already uses — closed inputs (`admits`), conclusions
nobody may write (`derived_only`), purpose-bound reads, endpoints nothing reads,
no confinement from absence, and scope binding — each pointing to the pins or
development tests that enforce it (`a_purpose_limited_record_is_read_only_for_its_purpose`,
`a_duty_is_not_an_action_because_nothing_reads_one`,
`floor_actualities_have_no_downstream_consumer`,
`no_confinement_reads_an_absent_home_family_or_work_entry`,
`scoped_authority_is_not_unary_permanent_answerability` and
`global_findings_cannot_lend_effects_to_unqualified_records`). It relates them
to information-flow control (Denning; Myers and Liskov), taint analysis and
contextual integrity, and says they check written rule forms and can miss a
semantically equivalent attack. "Where the engine comes from" says Nibli is
written and maintained by the book's author, so its agreement with the
constitution and tests is a consistency check within one project; relates it to
Datalog with stratified negation (Apt, Blair and Walker), Sergot et al.'s
British Nationality Act program, Catala and the OECD's Rules as Code paper, with
Hildebrandt's textbook on automated compliance; and states what is new — the
use, not the logic. Each citation is an item 46 registry entry.

The stratification refusals leave the chapters. Chapter 4's "Where the
protection stops" and Chapter 27's two protection sections state the protection
and its reach and point to the method, which now says the refusal is designed —
entitlements written as events downstream of personhood, with a prisoner still
a person — and specific to how a rule is written, the home-record variant
loading; it also carries the recognition and belief results from Chapter 4 and
Chapter 29's operative-bar refusal. The method's sealed scope stands: excerpts
with pointers, no rendered English, no proof traces, no compute backend. Two
amendment-audit needles follow their text into the method; one coverage setting
is relabelled. The exact prose is `session-drafted, author-approved under
delegated approval (2026-09-13)`.

No rule, pin or suite changes, so item 72's complete run on the same formal
inputs stands. The claim-discipline (including the method's quotation test),
reference, reader-coverage, receipt and floor-vector development tests pass, as
does the prose check once three new Chapter 27 sentences were rephrased to hold
its negation figure.

### Item 47 — Part V's evidence corrected and its lineage credited, 2026-09-25

The coercion joint opened with Tanzania's villagisation, which concerned forced
relocation rather than criminal confinement. It now opens with evidence on
confinement, each item stated as its item 46 entry supports: the Norwegian
random-judge study (imprisonment reduced reoffending, the gain coming from
defendants not working before prison while the previously employed lost
employment, for one country's prisons and the defendants whose sentence turned
on the judge), Pratt on Nordic penal exceptionalism, Chin on collateral
consequences, and the Bihar undertrials in the lapse paragraph. Davis and
Gilmore state the abolitionist case the joint answers. The cooperative pay
evidence moves to the valuation joint, where it bears on the choice, and
republic-level evidence joins the rest: Robodebt, the Dutch childcare-benefits
inquiry, SyRI, the NJAC judgment, Switzerland's chambers, Uruguay's colegiado,
the New Zealand, Chilean and EU resident franchises with López-Guerra and
Goodin, the debt brake and its 2023 ruling, T-025 and Grootboom, ADM Jabalpur
and the Forty-fourth Amendment, and Kesavananda, Article 79(3), Landau and
Waldron.

The lineage is credited where the argument uses each idea, and where the design
departs from it: Shue, respect/protect/fulfil and the minimum core (the UN
committee's duty to provide is conditional; this floor is not), Sen and Drèze,
Pettit, Anderson (her guarantee allows work conditions this design refuses),
Rahman, Raworth and Rockström, Ostrom, Nissenbaum and purpose limitation, the
three Rs, Article 12 of the CRC and the CRPD, Kymlicka and Shachar, Crépeau and
Hastie, and the Quebec Reference. Rawls is not credited, because item 46 could
not verify the plan's sentence. The opening names the principal lineage and
points to Part V. Every new footnote's locator was checked against the
registry, three replaced where the draft had not used the recorded one. The
Part V figure bindings drop Tanzania and gain sixteen rows (39). The
narrative-register decision records the change; the exact prose is
`session-drafted, author-approved under delegated approval (2026-09-13)`.

No rule, pin or suite changes, so item 72's complete run on the same formal
inputs stands. The claim-discipline, reference, reader-coverage and receipt
development tests pass, as does the prose check once one new "voided" became
"struck down".

### Item 46 — the plan's sources, checked and registered, 2026-09-25

The revision plan named about 150 sources in §8.2, §8.5, §9.1, §9.2 and §10 and
said its citations were written from memory. Seven research passes checked each
against a primary source or the most authoritative reachable copy: for case law
the court, date, citation and deciding passage; for statutes and treaties the
article and official text; for books and articles the edition, publisher,
pages or DOI where they could be read. The registry gains 144 entries, each with
a URL or DOI and notes saying what was actually read — full text, official page,
abstract or publisher's description — and the pinpoint verified; a book whose
page could not be read carries that limit rather than an invented page. Two
locators were supplied by a later publisher-page search, and both entries say
so.

Three sources are not registered, and `registry/plan-sources-2026-09.md`
records them with the claim each would have supported: Rawls's *Political
Liberalism*, whose sentence on the social minimum was found only in secondary
quotation, with the locators to check; and the Finnish basic-income report and
the 2018 Aadhaar judgment, already registered. The same record lists where the
plan described a source more strongly than it supports — among them that
Anderson's guarantee is conditional on working, that General Comment 12's duty
to provide applies where people cannot help themselves, that Neubauer gives
future generations no present rights, that the 2019 German sanctions ruling
declared the rules incompatible rather than void, that Chile now requires ten
years' residence, that Refugee Convention Article 33 has exceptions, and that
the incarceration study's gains are for defendants not working before prison.
Items 47 and 54–59 read those notes rather than the plan. The TODO's references
to item 46 now point there.

No rule, pin, chapter or suite changes, so item 72's complete run on the same
formal inputs stands. `registry/check.py` passes with 221 claims, and the
claim-discipline and reference development tests pass.

### Item 72 — an outreach report reaches the person it reports, 2026-09-25

Item 45's never-recorded-person case found that Article 1c's escalation, built
for the person nobody has come for, read personhood, while a report that nobody
was acting for someone was not a standing root, so a report about a person no
record had entered derived nothing until the worker also wrote the encounter.
The initiation card in the life-course family never read personhood at all, so
an appointment could complete for a name the design owed nothing. Dropping the
escalation's person guard was compared and refused: it would oblige the
alternate toward a name that still had no standing, no debt and no advocate's
duty. The report records an encounter, written by whoever saw the person, as a
first-contact entry is, so it is now a standing root in the universal-standing
block: `observe($writer, $entry, $subject, UninitiatedAssistanceScope) &
~public($subject) -> person($subject)`. It gives standing and nothing adverse,
and a report naming a public body makes nobody a person.

The initiation card cannot read `person` itself. Personhood is also concluded
from completed records (an arrest order, a carried standing status), so a
completion reading it merged `complete` with custody's negated guards, and the
engine refused it (`correct` -> `clean`). Five helper rules beside the standing
roots conclude `related($x, StandingEncounterEntry)` from the base encounters —
birth, first contact, presence, effective control and the report — and the card
reads that. Somebody whose standing comes only from custody or an arrest record
is reached by a first-contact entry, which anybody may write. The family-life
generator gives a card reading the helper a first-contact entry in its fixture
and a generated `without-standing` case in which nothing completes.

The stress case now runs the report alone to standing, every debt, the
advocate's and the alternate's duties, serve-first and both enrolment barriers,
with no custody or credibility finding and nobody made a person through a public
body; its defect pin resolved. Chapters 2 and 7 pin the new root, and Chapter
14's pins, which read the generated positive fixture, now say where the
participant's standing comes from. The opening note, Chapters 2, 7 and 14, the
method, the life-course contract card, the initiation receipt and the red-team
index record it; the exact prose is `session-drafted, author-approved under
delegated approval (2026-09-13)`. Rules move from 7,577 to 7,583. The affected
cases pass 2,678 pins across 672 cases (focused, partial). The first full
development run caught the ledger needle quoting the method's standing sentence,
which now follows it; all 137 authoring development tests pass, with five
declared ignored, as does the prose check once one Chapter 7 and one Chapter 14
sentence were reworded to hold their figures. `RIGHTS_VERIFY_JOBS=4 ./verify.sh`
passes 89,972 pins across 16,351 cases with complete contradiction checks and no
findings in 1,245.58s, and no known-defect pin remains: the six item 45 found
are resolved.

### Item 71 — a reading of the core must give its grounds, 2026-09-25

Item 45's expansive-reading case found that the independent effect reviewer
could withhold an amendment's certification by recording any contrary value in
the single-valued compatibility field: the two attesters' readings made the
record ambiguous, the reviewer gave no reasons, provision or changed effect, and
the proponent's challenge obliged a reader whose decision nothing could turn
into certification. The amendment contract said neither reviewer had a final
veto; for that record one did.

The compatibility field is no longer single-valued across attesters, so a binder
and a reviewer who read the core differently make nothing ambiguous; one
attester recording two readings still does. Certification gains a second route:
the binder's positive reading plus an independent final compatibility review's
recorded finding (`FinalReviewFoundCandidateCompatible`) by an actor authorised
for the record who is not the binder, the effect reviewer, the operator, one of
the three political bodies or an operator of the political result. Every other
field still needs both attesters, and both routes keep the `~contradict` guard,
so an established animal-core breach still blocks certification. A reading
refuses the candidate only when an attester records `ReasonedCoreIncompatibility`
and names a provision from a closed corridor vocabulary of fourteen, the
candidate change and the reasons; it withholds through `contradict` until the
final review finds the candidate compatible, which a helper concluding
`related($record, AmendmentFinalCompatibilityFound)` from base facts reads. A
challenge naming the record obliges the final reviewer to decide
(`DecideTheCandidatesCompatibilityWithTheCore`), read by nothing. The contract
card compares keeping the field single-valued with reasons required, treating a
contrary value as nothing, and letting the final review override every refusal.

The amendment generator writes fifteen new cases: the unreasoned reading alone
and answered, the final reviewer being the binder, the effect reviewer, the
operator, the Assembly or unauthorised, a reasoned breach refusing and
overturned, one naming no corridor provision, no change or no reasons, one
attester's two readings, and the challenge duty with its self-challenge control.
The stress case now runs the challenge to a certified candidate, its defect pin
resolved. Chapter 22 states the rule, and the contract card and red-team index
record it; the exact prose is `session-drafted, author-approved under delegated
approval (2026-09-13)`. The procedural-load table names the final reviewer.
Rules move from 7,558 to 7,577. The amendment, Chapter 22, ecological corridor
and amendments cases pass 1,347 pins across 275 cases (focused, partial).
The first full development run caught the final-review helper: it named the
effect reviewer `$review` to hold the final reviewer apart from it, and the D6
single-actor test read a rule whose named reviewer observes nothing as help
taking effect on one actor. The helper now requires that the final reviewer
holds neither attester role (`~authorized`), which says the same thing
directly, and all 137 authoring development tests pass, with five declared
ignored, as does the prose check. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes
89,960 pins across 16,350 cases with complete contradiction checks and no
findings in 1,245.94s, the defect pin for item 72 still reproducing.

### Item 70 — continuity stays public, witnesses stay independent, 2026-09-25

Item 45's agency-and-contractor case confirmed two gaps. The item 36 continuity
rule obliged whatever body a failure certification named, so continuity could be
certified onto a private contractor; and each recipient-side receipt route held
its witness apart from the provider by identity only, so the region that
contracted a provider could witness that provider's delivery, the duty-bearer
certifying its own discharge. The continuity rule now requires the named tier to
be publicly answerable (`authority($upper)`), and `public(CommonTier)` joins
`public(State)`. The five receipt rules now also hold the witness apart from the
State, the common tier and any body a presence witness records as the person's
region or locality, through two helper rules concluding `related($body, $x,
FloorDutyBearer)` from base facts, so the floor actualities' new negative read
sits outside every cycle. Those bodies are exactly the ones that owe a person
the floor, so a public principal behind a contracted provider is among them
without reading the delegation record.

The two counterfactuals quoting the receipt rules and the placement case that
removes the shelter route follow the new text; that case's edit lives on the
case itself, which a first update of the bases alone missed. Chapter 4's pins
require a public continuity tier and refuse one certified onto a private
provider, Chapter 5's refuse a delivery witnessed by the person's recorded
region, and the stress case now runs both refusals beside an independent
inspector completing the receipt. Chapters 4 and 5 state the rules, one Chapter
4 clause reworded to hold its negation figure, and the delivery and economic
decisions and the red-team index record them; the exact prose is
`session-drafted, author-approved under delegated approval (2026-09-13)`. Rules
move from 7,556 to 7,558. The fourteen affected cases pass 308 pins (focused,
partial). The first full development run caught the method's verbatim quotation
of the food receipt rule and one assertion-surface needle on its tail; both
follow the new text, and all 137 authoring development tests pass, with five
declared ignored, as does the prose check. `RIGHTS_VERIFY_JOBS=4 ./verify.sh`
passes 89,922 pins across 16,335 cases with complete contradiction checks and no
findings in 1,224.87s, the defect pins for items 71 and 72 still reproducing.

### Item 69 — a compared claimant cannot attest, 2026-09-24

Item 45's manipulated-urgency case found that a scarcity allocation held its
three attesters apart from the manager, reader and alternate but not from the
claimants it compared, so the continuity claimant could attest the imminent harm
that decided the allocation in their favour. The compared claims already named
each claimant, so a helper rule reading only base facts concludes
`related($claimant, $record, ScarcityInterestedClaimant)` for each claimant the
record's own source lists. The allocation requires its source, evidence
attester, reviewer and manager to be outside that set, and a defect finding
requires its attesters to be outside its target's set, so a losing claimant
cannot attest the withdrawal of the winner's allocation either. Recorded
shortfalls were checked and left alone: a claimant attesting their own unmet
claim finds against nobody and only raises a repair duty. A claimant's route is
the challenge.

The scarcity generator adds a refusal for each allocation attester and the
manager and each defect attester who is a compared claimant, with a control that
still qualifies. The stress case shows the honest allocation qualifying and the
claimant-attested one refused. Chapter 6 states the rule, and the scarcity
contract card and red-team index record it; the exact prose is `session-drafted,
author-approved under delegated approval (2026-09-13)`. Rules move from 7,555 to
7,556. The scarcity, Chapter 6 and stress cases pass 1,805 pins across 421 cases
(focused, partial). All 137 authoring development tests pass, with five declared
ignored, and the prose check passes. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes
89,919 pins across 16,335 cases with complete contradiction checks and no
findings in 1,162.84s, four defect pins still reproducing for items 70–72.

### Item 68 — who owes an election that cannot be held, 2026-09-24

Item 45's election case found the state-form decision promising the electoral
body "a legal duty and authority to call a fresh election" with only the
authority formal, and no rule for a People's Assembly whose term ends before its
successor can be elected; the ledger reserved any default outside the caretaker
limits for a new ruling. The state-form decision's new section 14 takes it under
the delegated approval, comparing no sitting chamber (refused: the corridor
keeps the Assembly able to sit), full continuation (a mandate extension), a
standing committee, a caretaker sitting and a court-set date, and naming who
bears the delay and what would reopen the choice.

Article 2b, hand-written after the franchise, holds five rules. A term-end
certification by a source and an independent reviewer, neither of them the
chamber, obliges the electoral body (`FSBOD_06`) to hold the election at the
first lawful opportunity, permits the outgoing chamber to sit and obliges it to
act only under existing law, the floor, oversight and the election's needs,
without amendment or irreversible measure; the permission and limit end when the
reviewer records that the successor has first met. A finding that the election
cannot yet be held, from a source and reviewer outside the chamber and the
Executive Council, obliges the Constitutional Court to review it promptly and
lifts no duty. A certified failure by the electoral body to hold an election
that could be held obliges the Court to order it. Nothing reads the duties or
the permission, which the procedural-load table classes as institutional.

The election stress case now runs the whole sequence, including a self-certified
term end and an executive impossibility finding that oblige nobody, and Chapter
20's pins carry the core of it. Chapter 20's new section "When the election
cannot be held" states the rules; the red-team index and the decision record
them; the exact prose is `session-drafted, author-approved under delegated
approval (2026-09-13)`. Rules move from 7,550 to 7,555. The stress cases and
Chapter 20 pass 106 pins across seven cases, the remaining five defect pins
still reproducing (focused, partial). The reader-coverage ledger gains the new
passage (243). All 137 authoring development tests pass, with five declared
ignored, once the reader-coverage failure that caught the unclassified section
was answered, and the prose check passes. `RIGHTS_VERIFY_JOBS=4 ./verify.sh`
passes 89,882 pins across 16,326 cases with complete contradiction checks and no
findings in 1,139.88s.

### Item 67 — appointment control made challengeable, 2026-09-24

Item 45's captured-appointments case showed a certified selection standing after
one of its own attesters recorded that one coalition directed both nominally
separate selectors: nothing read a record written after certification, and no
independent finding of appointment control existed, although the state-form
ruling makes concentration independently challengeable. Three routes now answer
it. Each examined-kind anchor — no majority control, no conflict of interest, no
prohibited political finance, no forbidden districting purpose — certifies an
absence in a one-value field, and twelve generated rules make any other value
there, written by one of the result's own source, evidence or review attesters,
withhold the record through `contradict($record,
DemocraticIntegrityAuthorization)`, as record ambiguity does in the newer
families; an uncredentialed writer reaches nothing. The integrity contract
`appointment-finding` names the target selection, the controller, the kind of
source, the mode of control and the selectors, under the three attesters, reader
and alternate every integrity finding carries, with the controller barred from
attesting; it withholds that selection alone and obliges the reader to correct
it.

The item also asked whether the captured-source fallback and the ordinary
selection may both hold one seat. They may not, but withholding the fallback
when the selection completes would read `complete` under negation inside its own
cone, which the engine refuses. The fallback therefore ends as its own record
says, on the lawful appointment, and where both are current for one seat the
body holding the fallback owes its end
(`EndTheFallbackAppointmentNowTheSeatIsLawfullyFilled`), read by nothing. The
finding is classified adverse in the procedural-load table. Chapter 17 states
the three routes, the integrity contract card and the state-form note above
record them, and the stress case's defect pin now passes as a withheld
selection. The exact prose is `session-drafted, author-approved under delegated
approval (2026-09-13)`.

Rules move from 7,527 to 7,550. The integrity generator writes 46 appointment
cases: the positive record and every omission, kind and identity variant, a
selection withheld with every personal protection unchanged, a different
selection left standing, a controller who attests refused, a contrary anchor
record withholding the selection, an uncredentialed one reaching nothing, and
the fallback duty for the same seat but not another. The integrity, state-form
and captured-appointments cases pass 1,681 pins across 459 cases (focused,
partial). All 137 authoring development tests pass, with five declared ignored,
and the prose check passes. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 89,868
pins across 16,326 cases with complete contradiction checks and no findings in
1,146.01s, five defect pins still reproducing for items 68–72.

### Item 45 — six stress tests, and the defects they found, 2026-09-24

Six scenarios the revision plan named now run as cases under
`tests/pins/stress/`, each stating what the design does and where it stops, with
a `:defect` pin marking each confirmed gap and a tracker item for each, 67–72,
placed right after this one under the resolve-before-defending rule. Three
read-only surveys of the source mapped the rules first; every gap below was then
reproduced by a pin, not taken from the survey.

**Captured appointments.** A selection takes effect only when its source,
evidence and review attesters all certify that no source holds majority control,
and one attester's refusal withholds it. Once certified, a later contrary record
reaches nothing and no independent appointment-control finding exists, although
the state-form ruling makes concentration independently challengeable (item 67).
**An election that cannot be held.** A declaration used to delay an election or
extend a mandate falls with its requisition when reviewers find it, the
requisitioned person keeps standing and the floor, and a caretaker holds no
confidence mandate. Calling a fresh election is a power, not a duty, and nothing
governs a lapsed Assembly (item 68). **Manipulated urgency.** One honest
attester recording a different mitigation key makes the allocation ambiguous and
it falls, while the shortfall's repair duty survives; the rival's challenge
obliges the reader to weigh their evidence. A compared claimant may attest,
however (item 69).

**Agency and contractor blame.** The debt, the common tier's backstop and the
region's duty read personhood and presence, never fault; a delegated duty binds
the delegate and keeps the principal bound; and split blame moves no continuity,
which the backstop never depended on. Continuity can be certified onto the
private contractor, and the region can witness its own contractor's delivery
(item 70). **An expansive reading of the core.** A contrary compatibility value
with no reasons blocks one amendment record and creates no right; a fresh record
is not reached by it. The proponent's challenge reaches no decision that can
certify (item 71). **A person nobody has recorded.** A first-contact entry by
whoever made the contact yields standing, every debt, the advocate's duty,
serve-first and the enrolment firewall, and no adverse conclusion; a report that
nobody acts for the person reaches nothing until the encounter is also written
(item 72).

The disclosure, non-delivery and lapse scenarios the plan also named are items
37–39, and the cultural-practice case waits for item 58. The red-team index
gains the six entries. No rule, chapter or generator changes. The six cases pass
78 pins, six of them reproducing the confirmed defects (focused, partial); the
reference, claim-discipline and reader-coverage development tests and the prose
check pass. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 89,723 pins across 16,280
cases with complete contradiction checks and no findings in 1,138.68s, the six
defect pins still reproducing.

### Item 44 — bodily safety and material security, 2026-09-24

Ruling D8 is implemented, and the floor has nine items. `secure` is bodily
safety: it keeps its entitlement, its debt `owe(State, Secure, $x)` and every
refusal, and it has no receipt route, so like belief and expression it is a base
predicate at the floor's stratum, answered by the protective duties rather than
by a receipt. `suffice` (*banzu*, enough for a purpose) is material security,
with its own entitlement and debt, `owe(State, Suffice, $x)`. The recipient-side
route, with its authorised witness distinct from the source, now concludes it
and keeps the `SecurityScope` observation. The ecological family's
existing-material-floor vocabulary admits it by the rule every other item has.
Care keeps `healthy`, and company keeps its route.

Chapter 4's pins gain fifteen (100): the new item's entitlement and debt for
Bela and for the one-line record, and a refusal of a rule confining someone for
lacking each of the nine items or the material-security debt. The floor suite
gains the `~suffice` refusal (104), and every pin file that lists a person's
floor debts and entitlements gains the new pair: Chapters 1, 2, 5, 20, 21, 25
and 27 and the delivery and child-record cases. From the same receipt the
received-outside-custody case and the delivery family's own pins (31) now derive
material security and not bodily safety, and
`delivery/counterfactual-receipt-certifies-safety` points the route back at
`secure` and shows the receipt certifying safety. The ecology, public-safety and
justice generators carry the ninth item in their floor lists, so each protective
instrument's floor refusals and the justice boundary cover it; the floor
development tests read nine items; and the procedural-load table names the
receipt route's new effect.

The coverage map gains the contract for the two items, the full-society ledger's
thirteen floor-invariant texts name them, and the delivery decision and taxonomy
map record the implementation. The opening note, its glossary and Chapters 1, 4,
5, 20, 21 and 29 use the ruled names; Chapter 4 states what separates the two,
and Chapter 5 that safety has no delivery certificate. The prose check no longer
bans "material security", which the ruling makes a floor item's name. The exact
prose is `session-drafted, author-approved under delegated approval
(2026-09-13)`.

Rules move from 7,524 to 7,527 and predicates from 92 to 93; derived predicates
and strata are unchanged. The affected chapter, floor and delivery cases pass
837 pins across 15 cases, the counterfactual passes four, and the 45 regenerated
ecology, public-safety and justice cases pass 752 (focused, partial). All 137
authoring development tests pass, with five declared ignored, and the prose
check passes. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 89,645 pins across
16,274 cases with complete contradiction checks and no findings in 1,141.73s.

### Item 43 — fast for help, slow for harm, 2026-09-24

Ruling D6 is implemented. Every beneficial record kind in item 42's table gains
a single-actor route: its source's record alone makes its duties, permissions
and barriers take effect, the independent reviewer named on the record owes
prompt review (`ReviewTheSingleActorRecordPromptly`), and the reviewer's
withdrawal on review (`SingleActorEffectWithdrawnOnReview`) switches them off; a
family's defect route withdraws them too where it has one. The completed record,
standing and every authority stay behind full procedure, so an adverse effect
that reads a completion cannot be reached through help one actor gave. Forty-two
routes cover the equality, life-course, knowledge, record-power, scarcity,
integrity, mobility, justice, public-safety and ecological families. Adverse
records and appointments over another person are unchanged, and generated cases
show both refused on one actor.

Before applying it, item 42's beneficial class was tightened to effects that
find against nobody and ask of others only what they already owe. Case relief
and court remedies become oversight; protected title, binding remedies and the
separate reparations become adverse; membership, which rests on a collective's
own acceptance, animal-coverage extensions, which change whose duties apply, and
the economic carries, which reconcile records, become institutional. Sixty-seven
effects remain beneficial. The duplicated attestation item 42 listed is removed
for them by this route; for adverse records and appointments it is the
independent confirmation D6 retains.

`floor_vector_tests::help_takes_effect_on_one_actor_and_harm_waits_for_review`
states the ruled property beside the unchanged completion symmetry: a rule whose
named reviewer observes nothing must conclude a duty, permission or barrier,
name a beneficial record kind, carry the withdrawal guard, owe prompt review in
a sibling rule and grant no permission another rule reads. Four sabotage
controls (an adverse kind, a completion, a missing guard, a missing review duty)
each fail, and a count guard caught a first draft that matched nothing. Each
family's generated cases show a single-actor record taking effect or refused by
class and its withdrawal on review; an accommodation corrected by a reviewed
defect is an integration case; Chapter 12's pins run the accommodation; and the
procedural-load table gains an acting-before-effect column. Four temporary-root
tests needed the classification copied into their roots.

Chapters 3 and 12, Part V's capture joint and the method describe the route, and
the ten families' contract cards record it; the exact prose is `session-drafted,
author-approved under delegated approval (2026-09-13)`. Rules move from 7,414 to
7,524, and the constitution grows to 57.7 MB, under the 75 MB trigger. Focused
runs pass the six template-shaped families and integrity (9,722 pins across
2,766 cases), mobility (1,597 across 384), justice (1,285 across 341), public
safety's protected facts (1,101 across 286), the affected ecological cases
(8,167 across 1,179) and the sixteen ecological single-actor cases (90). All 137
authoring development tests pass, with five declared ignored, and the prose
check passes. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 89,561 pins across
16,273 cases with complete contradiction checks and no findings in 1,116.54s.

### Item 42 — the procedural load, measured, 2026-09-24

`./generate.sh procedural-load` reads every gated effect in the constitution —
each completed record, permission, authority, restraint, stay, custody, standing
and delivery conclusion that rests on an authorised role — and writes
`book-1/source/procedural-load.md`: the roles that act on each effect's own
record, the roles that write identical fields onto it, and the roles on every
completed record it reads. `book-1/source/procedural-load-source.json`
classifies each effect as beneficial, adverse, power over others, oversight or
institutional, with a reason; 422 explicit entries cover every record kind and
every effect gated directly on raw records, and 94 permissions and authorities
inherit the class of the record they read, seven of them overridden where the
inherited class was wrong. The classification is conservative: an act that also
binds a custodian, an owner or a third party is adverse, and an appointment or a
role acting for somebody is power over others. Duties and barriers are outside
the table, which says so and counts them.

**The duplicated steps are listed by family.** The newer families attest each
record three times over: a source, an evidence attester and an independent
reviewer write the same fields. Public safety adds its authorising body to that
set, the ecological family has a source and a reviewer writing the same fields
on almost every record, and state-form and economic records repeat source,
record-review and acting-body attestations. Of the 516 effects, 83 are
beneficial; most need five or six roles today, and the delivery, supplement,
compensation and certificate conclusions need one. The table is a list, not a
score, and totals nothing.

Six development tests hold it: the classification covers the source and uses
every class; an unclassified record kind, a classification naming no effect and
a role with no function each fail; the three matching accommodation attesters
are found; and the report is current. The method gains "What an act needs before
it takes effect", a list of what the beneficial effects need, `session-drafted,
author-approved under delegated approval (2026-09-13)`. No rule or pin changes,
so item 41's complete run on the same formal inputs stands: 88,886 pins across
16,139 cases with no findings. Item 43 applies ruling D6 from the table.

### Item 41 — the signing restriction, stated and named, 2026-09-24

A credibility finding no longer reaches recognition, which the design removed
(item 21), and restricts only its subject's signature on a new credibility
finding (item 19): measured, `false/1` is read by the rule qualifying a new
candidate and by three duties owed to the subject, and a finding on the deceit
ground also withdraws the shield for that disclosure alone. The chapters called
it "voiding" more than fifty times and described the restricted act as "a new
adverse finding", both wider than the source, and listed "recognition loss"
among consequences that do not follow. The prose now names the mechanism as a
credibility finding and its effect as a signing restriction; Chapter 25 is
titled *Credibility Findings* with its file path kept, its opening states the
whole reach, and the recognition residue is gone except where the book explains
why no recognition status exists, uses recognition as a forbidden priority key
or tests a hostile rule.

**The reach stays with credibility findings.** Widening it to every office that
makes adverse findings was refused: every record completion elsewhere already
holds its attesters apart and requires an independent reviewer, the integrity
rules let a conflict of interest proved against an office withhold its
permission for the affected act, and the widening would turn a finding about one
act into a bar from public office and employment. Part V states the argument,
who bears its cost and what would reopen it; the credibility and recognition
decisions record it. Chapter 25's pins now show Bela, under a finding, as the
source of a floor-shortfall certification that obliges the Constitutional Court
to decide, beside the existing restricted signer (Lupo with Partnr),
unrestricted signers (the Targ4 pair) and restorations. The exact prose is
`session-drafted, author-approved under delegated approval (2026-09-13)`.

The rewrap moved three reviewed needles, held whole again; one ledger locator
follows the reworded Chapter 24 sentence, and two coverage headings follow their
renamed sections. No rule changes. Chapter 25's two cases and the credibility
cases pass 375 pins in 18.60s (focused, partial); the reference,
reader-coverage, claim-discipline, receipt and floor-vector development tests
pass, and the prose check passes with lower recorded figures for most chapters.
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,886 pins across 16,139 cases with
complete contradiction checks and no findings in 1,120.42s.

### Item 40 — who attests a scarcity comparison, 2026-09-24

The revision plan read Chapter 6 as letting the manager write the comparison it
is then reviewed on. The source disproves that premise: every allocation field,
the comparison outcome included, must be attested identically by a source, an
evidence and a review writer, each distinct from the manager, and anyone may
request review of a finding, allocation or shortfall without the manager's
permission. Chapter 6 now names the three attesters and says a disagreement
between their records defeats the allocation; the scarcity contract records the
disproved premise.

The review found a narrower gap it did not name: nothing required a claimant's
own evidence to reach the comparison. The allocation now carries a field,
`EachComparedClaimantsSubmittedEvidenceConsideredBeforeTheComparison` at the
single-valued `ScarcityClaimantEvidenceScope`, that all three attesters must
record, and the generated omission case withholds the allocation without it.
Evidence the requester submits with a later challenge obliges the reader to
weigh it (`WeighTheClaimantsSubmittedEvidenceAgainstTheComparison`). An entry
written by somebody who did not make the request creates no such duty, and the
duty leaves the allocation's authority intact. The worked challenge case and
Chapter 6's pins carry both steps. Neither conclusion establishes that the
evidence was persuasive or that anything was delivered.

Chapter 6's pins also asked whether the alternate reviewer owed a duty under the
name item 08 retired, so that FALSE pin could never fail. It now queries the
current `ReviewScarcityRequestSecureTheMinimumAndEscalateCourtRemedy`, which the
generated nonresponse case shows turning TRUE on certification. The exact prose
is `session-drafted, author-approved under delegated approval (2026-09-13)`.

Rules move from 7,412 to 7,414 and the inventory gains one case. The 403
scarcity and Chapter 6 cases pass 1,718 pins in 30.40s (focused, partial); the
reference, claim-discipline, reader-coverage, receipt and seventeen floor-vector
tests pass, and the prose check passes after one new sentence was reworded to
hold the chapter's figures. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,882
pins across 16,139 cases with complete contradiction checks and no findings in
1,181.35s.

### Item 39 — confinement guards and a lapse that is not an extension, 2026-09-24

No rule reads a missing home or family entry, but the floor firewall covers
floor items only, so a new rule confining someone for lacking a home record
would load while Chapter 28 said such grounds "cannot justify harsher
confinement". The development test
`no_confinement_reads_an_absent_home_family_or_work_entry` now fails if any rule
concluding custody (`prisoner`), placement (`fit`, `dwell`, `building`),
severity or a restraint reads an absent `home`, `family`, `parent`, `married`,
`sibling` or `work` entry, with three planted controls; no current rule does. It
checks written rule forms, and Chapter 28 now states exactly what is checked.

When a custody renewal is not made, confinement under that case loses its
authority, even after a conviction for grave injury, and nothing extends it. The
new `custody/authority-lapse` case, the severe placement fixture without its
current renewal, pins twelve results: the merits stand, custody authority and
confinement are absent, nothing manufactures release, the release-review duty
ends with current custody, a person still held is owed humane care, voice and
independent holding review through the independent holding report, and the
renewal made in time restores custody. Two rules owe the review that prevents an
administrative lapse: justice and appeal (`FSBOD_17`) must review each current
custody authority before its source-bound end, and a certified nonresponse moves
the review to the predeclared alternate constitutional panel (`FSBOD_25`).
Neither is a renewal. Chapter 29 states the consequence and Part V who bears it;
the time-model and public-safety decisions record both changes, and the exact
prose is `session-drafted, author-approved under delegated approval
(2026-09-13)`.

Rules move from 7,410 to 7,412. The lapse case passes twelve pins in 11.13s
(focused, partial); seventeen floor-vector tests and the reference,
claim-discipline, reader-coverage and receipt tests pass, and the prose check
passes after one Chapter 28 sentence was reworded to hold its negation figure.
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,873 pins across 16,138 cases with
complete contradiction checks and no findings in 1,182.16s.

### Item 38 — failure at scale, 2026-09-24

The chain after an ignored claim ends at the alternate in a recorded duty nobody
has discharged (item 08), which answers one claim, not a tier under-delivering
at scale. Six rules in Article 1b now answer failure at scale without counting
anything or finding anything about a person. A source and a separate independent
reviewer, neither of them the failing tier, must certify the same publicly
answerable tier (`authority($tier)`); the Constitutional Court (`FSBOD_18`) then
owes a decision on whether the pattern is an unconstitutional state of affairs.
Its declaration obliges the People's Assembly (`FSBOD_02`) to adopt a public
plan with reasons, deadlines and a source-bound end, the tier to carry it out,
and integrity and audit (`FSBOD_19`) to monitor it. A certified Assembly
nonresponse obliges the Court to order interim measures that secure the minimum
without administering the service, and provision and treasury (`FSBOD_07`) to
give the minimum the first claim on funds. Each rule states the certification in
full: an intermediate `related` conclusion reading `authority` joined the cycle
through `contradict` and was refused at load, while `obliged` is read by
nothing. The precedents the plan cites wait for item 46.

Chapter 7 gains fifteen pins (45) covering the whole sequence, a tier's
self-certification, a lone attester, a declaration without certification, a
pattern naming a private person, and care that stays undelivered. Chapter 7's
new section "When failure is general", a pointer from Chapter 30's bounded chain
and a Part V paragraph comparing the alternatives describe the route; the exact
prose is `session-drafted, author-approved under delegated approval
(2026-09-13)`. The coverage ledger reaches 242 passages, and the obligations
decision records the choice. Rules move from 7,404 to 7,410. Chapter 7's 45 pins
pass in 12.28s (focused, partial); the reference, claim-discipline,
reader-coverage, receipt and floor-vector development tests and the prose check
pass. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,861 pins across 16,137 cases
with complete contradiction checks and no findings in 1,178.59s.

### Item 37 — the shield and a disclosure made after the charge, 2026-09-24

Any defendant could disclose against the prosecuting court after the charge and
hold confinement until qualified reviewers found the prosecution unrelated; a
later deceit finding removed only that disclosure's protection. The repair
follows causation: a prosecution can answer only for what came before it.
Conviction route 1 now reads a two-place shield, `defend($w, $case)`, which
derives for each case in which the maker is cited unless both of the case's
temporal witnesses, `Chronicle` and `TemporalReview`, record that the disclosure
was made after that case's charge. Two rules, one per witness, express "not
both" over base facts. The one-place shield, route 2 and every existing verdict
are unchanged; the `temporal-T2` base's replacement of the court gate follows
the new text. Automatic pre-charge protection with an expedited review for later
disclosures, a specificity threshold and a costed universal review stage were
compared and rejected in the shield-scope decision, which names the protection
traded as the 2026-08-02 exposure-surface ruling requires: retaliation within a
pending case is answered by the custody requirements, and protection for an
earlier disclosure depends on the case's witnesses dating it honestly, both of
them, with a missing entry keeping it.

Chapter 24 gains eight pins (59): one witness's entry leaves protection in
place; both restore confinement for a disclosure against a recalled official and
for one against the prosecuting court; the one-place shield and a later case
keep the disclosure's protection; and a second, undated disclosure blocks again.
The credibility-voiding census now lists the two new rules, which read an
incident finding exactly as the one-place shield does. Chapter 24's opening, a
new section "A disclosure made after the charge", its cost paragraph and Part
V's shield argument describe the rule; the exact prose is `session-drafted,
author-approved under delegated approval (2026-09-13)`.

Chapter 24's two cases pass 118 pins, and the shield-independence, credibility,
red-team, counterfactual and `temporal-T2` cases pass 578 pins (focused,
partial). All authoring development tests pass once four stale resolution
receipts, left by the 2026-09-21 prose revisions, were rebound in a separate
commit. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,846 pins across 16,137
cases with complete contradiction checks and no findings in 1,207.62s.

### Item 36 — the floor's bearer by tier, and one name per item, 2026-09-24

Article 1b named only a bearer: "the public body named State" owed every floor
item, while Chapter 7 requires each duty to have a bearer, a function, a
jurisdiction and a scope. Five hand-written rules now divide the work, applying
the economic ruling's allocation of floor finance, equalisation, portability and
minimum standards to the common tier and of provision to regions and localities.
Every person is owed two common-tier duties, to finance, equalise and set
minimum standards and to backstop the floor, on personhood alone. A region owes
provision, and a locality reach and delivery, where a witness authorised for the
person records the person as present there. A failure certified identically by a
source and a separate independent reviewer, neither of them the failed body,
obliges the tier the certification names to assume continuity; the failed body's
own duty stands. The eight `owe(State, K, $x)` debts are unchanged and unread,
nothing reads the new duties, and none conditions the floor: a person no witness
has recorded keeps every entitlement, debt and common-tier duty.

Chapter 4's pins gain fifteen: the common-tier duties for a fresh person and for
the child's one-line record; regional and local duties from an authorised
presence record, and none from an unauthorised writer or from no record; no
delivery conclusion from presence; continuity passing up a tier on a two-party
certification but not on self-certification or a lone attester; and the refusal
of a rule confining a person for lacking the common tier's duty. The engine
reports another edge of the same cycle, so that pin matches the stratification
refusal itself. Chapters 4 and 7 describe the allocation. The floor's names now
read care and shelter where "health debt", "health entitlement", "dwelling
debt", "shelter debts" and "housing entitlement" stood; "personal-care debt" in
Chapters 1 and 14 names a private obligation kinship does not create, and stays.
The security split is item 44. The exact prose is `session-drafted,
author-approved under delegated approval (2026-09-13)`; the economic decision
records the allocation.

Rules move from 7,397 to 7,402; predicates, derived predicates and strata are
unchanged. Chapter 4's 85 pins pass in 21.55s (focused, partial). The sixteen
floor-vector, six reference, five claim-discipline and ten reader-coverage
development tests pass, and the prose check passes with lower recorded figures
for five chapters. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,830 pins across
16,137 cases with complete contradiction checks and no findings in 1,197.58s.

### Item 35 — the companion's CI, 2026-09-24

Every Book UI workflow run on `main` had failed since the companion landed. The
web job's formatting step ran `cargo fmt --all`, which follows path dependencies
into the adjacent Nibli checkout and stopped at its generated, ignored
`bindings.rs`; it now formats this repository's own packages, `rights-book-ui`
and `book-reason`, which were already formatted. The Windows desktop job failed
in `ui/scripts/prepare.py` with a `charmap` UnicodeDecodeError because bare
`read_text()` and `write_text()` use the platform encoding; every file read and
write in `ui/scripts/` and `ui/tests/` now names UTF-8. Locally `prepare.py`
runs with `EncodingWarning` raised as an error and writes a byte-identical
`cases.json`, and the UI input tests pass the same way. Run 36000105151 on
`5c0b10c9` passes all three jobs, web, Windows desktop and Ubuntu desktop, with
no check removed. No formal input changed, so the item 34 complete run on the
same source stands for the verifier.

### Item 34 — prose measurement as a development check, 2026-09-24

`tools/prose_lint.py` adapts the revision plan's lint, moved there from
`new-reviwes/`, into a development measurement of every ordered input the
manifest names. It reports negations and terms of art per 1,000 words, distinct
case names, sentences saying something is not established, terms the plan keeps
out of chapter prose, and harness names, at the file's own line numbers. Four
defects in the plan's script are repaired. Stripping fenced code shifted every
later line number, so all 63 findings in the method pointed at the wrong line;
link targets were scanned as prose, so 33 banned-term hits lived only in URLs;
the name list omitted the institutional fixtures the plan's own counts include;
and the term patterns missed inflections. Comments, inline code and bare URLs
are blanked in place, and only tokens with a letter or digit count as words.

Profiles follow the element. Chapters, derived and Part V alike, meet the plan's
thresholds; the opening meets them except for case names, which wait for its
reference material to move; the method is measured without limits because the
plan allows its terms there. Each input is held to the plan's threshold or its
figure recorded in `tools/prose_lint_baseline.json`, whichever is looser.
`--ratchet` only lowers a figure, and a restructure admits a new file explicitly
with `--admit`. It is a development check and adds no gate to `verify.sh`.

On the current manuscript it reproduces the plan's appendix: word counts within
three words, negation within 0.1 per 1,000 words, and case names exactly for all
30 derived chapters, including Chapter 25's 21 and Chapter 9's 14. Terms of art
read up to 1.7 per 1,000 higher where plurals now count. Part V and the method
differ because the plan excluded Part V's notes and measured the method with its
code. Every ordered input fails the plan's negation threshold today; the
baseline records where each starts.

Fourteen unit tests pass (`python3 -m unittest discover -s tests -p
test_prose_lint.py`, about one second): every check fires on a planted example
and stays silent on clean text, line numbers survive code, the ratchet only
lowers, every input has exactly one record, and no input regresses. A banned
term planted in Chapter 19 failed the manuscript test and was removed. The
contribution guide lists the commands. `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes
88,815 pins across 16,137 cases with complete contradiction checks and no
findings in 1,228.64s; the constitution, pins and chapters are unchanged.

### The revision rulings D1–D9 — 2026-09-24

The author ratified nine rulings and two follow-ups on 2026-09-24, answering the
questions the [revision tracker](TODO.md) reserved after the outside [revision
plan](new-reviwes/revision-plan-9.5.md). Another AI assistant wrote that plan
from the 170-page review PDF built at `93fa5662`; the rulings are the author's,
taken in one sitting. **Each is ratified but unimplemented** in the standing
sense: it creates no predicate, rule, pin, chapter or built artefact, and the
edition's text stays as it is until the implementing TODO item lands and adds
its dated supersession note here. Each ruling supersedes only what it names, and
each superseded passage below points back to this subsection.

**D1 — subtitle, reader and promise.** The subtitle becomes *A worked design for
a society, with its formal claims made executable.* The primary reader is the
serious non-specialist, with lawyers and policymakers second. The first page and
back cover carry: *"A constitution designed from the person with nothing, argued
in plain language, with every rule published so you can test it."* This
supersedes the subtitle in the title ruling, including its "catches its own
failures" clause; that ruling's warning about dropping the clause was put to the
author, who chose the new subtitle knowing it. The title is unchanged. Items 60
and 64 implement it.

**D2 — an argument section in every derived chapter.** Each derived chapter
keeps its pinned, flat account and then ends with one labelled, first-person
argument section: the reason for the rule, the strongest alternative with its
evidence, and what would change the choice. Part V becomes a synthesis of how
the choices fit together. This supersedes, for argument only, the voice
boundary's limit of the author's first person to three elements, the statements
that exactly three elements are exempt, and the register decision's closure of
derived chapters to non-derived content. The flat register of the derived
sections, the ban on inner lives, the refusal of composite citizens and
dramatised scenes, and the rule that argument creates no right, power or
exception all stand. Derived sections stay exactly as derived as before. The
length invariant and the digit rule come to be measured by section rather than
by file once item 53 builds that tooling. Items 53–59 and 63 implement it.

**D3 — documented cases outside Part V.** Documented, registry-sourced cases may
appear in the opening (Santoshi Kumari first, with the dispute over her death
intact), in a short labelled case opening each Part, and in argument sections.
They never appear in a derived section, and no case gains an invented inner
life. This supersedes the rule that the historical cases stay in Part V. Items
46, 47, 54–57 and 60 implement it.

**D4 — the constitution in plain language, in the companion.** Numbered
plain-language articles are published in the companion, not printed in the book,
and chapters cite article numbers. Articles trace to the rule families,
contracts and pins that implement them — by family, because most rules sit in
generated blocks with no article banner. The method stays the book's technical
appendix within its sealed scope. An interpretation article is added. Its burden
half (a restrictive conclusion needs complete positive evidence, and absence
never extends a power) traces to existing fail-closed rules and their pins. Its
protective-reading half (where a standard is open, the reading more protective
of the floor and of liberty prevails) is a declared non-formal article, listed
as a deliberate traceability gap. R4 stands, as does the rule that the formal
source lives beside the ordered inputs. Items 61 and 62 implement it.

**D5 — the child returns where the result differs.** R1's criterion becomes
"produces a different or instructive result" instead of "can run the rule".
Chapter 1 stays the case. Each removed section is recorded in
`CHILD_SLOT_EXEMPT` with its reason, so the membership test still decides. Items
52 and 54–57 implement it.

**D6 — fast for help, slow for harm.** An act that only gives or preserves
something for its subject — immediate care continuity, an accommodation, interim
protection — takes effect on one authorised actor, and prompt independent review
can correct it. Any appointment giving someone power over another person, and
every adverse act, keep full prior procedure; delivery evidence keeps its
independent witness. This supersedes, for that class only, the 2026-09-15
symmetry under which every record completion carries a distinctness constraint
and an independent review authority before it takes effect. Item 42 measures the
load first; item 43 implements the ruling and revises
`every_power_record_is_independently_reviewed` into the ruled property. **Implemented 2026-09-24 (item 43):** the
single-actor route concludes a beneficial record's duties and permissions, never
its completion, so the completion symmetry still holds and
`help_takes_effect_on_one_actor_and_harm_waits_for_review` states the rest.

**D7 — the structure.** The revision plan's table is adopted: Chapters 1 and 2,
9 and 10, and 23, 25 and 26 merge; Chapter 13 splits into commons and animals;
"An Ordinary Week" joins Part II and "Where This Could Fail" joins Part V; and
the map, glossary, roles and subject index move to the back matter beside the
method, so that at most five pages precede Chapter 1. R3's ordering rule stands.
This supersedes the reading-order decision's refusal to merge Voiding with
Clawback: its mechanical reason, the unscoped `:accept` closing chapter 25's
pins, is met by ordering the merged pins so that it comes last. Item 53
implements it, with the manifest, relocation and test changes it needs.

**D8 — nine floor items.** `secure` splits into bodily safety, a protection
guarantee with no receipt route, and material security, essential goods keeping
the current receipt route. Care keeps `healthy` and is called care throughout;
company is unchanged. This supersedes the eight-item floor, and the split still
needs the completed coverage-map contract, a refusal pin for every item and
formal proof. Item 44 implements it. **Implemented 2026-09-24 (item 44):**
`secure` is bodily safety with no receipt route, `suffice` is material security
on the receipt route, and every item has its entitlement, debt and refusal
pinned.

**D9 — the animal core as enacted.** The unamendable core stays as enacted:
direct protected-subject status, the ban on severe avoidable suffering, and the
ban on killing solely for the closed list of dispensable purposes. The
alternative-sensitive food rule stays ordinary amendable constitutional law,
bounded by the core, so a food purpose never licenses avoidable suffering. Item
58 defines "direct protection", "avoidable" and "dispensable" accordingly and
argues the result, including its implications for farming, subsistence and
culture. Nothing is superseded.

Controlling records: the dated 2026-09-24 section in
`book-1/appendix/decisions/narrative-register-decision.md` (D2, D3),
`child-with-nobody-decision.md` (D5), `reading-order-and-appendix-decision.md`
(D1, D4, D7), `delivery-and-receipt-decision.md` with
`book-1/appendix/maps/constitutional-taxonomy.md` (D8), and the ecological
decision (D9). D6 is recorded here and moves into the affected contract cards
when item 43 lands.

### Item 33 — final sequential assessment and shield precision, 2026-09-21

The fresh [manuscript review](reviews/2026-09-21-final-manuscript-review.md)
reads all 34 ordered inputs after items 21–32, starting at `ee588deb`, and
rates the resulting manuscript 9/10. It separately assesses argument,
precision, readability, pacing, evidence and navigation, with a rating and
assessment for every input. It is an internal AI-assisted judgment, not a
measurement, independent endorsement or required perfect score.

One material overstatement was confirmed against the current source. Article
6's shield guards conclude case-specific confinement and require a recorded
conviction; they do not prevent the conviction from being recorded. The
existing Rex sequence already retains `match(Rex, ConvictionRecorded)` while
custody is blocked. Chapter 24, its Part V argument and the optional method
now describe confinement under the conviction. The opening glossary already
had that scope. The exact revised prose is
`session-drafted, author-approved under delegated approval (2026-09-13)`;
the existing shield and narrative-register decisions retain the correction
and canonical text locations. The existing coverage entry and generated
report are aligned. No constitutional rule or executable expectation changes;
the chapter pin edit corrects only its explanatory comment.

The review rechecks recognition, severity, scarcity, delegated discretion,
capture and the difference between formal consequences and justified design.
The existing repairs remain supported by the inspected rules and cases.
An apparent standing conflict is disproved: seating derives `authority`, not
the institutional `public` entry excluded by the birth rule. No additional
material design defect is established, no unresolved defect is relabelled a
Book 2 operating problem, and no new impossibility claim is made. Density,
uneven pacing and the evidence's limited predictive force keep the editorial
assessment below 10/10.

The focused shield command passes 102 pins across two selected cases in
12.51s. Five claim-discipline tests pass in 0.84s, ten reader-coverage tests
in 0.27s, six reference-integrity tests in 1.61s and seven assembler tests
in 0.235s. The first reference run caught the link to the not-yet-created
review; creating its actual target resolves it without an exception. The
required child slots and exemptions, one-line fixture, 240 coverage entries,
manifest order, chapter/pin pairs and exempt prose channels remain intact.

The full book and sample are rebuilt as HTML, EPUB and PDF. Current rendered
word counts are 57,909 full, 40,481 derived (69.90%) and 13,633 sample;
Markdown excluding HTML comments totals 58,418. The PDFs remain 169 and 35
pages. Both EPUBs validate with zero errors or warnings. All 90 browser
views, PDF text and destination checks, and the inspected affected layouts
pass as detailed in the review. Rendering uses the same installed browser as
item 32 and adds no new builder, test gate, receipt or inventory. The proposal
and build README follow the current measurements. The reading artifacts stay
in ignored `output/book-1/`; no publisher contact or release is performed.

`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,815 pins across 16,137 cases,
with complete contradiction checks and no findings, in 1,188.54 seconds
(19m 48.54s). No required check is incomplete; the five-minute target remains
unmet. The run overlaps some artifact checks and supplies no new CPU or
peak-memory measurement. The final documentation and diff checks pass.
Item 33 is complete and removed; the ordered revision backlog has no pending
items. Gate C and Book 2 activation remain separate. The unrelated untracked
`book-1/book-1.zip` is untouched.

### Item 32 — rebuilt reading copies and reassessed sample, 2026-09-21

The existing builder rebuilds the full 34-input book and the five-chapter
sample as HTML, EPUB and PDF in ignored `output/book-1/`. No manuscript,
formal source, pin, suite, manifest, builder or stylesheet changes were
needed. The book README and publisher proposal now use the actual lengths
and current Chapter 26 title, *The Limits of a Finding*. Their exact revised
prose is `session-drafted, author-approved under delegated approval
(2026-09-13)`.

The rendered manuscript contains 57,838 whitespace-separated words, including
notes before the generated cover and contents. Derived chapters contribute
40,448 (69.93%); the sample contains 13,597. The separate Markdown measure
remains 58,347 words excluding HTML comments. The full PDF has 169 pages and
the sample 35. Chapter order, pin pairs, the one-entry child fixture, required
child slots, three exempt prose channels and majority-derived rule remain.

The reassessed sample retains complete Chapters 1, 5, 8, 21 and 31: standing,
delivery, ordinary freedom, access to justice and the extended argument.
Chapter 31 now supplies about 71% of the sample and 17% of the manuscript.
The proposal states that emphasis and its purpose: an editor can examine the
argument and its qualifications together. It does not present the selection
as proportional to the book or as satisfying an unspecified press's limits.
The September 20 review is identified as an earlier assessment; item 33's
fresh review remains pending. Preparation does not initiate publisher contact,
submission, release, Gate C or Book 2 work.

Seven existing assembler development tests pass in 0.423s. Six existing
reference-integrity tests pass in 1.57s. The 23 local links in the changed
documentation resolve, and all 31 proposal chapter titles match the manifest.
No new test, verification gate, receipt or inventory is introduced.

Rendering used the builder's declared Playwright 1.63.0 and existing Chromium
146.0.7680.80 through `--browser-executable`. Initial dependency retrieval
failed inside the sandbox; the authorised retry supplied the pinned packages.
The default browser build was absent, so the documented executable override
used the existing browser. Neither initial attempt is counted as a successful
PDF build. A temporary inspection selector also needed valid CSS syntax for
numeric-leading IDs; this was an inspection-helper error, not a book defect.

The 90 browser renders cover both HTML copies and every packaged EPUB XHTML
document at 360px and 1280px. There is no document overflow, clipped code,
missing local fragment, duplicate ID or unlabelled table. All nine full-book
tables and the sample's table have labelled regions and scoped headers.
All 16 note references and returns work in each HTML copy, and the keyboard
skip link reaches the main content. Opening navigation and EPUB contents
and method links were activated at both widths. The Tamil font loads.
EPUBCheck 5.4.0 reports zero errors and warnings for both packages under
EPUB 3.4 rules.

The full/sample PDFs contain 297/46 bookmarks, 228/37 valid internal link
destinations and 82/54 external links. Both retain tagged structure and
embedded prose/font licences. All pages were checked for empty output and
out-of-page text and visually inspected as contact sheets. Alphanumeric text
comparison found all 1,487/257 checked source blocks; the Tamil source text
also occurs exactly after whitespace removal. Selected contents, epigraph,
tables, notes, diagrams, case index and method code were inspected at reading
scale, with phone and desktop views where applicable. No material rendering
defect was found. EPUB inspection covers its packaged XHTML in Chromium and
the validator, without claiming every ebook application, actual-user
accessibility, independent reader testing or editorial endorsement.

`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,815 pins across 16,137 cases
in 1,133.59 seconds, with complete contradiction checks and no findings.
No check is incomplete; the five-minute target remains unmet. The source
and substantive inventory are unchanged. The final diff check passes, and
the untracked `book-1/book-1.zip` is left untouched. Item 32 is complete.

### Item 31 — evidence, attribution and inference, 2026-09-21

The source review corrects Part V and 21 existing registry entries under the
standing delegated approval. The exact canonical Part V is session-drafted
and author-approved under delegated approval (2026-09-13), recorded in the
existing narrative-register decision. The epigraph's Tamil wording and approved
English rendering remain unchanged; its poem, stanza and life-year attribution
were checked. The other ordered inputs introduce no additional empirical
study or population statistic requiring a source correction.

The Jharkhand cancellation count is withdrawn because it was not located in
the cited paper. The paper's mass-cancellation finding remains, with the
Postscript's precise locator. Its conditional transaction-failure extrapolation
cannot substitute for a card-cancellation count. The separate beneficiary
estimate remains attributed to the published research abstract, with its
qualification about transition management. Santoshi Kumari's death stays
disputed: contemporary family and activist reporting is now paired with a
report quoting the deputy commissioner's malaria explanation. The previously
cited follow-up did not repeat that dispute.

The Auroville passage specifies the power to constitute implementation
committees while retaining the distinction from master-plan consultation.
Owen's valuation mechanism and Cybersyn's existing network receive inspected
passage references. Tanzania's relocation and village-resident populations
remain separate. MONDRAGON's reports are identified as institutional accounts,
with workforce distinct from membership; EPI's projected compensation ratio
has a different denominator, period and population. History supplies no
isolated governance effect or demonstrated operation of this constitution.

The unchanged V-Dem snapshot reproduces its correlations, category summaries
and exploratory residual regression. The data README and footnotes now state
the survey window, scale, GDP unit, equal country weights and differing year
labels. Reproduction does not advance the snapshot's retrieval date. The
unbundled alternative-index comparison was not reproduced and is removed from
the reader argument; its historical record and licensing boundary remain.
The methodological distinction between comparing significance labels and
testing a difference remains, without implying a freshly executed comparison.

The registry and decision state the actual source-checking scope. Lin,
Muralidharan and Stodder were checked through published abstracts, not newly
inspected full empirical methods. Accessible original reports, articles,
legal texts, testimony and historical passages supply the other stated
checks. Full-text retrieval failures and background monographs not newly
read in full remain explicit. No unavailable study is labelled replicated,
and no source locator is treated as proof of the claim.

All five claim-discipline, six reference-integrity and ten reader-coverage
development checks pass in 0.74s, 1.30s and 0.20s respectively, following a
65-second incremental compilation. Four binding phrases follow corrected
prose; all 24 bindings remain. `git diff --check` passes. A direct `rustfmt`
check reports seven pre-existing differences in the touched test file; the
committed baseline produces the same seven differences. No formatting sweep
or exception expansion is included.

`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes 88,815 pins across 16,137 cases;
contradiction checks complete with no findings in 1,146.40 seconds. No check
is incomplete. The five-minute target remains unmet. The constitution,
substantive pins and suite inventory are unchanged.

The manuscript measures 58,347 whitespace-separated words excluding HTML
comments across 34 ordered inputs; derived chapters contain 40,763 words
(69.86%). The opening and optional method retain 3,943 and 3,936 words.
Chapter order, pin pairs, the one-entry child fixture, required child slots,
three exempt channels and carried appendix remain. Reading-copy inspection
and final assessment are separate pending items. No reader study, independent
endorsement, release or Gate C completion is claimed. Book 2 stays
collection-only.

### Item 30 — sequential developmental and line edit, 2026-09-21

All 34 ordered inputs received a fresh sequential read after the targeted
revisions: epigraph, opening, Chapters 1–31 and optional method. The review
checked purpose, context, worked consequences, progression, terminology and
endings. Twenty numbered chapters and the opening receive local edits; the
remaining thirteen inputs retain their wording where no concrete improvement
was needed. The existing narrative-register decision identifies the files and
records their exact canonical prose as session-drafted and author-approved
under delegated approval (2026-09-13).

The material scope correction is in Chapter 19. The enforcement firewall
prohibits using service access to enrol someone into an enforcement record;
it does not exclude lawful service-purpose records generally. The correction
follows the existing public-safety decision, record-power ruling and holding
contract's `RecordEnforcementFirewallScope`, without changing their policy or
formal implementation. Chapter 3 defines voiding on first sustained use;
Chapter 5 distinguishes the custody records from Marisol's added receipts and
explains the teaching-and-hearing rule's distinct evidence requirements.
Chapter 12 identifies the comparison subject, Chapter 14 separates overloaded
participation claims, and Chapters 18 and 21 keep Nell's right to be heard
distinct from recorded speech, a request or a hearing.

Other edits replace unexplained implementation terms with their meaning,
introduce the custody lease before following it, and put Chapter 30's bridge
to Part V after its final child case. Part V's five joints retain the sustained
arguments for the revised choices, their alternatives, cost bearers and
grounds for reconsideration; two local edits clarify a pronoun and amendment
terminology. Its hypothetical kitchen and ending remain. The optional method's
exact examples, limits and syntax remain unchanged. No case becomes biography,
no missing entry becomes evidence of an outside event, and no revision history
enters the reading sequence.

Four stale prose locators identified by the initial reference check are
updated in the existing full-society ledger and placement record. Historical
assurance judgments are not renewed and no exception is widened. The
reader-coverage generator retains the same 240 passages and unchanged report.
All six reference-integrity, ten reader-coverage and five claim-discipline
development checks pass, as does `git diff --check`.

Focused `./verify.sh --only` checks pass Chapter 3's 45 pins in 12.12s,
Chapter 19's 20 pins in 11.78s and Chapter 28's 73 pins in 12.61s. The complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across 16,137 cases**,
with complete contradiction checks and no findings, in **1,152.03s**. The
five-minute target remains unmet. The constitution, substantive pins and
suite inventory are unchanged throughout the run. Item 30 is complete.

The manuscript measures 58,090 whitespace-separated words excluding HTML
comments. Derived chapters contain 40,763 words (70.17%); the opening contains
3,943 and the method 3,936. Chapter order, pin pairs, the one-entry child
fixture, required child slots and exemptions, three exempt prose channels and
carried appendix remain. Evidence checking and reading-copy inspection retain
their separate pending items. No independent reader study, release or Gate C
completion is claimed. Book 2 stays collection-only.

### Item 29 — necessary qualifications and distinctive child returns, 2026-09-21

Under the standing delegated approval, 24 numbered chapters now keep each
qualification beside the claim it limits and remove repeated summaries that
add no further boundary. All numbered chapters received the review. Chapters
10, 11, 12, 18, 22, 28 and Part V retain their text because their qualifications
limit distinct claims. The separate prose comparison preserves the difference
between supplied evidence and truth, duties and performance, lawful authority
and execution, and absent evidence and proven failure. The cuts remove 1,096
words without using a phrase count as a quota or verification gate.

The child returns state their particular consequence. Nell's standing and
floor need no request, service file or caregiver gate. The Chapter 5 food
receipt and Chapter 27 shelter and food evidence remain added facts rather
than conclusions from birth. Chapter 7's bystander observation opens review;
it proves no delivery or breach. Chapter 6 supplies no evidence that Nell
belongs to its scarcity population. Chapter 19 does not invent an access
request or reviewer's nonresponse. Chapter 20's negative restraint result
grants no general immunity from separately qualified measures. Chapter 30
retains the silent positive-evidence alarms and the accepted counterfactual
inference from missing receipt, without treating acceptance as proof of
deprivation. Explanation and review following an adverse finding remain
conditional on that finding. No child fact, inner life or outside success is
invented, and the five child-section exemptions remain unchanged.

The existing narrative-register decision records the exact canonical text as
session-drafted and author-approved under delegated approval (2026-09-13).
Six existing reader-coverage descriptions now name the precise baseline or
additional evidence; the regenerated report retains all 240 passage IDs and
classifications. One full-society locator follows the retained custody rule
after its repeated summary was cut. An initial reference-integrity failure
identified that stale locator; correcting its owning record resolves the
failure without extending an exception or renewing historical assurance.

Focused `./verify.sh --only` checks pass Chapter 5's 16 pins in 12.12s and
Chapter 30's 37 pins in 13.16s. All six reference-integrity, ten reader-coverage
and five claim-discipline development checks pass, as does `git diff --check`.
The complete `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across
16,137 cases**, with complete contradiction checks and no findings, in
**1,123.37s**. The five-minute target remains unmet. The constitution,
substantive pins and suite inventory are unchanged throughout the run.
Item 29 is complete.

The manuscript measures 58,020 whitespace-separated words excluding HTML
comments across 34 ordered inputs. Derived chapters contain 40,699 words
(70.15%); opening and method remain 3,938 and 3,936 words. Chapter order,
pin pairs, required child slots and exemptions, the three exempt channels
and the carried-appendix boundary remain. This is editorial assessment, not
reader testing or a released edition. Book 2 stays collection-only until Gate C.

### Item 28 — governing questions and case progression, 2026-09-21

Under the standing delegated approval, Chapters 7, 9, 11, 13, 17, 19, 20, 29
and 30 now lead with a governing question and organise their supplied cases
around the consequences readers need to distinguish. Chapter 7 begins with
the waiting scarcity claimant and follows responsibility into review, repair
and delivery evidence. Chapter 11 keeps an intervention's end beside its
authority before turning to public finance. Chapter 13 introduces human
environmental rights, commons and future conditions, and animal interests
before their offices and remedies. The water case and the later disagreement
between offices retain their separate permissions and limits.

Chapter 17 follows the existing ordinary-bill records through a bounded
Council return, its missing-condition control and Assembly repassage under
the same rule. It explicitly distinguishes these supplied stages from an
actual legislative event; fiscal, executive and judicial authorities remain
separate. Public-result requirements precede the later institutional uses.
Chapter 19 begins with the complete holding and dependent uses before the
defect sequence. Chapter 20 keeps each authorisation with its withdrawal and
separates treaties, trade and lawful exit. Chapter 29 keeps the sentence,
current authority, duties and release together, then distinguishes physical
holding and other protective powers. Chapter 30 proceeds from reported
placement and evidenced conditions to review, nonresponse and withdrawal.
Chapter 9 already had a coherent case progression after item 27 and needed
only a clearer question and transitions; the review does not manufacture a
larger defect there.

The separate prose comparison retains every substantive protection and
qualification through the moves. No dialogue, biography, outside event or
successful operation is invented. The existing narrative-register decision
records the exact canonical text as session-drafted and author-approved under
the delegated 2026-09-13 instruction. The reader-coverage source and generated
report follow the new order with existing IDs preserved; two newly separated
sections bring the total to 240 passages. Two existing full-society locators
follow the renamed environmental heading. No historical assurance judgment
is renewed and no new verification gate is added.

Focused `./verify.sh --only` checks pass state-form main-57's seven pins in
12.90s and main-58's six pins in 12.63s, including the bill stages and the
missing-return control. All five claim-discipline, six reference-integrity
and ten reader-coverage development checks pass; the latter two pass again
after the final heading and locator update. The complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across 16,137 cases**,
with complete contradiction checks and no findings, in **1,175.87s**. The
five-minute target remains unmet. The constitution, substantive pins and
suite inventory are unchanged throughout the run. Item 28 is complete.

The manuscript measures 59,116 whitespace-separated words excluding HTML
comments across 34 ordered inputs. Derived chapters contain 41,795 words
(70.70%); opening and method remain 3,938 and 3,936 words. Chapter order,
pin pairs, required child slots and exemptions, the three exempt channels
and the carried-appendix boundary remain. This is editorial assessment, not
reader testing or a released edition. Book 2 stays collection-only until Gate C.

### Item 27 — identifiable cases and explicit roles, 2026-09-21

Under the standing delegated approval, the opening's optional case index and
Chapters 9, 10, 23–26 and 30 identify people and bodies by their role in each
example. The earnings claims remain separate; Bela's claims have their own
records. The appointment and voiding cases identify the subject, signers and
reviewer, with Nogra, Nogrb, Targ4 and Partnr retained as executable names.
The shield headings state the distinction made by each defendant's case.
Steps in one test are distinguished from separate examples, and the opening
warns against assembling isolated cases into a biography. The existing
narrative-register decision records the exact approved canonical prose.

Reviewing the recurring cases found a stale premise in Chapter 30: Ruk is
eligible for consideration of home confinement, but the supplied authority
names secure placement. The alarm compares the placement report with that
authority. The corrected explanation follows the current constitution and
Chapter 28's existing positive home-eligibility and conflicting-report pins.
There is no new policy or formal repair in this item. The source, pins,
identifiers and suite inventory are unchanged.

Two distinct prose locators, occurring three times, are updated in the
existing full-society ledger. The reader-coverage JSON and generated report
follow four renamed shield headings and correct the case descriptions that
reversed Don/Pax roles, implied actual voting, or misstated the scope of a
credibility finding. The placement entry now names the actual authority
comparison and its executing Chapter 28 pins. All 238 passage classifications
remain. These updates renew no historical assurance verdict and add no
inventory or verification gate.

Focused checks pass 56 Chapter 9 pins (12.93s), 34 Chapter 10 pins (12.66s),
102 Chapter 24 pins across two cases (18.10s), 198 Chapter 25 pins across two
cases (24.13s), 73 Chapter 28 pins (12.37s) and 37 Chapter 30 pins (13.54s).
All five claim-discipline, six reference-integrity and ten reader-coverage
checks pass. Initial reference and coverage failures identified the old
locators and headings; the existing records are updated and both suites pass
on recheck. The complete `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes
**88,815 pins across 16,137 cases**, with complete contradiction checks and
no findings, in **1,182.24s**. The five-minute target remains unmet. All formal
inputs remained unchanged throughout the run. Item 27 is complete.

The separate prose review measures 58,359 words across 34 ordered inputs,
excluding HTML comments and splitting on whitespace. The derived chapters
contain 41,038 words (70.32%); the opening is 3,938 words. The early Chapter 1
link and required opening sequence remain. No biography, testimony or outside
success is supplied, and this editorial review is not a reader study. The
chapter order, three exempt channels, required child tests and their exemptions
remain; Book 2 stays collection-only until Gate C.

### Item 26 — reader consequences and optional implementation detail, 2026-09-21

Under the standing delegated approval, Chapters 3, 4, 9, 18, 21, 22, 23, 26
and 27 now explain the supplied premises, tested consequences and relevant
limits without an engine lesson. The optional method integrates whole-input
rollback, ballot-rule coexistence, floor and shield dependency examples,
purpose-use inspection, and the separate amendment host's byte comparisons
and in-memory transitions with its existing worked cases. Its added formal
excerpts are existing executable statements and expectations. The host's
development command remains separate from routine Nibli verification.

The prisoner chapter leads with standing and its tested protective effect
beyond the prisoner. The ordinary chapters retain the sparse-record limits,
accepted harmful alternatives, exact scope of refused rules, absence of
delivery, distinction between review and relief, and lack of authenticated
approval or actual publication. No consequence or limitation requires a method
link to discover it. The review's arity lesson was already absent from the
current ordinary chapters; this finding is corrected, not treated as another
unperformed edit. The existing narrative-register decision records the exact
canonical prose as author-approved under the delegated 2026-09-13 authority.

The constitution, substantive pins and suite inventory are unchanged. Five
affected prose locators are updated in their existing records. The ecological
locator now names the actual environmental distinction, and the older
record-integrity summary is corrected from a general monotonicity claim to
the two tested ballot rules. Three renamed section entries are updated in
the owning reader-coverage JSON and its generated report; all 238 passage
classifications remain. These edits renew no historical assurance finding
and introduce no new inventory or verification gate.

Focused runs pass all 70 Chapter 4 pins (22.30s), 31 Chapter 18 pins (18.30s),
38 Chapter 27 pins (17.30s) and 32 Chapter 22 pins (12.39s). All five
claim-discipline, six reference-integrity, ten reader-coverage and sixteen
floor development tests pass. The amendment host's fourteen development
tests pass in 121.72s, including exact-source transitions on the actual
constitution. Initial reference and coverage failures identified the five
old locators and three old section names; the existing records are corrected
and both suites pass on recheck. The complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across 16,137 cases**,
with complete contradiction checks and no findings, in **1,145.04s**. The
five-minute target remains unmet. All formal inputs remained unchanged
throughout the run. Item 26 is complete.

The separate prose review measures 57,645 words across the 34 ordered inputs,
excluding HTML comments and splitting on whitespace; 40,642 words are derived
(70.50%). The method is 3,936 words. The chapter order, three exempt prose
channels, unnumbered epigraph and method, single birth fixture, required child
slots and their exemptions remain. Book 2 stays collection-only until Gate C.

### Item 25 — choices in the opening and direct routes to their arguments, 2026-09-21

Under the standing delegated approval, the opening now introduces the main
institutional choices before their detailed chapters. Public responsibility,
plural providers, residence-based voting, divided government, collective
execution, independent appointments, purpose limits, provisional disclosure
protection, emergency limits and the protected cores are stated as choices
requiring reasons. The text separates their aims from successful operation
and distinguishes rights from the instruments selected to serve them.

An early direct route reaches Chapter 1 before the optional reference material.
The argument map pairs mechanisms with the substantive Part V sections, and
a heading over the existing scarcity argument gives it a direct destination.
The glossary distinguishes the protected core from amendable institutions and
identifies the competence certificate separately from other certificates.
The exact approved text and retained child-argument sequence are recorded in
the existing reading-order decision. The child definition, bounded standing
result, methodological argument, prisoner pair, affect warning and book title
remain in their required order. No new prerequisite to the child case, source
rule, pin expectation, institutional permission or outside result is added.

The six reference-integrity checks and all 58 Chapter 1 pins pass; the focused
pin run takes 11.97s. A later reference check caught the glossary label's old
locator in `full-society-ledger.json`; its existing prose reference now names
the competence certificate explicitly, and all six reference checks pass again.
The complete `RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across
16,137 cases**, with complete contradiction checks and no findings, in
**1,148.88s**. The five-minute target remains unmet. All formal inputs remained
unchanged throughout the run; the ledger edit changes only a prose locator.

The separate prose review confirms the question, commitments, choices,
evidence boundary and reading routes without relying on the glossary. The
early Chapter 1 link follows 279 whitespace-delimited words. The 34 ordered
inputs now contain **56,943 words**, excluding HTML comments, of which
**40,716 are derived (71.50%)**; the opening and its optional reference material
contain 3,620. Child slots, exemptions, chapter order, unnumbered epigraph
and method, and Book 2's collection-only status remain unchanged.

### Item 24 — institutional alternatives and the protected cores, 2026-09-21

Under the standing delegated approval, Part V now separates reasons to protect
rights from reasons to choose or entrench an institutional instrument. Its
exact approved text and comparative assessment are recorded in the existing
state-form decision and the owning appointment, record-power, shield,
economic, emergency and ecological records. No source rule, pin expectation,
suite route or constitutional threshold changes in this item. The finding was
an incomplete comparative argument, not an executed new constitutional defect.

The revised comparisons cover appointment separation, privacy and coordination,
immediate case-specific shield protection, unitary and federal authority,
territorial and population-based representation, a prime minister and collective
executive, formal presidential functions, residence and diaspora membership,
provider forms and public continuity, and bounded emergency derogation. Each
identifies people bearing the cost, existing safeguards and reasons to prefer
an alternative. The protected-core argument separately addresses the continuing
standing of a defeated political participant, children and people unable to
participate, and animals' direct interests without a franchise. It distinguishes
the core from revisable instruments and examines the costs of entrenched
interpretation and the amendment threshold itself. Items 21–23 remain intact.

The current official German Basic Law Article 67 and OHCHR-hosted ICCPR Article 4
were read as primary legal comparisons and added to the existing claims registry
and Part V bindings. They establish constitutional provisions, not successful
operation or a causal case for this design. Historical examples retain their
qualified evidential roles. No external testimony, operation or review is claimed.

Focused checks pass 43 state-form pins, 102 shield pins across its chapter and
selected counterfactual, and 32 amendment pins. The five existing claim-discipline
tests and six reference-integrity tests pass on the final prose. The optional
`cargo fmt --all -- --check` reports formatting differences in ten authoring
files, reproduced against `HEAD` with the same formatter and edition settings;
this item does not include an unrelated formatting sweep. The complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` passes **88,815 pins across 16,137 cases**,
with complete contradiction checks and no findings, in **1,137.24s**.
The five-minute target remains unmet. All formal inputs were in place before
the run and remained unchanged throughout it; prose was reviewed separately.

Separate prose review checked the current-rule account, serious alternatives,
cost-bearers, actual safeguards, source qualifications and the distinction
between commitments and instruments. The ordered manuscript remains 34 inputs
and measures **56,409 whitespace-delimited words**, excluding HTML comments;
**40,716 are derived (72.18%)**. Part V accounts for 9,338. The chapter order,
unnumbered epigraph and method, child slots, exemptions and Book 2 boundary
remain intact. No reader-facing revision history is added.

### Item 23 — comparative scarcity decisions and separate protected claims, 2026-09-20

Under the standing delegated approval, the existing
[scarcity and conflict contract](book-1/appendix/contracts/scarcity-and-conflict-contract.md)
records a demonstrated missing allocation-specific reasons requirement and its
repair. The complete prior allocation fixture qualified without named compared
claims, a specified decision or comparative reasons. Its frozen six-pin case
first failed the two intended refusal expectations, then passed after the repair.
It remains registered with the generator without being rewritten into compliance.

The allocation now requires its own claims, decision and reasons, qualified
comparison of all relevant permitted considerations, a compatible comparison
and method, and publication of comparative reasons and rejected alternatives.
A usable equal share requires preserving that share; without one, materially
unequal claims permit reviewed comparative priority, and materially equal claims
permit disclosed rotation or lottery. No numerical weighting or engine-chosen
recipient is introduced. A shortfall identifies the unmet claim. A supported
comparison defect withdraws only the allocation while its genuine scarcity
finding and shortfall duties remain; procurement refusal defeats the scarcity
basis instead and requires restoration or procurement without cancelling rights.

The worked indivisible power-unit case compares earlier need and greater
resource-specific benefit for the waiting claim with imminent irreversible
interruption and continuity for the continuing claim. Its supplied qualified
decision, retained shortfall, challenge, defect, worth refusal, equal-claim
lottery alternatives and unequal-lottery refusal are executable. The complete
comparison/method matrix and explicit removal of that guard test the fixed
procedural ordering. The economic allocation route already carries an actual
allocation record and public reasons tied to source, facts, holder and effect;
the ecological connector still consumes that actual proof.

New ecological cases supply an existing essential water service conflicting
with river minimum flow after alternatives and procurement are examined. The
already-ratified dual-continuity rule retains both failure labels, least-harm
immediate continuity, alternatives, repair and the record's own end. It gives
no separate allocation or animal-use permission. The qualified Guardian/Animal
Advocate conflict decision remains distinct from an invasive fish intervention's
welfare, necessity, alternatives and independent prior-review conditions. No
new ceiling waiver or general human-purpose priority is enacted. No inventory,
measured flow, outside judgment, delivered provision or rescue is invented.

Chapters 6, 7, 13 and 31 contain the exact approved reader projection and
comparative argument. Their pins and existing coverage owners are updated;
all existing case IDs remain. The complete `./verify.sh` passes **88,815 pins
across 16,137 cases**, with complete contradiction checks and no findings, in
**1,184.07s**. The five-minute target remains unmet. All final formal inputs
were captured before that run; subsequent changes concerned prose only.
Focused checks include the frozen six-pin regression, 49-pin comparison-defect
sequence, four-pin method counterfactual, 12-pin water/allocation cases,
nine-pin animal permission and prior-review control, and chapter pairs with
78, 30 and 41 pins. Sixteen floor-vector, six reference, ten reader-coverage
and five claim-discipline development tests pass. The reference test first
found one new broken heading link, which was corrected and rechecked.
The final prose was reviewed separately. The 34 ordered reader inputs total
54,249 whitespace-delimited words excluding HTML comments, with 40,716 derived
(75.05%). Child fixtures, exemptions, editorial order and the Book 2 boundary
remain intact. Item 23 is complete and removed from the active backlog.

### Item 22 — severity limits secure placement without selecting it, 2026-09-20

Source and prose are `session-drafted, author-approved under delegated approval
(2026-09-13)`. The [public-safety decision](book-1/appendix/decisions/public-safety-defence-emergency-and-external-power-decision.md)
supersedes the qualifying-pair threshold and its exclusion of ordinary places.
Qualified grave injury or aggravated cruelty now permits consideration of
secure placement. Grave injury covers death, life-threatening injury and serious
enduring physical or psychological harm; aggravated cruelty requires deliberate
severe suffering. Victim count, raw acts, family status, wealth and poverty
cannot replace those findings or individual placement necessity. The existing
criminal merits, case and incident binding, independent review and procedure
remain required. These are supplied adjudicated findings, not the engine's
assessment of harm or predictions about a person's character.

Home confinement and ordinary supported residence remain eligible in a severe
case. The actual place needs its independent availability, lawful conditions,
necessity and review; a secure place additionally needs positive insufficiency
of less restrictive alternatives. The current lease, independent intake and
challenge protections remain separate. A different case cannot borrow severity
or secure eligibility. Conviction, gravity, placement and current authority are
not interchangeable. The source adds no numerical harm ranking, personal
score or gravity-based sentence-duration table.

The exact previous pair-only Ruk scenario survives under BoundaryTwoVictims;
it now lacks secure eligibility. Canonical Ruk receives an explicitly supplied
fictional grave-injury finding. The new 221-pin comparison covers grave harm to
one person, cruelty alone, ordinary placements despite severity, missing or
mismatched findings, forbidden placement grounds and retained floor and adult
ballot claims. The eight-pin explicit no-ceiling counterfactual exhibits the
greater discretion of necessity alone. Existing scenarios and every old matrix
query remain; their superseded home-eligibility expectations are revised openly.
Part V compares alternatives, identifies who bears mistaken classifications and
inadequate protection, and states grounds for reconsideration. Chapters 3, 28
and 29 project current consequences; chapter 27's refusal checks still pass.
No reader-facing development history, outside success or independent endorsement
is added. The old receipt-era placement report is labelled historical.

The complete `./verify.sh` run passes **88,315 pins across 16,091 cases**, with
complete contradiction checks and no findings, in **1,187.06s**. The five-minute
target is unmet. The run captured the execution case before its final additional
cross-case assertion; the expanded 50-pin file separately passes in 13.74s
against the same executable constitution. No old assertion was removed or
changed in that file. Together the checks exercise every current expectation.
The 102-pin floor file passes after correcting its superseded Ruk eligibility
expectation. Six reference, ten reader-coverage, five claim-discipline and
sixteen floor-vector development tests pass. Prose consistency was reviewed
separately; all 34 ordered inputs total 52,755 whitespace-delimited words,
excluding HTML comments, with 39,680 derived (75.22%). Child fixtures,
exemptions, editorial order and the Book 2 boundary remain intact. Item 22 is
complete and removed from the active backlog.

### Item 21 — contribution without a constitutional recognition status, 2026-09-20

Implementation and prose are `session-drafted, author-approved under delegated
approval (2026-09-13)`. The controlling
[`recognition decision`](book-1/appendix/decisions/recognition-purpose-decision.md)
supersedes the earlier retention, three-producer and blanket-loss rulings.
Current source confirmed that a case about one act withheld acknowledgment for
all unrelated contributions; the earlier claim that recognition was the only
consequence was false after the prospective-signature repair. Removal is chosen
after comparing penalty-free acknowledgment, contribution-specific withdrawal,
existing consequential-record correction and relevant official restrictions.

The two contribution producers, examination producer and `lose(Points, subject)`
producer are removed. `reward` remains derived-only with no producers or readers;
it cannot become an asserted replacement badge. Existing public-power `lose/3`
conclusions are unaffected. No personal credibility finding erases teaching,
employment, wages, supplements, student rights, political rights or the floor.
Purpose-specific fraud findings keep their separate compensation and insurance
consequences. Existing case and restoration checks remain; Appeals additionally
owes review of a continuing personal restriction without a subject's request.
A duty does not establish performance or automatic expiry. The restriction on
new adverse signatures needs a qualified disposition, not a public-esteem penalty.

The 25-pin regression failed five expectations against the prior source, then
passed after removal. The ten-pin explicit counterfactual retains the old rules
and reproduces the blanket loss. Chapter 26 checks unrelated care, wages,
supplements, exact-basis correction, restitution, floors and restoration.
Chapter 29 retains the now-accepted raw-release rule as a control; the actual
custody-bar rule still refuses. This dependency change is reflected in prose.
Chapters 10 and 26 are rewritten; 26 is titled *The Limits of a Finding* while
its stable file path remains `26-clawback.md`. Related reader references and
existing coverage owners are updated. Part V gives the comparative argument
and states the cost of review that never happens. The chapter 26 child exemption
now rests on Cira’s separate claim beside chapter 25’s Nell case; its membership
is unchanged. The complete `./verify.sh` run passes **88,069 pins across 16,089
cases**, with complete contradiction checks and no findings, in **1,170.78s**.
The five-minute target is unmet. Six reference, ten reader-coverage and five
claim-discipline development tests pass; the sixteen floor-vector tests pass
as well. Prose consistency and majority-derived length were checked separately
(39,381 of 52,005 whitespace-delimited manuscript words, 75.73%). The existing
source and counterfactual inventory is preserved, with two added regression
cases. Item 21 is complete and removed from the active backlog.

### How these rulings are written

**`Ratified but unimplemented` is a standing label with a fixed meaning.**
Where a ruling below carries it, that ruling creates **no** predicate, rule,
pin, finding, ceiling, duty, remedy, institution in law, chapter, programme,
age number, or public coverage claim. The division of labour is also fixed:
the formally audited specification owns the legal definitions, reach,
evidence, challenge, continuity, remedy, and failure polarity, and Book 1
renders them; Book 2 owns operation, capacity, staffing, costs, workflows,
and empirical feasibility; Nibli may consume a bounded authenticated finding
but performs no institutional act, authenticates nothing, proves no arrival,
and advances no clock. Each ruling then states only what is **specific** to
it — the predicates needing retain/replace/retire cards, the particular Book 2
surface, the particular thing Nibli must not be mistaken for — and names its
controlling record. Read the label as that whole paragraph every time.

A ruling that has since been implemented says so in a dated supersession note
in the same subsection. A structural landing is never an operational claim.

The 2026-08-15 no-external-reviewer dependency ruling is kept as its own
top-level section at the end of this file, because it cuts across every gate
rather than sitting in one group.

### Scope, mandate, and the two-book seam

**The author has chosen the expanded mandate.** The formally audited
specification is to become a complete constitutional interface for a free
society: universal standing; non-reciprocal material floors; liberties and due
process; democratic authority; public duties; separation of public functions;
records and accountability; locality; emergencies; and common-resource/
intergenerational limits. Book 1 must render that interface accurately for
readers. This is a scope decision, **not** a claim that the present specification
or book already achieves that coverage.

[`book-1/appendix/maps/constitutional-coverage-map.md`](book-1/appendix/maps/constitutional-coverage-map.md)
is the controlling planning artifact. Before a new domain is drafted, its row must
specify its holder, duty-bearer, minimum or limit, admissible evidence, failure,
interim continuity/remedy, appeal, audit, independent check, and Book 2 handoff.
A named right, body, or value is not a complete constitutional interface until it
is formalised, pinned, counterfactually tested, and described accurately in
derived prose.

This supersedes as a **scope class** the previous Book-2-only governance-vocabulary
ruling, learning-only delivery ruling, Book-2-only duty-bearer-oversight ruling,
and roster-integrity-as-disclosure-only ruling. The specification may now add
coherent constitutional rule families for public authority, democratic limits,
delivery evidence, breach, continuity, remedy, review, and record integrity;
Book 1 must project each landed family. Neither may smuggle in operations:
transition, tax rates, budgets, staffing, procurement, facilities, service
workflows, clocks/calendars, record technology, cryptography, and case
administration remain Book 2.

**The full-society volume, edition, and stopping boundary is author-ratified
(2026-08-07): two-book model + C then E + versioned closure.** The formally
audited specification owns the complete constitutional and social destination:
constitutional invariants, democratic steering, institutions, protected
private/civic life, and normal, failure, challenge, remedy, restoration, and
recovery interfaces. Book 1 is its controlled reader projection. Book 2 owns
staffing, costs, capacity, resources, technology, workflows, transition,
deployment, empirical feasibility, and operation under ordinary and declared
shock conditions. Book 2 remains collection-only until Book 1 — First Edition
ships at Gate C. Gate D permits only immutable Book 2 previews or release
candidates; Book 2 — First Edition and the integrated full-society release wait
for Gate E. E2 + P1 + D2 remains in force, and its “completed expansion” now
means cumulative Gate C completion. Only Gate E permits the bounded claim that
the exact paired editions model a fully functional society for the declared
reference envelope. The controlling decision is
`book-1/appendix/decisions/full-society-boundary-decision.md`.

The finished work must give an ordinary reader a reliably balanced view of
normal life, provision, care, creation, democratic choice, private freedom,
failure, coercion, repair, and recovery. “Exhaustive” may describe only the
disposition map for named axes, a named source version, a named envelope, and a
named review event. Every material social domain and cross-domain dependency
must receive an explicit disposition: constitutional invariant,
democratic/ordinary-law choice, protected private/civic freedom, Book 2
operation, or external assumption. This does **not** authorise constitutional
control of every human choice or assert timeless completeness. One reviewed
canonical source must own stable domain, role, body/institution, power,
dependency, scenario, claim, external-assumption, threshold and assurance-route
IDs; coverage, assurance, reader and Book 2 views are generated projections,
not parallel hand-maintained truths. Every “functional” or feasibility claim
is bound to a versioned reference envelope and declared adequacy,
equality/accessibility, continuity, resilience, sustainability and resource
criteria. Scope, Book 1 destination, Book 1 reader readiness, Book 2 operations
and the integrated two-book claim are separate cumulative gates. Scope inventory
and neutral decision briefs may precede an outstanding author ruling; each
contested rule family, dependent prose and public claim remains gated until its
own ruling lands.

**Book 1's public-edition boundary is author-ratified (2026-08-04): E2 + P1 +
D2, refined by the 2026-08-07 full-society boundary.** The pre-expansion
baseline remains public source and git history but receives no promoted edition,
canonical serialization, assembled release, or print identity. “Completed
expansion” now means cumulative Gate C completion. Gate C publishes Book 1 —
First Edition, its assembled digital artifacts, and its first Book 1 POD under
one provenance contract; it makes no operational or integrated full-society
claim. After Gate B and before Gate C, coherent milestones may appear only as
immutable, tagged First-Edition previews with permanent URLs, exact book and
nibli sources, full verification records, and visible supersession. They are design snapshots, not
promises of final chapter order, and no public release candidate becomes an
edition early. `main` is never an edition URL, and no public artifact may be
silently replaced. `book-1/appendix/decisions/edition-boundary-decision.md` controls
Book 1 publication mechanics; `book-1/appendix/decisions/full-society-boundary-decision.md`
controls Gate A–E labels, the two-book seam, and versioned closure. Neither
ruling creates a tag, release, site, preview, or print file by itself.

### Constitutional baselines

#### Taxonomy — 2026-08-03, extended by Class 10 on 2026-08-08

**The constitutional taxonomy is author-ratified (2026-08-03) and extended by
the author-ratified non-human-animal Class 10 (2026-08-08).**
`book-1/appendix/maps/constitutional-taxonomy.md` is an author-chosen planning
grammar for Phase 2, not a derivation, a claim that the named classes are
exhaustive, or current formal coverage. Liberty limits bind public power directly
and public institutions are responsible for preventing, investigating, and
remedying private interference; no general private horizontal effect is implied.
The equality ruling expressly binds its named public-facing private domains. The
later economic ruling adds independently triggered, function-specific public-like
duties for essential, dominant, gatekeeping, dependency-producing, locked-in, or
no-meaningful-exit private functions. Neither is a general horizontal effect; any
other direct private duty must still be named expressly.
Property is a conditional liberty bounded by floors and commons. Merit,
recognition, contribution, qualifying thresholds, and personal scores may not be
allocation keys for the unconditional floor, standing, authority, political
voice, or commons access. Purpose-limited contribution records may calculate a
separately enacted above-floor insurance supplement but may not read `reward` or
become a worth score. Commons and future-condition claims may be initiated by any
present person, a qualified association, the rights advocate, or the independently
checked collegial Future Conditions Guardian. Each route is independent; none
requires another claimant to be present. The 2026-08-08 ecological ruling settles
the Guardian's authority, evidence boundary, independence, Guardian-only
automatic-pause power, case-bound replay key, conflict controls, removal,
alternate advocate, independent substitute reviewer, and lack of a final veto.
Direct non-human-animal welfare and integrity is Class 10 rather than human
personhood or an extension of the ecological-commons class.
Structural refusals remain cross-cutting walls, not ordinary subject-matter
rules: Article 1's firewall, `admits`, `derived_only`, and verifier guards keep
their own enforcement and test obligations. This ruling adds no predicate, duty,
or present remedy.

#### State form and political membership — 2026-08-07

**The constitutional state form and political membership are author-ratified
(2026-08-07): federal constitutional parliamentary republic + residence-first
membership.** The common/federal, regional, and local tiers are protected;
common powers are enumerated by category, irreducible local competences are
constitutionally reserved, and regions hold the remainder under a justiciable
subsidiarity rule,
while no tier may lower universal standing, the material floor, equality, due
process, core liberties, or commons limits. The common institutions are a
population-proportional People's Assembly, a Regions Council of proportionally
delegated regional legislatures with equal aggregate regional weight and only a
suspensive ordinary-law veto, a collective Assembly-dependent
Executive Council, a non-executive Civic President, ordinary courts plus a final
Constitutional Court, and separately checked electoral, audit/integrity,
ombudsperson, and appointments functions. Subnational variation remains inside
the common corridor and protected local self-government.

Equal regional Council weight is an express territorial exception to population
weight, confined by the Council's limited mandate; it is not a second popular
chamber or an unqualified claim of equal individual weight in both chambers.

Ordinary residence creates political membership; the single general legal-
adulthood status adds an equal vote and candidacy at the person's one political
home per tier. Citizenship, immigration classification, property, documents,
wealth, contribution, and residence duration add no electoral weight or gate.
Registration is evidence, not the source. A person with several qualifying
connections chooses one nested home; custody, institutional placement, eviction,
or forced displacement cannot move it by itself. Each decision permits at most
one person-bound effective submission across jurisdictions. Nonresidents keep
universal rights but receive no ballot; a former resident retains the right to
return without gaining a diaspora vote. Conviction or custody alone removes
neither franchise nor candidacy. The 2026-08-08 family/life-course ruling owns
the general adulthood status: one common-tier statutory age applies uniformly,
but neither that ruling nor this one supplies its numeric value or reuses
`mature` as the gate.

The Assembly removes a government only by choosing its successor. Direct recall
is limited to directly elected single-holder offices and chooses the replacement
on the same ballot. Constitutional amendment requires two-thirds of the full
People's Assembly and more affirmative than negative valid national referendum
votes, with no turnout quorum; regional competence or boundary changes also
require Regions Council and each directly affected region. An elector initiative
may force a constitutional docket and recorded Assembly vote but cannot bypass
the two-thirds threshold. Secession is negotiated, rights-reviewed, and finally
ratified, never unilateral; this settles the internal federal path, while the
security/external-power ruling of 2026-08-08 retains external recognition,
defence, cross-border status, and international obligations — and adds that the
external-only defence mandate leaves **no military instrument against a
unilateral exit**. Budget, formation, review,
appointment, succession, and transfer failures may not extend ordinary power or
suspend the floor. A T3 contract may end legal effect on supplied time evidence;
it cannot advance a clock or cause an election.

Appointments use open nomination, reasoned screening, divided sources,
staggered nonrenewable mandates, cause-only removal, and independently
challengeable concentration. Majority appointment control by one current
government, chamber, party coalition, profession, or appointing source is a
legal incompatibility; divided sources do not by themselves prove its absence.
This is a design against capture, not a claim that capture becomes impossible.
The remaining office-term numbers, seat allocations, electoral metrics, and
finite fallback mechanics are delegated implementation choices inside the
ratified sources, limits, defaults, and prohibitions; they are not open author
policy.
Leaving those bounds requires a new author ruling.

The finite collective-decision boundary that limits how much of this the
engine can carry is measured under *Measured Nibli capability boundaries*
below.

**Ratified but unimplemented.** `mature`, `decide`, `choose`, `broken`,
`approves`, `authority`, and every current institutional constant and consumer
route require explicit retain/replace/retire cards; custody T3 supplies no
office term or election clock. **Implementation supersession, 2026-08-22:**
`FS-CVF-003`, all 51 state-form cards, their pins, counterfactuals, reviewed
references, and approved Book 1 prose are now formalized and prose-landed.
That structural landing leaves every power, body, and `FS-CLM-18` at
`Specified`/`ratified-unimplemented`: it proves no staffing, independence,
operation, delivery, institutional action, or changing-result authentication.
Controlling record:
`book-1/appendix/decisions/state-form-and-political-membership-decision.md`.

**Appointment control made challengeable, 2026-09-24 (item 67).** A contrary
anchor record from one of a result's own attesters, or an independently reviewed
appointment-control finding naming the controller, withholds a certified
selection; a fallback appointment for a seat a lawful selection fills is owed
its end. The integrity contract card records the design.

**An election that cannot be held, 2026-09-24 (item 68).** Article 2b makes the
electoral body's duty formal, lets an outgoing chamber sit only under caretaker
limits until its successor meets, and sends impossibility findings and failures
to hold an election to the Constitutional Court; decision section 14 records the
compared alternatives.

#### Substantive equality and anti-subordination — 2026-08-07

**The substantive-equality and anti-subordination baseline is author-ratified
(2026-08-07): universal substantive equality + structural repair.** Every person
receives equal and effective constitutional protection. Purpose or effect can
establish discrimination; intent and a single-ground comparator are unnecessary.
The open protected-ground architecture covers actual, past, anticipated,
perceived, associated, multiple, and intersectional status. Direct, indirect,
systemic, and associative discrimination; denial of accommodation; imposed
segregation or exclusion; harassment; and retaliation are in scope. A persistent
disparity is evidence capable of shifting a burden and opening systemic review,
not an automatic finding or an individual verdict.

Every public tier, institution, contractor, and delegated public function is
directly bound. Express direct private duties apply in employment, housing,
education, health and care, finance and insurance, utilities, transport, public
commerce, and dominant communication or platform services. Intimate choice and
genuinely voluntary cultural, affinity, safety, faith, and expressive association
remain protected. A mission-linked distinction is confined to a genuinely
expressive role; it does not excuse discrimination in general employment,
commercial service, housing, or essential access. Imposed status segregation and
inferior parallel systems are prohibited, while voluntary association requires
free exit and undiminished common rights and services.

Direct adverse status distinctions require an evidenced, genuine and determining
function-specific necessity with no less discriminatory effective alternative.
Indirect distinctions require a compatible purpose, evidential fit, necessity,
least-discriminatory means, and proportionality. Accommodation and remedial
positive measures use their own ratified contracts rather than pretending that
status is a job qualification. No test can balance away standing, the material
floor, core liberty, due process, the adult ordinary resident's equal popular
ballot, or remedy. Equal regional Council weight remains the existing confined
territorial exception. A credible barrier or group pattern shifts the burden only
in remedial equality proceedings; it never reverses a criminal burden, proves
guilt, or authorises punishment.

Accessibility is proactive and systemic; reasonable accommodation is an
individual duty. An undue-burden claim reads the responsible institution as a
whole, not one office or budget line, and a provider's genuine inability cannot
leave the person without the floor or core access: an effective alternative and
public continuity route must remain. Accessibility and accommodation do not
sunset as positive measures.

Equality diagnostics are purpose-limited, privacy-preserving, independently
governed, and separated from the canonical consequential person record. Aggregate
patterns may create a rebuttable presumption and audit but never an individual
standing, floor, sanction, risk, entitlement, or worth conclusion. A separately
enacted positive measure may use voluntary self-description and minimally
necessary, contestable eligibility evidence for that measure only; no reusable
identity registry or documentation gate follows.

Positive measures, including bounded targets and quotas, are permitted and are
required where ordinary equality and accommodation cannot dismantle evidenced
structural disadvantage. Political measures may govern candidate lists,
nominations, outreach, and appointments while preserving one equal ballot,
overall proportionality, the appointment anti-capture controls, and no separate
electorate or reserved-seat exception in general-government bodies. Each measure
needs an objective, evidence, independent review, alternate reviewer, a fresh
source-bound temporal contract, and an end when the objective is sustainably
achieved; review silence cannot renew it. Public institutions must repair
continuing historical barriers, while democratic law chooses compatible means
without erasing official provenance or permanent historical public answerability.

Affected people, chosen supporters, qualified associations, and the independent
rights advocate or ombud may initiate review without erasing the person's voice.
Ordinary courts give case-specific relief; the Constitutional Court alone gives
final general invalidation; advocacy, audit, adjudication, execution, and final
review stay separated. Individual continuity and relief coexist with related-case
re-audit, redesign, recurrence monitoring, and measures directed at
non-repetition. No remedy may take another person's standing, floor, core liberty,
due process, or political equality, or assign guilt solely by group membership.

**Ratified but unimplemented.** Book 2 owns collection and statistical
methods, accommodation operations, monitoring, enforcement workflows, and
empirical evaluation. Nibli may consume an authenticated external finding; it
is not a population-statistics, identity-authentication, or
institutional-liveness system. Controlling record:
`book-1/appendix/decisions/substantive-equality-and-anti-subordination-decision.md`.

**Statistics and diagnostic-evidence implementation, 2026-09-12.** The
official-statistics contract supplies scoped census, sampling, administrative,
planning and diagnostic authorization under positive necessity, privacy,
accessibility, retention, methodology and publication controls. Dataset,
purpose, source/method version and evidence period are joined to every
downstream use. Authorized conflicting values block a record; reviewed defects
withhold only the affected use. Constitutional, eligibility and enforcement
records remain separate, with no individual score or verdict consumer.

An aggregate protected-ground pattern can create a rebuttable presumption and
independent audit in a remedial equality proceeding, never a criminal burden.
Requests create review duties without authorizing private access; reviewed,
case-bound access and positively certified nonresponse have separate routes.
An independent alternate's duty does not depend on the challenged use staying
valid. No result establishes authentication, calculation, publication, deletion
or institutional action. Other equality measures and operations are not made
complete by this statistical interface.

`book-1/appendix/contracts/official-statistics-contract.md` owns this contract;
`statistics-source.json` and `./generate.sh statistics` explicitly produce its
ordinary rules and cases. Development tests guard the actual consumers against
named and generic person-side readers. The exact reader section "Counting
without ranking people" was separately approved on 2026-09-12 and inserted
unchanged in Book 1 chapter 1. Its supply is `session-drafted, author-approved`;
`book-1/source/statistics-reader-draft.md` preserves the approved wording.

#### Economic pluralism and the protected private sphere — 2026-08-07

**The economic-pluralism and protected-private-sphere settlement is
author-ratified (2026-08-07): bounded plural economy + protected voluntary
sphere.** Public, cooperative, commons, mutual, nonprofit, household, and private
arrangements are lawful; no form receives constitutional priority, guaranteed
share, profit, bailout, immunity, or monopoly. Democratic law chooses their mix
above the floor. Property, contract, enterprise, prices, accumulation, taxation,
and regulation remain conditional on standing, floors, substantive equality,
labour rights, core liberties, due process, anti-domination, remedy, and commons
limits. This does not enlarge the unamendable corridor beyond standing,
equality, the floor, due process, core liberties, and protected commons; other
economic rules remain constitutional but amendable only inside that corridor.
Price and lawfully held wealth may mediate ordinary above-floor private-property
transfers. `reward`, recognition, merit, a qualifying threshold, a personal
score, and raw work or contribution facts may not directly qualify property
access. A separately adjudicated wage or equity claim may support an ordinary
transfer; a purpose-bound grant, prize, or supplement uses separate equality-
compatible criteria. This expressly refines the broader 2026-08-03 property
wall without erasing its historical rationale or reopening recognition, merit,
score, threshold, or raw-contribution qualification. A provider label never
transfers the public duty to finance, secure, and maintain the noncontributory
floor. Price and ability to pay cannot gate it, and
cash, vouchers, or insurance count as delivery only where independent recipient-
side evidence establishes timely, accessible, adequate real access.

People may choose, refuse, leave, and change work without losing the floor.
Ordinary labour protections cover every ownership form and voluntary custody
work; worker status follows actual control and dependency, not a contractual
label. Narrow, reviewed minimum-service rules may protect life, health, safety,
or the floor, but they bind provider or bargaining-party continuity and may not
conscript a named worker, criminalize individual refusal, or withdraw the floor;
blanket strike bans are refused. Licensing requires evidenced
serious safety, fiduciary, or core-public-function risk, accessible alternative
proof, portability, review, expiry discipline, and anti-cartel safeguards.
Above-floor compensation, profit, savings, returns, grants, prizes, subsidies,
and bounded incentives are lawful but remain wholly separate from `reward`,
`false`, and `lose`; recognition stays binary, arity-one, non-ranked, and
unread.

Property and inheritance are conditional liberties. Personal possessions,
ordinary home use, plural tenure, and lawful productive activity receive strong
security without creating a right to a particular asset. Eviction or foreclosure
that threatens the housing floor requires legality, hearing, proportionality,
review, and real public continuity. Acquisition requires compatible public
purpose, necessity, reasons, hearing, review, and reliance-sensitive
compensation; unlawful privilege, windfall, penalty, and remediation liability
do not generate compensation. Public power and political weight cannot be
inherited. Statutory knowledge exclusivity remains time-bounded, may not block
a floor or commons duty, and must yield through an effective compatible access
route chosen by democratic law.

Contracts remain voluntary but cannot enforce deception, coercion,
unconscionable dependency, material non-disclosure, or waivers of protected
rights. Enterprises receive only functional legal capacities, never human
standing, a floor, ballot, candidacy, or inherent political weight. Limited
liability is a regulated privilege. Enterprise petition, testimony, and
attributed publication remain lawful; an enterprise treasury may not fund
candidates, parties, or purchased independent electoral advocacy. Associated
people retain their rights; unions and genuinely voluntary civic/advocacy
associations need a separate Class 5 finance card that prevents relabeling.
Existing equality duties attach in their named domains without a dominance
finding. Additional public-like duties require an independent, contestable
finding tied to a public-facing, essential, delegated, gatekeeping, or
systemically controlling function, using essentiality, dominance, network
effects, dependency, lock-in, information asymmetry, or lack of meaningful exit
as evidence.
Intimate dependency alone does not create public-scale status; coercion and
captivity remain subject to justice and public protection. The calibrated duties
may require access, continuity, reasons, portability, interoperability, audit,
challenge, and remedy.

Voluntary gifts, pooling, care, domestic production, mutual aid, lawful exchange,
belief, culture, worship, friendship, intimacy, and association are protected
from prescription, outcome certification, and social scoring. Each person keeps
independent floor access, legal capacity, confidential help, and free exit; no
family, household, association, or charity is presumed to supply the floor.
Privacy does not shelter violence, forced labour, confiscation, document control,
economic captivity, obstructed help, or denial of exit.

The tax system as a whole reflects capacity to contribute while democratic law
chooses rates, bases, and instruments. Collection cannot make a person floorless
or use debt imprisonment. Public finance requires legislative source, reporting,
audit, and fiscal-risk disclosure; there is no constitutional balanced-budget
rule or numeric debt ceiling, and debt cannot subordinate floors or commons.
The common tier maintains the public unit of account and accessible payment
backbone, including a non-digital route. Bounded monetary independence carries
published-reason, distributional-review, audit, and no-fiscal-veto duties. There
is no general right to credit. Contribution-based insurance may supplement but
never replace the floor. Personal insolvency provides a fresh start, wages and
pensions receive an effective protection route, shareholders bear residual risk,
and essential services continue through failure.

Genuine physical scarcity requires an authenticated, contestable, resource-
specific finding after reasonable alternatives, reserves, substitution,
coordination, and assistance are considered. Budget choice, price exclusion,
delay, artificial withholding, monopoly, and refusal to procure are failures,
not scarcity. Preserve each person's constitutional minimum wherever supply
makes that possible. If it does not, never redefine a reduced ration as the
minimum: record every unmet portion as failure and mitigate it by urgency,
accessibility, imminent irreversible harm, continuity harm, and individualized
resource-specific benefit after accommodation. That benefit cannot disguise a
lifespan, disability, productivity, or social-worth ranking. Wealth,
contribution, recognition, conviction, family status, disability stereotype,
productivity, social usefulness, generalized lifespan, and political favour are
forbidden priority keys; materially equal claims use a disclosed rotation or
lottery. This ruling creates no standing emergency economic power. Every
temporary authority needs its own source-bound temporal contract; missing
authority evidence ends the manager, not the independent floor or essential-
service continuity route.

The economic ruling deliberately supplements the state-form competence list:
the common tier owns the monetary/payment backbone, floor finance,
equalisation, portability, interregional commerce and competition, insolvency
baselines, common labour and consumer minima, and cross-regional private power.
Residual ownership, land, enterprise, service, cooperative, municipal-enterprise,
and local-development policy remains regional or local under subsidiarity;
stronger subnational protection survives, while no region may lower common
floors or safeguards or defeat portability. This is not a general commerce,
spending, taxation, or pre-emption power.

**Ratified but unimplemented.** Book 2 owns rates, budgets, models, prices,
production, inventories, monetary instruments, and empirical feasibility.
Nibli may consume a minimal authenticated, adjudicated, purpose-bound
projection through a distinct premise or result-record relation, never the raw
economic record; a consequential predicate protected by `derived_only` must
still be derived. Nibli neither calculates nor authenticates an economy,
proves scarcity or action, nor receives a universal economic score.
**Implementation supersession, 2026-08-28:** `FS-CVF-006`, the 28 economic
power contracts, three carry contracts, `FS-CCE-223` through `FS-CCE-367`, the
24-case acceptance matrix and its pins and counterfactuals, reviewed
references, and approved Book 1 prose are now formalized and prose-landed.
This structural landing remains source- and repository-bound: it proves no
operating economy, authenticated external finding, institutional action,
delivery, liveness, affordability, supply, capacity, stability, fiscal or
monetary performance, scarcity, or empirical feasibility. The current source
still awaits its receipt-bound repository audit. Controlling record:
`book-1/appendix/decisions/economic-pluralism-and-protected-private-sphere-decision.md`.

**Income-security supersession, 2026-09-05.** `FS-CVF-018` supplies the
positive above-floor interface the economic barriers presupposed: `pay/4` is
the purpose-limited contribution record (payer, `Contribution`, carrier,
scheme), `insure/3` the derived-only supplement conclusion per named event,
and the two rules join the record to an adjudicated event finding by a writer
authorised for that person at the same scheme and distinct from the carrier,
under the Court's absent individualised fraud finding; the guarantee rule
concludes with `PublicGuarantee` on the Court's carrier-insolvency finding.
Names are corpus-bound — `contribute`, `pension`, `earn`, `benefit`, and
`guarantee` do not exist in the lexicon and `deserve` is refused for its
wage-for-work places. **The engine accepts a rule that confines a person for
lacking a contribution record** (measured 2026-09-05: `pay` is a base
relation, no negative cycle), so `src/checks/repository.rs` holds `pay`
readable only by the two supplement rules, never under negation, never
concluded, with the `unguarded-contribution-reader` fixture as its watched
failing control. The family ships dormant, `person` is not a premise
(serve-first), stratification does not move, and no amount, adequacy,
solvency, funding, payment, or arrival follows. Contract card:
`book-1/appendix/contracts/income-security-and-social-insurance-contract.md`.

**Qualifications-and-compensation supersession, 2026-09-05.** `FS-CVF-019`
supplies the positive side of the licensing and compensation walls: `promise/3`
is the compensation record (payer, instrument kind, person — Wage, Profit,
Return, Grant, Prize, Subsidy, Incentive as constants), `provide/3` the
derived-only compensation and, with `Restitution`, restitution conclusion, and
`grant/3` the derived-only certificate (certifier, function, person). A
certificate derives from an authorised certifier distinct from the person and
is read by nothing; compensation derives from the promise plus an independent
same-kind attestation at a basis under the Court's absent fraud finding;
restitution is basis-bound. Mandatory licensing stays with `FS-POW-061`. The
purpose-limited-record guard is now table-driven over `pay` and `promise`, and
the no-reader list holds `insure`, `provide`, and `grant`. Inherited advantage,
metric gaming, and proportionate restitution are pinned. `certify`, `license`,
`award`, `compensate`, and `hire` are not corpus names. Contract card:
`book-1/appendix/contracts/qualifications-and-compensation-contract.md`.

#### Family, dependency, reproduction, and collective plurality — 2026-08-08

**The family, dependency, reproduction, and collective/plurality baselines are
author-ratified (2026-08-08): independent personhood + supported agency + modular
plural families + reproductive bodily authority + differentiated collective
rights.** A born child is an independent rights-holder. Best interests are a
primary substantive and procedural consideration, never an ownership claim or
automatic adult veto; the child is heard without a minimum speaking age and
receives increasing, decision-specific authority with accessible support.
Parent and caregiver powers are fiduciary, scoped, challengeable, and
reviewable. Separation requires individualized serious-harm or neglect evidence,
support before removal, the least disruptive effective response, continuity,
and reunification or a stable alternative. Poverty, disability, custody,
culture, or family form alone is insufficient. Safe, voluntary, education-
compatible child activity receives ordinary labour protection and wages;
hazardous, coerced, exploitative, or floor-conditioned work is prohibited.

Common-tier law sets one uniform legal-adulthood age. Status arises
automatically at that age; records evidence it but do not create it. Missing,
disputed, or corrected evidence cannot create a gap, and a later increase cannot
remove established adulthood. Conditions may confer a particular decision power
earlier but may not replace general adulthood, become a global capability score,
or leave anyone permanently below adult status. No separate political maturity
test, higher candidacy age, retroactive loss, or guardian ballot is permitted.
Plenary guardianship and status-based incapacity are refused. Adults choose,
change, or refuse supporters; when will and preferences remain unascertainable
after support, only a conflict-free, decision-specific, least-restrictive,
time-bounded best-interpretation route with independent review may act.

Family law is modular. Parentage, caregiving, household, support, property, and
decision-agent roles remain distinct and may be combined by pair or plural
families; marriage may remain a private or statutory label but carries no
constitutional privilege by itself. More than two legal parents may be
recognized through prior intent, adoption, or an adjudicated child-continuity
finding. Parents and expressly responsible caregivers owe capacity-bounded
duties, while public continuity stays immediate and non-delegable. Adult kinship
creates no compelled personal-care duty or inherited support debt. Every
caregiver and dependant keeps separate standing, floor access, voice, privacy,
help, respite, exit, and remedy. Domestic privacy does not shelter violence,
sexual abuse, coercive control, exploitation, serious neglect, captivity,
document control, or obstruction of help; intervention needs credible evidence,
least-restrictive scope, temporary emergency authority, review, continuity, and
repair. Imprisonment alone ends no parentage, contact, or custody relation.

Reproductive care is protected from coercion, discrimination, punishment, and
third-party authorization. The pregnant person controls continuation and
termination throughout pregnancy; no embryo or fetus receives independent
constitutional personhood capable of overriding that authority. Assisted
reproduction and gestational agreements remain lawful under consent and anti-
exploitation safeguards, and no agreement may control bodily decisions. Law must
avoid a child's parentage gap and may regulate compensation. Accurate birth,
adoption, donor, and gestational-origin records are preserved for access by the
person concerned, without creating automatic contact or relationship rights.
Age, disability, diagnosis, or residence setting never automatically removes
agency. Current informed treatment choice controls, including refusal of life-
sustaining care; otherwise a valid advance directive, a chosen representative
implementing known will and preferences, and only then the bounded best-
interpretation route apply. Palliative care is guaranteed. Medically assisted
dying for adults is democratically permissible, neither guaranteed nor
prohibited, and any enabling law needs explicit, informed, repeated consent,
independent review, protection from pressure, real care alternatives, and its
own temporal contract. A valid will leads succession subject to bounded
dependant claims, estate liabilities, collective title, and anti-domination;
intestacy follows functional legal relationships rather than marital rank.

Indigenous peoples receive protected internal and local self-government,
institutions, language, culture, education, collective land/resource title,
participation, restitution, and remedy. Other linguistic, religious, ethnic,
and minority communities receive culture, language, education, media,
association, accessibility, anti-assimilation, and participation protections;
territorial authority needs an independently established historical or
territorial basis. Membership uses self-identification plus the collective's
lawful acceptance process, permits multiple membership and exit, and carries
privacy and procedural challenge. Internal selection and customary law may
differ from general-government rules only inside universal standing, equality,
liberty, due process, individual voice, appeal, and Constitutional Court review.
Actual collective consent is required for permanent forced relocation,
extinguishment or irreversible impairment of collective title, sacred-site
destruction, hazardous-material placement, and comparable existential harms;
temporary lifesaving evacuation uses a separate emergency contract and never
extinguishes title. Other material effects require good-faith consultation,
accessible information, time, accommodation, reasons, and review, not a blanket
veto. The equal general-government ballot, common services for nonmembers, and
the negotiated-secession route remain unchanged.

**Ratified but unimplemented**, and it supplies no age number. `mature` must
be replaced; unary `family` retired as a placement or liberty proxy; `home`
split by legal effect; and absence-based `parent`, `married`, and `sibling`
independence replaced by case-bound conflict findings. `work(Care)`,
`healthy`, `free`, `public`, and `owe` cannot stand in for care, consent,
emancipation, collective status, or responsibility. Book 2 owns service
capacity, family and care proceedings, clinical and death-record operations,
land/title administration, language services, and consultation. Nibli does not
decide best interests, capacity, consent, pregnancy, parentage, death,
collective identity, membership, or title. Controlling record:
`book-1/appendix/decisions/family-dependency-reproduction-and-collective-plurality-decision.md`.

**Ordinary-half implementation, 2026-09-16.** The baseline landed 106
person-held barriers and no interface underneath them, so the domain could only
ever appear under strain. `book-1/appendix/contracts/family-life-ordinary-contract.md`
owns the ordinary half: scoped separable roles (parentage, caregiving,
household, material support, property/succession, decision agent) as
challengeable fiduciary powers over a named decision; the affected person's
participation with no minimum speaking age and no global capability score;
chosen, changeable, refusable support that may not substitute a decision while
support can still be given; the bounded best-interpretation route gated on
positive evidence that support was provided first; immediate non-delegable care
continuity; reproductive and bodily care without third-party authorization; the
fixed treatment order (current choice, directive, chosen representative, then
the bounded route); own origin-record access and correction; and the family's
defect and nonresponse routes. `family-life-source.json` and
`./generate.sh family-life` produce the rules and cases.

**It is a separate block beside the barriers, and the reason is a near-miss
worth keeping.** The generator was first given the barriers' own
`FAMILY-LIFE-COURSE` markers, and generating **silently replaced all 106 of
them**. The suite stayed green — the constitution loaded, the new family's cases
passed — and the deletion surfaced only because the spine's predicate count fell
from 95 to 92, `home`, `married` and `sibling` having lost their last reader.
The ordinary half now writes `FAMILY-LIFE-ORDINARY`. **A generated family whose
block name collides with an existing block deletes that block's contents, and
the same hazard exists for a family name colliding with an existing
`tests/pins/<dir>`.**

The family introduces no new relation name: predicates stay at 95, derived at
51, strata at 9, the floor at its eight rights, and only the rule count moves.
Nothing concludes about a person, `mature` is not read and no age is supplied;
Article 4's legacy absence-based `parent`, `married` and `sibling` checks are
untouched and still await their replacement. The exact reader section "What
holding a role in somebody's life actually is" is inserted in Book 1 chapter 9,
`session-drafted, author-approved under delegated approval (2026-09-13)`;
`book-1/source/family-life-ordinary-reader-draft.md` retains the wording.
Nothing here proves a child was heard, a supporter arrived, care continued or a
correction was published. With the family in place the complete inventory passed
on 2026-09-16: all 84,289 pins across 15,479 cases with complete contradiction
checks and no findings, in 836.96 seconds, and again at 84,304 across 15,480
after the accusation case landed; the nine known-defect pins still
reproduce, and the under-five-minute target is still not met.

**Mobility/plurality implementation and reader projection, 2026-09-13.**
`book-1/appendix/contracts/mobility-and-plurality-contract.md` defines the separate
mobility/access, membership, Indigenous/minority capacity, title, internal-law,
representation, consent, consultation, remedy and external no-evasion effects.
The explicit `mobility` generator owns their rules and substantive cases.
Current source/evidence/review, exact prerequisite joins, authorized-conflict
checks, free exit, positive defects, challenge and independent alternates have
separate boundaries; no membership finding produces standing, punishment,
political weight or a personal score. Removal and evacuation compatibility
are not coercive authority. The remaining public-safety/treaty/defence work
and all Book 2 operations remain separate. The exact Book 1 additions in
`book-1/source/mobility-reader-draft.md` are inserted in chapters 8 and 9,
session-drafted and author-approved under delegated approval (2026-09-13).
All 7,930 substantive pins
passed with complete contradiction scans and no findings in 531.67 seconds;
this does not meet the under-five-minute target.

#### Ecological, future-generation, commons, and non-human-animal — 2026-08-08

**Implementation and reader projection — 2026-09-14.**
`book-1/appendix/contracts/ecological-and-animal-protection-contract.md` owns the
case-level implementation. The explicit ecology generator supplies distinct
human environmental claims, non-substitutable commons axes, scientific and
actual democratic/court dependencies, liability and dual continuity, divided
Guardian/Animal Advocate functions, shared replay and historical finality,
direct animal interests, use-specific tests, particular remedies, record
withdrawal and separate source-bound windows. The actual amendment gate now
requires animal-core preservation and consumes independently reviewed exact
core-breach findings. No source label, generic compatibility or missing record
proves a lawful source transition.

The delegated-approved reader additions are retained in
`book-1/source/ecology-reader-draft.md`. The 130 cards and 9,373 ecological
cases are exported. All 77,902 pins across the complete 13,566-case inventory
passed with complete contradiction checks and no findings in 1,119.23 seconds
(18m39.23s); nine existing known-defect expectations still reproduce. The
under-five-minute target is not met and remains separate performance work.
This verifies the repository-bound legal interface, not institutional operation
or real-world protection. The author's latest instruction is to commit/push
this completed item and stop the goal, not start the next item.

**The ecological, future-generation, commons, and non-human-animal baselines are
author-ratified (2026-08-08): distinct environmental right + protected commons +
science-constrained democratic ceilings + directly protected sentient animals.**
Every present person has an independently enforceable right to clean, healthy,
and sustainable environmental conditions. This right is distinct from the
material floor and does not add a floor item. It may be violated before food,
water, health, dwelling, or another floor is lost; where ecological harm also
causes a floor deprivation, both claims remain available and neither substitutes
for the other. Climate and atmosphere, air, fresh and marine waters, soil,
biodiversity and genetic diversity, habitats, ecosystem integrity, connectivity
and resilience, regenerative and waste-absorption capacity, and the ecological
inheritance of land and nonrenewable resources are protected on separate,
non-substitutable axes. No sustainability score may hide one failed condition.
Protecting ecological function neither nationalises private, customary, or
collective title nor lets ownership waive the condition.

Future generations receive no present `person` record, ballot, presumed
preference, or fetal or embryonic standing. The protected object is the
ecological capability and reasonable option-space inherited by whoever later
exists. Versioned, multidimensional ceilings and budgets are enacted by
democratic law inside an independently reviewed scientific envelope; the
constitution supplies no number. A recalibration or weakening needs fresh public
evidence of equal or greater expected protection or corrected science. Cost,
profit, convenience, lobbying, or missing review is insufficient. Credible
serious or irreversible risk shifts the assessment, alternatives, and compliance
burden to the proponent without creating a zero-risk rule for harmless ordinary
life. Material-risk decisions require accessible information, participation,
reasons, cumulative and lifecycle assessment, independent assurance, challenge,
and interim relief before irreversible commitment.

The remedy order is avoidance, minimisation, in-place restoration, then genuinely
equivalent compensatory ecological restoration. An offset cannot legalise a
ceiling breach, irreplaceable loss, or local rights harm. Non-regression protects
outcomes rather than obsolete methods. Prevention and cessation do not depend on
fault. After causal connection is established, inherently hazardous activities
carry strict containment, restoration, and reasonable response-cost liability;
other liability follows proved causal contribution or control, while personal
punishment still needs individual culpability and due process. Unknown
or insolvent responsibility cannot stop public restoration. The common tier sets
common minima and governs interregional commons and cross-regional harm; stronger
regional, local, and collective rules survive. Indigenous title, governance,
consultation, and the ratified consent boundary survive, and conservation cannot
become a dispossession or assimilation device. Treaty, customs, and
extraterritorial machinery remain for the external-power ruling and Book 2.

The Future Conditions Guardian is a collegial independent office with open
nominations, divided selectors, mixed expertise, staggered nonrenewable terms,
cause-only removal, budget independence, conflict controls, and no selector,
government, party, industry, or profession controlling a majority. It may obtain
evidence, demand assessment and reasons, participate, publish warnings, and seek
review and remedy. An evidence-supported objection published by the collegial
Guardian—or by its predeclared alternate advocate acting in its place—identifying
credible serious or irreversible harm automatically pauses only the irreversible
authorisation pending expedited independent review. Any present person, qualified
association, or rights advocate retains an independent initiation route and may
request judicial interim relief, but their filing does not itself trigger that
automatic pause. The pause needs its own source-bound T3 contract and one
case-bound replay key binding the exact authorisation version, ground, and
reviewed evidence set across the Guardian, alternate advocate, successor office,
and reviewers. None may restart an unchanged key. Only materially new evidence
or a materially changed authorisation creates a new key and may support a new
pause. The predeclared alternate advocate preserves the Guardian's initiation and
automatic-pause route when the Guardian cannot act; a separately predeclared
independent substitute reviewer decides when the ordinary reviewer is
unavailable, conflicted, or captured. Neither absence may become approval or an
indefinite hold. The Guardian has no permanent veto, legislative or budget power,
programme control, scientific-oracle status, or authority to invent future
preferences. Scientific assessment, advocacy, authorisation, adjudication,
execution, and audit remain separate.

Public tiers, delegated functions, public authorisations, and materially
attributable private harm are directly bound. Diffuse cumulative harm ordinarily
runs through general law and sector ceilings rather than turning one harmless act
into constitutional guilt. Property, contract, finance, insurance, corporate
form, imports, exports, insolvency, and jurisdiction-shopping cannot legalise or
outsource prohibited harm. Environmental inequality uses the substantive-
equality machinery. Where an existing material floor and ecological ceiling
cannot immediately both be met, the least-harm route preserves the immediate
floor, records any ecological breach as breach, and triggers transition,
restoration, review, and non-repetition. Neither duty is renamed satisfied;
budget choice, artificial withholding, monopoly, and delay are not physical
impossibility. The ruling creates no standing ecological emergency power.

The taxonomy adds **Class 10: non-human-animal welfare and integrity**. Sentient
or credibly sentient animals are protected subjects with direct interests in
bodily integrity, continued life, species-appropriate conditions, and freedom
from severe or prolonged avoidable pain, fear, distress, deprivation, and
abandonment. Vertebrates, cephalopods, and decapod crustaceans are presumptively
covered, and independent evidence of a realistic possibility of sentience extends
the class; species, populations, and habitats remain separately protected by the
commons. Animals are not human `person`s and receive no human floor, ballot,
candidacy, political weight, or general human-equality status. They are also not
ordinary chattels whose protection an owner may waive. Custody and property may
allocate care and costs but cannot create a rescue veto.

All controlled uses carry a non-waivable welfare baseline. Lethal, invasive, or
high-severity uses additionally need a compatible serious purpose, necessity,
proportionality, no reasonably available materially less harmful means,
least-harm design, independent review, records, challenge, and a fresh temporal
contract. Deliberate cruelty, fighting, sexual use, abandonment, extreme
confinement, predictably harmful breeding, unnecessary painful mutilation, sport
or trophy killing, and severe harm solely for spectacle, gambling, amusement,
prestige, fashion, cosmetics, marketing, convenience, or profit are refused.
Every controlled use of a credibly sentient animal to produce food, including
nonlethal or non-severe production, is subject to the alternative-sensitive food
rule. Preference, tradition, convenience, and lower price are insufficient where
a safe, accessible, nutritionally adequate, materially less harmful route is
reasonably available; public transition may not withhold the food floor.
Cultural, religious, and Indigenous protection creates consultation,
accommodation, and non-assimilation duties but no exemption from the severe-
avoidable-suffering core. Research additionally requires replacement, reduction,
refinement, serious purpose, no validated alternative, public reporting, and
humane endpoints. Unrelieved severe or prolonged suffering is categorically
refused. Natural predation creates no offender or universal rescue duty.
Human-caused or controllable harm does; disease and
introduced-species control needs grave-harm evidence, exhausted feasible
nonlethal alternatives, humane means, review, and a fresh temporal contract.

The Animal Protection Advocate is institutionally separate from the Future
Conditions Guardian because ecosystem repair can conflict with an individual
animal's interests. Any person or qualified association may initiate; the
Advocate may investigate, seek rescue and interim relief, and pursue individual
or systemic remedies. Conflicts go to independent adjudication, and neither
office has a final veto. The unamendable corridor now includes direct animal
protected-subject status and the prohibitions on severe avoidable suffering and
dispensable killing; detailed standards remain democratically revisable inside
that core. Animal remedies cannot remove human standing, the material floor,
core liberty, due process, or political equality.

The ruling itself sets no numerical ceiling. Its case-level implementation is
tracked by the current-status paragraph above. Book 2 owns
measurements, numerical ceilings and budgets, inventories, models, species
standards, monitoring, restoration and veterinary capacity, and food and
research transition. Nibli does not measure ecology, decide sentience or
causation, choose a ceiling, authenticate science, or invent future
preferences. Controlling record:
`book-1/appendix/decisions/ecological-future-generation-commons-and-non-human-animal-decision.md`.

#### Public safety, defence, emergency, and external power — 2026-08-08

**Protective-power implementation — 2026-09-13.**
`book-1/appendix/contracts/public-safety-contract.md` owns the formal implementation
of the baseline below. Its explicit `public-safety` generator supplies separated
mandates, individual instruments, independent human identification and humane
holding, non-derogating emergency windows, actual justice/mobility/consent/exit
consumers, exact defect withdrawal and independent reader duties. Each instrument
has all eight floor-refusal tests; capability loss remains unread. Positive
relationship findings close the family/parent-absence confinement gap without
changing the conviction rule. Their induced credibility-cycle refusal retains
the original acceptance in an explicit counterfactual.

The delegated-approved reader revisions cover chapters 4, 7, 8, 9, 13 and 14,
the opening guide, Part V, method and hand-authored spine claims. This supersedes
the historical absence and migration descriptions below only for the implemented
formal interface. The complete verifier passed 12,860 pins across 4,190 cases
with complete contradiction scans and no findings in 1,552.00 seconds; nine
existing known-defect pins still reproduce. The five-minute target remains
unmet and is the next implementation priority.
No rule authenticates evidence, assesses actual necessity, advances a clock,
operates a force, delivers care or proves review, release or remedy.

**Non-carceral justice interface — 2026-09-13.** The separate implementation
contract is `book-1/appendix/contracts/non-carceral-justice-contract.md`; explicit
`justice` authoring supplies accessible procedures, assistance, survivor
protection, voluntary restoration, actual court-bound relief, non-coercive
enforcement, appeal, conditions and release continuity. Court consumers retain
the FSPOW_022/023/025 authority and full raw witnesses and add a justice-local
identity-conflict guard. A hearing record creates no court, conviction or
coercive power; restoration does not manufacture forgiveness. The legacy raw
assertion/withholding findings remain documented, not authenticated or silently
repaired. `justice-reader-draft.md` retains the chapter 8, 13 and 14 wording,
session-drafted and author-approved under delegated approval (2026-09-13).
Operational justice and the separate protective mandates remain outside this
bounded interface; formal verification results are recorded in its contract.

**The public-safety, defence, emergency, and external-power baselines are
author-ratified (2026-08-08): separated protective functions + no derogation
clause + jurisdiction-wide standing + civilian command.** Policing,
prosecution, adjudication, custodial execution, external defence, and security
intelligence are six distinct Class 6 mandates; **no new taxonomy class is
created** and emergency remains an overlay. Legal defence and victim protection
stay owned by the non-carceral justice interface and are cross-referenced, not
restated. Command is civilian; no serving armed-forces or intelligence member
holds a legislative, executive, judicial, or oversight seat; no secondment may
reconstitute a fused force; there is no military jurisdiction over civilians;
and civil assistance is unarmed, individually authorised, and carries no arrest,
search, detention, interrogation, crowd-control, or surveillance power. Secret
law, secret courts, and secret detention are refused.

Policing exists to prevent and respond to harm, never to maintain order as
such; a general order-maintenance power is refused. Arrest, pre-trial detention,
search, and seizure are named coercive instruments — absent from the record
today — each needing an individualised recorded ground, identification and
reasons, counsel and interpreter, third-party notification, prior independent
authorisation for search, and prompt **automatic** independent judicial review
of detention that the person need not request. Force must be strictly necessary,
least harmful, minimum, warned where feasible, stopped on achievement, and
followed by aid; the burden of lawfulness is on the public actor; lethal force
is lawful only where strictly unavoidable to protect life. Superior orders is no
defence, refusal of a manifestly unlawful order is protected, and every death or
serious injury is investigated by a body other than the deploying one.

The **categorical refusals** are torture and cruel, inhuman or degrading
treatment; enforced disappearance and secret detention; extrajudicial or
arbitrary killing; collective punishment and reprisal; indefinite detention
without charge or review; coerced confession; human shields and attacks on
people not taking part in hostilities; starvation or floor denial as weapon,
sanction, or inducement; experimentation without consent; indiscriminate or
superfluous-injury weapons and autonomous systems engaging human targets without
meaningful human control; and aggressive war.

**Emergency suspends nothing — there is no derogation clause.** A declaration
confers exactly procedural acceleration, resource redirection, compensated
requisition, and hazard-specific reviewable restriction. It cannot suspend a
right, legislate by decree, prevent the Assembly sitting, bypass a court,
postpone an election, extend a mandate, alter the franchise, or lower the floor,
and it creates no standing power between declarations. Rationing is not a new
power but an instance of the ratified physical-scarcity contract, forbidden
priority keys intact and every shortfall recorded as failure; compulsory
continuity keeps its already ratified narrow form and may not conscript a named
worker; price control is ordinary economic law needing no declaration. A managed
departure is a failure, never renamed compliance, with restoration owed. Each
declaration, renewal, and measure needs its own source-bound temporal contract
rejoining the exact declaration version; **the polarity is the same as custody
T3** — current authority is a positive premise, and its absence withholds the
restrictive conclusion. Do not call it inverse: cessation is a positive recorded
act or a fresh-evaluation claim, never derived from silence. Fail-safe evidence
rules never extend a restrictive power or cut off a claimant. Stated plainly:
this defeats a renewal never made, **not a frozen or replayed record**, and Book
1 cannot make an emergency end. A **predeclared alternate authorising route and
independent substitute reviewer** prevent whoever blocks the authorising body
from vetoing every lawful response; neither absence is approval or an indefinite
hold, and an unratified alternate authorisation ends.

Bulk, suspicionless, and population-scale collection is refused, as is buying or
exchanging what the republic could not lawfully collect. Covert measures need
prior individualised judicial authorisation, least intrusive means, defined
scope and duration, fresh authorisation per renewal, and later notification. The
**2026-08-02 temporary-assessment exclusion is extended by name** to every risk,
threat, loyalty, dangerousness, clearance, and watchlist product: none enters the
canonical consequential person record or conditions standing, floor, personhood,
franchise, liberty, remedy, or allocation. Secret evidence is never sole or
decisive; where disclosure is impossible the consequence does not follow.
Conscientious objection to a lethal or armed role is unconditional, with no
sincerity tribunal, a genuinely non-punitive equivalent, no loss of floor,
standing, franchise, candidacy or employment, and no repeated punishment.

Everyone within the republic's jurisdiction or effective control is **owed**
standing, the floor, due process, and remedy — written as a duty, not as a
record claim, because a person never entered is not recorded as missing. No
power may condition help on a record entry or treat an absent entry as a
finding. The **enforcement firewall** covers **enrolment** as well as collection
and transmission: floor access, care, schooling, courts, and crime reporting
never enrol, collect, or transmit status for enforcement. Non-refoulement is
absolute and collective expulsion refused, both inside the corridor; asylum is a
right to fair determination with advocate, interpreter and suspensive appeal;
expulsion needs an individual reasoned decision; no child is detained for an
immigration purpose and adult detention needs individualised necessity, judicial
authorisation, a maximum, and a real alternative; statelessness may not be
created and deprivation of nationality is never a punishment. Pushbacks,
evasive externalised processing, and jurisdiction-shopping are refused on the
ecological no-evasion principle — effective control, not formal territory, is
the test. A scarcity finding's **named population includes everyone within
jurisdiction or effective control**, so arrivals cannot be defined out of the
floor, and nationality, citizenship, immigration status, documentation, and
manner of entry join the forbidden priority keys. **Extradition, mutual legal
assistance, and transfer of a person** close the route around all of this:
individual judicial decision, suspensive appeal, and an express bar where
surrender would breach non-refoulement or expose the person to a categorical
refusal; a diplomatic assurance is weak evidence, never a cure.

A standing defence force is permitted, not required, with an **external-only**
mandate, an Assembly-held ceiling on size and armament, ordinary appropriation
with full audit access, and no jurisdiction over civilians. Delegated private
coercion, mercenaries, and paramilitaries are refused; private security holds no
more authority than any person. Force abroad requires prior Assembly
authorisation naming objective, basis, scope, geography, means and reporting;
an immediate defensive response is submitted without delay and **ends absent
ratification**. Aggressive and secret war are refused. A cyber or infrastructure
attack may constitute an armed attack where scale and effects are equivalent,
with evidenced attribution and the same limits. Arms transfers need an evidenced
non-misuse test; rescue, care for the wounded, humane treatment, humanitarian
access, and civilian protection are unconditional; and no one below legal
adulthood may be recruited or used in hostilities by the republic or any actor
it supports. Abuse of a protective office gets an integrity route outside the
abuser's chain, with non-recording and destruction treated as substantive
failures, and **no immunity, repose, amnesty, or pardon** for a categorically
refused act.

Treaties are executive-negotiated and Assembly-ratified, with Regions Council
and affected-region consent where regional competence or a boundary is touched;
no provisional application pre-empts ratification. No treaty, trade agreement,
or arbitration clause may lower standing, the floor, equality, due process, core
liberties, the commons ceiling, the animal core, or the non-derogable core, or
place them beyond constitutional review — **stated in the honest register the
amendment-semantics audit requires**: a rule addressed to ratifiers, reviewers,
and courts, not a claim that any current mechanism reads a treaty's actual
effect. The no-evasion rule generalises: nothing refused at home may be achieved
through trade, procurement, an affiliate, a supply chain, a flag, an arbitration
forum, or exported enforcement.

The **corridor** limits amendment; **non-derogability** limits emergency. Since
no derogation power exists, every corridor member is non-derogable and the
converse does not hold. Added to the corridor expressly: the categorical
refusals; non-refoulement and the ban on collective expulsion; prompt
independent judicial review of detention; the right to an effective remedy; and
the continued existence and capacity to sit of the People's Assembly and the
Constitutional Court. The common tier gains an express, bounded competence
addition — external representation and treaties, external defence and force
abroad, borders/entry/asylum/expulsion/extradition, external trade measures and
sanctions, security intelligence and its oversight, common minima for policing
and force, and cross-border hazard coordination — creating **no** general
security, foreign-affairs, policing, or emergency power and leaving ordinary
policing, civil protection, and disaster response with the regional and local
tiers under subsidiarity.

Oversight and remedy now join the source-bound finding route: exact readers,
action duties, positive non-response, alternates, continuity, individual
remedy, common-cause correction, affected-case re-audit, and recurrence
verification are legal conclusions. This ruling does **not** claim that a
reader receives a record, acts, corrects abuse, completes a remedy, or prevents
recurrence.

Three formal facts govern implementation. **(a) The floor firewall reaches the
confinement conclusion only** — the source discloses it, and the disclosed
non-floor control loads at zero errors — so every new coercive instrument sits
outside it unless deliberately placed inside. Each must carry its own rule
putting it upstream of personhood in the same change, plus a refusal pin for
**each** floor predicate and a re-measurement of the existing refusals.
**(b) The deprivation stays a leaf, not the instrument** — the existing
confinement conclusion is read by several rules, so an "instrument is a leaf"
invariant would be false of it; every removed capability joins the no-reader
list instead. **(c) `capture` means documented, not arrested** — its consumers
are credibility voiding and recognition, so an arrest reading would derive both;
arrest needs its own name. `severe` may not become a threat grading; `permits`,
`authority`, `free`, `travel`, `prisoner`, `public`, `defend`, `show`, and
`judge` each need explicit retain/replace/retire cards.

**Ratified but unimplemented.** Before any protective rule family lands, four
prose sites must be re-audited — the single-deprivation claim and its Part V
verdict **plus the same claim in `3-spine.md`'s hand-authored chapter list,
which no generator and no prose gate covers**; the two confinement
rule-statements, since shelter and recorded voice must extend to anyone the
state physically holds by any instrument; the absent-justifications sentence,
whose vocabulary half is false the moment justification vocabulary is admitted
while its reachability half survives by scoping the force test to the public
actor's own accountability; and the accountability endpoint. Note also that
the counted-claim gate matches neither small cardinals nor "exactly *n*", so
counting discipline here is human until it is extended — do not cite it as
this domain's guard. Book 2 owns **operation under ordinary and declared shock
conditions**: capability, doctrine, training, equipment, forensics, tradecraft,
border and reception operations, incident command, stockpiles, restoration,
procurement, and treaty and sanctions administration. Other states'
cooperation and recognition are named external assumptions. Nibli does not
detect a threat, decide necessity, proportionality, imminence or attribution,
authenticate a warrant, prove an order was given or refused, or end an
emergency. Controlling record:
`book-1/appendix/decisions/public-safety-defence-emergency-and-external-power-decision.md`.

#### Democratic and administrative integrity — 2026-09-09

**The democratic-and-administrative-integrity rulings are author-ratified
(2026-09-09): seven answers to the neutral brief's reserved questions.** A
constitutional money-and-influence record exists and is fully consequential:
a reviewed finding drawn from it — payer kind, instrument kind, recipient
kind, with the controlling payer attested where the nominal one is controlled
— may withhold public answerability for an office or candidacy, which is what
makes the three inherited treasury barriers catchable and is the Class 5
union/civic finance card the economic ruling promised. Disclosure is a
constitutional duty through the typed `obliged` bridge, with the reader's
non-response a positive duty on the alternate; its arrival half is a
liveness claim and is never written. Conflicts of interest, gifts, and
revolving doors are one family of attested incompatibilities whose bearer is
always the office, never the private party, with every amount and period
left to democratic law. A district plan drawn to entrench a named party,
coalition, or incumbent, or to dilute a protected ground, is a legal
incompatibility adjudicated on attested evidence with no metric, threshold,
or compute-backend route. Procurement integrity stays Book 2. Coordinated
information manipulation receives an actor-side finding only — coordination
and control, never truth, and never any person-side head, premise, or
vocabulary, protected by a development source guard and a watched failing
fixture. Genuine opposition rights are formalised, attached to not
supporting the government rather than to a party label, and any association
that fields candidates owes internal democratic minima; no power to
proscribe a party is created, and that refusal is recorded by name.

Every family is built on the appointment anti-capture idiom because the
corpus has no domain nouns: `observe/4`, closed `member/2` vocabularies,
`complete/3`, three mutually distinct attesters, and `authority` withheld.
Consequences reach public answerability only; the entrenched corridor is
unchanged. Polarity is positive — an absent or withheld finding manufactures
nothing.

**Formal implementation supersession, 2026-09-12.** C, A, B, D, G and F are
implemented. A now binds nominal and controlling payer identities after the
required single-case cost probe. C and A retain their examined-kind
state-form clearances and gain positive, current, independently reviewed
adverse findings. Disclosure no longer requires an unrelated prior finding;
certified reader nonresponse directly creates the alternate's duty. District
plans have their own permission and feed the electoral configuration, not
court-seat appointments. Opposition rights attach to an elected member or
group's own recorded non-support; internal democracy gates only the affected
association selection. Actor-side coordination findings require the actual
FS-POW-064 public-scale result and cannot classify people or adjudicate truth.

`book-1/appendix/contracts/integrity-record-contract.md` owns the shared finding
interface and links the family cards. `book-1/source/integrity-source.json` and the explicit
integrity generator produce rules and ordinary cases. The person-side source
guard and watched failing fixture live in the development tests; the retired
repository audit is not restored. Current incompatibilities withhold scoped
act permission, not permanent answerability, personal standing, floors or the
equal ballot. No `promise/3` or `pay/4` record is reused for political money.
The exact reader section "Keeping public decisions answerable" was approved
on 2026-09-12 and inserted unchanged in Book 1 chapter 9. Its supply is
`session-drafted, author-approved`; `book-1/source/integrity-reader-draft.md`
retains that approved wording, and the chapter's paired pins identify the
supporting integrity and disclosure cases.

Procurement process integrity remains Book 2, as do registers, audit staffing,
investigation, enforcement workflow and publication. Nibli authenticates no
payment, detects no coordination, proves no disclosure arrived, and advances
no clock. Controlling record:
`book-1/appendix/decisions/democratic-and-administrative-integrity-decision.md`.

#### Knowledge, communication, culture, and the free social field — 2026-09-15

**Implemented under the expanded mandate; no separate author ruling was
reserved for it.** The family settles what a public actor must establish before
it narrows learning and information access, expression and publication,
conscience, religion and non-belief, association and assembly, media and press
plurality, academic and scientific inquiry, artistic and cultural creation,
language and accessible communication, public information access, or sport,
leisure, friendship and mutual aid. Those fields are a closed vocabulary; a
record outside it completes nothing.

A reviewed restriction requires an evidenced rights or commons harm — those two
grounds only, and the evidence field names offence, disagreement and official
truth as what the ground is not — with least-restrictive effective means, a
defined reach and affected population, public reasons and accessible notice,
independent review with a suspensive challenge, no viewpoint, belief or
identity targeting, and an end. Enforcement is a separate dependent record that
rejoins the exact reviewed restriction, so one reviewed for a matter or a field
cannot be borrowed for another, and it reaches no floor, standing, ballot or
recognition.

**Residual freedom is written as a positive record, never inferred from
silence,** and its conclusions are duties on the public actor: not to require
permission, and not to register belief, opinion or association membership.
Public information carries access, correction and accessible-form permissions
with refusal confined to a reviewed ground; accessible communication and
inquiry/creation autonomy carry their own duties; a communication-concentration
finding is structural and actor-side, adjudicating no truth and no editorial
choice and classifying no audience or belief.

Three mutually distinct attesters plus a separate challenge reader and
independent alternate are required, none of them the acting body; disagreeing
attesters raise a record ambiguity that completes nothing. Every head is
`derived_only` upstream, so a forged entry is refused at assertion. The family
introduces no new relation name, so the spine's predicate, derived-predicate
and stratum counts and the eight floor rights are unchanged.

Book 2 owns schools, libraries, archives, broadcasters, platforms, translation
and interpretation, format production, publication workflow, funding
administration and media-market measurement. Nibli authenticates no
publication, measures no audience, detects no concentration, decides no
accessibility need and proves that no information arrived. The exact reader
section "What nobody has to ask permission for" is inserted in Book 1 chapter
8, `session-drafted, author-approved under delegated approval (2026-09-13)`;
`book-1/source/knowledge-reader-draft.md` retains the wording. Controlling
record: `book-1/appendix/contracts/knowledge-and-free-field-contract.md`.

#### Records, surveillance, and automated power — 2026-09-15

**Implemented under the expanded mandate; no separate author ruling was
reserved for it.** Nine record domains are closed as a vocabulary — identity and
status, health, care, education, workplace, housing, finance, policing and
public decisions — and so are the lawful purposes. A reviewed holding requires
authorized inputs from named lawful sources, necessity and minimisation,
privacy, security and access control, accuracy with correction, source-bound
retention and lawful deletion, subject access, challenge and anti-retaliation,
no transfer, sale or linkage outside the declared purpose, and **no enrolment
through floor access, care, schooling or the courts** — the existing enforcement
firewall restated where a record family could trip over it.

Watching and automating are separate dependent records that rejoin the actual
holding by subject, domain, holder, purpose, version, period, jurisdiction,
scope and end, so a holding reviewed for one person, domain or purpose licenses
nothing about another. Covert or biometric use needs prior individualised
authorization, least intrusive means, defined scope and duration, fresh
authorization per renewal and later notification; bulk, suspicionless and
population-scale collection is refused, as is buying what could not be
collected.

**Automated support is support.** It requires an accessible explanation of
inputs and reasoning, contest before effect, and human and independent review
deciding the outcome, with the reviewer distinct from the holder and from every
attester. A sole automated consequential decision is refused, and the
2026-08-02 temporary-assessment exclusion is carried by name: no risk, threat,
loyalty or dangerousness product enters the consequential person record.
Nothing in the family reads a computed value or concludes from one.

Access, correction, deletion and objection are bound to the subject's own record
and hand back no reusable classification. Retention concludes a deletion or
narrowing duty at the source-bound end, with no silent extension and no
recreation from copies. A reviewed defect withholds the exact use it names; a
certified reader nonresponse moves the review duty to the independent alternate.
The subject is an opaque handle and the family concludes nothing about a person:
an entry naming somebody, even beside a raw `rotten` report, takes no standing,
floor, ballot or credibility. The legacy writable surface the justice contract
records is untouched — nothing here authenticates or reinterprets it.

Book 2 owns storage, cryptography, identity technology, retention engineering,
deletion in practice, model development and evaluation, audit tooling and case
administration. The exact reader section "Keeping, watching, and letting a
machine help" is inserted in Book 1 chapter 1, `session-drafted,
author-approved under delegated approval (2026-09-13)`;
`book-1/source/record-power-reader-draft.md` retains the wording. Controlling
record: `book-1/appendix/contracts/record-power-contract.md`.

#### Physical scarcity, priority, and cross-domain conflict — 2026-09-15

**Implemented under the ratified scarcity ordering; no new author ruling was
taken, and none may be taken silently.** `FSPOW_081` and `FSPOW_082` already
carried a finding and an allocation with their review and temporal machinery.
What the rulings say most insistently and the formal source did not carry is now
there: the refused grounds by name — budget choice, price exclusion,
administrative delay, artificial withholding, monopoly, provider failure,
refusal to procure, each with its own case and each withholding the finding it
targets; the single admissible ground of authenticated, contestable,
resource-and-population-specific evidence after alternatives, reserves,
substitution, coordination, replenishment and assistance; and the express
statement that no standing emergency power follows.

**Allocation rejoins the actual finding**, so a shortage found for one resource
or population licenses nothing about another. It requires every minimum
preserved wherever usable supply permits, an effective usable equal share where
one exists, never dividing a threshold resource into equally useless pieces, and
rotation or lottery only among materially equal claims. The permitted mitigation
keys are closed — urgency, accessibility, imminent irreversible harm, continuity
harm, individualized resource-specific benefit after accommodation — and the
**fifteen forbidden keys are declared and each has its own case**: wealth,
contribution, recognition, conviction, family status, disability stereotype,
expected productivity, social usefulness, generalized lifespan, political
favour, nationality, citizenship, immigration status, documentation and manner
of entry. A recorded shortfall is recorded as failure, never as a redefined
minimum, and is no adverse fact about the person who went without.

The six named cross-domain conflicts — property against floor or commons,
expression against evidenced harm, privacy against public accountability, local
choice against portability, present claims against future conditions, emergency
action against non-derogable protection — each resolve on a rule stated in
advance, with the corridor's survival on the record. Each has its own case.

The exact reader section "When there is genuinely not enough" is inserted in
Book 1 chapter 8, `session-drafted, author-approved under delegated approval
(2026-09-13)`; `book-1/source/scarcity-reader-draft.md` retains the wording.
Book 2 owns evidence collection and assurance, inventories, forecasts,
quantities, reserves, production, capacity, queues and workflows. Nibli counts
no stock, forecasts no supply, measures no shortage and proves no procurement,
delivery, restoration or repair. Controlling record:
`book-1/appendix/contracts/scarcity-and-conflict-contract.md`.

#### Surviving guardrails and the democratic corridor

The existing guardrails survive the expansion: no floor may depend on work,
virtue, wealth, citizenship, documentation, score, compliance, or approval;
recognition remains arity-one, non-ranked, and non-operative; lawful compensation,
incentives, and above-floor insurance use separate legal relations and never read
`reward`; price, debt, property, employment, and insurance cannot condition the
floor; no provider's assertion alone proves delivery; and Part V cannot create a
right, power, or exception. The
democratic corridor is a **ratified constitutional design rule, not yet a formal
guarantee**: a majority chooses only among policies compatible with universal
standing, core floors, equal protection/non-discrimination, due process, core
liberties, commons constraints, direct non-human-animal protected-subject status,
the prohibitions on severe avoidable animal suffering and dispensable
killing, and — added 2026-08-08 — the categorical refusals on force, absolute
non-refoulement and the ban on collective expulsion, prompt independent judicial
review of detention, the right to an effective remedy, and the continued
existence and capacity to sit of the People's Assembly and the Constitutional
Court. The corridor governs **amendment**; the separate non-derogable list
governs **emergency**, and because no derogation power exists every corridor
member is also non-derogable while the converse does not hold.

### Time and temporal contracts

**T3 is author-ratified (2026-08-03) and implemented in stages for case-bound
Court custody (2026-08-05).** T1 reconciles witnessed predecessor/successor
records and selected carry. A witnessed passport selects one constitutional
lineage, and only its collision-free terminal accepted successor can give
carried adverse/clear status or public power current legal effect in a fresh
evaluation. T2 builds independently witnessed, typed event and record paths;
transitive cycles propagate conflict within their path type. T3 makes the exact
current, source-bound review a positive premise of custody authority and rejoins
the case subject, Court holder, Court judgment, injury victim, lease, window,
renewal, source, and current-record witnesses. Competing witnessed source,
window, case-subject, or lease bindings fail closed; the judgment and victim evidence
are case-bound but have no general truth or uniqueness proof. Compact
identifier-status conclusions are never sufficient alone: every
consequential consumer rejoins the exact raw tuple and its matching witness
fields. The current constitution still admits no duration arithmetic, sentence
clock, filing deadline, or operational cadence, and it cannot make a successor
arrive or prove that an outside clock advances. The reviewed contracts,
two-snapshot harness, adversarial matrix, and residual Book 2 boundary live in
`book-1/appendix/decisions/time-model-decision.md` and
`book-1/source/temporal-assurance-case.json`. Another public power requires its
own temporal contract; Book 2 continues to operate clocks, calendars, witnesses,
publication, recovery, and time services.

### Method and assurance rulings

#### The assurance portfolio — 2026-08-08

**The assurance portfolio is author-ratified (2026-08-08): seven
non-substitutable routes + one posture set + a claim-language rule per posture.**
This is a method ruling, not a constitutional one. It **upgrades no existing
claim's posture** and renames nothing inside an existing artifact. Underneath it
sits one sentence: **a route establishes what it executes, over what it was
given, and nothing further.**

The routes are formal entailment (Nibli, executed by `verify.sh`); versioned
quantitative and resource models; dynamic simulations; the claim registry;
operational assurance; reader and lived-experience studies; and repository
source-derived adversarial audit. Operational assurance is **restored by name**
because it is the only route that can carry an arrival claim and dropping it is
what lets a safety result read as "it works". R7 supplies Gate A's bounded
repository audit warrant and explicitly supplies no independent-human warrant.
A route may not be split or stretched to make a claim fit, and **one green route
may never stand in for another**.

A route is **built** when its route-owned evidence-producing check runs in
`verify.sh` — true of Nibli and the registry, and of nothing else. A structural
validator, evaluator fixture, or self-tested admission-gate component for a
dormant contract does not build its route. A route is **available** when its
evidence contract, admissibility criteria, named reviewer with the required
custody attestation, fixed in-repo gate, and watched-failing negative control
exist so outside evidence can be admitted. Both count; neither is satisfied by naming
one. Defining "built" as in-repo only would make operational assurance
impossible by construction, every arrival claim permanently unestablishable, and
the record-integrity verdict permanently frozen. **Every route must declare a
falsification condition and ship controls that must fail** — sabotage first,
trust after. The registry splits: its staleness gate covers only
script-refreshable entries, and pinned-source entries are schema-checked and
human re-cited. Do not count registry entries in prose; the ratio moves.

**A green `verify.sh` is four different things**, and the difference decides what
a claim may say. **Executable** — the pin suites and the record, temporal,
amendment, placement and counterfactual runs. **Pattern guard** — the jargon,
counted-claims, absence, floor-noticing, recognition-arity, no-counted-degree,
control-scope and evidence-vocabulary checks; one says of itself it is a pattern
guard and not a proof, another reads only a directive's spelling. **Freshness** —
the generator `--check` modes, the claim table, the registry gate, and the
reader-evidence structural and evaluator controls: artifact currency and
contract behaviour, not reader truth. **Inventory** — the operation sets, record classes and scenario
narratives: a reviewed threat model, explicitly not executable proof. Only the
executable kind warrants Derived. **Two checks are compound**: the spine check
shells the engine for its stratification and the assertion-surface check
consumes that output, so both are an engine result *and* an artifact comparison.
**`--quick` is not "proves nothing"** — it still builds the engine and runs the
stratification; what it omits is the chapter and floor pins, the record-snapshot,
temporal, amendment, placement and reader-evaluator executions, and the
counterfactuals. Say which half is being cited. No green run authenticates its
own trust root.

**The posture set is three bands.** Established: **Derived** (an executable
engine result over the exact current source, *or over a named bounded mutation
declared in a fixture or reviewed audit* — that clause is how a restriction
claim is established at all, since derivation is monotone, and such a row must
name its mutation); **Checked** (a mechanical check establishing a property of
the artifacts, never of the world and never of a semantic impossibility);
**Evidenced** (a claim about the world, with population, period, place and
method). Stated but not established: **Specified** (a complete contract, not
formalised) and **Reasoned** (argument against a named adversarial corpus,
permanently weaker than Derived and never citable as it). Not established:
**Unestablished**, carrying one mandatory disposition — routed-book-2,
external-assumption, route-unbuilt, evidence-pending, author-ruling-pending,
refused, or not-establishable. `route-unbuilt` means the assigned route is
neither built nor available; `evidence-pending` means the route is built or
available but no admitted evidence currently establishes the claim. A valid
`fail` or `not-evaluable` result remains recorded as that result and must
never be rewritten as `not-run`. Collapsing routing and refusal into one band
is deliberate: it makes "classification is a disposition, not assurance"
structural rather than exhortative. **The names avoid two words that already
carry load in the chapters, one of which inverts** — a chapter calls the vote
*guarded by refusal*, meaning the strongest protection available, while the
posture would have meant the weakest warrant.

**One posture per claim.** A claim carrying two is two claims and must be split
until each part carries one posture and one evidence kind. This disposes of
"partial formalisation", which is not a posture but an unsplit claim. Reserved
joint forms are the exception: an immutable verbatim string carries rows keyed
to its clauses.

**Three overlays answer what the gate actually asked.** A **safety** claim may
be Derived, with a supplied-records scope bound and an **enumerated**
non-extension clause — a gesture at further limits does not satisfy it. A
**liveness** claim — arrival, advance, action, release, delivery, repair — may
**never** be Derived, Checked or Reasoned; that is categorical and no scope bound
rescues it. Its only established posture is Evidenced through operational
assurance, which is not yet available, so every liveness claim is Unestablished
today: state the condition and the failure polarity, never the arrival, and
never read a duty or an alarm as evidence that anyone acted. A **feasibility**
claim needs models or simulations plus operational assurance, none available, so
it is Unestablished/route-unbuilt and **may not be written in Book 1 at all** —
a feasibility sentence with no route is a defect, not a gap.

**No Book 1 claim may take an established posture through a route that is
neither built nor available**; it takes Unestablished/route-unbuilt with
severity, consequence, owner, closure condition and public-claim restriction.
A built or available route with no establishing admitted evidence takes
Unestablished/evidence-pending; route availability alone never upgrades it.
Only sufficient admitted evidence permits the claim-appropriate established
posture, and any valid adverse result remains exact. **But building a route is
work, not a claim** — designing, pre-registering and piloting a route asserts
nothing and takes no posture. Gate C no longer waits for unbuilt R6: its claim is
narrowed to source binding, artifact integrity, navigation, internal consistency,
and mechanical accessibility. Any positive reader or actual-user accessibility
claim still requires claim-appropriate optional R6 evidence.

**No bridge into the engine.** No modelled, measured, simulated, operational or
reader result may enter through the compute backend, an external predicate, or
built-in arithmetic — it arrives only as an authenticated, adjudicated,
purpose-bound premise through a distinct relation, and a conclusion-only
predicate must still be derived. This ruling is the first to place numeric routes
beside formal entailment in one portfolio, which is exactly the composition the
oracle refusal exists to forbid.

**No aggregate score, total, percentage, coverage figure, or "N of M
established"** — Part V refuses a total in print and the reason carries here. One
conservative non-numeric rollup is permitted because the repository already
computes it: any non-refused row below an established posture yields the weaker
overall verdict. **No fourth Part V verdict token**; naming argument as a posture
classifies the method, not the verdict. **Presentation amended 2026-09-19,
revision item 11:** Part V retires the recurring three labels and states
reasoned conclusions directly; it adds no fourth label, score or stronger
claim posture. The current ruling in `assurance-portfolio-decision.md`
records this supersession. **No rewording of the byte-exact
artifact verdict string**, which its generator enforces, its reviewed source
stores, and the record-integrity case cross-references by literal needle.

The mapping table maps existing vocabularies onto the ceiling **without
renaming anything inside an artifact**, and a closure rule requires a mapping row
for every consequential enum. The **claim-assurance ledger is a generated
projection of the one canonical source**, which landed at stage 1 on 2026-08-09
(`full-society-ledger.json`, generated and gated by `13-full-society-ledger.py`)
— posture, route, evidence kind, scope
bound, disposition, owner, severity, closure condition, claim restriction and
mutation reference are fields on that source's claim records. A second
hand-maintained matrix of assurance truth is refused by the ratified
canonical-source mandate, in those words. One practical hazard when editing the
planning files: reviewed JSON references use `path::literal needle` and the
generators require each needle to occur **exactly once** in its target, with the
tracker and the coverage map carrying the most. Re-check needles mechanically
before committing an edit to either.

The ruling itself created no predicate, rule, pin, generator, verifier section,
chapter, or public claim, and reclassified nothing. The canonical source landed
at stage 1 on 2026-08-09, and each artifact keeps its own vocabulary
**permanently**: the ledger's mapping table binds every consequential reviewed
enum value to the canonical postures without renaming anything, and its
generator re-reads the sibling reviewed JSONs live, so a new reviewed enum
value fails `verify.sh` until its mapping row lands in the same change. The
controlling record is `book-1/appendix/decisions/assurance-portfolio-decision.md`.

#### The narrative register — 2026-08-08

**The narrative register is author-ratified (2026-08-08): it stays flat, and the
balance problem is a constitutional deficit rather than a prose one.** The four
approved channels remain a **closed list** and this ruling disposes of all four,
not only the passage: the author's first person is unchanged; the single
second-person domestic passage is retained under the rule below; the hostile
reviewer corpus stays Part-V-only and is **not** claim-free, since a quotation
carries its own posture and attribution; and the historical cases stay
Part-V-only, capped at their sourced members, and are **explicitly not a
substitute for derived ordinary life** — they are evidence about other societies
and how they failed. "Do not warm the cast in any pass, any part, ever" stands
unamended, as does the reading that the flatness is chosen rather than awaiting
repair.

**The register ruling amends the two 2026-08-02 rulings in exactly two ways, and
both are named rather than silent.** (a) The passage channel **widens** from
Part V — which the flatness ruling scoped it to in its own words — to **the
opening note and Part V**, because both are exempt elements in the first-person
channel. (b) **`method.md` is excluded despite being exempt**, not because it is
less exempt but because its scope is sealed by five decisions and its reading
contract is what keeps a documented renderer defect dormant. Derived chapters
stay closed by three standing rulings; no fourth exempt element exists. (R4 of
*The rebuild of Book 1*, 2026-09-16, overrides this narrowly for the directory, not
the book: `book-1/appendix/` is a carried archive, and the book's exempt elements
remain three.)

**Superseded in part 2026-09-24, not yet implemented (D2 and D3 of *The revision
rulings D1–D9*):** each derived chapter will close with one labelled argument
section, and documented cases may also appear in the opening, a labelled case
opening each Part, and those argument sections. The flat register, the ban on
inner lives and the refusal of composite citizens and dramatised scenes stand.

**Superseding prose-supply protocol — author-approved 2026-08-20.** Sessions may
draft the Book 1 manuscript, including its epigraph, opening note, numbered
chapters, Part V, and method, plus the launch essay and Book 1 companion prose.
The lifecycle is `session draft → exact displayed candidate → explicit author
approval of that version → canonical prose`. Any substantive wording, meaning,
or voice change requires renewed approval; mechanical Markdown wrapping or
formatting does not. Canonical session supply is recorded as
`session-drafted, author-approved`. Book 2, legacy `book.md` and `manifesto.md`,
and planning prose remain outside this drafting scope. All derivation, sourcing,
register, evidence, pin, Book 1/Book 2 seam, and verification rules remain
binding. `tmp.txt` is optional noncanonical scratch space, never the approval
record. This protocol supersedes only the older author-as-drafter and
ghost-writing dispositions; it does not widen any narrative channel or bypass
an approval or evidence gate.

**Superseding delegated approval — author instruction, 2026-09-13.** The author
pre-approves all recommended suggestions and asks the agent to implement,
verify, commit and push one TODO item at a time, repeating through the backlog
without asking questions. This includes the already displayed mobility prose
and recommended future prose: retain the exact text and record it as
`session-drafted, author-approved under delegated approval (2026-09-13)`.
Separate per-passage approval pauses are no longer required. Derivation,
licensing, register, sourcing, scope and verification requirements still apply;
never invent author memories, external evidence, reviews or operational success.
Record genuine blockers honestly while continuing other in-scope work.

**The passage rule** is form plus assertion. *Form:* generic second person,
never a name; **no interior state attributed to any person in the scene**,
though a force named in the negative is permitted, because naming what cannot
reach a household is a statement about the design rather than about a mind; no
dialogue, backstory, named place, or time-course beyond one scene, optionally
paired with a contrast scene in the reader's present society; and it closes by
re-declaring itself as argument rather than scene. *Assertion:* every concrete
detail traces to one of **three** permitted targets and says which — a
constitutional rule or fact, a derived chapter's fixed gloss of a floor item, or
a pinned fixture; **an arrival may appear only inside the hypothetical frame and
only where the same sentence names the entitlement that grounds it**; and the
disclosure that the supplied record shows no floor arrival outside confinement
travels with the passage. The slot sits inside the jargon sweep and the
counted-claims hard gate.

**The mood is claim-free; the predications are not.** "Imagine a household"
asserts nothing, but a definitional clause about the designed society, a
description of the reader's present world, and a universal negative are all
predications. The resolution is a **restatement-only register**: a passage may
restate only claims that already carry a posture elsewhere in Book 1, **no claim
may make its first appearance in an exempt-element passage**, and a passage may
never be the sole support for a ledger row, a coverage field, or a public claim.
It therefore adds no ledger row. A contrast scene about the reader's present
society is rhetorical contrast carrying no evidentiary weight, explicitly
outside Part V's sourcing discipline.

**The pre-repair passage remains recorded as dated history.** Its per-clause
trace is in the decision. Two results mattered: "the flat is warm and the water
is clean" traced to **chapter 8's gloss, not the constitution**, and did not name
the housing entitlement in the same sentence; "the record can hold an
accusation only as an entry with an author" **traced to nothing** because the
harm relations name the alleged offender and victim, expressly not the writer.
It was dated **non-conforming on arrival, trace and proximity as of
2026-08-08**. `OL-15-v1` was mechanically transcribed on 2026-08-20 and is the
current canonical, `session-drafted, author-approved` passage. It names the
housing entitlement in the arrival sentence, removes the unsupported writer
identity without claiming to close `FS-DFT-19`, and places the record-bounded,
outside-confinement delivery disclosure beside the scene. Section 7a of the
narrative-register decision controls its current conformance disposition.

**Delivery-interface supersession — 2026-08-26.** The narrative-register
decision preserves the 2026-08-08 deficit as history and now records the
current route-without-arrival boundary. `FS-CVF-015` is formalized and
prose-landed. Recipient-side routes derive food, shelter,
care, material security, and company from a matching receipt plus an authorised
independent witness; the legacy teaching route remains separate. The new routes
ship dormant, belief and the liberty of expression deliberately receive no
delivery head, and the supplied record still derives floor actualities only
through confinement. This repairs the formal-interface deficit without proving
offer, accessibility, adequacy, operation, actual-world arrival, completed
remedy, or future non-recurrence. The pinned-case portfolio rebalance's delivery
precondition is satisfied; the rebalance remains open. The separate
accusation-authorship gap also remains open.

On the reader-balance gate, three statements in this order, because the shorter
version is an overclaim: the deficit **does not by itself** make Gate C
unpassable, since its claim is protocol-relative; this ruling makes **no
prediction** about the outcome; and the deficit is a named, **disclosed input**
to the protocol's design, which may not be written so as to hide it. In the
reader-experience ledger a passage is enrolled for completeness, occupies no
person-posture or domain cell, contributes zero coverage, and reads as an
exempt source — otherwise one illustration would mask the deficit the ledger
exists to expose.

Refused by name: composite citizens, an invented antagonist, dramatised cast
scenes, warming the cast "slightly" later, a fourth exempt element, boxes inside
derived chapters, inferring an inner life from a record entry, a passage
introducing a claim that appears nowhere else, and the historical cases as a
substitute for derived ordinary life. The controlling record is
`book-1/appendix/decisions/narrative-register-decision.md`.

#### Reader-balance evidence protocol and threshold timing — 2026-08-09

**The reader-balance evidence protocol and its threshold timing are
author-ratified (2026-08-09): pre-registered pilot-and-fresh-holdout + the pass
rule's form now, its values after the pilot + a specified-but-unbuilt reader
route.** Summarise it as three statements, never one, because the shorter
version is an overclaim: the method, timing, disclosure, and ethics terms and
the pass rule's **form** are ratified; the rule's **values and severity
taxonomy** remain author-ruling-pending, reserved for a second ruling; and Gate
C's holdout condition is therefore not yet satisfiable, with every reader
comprehension, balance, and lived-effect claim keeping
Unestablished/route-unbuilt. Do not write "the reader-balance protocol is
ratified" bare, and note the roster's shorthand: this is "the reader-threshold
decision" that deliberately does not settle the threshold.

As of 2026-08-11, `reader-evidence.json`, its generated report, structural
checker, deterministic evaluator, and the fixed admission-gate component
implement dormant machinery only. Machine state is `threshold_status:
pending-pilot`, `holdout_status: not-frozen`, and result `not-run`. No
taxonomy label, threshold value, active completed attempt, or
`gate_admission_receipt` exists. Building and self-testing the gate component
does not make R6 built or available; FS-CLM-37 remains
Unestablished/route-unbuilt and Gate C is unchanged.

**Reader-evidence execution was withdrawn from the current Book 1 program by
the author on 2026-08-11.** No pilot, holdout, post-pilot threshold ruling, or
human screen-reader smoke test was run. R6 remains optional and unbuilt, and no
reader-comprehension, suitability, lived-effect, or actual-user accessibility
claim follows. The 2026-08-15 no-external-reviewer ruling supersedes the former
Gate C blocking consequence: Gate C now tests only source binding, artifact
integrity, navigation, internal consistency, and mechanical accessibility.
The dormant reader protocol may be revived as optional evidence without
becoming a publication dependency. Controlling records:
`book-1/appendix/decisions/reader-evidence-execution-withdrawal-decision.md` and
`book-1/appendix/decisions/full-society-boundary-decision.md`.

The Reader's Map, annotated contents, glossary, indexes, domain map, and
prose-equivalent diagrams landed in the exempt opening note at `67a520e`. The
public-minimum pilot kit and snapshot builder landed at `c8317ac` as draft
templates and tooling. Script 15 checks semantic source headings, local links,
and deterministic HTML/EPUB generation only. Automated artifact validation
does not warrant human accessibility; the human screen-reader test was
withdrawn rather than passed. No pilot snapshot or instrument is frozen or
pre-registered, no participant evidence exists, and R6 remains unavailable.

If a later author ruling revives the route, the sequencing rule construes the
dormant protocol as follows:
pre-registration binds **each round's instrument before that round**. The pilot
runs under a pre-registered pilot instrument carrying at most an expressly
provisional draft rule and draft severity taxonomy; the reserved ruling lands
**after the pilot and before the fresh holdout's pre-registration freeze**,
ratifying taxonomy and values together on pilot evidence; a holdout run under
an unratified, retroactive, or post-hoc rule is **void** and cannot feed Gate
C; and the route must be **available** — implemented evidence contract, named
reviewer with gate-bound custody, fixed digest-bound admission gate, and
watched-failing seeded control — before the holdout runs. Every active completed
attempt must then store the gate's exact `gate_admission_receipt`; only
`decision=admit` may establish FS-CLM-37. The assurance
portfolio's sentence that the pass rule "is itself a separate open ruling to be
made after a pilot" **remains true after this ruling** and must not be edited.

The form is a severity-weighted misconception rule defined non-numerically: no
aggregate may hide a core misconception — a core finding cannot be offset or
averaged away. Whether a single or repeated core finding fails is deliberately
reserved with the values, and the record contains **no threshold number, weight,
sample size, or example value**, because an illustrative number would anchor
the reserved ruling. The identification targets — ordinary constructive life,
democratic choice, private freedom, successful provision, repair, and the
prisoner as a stress test — are the **minimum pass-relevant set, not a closed
list**; the six unaided prompts and Gate C's own identification and distinction
conditions remain independent gate conditions the protocol does not discharge.
The deficit stays a named, disclosed input, and the disclosed-limits minimum is
fixed: the unimplemented-families sentence, the tested snapshot's exact version
identity, usability-not-population-statistics, the sampling and method limits
bounding Gate C's permitted claim, and that no reader result enters the engine
or proves another route's domain.

The reader route's availability contract is **specified and structurally
represented, not fulfilled** — recording a state space does not build it.
`reader-evidence.json` is the sole machine-readable source for reader-study
states, locally hash-chained and Git-transition-checked attempt history,
`gate_admission_receipt`, taxonomy, rule values, ratification basis,
`frozen_ratification`, and holdout pre-registration. Its checker validates
state, completeness, digest relationships, and allowed transitions over visible
normal first-parent Git history; its deterministic evaluator recomputes receipts
from a ratified rule and admitted coded outcomes. Those are artifact checks, not
reader evidence. The fixed gate is a separate component, and none of these
artifacts proves resistance to rewritten Git history or attests to external
freshness or custody truth.

The public record is exactly opaque study IDs, coded target/misconception
outcomes, artifact or commitment digests, coded deviations, and custody
attestations without identity material. Participant, session, coder, reviewer,
and custodian names, pseudonyms, identifiers and identity mappings; raw or
free-text responses; consent and withdrawal material; and direct contact,
demographic and accessibility records remain in private custody. Admissibility
is binding, not guidance: an ethics
breach — consent, withdrawal, data protection, accessible participation, fair
compensation, non-retaliation, trauma safeguards, independent review where
appropriate — makes a session inadmissible regardless of results; holdout
participants must be fresh (no exposure to drafts, previews, the pilot, or the
reviews corpus; pilot participants excluded); and the in-repo reviewer corpus
is **never admissible reader-study evidence**, remaining Reasoned design input
in the exempt elements. The falsification condition is declared: the
instrument must fail against a seeded unbalanced or planted-misconception
control, watched failing during the pilot's revise step. The checker's schema
and state mutations are not that seeded misconception control and do not satisfy
R6's negative-control prerequisite. The structural checker, deterministic
evaluator, fixed digest-bound gate, reviewer custody attestation, and seeded
control are distinct. The gate component is built and self-tested, but no
component test supplies an attempt's `gate_admission_receipt` or makes R6 built
or available. No reviewer is named and no gate-bound reviewer custody
attestation exists.

The pilot runs against a declared, versioned snapshot only after the Reader's
Map and navigation artifacts exist, and is evidence about the instrument and
that snapshot only. Every pilot or holdout freeze requires an external
prior-commit or custody binding whose public receipt carries the computed
`attested_payload_sha256`; a generic attestation is invalid, while external
existence and truth remain outside the checker. The holdout binding covers the
exact rule digest, revised instrument and rubric, private release-candidate
identity and artifact hashes, sample and recruitment rule, disclosure set,
study protocol, and preregistration `structural_checker_sha256`. Every holdout
attempt embeds `frozen_ratification`, including its independently validated
digest, rule, candidate, and pilot basis. The candidate commit must be an
ancestor of current `HEAD`. Each successor pilot or holdout pre-registration
binds `predecessor_attempt_sha256` and `prior_history_head_sha256` to its
frozen predecessor; those fields are null only when no predecessor exists.

If publishing the instrument would contaminate recruitment, a nonce-protected
commitment is held under named private custody and its preimage is revealed
after the holdout. Freeze, `completed_at` or `voided_at`, `revealed_at`, and
successor-attempt times are strict canonical UTC and ordered: freeze precedes
completion or void, reveal follows it, and a successor starts only afterward.
Any bound change voids that attempt and requires a new pre-registration and
genuinely fresh sample.

Attempts retain local hash chains and an active-attempt pointer, but a
snapshot-local chain is not enough. The root `history_transition` records
`previous_source_commit`, `previous_source_sha256`,
`previous_history_head_sha256`, and `history_head_sha256`. Script 14 binds
those fields to the nearest earlier normal first-parent commit that changed
`reader-evidence.json`, its exact source bytes and history head, and the
current history head. It preserves every prior attempt prefix and terminal
attempt and permits exactly one domain and one legal step per transition: one
pilot or holdout append, or the active nonterminal attempt becoming terminal.
The dormant source has null predecessor fields and a deterministic empty history
head. This validates visible normal first-parent Git transitions only; it cannot
prove resistance to rewritten Git history or external truth.

`holdout_status` follows the active attempt; the top-level result is the most
recent completed non-void outcome and persists independently. A `void` attempt
with `not-run` is valid. A later frozen, not-run, or void attempt never rewrites
an earlier valid `fail` as `not-run`; every attempt remains recorded, and no
void attempt can feed Gate C. Study IDs, coded-record commitments, custody IDs
and digests, and receipt IDs are globally unique across pilot and holdout
history, and every run carries exactly one freshness record.

Once values exist, end-to-end below-, exact-, and above-boundary fixtures are
derived from every value at reachable observations. Unreachable or out-of-domain
edges are explicit and fail closed. Candidate, `author-ratified`, frozen, and
completed stages each carry the watched-failing mutations relevant to that
stage; missing, empty, or inapplicable controls fail.

This ruling's former Gate C dependency is superseded by the 2026-08-15
no-external-reviewer amendment. It now defines an optional R6 reader-balance
protocol only. Mechanical accessibility remains separate and cannot be
reported as reader comprehension or actual-user accessibility.

Refused by name: an aggregate score deciding a pass; word, chapter,
demographic, or sentiment quotas and any fixed prisoner quota; population
statistics from usability evidence; reader evidence proving another route's
domain; any reader result entering the engine; declaring the route built or
available; predicting any session's outcome; and threshold values or
illustrative numbers before pilot evidence exists. The empirical protocol
remains **unexecuted**: the dormant reviewed source, generated report, structural
checker, deterministic evaluator, and fixed gate component create no frozen or
as-run instrument, session, threshold, reviewer, admitted evidence, predicate,
rule, pin, chapter, established posture, or public claim. A built gate component
is not a built or available R6 route; FS-CLM-37 remains
Unestablished/route-unbuilt and Gate C is unchanged. The controlling record is
`book-1/appendix/decisions/reader-evidence-protocol-decision.md`.

### Enacted machinery — generated and verifier-enforced

**`lose` and legacy `decide` are relation-wide conclusion-only — L1 + D1,
author-ratified and enacted 2026-08-04.** Both had been closed only by omission:
adding the exact `admits("lose")` or `admits("decide")` declaration made the
corresponding ground conclusion writable. Article 0 now reserves each relation
at every arity and through any future converted alias. Rule heads remain open:
the existing clawback and franchise rules still derive, and a hostile rule can
still derive either conclusion. `lose` remains an unread leaf with no amount,
proportionality, restoration, or remedy. `decide(_, Ballot)` is the current
franchise conclusion, **not** a cast ballot, delivery fact, tally, election, or
legal result; those interfaces need distinct names, and democratic redesign may
later retain or atomically replace the legacy relation. This is not a complete
closure sweep. The floor actualities, `obliged`, and `travel` remain
unadmitted and not conclusion-only pending their delivery, duty, and liberty
contracts under the generated assertion-surface audit. The controlling record is
`book-1/appendix/decisions/closure-gaps-decision.md`.

**The real symmetries and the necessary asymmetries are named, and the one that
is mechanical is checked — 2026-09-15.** Four recursive interfaces run through
every landed family, and naming them is what stops each new family reinventing a
weaker shape: right → duty → accessible delivery → breach → continuity → remedy
→ review → corrective control → monitored recurrence; power → lawful
source/trigger → evidence → limit → public reason → independent review → appeal
→ correction/end; harm → notice/voice → due process → least-coercive response →
repair/release; and democratic choice → authenticated mandate → bounded
implementation → public feedback → challenge → correction or peaceful
replacement. Each landed family's contract card states which of the four it
instantiates and where it stops.

**One of them is measurable across families and now is.** Every three-place
record completion in the constitution — 598 of them, across eleven families —
carries at least one distinctness constraint holding two roles apart and at
least one independent review authority. There are no exceptions, and
`floor_vector_tests::every_power_record_is_independently_reviewed` fails if one
appears. The challenge-reader and independent-alternate half is a **later
convention**, and the test says so rather than pretending otherwise: eight
families carry it on every completion, `ECONOMIC-CONSTITUTION` and `STATE-FORM`
predate it and carry it on none, and `AMENDMENT-ENACTMENT` is partial. Those
three sets are asserted by membership, so retrofitting an older family is
welcome but is a ruled change that has to move its contract card too.

**Narrowed 2026-09-24 and implemented the same day (D6 of *The revision rulings
D1–D9*, item 43):** an act that only gives or preserves something for its
subject takes effect on one authorised actor, with prompt independent review
able to withdraw it. Its completed record still carries both guards, so this
test still holds; appointments giving power over another person and every
adverse act keep full prior procedure, and
`help_takes_effect_on_one_actor_and_harm_waits_for_review` states that half.

**The asymmetries, and which are formal.** Recognition is optional, binary,
non-ranked and non-operative — checked, by arity, by no reader, by no self-join.
Punishment is coercive and carries a higher threshold — carried by the
conviction rule's own structure and its chapter pins. Public power is
presumptively reason-giving and auditable while private life is presumptively
private — the reasons, review and challenge fields on every power record, against
the knowledge family's residual-freedom duty running the other way. Accessibility
may require unequal resources to secure equal standing, and children, dependants
and people needing support hold rights without symmetric duties — **neither is
formal**, and neither should be read as checked: the first is a resource claim
this repository cannot make, and the second is the floor's unconditionality
above `person`, which the firewall protects but no test states as an asymmetry.

**Psychological claims are grounded by refusing to name a psychological state —
2026-09-15.** The item's third requirement is formal and now checked: no duty
among the 313 named duty constants compels or certifies believing, eating,
learning, accepting treatment, a relationship, fulfilment, satisfaction,
happiness, trust or loyalty, and no second compliance duty exists. The one duty
naming compliance is an institution carrying out a court's final protective
direction and is allowlisted by name.

**Where the guard is, and where it is not, is stated rather than implied.**
`happy` and `deserve` are corpus names, so keeping them out of the rules is this
test's doing. `trust`, `wellbeing`, `satisfaction`, `compliance`, `motivation`,
`attitude` and `loyalty` are **not** corpus names — the closure refuses them a
step earlier, and the test asserts that too, so it fails rather than passing
silently if the corpus ever admits one.

The item's first requirement — testing for autonomy, voice, non-humiliation,
relatedness, meaningful control, retaliation, status competition, coercive
incentives, learned helplessness, trust, care burden and the effects of being
watched — is **empirical and its route is unbuilt**. Nothing here tests any of
it, and no formal proof or service record may be read as evidence of any of it.
That is the second requirement, and it is a claim discipline rather than a
check.

**Claim-type discipline is checked where it is mechanical, and its limit is
measured — 2026-09-15.** Every historical case Part V argues from is bound to
its registry entry and to a phrase that must still appear in the prose, so a
figure cannot lose its source and a source cannot be dropped while the book
still leans on it. **Ruled the same day under delegated approval: Part V's
figures stay hand-written.** Inline registry ids would put machinery into an
exempt element whose point is that it reads as argument, and the handful of
figures does not justify a rendering step with no other consumer.

**The counted-claims rule is guarded on its digit half only, and the test says
so.** No derived chapter carries a digit — exact, currently zero, and
sabotage-tested. The spelled-out half is **not** guarded and must not be read as
guarded: measured on 2026-09-15, the derived chapters contain 60 uses of a
cardinal beside an inventory noun — "two witnesses", "three routes", "one
person's word" — and every one states a rule rather than counting the record's
contents. The banned shape is "four people have shelter"; the difference from
"it takes two auditors" is semantic, and an allowlist of 60 sentences would rot
faster than the prose it guarded. That half is prose review.

**Claim-scoped resolution receipts replace confession-as-ending — 2026-09-15.**
Twelve threads where the book identifies a defect, claims a repair or uses a
failure as a witness each end in exactly one of five states:
`resolved-for-claim`, `operationally-unresolved`, `externally-bounded`,
`irreducible-limitation`, `open-defect`. Naming a limitation and moving on is
not among them. Every receipt states what failed, what changed, how the former
attack is rerun, what still does not follow and what remains external or open,
and `./generate.sh resolution-receipts` projects the report.

**Three checks make it hard to cheat.** A `rerun` path that does not exist fails
— a repair with nothing to rerun is a claim, not a receipt. A `phrase` the book
no longer contains fails, so a receipt cannot outlive the thread it describes.
And **a chapter that narrates a repair with no receipt fails**, using the book's
own idiom for telling one, so a repair cannot be claimed in prose and left
without an ending. All three are sabotage-tested.

Five of the twelve are `resolved-for-claim`; the rest are not, and the report
gives disclosure no credit for closure. The generator refuses a table in which
everything is resolved, because this design has never been in that state.

**The mechanically testable accessibility checks are complete — 2026-09-15.**
Script 15 now validates the generated HTML's document language, its skip link
against a target that must exist, a text alternative on every image, non-empty
accessible names, an accessible name on every focusable region, and reading
order: every ordered input contributes exactly one top-level heading and they
appear in manifest order. Six accessibility mutations are watched failing beside
the four missing-link and four stale-PDF controls.

These are **properties of the artifact and of nothing else**. Human
screen-reader validation was withdrawn at `907ddd0`, remains optional evidence
rather than a gate, and no accessibility-for-users claim follows from a green
check. Binding a preview snapshot's exact HTML, EPUB and PDF identities waits
for Gate B and that snapshot's own gate.

**The reader-experience coverage ledger is landed — 2026-09-15.**
`reader-coverage-source.json` classifies all 86 derived-chapter and Part V
passages — domain, rule family, ordinary or protective function, setting, person
posture, trajectory, basis — and `./generate.sh reader-coverage` projects
`reader-coverage.md`. An unclassified passage, a record with no passage, a basis
path that does not exist and an unknown trajectory each fail, all four
sabotage-tested. No non-justice domain is explained only through custody.

**Its value is the gaps, and its first reading of them was wrong.** Forcing one
trajectory per section made eight domains look uncovered. Whether a passage
states a boundary is now read out of its own prose — what it says it does not
establish, matched in 55 of the 86 — and seven of the eight turn out to meet the
standard that way. A stated boundary is the weaker of the two accepted forms, so
it is counted separately, and **that set is now empty**: every domain the ledger
classifies carries at least one passage in which something goes wrong. The
assertion stays as an empty expectation, so a domain landing without one fails
the check rather than passing quietly.

**The seven passages take the shapes their own families support, which is why
none of them is a template.** Knowledge and culture left when chapter 14 gained "When
the finding itself turns out to be wrong". Emergency and defence left together
when chapter 9 gained "When the emergency broke the rule it was under" and "When
a public power was handed to a private force": a declaration found to have
derogated or governed by decree takes its requisition with it, and with the
requisition the restraint, the recorded loss and the operator's duty to
inventory, return or compensate; a defence structure found to have delegated
public coercion to a private actor takes the authorisation for force abroad and
its own ceiling-and-audit duty. Each is measured on the family's own withdrawal
cases, and **the accounting duty falling with the measure is printed rather than
smoothed** — the design can say a taking had no authority and cannot, from that
alone, establish that anything came back.

Collective and plurality rights left through the mobility family's withdrawal
sequence, where a reviewed defect against a consent record contradicts the
effect permission, lands preserve/correct/remedy and audit duties on the reader
and the auditor, withdraws both records, and leaves an unaffected record of the
same kind standing. Borders left through the **other** shape, and it is the one
worth knowing: an asylum determination completes, an authorised writer records a
second constitutional version on a single-valued scope, and the completion and
the duty to provide a fair determination both stop, because nothing chooses
between two authorised writers. **The cost of that refusal has a direction and
the passage says so** — the claimant waits, the writers who created the
ambiguity do not. Measured on both families' own fixtures: `false`, `prisoner`
and `lose` derive for the person at no point in either sequence.

Ecology and non-human animals left last. The ecological record defect withdraws
**only the exact affected reliance** — no safety, guilt, execution or repaired
outcome is inferred, the lawful history and the private evidence stay, and no
human standing, floor, ballot, liberty or collective right, and no animal's care,
is lost. It needs positive independent evidence of the exact defect and no
acknowledgement from whoever wrote the record, so an original writer cannot veto
a finding about their own record. And **a correction is not renewed authority**:
the replacement has no force until its own complete fresh contract derives. The
animal passage is the other half — the particular orders after an adjudicated
harm, each carrying its own boundary (cessation names the activity and not an
occupation; rescue and care are not conditioned on the owner's consent or papers;
custody is of an animal, never detention of a person; disqualification touches no
standing, floor, vote or unrelated private life) — and every order carries the
clause that no performed rescue, treatment, transfer, restoration or delivery is
inferred from it.

**The portfolio's postures are censused on the same source, and two are
thin.** The first reading said `creates` and `cares` were empty and was wrong
twice over: one posture per section, and the chapter preambles unclassified —
which is exactly where chapter 10 does its care work. The ledger now covers all
109 passages, preambles included, and a passage may hold several postures.
Every posture is occupied; `cares` and `creates` are carried by two passages
each, against 53 for `is acted upon`. `cares` gained its second when the
life-course baseline's ordinary half landed, which is the pattern to expect: a
thin posture is usually a missing interface rather than a missing paragraph. The empty set and the thin set are both
asserted by membership.
Separately, the delivery routes are now exercised rather than only described:
`tests/pins/delivery/received-outside-custody` derives all five recipient-side
floor actualities for a person nobody convicted, so the pinned portfolio no
longer reaches the floor only through a cell. The routes still ship dormant in
the shipped cast, and nothing establishes that anybody was fed or housed.

**The four chapter patterns are measured on the same source.** Every passage
declares which it follows — constructive provision, protected private/civic
agency, democratic agency, coercive rule, plus the two this book's own shape
adds: the record chapters and Part V's argument. All six are exercised. Coercive
passages are 18 of 86, and provision, private life and democratic agency
together outnumber them more than two to one, which is what keeps the prisoner a
stress test rather than the default inhabitant. A chapter being of one pattern is
the design — each has a subject — so the assertions are about the book's shape,
not the chapter's. **Whether a passage follows its pattern's own arc — seeks,
responds, receipt, challenge, continuity, boundary — is prose review and is not
checked.**

**The one real gap was constitutional rather than editorial, and it closed the
right way round.** Life course, family, care and reproduction appeared only under
strain because that baseline landed 106 person-held barriers and no
ordinary-operation interface — nothing derivable for a passage to show working,
and writing one would have fictionalised the coverage. The interface landed
first (`FAMILY-LIFE-ORDINARY`, 2026-09-16) and the passage followed it. **Keep
that ordering when the ledger next shows a domain only under strain**: the empty
expectation in `reader_coverage_tests` exists so the next such arrival fails the
check rather than being written around. The portfolio-rebalance and
chapter-pattern items consume this table; the ledger closes nothing and does not
claim to.

It is a map of what a reader meets, never of what a reader understood. No reader
has been asked anything, R6 remains unbuilt, and nothing in it is reader
evidence.

**The multidisciplinary adversarial audit is encoded — 2026-09-15.**
`adversarial-audit-source.json` binds all fifteen declared lenses —
constitutional law, public administration, disability and accessibility, public
health, care and the life course, labour and economy, consumer and civil
justice, policing and prison, media/science/culture/pluralism, local, migration
and collective governance, defence and external affairs, infrastructure,
ecology, data and AI governance, and quantitative modelling — to the checks and
cases that encode them and to what each finds. Four ways to weaken it fail and
are sabotage-tested: a lens naming a path that does not exist, a lens finding
nothing, a declared finding kind nobody raises, and a table with nothing open.

**Thirty-two findings across the seven declared kinds, nine open**, printed
first in the report. It is a repository audit over the source: it warrants no
independent human review, no reader response, no external truth and no
operation, and external submissions remain welcome optional evidence that no
gate waits for.

**The red-team index is landed — 2026-09-15.**
`book-1/appendix/maps/red-team-index.md` has one entry per named strategic
behaviour, each stating who gains, what they must know and coordinate, who bears
the cost that does not appear on the ledger, how it is detected or challenged,
and whether the design's answer creates a new veto, surveillance system or
score. Ten entries name an executable case for the wall they claim.

**A second entry said a different wall was not there; measuring it moved the
claim, and then one route was repaired — 2026-09-16.** An accusation nobody
signed is not inert. Two routes still take something on their own: an unsigned
deceit entry takes the whistleblower shield for that exposure — per claim rather
than per person, so a second unaccused exposure protects again — and stops the
accused being recognised. Both **withdraw by absence** rather than concluding
anything, which is why they are described rather than repaired: an accusation
not yet adjudicated is what the shield was built to ignore. The voiding rule
asks for a review body's judgment beside the lie.

**Severity is now court-bound**, and this is the ruled change. Two unsigned harm
entries used to derive `severe` outright, and severity decides where a confined
person spends the sentence. It now also reads `judge(Court, ·)` and
`cite(Court, ·, ·)` — **the two facts the confinement it decides had already
required**, since every consumer of `severe` requires `prisoner` and `prisoner`
requires both through the T3 gate. The repair therefore cost nothing on any path
where severity has an effect: Lalo, Don and Ruk keep it and are pinned
unchanged, while an entry about somebody no court has judged now concludes
nothing. The spine did not move, because conjuncts on existing rules add no
rule. `tests/pins/red-team/counterfactual-severity-without-the-court` is the
watched control, and `tests/pins/red-team/an-accusation-nobody-signed` runs the
routes in sequence with the person keeping personhood, the floor debt and
liberty throughout.

**It gives no accusation an author, and every statement of it says so.** The
harm relations still name the offender and the person harmed and never the
writer. The court is a condition on acting, not an author for the claim, and the
finding-with-no-finder class chapter 1 concedes is still open. Chapters 1 and 11
both carry the change; the deceit-route repair remains unspent because it would
flip `false(Lupo)`, which two chapters exhibit.

**The reader set is censused and asserted by membership**, by
`floor_vector_tests::an_unsigned_accusation_reaches_exactly_the_measured_set`
with a sabotage control — thirteen rules, each with the polarity it reads at. It
gives no accusation an author; it stops the set of things an authorless one
reaches from growing unnoticed. **It also found the row a hand census had
missed, which is the argument for having it**: `prisoner` reads an injury entry
inside the conviction rule, behind a Court judgment, a cited case, a recorded
conviction, twelve independently witnessed observations, an active custody
authorisation and four negative guards. **That row is what produced the severity
repair** — the same relation sat at both extremes, surrounded where liberty is
taken and unguarded where the severity deciding the confined person's
destination was set, and the first two of those conjuncts turned out to be free.

**One entry says the wall is not there, and pins it.** Collusion and honest
agreement are the same shape: nothing authenticates a witness or checks that
attested evidence is true, so a record founded on three matching lies completes
exactly as one founded on three matching truths.
`tests/pins/red-team/collusion-three-attesters-who-agree` pins that TRUE
deliberately, so the boundary is executable rather than only written down. What
the design does instead is raise the price — three distinct named writers, a
reader, an alternate, published reasons and a challenge route, all attributable —
which is a cost and not a barrier. Every contract card's "conditions on supplied
credentials, not proof of independence in practice" is saying this.

**Four behaviours are recorded as outside what Nibli can test**: adverse
selection, moral hazard, free-riding and black markets. Free-riding is
unobservable here *by construction* — the design demands nothing of the
recipient, and that is the thesis, so the cost of the choice is that this
particular gaming route leaves no trace. Their assigned routes are quantitative
models, games and simulations, empirical evidence and operational assurance,
none of which is built, and no claim about any of them follows from a green
verifier.

**Compositional closure is censused, and what is not censused is named —
2026-09-15.** The item's list of composition failures is answered one at a time
rather than in bulk. **Duty cycles** are impossible, not merely absent: nothing
reads a duty, so no chain of them can close. **Remedy loops** likewise — no
remedy conclusion is read. **Veto by withheld evidence** is checked: every record
completion that admits a challenge reader also carries an independent alternate
behind them, so whoever declines to read holds no veto. **Unbounded delegation**
is checked: every completion carries a source-bound end except seven kinds that
have nothing to end — a current selection, a published candidate, an effective
version, an escalation, and three positive findings that something already
happened — and that list is asserted by membership. **Rival final authority** is
checked: every family's final-route scope carries exactly one value, and the
outcome vocabularies that legitimately carry several are named rather than
skipped. **Contradictory writers** are handled per family by the record-ambiguity
machinery, which completes nothing when two attesters disagree about a
single-valued scope. **Cross-domain routes recreating a score or status gate**
are covered by the floor-vector guards and the forbidden-key list.

**That individually safe domains stay safe when joined is executable, not
argued.** `tests/pins/composition/three-families-joined` puts the knowledge,
record-power and scarcity families' own positive records side by side in one
knowledge base, asks each family's heads, and asks after a person standing
beside them. Each family completes exactly what it completes alone, none reaches
the person, and the shipped cast does not move.

The compound-shock half of the item — what continues, what narrows, who may
decide, who is protected first, what cannot be suspended — is the FS-SCN
scenario catalogue, which is a reviewed inventory with a declared ceiling of
kind I and maximum posture Checked. It is not upgraded by this census. Nor is
the liveness discipline: a bounded safety result still claims that nobody acted.

**The vector of protected conditions is guarded by development tests, enacted
2026-09-15.** `src/authoring/floor_vector_tests.rs` measures five properties of
the enacted lines and pairs each with a control that must fail: no standalone
numeric literal, so nothing can be weighted or totalled; no relation named
`sum`, `count`, `total`, `score`, `rank`, `aggregate`, `index`, `weight` or
their siblings; every read of a floor actuality negative and landing in `err`,
with Article 6's isolation marker the single such read; no floor actuality
derived from a different one, so abundance in one protected condition never
compensates for another; and recognition minted by its three doors, never read,
never joined with itself, and still arity one. Each control was executed against
the actual source, not a scratch copy.

The same file carries the **no-reader census**, which is where the
determination/action boundary is formal rather than editorial. `owe`, `become`,
`lose`, `insure`, `provide`, `grant`, `reward` and `prevents` are read by no
rule; the typed three-place `obliged` is read by no rule; and the legacy
two-place `obliged` has exactly one reader, the allowlisted bridge that concludes
the typed duty. That census is how this design says a duty is not an action and
an interface is not a capacity: not by asserting it in prose, but by leaving
nothing that could read a determination as evidence something happened. It
replaces the retired `verify.sh` rejection of an outside `obliged` consumer.

These replace pattern guards the 2026-09-12 decision retired with the old
`verify.sh` sections, and they are development tests: verification stays pins and
contradiction scans, and no gate is added to it. They establish properties of the
current rule text, not a semantic impossibility — a new relation name routes
around a name-based check, which is why the floor-read and substitution tests are
written over the head and body shape rather than over a vocabulary.

**The assertion surface is generated and verifier-enforced, enacted 2026-08-04.**
`book-1/source/7-assertion-surface.py` reconciles the engine's rule-produced
relations with active `admits` and `derived_only` declarations. Its reviewed JSON
ledger must classify every derived relation and provide an authority, provenance,
harm, challenge, and risk contract for every effective ground-writable premise.
A new head, ground-only admission, posture change, producer/consumer route change,
current ground-fact snapshot change, or missing contract field fails `verify.sh`;
edit the ledger and regenerate the Markdown audit together. The audit is an
inventory and reviewed threat model, not proof of real-world provenance. Its
scenarios are reviewed, not executable pins; the assurance case consumes the
inventory without upgrading it into runtime evidence.

**The record-integrity assurance case is generated and verifier-enforced,
enacted 2026-08-04.** `book-1/source/8-record-integrity-assurance.py` validates
the authored `record-integrity-assurance-case.json`, binds it to the exact
assertion-surface contract digest, and requires every current writable premise
to belong to a record class whose claims cover every mandatory assurance
dimension. Those classes are reviewed grouping choices, not derived or
exhaustive constitutional categories. The case separately treats every use of
absence as an effective premise requiring a route-specific closed-world contract;
the positive writable-relation census does not make deletion-authored absence
safe. The generated case's top verdict is deliberately
**NOT ESTABLISHED** for the live record system: the repository verifies bounded
consequences of supplied records, not their authorship, truth, genesis
completeness, persistence, append-only history, recovery, or eventual
advancement. The case specifies Book 1's
authorship, authority, basis, privacy, challenge, independent-witness,
correction, reconciliation, failure-default, reader, action, continuity,
recovery, and remedy controls while leaving identity, storage, cryptography,
replication, clocks, availability, release/toolchain integrity, and operations
to Book 2. A complete visible basis is also insufficient if a consequential
decision uses a prohibited shadow input. The assurance source, ledger,
generators, verifier, engine, and review can be weakened together; their green
result does not authenticate or authorise their own trust root. It changes no
constitutional rule by itself. In particular, one flat snapshot cannot
distinguish a deleted entry from one withheld or never written.

**Bounded flat-snapshot red-team enacted 2026-08-04:**
`9-record-integrity-red-team.py` executes selected unauthenticated status
additions, exact ground-entry deletions, rollback pairs, generic-judgment reuse,
and same-snapshot combinations. It proves their consequences in supplied flat
records and confirms that a raw `rotten` report is inert. It deliberately does
not duplicate T1/T2/T3 transition, carry, order, or custody tests; it does not
attribute a runtime write or absence, authenticate a record, recover an entry,
or make a successor arrive. It checks itemised `owe` debts and their opaque
event-abstraction `entitled` projections together against the full constitution.
No second registration, adulthood, release, or carry gate was added.

**Staged temporal assurance enacted 2026-08-05:**
`12-temporal-assurance.py` constructs cumulative T1, T2, and T3 sources and
executes every supplied-record case in a fresh engine process. T1 admits only
the collision-free terminal successor selected by the witnessed constitutional
lineage for adverse carry and public-power effect; T2 closes typed event and
record paths transitively and propagates type-scoped conflict; T3 rejoins exact,
independently witnessed source, window, case, lease, Court-judgment, and
injury-victim bindings. The protected-record alarm conservatively calls every
missing required carry `RecordDisappearance` because lawful record disposal is
not represented. Fresh processes are mandatory: derivation is monotonic inside
one process and does not retract an earlier conclusion when a later successor
is added. This is bounded safety evidence only. It does not authenticate
witnesses, prove a manifest complete, detect deletion before or inside the first
attested record, advance a clock, publish a successor, cause physical release,
or make Appeals act. A later operational claim can satisfy the record case only as scoped
`external_verified` evidence; it may not be relabelled as the repository-only
`current_verified` posture.

**Amendment enactment implementation supersession, 2026-09-13.**
`book-1/appendix/contracts/amendment-enactment-contract.md` now controls exact-source
binding and certified/published/effective-version separation. FSPOW_037's
existing collective-result interfaces carry base and candidate identities
through the supplied configuration, submission/result and certificate chain.
The amendment-enactment family consumes that actual result and authority,
independent byte/effect/corridor review, explicit vocabulary disposition,
publication and selection evidence, with conflict, defect and nonresponse
routes. It does not read `become` or change amendment thresholds.

`src/amendment_host.rs` and the separate `amendment-assurance` executable model
exact byte comparison, bounded effect tests, publication checks, one effective
pointer, stale-base/replay refusal, preserved rollback history and fresh-source
queries in memory. Their input is deliberately trusted; they authenticate no
signature and perform no real publication or deployment. They add no hash,
receipt, freshness or history gate to `verify.sh`. Host checks are separate
development tests. Book 2 owns real authentication, durable atomic storage,
publication, deployment, recovery, clocks and operational evidence.

The author approved the exact edits preserved in
`book-1/source/amendment-reader-draft.md` on 2026-09-13. They are inserted
unchanged in Book 1 chapters 1 and 12 and the method; their supply is
`session-drafted, author-approved`. The AS-01–AS-09 source-mutation scenarios
remain, with the label expectations superseded below. Their former absence
descriptions and RI-12's historical status do not describe the bounded
interface; no operational-authentication or semantic-completeness claim is upgraded.

**A reading of the core must give its grounds, 2026-09-25 (item 71).** An
effect reviewer's unreasoned contrary compatibility reading no longer makes the
record ambiguous; a reading refuses only when it names a corridor provision, the
candidate change and its reasons, and an independent final compatibility review
can certify in place of the reviewer's positive reading. The amendment contract
card records the design and the alternatives compared.

**Parallel amendment-label rules removed, 2026-09-19, revision item 19.**
The controlling amendment-enactment contract supersedes the retained Article 9
label mechanism under delegated author approval. Both the proposal-to-personal
`false` rule and the docket/tally-to-`become` rule are removed. Requiring a target
would leave its semantic gap and personal-name collision; changing only the
rejection relation would leave the unsupported law status. Neither parallel
rule is necessary to the constitutional commitments. The existing candidate-
bound certification, publication and effective-selection process remains the
amendment route, with its positive controls and missing-consent/conflict refusals.
The retained protected-register names confer no authority on their own.

A new docketed-Jala case first reproduces loss of recognition, a credibility
finding and a recognition-loss verdict. On the repaired source Jala keeps
recognition and obtains neither adverse conclusion; a targetless proposal
obtains no law label. An explicit counterfactual adds the two weaker rules and
reproduces both failures from the same facts. Active AS pins retain every
candidate mutation, scoped/refused rule and source-effect expectation, while
their superseded personal/law-label expectations become refusals. Historical
audit measurements are not renewed by the updates to existing descriptions
and locators. Item 19's other source questions remain open.

**Amendment semantics are now tested separately from amendment labels, enacted
2026-08-04.** Article 9 reads a proposer-supplied `adjust(amendment, label)` and
compares that label with the `permanent` register. It can derive a dead verdict
and suppress the otherwise-derived `become(amendment, Law)` label; it does not
inspect an exact candidate source change, verify the declared target, apply the
change, or bind the label to an effective version. Nothing reads `become`.
`approves` names the electorate, not the writer; no current rule authenticates
the tally, certificate, recount, challenge, or correction.
`10-amendment-semantics.py` executes exact bounded cases for a targetless label
beside a manually applied floor withdrawal, the same withdrawal beside falsely
harmless and honestly blocked labels, a concealed
one-token change that removes the food entitlement while the separate
anti-imprisonment firewall survives, false-target poisoning of an ordinary
reform, and a direct `admits("rich")` vocabulary widening. The harness manually
applies candidate mutations; it proves their consequences under supplied source
and facts, not proposal-to-diff causation, target truth, authorship, lawful
deployment, semantic completeness, or source-transition assurance. Direct
`admits` widening succeeds even with `Art_Evidence` placed on the candidate
register; it is source-visible and repository-reviewed but is neither an
Article 9 amendment nor constitutionally approved or entrenched. Do not repair
this by requiring an `adjust` fact or semantic-review certificate: withholding
that new premise would create a universal amendment veto while still proving no
target truth. No new constitutional rule or withholding gate was added, and
RI-12 remains `book1_target_unimplemented`.

**Placement exhaustiveness is generated and verifier-enforced, enacted
2026-08-05.** `11-placement-exhaustiveness.py` constructs the Cartesian product
of the current subject states and severity, family, and home axes from writable
premises rather than asserting `prisoner` or `severe`. Every confined row must derive `dwell`;
the exact destination is HighSec for severe cases, LowSec for non-severe family
cases, Homestay for non-severe people without family who have a home, and
deliberately absent for the no-home remainder. The audit separately mirrors
the axes for affirmative freedom and for personhood alone: the shelter debt and
entitlement remain while `fit`, `dwell`, and every destination remain absent. Its reviewed source
manifest inventories every current `fit`, `dwell`, and `building` producer and
destination, so a new route or MedSec-like constant cannot hide outside a
fixed query list. Executable source mutations create overlapping destinations,
delete the historical housing repair, remove a required destination, reverse a
route, and paint `dwell` onto every registered person; each must trip the base
contract. Every alarm-silence probe first supplies and pins a positive `put`
report, and the harmful copies still keep `err(_, Placement)` false. This is a
repository-before-merge guarantee over the current declared axes, **not** a
constitutional runtime exclusivity rule, authenticated placement act, appeal,
remedy, or free-person housing delivery. The live placement alarm still watches
only a reported ineligible Homestay placement. The deliberately housed-but-
destinationless row remains truthful, and the placement-input authority,
provenance, privacy, and challenge gaps remain owned by the future justice
interface rather than being closed by this audit.

**The full-society domain-and-layer ledger is generated and verifier-enforced
at stage 1, enacted 2026-08-09.** The native ledger module validates the
reviewed canonical source `full-society-ledger.json` and renders
`full-society-ledger.md` plus the structural navigation projection
`full-society-reader-ledger.md`; `verify.sh` runs it as a structural
step that also passes under `--quick`. The reader projection is not reader
evidence: it supplies no R6 result, comprehension finding, accessibility
validation, reader-suitability claim, Gate C evidence, or route availability. The source declares the named axes and the
stopping rule; fixes the `scope_disposition` enum, author-ratified as the
reader-facing five layers; and carries the social-domain records with per-layer
buckets, the coverage map's legacy rows frozen and split into one-posture claim
records under the ratified legend, the required bodies, the seven assurance
routes with their falsification status, the named external assumptions, and the
explicit envelope stub `FS-ENV-00`, which can route but never assure. The
generator enforces the ratified fail list — one posture per record, liveness
never established, no feasibility claim anywhere, no established posture on an
unbuilt route, distinct disposition keys, no aggregate score — plus the
defect-disposition compatibility table and the deferral rule that an empty
record type needs an owner and closure condition. It re-reads the six sibling
reviewed JSONs **live** for enum-mapping closure and sits off the 7→12 digest
chain, binding only the assurance-portfolio and full-society-boundary
decisions by digest. Its verdict line is byte-exact and deliberately weak:
routing inventory only, nothing established beyond each row's own posture,
Gate A not passed; the only rollup is the non-numeric worst-case one.

**Stages 2 and 3's executable half are enacted (2026-08-09).** Every declared
defect pin, repair narrative, open gap, and sibling residual is a keyed defect
row under a second live-read closure: the sibling JSONs' residual pools are
collected at check time, and every pool token must be cited by a defect row or
excluded with a recorded reason, in both directions. Resolution is generated,
never authored — a row resolves for its claim only with an eligible
disposition at an implemented stage, its typed control present, its affected
claim's posture established, and exactly one receipt bound; a critical
unresolved row derives blocking on its claim, and hand-authored resolution or
blocking fails generation. Receipts exist only for repairs whose
reintroduction controls already execute in the verifier and whose reader
narration already exists in print, each naming its residual sibling so no
receipt implies a narrower repair cured a wider defect; resolution is
claim-relative and asserts nothing beyond the affected row's own posture. The
register deficit and the live-record gap are critical and block their claims.
The coverage map's section-3 table is now a generated region of the ledger
(the ratified cell texts live verbatim-frozen on the legacy-row records and
render unchanged plus a generated split-claims column; the generator's
`--check` gates both outputs and refuses a region that would duplicate any
coverage-map needle), and the report carries a routed-rows-only Book 2
crosswalk with no operational fields. The grandfathered Part V passage has no
defect row, per the register decision's own words; its dated non-conformance
is a witness on the deficit rows, which the author's pending repair can now
cite. The source-derived power population landed through exact-prefix family
batches, ending at `7e5b5f6`: 210 actual legal-effect cards, one cross-power
temporal-contract template, 19 refusal or limit rows, eight formal-crosswalk
dispositions, and 210 power-bound function allocations. The source-anchor
census below remains the grain boundary; a hand-maintained competitor to any
projection remains refused.

**The full-society public-power source census is landed
(2026-08-13).** `book-1/source/full-society-power-source-manifest.json`,
checked by the native power-manifest module before the native ledger module in both
quick and full verification, binds the exact inspected source revision and
eight source digests. Its 237 rows keep direct legal effects at source grain:
209 require later power cards, 1 is a cross-power temporal-contract template, 19 are explicit refusals or limits, and 8
crosswalk narrow current formal fixtures. This is a candidate census, not an
FS-POW population or coverage proof. It creates no law, complete contract
card, lawful holder, operation, assurance, or Gate A result. The canonical
ledger binds the manifest digest, counts, status, and resolved allocation gaps.
Narrow ratified-unimplemented bodies and paired roles now cover appointments
qualification, custodial execution distinct from policing, independent
ecological science, ecological and animal regulation/inspection, emergency
alternates, Guardian alternates, and border/removal execution. Election
completeness and result service remain external operating assumptions, not
institutions to invent. The cross-power T3 rule remains a non-power temporal
template; its retained Derived active-custody application remains separate from
the Specified custodial executor mandate. Population completion removes only
the powers deferral and
makes Gate A condition one `met-in-form`; it creates no law, operation,
assurance, review event, or Gate A passage.

**The roles, life-course, scale, and power-position matrix is landed
(2026-08-09, FS-ROL).** One record per role, grouped by kind and layer-pinned
to universal standing — a role is never a floor-changing status, and one
person occupying many roles buys no higher floor. Three closures are
mechanical where the tracker's done-when asked for review: every domain
cited by at least one role, every named scale exercised, and every required
body carrying both an affected and a checking role position, the last
grounded in the protective-power ruling's plural initiation routes.
Duty-bearing roles (supporter, parentage, concentrated private function,
animal custodian, collective internal governance) carry a power_held object
whose affected counter-roles and checkers are both mandatory. Omitted
candidates and combinations are recorded with risk-based reasons — a closed
classification decision on the residual-coverage-exclusions precedent, with
a staleness check so an omission contradicting a citation fails generation.
Formal anchors keep the honesty split (derived predicate, asserted predicate
with its replace-card path, ratified-but-unimplemented doctrine), and a
constitution-predicate anchor must cite a `.nibli` needle, never only prose.
The Future Conditions Guardian and the Animal Protection Advocate joined the
required bodies (author-approved) because the coverage-map roster predates
the ecological ruling. Pairwise sufficiency is not mechanically established;
the repository audit checks only its declared source-derived criteria; the FS-POW decomposition is complete at `7e5b5f6`,
but no role or power record creates a predicate, operation, delivered remedy,
or Gate A claim.

**The functional-flow and cross-domain dependency map is landed
(2026-08-09, FS-DEP).** One record per flow per ordered endpoint pair,
endpoints typed against bodies, roles, domains, and external assumptions —
an assumption supplies, never receives — with each edge's layer derived
from its four-way class and an institutional steward beside the tracker
owner. An edge is routing, never delivery. The three ratified lifecycle
paths are pinned by needle and never flattened. Every edge declares its
predeclared alternate route with a doctrine needle or records the absence
as a named single point of failure — the accountability terminal's reader
gap is the flagship recorded residual, and no internal reader is
fabricated. Refused flows are recorded as walls, from the enforcement
firewall to payment-never-delivery to presumed family supply of the floor.
Four closures are mechanical: every domain an endpoint, every flow kind
exercised, every external assumption feeding an externally-assumed edge,
and every strongly connected region of the declared graph carrying at
least one declared, classified, bounded, owned loop witness — never say
"every cycle is classified"; the witness is regional, boundedness is
reviewed prose. The constitutional-closure audit now consumes those witnesses
and keeps self-certifying, deadlocking, single-veto, unbounded, bottleneck,
and cascade risks `bounded-unresolved` unless an eligible scoped blocking
defect makes them `block`; it admits no `rejected-by-control` state until a
route-bound executable-control receipt schema lands, and never treats
cyclicity alone as failure. The declared loops chain-check
into real
cycles (fiscal, electoral, service, ecological, audit, emergency, federal,
intelligence oversight). The settlement-backbone failure scenario row
landed author-approved (2026-08-09) as an FS-SCN stress record, and
FS-DEP-14 records the approval's basis — the question asked and the
author's choice. No dependency record creates a predicate, rule, remedy,
or claim.

**The whole-society scenario catalogue is landed (2026-08-09, FS-SCN).**
Every coverage-map section-7 row, the emergency decision's net-new
acceptance rows, dedicated collision records for the mandate's named
axes, and the named compound shocks are records of one catalogue, with
the kinds, collision axes, shock kinds, and protected-sphere forms as
closed enum-and-meanings vocabularies. The status literal is
`reviewed-inventory` and it means the assurance ceiling: kind I, maximum
posture Checked, never citable as proof or a counterexample harness. The
layer is pinned constitutional-invariant — a scenario states Book 1
invariant and failure behaviour; capacity and degradation are Book 2's
tests. Routes are routing, never delivery: the failure route carries
interim continuity, the recovery route carries remedy and restoration
together, and outcome prose uses the owed/duty register, never arrival —
an adversarial four-lens review (needles, closures, register, doctrine)
ran before landing and its arrival-register, invented-doctrine, and
case-bound-claim findings were fixed pre-commit. Seven closures are
mechanical: no domain still defers scenario applicability (all twelve
flipped to answers in the landing commit), every domain reached, every
kind, named axis, and named shock exercised, every ratified
protected-sphere test exercised against the protected domain, and every
critical dependency edge stressed or its omission recorded — the
recorded-omission exemption exists but is unused, since the
author-approved settlement record covers FS-DEP-14. Omitted candidates
are closed classification decisions with risk-based reasons
(acceptance-bullet inventories stay contract-card grain; book prose may
never source a scenario row; capacity variants are Book 2's; a
state-defined protected-sphere outcome is refused, not deferred).
Bounded witnesses live-read the sibling case inventories and establish
only what those artifacts' own postures state. Stewards respect the
ratified separations: force and assembly incidents sit with the
independent investigator, never the deploying body; intelligence
scenarios with oversight; animal-use authorisation with the
administration while the Advocate keeps the checking routes. Execution
of constitutional cases stays sequenced behind the relevant author
rulings and contract cards — the catalogue asserts no execution.
FS-SCN-22 carries the single-deprivation claim scoped to the current
source; the protective-rule-family landing's prose re-audit should sweep
it beside its four named sites. No scenario record creates a predicate,
rule, remedy, or claim.

**The constitutional-closure and model-allocation audit is enacted
(2026-08-13).** The native constitutional-closure module consumes the reviewed
full-society source after the native ledger module validates it, generates
`book-1/source/constitutional-closure-and-model-allocation-audit.md`, and
checks the projection and its watched-failing mutations in both quick and full
`verify.sh`. It computes each claim's `pass`, `block`, or
`bounded-unresolved` result by joining the claim to its assurance/model route,
closure contracts, affected defects, generated claim-relative resolution, and
eligible receipts; it refuses route substitution and undisposed structural
gaps. The ceiling is structural and artifact-bound: `pass` means only that the
reviewed structural contract passes this audit. It upgrades no claim posture
and establishes no delivery, liveness, feasibility, operation, or Gate A
closure. The audit may therefore be complete while claims remain blocked or
bounded-unresolved and while Gate A remains not passed.

**The reference envelope is versioned in structure, enacted 2026-08-09; values,
calibration, and every feasibility claim stay outside Book 1.** The envelope
array now carries the permanent `FS-ENV-00` pre-envelope identity — every Book
1 claim, defect row, and receipt stays keyed to it unmigrated, because Book 1
claims are not envelope-bound — beside `FS-ENV-01` (`envelope-v1-structure`),
whose fields are versioned with definitions, dependents, and
invariance statements and whose every value status names Book 2's Gate D
calibration as owner. **`calibrated` is refused outright in this contract**:
calibration becomes legal only through a deliberate future contract amendment,
and the operationally-assured stage and remedied path still require a
calibrated envelope through one shared helper, so a structure can route and be
reviewed but never assure operation. Gate A instead requires a non-stub,
versioned-structure envelope; it does not wait on Gate D calibration. The
dependency closure is the
design's thesis made mechanical: no established constitutional invariant may
depend on an envelope field — norm content is envelope-invariant — while the
envelope-relative claims (arrival, and live-record persistence under the
infrastructure-failure shock) must appear as dependents with their relativity
stated. The **functional criteria** are fixed as the seven-member union with
provenance (adequacy; accessibility/equality; continuity; resilience;
sustainability; resource; safety), each bound to the rulings' actual sentences
by needle, the drift across the five ratified variants recorded, and the
ledger's own materiality test aligned to the boundary decision's exact
wording. **Thresholds are meanings, not measurements**: each binds a ratified
sentence, classifies its lawful source (constitutional minimum or ceiling,
democratic policy target, scientific safety boundary, or operational
diagnostic — the source fixes the layer), separates its decision owner from
its measurement owner, and carries no numeric value; values arrive with their
classified lawful source, never here. The Gate A readiness row for the
envelope computes met-in-form: versioned structure is sufficient for the scope
and assurance-program gate. Values, calibration, operational assurance, and
remedied resolution remain Book 2 Gate D work, and Book 2 stays inactive until
Gate C.

### Four closing rulings — 2026-09-16

The tracker's last four open items were ruled together under the author's
instruction to finish them with sensible defaults. Each is a decision with its
evidence, not a task marked done.

**The five-minute verifier target is retired and replaced by a per-case
standard.** Five minutes was set against a 4,190-case inventory; that inventory
is now 15,481 cases, the profile says the cost is per-case fixed work rather
than case count, and the only sized lead left is a `nibli-reason` change worth
about 15% in a companion pinned at `979fe8b`. Fifteen percent of fourteen
minutes is not five, so the number had stopped describing anything a session
could act on, and an unreachable standard makes every honest report read as a
failure. **The standard is now per case**: 15,481 cases in 825.00–1,041.06
seconds across three runs of the same inventory on 2026-09-16 — **53–67 ms per
case**, the middle run 884.91 seconds for 84,314 pins across 15,481 cases — four
workers, release binary prebuilt. That spread is machine variance, and it is
itself why wall-clock was the wrong instrument. A run reports its per-case cost; an unexplained regression of more
than half against that band is the failure condition.

**The formal source's size is accepted, with a named trigger.**
`constitution.nibli` is 55 MB against GitHub's 100 MB hard limit. The one clean
lever — variables are 44.7% of the generated bulk — **was attempted and
reverted**. Wrapping the ten rule emitters compacted eight blocks, public safety
falling 17.6 to 10.2 MB, and then every generator failed: the counterfactual
machinery matches exact atom text, so `~($source = $review)` stops being found,
and ecology's and public safety's structural searches key on `$record`,
`$subject` and `$source` in rule heads. A working version needs a per-generator
keep-set threaded through the helper, because base field names like `$role_kind`
are underscored exactly like the prefixed ones. **Revisit when a family of
ecology's scale is planned or the file passes 75 MB**, and start from the
keep-set design rather than the emitter wrapper.

**The preview-snapshot binding stops being tracked as work.** Every mechanically
testable accessibility check exists; binding exact HTML, EPUB and PDF identities
waits on a Gate B snapshot that does not exist, so it is a condition on that
gate rather than a task. No accessibility-for-users claim follows from any of it.

**The adversarial audit's open findings now carry dispositions, checked.** Four
declared values — `route-unbuilt`, `public-claim-limited`,
`author-ruling-pending`, `blocks-gate` — plus a consequence saying what each
costs in what the project may claim. The generator refuses an open finding
without both and a closed one carrying either;
`every_open_finding_says_what_it_costs` sabotage-tests the refusal. **Disclosure
is deliberately not a disposition** — naming a limitation and moving on is what
the resolution receipts refuse and what this table used to permit. Nine open:
four public-claim-limited, three route-unbuilt, two author-ruling-pending, none
blocking a gate.

### The rebuild of Book 1 — 2026-09-16

Seven rulings from one sitting, taken after the root tracker was retired and
re-created the same evening as the ordered tracker for this work. Each is
**ratified but unimplemented** in the standing sense until its tracker item
lands; the supersession notes that follow implementation are added here as
each item lands. The controlling records are the first two files under
`book-1/appendix/decisions/`, the directory the fold (R5) populates.

**R1 — The child is the heart.** *(Landed in full. The chapter, its pins and
the opening argument landed 2026-09-17; the recurring section landed
2026-09-18 in every derived chapter that can run it, with four exemptions
carrying reasons — chapter 1 is the case, chapter 10 already carries Cira,
chapter 23 turns on a credential a one-entry record cannot hold, and chapters
24 and 26 turn on an exposure the child has not made and a recognition the
child has never held. Two membership tests hold it:
`every_derived_chapter_runs_the_child_or_says_why_not` and
`every_child_slot_loads_the_one_line_record`, the second of which caught a slot
whose second case did not load the record.)* The caregiverless child is the opening
argument (exempt), the first derived chapter (*The Child With Nobody* — a
record of one line, `born(Nell).`, introduced by fixture and never added to
the cast), and the closing test section `## The child with nobody` of every
engine chapter that can run its rule against a person with one entry,
asserted by membership with reasons for every exemption. "Most vulnerable" is
a **test on absent entries** — no private route to the floor, no chosen help,
not acted upon by public power, presence not guaranteed — and the design's
answer is that the public duty reads none of them: every debt and every
barrier has `person($x)` as its whole body. It is never a status:
`vulnerable` is not a corpus name and its refusal is pinned; the test is computed
from silence and reads into nothing; it captures a set (the unconscious
adult nobody has come for, the person whose only supporter is the one accused
of failing them, the unaccompanied newcomer, the person whose language nobody
present speaks) and excludes the prisoner and any child with a recorded
parent, which is the pair. This supersedes the 2026-08-17 through-line refusal
in exactly that scope and nothing else: the flat register (the affect warning
is now printed rather than implied), the `dignity`/`safety` bans, the title
ruling, the passage rule and `OL-15-v1` stand; the methodological claim stays
Reasoned in the exempt elements beside its one executable instance; the
receipt gap for a recipient who cannot choose a witness stays open and named.
The "hardest stress test" superlative is retired for the pair, which needs no
winner — the re-measurement the 2026-08-17 ruling required is recorded by
method in the record. Controlling record:
`book-1/appendix/decisions/child-with-nobody-decision.md`.

**Criterion narrowed 2026-09-24, not yet implemented (D5 of *The revision
rulings D1–D9*):** the section returns where the one-entry record produces a
different or instructive result, not wherever the rule can run, and each removal
is recorded as an exemption with its reason.

**R2 — Names.** Book 1 keeps *The Rights Nobody Has to Earn* and now speaks
it. **Book 2 is *What It Would Take*.** Both names are used in every exempt
element, the front matter, the READMEs, both trackers and the appendix;
**derived chapters stay name-free** by the author's choice, because a derived
sentence traces to a rule or a pin and a title is neither. The former
"book-1 references book-2 exactly once" constraint is retired; it was already
false. Rejected titles are recorded so they are not re-proposed.

**R3 — The reading order is editorial: engines before breaks.** *(Landed
2026-09-17: the manifest carries the rule and the thirty-one-chapter table with
fifteen reserved slots, the files carry the final numbers, and
`derived_chapters_run_engines_before_breaks` enforces the partition. Completed
2026-09-18: every reserved slot is filled, no `planned` entry remains, and the
share the ordering exists to protect is measured rather than asserted — 41 of
232 passages are coercive, against 153 of provision, private life and
democratic agency, bounded by `the_book_is_not_one_failure_first_formula`.)* Chapter order
was never computed — `3-spine.md` says so itself, and the runtime order is
the filename prefix — so "strictly computed, never chosen" below is
superseded, as is the 2026-08-17 "substance primacy is not allocatable". The
rule: *the chapters run in the order a person meets the design, not the order
its rules depend on one another* — who counts and what may be written, then
what is owed and how it reaches, then the life the design leaves alone, then
the public power that serves it, and only at the end what the design does to
a person and how it catches itself. `book-1/contents.json` records the rule
and the final thirty-one-chapter table with reserved slots; development
tests enforce it; the stratification remains the derivation record. Chapter
numbers in rulings dated before 2026-09-16 are pre-reorder and are not
rewritten; the applied maps under `tools/maps/` are the key. Controlling
record: `book-1/appendix/decisions/reading-order-and-appendix-decision.md`.

**Chapter table superseded 2026-09-24, not yet implemented (D7 of *The revision
rulings D1–D9*):** Chapters 1 and 2, 9 and 10, and 23, 25 and 26 merge; Chapter
13 splits; two chapters are added; and the reference material moves to the back.
The refusal to merge Voiding with Clawback is superseded. The ordering rule
stands.

**R4 — The appendix is a carried archive, not a fourth channel.** *(Landed
2026-09-17 with the move; confirmed by measurement 2026-09-18 — the
length-invariant figures recorded under R6 count the ordered inputs only, and
`book-1/appendix/` and `book-1/source/` are outside them.)* The planning
record moves under `book-1/appendix/`. The 2026-08-08 refusal of "a fourth
exempt element" is overridden **narrowly**: the appendix is a non-derived
element of the *directory*, not of the *book* — no passages, no register, not
an ordered input to any artifact, outside the edition boundary until a Gate
B/C decision binds it, unclassified in the coverage ledger, **outside the
length-invariant measurement**, and never the sole support for a chapter's
claim. The book's exempt elements remain three; the method part's sealed
scope is untouched and its "the repository is the appendix" pointer names
the directory.

**R5 — `new-book-plans/` ceases to exist.** *(Landed 2026-09-17. The
directory is gone, every path in this file points at its new home, and
`every_reviewed_reference_resolves_exactly_once` holds the rewrite against a
dated baseline.)* The forty-five human-facing
planning documents move to `book-1/appendix/{decisions,contracts,briefs,maps}`
with the `book-1-` prefix dropped; everything else — the constitution, every
`*-source.json`, the family pins, `counterfactual/`, the generated reports
and frozen audits, the reader-evidence kit, the reader drafts, the scripts and
the engine-measurement history — moves by flat rename to `book-1/source/`.
The relocation tool rewrites every path; the constitution, the rule-family
sources, every non-comment pins line, `4-strata.py`, `registry/` and
`reviews/` are asserted byte-identical. Every `book-1/source/` path in this
file is a pointer and is rewritten when the move lands.

**R6 — The re-measurement.** Custody remains the deepest derivation chain;
family and life course is now the widest family, its heads read by no rule;
in the shipped cast the floor actualities still derive only through
confinement. Depth and width name different subjects, which is why the
superlative goes and the pair stays.

*Re-measured 2026-09-18, after the rebuild.* All three findings hold, and the
third is now executable in the book rather than only in the source: chapter 27
runs the pair side by side and derives shelter and recorded speech for the
confined person and neither for the child, with both owed both. The reader
ledger's own census moved where the interfaces moved — `cares` and `associates`
left the thin set when the life-course and mobility families were rendered, and
`creates` is the only posture left at two passages or fewer, which is the
pattern the ledger predicts: a thin posture is a missing interface, not a
missing paragraph. Book 1 measures 68,666 derived words against 24,884
non-derived (opening note 7,468; Part V 9,245; method 8,171), so the
majority-derived invariant holds with a wide margin; `book-1/appendix/` and
`book-1/source/` are outside that measurement per R4.

**R7 — Two counts corrected.** "Fourteen derived chapters" becomes
manifest-derived — the digit-gate test reads `book-1/contents.json` instead
of counting the directory — and "references book-2 exactly once" is replaced
by R2.

### The revision of Book 1 — 2026-09-18

Two standing authoring rules arrived with the revision tracker, and both govern
every item in it and everything after. They are recorded here because a tracker
is deleted when its work lands and these are not task-shaped.

**Current design throughout — author instruction, 2026-09-18.** Every
reader-facing part, including the opening note, Part V and the optional method,
describes the design of the edition being read. Accounts of the book's own
development — earlier rules, repairs, additions, before-and-after versions —
come out, accurate and instructive or not, and the current rule is explained
directly with its rationale, consequences and limits. Revision history lives in
git and in the repository's decision records, outside the reading sequence.
Three things are expressly still admissible and must not be swept with the
rest: historical evidence about the world (Part V's cases), explicitly tested
alternatives and counterfactuals, and temporal behaviour inside the current
model. This narrows the repaired-defect register the 2026-08-02 clawback ruling
and the resolution receipts established for chapters — the receipts themselves
are repository records and are unaffected, but a chapter is no longer the place
a repair is told. "Current" means the design bound to the edition being read;
published editions stay immutable.

**Resolve before defending — author instruction, 2026-09-18.** Exposing,
labelling or explaining a defect does not substitute for correcting the design.
Establish the failure and its cause against the current source; attempt
substantive repair, including a different representation, a narrower power, a
replacement mechanism, simplification or removal, and treat an inherited choice
as movable unless it is one of the ratified commitments. Implement and verify
when a sound resolution exists, testing both the legitimate behaviour that must
remain and the harmful behaviour that must stop. Defend a remaining limitation
only when resolution is shown impossible inside stated, justified constraints,
recording the alternatives examined and their results in the controlling
decision — and a failed encoding, an inconvenient implementation, a tooling
limit or an undiscovered fix is not such a proof, nor does a finite search
establish universal impossibility. An adequate defence says why the constraint
is necessary, why this arrangement beats the alternatives, who bears the
remaining harm, what safeguards bound it, and what evidence would reopen it;
calling a cost a trade-off, disclosing it honestly, or pinning it does not meet
that bar. A constitutional defect may not be closed by transferring it to Book 2.

**Item 01 — the appeal stages, landed 2026-09-18.** `permits(Appeals, ·)` is
appellate relief already granted, per case, and is the one conclusion in the
record able to stop a conviction holding. Reading it as permission to appeal
made `all $x: prisoner($x) -> permits(Appeals, $x).` look like a universal
right of appeal; it is universal automatic relief, and the stratification
refusal it earns says nothing about the policy. Measured 2026-09-18: the
replacement encoding — `all $x: prisoner($x) -> obliged(Appeals, $x).`, a duty
on the appeal body — loads, and with it resident `prisoner(Ruk)` stays TRUE,
`permits(Appeals, Ruk)` stays FALSE and `free(Ruk)` stays FALSE. Both encodings
are now pinned in `book-1/21-a-way-to-be-heard.pins.nibli`, whose closing block
is last because its `:accept` is unscoped. The stages the design actually
carries — access under `JusticeAccessibleProcess` with its wealth, documentation
and status barrier reading `person($x)` as its whole body; the request needing
no operator permission; the duty to hear with its interim-relief clause and its
nonresponse route to a separated alternate; and relief — are separated in
chapter 21 and receipted as `appeal-read-as-relief`.

The same item found the method part printing the four standing roots with every
variable stripped, which is not what the source says and breaks that part's own
ground rule that rules appear exactly as the files write them.
`claim_discipline_tests::every_rule_the_method_part_quotes_is_a_rule_some_file_holds`
now requires every quoted rule to occur verbatim in some `.nibli` file, which is
why the two refused rules the part shows are pinned in chapter 21 and chapter 29
rather than only quoted. It is sabotage-tested against the exact defect. Its
scope is quoted **rules** — a statement opening `all ` and carrying an arrow;
the vocabulary listings are set in columns for the page and are outside it.

**Item 02 — the credibility finding, landed 2026-09-18.** Controlling record:
`book-1/appendix/decisions/credibility-finding-decision.md`. Measured before
changing anything: `false/1` is read by five rules — the three recognition
doors and Article 9's law label under negation, and the clawback leaf
positively — and by nothing else, so a voiding reaches recognition and one
recorded loss. No floor actuality, no `owe`, no `entitled`, no `person`, no
`prisoner`, no `travel`, no `decide`, no `authority` and no `permits` reads it;
credentials read the reconciled carried mark rather than this conclusion.
`a_credibility_voiding_reaches_exactly_the_measured_set` asserts that set by
membership with a sabotage control, because the chapter states it as the whole
consequence.

Three defects, all repaired. **The finding had no ground** while the paid door
in the same article already required one — so both signers must now cite the
same ground, **and the ground must belong to a closed vocabulary**. That second
condition is the one to remember, because requiring a shared ground alone
closes one hole and opens a worse one: `cite` is what the reward rule reads,
an examiner is paid for looking on any recorded ground, so two examiners who
looked on a complaint and found nothing would have voided the person they
cleared. `CredibilityGroundVocabulary` — a recorded lie, a fabricated record, a
concealed conflict, evidence withheld — is built the way every landed family
builds one: `member` is conclusion-only and one rule per admitted value is what
closes it. The shipped cast keeps `Complaint` for Bela (the occasion, which
still pays) and gains the adverse ground beside it, so `false(Bela)` is unmoved;
every exhibit in chapter 25 carries its own admitted ground so the conjunct each
exhibit is about stays the only failing one. **Expungement reached one route in three** — measured,
`clean(Bela)` derived and `false(Bela)` derived anyway — so `~clean` now guards
the multi-sig rule and the adjudicated-lie rule beside the carried mark;
Article 5's conflict rule deliberately does not, because that void tracks a
judgment still on the record and still conflicted, and
`a_recorded_expungement_reaches_every_voiding_that_turns_on_a_finding` asserts
the split. **The finding was unanswerable**: two rules now conclude a duty on
the audit body toward the person (personhood in the body, keeping it off
Article 9's proposals) and a review duty on the appeals body from the subject's
own contest of the cited ground. The first is written in the TYPED three-place
form, and the reason is worth carrying: the two-place `obliged(Review, ·)` is
Article 8b's marker duty, and a second producer of it made chapter 30's "silent
where the markers are silent" pins read TRUE. The full verifier caught it in
thirty-six cases; naming the duty keeps the two apart. Neither is read by anything, so neither is
evidence that a reason was given or a review happened.

A fourth finding was coverage rather than design, and it is the one to
remember. `false(Tyr)` was blocked twice over — by the carried void chapter 25
argues from, and by two independence findings nobody had written for that pair
— so `counterfactual/unguarded-pen`'s arming pin passed without testing the
conjuncts it exists to justify, and the comment claiming the verdict would flip
had stopped being true. The independence findings are now in the cast and
`counterfactual/unguarded-pen-and-no-conjuncts` applies the unguarded route and
the stripped conjuncts together, pinning `false(Tyr)` TRUE. **A claim in a
comment that a control would flip is not a control.**

Rule count 7082 → 7088; predicates, derived predicates, strata and the floor
unchanged. What was not done and why — a case-scoped `false/2` (the reach is
not case-shaped and the arity measured at roughly twenty-two times runtime when
`reward` was tried), an expiry (an ending that ran while nobody acted would
clear the findings of the people nobody reviewed), and a bar on a voided person
signing inside one record (the three routes examined each reintroduce the loop
or a finding with no finder) — is recorded with its examined alternatives in the
decision.

**Item 03 — the initiation gap, landed 2026-09-18.** Every route into this
design began with a record somebody wrote, and each was unconditional in the
ways that usually stop people while still waiting for an act. For a person
nobody had come for, the whole apparatus sat downstream of an act nobody was
obliged to perform. **Article 1c** closes it in the shape of Article 1b
directly above: `all $x: person($x) -> obliged(RightsAdvocate,
InitiateAssistanceAndRepresentationWithoutARequest, $x).` — personhood is the
whole body.

**It reads nothing about the person, and that is the ruling.** A duty scoped to
whoever is "unable to act" needs a capability finding this design refuses to
hold (`vulnerable` unadmitted, no global capability score, the life-course
corridor naming that refusal by constant). A duty computed from what is
*absent* in a record fires on everyone and discriminates nobody — the exact
ground the floor-delivery markers were refused on. So it is owed to everyone,
discharged in the asking wherever somebody asked, and load-bearing where
nothing else happened. The escalation takes no credential: `person($x) &
observe($writer, $entry, $x, UninitiatedAssistanceScope) ->
obliged(IndependentRightsAlternate, …)` — any writer at all, the justice
family's bare-request polarity pointed at a third party because the subject is
the one who cannot file. The entry concludes nothing about the person.

**What discharging it must contain** is the `initiation` card added to
FAMILY-LIFE-ORDINARY: a closed trigger vocabulary, a representative for one
named matter and not a status, a named receipt witness, no request and no
capability finding, the person's own voice retained, minimum information and no
enrolment, continuity unaffected by a gap in representation, a source-bound end,
three distinct attesters, a challenge reader and an independent alternate. The
generator produced 60 cases from the card.

**The receipt-witness answer, and the refusal that shaped it.** The delivery
rules read `authorized($w, DeliveryWitness, $p)` — a witness authorised *for*
the person, never *by* them — so the gap was never that an unrepresented person
may not have a witness; it was that nobody was answerable for naming one. The
card's first draft concluded `authorized(...)` directly and **the engine refused
it**: `Unstratifiable negation: 'authorized' -> 'contradict' (negative)`,
because every card's premises read `~contradict($record, …)` and `contradict`
is downstream of `authorized`. The head is a duty on the acting body to
authorise the named witness instead, which is what the design can honestly say.

Rule count 7088 → 7107; predicates, derived predicates, strata and the floor
unchanged. Chapters 1, 5, 7, 14, 19 and 21 carry it, and chapter 21's child
section — which said "no rule here obliges anybody to notice a child who has not
asked… the one this book has not closed" — is rewritten. The residual is
recorded as `nobody-obliged-to-begin`, state `operationally-unresolved`: a rule
over supplied records cannot discover a person no record has entered, and the
entry that opens the escalation must still be written by somebody who saw. What
changed is that the act nobody performs is owed by a named office rather than
waited for from the person who cannot perform it.

**Extended 2026-09-25 (item 72):** the report that opens the escalation is
also a standing root, so it reaches a person no other record has entered, and
the initiation appointment completes only for somebody a standing encounter
has reached.

**Item 04 — the shield's scope, landed 2026-09-19.** Controlling record:
`book-1/appendix/decisions/shield-scope-decision.md`. `defend/1` has exactly one
reader, the conviction rule's `~defend($offender)`, and it was unscoped: one
`show` naming any authority blocked every conviction of that person, for any
offence, indefinitely — a general immunity bought with a single write, against
an eligible set that only grows because answerability is never revoked.

**A second conviction route, beside the first rather than replacing it.** Same
T3 conjuncts; `~defend($offender)` becomes the positive case — the person is
shielded, and two answerable bodies have recorded against this exact case and
person that the prosecution is unrelated to the disclosure, with
`~($reviewer = $second)` and `~show($offender, ·)` barring each from being a
body this person exposed. The ratified fail-open polarity is untouched: absent
the finding the shield holds. The scope is the case; `defend` still derives and
still stands for every other case.

**`~show($offender, ·)` is the conjunct that makes the route worth having**,
because the body with the strongest motive to call a retaliatory prosecution
unrelated is the body the disclosure named. Measured on the cast: Sly exposed
the court, so the court is barred and the review body is eligible and alone —
the pair does not complete and `prisoner(Sly)` stays FALSE. One eligible body
is not enough.

**The Rex sequence resolves.** Measured: Rex carries the whole custody chain
(`match(Rex, ConvictionRecorded)`, `correct(Case_Rex, ActivePower)`), so the
shield was the only thing blocking the conviction. In chapter 24's pins,
`Appeals` and `Convocation` — the two bodies Rex did not expose — record the
finding, `prisoner(Rex)` derives, and `defend(Rex)` and `show(Rex, Review)` both
stay TRUE. Rule count 7107 → 7108; predicates, derived predicates, strata and
the floor unchanged, and the shipped cast holds no `ShieldConnectionScope`
observation, so the route ships dormant.

**The arity change was tried first and is recorded as refused, with the reason
that generalises.** `defend($w, $case)` needs the coverage written as
`defend($w) & ~unrelated($w, $case) -> defend($w, $case)`, where `$case` appears
only under negation — an unbound variable ranged over every constant in a 55 MB
base. The probe ran past ten minutes against a suite whose ordinary case costs
eleven seconds. **Bind a case variable positively or do not write the rule**;
the conviction rule already binds `$case`, which is why the second route lives
there. Reading an ordinary `cite(Court, $case, ·)` as the finding was also
refused: it is present in every prosecution and would empty the shield.

The initial scoping left a person who exposed every answerable body with no
eligible reviewer. **Superseded by item 10 below:** that blanket veto is
replaced by positive case-bound authority and independently reviewed
eligibility, with actual-conflict findings and party/separation exclusions.
The prior defence did not establish that the failure was necessary.

**Item 05 — the purpose of recognition, landed 2026-09-19.** Controlling
record: `book-1/appendix/decisions/recognition-purpose-decision.md`. The census
first: `reward/1` has three producers and no readers, `lose/2` one producer and
no readers, and a credibility finding reaches recognition and nothing else. The
book said all of that at length and never said why the mechanism exists.

**The purpose, stated.** A design of this kind cannot dodge "what about the
people who contribute more?" Answering with a number builds a currency; having
no concept of contribution changes the subject. Recognition is the third answer
— the design records that somebody contributed and the record confers nothing,
which is a positive statement about what noticing buys rather than an absence.

**Removal was compared and refused on a measured ground.** Recognition is the
only thing a credibility finding reaches, so deleting it leaves the harshest
conclusion this design reaches short of confinement with no content at all. The
pair holds each other up. **Keeping it because the slot is occupied is refused
as a reason** — an occupied slot is not safer than an empty one, and what stands
against a future score is the guards rather than the furniture; the prose says
so rather than claiming the stronger protection. **Renaming `Points`** — a unit
this design does not have — was examined and declined: sixty-one pin sites for a
cosmetic gain, and chapter 26 already names the thing correctly in prose.

**The thin-posture set is now empty**, and it emptied the way the ledger
predicts. `creates` was the last posture carried by two passages or fewer, and
what moved it was a passage about why recognition exists at all — an interface
question rather than a missing paragraph, which is the pattern every thin
posture in this book has followed. `THIN_POSTURES` stays asserted as an empty
expectation so a posture falling back to two fails rather than passes quietly.

Chapter 10 gains "Why it exists at all" and pins the claim: after the voiding
Bela keeps personhood, the food debt, the ballot and free movement. Chapter 26's
Cira section states the current boundary instead of its history — the argument
for reaching the student is given in full, the narrowing is shown to be a
repeal, and the cost is stated: some recognition somewhere rests on work that
was not done, and the design leaves it there rather than hold an instrument that
can reach a person who did nothing. The `student-clawback` receipt is repointed;
its old phrase was "the resolution was a deletion", which is one of the
repair-narration detector's own trigger forms.

**Arrival is Book 2's subject — author instruction, 2026-09-19.** "Whenever
there is *arrives* we should talk about book 2." Book 1 settles what is owed,
who owes it, what evidence establishes that something reached somebody, and what
follows when it does not; it stops where the question becomes whether anybody
went. Where a chapter states that a duty is not an arrival, it is naming the
seam rather than confessing a hole, and it points across it. Apply this at the
**boundary**, once per chapter where the chapter's limit is a liveness limit —
not at every occurrence of the word, which would be the repetitive
throat-clearing the line-edit item exists to remove. Derived chapters say "the
second book"; only the exempt elements use the title (R2).

**Item 06 — one standard for debt, evidence and delivery, landed 2026-09-19.**
The book's central comparison read a derived legal consequence as an arrival.
`dwell` for a confined person is concluded from the confinement — no receipt
names them, no witness attests anything, and the rule asks for neither — and
chapter 27 called it "the floor actually arrives" and said the prisoner "has the
thing", with Part V repeating it. That is a liveness claim resting on a formal
obligation, which is the one posture the assurance portfolio says may never be
Derived.

**The repair is the comparison run with equivalent evidence.** Chapter 27's pins
now give the child a receipt and a witness authorised for that child who is not
the provider; `dwell(Nell)` and `eats(Nell)` derive identically, and nothing else
about the child moves. So the asymmetry is a fact about the records, not about
the people — and the true asymmetry is sharper than the one the book claimed:
**the design concludes the floor for the person it holds and demands three facts
from the person it has never touched.** The person nobody has come for carries
the heavier burden of proof.

**The same word on two routes misleads no rule, and the reason is worth
keeping.** Nothing reads a floor actuality except to notice its absence, so no
conclusion anywhere is drawn from *how* shelter came to be concluded. The only
thing the two routes can mislead is a reader, which is why the repair is prose
plus the equivalent-evidence pins rather than a rule change. Chapter 5 gains
"Five things that are not each other" — entitlement, duty, supplied finding,
derived legal consequence, evidence of actual receipt — and names the custody
route as the one place the design's own standard is not uniform. Receipt:
`custody-shelter-read-as-arrival`.

**Item 07 — semantic reversals and overbroad claims, landed 2026-09-19.** Two
classes, both mechanical, both now guarded.

**Uncertainty read as negation.** Four boundary sections said a record *proves*
a negative where it is simply silent: "a compensation concluded here proves
nobody was paid", "the guarantee route proves no fund exists", "a completed
accommodation record proves no adjustment was provided", "proves that nothing
was procured, delivered, restored or repaired". Each asserts a positive negative
finding, which is the closed-world fallacy this design refuses everywhere else
— and they were in boundary sections, the places written to be scrupulous. The
distinction is `proves` followed by a **clause** rather than a noun phrase:
"proves nothing about the service" is fine, "proves nothing was procured" is
not. `claim_discipline_tests::no_boundary_claims_a_negative_it_cannot_establish`
catches the clause form across every ordered input, with the four originals as
sabotage controls. A sentence that DENIES the proving — "Neither device proves
that nobody judges a person elsewhere" — is correct and is stripped before
matching, on the repair detector's principle that a rule beats an allowlist.

**A syntax restriction stated as a universal theorem.** Chapter 27 draws the
firewall's edge exactly: the refusal reaches confinement, and a rule reading a
floor right's absence into credibility or recognition loads without complaint.
Chapters 4 and 18 said "punishing", which is the whole width wider. Chapter 4's
own pins accept `person($x) & ~believe($x) -> false($x)` under
`:accept-scoped` and watch it load, so the chapter's checks contradicted the
chapter's prose. Both now say *imprisoned*, and chapter 4 states the edge with
its own checks as the evidence. **The manuscript's accounts of the firewall now
agree.**

Swept and clean: "actually" across the derived chapters, every instance naming
the gap rather than claiming arrival — the one real offender was chapter 27's
"the floor actually arrives", removed by item 06. The exhaustive sequential read
for every remaining absolute is items 14 and 15, which read each chapter in
order; this item took the classes a sweep can find and left a guard behind.

**Item 08 — duties across institutional failure, landed 2026-09-19.** Two
findings, one repaired and one made visible.

**The escalation set was accidental, and scarcity was on the wrong side of it.**
Censused on the source: four families make the alternate escalate to a court —
`ECOLOGICAL-ANIMAL`, `FAMILY-LIFE-ORDINARY`, `PUBLIC-SAFETY`,
`SUBSTANTIVE-EQUALITY-ORDINARY` — and the rest stop at securing a remedy. The
four are the families whose subject can be urgent bodily harm or care, which is
a principled line, and `SCARCITY-AND-CONFLICT` sat outside it while being the
family about people going without essentials. Repaired at source:
`ReviewScarcityRequestAndSecureIndependentRemedy` becomes
`ReviewScarcityRequestSecureTheMinimumAndEscalateCourtRemedy`, regenerated
across 45 files. `NON-CARCERAL-JUSTICE` stays outside deliberately — performing
the unfulfilled review *is* that family's remedy, since the family is the court
route.

**What happens when the alternate fails too had never been asked.** Every family
answers a reader who does not act by moving the duty to a separated alternate,
and nothing had tested the next step.
`tests/pins/composition/reviewer-and-substitute-both-fail` runs it on a
certified scarcity nonresponse — a person went without an essential, asked, and
was not answered. The chain terminates at the alternate, because nothing in this
design reads a duty and there is no third office; what it terminates in is a
recorded duty nobody has discharged. **The pins establish what does not happen,
which is the part that matters**: the failure does not travel to the person — no
standing, no floor, no liberty, no credibility — and neither office gains
anything by not acting, no authority over the record, no permission, no way to
close a request by ignoring it. Public power does not extend itself through
failure, and the record of the failure outlives both refusals.

Chapter 30 gains "When the office itself is the thing that failed", covering the
seven named cases — no budget or first responder, an office term expiring
mid-remedy, the conflicted reviewer and failed substitute, a wrong identity or
political-home record, documents lost in a move, care at release, scarce
essentials in a declared emergency — each naming its right-holder, duty-bearer,
trigger, evidence, limit, failure, interim protection and review route, and each
closing on the same limit: the design now knows who to name when nobody comes,
which is a different achievement from somebody coming.

**Item 09 — traceable empirical claims, landed 2026-09-19.** The manuscript's
empirical surface is Part V and nothing else: censused, the derived chapters
carry no claim about the world, and every "where reliable evidence shows" in
them states a rule rather than reporting a finding. So the work was Part V's
nineteen figure-bearing sentences, and it found three things.

**Two mis-citations that read as correct.** Part V's Tanzania figure was bound
to `medina-2011-cybersyn` — the monograph about Chile — with a comment in the
test explaining the shared entry as deliberate. It was `scott-1998-ujamaa` all
along. And Auroville's entry cited the Supreme Court's master-plan judgment for
a *governance* holding; the holding Part V argues from is the other judgment of
the same day, *Auroville Foundation v. Natasha Storey*, which confines the
Residents' Assembly to advising the appointed Board. Both passed every check
that existed, because both named a real source that really exists.

**One claim the evidence would not bear, narrowed.** Part V said the state's
published total of 13 million villagers "counted, as James Scott showed, many
people who had never moved". Scott's own figure is the five million relocated;
the higher totals are other people's and disagree with each other. The sentence
now attributes the five million to Scott and argues from the spread itself —
the totals run from around nine million to thirteen and cannot all be right,
which makes the point about a state losing sight of its own actions better than
the attribution did. **A live claim was also sharpened rather than softened:**
Santoshi Kumari's contested cause of death is now named on both sides (the
district recorded malaria; her family and local activists said hunger) beside
what is *not* contested — the cancellation, confirmed by the block officer, and
the six-month refusal of rations.

**A comparison that changed its denominator.** Mondragon's nine-to-one is
highest-to-lowest pay inside a cooperative; the ~300-to-1 set against it is a
chief executive against a *typical* worker at the largest listed firms. Part V
now says so, and says which way the looseness runs. `epi-2026-ceo-pay-ratio` is
a new entry whose notes lead with the denominator warning.

**Two things are now mechanical.** `TRACED` grew from nine rows to seventeen and
is keyed by case rather than by entry, so one source may legitimately carry
several figures while two rows can never collapse onto one. And
`every_traced_source_carries_a_locator_a_reader_can_follow` requires a URL or a
DOI in the `source` field of every entry Part V argues from — it caught the
V-Dem entries on its first run, which named the OWID series without linking
them. It establishes that a locator is present, never that it resolves today.

**The reader now has a path.** The registry had never been named to a reader
anywhere in the ordered inputs. Part V's closing section says what it is, that
it ships with the book in the public domain, that derived figures carry the
script and the data snapshot, and that disputed figures and mismatched
denominators are marked in the entry and in the sentence. Registry: 69 entries
to 71; `registry/check.py` passes.

**Item 10 — causal and institutional justification, landed 2026-09-19.** Part V
distinguishes descriptive country associations, causal hypotheses, normative
reasons and consequences of the formal model. The income-adjusted democracy
correlation does not allocate causation between income and political freedom;
regime-group differences do not estimate transitions; absolute residuals of
country means do not measure the lower tail of individual wellbeing; and
different significance labels do not establish a difference between estimates.
The existing merged snapshot is reproducible without network access through
`registry/fetch/vdem_happiness.py --from-snapshot`. Its latest observations are
selected separately by series, with differing years in three of 141 countries.
The original numerical results are unchanged. The source record and registry
state the correction, preserve the historical comparison as historical, and
add the primary methodological references. Historical institutional examples
are also narrowed where they support a hypothesis rather than an isolated cause.

The permitted argument channel compares the principal choices with credible
alternatives: resource-specific scarcity priority and a lottery; unranked
recognition and no constitutional recognition; voluntary professional staffing;
divided appointments and majority or professional appointment; permanent and
act-specific answerability; provisional disclosure protection and a hearing
before protection; current authority and carry-forward authority; bounded and
linked personal records; territorial and population representation; collective
and prime-minister executives; residence and citizenship; usable essentials and
an income guarantee; public and voluntary funding; fiscal discretion and a
balanced-budget rule; and bounded emergency power and derogation. Each states
the reason, the cost, safeguards and evidence that would weaken the preference.
None treats compatibility in Nibli as evidence of institutional performance.

The comparison exposed a repairable defect in item 04's shield settlement.
Naming all reviewers made an unrelated prosecution impossible under its
`~show` exclusions. The controlling supersession is
`book-1/appendix/decisions/shield-scope-decision.md`: replace those exclusions
with case-specific reviewer authority and positive qualification by two
separated, authorised functions. The two deciders and two qualification
functions are distinct and cannot be the defendant, injured person or
prosecuting Court. An authorised actual-conflict finding blocks its participant
across favourable certifier pairs; a disclosure alone does not. Missing
qualification retains protection. The shield, standing, floor and credibility
rules do not change. This resolves allegation-as-disqualification; it does not
prove independence, a hearing or delivery in the world. The source has 7,110
rules, with the predicate inventory and strata unchanged.

Chapter 24 describes the current sequence, including qualification, later
disclosures and a subsequent conflict. Its focused check passes 94 pins in two
cases (11.68s). The isolated qualification matrix adds 57 cases and 456 pins,
including the earlier veto as an explicit counterfactual, each missing or
wrong-case premise, fused functions, parties in decision roles, actual
conflicts, certifier shopping and uncredentialed noise. All 15,939 pre-existing
case specifications and base definitions are preserved. The complete verifier
passes 87,372 pins across 15,996 cases with complete contradiction checks and
no findings in 1,091.95s; nine pre-existing known-defect pins still reproduce.
The five-minute target is not met. The registry passes with 74 entries; the
offline reproduction and snapshot round trip pass.

Development checks use `RUST_MIN_STACK=67108864 cargo test --release --locked`:
14 amendment tests pass (129.54s), the authoring run passes 128 with five
explicitly ignored and finds one stale Chapter 24 wording reference (765.27s).
Restoring the accurate referenced phrase resolves it; all six reference tests
then pass (1.40s). The runner passes 32 tests with three explicitly ignored
(0.06s). The initial default-stack debug run overflowed; its larger-stack debug
retry was interrupted, not counted as a pass. `cargo fmt --all --check` reports
pre-existing formatting differences; comparing both touched Rust files with
HEAD confirms identical formatting changes were already requested there.
`git diff --check` passes.

The exact changed reader prose is retained in its canonical files as
`session-drafted, author-approved under delegated approval (2026-09-13)`.
Separate prose review confirms 73,428 derived words against 28,413 other words
across all ordered inputs, including the epigraph. The broader Part V rewrite
is recorded under item 11 below; it retains the corrected inference and the
substantive qualification rule while removing the older inventories and revision
narration.

**Item 11 — Part V's five substantive tests, landed 2026-09-19.** The chapter is
rewritten around valuation, rotation, coercion, capture and the state. Each
joint states a strong objection, the current mechanism, the scope of its
evidence, a credible alternative, the cost and a reasoned conclusion. Detailed
constitutional inventories return to the derived chapters that own them. The
current chapter describes no superseded design or repair sequence and makes
no inference from AI agreement, human endorsement or invented reader testimony.
The exact text is retained in `book-1/31-the-five-joints.md` as
`session-drafted, author-approved under delegated approval (2026-09-13)`.
The same approval covers the method's short opening bridge, which is adjusted
to match Part V's actual closing promise; the full method rewrite is recorded
under item 12 below.

The controlling narrative-register decision retains the exact hypothetical
kitchen paragraph and supersedes the old passage and the requirement to quote
the reviewer corpus. Its generic household acquires no cast member's inner
life, and the passage makes no report of actual provision. The existing
assurance-portfolio decision retires recurring verdict labels in favour of
complete argumentative conclusions; their posture remains Reasoned. No
additional prose channel, aggregate score or verification gate is created.

The argument preserves the distinction between current authority and lasting
answerability, request and appellate relief, legal housing conditions and
recipient-side evidence, and each of those and actual operation. The shield
comparison uses item 10's qualified independent case review, not allegation
as disqualification. It also distinguishes a rule against appointment capture
from evidence that appointments obey it, and a supplied custody judgment from
an independent computation of necessity. The identical-input limit is stated
only for what a reasoner can infer from its premises, not as an impossibility
of institutional investigation or repair. Book 2 owns operation and transition;
that boundary neither defends a constitutional defect nor asserts that this
constitution is a working society.

The coercion argument compares confinement with compulsory non-carceral
remedies as well as voluntary restoration. A rejected voluntary settlement
does not itself warrant imprisonment. The preference for retaining a bounded
power is distinct from establishing the necessity of any particular use;
the supplied judgment is not treated as the machine deciding defences,
alternatives or a sentence.

The primary-source pass corrects the EPI denominator to industry-average
production and nonsupervisory compensation, pins its 2025 estimate, and replaces
an undated MONDRAGON pay-spread claim with the company's dated 2021 reported
scale. Its 2024 workforce is workers, not a membership count. Kerala's source
counts people trained rather than only unpaid volunteers; the Auroville
citation reaches the actual governance judgment and paragraphs. Unnecessary
settler, successive-constitution and telex-machine counts are removed. The
nine historical cases remain, with differing populations, contested findings
and causal limits stated where the argument uses them.

Existing source records receive 25 changed reference occurrences, corresponding
to 19 old passage locators, across five JSON files. Their targets resolve
uniquely, and no other JSON value changes; this renews no historical assurance
or retired workflow. The existing claim-to-source test keeps its 22 distinct
bindings, updated to the retained claims. The registry passes with 75 entries.
All five claim-discipline tests pass (1.48s); all six reference tests pass
(1.71s after the final prose edits). All 14 chapter footnotes are used,
defined and distinct, and the exact domestic paragraph matches its decision.

The complete verifier passes 87,372 pins across 15,996 cases with complete
contradiction checks and no findings in 1,080.57s (18m 1s). Nine existing
known-defect pins still reproduce; their expected results are not treated as
repairs or evidence that the whole design succeeds. The five-minute target is
not met. The constitution and all substantive pins are unchanged.
`git diff --check` passes.

Separate prose review measures Part V at 5,847 words, down from 12,293. Across
all ordered inputs, including the epigraph, 73,428 words are derived and 21,921
are other prose: 95,349 total, approximately 77% derived. The appendix and
source archive remain outside the reading sequence and length measurement.

**Item 12 — the optional method, landed 2026-09-19.** The method is rewritten around
complete examples of standing and debt, receipt evidence, and appeal stages,
followed by the distinct meanings of input refusal, contradiction checking
and a reproduced defect. It explains the choice of Nibli, closed-world and
closed-domain assumptions, variable joins, conditional inference, and the
separation between source and record changes. It describes the current design
without the book's development history or an inventory of every rule family.
The exact canonical `book-1/method.md` is
`session-drafted, author-approved under delegated approval (2026-09-13)`.

The appeal example includes the requester's lack of a permission requirement,
the reader's positive mandate, the resulting duty, Nia's granted relief and
Ruk's continuing custody. It explicitly identifies the universal hearing-duty
rule as an added test control, not an already resident guarantee for every
case. The automatic-relief encoding remains an executed refusal. The custody
account also recognises the qualified unrelated-prosecution route rather than
claiming there is only one producer. The floor explanation distinguishes an
event body's contribution to the dependency graph from a supplied actuality.

The contradiction account names represented constraint checks and their
completion limit, rather than unrestricted first-order consistency. The case
inventory's scan setting is explicit: deliberate counterfactual and transitional
exercises can keep their expected results without claiming a clean scan of
each altered source. `UNKNOWN`, resource exhaustion, rejection of an input and
`FALSE` are not interchangeable. The method identifies the current targetless
law-label and absence-based isolation defect expectations without treating a
passing defect pin as a repair. Source enactment, trusted in-memory host checks,
authentication and operation remain separate claims. It asserts no independent
reimplementation, external validation or real institutional success.

The rewrite resolves 17 method-reference occurrences (16 distinct locators)
across five existing source JSON files, including five previously stale
references. No other value in those records changes; historical assurance is
not renewed. The five resolved exceptions are removed from the existing
reference test. All remaining declared historical exceptions are preserved.

The quotation test initially saw only five of the eight displayed rules:
it joined a preceding pin directive to the rule and then failed its minimum
count. Its extractor now begins at `all` and retains wrapped continuation
lines, so directives and verdict comments cannot hide a quoted rule. The
existing minimum and exact source comparison stand. A deliberately malformed
rule after a refusal directive is rejected as absent from every source file;
the exact manuscript was restored. All five claim tests then pass (0.74s),
all six reference tests pass (1.33s), and the runner's contradiction and
incomplete-scan development check passes (0.01s).

Focused execution passes 175 pins across six cases: chapters 1, 3, 5 and 21,
the delivery suite, and its provider-independence counterfactual. Individual
runs take 10.66–11.76s. The complete verifier passes 87,372 pins across 15,996
cases with complete contradiction checks and no findings in 896.15s
(14m 56s). Nine existing known-defect pins still reproduce; these are not
treated as repaired by this explanatory rewrite. The five-minute target is
not met. The final quotation check passes again (0.96s), and
`git diff --check` passes. The constitution, suite inventory and all
substantive pins are unchanged.

Separate prose review measures the method at 3,083 words, down from 8,544.
Across all ordered inputs, including the epigraph, 73,428 words are derived
and 16,460 are other prose: 89,888 total, approximately 81.7% derived. The
scope remains technical exposition in the existing optional, unnumbered
channel. No source or appendix material becomes an additional argument
channel.

**Item 13 — opening promise and epigraph, landed 2026-09-19.** The
exact canonical opening and epigraph are `session-drafted, author-approved
under delegated approval (2026-09-13)`. The current presentation and exact
translation are recorded in `narrative-register-decision.md`. The opening
states the constitutional question, intended reader, proposal, structural
child test and paired prisoner test before sending the reader to chapter 1.
It preserves the universal-standing example, methodological claim as argument,
affect warning, title and distinction between a record, a consequence and
outside delivery. It promises no operation or justification by formal logic.

The first chapter link follows 616 words instead of 2,497. The complete
opening, including optional references, is 3,060 words instead of 7,349.
Contents, glossary, roles, cases, subject index and prose-equivalent diagrams
remain in that same exempt channel, accessible from a linked map. No argument
moves to an appendix or a new chapter. The complete manifest arc, chapter
openings, section purposes and endings were compared with the new contents:
provision and ordinary life lead into institutions, coercion and repair, then
the five arguments and optional method. The order is explicitly editorial,
engines before breaks. Subject routes for records, emergencies, mobility and
private life now name their owning chapters. The opening discards its own
development history, unsupported first-reader response, and detection-as-success
defence. The later chapter-wide developmental and line edits remain separate
open items.

The epigraph preserves the carried Tamil wording and supplies a plainer
English rendering with explicit verse breaks. It retains food, petty talk,
suffering inflicted and endured, ageing, death and the speaker's closing
challenge; it does not recast the whole poem as an argument against earning a
meal. Project Madurai's primary transcription was read with the surrounding
stanzas on 2026-09-19. Its heading is Yoga Sakthi / Varam Kettal, so the
reader attribution uses Varam Kettal (Asking for Boons), stanza 4. The existing
`bharati-yoga-siddhi` registry entry retains its identifier and records the
source title and canonical translation status. Historical versions remain in
Git and the legacy manuscript. This is an editorial rendering, not a claim
of independent scholarly review.

Seventeen locator occurrences in the existing full-society ledger follow
the retained opening material; all other JSON values are unchanged. Existing
reference tests pass (six, 1.35s), as do claim-discipline tests (five, 1.17s),
the child-section membership check and the registry check (75 claims).
Twelve opening anchor destinations resolve. A comparison confirms that the
Tamil words are unchanged; the decision quotes the exact canonical English.
The focused chapter 1 execution passes all 58 pins in 10.66s. Initial
`git diff --check` rejected Markdown's two-space verse breaks; CommonMark
backslash breaks preserve the lines and pass the check. A CommonMark render
confirms fourteen hard breaks across the two eight-line stanzas; this checks
markup, not assembled page layout. The final reference check also passes
(six tests, 1.53s). The full verifier passes 87,372 pins across 15,996 cases
with complete contradiction checks and no findings in 923.51s (15m 24s).
Nine existing known-defect pins still reproduce; none is claimed repaired by
this prose edit. The five-minute performance target is not met. The source,
suite inventory and substantive pins are unchanged.

Across ordered inputs, including the epigraph, 73,428 words are derived and
12,153 are other prose: 85,581 total, approximately 85.8% derived. The three
exempt channels, unnumbered poem and method, and Book 2 boundary remain.

**Item 14 — developmental edit of every numbered chapter, landed 2026-09-19.**
Chapters 1–31 were read and revised in order for purpose, progression, concrete
cases and endings. The exact canonical Markdown of all 31 numbered chapters
is `session-drafted, author-approved under delegated approval (2026-09-13)`.
The source remains authoritative; this pass changes the reader projection,
pin comments and existing prose locators, not constitutional rules or active
expectations. It adds no argument channel and claims no external operation.

The opening chapters distinguish standing, entitlement, initiation, specific
care authority and delivered provision without making absence prove either
success or deprivation. The ordinary-life chapters separate wages,
contribution, competence, recognition, property, environmental interests and
care roles. Concrete receipt, accommodation, ecological stay, consent and
withdrawal cases carry the distinctions. The institutional chapters explain
source, scope, current authority, review and continuity. The fresh-election
case contrasts complete positive deadline findings with the same record
without those findings; naming a deadline creates no authority or election.

The closing derived chapters distinguish shield, void, recognition loss,
custody, placement, physical holding and release. They describe current rules
and bounded counterfactual results, rather than their own repair history.
They no longer turn a rejected encoding into proof that every alternative is
impossible, a derived destination into an actual move, or a custody housing
conclusion into a delivered roof. The last derived chapter follows markers
through readers and correction duties without presenting a defective signal
as sound or disclosure as repair. Part V retains the five arguments, evidence
and alternatives, with its shield account aligned to the raw-deceit route.

Every required child section retains the one-line fixture and states the
local consequence or limit. The five documented exemptions remain. Chapter 7
merges overlapping duty and response sections; the existing coverage rows
follow those passages, preserving the substantive topics. Other renamed
sections, settings and exact locators are reconciled in the existing source
JSON. One previously stale chapter 27 locator is resolved and removed from
the existing test's exception set. The pre-existing stale Part V closing
heading in coverage, left after item 11, is also corrected. Historical audit
claims are not renewed, and no report or verification workflow is introduced.

Separate prose review measures 40,516 derived words and 12,200 other words,
52,716 across all ordered inputs including the epigraph, approximately 76.9%
derived. The previous totals were 73,428 and 12,153, or 85,581 overall. Cuts
follow chapter purpose and repetition, not a percentage target. The complete
line edit and assembled-format inspection remain the subsequent items.

Focused execution passes 1,304 pins across 32 case executions, covering every
derived chapter, including the two selected counterfactual companions.
Individual runs take 10.34–24.13s. The existing coverage tests pass (ten,
0.30s), including child-section and fixture checks; reference tests pass (six,
1.58s after final chapter 3 alignment); claim-discipline tests pass (five,
0.87s). The latter retain all Part V empirical bindings. The reference test
is strengthened by removing the resolved exception. A direct comparison of
all 24 edited pin files with HEAD confirms that every active statement,
expectation, refusal, scoped control and trusted precondition is unchanged.

The complete verifier passes 87,372 pins across 15,996 cases in 1,025.43s
(17m 5s), with contradiction checks complete and no findings. Nine existing
known-defect pins still reproduce; their passing results are not repairs.
The five-minute target is not met. The constitution and suite inventory are
unchanged. The final diff check passes.

**Substantive disposition remains open under item 19.** Correct exposition is
not a repair of the underlying design. The existing targetless amendment
label and absence-based isolation expectations still describe defects. The
source review also establishes or identifies these concrete questions for
repair and adversarial disposition:

- Raw deceit can withdraw the disclosure shield and recognition without the
  Review judgment required by the deceit-void route. The conviction rule's
  treatment of defences and the unscoped broken-Court input also need the
  adequacy review already required by the backlog.
- Paired voiding binds a subject and ground kind, not a separately identified
  incident. A currently void signer can still co-sign in the same record;
  the refusal of a direct negative guard does not prove the problem insoluble.
  The parent-judge route ignores the clean state, while a void target without
  personhood misses the explanation duty. The generic Appeals relief pair
  has no case or purpose binding. These are distinct from the independently
  qualified, case-specific unrelated-prosecution route.
- Proposal and person names share the false conclusion. The docket guard
  limits the borrowed void but does not type-separate the person from the
  proposal. A test lacking docketing is not proof of separation.
- Withdrawing holding authority also withdraws its dependent accuracy,
  security, retaliation, notification, contest and review duties in the
  record-power sequence. Requisition withdrawal drops the associated
  inventory, return and compensation duty; defence-structure withdrawal
  drops ceiling, appropriation and audit duties. Separate correction duties
  do not by themselves establish protection for retained data or past acts.
- Severity's harm entries have no writer or incident binding to the cited
  case, and severity attaches to a person across placements. The placement
  alarm covers only reported ineligible Homestay, not every conflicting or
  inappropriate destination. A destination-free housing conclusion does
  not complete a lawful placement. The raw release entry and temporal
  challenge intake likewise need their authority and continuity limits
  assessed, rather than an assumption that a label performs an act.

These questions require the resolve-before-defending standard, with exact
case tests, substantive repair where possible, and justified constraints and
alternatives for any retained limit. They are not delegated to Book 2 merely
because operation also needs outside evidence. Pure input-versus-world limits
must remain distinguished from missing constitutional safeguards.

**Item 15 — complete sequential line edit, 2026-09-19.** All 34 ordered
inputs were read in order: the epigraph, opening, chapters 1–31 and optional
method. The exact canonical Markdown of the opening, all numbered chapters
and method is `session-drafted, author-approved under delegated approval
(2026-09-13)`. The epigraph retains item 13's exact text and approval. The
canonical hypothetical household paragraph also remains unchanged and matches
the narrative-register decision exactly. No personal testimony, empirical
finding or external operation is added.

The edit removes repetitive introductions, unexplained metaphors and
unnecessary counted claims, while retaining qualifications that change the
meaning. Headings identify their subject or tested consequence. Employment
terminology includes unpaid care without confusing recognition with a worker's
legal relationship. Negation distinguishes missing evidence from an outside
failure; authority, entitlement, duties and their performance remain separate.
The account of subsidiarity explains the existing local, regional and common
limits from the source, rather than leaving the term unexplained. Part V
identifies its arguments as arguments, and the method distinguishes a query's
TRUE result from the truth of supplied evidence. Historical evidence and
sequences within the current model remain; no reader passage narrates the
book's own development.

The formal source, every pin file, suite inventory and runner are unchanged.
The substantive questions recorded under item 14 remain open for item 19;
this edit does not claim to repair them or transfer them to Book 2. Existing
coverage retains its rows and updates 24 headings. Twelve full-society ledger
locators and one placement-audit locator follow the edited text; a comparison
confirms all other JSON values and structure are unchanged. Historical audit
claims are not renewed, and no new report or verification gate is added.

The defined whitespace count across all ordered inputs is 39,872 derived
words and 12,169 other words: 52,041 total, approximately 76.6% derived.
The previous total was 52,716. The majority-derived rule, three exempt
channels, unnumbered epigraph and method, required child sections and their
documented exemptions remain intact. The reduction follows meaning and
rhythm, not a percentage target.

Focused execution passes 193 pins across six cases, covering chapters 3, 5,
9, 14, 19 and 21, with individual runs of 10.61–12.47s. Existing coverage tests
pass (ten, 0.25s), reference tests pass (six, 1.53s after final polishing),
and claim-discipline tests pass (five, 0.86s), including the method's exact
formal quotations. An intermediate reference check found a chapter 7 locator
made stale by the edit; it was corrected and the check passes. Separate prose
review caught and corrected a grammar error introduced while tightening
chapter 8. The diff check passes. The complete verifier passes 87,372 pins
across 15,996 cases in 940.86s (15m 41s), with contradiction checks complete
and no findings. Nine declared known-defect pins still reproduce; their
passing results are not repairs. The five-minute target is not met.

**Item 16 — navigation and assembled review formats, 2026-09-19.** The
current manifest, chapter titles, opening contents and numbered references
agree. The three original suspected references were already corrected by
earlier items: the opening describes editorial order, chapter 14 identifies
contribution in chapter 10 and leads next into mobility, and chapter 29 sends
asylum to chapter 15. Chapter 11's reference to chapter 9 concerns the
purpose-bound contribution supplement and reaches the intended material.
No ordered manuscript prose or substantive source changes in this item.

`tools/build_book.py` assembles the 34 current inputs from `contents.json`
as HTML, EPUB and PDF review copies in ignored `output/book-1/`. CommonMark
parsing preserves list continuations, verse breaks and linked footnotes.
Chapter links and fragments stay within the assembled edition; formal-source
and registry links identify actual repository files and follow `main`, without
claiming an immutable edition binding. Nested contents, numbered chapters,
unnumbered epigraph and method, labelled table regions and column headers,
keyboard navigation, font embedding and PDF bookmarks are supplied explicitly.
The old pilot assembler and artifacts remain historical; their retired receipt
and freshness workflow is not restored. No extra verification gate is added.

The exact new documentation in `README.md`, `book-1/README.md` and
`tools/book_assets/README.md`, and the review-copy front matter in the builder,
is `session-drafted, author-approved under delegated approval (2026-09-13)`.
The unmodified Noto Serif Tamil font comes from Google Fonts' upstream
repository, with its SIL OFL 1.1 text retained and embedded or bundled in each
format. Font licence whitespace is normalised without changing its wording.
The book's mixed licensing remains in force; historical files do not acquire
a new licence merely by appearing under `book-1/`.

Six new assembler development regressions pass, covering continued lists,
verse breaks, repeated footnote return links, balanced-parenthesis URLs,
separate chapter IDs, local destinations, unsupported markup and the current
EPUB spine and contents. Their command is `uv run --with markdown-it-py==4.2.0
--with mdit-py-plugins==0.6.1 python -m unittest discover -s tests
-p test_build_book.py` (0.117s). Existing reader-coverage tests pass (ten,
0.22s), reference-integrity tests pass (six, 1.81s after the documentation
changes), and claim-discipline tests pass (five, 0.87s).

Actual rendering used Chromium 151.0.7922.34 through Playwright 1.63.0.
HTML and all EPUB spine documents plus its navigation were rendered at 360px
and 1280px: 74 renders, with no missing in-document destinations, duplicate
IDs or horizontal document overflow. All six tables have labelled regions
and scoped column headers; all 14 note references and their return links
resolve. Keyboard activation of the skip link reaches the main content.
EPUBCheck 5.4.0 reports zero errors and zero warnings under its EPUB 3.4
rules after correcting package IDs and navigation markup. This is the
[W3C tool's released validator](https://github.com/w3c/epubcheck/releases/tag/v5.4.0),
not an external reader assessment.

The final PDF has 154 pages, 268 bookmarks, 174 internal link annotations,
58 external link annotations, a structure tree and embedded prose/font
licences. Every internal annotation has a real page destination. All pages
were checked for text bounds and empty output and viewed as contact sheets;
the epigraph, contents, tables, notes and method were also inspected at
reading scale across the formats. The final corrections keep short code
examples together and leave room for italic note text at the right margin.
Whitespace-normalised comparison finds all 1,383 source paragraph, list,
heading, table-cell and code blocks in the extracted PDF text. Tamil text
also extracts correctly; its printed glyphs are carried in a Type 3 font.
No clipping or reading-order defect was found in the inspected output.
EPUB inspection covers its packaged XHTML in Chromium and package validation;
it does not claim testing in every ebook application. No human accessibility,
screen-reader usability, comprehension or endorsement is claimed. The nine
modeled defects and item 14's source questions remain open for item 19.

Two initial attempts at the complete verifier ended with SIGTERM (exit 143)
without a final summary. Their last progress lines were at 275.5s and 292.1s;
neither counts as completed verification. The subsequent live-terminal run
of `RIGHTS_VERIFY_JOBS=4 ./verify.sh`, with regular session polling, completes:
87,372 pins pass across 15,996 cases in 959.11s (15m 59.11s), with complete
contradiction checks and no findings. Nine declared known-defect pins still
reproduce; this is not a repair of those defects. The five-minute target is
not met. The final diff check passes.

**Item 17 — practical contribution guidance, 2026-09-19.** The new
`CONTRIBUTING.md`, linked from both repository and book READMEs, gives an issue
route that requires no formal tooling and a focused branch/PR route against
`main`. Its file map separates prose, evidence, rules, owning generators,
rendering and Book 2's inactive collection. It explains the current-design
and resolve-before-defending constraints, appropriate focused and complete
checks, meaningful regression coverage, credit for corrections and shared
drafting, material AI assistance, editorial responsibility and independent
adaptations. It creates no new submission form or approval body and promises
neither acceptance nor a publishing agreement.

The exact new guide and documentation changes are `session-drafted,
author-approved under delegated approval (2026-09-13)`. The chapter 5 example
is explicitly an illustrative proposal, not an edit to that chapter or a claim
of a received contribution. Its context matches one current source line.
No external contributor, endorsement, submission or operation is invented.

Registry guidance now matches the existing 2026-09-15 claim-binding ruling:
Part V's figures remain hand-written with readable citations; internal IDs
belong in the existing bindings. Only the `spec.entry_fields.id` description
changes in `registry/claims.json`; a structural comparison confirms every
claim and all other values are unchanged. Snapshot guidance distinguishes CC0
claims, MIT OR Apache-2.0 scripts and upstream data licences. The retained
field/date utility is described accurately without making its freshness
threshold a completion gate. Prior licence grants remain in force.

The example's focused command passes 16 pins in one case (10.51s), labelled
partial. The guide's exact development commands pass the existing coverage
tests (ten, 0.24s), reference tests (six, 1.37s) and claim-discipline tests
(five, 0.81s). All 36 local documentation links and fragments resolve.
The ordered manuscript, constitution, substantive pins, suites, runner and
rendering machinery are unchanged. Item 14's concrete source questions and
the declared modeled defects remain for item 19.

The complete `RIGHTS_VERIFY_JOBS=4 ./verify.sh` run passes 87,372 pins across
15,996 cases in 901.84s (15m 1.84s), with complete contradiction checks and
no findings. Nine declared known-defect pins still reproduce. Their passing
expectations do not close the defects, and the five-minute target is not met.
The final diff check passes.

**Item 18 — publisher proposal and selected sample, 2026-09-19.**
`submission/README.md` supplies a 1,590-word publisher-neutral proposal:
synopsis, readers, distinct contribution, all current contents, representative
chapters, completion status, open contributions, mixed licences and the desired
editorial/print partnership. Its comparisons with Rawls, Wright and Ostrom
link to publisher descriptions, including Harvard/Belknap's description on
JSTOR. They claim neither novelty over those works nor endorsement or sales.
The author is identified only as dhilipsiva; no biography, external review,
reader testing or operational success is invented. No publisher was selected
or contacted. A named-press adaptation must use that press's current official
requirements; open-access support does not imply acceptance of open editing.

The exact proposal and accompanying README prose are `session-drafted,
author-approved under delegated approval (2026-09-13)`. The synopsis describes
the limited supplied birth record without converting absent entries into facts
about the child's actual life or ability. The proposal remains outside the
ordered reading sequence and adds no argument channel to the book. It states
that nine modeled defects and item 14's further source questions remain for
item 19, followed by the fresh whole-manuscript review. It calls for updating
the proposal and sample to that resulting text before submission. Preparing
the package does not close those issues or release Book 1 or activate Book 2.

The assembler's explicit `--sample` option selects complete chapters 1, 5, 8,
21 and 31 in manifest order. Separate `book-1-sample` filenames and an EPUB
identifier distinguish the selection from the full review copy. It preserves
chapter numbers and citations, labels the cover, retains internal navigation
within the sample and sends omitted-chapter references to the public `main`
manuscript. Those mutable links do not bind an immutable submitted edition.
The PDF outline handles a selection without an epigraph. New regression
coverage checks the selected order, missing selections, omitted-chapter links
and the packaged EPUB's spine, navigation and local destinations; all seven
assembler tests pass (0.202s).

The 34 ordered inputs still contain 52,041 whitespace-separated Markdown
words: 39,872 in derived chapters (76.62%). The sample contains 10,009 and
renders to 29 PDF pages with 36 bookmarks, 33 internal and 36 external links,
tagged structure and embedded licence notices. Sixteen browser renders cover
the HTML and all seven packaged EPUB XHTML files at 360 and 1,280 pixels:
no overflow, missing local targets or duplicate IDs. The labelled table,
keyboard skip link, EPUB chapter navigation and footnote/return links work
in Chromium. All PDF pages were inspected on contact sheets, with mobile
chapter and note views inspected separately; no out-of-page text or empty
pages were found. All 208 source text blocks checked occur in the PDF after
excluding its numbered footers. This is rendering inspection, not external
reader assessment or testing across all ebook applications. EPUBCheck 5.4.0
reports zero errors or warnings under EPUB 3.4 rules.

The proposal's numbered contents match the manifest, and all 29 local links
and fragments in the affected documentation resolve. The full HTML output
matches its previous render except for one trailing space in the embedded
font licence already cleaned during item 16. Temporary inspection checks
were corrected to use the renderer's actual `targets` field and exclude PDF
page-number footers before comparing text; neither initial helper failure
was a manuscript omission. Existing coverage tests pass (ten, 0.19s), reference
tests pass (six, 1.40s), and claim-discipline tests pass (five, 1.00s).

The complete `RIGHTS_VERIFY_JOBS=4 ./verify.sh` run passes 87,372 pins across
15,996 cases in 895.11s (14m 55.11s), with complete contradiction checks and
no findings. Nine declared known-defect pins still reproduce and remain open
for substantive resolution in item 19. The five-minute target is not met.
The ordered manuscript, constitution, pins, suites and verifier are unchanged.
The final diff check passes.

**Item 19 — final substantive repairs in progress, 2026-09-19.**
The amendment subtask removes only the two Article 9 label rules; a comparison
of active source lines confirms no other constitutional rule or fact changes
in this subtask. The existing 15,996 case configurations and 114 bases remain
identical; two cases are appended for the same facts on the live source and
an explicit weaker-rule counterfactual. The source-mutation scenarios retain
their adverse rules, refusal controls, source edits and source-effect results.
No actual amendment-enactment generator or interface changes.

The initial five-pin docketed-Jala regression fails on exactly three intended
properties: recognition, personal credibility and recognition loss (10.5s,
three findings, no harness errors). With the two weaker rules removed, the
expanded seven-pin case passes in 10.34s. Seventeen further focused selections
pass 196 pins over 17 case executions in 184.25s total: the weaker-rule
counterfactual, chapter 22, the floor suite, AS-01–AS-09, certification,
publication, effective selection, missing consent and competing selection.
This is focused evidence, not completion of the final full verifier.

Chapter 22, the method's amendment example, the controlling contract and
their exact current documentation changes are `session-drafted,
author-approved under delegated approval (2026-09-13)`. The prose describes
current candidate-bound authority and a deliberately weakened counterfactual;
it does not narrate the book's repairs. Existing coverage and design records
follow the changed mechanism and distinguish historical audit measurements.
The seven isolation defect expectations and all other item 14 source questions
remain open. Item 19, the submission refresh and the fresh full reading in
item 20 are not complete.

The initial reference checks identify the removed source and prose locators;
those existing entries are updated rather than added to the exception set.
Coverage tests pass (ten, 0.19s), final reference tests pass (six, 1.37s),
and claim-discipline tests pass (five, 0.77s). The diff check passes.

The credibility subtask implements a different representation, superseding
the claimed necessity of the same-record signer window in
`book-1/appendix/decisions/credibility-finding-decision.md`. Effective historical
findings bind a subject, incident, ground, evidence, independent reviewers,
procedure and eligibility at decision. Permission to make a new finding is a
separate conclusion and reads current disqualification and family conflicts.
An appointment alone does not permit a currently voided reviewer to sign.
Conflicting case bindings fail closed. The source does not authenticate the
outside historical witnesses or manufacture a completed judgment from a
prospective permission.

Raw deceit allegations no longer remove examination recognition or disclosure
protection. Disclosures have explicit incident identifiers. Restoration binds
the particular finding and appellate act; a generic judgment or forgiveness
flag does nothing, and another finding remains effective. A parent's judgment
does not automatically impose a personal finding. Every target of an effective
finding is owed reasons without a separate personhood entry. A carried
credential restriction cannot alone withhold recognition; the cast's existing
personal consequences now have explicit fictional case records. The existing
decision and source changes are `session-drafted, author-approved under
delegated approval (2026-09-13)`.

An isolated eight-pin prototype passes. The first full-source attempt rejects
a negative dependency through the appointment relation; the corrected
prospective-act representation passes eight pins, then 28, 50 and 55 expanded
pins. The actual constitution passes the 55-pin regression in 10.94s and again,
with the original unscoped examinations and raw accusation retained, in 11.22s.
The explicit counterfactual reinstating the unscoped immediate-effect rule
passes four pins in 10.10s and reproduces the currently voided signer's adverse
effect. These focused runs include their requested contradiction checks. The
following chapter-24 selection exposes an obsolete exact source-edit locator
in the no-dead-conjuncts counterfactual; it is not a completed selection. The
counterfactuals, chapter fidelity pins and reader projection are being migrated
to the prospective act. No complete item-19 verification has run, and the other
item-14 questions remain open.

The migrated chapter 25 passes 198 pins over the live and declared stripped-guard
variants. It retains the original scenarios and old-policy single-signer
control, adds incident-bound permissions and completed histories, and tests
restoration of an adjudicated parental conflict without erasing the underlying
relationship. A further red case finds that a renamed restored finding could
revive its consequence: three failures in 58 pins, no harness errors. The fix
binds restoration to subject, incident and ground across record names and
evidence labels, and closes the prospective route for that restored case.
Separate grounds and incidents remain separately adjudicable.

The repaired regression passes 62 pins in 11.03s; chapter 24 passes 102 pins
across two source variants in 10.63s; chapter 26 passes 41 pins in 10.27s.
The combined execution takes 32.57s. The two appointment-guard counterfactuals
pass six and five pins in 10.18s and 10.10s respectively (20.71s combined).
The explicit immediate-effect counterfactual still passes its four pins and
reproduces the old same-record harm. Full verification, development tests,
coverage/reference updates and the remaining item-14 repairs are still pending.
The current-design revisions to chapters 3, 10, 16, 23–26, the opening glossary
and the Part V shield discussion are also `session-drafted, author-approved
under delegated approval (2026-09-13)`. They describe the implemented case
requirements; the unrelated appellate-relief, custody, temporal, continuing-duty
and isolation questions remain open.

Restoration now explicitly separates the case-authorized decision actor and
reviewer from each other and from the subject. Qualified independent alternates
can act when an ordinary body is a party. The new 13-pin regression first
exposes eight failures; after repairing the rule and an actor-renaming fixture
error, it passes in 10.87s. Together with the 62-pin regression, chapter 25's
two variants and chapter 26, 314 pins pass across five selected executions in
46.51s with their contradiction checks. All 16 existing floor-vector development
tests pass in 1.31s. The exact chapter and decision additions are
`session-drafted, author-approved under delegated approval (2026-09-13)`.
These remain focused checks; item 19 is still open.

**Continuing duties.** Three adversarial withdrawal sequences confirm nine
lost protections: six data safeguards or contest rights, two declaration and
requisition duties, and the defence ceiling/appropriation/audit duty. The
authoring contracts now give continuing protections a separate basis in the
fully specified recorded undertaking. Current completion, permission and lack
of a later withdrawal or conflict no longer gate those duties. Exact actors,
record fields and independent witnesses remain required; no new authority or
performed act follows from a continuing duty. Record protections retain their
exact raw holding joins. Restrictions still require all their current authority
and dependency premises.

The existing generators implement the change; two additional record sequences
test withdrawal against the dependent record itself. The focused inventory
includes all 447 record-power cases and the public-safety core and review
cases, with the original sources, declared transformations and contradiction
checks. It passes 3,814 pins across 2,157 cases in 95.25s. This temporary
selection is not the complete book verifier and creates no persisted verdict
or new verification gate. The exact chapters 19, 20 and 30, contract and
decision revisions are `session-drafted, author-approved under delegated
approval (2026-09-13)`. Custody, severity, placement, temporal intake, isolation,
remaining metadata and final verification are still open.

**Positive isolation evidence.** The 37-pin red case confirms 16 failures:
the seven absence-based markers, missing complaint access, and the absence of
a positive condition-finding route. The replacement binds subject, holding,
holder, place, period and evidence to distinct authorised witness and reviewer.
Conflicting authorised descriptions block the finding; uncredentialed noise
does not. Neither actor may be the subject or holder. Human identification
and the floor survive a disputed period or procedure. A company receipt for
another encounter does not erase the recorded finding about its own period.
Routine custody review is owed without a breach finding, and a complaint opens
investigation without the holder's permission or a completed finding.

The expanded condition case passes 41 pins (10.74s); its explicit weaker-rule
counterfactual passes seven (10.00s); chapter 30 passes 37 (10.92s); and the
floor suite passes 100 (23.45s). These total 185 pins across four focused
executions in 55.11s, with requested contradiction checks complete. The floor
suite first exposes a stale incidental-cycle expectation for `~false` after
the parent-judgment route's removal. The same non-floor control is retained as
scoped acceptance; all floor refusals remain required. The seven isolation
queries retain their scenarios as ordinary negative regressions, and their
`:defect` annotations are removed after the repair.

The existing floor development check now forbids every downstream reader of
a floor actuality, with positive and negative mutation controls. All 16 floor
development tests pass in 19.10s. Reference tests expose two stale chapter-30
locators; correcting those existing references yields six passing tests in
1.35s, without expanding the historical exception set. Five claim-discipline
tests pass in 0.72s. The duty-related chapters 19 and 20 pass 32 focused pins
in 20.47s. The exact current-design revisions to chapters 4, 29, 30 and method,
the delivery decision and existing coverage/defect records are
`session-drafted, author-approved under delegated approval (2026-09-13)`.
These results do not close the remaining custody, severity, placement or
temporal-intake issues, and no complete item-19 run has yet been made.

**Custody merits, relief, execution and intake — 2026-09-20 integration.**
The prior raw-flag regression exhibits eight failures in eleven pins. Custody
now requires a separately qualified merits finding: subject, incident,
offence, evidence, victim and prosecutor; distinct uninvolved adjudicator and
reviewer; criminal proof, defences, public law, admissible evidence, counsel,
hearing, reasons, proportionality and repeat-proceeding limits; and witnessed
historical eligibility and order. An unscoped broken-Court mark is not a
universal amnesty. Qualified independent appellate relief has closed grounds
and exact case scope. Final relief follows subject, incident and offence
across renamed cases, while court-jurisdiction relief leaves a fresh lawful
proceeding possible. Raw clearing, generic judgments and completion flags
cannot replace that order.

Severity uses independently signed dimensions of the same adjudicated case.
A case cannot borrow another case's severity. Custody also requires a named,
available, independently reviewed placement, with positive necessity and
lawful-condition findings. No-home cases need positive ordinary supported-
residence evidence; absence of a home does not select HighSec or fabricate a
place. Conflicting qualified places withhold authority. Every reported
category, conflicting reports and structured wrong-place reports can trigger
review. A sentence or physical holding no longer produces delivered shelter
or speech: their duties remain, and shelter uses the independent recipient
route. Public-safety and statistics owning generators reflect this distinction.

Each custody lease needs positive independent intake access, counsel, retained
copies and reconciliation. A subject-retained submission or independent
receipt establishes a filing without the primary registry's entry. The
pending case and its aliases cannot sustain custody; a qualified uninvolved
disposition alone ends the suspension. The temporal and public-safety decisions
record the initial cyclic encodings and the separate-stage repair. The
delivery decision records why identical supplied inputs cannot distinguish
unreported delivery from non-delivery, and the duties and evidence routes
that remain required. These are bounded observation limits, not impossibility
claims about institutional reform or excuses for missing safeguards.

The extended prototype passes 377 pins across 79 cases in 14.48s. On the
actual source, 1,130 pins across 129 custody, placement and record cases pass
in 53.42s; 736 pins across 103 temporal, shield and red-team cases pass in
36.24s, with their contradiction checks complete and no findings. The old
single-signer and belief-void widening controls remain as attempted edits;
the current dependency graph refuses them. The explicit older-rule temporal
and credibility counterfactuals omit the new exact-place permission projection
whose shared predicate otherwise prevents those deliberately weaker models
from loading. The enacted projection is unchanged.

All 16 floor and dependency development checks pass in 18.31s. The complete
authoring development run reports 126 passes, three failures and five declared
ignored tests in 762.06s. The failures identify stale reference/disposition
metadata and omitted inventory defaults, not enacted-rule failures. Corrected
reference tests pass six in 1.43s, disposition tests pass three in 0.02s, and
the justice boundary test passes in 62.89s. The runner's development suite
passes 32 tests, with three declared ignored performance tests, in 0.06s.
The first complete substantive run stops at the older isolation-duty fixture;
its owning obligations generator now supplies positive condition evidence,
retaining both original reader-duty expectations. This is not yet a complete
item-19 pass.

The exact current-design revisions to chapters 3–5, 21, 23, 27–29, Part V and
method, and their controlling decisions and contracts, are `session-drafted,
author-approved under delegated approval (2026-09-13)`. The canonical household
paragraph and one-line child fixture remain unchanged. Existing source and
prose references are maintained; the two resolved constitution-reference
exceptions are removed, leaving three historical unmatched locators. No new
verification gate or persisted verdict is introduced. Final complete
verification, final artifact inspection and item 20 remain open.

**Item 19 complete — 2026-09-20.** The final complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` run passes **88,024 pins across 16,087
cases in 1,169.06s (19m 29.06s)**, with contradiction checks complete and no
findings. The five-minute target is not met. No active `:defect` annotations
remain in the execution inventory. The nine former live defects are negative
regressions under the enacted source; their explicitly weaker alternatives
retain the harmful results. The other item-14 mechanism questions are resolved
by the implemented case bindings, independent relief, continuing protections,
positive condition findings and custody contracts described above. The
necessary identical-input observation boundary is justified separately; it
does not stand in for any missing legal repair.

The amended obligations owner and all amendment and credibility cases pass
400 pins across 158 focused cases in 39.77s before the complete run. All
previously failing authoring checks pass on rerun: 129 authoring development
tests passed across the complete run and the targeted reruns, with five
declared ignored tests. All 32 runner development tests pass, with three
declared ignored performance tests. Seven book-renderer tests pass in 0.194s.
No test assertion was dropped to obtain these results. Older scenarios retain
their raw writes, assertions, stateful order and explicit source transformations;
changed legal expectations and incidental refusals are documented. Empty
fixture/edit lists and default-true scan values are explicit for the authoring
schema; they do not change native execution.

The separate prose/source review aligns the changed chapters, method, cast
cases, contracts and existing locators with the implemented rules. All Part V
figure bindings and source locators pass the existing claim-discipline tests;
this item adds no empirical statistic or study. The epigraph, exact hypothetical
household paragraph and one-line child fixture remain unchanged. Child coverage,
ordered contents and majority-derived coverage checks pass. The current
rendered-text whitespace count is 51,433 words, of which 39,385 belong to the
30 derived chapters (76.57%); the five-chapter sample is 10,036 words. The
proposal uses rounded counts and states that final editorial review is pending.

The full HTML, EPUB and 155-page PDF and the sample HTML, EPUB and 29-page PDF
are rebuilt. All PDF pages were inspected on contact sheets; 1,212 full-book
and 197 sample text blocks occur in the PDFs, with no out-of-page text or empty
pages. The full PDF has 268 bookmarks, 174 valid named internal links and 60
external links; the sample has 36, 33 and 36 respectively. Eight browser
renders at 360px and 1280px pass overflow, fragment, image and font checks;
selected mobile and desktop chapter views were inspected. EPUBCheck 5.4.0
reports zero errors and warnings for both books under EPUB 3.4 rules. Rendering
uses installed Chromium 146.0.7680.80 through the builder's supported explicit
browser option. An initial default-browser attempt lacked the expected browser,
and a cached browser lacked system libraries; neither is counted as a pass.

The exact final proposal and decision-record revisions are `session-drafted,
author-approved under delegated approval (2026-09-13)`. Item 19 is removed from
the tracker after its implementation and checks; item 20 remains open. No
submission, external endorsement, release or operational success is claimed.

**Item 20 complete — 2026-09-20.** A fresh reading of all 34 ordered inputs
from `28e88b2d` is recorded in
[the final manuscript review](reviews/2026-09-20-final-manuscript-review.md).
The assessment is 8.5/10 as a manuscript and 8/10 for publisher readiness:
editorial judgments within the same AI-assisted project, not independent
endorsements. It names the strongest chapters, serious objections, remaining
repetition, dense institutional passages and the limits of operational evidence.
No score was forced by further cosmetic work.

The opening and chapters 1, 3, 18, 24, 30 and Part V now consistently distinguish
custody duties from shelter or speech, raw allegations from case-bound severity,
proposal labels from exact-candidate authority, qualified merits from the
shield, and suspensive filings from final relief. Placement alarms and the
independent filing/intake routes are described at their implemented scope.
The chapter-18 pin comment now retains public-safety duties after permission
withdrawal. No formal rule or executable expectation changed. Existing prose
locators and the amendment record's description of the floor controls were
aligned with the current chapters and pins.

The kibbutz salary-reform share is now attributed directly to Uriel Leviatan's
paper, p. 12; Abramitzky remains the source for community count and incentive
analysis. The registry retains its stable identifier and numerical values.
Targeted primary-source checks and the bundled democracy calculation are
reported with their actual retrieval and replication limits. No comprehensive
new fact-check of every cited book or external review is claimed. All material
findings from this reading have supported repairs; none remains merely
disclosed or rhetorically defended.

The focused run passes 722 pins across 89 cases in 21.40s. The complete
`RIGHTS_VERIFY_JOBS=4 ./verify.sh` run passes **88,024 pins across 16,087 cases
in 1,198.57s (19m 58.57s)**, with complete contradiction checks and no findings.
The five-minute target is not met. No active known-defect expectations remain.
Five claim-discipline, ten reader-coverage, six reference-integrity and seven
renderer development tests pass. The two failed prose locators were repaired;
the established builder environment supplies the renderer's dependencies, and
the correctly filtered reader test run is counted instead of an initial
zero-test invocation. No new verification gate was added.

The bootstrap pin and repository overview now identify the published companion
revision actually used, `b707dad73876f7e1620ea8bb6f80242aa374c759`. Its changes
since `979fe8b` leave the engine packages used here unchanged. The overview
separates the new timing from historical CPU and memory measurements. This
does not claim a new clean-clone run or edit the companion's unrelated work.

The rebuilt manuscript contains 51,606 rendered-text words, of which 39,503
are derived (76.55%); the five-chapter sample contains 10,089. The proposal
uses approximately 52,000 and 10,000 words and links the completed review.
All 156 full-book and 29 sample PDF pages were visually inspected; all 1,212
and 197 checked text blocks occur, with no empty pages or out-of-page text.
The full/sample PDFs have 268/36 bookmarks, 174/33 valid internal destinations
and 61/37 external links. Ten browser renders at 360px and 1280px pass overflow,
fragment, image and font checks; selected views were inspected. Both EPUBs
pass EPUBCheck 5.4.0 under EPUB 3.4 rules with zero errors and warnings.

The exact revisions are `session-drafted, author-approved under delegated
approval (2026-09-13)`. The epigraph, canonical household paragraph, one-line
child fixture, child-section requirements and exemptions remain unchanged.
Item 20 is deleted after implementation and required checks, closing all
twenty items in the root revision backlog. Book 2 remains collection-only;
no publisher contact, submission, release, Gate C completion or successful
external operation is declared.

### Measured Nibli capability boundaries

Each is a dated measurement against a named engine revision, not a timeless
guarantee. Re-measure before relying on one.

The finite collective-decision boundary was measured on 2026-08-07 and
independently rechecked on 2026-08-08 against Nibli `a7d288a`, which contains
the required `4cb02aa` baseline. For definitive finite positive queries, Nibli
can observe exact facts in a supplied snapshot, find/count witnesses, aggregate
supplied weights, and compose supplied result certificates. It cannot generically
derive `ceil(2R/3)`, compare dynamic
tallies or shares, group affected regions, authenticate completeness, choose one
effective conflicting submission, or perform an institutional act. Exact-count
and compute nodes are query-only; a hand-enumerated two-of-three rule is not a
changing-roster majority rule. The external result pipeline keeps four roles
separate: election administration authenticates and classifies submissions;
independent assurance attests completeness; the result service computes and
certifies; authorised institutions execute. Missing completeness can become a
withholding veto, so every route needs alternate attestation and a failure
default. "No turnout quorum" is an absent legal condition, not an engine
primitive. No general empty-roster passage rule was supplied; each Book 1 result
card must state that default before formalisation. Since Nibli `5580618`, final
non-definitive witness enumeration fails closed rather than returning a partial
collection. At `5777ced`, text compute registration routes only
corpus-resolvable canonical relations; it declares no vocabulary or arity.
Arbitrary compute remains a caller-built native raw-IR query and is query-only
at assertion ingress. Neither repair supplies changing-roster computation or
certification.
`FALSE` does not affirmatively preserve current law or an incumbent.
See `book-1/source/nibli-finite-collective-decision-capability-audit.md`.

The versioned ecological and animal finding boundary was measured on 2026-08-11
against clean Nibli `main == origin/main` at `07734c8f`, using release
`nibli-pin` SHA-256 `87b5c7bf351e355352781905a7afedbf29f7c20b3e3d2fc69843921ba0a26f10`.
The exact binary passed this repository's full verifier. The supported seam is
narrow: Nibli may derive a bounded consequence from caller-admitted, finite,
fully keyed, case-bound ordinary facts or result certificates through the
ordinary KR/decomposed path. Authentication, scientific classification,
completeness, freshness, conflict priority, time, notification, and execution
remain external. Only a top-level definitive `TRUE` on that supported shape can
support the consequence; `FALSE` is closed-world non-entailment, while
`UNKNOWN`, `RESOURCE_EXCEEDED`, invalid input, and incomplete enumeration cannot
authorize. Flat raw body-only-variable rules, non-finite exact-zero, mixed-row
aggregation, and unsupported WIT raw-query parity are excluded. See
`book-1/source/nibli-versioned-ecological-and-animal-finding-capability-audit.md`.

The multi-power, multi-window protective-authority composition boundary was
measured on 2026-08-12 against clean Nibli
`main == origin/main == public main` at
`07734c8f7af71075cb70e91c112ff75d16a962d9`. The composition findings came
from separately digest-bound custom native probe sources and outputs recorded
in the audit; the custom executable binaries were not digest-bound. Separately,
release `nibli-pin` SHA-256
`87b5c7bf351e355352781905a7afedbf29f7c20b3e3d2fc69843921ba0a26f10`
ran the neutral text-boundary pin and passed this repository's full verifier;
those runs do not establish the custom native-surface findings. The supported
seam is narrow: exact case-bound ground queries over a raw rule shape that
retains every positive witness in its head and rejoins the authorising act and
version, measure, subject, scope, basis, authority, and window. On the dated
probe, exact keyed ground queries scaled approximately linearly; sibling
authority could not substitute, removing one measure's active authority left
the others unchanged, removing the source closed every dependent conclusion,
and a stale act version failed closed.

This audit does not establish currentness, freshness, replay protection,
conflict priority, publication, clock advancement, or institutional action.
`FALSE` remains closed-world non-entailment rather than affirmative cessation;
a duplicate or frozen renewal is indistinguishable from live evidence without
external witnesses, and a `Closed` sibling does not override a retained `Open`
record. Alternate and substitute routes derive only after their positive
records and actions are supplied. The compiled no-reader graph is an
observational current-program check, not a permanent seal. An unbound
eight-variable `find` exhibited catastrophic candidate expansion and a
process-level memory abort even with three expected rows, and an accepted flat
raw rule whose positive witness variables appeared only in the antecedent
silently under-derived. Formalisation must therefore use exact-ground queries
over the measured fully head-carried raw shape and must not rely on the unsafe
high-arity find/count/aggregate or antecedent-only-witness raw shapes. Any
ordinary-KR implementation needs separate vocabulary-admission,
compiler-lowering, compiled-shape, and behavioural validation against the
selected source before it can claim this boundary. Aggregate callers must also
validate that every projected binding is numeric because the current aggregate
silently filters symbolic or missing values. No authority-specific engine
feature is needed for the supported seam. See
`book-1/source/nibli-multi-power-multi-window-protective-authority-capability-audit.md`.

**Full-source and composed opaque floor execution restored 2026-08-05:** release
Nibli `5cec80080eea0334c87508e60813f8f70f487441` first removed the temporary
extracted-floor isolation. It repaired two independent global costs: eager
materialisation of every eligible relation and backward witness generation over
the global typed domain and every Skolem family without first using relation
anchors and bound siblings. The exact full-T3 food entitlement fell from a
180.24 s timeout at 1,850,676 KiB RSS on `225bba4` to 0.13–0.15 s and about
12,000 KiB on clean `5cec800` builds.

Release Nibli `4cb02aade43b394374c40e661907ad66df3af3fe` closes the residual
composition where the queried subject's only standing proof traverses the
high-arity T3 custody chain. The remaining cause was general: rule firing built
independent candidate vectors for unbound Neo-Davidsonian events before shared
individual-role bindings could constrain them; partial dependent-Skolem
candidates rebuilt already grounded dependencies; opaque-query candidate order
did not favour the ground subject; and an exact eligible one-positive derived
antecedent could not request its complete relation cone lazily. The repair uses
deterministic left-deep event joins under accumulated bindings, grounded/index
selectivity, guarded early role binding, query-ground dependency ordering,
partial-Skolem specialisation, and fail-closed exact relation-scoped lookup. No
book predicate or definitive logical verdict was special-cased or moved.

On binary-bound 2026-08-05 measurements, `5cec800` timed out on both the composed
Zed chapter query (180.13 s, 25,792 KiB RSS) and the generated placement query
(180.11 s, 25,912 KiB). The supplied `4cb02aa` release returned Zed TRUE in a
0.87 s median at no more than 13,364 KiB and the placement entitlement TRUE in a
0.23 s median at no more than 12,868 KiB. These are dated fixtures, not a timeless
performance guarantee. An independent clean build (SHA-256 `d5d0f86a494aedd943434c2fa0abd0acbe0d7df36142d983df5774bf2dd38999`)
reproduced the two results at 0.87 s / 13,324 KiB and 0.22 s / 12,796 KiB. Its
bytes differ from the supplied binary (`33ea9c9805b899fa054b75371c60eedc9b975d4be356a45b57de0d6323071fa7`),
so this establishes source identity and behaviour, not byte-reproducible builds.
Chapter 7 now pins the Zed composition directly. Each placement row also runs a
fresh one-pin probe against its complete generated candidate: confined standing
must traverse the actual T3 facts, with no direct `person` overlay and no
extracted floor source.

`EventPath` and `RecordPath` are third-place constants in ternary `precede/3`, not
binary relation heads. The T3 chain amplified both engine failures but was not
their complete cause; opaque abstraction exposed witness expansion but was not
required for the separate eager-materialisation defect.

### Historical — the protocol-v4 Stage 4 record

**Supersession note, 2026-08-27.** The following Stage 4 paragraphs are the
preserved protocol-v4 historical record. Protocol v6 supersedes their
current-audit and commit-sequencing statements for every new candidate;
current Gate A state is derived from `full-society-ledger.json`, not from the
historical prose below. The exact v1 closure remains accepted only by the
legacy tuple in the scope-review protocol.

**Stage 4's repository-adversarial protocol and checker are complete
(2026-08-15); Gate A closure is mechanically derived.** Schema v6 adds
append-only `scope_audits`. Each audit binds the exact source version,
validator-derived semantic scope digest, protocol digest, UTC execution time,
declared criteria, checker control entry points, command chain,
Gate-A-applicable finding set, result, policy basis, and byte-exact evidence
ceiling. Historical audit rows retain their recorded basis verbatim. Visible normal first-parent Git history preserves committed audit,
commission, proposal, and event prefixes under that bounded history model; it
does not prove resistance to rewritten Git history.

`FS-SAU-01` is the current-source passing repository audit. R7 is therefore
`built`, never `available`, and Gate A condition five is
`met-mechanically`. This establishes only reproducible repository structure
and watched-failing mutations over the declared criteria. It establishes no
independent-human warrant, reader response, external truth, operation,
delivery, feasibility, liveness, calibration, timeless completeness, or
authentication of its own trust root.

External `review_commissions`, `proposals`, and `review_events` remain
optional append-only evidence interfaces. If used, their roster, conflict,
chronology, blind-control, triage, check, intake, custody, and disposition rules
still apply. Empty optional populations carry no deferral and block no project
gate. Darshu, Dhanush, and the custodian remain a historical optional-review
designation only.

The closure schema binds the exact Gate A claim, immutable candidate Git id,
current source version and semantic scope digest, FS-ENV-01, the qualifying
repository audit and execution cutoff, checker-derived assurance and residual
sets, exact per-row claim limitations, and the required verifier command chain
and transcript digest. The closure commit may differ from its candidate only in
closure and acceptance metadata. The checker derives `passed` from a valid
record; no author ratification or other human act is required.

## Pending obligations that outlived the tracker — 2026-09-16

Two pieces of live work were in the root `TODO.md` when it was retired. Neither
is a `- [ ]` item and neither is closed, so they are recorded here rather than
lost with the file.

## Legacy harvest — before `book.md` and `manifesto.md` are deleted

- **Delete both files, in one commit, with the harvest manifest in the body.** The
  harvest gate is fully discharged as of 2026-08-03: the 55 references
  (`registry/claims.json`); the five bright lines (swept; result under Standing
  facts); the poem (stanza 4 and the author's translation are `book-1/epigraph.md`,
  the full two-stanza text consciously kept in git history and recorded so in the
  manifest); the nine historical cases (Part V, re-pointed as failure-mode evidence);
  the domestic vignette register (Part V's kitchen); and the privacy argument (Part
  V's capture joint). What remains is the deletion commit itself, and its timing is
  the author's: CLAUDE.md ties deletion to both new books existing, so the files
  stand until that is true or the author rules sooner. The commit message is the
  record of what was taken and what was consciously dropped.

---

## Data

The registry (`registry/claims.json`, CC0), its staleness gate and the first fetcher
exist and run inside `verify.sh`; see `registry/README.md`. What remains:

- **Part V's figures stay hand-written — ruled 2026-09-15.** The option the old
  bullet offered is taken: no value-injection or rendering step is built, because
  inline registry ids would put machinery into an exempt element whose point is
  that it reads as argument, and a handful of figures does not justify a renderer
  with no other consumer. Traceability is a name binding instead:
  `claim_discipline_tests::every_part_v_figure_rests_on_a_registry_entry` ties
  each historical case Part V argues from to its registry entry and to a phrase
  that must still be in the prose, so neither side can drift alone. More fetchers
  (WHO GHO, OWID, FAOSTAT…) still land as entries need them. Build the rendering
  step only when a consumer appears.

- **Re-cite the ported registry entries against published versions — done for
  every entry Book 1 argues from (2026-09-19), open for the rest.** The port
  (`dd25b49`) honestly stamped `retrieved: 2026-07` — book.md's own last
  verification — on the legacy entries without re-verifying them. Item 09 of the
  revision backlog swept the entries Part V actually uses and left the remainder
  alone: what is still on the 2026-07 stamp is book-2 or legacy material and no
  sentence in Book 1 rests on it. Work through the rest as Book 2 needs them,
  checking each against its source's current published version (the Muralidharan
  REStat move is the model — a working paper that became a journal article) and
  updating the entry and its `retrieved` date. The Kenya UBI entry carries its own
  warning: it must not reach Part V as a working paper.

  **Two lessons from the swept half, because both were mis-citations that read as
  correct.** A registry id is not a source — Part V's Tanzania figure was bound to
  the book about Chile, and a comment in the test blessed the collision as
  deliberate rather than asking why. And a *court* is not a *judgment*: Auroville's
  governance holding and the master-plan holding were handed down by the same court
  on the same day in different cases, and the entry named the wrong one. Check that
  the locator decides the question the prose asks, not merely that it exists.

- **Add Bregman's 15-hour workweek figure to the registry** when Part V or book-2 first
  cites his proposal — the one claim the research-brief corrections found no error in but
  no registry entry for either.

- **V-Dem description and inference — corrected 2026-09-19, revision item 10.**
  `registry/fetch/vdem_happiness.py --from-snapshot
  registry/data/vdem-happiness-2026-08-03.csv` reproduces the bundled inputs
  without a fetch. Raw r = 0.5126 and income-adjusted partial r = 0.1967 are
  descriptive associations, not evidence that material provision causes most
  wellbeing differences or that the vote deserves less protection. The merge
  takes the latest observation **per series**, not a common year; three GDP
  observations are from 2024 while the other year labels are 2025. Between-
  category differences compare different countries, not transitions, and
  cannot support an anti-gradualist claim. The absolute-residual regression
  concerns both tails of national averages, not a lower bound on individual
  wellbeing. Both archived same-sample index estimates are negative; one
  crossing a significance threshold is no test of a difference between them.
  Measurement sensitivity limits robustness and calls for investigation; it
  neither refutes the causal possibility nor makes the question meaningless.
  The prior causal and threshold-based interpretations are expressly
  superseded in `book-1/source/vdem-rederivation.md`, with the historical
  calculations retained there. Part V, the registry and the script state the
  same limits. Methodological sources are Cinelli/Forney/Pearl, Gelman/Stern,
  and the ASA statement, each linked in the chapter and registry.

  The EIU column in `democracy_vs_happiness_144.csv` remains the separate
  historical licensing question described in the working record: nothing in
  this revision republishes its derived values in the CC0 registry. The
  archived EIU calculation is not claimed as a newly executed replication.

- **Publish the registry with the book, not just in the repo.** The formalism stays
  invisible, so what the reader verifies is the data — which only works if the registry is
  reachable from the page they are reading. Front matter names it and gives the URL, every
  figure in the prose resolves to a registry id, and the registry ships CC0. This is the
  thing that earns the trust and the honest substitute for showing the constitution.

---

## Standing facts and methods — not tasks, and not history

*Moved here on 2026-09-16 from the root `TODO.md`, which was retired. It is
the part of that file a command cannot teach and a rename cannot re-derive,
so it belongs with the settled decisions rather than in a work tracker. The
generator list below is the tracker's; `./generate.sh` with no argument
prints the current one, which is authoritative.*

Landed work is not recorded here; that is what git is for. What survives is the small
set of things a command cannot teach you and a rename cannot re-derive.

~~~bash
./verify.sh                 # all substantive pins and contradiction scans
./verify.sh --only <file>   # selected pin file in its declared test contexts; partial
./verify.sh --list          # execution inventory
./generate.sh state-form   # explicit authoring; installs rules and generated tests
./generate.sh obligations
./generate.sh integrity
./generate.sh statistics
./generate.sh amendment
./generate.sh mobility
./generate.sh knowledge
./generate.sh procedural-load
./generate.sh reader-coverage
./generate.sh record-power
./generate.sh resolution-receipts
./generate.sh scarcity
./generate.sh spine
~~~

The native runner uses the adjacent Nibli source checkout. No hash, receipt,
Git-history, generated-report freshness, registry, or administrative checks run
inside verification. Counterfactuals apply explicit semantic edits from
`tests/pins/suites.json`; comment-only changes do not require copying the book.
Keep source and prose review separate from logical verification.

Check the exit status: 1 means a pin mismatch or contradiction, 2 means a
harness/incomplete-scan error, and 3 means a known-defect pin stopped reproducing.
A focused pass is not a whole-book pass. A clean scan covers the encoded model,
not every sentence of prose or the truth of outside evidence.

**Two facts about the floor that no command teaches.**

- **A floor line is a compile-time prohibition, not a declaration**, and since Article 1b
  it covers the duty as well as the nine rights. `entitled(every person, event { P() })`
  compiles to a rule with `person` in the body, so `P` sits downstream of `prisoner`; any
  later rule taking `~P` into that cone is an unstratifiable negative cycle and is
  refused. The floor is protected **because** it is reachable — at stratum 0 there would
  be no cycle to close and no protection at all. Where it stops is pinned in
  `04-what-you-are-owed.pins.nibli`: `~P -> false`, `~P -> lose(Points, ·)` and positive
  compulsion `prisoner -> P` all still load — each under `:accept-scoped`, so the control
  proves loadability without leaving the forbidden shape resident. It blocks punishment for
  ABSENCE, never manufacture, and it reaches `prisoner` only. Upstream the asymmetry is
  pinned by the `rights_floor_*` tests in `nibli-engine/tests/integration.rs` together with
  their negative control `punishment_rule_alone_is_stratifiable` — **cite them by test
  name, never by line.** That citation has already rotted once and a line range is exactly
  what a rebase in another repo breaks silently.
- **The widening hazard is rule-head position** — not place index, not the predicate.
  `every`/`all` forms widen the protected set; ground facts and `some` are inert. It
  cannot be banned, because the widening *is* the firewall, so the guarantee is the
  complement pins rather than a compile-time rule.

The graph counts live in exactly one generated place, `3-spine.md`'s stratification
block. `4-strata.py` disagrees with it and is blind to the floor by construction.

**Four disciplines, each learned by being burned.**

- **Re-derive a site list by census before executing any rename.** A list written in this
  file is a snapshot and every commit since is an invalidation. The v0.6 rename list
  missed one site outright, omitted two from its leave-alone list so a mechanical pass
  would have renamed them, and predated four occurrences a later pass introduced. Line
  numbers in it had rotted by 38.
- **Citation remaps must cover every file a commit touched**, not just the one being
  edited — a careful remap still rotted three citations because it was scoped to one
  file while another was edited in the same pass. Content-match against
  `git show HEAD~1:<path>`:
  ```
  python3 - <<'PY'
  import re, subprocess
  F='book-1/source/3-spine.md'
  old=subprocess.run(['git','show',f'HEAD~1:{F}'],capture_output=True,text=True).stdout.split('\n')
  new=open(F).read().split('\n'); todo=open('TODO.md').read()
  for m in re.finditer(re.escape(F.split('/')[-1])+r':(\d{1,4})', todo):
      a=int(m.group(1))
      if a>len(old) or not old[a-1].strip(): continue
      hits=[i+1 for i,l in enumerate(new) if l==old[a-1]]
      if a not in hits: print(m.group(0), '->', hits or 'GONE', '|', old[a-1][:50])
  PY
  ```
  Bare `:NNN` citations inheriting a filename from earlier in the sentence are **not**
  caught by this and still need reading by eye.
- **A rule that gets stricter can make an existing pin vacuous without flipping it**, and
  nothing in the harness can see that happen. When v0.7 required two bodies, a pin that
  had tested the epoch-carry guard began failing on body-difference *first* — still
  green, testing nothing. Check what a pin proves after tightening the rule it sits under.
- **Check whether a quantifier has anything to range over before blaming the quantifier.**
  "Different bodies" was parked as an engine limitation when the real problem was that
  `permits/2` had exactly one audit-pen issuer, so the quantifier had nothing to range
  over.

- **A `fit/2` pin for any placement other than Homestay is a vacuous green.** `fit`
  has one producing rule and only ever carries `Homestay`, so `? fit(Ruk, HighSec).
  => FALSE` passes forever regardless of the design — kind three of the three FALSEs.

- **The rule that decides whether expansion is cheap — re-verified 2026-08-01 against the
engine-driven generator.** *Ground facts over predicates that already occur in the
constitution are structurally free. Anything that introduces a predicate name, or a rule
head, is not.* Since `5-spine-gen.py` takes its strata from `nibli-pin --strata` rather than
from a regex, "free" means the engine reports the same graph: appending `person(Nova).
work(Nova, Census). clear(Nova).` to a copy of the constitution leaves `5-spine-gen.py
--check` reporting the spine current — predicate count, derived count, rule count, strata,
the floor list, the evidence list and therefore chapter order all unmoved. A body conjunct
is free too; the rule count counts arrows, not literals.

**A new predicate name costs more than a number now, and in one case costs nothing at all.**
Article 0a closed the record, so an unadmitted name does not load — `studies(Cira, Hano).`
is refused with *"`studies` is not admitted vocabulary"* until `admits("studies")` is written
above it, which is the visible, reviewable edit the closure exists to force. Admit it and
write it **only as a ground fact** and the evidence figure does not move at all: measured,
`nibli-pin --strata` never reports a predicate that appears in no rule, so the generated
block comes back byte-identical and `verify.sh`'s evidence gate sees nothing. The cost
lands when the name enters a **rule** — measured live when `put` joined (evidence 23 → 24,
the gate moving in the same commit). A **new rule** may also add a stratum, which would
add a chapter, which the computed order forbids.

**Structural freedom is not verdict freedom, and this is what will actually bite.**
Article 4's multi-sig quantifies over two auditor variables, so a new person naming
*existing* constants can complete a rule no existing pair could satisfy: four facts
(`person(Ann). choose(Electorate, Ann). judge(Ann, Tyr). capture(Ann, Tyr).`) flip
`false(Tyr)` FALSE→TRUE and destroy chapter 5's headline case — re-executed 2026-08-01,
still true. **Every argument position in every new fact must be a new constant**, except the
four institution constants — and even those need care, since `judge(Review, ·)` is the
deceit adjudication and `broken(Court).` is a universal amnesty. The rule is a heuristic;
`verify.sh` is the proof.

- **The five legacy bright lines were swept against the enacted rules; only BL1 ported.**
  **BL2** ("no negative scoring of persons") stood refuted by the constitution until the
  clawback ruling (2026-08-02): the student rule that docked Cira for a teacher's fraud
  is deleted, `lose(Points, Cira)` no longer derives, and BL2 stands **narrowed** —
  "no subtraction except by due process for one's own adjudicated fraud" — which the
  surviving wrongdoer rule satisfies.
  **BL3** ("merit never weights votes") survives vacuously: there is no arithmetic
  anywhere in the enacted lines, and the `floor_vector_tests` development guards
  keep it that way — no numeric literal, no aggregating relation — so weighting
  cannot be written. **BL4** and **BL5** are pod-and-tech-stack material and
  belong to book-2. **BL1** ported in narrowed form and is in chapter 1's closing
  section: the floor is unconditional *above* `person($x)`, and `person` is a roster of
  written facts with two producing rules, so personhood **is** an enrolment. Do not
  restate the unnarrowed BL1 in book-1; it would be false the way BL2 is false in
  `book.md`.

- **A protected-name register does not inspect or prevent a source edit.**
  The 2026-09-19 amendment supersession removes the parallel Article 9 label
  rules. Candidate-specific amendment status belongs to the exact-change
  interface, with independent effect review, compatibility, publication and
  effective selection. Retained source-mutation tests still show that a manual
  food-entitlement deletion changes consequences, a concealed grammar change
  can remove that entitlement while leaving its structural firewall intact,
  and an `admits("rich")` edit widens the writable record even when the
  candidate contains `permanent(Art_Evidence)`. These explicit counterfactual
  edits are not authorised transitions. Article 0a makes widening visible in
  source; neither it nor a passed check authenticates or physically prevents
  an edit outside the constitutional process.

- **`--allow-shell` stays opt-in, and do not ask upstream to make it unconditional.**
  nibli's pin language is closed by design — nothing under their `pins/` may reach outside
  the repo, and their own gate never passes the flag. We control our own invocation, so the
  gate costs us one flag in `verify.sh` and protects a guarantee that is theirs to keep.

- **An extra argument on a derived relation costs about 22x, and the cost lands in one file.**
  Measured 2026-08-01 on the release engine: rewriting all three `reward` heads from arity 1 to
  arity 2 takes `rights-floor.pins.nibli` from **15.07 s to 337.50 s**. A single probe is
  unaffected — it answers in about a tenth of a second either way — so the cost is not in the
  query, it is in re-saturating per pin, which is nibli's own open item *"Materialisation:
  incremental re-saturation (C3)"*. Two older figures for this are dead and should not be
  quoted: a claimed non-termination past fifteen minutes never reproduced, and a 38.9 s-against-
  2.1 s pair predates the `event { }` projection. This is the answer to "how expensive is one
  more argument here", which is the question anybody proposing one will ask first. It is not an
  argument against a second place on `reward`; that is refused on other grounds, and they are
  in `CLAUDE.md`.

- **"The Furnished Prison" — a rejected title that is a good part title.** Scored highest
  of the twenty title candidates on pick-up and lowest on legibility, so it lost the cover
  and is wasted sitting in git. It is the sentence that closes chapter 13's delivery-gap
  passage (`29-the-one-thing-taken.md`) — *"A society whose only working provision runs
  through its prisons has not built a floor; it has built a prison that happens to be
  furnished."* Primary candidate since the reach ruling
  (2026-08-02): the launch-essay headline; the Part-title and back-cover uses stay
  listed behind it. The title work is done; this is the one asset from it
  that outlived the decision.

## Licensing

The repo is deliberately **mixed-licence** — see `LICENSING.md` before adding files. In short: new prose is CC-BY-4.0, code is MIT OR Apache-2.0, the data registry is CC0, and everything committed before that decision (including `book.md` and `manifesto.md`) remains irrevocably CC0 under the root `LICENSE`.

## Files

- `book.md` — the entire book in a single Markdown file (~2970 lines).
- `manifesto.md` — a companion manifesto, structurally independent of the book.
- `tmp.txt` — the author's scratch notes/instructions for the section currently being drafted; absent when nothing is in flight. Read it for context on what's in progress; don't treat it as book content.

- `TODO.md` — the ordered revision backlog created 2026-09-24 from the outside
  [revision plan](new-reviwes/revision-plan-9.5.md) and its prose lint. Its items
  34–72 continue the numbering of the completed items 01–33, items 67–72 being
  the defects item 45's stress tests confirmed, and the author's
  rulings on its reserved questions are recorded in *The revision rulings D1–D9*
  above.
  Its predecessor, the backlog requested on 2026-09-18 after the manuscript
  review, completed items 01–20 on 2026-09-20 and items 21–33 on 2026-09-21; the
  final assessment is in `reviews/2026-09-21-final-manuscript-review.md`.
  Completion did not declare Book 1 released or activate Book 2. The tracker
  before that was **retired 2026-09-18, its work complete** (git retains it):
  the ordered tracker for the 2026-09-16 rebuild of Book 1, placing the child
  with nobody at the heart, engines before breaks, every family rendered, both
  books named, and the former `new-book-plans/` merged under `book-1/`. What it
  produced is recorded where it belongs rather than in a work list — the seven
  rulings and their supersession notes in *The rebuild of Book 1* above, the
  design in the two controlling decision records under
  `book-1/appendix/decisions/`, the chapter table in `book-1/contents.json`, and
  the standing measurements under R3 and R6. The root tracker it replaced was
  retired the same way on 2026-09-16.
- `book-2/TODO.md` — Book 2's own tracker, collection-only until Book 1 — First
  Edition actually ships at Gate C. Collect there; rule in this file.
- `book-1/appendix/` — the planning record carried with the book (R4/R5 of *The
  rebuild of Book 1*): `decisions/`, `contracts/`, `briefs/`, `maps/`. All
  forty-seven files arrived with the 2026-09-17 move; the two 2026-09-16
  controlling records sit among them. Non-derived, outside the reading order and the
  length measurement, never the sole support for a chapter's claim.
- `book-1/contents.json` — the chapter manifest: front matter, parts, every
  chapter with its number, file, title, role (derived/exempt), group
  (engine/break) and status (landed/planned), and — once the tree follows it —
  the ordering rule. `src/authoring/contents.rs` reads it; the digit gate, the
  coverage ledger, the receipts and the spine's generated reading-order block
  all take the chapter list from it, and `reference_integrity_tests` prove it
  matches the directory (prefix == position, pins paired, one live case per
  derived chapter), that the opening note's contents follow it, that every
  relative link resolves, and that every reviewed `path::needle` reference
  resolves exactly once against a dated baseline.
- `tools/prose_lint.py` — the prose measurement (item 34): negation, terms of
  art, case names, disclaimers, banned terms and harness names per ordered
  input, held to the plan's thresholds or to `tools/prose_lint_baseline.json`,
  whichever is looser. `--check`, `--ratchet`, `--admit`. A development check,
  not a verification gate.
- `tools/relocate.py` — `plan`, `apply`, `check`: derives a rename map from the
  manifest against the directory, git-moves the files, rewrites every reference
  (path stems longest-first, guarded basenames, chapter labels in one
  simultaneous pass, reader-coverage ids and order), and proves afterwards that
  no old reference survives and that the never-touched set is byte-identical.
  Applied maps live in `tools/maps/` and are the key between pre- and
  post-reorder chapter numbers.
- `book-1/source/` — the formal source and everything beside it, moved here by
  flat rename from the former `new-book-plans/` on 2026-09-17 (R5): the
  constitution, every `*-source.json`, the family pins, `counterfactual/`, the
  generated reports and frozen audit pairs, the reader-evidence kit and its
  pilot, the reader drafts, the scripts and the engine-measurement history. Not
  chapters, not an ordered input, outside the length measurement.
- `tests/pins/suites.json` — explicit execution inventory, shared bases,
  fixtures, and semantic counterfactual edits. Tests previously generated inside
  audit checkers now live under `tests/pins/`. Keep them substantive and readable.
- `ui/` — the Book 1 reader and live companion, published at
  https://dhilipsiva.dev/rights-nobody-has-to-earn/ by the separate website
  repository (`~/projects/dhilipsiva/dhilipsiva.dev`), which rebuilds it from
  this repository's `main` at each of its own deploys. Derived chapters never
  name it; the exempt elements, READMEs and proposal do.
- `verify.sh` — incrementally builds the native runner, then checks all pins and
  contradictions. `--only <pin-file>` is partial; `--list` shows inventory.
  There is no quick/full split, hash gate, receipt, commit gate, source revision
  binding, or automatic report generation. Do not reintroduce these workflows.
- `generate.sh` — explicit authoring for `state-form`, `obligations`, and
  `spine`. Edit `state-form-source.json` or `obligations-source.json`, then
  generate the relevant family when ready. State-form installs its constitution
  block and tests. Branch fields and substantive independence requirements retain
  their legal meaning. Unfinished source edits need not be enacted by verification.
- `book-1/source/counterfactual/` — pin expectations for semantic variants.
  The execution inventory applies deliberate statement changes to the live
  constitution in memory, not byte-copy freshness checks. No regeneration is
  required for comments or unrelated source edits.
- `book-1/source/4-strata.py` — intentionally wrong historical method exhibit;
  do not repair it.
- `registry/` — CC0 claims, source snapshots, and fetchers. Source quality and
  prose/statistical review remain editorial responsibilities, outside verification.

## book.md Structure

Six parts, each opened by a `{class: part}` attribute line (Leanpub/Markua syntax — preserve these markers exactly). Chapters are `#` headings, sections `##`, subsections `###`:

1. **The Need for Change** — the problem: physical abundance vs. financial scarcity
2. **Defining the Core** — fundamental human rights, employment redefined, merit points, and the technology chapters (The Technological Backbone, Proof of Personhood, Quantum-Secure & Privacy-Centric, Making It Simple)
3. **Implementing Fundamental Rights** — food/water/education, healthcare, shelter/mobility/communication, environmental stewardship
4. **The Roadmap to Minimum Viable Society (MVS)** — scaling from one family → village → cities/nations → planet
5. **Challenges, Collaboration, and the Path Forward** — seven chapters: Pitfalls & Skepticisms, Why This Is Not a Social Credit System, Learning from Those Who Tried, When the Pod Meets the State, Governance & Conflict Resolution, MVS in Action, A Hopeful Reckoning
6. Closing appendix — Bharati's Tamil poem with translations, then `# References & Data Sources` (book.md:2886)

## manifesto.md Structure

Two parts ("The Great Sleep", "The Great Awakening") of eight chapters each. Every chapter is an expansion of one line of Bharati's poem (தேடிச் சோறுநிதந் தின்று…), quoted as an italic Tamil epigraph directly under the chapter heading. The full poem appears in book.md's closing appendix — the two documents are linked through it. Keep the epigraph lines and their order intact when editing.

Note the manifesto's heading convention differs deliberately from the book's: it uses `# Part 1: …` (no `{class: part}` markers) and `## **Chapter N — Title**` with bold markers. Don't "normalize" it to match book.md.

## Settled Design Decisions — Do Not Contradict

**book-1 (the new book) — titled *The Rights Nobody Has to Earn*:**

- **The current formal floor is nine rights**, spelled `entitled(every person, event { P() })`, and its protection is a **compile-time prohibition**: a rule punishing someone for lacking a floor right is refused by the stratifier. The floor is protected *because* it is reachable — it sits inside the `prisoner` cone. This is the verified kernel, not the final social taxonomy under the 2026-08-03 mandate; any expansion still needs a completed map contract and formal proof. Do not restate the older claim that "nothing derives it, so nothing can retract it"; that had the mechanism backwards. **Nine items since 2026-09-24 (D8 of *The revision rulings D1–D9*, item 44):** `secure` is bodily safety, guaranteed as protection with no receipt route, and `suffice` is material security, carrying the receipt route; each has its entitlement, debt and refusal pinned.
- **The current formal duty-bearer is a thin constitutional layer** — an agent with real taxing and inter-community equalisation power, carefully limited. Mutual covenants was rejected because the constitution has no membership concept and covenants would gate the floor on one; naming-the-gap was rejected as evasion of a solved question. The book concedes coercion plainly and states its social-democratic ends outright — the novelty is the constraint mechanism, not the absence of a provider. Under the 2026-08-03 mandate, this is not the final public-branch or remedy architecture. **Tiers allocated 2026-09-24 (item 36):** the State remains the bearer of the unconditional debt, while the common tier, regions and localities carry its finance, backstop, provision and delivery duties, and a certified failure passes continuity up a tier.
- **The title is *The Rights Nobody Has to Earn*, subtitled *A design for a society worked out to the point where it catches its own failures*.** Not "utopia" — the word invites the naive-utopianism dismissal and belongs to the legacy book. **The title is chosen for legibility to a stranger, and that outranks elegance.** Two predecessors are dead and neither should be revived. *"Eight things every person is owed, and why no law can take them away"* carried two overclaims: "no law can take them away" is verified false — the refusal covers **imprisonment** and stops there, and a law voiding your credibility or docking your recognition for lacking a floor right loads fine (`04-what-you-are-owed.pins.nibli:52-57`) — and a count on a cover is the most permanent counted claim the project could make, in the one place it can never be revised chapter by chapter; the floor has already been six, then ten, then eight. *"Nothing Has to Happen First"* was accurate, survived every constraint, and failed the only test nobody had run: a stranger reads it and cannot tell what the book is about. **Test any future candidate on a reader who knows nothing, before testing it on the constraints.** Two things in the current wording are load-bearing and must not be tidied: **"nobody"**, because the universality is the thesis (`08:94`) — the same reason the dead subtitle needed "every person" rather than "you"; and **"catches its own failures"**, because "rights" is a settled noun that implies these things hold in practice and the book's second half is that they mostly do not. Drop that clause and the title overclaims. Do not restore *"and where the protection stops"* alongside "rights" — it says what "rights" already says. **Subtitle superseded 2026-09-24 (D1 of *The revision rulings D1–D9*):** it becomes *A worked design for a society, with its formal claims made executable.* The author chose it knowing this entry's warning about the dropped clause. The title and the test on a stranger stand.
- **"Standing" is reserved for universal personhood; the office sense is "public
  answerability"** — renamed book-wide 2026-08-18 (`13d0a7e`). The state-form and
  family/life-course rulings reserve the word, and the bodies specification already
  enforced that mechanically: an office-sense status field carrying it fails
  generation, and the checker's own error names the replacement. Only the prose had
  not moved, so chapter 2 taught the reserved word in the sense it is reserved
  against. **The formal layer was untouched** — nothing is called `standing`;
  `authority` carries the office sense and the `Standing*` constants feeding `person`
  carry universal personhood — so no pin query depends on the word and the suite moved
  by two comments and a filename. `book-1/02-standing.md` and its pin file are now
  `16-public-answerability.*`, and the path is hard-coded in
  `9-record-integrity-red-team.py`'s `REQUIRED_NARROWNESS_FILES`, one reviewed
  `artifact_ref`, two ledger path lists, eight opening-note links, and `3-spine.md`'s
  hand-authored chapter list, **which no generator covers**. **Three senses are held
  apart and the third had leaked back**: universal standing (personhood), public
  answerability (`authority`), and credibility (`false`). The last was migrated to
  "credibility" in v0.6 with three adjacent pins guarding it, and the sweep found it
  live again in `03:163`, in two "in good standing" idioms — one a few lines from
  "lost his credibility" about the same person — and in `3-spine.md:181`, which used
  "standing and points" for credibility and recognition against chapter 8's own pins.
  **Do not reintroduce "standing" for either the office or the credibility sense.**
  Chapter 2 keeps the ordinary word exactly once, as a mention that says why it is
  reserved; the opening note's glossary carries an entry for each sense; and the
  personhood and adjectival uses elsewhere are deliberately untouched.

- **Recognition is a bare fact and is never ranked** — not by writing a mark, not by counting entries. Decided 2026-07-30 against the proposal that verified learning mint grades for the student and outcome-conditioned perks for the teacher; all three halves are refused. **(a) Students earn nothing for being taught.** Being taught is not a contribution, the doors in `10-contribution.md:3-8` are the doors, and the constitution is **not** edited to add a student minting rule — that would commit the design to paying a child (Cira is `person` without `mature`, which is the whole of `09:43-51`). **(b) Counted degree on the reward side is refused, not merely unbuilt.** Degree needs no arithmetic here, only the same relation twice with the objects held apart — the idiom the severity rules use at `constitution.nibli:452-453` — and the cast already supplies the mirror, since Cira has two teachers. **(c) Article 3 stays unconditional**: a teacher's recognition does not depend on whether the student learned, the same shape as the auditor rule that pays for the examination and not the outcome (`10:85-87`). **State the disanalogy, never "there is no score here"** — that is verified false and its source sentence was deleted in `c0bede6`. Chapter 1 concedes a computed rating already — the paragraph opening *“Even that concedes something, and it should be said out loud rather than found later”*: severity rates an **act**, is reached through a process the accused was part of, and ends with the sentence; a grade rates a **person's capability**, with nothing adjudicated and no end. That concession is an exception the design paid for once, not a precedent. Enforced, not merely recorded — `floor_vector_tests::recognition_is_minted_never_read_and_never_counted` checks that nothing reads `reward`, that no rule joins `teaches` or `work` with itself, and that `reward` stays arity one and is still minted; all negative-controlled against the actual source. It replaces `verify.sh` sections 4, 4a and 4b, retired with the old script on 2026-09-12, and is a development test rather than a verification gate. Neither is a proof: a new name for a learning record (`studies` is in the corpus, unused) routes around both, and is caught by the evidence-count gate instead, which is the only reason the pair is sufficient. **This does not block the delivery route**: teaching that delivers `learn` is a floor actuality, not recognition. Delivering learning is not grading the learner.
- **`person` is on chapter 1's evidence list**, decided 2026-07-30 and extended
  by T1 on 2026-08-05. The list had claimed to be everything the record can hold
  while omitting the entry every floor claim hangs from. “Roster membership is
  not a claim the world makes about you” was already refuted: `person` can be
  written directly, and both recorded freedom and imprisonment can derive it.
  T1 adds a supportive continuity route: an independently witnessed standing
  status in an accepted predecessor keeps deriving personhood even when its
  required successor carry is absent, and the omission becomes a named defect.
  Do not turn those routes into a hand-maintained count.
  **`derived_only("person")` remains permanently refused** because it would
  reject the ground roster itself. The remaining first-record boundary is
  external: the transition rule cannot discover someone never entered, detect
  deletion before or inside the first attested record, authenticate its
  witnesses, or make a successor arrive. Chapter 1's prose list and
  `3-spine.md`'s generated base-predicate list therefore **deliberately
  disagree**: the prose lists what may be written, while the generated figure
  lists predicates with no producing rule. The chapter's load-bearing sentence
  remains *“the **conclusions** that matter are not writable”*, narrowed from
  “things,” because personhood matters and is writable.
- **The record is closed by name — Article 0a**, adopted 2026-07-31 (nibli `850cf96`). `admits("<rel>")` refuses a ground assertion of any unadmitted relation at assert time, and an `admits` line placed below the facts is refused as coming too late, so **widening the record is a visible, reviewable edit rather than a fact somebody types**. This is what makes chapter 1's first claim — *"Not may not. Cannot."* — true: before it, `rich(Adam).` loaded and answered TRUE, because the closure was at nibli's corpus of thousands rather than at this book's own list, and only an invented word ever failed. **It is extensional only.** Rules still derive `false`, `prisoner`, `err` and `obliged`; Article 0's `derived_only` is what closes those, and the two must never be described as one guard. The admitted roster deliberately includes `person`, while the generated base-predicate figure omits relations with a producing rule. *"What counts as evidence"* and *"what may be written"* answer different questions, and reconciling those lists would be an error rather than a tidy-up. One consequence worth stating where it bites: a ground assertion through the converse alias — `obligated_by(Warden, Ruk).` — is now refused as well, because `obliged` is not admitted (verified 2026-08-01; the refusal names `admits` and the repair). The converse mechanism survives in a **rule head**, where `err($x, Placement) -> obligated_by(Review, $x)` loads and derives the inverted `obliged(Ruk, Review)` — so the chapter-14 discriminator pair still earns its keep.
- **Earning may not alter sentence duration or severity — T3 supersession,
  2026-08-03.** The 2026-07-29 intention that a convicted person could earn a
  reduction in duration or severity is retired. T3 prohibits earned-time credit,
  severity-to-duration tables, character grading, and any personal score that
  decides a person's sentence. `reward` remains unread in the current kernel and
  can never become a condition of release, sentence reduction, or freedom.
  The older coercion diagnosis remains controlling: compelled work as the price
  of liberty is convict leasing, and nominally voluntary work is pressured when
  the alternative is longer confinement. The older stratifier tests that refuse
  `prisoner($x) & reward($x) -> free($x)` remain useful regression evidence, not
  a reason to invent a workaround.
- **Confinement houses you**, decided 2026-07-31 (v0.8). The combination *not severe / no family / no home* derived no placement at all — eligible for home confinement with no home to be confined in — and Adam and Kel stood in it. The rule is `prisoner($x) & fit($x, Homestay) & ~home($x) -> dwell($x)`, and it is **consistency, not new policy**: the design already held that the state houses whom it confines (`:460`, `:472`); this was the case those rules missed. **Do not restate it as "assume everybody has a home."** `home/1` is a fact the world reports, asserting it for everyone writes something that may be false, and closing the delivery gap by fiat is verified to *silence the isolation marker in the same edit* — the instrument that would have noticed goes quiet. **The framing that must survive after FS-CVF-015:** this closes a *placement* gap; the supplied record still contains no non-carceral shelter receipt, although the constitution now has an authorised independent recipient-side route that can derive `dwell` without custody. Chapters 11 and 13 distinguish the confinement-produced shelter conclusion from that dormant route. State the current rule rather than the retired absolute: shelter derives through the custody routes or through a matching recipient-side record independently attested by an authorised witness distinct from the source. Oversight of the duty-bearer (enablers, their checkers, a meta-study) was raised and **parked to book-2** under the former scope. **Superseded in scope 2026-08-03:** Book 1 now owns the constitutional mandate, independence, evidence, and remedy path for oversight; Book 2 retains inspection practice, staffing, and operating systems.
- **The audit feeds an obligation**, decided 2026-07-31 (v0.8), Article 8b. Chapter 14 used to argue the audit's powerlessness was **structural** — a pure observer, therefore nothing can follow. That was false and it was the comfortable kind of false: it turned a decision into a law of nature. `err($x, Placement) -> obliged(Review, $x)` and the `Isolation` twin both load and derive. **Spell it `obliged`.** The accidental route closed upstream on 2026-07-31 (nibli `e70f22f` renamed the converse alias `obligated` to `obligated_by`), so writing `obligated(...)` is now a compile error rather than a silently inverted fact — that was the realistic typo and it can no longer be made. **The mechanism is narrowed, not closed, and this repo narrowed it further by accident.** After the rename `obligated_by(Warden, Ruk)` still compiles to `obliged(Ruk, Warden)` — but re-measured 2026-08-01 against *this* constitution that ground assertion is now **refused**, because Article 0a closed the base vocabulary and `obliged` is not admitted. `admits` is extensional, so the converse survives in a **rule head**: `err($x, Placement) -> obligated_by(Review, $x)` loads and derives the inverted fact. Two things follow — the realistic forgery route is gone here, and the chapter-14 discriminator still earns its keep against a slip in a rule head. So the discriminating pin pair in the chapter-14 suite — `obliged(Review, Ruk)` TRUE **and** `obliged(Ruk, Review)` FALSE — is no longer the *only* defence, but it is still the only thing catching an argument-order slip within `obliged` itself. Keep both halves. **One constraint this puts on the method part**: nibli has a filed defect — its tracker bullet **"`obliged`-spelled every-duty renders the wrong obligated party"** — where the deontic collapse picks the event variable as duty-holder when back-translating the **base** spelling, which is ours; the converted `obligated_by` spelling binds correctly. Cited by title deliberately, not by line: this is the same file whose line numbers rotted twice inside one exchange. It cannot reach readers today because this repo runs `nibli-pin` only and never renders prose, but the method part exists to show readers the machinery, so if it ever prints a rendered sentence or a proof trace, check that party before it ships. **Two rules, not one**: the general form `err($x, $k) -> obliged(Review, $x)` loads and derives **nothing**, because a body-only variable does not bind over a derived relation on this engine — the same limitation the Article 4 note records. **Historical endpoint, superseded 2026-08-24 by FS-CVF-016:** v0.8 then stopped at an unread two-place duty. The current family preserves that compatibility conclusion, reads it through one allowlisted typed bridge, and derives exact reader, action, non-response, alternate, continuity, remedy, correction, re-audit, and recurrence duties. `verify.sh` now rejects any outside `obliged` consumer. This is still not operational teeth: no rule proves receipt, action, delivery, completed remedy, recurrence control, or institutional liveness.
- **The debt is itemised, and INVARIANT 1 was rewritten because it was broken**, both 2026-07-31 (v0.8). Article 1b now carries eight `owe(State, K, $x)` rules beside the surviving `Provision` token, which is kept because it is pinned in four other files. **Enumeration cost eight rules and no vocabulary** — the evidence list counts *predicates* and the generator never looks inside the parentheses, so a constant is free; the tracker had priced this as "eight new constants in the evidence vocabulary" and that was a category error. Verified: the firewall extends to each named debt (`~owe(State, Eats, $x) -> prisoner` is refused). **The constants are not joined to the predicates**: `Eats` and `eats` are unrelated and must never be "wired up" — the names match so the resemblance is visible, and chapter 8 turns on the gap. **INVARIANT 1 no longer says "no floor predicate in any rule body"** — that was false from v0.1, because Article 6's isolation marker reads `~meets`, and nothing checked it. It now reads: **a floor right may be read only into `err` — noticed, never acted on.** The stratifier does not enforce this; it refuses `~eats -> prisoner` as a negative cycle and accepts `~eats -> reward` or `~eats -> building` happily, so `verify.sh` guards it (negative-controlled). **Do not describe the delivery gap as something the design cannot detect** — verified, `owe(State, Eats, $x) & ~eats($x) -> err($x, Undelivered)` is accepted and derives, and the design already ships that shape for company and for none of the other seven. That asymmetry was discovered, then ruled: the markers are refused while the record holds no arrival facts — see the delivery-markers entry.
- **A control puts the base back; a premise does not**, settled 2026-08-01 with nibli
  `425567b`. `:accept` leaves its statement in the knowledge base, so every pin below one
  ran against a widened base — and that produced a real vacuous green here: with the
  complement controls loaded, `? prisoner(Adam).` passed against a constitution with
  **Article 6's conviction rule deleted outright** (re-measured 2026-08-01; silent under
  `:accept`, caught under `:accept-scoped`). **Write controls `:accept-scoped`.** The
  exceptions are allowlisted in `verify.sh` section 4d — do not re-derive the list here, it
  has grown twice; the script is the list — and the test for joining it is always the same:
  the accepted statement is a **premise the file goes on to query**, not a control. Chapter 1
  accepts a roster entry and asks what it derived; chapter 14 accepts the duty-breach rule and
  asks what it marks; chapter 5 accepts the void rule minus distinctness and asks what flips;
  chapter 9 accepts the disenfranchisement clause and asks what it failed to take. Scoping any
  of these makes its query meaningless — verified per file as each joined.
  The old workaround was "order the file so its controls come last", which is a rule
  nobody can see being broken; do not restore it, and do not delete the *history* those
  ordering comments recorded when deleting the *instruction*. **`derived_only` and
  `admits` cannot be scoped at all** — they survive the retraction by design, and asking
  is a harness error rather than a silent no-op.
  **This suite depends on `nibli-pin` giving each pin file a fresh engine, and that
  dependency is load-bearing rather than incidental.** `verify.sh` passes every file in one
  invocation; re-verified 2026-08-01 that a ground fact *and* an unscoped `:accept` in one
  file are both invisible to the next. It is the only reason chapter 8's widening — which
  was live, not latent: `lose(Points, Hano)` derived at the point the old ordering comment
  declared everything below sound — stopped at the file boundary instead of reaching the
  rest of the suite. If that isolation ever changes upstream, this suite's containment
  goes with it and every ordering assumption in every pin file has to be re-examined.

- **Provenance on `reward` is refused, and not because of the clawback fork**, decided
  2026-08-01. The proposal is to give `reward` a second place recording what the recognition
  was minted *for*, so a clawback could reach only what came from a fraud. **It keeps looking
  cheap, and that is why it needs a written refusal**: all three minting rules already bind the
  source and throw it away at the head — `$student`, `$task`, `$audited` — so the argument is
  sitting there unused and costs no new vocabulary. Refused on five grounds, and **each holds
  whichever way the Article 4 clawback fork is ruled**, so this does not reopen when that lands.
  **(a)** The recognition guard holds that nothing reads `reward`, by decision — and an arity-2
  `reward` exists only to be read; verified, the narrower rule `reward($t,$s) & false($t) ->
  lose(Points,$t)` loads and the absence guard catches it. **(b)** Section 4b forbids joining
  `teaches`/`work` with itself, and a second place puts counted degree one rule away.
  **(c)** There is nothing at the other end to narrow *from*: `lose` is a **leaf** —
  no rule body reads it, derivation is monotone, so nothing is ever actually taken from anybody,
  and the apparent clawback in the shipped cast is entirely the `~false` guards on the
  minting rules — all three doors since v0.9 — **never minting**. Do **not** restate this as "`lose` has no slot for it" — that was the first
  draft of this bullet and it is false, cheaply refuted: nibli gives `lose` (gismu `cirko`) a
  third place, `conditions`, and `lose(Points, $s, $t)` loads. **(d)** The corpus entry will not carry it — `reward` is gismu `cnemu`,
  arity 4, places `["subject","rewards","atypical","reward"]`, `Generic` tier with places never
  hand-verified; the natural `reward($teacher,$student)` puts the student in the *party rewarded*
  slot, not the deed slot. That is the ground `deserve`/`jerna` was rejected on. **(e)** Measured
  2026-08-01: all three heads at arity 2 take `rights-floor.pins.nibli` from **15.07 s to
  337.50 s**. **Chapter 6 already rules this way and needs no change** — *"The design's answer to
  that tension is not to sharpen the instrument. It is to put a hard ceiling on it"* (`06:107`),
  and `06:81-85` says whoever writes the narrower rule "will find they have written a repeal".
  **Enforced by the recognition guard's minting count, because nothing else can see it arrive.** Three TRUE pins
  (`reward(Esa/Quin/Gia)`) catch a *rewrite* of the heads, but **five** other `reward` pins go
  vacuous in that same edit — three more in chapter 10 plus `reward(Cira)` in `26-clawback` and
  `reward(Dev)` in `rights-floor` — because `reward/1` stops existing and a FALSE pin against a
  vanished relation still reads FALSE. Worse, the **additive** shape is invisible to everything:
  a fourth head at arity 2 added *beside* the three leaves `reward/1` deriving, so the absence
  guard and the whole pin suite stay green — verified. Section 4a also asserts `reward` is still
  mentioned at all, since "nothing reads `reward`" passes just as cleanly against a relation that
  has been renamed away.

- **Article 4's `~broken`/`~match(CarriedVoid)` signer checks stay, and are not dead code**, decided
  2026-08-01. No assignment can fail them today — every Review/Tribunal pen derives through
  Article 8's rules, which read both guards themselves, and `derived_only("permits")` closes the
  assertion route — but they were **live guards when v0.2 wrote them**, against sock-puppet
  credentials with a prior voiding. The raw `rotten` input is now inert by itself; the guard
  reads only an exactly witnessed void carried into the designated current record. The v0.3
  credential closure demoted the signer checks without anyone deciding to. They re-arm the
  moment any pen route omits the guards: measured, with one
  unguarded route added, `false(Tyr)` stays FALSE with them and flips TRUE without them — a
  carried-void signature counting. Both halves are **permanent fixtures**, not commit-body
  prose: `no-dead-conjuncts` (chapters 4 and 5's own pin files pass against the stripped copy —
  the subsumption) and `unguarded-pen` (the postulated future where the conjuncts are the
  deciding check). Same idiom as the rule's own `~($a = $b)`, kept for the day the assumption
  that hides it stops holding. **Do not delete them as dead code** — that proposal has now been
  made once, by the pass that produced this ruling — and note the reading consequence: chapter
  5's matched-history pins land on the *missing credential*, not on these conjuncts, and the prose
  ("that matched history blocks the credential") is correct about that.

- **The Esa passage tells the truth about Koa, and nobody seats him**, decided 2026-08-01.
  Chapter 5 claimed Koa *"examined Esa"* with *"the credential"*; the record holds a personhood
  entry and one `capture` fact — no judging, no seat, no pen — and both false clauses are now
  pinned FALSE in chapter 5's suite. The tracker's alternative repair, seating Koa on both
  bodies as the count-isolating fixture, is **refused**: chapter 2 pins `authority(Koa)` as its
  no-answerability exhibit (*"two facts in the whole constitution, no seat, no office"*) and chapter
  1's *"documented something"* rests on the same two facts, so that route falsifies two chapters
  to repair one. The count isolation lives in a **pin-file fixture** instead: Ambi, a fresh
  person seated by both bodies who judges and records — every act the rule asks of either
  signer — and cannot void alone, with the closing exhibit (the void rule minus distinctness,
  accepted; `false(Solo)` flips TRUE) proving distinctness was the only bar. That fixture is
  also the liveness proof for the `~($a = $b)` conjunct Article 4's header keeps *"for the day
  one person holds both pens."* Chapter 5's pin file joined `verify.sh` section 4d's
  allowlist for that closing exhibit — chapter 14's shape (the section's positive control
  counts the allowlisted files; it moves whenever the list does). The section headline reads: the rule does not count signatures, it counts
  signers.

- **Care walks through two existing doors, and chapter 10 says so**, decided 2026-08-02,
  answering the one point in `reviews/ai_review.md` the book had no reply to. The teaching half
  of raising a child mints through door one — a parent who teaches is a teacher, the record
  never asks the relationship, and the pair that voids a *judge* (Article 5) disqualifies no
  teacher. The tending half mints through the work door — nothing narrows what counts as work
  to the salaried or the public, and the task constant is free vocabulary. **Do not add a
  fourth door, a care predicate, or any ranking** — the review's maximal ask ("highest-status,
  most heavily subsidized") stays refused under recognition-is-never-ranked. The exhibit is a
  **pin-file caregiver pair** (chapter 10's suite; fresh names, one route each so each door is
  isolated), deliberately not a cast member: the shipped record holds no care entry, and the
  chapter's claim is calibrated to that — *"the doors are open to it"*, never *"the society
  sees it"*, with the entry-has-to-be-written limit stated in the same breath. Found while
  ruling: **neither door carries a maturity test** — a child
  who works or teaches mints today (both verified), beside a design that refused student
  minting partly as paying a child. That question was ruled the same day — see the next
  entry ("The doors carry no age test, and that is defended, not confessed").

- **The doors carry no age test, and that is defended, not confessed**, decided 2026-08-02.
  A child who works or teaches mints recognition today — verified both ways, and chapter 10's
  care coda now says so in the open, on Cira: the child the design will not grade is recognised
  the moment she contributes. **The maturity guard was measured before being refused**: with
  `mature` conjuncts on the teaching and work doors, four of the five recognised people go dark
  (`reward(Esa/Quin/Nima/Sata)` all flip), because the record barely holds `mature` and none of
  the recognised have it — so the guard's real meaning is *your work counts only once somebody
  files your adulthood*, a written entry upstream of esteem, the same shape the old earned-time
  proposal used, and it shuts the care doors for a young carer in the same edit. **Do not add a
  maturity conjunct to any door as cleanup.** The disanalogy with the grades ruling is the
  argument, stated as a pair in the coda: being taught is not a contribution, at any age;
  contributing is, at any age. The exploitation reading is answered by the design's own shape —
  recognition cannot pay, and what makes a child's work worth exploiting is the wage. Exhibits
  in chapter 10's suite: the work door isolated on Cira (her not-mature status pinned first),
  the teaching door isolated by file order on Pico, whose `reward` FALSE while tended flips TRUE
  two verdicts later when he teaches. Prose stays in the conditional voice — the shipped record
  holds no child work entry, and chapter 6's *"Cira earned nothing"* stays true.

- **The third door closes on voiding — enacted, not disclosed**, decided 2026-08-02,
  resolving the chapter-10 fork (v0.9). Article 4's examiner rule now carries
  `~false($auditor)` beside `~deceive`/`~broken`, the guard the teaching and work doors
  already had, so chapter 10's "all three doors close for the same reason" is true as
  printed. The disclosure branch — keep the rule and print that a voided person can still
  earn by examining people — was refused as a contradiction rather than a mere cost: an
  examination is nothing but its author's word, voiding is the finding that the word is
  worthless, and Vex — carried-void, therefore penless — was being paid for examinations
  the same voiding made incapable of counting toward anything. Enactment is the
  confinement ruling's shape: consistency with the design's stated intent, not new policy.
  Measured before landing: `reward(Vex)` TRUE→FALSE, `reward(Gia/Esa/Quin)` unchanged,
  every pin suite green against the guarded copy, rights-floor runtime unchanged (~15 s).
  **What it does not do**: nothing un-mints — `lose` stays a leaf and derivation stays
  monotone; the guard never-mints, chapter 6's register. **Two things it exposed, recorded
  where they bite**: the chapter's worked reasons needed correcting anyway — Lupo's door
  shuts twice over (per-pair `~deceive` for the lie itself, then the void) and Dev never
  documented anybody, so his "nothing at all" is carried by the void alone — and the third
  door still pays a person of intact credibility for a bare `judge`+`capture` pair with no
  credential and no grounds (measured post-guard: a fresh clean person mints from the two
  writes), which is the `capture`-precondition bullet's territory, not this ruling's.
  Exhibits pinned in chapter 10's suite: the Vex pair, and the closing forced-Bela probe
  (a voided person examines; the mint refuses; the clawback registers) —
  negative-controlled, exactly those two `reward` pins flip against the pre-guard
  constitution.

- **The floor-delivery markers are refused while the record holds no arrival facts**, ruled
  2026-08-02, closing the largest restored Phase-1 decision. Measured before ruling: the
  Undelivered rule loads and derives — and fires on the voided, the confined-and-housed, and
  the never-accused alike, because `eats` is asserted for nobody and `~eats` is true of every
  person alive. A marker that fires on everyone discriminates nobody; chapter 11's own alarm
  standard applies. **The asymmetry is chosen now**: the two existing markers audit the
  design's own act — Isolation is prisoner-scoped, Placement audits placement — and chapter 8
  says so in print ("watches what it does, and does not yet watch what it owes"). The
  `undelivered-marker` fixture holds the measurement permanently and is where the marker
  design starts when book-2's delivery layer generates arrival facts — the ruling expires
  with its premise. **Two economics facts recorded so cost never decides this again**: the
  general `err($x,$k) -> obliged(·,$x)` form NOW DERIVES (the per-kind cost-doubling that
  shaped the original bullet is dead; Article 8b keeps its two constant-named rules
  deliberately — correct, pinned, not worth the churn), and Article 0a charges nothing for a
  derived head. Do not build a floor-delivery marker into this constitution without arrival
  facts, and do not re-price the decision on rule-count.

- **The design does not say how long — ruled 2026-08-02; superseded in scope by
  T3 on 2026-08-03 and implemented narrowly on 2026-08-05.** Ordinary duration
  vocabulary remains outside the record (measured: `year(Term, Two).` and
  `earlier(Custody, Release).` are both refused), but the constitution now admits
  the bounded inputs needed for witnessed record replacement, order, and a
  case-bound custody review. The implemented T3 rule says whether the supplied
  fresh evaluation carries current reviewed authority. A witnessed passport
  selects one constitutional lineage, and only its collision-free terminal
  successor can supply current adverse carry or public power. Typed event and
  record paths close transitively; a direct or longer cycle taints the connected
  path and cannot support a review window. The custody gate rejoins independently
  witnessed case-subject, Court-holder, Court-judgment, injury-victim, lease,
  window, renewal, and source fields; conflicts are explicitly rejected for the
  source, window, case-subject, and lease bindings. It does not count days, establish a
  maximum measured sentence, or prove the outside service advances. Chapter 13's
  closing pins preserve that distinction. **Release stays an act, not an expiry**:
  missing, stale, disputed, or incompletely witnessed authority prevents custody
  from deriving but never manufactures `free`. Compact status tags cannot lend
  their authority to another raw tuple sharing the same identifier: consequential
  consumers rejoin the exact fields and matching witnesses, and the temporal
  harness attacks that aliasing directly. Because the engine is monotonic within
  one process, later facts do not retract already-derived authority; every
  current-effect case runs in a fresh process. Book 2 retains clocks, calendars,
  scheduling, availability, publication, recovery, and operational time
  administration. The superseded earning-shortens ruling is not a deferred
  permission: nothing measures duration, nothing reads `reward`, and no temporal
  premise may price a person. One harness nuance
  recorded in the pin file: `admits` for a *fresh* name loads
  from a probe (too-late guards only relations with prior facts), so refusal pins guard the
  shipped text and a probe may always widen on top.

- **The shield's exposure surface is unbounded, refused as composed**, ruled 2026-08-02,
  closing chapter 4's own open question. Every candidate bound dies on a prior ruling, and the
  settled entry names them so nobody re-proposes one at a time: a **time** bound is not part
  of the implemented custody-only T3 family and would require its own power-specific contract,
  rather than an individual exception; an **epoch-recency fact** is an unadjudicated
  shield-stripping write — one fact retires a target from everyone's shield surface, the
  finding-with-no-finder class chapter 1 conceded, handed to whoever writes it; **lapsing or
  re-certifying public answerability** revokes it, and Rebel's shield surviving Boss's recall (chapter 2's
  thesis, pinned three files over) is the standing refutation. Beneath all three, **the shield
  remains non-temporal**: T2 can represent a separately witnessed sequence, but no shield rule
  reads it, and within an unordered snapshot exposed-then-recalled and recalled-then-exposed
  remain indistinguishable. The
  reach-back is pinned in chapter 4's suite (Zeno: convicted, one `show` naming the recalled
  Boss, conviction blocked — negative-controlled on the single write). **Do not bound the
  surface by any route**; the growing list stays priced in chapter 4's costs section, and
  whoever proposes the bound again must say which of the three protections they trade for it —
  that sentence is in print and is the chapter's answer to the delay-gaming objection, which
  Part V's capture joint defends rather than dissolves.

- **The temporary-assessment exclusion is priced as an exile, not defended as an absence**,
  ruled 2026-08-02 — Part V, capture joint, verdict "Survives, narrowed." The design does not
  abolish capacity, risk or crisis assessments; it refuses them entry to the one record that
  reaches standing, liberty and the floor. **The firewall is the claim** — a hospital chart
  can inform care and can never void you — and **the exile is the price, stated in full**:
  assessments pushed outside live in records this design does not police, and power migrates
  toward whatever record matters. The lesser-harm defense: inside this record an assessment
  sits upstream of liberty forever; outside it, its harm is bounded by what the record
  refuses to hear. **The episode-fact route is refused for book-1, composed — do not
  re-propose an assessment entry, episode-scoped or otherwise**: it is a new admitted name
  (chapter 1's list and the twenty-four move), a finding-with-no-finder (the class chapter 1
  conceded), and it faces the inert-or-dangerous dilemma — read by nothing it is a third
  pretending instrument (refused at the markers and at duration), read by anything it is an
  assessment feeding consequences. The standing exhibit is already pinned (`dangerous(Adam).`
  refused, chapter 1's suite). Part V owns the argument; book-2 owns the one-way firewalled
  operational layer, filed there.

- **No record-person gets an inner life, and the flatness is evidence, not a defect**, ruled
  2026-08-02. Five of six reviewers asked for emotional register, an external antagonist, and
  characterisation of the named cast; the ruling holds the line the book's own thesis draws —
  the people in it are exactly the facts recorded about them, and inventing Bela's fear or
  Cira's confusion would fabricate the kind of entry the record refuses to hold. **Do not warm
  the cast in any pass, any part, ever**, and do not read the flatness as awaiting repair:
  Part V states the restraint once as chosen and cites the five-of-six as evidence that a
  reader who felt it has felt the design. Part V's approved register channels, each with its
  guard: the **author's first person** (historically supplied at the Voice ruling; current
  supply follows the 2026-08-20 exact-version author-approval protocol); one **second-person
  domestic vignette** — a household through food, care, housing and crisis,
  generic "you", never a cast name; the **hostile reviewer corpus as the antagonist**, quoted
  from `reviews/` and answered at the joints; and the **nine historical cases as the
  documented feeling**. Nothing else — no invented antagonist, no composite citizens, no
  dramatised cast scenes. **Amended 2026-08-08 by the narrative-register ruling**, which
  keeps the list closed and disposes of all four channels, but widens the passage channel
  from Part V to the opening note *and* Part V, adds the form/assertion rule and the
  restatement-only register, and dates the existing passage non-conforming on arrival,
  trace and proximity. The rationale above is unchanged and is not superseded; see
  `book-1/appendix/decisions/narrative-register-decision.md`.

  **Amended 2026-09-19, revision item 11:** the register and ban on invented
  inner lives stand. The reviewer corpus is no longer required as Part V's
  antagonist; its AI-generated reviews are feedback, not human testimony or
  independent validation. The current chapter states objections directly and
  makes no five-of-six endorsement or reader-experience claim. The controlling
  narrative-register decision retains the exact approved hypothetical kitchen
  paragraph and supersedes `OL-15-v1` for Part V. The nine sourced historical
  cases remain, with their evidence narrowed to what the sources support.

- **The thesis is ruled restated-structural, and the infant joins as the paired
  second stress case with framing primacy**, author-ratified 2026-08-17. The
  thesis lives only in the exempt elements and states what the design does —
  the floor's absence can never be a lawful ground for sanction, and the
  interfaces demand nothing of the recipient — never what is axiologically
  owed: "owes" does not survive verbatim, `dignity` may not become a
  load-bearing thesis noun (undefined term; the title ruling's failure mode),
  and `safety` may not be a thesis word (it names an envelope criterion that is
  Book 2's to calibrate). The methodological half splits per the one-posture
  rule: the general claim is Reasoned at best and never citable as Derived,
  beside the named executable instance (the universal-standing family).
  "Designing for the most vulnerable" reads only as unconditionality plus
  demand-nothing interfaces — never targeting, never a vulnerability score or
  status entry. The caregiverless infant is the **second stress case paired
  with the prisoner** — the two fail in opposite directions (the prisoner is
  the person public power acted upon; the infant is the person it has not) —
  and holds **framing primacy in the exempt elements where the landed rulings
  permit**: the stress-test framing may lead with the infant. Substance
  primacy is not allocatable — chapter order is computed, portfolio density
  follows the landed rules, and "hardest stress test" is a measured source
  property (today the prisoner's) to be re-measured now that the
  delivery/receipt families have landed. **The through-line stays refused** (agency clause,
  anti-monoculture guards, register ruling — an infant occupies only the
  *receives* and *is acted upon* postures, and it is the register's
  highest-risk subject because the reader supplies the affect unasked).
  The delivery/receipt precondition is now satisfied. The remaining sequence is
  the portfolio rebalance, then the reader ledger, then exact-version
  author-approved prose — the ruling settles the frame now precisely so it
  cannot be settled by accident later. A session may propose the thesis sentence under the 2026-08-20 protocol;
  approval of that exact displayed version makes it canonical. The controlling record is
  `book-1/appendix/decisions/thesis-framing-and-second-stress-case-decision.md`.
  **Superseded in scope 2026-09-16 by R1 of *The rebuild of Book 1***: the
  child is the heart — opening argument, first chapter, recurring test
  section — and chapter order is editorial; the §2 thesis-form constraints,
  the register, the no-score rule and the Reasoned posture stand.

- **The voice boundary is the derivation boundary**, ruled 2026-08-02. The plain, direct,
  reader-facing register of the derived chapters — fourteen at the ruling, manifest-derived
  since 2026-09-16 — is ratified as their voice — it is a
  voice, and the texture ruling cites its restraint as the thesis performed. The author's
  first person enters at exactly the three elements the derivation gate already exempts: the
  opening note, Part V, and the method part. **The seam is audible and deliberate** — a reader
  can hear the crossing from what the machine derived to where the author argues — so **do not
  warm the derived chapters and do not flatten the exempt three**; either direction erases a
  boundary that is doing epistemic work. Current supply follows the 2026-08-20 exact-version
  prose-supply protocol stated under *The narrative register* above.
  **All three exempt elements were session-drafted by the author's explicit instruction,
  each a recorded exception (2026-08-03).** The opening note: "You draft it yourself",
  reviewed in `tmp.txt`, adopted with no changes ("I have no changes to suggest"). Part V:
  "Same for Part V — draft it yourself, commit and push" — adversarially verified against
  the reviews corpus, the registry and the chapters before landing (three checkers; two
  misattributions and two overclaims caught and fixed pre-commit). The method part: ruled
  session-drafted in the planning session for its landing (AskUserQuestion, the same
  explicit-override basis), adversarially verified the same way. This line is the record of
  all three. Those are historical precedents, not exceptions required by the
  current protocol. Future first-person writing, the launch essay, and an
  epigraph translation follow the 2026-08-20 exact-version approval lifecycle. The
  full re-weave the tracker once priced (every chapter re-touched, dearer each week) is dead
  by this ruling, not deferred. **The 2026-08-08 narrative-register ruling
  originally reserved ordinary-life supply to the author; that supply
  disposition is superseded.** Its location rule remains: `method.md` is
  excluded from the ordinary-life passage channel despite being one of the three
  exempt elements, because its scope is sealed, not because it is less exempt.
  **Superseded in part 2026-09-24, not yet implemented (D2 of *The revision
  rulings D1–D9*):** each derived chapter will close with one labelled
  first-person argument section, so the seam moves inside the chapter rather
  than disappearing. Derived sections stay flat and derived.

- **The method part's scope is sealed — `book-1/method.md`, landed 2026-08-03.** Five
  decisions, each load-bearing: **(a) the filename is unnumbered and must never be
  renumbered** — verify.sh's jargon and counted-claims gates glob `book-1/[0-9]*.md`, and
  the file quotes the stratifier error verbatim, so a numbered name fails the build
  instantly; the epigraph is the precedent and `book-1/README.md` records the convention.
  **(b) No pin file** — non-derived elements are unpinned (Part V's precedent), and a new
  `*.pins.nibli` would enter the suite and break the `:expect-pins` reconciliation. **(c) No
  machine-rendered English and no proof traces, ever** — rules as written, error messages as
  printed, pin verdicts only; this is stated in the part's own reading contract and is what
  keeps nibli's still-open renderer defect (its tracker bullet **"`obliged`-spelled
  every-duty renders the wrong obligated party"**, cited by title, never by line) dormant.
  Adding a rendered example "for friendliness" arms it. **(d) Excerpts + pointer, never the
  full KR inline** — the part owns the resolution in print (the repository is the appendix);
  reproducing the constitution or the spine's generated table creates the documented
  staleness failure mode. **(e) The compute backend is not discussed** — if any future edit
  adds it, cite nibli README's "Trust boundary" callout by heading. The part's quoted error
  messages were reproduced against the built engine on landing day; the stress-surface
  paragraph's upstream inventory names artifact classes, never counts, because upstream
  counts rot. Book 1 names Book 2 at its seams — Part V and this file's closing lines —
  and by title since the 2026-09-16 naming ruling (R2); derived chapters never name
  either book. The former "exactly once" constraint is retired.

  **Current presentation, 2026-09-19, revision item 12:** these scope limits
  stand. The method remains optional, unnumbered and without a companion pin
  file; its examples point to existing executable cases. It quotes source
  statements and actual pin expectations, including refusal patterns, with
  no machine-rendered explanation, proof trace, full constitution or ordinary-
  life vignette. The dependency order is explained separately from editorial
  chapter order. The older verification-gate descriptions above are historical
  rationale, not reinstated gates. No compute-backend discussion is added.

- **The reach strategy: public construction from a home of its own**, ruled
  2026-08-02, qualified by E2 + P1 + D2 on 2026-08-04, and bounded to Gate C
  on 2026-08-07. The dedicated domain,
  platform syndication, public red team, and assembled-book capstone survive.
  The original instruction to serialize the pass-complete pre-expansion manuscript
  does not: E2 leaves that manuscript as public source and git history without a
  promoted edition. P1 replaces that immediate route with immutable, tagged
  expansion snapshots after Gate B and their snapshot-specific gates; they are
  previews, not editions or final spine-order serialization. Constitution and
  spine freeze may create a candidate, but no public candidate becomes an
  edition early. Only cumulative Gate C completion publishes Book 1 — First
  Edition, its assembled digital capstone, and D2's first Book 1 POD under one
  provenance contract. Building in public still performs the thesis: repo
  history is the method's proof, previews recruit the outside red team, and
  defect pins make known flaws declared features. Four companions remain: the
  launch essay (*The Furnished Prison* the standing headline candidate; inside
  the voice boundary — session draft permitted, exact author approval required),
  the method paper (formal-methods-for-law
  genre; cites the book, never the reverse dependency), run-it-yourself as a
  front-page claim once the exact nibli dependency is reproducible (`verify.sh`
  is the core artifact, not yet a one-repo reproduction), and the D2 physical
  edition whose lever is quality, not exclusivity. **The dilution guard from the
  adoption reviews rides along: reach adapts the packaging, never the design** —
  an audience that needs a different design is book-2's reader, not book-1's.

- **The student clawback is deleted, and `lose` stays a leaf**, ruled 2026-08-02 — the
  Article 4 clawback fork, decided as Branch A. The rule `teaches($t,$s) & false($t) ->
  lose(Points, $s)` recorded a loss against a student for a teacher's fraud — negative
  scoring of a person who did nothing — and is deleted outright; the wrongdoer rule
  `false($f) -> lose(Points, $f)` **stays**. Deletion, not narrowing, because the middle
  options were closed before the fork was ruled: students never mint (the grades ruling),
  so "claw back only recognition derived from the fraudulent teaching" had an empty
  target — narrowing was always repeal — and the wrongdoer-side narrowing needs
  provenance on `reward`, which is refused. **Legacy bright line 2 narrows with the
  ruling**: book-1's form is *"no subtraction except by due process for one's own
  adjudicated fraud"*, which the surviving rule satisfies; do not restate the unnarrowed
  "no negative scoring of persons" as book-1 doctrine. **The `lose`-reader half is ruled
  closed with it**: the leaf stays — the loss is recorded and read by nothing, chapter
  14's determination-then-stop register — and the measured-to-load consumer
  `all $x: lose(Points, $x) -> err($x, Recognition).` is **refused**, not merely
  unbuilt; do not re-propose it as a small rule. The Cira pin flipped its declared
  `:defect` to a plain FALSE, chapter 6's middle section tells the deletion as history
  (the repaired-defect register of the alarm and third-door repairs), and the exhibits
  that survive are `reward(Cira)` FALSE and `person(Fin)` FALSE — the student who earned
  nothing and the student who was never on the roster.

- **The Articles 6/7 polarity asymmetry is defended, not repaired**, ruled 2026-08-02 —
  the polarity contradiction, resolved as a chosen asymmetry. The shield stays
  fail-open (absence of a deceit finding protects; chapter 4's priced choice) and the
  conviction rule stays fail-closed (absence of granted relief convicts), because **the
  two absences differ in kind**: the shield's absent finding is an *accusation not yet
  adjudicated* — nobody is treated as a liar on no one's finding — while relief's
  absent finding is a *remedy not yet performed*, and a remedy present by default is
  not a remedy: read `~permits(Appeals, ·)` fail-open and no conviction ever derives,
  handing whoever declines to adjudicate a veto over every conviction. Both defaults
  give the benefit of absence to what the guarded machinery already concluded, against
  the same enemy — the captured or idle adjudicator. The defense is in print three
  ways: Article 6's head note, the severity note's re-dating (the "unexplained…still
  unresolved" sentence is gone the honest way), and chapter 3's default paragraph in
  "The other pen", pinned by the either-alone pair — the bare clear flag (Adam) and
  the bare appeals judgment (Voss) each leave the conviction standing while Nia's
  both-together block lifts it. The cost is stated, not hidden: a person whose case
  nobody takes up stays held, the mirror of chapter 4's window, weighed the same way.
  **The alternatives are refused as composed — do not re-propose either**:
  review-standing derived from `prisoner` and read under negation by the conviction
  rule is the measured stratifier cycle (Article 7's own landmine), and "pending"
  framing collides with chapter 5's "Not provisionally, not pending anything"
  because the relief pair itself remains case-unbound and unordered; an
  affirmative exhaustion fact is a new admitted name in the
  finding-with-no-finder class whose *withheld* entry blocks every conviction — one
  word upstream of liberty, pointed the other way.

- **The vocabulary batch — five rulings, one sitting**, 2026-08-02, landed as four
  constitution commits (`b51781f` kinship, `6c68306` forgive, `fc51b75` cite, `4231547`
  hears; evidence vocabulary 24 → 29, admits 25 → 30). Each ruling names its refused
  wider form so nothing trickles back. **(1) Kinship**: `married` (speni) and `sibling`
  (tunba) joined Article 4's independence check in both directions, beside `~parent` —
  dead guards, since the cast records no marriage or siblinghood; the gendered pair
  (`brother`/`sister`) was skipped so the record carries no axis it does not need, and
  **Article 5's judge-your-child rule was deliberately not extended**. Chapter 5 keeps
  the residue: closeness the record has no words for still co-signs. **(2) Expungement,
  mark only**: `forgive(Appeals, $x)` beside `judge(Appeals, $x)` derives `clean($x)`
  (a new `derived_only` — hold it apart from the asserted `clear`), and the effective
  witnessed carry became `match(CarriedVoid) & ~clean -> false`. The raw
  `rotten` report remains an input but is inert alone. Two writes, never one —
  the clear-guard shape.
  Word restored, clawback stops, earning doors reopen, and the measured retroactive consequence
  is told in print: a forgiven examiner's recorded past examinations start paying.
  **The pen-return form (four sites) was refused**: credential rules and the multi-sig's
  signer checks still read the mark itself, which stays on the record — forgiven, not
  erased; the way back for the person and the way back to power are different roads.
  **(3) Grounds, the paid door only**: `cite($auditor, $ground, $audited)` (sitna) is a
  condition on the examiner-reward rule — the bare pair that minted for a fresh clean
  person now earns nothing (the Yano exhibit, ch10) — with the limit stated where the
  guard is: any word fills the slot, so the record buys an author for the why, not its
  truth. **The void's own grounds stay deliberately unasked** — Article 4's multi-sig is
  untouched and chapter 5's who-never-why stays the open question, now narrowed to
  where it matters most. **(4) Delivery**: `teaches($t,$s) & hears($s,$t) -> learn($s)`
  — the first route to a floor actuality that does not run through a prison. `hears`
  (tirna) is a two-sided arrival record rating nobody; the entitlement stays
  unconditional; the route ships **dormant** (no cast arrival — the placement-marker
  posture, pinned live in ch8), and **cast exercise was declined**: Cira's hearing is a
  probe, not a shipped fact, so chapter 8's no-litany and chapter 13's sting stand.
  The grades ruling is untouched in both directions — being taught still mints nothing,
  and the rule joins `teaches` with `hears`, never with itself. **(5) Governance
  vocabulary — superseded in scope 2026-08-03**: the former ruling deferred community,
  transfer, tax, and term-of-office vocabulary to Book 2 because the present constitution
  cannot state portability, justice, or a complete public architecture. The ratified
  mandate now permits the constitutional vocabulary and interfaces those domains require.
  Add them as completed, mapped constitutional rule families—not one predicate at a
  time—and retain operational finance, administration, and infrastructure in Book 2.

- **The learning route was book-1's whole delivery story**, ruled 2026-08-02, and is
  **superseded in scope 2026-08-03**. `teaches` + `hears` -> `learn` remains the
  valid demonstration: two-sided arrival evidence upstream of a floor actuality,
  without ranking the learner. The expanded mandate now permits generic constitutional
  delivery, receipt, breach, interim-continuity, remedy, and review interfaces for every
  floor. A provider's self-authored assertion cannot alone count as receipt or delivery,
  and no floor may become conditional. Meal logistics, care workflow, facilities,
  staffing, procurement, capacity, and all other operating machinery remain Book 2.
  **Its shape was ruled on 2026-08-03:** the legacy route is not a generic delivery
  template—`hears` is audio-literal and uncontrolled, while `learn` is a personal
  outcome rather than neutral receipt. Future delivery families must use
  accessibility-neutral, recipient-side access/receipt evidence with a named
  authorised writer and challenge route; they may secure conditions but may not
  certify learning, health, belief, or another compelled personal state. This is
  a Phase-2 interface requirement, not a new current formal route.
  **Implementation supersession — 2026-08-26:** that last sentence records the
  state at the ruling. `FS-CVF-015` now supplies current formal recipient-side
  routes for `eats`, non-carceral `dwell`, `healthy` as care received, `secure`,
  and `meets`. Each requires a matching receipt, an authorised witness for the
  recipient, matching scope, and separation between witness and source; a
  predeclared alternate uses the same role, and roster membership is not a
  premise. The shipped record activates none of them. `learn` remains legacy;
  `believe` and the liberty of expression deliberately receive no delivery head.
  No route proves operation or actual-world arrival.

- **book-1 opens with one epigraph, and the poem's full text lives in git**, ruled
  2026-08-02. **Translation and attribution amended 2026-09-19, item 13:** the
  current exact rendering is recorded in `narrative-register-decision.md`
  under the 2026-09-13 delegated approval. It replaces the forced-rhyme version
  described below; the primary source's Varam Kettal subtitle supplies the
  reader attribution. The one-epigraph and no-structural-use limits stand.
  The remaining paragraph records the historical version.
  `book-1/epigraph.md` (unnumbered, so the chapter gates do not sweep it)
  carries stanza 4 of Bharati's *Yoga Siddhi* with the author's own translation ported
  verbatim from `book.md`'s appendix — the stanza whose subject, the daily search for a
  meal, is the condition the floor abolishes, closing on "Did you think I, too, would
  yield?". At most one, never structure, one sentence on who Bharati was — the settled
  constraints, all honored. **The current translation is the author's voice and
  remains theirs.** A session may propose a plainer rendering under the
  2026-08-20 protocol; it becomes canonical only after explicit approval of the
  exact displayed version. The full two-stanza text and both translations stay in git history when
  `book.md` is deleted, recorded in the deletion manifest as consciously kept there;
  the registry entry `bharati-yoga-siddhi` is the standing pointer.

- **The democracy/happiness analysis stands on V-Dem, re-derived**, ruled 2026-08-02 —
  the data-licensing fork, closed. EIU's index is non-redistributable and the registry
  is CC0, so EIU-derived values never enter it; cite-and-link was refused because Part
  V's worked example is the method performed and must be re-runnable by a stranger.
  The re-derivation is real work, priced in the Data section: V-Dem's Regimes of the
  World categories are not EIU's four, so the regime table and step sizes are
  re-derived and may shift — which the example's own discipline (test the claim you
  would love, report what survives) absorbs by design. The EIU registry entry stays
  source-reference-only, available for book-2 to cite-and-link.

- **Two severity refusals, both on lexical/structural grounds rather than taste.** **Directness is refused**: the committed corpus has exactly five relations with a `victim` place — `attack`, `bad`, `cruel`, `dangerous`, `injure` — and none means "directly"; `cause` (rinka) compiles but puts the person in the *effect* slot and is true of every injury in the cast, so as a boolean it routes nothing. Do not re-propose without a corpus name that carries the meaning. **Graded tiers are refused**: `building(MedSec, $x)` compiles and the constant is free, but `building/2` has no exclusivity constraint. Its relation-wide closure blocks direct ground assertions; it does not stop rules from deriving two placements for an offender matching two combinations, and `err(_, Placement)` is blind to that conflict at runtime. The full verifier's generated placement audit rejects a conflicting source outcome when run; it does not give the deployed society a reader or remedy. A graded outcome needs a mutual-exclusion marker built in the same edit.
- **Never route a constitutional judgment through the compute backend**, and the reason is not performance. An external predicate is a **trusted oracle, not something nibli proves**: a `true` reply is auto-asserted as a ground fact mid-query and never re-derived or checked (nibli `README.md:18`, and the *Trust boundary* callout in its compute-backend section — **cite that one by its heading, not a line number**; the citation has rotted twice, `:333` then `:323`, and is `:325` today). So a grade, tier or severity computed there enters the record as *a conclusion someone wrote*, which is exactly what chapter 1 says this design makes impossible. Embedding the backend changes who operates the oracle, not whether the result is derived. **Built-in arithmetic is different on trust and identical on lifecycle** — `product`/`sum`/`quotient` are computed locally with no third party, but the arithmetic fast path calls the same `assert_typed_fact`, so they leave the same untracked ground fact; "carries none of the oracle problem" is true of trust only. Two engine behaviours worth stating precisely because the earlier wording overstated both: an unreachable backend yields `UNKNOWN(BackendUnavailable)` and never `FALSE`, but a tuple already computed in that session still answers TRUE from the auto-asserted fact — an outage-*cache*, not a stall. And a universal over a number-bearing predicate is still vacuously true. It is **sometimes** no longer silent, and the earlier wording here overstated that twice. Since nibli `95cba22` a `[Domain]` note fires — but only where the restricting relation is **asserted**; put one rule in between and it goes quiet again (their corrected repro: `sum(every dog, 2, 2)` notes, the one-hop twin `sum(every animal, 2, 2)` does not). And **neither the note nor the proof step is reachable from `nibli-pin`**, which is the only binary this repo runs — verified 2026-08-01, no flag exposes it, and `nibli-host` still fails outright on a stale wasm. So treat the diagnostic as absent here, not as a safety net. What actually contains this is that there is nothing numeric to quantify over. The digit ban that kept it that way was a `verify.sh` section the 2026-09-12 decision retired; it is re-established as the development test `floor_vector_tests::nothing_in_the_enacted_lines_is_a_quantity`, which is negative-controlled and, being a development test, adds no gate to verification. Compute is legitimate for the claim registry and the method part; never for the society's own conclusions.
- **Chapter order is editorial — superseded 2026-09-16.** The sentence that stood here, "strictly computed, never chosen", was never true of the repository: `3-spine.md` generates the stratification and hand-lists the chapters, and the runtime order is the filename prefix. The rule is now *engines before breaks*, recorded in `book-1/contents.json` and enforced by development tests (R3 of *The rebuild of Book 1*). Exactly three elements remain exempt from the derivation gate and each is labelled in the text: the opening note, Part V, and the final method part. **From 2026-09-24, once implemented (D2 of *The revision rulings D1–D9*):** each derived chapter's closing argument section is a labelled exempt section as well.
- **The length invariant is "book-1 stays majority-derived, measured across the whole book"** — the derived chapters must outweigh the opening note plus Part V plus the method part, combined. **The ~38,000 target for Parts I–IV is retired — ruled 2026-08-03: content governs.** The book's length is an outcome, not a goal: expansion happens only where verified-untold material exists (TRUE in the engine, absent from prose), and every non-derived part landed under its old budget by exactly that rule (Part V ~5,500 against ~12,000; the method part ~4,100 against ~5,000). Figures are hand-maintained and go stale — re-run `wc -w book-1/*.md` before trusting any; measured 2026-08-03: derived 29,440 against 10,545 non-derived, the invariant holding with a wide margin. Do **not** restate the old cap — *"~14,500 against ~36,000 derived keeps it near 29%"* — and do not restate the retired 38,000 as a target; both belong to git history now. Since 2026-09-16 the measurement covers the ordered inputs only: `book-1/appendix/` and `book-1/source/` are outside it (R4). **From 2026-09-24, once implemented (D2 of *The revision rulings D1–D9*):** measured by section, derived sections against everything argued or exempt.

**Legacy `book.md` (below) — historical; do not port these into book-1 without re-checking them against the constitution:**

These were argued out and hardened in the text; several are stated as bright lines the book calls unamendable. A chapter edit made in isolation can easily re-introduce a passage the book now explicitly refutes, so check against this list before writing about points, identity, or voting.

- **Merit points are earn-only recognition** — never spent, gifted, priced, or converted. Recognition is also non-rivalrous: no fixed sectoral pools, because one nurse's acknowledgment must not come at another's expense. Inflation is controlled at the contribution level (per-task standards, caps, diminishing returns). See "What Merit Points Are—and Are Not". Scarce non-essentials above the floor are allocated by transparent non-market rules (need, rotation, waitlist, lottery), with merit only as a qualifying threshold. **Superseded for Book 1 by the 2026-08-07 economic settlement:** merit or recognition cannot qualify access to a physically scarce floor essential; above-floor allocation may use compatible democratic policy but never read `reward` as price, worth, or entitlement.
- **No negative scoring of persons.** Nobody's record is ever docked as punishment; sanctions reach *perks* through due process. Pollution and fraud are handled by regulating the enterprise — a different instrument from scoring a human being (book.md:339). **book-1 adopted this narrowed** (ruling 2026-08-02, in the settled decisions above): *no subtraction except by due process for one's own adjudicated fraud* — the wrongdoer clawback stands, the student clawback is deleted.
- **The rights floor is never gated on identity.** *Serve first, reconcile the record afterward*; `"no record found"` must read as *pending*, never *denied*. Stated as doctrine at book.md:746 and as the first bright line at book.md:2281. Every canteen, clinic, and shelter keeps a working non-biometric path.
- **One person, one vote is an unamendable floor.** Merit never weights votes; contribution can earn advisory voice (speaking slots, sponsorship), never a heavier ballot. book.md:2285 records the book correcting its own earlier drafts on this — don't reintroduce weighted voting anywhere.
- **Transparency is aimed at power, privacy at persons.** The three-tier data classification in the Quantum-Secure chapter (Public: budgets, flows, weights, tallies / Pod-visible: who-verified-whom, waitlists, standing tier / Private-ZK: biometrics, vote choice, medical records) resolves the transparency-vs-ZK tension. "Anyone can see who's amassing points" means tier and flagged anomalies, never an itemized feed.

## Editorial Decisions Already Executed

Deliberately removed — don't helpfully restore them: part-intro preview lists, and the mechanical "Coming Up Next" / "Next Steps" / "Ready to Begin" teasers (the three substantive Part-5 prose bridges were kept on purpose). Book headings carry **no** bold markers — `grep -c "^#\+ \*\*" book.md` should stay 0. Only one `## Key Takeaways` block survives, in Merit Points; that is intentional.

## Sourcing

Every statistic and named study in the book is listed under `# References & Data Sources`, grouped by chapter, with a primary source and URL. Adding an empirical claim means adding its reference there in the same commit. Don't fabricate figures; costed transition work belongs to Book 2 and needs real fiscal magnitudes and traceable evidence; the need for evidence is not itself an author gate.

## Vocabulary & Voice Conventions

The author has deliberately standardized terminology (see git history):

- **"employment"**, not "work" or "jobs" — the book redefines employment beyond salaried labor
- **Legacy `book.md` only:** **"merit points"** (lowercase in prose, "Merit Points" in headings) described recognition as replacing wages. Do not port that claim: Book 1 now keeps `reward` optional and non-operative while wages, compensation, returns, and supplements use separate legal relations.
- Recurring proper concepts (legacy `book.md` only — **MVS, pods and the tech stack are all out of scope for book-1**): **MVS** (Minimum Viable Society), **YAD** (Yet Another Device — government-issued device for those without smartphones), **Proof of Personhood** (Orb-style biometric identity), **pods** (local pods → regional councils → global federation), **local-first / offline-first micro-blockchains**, **quantum-secure**

**No counted claims in the prose.** Do not write "twenty-two entries", "four people have
shelter", "eight rights" or "one thing taken". Every counting claim in this book that has been
checked was *wrong*, not merely stale, and the design keeps moving — the evidence list changed
in a single commit, the floor may stop being eight, and more than one thing may become
takeable. The rule is not "delete numbers", which only makes the prose vaguer: **state the rule
that produces the count**. "Shelter derives for every confined person and for nobody else"
beats "four people have shelter" — it is stable under cast changes, more informative, and it is
what the book is about. `verify.sh` carries a **hard gate at zero** on this (since 2026-08-02;
the ratchet ran 14 → 0 across the chapter passes). Two allowlisted exceptions only: rhetorical
durations ("thirty years"), which are not claims about this design, and chapter 13's title —
"The One Thing Taken" *is* the single-deprivation claim, kept where the whole chapter defends
it, and the allowlist pins it to that file's line 1, so the phrase anywhere else counts.

Style — **legacy `book.md`/`manifesto.md` only; book-1's register is governed by the voice-boundary ruling in the settled decisions, and this warm first-person style must not be imported into derived chapters**: first-person, personal, and accessible; economic framing routinely contrasts *Keynesian*, *Marxist*, and *neoclassical* lenses (italicized); heavy use of **bold** for key claims. Both documents use curly quotes and em dashes throughout — match them. The technology chapters are intentionally more technical, with "Making It Simple: A Layman's Guide" as the deliberately non-technical retelling — keep that chapter jargon-free.

## Companion Repo

`~/projects/dhilipsiva/nibli` is a companion reasoning engine, cited by name in the tech backbone (book.md:923). Its `GUARANTEES.md` sets the register the tech chapters are converging on — state the guarantee, then name the sharp edge where it stops, rather than claiming unqualified safety. The nibli side of that convergence work lives in nibli's own `TODO.md`.

**The engine's assert-after-query contract, supplied by the engine session on 2026-08-17 for this repo to rely on:** a query answers the stratified model of everything asserted *before* it and nothing asserted *after* it — identical to loading the same statements in the same order into a fresh KB — and materialisation may only turn a non-definitive verdict definitive, never change a definitive one. This is the guarantee every `:accept-scoped` control and every mid-file assertion in a pin suite depends on, and it is the guarantee that broke: engine `176b132` folded an insert into an existing saturation while refusing only relations *read* under negation, when soundness needs every relation the delta can *reach* under negation. Growth propagated one hop and could not shrink the stratum above it, so an atom a query had already materialised survived a fact that should have retracted it. Fixed in `b97d1af` and gated upstream by a mechanism pin. **Two operational lessons, both verified here rather than taken on trust.** A binary sitting beside a checkout tells you nothing about what produced it — rebuild from a named sha and record the binary's own digest, because reading `HEAD` has already produced one wrong provenance record in this repo. And when a verdict looks wrong against this mechanism, **build the engine debug**: it carries a `debug_assert` comparing a resumed extension against a full recompute, compiled out of release, so a debug build panics where a release build silently returns the stale tuple.

**nibli's `utopia.nibli` keeps the dead word deliberately** — do not "finish" the v0.7 rename by touching it. It is `include_str!`-compiled into three binaries and its UI label is pinned byte-stable by `nibli/CLAUDE.md:161`, so renaming it is real cost in another repo for no gain here. The dependency is one-way: nibli contains zero references to this repo, verified across its whole tree.

## Repo identity

This repo was renamed from `dhilipsiva/utopia-reimagined` on 2026-07-30 (v0.7). **Never let anything reoccupy that name** — GitHub's redirect covers web, API and git over both protocols and carries issues, stars and forks, but it dies the instant something takes the old slug, including an accidental `gh repo create`. That is irreversible and there is no warning. Note also that a rename does not redirect `raw.githubusercontent.com` links, and does not fix anything that already resolved the old name into a stored identifier.

## Commits

- Author decision, 2026-09-13: automatically commit and push after each TODO
  item's implementation and required checks are complete, without waiting for
  a separate reminder. Repeat one item at a time without questions, using the
  author's delegated approval of recommended suggestions and prose above.
  This does not authorize including unrelated work or claiming unproved results.
- Make coherent, reviewable commits with affected rules, tests, and prose.
- Explain why the change is needed in the body, wrapped at about 72 columns.
- Run focused checks during authoring and the complete verifier for completion.
- No receipt, audit/closure successor, or separate tracker commit is required.

## 2026-08-15 no-external-reviewer dependency ruling

**Author-ratified: project completion and publication no longer depend on
external human reviewers, readers, panels, facilitators, coders, adjudicators,
custodians, or participants.** The author statement is: "Right now, I cannot
depend on other reviewers to finish the book. Please remove dependency on human
reviewers on this one and going forward."

This supersedes earlier project-gate consequences that kept Gate A, Gate C, or
Gate E false solely because an external human event was absent. It does not
alter constitutional requirements for independent courts, reviewers, auditors,
advocates, alternate authorisers, or separated public functions.

- Gate A uses the built repository source-derived adversarial audit (R7) plus a
  checker-derived closure record bound to an immutable verified candidate. No
  human act is required. R7 binds the semantic scope,
  protocol, declared criteria, checker controls, command chain, and
  Gate-A-applicable findings. It supplies no independent-human, reader,
  external-truth, operation, feasibility, liveness, calibration, timeless-
  completeness, or trust-root-authentication warrant.
- R6 reader and lived-experience evidence remains optional, unbuilt, and non-
  substitutable. No gate waits for it. Its absence means only that no reader
  comprehension, balance, suitability, lived-effect, or actual-user
  accessibility claim may be made.
- Gate C is narrowed to source binding, artifact integrity, navigation,
  internal consistency, and mechanical accessibility. Those checks may release
  the artifact but cannot establish reader suitability or accessibility for
  actual users.
- Gate E uses reproducible cross-book structural, model, provenance, and
  assurance checks plus a checker-derived closure record bound to an immutable
  verified candidate. No human act is required, and it makes no human-reviewed
  or successful-real-world-operation claim.
- External human review and reader studies remain admissible optional evidence.
  If recorded, their strict provenance, custody, chronology, conflict, and
  disposition rules still apply. They never delay project completion or
  publication.
