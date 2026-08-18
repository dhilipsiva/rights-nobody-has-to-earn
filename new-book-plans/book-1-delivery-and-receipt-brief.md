<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery and Receipt Rule Families — Neutral Decision Brief

> **Status: brief only, 2026-08-18. This decides nothing.** It is the neutral
> inventory the tracker permits ahead of an outstanding author ruling. No
> predicate, rule, pin, fixture, chapter, defect row, receipt, posture, or
> public claim is created or altered here. Options are presented with their
> costs; where the session has a reading it is marked as such and is not a
> ruling.

## Why this brief exists now

Two of the project's open **critical** defects close on the same family, and
nothing else closes them:

- **FS-DFT-16** — *"Five floor items have no producing rule and two arrive only
  through confinement."* Closure: *"accessibility-neutral, recipient-side
  delivery and receipt families land for the floor items with no producing
  rule."*
- **FS-DFT-17** — *"No arrival of any floor item to a free person is
  representable."* Closure: *"arrival facts land and the delivery families
  produce them."*

Two more wait behind them. **FS-DFT-18** (the pinned portfolio points at the
cage) records in its own closure condition that *"the delivery families land
first; the rebalance is the consumer that can only run afterwards"*, and
**FS-DFT-26** (floor-delivery markers refused) expires when arrival facts
exist. The thesis ruling sequences the same way: delivery/receipt families,
then the portfolio rebalance, then the reader ledger, then author-drafted
prose.

The drafting gate is passed. The coverage map's material-floor row is
`coverage-ready`, and the assertion surface already carries a governed hook for
each of the eight actualities — `pending_interface`, contract id
`floor-actuality-<name>-pending`. This family is their named owner.

## 1. The measurement, taken against the current source

Producing rules for each floor actuality, measured 2026-08-18 at `a0b5e85`:

| actuality | producing rules | ground facts | route |
|---|---|---|---|
| `secure` | 0 | 0 | none |
| `eats` | 0 | 0 | none |
| `healthy` | 0 | 0 | none |
| `believe` | 0 | 0 | none |
| `meets` | 0 | 0 | none |
| `dwell` | 3 | 0 | all three `prisoner`-gated |
| `expresses` | 1 | 0 | `prisoner`-gated |
| `learn` | 1 | 0 | `teaches` + `hears`, non-carceral, ships dormant |

So the deficit is exactly as the register ruling states it, and it should be
stated as the rule rather than the count: **no floor actuality has a rule that
reaches a free person, and the only non-carceral route that exists produces no
cast arrival.**

**Vocabulary status.** All eight are **unadmitted and not conclusion-only**.
They are closed by omission from `admits` alone. (A grep will appear to show
`derived_only("eats")`; that string occurs only inside a comment at
`constitution.nibli:1650` and is not a declaration. Check the line before
relying on it.)

## 2. What is already ruled, and is not reopened here

- **Shape.** Delivery families must use accessibility-neutral, recipient-side
  access/receipt evidence with a named authorised writer and a challenge route.
  They may secure **conditions** but may not certify learning, health, belief,
  or another compelled personal state.
- **The legacy route is not the template.** `hears` is audio-literal and
  uncontrolled; `learn` is a personal outcome rather than neutral receipt.
- **No provider self-certification.** A provider's own assertion cannot alone
  count as delivery or receipt. The economic ruling repeats this for cash,
  vouchers and insurance.
- **No floor becomes conditional**, and price or ability to pay cannot gate it.
- **Book 2 keeps** logistics, staffing, procurement, budgets, routing,
  maintenance, capacity and graceful degradation.

## 3. The constraint that governs the whole family

The constitution has already measured this and recorded it in its own comment
at `constitution.nibli:1647-1656`. It is the single most important input here,
and it forecloses the obvious repair:

> `derived_only` refuses ground assertion and is what MAKES a head
> rule-writable-only — re-measured today against a copy with
> `derived_only("eats")` inserted: the fiat rule loads and `eats(Adam)`
> derives. `admits` closes ground vocabulary and says nothing about rule heads.
> **There is no compile-time guard available from either direction; a fiat rule
> is syntactically indistinguishable from a legitimate delivery route.**

And its own answer:

> What tells them apart is upstream evidence: a delivery route derives an
> actuality FROM a record of something reaching a person (the shape the
> teaching->learn route would have), while a fiat derives it from the roster
> alone. So the defence is review of rule diffs against one question — **what
> arrival evidence sits upstream of this head?** — and any provisioning layer
> built on this file must keep a delivery record and a derived actuality as
> different things.

Two consequences follow, and both are load-bearing:

1. **Declaring the actualities `derived_only` is not the safety measure it
   looks like.** It closes the fiat *fact* and leaves the fiat *rule* open. It
   is still worth doing for the same reason L1+D1 was done for `lose` and
   `decide` — closure by declaration rather than by omission — but it must not
   be described as preventing fiat.
