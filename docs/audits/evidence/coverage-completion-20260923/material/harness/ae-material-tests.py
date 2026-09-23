import ast,json,sys,time
from pathlib import Path
import numpy as np
out=Path(__file__).parent;root=out.resolve().parents[2]
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
client=Client();project=(out/'Liquid Glass.aep').resolve().as_posix()
t=ast.parse((out/'ae-demo.py').read_text(encoding='utf-8'));exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'demo-functions','exec'))
helpers='''var c=app.project.itemByID(1),card=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===21)card=c.layer(i);if(!card)throw Error("Missing card");
function sample(point){var w=null;for(var j=1;j<=app.project.numItems;j++)if(app.project.item(j).name==="Material test sampler")w=app.project.item(j);if(!w){w=app.project.items.addComp("Material test sampler",1440,900,1,6,25);w.layers.add(c).name="scene";var n=w.layers.addNull();n.name="probe";n.property("ADBE Effect Parade").addProperty("ADBE Slider Control");}var p=w.layer("probe").property("ADBE Effect Parade").property(1).property(1),a=[];for(var k=0;k<4;k++){p.expression='thisComp.layer("scene").sampleImage(['+point[0]+','+point[1]+'],[0.49,0.49],true,0)['+k+']';a.push(p.value);if(p.expressionError)throw Error(p.expressionError);}return a;}
'''
rows=[]
def record(name,passed,**extra):
    row={'case':name,'status':'PASS' if passed else 'FAIL',**extra};rows.append(row);(out/'material-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(row),flush=True);assert passed,row
full=call('material-full',helpers+'app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);return JSON.stringify(sample([243.5,450.5]));')
base=call('material-disabled',helpers+'card.enabled=false;app.purge(PurgeTarget.ALL_CACHES);var value=sample([243.5,450.5]);card.enabled=true;return JSON.stringify(value);')
semi=call('material-semi',helpers+'card.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(37.125);app.purge(PurgeTarget.ALL_CACHES);return JSON.stringify(sample([243.5,450.5]));')
expected=np.array(base)+(np.array(full)-base)*.37125;error=float(np.max(np.abs(np.array(semi)-expected)))
record('partial alpha is applied once by AE',error<.002,full=full,background=base,partial=semi,maximum_error=error)
call('material-full-restored',helpers+'card.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(100);return JSON.stringify("restored");')
points=[[70.5,200.5],[1300.5,820.5]]
on=call('material-outside',helpers+'return JSON.stringify([sample([70.5,200.5]),sample([1300.5,820.5])]);')
off=call('material-outside-disabled',helpers+'var ls=[];for(var i=1;i<=c.numLayers;i++)if(c.layer(i) instanceof ShapeLayer){ls.push(c.layer(i));c.layer(i).enabled=false;}app.purge(PurgeTarget.ALL_CACHES);var value=[sample([70.5,200.5]),sample([1300.5,820.5])];for(var i=0;i<ls.length;i++)ls[i].enabled=true;return JSON.stringify(value);')
record('exterior pixels unchanged',np.array_equal(on,off),points=points,actual=on,reference=off)
hole=call('material-hole',helpers+'''var mask=card.property("ADBE Mask Parade").addProperty("ADBE Mask Atom");mask.name="Regression cutout";var s=new Shape();s.vertices=[[650,400],[750,400],[750,500],[650,500]];s.closed=true;mask.property("ADBE Mask Shape").setValue(s);mask.maskMode=MaskMode.SUBTRACT;app.purge(PurgeTarget.ALL_CACHES);return JSON.stringify(sample([700.5,450.5]));''')
hole_ref=call('material-hole-disabled',helpers+'card.enabled=false;app.purge(PurgeTarget.ALL_CACHES);var value=sample([700.5,450.5]);card.enabled=true;return JSON.stringify(value);')
record('cutout interior unchanged',np.array_equal(hole,hole_ref),actual=hole,reference=hole_ref)
call('material-hole-restored',helpers+'card.property("ADBE Mask Parade").property("Regression cutout").remove();card.name="Glass - native rounded card";card.property("ADBE Root Vectors Group").property(1).name="Editable native geometry";app.project.save();return JSON.stringify("restored and renamed");')
preset=(root/'examples/liquid-glass.ffx').resolve().as_posix()
call('material-preset',helpers+'for(var i=1;i<=c.numLayers;i++)c.layer(i).selected=false;var ps=c.selectedProperties;for(var i=ps.length-1;i>=0;i--)try{ps[i].selected=false;}catch(ignore){}card.selected=true;var e=card.property("ADBE Effect Parade").property(1);e.name="Liquid Glass";e.selected=true;card.savePreset(new File('+json.dumps(preset)+'));return JSON.stringify({bytes:File('+json.dumps(preset)+').length});')
copy=call('material-ffx-new',helpers+'var l=c.layers.addShape();l.name="Preset - arbitrary Bezier";l.adjustmentLayer=true;l.property("ADBE Transform Group").property("ADBE Position").setValue([0,0]);var g=l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");g.name="Unrelated group name";var v=g.property("ADBE Vectors Group"),p=v.addProperty("ADBE Vector Shape - Group").property("ADBE Vector Shape"),s=new Shape();s.vertices=[[80,250],[500,260],[330,620],[100,550]];s.inTangents=[[-20,70],[-100,-90],[80,-40],[-10,30]];s.outTangents=[[100,-50],[20,90],[-90,40],[-30,-50]];s.closed=true;p.setValue(s);v.addProperty("ADBE Vector Graphic - Fill");for(var i=1;i<=c.numLayers;i++)c.layer(i).selected=false;l.selected=true;l.applyPreset(new File('+json.dumps(preset)+'));return JSON.stringify({id:l.id});')
time.sleep(3)
state=call('material-ffx-ready','var c=app.project.itemByID(1),l=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id==='+str(copy['id'])+')l=c.layer(i);var e=l.property("ADBE Effect Parade").property(1),r=c.layer(e.property(426).value);return JSON.stringify({effects:l.property("ADBE Effect Parade").numProperties,state:e.property(427).value,owner:c.layer(r.property("ADBE Effect Parade").property(1).property(1).value).id});')
record('FFX adapts to renamed arbitrary Bezier',state['effects']==1 and state['state']>=4 and state['owner']==copy['id'],binding=state)
call('material-ffx-hidden','var c=app.project.itemByID(1);for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id==='+str(copy['id'])+')c.layer(i).enabled=false;app.project.save();return JSON.stringify("saved");')
