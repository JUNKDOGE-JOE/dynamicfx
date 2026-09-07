#!/usr/bin/env python3
"""Production WGSL AE acceptance driver; every host call has a unique run tag.

Uses the existing smoke runner's COMMON/execute and the optional aemcp test
transport. Import, fixtures and --dry-run do not contact AE. The normal modes
do mutate only the explicitly prepared test project; no retry, install or quit.

Workflow (explicit --name labels are optional; defaults are unique): setup,
state after returning to AE idle, capture, resources --phase none, resources
--phase assign, return to idle, resources --phase assigned, keyframes, save,
reopen, state after idle, capture, keys-read, resources --phase assigned, queue.
The root operator runs aerender separately on QUEUE_PROJECT, then checks its
output with check_acceptance.py --queue <queue.json> --exports <OUT>.
JSON capture files can be supplied directly to check_acceptance.py.
Run actual UI language-change Undo/Redo and viewport checks separately.
Capture defaults to Full; --resolution half/quarter records sampleImage under
that setting, which does not itself prove the physical viewport resolution.
"""
import argparse
import datetime
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "scripts/out/010/ae2026"
PROJECT = OUT / "native-wgsl-010.aep"
QUEUE_PROJECT = OUT / "native-wgsl-010-aerender.aep"
spec = importlib.util.spec_from_file_location("macos_smoke", ROOT / "scripts/macos/ae_smoke.py")
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)
BASE_COMMON = smoke.COMMON


def shader(language, body, fields="", annotations="", extra=False):
    if language == "glsl":
        source = smoke.shader(body, fields, annotations)
        if extra:
            source = source.replace("void main()", "layout(set=0,binding=3) uniform texture2D u_extra;\nvoid main()")
        return source
    return annotations + "\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n" + fields + "\n}\n" + """@group(0) @binding(0) var u_input: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;
@group(0) @binding(2) var<uniform> fx: FxUniforms;
""" + ("@group(0) @binding(3) var u_extra: texture_2d<f32>;\n" if extra else "") + "@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\n" + body + "\n}\n"


def envelope(language, graph, passes):
    result = "@dynamicfx 1\n@graph\n" + graph + "\n@end\n"
    for name, body in passes:
        if language == "wgsl":
            body = "\n".join("@" + line if line.startswith("@") else line for line in body.splitlines()) + "\n"
        result += "@pass " + name + "\n" + body + "@endpass\n"
    return result


