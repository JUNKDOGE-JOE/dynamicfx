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
if len(sys.argv)<2:
    reset('expanded')
    coverage=(out.parent/'coverage-owner-20260923/coverage.glsl').read_text(encoding='utf-8')
    coverage=coverage.replace('// @param coverage hint:coverage','// @param coverage hint:coverage\n// @param padding hint:canvas min:0 max:512 default:128').replace('float u_frame;','float u_frame; float padding;')
    reference=coverage.replace('input, coverage ->','input ->').replace('// @param coverage hint:coverage\n','').replace('layout(set=0,binding=3) uniform texture2D u_cov;','').replace('u_cov','u_in').replace(').r',').a')
    edit('expanded','for(var n=0;n<targets.length;n++){var l=targets[n],p=l.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),s=p.value;s.vertices=[[-80,40],[880,40],[400,560]];p.setValue(s);while(l.property("ADBE Mask Parade").numProperties)l.property("ADBE Mask Parade").property(1).remove();var fx=l.property("ADBE Effect Parade");if(n===2){var e=fx.addProperty("DynamicFx");e.moveTo(1);}fx=l.property("ADBE Effect Parade");fx.property(1).property(3).expression="`"+(n===2?'+json.dumps(reference)+':'+json.dumps(coverage)+')+"`;0";fx.property(1).property(3).expressionEnabled=true;}')
    time.sleep(3)