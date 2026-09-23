import json,sys,time
from pathlib import Path
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
out=Path(__file__).parent;client=Client()
def call(name,code):
    args={'code':'(function(){'+code+'})()','timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
    (out/(name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    print(json.dumps({'step':name,'data':d['content']},ensure_ascii=True),flush=True)
    return d['content']
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix()
call('mfr-open','if(!app.project.file||app.project.file.name!=="lifecycle.aep")throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));app.project.save(new File('+json.dumps((out/'mfr-base.aep').resolve().as_posix())+'));return "opened";')
time.sleep(3)
call('mfr-animate','''if(app.project.file.name!=="mfr-base.aep")throw Error("Unexpected project");
var ids=[1,30,14],owners=[13,44,26];for(var n=0;n<ids.length;n++){var c=app.project.itemByID(ids[n]);for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");for(var j=fx.numProperties;j>=1;j--)if(fx.property(j).matchName==="DynamicFx Host Shape Probe")fx.property(j).remove();if(l.id===owners[n]){var v=l.property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group"),p=v.property("ADBE Vector Shape - Group").property("ADBE Vector Shape"),a=p.value,b=p.value;b.vertices=[[200,500],[600,500],[400,150]];p.setValueAtTime(0,a);p.setValueAtTime(1.88,b);v.property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValueAtTime(0,37.125);v.property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValueAtTime(1.88,83.375);}}}return "animated, probes removed";''')
time.sleep(3)
for depth in [8,16,32]:
    for kind,comp in [('ordinary',1),('adjustment',30)]:
        name=f'mfr-{kind}-{depth}'
        project=(out/(name+'.aep')).resolve().as_posix();output=(out/(name+'_[#####].psd')).resolve().as_posix()
        call(name+'-prepare','''if(!/^mfr-/.test(app.project.file.name))throw Error("Unexpected project");while(app.project.renderQueue.numItems)app.project.renderQueue.item(1).remove();app.project.bitsPerChannel='''+str(depth)+''';var q=app.project.renderQueue.items.add(app.project.itemByID('''+str(comp)+'''));q.timeSpanStart=0;q.timeSpanDuration=48/25;var om=q.outputModule(1);om.applyTemplate("Photoshop");om.file=new File('''+json.dumps(output)+''');app.project.save(new File('''+json.dumps(project)+'''));return JSON.stringify({depth:app.project.bitsPerChannel,frames:48,comp:q.comp.id});''')
call('mfr-preferences','return JSON.stringify({disk:app.preferences.havePref("Disk Cache Controls","Enable Disk Cache")?app.preferences.getPrefAsLong("Disk Cache Controls","Enable Disk Cache"):null});')
