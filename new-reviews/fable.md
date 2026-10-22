I'll start by reading the uploaded files.All 14 pin files reconcile exactly with their declared `:expect-pins` (458 pins total). Now checking whether the prose sentences the pins say they guard still exist in the manuscript:The full review is in the file. The headline findings:

**Mechanical verification passed.** All 14 pin files reconcile exactly with their `:expect-pins` (458 pins; convention = queries + refuse + accept + require). Nine `:defect` pins remain open, all disclosed in the prose. I couldn't run `verify.sh`, so nothing claims a pin passes — the checks below are about what the pins *prove*, not whether they're green.

**Three real gaps in what the pins prove:**
- **Ch 5 kinship controls are confounded.** The Sirefa/Scionfa (parent) and Mara/Maro (married) blocks omit `authorized(…, IndependentReviewer, …)`, which the Una/Uno control and the closing exhibit rule show is a conjunct. So both FALSE verdicts fail on two conjuncts and isolate nothing — the exact "FALSE for the wrong reason" the pin preamble warns against. The comment "the marriage is the only failing conjunct" is currently untrue.
- **`broken(Vex)` FALSE is never pinned.** Every Vex exhibit (answerable / not believable / powerless) depends on the pen being withheld by the carried void alone. A stray recall entry would keep it green for the wrong reason.
- **Delivery receipts, the action-duty chain, and `public(witness)` have no query pins in the upload** — and those are exactly the passages written in spec-register rather than the book's voice. The correlation is the finding.

**One internal contradiction.** Ch 12 line 3047 says voiding needs signers "both with their credibility intact." Ch 5 pins the opposite: Lupo, already voided, holds a pen and his co-signature sticks.

