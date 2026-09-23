import json,os,re,subprocess,sys,time
from pathlib import Path
out=Path(__file__).parent
kind=sys.argv[1];depth=int(sys.argv[2]);mode=sys.argv[3]
stem=f'mfr-{kind}-{depth}-{mode.lower()}'
label=stem+os.environ.get('MFR_RUN_TAG','')
dest=out/label;dest.mkdir(exist_ok=True)
trace=Path(os.environ['TEMP'])/'dynamicfx-render-trace-main.log';main=Path(os.environ['TEMP'])/'dynamicfx.log'
offsets=[p.stat().st_size if p.exists() else 0 for p in [trace,main]]
env=os.environ.copy();env['DYNAMICFX_RENDER_TRACE']='1';env['DYNAMICFX_VERBOSE_LOG']='1'
project=out/(stem+'.aep')
if not project.exists():project=out/(f'mfr-{kind}-{depth}.aep')
args=[r'C:/Program Files/Adobe/Adobe After Effects 2026/Support Files/aerender.exe','-project',str(project.resolve()),'-rqindex','1','-output',str((dest/'frame_[#####].psd').resolve()),'-mfr',mode,'75','-v','ERRORS_AND_PROGRESS']
start=time.monotonic()
with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:
    process=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW,env=env)
    print(json.dumps({'label':label,'pid':process.pid,'started':True}),flush=True)
    code=process.wait(timeout=240)
texts=[]
for p,offset in zip([trace,main],offsets):
    with p.open('rb') as f:f.seek(offset);texts.append(f.read().decode('utf-8',errors='replace'))
for name,text in zip(['trace','main'],texts):(out/(label+'.'+name+'.log')).write_text(text,encoding='utf-8')
digests=re.findall(r'native frame digest: pid=(\d+) time=(-?\d+)/(\d+) depth=(\d+) world=(\d+)x(\d+) origin=Point \{ h: (-?\d+), v: (-?\d+) \} hash=([0-9a-f]+)',texts[1])
frames={};conflicts=[]
for pid,t,scale,d,w,h,x,y,digest in digests:
    key=str(round(int(t)/int(scale)*25));value=[int(d),int(w),int(h),int(x),int(y),digest]
    if key in frames and frames[key]!=value:conflicts.append(key)
    frames[key]=value
events=[];threads=set();engine_pids=set()
for line in texts[0].splitlines():
    bits=line.split('|')
    if len(bits)!=8 or bits[1]!='main':continue
    event,label_,pid,id_,thread,t,scale,ns=bits;threads.add((pid,thread));engine_pids.add(pid)
    events.append((int(ns),1 if event=='begin' else -1,(pid,id_,thread,t,scale)))
active=peak=0
for _,delta,_ in sorted(events):active+=delta;peak=max(peak,active)
report={'case':label,'returncode':code,'elapsed':round(time.monotonic()-start,3),'frames':frames,'distinct_hashes':len({v[-1] for v in frames.values()}),'conflicts':conflicts,'gate_errors':texts[1].count('coverage gate refused:'),'peak_callbacks':peak,'threads':len(threads),'engine_pids':sorted(engine_pids),'unbalanced_callbacks':active,'output_files':len(list(dest.glob('*.psd')))}
report['status']='PASS' if code==0 and len(frames)==48 and not conflicts and not report['gate_errors'] and active==0 and (mode=='OFF' or peak>=2) else 'FAIL'
(out/(label+'-summary.json')).write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps({k:v for k,v in report.items() if k!='frames'}),flush=True)
sys.exit(0 if report['status']=='PASS' else 1)
