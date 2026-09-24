<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Red-team index: gaming the composed design

One entry per named strategic behaviour, under the author-directed TODO item.
Each says who gains, what they must know and coordinate, who pays the cost that
does not appear on the ledger, how it is detected or challenged, and whether the
design's answer creates a new veto, surveillance system or score. Where a legal
wall exists the entry names the executable case that tests it. **Where Nibli
cannot test the behaviour, the entry says so instead of implying a wall.**

The division is the assurance portfolio's: Nibli tests legal walls over supplied
records. Behaviour and scale belong to quantitative models, games and
simulations, empirical evidence and Book 2 operations — none of which is built.
Assume neither universal selfishness nor universal altruism; every entry below
describes what the rules do, not what people do.

## Tested legal walls

**Capture — one actor in every seat.** *Gains:* whoever controls the body being
reviewed. *Needs:* control of appointments, not information. *Hidden cost:* the
person the record is about, who now has no independent route. *Detected:* not
detected — refused, at the point of completion. Every family requires three
mutually distinct attesters, none of them the acting body, plus a challenge
reader and an independent alternate distinct from each other and from all three.
*Case:* `tests/pins/red-team/total-capture-one-actor-every-seat`, which puts one
actor in all six seats and completes nothing. *New veto, surveillance or score:*
none — the requirement is on the supplied credentials, not on watching anybody.
*Limit:* this refuses the arrangement in the record. It does not establish that
any real body is independent.

**Patronage — one source controlling appointments.** *Gains:* the appointing
source. *Needs:* control of a nomination route. *Hidden cost:* everyone the
appointed body reviews. *Detected:* the appointment anti-capture idiom — divided
sources, staggered nonrenewable mandates, cause-only removal, majority control
by one government, chamber, coalition, profession or source as a legal
incompatibility. *Cases:* the state-form appointment family and
`tests/pins/stress/captured-appointments`. *New veto, surveillance or score:*
none. *Limit:* divided sources do not by themselves prove capture is absent, and
the ruling says so.

**Bribery — paying for a decision.** *Gains:* the payer. *Needs:* a payment and
a willing office. *Hidden cost:* whoever the decision was supposed to serve.
*Detected:* the integrity family's money-and-influence record — payer kind,
instrument kind, recipient kind, with the controlling payer attested where the
nominal one is controlled — withholding public answerability for the office or
candidacy. *New veto, surveillance or score:* none on the person; the record is
actor-side and reaches answerability only. *Limit:* Nibli authenticates no
payment.

**Rent-seeking through licensing.** *Gains:* incumbents. *Needs:* a licensing
power and a plausible risk story. *Hidden cost:* everyone excluded from the
occupation and everyone who pays the resulting price. *Detected:* mandatory
licensing requires evidenced serious safety, fiduciary or core-public-function
risk, accessible alternative proof, portability, review, expiry discipline and
anti-cartel safeguards. *Cases:* the economic licensing card, `FS-POW-061`.
*New veto, surveillance or score:* none.

**Regulatory arbitrage — move the act.** *Gains:* whoever is refused here.
*Needs:* another jurisdiction or a friendly affiliate. *Hidden cost:* the people
in whichever place has the weaker rule. *Detected:* every record rejoins its own
jurisdiction, and the no-evasion rule generalises across trade, procurement,
affiliates, supply chains, flags, arbitration forums and exported enforcement.
*Cases:* the mobility family's external no-evasion cases; the ecological
no-evasion rule. *Limit:* effective control is the stated test, and no rule
observes where anything actually happened.

**Strategic withholding — never answer.** *Gains:* the reader who would have to
act. *Needs:* nothing at all, which is what makes it the cheapest attack here.
*Hidden cost:* the requester, who waits. *Detected:* a certified reader
nonresponse is a positive finding that moves the duty to an independent
alternate; silence itself still proves nothing either way. Every completion
admitting a challenge reader carries an alternate behind them, checked across
families. *Cases:* each family's `request-and-certified-nonresponse`. *New
veto:* the opposite — this exists to remove one.

**Misreporting — attest a convenient value.** *Gains:* whoever wants the record
to say something else. *Needs:* one attester. *Hidden cost:* the record's
subject. *Detected:* two attesters disagreeing about a single-valued scope raise
a record ambiguity and nothing completes, so a reader cannot pick whichever
value would work. *Cases:* each family's `conflicting-*` cases. *Limit:* this
catches disagreement, not agreement — see collusion.

