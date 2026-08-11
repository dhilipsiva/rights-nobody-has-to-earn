#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Build deterministic, accessible Book 1 artifacts for reader-study pilots.

These outputs are study snapshots, not editions or release candidates.  The
builder deliberately uses only Python's standard library so the structural
check can run anywhere the repository verifier runs.  A PDF is rendered from
the generated HTML by a browser outside this script.  It is bound only when a
prior manifest proves that the retained PDF came from the same generated HTML.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import subprocess
import tempfile
import xml.etree.ElementTree as ET
import zipfile
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parent.parent
BOOK = ROOT / "book-1"
OUTPUT_BASENAME = "book-1-pilot-snapshot"
FIXED_ZIP_TIME = (1980, 1, 1, 0, 0, 0)
MANIFEST_SCHEMA = "book-1-pilot-snapshot-manifest/v1"
ARTIFACT_FORMAT = "book-1-pilot-reader-artifacts/v1"

REQUIRED_OPENING_SECTIONS = (
    "Reader's Map",
    "Annotated contents",
    "Concise glossary",
    "Roles, bodies, and cases",
    "Domains and chapters",
    "Accessible diagrams",
)

CSS = """
:root { color-scheme: light; font-family: Georgia, 'Times New Roman', serif; }
body { color: #171717; background: #fff; line-height: 1.62; margin: 0 auto;
       max-width: 48rem; padding: 1.25rem; }
a { color: #0645ad; text-decoration-thickness: .08em; }
a:focus { outline: .2rem solid #b45309; outline-offset: .15rem; }
.skip-link { position: absolute; left: -10000px; top: auto; }
.skip-link:focus { left: 1rem; top: 1rem; background: #fff; padding: .6rem;
                   z-index: 2; }
header[role=banner] { border-bottom: .12rem solid #555; margin-bottom: 2rem; }
.status { border: .12rem solid #7c2d12; padding: .8rem 1rem; font-weight: 700; }
nav ol { padding-left: 1.4rem; }
main article { break-before: page; page-break-before: always; }
main article:first-child { break-before: auto; page-break-before: auto; }
h1, h2, h3, h4 { line-height: 1.22; break-after: avoid; }
blockquote { border-left: .25rem solid #777; margin-left: 0; padding-left: 1rem; }
pre { overflow-x: auto; border: .08rem solid #777; padding: .8rem; white-space: pre-wrap; }
code { font-family: ui-monospace, Consolas, monospace; }
table { border-collapse: collapse; width: 100%; }
th, td { border: .08rem solid #777; padding: .45rem; text-align: left; vertical-align: top; }
caption { font-weight: 700; text-align: left; margin-bottom: .4rem; }
@media print {
  body { max-width: none; padding: 0; font-size: 11.5pt; }
  nav a { color: #171717; text-decoration: none; }
  a[href^='http']::after { content: ' (' attr(href) ')'; font-size: 85%; }
  @page { size: A4; margin: 20mm 18mm 22mm; }
}
""".strip()


class ArtifactError(RuntimeError):
    """Raised when a source or generated accessibility contract is invalid."""


@dataclass(frozen=True)
class SourceDocument:
    path: Path
    title: str
    body_html: str
    heading_ids: tuple[str, ...]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def ordered_inputs() -> list[Path]:
    numbered = sorted(BOOK.glob("[0-9][0-9]-*.md"))
    paths = [BOOK / "epigraph.md", *numbered, BOOK / "method.md"]
    missing = [str(path.relative_to(ROOT)) for path in paths if not path.is_file()]
    if missing:
        raise ArtifactError(f"missing ordered input(s): {', '.join(missing)}")
    if BOOK / "00-opening-note.md" not in paths:
        raise ArtifactError("ordered inputs omit book-1/00-opening-note.md")
    return paths


def slug(value: str) -> str:
    plain = re.sub(r"[`*_\[\]()]", "", value).lower()
    result = re.sub(r"[^a-z0-9]+", "-", plain).strip("-")
    return result or "section"


