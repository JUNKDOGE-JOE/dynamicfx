import ast,json,sys,time
from pathlib import Path
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
out=Path(__file__).parent;client=Client()
t=ast.parse((out/'mfr-prepare.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'functions','exec'))
for depth in [8,16,32]:
    for kind in ['ordinary','adjustment']:
        for mode in ['off','on']:
            label=f'mfr-{kind}-{depth}-{mode}';base=(out/f'mfr-{kind}-{depth}.aep').resolve().as_posix()
            call(label+'-open','if(!app.project.file||!/^mfr-/.test(app.project.file.name))throw Error("Unexpected project");app.project.save();app.open(new File('+json.dumps(base)+'));return "opened";')
            time.sleep(2)
            call(label+'-save','app.project.save(new File('+json.dumps((out/(label+'.aep')).resolve().as_posix())+'));var c=app.project.renderQueue.item(1).comp,states=[];for(var i=1;i<=c.numLayers;i++){var fx=c.layer(i).property("ADBE Effect Parade");if(fx.numProperties&&fx.property(1).matchName==="DynamicFx")states.push(fx.property(1).property(427).value);}return JSON.stringify({states:states});')
