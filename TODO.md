<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# TODO — audited specification and Book 1 projection

**This tracker covers the principal formally audited constitutional
specification and its Book 1 reader projection.** Book 2 has its own inactive
tracker. This file is strictly future-facing: a bullet is deleted only after its
implementation is complete and the relevant pins and full verifier pass.
Drafted work is not
landed work. History belongs in git.

The repo is producing one formally audited specification, two controlled books,
and a clean legacy deletion:

- **The formally audited specification is the principal product.** Its exact
  version includes `new-book-plans/constitution.nibli`, the reviewed decisions
  and canonical contracts that define its scope, executable pins and
  counterfactuals, generated projections, and executable tests of their formal
  consequences. No individual report or partial green
  check is the product by itself.

- **book-1** — the active reader-facing derivation of the destination, in two
  parts with a deliberate seam. It does not override or complete the formal
  specification:
  - **Parts I–V — the constitutional and social destination.** What the society
    must guarantee, permit, organise and constrain, including normal, failure and
    recovery interfaces — never transition or costed operation. Derived from the
    constitution and **gated on it**.
    **Jargon-free** — a general reader finishes Part V and stops, and the
    formalism is never mentioned in these parts.
  - **Final part — the method, explicitly optional.** The constitution, the
    derived spine, the compile-time firewall, the evidence/conclusion split, and
    what the logic refused. Labelled as a different kind of reading. The only
    place the formalism appears, and what answers "you built a machine and hid it".
- **book-2** — **how the destination would be operated and reached within a
  declared, versioned reference envelope, including its local starting-state
  conditions.** It remains agnostic to any single existing local society and may
  compare destination-compatible transition paths without silently changing the
  audited destination. It owns
  staffing, costs, capacity, resources, technology, workflows, transition,
  deployment, empirical feasibility, and operation under ordinary and declared
  shock conditions. Its tracker is `book-2/TODO.md` — collect there, but do not
  execute Book 2 work until Book 1 — First Edition actually ships at Gate C.
  book-1 references it once, at the end.
- **`book.md` and `manifesto.md`** — legacy, to be **deleted** once both new books
  are written. Nothing in this tracker improves them. The one obligation they carry
  is that no valuable material is lost on the way out; what still needs porting is
  itemised under **Legacy harvest** below (the 55 sourced references are already in
  `registry/claims.json`, and the five bright lines are swept — the result stands
  under **Standing facts and methods**; the clawback consequence it forced was ruled
  2026-08-02, bright line 2 standing narrowed).

**THE WORKING ORDER.** All fourteen chapter passes are complete (2026-08-02) and
their records live in git, not here. What remains runs in two preliminary phases,
then a full-society expansion backlog and cross-cutting sections:

1. **Phase 1 — author-gated decisions.** The volume, edition, stopping
   boundary, state form, political membership, substantive equality, bounded
   plural economy/protected private sphere, family/dependency/reproduction/
   collective-plurality, ecological/future-generation/commons/non-human-
   animal, and public-safety/defence/emergency/external-power baselines are
   settled, as are the assurance portfolio, the narrative register, and the
   reader-evidence protocol with its threshold timing. Execution of the human
   reader route and its reserved post-pilot threshold ruling was withdrawn from
   the current Book 1 program at `907ddd0`. The 2026-08-15 ruling at
   `a8d6fd5` makes R6 optional and removes external-human participation from
   Gate A, Gate C, Gate E, publication, and project completion. The book's
   stated thesis and its paired second stress case were ruled 2026-08-17 at
   `ef0a48c`. **No author ruling is currently blocking implementation work**;
   the outstanding author obligation is drafted prose, not a decision. Neutral
   inventory and decision briefs may proceed in parallel; each still-gated
   domain's rules, prose, and public claim wait for its own author ruling. This section has been
   destroyed by tooling once and is watched accordingly.
2. **Phase 2 — engine handoffs (nibli).** The three capability audits are closed
   and recorded in source-bound planning artifacts, and the fail-closed witness
   enumeration and corpus-scoped text-compute repairs have landed, as have the
   two curated approval names the `approves` split needed (engine `8935611`).
   **No engine handoff is open.** The soundness regression that made the suite
   red is closed — engine `b97d1af`,
   verified here before acceptance — so the default `./verify.sh` path is green
   again and needs no pin. No repair here is an election result service;
   democratic formalisation still uses authenticated external result
   certificates.
3. **Full-society expansion — implementation backlog.** The ratified constitutional
   mandate remains the legal spine, but the completion target also requires a
   versioned claim-assurance, defect-disposition, and response-stage map for
   declared social axes and envelope, a functional cross-domain model, and a
   reader-facing structural contract, ordered from scope mapping through the
   repository adversarial audit and checker-derived closure.

The remaining sections are cross-cutting: the book-1 work that remains around the
finished text (the pre-expansion text was complete as of 2026-08-03 — epigraph, opening note,
the derived chapters, Part V and the method part are all present in source; what is left is
the full-society destination expansion, the living Creative Commons publication
and conversation layer, and the licence files), the reach
plan, data work, legacy harvest, and a pointer to book-2's own tracker. **Standing
facts and methods** closes the file and holds knowledge, not tasks.

Plain bullets, never numbered. Delete a bullet entirely when it fully lands;
update it if only partly done. Before rewording or deleting any heading, run a
needle census and migrate any live `owner_ref` or `source_ref` to its controlling
decision record; several strings are also *prefixes* of longer needles. Read-only inventory, evidence gathering and neutral
decision briefs may run in parallel; shared-tree edits, verification and commits land
serially, one owned item at a time.

**THE FORMAL-SPECIFICATION WORKING UNIT — author-ratified 2026-08-30.** A
top-level constitutional implementation item is one coherent assurance batch,
not a promise that every line is written in one sitting. Before shared-tree
editing, expand the active item in `tmp.txt` into authoring/review slices
targeting no more than four hours of active work. Every slice states its formal
surface, positive and negative checks, counterfactual impact, governed sources
and projections, Book 1 reader consequence, and Book 2 handoff.

The workflow states are **Planned**, **Drafting**, **Drafted — not audited**,
**Frozen candidate**, and **Audited/landed**. They describe work, not truth. They
must never be confused with the claim postures Derived, Checked, Evidenced,
Specified, Reasoned, or Unestablished. A drafted slice receives no audit warrant,
may make no public completion claim, and remains part of its open parent item.

Use focused checks while editing and the complete `./verify.sh` before calling
an implementation verified. Ordinary commits and tracker updates need no receipt,
audit/closure successor, staged-byte manifest, or scratch-file removal. The
2026-09-12 author decision supersedes the earlier administrative batch protocol.

Verification means substantive Nibli pins plus contradiction scans. Keep the
complete run within a few minutes; do not add hashing, provenance, freshness,
publication, or audit-ledger gates. Prose consistency is a separate review.

Author instruction, 2026-09-13: implement one TODO item at a time, verify,
commit, push and repeat through the backlog without questions. Recommended
suggestions and prose have delegated author approval; separate per-passage
approval pauses are superseded. Do not invent personal testimony or external
evidence, and do not delete unfinished work merely to empty the tracker.

Bullets prefixed **[AUTHOR-GATED]** need the author's own voice, personal memory,
or a design decision — they are collected in phase 1 rather than scattered.

**THE INCLUSION GATE — applies to Parts I–V only.** Those parts describe a
destination, not a route. Before any passage goes in, two tests: (a) does it
describe what the society must guarantee, permit, organise, or constrain — not
transition, costed operation, or how anyone gets there? and (b) does the
constitution derive it? A passage failing (a) belongs to **book-2**;
one failing (b) belongs in Part V's explicitly-not-derived section, or in the
opening note, or nowhere. Exactly three things in book-1 are exempt and each is
labelled as such: the opening note, Part V, and the final method part.
Anything about building up, scaling out, phasing in, or persuading anyone is out
of Parts I–V by construction.

The **final part is exempt and inverted**: it is *about* the constitution rather
than gated on it, and it is the one place jargon is allowed. Keep the seam sharp —
if a reader cannot tell they have crossed into a different kind of chapter, the
seam has failed.

Settled design decisions live in `CLAUDE.md`, not here. Planning material is in
`new-book-plans/`; the full-society expansion backlog below turns the ratified
constitutional boundaries and full-society boundary into active work.

---

## Phase 1 — Author-gated decisions. Rule each before its dependent implementation.