def strip_inline_markdown(value: str) -> str:
    value = re.sub(r"!\[([^]]*)\]\([^)]+\)", r"\1", value)
    value = re.sub(r"\[([^]]+)\]\([^)]+\)", r"\1", value)
    return re.sub(r"[`*_]", "", value).strip()


def render_inline(value: str) -> str:
    tokens: list[str] = []

    def protect_code(match: re.Match[str]) -> str:
        tokens.append(f"<code>{html.escape(match.group(1))}</code>")
        return f"\x00{len(tokens) - 1}\x00"

    value = re.sub(r"`([^`]+)`", protect_code, value)
    escaped = html.escape(value, quote=True)
    escaped = re.sub(
        r"!\[([^]]+)\]\(([^)]+)\)",
        lambda match: (
            f'<img src="{html.escape(match.group(2), quote=True)}" '
            f'alt="{html.escape(match.group(1), quote=True)}"/>'
        ),
        escaped,
    )
    escaped = re.sub(
        r"\[([^]]+)\]\(([^)]+)\)",
        lambda match: (
            f'<a href="{html.escape(match.group(2), quote=True)}">{match.group(1)}</a>'
        ),
        escaped,
    )
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", escaped)
    escaped = re.sub(r"(?<!_)_([^_]+)_(?!_)", r"<em>\1</em>", escaped)
    for index, token in enumerate(tokens):
        escaped = escaped.replace(f"\x00{index}\x00", token)
    return escaped


def markdown_to_html(text: str, source_stem: str) -> tuple[str, str, tuple[str, ...]]:
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    output: list[str] = []
    paragraph: list[str] = []
    quote: list[str] = []
    code_lines: list[str] = []
    table_lines: list[str] = []
    list_kind: str | None = None
    in_code = False
    in_comment = False
    title = ""
    heading_ids: list[str] = []
    used_ids: set[str] = set()

    def flush_paragraph() -> None:
        if paragraph:
            if source_stem == "epigraph":
                content = "<br/>\n".join(
                    render_inline(part.strip()) for part in paragraph
                )
            else:
                content = render_inline(
                    " ".join(part.strip() for part in paragraph)
                )
            output.append(f"<p>{content}</p>")
            paragraph.clear()

    def flush_quote() -> None:
        if quote:
            output.append(f"<blockquote><p>{render_inline(' '.join(quote))}</p></blockquote>")
            quote.clear()

    def flush_list() -> None:
        nonlocal list_kind
        if list_kind is not None:
            output.append(f"</{list_kind}>")
            list_kind = None

    def flush_table() -> None:
        if not table_lines:
            return
        rows = [
            [cell.strip() for cell in line.strip().strip("|").split("|")]
            for line in table_lines
        ]
        if len(rows) >= 2 and all(re.fullmatch(r":?-{3,}:?", cell) for cell in rows[1]):
            output.append('<div role="region" aria-label="Table" tabindex="0"><table>')
            output.append("<thead><tr>" + "".join(f"<th scope=\"col\">{render_inline(cell)}</th>" for cell in rows[0]) + "</tr></thead>")
            output.append("<tbody>")
            for row in rows[2:]:
                output.append("<tr>" + "".join(f"<td>{render_inline(cell)}</td>" for cell in row) + "</tr>")
            output.append("</tbody></table></div>")
        else:
            for line in table_lines:
                output.append(f"<p>{render_inline(line)}</p>")
        table_lines.clear()

    def flush_all() -> None:
        flush_paragraph()
        flush_quote()
        flush_table()
        flush_list()

    for raw in lines:
        line = raw.rstrip()
        if line.startswith("```"):
            flush_all()
            if in_code:
                output.append(f"<pre><code>{html.escape(chr(10).join(code_lines))}</code></pre>")
                code_lines.clear()
                in_code = False
            else:
                in_code = True
            continue
        if in_code:
            code_lines.append(line)
            continue
        if in_comment:
            if "-->" not in line:
                continue
            line = line.split("-->", 1)[1]
            in_comment = False
        while "<!--" in line:
            prefix, remainder = line.split("<!--", 1)
            if "-->" in remainder:
                line = prefix + remainder.split("-->", 1)[1]
            else:
                line = prefix
                in_comment = True
                break
        if not line.strip() and in_comment:
            continue
        if line.startswith("|") and line.endswith("|"):
            flush_paragraph()
            flush_quote()
            flush_list()
            table_lines.append(line)
            continue
        flush_table()
        heading = re.match(r"^(#{1,6})\s+(.+?)\s*$", line)
        if heading:
            flush_all()
            level = len(heading.group(1))
            visible = strip_inline_markdown(heading.group(2))
            if not title and level == 1:
                title = visible
            base = f"{source_stem}-{slug(visible)}"
            heading_id = base
            suffix = 2
            while heading_id in used_ids:
                heading_id = f"{base}-{suffix}"
                suffix += 1
            used_ids.add(heading_id)
            heading_ids.append(heading_id)
            output.append(f'<h{level} id="{heading_id}">{render_inline(heading.group(2))}</h{level}>')
            continue
        unordered = re.match(r"^[-*]\s+(.+)$", line)
        ordered = re.match(r"^\d+[.)]\s+(.+)$", line)
        if unordered or ordered:
            flush_paragraph()
            flush_quote()
            kind = "ul" if unordered else "ol"
            if list_kind != kind:
                flush_list()
                output.append(f"<{kind}>")
                list_kind = kind
            item = unordered.group(1) if unordered else ordered.group(1)
            output.append(f"<li>{render_inline(item)}</li>")
            continue
        if line.startswith("> "):
            flush_paragraph()
            flush_list()
            quote.append(line[2:].strip())
            continue
        if re.fullmatch(r"-{3,}", line.strip()):
            flush_all()
            output.append("<hr/>")
            continue
        if not line.strip():
            flush_all()
            continue
        flush_quote()
        flush_list()
        paragraph.append(line)

    if in_code:
        raise ArtifactError(f"{source_stem}: unclosed fenced code block")
    if in_comment:
        raise ArtifactError(f"{source_stem}: unclosed HTML comment")
    flush_all()
    if not title:
        if source_stem != "epigraph":
            raise ArtifactError(f"{source_stem}: missing level-one title")
        title = "Epigraph"
        heading_ids.insert(0, "epigraph-epigraph")
        output.insert(0, '<h1 id="epigraph-epigraph">Epigraph</h1>')
    return title, "\n".join(output), tuple(heading_ids)


