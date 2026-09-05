<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# TODO — audited specification and Book 1 projection

**This tracker covers the principal formally audited constitutional
specification and its Book 1 reader projection.** Book 2 has its own inactive
tracker. This file is strictly future-facing: a bullet is deleted only after its
frozen candidate receives a full receipt, its repository audit and closure land,
and the tracker deletion passes the exact successor gate. Drafted work is not
landed work. History belongs in git.

The repo is producing one formally audited specification, two controlled books,
and a clean legacy deletion:

- **The formally audited specification is the principal product.** Its exact
  version includes `new-book-plans/constitution.nibli`, the reviewed decisions
  and canonical contracts that define its scope, executable pins and
  counterfactuals, generated projections, and the receipt-bound audit and closure
  identifying one verified candidate. No individual report or partial green
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

Multiple slices may share one authoritative run only while uncommitted and only
when they form one rule family or assurance concern. Use quick, focused,
fingerprint, and governed refresh/check modes while the candidate changes. When
all slices agree, freeze and fully stage the whole batch, run one full
`--emit-receipt`, commit that exact candidate, and land its immediate audit and
closure successors. Only the final tracker successor deletes the parent item.
Never create semantic WIP commits for later receipt sharing, combine unrelated
items merely to amortise the gate, or call a focused result an audit.

Before freeze, move every durable decision from `tmp.txt` into its governed
source and remove the scratch file. Receipt emission rejects a non-ignored
untracked file; scratch memory cannot ride inside an audited candidate.

Verification speed is a maintained property of the assurance system. Optimise
it only through deterministic, watched, fail-closed mechanisms such as bounded
parallel scheduling, compilation reuse, phase timing, and immutable parse/source
caches. Do not skip semantic suites or retain semantic/mutant verdicts. Every
status update and ETA separates active drafting/review, full-gate runtime, and
audit/closure administration.

New verifier coverage enters the authoritative gate only for a named material
defect or evidence gap and with a watched failing control. Performance machinery
likewise needs a measured bottleneck and equivalence controls; neither assurance
growth nor optimisation proceeds by intuition alone.

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
The substantive-equality and anti-subordination baseline is likewise author-
ratified but unimplemented. Its controlling contract is
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

- [ ] **Authorize qualifications, licensing, compensation, and positive incentives
  without ranking people.**
  - Permit mandatory licensing only for evidenced serious safety, fiduciary, or
    core-public-function risk; otherwise prefer certification. Require relevant
    standards, accessible alternative proof, portability, independent challenge,
    expiry/renewal, anti-cartel safeguards, and a fresh temporal contract.
  - Implement wages, profits, savings, returns, grants, prizes, subsidies, and
    bounded incentives through distinct legal relations. No rule may read
    `reward`, `false`, `lose`, or missing recognition to alter compensation,
    property, benefits, pensions, insurance, authority, or political weight.
  - Test valid and pretextual licensing, credential-cartel capture, accessible
    alternative proof, inherited advantage, metric gaming, and proportionate
    fraud restitution. Recognition remains binary, arity-one, non-ranked, and
    unread.

- [ ] **Constrain concentrated private and hybrid power.**
  - Preserve equality's named direct private reach without a dominance finding.
    Separately create an independent, contestable, function-specific public-scale
    trigger for essential, dominant, delegated, gatekeeping, network, dependency,
    lock-in, information-asymmetry, or no-meaningful-exit power.
  - Bind only the affected public-facing, essential, delegated, gatekeeping, or
    systemically controlling function. Intimate dependency alone does not create
    public-like transparency duties; coercion, captivity, violence, and obstruction
    of exit instead route through justice and public protection.
  - Calibrate access, continuity, reasons, transparency, portability,
    interoperability, audit, challenge, and remedy. Give findings a source, scope,
    tier/jurisdiction, contestability, temporal status, expiry consequence, and
    conflict rule; regional/local findings cover their jurisdiction and the common
    tier owns cross-regional cases and minima.
  - Test each named actor with and without the trigger, an equality-bound actor
    without it, an expired finding, regional and cross-regional gatekeepers,
    relabeling, confidentiality, capture, and structural remedies that preserve
    workers, users, savers, and essential services.

- [ ] **Protect democratic and administrative integrity.**
  - Cover parties and opposition, districting, campaign finance, political
    advertising, lobbying, gifts, procurement, conflicts of interest, revolving
    doors, corruption, and coordinated information manipulation.
  - Define permitted writers and money/influence records; proportionate disclosure
    and privacy; and campaign, procurement, ethics, and anti-corruption oversight,
    challenge, correction, remedy, disqualification, and enforcement through the
    landed appointment and anti-capture interfaces.
  - Test shell actors, third parties, media/platform concentration, selective
    enforcement, audit starvation, and manufactured withholding against those
    controls.

