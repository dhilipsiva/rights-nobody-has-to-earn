This is a rare book. 15 chapters, ~39k words, 417 executable queries, 18 negative-capability proofs (`:refuse`), 17 positive existence proofs (`:accept`). It's both an argument and a program. The prose makes claims. The pins file makes those claims checkable.

Here is the full read-through, validated against your `combined.pins.md`.

### How I verified it

**Mechanism:** `combined.md` is the prose. `combined.pins.md` is the fidelity layer. Each chapter has an `:expect-pins N` header. A pin is either:

* `? predicate(args). => TRUE/FALSE` — does this fact derive in the supplied record?
* `:refuse reasoning /pattern/` — this write *must not load* (vocabulary closure, derived-only)
* `:accept` — load a fact/rule and see what it does to the base
* `:defect`, `:require` — guardrails for known weaknesses

**What it proves:** The chapter's load-bearing sentences are true in the current `constitution.nibli` and `state.nibli`. If a pin flips FALSE->TRUE, the prose is lying.

**Coverage:** 14 pinned chapters + 1 unpinned meta-essay.

| Chapter | expect-pins | Core proof type |
| --- | --- | --- |
| 1 What Counts as Evidence | 40 | Closure + derivation |
| 2 Public Answerability | 22 | Never-revoked |
| 3 Who Holds the Pen | 23 | Bare flag derives nothing |
| 4 The Shield | 34 | Shield blocks conviction, can't be tightened |
| 5 Voiding | 39 | Two pens, two houses, voided pen can't void |
| 6 Clawback | 39 | Recognition shuts off, epoch carry, forgiveness |
| 7 Prisoner Is Person | 22 | Second door, free(Nemo) persists |
| 8 What You Are Owed | 51 | Owed in full, by whom, legacy learning dormant |
| 9 Vote Conviction Does Not Take | 24 | Missing evidence != not yet, and then nothing happens |
| 10 Contribution | 27 | Three doors gated on voiding, care routes |
| 11 Where People Are Put | 45 | Severity + typed home, family does no work, alarm works |
| 12 Changing Rules | 26 | Docket-restricted, loss against proposal |
| 13 One Thing Taken | 35 | Hano/Jala lists, term that releases nobody |
| 14 When System Notices It Broke | 31 | Audit -> duty, isolation + placement markers |

Last chapter "The Five Joints" has 0 pins by design - it's the author speaking outside the machine.

## Chapter by chapter

### 1. What Counts as Evidence — Rating: 9.5/10

**Thesis:** The record's vocabulary is closed. You cannot write "reliable," "dangerous," "rich," "severe" or "prisoner" directly. Personhood is now derivable from birth OR first-contact/jurisdiction/control.

**Validation:**
* Pins 1.6b are the keystone of the whole book: `rich(Adam).` and `dangerous(Adam).` are `:refuse /not admitted vocabulary/` — proof that Article 0a `admits` actually closed the lexicon. Before 0a, nibli would have answered TRUE for any real corpus name like `rich`. Now it aborts.
* `severe(Zed).` and `prisoner(Zed).` and `authority(Pax).` all `:refuse /derived-only/` — you cannot declare severity, imprisonment, or authority. This fixes the oldest cheat in the draft.
* `person(Wraith)` unchanged when you add `family(Hano)` or `deceive(Rebel)` — proves findings don't pollute personhood. The ordering note is deliberate and correct.
* The severity pairing fix is pinned beautifully: `attack(Probe,Vic1)` + `cruelty(Probe,Vic1)` on *same* victim => severe, but attack on Vic1 + cruelty on Vic2 => no severity. This refutes the old "any two of three" gloss.
* `permanent(Art_Floor) => TRUE` but `permanent(Art_Evidence) => FALSE` — the uncomfortable admission is pinned. The evidence list itself is not constitutionally entrenched. The prose admits this. That honesty is the chapter's strength.

**Weakness:** You correctly note severity still attaches to a *person*, not an act. That's a person-rating, computed not written, but a rating still. The `family` entry is kept though it does zero work — a vestigial human detail that undermines your "no impression" claim slightly.

**Prose:** Best in the book. Flat by choice.

### 2. Public Answerability, and Why It Is Never Revoked — 9/10

**Thesis:** Answerability = being examinable + shielded whistleblower protection applies to you. It comes from being a public institution OR being seated by a seating body. It is never revoked.

**Validation:** Pins test that `public(Court)` + `authority(Court)` both TRUE, and that `choose(Convocation, Hex)` gives authority without electorate or office. This kills the old two-routes-only sentence. The never-revoked property is exercised via Rex, Sly etc staying shielded.

**Strength:** Clears up the English conflation of "answerable" (can be exposed) vs "powerful" (can act). 

