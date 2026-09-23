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
execute('wgsl-open','if(!app.project.file||app.project.file.name!=="ffx.aep")throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));app.project.save(new File('+json.dumps(project)+'));return "opened";',False)
time.sleep(3)
shader='''@dynamicfx 1
@graph
pass main: input, coverage -> output
@end
@pass main
// @param coverage hint:coverage
struct FxUniforms { u_resolution: vec2<f32>, u_time: f32, u_frame: f32 }
@@group(0) @binding(0) var u_in: texture_2d<f32>;
@@group(0) @binding(1) var u_s: sampler;
@@group(0) @binding(2) var<uniform> fx: FxUniforms;
@@group(0) @binding(3) var u_cov: texture_2d<f32>;
@@fragment fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
 let p = vec2<i32>(uv * vec2<f32>(textureDimensions(u_cov)));
 return vec4<f32>(0.0,0.0,0.0,textureLoad(u_cov,p,0).r);
}
@endpass
'''
edit('wgsl','for(var n=0;n<2;n++){var e=targets[n].property("ADBE Effect Parade").property(1);e.property(2).setValue(2);e.property(3).expression="`"+'+json.dumps(shader)+'+"`;0";}')
time.sleep(3)
reports=[];references={}
for depth in [8,16,32]:check('wgsl-coverage',depth)
(out/'wgsl-coverage-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
for language,source in [(2,shader),(1,(out.parent/'coverage-owner-20260923/coverage.glsl').read_text(encoding='utf-8'))]:
    rejected=source.replace('input, coverage ->','prev, coverage ->')
    edit('temporal-'+str(language),'var e=targets[0].property("ADBE Effect Parade").property(1);e.property(2).setValue('+str(language)+');e.property(3).expression="`"+'+json.dumps(rejected)+'+"`;0";')
    time.sleep(2)
    result=json.loads(execute('temporal-'+str(language)+'-state',targets+'var e=targets[0].property("ADBE Effect Parade").property(1),r={};for(var j=1;j<=e.numProperties;j++){var p=e.property(j);if(p.name.indexOf("State Token")===0)r.token=p.value;if(p.name.indexOf("Status")===0)r.status=p.name;}return JSON.stringify(r);'))
    assert result['token']%4==2 and result['token']//4==7,result
execute('wgsl-save','app.project.save();return "saved";')
