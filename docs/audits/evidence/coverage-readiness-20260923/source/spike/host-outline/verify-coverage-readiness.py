"""Recompute coverage copy, FFX, cache and offline acceptance from native logs."""
import gzip
import json
from pathlib import Path
import re
import sys

root = Path(sys.argv[1]) / 'raw'

def frame(name, depth=32):
    text = gzip.decompress((root / (name + '.native.log.gz')).read_bytes()).decode('utf-8').replace('\r\n', '\n')
    matches = list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])', text))
    assert len(matches) == 1, (name, len(matches))
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

def mismatch(a, b):
    assert len(a) == len(b)
    return sum(x != y for x, y in zip(a, b))

reference = frame('after4-reference')
assert any(reference) and max(reference) < 0x3f800000
assert mismatch(frame('before-immediate'), frame('before-reference')) == 42331
assert mismatch(frame('after-immediate'), frame('after-reference')) == 42331

warm = frame('after4-immediate')
assert set(warm) == {0, 0x3f800000}
assert sum(value != 0 for value in warm) == 140000
assert 'coverage gate refused: E60' in (root / 'after4-gate.main.log').read_text(encoding='utf-8')
assert frame('after4-recovered') == reference

for case in ['new', 'existing']:
    pending = frame('ffx-' + case + '-immediate')
    assert set(pending) == {0, 0x3f800000}
    assert sum(value != 0 for value in pending) == 140000
    messages = (root / ('ffx-' + case + '-immediate.main.log')).read_text(encoding='utf-8')
    assert 'coverage gate refused: E60' in messages
    assert frame('ffx-' + case + '-recovered') == reference
summary = json.loads((root / 'ffx-summary.json').read_text(encoding='utf-8'))
assert summary[2]['binding']['count'] == 1
assert summary[2]['binding']['previous'] != summary[2]['binding']['state']
assert all(row['status'] == 'PASS' for row in summary)

assert frame('cold') == frame('after3-reference')
assert 'coverage gate refused:' not in (root / 'cold.main.log').read_text(encoding='utf-8')
assert 'project_flag=false render_engine=true' in (root / 'cold.main.log').read_text(encoding='utf-8')
assert frame('pending') == reference
missing = frame('missing')
assert set(missing) == {0, 0x3f800000}
assert 'coverage gate refused: E60' in (root / 'missing.main.log').read_text(encoding='utf-8')

for depth in [8, 16, 32]:
    original = frame('readiness-reference-' + str(depth), depth)
    assert any(original)
    for kind in ['production', 'adjustment']:
        assert frame(f'readiness-{kind}-{depth}', depth) == original

print(json.dumps({'status': 'PASS', 'baseline_failure_pixels': 42331,
    'warm_copy_protection': 1, 'copy_recovery': 1, 'ffx_protection_and_recovery': 4,
    'prepared_offline': 1, 'save_time_validation': 1, 'missing_reader_offline': 1,
    'depth_regressions': 6}))
