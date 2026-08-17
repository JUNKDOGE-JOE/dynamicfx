#!/usr/bin/env python3
"""TR-M5-001 numeric gate over the m5all.log sampleImage probes (ADR-0021).

Usage: python check_probes.py <path-to-m5all.log>

Parses `PROBE <comp> <tag> bpc<N> = <float>` lines (occurrences kept in
order) and asserts the ADR-0021 fixture matrix. Prints one CHECK line per
assertion ending in PASS/FAIL/RECORD; exits non-zero when any gate fails.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict

# End-to-end 16-bpc expectation (ADR-0022 f32 working format): v stays
# f32 through the pipeline -> round(v*32768) once at the boundary ->
# /32768 in sampleImage. 0.3140625 lands on 10291/32768 = 0.31405640;
# tolerance 2e-4 still rejects the 8-bit value 80/255 = 0.31372549
# (off by 3.4e-4).
RAMP_AT_100 = 0.3140625
RAMP_AT_9 = 0.0296875
RAMP_AT_10 = 0.0328125


def main() -> int:
    text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    occ: dict[tuple[str, str, int], list[float]] = defaultdict(list)
    for m in re.finditer(r"^PROBE (\S+) (\S+) bpc(\d+) = (-?[\d.]+)$", text, re.M):
        occ[(m.group(1), m.group(2), int(m.group(3)))].append(float(m.group(4)))

    fails = 0

    def get(comp, tag, bpc, n=0):
        vals = occ.get((comp, tag, bpc), [])
        return vals[n] if len(vals) > n else None

    def check(label, value, expected, tol):
        nonlocal fails
        if value is None:
            print(f"CHECK {label} MISSING FAIL")
            fails += 1
            return
        ok = abs(value - expected) <= tol
        print(f"CHECK {label} = {value:.9f} expect {expected:.9f} tol {tol} {'PASS' if ok else 'FAIL'}")
        if not ok:
            fails += 1

    def stair(label, lo, hi, expect_distinct):
        nonlocal fails
        if lo is None or hi is None:
            print(f"CHECK {label} MISSING FAIL")
            fails += 1
            return
        distinct = (hi - lo) > 1e-6
        ok = distinct == expect_distinct
        kind = "distinct" if distinct else "collapsed"
        print(f"CHECK {label} {lo:.9f} vs {hi:.9f} -> {kind} {'PASS' if ok else 'FAIL'}")
        if not ok:
            fails += 1

    # 16-bpc, multi-pass: exact fractions + the staircase pair through the
    # 3-pass chain (intermediate-format evidence). The direct m5ramp comp's
    # probe sliders never evaluate at 16-bpc in this scripted session (a
    # measured host caching quirk recorded in the audit; the same comp is
    # fully probed at 8-bpc below), so single-pass 16-bpc evidence rides
    # the HDR generator instead, which also pins the ADR-0022 clamp.
    comp = "m5chain"
    check(f"{comp}@16 p100", get(comp, "p100", 16), RAMP_AT_100, 2e-4)
    check(f"{comp}@16 p9", get(comp, "p9", 16), RAMP_AT_9, 2e-4)
    check(f"{comp}@16 white", get(comp, "white", 16), 1.0, 1e-4)
    stair(f"{comp}@16 stair(9,10)", get(comp, "p9", 16), get(comp, "p10", 16), True)
    # 16-bpc, single-pass + boundary clamp (ADR-0022 §2): over-white and
    # negatives cannot exist in U15.
    check("m5hdr@16 over(clamped)", get("m5hdr", "over", 16), 1.0, 1e-4)
    check("m5hdr@16 neg(clamped)", get("m5hdr", "neg", 16), 0.0, 1e-4)
    check("m5hdr@16 one", get("m5hdr", "one", 16), 1.0, 1e-4)
    check("m5hdr@16 ramp", get("m5hdr", "ramp", 16), RAMP_AT_100, 2e-4)

    # 32-bpc: over-white and negative survival, direct and chained.
    for comp in ("m5hdr", "m5hdrchain"):
        check(f"{comp}@32 over", get(comp, "over", 32), 2.0, 1e-3)
        check(f"{comp}@32 neg", get(comp, "neg", 32), -0.5, 1e-3)
        check(f"{comp}@32 ramp", get(comp, "ramp", 32), RAMP_AT_100, 5e-4)
    check("m5hdr@32 one", get("m5hdr", "one", 32), 1.0, 1e-4)

    # 8-bpc canary: same shader, 8-bit tolerances; the probe pair MUST
    # collapse at 8-bit (validates the 16-bpc staircase is discriminating).
    check("m5ramp@8 p100", get("m5ramp", "p100", 8), RAMP_AT_100, 2.5e-3)
    check("m5ramp@8 white", get("m5ramp", "white", 8), 1.0, 1e-4)
    stair("m5ramp@8 stair(9,10)", get("m5ramp", "p9", 8), get("m5ramp", "p10", 8), False)

    # Alpha semantics per depth: measured, must be cleanly one of the two.
    for bpc in (8, 16, 32):
        rec, white = get("m5alpha", "rec", bpc), get("m5alpha", "white", bpc)
        s_r, s_a = get("m5alpha", "srcR", bpc), get("m5alpha", "srcA", bpc)
        if rec is None or white in (None, 0):
            print(f"CHECK alpha@{bpc} MISSING FAIL")
            fails += 1
            continue
        frac = rec / white
        if abs(frac - 0.5) <= 0.02:
            sem = "premultiplied"
        elif abs(frac - 1.0) <= 0.02:
            sem = "straight"
        else:
            print(f"CHECK alpha@{bpc} rec/white = {frac:.6f} matches neither FAIL (src r={s_r} a={s_a})")
            fails += 1
            continue
        print(f"CHECK alpha@{bpc} rec/white = {frac:.6f} SEMANTICS {sem} (src r={s_r} a={s_a}) PASS")

    # Color pair: second bpc32 occurrence = step-5 unmanaged re-read (gate);
    # third = managed leg (record only; absent when no space was accepted).
    check("m5hdr@32 unmanaged over", get("m5hdr", "over", 32, 1), 2.0, 1e-3)
    managed = get("m5hdr", "over", 32, 2)
    if managed is None:
        print("CHECK m5hdr@32 managed RECORD (no managed leg; see COLOR lines in log)")
    else:
        print(f"CHECK m5hdr@32 managed over = {managed:.9f} RECORD")

    print(f"CHECKS_RESULT fails={fails}")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
