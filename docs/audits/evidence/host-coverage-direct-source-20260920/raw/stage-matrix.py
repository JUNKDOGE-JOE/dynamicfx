import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent
client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
def execute(name,code):
    args={'code':code,'timeout_sec':30}
    (out/(name+'.args.json')).write_text(json.dumps(args,indent=2),encoding='utf-8')
    result=client.call('ae_exec',args,timeout=40)
    (out/(name+'.result.json')).write_text(json.dumps(result,indent=2),encoding='utf-8')
    data=Client.data(result)
    if not data.get('ok'):raise RuntimeError(data)
    return data['content']
def tail(offset):
    with log.open('rb') as f:f.seek(offset);return f.read().decode('utf-8',errors='replace')
guard='if(!app.project.file||app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");'
project=(out/'source-matrix.aep').resolve().as_posix()
execute('v021-open','(function(){if(app.project.file && app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");if(!app.project.file)app.open(new File('+json.dumps(project)+'));return app.version;})()')
setup='''(function(){GUARD
var names=["DS_stage_reference","DS_stage_external"],rows=[];
for(var n=0;n<2;n++){
 var c=null;for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i).name===names[n])c=app.project.item(i);
 if(c)throw Error("Fixture already exists; inspect before rerunning setup");
 c=app.project.itemByID(n===0?789:850).duplicate();c.name=names[n];
 var l=c.layer(n===0?1:2),fx=l.property("ADBE Effect Parade");l.adjustmentLayer=n===1;
 if(n===1){while(fx.numProperties)fx.property(1).remove();}
 else{fx.property(1).property(6).setValue(1);fx.property(1).property(9).setValue(1);}
 l.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(37.125);
 var mask=l.property("ADBE Mask Parade").addProperty("ADBE Mask Atom"),s=new Shape();
 s.vertices=[[350,230],[450,230],[450,370],[350,370]];s.inTangents=[[0,0],[0,0],[0,0],[0,0]];s.outTangents=s.inTangents;s.closed=true;
 mask.property("ADBE Mask Shape").setValue(s);mask.maskMode=MaskMode.SUBTRACT;
 mask.property("ADBE Mask Feather").setValue([11.25,7.75]);mask.property("ADBE Mask Offset").setValue(4.25);mask.property("ADBE Mask Opacity").setValue(68.75);
 var e=c.layer(1).property("ADBE Effect Parade").property(1);e.property(4).setValue(true);e.property(2).setValue(21);
 if(n===1){e.property(6).setValue(2);e.property(9).setValue(2);}
 rows.push({comp:c.id,target:l.id,reader:c.layer(1).id});
}
app.project.save();return JSON.stringify(rows);})()'''.replace('GUARD',guard)
rows=json.loads(execute('stage-matrix-setup',setup));print(rows,flush=True)
time.sleep(2)
reports=[]
for depth in (8,16,32):
 for stage,mode in [('source',13),('masks',14)]:
    name=f'matrix-{depth}-{stage}'
    ref,external=[r['comp'] for r in rows]
    offset=log.stat().st_size
    command='(function(){'+guard+f'app.project.bitsPerChannel={depth};var c=app.project.itemByID({ref}),e=app.project.itemByID({external}).layer(1).property("ADBE Effect Parade").property(1);c.layer(1).property("ADBE Mask Parade").property(1).maskMode=MaskMode.'+('NONE' if stage=='source' else 'SUBTRACT')+f';e.property(2).setValue({mode});e.property(5).setValue(e.property(5).value+1);return e.property(5).value;'+'})()'
    serial=int(execute(name+'-stage',command))
    deadline=time.monotonic()+18
    while time.monotonic()<deadline:
        raw=tail(offset)
        if f'BACKGROUND_COMPLETE comp={external} serial={serial}' in raw:break
        time.sleep(.5)
    else:raise RuntimeError('Missing stage completion')
    (out/(name+'-stage.native.log')).write_text(raw,encoding='utf-8')
    if 'error=0' not in raw or 'error=3' in raw:raise RuntimeError('Stage setter failure')
    arrays=[]
    for kind,comp in [('reference',ref),('external',external)]:
        start=log.stat().st_size
        dest=(out/(name+'-'+kind+'.png')).resolve().as_posix()
        code='(function(){'+guard+f'var c=app.project.itemByID({comp});c.layer(1).property("ADBE Effect Parade").property(1).property(2).setValue(21);app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return "rendered";})()'
        execute(name+'-'+kind,code)
        for _ in range(20):
            if Path(dest).exists() and Path(dest).read_bytes().endswith(b'IEND\xaeB`\x82'):break
            time.sleep(.5)
        raw=tail(start);(out/(name+'-'+kind+'.native.log')).write_text(raw,encoding='utf-8')
        frames={}
        for m in re.finditer(r'SMART_ALPHA id=(\d+)\nworld=(\d+)x(\d+) depth=(\d+) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw):
            idx,w,h,d,x,y=map(int,m.groups()[:6]);words=np.array(json.loads(m[7]),dtype=np.uint32).reshape(h,w)
            canvas=np.zeros((600,800),dtype=np.uint32)
            if not (d==depth and x>=0 and y>=0 and x+w<=800 and y+h<=600):raise RuntimeError('Unexpected frame metadata')
            canvas[y:y+h,x:x+w]=words;frames[idx]=canvas
        arrays.append(frames)
    for idx in (1,2):
        expected=arrays[0][0];actual=arrays[1][idx]
        changed=int(np.count_nonzero(expected!=actual))
        item={'depth':depth,'stage':stage,'checkout_id':idx,'pixels':480000,'native_word_mismatches':changed,'status':'PASS' if changed==0 else 'FAIL'}
        reports.append(item);print(item,flush=True)
(out/'stage-pixel-matrix.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
execute('stage-matrix-save','(function(){'+guard+'app.project.save();return "saved";})()')
