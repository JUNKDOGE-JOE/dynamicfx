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
var c=comp("DFX_aerender_fresh");var s=state(c);if(s.token%4!==1)throw Error("Fresh not ready");
app.project.bitsPerChannel=8;app.purge(PurgeTarget.ALL_CACHES);
var rq=app.project.renderQueue;while(rq.numItems>0)rq.item(1).remove();
var i=rq.items.add(c);i.timeSpanStart=0;i.timeSpanDuration=1/25;
var om=i.outputModule(1);om.applyTemplate("Photoshop");om.file=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/fresh_[#####].psd");
app.project.save(new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026"+"/native-aerender-fresh.aep"));return s;
})();var res={ok:true,data:data};}catch(e){var res={ok:false,error:String(e),line:e.line};}
var f=new File("/Users/junk_doge/Documents/DynamicFX/scripts/out/macos/ae2026/fresh-queue.json");f.encoding='UTF-8';if(!f.open('w'))throw Error(f.error);f.write(encode(res));f.close();return encode(res);})();