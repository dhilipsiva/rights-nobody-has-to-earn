<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Appointment Anti-Capture Contract

**Status: Specified contract card, session-drafted and author-approved
2026-09-08, including the choices in section 11.** This is not a new author
ruling. It is the contract card for the named source kinds and control modes of
the appointment anti-capture incompatibility under the ratified state-form
settlement of 2026-08-07 — its appointments section's incompatibility sentence
and its immediately following instruction that future body cards make both
direct and de facto appointment control observable. It reopens neither. Where it
chooses between shapes that ruling leaves open, the choice is named in section
11 and stands only once approved. Approval of this card is not a landing:
nothing is landed until it is drafted into the source and carried through one
`--emit-receipt` candidate.

## 1. Decision

The anti-capture finding names the source kinds it examined and the control
modes it covered, in the constitution, rather than accepting a single token
three attesters agree to repeat. The appointment-control source kind and the
appointment-control mode each become a closed vocabulary of constitutional
constants, and the incompatibility conclusion derives only when the finding
carries a vocabulary member for every kind the ruling enumerates and for both
modes it names.

The ruling's second sentence — "Divided sources alone do not prove that the same
coalition does not control them" — becomes executable for the first time: a
finding that attests divided sources and nothing else derives nothing.

## 2. What already exists, and is extended rather than restated

- **FS-POW-028** (independent-body appointment selection) and **FS-POW-035**
  (court and oversight seat allocation) are the two cards carrying the
  attestation today, across 24 rule lines. Both are
  `ratified-unimplemented` / `Specified`. Their selection, seat, selector,
  qualification-authority, and fallback structure is untouched by this card.
- **`CompletedDividedSourceSelection, SelectionDispositionScope`** (8 rule
  lines) stays exactly as it is. This card does not weaken it; it stops it from
  being mistaken for the anti-capture finding.
- **`FormalGovernmentAppointmentSeparate, AppointmentBoundaryScope`** (3 rule
  lines) stays as it is.
- **FS-POW-027, 029, 030** (qualification review, cause-only removal, vacancy
  and capture fallback) are not touched. The vacancy-veto protection the ruling
  requires is theirs, not this card's.
- The `member/2` machinery landed at `733e5fb`: corpus `cmima`,
  `derived_only("member")`, not admitted, ground-headed rules only, with the
  producer-set check in `src/checks/repository.rs`. This card adds vocabularies
  to that existing machinery and adds no new relation.

## 3. The measurement this card exists to repair

Measured 2026-09-08 at `83dd301`.

The ruling enumerates five source kinds. The formal source names none of them:

| ratified source kind | constitutional constant today |
|---|---|
| one current government | none — the `*Government*` constants carry tier, caretaker, and appointment-branch senses |
| one chamber | none — `BothChambersParticipate` is a procedure-participation constant, unrelated sense |
| one party coalition | none — `BoundPartyScope`, `EconomicBoundPartyBinding`, `ThirdPartyReproductiveVetoRefusal` are unrelated senses of "party" |
| one profession | none |
| one appointing source | none |

The entire incompatibility is carried by one fixed token:

```
observe($source,   $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
observe($evidence, $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
observe($review,   $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
```

`AntiCaptureScope` has exactly one inhabitant. All three atoms are result-level;
neither is mirrored on the record or the temporal record. "Direct and de facto"
appears only inside the constant's own spelling, where no rule can read it.

**Consequence, stated plainly.** A finding that examined one appointing source,
found it divided, and never looked at the coalition passes today by attestation.
That is the precise case the ruling's second sentence was written against, and
it is the case the current source cannot refuse.

This is the same defect shape repaired for FS-POW-064 at `733e5fb` — a ratified
sentence that enumerates categories, formalized as one unanalysed attestation.

## 4. The named vocabularies

**`AppointmentControlSourceKindVocabulary`** — five members, one per ratified
source kind, quoted from the ruling and not invented:

