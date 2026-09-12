<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Money and Influence Contract

> **Status: Specified contract card, session-drafted 2026-09-09 under ruling A
> of `book-1-democratic-and-administrative-integrity-decision.md`; awaiting
> author approval of this exact text including the choices in section 11.**
> This card makes the money-and-influence finding consequential. It creates no
> register, no publication duty, no amount, and no claim that any payment is
> visible to anyone.

## Implementation follow-up — 2026-09-12

The deferred identity batch is implemented. State-form results bind
`$payer` in `PoliticalFinancePayerScope` and `$controlling_payer` in
`PoliticalFinanceControllingPayerScope`; the source, evidence writer and
reviewer must agree on each. The release-mode `payer_identity_cost_probe`
measured the original candidacy fixture at 24.70 ms and the identity-bound
fixture at 21.70 ms. This is a single-case probe, not a whole-book timing claim.
The probe is retained as an explicitly ignored development benchmark.

The common `book-1-integrity-record-contract.md` also supplies a positive
adverse-finance finding naming nominal and controlling identities, payer,
instrument and recipient kinds, the affected recipient/act, and an established
prohibited combination. Its incompatibility withholds **permission for that
act**, not the permanent answerability described by one-place `authority`.
Kinds do not adjudicate a payment by themselves, and no amount implies guilt.

`tests/pins/integrity/payer-identity/` checks missing and mismatched identities.
`money-finding/` covers the kinds, shell-control omission, independent review,
an unrelated act, the stateful adverse consequence, and the protected ballot.
The canonical semantic generators are now `src/authoring/state_form.rs` and
`src/authoring/integrity.rs`. Counts, old paths and administrative procedures
below describe the earlier kinds-only batch; they impose no current gate.

## 1. Decision

Every reviewed state-form result now attests that the political-finance kinds
were examined and no prohibited combination was found, or it does not derive.
The finding names the payer kinds it examined, the instrument kinds it
examined, and the recipient kinds it covered, in the constitution, as closed
vocabularies of constitutional constants concluded by exactly one ground rule
each and read by the reviewed-result rule through `member/2`.

The consequence is the ruling's own: a finding that fails derives neither the
reviewed result nor the holder's authority, so **public answerability for the
office or candidacy is withheld**. That is what makes the three inherited
treasury barriers catchable for the first time. Today each of FS-CCE-266,
FS-CCE-268, and FS-CCE-367 is one rule of the form
`person($x) -> prevents($x, <Constant>)` — a named barrier that reads no
treasury, no payment, no candidate, and no party. They remain, unchanged, as
the person-held protection; this card supplies the finding that can fail.

`ControllingPartyPayer` is the shell-actor test and it is why the family is
conjunctive: a finding that names the nominal payer and never asks whether
that payer is controlled by another has not examined the question, and an
unexamined kind is not an absent one.

## 2. What already exists, and is extended rather than restated

The economic ruling already bans enterprise-treasury electoral funding and
already promised that "a later Class 5 card must govern their political
finance, prevent relabeling from evading the enterprise rule, and preserve
equal ballots and individual rights." **This card is that promised Class 5
card.** It keeps the ruling's own distinction: an enterprise treasury, a
union, and a genuinely voluntary civic or advocacy association are three
separate payer kinds, so a relabelling is a change of attested kind rather
than an escape, and no union is silently classified as an enterprise.

The family reuses the state-form generator whole, as the office-integrity
family (ruling C) did before it. The reviewed JSON gains seventeen result-side
fields on every branch; `src/checks/state_form.rs` renders the ground rules,
the `observe/4` premises, and the `member/2` gate from the same
`EXAMINED_KIND_FAMILIES` table that already carries appointment anti-capture
and office integrity. The three-attester wall, the current-source rejoin, the
independent-review premise, and the holder-authority composition are unchanged.

**Why this family sits in the state-form machinery rather than the economic
machinery.** The ruling says the finding withholds authority "on the same
gating idiom that withholds it for majority appointment control", and that
conclusion is the state-form one. The economic families are hand-written
per-power rules validated by `src/checks/ledger.rs` against reviewed
`economic_power_rule_contracts`, and a new economic power would additionally
need a row in the 237-row source census, which contains none for political
finance. Nothing is moved across the seam: the three barrier effects stay
exactly where they are, in FS-CVF-017.

