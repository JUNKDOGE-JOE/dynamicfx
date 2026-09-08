"""Render the continuous Siri example across depths, resolutions and subframes."""

import argparse
import hashlib
import json
from pathlib import Path
import os
import subprocess

import numpy as np
from PIL import Image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--background', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    source = (repo / 'examples/siri-reference.glsl').read_text()
    fixture = source.replace('pill_center.y * R.y', '93.0').replace('pill_center * R', 'vec2(582.0,93.0)')
    fixture = fixture.replace('center.x * R.x', '582.0').replace('center.y * R.y', '179.4')
    shader = out / 'fixture.glsl'
    shader.write_text(fixture)
    background = np.fromfile(args.background, '<f4').reshape(2532, 1170, 4)
    binary = repo / 'scripts/quality/target/debug/dynamicfx-quality.exe'
    runs, frames = [], {}
    for scale, width, height in [(1, 1170, 2532), (2, 585, 1266), (4, 293, 633)]:
        path = out / f'input-{scale}.f32'
        np.stack([np.asarray(Image.fromarray(background[:, :, c]).resize((width, height), Image.Resampling.BOX))
                  for c in range(4)], axis=-1).astype('<f4').tofile(path)
        times = [1 + 1 / 60, 10, 10 + 1 / 60] if scale == 1 else [1 + 1 / 60]
        for time in times:
            for depth in ([8, 16, 32] if time < 2 else [16]):
                raw = out / 'frame.f32'
                command = [str(binary), str(shader), str(raw), str(width), str(height), '1170', '2532',
                           str(time), str(depth), 'rgba:' + str(path)]
                env = dict(os.environ, DFX_QUALITY_BENCH_FRAMES='12' if scale == 1 and depth == 16 and time < 2 else '0')
                result = subprocess.run(command, capture_output=True, text=True, timeout=45, env=env)
                record = dict(scale=scale, depth=depth, time=time, command=command, exit=result.returncode,
                              stdout=result.stdout, stderr=result.stderr)
                runs.append(record)
                (out / 'runs.json').write_text(json.dumps(runs, indent=2))
                if result.returncode:
                    raise RuntimeError(result.stderr)
                pixels = np.fromfile(raw, '<f4').reshape(height, width, 4)
                assert np.isfinite(pixels).all()
                frames[(scale, depth, time)] = pixels
                print(json.dumps(dict(scale=scale, depth=depth, time=time, finite=True)), flush=True)
    a = frames[(1, 16, 10)]
    b = frames[(1, 16, 10 + 1 / 60)]
    motion = float(np.abs(a[150:270, 350:820, :3] - b[150:270, 350:820, :3]).mean())
    if motion <= 1e-4:
        raise AssertionError('The motion must continue beyond the reference duration')
    t = 1 + 1 / 60
    metrics = []
    for (scale, depth, time), pixels in frames.items():
        if time != t:
            continue
        h, w = pixels.shape[:2]
        baseline = np.stack([np.asarray(Image.fromarray(frames[(1, 16, t)][:, :, c]).resize((w, h), Image.Resampling.BOX))
                             for c in range(3)], axis=-1)
        y, x = slice(round(150*h/2532), round(270*h/2532)), slice(round(350*w/1170), round(820*w/1170))
        error = np.abs(pixels[y, x, :3] - baseline[y, x])
        metrics.append(dict(scale=scale, depth=depth, mae=float(error.mean()), p99=float(np.percentile(error, 99))))
    (out / 'summary.json').write_text(json.dumps(dict(source_sha256=hashlib.sha256((repo / 'examples/siri-reference.glsl').read_bytes()).hexdigest(),
        normalized_source_sha256=hashlib.sha256(source.encode()).hexdigest(),
        runner_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), finite_runs=len(runs), metrics=metrics,
        motion_at_ten_seconds=motion, scope='Headless production renderer; native AE checked separately.'), indent=2))


if __name__ == '__main__':
    main()
