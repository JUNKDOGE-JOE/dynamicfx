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
execute('postcancel-open','if(!app.project.file||app.project.file.name!=="cancel.aep")throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));app.project.save(new File('+json.dumps(project)+'));return "opened";',False)
time.sleep(3)
all_depths('postcancel-baseline')
execute('postcancel-save','app.project.save();return "saved";')
