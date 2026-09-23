import ast,json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
tree=ast.parse((out/'matrix.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'matrix-functions','exec'))
targets='var comps=[app.project.itemByID(1),app.project.itemByID(30),app.project.itemByID(14)],ids=[13,44,26],targets=[];for(var n=0;n<comps.length;n++){var found=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j).id===ids[n])found=comps[n].layer(j);if(!found)throw Error("Missing owner");targets.push(found);}'
base=(out.parent/'coverage-owner-20260923/coverage.glsl').read_text(encoding='utf-8').replace('// @param coverage hint:coverage','// @param coverage hint:coverage\n// @param padding hint:canvas min:0 max:512 default:128').replace('float u_frame;','float u_frame; float padding;')
def shader(reference,shift):
 s=base
 tex='u_in' if reference else 'u_cov';channel='a' if reference else 'r'
 if reference:s=s.replace('input, coverage ->','input ->').replace('// @param coverage hint:coverage\n','').replace('layout(set=0,binding=3) uniform texture2D u_cov;','')
 body='void main(){ivec2 size=textureSize(sampler2D('+tex+',u_s),0);ivec2 p=ivec2(v_uv*vec2(size))-ivec2('+str(shift)+',0);float a=(all(greaterThanEqual(p,ivec2(0)))&&all(lessThan(p,size)))?texelFetch(sampler2D('+tex+',u_s),p,0).'+channel+':0.0;outColor=vec4(0,0,0,a);}\n@endpass'
 return s[:s.index('void main()')]+body
reports=[]
for shift in [0,128]:
 code='for(var n=0;n<targets.length;n++){var e=targets[n].property("ADBE Effect Parade").property(1);e.property(3).expression="`"+(n===2?'+json.dumps(shader(True,shift))+':'+json.dumps(shader(False,shift))+')+"`;0";}'
 edit('fixed-shift-'+str(shift),code);time.sleep(3)
 for depth in [8,16,32]:
  ref,rm=render('fixed-shift-'+str(shift),'reference',14,26,depth,.5,1)
  border=int(np.count_nonzero(ref[:,:128]))
  for kind,comp,owner in [('ordinary',1,13),('adjustment',30,44)]:
   a,m=render('fixed-shift-'+str(shift),kind,comp,owner,depth,.5,1);delta=int(np.count_nonzero(a!=ref))
   row={'case':'fixed-shift-'+str(shift),'kind':kind,'depth':depth,'pixels':480000,'mismatches':delta,'outside_sample_nonzero':border if shift else None,'reference':rm,'actual':m,'status':'PASS' if delta==0 and (shift==0 or border>0) else 'FAIL'}
   reports.append(row);(out/'fixed-shift-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8');print(json.dumps(row),flush=True)
execute('shift-save','app.project.save();return "saved";')
