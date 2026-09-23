import json,sys,subprocess
from pathlib import Path
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
out=Path(__file__).resolve().parent
code='(function(){var c=app.project.itemByID(247),p=c.layer(1).property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property(1).property("ADBE Vector Shape"),s=p.value;p.setValueAtTime(0,s);s.vertices=[[35,20],[113,30],[50,75]];p.setValueAtTime(1,s);c.time=1.5;return JSON.stringify({keys:p.numKeys,cti:c.time});})()'
r=Client().call("ae_exec",{"code":code,"timeout_sec":15},timeout=25);(out/"animate-setup.json").write_text(json.dumps(r));print(Client.data(r).get("ok"),flush=True)
for t in [0.75,0.0,0.25]:
 d=out/("animated-"+str(t));d.mkdir(exist_ok=True);(d/"setup.result.json").write_bytes((out/"setup.result.json").read_bytes())
 source=(out/"native_cases.py").read_text(encoding="utf-8").replace("results=[]",'rows=[r for r in rows if r["name"]=="curve"]\nresults=[]').replace("valueAtTime(0,false)","valueAtTime("+str(t)+",false)").replace("saveFrameToPng(0,","saveFrameToPng("+str(t)+",")
 (d/"run.py").write_text(source,encoding="utf-8");subprocess.run([sys.executable,str(d/"run.py")],check=True)
