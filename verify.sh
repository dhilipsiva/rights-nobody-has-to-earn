#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# verify.sh — the one check for book-1. First failure wins.
#
# This project's recurring failure mode is a hand-maintained number going stale:
# the spine went stale twice, the evidence count moved from 21 to 22 undetected,
# a pin-file NOTE drifted from its own file, and a proportion cap was computed
# against a spine that did not exist. Every check below exists because one of
# those actually happened.
#
#   ./verify.sh          full run: every declared chapter/floor pin; reviewed
#                       record, temporal, amendment, placement, reader, and
#                       counterfactual suites.
#   ./verify.sh --quick  everything except the executable suites
#                       NOTE --quick cannot execute chapter/floor pins, record
#                       snapshots, temporal transitions, amendment candidates,
#                       placement cases, reader-evaluator fixtures, or stale
#                       counterfactual fixtures. Run the
#                       FULL suite after any constitution edit — that is how a
#                       stale fixture shipped once.
#
# NEVER use nibli-host: its wasm predates the derived_only and entitled corpus
# entries, so it silently drops the rights floor and every conclusion-only gate
# and then reports a clean run over a constitution it is not reading.

set -uo pipefail
cd "$(dirname "$0")"

# NIBLI_PIN set explicitly means "use exactly this binary" — the engine rebuild
# below is skipped, because overriding the path is how you deliberately test an
# older or patched build. Unset means "use the checkout", and then this script is
# responsible for the binary matching the source.
NIBLI_PIN_OVERRIDDEN=0
[ -n "${NIBLI_PIN:-}" ] && NIBLI_PIN_OVERRIDDEN=1
NIBLI_SRC="${NIBLI_SRC:-$HOME/projects/dhilipsiva/nibli}"
PIN="${NIBLI_PIN:-$NIBLI_SRC/target/release/nibli-pin}"
KB=new-book-plans/constitution.nibli
SPINE=new-book-plans/3-spine.md
CF=new-book-plans/counterfactual
RED_TEAM=new-book-plans/9-record-integrity-red-team.py
AMENDMENT_AUDIT=new-book-plans/10-amendment-semantics.py
PLACEMENT_AUDIT=new-book-plans/11-placement-exhaustiveness.py
TEMPORAL_AUDIT=new-book-plans/12-temporal-assurance.py
LEDGER_AUDIT=new-book-plans/13-full-society-ledger.py
CLOSURE_AUDIT=new-book-plans/16-constitutional-closure.py
POWER_SOURCE_AUDIT=new-book-plans/17-full-society-power-source-manifest.py
READER_AUDIT=new-book-plans/14-reader-evidence.py
READER_GATE=new-book-plans/reader-evidence-admission-gate.py
PILOT_READER_ARTIFACTS=new-book-plans/15-pilot-reader-artifacts.py
QUICK=0
ONLY=""
case "${1:-}" in
  --quick) QUICK=1 ;;
  --only)  ONLY="${2:-}" ;;
  --table) exec python3 new-book-plans/6-claim-table.py ;;
  "")      ;;
  *)       printf 'usage: ./verify.sh [--quick | --only <pinfile> | --table]\n' >&2; exit 2 ;;
esac

