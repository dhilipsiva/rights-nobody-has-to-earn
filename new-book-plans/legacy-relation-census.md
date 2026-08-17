<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Legacy Relation and Institutional-Constant Census

> **Status: measurement only, 2026-08-16.** This is the producer-and-consumer
> census the state-form ruling requires *before* any retain/replace/retire
> card. It records what each name currently does. It decides nothing: no
> relation is retained, replaced, or retired here, and no card is opened or
> closed. The dispositions are owned by the TODO item that defines the
> democratic ceiling and majority process, and by the coverage row whose
> closure condition names "retain/replace/retire cards for the legacy
> relations".

The controlling requirement is in
[`book-1-state-form-and-political-membership-decision.md`](book-1-state-form-and-political-membership-decision.md),
section 9: *"Every retain/replace/retire card begins with a current
producer-and-consumer census. The name's apparent story does not define its
impact."* That sentence is the whole reason this file exists — three of the six
relations below carry legal effects their names do not suggest.

## Provenance and its limits

- Source read in full: `new-book-plans/constitution.nibli` at repository commit
  `4957b44a65a7692a268a8b85512c0e4cff1615a1`; the constitution itself last
  changed at `f276fc02b8825d80142a7915fa194605629fc4d7`.
- Heads, bodies, and ground facts were classified mechanically (comments
  stripped, rules split at `->`), not by eye.
- Cross-checked against the two generated inventories — `3-spine.md`'s
  generated block and `assertion-surface-audit.md`. **Those artifacts are
  authoritative and this reading is not.** One disagreement was found and is
  recorded below; it was in the artifact's hand-authored prose, not in this
  census.
- Engine behaviour quoted here came from probes, not from a verification run.
  A probe says what one binary answered once; `./verify.sh` is what
  establishes a claim. Nothing in this file should be cited as executed
  evidence.
- **Every count here is a dated measurement, not a standing property.** The
  design keeps moving and counts in this repository have gone stale before.
  Re-measure before relying on one.

## The headline finding: `mature` is already gone

`mature` does not exist in the current constitution. Not as a rule head, not in
a rule body, not as a ground fact, not in `admits`, not in `derived_only`. It
was removed at `f276fc0` together with its franchise rule and its four cast
facts, and the replacement route is `at($x, GeneralAdulthood)` feeding
`decide($x, Ballot)` — adulthood as bounded supplied evidence with automatic
acquisition, rather than an asserted maturity label.

Three consequences worth stating plainly, because the tree has not caught up:

1. **The name is reserved by omission only.** `admits("mature")` is absent, so
   a ground assertion is refused today — but `derived_only("mature")` is also
   absent, so nothing reserves the name against a future rule head or a
   re-added `admits` line. That is precisely the gap the `lose`/`decide`
   closure ruling was written to shut for two other relations, and it is open
   here.
2. **Two pinned queries went quiet rather than false.**
   `universal-standing.pins.nibli` still asks `? mature(AgeDispute).` and
   `? mature(Supported).`, both expecting FALSE. They were meaningful when
   `mature` was admitted and asserted; they are now the empty kind of FALSE
   that chapter 9's own pin taxonomy warns about — nothing could have made
   them true. No check in `verify.sh` catches that class.
3. **Several ruling texts still describe `mature` as live**, including
   `CLAUDE.md` and `book-1-closure-gaps-decision.md`, which still prints the
   deleted rule as the current franchise producer. Whether those are frozen
   historical records or stale current claims is a reconciliation decision,
   not a census finding.

## The six relations

### `mature`
Retired. No producer, no consumer, no ground fact, not admitted, not reserved.
See above. **What breaks if retired today: nothing — it already is.** The live
question is the opposite one: whether to reserve the name.

### `decide` — arity 2, derived-only, a leaf
- **Producers:** two Article 2 heads — general adulthood produces
  `decide($x, Ballot)`; a separate early-authority premise produces
  `decide($x, $decision)` for any decision expressly other than the ballot.
  The second ships dormant: the constitution's own cast contains no
  early-authority fact.
- **Consumers: none.** No rule body reads it. Its entire legal effect today is
  being queryable.
- **Writability:** refused twice — `admits` absent, and `derived_only("decide")`
  reserves it at every arity and through any converted alias.
- **What breaks if retired:** chapter 9 entirely, chapter 13's
  single-deprivation pin, chapter 6's franchise-survives-voiding pin, and the
  chapter-9 spine entry. Cheap in the engine, expensive in the prose.
- **A hazard the rulings do not record:** chapter 9's resident hostile
  disenfranchisement exhibit was rewritten to read a constant nothing derives,
  so that rule body is now unsatisfiable rather than merely non-controlling.
  The pin's claim — that the clause compiles and stratifies — still holds, but
  it now holds for a different and stronger reason than its comment says.

### `choose` — arity 2, admitted, directly writable
- **Producers: none.** A pure evidence predicate: the seating certificate.
- **Consumers:** four rule bodies, all positive, feeding two heads — the two
  seating routes to `authority`, and the two credential routes to
  `permits(Review, ·)` and `permits(Tribunal, ·)`. Place 1 carries only the two
  seating-body constants.
- **What breaks if retired:** the exhibits of chapters 2, 3 and 5, both
  `authority` routes, both credential routes, two counterfactual fixtures, and
  a floor complement control that uses `choose` at an arity the constitution
  never uses.
