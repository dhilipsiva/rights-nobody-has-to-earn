<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Assurance Portfolio Decision

> **Status: author-ratified on 2026-08-08; R6 execution withdrawn from the
> current Book 1 program on 2026-08-11; formal implementation otherwise
> pending.**
> This decision selects the assurance portfolio: which route warrants which
> claim, what posture every claim carries, and what language each posture
> permits. It adds no predicate, rule, fact, pin, generator, `verify.sh`
> section, chapter, edition, or public coverage claim, and it **upgrades no
> existing claim's posture**. It renames nothing inside an existing artifact.

## 1. Decision

Book 1 adopts one portfolio with four parts:

1. **Seven assurance routes, none substitutable for another.** Each route
   states what it warrants, what it cannot, whether it is available today, and
   how it can be falsified.
2. **One canonical posture set.** Every substantive claim carries exactly one
   posture. Existing artifacts keep their own vocabularies; this ruling supplies
   the shared ceiling and a mapping onto it.
3. **A claim-language rule per posture** — the permitted verbs, the required
   scope bound, the forbidden extensions, and which posture may cite which.
   This is the ruling's real product.
4. **Three overlays** for the questions no single route answers: safety,
   liveness, and feasibility.

Underneath all four sits one sentence the rest of this document elaborates:
**a route establishes what it executes, over what it was given, and nothing
further.**

### 1a. What this ruling refuses at the outset

- **An aggregate assurance score, total, percentage, coverage figure, or
  "N of M established".** Part V refuses a total in print and gives the reason:
  a total is how an argument gets skipped. A portfolio score would be the same
  device wearing a different name.
- **A fourth Part V verdict token.** That vocabulary is closed at three and was
  "printed here once and never expanded". Naming argument as a posture
  classifies the *method*; it does not add a verdict.
- **Rewording a machine-enforced string.** Section 8 lists them.
- **Renaming a token inside an existing artifact.** The ruling maps; it does not
  rewrite reviewed sources.
- **A posture meaning "partly derived".** Section 5 disposes of partial
  formalisation as an unsplit claim rather than a weaker posture.
- **A new prose gate or `verify.sh` section before the ledger exists.**
- **Any edit to `method.md`**, whose five sealed decisions stand.
- **A route invented later to cover an awkward claim.** The list is fixed at
  seven and a route may not be split, renamed, or stretched to make a claim fit.

## 2. The seven routes

The gate bullet named five routes; the canonical-source tracker item names six.
The six-item list wins and **operational assurance is restored by name**,
because it is the only route that can carry an arrival claim — dropping it is
precisely what lets a safety result be read as "it works". A seventh is added
because an existing gate condition already demands a warrant none of the other
six produces.

| | Route | Warrants | Cannot warrant | Today |
| --- | --- | --- | --- | --- |
| **R1** | Formal entailment — Nibli, executed by `verify.sh` | What the rules do and refuse to do, over the sources and records supplied | Anything about the world, arrival, resources, comprehension, or its own trust root | **Built** |
| **R2** | Versioned quantitative and resource models | Magnitudes bound to a named envelope version, with stated uncertainty | Legal effect, entailment, lived effect, or that anyone acted | Unbuilt |
| **R3** | Dynamic simulations | Behaviour of queues, flows, shocks and transitions under stated assumptions | That the modelled behaviour occurred, or that the assumptions hold | Unbuilt |
| **R4** | The claim registry | Empirical figures about the world, with provenance, population, period, place and method intact | Any constitutional consequence, and any generalisation to this design | **Built**, split — see 2a |
| **R5** | Operational assurance | That something was actually done, delivered, advanced, or independently reproduced | Legal effect, and anything outside its dated scope | Not yet **available** |
| **R6** | Reader and lived-experience studies | Comprehension, balance, and human effects, for the tested audience within the disclosed sampling and method limits | Population statistics, authority over the people studied, or any legal or empirical claim | Unbuilt; execution withdrawn from the current Book 1 program |
| **R7** | Independent multidisciplinary and adversarial review | That an omission was proposed and received a reasoned public disposition | Derivation, measurement, authentication, or a veto — a reviewer compels a disposition, not acceptance | Unbuilt |

**Non-substitution.** One green route may not stand in for another. A route may
not be split, renamed, or reinterpreted to make a claim fit. This restates
existing doctrine and weakens none of it.

**Current-program disposition.** The 2026-08-11 author ruling withdraws
execution of R6 from the current Book 1 program because the required independent
human roles are unavailable. It neither removes R6 from the portfolio nor
changes its warrant. R6 stays unbuilt, FS-CLM-37 stays
Unestablished/route-unbuilt, and the reader condition of Gate C stays
unsatisfied. The dormant protocol may be revived only by a later author ruling
that restarts at its original pre-registration boundary. See
[`book-1-reader-evidence-execution-withdrawal-decision.md`](book-1-reader-evidence-execution-withdrawal-decision.md).