pass() { printf '  \033[32mok\033[0m   %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$1"; [ -n "${2:-}" ] && printf '       %s\n' "$2"; exit 1; }
step() { printf '\n\033[1m%s\033[0m\n' "$1"; }

# THE BINARY MUST MATCH THE SOURCE, AND THIS IS WHY THE SCRIPT DOES IT.
# On 2026-07-31 nibli landed NAF materialisation. The commit was on main and
# materialize.rs was on disk, but target/release/nibli-pin was three days old, so
# this suite silently kept running the OLD engine — same green result, same 47
# minutes, and the obvious conclusion would have been that the upstream work did
# nothing. A stale binary is invisible: every pin still passes, because the pins
# check the constitution, not the engine.
#
# Incremental and free: ~0.2 s when nothing changed, measured, against a ~29 min
# historical run. Quick mode builds too: the spine and assertion audit must read
# the same release engine that the full pin suite will use; the assurance case is
# then checked against that reviewed assertion contract.
# `--manifest-path` rather than `cd`, so this works from any directory.
#
# A FUNCTION because `--only` needs it too. A fast per-file loop that skipped the
# rebuild would be the stale-binary bug again, in the mode people run most.
build_engine() {
  step "engine"
  if [ "$NIBLI_PIN_OVERRIDDEN" = "1" ]; then
    pass "NIBLI_PIN set — using $PIN as-is, not rebuilding"
  elif [ -f "$NIBLI_SRC/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    out=$(cargo build --release --bin nibli-pin --manifest-path "$NIBLI_SRC/Cargo.toml" 2>&1) \
      || fail "nibli-pin failed to build" "$out"
    rev=$(git -C "$NIBLI_SRC" rev-parse --short HEAD 2>/dev/null || echo "not-a-git-checkout")
    dirty=$(git -C "$NIBLI_SRC" status --porcelain 2>/dev/null | head -1)
    pass "engine built from ${rev}$([ -n "$dirty" ] && echo ' +uncommitted')"
  else
    printf '  \033[33mwarn\033[0m no nibli source at %s — running whatever binary is there\n' "$NIBLI_SRC"
    printf '       set NIBLI_SRC, or NIBLI_PIN to silence this deliberately\n'
  fi
  [ -x "$PIN" ] || fail "no nibli-pin at $PIN" "build it release, or set NIBLI_PIN"
}

# ── --only: one pin file, for the iteration loop during a chapter pass ───────
# Every pin file used to document its own `nibli-pin --kb …` command. Thirteen of
# them worked while missing the cross-file reconciliation; TWO WERE SIMPLY BROKEN
# — chapters 8 and 14 carry `:require` preconditions, their documented command
# omitted `--allow-shell`, and pasting it gave exit 2 plus a pin-count mismatch
# that read as though the file had been edited. This mode exists so there is one
# command that cannot be pasted wrong: it passes the flag, it rebuilds the engine,
# and it picks the right knowledge base.
if [ -n "$ONLY" ]; then
  [ -f "$ONLY" ] || fail "no such pin file: $ONLY" "usage: ./verify.sh --only <pinfile>"
  case "$ONLY" in
    *.pins.nibli) ;;
    *) fail "not a pin file: $ONLY" "expected a path ending .pins.nibli" ;;
  esac
  # A counterfactual pin file is asked against ITS OWN constitution, not the real
  # one — that is the entire point of those fixtures, and running them against the
  # live file would invert every verdict in them.
  kb="$KB"
  case "$ONLY" in "$CF"/*) kb="${ONLY%.pins.nibli}.nibli" ;; esac
  build_engine
  step "pins — $ONLY"
  printf '  against %s\n' "$kb"
  "$PIN" --allow-shell --kb "$kb" "$ONLY"
  rc=$?
  # Propagate, including 3 (a pinned defect stopped reproducing). Swallowing that
  # here would hide the one outcome the :defect markers exist to announce.
  printf '\n\033[33mpartial\033[0m one file against one knowledge base. NOT checked: the\n'
  printf '        cross-file :expect-pins reconciliation, the spine, assertion audit,\n'
  printf '        record-integrity assurance case, bounded red-team contract, temporal assurance, amendment audit, placement audit, reader-evidence contract, power-source manifest, full-society ledger, or constitutional-closure audit,\n'
  printf '        the jargon sweep,\n'
  printf '        the counted-claims ratchet, the absence and arity guards, and whether\n'
  printf '        the counterfactual fixtures are stale. Run ./verify.sh before committing.\n'
  exit $rc
fi

# ── 1. the spine is regenerated from the constitution ────────────────────────
build_engine
STRATA_FILE=$(mktemp) || fail "could not create a strata cache"
trap 'rm -f -- "$STRATA_FILE"' EXIT

step "spine"
out=$(NIBLI_PIN="$PIN" NIBLI_STRATA_CACHE_OUT="$STRATA_FILE" python3 new-book-plans/5-spine-gen.py "$KB" "$SPINE" --check 2>&1) \
  && pass "3-spine.md is current" \
  || fail "3-spine.md is stale" "$out — rerun 5-spine-gen.py without --check"

# ── 2. every rule-produced and ground-writable relation has a contract ───────
step "assertion surface"
out=$(NIBLI_STRATA_FILE="$STRATA_FILE" python3 new-book-plans/7-assertion-surface.py --check 2>&1) \
  && pass "$out" \
  || fail "assertion-surface audit failed" "$out"

# ── 2a. record classes, assurance controls, and limits stay reviewable ────────
step "record-integrity assurance"
out=$(python3 new-book-plans/8-record-integrity-assurance.py --check 2>&1) \
  && pass "$out" \
  || fail "record-integrity assurance case failed" "$out"

# ── 2b. bounded record snapshots stay reviewed and digest-bound ─────────────
step "record-integrity red-team contract"
out=$(python3 "$RED_TEAM" --check 2>&1) \
  && pass "$out" \
  || fail "record-integrity red-team contract failed" "$out"

# ── 2c. amendment labels stay separate from candidate-source effects ─────────
step "amendment-semantics contract"
out=$(python3 "$AMENDMENT_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "amendment-semantics audit failed" "$out"

# ── 2d. placement combinations and source mutations stay reviewed ───────────
step "placement-exhaustiveness contract"
out=$(python3 "$PLACEMENT_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "placement-exhaustiveness audit failed" "$out"

# ── 2e. staged time candidates and exact source/effect bindings ─────────────
step "temporal-assurance contract"
out=$(python3 "$TEMPORAL_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "temporal-assurance contract failed" "$out"

# ── 2f. chapter 1's headline number ──────────────────────────────────────────
# Only the generated block is machine-owned. A new PREDICATE name (not a new
# ground fact) moves this and may falsify prose that describes the list.
n=$(grep -o 'Evidence predicates ([0-9]*)' "$SPINE" | grep -o '[0-9]*')
# 2026-08-18: 41 -> 42. `receives` joined the record as the recipient-side
# half of a delivery. Chapters 1, 3 and 5 were re-read against the list: they
# describe what the record may hold in ordinary English and never the relation
# name, which the jargon sweep guarantees, so no prose claim moved with it.
# 2026-08-17: 40 -> 41. `approves` split into `ratifies` (amendment tally) and
# `endorses` (case-bound custody review), so one name became two. Chapters 1, 3,
# 5, 12 and 13 were re-read against the list: they describe the SENSE in
# ordinary English ("the electorate approves it", "separate routes must approve
# it") and never the relation name, which the jargon sweep guarantees, so no
# prose claim moved with the split.
[ "$n" = "42" ] && pass "evidence vocabulary is 42" \
  || fail "evidence vocabulary is $n, not 42" "chapters 1, 3 and 5 describe the list; re-read them against it"

# ── 2g. the dormant reader contract stays structural and privacy-minimal ─────
# This is an artifact check. It is not R6's dedicated executable evidence gate,
# external custody/freshness attestation, reviewer custody, or seeded control.
step "reader-evidence contract"
out=$(python3 "$READER_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "reader-evidence contract failed" "$out"

# Component check in both quick and full modes. A passing self-test proves only
# that the fixed gate can reject its own mutations and emit its exact interface;
# it neither supplies a gate_admission_receipt nor makes R6 built or available.
step "reader-evidence admission-gate component"
out=$(python3 "$READER_GATE" --self-test 2>&1) \
  && pass "$out" \
  || fail "reader-evidence admission-gate self-test failed" "$out"

# Structural generation runs in both quick and full modes and writes only to a
# temporary directory. PDF rendering and human screen-reader attestation remain
# external prerequisites; this check does not create or attest either one.
step "pilot reader artifacts"
out=$(python3 "$PILOT_READER_ARTIFACTS" --check 2>&1) \
  && pass "$out" \
  || fail "pilot reader artifacts failed" "$out"

# ── 2h. source-identified public powers stay census-bound ────────────────
# Inventory only: exact source digests, one source anchor per row, closed
# dispositions, reviewed totals, and watched-failing mutations. It creates no
# FS-POW contract card, lawful holder, operation, assurance, or Gate A result.
step "full-society power source manifest"
out=$(python3 "$POWER_SOURCE_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "full-society power source manifest failed" "$out"

# ── 2i. the full-society domain-and-layer ledger stays reviewed and generated ─
# Structural only: schema, enum-mapping closure over the sibling reviewed
# JSONs (read live, not digest-pinned), split/posture/compatibility rules,
# needles, bound-decision digests, and the ledger's own negative controls. It
# executes no engine query and establishes no row beyond its own posture —
# classification is routing, not assurance.
step "full-society ledger"
out=$(python3 "$LEDGER_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "full-society ledger failed" "$out"

# 2j. claim-scoped constitutional closure and model allocation
# Structural only: computes pass, block, or bounded-unresolved from the
# reviewed canonical source. A pass upgrades no claim posture and establishes
# no delivery, liveness, feasibility, operation, or Gate A closure.
step "constitutional-closure and model-allocation audit"
out=$(python3 "$CLOSURE_AUDIT" --check 2>&1) \
  && pass "$out" \
  || fail "constitutional-closure audit failed" "$out"

# ── 3. no formalism leaks into Parts I-V ─────────────────────────────────────
step "prose"
# The pattern is wider than the tracker's original. Negative-controlled: the old
# one missed "stratifier" — the likeliest leak of all, since it is the word the
# tracker itself uses constantly. `strat` alone is too greedy (it matches
# "demonstrate"), hence the three explicit stems.
JARGON="nibli|predicat|stratum|strata|stratif|compil|assert|rule head|quantif|derivation|knowledge base|first-order|negation|conjunct"
if hits=$(grep -rniE "$JARGON" book-1/[0-9]*.md); then
  fail "jargon in a derived chapter" "$hits"
else
  pass "jargon sweep clean across $(ls book-1/[0-9]*.md | wc -l) chapters"
fi

# ── 3b. counted claims in the prose — a RATCHET, not a gate ──────────────────
# Every counting claim in this book that has been checked was WRONG, not merely
# stale: "every convicted person" named four of seven; "one person verifiably has
# shelter" was four; "everything else on both lists is identical" differed in four
# places; "the four cases exhaust the combinations" covered seven of eight. The
# design also keeps moving — the evidence list went 21 -> 22 in one commit, the
# floor may stop being eight, and more than one thing may become takeable — so a
# number in the prose is a maintenance liability with a bad accuracy record.
#
# The rule is NOT "delete numbers", which only makes the prose vaguer. It is
# **state the rule that produces the count**: "shelter derives for every confined
# person and for nobody else" beats "four people have shelter" — stable under cast
# changes, more informative, and it is what the book is actually about.
#
# HARD GATE since 2026-08-02: the ratchet ran 14 -> 0 across the chapter passes
# and any counted claim is now a failure outright. Two allowlisted exceptions:
# rhetorical durations, which are not claims about this design, and chapter 13's
# title — "The One Thing Taken" IS the single-deprivation claim, kept on purpose
# where the whole chapter defends it; the allowlist pins it to line 1 of that one
# file, so the phrase reappearing anywhere else (including ch13's own body) counts.
COUNTED='\b(eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|hundred)([- ][a-z]+)?\b|\beight\b|\bexactly one\b|\bone thing\b|\bsingle deprivation\b'
COUNTED_OK='(thirty|forty|fifty|hundred) (year|second|minute|mile)|^book-1/13-the-one-thing-taken\.md:1:# The One Thing Taken$'
n=$(grep -rniE "$COUNTED" book-1/[0-9]*.md | grep -viE "$COUNTED_OK" | wc -l)
if [ "$n" -gt 0 ]; then
  fail "counted claims in the prose: $n (the gate is zero)" \
"$(grep -rniE "$COUNTED" book-1/[0-9]*.md | grep -viE "$COUNTED_OK" | tail -5)
       State the rule that produces the count, do not count the instances."
else
  pass "counted claims at zero (hard gate; ch13's title is the allowlisted exception)"
fi

# ── 3c. every pin is reachable from a claim comment ──────────────────────────
# The claim-to-query table (./verify.sh --table) is extracted, not generated:
# a query's claim is the nearest preceding comment block in its pin file.
# Measured 2026-08-02: all queries reachable, none inheriting header or
# taxonomy boilerplate — this check keeps extraction possible as suites grow.
out=$(python3 new-book-plans/6-claim-table.py --check 2>&1) \
  && pass "$out" \
  || fail "a pin query has no reachable claim comment" "$out"

# ── 3d. the claim registry ───────────────────────────────────────────────────
# Schema plus the staleness gate (registry/check.py) — a fetchable entry whose
# retrieval date fell behind fails here and names the script that refreshes it.
if [ -f registry/claims.json ]; then
  out=$(python3 registry/check.py 2>&1) \
    && pass "$out" \
    || fail "registry check failed" "$out"
fi

# ── 4. the absence claims ────────────────────────────────────────────────────
# Each is a load-bearing sentence that no query can hold. Test the rule BODIES:
# a bare grep also matches the predicate's own rule head, which is a check that
# can never fail. The positive control proves the command still works.
step "absences (nothing reads these)"
body() {
  awk -F'->' -v p="$1" '
    /^[^#]/ && /->/ {
      if ($1 ~ ("(^|[^[:alnum:]_])" p "\\(")) print NR ": " $0
    }
  ' "$KB"
}
ctl=$(body 'false' | wc -l)
[ "$ctl" -ge 1 ] || fail "absence check is broken" "positive control /false/ returned 0 lines; it should return 5"
pass "positive control: /false/ appears in $ctl rule bodies"
# `reward` is here by DECISION, not by observation: recognition feeds nothing, so
# it can never become an input to a ranking of persons. See the counting guard
# below — the two together are what "recognition is never ranked" reduces to.
#
# `building` is a decision too, and a different kind from the rest of this list.
# The others record that nothing happens to follow from a fact. This one is
# chapter 13's central admission made checkable: the design fixes WHERE a
# convicted person is and says nothing about what happens there, and the proof
# of that is that the placement is read by nothing. Note what it cuts against —
# if conditions vocabulary is ever admitted, a rule granting facility-specific
# protections would legitimately read `building` and this guard would fail. That
# is intended. Removing it should be a deliberate act with the chapter rewritten
# in the same commit, not a quiet consequence of some other edit.
# `err` LEFT this list in v0.8 and `obliged` took its place. The audit now feeds
# an obligation (Article 8b), so "nothing reads err" is false by design — chapter
# 14 says so. What still has to hold is that the chain ENDS: nothing reads the
# duty either. If `obliged` ever gains a reader the design has grown a third
# link, and chapter 14's closing argument has to be rewritten around it rather
# than quietly outgrown. That is what this line is guarding.
for p in owe become travel obliged lose reward building; do
  n=$(body "$p" | wc -l)
  [ "$n" = "0" ] && pass "nothing reads $p" \
    || fail "$p is now read by a rule" "$(body "$p") — the prose saying nothing follows from it is now false"
done
# Recognition carries no numeric quantity anywhere. Digits inside a typed
# identifier such as T3LifeCourseNonborrowing or Book2LifeCourseBoundary name a
# source contract; they are not numeric literals and must not trip this guard.
numeric_literal_re='(^|[^[:alnum:]_])[0-9]+([^[:alnum:]_]|$)'
if grep -vE '^\s*#' "$KB" | grep -qE "$numeric_literal_re"; then
  fail "a numeric literal appeared in an enacted line" "chapter 10 turns on there being no arithmetic"
else
  pass "no numeric literals in the constitution"
fi

# ── 4c. INVARIANT 1: a floor right may be noticed, never acted on ────────────
# The constitution states this at :103 and until v0.8 nothing enforced it — and
# the version nobody enforced was WRONG. It said a floor predicate must appear in
# no rule body at all, and Article 6's isolation marker has broken that since
# v0.1: `prisoner($p) & ~meets($p) -> err($p, Isolation)` reads `meets`. The
# blanket ban also forbade the one thing that marker does, which is notice that a
# floor right did not arrive — the only delivery check in the file.
#
# So the rule is narrower and this enforces the narrow form: a floor right may be
# read ONLY into `err`. Noticing is allowed; acting on it is not.
#
# The stratifier does NOT make this check for you. It refuses `~eats -> prisoner`
# because that is a negative cycle through the person cone, and it accepts
# `~eats -> reward` and `~eats -> building` quite happily. Those are the rules
# this guard exists for, and nothing else in the suite would see them.
step "the floor is noticed, never acted on (INVARIANT 1)"
for p in secure eats dwell healthy learn expresses believe meets; do
  bad=$(awk -F'->' -v p="(^|[^a-z])$p\\\\(" '/^[^#]/ && /->/ && $1 ~ p && $2 !~ /err\(/ {print NR": "$0}' "$KB")
  [ -z "$bad" ] || fail "$p is read into something other than err — INVARIANT 1" \
    "$bad
       A floor right may be noticed (head err) and never acted on. See constitution :103."
done
noticed=$(awk -F'->' '/^[^#]/ && /->/ && $2 ~ /err\(/ && $1 ~ /(^|[^a-z])(secure|eats|dwell|healthy|learn|expresses|believe|meets)\(/ {c++} END{print c+0}' "$KB")
[ "$noticed" -ge 1 ] || fail "the noticing check is vacuous" \
  "no floor right is read into err at all, so the guard above proves nothing — \
Article 6's isolation marker should be one"
pass "floor rights reach only err ($noticed such rule, none reaching a consequence)"

# ── 4a. `reward` is one place wide, and that is a refusal ────────────────────
# Provenance on recognition — `reward($teacher, $student)`, so a clawback could
# reach what came from the fraud and leave the rest — was REFUSED 2026-08-01, not
# deferred. The grounds are in CLAUDE.md and hold on both branches of the open
# Article 4 clawback fork.
#
# NOTHING ELSE HERE CAN SEE IT ARRIVE, which is the entire reason this exists.
# Rewriting the three heads is caught by the pin suite, but only as "reward(Esa)
# regressed" — and it makes five OTHER reward pins vacuous in the same edit, since
# a FALSE pin against a relation that has stopped existing still reads FALSE.
# The additive shape is worse: derivation is monotone, so a FOURTH head at arity 2
# added beside the three leaves `reward/1` deriving, and the absence loop above and
# the whole pin suite stay green. Verified both ways before this was trusted.
#
# The mention count is not decoration. "nothing reads reward" passes just as
# cleanly against a `reward` that has been renamed out of the file, so one check
# de-vacuums the other.
arity2() { awk -v p="$1" '/^[^#]/ && $0 ~ "(^|[^a-z_])" p "\\([^)]*," {print NR": "$0}' "$KB"; }
n=$(awk '/^[^#]/ && $0 ~ "(^|[^a-z_])reward\\(" {c++} END {print c+0}' "$KB")
[ "$n" -ge 1 ] || fail "the reward guards are vacuous" \
  "no enacted line mentions reward at all — 'nothing reads reward' above is proving nothing"
ctl=$(arity2 'judge' | wc -l)
[ "$ctl" -ge 1 ] || fail "the arity guard is broken" \
  "positive control /judge(_, _)/ returned 0; the multi-sig rule carries two places"
bad=$(arity2 'reward')
[ -z "$bad" ] && pass "reward is one place wide ($n enacted mentions; control saw $ctl two-place judge)" \
  || fail "reward has grown a second place — provenance on recognition is refused" \
          "$bad
       See CLAUDE.md: refused on grounds that hold on both sides of the Article 4 fork."

# ── 4b. no counted degree on the reward side ─────────────────────────────────
# The digit ban above stops a quantity being WRITTEN. This stops one being
# COUNTED, which is the cheaper route and the one chapter 10 points at directly:
# degree needs no arithmetic here, only the same relation twice with the two
# objects held apart. Severity is built exactly that way (:452-453), and the
# cast already supplies the reward-side mirror — Cira has two teachers — so
# "count what a person was taught" is one rule away and always has been.
#
# Chapter 10 now says that will not be built, so this is where that stops being
# a sentence. It is a PATTERN guard on the zero-vocabulary route, not a proof: a
# rule may still count by inventing a new name for a learning record (`studies`
# is in the corpus, unused). That route is caught upstream instead — a new
# evidence name moves the count off 24 and trips the check at the top of this
# file. Between them the two doors are covered; neither alone is.
selfjoin() { awk -F'->' -v p="$1" '/^[^#]/ && /->/ { s=$1; n=gsub(p, "", s); if (n>=2) print NR": "$0 }' "$KB"; }
ctl=$(selfjoin 'judge' | wc -l)
[ "$ctl" -ge 1 ] || fail "the counting guard is broken" "positive control /judge/ returned 0; the multi-sig rule joins it twice"
pass "positive control: /judge/ is joined with itself in $ctl rule"
for p in 'teaches' 'work[(]'; do
  n=$(selfjoin "$p" | wc -l)
  [ "$n" = "0" ] && pass "no rule counts ${p%%[*} entries" \
    || fail "${p%%[*} is now joined with itself — that is counted degree on the reward side" \
            "$(selfjoin "$p") — chapter 10 says this will not be built"
done

# ── 4d. a control must not pollute the base below it ─────────────────────────
# `:accept` leaves its statement in the KB, so every pin below one runs against a
# widened base. That produced a real vacuous green here: with the four complement
# controls loaded, `? prisoner(Adam).` passed with Article 6's conviction rule
# DELETED outright — re-measured against a sabotaged copy of the constitution on
# 2026-08-01, silent under `:accept` and caught under `:accept-scoped`. The
# workaround was to order every file so its controls came last, which is a rule
# nobody can see being broken.
#
# nibli 425567b made the scope real, so the ordering rule is replaced by this
# check. Be exact about what it covers: it reads the SPELLING of the directive
# and nothing more. The retraction is the engine's, and one that fails is a
# harness error (exit 2), not a pin that goes quietly green — so this section
# cannot tell you the base was put back, only that the file asked for it.
#
# FIVE FILES KEEP THE PLAIN FORM DELIBERATELY, and they are allowlisted rather
# than fixed, because in them an accepted statement is a PREMISE the file goes
# on to query. Chapter 1 writes a roster entry; chapter 5 keeps its closing
# no-distinctness rule resident; chapter 14 keeps the duty-breach rule resident;
# and chapter 9 keeps its disenfranchisement rule resident. Chapters 6 and 9
# also keep exact-name `admits` declarations resident so their following
# refusals prove `lose` and `decide` stay closed after the real widening edit,
# not merely because the names were omitted. Retract those premises and the
# queries or refusal controls below them have nothing to stand on. Anything NOT
# on this list is a control, and a control that does not clean up is the bug this
# section exists to catch.
unscoped_ok="01-what-counts-as-evidence 05-voiding 06-clawback 09-the-vote-conviction-does-not-take 14-when-the-system-notices-it-broke"
ctl=0; bad=""
for f in new-book-plans/*.pins.nibli new-book-plans/counterfactual/*.pins.nibli book-1/*.pins.nibli; do
  grep -q '^:accept$' "$f" || continue
  b=$(basename "$f" .pins.nibli)
  case " $unscoped_ok " in
    *" $b "*) ctl=$((ctl + 1)) ;;
    *) bad="$bad$f: $(grep -c '^:accept$' "$f") unscoped
" ;;
  esac
done
# POSITIVE CONTROL — the allowlisted files must actually still match `^:accept$`.
# Without this, a change to the pin syntax makes the loop above match nothing and
# report every file clean, which is the exact shape of green this repo keeps
# getting fooled by.
[ "$ctl" = "5" ] || fail "the control-scope guard is broken" \
  "expected every allowlisted file to carry a plain :accept; matched $ctl of 5. Either the syntax moved or a premise site was converted."
[ -z "$bad" ] && pass "every control is written :accept-scoped (the engine, not this check, puts the base back)" \
  || fail "a control leaves its statement in the KB — every pin below it runs against a widened base" \
          "$bad
Write these :accept-scoped, or allowlist the file above if the statement is a premise a later query needs."

# ── 5. the pin suites ────────────────────────────────────────────────────────
if [ "$QUICK" = "1" ]; then
  printf '\n\033[33mskipped\033[0m chapter/floor pins, record snapshots, temporal, amendment, placement and reader-evaluator executions, and counterfactuals (--quick)\n'
  exit 0
fi

# The dormant pre-pilot source carries no threshold values. Execution exercises
# the state machine, evaluator, and watched-failing mutations required by each
# populated candidate, author-ratified, frozen, or completed stage; required
# missing, empty, or falsely inapplicable controls fail closed. Once values
# exist, the same command derives end-to-end below-, exact-, and above-boundary
# fixtures at reachable observations; edge-domain cases stay explicit and
# fail-closed. No illustrative threshold is committed in advance. These remain
# artifact controls (Checked), not reader evidence or a Derived claim, and they
# do not make R6 built or available.
step "reader-evidence evaluator controls"
out=$(python3 "$READER_AUDIT" --check --execute 2>&1) \
  && pass "$out" \
  || fail "reader-evidence evaluator controls failed" "$out"

step "chapter/floor pins"

declared=$(grep -h ':expect-pins' new-book-plans/rights-floor.pins.nibli book-1/*.pins.nibli | awk '{s+=$2} END {print s}')
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/rights-floor.pins.nibli book-1/*.pins.nibli 2>&1)
# `pins ?[,(]` not `pins,` — a file carrying :defect markers prints "15 pins (1
# defects)," with a SPACE before the paren, and the older pattern silently dropped
# exactly those files from the display while the reconciliation below still passed.
# Caught by reading the raw output; the first attempt at this fix used `pins[,(]`
# and also matched nothing, for the same off-by-one-character reason.
echo "$out" | grep -E 'pins ?[,(]|✗' | sed 's/^/  /'
echo "$out" | grep -q 'FINDING\|HARNESS ERROR' && fail "a pinned property regressed"

# A PINNED DEFECT THAT STOPS REPRODUCING IS THE BEST OUTCOME HERE, and until
# 2026-08-01 this script gave it the worst diagnosis it has. nibli-pin exits 3 and
# prints a RESOLVED DEFECTS block naming the repair; that block contains neither
# `FINDING` nor `PASS —`, so the regression grep above passed it through and the
# PASS-line parse below reported "the suite did not report a PASS line" under the
# hint "a file was added or dropped from the run". The pins carrying `:defect`
# markers are declared defects — a count here rots (this comment once said twelve;
# the live number was nine); the markers themselves are the list — and this is the
# outcome `:defect` was adopted FOR, yet it read as a broken harness. Reproduced before fixing: a `:defect` marker on a query that in fact
# holds produces exactly that.
if echo "$out" | grep -q 'NO LONGER REPRODUCE'; then
  printf '  \033[32m✓\033[0m    a pinned defect no longer reproduces — the artifact improved\n'
  echo "$out" | grep -E '^\s*✓' | sed 's/^/  /'
  fail "update the pin and the prose that describes it" \
       "This is a REPAIR, not a regression. Find the chapter that calls this a flaw and rewrite it, then drop the :defect marker in the same commit."
fi

ran=$(echo "$out" | sed -n 's/.*PASS — \([0-9]*\) pins.*/\1/p')
[ -n "$ran" ] || fail "the suite did not report a PASS line"
[ "$ran" = "$declared" ] && pass "$ran pins, 0 findings — matches the sum of :expect-pins" \
  || fail "ran $ran pins but the files declare $declared" "a file was added or dropped from the run"

