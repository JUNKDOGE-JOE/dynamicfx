import json,subprocess
from pathlib import Path
import numpy as np
from PIL import Image
root=Path(__file__).resolve().parents[3];out=Path(__file__).parent
binary=root/'scripts/quality/target/release/dynamicfx-quality.exe';source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
background=np.fromfile(out/'background.f32',dtype='<f4').reshape(400,640,4)
coverage=np.fromfile(out/'coverage.f32',dtype='<f4').reshape(400,640,4)
rows=[]
def run(name,shader,w,h,depth,external,bg):
    src=out/(name+'.glsl');src.write_text(shader,encoding='utf-8');dest=out/(name+'.f32');manifest=out/(name+'.json')
    manifest.write_text(json.dumps({'host_alpha':{'width':w,'height':h,'path':str(external.resolve())}},indent=2),encoding='utf-8')
    args=[str(binary),str(src),str(dest),str(w),str(h),'640','400','0',str(depth),'rgba:'+str(bg),str(manifest)]
    p=subprocess.run(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (out/(name+'.stdout.log')).write_bytes(p.stdout);(out/(name+'.stderr.log')).write_bytes(p.stderr)
    if p.returncode:raise RuntimeError(p.stderr.decode(errors='replace'))
    return np.fromfile(dest,dtype='<f4').reshape(h,w,4)
for ds in [1,2,4]:
    w,h=640//ds,400//ds
    cov=coverage.reshape(h,ds,w,ds,4).mean(axis=(1,3));bg=background.reshape(h,ds,w,ds,4).mean(axis=(1,3))
    cf=out/f'coverage-{ds}.f32';bf=out/f'background-{ds}.f32';cov.astype('<f4').tofile(cf);bg.astype('<f4').tofile(bf)
    for depth in [8,16,32]:
        name=f'glass-{depth}-{ds}';a=run(name,source,w,h,depth,cf,bf)
        assert np.isfinite(a).all()
        Image.fromarray(np.rint(np.clip(a,0,1)*255).astype('uint8'),'RGBA').save(out/(name+'.png'))
        rows.append({'case':name,'status':'PASS','size':[w,h],'alpha_min':float(a[:,:,3].min()),'alpha_max':float(a[:,:,3].max())})
identity=source.replace('label:"Amount" min:0 max:1 default:1','label:"Amount" min:0 max:1 default:0')
a=run('amount-zero',identity,640,400,32,out/'coverage-1.f32',out/'background-1.f32')
delta=float(np.max(np.abs(a-background)));assert delta<0.000002
rows.append({'case':'amount-zero','maximum_float_error':delta,'status':'PASS'})
binary_coverage=(coverage[:,:,:1]>.5).astype('<f4').repeat(4,axis=2);binary_coverage.tofile(out/'binary-coverage.f32')
field_source=source[:source.index('@pass Glass')]
field_source=field_source.replace('pass Glass: input, surface, host_alpha -> output','pass Decode: surface -> output')
field_source+='''@pass Decode
#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};
void main(){outColor=texelFetch(sampler2D(u_in,u_s),ivec2(v_uv*vec2(textureSize(sampler2D(u_in,u_s),0))),0);}
@endpass
'''
packed=[]
for depth in [8,32]:
    p=run('field-'+str(depth),field_source,640,400,depth,out/'binary-coverage.f32',out/'background-1.f32')
    packed.append(np.rint(p*255).astype('uint8'))
assert np.array_equal(*packed)
rows.append({'case':'float-field-transport-through-8-and-32-bpc','compared_pixels':256000,'different_bytes':int(np.count_nonzero(packed[0]!=packed[1])),'status':'PASS'})
(out/'quality-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(rows),flush=True)