## 3. The measurement this card exists to repair

Measured at `c0ca1ca`: an appropriation, an appointment, a candidacy, and a
seat allocation each derived the holder's authority from a record that
attested nothing about who funded the office-holder or candidate, by what
instrument, or on whose behalf. The three barrier constants were the design's
entire answer to money in politics, and each one is a single rule reading no
payment. The words campaign, lobbying, party finance, and political
advertising occurred in no ratified decision record before the 2026-09-09
ruling.

## 4. The named vocabularies

| scope | vocabulary | members |
|---|---|---|
| `PoliticalFinancePayerKindScope` | `PoliticalFinancePayerKindVocabulary` | `PersonPayer`, `EnterpriseTreasuryPayer`, `UnionPayer`, `VoluntaryCivicAssociationPayer`, `OutsideJurisdictionPayer`, `ControllingPartyPayer` |
| `PoliticalFinanceInstrumentKindScope` | `PoliticalFinanceInstrumentKindVocabulary` | `ContributionInstrument`, `ExpenditureInstrument`, `InKindProvisionInstrument`, `LoanOrGuaranteeInstrument`, `PurchasedIndependentAdvocacyInstrument` |
| `PoliticalFinanceRecipientKindScope` | `PoliticalFinanceRecipientKindVocabulary` | `CandidateRecipient`, `PartyOrCoalitionRecipient`, `BallotQuestionRecipient`, `OfficeHolderRecipient`, `PublicDecisionRecipient` |

The anchor is `NoProhibitedPoliticalFinanceIncompatibility` under
`PoliticalFinanceScope`. The payer list is the economic ruling's own
categories plus the two the integrity ruling adds — an out-of-jurisdiction
source and a controlling party. No amount, threshold, publication floor, or
reporting period appears anywhere in this family; all four are democratic law.

## 5. Contract fields

The gates land on every reviewed-result rule, which every holder-authority
rule reads as a premise, so all 273 rules physically carry the conjuncts. On
each result the family requires the anchor plus one attestation per payer
kind, per instrument kind, and per recipient kind — all sixteen, each gated by
its `member(...)` conjunct — from `source`, `evidence`, and `review`, which
the rule already holds mutually distinct and separately authorised.
Seventeen fields × three attesters × 273 rules is 13,923 added atoms, plus
sixteen ground rules and 4,368 `member` conjuncts.

**What the family does not carry, stated plainly.** These are *kinds*, not
identities. The finding must attest which kinds it examined; it does not name
the payer. The recipient is already bound — every rule is about one office's
result record — but the payer is not. The author ruled on 2026-09-09 that the
payer binding lands as its own measured batch rather than here, so until it
does, a finding can say that an enterprise treasury funded this office and
cannot say which one.

## 6. Formal migration contract

Nothing is replaced. The anti-capture and office-integrity families keep their
anchors, their kinds, and their branches. The political-finance anchor is
**universal**, like office integrity: `validate_examined_kind_family` refuses a
source in which any branch omits it, and the watched self-control exempts each
branch in turn and requires the refusal. Every existing positive case must
supply the seventeen new attestations to keep deriving.

## 7. Executable cases

Measured 2026-09-10 with release `nibli-pin` against one generated FSPOW_036
candidacy case (the fixture FSACC-021 reuses; 257 supplied facts), asking both
`authority(FSBOD_06, FSPOW_036, ·)` and `complete(·, FSPOW_036, ·)`:

| case | supplied | result |
|---|---|---|
| positive control | all 257 facts | both **TRUE** |
| shell actor unexamined | the three `ControllingPartyPayer` attestations removed (254 facts) | both **FALSE** |
| today's shape | all sixteen kinds removed, blanket anchor still attested by all three (209 facts) | both **FALSE** |

The second row is the shell-actor test executing: a finding that names its
nominal payer and never asks whether that payer is controlled by another
derives neither the result nor the authority. The positive control is the
load-bearing half — without it the two refusals would prove nothing.