### 2a. R4 is two warrants, not one

The registry's staleness gate applies only to entries carrying a fetch script;
the rest are pinned sources whose provenance is the pinned version, refreshed by
a human re-cite pass rather than a gate. These are different warrants and an
Evidenced claim must name which it rests on:

- **R4a — script-refreshable.** Value written by a named script, date-stamped,
  and date-gated by `registry/check.py`.
- **R4b — pinned source.** Schema-checked, provenance by pinned version, re-cited
  by hand. It does not go stale by date, and it is not gated.

Do not count the registry's entries in prose. Most entries are R4b, and the
ratio moves.

### 2b. Built, available, and the difference

A route is **built** when its route-owned, evidence-producing check exists
and runs in `verify.sh`. A structural validator, generator, or evaluator
fixture that checks only a dormant contract does not build its route. A fixed
admission-gate component whose self-test checks only its interface is likewise
route machinery, not the route-owned evidence-producing check, and does not
build R6. The test fits R1 and R4 and no other, because the remaining routes
produce evidence outside this repository by their nature.

A route is **available** when its evidence contract, admissibility criteria,
named reviewer with the required custody attestation, fixed in-repo gate, and
watched-failing negative control exist, so that outside evidence can be
*admitted* and checked when it arrives. R5, R6, R7 and any externally produced
R2 or R3 result become usable through availability, never through being built.

Both count as established routes. **Neither is satisfied by naming one.**
Defining built as "gated in this repository" and stopping there would make
operational assurance permanently impossible, every arrival claim permanently
unestablishable, and the record-integrity verdict permanently frozen at NOT
ESTABLISHED. That consequence is the reason availability is a separate test.

### 2c. Every route declares a negative control

A route counts as built or available only when it declares a falsification
condition, ships a control or mutant set that must fail, and is gated by a named
check. The repository already holds itself to this — every generator carries
in-memory controls and prints their count, and the pattern guards carry positive
controls — and the house rule is stated in the method part: sabotage first,
trust after; a check that has never been watched failing is not yet a check.

Four routes have no falsification story yet. That is a reason they are unbuilt,
not an exemption from the requirement.

## 3. What a green `verify.sh` actually means

"Checked by `verify.sh`" currently conflates four different things. They are
separated here because the difference decides what a claim may say.

| Kind | Where | What a green result establishes |
| --- | --- | --- |
| **E — executable** | The chapter and floor pin suites; the record-snapshot, temporal, amendment and placement executions; the counterfactual fixtures | The engine ran and the claim held over the exact sources and records supplied |
| **G — pattern guard** | The jargon sweep, the counted-claims gate, the absence checks, the floor-noticing invariant, the recognition-arity and no-counted-degree guards, the control-scope check, the evidence-vocabulary check | A regex or text pass over the source found nothing. One of these says of itself that it is a pattern guard and not a proof; another reads only the spelling of a directive |
| **F — freshness** | The generator `--check` modes, the claim-table check, the registry staleness gate, the reader-evidence structural/evaluator controls, and the fixed reader-admission-gate component self-test | Reviewed source and generated artifact agree, and the checker's own mutants fail. Reader-evaluator fixtures and the gate self-test establish contract or interface behaviour only. Currency and artifact behaviour, not reader truth |
| **I — inventory** | The assertion-surface operation sets, the record classes, the red-team scenario narratives | A census is complete and reviewed assumptions are drift-sensitive. Explicitly a reviewed threat model rather than executable proof |

**Only kind E warrants Derived.** Kinds G, F and I warrant Checked and nothing
stronger. An inventory may never be cited as a proof or as a counterexample
harness.

**Two of these are compound, and the ruling says so rather than simplifying.**
The spine check and the assertion-surface check both run the engine over the
current constitution and then compare the result against a generated artifact:
the first shells the engine for its stratification, and the second consumes that
output. They are therefore an engine result *and* a freshness comparison, and a
claim resting on them must say which half it uses.

**What the quick mode omits**, stated exactly, because "quick proves nothing" is
false: it skips the chapter and floor pins, the record-snapshot executions, the
temporal executions, the amendment candidates, the placement matrix, the
reader-evaluator fixtures, and the counterfactual fixtures. It still builds the
engine, runs stratification, validates the reader contract structurally, and
self-tests the fixed admission-gate component. A quick run may support a
Checked claim about the artifacts and a narrow Derived claim about
stratification; it may support nothing else. The gate-component self-test and a
full reader-evaluator control run are both kind F, never reader evidence, and
never establish R6 availability or a reader result.

