<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Office Integrity Contract

> **Status: Specified contract card, session-drafted and author-approved
> 2026-09-09 under ruling C of
> `book-1-democratic-and-administrative-integrity-decision.md`, including the
> choices in section 11.**
> This card implements conflicts of interest, gifts, and revolving doors as one
> family of attested incompatibilities. It creates no register, no finding
> about any person, no amount, no period, and no claim that any office is
> clean.

## Implementation follow-up — 2026-09-12

The examined-kind clearance remains. The common
`book-1-integrity-record-contract.md` additionally supplies an independently
attested adverse finding keyed to the affected act. It names the office,
counterparty, interest kind, relationship kind and incompatibility mode,
within the supplied matter and legal period. It withholds act permission,
derives recusal and independent-alternate review/remedy duties, and expressly
refuses former-holder representation in the affected dealings. The private
counterparty is not made the duty-bearer.

`tests/pins/integrity/office-finding/` includes each named kind, self-review,
missing/stale records, equal treatment of governing and opposition holders,
an unrelated act, and former-holder representation. No personal standing,
floor, ballot or employment status is taken. The old clearance counts and
retired checker paths below remain history, not a verification workflow.

The reader projection is now the exact, separately author-approved section
"Keeping public decisions answerable" in Book 1 chapter 9 (2026-09-12).
The earlier chapter 3 candidate below is retained as history, not pending work
and not the wording approved for this implementation.

## 1. Decision

Every reviewed state-form result now attests that the office-integrity kinds
were examined and found absent, or it does not derive. The finding names the
material-interest kinds it examined, the counterparty relationships it
examined, and the incompatibility modes it covered, in the constitution, on the
appointment anti-capture pattern: each becomes a closed vocabulary of
constitutional constants concluded by exactly one ground rule, and the reviewed
result derives only when the three mutually distinct attesters carry a
vocabulary member for every kind and every mode. A finding that attests the
blanket anchor and nothing else derives nothing.

The bearer is the office, never the private party. The interest is the
holder's own, the household's, or a controlled entity's; the relationship is
the office's to the counterparty — regulated, contracting, adjudicated, or
appointed; the modes are the ruling's three incompatibilities — acting while
conflicted, a benefit from a counterparty, and a former holder's dealing with
a counterparty in the cooling period. Whether a benefit is *de minimis* and
how long the period runs are democratic law and appear nowhere in the source.

## 2. What already exists, and is extended rather than restated

The state-form ruling already requires every office's decisions to carry
"recusal rules, and an effective remedy," sends a challenge to the Court's own
composition or recusal to a predeclared alternate panel, and lists "conflict
and recusal rules" among every power's contract fields. FS-POW-030 already
carries a *conflicted-source* fallback for appointments. None of that made a
conflict observable: no rule read an interest, a counterparty, or a benefit,
and the words gift, conflict of interest, and revolving door occurred in no
ratified record until the 2026-09-09 ruling.

The family reuses the state-form generator whole. The reviewed JSON gains
eleven result-side fields on every branch; `src/checks/state_form.rs` renders
the ground rules, the `observe/4` premises, and the `member/2` gate from one
table of examined-kind families that the appointment anti-capture family
already occupied. The three-attester wall, the current-source rejoin, the
independent-review premise, and the holder-authority composition are unchanged
and are what the new conjuncts sit inside.

## 3. The measurement this card exists to repair

Measured at `0e553e6` before drafting: an appropriation, a court judgment, an
appointment, a licence, and a contract award each derived the holder's
authority from a record that attested nothing about the holder's interests,
the counterparty's relationship to the office, or any benefit passing between
them. Not a gap in a rule — no rule existed for the gap to be in. The
appointment anti-capture slice had already shown what a blanket token costs:
three attesters agreeing to repeat one constant is indistinguishable from
three attesters having examined anything.

## 4. The named vocabularies

| scope | vocabulary | members |
|---|---|---|
| `MaterialInterestKindScope` | `MaterialInterestKindVocabulary` | `OwnMaterialInterest`, `HouseholdMaterialInterest`, `ControlledEntityMaterialInterest` |
| `CounterpartyRelationshipKindScope` | `CounterpartyRelationshipKindVocabulary` | `RegulatedCounterparty`, `ContractingCounterparty`, `AdjudicatedCounterparty`, `AppointedCounterparty` |
| `OfficeIntegrityModeScope` | `OfficeIntegrityModeVocabulary` | `ConflictedAct`, `CounterpartyBenefit`, `FormerHolderCounterpartyDealing` |

