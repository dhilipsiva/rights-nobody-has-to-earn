#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Print, or check, the rulings index at the top of CLAUDE.md.

One line per ruling: its heading, which carries its date, and the controlling record under
book-1/appendix/ that its text names, where it names one. A ruling is a dated heading inside
"Author-Ratified Rulings", or a dated top-level section that calls itself an author decision or
a ruling. The undated group headings share the last line. `--check` exits 1 when the index in
CLAUDE.md (the lines after "## Rulings index" up to the first blank line) differs.
"""
import re
import sys
from pathlib import Path

CLAUDE = Path(__file__).resolve().parent.parent / "CLAUDE.md"
DATE = re.compile(r"\d{4}-\d\d-\d\d")
RECORD = re.compile(r"book-1/appendix/[\w./-]+\.md")
HEADING = re.compile(r"^(#{2,4}) (.+)$")


def generate(text):
    lines = text.split("\n")
    heads = [(i, m.group(1), m.group(2)) for i, line in enumerate(lines)
             if (m := HEADING.match(line))]
    start = next(i for i, _, title in heads if title == "Author-Ratified Rulings")
    end = next(i for i, level, _ in heads if level == "##" and i > start)
    entries, undated = [], []
    for n, (i, level, title) in enumerate(heads):
        inside = start < i < end
        ruling = DATE.search(title) and (
            inside or (level == "##" and re.search(r"author decision|ruling", title)))
        if not ruling:
            if inside:
                undated.append(title)
            continue
        following = heads[n + 1][0] if n + 1 < len(heads) else len(lines)
        record = RECORD.search("\n".join(lines[i + 1:following]))
        entries.append(f"- {title}" + (f" → `{record.group(0)}`" if record else ""))
    entries.append("- Undated, under Author-Ratified Rulings: " + "; ".join(undated))
    return entries


def current(text):
    lines = text.split("\n")
    at = lines.index("## Rulings index") + 1
    block = []
    while at < len(lines) and lines[at]:
        block.append(lines[at])
        at += 1
    return block


def main():
    text = CLAUDE.read_text(encoding="utf-8")
    entries = generate(text)
    if sys.argv[1:] == ["--check"]:
        if current(text) != entries:
            print("CLAUDE.md's rulings index is stale; regenerate it with tools/rulings_index.py",
                  file=sys.stderr)
            return 1
        print(f"rulings index current: {len(entries) - 1} rulings")
        return 0
    print("\n".join(entries))
    return 0


if __name__ == "__main__":
    sys.exit(main())
