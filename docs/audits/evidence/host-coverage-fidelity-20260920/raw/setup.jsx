(function(){
if(!app.project.file||app.project.file.parent.name!=="engineering-20260920"||app.project.dirty)throw Error("Expected saved engineering fixture");
app.project.save(new File("E:/Code/AePlugin_Dynamicfx/scripts/out/public-push-20260911/scripts/out/fidelity-20260920/host-shape-native.aep"));var rows=[];
var cases=[{name:"curve",id:247},{name:"half",id:262},{name:"masked",id:277},{name:"hole",id:307}];
for(var i=0;i<cases.length;i++){
 var c=app.project.itemByID(cases[i].id).duplicate();c.name="HS_fidelity_"+cases[i].name;var l=c.layer(1),f=l.property("ADBE Effect Parade");while(f.numProperties)f.property(f.numProperties).remove();
 var m=l.property("ADBE Mask Parade");for(var k=m.numProperties;k>=1;k--)if(m.property(k).property("ADBE Mask Shape").expression.indexOf("// DynamicFX internal coverage raster carrier;")===0)m.property(k).remove();
 var ref=c.duplicate();ref.name="HS_fidelity_reference_"+cases[i].name;ref.layer(2).remove();ref.layer(1).adjustmentLayer=false;
 var text=c.layers.addText("");text.name="Sampling oracle (test only)";text.moveToEnd();text.enabled=false;rows.push({name:cases[i].name,id:c.id,reference:ref.id,oracleIndex:text.index});
}
return JSON.stringify({version:app.version,expressionEngine:app.project.expressionEngine,rows:rows});})()