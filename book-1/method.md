# The Method

This part is optional, and it is a different kind of reading. The note at the
front of this book promised that if you ever wanted to see the machinery, the
last part would show it. The fifth part closed by promising what, exactly: the
rules as they are actually written, the checks that run, the failures they
caught, and the things I wanted that the logic refused. This is that part.
Everything between the note at the front and the fifth part was prose a
machine had checked; the note, the fifth part and this one are me speaking —
but where Part V argued, this part shows.

Two ground rules, stated here so you can hold me to them. First: every English
sentence in this part is mine. You will see rules exactly as they are written
in the files and the machine's error messages exactly as it prints them; what
you will not see is the machine's English translation of any rule, or a proof
transcript — it renders no prose in this book. Where a quoted line is wider
than the page, it is broken to fit and nothing else about it is changed; the
files hold each statement whole. Second: everything shown here is runnable.
The book's repository is public — it holds the constitution, the chapters,
the pins and the checks — and one command re-checks the claims the chapters
stand on, building the engine from its own public repository before any pin
runs.
This part is the guide to that machinery, not a substitute for it. The
repository is the appendix.

Why does this part exist at all? Because a book that says "a machine checked
these promises" and then keeps the machine out of sight has asked for exactly
the kind of trust the rest of the book abolishes. The chapters are the
intuition. The files quoted here are the machinery. The checks described here
are the evidence. You are owed all three, in one object, and this is where the
other two live.

## The rules as written

The society you have been reading about is a file. Everything the record may
say is written in a small formal language read by a reasoning engine called
nibli, and the file — the constitution — is the single source every chapter
was gated on. Here are three of its lines, exactly as they appear:

```
entitled(every person, event { eats() }).
entitled(every person, event { dwell() }).
entitled(every person, event { believe() }).
```

That is what a floor right looks like. One line for each thing owed, and the
two words that carry the book are the plainest ones: *every person* — not
every citizen, every member, every adult, every contributor. The wrapper
around the owed thing is not decoration either: the file's own margin notes
record the measurement that removing it kills the protection the next
sections describe. The floor is not a preamble. It is load-bearing text in a
running system, and the current full list — like every figure in this part —
lives in the repository, not on this page, for a reason I will come to.

Those lines still need a person. Four new roots now supply that standing, exactly
as written:

```
all : born() & ~public() -> person().
all : at(, FirstContact) & ~public() -> person().
all : at(, RepublicJurisdiction) & ~public() -> person().
all : at(, EffectiveControl) & ~public() -> person().
```

Birth, first contact, physical presence and effective control do not read
citizenship, documents, age, capacity or an identity match. The supplied subject
may be a temporary encounter name. The rules derive personhood from it and
nothing else: not civil identity, family, guilt, liability, record ownership or
authority. The negative public check prevents a public body from entering through
a person root. These are consequences of supplied facts, not proof that an
encounter was observed or help was delivered.

Here is the final gate that makes someone a prisoner — the only rule in the
file whose conclusion is `prisoner`:

```
all $offender: all $victim: all $case: all $renewal:
  injure($offender, $victim) & judge(Court, $offender) &
  cite(Court, $case, $offender) &
  match($offender, ConvictionRecorded) &
  observe(Chronicle, $case, $offender, CaseScope) &
  observe(TemporalReview, $case, $offender, CaseScope) &
  observe(Chronicle, $case, Court, HolderScope) &
  observe(TemporalReview, $case, Court, HolderScope) &
  observe(Chronicle, $case, CourtJudgment, JudgmentScope) &
  observe(TemporalReview, $case, CourtJudgment, JudgmentScope) &
  observe(Chronicle, $case, $victim, InjuryVictimScope) &
  observe(TemporalReview, $case, $victim, InjuryVictimScope) &
  authorized($renewal, ActiveCustody, $case) &
  observe(Chronicle, $renewal, ActiveCustody, PowerScope) &
  observe(TemporalReview, $renewal, ActiveCustody, PowerScope) &
  observe(Chronicle, $renewal, $case, CaseBindingScope) &
  observe(TemporalReview, $renewal, $case, CaseBindingScope) &
  correct($renewal, ActivePower) &
  ~permits(Appeals, $offender) & ~broken(Court) &
  ~defend($offender) & ~free($offender)
  -> prisoner($offender).
```

It is longer than the earlier rule because no compact status is trusted by
itself. A recorded injury and a court judgment must be tied by two witnesses to
the same case, person, court, judgment basis, and injured person. The custody
lease must be tied by both witnesses to that case and power. Its
`ActivePower` conclusion is produced only after another rule rejoins the exact
lease, case, window, renewal, constitutional source, and canonical terminal
record without a competing binding or conflicted order path. Then this final
gate rejoins the case and lease evidence again. No standing appellate relief. A
court not itself marked broken. No derived defence. Not already free. Then, and
only then: prisoner. There is no second route, discretion, or “unless
circumstances warrant.” When chapter 13 called imprisonment the one thing this
society takes, this rule is the whole of what it was describing.

And here is the rule the voiding chapters orbit — the one that can declare a
person's word worthless:

