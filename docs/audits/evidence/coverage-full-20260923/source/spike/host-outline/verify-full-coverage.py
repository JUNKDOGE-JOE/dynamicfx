"""Verify the recorded production fidelity matrix and expanded-input repair."""
import gzip,json,re,sys
from pathlib import Path

root=Path(sys.argv[1]);raw=root/'raw'
def frame(name,depth,width=800,height=600):
    text=gzip.decompress((raw/(name+'.native.log.gz')).read_bytes()).decode('utf-8').replace('\r\n','\n')
    matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text))
    assert len(matches)==1,(name,len(matches))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);assert d==depth
    values=json.loads(m[6]);assert len(values)==w*h
    pixels=[0]*(width*height);left,top,right,bottom=max(x,0),max(y,0),min(x+w,width),min(y+h,height)
    for row in range(top,bottom):
        start=(row-y)*w+left-x;pixels[row*width+left:row*width+right]=values[start:start+right-left]
    return pixels
comparisons=0
for filename in ['final-core-summary.json','more-summary.json','fixed-shift-summary.json']:
    rows=json.loads((raw/filename).read_text(encoding='utf-8'))
    for row in rows:
        case,kind,depth=row['case'],row['kind'],row['depth'];t=row.get('time',.5);ds=row.get('downsample',1)
        width,height=(800+ds-1)//ds,(600+ds-1)//ds
        prefix='final-' if filename=='final-core-summary.json' else ''
        ref=frame(f'{prefix}{case}-reference-{depth}-{t}-{ds}',depth,width,height)
        actual=frame(f'{prefix}{case}-{kind}-{depth}-{t}-{ds}',depth,width,height)
        assert any(ref) and ref==actual,(case,kind,depth,t,ds)
        if case=='fixed-shift-128':
            assert sum(ref[y*800+x]!=0 for y in range(600) for x in range(128))=={8:3548,16:3554,32:3554}[depth]
        comparisons+=1
for depth in [8,16,32]:
    ref=frame(f'shift-128-reference-{depth}-0.5-1',depth)
    bad=frame(f'shift-128-ordinary-{depth}-0.5-1',depth)
    assert sum(a!=b for a,b in zip(ref,bad))=={8:3548,16:3554,32:3554}[depth]
assert json.loads((raw/'empty-control-summary.json').read_text())['rgba_mismatches']==0
roi=json.loads((raw/'roi-summary.json').read_text());assert len(roi)==15
assert all(row['ordinary']==row['reference'] for row in roi)
print(json.dumps({'status':'PASS','native_comparisons':comparisons,'roi_comparisons':len(roi),'retained_boundary_failures':3,'empty_32_control':'PASS'}))
