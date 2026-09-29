#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Generate the companion's pages on what was checked and what is left open.

`ui/assurance.json` holds what two companion pages show, generated from the
records that carry it so that neither page is a second hand-written account:

- the second engine, from `book-1/source/measurements/second-engine-results.json`
  (each case replayed, its queries and any answer that differs from its pin)
  and its report (what the translation keeps and what the comparison leaves
  out), with the method's sentence saying where that cross-check stops;
- the current limits: the adversarial audit's open findings with the claim
  each withholds, every active declared defect (found by scanning each pin
  file for a `:defect` line) and each derived chapter's "What this cannot
  settle", quoted. Its pointer to the chapter's runnable cases and any
  sentence handing on to the next chapter or part are navigation rather
  than limits, and are left out.

Repaired defects are not listed: the pages describe the current design, and
the repository's records keep the rest.

    python3 tools/assurance_pages.py           write ui/assurance.json
    python3 tools/assurance_pages.py --check   fail unless it is current

No dependencies, so the companion's tests can import it.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "ui" / "assurance.json"
RESULTS = ROOT / "book-1" / "source" / "measurements" / "second-engine-results.json"
REPORT = ROOT / "book-1" / "source" / "measurements" / "second-engine-report.md"
METHOD = ROOT / "book-1" / "method.md"
AUDIT = ROOT / "book-1" / "source" / "adversarial-audit-source.json"
MANIFEST = ROOT / "book-1" / "contents.json"
LIMITS_HEADING = "What this cannot settle"
# Navigation inside a limits section: the pointer to the chapter's runnable
# cases, and sentences handing on to the next chapter or part.
POINTER = re.compile(r"^Run it:")
HANDOFF = re.compile(r"^(The next (chapter|chapters|part)\b|Part V\b)")
METHOD_SECTION = "What has been checked, and where each check stops"


def unique_keys(pairs):
    """Refuse a JSON object whose keys repeat; a plain load keeps the last."""
    keys = [key for key, _ in pairs]
    repeated = sorted({key for key in keys if keys.count(key) > 1})
    if repeated:
        raise ValueError(f"repeated keys: {repeated}")
    return dict(pairs)


def report_list(report: str, heading: str) -> list[str]:
    """The bullet items of one `## heading` section of the report."""
    marker = f"## {heading}\n"
    if marker not in report:
        raise ValueError(f"the report has no section {heading!r}")
    section = report.split(marker, 1)[1].split("\n## ", 1)[0]
    items = [line[2:].strip() for line in section.splitlines() if line.startswith("- ")]
    if not items:
        raise ValueError(f"the report's section {heading!r} lists nothing")
    return items


def method_limit(method: str) -> str:
    """The method's sentence on where the second-engine cross-check stops."""
    start = method.index("**The core replayed in a second engine.**")
    paragraph = " ".join(method[start:].split("\n\n", 1)[0].split())
    sentences = re.split(r"(?<=[.:])\s+(?=[A-Z])", paragraph)
    found = [s for s in sentences if "misreading shared by both engines" in s]
    if len(found) != 1:
        raise ValueError("the method no longer says where the cross-check stops")
    return found[0]


def group(case_id: str) -> str:
    if case_id.startswith("book-1/source/"):
        return "source pins"
    if case_id.startswith("book-1/"):
        return "chapter pins"
    return case_id.split("/", 1)[0]


def engine(results: dict, report: str, method: str) -> dict:
    ran = {key: value for key, value in results["cases"].items() if "queries" in value}
    cases = [
        {"id": key, "queries": value["queries"], "differences": len(value["differences"])}
        for key, value in sorted(ran.items())
    ]
    groups: dict[str, int] = {}
    for case in cases:
        groups[group(case["id"])] = groups.get(group(case["id"]), 0) + 1
    return {
        "clingo": results["clingo"],
        "statements": results["translated_statements"],
        "queries": sum(case["queries"] for case in cases),
        "differences": sum(case["differences"] for case in cases),
        "groups": [{"name": name, "cases": count} for name, count in sorted(groups.items())],
        "keeps": report_list(report, "What the translation keeps"),
        "leaves_out": report_list(report, "What the comparison leaves out"),
        "limit": method_limit(method),
        "method_section": METHOD_SECTION,
        "cases": cases,
    }


