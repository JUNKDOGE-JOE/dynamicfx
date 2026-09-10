#!/usr/bin/env python3
"""Package a verified Windows AEX with its dependencies and instructions."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact', type=Path, required=True)
    parser.add_argument('--verification', type=Path, required=True)
    parser.add_argument('--third-party', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
    artifact = args.artifact.read_bytes()
    verified = json.loads(args.verification.read_text())
    assert verified['sha256'] == sha(artifact) and verified['version'] == version
    notices = json.loads((args.third_party / 'manifest.json').read_text())
    assert notices['version'] == version and notices['target'] == 'x86_64-pc-windows-msvc'
    assert notices['cargo_lock_sha256'] == sha((ROOT / 'Cargo.lock').read_bytes())
    assert notices['cargo_toml_sha256'] == sha((ROOT / 'Cargo.toml').read_bytes())
    files = [notices['rust_standard_library']]
    for package in notices['packages']:
        files.extend(package['files'])
    for entry in files:
        path = (args.third_party / entry['file']).resolve()
        path.relative_to(args.third_party.resolve())
        assert sha(path.read_bytes()) == entry['sha256']
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    inputs = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'build.rs', *sorted((ROOT / 'src').rglob('*.rs'))]
    identity = dict(version=version, source_commit=commit, binary_sha256=sha(artifact),
                    build_command='cargo build --offline --release',
                    source_files={p.relative_to(ROOT).as_posix(): sha(p.read_bytes()) for p in inputs},
                    host_scope='AE 2026 runtime-code equivalence; exact versioned bytes not installed',
                    verification=verified)
    install = f'''DynamicFX {version} - Windows x64 / DirectX 12

Close After Effects before copying the plug-in. Copy DynamicFx.aex to:
C:\\Program Files\\Adobe\\Adobe After Effects 2026\\Support Files\\Plug-ins\\DynamicFx\\DynamicFx.aex
Create the DynamicFx directory if needed; this may require administrator rights.
Use the version-specific AE directory, never shared Common/Plug-ins/7.0/MediaCore.
Restart AE after installation. The effect and match name are DynamicFx.

Verify SHA-256 before installing (PowerShell): Get-FileHash .\\DynamicFx.aex
Expected AEX SHA-256: {sha(artifact)}

Validation: Windows x64 build, CPU suites and real-DX12 compiler recovery.
AE 2026 valid rendering was verified on the same runtime code before the version
metadata bump; the exact new binary was not reinstalled for this packaging run.
Other AE years and macOS 0.1.1 are not accepted by these results.

Known limitation: Source edits may not restore with one Undo; Redo may be
unavailable. Keep prior shader text and explicitly restore it when needed.
The optional gradient editor is disabled. No accounts or services are required.

Source and full instructions:
https://github.com/JUNKDOGE-JOE/dynamicfx/tree/v{version}
'''
    args.out.mkdir(parents=True, exist_ok=True)
    plugin_zip = args.out / f'DynamicFX-{version}-windows-x64.zip'
    if plugin_zip.exists():
        raise ValueError('Refusing to replace a frozen package: ' + str(plugin_zip))
    with zipfile.ZipFile(plugin_zip, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        z.writestr('DynamicFx.aex', artifact)
        z.writestr('INSTALL.txt', install)
        z.write(ROOT / 'LICENSE', 'LICENSE')
        z.write(ROOT / 'Cargo.lock', 'build/Cargo.lock')
        z.writestr('build/source-identity.json', json.dumps(identity, indent=2) + '\n')
        for p in sorted(args.third_party.rglob('*')):
            if p.is_file():
                z.write(p, 'THIRD_PARTY/' + p.relative_to(args.third_party).as_posix())
    sums = []
    for p in [plugin_zip]:
        with zipfile.ZipFile(p) as z:
            assert z.testzip() is None
        sums.append(f'{sha(p.read_bytes())}  {p.name}')
    sums.append(f'{sha(artifact)}  DynamicFx.aex')
    (args.out / 'SHA256SUMS.txt').write_text('\n'.join(sums) + '\n')
    (args.out / 'source-identity.json').write_text(json.dumps(identity, indent=2) + '\n')
    print('\n'.join(sums))


if __name__ == '__main__':
    main()