# ── 5b. universal-standing non-power family ─────────────────────────────────
# Kept outside the historical chapter/floor total so the 555-pin baseline stays
# an exact regression signal. These pins execute the new birth, first-contact,
# jurisdiction, effective-control, identity-boundary, and public-body controls.
step "universal-standing executions"
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/universal-standing.pins.nibli 2>&1) \
  && pass "$out" \
  || fail "universal-standing execution failed" "$out"

# 5c. liberty and ecological non-power family
# This suite proves normative barriers only. It supplies no measurement,
# institution, operation, delivery, restoration, or T3 evidence.
step "liberty and ecological executions"
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/liberty-environment.pins.nibli 2>&1) \
  && pass "$out" \
  || fail "liberty and ecological execution failed" "$out"

# 5d. substantive-equality and anti-subordination non-power family
# This executes normative barriers only. It supplies no population statistics,
# identity authentication, programme operation, delivery, or T3 evidence.
step "substantive equality executions"
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/substantive-equality.pins.nibli 2>&1) \
  && pass "$out" \
  || fail "substantive equality execution failed" "$out"

# 5e. bodily-autonomy, care, family, and life-course non-power family
# This executes source-bound normative barriers and typed evidence routes only.
# It supplies no clinical finding, service, operation, delivery, or T3 evidence.
step "delivery and receipt executions"
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/delivery-receipt.pins.nibli 2>&1) \
  && pass "$out" \
  || fail "delivery and receipt execution failed" "$out"

