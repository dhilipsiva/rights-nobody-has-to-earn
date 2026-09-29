# TODO: From 8/10 to 9.5+

This file lists the specific, honest improvements that would move the book and
ecosystem from its current state to a 9.5+. Each item states what is wrong now,
what "done" looks like, the specific steps to get there, and whether it needs
engine work.

The current rating is 8/10. The weaknesses are: accessibility (steep learning
curve), length (overloaded chapters), self-reference (single maintainer), UI
onboarding (newcomer lost), and the prose-formal gap (wide). A 9.5+ fixes
these without losing the rigor that makes the project unique.

This document is self-sufficient: you do not need to read the book or the code
to understand what each item asks for. Each item includes the context needed
to evaluate it.

---

## Context: What the project is

The project is a constitutional design expressed in a formal logic engine
(Nibli), projected into a plain-language book (Book 1, 30 chapters, ~98,000
words), with an interactive companion (UI) that runs the constitution on the
reader's device. The book's claims are checked by executable tests (pins):
90,187 pins across 16,353 cases. Each chapter closes with an argument section
that defends its rule against the strongest alternative.

The ecosystem has these components:
- **Book 1** (`book-1/`): 30 chapters, 5 part openers, opening note, epigraph,
  method, reference.
- **Pins** (`book-1/*.pins.nibli`): executable tests that check the book's
  claims against the constitution.
- **Constitution** (`book-1/source/constitution.nibli`): the formal rules.
- **Engine** (Nibli): the logic engine that runs the constitution.
- **UI** (`ui/`): the interactive companion, deployed at
  https://dhilipsiva.dev/rights-nobody-has-to-earn/.
- **Articles** (`ui/articles.json`): 31 plain-language articles restating the
  constitution.
- **Method** (`book-1/method.md`): the technical appendix explaining how the
  rules are run and tested.

The project's strengths are: the formal verification apparatus, the argued
defenses, the honest limits, and the UI's interactivity. Its weaknesses are:
the steep learning curve, the overloaded chapters, the single-maintainer
self-reference, and the gap between prose and formal language.

---

## Item 1: Split the three overloaded chapters

### Current state

Three chapters try to cover too much:

- **Chapter 13** (`book-1/13-creatures-without-a-ballot.md`, 485 lines): covers
  sentience classification, the welfare baseline, serious purposes, the food
  rule, prohibited harm, the core beyond amendment, farming, subsistence, the
  Animal Protection Advocate, and the court between offices. This is at least
  two chapters of material.

- **Chapter 27** (`book-1/27-the-one-thing-taken.md`, 381 lines): covers
  custody authority, renewal, release, physical holding, protective
  restrictions, policing, force, intelligence, and conscientious objection.
  The policing/force/intelligence sections are a separate chapter compressed
  into this one.

- **Chapter 29** (`book-1/29-the-five-joints.md`, 625 lines): tries to
  synthesize the entire book. It covers five commitments, four contributions,
  five joints, and many historical cases (Owen, Lin, MONDRAGON, kibbutzim,
  Cybersyn, Auroville, WIR, Kerala). The historical cases feel rushed.

### Target state

- Chapter 13 becomes two chapters:
  - **Chapter 13a: The animal core.** Sentience classification, the welfare
    baseline, prohibited harm, the core beyond amendment. This is the
    conceptual foundation.
  - **Chapter 13b: Uses and food.** Serious purposes, the food rule, farming,
    subsistence, the Animal Protection Advocate, the court between offices.
    This is the applied design.
  - The split happens at the natural boundary: the core beyond amendment ends
    13a; the food rule and its applications begin 13b.

- Chapter 27 becomes two chapters:
  - **Chapter 27a: Custody and release.** Authority, renewal, one-thing-taken,
    release, physical holding. This is the custody design.
  - **Chapter 27b: Coercive power.** Policing, force, intelligence,
    conscientious objection. This is the coercive power design.
  - The split happens at the natural boundary: release ends 27a; policing and
    force begin 27b.

