<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Reader Evidence Protocol Decision

> **Status: author-ratified on 2026-08-09; dormant machine components
> implemented, empirical protocol execution pending.** The 2026-08-09 decision
> supplies the reader-balance evidence protocol and its threshold timing. It
> ratifies no threshold values, runs no session, builds no instrument, and
> makes the reader route **neither built nor available**. The later dormant
> source, generator, structural checker, deterministic evaluator, and fixed
> admission-gate component create no instrument, session, threshold, reviewer,
> admitted evidence, release, or claim upgrade. Building and self-testing the
> gate component does not build or make R6 available. Neither the ruling nor
> those artifacts add a predicate, rule, fact, pin, chapter, or public coverage
> claim; reword a gate's permitted-claim string; or change a sentence of the
> book.

## 1. Decision

The reader-balance evidence protocol is the **pre-registered
pilot-and-fresh-holdout method**, ratified as specified in the tracker's
sessions item and restated in section 2. The pass rule's **form** is ratified —
a severity-weighted misconception rule under which no aggregate may hide a core
misconception — and its **values and severity taxonomy are reserved** for a
second author ruling that must land after the pilot and before the fresh
holdout's pre-registration freeze. The measured ordinary-life deficit is a
named, disclosed input to the protocol's design, and the disclosed-limits
minimum in section 5 is fixed now. The reader route's availability contract is
specified in section 6. Its dormant machine structure is now materialised, but
the availability tuple remains incomplete.

Summarised anywhere, this ruling is three statements, not one, because the
shorter version is an overclaim: the method, timing, disclosure, ethics terms,
and the pass rule's *form* are author-ratified; the rule's *values* remain
author-ruling-pending, reserved for the second ruling; and the public-edition
gate's holdout condition is therefore not yet satisfiable, with every reader
comprehension, balance, and lived-effect claim keeping its
Unestablished/route-unbuilt disposition until the reader route is available.

One naming note, so the tracker's shorthand does not mislead: the roster calls
this "the reader-threshold decision." This ruling settles the **protocol and
its timing** only. The threshold itself is the reserved second ruling, and
calling this decision by the shorthand does not make that ruling made.

### 1a. What this refuses at the outset

An aggregate score deciding a pass; word, chapter, demographic, or sentiment
quotas, and any fixed prisoner/non-prisoner quota; population-statistics
claims from usability evidence; reader evidence offered to prove a domain
assigned to another assurance route; any reader result entering the reasoning
engine by any path; declaring the reader route built or available; predicting
the outcome of any session; ratifying threshold values before pilot evidence
exists; illustrative threshold numbers, which would anchor the reserved ruling;
naming a reviewer this ruling does not have; and rewording either standing
permitted-claim string.

## 2. The method, ratified as specified

The sequence is: pre-register the pilot instrument; run the pilot; revise; the
author ratifies the pass rule; pre-register the ratified rule and the revised
instrument; then run the fresh holdout. Each step is what it says and nothing
further — running the pilot establishes nothing about the book, and only the
holdout, judged under the ratified pre-registered rule, can feed the
public-edition gate's reader condition.

- **Sample.** Non-specialists with varied reading confidence, language
  backgrounds, and accessibility needs. The evidence is usability evidence
  about the tested audience, never population statistics.
- **Unaided prompts.** What do people do in ordinary life? What may they choose
  privately and democratically? What do public bodies do when nothing has
  failed? How does something owed arrive and get repaired? Why is the prisoner
  present? What remains operational or externally assumed, and what can no
  model guarantee?
- **Identification targets.** Readers must identify ordinary constructive life,
  democratic choice, private freedom, successful provision, repair, and the
  prisoner as a stress test rather than the society's central inhabitant. This
  is the **minimum pass-relevant set, not a closed list**: the instrument must
  also cover the unaided prompts above and the public-edition gate's own
  identification and distinction conditions — normal functions, freedoms,
  guarantees, remedies, Book 2 dependencies, external assumptions, and the
  response-stage distinctions — which remain independent gate conditions this
  protocol does not discharge.
