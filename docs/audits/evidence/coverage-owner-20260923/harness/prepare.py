from pathlib import Path
import json
out=Path(__file__).parent
shader='''@dynamicfx 1
@graph
pass main: input, coverage -> output
@end
@pass main
#version 450
// @param coverage hint:coverage
layout(location=0) in vec2 v_uv;
layout(location=0) out vec4 outColor;
layout(set=0,binding=0) uniform texture2D u_in;
layout(set=0,binding=1) uniform sampler u_s;
layout(set=0,binding=2) uniform FxUniforms { vec2 u_resolution; float u_time; float u_frame; };
layout(set=0,binding=3) uniform texture2D u_cov;
void main(){ float a=texelFetch(sampler2D(u_cov,u_s),ivec2(v_uv*vec2(textureSize(sampler2D(u_cov,u_s),0))),0).r; outColor=vec4(0,0,0,a); }
@endpass'''
(out/'coverage.glsl').write_text(shader,encoding='utf-8')
setup='''(function(){
if(app.project.file||app.project.dirty||app.project.numItems!==0)throw Error("Expected empty project");
var found={};for(var i=0;i<app.effects.length;i++){var n=app.effects[i].matchName;if(n==="DynamicFx"||n==="DynamicFx Coverage Reader"||n==="DynamicFx Host Shape Probe")found[n]=true;}
if(!found["DynamicFx Coverage Reader"])throw Error("Reader component not loaded");
app.project.bitsPerChannel=32;
function shape(c){
 var l=c.layers.addShape();l.name="Original shape";
 l.property("ADBE Transform Group").property("ADBE Position").setValue([0,0]);
 var g=l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group"),v=g.property("ADBE Vectors Group");
 var p=v.addProperty("ADBE Vector Shape - Group"),s=new Shape();s.vertices=[[200,150],[600,150],[400,500]];s.inTangents=[[0,0],[0,0],[0,0]];s.outTangents=s.inTangents;s.closed=true;p.property("ADBE Vector Shape").setValue(s);
 var f=v.addProperty("ADBE Vector Graphic - Fill");f.property("ADBE Vector Fill Color").setValue([1,1,1]);f.property("ADBE Vector Fill Opacity").setValue(37.125);
 var m=l.property("ADBE Mask Parade").addProperty("ADBE Mask Atom"),h=new Shape();h.vertices=[[350,230],[450,230],[450,370],[350,370]];h.inTangents=[[0,0],[0,0],[0,0],[0,0]];h.outTangents=h.inTangents;h.closed=true;m.property("ADBE Mask Shape").setValue(h);m.maskMode=MaskMode.SUBTRACT;m.property("ADBE Mask Feather").setValue([11.25,7.75]);m.property("ADBE Mask Offset").setValue(4.25);m.property("ADBE Mask Opacity").setValue(68.75);
 return l;
}
var records=[];
for(var n=0;n<3;n++){
 var name=["COV_production_pixels","COV_reference_pixels","COV_adjustment_pixels"][n],c=app.project.items.addComp(name,800,600,1,2,25);
 if(n===2)c.layers.addSolid([0,0,1],"Background",800,600,1,2);
 var l=shape(c);l.adjustmentLayer=n===2;
 if(n!==1){var e=l.property("ADBE Effect Parade").addProperty("DynamicFx");e.property(3).expression="`"+SHADER+"`;0";e.property(3).expressionEnabled=true;}
 var probe=l.property("ADBE Effect Parade").addProperty("DynamicFx Host Shape Probe");probe.property(2).setValue(21);probe.property(4).setValue(false);
 records.push({name:name,comp:c.id,owner:l.id});
}
app.project.save(new File(PROJECT));return JSON.stringify({records:records,found:found,version:app.version});
})()'''.replace('SHADER',json.dumps(shader)).replace('PROJECT',json.dumps((out/'production-coverage.aep').resolve().as_posix()))
(out/'setup.jsx').write_text(setup,encoding='utf-8')
(out/'inspect.jsx').write_text('''(function(){if(!app.project.file||app.project.file.name!=="production-coverage.aep")throw Error("Unexpected project");var rows=[];for(var i=1;i<=app.project.numItems;i++){var c=app.project.item(i);if(!(c instanceof CompItem))continue;var layers=[];for(var j=1;j<=c.numLayers;j++){var l=c.layer(j),fx=l.property("ADBE Effect Parade"),effects=[];for(var k=1;k<=fx.numProperties;k++){var e=fx.property(k),props=[];if(e.matchName==="DynamicFx")for(var p=e.numProperties-5;p<=e.numProperties;p++){var q=e.property(p);props.push({index:p,name:q.name,value:q.propertyType===PropertyType.PROPERTY?q.value:null});}effects.push({name:e.matchName,props:props});}layers.push({id:l.id,name:l.name,index:l.index,enabled:l.enabled,locked:l.locked,shy:l.shy,effects:effects});}rows.push({id:c.id,name:c.name,layers:layers});}return JSON.stringify(rows);})()''',encoding='utf-8')