The anchor is `NoConflictGiftOrRevolvingDoorIncompatibility` under
`OfficeIntegrityScope`. Each member's ground rule has the shape

```
all $source: all $record: observe($source, $record, HouseholdMaterialInterest, MaterialInterestKindScope) -> member(HouseholdMaterialInterest, MaterialInterestKindVocabulary).
```

and is read by every reviewed-result rule through `member(...)`. The
household kind reads the kinship vocabulary the 2026-08-02 batch fixed —
`parent`, `married`, `sibling` — and nothing wider; guilt by association is
refused by the ruling and is what the widening fixture in section 7 names.

## 5. Contract fields

The gates land on every reviewed-result rule, which every holder-authority
rule reads as a premise; the generator composes the authority rule from the
result premises, so all 273 rules physically carry the conjuncts. On each
result the family requires:

- the anchor, attested by `source`, `evidence`, and `review`;
- one absence attestation per material-interest kind — all three, because an
  unexamined kind is not an absent one;
- one absence attestation per counterparty relationship — all four, because a
  benefit from a party the office appoints is as disqualifying as one from a
  party it regulates;
- one coverage attestation per mode — all three, because the ruling names
  three;
- each kind and mode gated by its `member(...)` conjunct, so the vocabulary is
  read rather than declared and inert.

State-form branch fields are result-side only; the record and temporal record
carry record identity and nothing about a finding. Eleven fields × three
attesters × 273 rules is 9,009 added atoms, plus ten ground rules and 2,730
`member` conjuncts. The eleven hand-written economic pin files that supply
state-form records for the fiscal cards as dependencies carry the eleven
attestations in each of their 130 fixtures, added by script rather than by
exempting the cards a gift most concerns.

## 6. Formal migration contract

Nothing is replaced. `NoMajorityDirectOrDeFactoControl` and its seven kinds
keep their meaning and their branches. The office-integrity anchor is
**universal**: `validate_examined_kind_family` refuses a source in which any
branch omits it, so a branch cannot decline the family by leaving the anchor
off, and the watched self-control exempts each branch in turn and requires the
refusal. Every existing positive case must supply the eleven new attestations
to keep deriving; none passes unchanged, which is the point.

## 7. Executable cases

Measured 2026-09-09 with release `nibli-pin` against one generated FSPOW_007
appropriation case (`SFMainP008`, the fixture FSACC-020 reuses; 188 supplied
facts), asking both `authority(FSBOD_02, FSPOW_007, ·)` and
`complete(·, FSPOW_007, ·)`:

| case | supplied | result |
|---|---|---|
| positive control | all 188 facts | both **TRUE** |
| one kind unexamined | the three household-interest attestations removed (185 facts) | both **FALSE** |
| today's shape | all ten kinds and modes removed, blanket anchor still attested by all three (158 facts) | both **FALSE** |

The positive control is the load-bearing half; without it the two FALSE rows
would prove nothing.

Acceptance case `FSACC-020-office-integrity` pins the positive appropriation
result and four negatives — the anchor removed, then each vocabulary removed
in turn — so that each of the three vocabularies is shown to gate on its own.

Structural controls, both watched failing:

- `counterfactual/unnamed-office-integrity-kind` — the constitution plus one
  ground rule naming **party membership** as a material-interest kind, the
  guilt-by-association widening the ruling refuses by name. The producer-set
  check in `src/checks/repository.rs` must reject it, as the third entry in
  its widening-control table.
- `validate_examined_kind_family_self_controls` in
  `src/checks/state_form.rs` — drops each named kind and mode from each branch
  in turn, and separately drops the anchor from each branch inside the full
  population, requiring the completeness rule to refuse every one.

No gate-removal fixture, for the reason the anti-capture card recorded: the
kinds are fixed constants in the rule, so stripping the `member` conjuncts
changes nothing a probe could observe; what they buy is held by the
self-controls and by the repository check that refuses a source in which no
rule reads `member`.

## 8. What this does not establish

