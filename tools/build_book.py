#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "markdown-it-py==4.2.0",
#   "mdit-py-plugins==0.6.1",
#   "playwright==1.63.0",
#   "pymupdf==1.28.2",
# ]
# ///
"""Assemble the current Book 1 reading sequence as HTML, EPUB and PDF.

Explicit authoring only: this does not execute constitutional verification,
certify a release, freeze a study, or create provenance receipts.
"""

from __future__ import annotations

import argparse
import base64
import copy
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import partial
import html
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import re
import threading
from urllib.parse import quote, unquote, urlsplit
import xml.etree.ElementTree as ET
import zipfile

from markdown_it import MarkdownIt
from mdit_py_plugins.footnote import footnote_plugin


ROOT = Path(__file__).resolve().parent.parent
BOOK = ROOT / "book-1"
ASSETS = Path(__file__).resolve().parent / "book_assets"
TITLE = "The Rights Nobody Has to Earn"
AUTHOR = "dhilipsiva"
REPOSITORY = "https://github.com/dhilipsiva/rights-nobody-has-to-earn/blob/main/"
EPUB_NS = "http://www.idpf.org/2007/ops"
SAMPLE_CHAPTERS = (1, 4, 8, 21, 29)
UI_PREFIX = "/rights-nobody-has-to-earn/"
UI_ORIGIN = "https://dhilipsiva.dev"


@dataclass
class Document:
    path: Path
    title: str
    number: int | None
    part: str | None
    tree: ET.Element
    targets: dict[str, str]

    @property
    def stem(self) -> str:
        return self.path.stem

    @property
    def label(self) -> str:
        return f"{self.number}. {self.title}" if self.number else self.title


def slug(text: str) -> str:
    """The source's GitHub-style heading fragments, including Unicode letters."""
    return re.sub(r"[^\w\- ]", "", text.lower()).replace(" ", "-")


def parser() -> MarkdownIt:
    return MarkdownIt("commonmark", {"html": True, "xhtmlOut": True}).enable("table").use(footnote_plugin)


def document(path: Path, number: int | None = None, part: str | None = None) -> Document:
    md = parser()
    env: dict = {}
    tokens = md.parse(path.read_text(encoding="utf-8"), env)
    # Approval and licensing comments belong in source, not on the printed page.
    # The manuscript needs no raw HTML; fail visibly if that changes.
    def remove_comments(items: list) -> list:
        result = []
        for token in items:
            if token.type in {"html_block", "html_inline"}:
                if not token.content.lstrip().startswith("<!--"):
                    raise ValueError(f"{path}: raw HTML requires an explicit rendering decision")
                continue
            if token.children:
                token.children = remove_comments(token.children)
            result.append(token)
        return result

    tokens = remove_comments(tokens)
    counts: dict[str, int] = {}
    title = ""
    previous_level = 0
    for index, token in enumerate(tokens):
        if token.type != "heading_open":
            continue
        level = int(token.tag[1])
        if level > previous_level + 1:
            raise ValueError(f"{path}: skipped heading level")
        previous_level = level
        text = "".join(t.content for t in tokens[index + 1].children or [] if t.type in {"text", "code_inline"})
        if level == 1:
            if title:
                raise ValueError(f"{path}: more than one chapter title")
            title = text
        base = slug(text)
        count = counts.get(base, 0)
        counts[base] = count + 1
        token.attrSet("id", base if count == 0 else f"{base}-{count}")
    body = md.renderer.render(tokens, md.options, env)
    tree = ET.fromstring(f"<div>{body}</div>")
    if path.name == "epigraph.md":
        if title:
            raise ValueError("The epigraph must remain unnumbered and unheaded in source")
        title = "Epigraph"
        heading = ET.Element("h1", {"id": "epigraph", "class": "sr-only"})
        heading.text = title
        tree.insert(0, heading)
        for paragraph in tree.findall("p"):
            if any("\u0b80" <= char <= "\u0bff" for char in "".join(paragraph.itertext())):
                paragraph.set("lang", "ta")
    elif not title:
        raise ValueError(f"{path}: missing chapter title")

    targets = {}
    for element in tree.iter():
        if original := element.get("id"):
            if original in targets:
                raise ValueError(f"{path}: duplicate fragment {original}")
            targets[original] = f"{path.stem}-{original}"
            element.set("id", targets[original])
        if element.tag == "th":
            element.set("scope", "col")
        if element.tag == "img":
            raise ValueError(f"{path}: image assets need an explicit inclusion and text-alternative decision")
        if element.get("class") == "footnotes":
            element.set("role", "doc-endnotes")
            heading = ET.Element("h2")
            heading.text = "Notes"
            element.insert(0, heading)
        if element.tag == "a" and element.get("href", "").startswith("#fn"):
            backlink = element.get("class") == "footnote-backref"
            element.set("role", "doc-backlink" if backlink else "doc-noteref")
            number_label = re.search(r"\d+", element.get("href", ""))
            label = number_label.group() if number_label else ""
            element.set("aria-label", f"Back to citation {label}" if backlink else f"Note {label}")
    if number:
        heading = tree.find("h1")
        assert heading is not None
        heading.text = f"{number}. {heading.text or ''}"
    table_number = 0
    for parent in list(tree.iter()):
        for index, child in enumerate(list(parent)):
            if child.tag == "table":
                table_number += 1
                wrapper = ET.Element("div", {
                    "class": "table-wrap", "role": "region", "tabindex": "0",
                    "aria-label": f"{title}: table {table_number}",
                })
                parent.remove(child)
                wrapper.append(child)
                parent.insert(index, wrapper)
    return Document(path.resolve(), title, number, part, tree, targets)


