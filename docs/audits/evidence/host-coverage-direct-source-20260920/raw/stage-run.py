import argparse,json,os,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
p=argparse.ArgumentParser()
p.add_argument('name');p.add_argument('comp');p.add_argument('mode',type=int)
p.add_argument('--adjustment',choices=['true','false']);p.add_argument('--render',action='store_true')
a=p.parse_args();out=Path(__file__).parent
log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
start=log.stat().st_size if log.exists() else 0
code='''(function(){
if(!app.project.file||app.project.file.name!=="source-matrix.aep")throw Error("Unexpected project");
var c=null;for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i).name===COMP)c=app.project.item(i);
if(!c)throw Error("Missing comp");var l=c.layer(1),e=l.property("ADBE Effect Parade").property(1);
ADJUST
e.property(2).setValue(MODE);e.property(5).setValue(e.property(5).value+1);
return JSON.stringify({comp:c.id,layer:l.id,adjustment:l.adjustmentLayer,serial:e.property(5).value});
})()'''.replace('COMP',json.dumps(a.comp)).replace('MODE',str(a.mode)).replace('ADJUST','l.adjustmentLayer='+a.adjustment+';' if a.adjustment else '')
client=Client(); args={'code':code,'timeout_sec':20}
(out/(a.name+'.args.json')).write_text(json.dumps(args,indent=2),encoding='utf-8')
r=client.call('ae_exec',args,timeout=30)
(out/(a.name+'.result.json')).write_text(json.dumps(r,indent=2),encoding='utf-8')
d=Client.data(r)
if not d.get('ok'):raise RuntimeError(d)
info=json.loads(d['content']);print(info,flush=True)
deadline=time.monotonic()+18
while time.monotonic()<deadline:
    with log.open('rb') as f:f.seek(start);raw=f.read().decode('utf-8',errors='replace')
    if f'BACKGROUND_COMPLETE comp={info["comp"]} serial={info["serial"]}' in raw or f'BACKGROUND_FAILED comp={info["comp"]}' in raw:break
    time.sleep(.5)
else:raise RuntimeError('No native completion; do not replay')
if a.render:
    dest=(out/(a.name+'.png')).resolve().as_posix()
    render='(function(){var c=app.project.itemByID('+str(info['comp'])+');app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return "rendered";})()'
    rr=client.call('ae_exec',{'code':render,'timeout_sec':30},timeout=40)
    (out/(a.name+'.render.json')).write_text(json.dumps(rr,indent=2),encoding='utf-8')
    if not Client.data(rr).get('ok'):raise RuntimeError(Client.data(rr))
    for _ in range(20):
        path=Path(dest)
        if path.exists() and path.read_bytes().endswith(b'IEND\xaeB`\x82'):break
        time.sleep(.5)
with log.open('rb') as f:f.seek(start);raw=f.read().decode('utf-8',errors='replace')
(out/(a.name+'.native.log')).write_text(raw,encoding='utf-8')
print(raw[-10000:])