- Chapter 29 is compressed:
  - The five commitments, four contributions, and five joints stay.
  - The historical cases move to a "History" appendix in the reference or the
    method, with Chapter 29 referencing them briefly.
  - Each historical case gets a short paragraph: what happened, what it shows,
    and what it does not show.
  - Chapter 29 falls from 625 lines to ~350 lines.

### Steps

1. Read Chapters 13, 27, and 29 in full.
2. Identify the split points (the natural boundaries described above).
3. Write the new chapters, preserving the existing prose where it is clear and
   cutting where it is repetitive.
4. Move the pins: each new chapter inherits the pins of its parent chapter.
   The pin files are named after the chapter, so they are renamed accordingly.
5. Update `book-1/contents.json` to reflect the new chapter structure.
6. Update the UI's `chapter-cases.json` to map the new chapters to their
   cases.
7. Update the method's chapter references.
8. Run the full verifier (`./verify.sh`) to confirm all pins still pass.

### Acceptance criteria

- No chapter exceeds 350 lines (except the method and reference, which are
  back matter).
- Each new chapter has its own argument section.
- The pins all pass.
- The UI's chapter map reflects the new structure.
- The book's total word count falls by at least 10%.

### Engine work

No. This is prose restructuring. The pins stay with their chapters; the split
chapters inherit them.

### Dependencies

None. This item can be done independently.

---

## Item 2: Add a UI onboarding path

### Current state

The UI opens with "Pick a person. Walk their forks." A newcomer who has not
read the book does not know:
- What a "fork" is.
- What a "joint" is.
- What the four lenses mean.
- Why they should care.
- Where to start.

The learning curve is steep. The UI assumes the reader has read the book.

### Target state

