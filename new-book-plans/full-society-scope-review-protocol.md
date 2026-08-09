<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Full-Society Scope-Review Protocol

> **Status: author-confirmed 2026-08-09 — basis recorded.** This document specifies
> the protocol under which the full-society ledger's independent scope review
> would run — route R7's evidence contract and admissibility criteria, the
> resolving protocol for reviewer proposals, and the commitment scheme for the
> review's own controls. Specifying is naming, not building: this document
> makes route R7 neither built nor available, names no reviewer, designates no
> severity owner and no independent checker, constructs no plant and no seed,
> publishes no commitment, opens no window, runs no review, and asserts
> nothing. It adds no predicate, rule, fact, pin, chapter, release, or public
> coverage claim; it upgrades no claim's posture; it rewords no
> permitted-claim string; and it leaves FS-RTE-07 unbuilt. The commitment
> machinery landed beside it lives in the canonical source and its generator,
> which remain operative; this document binds meanings onto that machinery and
> adds none of its own. Confirmation is a recorded author act, never a
> rewording: when it lands, the status line above and the confirmation record
> in section 10 change, and nothing else in this document needs to.

## 1. What this protocol is

This is R7 work under the assurance portfolio's own licence — building a route
is work, not a claim — and it takes no posture. The portfolio defines an
available route as one whose evidence contract, admissibility criteria, named
reviewer, and in-repo gate exist. Even confirmed, this document supplies at
most the evidence-contract and admissibility-criteria half of that closure
condition; no reviewer is named and no review event exists, so the route
remains unbuilt and unavailable, and whether any half "exists" for
availability purposes is a call this document does not make. The canonical
source's route record makes that call, and it records the route unbuilt.

The review this protocol governs is the one closure condition five of the
full-society boundary decision names: an independent scope review that leaves
no proposal without a public disposition. Its object is the versioned scope
map — the canonical source's declared axes at a named source version — and its
task is adversarial: to propose material omissions the map has not disposed
of. It is not a proof, a measurement, an authentication, or a vote, and it
carries no veto. It establishes at most that named independent reviewers
examined a named scope and that their proposals received reasoned public
dispositions.

## 2. The evidence contract

The canonical source (`full-society-ledger.json`) and its generator
(`13-full-society-ledger.py`) are the operative schema for review-event and
proposal records. This protocol binds what those fields mean in the review's
lifecycle and adds no field. If this document and the generator ever diverge,
the divergence is a defect to be repaired, not an ambiguity to be construed,
and the generator's refusal governs until the repair lands.

The semantic bindings:

- A **review event** records who reviewed — identity and discipline per
  reviewer — under which protocol version, over what received window, with
  what cut-off date, and how both of its controls ended. Its `protocol_ref`
  resolves to this document by the reference discipline the generator
  enforces, and the commissioning commitment pins the exact bytes of the
  version that governed the event.
- **Independence is a computation, not a self-label.** An event's
  independence flag may be true only where named reviewer identities, a
  resolvable protocol reference, and passed seeded and planted-omission
  controls are all present; the generator refuses the flag without them. An
  event that cannot compute independence records the flag false and says why
  in its outcome strings.
- A **proposal** records what was proposed, by whom, when it was received and
  when it was triaged, the materiality finding with its reason, and exactly
  one public disposition with reasons. The record obligations per disposition
  are section 4's subject.
- A failed or non-independent event is **recorded, not deleted**. The honesty
  of the record is part of the contract: a review that missed its plant is a
  fact about the review, and erasing it would be the exact defect the ledger
  exists to refuse.

## 3. Admissibility

1. **The independence test.** Named reviewer identities, a resolving protocol
   — this document, version-pinned at commissioning — and passed seeded and
   planted-omission controls. None of the three substitutes for another, and
   no self-description supplies any of them.
