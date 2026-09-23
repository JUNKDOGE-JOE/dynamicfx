import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
guard='if(!app.project.file||app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");'
def execute(name,body,timeout=30):
    args={'code':'(function(){'+guard+body+'})()','timeout_sec':timeout};r=client.call('ae_exec',args,timeout=timeout+10)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':r},indent=2),encoding='utf-8');d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(d)
    return d['content']
def read_native(text,index):
    matches=list(re.finditer(r'SMART_ALPHA id='+str(index)+r'\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text))
    if len(matches)!=1:raise ValueError('Expected one native checkout, got '+str(len(matches)))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);pixels=np.array(json.loads(m[6]),dtype=np.uint32).reshape(h,w)
    canvas=np.zeros((600,800),dtype=np.uint32);l,t,r,b=max(x,0),max(y,0),min(x+w,800),min(y+h,600)
    if l<r and t<b:canvas[t:b,l:r]=pixels[t-y:b-y,l-x:r-x]
    return canvas,{'width':w,'height':h,'depth':d,'origin':[x,y]}
project=(out/'native-reader.aep').resolve().as_posix()
execute('reopen','app.project.save();app.open(new File('+json.dumps(project)+'));return JSON.stringify({items:app.project.numItems,layers:app.project.itemByID(910).numLayers});')
time.sleep(2)
setup='''var c=app.project.itemByID(910),owner=null;
for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");if(l.id===913)owner=l;
for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx Host Shape Probe" && !fx.property(j).property(11).value){fx.property(j).property(2).setValue(l.id===913&&j===1?21:1);fx.property(j).property(4).setValue(false);}
if(l.id===928)l.enabled=false;}
if(!owner)throw Error("Missing owner");
for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i).name==="DS_reader_frame_reference")throw Error("Already prepared");
var ref=app.project.itemByID(866).duplicate();ref.name="DS_reader_frame_reference";var rl=ref.layer(1);rl.adjustmentLayer=false;var e=rl.property("ADBE Effect Parade").property(1);e.property(2).setValue(21);e.property(4).setValue(false);e.property(10).setValue(false);e.property(6).setValue(1);e.property(9).setValue(1);
var targets=[owner,rl];for(var n=0;n<2;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),path=v.property("ADBE Vector Shape - Group").property("ADBE Vector Shape");var a=path.value,b=path.value;b.vertices=[[220,120],[580,180],[350,520]];path.setValueAtTime(0,a);path.setValueAtTime(1,b);var opacity=v.property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity");opacity.setValueAtTime(0,37.125);opacity.setValueAtTime(1,78.25);}
c.time=1.6;ref.time=1.6;app.project.save();return JSON.stringify({actual:c.id,owner:owner.id,reference:ref.id});'''
info=json.loads(execute('frame-setup',setup));(out/'frame-fixture.json').write_text(json.dumps(info,indent=2),encoding='utf-8');print(info,flush=True)
reports=[]
def render(case,depth,seconds):
    arrays=[];metadata=[]
    for kind,comp,index in [('actual',info['actual'],2),('reference',info['reference'],0)]:
        name=f'{case}-{depth}-{str(seconds).replace(".","_")}-{kind}';offset=log.stat().st_size;dest=(out/(name+'.png')).resolve().as_posix()
        execute(name,f'app.project.bitsPerChannel={depth};var c=app.project.itemByID({comp});c.time=1.6;app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng({seconds},new File('+json.dumps(dest)+'));return JSON.stringify({cti:c.time,requested:'+str(seconds)+'});')
        for _ in range(20):
            if Path(dest).exists() and Path(dest).read_bytes().endswith(b'IEND\xaeB`\x82'):break
            time.sleep(.5)
        with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
        (out/(name+'.native.log')).write_text(raw,encoding='utf-8');a,meta=read_native(raw,index);arrays.append(a);metadata.append(meta)
    changed=int(np.count_nonzero(arrays[0]!=arrays[1]));report={'case':case,'depth':depth,'requested':seconds,'cti':1.6,'pixels':480000,'native_word_mismatches':changed,'worlds':metadata,'status':'PASS' if not changed else 'FAIL'}
    reports.append(report);(out/'frame-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8');print(report,flush=True)
    return changed
for depth in (8,16,32):
    for seconds in (1,0,.5):render('animated',depth,seconds)
targets=f'var c=app.project.itemByID({info["actual"]}),a=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===913)a=c.layer(i);var targets=[a,app.project.itemByID({info["reference"]}).layer(1)];'
execute('set-transform',targets+'for(var n=0;n<2;n++){var t=targets[n].property("ADBE Transform Group");t.property("ADBE Anchor Point").setValue([400,300]);t.property("ADBE Position").setValue([420,310]);t.property("ADBE Scale").setValue([93,107]);t.property("ADBE Rotate Z").setValue(13);}return "set";')
for depth in (8,16,32):render('transform2d',depth,.5)
execute('set-3d',targets+'for(var n=0;n<2;n++){targets[n].threeDLayer=true;var t=targets[n].property("ADBE Transform Group");t.property("ADBE Anchor Point").setValue([400,300,0]);t.property("ADBE Position").setValue([400,300,0]);t.property("ADBE Scale").setValue([95,105,100]);t.property("ADBE Orientation").setValue([15,20,5]);}return "set";')
for depth in (8,16,32):render('transform3d',depth,.5)
execute('set-repeater',targets+'for(var n=0;n<2;n++){targets[n].threeDLayer=false;var t=targets[n].property("ADBE Transform Group");t.property("ADBE Anchor Point").setValue([0,0]);t.property("ADBE Position").setValue([0,0]);t.property("ADBE Scale").setValue([100,100]);t.property("ADBE Rotate Z").setValue(0);targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").addProperty("ADBE Vector Filter - Repeater");}return "set";')
for depth in (8,16,32):render('repeater',depth,.5)
execute('frame-save','app.project.save();return "saved";')
