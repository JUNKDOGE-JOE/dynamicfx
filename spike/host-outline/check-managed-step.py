import json
import os
from pathlib import Path
import sys
import time
from mcp_client import Client

OUT = Path(__file__).resolve().parents[2] / 'scripts/out/host-shape-native-20260910'
label, action = sys.argv[1:3]
actions = {
    'disable-first': 'f.property(1).property(7).setValue(false);',
    'disable-last': 'f.property(2).property(7).setValue(false);',
    'reenable': 'f.property(1).property(7).setValue(true);',
    'undo': 'app.executeCommand(16);',
    'redo': 'app.executeCommand(2035);',
}
expected = {
    'disable-first': ([0, 1], 2), 'disable-last': ([0, 0], 1),
    'reenable': ([1, 0], 2), 'undo': ([1, 0], 1), 'redo': ([1, 0], 2),
}
if action not in actions or (OUT / (label + '.result.json')).exists():
    raise ValueError('Invalid action or reused evidence name')
code = '''(function(){var p=app.project;if(!p.file||p.file.name!=="host-shape-native.aep")throw Error("Wrong project");
var c=p.itemByID(174);if(c.name!=="HS_auto_base")throw Error("Wrong fixture");
var f=c.layer(1).property("ADBE Effect Parade");if(f.numProperties!==2)throw Error("Wrong effect count");
ACTION return "applied";})()'''.replace('ACTION', actions[action])
args = {'code': code, 'timeout_sec': 15}
(OUT / (label + '.args.json')).write_text(json.dumps(args, indent=2), encoding='utf-8')
log = Path(os.environ['TEMP']) / 'dynamicfx-host-shape-probe.log'
offset = log.stat().st_size
client = Client()
result = client.call('ae_exec', args, timeout=25)
(OUT / (label + '.result.json')).write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
if not Client.data(result).get('ok'):
    raise RuntimeError('Uncertain mutation; inspect before retrying')
time.sleep(2)
state_args = json.loads((OUT / 'v009-lifecycle-state.args.json').read_text(encoding='utf-8'))
state = client.call('ae_exec', state_args, timeout=25)
(OUT / (label + '.state.json')).write_text(json.dumps(state, ensure_ascii=False, indent=2), encoding='utf-8')
with log.open('rb') as file:
    file.seek(offset)
    (OUT / (label + '.native.log')).write_bytes(file.read())
data = Client.data(state)
if not data.get('ok'):
    raise RuntimeError('State read failed')
rows = json.loads(data['content'])['rows']
base = next(row for row in rows if row['id'] == 174)
duplicate = next(row for row in rows if row['id'] == 188)
owners, masks = expected[action]
checks = {'base_owner_flags': base['owners'] == owners,
          'base_masks': len(base['masks']) == masks,
          'duplicate_unchanged': duplicate['owners'] == [1, 1] and len(duplicate['masks']) == 2,
          'user_masks_preserved': all(row['masks'][0]['head'] == [[40,40],[80,40],[80,80],[40,80]] for row in rows)}
(OUT / (label + '.checks.json')).write_text(json.dumps(checks, indent=2), encoding='utf-8')
print(json.dumps({'action': action, 'checks': checks, 'rows': rows}, ensure_ascii=True))
if not all(checks.values()):
    raise SystemExit(1)
