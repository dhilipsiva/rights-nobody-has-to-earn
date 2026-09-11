<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Disclosure Duty Contract

> **Status: Specified contract card, session-drafted 2026-09-11 under ruling B
> of `book-1-democratic-and-administrative-integrity-decision.md`; awaiting
> author approval of this exact text including the choices in section 11.**
> This card formalises what disclosure is owed and to whom. It does not, and
> cannot, establish that anyone disclosed anything.

## 1. Decision

Disclosure becomes a constitutional duty through the landed typed
`obliged(bearer, duty, standard)` bridge. Four effects join the obligations
family:

| id | effect | duty kind |
|---|---|---|
| FS-CCE-387 | Disclosure duty | `DisclosureDutyKind` |
| FS-CCE-388 | Disclosure reader and action duty | `DisclosureReaderActionDutyKind` |
| FS-CCE-389 | Certified positive disclosure nonresponse | `DisclosureNonresponseKind` |
| FS-CCE-390 | Disclosure alternate escalation | `DisclosureAlternateEscalationKind` |

The bearer is an office holder, a candidate, a party or coalition, or an actor
the money-and-influence family (ruling A) names. The reader is the independent
audit and integrity function the state-form ruling already mandates; the
alternate is the ombudsperson or rights advocate. All four are Role-class
duties under the public bearer mode, so breach does not automatically punish
the holder or remove status — the obligations decision's own disposition.

**The half this card exists for is the third and fourth rows.** A reader's
nonresponse is a *certified positive record*, not silence, and it derives the
alternate's duty. Audit starvation therefore produces a derivable duty on a
second office rather than nothing at all. That is the whole answer this design
has to a captured or defunded reader, and it is deliberately not a claim that
the second office acts either.

## 2. What already exists, and is extended rather than restated

FS-CVF-016 landed the bridge, the origin join, and the finding-side reader,
nonresponse, and alternate machinery at FS-CCE-213 through FS-CCE-215. Ruling B
says the non-response mechanism should work "as it is in the landed obligations
family", and this card takes that literally: the disclosure effects reuse the
family's origin materialisation, its collision rules, its independent-review
premise, and its barrier idiom. `prevents($disclosure_nonresponse,
SilenceAsDisclosureAction)` is the exact mirror of FS-CCE-214's
`SilenceAsFindingAction`.

What did not exist was any disclosure duty at all. Before this card the words
disclosure, register, and filing appeared in no ratified decision record, and
the obligations family knew only about *findings* — a reader who received a
finding had duties, and a bearer who owed the public an account of their money,
interests, or contacts had none.

**Separate scopes, deliberately.** Each disclosure effect declares its own
bearer, duty, and standard scope rather than reusing the finding-side ones, so
a disclosure duty cannot be satisfied by a finding-side atom and a finding's
reader cannot be borrowed as a disclosure reader. The two routes stay distinct
at the atom level, not merely by naming convention.

## 3. The measurement this card exists to repair

Measured at `8bcbf93`: the obligations family derived reader, action,
nonresponse, alternate, continuity, remedy, correction, re-audit, and
recurrence duties — all of them downstream of a *finding*. Nothing obliged any
office holder, candidate, or party to disclose anything to anyone, so the
audit and integrity function had no disclosure to read, and the question of
what happens when it does not read one could not be posed in the source.

## 4. Formal migration contract

- **No new relation.** All four effects conclude through `obliged/3`, which is
  `derived_only` and already confined by `verify.sh` to one allowlisted
  consumer. `disclose`, `declare`, `register`, `publish`, `report`, and
  `reveal` are not corpus names, which is why the duty is expressed as a duty
  kind rather than a verb.
- **One barrier constant added**, `SilenceAsDisclosureAction`, mirroring
  `SilenceAsFindingAction`.
- **Two watched failing controls**, both generated omissions:
  `FS-CCE-389 omission PositiveDisclosureNonresponseScope` and
  `FS-CCE-390 omission DisclosureEscalationDutyScope`, registered together as
  the `disclosure-nonresponse-alternate` watched mutation case. Without the
  certified nonresponse the reader's silence derives nothing; without the
  escalation duty a starved reader produces no second office's duty.
