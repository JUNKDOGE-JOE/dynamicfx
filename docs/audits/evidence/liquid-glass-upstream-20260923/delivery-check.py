import hashlib,json,zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=Path(__file__).parent
sha=lambda b:hashlib.sha256(b).hexdigest()
source=(root/'examples/liquid-glass.glsl').read_text(encoding='utf-8')
ffx=(root/'examples/liquid-glass.ffx').read_bytes()
assert any(source.encode(encoding) in ffx for encoding in ['utf-8','utf-16-be','utf-16-le'])
archive=root/'dist/upstream-liquid-glass-final/DynamicFX-0.1.1-coverage-candidate-windows-x64.zip'
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
    names=['liquid-glass.glsl','liquid-glass.ffx','apply-liquid-glass.jsx','liquid-glass-upstream.md','licenses/liquid-glass-studio-MIT.txt','licenses/glsl-color-functions-MIT.txt']
    for name in names:assert z.read('LiquidGlass/'+name)==(root/'examples'/name).read_bytes()
    for name,expected in [('DynamicFx.aex','6d188d2251fdecf89f18611fd429cbc4cd9b8c056dc3ad8f25dc35f74eca6a97'),('DynamicFxCoverageReader.aex','29695b72fc8a26222a51319c3e42f86696b6d15e78efb02d968bc264ff173be6')]:
        matches=[n for n in z.namelist() if n.endswith('/'+name) or n==name];assert len(matches)==1
        assert sha(z.read(matches[0]))==expected
report={'status':'PASS','archive_sha256':sha(archive.read_bytes()),'zip_crc':'PASS','material_members_exact':len(names),'ffx_embeds_exact_source':True,'native_pair_unchanged':True,'gpu_cases':len(json.loads((out/'gpu-summary.json').read_text())),'ae_frames':len(json.loads((out/'ae-matrix-summary.json').read_text())),'source_sha256':sha((root/'examples/liquid-glass.glsl').read_bytes()),'ffx_sha256':sha(ffx)}
(out/'delivery-check.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report))
