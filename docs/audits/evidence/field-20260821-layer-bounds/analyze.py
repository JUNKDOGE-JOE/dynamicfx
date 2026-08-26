"""Measure where each test tile has visible pixels.

The comp is 3072x1024: tile A (x 0..1024), B (1024..2048), C (2048..3072),
each tile's layer centred at the tile centre. The logo itself occupies the
central 512x512 square of a tile; anything visible outside that square is
shader content that needed canvas beyond the 512x512 layer bounds.
"""
import sys
import numpy as np
from PIL import Image

path = sys.argv[1]
im = np.asarray(Image.open(path).convert("RGBA")).astype(np.int32)
h, w, _ = im.shape
assert (w, h) == (3072, 1024), (w, h)
tiles = {}
for name, x0 in (("A", 0), ("B", 1024), ("C", 2048)):
    tiles[name] = im[:, x0:x0 + 1024]

def report(name, t):
    a = t[..., 3]
    vis = a > 0
    ys, xs = np.nonzero(vis)
    if len(xs) == 0:
        print(f"{name}: fully transparent")
        return
    bbox = (xs.min(), ys.min(), xs.max() + 1, ys.max() + 1)
    inner = np.zeros_like(vis)
    inner[256:768, 256:768] = True
    outside = vis & ~inner
    print(f"{name}: visible bbox x{bbox[0]}..{bbox[2]} y{bbox[1]}..{bbox[3]} "
          f"size {bbox[2]-bbox[0]}x{bbox[3]-bbox[1]}; visible px {vis.sum()}, "
          f"outside the 512 square {outside.sum()} ({100*outside.sum()/vis.sum():.1f}%)")

for n in "ABC":
    report(n, tiles[n])

d_bc = np.abs(tiles["B"] - tiles["C"])
d_ab = np.abs(tiles["A"] - tiles["C"])
print(f"B vs C: mean |diff| {d_bc.mean():.3f}, max {d_bc.max()}, px differing >8: {(d_bc.max(axis=2) > 8).sum()}")
print(f"A vs C: mean |diff| {d_ab.mean():.3f}, max {d_ab.max()}, px differing >8: {(d_ab.max(axis=2) > 8).sum()}")
# Inner 512 square only: does A match C where A could render at all?
ia = tiles["A"][256:768, 256:768]
ic = tiles["C"][256:768, 256:768]
ib = tiles["B"][256:768, 256:768]
print(f"inner square A vs C: mean |diff| {np.abs(ia-ic).mean():.3f}; B vs C: {np.abs(ib-ic).mean():.3f}")