**No green run authenticates its own trust root.** The constitution, reviewed
contracts, generators, verifier, engine and human review can be weakened
together, and three artifacts already say so independently. That limit belongs
to every posture in this document.

## 4. The canonical posture set

Three bands. The band is what a reader must remember; the name is what a ledger
row writes.

**Band one — established by a built or available route.**

- **Derived.** An executable engine result over the exact current source, **or
  over a named bounded mutation of it declared in a fixture or reviewed audit**,
  establishes the claim for the records supplied. The mutation clause is not a
  loophole: it is how a restriction claim is established at all, since
  derivation is monotone and no probe stacked on the constitution can test what
  the constitution refuses. A Derived row resting on a mutation must name it, so
  the mutation is never invisible.
- **Checked.** A mechanical repository check establishes a property of the
  artifacts — the source text, a generated projection, or a reviewed census —
  never of the world and never of a semantic impossibility. Evidence kind G, F
  or I.
- **Evidenced.** A built or available evidence route establishes a claim about
  the world outside this repository, for a named population, period, place and
  method.

**Band two — stated, not established.**

- **Specified.** Book 1 states a complete constitutional contract — holder,
  duty-bearer, minimum or limit, admissible evidence, failure, interim
  continuity, remedy, appeal, audit, independent check — that is not formalised.
  Every ratified-but-unimplemented family is Specified.
- **Reasoned.** A reasoned argument against a named adversarial corpus reaches a
  verdict. Permanently weaker than Derived, and never citable as it.

**Band three — not established.**

- **Unestablished**, carrying exactly one mandatory disposition:
  `routed-book-2`, `external-assumption`, `route-unbuilt`,
  `evidence-pending`, `author-ruling-pending`, `refused`, or
  `not-establishable`.

Collapsing routing, refusal and the unbuilt route into one band is deliberate.
It makes the existing rule that classification is a disposition and not
assurance **structural instead of exhortative**: routing a claim to the second
volume leaves it, on its face, unestablished.
`evidence-pending` is the different missing-warrant state: the route exists,
but no admitted evidence currently establishes the claim.

### 4a. Why these names

`Derived` keeps its ordinary sense in the chapters. `Checked` and `Reasoned` are
chosen over the obvious alternatives because those alternatives already carry
load in shipped prose and one of them **inverts**: a chapter describes the vote
as guarded by refusal, meaning protected at the strongest available strength,
while the posture would have meant the weakest available warrant. A posture name
that reverses a reader's sense of a word in the same book is a defect, not a
coincidence of vocabulary.

## 5. One posture per claim

Every substantive claim carries exactly one posture. A claim that would carry
two is not a mixed claim — it is two claims, and it must be split until each
part carries one posture and one evidence kind. Atomicity is a precondition of
the ledger, not a matter of taste.

This disposes of "partial formalisation", which is not a posture. A partly
formalised domain is a Derived part, a Specified part, and usually an
Unestablished part, each with its own closure condition.

**Reserved joint forms are the one exception.** A verdict fixed as an immutable
verbatim string may carry two or more rows keyed to its clauses. Its text is
stored once and never re-derived from the rows.

## 6. Claim language, posture by posture

The permitted forms below are the ceiling, not a template to copy. Every claim
still needs its own scope bound.

### Derived

**May say:** what the rules do, in the present tense, over what was supplied —
refuses; does not derive; is refused; derives for every confined person and for
nobody else; loads and derives; cannot be written as a fact of the record. A
scope bound is **mandatory** wherever the claim concerns records: for the
records supplied, in the current source, in this design's own record.

**May not say:** always; can never; no law can; guarantees; ensures; proves; in
practice; in the deployed system; in time; will. No arrival or institutional
action verb. No extension of a refusal past the conclusion it reaches.

The standing worked example is a dead subtitle: *why no law can take them away*.
It was a Derived claim written in overreach language. The refusal covers
imprisonment and stops there, and a rule voiding credibility or docking
recognition for lacking a floor right loads without error. The sentence was not
slightly too strong; it named the wrong conclusion.

### Checked

**May say:** no rule in the current source reads this; the checks refuse a rule
of this shape; every rule-produced name in this version has a contract; the list
is current for this version of the source; the pattern is checked, and here is
what it cannot see. The evidence kind must be legible from the sentence.

**May not say:** nothing can read it; the design cannot do it; it is impossible;
proves; verified, used of a semantic property; exhaustive beyond the named
source version. A freshness result may not be offered as evidence about the
constitution, and an inventory may not be offered as a proof. **A Checked claim
may never be cited, quoted or paraphrased as Derived.**

### Evidenced

**May say:** the claim as its source states it, with population, period, place,
method and caveats intact and the source reachable by identifier. A modelled or
measured value carries its envelope version, or its engine commit and date.

