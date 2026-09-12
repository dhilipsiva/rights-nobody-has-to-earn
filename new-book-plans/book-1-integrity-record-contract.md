<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Integrity records — common constitutional contract

Implementation of the 2026-09-09 democratic and administrative integrity
decision, under the simplified verification decision of 2026-09-12. This
contract describes legal effects over supplied records, not operating bodies,
successful investigation, authentication, disclosure arrival, or enforcement.

## Source and scope

`integrity-source.json` owns the new finding vocabularies and consequence
rules. `./generate.sh integrity` explicitly writes the
`DEMOCRATIC-INTEGRITY-RULES` block in `constitution.nibli` and the isolated
ordinary cases in `tests/pins/integrity/`. State-form clearance fields remain
in `state-form-source.json`; disclosure retains the FS-CVF-016 typed duty
bridge and its explicit obligations generator. There is no receipt, hash,
ledger successor, or report gate.

Family contracts:

- [Money and influence](book-1-money-and-influence-contract.md), including payer identity.
- [Office integrity](book-1-office-integrity-contract.md).
- [Disclosure duties](book-1-disclosure-duty-contract.md).
- [Districting](book-1-districting-integrity-contract.md).
- [Opposition and party democracy](book-1-opposition-and-party-democracy-contract.md).
- [Coordinated operations](book-1-coordinated-operation-contract.md).

The reader projection is [Keeping public decisions answerable](../book-1/09-the-vote-conviction-does-not-take.md#keeping-public-decisions-answerable)
in Book 1 chapter 9: `session-drafted, author-approved` on 2026-09-12, with
the exact approved wording retained in `integrity-reader-draft.md`. The
chapter's paired pins map its claims to the isolated executable cases.

Each finding uses `observe/4` on an exact finding record. Its source,
evidence writer, and independent reviewer must have different identities and
the record-specific authorizations `ElectoralOrIntegritySourceAuthority`,
`IndependentIntegrityEvidenceAuthority`, and `IndependentIntegrityReviewAuthority`.
An act, office, association, opposition group, or operation cannot review its
own record. This is a constraint on supplied identities, not proof that the
writers are independent in the world.

Each attests the same target, source version, jurisdiction, legal scope, end
condition, positive current/reconciled disposition, evidence disposition,
reader, independent alternate, challenge, correction, and remedy route. The
final route is the Constitutional Court or the independently constituted
substitute panel when the Court's own composition is at issue. The reader and
alternate have separate authorizations and cannot be the same office.
The source fixes a proportionate, purpose-limited private record and the
non-use wall protecting personal status, floors, the equal ballot and liberty.
No field publishes a small payer's identity. Publication thresholds, amounts,
periods, and cooling-off periods remain democratic law.

`CurrentReconciledIntegrityRecord` is a supplied current-selection certificate,
not an observed clock. Replacing it with a stale or superseded disposition
withholds every effect. False certificates remain an external trust-root
failure; the reasoner neither authenticates them nor discovers time passing.
Corrections and expiry require a fresh supplied record/snapshot, not an
assumption that adding a correction erases earlier evidence.

## Consequences and polarity

A current adverse finding derives
`contradict(target, DemocraticIntegrityAuthorization)`. This is a **named
legal incompatibility**, not a `:contradictions` scanner finding. The affected
state-form result or plan/selection permission reads that exact marker and is
withheld. An absent, incomplete, mismatched, withheld or stale finding cannot
derive it. Existing positive clearance requirements still apply: failure to
derive permission from incomplete evidence is not a finding of misconduct.

The marker gates acts, not permanent `authority(person)` answerability.
Residence, political-home continuity, adulthood and the equal-ballot branches
do not consume it. An unrelated act does not consume it either. State-form
office permissions are `authority(holder, power, record)`; confusing these
with the older one-place answerability conclusion would contradict Book 1.

The existing `contradict` relation is used because its producers read supplied
records. Using `collide` here was tested and refused: custody already derives
that relation from `complete`, making a negative `complete` → `collide` gate
unstratifiable. This is an expected engine refusal, not a Nibli bug, and no
new admitted relation is required.

## Cases and review

Every contract has a positive case, missing-field cases, withheld and stale
cases, self-review refusal, and a counterfactual removing only the named
source/reviewer inequality. Variable kinds are exercised individually and an
unratified kind is rejected. Stateful cases first establish a permission,
then add an adverse finding and check the changed permission alongside the
unchanged personal protections. The complete verifier executes these cases
against the actual constitution and checks formal contradictions.

The source guard for actor-side manipulation belongs to the development test
suite, not a resurrected repository-audit gate. Contract-specific cards name
the evidence boundary and Book 2 handoff. None of these cases establishes an
institutional action, a complete register, independence in practice, or future
delivery of a duty.
