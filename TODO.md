# TODO — Book 1: the child at the heart, engines before breaks

Created 2026-09-16 on the author's instruction, after the root tracker had been
retired earlier the same day: *"Just plan and let us create a list of ordered
TODO items in a TODO.md file and work on it one at a time."* This file is that
list. It is a work list, not a second statement of any gate or discipline: the
controlling text of every rule it names is `CLAUDE.md`, and the two decision
records under `book-1/appendix/decisions/` are the controlling records for the
design it executes.

## How to work this file

- **One item at a time, in order.** Pick the first open item, implement it,
  run `cargo test --release --bin generate`, then `./verify.sh` (complete; run
  the two **sequentially, never beside each other** — a concurrent run starved
  the verifier and OOM-killed a test binary on 2026-09-16), commit, push,
  delete the item. Update an item if it is partly done.
- **Rules that bind every item** (one line each; the rulings in `CLAUDE.md`
  control): verification is Nibli pins and contradiction scans only — add no
  hashing, receipt or freshness gate; no digit and no counted claim in a
  derived chapter — state the rule that produces the count; never route a
  judgment through the compute backend; every argument position in a new fact
  is a new constant unless it is an institution; a derived chapter derives from
  the constitution and carries a paired `.pins.nibli` registered in
  `tests/pins/suites.json`; the register is flat — no interior state, never a
  cast name in a passage, never warm the cast; never invent testimony,
  external evidence, reviews or operational success; no arrival verb, no
  guarantee-of-growth or operation claim in prose (register-audit every prose
  batch before committing); moved author-approved text moves verbatim, and
  substantive rewording needs the exact-version record; new prose lands as
  `session-drafted, author-approved under delegated approval (2026-09-16)`.
- **Never touched in content** by the relocation tool (asserted byte-identical
  after a move): the constitution, the eight rule-family `*-source.json`,
  every non-comment line of every `*.pins.nibli`, `4-strata.py`, `registry/`,
  `book.md`, `manifesto.md`, `1.md`, `2.md`, `tmp.txt`, `reviews/`,
  `new-reviwes/`.
- Commits end with the standing attribution lines and explain why in the body.

## Where the rulings and the design live

Item 1 landed on 2026-09-16: the seven rulings are recorded in `CLAUDE.md`
under *The rebuild of Book 1 — 2026-09-16*, and the design this tracker
executes is in the two controlling records —
`book-1/appendix/decisions/child-with-nobody-decision.md` (the definition of
"most vulnerable", the chapter, the recurring section, the re-measurement)
and `book-1/appendix/decisions/reading-order-and-appendix-decision.md` (the
rule and the final chapter table, the names, the appendix override, the move,
the manifest, the tool and the checks, the engines and the breaks). Item
numbers below refer to those records' section names where a design detail is
needed.

## Items

### Phase A — the heart and the order

2. **Give the book a contents manifest, a relocation tool, and
   reference-integrity tests.** `book-1/contents.json` in the *current* order
   with roles/groups and no `rule` field; `src/authoring/contents.rs`; the three
   Rust sites, `combine.sh`, the pilot line switched to the manifest;
   `tools/relocate.py`; `reference_integrity_tests.rs` with the needle baseline
   measured **before** anything moves. Nothing moves. *Done when:* `cargo test`
   green with the fixed 14 and the two literal filenames gone; `./verify.sh`
   green; `bash combine.sh` and a pilot build produce the same ordered inputs
   as before.
3. **Move `new-book-plans/` under `book-1/`: the planning record to
   `book-1/appendix/`, everything else to `book-1/source/`.** One map
   (`tools/maps/2026-09-16-fold.json`: the 45 appendix moves with their
   renamed basenames, plus the flat `new-book-plans/` → `book-1/source/`
   prefix for the rest) → `apply` → `check`; `appendix/README.md` (R4 and the
   map); LICENSING clause; CLAUDE.md Files section rewritten (`new-book-plans/`
   entry retired; `book-1/source/` and `book-1/appendix/` described), the
   "no fourth exempt element" and `bound_sources_sha256` annotations;
   `book-1/README.md` gains the two subdirectories in its conventions;
   `book-2/TODO.md` cites repointed; the method part's pointer sentence names
   `book-1/appendix/` and `book-1/source/` (a pointer, within sealed rule (d)).
   *Done when:* `check` passes (every `new-book-plans/` string gone outside
   `reviews/` and git history; constitution, rule-family sources, pins
   non-comment lines and `4-strata.py` byte-identical; `git status` exactly the
   planned renames; `new-book-plans/` absent); `cargo test` (needle set
   unchanged modulo rewrite); `./verify.sh` (the runner's constitution path and
   every suites base/fixture/pin path resolve); `./generate.sh spine` and the
   three report generators byte-stable on a second run; `bash combine.sh` and a
   pilot build unchanged.
4. **Run the constitution against a person with one entry.**
   `tests/pins/records/child_with_nobody/{fixture,expect}.nibli`
   (`born(Nell).`; person TRUE, eight owe, eight entitled, every barrier,
   actualities FALSE, `vulnerable(Nell).` refused, Ori FALSE); base
   `counterfactual/no-birth-standing` + its pins file; the Nell case on
   `counterfactual/undelivered-marker`; `suites.json` entries. No prose. *Done
   when:* `./verify.sh --only` each, then full; `:expect-pins` floors set.
