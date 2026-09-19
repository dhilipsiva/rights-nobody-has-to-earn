<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1: revision toward an exceptional finished book

Created 2026-09-18 at the author's request: "Create a TODO to make book 10/10."
This is a new revision backlog following the completed 2026-09-16 rebuild.
It does not reopen completed rebuild tasks or declare this edition released.

The starting review covered all 31 numbered chapters, the opening note,
epigraph and method at `da36b0db`: approximately 93,600 words, rated 7.5/10 as
a manuscript and 6/10 for publisher readiness. These are editorial judgments,
not measurements of correctness. The review did not execute the formal pins
or comprehensively fact-check the empirical claims. Its suspected design
defects must be checked against the current source before implementation.

**The objective is a compelling, defensible, precise and readable book.** A
perfect score cannot be guaranteed or made true by closing a checklist. Finish
the work below, then reassess the resulting manuscript without protecting the
starting diagnosis or promising a particular rating.

## Execution contract

- The current request creates the plan. Implementation is subsequent work;
  creating this file does not mark any item implemented or verified.
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

## II. Strengthen evidence and justification

### 10. Defend causal claims and institutional choices against real alternatives

- [ ] Rebuild the reasoning behind the principal choices, especially those
  discussed in chapters 6, 11, 16-20, 24 and Part V.

Correct the inference from income-adjusted democracy/happiness correlations.
Explain what measurement sensitivity establishes and what it leaves uncertain.
For consequential choices such as territorial representation, collective
executive authority, appointment independence and default protections, compare
the strongest plausible alternative on stated criteria. Keep normative reasons
distinct from empirical predictions and from properties of the formal model.
Put non-derived justification in the permitted exempt prose channels.

**Done when:** the reader can identify the reason for each major choice, its
principal cost, a credible alternative and what evidence would weaken the
argument; counterexamples are addressed without changing definitions to win.

### 11. Rebuild Part V around five substantive tests of the design

- [ ] Rewrite `book-1/31-the-five-joints.md` after the substantive and sourcing
  work, retaining its role as the book's argument and evidence section.

For each joint, state the strongest objection, the current mechanism, supporting
evidence, the best alternative, the remaining cost and a justified conclusion.
Remove long inventories already explained in the chapters. Update objections
that target superseded rules. Identify AI critiques as AI-generated feedback;
attribute human criticism only where an actual source supports it. Treat the
kitchen and other imagined scenes as hypothetical, without invented testimony.

**Done when:** all five joints examine the final design; each conclusion follows
from its argument; no success is inferred from AI agreement; and every retained
design failure meets the resolve-before-defending standard. If the argument
reveals a repairable failure, return it to substantive implementation rather
than making its disclosure the conclusion. Present justified residual costs
without repeating the whole constitutional catalogue or the repair history.

### 12. Make the optional method accurate and useful to a skeptical reader

- [ ] Rewrite `book-1/method.md` around the method's actual explanatory value.

Explain the chosen modeling language, supplied premises, derivation, refusal,
contradiction checks and empirical limits using a small number of complete
examples. Integrate item 01's appeal correction. Remove development history,
including accurate accounts of earlier attempts and subsequent fixes. Teach
through the current rules and explicitly tested counterfactuals, not through
the author's route to them. Remove internal labels that explain no claim.
State what verification establishes, what it assumes, what it does not check,
and which failures remain in the current design. Preserve its permitted scope
and optional, unnumbered position.

**Done when:** the worked examples match executed behavior; a reader can tell
model consistency from truth, feasibility and justice; and the method explains
the current design without narrating its development or overclaiming its reach.

## III. Make the book work as a book

### 13. Give the opening a clear promise and a short route into chapter 1

- [ ] Edit the opening note and epigraph, then review the complete reading arc
  against `book-1/contents.json`.

State the central question, intended reader, nature of the proposal and how the
child/prisoner comparison will test it. Give essential terms when needed; make
reference material easy to skip and return to within the permitted structure.
Do not add a new exempt chapter or make the carried appendix the sole support
for a claim. Keep engines before breaks and the balance of provision, ordinary
freedom, democratic choice, coercion and repair. Improve the epigraph's clarity
without inventing authorial experience or sacrificing the source's meaning.

**Done when:** the reader can reach the first substantive case without reading
a manual; the opening's promise matches the finished book; and the epigraph
and closing argument support that promise without redundant explanation.

### 14. Complete a developmental edit of every numbered chapter

- [ ] Read and revise chapters 1-31 in sequence after items 01-13, checking
  each chapter's purpose, progression, concrete cases and ending.

Use these specific priorities; none is a license to omit the remaining chapters:

| Chapters | Editorial job |
|---|---|
| 1-3 | Establish the human test, standing and evidentiary vocabulary without repeating the introduction. |
| 4-7 | Make the path from entitlement through receipt and failure intelligible; retain honest scarcity and initiation limits. |
| 8-12 | Show ordinary freedom and economic life; distinguish money, recognition, competence and equal access. |
| 13-15 | Give ecology, relationships and belonging distinct arguments with clear boundaries and human consequences. |
| 16-20 | Make institutions and public limits navigable; explain how functions connect without reciting records. |
| 21-23 | Give access to justice proportionate attention and make amendment and authorization comprehensible. |
| 24-26 | Present the revised shield, adverse findings and restoration with their reasons and costs. |
| 27-30 | Keep retained rights, placement, confinement and institutional correction distinct and mutually consistent. |
| 31 | Ensure the five tests answer the book just read and end on a supported conclusion. |

