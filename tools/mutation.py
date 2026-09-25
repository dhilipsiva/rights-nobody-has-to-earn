#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Mutation testing over the constitution's rules (development tool, item 49).

A mutant is the live constitution with one rule changed in one way: a body
conjunct dropped, a negation flipped, or two variables swapped inside one atom.
Each mutant runs, as an in-memory edit of the live source, against the cases
whose pins name the mutated rule's head (its distinguishing constants, or its
relation when the head has none). A mutant that every selected pin still passes
survives, and a survivor is a finding: a missing pin, a deliberately dormant
guard, or a defect for its own item. Its disposition is recorded by hand in
book-1/source/measurements/mutation-dispositions.json, and the report lists
every survivor with it. There is no mutation score: the assurance portfolio
refuses aggregate scores and percentages.

Sampling, stated because the result depends on it:
  - rules are sampled per family (a generated block of the constitution, or
    the hand-written articles outside every block), --per-family rules each,
    chosen by a seeded shuffle;
  - each sampled rule gets one mutation, the operator chosen by the same seed
    among those that apply to it;
  - each mutant runs only the cases whose pin queries name its head token and
    whose base, the live source or a temporal stage or counterfactual built
    on it, leaves the mutated rule intact; at most --max-cases of them: half
    from cases expecting that head TRUE somewhere and half from cases
    expecting it FALSE, each half ranked by how many of the mutated rule's own
    constants their pins and fixtures mention, then by how many queries name
    the head, ties broken by the seed.
A survivor therefore means that no selected pin observed the change, not that
no pin anywhere could. Mutants that would leave a variable bound by no
positive atom are not generated; a mutant the engine refuses to load (an
unstratifiable rule) is recorded as refused, and one that exhausts the time or
memory bound as resource, and each is replaced by another sample.

The working tree is never touched. Each mutant runs from a temporary root whose
book-1 and pin directories are symlinks to the real ones and whose
tests/pins/suites.json is written for that mutant alone.

Usage:
  python3 tools/mutation.py run [--per-family N] [--max-cases N] [--seed S]
                                [--family NAME] [--jobs N] [--out PATH]
  python3 tools/mutation.py rerun [--results PATH]   # survivors, after new pins
  python3 tools/mutation.py report [--results PATH]
