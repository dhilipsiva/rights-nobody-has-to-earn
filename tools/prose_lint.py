#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Prose measurement for Book 1, revision item 34.

Adapted from the revision plan's lint (new-reviwes/revision-plan-9.5.md, section
6.3). It measures reader load in each ordered input named by
book-1/contents.json: negations and terms of art per 1,000 words, distinct case
names, sentences that say something is not established, terms the plan keeps
out of chapter prose, and test-harness names. The measures are proxies for
reader load, not goals; none of them overrides meaning.

Every input is held to the plan's threshold or to its recorded figure,
whichever is looser, so a chapter may not get worse while it waits for its
rewrite. Improvements are recorded with --ratchet, which only lowers figures.

    python3 tools/prose_lint.py              report every ordered input
    python3 tools/prose_lint.py --check      fail on any regression
    python3 tools/prose_lint.py --ratchet    record figures that improved
    python3 tools/prose_lint.py --admit FILE record a new input's figures
    python3 tools/prose_lint.py FILE...      report the named files only

It is a development check, not a gate in verify.sh.
"""

import argparse
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "book-1" / "contents.json"
BASELINE = ROOT / "tools" / "prose_lint_baseline.json"

NEG = re.compile(r"\b(?:not|no|neither|nor|cannot|never|none)\b|n['’]t\b", re.I)
JARGON = re.compile(
    r"\b(?:deriv\w*|supplied|qualif\w*|establish\w*|readers?|effective|windows?|"
    r"leases?|leased|carr(?:y|ies|ied|ying)|selected current records?|pens?|"
    r"credentials?)\b",
    re.I,
)
DISCLAIMER = re.compile(
    r"[^.]*(?:not establish|establish(?:es)? (?:no|neither)|supplies no|"
    r"proves? (?:no|neither))[^.]*\.",
    re.I,
)
# Terms kept out of chapter prose; the plan allows them in "Run it" lines and
# the technical appendix.
# "material security" left this list when ruling D8 (2026-09-24, item 44) made
# it the canonical name of a floor item; the plan banned it only while the book
# used it loosely for what is now bodily safety.
BANNED = [
    r"\bpins?\b", r"\bfixtures?\b", r"\bcounterfactual\w*", r"\bstratif\w*",
    r"\bthe model\b", r"\bthe checks\b", r"\bdwelling debt\b", r"\bsecurity debt\b",
    r"\bhealth debt\b", r"\bvoid(?:ed|ing|s)?\b",
    r"\brecognition loss\b",
]
HARNESS = [
    "Targ4", "Targo", "Nogra", "Nogrb", "Partnr", "Frisk", "Sock", "Probe",
    "Vote_Probe", "Amend_Sneak", "Amend_Decoy", "Amend_Floor", "Ambi", "Solo",
    "Purga", "Hunch", "SchemeM", "PublicGuarantee", "HighSec", "Homestay",
]
FIXTURES = [
    "Provender", "Ledgerwitness", "Ledgerhouse", "Foundry", "Steward", "Assay",
    "Assayer", "Harrow", "Quillon",
]
# The recurring cast (item 51): at most fifteen people, whose facts stay the
# same wherever they appear. Every other person belongs to one home chapter
# and is described by role elsewhere; the opening's index may name everyone.
RECURRING = [
    "Nell", "Hano", "Ruk", "Bela", "Cira", "Marisol", "Esa", "Adam", "Ivo",
    "Kel", "Gia", "Wren", "Iris", "Tove", "Mael",
]
HOME = {
    "Ori": "01", "Marlo": "09", "Ansel": "09", "Coll": "09", "Nima": "10",
    "Pico": "10", "Ona": "10", "Quin": "10", "Sata": "10", "Yano": "10",
    "Koa": "16", "Nia": "21", "Faro": "24", "Pax": "24", "Lior": "24",
    "Dara": "24", "Sena": "24", "Dev": "25", "Edo": "25", "Mira": "25",
    "Tyr": "25", "Saba": "25", "Fin": "26", "Zed": "27", "Lalo": "28",
    "Nando": "28", "Opal": "28", "Jala": "29",
}
CAST = RECURRING + sorted(HOME)
# A case name is anything a reader must hold in mind as a particular: the cast,
# institutional fixtures and harness names alike. A harness name is counted
# once here and also listed where it occurs, because the count measures load
# and the list says where to repair it.
NAMES = sorted(set(CAST + FIXTURES + HARNESS))

# Thresholds from the plan, per kind of element. None means measured and
# reported but not held to a limit.
PROFILES = {
    "chapter": {"negation": 20.0, "jargon": 5.0, "names": 5, "disclaimers": 2,
                "banned": 0, "harness": 0, "strays": 0},
    "opening": {"negation": 20.0, "jargon": 5.0, "names": None, "disclaimers": 2,
                "banned": 0, "harness": 0, "strays": None},
    "method": {"negation": None, "jargon": None, "names": None, "disclaimers": None,
               "banned": None, "harness": None, "strays": None},
}
METRICS = ("negation", "jargon", "names", "disclaimers", "banned", "harness", "strays")


def ordered_inputs():
    """(path, profile) for every measured ordered input, in manifest order."""
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    out = []
    for name in manifest["front"]:
        if name == "00-opening-note.md":
            out.append((f"book-1/{name}", "opening"))
    for part in manifest["parts"]:
        for chapter in part["chapters"]:
            if chapter.get("file"):
                out.append((f"book-1/{chapter['file']}", "chapter"))
    for name in manifest.get("back", []):
        out.append((f"book-1/{name}", "method"))
    return out


def _blank(match):
    return "\n" * match.group(0).count("\n")


def measurable(raw):
    """The prose a reader reads, with every line kept in place.

    Code, comments, link targets, URLs, quotations and "Run it" lines are
    blanked rather than removed, so reported line numbers are the file's own.
    """
    text = re.sub(r"<!--.*?-->", _blank, raw, flags=re.S)
    text = re.sub(r"^[ \t]*(```|~~~).*?^[ \t]*\1[^\n]*$", _blank, text, flags=re.S | re.M)
    text = re.sub(r"^[ \t]*(?:Run it:|>).*$", "", text, flags=re.M)
    text = re.sub(r"!?\[([^\]]*)\]\([^)\s]*(?:\([^)\s]*\)[^)\s]*)*\)", r"\1", text)
    text = re.sub(r"<https?://[^>]+>", " ", text)
    text = re.sub(r"https?://\S+", " ", text)
    text = re.sub(r"`[^`\n]*`", " ", text)
    return text


def words(text):
    return [t for t in text.split() if re.search(r"[A-Za-z0-9]", t)]


def measure(raw, chapter=None):
    """Figures for one input. `chapter` is its two-digit number, when it is a
    numbered chapter, so a name outside its home chapter can be counted."""
    text = measurable(raw)
    flat = " ".join(text.split())
    count = max(1, len(words(text)))
    per_k = lambda n: round(1000 * n / count, 1)
    names = sorted(n for n in NAMES if re.search(rf"\b{re.escape(n)}\b", flat))
    locations = []
    banned = harness = 0
    for lineno, line in enumerate(text.splitlines(), 1):
        for pattern in BANNED:
            for m in re.finditer(pattern, line, re.I):
                banned += 1
                locations.append((lineno, "banned term", m.group(0)))
        for name in HARNESS:
            for m in re.finditer(rf"\b{re.escape(name)}\b", line):
                harness += 1
                locations.append((lineno, "harness name", m.group(0)))
    figures = {
        "words": len(words(text)),
        "negation": per_k(len(NEG.findall(flat))),
        "jargon": per_k(len(JARGON.findall(flat))),
        "names": len(names),
        "disclaimers": len(DISCLAIMER.findall(flat)),
        "banned": banned,
        "harness": harness,
        "strays": 0,
    }
    if chapter is not None:
        for name, home in HOME.items():
            if home != chapter and re.search(rf"\b{re.escape(name)}\b", flat):
                figures["strays"] += 1
                locations.append((0, "name outside its home chapter", name))
    return figures, names, sorted(locations)


def limit(profile, metric, recorded):
    """The figure an input may not exceed: the plan's threshold or its record."""
    threshold = PROFILES[profile][metric]
    if threshold is None:
        return None
    if recorded is None or metric not in recorded:
        return threshold
    return max(threshold, recorded[metric])


