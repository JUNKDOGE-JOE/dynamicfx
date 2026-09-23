import ast,json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix();project=(out/'full.aep').resolve().as_posix()
tree=ast.parse((out/'matrix.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'matrix-functions','exec'))
targets='var comps=[app.project.itemByID(1),app.project.itemByID(30),app.project.itemByID(14)],ids=[13,44,26],targets=[];for(var n=0;n<comps.length;n++){var found=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j).id===ids[n])found=comps[n].layer(j);if(!found)throw Error("Missing owner");targets.push(found);}'
reports=[];references={}
def all_depths(name,seconds=.5):
    for d in [8,16,32]:check(name,d,seconds)
    (out/'extra-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
reset('modifiers-extra')
edit('open-stroke','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),p=v.property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),s=p.value;s.closed=false;p.setValue(s);v.property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(0);var t=v.addProperty("ADBE Vector Graphic - Stroke");t.property("ADBE Vector Stroke Width").setValue(23.5);t.property("ADBE Vector Stroke Opacity").setValue(61.25);}')
all_depths('open-stroke')
edit('trim','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),t=v.addProperty("ADBE Vector Filter - Trim");t.property("ADBE Vector Trim Start").setValue(12);t.property("ADBE Vector Trim End").setValue(71);t.property("ADBE Vector Trim Offset").setValue(35);}')
all_depths('trim')
reset('offset')
edit('offset','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),t=v.addProperty("ADBE Vector Filter - Offset");t.property("ADBE Vector Offset Amount").setValue(31);t.moveTo(2);}')
all_depths('offset')
edit('round','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),t=v.addProperty("ADBE Vector Filter - RC");t.property("ADBE Vector RoundCorner Radius").setValue(38);t.moveTo(2);}')
all_depths('round-corners')
reset('disjoint')
edit('disjoint','for(var n=0;n<targets.length;n++){var v=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),t=v.addProperty("ADBE Vector Shape - Ellipse");t.property("ADBE Vector Ellipse Size").setValue([83,71]);t.property("ADBE Vector Ellipse Position").setValue([100,450]);t.moveTo(2);}')
all_depths('disjoint')
reset('mask-animation')
edit('mask-animation','for(var n=0;n<targets.length;n++){var m=targets[n].property("ADBE Mask Parade").property(1);m.property("ADBE Mask Feather").setValueAtTime(0,[2.5,6.75]);m.property("ADBE Mask Feather").setValueAtTime(1,[21.25,17.5]);m.property("ADBE Mask Offset").setValueAtTime(0,-8.25);m.property("ADBE Mask Offset").setValueAtTime(1,15.75);m.property("ADBE Mask Opacity").setValueAtTime(0,39.5);m.property("ADBE Mask Opacity").setValueAtTime(1,86.25);}')
for t in [1,0,.32]:all_depths('mask-animation',t)
reset('camera')
edit('camera','for(var n=0;n<targets.length;n++){targets[n].threeDLayer=true;targets[n].property("ADBE Transform Group").property("ADBE Orientation").setValue([18,23,5]);var camera=comps[n].layers.addCamera("Regression camera",[400,300]);camera.property("ADBE Transform Group").property("ADBE Position").setValue([360,270,-850]);camera.property("ADBE Transform Group").property("ADBE Point of Interest").setValue([400,300,0]);}')
all_depths('camera-3d')
execute('extra-save','app.project.save();return "saved";')
