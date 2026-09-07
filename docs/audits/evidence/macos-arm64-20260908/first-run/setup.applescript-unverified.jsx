(function(){
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

try {var data=(function(){

if(app.project && (app.project.dirty || app.project.numItems>0))throw Error("Existing project: refusing replacement");
app.newProject(); app.project.bitsPerChannel=32; app.project.workingSpace="";
var sources={"DFX_gradient": "#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(v_uv.x,v_uv.y,0.25,1.0);}\n", "DFX_multi": "@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n@pass gen\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(v_uv.x,v_uv.y,0.25,1.0);}\n@endpass\n@pass inv1\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n@pass inv2\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n", "DFX_param": "#version 450\n// @param gain label:\"Gain\" min:0 max:2 default:0.25\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\nfloat gain;\n};\nvoid main(){outColor=vec4(gain,0.0,0.0,1.0);}\n", "DFX_hdr": "#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(2.0,-0.5,0.123456,1.0);}\n", "DFX_temporal": "@dynamicfx 1\n@graph\npass acc: prev -> output\n@end\n@pass acc\n#version 450\n// @window 4\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(texture(sampler2D(u_input,u_sampler),v_uv).rgb+vec3(0.1),1.0);}\n@endpass\n", "DFX_invalid": "#version 450\nthis is deliberately invalid shader source\n"};
for(var n in sources) {var c=app.project.items.addComp(n,320,240,1,2,25);
var l=c.layers.addSolid([0.05,0.4,0.1],"input",320,240,1,2);
var f=l.property("ADBE Effect Parade").addProperty("DynamicFx");
if(prop(f,"Language").value!==1)throw Error("Language is not GLSL");source(f,sources[n]);
var probe=c.layers.addNull(2);probe.name="probe";
probe.property("ADBE Effect Parade").addProperty("ADBE Slider Control");}
comp("DFX_gradient").openInViewer();
return {version:app.version,build:app.buildNumber,os:$.os,states:states()};

})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026/setup.json");f.encoding='UTF-8';f.open('w');f.write(JSON.stringify(res));f.close();return JSON.stringify(res);})();