```
all $a: all $b: all $audited: permits(Review, $a) &
permits(Tribunal, $b) &
authorized($a, IndependentReviewer, $audited) &
authorized($b, IndependentReviewer, $audited) &
judge($a, $audited) & capture($a, $audited) &
~deceive($a, $audited) & judge($b, $audited) &
capture($b, $audited) & ~deceive($b, $audited) &
~($a = $b) & ~broken($a) & ~broken($b) &
~match($a, CarriedVoid) & ~match($b, CarriedVoid)
-> false($audited).
```

It is long because it counts qualified, case-bound reviewers, not signatures:
two distinct people, each credentialed by a different body, each affirmatively
authorized as an independent reviewer for this audited person, each having
examined and documented the person, and neither lying, broken, or bearing an
effective carried void. Missing parent, marriage, or sibling entries prove
nothing about independence. The authorization facts are live in the shipped
record, and a source-removal counterfactual proves that restoring the old
absence-based route changes the result.

Two shorter blocks do more work than everything above. The first closes what
may be *written*:

```
admits("free").       admits("hears").      admits("home").
admits("injure").     admits("judge").      admits("married").
admits("authorized"). admits("carries").    admits("challenge").
admits("continue").   admits("date").       admits("limit").
admits("list").       admits("observe").    admits("passport").
admits("replace").
admits("at").         admits("born").
```

The record's base vocabulary is declared, name by name, and a ground fact in
any other vocabulary is refused at the moment of writing. The second closes
what may only be *concluded*:

```
derived_only("false").
derived_only("reward").
derived_only("prisoner").
derived_only("complete").
derived_only("correct").
derived_only("match").
derived_only("precede").
derived_only("time").
```

A name in this block may never be asserted at all — every appearance of
`prisoner` in the record must be the conclusion of a rule, which is chapter
1's most-guarded sentence made mechanical: "the conclusions that matter are
not writable. They are only derivable." These are two different guards on
two different doors — one closes the record's vocabulary, the other makes a
name conclusion-only — and the book has been careful never to describe them
as one. One name sits differently from everything around it: `person`. It
has producing rules *and* direct ground entries, and birth, encounter, custody,
release and carried status provide independent producing routes. It is therefore
a conclusion the machinery can reach that the record must nonetheless admit.
*What counts as evidence* and *what may be written* are different sets:
`person` is the clearest difference between them, and chapter 1 put personhood
on its evidence list for precisely that reason. A failed lookup does not reverse
a root already supplied, and correction or a death mark does not retract
personhood.

Try to break the first closure. Add one flattering fact the record does not
admit — `rich(Adam).` — and the engine answers at assert time:

```
[Reasoning Error] `rich` is not admitted vocabulary: this knowledge
base declared its base vocabulary closed with `admits("…")`, and
`rich` is not in it. Add `admits("rich")` ABOVE the first `rich`
assertion if this relation really belongs in the record — a visible,
reviewable edit, which is the point.
```

I did not write that error message, and its last clause is the engine
agreeing with chapter 1 in its own words: widening the record is a loud,
reviewable act, never a fact somebody quietly types.

One honest note on why you are seeing excerpts and not the whole file. It is
not length. A constitution like this is at its truest where you can run it,
not where I can frame it — printed in full here, it would be text; in the
repository, it is a system that answers back. The note at the front promised
that this part shows all of it; the way it shows all of it is by handing you
the place where all of it runs. The full file is dedicated to
the public domain, and the repository is
`github.com/dhilipsiva/rights-nobody-has-to-earn`. Clone it — and the
engine's repository beside it, which the check will name if it is missing —
then run the one command, `./verify.sh`. The pins pass, or this book has a
defect its author did not know about.

## The order of the chapters, and the tools that got it wrong

The chapters of this book are in a computed order. I chose the words in them;
I did not choose their sequence.

The mechanism: every relation in the constitution sits in a layer. At the
bottom are the record's plain words — the things the world may report, like an
injury, a judgment, a marriage, a lesson heard. Above them sit the conclusions
those words feed, and above those the conclusions *those* feed — personhood,
imprisonment, everything owed — up to the marks the audit leaves. Those marks
can now feed source-bound action, non-response, continuity, alternate, and
remedy conclusions. The chain ends in legal effects rather than an unread duty;
it still does not perform an act outside the reasoning system. A rule's
conclusion never sits below something it reads. The engine computes those
layers from the file, a script turns the layers into a chapter order, and the
book follows it: the chapter about a relation cannot come before the chapters
about what it rests on. The computed table — how many relations, how many
derived, how many layers, and the full list of which sits where — lives in a
planning file in the repository, inside a block marked *generated*, and a check
fails the build the moment that block disagrees with the constitution.

You have already felt this order, whether or not you noticed. The book opened
on the record itself — what may be written, and by whom — because everything
else rests on it. It ended, before Part V, on the audit and the legal
consequences that now sit above its findings. Chapter 14 ends there because the
next missing link is not another conclusion. It is action in the world.

Every chapter in
between sits where its subject sits in the layering: the voiding chapter
follows the chapter on the pens because the voiding rule reads the pens'
conclusions. The order of the book is the direction the rules read, and the
check that guards the spine fails the moment the two disagree.

