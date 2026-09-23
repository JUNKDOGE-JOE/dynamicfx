import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix()
project=(out/'full.aep').resolve().as_posix()
def execute(name,code,guard=True):
    prefix='if(!app.project.file||app.project.file.name!=="full.aep")throw Error("Unexpected project");' if guard else ''
    args={'code':'(function(){'+prefix+code+'})()','timeout_sec':25}
    result=client.call('ae_exec',args,timeout=35)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':result},ensure_ascii=False,indent=2),encoding='utf-8')
    data=Client.data(result)
    if not data.get('ok'):raise RuntimeError(json.dumps(data,ensure_ascii=True))
    return data['content']
targets='var comps=[app.project.itemByID(1),app.project.itemByID(30),app.project.itemByID(14)],ids=[13,44,26],targets=[];for(var n=0;n<comps.length;n++){var found=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j).id===ids[n])found=comps[n].layer(j);if(!found)throw Error("Missing owner");targets.push(found);}'
def reset(name):
    execute(name+'-reset','if(!app.project.file||!(/^(production-coverage|full)\\.aep$/.test(app.project.file.name)))throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));'+targets+'for(var n=0;n<targets.length;n++){var fx=targets[n].property("ADBE Effect Parade");for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx Host Shape Probe"){fx.property(j).property(2).setValue(1);fx.property(j).property(6).setValue(0);fx.property(j).property(9).setValue(0);}}app.project.save(new File('+json.dumps(project)+'));return "reset";',False)
    time.sleep(3)
def edit(name,code):execute(name+'-setup',targets+code+'return "configured";')
def native(raw,depth,ds):
    matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw))
    if len(matches)!=1:raise RuntimeError('Expected one native frame, got '+str(len(matches)))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);assert d==depth
    width,height=(800+ds-1)//ds,(600+ds-1)//ds
    values=np.array(json.loads(m[6]),dtype=np.uint32).reshape(h,w);result=np.zeros((height,width),dtype=np.uint32)
    l,t,r,b=max(x,0),max(y,0),min(x+w,width),min(y+h,height)
    if l<r and t<b:result[t:b,l:r]=values[t-y:b-y,l-x:r-x]
    return result,{'world':[w,h],'origin':[x,y],'depth':d,'canvas':[width,height]}
def render(case,kind,comp,owner,depth,seconds,ds):
    name=f'{case}-{kind}-{depth}-{seconds}-{ds}';offset=log.stat().st_size
    dest=(out/(name+'.png')).resolve()
    code=targets+'for(var n=0;n<targets.length;n++){comps[n].resolutionFactor=['+str(ds)+','+str(ds)+'];comps[n].time=1.6;var fx=targets[n].property("ADBE Effect Parade");for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx Host Shape Probe")fx.property(j).property(2).setValue(targets[n].id==='+str(owner)+'?21:1);}app.project.bitsPerChannel='+str(depth)+';app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID('+str(comp)+').saveFrameToPng('+str(seconds)+',new File('+json.dumps(dest.as_posix())+'));return JSON.stringify({cti:app.project.itemByID('+str(comp)+').time,requested:'+str(seconds)+',resolution:app.project.itemByID('+str(comp)+').resolutionFactor});'
    execute(name,code)
    for _ in range(40):
        if dest.exists() and dest.read_bytes().endswith(b'IEND\xaeB`\x82'):break
        time.sleep(.25)
    with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
    (out/(name+'.native.log')).write_text(raw,encoding='utf-8')
    return native(raw,depth,ds)
