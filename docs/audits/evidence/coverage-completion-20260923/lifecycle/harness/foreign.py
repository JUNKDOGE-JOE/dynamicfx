import json,sys,time
from pathlib import Path
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
out=Path(__file__).parent;c=Client();rows=[]
pre='if(!app.project.file||app.project.file.name!=="lifecycle.aep")throw Error("Unexpected project");var c=app.project.itemByID(1),o=null,r=null;for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),f=l.property("ADBE Effect Parade");if(l.id===13)o=l;if(f.numProperties&&f.property(1).matchName==="DynamicFx Coverage Reader")r=l;}'
snap='return JSON.stringify({owner:o?o.id:null,reader:r?r.id:null,state:o?o.property("ADBE Effect Parade").property(1).property(427).value:null,layers:c.numLayers,items:app.project.numItems});'
def execute(name,code):
    args={'code':'(function(){'+pre+code+'})()','timeout_sec':25};r=c.call('ae_exec',args,timeout=35)
    (out/('foreign-'+name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    return json.loads(d['content'])
def act(name,code):
    execute(name+'-action',code+'return JSON.stringify("applied");');time.sleep(2);return execute(name,snap)
def check(name,condition,state):
    row={'case':name,'status':'PASS' if condition else 'FAIL','state':state};rows.append(row)
    (out/'foreign-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(row),flush=True)
    assert condition,row
before=execute('baseline',snap);reader=before['reader'];items=before['items']
s=act('reference','var x=c.layers.addNull();x.name="Foreign reader user";var e=x.property("ADBE Effect Parade").addProperty("ADBE Layer Control");e.property(1).setValue(r.index);o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=false;')
check('foreign layer selector preserves reader after last opt-out',s['reader']==reader and s['state']==0,s)
s=act('reenable','o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=true;')
check('foreign reader remains usable without recreation',s['reader']==reader and s['state']>=4,s)
s=act('release','c.layer("Foreign reader user").remove();var x=c.layers.add(r.source);x.name="Foreign source user";x.enabled=false;o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=false;')
check('shared source preserved while owned reader removed',s['reader'] is None and s['state']==0 and s['items']==items+1,s)
s=act('fresh','c.layer("Foreign source user").remove();o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=true;')
check('new opt-in creates and validates independent reader',s['reader'] is not None and s['state']>=4,s)
s=act('modified-off','r.locked=false;r.position.setValue([435,300]);r.locked=true;o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=false;')
check('modified reader survives opt-out as conflict',s['reader'] is not None and s['state']==3,s)
s=act('modified-recover','r.locked=false;r.position.setValue([400,300]);r.locked=true;o.property("ADBE Effect Parade").property(1).property(3).expressionEnabled=true;')
check('restored structure recovers',s['state']>=4,s)
s=act('owner-delete','app.beginUndoGroup("Delete regression owner");o.remove();app.endUndoGroup();')
check('owner deletion cleans known orphan',s['owner'] is None and s['reader'] is None,s)
s=act('undo-reader','app.executeCommand(16);')
check('Undo cleanup restores orphan without deletion loop',s['owner'] is None and s['reader'] is not None,s)
s=act('undo-owner','app.executeCommand(16);')
check('Undo original restores native link',s['owner']==13 and s['reader'] is not None and s['state']>=4,s)
execute('save','app.project.save();'+snap)