Acceptance case `FSACC-021-political-finance` pins the positive candidacy
result and four negatives — the anchor removed, then each of the three
vocabularies removed in turn — so each vocabulary is shown to gate on its own.

Structural controls, both watched failing:

- `counterfactual/unnamed-political-finance-payer` — the constitution plus one
  ground rule naming an **anonymous intermediary** as a ratified payer kind:
  the exact widening that would let the shell-actor test be satisfied by a
  category the ruling does not name. The producer-set check in
  `src/checks/repository.rs` must reject it, as the fourth entry in its
  widening-control table.
- `validate_examined_kind_family_self_controls` in `src/checks/state_form.rs`
  — drops each kind from each branch in turn and separately drops the anchor
  from each branch inside the full population, requiring refusal every time.

## 8. What this does not establish

It does not establish that any payment occurred, that any register exists or
is complete, that any payer was identified, that any shell was pierced, that
any writer is independent, that any challenge is heard, or that any
disqualification is executed. It reads no amount, compares no sums, and ranks
nobody. It authenticates no attester and proves no attestation true.

It does not reach universal standing, the material floor, the equal ballot,
core liberties, due process, or remedy — a person who paid, or was paid, keeps
every right the corridor protects. It creates no publication duty and no donor
register; the ruling refuses a register that is itself a re-identification
surface, and proportionate publication remains democratic law under the
equality-diagnostics data wall. It does not reverse the economic ruling:
enterprise petition, testimony, and attributed publication stay lawful.

## 9. Assurance posture

Derived, for the exact bounded claim that no reviewed state-form result and no
holder authority follows from an attestation that fails to name every ratified
payer kind, instrument kind, and recipient kind. Safety claim, supplied-records
scope bound, with the enumerated non-extension clause in section 8.

## 10. Evidence

- `new-book-plans/state-form-source.json` — seventeen fields on each of 131
  branches; the only hand-edited member of the family.
- `new-book-plans/constitution.nibli` — the regenerated `STATE-FORM-RULES`
  block: 307 statements.
- `new-book-plans/state-form.pins.nibli` — including FSACC-021.
- `new-book-plans/counterfactual/unnamed-political-finance-payer.{,pins.}nibli`.
- The eleven `economic-power-0nn.pins.nibli` and eleven
  `counterfactual/no-economic-independent-current-review-0nn.pins.nibli`
  dependency fixtures, extended.
- `src/checks/state_form.rs`, `src/checks/repository.rs`, `src/checks/pins.rs`.

## 11. Choices made under the ratified text, for approval

1. **Kinds, not identities — the one choice worth your attention.** The
   ruling says the record is "attested at source grain: who paid or influenced,
   by what kind of instrument, to what kind of recipient". This card
   implements the *kinds* and leaves the payer unnamed. Adding
   `["$payer", "PoliticalFinancePayerScope"]` as a bound variable field would
   make all three attesters agree on a payer identity, which is closer to the
   ruling's words and is what a shell-actor finding would cite. It is deferred
   rather than refused because it adds a free variable to 273 rules whose
   effect on the three-hour gate I have not measured, and a wrong guess costs
   a full run. **Ruled 2026-09-09: kinds now, identity as its own
   measured batch.** The follow-up must probe the free-variable cost against
   one generated case before it freezes anything. That follow-up was tracked
   in `TODO.md` and completed on 2026-09-12; the implementation note above
   records its measured result and both identity bindings.
2. **Universal scope**, as for office integrity: every branch is an office or
   candidacy act, and the fiscal cards are exactly where the exclusion would
   hurt.
3. **This family sits in state-form, not economic machinery** — section 2
   gives the two reasons, and the three barrier effects do not move.
4. **`ControllingPartyPayer` is a payer kind rather than a separate
   vocabulary**, so the shell test is a conjunct of the same absence claim
   rather than a second claim a finding could satisfy separately.
5. **The widening fixture names an anonymous intermediary**, because that is
   the category whose admission would hollow out the shell-actor test.
6. **No derived prose in this batch**, on the anti-capture and office-integrity
   precedent; the reader-facing sentence belongs with the disclosure family
   (ruling B), which is what a reader would see.