**Done when:** every chapter advances the argument, each example earns its
space, important transitions work, and no chapter is merely a catalogue or
change log. Each required child section states the relevant consequence or
limitation concisely; its required fixture and documented exemptions remain.

### 15. Perform a complete line edit for precision, rhythm and economy

- [ ] Edit every ordered input, including the unnumbered material, after the
  developmental changes are stable.

Remove repetitive throat-clearing, legal inventories and redundant
qualifications. Remove all narration of earlier/newer versions, including
"we added," "we fixed," "used to," and "now" when they describe the book's
revision history. Rewrite the passage as a direct account of the current design;
retain its present rationale and limitations, not the story of the change.
Apply this to every reader-facing part, including the opening, Part V and
method, and to each future revision. Keep qualifications that change meaning.
Replace jargon with concrete subjects and verbs within the
established voice and passage rules. Check pronouns, negation, terminology,
sentence rhythm, unnecessary rhetorical certainty and duplicated examples.
Let content determine length; do not cut to an arbitrary percentage or pad to
reach a score. Remeasure the majority-derived requirement after restructuring.

**Done when:** a full sequential reading finds no narration of the book's own
development, no known material ambiguity, avoidable repetition or accidental
voice change; the important distinctions survive the edit; and the book remains
majority-derived by its defined count. Check meaning, not just keywords:
historical evidence and time within the current model are not revision history.

### 16. Repair navigation and check the assembled reading experience

- [ ] Reconcile the manifest, headings, opening contents, cross-references,
  terminology, citations and any existing assembled edition artifacts.

Start with "computed order" in the opening, the contribution reference to
"the next chapter" in chapter 14, and asylum's Chapter 4 reference in chapter
29. Validate destinations as well as link syntax. Check heading hierarchy,
table readability, link labels, footnotes and mechanical accessibility in the
actual formats prepared for submission. Inspect their rendered output; do not
infer that a Markdown check establishes correct PDF or ebook layout.

**Done when:** references reach the intended material; numbering and contents
agree; every prepared format has been inspected for broken navigation, clipping
and reading-order defects; and no claim of actual-user accessibility or reader
comprehension is made without the relevant evidence.

## IV. Prepare a credible open book and reassess it

### 17. Make open contribution practical without blurring authorship or authority

- [ ] Review existing contributor guidance and add or revise a concise
  contribution guide covering prose, evidence and formal changes.

Explain how anyone proposes a correction or co-authored addition, the checks
appropriate to it, attribution, mixed licensing, editorial integration and the
difference between the maintained edition and independent forks. Provide a
small concrete contribution example. Respect existing license grants and make
no promise of universal acceptance or publisher agreement. Document the actual
workflow; do not create a new administrative approval system.

**Done when:** a new contributor can locate the right file, submit a bounded
change and understand credit and review expectations; formal and prose changes
remain consistent; and the invitation to participate is accurate.

### 18. Prepare a concise publisher submission package for the revised book

- [ ] Prepare a synopsis, intended readership, distinct contribution, current
  contents, representative sample selection and accurate completion statement.

Describe the open contribution model, existing licensing and desired print or
editorial partnership accurately. Explain the work's relationship to existing
political and constitutional writing without unsupported novelty claims.
Research current submission requirements when adapting to a specific publisher;
do not infer that open-access publication means acceptance of an open-editing
model. Keep the package consistent with the final manuscript and its limits.

**Done when:** the package is ready for a concrete submission decision, with no
invented endorsement, external review or operational validation. Actual contact,
submission, contracts and publication are separate actions, not completion
conditions for this revision plan.

### 19. Run the final formal and manuscript checks on the revised state

- [ ] Execute the complete substantive verifier and relevant existing
  development checks; perform a separate final prose and source consistency pass.

Use the final current constitution and suites, preserving expected refusals,
counterfactuals, stateful cases and trusted preconditions. Classify known-defect
expectations separately from ordinary successful behavior. A passing suite that
asserts a defect remains evidence of that defect. Report elapsed time instead
of assuming the performance target. Check that all factual citations and the
submission package still describe the final text.

**Done when:** required checks actually complete; failures and incomplete
contradiction checks are resolved; any retained modeled defect satisfies the
resolve-before-defending standard with supporting evidence; and no material
defect is relabeled as a success merely because its pin passes. A described
but unresolved defect, resource limit or tooling blocker cannot pass this item.

### 20. Give the finished manuscript a fresh, evidence-based review

- [ ] Read the final ordered manuscript as a whole and reassess argument,
  coherence, evidence, prose, structure, originality and publication readiness.

Use the strongest objections, not agreement with the politics, as the test.
Distinguish formal results, editorial judgment and optional external feedback.
Name the remaining weaknesses and the strongest chapters with specific examples.
If a material defect remains, add a concrete repair task and complete its
necessary validation before declaring this backlog finished. A fallback defense
is available only under the resolve-before-defending rule, never as a shortcut
around implementation. Do not create cosmetic work or keep revising merely to
force a higher numerical rating.

**Done when:** the substantive problems in this review have supported
dispositions under the resolve-before-defending rule; all chapters meet the
observable standards above; no known material error remains merely disclosed
or rhetorically defended; and the final assessment states the book's actual
quality without guaranteeing "10/10."

## Licence

This new planning text is licensed under CC-BY-4.0. See the full
[licence text](book-1/LICENSE-CC-BY) and the repository's
[licensing policy](LICENSING.md).