**May not say:** a figure with no reachable entry; a value whose staleness gate
is failing; a single study or historical case generalised to this design or to
any society; an empirical claim used to establish a constitutional consequence,
or the reverse; a spliced or reconstructed quotation; a figure whose caveats
were dropped to make the sentence shorter.

### Specified

**May say:** Book 1 requires; the contract is; a rule family must supply; the
design's answer is; ratified but unimplemented; author-ratified on a date with
formal implementation pending. **The unimplemented marker is mandatory** wherever
a reader could take the sentence as describing something the machine already
does.

**May not say:** the constitution refuses; the rules prevent; the record cannot
hold; is protected; is guaranteed; the design catches — any indicative reading
as a current verified property. A Specified claim may not be cited to support a
Derived one, and may not appear as content in a chapter gated on derivation.

### Reasoned

**May say:** the three closed verdict tokens, printed beside the reasoning that
produced them; the objection and its source; what the argument concedes before
what it keeps. First person, in the three exempt elements only.

**May not say:** a fourth verdict token at any strength in any artifact; a total,
score, tally, percentage or aggregate. A verdict may not be cited elsewhere in
the book, in a planning artifact, or in a public claim as though a machine
established it, and repetition in a derived chapter does not upgrade it.
Reasoned argument creates no right, power or exception.

**A sourced report of an identified draft reader's or reviewer's reaction, used
as the origin of an objection or a design concession, is Reasoned and is
permitted in the exempt elements.** That register is already in print and this
ruling does not disturb it.

### Unestablished, by disposition

- **routed-book-2** — may say that the second volume owns the question, and must
  leave the answer open. May not imply the routed answer is favourable, likely
  or known; may not describe the claim as covered, handled, addressed or
  accounted for; may not add another in-book pointer to that volume.
- **external-assumption** — must name the owner, the evidence status, the
  consequence if it fails, and the claim limit. Plausibility is not evidence.
- **route-unbuilt** — planning and generated artifacts only, always with
  severity, consequence, owner, closure condition and public-claim restriction.
  **The disposition may not appear in book prose.** It is not satisfied by
  disclosure alone where the severity is critical, and it is not closed by
  assigning it outward.
- **evidence-pending** — the assigned route is built or available, but no
  admitted evidence currently establishes the claim. A valid `fail` or
  `not-evaluable` result remains recorded as that result and must never be
  rewritten as `not-run`. This disposition permits no positive public claim,
  may not be used while the route is neither built nor available, and closes
  only when admissible evidence establishes the bounded claim. Reader holdout
  lifecycle and result remain orthogonal. Snapshot-local attempt hashes are
  necessary but not sufficient: script 14 also requires the root
  `history_transition` fields `previous_source_commit`,
  `previous_source_sha256`, `previous_history_head_sha256`, and
  `history_head_sha256` to bind the nearest earlier normal first-parent commit
  that changed `reader-evidence.json`, its exact source bytes and prior history
  head, and the current history head. The transition preserves prior attempt
  prefixes and terminal attempts and permits exactly one domain and one legal
  step: one append, or an active nonterminal attempt becoming terminal. A
  successor pre-registration binds `predecessor_attempt_sha256` and
  `prior_history_head_sha256` to its frozen predecessor. A void attempt cannot
  feed a gate, and it does not erase the latest completed non-void result. A
  `void` attempt with `not-run` is valid. These checks cover visible normal
  first-parent Git history only; they cannot establish resistance to rewritten
  history or external truth.
- **author-ruling-pending** — neutral inventory, gap discovery, evidence
  collection and option briefs may proceed. A brief's recommendation is not a
  decision, and dependent prose, formalisation, pins and public claims wait.
- **refused** — may state what the design will not claim, with the reason and
  the cost. May not present a refusal as an absence or an absence as a refusal,
  may not quietly satisfy a control by relabelling it refused, and may not
  refuse a claim in one artifact while asserting it in another.
- **not-establishable** — may name the boundary. May not soften it to "not yet",
  imply a future route would fix an in-principle limit, or use it to excuse a
  control a built route could establish.

## 7. Safety, liveness, and feasibility

These are the three questions no route answers together, and separating them is
what the gate asked for.

**Safety** — nothing bad follows from the records supplied — may be **Derived**.
Required shape: a positive sentence naming what was executed and over what,
followed by an **enumerated** list of what it does not establish. A gesture at
further limits does not satisfy the second half.

