import json,sys,time
from pathlib import Path
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
out=Path(__file__).resolve().parent;c=Client();results=[]
prefix='(function(){if(!app.project.file||app.project.file.parent.name!=="regression-20260920")throw Error("Wrong fixture");var l=app.project.itemByID(174).layer(1),f=l.property("ADBE Effect Parade");'
state='var m=l.property("ADBE Mask Parade"),rows=[];for(var i=1;i<=m.numProperties;i++){var p=m.property(i).property("ADBE Mask Shape"),owned=p.expression.indexOf("// DynamicFX internal coverage raster carrier;")===0;rows.push({owned:owned,mode:m.property(i).maskMode,userVertices:owned?null:p.value.vertices});}return JSON.stringify({owners:[f.property(1).property(7).value,f.property(2).property(7).value],masks:rows});})()'
for label,action,owners,count in [("share","f.property(2).property(7).setValue(true);",[1,1],2),("disable-first","f.property(1).property(7).setValue(false);",[0,1],2),("disable-last","f.property(2).property(7).setValue(false);",[0,0],1),("reenable","f.property(1).property(7).setValue(true);",[1,0],2),("undo","app.executeCommand(16);",[1,0],1),("redo","app.executeCommand(2035);",[1,0],2)]:
 args={"code":prefix+action+'return JSON.stringify({applied:true});})()',"timeout_sec":15};r=c.call("ae_exec",args,timeout=25);(out/("life-"+label+".action.json")).write_text(json.dumps({"args":args,"result":r}))
 if not Client.data(r).get("ok"):raise RuntimeError(label)
 time.sleep(1.5);args={"code":prefix+state,"timeout_sec":15};r=c.call("ae_exec",args,timeout=25);(out/("life-"+label+".state.json")).write_text(json.dumps({"args":args,"result":r}));row=json.loads(Client.data(r)["content"])
 passed=row["owners"]==owners and len(row["masks"])==count and row["masks"][0]["userVertices"]==[[40,40],[80,40],[80,80],[40,80]]
 results.append({"step":label,"status":"PASS" if passed else "FAIL","state":row});print(label,passed,flush=True)
(out/"lifecycle-checks.json").write_text(json.dumps(results,indent=2))
