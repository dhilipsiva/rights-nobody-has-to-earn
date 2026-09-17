<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Records, surveillance and automated power

Implementation contract for the author-directed TODO item of the same name,
under the 2026-08-03 expanded mandate and the 2026-09-12 verification ruling.
It supplies the conditions a holder must establish before it keeps, watches,
profiles or automates over somebody's record, and what it owes while it has
not. It stores nothing, authenticates nothing and operates no system.

## The record domains, and what a holding must establish

Nine domains are named and closed as a vocabulary: identity and status, health,
care, education, workplace, housing, finance, policing, and public decisions.
A record outside that list completes nothing, which is what the
`record-power-no-domain-vocabulary` counterfactual exists to show. The lawful
purposes are closed too — delivering the service recorded, administering the
lawful decision, protecting the subject's own rights, and independent oversight
of the holder — with their own counterfactual.

A reviewed holding needs authorized inputs from named lawful sources, necessity
and minimisation for the declared purpose, privacy, security and access control,
accuracy with correction on request, source-bound retention and lawful deletion,
subject access, challenge and protection from retaliation, no transfer, sale or
linkage outside the declared purpose, and no enrolment through floor access,
care, schooling or the courts. That last field is the existing enforcement
firewall restated where a record family can trip over it.

## Watching and automating are separate, dependent records

Processing rejoins the actual reviewed holding — subject, domain, holder,
purpose, version, period, jurisdiction, legal scope and end — so a holding
reviewed for one person cannot be borrowed for another, nor a holding for one
domain moved to another, nor one purpose swapped for a different one. All three
refusals are pinned. On top of the rejoin it needs prior individualised
authorization for covert or biometric use, least intrusive means with a defined
scope and duration, fresh authorization for each renewal, later notification to
the subject, no bulk, suspicionless or population-scale collection, and no
purchase or exchange of what could not lawfully be collected.

Automated or assisted decisions are the second dependent record, and the design
treats them as support rather than as the decision. They need an accessible
explanation of inputs and reasoning, a real opportunity to contest before
effect, human and independent review deciding the outcome, no sole automated
consequential decision, and no risk, threat, loyalty or dangerousness product in
the consequential person record — the 2026-08-02 temporary-assessment exclusion,
carried by name. The reviewer must not be the holder or any of the three
attesters. Remove that separation and nothing follows, which its paired
counterfactual shows.

**No computed output is an oracle here.** Nothing in the family reads a
computed value, and nothing concludes from one; the automated contract's
conclusions are duties to explain, to allow contest and to decide with human and
independent review, plus the subject's permission to contest.

## Access, retention, defect and nonresponse

Subject access carries an inspect, correct, delete or object permission bound to
the subject's own record, not a reusable identity, with correction, deletion and
objection private and accessible and no publication of the record or the
identity. Retention concludes a deletion or narrowing duty when the source-bound
end is reached or the purpose exhausted, with a record of what was done, no
silent extension and no recreation from copies, and a verification duty on the
reader.

A reviewed defect withholds the exact use it names — unauthorized input, purpose
creep, bulk collection, purchased unlawful data, an unexplained automated
effect, a sole automated consequential decision, uncorrected inaccuracy,
unlawful retention, blocked access or correction, retaliation for objecting, or
a breach of a non-use wall — and creates review and correction duties. A
certified reader nonresponse moves the review duty to the independent alternate.
Neither decides the underlying request.

## Polarity, separation and the person wall

Every conclusion is positive; a withheld record and a missing field both
conclude nothing, and each has its own case. Three mutually distinct attesters
must agree and none of them may be the holder; a challenge reader and an
independent alternate are distinct from each other, from the attesters and from
the holder. Two attesters disagreeing about a single-valued scope raise a record
ambiguity and nothing completes.

The subject is an opaque handle. The family concludes nothing about a person:
an entry naming somebody, even beside a raw `rotten` report, leaves personhood,
the floor debt, the ballot and the conscience barrier exactly where they were,
and derives no voiding, no confinement, no loss and no severity. That is a
pinned case. The legacy writable surface the justice contract records is
untouched — this family adds no authentication to it and reinterprets none of
it — and every head here is `derived_only` upstream, so a forged entry is
refused at assertion.

## Classification and Book 2 handoff

Holding, processing, automated support and oversight are Class 6 public
functions; privacy, access, non-use and non-retaliation protect Classes 1 and 3;
notice, explanation, contest and the alternate are Class 4 fair process; the
entry, its defects and its corrections are Class 7. No new taxonomy class and no
omnibus record predicate is introduced, and the family adds no relation name, so
the spine's predicate, derived-predicate and stratum counts and the eight floor
rights are unchanged.

Book 2 owns storage, cryptography, identity technology, retention engineering,
deletion in practice, model development and evaluation, audit tooling, case
administration and capacity. Nibli authenticates no record, observes no
execution, detects no surveillance, evaluates no model and proves that no entry
was deleted, corrected, explained or contested.

## Executable cases and what they check

`record-power-source.json` is the semantic input to `./generate.sh record-power`.
It writes the `RECORD-POWER-RULES` block — 120 rules — and 445 isolated cases
under `tests/pins/record-power/`. Every contract has positive, withheld,
per-scope omission, unauthorized-attester, stale, self-review, holder-as-reviewer,
fused-alternate, mismatched-version, period-drift and conflicting-value cases,
and every vocabulary value is exercised beside an unapproved one. The two
dependent contracts add missing-authorization and wrong-subject, wrong-domain,
wrong-version, wrong-period and wrong-purpose cases.

Four counterfactuals remove one premise each — independent-review distinctness,
the domain vocabulary, the purpose vocabulary, and the separation of the human
reviewer from the holder — each paired with the positive control showing what it
prevented.

Eight integration cases carry the sequences: a holding, the processing it
licenses and the automated support that reads it all standing, then one defect
against the holding stopping all three while the subject handle is still not a
person and the cast's rights survive; an unrelated defect leaving the record
standing; the holding failing to cover another subject, another domain or
another purpose; the automated reviewer refused when it is the holder, with its
counterfactual; a request and its certified nonresponse; a noted entry
concluding nothing; and the forged-head refusals.

## What the verifier reported

With this family in place the complete inventory passed on 2026-09-15 against
companion `979fe8b`: all **81,028 pins across 14,523 cases** with complete
contradiction checks and no findings, in 831.18 seconds, four workers and the
release binary prebuilt. The nine existing known-defect expectations still
reproduce. Peak resident memory was 21,844,384 KiB at 395% utilisation, with no
major page faults and no swaps; other machine activity was not controlled.

That is a result about the represented model over supplied records. It stores
nothing, authenticates no input, observes no watching, evaluates no model, and
is not evidence that any record anywhere was kept lawfully, corrected, deleted,
explained or contested.