Each of these is a design decision, not a task. Record the ruling in `CLAUDE.md` when it
lands, so it is not re-proposed. **This section was destroyed by tooling once** — a
tracker-edit slice in `412e5a4` anchored on the next `---` after a separator an earlier
cleanup had removed, and swallowed 145 lines, all but one of them open; the loss went
unnoticed because nothing checks this file, and a later commit then described the
emptied section as "every earlier decision was ruled", which was false. Treat these as
the most expensive lines in the file. Line numbers cited inside bullets may predate
later edits — re-derive before trusting.

The author has directed the project toward a well-balanced two-book model. Book
1's use of “comprehensive” remains bounded to its Gate B/C declared scope, and
the Gate E claim is bounded to a declared reference envelope. The two-book
container, C-then-E release sequence, and versioned stopping boundary are settled.
The state form, residence-first political membership, substantive equality,
bounded plural economy/protected voluntary sphere, family/dependency/
reproduction/collective-plurality, ecological/future-generation/commons/
non-human-animal, and public-safety/defence/emergency/external-power baselines
are also settled, as are the assurance portfolio, the narrative register, the
reader-evidence protocol with its threshold timing, and the delivery-and-receipt
ruling of 2026-08-18 that fixes which floor items get an arrival route, who may
write the recipient-side receipt, and that the routes ship dormant. Execution of that
protocol and its reserved threshold ruling is withdrawn from the current Book 1
program at `907ddd0`; no pilot, threshold, holdout, accessibility pass, or
reader claim follows. Scope inventory and other author-ruling work may proceed.
Gate C remains open only for its mechanical artifact and release work; the
project may not present withdrawal as completion of those checks.

- **Channel precedents worth carrying.** Two corpus requests closed (`ratifies`
  and `endorses` for the approval split; `receives` for the recipient-side half
  of a delivery). The prompts are in git history; three lessons are not, and
  each cost a round trip.
  - **Resolve a candidate lemma to its English corpus name; never guess English
    spellings.** `nibli-lexicon`'s `by_provenance` is that bridge. Both
    near-misses on this channel were the same shape — a Lojban source word in
    hand, English spellings guessed, the name declared absent. `cpacu` maps to
    `get`, which existed at the sha we measured against.
  - **Say whether a converse-sense name must be an independent relation or an
    alias.** A `Swap` alias compiles to the same stored relation with places
    exchanged, so asking for `receives` as the converse of `gives` would have
    made the giver's assertion establish receipt — the one thing the delivery
    family forbids, arriving through a curation choice no rule review inspects.
    The engine session caught this unasked; the request could not distinguish
    the two and the result turned on it entirely.
  - **Verify the reply, do not accept it.** Rebuild from the named sha rather
    than trusting the binary beside the checkout, and measure the property the
    request exists for instead of reading it off a declaration.
## Full-society expansion — implementation backlog

This is the canonical merged redesign backlog for the ratified constitutional
mandate and the author-directed full-society completion target. Except for the
staged T3 path already named below, it is future-facing: the mandate, coverage
map, taxonomy, democratic corridor, domain ledger, system map and reader contract
set requirements, but do not make an unimplemented predicate, duty, institution,
operation, delivery route, remedy, social outcome or narrative current.
The federal parliamentary state form and residence-first political membership
are formalized and prose-landed; their institutions remain operationally
unimplemented. Their controlling contract is
`new-book-plans/book-1-state-form-and-political-membership-decision.md`;
source-supplied office parameters and later operational mechanics remain
delegated implementation choices only inside its hard constraints, not a
reopened author gate.
The substantive-equality and anti-subordination baseline is author-ratified.
Its person-held barriers and the statistical/diagnostic interface are
formalized; this does not complete its other remedial or operational contracts.
Its controlling contract is
`new-book-plans/book-1-substantive-equality-and-anti-subordination-decision.md`;
statistical thresholds, quota values, operational methods, and programme
workflows remain delegated only inside its legal reach, proof, data, continuity,
and remedy constraints.
The economic-pluralism and protected-private-sphere settlement is
author-ratified, formalized, and prose-landed. Its formal effects remain source-
and repository-bound and create no operated economy, delivery, liveness,
calibration, or external truth. Its controlling contract is
`new-book-plans/book-1-economic-pluralism-and-protected-private-sphere-decision.md`;
rates, budgets, quantities, prices, production, inventories, actuarial methods,
staffing, workflows, monetary instruments, capacity, and empirical feasibility
remain Book 2 work. Later implementation may choose only inside its plural-form,
floor, labour, property, contract, private-power, scarcity, federal, temporal,
data, and remedy limits.

The family, dependency, reproduction, and collective/plurality settlement is
likewise author-ratified but unimplemented. Its controlling contract is
`new-book-plans/book-1-family-dependency-reproduction-and-collective-plurality-decision.md`.
Common-tier law must choose the uniform adulthood age; later law may define
decision-specific early authority, family and support procedures, assisted-
reproduction safeguards and compensation, any assisted-dying regime, succession
mechanics, and collective membership and consultation procedures only inside the
ratified rights and continuity limits. Book 2 owns care services, family
proceedings, reproductive and palliative capacity, land/title administration,
language services, consultation, records operations, staffing, costs, and
workflows. Those delegated details do not reopen children's independent rights,
automatic adulthood, supported agency, bodily authority throughout pregnancy,
public-first care, family plurality, or rights-bounded collective autonomy.

The ecological, future-generation, commons, and non-human-animal settlement is
likewise author-ratified on 2026-08-08 and unimplemented. Its controlling
contract is
[`new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md`](new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md).
The existing material-floor inventory remains unchanged; the environmental
right is distinct, Class 9 continues to govern commons and future ecological
capability, and credibly sentient non-human animals are Class 10 protected
subjects rather than constitutional persons. Numerical ceilings, measurement,
models, inventories, monitoring, facilities, staffing, transition, and
empirical feasibility remain Book 2 work. Those delegated operations do not
reopen the science/precaution/non-regression boundary, dual floor-and-ceiling
continuity, separate Guardian and Advocate, direct animal protection,
categorical refusals, enhanced-use tests, or fresh temporal contracts.

### Scope and guardrails

- The ratified scope and contract boundary live in
  [`new-book-plans/book-1-constitutional-coverage-map.md`](new-book-plans/book-1-constitutional-coverage-map.md),
  with the taxonomy, time, and edition decisions beside it. `CLAUDE.md` owns the
  settled rulings; this section owns the work still required to implement them.
- `reviews.md` is a tracked but undated collection of overlapping reviews. It is an
  idea source, not a specification: some findings describe older drafts and some
  recommendations conflict with settled decisions.
- Preserve the distinction between a **verified derived claim**, a **Part V
  specification or argument**, and a **Book 2 operational design**. Keep formal
  methods vocabulary out of derived chapters; the opening note, Part V, and
  `method.md` are the labelled exceptions.
- Do not pursue symmetry by making recognition rankable, the existing
  recognition predicate `reward` operative, or standing purchasable. The economic
  ruling permits above-floor wages, profits, savings, returns, grants, prizes,
  subsidies, incentives, and contribution-based supplements through legal
  relations distinct from recognition. Grants, prizes, subsidies, restitution,
  and supplements also require purpose-specific, equality-compatible criteria;
  none may read `reward`, `false`, `lose`, or a personal-worth score.
  Do not mistake a provider's assertion for usable delivery, or call a chain self-
  healing while `owe`, `become`, or `obliged` remains unread.
- Standing and entitlement to a material floor may not depend on work, virtue,
  wealth, citizenship, documentation, score, compliance, contribution, a qualifying
  test, or official approval. Necessary, proportionate, contestable clinical or
  needs evidence may select a safe/accessibility-adjusted way to deliver what remains
  owed; it may not terminate or reduce the minimum. The audited specification
  defines constitutional interfaces and Book 1 renders them; transition,
  operations, infrastructure and service logistics remain Book 2 until the
  author rules otherwise.

### Expansion foundation — Map the whole society before adding rule families

