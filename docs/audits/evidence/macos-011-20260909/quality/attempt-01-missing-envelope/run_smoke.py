#!/usr/bin/env python3
"""Narrow v0.1.1 Metal valid-shader smoke; no AE or fault injection."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
from datetime import datetime, timezone

import numpy as np
from PIL import Image


ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
BINARY = ROOT / "scripts/quality/target/aarch64-apple-darwin/debug/dynamicfx-quality"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    cases = [
        (f"{lang}-d{depth}", OUT / "fixtures" / f"smoke.{lang}",
         321, 239, 321, 239, 1.0, depth)
        for depth in (8, 32) for lang in ("glsl", "wgsl")
    ]
    # Exact tagged example; point defaults remain the runner's zero values.
    # This checks all eight real passes, not AE fixture positioning or quality.
    cases.append(("siri-reference-d32", ROOT / "examples/siri-reference.glsl",
                  585, 1266, 1170, 2532, 1.0, 32))
    environment = os.environ.copy()
    for key in ("DYNAMICFX_BACKEND", "DFX_QUALITY_RENDER_SOURCE", "DFX_QUALITY_BENCH_FRAMES"):
        environment.pop(key, None)
    runs, checks, images = [], [], {}

    def check(name, value, **measurements):
        checks.append(dict(name=name, passed=bool(value), **measurements))

    for name, source, width, height, logical_width, logical_height, time, depth in cases:
        output = OUT / (name + ".f32")
        command = [str(BINARY), str(source), str(output), str(width), str(height),
                   str(logical_width), str(logical_height), str(time), str(depth), "ramp"]
        result = subprocess.run(command, cwd=ROOT, env=environment,
                                capture_output=True, text=True, timeout=60)
        log = dict(command=command, exit_code=result.returncode,
                   stdout=result.stdout, stderr=result.stderr)
        (OUT / (name + ".log")).write_text(json.dumps(log, indent=2) + "\n")
        runs.append(log)
        (OUT / "smoke-executions.json").write_text(json.dumps(runs, indent=2) + "\n")
        if result.returncode:
            raise RuntimeError(name + " failed; original log retained")
        pixels = np.fromfile(output, dtype="<f4").reshape(height, width, 4)
        images[name] = pixels
        metadata = json.loads(result.stdout.strip().splitlines()[-1])
        log.update(metadata=metadata, source_sha256=sha(source), output_sha256=sha(output))
        check(name + " actual Metal", "backend=Metal" in metadata["adapter"])
        check(name + " finite", np.isfinite(pixels).all())
        check(name + " alpha", np.all(pixels[:, :, 3] == 1.0))
        Image.fromarray(np.rint(np.clip(pixels, 0, 1) * 255).astype("uint8"),
                        "RGBA").save(OUT / (name + ".png"))
        if name.startswith("siri-reference"):
            check(name + " eight passes", metadata["passes"] == 8)
            ramp = np.arange(width, dtype=np.float32)[None, :, None] / (width - 1)
            difference = float(np.abs(pixels[:, :, :3] - ramp).max())
            check(name + " modifies input", difference > 0.0001, max_abs=difference)
        else:
            reference = np.empty_like(pixels)
            reference[:, :, 0] = np.arange(width)[None, :] / (width - 1)
            reference[:, :, 1] = (np.arange(height)[:, None] + 0.5) / height
            reference[:, :, 2] = 0.25
            reference[:, :, 3] = 1.0
            error = float(np.abs(reference - pixels).max())
            check(name + " numeric oracle", error < (1 / 255 + 1e-6 if depth == 8 else 2e-6),
                  max_abs=error)
    for depth in (8, 32):
        check(f"GLSL WGSL d{depth} exact pixels",
              np.array_equal(images[f"glsl-d{depth}"], images[f"wgsl-d{depth}"]))
    report = dict(
        timestamp_utc=datetime.now(timezone.utc).isoformat(),
        source_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        root_lock_sha256=sha(ROOT / "Cargo.lock"),
        quality_lock_sha256=sha(ROOT / "scripts/quality/Cargo.lock"),
        binary_sha256=sha(BINARY),
        source_files={str(p.relative_to(ROOT)): sha(p) for p in [
            ROOT / "src/render.rs", ROOT / "src/frontend/glsl.rs", ROOT / "src/frontend/wgsl.rs",
            ROOT / "scripts/quality/src/main.rs", ROOT / "examples/siri-reference.glsl"]},
        scope="Metal valid-shader smoke only; no AE, no Windows FXC error recovery, no calibrated Siri visual acceptance. Exact Siri source uses zero-valued point defaults in this runner.",
        renders=len(cases), checks=checks, passed=all(c["passed"] for c in checks), runs=runs,
    )
    (OUT / "smoke-summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(dict(renders=len(cases), checks=len(checks), passed=report["passed"],
                         failures=[c for c in checks if not c["passed"]])))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
