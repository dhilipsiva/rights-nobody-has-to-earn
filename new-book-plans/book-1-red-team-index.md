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
appointed body reviews. *Detected:* the appointment anti-capture idiom —
divided sources, staggered nonrenewable mandates, cause-only removal, majority
control by one government, chamber, coalition, profession or source as a legal
incompatibility. *Cases:* the state-form appointment family. *New veto,
surveillance or score:* none. *Limit:* divided sources do not by themselves
prove capture is absent, and the ruling says so.

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
asserted by membership so the set cannot grow unnoticed. Three of them need
nothing but the entry: an unsigned deceit entry takes the shield that exposing a
particular office holder earned, an unsigned deceit entry stops the accused being
recognised for anything, and two unsigned harm entries about one offender and one
victim raise the severity that decides where a convicted person is held. The
shield route is per claim rather than per person — a second, unaccused exposure
protects again, which the case pins beside the first. Voiding credibility is the
one that asks for more, and it asks for exactly one thing: a review body's
judgment beside the lie.
*Case:* `tests/pins/red-team/an-accusation-nobody-signed` pins those four in
sequence, including the person keeping personhood, the floor debt and liberty
throughout.

**The same relation sits at both extremes, which is the sharpest thing the
census found.** `prisoner` reads an injury entry too — inside the conviction
rule, where it is surrounded by a Court judgment, a cited case, a recorded
conviction, twelve independently witnessed observations, an active custody
authorisation and four negative guards. So the design already knows how to
surround an accusation before it acts on one. It does that for confinement and
not for severity, which is the same record deciding where the confined person is
then held.

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
