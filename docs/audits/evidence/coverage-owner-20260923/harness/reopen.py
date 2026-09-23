import json,os,re,sys,shutil,time
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[6]/'spike/host-outline'))
from mcp_client import Client
out=Path(__file__).parent;client=Client();log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log'
shutil.copyfile(out/'production-coverage.aep',out/'production-lifecycle.aep')
offset=log.stat().st_size
dest=(out/'reopen-adjustment-32.png').resolve().as_posix()
code='(function(){if(!app.project.file||app.project.file.name!=="production-coverage.aep"||app.project.dirty)throw Error("Unexpected or unsaved project");app.open(new File('+json.dumps((out/'production-baseline.aep').resolve().as_posix())+'));app.project.bitsPerChannel=32;app.purge(PurgeTarget.ALL_CACHES);app.project.itemByID(30).saveFrameToPng(0,new File('+json.dumps(dest)+'));app.project.save(new File('+json.dumps((out/'production-coverage.aep').resolve().as_posix())+'));return JSON.stringify({items:app.project.numItems,dirty:app.project.dirty});})()'
args={'code':code,'timeout_sec':25};r=client.call('ae_exec',args,timeout=35)
(out/'reopen.result.json').write_text(json.dumps({'args':args,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
d=Client.data(r)
if not d.get('ok'):raise RuntimeError(json.dumps(d,ensure_ascii=True))
for _ in range(40):
 if Path(dest).exists() and Path(dest).read_bytes().endswith(b'IEND\xaeB`\x82'):break
 time.sleep(.25)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
(out/'reopen-adjustment-32.native.log').write_text(raw,encoding='utf-8')
def array(text):
 matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=32 origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text))
 if len(matches)!=1:raise RuntimeError('Unexpected frame count '+str(len(matches)))
 m=matches[0];w,h,x,y=map(int,m.groups()[:4]);a=np.zeros((600,800),dtype=np.uint32);a[y:y+h,x:x+w]=np.array(json.loads(m[5]),dtype=np.uint32).reshape(h,w);return a
a=array((out/'reference-32.native.log').read_text(encoding='utf-8'));b=array(raw);count=int(np.count_nonzero(a!=b))
report={'case':'saved native links render immediately after reopen','depth':32,'pixels':480000,'native_word_mismatches':count,'nonzero_reference':int(np.count_nonzero(a)),'status':'PASS' if count==0 and np.any(a) else 'FAIL'}
(out/'reopen-summary.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report))
