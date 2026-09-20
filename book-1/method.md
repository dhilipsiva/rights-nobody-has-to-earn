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
the chapters and their companion files.

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

The constitution also derives standing from supplied contact, presence and
effective control. It assigns an initiation duty without requiring Nell to
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
  ~($w = $src) -> eats($p).
```

Every repeated variable must refer to the same thing. The witness must be
authorised for this recipient, attest this item at food scope, and differ
from its source. An attestation about another item, person or kind of
provision cannot fill the gap.

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
a final judgment. An unrelated prosecution still needs the independently
qualified finding required to answer a disclosure shield.

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
The directive specifies the expected error class and a distinguishing part
of its message. If the statement loads, or fails for the wrong reason, the
test fails. The complete cases are in
[chapter 3's pins](03-what-counts-as-evidence.pins.nibli).

Neither refusal establishes that Adam is poor or Zed is innocent. They
establish which statements this input interface accepts. A permitted injury
or judgment entry can still be false, and a changed constitutional source
can widen the interface. The language checks the record supplied under its
rules; it does not appoint or authenticate the people supplying it.

The floor has a different protection. In the current source, adding this
rule is refused:

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

This prevents the displayed use of a missing floor actuality as a ground of
punishment. The [floor tests](source/rights-floor.pins.nibli) distinguish
refused uses from accepted controls across the relevant relations. They do
not establish that every hostile rule is unwritable. For example, a base
contribution record does not acquire the floor's structural protection
merely because using it to punish someone would be unjust. Removing an
entitlement from the source is a different attack. Rule review and the amendment's
source-effect tests therefore matter alongside the stratifier.

The layers used by that check are not the book's reading order. The engine
computes dependencies among relations. The [contents](contents.json) place
chapters in an editorial sequence: standing and provision, ordinary life,
public power, then coercion and correction. A later chapter can explain a
premise used earlier without changing the order in which the logic depends
on it.

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
much as its pass status. The separate amendment host uses trusted local
input and in-memory state. Its successful transitions establish no real
authentication, democratic approval, publication or deployment.

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
