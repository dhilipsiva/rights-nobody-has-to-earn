# SPDX-License-Identifier: MIT OR Apache-2.0
"""Development regressions for the prose measurement (revision item 34)."""

import json
from pathlib import Path
import tempfile
import unittest

from tools import prose_lint as lint


def figures(text):
    return lint.measure(text)[0]


class MeasurementTests(unittest.TestCase):
    def test_negations_are_counted_per_thousand_words(self):
        f = figures("It is not here and it can’t be found, and no one knows.")
        self.assertEqual(f["words"], 13)
        self.assertEqual(f["negation"], round(1000 * 3 / 13, 1))

    def test_clean_prose_carries_nothing(self):
        f = figures("# A chapter\n\nNell is owed food, and a witness records its receipt.\n")
        for metric in ("negation", "jargon", "disclaimers", "banned", "harness"):
            self.assertEqual(f[metric], 0, metric)
        self.assertEqual(f["names"], 1)

    def test_terms_of_art_include_their_inflections(self):
        f = figures("Readers, windows, leases, carries and derived findings.")
        self.assertEqual(f["jargon"], round(1000 * 5 / 7, 1))

    def test_link_targets_urls_and_comments_are_not_prose(self):
        source = (
            "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n"
            "See [the chapter](18-the-vote-conviction-does-not-take.md) and "
            "[the pins](../book-1/25-voiding.pins.nibli) at https://example.org/none.\n"
        )
        f = figures(source)
        self.assertEqual((f["negation"], f["banned"]), (0, 1))  # the link text "pins" stays prose

    def test_code_quotations_and_run_it_lines_are_blanked_in_place(self):
        source = "\n".join([
            "# Title",
            "```",
            "a pin, a counterfactual",
            "no stratification here",
            "```",
            "> a quoted void",
            "Run it: the pins for this chapter.",
            "The model is not the book.",
        ])
        _, _, locations = lint.measure(source)
        self.assertEqual(locations, [(8, "banned term", "The model")])

    def test_names_count_cast_fixtures_and_harness_once_each(self):
        f, names, locations = lint.measure("Nell met Foundry and Targ4; Targ4 left.")
        self.assertEqual(names, ["Foundry", "Nell", "Targ4"])
        self.assertEqual(f["harness"], 2)
        self.assertEqual([t for _, k, t in locations if k == "harness name"], ["Targ4", "Targ4"])

    def test_a_name_outside_its_home_chapter_is_a_stray(self):
        text = "Nell and Koa met."
        self.assertEqual(lint.measure(text, chapter="03")[0]["strays"], 1)
        self.assertEqual(lint.measure(text, chapter="16")[0]["strays"], 0)
        self.assertEqual(lint.measure(text)[0]["strays"], 0)
        self.assertEqual(lint.measure("Tove and Iris met.", chapter="03")[0]["strays"], 0)

    def test_disclaimer_sentences(self):
        f = figures("This does not establish delivery. It proves no arrival. Nell is owed food.")
        self.assertEqual(f["disclaimers"], 2)


class RatchetTests(unittest.TestCase):
    def test_an_input_is_held_to_its_record_or_the_threshold(self):
        recorded = {"negation": 40.0, "jargon": 3.0, "names": 9, "disclaimers": 1,
                    "banned": 2, "harness": 0, "strays": 0}
        baseline = {"book-1/x.md": recorded}
        worse = dict(recorded, negation=40.1, jargon=4.9)
        self.assertEqual(lint.regressions("book-1/x.md", "chapter", dict(worse, words=1), baseline),
                         ["negation 40.1 exceeds 40.0"])
        better = dict(recorded, negation=18.0)
        self.assertEqual(lint.regressions("book-1/x.md", "chapter", dict(better, words=1), baseline), [])

    def test_an_input_without_a_record_meets_the_plan_thresholds(self):
        f = dict(words=1, negation=21.0, jargon=5.0, names=6, disclaimers=2, banned=1, harness=0,
                 strays=1)
        self.assertEqual(lint.regressions("book-1/new.md", "chapter", f, {}),
                         ["negation 21.0 exceeds 20.0", "names 6 exceeds 5", "banned 1 exceeds 0",
                          "strays 1 exceeds 0"])

    def test_the_method_is_measured_but_not_limited(self):
        f = dict(words=1, negation=90.0, jargon=90.0, names=40, disclaimers=20, banned=30, harness=5,
                 strays=3)
        self.assertEqual(lint.regressions("book-1/method.md", "method", f, {}), [])

    def test_ratchet_only_lowers_and_admits_explicitly(self):
        old = {"book-1/a.md": {m: 10 for m in lint.METRICS}, "book-1/gone.md": {m: 1 for m in lint.METRICS}}
        measured = [
            ("book-1/a.md", "chapter", {m: (12 if m == "negation" else 8) for m in lint.METRICS}),
            ("book-1/new.md", "chapter", {m: 3 for m in lint.METRICS}),
            ("book-1/other.md", "chapter", {m: 3 for m in lint.METRICS}),
        ]
        new = lint.ratchet(old, measured, admit={"book-1/new.md"})
        self.assertEqual(new["book-1/a.md"]["negation"], 10)
        self.assertEqual(new["book-1/a.md"]["jargon"], 8)
        self.assertIn("book-1/new.md", new)
        self.assertNotIn("book-1/other.md", new)
        self.assertNotIn("book-1/gone.md", new)

    def test_baseline_round_trips(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "baseline.json"
            lint.save_baseline({"b": {"negation": 1.0}, "a": {"negation": 2.0}}, path)
            self.assertEqual(list(json.loads(path.read_text(encoding="utf-8"))), ["a", "b"])
            self.assertEqual(lint.load_baseline(path)["b"]["negation"], 1.0)


class ManuscriptTests(unittest.TestCase):
    def test_every_measured_input_has_a_record_and_nothing_else_does(self):
        inputs = {path for path, _ in lint.ordered_inputs()}
        self.assertEqual(set(lint.load_baseline()), inputs)

    def test_no_ordered_input_regresses(self):
        baseline = lint.load_baseline()
        failures = []
        for path, profile in lint.ordered_inputs():
            f = lint.measure((lint.ROOT / path).read_text(encoding="utf-8"))[0]
            failures += [f"{path}: {p}" for p in lint.regressions(path, profile, f, baseline)]
        self.assertEqual(failures, [], "\n".join(failures))


if __name__ == "__main__":
    unittest.main()
