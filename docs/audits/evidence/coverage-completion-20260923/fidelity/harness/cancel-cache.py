import ctypes,json,sys,time
from pathlib import Path
import numpy as np
from PIL import Image
sys.path.insert(0,r'<repository>/spike/host-outline')
from mcp_client import Client
out=Path(__file__).parent;client=Client()
rows=json.loads((out/'preview-cancel-done.json').read_text())['rows']
frames=sorted({max(0,min(749,round(r['time']*25)+i)) for r in rows for i in range(-3,9)})
u=ctypes.WinDLL('user32');u.GetForegroundWindow.restype=ctypes.c_void_p;foreground=u.GetForegroundWindow()
for phase in ['cached','cold']:
    for offset in range(0,len(frames),8):
        batch=frames[offset:offset+8]
        outputs=[(out/f'cancel-{phase}-{frame:04}.png').resolve().as_posix() for frame in batch]
        code='(function(){if(!app.project.file||app.project.file.name!=="cancel.aep")throw Error("Unexpected project");var c=app.project.itemByID(30),frames='+json.dumps(batch)+',paths='+json.dumps(outputs)+';'+('app.purge(PurgeTarget.ALL_CACHES);' if phase=='cold' and offset==0 else '')+'for(var i=0;i<frames.length;i++)c.saveFrameToPng(frames[i]/25,new File(paths[i]));return JSON.stringify({phase:'+json.dumps(phase)+',frames:frames});})()'
        r=client.call('ae_exec',{'code':code,'timeout_sec':25},timeout=35)
        (out/f'cancel-{phase}-{offset}.json').write_text(json.dumps({'code':code,'result':r},ensure_ascii=False,indent=2),encoding='utf-8')
        if not Client.data(r).get('ok'):raise RuntimeError(json.dumps(Client.data(r),ensure_ascii=True))
        for path in map(Path,outputs):
            for _ in range(80):
                if path.exists() and path.read_bytes().endswith(b'IEND\xaeB`\x82'):break
                time.sleep(.1)
        print(json.dumps({'phase':phase,'frames':batch}),flush=True)
report=[]
for frame in frames:
    a=np.array(Image.open(out/f'cancel-cached-{frame:04}.png').convert('RGBA'))
    b=np.array(Image.open(out/f'cancel-cold-{frame:04}.png').convert('RGBA'))
    report.append({'frame':frame,'different_channels':int(np.count_nonzero(a!=b)),'max_difference':int(np.abs(a.astype(int)-b.astype(int)).max())})
result={'status':'PASS' if all(r['different_channels']==0 for r in report) else 'FAIL','frames':len(frames),'foreground_unchanged':foreground==u.GetForegroundWindow(),'comparisons':report}
(out/'cancel-cache-summary.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps({k:v for k,v in result.items() if k!='comparisons'}),flush=True)
assert result['status']=='PASS',result
