# SPDX-License-Identifier: MIT OR Apache-2.0
"""Development regressions for the explicit book assembler."""

from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET
import zipfile

from tools import build_book as book


class MarkdownRenderingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.root_patch = patch.object(book, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)

    def doc(self, name, source):
        path = self.root / name
        path.write_text(source, encoding="utf-8")
        return book.document(path)

    def test_list_continuations_and_verse_breaks_survive(self):
        doc = self.doc("chapter.md", "# A chapter\n\n- First line\n  continued here.\n\nVerse one\\\nVerse two\n")
        item = doc.tree.find("ul/li")
        self.assertIn("continued here.", "".join(item.itertext()))
        self.assertEqual(len(doc.tree.findall("p/br")), 1)

    def test_footnotes_preserve_balanced_urls_and_distinct_return_targets(self):
        source = "# A chapter\n\nA claim.[^a] And again.[^a]\n\n[^a]: [Study](https://example.org/study_(2020)).\n"
        first = self.doc("first.md", source)
        second = self.doc("second.md", source)
        combined = ET.fromstring("<div>" + "".join(book.article(d, [first, second], "html") for d in [first, second]) + "</div>")
        ids = [node.get("id") for node in combined.iter() if node.get("id")]
        self.assertEqual(len(ids), len(set(ids)))
        for link in combined.iter("a"):
            href = link.get("href")
            if href.startswith("#"):
                self.assertIn(href[1:], ids)
            else:
                self.assertEqual(href, "https://example.org/study_(2020)")
        self.assertEqual(len(combined.findall('.//a[@role="doc-backlink"]')), 4)

    def test_chapter_fragments_and_repository_files_resolve_differently(self):
        first = self.doc("first.md", "# First\n")
        second = self.doc("second.md", "# Second\n\n## A person's claim\n\n## A person's claim\n")
        (self.root / "rules.nibli").write_text("Person is person.\n")
        self.assertEqual(book.resolve_link("second.md#a-persons-claim-1", first, [first, second], "epub"),
                         "second.xhtml#second-a-persons-claim-1")
        self.assertEqual(book.resolve_link("rules.nibli", first, [first, second], "html"),
                         book.REPOSITORY + "rules.nibli")
        for href in ["missing.md", "second.md#missing", "../outside.md"]:
            with self.subTest(href=href), self.assertRaises(ValueError):
                book.resolve_link(href, first, [first, second], "html")

    def test_epigraph_keeps_tamil_language_and_unheaded_source(self):
        doc = self.doc("epigraph.md", "யாதும் ஊரே\\\nயாவரும் கேளிர்\n\nEvery town.\n")
        self.assertEqual(doc.tree.find("p").get("lang"), "ta")
        self.assertEqual(doc.tree.find("h1").get("class"), "sr-only")
        self.assertIsNone(doc.number)

    def test_unsupported_markup_is_not_silently_published(self):
        (self.root / "diagrams").mkdir()
        (self.root / "diagrams" / "flow.svg").write_text("<svg xmlns='http://www.w3.org/2000/svg'/>")
        for source in ["# Title\n\n<div>Unexpected HTML</div>\n", "# Title\n\n![Unbundled](image.png)\n",
                       "# Title\n\n![](diagrams/flow.svg)\n", "# Title\n\n![A flow](diagrams/missing.svg)\n"]:
            with self.subTest(source=source), self.assertRaises(ValueError):
                self.doc("chapter.md", source)

    def test_a_diagram_is_inlined_for_print_and_packaged_for_the_epub(self):
        (self.root / "diagrams").mkdir()
        (self.root / "diagrams" / "flow.svg").write_text("<svg xmlns='http://www.w3.org/2000/svg'/>")
        doc = self.doc("chapter.md", "# Title\n\n![A flow of facts](diagrams/flow.svg)\n")
        self.assertEqual(book.diagrams(doc), ["diagrams/flow.svg"])
        html_copy = ET.fromstring(book.article(doc, [doc], "html"))
        image = html_copy.find(".//img")
        self.assertTrue(image.get("src").startswith("data:image/svg+xml;base64,"))
        self.assertEqual(image.get("alt"), "A flow of facts")
        epub_copy = ET.fromstring(book.article(doc, [doc], "epub"))
        self.assertEqual(epub_copy.find(".//img").get("src"), "diagrams/flow.svg")
        self.assertIn(f"]({book.RAW}book-1/diagrams/flow.svg)", book.ui_markdown(doc, [doc]))

    def test_a_repository_link_prints_its_path_only_in_the_print_copy(self):
        (self.root / "rules.nibli").write_text("Person is person.\n")
        doc = self.doc("chapter.md", "# Title\n\nSee [the rules](rules.nibli) here.\n")
        printed = ET.fromstring(book.article(doc, [doc], "html"))
        note = printed.find(".//span[@class='print-only']")
        self.assertEqual(note.text, " (repository: rules.nibli)")
        self.assertEqual(note.tail, " here.")
        for mode in ("epub", "ui"):
            self.assertIsNone(ET.fromstring(book.article(doc, [doc], mode)).find(".//span[@class='print-only']"))


