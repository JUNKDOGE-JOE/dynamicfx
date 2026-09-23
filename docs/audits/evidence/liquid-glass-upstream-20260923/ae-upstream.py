import json,sys,time,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=Path(__file__).parent
sys.path.insert(0,str(root.parents[2]/'spike/host-outline'));from mcp_client import Client
client=Client();source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
project=(out/'Upstream Glass.aep').resolve().as_posix()
def call(name,body,guard=True):
    prefix='if(!app.project.file||decodeURI(app.project.file.name)!=="Upstream Glass.aep")throw Error("Unexpected project");' if guard else ''
    args={'code':'(function(){'+prefix+body+'})()','timeout_sec':30};r=client.call('ae_exec',args,timeout=40);(out/(name+'.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8');d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    result=json.loads(d['content']);print(json.dumps({'step':name,'data':result},ensure_ascii=True),flush=True);return result
if sys.argv[1]=='prepare':
    call('prepare','if(!app.project.file||decodeURI(app.project.file.name)!=="Liquid Glass.aep")throw Error("Unexpected project");app.project.save(new File('+json.dumps(project)+'));var c=app.project.itemByID(1),rows=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l instanceof ShapeLayer){var fx=l.property("ADBE Effect Parade");if(fx.numProperties!==1||fx.property(1).matchName!=="DynamicFx")throw Error("Unexpected effects");fx.property(1).remove();var e=fx.addProperty("DynamicFx");e.name="Liquid Glass Studio";e.property(3).expression="`"+'+json.dumps(source)+'+"`;0";rows.push(l.id);}}app.project.save();return JSON.stringify(rows);',False)
    time.sleep(3)
    call('ready','var c=app.project.itemByID(1),r=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l instanceof ShapeLayer){var e=l.property("ADBE Effect Parade").property(1);r.push({id:l.id,coverage:e.property(427).value,token:e.property(6).value});}}app.project.save();return JSON.stringify(r);')
elif sys.argv[1]=='queue':
    call('queue','var c=app.project.itemByID(1);if(app.project.renderQueue.numItems)throw Error("Queue not empty");var q=app.project.renderQueue.items.add(c);q.setSettings({"Resolution":"Full","Color Depth":"Current Settings","Quality":"Best"});q.timeSpanStart=0;q.timeSpanDuration=1/25;q.outputModule(1).applyTemplate("Photoshop");q.outputModule(1).file=new File('+json.dumps((out/'hero_[#####].psd').resolve().as_posix())+');app.project.save();return JSON.stringify("queued");')
elif sys.argv[1]=='render':
    args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',project,'-rqindex','1','-mfr','ON','75']
    with (out/'hero-render.log').open('wb') as f:p=subprocess.run(args,stdout=f,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW,timeout=180)
    print(json.dumps({'returncode':p.returncode}));assert p.returncode==0