That check exists because of a failure worth telling. The first version of
the spine was computed correctly — and then the constitution changed
underneath it, and nothing in the repository noticed. The numbers were not
wrong when written; they were wrong when read, which is worse, because every
reader after the change was being shown a careful, precise, stale answer.
The repair was not "be more careful". It was: stop letting a hand-maintained
copy exist at all.

The repository keeps one tool that got this wrong, on purpose, as a
standing exhibit: a small script that tried to compute the layers itself,
by hand-rolled parsing. It cannot see inside the `event { … }` wrapper you
met a page ago — so it never sees the floor at all. It looked sound, it
agreed with itself perfectly, and it disagreed with the engine on every
figure it printed. It is still in the repository, labelled wrong, kept as
the standing argument for taking every figure from the engine that actually
runs the rules. The spine generator carries the history of a second: its
own first form rebuilt the dependency graph out of text-matching rules — a
second implementation of the engine's layering, maintained by someone who
could not see it — and it disagreed with the engine in three places, two of
its numbers right only because two of its errors cancelled. The corrected
generator records that history in its own header, where the next person
tempted to reimplement will find it. Even the fixture-regeneration
instructions in the repository — the commands that rebuild the sabotaged
constitution copies the checks section describes — have been wrong twice,
including once in a way that matched nothing and silently wrote a
byte-identical copy: a fixture that tested nothing while looking freshly
made.

I am telling you this in the part that is supposed to earn your confidence
because it is the shape of the whole method. Every tool written to check the
machine became a thing the machine had to check. The response each time was
not resolve; it was structure — regenerate instead of transcribe, point
instead of copy, and make every checker prove it can fail before trusting a
word it says. That is also why this part quotes so few numbers: a figure
copied onto this page joins the class of things that rot, and the repository
is where the living figures are.

## What the logic refused

The conviction rule you read above refuses to convict anyone whose appeal
has been granted — the `~permits(Appeals, $offender)` conjunct. I wanted to
write the guarantee that ought to sit beside it. Every prisoner may appeal.
As a rule, one line:

```
all $x: prisoner($x) -> permits(Appeals, $x).
```

The engine's answer, exactly as printed:

```
[Stratification Error] Unstratifiable negation: strongly-connected
component containing 'prisoner' -> 'permits' (negative)
```

A universal right of appeal cannot be expressed in this constitution.

Follow the loop once, because it is the same loop the whole book stands on.
Conviction reads the *absence* of appellate permission. My rule derives that
permission *from* conviction. So the conclusion would feed the very absence
it was read from — reasoning passing through its own result — and the engine
refuses the shape. Not the policy: the shape. You have seen this refusal
twice already, from the reader's side, in the chapter on prisoners remaining
persons: it is what stopped the heresy law an attacker would write, and what
stopped the persons-only shield a careful designer would write. This is the
third case, and the difference is only whose rule died: the same wall, met
from inside, by the author. One mechanism, no special pleading, and none of
the three outcomes chosen by whoever was writing that day.

That chapter also named the trap, and it applies to me exactly as printed
there: a refusal leaves the *loose* rule in force. When the engine refused my
guarantee, prisoners did not get a right of appeal — relief stayed what it
was, an act somebody performs, granted case by case, with the cost the book
prices where it weighs that default: a person whose case nobody takes up
stays held. Being told no is not being kept safe. The refusal told me the
guarantee I wanted cannot exist in this shape, and that what exists instead
would keep running whether or not I made my peace with it.

A second refusal shows something less comfortable: a wall can block a bad
intention without proving a good policy. An earlier version would have let a
convicted person earn their sentence shorter, and the obvious way to write that
was release earned from inside:

```
all $x: prisoner($x) & reward($x) -> free($x).
```

```
[Stratification Error] Unstratifiable negation: strongly-connected
component containing 'prisoner' -> 'free' (negative)
```

Conviction reads *not already free*; a release derived from conviction loops
straight back through it. That tells a fact about the current formal shape. It
does not decide whether earned-time credit is just. The ratified temporal contract
answers that question separately, and the enacted path contains no route from
character, contribution, or recognition to a shorter sentence. Release remains an
entry someone writes,
never an output the machinery computes, and it carries the concession chapter 1
already made: the entry does not say whose decision it records. The temporal rule
instead constrains the authority holding the exact person in the exact case. If
the current reviewed authority is absent, custody does not derive; `free` is not
manufactured in its place.

What it felt like, since the opening note promised I would say: not like
being corrected. Like leaning on a wall I had built myself and finding it did
not care that I was its builder. The machine holds no opinions about appeals
or mercy. It found a shape twice, and both times it returned a fact my draft had
to answer. That made the later decision more honest — not because it is wise,
but because it cannot be talked out of anything, including by me.

## The checks, and the check on the checks

