<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Full-Society Scope-Review Protocol

> **Status: repository-enforced 2026-08-23 -- receipt-aware mechanical-closure protocol v5.**
> Protocol v5 supersedes v4 for every new candidate. Gate A uses an append-only,
> source-derived repository audit and a mechanically derived closure record
> bound to one immutable, content-addressed full-verification receipt. Exact
> audit, closure, and tracker successors may reuse that full result only while
> its heavyweight dependency manifest remains byte-identical and their narrower
> structural validators pass. External human review remains welcome optional
> evidence. It never blocks Gate A, Gate C, Gate E, a book release, or project
> completion.

Adopting protocol v5 administratively reopens Gate A while its receipt-bound
audit and closure are pending. This changes no constitutional rule, Book 1
prose, claim, allocation, census, or operational posture.

This migration uses source version
`fs-ledger-2026-08-23-verification-receipt-v2`; later schema-v2 candidates bind
their own new source versions.

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

A protocol-v5 passing row also binds the exact tracked schema-v2 verification
receipt whose local expanded evidence was present when the row was admitted.

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

For any semantic, executable, verifier, fixture, engine-binding, or
generated-artifact change:

1. Regenerate and reread every source-derived projection.
2. Stage the complete candidate, with no unstaged change or non-ignored
   untracked file.
3. Run one full `./verify.sh --emit-receipt
   new-book-plans/verification-receipts`; full verification already includes
   the structural path, so no unchanged quick run precedes it.
4. Commit exactly the staged tree bound by that receipt.
5. Append the receipt-bound current-source audit, recording every
   Gate-A-applicable defect row, and admit it through the `audit` commit gate.
6. Correct any failure by producing a new full receipt. Never silently fall
   back from failed reuse to a full run.
7. Preserve any failed committed audit as an append-only prefix.

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

Gate A v5 closes through a verified fully staged candidate followed by
classified administrative successors. The fully staged candidate has a null
closure record and
`not-passed` gate state. Its staged tree, path/mode/blob manifest, verifier
inputs, fixtures, generated artifacts, engine identity, sanitised environment,
command outcomes, and transcript digest are bound by one schema-v2 receipt.

The audit successor may append only the exact receipt-derived current audit,
track that compact receipt, and refresh its deterministic projections. The
closure successor may change only closure metadata, acceptance metadata, and
their deterministic projections. Its closure record cites the verified
candidate, exact current audit, versioned-structure envelope, checker-derived
assurance and residual sets, exact claim limitations, and receipt. A final
tracker successor may delete only the completed unreferenced task while every
active `path::needle` multiplicity remains unchanged. Each successor must be a
normal single-parent commit with no intervening unclassified change, preserve
the receipt's heavyweight dependency manifest byte-for-byte, pass its strict
diff validator, and rerun the structural verifier. The validator derives
`passed` from the closure record and rejects semantic or audit drift. No
author ratification, reviewer action, or other human act is required.

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

## 7. Receipt, lock, and reuse contract

Schema v2 receipts are compact, canonical JSON files named by their SHA-256
self-digest under `new-book-plans/verification-receipts/`. Expanded manifests
and transcripts live under the Git common directory so linked worktrees share
the same evidence cache. Reuse fails closed when that local evidence is absent;
it never claims the digest alone reproduces or independently authenticates the
run.

Emission requires one fully staged candidate, no unstaged tracked change, and
no non-ignored untracked file. The receipt binds the prospective Git tree and
its parent, every tracked path/mode/blob entry, a closed dependency
classification, verifier and fixture bytes, generated outputs, the resolved
`nibli-pin` binary and source identity, a sanitised outcome-relevant
environment, command results, and transcript digest. The receipt self-digest is
computed over canonical JSON without the self-digest field; its filename must
match. Changing any bound byte, path, mode, engine, or environment member
requires a fresh full run.

Heavyweight verifier entry points share a kernel `flock` under the Git common
directory. Contention fails immediately with exit 75 and secret-free owner
details unless the caller supplies an explicit bounded wait. A nested verifier
child may inherit ownership only after validating the live owner, process
ancestry, start identity, and random token. Process exit, including a crash,
releases the kernel lock; diagnostic metadata never authorises breaking a busy
lock.

Scripts 13 and 16 use `--refresh-and-check` to snapshot immutable inputs,
execute each watched mutation independently once, render and byte-check
temporary outputs, replace the complete output set atomically with rollback,
reread the installed bytes, and reject concurrent input drift. Caches may
retain immutable source bytes, parsed forms, digests, counts, fingerprints, and
Git objects, never mutant-derived verdicts.

State-form execution retains 64 main and 17 counterfactual shards. Shard-index
schema v2 partitions the canonical query stream into contiguous blocks by
exact rendered UTF-8 bytes including transitive fixture closure. A sliding
four-worker pool preserves canonical output order, stops launching after
failure, and terminates and reaps remaining workers.

`--commit-gate` validates only one named transition:

- `audit` permits the receipt-bound audit row, the compact receipt, and exact
  audit-derived projections;
- `closure` permits closure and acceptance metadata plus their exact
  projections; and
- `tracker` permits only the completed tracker deletion and requires a fresh
  census of every active `path::needle` reference.

All three rerun the structural verifier and staged-diff check. Missing local
evidence, a merge, an intervening unclassified commit, an unexpected path or
mode, source drift, generated-byte drift, or a broadened delta rejects reuse
without launching a silent full run. The evidence claim is therefore
byte-identical heavyweight inputs plus a narrowly revalidated administrative
delta, not byte identity of the whole successor tree and not an exact or
hermetic execution environment.

The only accepted schema-v1 receipt is the historical tuple:

- candidate `e0e0ca1a09dc8bceaac95f29ab5f1afdc9795bb5`;
- source `fs-ledger-2026-08-21-state-form-prose-v1`;
- audit `FS-SAU-34`; and
- transcript digest
  `dc0eb1d869629a9093457fcc8a7c48d5a438777bae756e24a0447e4d60e1032f`.

That exact allowlist preserves the earlier closure without upgrading its
evidence. Every later candidate requires schema v2; omission of a schema cannot
downgrade a new receipt.

## 8. Superseded protocol-v4 record

The following text preserves the protocol-v4 status and closure rule as
historical context, not current instructions:

> **Status: repository-enforced 2026-08-15 -- mechanical-closure protocol v4.**
> This protocol supersedes every requirement for a human act to complete a
> project or publication gate. Gate A uses an append-only, source-derived
> repository audit and a mechanically derived closure record bound to an
> immutable verified candidate. External human review remains welcome optional
> evidence. It never blocks Gate A, Gate C, Gate E, a book release, or project
> completion.

> Gate A closes through two machine-checkable commits. The first freezes an
> immutable candidate with a null closure record and `not-passed` gate state,
> then runs the exact verifier chain and records its transcript digest. The
> second may change only closure metadata, acceptance metadata, and generated
> projections. Its closure record must cite that candidate, the exact current
> audit, the versioned-structure envelope, checker-derived assurance and
> residual sets, exact claim limitations, and the complete verification
> receipt. The validator derives `passed` from that record and rejects semantic
> or audit drift. No author ratification, reviewer action, or other human act is
> required. Any semantic map change after the candidate requires a new audit
> and candidate.

Author statement, 2026-08-15: "Right now, I cannot depend on other reviewers to finish the book. Please remove dependency on human reviewers on this one and going forward"
