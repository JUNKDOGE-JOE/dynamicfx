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
app.project.bitsPerChannel=8;app.purge(PurgeTarget.ALL_CACHES);var c=comp("DFX_final_export");if(state(c).token%4!==1)throw Error("Fresh source not published");var made=[];for(var i=1;i<=app.project.renderQueue.numItems;i++)app.project.renderQueue.item(i).render=false;
for(var n=0;n<2;n++){var c=comp(n===0?"DFX_final_export":"DFX_siri");for(var j=0;j<3;j++){var res=["Full","Half","Quarter"][j];var q=app.project.renderQueue.items.add(c);q.setSettings({"Resolution":res,"Quality":"Best","Color Depth":"Current Settings"});q.timeSpanStart=n===0?0:1;q.timeSpanDuration=1/25;var om=q.outputModule(1);om.applyTemplate("Photoshop");om.file=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/final_"+(n===0?"ramp":"siri")+"_"+res.toLowerCase()+"_[#####].psd");made.push({index:q.index,comp:c.name,resolution:q.getSettings(GetSettingsFormat.STRING)["Resolution"],output:om.file.fsName,start:q.timeSpanStart,duration:q.timeSpanDuration});}}
app.project.save(new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/final-aerender.aep"));return {queues:made,project:app.project.file.fsName,state:state(comp("DFX_final_export"))};
})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026/final-queue.json");f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);})();