One command runs everything: `./verify.sh`, at the root of the repository.
It rebuilds the engine from source and prints the engine commit it built,
then checks the generated spine and audits: the assertion surface, the
record-integrity assurance case, the bounded record red-team, and the
JSON-backed amendment-semantics, placement-exhaustiveness, and staged temporal
assurance audits. It also
runs the sweeps that keep the chapters' prose inside the rules, the guards on
what nothing may read, the full pin suite, the executable record snapshots,
and the source counterfactuals. It stops at the first failure, naming the claim
that stopped being true. The rebuild-and-print exists because this repository
once spent three days running a stale binary to a green result, and a check
that does not pin down *what is doing the checking* is a rumor with a progress
bar.

The script's own history is the honest part, so here it is in the register
this part owes you — the claim, then where it broke.

The suite reports its runtime; for months it claimed about fifteen minutes,
and the real figure was closer to fifty. Durations printed there are now
measured and dated, and trusted only until re-measured.

The pin files can carry control statements — rules loaded deliberately so a
query can show what they change. The first control mechanism left its
statement in the knowledge base after the check, so every pin below one ran
against a quietly widened base. That was not theoretical: measured while
repairing it, a conviction pin passed against a copy of the constitution
with the conviction rule deleted outright — the greenest possible light,
certifying nothing. Controls are now scoped: the engine itself, not
discipline, puts the base back.

The sweep that keeps engine jargon out of the reading chapters failed, as
first written, to catch the exact leak it was written for. It earned its
place only after being made to fail against a sabotaged copy — and that
became the house rule for every check in the script: sabotage first, trust
after. A check that has never been watched failing is not yet a check.

The pin suite itself is the book's claims made executable. Each chapter has
a companion file of pins — a query against the constitution and the verdict
it must return:

```
? prisoner(Adam).
# => TRUE
```

The whole set runs on every check, each file declares how many pins it
carries so a file cannot be quietly hollowed out, and a changed verdict
stops the build and names the chapter whose sentence just became false. A
separate universal-standing group runs twenty-nine more pins: birth, contact,
presence and control roots; missing records; disputed age; false alias links;
mistaken death; disability and refused support; retained correction provenance;
and the refusal to treat a public body as the contact subject.

One of the sweeps deserves its own paragraph, because it polices me. The
chapters you read are forbidden — mechanically, by the same script — from
containing counted claims: no fixed total for the floor, no hand-maintained
headcount of who is sheltered, not because numbers are vulgar but because every
counted claim in this book that was ever checked turned out to be wrong or soon
became stale. The floor itself has changed more than once.

The rule the sweep enforces is: state the rule that produces the result, never
the count. “Shelter derives through current custody routes or through a matching
recipient-side record independently attested by an authorised witness who is not
the source” survives a cast change. A headcount of sheltered people does not.

This part sits outside the sweeps by its filename, deliberately — the error
messages quoted here would trip the jargon sweep, and this paragraph's examples
would trip the counting one — so it holds itself to the doctrine the hard way,
which is why the figures here point at the repository instead of standing on
this page.

Some pins are stranger, and they are this method's most honest invention.
The book argues about flaws the design still has — chapters stand on them.
Those flaws are pinned too, marked as defects, with the marker recording
what would flip them. From chapter 12's file, exactly as written:

```
:defect "a totality guard: a targetless proposal receives no law label"
? become(Amend_Sneak, Law).
# => TRUE
```

A targetless proposal still receives the law label here — a real defect in the
label guard, not proof that any change took effect — and chapter 12's argument
depends on it being real. If a future edit quietly fixed it, the suite would not
celebrate; it stops the build and says, in its own words, that this is a repair,
not a regression — find the chapter that calls this a flaw and rewrite it, then
drop the marker in the same commit.
The markers in the pin files are the complete list of declared flaws — a
count here would rot, the markers are the list — and each one is a tripwire
in both directions: the flaw cannot silently persist, and it cannot silently
vanish while the prose goes on confessing it.

That pin tests the amendment's label verdict. A separate audit tests selected
source effects. Its reviewed source is
`new-book-plans/amendment-semantics-audit.json`; the script
`new-book-plans/10-amendment-semantics.py` validates that contract, applies
each exact candidate mutation to a temporary copy of the constitution, runs
the paired queries in a fresh engine, and generates
`new-book-plans/amendment-semantics-audit.md`. These are bounded source
counterfactuals: the candidate edits are applied manually by the audit, not
by the amendment machinery.

The cases walk both directions. A targetless proposal sits beside a direct
floor deletion. Another declares a harmless target beside the same deletion;
a third declares the floor honestly and receives a blocked label, yet the
entitlement is still absent in the audit's manually constructed source. A
one-line floor-shape edit hides its effect more subtly: the food promise
disappears while the anti-sanction refusal remains. The reverse false-target
poisoning attack adds a registered target label to an ordinary proposal and
blocks its law label without any modelled mint or floor effect. A separate case adds
`Art_Evidence` to the register and directly adds `admits("rich")`; `rich(Adam).`
becomes writable because no proposal binds the two. It is kept outside the
amendment cases because a direct vocabulary edit bypasses Article 9.