def open_findings(audit: dict) -> list[dict]:
    findings = []
    for lens in audit["lenses"]:
        for finding in lens["findings"]:
            if finding.get("open"):
                findings.append({
                    "lens": lens["lens"],
                    "finding": finding["finding"],
                    "withholds": finding["consequence"],
                    "disposition": finding["disposition"],
                })
    return findings


def declared_defects(root: Path = ROOT) -> list[dict]:
    """Every `:defect` line in a pin or fixture file: an active expectation
    that a known defect still reproduces."""
    defects = []
    for base in (root / "book-1", root / "tests"):
        for path in sorted(base.rglob("*.nibli")):
            if path.name == "constitution.nibli":
                continue
            for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                stripped = line.strip()
                if stripped.startswith(":defect"):
                    defects.append({
                        "file": path.relative_to(root).as_posix(),
                        "line": number,
                        "reason": stripped[len(":defect"):].strip().strip('"'),
                    })
    return defects


def plain(markdown: str) -> str:
    """Markdown to plain text for a quotation: links keep their words, and
    emphasis, inline code, note references and comments are removed."""
    text = re.sub(r"<!--.*?-->", "", markdown, flags=re.S)
    text = re.sub(r"\[\^[^\]]+\]", "", text)
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"`([^`]*)`", r"\1", text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"\1", text)
    text = re.sub(r"\*([^*]+)\*", r"\1", text)
    return " ".join(text.split())


def chapter_limits(manifest: dict, root: Path = ROOT) -> list[dict]:
    chapters = []
    for part in manifest["parts"]:
        for chapter in part["chapters"]:
            if chapter.get("role") != "derived" or chapter.get("status") != "landed":
                continue
            path = root / "book-1" / chapter["file"]
            text = path.read_text(encoding="utf-8")
            marker = f"## {LIMITS_HEADING}\n"
            if marker not in text:
                continue
            section = text.split(marker, 1)[1].split("\n## ", 1)[0]
            paragraphs = []
            for block in re.split(r"\n\s*\n", section.strip()):
                items = re.split(r"\n(?=[-*] )", block.strip())
                for item in items:
                    item = plain(re.sub(r"^[-*] ", "", item.strip()))
                    if not item or POINTER.match(item):
                        continue
                    sentences = re.split(r"(?<=[.?!])\s+(?=[A-Z])", item)
                    kept = [s for s in sentences if not HANDOFF.match(s)]
                    if kept:
                        paragraphs.append(" ".join(kept))
            chapters.append({
                "number": chapter["number"],
                "title": chapter["title"],
                "file": f"book-1/{chapter['file']}",
                "paragraphs": paragraphs,
            })
    return chapters


def sources() -> dict:
    """Every record the pages are generated from, as loaded."""
    return {
        "results": json.loads(RESULTS.read_text(encoding="utf-8"), object_pairs_hook=unique_keys),
        "report": REPORT.read_text(encoding="utf-8"),
        "method": METHOD.read_text(encoding="utf-8"),
        "audit": json.loads(AUDIT.read_text(encoding="utf-8")),
        "manifest": json.loads(MANIFEST.read_text(encoding="utf-8")),
    }


def build(loaded: dict | None = None, root: Path = ROOT) -> dict:
    loaded = loaded if loaded is not None else sources()
    return {
        "license": "CC-BY-4.0",
        "generated_by": "tools/assurance_pages.py",
        "engine": engine(loaded["results"], loaded["report"], loaded["method"]),
        "limits": {
            "findings": open_findings(loaded["audit"]),
            "defects": declared_defects(root),
            "chapters": chapter_limits(loaded["manifest"], root),
        },
    }


def render(document: dict) -> str:
    return json.dumps(document, indent=2, ensure_ascii=False) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true",
                        help="fail unless ui/assurance.json matches its sources")
    args = parser.parse_args()
    fresh = render(build())
    if args.check:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.exists() else ""
        if current != fresh:
            print("ui/assurance.json is stale; run python3 tools/assurance_pages.py",
                  file=sys.stderr)
            return 1
        print("ui/assurance.json is current")
        return 0
    OUTPUT.write_text(fresh, encoding="utf-8")
    document = json.loads(fresh)
    print(f"wrote ui/assurance.json: {len(document['engine']['cases'])} engine cases, "
          f"{len(document['limits']['findings'])} open findings, "
          f"{len(document['limits']['defects'])} declared defects, "
          f"{len(document['limits']['chapters'])} chapter limits")
    return 0


if __name__ == "__main__":
    sys.exit(main())
