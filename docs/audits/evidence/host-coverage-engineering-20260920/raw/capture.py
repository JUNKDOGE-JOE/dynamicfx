import sys,json,os,time,re
from pathlib import Path
from PIL import Image
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
out=Path(__file__).resolve().parent
reference=json.loads(Client.data(json.loads((out.parent/"regression-20260920/triangle-expression.result.json").read_text()))["content"])["reference"]
log=Path(os.environ["TEMP"])/"dynamicfx-host-shape-probe.log"
results=[]
for name,cid,rid in [("small",202,reference),("baseline",174,63)]:
 start=log.stat().st_size;image=out/(name+".png");ref=out/(name+"-reference.png")
 code='(function(){if(!app.project.file||app.project.file.parent.name!=="engineering-20260920")throw Error("Wrong fixture");var c=app.project.itemByID(COMP_ID);c.layer(1).property("ADBE Effect Parade").property(1).property(4).setValue(true);app.purge(PurgeTarget.ALL_CACHES);c.saveFrameToPng(0,new File(IMAGE));app.project.itemByID(REFERENCE_ID).saveFrameToPng(0,new File(REFIMAGE));return JSON.stringify({width:c.width,height:c.height,id:c.id});})()'
 code=code.replace("COMP_ID",str(cid)).replace("REFERENCE_ID",str(rid)).replace("REFIMAGE",json.dumps(ref.as_posix())).replace("IMAGE",json.dumps(image.as_posix()));args={"code":code,"timeout_sec":40};r=Client().call("ae_exec",args,timeout=50);(out/(name+".result.json")).write_text(json.dumps({"args":args,"result":r}));t=time.monotonic()
 while time.monotonic()-t<50 and not(image.exists() and ref.exists()):time.sleep(.25)
 with log.open("rb") as f:f.seek(start);raw=f.read()
 (out/(name+".native.log")).write_bytes(raw)
 text=raw.decode().rsplit("PF_RENDER\n",1)[-1];candidates=[]
 for part in re.split(r"(?m)^path\[\d+\]",text)[1:]:
  pts=[(float(x),float(y)) for x,y in re.findall(r"vertex\[\d+\]=PF_PathVertex \{ x: ([\d.e+-]+), y: ([\d.e+-]+)",part)]
  if pts and pts[0]==(-1234,-5678):candidates.append(pts)
 if len(candidates)!=1:raise RuntimeError("Missing PF coverage path: "+name)
 pts=candidates[0];w,h=map(int,pts[1]);pixels=bytearray(w*h)
 for (x,y),(packed,bh) in zip(pts[2::2],pts[3::2]):
  bw=int(packed);alpha=round((packed-bw)*4*255)
  for row in range(int(y),int(y+bh)):pixels[row*w+int(x):row*w+int(x)+bw]=bytes([alpha])*bw
 actual=Image.frombytes("L",(w,h),bytes(pixels));actual.save(out/(name+"-decoded.png"));expected=Image.open(ref).convert("RGBA").getchannel("A");assert expected.size==(w,h)
 differences=[abs(a-b) for a,b in zip(pixels,expected.tobytes())];row={"name":name,"route":"native PF PathQuery","size":[w,h],"vertices":len(pts),"max_error_8bit":max(differences),"different_pixels":sum(d!=0 for d in differences),"status":"PASS" if max(differences)<=1 else "FAIL"};results.append(row);print(json.dumps(row),flush=True);(out/"pixel-checks.json").write_text(json.dumps(results,indent=2))
