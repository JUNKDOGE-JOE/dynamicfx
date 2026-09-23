import json,subprocess,sys
from host import call,out
reports=[]
for depth in [8,16,32]:
    call('depth-'+str(depth),'(function(){if(!app.project.file||decodeURI(app.project.file.name)!=="Parameter acceptance.aep")throw Error("Unexpected project");app.project.bitsPerChannel='+str(depth)+';return JSON.stringify(app.project.bitsPerChannel);})()')
    subprocess.run([sys.executable,str(out/'parameters.py'),'read',str(depth)],check=True)
    rows=json.loads((out/('parameter-summary-'+str(depth)+'.json')).read_text())
    for row in rows:
        kind=row['name'].split('-')[1]
        if kind=='percent':expected=[.25 if row['name']=='1-percent' else .37125,0,0,1]
        else:
            alpha=128/255 if kind=='alpha' else 1
            expected=[64/255*alpha,128/255*alpha,alpha,alpha]
        error=max(abs(a-b) for a,b in zip(row['rgba'],expected))
        tolerance={8:2/255,16:2/32768,32:1e-6}[depth]
        assert row['token']%4==1 and error<=tolerance,(depth,row,error)
        reports.append({'depth':depth,'case':row['name'],'maximum_error':error,'tolerance':tolerance,'status':'PASS'})
(out/'depth-summary.json').write_text(json.dumps(reports,indent=2));print(json.dumps({'cases':len(reports),'status':'PASS'}))