**Liveness** — a successor arrives, a clock advances, a reviewer acts, a person
is released, a meal is served, a record is corrected — may **never** be Derived,
Checked or Reasoned. This is categorical, not a calibration, and no scope bound
rescues it. Its only established posture is Evidenced through operational
assurance, which is not yet available, so **every liveness claim is
Unestablished today**. Book 1 may state the condition and the failure polarity —
the rules make this a condition of that; the absence of it is marked as a named
failure; nothing here makes it happen. Book 1 may not state the arrival. A duty,
an obligation marker or an alarm is not evidence that anyone acted.

**Feasibility** — there is enough, it costs this, it scales, capacity holds
under shock — needs quantitative models or simulations together with operational
assurance. None is available. Feasibility claims are
Unestablished/route-unbuilt and **may not be written in Book 1 at all**. The
absence of a costing is not neutral: a feasibility sentence with no route is a
defect, not a gap.

## 8. Reserved wordings

Two joint forms are fixed and may not be reworded, replaced, or joined by a
third.

- **The artifact-level form** is the temporal case's verdict string, which is
  enforced byte-exact by its generator, emitted into the generated report, stored
  in the reviewed source, and cross-referenced by literal needle from the
  record-integrity case. Rewording it fails the verifier and breaks two other
  artifacts.
- **The prose-level form** is the method part's bounded-safety template: a
  positive scope sentence followed by an enumerated list of what it does not
  prove.

The chapter distinction between a measurement and an assurance stands. This
ruling does not redefine *verified* or *assurance* upward anywhere in book
prose.

## 9. Book 1 claim classes and their required postures

| Claim class | Required posture |
| --- | --- |
| Structural refusal — a rule shape the engine or a guard rejects | Derived, pinned, with a counterfactual fixture wherever monotonicity hides the restriction |
| Entailment about a named case | Derived, with the supplied-records scope bound |
| Absence — nothing reads this relation | Checked, phrased as "no rule in the current source reads it", never "nothing can" |
| Vocabulary, census, contract completeness | Checked, naming the source version |
| Ratified-but-unimplemented constitutional family | Specified, with the unimplemented marker, and not as content in a derived chapter |
| Survival of an objection | Reasoned |
| Empirical fact about the world | Evidenced, naming R4a or R4b |
| Arrival, delivery, release, correction, institutional action | Unestablished — liveness |
| Resource, cost, capacity, shock | Unestablished/route-unbuilt |
| Reader comprehension, balance, lived effect | Unestablished/route-unbuilt while R6 is neither built nor available; Unestablished/evidence-pending once R6 is available but no matching valid holdout pass establishes the claim; Evidenced only after that pass, bounded to the tested audience and disclosed method limits |
| Self-authentication of the trust root | Unestablished/refused, permanently |

The record-integrity verdict moves off NOT ESTABLISHED only on the condition
that artifact already states: every non-refused control reaching a verified
posture. That splits by route — most of its outstanding controls are Specified
with formalisation as the closure condition, and at least one is external and
waits on operational assurance becoming available. A refused control is exempt
from the upgrade condition and may not be used to satisfy it.

## 10. The unbuilt-route rule, and why building is not claiming

**No Book 1 claim may take an established posture through a route that is
neither built nor available.** A claim whose only route is missing takes
Unestablished/route-unbuilt and records severity, consequence, owner, closure
condition and public-claim restriction, through the mechanism the full-society
boundary already uses for unresolved items. Naming a route does not build it,
and a handoff to the second volume does not build it either.

**Building a route is work, not a claim.** Designing, pre-registering, piloting
and reviewing a route is permitted and required while the route is unbuilt. That
activity takes no posture and asserts nothing, so it does not violate this rule.

This distinction is what keeps the rule from freezing the gate ladder. The
public-edition gate depends on reader evidence, the reader route is unbuilt, and
its pass rule is itself a separate open author ruling to be made after a pilot.
That is a **sequence**, not a deadlock: the pilot may run and the rule may then
be ratified. Separately, R6 becomes available only when its evidence contract,
admissibility criteria, named reviewer with a gate-bound custody attestation,
dedicated executable evidence-admission/evaluation gate, and watched-failing
seeded control are complete. The dormant contract, structural checker,
deterministic evaluator, and fixed gate component do not complete that tuple.
The gate component's self-test supplies no reader evidence, reviewer custody, or
seeded-control result and does not build or make R6 available. Every active
completed attempt must store `gate_admission_receipt` as the fixed digest-bound
gate's exact output. Only a matching valid holdout pass whose exact receipt has
`decision=admit` may establish FS-CLM-37. A gate condition requiring a route to
exist is a sequencing requirement; it is not a Book 1 claim and this rule does
not bite on it.

## 11. No bridge into the reasoning engine

No modelled, measured, simulated, operational or reader result may enter the
reasoning engine through the compute backend, an external predicate, or built-in
arithmetic. An external predicate is a trusted oracle rather than something the
engine proves: a positive reply is auto-asserted as a ground fact mid-query and
never re-derived. The arithmetic fast path leaves the same untracked fact.

