import json,sys,time
from pathlib import Path
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
out=Path(__file__).parent;root=out.resolve().parents[2];client=Client()
source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
project=(out/'Liquid Glass.aep').resolve().as_posix()
def call(name,code,guard=True):
    prefix='if(!app.project.file||decodeURI(app.project.file.name)!=="Liquid Glass.aep")throw Error("Unexpected project");' if guard else ''
    args={'code':'(function(){'+prefix+code+'})()','timeout_sec':30};r=client.call('ae_exec',args,timeout=40)
    (out/(name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    print(json.dumps({'step':name,'data':d['content']},ensure_ascii=True),flush=True)
    return json.loads(d['content'])
background='''#version 450
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms {vec2 u_resolution;float u_time;float u_frame;};
void main(){vec2 uv=v_uv;vec3 c=mix(vec3(.22,.23,.66),vec3(.88,.30,.39),smoothstep(-.15,1.25,uv.x+.2*uv.y));
c=mix(c,vec3(.15,.77,.77),.85*exp(-dot((uv-vec2(.1,.85))*vec2(1.3,1.0),(uv-vec2(.1,.85))*vec2(1.3,1.0))*5.0));
c=mix(c,vec3(1.0,.76,.30),.85*exp(-dot((uv-vec2(.88,.1))*vec2(1.2,1.0),(uv-vec2(.88,.1))*vec2(1.2,1.0))*7.0));
vec2 p=uv*u_resolution+vec2(12.0*u_time,0.0);vec2 d=min(mod(p,90.0),90.0-mod(p,90.0));float g=min(d.x,d.y);float line=1.0-smoothstep(.45,.45+max(fwidth(g),.7),g);
outColor=vec4(mix(c,vec3(1.0),line*.18),1.0);}
'''
mode=sys.argv[1]
if mode=='setup':
    code='if(!app.project.file||app.project.file.name!=="resources.aep")throw Error("Unexpected project");app.project.save();app.newProject();app.project.bitsPerChannel=32;app.project.workingSpace="";var c=app.project.items.addComp("Liquid Glass - edit the native shapes",1440,900,1,6,25);c.hideShyLayers=true;var b=c.layers.addSolid([0,0,0],"Color field",1440,900,1,6),f=b.property("ADBE Effect Parade").addProperty("DynamicFx");f.property(3).expression="`"+'+json.dumps(background)+'+"`;0";'
    code+='''function text(name,value,size,position,color){var l=c.layers.addText(value);l.name=name;var p=l.property("ADBE Text Properties").property("ADBE Text Document"),d=p.value;d.font="SegoeUI-Semibold";d.fontSize=size;d.applyFill=true;d.fillColor=color;d.justification=ParagraphJustification.LEFT_JUSTIFY;p.setValue(d);l.property("ADBE Transform Group").property("ADBE Position").setValue(position);return l;}
text("Heading","LIQUID GLASS",86,[90,145],[.09,.12,.22]);
text("Kicker","DYNAMICFX   /   MATERIAL STUDY 01",18,[94,58],[.13,.15,.25]);
text("Refraction type","FREEFORM",166,[255,512],[.11,.16,.23]);
text("Footer","EDIT THE SHAPE. THE MATERIAL FOLLOWS.",19,[94,817],[.10,.15,.22]);
text("Details","NATIVE ALPHA     /     REFRACTION     /     ANIMATABLE",13,[94,847],[.16,.20,.29]);
'''
    code+='app.project.save(new File('+json.dumps(project)+'));return JSON.stringify({comp:c.id,items:app.project.numItems});'
    call('demo-background',code,False);time.sleep(3)
    code='var c=null;for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i) instanceof CompItem)c=app.project.item(i);if(!c)throw Error("Missing comp");var shader='+json.dumps(source)+';'
    code+='''function glass(name,kind,center,size){var l=c.layers.addShape();l.name=name;l.adjustmentLayer=true;l.property("ADBE Transform Group").property("ADBE Position").setValue([0,0]);var g=l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");g.name="Edit this shape";var v=g.property("ADBE Vectors Group"),p=v.addProperty(kind==="ellipse"?"ADBE Vector Shape - Ellipse":"ADBE Vector Shape - Rect");p.property(kind==="ellipse"?"ADBE Vector Ellipse Size":"ADBE Vector Rect Size").setValue(size);p.property(kind==="ellipse"?"ADBE Vector Ellipse Position":"ADBE Vector Rect Position").setValue(center);if(kind!=="ellipse")p.property("ADBE Vector Rect Roundness").setValue(82);var fill=v.addProperty("ADBE Vector Graphic - Fill");fill.property("ADBE Vector Fill Color").setValue([1,1,1]);var e=l.property("ADBE Effect Parade").addProperty("DynamicFx");e.property(3).expression="`"+shader+"`;0";return l;}
var card=glass("Glass - rounded card","rect",[700,448],[930,430]);
var circle=glass("Glass - circle","ellipse",[1160,655],[222,222]);
var pill=glass("Glass - capsule","rect",[328,694],[300,100]);
card.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Rect").property("ADBE Vector Rect Size").setValueAtTime(0,[930,430]);
card.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Rect").property("ADBE Vector Rect Size").setValueAtTime(3,[1000,400]);
c.time=0;app.project.save();return JSON.stringify({card:card.id,circle:circle.id,pill:pill.id,comp:c.id});'''
    call('demo-glass',code);time.sleep(3)
    call('demo-ready','var c=app.project.item(1),rows=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");if(l instanceof ShapeLayer){var e=fx.property(1);rows.push({id:l.id,name:l.name,state:e.property(427).value,reader:e.property(426).value});}}app.project.save();return JSON.stringify(rows);')
elif mode=='capture':
    dest=(out/'ae-hero-32-full.png').resolve().as_posix()
    call('demo-hero','var c=app.project.item(1);app.project.bitsPerChannel=32;c.resolutionFactor=[1,1];app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File('+json.dumps(dest)+'));return JSON.stringify({depth:32,size:[c.width,c.height]});')
elif mode=='resume-shapes':
    args=json.loads((out/'demo-glass.json').read_text(encoding='utf-8'))['args'];args['code']=args['code'].replace('app.project.file.name!=="Liquid Glass.aep"','decodeURI(app.project.file.name)!=="Liquid Glass.aep"')
    r=client.call('ae_exec',args,timeout=40);(out/'demo-glass-recovered.json').write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r);print(json.dumps(d,ensure_ascii=True),flush=True)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    time.sleep(3)
    call('demo-ready','var c=app.project.itemByID(1),rows=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");if(l instanceof ShapeLayer){var e=fx.property(1);rows.push({id:l.id,name:l.name,state:e.property(427).value,reader:e.property(426).value});}}app.project.save();return JSON.stringify(rows);')
