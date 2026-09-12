<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Counterfactual fixtures

These pins ask what changes when a constitutional statement is removed, replaced,
or added. Extra facts do not remove a condition from an existing rule: a claim
such as "without this line, the protection disappears" must run against an
actually changed source.

The executable edits now live in
[`tests/pins/suites.json`](../../tests/pins/suites.json). The runner applies them
in memory to the current constitution. This directory retains the original pin
files and these explanations, not complete copied constitutions. There is
nothing to regenerate after a comment-only constitution edit, and no source hash
or historical diff-shape check.

Run from the repository root:

```bash
./verify.sh
./verify.sh --list
./verify.sh --only new-book-plans/counterfactual/no-person-line.pins.nibli
```

The last command is a focused, explicitly partial result. A counterfactual's pins
are expected to pass in its changed world, not necessarily against the live
constitution. Deliberate counterfactuals normally use `scan: false`: their pins
test the intended consequences without claiming that every intentionally
sabotaged world is contradiction-free. The live constitution and ordinary
scenarios retain their contradiction scans.

## Editing a counterfactual

Each named base derives from `live` or another named base and supplies ordered
`{ "before": "...", "after": "..." }` edits. Deletion uses an empty `after`;
addition uses an empty `before`. Replacement supplies both. A nonempty target
must match exactly one contiguous block of statement lines. Blank lines,
comment-only lines, and outer line whitespace are ignored; statement content
remains exact. Review the relevant edit when changing an affected rule.

The runner then loads the scenario fixtures and executes its pins in isolated
state. Keep controls beside a changed verdict so a test cannot pass merely
because the entire rule family stopped working. Detailed inventory and
authoring instructions are in [the pin-suite guide](../../tests/pins/README.md).

State-form and obligations cases have explicit authoring commands:
`./generate.sh state-form` and `./generate.sh obligations`. These update rules,
aggregate pins, and extracted case assets. Ordinary verification never invokes
them or enacts a pending authoring-source change.

## Why these changes are tested

- Deleting a statement shows what depends on it: personhood, a public body's
  answerability, a credential route, or a first-contact standing root.
- Replacing a rule isolates a restriction. `no-dead-conjuncts` removes
  Article 4's broken/carried-void signer checks and deliberately reuses chapters
  4 and 5's pin files: their unchanged answers show those conjuncts decide
  nothing in the current examples.
- Adding a statement explores a postulated future. `unguarded-pen` adds a
  credential route that omits those checks; its pins show a carried-void
  signature counting through that route.

`undelivered-marker` adds the rejected rule
`all $x: owe(State, Eats, $x) & ~eats($x) -> err($x, Undelivered).`
Its pins show the marker firing on the voided, confined, and never-accused alike.
That is the reason missing arrival evidence must not itself become proof of
non-delivery.

`no-delivery-independence` removes only the food rule's witness/source
disequality. A kitchen can then attest its own delivery; the independent food
route and shelter's refusal of self-certification remain controls.

`no-state-form-independent-current-review` removes only the shared
source-writer/temporal-reviewer disequality. Each card's fused-role example gains
the authority withheld by the live source, while properly separated controls
remain true. Its executable examples are under `tests/pins/state-form/`.

The obligations variants remove the source-writer/record-reviewer separation,
the family's source-dependent conclusion rules, or only the typed finding-reader
bridge. They distinguish fused-role effects, lost legal conclusions with
personhood intact, and lost typed duties with legacy findings intact. Their
executable examples are under `tests/pins/obligations/`.

The card-specific economic variants remove only the source-writer/reviewer
separations in one card's current and reviewed-result rules. Dependency records
keep remedy and execution powers joined to the exact reviewed result consumed.

| Variant | Deleted statement | Tested consequence |
|---|---|---|
| `no-person-line` | Prisoner-to-person standing rule | A heresy rule can load and make the population imprisonable for belief. |
| `no-public-court` | `public(Court).` | Court answerability and Sly's shield disappear. |
| `no-choose-boss` | `choose(Electorate, Boss).` | Boss's answerability disappears and the whistleblower Rebel is confined. |
| `no-first-contact-standing` | First-contact standing rule | An unregistered contact loses that standing root, floor, and State debt; other roots remain controls. |

## Liberty and ecological fixtures

The `no-environmental-right` fixture deletes the present-person environmental-conditions rule while retaining environmental information and the material floor.
The `no-class9-climate-axis` fixture deletes only the climate-axis ceiling rule while retaining the clean-air axis and the material-floor boundary.
Both are one-statement deletion variants applied to the current constitution by
the suite inventory.

## Substantive-equality fixtures