A value from another route reaches the engine only as an authenticated,
adjudicated, purpose-bound premise through a distinct relation, and a
conclusion-only predicate must still be derived. Compute remains legitimate for
the claim registry and the method part, and never for the society's own
conclusions.

This clause exists because this ruling is the first artifact to place numeric
routes beside formal entailment in one portfolio, which is exactly the
composition the oracle refusal was written to forbid.

## 12. No score, no total

The portfolio produces no aggregate. No percentage, no coverage figure, no count
of established claims, no portfolio health number, and no ranking of domains by
posture.

One narrow exception is permitted because the repository already computes it and
because it is conservative rather than flattering: a **single non-numeric
verdict** may be derived from a posture census by the rule that any non-refused
row below an established posture yields the weaker overall verdict. That is a
worst-case rollup, not a score, and it may not be converted into a figure.

## 13. Mapping existing vocabularies

Existing artifacts keep their own terms. This table is the ceiling they map
onto, not a rename, and every consequential enum in a reviewed source needs a
row here before the ledger can check it.

| Existing term | Where | Canonical posture |
| --- | --- | --- |
| Executed pin verdict | Chapter, floor and counterfactual suites | Derived |
| Structural negative control passed | Generator self-checks | Checked (kind F) |
| Report is current | Generator `--check` modes | Checked (kind F) |
| Fixed reader-admission-gate component self-test passes | `verify.sh` quick and full modes | Checked (kind F): interface behaviour only, not R6 availability or reader evidence |
| Reviewed operation set, record class, scenario | Assertion-surface and record-integrity inventories | Checked (kind I) |
| `current_verified` | Record-integrity controls | Derived or Checked, per its evidence kind |
| `external_verified` | Record-integrity controls | Evidenced |
| `book1_target_unimplemented` | Record-integrity controls | Specified |
| `book2_external_assumption` | Record-integrity controls | Unestablished/external-assumption |
| `refused_or_unprovable` | Record-integrity controls | Unestablished/refused or /not-establishable |
| NOT ESTABLISHED | Record-integrity top verdict | The conservative rollup of section 12 |
| Formal now | Coverage map | Derived |
| Partial formalisation | Coverage map | **Not a posture — split the row** |
| Part V specification only | Coverage map | Specified, or Reasoned where it is an argument |
| Absent | Coverage map | Unestablished, with its disposition named |
| Survives / Survives, narrowed / Fails as stated | Part V | Reasoned |
| `book1_target_unimplemented` on a prose target | Amendment audit | Specified |
| Narrowness-impact terms | All generators | Not postures — they record an artifact's effect on a standing claim, and both must be stated |

**Closure rule.** Every enum key in a reviewed source whose name concerns
posture, status, disposition or verification meaning needs a mapping row. The
ledger fails when a reviewed source introduces one that has none. The remaining
per-case and per-scenario enums are to be mapped in the same change that builds
the ledger, mechanically rather than by transcription.

## 14. The generated claim-assurance ledger

The ledger is a **generated projection of the one canonical source**, not a
parallel reviewed file. Posture, route, evidence kind, scope bound, disposition,
owner, severity, closure condition, public-claim restriction and any mutation
reference are **fields on the canonical source's claim records**; the generator
renders and gates them exactly as the coverage and reader views are rendered.
A second hand-maintained matrix of assurance truth is refused by the ratified
canonical-source mandate, in those words.