reports=[];references={}
def check(case,depth,seconds=.5,ds=1,previous=None,allow_empty=False):
    reference,ref_meta=render(case,'reference',14,26,depth,seconds,ds)
    delta=None
    if previous:
        before=references[(previous,depth,seconds,ds)];delta=int(np.count_nonzero(reference!=before))
        if delta==0:raise AssertionError('No actual reference change: '+case)
    references[(case,depth,seconds,ds)]=reference
    for kind,comp,owner in [('ordinary',1,13),('adjustment',30,44)]:
        actual,meta=render(case,kind,comp,owner,depth,seconds,ds)
        count=int(np.count_nonzero(actual!=reference))
        row={'case':case,'kind':kind,'depth':depth,'time':seconds,'downsample':ds,'pixels':int(reference.size),'nonzero_reference':int(np.count_nonzero(reference)),'reference_delta':delta,'mismatches':count,'reference':ref_meta,'actual':meta,'status':'PASS' if count==0 and (allow_empty or np.any(reference)) else 'FAIL'}
        reports.append(row);(out/'summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8');print(json.dumps(row),flush=True)

reset('animated')
edit('animated','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),p=v.property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),a=p.value,b=p.value;b.vertices=[[220,120],[580,180],[350,520]];b.inTangents=[[-20,15],[-30,-20],[20,20]];b.outTangents=[[35,-20],[20,30],[-20,-25]];p.setValueAtTime(0,a);p.setValueAtTime(1,b);var f=v.property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity");f.setValueAtTime(0,37.125);f.setValueAtTime(1,78.25);}')
for depth in [8,16,32]:
    for seconds in [1,0,.5,.36]:check('animated',depth,seconds)
edit('transform2d','for(var n=0;n<targets.length;n++){var t=targets[n].property("ADBE Transform Group");t.property("ADBE Anchor Point").setValue([400,300]);t.property("ADBE Position").setValue([420,310]);t.property("ADBE Scale").setValue([93,107]);t.property("ADBE Rotate Z").setValue(13);}')
for depth in [8,16,32]:check('transform2d',depth)
edit('transform3d','for(var n=0;n<targets.length;n++){targets[n].threeDLayer=true;var t=targets[n].property("ADBE Transform Group");t.property("ADBE Anchor Point").setValue([400,300,0]);t.property("ADBE Position").setValue([400,300,0]);t.property("ADBE Scale").setValue([95,105,100]);t.property("ADBE Orientation").setValue([15,20,5]);}')
for depth in [8,16,32]:check('transform3d',depth)
reset('modifiers')
for depth in [8,16,32]:check('plain',depth)
edit('repeater','for(var n=0;n<targets.length;n++){var r=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").addProperty("ADBE Vector Filter - Repeater");r.property("ADBE Vector Repeater Copies").setValue(3);var t=r.property("ADBE Vector Repeater Transform");t.property("ADBE Vector Repeater Position").setValue([60,15]);t.property("ADBE Vector Repeater Rotation").setValue(-5);t.property("ADBE Vector Repeater Scale").setValue([92,104]);t.property("ADBE Vector Repeater Opacity 2").setValue(60);}')
for depth in [8,16,32]:check('repeater',depth,previous='plain')
edit('merge','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),e=v.addProperty("ADBE Vector Shape - Ellipse");e.property("ADBE Vector Ellipse Size").setValue([100,60]);e.property("ADBE Vector Ellipse Position").setValue([400,270]);e.moveTo(2);var m=v.addProperty("ADBE Vector Filter - Merge");m.property("ADBE Vector Merge Type").setValue(3);m.moveTo(3);}')
for depth in [8,16,32]:check('merge',depth,previous='repeater')
edit('stroke','for(var n=0;n<targets.length;n++){var s=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").addProperty("ADBE Vector Graphic - Stroke");s.property("ADBE Vector Stroke Width").setValue(12.5);s.property("ADBE Vector Stroke Opacity").setValue(77);}')
for depth in [8,16,32]:check('stroke',depth,previous='merge')
for ds in [2,4]:
    for depth in [8,16,32]:check('downsample',depth,ds=ds)
edit('par','for(var n=0;n<comps.length;n++)comps[n].pixelAspect=1.333333;')
for depth in [8,16,32]:check('par',depth)
execute('save','app.project.save();return "saved";')
