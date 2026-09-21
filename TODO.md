<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1: revision toward an exceptional finished book

Created 2026-09-18 and refreshed 2026-09-20 at the author's request:
"Refresh TODO to make book a 10/10."

**Active: items 32–33 below are pending.** Items 01–23 were
completed on 2026-09-20 and items 24–31 on 2026-09-21. The first round's
[final manuscript review](reviews/2026-09-20-final-manuscript-review.md),
subsequent coherent commits and `CLAUDE.md` retain the assessments, repairs and
validation. This refresh opens further work; it does not reverse those
completed implementations or declare a released edition or Gate C. Book 2
remains collection-only.

The fresh review in the conversation read all 34 ordered manuscript inputs at
`c229da66`: the epigraph, opening note, chapters 1–31 and optional method,
approximately 52,000 words. It rated the manuscript **8/10**. That assessment
is distinct from the earlier review's 8.5/10 and is an editorial judgment,
not a correctness measurement or independent endorsement. It spot-checked
selected sources but did not rerun the Nibli verifier or audit every citation.
Suspected design problems must be checked against the current source before
implementation; a request for stronger justification is not an established
formal contradiction.

**The objective is a compelling, defensible, precise and readable book.** A
perfect score cannot be guaranteed or made true by closing a checklist.
The next assessment must judge the resulting book without protecting either
review's diagnosis or promising a particular rating.

The strengths to preserve are the child/prisoner pairing, the distinction
between entitlement and delivery evidence, care without ownership, accessible
review without assumed relief, and the separation of duties from performance.
The priorities are the justification and scope of recognition loss, the
reasoning behind consequential policy boundaries, worked conflicts, a less
technical reader spine, less repetition, and earlier access to the argument
for the design. Chapters 1, 5, 14 and 21, the opening and the optional method
are useful models of clarity, not exemptions from the final sequential read.

## Execution contract

- This refresh establishes the next plan. All new items begin pending; none
  of the proposed alternatives below is enacted by its inclusion here.
- When execution begins, work one numbered item at a time, in order. Each
  item includes its necessary rules, pins, prose and checks. Keep later items
  open until their own completion criteria are met.
- Recheck each review finding against the current chapter, constitution,
  substantive pins and controlling decision. Correct the review if its premise
  is wrong. A suspected defect is not an executed failing case.
- The constitution and substantive pins own derived behavior. Fix that source
  before projecting changed behavior into chapters. Update an owning JSON or
  generator when appropriate; do not patch only generated output. Maintain
  affected existing coverage records instead of creating parallel inventories.
- Use the author's standing delegated approval for recommended changes and
  prose. Record substantive policy supersessions in their existing controlling
  decisions and summarize them in CLAUDE.md; retain the actual approved text.
  A TODO recommendation alone does not silently supersede constitutional law.
- Preserve universal standing, unconditional floors, the ordinary-life balance,
  the chapter/pin relationship, and the Book 1/Book 2 boundary. Book 1 assigns
  constitutional duties and remedies; Book 2 owns operation and transition.
  Book 2 stays collection-only until Gate C. Preserve the legacy manuscripts.
- Preserve the three exempt prose channels, unnumbered epigraph and method,
  editorial order, and the majority-derived length rule. The existing appendix
  is a carried archive, not an extra channel for new reader-facing arguments.
- **Current design throughout — author instruction, 2026-09-18.** Every
  reader-facing part, including the opening, Part V and optional method, must
  describe the design of the edition being read. Remove accounts of the book's
  own development, earlier rules, repairs, additions and before/after versions,
  even when accurate or instructive. Explain the current rule, its rationale,
  consequences and limits directly. Keep revision history in Git and repository
  decision records outside the reading sequence. This is a continuing authoring
  rule, not a one-time reduction of repetition. Historical evidence about the
  world, explicitly tested alternatives and temporal behavior within the current
  model remain admissible; they must not narrate superseded designs. Preserve
  immutable published editions; "current" means the design bound to that edition.
- Respect the required child test and its documented exemptions. Improve its
  usefulness and brevity without silently deleting its membership requirement.
- Preserve the deliberate flat register. Improve cases through selection,
  sequence and clear roles, without invented dialogue, biography, feelings,
  testimony or successful outside events. Normative argument and historical
  evidence retain their authorised channels; derived chapters remain derived.
- Use focused substantive checks during implementation, then the complete
  `./verify.sh` for item completion, plus relevant existing development checks
  when their machinery changes. Review prose consistency separately. Report
  actual commands, elapsed time, failures, contradictions and incomplete checks.
  Do not introduce receipts, hashes, freshness gates or administrative audits.
- Do not delete substantive regression coverage to obtain a pass. When policy
  intentionally changes, replace superseded expected behavior explicitly while
  retaining the old exploit or counterexample as a regression scenario.
- Commit and push each implemented item after its required checks under the
  standing repository instruction. Preserve unrelated work. Delete completed
  tracker items; the coherent commit and current source retain their result.
