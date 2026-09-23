import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'scripts/out/host-shape-native-20260910'
DEST = ROOT / 'docs/audits/evidence/host-shape-native-20260910/managed-carrier'
PREFIXES = ('coverage-carrier-approved', 'coverage-expression-reflection',
            'coverage-alpha-', 'coverage-v3-', 'coverage-v4-',
            'coverage-blocks-expression-v3', 'coverage-blocks-expression-v4',
            'v006-', 'v007-', 'v008-', 'install-v006', 'install-v007', 'install-v008',
            'launch-v006', 'launch-v007', 'launch-v008', 'quit-for-v006',
            'quit-for-v007', 'reopen-v006', 'reopen-v007', 'reopen-v008',
            'v009-', 'install-v009', 'launch-v009', 'quit-for-v009', 'reopen-v009')


def freeze(source, relative):
    destination = DEST / relative
    payload = source.read_bytes()
    if destination.exists() and destination.read_bytes() != payload:
        raise RuntimeError('Frozen evidence changed: ' + str(relative))
    destination.parent.mkdir(parents=True, exist_ok=True)
    if not destination.exists():
        shutil.copy2(source, destination)
    return {'path': relative.as_posix(), 'sha256': hashlib.sha256(payload).hexdigest()}


if __name__ == '__main__':
    records = []
    for source in sorted(OUT.iterdir()):
        if source.is_file() and source.name.startswith(PREFIXES):
            records.append(freeze(source, Path('raw') / source.name))
    for folder in ['v007-source', 'v008-source']:
        for source in sorted((OUT / folder).rglob('*')):
            if source.is_file():
                records.append(freeze(source, Path('raw') / folder / source.relative_to(OUT / folder)))
    native = ROOT / 'spike/host-outline/native'
    for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'src/lib.rs',
                 'src/read.rs', 'src/idle.rs', 'src/carrier.rs']:
        records.append(freeze(native / name, Path('source/native') / name))
    for name in ['coverage-blocks-expression.js', 'coverage-alpha-bound-expression.js',
                 'setup-managed-fixture.jsx', 'freeze-managed-evidence.py',
                 'check-managed-step.py', 'capture-native.py', 'verify-coverage-blocks.py']:
        records.append(freeze(ROOT / 'spike/host-outline' / name, Path('source') / name))
    (DEST / 'manifest.json').write_text(json.dumps(records, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'frozen': len(records), 'directory': str(DEST)}))
