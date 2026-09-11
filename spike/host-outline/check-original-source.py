import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'scripts/out/host-shape-native-20260910'
DEST = ROOT / 'docs/audits/evidence/host-shape-native-20260910/original-source-v005'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def result(label):
    r = json.loads((OUT / (label + '.result.json')).read_text(encoding='utf-8-sig'))
    d = r.get('structuredContent') or json.loads(r['content'][0]['text'])
    if not d.get('ok'):
        raise ValueError('Expected successful MCP result: ' + label)
    return json.loads(d['content'])


def copy(source, name=None):
    target = DEST / (name or source.name)
    if target.exists() and sha(source) != sha(target):
        raise ValueError('Evidence already exists with other bytes: ' + str(target))
    if not target.exists():
        shutil.copy2(source, target)
    return {'path': target.name, 'sha256': sha(target)}


samples = result('sample-original-alpha')['samples']
assert [v['alpha'] for v in samples] == [[0,1,0], [0,1,0], [0,1,0], [1,1,1]]
for label, expected in [('v005-self-shape', [1,1,1,1,1,1]), ('v005-self-control', [0,1,0,0,1,0])]:
    text = (OUT / (label + '.native.log')).read_text(encoding='utf-8')
    values = list(map(float, re.findall(r'original_alpha\[[^]]+\]=([\d.]+)', text)))
    assert values == expected
assert result('sample-builtin-source')['alphaText'] == '[1,1,1]'
assert 'budget exceeded' in result('coverage-partial-opacity')['error']
assert 'budget exceeded' in result('coverage-v2-partial-opacity')['error']
assert result('coverage-v2-restore-readback')['vertices'] == [[200,150]]
standby = result('v005-standby')
assert standby['dirty'] is False and standby['items'] == 18
assert standby['partialExpressionDisabled']
assert '9 passed; 0 failed' in (OUT / 'v005-tests.txt').read_text(encoding='utf-8-sig')
comparisons = json.loads((OUT / 'coverage-block-comparisons.json').read_text(encoding='utf-8'))
assert all(r['within_one_8bit_level'] for r in comparisons['rows'])
first = json.loads((OUT / 'coverage-invalidation-comparison.json').read_text(encoding='utf-8'))
fixed = json.loads((OUT / 'coverage-v2-edge-comparison.json').read_text(encoding='utf-8'))
assert first['max_error_8bit'] == 31 and first['pixels_changed_after_original_path_edit'] == 8509
assert fixed['max_error_8bit'] == 1
subprocess.run(['git','diff','--quiet','HEAD','--','src','Cargo.toml','Cargo.lock','build.rs','scripts/macos.py'], cwd=ROOT, check=True)
install = Path('C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/Plug-ins/DynamicFx')
assert sha(install / 'DynamicFx.aex') == 'c91db8c0435ccf90e0fac33f5473825c29bbd959cfeef8cc5fcebe80092b0d0a'
assert sha(install / 'DynamicFxHostShapeProbe.aex') == sha(OUT / 'DynamicFxHostShapeProbe-v005.aex')
pixel_controls = {}
for a,b in [('v005-self-control.png','reference-t0.png'), ('v005-self-shape.png','auxiliary-shape-t0.png'), ('v005-coverage-blocks.png','auxiliary-shape-t0.png')]:
    left, right = Image.open(OUT/a).convert('RGBA'), Image.open(OUT/b).convert('RGBA')
    assert left.size == right.size and left.tobytes() == right.tobytes()
    pixel_controls[a] = {'reference': b, 'equal_rgba_bytes': True}

labels = ['coverage-resume-state','sample-original-alpha','quit-for-v005','reopen-v005-state',
          'reopen-v005-fixture','v005-enable','v005-bind-self','v005-original-shape','v005-self-shape',
          'v005-self-control','prepare-builtin-source','inspect-builtin-source','sample-builtin-source',
          'prepare-coverage-blocks','read-coverage-blocks','enable-coverage-blocks','v005-coverage-blocks',
          'prepare-coverage-animated','coverage-animated-t1','coverage-reference-t1',
          'coverage-animated-t0','coverage-reference-t0','coverage-animated-t05','coverage-reference-t05',
          'coverage-partial-opacity','coverage-edit-path','coverage-animated-edited-t05',
          'coverage-reference-edited-t05','coverage-restore-path','update-coverage-blocks-v2',
          'coverage-v2-edit-path','coverage-v2-edited-t05','coverage-v2-restore-path',
          'coverage-v2-restore-readback','coverage-v2-partial-opacity','v005-standby']
files = []
for label in labels:
    for source in sorted(OUT.glob(label + '.*')):
        if source.suffix in ('.json','.log','.png'):
            files.append(copy(source))
for name in ['v005-coverage-blocks-late.native.log','coverage-block-comparisons.json',
             'coverage-invalidation-comparison.json','coverage-v2-edge-comparison.json',
             'install-v005.json','launch-v005.json','v005-tests.txt','coverage-blocks-expression-v1.js']:
    files.append(copy(OUT/name))
for source in sorted(OUT.glob('*-decoded.png')):
    if source.name.startswith(('v005-', 'coverage-')):
        files.append(copy(source))
sources = [ROOT / ('spike/host-outline/native/' + s) for s in ['Cargo.toml','Cargo.lock','build.rs','src/lib.rs','src/read.rs','src/idle.rs']]
sources += [ROOT / ('spike/host-outline/' + s) for s in ['coverage-blocks-expression.js','sample-original-alpha.jsx','edit-coverage-fixture.jsx','verify-coverage-blocks.py','capture-native.py','check-original-source.py']]
source_hashes = {str(p.relative_to(ROOT)): sha(p) for p in sources}
for n,p in enumerate(sources):
    copy(p, f'source-{n:02}-{p.name}.txt')
checks = {'status':'PASS for recorded feasibility checks; production feature NOT_RUN',
          'source_baseline':'ff895649bf99ce52055e066dd986f49e7bf6956e',
          'diagnostic_sha256':sha(OUT/'DynamicFxHostShapeProbe-v005.aex'),
          'fixture_sha256':sha(OUT/'host-shape-native-v005-final.aep'),
          'fixture_location':'scripts/out/host-shape-native-20260910/host-shape-native-v005-final.aep',
          'native_tests':9,'source_samples':samples,'comparisons':comparisons,
          'first_edge_failure':first,'edge_fix':fixed,'pixel_controls':pixel_controls,
          'source_files':source_hashes,'files':files,'standby':standby,
          'pending':'User decision on managed data mask; partial alpha budget, original masks and production lifecycle unresolved'}
(DEST/'checks.json').write_text(json.dumps(checks, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps({'recorded_files':len(files),'source_snapshots':len(sources),'tests':9,'controls':len(pixel_controls),'feature':'NOT_RUN'}))
