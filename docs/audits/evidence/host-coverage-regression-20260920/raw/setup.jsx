(function(){
if(!app.project.file || app.project.file.parent.name!=="regression-20260920")throw Error("Wrong fixture");
var names=["triangle","rectangle","ellipse","curve","half","masked","transformed","hole"],rows=[];
for(var i=0;i<names.length;i++){
 var name=names[i],c=app.project.items.addComp("HS_auto_rt_"+name,128,96,1,2,24),l=c.layers.addShape();l.name="renamed host "+name;l.adjustmentLayer=true;
 l.property("ADBE Transform Group").property("ADBE Anchor Point").setValue([0,0]);l.property("ADBE Transform Group").property("ADBE Position").setValue([0,0]);
 var g=l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");g.name="renamed geometry";var v=g.property("ADBE Vectors Group");
 if(name==="rectangle"||name==="ellipse") {var sh=v.addProperty(name==="rectangle"?"ADBE Vector Shape - Rect":"ADBE Vector Shape - Ellipse");sh.property(name==="rectangle"?"ADBE Vector Rect Size":"ADBE Vector Ellipse Size").setValue([70,60]);sh.property(name==="rectangle"?"ADBE Vector Rect Position":"ADBE Vector Ellipse Position").setValue([64,48]);}
 else {var sh=v.addProperty("ADBE Vector Shape - Group"),s=new Shape();s.vertices=[[20,15],[108,15],[64,80]];s.inTangents=[[0,0],[0,0],[0,0]];s.outTangents=[[0,0],[0,0],[0,0]];s.closed=true;if(name==="curve"){s.outTangents[0]=[20,-12];s.inTangents[2]=[35,0];}sh.property("ADBE Vector Shape").setValue(s);}
 var fill=v.addProperty("ADBE Vector Graphic - Fill");fill.property("ADBE Vector Fill Color").setValue([1,1,1]);fill.property("ADBE Vector Fill Opacity").setValue(name==="half"?50:100);
 if(name==="transformed"){g.property("ADBE Vector Transform Group").property("ADBE Vector Rotation").setValue(12);g.property("ADBE Vector Transform Group").property("ADBE Vector Scale").setValue([80,90]);}
 if(name==="masked"||name==="hole"){var m=l.property("ADBE Mask Parade").addProperty("ADBE Mask Atom"),ms=new Shape();ms.vertices=[[50,35],[78,35],[78,60],[50,60]];ms.closed=true;m.property("ADBE Mask Shape").setValue(ms);m.maskMode=name==="hole"?MaskMode.SUBTRACT:MaskMode.ADD;}
 var f=l.property("ADBE Effect Parade").addProperty("DynamicFx Host Shape Probe");f.property(4).setValue(true);f.property(7).setValue(true);
 var bg=c.layers.addSolid([0.2,0.3,0.4],"background",128,96,1,2);bg.moveToEnd();rows.push({name:name,id:c.id});
}
return JSON.stringify({rows:rows,version:app.version});})()