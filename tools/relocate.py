#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Relocate Book 1 files and rewrite every reference to them.

    tools/relocate.py plan  [--out MAP]        derive a rename map from book-1/contents.json vs disk
    tools/relocate.py apply MAP [--dry-run]    git-mv the files and rewrite the references
    tools/relocate.py check MAP                prove nothing dangles and nothing else moved

A map is JSON:

    {"name": "...", "date": "YYYY-MM-DD", "notes": "...",
     "moves":    [{"from": "book-1/04-what-you-are-owed.md", "to": "book-1/01-what-you-are-owed.md"}, ...],
     "rewrites": [{"from": "book-1/source/", "to": "book-1/source/"}, ...],
     "labels":   {"8": "1", ...}}

`moves` are files or directories, applied with `git mv`. Text rewrites are derived
from the moves — every from-path stem to its to-path stem, and every changed
basename stem guarded by a boundary character — plus the explicit `rewrites`,
applied longest-from-string first over the text scope. `labels` rewrites
"Chapter N"/"chapter N"/"chapter-N" in one simultaneous pass over the numbered
chapters, the method part and pins-file comments only, never inside a footnote
definition, where "chapter N" cites another book. Afterwards the
reader-coverage source's record ids are renumbered and its records re-sorted
into manifest order.

Two further keys restructure rather than rename. Their content is editorial, so
it is prepared by hand at each destination before `apply` runs:

     "merges": [{"into": "book-1/09-work-pay-and-contribution.md",
                 "from": ["book-1/09-earning-above-the-floor.md", "book-1/10-contribution.md"]}, ...],
     "splits": [{"from": "book-1/13-a-place-in-which-life-remains-possible.md",
                 "into": ["book-1/12-a-place-in-which-life-remains-possible.md",
                          "book-1/13-creatures-without-a-ballot.md"]}, ...]

`apply` removes every merged or split source that is not itself a destination
and rewrites references to a merged source to its destination. A reference to a
split source goes to the destination whose text holds its `::needle`, or to the
first destination when it names none; every such default is printed for review.
`check` holds each group to account: the pins statements of the sources at
HEAD, blank lines and `:expect-pins` aside, are exactly the destinations' (a
split may repeat a statement in several destinations but may add none), the
`:expect-pins` totals agree, and every prose paragraph of a source that no
destination carries verbatim is listed, because a seam is edited by hand.

Content this tool never changes, and `check` asserts byte-identical against HEAD
(through the rename): the constitution, the eight rule-family sources, every
pin statement of every pins file (comment and `:require` lines carry paths and
are rewritten), 4-strata.py, registry/, the legacy manuscripts, tmp.txt,
reviews/ and new-reviwes/.
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = "book-1/contents.json"

FAMILY_SOURCES = {
    "equality-source.json", "family-life-source.json", "integrity-source.json", "knowledge-source.json",
    "obligations-source.json", "record-power-source.json", "scarcity-source.json",
    "state-form-source.json", "statistics-source.json",
}

# Text scope: files whose path references are rewritten. Pins files are in scope
# for their comment lines only (handled specially).
TEXT_GLOBS = [
    "CLAUDE.md", "README.md", "AGENTS.md", "LICENSING.md", "TODO.md",
    "verify.sh", "generate.sh", "combine.sh",
    "book-1/*.md", "book-1/appendix/*.md", "book-1/appendix/*/*.md", "book-1/contents.json",
    "book-1/source/*.md", "book-1/source/*.json", "book-1/source/*.py",
    "book-1/source/counterfactual/README.md", "book-1/source/reader-evidence-pilot/*.md",
    "book-2/*.md",
    "tests/pins/suites.json",
    "book-1/source/*.md", "book-1/source/*.json", "book-1/source/*.py",
    "book-1/source/counterfactual/README.md", "book-1/source/reader-evidence-pilot/*.md",
    "src/*.rs", "src/*/*.rs", "src/bin/*.rs",
    "tools/*.py", "tests/*.py",
    "ui/*.json", "ui/*.md", "ui/tests/*.py", "ui/tests/*.js", "ui/src/*.rs", "ui/src/bin/*.rs",
]
# The tool's own examples name paths, and must not rewrite themselves.
TEXT_EXCLUDE_BASENAMES = FAMILY_SOURCES | {"4-strata.py", "relocate.py"}

