#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Replay pins through a second engine, clingo (development tool, item 50).

The constitution is translated statement by statement into clingo's input
language and each selected inventory case's pin files are replayed through
it: facts asserted in order, each query answered against the stable model of
everything asserted so far. The tool reports every query whose clingo answer
differs from the verdict the pin file expects, which the Nibli verifier has
already confirmed.

What the translation does:
  - constants are quoted strings and `$variables` become clingo variables;
  - `event { eats() }` becomes the constant "event:eats";
  - `~atom` is negation as failure, `~($a = $b)` is `!=`, and `($a = $b)`
    is `=`;
  - a disjunction inside a rule body becomes one rule per disjunct;
  - `entitled(every person, event { P() })` becomes a rule from `person`;
  - `admits` and `derived_only` declarations are dropped, because clingo
    has no input refusal; refused statements are therefore never asserted.

What the comparison covers and what it does not:
  - clingo computes the stable model of the translated program, which for a
    stratified program is the same perfect model Nibli computes; a case
    whose program had several models or none would be reported as such;
  - each case is grounded once; every fact and accepted rule its fixtures
    and pins assert is guarded by an external atom of its own, switched on
    at its position, so each query sees exactly the statements asserted
    before it (declaring a fact itself external would be wrong, because
    clingo ignores that declaration for an atom a rule can also derive);
  - `:refuse`, `:accept-scoped` and `:require` statements are skipped, so
    refusals, scoped loadability and shell checks are not compared;
  - `:accept` statements are asserted, as Nibli keeps them;
  - contradiction scans are not compared;
  - only live-source cases run; counterfactual cases edit the source.

The translation is made inside this project from the same source, so this is
a cross-check of the engine's evaluation, not an independent reproduction of
the design by someone else.

Usage (clingo comes from PyPI through uv):
  uv run --with clingo==5.8.2 python tools/second_engine.py run CASE_ID...
      [--out PATH]
  uv run --with clingo==5.8.2 python tools/second_engine.py report
      [--results PATH ...]
