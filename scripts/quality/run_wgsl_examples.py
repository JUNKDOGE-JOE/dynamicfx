#!/usr/bin/env python3
"""Render shipped WGSL examples through the production quality runner.

Requires NumPy and Pillow. Build scripts/quality first. This tests actual
Metal output, not the former WGSL spike's generated GLSL interface. Results
are headless GPU evidence; U15 uses the f32 working buffer, not AE's boundary.
"""
import argparse
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[2]


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preview(data):
    # Presentation only: the .f32 dump retains unbounded original values.
    return Image.fromarray(np.round(np.clip(data, 0, 1) * 255).astype(np.uint8), "RGBA")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path,
                        default=ROOT / "scripts/quality/target/debug/dynamicfx-quality")
    parser.add_argument("--output", type=Path,
                        default=ROOT / "scripts/out/010/wgsl-examples")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    binary = args.binary.resolve()
    sources = [ROOT / "examples/wgsl-field.wgsl", ROOT / "examples/wgsl-multipass.wgsl"]
    render_records = []
    images = {}
    checks = []

    def check(name, passed, **measurements):
        checks.append(dict(name=name, passed=bool(passed), **measurements))

    def render(source, scale, depth, time, suffix="", pattern="ramp"):
        w, h = 1280 // scale, 720 // scale
        name = f"{source.stem}-s{scale}-d{depth}-t{time:g}{suffix}"
        dump = out / (name + ".f32")
        command = [str(binary), str(source), str(dump), str(w), str(h),
                   "1280", "720", str(time), str(depth), pattern]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        with (out / "renders.log").open("a", encoding="utf-8") as log:
            log.write(json.dumps({"command": command, "exit_code": result.returncode}) + "\n")
            log.write(result.stdout + result.stderr + "\n")
        if result.returncode:
            raise RuntimeError(f"{name} failed; preserved in {out / 'renders.log'}")
        record = json.loads(result.stdout.strip().splitlines()[-1])
        record.update(output_sha256=sha256(dump), input_pattern=pattern)
        data = np.fromfile(dump, dtype="<f4").reshape(h, w, 4)
        record.update(rgb_min=float(data[:, :, :3].min()), rgb_max=float(data[:, :, :3].max()),
                      rgb_std=float(data[:, :, :3].std()))
        render_records.append(record)
        check(name + ": finite", np.isfinite(data).all())
        check(name + ": opaque input/output", np.all(data[:, :, 3] == 1.0))
        preview(data).save(out / (name + ".png"))
        return data

    for source in sources:
        for scale in (1, 2, 4):
            for depth in (8, 16, 32):
                # Deliberately request t=1.25 before t=0, then re-request t=1.25.
                for time in (1.25, 0.0):
                    images[source.stem, scale, depth, time] = render(source, scale, depth, time)
            d8 = images[source.stem, scale, 8, 1.25]
            d16 = images[source.stem, scale, 16, 1.25]
            d32 = images[source.stem, scale, 32, 1.25]
            error8 = np.abs(d8 - np.clip(d32, 0, 1))
            check(f"{source.stem} scale {scale}: f32 working-depth identity",
                  np.array_equal(d16, d32))
            # One/two ordinary color passes: this bounds the measured rounded
            # 8-bpc result, not arbitrary shaders or a claim about AE U15 I/O.
            check(f"{source.stem} scale {scale}: 8-bpc mean error",
                  float(error8.mean()) < 0.006,
                  mean_abs_error=float(error8.mean()), max_abs_error=float(error8.max()))
        again = render(source, 1, 32, 1.25, suffix="-repeat")
        initial = images[source.stem, 1, 32, 1.25]
        check(source.stem + ": repeated requested time is identical", np.array_equal(initial, again))
        change = float(np.abs(initial - images[source.stem, 1, 32, 0.0]).mean())
        check(source.stem + ": time changes the field", change > 0.001, mean_abs_change=change)
        render(source, 1, 32, 1.25, suffix="-black", pattern="black")

    scale_measurements = []
    contact = Image.new("RGB", (960, 2 * 390), (25, 25, 25))
    draw = ImageDraw.Draw(contact)
    for row, source in enumerate(sources):
        full = images[source.stem, 1, 32, 1.25]
        for col, scale in enumerate((1, 2, 4)):
            data = images[source.stem, scale, 32, 1.25]
            reduced = full.reshape(720 // scale, scale, 1280 // scale, scale, 4).mean(axis=(1, 3))
            error = np.abs(data - reduced)
            # Descriptive reference, not an AA proof: full image is not a
            # supersampled ground truth; the ramp's texel grid also changes.
            scale_measurements.append(dict(source=source.name, scale=scale,
                                           mean_abs_vs_full_box=float(error.mean()),
                                           max_abs_vs_full_box=float(error.max())))
            image = preview(data)
            contact.paste(image.resize((320, 180), Image.Resampling.LANCZOS), (col * 320, row * 390 + 25))
            # Same logical 160x80 crop around the right-hand contour, enlarged
            # nearest-neighbor to reveal actual fragment coverage and cells.
            crop = image.crop((800 // scale, 320 // scale, 960 // scale, 400 // scale))
            contact.paste(crop.resize((320, 160), Image.Resampling.NEAREST), (col * 320, row * 390 + 220))
            draw.text((col * 320 + 8, row * 390 + 5), f"{source.stem}  1/{scale}  32-bpc", fill="white")
    contact.save(out / "contact-sheet.png")
    report = dict(
        timestamp_utc=datetime.now(timezone.utc).isoformat(),
        command="scripts/quality/run_wgsl_examples.py",
        binary_sha256=sha256(binary),
        source_sha256={str(p.relative_to(ROOT)): sha256(p) for p in sources},
        frontend="production frontend_for(LanguageId::WGSL)",
        evidence_scope="headless GPU; not AE host acceptance, U15 boundary, Windows, or an exhaustive AA metric",
        logical_size=[1280, 720],
        render_count=len(render_records),
        checks=checks,
        scale_measurements=scale_measurements,
        renders=render_records,
        passed=all(c["passed"] for c in checks),
    )
    (out / "summary.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"passed": report["passed"], "renders": len(render_records),
                      "checks": len(checks), "summary": str(out / "summary.json"),
                      "contact_sheet": str(out / "contact-sheet.png")}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
