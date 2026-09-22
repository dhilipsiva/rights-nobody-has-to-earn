#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assemble reasoning resources from the current source and declared counterfactual."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
UI = ROOT / 'ui'

def main():
    suites = json.loads((ROOT / 'tests/pins/suites.json').read_text(encoding='utf-8'))
    bases = suites.get('bases', suites.get('contexts', {}))
    edit = bases['counterfactual/no-delivery-independence']['edits'][0]
    source = (ROOT / 'book-1/source/constitution.nibli').read_text(encoding='utf-8')
    if source.count(edit['before']) != 1:
        raise ValueError('Declared counterfactual must match exactly once')
    fields = dict(line.split() for line in (ROOT / 'engine.pin').read_text(encoding='utf-8').splitlines() if line and not line.startswith('#'))
    data = {'constitution': source, 'edit': edit, 'engine_revision': fields['revision'],
            'cases': json.loads((UI / 'scenarios.json').read_text(encoding='utf-8'))}
    out = UI / 'generated'
    out.mkdir(exist_ok=True)
    (out / 'reason-inputs.json').write_text(json.dumps(data, ensure_ascii=False), encoding='utf-8')
    print(f'Prepared full constitution ({len(source.encode()):,} bytes) and {len(data["cases"])} independent scenarios')

if __name__ == '__main__':
    main()