- [ ] **Constitute official statistics and planning information without building a
  population score.**
  - Authorize censuses, representative sampling, administrative statistics and
    public planning data only with necessity, purpose limits, minimization,
    accessibility, privacy, correction, retention/deletion control, independent
    methodology, uncertainty disclosure and publication.
  - Put equality diagnostics under the ratified data wall: separate purpose-
    limited person evidence from the canonical consequential record and from
    eligibility/enforcement data; protect small groups and intersections from re-
    identification; make non-response non-adverse; and provide access, correction,
    challenge, independent governance, and anti-retaliation.
  - Aggregate patterns may create a rebuttable equality presumption and systemic
    audit, but never an individual standing, floor, sanction, risk, entitlement,
    worth, guilt, or punishment result. Test undercount, classification harm,
    political manipulation, stale data, method drift, suppression, identity reuse,
    and attempts to feed diagnostics into an individual consequence.
  - Book 2 owns collection, sampling, linkage, privacy technology, calibration,
    thresholds, and empirical evaluation. The audited specification owns
    authority, purposes, permitted consumers, non-use walls, burdens,
    contestability, and public accountability; Book 1 renders those controls.
    Nibli may consume an authenticated, contestable, bounded finding; do not
    open an engine handoff merely for statistics or claim it computes disparities,
    authenticates identity, or proves institutional action.

- [ ] **Build amendment enactment and effective-version assurance outside the
  reasoning engine.**
  - Bind exact base/candidate byte identity and bounded semantic effects to a
    supplied result admitted through the landed bounded result-certificate
    interface. Keep corridor compatibility separate from political consent.
  - Distinguish that certified candidate from publication, uniquely effective
    deployment, and later rollback or supersession. The current amendment audit
    manually applies candidates and proves only named bounded consequences.
  - The audited specification defines exact-source identity and the successor,
    conflict, replay, and remedy contract; Book 1 renders that contract. A host
    harness and Book 2 must authenticate digests/signatures, store and publish
    versions, select/deploy the effective source, preserve rollback evidence, and
    launch fresh reasoner sessions against that exact version.
  - Test stale base, replay, divergent candidates, unauthorised vocabulary change,
    semantic mismatch, rollback, and query against the wrong source. Nibli may
    reason about supplied version facts but may not be credited with authenticating,
    publishing, selecting, or deploying them.

- [ ] **Complete mobility, collective/plurality rights, and external relations.**
  - Cover newcomers, migrants, refugees, stateless people, borders, asylum,
    expulsion, extradition, accessible language and process, and continuity of
    standing, floors, liberty, and remedy.
  - Implement differentiated collective rights. Protect Indigenous internal and
    local self-government, institutions, language, culture, education, collective
    land/resource title, participation, restitution, and remedy. Protect linguistic,
    religious, ethnic, and other minorities in culture, language, education, media,
    association, accessibility, anti-assimilation, and participation; require an
    independently established historical or territorial basis before giving a
    minority territorial authority.
  - Define membership through self-identification plus the collective's lawful
    acceptance process. Permit multiple memberships and free exit; protect privacy
    and provide an independent procedural challenge without making membership a
    reusable worth, risk, floor, or political-weight score.
  - Permit internal selection and customary law to differ from general-government
    rules only inside universal standing, equality, liberty, due process, individual
    voice, appeal, and Constitutional Court review. Preserve one equal
    general-government ballot and common services for members and nonmembers.
  - Require actual collective consent before permanent forced relocation,
    extinguishment or irreversible impairment of collective title, transfer of
    sovereignty over collective lands, sacred-site destruction, hazardous-material
    placement, or a comparable existential harm. Temporary lifesaving evacuation
    needs its own emergency contract and cannot extinguish title. For other material
    effects require good-faith consultation, accessible information, adequate time,
    accommodation, public reasons, and review rather than a blanket veto.
  - Split formal work by direct legal effect across the existing taxonomy; create
    no new class or omnibus `collective` predicate. Give collective membership,
    authority, title, consultation, and consent findings complete source, writer,
    evidence, privacy, reader, challenge, correction, carry, end, alternate-route,
    continuity, remedy, failure-polarity, and temporal contracts.
  - Nibli may consume an authenticated, bounded membership, title, consultation, or
    consent finding but may not decide collective identity, membership, territory,
    title, consent, or institutional action. Apply the completed finite
    collective-decision audit to any supplied result certificate; do not invent
    generic tally, roster-completeness, signature, or enactment semantics.
  - Preserve a former resident's return right without creating a nonresident ballot.
    The ratified public-safety and external-power work owns accessible evidence and
    border operation, not the right's existence. Book 2 owns land/title administration,
    language and consultation services, records, staffing, costs, capacity, and
    workflows.
  - Acceptance must cover disputed and multiple membership, exit and privacy,
    internal dissent, customary-law conflict, nonmember residents, language access,
    title and restitution, each consent-required harm, consultation-only projects,
    temporary evacuation, and forced assimilation or relocation. Prove that
    collective autonomy never lowers individual rights, common services, or
    political equality and never creates a second general-government ballot.
  - Treaty, diplomacy, external trade, war/peace and humanitarian authority is now
    ratified and owned by the public-safety and external-power implementation bullet
    below; do not duplicate it here. This item retains mobility,
    collective/plurality rights, and the external no-evasion rule: public
    procurement, investment, trade or corporate form may not export labour
    exploitation, ecological damage or rights violations that would be unlawful at
    home. Other states' cooperation and supply-chain facts remain external
    assumptions, not derived facts.