NEVER_TOUCHED = [
    "**/constitution.nibli", "**/4-strata.py",
    "registry/**", "book.md", "manifesto.md", "1.md", "2.md", "tmp.txt",
    "reviews/**", "new-reviwes/**",
] + [f"**/{name}" for name in sorted(FAMILY_SOURCES)]

LABEL_GLOBS = ["book-1/[0-9][0-9]-*.md", "book-1/method.md", "book-1/reference.md"]
BOUNDARY = "([`\"' \t\\[/"


def git(*args: str, check: bool = True) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if check and result.returncode != 0:
        raise SystemExit(f"git {' '.join(args)} failed:\n{result.stderr}")
    return result.stdout


def tracked() -> list[str]:
    return git("ls-files", "-z").split("\0")[:-1]


def matches(path: str, globs: list[str]) -> bool:
    for pattern in globs:
        if "**" in pattern:
            prefix = pattern.split("**")[0]
            suffix = pattern.split("**")[-1].lstrip("/")
            if path.startswith(prefix) and (not suffix or fnmatch.fnmatch(os.path.basename(path), suffix) or path.endswith(suffix)):
                return True
        elif fnmatch.fnmatch(path, pattern):
            return True
    return False


def text_scope() -> list[str]:
    files = [p for p in tracked() if matches(p, TEXT_GLOBS) and os.path.basename(p) not in TEXT_EXCLUDE_BASENAMES]
    return sorted(set(files))


def pins_files() -> list[str]:
    return sorted(p for p in tracked() if p.endswith(".pins.nibli"))


def load_map(path: str) -> dict:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    data.setdefault("moves", []); data.setdefault("rewrites", []); data.setdefault("labels", {})
    data.setdefault("merges", []); data.setdefault("splits", [])
    return data


def groups(mapping: dict) -> list[tuple[list[str], list[str]]]:
    """Every merge and split as (sources, destinations)."""
    out = [(m["from"], [m["into"]]) for m in mapping["merges"]]
    out += [([s["from"]], s["into"]) for s in mapping["splits"]]
    return out


def retired(mapping: dict) -> set[str]:
    """Merged or split sources that are not themselves a destination."""
    sources = {p for src, _ in groups(mapping) for p in src}
    destinations = {p for _, dst in groups(mapping) for p in dst}
    return sources - destinations


def created(mapping: dict) -> set[str]:
    """Destinations of a merge or split that are not themselves a source."""
    sources = {p for src, _ in groups(mapping) for p in src}
    destinations = {p for _, dst in groups(mapping) for p in dst}
    return destinations - sources


def stem(path: str) -> str:
    for suffix in (".pins.nibli", ".md", ".json", ".py", ".nibli"):
        if path.endswith(suffix):
            return path[: -len(suffix)]
    return path


def derived_rewrites(mapping: dict) -> list[tuple[str, str, bool]]:
    """(from, to, guarded) pairs, longest from-string first."""
    pairs: dict[str, tuple[str, bool]] = {}
    for move in mapping["moves"]:
        src, dst = move["from"], move["to"]
        if src.endswith("/") or (ROOT / src).is_dir() or (ROOT / dst).is_dir() and not (ROOT / src).exists():
            pairs[src.rstrip("/") + "/"] = (dst.rstrip("/") + "/", False)
            continue
        pairs[stem(src)] = (stem(dst), False)
        base_src, base_dst = os.path.basename(stem(src)), os.path.basename(stem(dst))
        if base_src != base_dst:
            pairs[base_src] = (base_dst, True)
    for merge in mapping["merges"]:
        for src in merge["from"]:
            if src == merge["into"]:
                continue
            pairs[stem(src)] = (stem(merge["into"]), False)
            base_src, base_dst = os.path.basename(stem(src)), os.path.basename(stem(merge["into"]))
            if base_src != base_dst:
                pairs[base_src] = (base_dst, True)
    for rewrite in mapping["rewrites"]:
        pairs[rewrite["from"]] = (rewrite["to"], False)
    return sorted(((k, v[0], v[1]) for k, v in pairs.items()), key=lambda t: -len(t[0]))


