import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
label=sys.argv[1] if len(sys.argv)>1 else 'before'
def call(name,code):
 args={'code':'(function(){'+code+'})()','timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
 (out/(label+'-'+name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
 d=Client.data(r)
 if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
 return d['content']
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix()
project=(out/(label+'.aep')).resolve().as_posix()
call('prepare','if(!app.project.file||!(/^(production-coverage|cold|before|after[0-9]*)\\.aep$/.test(app.project.file.name)))throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));for(var i=1;i<=app.project.numItems;i++){var c=app.project.item(i);if(!(c instanceof CompItem))continue;for(var j=1;j<=c.numLayers;j++){var fx=c.layer(j).property("ADBE Effect Parade");for(var k=1;k<=fx.numProperties;k++)if(fx.property(k).matchName==="DynamicFx Host Shape Probe"){fx.property(k).property(2).setValue(1);fx.property(k).property(6).setValue(0);}}}app.project.save(new File('+json.dumps(project)+'));return "prepared";')
time.sleep(3)
guard='if(!app.project.file||app.project.file.fsName.replace(/\\\\/g,"/")!=='+json.dumps(project)+')throw Error("Unexpected project");'
shape='var p=L.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),s=p.value;s.vertices=[[50,150],[300,150],[175,500]];p.setValue(s);'
if len(sys.argv)>2 and sys.argv[2]=='warm':
 shape=shape.replace('[[50,150],[300,150],[175,500]]','[[200,500],[600,500],[400,150]]')
 call('warm',guard+'app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID(30).saveFrameToPng(0,new File('+json.dumps((out/(label+'-warm.png')).resolve().as_posix())+'));return "warm";')
 time.sleep(1)
offset=log.stat().st_size
dest=(out/(label+'-immediate.png')).resolve().as_posix()
code=guard+'var c=app.project.itemByID(30),owner=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===44)owner=c.layer(i);if(!owner)throw Error("Missing owner");owner.enabled=false;var copy=owner.duplicate();copy.enabled=true;copy.name="Immediate copy";'+shape.replace('L.property','copy.property')+'var fx=copy.property("ADBE Effect Parade"),e=fx.property(1);fx.property(2).property(2).setValue(21);var before={copy:copy.id,state:e.property(427).value,reader:c.layer(e.property(426).value).id};app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return JSON.stringify(before);'
if len(sys.argv)>2 and sys.argv[2]=='warm': code=code.replace('app.purge(PurgeTarget.ALL_CACHES);','')
info=json.loads(call('immediate',code));print(json.dumps(info),flush=True)
time.sleep(1)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/(label+'-immediate.native.log')).write_text(raw,encoding='utf-8')
call('reference-setup',guard+'var c=app.project.itemByID(30);for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id==='+str(info['copy'])+')c.layer(i).property("ADBE Effect Parade").property(2).property(2).setValue(1);var L=app.project.itemByID(14).layer(1);'+shape+'L.property("ADBE Effect Parade").property(1).property(2).setValue(21);return "reference set";')
offset=log.stat().st_size;dest=(out/(label+'-reference.png')).resolve().as_posix()
call('reference',guard+'app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID(14).saveFrameToPng(0,new File('+json.dumps(dest)+'));return "rendered";')
time.sleep(1)
with log.open('rb') as f:f.seek(offset);reference=f.read().decode('utf-8',errors='replace')
(out/(label+'-reference.native.log')).write_text(reference,encoding='utf-8')
def array(text):
 matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=32 origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text))
 if len(matches)!=1:raise RuntimeError('Expected one frame, got '+str(len(matches)))
 m=matches[0];w,h,x,y=map(int,m.groups()[:4]);words=np.array(json.loads(m[5]),dtype=np.uint32).reshape(h,w);a=np.zeros((600,800),dtype=np.uint32);l,t,r,b=max(x,0),max(y,0),min(x+w,800),min(y+h,600);a[t:b,l:r]=words[t-y:b-y,l-x:r-x];return a
a,b=array(raw),array(reference);mismatches=int(np.count_nonzero(a!=b));report={'case':'copy edit and immediate render before idle','binding_before_render':info,'native_word_mismatches':mismatches,'reference_nonzero':int(np.count_nonzero(b))}
(out/(label+'-summary.json')).write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)
call('save',guard+'app.project.save();return "saved";')
