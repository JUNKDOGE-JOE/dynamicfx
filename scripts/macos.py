#!/usr/bin/env python3
"""Package, inspect, and install DynamicFX's native Apple Silicon bundle.

No third-party Python packages. Packaging does not install or launch a host.
The resource parser follows the pipl 0.1.1 classic resource-map format.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import plistlib
import pwd
import re
import shutil
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
# Release ZIPs keep the verified bundle at the root; source checkouts use
# the build output. The explicit --bundle option always takes precedence.
DEFAULT_BUNDLE = (ROOT / "DynamicFx.plugin" if (ROOT / "DynamicFx.plugin").is_dir()
                  else ROOT / "target/macos-arm64/DynamicFx.plugin")


def run(*args):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT).strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def properties(path):
    """Find PiPL 16000 through the resource map, then decode its properties."""
    data = path.read_bytes()
    data_off, map_off, data_len, map_len = struct.unpack_from(">4I", data)
    require(data_off + data_len <= len(data) and map_off + map_len <= len(data),
            "Truncated resource file")
    type_off = map_off + struct.unpack_from(">H", data, map_off + 24)[0]
    type_count = struct.unpack_from(">H", data, type_off)[0] + 1
    candidates = []
    for i in range(type_count):
        at = type_off + 2 + 8 * i
        kind, count, ref_off = struct.unpack_from(">4sHH", data, at)
        if kind != b"PiPL":
            continue
        for j in range(count + 1):
            ref = type_off + ref_off + 12 * j
            resource_id = struct.unpack_from(">h", data, ref)[0]
            offset = struct.unpack_from(">I", data, ref + 4)[0] & 0xFFFFFF
            if resource_id == 16000:
                start = data_off + offset
                size = struct.unpack_from(">I", data, start)[0]
                candidates.append(data[start + 4:start + 4 + size])
    require(len(candidates) == 1, "Expected exactly one PiPL resource with id 16000")
    payload = candidates[0]
    version, count = struct.unpack_from(">II", payload)
    require(version == 0, "Unsupported PiPL version")
    result, offset = {}, 8
    for _ in range(count):
        vendor, key, ident, size = struct.unpack_from(">4s4sII", payload, offset)
        offset += 16
        require(vendor == b"8BIM" and ident == 0, "Unexpected PiPL property identity")
        require(offset + size <= len(payload), "Truncated PiPL property")
        require(key not in result, "Duplicate PiPL property")
        result[key] = payload[offset:offset + size]
        offset += (size + 3) & ~3
    require(offset == len(payload), "PiPL trailing bytes")
    return result


def pstring(value):
    return value[1:1 + value[0]].decode("utf-8")


def verify(bundle):
    bundle = bundle.resolve()
    contents = bundle / "Contents"
    with (contents / "Info.plist").open("rb") as stream:
        info = plistlib.load(stream)
    require(info["CFBundleExecutable"] == "DynamicFx", "Unexpected executable name")
    require(info["CFBundlePackageType"] == "eFKT", "Not an AE effect bundle")
    require((contents / "PkgInfo").read_bytes() == b"eFKTFXTC", "Invalid PkgInfo")
    binary = contents / "MacOS/DynamicFx"
    resource = contents / "Resources/DynamicFx.rsrc"
    architectures = run("/usr/bin/lipo", "-archs", str(binary)).split()
    require(architectures == ["arm64"], "Expected an arm64-only executable")
    symbols = run("/usr/bin/nm", "-gU", str(binary)).splitlines()
    require(any(line.endswith(" _DynamicFxMain") for line in symbols), "DynamicFxMain is not exported")
    dependencies = run("/usr/bin/otool", "-L", str(binary)).splitlines()[2:]
    require(all(line.strip().startswith(("/usr/lib/", "/System/Library/"))
                for line in dependencies), "Non-system dynamic dependency in bundle")
    props = properties(resource)
    require(props[b"kind"] == b"eFKT", "PiPL is not an AE effect")
    require(pstring(props[b"ma64"]) == "DynamicFxMain", "Missing ARM64 PiPL entry point")
    require(b"mi64" not in props, "arm64-only bundle advertises Intel code")
    require(pstring(props[b"eMNA"]) == "DynamicFx", "Unexpected match name")
    run("/usr/bin/codesign", "--verify", "--deep", "--strict", "--verbose=2", str(bundle))
    return {
        "bundle": str(bundle), "version": info["CFBundleShortVersionString"],
        "architectures": architectures, "binary_sha256": sha(binary),
        "resource_sha256": sha(resource), "binary_bytes": binary.stat().st_size,
        "pipl_outflags": "0x%08X" % struct.unpack(">I", props[b"eGLO"])[0],
        "pipl_outflags2": "0x%08X" % struct.unpack(">I", props[b"eGL2"])[0],
        "signature": run("/usr/bin/codesign", "-d", "--verbose=2", str(bundle)),
    }


def package(artifact_dir, bundle):
    require(bundle.suffix == ".plugin", "Package output must end in .plugin")
    require(not any(part.lower() in ("plug-ins", "mediacore") or part.lower().endswith(".app")
                    for part in bundle.parts), "Package outside Adobe hosts; use install for installation")
    binary = artifact_dir / "libdynamicfx.dylib"
    resource = artifact_dir / "dynamicfx.rsrc"
    require(binary.is_file() and resource.is_file(), "Build dylib and PiPL resource first")
    manifest = (ROOT / "Cargo.toml").read_text()
    package_section = manifest.split("[package]", 1)[1].split("[", 1)[0]
    version = re.search(r'^version\s*=\s*"([^"]+)"', package_section, re.M).group(1)
    bundle.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".package-", dir=bundle.parent) as tmp:
        stage = Path(tmp) / "DynamicFx.plugin"
        contents = stage / "Contents"
        (contents / "MacOS").mkdir(parents=True)
        (contents / "Resources").mkdir()
        shutil.copy2(binary, contents / "MacOS/DynamicFx")
        shutil.copy2(resource, contents / "Resources/DynamicFx.rsrc")
        (contents / "PkgInfo").write_bytes(b"eFKTFXTC")
        info = {
            "CFBundleDevelopmentRegion": "en", "CFBundleExecutable": "DynamicFx",
            "CFBundleIdentifier": "com.dynamicfx.plugin", "CFBundleName": "DynamicFx",
            "CFBundleInfoDictionaryVersion": "6.0", "CFBundlePackageType": "eFKT",
            "CFBundleSignature": "FXTC", "CFBundleVersion": version,
            "CFBundleShortVersionString": version, "CSResourcesFileMapped": True,
        }
        with (contents / "Info.plist").open("wb") as stream:
            plistlib.dump(info, stream)
        # These describe the packaging environment, not proven build provenance:
        # `package` also accepts an already built and host-verified artifact.
        packaging_context = {
            "commit": run("git", "-C", str(ROOT), "rev-parse", "HEAD"),
            "dirty": bool(run("git", "-C", str(ROOT), "status", "--porcelain")),
            "rustc": run("rustc", "--version"), "cargo": run("cargo", "--version"),
            "target": "aarch64-apple-darwin", "source_binary_sha256": sha(binary),
            "cargo_lock_sha256": sha(ROOT / "Cargo.lock"),
            "packaged_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        }
        (contents / "Resources/packaging-context.json").write_text(
            json.dumps(packaging_context, indent=2) + "\n")
        # Ad-hoc signature; ADR-0044 permits a clearly labelled, non-notarized release.
        run("/usr/bin/codesign", "--force", "--sign", "-", "--options", "runtime",
            "--timestamp=none", str(stage))
        verify(stage)
        if bundle.exists():
            shutil.rmtree(bundle)
        shutil.move(str(stage), str(bundle))
    report = verify(bundle)
    report["packaging_context"] = packaging_context
    bundle.with_suffix(".verification.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def install(year, bundle):
    require(sys.platform == "darwin", "Installation requires macOS")
    root = Path("/Applications") / ("Adobe After Effects " + year)
    app = root / ("Adobe After Effects " + year + ".app")
    require(app.is_dir(), "After Effects " + year + " is not installed")
    processes = run("/bin/ps", "-axo", "comm=").splitlines()
    require(not any("/After Effects" in proc or Path(proc).name == "aerender"
                    for proc in processes), "Close After Effects and aerender before installing")
    # `sudo` elevates only installation. Still inspect the invoking user's
    # shared plug-ins, not root's otherwise unrelated Library directory.
    invoking_home = (Path(pwd.getpwnam(os.environ["SUDO_USER"]).pw_dir)
                     if os.environ.get("SUDO_USER") else Path.home())
    for base in [Path("/Library/Application Support"), invoking_home / "Library/Application Support"]:
        shared = base / "Adobe/Common/Plug-ins/7.0/MediaCore"
        require(not shared.exists() or not any("dynamicfx" in p.name.lower() for p in shared.rglob("*")),
                "A shared MediaCore DynamicFx copy exists; move it explicitly before installing")
    source_report = verify(bundle)
    destination = root / "Plug-ins/DynamicFx/DynamicFx.plugin"
    for candidate in (root / "Plug-ins").rglob("*"):
        require(not (candidate.suffix == ".plugin" and "dynamicfx" in candidate.name.lower()
                     and candidate != destination), "Another DynamicFx bundle exists: " + str(candidate))
    require(not destination.is_symlink(), "Installation destination must not be a symlink")
    writable_parent = destination.parent
    while not writable_parent.exists():
        writable_parent = writable_parent.parent
    require(os.access(writable_parent, os.W_OK),
            "Adobe's plug-in directory requires administrator access. After building as your normal user, "
            "run only the installer with sudo: sudo bash scripts/install.sh " + year)
    destination.parent.mkdir(parents=True, exist_ok=True)
    backup = None
    if destination.exists():
        backup = Path(tempfile.mkdtemp(prefix="DynamicFx-backup-", dir="/private/tmp")) / "DynamicFx.plugin"
        shutil.copytree(destination, backup)
    with tempfile.TemporaryDirectory(prefix=".install-", dir=destination.parent) as tmp:
        stage = Path(tmp) / "DynamicFx.plugin"
        shutil.copytree(bundle, stage)
        verify(stage)
        retired = Path(tmp) / "previous.plugin"
        if destination.exists():
            destination.rename(retired)
        try:
            stage.rename(destination)
            installed = verify(destination)
            require(installed["binary_sha256"] == source_report["binary_sha256"] and
                    installed["resource_sha256"] == source_report["resource_sha256"],
                    "Installed artifact hash mismatch")
        except Exception:
            if destination.exists():
                shutil.rmtree(destination)
            if retired.exists():
                retired.rename(destination)
            raise
    installed["backup"] = str(backup) if backup else None
    return installed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    pack = sub.add_parser("package", help="Package existing artifacts; does not rebuild or install")
    pack.add_argument("artifact_dir", type=Path)
    pack.add_argument("--output", type=Path, default=DEFAULT_BUNDLE)
    check = sub.add_parser("verify", help="Validate Mach-O, symbols, PiPL, dependencies and signature")
    check.add_argument("bundle", type=Path, nargs="?", default=DEFAULT_BUNDLE)
    install_parser = sub.add_parser("install", help="Verify and install into one explicit AE year")
    install_parser.add_argument("year", choices=["2023", "2024", "2025", "2026"])
    install_parser.add_argument("--bundle", type=Path, default=DEFAULT_BUNDLE)
    args = parser.parse_args()
    try:
        if args.command == "package":
            report = package(args.artifact_dir.resolve(), args.output.resolve())
        elif args.command == "verify":
            report = verify(args.bundle)
        else:
            report = install(args.year, args.bundle)
        print(json.dumps(report, indent=2))
    except (OSError, ValueError, KeyError, struct.error, subprocess.CalledProcessError) as exc:
        print("ERROR: " + str(exc), file=sys.stderr)
        if isinstance(exc, subprocess.CalledProcessError):
            print(exc.output, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
