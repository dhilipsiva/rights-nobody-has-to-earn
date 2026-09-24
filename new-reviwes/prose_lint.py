#!/usr/bin/env python3
"""Prose lint for the revision plan (section 6.3).

Usage:  python3 prose_lint.py chapters/*.md
Exit code 1 if any chapter breaks a threshold, so it can gate CI.
Edit the lists and thresholds below as decisions are made.
"""
import re
import sys

# Thresholds (per chapter)
MAX_NEG_PER_K = 20        # negations per 1,000 words (aim for 15)
MAX_JARGON_PER_K = 5      # terms of art per 1,000 words (book-wide now: ~13)
MAX_CASE_NAMES = 5        # distinct case names per chapter
MAX_DISCLAIMERS = 2       # "does not establish"-type sentences per chapter

NEG = r"\b(?:not|no|neither|nor|cannot|never|none)\b|n't\b"
JARGON = (r"\b(?:deriv\w*|supplied|qualif\w*|establish\w*|reader|effective|"
          r"window|lease|carr(?:y|ied)|selected current record)\b")
DISCLAIMER = (r"[^.]*(?:not establish|establish(?:es)? (?:no|neither)|"
              r"supplies no|proves? (?:no|neither))[^.]*\.")

# Terms that should not appear in chapter prose (allowed in "Run it" lines / Appendix B).
BANNED = [r"\bpins?\b", r"\bfixtures?\b", r"\bcounterfactual\w*", r"\bstratif\w*",
          r"\bthe model\b", r"\bthe checks\b", r"\bdwelling debt\b",
          r"\bsecurity debt\b", r"\bhealth debt\b", r"\bmaterial security\b",
          r"\bvoid(?:ed|ing|s)?\b", r"\brecognition loss\b"]
HARNESS = ["Targ4", "Targo", "Nogra", "Nogrb", "Partnr", "Frisk", "Sock", "Probe",
           "Vote_Probe", "Amend_Sneak", "Amend_Decoy", "Amend_Floor", "Ambi", "Solo",
           "Purga", "Hunch", "SchemeM", "PublicGuarantee", "HighSec", "Homestay"]
# Recurring cast (edit to the chosen stable cast); any capitalised name in
# CASE_NAMES counts toward the per-chapter limit.
CASE_NAMES = ["Nell", "Ori", "Bela", "Cira", "Ansel", "Coll", "Marlo", "Esa", "Fin",
              "Dev", "Nima", "Pico", "Ona", "Boss", "Rebel", "Gia", "Wren", "Vex",
              "Don", "Pax", "Sly", "Kel", "Rex", "Sena", "Lupo", "Mira", "Nia", "Ruk",
              "Hano", "Jala", "Ivo", "Lalo", "Adam", "Nando", "Zed", "Koa", "Hex",
              "Quin", "Tyr", "Opal", "Sata", "Yano", "Marisol", "Brix", "Dunya",
              "Quillon", "Harrow"] + HARNESS


def strip_markup(text):
    text = re.sub(r"```.*?```", " ", text, flags=re.S)       # code blocks
    text = re.sub(r"^\s*(?:Run it:|>).*$", " ", text, flags=re.M)  # allowed lines
    return text


def lint(path):
    raw = open(path, encoding="utf-8").read()
    text = strip_markup(raw)
    flat = re.sub(r"\s+", " ", text)
    words = max(1, len(flat.split()))
    per_k = lambda n: round(1000 * n / words, 1)
    neg = per_k(len(re.findall(NEG, flat, re.I)))
    jar = per_k(len(re.findall(JARGON, flat, re.I)))
    disc = len(re.findall(DISCLAIMER, flat, re.I))
    names = sorted({n for n in CASE_NAMES if re.search(rf"\b{re.escape(n)}\b", flat)})
    problems = []
    if neg > MAX_NEG_PER_K:
        problems.append(f"negations {neg}/1k > {MAX_NEG_PER_K}")
    if jar > MAX_JARGON_PER_K:
        problems.append(f"jargon {jar}/1k > {MAX_JARGON_PER_K}")
    if len(names) > MAX_CASE_NAMES:
        problems.append(f"{len(names)} case names > {MAX_CASE_NAMES}: {', '.join(names)}")
    if disc > MAX_DISCLAIMERS:
        problems.append(f"{disc} disclaimer sentences > {MAX_DISCLAIMERS}")
    for lineno, line in enumerate(text.splitlines(), 1):
        for pat in BANNED:
            for m in re.finditer(pat, line, re.I):
                problems.append(f"line {lineno}: banned term '{m.group(0)}'")
        for h in HARNESS:
            if re.search(rf"\b{re.escape(h)}\b", line):
                problems.append(f"line {lineno}: harness name '{h}'")
    summary = f"{path}: {words} words, neg {neg}/1k, jargon {jar}/1k, names {len(names)}, disclaimers {disc}"
    return summary, problems


def main(paths):
    failed = False
    for p in paths:
        summary, problems = lint(p)
        print(summary)
        for pr in problems[:40]:
            print("   -", pr)
        if len(problems) > 40:
            print(f"   ... {len(problems) - 40} more")
        failed |= bool(problems)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
