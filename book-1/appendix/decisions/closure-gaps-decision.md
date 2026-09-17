<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Closure-Gaps Decision

> **Status: author-ratified and implemented L1 + D1 on 2026-08-04;
> verification recorded in Section 11.** The ruling reserves `lose` and legacy
> `decide` relation-wide for conclusions produced by rules. It does not settle
> the other omission-closed relations.

## 1. Decision and scope

Before ratification, two current legal effects were derived by rules but
protected from direct assertion only accidentally:

| Relation | Current producer | Pre-ratification assertion posture |
| --- | --- | --- |
| `lose(Points, $x)` | `false($x) -> lose(Points, $x)` | A forged loss is refused because `lose` is absent from `admits`, not because it is declared conclusion-only. |
| `decide($x, Ballot)` | `person($x) & mature($x) -> decide($x, Ballot)` | A forged franchise conclusion is refused for the same omission-only reason. |

Adding some other admitted name does not open either relation. The measured
bypass is an **exact-name** reclassification: without a `derived_only`
declaration, adding `admits("lose")` or `admits("decide")` makes the corresponding
ground assertion writable. The current refusal message itself recommends that
edit. The author chose L1 + D1: each name is now reserved, relation-wide, for
conclusions produced by rules.

These are the **two low-risk members**, not the whole class. A 2026-08-04 engine
census finds twelve authored derived relations on neither guarded list: the two
above, the eight floor-actuality names, `obliged`, and `travel`. `person` is the
settled mixed case: both admitted and rule-produced. The remaining ten need
delivery, duty, or liberty contracts before their assertion posture can be
ruled; this decision must not close them by analogy.

## 2. Pre-ratification measurements

Recorded 2026-08-04 on scratch overlays, without changing the tracked
constitution, against the release `nibli-pin` last built by this repository's
verifier from nibli commit `9b84be3`. The adjacent checkout's engine paths were
clean; its dirt and subsequent move to `9ebeb33` were documentation/setup-only,
and the binary was not rebuilt merely to relabel that unchanged engine code.

- the current base, L1 alone, D1 alone, and L1 + D1 each passed the full current
  536-pin suite with zero findings and the same nine declared defects still
  reproducing;
- exact-name admission made forged two-place `lose` and `decide` facts load,
  while either `derived_only` declaration independently refused both the live
  two-place form and the corpus's three-place form;
- legitimate rule-head derivation remained available;
- the combined overlay produced byte-identical engine strata and required no
  generated-spine change; and
- after both declarations were propagated to scratch copies of all six
  counterfactual constitutions, every fixture retained its required one-line
  diff class and every counterfactual pin suite passed.

These are compatibility measurements, not the author ruling and not permanent
clearance. Re-run them against the exact release engine and source that will be
committed.

## 3. What `derived_only` means here

`derived_only` is an assertion boundary, not a policy engine.

- It refuses a direct ground assertion even if the same name is later added to
  `admits`.
- It applies to the relation, not one tuple shape. The corpus gives both `lose`
  and `decide` three places even though this constitution uses two. Every arity
  is reserved, and any future converted alias would compile to the same closed
  relation.
- It still permits a rule head to derive the relation. The existing franchise
  and clawback rules therefore remain valid.
- It does not authorise a rule, protect the rule from repeal, authenticate its
  premises, or make an inert conclusion operative.

That namespace cost is part of the ruling. Under the ratified choice,
`lose` means a rule-concluded legal loss marker, never raw evidence that a loss
occurred. `decide` means a rule-concluded legal status, not a cast ballot, a
tally, a resolution, or proof that an election occurred. Future democratic
work must keep that legal conclusion separate from accessible ballot provision,
privacy-preserving participation evidence, tally, and legal result. Whether the
legacy name remains for the first of those stages is not decided here.

## 4. What remains open after hardening

| Area | Not fixed by these declarations |
| --- | --- |
| Loss | `lose` remains a leaf read by no rule. There is still no amount, proportionality, restoration, or systemic remedy. Forged or withheld premises can still produce or suppress the conclusion. A rule edit can still create another loss route. |
| Franchise | `mature` can still be forged or withheld; the franchise rule can still be changed or deleted; a hostile rule head can still derive `decide`; and the model still provides no election, accessible ballot, casting evidence, honest count, or review. |
| Both | The rule-write and deletion trust bases remain. `derived_only` changes a one-fact bypass into a visible rule or declaration edit; it does not make such edits legitimate. |