- External human review is optional, not a completion dependency. Never invent
  reader testing, expert endorsement, empirical evidence or operational success.
  Do not treat AI agreement as independent validation.

## Resolve before defending — author instruction, 2026-09-18

This rule governs every item and its completion criteria. Exposing, labeling or
explaining a defect is not a substitute for correcting the design.

1. Establish the actual failure and its cause against the current source. Try
   substantive repairs, including a different representation, narrower power,
   replacement mechanism, simplification or removal. Examine nonessential
   design choices rather than treating an inherited decision as an immovable
   constraint. Preserve the constitutional commitments stated above and record
   any policy supersession under the standing delegated approval.
2. Implement and verify a sound resolution when available. Test both the
   legitimate behavior that must remain and the harmful behavior that must stop,
   including downstream interactions. Rewrite the reader-facing account to
   describe the resulting current design; do not narrate the repair history.
3. Defend a remaining limitation only when resolution is shown to be impossible
   within explicitly stated, justified constraints. Record the alternatives
   examined, their results and the necessary constraint preventing resolution in
   the existing repository decision record. A failed encoding, inconvenient
   implementation, elapsed effort, tooling limit or lack of a discovered fix is
   not proof that the constitutional problem cannot be solved. State the scope
   of any impossibility result; do not infer universal impossibility from a
   finite search. Uncertainty or an execution blocker leaves the issue open.
4. An adequate fallback defense must explain why the constraint is necessary,
   why the chosen arrangement is preferable to available alternatives, who bears
   the remaining harm, what safeguards and remedies limit it, and what evidence
   would require reconsideration. Merely calling a cost a trade-off, disclosing
   it honestly, or showing that a pin reproduces it does not meet this standard.
5. If neither resolution nor an adequate defense is established, keep the item
   open and pursue redesign, removal or narrowing of the unjustified mechanism
   or claim. Do not transfer a constitutional defect to Book 2 to close it.

Completion must distinguish a verified repair, a finding disproved by current
evidence, a necessarily constrained and adequately defended limitation, and an
unresolved defect or blocker. Only the first three can close the relevant issue.
Reader-facing prose states present rules and justified limits; the attempted
solutions and development history stay outside the book's reading sequence.

## Ordered revision backlog

Work in this order. Item 32 rebuilds and inspects the reading copies of
the revised manuscript. Each item includes its
own relevant validation under the execution contract; item 33 is a final
integration review, not permission to defer earlier checks. Chapter numbers
refer to the [current reading sequence](book-1/contents.json).

- [ ] **32. Rebuild and inspect the reading copies and sample.**

  **Scope:** Opening navigation, full and sample HTML/EPUB/PDF, book README and
  publisher proposal; use the existing builder and relevant development checks.

  Rebuild the final revised manuscript. Check chapter and section links,
  contents, glossary, case index, footnotes, diagrams and method references.
  Inspect tables, Tamil text, code wrapping, page breaks and narrow-screen
  reading in affected sections and across the full sequence. Validate the EPUB
  and relevant PDF navigation with the established tools. Reassess the sample
  after substantive changes and update measured manuscript/sample lengths and
  proposal claims to match the actual text. Preserve the majority-derived rule.

  **Done when:** the generated copies and selected sample reflect the current
  source, navigation works and material rendering defects are repaired. The
  proposal makes no stronger claim than the manuscript. Mechanical checks are
  reported at their actual scope, without claiming actual-user accessibility
  or initiating publisher contact, submission or release.

- [ ] **33. Reassess the whole book and close only resolved work.**

  **Scope:** All ordered chapters, current constitutional source and pins,
  affected decision records, and the completed reading artifacts.

  Perform a fresh adversarial reading after the revisions. Revisit recognition,
  severity, scarcity, delegated discretion, institutional capture and the
  difference between formal safeguards and a justified design. Check current
  prose against the rules and cases it describes. Assess argument, precision,
  readability, pacing, evidence and navigation separately from formal results.
  Repair material findings and repeat the affected checks; do not close a
  finding because an earlier review or item called it complete.

  Run the complete substantive verifier and relevant existing development
  checks, reporting actual elapsed time, failures, contradiction findings and
  incomplete checks. The five-minute target is a target, not an assumed result;
  no expectation may be dropped or result cached to claim it. Refresh rendered
  artifacts if this review changes their inputs.

  **Done when:** each material finding has a supported disposition under the
  execution and resolve-before-defending rules, all required checks complete,
  and the final review rates the resulting manuscript honestly. Unresolved
  defects remain open with concrete next work. A passing suite, AI agreement
  or an empty checklist does not require a 10/10 rating or declare Gate C.

## Licence

This new planning text is licensed under CC-BY-4.0. See the full
[licence text](book-1/LICENSE-CC-BY) and the repository's
[licensing policy](LICENSING.md).