def regressions(path, profile, figures, baseline):
    recorded = baseline.get(path)
    out = []
    for metric in METRICS:
        bound = limit(profile, metric, recorded)
        if bound is not None and figures[metric] > bound:
            out.append(f"{metric} {figures[metric]} exceeds {bound}")
    return out


def load_baseline(path=BASELINE):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {}


def save_baseline(baseline, path=BASELINE):
    ordered = {k: baseline[k] for k in sorted(baseline)}
    Path(path).write_text(json.dumps(ordered, indent=2) + "\n", encoding="utf-8")


def record(figures):
    return {m: figures[m] for m in METRICS}


def ratchet(baseline, measured, admit=()):
    """Lower recorded figures that improved; add only the admitted inputs.

    Entries for inputs that left the manifest are dropped. A figure is never
    raised: a restructure admits the new file explicitly instead.
    """
    current = {path for path, _, _ in measured}
    updated = {k: v for k, v in baseline.items() if k in current}
    for path, profile, figures in measured:
        if path in updated:
            updated[path] = {m: min(updated[path].get(m, figures[m]), figures[m]) for m in METRICS}
        elif path in admit:
            updated[path] = record(figures)
    return updated


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("files", nargs="*")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--ratchet", action="store_true")
    parser.add_argument("--admit", action="append", default=[])
    parser.add_argument("--locations", action="store_true", help="list every flagged location")
    args = parser.parse_args(argv)

    inputs = ordered_inputs()
    if args.files:
        wanted = {str(Path(f)) for f in args.files}
        inputs = [(p, k) for p, k in inputs if p in wanted] + [
            (f, "chapter") for f in wanted if f not in {p for p, _ in inputs}
        ]
    baseline = load_baseline()
    measured = []
    failed = False
    for path, profile in inputs:
        chapter = Path(path).name[:2] if profile == "chapter" else None
        figures, names, locations = measure((ROOT / path).read_text(encoding="utf-8"), chapter)
        measured.append((path, profile, figures))
        problems = regressions(path, profile, figures, baseline)
        failed |= bool(problems)
        if args.check and not problems:
            continue
        print(f"{path} [{profile}]: {figures['words']} words, negation {figures['negation']}/1k, "
              f"jargon {figures['jargon']}/1k, names {figures['names']}, "
              f"disclaimers {figures['disclaimers']}, banned {figures['banned']}, "
              f"harness {figures['harness']}, strays {figures['strays']}")
        for problem in problems:
            print(f"   regression: {problem}")
        if args.locations:
            if names:
                print(f"   names: {', '.join(names)}")
            for lineno, kind, term in locations:
                print(f"   line {lineno}: {kind} '{term}'")
    if args.ratchet or args.admit:
        save_baseline(ratchet(baseline, measured, set(args.admit)))
        return 0
    return 1 if (args.check and failed) else 0


if __name__ == "__main__":
    sys.exit(main())