**Burden-shifting between domains.** *Gains:* whoever was refused in one place.
*Needs:* an authorization of the right shape elsewhere. *Hidden cost:* the
person the second domain's rules were protecting. *Detected:* every dependent
record rejoins the exact authorization it names — subject, domain, purpose,
version, period, jurisdiction, scope, end — rather than any authorization of the
right shape. *Case:* `tests/pins/red-team/burden-shifting-between-families`,
plus each family's `wrong-*` cases. *New veto, surveillance or score:* none.

**Goodhart — optimise the measure.** *Gains:* whoever is measured. *Needs:* a
measure. *Hidden cost:* whatever the measure displaced. *Detected:* there is no
measure. No enacted line carries a numeric literal and no relation aggregates,
scores or ranks, checked by `floor_vector_tests`. *Limit:* this is a property of
the current rule text, not an impossibility; a new relation name routes around a
name-based check.

**Calling a choice a shortage.** *Gains:* whoever did not procure. *Needs:* the
word. *Hidden cost:* whoever goes without. *Detected:* seven refused grounds
named — budget choice, price exclusion, administrative delay, artificial
withholding, monopoly, provider failure, refusal to procure — each with its own
case, each withholding the finding it targets and owing back what was withheld.
*Cases:* `tests/pins/scarcity/priority/not-a-shortage-*`.

## Stress tests — 2026-09-24

Item 45 of the revision tracker pins six scenarios the outside revision plan
named, each in `tests/pins/stress/`. Each case states what the design does and
where it stops; a `:defect` pin marks each confirmed gap, and each gap has its
own tracker item (67–72) under the resolve-before-defending rule.

**Captured appointments — two nominally separate selectors, one coalition.**
*Gains:* the coalition. *Needs:* three attesters willing to certify that no
source holds majority control. *Hidden cost:* everyone the appointed body
reviews. *Detected:* one attester's refusal to certify withholds the selection;
after certification, a contrary record by one of its own attesters, or an
independently reviewed appointment-control finding naming the controller,
withholds it (item 67), and a fallback for the same seat is owed its end.
*Case:* `tests/pins/stress/captured-appointments`. *New veto, surveillance or
score:* none.

**An election that cannot be held.** *Gains:* whoever holds office when the
election falls due. *Needs:* a real hazard. *Hidden cost:* the electorate.
*Detected:* a declaration used to delay the election or extend a mandate is
withdrawn with every measure resting on it, and the caretaker holds no
confidence mandate. *Gap:* holding the election is a power, not a duty, and
nothing governs a lapsed Assembly (item 68). *Case:*
`tests/pins/stress/election-cannot-be-held`.

**Manipulated urgency in a scarcity allocation.** *Gains:* the claimant whose
urgency decides the allocation. *Needs:* an attester's credential for the
allocation record. *Hidden cost:* the other claimant. *Detected:* one honest
attester who records a different mitigation key makes the record ambiguous and
the allocation falls; a challenge with the requester's own evidence obliges the
reader to weigh it. *Gap:* a compared claimant may attest (item 69). *Case:*
`tests/pins/stress/manipulated-urgency`.

**Agency and contractor blaming each other.** *Gains:* both, by delay. *Needs:*
a dispute. *Hidden cost:* the person waiting. *Detected:* not needed — the debt,
the common tier's backstop and the region's duty read personhood and presence,
not fault, and split blame moves no continuity because the backstop never
depended on it. *Gaps:* continuity can be certified onto a private contractor,
and the duty-bearing region can witness its own contractor's delivery (item 70).
*Case:* `tests/pins/stress/agency-and-contractor-blame`.

**An amendment blocked by an expansive reading of the core.** *Gains:* whoever
holds the effect reviewer's seat. *Needs:* that seat. *Hidden cost:* the
proponent and everyone the amendment would protect. *Detected:* the block holds
one record only; a fresh record with its own reviewer is not reached by it, and
the reading creates no right. *Gap:* the contrary value needs no reasons, and
the proponent's challenge reaches no decision that can certify (item 71).
*Case:* `tests/pins/stress/core-read-expansively`.

**A person nobody has recorded.** *Gains:* nobody; this is the design's own
blind spot. *Hidden cost:* the person. *Detected:* a first-contact entry by
whoever made the contact gives standing, every debt and the advocate's duty,
with serve-first and no enrolment. *Gap:* a report that nobody is acting for
the person reaches nothing until the encounter is also written (item 72).
*Case:* `tests/pins/stress/never-recorded-person`.

## The wall that is not there

