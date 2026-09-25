# SPDX-License-Identifier: MIT OR Apache-2.0
"""Development checks for the executable-input boundary and source checkpoints."""
import importlib.util
import json
import re
from pathlib import Path
import subprocess
import unittest

UI = Path(__file__).resolve().parents[1]
ROOT = UI.parent
spec = importlib.util.spec_from_file_location('prepare', UI/'scripts/prepare.py')
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


class ExecutableInputs(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.public = json.loads((UI/'generated/cases.json').read_text(encoding='utf-8'))
        cls.cases = {c['id']: c for c in cls.public['cases']}
        cls.game = json.loads((UI/'game.json').read_text(encoding='utf-8'))

    def test_complete_catalogue_and_no_answer_fields(self):
        self.assertEqual(len(self.game['scenarios']), 33)
        self.assertEqual(sum(len(f['steps']) for f in self.game['scenarios']), 73)
        self.assertEqual(len(self.game['joints']), 14)
        self.assertEqual(len(self.game['faults']), 21)
        self.assertEqual(len(self.cases), 91)
        def visit(value):
            if isinstance(value, dict):
                self.assertFalse({'outcome', 'outcomes', 'expected', 'verdicts', 'status', 'tally'} & value.keys())
                for v in value.values(): visit(v)
            elif isinstance(value, list):
                for v in value: visit(v)
        visit(self.public)
        visit(self.game)

    def test_every_step_has_full_queries_and_existing_sources(self):
        for c in self.cases.values():
            self.assertTrue(c['queries'])
            self.assertTrue(all(q.endswith('.') for q in c['queries']))
            for source in c['sources']:
                self.assertTrue((ROOT/source['path']).is_file(), source)
            for line in c['record']:
                self.assertFalse(line.startswith((':', '?', '#')), line)

    def test_fixture_isolation_evidence_removal_and_one_entry_child(self):
        self.assertEqual(self.cases['nell:0']['record'], ['born(Nell).'])
        self.assertEqual(len(self.cases['nell:1']['record']), 4)
        self.assertEqual(self.cases['newcomer:1']['record'], [])
        self.assertIn('at(MPNewcomer, RepublicJurisdiction).', self.cases['newcomer:0']['record'])
        for ident in ('emergency:0', 'scarcity:0', 'nell-record:0'):
            self.assertGreater(len(self.cases[ident]['record']), 100)
        self.assertNotIn('public(Pax).', self.cases['shield:2']['record'])

    def test_counterfactuals_compare_identical_records(self):
        source = (ROOT/'book-1/source/constitution.nibli').read_text(encoding='utf-8')
        for joint in self.game['joints']:
            if not joint['measured']:
                self.assertFalse(joint['queries'])
                continue
            a, b = (self.cases[joint['id']+':'+side] for side in ('canonical','modified'))
            self.assertEqual(a['record'], b['record'])
            self.assertEqual(a['queries'], b['queries'])
            self.assertIsNone(a['counterfactual'])
            for edit in self.public['counterfactuals'][b['counterfactual']]:
                self.assertEqual(source.count(edit['before']), 1)
                self.assertNotEqual(edit['before'], edit['after'])

    def test_later_release_and_conflict_premises_are_not_in_earlier_records(self):
        self.assertFalse(any('Chapter29_Hano_Completion' in s for s in self.cases['hano-release:0']['record']))
        self.assertTrue(any('Chapter29_Hano_Completion' in s for s in self.cases['hano-release:1']['record']))
        conflict='observe(ShieldAppointmentsReview, Case_Dara, Appeals, ConflictedShieldReviewerScope).'
        self.assertNotIn(conflict,self.cases['rex:2']['record'])
        self.assertIn(conflict,self.cases['rex:3']['record'])

    def test_existing_trusted_source_preconditions(self):
        # These repository-authored shell preconditions remain development checks.
        commands = set()
        for spec in json.loads((UI/'case-map.json').read_text(encoding='utf-8'))['cases'].values():
            for checkpoint in spec['snapshots']:
                _, end = prepare.statements(checkpoint['source'],checkpoint['through'],checkpoint['occurrence'])
                for line in (ROOT/checkpoint['source']).read_text(encoding='utf-8').splitlines()[:end]:
                    if line.startswith(':require '): commands.add(line.removeprefix(':require '))
        for command in sorted(commands):
            subprocess.run(command, shell=True, cwd=ROOT, check=True)


def chapter_of(path):
    """The chapter number of a `book-1/NN-…` path, or None."""
    match = re.match(r'book-1/(\d\d)-', path)
    return int(match.group(1)) if match else None


class ChapterCases(unittest.TestCase):
    """Every derived chapter points to companion cases that run (item 63)."""
    @classmethod
    def setUpClass(cls):
        cls.cases = {c['id']: c for c in json.loads((UI/'generated/cases.json').read_text(encoding='utf-8'))['cases']}
        cls.game = json.loads((UI/'game.json').read_text(encoding='utf-8'))
        cls.moved = json.loads((UI/'chapter-cases.json').read_text(encoding='utf-8'))['moved']
        manifest = json.loads((ROOT/'book-1/contents.json').read_text(encoding='utf-8'))
        cls.chapters = {c['number']: c for part in manifest['parts'] for c in part['chapters']
                        if c.get('status') == 'landed'}

    def runs(self, owner, primary):
        """Mirror of `chapters_of` in the companion: its own chapter and every
        chapter whose pins its records draw on."""
        ids = [k for k in self.cases if k.split(':')[0] == owner]
        found = {primary}
        for ident in ids:
            for source in self.cases[ident]['sources']:
                if chapter_of(source['path']) is not None:
                    found.add(chapter_of(source['path']))
        return found

    def test_every_derived_chapter_runs_a_case(self):
        covered = set()
        for fork in self.game['scenarios']:
            covered |= self.runs(fork['id'], fork['chapter'])
        for joint in self.game['joints']:
            if joint['measured']:
                covered |= self.runs(joint['id'], joint['chapter'])
        derived = {n for n, c in self.chapters.items() if c.get('role') == 'derived'}
        self.assertTrue(derived)
        self.assertEqual(derived - covered, set(), 'a derived chapter has no runnable companion case')

    def test_every_derived_chapter_points_to_its_own_cases(self):
        for n, chapter in self.chapters.items():
            if chapter.get('role') != 'derived':
                continue
            text = (ROOT/'book-1'/chapter['file']).read_text(encoding='utf-8')
            derived = text.split('\n## Argument:')[0]
            links = re.findall(r'rights-nobody-has-to-earn/cases/#chapter-(\d+)\)', derived)
            self.assertEqual(links, [str(n)], f'chapter {n} must point once to its own cases')

    def test_sabotage_a_chapter_without_a_case_is_found(self):
        game = self.game
        try:
            self.game = {**game, 'scenarios': [f for f in game['scenarios'] if f['id'] != 'week'],
                         'joints': game['joints']}
            with self.assertRaises(AssertionError):
                self.test_every_derived_chapter_runs_a_case()
        finally:
            self.game = game

    def test_authored_citations_name_headings_that_exist(self):
        def headings(n):
            text = (ROOT/'book-1'/self.chapters[n]['file']).read_text(encoding='utf-8')
            return {h.strip().replace('’', "'") for h in re.findall(r'^#{1,3} (.*)$', text, re.M)}
        part_v = headings(29)
        cited = [x['cost']['title'] for x in self.game['scenarios'] + self.game['joints']]
        cited += [f['book'] for f in self.game['faults'] if f['book'].startswith(('Chapter', 'Part V'))]
        for title in cited:
            chapter, _, heading = title.partition(' · ')
            heading = heading.replace('’', "'")
            if chapter == 'Part V':
                self.assertIn(heading, part_v, title)
            else:
                self.assertIn(heading, headings(int(chapter.removeprefix('Chapter '))), title)

    def test_moved_material_links_exist(self):
        self.assertTrue(self.moved)
        for entry in self.moved:
            self.assertIn(entry['chapter'], self.chapters)
            self.assertTrue(entry['links'])
            for link in entry['links']:
                path, _, anchor = link['path'].partition('#')
                self.assertTrue((ROOT/path).exists(), link)
                if anchor:
                    text = (ROOT/path).read_text(encoding='utf-8')
                    slugs = {re.sub(r'[^a-z0-9 -]', '', h.lower()).replace(' ', '-')
                             for h in re.findall(r'^#{1,6} (.*)$', text, re.M)}
                    self.assertIn(anchor, slugs, link)

    def test_deep_links_name_real_cases(self):
        for fork in self.game['scenarios']:
            self.assertRegex(fork['id'], r'^[a-z][a-z-]*$')
        for joint in self.game['joints']:
            self.assertRegex(joint['id'], r'^j-[a-z-]+$')


if __name__ == '__main__':
    unittest.main()
