# SPDX-License-Identifier: MIT OR Apache-2.0
"""Development checks for the executable-input boundary and source checkpoints."""
import importlib.util
import json
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
        self.assertEqual(len(self.game['scenarios']), 30)
        self.assertEqual(sum(len(f['steps']) for f in self.game['scenarios']), 64)
        self.assertEqual(len(self.game['joints']), 12)
        self.assertEqual(len(self.game['faults']), 21)
        self.assertEqual(len(self.cases), 78)
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


if __name__ == '__main__':
    unittest.main()