SPLIT_TAIL = r"((?:\.pins\.nibli|\.md)?)((?:::(?:[^\"\\\n]|\\.)*)?)"


def split_rewrite(text: str, mapping: dict, report: list[str], where: str) -> tuple[str, int]:
    """Send each reference to a split source to the destination holding its needle."""
    count = 0
    for split in mapping["splits"]:
        src = stem(split["from"])
        bodies = {}
        for dst in split["into"]:
            for ext in (".md", ".pins.nibli"):
                path = ROOT / (stem(dst) + ext)
                bodies[(stem(dst), ext)] = path.read_text(encoding="utf-8") if path.is_file() else ""
        for name in {src, os.path.basename(src)}:
            prefix = r"(?<![A-Za-z0-9_.-])" if name == src else r"(?<![A-Za-z0-9_.\-/])"
            pattern = re.compile(prefix + re.escape(name) + r"(?![A-Za-z0-9_-])" + SPLIT_TAIL)
            default = stem(split["into"][0]) if name == src else os.path.basename(stem(split["into"][0]))

            def one(m: re.Match) -> str:
                nonlocal count
                ext, needle = m.group(1), m.group(2)
                choice = None
                if needle:
                    raw = needle[2:]
                    try:
                        wanted = json.loads('"' + raw + '"')
                    except json.JSONDecodeError:
                        wanted = raw
                    holders = [dst for (dst, e), body in bodies.items() if e == (ext or ".md") and wanted in body]
                    if len(holders) == 1:
                        choice = holders[0] if name == src else os.path.basename(holders[0])
                if choice is None:
                    choice = default
                    if choice != name:
                        report.append(f"{where}: {m.group(0)[:100]!r} -> {choice}{ext} (default)")
                count += 1
                return choice + ext + needle

            text = pattern.sub(one, text)
    return text, count


def rewrite_text(text: str, pairs: list[tuple[str, str, bool]]) -> tuple[str, int]:
    count = 0
    for src, dst, guarded in pairs:
        if guarded:
            pattern = re.compile(r"(?<![A-Za-z0-9_.-])" + re.escape(src) + r"(?![A-Za-z0-9_-])")
            text, n = pattern.subn(dst, text)
        else:
            n = text.count(src)
            text = text.replace(src, dst)
        count += n
    return text, count


LABEL = re.compile(r"\b([Cc]hapters?)((?:[  ]+\d{1,2})(?:(?:,[  ]*|[  ]+and[  ]+|[  ]+or[  ]+|[  ]*[–-][  ]*)\d{1,2})*)\b")
LABEL_HYPHEN = re.compile(r"\bchapter-(\d{1,2})\b")


FOOTNOTE = re.compile(r"^\[\^[^\]]+\]:")


def rewrite_labels(text: str, labels: dict[str, str]) -> tuple[str, int]:
    """Relabel chapter numbers outside footnote definitions. A footnote cites
    another book, and its "chapter 6" is that book's chapter, not this one's."""
    count = 0
    out: list[str] = []
    block: list[str] = []
    in_note = False
    for line in text.split("\n"):
        if FOOTNOTE.match(line):
            in_note = True
        elif line.strip() and not line.startswith((" ", "\t")):
            in_note = False
        if in_note:
            if block:
                chunk, n = _rewrite_labels("\n".join(block), labels)
                out.append(chunk)
                count += n
                block = []
            out.append(line)
        else:
            block.append(line)
    if block:
        chunk, n = _rewrite_labels("\n".join(block), labels)
        out.append(chunk)
        count += n
    return "\n".join(out), count


