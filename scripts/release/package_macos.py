#!/usr/bin/env python3
"""Archive an already built/signed bundle without rebuilding or re-signing it."""
import argparse
import hashlib
import importlib.util
import json
import re
from pathlib import Path
import shutil
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("macos", ROOT / "scripts/macos.py")
macos = importlib.util.module_from_spec(spec)
spec.loader.exec_module(macos)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, default=ROOT / "target/macos-arm64/DynamicFx.plugin")
    parser.add_argument("--build-record", type=Path, required=True)
    parser.add_argument("--third-party", type=Path, required=True,
                        help="Verified output of dependency_notices.py; kept outside the signed bundle")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        raise SystemExit("Archive already exists; choose a fresh path to preserve the previous artifact")
    build = json.loads(args.build_record.read_text())
    before = macos.verify(args.bundle)
    for key in ("version", "binary_sha256", "resource_sha256"):
        if before[key] != build["bundle"][key]:
            raise SystemExit("Bundle differs from the recorded build: " + key)
    for name, expected in build["source_files"].items():
        if sha(ROOT / name) != expected:
            raise SystemExit("Build input changed: " + name)
    notice_root = args.third_party.resolve()
    notices = json.loads((notice_root / "manifest.json").read_text())
    if (notices["version"] != before["version"] or
            notices["target"] != "aarch64-apple-darwin" or
            notices["cargo_lock_sha256"] != sha(ROOT / "Cargo.lock") or
            notices["cargo_toml_sha256"] != sha(ROOT / "Cargo.toml")):
        raise SystemExit("Third-party notices do not match the recorded build")
    notice_files = [notices["rust_standard_library"]]
    for package in notices["packages"]:
        notice_files.extend(package["files"])
    for entry in notice_files:
        path = notice_root / entry["file"]
        path.resolve().relative_to(notice_root)
        if sha(path) != entry["sha256"]:
            raise SystemExit("Third-party notice hash mismatch: " + entry["file"])
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="dynamicfx-release-") as tmp:
        stage = Path(tmp)
        shutil.copytree(args.bundle, stage / "DynamicFx.plugin")
        for name in ("README.md", "LICENSE"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copytree(ROOT / "examples", stage / "examples", ignore=shutil.ignore_patterns(".DS_Store"))
        for name in ("macos-arm64.md", "shader-quality.md"):
            (stage / "docs").mkdir(exist_ok=True)
            shutil.copy2(ROOT / "docs" / name, stage / "docs" / name)
        shutil.copytree(ROOT / "skills/dynamicfx-shaders", stage / "skills/dynamicfx-shaders", ignore=shutil.ignore_patterns("__pycache__", ".DS_Store"))
        (stage / "scripts").mkdir()
        for name in ("install.sh", "macos.py"):
            shutil.copy2(ROOT / "scripts" / name, stage / "scripts" / name)
        (stage / "build").mkdir()
        shutil.copy2(ROOT / "Cargo.lock", stage / "build/Cargo.lock")
        (stage / "build/source-identity.json").write_text(json.dumps({
            "version": before["version"], "source_commit": build["source_commit"],
            "source_tree": build["tree"], "source_files": build["source_files"],
            "binary_sha256": before["binary_sha256"], "resource_sha256": before["resource_sha256"],
            "build_command": build["build_command"], "signature": "ad-hoc; not notarized",
        }, indent=2) + "\n")
        (stage / "INSTALL.txt").write_text(f"""DynamicFX {before['version']} — macOS Apple Silicon

For native Apple Silicon After Effects 2026. Windows 0.1.0 is pending.
This bundle is ad-hoc signed. It is not Developer ID signed or notarized.

1. Download from https://github.com/JUNKDOGE-JOE/dynamicfx/releases/tag/v0.1.0
   and compare the ZIP's SHA-256 with the release's SHA256SUMS.txt.
2. Close After Effects and aerender. Unzip the archive. Keep any older
   DynamicFx.plugin as a backup outside the Plug-ins folder.
3. Copy DynamicFx.plugin into:
   /Applications/Adobe After Effects 2026/Plug-ins/DynamicFx/
   Create the DynamicFx folder if needed. Finder may ask for an admin password.
   Do not install into Adobe's shared MediaCore folder or keep duplicate copies.
4. If macOS blocks this verified download, open Terminal in the unzipped
   directory and remove quarantine from this bundle only, then copy it again:
     xattr -dr com.apple.quarantine DynamicFx.plugin
   Do not disable Gatekeeper system-wide.
5. Start After Effects and apply DynamicFx. GLSL is the default. Choose WGSL
   in Language for examples ending in .wgsl, then paste their complete source
   as a backtick Source expression ending in ;0. See examples/README.md and
   skills/dynamicfx-shaders/wgsl.md for escaping and the exact ABI.

Optional verified installer (requires Python 3 and Xcode Command Line Tools):
  sudo bash scripts/install.sh 2026 --bundle "$PWD/DynamicFx.plugin"
It backs up the previous bundle, verifies architecture/signature, and refuses
a running AE/aerender or duplicate/shared copies. Enter your password directly
in the terminal; it is not a DynamicFX account or license password.

Prefer 16/32-bpc projects for subtle glows and multipass fields. The optional
gradient editor is disabled. Older GLSL-only versions cannot render WGSL
projects; keep the source and reinstall this version to render them.

The archive includes source identity, the exact build/Cargo.lock used, and
third-party/THIRD_PARTY_NOTICES.txt with dependency license texts. Keep the
third-party directory intact alongside this archive's MIT LICENSE.
Windows builders should check out v0.1.0, copy that lockfile to the repository
root, build --locked, and validate the resulting AEX in Windows AE before
uploading a separately identified Windows asset to the same release.
""")
        # Keep the user archive compact. Links to repository-only audits or
        # build tools resolve to this release's exact source tag online.
        for path in stage.rglob("*.md"):
            def link(match):
                label, target = match.group(1), match.group(2)
                if target.startswith(("http://", "https://", "mailto:", "#")):
                    return match.group(0)
                name, sep, anchor = target.partition("#")
                resolved = (path.parent / name).resolve()
                if resolved.exists():
                    return match.group(0)
                original = (ROOT / path.relative_to(stage)).parent.joinpath(name).resolve()
                relative = original.relative_to(ROOT)
                if original.is_file() and original.suffix.lower() in (".png", ".svg", ".jpg", ".webp"):
                    resolved.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(original, resolved)
                    return match.group(0)
                url = f"https://github.com/JUNKDOGE-JOE/dynamicfx/blob/v{before['version']}/{relative.as_posix()}"
                return f"[{label}]({url}{sep}{anchor})"
            path.write_text(re.sub(r"\[([^]]+)\]\(([^)]+)\)", link, path.read_text()))
        # Preserve upstream notices verbatim; never run link rewriting on them.
        shutil.copytree(notice_root, stage / "third-party")
        files = sorted(p for p in stage.rglob("*") if p.is_file())
        (stage / "SHA256SUMS").write_text("".join(f"{sha(p)}  {p.relative_to(stage).as_posix()}\n" for p in files))
        with zipfile.ZipFile(args.out, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for path in sorted(p for p in stage.rglob("*") if p.is_file()):
                if path.is_symlink():
                    raise RuntimeError("Unexpected symlink in release stage")
                archive.write(path, path.relative_to(stage).as_posix())
    after = macos.verify(args.bundle)
    if before != after:
        raise RuntimeError("Source bundle identity changed while archiving")
    print(json.dumps({"archive": str(args.out), "archive_sha256": sha(args.out), "archive_bytes": args.out.stat().st_size, "bundle": before}, indent=2))


if __name__ == "__main__":
    main()
