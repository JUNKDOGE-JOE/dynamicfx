import ast,json,os,re,sys,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
base=(out.parent/'coverage-owner-20260923/production-baseline.aep').resolve().as_posix();project=(out/'full.aep').resolve().as_posix()
tree=ast.parse((out/'matrix.py').read_text(encoding='utf-8'))
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),'matrix-functions','exec'))
targets='var comps=[app.project.itemByID(1),app.project.itemByID(30),app.project.itemByID(14)],ids=[13,44,26],targets=[];for(var n=0;n<comps.length;n++){var found=null;for(var j=1;j<=comps[n].numLayers;j++)if(comps[n].layer(j).id===ids[n])found=comps[n].layer(j);if(!found)throw Error("Missing owner");targets.push(found);}'
reset('roi')
points=[[50.5,50.5],[400.5,200.5],[400.5,300.5],[250.5,150.5],[350.5,225.5]]
reports=[]
for depth in [8,16,32]:
    reference,_=render('roi-full','reference',14,26,depth,0,1)
    code=targets+'app.project.bitsPerChannel='+str(depth)+';var points='+json.dumps(points)+',rows=[];for(var n=0;n<targets.length;n++){var fx=targets[n].property("ADBE Effect Parade");for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx Host Shape Probe")fx.property(j).property(2).setValue(1);if(n===1)continue;var control=comps[n].layers.addNull(),e=control.property("ADBE Effect Parade").addProperty("ADBE Slider Control");for(var k=0;k<points.length;k++){e.property(1).expression="thisComp.layer("+targets[n].index+").sampleImage(["+points[k][0]+","+points[k][1]+"],[0.5,0.5],true,0)[3]";e.property(1).expressionEnabled=true;app.purge(PurgeTarget.ALL_CACHES);rows.push({kind:n===0?"ordinary":"reference",point:points[k],alpha:e.property(1).value,error:e.property(1).expressionError});}control.remove();}return JSON.stringify(rows);'
    rows=json.loads(execute('roi-'+str(depth),code))
    for i,point in enumerate(points):
        actual=rows[i]['alpha'];expected=rows[len(points)+i]['alpha'];word=int(reference[int(point[1]),int(point[0])])
        native_value=word/255 if depth==8 else word/32768 if depth==16 else np.array([word],dtype=np.uint32).view(np.float32)[0].item()
        report={'depth':depth,'point':point,'ordinary':actual,'reference':expected,'full_native_alpha':native_value,'status':'PASS' if actual==expected and not rows[i]['error'] and not rows[len(points)+i]['error'] else 'FAIL'}
        reports.append(report);print(json.dumps(report),flush=True)
    (out/'roi-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
execute('roi-save','app.project.save();return "saved";')
