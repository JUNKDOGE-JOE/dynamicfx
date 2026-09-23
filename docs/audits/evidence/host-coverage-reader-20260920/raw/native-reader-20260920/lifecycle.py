import json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
fixture=json.loads((out/'fixture.json').read_text());comp=fixture['comp'];owner=fixture['owner']
pre='if(!app.project.file||app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");var c=app.project.itemByID('+str(comp)+');function layer(id){for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===id)return c.layer(i);throw Error("Missing layer");}var owner=layer('+str(owner)+');'
snapshot='''var readers=[],owners=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade"),row={id:l.id,enabled:l.enabled,locked:l.locked,shy:l.shy,effects:[]};for(var j=1;j<=fx.numProperties;j++){var e=fx.property(j);if(e.matchName!=="DynamicFx Host Shape Probe")continue;var r={index:j,managed:e.property(10).value,role:e.property(11).value,mode:e.property(2).value,source:e.property(6).value?c.layer(e.property(6).value).id:0,linked:e.property(9).value?c.layer(e.property(9).value).id:0};row.effects.push(r);}if(row.effects.length){if(row.effects[0].role)readers.push(row);else owners.push(row);}}return JSON.stringify({layers:c.numLayers,items:app.project.numItems,readers:readers,owners:owners});'''
def execute(name,code):
    args={'code':'(function(){'+pre+code+'})()','timeout_sec':30};r=client.call('ae_exec',args,timeout=40)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':r},indent=2),encoding='utf-8');d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(d)
    return d['content']
def snap(name):
    d=json.loads(execute(name,snapshot));print(name,d,flush=True);return d
def act(name,code):
    offset=log.stat().st_size;execute(name+'-action',code+'return "ok";');time.sleep(3)
    with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
    (out/(name+'.native.log')).write_text(raw,encoding='utf-8');return snap(name)
results=[]
def check(name,condition):
    results.append({'case':name,'status':'PASS' if condition else 'FAIL'})
    (out/'lifecycle-summary.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
    if not condition:raise AssertionError(name)
s=snap('created');check('automatic creation',len(s['readers'])==1 and s['owners'][0]['effects'][0]['linked']==s['readers'][0]['id'] and not s['readers'][0]['enabled'] and s['readers'][0]['locked'])
offset=log.stat().st_size;dest=(out/'auto-32.png').resolve().as_posix()
execute('auto-32','app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return "rendered";')
time.sleep(1)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/'auto-32.native.log').write_text(raw,encoding='utf-8')
def native(text,index):
    m=re.search(r'SMART_ALPHA id='+str(index)+r'\nworld=(\d+)x(\d+) depth=32 origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text)
    if not m:raise ValueError('Missing native frame')
    w,h,x,y=map(int,m.groups()[:4]);a=np.zeros((600,800),dtype=np.uint32);a[y:y+h,x:x+w]=np.array(json.loads(m[5]),dtype=np.uint32).reshape(h,w);return a
ref=(out.parent/'direct-source-20260920/matrix-32-masks-reference.native.log').read_text(encoding='utf-8')
check('automatic reader native float32 alpha',np.array_equal(native(raw,2),native(ref,0)))
s=act('shared-effect','owner.property("ADBE Effect Parade").property(1).duplicate();')
check('two effects share one reader',len(s['readers'])==1 and len(s['owners'][0]['effects'])==2 and all(e['linked']==s['readers'][0]['id'] for e in s['owners'][0]['effects']))
s=act('disable-first','owner.property("ADBE Effect Parade").property(1).property(10).setValue(false);')
check('first owner release',len(s['readers'])==1 and s['owners'][0]['effects'][0]['linked']==0 and s['owners'][0]['effects'][1]['linked']==s['readers'][0]['id'])
s=act('disable-last','owner.property("ADBE Effect Parade").property(2).property(10).setValue(false);')
check('last owner cleanup including source',len(s['readers'])==0 and s['items']==fixture['items'])
s=act('undo-cleanup','app.executeCommand(16);')
check('cleanup Undo remains restored',len(s['readers'])==1 and s['items']==fixture['items']+1)
s=act('redo-cleanup','app.executeCommand(2035);')
check('cleanup Redo',len(s['readers'])==0 and s['items']==fixture['items'])
s=act('reenable','owner.property("ADBE Effect Parade").property(1).property(10).setValue(true);')
check('explicit reenable',len(s['readers'])==1)
s=act('duplicate-owner','var copy=owner.duplicate();copy.name="Renamed duplicate coverage owner";')
check('duplicate layer independent binding',len(s['readers'])==2 and len(s['owners'])==2 and len(set(o['effects'][0]['linked'] for o in s['owners']))==2 and all(any(r['id']==o['effects'][0]['linked'] and r['effects'][0]['source']==o['id'] for r in s['readers']) for o in s['owners']))
s=act('rename-owner','owner.name="Renamed original coverage owner";')
check('renaming keeps ownership',len(s['readers'])==2 and len(s['owners'])==2)
execute('save','app.project.save();return "saved";')
print(json.dumps(results),flush=True)