5. **Write the heart chapter — *The Child With Nobody* — as chapter 1.**
   Insert at 01 (manifest edit → `plan` → `apply` shifts the fourteen by one;
   map `2026-09-16-heart.json`); chapter prose (D3 shape) + paired pins file +
   suites case listing the fixture + ledger rows + opening-note contents/
   glossary/index/domain entries; `claim_discipline` and coverage green. *Done
   when:* `check`; `cargo test`; `./verify.sh`; `reader-coverage.md`
   regenerated; the chapter contains no digit and no predicate of state.
6. **Open the book on the argument.** Opening note: the argument block (D4),
   `### The child and the prisoner` replacing "Why the prisoner appears so
   early", line 18 back-reference, `:172`; Book 2 named on the first page;
   Part V `:556`/`:836` and `method.md:942` seam sentences; recorded as
   `session-drafted, author-approved under delegated approval (2026-09-16)`.
   *Done when:* navigation test green; `cargo test`; `./verify.sh`; no arrival
   verb, no guarantee-of-growth claim (register audit before commit).
7. **Rule the reading order and renumber once to the final table.** Manifest
   gets `rule` and the D1 table with `planned` entries; `plan` → map
   `2026-09-16-reorder.json` → `apply` → `check`;
   `derived_chapters_run_engines_before_breaks` added; `3-spine.md` header
   corrected, generated contents block, old hand list re-headed "superseded
   2026-09-16 — pre-reorder numbers"; opening note 22-24 and 729-741 rewritten
   to the rule; `method.md:221-253` rewritten (the stratification is the
   derivation record; the reading order is ruled); `book-1/README.md:8-11`,
   `README.md:44`; the three forward cross-references the validator reports
   are reworded; regenerated reports. *Done when:* `check`; `cargo test` (incl.
   `spine_contents_block_is_current`); `./verify.sh`; `bash combine.sh` yields
   the landed chapters in manifest order.

### Phase B — the engines (one chapter per item; unrendered mass first, then a receiving chapter before its host is cut)

Each Phase B item: chapter + pins + suites + ledger rows + child slot +
opening-note entries + host cut + `UNRENDERED_FAMILIES`/`THIN_POSTURES`
membership updated + regenerated reports + `cargo test` + `./verify.sh`.

8. **09 — Earning Above the Floor** (NEW; cuts 08:758-779, 10:95-114; closes
   INCOME-SECURITY, QUALIFICATIONS-COMPENSATION, half of ECONOMIC).
9. **11 — What Money Cannot Buy** (NEW; cuts 08:781-800; closes ECONOMIC,
   PUBLIC-SCALE-VOCABULARY).
10. **Implement the substantive-equality ordinary half** (`equality-source.json`,
    `src/authoring/equality.rs` from `family_life.rs` by substitution, block
    `SUBSTANTIVE-EQUALITY-ORDINARY`, contract card, the cases in D5;
    `CHALLENGE_SKELETON` gains the family). Interface first, passage second.
11. **12 — The Same Route for Everyone** (NEW; cuts 08:637-661; closes
    SUBSTANTIVE-EQUALITY).
12. **08 — What Nobody Has to Ask Permission For** (PROMOTED + liberties
    section; half of LIBERTY-ECOLOGY).
13. **13 — A Place in Which Life Remains Possible** (PROMOTED + Class 9
    section; closes LIBERTY-ECOLOGY).
14. **05 — Whether It Arrived** (PROMOTED from 08 and 11).
15. **06 — When There Is Genuinely Not Enough** (PROMOTED).
16. **07 — Who Owes, and What Follows** (PROMOTED from 08 and 14; receipt
    `unread-duty` moves only if its phrase moves).
17. **15 — Arriving and Belonging** (PROMOTED).
18. **14 — Holding a Role in Somebody's Life** (PROMOTED).
19. **17 — How Public Power Is Built** (PROMOTED + new state-form sections).
20. **19 — What May Be Kept About You** (PROMOTED from 01).
21. **20 — A Crisis Does Not Suspend the Republic** (PROMOTED).
22. **21 — A Way to Be Heard** (PROMOTED).
23. **02 — Who Counts** (PROMOTED from 07; opens by generalising chapter 1's
    birth root to the other three; may add a `records/first_contact_*` case).
24. **The child slot in every kept engine chapter** (03, 04, 16, 18, 22; the
    exempt list 10/23 with reasons) and the membership tests
    (`CHILD_SLOT_CHAPTERS`, fixture listing, anti-monoculture child form).

### Phase C — the breaks, read last

25. **The child slot in the break chapters** (25, 27, 28, 29, 30 incl. the
    Undelivered case; 24/26 exempt with reasons); Part V's coercion joint
    sentence and the pair in its family section.
26. **Close the rebalance.** `breaks_come_after_engines` coercive-share bound
    and `every_family_states_a_boundary_where_it_is_rendered` landed;
    `UNRENDERED_FAMILIES` empty; `THIN_POSTURES` re-censused; length invariant
    re-measured (`wc -w`, appendix and source excluded) and recorded; R6
    re-measurement recorded; CLAUDE.md implementation-supersession notes on
    R1/R3/R4/R5; no `planned` entry left in the manifest.

## Verification for every item

- `cargo test --release --bin generate`, then `./verify.sh` (complete; ~15
  minutes; nine known-defect pins still reproduce), sequentially. After a
  renumber or move additionally `python3 tools/relocate.py check <map>`,
  `./generate.sh spine`, the three report generators byte-stable on a second
  run, `bash combine.sh`, and a pilot build into its temp directory.
- Prose items get a register audit before commit: no arrival verbs, no
  guarantee-of-growth or operation claims, no interior state, no digits, no
  cast name in a passage.