"""
import argparse
import hashlib
import json
import os
import random
import re
import resource
import signal
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "book-1" / "source" / "constitution.nibli"
SUITES = ROOT / "tests" / "pins" / "suites.json"
RUNNER = ROOT / "target" / "release" / "rights-verify"
RESULTS = ROOT / "book-1" / "source" / "measurements" / "mutation-results.json"
DISPOSITIONS = ROOT / "book-1" / "source" / "measurements" / "mutation-dispositions.json"
REPORT = ROOT / "book-1" / "source" / "measurements" / "mutation-survivors.md"

BEGIN = re.compile(r"^# <([A-Z0-9-]+)-RULES-BEGIN>$")
END = re.compile(r"^# <([A-Z0-9-]+)-RULES-END>$")
HAND = "HAND-WRITTEN"
VARIABLE = re.compile(r"\$[a-z][a-z0-9_]*")
CONSTANT = re.compile(r"\b[A-Z][A-Za-z0-9_]*\b")
SELECTION = (
    "cases resting on the live source, directly or through a derived base whose edits "
    "leave the mutated rule intact, whose pin queries name the mutated rule's head "
    "constants (or its relation when the head has none): half expecting the head TRUE "
    "and half FALSE, each ranked by how many of the rule's own constants their pins "
    "and fixtures mention"
)
DISPOSITION_KINDS = {
    "missing-pin": "no pin observes the change; a pin should",
    "dormant-guard": "a deliberately dormant guard the design keeps",
    "equivalent": "the mutant is logically equivalent to the rule",
    "defect": "a design defect, owed its own item",
    "subsumed": "another rule or guard already produces the same result",
    "checked-by-shape": "a development test on the written rules rejects the mutant",
}


def rules():
    """Every one-line rule statement, with its line number and family."""
    family = HAND
    found = []
    for number, line in enumerate(SOURCE.read_text(encoding="utf-8").splitlines(), 1):
        begin, end = BEGIN.match(line), END.match(line)
        if begin:
            family = begin.group(1)
            continue
        if end:
            family = HAND
            continue
        if line.startswith("all ") and " -> " in line and line.endswith("."):
            found.append({"line": number, "family": family, "text": line})
    return found


def split_rule(text):
    """(quantifier prefix, body atoms, head) of `all ...: body -> head.`"""
    arrow = text.index(" -> ")
    left, head = text[:arrow], text[arrow + 4 : -1]
    prefix_end = 0
    for match in re.finditer(r"all \$[a-z][a-z0-9_]*: ", left):
        if match.start() == prefix_end:
            prefix_end = match.end()
    prefix, body = left[:prefix_end], left[prefix_end:]
    return prefix, body.split(" & "), head


def join_rule(prefix, atoms, head):
    return f"{prefix}{' & '.join(atoms)} -> {head}."


def positive(atom):
    return not atom.startswith("~") and not atom.startswith("(")


def bound_elsewhere(atoms, i):
    """Whether every variable of atom i is bound by another positive atom, so
    removing or negating it leaves the rule safe."""
    others = set()
    for j, atom in enumerate(atoms):
        if j != i and positive(atom):
            others |= set(VARIABLE.findall(atom))
    return set(VARIABLE.findall(atoms[i])) <= others


def mutations(text):
    """Every applicable (operator, index, mutated text) for one rule. A mutant
    that would leave a variable bound by no positive atom is skipped: it is an
    unsafe rule, not a change a pin could be expected to observe."""
    prefix, atoms, head = split_rule(text)
    out = []
    if len(atoms) > 1:
        for i, atom in enumerate(atoms):
            if not positive(atom) or bound_elsewhere(atoms, i):
                out.append(("drop-conjunct", i, join_rule(prefix, atoms[:i] + atoms[i + 1 :], head)))
    for i, atom in enumerate(atoms):
        if atom.startswith("~"):
            flipped = atom[1:]
        elif bound_elsewhere(atoms, i):
            flipped = "~" + atom
        else:
            continue
        out.append(("flip-negation", i, join_rule(prefix, atoms[:i] + [flipped] + atoms[i + 1 :], head)))
    for i, atom in enumerate(atoms):
        names = list(dict.fromkeys(VARIABLE.findall(atom)))
        if len(names) >= 2 and "=" not in atom:
            a, b = names[0], names[1]
            # Swapping two variables that occur nowhere else changes nothing.
            elsewhere = set(VARIABLE.findall(head))
            for j, other in enumerate(atoms):
                if j != i:
                    elsewhere |= set(VARIABLE.findall(other))
            if a not in elsewhere and b not in elsewhere:
                continue
            swapped = VARIABLE.sub(lambda m: b if m.group(0) == a else a if m.group(0) == b else m.group(0), atom)
            if swapped != atom:
                out.append(("swap-role", i, join_rule(prefix, atoms[:i] + [swapped] + atoms[i + 1 :], head)))
    return out


def head_tokens(head):
    tokens = [c for c in CONSTANT.findall(head)]
    if tokens:
        return tokens
    return [head.split("(", 1)[0] + "("]


def chain_edits(inventory, case):
    """The texts every edit on the way from the live source to this case
    replaces, or None when the case does not rest on the live source."""
    befores = [e["before"] for e in case.get("edits", [])]
    name = case.get("base")
    seen = set()
    while name != "live":
        base = inventory["bases"].get(name)
        if base is None or base.get("path") or name in seen:
            return None
        seen.add(name)
        befores += [e["before"] for e in base.get("edits", [])]
        name = base.get("base")
    return befores


def case_index(inventory):
    """Map each case resting on the live source to the queries its pin files
    ask, the constants its pins and fixtures mention, and the texts its edits
    replace. A case on a derived base, such as the temporal stages or a
    counterfactual, can observe a mutant of any rule its edits leave intact."""
    index = []
    cache = {}

    def read(rel):
        if rel not in cache:
            path = ROOT / rel
            cache[rel] = path.read_text(encoding="utf-8") if path.exists() else ""
        return cache[rel]

    for case in inventory["cases"]:
        befores = chain_edits(inventory, case)
        if befores is None:
            continue
        queries, words = [], set()
        for rel in case.get("pins", []):
            text = read(rel)
            lines = text.splitlines()
            for at, line in enumerate(lines):
                if line.startswith("? "):
                    after = lines[at + 1].strip() if at + 1 < len(lines) else ""
                    queries.append((line, after.endswith("TRUE")))
            words |= set(CONSTANT.findall(text))
        for rel in case.get("fixtures", []):
            words |= set(CONSTANT.findall(read(rel)))
        index.append((case, queries, words, befores))
    return index


def select_cases(index, tokens, limit, rng, rule=""):
    """The cases whose queries name the head, ranked first by how many of the
    mutated rule's own constants their pins and fixtures mention, so that a
    head many families share still meets the mutated rule's own family. Half
    the places go to cases expecting the head TRUE somewhere and half to cases
    expecting it FALSE, because a weakened rule is caught by a FALSE that turns
    TRUE and a strengthened one by a TRUE that turns FALSE."""
    own = set(CONSTANT.findall(rule))
    expects_true, expects_false = [], []
    for case, queries, words, befores in index:
        named = [positive for q, positive in queries if any(t in q for t in tokens)]
        if not named:
            continue
        if rule and any(b and (b in rule or rule in b) for b in befores):
            continue
        item = (len(own & words), len(named), rng.random(), case)
        if any(named):
            expects_true.append(item)
        if not all(named):
            expects_false.append(item)
    rank = lambda item: (-item[0], -item[1], item[2])
    chosen, seen = [], set()

    def take(group, count):
        for item in sorted(group, key=rank):
            if count <= 0 or len(chosen) >= limit:
                return
            if item[3]["id"] not in seen:
                chosen.append(item[3])
                seen.add(item[3]["id"])
                count -= 1

    take(expects_true, limit // 2)
    take(expects_false, limit - len(chosen))
    take(expects_true + expects_false, limit - len(chosen))
    return chosen


def temporary_root(inventory, mutant_edit, cases):
    """A root that shares every input except the inventory."""
    root = Path(tempfile.mkdtemp(prefix="mutant-"))
    (root / "verify.sh").symlink_to(ROOT / "verify.sh")
    (root / "book-1").symlink_to(ROOT / "book-1")
    pins = root / "tests" / "pins"
    pins.mkdir(parents=True)
    for entry in (ROOT / "tests" / "pins").iterdir():
        if entry.name != "suites.json":
            (pins / entry.name).symlink_to(entry)
    for entry in (ROOT / "tests").iterdir():
        if entry.name != "pins":
            (root / "tests" / entry.name).symlink_to(entry)
    bases = dict(inventory["bases"])
    selected = []
    for case in cases:
        # The mutant is applied on top of the case's own base, so a case on a
        # temporal stage or a counterfactual sees the same mutated rule.
        name = f"mutant-{case.get('base', 'live')}"
        bases[name] = {"base": case.get("base", "live"), "edits": [mutant_edit]}
        copy = dict(case)
        copy["base"] = name
        selected.append(copy)
    written = dict(inventory)
    written["bases"] = bases
    written["cases"] = selected
    (pins / "suites.json").write_text(json.dumps(written, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return root


def limit_memory(gigabytes):
    def apply():
        limit = gigabytes * 1024**3
        resource.setrlimit(resource.RLIMIT_AS, (limit, limit))
        os.setsid()

    return apply


def run_mutant(inventory, mutant, cases, jobs, timeout, memory):
    """Run one mutant. A mutant that exhausts the time or memory bound is
    recorded as `resource`: the change was observable, but no pin caught it."""
    root = temporary_root(inventory, {"before": mutant["before"], "after": mutant["after"]}, cases)
    process = subprocess.Popen(
        [str(RUNNER)],
        cwd=root,
        env={**os.environ, "RIGHTS_VERIFY_JOBS": str(jobs)},
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        preexec_fn=limit_memory(memory),
    )
    try:
        output, _ = process.communicate(timeout=timeout)
        code = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        code = None
    finally:
        subprocess.run(["rm", "-rf", str(root)], check=False)
    # A case's own refusal pins print stratification errors on every run, so
    # only a mutant line that failed to load counts as a refusal, and running
    # out of memory is checked first.
    if code is None or code < 0 or "memory allocation" in output:
        outcome = "resource"
    elif code == 0:
        outcome = "survived"
    elif code in (1, 3):
        outcome = "killed"
    elif "mutant:" in output and "failed to load" in output:
        outcome = "refused"
    else:
        outcome = "error"
    tail = [line for line in output.splitlines() if line.strip()][-6:]
    return outcome, tail


def mutant_id(rule, operator, index):
    digest = hashlib.sha256(rule["text"].encode()).hexdigest()[:10]
    return f"{rule['family'].lower()}-{digest}-{operator}-{index}"


def command_run(args):
    if not RUNNER.exists():
        sys.exit("build the runner first: cargo build --release --locked --bin rights-verify")
    inventory = json.loads(SUITES.read_text(encoding="utf-8"))
    index = case_index(inventory)
    rng = random.Random(args.seed)
    by_family = {}
    for rule in rules():
        by_family.setdefault(rule["family"], []).append(rule)
    families = sorted(by_family)
    if args.family:
        families = [f for f in families if f == args.family]
    results = []
    for family in families:
        pool = by_family[family][:]
        rng.shuffle(pool)
        taken = 0
        for rule in pool:
            if taken >= args.per_family:
                break
            options = mutations(rule["text"])
            if not options:
                continue
            operator, position, after = rng.choice(options)
            _, _, head = split_rule(rule["text"])
            tokens = head_tokens(head)
            cases = select_cases(index, tokens, args.max_cases, rng, rule["text"])
            if not cases:
                continue
            mutant = {
                "id": mutant_id(rule, operator, position),
                "family": family,
                "line": rule["line"],
                "operator": operator,
                "position": position,
                "before": rule["text"],
                "after": after,
                "head_tokens": tokens,
                "cases": [c["id"] for c in cases],
            }
            outcome, tail = run_mutant(inventory, mutant, cases, args.jobs, args.timeout, args.memory_gb)
            mutant["outcome"] = outcome
            mutant["tail"] = tail
            print(f"{outcome:9} {mutant['id']} ({len(cases)} cases)", flush=True)
            results.append(mutant)
            write_results(args, results)
            if outcome in ("survived", "killed"):
                taken += 1
    write_results(args, results)


def write_results(args, results):
    out = Path(args.out)
    previous = json.loads(out.read_text(encoding="utf-8")) if out.exists() and args.family else {"mutants": []}
    kept = [m for m in previous.get("mutants", []) if not args.family or m["family"] != args.family]
    document = {
        "sampling": {
            "per_family": args.per_family,
            "max_cases": args.max_cases,
            "seed": args.seed,
            "selection": SELECTION,
            "timeout_seconds": args.timeout,
            "memory_gb": args.memory_gb,
        },
        "mutants": kept + results,
    }
    out.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def command_rerun(args):
    """Run the recorded survivors again, with cases re-selected from the
    current pins, and record which are now killed."""
    inventory = json.loads(SUITES.read_text(encoding="utf-8"))
    index = case_index(inventory)
    rng = random.Random(args.seed)
    out = Path(args.results)
    document = json.loads(out.read_text(encoding="utf-8"))
    document["sampling"]["selection"] = SELECTION
    source = SOURCE.read_text(encoding="utf-8")
    for mutant in document["mutants"]:
        if mutant["outcome"] != "survived":
            continue
        if mutant["before"] not in source:
            mutant["outcome"] = "stale"
            print(f"stale     {mutant['id']} (the rule changed)", flush=True)
            continue
        cases = select_cases(index, mutant["head_tokens"], document["sampling"]["max_cases"], rng, mutant["before"])
        outcome, tail = run_mutant(inventory, mutant, cases, args.jobs, args.timeout, args.memory_gb)
        mutant["cases"] = [c["id"] for c in cases]
        mutant["outcome"], mutant["tail"] = outcome, tail
        print(f"{outcome:9} {mutant['id']} ({len(cases)} cases)", flush=True)
        out.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def command_report(args):
    document = json.loads(Path(args.results).read_text(encoding="utf-8"))
    dispositions = json.loads(DISPOSITIONS.read_text(encoding="utf-8")) if DISPOSITIONS.exists() else {}
    survivors = [m for m in document["mutants"] if m["outcome"] == "survived"]
    missing = [m["id"] for m in survivors if m["id"] not in dispositions]
    unknown = [k for k, v in dispositions.items() if v.get("kind") not in DISPOSITION_KINDS]
    stale = [k for k in dispositions if k not in {m["id"] for m in survivors}]
    sampling = document["sampling"]
    lines = [
        "<!-- Generated by python3 tools/mutation.py report; edit mutation-dispositions.json. -->",
        "# Mutation survivors",
        "",
        "A development measurement, not a verification gate and not a score. Each",
        "mutant changes one rule of the live constitution in one way and runs",
        "against the cases whose pins name that rule's head. A survivor is a",
        "mutant every selected pin still passes; its disposition says why.",
        "",
        f"Sampling: {sampling['per_family']} rules per family, at most "
        f"{sampling['max_cases']} cases per mutant, seed {sampling['seed']}; "
        f"cases are {sampling['selection']}. A survivor means no selected pin "
        "observed the change, not that no pin anywhere could.",
        "",
        "| Mutant | Family | Operator | Disposition | Reason |",
        "|---|---|---|---|---|",
    ]
    for m in survivors:
        d = dispositions.get(m["id"], {})
        lines.append(
            f"| `{m['id']}` | {m['family']} | {m['operator']} {m['position']} | "
            f"{d.get('kind', 'UNDISPOSED')} | {d.get('reason', '')} |"
        )
    lines += ["", "## The survivors in full", ""]
    for m in survivors:
        lines += [f"### `{m['id']}`", "", "Before:", "", "```", m["before"], "```", "", "After:", "", "```", m["after"], "```", ""]
    lines += ["## Mutants run", ""]
    for outcome in ("killed", "survived", "resource", "refused", "error", "stale"):
        names = [m["id"] for m in document["mutants"] if m["outcome"] == outcome]
        lines.append(f"- **{outcome}**: {', '.join(f'`{n}`' for n in names) or 'none'}")
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    problems = []
    if missing:
        problems.append(f"undisposed survivors: {', '.join(missing)}")
    if unknown:
        problems.append(f"unknown disposition kinds: {', '.join(unknown)}")
    if stale:
        problems.append(f"dispositions for mutants that are not survivors: {', '.join(stale)}")
    if problems:
        sys.exit("mutation report: " + "; ".join(problems))
    print(f"mutation report: {len(survivors)} survivors, all disposed")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run")
    run.add_argument("--per-family", type=int, default=3)
    run.add_argument("--max-cases", type=int, default=24)
    run.add_argument("--seed", type=int, default=49)
    run.add_argument("--family")
    run.add_argument("--jobs", type=int, default=4)
    run.add_argument("--out", default=str(RESULTS))
    run.add_argument("--timeout", type=int, default=600)
    run.add_argument("--memory-gb", type=int, default=12)
    rerun = sub.add_parser("rerun")
    rerun.add_argument("--results", default=str(RESULTS))
    rerun.add_argument("--seed", type=int, default=49)
    rerun.add_argument("--jobs", type=int, default=4)
    rerun.add_argument("--timeout", type=int, default=300)
    rerun.add_argument("--memory-gb", type=int, default=16)
    report = sub.add_parser("report")
    report.add_argument("--results", default=str(RESULTS))
    args = parser.parse_args()
    {"run": command_run, "rerun": command_rerun, "report": command_report}[args.command](args)


if __name__ == "__main__":
    main()