def _rewrite_labels(text: str, labels: dict[str, str]) -> tuple[str, int]:
    count = 0

    def numbers(match: re.Match) -> str:
        nonlocal count
        def one(m: re.Match) -> str:
            nonlocal count
            new = labels.get(m.group(0))
            if new is None:
                return m.group(0)
            count += 1
            return new
        return match.group(1) + re.sub(r"\d{1,2}", one, match.group(2))

    text = LABEL.sub(numbers, text)

    def hyphen(m: re.Match) -> str:
        nonlocal count
        new = labels.get(m.group(1))
        if new is None:
            return m.group(0)
        count += 1
        return f"chapter-{new}"

    text = LABEL_HYPHEN.sub(hyphen, text)
    return text, count


NUMBERED_LINK = re.compile(r"\[(\d{1,2})\]\((?:[^)]*/)?(\d{2})-[^)#]*\.md(?:#[^)]*)?\)")


def fix_numbered_links(text: str) -> tuple[str, int]:
    """A link whose text is a bare chapter number must carry its target's prefix:
    `[28](28-where-people-are-put.md)`. The path pass retargets the file; this
    pass makes the visible number follow it."""
    count = 0

    def one(m: re.Match) -> str:
        nonlocal count
        want = str(int(m.group(2)))
        if m.group(1) == want:
            return m.group(0)
        count += 1
        return "[" + want + m.group(0)[len(m.group(1)) + 1:]

    return NUMBERED_LINK.sub(one, text), count


def manifest_order() -> list[str]:
    data = json.loads((ROOT / MANIFEST).read_text(encoding="utf-8"))
    return [f"book-1/{c['file']}" for part in data["parts"] for c in part["chapters"] if c.get("file")]


COVERAGE = "book-1/source/reader-coverage-source.json"


def heading_position(chapter: str, section: str) -> int:
    """Where a coverage record's passage sits in its chapter: the preamble first,
    then each `## ` heading in order, and an unknown section last."""
    path = ROOT / chapter
    if section == "(preamble)" or not path.is_file():
        return -1 if section == "(preamble)" else 10**6
    for index, line in enumerate(path.read_text(encoding="utf-8").split("\n")):
        if line.startswith("## ") and line[3:].strip() == section:
            return index
    return 10**6


def resort_reader_coverage(labels: dict[str, str], dry_run: bool, mapping: dict | None = None) -> int:
    """Renumber record ids through `labels`, key the records of every merge and
    split destination afresh in reading order, and sort every record into the
    manifest's chapter order and each chapter's heading order."""
    path = ROOT / COVERAGE
    if not path.is_file() or not (labels or (mapping and groups(mapping))):
        return 0
    raw = path.read_text(encoding="utf-8")
    data = json.loads(raw)
    assert json.dumps(data, indent=2, ensure_ascii=False) + "\n" == raw, f"{path} does not round-trip; refusing to re-serialise it"
    order = {chapter: i for i, chapter in enumerate(manifest_order())}
    rekeyed = {dst for _, dsts in groups(mapping or {"merges": [], "splits": []}) for dst in dsts
               if dst.endswith(".md") and re.match(r"\d\d-", os.path.basename(dst)) and not os.path.basename(dst).startswith("00-")}
    changed = 0
    for record in data["records"]:
        if record["chapter"] in rekeyed:
            continue
        old_prefix, rest = record["id"].split("-", 1)
        new_prefix = labels.get(str(int(old_prefix)))
        if new_prefix is not None and new_prefix != str(int(old_prefix)):
            record["id"] = f"{int(new_prefix):02d}-{rest}"
            changed += 1
    indexed = list(enumerate(data["records"]))
    indexed.sort(key=lambda t: (order.get(t[1]["chapter"], 10**6),
                                heading_position(t[1]["chapter"], t[1]["section"]), t[0]))
    data["records"] = [r for _, r in indexed]
    for chapter in rekeyed:
        number = int(os.path.basename(chapter)[:2])
        rows = [r for r in data["records"] if r["chapter"] == chapter]
        for index, record in enumerate(rows):
            new_id = f"{number:02d}-{index:02d}"
            if record["id"] != new_id:
                record["id"] = new_id
                changed += 1
    ids = [r["id"] for r in data["records"]]
    duplicates = sorted({i for i in ids if ids.count(i) > 1})
    if duplicates and not dry_run:
        raise SystemExit(f"{COVERAGE}: record ids collide after renumbering: {duplicates[:10]}")
    if not dry_run:
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return changed