def read_documents() -> list[Document]:
    manifest = json.loads((BOOK / "contents.json").read_text(encoding="utf-8"))
    rows = [(name, None, None, None) for name in manifest["front"]]
    for index, part in enumerate(manifest["parts"], 1):
        label = f"Part {roman(index)} — {part['title']}"
        # A Part's opening case, once written, heads the Part; a reserved
        # chapter or opener has no file yet and is not an input.
        opener = part.get("opener", {})
        if opener.get("status") == "landed":
            rows.append((opener["file"], None, label, opener["title"]))
        rows.extend((c["file"], c["number"], label, c["title"])
                    for c in part["chapters"] if c["status"] == "landed")
    rows.extend((name, None, None, None) for name in manifest["back"])
    documents = []
    seen = set()
    for name, number, part, expected_title in rows:
        path = (BOOK / name).resolve()
        if not path.is_relative_to(BOOK) or path in seen:
            raise ValueError(f"Invalid or repeated reading input: {name}")
        seen.add(path)
        doc = document(path, number, part)
        if expected_title and doc.title != expected_title:
            raise ValueError(f"Manifest title differs from {name}: {doc.title}")
        documents.append(doc)
    return documents


def roman(number: int) -> str:
    result = ""
    for value, letters in [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
                           (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
                           (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")]:
        count, number = divmod(number, value)
        result += letters * count
    return result


def resolve_link(href: str, source: Document, documents: list[Document], mode: str) -> str:
    parsed = urlsplit(href)
    if parsed.scheme or parsed.netloc:
        if parsed.scheme not in {"https", "http", "mailto"}:
            raise ValueError(f"Unsupported link scheme: {href}")
        return href
    if parsed.query:
        raise ValueError(f"Local link queries are unsupported: {href}")
    path = (source.path.parent / unquote(parsed.path)).resolve() if parsed.path else source.path
    if not path.is_relative_to(ROOT) or not path.is_file():
        raise ValueError(f"{source.path}: missing local destination {href}")
    target = next((d for d in documents if d.path == path), None)
    if target:
        fragment = unquote(parsed.fragment)
        if fragment and fragment not in target.targets:
            raise ValueError(f"{source.path}: missing fragment {href}")
        anchor = target.targets[fragment] if fragment else target.stem
        if mode == "ui":
            return f"{UI_PREFIX}read/{target.stem}/#{anchor}"
        prefix = "" if mode == "html" else f"{target.stem}.xhtml"
        return f"{prefix}#{anchor}"
    # Formal source, fixtures, registry and instructions remain repository links.
    # They are checked as actual files rather than being mistaken for chapters.
    relative = path.relative_to(ROOT).as_posix()
    suffix = f"#{quote(unquote(parsed.fragment))}" if parsed.fragment else ""
    return REPOSITORY + quote(relative, safe="/") + suffix


def article(doc: Document, documents: list[Document], mode: str) -> str:
    tree = copy.deepcopy(doc.tree)
    for link in tree.iter("a"):
        link.set("href", resolve_link(link.get("href", ""), doc, documents, mode))
        if mode == "epub" and link.get("role") == "doc-noteref":
            link.set("epub:type", "noteref")
    body = "".join(ET.tostring(child, encoding="unicode", method="xml") for child in tree)
    part = f'<p class="part-label">{html.escape(doc.part)}</p>' if doc.part else ""
    kind = ' class="epigraph"' if doc.stem == "epigraph" else ""
    return f'<article id="{doc.stem}"{kind}>{part}{body}</article>'


def contents(documents: list[Document], mode: str) -> str:
    rows = []
    last_part = None
    for doc in documents:
        if doc.part != last_part:
            if last_part:
                rows.append('</ol></li>')
            if doc.part:
                rows.append(f'<li class="part"><span>{html.escape(doc.part)}</span><ol>')
            last_part = doc.part
        href = f"#{doc.stem}" if mode == "html" else f"{doc.stem}.xhtml#{doc.stem}"
        rows.append(f'<li><a href="{href}">{html.escape(doc.label)}</a></li>')
    if last_part:
        rows.append('</ol></li>')
    return "<ol>" + "".join(rows) + "</ol>"


def select_sample(documents: list[Document]) -> list[Document]:
    selected = [doc for doc in documents if doc.number in SAMPLE_CHAPTERS]
    if tuple(doc.number for doc in selected) != SAMPLE_CHAPTERS:
        raise ValueError('The publisher sample chapters must exist in reading order')
    return selected


def ui_markdown(doc: Document, documents: list[Document]) -> str:
    """Keep source Markdown; rewrite only link destinations via the same validator."""
    source = re.sub(r"<!--.*?-->", "", doc.path.read_text(encoding="utf-8"), flags=re.S)
    # Use parser offsets indirectly: destinations themselves are replaced, so nested
    # labels, balanced URL parentheses, note definitions and Tamil stay verbatim.
    for node in doc.tree.iter("a"):
        href = node.get("href", "")
        if node.get("role") in {"doc-noteref", "doc-backlink"}:
            continue
        resolved = resolve_link(href, doc, documents, "ui")
        if resolved != href:
            source = source.replace(f"]({href})", f"]({UI_ORIGIN}{resolved})" if resolved.startswith("/") else f"]({resolved})")
            source = re.sub(r"(?m)^(\s*\[[^\]]+\]:\s*)" + re.escape(href) + r"(?=\s|$)",
                            lambda m: m[1] + (UI_ORIGIN if resolved.startswith("/") else "") + resolved, source)
    return source.strip() + "\n"


def export_ui(output: Path, documents: list[Document]) -> None:
    """Export reader data, without introducing a second Markdown renderer."""
    output.mkdir(parents=True, exist_ok=True)
    pages = []
    for index, doc in enumerate(documents):
        body = article(doc, documents, "ui")
        text = " ".join(" ".join(doc.tree.itertext()).split())
        paragraphs = [" ".join(p.itertext()).strip() for p in doc.tree.findall("p")]
        description = next((p for p in paragraphs if len(p) > 40), doc.title)
        description = description[:197].rsplit(" ", 1)[0] + "…" if len(description) > 200 else description
        pages.append({
            "stem": doc.stem, "title": doc.title, "label": doc.label,
            "number": doc.number, "part": doc.part, "order": index,
            "path": f"{UI_PREFIX}read/{doc.stem}/",
            "canonical": f"{UI_ORIGIN}{UI_PREFIX}read/{doc.stem}/",
            "source": REPOSITORY + "book-1/" + doc.path.name,
            "description": description, "html": body, "text": text,
            "markdown": ui_markdown(doc, documents),
            "sections": [{"id": n.get("id"), "title": "".join(n.itertext()), "level": int(n.tag[1])}
                         for n in doc.tree.iter() if re.fullmatch(r"h[1-6]", n.tag) and n.get("id")],
            "previous": documents[index - 1].stem if index else None,
            "next": documents[index + 1].stem if index + 1 < len(documents) else None,
        })
    result = {"title": TITLE, "author": AUTHOR, "license": "CC-BY-4.0",
              "prefix": UI_PREFIX, "origin": UI_ORIGIN, "pages": pages,
              "word_count": sum(len(p["text"].split()) for p in pages)}
    (output / "book.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def cover(sample: bool = False) -> str:
    status = 'Book 1 · Selected chapters' if sample else 'Book 1 · Review copy'
    numbers = [str(n) for n in SAMPLE_CHAPTERS]
    listed = ", ".join(numbers[:-1]) + " and " + numbers[-1]
    selection = (f'<p>Chapters {listed}. Cross-references beyond this '
                 'selection open the public manuscript.</p>') if sample else ''
    return f'''<header class="cover"><h1>{TITLE}</h1>
<p>{AUTHOR}</p><p class="edition-status">{status}</p>{selection}
<p class="read-online">The current manuscript, with a companion that runs its cases in the
browser, is published at <a href="{UI_ORIGIN}{UI_PREFIX}">dhilipsiva.dev{UI_PREFIX}</a>.</p>
<p class="licence">Book prose: <a href="https://creativecommons.org/licenses/by/4.0/">CC BY 4.0</a>.
The repository contains separately licensed formal source, code and data;
see <a href="{REPOSITORY}LICENSING.md">the licence map</a>.
Tamil typography uses Noto Serif Tamil, Copyright 2022 The Noto Project Authors,
under the SIL Open Font License 1.1.</p></header>'''


def html_document(documents: list[Document], css: str, sample: bool = False) -> str:
    font = base64.b64encode((ASSETS / "NotoSerifTamil.ttf").read_bytes()).decode("ascii")
    css = css.replace('url("NotoSerifTamil.ttf")', f'url("data:font/ttf;base64,{font}")')
    licence = (ASSETS / "OFL-NotoSerifTamil.txt").read_text(encoding="utf-8")
    return f'''<!doctype html><html lang="en"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{TITLE}</title><meta name="author" content="{AUTHOR}"/>
<style>{css}</style><!-- Embedded font licence:\n{licence}\n--></head><body>
<a class="skip-link" href="#main-content">Skip to the book</a>{cover(sample)}
<nav class="book-contents" aria-label="Book contents"><h1>Contents</h1>{contents(documents, "html")}</nav>
<main id="main-content" tabindex="-1">{''.join(article(d, documents, "html") for d in documents)}</main>
</body></html>'''


def xhtml(title: str, body: str) -> str:
    return f'''<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="{EPUB_NS}" lang="en" xml:lang="en">
<head><meta charset="utf-8"/><title>{html.escape(title)}</title>
<link rel="stylesheet" href="book.css"/></head><body class="ebook">{body}</body></html>'''


def write_epub(path: Path, documents: list[Document], css: str, sample: bool = False) -> None:
    items = [('cover', 'cover.xhtml', 'application/xhtml+xml', ''),
             ('nav', 'nav.xhtml', 'application/xhtml+xml', ' properties="nav"'),
             ('css', 'book.css', 'text/css', ''),
             ('font', 'NotoSerifTamil.ttf', 'font/ttf', ''),
             ('font-licence', 'OFL-NotoSerifTamil.txt', 'text/plain', '')]
    items += [(f"doc-{d.stem}", f"{d.stem}.xhtml", 'application/xhtml+xml', '') for d in documents]
    modified = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    manifest = ''.join(f'<item id="{ident}" href="{name}" media-type="{mime}"{extra}/>' for ident, name, mime, extra in items)
    spine = '<itemref idref="cover"/>' + ''.join(f'<itemref idref="doc-{d.stem}"/>' for d in documents)
    package = f'''<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id" xml:lang="en">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="book-id">https://github.com/dhilipsiva/rights-nobody-has-to-earn/book-1{'/sample' if sample else ''}</dc:identifier>
<dc:title>{TITLE}</dc:title><dc:creator>{AUTHOR}</dc:creator><dc:language>en</dc:language>
<dc:rights>Book prose CC BY 4.0; embedded Noto Serif Tamil under SIL OFL 1.1.</dc:rights>
<meta property="dcterms:modified">{modified}</meta></metadata>
<manifest>{manifest}</manifest><spine>{spine}</spine></package>'''
    with zipfile.ZipFile(path, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('mimetype', 'application/epub+zip', compress_type=zipfile.ZIP_STORED)
        archive.writestr('META-INF/container.xml', '''<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles>
<rootfile full-path="EPUB/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>''')
        archive.writestr('EPUB/package.opf', package)
        archive.writestr('EPUB/book.css', css)
        archive.write(ASSETS / 'NotoSerifTamil.ttf', 'EPUB/NotoSerifTamil.ttf')
        archive.write(ASSETS / 'OFL-NotoSerifTamil.txt', 'EPUB/OFL-NotoSerifTamil.txt')
        archive.writestr('EPUB/cover.xhtml', xhtml(TITLE, cover(sample)))
        nav = f'<nav epub:type="toc" role="doc-toc" class="book-contents" title="Book contents"><h1 id="contents-title">Contents</h1>{contents(documents, "epub")}</nav>'
        archive.writestr('EPUB/nav.xhtml', xhtml('Contents', nav))
        for doc in documents:
            archive.writestr(f'EPUB/{doc.stem}.xhtml', xhtml(doc.label, '<main>' + article(doc, documents, 'epub') + '</main>'))


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:
        pass


def repair_outline(pdf: object, documents: list[Document]) -> None:
    """Restore whitespace dropped by Chromium at wrapped headings."""
    titles = [TITLE, 'Contents']
    titles += [''.join(element.itertext()) for doc in documents for element in doc.tree.iter()
               if re.fullmatch(r'h[1-6]', element.tag)]
    canonical = {re.sub(r'\s+', '', title): title for title in titles}
    outline = pdf.get_toc(simple=False)
    for entry in outline:
        entry[1] = canonical[re.sub(r'\s+', '', entry[1])]
    # The poem's source has no visible title, so Chromium omits its bookmark.
    # Resolve its real destination; pagination is never hard-coded.
    if any(doc.stem == 'epigraph' for doc in documents):
        page, x, y = pdf.resolve_link('#nameddest=epigraph')
        if page < 0:
            raise ValueError('PDF epigraph destination is missing')
        position = next(i for i, entry in enumerate(outline) if entry[1] == 'Contents') + 1
        outline.insert(position, [1, 'Epigraph', page + 1,
                                  {'kind': 1, 'page': page, 'to': (x, y), 'zoom': 0.0}])
    pdf.set_toc(outline, collapse=1)


def write_pdf(html_path: Path, pdf_path: Path, executable: str | None,
              documents: list[Document], sample: bool = False) -> None:
    from playwright.sync_api import sync_playwright
    import pymupdf

    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(html_path.parent)))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(executable_path=executable)
            try:
                page = browser.new_page()
                page.goto(f'http://127.0.0.1:{server.server_port}/{quote(html_path.name)}', wait_until='networkidle')
                page.evaluate('document.fonts.ready')
                page.pdf(path=str(pdf_path), prefer_css_page_size=True, print_background=True,
                         tagged=True, outline=True, display_header_footer=True,
                         header_template='<span></span>',
                         footer_template='<div style="width:100%;text-align:center;font-size:9px;color:#555"><span class="pageNumber"></span></div>')
            finally:
                browser.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    temporary = pdf_path.with_suffix('.tmp.pdf')
    with pymupdf.open(pdf_path) as pdf:
        metadata = pdf.metadata
        metadata.update(title=TITLE, author=AUTHOR,
                        subject='Book 1 selected chapters' if sample else 'Book 1 review copy')
        pdf.set_metadata(metadata)
        repair_outline(pdf, documents)
        for name, source in [('OFL-NotoSerifTamil.txt', ASSETS / 'OFL-NotoSerifTamil.txt'),
                             ('LICENSE-CC-BY.txt', BOOK / 'LICENSE-CC-BY')]:
            pdf.embfile_add(name, source.read_bytes(), filename=name)
        pdf.save(temporary)
    temporary.replace(pdf_path)


def main() -> None:
    args_parser = argparse.ArgumentParser(description=__doc__)
    args_parser.add_argument('--output-dir', type=Path, default=ROOT / 'output/book-1')
    args_parser.add_argument('--browser-executable', help='Optional existing Chromium executable')
    args_parser.add_argument('--no-pdf', action='store_true', help='Build HTML and EPUB without launching Chromium')
    args_parser.add_argument('--sample', action='store_true', help='Build the selected publisher sample: chapters 1, 5, 8, 21 and 31')
    args_parser.add_argument('--ui-export', type=Path, help='Export the full reader JSON only, using the existing renderer and link checks')
    args = args_parser.parse_args()
    docs = read_documents()
    if args.ui_export:
        if args.sample:
            args_parser.error('--ui-export always exports the complete book')
        export_ui(args.ui_export, docs)
        print(f'Exported {len(docs)} ordered UI inputs in {args.ui_export}')
        return
    if args.sample:
        docs = select_sample(docs)
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    css = (ASSETS / 'book.css').read_text(encoding='utf-8')
    stem = 'book-1-sample' if args.sample else 'book-1-review'
    html_path = output / f'{stem}.html'
    html_path.write_text(html_document(docs, css, args.sample), encoding='utf-8')
    write_epub(output / f'{stem}.epub', docs, css, args.sample)
    if not args.no_pdf:
        write_pdf(html_path, output / f'{stem}.pdf', args.browser_executable, docs, args.sample)
    print(f'Built {len(docs)} ordered inputs in {output}')


if __name__ == '__main__':
    main()
