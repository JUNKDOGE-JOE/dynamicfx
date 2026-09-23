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
reports=[];references={}
reset('more')
edit('nested-group','for(var n=0;n<targets.length;n++){var t=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vector Transform Group");t.property("ADBE Vector Anchor").setValue([400,300]);t.property("ADBE Vector Position").setValue([415,310]);t.property("ADBE Vector Scale").setValue([-92,107]);t.property("ADBE Vector Rotation").setValue(11);t.property("ADBE Vector Skew").setValue(14);t.property("ADBE Vector Skew Axis").setValue(23);}')
for depth in [8,16,32]:check('negative-group-skew',depth)
edit('parent','for(var n=0;n<targets.length;n++){var p=comps[n].layers.addNull();p.name="Regression parent";p.property("ADBE Transform Group").property("ADBE Position").setValue([40,-15]);p.property("ADBE Transform Group").property("ADBE Scale").setValue([95,103]);p.property("ADBE Transform Group").property("ADBE Rotate Z").setValue(-7);targets[n].setParentWithJump(p);}')
for depth in [8,16,32]:check('parent',depth)
reset('expression')
expression='createPath([[200+25*time,150],[600-30*time,150+10*time],[400,500-35*time]],[[0,0],[-30,-15],[20,0]],[[20,-10],[20,30],[0,0]],true)'
edit('expression','for(var n=0;n<targets.length;n++){var p=targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Shape - Group").property("ADBE Vector Shape");p.expression='+json.dumps(expression)+';p.expressionEnabled=true;}')
for depth in [8,16,32]:
 for seconds in [1,.24]:check('expression',depth,seconds)
(out/'more-summary.json').write_text(json.dumps(reports,indent=2),encoding='utf-8')
reset('empty')
edit('empty','for(var n=0;n<targets.length;n++)targets[n].property("ADBE Root Vectors Group").property(1).property("ADBE Vectors Group").property("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Opacity").setValue(0);')
empty=[]
for depth in [8,16,32]:
 for kind,comp in [('reference',14),('ordinary',1),('adjustment',30)]:
  name=f'empty-{kind}-{depth}';dest=(out/(name+'.png')).resolve()
  execute(name,targets+'for(var n=0;n<targets.length;n++){var fx=targets[n].property("ADBE Effect Parade");for(var j=1;j<=fx.numProperties;j++)if(fx.property(j).matchName==="DynamicFx Host Shape Probe")fx.property(j).property(2).setValue(1);}app.project.bitsPerChannel='+str(depth)+';app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID('+str(comp)+').saveFrameToPng(0,new File('+json.dumps(dest.as_posix())+'));return "rendered";')
  for _ in range(40):
   if dest.exists() and dest.read_bytes().endswith(b'IEND\xaeB`\x82'):break
   time.sleep(.25)
  a=np.array(Image.open(dest).convert('RGBA'));ok=bool(np.all(a==[0,0,255,255])) if kind=='adjustment' else bool(np.all(a[:,:,3]==0))
  row={'case':'empty','kind':kind,'depth':depth,'status':'PASS' if ok else 'FAIL'};empty.append(row);print(json.dumps(row),flush=True)
  (out/'empty-summary.json').write_text(json.dumps(empty,indent=2),encoding='utf-8')
execute('more-save','app.project.save();return "saved";')