def cmd_plan(args: argparse.Namespace) -> int:
    data = json.loads((ROOT / MANIFEST).read_text(encoding="utf-8"))
    by_slug: dict[str, tuple[int, str]] = {}
    for part in data["parts"]:
        for chapter in part["chapters"]:
            if chapter.get("file"):
                slug = chapter["file"][3:-3]
                by_slug[slug] = (chapter["number"], chapter["file"])
    moves, labels = [], {}
    for name in sorted(os.listdir(ROOT / "book-1")):
        if not (name.endswith(".md") and name[:2].isdigit() and name[2] == "-"):
            continue
        if name == "00-opening-note.md":
            continue  # front matter: never renumbered
        slug = name[3:-3]
        if slug not in by_slug:
            print(f"warning: book-1/{name} is not in the manifest", file=sys.stderr)
            continue
        number, target = by_slug[slug]
        if target != name:
            moves.append({"from": f"book-1/{name}", "to": f"book-1/{target}"})
            pins_from, pins_to = f"book-1/{name[:-3]}.pins.nibli", f"book-1/{target[:-3]}.pins.nibli"
            if (ROOT / pins_from).is_file():
                moves.append({"from": pins_from, "to": pins_to})
            labels[str(int(name[:2]))] = str(number)
    plan = {"name": args.name, "date": args.date, "notes": "", "moves": moves, "rewrites": [], "labels": labels}
    text = json.dumps(plan, indent=2) + "\n"
    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(text, encoding="utf-8")
        print(f"wrote {args.out}: {len(moves)} moves, {len(labels)} labels")
    else:
        print(text, end="")
    return 0


