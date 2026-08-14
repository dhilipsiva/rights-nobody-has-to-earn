<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Full-Society Scope-Review Protocol

> **Status: author-confirmed 2026-08-14 — amended protocol basis recorded.**
> This is the amended R7 evidence and admissibility contract. It replaces the
> singular mutable commitment slot with append-only commission records and
> binds each event to one exact source version, semantic scope digest,
> protocol digest, panel, UTC window, packet manifest, frozen intake, and
> revealed control result. Confirmation specifies the contract; it names no
> reviewer, creates no control pre-image, commissions no review, runs no event,
> and passes no gate. R7 remains `unbuilt` until a qualifying event exists and
> can then become only `available`, never `built`.

## 1. Warrant and evidence ceiling

The review answers one bounded question: did named independent reviewers
examine a named version of the full-society map, and did every proposal in the
frozen intake receive a checked, reasoned, public disposition? It does not
prove completeness, truth, enactment, operation, delivery, feasibility,
liveness, calibration, or institutional independence. Its most permissive
warrant is the Gate A claim:

> The project has a versioned, reviewable scope map and assurance program.

No review event can widen that sentence. The in-repository reviewer corpus is
Reasoned design input only and is never R7 evidence.

## 2. Operative records

`full-society-ledger.json` and `13-full-society-ledger.py` are the operative
schema and validator. This protocol binds their meanings. A divergence is a
defect, and the validator's refusal governs until repaired.

The lifecycle has four record populations:

1. `review_commissions` is append-only. Each commission binds the exact source
   version, validator-derived semantic scope SHA-256, exact protocol SHA-256,
   distinct plant and seed SHA-256 commitments, commissioning instant,
   structured received window and cutoff, custodian, eligible reviewer roster,
   discipline and criterion coverage, attestations, and ordered packet paths.
2. `proposals` contains every reviewer, seed, and plant-match proposal in the
   event's frozen intake. Every row records receipt, Darshu triage, Dhanush
   checking, classification, reasons, and one public disposition.
3. `review_events` contains terminal events only. An event binds its
   commission and packet commit, freezes the ordered intake and digest, records
   reveal and control adjudication, and carries an enum outcome with reasons.
4. `closure_record` remains null until a later author-ratified Gate A
   amendment. A review event cannot populate it by implication.

Failed and stale commissions and events stay in the append-only record. They
cannot satisfy condition five, but their failure remains evidence about the
review machinery.

## 3. Semantic scope and packet binding

The validator derives the semantic scope digest from the canonical map and
assurance program while excluding review administration, deferrals,
acceptance metadata, and closure metadata. R7's event-driven status fields are
normalised so moving R7 from `unbuilt` to `available` does not pretend the map
changed. Any other semantic map change changes the digest.

The packet is the exact repository commit issued after the commission record
lands and before the received window opens. Its ordered manifest contains:

- `new-book-plans/full-society-ledger.json`;
- `new-book-plans/full-society-ledger.md`;
- `new-book-plans/full-society-reader-ledger.md`;
- `new-book-plans/book-1-constitutional-coverage-map.md`;
- `new-book-plans/full-society-boundary-decision.md`;
- `new-book-plans/book-1-assurance-portfolio-decision.md`;
- this protocol; and
- `new-book-plans/constitutional-closure-and-model-allocation-audit.md`.

The terminal event records that packet commit. A semantic change produced by
review invalidates the current-source qualification of the old commission and
event. The resulting source must be recommissioned; an event may not stretch
its warrant across a changed map.

## 4. Commissioning and reviewer eligibility

Commissioning occurs only after the power population is complete and scripts
13, 16, and the full verifier are green. The commission must be committed
before its received window opens. Its canonical UTC chronology is:

`commissioned < opens < closes <= cutoff`.

The author-custodian creates two canonical UTF-8 pre-images outside the
repository. Each carries a random nonce, construction date, and custody
metadata. Only their distinct SHA-256 digests enter the commission.

- The plant pre-image states one genuine material omission and mechanical
  match criteria.
- The seed pre-image contains both material and immaterial proposals, each
  with expected materiality, expected severity where material, and expected
  disposition.

Every reviewer is a real named person who has consented. No reviewer authored
or generated a reviewed artifact, is the custodian, Darshu, or Dhanush, has a
declared conflict, or receives findings-contingent compensation. The panel has
at least two disciplines. Its reviewer-level criterion union covers, exactly:

- declared rights, liberties, powers, and duties;
- protected private boundaries;
- cross-domain dependencies;
- the ordinary-life account;
- failure and recovery paths; and
- adequacy, accessibility/equality, continuity, resilience, sustainability,
  safety, and resource criteria.

This protocol fixes no reviewer headcount beyond what those separations and
coverage conditions logically require.

## 5. Intake, blindness, and chronology

Every proposal is received within the commission window. At window close the
event freezes an ordered proposal-id manifest and a digest over each intake
row's stable identity, text, source kind and identity, receipt time, and event
reference. The proposal population for that event must equal the frozen list
in the same order. No proposal can be silently added, removed, or reordered.