def fixtures(nonce="fixture"):
    result = {}
    for lang in ("glsl", "wgsl"):
        g = lang == "glsl"
        ramp = shader(lang, "outColor=vec4(v_uv,0.625,1.0);" if g else "return vec4<f32>(v_uv,0.625,1.0);")
        inverse = shader(lang, "vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);" if g else "let c=textureSample(u_input,u_sampler,v_uv);return vec4<f32>(vec3<f32>(1.0)-c.rgb,1.0);")
        gain = shader(lang, "outColor=vec4(gain,0.0,0.0,1.0);" if g else "return vec4<f32>(fx.gain,0.0,0.0,1.0);", "float gain;" if g else "gain: f32,", '// @param gain label:"Gain" min:0 max:2 default:0.25')
        temporal = shader(lang, "outColor=vec4(texture(sampler2D(u_input,u_sampler),v_uv).rgb+vec3(0.1),1.0);" if g else "return vec4<f32>(textureSample(u_input,u_sampler,v_uv).rgb+vec3<f32>(0.1),1.0);", annotations="// @window 4")
        layer = shader(lang, "vec4 side=texture(sampler2D(u_extra,u_sampler),v_uv);vec4 self=texture(sampler2D(u_input,u_sampler),v_uv);outColor=mix(self,vec4(side.rgb,1.0),side.a);" if g else "let side=textureSample(u_extra,u_sampler,v_uv);let own=textureSample(u_input,u_sampler,v_uv);return mix(own,vec4<f32>(side.rgb,1.0),side.a);", annotations='// @param side label:"Side Layer" hint:layer', extra=True)
        gradient = shader(lang, "outColor=vec4(texture(sampler2D(u_extra,u_sampler),vec2(v_uv.x,0.5)).rgb,1.0);" if g else "return vec4<f32>(textureSample(u_extra,u_sampler,vec2<f32>(v_uv.x,0.5)).rgb,1.0);", annotations='// @param ramp label:"Ramp" hint:gradient', extra=True)
        path = shader(lang, "ivec2 sz=textureSize(sampler2D(u_extra,u_sampler),0);vec4 p=texelFetch(sampler2D(u_extra,u_sampler),ivec2(0),0);outColor=vec4(p.xy,float(sz.x)/16.0,1.0);" if g else "let sz=textureDimensions(u_extra,0);let p=textureLoad(u_extra,vec2<i32>(0),0);return vec4<f32>(p.xy,f32(sz.x)/16.0,1.0);", annotations='// @param outline label:"Outline" hint:path', extra=True)
        sources = {
            "uv": ramp,
            "multi": envelope(lang, "pass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output", [("gen", ramp), ("inv1", inverse), ("inv2", inverse)]),
            "hdr": shader(lang, "outColor=vec4(2.0,-0.5,0.123456,1.0);" if g else "return vec4<f32>(2.0,-0.5,0.123456,1.0);"),
            "gain": gain,
            "temporal": envelope(lang, "pass acc: prev -> output", [("acc", temporal)]),
            "invalid": "#version 450\ninvalid shader text\n" if g else "@fragment fn main( { definitely invalid WGSL\n",
            "layer": envelope(lang, "pass main: input, side -> output", [("main", layer)]),
            "gradient": envelope(lang, "pass main: input, ramp -> output", [("main", gradient)]),
            "path": envelope(lang, "pass main: input, outline -> output", [("main", path)]),
        }
        for kind, source in sources.items():
            # A unique source comment prevents accidental reuse of an older run's
            # disk-cache identity; it is deliberately not a shader uniform.
            if source.startswith("@dynamicfx"):
                at = source.index("\n", source.index("@pass ")) + 1
                source = source[:at] + "// acceptance " + nonce + "\n" + source[at:]
            else:
                source = "// acceptance " + nonce + "\n" + source
            result["DFX_010_" + lang + "_" + kind] = {
                "language": 1 if g else 2, "kind": kind, "source": source,
                "width": 160 if kind in ("layer", "gradient", "path") else 321,
                "height": 120 if kind in ("layer", "gradient", "path") else 239,
            }
    return result


EXTRA = r'''
function named(f,n){var found=null;for(var i=1;i<=f.numProperties;i++){var p=f.property(i);
if(p.name===n){if(found)throw Error("Ambiguous property "+n);found=p;}}
if(!found)throw Error("Named property not ready: "+n);return found;}
function source(f,s){var q=s.replace(/\\/g,"\\\\").replace(/`/g,"\\`").replace(/\$\{/g,"\\${");prop(f,"Source").expression="`"+q+"`;0";}
function c010(lang,kind){return comp("DFX_010_"+lang+"_"+kind);}
function s010(c){var s=state(c);s.language=named(fx(c),"Language").value;s.width=c.width;s.height=c.height;s.resolution=c.resolutionFactor;s.code=s.token%4===2?Math.floor(s.token/4):0;return s;}
function states010(){var a=[];for(var n in FIXTURES)if(FIXTURES.hasOwnProperty(n))a.push(s010(comp(n)));return a;}
function ready010(){var a=states010();for(var i=0;i<a.length;i++){var s=a[i],invalid=s.name.indexOf("_invalid")>=0;
if(s.token%4!==(invalid?2:1))throw Error("Not published: "+s.name+" token="+s.token);
if(invalid && s.code!==(s.language===2?21:17))throw Error("Unexpected diagnostic "+s.name+" E"+s.code);}return a;}
function rgba(c,x,y,t){var a=[];for(var ch=0;ch<4;ch++)a.push(sample(c,x,y,t,ch));return a;}
function setResolution(d){for(var n in FIXTURES)if(FIXTURES.hasOwnProperty(n))comp(n).resolutionFactor=[d,d];}
function ownProject(){if(!app.project||!app.project.file||app.project.file.fsName!==PROJECT)throw Error("Not the saved acceptance project");}
function save010(){app.project.save(new File(PROJECT));return app.project.file.fsName;}
'''


