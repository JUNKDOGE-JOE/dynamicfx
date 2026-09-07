#!/usr/bin/env python3
"""Native AE 2026 smoke runner. Uses AE's public AppleScript/JSX API.

The setup refuses a dirty/existing project. Every call saves its exact JSX,
result JSON and AppleScript stdout/stderr. No blind retry of mutations.
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APP = "/Applications/Adobe After Effects 2026/Adobe After Effects 2026.app"
OUT = ROOT / "scripts/out/macos/ae2026"
TRANSPORT = "applescript"

COMMON = r'''
function encode(v) {
 if(v===null || v===undefined)return "null";
 if(typeof v==="boolean")return v?"true":"false";
 if(typeof v==="number")return isFinite(v)?String(v):"null";
 if(typeof v==="string")return '"'+v.replace(/[\\"\u0000-\u001f]/g,function(c){
 var h=c.charCodeAt(0).toString(16);return "\\u"+("0000"+h).slice(-4);})+'"';
 var a=[];if(v instanceof Array){for(var i=0;i<v.length;i++)a.push(encode(v[i]));return "["+a.join(",")+"]";}
 for(var k in v)if(v.hasOwnProperty(k))a.push(encode(k)+":"+encode(v[k]));return "{"+a.join(",")+"}";
}
function comp(n) { for(var i=1;i<=app.project.numItems;i++) {
  var c=app.project.item(i); if(c instanceof CompItem && c.name===n)return c;
} throw Error("missing comp "+n); }
function fx(c) {return c.layer("input").property("ADBE Effect Parade").property(1);}
function prop(f,n) {for(var i=1;i<=f.numProperties;i++) {
  if(f.property(i).name.indexOf(n)===0)return f.property(i);
} throw Error("missing property "+n);}
function source(f,s) {prop(f,"Source").expression="`"+s+"`;0";}
function state(c) {var f=fx(c);return {name:c.name,token:prop(f,"State Token").value,
 status:prop(f,"Status").name,source:prop(f,"Source").expression.length};}
function states() {var a=[];for(var i=1;i<=app.project.numItems;i++) {
 var c=app.project.item(i);if(c instanceof CompItem && c.name.indexOf("DFX_")===0)a.push(state(c));
}return a;}
function sample(c,x,y,t,ch) {
 var p=c.layer("probe").property("ADBE Effect Parade").property(1).property(1);
 p.expression='thisComp.layer("input").sampleImage(['+x+','+y+'],[0.49,0.49],true,'+t+')['+ch+']';
 var v=p.valueAtTime(t,false);if(p.expressionError)throw Error(p.expressionError);return v;
}
'''


def execute(name, code, timeout=120):
    OUT.mkdir(parents=True, exist_ok=True)
    script = OUT / f"{name}.jsx"
    result = OUT / f"{name}.json"
    if result.exists():
        raise RuntimeError(f"Result already exists: {result}; choose a new run name")
    wrapper = "(function(){" + COMMON + "\ntry {var data=(function(){\n" + code + "\n})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}\n"
    wrapper += f"var f=new File({json.dumps(str(result))});f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);}})();"
    script.write_text(wrapper)
    if TRANSPORT == "aemcp":
        # Optional driver only; the installed effect has no bridge dependency.
        sys.path.insert(0, str(ROOT / "scripts"))
        import aemcp
        response = aemcp.exec_js(wrapper, timeout_ms=timeout * 1000)
        (OUT / f"{name}.bridge.json").write_text(json.dumps(response, indent=2))
        if not result.exists():
            raise RuntimeError(f"No sentinel for {name}; unverified bridge outcome: {response}")
        data = json.loads(result.read_text())
        print(json.dumps({"step": name, **data}, ensure_ascii=False))
        if not data["ok"]:
            raise RuntimeError(data)
        return data["data"]
    apple = f'tell application {json.dumps(APP)} to DoScriptFile (POSIX file {json.dumps(str(script))})'
    try:
        p = subprocess.run(["osascript", "-e", apple], text=True, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as e:
        (OUT / f"{name}.timeout.txt").write_text(str(e))
        raise RuntimeError(f"Unverified outcome for {name}; inspect AE and result before continuing")
    (OUT / f"{name}.apple.txt").write_text(f"exit={p.returncode}\nstdout={p.stdout}\nstderr={p.stderr}")
    if not result.exists():
        raise RuntimeError(f"No sentinel for {name}: {p.stderr}")
    data = json.loads(result.read_text())
    print(json.dumps({"step": name, **data}, ensure_ascii=False))
    if not data["ok"]:
        raise RuntimeError(data)
    return data["data"]


def shader(body, fields="", annotations=""):
    return "#version 450\n" + annotations + "\n" + r'''
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_input;
layout(set=0,binding=1) uniform sampler u_sampler;
layout(set=0,binding=2) uniform FxUniforms {
vec2 u_resolution; float u_time; float u_frame;
''' + fields + "\n};\nvoid main(){" + body + "}\n"


def setup():
    ramp = shader("outColor=vec4(v_uv.x,v_uv.y,0.25,1.0);")
    inv = shader("vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);")
    multi = "@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n"
    for name, body in [("gen", ramp), ("inv1", inv), ("inv2", inv)]:
        multi += f"@pass {name}\n{body}@endpass\n"
    temporal = "@dynamicfx 1\n@graph\npass acc: prev -> output\n@end\n@pass acc\n" + shader("outColor=vec4(texture(sampler2D(u_input,u_sampler),v_uv).rgb+vec3(0.1),1.0);", annotations="// @window 4") + "@endpass\n"
    sources = {
        "DFX_gradient": ramp,
        "DFX_multi": multi,
        "DFX_param": shader("outColor=vec4(gain,0.0,0.0,1.0);", "float gain;", '// @param gain label:"Gain" min:0 max:2 default:0.25'),
        "DFX_hdr": shader("outColor=vec4(2.0,-0.5,0.123456,1.0);"),
        "DFX_temporal": temporal,
        "DFX_invalid": "#version 450\nthis is deliberately invalid shader source\n",
    }
    (OUT / "fixtures.json").parent.mkdir(parents=True, exist_ok=True)
    (OUT / "fixtures.json").write_text(json.dumps(sources, indent=2))
    return execute("setup", '''
if(app.project && (app.project.dirty || app.project.numItems>0))throw Error("Existing project: refusing replacement");
app.newProject(); app.project.bitsPerChannel=32; app.project.workingSpace="";
var sources=''' + json.dumps(sources) + ''';
for(var n in sources) {var c=app.project.items.addComp(n,320,240,1,2,25);
var l=c.layers.addSolid([0.05,0.4,0.1],"input",320,240,1,2);
var f=l.property("ADBE Effect Parade").addProperty("DynamicFx");
if(prop(f,"Language").value!==1)throw Error("Language is not GLSL");source(f,sources[n]);
var probe=c.layers.addNull(2);probe.name="probe";
probe.property("ADBE Effect Parade").addProperty("ADBE Slider Control");}
comp("DFX_gradient").openInViewer();
return {version:app.version,build:app.buildNumber,os:$.os,states:states()};
''')


def capture(tag):
    return execute(tag, '''
var ready=states();for(var q=0;q<ready.length;q++) {
var wanted=ready[q].name==="DFX_invalid"?2:1;
if(ready[q].token%4!==wanted)throw Error("Fixture not published: "+ready[q].name);}
app.purge(PurgeTarget.ALL_CACHES);var r={states:ready,depths:[]};
for(var d=0;d<3;d++){app.project.bitsPerChannel=[8,16,32][d];app.purge(PurgeTarget.ALL_CACHES);
var c=comp("DFX_gradient"),m=comp("DFX_multi"),h=comp("DFX_hdr");
var a={depth:app.project.bitsPerChannel,gradient:[],multi:[],hdr:[]};
for(var ch=0;ch<3;ch++){a.gradient.push(sample(c,160,120,0,ch));
a.multi.push(sample(m,160,120,0,ch));a.hdr.push(sample(h,160,120,0,ch));}r.depths.push(a);}
app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);
r.temporal=[];for(var i=0;i<4;i++){var t=[0,0.12,0.04,0.8][i];r.temporal.push({t:t,v:sample(comp("DFX_temporal"),160,120,t,0)});}
r.invalid=[];for(var ch=0;ch<3;ch++)r.invalid.push(sample(comp("DFX_invalid"),160,120,0,ch));
comp("DFX_gradient").saveFrameToPng(0,new File(OUT+"/"+TAG+"_gradient.png"));
comp("DFX_multi").saveFrameToPng(0,new File(OUT+"/"+TAG+"_multi.png"));
app.project.save(new File(OUT+"/native-smoke.aep"));return r;
'''.replace("OUT", json.dumps(str(OUT))).replace("TAG", json.dumps(tag)))


def main():
    global TRANSPORT
    ap = argparse.ArgumentParser()
    ap.add_argument("mode", choices=["setup", "state", "capture", "keyframes", "reopen", "queue", "script"])
    ap.add_argument("--name")
    ap.add_argument("--file", type=Path)
    ap.add_argument("--transport", choices=["applescript", "aemcp"], default="applescript")
    args = ap.parse_args()
    TRANSPORT = args.transport
    tag = args.name or args.mode
    if args.mode == "setup":
        setup()
    elif args.mode == "state":
        execute(tag, "return states();")
    elif args.mode == "capture":
        capture(tag)
    elif args.mode == "keyframes":
        execute(tag, '''var c=comp("DFX_param");var p=prop(fx(c),"Gain");
p.setValueAtTime(0,0.25);p.setValueAtTime(1,0.75);app.purge(PurgeTarget.ALL_CACHES);
var r={t0:sample(c,160,120,0,0),t1:sample(c,160,120,1,0),keys:p.numKeys};
app.project.save(new File(''' + json.dumps(str(OUT / "native-smoke.aep")) + "));return r;")
    elif args.mode == "reopen":
        execute(tag, "app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);app.open(new File(" + json.dumps(str(OUT / "native-smoke.aep")) + "));return states();")
    elif args.mode == "queue":
        execute(tag, '''app.project.bitsPerChannel=8;app.purge(PurgeTarget.ALL_CACHES);
var i=app.project.renderQueue.items.add(comp("DFX_multi"));i.timeSpanStart=0;i.timeSpanDuration=1/25;
var om=i.outputModule(1);om.applyTemplate("Photoshop");om.file=new File(''' + json.dumps(str(OUT / "aerender_[#####].psd")) + ''');
app.project.save(new File(''' + json.dumps(str(OUT / "native-aerender.aep")) + '''));
return {templates:om.templates,project:app.project.file.fsName};''')
    elif args.mode == "script":
        execute(tag, args.file.read_text())


if __name__ == "__main__":
    main()
