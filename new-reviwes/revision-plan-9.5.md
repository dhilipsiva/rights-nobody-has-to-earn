# From 6 to 9.5+: Revision plan for *The Rights Nobody Has to Earn* (Book 1)

*Based on the review-copy PDF (170 pp., generated 24 Sep 2026). Metrics come from a text extraction of that PDF and are approximate. Scores are my judgement. "Review N" refers to the seven reviews in our conversation, numbered in the order you pasted them (the inline review is 7).*

**What 9.5+ means here:** the best book in its genre that a serious reader could pick up. Specialists in each field it touches find it situated, correct and new. A non-specialist finishes it and can explain its argument.

Two honest caveats before the work:

1. **This is a rewrite, not an edit.** Most of the chapter prose will change, parts of the constitution will change, and new research is needed.
2. **No checklist guarantees a score.** This plan removes every reason I found for marking the book down. The last half-point comes from things only other people can supply: independent expert review, independent reproduction, and readers who are not you, me or another AI.

---

## Contents

1. [The bar](#1-the-bar)
2. [The twelve moves that matter most](#2-the-twelve-moves-that-matter-most)
3. [Baseline](#3-baseline)
4. [Architecture: decide what the book is](#4-architecture-decide-what-the-book-is)
5. [Put the constitution in the book](#5-put-the-constitution-in-the-book)
6. [Prose: from 3 to 9](#6-prose-from-3-to-9)
7. [Design fixes to the constitution itself](#7-design-fixes-to-the-constitution-itself)
8. [Argument: from 7 to 9.5](#8-argument-from-7-to-95)
9. [Evidence and lineage: from 5 to 9.5](#9-evidence-and-lineage-from-5-to-95)
10. [The formal layer: from drag to asset](#10-the-formal-layer-from-drag-to-asset)
11. [Honesty and disclosure: from 9 to 9.5](#11-honesty-and-disclosure-from-9-to-95)
12. [External validation](#12-external-validation)
13. [Production and accessibility](#13-production-and-accessibility)
14. [Chapter-by-chapter punch list](#14-chapter-by-chapter-punch-list)
15. [Sequencing, risks and the definition of done](#15-sequencing-risks-and-the-definition-of-done)
16. [Appendix: baseline metrics per chapter](#appendix-baseline-metrics-per-chapter)

---

## 1. The bar

| Dimension | Now | Target | What gets it there |
|---|---|---|---|
| Ideas and mechanisms | 8 | 9.5 | Name the contributions; fix the design flaws; show the design is light on ordinary life (§7, §8, §10) |
| Intellectual honesty | 9 | 9.5 | Keep it; make limits specific and ranked; disclose the verification stack (§8.4, §11) |
| Argument | 7 | 9.5 | Reasons at the point of use; argument proportional to stakes; opponents agree they have been stated fairly (§8) |
| Evidence and scholarship | 5 | 9.5 | The right reference class — constitutional case law and the rights literature — at the point of use (§9) |
| Readability for the stated audience | 3 | 9 | Constitution in the book; test narration out; negation, jargon and names cut (§4–§6) |
| Independent validation | none | required | Human expert and lay readers; independent reproduction of the formal core (§12) |

---

## 2. The twelve moves that matter most

Ranked by impact.

1. **Separate the book from its test suite.** The main text is an argument for people; the formal casebook lives in the companion. (§4)
2. **Put the constitution in the book** as plain-language numbered articles, traceable to the rules. (§5)
3. **Rebuild every chapter on one template:** commitment → reason → strongest alternative and evidence → case → cost → the one new limitation. (§4.3)
4. **State the global limits once, then cut the hedging.** Negation runs at about four times the rate of academic prose. (§6)
5. **Replace terms of art and test-harness names**, and keep a small, stable, neutrally named cast. (§6)
6. **Bring in the right evidence**: constitutional case law on enforceable floors, entrenchment, collective executives and resident voting. Credit the intellectual lineage. (§9)
7. **Fix the design issues**: the shield's free option, the floor's bearer, the floor's terms, systemic non-performance, procedural load. (§7)
8. **Argue the biggest commitments in proportion to their stakes**: the animal food rule and core, the entrenched commons, the unamendable core. (§8.2)
9. **Name the formal contribution** — constraints on how facts may flow into consequences — and situate Nibli. (§10)
10. **Rank the limitations** in a new closing chapter: where this could fail, and what would show it. (§8.4)
11. **Open with the stakes**: documented cases at the front of the book and of each Part. (§8.5)
12. **Get independent human review and independent reproduction** before calling it done. (§12)

---

## 3. Baseline

| Measure | Now | Target | Note |
|---|---|---|---|
| Pages before Chapter 1 | 15 | ≤ 5 | Map, glossary and indexes move to the back |
| Negations per 1,000 words (chapters) | 26–58; book-wide 33 | ≤ 20 in every chapter; aim for 15 | Brown corpus baseline: 10 overall, 7.7 in "learned" prose, 5.5 in government prose |
| Sentences saying something is not established | 127 (about one per page) | ≤ 30, each specific to its chapter | |
| Forms of "establish" | 294 | ≤ 60 | |
| Limitation sections ("What none of this proves…", etc.) | 22 | 0 as headings | At most one "What this can't settle" paragraph per chapter |
| "The child with nobody" sections | 25 | ≤ 8 | Keep only where Nell's case differs |
| Case and fixture names | 59 case names + ~15 fixtures | ≤ 15 recurring; ≤ 5 per chapter | Chapter 25 alone has 21 |
| Test-harness names in prose | ~20 | 0 | Targ4, Nogra, Nogrb, Partnr, Sock, Probe, Amend_Sneak… |
| Mentions of "recognition" | 41 | Only in the chapter that rejects it | A status the design does not have |
| Terms of art per 1,000 words (lint proxy: establish, derive, supplied, qualified, reader, effective, carry, lease, window) | book-wide 13; up to 29 (Ch 26) | ≤ 5 in every chapter | The same list the lint script uses |
| Part V | 8.5k words (~17% of the main text), arriving on p. 136 | Arguments distributed; Part V becomes synthesis | |

**Sentence length is not the problem.** Chapters read at Flesch–Kincaid grade 8–14 with 12–17-word sentences, which is normal for serious non-fiction. Don't chase readability formulas. The difficulty comes from negation, abstraction, terms of art, the number of names, and a structure that delivers reasons last.

---
## 4. Architecture: decide what the book is

The book is currently two books fighting: a constitutional argument for people and a verification report for machines. At present the report narrates the argument. Everything else in this plan depends on separating them.

### 4.1 Decisions to make first

- [ ] **Primary reader.** Recommendation: a serious non-specialist — the reader of Sen's *Development as Freedom* or Sandel's *Justice* — with lawyers and policymakers as the secondary audience. The book's own opening already promises this reader ("You need no knowledge of formal logic"). If you choose specialists instead, the readability bar shifts, but every other section still applies.
- [ ] **Reader promise**, one sentence, used on the first page and the back cover. Draft: *"A constitution designed from the person with nothing, argued in plain language, with every rule published so you can test it."*
- [ ] **What lives where:**

| Layer | Contains | For |
|---|---|---|
| Main text | Opening, five Parts of argued chapters, closing | Everyone |
| Appendix A | The constitution as numbered articles | Everyone; lawyers especially |
| Appendix B | How the checks work (≤ 3k words) | Curious readers |
| Companion (web and repo) | Full casebook, pins, counterfactuals, runnable examples, defect list | Verifiers |

- [ ] **Design-fix decisions** (§7): shield, floor bearer, floor list, systemic remedy, procedural asymmetry, signing-restriction scope, the food rule and animal core. **Freeze these before rewriting prose**, or you'll rewrite chapters twice.

### 4.2 Proposed structure

A proposal, not a prescription. The merges and splits follow from the metrics (appendix) and from reader load.

**Opening** (≤ 5 pp.)
- A documented case.
- The question and the thesis.
- Nell and the prisoner.
- The three distinctions stated once: record vs world, entitlement vs delivery, conclusion vs event.
- What the book claims and what it doesn't.
- How to read it.

**Part I — Who counts, and what they are owed**
1. The Child With Nobody *(current 1 + 2)*
2. What the Record May Say *(3)*
3. What You Are Owed *(4)*
4. Whether It Arrived *(5)*
5. When There Is Genuinely Not Enough *(6)*
6. Who Owes, and What Follows *(7, with the new systemic remedy)*

**Part II — The life the design leaves alone**
7. An Ordinary Week *(new: the design seen from a typical resident; §8.6)*
8. What Nobody Has to Ask Permission For *(8)*
9. Work, Pay and Contribution *(9 + 10)*
10. What Money Cannot Buy *(11)*
11. The Same Route for Everyone *(12)*
12. A Place in Which Life Remains Possible *(13: commons and future conditions)*
13. Creatures Without a Ballot *(13: animals, expanded — working title)*
14. Holding a Role in Somebody's Life *(14)*
15. Arriving and Belonging *(15)*

**Part III — The public power that serves it**
16. Answerability and Authority *(16)*
17. How Public Power Is Built *(17)*
18. The Vote Conviction Does Not Take *(18)*
19. What May Be Kept About You *(19)*
20. A Crisis Does Not Suspend the Republic *(20)*
21. A Way to Be Heard *(21)*
22. Changing the Rules *(22)*

**Part IV — What the design does to a person, and how it catches itself**
23. The Shield *(24)*
24. Findings About People *(23 + 25 + 26)*
25. A Prisoner Is a Person *(27)*
26. Where People Are Put *(28)*
27. The One Thing Taken *(29)*
28. When the System Notices It Broke *(30)*

**Part V — The argument**
29. The Five Joints *(31, rewritten as synthesis)*
30. Where This Could Fail *(new; §8.4)*

**Back matter:** Appendix A (the constitution) · Appendix B (how the checks work) · notes · bibliography · glossary · index.

**Length targets:** main text 45–55k words (about the same as now; the composition changes), Appendix A 10–15k, Appendix B ≤ 3k.

### 4.3 The chapter template

Every chapter in Parts I–IV follows the same shape:

1. **The situation.** A case or a documented event, in one paragraph.
2. **The commitment.** The article or articles, quoted briefly, by number.
3. **Why.** The reason, moved forward from Part V.
4. **The strongest alternative.** Named, with who holds it and the best evidence for it.
5. **How it works.** One to three cases, with a comparison table wherever premises vary (the tables on pp. 30 and 122 are the model).
6. **What it costs,** and who bears the cost.
7. **What this chapter can't settle.** Only the limitation that is new here. Review 5's test: *"What is the new limitation in this chapter?"*
8. **Run it.** One line pointing to the companion cases.

**Definition of done for a chapter:**
- It passes the prose lint (§6.3).
- It uses ≤ 5 case names.
- It cites at least one real-world precedent and one source for the strongest alternative.
- A lay reader can state the commitment and its reason after one read.

---

## 5. Put the constitution in the book

A reader of the current book never sees the text of a single article. Reviews 6 and mine made this point independently; it's the top fix.

- [ ] Draft the constitution as numbered plain-language articles (Appendix A), organised by Part. Mark the protected-core articles visibly.
- [ ] Add an **interpretation article**: where a standard is uncertain ("adequate", "reasonably available", "least restrictive"…), the reading more protective of the floor and of liberty prevails, and the burden of justification lies on the public actor. Precedents for interpretation clauses include section 39 of South Africa's Constitution and the *pro persona* principle in Article 1 of Mexico's Constitution (2011 reform).
- [ ] Give every article an ID, and annotate every Nibli rule with the article(s) it implements.
- [ ] Build a **traceability check**:
  - every article has at least one rule and one pin;
  - every rule belongs to an article;
  - gaps are reported in Appendix B.
- [ ] Mark checkable claims in chapter prose with pin IDs. CI fails if a marker doesn't resolve to a passing pin.
- [ ] Generate Appendix A's skeleton from the annotated rules, then edit by hand. Don't write it as a second free-hand paraphrase.
- [ ] Compare each article with existing constitutions using the Constitute Project. Record lineage (what's borrowed) and novelty (what's new); this feeds §9.
- [ ] Commission a sample conformance review: a lawyer reads 20 articles against their rules and pins and reports mismatches.

**Done when:** no rule lacks an article, and no article lacks a test.

---

## 6. Prose: from 3 to 9

### 6.1 State the limits once

- [ ] Write one page in the Opening stating the global limits:
  - the rules read records, not the world;
  - a conclusion doesn't prove its inputs;
  - an entitlement isn't a delivery;
  - operation belongs to Book 2.
- [ ] Delete every restatement of those general limits from the chapters. Keep only chapter-specific limits: a false witness, an absent record, a stopped clock, an unperformed remedy. These are different problems (Review 5's distinction); name the one that is new.
- [ ] Remove the 22 limitation headings. Allow at most one "What this can't settle" paragraph per chapter.
- [ ] Cut the 25 "The child with nobody" sections to ≤ 8. Keep those where Nell's case produces a different or instructive result, e.g. scarcity keys, the appointment route, arrival.

### 6.2 Terms of art

| Now | Replace with | Why |
|---|---|---|
| derive / derives / derived (87) | follows; the rules conclude | Formal register |
| supplied (112) | recorded; given | |
| qualified (49), as in "qualified finding" | complete; properly made (define once) | Reads as "hedged" in ordinary English |
| effective (63), as in "effective finding" | in force | "Effective" also means "works", as in "effective remedy" |
| reader (35), for an institution | the responding office; the office that must act | Collides with the book's actual reader |
| voiding / void (55) | signing restriction (Review 5: "restriction on adverse-finding authority") | "Void" invites a far broader reading than the rule has |
| pen; credential | authority to sign findings | |
| carry; carried history | move to the companion | |
| lease (custody) | custody authorisation and its review date | |
| window | period | |
| selected current record | the version in force | |
| recognition; recognition loss (41) | only in the chapter that rejects recognition | Repeated denials of a status the reader was never offered |
| dwelling debt; security debt; health debt; material security | the floor's canonical names (§7.3) | Look like rule names leaking into prose |

### 6.3 Prose lint (automate it)

- [ ] Add a CI check over chapter prose that flags:
  - the replaced terms above;
  - harness names (§6.4);
  - "pin", "fixture", "counterfactual", "stratif\*", "the model", "the checks" (allowed only in "Run it" lines and Appendix B);
  - negation density above 20 per 1,000 words in any chapter;
  - terms of art above 5 per 1,000 words;
  - more than five distinct case names in a chapter;
  - more than two "does not establish"-type sentences in a chapter.
- [ ] Start from the included `prose_lint.py`; edit its lists as decisions are made.

### 6.4 Names

- [ ] **Retire harness names** and replace them with descriptions ("a second examiner", "a secure facility"): Targ4, Targo, Nogra, Nogrb, Partnr, Frisk, Sock, Probe, Vote_Probe, Amend_Sneak, Amend_Decoy, Amend_Floor, Ambi, Solo, Purga, Hunch, SchemeM, PublicGuarantee, HighSec, Homestay.
- [ ] **Institutional fixtures** (Provender, Ledgerwitness, Ledgerhouse, Foundry, Steward, Assay, Assayer, Harrow): use descriptions such as "the food provider" or "an independent witness" unless a name recurs.
- [ ] **Neutralise names that prejudge.** "Sly" implies cunning in a case that leaves the disclosure's truth open (Review 5). "Boss" and "Rebel" do the same to the official and the discloser. Check Vex, Hex, Rex, Lupo and Don too.
- [ ] **Choose a stable cast** of ≤ 15 recurring people whose facts never change between chapters. Suggested anchors:
  - Nell, the child with nobody;
  - Hano, the prisoner;
  - one discloser and one official;
  - Bela, a teacher with a finding against her, and Cira, her learner;
  - Marisol, for delivery;
  - the two scarcity claimants, named;
  - Ruk, for secure placement.
- [ ] **When a test changes the facts,** say so in plain words ("Suppose instead…") or use a different person.

### 6.5 Voice

- [ ] Lead with what is true, then what isn't. Prefer "Nell is owed food; nobody has yet shown that food arrived" to paired negations.
- [ ] Keep the sentence lengths; vary the rhythm.
- [ ] Let the principles stand as short paragraphs. The book's best lines are its aphorisms — "Nobody acquires a person through their need, and nobody acquires an office through the failure to replace them" — and they're currently buried.
- [ ] Use comparison tables wherever premises vary.
- [ ] Rewrite the densest chapters from scratch rather than editing them: 26, 6, 23, 25 and 24 have the most terms of art (21–29 per 1,000 words).

---
## 7. Design fixes to the constitution itself

For each fix: decide, change `constitution.nibli`, add pins that demonstrate the change, and only then write the prose.

### 7.1 The shield's free option

**Problem.** Any defendant can disclose against the prosecuting court (Sly's case in Chapter 24). That blocks confinement under a conviction until three things happen:
- two independent deciders find the prosecution unrelated;
- a qualifier and a separate qualification reviewer positively establish those deciders' eligibility;
- none of them is conflicted.

A deceit finding costs the defendant almost nothing. The shield for that disclosure falls, and a signing restriction bites on a power they don't hold. If defendants act on that incentive, the shield becomes a universal extra stage in custodial cases rather than a protection for whistleblowers.

**Options:**
- **A.** An automatic shield for disclosures made before the prosecution began, or lodged with an independent recipient before charge.
- **B.** For disclosures made after charge: provisional protection, plus an expedited unrelatedness review that is pre-staffed and has a predeclared alternate.
- **C.** A specificity threshold, requiring the disclosure to identify an act of authority. This is risky, because an implicated authority would judge specificity.
- **D.** Accept the equilibrium and rename it as a universal custodial review stage; then cost and staff it.

**Recommendation:** A + B, and cost it either way.

- [ ] Add pins for:
  - a post-charge disclosure against the prosecuting court;
  - a pre-charge disclosure;
  - an expedited review completed in time;
  - both the reviewer and the alternate unavailable.

### 7.2 The floor's bearer

**Problem.** "The public body named State owes each floor item to every person" fails the book's own Chapter 7 standard, which requires every duty to have "a bearer, a function, a jurisdiction and a scope".

- [ ] Assign the floor by tier. One example split:
  - common tier: finance, equalisation, portability and minimum standards;
  - regions: provision;
  - localities: outreach and delivery;
  - on certified failure, continuity passes up a tier.
- [ ] Replace `owe(State, …)` with tiered duties. Add pins for each tier and for the failover.

### 7.3 The floor's list and names

- [ ] **Decide what "safety" means.** The prose also uses "security debt" and "material security". Physical safety and material security are different rights, and they are delivered differently.
- [ ] **Decide whether health sits inside "care" or is its own item.** The prose uses "health debt" and "health entitlement" four times.
- [ ] **Pick one of "shelter" and "dwelling"** and use it everywhere.
- [ ] **Separate what the State delivers from what it must respect or enable.** Delivered provisions: food, shelter, care or health, learning, and safety or security. Guarantees: expression, belief, company.
- [ ] **Reframe "company" as human contact.** It means:
  - no imposed isolation, anchored to the Nelson Mandela Rules on solitary confinement;
  - a duty to enable contact for people in custody or institutional care;
  - no receipt-and-witness delivery route outside those settings.

### 7.4 Systemic non-performance

**Problem.** The duty chain ends with "duties still owed". Courts may not take over administration. If the State under-delivers at scale, nothing compels the next act. Every constitution with an enforceable floor meets this problem; this one should say how it answers it.

**Options** (choose one or more):
- **A.** A declared unconstitutional state of affairs that triggers a public plan with deadlines and monitored follow-up (Colombia, T-025 of 2004).
- **B.** Supervisory orders with independent monitoring commissioners (India's right-to-food case, *PUCL v Union of India*).
- **C.** Floor-first appropriation: floor finance has first claim on the budget, and under-appropriation triggers expedited review.
- **D.** A mandatory, reasoned response from the Assembly to a certified pattern of shortfalls.

- [ ] Encode the chosen route. Add pins for the sequence: persistent shortfall pattern → declaration → duty to produce a plan → monitoring.

### 7.5 Procedural load: fast for help, slow for harm

**Problem.** A single accommodation needs:
- separate source, evidence and review attesters;
- a challenge reader;
- an independent alternate.

An appointment for a person with nobody needs three attesters who agree. The design's safe default — no authority without complete positive evidence — protects against coercion, but it also slows help.

- [ ] Classify every derived conclusion as either **beneficial** (it gives, preserves or releases) or **adverse** (it restricts, takes, confines or finds against).
- [ ] Generate a **signature budget** table from the source: the distinct roles required before each conclusion takes effect. Publish it in Appendix B.
- [ ] Set budgets. Beneficial acts take effect on one authorised actor, with review afterwards. Adverse acts keep full prior procedure.
- [ ] Remove duplicate steps. Part V already says it: "More signatures have no value in themselves."

### 7.6 Other fixes

- [ ] **Scope of the signing restriction (voiding).** State its reach plainly: it restricts a power that only credentialed examiners hold. Then either widen its purpose (for example, to any office that makes adverse findings, such as inspectors and investigators) or shrink it to one section.
- [ ] **Confinement for lacking a home record.** Chapter 28's prose forbids harsher confinement on grounds of family, home or poverty. Yet an isolated experiment shows that a rule confining someone for lacking a home record loads. Add a flow check that forbids reading absent home, family or status entries into confinement or placement.
- [ ] **Who writes the scarcity evidence.** The manager writes the comparison it is then reviewed on. Require independent attestation of the comparative evidence, plus a duty to consider counter-evidence from claimants.
- [ ] **The cost of custody lapsing.** When custody authority lapses through administrative failure, release is owed. State the public-safety cost in the argument. Add a pre-staffed priority lane for authorities about to lapse, with no automatic extension.
- [ ] **The food-use rule and the animal core.** See §8.2.

### 7.7 Stress tests

Write each as a worked case in the text and as a pin set in the companion.

- [ ] Strategic disclosure after charge.
- [ ] Mass non-delivery: a region under-provides food for a year.
- [ ] Captured appointments: two nominally separate selectors controlled by one coalition.
- [ ] Custody authority lapses through backlog for a person convicted of grave injury.
- [ ] An emergency makes it physically impossible to hold an election on schedule.
- [ ] Scarcity with manipulated urgency evidence.
- [ ] An agency and its contractor blame each other in plural provision.
- [ ] A rights-preserving amendment blocked by an expansive reading of the core.
- [ ] A cultural practice against the animal core (§8.2).
- [ ] A person never recorded (Ori) is reached through outreach.

---

## 8. Argument: from 7 to 9.5

### 8.1 Move the reasons to their mechanisms

- [ ] For each argument in Part V, move its first statement into the chapter where the mechanism appears: the reason, the strongest alternative, the cost, and what would change the choice. This is Review 5's structure.
- [ ] Rewrite Part V as synthesis: how the choices fit together, where they trade off against each other, and the commitments underneath them.
- [ ] State the book's contributions explicitly, in the Opening and at the close:
  1. separating entitlement, duty, evidence and delivery as a principle of constitutional design;
  2. constraints on how facts may flow into consequences (§10.1);
  3. testing a design with paired edge cases (the child with nobody, the prisoner);
  4. specific mechanisms:
     - the one thing taken;
     - answerability that survives recall;
     - severity as a ceiling, not a selector;
     - false-scarcity controls;
     - silence never counting as approval;
     - the firewall between services and enforcement.
- [ ] Defend the founding method against its obvious objection: that designing around the extreme case imposes its procedural weight on ordinary life. The new chapter "An Ordinary Week" is the answer (§8.6).

### 8.2 Argue in proportion to the stakes

The requirement for 9.5 is that each of these choices be explicit and argued. It is not that they go any particular way.

**The animal food rule and the animal core**
- [ ] State plainly what the food rule implies for farming, household production and subsistence. The rule covers "every controlled use to produce food … including nonlethal production" and requires that no reasonably available, materially less-harmful alternative exist.
- [ ] Decide, and say, whether the food rule sits inside the unamendable core. The answer turns on "direct protection", "avoidable" and "dispensable"; define all three.
- [ ] Argue against the strongest alternatives:
  - welfare regimes (TFEU Art 13; Basic Law Art 20a; UK Animal Welfare (Sentience) Act 2022);
  - political theories of animal rights (Donaldson & Kymlicka, *Zoopolis*; Nussbaum, *Justice for Animals*; Korsgaard, *Fellow Creatures*);
  - humane-farming positions.
- [ ] Work one case of culture against the core. Jallikattu is the obvious test of the book's position that cultural and Indigenous rights get "least restrictive accommodation" but "do not make avoidable severe harm harmless": see *Animal Welfare Board of India v A. Nagaraja* (2014), and the 2023 Constitution Bench decision upholding Tamil Nadu's amendment.
- [ ] Spell out the transition duties: to the human food floor, to workers, and to communities.

**The entrenched commons**
- [ ] Give the commons their own argument. The argument that "a person who loses a political contest must remain a member" covers the commons only as far as survival depends on them, but the protected commons reach further: biodiversity, connectivity, the inheritance of nonrenewable resources. The strongest available argument is intertemporal freedom, from the German Constitutional Court's *Neubauer* decision (2021).
- [ ] Confront the power it gives interpreters with evidence:
  - *Kesavananda Bharati* (1973) and India's basic-structure doctrine;
  - Article 79(3) of Germany's Basic Law;
  - Colombia's substitution doctrine;
  - Roznai, *Unconstitutional Constitutional Amendments* (2017);
  - Landau, "Abusive Constitutionalism" (2013).

**The unamendable core in general**
- [ ] Engage Waldron's case against judicial review by name.
- [ ] Say what stops interpreters from expanding the core: the interpretation article, public reasons, challenge, and the amendment route for institutional machinery.

**The other big institutional bets**
- [ ] **Residence-based franchise.** Engage the debate over whether the vote belongs to everyone affected or everyone subject to public power (Goodin 2007; López-Guerra 2014). Cite precedents: New Zealand and Chile, and EU citizens voting in local elections.
- [ ] **Collective executive and equal-weight regional chamber.** Switzerland (Federal Council; Council of States) is the precedent; Uruguay's colegiado (1952–67) is the cautionary case.
- [ ] **Retained confinement.** Engage abolition by name (Davis 2003; Gilmore 2007) and use the incarceration evidence in §9.

### 8.3 The opponent test

- [ ] For each contested choice, find a reader who holds the opposing view. Ask them to confirm their position is stated fairly, and revise until they agree.

### 8.4 Rank the limitations: a new chapter, "Where This Could Fail"

A draft ranking, to be revised as evidence comes in:

1. The State fails to perform at scale.
2. Open-textured standards are interpreted narrowly: "adequate", "usable", "reasonably available", "least restrictive", "meaningful exit", "real", "genuinely".
3. Independent review is captured.
4. Procedure is too slow and too costly.
5. Inputs lack integrity: records are omitted or forged, or people are never recorded.
6. The floor isn't fiscally feasible (Book 2 territory).

For each: what evidence would show it failing, what the design does about it now, and what a fix would take.

### 8.5 Put the stakes up front

- [ ] Open the book with a documented case: Santoshi Kumari, currently on p. 144, stated with the dispute over the cause of death intact.
- [ ] Open each Part with a documented case, keeping the book's rule against inventing inner lives. Candidates:
  - Part I: Santoshi Kumari;
  - Part II: the Dutch childcare-benefits scandal, or Robodebt;
  - Part III: *ADM Jabalpur* (1976) and the Emergency;
  - Part IV: *Hussainara Khatoon* (1979), on undertrials held longer than their possible sentences.
- [ ] Don't add composite or fictional inner lives to the test cases, as Review 3 suggested. Documented cases supply the stakes without breaking the book's own evidentiary rule.

### 8.6 Two new walkthroughs

- [ ] **"An Ordinary Week"** (new chapter). An adult resident works, rents, visits a clinic, has a child in school, votes, has a dispute with a landlord and is stopped by the police. Show which articles touch them, how lightly, and where the machinery stays invisible. This answers the objection to the founding method.
- [ ] **"A claim that fails."** Follow one claim from first contact through assistance, provision, evidence and dispute, to a failed response by both the ordinary reviewer and the alternate, and on to remedy (Review 5's suggestion). Label all timings and staffing as illustrative.

---
## 9. Evidence and lineage: from 5 to 9.5

**Principles:**
- Sources go at the point of use, and each one does work: it supports, challenges, or supplies the strongest alternative to a specific mechanism.
- No literature-review chapter.
- Notes run throughout the book, not only in Part V.

> **Verify every citation below before use.** They are from memory, not checked against the sources. Have the comparative constitutional lawyer in §12 review the case-law list.

### 9.1 Lineage to credit (Opening and Part V)

| Idea in the book | Credit |
|---|---|
| Unconditional subsistence as a basic right | Shue, *Basic Rights* (1980) |
| Respect / protect / fulfil | Eide's reports for the UN on the right to food; CESCR General Comment 3 (1990, the minimum core) and General Comment 12 (1999, food) |
| Entitlement ≠ delivery; a budget choice "is not an authorised substitute for physical scarcity" | Sen, *Poverty and Famines* (1981); Drèze & Sen, *Hunger and Public Action* (1989) |
| A floor "that neither private dependence nor public punishment may withdraw" | Pettit, *Republicanism* (1997) and *On the People's Terms* (2012), on non-domination |
| "a person who loses a political contest must remain a member" | Anderson, "What Is the Point of Equality?" (1999); Rawls, *Political Liberalism* (1993), on the social minimum as a constitutional essential |
| Private power as government | Anderson, *Private Government* (2017); Rahman, "The New Utilities" (2018) |
| "The human floor and the ecological ceiling" | Raworth, *Doughnut Economics* (2017); Rockström et al., "A safe operating space for humanity" (2009) |
| Governing commons | Ostrom, *Governing the Commons* (1990) |
| Purpose-limited records | Nissenbaum on contextual integrity (2004; *Privacy in Context*, 2010); GDPR Art 5(1)(b) |
| Replacement before reduction and refinement | Russell & Burch, the 3Rs (1959) |
| Support before substituted decisions | CRPD Art 12 and General Comment 1 (2014) |
| No minimum speaking age | UN Convention on the Rights of the Child, Art 12 |
| Belonging without surrendering rights | Kymlicka, *Multicultural Citizenship* (1995), on internal restrictions vs external protections; Shachar, *Multicultural Jurisdictions* (2001) |
| Firewall between services and enforcement | Crépeau & Hastie, "The Case for 'Firewall' Protections for Irregular Migrants" (2015) |
| Exit only by negotiated settlement | *Reference re Secession of Quebec* (1998) |

### 9.2 Evidence by chapter

| Chapter (current no.) | Precedent or support | Strongest alternative to engage |
|---|---|---|
| 1–3 Standing and the record | SDG 16.9 (legal identity for all); the 1954 and 1961 statelessness conventions; the Aadhaar studies already cited | Identity-first welfare, justified by reducing leakage |
| 4–5 Floor and delivery | CESCR GC 4 (adequacy of housing), GC 12, GC 14 (availability, accessibility, acceptability, quality); MGNREGA social audits and MKSS public hearings as recipient-side verification in practice | Administrative or biometric verification of delivery |
| 6 Scarcity | Sen (1981); disability-rights challenges to COVID-19 triage protocols (e.g., the US HHS Office for Civil Rights resolution with Alabama, 2020) | Persad, Wertheimer & Emanuel (2009) and Emanuel et al. (2020), which include life-years — a key the book forbids; Elster, *Solomonic Judgements* (1989), on lotteries |
| 7 Who owes, and remedy | *Grootboom* (2000), *TAC* (2002), *Mazibuko* (2009); *PUCL v Union of India* (2001 onward); Colombia T-025 (2004); Sabel & Simon, "Destabilization Rights" (2004); Tushnet, *Weak Courts, Strong Rights* (2008); Bilchitz, *Poverty and Fundamental Rights* (2007), defending the minimum core; Young, *Constituting Economic and Social Rights* (2012) | Reasonableness review (*Grootboom*, *Mazibuko*); legislative primacy (Waldron) |
| 8 Freedoms | Siracusa Principles (1984); Barak, *Proportionality* (2012) | The categorical approach of the US First Amendment |
| 9–10 Work and contribution | ILO forced-labour conventions (C29, C105); the ILO Committee on Freedom of Association on minimum service; ILO Recommendation 198 (2006) on the employment relationship; *Uber BV v Aslam* (2021); UK evidence on benefit sanctions (National Audit Office 2016; the Welfare Conditionality project 2018); the German Constitutional Court's rulings of 2010 (subsistence minimum) and 2019 (sanctions) | Reciprocity: White, *The Civic Minimum* (2003). Basic income: Van Parijs & Vanderborght (2017) and Finland's 2017–18 experiment |
| 11 Money, property, private power | EU Digital Markets Act (2022) on gatekeepers; Mondragon (already cited) | Germany's debt brake (2009) and the Constitutional Court's 2023 budget ruling: the live example of the rule this book prohibits |
| 12 Equality | CRPD on reasonable accommodation; *Griggs v Duke Power* (1971); Crenshaw (1989) on intersectionality; *Indra Sawhney* (1992) on limits to reservations | *SFFA v Harvard* (2023) |
| 13 Commons, future, environment | *Urgenda* (2019); *Neubauer* (2021); Wales's Well-being of Future Generations Act (2015) and its Commissioner, a comparator for the Future Conditions Guardian; Hungary's ombudsman for future generations; for the river case, Te Awa Tupua (Whanganui River) Act 2017 and Colombia's Atrato River ruling T-622 of 2016 | Discounted cost–benefit analysis (the Stern Review of 2007 against Nordhaus); weak sustainability |
| 13 Animals | TFEU Art 13; Basic Law Art 20a; the UK Animal Welfare (Sentience) Act 2022, which covers the same taxa the book presumes sentient, and the Birch et al. review (2021) behind it; EU Directive 2010/63/EU on research animals; *Nagaraja* (2014) | Welfarism; humane farming; cultural practice (the 2023 jallikattu decision) |
| 14 Roles | CRPD Art 12 and General Comment 1; UNCRC Art 12 | The best-interests model (Mental Capacity Act 2005, England and Wales) |
| 15 Arrival and belonging | Refugee Convention Art 33; UNDRIP Art 32; ILO Convention 169; *Saramaka v Suriname* (2007); *Tsilhqot'in* (2014); *Niyamgiri* (2013); Carens, *The Ethics of Immigration* (2013) | Wellman, "Immigration and Freedom of Association" (2008) |
| 16 and 24 Answerability and the shield | EU Whistleblower Directive 2019/1937, whose Art 21 reverses the burden of proof in retaliation cases | Case-specific protective orders (Part V's own alternative) |
| 17 Public power | Switzerland's Federal Council and Council of States; Uruguay's colegiado; Lijphart, *Patterns of Democracy* (2012); Tsebelis & Money, *Bicameralism* (1997); the Quebec Reference; India's NJAC judgment (2015) on judicial appointments | Westminster cabinet government; a second chamber apportioned by population |
| 18 The vote | Resident voting in New Zealand and Chile; EU citizens in local elections; *Hirst v UK* (2005); *Sauvé* (2002); *August* (1999) and *NICRO* (2004) | Citizenship with easy naturalisation; Goodin (2007) on enfranchising all affected interests |
| 19 Records | The SyRI judgment (The Hague, 2020); the Dutch childcare-benefits scandal; the Robodebt Royal Commission (2023); *Puttaswamy* (2017, 2018); Eubanks, *Automating Inequality* (2018); GDPR Art 22 | Integrated data for better targeting ("tell us once") |
| 20 Crisis | ICCPR Art 4 (move it here from Part V); *ADM Jabalpur* (1976) and the 44th Amendment (1978); Article 48 of the Weimar constitution | Ackerman's supermajoritarian escalator, "The Emergency Constitution" (2004); Gross & Ní Aoláin, *Law in Times of Crisis* (2006) |
| 21 Being heard | Strang et al., the Campbell review of restorative justice conferencing (2013) | Prosecution-centred models |
| 22 Amendment | *Kesavananda Bharati* (1973); Basic Law Art 79(3); Colombia C-141 of 2010; Roznai (2017); Landau (2013); Elkins, Ginsburg & Melton, *The Endurance of National Constitutions* (2009) | Fully revisable constitutions; Waldron |
| 27–29 Custody | The Nelson Mandela Rules (2015); Chin, "The New Civil Death" (2012); the Uniform Collateral Consequences of Conviction Act; Pratt on Nordic penal exceptionalism (2008); Bhuller et al., "Incarceration, Recidivism, and Employment" (2020); *Hussainara Khatoon* (1979) | Abolition: Davis, *Are Prisons Obsolete?* (2003); Gilmore, *Golden Gulag* (2007) |
| 30 Detecting breaches | National preventive mechanisms under OPCAT; the Paris Principles for national human rights institutions | Internal inspection |
| Method | Sergot et al., "The British Nationality Act as a Logic Program" (1986); Catala (2021); OECD, *Cracking the Code* (2020); Apt, Blair & Walker (1988) on stratified negation; Denning (1976) and Myers & Liskov (1997) on information-flow control | Critiques of computational law, e.g., Hildebrandt, *Law for Computer Scientists and Other Folk* (2020) |

- [ ] **Replace the evidence in the "Coercion" joint.** Villagisation is about forced relocation and doesn't bear on criminal confinement. Use the custody evidence above.
- [ ] **Re-anchor Part V's communes and cooperatives.** They are evidence for specific choices (valuation, rotation). Add republic-level evidence for the rest.

---

## 10. The formal layer: from drag to asset

### 10.1 Name the contribution

- [ ] Write a short section, in Appendix B plus about two pages in the Opening or Part V, naming the constraint types the book already uses:

| Constraint | In the book |
|---|---|
| Closed inputs | Only admitted kinds of entry; "rich", "dangerous" and "vulnerable" are refused |
| Unforgeable conclusions | Some conclusions can only follow from premises, never be written directly: prisoner, public answerability, severity, audit markers |
| Purpose-bound reads | Contribution records only support supplements; pay promises only support compensation; records only serve their stated purpose |
| Endpoints | A loss (such as movement) can't be read into another consequence; delivery conclusions feed nothing |
| No punishment from absence | A missing floor condition can't ground confinement |
| Scope binding | A finding binds its subject, case and incident and can't be borrowed for another |

- [ ] Relate these to information-flow control, taint analysis and contextual integrity.
- [ ] Say plainly what they are: checks on written rule forms that can miss semantically equivalent attacks.
- [ ] Frame the thesis in one line: *a right is protected not only by stating it, but by limiting what the rest of the law may do with facts about the person.*

### 10.2 Demote what doesn't carry weight

- [ ] Move the stratification refusals out of chapter prose and into Appendix B and the companion. That covers the hostile floor rule and the prisoner-to-person counterfactuals.
- [ ] Describe these refusals accurately. They come from the constitution's structure (entitlements written as events, plus the rule that a prisoner is a person) meeting the engine's stratification. They are designed, not accidental, but they are specific to how a rule is written: the home-record variant loads.

### 10.3 Situate Nibli

- [ ] Say who builds and maintains Nibli.
- [ ] Relate it to Datalog with stratified negation, and to earlier law-as-code work: Sergot et al. (1986), Catala, and the Rules as Code movement.
- [ ] State what is new here.

### 10.4 Measure the tests

- [ ] Publish counts of rules, articles and pins, with test coverage by article.
- [ ] Run mutation testing on the rules (mutate conditions and count how many mutants the pins catch). Publish the score.
- [ ] Publish the `:defect` list in the book, not only in the repository.

### 10.5 Independent reproduction and a red team

- [ ] Reimplement a core subset — standing, floor, delivery, custody, shield — in a mainstream engine such as Soufflé or clingo, ideally by someone else. Compare verdicts on shared fixtures and publish the differences.
- [ ] Run a public red-team challenge: *write a rule that harms Nell or a prisoner and still passes the suite.* Publish what was found and what changed.
- [ ] Consider a peer-reviewed paper on the flow constraints, for example at JURIX, ICAIL, or a programming-languages or security workshop. It buys independent expert review of the formal layer.

### 10.6 The companion

- [ ] Map every chapter's "Run it" line to runnable cases in the browser companion.
- [ ] Put everything that moves out of the prose in the companion: counterfactuals, harness cases, carry mechanics, byte-level amendment checks.

---

## 11. Honesty and disclosure: from 9 to 9.5

- [ ] Make the AI-assistance statement specific: which parts (drafting, editing, code, tests), and how the outputs were checked.
- [ ] Disclose, in the Opening as well as the Method, that the constitution, the pins and (if so) the engine share a maintainer.
- [ ] Replace the distributed disclaimers with the ranked-limitations chapter (§8.4) and the published defect list (§10.4).
- [ ] Keep the book's best habit: every contested empirical claim is reported with its dispute intact.

---

## 12. External validation

This is the part no AI can supply.

| Reader | What you need from them |
|---|---|
| Comparative constitutional lawyer | Lineage, case law, entrenchment, the drafting of Appendix A |
| Scholar of economic and social rights | The floor, remedies, systemic non-performance |
| Formal-methods or logic-programming specialist | Appendix B, the flow constraints, tests, reproduction |
| Animal law or ethics scholar | The food rule and the animal core |
| Frontline practitioner (welfare rights, public defence, prison monitoring) | Realism, stakes, procedural load |
| Three serious non-specialist readers | Finish rate, a confusion log, recall |

- [ ] Ask lay readers:
  - Where did you stop?
  - What is the book's claim, in one sentence?
  - Explain five of its mechanisms back to me.
  - Mark every sentence you had to reread.
- [ ] Ask the experts what's wrong, what's missing and what's new. Separate "rejects the premise" from "found a defect".
- [ ] Recruit reviewers early; their calendars are the longest dependency.
- [ ] Don't count more AI reviews, including mine, as validation.

**Done when:**
- all three lay readers finish;
- each states the thesis and explains at least five core mechanisms;
- each expert's major objections are addressed, or argued against in the text;
- the independent reproduction agrees on the core subset.

---

## 13. Production and accessibility

- [ ] Remove web artefacts:
  - "Continue to Chapter 1, or return to the contents.";
  - the "↩︎" markers in the notes;
  - link phrasing that doesn't work in print ("the choices below lead directly to them");
  - the duplicated "Begin with Chapter 1".
- [ ] Keep front matter to ≤ 5 pages before Chapter 1; move the map, glossary, roles list and subject index to the back.
- [ ] Draw real diagrams, each with its prose equivalent: the duty chain, delivery evidence, the placement ceiling, the democratic corridor, the flow constraints.
- [ ] Add a full bibliography and an index.
- [ ] Make cross-references numeric ("the chapter on roles" becomes "Chapter 14").
- [ ] Publish an accessible plain-language summary edition. The constitution requires plain language of public bodies; the book should meet its own standard.
- [ ] Optional: a subtitle that states the genre, e.g. *A constitution designed from the person with nothing*.

---
## 14. Chapter-by-chapter punch list

Per-chapter metrics are in the appendix. Priority: **P1** = rewrite first, highest reader cost; **P2** = substantial revision; **P3** = targeted edits.

**Part I**

- **Ch 1 The Child With Nobody (P1).**
  - Keep it as the opening case and merge Chapter 2 into it.
  - Halve its negation density, the highest in the book (58 per 1,000 words).
  - Keep Ori: the gap between a person and their record in one example.
  - Move the "delete the birth rule" counterfactual to the companion.
- **Ch 2 Who Counts (P3).** Merge into Chapter 1 (654 words). Keep the four ways a person enters the record, and the temporary encounter name as a handle, not an identity.
- **Ch 3 What Counts as Evidence (P1).**
  - Keep it as the method explained in plain words.
  - Cut 14 case names to ≤ 4.
  - Move the vocabulary-change and carry mechanics to the companion.
- **Ch 4 What You Are Owed (P2).**
  - Fix the floor's list and names (§7.3) and its bearer (§7.2).
  - Compare each item's definition with CESCR General Comments 4, 12 and 14.
- **Ch 5 Whether It Arrived (P1).**
  - Make this the showcase for evidence from the recipient's side; add social audits as the real-world precedent.
  - Keep the claims table on p. 30.
  - Move the custody material to Part IV.
  - Halve its negation density (50 per 1,000 words).
- **Ch 6 When There Is Genuinely Not Enough (P2).**
  - Add Sen, the allocation literature and the disability-triage cases.
  - Fix who writes the comparative evidence (§7.6).
  - Keep the one-unit table.
- **Ch 7 Who Owes, and What Follows (P2).**
  - Add the systemic remedy (§7.4) and the tiered bearer.
  - Credit the respect/protect/fulfil lineage.

**Part II**

- **New: An Ordinary Week (P1).** See §8.6.
- **Ch 8 What Nobody Has to Ask Permission For (P3).** Keep it short; add the proportionality lineage.
- **Ch 9 + 10 Work, Pay and Contribution (P1).**
  - Merge the two chapters.
  - Cut the 14 and 15 case and fixture names in these chapters to ≤ 5 in the merged chapter. They include Ansel, Coll, Marlo, Foundry, Steward, Assay, Ledgerhouse, SchemeM, PublicGuarantee, Brix and Dunya.
  - Introduce and reject "recognition" here, once, with Part V's reasoning.
  - Engage White's reciprocity argument.
- **Ch 11 What Money Cannot Buy (P2).**
  - Engage the German debt brake.
  - Credit Anderson and Rahman.
  - Use the Digital Markets Act for gatekeeping.
- **Ch 12 The Same Route for Everyone (P3).**
  - Credit the CRPD, *Griggs* and Crenshaw.
  - Engage *SFFA*.
  - Use India's reservations debate on time-bound measures.
- **Ch 13 A Place in Which Life Remains Possible (P1).** Split it: at 2,817 words and Flesch–Kincaid grade 14.0, it is among the hardest chapters.
  - The commons chapter credits Ostrom and Raworth and uses *Neubauer* and the Wales Commissioner.
  - The river case gets the Whanganui and Atrato precedents.
  - The animals chapter follows §8.2.
- **Ch 14 Holding a Role in Somebody's Life (P3).** Credit CRPD Article 12; engage the best-interests model.
- **Ch 15 Arriving and Belonging (P2).**
  - Credit the firewall literature and Kymlicka.
  - Add the free, prior and informed consent precedents, including *Niyamgiri*.

**Part III**

- **Ch 16 Public Answerability (P2).**
  - Give Boss and Rebel neutral names.
  - Keep the recall insight up front; it already opens the chapter and is one of the book's best.
  - Move the counterfactual that removes Boss's answerability route to the companion.
- **Ch 17 How Public Power Is Built (P2).**
  - Add the Swiss and Uruguayan comparisons and the Quebec Reference.
  - Add a diagram of the bodies.
- **Ch 18 The Vote Conviction Does Not Take (P1).**
  - Halve its negation density (50 per 1,000 words).
  - Add precedents on resident voting and prisoner voting.
  - Move the formal material on why an added rule doesn't repeal the existing one to the companion.
- **Ch 19 What May Be Kept About You (P2).** This should become one of the strongest chapters: SyRI, the Dutch childcare-benefits scandal, Robodebt, *Puttaswamy*.
- **Ch 20 A Crisis Does Not Suspend the Republic (P2).**
  - Add *ADM Jabalpur* and the 44th Amendment.
  - Engage Ackerman.
  - Move ICCPR Article 4 here from Part V.
- **Ch 21 A Way to Be Heard (P3).** Keep the Nia/Ruk contrast; add the restorative-justice evidence.
- **Ch 22 Changing the Rules (P2).**
  - Add the basic-structure and eternity-clause evidence.
  - Decide the scope of the animal core.
  - Move the source-editing tests and the Amend_Sneak, Amend_Decoy and Amend_Floor proposal names to the companion.

**Part IV**

- **Ch 23 Who Holds the Pen (P1).**
  - Fold it into "Findings About People".
  - Move the mechanics to the companion; it is among the densest chapters in terms of art (24 per 1,000 words).
- **Ch 24 The Shield (P1).**
  - Fix the incentive (§7.1).
  - Keep the distinction that a disclosure against a private person (Don against Pax) opens no shield.
  - Use neutral names.
- **Ch 25 + 26 Voiding / The Limits of a Finding (P1).**
  - Merge them with Chapter 23 and rename the mechanism.
  - Cut the 21 names in Chapter 25 to ≤ 4.
  - State the restriction's reach in the first paragraph.
- **Ch 27 A Prisoner Is a Person (P2).**
  - Keep the title and the first line.
  - Rebuild the chapter on the principle, with evidence on civil death, prisoner voting and the Mandela Rules.
  - Move the formal demonstrations to the companion.
- **Ch 28 Where People Are Put (P1).**
  - Keep the severity table on p. 122.
  - Cut 13 names to ≤ 5.
  - Add the check against confinement for a missing home record (§7.6).
- **Ch 29 The One Thing Taken (P2).**
  - Add evidence on collateral consequences and incarceration.
  - State the cost when custody authority lapses.
  - Keep the absolute prohibitions.
- **Ch 30 When the System Notices It Broke (P3).** Add OPCAT and national human rights institutions as comparators.

**Part V and the Method**

- **Ch 31 The Five Joints (P1).** It becomes synthesis (§8.1). The ranked limitations get a new chapter (§8.4).
- **The Method (P2).** It becomes Appendix B (§10).

---

## 15. Sequencing, risks and the definition of done

### 15.1 Phases

| Phase | Work | Size | Depends on |
|---|---|---|---|
| 0 | Decisions: reader, promise, architecture, design fixes | S | — |
| 1 | Constitution articles, traceability, vocabulary, lint; design fixes in the source and pins | L | 0 |
| 2 | Research and citation verification (§9) | L | 0; runs in parallel with 1 |
| 3 | Rewrite Parts I–IV on the template, P1 chapters first | XL | 1, 2 |
| 4 | Part V as synthesis; ranked limits; the Opening; both walkthroughs | L | 2, 3 |
| 5 | Appendix B, flow constraints, mutation testing, reproduction, red team | M–L | 1 |
| 6 | Human review and revision | M, but a long calendar | 3, 4, 5 |
| 7 | Production and accessibility | M | 6 |

Recruit reviewers during phase 1.

### 15.2 Risks

| Risk | Mitigation |
|---|---|
| Scope explodes | Freeze design decisions before rewriting prose; keep research at the point of use |
| Precision is lost in the rewrite | Conformance checks (§5); one reviewer focused on precision |
| Design changes cascade through the tests | Change the pins first, the prose last |
| Feedback on contested commitments polarises | Separate "disagrees with the premise" from "found a defect" |
| The book loses its identity | The goal is *precise and readable*, not readable instead of precise. The honesty is the brand; make it specific, not rarer |

### 15.3 What not to do

- Don't add disclaimers. Remove them.
- Don't shorten sentences; they're already short. Fix negation, jargon, names and structure.
- Don't give test cases composite or invented inner lives. Use documented cases.
- Don't add a separate objections chapter. Move each objection to its mechanism.
- Don't add new domains. Go deeper on fewer.
- Don't let Appendix A become a second paraphrase. Generate it from annotated rules, then edit.
- Don't count more AI reviews as validation.

### 15.4 Definition of done (9.5+)

- [ ] ≤ 5 pages before Chapter 1, and the Opening starts with a documented case.
- [ ] Every chapter follows the template and passes the prose lint.
- [ ] Appendix A is complete, and traceability holds in both directions.
- [ ] The design fixes have landed with pins: shield, bearer, floor list, systemic remedy, procedural asymmetry, the home-record check, scarcity evidence.
- [ ] The decision on the animal food rule and core is explicit and argued.
- [ ] Every mechanism has a precedent and a source for the strongest alternative, and every citation is verified.
- [ ] "Where This Could Fail" ranks the limitations, each with what would falsify it.
- [ ] Appendix B names the flow constraints, situates Nibli, and publishes coverage, the mutation score and the defect list.
- [ ] The independent reproduction of the core subset agrees, and the red-team results are published.
- [ ] Three lay readers finish and pass the recall test; expert objections are addressed; the opponent tests are passed.
- [ ] No web artefacts; diagrams, bibliography and index are in place; the accessible summary edition is published.

---

## Appendix: baseline metrics per chapter

Bold marks the highest-cost values: negations ≥ 45, terms of art ≥ 20, case names ≥ 10.

| Ch | Title | Pages | Words | FK grade | Negations /1k | Terms of art /1k | Case names |
|---|---|---|---|---|---|---|---|
| 1 | The Child With Nobody | 16-18 | 1,138 | 8.4 | **58.0** | 8.8 | 2 |
| 2 | Who Counts | 19-20 | 654 | 9.7 | 41.3 | 15.3 | 1 |
| 3 | What Counts as Evidence | 21-25 | 1,804 | 11.1 | 37.7 | 14.4 | **14** |
| 4 | What You Are Owed | 26-28 | 918 | 10.7 | 34.9 | 7.6 | 2 |
| 5 | Whether It Arrived | 29-31 | 1,120 | 10.8 | **50.0** | 17.0 | 7 |
| 6 | When There Is Genuinely Not Enough | 32-35 | 1,320 | 12.7 | 32.6 | **24.2** | 1 |
| 7 | Who Owes, and What Follows | 36-41 | 1,915 | 12.3 | 30.3 | 14.1 | 1 |
| 8 | What Nobody Has to Ask Permission For | 42-43 | 748 | 11.2 | 42.8 | 16.0 | 1 |
| 9 | Earning Above the Floor | 44-47 | 1,427 | 11.4 | 42.7 | 18.9 | **14** |
| 10 | Contribution | 48-50 | 833 | 10.2 | 42.0 | 19.2 | **15** |
| 11 | What Money Cannot Buy | 51-55 | 1,650 | 12.8 | 39.4 | 6.1 | 2 |
| 12 | The Same Route for Everyone | 56-59 | 1,354 | 13.8 | 33.2 | 13.3 | 3 |
| 13 | A Place in Which Life Remains Possible | 60-67 | 2,817 | 14.0 | 36.2 | 9.2 | 1 |
| 14 | Holding a Role in Somebody's Life | 68-71 | 1,449 | 12.2 | 40.0 | 11.7 | 3 |
| 15 | Arriving and Belonging | 72-75 | 1,265 | 14.1 | 40.3 | 9.5 | 1 |
| 16 | Public Answerability, and Why It Is Never Revoked | 76-78 | 1,070 | 11.5 | 42.1 | 15.9 | 8 |
| 17 | How Public Power Is Built | 79-84 | 2,121 | 13.6 | 33.0 | 10.4 | 1 |
| 18 | The Vote Conviction Does Not Take | 85-88 | 1,152 | 12.2 | **50.3** | 11.3 | 4 |
| 19 | What May Be Kept About You | 89-92 | 1,338 | 12.0 | 28.4 | 5.2 | 2 |
| 20 | A Crisis Does Not Suspend the Republic | 93-96 | 1,174 | 13.7 | 35.8 | 6.0 | 1 |
| 21 | A Way to Be Heard | 97-99 | 965 | 11.7 | 36.3 | 13.5 | 3 |
| 22 | Changing the Rules | 100-103 | 1,194 | 13.1 | 32.7 | 15.9 | 5 |
| 23 | Who Holds the Pen | 104-106 | 1,145 | 11.3 | 31.4 | **23.6** | 9 |
| 24 | The Shield | 107-110 | 1,467 | 10.9 | 37.5 | **21.1** | 6 |
| 25 | Voiding | 111-114 | 1,392 | 10.5 | 35.9 | **22.3** | **21** |
| 26 | The Limits of a Finding | 115-117 | 804 | 10.2 | 37.3 | **28.6** | **10** |
| 27 | A Prisoner Is a Person | 118-120 | 997 | 10.4 | 34.1 | 17.1 | 3 |
| 28 | Where People Are Put | 121-124 | 1,291 | 12.8 | **45.7** | 17.8 | **13** |
| 29 | The One Thing Taken | 125-130 | 2,084 | 13.0 | 42.2 | 13.9 | 7 |
| 30 | When the System Notices It Broke | 131-135 | 1,873 | 12.0 | 33.6 | 17.1 | 5 |
| 31 | The Five Joints | 136-154 | 8,542 | 12.9 | 25.9 | 8.5 | 1 |
| Method | The Method | 158-170 | 3,963 | 10.7 | 23.2 | 15.6 | — |

**How these were measured.** Text extracted with `pdftotext` from the review-copy PDF, by page range (Chapter 31 excludes the notes on pp. 155–157), with page numbers and running Part headers removed. Flesch–Kincaid grade via `textstat`. Negations: *not, no, neither, nor, cannot, never, none* (the Brown corpus baseline also counts *n't*). Terms of art: the lint script's list (*establish\*, derive\*, supplied, qualif\*, reader, effective, window, lease, carry/carried*). Case names: 59 case identifiers plus fixtures and harness names. These are proxies for reader load, not goals in themselves; re-run `prose_lint.py` on each revised chapter to track progress.