def code_for(mode, *, phase="none", resolution=1, script=None):
    if mode == "setup":
        return r'''
if(app.project&&(app.project.dirty||app.project.numItems>0))throw Error("Existing/dirty project: refusing replacement");
app.newProject();app.project.bitsPerChannel=32;app.project.workingSpace="";
var defaults=[];for(var n in FIXTURES)if(FIXTURES.hasOwnProperty(n)){var v=FIXTURES[n];
var c=app.project.items.addComp(n,v.width,v.height,1,2,25);c.resolutionFactor=[1,1];
if(v.kind==="layer")c.layers.addSolid([20/255,180/255,220/255],"sourceLayer",v.width,v.height,1,2);
var color=v.kind==="layer"?[200/255,40/255,40/255]:[0.05,0.4,0.1];
var l=c.layers.addSolid(color,"input",v.width,v.height,1,2);
var f=l.property("ADBE Effect Parade").addProperty("DynamicFx");var lp=named(f,"Language");
defaults.push({name:n,value:lp.value});if(lp.value!==1)throw Error("GLSL default changed");lp.setValue(v.language);source(f,v.source);
if(v.kind==="path"){var mask=l.property("ADBE Mask Parade").addProperty("ADBE Mask Atom");var shape=new Shape();
shape.vertices=[[40,30],[120,30],[120,90],[40,90]];shape.closed=true;mask.property("ADBE Mask Shape").setValue(shape);mask.maskMode=MaskMode.NONE;}
var probe=c.layers.addNull(2);probe.name="probe";probe.property("ADBE Effect Parade").addProperty("ADBE Slider Control");}
c010("wgsl","uv").openInViewer();return {mode:"setup",version:app.version,build:app.buildNumber,os:$.os,defaults:defaults,states:states010(),project:save010()};
'''
    if mode == "state":
        return 'return {mode:"state",version:app.version,build:app.buildNumber,project:app.project.file?app.project.file.fsName:null,dirty:app.project.dirty,depth:app.project.bitsPerChannel,states:states010()};'
    if mode == "capture":
        return r'''
setResolution(RESOLUTION);var ready=ready010();var r={mode:"capture",states:ready,resolutionSetting:[RESOLUTION,RESOLUTION],sampleImageDoesNotProvePhysicalPreview:true,points:[[80.5,59.5],[160.5,119.5],[240.5,179.5]],depths:[]};
for(var d=0;d<3;d++){app.project.bitsPerChannel=[8,16,32][d];app.purge(PurgeTarget.ALL_CACHES);var row={depth:app.project.bitsPerChannel,pairs:{}};
for(var k=0;k<3;k++){var kind=["uv","multi","hdr"][k];row.pairs[kind]={};for(var l=0;l<2;l++){var lang=["glsl","wgsl"][l],c=c010(lang,kind),a=[];
for(var p=0;p<r.points.length;p++)a.push(rgba(c,r.points[p][0],r.points[p][1],0));row.pairs[kind][lang]=a;}}r.depths.push(row);}
app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);r.temporal={};r.invalid={};
for(var l=0;l<2;l++){var lang=["glsl","wgsl"][l],a=[];for(var j=0;j<4;j++){var t=[0,0.12,0.04,0.8][j];a.push({t:t,rgba:rgba(c010(lang,"temporal"),160.5,119.5,t)});}r.temporal[lang]=a;r.invalid[lang]=rgba(c010(lang,"invalid"),160.5,119.5,0);}
r.project=save010();return r;
'''.replace("RESOLUTION", str(resolution))
    if mode == "resources":
        if phase == "assign":
            return r'''
ready010();var values=[];for(var i=0;i<2;i++){var lang=["glsl","wgsl"][i],c=c010(lang,"layer");named(fx(c),"Side Layer").setValue(c.layer("sourceLayer").index);
named(fx(c010(lang,"path")),"Outline").setValue(1);var g=fx(c010(lang,"gradient"));
if(named(g,"Ramp Stops").value!==2)throw Error("Unexpected gradient stop count");
named(g,"Ramp 01 Color").setValue([1,0,0,1]);named(g,"Ramp 02 Color").setValue([0,0,1,1]);
values.push({language:lang,layer:named(fx(c),"Side Layer").value,path:named(fx(c010(lang,"path")),"Outline").value,stops:named(g,"Ramp Stops").value});}
return {mode:"resources-assign",values:values,project:save010(),next:"Return to AE idle, then run resources --phase assigned with a new tag"};
'''
        return r'''
ready010();setResolution(1);app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);var r={mode:"resources",phase:PHASE,depth:32,resolutionSetting:[1,1],pairs:{},gradientPoints:[[40.5,60.5],[80.5,60.5],[120.5,60.5]]};
for(var i=0;i<2;i++){var lang=["glsl","wgsl"][i],lc=c010(lang,"layer"),pc=c010(lang,"path"),gc=c010(lang,"gradient");
var layerSel=named(fx(lc),"Side Layer").value,pathSel=named(fx(pc),"Outline").value;
if(PHASE==="none"&&(layerSel!==0||pathSel!==0))throw Error("None phase must precede assignments");
if(PHASE==="assigned"&&(layerSel!==lc.layer("sourceLayer").index||pathSel!==1))throw Error("Assigned selectors were not retained");
var samples=[];for(var p=0;p<r.gradientPoints.length;p++)samples.push(rgba(gc,r.gradientPoints[p][0],r.gradientPoints[p][1],0));
r.pairs[lang]={layer:rgba(lc,80.5,60.5,0),path:rgba(pc,80.5,60.5,0),gradient:samples,layerSelector:layerSel,pathSelector:pathSel,gradientStops:named(fx(gc),"Ramp Stops").value};}
r.project=save010();return r;
'''.replace("PHASE", json.dumps(phase))
    if mode in ("keyframes", "keys-read"):
        mutation = 'p.setValueAtTime(0,0.25);p.setValueAtTime(1,0.75);' if mode == "keyframes" else ""
        return r'''
ready010();setResolution(1);app.project.bitsPerChannel=32;var r={mode:"keyframes",written:WRITTEN,pairs:{}};
for(var i=0;i<2;i++){var lang=["glsl","wgsl"][i],c=c010(lang,"gain"),p=named(fx(c),"Gain");MUTATION
app.purge(PurgeTarget.ALL_CACHES);r.pairs[lang]={keys:p.numKeys,t0:rgba(c,160.5,119.5,0),t1:rgba(c,160.5,119.5,1),value0:p.valueAtTime(0,false),value1:p.valueAtTime(1,false)};}
r.project=save010();return r;
'''.replace("MUTATION", mutation).replace("WRITTEN", "true" if mutation else "false")
    if mode == "save":
        return 'ready010();return {mode:"save",project:save010(),states:states010()};'
    if mode == "reopen":
        return 'ownProject();if(app.project.dirty)throw Error("Save the test project before reopening");app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);app.open(new File(PROJECT));return {mode:"reopen",project:app.project.file.fsName,states:states010()};'
    if mode == "queue":
        return r'''
ready010();setResolution(1);app.project.bitsPerChannel=8;app.purge(PurgeTarget.ALL_CACHES);var r={mode:"queue",depth:8,expectedFrames:[0,1],items:[]};
for(var q=1;q<=app.project.renderQueue.numItems;q++)app.project.renderQueue.item(q).render=false;
for(var l=0;l<2;l++)for(var k=0;k<2;k++)for(var d=0;d<3;d++){var lang=["glsl","wgsl"][l],kind=["uv","multi"][k],res=["Full","Half","Quarter"][d];
var c=c010(lang,kind),q=app.project.renderQueue.items.add(c);q.render=true;q.setSettings({"Resolution":res,"Quality":"Best","Color Depth":"Current Settings"});q.timeSpanStart=0;q.timeSpanDuration=1/25;
var om=q.outputModule(1),found=false;for(var t=0;t<om.templates.length;t++)if(om.templates[t]==="Photoshop")found=true;if(!found)throw Error("Photoshop template missing");om.applyTemplate("Photoshop");
om.file=new File(OUT+"/"+TAG+"_"+lang+"_"+kind+"_"+res.toLowerCase()+"_[#####].psd");
r.items.push({language:lang,kind:kind,resolution:res,logical:[c.width,c.height],physical:[Math.ceil(c.width/[1,2,4][d]),Math.ceil(c.height/[1,2,4][d])],start:q.timeSpanStart,duration:q.timeSpanDuration,output:om.file.fsName});}
app.project.save(new File(QUEUE_PROJECT));r.project=app.project.file.fsName;return r;
'''
    if mode == "script":
        if script is None:
            raise ValueError("script mode requires --file")
        return script.read_text()
    raise ValueError(mode)