- `CurrentGovernmentAppointmentSource`
- `ChamberAppointmentSource`
- `PartyCoalitionAppointmentSource`
- `ProfessionAppointmentSource`
- `SingleAppointingBodyAppointmentSource`

**`AppointmentControlModeVocabulary`** — two members, from the ruling's
"both direct and de facto appointment control observable":

- `DirectAppointmentControl`
- `DeFactoAppointmentControl`

Each member is a ground-headed rule in the landed idiom, e.g.

```
all $source: all $record: observe($source, $record, PartyCoalitionAppointmentSource, AppointmentControlSourceKindScope) -> member(PartyCoalitionAppointmentSource, AppointmentControlSourceKindVocabulary).
```

Seven ground rules total. `member` remains `derived_only` and unadmitted, so no
supplied fact can widen either vocabulary; widening requires a visible source
edit, which is what the producer-set check watches.

## 5. Contract fields

The gates land on the reviewed-result rule, which every holder-authority rule
reads as a premise; because the generator composes the authority rule from the
result premises, both carry them. The anti-capture conclusion requires, on the
result:

- one absence attestation per source kind — all five, because control by any one
  is disqualifying and an unexamined kind is not an absent one;
- one coverage attestation per control mode — both, because the ruling says
  both;
- each gated by its `member(...)` conjunct, so the vocabulary is read rather
  than declared and inert.

**Correction to the shape this card first assumed, 2026-09-08.** The FS-POW-064
record-level mirroring idiom does *not* transfer to this family, and asserting
it would have been wrong. State-form branch fields are result-side only; the
record and temporal record carry record identity — source family, version,
epoch, power, jurisdiction, legal scope — not finding content. This family's
wall is instead that each field is observed by `source`, `evidence` and `review`,
which the rule already holds mutually distinct and separately authorised. Seven
fields × three attesters × 24 rules is 504 added atoms, plus seven ground rules.

## 6. Formal migration contract

`NoMajorityDirectOrDeFactoControl` is **retained** beside the new conjuncts
rather than replaced (section 11, choice 5). It keeps its current meaning as the
finding's headline disposition; it stops being sufficient. Every existing
positive case must supply the seven new attestations to keep deriving, so no
current pin passes unchanged — which is the point, and is why the batch is
large.

## 7. Executable cases

Measured 2026-09-08 against one generated FSPOW_028 positive case
(`SFMainP040`, 194 supplied facts), asking both
`authority(FSBOD_01, FSPOW_028, ·)` and `complete(·, FSPOW_028, ·)`:

| case | supplied | result |
|---|---|---|
| positive control | all 194 facts | both **TRUE** |
| one kind unexamined | party-coalition attestations removed | both **FALSE** |
| today's shape | all seven named grounds removed, blanket anchor still attested by all three | both **FALSE** |

The positive control is the load-bearing half: without it the two FALSE rows
would prove nothing, which is how a negative case passes for the wrong reason.

Structural controls, both watched failing:

- `counterfactual/unnamed-appointment-control-source` — the constitution plus one
  ground rule naming a source kind the ruling does not name. The producer-set
  check in `src/checks/repository.rs` must reject it, on the
  `unnamed-public-scale-trigger` precedent.
- `validate_appointment_anti_capture_self_controls` in
  `src/checks/state_form.rs` — drops each named kind and each control mode from
  each anchored branch in turn, and requires the completeness rule to refuse
  every one. This is what stops the ratified set being narrowed one branch at a
  time.

**No gate-removal fixture, and the reason is recorded rather than omitted.** The
kinds and modes are fixed constants in the rule, not variables, so stripping the
`member` conjuncts changes nothing a probe could observe. What the conjuncts buy
is that deleting a vocabulary ground rule stops the family deriving; the
self-controls above hold that, and `src/checks/repository.rs` separately refuses
a source in which no rule reads `member` at all.

## 8. What this does not establish