Ordered by dependency: the roles matrix (579f8b1), the dependency map
(4beaa58), the scenario catalogue (ad58f74), and the generated
constitutional-closure/model-allocation audit (b633e44) have landed. The
audit supplies the assurance-allocation projection and truthfully emits
claim-scoped `block` or `bounded-unresolved` results; it does not pass Gate A.
The structural envelope and functional meanings landed at `dccea62`; the
`950d1a0` ownership clarification moves every remaining value and calibration
task solely to `book-2/TODO.md`. Content commit `ab814fc` adds the structural
reader projection and corrects Gate A's condition-one and condition-three
computations without weakening any later-gate claim blocker. The source-derived
power population landed through the staged family commits, with the final
formal-transition batch at `7e5b5f6`. The repository adversarial audit landed at `a8d6fd5`; schema v6 and
mechanical-closure candidate `2aeca61` remove the remaining human-act
dependency, and Gate A closed mechanically under protocol v4 at `405e480`.
Protocol v6's receipt migration records current Gate A state only in the
canonical ledger; this historical foundation summary does not override it.

### Expansion phase 2 — Specify the comprehensive constitution

“Comprehensive” here is bounded to Book 1's declared source version and scope at
Gate B; it does not claim Book 2 operations or feasibility.

- [ ] **Bring the expanded complete verifier back below five minutes.**
  - Current measurement, 2026-09-15 at `3e199dc1` against companion `979fe8b`:
    all 77,902 pins across 13,566 cases pass with complete contradiction checks
    and no findings in **824.25 seconds (13m44.25s)**, four workers, release
    binary prebuilt, peak RSS 21,626,012 KiB, 394% utilisation. The earlier
    275.04-second result covered only the pre-expansion 4,190-case inventory
    and is not a current timing.
  - **The profiling bullet is done.** `new-book-plans/nibli-performance-candidate.md`
    records the phase table, the case shapes, and each lead's measured size.
    Headline: queries 1,652 and fixtures 624 of about 3,143 cumulative
    worker-seconds; the target needs roughly 1,200.
  - **The remaining cost is engine-internal, not scheduling.** Preparation
    already builds each of the 127 variant bases exactly once. Batching already
    removes two thirds of the naive fixture volume, and hoisting what a batch's
    members share is worth nothing because that intersection is empty by
    construction. Caching compiled fixture statements is worth about 60
    worker-seconds.
  - **The largest sized lead is about 15%:** a first query in a fresh snapshot
    costs 37-48 ms whatever it asks, because `Clone for KnowledgeBaseInner`
    drops both the saturation and the witness closure, and every case then
    re-derives the same 256 witness activations. Carrying them across a
    snapshot, with a domain dependency cone so an insert outside it keeps the
    stamp, is a `nibli-reason` change with its own soundness burden — the
    companion's reasoner, oracle/differential and mutation checks, then a full
    book run.
  - Preserve every substantive expectation, actual-source ordinary case,
    explicit counterfactual, scoped/stateful sequence, shell precondition and
    complete contradiction check. Keep the fixed one-to-four-worker pool.
  - Reuse immutable preparation within the process, not earlier verdicts.
    Do not add hashes, receipts, freshness/report gates, reduced constitutions
    or skipped checks.
  - **Done when:** the complete current inventory passes in under five minutes
    with the release binary prebuilt, with measured timing and remaining
    resource limits reported honestly.

- [ ] **Protect knowledge, communication, culture, and the free social field.**
  - Cover learning and information access; expression, conscience, religion and
    non-belief; association; media/press plurality; academic, scientific and
    artistic freedom; language/accessibility; public information; sport, leisure,
    friendship, love, mutual aid, clubs and voluntary creation.
  - Secure the conditions and liberties for these activities without certifying
    official truth, taste, belief, creativity, relationship or personal fulfilment.
  - State residual freedom expressly: private/civic life remains free unless an
    evidenced rights or commons harm justifies a least-restrictive, reviewable rule.

- [ ] **Constrain records, surveillance, and automated power across every domain.**
  - Preserve the legacy assertion/withholding cases and distinguish them from
    the new justice interfaces; see the justice contract's "Legacy assertion
    boundary". Court consumers must reject conflicting single-valued identity
    bindings without treating the old multi-valued remedy/classification slots
    as exclusive. Authentication and observed execution remain external.
  - Extend the record contract to identity/status, health, care, education,
    workplace, housing, finance, policing and public-decision records.
  - Cover surveillance, biometrics, profiling and automated/AI-assisted decisions:
    authorised inputs, purpose limits, privacy, explanation, contestability,
    correction, human/independent review, non-use walls, retention/deletion and
    remedy.
  - Technology, storage and algorithms remain Book 2 operations or external
    evidence. A computed output is never a constitutional oracle.

### Expansion phase 3 — Make the architecture elegant without making it false

- [ ] **Name the real symmetries and necessary asymmetries.**
  - Real recursive interfaces:
    - right → duty → accessible delivery → breach → continuity → remedy → review
      → corrective control → monitored recurrence over a declared horizon;
    - power → lawful source/trigger → evidence → limit → public reason → independent
      review → appeal → correction/end;
    - harm → notice/voice → due process → least-coercive response → repair/release;
    - democratic choice → authenticated mandate → bounded implementation → public
      feedback → challenge → correction or peaceful replacement.
  - Necessary asymmetries:
    - recognition is optional, binary, non-ranked, and non-operative;
    - punishment is coercive and requires a higher proof threshold;
    - public power is presumptively reason-giving and auditable subject to
      narrow lawful confidentiality, while private life is presumptively private;
    - accessibility may require unequal resources to secure equal standing;
    - children, dependants and people needing support retain rights without symmetric
      capacity or contribution duties.

- [ ] **State the determination/action boundary accurately.**
  - Book 1 must identify who owes what, what counts as ordinary lawful function,
    delivery, failure, continuity, remedy, public accountability, and protected
    freedom from direction.
  - Book 2 must specify how people, institutions, funding, resources, technology and
    real-world operations make those duties happen and how the system degrades under
    scarcity or shock.
  - Do not describe an infinite chain of duties as action, an interface as capacity,
    a simulation as deployment, or an external premise as a constitutional fact.

- [ ] **Use a vector of protected conditions, never a total social score.**
  - No abundance in one domain compensates for torture, homelessness, exclusion,
    disenfranchisement, ecological destruction or loss of standing.
  - Use aggregate, privacy-preserving disparity, capacity and outcome measures for
    public learning; never convert them into individual worth or risk labels.
  - Record trade-offs and Pareto/conflict boundaries openly; do not collapse
    heterogeneous floors, liberties, commons and democratic choices into one number.

- [ ] **Define cross-domain priority, conflict, and physical-scarcity rules.**
  - Admit scarcity only from authenticated, contestable, resource/population-
    specific evidence after alternatives, reserves, substitution, coordination,
    replenishment, and mutual assistance. Budget choice, price exclusion,
    administrative delay, artificial withholding, monopoly, provider failure, and
    refusal to procure remain constitutional failures, not physical scarcity.
  - Preserve each constitutional minimum wherever usable supply permits. Never
    redefine a reduced ration as the minimum; record every shortfall as failure.
    Use an effective usable equal share where one exists; never divide a threshold
    resource into equally useless pieces. Otherwise mitigate through urgency,
    accessibility, imminent irreversible harm, continuity harm, and individualized
    resource-specific benefit after accommodation.
  - Forbid wealth, contribution, recognition, conviction, family status,
    disability stereotype, expected productivity, social usefulness, generalized
    lifespan, and political favour as priority keys. Use disclosed rotation or
    lottery only among materially equal claims.
  - Require public reasons, challenge, independent review, interim alternatives,
    replenishment, fresh reassessment evidence, a source-bound end, and repair.
    Scarcity creates no standing emergency power; missing authority ends the
    restrictive manager while the independent floor/continuity route survives.
  - Resolve other conflicts through typed rules rather than hidden priority:
    property versus floor/commons, expression versus evidenced harm, privacy versus
    public accountability, local choice versus portability, current claims versus
    future conditions, and emergency action versus non-derogable protections.
  - Any departure from the ratified scarcity ordering needs a new author ruling;
    implementation code may not choose it silently. **Book 2 handoff:** evidence
    collection/assurance, inventories, forecasts, quantities, reserves, production,
    capacity, queues, workflows, and empirical evaluation.

- [ ] **Test compositional closure and graceful degradation.**
  - For formal interfaces, prove within the declared model that individually safe
    domains remain safe when joined; test quantitative, dynamic, empirical,
    operational and lived compositions through their assigned assurance routes.
    Search for duty cycles, contradictory writers, duplicated final authority, veto
    by withheld evidence, remedy loops, unbounded delegation and cross-domain routes
    that recreate a forbidden score or status gate.
  - For normal operation and each compound shock, state what continues, what narrows,
    who may decide, who is protected first, what cannot be suspended, how review
    arrives, and how ordinary authority is restored.
  - A bounded safety proof may not claim that people, clocks, supplies, institutions
    or other states actually act. Assign every liveness premise to Book 2 or an
    external assurance owner.

