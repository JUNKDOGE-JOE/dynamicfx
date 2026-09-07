#!/usr/bin/env python3
"""Check final native PSD renders at three preview scales; NumPy/Pillow required."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/m5'))
from check_deep import Psd


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('directory', type=Path)
    ap.add_argument('--reference', type=Path, required=True)
    args = ap.parse_args()
    records, checks = [], []

    def check(name, ok, **values):
        checks.append(dict(name=name, passed=bool(ok), **values))

    siri = {}
    for kind, logical, frames in [('ramp', (321, 239), [0, 1]), ('siri', (1280, 720), [25, 26])]:
        for scale, denominator in [('full', 1), ('half', 2), ('quarter', 4)]:
            for frame in frames:
                path = args.directory / f'final_{kind}_{scale}_{frame:05d}.psd'
                p = Psd(path)
                expected_size = [(v + denominator - 1) // denominator for v in logical]
                check(path.name + ' format', [p.width, p.height] == expected_size and p.depth == 8 and p.channels == 4,
                      size=[p.width, p.height], expected=expected_size, depth=p.depth, channels=p.channels)
                rgba = np.stack([np.frombuffer(plane, np.uint8).reshape(p.height, p.width) for plane in p.planes], axis=2)
                check(path.name + ' opaque', np.all(rgba[:, :, 3] == 255))
                records.append(dict(file=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
                if kind == 'ramp':
                    yy, xx = np.mgrid[:p.height, :p.width]
                    expected = np.stack([(xx + .5) / p.width, (yy + .5) / p.height, np.full_like(xx, .625, dtype=float)], axis=2)
                    error = np.abs(rgba[:, :, :3] / 255 - expected)
                    # Quantization of generator and two inversions allows one U8 level.
                    check(path.name + ' analytic UV', error.max() <= 1 / 255 + 1e-6,
                          max_error=float(error.max()), mean_error=float(error.mean()), tolerance=1 / 255 + 1e-6)
                elif frame == 25:
                    rgb = rgba[:, :, :3] / 255
                    siri[denominator] = rgb
                    Image.fromarray(rgba, 'RGBA').save(args.directory / f'final_siri_{scale}.png')
                    # Each middle edge must retain light; central content stays black.
                    h, w = rgb.shape[:2]
                    masks = [rgb[:max(1,h//8), w//3:2*w//3], rgb[7*h//8:, w//3:2*w//3],
                             rgb[h//3:2*h//3, :max(1,w//8)], rgb[h//3:2*h//3, 7*w//8:]]
                    peaks = [float(m.max()) for m in masks]
                    check(path.name + ' four edges', min(peaks) > .15, peaks=peaks, minimum=.15)
                    check(path.name + ' center', float(rgb[h//2,w//2].max()) < .05)
    ref = np.fromfile(args.reference, dtype='<f4').reshape(720, 1280, 4)[:, :, :3]
    error = np.abs(siri[1] - ref)
    check('Full native versus same-time production GPU float', error.max() <= 1 / 255 + 1e-6,
          max_error=float(error.max()), mean_error=float(error.mean()), tolerance=1 / 255 + 1e-6)
    for d in [2, 4]:
        reduced = siri[1].reshape(720//d, d, 1280//d, d, 3).mean(axis=(1, 3))
        error = np.abs(siri[d] - reduced)
        mean, p99 = float(error.mean()), float(np.quantile(error, .99))
        # Fixed Siri fixture acceptance, not a universal filter-quality bound.
        check(f'Siri 1/{d} versus box-reduced Full', mean < .003 and p99 < .03,
              mean_error=mean, p99_error=p99, mean_limit=.003, p99_limit=.03)
    result = dict(passed=all(c['passed'] for c in checks), checks=checks, outputs=records,
                  reference=dict(path=str(args.reference), sha256=hashlib.sha256(args.reference.read_bytes()).hexdigest()))
    print(json.dumps(result, indent=2))
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