class CurrentEditionTests(unittest.TestCase):
    def test_epub_reading_order_navigation_and_all_local_destinations(self):
        self.check_epub(book.read_documents(), sample=False)

    def test_sample_keeps_chapter_numbers_and_links_outside_the_selection(self):
        docs = book.select_sample(book.read_documents())
        self.assertEqual([d.number for d in docs], [1, 4, 8, 21, 29])
        self.assertEqual(book.resolve_link("02-what-the-record-may-say.md#the-child-with-nobody", docs[0], docs, "html"),
                         book.REPOSITORY + "book-1/02-what-the-record-may-say.md#the-child-with-nobody")
        with self.assertRaises(ValueError):
            book.select_sample(docs[:-1])
        self.check_epub(docs, sample=True)

    def check_epub(self, docs, sample):
        css = (book.ASSETS / "book.css").read_text()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "book.epub"
            book.write_epub(path, docs, css, sample)
            with zipfile.ZipFile(path) as archive:
                self.assertEqual(archive.infolist()[0].filename, "mimetype")
                self.assertEqual(archive.infolist()[0].compress_type, zipfile.ZIP_STORED)
                package = ET.fromstring(archive.read("EPUB/package.opf"))
                ns = {"p": "http://www.idpf.org/2007/opf", "h": "http://www.w3.org/1999/xhtml"}
                identifier = package.find("p:metadata/{http://purl.org/dc/elements/1.1/}identifier", ns).text
                self.assertEqual(identifier.endswith("/sample"), sample)
                spine = [node.get("idref") for node in package.findall("p:spine/p:itemref", ns)]
                self.assertEqual(spine, ["cover"] + [f"doc-{d.stem}" for d in docs] + ["back-cover"])
                titles = [node.text for node in package.iter("{http://purl.org/dc/elements/1.1/}title")]
                self.assertEqual(titles, [book.TITLE, book.SUBTITLE[:-1]])
                self.assertEqual(package.find(".//{http://purl.org/dc/elements/1.1/}description").text, book.PROMISE)
                drawn = {d for doc in docs for d in book.diagrams(doc)}
                images = {node.get("href") for node in package.findall("p:manifest/p:item", ns)
                          if node.get("media-type") == "image/svg+xml"}
                self.assertEqual(images, drawn)
                for image in images:
                    self.assertIn("EPUB/" + image, archive.namelist())
                for node in package.findall("p:manifest/p:item", ns):
                    self.assertRegex(node.get("id"), r"^[A-Za-z_][A-Za-z0-9_.-]*$")
                trees = {name: ET.fromstring(archive.read(name)) for name in archive.namelist() if name.endswith(".xhtml")}
                nav = trees["EPUB/nav.xhtml"]
                links = nav.findall(".//h:a", ns)
                self.assertEqual([node.text for node in links], [d.label for d in docs])
                for item in nav.findall(".//h:li", ns):
                    self.assertIn(item[0].tag, {"{" + ns["h"] + "}a", "{" + ns["h"] + "}span"})
                    if item[0].tag.endswith("span"):
                        self.assertEqual(item[1].tag, "{" + ns["h"] + "}ol")
                for name, tree in trees.items():
                    for node in tree.findall(".//h:a", ns):
                        href = node.get("href")
                        if ":" in href:
                            continue
                        file, _, fragment = href.partition("#")
                        destination = "EPUB/" + file if file else name
                        self.assertIn(destination, trees, (name, href))
                        ids = {e.get("id") for e in trees[destination].iter()}
                        self.assertIn(fragment, ids, (name, href))


