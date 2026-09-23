import json, sys, time
from pathlib import Path
sys.path.insert(0, r'<repository>/spike/host-outline')
from mcp_client import Client
out = Path(__file__).parent
client = Client()
prefix = '''if(!app.project.file||!(/^(full|lifecycle)\\.aep$/.test(app.project.file.name)))throw Error("Unexpected project");
var c=app.project.itemByID(1),o=null,r=null;for(var i=1;i<=c.numLayers;i++){var l=c.layer(i);if(l.id===13)o=l;var f=l.property("ADBE Effect Parade");if(f.numProperties&&f.property(1).matchName==="DynamicFx Coverage Reader")r=l;}if(!o||!r)throw Error("Missing fixture");
'''
def call(name, code):
    args={'code':'(function(){'+prefix+code+'})()', 'timeout_sec':25}
    result=client.call('ae_exec',args,timeout=35)
    (out/(name+'.json')).write_text(json.dumps({'args':args,'result':result},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(result)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    return json.loads(d['content'])
read='return JSON.stringify({state:o.property("ADBE Effect Parade").property(1).property(427).value,reader:r.id,position:r.position.value,layerCount:c.numLayers});'
if sys.argv[1]=='before':
    print(call('before-baseline', 'app.project.save(new File('+json.dumps((out/'lifecycle.aep').as_posix())+'));'+read))
    print(call('before-move', 'r.locked=false;r.position.setValue([435,300]);r.locked=true;'+read))
    time.sleep(3)
    print(call('before-idle',read))
    print(call('before-restore','r.locked=false;r.position.setValue([400,300]);r.locked=true;'+read))
    time.sleep(3)
    print(call('before-restored','app.project.save();'+read))
elif sys.argv[1]=='tamper':
    cases=[
        ('position','r.position.setValue([435,300]);','r.position.setValue([400,300]);'),
        ('anchor','r.anchorPoint.setValue([390,300]);','r.anchorPoint.setValue([400,300]);'),
        ('scale','r.scale.setValue([100,90]);','r.scale.setValue([100,100]);'),
        ('rotation','r.rotation.setValue(8);','r.rotation.setValue(0);'),
        ('opacity','r.opacity.setValue(95);','r.opacity.setValue(100);'),
        ('expression','r.position.expression="value";','r.position.expression="";'),
        ('keyframe','r.position.setValueAtTime(0,[400,300]);r.position.setValueAtTime(1,[400,300]);','while(r.position.numKeys)r.position.removeKey(1);'),
        ('separated','r.position.dimensionsSeparated=true;','r.position.dimensionsSeparated=false;'),
        ('start','r.startTime=0.25;','r.startTime=0;r.inPoint=0;r.outPoint=2;'),
        ('stretch','r.stretch=90;','r.stretch=100;r.inPoint=0;r.outPoint=2;'),
        ('inpoint','r.inPoint=0.25;','r.inPoint=0;'),
        ('outpoint','r.outPoint=1.5;','r.outPoint=2;'),
        ('disabled','r.property("ADBE Effect Parade").property(1).enabled=false;','r.property("ADBE Effect Parade").property(1).enabled=true;'),
        ('marker','r.source.comment="externally modified";','r.source.comment="DynamicFX coverage reader source; schema=1";'),
        ('mask','r.property("ADBE Mask Parade").addProperty("ADBE Mask Atom");','r.property("ADBE Mask Parade").property(1).remove();'),
        ('parent','r.parent=o;','r.parent=null;r.position.setValue([400,300]);'),
        ('adjustment','r.adjustmentLayer=true;','r.adjustmentLayer=false;'),
        ('visible','r.enabled=true;','r.enabled=false;'),
    ]
    results=[]
    baseline=call('after-baseline',read)
    assert baseline['state']>=4, baseline
    for name,change,restore in cases:
        call(name+'-change','r.locked=false;'+change+'r.locked=true;'+read)
        time.sleep(2)
        changed=call(name+'-checked',read)
        call(name+'-restore','r.locked=false;'+restore+'r.locked=true;'+read)
        time.sleep(2)
        restored=call(name+'-recovered',read)
        row={'case':name,'changed':changed,'restored':restored,'status':'PASS' if changed['state']==3 and restored['state']>=4 and changed['reader']==restored['reader']==baseline['reader'] else 'FAIL'}
        results.append(row);print(json.dumps(row),flush=True)
        (out/'tamper-summary.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
        if row['status']!='PASS':raise AssertionError(row)
    print(call('after-save','app.project.save();'+read))
