import json,os,re,sys,time,shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
guard='if(!app.project.file||app.project.file.name!=="production-coverage.aep")throw Error("Unexpected project");var c=app.project.itemByID(30),owner=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===44)owner=c.layer(i);if(!owner)throw Error("Owner missing");'
def call(name,code):
 args={'code':'(function(){'+guard+code+'})()','timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
 (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
 d=Client.data(r)
 if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
 return d['content']
for name in ['control-background-32.native.log','control-summary.json','control-disable.result.json','control-render.result.json','control-restore.result.json']:
 if (out/name).exists():shutil.copyfile(out/name,out/name.replace('control-','control-second-',1))
previous=json.loads(call('control-disable','var probe=owner.property("ADBE Effect Parade").property(2),previous=probe.property(6).value,modes=[];for(var i=1;i<=app.project.numItems;i++){var other=app.project.item(i);if(!(other instanceof CompItem)||other.id===30)continue;for(var j=1;j<=other.numLayers;j++){var layer=other.layer(j),fx=layer.property("ADBE Effect Parade");for(var k=1;k<=fx.numProperties;k++)if(fx.property(k).matchName==="DynamicFx Host Shape Probe"){modes.push([other.id,layer.id,k,fx.property(k).property(2).value]);fx.property(k).property(2).setValue(1);}}}probe.property(6).setValue(0);owner.property("ADBE Effect Parade").property(1).enabled=false;return JSON.stringify({original:previous,modes:modes});'))
time.sleep(3)
try:
 offset=log.stat().st_size;dest=(out/'control-background-32.png').resolve()
 call('control-render','app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest.as_posix())+'));return "rendered";')
 for _ in range(40):
  if dest.exists() and dest.read_bytes().endswith(b'IEND\xaeB`\x82'):break
  time.sleep(.25)
 with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
 (out/'control-background-32.native.log').write_text(raw,encoding='utf-8')
 matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=32 origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw))
 if len(matches)!=1:raise RuntimeError('Expected one control alpha readback, got '+str(len(matches)))
 m=matches[0]
 values=json.loads(m[5]);opaque=sum(v==0x3f800000 for v in values)
 result={'case':'disabled shader exposes background, not original alpha','opaque_words':opaque,'words':len(values),'status':'PASS' if opaque==len(values) and opaque>0 else 'FAIL'}
 (out/'control-summary.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result))
finally:
 print(call('control-restore','var modes='+json.dumps(previous['modes'])+';for(var i=0;i<modes.length;i++){var other=app.project.itemByID(modes[i][0]);for(var j=1;j<=other.numLayers;j++)if(other.layer(j).id===modes[i][1])other.layer(j).property("ADBE Effect Parade").property(modes[i][2]).property(2).setValue(modes[i][3]);}owner.property("ADBE Effect Parade").property(2).property(6).setValue('+str(previous['original'])+');owner.property("ADBE Effect Parade").property(1).enabled=true;app.project.save();return JSON.stringify({enabled:owner.property("ADBE Effect Parade").property(1).enabled,dirty:app.project.dirty});'))