Chapter 9's resident disenfranchisement exhibit must continue to load and leave
Hano enfranchised while the wider franchise rule survives. Chapter 6's loss marker
must remain inert. Any prose suggesting that either declaration supplies more
would turn a narrow closure into an overclaim.

## 5. Author ruling

The axes are independent. Similar mechanics are not a reason to conceal a
different semantic choice.

### Loss relation

- **L1 — harden `lose` relation-wide (adopted).** Reserve it for legal
  conclusions. A future factual report of material or personal loss must use a
  different admitted relation with its own writer and challenge contract.
- **L0 — leave `lose` omission-closed (refused).** This would preserve future
  reuse of the corpus relation but knowingly leave its current safety dependent
  on the admitted list never widening to it.

### Franchise relation

- **D1 — harden legacy `decide` relation-wide (adopted).** Treat
  `decide(_, Ballot)` as the current derived franchise conclusion. The expansion
  will need distinct names for the other democratic interfaces, but whether it
  retains this relation for the franchise or replaces it is a later design
  decision. If replaced, retire it only in the same change that lands equivalent
  closure, pins, and prose.
- **D0 — defer `decide` until the democratic vocabulary is redesigned
  (refused).** This would avoid reserving a broad corpus word now, but keep a
  known assertion edge open throughout that work.

**Ratified composite ruling: L1 + D1.** It closes two unambiguous current
bypasses without pretending that a ballot has been cast or that a recorded loss
does anything. It is deliberately scoped; it does not ratify an automatic rule
that every derived name must be `derived_only`.

## 6. Required formal controls

The L1 + D1 implementation contract required every applicable control below to
pass. Section 11 records the completed verification.

1. Put the declaration in Article 0 above every ground fact. Do not add the name
   to `admits` as the implementation.
2. Exercise the actual bypass in a scratch negative control: without the
   declaration, add the exact `admits` entry, assert the forged fact, and prove
   its query becomes TRUE. With the declaration, repeat the same admission and
   assertion; it must refuse as “declared derived-only,” remain absent, and
   leave any legal conjunct in the same assertion uncommitted. This proves the
   new guard and its atomicity rather than merely matching the old omission's
   different error text.
3. Refuse both the constitution's two-place form and the corpus's three-place
   form. Confirm that no current converted alias exists.
4. Preserve a positive derivation for each adopted relation. Chapter 6 already
   derives loss for Bela; Chapter 9 already derives the franchise for Hano.
5. Prove rule-head permeability deliberately: the existing producer and a
   scoped hostile-rule control must still load. This records what the boundary
   does not protect.
6. Measure L1 alone, D1 alone, and L1 + D1 against the current full pin set. Any
   changed verdict returns the item to the author gate; an older 526-pin result
   is historical evidence, not current clearance.
7. Compare engine strata and the generated spine region before and after. These
   declarations should add no rule, dependency edge, stratum, or chapter move.

The permanent content pins belong beside the claims they protect:

```nibli
:accept
admits("lose").

:refuse reasoning /`lose` is declared derived-only/
person(Loss_Probe) & lose(Points, Loss_Probe).

? person(Loss_Probe).
# => FALSE

:refuse reasoning /`lose` is declared derived-only/
lose(Points, Ghost, Case).
```

```nibli
:accept
admits("decide").

:refuse reasoning /`decide` is declared derived-only/
person(Vote_Probe) & decide(Vote_Probe, Ballot).

? person(Vote_Probe).
# => FALSE

:refuse reasoning /`decide` is declared derived-only/
decide(Ghost, Ballot, Election).
```

Each block pins the exact-name admission as a resident premise, the live
two-place refusal, atomic rollback of its otherwise legal `person` conjunct,
and the full corpus arity. The existing Bela/Hano TRUE queries are the
non-vacuous derivation complements. Duplicating the same assertions in
`rights-floor.pins.nibli` adds runtime but no discriminator; that file is not an
exhaustive closure roster today.

## 7. Editorial and artifact contract

The constitutional and reader-facing disposition must be atomic.

- Rewrite both soft-edge passages to record the selected outcome. Under L1 or
  D1, say that the relation used to be closed by omission and is now explicitly
  conclusion-only. Under L0 or D0, say that the author deliberately retained
  omission-only closure for the stated namespace reason; do not leave it
  sounding like an unmade decision.
- Under L1, preserve that `lose` is inert and rule-writable. Under D1, preserve
  the adulthood, repeal, delivery, and honest-count failures, and call
  `decide(_, Ballot)` a franchise conclusion rather than a cast vote.