**Collusion — attesters who agree.** *Gains:* all of them. *Needs:* three named
writers willing to sign the same falsehood, plus a reader and an alternate who
do not object. *Hidden cost:* the record's subject, entirely. *Detected:* **not
detected.** Collusion and honest agreement are the same shape, and nothing here
authenticates a witness or checks that attested evidence is true, so a
restriction founded on three matching lies completes exactly as one founded on
three matching truths. *Case:* `tests/pins/red-team/collusion-three-attesters-who-agree`
pins that outcome TRUE deliberately, so the boundary is executable rather than
only written down.

What the design does instead is raise the price: three distinct named writers, a
reader, an independent alternate, published reasons and a challenge route, all
attributable to somebody. That is a cost, not a barrier, and detection is
external. Every contract card that says "these are conditions on supplied
credentials, not proof of independence in practice" is saying this.

**An accusation nobody signed.** *Gains:* whoever writes it. *Needs:* one line,
and for the severity route two. *Hidden cost:* the person it names, who has no
author to challenge. *Detected:* **not detected.** The harm relations name the
alleged offender and the person harmed, never the writer, so there is nobody to
put a question to. Thirteen rules read one of these relations, censused by
`floor_vector_tests::an_unsigned_accusation_reaches_exactly_the_measured_set` and
asserted by membership so the set cannot grow unnoticed. **Two still need
nothing but the entry**: an unsigned deceit entry takes the shield that exposing
a particular office holder earned, and an unsigned deceit entry stops the accused
being recognised for anything. The shield route is per claim rather than per
person — a second, unaccused exposure protects again, which the case pins beside
the first. Both withdraw something by absence rather than concluding anything,
which is why they are described rather than repaired: an accusation not yet
adjudicated is what the shield was built to ignore.
*Case:* `tests/pins/red-team/an-accusation-nobody-signed` pins the routes in
sequence, including the person keeping personhood, the floor debt and liberty
throughout.

**The third route was repaired on 2026-09-16, and the census is what found it.**
`prisoner` reads an injury entry too — inside the conviction rule, surrounded by
a Court judgment, a cited case, a recorded conviction, twelve independently
witnessed observations, an active custody authorisation and four negative guards.
The same relations were raising severity, which decides where the confined person
is then held, with nothing at all. Severity now reads the Court's judgment and
its cited case: the two facts the confinement it decides had already required, so
every person severity can reach already had them and the repair moved nobody —
Lalo, Don and Ruk are pinned unchanged. *Control:*
`tests/pins/red-team/counterfactual-severity-without-the-court` strips the
conjuncts and watches two unsigned entries derive severity again.

**The census counts two conviction routes reading that injury entry, not one,
since the shield was scoped on 2026-09-19.** Both sit behind the same T3 gate and
differ only in what they ask about the shield: the first that none is held, the
second that two answerable bodies the person did not expose have found this
prosecution unrelated to their disclosure. An unsigned injury entry is no nearer a
conviction for there being two doors behind the same wall — every conjunct the
first route requires, the second requires — but the count is recorded here because
`an_unsigned_accusation_reaches_exactly_the_measured_set` asserts it by
membership, and a route added without this note would fail that test rather than
pass quietly. Controlling record:
`book-1/appendix/decisions/shield-scope-decision.md`.

**It gives no accusation an author, and the entry says so.** The harm relations
still name the offender and the person harmed and never the writer. What changed
is that an entry nobody signed can no longer conclude something adverse on its
own; it now needs a court that judged the person and cited a case. The remaining
two routes take by absence, and the finding-with-no-finder class chapter 1
concedes is still open.

What the design does instead is far less than in the collusion entry, and it
should not be dressed up. The entry is visible, it names the person it is about,
and it sits where it can be disputed by whoever notices — which is more than a
file of impressions gives anybody, and is the whole of it. Chapter 1 states this
per route rather than in general, because stating it in general got it wrong in
both directions at once.

## Outside what Nibli can test

These are behaviours at scale, and no rule over supplied records reaches them.
They are recorded so the absence is deliberate rather than an oversight.

**Adverse selection** and **moral hazard** need a population responding to an
incentive over time. **Free-riding** needs a contribution decision the design
deliberately does not record — it demands nothing of the recipient, which is the
thesis, and the cost of that choice is exactly that free-riding is unobservable
here. **Black markets** need prices and quantities; the design's answer to
exclusion by price is the false-scarcity finding, which addresses the public
actor's account of a shortage, not the market that forms around it.
**Collusion's detection**, above, is the same kind of gap.

Quantitative models, games and simulations, empirical evidence and operational
assurance are the assigned routes for all of these. None is built. No claim
about any of them may be made from a green verifier.