The generator must fail on: a claim with no posture; a posture unsupported by
its route; a Derived row whose evidence kind is not executable; a Derived row
resting on a mutation with no mutation reference; a Checked row phrased as an
impossibility; an Evidenced row with no reachable source or a failing staleness
gate; a Specified row without its unimplemented marker; a Reasoned row cited as
Derived; an Unestablished row with no disposition, or with a `route-unbuilt`
disposition and no severity, owner, closure condition and claim restriction; an
`evidence-pending` row whose route is neither built nor available; an
`evidence-pending` claim whose valid `fail` or `not-evaluable` result was
rewritten as `not-run`; a reader attempt whose local hash chain is invalid; a
reader `history_transition` that is absent or does not match the nearest
earlier normal first-parent commit changing `reader-evidence.json`, the exact
prior source bytes, previous history head, or current history head; a transition
that rewrites an attempt prefix or terminal attempt, changes both domains,
advances more than one step, or performs anything other than one append or an
active nonterminal-to-terminal change; a successor pre-registration whose
`predecessor_attempt_sha256` or `prior_history_head_sha256` is absent or
mismatches its frozen predecessor; an active-attempt change that overwrites the
latest completed non-void result; a study ID,
coded-record commitment, custody ID or digest, or receipt ID duplicated anywhere
across pilot and holdout history; a run with anything other than exactly one
freshness record; a void attempt that feeds a gate; a pilot or holdout freeze
with no external prior-commit or custody binding, or whose computed
`attested_payload_sha256` does not match the exact bound payload; a holdout
freeze that fails to bind the exact rule, protocol, instrument/rubric,
candidate, sample/recruitment, and disclosure digests; a pre-registration or
receipt whose `structural_checker_sha256` does not match the checker it binds; a
holdout with no `frozen_ratification`, or whose embedded historical rule,
candidate, pilot basis, or digest fails independent validation; a candidate
commit that is not an ancestor of current `HEAD`; a noncanonical UTC freeze,
`completed_at`, `voided_at`, or `revealed_at`, or a chronology in which
freeze does not precede the terminal event, reveal does not follow it, or a
successor begins before the predecessor's required events; an active completed
attempt with no `gate_admission_receipt`, a receipt other than the exact output
of the fixed digest-bound gate, or a decision other than `admit` offered to
establish FS-CLM-37; a structural checker or evaluator fixture substituted for
the dedicated R6 gate, gate-bound reviewer custody, or watched seeded control;
once values exist, a missing derived end-to-end below-, exact-, or
above-boundary fixture at a reachable observation, or an edge-domain case that
is neither explicit nor fail-closed; a populated candidate, `author-ratified`,
frozen, or completed stage with a required watched-failing mutation missing,
empty, or falsely marked inapplicable; an Evidenced reader claim without a
matching valid holdout pass and exact gate admission; a liveness claim
in any posture but Unestablished; a feasibility claim present at all in Book 1;
an established posture resting on a route that is neither built nor available;
a route with no declared negative control; a reviewed enum with no mapping row;
and any aggregate score.

For external custody, these failures validate the recorded structure and exact
payload binding only; they do not establish external existence or truth. The
Git transition check is bounded to visible normal first-parent history and does
not establish resistance to rewritten Git history.

Building it is tracked with the canonical source, not as a competing item.

## 15. Book 1, Book 2, and the engine

Book 1 owns the route list, the posture set, the claim-language rules, the three
overlays, the reserved wordings, the claim-class allocation, the unbuilt-route
rule, the no-bridge clause, and the ledger contract.

Book 2 owns the operational side of the routes it will run: model construction,
versioning and calibration; simulation design and validation; operational
evidence collection, provenance and independent reproduction; the admissibility
contract that lets outside evidence be admitted here; and the staffing, cost and
capacity of all of it. It may not upgrade a posture, invent a route, or convert
a routed claim into a closed one.

The engine may consume an authenticated, purpose-bound premise from another
route through a distinct relation. It is not credited with measuring, modelling,
simulating, authenticating, reading a person, or establishing that anything
happened.

## 16. Acceptance cases for implementation

| Situation | A complete answer must identify |
| --- | --- |
| A chapter sentence says the design refuses something | Derived; the exact conclusion the refusal reaches; the scope bound; the pin, and the counterfactual where monotonicity hides the restriction |
| A sentence says nothing reads a relation | Checked; the source version; the phrasing that stops short of impossibility |
| A quick run is green and someone cites it | Which half is being cited — the stratification engine result, or the artifact comparison — and the explicit list of what quick omits |
| The fixed R6 gate component self-test is green | Checked (kind F) for its interface only; R6 remains unbuilt and unavailable without the complete route tuple |
| An inventory is offered as proof | Refusal, with the inventory's own limit clause quoted |
| A figure appears in prose | Evidenced; R4a or R4b; the reachable identifier; population, period, place, method, caveats |
| A ratified-but-unimplemented family is described | Specified; the unimplemented marker; the bar on appearing as chapter content |
| A Part V verdict is quoted elsewhere | Refusal; Reasoned may not be cited as Derived, and repetition is not an upgrade |
| Someone asks whether the floor arrives | Unestablished — liveness; the condition and the failure polarity may be stated; the arrival may not |
| Someone asks whether it is affordable | Unestablished/route-unbuilt; the sentence is not written |
| A routed claim is called covered | Refusal; routing is a disposition, and the row stays Unestablished on its face |
| A model result is proposed as an engine input | The no-bridge clause; the distinct authenticated relation; the conclusion-only predicate still derived |
| A reader pilot has run but the pass rule is unratified | Building is work, not a claim; the route is not yet available; the gate's claim is not yet sayable |
| An independent reviewer proposes an omission | R7; a reasoned public disposition; add, classify with reasons, or retain as a severity-rated limit |
| R6 is available but the holdout is absent, not evaluable, or validly fails | Unestablished/evidence-pending; retain the exact result and permit no positive reader claim |
| A matching valid holdout passes under available R6 in a valid `history_transition`, carries the required frozen-predecessor pre-registration bindings, and its stored `gate_admission_receipt` exactly matches the fixed digest-bound gate's output with `decision=admit` | Evidenced for the tested audience and disclosed method limits; no wider claim and no substitution for another route |