It does not establish that any body is independent, that any concentration is
detected, that any challenge is filed or heard, that any remedy completes, or
that capture becomes impossible. The ruling says so in its own words —
"This architecture reduces and exposes capture; it does not claim that capture
becomes impossible" — and that sentence's register governs any prose derived
from this card. It authenticates no attester, proves no attestation true, and
advances no clock. FS-POW-028 and FS-POW-035 remain
`ratified-unimplemented` / `Specified` afterwards; this card changes what the
source can refuse, not what the world does.

It does not close FS-SCN-24 or FS-SCN-70, which are `reviewed-inventory` records
with a maximum posture of Checked and are never a counterexample harness. It
discharges the formalization half of their closure condition.

## 9. Assurance posture

Derived, for the exact bounded claim that the anti-capture conclusion does not
follow from an attestation that fails to name every ratified source kind and
both control modes. Safety claim, supplied-records scope bound, with the
enumerated non-extension clause in section 8.

## 10. Evidence

- `book-1-state-form-and-political-membership-decision.md:205-208` — the
  incompatibility sentence and the divided-sources sentence.
- `book-1-state-form-and-political-membership-decision.md:210-211` — the
  ratified instruction that both direct and de facto control be made observable.
- `full-society-ledger.json` — FS-SCN-24 and FS-SCN-70, both `critical`, whose
  failure routes restate the incompatibility verbatim.
- `constitution.nibli` — the 24 rule lines and the single-inhabitant
  `AntiCaptureScope`, measured in section 3.
- `733e5fb` — the FS-POW-064 repair this card follows in shape.

## 11. Choices made under the ratified text, for approval

1. **The five kinds are the closed set.** They are quoted from the ruling. An
   extensible set would let a finding name a kind the ruling never authorised.
   *Alternative:* leave the vocabulary open — refused here, because an open
   vocabulary is the defect being repaired.
2. **`SingleAppointingBodyAppointmentSource` names the fifth kind.** The
   ruling's phrase is the bare "appointing source", which reads ambiguously
   against the other four. *Alternative:* `AppointingSourceAppointmentSource`,
   which is faithful but reads badly.
3. **Control mode is a second vocabulary, not folded into the kind.** Five kinds
   × two modes as a single cross-product vocabulary would need ten members and
   would make "both modes examined" underivable without also fixing the kind.
   *Alternative:* the cross product — costs 10 members and 720 added atoms
   rather than 504, and gains per-kind mode granularity the ruling does not ask
   for.
4. **All five kind-absences are required, conjunctively.** This is the reading
   the ruling's second sentence forces. *Alternative:* the finding names the one
   kind it examined — cheaper, and reintroduces exactly the unexamined-kind hole.
5. **The blanket token is retained, not replaced.** Retention keeps the
   headline disposition and the existing reviewed references stable.
   *Alternative:* replace it — a smaller source but a wider blast radius across
   reviewed JSON needles.
6. **The batch covers both cards together**, because a finding that is gated on
   FS-POW-028 and ungated on FS-POW-035 is gated nowhere. *Alternative:* stage
   FS-POW-028 first — refused; it would ship a half-closed wall behind a receipt
   that implies a closed one.

**Cost, as built.** The 24 attesting rule lines are 12 reviewed-result rules
(`complete($result, FSPOW_028|FSPOW_035, $record)`) paired with 12
holder-authority rules, and each authority rule reads its reviewed result as a
premise; the generator composes the authority rule from the result premises, so
both carry the new conjuncts. Seven fields × three attesters × 24 rules is 504
added atoms, plus seven ground-headed vocabulary rules — 4060 constitution lines
to 4067. The whole state-form artifact set is generated from
`state-form-source.json`, so the edit was 12 branch records and the four
generated outputs followed; nine bound digests, `STATEMENT_COUNT` (274 → 281),
and the approved relation-signature set moved with them. Pin inventories did not
move (391 main, 51 counterfactual). Stratification did not move: 89 predicates,
44 derived, 7 strata, rules 1583 → 1590, chapter order unchanged. All 61
hand-maintained counterfactual fixtures were regenerated by three-way merge and
re-measured against their declared shapes.
