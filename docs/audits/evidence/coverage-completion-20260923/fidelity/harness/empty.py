import ast,json,os,re,sys,time
from pathlib import Path
import numpy as np
from PIL import Image
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix();project=(out/'full.aep').resolve().as_posix()
tree=ast.parse((out/'matrix.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'matrix-functions','exec'))
targets='var comps=[app.project.itemByID(1),app.project.itemByID(30),app.project.itemByID(14)],ids=[13,44,26],targets=[];for(var n=0;n<comps.length;n++){var found=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j).id===ids[n])found=comps[n].layer(j);if(!found)throw Error("Missing owner");targets.push(found);}'
reset('empty-current')
edit('empty-current','for(var n=0;n<targets.length;n++)targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(0);')
rows=[]
for d in [8,16,32]:
    images={}
    for kind,comp in [('reference',14),('ordinary',1),('adjustment',30),('disabled',30)]:
        name=f'empty-{kind}-{d}';dest=(out/(name+'.png')).resolve()
        execute(name,targets+'targets[1].enabled='+('false' if kind=='disabled' else 'true')+';app.project.bitsPerChannel='+str(d)+';app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID('+str(comp)+').saveFrameToPng(0,new File('+json.dumps(dest.as_posix())+'));return "rendered";')
        for _ in range(80):
            if dest.exists() and dest.read_bytes().endswith(b'IEND\xaeB`\x82'):break
            time.sleep(.25)
        images[kind]=np.array(Image.open(dest).convert('RGBA'))
    row={'depth':d,'ordinary_nonzero_alpha':int(np.count_nonzero(images['ordinary'][:,:,3])),'reference_nonzero_alpha':int(np.count_nonzero(images['reference'][:,:,3])),'adjustment_disabled_differences':int(np.count_nonzero(images['adjustment']!=images['disabled']))}
    row['status']='PASS' if all(row[k]==0 for k in ['ordinary_nonzero_alpha','reference_nonzero_alpha','adjustment_disabled_differences']) else 'FAIL';rows.append(row)
    (out/'empty-summary.json').write_text(json.dumps(rows,indent=2),encoding='utf-8');print(json.dumps(row),flush=True)
    assert row['status']=='PASS',row
execute('empty-restored',targets+'targets[1].enabled=true;app.project.save();return "saved";')
