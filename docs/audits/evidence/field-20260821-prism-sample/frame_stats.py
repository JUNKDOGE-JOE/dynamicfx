"""Per-frame statistics for a PNG sequence: flags frames whose coverage or
luminance drops sharply against both neighbours (a "missing frame")."""
import sys, os, glob, re
import numpy as np
from PIL import Image

def load_rgba01(path):
    if path.lower().endswith(".psd"):
        from psd_read import read_psd_rgba
        return read_psd_rgba(path)
    im = Image.open(path).convert("RGBA")
    return np.asarray(im).astype(np.float32) / 255.0

def stats(path):
    a = load_rgba01(path)
    alpha = a[..., 3]
    luma = (0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2])
    return {
        "w": a.shape[1], "h": a.shape[0],
        "alpha_mean": float(alpha.mean()),
        "coverage": float((alpha > 0.004).mean()),
        "luma_mean": float(luma.mean()),
        "luma_p99": float(np.percentile(luma, 99)),
        "nonblack": float(((luma > 0.02) & (alpha > 0.004)).mean()),
    }

def _safe_stats(f):
    try:
        return stats(f), None
    except Exception as e:
        return None, str(e)

def main(folder, pattern="*.png"):
    files = sorted(glob.glob(os.path.join(folder, pattern)), key=lambda p: [int(t) if t.isdigit() else t for t in re.split(r"(\d+)", os.path.basename(p))])
    from multiprocessing import Pool
    with Pool(min(12, os.cpu_count() or 4)) as pool:
        results = pool.map(_safe_stats, files)
    rows = [(os.path.basename(f), s, err) for f, (s, err) in zip(files, results)]
    print(f"{'frame':>28} {'alpha':>7} {'cover':>7} {'luma':>7} {'p99':>7} {'nonblk':>7}")
    flagged = []
    vals = [r[1] for r in rows]
    for i, (name, s, err) in enumerate(rows):
        if s is None:
            print(f"{name:>28} ERROR {err}")
            flagged.append((name, "unreadable"))
            continue
        mark = ""
        prev_ = vals[i - 1] if i > 0 else None
        next_ = vals[i + 1] if i + 1 < len(vals) else None
        neigh = [v for v in (prev_, next_) if v]
        if neigh:
            ref_cov = np.median([v["coverage"] for v in neigh])
            ref_nb = np.median([v["nonblack"] for v in neigh])
            if (ref_cov > 0.01 and s["coverage"] < 0.5 * ref_cov) or (ref_nb > 0.01 and s["nonblack"] < 0.5 * ref_nb):
                mark = "  <-- DROP"
                flagged.append((name, f"coverage {s['coverage']:.3f} vs {ref_cov:.3f}, nonblack {s['nonblack']:.3f} vs {ref_nb:.3f}"))
        if s["coverage"] == 0.0:
            mark = "  <-- EMPTY"
            if not flagged or flagged[-1][0] != name:
                flagged.append((name, "fully transparent"))
        print(f"{name:>28} {s['alpha_mean']:7.3f} {s['coverage']:7.3f} {s['luma_mean']:7.3f} {s['luma_p99']:7.3f} {s['nonblack']:7.3f}{mark}")
    print(f"\n{len(rows)} frames, {len(flagged)} flagged")
    for name, why in flagged:
        print(f"  {name}: {why}")
    return flagged

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "*.png")