def read_documents() -> list[SourceDocument]:
    documents: list[SourceDocument] = []
    for path in ordered_inputs():
        title, body, heading_ids = markdown_to_html(path.read_text(encoding="utf-8"), path.stem)
        documents.append(SourceDocument(path, title, body, heading_ids))
    return documents


def source_fragment_targets(document: SourceDocument) -> dict[str, str]:
    """Map source Markdown fragments to generated globally unique heading IDs."""
    targets: dict[str, str] = {}
    counts: dict[str, int] = {}
    heading_index = 0
    text = document.path.read_text(encoding="utf-8")
    for match in re.finditer(r"^(#{1,6})\s+(.+?)\s*$", text, re.M):
        visible = strip_inline_markdown(match.group(2))
        base = slug(visible)
        count = counts.get(base, 0) + 1
        counts[base] = count
        source_fragment = base if count == 1 else f"{base}-{count - 1}"
        if heading_index >= len(document.heading_ids):
            raise ArtifactError(
                f"{document.path.relative_to(ROOT)}: heading map is incomplete"
            )
        targets[source_fragment] = document.heading_ids[heading_index]
        heading_index += 1
    expected_source_headings = len(document.heading_ids)
    if document.path.name == "epigraph.md":
        expected_source_headings -= 1
    if heading_index != expected_source_headings:
        raise ArtifactError(
            f"{document.path.relative_to(ROOT)}: heading map does not match rendered IDs"
        )
    return targets


def external_href(value: str) -> bool:
    parsed = urlsplit(value)
    return bool(parsed.scheme or parsed.netloc or value.startswith("//"))


