import json, os, re, sys, time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent
client=Client()
log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
guard='if(!app.project.file||app.project.file.name!=="production-coverage.aep")throw Error("Unexpected project");'
def execute(name,code):
    args={'code':'(function(){'+guard+code+'})()','timeout_sec':25}
    result=client.call('ae_exec',args,timeout=35)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':result},ensure_ascii=False,indent=2),encoding='utf-8')
    data=Client.data(result)
    if not data.get('ok'):raise RuntimeError(json.dumps(data,ensure_ascii=True))
    return data['content']
def array(raw,index):
    matches=list(re.finditer(r'SMART_ALPHA id='+str(index)+r'\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw))
    if len(matches)!=1:raise ValueError('Expected one native frame, got '+str(len(matches)))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);values=np.array(json.loads(m[6]),dtype=np.uint32).reshape(h,w)
    canvas=np.zeros((600,800),dtype=np.uint32);l,t,r,b=max(x,0),max(y,0),min(x+w,800),min(y+h,600)
    if l<r and t<b:canvas[t:b,l:r]=values[t-y:b-y,l-x:r-x]
    return canvas,{'width':w,'height':h,'depth':d,'origin':[x,y]}
def render(name,comp,depth,index=0):
    offset=log.stat().st_size
    dest=(out/(name+'.png')).resolve()
    execute(name,'app.project.bitsPerChannel='+str(depth)+';var c=app.project.itemByID('+str(comp)+');app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest.as_posix())+'));return "render requested";')
    for _ in range(40):
        if dest.exists() and dest.read_bytes().endswith(b'IEND\xaeB`\x82'):break
        time.sleep(.25)
    with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
    (out/(name+'.native.log')).write_text(raw,encoding='utf-8')
    if not dest.exists():raise RuntimeError('PNG missing: '+name)
    return array(raw,index)
reports=[]
for depth in map(int,sys.argv[1:] or ['32']):
    a,am=render('reference-'+str(depth),14,depth)
    for kind,comp in [('production',1),('adjustment',30)]:
        b,bm=render(kind+'-'+str(depth),comp,depth)
        count=int(np.count_nonzero(a!=b))
        report={'kind':kind,'depth':depth,'pixels':480000,'nonzero_reference':int(np.count_nonzero(a)),'native_word_mismatches':count,'reference':am,'production':bm,'status':'PASS' if count==0 and np.any(a) else 'FAIL'}
        reports.append(report)
        (out/('matrix-'+kind+'-'+str(depth)+'.json')).write_text(json.dumps(report,indent=2),encoding='utf-8')
        print(json.dumps(report),flush=True)
