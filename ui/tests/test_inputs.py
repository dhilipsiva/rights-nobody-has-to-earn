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



def pinned_verdicts(path):
    """Every query a pin file asks, with the verdict it requires; a refused
    statement maps to REFUSED. A query pinned two ways maps to both."""
    lines = path.read_text(encoding='utf-8').splitlines()
    found, refusing = {}, False
    for index, line in enumerate(lines):
        text = line.strip()
        if text.startswith(':refuse'):
            refusing = True
            continue
        if refusing and text and not text.startswith(('#', ':')):
            found.setdefault(text, set()).add('REFUSED')
            refusing = False
            continue
        if text.startswith('? '):
            for later in lines[index + 1:index + 3]:
                match = re.match(r'#\s*=>\s*(\w+)', later.strip())
                if match:
                    found.setdefault(text[2:].strip(), set()).add(match.group(1))
                    break
    return found


def guided_mismatches(steps, pins):
    """Steps that state a conclusion their pin file does not return."""
    return [step['query'] for step in steps if pins.get(step['query']) != {step['verdict']}]


class FrontDoor(unittest.TestCase):
    """The companion's front door (item 83): a short panel, a guided first run
    held to Chapter 1's pins, an explainer, a themed dossier and a book map."""
    @classmethod
    def setUpClass(cls):
        cls.start = json.loads((UI/'start.json').read_text(encoding='utf-8'))
        cls.game = json.loads((UI/'game.json').read_text(encoding='utf-8'))
        cls.pins = pinned_verdicts(ROOT/cls.start['guided']['pins'])

    def test_the_panel_is_short_and_says_where_to_begin(self):
        panel = ' '.join(self.start['panel'])
        self.assertLessEqual(len(panel.split()), 200)
        for word in ('fork', 'joint'):
            self.assertIn(word, panel)
        for lens in self.game['lenses']:
            self.assertIn(lens['label'], panel)
        fork = next(f for f in self.game['scenarios'] if f['id'] == self.start['guided']['fork'])
        self.assertIn(fork['title'].rstrip('.'), panel)

    def test_the_guided_run_says_only_what_the_pins_return(self):
        steps = self.start['guided']['steps']
        self.assertGreaterEqual(len(steps), 5)
        self.assertEqual(guided_mismatches(steps, self.pins), [])
        self.assertIn('FALSE', {s['verdict'] for s in steps}, 'the run must show what does not follow')

    def test_sabotage_a_step_claiming_a_delivery_is_found(self):
        planted = [dict(s) for s in self.start['guided']['steps']]
        for step in planted:
            if step['query'] == 'eats(Nell).':
                step['verdict'] = 'TRUE'
        self.assertEqual(guided_mismatches(planted, self.pins), ['eats(Nell).'])

    def test_the_explainer_names_a_real_fork_and_a_measured_joint(self):
        explainer = self.start['explainer']
        self.assertIn(explainer['fork'], {f['id'] for f in self.game['scenarios']})
        joint = next(j for j in self.game['joints'] if j['id'] == explainer['joint'])
        self.assertTrue(joint['measured'])
        self.assertIn(joint['title'], ' '.join(explainer['text']))

    def test_every_dossier_entry_has_a_theme(self):
        themes = {f['theme'] for f in self.game['faults']}
        self.assertEqual(themes, {'limits', 'costs', 'objections'})

    def test_every_chapter_and_opening_case_has_a_map_summary(self):
        import sys
        sys.path.insert(0, str(ROOT/'tools'))
        import annotated_contents
        summaries = annotated_contents.summaries()
        manifest = json.loads((ROOT/'book-1/contents.json').read_text(encoding='utf-8'))
        for part in manifest['parts']:
            if part.get('opener', {}).get('status') == 'landed':
                self.assertIn(part['opener']['file'], summaries)
            for chapter in part['chapters']:
                if chapter.get('status') == 'landed':
                    self.assertTrue(summaries.get(chapter['file']), chapter['file'])



# Predicates whose argument, at this position, is the person a query is about.
PERSON_PLACES = {'person': 0, 'prisoner': 0, 'travel': 0, 'decide': 0, 'defend': 0,
                 'false': 0, 'eats': 0, 'dwell': 0, 'healthy': 0, 'expresses': 0,
                 'owe': 2, 'entitled': 0}
CLAIMS = re.compile(r'\b(prove[sd]?|proof|shows?|establish(es|ed)?|TRUE|FALSE|REFUSED)\b')


def game_queries(game):
    for fork in game['scenarios']:
        for step in fork['steps']:
            yield from step['queries']
    for joint in game['joints']:
        yield from joint['queries']


def arguments(text):
    match = re.match(r'\s*([A-Za-z_]+)\((.*)\)\.\s*$', text)
    return (match[1], [a.strip() for a in match[2].split(',')]) if match else (text, [])


def gloss_problems(game):
    """Each query's gloss says in one sentence what it asks, names the person
    it asks about, claims no result, and reads the same wherever it recurs."""
    queries = list(game_queries(game))
    names = set()
    for query in queries:
        predicate, args = arguments(query['text'])
        place = PERSON_PLACES.get(predicate)
        if place is not None and place < len(args) and re.fullmatch(r'[A-Z][a-z]+', args[place]):
            names.add(args[place])
    problems, seen = [], {}
    for query in queries:
        text, gloss = query['text'], query.get('gloss', '')
        if not gloss:
            problems.append(f'{text}: no gloss')
            continue
        if not gloss.startswith('Asks whether ') or not gloss.endswith('.') or '. ' in gloss:
            problems.append(f'{text}: not one sentence saying what it asks')
        if CLAIMS.search(gloss):
            problems.append(f'{text}: the gloss claims a result')
        for name in sorted(set(arguments(text)[1]) & names):
            if not re.search(rf'\b{name}\b', gloss):
                problems.append(f'{text}: the gloss does not name {name}')
        if seen.setdefault(text, gloss) != gloss:
            problems.append(f'{text}: glossed two ways')
    return problems


