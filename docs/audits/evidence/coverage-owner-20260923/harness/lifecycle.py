import json,sys,time,shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;c=Client();results=[]
pre='if(!app.project.file||app.project.file.name!=="production-coverage.aep")throw Error("Unexpected project");var c=app.project.itemByID(1),owner=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===13)owner=c.layer(i);if(!owner)throw Error("Owner missing");'
snap='''var owners=[],readers=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade"),effects=[];for(var j=1;j<=fx.numProperties;j++){var e=fx.property(j);if(e.matchName==="DynamicFx")effects.push({index:j,enabled:e.property(3).expressionEnabled,linked:e.property(426).value?c.layer(e.property(426).value).id:0,state:e.property(427).value});if(e.matchName==="DynamicFx Coverage Reader")readers.push({id:l.id,source:e.property(1).value?c.layer(e.property(1).value).id:0,enabled:l.enabled,locked:l.locked,shy:l.shy});}if(effects.length)owners.push({id:l.id,effects:effects});}return JSON.stringify({layers:c.numLayers,items:app.project.numItems,owners:owners,readers:readers});'''
def execute(name,code):
    args={'code':'(function(){'+pre+code+'})()','timeout_sec':25}
    r=c.call('ae_exec',args,timeout=35);(out/('life-'+name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    return d['content']
def act(name,code):
    execute(name+'-action',code+'return "ok";');time.sleep(3)
    state=json.loads(execute(name,snap));print(name,json.dumps(state),flush=True);return state
def check(name,condition):
    results.append({'case':name,'status':'PASS' if condition else 'FAIL'})
    (out/'lifecycle-summary.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
    if not condition:raise AssertionError(name)
execute('save-baseline','app.project.save();return "saved";')
shutil.copyfile(out/'production-coverage.aep',out/'production-baseline.aep')
baseline=json.loads(execute('baseline',snap));items=baseline['items']
check('automatic creation',len(baseline['readers'])==1 and baseline['owners'][0]['effects'][0]['state']==1)
s=act('share','owner.property("ADBE Effect Parade").property(1).duplicate();')
check('two effects share reader',len(s['readers'])==1 and len(s['owners'][0]['effects'])==2 and all(e['state']==1 and e['linked']==s['readers'][0]['id'] for e in s['owners'][0]['effects']))
s=act('first-off','owner.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=false;')
check('first opt-out retains shared reader',len(s['readers'])==1 and s['owners'][0]['effects'][0]['state']==0 and s['owners'][0]['effects'][1]['state']==1)
s=act('last-off','owner.property("ADBE Effect Parade").property(2).property(3).expressionEnabled=false;')
check('last opt-out removes reader and unused source',len(s['readers'])==0 and s['items']==items-1)
s=act('undo-cleanup','app.executeCommand(16);')
check('Undo restores reader without immediate removal',len(s['readers'])==1 and s['items']==items)
s=act('redo-cleanup','app.executeCommand(2035);')
check('Redo removes reader and source again',len(s['readers'])==0 and s['items']==items-1)
s=act('reenable','owner.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=true;')
check('reenable rebuilds reader automatically',len(s['readers'])==1 and s['owners'][0]['effects'][0]['state']==1)
s=act('duplicate-owner','var copy=owner.duplicate();copy.name="Independent duplicate";')
check('duplicate original eventually gets separate reader',len(s['readers'])==2 and len(s['owners'])==2 and len(set(o['effects'][0]['linked'] for o in s['owners']))==2 and all(any(r['id']==o['effects'][0]['linked'] and r['source']==o['id'] for r in s['readers']) for o in s['owners']))
execute('save','app.project.save();return "saved";')
print(json.dumps(results),flush=True)