**Specific weak terms, as you asked:** "There is nowhere to put it" (Ch 1 — true of relation names, false of the free constant slots: `cite(Koa, Untrustworthy, Esa)` loads and persists, inert); "attributable" (Ch 5, for `authorized/3`, which names no finder); "the mechanism that broke first" (Ch 15 kibbutz — your reading, not Abramitzky's); and the constant `Points` in `lose(Points, X)`, which a hostile reader of the machine part will quote against Ch 10.

**Ratings (of 10):** Ch 2 and Ch 4 at 9; Ch 6, 7, 10, 11 at 8.5; Ch 1, 3, 9, 12, 13 at 8; Ch 5 and 14 at 7.5; Ch 15 at 7; Ch 8 at 6.5 (argument 9, second half is ~1,600 words of unpinned spec). Manuscript overall about 8.

Where I still disagree with the book rather than just flagging it: Ch 15 names Meta, DeepSeek, Gemini and Grok as reviewers without saying they're AI systems, and cites one calling a chapter "excellent" in a section that promised a knife. That should be said plainly. The Ch 15 historical cases otherwise check out, including the Jharkhand 1.5–2M figure, which is the paper's own abstract wording.


# Book 1 — Review of the manuscript and its fidelity pins

Files reviewed: `combined.md` (15 chapters, 45,628 words) and `combined_pins.md` (14 pin files, 458 pins).

## 0. What this review could and could not check

I cannot run `verify.sh` or load `constitution.nibli` here, so nothing below claims a pin passes or fails. What I could do, and did:

- Count every pin file's queries, `:refuse`, `:accept`/`:accept-scoped` and `:require` lines and reconcile them against each file's `:expect-pins`.
- Check every prose sentence quoted in a pin comment against the current manuscript, to find pins whose anchor sentence has been rewritten out from under them.
- Read every pin block for the failure mode the pin files themselves warn about ("FALSE for the wrong reason" / vacuous greens) and for premises the prose asserts but no pin holds.
- Cross-check each chapter's summary of another chapter's rule against that chapter and its pins.
- Spot-check the historical and empirical claims in Chapter 15 against what I know, plus one web check on the Jharkhand figure.

Scope limit: pin files for state-form, delivery receipts, liberty shell, equality, family/life-course and Class Nine are not in the upload (several headers say coverage is "delegated to `state-form.pins.nibli`"). Where the prose leans on those, I say "unverified here", not "unverified".

## 1. Verdict in brief

The fourteen derived chapters are an unusual and largely successful thing: a constitutional argument in which nearly every load-bearing sentence has a machine check behind it, the checks are genuinely adversarial, and the book confesses its own defects rather than describing intentions. The Boss/Rebel argument (Ch 2), the personhood loop (Ch 7), the Cira repair (Ch 6), the grade-from-teaching-entries insight (Ch 10), the farmhouse-to-"filled from the wrong side" arc (Ch 11) and the Jala borrowed-vocabulary section (Ch 12) are each strong enough to carry a chapter on their own.

Two problems run through the whole manuscript and are the reason the ratings below are not uniformly high:

1. **Spec-register paste with no pins behind it.** Roughly 5,000–6,000 words — most of Ch 8's second half, the state-form paragraphs in Chs 1, 2, 3, 9, 12, the "Family and the life course" and "Class Nine" material in Ch 15 — are written in the voice of a constitutional specification, not the book's voice, and none of it is exercised by the supplied pins. The correlation is the finding: wherever the prose stops sounding like the author, the checks stop too. The book's own method says a sentence without a pin is a sentence that may have started lying.
2. **Changelog residue.** "Used to" (16), "no longer" (37), "an earlier version… a later version…" — the reader of a finished book has no "before". Some of this is deliberate history (the farmhouse, the alarm, Cira) and it works there. Much of it is revision narration that should be cut to the present tense.

Verification found nine specific issues (Section 2), one internal contradiction between chapters (Section 3), and six stale pin anchors. None is fatal; two are real gaps in what the pins prove.

Overall: **8 / 10** as a manuscript; the derived chapters alone would be 8.5–9.

## 2. Verification results

### 2.1 What reconciles

Every pin file's declared `:expect-pins` matches its actual pin count under the convention *queries + refuse + accept + require*:

| Ch | File | expect | queries | refuse | accept | require | defect-tagged |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | What Counts as Evidence | 40 | 28 | 5 | 7 | 0 | 0 |
| 2 | Public Answerability | 22 | 21 | 1 | 0 | 0 | 0 |
| 3 | Who Holds the Pen | 23 | 22 | 1 | 0 | 0 | 0 |
| 4 | The Shield | 34 | 33 | 1 | 0 | 0 | 0 |
| 5 | Voiding | 39 | 38 | 0 | 1 | 0 | 0 |
| 6 | Clawback | 39 | 36 | 2 | 1 | 0 | 0 |
| 7 | A Prisoner Is a Person | 22 | 19 | 2 | 1 | 0 | 0 |
| 8 | What You Are Owed | 51 | 44 | 2 | 3 | 2 | 0 |
| 9 | The Vote | 24 | 19 | 2 | 3 | 0 | 0 |
| 10 | Contribution | 27 | 27 | 0 | 0 | 0 | 0 |
| 11 | Where People Are Put | 45 | 45 | 0 | 0 | 0 | 0 |
| 12 | Changing the Rules | 26 | 24 | 0 | 0 | 2 | 2 |
| 13 | The One Thing Taken | 35 | 33 | 2 | 0 | 0 | 0 |
| 14 | When the System Notices | 31 | 28 | 0 | 1 | 2 | 7 |

Nine `:defect` pins remain open (the Amend_Sneak totality guard ×2, the isolation marker ×7). All are disclosed in the prose.

### 2.2 Findings

**V1 — Confounded kinship controls (Ch 5 pins, lines ~921 and ~1034). Real gap.**
The Sirefa/Scionfa (parent) block and the Mara/Maro (married) block both assert seats, judgments and captures against Targ / Targ2 and pin `false(...)` FALSE, with comments saying kinship is "the only failing conjunct". Neither block supplies `authorized(signer, IndependentReviewer, target)`. The Una/Uno control, the Unrela/Unrelb control, the Lupo/Partnr block and the closing exhibit rule all show that the independence finding *is* a conjunct of the void rule. So both kinship blocks fail on two conjuncts at once and isolate nothing — exactly the "FALSE for the wrong reason" the file's own preamble warns about. Worse: the prose now says "the absence of parent, marriage, sibling … records is not enough" and "Missing kinship never certifies independence", which reads as though the kinship conjuncts were *replaced* by the affirmative finding. If they were, these two blocks test a rule that no longer exists and pass forever. Fix: add the two `authorized(…, IndependentReviewer, …)` lines to each kinship block so the FALSE isolates kinship; or, if kinship no longer blocks, delete the blocks and the two prose sentences that still imply it does.

**V2 — "THE FOUR FACTS" is stale (Ch 5 pins, line 767).**
The Bela block pins two credentials, two judgments, two captures and the void. Under the current rule the two `authorized(Gia/Hex, IndependentReviewer, Bela)` findings are also premises. They are unpinned, and the comment's "delete any one of these and false(Bela) stops deriving" is no longer the whole list. Same shape in Ch 10: `reward(Gia)` TRUE requires a `cite(Gia, …, Bela)` that is never pinned; and Ch 11: `at(Hano, PlacementHome)` is asserted in prose and never pinned TRUE.

**V3 — `broken(Vex)` FALSE is never pinned (Chs 2, 3, 5, 6).**
The prose says Vex "was never recalled" and every Vex pin relies on the pen being withheld *by the carried void alone*. If a recall entry for Vex ever crept into the cast, `permits(Review, Vex)` FALSE would stay green for the wrong reason and the three-way separation exhibit (answerable / not believable / powerless) would silently lose its third leg. One pin fixes it.

**V4 — `LowSec` exists in the pins and nowhere in the prose (Ch 1 pin line 151; Ch 11 pins lines 2164, 2217).**
`building(LowSec, X)` is pinned FALSE three times. The prose names only Homestay and HighSec, and Ch 13 says "the facilities are named in the rules that send people to them and nowhere else". Either LowSec is dead vocabulary — in which case these three are the vacuous "kind one" FALSEs the preamble tells you not to write — or there is a third destination the reader is never told about. Decide which, and either drop the pins or add the facility to Ch 11.

**V5 — Free-vocabulary constants undercut Chapter 1's opening claim.**
The chapter opens: "when someone wants to say something about you that is not on the list, they cannot. Not *may not*. Cannot. There is nowhere to put it." That is true of *relation names*. It is not true of the constant slots: the pins themselves say the task constant in `work(Sata, Care)` is "free vocabulary like any other", that "any word fills" the `cite` slot, and that `Two` in `year(Term_Ruk, Two)` is "an opaque name". So `cite(Koa, Untrustworthy, Esa)` or `work(Hano, PersonOfInterest)` loads, persists, and is a mark that sits in the file describing what somebody thinks you are like. It is inert — no rule reads it — and the chapter's later concession about Koa ("Not a mark that cannot be made; a mark that does nothing on its own") is the honest position. The opening paragraph overstates it. Specific weak phrase: **"There is nowhere to put it"** and **"The sentence does not go anywhere"** — both should be narrowed to "no rule anywhere reads it".

**V6 — Overloaded relations the book does not disclose.**
Chapter 12's thesis is that borrowed vocabulary is where rules meet unexpectedly. Chapter 1 presents the list as if each English entry were its own word. Four are not:
- `broken/1` is both *someone was recalled* and *the court is compromised* (Ch 3's "recalled or otherwise broken" is the only hint).
- `cite/3` is both *an examiner cited the grounds they looked on* (`cite(Yano, Hunch, Adam)`) and *a court tied a case to a person* (`cite(Court, Case_Voss, Voss)`). Whether a court's case-tie can satisfy the examiner-pay rule's grounds conjunct, or vice versa, is untested.
- `at/2` is both *typed adulthood evidence* (`at(Esa, GeneralAdulthood)`) and *placement-home availability* (`at(Hano, PlacementHome)`).
- `judge/2` is conviction, examination, and appeal (Ch 3 discusses this one; the other three are undisclosed).
Ch 1 should say that the list is a list of relations and that several English entries share one, because that is the surface Ch 12 tells the reader to watch.

**V7 — The name `Points`.**
`lose(Points, X)` is the only conclusion the machine ever records about recognition, and its constant is called *Points*, in a book whose Chapter 10 says "recognition carries no quantity anywhere in this design … It is not a currency". It is only a name. It is also the one word a hostile reader of the machine part will quote. Rename it (`Standing`, `Esteem`, `Recognition`).

**V8 — Delivery receipts, the action-duty chain, and "two public witnesses" have no query pins here.**
Ch 8 describes the recipient-side receipt route in detail (record from a source, authorised witness for that person, witness ≠ source, predeclared alternate, no roster requirement). Not one pin in the upload writes a receipt and watches `eats(X)` derive, or writes a provider-only receipt and watches it refuse. Ch 14's "independently named reader … alternate … continuity … individual remedy … recurrence checking" is backed by two `:require` greps and no query. And nothing pins that `observe(Chronicle, …)` and `observe(TemporalReview, …)` require `public(Chronicle)`/`public(TemporalReview)` — if witness names are free constants, two invented witnesses agreeing is the forgery Ch 5 says the design reports. If these live in other pin files, the chapter headers should say so the way Chs 1/2/3/9/12 do for state-form; if they don't exist, the prose is ahead of the checks.

**V9 — Six stale pin anchors (comments quote prose that no longer exists).**
- Ch 2 pin-file header still reads "Standing, and Why It Is Never Revoked" (line 244).
- Ch 4, line 555: "Pax has none." → prose now "Pax is not."
- Ch 10, line 1983: "it walks through two of the three" — the care paragraph was rewritten.
- Ch 13, line 2580: section quoted as "What the design does not say is WHEN" — now "And the authority must be current".
- Ch 13, line 2617: "A term could be written into this design tomorrow and nothing in it would count the term down" — sentence gone; the pin (admit `year`, state a term, nothing moves) is still a good measurement but its prose anchor is missing.
- Ch 14, line 2814: "The design could mark the review body for having ignored the duty; that rule was tried and it works" — sentence gone; the chapter now narrates three generations of its own position instead.
Also: a duplicated line `authorized(Ambi, IndependentReviewer, Solo).` at pins 963–964 (harmless), and the Ch 1 pin comment "For Hano's family label is recorded only to prove…" is ungrammatical.

## 3. Cross-chapter consistency

**One contradiction.** Ch 12, line 3047: "Voiding a person properly takes two examiners, seated by different bodies, neither related to her, both with their credibility intact, in the same period." Ch 5 says the opposite twice and pins it: Lupo, already voided, is seated, holds a pen, and co-signs Frisk into the same condition — "He is a discredited man and a credentialed one at the same moment, and he can put his name to a stranger's voiding and make it stick." "Both with their credibility intact" is false under the design; the same-period void does not read `false`. "Neither related to her" is also loose — the parent rule (Ch 5) voids the *judge* and does not block the void. Fix the sentence in Ch 12.

**Undeclared premise.** Ch 11, line 2696: "Don is severe without either entry." Ch 4 gives Don one injury (Pax). Severity needs a second person, and the pin comment supplies what the prose never does — Don was cruel to someone else. A reader cannot reconstruct Don's severity from the book. One clause fixes it.

**Consistent, and worth saying so:** the Rebel/Boss shield across Chs 1, 2, 4; Vex across Chs 2, 3, 5, 6, 10 (forgiveness restores word and pay, never the pen); the Nia/Adam/Voss relief asymmetry; Kel and Adam's convict-then-housed state across Chs 8, 11, 13; the "record only adds" thinness of the vote; the isolation marker's roster (Rex included). The cast is disciplined and the chapters agree with each other.

**Two design questions rather than defects.** The custody fixtures require `endorses(Electorate, Case_X)` for every custody case — the electorate endorsing individual imprisonments is either a placeholder for a body not yet named or a plebiscitary design the prose never defends. And `authorized(X, IndependentReviewer, Y)` — the finding Ch 5 calls "an attributable decision whose source and conflicts can be contested" — has no author slot. **"Attributable"** is the wrong word for a relation that names no finder; it is exactly the `deceive` shape Ch 1 confesses.

## 4. The two systemic problems, located

**Register.** Passages that read as specification rather than argument, and have no pins in the upload:
- Ch 1: "The later rules for federal government…" through "…that a future source cannot change it" (lines ~262–288), plus the leaked authoring comment `<!-- Coverage owner: FS-CVF-003. -->` at line 282.
- Ch 2: "Do not read those older routes as the map of the federal government…" paragraph.
- Ch 3: the four "state-form" paragraphs at the end of "What this rests on".
- Ch 7: "The standing roots do not replace this line" (whole section).
- Ch 8: from "Care is not a family invoice" through the end of "Where the protection stops", and most of "Owed by whom" — about 1,600 words.
- Ch 9: "The constitution adds the political frame…", "The home custody does not move" (most of it), "And the same silence as before".
- Ch 12: the Assembly/referendum threshold paragraphs.
- Ch 15: the anti-subordination paragraphs in Capture; the entire "Family and the life course" subsection; Class Nine — roughly 2,500 of 8,100 words, in a chapter that opens "this part is me speaking".

Internal labels that reached the reader: `FS-CVF-003` (282), `Gate A` (1788), `T3` and `Book 2 handoff` (4131–4132), `Class Nine` (4174). None is defined for a reader.

**Changelog residue.** Where history is the argument (Ch 6 Cira, Ch 11 farmhouse and alarm, Ch 12 Jala) keep it. Elsewhere — "the most consequential change in this chapter" (Ch 1, the reader has no prior chapter), "The gap that used to be here" (Ch 3), "for most of this design's life" (five occurrences), "An earlier version of this chapter … A later version …" (Ch 14) — the reader is being told what a draft said. Convert to present tense or cut.

## 5. Chapter by chapter

Ratings are out of 10 and weigh argument, pin fidelity, and prose together. "Carries" = what the chapter proves and how well; "Weak" = specific sentences or claims; "Fix" = the one edit that would move the rating most.

### 1. What Counts as Evidence — 8
Carries: the writable/derivable distinction; the severity walk (an entry that "looked like a fact. It was a verdict"); the loud-door closure and its one-sidedness; the broken-court lever; the honest ending that some *premises* are verdicts too.
Weak: the opening overclaim (V5); the list itself has grown until "short and closed" is no longer something a reader can hear — the T3 entries ("a list put record inventories, events, or entries in order; a time service named the boundaries…") are eleven lines of machinery in the middle of a rhetorical list; undisclosed overloads (V6); leaked `FS-CVF-003`; "Read that slowly, because it is the most consequential change in this chapter" narrates revision.
Fix: split the list into the human entries and a one-sentence pointer to the record-keeping entries; narrow "nowhere to put it".

### 2. Public Answerability — 9
Carries: Boss/Rebel is the book's cleanest argument and is fully pinned; the Vex three-way separation is executed on one person in one snapshot; the closing deletion cost is stated exactly.
Weak: the federal-government paragraph interrupts; "the electorate and the assembly, neither of which is answerable either" is unpinned; the Ch 3 claim that re-seating a forgiven Vex "would move nothing" is not in the supplied pins.
Fix: move the state-form paragraph to a footnote or cut it; add the two FALSE authority pins.

### 3. Who Holds the Pen — 8
Carries: Sock/Puppet as a confessed past defect; the hurried door as a maxim executed; the victim-need-not-exist observation; the opposite-default argument for relief vs shield is the most careful paragraph in the chapter.
Weak: the credential conditions paragraph ("the witnessed current record positively carries your clear status forward, and no reconciled void follows you… the selected end of one witnessed constitutional record line") cannot be pictured by a reader; `broken(Vex)` unpinned (V3); the hurried-door and sabotaged-copy checks are not in these pins; the closing state-form paragraphs are spec.
Fix: rewrite the conditions as a numbered list of four things in plain words, then say the machine's version once.

### 4. The Shield — 9
Carries: Don → Sly → Kel → Rex → the persons-only refusal → `public(Pax)` → Zeno is the best-verified chapter in the book; the harm-asymmetry argument for protection-by-default is stated without softening; the "no self-defence, only intent, and intent only ever points at you" paragraph is a genuine finding.
Weak: "the review body examined the exposure" is repeated as if structural; Ch 1 already conceded `deceive` names no finder — one clause here should too. The T2 "Should it be bounded" paragraph is spec-adjacent.
Fix: one clause on the finder.

### 5. Voiding — 7.5
Carries: guards that turn on the examiner; the circularity refusal ("the question would have to answer itself before it could be asked"); the record knows *who* and never *why*.
Weak: V1 and V2 — the chapter's independence and kinship claims are the least cleanly verified in the suite; "What has to cross the record" is machine-language for two paragraphs; **"attributable"** for an authorless finding.
Fix: repair the two kinship blocks (V1) and pin Bela's two independence findings (V2). This is the chapter where the pins most need work.

### 6. Clawback — 8.5
Carries: "Recognition is not a balance that can be debited; it is an answer, and the answer stops being yes"; the Cira section is the moral centre of the book and the narrow-was-always-repeal reasoning is exactly right; "the worst it can now do is reach for what the guilty were given".
Weak: `Points` (V7); the chapter says "nothing reads the loss" and "consulted by no rule anywhere" — true, but Ch 12 shows the loss rule *fires* on non-persons (`lose(Points, Amend_Floor)`); a sentence here would pre-empt the reader's surprise there.
Fix: rename the constant; one sentence on the rule firing on anything voided.

### 7. A Prisoner Is a Person — 8.5
Carries: the loop argument is the intellectual spine of the design and is pinned both ways (heresy law refused; persons-only shield refused); "found, not designed" and "a wall believed indestructible is exactly the wall nobody watches" are the right kind of honesty; the wall/promise distinction.
Weak: the Zed set-up now needs a paragraph of custody-fixture caveats before the punch lands; the final section is spec-register and ends the chapter on "outside Gate A"; the tidied-grammar attack is claimed measured but not in these pins.
Fix: fold the fixture caveat into one sentence and end the chapter on "What that means".

### 8. What You Are Owed — 6.5
Carries: "Owed is not the same as delivered" is the book's spine and the pins for it are thorough (every item, both sides, one person); the fiat-rule attack ("the lie is in the rule") and the threaded-name deletion are excellent.
Weak: the second half is the manuscript's largest register break — care, receipts, liberty shell, equality, accessibility, positive measures, duties-by-kind, excuses, civic duties — about 1,600 words in a legal-drafting voice with no pins in the upload (V8). The argument for *belief* and *company* on the floor is two paragraphs; the specification of accommodation refusals is longer.
Fix: cut the spec material to one paragraph naming the families and where they are checked; keep the shape "duty ≠ offer ≠ arrival ≠ operation" in the book's voice.

### 9. The Vote Conviction Does Not Take — 8
Carries: "not yet / taken / evidence absent" is a real trichotomy; the record-only-adds thinness ("Nobody has to refuse the attack; they have to decline the repeal") is the sharpest structural observation about the franchise in the book and is pinned resident.
Weak: "The home custody does not move" and the closing section are spec; two adulthood names (`match(·, GeneralAdult)`, `at(·, GeneralAdulthood)`) appear in the pins and neither is explained.
Fix: halve the state-form material; keep the Esa evidence-absent case.

### 10. Contribution — 8.5
Carries: "There is no number" and "What is lost" — the observation that teaching entries read backwards are a grade, that the count doesn't go through the doors, and that the design counts on the way into punishment and not into reward, is original and the chapter is honest that the defence "came after the asymmetry".
Weak: the child-labour and care paragraphs at the top are defensive spec; `Points`; `cite(Gia,…)` unpinned; "even the void's own examinations still ask for no reason" is right and could be one sentence shorter.
Fix: move the care/child paragraphs down and compress them.

### 11. Where People Are Put — 8.5
Carries: the farmhouse ("reading a rule and reading a description of a rule feel like the same activity and are not"); the alarm that was wrong every time it sounded; "the case nobody wrote a rule for"; "filled, from the wrong side". This is the chapter that best shows the method finding things.
Weak: Don's severity undeclared; LowSec (V4); `at(Hano, PlacementHome)` unpinned; the matrix paragraph ("every combination of confinement state, severity, legacy family entry…") is spec.
Fix: one clause for Don; resolve LowSec.

### 12. Changing the Rules — 8
Carries: the register guarding its own name; the Jala section is the best writing in the book about vocabulary ("This record does not know the difference between a person and a proposal. It knows names, and what is written about them"); "The strongest protection in this design is the impossibility of writing certain rules. The weakest is the integrity of the record those rules are written in."
Weak: the Ch 5 contradiction at line 3047 (Section 3); the Assembly/referendum thresholds are spec; the totality guard is still an open `:defect` (disclosed, fine).
Fix: correct line 3047.

### 13. The One Thing Taken — 8
Carries: "three of them run the wrong way"; "the difference between a design that has thought about imprisonment and one that has thought about the decision to imprison"; "Leaving custody returns a person to a supplied record with no non-carceral receipt".
Weak: the T3 authority paragraph is machine-language ("unconflicted event order and record-entry order… the selected end of the witnessed constitutional record line"); two stale pin anchors (V9).
Fix: rewrite the authority paragraph as three plain conditions; re-anchor the two pins.

### 14. When the System Notices It Broke — 7.5
Carries: the two alarms — one right about everyone and therefore saying nothing, one wrong about the people it named — is the clearest statement of self-audit's limits I have read; "It cannot be bought. It can be starved"; "a new kind of wrong begins life unowed".
Weak: half the chapter narrates its own previous versions; the action-duty chain has no query pins (V8); stale anchor at pin 2814; the "adulthood case" paragraph belongs in Ch 9.
Fix: cut the version history to one sentence and let the current position stand.

### 15. The Five Joints — 7
Carries: the method demonstration on the author's own democracy/happiness claim is the right way to open; the five joints are the right five; the historical cases are accurate where checkable (Section 6); the concessions are the best in the book — "social democracy with extra steps … the answer is yes", "I accept that sentence as an ambition, not as a description of what this version has proved", "that book needs people I have not met".
Weak: (a) ~2,500 words of spec in a chapter that promises a person arguing — the Family/life-course subsection and Class Nine in particular are lists of barriers with no argument around them; (b) the reviewers named Meta, DeepSeek, Gemini and Grok are AI systems and the text never says so — "the corpus's angriest" and "a hostile graded read" read as human colleagues, and citing that read as calling a chapter "excellent" is praise in a section that promised a knife; (c) Rotation: "the mechanism that broke first was … rotation" is the author's reading of the kibbutz unwinding; the best-documented account (Abramitzky) leads with the 1980s debt crisis, brain drain and adverse selection, with rotation fatigue as one strand — **"the mechanism that broke first"** is overstated; (d) internal labels `T3`, `Book 2 handoff`, `Class Nine`.
Fix: say plainly that the draft reviewers were AI models; cut the spec subsections to a paragraph each and put the argument back around them.

## 6. Chapter 15 empirical spot-checks

| Claim | Status |
|---|---|
| Owen bought New Harmony in 1825; ~800 settlers; dissolved within two years | Consistent with the standard accounts; "seven constitutions" is one of the counts in circulation |
| London labour exchange priced labour by the hour; valuators; warehouse of the unwanted | Consistent (National Equitable Labour Exchange, 1832–34) |
| China work points; "eating from one big pot"; exit rights removed in 1958 as the break | Consistent; the exit-rights mechanism is Justin Yifu Lin's argument and could be named |
| Kibbutzim: Degania 1910; ~270 communities; ~3 in 4 on differential pay by ~2010 | Consistent with Abramitzky's figures; the "rotation broke first" causal claim is the author's (see Ch 15 weak (c)) |
| Ujamaa: Arusha 1967; compulsory from 1973; ≥5 million moved; 13 million claimed; Scott | Consistent with *Seeing Like a State* |
| Mondragon: 1956; ~70,000; pay ceilings up to ~9:1 vs ~300:1; Fagor 2013 redeployment; bank, insurance, university; overseas non-members | Consistent |
| Kerala People's Plan 1996; a third+ of plan budget; ~100,000 trained; elite capture where training thin | Consistent |
| Cybersyn: ~500 telex machines; dismantled after the coup | Consistent |
| Auroville: emergency takeover, Foundation statute, Supreme Court affirmation "five decades later" | Consistent (1980 Act, 1988 Foundation Act, 2022 judgment) |
| WIR: 1934; 92 years; countercyclical | Consistent |
| Jharkhand: >1 million cards cancelled; most cancelled cards genuine (Drèze et al.); "1.5–2 million legitimate beneficiaries lost access at some point" (Muralidharan–Niehaus–Sukhtankar); Santoshi Kumari, Sept 2017, cause contested | Consistent — the 1.5–2M figure is the paper's own abstract wording |
| Democracy/happiness r≈0.5 → ≈0.2 after income control | Author's own analysis; unverifiable here; direction matches the literature |

## 7. Prioritised edit list

1. Ch 5 pins: add `authorized(…, IndependentReviewer, …)` to the Sirefa/Scionfa and Mara/Maro blocks, or delete them and the two prose sentences implying kinship still blocks. (V1)
2. Ch 12 line 3047: remove "both with their credibility intact" and "neither related to her". (Section 3)
3. Pin `broken(Vex)` FALSE once. (V3)
4. Resolve LowSec: dead vocabulary or third destination. (V4)
5. Delete the `<!-- Coverage owner -->` comment and the four internal labels (`Gate A`, `T3`, `Book 2`, `Class Nine`) or define them for a reader.
6. Rename `Points`. (V7)
7. Ch 1: narrow "nowhere to put it" to relations; disclose the four overloaded relations. (V5, V6)
8. Ch 11: state Don's second victim.
9. Re-anchor the six stale pin comments and the Ch 2 pin-file header. (V9)
10. Ch 15: state that the named reviewers are AI systems; soften "the mechanism that broke first".
11. Ch 8 second half, Ch 9 state-form sections, Ch 15 family/Class Nine: cut to the book's voice, or add the pins and say where they live. (V8, Section 4)
12. Global: convert changelog phrasing to present tense except where the history is the argument.

The pins are the strongest part of this project and the book is right to trust them. The places to distrust are the ones where a chapter got longer without its pin file getting longer.