2. **The corpus exclusion.** The in-repo reviewer corpus (`reviews/`,
   `reviews.md`) is never admissible independent-review evidence for closure
   condition five — by doctrine and by computation: it has no named reviewer
   identities bound to this protocol, no received window, and no controls, and
   nothing it could add would supply them. It remains what the assurance
   portfolio already says it is: Reasoned design input, permitted in the
   exempt elements.
3. **The conflict bar.** A reviewer is inadmissible who authored or generated
   any reviewed artifact; who holds the event's severity-owner,
   independent-checker, or pre-image-custodian role; who is compensated
   contingent on findings, dispositions, or the event passing its controls; or
   who reviews under an identity the event record cannot carry. Compensation
   as such is no bar; contingency is.
4. **Window and cut-off discipline.** Every proposal's received date falls
   inside the event's received window. A finding arriving after the cut-off
   date is not a proposal of this event; it takes the reopening path of the
   full-society boundary decision, cited here rather than restated — review
   does not remain open forever, and reopening is that decision's rule to
   state, not this document's.
5. **The ethics of the controls.** The plant and the seeds test the
   machinery, never the reviewers as people. No reviewer is individually
   scored, ranked, or reported against for missing the plant or triaging a
   seed; control outcomes attach to the event, and the event as a whole passes
   or fails its controls.

## 4. The resolving protocol

Triage runs the same way for every proposal, seeded or genuine, because the
severity owner cannot know which is which until triage is done.

1. **Received.** The proposal enters the record with its received date, its
   source, and its text.
2. **Materiality finding.** The named severity owner finds the proposal
   material or immaterial under the confirmed severity rubric applied by
   reference — the rubric is `severity_rubric` in the canonical source, bound
   to `stopping_rule.materiality_test`, and this protocol cites both and
   paraphrases neither. A material finding carries a severity class of
   critical or material; minor is the editorial band inside the rubric, not a
   materiality escape hatch.
3. **Independent check.** A checker distinct from the severity owner reviews
   the classification. On an independent event the check is a resolvable
   record reference; on a non-independent event its absence is recorded with
   its reason, never fabricated.
4. **Public disposition.** Exactly one of the three ratified outcomes, with
   its record obligation:
   - **added** — records were created: the proposal names each created record
     by stable identity;
   - **classified out with reasons** — an outward classification carries the
     matching Unestablished disposition; only a duplicate or immaterial
     classification carries none, because those route nothing outward;
   - **retained as a limit** — retention creates or joins a stable defect
     row, and the severity, consequence, owner, closure condition, and
     public-claim limitation ride on that row.
5. **Dates and reasons throughout.** The record carries received and triaged
   dates plus reasons at every step.

Two sentences of the boundary decision govern the whole flow and are echoed
here exactly: "Reviewers compel a reasoned disposition, not automatic
acceptance and not an individual veto." And: "Classification is a disposition,
not assurance" — routing an item outward does not establish it.

**Role separation.** Whoever holds a pre-image may not triage. The pre-image
custodian may not act as the event's severity owner or its independent
checker, and both the severity owner and the checker must remain blind to
which received proposals are seeds until every seed's triaged date is
recorded. Custody is author-held outside the repository, so the author is
excluded from the severity-owner and independent-checker roles for any event
whose pre-images the author holds. The severity-owner designation itself
remains an author checkpoint; this protocol constrains who may hold the role
and designates nobody.

## 5. Reviewer selection, without naming

A reviewer must be named at commissioning. This protocol names none,
deliberately: naming a reviewer would begin implementation, and inventing one
would be worse.

The criteria a commissioning must satisfy, all of them structural rather than
numeric — this protocol sets no headcount, no panel size, and no
per-discipline quota:

- **Multidisciplinarity.** No single discipline supplies every reviewer, and
  the panel's declared disciplines must, between them, be competent to the
  ratified materiality test's full criterion list — declared rights,
  liberties, powers, duties, protected private boundaries, cross-domain
  dependencies, the ordinary-life account, failure and recovery paths, and
  the adequacy, accessibility/equality, continuity, resilience,
  sustainability, safety, and resource criteria.