It does not establish that any holder is free of conflict, that any gift was
refused, that any former holder stayed away, that any interest register
exists or is complete, that any recusal happened, that any challenge is filed
or heard, or that any remedy completes. It authenticates no attester, proves
no attestation true, reads no amount, counts no days, and advances no clock.
Every state-form power remains `ratified-unimplemented` / `Specified`; this
card changes what a finding must name and what the source can refuse.

It does not bind procurement, which stays Book 2 under ruling E; the office
holder who awards a contract is bound here because that is an office act. It
does not reach the private counterparty, which the money-and-influence family
(ruling A) and FS-POW-064 reach. It does not create a probity score, a
register of associations, or any reading of an interest into standing, the
floor, the ballot, or an allocation.

## 9. Assurance posture

Derived, for the exact bounded claim that no reviewed state-form result and no
holder authority follows from an attestation that fails to name every
ratified interest kind, counterparty relationship, and incompatibility mode.
Safety claim, supplied-records scope bound, with the enumerated non-extension
clause in section 8.

## 10. Evidence

- `new-book-plans/state-form-source.json` — eleven fields on each of 131
  branches; the only hand-edited member of the family.
- `new-book-plans/constitution.nibli` — the regenerated `STATE-FORM-RULES`
  block: 291 statements.
- `new-book-plans/state-form.pins.nibli` — 396 main pins, including FSACC-020.
- `new-book-plans/counterfactual/unnamed-office-integrity-kind.{,pins.}nibli` —
  listed on the repository guard's control table, not on FS-CVF-003's
  `counterfactual_refs`, which is a checker-owned policy surface; the
  anti-capture fixture sits the same way.
- `new-book-plans/economic-power-{062,071,072,073,074,075,076,077,084,085,088}.pins.nibli`
  — dependency fixtures extended.
- `src/checks/state_form.rs`, `src/checks/repository.rs`, `src/checks/pins.rs`.

One named evidence gap, not closed here: FS-CVF-003's `formal_statement_refs`
carried 274 digests against a 281-statement block before this batch and
carries the same 274 against 291 after it. The ledger checks that list only
for non-emptiness; binding it to the rendered block is verifier coverage with
a named defect and belongs to its own small batch.

## 11. Choices made under the ratified text, for approval

1. **Universal scope.** Every branch carries the family, not only the
   adjudicative and appointive ones, because every branch is an office act and
   the ruling's bearer is the office. The alternative — a curated subset — was
   priced and refused: it would have excluded the fiscal cards, where a gift
   bites hardest, to save a mechanical edit.
2. **One counterparty vocabulary serves gift and revolving door.** The
   relationship that makes a benefit disqualifying is the relationship that
   makes a post-office move disqualifying; naming it twice would invite the
   two lists to drift.
3. **Three modes as constants**, mirroring the two anti-capture control
   modes, rather than three separate anchors: one absence claim, three named
   things it must cover.
4. **The economic dependency fixtures were extended by script**, 130 fixtures
   in eleven files, rather than exempting FSPOW_005, 006, and 007.
5. **The widening fixture names party membership.** It is the exact category
   the ruling refuses — guilt by association — so the watched control names
   the harm rather than an arbitrary token.
6. **No derived prose in this batch.** The anti-capture batch set the
   precedent: the family narrows what an existing, already-projected family
   will accept, and its reader-facing sentence belongs with the disclosure
   family (ruling B), which is what a reader would see. A paragraph for
   `book-1/03-who-holds-the-pen.md` is drafted below for approval; if
   approved it lands in the disclosure batch with its prose reference.
7. **FS-CVF-003's statement digests are left as found**, with the gap named
   in section 10, rather than regenerated under a formula the checker does
   not bind.

Earlier candidate, superseded by the approved chapter 9 section (would follow "A positive
result for that exact power then yields only the holder's current lawful
authority."):

> The same result must also say what it looked for. An office holder acting on
> a matter in which they, their household, or a body they control has an
> interest; a benefit passing from a party the office regulates, contracts
> with, adjudicates, or appoints; a former holder dealing with such a party in
> the period the law sets — each is a legal incompatibility, and the record
> derives the holder's authority only when independent reviewers attest that
> every one of those kinds was examined and none was found. A record that
> attests none of them, however many signatures it carries, derives nothing.
> What the rules cannot do is see an interest nobody wrote down.
