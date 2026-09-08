(function(){
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

var FIXTURES={"DFX_010_glsl_uv": {"language": 1, "kind": "uv", "source": "// acceptance reference\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(v_uv,0.625,1.0);}\n", "width": 321, "height": 239}, "DFX_010_glsl_multi": {"language": 1, "kind": "multi", "source": "@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n@pass gen\n// acceptance reference\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(v_uv,0.625,1.0);}\n@endpass\n@pass inv1\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n@pass inv2\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n", "width": 321, "height": 239}, "DFX_010_glsl_hdr": {"language": 1, "kind": "hdr", "source": "// acceptance reference\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(2.0,-0.5,0.123456,1.0);}\n", "width": 321, "height": 239}, "DFX_010_glsl_gain": {"language": 1, "kind": "gain", "source": "// acceptance reference\n#version 450\n// @param gain label:\"Gain\" min:0 max:2 default:0.25\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\nfloat gain;\n};\nvoid main(){outColor=vec4(gain,0.0,0.0,1.0);}\n", "width": 321, "height": 239}, "DFX_010_glsl_temporal": {"language": 1, "kind": "temporal", "source": "@dynamicfx 1\n@graph\npass acc: prev -> output\n@end\n@pass acc\n// acceptance reference\n#version 450\n// @window 4\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(texture(sampler2D(u_input,u_sampler),v_uv).rgb+vec3(0.1),1.0);}\n@endpass\n", "width": 321, "height": 239}, "DFX_010_glsl_invalid": {"language": 1, "kind": "invalid", "source": "// acceptance reference\n#version 450\ninvalid shader text\n", "width": 321, "height": 239}, "DFX_010_glsl_layer": {"language": 1, "kind": "layer", "source": "@dynamicfx 1\n@graph\npass main: input, side -> output\n@end\n@pass main\n// acceptance reference\n#version 450\n// @param side label:\"Side Layer\" hint:layer\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nlayout(set=0,binding=3) uniform texture2D u_extra;\nvoid main(){vec4 side=texture(sampler2D(u_extra,u_sampler),v_uv);vec4 self=texture(sampler2D(u_input,u_sampler),v_uv);outColor=mix(self,vec4(side.rgb,1.0),side.a);}\n@endpass\n", "width": 160, "height": 120}, "DFX_010_glsl_gradient": {"language": 1, "kind": "gradient", "source": "@dynamicfx 1\n@graph\npass main: input, ramp -> output\n@end\n@pass main\n// acceptance reference\n#version 450\n// @param ramp label:\"Ramp\" hint:gradient\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nlayout(set=0,binding=3) uniform texture2D u_extra;\nvoid main(){outColor=vec4(texture(sampler2D(u_extra,u_sampler),vec2(v_uv.x,0.5)).rgb,1.0);}\n@endpass\n", "width": 160, "height": 120}, "DFX_010_glsl_path": {"language": 1, "kind": "path", "source": "@dynamicfx 1\n@graph\npass main: input, outline -> output\n@end\n@pass main\n// acceptance reference\n#version 450\n// @param outline label:\"Outline\" hint:path\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nlayout(set=0,binding=3) uniform texture2D u_extra;\nvoid main(){ivec2 sz=textureSize(sampler2D(u_extra,u_sampler),0);vec4 p=texelFetch(sampler2D(u_extra,u_sampler),ivec2(0),0);outColor=vec4(p.xy,float(sz.x)/16.0,1.0);}\n@endpass\n", "width": 160, "height": 120}, "DFX_010_wgsl_uv": {"language": 2, "kind": "uv", "source": "// acceptance reference\n\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@group(0) @binding(0) var u_input: texture_2d<f32>;\n@group(0) @binding(1) var u_sampler: sampler;\n@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(v_uv,0.625,1.0);\n}\n", "width": 321, "height": 239}, "DFX_010_wgsl_multi": {"language": 2, "kind": "multi", "source": "@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n@pass gen\n// acceptance reference\n\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(v_uv,0.625,1.0);\n}\n@endpass\n@pass inv1\n\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nlet c=textureSample(u_input,u_sampler,v_uv);return vec4<f32>(vec3<f32>(1.0)-c.rgb,1.0);\n}\n@endpass\n@pass inv2\n\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nlet c=textureSample(u_input,u_sampler,v_uv);return vec4<f32>(vec3<f32>(1.0)-c.rgb,1.0);\n}\n@endpass\n", "width": 321, "height": 239}, "DFX_010_wgsl_hdr": {"language": 2, "kind": "hdr", "source": "// acceptance reference\n\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@group(0) @binding(0) var u_input: texture_2d<f32>;\n@group(0) @binding(1) var u_sampler: sampler;\n@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(2.0,-0.5,0.123456,1.0);\n}\n", "width": 321, "height": 239}, "DFX_010_wgsl_gain": {"language": 2, "kind": "gain", "source": "// acceptance reference\n// @param gain label:\"Gain\" min:0 max:2 default:0.25\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\ngain: f32,\n}\n@group(0) @binding(0) var u_input: texture_2d<f32>;\n@group(0) @binding(1) var u_sampler: sampler;\n@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(fx.gain,0.0,0.0,1.0);\n}\n", "width": 321, "height": 239}, "DFX_010_wgsl_temporal": {"language": 2, "kind": "temporal", "source": "@dynamicfx 1\n@graph\npass acc: prev -> output\n@end\n@pass acc\n// acceptance reference\n// @window 4\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(textureSample(u_input,u_sampler,v_uv).rgb+vec3<f32>(0.1),1.0);\n}\n@endpass\n", "width": 321, "height": 239}, "DFX_010_wgsl_invalid": {"language": 2, "kind": "invalid", "source": "// acceptance reference\n@fragment fn main( { definitely invalid WGSL\n", "width": 321, "height": 239}, "DFX_010_wgsl_layer": {"language": 2, "kind": "layer", "source": "@dynamicfx 1\n@graph\npass main: input, side -> output\n@end\n@pass main\n// acceptance reference\n// @param side label:\"Side Layer\" hint:layer\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@group(0) @binding(3) var u_extra: texture_2d<f32>;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nlet side=textureSample(u_extra,u_sampler,v_uv);let own=textureSample(u_input,u_sampler,v_uv);return mix(own,vec4<f32>(side.rgb,1.0),side.a);\n}\n@endpass\n", "width": 160, "height": 120}, "DFX_010_wgsl_gradient": {"language": 2, "kind": "gradient", "source": "@dynamicfx 1\n@graph\npass main: input, ramp -> output\n@end\n@pass main\n// acceptance reference\n// @param ramp label:\"Ramp\" hint:gradient\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@group(0) @binding(3) var u_extra: texture_2d<f32>;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nreturn vec4<f32>(textureSample(u_extra,u_sampler,vec2<f32>(v_uv.x,0.5)).rgb,1.0);\n}\n@endpass\n", "width": 160, "height": 120}, "DFX_010_wgsl_path": {"language": 2, "kind": "path", "source": "@dynamicfx 1\n@graph\npass main: input, outline -> output\n@end\n@pass main\n// acceptance reference\n// @param outline label:\"Outline\" hint:path\nstruct FxUniforms {\nu_resolution: vec2<f32>,\nu_time: f32,\nu_frame: f32,\n\n}\n@@group(0) @binding(0) var u_input: texture_2d<f32>;\n@@group(0) @binding(1) var u_sampler: sampler;\n@@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@@group(0) @binding(3) var u_extra: texture_2d<f32>;\n@@fragment\nfn main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {\nlet sz=textureDimensions(u_extra,0);let p=textureLoad(u_extra,vec2<i32>(0),0);return vec4<f32>(p.xy,f32(sz.x)/16.0,1.0);\n}\n@endpass\n", "width": 160, "height": 120}};
var PROJECT="<REPO>/scripts/out/010/ae2026/native-wgsl-010.aep";
var QUEUE_PROJECT="<REPO>/scripts/out/010/ae2026/native-wgsl-010-aerender.aep";
var OUT="<REPO>/scripts/out/010/ae2026";
var TAG="010c-ui-fresh-undo";

// These single-parameter fixtures can expose either their authored label or
// the legal static pool label. Resolve the exact union once; never prefer a
// label silently if both rows exist, and never guess an index or prefix.
function named(f,n){var aliases={"Gain":"Float 01","Side Layer":"Layer 01","Outline":"Mask 01"};
var alternate=aliases.hasOwnProperty(n)?aliases[n]:null,found=null;for(var i=1;i<=f.numProperties;i++){var p=f.property(i);
if(p.name===n||(alternate!==null&&p.name===alternate)){if(found)throw Error("Ambiguous property "+n);found=p;}}
if(!found)throw Error("Named property not ready: "+n);return found;}
// Gradient child rows keep their setup names until AE opens their effect UI.
// Both names identify this fixture's one gradient; never guess a row index.
function gradientNamed(f,role){var names=role==="Stops"?["Ramp Stops","Gradient 01 Stops"]:["Ramp "+role,"G01 Stop "+role];
var found=null;for(var i=1;i<=f.numProperties;i++){var p=f.property(i);if(p.name===names[0]||p.name===names[1]){
if(found)throw Error("Ambiguous gradient property "+role);found=p;}}
if(!found)throw Error("Gradient property missing: "+role);return found;}
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

try {var data=(function(){
// Strictly read-only AE payload. No source/probe writes, time/depth changes,
// selection, purge, save, open, or fallback row mutation. Probe expressions
// were fixed by setup; reading their valueAtTime evaluates but never rewrites.
function uiReadAttempt(fn){try{return {ok:true,value:fn()};}catch(e){return {ok:false,error:String(e)};}}
function uiRow(p){
    var r={name:uiReadAttempt(function(){return p.name;}),
        matchName:uiReadAttempt(function(){return p.matchName;}),
        index:uiReadAttempt(function(){return p.propertyIndex;}),
        value:uiReadAttempt(function(){return p.value;}),
        numKeys:uiReadAttempt(function(){return p.numKeys;}),
        valueAtZero:uiReadAttempt(function(){return p.valueAtTime(0,false);}),
        expression:uiReadAttempt(function(){return p.expression;}),
        expressionEnabled:uiReadAttempt(function(){return p.expressionEnabled;}),
        expressionError:uiReadAttempt(function(){return p.expressionError;})};
    // ExtendScript has no reliable public Hidden-flag getter. Include every
    // exact row even when hidden, and retain read errors instead of guessing.
    r.visibility="not inferred; hidden rows are not filtered";
    return r;
}
function uiExactRows(group,names){
    var r={acceptedNames:names,missing:true,ambiguous:false,rows:[],enumerationErrors:[]};
    for(var i=1;i<=group.numProperties;i++){
        try{var p=group.property(i),n=p.name,match=false;
            for(var j=0;j<names.length;j++)if(n===names[j])match=true;
            if(match)r.rows.push(uiRow(p));
        }catch(e){r.enumerationErrors.push({index:i,error:String(e)});}
    }
    r.missing=r.rows.length===0;r.ambiguous=r.rows.length>1;
    // Never silently choose a dynamic label over a distinct static-label row.
    r.unique=r.rows.length===1;
    return r;
}
var uiFound=[];
if(app.project)for(var uiI=1;uiI<=app.project.numItems;uiI++){
    var uiC=app.project.item(uiI);
    if(uiC instanceof CompItem && uiC.name==="DFX_010_ui_defaults")uiFound.push(uiC);
}
if(uiFound.length!==1)return {mode:"ui-fresh-read",compMatches:uiFound.length,
    missing:uiFound.length===0,ambiguous:uiFound.length>1};
var uiComp=uiFound[0],uiInput=uiComp.layer("input"),uiEffects=uiInput.property("ADBE Effect Parade"),uiFx=[];
for(var uiE=1;uiE<=uiEffects.numProperties;uiE++){
    var uiCandidate=uiEffects.property(uiE);
    if(uiCandidate.matchName==="DynamicFx")uiFx.push(uiCandidate);
}
if(uiFx.length!==1)return {mode:"ui-fresh-read",comp:uiComp.name,effectMatches:uiFx.length,
    missing:uiFx.length===0,ambiguous:uiFx.length>1};
var uiF=uiFx[0];
var uiResult={mode:"ui-fresh-read",comp:uiComp.name,time:uiComp.time,
    depth:app.project.bitsPerChannel,resolution:uiComp.resolutionFactor,
    project:app.project.file?app.project.file.fsName:null,dirty:app.project.dirty,
    readSource:uiExactRows(uiF,["Source (use expression)"]),
    readState:uiExactRows(uiF,["State Token (internal)"]),
    readPlan:uiExactRows(uiF,["Plan Token (internal)"]),
    readLanguage:uiExactRows(uiF,["Language"]),
    gain:uiExactRows(uiF,["Fresh Gain","Float 01"]),
    color:uiExactRows(uiF,["Fresh Color","Color 01"]),
    alpha:uiExactRows(uiF,["Fresh Color A","Float 02"]),
    angle:uiExactRows(uiF,["Fresh Angle","Angle 01"]),
    probePoint:[160.5,119.5],probeTime:0,pixels:[],probeRows:[]};
var uiProbeLayer=uiReadAttempt(function(){return uiComp.layer("DFX_UI_FIXED_PROBE");});
if(!uiProbeLayer.ok||!uiProbeLayer.value){uiResult.probeError=uiProbeLayer.ok?"probe layer missing":uiProbeLayer.error;}
else {
    var uiPG=uiProbeLayer.value.property("ADBE Effect Parade");
    var uiProbeNames=["DFX_UI_R","DFX_UI_G","DFX_UI_B","DFX_UI_A"];
    for(var uiCh=0;uiCh<4;uiCh++){
        var uiMatches=[];
        for(var uiP=1;uiP<=uiPG.numProperties;uiP++){
            var uiPE=uiPG.property(uiP);if(uiPE.name===uiProbeNames[uiCh])uiMatches.push(uiPE);
        }
        if(uiMatches.length!==1){uiResult.pixels.push(null);uiResult.probeRows.push({name:uiProbeNames[uiCh],matches:uiMatches.length});continue;}
        var uiScalar=uiMatches[0].property("ADBE Slider Control-0001");
        var uiValue=uiReadAttempt(function(){return uiScalar.valueAtTime(0,false);});
        uiResult.pixels.push(uiValue.ok?uiValue.value:null);
        uiResult.probeRows.push({name:uiProbeNames[uiCh],sample:uiValue,property:uiRow(uiScalar)});
    }
}
return uiResult;

})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("<REPO>/scripts/out/010/ae2026/010c-ui-fresh-undo.json");f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);})();