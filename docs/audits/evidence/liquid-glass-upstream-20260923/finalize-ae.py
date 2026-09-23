import ast,json,sys,time
from pathlib import Path
out=Path(__file__).parent;root=out.resolve().parents[2]
sys.path.insert(0,r'E:/Code/AePlugin_Dynamicfx/spike/host-outline')
from mcp_client import Client
client=Client()
t=ast.parse((out/'ae-upstream.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'helpers','exec'))
script=(root/'examples/apply-liquid-glass.jsx').resolve().as_posix()
body='var c=app.project.itemByID(1);c.openInViewer();for(var i=1;i<=c.numLayers;i++)c.layer(i).selected=false;var card=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===21)card=c.layer(i);if(!card)throw Error("Missing card");card.selected=true;'
body+='$.evalFile(new File('+json.dumps(script)+'));$.evalFile(new File('+json.dumps(script)+'));var e=card.property("ADBE Effect Parade");return JSON.stringify({effects:e.numProperties,name:e.property(1).name,adjustment:card.adjustmentLayer,error:e.property(1).property(3).expressionError});'
r=call('apply-twice',body)
assert r['effects']==1 and r['name']=='Liquid Glass Studio' and r['adjustment'] and not r['error']
call('clean-demo','var c=app.project.itemByID(1);for(var i=c.numLayers;i>=1;i--)if(c.layer(i).id===85)c.layer(i).remove();for(var i=app.project.numItems;i>=1;i--)if(app.project.item(i).name==="Material test sampler")app.project.item(i).remove();while(app.project.renderQueue.numItems)app.project.renderQueue.item(1).remove();c.time=0;c.resolutionFactor=[1,1];app.project.bitsPerChannel=32;app.project.save();return JSON.stringify("saved");')
time.sleep(3)
r=call('final-demo-state','var c=app.project.itemByID(1),rows=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l instanceof ShapeLayer){var e=l.property("ADBE Effect Parade").property(1);rows.push({id:l.id,name:l.name,enabled:l.enabled,effects:l.property("ADBE Effect Parade").numProperties,state:e.property(427).value,token:e.property(6).value,error:e.property(3).expressionError});}}app.project.save();return JSON.stringify({version:app.version,depth:app.project.bitsPerChannel,queue:app.project.renderQueue.numItems,shapes:rows});')
assert len(r['shapes'])==3 and r['queue']==0 and all(s['enabled'] and s['effects']==1 and s['state']>=4 and s['token']%4==1 and not s['error'] for s in r['shapes'])