- [ ] **Red-team incentives, capture, and strategic behavior across the composed
  society.**
  - Test capture, collusion, rent-seeking, bribery, patronage, regulatory arbitrage,
    strategic withholding/misreporting, Goodhart effects, adverse selection, moral
    hazard, free-riding, black markets and burden-shifting into another domain or
    jurisdiction.
  - For every mechanism state who benefits from gaming it, the information and
    coordination required, who bears the hidden cost, how it is detected/challenged,
    and whether the response creates a new veto, surveillance system or score.
  - Nibli may test legal walls; quantitative models, games/simulations, empirical
    evidence and Book 2 operations test behavior and scale. Do not assume either
    universal selfishness or universal altruism.

### Expansion phase 4 — Build a structurally navigable reader experience

**Tracker:** Reader's Map in the exempt opening note (67a520e);
reader-evidence execution withdrawn from the current Book 1 program (907ddd0).
The latter removes the post-pilot threshold and reader-session objectives from
this program without reporting reader evidence or a reader-result pass. Machine
accessibility work remains open; R6 remains optional and unbuilt, and FS-CLM-37
remains Unestablished/route-unbuilt. Gate C no longer depends on R6.

- [ ] **Build a source-bound reader-experience coverage ledger before rewriting.**
  - For every derived chapter and substantive Part V passage record: social domain
    and rule family; normal function and protective/corrective function; setting;
    person posture (chooses, creates, cares, works, associates, requests, receives,
    challenges, governs, or is acted upon); trajectory (works, contested, fails,
    continuity/remedy, unresolved); roles/life stages/access conditions; and exact
    rule/fact/pin or exempt-source basis. For any passage about a design, model,
    argument, evidence, or reader defect, also project its stable defect/claim/
    consequence IDs, defect disposition, response stage, assurance ceiling, and
    receipt or unresolved claim restriction.
  - Generate the report as a projection of the canonical full-society source and
    fail verification on an unclassified passage or a completed constitutional row
    with no reader-facing mapping. Counts may appear in the generated audit, never
    as hand-maintained prose claims.
  - **Done when:** every completed governed/provided domain has traceable ordinary-
    operation, credible failure/abuse or boundary, and—where claimed—end-to-end
    continuity/remedy cases in its assigned assurance route. Use Nibli pins for
    formalized legal claims; use reviewed specifications, quantitative/dynamic
    models, operational evidence or lived-experience methods for their assigned
    claims. A protected private/civic domain needs traceable non-interference and
    non-recording/non-compulsion limits plus recourse; ordinary-life illustration
    remains non-evidentiary under the author-ruled narrative register. A non-justice
    domain represented only through prison or custody fails.

- [ ] **Replace confession-as-ending with claim-scoped resolution receipts.**
  - Whenever the book identifies a design, model, argument, evidence, or reader
    defect; claims a former contradiction was repaired; or uses a narrated harm/
    hostile case as a witness of such a defect, end that thread in exactly one
    honest state: a verifiable claim-scoped resolution; an interface that remains
    operationally unresolved;
    an externally bounded or irreducible limitation with the public claim narrowed;
    or an open defect that blocks the affected claim. Naming the limitation and
    moving on is not a resolution.
  - The jargon-free reader receipt states what failed, why it failed, what changed
    or responded, what now follows, how the former attack was rerun, what still
    does not follow, and which dependency remains external or open. Use
    `eliminated-structurally`, `prevented`,
    `protected-consequence-contained`, and `remedied` language only for the
    narrower claim whose defect disposition, response stage, and assurance close it.
  - Link each reader receipt to the generated technical receipt with exact claim
    and defect IDs, source change, hostile witness/mutation, relevant pin/model/
    evidence or operational test, negative control, scope, residual, and gate.
    Parts I-V remain ordinary language; the public ledger supplies the derivable
    and reproducible detail. Do not widen sealed `method.md` merely to duplicate
    the ledger.
  - Fail the reader audit when prose says `resolved`, `fixed`, `prevents`,
    `contains`, `restores`, `ensures`, or an equivalent without an eligible
    defect disposition, response stage, assurance record, and receipt; when a
    receipt proves only detection or specification; when a repaired passage omits
    its residual boundary; or when a confessed limit
    has no defect record, owner, claim restriction, and gate consequence.
  - **Done when:** every claimed repair has a version-bound receipt and the
    generated navigation exposes its assigned verification route, exact resolution,
    and surviving boundary without giving disclosure any credit as closure. No
    external reader event is required.

- [ ] **Rebalance the pinned case portfolio without fictionalising it.**
  - **This item is the consumer, not the fix.** The 2026-08-08 narrative-register
    ruling measured the then-current deficit, and the delivery family has now
    repaired its formal-interface part. Recipient-side routes exist for food,
    non-carceral shelter, care, material security, and company; legacy learning
    remains separate; every route ships dormant. The supplied record still
    derives floor actualities only through confinement, every `home` and `family`
    entry names a convicted person, care and work remain thin, and the franchise
    and movement derive broadly while nothing reads either. State the rules,
    never cast or chapter counts — those move with the criterion, which is why
    the counted-claims gate exists:
    `for f in book-1/*.pins.nibli; do grep -qE "prisoner|dwell|severe|fit\(|building\(|defend" "$f" || echo "no confinement query: $f"; done`
  - **Dependency order.** The delivery and receipt precondition is satisfied;
    this portfolio rebalance is now runnable and remains open. The formal family
    does not supply cast receipts, ordinary-life cases, operation, actual arrival,
    or authority to fictionalise them. No public claim may describe the book as
    showing ordinary social life until this item supplies its own reviewed case
    coverage. The accusation-authorship gap also remains: no adverse ground
    relation carries an authorship place. See
    [`new-book-plans/book-1-narrative-register-decision.md`](new-book-plans/book-1-narrative-register-decision.md).
  - Preserve the prisoner as the hardest stress test, not the default inhabitant.
    Cover ordinary provision and care; family/dependency; learning and knowledge;
    work/property/exchange/commons; association, conscience and creation;
    voting/deliberation/local government; mobility/newcomer portability; civil
    dispute/repair; emergency continuity; and institutional correction.
  - For every public body show one lawful ordinary function and one accountability
    path. For every materially operative role/status, pin equal standing/floor or
    the exact lawful distinction.
  - No role may appear only as an object of intervention when the constitution gives
    it agency. Do not add decorative demographic labels or pretend a full Cartesian
    product is meaningful; use reviewed pairwise/high-consequence coverage.
  - **The agency clause is what decides any proposed second lead case, and it bites
    hardest on the most sympathetic one.** A candidate whose only available postures
    are *receives* and *is acted upon* cannot carry a protected private/civic domain,
    however sharp it is as a delivery test. An infant without a caregiver is the
    worked example: it is the strongest available stress case for delivery, receipt,
    and the standing root, and it fails the agency clause as a through-line, because
    the constitution gives the child voice, weight and decision-specific early
    authority that an infant cannot exercise. Pair a second case with the prisoner
    rather than substituting one lens for another — a book explaining every domain
    through one subject fails the same way whether the subject is a prisoner or an
    infant. See
    [`new-book-plans/book-1-thesis-framing-and-second-stress-case-brief.md`](new-book-plans/book-1-thesis-framing-and-second-stress-case-brief.md).
    The framing ruling has since landed: the infant is ratified as the paired
    second stress case with framing primacy in the exempt elements where the
    landed rulings permit, the through-line stays refused, and this item
    consumes the pairing, the primacy re-measurement, and the standing-root
    liveness case when it runs — see
    [`new-book-plans/book-1-thesis-framing-and-second-stress-case-decision.md`](new-book-plans/book-1-thesis-framing-and-second-stress-case-decision.md).

- [ ] **Use constructive, private/civic, democratic, and coercive chapter patterns,
  not one failure-first formula.**
  - Constructive provision: person seeks a floor → body/duty responds → accessible
    receipt/effect → challenge if needed → continuity/remedy → boundary.
  - Protected private/civic agency: person chooses, creates, associates, cares or
    cooperates without permission → non-interference and enabling conditions →
    narrow evidenced-harm rule if applicable → recourse against interference.
  - Democratic/co-operative agency: people deliberate and organise → authenticated,
    bounded collective choice → implementation → feedback, challenge and peaceful
    correction/replacement.
  - Coercive/protective rule: power is proposed → lawful trigger/evidence → limit
    → independent review/appeal → correction/end → boundary.
  - Show the rule working before or beside its strongest credible failure. Do not
    force an attack section where the rule family supports no such claim.
  - Prefer chapter-local reminders to long backward cross-references. Preserve the
    record-people's deliberately flat inner lives; do not invent biographies,
    emotions or composite citizens as evidence.