- **Qualitative pass shape.** A pass requires readers to identify constructive
  functions as well as restraints, trace one successful delivery/remedy path
  and one democratic choice, and recognise the prisoner as a stress test. How
  these are judged in detail belongs to the rule whose values are reserved.

## 3. The sequencing rule

This is the timing half of the ruling, and it construes a sentence the tracker
already carries. The sessions item says to pre-register questions, coding
rubric, sample, and pass rule, then run a pilot — and also says the rule is
ratified by the author after the pilot. Both are honoured by reading
pre-registration as binding **each round's instrument before that round**:

1. **The pilot runs under a pre-registered pilot instrument** — questions,
   coding rubric, and sample rule fixed before the pilot begins, carrying at
   most a draft pass rule and a draft severity taxonomy, both expressly
   provisional and non-binding. The pilot is instrument work: it establishes
   what it executes, over what it was given, and nothing further.
2. **The reserved ruling lands after the pilot and before the holdout's
   pre-registration freeze.** It ratifies the severity taxonomy and the values
   together, on pilot evidence. Ratifying them earlier would invent a standard
   before evidence exists; ratifying them later than the freeze would make
   "pre-registered" false.
3. **A holdout run under an unratified, retroactively chosen, or post-hoc
   amended rule is void** and cannot feed the public-edition gate. Voiding is a
   lifecycle/admissibility finding, not permission to delete the attempt or
   rewrite a result.
4. **The reader route must be available before the holdout runs** — the
   implemented evidence contract, named reviewer with gate-bound custody, fixed
   digest-bound admission gate, and watched-failing seeded control must all
   exist. Every active completed attempt then stores the gate's exact
   `gate_admission_receipt`; only `decision=admit` may establish FS-CLM-37.

This is the assurance portfolio's own sequence made explicit for this route:
the pilot may run and the rule may then be ratified. Separately, the route
becomes available only after its evidence contract, admissibility criteria,
named reviewer, evidence-admission gate, and watched-failing seeded control are
complete; only a matching valid holdout pass can then make the gate's reader
claim sayable. Nothing here shortens that sequence.

Implementation preserves every pilot and holdout attempt through two coupled
checks. Inside the current source, attempt hashes are chained and an
active-attempt pointer selects the current one. That snapshot-local closure is
insufficient by itself. The root `history_transition` records
`previous_source_commit`, `previous_source_sha256`,
`previous_history_head_sha256`, and `history_head_sha256`. The checker finds
the nearest earlier commit on normal first-parent ancestry that changed
`reader-evidence.json`, binds its exact source bytes and history head, preserves
every earlier attempt prefix, and permits one domain and one step only: append
one attempt, or move the active attempt from a nonterminal to a terminal state.
A terminal attempt is immutable.

Each successor pre-registration binds `predecessor_attempt_sha256` and
`prior_history_head_sha256` to the exact frozen predecessor attempt and prior
history head. The dormant source carries null predecessor fields and the
deterministic empty history head.

The axes stay orthogonal: `holdout_status` follows the active attempt, while the
top-level result is the most recent completed non-void outcome and persists
independently. A `void` attempt with `not-run` is valid. A later frozen,
not-run, or void attempt cannot rewrite an earlier valid `fail` as `not-run`.
Every attempt keeps its own record and result, if any, but no void attempt feeds
Gate C; a replacement is a new pre-registration and genuinely fresh sample,
not an edit to the old attempt.

Study IDs, coded-record commitments, custody IDs and digests, and receipt IDs
are globally unique across the complete pilot/holdout history. Every run carries
exactly one freshness record; zero, duplicates, or a record scoped to another
run are invalid. The checker verifies these bindings over the visible normal
first-parent Git history. It does not prove resistance to rewritten Git history
or the external truth of a freshness, custody, or study statement.

## 4. The pass rule's form, values reserved

The form is a **severity-weighted misconception rule**: reader misconceptions
are classified by a severity taxonomy, and no aggregate score may hide a core
misconception — a finding in the core tier cannot be offset, averaged away, or
outvoted by favourable results elsewhere. Severity weighting is defined
non-numerically here by that veto structure; this decision deliberately
contains no threshold number, weight, sample size, or example value.

