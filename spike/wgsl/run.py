#!/usr/bin/env python3
"""Run the non-shipping conformance/GPU probe and preserve reproducible evidence.

Requires Python 3 stdlib, cached Cargo dependencies, Rust stable, and GPU access.
No After Effects operation or plugin install is performed.
"""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import sys
import zlib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
EVIDENCE = HERE / "evidence"
OUT = HERE / "out"


def capture(command):
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def run(command, log_name):
    with (EVIDENCE / log_name).open("w") as log:
        process = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    print(f"exit={process.returncode}: {' '.join(command)}; log=spike/wgsl/evidence/{log_name}")
    if process.returncode:
        print((EVIDENCE / log_name).read_text()[-4000:])
        raise SystemExit(process.returncode)


def png_rgba(raw, width, height):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    rows = b"".join(b"\0" + raw[y * width * 4:(y + 1) * width * 4] for y in range(height))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


def main():
    EVIDENCE.mkdir(exist_ok=True)
    OUT.mkdir(exist_ok=True)
    source_paths = [
        "src/render.rs", "src/plan.rs", "src/binding.rs", "src/definition/effect.rs",
        "src/definition/param.rs", "src/frontend/mod.rs", "src/frontend/annotation.rs",
        "src/frontend/envelope.rs", "src/frontend/glsl.rs", "src/frontend/grammar.rs",
        "spike/wgsl/src/main.rs", "spike/wgsl/Cargo.toml", "spike/wgsl/Cargo.lock",
        "spike/wgsl/run.py",
    ] + [str(p.relative_to(ROOT)) for p in sorted((HERE / "fixtures").glob("*"))]
    info = {
        "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baseline_commit": capture(["git", "rev-parse", "HEAD"]),
        "branch": capture(["git", "branch", "--show-current"]),
        "os": platform.platform(), "architecture": platform.machine(),
        "rust": capture(["rustc", "+stable", "--version"]),
        "naga": "29.0.4", "wgpu": "29.0.4", "ae_run": False,
        "backend_override": os.environ.get("DYNAMICFX_BACKEND"),
        "source_sha256": {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in source_paths},
    }
    (EVIDENCE / "run.json").write_text(json.dumps(info, indent=2) + "\n")
    common = ["--offline", "--locked", "--manifest-path", "spike/wgsl/Cargo.toml"]
    run(["cargo", "+stable", "test", *common], "tests.log")
    run(["cargo", "+stable", "run", *common, "--", "spike/wgsl/out"], "gpu.log")
    artifacts = {}
    for path in sorted(OUT.glob("*.rgba")):
        raw = path.read_bytes()
        (EVIDENCE / (path.name + ".zlib")).write_bytes(zlib.compress(raw, level=9))
        artifacts[path.name] = {"raw_bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
        if path.name.startswith("U8-"):
            (EVIDENCE / (path.stem + ".png")).write_bytes(png_rgba(raw, 256, 128))
    for path in OUT.glob("*.metal"):
        (EVIDENCE / path.name).write_bytes(path.read_bytes())
    info["artifacts"] = artifacts
    info["finished_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    info["result"] = "PASS"
    (EVIDENCE / "run.json").write_text(json.dumps(info, indent=2) + "\n")
    print("RESULT=PASS; raw framebuffers are zlib streams, tightly packed 256x128 RGBA (U8 or little-endian float32)")


if __name__ == "__main__":
    main()
