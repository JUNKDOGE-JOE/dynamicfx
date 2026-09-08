#!/usr/bin/env python3
"""Check retained UI readbacks offline; this script never contacts AE.

Actions/keystroke counts come from the separate operator transcription.
These assertions validate the captured states, not replayed UI gestures.
"""
import hashlib
import json
import math
from pathlib import Path

HERE = Path(__file__).resolve().parent
checks, files = [], {}


def check(name, passed, observed=None):
    checks.append(dict(name=name, passed=bool(passed), observed=observed))


def near(name, actual, expected, tolerance=1e-6):
    if not isinstance(actual, list):
        actual, expected = [actual], [expected]
    check(name, len(actual) == len(expected) and all(
        isinstance(a, (int, float)) and math.isfinite(a) and abs(a - e) <= tolerance
        for a, e in zip(actual, expected)
    ), dict(actual=actual, expected=expected, tolerance=tolerance))


def load(name, wrapped=True):
    raw = (HERE / name).read_bytes()
    files[name] = hashlib.sha256(raw).hexdigest()
    data = json.loads(raw)
    if wrapped:
        check(name + ' completed', data.get('ok') is True)
        return data['data']
    return data


def row(data, key):
    value = data[key]
    assert value['unique'] and not value['missing'] and not value['ambiguous']
    assert not value['enumerationErrors'] and len(value['rows']) == 1
    record = value['rows'][0]
    assert all(record[field]['ok'] for field in ['name', 'value', 'numKeys', 'expression'])
    return record


def value(data, key):
    return row(data, key)['value']['value']


