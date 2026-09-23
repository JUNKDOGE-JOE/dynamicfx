from pathlib import Path
import json, sys
sys.path.insert(0, str(Path(__file__).resolve().parents[6] / 'spike/host-outline'))
from mcp_client import Client

out = Path(__file__).parent
name = sys.argv[1]
code = (out / (name + '.jsx')).read_text(encoding='utf-8') if name != 'status' else 'JSON.stringify({version:app.version,project:app.project.file?app.project.file.fsName:null,dirty:app.project.dirty,items:app.project.numItems})'
try:
    arguments = {'code':code, 'timeout_sec':25}
    result = Client().call('ae_exec', arguments, timeout=35)
    (out / (name + '.result.json')).write_text(json.dumps({'args':arguments,'result':result},ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(Client.data(result),ensure_ascii=True))
except Exception as error:
    print(json.dumps({'error':str(error)},ensure_ascii=True))
    raise SystemExit(1)
