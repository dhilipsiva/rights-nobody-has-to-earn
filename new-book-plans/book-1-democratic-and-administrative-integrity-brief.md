<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Democratic and Administrative Integrity — Neutral Decision Brief

> **Superseded 2026-09-09.** The seven questions in section 4 were put to the
> author and ruled the same day; the controlling record is
> `book-1-democratic-and-administrative-integrity-decision.md`. This brief is
> retained as the dated measurement and inventory that preceded the ruling and
> is not edited further.

> **Status: brief only, 2026-09-08, measured at `83dd301`. This decides
> nothing.** It is the neutral inventory the tracker permits ahead of an
> outstanding author ruling. No predicate, rule, pin, fixture, constant,
> chapter, defect row, ledger row, receipt, posture, or public claim is created
> or altered here, and no needle in any reviewed source is touched. Options are
> presented with their costs; where the session has a reading it is marked as
> such and is not a ruling.

## Why this brief exists now

The tracker's next open item asks for parties and opposition, districting,
campaign finance, political advertising, lobbying, gifts, procurement,
conflicts of interest, revolving doors, corruption, and coordinated information
manipulation — with permitted writers, money and influence records,
proportionate disclosure, and oversight running "through the landed appointment
and anti-capture interfaces."

The measurement below finds that the item is **not one batch**. It is one
ratified, unimplemented slice that can proceed today, plus a much larger body of
work that has no ratified source text at all. CLAUDE.md governs the split
directly: "each contested rule family, dependent prose and public claim remains
gated until its own ruling lands. Neutral inventory and decision briefs may
proceed."

Writing the whole item as a single assurance batch would therefore require
inventing constitutional policy the author has not ruled. Writing none of it
would leave two **critical** scenario rows unaddressed whose closure condition is
already satisfiable.

## 1. The measurement

Taken 2026-09-08 against `constitution.nibli`, the reviewed ledger, the
237-row source-derived power census, and every ratified decision record.

### 1a. Ratified source text for the item's named subjects

| subject | occurrences in any ratified decision record |
|---|---|
| campaign (finance or otherwise) | 0 |
| lobbying / lobbyist | 0 |
| political advertising | 0 |
| gifts | 0 |
| conflict of interest | 0 |
| revolving door | 0 |
| corruption | 0 |
| party finance | 0 |
| gerrymander | 0 |
| coordinated information manipulation | 0 |
| districting | 3 mentions, all as a *delegated numeric choice* (see §2c) |
| procurement | 5 mentions, none about procurement integrity (see §2d) |

### 1b. The source-derived power census

`full-society-power-source-manifest.json` derives 237 candidate rows from the
ratified rulings at source grain. **None** is a campaign-finance, lobbying,
advertising, gift, procurement-integrity, conflict-of-interest, revolving-door,
anti-corruption, districting, or information-manipulation power. The census is
the mechanical answer to "what did the rulings actually authorise", and this
domain is not in it.

### 1c. What *is* formalized, and its exact shape

Three effects touch money in politics, all inherited from the economic ruling:

| id | title | posture |
|---|---|---|
| FS-CCE-266 | Enterprise Treasury Electoral Spending | Derived |
| FS-CCE-268 | Civic Association Enterprise Relabeling | Derived |
| FS-CCE-367 | Civic Union Political Finance Separate Treatment Denial | Derived |

Their entire formal content is one rule each, of this shape
(`constitution.nibli:1488`):

```
all $x: person($x) -> prevents($x, EnterpriseTreasuryElectoralSpending).
```

This is honest for what it derives — a registered person is protected against a
named boundary, and the negative test confirms an unregistered handle is not —
but it reads no treasury, payment, candidate, party, or spend. It is a **named
barrier, not a rule that catches a violation.** The idiom is the constitution's
dominant one for ratified prohibitions: 319 rules share that exact shape.

The coverage map's Democracy row states the same conclusion in its own words:
those three effects are formalized, and "unrelated democratic architecture
remains unimplemented."

### 1d. The corpus gate

The delivery family was blocked once by the lexicon, so it was checked first
here. Probing the nibli corpus for this domain's natural vocabulary:

| name | in corpus? |
|---|---|
| `bribe`, `corrupt`, `donate`, `fund`, `finance`, `lobby`, `campaign`, `advertise`, `disclose`, `declare`, `register`, `vote`, `elect`, `nominate`, `appoint`, `gift`, `give`, `spend`, `procure`, `tender`, `bid`, `conceal`, `publish`, `report`, `reveal`, `conflict`, `benefit`, `own`, `sell`, `buy` | **absent** |
| `influence` (`xlura`) | present — places `subject, influences, action, influence` |
| `control` (`jitro`) | present — places `manager, object, place3` |
| `depend` (`lacri`), `compete` (`jivna`), `promise` (`nupre`) | present |
| `pay` (`pleji`) | present but **unavailable** — `src/checks/repository.rs` holds it readable only by the two FS-CVF-018 supplement rules |

