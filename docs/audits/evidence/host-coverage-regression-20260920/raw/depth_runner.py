import json,sys,subprocess
from pathlib import Path
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
out=Path(__file__).resolve().parent
for bpc in [8,32,16]:
 r=Client().call("ae_exec",{"code":"app.project.bitsPerChannel="+str(bpc)+"; JSON.stringify({bpc:app.project.bitsPerChannel});","timeout_sec":10},timeout=20)
 (out/("depth-"+str(bpc)+".json")).write_text(json.dumps(r));print("depth",bpc,Client.data(r).get("ok"),flush=True)
 directory=out/("depth-"+str(bpc));directory.mkdir(exist_ok=True)
 (directory/"setup.result.json").write_bytes((out/"setup.result.json").read_bytes())
 source=(out/"native_cases.py").read_text(encoding="utf-8").replace("results=[]",'rows=[r for r in rows if r["name"] in ["curve","half"]]\nresults=[]')
 (directory/"run.py").write_text(source,encoding="utf-8")
 result=subprocess.run([sys.executable,str(directory/"run.py")]);print("exit",result.returncode,flush=True)
