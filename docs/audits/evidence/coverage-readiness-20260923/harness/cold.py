import json,os,re,subprocess,time,sys
from pathlib import Path
import numpy as np
from frame import canvas
out=Path(__file__).parent
label=sys.argv[1] if len(sys.argv)>1 else "cold"
log=Path(os.environ['TEMP'])/'dynamicfx-host-shape-probe.log';main=Path(os.environ['TEMP'])/'dynamicfx.log'
offset=log.stat().st_size;mo=main.stat().st_size
args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',str(out/(label+'.aep')),'-rqindex','1','-mfr','OFF','100','-v','ERRORS_AND_PROGRESS']
env=os.environ.copy();env['DYNAMICFX_VERBOSE_LOG']='1'
start=time.monotonic()
with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:
    process=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW,env=env)
    print(json.dumps({'aerender_pid':process.pid,'started':True}),flush=True)
    code=process.wait(timeout=150)
with log.open('rb') as f:f.seek(offset);raw=f.read().decode('utf-8',errors='replace')
with main.open('rb') as f:f.seek(mo);messages=f.read().decode('utf-8',errors='replace')
(out/(label+'.native.log')).write_text(raw,encoding='utf-8');(out/(label+'.main.log')).write_text(messages,encoding='utf-8')
live=json.loads((out/'live-pids.json').read_text(encoding='utf-8-sig'));live=set(live if isinstance(live,list) else [live])
blocks=re.findall(r'(\[time=\d+ pid=(\d+) thread=[^\n]*\]\nSMART_ALPHA id=0\nworld=[^\n]*\nalpha_words=\[[^\n]*\])',raw)
frames=[(text,int(pid)) for text,pid in blocks if int(pid) not in live]
if len(frames)!=1:raise RuntimeError('Expected one independent frame, got '+str(len(frames)))
actual=canvas(frames[0][0]);reference=canvas((out/('after4-reference.native.log' if label in ['pending','missing'] else 'after3-reference.native.log')).read_text(encoding='utf-8'))
changed=int(np.count_nonzero(actual!=reference))
report={'case':'independent aerender first frame','returncode':code,'engine_pid':frames[0][1],'elapsed_seconds':round(time.monotonic()-start,3),'pixels':480000,'native_word_mismatches':changed,'gate_errors':messages.count('coverage gate refused:'),'status':'PASS' if code==0 and changed==0 else 'FAIL'}
if label=='missing':
 report['case']='missing reader remains unverified offline';report['opaque_words']=int(np.count_nonzero(actual==0x3f800000));report['status']='PASS' if code==0 and report['gate_errors']>0 and set(np.unique(actual))=={0,0x3f800000} else 'FAIL'
(out/(label+'-summary.json')).write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)

raise SystemExit(0 if report["status"]=="PASS" else 1)