The audit proves the engine-visible consequences of those exact supplied
sources, and no more. It does not prove that `become` enacted an edit, that
`false` prevented one, that a declared target is true or complete, who wrote
or authorised the source, whether it was lawfully deployed, or whether one
constitutional version safely became another. No rule reads `become`; the
label verdict and the candidate mutation meet only because the test author
placed them beside each other. The audit exposes that boundary. It does not
close it or add a new withholding gate.

Placement has its own audit because a walk through named people cannot prove
that every case in the current routing table was visited. Its reviewed source is
`new-book-plans/placement-exhaustiveness-audit.json`; the script
`new-book-plans/11-placement-exhaustiveness.py` generates the full product of
subject state, severity, legacy family-entry presence, and typed
PlacementHome availability. Family is an intentionally inert regression axis:
its two states must produce identical outcomes when the other inputs match.
Severity selects HighSec. A non-severe confined person is Homestay-eligible,
and the typed home record determines whether Homestay is an actual destination
or housing derives without a fabricated location. The audit asks separately
about eligibility, housing, and every known destination. It mirrors the same
cases twice without confinement: once with affirmative freedom and once with
personhood alone. In both, the shelter debt and entitlement remain while
housing and destinations do not appear.

The source manifest is part of that contract. A new producer or destination
cannot hide outside a fixed query list. Temporary mutations add an overlapping
route, delete the historical housing repair, remove a required destination,
reverse a route, make family consequential, and paint housing onto every
registered person. Each harmful copy must be caught by ordinary expectations
before the audit is trusted. Each alarm-silence probe supplies and confirms a
placement report first, so a missing report cannot make the result green by
accident. The result is exhaustive for the current declared subject states,
axes, and source. It is not a runtime exclusivity rule, evidence that a reported
placement happened, a placement appeal or remedy, or delivery of housing to a
free person.

A second registry guards when new rule families may be written. Each authored
formal statement has one stable engine fingerprint and belongs to exactly one
coverage family. A planned family may contain no rules, pins or claim-bearing
prose. A coverage-ready family has complete source-bound effect contracts but no
formal rules. Formalized means the exact fingerprints, pins, executable negative
tests and counterfactuals all resolve. Prose-landed additionally requires every
declared chapter and Part V reference. An unassigned new statement, a changed
fingerprint, or prose arriving before its family is ready fails verification.
That is a pre-drafting gate over constitutional coverage, not an operational
claim and not a route for Book 2 delivery machinery.

The liberty-and-ecology family is the first new family carried through that
gate after universal standing. Its coverage record contains twenty-five
non-power effects. It preserves the material-floor inventory exactly, then
separates person-held liberties, the environmental right of people living now,
and the Class Nine commons ceiling. The formal source registers one declaration
and thirty-eight rules under the family, and its executable suite checks forty-two
expectations. Two source-removal counterfactuals separately delete the
environmental-right root and the climate-axis ceiling, so one side cannot make
the other's absence look green.

The new `prevents` relation is conclusion-only. It records a constitutional
barrier; it does not make the parser reject every incompatible sentence, measure
an ecological condition, choose a ceiling number, operate an institution, or
deliver a remedy. The floor inventory is checked independently so ecology cannot
become a ninth floor item or erase one of the eight existing items. The Class
Nine axes remain separate rather than becoming a score. The family neither
changes the T3 supplied-record path nor borrows T3's executable evidence for a
different power. Its green result is therefore a formal coverage result over
supplied records, not proof of environmental liveness, liberty enforcement, or
restoration.

The substantive-equality family follows the same gate without compressing its
content into one omnibus equality fact. Fifty-four source-bound non-power
effects each have one registered `prevents` rule and one exact statement
fingerprint. Its executable suite checks fifty-eight expectations. Three
single-deletion counterfactuals separately remove the direct-discrimination
barrier, the diagnostic-data wall, and the end rule for a positive measure;
the surrounding equality controls remain in place so each loss is attributable
to the deleted statement rather than to a collapsed family.

The effects distinguish ten forms of subordination, public and specified
private reach, direct and indirect proof, protected association boundaries,
diagnostic data, bounded positive measures, historical repair, initiation,
separated functions, individual and systemic remedies, and the federal
corridor. They preserve the criminal burden and refuse to turn absence from a
record into proof of withholding. The rules prove only that the declared
constitutional barriers follow from supplied records. They do not establish a
population disparity, an operating data system, an authorised measure, an
accessible route, institutional action, or completed repair. The family adds no
power allocation and neither changes nor borrows the T3 temporal path.

The bodily-autonomy, health, care, family, and life-course family is the
next completed non-power family. Its registry holds one hundred and two
source-bound effects and one hundred and twenty-four exact formal-statement
fingerprints. The executable suite checks one hundred and eighteen pins. Four
source-removal counterfactuals isolate automatic adulthood, the family-status
confinement wall, affirmative reviewer independence, and pregnancy authority.

The migration retired mature as a legal-status proxy, made GeneralAdulthood
evidence typed and non-constitutive, made legacy family facts inert in
placement, split civil home from PlacementHome availability, and replaced
missing kinship with affirmative case-bound reviewer authorization. The wider
effects cover children's independent rights, modular family recognition,
parentage and origin, public-first care, domestic protection and separation,
supported agency, reproduction, ageing, treatment, end of life, death, and
succession. Each is a constitutional barrier over supplied premises. The family
adds no power allocation, preserves all two hundred and ten power cards and the
separate T3 template, and forbids borrowing T3's custody record for a
life-course power. Its green suite establishes no age value, service route,
institution, operation, delivery, liveness, or completed remedy.

