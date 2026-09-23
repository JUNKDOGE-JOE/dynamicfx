import json,os,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
original=(out.parent/'direct-source-20260920/source-matrix.aep').resolve().as_posix()
destination=(out/'native-reader.aep').resolve().as_posix()
code='''(function(){
if(app.project.file&&app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");
if(!app.project.file)app.open(new File(ORIGINAL));
for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i).name==="HS_reader_native")throw Error("Already created");
var c=app.project.itemByID(866).duplicate();c.name="HS_reader_native";
var l=c.layer(1),e=l.property("ADBE Effect Parade").property(1);l.adjustmentLayer=true;e.property(2).setValue(21);e.property(4).setValue(true);e.property(10).setValue(false);
app.project.save(new File(DESTINATION));return JSON.stringify({comp:c.id,owner:l.id,layers:c.numLayers,items:app.project.numItems,version:app.version});
})()'''.replace('ORIGINAL',json.dumps(original)).replace('DESTINATION',json.dumps(destination))
args={'code':code,'timeout_sec':30};r=client.call('ae_exec',args,timeout=40)
(out/'setup.result.json').write_text(json.dumps({'args':args,'result':r},indent=2),encoding='utf-8')
d=Client.data(r)
if not d.get('ok'):raise RuntimeError(d)
info=json.loads(d['content']);(out/'fixture.json').write_text(json.dumps(info,indent=2),encoding='utf-8');print(info,flush=True)
time.sleep(2);offset=log.stat().st_size
code='(function(){if(app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");var c=app.project.itemByID('+str(info['comp'])+');c.layer(1).property("ADBE Effect Parade").property(1).property(10).setValue(true);return "enabled";})()'
r=client.call('ae_exec',{'code':code,'timeout_sec':15},timeout=25);(out/'enable.result.json').write_text(json.dumps(r,indent=2),encoding='utf-8')
time.sleep(4)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/'enable.native.log').write_text(raw,encoding='utf-8');print(raw[-6000:])