class QueryGlosses(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.game = json.loads((UI/'game.json').read_text(encoding='utf-8'))

    def test_every_query_says_what_it_asks(self):
        self.assertEqual(gloss_problems(self.game), [])
        self.assertEqual(len(list(game_queries(self.game))), 249)

    def test_sabotage_a_missing_wrong_or_claiming_gloss_is_found(self):
        def first(game, text):
            return next(q for q in game_queries(game) if q['text'] == text)
        game = json.loads(json.dumps(self.game))
        del first(game, 'born(Nell).')['gloss']
        self.assertEqual(gloss_problems(game), ['born(Nell).: no gloss'])
        game = json.loads(json.dumps(self.game))
        claiming = [q for q in game_queries(game) if q['text'] == 'eats(Marisol).']
        for query in claiming:
            query['gloss'] = 'Asks whether the rules prove that Marisol ate.'
        self.assertEqual(gloss_problems(game), ['eats(Marisol).: the gloss claims a result'] * len(claiming))
        game = json.loads(json.dumps(self.game))
        query = first(game, 'owe(State, Eats, Zed).')
        query['gloss'] = query['gloss'].replace('Zed', 'Nell')
        self.assertIn('owe(State, Eats, Zed).: the gloss does not name Zed', gloss_problems(game))



def assurance_tool():
    import sys
    sys.path.insert(0, str(ROOT/'tools'))
    import assurance_pages
    return assurance_pages


class AssurancePages(unittest.TestCase):
    """The pages on what was checked and what is left open are generated from
    the records that carry each, and must follow them."""

    @classmethod
    def setUpClass(cls):
        cls.tool = assurance_tool()
        cls.committed = (UI/'assurance.json').read_text(encoding='utf-8')

    def test_the_pages_match_their_sources(self):
        self.assertEqual(self.tool.render(self.tool.build()), self.committed,
                         'ui/assurance.json is stale; run python3 tools/assurance_pages.py')

    def test_sabotage_a_changed_source_is_found(self):
        import copy
        loaded = self.tool.sources()
        changed = copy.deepcopy(loaded)
        case = next(iter(changed['results']['cases'].values()))
        case['differences'] = [{'query': 'person(Nell).', 'expected': 'TRUE', 'clingo': 'FALSE'}]
        self.assertNotEqual(self.tool.render(self.tool.build(changed)), self.committed)
        changed = copy.deepcopy(loaded)
        changed['audit']['lenses'][0]['findings'][0]['open'] = True
        changed['audit']['lenses'][0]['findings'][0].setdefault('disposition', 'route-unbuilt')
        changed['audit']['lenses'][0]['findings'][0].setdefault('consequence', 'A planted claim.')
        self.assertNotEqual(self.tool.render(self.tool.build(changed)), self.committed)

    def test_the_second_engine_page_carries_the_report_and_the_methods_limit(self):
        engine = json.loads(self.committed)['engine']
        method = ' '.join((ROOT/'book-1/method.md').read_text(encoding='utf-8').split())
        self.assertIn(engine['limit'], method)
        self.assertIn('misreading shared by both engines', engine['limit'])
        self.assertTrue(engine['keeps'] and engine['leaves_out'])
        self.assertEqual(engine['queries'], sum(c['queries'] for c in engine['cases']))

    def test_every_chapter_with_limits_is_quoted_without_navigation(self):
        limits = json.loads(self.committed)['limits']
        manifest = json.loads((ROOT/'book-1/contents.json').read_text(encoding='utf-8'))
        stated = [c['number'] for part in manifest['parts'] for c in part['chapters']
                  if c.get('role') == 'derived' and c.get('status') == 'landed'
                  and '## What this cannot settle\n' in (ROOT/'book-1'/c['file']).read_text(encoding='utf-8')]
        self.assertEqual([c['number'] for c in limits['chapters']], stated)
        for chapter in limits['chapters']:
            for paragraph in chapter['paragraphs']:
                self.assertFalse(paragraph.startswith('Run it:'), chapter['number'])
                self.assertNotIn('The next chapter', paragraph, chapter['number'])

    def test_declared_defects_are_found_and_none_is_active(self):
        import tempfile
        self.assertEqual(json.loads(self.committed)['limits']['defects'], [])
        with tempfile.TemporaryDirectory() as tmp:
            planted = Path(tmp)/'book-1'/'planted.pins.nibli'
            planted.parent.mkdir(parents=True)
            planted.write_text(':defect "a planted defect"\n? person(Nell).\n# => FALSE\n', encoding='utf-8')
            found = self.tool.declared_defects(Path(tmp))
        self.assertEqual(found, [{'file': 'book-1/planted.pins.nibli', 'line': 1, 'reason': 'a planted defect'}])

    def test_a_results_file_with_a_repeated_case_is_refused(self):
        with self.assertRaises(ValueError):
            json.loads('{"cases": {"a": {}, "a": {}}}', object_pairs_hook=self.tool.unique_keys)


if __name__ == '__main__':
    unittest.main()
