import sys,json,time
from pathlib import Path
sys.path.insert(0,"E:/Code/AePlugin_Dynamicfx/spike/host-outline")
from mcp_client import Client
OUT=Path(__file__).resolve().parent
label=sys.argv[1];source=Path(sys.argv[2]);target=OUT/(label+".result.json")
if target.exists():raise RuntimeError("Evidence label already used")
args={"code":source.read_text(encoding="utf-8"),"timeout_sec":40}
(OUT/(label+".args.json")).write_text(json.dumps(args),encoding="utf-8")
t=time.monotonic();r=Client().call("ae_exec",args,timeout=50);target.write_text(json.dumps(r),encoding="utf-8");(OUT/(label+".timing.json")).write_text(json.dumps({"seconds":time.monotonic()-t}));data=Client.data(r)
try:
 summary=json.loads(data.get("content","null"))
 if isinstance(summary,dict) and "points" in summary:
  summary["pointCount"]=len(summary.pop("points"))
 print(json.dumps({"ok":data.get("ok"),"result":summary},ensure_ascii=True)[:1500])
except (ValueError,TypeError):print(json.dumps(data,ensure_ascii=True)[:1500])
if not Client.data(r).get("ok"):raise SystemExit(1)