## 17. Evidence and limits

What this decision does **not** establish:

- that any claim in this repository is correctly classified today — nothing is
  reclassified by this ruling, and no posture is upgraded by it;
- that the built routes are sound, only that they run and can be watched
  failing;
- that the unbuilt routes will be built, or that anyone owns them yet beyond the
  owners already recorded;
- that a posture prevents an overclaim — it makes one legible to a reviewer, and
  a portfolio is only as honest as the review that applies it;
- that the ledger's fail conditions are complete, or that they will catch the
  next failure mode rather than the last one; or
- that any of this authenticates its own trust root. The routes, the
  contracts, the generators, the verifier, the engine and the review can be
  weakened together.

**Remaining boundary:** this is an allocation of warrants and a discipline of
language. It is not evidence, and it does not make any claim in Book 1 truer
than it was.

## 18. Ratification record

On 2026-08-08 the author ratified:

- [x] seven non-substitutable assurance routes, with operational assurance
  restored by name and independent review added because an existing gate
  condition demands a warrant no other route produces;
- [x] the split between a **built** in-repo route and an **available** external
  route, so that operational assurance and the record-integrity verdict are not
  frozen by construction;
- [x] a declared negative control as a precondition of any route counting;
- [x] the separation of a green verifier result into executable, pattern-guard,
  freshness and inventory kinds, with only the executable kind warranting
  Derived, and the compound checks and the quick mode's omissions stated exactly;
- [x] the canonical posture set in three bands — Derived, Checked, Evidenced;
  Specified, Reasoned; Unestablished with a mandatory disposition — named so
  that no posture inverts a word's sense in the book;
- [x] Derived extended to a named bounded mutation with a mandatory mutation
  reference, so restriction claims have a posture at all;
- [x] one posture per claim, partial formalisation disposed of as an unsplit
  claim, and reserved joint forms carrying rows keyed to their clauses;
- [x] the per-posture claim-language rules and their citation restrictions;
- [x] safety as Derivable with an enumerated non-extension clause; liveness as
  categorically never Derived, Checked or Reasoned and Unestablished today;
  and feasibility as unwritable in Book 1;
- [x] the reserved wordings, including the byte-exact artifact verdict string;
- [x] the claim-class allocation and the record-integrity upgrade condition;
- [x] the unbuilt-route rule together with the clause that building a route is
  work rather than a claim, which keeps the gate ladder a sequence;
- [x] the no-bridge clause against the compute backend, external predicates and
  built-in arithmetic;
- [x] the refusal of any aggregate score, with one conservative non-numeric
  rollup permitted because the repository already computes it;
- [x] the mapping of existing vocabularies onto the ceiling without renaming
  anything inside an artifact, plus the closure rule for future enums; and
- [x] the ledger as a generated projection of the one canonical source, with its
  fail conditions and its tracking folded into that source's item.

This ruling is ratified and **unimplemented**. It creates no predicate, rule,
fact, pin, generator, verifier section, chapter, programme, release or public
coverage claim, and it upgrades no existing claim's posture. Each artifact still
carries its own vocabulary until the ledger lands.

### Subsequent disposition extension

On 2026-08-11 the author selected “Add evidence-pending (Recommended)” when
asked how to represent the interval in which a route is built or available but
the evidence required for its claim is not admitted or does not establish it,
then instructed implementation of the plan carrying that selection. This
extends only the Unestablished disposition enum and the checks that keep it
honest. It builds no route, admits no evidence, changes no result, upgrades no
claim's posture, changes no gate, and closes no tracker item.

### Current reader-route implementation note

The later dormant reader implementation includes the reviewed machine source,
generated report, structural checker, deterministic evaluator, and fixed
admission-gate component. The gate component's self-test is Checked kind F
only. No taxonomy label, threshold value, pilot, valid seeded-control run,
second threshold ruling, frozen or completed holdout, named reviewer,
gate-bound reviewer custody attestation, or `gate_admission_receipt` exists.
The dormant `history_transition` has null `previous_source_commit`,
`previous_source_sha256`, and `previous_history_head_sha256` values and the
deterministic empty `history_head_sha256`; no pre-registration predecessor
binding exists. R6 therefore remains unbuilt and unavailable, FS-CLM-37 remains
Unestablished/route-unbuilt, and the public-edition gate is unchanged. This
current-state note is not the reserved second author ruling.
