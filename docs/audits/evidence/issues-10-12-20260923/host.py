import json,sys
from pathlib import Path
out=Path(__file__).resolve().parent
sys.path.insert(0,r'E:/Code/AePlugin_Dynamicfx/spike/host-outline')
from mcp_client import Client
def call(name,code,timeout=30):
    args={'code':code,'timeout_sec':timeout}
    response=Client().call('ae_exec',args,timeout=timeout+10)
    (out/(name+'.json')).write_text(json.dumps({'args':args,'result':response},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(response)
    if not d.get('ok'):raise RuntimeError(d.get('error',d))
    data=json.loads(d['content']);print(json.dumps({'step':name,'result':data},ensure_ascii=True),flush=True);return data
