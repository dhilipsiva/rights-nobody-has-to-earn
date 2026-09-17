<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Nibli Finite Collective-Decision Capability Audit

> **Status: completed capability audit; measured 2026-08-07 and independently
> rechecked 2026-08-08.** This record constrains later formalisation. It adds no
> constitutional predicate, rule, fact, pin, institution, result, or release.

## 1. Evidence stamp

The original read-only engine audit ran against Nibli
`7904d2c175dddc610f3b66d6e083ebccdd3e3c08`, workspace version `0.1.0`. At that
time, `main` did not contain the required book baseline
`4cb02aade43b394374c40e661907ad66df3af3fe`; the audit therefore reported that
ancestry failure rather than hiding it.

On 2026-08-08, the book session independently rechecked clean engine
`main == origin/main` at `a7d288a9ab9b9eb4e0f282469c3bc278ec94894e`.
That merge has parents `7904d2c` and `4cb02aa` and therefore contains the
required baseline. A fresh release `nibli-pin` had SHA-256
`932b0fb2530711cadb6a9b2ded26bbe80f972bf82dd18726afa39ce2a8fc7293`.
The neutral surface fixture passed 21 of 21 pins, the native probes reproduced
the findings below, and the full book verifier passed with that binary. The
exact pin rerun was:

```text
/tmp/nibli-a7d288-codex-20260808/release/nibli-pin \
  /tmp/nibli-capability-a7d288-probe/surface-boundaries.pins.nibli
```

The native probe, raw-witness reproducer, and text-compute reproducer used
the same temporary Cargo project:

```text
cargo run --manifest-path /tmp/nibli-capability-a7d288-probe/Cargo.toml \
  --target-dir /tmp/nibli-a7d288-probe-target --quiet
cargo run --manifest-path /tmp/nibli-capability-a7d288-probe/Cargo.toml \
  --target-dir /tmp/nibli-a7d288-probe-target --bin raw_witness --quiet
cargo run --manifest-path /tmp/nibli-capability-a7d288-probe/Cargo.toml \
  --target-dir /tmp/nibli-a7d288-probe-target --bin boundary --quiet
```

Those probe sources are not a committed fixture and the `/tmp` paths are
non-durable; this record preserves their measured conclusions, not a false claim
of permanent reproducibility. These are dated, source-bound observations, not
timeless engine guarantees.

## 2. Measured boundary

| Constitutional case | Nibli can establish from supplied facts | Outside Nibli / responsible component |
| --- | --- | --- |
| Two-thirds of the full Assembly | Exact literal roster and vote observations over the supplied snapshot; a two-named-voters-among-three fixture | Independent completeness assurance; result service computes generic `ceil(2R/3)` as the roster changes |
| National referendum | Exact counts of definitively classified supplied records | Election administration authenticates and classifies records; independent assurance attests completeness; result service compares counts, applies tie failure, and certifies |
| Regions Council and affected regions | Sum definitively supplied numeric weights; conjoin supplied certificates | Independent assurance attests complete regional rosters; result service compares, groups, and certifies every-region consent |
| Return and repassage | Represent return, reasons, and a supplied same-rule certificate | Enforce return-once history and determine that the dynamic original passage rule was met |
| Constructive confidence or recall | Require result and successor certificates tied to one decision | Result service computes and certifies the majority, unique successor, and tie outcome; authorised institutions remove or install |
| Initiative and counterproposal | Route a supplied winning or tie certificate to an explicit legal consequence | Compute and compare shares, prove both passed, and authenticate the outcome |
| One effective submission | Represent record identity and detect conflicting person/decision choices | Election administration authenticates records; independent assurance attests completeness; the result service applies Book 1's functional-key/conflict rule and certifies the effective record |

No general empty-roster passage rule was supplied. Nibli can report an exact zero
for a supplied empty snapshot; it cannot decide whether zero members satisfy a
legal threshold. Book 1 must specify that result for each route before
formalisation. Likewise, "no turnout quorum" means that Book 1 omits a turnout
condition; it is not an engine primitive.

