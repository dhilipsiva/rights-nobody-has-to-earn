# SPDX-License-Identifier: MIT OR Apache-2.0
"""Development checks for the indexes of what would change each choice and who
bears its cost: `python3 -m unittest discover -s tests -p test_reconsider_index.py`."""

import contextlib
import importlib.util
import io
import json
import re
import shutil
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("reconsider_index", ROOT / "tools" / "reconsider_index.py")
tool = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tool)


def replace_words(text: str, old: str, new: str) -> str:
    """Replace a phrase however the chapter wraps it across lines."""
    pattern = r"\s+".join(re.escape(word) for word in old.split())
    changed, count = re.subn(pattern, lambda _: new, text)
    assert count == 1, f"{old!r} occurs {count} times"
    return changed


def run(*argv: str) -> tuple[int, str]:
    """The generator's exit status and what it printed as an error."""
    errors = io.StringIO()
    with contextlib.redirect_stderr(errors), contextlib.redirect_stdout(io.StringIO()):
        status = tool.main(list(argv))
    return status, errors.getvalue()


class ReconsiderIndexTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((ROOT / tool.MANIFEST).read_text(encoding="utf-8"))
        self.chapters = tool.derived_chapters(self.manifest)

    def planted(self, number: int, change) -> Path:
        """A copy of the inputs under a temporary root, with one chapter changed."""
        root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        for relative in (tool.MANIFEST, tool.SOURCE, tool.REFERENCE, tool.OUTPUT):
            (root / relative).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(ROOT / relative, root / relative)
        for chapter in self.chapters:
            text = (ROOT / "book-1" / chapter["file"]).read_text(encoding="utf-8")
            if chapter["number"] == number:
                text = change(text)
            (root / "book-1" / chapter["file"]).write_text(text, encoding="utf-8")
        return root

    def test_check_passes_on_the_committed_outputs(self):
        self.assertEqual(run("--check"), (0, ""))

    def test_check_fails_on_a_chapter_planted_without_a_condition(self):
        def unconditional(text):
            argued = text.index("\n## Argument:")
            head, tail = text[:argued], text[argued:]
            return head + re.sub(r"I would (reconsider|reopen|move toward|add|narrow|redraw)",
                                 "I once weighed", tail)
        root = self.planted(7, unconditional)
        status, error = run("--check", "--root", str(root))
        self.assertEqual(status, 1)
        self.assertIn("Chapter 7's argument states no condition", error)
        # The same copy with the chapter unchanged passes, so the plant is what fails.
        self.assertEqual(run("--check", "--root", str(self.planted(7, lambda text: text)))[0], 0)

    def test_an_unassigned_sentence_or_a_vanished_one_is_found(self):
        def extra(text):
            return replace_words(text, "Either would reverse the comparison this choice rests on.",
                                 "Either would reverse the comparison this choice rests on. "
                                 "I would reconsider if nobody read this index.")
        status, error = run("--check", "--root", str(self.planted(1, extra)))
        self.assertEqual(status, 1)
        self.assertIn("0 assignments for the condition “I would reconsider if nobody read this index.”", error)

        def vanished(text):
            return replace_words(text, "My rule puts the cost on whoever funds provision", "My rule charges the public")
        status, error = run("--check", "--root", str(self.planted(1, vanished)))
        self.assertEqual(status, 1)
        self.assertIn("the cost “My rule puts the cost on whoever funds provision” occurs 0 times", error)

    def test_costs_counted_above_need_the_section_that_counts_them(self):
        def uncounted(text):
            return replace_words(text, "## What the lightness costs", "## What the week shows")
        status, error = run("--check", "--root", str(self.planted(7, uncounted)))
        self.assertEqual(status, 1)
        self.assertIn("Chapter 7's argument cites costs counted above, and no section counts them", error)

    def test_every_derived_chapter_is_listed_and_every_quotation_is_its_chapters(self):
        document = tool.build()
        for key in ("conditions", "costs"):
            listed = {e["chapter"] for group in document[key] for e in group["entries"]}
            self.assertEqual(listed, {c["number"] for c in self.chapters}, key)
            for group in document[key]:
                for entry in group["entries"]:
                    chapter = tool.plain((ROOT / entry["file"]).read_text(encoding="utf-8"))
                    self.assertIn(entry["text"], chapter)

    def test_the_book_lists_chapters_and_the_companion_quotes(self):
        document = tool.build()
        reference = (ROOT / tool.REFERENCE).read_text(encoding="utf-8")
        region = reference[reference.index(tool.BEGIN):reference.index(tool.END)]
        for group in document["conditions"] + document["costs"]:
            self.assertIn(f"**{group['title']}:**", region)
            for entry in group["entries"]:
                self.assertNotIn(entry["text"], region)
        self.assertIn(tool.COMPANION, region)


if __name__ == "__main__":
    unittest.main()