Reserved for the second ruling, on pilot evidence: the severity taxonomy
itself, including what counts as a core misconception; whether a single core
finding or a repeated one fails — the tracker states the veto both ways, and
choosing between them is a value judgment that belongs with the values; the
pass/fail mapping for non-core severities; and every quantitative or
qualitative threshold. The second ruling may not reopen the method, the
disclosure minimum, the ethics terms, the no-aggregate veto, or the
non-substitution of routes.

Once the reserved values exist, the executable controls derive end-to-end
below-, exact-, and above-boundary fixtures from every value at reachable
observations. An unreachable or out-of-domain edge is explicit and fails closed;
it may not disappear from the fixture set or be replaced by a hard-coded
illustrative number.

## 5. The deficit as disclosed input

The register ruling's three statements govern and are restated as binding on
the protocol's drafting: the deficit does not by itself make the
public-edition gate unpassable, since the gate's claim is protocol-relative;
this ruling makes no prediction about the outcome; and the deficit is a named,
disclosed input the protocol may not be written so as to hide.

The protocol's disclosed limits must contain, at minimum:

- the statement that the book's ordinary-life account rests on unimplemented
  families at the time of testing — the register ruling's mandatory sentence;
- the tested snapshot's exact version identity;
- that the evidence is usability evidence about the tested audience, not
  population statistics;
- the sampling and method limits that bound the public-edition gate's
  permitted claim; and
- that no reader result enters the reasoning engine, and none proves a domain
  assigned to another assurance route.

A pre-registration document missing any element of this minimum is not the
declared protocol.

## 6. The reader route's availability contract, specified and dormant

The assurance portfolio makes the reader route available only when its
evidence contract, admissibility criteria, named reviewer with gate-bound
custody attestation, dedicated executable evidence-admission/evaluation gate,
and watched-failing seeded control exist — and naming a route does not build it.
This section specifies what each
must contain. The later machine record materialises their state space and
digest relationships only. **Specifying and structurally recording are not
building: R6 remains neither built nor available**, and the availability tuple
below remains incomplete.

- **Evidence contract.** Private custody retains the pre-registration, tested
  snapshot, instrument and rubric as run, admissibility source records, and
  participant/session/coder/reviewer/custodian identity material. The public
  record contains exactly opaque study IDs, coded target/misconception outcomes,
  artifact or commitment digests, coded deviations, and custody attestations
  without identity material. Names, pseudonyms, identifiers and identity
  mappings; raw or free-text responses; consent and withdrawal material; and
  direct contact, demographic and accessibility records remain outside the
  public repository.
- **Admissibility criteria.** Ethics compliance per section 8 — a session run
  in breach is inadmissible regardless of its results. Freshness — holdout
  participants have no prior exposure to drafts, previews, the pilot, or the
  reviews corpus, and pilot participants are excluded from the holdout. The
  in-repo reviewer corpus is never admissible reader-study evidence: it has no
  consent, compensation, or sampling frame, and it remains what the assurance
  portfolio already says it is — Reasoned design input, permitted in the
  exempt elements.
- **Named reviewer.** One must be named at implementation, with a custody
  attestation bound to the dedicated gate interface. This ruling names none,
  deliberately: naming a reviewer would begin implementation, and inventing one
  would be worse.
- **Structural checker.** The implemented dormant checker validates states,
  completeness, digest relationships, the pre-registration and receipt
  `structural_checker_sha256` bindings, the root `history_transition`, the
  frozen-predecessor `predecessor_attempt_sha256` and
  `prior_history_head_sha256` bindings, and the deterministic evaluator by
  which a future receipt is recomputed from a ratified rule and admitted coded
  outcomes. With all threshold fields unset it produces no reader verdict. Its
  checks and mutations are artifact evidence only.
- **In-repo evidence-admission/evaluation gate.** The fixed executable gate
  component exists and its `--self-test` runs in quick and full verification.
  That component is distinct from the structural checker, and building it does
  not build or make R6 available. Every active completed attempt must store
  `gate_admission_receipt` as the gate's exact digest-bound output; only such an
  output with `decision=admit` may establish FS-CLM-37. No active completed
  attempt, named reviewer, or gate-bound reviewer custody attestation exists.
  Neither checker nor gate attests to the external truth of a freshness or
  custody statement.

