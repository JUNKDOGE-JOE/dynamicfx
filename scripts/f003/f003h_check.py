#!/usr/bin/env python3
"""TR-0037-001 pixel gate for the f003h leg (ADR-0037, public issue #5).

Reads the two probe PSDs the leg rendered and asserts the encoded parameter
values arrived in the shader unclamped. Expected values are exact 8-bit
integers by construction (see f003h_range.jsx); tolerance 1 absorbs the
host's rounding of the .5 boundaries it never actually reaches here.

Usage:
  python scripts/f003/f003h_check.py scripts/out/f003/2025
Exit 0 when both frames match, 1 otherwise. The thermal PSD is visual
evidence only and is not gated here.
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "m1"))
from check_psd_rgb import read_rgb  # noqa: E402

# (file tag, expected rgb8, what it encodes)
EXPECT = [
    ("f003h_defaults_00000.psd", (51, 51, 153), "defaults wide=40 neg=-0.6 count=60"),
    ("f003h_set_00000.psd", (191, 153, 204), "set wide=150 neg=0.2 count=80"),
]
# What the 0.0.3 artifact produced (clamped to 1.0 / 0.0 / 10): a stale
# binary or a served-from-cache frame reads as this, never as the values above.
OLD = (1, 128, 26)
TOL = 1


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    out_dir = sys.argv[1]
    ok_all = True
    for name, exp, what in EXPECT:
        path = os.path.join(out_dir, name)
        if not os.path.exists(path):
            print(f"MISSING {path}")
            ok_all = False
            continue
        # Sample the frame centre; the encoding is uniform across the frame.
        raw, as8, depth, w, h = read_rgb(path, 80, 60)
        ok = all(abs(a - e) <= TOL for a, e in zip(as8, exp))
        stale = all(abs(a - o) <= TOL for a, o in zip(as8, OLD))
        verdict = "PASS" if ok else ("FAIL (matches the OLD clamped artifact)" if stale else "FAIL")
        print(f"{name} ({w}x{h}, {depth}-bit) rgb8={as8} expect={list(exp)} [{what}] -> {verdict}")
        ok_all &= ok
    print("RESULT", "PASS" if ok_all else "FAIL")
    return 0 if ok_all else 1


if __name__ == "__main__":
    sys.exit(main())
