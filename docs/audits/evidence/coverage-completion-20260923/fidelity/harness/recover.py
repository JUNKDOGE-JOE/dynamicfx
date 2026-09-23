import json,os,sys,time
from pathlib import Path
import numpy as np
from frame import canvas
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();label=sys.argv[1];log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
info=json.loads((out/(label+'-summary.json')).read_text());copy=info['binding_before_render']['copy']
guard='if(!app.project.file||app.project.file.name!=='+json.dumps(label+'.aep')+')throw Error("Unexpected project");var c=app.project.itemByID(30),copy=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id==='+str(copy)+')copy=c.layer(i);if(!copy)throw Error("Copy missing");'
def call(name,code):
 args={'code':'(function(){'+guard+code+'})()','timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
 (out/(label+'-'+name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
 d=Client.data(r)
 if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
 return d['content']
state=json.loads(call('recovery-setup','app.project.itemByID(14).layer(1).property("ADBE Effect Parade").property(1).property(2).setValue(1);copy.property("ADBE Effect Parade").property(2).property(2).setValue(21);var e=copy.property("ADBE Effect Parade").property(1),reader=c.layer(e.property(426).value);return JSON.stringify({state:e.property(427).value,reader:reader.id,source:c.layer(reader.property("ADBE Effect Parade").property(1).property(1).value).id});'))
print(json.dumps(state),flush=True)
offset=log.stat().st_size;dest=(out/(label+'-recovered.png')).resolve().as_posix()
call('recovered','app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));app.project.save();return "rendered and saved";')
time.sleep(1)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/(label+'-recovered.native.log')).write_text(raw,encoding='utf-8')
a=canvas(raw);b=canvas((out/(label+'-reference.native.log')).read_text(encoding='utf-8'));count=int(np.count_nonzero(a!=b))
report={'case':'post-idle recovery','binding':state,'pixels':480000,'native_word_mismatches':count,'status':'PASS' if count==0 and state['source']==copy and state['state']>=4 else 'FAIL'}
(out/(label+'-recovery-summary.json')).write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)