- [ ] **Finish machine-checkable accessible navigation and visual validation.**
  - The annotated contents, concise glossary, role/body and case indexes,
    domain-to-chapter map, and selected text-equivalent diagrams landed with
    the Reader's Map at `67a520e`.
  - Script 15 checks semantic source headings, local link targets, and
    deterministic HTML/EPUB generation. Bind the exact future preview
    snapshot's HTML, EPUB, and PDF artifacts and complete every mechanically
    testable heading, reading-order, extraction, keyboard, navigation, link,
    and text-alternative check required by its snapshot-specific gate.
  - Human screen-reader validation was withdrawn from the current program at
    `907ddd0` and is optional evidence, not a gate. Automated checks may
    warrant only properties of the artifacts and may not support an
    accessibility-for-users claim.
  - No meaning may depend only on colour, layout, vision, hearing, fine motor
    control, or specialist notation.
  - Readability formulas are diagnostic flags, not truth or pass/fail targets.
    Each visual must earn its cognitive and accessibility cost.

### Expansion phase 5 — Evidence, psychology, and repository red-team

- [ ] **Apply claim-type-specific scientific, statistical, formal, and normative
  discipline to every expansion.**
  - Empirical/descriptive claims need traceable data or primary sources, measurement
    definitions, representativeness limits and uncertainty. Causal claims also need
    an identification strategy, plausible alternatives and sensitivity analysis.
  - Predictive/feasibility claims need calibration, baselines, held-out or robustness
    tests, sensitivity to the reference envelope and explicit falsifiers. Formal
    claims need definitions, executable proof or derivation where applicable,
    countermodels/adversarial cases and a precise scope boundary.
  - Normative claims need stated values, alternatives, trade-offs, dissent and the
    lawful author/democratic decision owner; citation can inform but cannot prove a
    value choice. Psychological/lived-experience claims need ethical methods and may
    not be inferred from a formal or administrative record. If optional admissible
    evidence is unavailable, omit the positive claim or keep it explicitly
    Unestablished; no project gate waits for participants.
  - Pre-register acceptance criteria where feasible; publish code, data, provenance,
    sensitivity tests and null/negative results subject to privacy and licence. Use
    group-level outcomes for institutional repair, never individual worth.

- [ ] **Ground psychological claims without turning people into variables.**
  - Test for autonomy, voice, non-humiliation, relatedness, meaningful control,
    retaliation, status competition, coercive incentives, learned helplessness,
    trust, care burden and the effects of being watched or scored.
  - Where optional lived-experience evidence exists, separate ordinary and
    coercive institutions. Otherwise do not claim psychology, wellbeing, or
    compliance from a formal proof or service record.
  - Protect refusal and exit where compatible with others' rights; conditions may be
    secured, but belief, eating, learning, treatment, relationship and fulfilment
    may not be compelled or certified.

- [ ] **Run the source-derived multidisciplinary adversarial audit before completion.**
  - Encode the declared lenses for constitutional law, public administration,
    disability/accessibility, public health, care/life course, labour/economy,
    consumer/civil justice, policing/prison, media/science/culture/pluralism,
    local/migration/collective governance, defence/external affairs,
    infrastructure, ecology, data/AI governance, and quantitative modelling as
    source-bound criteria and watched-failing mutations.
  - The repository audit must identify omitted domains, unowned dependencies,
    hidden liveness assumptions, private-power blind spots, impossible-operation
    overclaims, totalising rules, and narrative distortions. Every material
    finding creates or joins a stable defect ID with severity, consequence,
    owner, closure condition, affected claims, applicable gates, and a public
    claim limitation. Critical unresolved findings block only gates whose exact
    permitted claim they affect; disclosure is not closure.
  - External multidisciplinary or lived-experience submissions remain welcome
    optional evidence. If received, give them traceable public dispositions, but
    no recruitment, panel, submission, or response is required for completion.


### Explicitly rejected expansion proposals

- Restoring fixed counts in reader-facing book prose; counts are intentionally gated
  and have historically rotted.
- Treating `reward` as punishment's inverse, granting standing for contribution, or
  allowing recognition to buy material security, authority, or voting power.
- Calling the current record a self-healing closed circle, a mathematical group, a
  Bayesian model, or proof that random sampling cannot be captured.
- Adding provider-authored delivery facts as evidence that the floor was met.
- Adding formal-language explanations, histories, or implementation detail throughout
  derived chapters instead of using the existing Part V/method boundary.
- Treating a transition roadmap, full operational economy, record-storage technology,
  or implementation logistics as Book 1 prose unless the author replaces the current
  seam. The destination may specify complete functional interfaces without pretending
  to staff, fund, build or deploy them.
- Treating exhaustiveness as permission to regulate every harmless private practice.
  Unauthorised public power fails closed; unclassified harmless private life defaults
  to freedom.
- Asking Nibli, a spreadsheet, a simulation, an empirical registry, or a reader study
  to prove the domains assigned to the other assurance methods.
- Declaring narrative balance from word counts, sentiment, demographic decoration or
  a fixed prisoner/non-prisoner quota without reviewed context and reader evidence.

### Expansion completion standard — cumulative gates, not one finish line

These gates are cumulative but not interchangeable. The author-ratified
2026-08-07 boundary fixes the two-book seam, C-then-E publication sequence, and
versioned closure; `new-book-plans/full-society-boundary-decision.md` controls.
Gates D and E are project-level reference gates whose executable work lives only
in `book-2/TODO.md` after Book 1 — First Edition ships at Gate C. A later formal,
operational, or reader test cannot substitute for an earlier missing condition.

#### Gate A — Scope and assurance foundation

- the canonical source covers every material domain, role, power, dependency,
  scenario and claim, or visibly classifies it out with reasons;
- all projections regenerate from that source; unresolved items carry severity,
  consequence, owner and closure condition, and critical gaps block the affected
  claim;
- the versioned reference envelope, assurance allocation, stopping rule and
  decision briefs are reviewable; and
- a current-source repository adversarial audit covers the declared criteria,
  exact checker controls, command chain, and every Gate-A-applicable defect;
- every material known defect has a stable ID and claim/consequence/scope row
  with one current defect disposition and response stage, the required evidence
  state or an explicit evidence gap and closure condition, and a generated
  resolution status bounded by its claim-assurance ceiling; history remains
  versioned; and
- no critical unresolved defect affecting the Gate A permitted claim is hidden
  by classification, assignment, disclosure, or a stopping-rule decision.

**Artifact and permitted claim:** the map and test program may be public, but no
book preview, release candidate, or edition may publish. The project has a
versioned, reviewable map and test program; it has not yet described or operated
a complete society.

#### Gate B — Expanded Book 1 constitutional/social destination

- every applicable right, liberty, public function/power, expressly bound private
  power, record and commons condition has a complete contract card, owner,
  adversarial case, counterfactual and accurate reader account;
- every floor has unconditional accessible delivery, recipient-side access/receipt
  evidence, continuity, remedy and corrective-control interfaces without pretending
  Book 1 supplies capacity;
- every public body performs an ordinary function and is independently checked; the
  democratic corridor and residual private/civic free field are explicit; and
- domain journeys, collisions and shocks establish the claimed constitutional
  invariants, lawful narrowing, challenge, restoration and model boundaries, with no
  critical constitutional, equality, safety or hidden-power gap.
- every claim that a constitutional failure is resolved joins to a receipt whose
  defect disposition, response stage, posture, route, and evidence can close that
  claim. `detected` and `interface-specified` never count as resolution;
  `externally-bounded-assumption` may remain only where the permitted claim is
  explicitly conditioned on the named premise. `irreducible-limitation` and
  `open-defect` may remain only where the permitted claim is narrowed so it does
  not assert the unresolved consequence; any critical defect still applicable to
  the conditioned or narrowed claim blocks. Book 1 may not count
  `operationally-assured-in-envelope` or remedy liveness as its own achievement.

**Artifact and permitted claim:** immutable Book 1 — First Edition previews may
publish under P1 after Gate B and their snapshot-specific gates pass. A preview
may say that it describes a comprehensive, versioned constitutional and social
destination for its declared scope. It may not claim reader suitability,
staffing, resources, feasibility, deployment, outside liveness, or an operational
society.

