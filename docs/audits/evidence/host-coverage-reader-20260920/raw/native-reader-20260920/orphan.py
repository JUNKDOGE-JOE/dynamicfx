import json,os,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
path=(out/'native-reader.aep').resolve().as_posix()
def execute(name,code):
    args={'code':code,'timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
    (out/(name+'.result.json')).write_text(json.dumps({'args':args,'result':r},indent=2),encoding='utf-8');d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(d)
    return d['content']
execute('v027-open','(function(){if(app.project.file&&app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");if(!app.project.file)app.open(new File('+json.dumps(path)+'));return app.version;})()')
time.sleep(3)
snapshot='''(function(){if(app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");var c=app.project.itemByID(910),rows=[];for(var i=1;i<=c.numLayers;i++){var l=c.layer(i),fx=l.property("ADBE Effect Parade");if(fx.numProperties&&fx.property(1).matchName==="DynamicFx Host Shape Probe"){var e=fx.property(1);rows.push({id:l.id,role:e.property(11).value,source:e.property(6).value?c.layer(e.property(6).value).id:0});}}return JSON.stringify({layers:c.numLayers,items:app.project.numItems,rows:rows});})()'''
before=json.loads(execute('orphan-before',snapshot));print(before,flush=True)
reports=[]
for name,body,expected_layers,expected_items in [
 ('orphan-delete-owner','app.project.layerByID(913).remove();',before['layers']-2,before['items']-1),
 ('orphan-undo-cleanup','app.executeCommand(16);',before['layers']-1,before['items']),
 ('orphan-undo-owner','app.executeCommand(16);',before['layers'],before['items'])]:
    offset=log.stat().st_size
    execute(name+'-action','(function(){if(app.project.file.name!=="native-reader.aep")throw Error("Unexpected project");'+body+'return "ok";})()')
    time.sleep(3)
    with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
    (out/(name+'.native.log')).write_text(raw,encoding='utf-8')
    current=json.loads(execute(name,snapshot));passed=current['layers']==expected_layers and current['items']==expected_items
    report={'case':name,'status':'PASS' if passed else 'FAIL','observed':current,'expected_layers':expected_layers,'expected_items':expected_items};reports.append(report);print(report,flush=True)
    (out/'orphan-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
    if not passed:raise AssertionError(name)
execute('orphan-save','(function(){app.project.save();return "saved";})()')
