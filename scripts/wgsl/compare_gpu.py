#!/usr/bin/env python3
"""Compare production GLSL/WGSL frontends through the shared Metal renderer.

Uses the existing algorithm fixtures, not the historical spike's interface
adapter. Requires NumPy/Pillow for output checks/previews. No AE operations.
"""
import datetime
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "scripts/out/010/wgsl-gpu"
RUNNER = ROOT / "scripts/quality/target/debug/dynamicfx-quality"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def make_source(language, passes):
    fixture = ROOT / "spike/wgsl/fixtures"
    graph = "pass field: input -> output" if passes == 1 else "pass field: input -> fieldImage\npass mix: fieldImage, input -> output"
    text = f"@dynamicfx 1\n@graph\n{graph}\n@end\n"
    for name in ["field", "mix"][:passes]:
        text += f"@pass {name}\n"
        body = (fixture / f"{name}.{language}").read_text()
        # The historical spike passed one annotation map to both languages;
        # this CLI runner reads each file independently. Copy its same public
        # parameter declarations into the GLSL fixture as well.
        if language == "glsl" and name == "field":
            annotations = "\n".join(line for line in (fixture / "field.wgsl").read_text().splitlines() if line.startswith("// @param "))
            body = body.replace("#version 450\n", "#version 450\n" + annotations + "\n", 1)
        for line in body.splitlines():
            text += ("@" if line.startswith("@") else "") + line + "\n"
        text += "@endpass\n"
    path = OUT / f"{passes}pass.{language}"
    path.write_text(text)
    return path


def main():
    global OUT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=OUT)
    OUT = parser.parse_args().out.resolve()
    OUT.mkdir(parents=True, exist_ok=True)
    if (OUT / "result.json").exists():
        raise SystemExit("Existing result; preserve it and choose a new output directory before rerun")
    source_files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "src/render.rs", *sorted((ROOT / "src/frontend").glob("*.rs")), ROOT / "scripts/quality/src/main.rs"]
    source_identity = {str(p.relative_to(ROOT)): sha(p) for p in source_files}
    with (OUT / "build.log").open("w") as log:
        subprocess.run(["cargo", "+stable", "build", "--manifest-path", str(ROOT / "scripts/quality/Cargo.toml"), "--offline"], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    rows = []
    sources = {(language, passes): make_source(language, passes) for language in ["glsl", "wgsl"] for passes in [1, 2]}
    for depth in [8, 16, 32]:
        for passes in [1, 2]:
            for scale in [1, 2, 4]:
                w, h = (321 + scale - 1) // scale, (239 + scale - 1) // scale
                arrays, metadata = {}, {}
                for language in ["glsl", "wgsl"]:
                    tag = f"{language}-{depth}-{passes}pass-{scale}x"
                    output = OUT / f"{tag}.f32"
                    command = [str(RUNNER), str(sources[language, passes]), str(output), str(w), str(h), "321", "239", "1.25", str(depth), "ramp"]
                    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
                    (OUT / f"{tag}.log").write_text(result.stdout + "\n" + result.stderr)
                    if result.returncode:
                        raise RuntimeError(f"{tag} failed; inspect its retained log")
                    metadata[language] = json.loads(result.stdout.strip().splitlines()[-1])
                    arrays[language] = np.fromfile(output, dtype="<f4").reshape(h, w, 4)
                    if depth == 8 and scale == 1:
                        Image.fromarray(np.clip(arrays[language] * 255 + .5, 0, 255).astype(np.uint8)).save(OUT / f"{tag}.png")
                delta = np.abs(arrays["glsl"] - arrays["wgsl"])
                row = {"depth": depth, "passes": passes, "scale": scale, "size": [w, h], "max_abs": float(delta.max()), "mean_abs": float(delta.mean()), "byte_equal": bool(np.array_equal(arrays["glsl"], arrays["wgsl"])), "finite": bool(np.isfinite(arrays["wgsl"]).all()), "metadata": metadata}
                row["passed"] = row["finite"] and row["max_abs"] <= (0 if depth == 8 else 1e-5)
                rows.append(row)
                print(json.dumps({k: v for k, v in row.items() if k != "metadata"}), flush=True)
    unchanged = source_identity == {str(p.relative_to(ROOT)): sha(p) for p in source_files}
    report = {"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "kind": "production frontends and production GPU renderer; no AE boundary", "command": " ".join(sys.argv), "runner_sha256": sha(RUNNER), "source_sha256": source_identity, "source_unchanged_during_run": unchanged, "pairs": rows, "passed": unchanged and all(r["passed"] for r in rows)}
    (OUT / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