#### Gate C — Book 1 public-edition readiness

- the full verifier, generated closure/reader projections, adversarial cases,
  deterministic artifact checks, and mechanically testable accessibility checks
  pass for the exact release candidate;
- source, ordered inputs, artifact hashes, navigation, internal links, reading
  order, text alternatives, licence/provenance, and the Gate C closure record
  bind the exact HTML, EPUB, PDF, and print identities;
- each governed/provided domain has applicable ordinary-success, failure, and
  recovery coverage; each protected private/civic domain has agency,
  non-interference, evidenced-harm and recourse coverage; and no non-carceral
  domain is explained only through prison, punishment, or institutional failure;
- every claimed repair is traceable to a version-bound technical and reader-facing
  resolution receipt, with structural elimination, prevention, containment,
  remedy, external bounds, irreducible limits, and open defects kept distinct; and
- no reader-comprehension, reader-suitability, lived-effect, or actual-user
  accessibility claim is made without optional claim-appropriate evidence.

**Artifact and permitted claim:** publish Book 1 — First Edition, its assembled
digital artifacts, and its first POD atomically under one provenance and Gate C
closure record. The edition may claim exact source binding, artifact integrity,
navigation, internal consistency, and mechanical accessibility. It may not claim
suitability for a tested audience, accessibility for actual users, staffing,
resources, feasibility, deployment, outside liveness, or an operational society.
If any matching source, artifact, POD identity, provenance, or Gate C record
fails, the public object remains a preview and Book 2 does not activate. No
external reviewer or reader event is required.

#### Gate D — Book 2 operational model

- every Book 1 interface has a costed, staffed and accountable operator/model, a
  visible external assumption, or “Book 2 operation not applicable” where the
  protected condition is non-operation/non-interference; any recourse operation is
  mapped separately, and the reference envelope is calibrated and versioned;
- all applicable operational domains in the canonical generated set—including
  material/care, economy, equality/life course, democracy/integrity/statistics,
  justice/safety/defence/external relations, ecology, knowledge/free life, records/
  technology, transition, gameability and reader experience—meet their
  pre-registered adequacy, accessibility/equity, continuity, resilience,
  sustainability and fiscal/resource-feasibility thresholds in ordinary and
  declared shock cases;
- the Book 2 reader-facing structural view covers ordinary agency,
  maintenance, failure/degradation, and recovery and passes its deterministic
  source-binding, navigation, consistency, and artifact checks; any comprehension
  or lived-operation claim remains optional and separately evidenced;
- models publish code/data, uncertainty, sensitivity, negative results, capacity
  and failure boundaries; simulations and pilots state external-validity limits; and
- any unresolved critical floor, equality, safety, capacity, feasibility,
  hidden-power or cross-domain dependency gap blocks the affected operational claim.
  A non-critical residual needs severity, consequence, owner, closure condition and
  an explicit public-claim limitation.
- neither `interface-specified` nor `implemented-in-assigned-route` is
  operational closure. `operationally-assured-in-envelope` requires an Evidenced
  claim through operational assurance: a staffed, costed, accountable end-to-end
  route exercised in ordinary, failure, continuity and recovery cases in the
  named test/pilot envelope. It supports only Gate D's reproducible operational-
  design claim, never deployment or generalisation; any narrower exercise
  generates only the narrower resolution status.

**Artifact and permitted claim:** only immutable Book 2 — First Edition previews
or release candidates may publish. Book 2 may say that it supplies a reproducible
operational design within the named envelope. It may not publish Book 2 — First
Edition or claim deployment, generalisation beyond the envelope, or an integrated
functional society.

#### Gate E — Integrated two-book full-society claim

- every guarantee and democratic choice crosswalks to its operational path and
  back; each private-freedom boundary crosswalks either to enabling/recourse
  operations or to an explicit non-operation/non-interference disposition;
  constitutional rules survive operational scarcity and operations respect floors,
  liberties, equality, democracy, privacy and commons;
- cross-domain journeys and compound shocks pass their declared safety, continuity,
  recovery and feasibility gates with no hidden critical assumption;
- the generated cross-book reader-facing projection exposes ordinary life,
  agency, maintenance, constraints, failure, and recovery across both books
  without reducing Book 2 to a crisis/cost manual; and
- reproducible structural, model, provenance, assurance, and negative-control
  checks pass for the exact pair, followed by a checker-derived closure record
  bound to an immutable verified candidate. No human act is required. Optional
  external evidence retains its own provenance and limits but is not a Gate E
  dependency.
- the paired releases generate one compatible cross-book defect projection:
  every Book 1 interface row has generated `resolved-for-claim` supported by an
  eligible defect disposition, claim posture, route, evidence, and, where liveness
  is involved, `operationally-assured-in-envelope`; is explicitly non-applicable
  under `scope_disposition`; or remains under a named non-resolution defect
  disposition with an exact public-claim condition or narrowing. Any
  critical residual still applicable to that conditioned or narrowed claim blocks.
  No defect disappears at the seam, and every integrated resolution receipt names
  the exact compatible editions, envelope, assurance route, and evidence versions.

**Artifact and permitted claim:** atomically publish Book 2 — First Edition and
an immutable integrated release manifest pairing the exact compatible Book 1 and
Book 2 editions, artifact hashes, canonical-source version, reference-envelope
version, assurance and audit records, external assumptions, and residual limits.
If the pairing, integrated checks, or manifest fails, Book 2 remains a preview or
release candidate. Only Gate E permits the bounded claim that the exact paired
editions provide a reproducible integrated constitutional and operational design
for the declared
reference envelope. The claim remains versioned, falsifiable and open to the
stopping rule; it never means human-reviewed correctness, successful deployment,
real-world functioning, timeless completeness, prescription of every harmless
private life, or control of every external condition.

---

## book-1 — remaining work

- **Parts I–IV second expansion wave — DONE 2026-08-03, with its stop.** Ruled
  content-governs (the ~38,000 target retired; CLAUDE.md's length entry carries the
  ruling) and swept all fourteen chapters. Outcome, measured the same day: chapters
  1–14 went **29,440 → 35,071** words across 24 commits, against 10,722 non-derived —
  majority-derived holds at roughly three quarters. The retired target was approached
  from below by material rather than aimed at, which is the only way this wave would
  have accepted reaching it.
  - **The stop-map, checked 2026-08-03.** Thirteen chapters took new material. **13 is
    dry and was declined on the merits, not skipped**: its two candidate sharpenings —
    that the time words are refused at the door rather than merely unused, and that
    release has the shape it has because derivation only ever adds — were read against
    the chapter and judged already covered by its existing passage, so writing them
    would have been padding against the wave's own rule. Start a third wave elsewhere.
  - **Four fidelity corrections landed first**, because wrong prose outranks untold
    prose: chapter 1 listed "a date" among what the record holds; chapter 5's bolded
    headline read "judge your family" over a rule reading parenthood alone; chapter 8
    claimed a mark was "the only form the design allows"; chapter 14 claimed the audit
    "cannot be gamed from below" when it can be starved. A fifth was found by
    measurement mid-wave and folded into chapter 6.
  - **The wave corrected the constitution, not only the book.** Chapter 8's planned
    passage rested on a margin note claiming that threading the universal into a floor
    line fakes delivery for everyone "while every entitlement pin stays green".
    Measured before writing: it fabricates no actuality and instead deletes the
    entitlement — the opposite failure, and the guard is the entitlement pins, not the
    actuality ones. Note corrected, all six fixtures regenerated. Nothing checked that
    comment, which is why it rotted.
  - **The adversarial pass earned its place and then some.** Two checkers over the
    whole wave diff returned twenty-two findings, and seven were factual errors in new
    prose — most importantly that a voiding does not take the pen (the credential
    rules read the carried mark, never the current voiding, which is the design's own
    disclosed exploit), that a recall does not strip a seat, and that the record does
    hold a word for intent. Three correction commits landed before this close. **Do
    not skip this pass on a future wave**: per-commit gates cannot see cross-chapter
    drift, and every one of the seven had passed its own chapter's full suite.
  - **Two commit bodies carry word counts off by a little** (chapter 3 says 2,652 for
    2,651; the method addendum says 4,255 for 4,270) because the count was composed
    before the final edit. The figures above are the authoritative re-measurement;
    pushed bodies were not rewritten.
  - **Process notes worth keeping.** Two gates failed for a reason that was not the
    prose — editing another chapter's files while a full run was in flight breaks the
    cross-file pin reconciliation, so gate strictly serially. And the two-second
    `--quick` prose pre-check before each five-minute full run caught three violations
    in this wave at a fortieth of the cost; the counted-claims gate stopped "one thing"
    twice more, which is now four times across two waves.

