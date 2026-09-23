"""Verify frozen production reader pixels and lifecycle readbacks."""
import gzip
import json
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
raw = root / 'raw'

def frame(name, depth):
    text = gzip.decompress((raw / (name + '.native.log.gz')).read_bytes()).decode('utf-8').replace('\r\n', '\n')
    matches = list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])', text))
    assert len(matches) == 1, (name, 'frame count', len(matches))
    match = matches[0]
    width, height, actual_depth, x, y = map(int, match.groups()[:5])
    assert actual_depth == depth
    words = json.loads(match[6])
    assert len(words) == width * height
    canvas = [0] * 480000
    left, top, right, bottom = max(x, 0), max(y, 0), min(x + width, 800), min(y + height, 600)
    for row in range(top, bottom):
        start = (row - y) * width + left - x
        canvas[row * 800 + left:row * 800 + right] = words[start:start + right - left]
    return canvas

comparisons = []
for depth in [8, 16, 32]:
    reference = frame(f'reference-{depth}', depth)
    full = {8: 255, 16: 32768, 32: 0x3f800000}[depth]
    assert 0 < max(reference) < full
    assert reference[50 * 800 + 50] == 0
    assert reference[300 * 800 + 400] < reference[200 * 800 + 400]
    for kind in ['production', 'adjustment']:
        actual = frame(f'{kind}-{depth}', depth)
        mismatches = sum(a != b for a, b in zip(reference, actual))
        assert mismatches == 0, (kind, depth, mismatches)
        comparisons.append({'kind': kind, 'depth': depth, 'pixels': len(actual), 'mismatches': mismatches})
assert frame('reopen-adjustment-32', 32) == frame('reference-32', 32)
control = frame('control-background-32', 32)
reference = frame('reference-32', 32)
assert sum(a != b for a, b in zip(control, reference)) == 140000
assert set(control) == {0, 0x3f800000}

def snapshot(name):
    result = json.loads((raw / ('life-' + name + '.json')).read_text(encoding='utf-8'))['result']
    data = result.get('structuredContent')
    if data is None:
        data = json.loads('\n'.join(block['text'] for block in result['content'] if block['type'] == 'text'))
    assert data['ok']
    return json.loads(data['content'])

base = snapshot('baseline')
items = base['items']
assert len(base['readers']) == 1 and base['owners'][0]['effects'][0]['state'] == 1
shared = snapshot('share')
assert len(shared['readers']) == 1 and len(shared['owners'][0]['effects']) == 2
assert all(e['state'] == 1 and e['linked'] == shared['readers'][0]['id'] for e in shared['owners'][0]['effects'])
first = snapshot('first-off')
assert len(first['readers']) == 1 and [e['state'] for e in first['owners'][0]['effects']] == [0, 1]
for name in ['last-off', 'redo-cleanup']:
    value = snapshot(name)
    assert len(value['readers']) == 0 and value['items'] == items - 1
undo = snapshot('undo-cleanup')
assert len(undo['readers']) == 1 and undo['items'] == items
enabled = snapshot('reenable')
assert len(enabled['readers']) == 1 and enabled['owners'][0]['effects'][0]['state'] == 1
copy = snapshot('duplicate-owner')
assert len(copy['readers']) == 2 and len(copy['owners']) == 2
assert len({owner['effects'][0]['linked'] for owner in copy['owners']}) == 2
assert all(any(reader['id'] == owner['effects'][0]['linked'] and reader['source'] == owner['id']
               for reader in copy['readers']) for owner in copy['owners'])
print(json.dumps({'status': 'PASS', 'pixel_comparisons': comparisons, 'reopen_comparisons': 1, 'negative_controls': 1, 'lifecycle_cases': 8}))