**Risk:** Still relies on `public(X)` being written directly — who writes that?

### 3. Who Holds the Pen — 9/10

**Thesis:** A pen (auditor credential) is not authority. You need a pen to document, but documentation alone does nothing. Two pens from *different* houses are needed to void.

**Validation:**
* `THE BARE FLAG DERIVES NOTHING` and `THE BARE JUDGMENT DERIVES NOTHING` pins: a naked `capture` or `judge` fact with no auditor entry derives nothing. This was ruled 2026-08-02 and is now guarded.
* Independence condition pinned: one person holding both pens cannot void.

**Strength:** This is where the design becomes anti-collusion. Electorate + Convocation split is load-bearing.

### 4. The Shield — 8.5/10

**Thesis:** `expose(Authority)` blocks `prisoner(Exposer)`. The shield was too easy to get, so it was tightened to require a real authority target, and cannot be tightened to require a *judged* authority.

**Validation:**
* `THE TIGHTENING THAT CANNOT BE MADE` — attempt to require that exposed authority already be judged fails stratifier. Pinned as impossible. That's an honest limit.
* Frame-up from outside and reach-back pins test that shield survives fabricated cases.
* Don / Sly / Kel / Rex cast shows escalation: claim against non-power, unexamined claim, caught claim.

**Weakness:** Shield is strong — strong instruments get picked up by wrong hands, as you note. The prose doesn't fully answer the cost.

### 5. Voiding — 9/10

**Thesis:** Voiding = two auditors, different houses, same target, both with pen. Voided pen holder cannot void in same snapshot. Void closes forward only.

**Validation:**
* `FIXTURES. MUST BE LAST` block — ground facts persist check. Careful hygiene.
* `A voided man holding the pen, in the same snapshot` — proves temporal ordering: void applies next round, not instant.
* Intimate pair finally named: distinctness was the only bar, not relationship.

**Strength:** Most formally tight chapter. The "what actually closes it, one round late" pin is elegant.

### 6. Clawback — 8.5/10

**Thesis:** When credibility is voided, recognition stops. No partial forfeit. Loss recorded. Forgiveness restores recognition but does not rewrite history.

**Validation:**
* Epoch carry pinned: void in earlier period carries across record replacement via two witnesses.
* Forgiveness measured: `forgive` removes void marker but loss remains in past.

**Risk:** The economy of esteem is total — lose everything at once. You defend it as simplicity, but it's harsh.

### 7. A Prisoner Is a Person — 9.5/10

**Thesis:** `prisoner(X)` => `person(X)` is load-bearing, not sentimental. Conviction now requires case-tie with two public witnesses on each tie + custody lease with limit, period, constitutional source.

**Validation:** `THE SECOND DOOR. MUST STAY LAST: free(Nemo) is a ground fact and persists` — ensures free people aren't made prisoners by missing facts. Zed example pinned with full case-tie requirements.

**Strength:** This chapter makes the book structural, not promissory.

### 8. What You Are Owed — 8/10

**Thesis:** Floor: safe, eat, housing (weatherproof, warm, water, sanitation), care, learn, speak, believe, company. Owed without condition. Cannot be bought with work/payment/compliance.

**Validation:** 51 pins, largest. `OWED IN FULL, ENUMERATED, ONE PERSON` enumerates floor for Hano. `legacy learning` route marked dormant — honest.
* Source-bound finding/action contract: `& obliged($reader, $subject) -> obliged($reader, $reader_duty, $reader_standard)` + `derived_only("obliged")` — duty does not prove action.

**Weakness:** Formal arrival heads only cover food, shelter, care, material security, company + legacy learning. You list learning, mobility, communication as standard but admit they are not formalised. Gap is acknowledged in Five Joints.

### 9. The Vote Conviction Does Not Take — 8.5/10

**Thesis:** Franchise and candidacy follow personhood + typed adulthood evidence, not punishment status. Adulthood attaches automatically, cannot be granted.

**Validation:** `MISSING EVIDENCE LOOKS LIKE NOT YET` — missing typed adult evidence = not yet adult, not denial. `AND THEN NOTHING HAPPENS` must be last and unscoped — proves no further deprivation follows.

**Strength:** Theorem completion: if prisoner => person, and person + adult => vote, then prisoner votes. Clean.

### 10. Contribution — 8/10

**Thesis:** Three doors: teach, work, honest audit. Care = work. No age gate. Void closes doors forward. Recalled status also closes. Empty pair stops paying.

**Validation:**
* `THE THIRD DOOR IS GATED ON VOIDING, and it was not always` — historical fix pinned.
* Care routes: each route tested separately.
* `THE VOID CLOSES THE DOOR FORWARD` — ground facts persist, forward closure only.