try:
    identity = 'bdfc9f1d978cf052e01db97d69b1d61c1ad9cc073b7691a82e2cb2d90baaa141'
    for name in ['install-ae2026.json', '010c-aerender-identity-before.json', '010c-aerender-identity-after.json']:
        record = load(name, False)
        check(name + ' exact executable identity', record['binary_sha256'] == identity)
    setup = load('010c-setup.json')
    check('exact AE version', setup['version'] == '26.3x87' and setup['build'] == 87)
    check('18 fresh fixture instances default to GLSL', len(setup['defaults']) == 18 and all(x['value'] == 1 for x in setup['defaults']))
    actions = load('010c-ui-actions.json', False)
    check('operator record identifies candidate', actions['candidateBinarySha256'] == identity)
    check('UI record explicitly a transcription, not captured images', 'transcription' in actions['recordType'] and 'not captured image files' in actions['recordType'])
    check('Source Undo FAIL remains disclosed', 'Known FAIL' in actions['sourceUndo'] and 'ADR0045' in actions['sourceUndo'])

    fresh = {phase: load('010c-ui-fresh-' + phase + '.json') for phase in ['before', 'wgsl', 'undo', 'redo']}
    sources = [row(d, 'readSource')['expression']['value'] for d in fresh.values()]
    check('fresh Language gestures preserve exact source bytes', all(s == sources[0] for s in sources))
    names = {'gain': 'Fresh Gain', 'color': 'Fresh Color', 'alpha': 'Fresh Color A', 'angle': 'Fresh Angle'}
    for phase, data in fresh.items():
        active = phase in ['wgsl', 'redo']
        check(phase + ' language', value(data, 'readLanguage') == (2 if active else 1))
        state = value(data, 'readState')
        check(phase + ' active/E17 state', state % 4 == 1 if active else state == 70)
        check(phase + ' plan is published/cleared', value(data, 'readPlan') > 0 if active else value(data, 'readPlan') == 0)
        check(phase + ' fresh streams remain constant', all(row(data, k)['numKeys']['value'] == 0 for k in names))
        near(phase + ' gain default/restoration', value(data, 'gain'), .25 if active else 0)
        near(phase + ' color default/restoration at float precision', value(data, 'color'), [.25, .6, 1, 1] if active else [1, 1, 1, 1])
        near(phase + ' alpha default/restoration', value(data, 'alpha'), .4 if active else 0)
        near(phase + ' angle default/restoration at float precision', value(data, 'angle'), 12.34567 if active else 0)
        near(phase + ' rendered output', data['pixels'], [.0625, .15, .25, .4] if active else [0, 0, 0, 1], 1 / 255 + 1e-6)
        if active:
            check(phase + ' exact authored labels', {k: row(data, k)['name']['value'] for k in names} == names)
    check('fresh Undo restores values and words', all(value(fresh['undo'], k) == value(fresh['before'], k) for k in [*names, 'readState', 'readPlan']))
    check('fresh Redo restores values and words', all(value(fresh['redo'], k) == value(fresh['wgsl'], k) for k in [*names, 'readState', 'readPlan']))

    for suffix, expected in [('green', [0, .25, 0, 1]), ('envelope', [.25, .4, .6, 1])]:
        data = load('010c-ui-source-' + suffix + '.json')
        check(suffix + ' source uses only CR line breaks', '\r' in data['source'] and '\n' not in data['source'])
        check(suffix + ' annotation text retained', '// @param gain label:"Gain" min:0 max:2 default:0.25' in data['source'])
        check(suffix + ' WGSL Active state', data['state']['language'] == 2 and data['state']['token'] % 4 == 1 and data['state']['code'] == 0)
        check(suffix + ' readback did not write AE properties', data['readOnly'] is True)
        near(suffix + ' gain value', data['value0'], .25)
        near(suffix + ' rendered color', data['rgba'], expected)
        if suffix == 'envelope':
            check('CR envelope two-pass status and escaping', 'compiled: 2 passes' in data['state']['status'] and data['source'].count('@pass ') == 2 and '@@group' in data['source'])

    language = {phase: load('010c-ui-language-' + phase + '.json') for phase in ['glsl', 'undo', 'redo']}
    check('keyframed Language gestures preserve exact source', len({d['source'] for d in language.values()}) == 1)
    for phase, data in language.items():
        active = phase == 'undo'
        check(phase + ' keyframed language/state', data['state']['language'] == (2 if active else 1) and data['state']['code'] == (0 if active else 17))
        check(phase + ' both original keys retained', data['keys'] == 2 and data['value0'] == .25 and data['value1'] == .75)
        check(phase + ' keyframed readback read-only', data['readOnly'] is True)
        near(phase + ' keyframed rendering', data['rgba'], [.25, 0, 0, 1] if active else [.05, .4, .1, 1], 1e-6)

    failed = load('010c-resources-assign.json', False)
    check('initial hidden gradient write remains a failure', failed['ok'] is False and failed['line'] == 69 and '隐藏' in failed['error'])
    inspected = load('010c-gradient-inspect.json')
    check('failed assign retains partial selector changes', inspected['selectors'] == {'glslLayer': 3, 'glslPath': 1, 'wgslLayer': 0, 'wgslPath': 0})
    colors = {r['name']: r['value'] for r in inspected['rows']}
    check('first failed assignment left GLSL gradient colors untouched', colors['G01 Stop 01 Color'] == [0, 0, 0, 1] and colors['G01 Stop 02 Color'] == [1, 1, 1, 1])
    assigned = load('010c-resources-assign-after-ui.json')
    check('both assignments complete after actual ECW initialization', len(assigned['values']) == 2 and all(v['layer'] == 3 and v['path'] == 1 and v['stops'] == 2 for v in assigned['values']))
    for lang in ['glsl', 'wgsl']:
        for suffix, resolution in [('open', [1, 1]), ('half', [2, 2]), ('quarter', [4, 4])]:
            data = load('010c-viewport-' + lang + '-' + suffix + '.json')
            check(lang + ' viewport ' + suffix + ' requested resolution/extent', data['state']['resolution'] == resolution and [data['state']['width'], data['state']['height']] == [321, 239])
except (AssertionError, KeyError, TypeError, ValueError, OSError, IndexError) as error:
    check('record structure supports complete UI verification', False, str(error))

report = {
    'passed': all(c['passed'] for c in checks),
    'scope': 'Offline assertions over retained UI readbacks; no AE actions executed',
    'limits': [
        'Single keystrokes and physical visual observations come from 010c-ui-actions.json operator transcription.',
        'Source single-Undo remains FAIL under ADR0045 and is not recertified here.',
        'Source green/envelope helper accepts Gain or Float 01; those records do not independently prove exact Gain display text.',
        'Fresh-default records do expose exact authored labels; viewport numerical pixels are checked separately in the 24 PSDs.',
    ],
    'files': [{'file': k, 'sha256': v} for k, v in sorted(files.items())],
    'checks': checks,
}
print(json.dumps(report, indent=2, ensure_ascii=False))
raise SystemExit(0 if report['passed'] else 1)
