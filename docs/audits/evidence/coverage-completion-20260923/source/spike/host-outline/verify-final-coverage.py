"""Independently compare the current-artifact native coverage records."""
import gzip,json,re,sys
from pathlib import Path

root=Path(sys.argv[1]);raw=root/'raw' if (root/'raw').exists() else root

def text(name):
    path=raw/name
    return path.read_text(encoding='utf-8') if path.exists() else gzip.decompress(path.with_name(path.name+'.gz').read_bytes()).decode('utf-8')

def frame(name,depth=32,width=800,height=600):
    matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=(8|16|32) origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text(name+'.native.log').replace('\r\n','\n')))
    assert len(matches)==1,(name,len(matches))
    m=matches[0];w,h,d,x,y=map(int,m.groups()[:5]);assert d==depth
    words=json.loads(m[6]);assert len(words)==w*h
    pixels=[0]*(width*height);left,top,right,bottom=max(x,0),max(y,0),min(x+w,width),min(y+h,height)
    for row in range(top,bottom):
        start=(row-y)*w+left-x;pixels[row*width+left:row*width+right]=words[start:start+right-left]
    return pixels

counts={}
for filename,expected in [('final-core-summary.json',78),('more-summary.json',24),('extra-summary.json',54),('fixed-shift-summary.json',12),('wgsl-coverage-summary.json',6)]:
    rows=json.loads(text(filename));assert len(rows)==expected
    for row in rows:
        case,kind,depth=row['case'],row['kind'],row['depth'];t=row.get('time',.5);ds=row.get('downsample',1)
        width,height=(800+ds-1)//ds,(600+ds-1)//ds;prefix='final-' if filename=='final-core-summary.json' else ''
        reference=frame(f'{prefix}{case}-reference-{depth}-{t}-{ds}',depth,width,height)
        actual=frame(f'{prefix}{case}-{kind}-{depth}-{t}-{ds}',depth,width,height)
        assert any(reference) and reference==actual,(case,kind,depth,t,ds)
        if case=='fixed-shift-128':assert sum(reference[y*800+x]!=0 for y in range(600) for x in range(128))=={8:3548,16:3554,32:3554}[depth]
    counts[filename]=len(rows)
roi=json.loads(text('roi-summary.json'));assert len(roi)==15 and all(r['ordinary']==r['reference'] for r in roi)
empty=json.loads(text('empty-summary.json'));assert len(empty)==3 and all(r['status']=='PASS' and r['adjustment_disabled_differences']==r['ordinary_nonzero_alpha']==r['reference_nonzero_alpha']==0 for r in empty)
reference=frame('after5-reference');assert reference==frame('after5-recovered')
assert set(frame('after5-immediate'))=={0,0x3f800000}
for label in ['new','existing']:
    assert 'coverage gate refused: E60' in text('ffx-'+label+'-immediate.main.log')
    assert set(frame('ffx-'+label+'-immediate'))=={0,0x3f800000}
    assert reference==frame('ffx-'+label+'-recovered')
print(json.dumps({'status':'PASS','native_comparisons':sum(counts.values()),'groups':counts,'roi':15,'empty_depths':3,'copy_ffx_recoveries':3}))