The state-form and political-membership family completes the existing
constitutional architecture rather than enlarging its census. No reviewed row
was added. The family formalises the existing fifty-one state-form power cards,
uses the existing thirty-eight body cards, and repairs holders and functions
inside the existing two hundred and ten power allocations. It adds no power,
claim, status, body, census grain, or allocation.

Its formal seam has three positive stages. First, separately authorised source,
record-review, temporal, and temporal-review roles establish
`complete(record, StateFormCurrent, temporal_record)` for one exact source
family, version, epoch, power, jurisdiction, legal scope, reconciliation, and
time record. Second, card-specific raw fields and the separated result roles
establish `complete(result, FSPOW_nnn, record)`. Only then may
`authority(holder, FSPOW_nnn, record)` follow. That three-place authority is the
sole state-form legal effect. It is not the older one-place answerability
relation, and this family derives no state-form `match` conclusion.

Dynamic elections, confidence decisions, appointments, initiatives, recalls,
amendments, successions, and secession consume externally supplied
administration, completeness, result, review, and execution records. Static
competence, lawmaking, adjudication, continuity, and local-power cards consume
their own positive source-supplied fields.

No affirmative result follows when a required positive premise is absent. The
machinery does not authenticate an attestation, discover an omitted conflict,
or determine from an outside clock that a record is stale. A false attestation
can support the formal result when it is supplied in the required shape. A zero
roster, tie, conflict, or other failure has only the consequence positively
specified for that exact route; absence alone creates nothing. `FALSE`
preserves no law, office, mandate, or claimant protection. Any continuation
requires its own positive current premise and end condition.

The dedicated suite
`new-book-plans/state-form.pins.nibli` gives every card positive, negative, and
counterfactual anchors. The single-mutation copy
`new-book-plans/counterfactual/no-state-form-independent-current-review.nibli`
removes only `~($source = $temporal_review)` from the shared currentness rule.
In that copy the source writer can also serve as temporal reviewer, and the
watched fused-role case for every card derives authority that the real source
withholds; properly separated controls remain green. Its paired pin file and
the exact-diff check bind the copy to that one mutation.

The suite proves bounded legal effects over supplied records. It does not
authenticate a source or certificate, calculate a changing tally, resolve
competing submissions, choose a term, seat count, threshold, tolerance, or
fallback number, operate an institution, advance time, publish a result, deploy
a source, transfer an office in the world, or deliver the floor. It does not
reuse custody's T3 source, lease, case, or renewal. Every affected power, body,
and claim therefore remains `Specified` and `ratified-unimplemented`: formal law
is not staffing, independence in fact, operation, delivery, or institutional
action.

The non-reciprocal-obligations family is a completed cross-cutting family. It
adds no power, body, claim, allocation, or status. It preserves the existing
floor entitlements and debts, then keeps public duties, delegated public
functions, express private duties, civic-duty limits, duties to a person, duties
to a source-defined common, role duties, voluntary commitments, conflict,
excuse, continuity, and remedy in separately tested effects.

Public responsibility is separated into respect, protect, fulfil, continue, and
remedy. Each conclusion rejoins the exact source, version, bearer, jurisdiction,
function, scope, evidence authority, independent reader, failure polarity, and
temporal record that permits it. A contractor inherits only the duties of the
public function it performs; it derives neither public status nor authority. A
private duty follows only from an express current source.

Conflict and excuse are positive-record routes. Missing or conflicting evidence
creates no duty, priority, discharge, alternate, or remedy. An excuse may move
performance but cannot erase the protected claim. The non-reciprocity checks
separately prove that work, payment, identity, compliance, contribution,
approval, a role duty, a civic duty, or a voluntary commitment is not a premise
of personhood, the floor, or the public debt.

The existing Review and Appeals obligations now feed typed action routes.
Their dedicated suite checks the named reader, wrong-recipient withholding,
non-response, alternate continuity, individual remedy, systemic correction,
affected-case re-examination, and recurrence-check duties, alongside
source-removal counterfactuals. A green result proves those legal effects over
supplied records. It does not prove notice, staffing, response, enforcement,
delivery, completed repair, recurrence monitoring, or institutional life.

The delivery-and-receipt family adds one recipient-side evidence relation,
`receives`, and five direct heads. Food, non-carceral shelter, care, material
security, and company derive only when the same rule joins a recipient, item,
source, authorised witness for that person, matching scope, and the witness's
separation from the source. The legacy `teaches` plus `hears` route to `learn`
remains unchanged.

The family suite keeps the shipped record dormant, then exercises the routes
with probe evidence. Correctly matched independent evidence derives the selected
actuality. Missing attestation, missing receipt, wrong scope, wrong person,
unauthorised witness, and provider self-certification do not. A predeclared
alternate succeeds through the same authorised role. A recipient who is absent
from the person roster can still derive the actuality, while remaining absent
from that roster.