2. **The stated defence is human review, and that is mechanisable.** "What
   arrival evidence sits upstream of this head?" is a structural question about
   a rule body. See §6.

## 4. The design problem, per floor item

The ruled bar on certifying a personal state does not fall evenly. Three
groups, and the third is the hard one:

**(a) Material conditions — the straightforward cases.** `eats`, `secure`,
`dwell`, `meets`. Something reaches a person and the reaching is recordable
without describing an inner state. These are where the lifecycle applies most
cleanly.

**(b) Liberty-shaped — `expresses`, `believe`.** These are not delivered at
all. Nothing arrives; what is owed is non-interference plus the conditions that
make exercise possible. `believe` in particular is conscience — certifying that
a person believes is exactly the compelled personal state the design refuses,
and the firewall exists to stop belief being read at all. A "delivery route to
`believe`" may be a category error rather than a gap.

**(c) Personal states with a public precondition — `healthy`, `learn`.** Health
and learning are outcomes; care access and teaching access are conditions. The
ruled bar permits securing the condition and forbids certifying the outcome —
but the floor entitlement is written to the *outcome*
(`entitled(every person, event { healthy() })`).

**This is the brief's principal structural finding.** For groups (b) and (c),
closing FS-DFT-16 by giving every actuality a producing rule would require
deriving precisely the personal states the design refuses to certify. The
defect's own closure condition is narrower than it first reads — it says
*"delivery and receipt families land for the floor items with no producing
rule"*, not *"every floor item acquires a producing rule"*. Whether some
actualities are **deliberately left underived with a stated reason** is a real
question this family must answer rather than route around.

## 5. Open question A — which floor items get an arrival route

- **Option A1 — material items only.** Land routes for `eats`, `secure`,
  `dwell`, `meets`; leave `healthy`, `learn`, `expresses`, `believe`
  deliberately underived, each with a recorded reason distinguishing
  category error from unbuilt route. *Cost:* FS-DFT-16 closes only partly and
  needs its closure condition restated; the book must say plainly why four of
  eight have no arrival, which is a harder paragraph than it sounds.
- **Option A2 — condition predicates for the outcome items.** Land routes to
  *access* conditions for `healthy` and `learn` alongside the material four,
  and keep the outcome actualities underived. *Cost:* new vocabulary for the
  access conditions and a visible seam between "care reached you" and
  "you are healthy" — which is honest, and is also a seam a reader can
  misread as evasion.
- **Option A3 — all eight.** *Cost:* requires certifying personal states.
  Session reading: this is refused by the standing ruling and is recorded here
  only so the option is disposed of rather than omitted.

*Session reading, not a ruling:* A2 for `healthy`/`learn`, A1's treatment for
`expresses`/`believe` (liberty, not delivery). That combination closes the two
criticals for everything that can honestly close, and turns the remainder into
a stated boundary rather than a silent gap.

## 6. Open question B — the receipt writer who is not the provider

Recorded in the thesis brief §4 as having **no current owner**, and unchanged:
`medium-neutral` and `accessibility-neutral` bind the *channel*. Neither
reaches a recipient who can form no acknowledgment through any channel. For
that person both obvious authorised writers fail — the provider is refused by
settled rule, and a chosen supporter is unavailable to anyone whose defining
condition is having none.

Candidates, none ruled:

- **B1 — an independent observer separated from the provider.** *Cost:* a new
  role and a new separation rule; risks becoming a second provider-side
  attestation if the separation is not enforced.