def cmd_apply(args: argparse.Namespace) -> int:
    mapping = load_map(args.map)
    pairs = derived_rewrites(mapping)
    labels = mapping["labels"]
    if git("status", "--porcelain").strip() and not args.allow_dirty:
        raise SystemExit("working tree is not clean; commit or stash first (or pass --allow-dirty)")
    for path in sorted(retired(mapping) | created(mapping)):
        if not (ROOT / path).is_file():
            raise SystemExit(f"{path} does not exist; a merge or split destination is prepared by hand before apply")
    retiring = retired(mapping)
    # Prepared destinations are new files, so they join the scope explicitly.
    fresh = sorted(created(mapping))
    scope_before = sorted((set(text_scope()) | {p for p in fresh if matches(p, TEXT_GLOBS)}) - retiring)
    pins_before = sorted((set(pins_files()) | {p for p in fresh if p.endswith(".pins.nibli")}) - retiring)
    sources = {m["from"].rstrip("/") for m in mapping["moves"]}
    for move in mapping["moves"]:
        src, dst = ROOT / move["from"], ROOT / move["to"]
        if not src.exists():
            raise SystemExit(f"cannot move {move['from']}: it does not exist")
        # A destination that is itself a source is a chain (every insertion
        # shifts its neighbours by one); anything else in the way is a clash.
        if dst.exists() and move["to"].rstrip("/") not in sources:
            raise SystemExit(f"cannot move {move['from']} to {move['to']}: the destination exists")
    if args.dry_run:
        print(f"would apply {len(mapping['moves'])} moves and {len(pairs)} rewrite pairs")
    else:
        # Chained renames (a destination that is another move's source) go
        # through a temporary name so they never collide; everything else moves
        # directly, in the listed order, so files can leave a directory before
        # the directory itself moves.
        chained = [m for m in mapping["moves"] if m["to"].rstrip("/") in sources]
        for move in chained:
            git("mv", move["from"], move["from"] + ".relocating")
        for move in mapping["moves"]:
            (ROOT / move["to"]).parent.mkdir(parents=True, exist_ok=True)
            source = move["from"] + ".relocating" if move in chained else move["from"]
            git("mv", source, move["to"])
        for path in sorted(retiring):
            git("rm", "-q", path)
        for path in fresh:
            git("add", path)

    def after(path: str) -> str:
        for move in mapping["moves"]:
            src, dst = move["from"].rstrip("/"), move["to"].rstrip("/")
            if path == src:
                return dst
            if path.startswith(src + "/"):
                return dst + path[len(src):]
        return path

    # In a dry run nothing has moved yet, so files are read where they are.
    locate = (lambda p: ROOT / p) if args.dry_run else (lambda p: ROOT / after(p))
    total = 0
    defaults: list[str] = []
    for old_path in scope_before:
        path = locate(old_path)
        text = path.read_text(encoding="utf-8")
        new, n = rewrite_text(text, pairs)
        new, s = split_rewrite(new, mapping, defaults, after(old_path))
        n += s
        m = 0
        if labels and (matches(after(old_path), LABEL_GLOBS)):
            new, m = rewrite_labels(new, labels)
        new, k = fix_numbered_links(new)
        m += k
        if new != text:
            total += n + m
            if args.dry_run:
                print(f"  {after(old_path)}: {n} path rewrites, {m} label rewrites")
            else:
                path.write_text(new, encoding="utf-8")
    for old_path in pins_before:
        path = locate(old_path)
        lines = path.read_text(encoding="utf-8").split("\n")
        n = m = 0
        out = []
        for line in lines:
            # Comments and `:require` shell lines carry paths; pin statements do not.
            if line.lstrip().startswith(("#", ":require")):
                line, a = rewrite_text(line, pairs)
                line, c = split_rewrite(line, mapping, defaults, after(old_path))
                a += c
                b = 0
                if labels:
                    line, b = rewrite_labels(line, labels)
                n += a; m += b
            out.append(line)
        new = "\n".join(out)
        if n + m:
            total += n + m
            if args.dry_run:
                print(f"  {after(old_path)}: {n} path rewrites, {m} label rewrites (comments)")
            else:
                path.write_text(new, encoding="utf-8")
    renumbered = resort_reader_coverage(labels, args.dry_run, mapping)
    print(f"{'would rewrite' if args.dry_run else 'rewrote'} {total} references; {renumbered} reader-coverage ids renumbered")
    if defaults:
        print(f"{len(defaults)} reference(s) to a split source went to its first destination; review each:")
        print("\n".join("  " + line for line in defaults))
    if not args.dry_run:
        Path(args.map).parent.mkdir(parents=True, exist_ok=True)
    return 0


def head_content(path: str) -> bytes | None:
    result = subprocess.run(["git", "show", f"HEAD:{path}"], cwd=ROOT, capture_output=True)
    return result.stdout if result.returncode == 0 else None


