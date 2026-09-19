<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The whistleblower shield: scoped to the case

## Positive reviewer eligibility — item 10 supersession, 2026-09-19

The current design replaces the blanket `~show` exclusions described in the
historical decision below. It does not retain them as a fallback. This is a
substantive supersession under delegated approval, required by the
resolve-before-defending rule: naming all reviewers was a repairable defect,
not a demonstrated necessary cost of protecting disclosures.

The old alternative is retained as `counterfactual/shield-disclosure-veto` in
the substantive suite. Given the same case, favourable independent
qualification and unrelatedness findings, naming the reviewers keeps the
conviction blocked under that alternative. The current rule permits the
unrelated prosecution without rejecting a disclosure or reading the
claimant's credibility. Both alternatives run against the full current
constitution with the one exact rule substitution declared in the suite.

### Current legal rule and supplied premises

The two reviewers still need public answerability and a matching finding that
this prosecution of this person is unrelated to the disclosure. Each also
needs `authorized(_, ShieldConnectionReviewer, case)`: a lawful, current
mandate to decide that particular connection question, not an inference from
permanent answerability. The rule retains the ordinary conviction conditions,
including the current custody authority, appellate relief and release guards.

Reviewer eligibility is positive. A `ShieldQualificationAuthority` and a
separate `ShieldQualificationReviewAuthority`, each authorised for the case,
must both supply `IndependentShieldReviewerScope` for each decider. This is a
bounded finding of lawful current appointment and actual independence from
the parties and disclosures, reached with notice, reasons, an opportunity to
be heard and independent challenge. These are functions within the existing
appointments and justice settlement, not a new final court or an operating
institution claimed to exist. Assignment, actual independence, procedure and
authenticity remain externally supplied evidence; Nibli does not perform or
authenticate them.

The qualifier, qualification reviewer and both deciders are pairwise
distinct. None may be the offender, injured person or prosecuting Court.
Every authorisation and attestation binds the same case, so one case's
qualified review cannot lend authority to another. The existing justice
access, interim-relief, independent challenge, nonresponse and alternate
duties remain applicable. Their performance is not inferred from the finding.

An authorised `ConflictedShieldReviewerScope` finding by either qualification
function derives `contradict(case, participant, ShieldReviewerConflict)`.
Any participant in the deciding or qualification functions with that finding
is barred. The contradiction is shared across candidate certifier pairs: a
second favourable certificate cannot erase it. A raw disclosure or an
unauthorised conflict allegation cannot supply the adverse finding. No claim
about the defendant's honesty, sincerity or general credibility decides
eligibility for the shield.

### Alternatives and the remaining cost

Keeping the blanket exclusion preserves a claimant's unilateral veto over
all deciders; the counterfactual executes that failure. Removing it without
replacement would allow unsupported agreement or self-qualification to
overcome protection, so the positive mandate, qualification and separation
conditions are required. Letting the prosecuting Court decide, combining
qualification with decision, and allowing a party to fill either role are
expressly barred and tested. A positive conflict beats favourable
certification, including when different certifiers are offered.

Requiring the discloser to win a retaliation hearing before protection begins
would expose them to conviction during a withheld review. Automatically
expiring the shield would have the same defect when the hearing never
occurred. The current default remains protection pending a sufficient case
finding. Missing qualified adjudication still delays an unrelated prosecution;
the victim keeps assistance, protective and remedial routes, while no
ordinary protective measure may borrow the shield decision as its own ground.

The repair removes the **allegation-as-disqualification** mechanism. It does
not prove prompt hearings or independence in fact. The necessity that remains
is narrow: a conclusion requiring positive eligibility cannot derive when
that evidence is absent. This says nothing about an inability to obtain that
evidence, and is not a proof that all possible institutional arrangements
share the same delays. A procedure offering equivalent immediate protection
with fewer obstructed unrelated cases would weaken the argument for this
choice. Evaluation must count missed retaliation as well as delay and must
not mistake supplied favourable findings for successful operation.

### Coverage and prose

`tests/pins/shield-independence/` runs the positive case with every decision
participant named, the old-veto counterfactual, removal and wrong-case
substitution of every required positive premise, every pair of fused
functions, every function occupied by defendant/victim/Court, conflict in
each function, attempted certifier shopping, uncredentialed noise and a
conflict in another case. Chapter 24 also executes the ordered sequence:
agreement without qualification, qualified unrelated-case decision, fresh
disclosures, and a subsequent established conflict. Sly's court-exclusion
test supplies all other positive conditions so the court bar remains live.
The shipped cast carries none of these new authorisations; the new route is
exercised through fixtures.

The exact Chapter 24 and Part V prose is retained in its canonical files as
`session-drafted, author-approved under delegated approval (2026-09-13)`.
The existing resolution entry is updated in its owning JSON; no new receipt
or administrative gate is introduced.

## Historical first scoping decision — superseded as stated above