- **B2 — the rights advocate** (FS-BOD-20 already exists and is mandated to act
  without replacing the person's voice). *Cost:* loads a general advocacy
  office with a per-delivery evidentiary function it was not specified for.
- **B3 — absence of receipt as the operative trigger.** The lifecycle item
  already says a missing receipt must invite outreach and challenge and never
  terminate entitlement, so this route makes the absence do the work and
  requires no writer at all. *Cost:* the trigger fires on everyone while no
  arrival facts exist — which is exactly the measured reason the floor-delivery
  markers were refused (FS-DFT-26). B3 is therefore **not available on its own
  before arrival facts exist**; it is a consumer of this family, not a
  substitute for it.

*Session reading, not a ruling:* B1 or B2 must carry the positive route, with
B3 layered on afterwards as the failure path. The measured marker problem means
B3 alone would reproduce the defect it is meant to close.

## 7. The corpus is a hard gate, and it currently blocks the recipient side

Predicate names must exist in nibli's curated corpus; an unknown name is a
compile error. Measured 2026-08-18 against
`nibli-lexicon/src/corpus/predicates.rs` at engine `8935611`:

| candidate | status |
|---|---|
| `gives` | **available, Curated**, places `giver / gift / recipient` |
| `grant` | available, Curated, places `agent / action / resource`, authorization-shaped |
| `provide` | available, **Generic, `TODO(corpus): guessed places`** |
| `serve`, `observe` | available, Generic (unverified places) |
| `attend`, `shelter` | available, GlossDerived (weakest); `shelter`'s x1 is the shelter, not the person |
| `receive`, `accept`, `obtain`, `access`, `reach`, `arrive`, `take`, `collect`, `acknowledge`, `attest`, `witness`, `confirm` | **absent** |

**The finding: the corpus supplies provider-side verbs and lacks recipient-side
ones — which is precisely the half the ruled design requires.** `provide`
carries the same unverified-places marker that got `reward`'s provenance
proposal refused, so it should not be adopted on its face.

This means the family needs a **corpus request** on the `ratifies`/`endorses`
precedent, and that is a channel round-trip to the nibli session. It is
independent of questions A and B and can start immediately. Naming the needed
senses is a decision the author may want to make first, but establishing
*availability* does not require a ruling.

**Narrowed on measurement, 2026-08-18 — the request is one name, not nine.**
Drafting the request against the engine rather than the corpus file showed the
ask is much smaller than the table above suggests. Two of the three roles a
delivery rule needs are already available: `gives` at arity 3
(`giver / gift / recipient`) is the provider half, and `observe` at arity 4 is
**already this source's independent-witness idiom** — `observe(Chronicle, …)`
beside `observe(TemporalReview, …)` at a named scope, the pattern the T3 rules
use. Only the recipient-side arrival predicate is missing.

Measured with `carries` standing in for the absent name, the whole shape
compiles and derives, and the property the family exists for holds: a person
with a provider record, a recipient-side record and two independent witnesses
derives the actuality, while a person with **only** the provider's record
derives nothing. So the structure is not in question — the name is. The request
is recorded in `TODO.md` and asks for `receives/3`, preferring the stative
`terdu'a` (the exact converse of `dunda`, which already sources `gives`) over
the volitional `cpacu`, because "gets / procures / accepts" would be false of
precisely the recipients §6 is about.

**Returned 2026-08-18 — the vocabulary gate is closed.** Engine `bc03c9a`
curates `receives` at arity 3, places `recipient / gift / donor`, source
`terdu'a`, `CorpusTier::Curated`. Verified here rather than on the reply: the
suite is green against a rebuild of that sha with no verdict moved.

Two things from that exchange bear on how the family gets built, and both
belong here rather than only in the tracker.

**A curation choice could have defeated the rule silently.** `terdu'a` is the
converse of `dunda`, so the tidy modelling is a converse alias — and an alias
compiles to the same stored relation with places exchanged, which would have
made `gives(Kitchen, Meal, Bo)` establish `receives(Bo, Meal, Kitchen)`. The
giver's word would have established receipt through a vocabulary decision, in a
place no rule review looks. `receives` is an independent relation, and this was
confirmed behaviourally in both directions rather than read off the `swap:
None` declaration. **The pins that establish it must ship beside the delivery
rule when it lands** — they have nowhere to live until then.

**The brief's own corpus table was wrong in one row.** `get` is curated at
arity 3 from `cpacu` — the second candidate above — so "every recipient-side
spelling is a compile error" was false. It changes no conclusion, because
`cpacu`'s volitional sense is rejected on the merits and that rejection is what
selected `terdu'a`. The method fix: resolve a candidate lemma to its English
corpus name through `nibli-lexicon`'s provenance index instead of guessing
spellings.

## 8. A mechanical guard this family could ship

The constitution names the defence — *what arrival evidence sits upstream of
this head?* — and leaves it to human review. That question is structural and
checkable: a rule whose head is a floor actuality must contain, in its body, at
least one relation from a declared arrival vocabulary.

Such a check would have caught the fiat rule the constitution's own comment
describes, and it fits the repository's existing pattern of guards that watch
what no compile-time mechanism can (`verify.sh`'s absence loop, the
recognition-arity guard, the no-counted-degree checks). It needs a
watched-failing negative control like every other guard here.

Offered as a proposal, not a decision. It does not remove the need for diff
review; it removes the cheapest way to forget.

## 9. What this brief does not do

It creates no predicate, rule, pin, fixture, chapter, defect row, receipt,
posture, coverage change, or public claim. It closes no defect and moves no
gate. FS-DFT-16 and FS-DFT-17 remain open and critical; FS-CLM-05 and
FS-CLM-06 keep their current postures. Nothing here authorises drafting prose:
the coverage contract's bar on claim-bearing pins and prose stands until the
rule family itself lands.

## 10. Sequence after a ruling

1. Corpus request for the recipient-side senses (§7) — startable now.
2. The rule family: delivery and receipt relations, authorised writer,
   challenge route, failure and interim-continuity path.
3. Pins, and the counterfactual fixtures that show what the world loses.
4. The upstream-arrival guard (§8) with its negative control.
5. Assertion-surface contracts for each `floor-actuality-*-pending` hook.
6. Ledger, defect resolution, Gate A.
7. Only then the portfolio rebalance (FS-DFT-18) and the prose.