- **Binding limit:** `choose` may never become a floor predicate — doing so
  un-stratifies Article 8 and silently drops the credential rule, so the
  capture defence dies while the amendment engine keeps running.
- Its reviewed writable-premise contract names this specification as its
  closure, and its refused alternative is binding: recall or lapse may remove a
  current pen, but must not erase historical exposure and thereby destroy a
  whistleblower's shield retroactively.

### `broken` — arity 1, admitted, directly writable
- **Producers: none.** Pure evidence.
- **Consumers:** five rule bodies, **all under negation**, feeding four heads —
  voiding, the examiner reward, conviction, and both credential routes. There
  is no positive reader anywhere.
- **The conviction rule guards a constant, not a variable.** It reads
  `~broken(Court)`, so a single write is a universal amnesty. That is the
  cheapest documented catastrophic write in the assertion-surface audit.
- **What breaks if retired:** chapter 2's central distinction — that
  answerability survives recall while power does not — has no other formal
  carrier; the multi-signature rule loses one of its two deliberately kept
  guards; both credential rules lose their recall check.
- **An undocumented complication:** the pinned corpus also asserts `broken` of
  a *record entry*, not a power-holder. It is inert, but one name currently
  spans two readings, and a replacement must decide whether they are one
  relation.

### `approves` — arity 2, admitted, directly writable
- **Producers: none.** Pure evidence.
- **Consumers:** three rule bodies, all positive, feeding three heads — the
  amendment label, and two custody-lease conclusions.
- **The structural hazard is the point of the census.** The same relation at
  the same arity carries a democratic tally outcome *and* a case-bound custody
  renewal approval. Nothing in the source separates them except which constant
  sits in place 1 and which rule reads the tuple. An approval written by an
  amendment-side writer and one written by a custody-side writer are
  indistinguishable in the record. This is the relation whose apparent story
  most badly understates its impact.
- No rule authenticates a tally, certificate, recount, challenge, or
  correction; `approves` names the electorate, never the writer.

### `authority` — arity 1, derived-only, single consumer
- **Producers:** three heads — the two seating routes and the public-body
  route.
- **Consumers: exactly one**, positive: the whistleblower shield. Its entire
  legal effect is that one rule.
- **Writability:** refused; `derived_only("authority")` is declared and pinned
  in-suite as one of the two worst exploits, dead.
- **Deliberately never revoked**, so that a recall cannot retroactively strip a
  whistleblower's protection. Seating is forever exposable; only power is
  revocable.
- **Two asymmetries the census surfaces that no prose accounts for:**
  (a) the witness and time services became exposable targets as a side effect
  of being declared public for the temporal envelope, not by a stated design
  choice; and (b) the bodies that confer legitimacy or power — the electorate,
  the amendment-docketing assembly, and the second pen — are **not** exposable,
  so exposing any of them confers no shield.

## Institutional constants

Measured by non-comment occurrence. The findings that bear on a disposition:

- **`Court`** is the most-used constant, but most of its uses are as a
  truth-carrier that fires unconditional barriers — nothing about the Court's
  function is read there. Its load-bearing uses are conviction and custody.
- **`Appeals`** is the only constant appearing in three namespaces at once: a
  public body, a credential holder, and the duty-bearer of every `obliged`
  head.
- **`State`** carries the whole floor debt and is exposable through one public
  fact. The debt has no readers; it is a leaf.
- **`Tribunal`** is the thinnest constant in the file — it exists only as a
  namespace token inside the permission relation, with no public fact, no
  judgment, no duty route, and no ground fact of its own. It is nonetheless one
  of the two cross-body pens the voiding rule depends on.
- **`TemporalReview`** is simultaneously a witness, a carry attestor, a
  challenge destination, and an approver — four functions on one name, with no
  rule separating them.
- **`Electorate`** and **`Assembly`** have no public fact and are never seated,
  so neither is exposable.

The ruling's warning applies directly to all of them: none may silently become,
or coexist unexamined with, a new election, seat, recall, approval, office,
public debtor, court, or lawful-power interface. In particular, the current
executive-adjacent constant is not the Executive Council, the current Court is
not the Constitutional Court, and the current duty-bearer constant is not a
completed federal government.

## One disagreement found, and it was in the artifact

`3-spine.md`'s hand-authored prose claimed that `authority` was the only
derived stratum-0 predicate with a wholly negation-free cone. Its own generated
block, twelve lines above, marks several. The generated block is authoritative;
the prose was written before the temporal layer landed and nothing gates it,
because the generator's `--check` covers only the marked region. The sentence
was corrected in the same change that added this census, and rewritten to point
at the generated block instead of restating its contents — which is how it went
stale in the first place.

## What this census does not do

It makes no disposition. It opens no card. It does not decide whether any
relation is retained, replaced, or retired, and it supplies no replacement
vocabulary. It establishes nothing about the world, nothing about any
institution's existence or independence, and nothing about the engine beyond
what a probe answered once on a dated binary.

The ruling's sequencing constraint stands and is the reason the dispositions
are withheld here: a replacement must preserve or deliberately revise every
downstream verdict and assurance case **atomically**, and the narrowness-impact
gate must first be applied to the chapters, the temporal case, the affected
Part V verdicts, and the method part.