class PrintEditionTests(unittest.TestCase):
    """What the review copy prints: its subtitle, its promise, and no web artefacts."""

    def test_cover_back_cover_and_metadata_carry_the_subtitle_and_promise(self):
        docs = book.read_documents()
        css = (book.ASSETS / "book.css").read_text()
        page = book.html_document(docs, css)
        cover = page[page.index('<header class="cover">'):page.index("</header>")]
        self.assertIn(book.SUBTITLE, cover)
        back = page[page.index('<section class="back-cover"'):]
        self.assertIn(book.PROMISE, back)
        self.assertIn(book.PROMISE, (book.BOOK / "00-opening-note.md").read_text(encoding="utf-8").replace("\n", " "))

    def test_print_carries_no_web_artefact(self):
        docs = book.read_documents()
        css = (book.ASSETS / "book.css").read_text()
        print_rules = css[css.index("@media print"):]
        self.assertRegex(print_rules, r"\.footnote-backref \{ display: none; \}")
        self.assertRegex(print_rules, r"\.print-only \{ display: inline; \}")
        page = book.html_document(docs, css)
        for phrase in ("Continue to Chapter 1", "return to the contents", "click", "Click"):
            self.assertNotIn(phrase, page, phrase)
        images = list(ET.fromstring(f"<div>{''.join(book.article(d, docs, 'html') for d in docs)}</div>").iter("img"))
        self.assertTrue(images)
        for image in images:
            self.assertTrue(image.get("alt", "").strip())


