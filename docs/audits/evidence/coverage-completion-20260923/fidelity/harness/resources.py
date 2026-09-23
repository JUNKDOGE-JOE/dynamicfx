import importlib.util,json,sys,time
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=Path(__file__).parent
sys.path.insert(0,str(root.parents[2]/'spike/host-outline'))
from mcp_client import Client
spec=importlib.util.spec_from_file_location('acceptance',root/'scripts/wgsl/ae_acceptance.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
project=(out/'resources.aep').resolve().as_posix();a.PROJECT=Path(project)
fixtures=a.fixtures('coverage-final-shared-dispatch')
common=a.BASE_COMMON+a.EXTRA+'\nvar PROJECT='+json.dumps(project)+';var FIXTURES='+json.dumps(fixtures)+';'
common=common.replace('app.project.file.fsName!==PROJECT',r'app.project.file.fsName.replace(/\\/g,"/")!==PROJECT')
client=Client()
def call(name,code):
    wrapper='(function(){'+common+'try {var data=(function(){'+code+'})();return encode({ok:true,data:data});}catch(e){return encode({ok:false,error:String(e),line:e.line});}})()'
    args={'code':wrapper,'timeout_sec':60};r=client.call('ae_exec',args,timeout=75)
    (out/(name+'.bridge.json')).write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
    d=Client.data(r)
    if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
    data=json.loads(d['content']);(out/(name+'.json')).write_text(json.dumps(data,indent=2),encoding='utf-8')
    if not data.get('ok'):raise RuntimeError(data)
    print(json.dumps({'step':name,'status':'PASS'}),flush=True)
    return data
mode=sys.argv[1]
if mode=='setup':
    call('resources-close','if(!app.project.file||!(/^(full|ffx)\\.aep$/.test(app.project.file.name)))throw Error("Unexpected project");app.project.save();app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);return {closed:true};')
    code=a.code_for('setup').replace('c010("wgsl","uv").openInViewer();','')
    call('resources-setup',code);time.sleep(3);call('resources-state',a.code_for('state'))
elif mode=='capture':
    call('resources-capture',a.code_for('capture'))
    call('resources-none',a.code_for('resources',phase='none'))
elif mode=='assign':
    call('resources-assign',a.code_for('resources',phase='assign'));time.sleep(2)
    call('resources-assigned',a.code_for('resources',phase='assigned'))
    call('resources-keyframes',a.code_for('keyframes'))
elif mode=='assign-expressions':
    code=a.code_for('resources',phase='assign').replace('v.first.setValue([1,0,0,1]);v.second.setValue([0,0,1,1]);','v.first.expression="[1,0,0,1]";v.first.expressionEnabled=true;v.second.expression="[0,0,1,1]";v.second.expressionEnabled=true;')
    call('resources-assign-expressions',code);time.sleep(2)
    call('resources-assigned',a.code_for('resources',phase='assigned'))
    call('resources-keyframes',a.code_for('keyframes'))
elif mode=='reopen':
    call('resources-save',a.code_for('save'));call('resources-reopen',a.code_for('reopen'));time.sleep(3)
    call('resources-reopened-state',a.code_for('state'));call('resources-reopened-assigned',a.code_for('resources',phase='assigned'));call('resources-reopened-keys',a.code_for('keys-read'))
elif mode=='postcancel':
    call('resources-postcancel-open','if(!app.project.file||app.project.file.name!=="full.aep")throw Error("Unexpected project");app.project.save();app.open(new File(PROJECT));return {opened:true};');time.sleep(3)
    call('resources-postcancel-capture',a.code_for('capture'))
    call('resources-postcancel-clear','for(var i=0;i<2;i++){var lang=["glsl","wgsl"][i];named(fx(c010(lang,"layer")),"Side Layer").setValue(0);named(fx(c010(lang,"path")),"Outline").setValue(0);var g=fx(c010(lang,"gradient"));gradientNamed(g,"01 Color").setValue([0,0,0,1]);gradientNamed(g,"02 Color").setValue([1,1,1,1]);}return {cleared:true};')
    call('resources-postcancel-none',a.code_for('resources',phase='none'))
    call('resources-postcancel-assign',a.code_for('resources',phase='assign'));time.sleep(2)
    call('resources-postcancel-assigned',a.code_for('resources',phase='assigned'))
    call('resources-postcancel-keys',a.code_for('keys-read'))
