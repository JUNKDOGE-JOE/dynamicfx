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

reports=json.loads((out/"extra-summary.json").read_text(encoding="utf-8"))
edit('camera-recovered','for(var n=0;n<targets.length;n++){targets[n].threeDLayer=true;targets[n].property("ADBE Transform Group").property("ADBE Orientation").setValue([18,23,5]);var camera=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j) instanceof CameraLayer)camera=comps[n].layer(j);if(!camera)camera=comps[n].layers.addCamera("Regression camera",[400,300]);camera.property("ADBE Transform Group").property("ADBE Position").setValue([360,270,-850]);camera.property("ADBE Transform Group").property("ADBE Anchor Point").setValue([400,300,0]);}')
all_depths('camera-3d')
execute('extra-save','app.project.save();return "saved";')