- [ ] **Add the missing non-carceral justice interface.**
  - Use the landed court-holder and independent-review contracts; specify notice,
    counsel/advocacy, hearing, challenge, reparation, remedy, and enforcement across
    civil, administrative, family, labour, consumer, constitutional, and criminal
    justice.
  - Specify investigation/policing, prosecution, defence, adjudication, execution,
    and review procedures without reconstituting their holders. Add victim/survivor
    protection, restitution, reparation, restorative options, and least-coercive
    response without compelling forgiveness.
  - Add explicit carceral limits: bodily integrity, communication, conditions,
    proportionality, release review, post-release continuity, and reintegration.
    Prison remains the hardest stress test, not the default social case.

- [ ] **Implement the ratified public-safety, defence, emergency, and
  external-power baselines.**
  - Follow the author-ratified 2026-08-08 contract in
    [`new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md`](new-book-plans/book-1-public-safety-defence-emergency-and-external-power-decision.md).
    Create **no** new taxonomy class and no omnibus `security`, `emergency`,
    `threat`, `enemy`, `danger`, `border`, `defence`, or `war` conclusion.
  - Formalise six separated Class 6 protective mandates — policing, prosecution,
    adjudication, custodial execution, external defence, security intelligence —
    each with democratic source, mandate, trigger, scope, conflict/recusal rule,
    non-delegable limit, review, appeal, end condition and temporal status. Legal
    defence and victim protection belong to the non-carceral justice interface
    above; cross-reference it rather than restating or narrowing it.
  - Formalise civilian command and the anti-fusion incompatibilities: no serving
    armed-forces or intelligence member in a legislative, executive, judicial or
    oversight seat; no secondment or joint command reconstituting a fused force;
    no military jurisdiction over civilians; civil assistance unarmed,
    individually authorised, and carrying no arrest, search, detention,
    interrogation, crowd-control or surveillance power.
  - Add arrest, pre-trial detention, search and seizure as named coercive
    instruments — none exists in the record today. Each needs an individualised
    recorded ground, identification and reasons, counsel/interpreter/
    accommodation, third-party notification, prior independent authorisation for
    search with a narrow reported immediate-danger exception, and prompt
    **automatic** independent judicial review of detention. Do not reuse
    `capture`: it means *documented*, and its consumers are credibility voiding
    and recognition, so an arrest reading derives both.
  - Formalise the force test — strict necessity, no reasonably available less
    harmful means, minimum sufficiency, warning where feasible, cessation, aid,
    burden of lawfulness on the public actor, lethal force only where strictly
    unavoidable to protect life — plus independent investigation of every death
    and serious injury by a body other than the deploying one, no orders defence,
    and command responsibility.
  - Put the categorical refusals inside the corridor: torture and cruel, inhuman
    or degrading treatment; enforced disappearance and secret detention;
    extrajudicial or arbitrary killing; collective punishment and reprisal;
    indefinite detention without charge or review; coerced confession; human
    shields and attacks on people not taking part in hostilities; starvation or
    floor denial as weapon, sanction or inducement; experimentation without
    consent; indiscriminate and superfluous-injury weapons and autonomous systems
    engaging human targets without meaningful human control; aggressive war.
  - **Two structural obligations attach to every new coercive instrument**,
    because the existing walls do not cover it. The floor firewall reaches the
    confinement conclusion only — the source discloses this and the disclosed
    non-floor control loads at zero errors — so each instrument must carry its
    own rule placing it upstream of personhood **in the same change**, and must
    ship a refusal pin for **each** floor predicate, not one sample, plus a
    re-measurement of the existing refusals. And the **deprivation** stays a
    leaf, not the instrument: every capability an instrument removes joins the
    no-reader guard family in the same change, while the instrument itself may
    be read where the design requires it.
  - **The same firewall boundary already bites on an existing conclusion through an
    existing status, and those two obligations do not by their terms reach it.**
    Measured 2026-08-17: confinement for the absence of a floor right is refused by
    the stratifier, while confinement for the absence of a family status, and for
    the absence of a parent relation, both load. `FamilyStatusNoConfinement` is the
    barrier that would forbid them and it is a `prevents` leaf that no rule reads.
    Institutionalisation for want of a family is the historical harm the ratified
    family baseline refuses in words. Fix it inside a coherent rule family with its
    own refusal pins and re-measurement, never by bolting a conjunct onto the
    conviction rule. Recorded in
    [`new-book-plans/book-1-thesis-framing-and-second-stress-case-brief.md`](new-book-plans/book-1-thesis-framing-and-second-stress-case-brief.md);
    a probe is not a verification run and this needs re-measuring under
    `./verify.sh` before any repair claims it.
  - Keep protective restriction separate from punishment. A quarantine,
    exclusion order, border hold, or pre-expulsion detention carries no punitive
    consequence, never feeds severity, placement, conviction or recognition,
    reads no risk or dangerousness assessment, and gets its own trigger,
    evidence, review and temporal contract. Anyone the state physically holds by
    any instrument is owed shelter and a recorded voice — extend both
    protections in the same change that creates the instrument.
  - Formalise the non-derogating emergency overlay. It confers exactly
    procedural acceleration, resource redirection, compensated requisition, and
    hazard-specific reviewable restriction; it suspends no right, institution,
    election or remedy, creates no decree power, extends no mandate, and leaves
    no standing power between declarations. Rationing routes through the
    ratified physical-scarcity contract rather than becoming a new power;
    compulsory continuity keeps its narrow ratified form; price control is
    ordinary economic law needing no declaration.
  - Give the declaration, **each renewal separately**, and each individual
    measure its own source-bound temporal contract, with every measure rejoining
    the exact declaration version rather than a compact status tag. Do not borrow
    the custody T3 contract for a different power. The polarity is the **same**
    as the custody gate — current authority is a positive premise — so do not
    write an absence-derives-cessation rule; cessation is a positive recorded act
    or a fresh-evaluation claim. Pin the frozen-and-replayed-record limit
    honestly rather than claiming it is closed.
  - Formalise the predeclared alternate authorising route and the predeclared
    independent substitute reviewer, both bound by identical limits and ratified
    at the ordinary body's first opportunity. Neither absence becomes approval
    nor an indefinite hold, and an unratified alternate authorisation ends.
  - Formalise the intelligence limits and extend the temporary-assessment
    exclusion **by name**: no bulk or suspicionless collection, no acquisition of
    what could not lawfully be collected, prior individualised judicial
    authorisation, defined scope and duration, fresh authorisation per renewal,
    later notification, and no risk, threat, loyalty, dangerousness, clearance or
    watchlist product entering the canonical consequential person record or
    conditioning standing, floor, franchise, liberty, remedy or allocation.
    Secret evidence is never sole or decisive.
  - Formalise unconditional conscientious objection with no sincerity tribunal, a
    genuinely non-punitive equivalent, no loss of floor, standing, franchise,
    candidacy, education or employment, no repeated punishment, and the duty to
    refuse a manifestly unlawful order with protection for the refuser.
  - Formalise jurisdiction-wide standing as a **duty and a limit on power**, not
    as a record claim — a person never entered is not recorded as missing, so no
    power may condition help on a record entry or treat an absent entry as a
    finding. Build the enforcement firewall to cover **enrolment** as well as
    collection and transmission.
  - Formalise absolute non-refoulement, the ban on collective expulsion, asylum
    as a right to fair determination with advocate/interpreter/suspensive appeal,
    individual reasoned expulsion, no immigration detention of children, adult
    detention only on individualised necessity with judicial authorisation, a
    maximum and a real alternative, statelessness prevention, and the refusal of
    pushbacks, evasive externalised processing and jurisdiction-shopping —
    effective control, not formal territory, is the test. A scarcity finding's
    named population includes everyone within jurisdiction or effective control,
    and nationality, citizenship, immigration status, documentation and manner of
    entry join the forbidden priority keys.
  - Formalise extradition, mutual legal assistance and transfer of a person as
    the route that must not become an escape: individual judicial decision,
    suspensive appeal, and an express bar where surrender would breach
    non-refoulement or expose the person to a categorical refusal. A diplomatic
    assurance is weak evidence, never a cure.
  - Formalise the external-only defence mandate, the Assembly ceiling on size and
    armament, full audit access, the refusal of delegated private coercion,
    mercenaries and paramilitaries, prior authorisation for force abroad with the
    lapse of an unratified defensive response, the refusal of aggressive and
    secret war, the cyber/infrastructure threshold with evidenced attribution,
    arms-transfer control, unconditional humanitarian duties, and the bar on
    anyone below legal adulthood being recruited or used in hostilities.
  - Formalise the abuse-of-office route with no immunity, repose, amnesty or
    pardon for a categorically refused act, and treat non-recording, destruction
    and falsification of a coercive record as substantive failures attributable
    to the power rather than evidential misfortunes borne by the person.
  - Formalise treaty ratification and the supremacy limit **in the register the
    amendment-semantics audit requires**: a rule addressed to ratifiers,
    reviewers and courts, with the express statement that no current mechanism
    reads a treaty's actual effect. Generalise the no-evasion rule across trade,
    procurement, affiliates, supply chains, flags, arbitration fora and exported
    enforcement.
  - Add the express, bounded common competence and state its disclaimer:
    external representation and treaties, external defence and force abroad,
    borders/entry/asylum/expulsion/extradition, external trade and sanctions,
    security intelligence and oversight, common policing and force minima, and
    cross-border hazard coordination — creating no general security,
    foreign-affairs, policing or emergency power, preserving regional civil
    protection and residual competence, and permitting stronger compatible
    subnational protection.
  - Write oversight and remedy against the existing endpoint rather than over it:
    nothing reads `obliged`, so specify who must be able to look and what must
    follow, and do not upgrade a specification into an assurance.
  - **Before any of the above lands, re-audit four prose sites** and revise them
    in the same content change: the single-deprivation claim, its Part V verdict,
    **and the same claim in `new-book-plans/3-spine.md`'s hand-authored chapter
    list, which no generator and no prose gate covers**; the two confinement
    rule-statements in chapter 8, since shelter and recorded voice must extend to
    anyone the state physically holds; the absent-justifications sentence in
    chapter 4, whose vocabulary half is false the moment justification vocabulary
    is admitted while its reachability half survives; and the accountability
    endpoint in chapter 14. Note that the counted-claim gate matches neither
    small cardinals nor "exactly *n*", so counting discipline here is human until
    that gate is extended — do not cite it as this domain's guard.
  - Census and explicitly dispose of `capture`, `permits`, `authority`, `free`,
    `travel`, `severe`, `prisoner`, `err`/`obliged`, `public`, `defend`, `show`
    and `judge` before reuse. `severe` may not become a threat grading.
  - Nibli may consume a bounded authenticated authorisation, declaration, order
    or review record. It does not detect a threat, decide necessity,
    proportionality, imminence or attribution, authenticate a warrant, prove an
    order was given or refused, advance a clock, end an emergency, or perform an
    institutional act. Formalisation is constrained to the supported exact-ground
    seam and exclusions recorded in
    `new-book-plans/nibli-multi-power-multi-window-protective-authority-capability-audit.md`.
  - **Book 2 handoff:** force capability, doctrine, training, equipment and
    less-lethal options; forensics, investigation practice and case
    administration; intelligence tradecraft and information security; border,
    reception, asylum-processing and non-custodial-alternative operations;
    incident command, stockpiles, disaster logistics, continuity planning and
    infrastructure restoration; defence procurement, readiness and sustainment;
    treaty negotiation, sanctions administration and consular practice; staffing,
    costs, capacity and empirical feasibility; and the assurance that an outside
    clock advances, a required record arrives, and a reviewer acts. Other states'
    cooperation, recognition and readmission are **named external assumptions**,
    not derived facts and not Book 2 deliverables.