# ── 5f. bodily-autonomy, care, family, and life-course non-power family
step "family and life-course executions"
out=$("$PIN" --allow-shell --kb "$KB" new-book-plans/family-life-course.pins.nibli 2>&1) \
  && pass "$out" \
  || fail "family and life-course execution failed" "$out"

# ── 6. bounded record-snapshot red-team ─────────────────────────────────────
# These ephemeral KBs exercise additions, exact deletions, two-entry matrices,
# and the remaining single-snapshot record levers. They stay outside the chapter
# :expect-pins sum. A pass reproduces the reviewed flat-record harms and
# boundaries; staged transition, carry, order, and renewal evidence belongs to
# the temporal suite below.
step "record-integrity red-team snapshots"
out=$(NIBLI_PIN="$PIN" python3 "$RED_TEAM" --check --execute 2>&1) \
  && pass "$out" \
  || fail "record-integrity red-team execution failed" "$out"

# ── 6b. cumulative T1, T2, and T3 candidates ────────────────────────────────
step "temporal-assurance executions"
out=$(NIBLI_PIN="$PIN" python3 "$TEMPORAL_AUDIT" --check --execute 2>&1) \
  && pass "$out" \
  || fail "temporal-assurance execution failed" "$out"

# ── 6c. amendment-label/source-effect counterexamples ────────────────────────
# The audit manually applies reviewed exact candidate mutations in isolated
# copies and checks their consequences. It proves neither enactment nor an
# authenticated source transition.
step "amendment-semantics executions"
out=$(NIBLI_PIN="$PIN" python3 "$AMENDMENT_AUDIT" --check --execute 2>&1) \
  && pass "$out" \
  || fail "amendment-semantics execution failed" "$out"