Ruled 2026-09-19 under the standing delegated approval, against the revision
tracker's *resolve before defending* rule. It changes Article 6's conviction
gate, leaves Article 7's shield rule untouched, and records the alternatives
examined.

## What the shield was, measured before changing it

`defend/1` has exactly one reader: the conviction rule's `~defend($offender)`.
Measured against the source at `69dd2712`, that conjunct is unscoped, so a
single `show($w, $o)` naming any authority — and answerability is never revoked,
so the eligible set only ever grows — blocks every conviction of that person,
for any offence, indefinitely, with no finding required and none possible
except a deceit finding against that particular exposure.

The cast shows the cost directly. Rex injured Sena, was judged, was found to
have lied about the court in order to escape the prosecution, and had his
credibility voided for it. Rex then exposed the review body. Nobody has
examined that exposure, so the shield holds and Rex is not a prisoner.
Measured: Rex carries the whole custody chain — `match(Rex,
ConvictionRecorded)` and `correct(Case_Rex, ActivePower)` both derive — so the
shield is the only thing blocking the conviction.

## What changed

A second conviction route, beside the existing one rather than replacing it.
It carries the same T3 conjuncts and swaps `~defend($offender)` for the
positive case: the person is shielded, and two answerable bodies have recorded,
against this exact case and this exact person, that the prosecution is
unrelated to the disclosure — `observe($body, $case,
ShieldUnrelatedToTheDisclosure, ShieldConnectionScope)` beside
`observe($body, $case, $offender, ShieldSubjectScope)` — with
`~($reviewer = $second)` and `~show($offender, $reviewer)`,
`~show($offender, $second)`.

Four properties, each deliberate.

- **The polarity is unchanged.** Absent the finding, the shield holds. A
  captured court still cannot jail a whistleblower faster than an honest review
  can convene, which is the 2026-08-02 ratified default, and nobody loses a
  protection on nobody's finding.
- **The scope is the case.** A finding for one case says nothing about another.
  `defend` is untouched and still derives; what stops is its reach into the one
  case named. Neither the finding nor its absence reaches standing, the floor,
  the ballot, recognition or credibility.
- **The decider may not be the accused.** `~show($offender, ·)` bars each body
  from being one this person exposed. Without it the route is worthless: the
  body with the strongest motive to call a retaliatory prosecution unrelated is
  exactly the body the disclosure named.
- **Two bodies, not one.** Measured on the cast: Sly exposed the court, so the
  court is barred and the review body is eligible and alone; the pair does not
  complete and `prisoner(Sly)` stays FALSE. One eligible body is not enough.

Rule count 7107 → 7108. Predicates, derived predicates, strata and the floor
are unchanged; the shipped cast holds no `ShieldConnectionScope` observation,
so the route ships dormant and no conviction moves.

The Rex sequence resolves in `book-1/24-the-shield.pins.nibli`: Rex exposed the
court and the review body, so neither may decide; `Appeals` and `Convocation`
record the finding, `prisoner(Rex)` derives, and `defend(Rex)` and
`show(Rex, Review)` both stay TRUE.

## What was not done, and why

**The shield is not given a second place.** `defend($w, $case)` was the first
design tried. Writing the coverage as
`defend($w) & ~unrelated($w, $case) -> defend($w, $case)` leaves `$case`
appearing only under negation, so the engine must range it over every constant
in a 55 MB knowledge base; the probe ran past ten minutes against a suite whose
ordinary case costs eleven seconds, which is the unbound-variable expansion the
multi-power capability audit records. Binding `$case` positively instead means
binding it where it already is bound — in the conviction rule — which is what
the second route does. The arity change would also have moved every `defend`
pin in the suite for no gain.

**The finding is not inferred from an ordinary case citation.**
`cite(Court, $case, $offender)` is present in every prosecution, so reading it
as evidence that the prosecution is unrelated to a disclosure would empty the
shield entirely. The finding needs its own marker, and `observe/4` carries
writer, record, value and scope, which is the vocabulary every landed family
uses for exactly this.

**Fail-closed protection is refused, again.** Requiring a positive retaliation
finding before the shield attaches is the arrangement the 2026-08-02 polarity
ruling rejected, and the reasoning is unchanged: it hands whoever declines to
adjudicate a veto over every shield, and the person it strands is the one the
instrument exists for.

**A person who exposes every answerable body keeps the shield in every case.**
This is the chosen direction of failure and it is not repaired. The eligible
set is the answerable bodies minus those the person exposed; exposing all of
them empties it. The alternative is allowing an accused body to clear the path
to the conviction of its accuser, which is the sequence the shield interrupts.
Who bears the cost: whoever the unrelated offence was against, in the narrow
case of a defendant who has exposed every answerable body. What bounds it: the
exposures are each individually examinable, the deceit route voids a serial
false accuser's credibility, and each new exposure is a fresh attributable
write. What would reopen it: a representation in which eligibility to decide
can be established positively rather than by the absence of an exposure, which
would need a conflict finding this design does not currently hold.