The `no-direct-equality`, `no-equality-data-wall`, and
`no-positive-measure-end` fixtures each delete exactly one independently
registered equality effect. Their controls keep an adjacent equality effect and
the common status baseline live, so a green result cannot come from losing the
whole family. Their inventory edits delete, respectively:

- `all $x: person($x) -> prevents($x, DirectDiscrimination).`
- `all $x: person($x) -> prevents($x, EqualityDiagnosticRecordReuse).`
- `all $x: person($x) -> prevents($x, ExpiredPositiveMeasureContinuation).`

## Family and life-course fixtures

The family and life-course suite adds four one-line deletion fixtures:

- `no-automatic-adulthood` removes the automatic general-adulthood protection while retaining the ballot control;
- `no-family-confinement-wall` removes only the explicit family-status placement wall;
- `no-missing-kinship-independence` removes only the affirmative-independence boundary; and
- `no-pregnancy-authority` removes only pregnancy continuation and termination authority while retaining the fetal-override refusal.

Their inventory entries apply those deletions to the current source at run time.

## Economic, labour, property, and fiscal fixtures

The economic suite adds three one-line deletion fixtures, three family-wide
deletion fixtures, and 28 card-specific changed-line fixtures:

- `no-economic-floor-gate` removes only the rule that refuses price, wealth,
  credit, contribution, insurance, or ability-to-pay gates on the floor;
- `no-economic-data-wall` removes only the purpose limitation on contribution
  records;
- `no-economic-work-freedom` removes only the choose, refuse, leave, and change
  work barrier; and
- `no-economic-direct-effects` removes all 145 independently registered
  person-held economic effects, proves each exact effect disappears, and keeps
  personhood as a control;
- `no-economic-power-duty-bridges` removes the 171 power-conditioned and
  alternate-review duty conclusions while retaining a source-bound always-on
  duty and the direct-effect family as controls; and
- `no-economic-carry-results` removes the six non-power benefit, title, and
  liability current/result rules, proves every exact carry conclusion
  disappears, and keeps a person-held economic barrier as a control; and
- `no-economic-independent-current-review-061` through `-088` each remove the
  source-writer/current-reviewer separations from exactly one economic power
  card.

The deletion variants retain adjacent effects as controls. The inventory names
the actual statement edits; the runner rejects missing or ambiguous targets
before executing the corresponding pins. No copied constitution or separate
diff-shape report is maintained.

## Income security and social insurance fixtures

The income-security suite adds one deletion fixture, one changed-line fixture,
and one added-line fixture:

- `no-income-supplement-rule` removes only the rule that concludes the
  supplement from a contribution record and an independent adjudicated event;
  the guarantee rule and the floor are retained as controls.
- `no-income-adjudicator-independence` strips only the `~($a = $carrier)`
  conjunct from that rule, so the carrier adjudicating the event it would have
  to pay for starts counting; the independent route and the guarantee route's
  own refusal are the controls.
- `unguarded-contribution-reader` is the constitution *plus* a rule that
  conditions confinement on the absence of a contribution record. The engine
  accepts it — `pay` is a base relation, so there is no negative cycle for the
  stratifier to refuse — and its pins show it confining the whole roster. The
  inventory appends
  `all $x: person($x) & ~pay($x, Contribution, Carrier, SchemeA) -> prisoner($x).`
  This tests the consequence of that hostile reader; it does not automatically
  prohibit every future reader.

The first two inventory edits delete the supplement rule or replace it with the
same statement minus the adjudicator/carrier disequality. The guarantee rule
remains unchanged.

## Qualifications and compensation fixtures

The qualifications-and-compensation suite adds one deletion fixture, one
changed-line fixture, and one added-line fixture:

- `no-certificate-rule` removes only the rule that concludes a certificate from
  an authorised certifier's attestation; compensation and the floor are retained
  as controls.
- `no-compensation-attester-independence` strips only the `~($a = $payer)`
  conjunct from the compensation rule, so a payer attesting its own promise
  starts counting; the independent route and the restitution rule's own refusal
  are the controls.
- `unguarded-compensation-reader` is the constitution *plus* a rule that
  conditions confinement on the absence of a promised wage. The engine accepts
  it. Its inventory entry appends
  `all $x: person($x) & ~promise(Firm, Wage, $x) -> prisoner($x).`
  The pins expose the consequence; reviewing new readers for the intended
  purpose limit remains necessary.

The first two are explicit statement edits against the current constitution,
like the income-security variants.

## Public-scale vocabulary fixtures

The public-scale vocabulary suite adds three changed-line fixtures and one
added-line fixture. All four use the same composed FS-POW-064 finding, whose
record names a trigger, a function class and a tier allocation that the ratified
vocabularies do not contain:

- `no-public-scale-trigger-vocabulary` strips only
  `member($trigger, PublicScaleTriggerVocabulary) & ` from the reviewed-result
  rule;
- `no-public-scale-function-class-vocabulary` strips only
  `member($function_class, PublicScaleFunctionClassVocabulary) & `; and
- `no-public-scale-tier-allocation-vocabulary` strips only
  `member($allocation, PublicScaleTierAllocationVocabulary) & `.

Each pairs the flipped verdict with the membership query that stays FALSE, so a
green result cannot come from having also broken the vocabulary. Against the
real constitution the same finding is refused; here it derives while the token it
names is still not a member. That pair is what makes "the finding may state only
a named ground" an executed claim rather than an attested one. Each inventory
edit replaces the affected rule exactly once; a missing target fails instead of
silently testing an unchanged copy.

`unnamed-public-scale-trigger` is the constitution *plus* one more ground rule,
for a token no ratified vocabulary contains. Its inventory entry appends

```
all $source: all $record: observe($source, $record, EconAnnualRevenueOver500M, PublicScaleTriggerScope) -> member(EconAnnualRevenueOver500M, PublicScaleTriggerVocabulary).
```

Its pins show the finding deriving and the token holding membership, which is
the harm the enumeration exists to prevent: the vocabulary grows by one source
edit and the finding stops noticing. The former producer-set administrative
check is retired; these pins test the source change itself.

Every fixture whose record composes an FS-POW-064 finding carries the function
class and the tier allocation on the record and the temporal record as well as
on the result, because both the current-selection rule and the reviewed-result
rule read them there. A record that carries only the result-side attestation
derives nothing, and every negative case above it would then pass for the wrong
reason.

## Appointment anti-capture fixture

`unnamed-appointment-control-source` is the constitution *plus* one more ground
rule, for a source kind the ratified state-form sentence does not name.
Its inventory entry adds

```
all $source: all $record: observe($source, $record, IncumbentCoalitionAffiliateAppointmentSource, AppointmentControlSourceKindScope) -> member(IncumbentCoalitionAffiliateAppointmentSource, AppointmentControlSourceKindVocabulary).
```

The pins record whether that added vocabulary entry affects the tested result.

The anti-capture family has no gate-removal fixture, and the reason is worth
recording rather than leaving as an omission. Its five source kinds and two
control modes are fixed constants in the rule, not variables, so stripping the
`member` conjuncts changes nothing a probe could see — the observations are
still required. What the conjuncts buy is that deleting a required vocabulary ground rule stops the affected
family deriving. Review that dependency directly when changing the authored
rules; the old source-schema self-controls are no longer verification gates.

The measurement the family exists to repair, taken 2026-09-08 against one
generated FSPOW_028 positive case: with all 194 supplied facts the reviewed
result and the holder authority both derive; with the three party-coalition
attestations removed both stop; and with all seven named grounds removed but the
blanket `NoMajorityDirectOrDeFactoControl` attestation still present — which is
exactly the shape the source carried before this family — both stop as well. An
unexamined kind is no longer indistinguishable from an absent one.

## Office integrity fixture

`unnamed-office-integrity-kind` is the constitution *plus* one more ground
rule, for a material-interest kind the 2026-09-09 ruling does not name — party
membership, which is the guilt-by-association widening the ruling refuses.
Its inventory entry adds

```
all $source: all $record: observe($source, $record, PartyMembershipInterest, MaterialInterestKindScope) -> member(PartyMembershipInterest, MaterialInterestKindVocabulary).
```

The corresponding pins describe the consequence of that vocabulary addition.

The family has no gate-removal fixture for the reason the anti-capture section
above records: its kinds are literal constants. The `member` conjuncts retain
the dependency on the named vocabulary; the retired source-schema checker is
not part of the pin suite.

One consequence of a universal state-form field that is easy to miss: every
hand-written pin file that supplies a state-form result as a *dependency*
must carry the new attestations too, including counterfactual economic cases,
not only the positive economic-power suites. Review the actual inventory and
all affected fixtures when changing that shared dependency.

## Money and influence fixture

`unnamed-political-finance-payer` is the constitution *plus* one more ground
rule, for a payer kind the 2026-09-09 money-and-influence ruling does not name
— an anonymous intermediary, which is precisely the category whose admission
would hollow out the `ControllingPartyPayer` shell test. Its inventory entry adds

```
all $source: all $record: observe($source, $record, AnonymousIntermediaryPayer, PoliticalFinancePayerKindScope) -> member(AnonymousIntermediaryPayer, PoliticalFinancePayerKindVocabulary).
```

Its pins describe the changed vocabulary. The examined-kind families share an
authoring table, so changing a universal field requires reviewing every case
that supplies a state-form result, including the economic counterfactuals.
