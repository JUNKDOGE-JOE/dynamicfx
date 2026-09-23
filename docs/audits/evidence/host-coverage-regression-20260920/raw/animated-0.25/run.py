import json,os,re,sys,time
from pathlib import Path
from PIL import Image
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
OUT=Path(__file__).resolve().parent
LOG=Path(os.environ["TEMP"])/"dynamicfx-host-shape-probe.log"
client=Client()
def call(label,code):
 args={"code":code,"timeout_sec":35};(OUT/(label+".args.json")).write_text(json.dumps(args),encoding="utf-8")
 t=time.monotonic();r=client.call("ae_exec",args,timeout=45);(OUT/(label+".result.json")).write_text(json.dumps(r),encoding="utf-8");data=Client.data(r)
 if not data.get("ok"):raise RuntimeError(str(data))
 return json.loads(data["content"]),time.monotonic()-t
rows=json.loads(Client.data(json.loads((OUT/"setup.result.json").read_text()))["content"])["rows"]
rows=[r for r in rows if r["name"]=="curve"]
results=[]
for row in rows:
 name=row["name"];before=LOG.stat().st_size
 base=OUT/(name+"-expression.png");ref=OUT/(name+"-expression-reference.png")
 code='(function(){if(!app.project.file||app.project.file.parent.name!=="regression-20260920")throw Error("Wrong fixture");var c=app.project.itemByID(ID),l=c.layer(1),m=l.property("ADBE Mask Parade"),car=null;for(var i=1;i<=m.numProperties;i++){var p=m.property(i).property("ADBE Mask Shape");if(p.expression.indexOf("// DynamicFX internal coverage raster carrier;")===0)car=p;}if(!car)throw Error("Carrier absent");l.property("ADBE Effect Parade").property(1).property(4).setValue(false);var points=car.valueAtTime(0.25,false).vertices,error=car.expressionError;\nvar r=c.duplicate();r.name="HS_ref_rt_NAME";r.layer(2).remove();r.layer(1).adjustmentLayer=false;r.layer(1).property("ADBE Effect Parade").property(1).remove();var rm=r.layer(1).property("ADBE Mask Parade");for(var k=rm.numProperties;k>=1;k--){if(rm.property(k).property("ADBE Mask Shape").expression.indexOf("// DynamicFX internal coverage raster carrier;")===0)rm.property(k).remove();}\nc.saveFrameToPng(0.25,new File(BASE));r.saveFrameToPng(0.25,new File(REF));return JSON.stringify({name:"NAME",id:c.id,reference:r.id,points:points,carrierVertices:points.length,expressionError:error,bpc:app.project.bitsPerChannel});})()'
 code=code.replace("(ID)","("+str(row["id"])+")").replace("NAME",name).replace("BASE",json.dumps(base.as_posix())).replace("REF",json.dumps(ref.as_posix()))
 try:
  data,elapsed=call(name+"-expression",code)
  deadline=time.monotonic()+45
  while (not base.exists() or not ref.exists()) and time.monotonic()<deadline:time.sleep(.25)
  with LOG.open("rb") as f:f.seek(before);raw=f.read()
  (OUT/(name+".native.log")).write_bytes(raw)
  pts=data.pop("points")
  w,h=map(int,pts[1]);pixels=bytearray(w*h)
  for (x,y),(packed,bh) in zip(pts[2::2],pts[3::2]):
   bw=int(packed);alpha=round((packed-bw)*4*255)
   for y2 in range(int(y),int(y+bh)):pixels[y2*w+int(x):y2*w+int(x)+bw]=bytes([alpha])*bw
  im=Image.frombytes("L",(w,h),bytes(pixels));im.save(OUT/(name+"-decoded.png"));expected=Image.open(ref).convert("RGBA").getchannel("A");diff=[abs(a-b) for a,b in zip(pixels,expected.tobytes())]
  result={**data,"status":"PASS" if max(diff)<=1 else "FAIL","max_error_8bit":max(diff),"different_pixels":sum(v!=0 for v in diff),"pixels":len(diff),"route":"expression path readback, not PF checkout","mcp_seconds":elapsed,"reference":ref.name}
 except Exception as e:
  result={"name":name,"status":"FAIL","error":str(e)}
 results.append(result);(OUT/"native-cases.json").write_text(json.dumps(results,indent=2),encoding="utf-8");print(json.dumps(result),flush=True)