## 7. The negative control

The route's falsification condition is declared now: the instrument, applied
to a **seeded control** — a text or coded response sample deliberately written
to be unbalanced or to plant core misconceptions — must fail it. The pilot's
revise step must include watching the control fail, and the control ships with
the instrument when the route is built. An instrument that cannot be made to
fail is not an instrument; the house rule carries over unchanged — sabotage
first, trust after.

The dormant checker's schema and state mutations are not that seeded
misconception control and do not satisfy this prerequisite. No valid control
run exists until the pilot actually watches the instrument reject the seed.
Candidate, `author-ratified`, frozen, and completed states must also carry the
watched-failing mutations relevant to that populated stage; a required mutation
set that is missing, empty, or falsely marked inapplicable fails closed. Those
artifact mutations remain distinct from the seeded misconception control.

## 8. Ethics as admissibility

The ethics terms are ratified as **binding admissibility criteria**, not
guidance: obtain informed consent; permit withdrawal; minimise and protect
data; provide accessible participation and fair compensation; prevent
retaliation; add trauma safeguards where coercive experience is discussed; and
use independent ethics and safety review where appropriate. A session run in
breach of any of these is inadmissible regardless of what it found. Reader
evidence never purchases authority over the people studied, and the route's
warrant stays what the portfolio gave it: comprehension, balance, and human
effects for the tested audience, nothing legal, nothing empirical about
populations, nothing about the people themselves.

## 9. Objects under test

- **The pilot** runs against a declared, versioned snapshot, and only after the
  reader-facing navigation artifacts — the Reader's Map, glossary, and
  accessible-navigation work — exist. Piloting earlier would conflate
  instrument defects with known-missing scaffolding. Pilot results are
  evidence about the instrument and that snapshot only, never about the
  release candidate and never about the book's suitability. Its freeze carries
  an external prior-commit or custody binding to the pre-registration and
  tested snapshot, including the computed `attested_payload_sha256`; an opaque
  external receipt digest without that exact payload binding is insufficient.
  The checker verifies the binding, not whether the external custodian exists
  or tells the truth.
- **The holdout** runs against the frozen private release candidate after the
  expansion freezes, exactly as the edition contract already provides. That
  rule is cited here, not amended. Its external pre-registration freeze binds
  the exact ratified-rule digest, revised instrument and rubric, private
  release-candidate identity and artifact hashes, sample and recruitment rule,
  disclosure set, and study protocol. The freeze carries the computed
  `attested_payload_sha256`, and its pre-registration and later receipt bind the
  exact `structural_checker_sha256`. Every holdout embeds
  `frozen_ratification`, whose historical rule, candidate, pilot basis, and
  digest validate independently; the candidate commit must be an ancestor of
  current `HEAD`. A successor pre-registration also binds
  `predecessor_attempt_sha256` and `prior_history_head_sha256` to the exact
  frozen predecessor attempt and prior history head; those fields are null only
  where the schema declares that no predecessor exists. The public record keeps
  only the binding digests and privacy-safe custody attestation. If exposing the instrument could contaminate
  recruitment, publish a nonce-protected commitment under named private custody
  and reveal its preimage after the holdout. Any bound change voids that attempt
  and requires a new pre-registration and genuinely fresh sample.

Every freeze and transition time is canonical UTC. A freeze precedes its
`completed_at` or `voided_at`; when a commitment is revealed, `revealed_at`
follows that terminal event; and a successor attempt begins only after the
predecessor's required terminal and reveal events.

## 10. The accessibility seam

The public-edition gate's permitted claim names one compound object: the
declared accessibility and reader-balance protocol. **This ruling declares the
reader-balance half only.** The accessibility half — semantic navigation,
screen-reader, EPUB, HTML and PDF paths, and the gate's accessibility checks —
stays owned by the accessible-navigation item and is not declared here. The
gate's claim requires both halves, and neither half's pass substitutes for the
other; the portfolio's non-substitution rule applies between them as it does
between routes. What does belong to this protocol from that seam is
sample-side: accessible participation and the sample's varied accessibility
needs are terms of this protocol.