class BackMatterTests(unittest.TestCase):
    """The generated back matter follows the text it is drawn from."""

    def test_works_cited_and_index_are_current(self):
        import importlib.util
        for name, check in (("bibliography", lambda m: m.render() == m.OUTPUT.read_text(encoding="utf-8")),
                            ("book_index", lambda m: m.replace(m.TARGET.read_text(encoding="utf-8"))
                             == m.TARGET.read_text(encoding="utf-8"))):
            spec = importlib.util.spec_from_file_location(name, book.ROOT / "tools" / f"{name}.py")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            self.assertTrue(check(module), f"run tools/{name}.py")

    def test_every_note_citing_a_work_reaches_the_works_cited(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location("bibliography", book.ROOT / "tools" / "bibliography.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        listed = {module.identity(line[2:].rsplit(" Cited in ", 1)[0])
                  for line in module.OUTPUT.read_text(encoding="utf-8").splitlines() if line.startswith("- ")}
        cited = {module.identity(work) for name, _ in module.inputs()
                 for note in module.notes((book.BOOK / name).read_text(encoding="utf-8"))
                 if not module.continues(note) for work in module.note_works(note)}
        self.assertTrue(cited)
        self.assertEqual(cited - listed, set())


class SummaryEditionTests(unittest.TestCase):
    def test_summary_edition_carries_every_article_in_order(self):
        import json
        articles = json.loads((book.ROOT / "ui" / "articles.json").read_text(encoding="utf-8"))["articles"]
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "constitution-in-plain-language.md"
            source.write_text(book.summary_markdown(), encoding="utf-8")
            doc = book.document(source)
            numbers = [int(re.match(r"Article (\d+)\.", "".join(h.itertext())).group(1))
                       for h in doc.tree.iter("h3")]
            self.assertEqual(numbers, [a["number"] for a in articles])
            text = "".join(doc.tree.itertext())
            self.assertIn(book.PROMISE, text)
            for article in articles:
                for clause in article["text"]:
                    self.assertIn(clause, text)
            css = (book.ASSETS / "book.css").read_text()
            path = Path(directory) / "summary.epub"
            book.write_epub(path, [doc], css, "summary")
            with zipfile.ZipFile(path) as archive:
                package = ET.fromstring(archive.read("EPUB/package.opf"))
                ns = {"p": "http://www.idpf.org/2007/opf"}
                identifier = package.find("p:metadata/{http://purl.org/dc/elements/1.1/}identifier", ns).text
                self.assertTrue(identifier.endswith("/summary"))
                spine = [node.get("idref") for node in package.findall("p:spine/p:itemref", ns)]
                self.assertEqual(spine, ["cover", f"doc-{doc.stem}", "back-cover"])
            self.assertIn("plain language", book.cover("summary"))


class UiExportTests(unittest.TestCase):
    def test_all_inputs_share_renderer_order_notes_and_citation_links(self):
        import json
        docs = book.read_documents()
        with tempfile.TemporaryDirectory() as directory:
            book.export_ui(Path(directory), docs)
            result = json.loads((Path(directory) / 'book.json').read_text())
        manifest = json.loads((book.BOOK / 'contents.json').read_text(encoding='utf-8'))
        inputs = len(manifest['front']) + len(manifest['back']) + sum(
            (part.get('opener', {}).get('status') == 'landed')
            + sum(c['status'] == 'landed' for c in part['chapters'])
            for part in manifest['parts'])
        self.assertEqual(len(result['pages']), inputs)
        self.assertEqual([p['stem'] for p in result['pages']], [d.stem for d in docs])
        ids = {p['path']: {e.get('id') for e in ET.fromstring(p['html']).iter() if e.get('id')}
               for p in result['pages']}
        for row, doc in zip(result['pages'], docs):
            self.assertNotIn('<!--', row['markdown'])
            self.assertEqual(row['html'], book.article(doc, docs, 'ui'))
            self.assertIn(''.join(doc.tree.find('h1').itertext()), row['text'])
            for node in ET.fromstring(row['html']).iter('a'):
                href = node.get('href', '')
                if href.startswith(book.UI_PREFIX):
                    path, _, anchor = href.partition('#')
                    self.assertIn(path, ids)
                    self.assertIn(anchor, ids[path])
        self.assertIn('lang="ta"', result['pages'][0]['html'])
        part_v = next(p for p in result['pages'] if p['number'] == 29)
        self.assertIn('footnote-backref', part_v['html'])

    def test_ui_markdown_keeps_balanced_external_urls_and_note_definitions(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(book, 'ROOT', Path(directory)):
            first = Path(directory) / 'one.md'
            first.write_text('# One\n\n<!-- editorial -->\n\n[Next](two.md#second)\n\nA claim.[^n]\n\n[^n]: [Study](https://example.org/study_(2020)).\n')
            second = Path(directory) / 'two.md'
            second.write_text('# Two\n\n## Second\n')
            docs = [book.document(first), book.document(second)]
            md = book.ui_markdown(docs[0], docs)
            self.assertNotIn('editorial', md)
            self.assertIn('https://dhilipsiva.dev/rights-nobody-has-to-earn/read/two/#two-second', md)
            self.assertIn('[^n]: [Study](https://example.org/study_(2020)).', md)


if __name__ == "__main__":
    unittest.main()