def resolve_body_href(
    value: str,
    source: SourceDocument,
    documents: list[SourceDocument],
    output_context: str,
) -> str:
    if external_href(value):
        return value
    parsed = urlsplit(value)
    if parsed.query:
        raise ArtifactError(
            f"{source.path.relative_to(ROOT)}: local link queries are unsupported: {value}"
        )
    raw_path = unquote(parsed.path)
    if raw_path:
        target_path = (source.path.parent / raw_path).resolve()
    else:
        target_path = source.path.resolve()
    by_path = {document.path.resolve(): document for document in documents}
    target = by_path.get(target_path)
    if target is None:
        raise ArtifactError(
            f"{source.path.relative_to(ROOT)}: local link is not an ordered input: {value}"
        )
    if parsed.fragment:
        fragment = unquote(parsed.fragment)
        generated_id = source_fragment_targets(target).get(fragment)
        if generated_id is None:
            raise ArtifactError(
                f"{source.path.relative_to(ROOT)}: local link fragment does not exist: {value}"
            )
    else:
        generated_id = target.heading_ids[0]
    if output_context == "html":
        return f"#{generated_id}"
    if output_context == "epub":
        chapter_index = documents.index(target) + 1
        return f"chapter-{chapter_index:02d}.xhtml#{generated_id}"
    raise ArtifactError(f"unknown link output context: {output_context}")


def rewrite_document_links(
    documents: list[SourceDocument],
    output_context: str,
) -> list[SourceDocument]:
    rewritten: list[SourceDocument] = []
    for source in documents:
        def replace_href(match: re.Match[str]) -> str:
            original = html.unescape(match.group(1))
            resolved = resolve_body_href(
                original, source, documents, output_context
            )
            return f'<a href="{html.escape(resolved, quote=True)}">'

        body = re.sub(r'<a href="([^"]+)">', replace_href, source.body_html)
        rewritten.append(
            SourceDocument(source.path, source.title, body, source.heading_ids)
        )

    known_ids = {
        heading_id
        for document in rewritten
        for heading_id in document.heading_ids
    }
    for document in rewritten:
        for encoded in re.findall(r'<a href="([^"]+)">', document.body_html):
            href = html.unescape(encoded)
            if external_href(href):
                continue
            if output_context == "html":
                if not href.startswith("#") or href[1:] not in known_ids:
                    raise ArtifactError(
                        f"{document.path.relative_to(ROOT)}: invalid combined-HTML href: {href}"
                    )
                continue
            match = re.fullmatch(
                r"chapter-(\d{2})\.xhtml#([a-z0-9-]+)", href
            )
            if match is None:
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: invalid EPUB href: {href}"
                )
            chapter_index = int(match.group(1))
            if not 1 <= chapter_index <= len(rewritten):
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: EPUB chapter target is absent: {href}"
                )
            if match.group(2) not in rewritten[chapter_index - 1].heading_ids:
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: EPUB fragment target is absent: {href}"
                )
    return rewritten


def html_document(documents: list[SourceDocument], snapshot_id: str) -> bytes:
    documents = rewrite_document_links(documents, "html")
    nav = "\n".join(
        f'<li><a href="#{document.heading_ids[0]}">{html.escape(document.title)}</a></li>'
        for document in documents
    )
    articles = "\n".join(
        f'<article aria-labelledby="{document.heading_ids[0]}">{document.body_html}</article>'
        for document in documents
    )
    value = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>The Rights Nobody Has to Earn - pilot study snapshot</title>
<style>{CSS}</style>
</head>
<body>
<a class="skip-link" href="#main-content">Skip to the book</a>
<header role="banner">
<p class="status">Pilot study snapshot. Not an edition, release candidate, or Gate C artifact.</p>
<p>Snapshot identifier: <code>{snapshot_id}</code></p>
<nav aria-label="Book contents"><h2>Contents</h2><ol>{nav}</ol></nav>
</header>
<main id="main-content">{articles}</main>
</body>
</html>
"""
    return value.encode("utf-8")


def xhtml_document(document: SourceDocument) -> bytes:
    return f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en" xml:lang="en">
<head><meta charset="utf-8"/><title>{html.escape(document.title)}</title>
<link rel="stylesheet" type="text/css" href="style.css"/></head>
<body><main><article aria-labelledby="{document.heading_ids[0]}">
{document.body_html}
</article></main></body></html>
""".encode("utf-8")


