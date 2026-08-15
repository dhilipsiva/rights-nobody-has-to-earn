<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Full-Society Scope-Review Protocol

> **Status: repository-enforced 2026-08-15 -- mechanical-closure protocol v4.**
> This protocol supersedes every requirement for a human act to complete a
> project or publication gate. Gate A uses an append-only, source-derived
> repository audit and a mechanically derived closure record bound to an
> immutable verified candidate. External human review remains welcome optional
> evidence. It never blocks Gate A, Gate C, Gate E, a book release, or project
> completion.

## 1. Warrant and evidence ceiling

The repository audit answers one bounded question: does the named source
version pass the declared structural, reference-closure, projection-freshness,
allocation, defect-disposition, and watched-failing-mutation checks under the
checked-in protocol?

Its most permissive Gate A claim remains:

> The project has a versioned, reviewable scope map and assurance program.

A passing audit does not prove that the map is complete in any timeless or
real-world sense. It supplies no independent-human warrant, reader response,
external truth, operation, delivery, feasibility, liveness, calibration,
institutional independence, or authentication of the audit's own trust root.
No author, reviewer, reader, panel, custodian, or other person must perform a
later act for the bounded Gate A claim to close.

This project-level rule does not weaken any constitutional rule that requires
independent courts, reviewers, auditors, alternate authorisers, advocates, or
other separated public functions. Those are properties of the proposed
society, not dependencies on outside reviewers finishing the manuscript.

## 2. Canonical repository audit

The canonical ledger stores audits in the append-only `scope_audits` array.
Each `FS-SAU-*` row binds:

- a stable ID and title;
- the exact ledger `source_version`;
- the checker-derived semantic scope SHA-256;
- the SHA-256 of this protocol;
- a canonical UTC execution time;
- the checker-owned repository-adversarial method;
- the exact declared materiality criteria;
- the exact watched-control entry points;
- the exact command chain;
- every Gate-A-applicable defect row;
- a terminal result;
- the checker-owned closure-policy basis; and
- the byte-exact evidence ceiling.

Historical audit rows retain their recorded basis verbatim. A current-source
audit must use the checker-owned policy reference and may not depend on a new
author act.

A current-source passing audit is one whose source version, semantic digest,
protocol digest, method, criteria, controls, command chain, finding set, result,
and evidence ceiling all match the checker-owned contract. A stale or failed
audit stays in history and cannot satisfy Gate A.

The semantic scope digest excludes audit administration, deferrals, acceptance
metadata, and closure metadata. This prevents the audit from digesting itself
while still binding every semantic map and assurance-program field.

## 3. Execution and findings

For a semantic source change:

1. Regenerate every source-derived projection.
2. Run the exact checker-owned audit command chain.
3. Record all Gate-A-applicable defect rows in the audit finding set.
4. Correct any failing structural or watched-mutation condition and rerun.
5. Preserve any failed committed audit as an append-only prefix.
6. Freeze a passing current-source audit before creating a closure candidate.

A finding is resolved only by a source change, an exact classified-out
disposition already permitted by the schema, or a retained defect row with its
severity, consequence, owner, closure condition, gate applicability, and
public-claim restriction intact. Disclosure cannot cure a critical defect.

The audit is reproducible but not independent of the repository machinery.
That limitation is deliberate and public. The project removes the external
human dependency by narrowing its claim, not by relabelling automation as a
human or independent review.

## 4. Optional external review

The existing `review_commissions`, `proposals`, and
`review_events` interfaces remain available for optional external
feedback. If used, they retain their strict append-only history,
source/protocol binding, reviewer conflict rules, UTC chronology, frozen
intake, blind control handling, Darshu triage, Dhanush checking, and public
dispositions.

Those records may add evidence or trigger a semantic correction. They do not
control R7 state or any project gate. A semantic correction triggered by
optional feedback requires a new current-source repository audit. No empty
optional-review array requires a deferral.

The historical Darshu, Dhanush, and custodian designation is therefore
`retired-as-project-gate-dependency`. It remains meaningful only
inside an optional commission that actually uses that protocol.

## 5. Mechanical Gate A closure

R7 is `built` because its repository checks and watched-failing mutations are
part of the ordinary verifier. It is not `available`: that term is reserved for
admissible external evidence. R7 warrants only the bounded repository-audit
claim in section 1.

Gate A condition five is met mechanically only when a current-source passing
audit covers the exact criteria, controls, commands, and Gate-A-applicable
finding set.

Gate A closes through two machine-checkable commits. The first freezes an
immutable candidate with a null closure record and `not-passed` gate state, then
runs the exact verifier chain and records its transcript digest. The second may
change only closure metadata, acceptance metadata, and generated projections.
Its closure record must cite that candidate, the exact current audit, the
versioned-structure envelope, checker-derived assurance and residual sets,
exact claim limitations, and the complete verification receipt. The validator
derives `passed` from that record and rejects semantic or audit drift. No author
ratification, reviewer action, or other human act is required. Any semantic map
change after the candidate requires a new audit and candidate.

## 6. Going-forward project rule

No project completion gate or publication gate may require recruiting,
scheduling, receiving work from, or obtaining ratification or approval by any
human reviewer, reader, author, panel, custodian, or participant.
External feedback and reader studies are optional evidence only.

Where a former gate depended on such evidence, the permitted claim must be
narrowed to what the repository can reproducibly establish. In particular,
mechanical publication checks may establish artifact structure, source binding,
navigation, consistency, and accessibility mechanics; they may not establish
reader comprehension, lived effect, suitability, or accessibility for actual
users. Those human-response claims remain unestablished unless optional
evidence later supports them.

Author statement, 2026-08-15: "Right now, I cannot depend on other reviewers to finish the book. Please remove dependency on human reviewers on this one and going forward"