- [ ] **Implement the ratified environmental right, Class 9 commons, and
  Class 10 non-human-animal protections.**
  - Follow the author-ratified 2026-08-08 contract in
    [`new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md`](new-book-plans/book-1-ecological-future-generation-commons-and-non-human-animal-decision.md).
    Keep the existing material-floor inventory unchanged and create no omnibus
    `environment`, `ecology`, `sustainable`, `animal`, `sentient`,
    `balance`, or `worth` conclusion.
  - Preserve three non-collapsed holders and effects: every person's independently
    enforceable environmental right; Class 9 protected commons and future
    ecological capability; and credibly sentient animals as direct Class 10
    protected subjects without `person`, floor, ballot, property, contract, or
    human-equality status.
  - Implement the environmental right's public and material-control private reach;
    clean air, safe water, healthy soil/food systems and hazardous-exposure
    protection; information and uncertainty disclosure; prior cumulative,
    distributional, cross-boundary and long-latency assessment; accessible
    participation; reasons; retaliation protection; review, interim protection,
    correction, continuity and remedy. Route unequal exposure through substantive
    equality without creating a vulnerability or ecological-worth score.
  - Give each Class 9 common a versioned, multidimensional ceiling, minimum
    condition or resource budget bound to source/method version, protected
    condition, place, territory, jurisdiction, population and temporal scope.
    Preserve climate, water, air, soil, biodiversity, habitat, extraction and waste
    as non-substitutable axes; no offset, payment or scalar pass may excuse a breach
    or replace an irreplaceable, unique, sacred or functionally non-substitutable
    system.
  - Keep science and law distinct: independently reviewed science proposes or
    authenticates evidence; democratic law enacts the operative ceiling and may
    choose stronger protection; the Guardian advocates and challenges; an
    independent court reviews legality. Implement proportionate precaution,
    outcome-based non-regression, cumulative/supply-chain/no-jurisdiction-shopping
    controls, public reasons, correction and challenge.
  - Give independent commons/future-condition initiation routes to every present
    person, a qualified association, the ordinary rights advocate, and the
    collegial Future Conditions Guardian. No route claims an unborn person's
    preferences or depends on proprietary injury. A present person, qualified
    association or rights advocate may request judicial interim relief but does
    not trigger the automatic stay.
  - Formalise avoid → minimise at source → restore in place → tightly bounded
    replaceable residual compensation, followed separately by human/collective
    reparation. Record additionality, durability, functional/place relevance,
    no double counting, monitoring, correction and repair where ecological
    equivalence is claimed.
  - Implement dual floor-and-ceiling continuity: neither floor denial nor ceiling
    breach is success. If no present route satisfies both, preserve immediate
    human continuity through the least-harm route, record both shortfalls, obtain
    alternatives, repair harm, and use a source-bound transition. Budget choice,
    delay, monopoly, artificial withholding or refusal to procure proves neither
    physical nor ecological scarcity.
  - Implement tiered ecological liability without making urgent protection wait:
    immediate no-fault prevention, cessation, containment, animal/human continuity
    and public restoration; strict restoration and reasonable-response-cost
    liability for causally connected inherently hazardous activity; adjudicated
    contribution/control otherwise; public continuity where a responsible actor
    is unknown, insolvent or absent; and separate proof of culpability before any
    punitive or criminal consequence.
  - Create the collegial Future Conditions Guardian under the ratified divided,
    anti-capture appointment contract, separate from regulator, scientist, court,
    auditor and operator. Only its evidence-supported objection, or that of its
    predeclared alternate advocate acting in its place, automatically pauses the
    irreversible part through a fresh T3 window pending expedited independent
    review, with essential-floor continuity.
  - Bind one shared replay key across the Guardian, alternate, successor, primary
    reviewer and substitute reviewer to the case, challenged authorization
    version, evidence version and ground. A final resolution prevents restart on
    that key; only materially new authenticated evidence or a materially changed
    authorization may open another. Silence, vacancy, conflict or capture is never
    approval: use the predeclared alternate advocate for the Guardian function and
    a separate substitute reviewer for review. The Guardian has no policy veto,
    final merits power, budget, programme or scientific-oracle status.
  - Implement Class 10's sentience presumption and evidence-responsive extension,
    direct bodily/life/species-appropriate interests, dependency-created welfare,
    rescue, care, habitat, movement/social opportunity, humane handling and death,
    and protection from abandonment, exploitative overwork, extreme confinement,
    harmful breeding and avoidable invasive intervention. Never rank an animal by
    owner, price, affection, recognition, productivity, rarity alone or a total
    animal-worth score.
  - Apply the non-waivable evidence-based welfare baseline to **every** controlled
    use. Ordinary non-food/non-lethal/non-invasive/non-high-severity use needs no
    enhanced serious-purpose test merely because it is a use. Apply the enhanced
    purpose, necessity, feasible-alternatives, least-harm, care, prior-review and
    source-bound-end test only to lethal, invasive or high-severity use. Food-
    producing use receives the welfare baseline plus the separate strict rule
    below.
  - Apply the strict food rule to **every controlled use of a credibly sentient
    animal to produce food**, not only slaughter or an already classified
    high-severity use. It is lawful only where no safe, accessible, nutritionally
    adequate, materially less-harmful alternative is reasonably available to the
    affected people. Taste, habit, prestige, profit, advertising or price alone
    is insufficient; transition may never withhold the human food floor.
  - Apply the strict research/testing/education rule: serious health, safety or
    ecological purpose; replacement, reduction and refinement with replacement
    first; no scientifically valid non-animal/materially less-harmful alternative;
    independent project review; minimum animals/least-harm design; pain relief,
    humane endpoints, aftercare and public registration/results. Categorically
    refuse every unrelieved severe or prolonged procedure; no claimed purpose,
    prestige, curiosity or commercial benefit may override that ban.
  - Formalise the remaining categorical refusals, including fighting, sexual use,
    deliberate cruelty, punitive treatment, dispensable killing/severe suffering,
    abandonment, extreme confinement, seriously harmful breeding, painful
    purposeless mutilation, and cosmetic/marketing use. Put direct protected-
    subject status and the severe-avoidable-suffering and dispensable-killing core
    inside the unamendable democratic corridor.
  - Cover companion/domestic, farmed/working, captive and wild animals without
    ownership-form exceptions. Natural predation creates no offender and no
    universal rescue duty. Prioritise human-caused or controlled pollution,
    infrastructure, entanglement, habitat, capture, trade and killing harms.
    Permit grave disease/danger/introduced-population control only after
    authenticated risk, human-cause prevention and feasible nonlethal exclusion,
    treatment, vaccination, relocation, fertility or habitat routes fail or would
    cause greater grave harm; require the least-painful reliable method, review,
    reassessment, repair and fresh T3.
  - Create the separate Animal Protection Advocate with initiation, evidence,
    inspection, rescue/cessation and remedy powers but no ownership, custody,
    prosecution, permit, budget, programme, adjudication or veto. When Guardian
    and Advocate positions conflict, require an independent adjudicated result
    keeping human right/floor, Class 9, Class 10, collective rights, alternatives,
    uncertainty, reversibility and continuity separate and applying categorical
    refusals first.
  - Give each ecological and animal finding an authorised source/writer, exact
    version/place/jurisdiction/time scope, independent reader, privacy boundary,
    reasons, challenge, correction, carry, end, alternate route, audit, continuity,
    remedy and failure polarity. A missing record cannot block urgent protection,
    but FALSE remains non-derivability rather than proof of safety; Nibli consumes
    only authenticated bounded findings and never measures, authenticates,
    classifies, advances a clock or operates an institution.
  - Give every permit, ceiling, Guardian/alternate automatic stay, judicial
    interim-relief order, restoration order, research approval, disease-control
    action and lethal/high-severity authority its own source-bound temporal
    contract. Custody T3 cannot be reused, reviewer silence cannot approve, and
    correction cannot silently extend expired harmful authority.
  - **Book 2 handoff:** numerical ceilings, budgets, measurements, models,
    inventories, uncertainty methods, monitoring, laboratories, restoration,
    husbandry, veterinary/rescue/shelter/inspection capacity, food/research/
    worker/community transition, cross-border coordination, staffing, costs and
    empirical feasibility. Book 1 must derive the legal interface without claiming
    those operations exist.
  - **Done when:** contract cards, pins, counterfactuals and reader prose cover
    ordinary compliance and breach; independent environmental-right and commons
    claims; non-substitutable ceilings; precaution/non-regression; floor/ceiling
    collision; Guardian/alternate automatic stay, ordinary-route judicial interim
    relief, shared replay, substitute review and no veto; tiered liability;
    ordinary controlled use; enhanced lethal/invasive/high-severity use; strict
    food and research routes; categorical refusals; domestic/farmed/captive/
    wild cases; Guardian/Advocate conflict; rescue/restoration/remedy; corridor
    refusal; record mismatch/withholding/correction; and fresh temporal expiry.

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

