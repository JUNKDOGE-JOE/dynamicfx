import json,os,sys,time
from pathlib import Path
import numpy as np
from frame import canvas
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log';mainlog=Path(os.environ['TEMP'])/'dynamicfx.log'
def call(name,code):
 args={'code':'(function(){'+code+'})()','timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
 (out/('ffx-'+name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
 d=Client.data(r)
 if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
 return d['content']
def capture(name,code):
 offset=log.stat().st_size;mo=mainlog.stat().st_size
 info=json.loads(call(name,code));time.sleep(1)
 with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
 with mainlog.open('rb') as f:f.seek(mo);messages=f.read().decode('utf-8',errors='replace')
 (out/('ffx-'+name+'.native.log')).write_text(raw,encoding='utf-8');(out/('ffx-'+name+'.main.log')).write_text(messages,encoding='utf-8')
 return info,canvas(raw),messages
select='for(var i=1;i<=c.numLayers;i++)c.layer(i).selected=false;var selected=c.selectedProperties;for(var i=selected.length-1;i>=0;i--)try{selected[i].selected=false;}catch(ignore){}'
preset=(out/'coverage-test.ffx').resolve().as_posix()
project=(out/'ffx.aep').resolve().as_posix()
setup='if(!app.project.file||app.project.file.name!=="after5.aep")throw Error("Unexpected project");var c=app.project.itemByID(30),source=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===44)source=c.layer(i);if(!source)throw Error("Source missing");'+select+'source.selected=true;source.property("ADBE Effect Parade").property(1).selected=true;source.savePreset(new File('+json.dumps(preset)+'));for(var i=1;i<=app.project.numItems;i++){var comp=app.project.item(i);if(!(comp instanceof CompItem))continue;for(var j=1;j<=comp.numLayers;j++){var fx=comp.layer(j).property("ADBE Effect Parade");for(var k=1;k<=fx.numProperties;k++)if(fx.property(k).matchName==="DynamicFx Host Shape Probe")fx.property(k).property(2).setValue(1);}}for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===47)c.layer(i).enabled=false;app.project.save(new File('+json.dumps(project)+'));return JSON.stringify({presetBytes:File('+json.dumps(preset)+').length,sourceState:source.property("ADBE Effect Parade").property(1).property(427).value});'
if len(sys.argv)<2: print('preset',call('save-preset',setup),flush=True)
guard='if(!app.project.file||app.project.file.name!=="ffx.aep")throw Error("Unexpected project");var c=app.project.itemByID(30);'
dest=lambda name:json.dumps((out/('ffx-'+name+'.png')).resolve().as_posix())
render=lambda name:'app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+dest(name)+'));'
count='var fx=target.property("ADBE Effect Parade"),count=0,main=null;for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx"){count++;main=fx.property(j);}if(count!==1)throw Error("Expected one DynamicFx, got "+count);if(main.propertyIndex!==1)main.moveTo(1);fx=target.property("ADBE Effect Parade");var main=fx.property(1);fx.property(2).property(2).setValue(21);fx.property(2).property(6).setValue(0);'
create='app.project.itemByID(14).layer(1).copyToComp(c);var target=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i) instanceof ShapeLayer&&c.layer(i).id!==44&&c.layer(i).id!==47)target=c.layer(i);' if len(sys.argv)<2 else 'var target=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===51)target=c.layer(i);if(!target||!(target instanceof ShapeLayer)||target.property("ADBE Effect Parade").numProperties!==1)throw Error("Fresh target changed");'
info,immediate,messages=capture('new-immediate',guard+create+'target.name="Fresh preset target";target.adjustmentLayer=true;target.enabled=true;'+select+'target.selected=true;target.applyPreset(new File('+json.dumps(preset)+'));'+count+'var row={target:target.id,count:count,state:main.property(427).value,linked:main.property(426).value?c.layer(main.property(426).value).id:0};'+render('new-immediate')+'return JSON.stringify(row);')
target=info['target'];lookup=guard+'var target=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id==='+str(target)+')target=c.layer(i);if(!target)throw Error("Target missing");'
reference=canvas((out/'after5-reference.native.log').read_text(encoding='utf-8'))
reports=[]
def protected(kind,info,pixels,messages):
 report={'case':kind,'binding':info,'opaque_words':int(np.count_nonzero(pixels==0x3f800000)),'old_data_rejected':'coverage gate refused: E60' in messages,'status':'PASS' if 'coverage gate refused: E60' in messages and set(np.unique(pixels))=={0,0x3f800000} else 'FAIL'}
 reports.append(report);print(json.dumps(report),flush=True);(out/'ffx-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
protected('FFX new instance before idle',info,immediate,messages)
def recover(name):
 time.sleep(3)
 info,pixels,messages=capture(name,lookup+'var main=target.property("ADBE Effect Parade").property(1),reader=c.layer(main.property(426).value);var row={target:target.id,state:main.property(427).value,reader:reader.id,source:c.layer(reader.property("ADBE Effect Parade").property(1).property(1).value).id};'+render(name)+'app.project.save();return JSON.stringify(row);')
 mismatches=int(np.count_nonzero(pixels!=reference));report={'case':name,'binding':info,'native_word_mismatches':mismatches,'pixels':480000,'status':'PASS' if mismatches==0 and info['source']==target else 'FAIL'}
 reports.append(report);print(json.dumps(report),flush=True);(out/'ffx-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
recover('new-recovered')
info,immediate,messages=capture('existing-immediate',lookup+select+'target.selected=true;target.property("ADBE Effect Parade").property(1).property(3).selected=true;var previous=target.property("ADBE Effect Parade").property(1).property(427).value;target.applyPreset(new File('+json.dumps(preset)+'));'+count+'var row={target:target.id,count:count,previous:previous,state:main.property(427).value,linked:main.property(426).value?c.layer(main.property(426).value).id:0};'+render('existing-immediate')+'return JSON.stringify(row);')
protected('FFX existing instance before idle',info,immediate,messages)
recover('existing-recovered')