- Add a matching local admission-plus-refusal block, and update the file's
  derived `:expect-pins` value, only for an adopted declaration. In Chapter 9 it
  must precede the final resident `:accept` block, after which nothing may run.
  `admits` is intentionally unscoped here; per-file engine freshness contains
  the widened scratch vocabulary.
- Add Chapter 6 to `verify.sh`'s explicit unscoped-premise allowlist and rederive
  that guard's positive-control count. Chapter 9 is already allowlisted for its
  resident hostile rule. Do not weaken the guard globally to accommodate the
  new `admits` premise.
- Repair the already-stale hand-written writable-surface paragraph in
  `3-spine.md`. It predates Article 0a, still calls `entitled` and `owe` open,
  and carries obsolete counts and line references. Replace it with structural
  language: `admits` names the candidate base vocabulary; `derived_only`
  independently removes conclusion-only relations from direct assertion; and
  `person` is the admitted, rule-produced mixed case. Do not refresh another
  hand-maintained count. Leave generated regions alone.
- Sweep the rest of `3-spine.md`'s hand-written region for the same mechanism.
  In particular, retire or correct its already-stale “`lose` has no adversarial
  pins” label when L1 lands. Do not edit the generated block to do this.
- If either declaration changes the constitution, regenerate all six
  counterfactual constitution copies. Their deliberate one-line deltas and pin
  sidecars remain unchanged.
- Audit Chapter 1, Part V, and `method.md`; current claims are expected to remain
  true, but expectation is not a substitute for the narrowness-impact check.

## 8. Preventing the next omission

This patch must not be described as a complete closure sweep. Before expansion
widens the vocabulary, generate an assertion-surface census that classifies every
authored derived relation as exactly one of:

1. explicitly `derived_only`;
2. admitted and rule-produced under a named mixed/base-fact contract; or
3. deliberately unadmitted pending a named interface decision.

Seed the third class with the eight floor actualities, `obliged`, and `travel`.
The census must fail on an unclassified new relation and on a posture change
whose register entry was not updated. It must not infer that every rule head
should be closed; doing so would contradict the settled `person` decision and
pre-empt the delivery, remedy, and liberty designs.

## 9. Landing sequence

1. Ratify L1/L0 and D1/D0, including the relation-wide namespace consequence.
2. Re-run the independent and combined scratch measurements against the current
   release engine and full pin set.
3. In one content change, add each adopted declaration and local pin, record
   both selected outcomes in their respective prose passages, repair the
   `3-spine.md` manual paragraph, and regenerate fixtures if the constitution
   changed.
4. Run `./verify.sh --quick`, both affected chapter suites, the spine generator
   with `--check`, and then the full `./verify.sh`.
5. Commit the content together with the ratified status in this artifact and
   the authoritative ruling in `CLAUDE.md`. In a separate tracker commit, close
   this author gate in `TODO.md`, remove the now-false historical soft-edge
   statement, and cite the content commit.
6. Run the full verifier again before the TODO tracker commit. Do not close the
   generated assertion-surface census task merely because these two names land.

## 10. Ratification record

- [x] **L1 — `lose` is relation-wide conclusion-only (adopted 2026-08-04).**
- [ ] **L0 — retain omission-only closure for `lose` (not selected).**
- [x] **D1 — legacy `decide` is relation-wide conclusion-only (adopted 2026-08-04).**
- [ ] **D0 — defer `decide` to the democratic vocabulary redesign (not selected).**

The author ratified this gate by replying **L1 + D1** on 2026-08-04.

## 11. Implementation and verification record

Implemented and measured 2026-08-04:

- Article 0 reserves `lose` and `decide` relation-wide without admitting either
  name or changing a producing rule;
- Chapters 6 and 9 each keep the exact-name admission resident only at the end
  of their relevant pin context, refuse the live and corpus arities, prove
  atomic rollback, and preserve a positive rule derivation;
- the chapter suites pass 39 and 23 pins respectively, and the reconciled full
  suite passes **544 pins with 0 findings** while all nine declared defects
  continue to reproduce;
- the generated spine block remains unchanged, and its hand-written assertion
  explanation is now structural rather than count-based; and
- all six regenerated counterfactual constitutions retain their required diff
  shapes and all six counterfactual suites pass.

The quick verifier, both targeted suites, the spine `--check`, and the full
verifier all passed. This closed only L1 + D1. The assertion-surface census
landed later as a separate generated audit; the interface decisions it records
as pending remain open.