**Strength:** Solves "where does raising a child fit" without creating family invoice.

### 11. Where People Are Put — 9/10

**Thesis:** Placement = severity => HighSec else Homestay *requires* `at(person, PlacementHome)` typed fact. Family entry does no work. Old unary family kept but inert.

**Validation:** 45 pins. Alarm exercised: `building(HighSec,Ruk) => TRUE` while `err(Ruk,Placement) => FALSE` — old marker false-alarmed on correct placement, repaired marker is silent. Farmhouse example shows Homestay not civil residence.

**Strength:** Removes discretionary placement decision. This is the biggest practical win.

### 12. Changing the Rules — 8/10

**Thesis:** Two change interfaces: old `becomes law` (assembly proposes, electorate approves, dies unless marked dead) holds only outcome fact, not count; state-form amendment requires positive current record, full Assembly result, national referendum, etc. Docket-restricted, not type-safe.

**Validation:** `THE RULE IS DOCKET-RESTRICTED, NOT TYPE-SAFE` — fact persists, keep last. Loss recorded against proposal.

**Weakness:** You note older route authenticates nothing. That's still true. New route requires current records but supplied record does not authenticate itself. Legitimacy gap remains, admitted in Ch14/15.

### 13. The One Thing Taken — 9/10

**Thesis:** Punishment = loss of free movement, nothing else. Hano list vs Jala list pinned item by item. No reduction of floor, vote, speech, credibility, recognition, placement-family link. Sentence has term that releases nobody — end is not release, need separate `finished`.

**Validation:** Hano/Jala lists are most readable pins in set. `TERM THAT RELEASES NOBODY` must be last.

**Strength:** One-sentence theory of punishment. The clarity is the point.

### 14. When the System Notices It Broke — 9/10

**Thesis:** `err(Person, Reason)` turns breach from invisible absence into askable fact. Markers: Isolation (every prisoner marked, but not shielded/cleared), Placement (now report-based), Duty, carry failures, order conflicts, adulthood evidence, custody without authority.

**Validation:**
* v0.9 placement repair validated: `err(Ruk,Placement) => FALSE` when correctly placed, true when reported misplaced (Ch11 probes).
* v0.8 duty: `obliged(Review,Ruk) => TRUE` via isolation marker alone — duty owed to category, not case. Converse `obliged(Ruk,Review) => FALSE` discriminator prevents backwards reading. Critical.
* `all $x: obliged(Review,$x) & ~capture(Review,$x) -> err($x,Duty)` — the exhibit you chose *not* to build, though stratifier permits it. You leave powerlessness as choice, not structure. Honest and painful.

### 15. The Five Joints — 9/10 unpinned

Valuation, rotation, coercion, capture, state. This is where you admit what the machine cannot prove: authentic records, honest elections, rosters, money, staffing, action, delivery, enforcement, feasibility, legitimacy. The repository checks reject unreviewed widening, but no amendment procedure approves widening, no entrenchment protects list.

This chapter saves the book from its own formalism. The flat characters complaint you keep — "There are no humans in this book. There are only variables." — is correct and intentional. You are right to keep it.

## Overall

**What works:**
* Closed vocabulary via `admits` is now real, not nibli-lexicon closure. The two `rich/dangerous` refuse pins are the whole opening claim made checkable.
* Severity derived from pairing, not written. Pairing logic pinned with fresh names disjoint from roster.
* Dual-house voiding + shield + never-revoked answerability = credible anti-retaliation design.
* Personhood from multiple roots (birth, contact, jurisdiction, control) with temporary name — solves statelessness without proving civil identity.
* Audit markers as first-class facts. The system can say "I am broken."

**What survives as risk:**
1.  **Entrenchment:** `permanent(Art_Evidence) => FALSE` — the evidence list that makes everything else checkable is not itself protected. Visible edit, but still editable.
2.  **Conclusions that look like facts:** `deceive` and `family` are written with no finder named, no definition. You admit this is a smaller version of the impression problem.
3.  **Legitimacy:** No argument makes anyone adopt these rules. Funding, operation, enforcement not derived. You state this in Ch15, but it's not fixed.
4.  **Severity as person-rating:** The only rating that travels with a person for whole sentence.

**Overall rating:** 8.9 / 10 for technical rigor + honesty. As prose, 8.5 / 10 — demanding but consistent voice. As verifiable spec, 9.5 / 10 — few books expose their own failure modes in executable tests.

If you want next steps, I can:
* Generate a diff of prose sentences vs pins that are one edit away from flipping
* Produce a dependency graph of which pins guard which paragraph
* Draft the missing "arrival heads" for learning, mobility, communication you flag in Ch8