"""
import argparse
import json
import re
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "book-1" / "source" / "constitution.nibli"
SUITES = ROOT / "tests" / "pins" / "suites.json"
RESULTS = ROOT / "book-1" / "source" / "measurements" / "second-engine-results.json"
REPORT = ROOT / "book-1" / "source" / "measurements" / "second-engine-report.md"

VAR = re.compile(r"\$([a-z][a-z0-9_]*)")
CONST = re.compile(r'(?<![A-Za-z0-9_$"])([A-Z][A-Za-z0-9_]*)')
EVENT = re.compile(r"event \{ ([a-z_]+)\(\) \}")
TERM = r"(\$[a-z0-9_]+|[A-Z][A-Za-z0-9_]*)"


def term_text(text):
    text = EVENT.sub(lambda m: f'"event:{m.group(1)}"', text)
    text = CONST.sub(lambda m: f'"{m.group(1)}"', text)
    return VAR.sub(lambda m: "V_" + m.group(1), text)


def split_top(text, sep):
    """Split on a separator outside parentheses and braces."""
    parts, depth, start, i = [], 0, 0, 0
    while i < len(text):
        ch = text[i]
        if ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
        elif depth == 0 and text.startswith(sep, i):
            parts.append(text[start:i])
            start = i + len(sep)
            i = start
            continue
        i += 1
    parts.append(text[start:])
    return [p.strip() for p in parts]


def strip_parens(text):
    text = text.strip()
    while text.startswith("(") and text.endswith(")"):
        depth = 0
        for i, ch in enumerate(text):
            depth += ch == "("
            depth -= ch == ")"
            if depth == 0 and i < len(text) - 1:
                return text
        text = text[1:-1].strip()
    return text


def literal(atom):
    atom = atom.strip()
    m = re.fullmatch(r"~\(" + TERM + r" = " + TERM + r"\)", atom)
    if m:
        return f"{term_text(m.group(1))} != {term_text(m.group(2))}"
    m = re.fullmatch(r"\(?" + TERM + r" = " + TERM + r"\)?", atom)
    if m:
        return f"{term_text(m.group(1))} = {term_text(m.group(2))}"
    if atom.startswith("~"):
        return "not " + term_text(atom[1:])
    return term_text(atom)


def alternatives(body):
    """The body as literal lists, one per expansion of its disjunctions."""
    expansions = [[]]
    for conj in split_top(body, " & "):
        inner = strip_parens(conj)
        if conj.startswith("(") and len(split_top(inner, " | ")) > 1:
            options = [[literal(a) for a in split_top(strip_parens(alt), " & ")]
                       for alt in split_top(inner, " | ")]
            expansions = [e + o for e in expansions for o in options]
        else:
            expansions = [e + [literal(conj)] for e in expansions]
    return expansions


def bound(lits):
    out = set()
    for lit in lits:
        if not lit.startswith("not ") and "!=" not in lit:
            out |= set(re.findall(r"V_[a-z0-9_]+", lit))
    return out


def statement(line):
    """One Nibli statement as clingo statements; None if unsupported."""
    line = line.strip()
    if not line or line.startswith("#"):
        return []
    if line.startswith("admits(") or line.startswith("derived_only("):
        return []
    m = re.fullmatch(r"entitled\(every person, event \{ ([a-z_]+)\(\) \}\)\.", line)
    if m:
        return [f'entitled(V_x, "event:{m.group(1)}") :- person(V_x).']
    if " -> " in line:
        body, head = line[:-1].split(" -> ")
        body = re.sub(r"^(all \$[a-z0-9_]+: )+", "", body)
        head = term_text(head.strip())
        out = []
        for lits in alternatives(body):
            used = set(re.findall(r"V_[a-z0-9_]+", head)) | {
                v for lit in lits for v in re.findall(r"V_[a-z0-9_]+", lit)}
            if used - bound(lits):
                return None  # unsafe: nothing binds a variable
            out.append(f"{head} :- {', '.join(lits)}.")
        return out
    if line.endswith("."):
        return [term_text(line)]
    return None


def translate(out_path):
    lines, unsupported = [], []
    for line in SOURCE.read_text(encoding="utf-8").splitlines():
        converted = statement(line)
        if converted is None:
            unsupported.append(line)
        else:
            lines += converted
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return len(lines), unsupported


def run_case(clingo, program, case):
    """Ground the case once, with every fact its fixtures and pins will assert
    declared external, then answer each query by switching on the facts
    asserted before it. An accepted rule carries a guard atom switched on at
    its position."""
    events = []  # ("fact", term) | ("rule", text) | ("query", ...)
    for fixture in case.get("fixtures", []):
        for line in (ROOT / fixture).read_text(encoding="utf-8").splitlines():
            events += [("assert", s) for s in (statement(line) or [])]
    untranslated = []
    for pin in case["pins"]:
        lines = (ROOT / pin).read_text(encoding="utf-8").splitlines()
        directive, i = None, 0
        while i < len(lines):
            line = lines[i].strip()
            i += 1
            if not line or line.startswith("#"):
                continue
            if line.startswith(":"):
                directive = line.split()[0]
                continue
            if line.startswith("? "):
                query = line[2:].strip().rstrip(".")
                expected = None
                if i < len(lines) and lines[i].strip().startswith("# =>"):
                    expected = lines[i].strip()[4:].strip()
                    i += 1
                events.append(("query", (pin, i, query, expected)))
                directive = None
                continue
            if directive in (":refuse", ":accept-scoped", ":require"):
                directive = None
                continue
            directive = None
            converted = statement(line)
            if converted is None:
                untranslated.append({"pin": pin, "line": i, "query": line,
                                     "expected": "translatable", "clingo": "untranslated"})
                continue
            events += [("assert", s) for s in converted]

    program_parts, switches, guard = [], [], 0
    for kind, payload in events:
        if kind != "assert":
            switches.append(None)
            continue
        # Every assertion is switched on through its own guard atom. Declaring
        # the fact itself external would be wrong: clingo ignores an external
        # declaration for an atom some ground rule can also derive, so a
        # person asserted in a case and also derivable from custody would
        # never be switched on.
        guard += 1
        atom = f"asserted({guard})"
        program_parts.append(f"#external {atom}.")
        if ":-" in payload:
            program_parts.append(payload[:-1] + f", {atom}.")
        else:
            program_parts.append(f"{payload[:-1]} :- {atom}.")
        switches.append(atom)

    ctl = clingo.Control(["--warn=none"])
    ctl.load(str(program))
    ctl.add("base", [], "\n".join(program_parts))
    ctl.ground([("base", [])])
    results, solves, on = list(untranslated), 0, set()
    for (kind, payload), atom in zip(events, switches):
        if kind == "assert":
            if atom not in on:
                ctl.assign_external(clingo.parse_term(atom), True)
                on.add(atom)
            continue
        pin, line, query, expected = payload
        models = []
        ctl.solve(on_model=lambda m: models.append({str(s) for s in m.symbols(atoms=True)}))
        solves += 1
        target = str(clingo.parse_term(term_text(query)))
        if len(models) == 1:
            got = "TRUE" if target in models[0] else "FALSE"
        else:
            got = f"{len(models)} models"
        results.append({"pin": pin, "line": line, "query": query,
                        "expected": expected, "clingo": got})
    return results, solves


def command_run(args):
    import clingo

    inventory = json.loads(SUITES.read_text(encoding="utf-8"))
    cases = {c["id"]: c for c in inventory["cases"]}
    with tempfile.TemporaryDirectory() as tmp:
        program = Path(tmp) / "constitution.lp"
        count, unsupported = translate(program)
        if unsupported:
            sys.exit(f"untranslated statements: {unsupported[:3]}")
        out = Path(args.out)
        document = json.loads(out.read_text(encoding="utf-8")) if out.exists() else {
            "clingo": clingo.__version__, "cases": {}}
        document["translated_statements"] = count
        for case_id in args.cases:
            case = cases[case_id]
            if case.get("base") != "live" or case.get("edits"):
                document["cases"][case_id] = {"skipped": "not the live source"}
                continue
            start = time.time()
            results, solves = run_case(clingo, program, case)
            differences = [r for r in results if r["expected"] != r["clingo"]]
            document["cases"][case_id] = {
                "queries": len(results), "solves": solves,
                "seconds": round(time.time() - start, 1),
                "differences": differences,
            }
            print(f"{case_id}: {len(results)} queries, {len(differences)} differences",
                  flush=True)
            out.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n",
                           encoding="utf-8")


def command_report(args):
    cases, clingo_version, statements = {}, None, None
    for path in args.results:
        document = json.loads(Path(path).read_text(encoding="utf-8"))
        cases.update(document["cases"])
        clingo_version = document.get("clingo", clingo_version)
        statements = document.get("translated_statements", statements)
    ran = {k: v for k, v in cases.items() if "queries" in v}
    queries = sum(v["queries"] for v in ran.values())
    differences = [(k, d) for k, v in ran.items() for d in v["differences"]]
    groups = {}
    for case_id in ran:
        group = "chapter pins" if case_id.startswith("book-1/") and "/source/" not in case_id \
            else "source pins" if case_id.startswith("book-1/source/") \
            else case_id.split("/", 1)[0]
        groups[group] = groups.get(group, 0) + 1
    lines = [
        "<!-- Generated by tools/second_engine.py report. -->",
        "# The core replayed in a second engine",
        "",
        f"The constitution was translated statement by statement into clingo's",
        f"input language ({statements} clingo statements) and each case below was",
        f"replayed through clingo {clingo_version}: facts asserted in order, each",
        "query answered against the stable model of everything asserted so far,",
        "and the answer compared with the verdict the pin file expects.",
        "",
        f"{len(ran)} cases and {queries} queries were compared; "
        f"{len(differences)} answers differ. By group: "
        + ", ".join(f"{group} {count}" for group, count in sorted(groups.items()))
        + ".",
        "",
        "This is a cross-check of the engine's evaluation, made inside this",
        "project from the same source. It is not an independent reproduction",
        "of the design, and it does not compare refusals, scoped acceptances,",
        "shell checks or contradiction scans.",
        "",
    ]
    if differences:
        lines += ["## Differences", "", "| Case | Query | Pins expect | clingo |", "|---|---|---|---|"]
        for case_id, d in differences:
            lines.append(f"| `{case_id}` | `{d['query']}` | {d['expected']} | {d['clingo']} |")
        lines.append("")
    lines += ["## Cases", "", "| Case | Queries | Solves | Differences |", "|---|---|---|---|"]
    for case_id in sorted(ran):
        v = ran[case_id]
        lines.append(f"| `{case_id}` | {v['queries']} | {v['solves']} | {len(v['differences'])} |")
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"{len(ran)} cases, {queries} queries, {len(differences)} differences")


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run")
    run.add_argument("cases", nargs="+")
    run.add_argument("--out", default=str(RESULTS))
    report = sub.add_parser("report")
    report.add_argument("--results", nargs="+", default=[str(RESULTS)])
    args = parser.parse_args()
    {"run": command_run, "report": command_report}[args.command](args)


if __name__ == "__main__":
    main()