def run(mode, name=None, phase="none", resolution=1, script=None, dry_run=False):
    OUT.mkdir(parents=True, exist_ok=True)
    tag = name or mode + "-" + datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ") + "-" + uuid.uuid4().hex[:6]
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", tag):
        raise ValueError("Run name must contain only letters, digits, dot, underscore or dash")
    for suffix in (".jsx", ".json", ".request.json", ".dry.jsx"):
        if (OUT / (tag + suffix)).exists():
            raise RuntimeError("Run tag already reserved; inspect the previous outcome, never blindly retry")
    fixture_data = fixtures(tag if mode == "setup" else "reference")
    constants = "\nvar FIXTURES=" + json.dumps(fixture_data) + ";\nvar PROJECT=" + json.dumps(str(PROJECT)) + ";\nvar QUEUE_PROJECT=" + json.dumps(str(QUEUE_PROJECT)) + ";\nvar OUT=" + json.dumps(str(OUT)) + ";\nvar TAG=" + json.dumps(tag) + ";\n"
    common = BASE_COMMON + constants + EXTRA
    code = code_for(mode, phase=phase, resolution=resolution, script=script)
    request = {"tag": tag, "mode": mode, "phase": phase if mode == "resources" else None, "dry_run": dry_run, "transport": "aemcp", "code_sha256": hashlib.sha256((common + code).encode()).hexdigest()}
    # Reserve before invoking the transport, including calls whose outcome may
    # become unknown. Even an absent result must not make a mutation retryable.
    with (OUT / (tag + ".request.json")).open("x") as f:
        json.dump(request, f, indent=2); f.write("\n")
    if mode == "setup":
        (OUT / (tag + ".fixtures.json")).write_text(json.dumps(fixture_data, indent=2) + "\n")
    if dry_run:
        path = OUT / (tag + ".dry.jsx")
        path.write_text("(function(){\n" + common + "\n" + code + "\n})();\n")
        print(json.dumps({"dry_run": True, "script": str(path), "tag": tag}))
        return
    smoke.OUT = OUT; smoke.TRANSPORT = "aemcp"; smoke.COMMON = common
    return smoke.execute(tag, code)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["fixtures", "setup", "state", "capture", "resources", "keyframes", "keys-read", "save", "reopen", "queue", "script"])
    parser.add_argument("--name")
    parser.add_argument("--phase", choices=["none", "assign", "assigned"], default="none")
    parser.add_argument("--resolution", choices=["full", "half", "quarter"], default="full")
    parser.add_argument("--file", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.mode == "fixtures":
        print(json.dumps(fixtures(args.name or "offline"), indent=2)); return
    run(args.mode, args.name, args.phase, {"full": 1, "half": 2, "quarter": 4}[args.resolution], args.file, args.dry_run)


if __name__ == "__main__":
    main()
