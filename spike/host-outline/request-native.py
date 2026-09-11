"""Trigger one idle diagnostic through MCP parameter writes; never activate AE."""
import json
import os
from pathlib import Path
import re
import sys
import time
from mcp_client import Client

OUT = Path(__file__).resolve().parents[2] / 'scripts/out/host-shape-native-20260910'
label, comp_id, mode, seconds, serial = sys.argv[1:6]
comp_id, mode, serial, seconds = int(comp_id), int(mode), int(serial), float(seconds)
if not re.fullmatch(r'[A-Za-z0-9_-]+', label) or mode not in (2, 3, 4, 6, 7) or not 1 <= serial <= 1_000_000:
    raise ValueError('Invalid diagnostic request')
log = Path(os.environ['TEMP']) / 'dynamicfx-host-shape-probe.log'
initial = log.read_text(encoding='utf-8')
ready = re.findall(r'BACKGROUND_READY comp=' + str(comp_id) + r' layer=(\d+) effect=1 serial=(\d+)', initial)
if not ready:
    raise RuntimeError('No background-ready event for this fixture; wait for the new probe idle scan')
result_path = OUT / (label + '.result.json')
if result_path.exists():
    raise RuntimeError('Evidence name already exists')
offset = log.stat().st_size
code = '''(function(){
if(!app.project.file||app.project.file.name!=="host-shape-native.aep")throw Error("Wrong project");
var c=app.project.itemByID(COMP);
if(!c||c.name.indexOf("HS_")!==0)throw Error("Wrong fixture");
var p=c.layer(1).property("ADBE Effect Parade").property(1);
if(p.matchName!=="DynamicFx Host Shape Probe"||p.numProperties<5||p.property(5).name!=="Background request serial")throw Error("Background probe is not loaded");
if(MODE>p.property(2).maxValue)throw Error("Requested mode is not available in the loaded probe");
if(p.property(5).value===SERIAL)throw Error("Serial has already been submitted; inspect its result");
p.property(1).setValue(SECONDS);p.property(2).setValue(MODE);p.property(5).setValue(SERIAL);
return JSON.stringify({comp:c.id,serial:p.property(5).value,mode:p.property(2).value,layerSeconds:p.property(1).value});
})()'''.replace('COMP', str(comp_id)).replace('MODE', str(mode)).replace('SERIAL', str(serial)).replace('SECONDS', json.dumps(seconds, allow_nan=False))
args = {'code':code, 'undo_group_name':'Request background host-shape diagnostic', 'timeout_sec':20}
(OUT / (label + '.args.json')).write_text(json.dumps(args,indent=2),encoding='utf-8')
result = Client().call('ae_exec',args,timeout=30)
result_path.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
if not Client.data(result).get('ok'):
    raise RuntimeError('MCP request did not succeed; do not resubmit without readback')
deadline = time.monotonic() + 30
terminal = re.compile(r'BACKGROUND_(COMPLETE|FAILED) comp=' + str(comp_id) + r' serial=' + str(serial) + r'\b')
while time.monotonic() < deadline:
    with log.open('rb') as file:
        file.seek(offset)
        output = file.read().decode('utf-8')
    (OUT / (label + '.native.log')).write_text(output,encoding='utf-8')
    match = terminal.search(output)
    if match:
        print(output)
        raise SystemExit(0 if match.group(1) == 'COMPLETE' else 1)
    time.sleep(0.25)
raise RuntimeError('No terminal diagnostic event; preserve this uncertain attempt without automatic retry')
