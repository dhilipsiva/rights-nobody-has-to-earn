# SPDX-License-Identifier: MIT OR Apache-2.0
"""Read each reading input's one-line summary from the annotated contents.

The annotated contents in `book-1/reference.md` describe every chapter, Part
opening case and piece of back matter in one list item each:

    - [Chapter 6: Who Owes, and What Follows](06-who-owes-and-what-follows.md) — public
      responsibility, continuity and remedy ...

The companion's book map shows those summaries beside its contents, read from
this file so that they are written once. No dependencies, so the companion's
tests can import it without the book builder's environment.
"""

from __future__ import annotations

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
REFERENCE = ROOT / "book-1" / "reference.md"
ITEM = re.compile(r"- \[[^\]]+\]\(([^)#\s]+)\)\s*—\s*(.+)")


def summaries(text: str | None = None) -> dict[str, str]:
    """File name to summary, for every item of the annotated contents."""
    if text is None:
        text = REFERENCE.read_text(encoding="utf-8")
    heading = "## Annotated contents"
    if heading not in text:
        raise ValueError("the reference material has no annotated contents")
    section = text.split(heading, 1)[1].split("\n## ", 1)[0]
    found: dict[str, str] = {}
    for item in re.split(r"\n(?=- )", section):
        flat = " ".join(item.split())
        match = ITEM.match(flat)
        if match:
            found[match[1]] = match[2].strip()
    return found


if __name__ == "__main__":
    for name, summary in summaries().items():
        print(f"{name}: {summary}")