- **A latent indexing bug fixed in the same change**, section 11 choice 2.

## 5. Executable cases

The family's generator produces, for each of the four effects, a positive pin
and a per-conclusion source-bound pin, plus the standard omission negative that
the ledger's `negative_test` rows declare. The two disclosure-specific
omissions above are additional.

Measured 2026-09-11 against the regenerated family. The obligations block
grew from 65 to 70 exact statements and from 25 to 29 effects; the generated
suite grew from 122 to 132 pin cases and from 142 to 153 queries, and the
counterfactual pin counts moved from 25/26/28 to 29/30/28. Every one of those
is a generated consequence of four effects, not a hand-authored number.

`./verify.sh --quick` passes on the complete tree, and the family's own check
reports `obligations: PASS - 29 effects, 70 exact statements, 153 main pins,
29/30/28 counterfactual pins, ..., 13 watched mutation seams, and one exact
obliged consumer`.

**One defect found and repaired while measuring.** That report's seam count
was a hard-coded `12` in the format string rather than a value read from
`WATCHED_MUTATION_CASES`. It would have gone on printing `12` with thirteen
seams in the table — a check reporting the wrong number of its own controls.
The count is now derived from the table, so the two cannot drift apart again.

## 6. What this does not establish

It does not establish that any disclosure was made, filed, received, read,
published, or acted on; that any register exists or is complete; that any
reader is independent, staffed, or funded; that any alternate exists; or that
any nonresponse was ever certified. **Every one of those is an arrival, and
arrival is a liveness claim that may never be Derived.** The duty and the
failure polarity are what this card establishes; the world is not.

It creates no publication duty, no threshold, no amount, no filing period, and
no register schema — all four are democratic law and Book 2 operation. It does
not reach standing, the floor, the equal ballot, core liberties, due process,
or remedy.

## 7. Assurance posture

Derived, for the exact bounded claim that the disclosure duties, the certified
positive nonresponse, and the alternate escalation do not follow from a record
that omits a required source-bound field. Safety claim, supplied-records scope
bound, with the enumerated non-extension clause in section 6.

## 8. Evidence

- `src/checks/obligations.rs` — four `Effect` rows, their scope and conclusion
  arms, the two omission controls, and the `effect_index` repair.
- `new-book-plans/constitution.nibli` — the regenerated `OBLIGATIONS-RULES`
  block.
- `new-book-plans/obligations.pins.nibli` and the four delegated counterfactual
  files, all generated.
- `new-book-plans/full-society-ledger.json` — FS-CCE-387..390 and FS-CVF-016
  extended to 29 effects.

## 9. Choices made under the ratified text, for approval

1. **Four effects, not one.** The duty, its reader, the certified nonresponse,
   and the escalation are separate legal effects because the obligations family
   already separates them on the finding side, and collapsing them would make
   the nonresponse unobservable — which is the one thing ruling B singles out.
2. **A latent indexing bug is fixed in this batch rather than deferred.**
   `obligations.rs` looked effects up positionally as `EFFECTS[number - 198]`
   in two places, one of which runs for every effect. That is correct only
   while the numbers stay contiguous from 198 — an invariant nothing documented
   and nothing checked, and one that FS-CCE-387 breaks by construction. Both
   lookups now index by position in `EFFECTS`, which is what the rule sets are
   actually built from. Behaviour is identical for 198–222. It is in this batch
   because the batch cannot build without it.
3. **Role class, public bearer mode**, matching the finding-side reader duties
   and carrying the obligations decision's disposition that breach does not
   automatically punish the holder.
4. **Separate scopes per effect** rather than reuse of the finding-side atoms,
   so the two routes cannot satisfy each other.
5. **No derived prose in this batch**, on the precedent of the three landed
   integrity families. Disclosure is the first of them a reader would actually
   see, so its paragraph should be drafted once D, G, and F are known and the
   whole item can be projected together rather than in four passes.