## 11. Evidence and limits

What this decision does **not** establish:

- that readers will pass, or that any session will be run — no prediction is
  made in either direction;
- when the pilot runs, or that the navigation artifacts it waits on will land
  on any schedule;
- that the reader route will be built or become available — the availability
  state is structurally represented, but its tuple remains incomplete;
- that the instrument is valid — nothing is valid before its control has been
  watched failing;
- that the public-edition gate's holdout condition is satisfiable today — it
  is not, and reader comprehension, balance, and lived-effect claims remain
  Unestablished/route-unbuilt; or
- that a protocol prevents an overclaim. It makes one legible: the disclosed
  limits and the deviation log are the mechanism.

**Remaining boundary:** this is a rule about how reader evidence will be
collected and judged. It collects none, and the reserved threshold ruling —
the pass rule's values and severity taxonomy, after the pilot, before the
holdout's pre-registration freeze — remains the author's.

## 12. Ratification record

On 2026-08-09 the author ratified:

- [x] the pre-registered pilot-and-fresh-holdout method as specified in the
  sessions item — sequence, sample, unaided prompts, and qualitative pass
  shape;
- [x] the identification targets as the minimum pass-relevant set rather than
  a closed list, with the gate's own identification and distinction conditions
  remaining independent;
- [x] the pass rule's form — severity-weighted, with no aggregate hiding a
  core misconception — with its values and severity taxonomy reserved;
- [x] the sequencing rule: pre-registration binds each round's instrument
  before that round; the reserved ruling lands after the pilot and before the
  holdout's pre-registration freeze; a holdout under an unratified or post-hoc
  rule is void;
- [x] route availability before the holdout, so holdout evidence is collected
  under a standing admissibility contract;
- [x] the deficit as a named, disclosed input, with the disclosed-limits
  minimum fixed as listed;
- [x] the availability contract specified at contract level — evidence
  contract, admissibility criteria including freshness and the reviewer-corpus
  exclusion, a reviewer to be named at implementation, and an in-repo gate —
  with the route remaining neither built nor available;
- [x] the declared falsification condition and the seeded control watched
  failing during the pilot's revise step;
- [x] the ethics terms as binding admissibility criteria, with breach making a
  session inadmissible regardless of results;
- [x] the pilot object — a declared, versioned snapshot, after the navigation
  artifacts exist — and the holdout object cited from the edition contract
  unchanged;
- [x] the accessibility seam: this ruling declares the reader-balance half
  only, the accessibility half stays separately owned, and neither substitutes
  for the other; and
- [x] the refusals in section 1a, including no illustrative threshold numbers
  anywhere in this record.

The 2026-08-09 decision changed planning only and created no implementation.
On 2026-08-11 the author instructed implementation of the dormant record
contract and approved the `evidence-pending` Unestablished disposition. The
resulting reviewed source, generated report, structural checker,
deterministic evaluator, and fixed gate component implement only dormant
machinery. Their current states are `pending-pilot`, `not-frozen`, and
`not-run`; they contain no taxonomy labels, threshold values, active completed
attempt, or `gate_admission_receipt`. The dormant `history_transition` has
null `previous_source_commit`, `previous_source_sha256`, and
`previous_history_head_sha256`, plus the deterministic empty
`history_head_sha256`; no pre-registration predecessor binding exists. The gate component's self-test is not a
reader result and does not build or make R6 available. These components create
no instrument, session, reviewer, admitted evidence, release, predicate, rule,
pin, chapter, established posture, or public claim. No pilot or valid
seeded-control run has occurred, no second threshold ruling has landed, no
holdout has been frozen or run, R6 remains neither built nor available, and
FS-CLM-37 remains Unestablished/route-unbuilt.

This 2026-08-11 paragraph is a current-state implementation note, **not** the
reserved second author ruling. The 2026-08-09 ratification record above remains
intact. The second ruling cannot be appended until a valid completed pilot and
frozen pilot decision packet exist and the author ratifies the exact candidate
rule and digest.