def cmd_check(args: argparse.Namespace) -> int:
    mapping = load_map(args.map)
    pairs = derived_rewrites(mapping)
    problems: list[str] = []
    before_of: dict[str, str] = {}
    for move in mapping["moves"]:
        src, dst = move["from"].rstrip("/"), move["to"].rstrip("/")
        if (ROOT / src).exists():
            problems.append(f"{src} still exists")
        if not (ROOT / dst).exists():
            problems.append(f"{dst} does not exist")
    def before(path: str) -> str:
        for move in mapping["moves"]:
            src, dst = move["from"].rstrip("/"), move["to"].rstrip("/")
            if path == dst:
                return src
            if path.startswith(dst + "/"):
                return src + path[len(dst):]
        return path
    for path in sorted(retired(mapping)):
        if (ROOT / path).exists():
            problems.append(f"{path} was merged or split away and still exists")
    for _, destinations in groups(mapping):
        for path in destinations:
            if not (ROOT / path).is_file():
                problems.append(f"{path} does not exist")
    grouped = {p for sources, destinations in groups(mapping) for p in sources + destinations}
    notices = check_groups(mapping, problems)
    # No from-string survives in the text scope (unguarded pairs only; basename
    # pairs may legitimately reappear as other files' names). A retired split
    # source is a from-string too.
    gone = [(stem(p), "", False) for p in retired(mapping) if not any(p == m["from"] for m in mapping["merges"])]
    survivors = 0
    for path in text_scope():
        text = (ROOT / path).read_text(encoding="utf-8")
        for src, _dst, guarded in pairs + gone:
            if guarded:
                continue
            if src in text:
                survivors += 1
                if survivors <= 25:
                    problems.append(f"{path} still mentions {src!r}")
    for path in pins_files():
        text = (ROOT / path).read_text(encoding="utf-8")
        for line in text.split("\n"):
            if line.lstrip().startswith(("#", ":require")):
                for src, _dst, guarded in pairs:
                    if not guarded and src in line:
                        problems.append(f"{path} comment still mentions {src!r}")
                        break
    # Content that must not change, compared to HEAD through the rename.
    for path in tracked():
        if matches(path, NEVER_TOUCHED):
            old = head_content(before(path))
            new = (ROOT / path).read_bytes()
            if old is None:
                problems.append(f"{path}: no HEAD counterpart at {before(path)}")
            elif old != new:
                problems.append(f"{path}: content changed, and it is in the never-touched set")
    for path in pins_files():
        if path in grouped:
            continue
        old = head_content(before(path))
        if old is None:
            problems.append(f"{path}: pins file has no HEAD counterpart at {before(path)}")
            continue
        strip = lambda b: [l for l in b.decode("utf-8").split("\n") if not l.lstrip().startswith(("#", ":require"))]
        if strip(old) != strip((ROOT / path).read_bytes()):
            problems.append(f"{path}: a non-comment pins line changed")
    # git status: deletions are moves, additions are moves, modifications are in scope.
    expected_from = {m["from"].rstrip("/") for m in mapping["moves"]} | retired(mapping)
    expected_to = {m["to"].rstrip("/") for m in mapping["moves"]} | created(mapping)
    scope = set(text_scope()) | set(pins_files())
    for line in git("status", "--porcelain", "-uall", "--no-renames").split("\n"):
        if not line.strip():
            continue
        status, path = line[:2], line[3:]
        under_from = any(path == f or path.startswith(f + "/") for f in expected_from)
        under_to = any(path == t or path.startswith(t + "/") for t in expected_to)
        if "D" in status and not under_from:
            problems.append(f"{path} was deleted and is not a planned move")
        elif ("A" in status or "?" in status) and not (under_to or path.startswith("tools/maps/")):
            problems.append(f"{path} was added and is not a planned move")
        elif status.strip() == "M" and path not in scope and not path.startswith("tools/maps/"):
            problems.append(f"{path} was modified and is outside the text scope")
    if notices:
        print(f"{len(notices)} source paragraph(s) no destination carries verbatim; review each seam:")
        print("\n".join("  " + line for line in notices))
    if problems:
        print("\n".join(problems))
        print(f"check: {len(problems)} problem(s)")
        return 1
    print("check: clean — every planned move landed, no reference survives, nothing else changed")
    return 0