- **Adversarial posture.** The review's task is to propose material
  omissions. The packet invites attack on the map's completeness, and a
  reviewer's success condition is a well-formed proposal, not agreement with
  the design.
- **Recorded identity.** Each reviewer is recorded in the event with identity
  and discipline, in the operative schema's shape. An anonymous review may be
  many things; it is not this route.
- **No disqualified role.** Section 3's conflict bar applies at selection,
  not only at adjudication.

## 6. The commitment scheme

The mechanics are landed in the canonical source now; the contents arrive only
at commissioning. Nothing in this section constructs a plant, a seed, or a
commitment.

**The pre-images.** At commissioning, the custodian constructs separate
pre-image files — a plant file and a seeds file — each a canonical UTF-8 file,
author-held outside the repository, never committed to it. Each file carries a
random nonce, so its digest cannot be dictionary-tested against candidate
contents, plus its construction date and custodian identity.

- The **plant file** states one genuine material omission from the canonical
  source at the commissioned version, precisely enough that a match is
  mechanical, together with pre-committed match criteria: what a reviewer
  proposal must contain to count as having found it.
- The **seeds file** states the full text of each seeded proposal — seeds on
  both sides of the materiality line — each with its expected materiality
  finding, its expected severity class where material, and its expected
  disposition class.

**The commitment.** Before the received window opens, the SHA-256 digests of
both pre-image files enter the canonical source's `review_protocol` record in
a dated commit, beside a digest of this document's exact bytes — the pin that
makes the review run under a hash-committed protocol, and the pin that stops a
later edit of this document from retroactively governing a running event. The
generator refuses an independent review event whose commitment is absent,
malformed, or dated after the window opened. Publishing the commitment is an
author act; this document defines the record it fills and fills nothing.

**Blindness.** The severity owner and the independent checker are blind to
seed identities until every seed's triaged date is recorded. Reviewers are
blind to the plant's content. The existence of one committed plant and of
seeds on both sides of the materiality line is public by this protocol — the
content is the secret, never the fact of the control.

**Reveal and adjudication.** At the cut-off date, or when triage of every
received proposal completes, whichever is later, the custodian publishes both
pre-images. Anyone may re-hash them against the published commitment. The
independent checker — never the custodian — applies the pre-committed match
criteria and the expected classifications, and the outcomes are recorded on
the event. The outcome strings follow this shape, each beginning with the word
the generator requires:

- planted-omission outcome: `passed — the committed omission was
  independently proposed before reveal; pre-image verified against the
  published commitment`;
- seeded-control outcome: `passed — every seed was triaged to its committed
  expected classification before reveal; pre-image verified against the
  published commitment`.

**After reveal.** Two obligations. The plant enters the proposal stream and
receives an ordinary public disposition like any material proposal — it is
genuine, so it cannot vanish, and if its checked classification is critical it
blocks the gate like any critical item; the control buys it no dispensation.
And each seed's record states its seeded origin in its source field, dated
after its triage, so the record is honest without the blindness ever having
been broken.

## 7. The review packet

The packet is the repository at the pinned commissioning commit, presented
through named artifacts at a named source version: the canonical source, its
rendered ledger projection, the constitutional coverage map, the full-society
boundary decision, the assurance-portfolio decision, and this protocol, with
the rest of the public repository reachable and none of it excluded.

The plant is withheld by construction, not by access control: it is a genuine
omission, so it appears nowhere in the repository, and its pre-image lives
outside it. The stopping rule's no-hiding rule still governs it. What is
sealed is the plant's content, never its existence — this protocol publicly
declares that one committed material omission is withheld for the duration of
the received window. The withholding is bounded by mandatory reveal at
adjudication; no closure record can exist before that reveal, because the
generator refuses a closure record until an independent event exists and
independence cannot compute true before the controls are adjudicated; and on
reveal the plant receives an ordinary public disposition.

## 8. The falsification condition

