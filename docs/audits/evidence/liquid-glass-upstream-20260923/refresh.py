import ast,json,sys,time
from pathlib import Path
out=Path(__file__).parent;root=out.resolve().parents[2]
sys.path.insert(0,r'E:/Code/AePlugin_Dynamicfx/spike/host-outline')
from mcp_client import Client
client=Client()
t=ast.parse((out/'ae-upstream.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'helpers','exec'))
source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
call('algebraic-refresh','var c=app.project.itemByID(1),ids=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l instanceof ShapeLayer){l.property("ADBE Effect Parade").property(1).property(3).expression="`"+'+json.dumps(source)+'+"`;0";ids.push(l.id);}}app.project.save();return JSON.stringify(ids);')
time.sleep(3)
call('algebraic-ready','var c=app.project.itemByID(1),r=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l instanceof ShapeLayer){var e=l.property("ADBE Effect Parade").property(1);if(e.property(3).expressionError)throw Error(e.property(3).expressionError);r.push({id:l.id,state:e.property(427).value,token:e.property(6).value});}}app.project.save();return JSON.stringify(r);')