def zip_write(archive: zipfile.ZipFile, name: str, value: bytes, *, compress: bool = True) -> None:
    info = zipfile.ZipInfo(name, FIXED_ZIP_TIME)
    info.compress_type = zipfile.ZIP_DEFLATED if compress else zipfile.ZIP_STORED
    info.external_attr = 0o644 << 16
    archive.writestr(info, value)


def epub_document(documents: list[SourceDocument], snapshot_id: str) -> bytes:
    documents = rewrite_document_links(documents, "epub")
    with tempfile.NamedTemporaryFile(suffix=".epub") as stream:
        with zipfile.ZipFile(stream.name, "w") as archive:
            zip_write(archive, "mimetype", b"application/epub+zip", compress=False)
            zip_write(
                archive,
                "META-INF/container.xml",
                b'''<?xml version="1.0"?>\n<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="EPUB/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>\n''',
            )
            manifest_items = []
            spine_items = []
            nav_items = []
            for index, document in enumerate(documents, start=1):
                item_id = f"chapter-{index:02d}"
                filename = f"{item_id}.xhtml"
                zip_write(archive, f"EPUB/{filename}", xhtml_document(document))
                manifest_items.append(
                    f'<item id="{item_id}" href="{filename}" media-type="application/xhtml+xml"/>'
                )
                spine_items.append(f'<itemref idref="{item_id}"/>')
                nav_items.append(
                    f'<li><a href="{filename}#{document.heading_ids[0]}">{html.escape(document.title)}</a></li>'
                )
            nav = f'''<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en"><head><title>Contents</title></head><body><nav epub:type="toc" aria-label="Book contents"><h1>Contents</h1><ol>{''.join(nav_items)}</ol></nav></body></html>'''.encode("utf-8")
            zip_write(archive, "EPUB/nav.xhtml", nav)
            zip_write(archive, "EPUB/style.css", CSS.encode("utf-8"))
            package = f'''<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id" xml:lang="en">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="book-id">urn:sha256:{snapshot_id}</dc:identifier><dc:title>The Rights Nobody Has to Earn - pilot study snapshot</dc:title><dc:language>en</dc:language><dc:rights>CC-BY-4.0</dc:rights><meta property="dcterms:modified">1980-01-01T00:00:00Z</meta></metadata>
<manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="css" href="style.css" media-type="text/css"/>{''.join(manifest_items)}</manifest>
<spine>{''.join(spine_items)}</spine></package>'''.encode("utf-8")
            zip_write(archive, "EPUB/package.opf", package)
        return Path(stream.name).read_bytes()


def source_manifest(paths: list[Path]) -> list[dict[str, str]]:
    return [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256_file(path)}
        for path in paths
    ]


def generator_manifest() -> dict[str, str]:
    path = Path(__file__).resolve()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "sha256": sha256_file(path),
    }


def snapshot_identity(
    inputs: list[dict[str, str]], generator: dict[str, str]
) -> dict[str, object]:
    return {
        "manifest_schema": MANIFEST_SCHEMA,
        "artifact_format": ARTIFACT_FORMAT,
        "generator": generator,
        "ordered_inputs": inputs,
    }