- **Add `LICENSE-MIT` + `LICENSE-APACHE` — now unblocked.** The condition ("when the
  harness and fetchers are written") is met: `registry/check.py`,
  `registry/fetch/worldbank.py` and `new-book-plans/6-claim-table.py` exist, the first
  two already carrying `SPDX-License-Identifier: MIT OR Apache-2.0` headers. Fetch both
  canonical texts (per `LICENSING.md`), mirror nibli's layout, and add the SPDX header
  to `6-claim-table.py` and `verify.sh` in the same commit.

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

- **Align the current time account before any public expansion snapshot.**
  - Chapter 13's ordinary-language/admitted-fact distinction has landed. Remove
    or narrow the remaining permanent-refusal wording in Chapters 4 and 13 that
    contradicts T3 as a ratified future target.
  - Cross-read Chapters 4, 5, and 13 against one current model: flat snapshots
    have no internal order; epoch carry is an external/manual cross-snapshot
    convention; no current duration or automatic expiry exists.
  - Keep this prose-only correction separate from the ratified T3 implementation
    gate.
    Re-run the relevant prose, claim, and pin checks before publishing a
    snapshot that contains the affected chapters.

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

```
./verify.sh                 # 34.89 s measured 2026-08-05 with an independently
                            #   built clean 4cb02aa release supplied through NIBLI_PIN:
                            #   engine, spine, assertion surface, assurance case,
                            #   bounded red-team, amendment-semantics and placement
                            #   contracts,
                            #   evidence count, jargon,
                            #   counted-claims hard
                            #   gate, claim-comment check,
                            #   registry check, absences, INVARIANT 1, the arity and
                            #   counting guards, control scope, the pin
                            #   suite (555 pins) with cross-file :expect-pins
                            #   reconciliation, 15 record snapshots / 108 pins,
                            #   40 temporal processes / 236 pins, nine amendment
                            #   candidates / 44 pins, 24 placement rows / 336 pins,
                            #   24 cold composed floor probes / 24 pins, five
                            #   placement mutations / 73 pins, five placement
                            #   mutation-baseline sabotages, one composed-standing-
                            #   removal sabotage, the other executable
                            #   controls, one record failing-pin control, and source
                            #   counterfactuals in three diff classes — line deleted,
                            #   line changed, line added
./verify.sh --quick         # 2.22 s with the same pinned binary (2026-08-05): skips
                            #   chapter/floor pins, executable record snapshots,
                            #   amendment and placement executions, and counterfactuals
                            #   — drafting feedback only, never a semantic gate
./verify.sh --only <file>   # one pin file, selected release engine, --allow-shell, and
                            #   the fixture's own KB chosen for counterfactual files;
                            #   partial by design — use while the batch changes
./verify.sh --table         # emit the claim-to-query table extracted from the pins
```

**Current verifier rule, protocol v6, superseding the historical command notes
above on 2026-08-27.** Any semantic, executable, verifier, fixture,
engine-binding, or generated-artifact candidate is fully staged and receives one `./verify.sh
--emit-receipt new-book-plans/verification-receipts` run.
Full already contains the quick path. Only its exact audit, closure, and tracker
successors may reuse it through the named `--commit-gate` transition while the
heavyweight manifest remains byte-identical and the narrow structural gate
passes. Missing local evidence or any unclassified delta fails closed without a
silent full run. Heavyweight entry points share one Git-common-directory lock;
contention exits 75 unless an explicit bounded `--wait-for-lock SECONDS` is
supplied. The older timings, suite inventory, and v1 workflow remain
historical measurements, not current commit instructions.

The 2026-08-30 workflow clarification changes candidate construction, not that
gate. A coherent batch may be built through several bounded **uncommitted**
authoring slices, using quick and focused modes while bytes move. `Drafted — not
audited` carries no receipt and authorises no semantic commit. Once frozen, the
whole fully staged batch receives one receipt and one candidate commit; changed
bytes, intermediate semantic commits, or a second candidate cannot share it.

The only exception is the exact `FS-SAU-42` forward recovery defined in section
5 of `new-book-plans/full-society-scope-review-protocol.md`. It validates the
two named historical v5 receipt/audit epochs and the named closed anchor from
committed bytes, requires their digest-bound local evidence, performs no
ancestor search, and is consumed by its successful closure.

Prefer it to any check by hand. It exits non-zero on the first failure and names the
claim that stopped being true — including exit 3, the failure that is good news: a
pinned `:defect` stopped reproducing, and the script names it a REPAIR, not a
regression, because the response is to drop the marker and rewrite the prose that
called it a flaw, never to debug the harness. The script incrementally builds one
native `rights-verify` executable whose embedded engine comes from the adjacent
Nibli source checkout; receipt emission binds both that source revision and the
exact verifier bytes. For standalone manual queries, use the **release**
`nibli-pin` at or after `4cb02aade43b394374c40e661907ad66df3af3fe`, never
`nibli-host`. A stale binary can preserve logical
verdicts while violating bounded completion: pre-`5cec800` builds failed ordinary
full-source opaque queries, and `5cec800` restored those but could still time out when
standing itself had to traverse the T3 custody chain. `4cb02aa` closes that composed
boundary, so a green pin result alone does not establish engine freshness. **Gate on
the verifier's exit status, never on its output**: piping to `tail` swallows the exit, and `echo $?`
followed by `&&` gates on the echo — both shapes shipped a red commit on 2026-08-02.
The pre-receipt safe chain was `./verify.sh > /dev/null 2>&1 && git commit …`.
That shell-shape lesson still applies, but protocol v6 uses `--emit-receipt` for
the semantic commit and the exact named `--commit-gate` for each permitted
administrative successor.

**Every check was negative-controlled before it was trusted, and one failed the
control.** The jargon pattern this file used to specify (`stratum|strata`) does not
match *stratifier* — the likeliest leak of all, since it is the word this tracker uses
constantly — and a chapter containing it passed; `strat` alone is too greedy, it matches
"demonstrate". The shipped pattern uses three explicit stems. For the same reason every
structural check carries a positive control: a grep that also matches a predicate's own
rule head can never fail, which is a trap this repo fell into twice in one day.

**Extending it as the book grows is the standing job**, and a new check earns its place
by failing against a sabotaged copy before it is trusted, never after.

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
