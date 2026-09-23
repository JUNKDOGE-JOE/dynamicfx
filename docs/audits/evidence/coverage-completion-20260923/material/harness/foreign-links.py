import ast,json,sys,time
from pathlib import Path
out=Path(__file__).parent;root=out.resolve().parents[2]
sys.path.insert(0,r'<repository>/spike/host-outline');from mcp_client import Client
client=Client();t=ast.parse((out/'ae-demo.py').read_text(encoding='utf-8'));exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'demo-functions','exec'))
source=(root/'scripts/out/coverage-owner-20260923/coverage.glsl').read_text(encoding='utf-8')
created=call('links-setup','var c=app.project.items.addComp("Foreign link regression",320,240,1,2,25),o=c.layers.addShape();o.name="Owner";var v=o.property("ADBE Root Vectors Group"),g=v.addProperty("ADBE Vector Group").property("ADBE Vectors Group");g.addProperty("ADBE Vector Shape - Rect");g.addProperty("ADBE Vector Graphic - Fill");var e=o.property("ADBE Effect Parade").addProperty("DynamicFx");e.property(3).expression="`"+'+json.dumps(source)+'+"`;0";var child=c.layers.addSolid([1,1,1],"Foreign child",320,240,1,2);child.enabled=false;return JSON.stringify({comp:c.id,owner:o.id,child:child.id});')
time.sleep(3)
prefix='var c=app.project.itemByID('+str(created['comp'])+'),o=null,child=null;for(var i=1;i<=c.numLayers;i++){if(c.layer(i).id==='+str(created['owner'])+')o=c.layer(i);if(c.layer(i).id==='+str(created['child'])+')child=c.layer(i);}var e=o.property("ADBE Effect Parade").property(1),r=null;for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");if(fx.numProperties&&fx.property(1).matchName==="DynamicFx Coverage Reader")r=l;}'
snap='return JSON.stringify({reader:r?r.id:null,state:e.property(427).value,parent:child.parent?child.parent.id:null,matte:child.trackMatteLayer?child.trackMatteLayer.id:null});'
baseline=call('links-baseline',prefix+snap);assert baseline['state']>=4
rows=[]
for kind,action,restore in [('parent','child.parent=r;','child.parent=null;'),('matte','child.setTrackMatte(r,TrackMatteType.ALPHA);','child.removeTrackMatte();')]:
    call('links-'+kind+'-off',prefix+action+'e.property(3).expressionEnabled=false;'+snap);time.sleep(2)
    state=call('links-'+kind+'-preserved',prefix+snap)
    row={'case':kind+' reference preserves helper','state':state,'status':'PASS' if state['reader']==baseline['reader'] and state['state']==0 else 'FAIL'};rows.append(row);assert row['status']=='PASS',row
    call('links-'+kind+'-restore',prefix+restore+'e.property(3).expressionEnabled=true;'+snap);time.sleep(2)
    assert call('links-'+kind+'-ready',prefix+snap)['state']>=4
call('links-owned-matte',prefix+'r.locked=false;r.setTrackMatte(child,TrackMatteType.ALPHA);r.locked=true;'+snap);time.sleep(2)
state=call('links-owned-matte-conflict',prefix+snap);rows.append({'case':'modified helper with matte rejected','state':state,'status':'PASS' if state['state']==3 else 'FAIL'});assert state['state']==3
call('links-owned-matte-restored',prefix+'r.locked=false;r.removeTrackMatte();r.locked=true;'+snap);time.sleep(2)
call('links-cleanup',prefix+'e.property(3).expressionEnabled=false;'+snap);time.sleep(2)
state=call('links-cleaned',prefix+snap);rows.append({'case':'unreferenced helper removed','state':state,'status':'PASS' if state['reader'] is None else 'FAIL'});assert state['reader'] is None
call('links-fixture-removed','app.project.itemByID('+str(created['comp'])+').remove();app.project.save();return JSON.stringify("removed test comp");')
(out/'foreign-links-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(rows),flush=True)