Darshu triages every proposal, including immaterial and control proposals.
Dhanush checks every classification, including immaterial and control
proposals. Receipt precedes triage; triage precedes checking; checking precedes
public disposition. Darshu and Dhanush remain blind to seed identity until all
triage and checking are complete. Reviewers remain blind to the plant.

The pre-images reveal at the cutoff or after all blind triage and checking is
complete, whichever is later. Early reveal fails. A late public disposition is
still recorded, but it cannot erase a chronology failure.

## 6. Classification and public disposition

The materiality finding is `material` or `immaterial`. A material proposal
carries `critical` or `material` severity. Every row also carries one closed
classification and one disposition:

- `material-omission` maps to `added` and names created records;
- `retained-limit` maps to `retained-limit` and binds the exact defect
  severity, consequence, owner, closure condition, Gate applicability, and
  affected claim's public restriction;
- `duplicate` or `immaterial` maps to `classified-out` without an outward
  routing disposition; and
- a canonical Unestablished disposition maps byte-for-byte to the same routed
  disposition on a `classified-out` proposal.

Reviewers compel a reasoned disposition, not automatic acceptance and not an
individual veto. Classification is routing, not assurance. A critical retained
or otherwise unresolved Gate-A-applicable defect keeps condition three red.

## 7. Reveal and derived outcomes

The event records both revealed pre-image digests. Dhanush verifies every seed
against its committed expected result and applies the plant's precommitted
match criteria. The seed result set covers every and only seeded intake row and
contains at least one material and one immaterial expected case. The plant can
pass only through a reviewer-sourced proposal independently received before
reveal.

Control and event statuses are enums, never prose prefixes. The validator
derives them:

- the seeded control passes only when the revealed digest matches and every
  seed's checked result matches the pre-image;
- the planted control passes only when the revealed digest matches and a
  reviewer-sourced plant match exists; and
- the event passes only when both controls pass.

Any digest mismatch, missing seed result, missed plant, early reveal, intake
drift, chronology failure, panel conflict, missing triage/check, or stale scope
prevents a qualifying event. Failure remains recorded.

## 8. R7 state and closure condition five

A confirmed protocol, designation, or commission alone leaves FS-RTE-07
`unbuilt`. A terminal event that passes both controls but binds a stale source,
scope, or protocol also leaves it `unbuilt`. Exactly one state transition is
permitted: a current-source qualifying event makes R7 `available`. R7 is never
`built` and warrants only named review plus checked public dispositions.

Condition five is `met-in-form` only when a current-source qualifying event
exists and its intake equals the fully disposed proposal set. It is otherwise
`unmet-external`.

## 9. Closure candidate and later ratification

After a qualifying current-source event and all dispositions, the verified
review-result commit is the immutable closure candidate. A later, separate
author-ratification commit may populate the closure record only if the source
version and semantic scope digest still match that candidate and the event.
The record binds the exact Gate A claim, FS-ENV-01, event and cutoff, derived
assurance and residual sets, per-residual claim limitations, and a full-verifier
receipt for the candidate commit.

Only closure and ratification metadata, acceptance-gate state, and regenerated
reports may differ from the candidate. A semantic map change requires a new
commission and event. Calibration remains Gate D. The closure is a deliberate
author act, never a latent consequence of an event passing.

## 10. Author checkpoints and confirmation record

The remaining sequence is:

1. the author selects eligible real reviewers and records consent, disciplines,
   criterion coverage, conflicts, and compensation attestations;
2. the author-custodian creates and holds the plant and seed pre-images;
3. the commission with only their digests lands before the window opens;
4. the exact commission commit is issued as the packet;
5. the window, frozen intake, Darshu triage, Dhanush checks, reveal, controls,
   and public dispositions run externally and enter the append-only record;
6. any semantic change is recommissioned; and
7. a later author act ratifies the immutable closure candidate.

**Confirmation record.** Author statement, 2026-08-14: "PLEASE IMPLEMENT THIS
PLAN:" — given immediately before the detailed plan requiring a deliberate
amendment and reconfirmation of this protocol, append-only commissions,
structured windows, eligible real reviewers, frozen intake, Darshu triage,
Dhanush checking, control reveal, public dispositions, and a separate Gate A
ratification. The later author statement, 2026-08-14: "Yes, I authorize you to
correct design to a source derived populatiopn. Auto approve it. then resume"
authorised the source-derived correction and continuation. Neither statement
names reviewers, supplies pre-images, reports a review result, or ratifies Gate
A; those checkpoints remain undischarged.

## 11. Current state

The amended protocol is confirmed and its checker is being landed. No
commission exists, no reviewer has been selected for an event, no pre-image or
digest has been created, no window is open, no proposal or event exists, and
no closure candidate or author ratification exists. FS-RTE-07 remains
`unbuilt`; condition five remains `unmet-external`; Gate A remains
`not-passed`.