def snapshot_identifier(identity: dict[str, object]) -> str:
    encoded = json.dumps(identity, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return sha256_bytes(encoded)


def git_source_revision(paths: list[Path], *, require_match: bool) -> dict[str, object]:
    try:
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        if require_match:
            raise ArtifactError("cannot resolve the committed source revision") from exc
        return {
            "vcs": "git",
            "commit": None,
            "bound_paths_match_commit": False,
            "mismatched_paths": [path.relative_to(ROOT).as_posix() for path in paths],
        }

    mismatched: list[str] = []
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        try:
            committed = subprocess.run(
                ["git", "show", f"{commit}:{relative}"],
                cwd=ROOT,
                check=True,
                capture_output=True,
            ).stdout
        except (OSError, subprocess.CalledProcessError):
            mismatched.append(relative)
            continue
        if committed != path.read_bytes():
            mismatched.append(relative)
    if require_match and mismatched:
        raise ArtifactError(
            "freeze-grade output requires committed bound paths; mismatch: "
            + ", ".join(mismatched)
        )
    return {
        "vcs": "git",
        "commit": commit,
        "bound_paths_match_commit": not mismatched,
        "mismatched_paths": mismatched,
    }


def validate_sources(documents: list[SourceDocument]) -> None:
    opening = (BOOK / "00-opening-note.md").read_text(encoding="utf-8")
    for section in REQUIRED_OPENING_SECTIONS:
        if f"## {section}" not in opening:
            raise ArtifactError(f"opening note missing required section: {section}")
    for document in documents:
        text = document.path.read_text(encoding="utf-8")
        for match in re.finditer(r"!\[([^]]*)\]\([^)]+\)", text):
            if not match.group(1).strip():
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: image lacks text alternative"
                )
        levels = [len(match.group(1)) for match in re.finditer(r"^(#{1,6})\s+", text, re.M)]
        if document.path.name == "epigraph.md":
            if levels:
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: epigraph must remain unheaded"
                )
            continue
        if not levels or levels[0] != 1:
            raise ArtifactError(f"{document.path.relative_to(ROOT)}: first heading must be H1")
        for before, after in zip(levels, levels[1:]):
            if after > before + 1:
                raise ArtifactError(
                    f"{document.path.relative_to(ROOT)}: heading level skips H{before} to H{after}"
                )


def validate_epub(value: bytes, expected_documents: int) -> None:
    with tempfile.NamedTemporaryFile(suffix=".epub") as stream:
        Path(stream.name).write_bytes(value)
        with zipfile.ZipFile(stream.name) as archive:
            names = archive.namelist()
            xml_entries = [
                name
                for name in names
                if name.endswith((".xml", ".xhtml", ".opf"))
            ]
            for name in xml_entries:
                try:
                    ET.fromstring(archive.read(name))
                except ET.ParseError as exc:
                    raise ArtifactError(f"EPUB XML is malformed: {name}: {exc}") from exc
            if names[0] != "mimetype":
                raise ArtifactError("EPUB mimetype must be the first archive member")
            info = archive.getinfo("mimetype")
            if info.compress_type != zipfile.ZIP_STORED:
                raise ArtifactError("EPUB mimetype must be uncompressed")
            if archive.read("mimetype") != b"application/epub+zip":
                raise ArtifactError("EPUB mimetype is invalid")
            chapters = [name for name in names if re.fullmatch(r"EPUB/chapter-\d{2}\.xhtml", name)]
            if len(chapters) != expected_documents:
                raise ArtifactError("EPUB spine does not contain every ordered input")
            if b'epub:type="toc"' not in archive.read("EPUB/nav.xhtml"):
                raise ArtifactError("EPUB navigation document lacks a table of contents")


def prior_pdf_output(
    manifest_path: Path,
    pdf_path: Path,
    *,
    snapshot_id: str,
    html_sha256: str,
) -> dict[str, str] | None:
    if not pdf_path.is_file():
        return None
    if not manifest_path.is_file():
        raise ArtifactError(
            "pre-existing PDF lacks the prior manifest required for binding"
        )
    try:
        prior = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ArtifactError("prior snapshot manifest is invalid JSON") from exc
    if not isinstance(prior, dict):
        raise ArtifactError("prior snapshot manifest must be a JSON object")
    if prior.get("snapshot_id") != snapshot_id:
        raise ArtifactError("pre-existing PDF belongs to a different snapshot identifier")
    prior_outputs = prior.get("outputs")
    if not isinstance(prior_outputs, list):
        raise ArtifactError("prior snapshot manifest outputs must be a list")
    prior_html = [
        item
        for item in prior_outputs
        if isinstance(item, dict) and item.get("format") == "html"
    ]
    if len(prior_html) != 1 or prior_html[0].get("sha256") != html_sha256:
        raise ArtifactError("pre-existing PDF source HTML digest does not match")
    pdf_bytes = pdf_path.read_bytes()
    if not pdf_bytes:
        raise ArtifactError("pre-existing PDF must be nonempty")
    if not pdf_bytes.startswith(b"%PDF-"):
        raise ArtifactError("pre-existing PDF lacks the %PDF- header")
    return {
        "format": "pdf",
        "path": pdf_path.name,
        "sha256": sha256_bytes(pdf_bytes),
        "source_html_sha256": html_sha256,
    }