---

## Reach — delivery and edition boundary ruled 2026-08-04

The ratified policy is E2 + P1 + D2, refined on 2026-08-07 by the two-book,
C-then-E, versioned-closure boundary. The current-T0 baseline remains public
source and git history but receives no canonical serialization, assembled
edition, edition tag, or print identity. “Completed expansion” means cumulative
Gate C completion; Gate C publishes Book 1 — First Edition, its assembled digital
artifacts, and its first Book 1 POD without making an operational or integrated
full-society claim.

Building in public survives through P1: coherent expansion milestones may be
published as immutable, tagged previews only after Gate B and their
snapshot-specific gates. They are design snapshots with provisional order, not
editions or final serialization. Constitution and spine freeze may create a
private release candidate, but every public pre-Gate-C object remains a preview.
The controlling publication mechanics are in
[`new-book-plans/book-1-edition-boundary-decision.md`](new-book-plans/book-1-edition-boundary-decision.md);
the gate, claim, and stopping contract is
[`new-book-plans/full-society-boundary-decision.md`](new-book-plans/full-society-boundary-decision.md).

The publication objective is honest public conversation rather than revenue.
`Living` names the public project, source, audit record, and future versions; it
never authorises silent mutation of a released object. The working source may
continue to evolve under its visible gate status, while every public preview and
edition remains immutable, permanently citable, reproducible, and linked to its
superseding versions. An external publisher may steward a particular edition but
may not become the sole custodian of the book or of its future development.

- **Implement the ratified E2 + P1 + D2 edition contract.**
  - Do not create a promoted artifact from the current-T0 baseline. Before the
    first expansion preview, audit the root README and opening note for
    unregistered-standing overclaims, and have the author replace the final-page
    publication-order promise with the single permitted, scope-only Book 2
    pointer.
  - Create a machine-readable ordered-input manifest and reproducible assembled
    reader artifacts. A repository archive is not a book artifact: it also
    contains legacy manuscripts, reviews, plans, and verification files.
  - Record an immutable namespaced tag; full book-repository and nibli commit
    SHAs, each verified from a clean tree; the full verification transcript/date;
    registry snapshot; known limits; licences; and artifact hashes. Never cite
    `main`, move a tag, or replace an asset in place.
  - Publish coherent milestones, when useful, only after Gate B and their
    snapshot-specific gates, as immutable tags such as
    `book-1-v1.0.0-preview.1`; preserve superseded previews and mark their order
    provisional. After the expansion freezes, use an immutable candidate for
    deterministic release checks. Publish `book-1-v1.0.0`, the assembled digital
    capstone, and matching POD atomically only after cumulative Gate C and an
    explicit author-ratified closure record pass.
  - Give every version a permanent URL; only `latest` navigation may move. New
    content creates a new version, and withdrawal means visibly disrecommended,
    not silently erased.

- **Implement the living, evaluable, evolvable Creative Commons conversation
  layer without weakening immutable releases.**
  - Before implementation begins, record the author-ratified publication purpose
    and publisher-custody boundary in `CLAUDE.md`; this tracker specifies the work
    but is not the permanent home for a settled ruling.
  - Until Gate B, describe the public repository and history as a living
    construction record, never a living edition. Public discussion may begin
    against commit-addressed construction records before Gate B; no pre-Gate-B
    object may be presented as a preview, edition, or Gate B claim. After Gate B,
    immutable previews may expose the conversation; after Gate C, the living
    project may point to the immutable First Edition and later immutable versions.
    Every page must state which object it is, its source version, gate status, and
    whether it is citable.
  - For every substantive public change, publish a change record linking affected
    claim IDs, defect IDs, `defect_disposition` and `response_stage` transitions,
    resolution receipts, assurance evidence, audit cut-off, verification status,
    and superseded version. The
    working source may change; a released artifact, manifest, tag, or print
    interior never changes in place. An ISBN/DOI identifies one exact release;
    its metadata may add correction or supersession links but may not repoint the
    identifier to changed content.
    Every substantive successor release reruns its applicable cumulative gates;
    no prior version's green result is inherited.
  - Put the evaluable surface beside the reading surface: exact source, reproducible
    artifacts, verifier instructions, claim-assurance, defect-disposition, and
    response-stage ledgers, resolution receipts, known limits, external assumptions,
    audit record and cut-off, and release manifest. A reader should not need
    repository archaeology to learn what is established, unresolved, superseded, or
    outside the model.
  - Provide a public, accessible submission channel with stable finding IDs, a code
    of conduct, moderation and safety/privacy rules, and a visible submission cut-off.
    Give each material criticism a traceable response and proposal disposition;
    create or join a defect record when warranted. A comment, vote, credential, or
    volume of agreement does not automatically become evidence, a rule, or a veto.
  - Keep new book prose under `CC-BY-4.0`; keep code, registry data, snapshots, and
    third-party material under the repository's actual licence map. Incorporate a
    contribution only with explicit compatible permission and attribution; otherwise
    link or quote within lawful limits without silently relicensing the contributor.
    Add no DRM or downstream term that restricts exercise of the Creative Commons
    grant.
  - Measure the conversation by whether substantive objections receive inspectable
    dispositions and cause verified repair, explicit claim narrowing, or a visible
    open defect, rather than by sales, praise, follower counts, or agreement.
  - **Done when:** the public surface exposes an immutable version, reproducible
    checks, successor comparison, optional finding submission, and traceable public
    dispositions for any received findings. No submission or outside reader is
    required for release.

- **Seek a publishing steward only on terms that preserve the living public book.**
  - After a Gate B preview exists, prepare a compact proposal containing the thesis,
    intended readers, representative chapters, method/evaluation surface, immutable
    release model, Creative Commons terms, and the kind of editorial, accessibility,
    library/discovery, and print-quality help sought. Revenue is not a selection
    criterion.
  - Evaluate mission-aligned open-access, university, independent, and publishing-
    service routes against one public matrix. A publisher may edit, review, design,
    distribute, archive, and sell a named immutable edition; it is not the canonical
    mutable source and receives no authority over defect or assurance verdicts.
  - Contract red lines: the author retains copyright and future-revision/publication
    rights; book prose remains `CC-BY-4.0`; the canonical source, free digital
    edition, verifier, registry, audit record, and release archive stay public.
    Grant no exclusive right in the CC-BY prose. Any edition-specific exclusivity
    is limited to publisher-created assets or services and may not restrict the
    existing Creative Commons grant, public source, or future editions. Permit no
    DRM, silent replacement, exclusive canonical URL, suppression of prior versions,
    or control over future editions; require every publisher-issued copy to name its
    exact source, licence, errata, and supersession path. Run the checked-in
    contract red-line checklist and record the author's decision before signing;
    professional advice is optional and not a project gate.
  - An external publisher is optional and is not Gate C. If no steward accepts these
    terms, publish the Gate C digital artifacts and POD directly under the same
    provenance, accessibility, licence, and immutability contract.
  - **Done when:** the comparison records accept/reject reasons against the red lines
    and either a compliant edition-specific agreement is signed or the self-
    publication route is release-ready without weakening the public conversation.

- **The site.** A dedicated domain — **registering it is the author's own task**
  — plain, built from the Markdown that already exists; immutable preview
  snapshots during expansion, then final chapters in computed order. Link the
  exact release/source, claim-assurance, defect-disposition, and response-stage
  ledgers, resolution receipts, review channel, and one-command suite from the
  reading surface.
  Platforms syndicate *from* it: CC-BY means they will copy regardless, so the
  canonical home must name itself.
- **The launch essay. [AUTHOR-GATED]** A standalone distillation for someone who will
  never read the book, carrying the thesis and the honest second half in miniature. *The
  Furnished Prison* is the standing headline candidate. First-person territory: the voice
  protocol applies — sessions may draft candidates, and only exact-version author
  approval makes prose canonical.
- **The method paper.** JURIX/ICAIL/formal-methods-for-law genre: the derivation gate,
  the pin suite, the counterfactual classes, the defect markers — the methodology made
  citable. Coordinate with the method part rather than duplicating it; the paper cites
  the book, the book does not depend on the paper.
