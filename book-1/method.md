# The Method

This part is optional. It shows how to inspect a claim the book makes: find
the rule, identify the supplied facts, ask what follows, and try a case that
should produce a different answer. The machine checks those consequences.
Whether the English describes them accurately remains a separate review.

I use Nibli because it makes the relationships in this proposal explicit.
Facts name people, acts and records; rules state which combinations permit a
conclusion. The language can distinguish an entry from a conclusion nobody
may write directly, and can reject certain circular uses of absence. Those
properties help test this design. They do not make Nibli the only possible
language for a constitution or make every policy it accepts defensible.

The examples quote formal statements and the verdicts expected by executable
tests, with long lines wrapped for the page. The linked executable
files keep each statement on one line. The examples contain no
machine-written explanation or proof transcript. Each example names the
source or test a reader can inspect. The complete constitution stays in the
[repository](https://github.com/dhilipsiva/rights-nobody-has-to-earn), beside
the chapters and their companion files. A browser companion at
[the book's website](https://dhilipsiva.dev/rights-nobody-has-to-earn/)
executes a selected set of records against the same compiled constitution on
the reader's own device and displays the returned verdicts; it packages no
expected answers.

## From a supplied fact to a consequence

Nell's example begins with one supplied entry about Nell:

```nibli
born(Nell).
```

It is a fixture for the case, not a report of a birth outside the model. The
constitution contains these statements:

```nibli
all $child: born($child) & ~public($child) -> person($child).
all $x: person($x) -> owe(State, Eats, $x).
entitled(every person, event { eats() }).
```

A relation name precedes its arguments: `born(Nell)` records a birth for the
named subject. A name beginning with `$` is a variable; `all` quantifies it.
The `&` joins conditions and `->` points from conditions to their consequence.
The standing rule reads a birth and the absence of a public-body status,
then concludes personhood. The debt rule reads personhood and concludes that
the State owes that person food.

The entitlement statement has a different shape. `every person` specifies
who holds the right; `event { eats() }` names its content. Putting eating
inside that wrapper does not assert that anybody ate. An entitlement, a
debt and its fulfilment are different propositions.

The opening chapter's companion file asks:

```nibli
? person(Nell).
# => TRUE

? owe(State, Eats, Nell).
# => TRUE

? entitled(Nell, event { eats() }).
# => TRUE

? eats(Nell).
# => FALSE
```

The question mark introduces a query. The line beginning `# =>` states the
verdict the test requires; it does not cause that verdict. All these answers
come from the same case. Personhood supplies the debt and entitlement, while
the case supplies no matching evidence from which eating derives.

Nibli uses a closed world and a closed domain. In the relevant definite
queries, what cannot be derived from the supplied knowledge is `FALSE`;
quantifiers range over the entities represented in that knowledge base.
Neither assumption turns the file into a complete account of the world.
`eats(Nell)` returning `FALSE` does not establish that a child went hungry.
Similarly, `person(Ori)` returns `FALSE` in a case with no standing entry for
Ori. It does not establish that a real person lacks rights.

The constitution also derives standing from supplied contact, presence,
effective control and a report that nobody is acting for someone. It assigns an initiation duty without requiring Nell to
request help. Those are further rules; none lets the engine discover a
person whose encounter never reaches any input. A legal duty to find and
assist someone is different from a program having observed them.

This example is executed in
[chapter 1's pins](01-the-child-with-nobody.pins.nibli), using the
[birth fixture](../tests/pins/records/child_with_nobody/fixture.nibli).
The rules are in the [constitutional source](source/constitution.nibli).

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

Every repeated variable must refer to the same thing. The witness must be
authorised for this recipient, attest this item at food scope, and differ
from its source. It must also be none of the public bodies whose duty the
delivery discharges: the State, the common tier, or a region or locality
recorded as providing for the recipient. An attestation about another item,
person or kind of provision cannot fill the gap.

Chapter 5 supplies the evidence in stages:

```nibli
receives(Marisol, FoodForMarisol, Provender).
? eats(Marisol).
# => FALSE

authorized(Ledgerwitness, DeliveryWitness, Marisol).
observe(Ledgerwitness, FoodForMarisol, Marisol, FoodScope).
? eats(Marisol).
# => TRUE

? person(Marisol).
# => FALSE

? dwell(Marisol).
# => FALSE
```

The receipt alone is insufficient. Adding the matching attestation supplies
the missing premises and changes the food answer. Personhood is not a
premise of this route, so a prior roster entry is not required for the
recipient-side evidence to count. Food evidence supplies no shelter
conclusion. These are tests of the rule's reach, not just repeated positive
examples.

There is a second way to test the independence condition: change the rule.
The receipt suite supplies a provider as its own witness and gets `FALSE`.
An explicitly declared counterfactual removes only `~($w = $src)` from the
food rule and gets `TRUE` for provider self-certification. An independent
witness still succeeds, and the unchanged shelter rule still rejects a
self-certifying provider. Those controls help attribute the difference to
the particular condition removed.

Adding evidence and changing the constitution ask different questions. The
ordinary case asks what the present rule does with those facts. The altered
source asks what that condition prevents. The runner applies the declared
edit to an isolated copy; the experiment does not amend the constitution.
The [receipt suite](source/delivery-receipt.pins.nibli), its
[source counterfactual](source/counterfactual/no-delivery-independence.pins.nibli)
and [chapter 5's sequence](05-whether-it-arrived.pins.nibli) retain the
complete cases.

Even the positive result rests on supplied statements. The engine has not
authenticated the witness, watched a meal arrive, measured its adequacy or
established the capacity to provide tomorrow's meal. False evidence in the
required shape can make the query return `TRUE`. Inspection of the rule
and investigation of the event are separate tasks.

## A request, a duty and relief

Predicate names need definitions. `permits(Appeals, Ruk)` sounds like
permission to lodge an appeal, but in this constitution it means appellate
relief already granted. The distinction changes what a rule says.

The request route is separate. Its rule is:

```nibli
all $reader: all $request: all $requester:
  challenge($requester, $reader, $request) &
  authorized($reader, JChallengeReaderAuthority, $request) &
  ~($requester = $reader)
  -> obliged($reader, ReviewJusticeRequest, $request).
```

The requester needs no official role or previous permission. The reader
needs a mandate for the request and must be a different actor. This complete
fixture supplies those premises:

```nibli
challenge(AnyRequester, IndependentJusticeReader, BareJusticeRequest).
authorized(IndependentJusticeReader, JChallengeReaderAuthority,
  BareJusticeRequest).

? obliged(IndependentJusticeReader, ReviewJusticeRequest,
  BareJusticeRequest).
# => TRUE
```

What follows is a duty to review. No favourable decision, confinement or
release follows from that request. The broader justice rules separately
provide access, assistance, interim protection and an alternate duty when
nonresponse is established. A duty is not a record that a hearing occurred.
This request is executed in its
[own case](../tests/pins/justice/appeal/request-without-operator-or-previous-permission/expect.pins.nibli)
and in [chapter 21's companion file](21-a-way-to-be-heard.pins.nibli).

A qualified relief record has a separate case-specific effect:

```nibli
all $case: all $order: all $subject: responsible($order, CustodyRelief) & list($order, $subject, $case, CustodyRelief) -> clean($case, CustodyConfinementBar).
```

The first premise is itself a conclusion from the complete relief contract:
subject, case, incident, offence, recognised ground, evidence, procedure,
qualified independent actors and witnessed decision order. A bare clearing
entry or generic Appeals judgment cannot produce it.

Nia's record supplies a qualified final order for Case_Nia. Ruk's does not.
The chapter's pins obtain:

```nibli
? permits(Appeals, Nia).
# => TRUE
? prisoner(Nia).
# => FALSE

? prisoner(Ruk).
# => TRUE
? permits(Appeals, Ruk).
# => FALSE
```

The permission summaries display relief; the custody rule checks the bar against
its exact case. Absence of a bar does not alone confine Ruk. Qualified merits,
current authority, lawful placement and accessible challenge intake must also
be present. A pending filing suspends that authority without pretending to be
a final judgment. Confinement in an unrelated prosecution still needs the
independently qualified finding required to answer a disclosure shield.

Consider this proposed rule, which the test asks the engine to reject:

```nibli
:refuse reasoning /Unstratifiable/
all $subject: all $case: prisoner($subject, $case) -> clean($case, CustodyConfinementBar).
```

It gives every confinement an automatic final bar against itself. It does not
establish a right to request a hearing. Custody requires that bar to be absent,
while this proposed rule produces it from custody, creating a negative cycle.

Nibli's stratification check rejects that negative cycle. Relations can
depend on conclusions in their own layer through positive rules, but a
negative dependency must reach a lower layer. The proposed loop cannot meet
that ordering. This is a formal reason for refusing this encoding, not a
finding that universal access to appeal is impossible or undesirable.

The same chapter performs a different controlled experiment:

```nibli
:accept
all $x: prisoner($x) -> obliged(Appeals, $x).

? obliged(Appeals, Ruk).
# => TRUE
? prisoner(Ruk).
# => TRUE
? permits(Appeals, Ruk).
# => FALSE
? free(Ruk).
# => FALSE
```

This added test rule imposes a hearing duty without granting relief. It is
not a claim that Ruk already has this duty recorded in the unchanged case:
the companion file first checks that `obliged(Appeals, Ruk)` is `FALSE`.
The experiment establishes that this duty can be expressed without the
refused loop. It does not establish performance of the duty or add the rule
to the published constitution.

The accepted rule remains available for the subsequent queries in this
isolated test. Acceptance-only controls instead use `:accept-scoped`, which
restores the prior state after checking that the statement loads. The
distinction prevents one control from silently changing the meaning of later
tests. Separate case snapshots prevent it from changing another case.

## A refusal is a result about an input

Some inputs are refused before an ordinary query can be answered. The
constitution declares its permitted base vocabulary with `admits`, and
marks certain conclusions with `derived_only`. Chapter 3 executes both
boundaries:

```nibli
:refuse reasoning /not admitted vocabulary/
rich(Adam).

:refuse reasoning /declared derived-only/
prisoner(Zed).
```

The first statement uses a relation outside this record's admitted base
vocabulary. The second tries to write a conclusion that must be derived.
An admission must precede the input that needs it; a later declaration does
not retroactively admit a rejected statement.
The directive specifies the expected error class and a distinguishing part
of its message. If the statement loads, or fails for the wrong reason, the
test fails. The complete cases are in
[chapter 3's pins](03-what-counts-as-evidence.pins.nibli).

Neither refusal establishes that Adam is poor or Zed is innocent. They
establish which statements this input interface accepts. A permitted injury
or judgment entry can still be false, and a changed constitutional source
can widen the interface. The language checks the record supplied under its
rules; it does not appoint or authenticate the people supplying it.

### Rejecting a whole input

The ballot case tests what happens when an otherwise permissible assertion
shares an input with a forged conclusion:

```nibli
:accept
admits("decide").

:refuse reasoning /`decide` is declared derived-only/
person(Vote_Probe) & decide(Vote_Probe, Ballot).

? person(Vote_Probe).
# => FALSE
```

The exact-name admission remains loaded, but it cannot override the
derived-only restriction on `decide`. The refusal rolls back the whole mixed
assertion, including the otherwise admissible person entry. This checks the
boundary of that input operation; it establishes neither an actual person's
standing nor the authenticity of a different, accepted entry. The complete
sequence is in [Chapter 18's pins](18-the-vote-conviction-does-not-take.pins.nibli).

That file then keeps this added rule loaded while asking about Hano:

```nibli
:accept
all $x: person($x) & match($x, GeneralAdult) & ~prisoner($x) -> decide($x, Ballot).

? decide(Hano, Ballot).
# => TRUE
```

Hano's entitlement still follows from the broader ballot rule. The added
sufficient route does not replace it. This result is specific to those rules;
elsewhere an added fact can defeat a condition expressed through absence.

### Why the hostile floor rule is refused

The floor has a different protection, and it is designed rather than
incidental. In the current source, adding this rule is refused:

```nibli
:refuse reasoning /'prisoner' -> 'eats'/
all $x: person($x) & ~eats($x) -> prisoner($x).
```

The event body in an entitlement contributes a relation dependency to the
engine's graph without asserting that the person ate. Personhood also lies
downstream of custody: a prisoner remains a person. The proposed rule would
feed the absence of eating back into custody through that dependency and is
refused. The entitlement and actuality queries test both sides of this
distinction: the right derives while the meal does not.

The protection is specific to how the hostile rule is written. Entitlements are
written as events downstream of personhood, and a rule that feeds a floor
condition's absence back into custody closes a loop the engine refuses. A rule
confining someone for lacking a home record loads, because home status sits
outside that loop; a development check described below forbids it instead. Two
further results show the edge. A rule recording a loss of recognition for an
absence of company loads, although the design has no general recognition
status to withdraw. A rule making an absence of belief a ground for a
credibility finding is refused, because the credibility finding lies inside
the same custody graph. Neither result settles every differently expressed
attack, and an accepted alternative acquires no authority in the constitution;
each experiment is discarded after its check. [Chapter 4's
pins](04-what-you-are-owed.pins.nibli) hold both. A rule producing an operative
bar against a custody case is refused for the same reason: the bar would feed
back into the custody it is written to end.

The counterfactual removing the prisoner-to-person rule permits the
belief-absence attack. Another replaces
the event-shaped entitlement with a plain label; the tested entitlement
disappears and the hostile rule can load. These changes affect the rules'
meaning, not merely their presentation. Chapter 27 keeps both results and
their limits beside Zed's case.

The refusal prevents the displayed use of a missing floor actuality as a
ground of punishment. The [floor tests](source/rights-floor.pins.nibli)
distinguish refused uses from accepted controls across the relevant relations.
They do not establish that every hostile rule is unwritable. Removing an
entitlement from the source is a different attack. Rule review and the
amendment's source-effect tests therefore matter alongside the stratifier.

The stratifier does not judge intention. Adding a personhood condition to the
disclosure shield also creates a negative cycle: personhood can follow from
custody, while custody checks for the shield's absence. The proposed rule is
refused in [Chapter 24's pins](24-the-shield.pins.nibli), leaving the existing
shield in force. That failure of one encoding establishes no impossibility
of expressing the policy another way. The appeal example above illustrates
why distinguishing a duty from accomplished relief can change the result.

The stratifier's layers are not the book's reading order. The engine
computes dependencies among relations. The [contents](contents.json) place
chapters in an editorial sequence: standing and provision, ordinary life,
public power, then coercion and correction. A later chapter can explain a
premise used earlier without changing the order in which the logic depends
on it.

### Checking how a record is used

Contribution records illustrate a separate check. They are base inputs, so
the stratifier can accept a hostile rule that confines someone for lacking
one: there is no cycle back through a derived contribution. The executable
counterfactual demonstrates the harmful consequence. Acceptance therefore
cannot establish that the rule respects the record's purpose.

The development check
`a_purpose_limited_record_is_read_only_for_its_purpose` inspects the
constitution's statements. Contribution records may be read only to support
their supplement; pay promises may be read only to support their compensation.
Neither may be read under negation, produced by a rule, or asserted by the
constitution itself. The hostile contribution and compensation alternatives
are negative controls: the same inspection must detect their violations.

A companion check, `floor_actualities_have_no_downstream_consumer`, finds
any rule using a floor delivery conclusion as a premise, whether present or
absent. It rejects both kinds of added consumer. This prevents a receipt or
its absence from becoming a penalty or a breach finding through such a rule.
Both checks are in [the floor development tests](../src/authoring/floor_vector_tests.rs).
They inspect the current written rule forms. They are separate from executing
Nibli, and neither promises to recognise every semantically equivalent attack
written in a different form. They add no lawful power to an accepted experiment.

## Six ways a fact is kept from a consequence

A right is protected not only by stating it, but by limiting what the rest of
the law may do with facts about the person. Six kinds of constraint govern how
a fact in the record can become a consequence, and each is enforced in a
particular place:

| Constraint | What it stops | Where it is enforced |
|---|---|---|
| Closed inputs | An entry of a kind the constitution does not admit, such as a record that someone is rich or dangerous. | `admits` declarations, refused at input; [Chapter 3's pins](03-what-counts-as-evidence.pins.nibli). |
| Conclusions nobody may write | A written custody, ballot or answerability conclusion. | `derived_only` declarations, refused at input; Chapters 3 and 18's pins. |
| Purpose-bound reads | A contribution or pay record read for anything but its supplement or compensation. | `a_purpose_limited_record_is_read_only_for_its_purpose`. |
| Endpoints nothing reads | A duty, a delivery conclusion or a recorded loss used as a premise for something else. | `a_duty_is_not_an_action_because_nothing_reads_one` and `floor_actualities_have_no_downstream_consumer`. |
| No confinement from absence | A missing floor condition, or a missing home, family or work entry, used as a ground for confinement. | The refusal above, pinned in [the floor suite](source/rights-floor.pins.nibli) and Chapter 4; `no_confinement_reads_an_absent_home_family_or_work_entry`. |
| Scope binding | A finding about one subject, case, incident or record lent to another. | The case- and record-bound joins in each family and their generated wrong-case and borrowed-record cases; `scoped_authority_is_not_unary_permanent_answerability` and `global_findings_cannot_lend_effects_to_unqualified_records`. |

The named checks are development tests in the repository's
[authoring code](../src/authoring/); the rest are pins the verifier executes.

These are information-flow constraints. Dorothy Denning's lattice model asked
which classes of information may flow into which others, and Andrew Myers and
Barbara Liskov let the owners of data control where it passes; taint analysis
asks whether an untrusted input reaches a sensitive use.[^flow] Here the
information is a fact about a person and the sensitive use is a consequence for
them. Closed inputs and unwritable conclusions limit what may enter and what
may be claimed; purpose-bound reads and endpoints are taint rules; and the
purpose-bound reads apply the norm Helen Nissenbaum calls contextual integrity,
that a flow of information answers to the context it came from.

Each constraint is checked on the written form of the rules. A rule that
reaches the same consequence under a new relation name, or through a shape a
check does not inspect, can pass it. The endpoint and floor checks inspect rule
shape rather than vocabulary for that reason, and none of the checks
establishes that every harmful rule is unwritable. A refused rule proves a
property of one encoding; an accepted one proves only that it loads.

## Comparing and selecting an amendment

The amendment cases distinguish records claiming to concern the same text
from a program comparing that text. Nibli checks the supplied certification,
publication and effective-selection evidence. The separate
[amendment host](../src/amendment_host.rs) compares the exact source strings
and version identifiers in a trusted local review, then models selection in
memory. It interprets permission under the effective base, not a candidate's
self-authored authority.

One [development test](../src/amendment_host_tests.rs) appends a single newline
to a reviewed candidate. Certification fails because the submitted bytes no
longer match; the effective version and transition history remain unchanged.
Changing the base text or its identifier also fails. Exact comparison protects
the identity of the supplied text, even where a difference might leave its
meaning unchanged. It does not establish that the review was honest.

Other tests follow the transitions. Certification alone does not select a
version. Publication evidence must concern the same bytes. After one successor
is selected, another candidate authorised against the previous base is stale.
A query must use the selected source and a current lease: the host's record
of the version and selection generation it may query. Returning to identical
earlier text needs fresh authority and a new version occurrence; it does not
revive an old lease. A fresh session loads the selected text before the host
changes its effective version, so it cannot silently keep querying the old
rules as if they were the new ones.

These are single-process, in-memory checks over trusted input. They perform
no authentication, public publication, institutional adoption or deployment,
and write no replacement constitution. The host's development tests can be
run separately with `cargo test --release --bin amendment-assurance`; they
are not an additional routine verification gate. Chapter 22 states the legal
conditions and the same limits without requiring this implementation lesson.

## What a contradiction check establishes

A selected query can agree with its expected answer while the loaded model
contains a conflict elsewhere. The runner therefore also asks the engine
for a contradiction report wherever the suite declares that check.

Its development test supplies this small conflicting record:

```nibli
person(Ara).
~person(Ara).
```

Here the negative is itself a supplied assertion, rather than an inference
from an absent entry. Together with the positive statement it produces a
contradiction finding, and the runner fails the case. A companion development
check makes the knowledge base require recovery; an incomplete scan then
fails too. These examples are in the runner's
[development tests](../src/pin.rs), separate from the constitutional cases.

The engine checks represented integrity and disjunctive constraints,
explicit negative assertions, arity conflicts and equality conflicts. Its
report distinguishes violations from checks it could not finish. Both must
be empty for a declared scan to pass. This is not a decision procedure for
unrestricted first-order consistency, nor a search for every inconsistency
between the prose and the world. An injustice the model does not represent
as a conflict will not become one because a scan completed.

Query verdicts also need their scope kept intact:

| Outcome | What it says |
|---|---|
| `TRUE` | The query follows from the loaded facts and rules, assuming the implementation is correct. |
| `FALSE` | The query is not derivable there. It is not an observation that the event did not occur. |
| `UNKNOWN` | The search did not decide the query. This is different from a definite negative result. |
| `RESOURCE_EXCEEDED` | A resource bound stopped the search. The verifier treats that as a failure to complete, not a verdict about the claim. |

A pin can explicitly expect `UNKNOWN`. That expectation cannot make an
unfinished contradiction check count as clean. A refused assertion is
different again: the input failed to enter the knowledge base. The engine's
[guarantees document](https://github.com/dhilipsiva/nibli/blob/main/GUARANTEES.md)
states its inference contract and the scope of its own testing and proofs.

## What an act needs before it takes effect

Every record that takes effect names the roles that must act on it first. A
generated table lists them for each effect in the constitution:
[`book-1/source/procedural-load.md`](source/procedural-load.md), produced by
`./generate.sh procedural-load` from the constitution and a reviewed
classification. Each effect is classed as beneficial (it gives, preserves or
releases something for its subject and finds against nobody), adverse (it restricts, takes, confines or
finds against), power over others, oversight of a public act, or institutional
configuration. The roles are read from the rules; the class is a judgment the
classification records with its reason.

The table is a list, not a score. An effect classed beneficial takes effect on
its source's record alone: the source records every field, the independent
reviewer, challenge reader and alternate are named, and the reviewer owes
prompt review and can withdraw the effect. The table's *acting before effect*
column shows that single actor. The completed record of each beneficial kind
still needs its family's full procedure:

- An accommodation, care continuity, participation of the affected person,
  bodily and reproductive autonomy, access to one's own origin or other
  records, accessible communication, public information, inquiry and creative
  autonomy, a member's recorded opposition, or a recorded shortfall: a source,
  an evidence attester and an independent reviewer who record the same fields,
  a reader and an independent alternate.
- Access to a process, a hearing, an appeal, assistance, custodial safeguards,
  release review or continuity, survivor support or voluntary restoration in
  the justice family, and nationality recognition or cultural protection in
  the mobility family: the same three matching attesters, a challenge reader,
  an independent alternate and an audit reader.
- A finding that somebody is held, or that a disclosure is protected: a source
  and an independent reviewer who record the same fields, a challenge reader
  and an independent alternate.
- A protection claim, interim protection or a continuity measure in the
  ecological family: a source and an independent reviewer who record the same
  fields, a challenge reader, an independent alternate, an audit reader and an
  acting body.
- A delivery conclusion, an insurance supplement, a compensation or a
  certificate, which have no separate completed record: one authorised witness,
  adjudicator, attester or certifier.

Where more than one role writes the same fields onto one record, the
attestation is repeated. The table lists each such pattern with its family. The
most common are the matching source, evidence attester and independent reviewer
of the newer families and the matching source and independent reviewer of the
ecological family.

## Where the engine comes from

Nibli is written and maintained by the author of this book, who also wrote the
constitution and its tests. Its agreement with them is therefore a consistency
check within one project, not an independent confirmation; a reimplementation
elsewhere would be a different kind of evidence. Nibli compiles its rule
language into first-order logic and answers queries by backward chaining over
the supplied facts, under the closed-world and closed-domain assumptions
described above.

Its negation follows the stratified semantics of Apt, Blair and Walker: a rule
may use the absence of a relation only when that relation is settled at a lower
level, and a program in which a relation depends on its own absence has no
canonical meaning and is refused.[^stratified] In substance this is Datalog
with stratified negation, with events, a fixed lexicon of relation names and
declarations such as `admits` and `derived_only` added.

Writing law as a logic program is older than this book. Marek Sergot and
colleagues wrote much of the British Nationality Act 1981 as a logic program in
1986; they treated the Act's negations as failures to prove, justified under a
closed-world assumption, and found its rule for abandoned infants to be a
default that later information could withdraw, the question this book's child
case meets from the other side.[^sergot] Catala is a programming language for
statutes built around their general cases and exceptions,[^catala] and the
OECD's *Cracking the Code* describes governments publishing an official
machine-consumable version of their rules beside the human-readable one, with a
warning that a technical fix can become the default without asking whether it
is appropriate and legitimate.[^rules-as-code] Mireille Hildebrandt's textbook
on law for computer scientists examines how automated compliance can execute
rules without and beyond the law.[^hildebrandt]

What is new here is the use, not the logic: a constitution's protections
written as limits on how facts about a person may flow into consequences, and
tested against paired edge cases, a child nobody has come for and a person the
state holds.

## Running the checks

The repository needs its Rust toolchain and the Nibli repository checked
out beside it, as specified in [the project instructions](../README.md).
`./bootstrap.sh` can prepare the adjacent engine at the revision recorded in
`engine.pin`; it leaves an existing engine checkout unchanged. The verifier
builds the checkout actually present, so that input matters when reproducing
a result. From this book's repository, run:

```sh
./verify.sh
```

That command incrementally builds the release verifier and executes the
substantive case inventory. To inspect the inventory or run the first worked
example alone:

```sh
./verify.sh --list
./verify.sh --only book-1/01-the-child-with-nobody.pins.nibli
```

The focused command reports a partial result. It cannot establish that a
change preserves the rest of the book. The full command reports measured
runtime; a duration describes that run, not the strength of its conclusions.

Without a toolchain, the [published companion](https://dhilipsiva.dev/rights-nobody-has-to-earn/)
runs its own selected records in the browser. It loads the compiled
constitution once, builds a fresh knowledge base for each record, and shows
`TRUE`, `FALSE` or a refusal exactly as returned. Its records are a chosen
subset, not this inventory, and a result there carries the same limits as
one here.

The [suite inventory](../tests/pins/suites.json) identifies the source,
fixtures, pin files, explicit source edits and contradiction-check setting
for each case. Ordinary cases use the constitution; deliberate alternatives
state their changes. Some counterfactual and transitional exercises do not
request a contradiction scan and retain their own expected consequences.
A full pass means all required checks completed, not that every alternative
source was declared consistent.

Each pin file states how many checks it contains, so losing a check fails
instead of silently reducing the suite. Assertions within a sequence affect
later queries in that sequence; they do not leak into another case. An
ambiguous source edit, missing file, malformed pin, unexpected verdict or
incomplete required check fails the run. Explicitly trusted shell
preconditions retain their separate, declared permission.

The inventory is not a census of every possible policy or record. A matrix
can cover every combination of its named axes and still omit a consequential
axis. Positive examples need matching negative cases, controls and review
of downstream effects. The receipt counterfactual demonstrates one such
comparison; it supplies no universal guarantee about every future rule.

Verification executes the formal material. It does not certify prose,
citations, figures, navigation or the justice of a policy. Those require
their own appropriate review. Authoring commands such as
`./generate.sh spine` are separate from verification: producing a report is
not evidence that its claims pass. The chapter/pin pairs and current source
are the places to inspect a particular conclusion.

## Testing the tests

A passing suite shows that the pins hold. It does not show that the pins
would notice if a rule were wrong. Development tools ask that question from
outside the suite. None runs inside `./verify.sh`, and none produces a score.

One changes the rules. `tools/mutation.py` takes a seeded sample of
rules in each family and makes one small change to each: it drops a
condition, negates one, or swaps two roles inside one. The changed
constitution then runs the cases whose pins ask about that rule's conclusion,
including cases built on a temporal stage or a counterfactual that leave the
rule in place. A change that every selected pin still passes is a survivor.
The recorded run sampled three rules per family with seed 49 and ran each
change against at most twenty-four cases, half expecting the conclusion and
half expecting its absence. Its [survivors](source/measurements/mutation-survivors.md)
are listed with a disposition each.

A survivor is a question, not a verdict. Where a case can observe the change,
the suite carries the pin that does: an office that requests its own review is
owed none, a witness who observed nothing certifies nothing, and a condition
record bound twice is a collision. Where the change drops one attester's copy
of a field the other attesters still record, no generated case sees it,
because a generated omission case removes a field from every attester at once.
Four checks on the written rules reject that change instead: in ten families
every attester on a record records every field; in every family each field is
recorded by the same attesters in every rule; a single-actor route asks its
one actor for everything the full route asks of its source; and both timing
witnesses record every field. Each remaining survivor is listed as a missing
pin in a generated family, or as a change another rule makes unobservable,
with the reason. A survivor means that no selected pin noticed the change, not
that no pin could; a sample is not a census.

## A pass can include a reproduced defect

An expected answer is not necessarily a desirable outcome. One amendment
counterfactual tests two weaker rules: a declared protected target marks a
proposal false, and a proposed and approved name receives a law label unless
so marked. In that deliberately changed source, these entries have this result:

```nibli
suggest(Assembly, Amend_Sneak).
ratifies(Electorate, Amend_Sneak).

? become(Amend_Sneak, Law).
# => TRUE
```

The proposal declares no target, yet receives the law label. The passing
counterfactual reproduces the weakness of those rules. It supplies no
amendment authority in the actual constitution. The same entries on the
actual source produce no law label; amendment status instead needs the
candidate-specific certification, publication and effective-selection process.
The paired checks use the same
[supplied facts](../tests/pins/amendments/person-proposal-separation/fixture.nibli),
with [ordinary expectations](../tests/pins/amendments/person-proposal-separation/expect.pins.nibli)
and [counterfactual expectations](../tests/pins/amendments/person-proposal-separation/counterfactual.pins.nibli).

The cases also check that docketing a person's name supplies no personal
credibility loss on the actual source, while the weaker rules do produce
that loss. This is why the case's base and expected consequence matter as
much as its pass status.

The isolation cases make the same distinction between a rule and its tested
alternative. The [ordinary cases](../tests/pins/custody/condition-findings/expect.pins.nibli)
require positive, independently reviewed evidence of denied contact tied to a
person, holding, place and period. Missing company receipts do not qualify.
The [counterfactual](../tests/pins/custody/condition-findings/counterfactual.pins.nibli)
adds a weaker rule that treats every prisoner's missing company conclusion as
isolation; its expected markers reproduce that unsound inference.
[Chapter 30's pins](30-when-the-system-notices-it-broke.pins.nibli) separately
check that routine custody review remains owed without a breach finding.

The `:defect` markers are a list of declared defect expectations, not a
complete inventory of everything that could be wrong. A passing suite can
contain those expectations and can miss an untested failure. It must be read
with its defect report and its coverage limits.

The book and its checks include work produced with AI assistance. That is
not independent review. The constitution, examples and verifier are
maintained within the same project, and this book presents no independent
reimplementation or operational validation of the whole design. Public
source makes disagreement and reproduction possible; it does not establish
that either has happened.

The formal result is conditional: these consequences follow from these
premises under these rules and this implementation. Truth of the premises,
feasibility of the institutions and justice of the choices each need their
own argument and evidence. A clean contradiction report cannot supply them.

*The Rights Nobody Has to Earn* states the constitutional proposal and its
tests. *What It Would Take* owns the account of operation and transition.
A constitutional defect belongs to the first book; a claim that provision
or release actually occurred needs evidence beyond either book's rules.

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
