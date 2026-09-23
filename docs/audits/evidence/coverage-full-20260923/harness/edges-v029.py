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
    reference=coverage.replace('input, coverage ->','input ->').replace('// @param coverage hint:coverage\n','').replace('layout(set=0,binding=3) uniform texture2D u_cov;','').replace('u_cov','u_in')
    edit('expanded','for(var n=0;n<targets.length;n++){var l=targets[n],p=l.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),s=p.value;s.vertices=[[-80,40],[880,40],[400,560]];p.setValue(s);while(l.property("ADBE Mask Parade").numProperties)l.property("ADBE Mask Parade").property(1).remove();var fx=l.property("ADBE Effect Parade");if(n===2){var e=fx.addProperty("DynamicFx");e.moveTo(1);}fx=l.property("ADBE Effect Parade");fx.property(1).property(3).expression="`"+(n===2?'+json.dumps(reference)+':'+json.dumps(coverage)+')+"`;0";fx.property(1).property(3).expressionEnabled=true;}')
    time.sleep(3)
def native(raw,depth,ds):
    matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',raw))
    if len(matches)!=1:raise RuntimeError('Native frame count '+str(len(matches)))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);assert d==depth
    values=np.array(json.loads(m[6]),dtype=np.uint32).reshape(h,w);a=np.zeros((856,1056),dtype=np.uint32)
    l,t,r,b=max(x,-128),max(y,-128),min(x+w,928),min(y+h,728)
    a[t+128:b+128,l+128:r+128]=values[t-y:b-y,l-x:r-x]
    return a,{'world':[w,h],'origin':[x,y]}
reports=[]
for depth in [8,16,32]:
    ref,rm=render('expanded-v029','reference',14,26,depth,.5,1)
    outside=ref.copy();outside[128:728,128:928]=0
    for kind,comp,owner in [('ordinary',1,13),('adjustment',30,44)]:
        a,m=render('expanded-v029',kind,comp,owner,depth,.5,1)
        delta=int(np.count_nonzero(a!=ref));row={'case':'expanded','kind':kind,'depth':depth,'mismatches':delta,'outside_reference':int(np.count_nonzero(outside)),'reference':rm,'actual':m,'status':'PASS' if delta==0 and np.any(outside) else 'FAIL'}
        reports.append(row);(out/'edges-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8');print(json.dumps(row),flush=True)
execute('expanded-save','app.project.save();return "saved";')