- **Make run-it-yourself true as a launch claim.** `verify.sh` and its `--only`
  mode are the core artifact, but the script currently defaults to an adjacent,
  mutable nibli checkout. Supply and test a pinned two-checkout or bootstrap path
  from clean inputs, and publish the exact engine commit; only then say “clone,
  one command, the pins pass.”
- **Print-on-demand for the Gate C-complete expansion only (D2).** A priced,
  well-made
  physical edition of a free text. Quality is the lever and revenue a side
  effect: the typography is canonical because it is first and good, never
  because it is exclusive. Generate it only from the final tagged First
  Edition; put the edition, source commit, licence, print-file identity, and
  errata URL inside every copy, and mint a new version for any changed interior.

---

## Data

The registry (`registry/claims.json`, CC0), its staleness gate and the first fetcher
exist and run inside `verify.sh`; see `registry/README.md`. What remains:

- **The rendering and Part V traceability step — build it beside the first prose that
  cites a registry id.**
  Nothing in book-1's derived chapters may carry a number (the counted-claims gate), so
  value-injection waited for the empirical writing it serves — and Part V now exists:
  its frame and capture joint carry registry-backed numbers as hand-written prose,
  checked against the registry by the landing verification. Build the step beside
  those figures, or rule that Part V's handful stays hand-checked. Do not build past
  its consumers. More fetchers (WHO GHO, OWID, FAOSTAT…) land the
  same way — as entries need them.
  - This task owns point-of-claim traceability: every empirical statement needs its
    registry ID and source; causal language must match the evidence, uncertainty, and
    instrument sensitivity the record supports.

- **Re-cite the ported registry entries against published versions.** The port
  (`dd25b49`) honestly stamped `retrieved: 2026-07` — book.md's own last verification —
  on the legacy entries without re-verifying them, and most of the registry still
  carries that stamp. The sweep the old plan deferred to "as each reference is ported"
  is now due, since the porting is done: work through the pinned entries, check each
  against its source's current published version (the Muralidharan REStat move is the
  model — a working paper that became a journal article), update the entry and its
  `retrieved` date. The Kenya UBI entry carries its own warning: it must not reach
  Part V as a working paper.

- **Add Bregman's 15-hour workweek figure to the registry** when Part V or book-2 first
  cites his proposal — the one claim the research-brief corrections found no error in but
  no registry entry for either.

- **The V-Dem re-derivation is DONE (2026-08-03) — Part V's worked example has its
  numbers and a better third act.** `registry/fetch/vdem_happiness.py` derives
  everything from OWID's CC BY series (V-Dem polyarchy + RoW, WHR ladder, WB GDP);
  three registry entries + snapshot landed; working record at
  `new-book-plans/vdem-rederivation.md`. Robust across instruments: the income-control
  narrowing (partial r ≈ 0.20, was 0.195) and the step pattern (+0.02/+0.59/+1.09 —
  bottom step buys nothing). Changed: the floor claim is **instrument-fragile**, not
  cleanly refuted — it survives the income control narrowed on polyarchy (p = 0.032)
  and dies-or-marginal on the alternative index over the identical sample — so the
  worked example's third act becomes "a verdict that tracks the instrument is not
  citable", which is a stronger methods lesson than the refutation it replaces. Part V's
  frame now runs this arc (landed 2026-08-03); the EIU-era sub-bullets below remain
  the historical working reference. FLAG for the author: `democracy_vs_happiness_144.csv`
  in the repo root (CC0 under the root LICENSE, committed pre-ruling) carries EIU
  index values — same grounds as the registry ruling, worth a look.
  - **Do NOT use the floor claim.** Its headline finding — "democracy behaves like a
    floor on subjective wellbeing", from regressing |residual| on democracy score,
    p = 0.0004, which is exactly how convincing it looks — is the one claim it never
    controls for income, and
    **it does not survive**: adding log GDP gives democracy b = −0.0196, t = −0.91,
    **p = 0.37**, while log GDP itself is b = −0.336, t = −2.53, p = 0.011. Within
    income tertiles the dispersion goes the *wrong* way for the democracy story. The
    compression is income, misattributed. This is precisely the claim book-1 would most
    want to be true — a floor effect, in a book about floors — which is exactly why it
    must not be used. An economist kills it in one regression.
  - **Use the income result instead: it supports the book's real thesis better.** What
    compresses the dispersion of human wellbeing across countries is material provision,
    not the franchise. A book whose floor is material-and-personal guarantees, and which
    deliberately demoted the vote *off* the floor to a rule, just got empirical support
    for exactly that ordering.
  - **Use the step sizes.** Authoritarian → Hybrid buys **+0.16** — nothing. Hybrid →
    Flawed +0.73. Flawed → Full +1.01. Partial democratisation does approximately
    nothing; the gain is concentrated at the top of the scale.
  - Still to do from the ruling: record `demo-happy.txt` in the registry as "prior
    analysis, independently re-derived", with the CSV's provenance pinned: WHR 2025 (2022–2024
    average) merged with EIU 2025, 144 countries matched from EIU's 166 and WHR's 147.

- **Publish the registry with the book, not just in the repo.** The formalism stays
  invisible, so what the reader verifies is the data — which only works if the registry is
  reachable from the page they are reading. Front matter names it and gives the URL, every
  figure in the prose resolves to a registry id, and the registry ships CC0. This is the
  thing that earns the trust and the honest substitute for showing the constitution.

---

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

## book-2

book-2 has its own tracker: `book-2/TODO.md` — unordered until its chapters are
decided, seeded from the hold list, adoption reviews, and the 2026-08-05
full-society operational completion contract. The discipline is unchanged:
**do not work book-2 items until Book 1 — First Edition actually ships at Gate
C**; collect there, rule here.
Every Book 1 domain card must nevertheless name its Book 2 operator/evidence owner
or an explicit external assumption so the seam cannot hide an unfunded,
unstaffed, unmeasured or physically impossible promise.

---

## Standing facts and methods — not tasks, and not history

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
  it covers the duty as well as the eight rights. `entitled(every person, event { P() })`
  compiles to a rule with `person` in the body, so `P` sits downstream of `prisoner`; any
  later rule taking `~P` into that cone is an unstratifiable negative cycle and is
  refused. The floor is protected **because** it is reachable — at stratum 0 there would
  be no cycle to close and no protection at all. Where it stops is pinned in
  `08-what-you-are-owed.pins.nibli`: `~P -> false`, `~P -> lose(Points, ·)` and positive
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
  F='new-book-plans/3-spine.md'
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
  anywhere in the enacted lines and `verify.sh`'s digit ban keeps it that way, so
  weighting cannot be written. **BL4** and **BL5** are pod-and-tech-stack material and
  belong to book-2. **BL1** ported in narrowed form and is in chapter 1's closing
  section: the floor is unconditional *above* `person($x)`, and `person` is a roster of
  written facts with two producing rules, so personhood **is** an enrolment. Do not
  restate the unnarrowed BL1 in book-1; it would be false the way BL2 is false in
  `book.md`.

- **Article 9 does not semantically entrench the evidence vocabulary.**
  The source audit applies `permanent(Art_Evidence).` and still executes a direct
  vocabulary widening: `rich(Adam)` becomes writable. Article 9's general rule marks
  dead a docketed proposal that DECLARES a registered target and does nothing to the
  source itself. `adjust` is self-declared, so a targetless proposal and one naming a
  harmless target both receive the otherwise-derived law label.
  In the reverse direction, `false(Amend_Floor)` remains true and `become` remains false
  while an independently constructed source deletion removes the food entitlement and
  makes the adverse rule loadable.
  The executable source audit goes further: a concealed grammar change can remove the
  food entitlement while the separate anti-imprisonment firewall survives, and direct
  `admits("rich")` widening bypasses Article 9 entirely. Article 0a therefore makes
  widening *source-visible*, not approved, authenticated, or semantically entrenched.
  Nothing reads `become`, and the audit manually applies its candidates; it proves no
  proposal-to-source transition. A future entrenchment design must bind an exact change,
  independent effect review, compatibility verdict, and effective version.

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
  passage (`13-the-one-thing-taken.md`) — *"A society whose only working provision runs
  through its prisons has not built a floor; it has built a prison that happens to be
  furnished."* Primary candidate since the reach ruling
  (2026-08-02): the launch-essay headline; the Part-title and back-cover uses stay
  listed behind it. The title work is done; this is the one asset from it
  that outlived the decision.