def build(output_dir: Path, *, require_committed: bool = True) -> dict[str, object]:
    documents = read_documents()
    validate_sources(documents)
    source_paths = [document.path for document in documents]
    inputs = source_manifest(source_paths)
    generator = generator_manifest()
    identity = snapshot_identity(inputs, generator)
    snapshot_id = snapshot_identifier(identity)
    html_bytes = html_document(documents, snapshot_id)
    epub_bytes = epub_document(documents, snapshot_id)
    validate_epub(epub_bytes, len(documents))
    source_revision = git_source_revision(
        [*source_paths, Path(__file__).resolve()],
        require_match=require_committed,
    )

    html_path = output_dir / f"{OUTPUT_BASENAME}.html"
    epub_path = output_dir / f"{OUTPUT_BASENAME}.epub"
    pdf_path = output_dir / f"{OUTPUT_BASENAME}.pdf"
    manifest_path = output_dir / f"{OUTPUT_BASENAME}-manifest.json"
    html_sha256 = sha256_bytes(html_bytes)
    pdf_output = prior_pdf_output(
        manifest_path,
        pdf_path,
        snapshot_id=snapshot_id,
        html_sha256=html_sha256,
    )

    output_dir.mkdir(parents=True, exist_ok=True)
    html_path.write_bytes(html_bytes)
    epub_path.write_bytes(epub_bytes)
    outputs = [
        {"format": "html", "path": html_path.name, "sha256": html_sha256},
        {"format": "epub", "path": epub_path.name, "sha256": sha256_bytes(epub_bytes)},
    ]
    if pdf_output is not None:
        outputs.append(pdf_output)
    manifest = {
        "spdx": "CC-BY-4.0",
        "manifest_schema": MANIFEST_SCHEMA,
        "artifact_format": ARTIFACT_FORMAT,
        "artifact_status": "pilot-study-snapshot-not-an-edition",
        "snapshot_id": snapshot_id,
        "snapshot_identity": identity,
        "generator": generator,
        "source_revision": source_revision,
        "ordered_inputs": inputs,
        "outputs": outputs,
        "pdf_source": html_path.name,
        "accessibility_contract": {
            "semantic_navigation": True,
            "text_alternatives_required": True,
            "layout_or_colour_only_meaning_prohibited": True,
            "human_screen_reader_attestation": "external-pending",
        },
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def watched_link_controls(documents: list[SourceDocument]) -> int:
    source_index = next(
        index
        for index, document in enumerate(documents)
        if document.path.name == "00-opening-note.md"
    )
    source = documents[source_index]
    controls = (
        ("missing local file", "does-not-exist.md", "not an ordered input"),
        (
            "missing local fragment",
            "15-the-five-joints.md#does-not-exist",
            "fragment does not exist",
        ),
    )
    watched = 0
    for output_context in ("html", "epub"):
        for label, href, expected in controls:
            mutated = list(documents)
            mutated[source_index] = SourceDocument(
                source.path,
                source.title,
                source.body_html + f'<p><a href="{href}">link control</a></p>',
                source.heading_ids,
            )
            try:
                rewrite_document_links(mutated, output_context)
            except ArtifactError as exc:
                if expected not in str(exc):
                    raise ArtifactError(
                        f"{label} failed for the wrong reason in {output_context}: {exc}"
                    ) from exc
                watched += 1
            else:
                raise ArtifactError(
                    f"{label} did not fail in {output_context}"
                )
    return watched


def watched_pdf_controls(base: Path) -> int:
    valid_dir = base / "valid-pdf-binding"
    initial = build(valid_dir, require_committed=False)
    pdf_path = valid_dir / f"{OUTPUT_BASENAME}.pdf"
    pdf_path.write_bytes(b"%PDF-1.7\n%%EOF\n")
    rebound = build(valid_dir, require_committed=False)
    pdf_outputs = [
        item for item in rebound["outputs"] if item["format"] == "pdf"
    ]
    html_outputs = [
        item for item in rebound["outputs"] if item["format"] == "html"
    ]
    if len(pdf_outputs) != 1 or len(html_outputs) != 1:
        raise ArtifactError("valid PDF binding did not produce one HTML and one PDF")
    if pdf_outputs[0].get("source_html_sha256") != html_outputs[0]["sha256"]:
        raise ArtifactError("valid PDF binding did not bind its source HTML digest")
    if initial["snapshot_id"] != rebound["snapshot_id"]:
        raise ArtifactError("adding a valid PDF changed the snapshot identifier")

    controls = (
        ("stale snapshot", b"%PDF-1.7\n%%EOF\n", "snapshot", "different snapshot"),
        ("stale HTML", b"%PDF-1.7\n%%EOF\n", "html", "HTML digest"),
        ("empty PDF", b"", None, "nonempty"),
        ("wrong PDF header", b"not-a-pdf", None, "%PDF-"),
    )
    watched = 0
    for label, pdf_bytes, mutation, expected in controls:
        control_dir = base / slug(label)
        manifest = build(control_dir, require_committed=False)
        control_pdf = control_dir / f"{OUTPUT_BASENAME}.pdf"
        control_manifest = control_dir / f"{OUTPUT_BASENAME}-manifest.json"
        control_pdf.write_bytes(pdf_bytes)
        if mutation == "snapshot":
            manifest["snapshot_id"] = "0" * 64
        elif mutation == "html":
            html_output = next(
                item for item in manifest["outputs"] if item["format"] == "html"
            )
            html_output["sha256"] = "0" * 64
        if mutation is not None:
            control_manifest.write_text(
                json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
            )
        try:
            build(control_dir, require_committed=False)
        except ArtifactError as exc:
            if expected not in str(exc):
                raise ArtifactError(
                    f"{label} failed for the wrong reason: {exc}"
                ) from exc
            watched += 1
        else:
            raise ArtifactError(f"{label} PDF control did not fail")
    return watched


def check() -> None:
    with tempfile.TemporaryDirectory(prefix="book-1-pilot-artifacts-") as directory:
        root = Path(directory)
        first = build(root / "first", require_committed=False)
        second = build(root / "second", require_committed=False)
        if first["snapshot_id"] != second["snapshot_id"]:
            raise ArtifactError("snapshot identifier is not deterministic")
        first_outputs = [(item["format"], item["sha256"]) for item in first["outputs"]]
        second_outputs = [(item["format"], item["sha256"]) for item in second["outputs"]]
        if first_outputs != second_outputs:
            raise ArtifactError("generated HTML/EPUB bytes are not deterministic")
        identity = first.get("snapshot_identity")
        if not isinstance(identity, dict):
            raise ArtifactError("manifest lacks the structured snapshot identity")
        if snapshot_identifier(identity) != first["snapshot_id"]:
            raise ArtifactError("manifest snapshot identity does not reproduce its identifier")
        revision = first.get("source_revision")
        if not isinstance(revision, dict) or not revision.get("commit"):
            raise ArtifactError("manifest lacks Git source-revision provenance")
        link_controls = watched_link_controls(read_documents())
        pdf_controls = watched_pdf_controls(root / "pdf-controls")
        print(
            "15-pilot-reader-artifacts: ordered inputs and accessible HTML/EPUB "
            f"are structurally valid and deterministic; {link_controls} missing-link "
            f"and {pdf_controls} stale/invalid-PDF mutations watched failing; PDF "
            "rendering and human screen-reader attestation remain external"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true")
    group.add_argument("--output-dir", type=Path)
    args = parser.parse_args()
    try:
        if args.check:
            check()
        else:
            manifest = build(args.output_dir.resolve())
            print(
                f"15-pilot-reader-artifacts: wrote pilot HTML/EPUB and manifest "
                f"for snapshot {manifest['snapshot_id']}"
            )
        return 0
    except (ArtifactError, OSError, ValueError, zipfile.BadZipFile) as exc:
        print(f"15-pilot-reader-artifacts: {exc}", file=__import__("sys").stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