# ── 6d. current placement matrix and bounded source mutations ────────────────
# The audit generates every declared subject state and severity/family/home
# case, including both non-confined mirrors, then proves that missing,
# duplicate, reversed, and painted housing mutations trip the reviewed source
# contract. It adds no runtime alarm.
step "placement-exhaustiveness executions"
out=$(NIBLI_PIN="$PIN" python3 "$PLACEMENT_AUDIT" --check --execute 2>&1) \
  && pass "$out" \
  || fail "placement-exhaustiveness execution failed" "$out"

# ── 7. the standing source counterfactuals ───────────────────────────────────
# Derivation is monotone and probe facts load on top, so no probe can test a
# restriction. These persistent fixtures execute the book's named standing
# source-removal/change/addition claims; the amendment audit above separately
# executes its reviewed ephemeral candidates. Regenerate after any KB edit.
step "counterfactuals"
# The staleness guard used to assign `d` and never read it, so a fixture that had
# drifted — or been regenerated by the README's own command, which matched nothing
# and produced a byte-identical copy — passed green while proving nothing. Compare
# CONTENT, not a line count: `grep -c ''` reads a delete-plus-add as no change.
# THREE CLASSES OF FIXTURE, and the diff shape is the fixture's identity:
#   1:0  a line deleted  — what the world loses without it
#   1:1  a line changed  — no-dead-conjuncts strips Article 4's
#        ~broken/~match(CarriedVoid)
#        conjuncts; chapters 4 and 5's OWN pin files must pass against it, which
#        is the standing proof those conjuncts decide nothing today (and the
#        proof strengthens automatically as those suites grow)
#   0:1  a line added    — unguarded-pen is the credential route somebody might
#        someday write; its pins show the kept conjuncts are the only thing
#        standing between that line and a carried-void signature counting; and
#        undelivered-marker is the delivery marker the 2026-08-02 ruling
#        refused, whose pins show it firing on everyone the record owes —
#        the measured reason it stays unbuilt while no arrival facts exist
# The shape check is not pedantry: the (1:1) class would read as "stale" to the
# old exactly-one-line rule, and a byte-identical copy — the README's own
# historical failure — reads as (0:0) and fails every class.
#
# NOTE these pins are OUTSIDE the :expect-pins reconciliation above, checked
# per-file here. Do not "fix" the headline sum to include them.
for spec in no-person-line:1:0 no-public-court:1:0 no-choose-boss:1:0             no-first-contact-standing:1:0 no-environmental-right:1:0             no-class9-climate-axis:1:0 no-direct-equality:1:0             no-equality-data-wall:1:0 no-positive-measure-end:1:0             no-automatic-adulthood:1:0 no-family-confinement-wall:1:0             no-missing-kinship-independence:1:0 no-pregnancy-authority:1:0             no-dead-conjuncts:1:1 no-delivery-independence:1:1 \
            unguarded-pen:0:1 undelivered-marker:0:1; do
  f=${spec%%:*}; want="${spec#*:}"
  removed=$(diff "$KB" "$CF/$f.nibli" | grep -c '^<')
  added=$(diff "$KB" "$CF/$f.nibli" | grep -c '^>')
  [ "$removed:$added" = "$want" ] \
    || fail "$f does not differ from the constitution the way its class requires" \
            "$removed removed, $added added; expected ${want%:*} removed, ${want#*:} added — regenerate it from $KB per $CF/README.md"
  case "$f" in
    no-dead-conjuncts) pins="book-1/05-voiding.pins.nibli book-1/04-the-shield.pins.nibli" ;;
    *)                 pins="$CF/$f.pins.nibli" ;;
  esac
  out=$("$PIN" --kb "$CF/$f.nibli" $pins 2>&1)
  echo "$out" | grep -q 'PASS' && pass "$f" \
    || fail "$f" "$(echo "$out" | tail -3) — is the fixture stale? regenerate it from $KB"
done

printf '\n\033[32mall checks passed\033[0m\n'
