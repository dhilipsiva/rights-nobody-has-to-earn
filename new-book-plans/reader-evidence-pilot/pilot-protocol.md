<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Reader-Evidence Pilot Protocol

> **Status: draft template; not pre-registered or frozen.** This protocol has
> not recruited anyone, run a session, produced evidence, or supplied a release
> rule. Bracketed fields must be completed privately before a valid freeze.

## 1. Purpose and warrant

This pilot tests whether the instrument and coding process can observe how a
bounded sample understands a declared Book 1 snapshot. It may expose defects
in the questions, rubric, navigation, accessibility, or tested prose. It does
not test release suitability, estimate a population, establish any domain
assigned to another assurance route, or create a reader claim about the book.

The pilot must run only after the Reader's Map, glossary, and accessible
navigation exist. It tests one exact versioned snapshot and the exact
pre-registered instrument and rubric. No reader result enters Nibli.

## 2. Private commissioning record

Before the freeze, private custody records:

- opaque study ID: `[opaque-study-id]`;
- tested snapshot identity and manifest: `[private-manifest-reference]`;
- repository commit containing the bound public templates: `[commit]`;
- facilitator identity: `[private]`;
- coder A identity: `[private]`;
- coder B identity: `[private]`;
- adjudicator identity: `[private]`;
- reviewer/custodian identity: `[private]`;
- recruitment channels and dates: `[private]`;
- applicable compensation-rate source and frozen rate: `[private]`; and
- independent ethics/safety review disposition: `[private]`.

None of those identities, mappings, contact details, or compensation records
belongs in the public repository.

## 3. Required role separation

The facilitator, coder A, coder B, adjudicator, and reviewer/custodian are
separate people. The author holds none of those study roles. The combined
reviewer/custodian role may be held by one person, but that person may not
facilitate, code, or adjudicate.

- **Facilitator:** obtains session confirmation, gives the exact script and
  prompts, protects accessibility and safety, and records procedural
  deviations. The facilitator never teaches, repairs, scores, or adjudicates a
  response.
- **Coders A and B:** code the same private response records independently.
  Each is blind to the other's initial codes and notes until both initial
  records are irrevocably submitted to the custodian.
- **Adjudicator:** receives both locked initial records and resolves or marks
  disagreements under the frozen rubric. The adjudicator does not invent a
  release taxonomy or threshold.
- **Reviewer/custodian:** controls identity mappings and private records,
  verifies role separation and admissibility, freezes artifacts and decision
  packets, publishes privacy-minimal attestations, and operates the later
  digest-bound admission interface. The custodian does not change a content
  code to improve an outcome.

Recruitment administration may be performed by a separate coordinator or by
the facilitator, but screening details must not be disclosed to coders.
Compensation is never contingent on admissibility, coding, findings, or
control outcome.

## 4. Design fixed for this pilot

The study is remote and moderated. It recruits non-specialist readers with
varied reading confidence, language backgrounds, and accessibility needs.
Variation guides recruitment; it is not a demographic quota or a pass rule.

The operator targets 10 admissible completed sessions, requires at least 8,
and permits no more than 12 attempted sessions. The attempt definition and
stopping rule are fixed in `sample-and-recruitment-rule.md`.

Each participant examines the frozen snapshot independently before the
session. In the session, the facilitator asks the six ratified prompts in the
frozen order and without content assistance. Diagnostic feedback begins only
after every unaided response has been closed.

## 5. Frozen study packet

The pilot pre-registration binds exact digests for:

1. this protocol and the controlling protocol decision;
2. the private runnable instrument and participant-facing disclosure, bound by
   the public minimum in `instrument-template.md` and an exact SHA-256 reference;
3. the private exact rubric codebook, bound by the public minimum in
   `coding-rubric-template.md` and an exact SHA-256 reference;
4. `sample-and-recruitment-rule.md`;
5. `disclosures-and-ethics.md`;
6. the private exact falsification rule and seeded-control preimage, bound by
   `provisional-falsification-rule-template.md`, `seeded-control-template.md`,
   nonce-protected commitments, and exact custody references;
7. the tested snapshot manifest and its HTML, EPUB, and PDF artifact hashes;
8. the accessible-navigation validation record;
9. the private recruitment and role-assignment records; and
10. the external freeze payload and custody attestation.

The public freeze receipt must bind the exact computed
`attested_payload_sha256`. An opaque receipt digest with no payload binding is
insufficient. Freeze time uses canonical UTC and follows registration. The
frozen, `not-run` attempt is committed before recruitment or collection.

## 6. Session sequence

1. Confirm the participant has the correct snapshot and usable format.
2. Reconfirm informed consent, compensation, withdrawal, privacy, and content
   warnings without asking for a public reason or personal history.
3. Resolve access or connection problems before the unaided section begins.
4. Deliver the six prompts exactly as published in
   `instrument-template.md` and bound in the private runnable instrument.
5. Close each answer before moving on. A participant may say they do not know,
   decline an answer, pause, or withdraw.
6. Only after all unaided prompts are closed, ask the diagnostic questions.
7. End the session, confirm compensation handling, and transfer the private
   record to the custodian.
8. Record any deviation immediately; do not repair the record after seeing
   codes.

## 7. Coding and adjudication

The custodian gives coders response records bearing opaque session IDs and no
direct identity, demographic, accessibility, compensation, or recruitment
data. Each coder applies the frozen rubric independently and submits an
immutable initial record. The adjudicator receives both records only after
both are locked.

Admissibility is an orthogonal finding. Inadmissible and withdrawn sessions
remain counted as attempts where the attempt definition is met, retain their
privacy-minimal lifecycle record, and publish no target or misconception
outcomes. A favourable answer never cures an ethics or protocol breach.

The privately held seeded transcript is processed through the same
instrument-facing coding workflow. Its control result is governed only by the
privately frozen rule satisfying
`provisional-falsification-rule-template.md`; it is not a participant session,
does not enter the sample, and supplies no release result.

## 8. Completion, invalidity, and successors

The pilot may be recorded as completed only when:

- the frozen packet and tested snapshot remained unchanged;
- the sample rule ended with at least 8 admissible completions;
- session admissibility and coder disagreements are closed or explicitly
  recorded as unresolved under the frozen rubric;
- exactly one study-freshness custody record binds the run;
- the protocol-validity finding is valid;
- the seeded control was observed as `watched-failing`; and
- the completion receipt binds every coded record, deviation, custody record,
  coder record, and control transcript.

If the sample minimum is not met, the seeded control is `failed-to-fail` or
`indeterminate`, coding remains indeterminate, a bound artifact changes, or a
binding/admissibility requirement fails, preserve the evidence and record the
attempt as void with a coded reason. Do not rewrite it as not run and do not
delete it. Revision requires a newly frozen successor attempt and a newly
recruited pilot sample under that attempt.

The pilot receipt is created at completion. The decision packet is frozen
strictly afterward and binds the pre-registration, tested snapshot, admitted
coded evidence, exclusions, disagreements, deviations, revised instrument,
and control transcript. A separate sensitivity brief compares defensible
later rule choices without consulting or creating holdout evidence.

## 9. Interpretation boundary

No pilot outcome is a pass or fail for Book 1. No aggregate, sentiment,
demographic balance, word count, or number of favourable answers may be
reported as a release verdict. The pilot supplies instrument evidence and a
basis for a later author choice. The complete severity taxonomy, stable
misconception IDs, core set, thresholds, denominator, repetition unit, and
missing/adjudication policies remain unset until that later candidate and
author ruling.
