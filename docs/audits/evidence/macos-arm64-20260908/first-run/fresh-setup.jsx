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

try {var data=(function(){
var c=app.project.items.addComp("DFX_aerender_fresh",321,239,1,1,25);
var l=c.layers.addSolid([0,0,0],"input",321,239,1,1);
var f=l.property("ADBE Effect Parade").addProperty("DynamicFx");source(f,"@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n@pass gen\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){outColor=vec4(v_uv.x,v_uv.y,0.375,1.0);}\n@endpass\n@pass inv1\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n@pass inv2\n#version 450\n\n\nlayout(location=0) in vec2 v_uv;\nlayout(location=0) out vec4 outColor;\nlayout(set=0,binding=0) uniform texture2D u_input;\nlayout(set=0,binding=1) uniform sampler u_sampler;\nlayout(set=0,binding=2) uniform FxUniforms {\nvec2 u_resolution; float u_time; float u_frame;\n\n};\nvoid main(){vec4 c=texture(sampler2D(u_input,u_sampler),v_uv);outColor=vec4(1.0-c.rgb,1.0);}\n@endpass\n");return state(c);
})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026/fresh-setup.json");f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);})();