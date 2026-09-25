# The Method

This optional part is the book's technical appendix: how to inspect a claim,
what the machine checks, and where each check stops. To inspect a claim, find
the rule and the supplied facts, ask what follows, and try a case that should
give a different answer. The machine checks those consequences; whether the
English describes them is a separate review.

I use Nibli because it makes the relationships in this proposal explicit. Facts
name people, acts and records; rules state which combinations permit a
conclusion. The language distinguishes an entry from a conclusion nobody may
write and refuses certain circular uses of absence. That helps test this
design; it does not make every policy Nibli accepts defensible.

The examples quote formal statements and the verdicts executable tests expect,
wrapped for the page, with no machine-written explanation or proof transcript.
The complete constitution is in the
[repository](https://github.com/dhilipsiva/rights-nobody-has-to-earn). [The
book's website](https://dhilipsiva.dev/rights-nobody-has-to-earn/) publishes
the same constitution as numbered plain-language articles and runs selected
records against it in the browser.

## From a supplied fact to a consequence

Nell's case begins with one entry, a fixture rather than a report of a birth
outside the model:

```nibli
born(Nell).
```

The constitution contains these statements:

```nibli
all $child: born($child) & ~public($child) -> person($child).
all $x: person($x) -> owe(State, Eats, $x).
entitled(every person, event { eats() }).
```

A relation name precedes its arguments. A name beginning with `$` is a
variable, which `all` quantifies; `&` joins conditions and `->` points to the
consequence. The first rule reads a birth and the absence of a public-body
status and concludes personhood; the second concludes that the State owes that
person food. The entitlement has a different shape: `event { eats() }` names
the content of the right without asserting that anybody ate. Chapter 1's tests
ask:

```nibli
? person(Nell).
# => TRUE
? owe(State, Eats, Nell).
# => TRUE
? eats(Nell).
# => FALSE
```

The line beginning `# =>` states the verdict the test requires. Nibli uses a
closed world and a closed domain: what cannot be derived from the supplied
knowledge is `FALSE`, and quantifiers range over what that knowledge names. So
`eats(Nell)` returning `FALSE` does not establish that a child went hungry; it
says the case supplies no evidence from which eating follows.

The constitution also derives standing from supplied contact, presence,
effective control and a report that nobody is acting for someone, and it
assigns an initiation duty without requiring Nell to ask. None of those rules
lets the engine discover a person whose encounter never reaches any input. The
case is in [Chapter 1's pins](01-the-child-with-nobody.pins.nibli) and the
rules in the [constitutional source](source/constitution.nibli).

## What changes when evidence is supplied

The food route requires a receipt and a matching independent attestation:

```nibli
all $p: all $item: all $src: all $w:
  receives($p, $item, $src) &
  authorized($w, DeliveryWitness, $p) &
  observe($w, $item, $p, FoodScope) &
  ~($w = $src) & ~related($w, $p, FloorDutyBearer) &
  ~($w = State) & ~($w = CommonTier) -> eats($p).
```

A repeated variable must name the same thing each time. The witness must be
authorised for this recipient, attest this item at food scope, and be neither
the source nor one of the public bodies whose duty the delivery discharges.
Chapter 4 supplies the evidence in stages:

```nibli
receives(Marisol, FoodForMarisol, Provender).
? eats(Marisol).
# => FALSE
authorized(Ledgerwitness, DeliveryWitness, Marisol).
observe(Ledgerwitness, FoodForMarisol, Marisol, FoodScope).
? eats(Marisol).
# => TRUE
? dwell(Marisol).
# => FALSE
```

The receipt alone is not enough, and food evidence yields no shelter
conclusion. A declared counterfactual removes only `~($w = $src)`, and a
provider certifying its own delivery then succeeds, which ties the refusal to
that one condition.

Adding evidence and changing the constitution ask different questions. The
ordinary case asks what the rule does with those facts; the altered source asks
what the condition prevents. The runner applies the declared edit to an
isolated copy, so nothing is amended. The [receipt
suite](source/delivery-receipt.pins.nibli), its
[counterfactual](source/counterfactual/no-delivery-independence.pins.nibli) and
[Chapter 4's sequence](04-whether-it-arrived.pins.nibli) hold the cases. The
positive result rests on supplied statements: false evidence in the required
shape also returns `TRUE`.

## A request, a duty and relief

`permits(Appeals, Ruk)` sounds like permission to lodge an appeal; in this
constitution it means appellate relief already granted. The request route is
separate:

```nibli
all $reader: all $request: all $requester:
  challenge($requester, $reader, $request) &
  authorized($reader, JChallengeReaderAuthority, $request) &
  ~($requester = $reader)
  -> obliged($reader, ReviewJusticeRequest, $request).
```

The requester needs no role or previous permission; the reader needs a mandate
for the request and must be someone else. What follows is a duty to review, not
a favourable decision or a record that a hearing occurred. Relief is a third
thing: a case-specific bar that only a qualified relief record, with its
subject, case, grounds, evidence and independent actors, produces. In [Chapter
21's pins](21-a-way-to-be-heard.pins.nibli), Nia's qualified final order bars
her confinement and Ruk, with none, stays held. A pending filing is a fourth
thing again: it suspends custody authority while the challenge is decided,
without pretending to be a final judgment.

A rule granting every confinement an automatic bar against itself is refused:

```nibli
:refuse reasoning /Unstratifiable/
all $subject: all $case:
  prisoner($subject, $case) -> clean($case, CustodyConfinementBar).
```

Custody requires the bar to be absent, and this rule would produce it from
custody: a negative cycle. Nibli layers its relations so that any use of
absence reads a lower layer, and this loop cannot be layered. The refusal
concerns the encoding, not the policy: the same file accepts a duty to hear
every prisoner and shows it grants no relief.

```nibli
:accept
all $x: prisoner($x) -> obliged(Appeals, $x).
? obliged(Appeals, Ruk).
# => TRUE
? permits(Appeals, Ruk).
# => FALSE
```

An `:accept` keeps the added rule for the rest of that test; `:accept-scoped`
checks that a statement loads and then restores the prior state; and each case
runs in its own knowledge base.

## A refusal is a result about an input

Some inputs are refused before any query, and some rules when they are added.
The layering that refuses a rule concerns how relations depend on one another,
not reading order, which the [contents](contents.json) set editorially. The
constitution declares its permitted base vocabulary with `admits` and marks
conclusions nobody may write with `derived_only`. Chapter 2 executes both:

```nibli
:refuse reasoning /not admitted vocabulary/
rich(Adam).
:refuse reasoning /declared derived-only/
prisoner(Zed).
```

The directive names the expected error; if the statement loads, or fails for
another reason, the test fails. Neither refusal says Adam is poor or Zed
innocent, only which statements the interface accepts; a permitted entry can
still be false. The cases are in [Chapter 2's
pins](02-what-the-record-may-say.pins.nibli).

### Rejecting a whole input

When a permitted entry shares one assertion with a forged conclusion, the whole
assertion is refused. [Chapter 18's
pins](18-the-vote-conviction-does-not-take.pins.nibli) admit the name `decide`
and then submit `person(Vote_Probe) & decide(Vote_Probe, Ballot).`; the refusal
rolls back the person entry too, and `person(Vote_Probe)` stays `FALSE`. An
admission cannot override a derived-only declaration. The same file then adds a
narrower ballot rule and keeps it loaded:

```nibli
:accept
all $x: person($x) & match($x, GeneralAdult) & ~prisoner($x)
  -> decide($x, Ballot).
? decide(Hano, Ballot).
# => TRUE
```

Hano's ballot still follows from the broader rule: an added sufficient route
repeals nothing, though elsewhere an added fact can defeat a condition written
as an absence.

### Why the hostile floor rule is refused

The floor has a designed protection. Adding this rule is refused:

```nibli
:refuse reasoning /'prisoner' -> 'eats'/
all $x: person($x) & ~eats($x) -> prisoner($x).
```

An entitlement's event body gives the engine a dependency on the floor
condition without asserting that anyone received it, and a prisoner remains a
person. A rule confining someone for lacking a floor condition would feed that
absence back into custody, which the layering refuses. The protection is
specific to how the hostile rule is written: a rule confining someone for
lacking a home record loads, because home status sits outside that loop, so a
development check forbids it instead. A rule making an absence of belief a
ground for a credibility finding is refused, because that finding lies inside
the same graph. Neither result settles every differently expressed attack, and
an accepted alternative acquires no authority in the constitution; each
experiment is discarded after its check. Two counterfactuals show what the
refusal rests on: [removing the
rule](source/counterfactual/no-person-line.pins.nibli) that keeps a prisoner a
person lets the belief rule load, and [writing the belief entitlement as a
plain label](source/counterfactual/entitlement-as-plain-label.pins.nibli) makes
it disappear and lets the same rule load, while the food entitlement, still
written as an event, keeps its refusal. [Chapter 3's
pins](03-what-you-are-owed.pins.nibli) and the [floor
suite](source/rights-floor.pins.nibli) hold the rest.

### Checking how a record is used

A contribution record is a base input, so the engine accepts a rule confining
someone for lacking one; no cycle forms. Two development checks in [the floor
development tests](../src/authoring/floor_vector_tests.rs) do that work
instead: one lets a contribution or pay record be read only for its supplement
or compensation, never under negation, and the other rejects any rule taking a
delivery conclusion, present or absent, as a premise. Both read the written
rules, so an equivalent rule in another form can pass them.

## Six ways a fact is kept from a consequence

A right is protected by limits on what the rest of the law may do with facts
about the person:

| Constraint | What it stops | Where it is enforced |
|---|---|---|
| Closed inputs | An entry of a kind not admitted, such as a record that someone is rich or dangerous. | `admits`, refused at input; [Chapter 2's pins](02-what-the-record-may-say.pins.nibli). |
| Conclusions nobody may write | A written custody, ballot or answerability conclusion. | `derived_only`, refused at input; Chapters 2 and 18. |
| Purpose-bound reads | A contribution or pay record read for anything but its own purpose. | `a_purpose_limited_record_is_read_only_for_its_purpose`. |
| Endpoints nothing reads | A duty, a delivery conclusion or a recorded loss used as a premise. | `a_duty_is_not_an_action_because_nothing_reads_one`; `floor_actualities_have_no_downstream_consumer`. |
| No confinement from absence | A missing floor condition, or a missing home, family or work entry, used to confine. | The refusal above; `no_confinement_reads_an_absent_home_family_or_work_entry`. |
| Scope binding | A finding about one subject, case or record lent to another. | Case-bound joins and their wrong-case cases; `scoped_authority_is_not_unary_permanent_answerability`; `global_findings_cannot_lend_effects_to_unqualified_records`. |

These are information-flow constraints, in the line of Dorothy Denning's
lattice model, Andrew Myers and Barbara Liskov's owner-controlled flows and
taint analysis; the purpose-bound reads apply what Helen Nissenbaum calls
contextual integrity.[^flow] Here the information is a fact about a person and
the sensitive use a consequence for them. Each constraint is checked on the
written form of the rules, so a rule reaching the same consequence under a new
relation name, or through a shape no check inspects, can pass.

## Versions of the record

A snapshot of the record gives way to a successor only through a transition
both timing witnesses, `Chronicle` and `TemporalReview`, attest; two
predecessors, two successors or a cycle is a collision no transition survives,
and only the last accepted successor gives an adverse status or a public power
its effect. A status passes on only when both witnesses record that it carries;
one the transition omits ends with the old version as a named defect. Standing
works the other way round, because it protects:

```nibli
all $x: all $after: all $before:
  succeed($after, Transition) &
  replace($after, $before, Chronicle) &
  authorized($x, StandingStatus, $before) &
  complete($x, StandingScope) &
  observe(Chronicle, $x, $before, StandingScope) &
  observe(TemporalReview, $x, $before, StandingScope) -> person($x).
```

A witnessed standing status in the predecessor keeps its subject a person even
when the successor omits it. The custody authorisation and its review date, a
lease and its window in the source, apply the same mechanism to one case:
custody follows only while a current, witnessed lease names the case, court,
subject, period and source, and a missing or stale renewal ends the authority
without releasing anyone.

## Comparing and selecting an amendment

Nibli checks an amendment's supplied certification, publication and selection
evidence. A separate [amendment host](../src/amendment_host.rs) compares the
exact candidate and base text and models selection in memory; its [development
tests](../src/amendment_host_tests.rs) show that one added newline defeats
certification, that certification alone selects nothing, and that a candidate
authorised against a replaced base is stale. These are checks over trusted
input: they authenticate, publish and deploy nothing.

## What a contradiction check establishes

A query can match its expected answer while the model holds a conflict
elsewhere, so the runner also asks for a contradiction report wherever a case
declares one; supplied `person(Ara).` and `~person(Ara).`, its development test
fails. The engine checks represented constraints, explicit negative assertions,
arity and equality conflicts, and reports separately any check it could not
finish; both lists must be empty. It is not a decision procedure for
unrestricted first-order consistency, and an injustice the model does not
represent as a conflict does not become one because a scan completed. A refused
assertion is different again: it never entered the knowledge base.

| Outcome | What it says |
|---|---|
| `TRUE` | The query follows from the loaded facts and rules, if the implementation is correct. |
| `FALSE` | The query does not follow. It is not an observation that the event did not occur. |
| `UNKNOWN` | The search did not decide the query. |
| `RESOURCE_EXCEEDED` | A resource bound stopped the search; the verifier counts that as a failure to complete. |

The engine's [guarantees
document](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md) states
its inference contract.

## What has been checked, and where each check stops

Each assurance below says what was checked, by which route, and where it stops.
None is added to another or reduced to a score, because a total would hide the
one that failed.

**Pins and contradiction scans.** `./verify.sh` executes every case in the
[inventory](../tests/pins/suites.json): each chapter's pins, the families'
generated cases, the temporal, amendment and placement cases and the declared
counterfactuals, with contradiction scans where asked. It establishes
consequences of the supplied records under the loaded rules, and stops at the
prose, the truth of the evidence and every question of operation.

**Rules checked by their shape.** Development tests inspect the written rules
rather than run them: the six constraints above; four checks that every
attester records every field of its record, that each field is attested the
same way across its family, that a single-actor route asks its one actor for
everything the full route asks, and that both timing witnesses record every
field; and a check holding help and harm to their different procedures. They
stop at the written form, which a new relation name can route around.

**What each act needs before it takes effect.** A generated table,
[`procedural-load.md`](source/procedural-load.md), lists for every effect the
roles that must act on its record, with a reviewed class: beneficial, adverse,
power over others, oversight or institutional.

- An act that only gives or preserves something for its subject takes effect on
  its source's record alone; a named independent reviewer owes prompt review
  and can withdraw it.
- An adverse act, and any appointment giving power over another person, waits
  for its full record: in the newer families a source, an evidence attester and
  an independent reviewer recording the same fields, a challenge reader and an
  independent alternate.
- A delivery, supplement, compensation or certificate conclusion needs one
  authorised witness, adjudicator, attester or certifier, held apart from the
  party its rule names.

The roles are read from the rules; each class is a recorded judgment with its
reason.

**Rules deliberately broken.** `tools/mutation.py` drops a condition, negates
one or swaps two roles in one rule, then runs the cases whose pins ask about
that rule's conclusion; the recorded run sampled three rules per family with
seed 49. A change no selected pin notices is a survivor, and the [survivors
list](source/measurements/mutation-survivors.md) gives each a disposition:

- Changes that drop or negate one attester's copy of a field the other
  attesters still record, or ask a single-actor route for less than the full
  route: no generated case sees them, because an omission case removes a field
  from every attester at once, and the four shape checks above reject each one.
- The Animal Protection Advocate's appointment, one of sixteen variants of
  which the generated cases complete only one: a missing pin.
- The alternate record reviewer's route for the price-control power, where the
  generator pins one alternate route per power: a missing pin.
- The guard on the record-order closure, whose removal only lets an entry on a
  cycle precede itself, which every rule reading that order already excludes.

A sample is not a census: a survivor means no selected pin noticed the change,
not that none could.

**The core replayed in a second engine.** `tools/second_engine.py` translates
the constitution into the input language of clingo 5.8.2, an established answer
set solver, and replays the cases for standing, the floor, delivery, custody
and the shield: 129 cases and 1,673 queries, each answer agreeing with the
verdict its pin records. The
[report](source/measurements/second-engine-report.md) lists them; refusals,
scoped acceptances, shell checks and contradiction scans are not compared. It
is a cross-check within this project, not an independent reproduction: a
misreading shared by both engines would pass both.

**The articles traced to the rules.** The plain-language constitution on the
website numbers its articles by Part, and each names the rule families, records
and tests that implement it:

- Who counts, and what they are owed: Articles 1–6, formal.
- The life the design leaves alone: Articles 7–14, formal.
- The public power that serves it: Articles 15–23, formal.
- What the design does to a person, and how it catches itself: Articles 24–29,
  formal.
- General provisions: Article 30, formal; Article 31, Interpretation, partly
  formal. Its burden rule, that a restrictive conclusion needs complete
  positive evidence and absence never extends a power, traces to the
  fail-closed rules. Its protective reading, that an open standard is read in
  favour of the floor and of liberty, is a principle for the people who apply
  the rules, and no rule states it.

[`articles_tests.rs`](../src/authoring/articles_tests.rs) requires every
rule-bearing family to belong to an article, every named family, record and
test to exist, and every cited article to resolve. It checks the map, not the
wording.

**The adversarial audit's open findings.** The
[audit](source/adversarial-audit.md) reads the source through fifteen
disciplinary lenses, from constitutional law to quantitative modelling. Its
open findings, each with the claim it withholds:

- No rule proves that an office received a request, acted or completed a
  remedy; operational evidence is the unbuilt route, so no such claim is made.
- No finding proves that a hearing occurred or compensation arrived; the same
  route is unbuilt.
- Nothing counts stock, forecasts supply or proves procurement; quantitative
  models and operational evidence are unbuilt.
- The accessibility checks inspect the built files only, so no claim of
  accessibility for readers follows.
- Foreign recognition, cooperation and readmission are external assumptions and
  are not claimed.
- The rule against counting the record's contents in the chapters is checked
  only where a count is written in digits; spelled-out numbers are left to
  prose review.

It is the project auditing its own repository, not an independent review.

**Declared defects.** A pin can expect a known defect and fail when the defect
stops reproducing. No such expectation is active. Some counterfactuals
reproduce a weakness on purpose: an [amendment
case](../tests/pins/amendments/person-proposal-separation/counterfactual.pins.nibli)
restores weaker rules under which a proposal naming no target becomes law, and
a [custody
case](../tests/pins/custody/condition-findings/counterfactual.pins.nibli) reads
every prisoner's missing company conclusion as isolation. Each passes by
reproducing the weakness, and the same facts on the actual source give neither
result, so a pass must be read with the source it ran on.

## Running the checks

The repository needs its Rust toolchain and the Nibli repository beside it, as
[the project instructions](../README.md) describe; `./bootstrap.sh` fetches the
engine at the revision in `engine.pin` and leaves an existing checkout alone,
so the verifier builds whichever engine is present. Then run:

```sh
./verify.sh
./verify.sh --list
./verify.sh --only book-1/01-the-child-with-nobody.pins.nibli
```

The first executes the whole inventory, the second lists it, and the third runs
one file as a partial result. Each pin file states how many checks it holds, so
a lost check fails, as does an ambiguous edit, a missing file, an unexpected
verdict or an unfinished required check.
The inventory is not a census of every possible policy or record. A matrix can
cover every combination of its named axes and still omit one that matters.

Verification executes the formal material. It does not certify prose,
citations, figures, navigation or the justice of a policy; those need their own
review, and a generated report is not evidence that its claims pass.

## Where the engine comes from

Nibli compiles its rule language into first-order logic and answers queries by
backward chaining over the supplied facts. Its negation follows the stratified
semantics of Apt, Blair and Walker: a program in which a relation depends on
its own absence has no canonical meaning and is refused.[^stratified] In
substance it is Datalog with stratified negation, with events, a fixed lexicon
of relation names and the `admits` and `derived_only` declarations added.

Writing law as a logic program is older than this book. Marek Sergot and
colleagues wrote much of the British Nationality Act 1981 as one in 1986 and
found its rule for abandoned infants to be a default later information could
withdraw, the question this book's child case meets from the other
side.[^sergot] Catala is a programming language for statutes built around
general cases and exceptions;[^catala] the OECD's *Cracking the Code* describes
publishing machine-consumable rules beside the human-readable ones, warning
that a technical fix can become the default without anyone asking whether it is
legitimate;[^rules-as-code] and Mireille Hildebrandt examines how automated
compliance can execute rules without and beyond the law.[^hildebrandt] What is
new here is the use, not the logic: a constitution's protections written as
limits on how facts about a person may flow into consequences, and tested
against a child nobody has come for and a person the state holds.

I maintain Nibli as well as the constitution and its tests, and much of this
work was done with AI assistance, so the agreement of engine, rules and tests
is a check within one project, not independent confirmation. The formal result
is conditional: these consequences follow from these premises under these rules
and this implementation. The truth of the premises, the feasibility of the
institutions and the justice of the choices need their own argument and
evidence, and *What It Would Take* owns the account of operation.

[^flow]: Dorothy E. Denning, [“A Lattice Model of Secure Information
    Flow”](https://doi.org/10.1145/360051.360056), *Communications of the ACM*
    19(5) (1976), 236–243; Andrew C. Myers and Barbara Liskov, [“A Decentralized
    Model for Information Flow Control”](https://doi.org/10.1145/268998.266669),
    SOSP '97, 129–142; Helen Nissenbaum, [“Privacy as Contextual
    Integrity”](https://digitalcommons.law.uw.edu/wlr/vol79/iss1/10),
    *Washington Law Review* 79 (2004), 119.

[^stratified]: Krzysztof R. Apt, Howard A. Blair and Adrian Walker, [“Towards
    a Theory of Declarative
    Knowledge”](https://doi.org/10.1016/B978-0-934613-40-8.50006-3), in
    *Foundations of Deductive Databases and Logic Programming* (1988), 89–148.

[^sergot]: M. J. Sergot, F. Sadri, R. A. Kowalski, F. Kriwaczek, P. Hammond and
    H. T. Cory, [“The British Nationality Act as a Logic
    Program”](https://doi.org/10.1145/5689.5920), *Communications of the ACM*
    29(5) (1986), 370–386, at 379 and 381–382.

[^catala]: Denis Merigoux, Nicolas Chataing and Jonathan Protzenko, [“Catala: A
    Programming Language for the Law”](https://doi.org/10.1145/3473582),
    *Proceedings of the ACM on Programming Languages* 5 (ICFP) (2021).

[^rules-as-code]: James Mohun and Alex Roberts, [*Cracking the Code: Rulemaking
    for Humans and Machines*](https://doi.org/10.1787/3afe6ba5-en), OECD
    Working Papers on Public Governance No. 42 (2020), pp. 2 and 13.

[^hildebrandt]: Mireille Hildebrandt, [*Law for Computer Scientists and Other
    Folk*](https://doi.org/10.1093/oso/9780198860877.001.0001) (Oxford
    University Press, 2020), chapter 10.
