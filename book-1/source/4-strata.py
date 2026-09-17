#!/usr/bin/env python3
"""Compute the predicate dependency graph and stratification of a nibli KR file.

RETAINED WRONG, ON PURPOSE — do not fix. This script is the method part's
tooling-blindness exhibit: a hand-rolled parser that looked sound, agreed with
itself, and disagreed with the engine, kept as the standing argument for taking
every figure from `nibli-pin --strata` instead. Nothing consumes it —
`5-spine-gen.py` owns the generated region of `3-spine.md` and reads the
engine — and `3-spine.md` keeps the discrepancy deliberately.

What it is blind to, named so nobody rediscovers it as a bug: the fact branch
takes only the first predicate on a line, so a floor line registers as a fact
of `entitled` and nothing else; and the declaration keywords `derived_only`
and `admits` register as predicates in their own right. Re-measured
2026-08-02 against the same constitution: this script reported 46 predicates,
19 derived and 48 rules where the engine reported 50, 26 and 56.

Standard Ullman fixpoint: for rule H :- ... P ..., stratum(H) >= stratum(P);
for H :- ... ~P ..., stratum(H) > stratum(P). Iterate to fixpoint; if a
stratum exceeds |preds|, the program is unstratifiable.
"""
import re, sys
from collections import defaultdict

path = sys.argv[1]
lines = [l.strip() for l in open(path) if l.strip() and not l.strip().startswith('#')]

LIT = re.compile(r'(~?)\s*([a-z_][A-Za-z0-9_]*)\s*\(')

def strip_quant(s):
    while True:
        m = re.match(r'^all\s+\$[A-Za-z0-9_]+\s*:\s*', s)
        if not m:
            return s
        s = s[m.end():]

rules = []          # (head_pred, [(neg, body_pred)])
head_preds = set()
all_preds = set()
fact_preds = set()

for l in lines:
    l = l.rstrip('.')
    if '->' in l:
        body_s, head_s = l.split('->', 1)
        body_s = strip_quant(body_s.strip())
        hm = LIT.search(head_s)
        if not hm:
            continue
        head = hm.group(2)
        head_preds.add(head)
        all_preds.add(head)
        body = []
        for part in body_s.split('&'):
            part = part.strip()
            if '=' in part and not LIT.search(part):
                continue            # disequality guard, no predicate dep
            m = LIT.search(part)
            if not m:
                continue
            neg = part.lstrip().startswith('~')
            body.append((neg, m.group(2)))
            all_preds.add(m.group(2))
        rules.append((head, body))
    else:
        m = LIT.search(strip_quant(l))
        if m:
            fact_preds.add(m.group(2))
            all_preds.add(m.group(2))

# fixpoint
stratum = {p: 0 for p in all_preds}
n = len(all_preds)
for _ in range(n + 2):
    changed = False
    for head, body in rules:
        for neg, p in body:
            want = stratum[p] + 1 if neg else stratum[p]
            if stratum[head] < want:
                stratum[head] = want
                changed = True
    if not changed:
        break
if changed:
    print("UNSTRATIFIABLE"); sys.exit(1)

# is a predicate's whole dependency cone negation-free?
deps = defaultdict(list)
for head, body in rules:
    for neg, p in body:
        deps[head].append((neg, p))

def cone_monotone(p, seen=None):
    if seen is None:
        seen = set()
    if p in seen:
        return True
    seen.add(p)
    for neg, q in deps.get(p, []):
        if neg:
            return False
        if not cone_monotone(q, seen):
            return False
    return True

base = sorted(p for p in all_preds if p not in head_preds)
print(f"predicates={len(all_preds)}  derived={len(head_preds)}  base-only={len(base)}  rules={len(rules)}")
print(f"max stratum = {max(stratum.values())}\n")

by_s = defaultdict(list)
for p in all_preds:
    by_s[stratum[p]].append(p)

for s in sorted(by_s):
    print(f"--- STRATUM {s} ---")
    for p in sorted(by_s[s]):
        kind = "base " if p in base else ("MONO " if cone_monotone(p) else "nonmono")
        print(f"  {kind:8} {p}")
    print()

print("=== negative edges (what each sanction waits on) ===")
for head, body in rules:
    for neg, p in body:
        if neg:
            print(f"  {head}  <-NOT-  {p}")
