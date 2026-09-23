import json,subprocess
from pathlib import Path
import numpy as np
from PIL import Image
root=Path(__file__).resolve().parents[3];out=Path(__file__).parent
binary=root/'scripts/quality/target/release/dynamicfx-quality.exe';source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
rows=[]
def cpu_distance(mask):
    padded=np.pad(mask,1);near=np.zeros_like(padded)
    for dy in [-1,0,1]:
        for dx in [-1,0,1]:near|=np.roll(np.roll(padded,dy,axis=0),dx,axis=1)
    edges=np.argwhere(near&~padded);points=np.argwhere(padded);result=np.zeros(padded.shape,dtype='float64')
    for start in range(0,len(points),256):
        p=points[start:start+256];d=((p[:,None,:]-edges[None,:,:])**2).sum(axis=2).min(axis=1);result[p[:,0],p[:,1]]=np.sqrt(d)
    return result[1:-1,1:-1]
def render(name,shader,cov,bg,depth,logical):
    h,w=cov.shape[:2];src=out/(name+'.glsl');src.write_text(shader,encoding='utf-8');cp=out/(name+'-coverage.f32');bp=out/(name+'-input.f32');cov.astype('<f4').tofile(cp);bg.astype('<f4').tofile(bp)
    manifest=out/(name+'-inputs.json');manifest.write_text(json.dumps({'host_alpha':{'width':w,'height':h,'path':str(cp.resolve())}}),encoding='utf-8');dest=out/(name+'.f32')
    p=subprocess.run([str(binary),str(src),str(dest),str(w),str(h),str(logical[0]),str(logical[1]),'0',str(depth),'rgba:'+str(bp),str(manifest)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    (out/(name+'.stdout.log')).write_bytes(p.stdout);(out/(name+'.stderr.log')).write_bytes(p.stderr)
    assert p.returncode==0,p.stderr.decode(errors='replace');a=np.fromfile(dest,dtype='<f4').reshape(h,w,4);assert np.isfinite(a).all();return a
def check(name,ok,**values):
    rows.append({'case':name,'status':'PASS' if ok else 'FAIL',**values});(out/'gpu-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(rows[-1]),flush=True);assert ok,rows[-1]
field=source[:source.index('@pass FrostX')];start=field.index('@graph');end=field.index('@end')
field=field[:start]+'@graph\npass DistanceX: host_alpha -> distance_x\npass Distance: distance_x -> output\n'+field[end:]
w,h=193,129;y,x=np.mgrid[:h,:w];masks={'rect':(x>25)&(x<165)&(y>20)&(y<108),'ring':((x-95)**2+(y-64)**2<52**2)&((x-95)**2+(y-64)**2>21**2),'disjoint':((x-45)**2+(y-64)**2<28**2)|((x>115)&(x<170)&(y>25)&(y<105))}
optical=source.replace('label:"Frost Radius (px)" min:0 max:12 default:1','label:"Frost Radius (px)" min:0 max:12 default:0').replace('label:"Dispersion" min:0 max:50 default:7','label:"Dispersion" min:0 max:50 default:0').replace('label:"Fresnel Strength" min:0 max:1 default:0.2','label:"Fresnel Strength" min:0 max:1 default:0').replace('label:"Glare Strength" min:0 max:1.2 default:0.9','label:"Glare Strength" min:0 max:1.2 default:0')
w,h=193,129;y,x=np.mgrid[:h,:w];mask=masks['rect'];cov=mask[:,:,None].repeat(4,axis=2).astype('float32');bg=np.stack([(x+.5)/w,(y+.5)/h,np.full_like(x,.3,dtype='float64'),np.ones_like(x)],axis=-1)
a=render('optical-reference',optical,cov,bg,32,[w,h]);errors=[]
for px in [30,33,38,44,60]:
    d=px-25-.5;r=max(0,min(1,1-d/20));theta_i=np.arcsin(r*r);theta_t=np.arcsin(np.sin(theta_i)/1.4);shift=max(0,np.tan(theta_i-theta_t))*70.710678 if d<20 else 0
    expected=np.array([np.clip(px+.5+shift,.5,w-.5)/w,64.5/h,.3,1]);errors.append(float(np.max(np.abs(a[64,px]-expected))))
probe=optical[:optical.index('float spread=clamp(dispersion_strength')]+"outColor=vec4(offset*u_resolution,edgeFactor,1.0);}\n@endpass\n"
a=render('displacement-reference',probe,cov,bg,32,[w,h]);errors=[]
for px in [30,33,38,44,60]:
    d=px-25-.5;r=max(0,min(1,1-d/20));ti=np.arcsin(r*r);tt=np.arcsin(np.sin(ti)/1.4);factor=max(0,np.tan(ti-tt)) if d<20 else 0
    expected=np.array([factor*70.710678,0,factor,1]);errors.append(float(np.max(np.abs(a[64,px]-expected))))
check('upstream Snell displacement isolated from filtering',max(errors)<.0001,maximum_error_pixels=max(errors),samples=len(errors))