- A "Start here" panel on the UI home that explains, in under 200 words:
  - What the book is (a constitutional design).
  - What a fork is (a walk through a person's rules).
  - What a joint is (a comparison of the constitution with one declared
    change).
  - What the four lenses do (live a life, break it, run the republic, rewrite
    the joints).
  - Where to start (Nell's first fork).

- A "Read the book first" link that opens the opening note.

- A "Try one fork" button that walks the user through Nell's first fork
  step-by-step, with each step explained:
  - Step 1: "This is Nell. She has one entry: a birth."
  - Step 2: "The rules conclude she is a person."
  - Step 3: "The rules conclude she is owed the floor."
  - Step 4: "The rules conclude nothing else. She has no family, no home, no
    record of food."
  - Step 5: "This is the design's promise: a person with nothing is owed
    everything."

- The dossier entries are grouped by theme:
  - **Limits:** "Capture via silence," "The delivery gap," "Authority ending
    and physical release."
  - **Costs:** "The shield costs time," "Nobody staffs the night shift," "One
    file per door."
  - **Objections:** "It can prove a citizen is starving," "Innocent students
    lose their credentials."

### Steps

1. Read the UI source (`ui/`) to understand the current layout.
2. Design the "Start here" panel (prose and layout).
3. Implement the panel in the UI.
4. Implement the "Try one fork" walkthrough.
5. Group the dossier entries by theme.
6. Test the onboarding with someone who has not read the book.

### Acceptance criteria

- A newcomer can explain what a fork is after reading the "Start here" panel.
- A newcomer can complete Nell's first fork without reading the book.
- The dossier entries are grouped by theme.
- The UI's home page does not overwhelm a newcomer.

### Engine work

No. This is UI prose and layout.

### Dependencies

None. This item can be done independently.

---

## Item 3: Bridge the prose-formal gap

### Current state

The derived chapters state rules in plain language. The pins state the same
rules in Nibli. The gap between "Nell is owed food" and
`owe(State, Eats, Nell)` is wide. A reader who wants to verify a claim must
learn Nibli or trust the prose.

The method explains the syntax, but the gap is wide. The UI shows the results
of forks but not the queries that produced them.

### Target state

- Each derived chapter includes a "Verify this" box that shows the Nibli query
  for each major claim, with a plain-language gloss. Example:

  | Prose | Nibli | Gloss |
  |---|---|---|
  | "Nell is owed food." | `? owe(State, Eats, Nell). # => TRUE` | "The rules conclude that the State owes Nell food." |
  | "Nell is not a prisoner." | `? prisoner(Nell). # => FALSE` | "The rules do not conclude that Nell is a prisoner." |

- The method includes a "Nibli in 10 minutes" tutorial that teaches the syntax
  through the book's own examples:
  - `all $x: ...` means "for every x, if ..."
  - `&` means "and"
  - `->` means "implies"
  - `~` means "not"
  - `?` means "query"
  - `# => TRUE` means "the rules conclude this"
  - `# => FALSE` means "the rules do not conclude this"

- The UI's fork results include a "Show the query" toggle that reveals the
  Nibli behind each conclusion.

### Steps

1. Read the method's existing Nibli explanation.
2. Write the "Nibli in 10 minutes" tutorial.
3. For each derived chapter, identify the major claims and their corresponding
   Nibli queries (from the pins).
4. Add the "Verify this" boxes to the chapters.
5. Implement the "Show the query" toggle in the UI.
6. Test the toggle with a few forks.

### Acceptance criteria

- Every derived chapter has at least one "Verify this" box.
- The "Nibli in 10 minutes" tutorial teaches the syntax in under 1,000 words.
- The UI's "Show the query" toggle works for at least 10 forks.
- A reader can verify a claim without learning Nibli (by reading the gloss).

### Engine work

Possibly. The "Show the query" toggle needs the UI to map prose claims to
their Nibli queries. If the mapping is already in the pins, this is UI work.
If not, the engine may need to expose a query registry (a mapping from prose
claims to Nibli queries).

### Dependencies

None. This item can be done independently.

---

## Item 4: Add independent validation

### Current state

The constitution, engine, tests, and book share a maintainer. The pins check
the book against the constitution, but both are the same author's work. The
book acknowledges this honestly ("the constitution, its tests and the engine
share a maintainer, and their agreement is a check within one project rather
than an independent confirmation").

A 9.5+ needs some form of external check. The project already has:
- A second-engine replay (mentioned in the method).
- An adversarial audit (mentioned in the method).
- Mutation testing (rules deliberately broken to see if tests notice).

But these are not prominent. A 9.5+ makes them visible and adds a second
engine.

### Target state

- A "Validation" section in the method that lists what has been independently
  checked and what has not:
  - **Second engine:** core cases replayed in a second engine through a
    translation. The page shows the translation and the results.
  - **Adversarial audit:** the open findings are listed with their status.
  - **Mutation testing:** the mutations are listed with their results.
  - **Red-team:** a red-team exercise where someone tries to break the design.

- A "Second engine" page that shows:
  - What the second engine is.
  - How the translation works.
  - What cases have been replayed.
  - A table of cases, their verdicts in the first engine, and their verdicts
    in the second engine.
  - An honest statement of what the second engine does and does not establish.

- A "Known defects" page that lists:
  - The adversarial audit's open findings.
  - The book's stated limits.
  - The defects that have been repaired (with what repaired them).

### Steps

1. Read the method's existing second-engine and adversarial audit sections.
2. Design the "Validation" section.
3. Build the second engine (with the user's help).
4. Replay the core cases in the second engine.
5. Write the "Second engine" page.
6. Write the "Known defects" page.
7. Add a red-team exercise (ask someone to try to break the design).

### Acceptance criteria

- The "Validation" section is in the method.
- The second engine exists and replays at least 10 core cases.
- The "Second engine" page shows the results.
- The "Known defects" page lists all known defects.
- The red-team exercise has been done and its results are recorded.

### Engine work

Yes. The second engine is a significant implementation effort. The user has
offered to help with this.

### Dependencies

- Item 10 (Second engine page) depends on this item.

---

## Item 5: Compress the institutional design chapters

### Current state

Chapters 17 (How Public Power Is Built) and 22 (Changing the Rules) are the
least accessible. They assume familiarity with constitutional design.

- **Chapter 17** (`book-1/17-how-public-power-is-built.md`, 396 lines): the
  table of common bodies is clear but dense. The chapter covers tiers, bodies,
  separated functions, appointments, caretaker government, budget deadlock,
  and secession. The secession section feels tacked on.

- **Chapter 22** (`book-1/22-changing-the-rules.md`, 271 lines): the
  exact-change route is technical. The chapter covers the political route,
  certification, publication, the protected core, and the amendment procedure.

### Target state

- Chapter 17 opens with a "Why divide power?" section that explains the
  problem in plain language before introducing the institutions:
  - "A constitution that gives one body all the power will be abused. A
    constitution that gives every body a veto will not act. This chapter
    designs a government that can act and cannot abuse."

- The table of common bodies is supplemented by a "How a bill becomes law"
  diagram (already exists as an SVG in the back matter) that is referenced from
  the chapter.

- Chapter 22 opens with a "Why entrench a core?" section that explains the
  problem before introducing the amendment route:
  - "A majority that can change anything can take everything. A constitution
    that cannot be changed at all cannot correct its mistakes. This chapter
    designs a core that cannot be amended and a route for everything else."

- The exact-change route is illustrated with a worked example:
  - A specific proposal (e.g., "add a new floor item: internet access").
  - Its certification (the exact text is reviewed).
  - Its publication (the certified text is published).
  - Its selection (the published text becomes the version in force).

### Steps

1. Read Chapters 17 and 22 in full.
2. Write the "Why divide power?" section for Chapter 17.
3. Write the "Why entrench a core?" section for Chapter 22.
4. Add the worked example to Chapter 22.
5. Reference the "How a bill becomes law" diagram from Chapter 17.
6. Run the full verifier to confirm all pins still pass.

### Acceptance criteria

- Chapter 17 opens with a plain-language explanation of why power is divided.
- Chapter 22 opens with a plain-language explanation of why the core is
  entrenched.
- The exact-change route has a worked example.
- The "How a bill becomes law" diagram is referenced from Chapter 17.
- The pins all pass.

### Engine work

No. This is prose restructuring.

### Dependencies

None. This item can be done independently.

---

## Item 6: Add a "Book in 10 minutes" summary

### Current state

The book is 98,299 words. A newcomer cannot tell what it says without reading
all of it. The opening note is good but does not summarize the design.

### Target state

- A "The design in 10 minutes" page (or a section in the opening note) that
  states, in under 1,000 words:
  - **The floor:** food, shelter, care, learning, bodily safety, material
    security, expression, belief, company. Owed to every person on personhood
    alone.
  - **The routes into personhood:** birth, first contact, presence, effective
    control, a report that nobody is acting for someone.
  - **The one thing a sentence takes:** movement. Nothing else.
  - **The four emergency powers:** faster procedure, redirected resources,
    requisition with return or compensation, restriction aimed at the named
    hazard.
  - **The protected core:** standing, the floor, equal protection, due process,
    core liberties, the commons, the bans on torture and collective expulsion,
    prompt review of detention, a remedy that works, the capacity of the
    Assembly and the Court to sit, and three animal protections.
  - **The five commitments:** nothing owed waits on merit; a fact reaches a
    person only by a route written for it; no party certifies itself; power is
    current and answerability lasts; silence decides nothing.

- This page links to the chapters that argue each point.
- The UI home includes a "The design in 10 minutes" link.

### Steps

1. Write the "The design in 10 minutes" summary.
2. Add it to the opening note or as a separate page.
3. Link each point to its chapter.
4. Add the link to the UI home.

### Acceptance criteria

- The summary is under 1,000 words.
- Each point links to its chapter.
- The UI home has a link to the summary.
- A newcomer can explain the design in 10 minutes.

### Engine work

No. This is prose.

### Dependencies

None. This item can be done independently.

---

## Item 7: Make the pins accessible to non-programmers

### Current state

The pins are Nibli files with formal queries. A non-programmer cannot read
them. The method explains the syntax, but the gap is wide.

### Target state

- Each pin file includes a plain-language comment above each query, explaining
  what it tests and why. (Some already do this; make it consistent.)

  Example:
  ```nibli
  # "Nell is owed food." This is the first floor debt.
  ? owe(State, Eats, Nell).
  # => TRUE
  ```

- A "Pin tutorial" in the method that walks through Chapter 1's pins,
  explaining each query and its expected verdict:
  - "This query asks: is Nell owed food? The expected verdict is TRUE, because
    the rules conclude that every person is owed food."
  - "This query asks: is Nell a prisoner? The expected verdict is FALSE,
    because the rules do not conclude that Nell is a prisoner."

- The UI's "Show the query" toggle (item 3) includes a plain-language gloss for
  each query.

### Steps

1. Read Chapter 1's pins.
2. Write the "Pin tutorial" in the method.
3. Add plain-language comments to all pin files (consistently).
4. Implement the "Show the query" toggle in the UI (if not done in item 3).

### Acceptance criteria

- Every pin file has plain-language comments above each query.
- The "Pin tutorial" is in the method.
- The UI's "Show the query" toggle includes a plain-language gloss.
- A non-programmer can read a pin file and understand what it tests.

### Engine work

No. This is documentation.

### Dependencies

- Item 3 (Bridge the prose-formal gap) should be done first, as it introduces
  the "Show the query" toggle.

---

## Item 8: Add a "What would change my mind" page

### Current state

Each chapter's argument section ends with "I would reconsider on evidence
that..." These are scattered across 30 chapters. A reader cannot easily see
what evidence would change the design.

### Target state

- A "What would change my mind" page that collects every reconsideration
  condition from every chapter, grouped by theme:
  - **Delivery:** "I would reconsider if witnessed receipts proved easy to
    forge in concert at scale."
  - **Scarcity:** "I would reconsider if the permitted grounds proved to favour
    the same groups a lifespan figure would."
  - **Equality:** "I would reconsider the measures if continuation findings
    kept renewing them while the disadvantage stayed where it was."
  - **Custody:** "I would reconsider on evidence from outside Norway that
    confinement's protective gains vanish or reverse."
  - **Emergency:** "I would reconsider on evidence that emergencies bring
    needs that no hazard-specific, reviewable measure can meet."
  - **Amendment:** "I would reconsider if readings of the core repeatedly
    blocked reforms that kept its protections whole."

- Each condition links to the chapter that states it.
- This page is linked from the opening note and the UI.

### Steps

1. Read all 30 chapters' argument sections.
2. Extract every "I would reconsider" condition.
3. Group them by theme.
4. Write the "What would change my mind" page.
5. Link each condition to its chapter.
6. Add the page to the opening note and the UI.

### Acceptance criteria

- Every "I would reconsider" condition from every chapter is on the page.
- The conditions are grouped by theme.
- Each condition links to its chapter.
- The page is linked from the opening note and the UI.

### Engine work

No. This is prose and cross-referencing.

### Dependencies

None. This item can be done independently.

---

## Item 9: Add a "Costs and who bears them" page

### Current state

Each chapter's argument section states costs and who bears them. These are
scattered. A reader cannot easily see the full distribution of costs.

### Target state

- A "Costs and who bears them" page that collects every cost from every
  chapter, grouped by who bears them:
  - **Claimants:** "A person whose harms all fall below the line is kept from
    a secure place however strong the forecast."
  - **Providers:** "The cost falls on honest providers, who must find a
    witness."
  - **Reviewers:** "Review offices owe answers to weak requests as well as
    strong ones, and whoever funds them pays for both."
  - **Later taxpayers:** "My choice costs later taxpayers, who pay for debt an
    earlier majority chose."
  - **People confined:** "The cost falls on the person held, whose food, care
    and company become the prison's to set."
  - **Animals:** "Owners, farmers, researchers and households lose the power
    to settle an animal's interests."

- Each cost links to the chapter that states it.
- This page is linked from the opening note and the UI.

### Steps

1. Read all 30 chapters' argument sections.
2. Extract every cost statement.
3. Group them by who bears them.
4. Write the "Costs and who bears them" page.
5. Link each cost to its chapter.
6. Add the page to the opening note and the UI.

### Acceptance criteria

- Every cost statement from every chapter is on the page.
- The costs are grouped by who bears them.
- Each cost links to its chapter.
- The page is linked from the opening note and the UI.

### Engine work

No. This is prose and cross-referencing.

### Dependencies

None. This item can be done independently.

---

## Item 10: Add a "Second engine" page

### Current state

The method mentions that core cases have been replayed in a second engine
through a translation. This is buried in the method. A 9.5+ makes it prominent.

### Target state

- A "Second engine" page that explains:
  - What the second engine is (an independent implementation of the
    constitution's rules).
  - How the translation works (the first engine's rules are translated into
    the second engine's language).
  - What cases have been replayed (at least 10 core cases).
  - A table of cases, their verdicts in the first engine, and their verdicts
    in the second engine.
  - An honest statement of what the second engine does and does not establish:
    - "The second engine checks that the rules are consistent across two
      implementations. It does not check that the rules are just, that the
      society operates, or that the design is affordable."

### Steps

1. Build the second engine (with the user's help).
2. Translate the constitution's rules into the second engine's language.
3. Replay at least 10 core cases.
4. Write the "Second engine" page.
5. Add the page to the method and the UI.

### Acceptance criteria

- The second engine exists.
- At least 10 core cases have been replayed.
- The "Second engine" page shows the results.
- The page states honestly what the second engine does and does not establish.

### Engine work

Yes. The second engine is a significant implementation effort. The user has
offered to help with this.

### Dependencies

- Item 4 (Add independent validation) should be done first, as it includes
  the second engine.

---

## Item 11: Add a "Known defects" page

### Current state

The adversarial audit's open findings are listed in the method. The book's
stated limits are scattered across chapters. A 9.5+ collects them.

### Target state

- A "Known defects" page that lists:
  - **Adversarial audit's open findings:** the findings from the adversarial
    audit that are still open.
  - **The book's stated limits:** the "What this cannot settle" sections from
    each chapter, collected.
  - **Defects that have been repaired:** the defects that were found and
    repaired, with what repaired them.

- Each defect links to the chapter or method section that discusses it.
- The page states honestly what is known to be wrong and what is not.

### Steps

1. Read the method's adversarial audit section.
2. Read all 30 chapters' "What this cannot settle" sections.
3. Collect all defects.
4. Write the "Known defects" page.
5. Link each defect to its chapter or method section.
6. Add the page to the method and the UI.

### Acceptance criteria

- All adversarial audit open findings are on the page.
- All "What this cannot settle" sections are on the page.
- All repaired defects are on the page, with what repaired them.
- Each defect links to its chapter or method section.
- The page is linked from the method and the UI.

### Engine work

No. This is documentation.

### Dependencies

None. This item can be done independently.

---

## Item 12: Add a "How to contribute" page

### Current state

The book is a single-maintainer project. A 9.5+ makes it easier for others to
contribute.

### Target state

- A "How to contribute" page that explains:
  - **How to run the pins:** `./verify.sh` (all pins), `./verify.sh --only
    <pin-file>` (focused).
  - **How to propose a change to the constitution:** edit the source, run the
    pins, confirm they pass.
  - **How to propose a change to the prose:** edit the chapter, run the prose
    lint, confirm it passes.
  - **How to report a defect:** open an issue on the repository.
  - **How to build a second engine:** see the "Second engine" page.
  - **The project's standards:** pins must pass, prose must be jargon-free,
    arguments must engage the strongest alternative.

- The page links to the method, the pins, and the UI.

### Steps

1. Write the "How to contribute" page.
2. Add it to the method and the UI.

### Acceptance criteria

- The "How to contribute" page explains all the ways to contribute.
- The page links to the method, the pins, and the UI.
- The page states the project's standards.

### Engine work

No. This is documentation.

### Dependencies

- Item 10 (Second engine page) should be done first, as it explains how to
  build a second engine.

---

## Item 13: Compress the historical cases in Chapter 29

### Current state

Chapter 29 includes many historical cases (Owen, Lin, MONDRAGON, kibbutzim,
Cybersyn, Auroville, WIR, Kerala). These are well-chosen but feel rushed. A
9.5+ gives them room to breathe.

### Target state

- The historical cases are moved to a "History" appendix in the reference or
  the method, with Chapter 29 referencing them briefly.
- Each historical case gets a short paragraph:
  - **What happened:** a brief account.
  - **What it shows:** the lesson for the design.
  - **What it does not show:** the limits of the lesson.
- Chapter 29 is compressed to the five commitments, four contributions, and
  five joints, with the historical cases as illustrations rather than
  arguments.

### Steps

1. Read Chapter 29 in full.
2. Identify the historical cases.
3. Write the "History" appendix.
4. Compress Chapter 29.
5. Run the full verifier to confirm all pins still pass.

### Acceptance criteria

- The historical cases are in the "History" appendix.
- Each historical case has a short paragraph (what happened, what it shows,
  what it does not show).
- Chapter 29 is compressed to ~350 lines.
- The pins all pass.

### Engine work

No. This is prose restructuring.

### Dependencies

- Item 1 (Split the three overloaded chapters) should be done first, as it
  also compresses Chapter 29.

---

## Item 14: Add a "Book map" to the UI

### Current state

The UI has a "Read the book" link that opens the reader. A newcomer cannot
easily see the book's structure.

### Target state

- A "Book map" page in the UI that shows:
  - The five parts.
  - The 30 chapters (or 32 after the split in item 1).
  - The five part openers.
  - The back matter (method, bibliography, reference).
  - A one-line summary of each chapter.
- Each chapter links to its reader page.
- The book map is linked from the UI home.

### Steps

1. Read `book-1/contents.json`.
2. Design the "Book map" page.
3. Implement the page in the UI.
4. Link each chapter to its reader page.
5. Add the page to the UI home.

### Acceptance criteria

- The "Book map" page shows the book's structure.
- Each chapter links to its reader page.
- The page is linked from the UI home.

### Engine work

No. This is UI layout.

### Dependencies

- Item 1 (Split the three overloaded chapters) should be done first, as it
  changes the chapter structure.

---

## Item 15: Add a "Fork explained" page

### Current state

The UI uses "forks" and "joints" without explaining them. A newcomer does not
know what they are.

### Target state

- A "Forks and joints" page in the UI that explains:
  - **What a fork is:** a walk through a person's rules. Example: "Nell's
    first fork asks: what follows from a birth entry? The rules conclude she
    is a person, owed the floor, and nothing else."
  - **What a joint is:** a comparison of the constitution with one declared
    change. Example: "The 'Delete one rule' joint asks: what happens if the
    birth rule is deleted? The rules conclude nothing for Nell."
  - **How to run them:** click a fork or joint, watch the rules execute, read
    the results.
- The page includes a worked example (Nell's first fork, the "Delete one rule"
  joint).
- This page is linked from the UI home.

### Steps

1. Read the UI source to understand how forks and joints work.
2. Write the "Forks and joints" page.
3. Add the page to the UI home.

### Acceptance criteria

- The "Forks and joints" page explains forks and joints.
- The page includes a worked example.
- The page is linked from the UI home.

### Engine work

No. This is UI documentation.

### Dependencies

None. This item can be done independently.

---

## Summary

### Items that need engine work

- **Item 3** (possibly): the "Show the query" toggle needs a query registry.
- **Item 4:** a second engine implementation.
- **Item 10:** the second engine page.

### Items that are prose, documentation, or UI

- Items 1, 2, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15.

### Dependencies

- Item 10 depends on Item 4.
- Item 12 depends on Item 10.
- Item 14 depends on Item 1.
- Item 13 depends on Item 1.
- Item 7 depends on Item 3.

### Realistic timeline

- **Phase 1 (UI and documentation, 1-2 weeks):** Items 2, 6, 7, 14, 15.
- **Phase 2 (prose restructuring, 2-4 weeks):** Items 1, 5, 13.
- **Phase 3 (cross-referencing and documentation, 1-2 weeks):** Items 3, 8, 9,
  11, 12.
- **Phase 4 (second engine, 4-8 weeks):** Items 4, 10.

A 9.5+ is achievable. The hardest part is the second engine (items 4 and 10).
The rest is honest work that the project's existing standards already imply.

### How to use this file

1. Read the "Context" section to understand the project.
2. Read each item's "Current state" and "Target state" to understand what is
   being asked.
3. Pick an item and do it.
4. Run the full verifier (`./verify.sh`) after each item to confirm nothing
   broke.
5. Check off the item when its acceptance criteria are met.