**Session reading, not a ruling:** the gate is real but not fatal, because the
recently landed families do not use domain nouns at all. FS-POW-064 and the
state-form family express everything through `observe/4`, `complete/3`,
`authorized/3`, scope constants, and `member/2` vocabularies. A family here
would be built the same way. `control` and `influence` are the two names that
could carry this domain's meaning natively; each would be a new admitted
consequential premise and is exactly the kind of addition that needs its own
ruling, not a drafting decision.

## 2. What is already ruled, and is not reopened here

### 2a. The anti-capture incompatibility — ratified

`book-1-state-form-and-political-membership-decision.md:205-208`:

> It is a legal incompatibility for a current government, chamber, party
> coalition, profession, or appointing source to obtain majority appointment
> control of a court or independent oversight body. Divided sources alone do not
> prove that the same coalition does not control them.

And at `:210-211`, a ratified formalization instruction that has not been
carried out:

> Future body cards must make both direct and de facto appointment control
> **observable** and allocate the exact seats, selectors, qualification
> authority, and fallback.

### 2b. The entrenched electoral corridor — ratified

`:102-108` fixes equal ballots, proportional outcomes, accessible
participation, **genuine opposition rights**, independent administration,
contestable certification, and no wealth, recognition, contribution, score, or
service-receipt weighting. Mechanics (MMP, STV, list) are delegated to ordinary
law inside that corridor.

### 2c. Districting — ratified only as a contract precondition

`:109-113` requires that "a Book 1 electoral card must define the outcome
metric, tolerance, district-magnitude and threshold constraints that make
'proportional' testable," and `:806-808` confirms the formal source "does not
invent a numeric age, duration, seat count, threshold, tolerance, district
magnitude, or fallback count." So districting has a ratified *testability
requirement* and an explicit refusal to supply numbers — not a substantive
anti-gerrymander rule.

### 2d. Procurement — routed to Book 2

Every ratified mention routes procurement to Book 2 operations or uses it in the
no-evasion list. Procurement *integrity* as a constitutional surface is unruled.

### 2e. Standing constraints that bind whatever lands

Unchanged and not reopened: no aggregate score or personal score anywhere near
these records; the temporary-assessment exclusion extended by name to every
risk, threat, loyalty, dangerousness, clearance, and watchlist product; bulk and
population-scale collection refused; the equality-diagnostics data wall; no
route through the compute backend; and liveness never Derived — a disclosure
duty is not evidence that anyone disclosed.

## 3. The one slice that needs no new ruling

Two **critical** scenario rows sit on exactly this slice, and both already carry
the closure condition "constitutional cases execute only after the relevant
author ruling and contract cards land" — the ruling exists (§2a); only the cards
do not:

- **FS-SCN-24** — *One coalition controls both political chambers.* Failure
  route: "Majority appointment control by one coalition is a legal
  incompatibility, challengeable independently."
- **FS-SCN-70** — *Connected offices fall under one controlling interest.*
  Failure route: "Majority appointment control by one interest is a legal
  incompatibility — challengeable independently, and divided sources alone prove
  nothing." Its `source_refs` cite the ratified sentence in §2a directly.

### The defect, measured

The ruling names five source kinds. The formal source names none of them.

| ratified source kind | constitutional constant |
|---|---|
| one current government | **none** (the `*Government*` constants carry tier, caretaker, and appointment-branch senses) |
| one chamber | **none** (`BothChambersParticipate` is a procedure-participation constant, unrelated sense) |
| one party coalition | **none** (`BoundPartyScope`, `EconomicBoundPartyBinding`, `ThirdPartyReproductiveVetoRefusal` are unrelated senses of "party") |
| one profession | **none** |
| one appointing source | **none** |

The whole incompatibility is carried by a single fixed attested token. Measured:

```
observe($source,   $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
observe($evidence, $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
observe($review,   $result, NoMajorityDirectOrDeFactoControl, AntiCaptureScope)   × 24
```

`AntiCaptureScope` has exactly one inhabitant. The 24 rule lines belong to
FS-POW-028 (independent-body appointment selection) and FS-POW-035 (court and
oversight seat allocation). Beside it sits
`CompletedDividedSourceSelection, SelectionDispositionScope` (8 rule lines) —
the very thing the ruling says "do not by themselves prove" absence of control.

