#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Package executable inputs; query expectations are development data only."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UI = ROOT / 'ui'


def statements(path, through=None, occurrence=1):
    """Snapshot admitted premises, excluding refused and retracted controls."""
    record, skip, seen = [], False, 0
    for number, raw in enumerate((ROOT / path).read_text(encoding='utf-8').splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith('#'):
            continue
        if line.startswith('? '):
            if line[2:] == through:
                seen += 1
                if seen == occurrence:
                    return record, number
            continue
        if line.startswith(':'):
            directive = line.split()[0]
            if directive in (':refuse', ':accept-scoped'):
                skip = True
            elif directive not in (':expect-pins', ':accept', ':require'):
                raise ValueError(f'Unsupported snapshot directive: {path}:{number}: {line}')
            continue
        if skip:
            skip = False
        else:
            if not line.endswith('.'):
                raise ValueError(f'Incomplete statement: {path}:{number}')
            record.append(line)
    if through:
        raise ValueError(f'Missing checkpoint {path}: {through} occurrence {occurrence}')
    return record, number


def main():
    suites = json.loads((ROOT / 'tests/pins/suites.json').read_text(encoding='utf-8'))
    game = json.loads((UI / 'game.json').read_text(encoding='utf-8'))
    mapping = json.loads((UI / 'case-map.json').read_text(encoding='utf-8'))['cases']
    source = (ROOT / 'book-1/source/constitution.nibli').read_text(encoding='utf-8')
    fields = dict(line.split() for line in (ROOT / 'engine.pin').read_text(encoding='utf-8').splitlines()
                  if line and not line.startswith('#'))
    queries = {step['id']: step['queries'] for fork in game['scenarios'] for step in fork['steps']}
    for joint in game['joints']:
        if joint['measured']:
            for side in ('canonical', 'modified'):
                queries[joint['id'] + ':' + side] = joint['queries']
    assert queries.keys() == mapping.keys()
    cases, edits = [], {}
    for ident, spec in mapping.items():
        record, references = [], [{'path': p} for p in spec['references']]
        for snapshot in spec['snapshots']:
            path = snapshot['source']
            contexts = [c for c in suites['cases'] if c['pins'] == [path]]
            if len(contexts) != 1 or contexts[0].get('edits'):
                raise ValueError(f'Ambiguous or edited snapshot context: {path}')
            for fixture in contexts[0]['fixtures']:
                facts, end = statements(fixture)
                record.extend(facts)
                references.append({'path': fixture, 'through_line': end})
            facts, end = statements(path, snapshot['through'], snapshot['occurrence'])
            record.extend(facts)
            references.append({'path': path, 'through_line': end, 'query': snapshot['through'],
                               'occurrence': snapshot['occurrence']})
        record.extend(spec['add'])
        cf = spec.get('counterfactual')
        if cf:
            edit = suites['bases'][cf]
            assert edit['base'] == 'live'
            for change in edit['edits']:
                if source.count(change['before']) != 1:
                    raise ValueError(f'Ambiguous counterfactual: {cf}')
            edits[cf] = edit['edits']
        person = next((s['person'] for s in game['scenarios'] if ident.startswith(s['id'] + ':')), None)
        person = 'MPNewcomer' if person == 'Newcomer' else person
        qs = [q['text'] for q in queries[ident]]
        if person:
            for item, predicate in [('Eats','eats'),('Dwell','dwell'),('Healthy','healthy'),('Learn','learn'),
                                    ('Secure','secure'),('Expresses','expresses'),('Believe','believe'),('Meets','meets')]:
                qs += [f'owe(State, {item}, {person}).', f'{predicate}({person}).']
        cases.append({'id': ident, 'record': list(dict.fromkeys(record)),
                      'queries': list(dict.fromkeys(qs)), 'counterfactual': cf,
                      'sources': references, 'renames': spec.get('renames', {})})
    public = {'version': 2, 'license': 'CC0-1.0', 'engine_revision': fields['revision'],
              'constitution_source': 'book-1/source/constitution.nibli',
              'cases': cases, 'counterfactuals': edits}
    out = UI / 'generated'
    out.mkdir(exist_ok=True)
    (out / 'cases.json').write_text(json.dumps(public, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    (out / 'reason-inputs.json').write_text(json.dumps({**public, 'constitution': source}, ensure_ascii=False), encoding='utf-8')
    print(f'Prepared complete constitution and {len(cases)} input-only records')


if __name__ == '__main__':
    main()
