import ast,json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
guard='if(!app.project.file||app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");'
info=json.loads((out/'frame-fixture.json').read_text());reports=json.loads((out/'frame-summary.json').read_text())
module=ast.parse((out/'frames.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in module.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'frames-functions','exec'))
targets=f'var c=app.project.itemByID({info["actual"]}),a=null;for(var i=1;i<=c.numLayers;i++)if(c.layer(i).id===913)a=c.layer(i);var targets=[a,app.project.itemByID({info["reference"]}).layer(1)];'
execute('configure-repeater3',targets+'for(var n=0;n<2;n++){var r=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Filter - Repeater");r.property("ADBE Vector Repeater Copies").setValue(3);var t=r.property("ADBE Vector Repeater Transform");t.property("ADBE Vector Repeater Position").setValue([60,15]);t.property("ADBE Vector Repeater Rotation").setValue(-5);t.property("ADBE Vector Repeater Scale").setValue([92,104]);t.property("ADBE Vector Repeater Opacity 2").setValue(60);}return "set";')
for depth in (8,16,32):render('repeater3',depth,.5)
execute('configure-merge-hole',targets+'for(var n=0;n<2;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group");var ellipse=v.addProperty("ADBE Vector Shape - Ellipse");ellipse.property("ADBE Vector Ellipse Size").setValue([100,60]);ellipse.property("ADBE Vector Ellipse Position").setValue([400,270]);ellipse.moveTo(2);var merge=v.addProperty("ADBE Vector Filter - Merge");merge.property("ADBE Vector Merge Type").setValue(3);merge.moveTo(3);}return "set";')
for depth in (8,16,32):render('merge-hole-repeater3',depth,.5)
execute('configure-stroke',targets+'for(var n=0;n<2;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),stroke=v.addProperty("ADBE Vector Graphic - Stroke");stroke.property("ADBE Vector Stroke Width").setValue(12.5);stroke.property("ADBE Vector Stroke Opacity").setValue(77);}return "set";')
for depth in (8,16,32):render('stroke-merge-repeater3',depth,.5)
execute('modifiers-save','app.project.save();return "saved";')