Both atoms are result-level only. Neither is mirrored on the record or the
temporal record.

**This is the same defect shape the just-landed batch repaired for FS-POW-064**
(`733e5fb`): a ratified sentence that enumerates kinds, formalized as one
unanalysed attestation, so a finding passes by assertion rather than by
derivation. There, the repair was a `member/2` vocabulary with `derived_only`
and ground-headed rules, gate conjuncts on the reviewed-result rule,
record-level field mirroring, a producer-set check in
`src/checks/repository.rs`, and a hostile-widening counterfactual. The same
pattern applies here without any new author policy: the five kinds are quoted
from the ruling, not invented.

**Session reading, not a ruling:** this slice is a well-formed coherent
assurance batch today. It closes the gap between a ratified sentence and its
formal source, cites its own authority verbatim, needs no new corpus name, and
discharges the ratified `:210-211` instruction that appointment control be made
observable. It does not touch any subject in §1a.

What it would still **not** establish: that any body is actually independent,
that any concentration is detected, that a challenge is heard, or that capture
becomes impossible. The ruling says so itself — "This architecture reduces and
exposes capture; it does not claim that capture becomes impossible" — and the
batch's prose must carry that sentence's register.

## 4. Open questions reserved to the author

Each of these is a rule family with no ratified source. Listed with the fork the
author would be deciding, not with a recommendation.

**A. Money and influence records.** Does a constitutional money-or-influence
record exist at all, or is the domain left to ordinary law inside the entrenched
corridor? The economic ruling already bans enterprise-treasury electoral funding
and the civic/union relabel; a positive record is a different instrument. Cost of
"yes": a new admitted consequential premise naming who paid or influenced whom —
the finding-with-no-finder class chapter 1 conceded, and a standing
re-identification surface for small donors. Cost of "no": the three barrier
constants remain the whole of the design's answer on money in politics.

**B. Disclosure as a constitutional duty.** "Proportionate disclosure and
privacy" is the item's own phrase. A disclosure *duty* is expressible today
through the landed `obliged` bridge; a disclosure *arrival* is a liveness claim
and may never be Derived. Does the author want the duty formalized knowing the
non-response side is exactly what the design cannot prove?

**C. Conflicts, gifts, and revolving doors.** These are recusal and
incompatibility rules, structurally the nearest neighbours of §3. The state-form
ruling already requires every office to have "conflict and recusal rules." Does
that sentence authorise a family, or does it require its own ruling? Session
reading: it authorises office-bound recusal, and nothing about gifts or
post-office employment.

**D. Districting.** §2c gives a testability requirement and refuses numbers. A
substantive anti-gerrymander rule needs a metric, and a metric is a number.
This is the sharpest collision with the ratified no-numeric-values boundary and
the "never route a judgment through the compute backend" wall.

**E. Procurement integrity.** Currently Book 2 by every ratified mention. Moving
it to Book 1 is a scope decision.

**F. Coordinated information manipulation.** The highest-risk item in the list.
It collides with core liberties, with the refusal of population-scale
collection, and with the temporary-assessment exclusion — a "manipulation
finding" about a person is precisely the product that exclusion keeps out of the
consequential record. Session reading: this one needs its own ruling before any
inventory of it is safe to draft.

**G. Parties and opposition.** "Genuine opposition rights" is ratified (§2b) but
has no card. Party *regulation* — recognition, internal democracy, proscription
— is unruled and touches association liberty directly.

## 5. What this brief does not do

It creates no predicate, rule, constant, pin, counterfactual, contract card,
ledger row, defect row, receipt, posture, or public claim. It changes no
reviewed source, no needle, and no digest. It does not resolve FS-SCN-24 or
FS-SCN-70, does not alter the coverage map's Democracy row, and does not delete
or amend the tracker item. It is not an author ruling and does not anticipate
one; §3's "session reading" is a reading of already-ratified text, not a new
policy.

## 6. Sequence

1. The §3 anti-capture slice may proceed now as one coherent assurance batch
   under the ordinary protocol-v6 chain, on the ratified authority in §2a alone.
2. Questions A–G in §4 wait for an author ruling. Each that lands gets its own
   contract card and its own batch; none is a drafting decision.
3. The tracker item is deleted only when its whole scope has landed, so it
   survives step 1 and must be rewritten to record what remains — the tracker's
   own instruction: "Delete a bullet when it fully lands; update it if partly
   done."
