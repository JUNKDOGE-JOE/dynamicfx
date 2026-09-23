import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
def execute(name,code):
    args={'code':code,'timeout_sec':30};result=client.call('ae_exec',args,timeout=40)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':result},indent=2),encoding='utf-8')
    data=Client.data(result)
    if not data.get('ok'):raise RuntimeError(data)
    return data['content']
def tail(offset):
    with log.open('rb') as f:f.seek(offset);return f.read().decode('utf-8',errors='replace')
def frames(raw):
    result={}
    for m in re.finditer(r'SMART_ALPHA id=(\d+)\nworld=(\d+)x(\d+) depth=(\d+) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw):
        idx,w,h,d,x,y=map(int,m.groups()[:6]);canvas=np.zeros((600,800),dtype=np.uint32)
        canvas[y:y+h,x:x+w]=np.array(json.loads(m[7]),dtype=np.uint32).reshape(h,w);result[idx]=canvas
    return result
guard='if(!app.project.file||app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");'
project=(out/'source-matrix.aep').resolve().as_posix()
execute('v023-open','(function(){if(app.project.file && app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");if(!app.project.file)app.open(new File('+json.dumps(project)+'));return app.version;})()')
time.sleep(2)
reports=[]
for depth in (8,16,32):
    name=f'bridge-{depth}-masks';offset=log.stat().st_size
    code='(function(){'+guard+f'app.project.bitsPerChannel={depth};var c=app.project.itemByID(895),reader=c.layer(1),e=c.layer(2).property("ADBE Effect Parade").property(1);reader.enabled=false;reader.property("ADBE Effect Parade").property(1).property(2).setValue(22);e.property(2).setValue(24);e.property(4).setValue(true);e.property(5).setValue(e.property(5).value+1);return e.property(5).value;'+'})()'
    serial=int(execute(name+'-stage',code));deadline=time.monotonic()+18
    while time.monotonic()<deadline:
        raw=tail(offset)
        if f'BACKGROUND_COMPLETE comp=895 serial={serial}' in raw:break
        time.sleep(.5)
    else:raise RuntimeError('Stage request did not complete')
    (out/(name+'-stage.native.log')).write_text(raw,encoding='utf-8')
    if raw.count('after: -1, applied: 1')!=2:raise RuntimeError(raw)
    dest=(out/(name+'.png')).resolve().as_posix()
    execute(name,'(function(){'+guard+'var c=app.project.itemByID(895);c.layer(2).property("ADBE Effect Parade").property(1).property(2).setValue(21);app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return "rendered";})()')
    for _ in range(20):
        if Path(dest).exists() and Path(dest).read_bytes().endswith(b'IEND\xaeB`\x82'):break
        time.sleep(.5)
    raw=tail(offset);(out/(name+'.native.log')).write_text(raw,encoding='utf-8');actual=frames(raw)
    expected=frames((out/f'matrix-{depth}-masks-reference.native.log').read_text(encoding='utf-8'))[0]
    for idx in (1,2):
        item={'depth':depth,'stage':'masks -> all reader effects','checkout_id':idx,'pixels':480000,'native_word_mismatches':int(np.count_nonzero(actual[idx]!=expected)),'reader_video_enabled':False,'status':'PASS' if np.array_equal(actual[idx],expected) else 'FAIL'}
        reports.append(item);print(item,flush=True)
(out/'bridge-pixel-matrix.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
execute('bridge-matrix-save','(function(){'+guard+'app.project.save();return "saved";})()')