The source deliberately adds no delivery head for `believe` or `expresses`.
Belief is protected from certification. The liberty of expression remains a
person-held limit, while the older confinement route can still put recorded
speech into the supplied record. Care deriving `healthy` retains the book's
fixed meaning of care received and does not establish that the person is well.

The independence guard needs a source mutation because derivation is monotone.
`no-delivery-independence` removes only the food rule's witness-source
disequality. In that copy a kitchen can provide a meal, attest its own delivery,
and derive `eats`; the independent control remains true and the untouched shelter
rule still refuses provider self-certification. The fixture proves what the
guard prevents without pretending that a probe can test a restriction by adding
facts.

A missing receipt derives no `Undelivered` finding. It supplies no arrival and
leaves entitlement untouched. Outreach, challenge, continuity, remedy,
corrective control, affected-case re-examination, and recurrence checking remain
separate obligation effects with their own positive records. Recurrence checking
is bounded to its declared supplied horizon and proves nothing about all future
time.

These checks establish a formal interface over supplied records. They do not
authenticate a writer, prove an offer or accessibility, operate a service, make
anything arrive, complete a remedy, implement a correction, monitor recurrence
in the world, or establish future non-recurrence.

The economic, labour, property, and fiscal family follows the same discipline
without adding one omnibus conclusion called an economy. Work freedom and
labour protection, licensing, possession and acquisition, knowledge
exclusivity, contract and consumer protection, enterprise capacity, private
power, protected voluntary life, taxation, appropriation, spending, borrowing,
money, credit, insolvency, scarcity, and remedy remain separately testable legal
effects.

A restrictive or managerial power requires its own current source, lawful
holder, jurisdiction, scope, evidence, independent review, bounded result, and
end. Occupational licences, knowledge exclusivity, private-power findings,
monetary office, temporary management, and the carry of a benefit, title, or
liability do not borrow custody's time record. Missing or conflicting current
evidence supplies no restrictive authority, while the independent floor and
essential-service continuity remain.

Review silence supplies no approval or continuation. Record review, temporal
review, independent review, audit, and final review each have a distinct
predeclared alternate for certified unavailability. Each alternate is separate
from the primary actors and the other alternates, and its branch produces only
the matching duty. It cannot complete the power, enlarge the result, or keep
expired authority alive.

The negative checks keep recognition, voiding, loss, raw work, contribution,
wealth, conviction, family status, and personal scores out of wages, property,
credit, insurance, political weight, the floor, and scarcity priority. Raw
financial and equality records remain outside the consequential person record;
only a narrow, authenticated, purpose-bound result can enter the rule that needs
it.

A green result establishes only those bounded legal effects over the supplied
records. It does not value property, calculate a tax, clear a market, measure
dominance, forecast prices or inflation, prove scarcity, operate a payment,
choose a budget, staff an institution, preserve a service in the world, or make
any remedy happen.

The last class of check exists because of a limit in the logic itself. The
engine only ever adds conclusions as facts are added — it never retracts on
its own — so no probe stacked *on top* of the constitution can test what the
constitution refuses or restricts. Every claim of the form "remove this line
and the world loses that" would stay an argument forever. So the repository
keeps sabotaged copies of the constitution — each differing from the real
one in exactly one deliberate way: a line deleted, a line changed, a line
added — and the suite verifies each copy differs in exactly the promised
shape, then runs pins against the sabotaged world to show the loss actually
happens. When this book says the multi-sig's dormant guards decide nothing
today, or that one added credential route would let a carried void count, a
fixture executed that claim. This was the first source-mutation suite in the
method; the amendment-semantics audit now executes its own bounded candidate
deletions, replacements and additions for the narrower question of whether a
declared target matches an effect.

The record red-team adds a second and narrower class. Its ephemeral copies
delete named ground entries — typed adulthood evidence, a directly supplied
person fact, raw void marks, and the two parts of forgiveness — and execute the
remaining single-record consequences. It also proves that a raw void mark alone
is now inert. The adulthood cases demonstrate an evidence-bound ballot
conclusion; they neither constitute adulthood nor observe an outside clock.
Transition, carry, order, and renewal belong to the staged temporal assurance
case described next. The red-team tests prove what follows from supplied
records. They do not show that a live deletion occurred, identify who withheld
or removed anything, or recover the missing entry.

The temporal assurance case adds the transitions that one-file chapter pins
cannot supply. Its reviewed JSON binds the exact constitutional source and
input contracts; its script constructs cumulative copies with record
reconciliation, then order, then case-bound custody. Every case runs in a fresh
process because the engine only adds conclusions: appending a later record to an
old process would not retract an earlier conclusion and therefore could not
honestly test which record is current.

