import ast,json,subprocess,sys,time
from pathlib import Path
import numpy as np
from PIL import Image
out=Path(__file__).parent;root=out.resolve().parents[2]
sys.path.insert(0,r'<repository>/spike/host-outline');from mcp_client import Client
sys.path.insert(0,str(root/'scripts/m5'));from check_deep import Psd
client=Client();project=(out/'Liquid Glass.aep').resolve().as_posix()
t=ast.parse((out/'ae-demo.py').read_text(encoding='utf-8'));exec(compile(ast.Module(body=[n for n in t.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'demo-functions','exec'))
if sys.argv[1]=='prepare':
    call('matrix-save','app.project.save();return JSON.stringify("saved");')
    for depth in [8,16,32]:
        target=(out/f'matrix-{depth}.aep').resolve().as_posix()
        code='if(!app.project.file||!(decodeURI(app.project.file.name)==="Liquid Glass.aep"||/^matrix-/.test(app.project.file.name)))throw Error("Unexpected project");var c=app.project.itemByID(1);while(app.project.renderQueue.numItems)app.project.renderQueue.item(1).remove();app.project.bitsPerChannel='+str(depth)+';var rows=[];'
        for res,ds in [('Full',1),('Half',2),('Quarter',4)]:
            for seconds in [0,3]:
                path=(out/f'ae-{depth}-{ds}-{seconds}_[#####].psd').resolve().as_posix()
                code+='var q=app.project.renderQueue.items.add(c);q.setSettings({"Resolution":'+json.dumps(res)+',"Quality":"Best","Color Depth":"Current Settings"});q.timeSpanStart='+str(seconds)+';q.timeSpanDuration=1/25;q.outputModule(1).applyTemplate("Photoshop");q.outputModule(1).file=new File('+json.dumps(path)+');rows.push({resolution:'+str(ds)+',time:'+str(seconds)+'});'
        code+='app.project.save(new File('+json.dumps(target)+'));return JSON.stringify(rows);'
        call('matrix-'+str(depth)+'-prepare',code,False)
    call('matrix-restore','if(app.project.file.name!=="matrix-32.aep")throw Error("Unexpected project");app.open(new File('+json.dumps(project)+'));return JSON.stringify("restored");',False)
elif sys.argv[1]=='render':
    reports=[]
    for depth in [8,16,32]:
        args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',str((out/f'matrix-{depth}.aep').resolve()),'-mfr','ON','75']
        with (out/f'matrix-{depth}.stdout.log').open('wb') as stdout,(out/f'matrix-{depth}.stderr.log').open('wb') as stderr:
            p=subprocess.run(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW,timeout=180)
        assert p.returncode==0,(depth,p.returncode)
        for ds in [1,2,4]:
            for seconds in [0,3]:
                paths=list(out.glob(f'ae-{depth}-{ds}-{seconds}_*.psd'));assert len(paths)==1,paths
                a=Psd(str(paths[0]));assert [a.width,a.height]==[1440//ds,900//ds]
                pixels=np.stack([np.frombuffer(v,dtype='uint8') for v in a.planes],axis=-1).reshape(a.height,a.width,a.channels)
                assert pixels[:,:,:3].max()>200 and np.unique(pixels[:,:,:3]).size>150
                Image.fromarray(pixels,'RGBA').save(out/f'ae-{depth}-{ds}-{seconds}.png')
                reports.append({'project_depth':depth,'resolution':ds,'time':seconds,'physical_size':[a.width,a.height],'export_depth':a.depth,'status':'PASS'})
        print(json.dumps({'depth':depth,'rendered':6}),flush=True)
    (out/'ae-matrix-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
