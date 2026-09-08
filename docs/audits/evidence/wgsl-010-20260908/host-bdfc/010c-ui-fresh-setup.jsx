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
var TAG="010c-ui-fresh-setup";

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
// Run once through ae_acceptance.py script mode. This only adds the named
// temporary comp; it refuses to touch an existing comp with that exact name.
// COMMON supplies ownProject(), named(), source(), and save010().
ownProject();
var uiName="DFX_010_ui_defaults";
for(var uiI=1;uiI<=app.project.numItems;uiI++){
    var uiExisting=app.project.item(uiI);
    if(uiExisting instanceof CompItem && uiExisting.name===uiName)
        throw Error("Temporary UI fixture already exists; do not retry setup");
}
var uiComp=app.project.items.addComp(uiName,321,239,1,2,25);
var uiInput=uiComp.layers.addSolid([0,0,0],"input",321,239,1,2);
var uiEffect=uiInput.property("ADBE Effect Parade").addProperty("DynamicFx");
var uiLanguage=named(uiEffect,"Language");
if(uiLanguage.value!==1)throw Error("Expected GLSL on fresh effect");
uiLanguage.setValue(1);
var uiInitial="// Paste as the Source expression's backtick string, followed by ;0.\n// Fresh bindings: Float 01=gain, Color 01=tint RGB, Float 02=tint alpha,\n// Angle 01=angle. The angle deliberately tests defaults without changing RGB.\n// @param gain label:\"Fresh Gain\" min:0 max:2 default:0.25\n// @param tint label:\"Fresh Color\" hint:color default:0.25,0.6,1,0.4\n// @param angle label:\"Fresh Angle\" hint:angle default:12.34567\nstruct FxUniforms {\n    u_resolution: vec2<f32>,\n    u_time: f32,\n    u_frame: f32,\n    gain: f32,\n    tint: vec4<f32>,\n    angle: f32,\n}\n@group(0) @binding(2) var<uniform> fx: FxUniforms;\n@fragment\nfn main() -> @location(0) vec4<f32> {\n    return vec4<f32>(fx.tint.rgb * fx.gain, fx.tint.a);\n}\n";
source(uiEffect,uiInitial);
var uiProbe=uiComp.layers.addNull(2);uiProbe.name="DFX_UI_FIXED_PROBE";
var uiProbeNames=["DFX_UI_R","DFX_UI_G","DFX_UI_B","DFX_UI_A"];
var uiExpressions=[];
for(var uiCh=0;uiCh<4;uiCh++){
    var uiControl=uiProbe.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
    uiControl.name=uiProbeNames[uiCh];
    var uiSlider=uiControl.property("ADBE Slider Control-0001");
    var uiExpr='thisComp.layer("input").sampleImage([160.5,119.5],[0.49,0.49],true,0)['+uiCh+']';
    uiSlider.expression=uiExpr;uiExpressions.push(uiExpr);
}
uiComp.openInViewer();
for(var uiLayer=1;uiLayer<=uiComp.numLayers;uiLayer++)uiComp.layer(uiLayer).selected=false;
uiInput.selected=true;
return {mode:"ui-fresh-setup",comp:uiName,language:uiLanguage.value,
    initialSource:uiInitial,probeExpressions:uiExpressions,probePoint:[160.5,119.5],probeTime:0,
    depth:app.project.bitsPerChannel,project:save010(),
    next:"Return to idle; initial WGSL text under GLSL must be E17. Switch actual Language to WGSL to test first-publication defaults in UCP."};

})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("<REPO>/scripts/out/010/ae2026/010c-ui-fresh-setup.json");f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);})();