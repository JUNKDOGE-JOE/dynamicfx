import json,time
from host import call,out
prefix='if(!app.project.file||decodeURI(app.project.file.name)!=="Parameter acceptance.aep")throw Error("Unexpected project");'
find='var c=null;for(var i=1;i<=app.project.numItems;i++)if(app.project.item(i).name==="1-percent")c=app.project.item(i);var e=c.layer("Material").property("ADBE Effect Parade").property(1),v=e.property(11);'
call('percent-keyframes','(function(){'+prefix+find+'v.setValueAtTime(0,25);v.setValueAtTime(1,75);e.property(3).expression=e.property(3).expression.replace("hint:percent ","");return JSON.stringify({keys:v.numKeys,v0:v.keyValue(1),v1:v.keyValue(2)});})()')
time.sleep(2)
body=prefix+find+'var p=c.layer("Sampler").property("ADBE Effect Parade").property(1).property(1),r={keys:v.numKeys,v0:v.keyValue(1),v1:v.keyValue(2),samples:[]};app.purge(PurgeTarget.ALL_CACHES);for(var j=0;j<3;j++){var t=j/2;p.expression="thisComp.layer(\\\"Material\\\").sampleImage([32,32],[0.49,0.49],true,"+t+")[0]";r.samples.push(p.value);}return JSON.stringify(r);'
r=call('percent-hint-removed','(function(){'+body+'})()');assert r['keys']==2 and r['samples']==[.25,.5,.75],r
call('percent-hint-restored','(function(){'+prefix+find+'e.property(3).expression=e.property(3).expression.replace("min:0 max:100","hint:percent min:0 max:100");app.project.save();return JSON.stringify("saved");})()')
time.sleep(2)
call('parameter-reopen','(function(){'+prefix+'var f=app.project.file;app.project.save();app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);app.open(f);return JSON.stringify({opened:true,items:app.project.numItems});})()')
time.sleep(2)
r=call('percent-after-reopen','(function(){'+body+'})()');assert r['keys']==2 and r['samples']==[.25,.5,.75],r
(out/'lifecycle-summary.json').write_text(json.dumps({'status':'PASS','keys_preserved':2,'samples':r['samples'],'removed_restored_hint':True,'reopened':True},indent=2))