A fixed two-of-three rule remains true after a fourth roster member is added. It
therefore proves a literal witness pattern, not a roster-parametric majority.
No such rule may be presented as generic election semantics.

## 3. Surface contract

- Ordinary KR rules can join facts, require fixed finite witnesses, and compose
  supplied result certificates.
- Exact-count and executable-compute nodes are query-only. They cannot be stored
  as facts or used in rule antecedents or conclusions. A count result is not a
  numeric term available to a later comparison. `NibliEngine::validate` is only
  a compile check and accepts query IR; admission checks must use `assert_text`
  or `KnowledgeBase::validate_assertion`.
- `nibli-pin` can pin an exact-count verdict but cannot return a tally or feed one
  pin's count into another rule.
- Native APIs expose find, count, and aggregate for definitive finite positive
  queries. Any comparison performed by the caller is caller policy, not a Nibli
  derivation. The raw non-definitive enumeration path described in section 5 is
  excluded.
- Component hosts expose query/find rather than a distinct count or aggregate
  operation; a host may count returned bindings, subject to the completeness
  limits below.

An asserted result certificate is an ordinary trust-root premise. A green proof
shows only that the legal conclusion follows from that premise. It does not
authenticate the signer, roster, submissions, publication, freshness,
completeness, correction, availability, or challenge route.

## 4. Completeness, identity, and defaults

An incomplete roster and a genuinely smaller roster are observationally
identical when given the same facts. Positive completeness evidence can block
authority when absent, but its writer then holds a withholding veto. The legal
contract therefore needs independent, redundant or alternate attestation and a
failure route; Book 2 must operate it.

Repeated assertion of one structural fact remains one logical tuple. Distinct
record constants remain distinct even when their content matches, and multiple
derivations do not create extra vote weight. Conflict detection does not select
a winning record. Domain record identity must be explicit in logical arguments;
fact-registry identifiers are only provenance and retraction metadata.

`FALSE` means not derivable from the current closed-world snapshot. It is not a
classical negation and cannot affirmatively preserve an incumbent, ordinary
term, or current law. Persistence and other failure defaults require positive
rules driven by supplied outcome certificates. `UNKNOWN(reason)` and
`RESOURCE_EXCEEDED(kind)` remain distinct non-definitive results.

## 5. Confirmed engine follow-ups

Two defects are separate from completion of this audit:

1. On both audited sources, the raw `KnowledgeBase` API returned an empty
   `query_find` result and zero `count_witnesses` for a witness set whose
   entailment and exact-zero query were `Unknown(NafDependent)`. Find/count must
   refuse incomplete enumeration. The same omitted path is suspected for
   `Unknown(NonFinite)` and belongs in the repair acceptance test.
2. Registering a novel compute predicate such as `quorum` does not make that
   name available to text compilation; the compiler rejects it as outside the
   corpus before registration can produce a compute node. The implementation or
   the public documentation must be narrowed to one truthful contract.

Neither defect authorises a book-side workaround. Collective-decision
formalisation must not use unsafe raw enumeration or pretend a fixed finite rule
is dynamic arithmetic. It may proceed through authenticated, contestable,
bounded external result certificates while the engine follow-ups remain open.

## 6. Responsibility seam

Book 1 owns the legal threshold, admissible record shape, completeness and
uniqueness requirements, tie and empty-roster defaults, challenge, correction,
continuity, and the exact legal effect of a certified result. Book 2 operates
roster and submission administration, classification, arithmetic, grouping,
authentication, publication, availability, recount, certification, correction,
independently authored alternate completeness assurance, and institutional
execution through the bodies Book 1 authorises. Nibli may consume the resulting
bounded premises and prove their Book 1 consequences; it is not the election
service, completeness assurer, or executing institution.