The route's declared falsification condition, restated with both failure
directions and the commitment's own failure mode:

- **Missed plant.** A review event whose reviewers do not independently
  propose the committed omission before reveal fails its planted-omission
  control.
- **Passed seeds.** An event whose triage classifies any seed away from its
  committed expected classification — material found immaterial, immaterial
  found material, or a wrong severity class — fails its seeded control.
- **Broken commitment.** A reveal that does not hash byte-exactly to the
  published commitment voids both controls. A control that cannot be verified
  is a control that failed.
- **Consequence, uniform.** A failed event keeps its independence flag false,
  its outcome strings state the failure, it is recorded rather than deleted,
  and it can never satisfy closure condition five. The revealed plant still
  receives its public disposition. The house rule carries over unchanged:
  sabotage first, trust after; a check that has never been watched failing is
  not yet a check.

Stating this ships nothing. The controls that watch the commitment machinery
fail are the generator's; the route remains unbuilt until a reviewer is named
and an event runs, and neither this section nor the landed mechanics makes it
available.

## 9. What this document does not do

It does not name a reviewer, designate a severity owner or an independent
checker, or select a custodian beyond recording that custody is author-held
outside the repository. It does not construct a plant or a seed, publish a
commitment, open a received window, or create any review-event or proposal
record. It does not remove any deferral record, make R7 built or available,
change the route's status, or move any Gate A readiness condition. It does not
upgrade any posture, reword any permitted-claim string or the rubric's class
definitions, or admit the in-repo reviewer corpus for any purpose beyond its
standing Reasoned role. It does not predict the review's outcome in either
direction, set a reviewer headcount or quota, add a schema field of its own,
or derive any count, score, or aggregate from review records.

## 10. Author checkpoints and the commissioning sequence

The commissioning sequence, in dependency order. Every step is an author
checkpoint or follows one, and each is discharged by a record elsewhere —
never by editing this document.

1. **Confirm this protocol.** The status line flips to its author-confirmed
   form, the confirmation record below fills, and the canonical source's
   protocol status flips in the same change — the generator holds the two in
   lockstep.
2. **Designate the severity owner and the independent checker**, under
   section 4's exclusions: neither may be the pre-image custodian, and
   author-held custody excludes the author from both roles. Discharged by the
   `severity_owner` and `independent_check` references material proposals will
   carry.
3. **Name the reviewers**, under section 5's criteria. Discharged by the
   event record's reviewer entries.
4. **Construct the pre-images and publish the commitment** — a dated commit
   entering both digests and this document's digest in the canonical source,
   before the received window opens.
5. **Issue the packet; run the window and the triage.**
6. **Reveal, adjudicate, and record the event**, flipping its independence
   flag only if both controls passed.
7. **The closure record**, far downstream, remains its own author
   ratification behind every other closure condition; nothing in this
   sequence reaches it.

**Confirmation record.** Confirmation follows the severity rubric's
convention: the author's statement is quoted verbatim beside the question it
answered, here, in this section. Author statement, 2026-08-09: "I confirm the
scope-review protocol — record the confirmation" — given in answer to whether
this document, landed as a candidate in 74d037e with its evidence contract,
admissibility criteria, resolving protocol, reviewer-selection criteria,
commitment scheme with author-held custody and its role exclusions, and
commissioning sequence, is confirmed as the protocol under which the
independent scope review will run. Every later step of the sequence above
remains undischarged: no reviewer is named, no severity owner is designated,
no commitment is published, and no review has run.

## 11. Evidence and limits

This document establishes nothing. It does not establish that any review will
run, that a review would pass its controls, that the protocol is valid —
nothing is valid before its control has been watched failing against a real
event — or that R7 will be built or become available. Closure condition five
computes unmet-external today and stays so until an independent event exists;
this document does not move it. A protocol does not prevent a capture or a
miss; it makes one legible — the commitment, the record, and the outcome
strings are the mechanism, and the record of a failed review is as much a
product of this protocol as the record of a passed one.