At the first stage, two witnesses bind exact manifest members and record
replacement. A separately witnessed passport selects one constitutional
lineage; only its collision-free terminal successor gives carried voids,
clearance, or public power current effect. A carried personhood route remains available on an accepted transition only when
both predecessor-standing witnesses are supplied; withholding either can still
erase that T1 route and its omission marker. It cannot erase personhood supplied
or derived independently through birth, contact, presence, control, custody or
release. The universal-standing suite tests those roots separately; T1 neither
creates nor authenticates them. Audit reports retain the same supplied-record
boundary. At the second stage, independently witnessed
event and record entries form separate typed paths. The paths close
transitively, so a longer cycle is caught as well as a direct reversal, and the
resulting conflict spreads through the connected path without leaking into the
other path type. A conflicted path cannot support a review window.

At the last stage, custody rejoins the raw witnessed fields behind every compact
tag: case subject, court holder, court judgment, injured person, lease, window,
renewal, source, and canonical terminal record. Competing witnessed
source/window/case-subject/lease bindings fail closed. The executable cases attack
omitted and forged carry, disjoint or replayed lineages, direct and transitive
order conflict, tuple aliasing, source mismatch, case reuse, withheld standing
witnesses, and challenge intake. A supplied challenge suspends its exact lease;
the paired challenge-absent process can remain active. The
protected-record alarm is intentionally conservative: the constitution has no
way to represent lawful disposal of such a record, so every missing required
carry is reported as a disappearance. That proves a mismatch, not unlawful
deletion.

This is bounded safety evidence for supplied records. The separate root rules
and pins establish universal standing only when one of their external birth or
encounter facts is supplied. Neither suite observes that fact, preserves a
challenge that intake withholds, operates correction or identity reconciliation,
proves a manifest complete, authenticates outside witnesses, detects deletion
before or inside the first attested record, advances a clock, makes a later record
arrive, serves a person, releases anyone physically, or makes Appeals perform its
duty.

There is a second thing to say about the fixtures, and it is a weakness
rather than a strength, so it belongs here rather than in a footnote.
Source counterfactuals exist for a few rule and fact changes — a seat, a
court's answerability. The amendment audit executes a bounded set of candidate
source edits, and the record red-team names a few more ground-entry deletions.
The placement audit is one bounded exception to accidental coverage: across
its declared subject states, severity, inert family axis, and typed
PlacementHome axis it generates the whole outcome table and rejects missing,
conflicting, reversed, family-sensitive, and painted-delivery routes. None
of those is a complete census of semantic effects. Deleting the evidence behind a named
person's severity, or the line that makes the duty-bearer answerable, still
depends on whatever other check happens to read it. Those changes may turn a
chapter's pins red, which sounds like a safety net and is not, because they do
it only where a chapter happened to argue about the changed fact and pinned it
for reasons of its own. Coverage outside the declared fixtures and generated
matrix is an accident of what the chapters chose to say, never a guarantee. A
change nobody happened to pin is a change nothing sees. I would rather write
that down than let the green tick imply otherwise.

Now the paragraph this part exists to hold, beside the machinery it
qualifies. Everything above runs, and a stranger can run it: clone the
repository, one command, the pins pass. Here is what that does not mean.
Every probe, fixture and refusal in this book's repository was written by me
and by AI sessions working with me, against a cast of a few dozen
record-people. No one has independently reimplemented the checker. No
outside red team has attacked this constitution. And the engine that blesses
the book shares the book's author. Two things narrow that, and neither
closes it. In this repository, the sabotage fixtures, amendment-semantics
audit, placement audit, record red-team and temporal assurance case execute
selected source and record mutations rather than merely describing them. Upstream, in the engine's own
repository, the engine — not this constitution — is checked against work
that is not its own: external solvers are run as referees over the fragments
of the logic they can hear, though each starts from the engine's
already-compiled output, so the language front-end sits outside them; the
criterion that refuses unstratifiable rules is implemented a second time
from its mathematical statement rather than from the engine's code, and the
two must agree on every random program thrown at them; a proof assistant's
kernel checks model-level proofs of the core algorithms; mutation and fuzz
testing beat on the rest. The engine's guarantees document holds that
inventory in the register I have tried to keep here — state the guarantee
flatly, then name where it stops. So, flatly, where this one stops: every
verification artifact in the engine's repository is authored, run and
interpreted inside that repository. No one outside this project has verified
the engine. I will not promise you a red team I do not have. The suite is
public and it is runnable, and that is not a defense — it is a standing
invitation to become the outside reader this book is still missing.

## The last page

The book ends here, so let me say what it was. Chapters describing a society
from its rules outward, checked by a machine that cannot be flattered; one
part of argument, mine, where the machine has no say; and this part, which
put the machine itself on the table. The confessions along
the way were the product working. The refusals were the design holding. What
I could not defend, I said so, out loud, in the plainest words I could find.

One thing this book has deliberately not told you, and it is the largest
thing. It has described what a society is when certain things cannot be
taken from anyone — and it has said almost nothing about how to build one.
What it would cost. Who would go first. How anything arrives at scale, from
bread to housing to the people who show up when the alarm sounds. Those
questions are real, they are answerable, and they deserved better than a
final chapter's gesture at them.

They are the next book. This one was the destination; that one is the road.
If you have read this far — past the rules, past the failures, past the
paragraph where I told you exactly who has not yet checked this work — then
you are the reader that book will be written for.