def pin_statements(text: str) -> tuple[list[str], int]:
    """A pins file's statements, comments, blank lines and `:expect-pins` aside,
    and the pin count it declares."""
    lines, expected = [], 0
    for line in text.split("\n"):
        stripped = line.strip()
        if not stripped or stripped.startswith(("#", ":require")):
            continue
        if stripped.startswith(":expect-pins"):
            expected += int(stripped.split()[1])
            continue
        lines.append(stripped)
    return lines, expected


def paragraphs(text: str) -> list[str]:
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    out = []
    for block in re.split(r"\n\s*\n", text):
        block = " ".join(block.split())
        if block and not block.startswith("#"):
            out.append(block)
    return out


def check_groups(mapping: dict, problems: list[str]) -> list[str]:
    """Hold every merge and split to account against HEAD."""
    from collections import Counter
    notices: list[str] = []
    for sources, destinations in groups(mapping):
        old_pins = [p for p in sources if p.endswith(".pins.nibli")]
        new_pins = [p for p in destinations if p.endswith(".pins.nibli")]
        if old_pins or new_pins:
            before_lines: Counter = Counter()
            before_total = 0
            for path in old_pins:
                content = head_content(path)
                if content is None:
                    problems.append(f"{path}: no HEAD counterpart")
                    continue
                lines, total = pin_statements(content.decode("utf-8"))
                before_lines.update(lines)
                before_total += total
            after_lines: Counter = Counter()
            after_total = 0
            for path in new_pins:
                if (ROOT / path).is_file():
                    lines, total = pin_statements((ROOT / path).read_text(encoding="utf-8"))
                    after_lines.update(lines)
                    after_total += total
            label = " + ".join(old_pins) + " -> " + " + ".join(new_pins)
            lost = before_lines - after_lines
            invented = [line for line in after_lines if line not in before_lines]
            if lost:
                problems.append(f"{label}: {sum(lost.values())} statement(s) lost, e.g. {next(iter(lost))!r}")
            if invented:
                problems.append(f"{label}: {len(invented)} statement(s) no source held, e.g. {invented[0]!r}")
            if len(new_pins) == 1 and after_lines != before_lines and not lost:
                problems.append(f"{label}: a merge repeats a statement its sources held once")
            if before_total != after_total:
                problems.append(f"{label}: :expect-pins totals {before_total} before and {after_total} after")
        old_prose = [p for p in sources if p.endswith(".md")]
        new_prose = [p for p in destinations if p.endswith(".md")]
        carried = " ".join(" ".join((ROOT / p).read_text(encoding="utf-8").split())
                           for p in new_prose if (ROOT / p).is_file())
        for path in old_prose:
            content = head_content(path)
            if content is None:
                continue
            for paragraph in paragraphs(content.decode("utf-8")):
                if paragraph not in carried:
                    notices.append(f"{path}: {paragraph[:110]}")
    return notices


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    plan = sub.add_parser("plan"); plan.add_argument("--out"); plan.add_argument("--name", default="reorder"); plan.add_argument("--date", default="")
    apply_ = sub.add_parser("apply"); apply_.add_argument("map"); apply_.add_argument("--dry-run", action="store_true"); apply_.add_argument("--allow-dirty", action="store_true")
    check = sub.add_parser("check"); check.add_argument("map")
    resort = sub.add_parser("resort"); resort.add_argument("map"); resort.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    return {"plan": cmd_plan, "apply": cmd_apply, "check": cmd_check, "resort": cmd_resort}[args.command](args)


def cmd_resort(args: argparse.Namespace) -> int:
    """Re-sort the reader-coverage ledger after hand repairs, without relabelling."""
    changed = resort_reader_coverage({}, args.dry_run, load_map(args.map))
    print(f"{'would re-key' if args.dry_run else 're-keyed'} {changed} reader-coverage ids")
    return 0


if __name__ == "__main__":
    sys.exit(main())
