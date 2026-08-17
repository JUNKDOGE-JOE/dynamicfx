#!/usr/bin/env python3
"""TR-M6-001 numeric gate for ADR-0025 windowed re-simulation.

Usage: python check_m6.py <out-dir>

The law under test: value(F) = min(F+1, W) * step — exact, at any depth,
in ANY evaluation order, identically in interactive probes, the
MFR-concurrent render queue, and a fresh aerender process.
"""

from __future__ import annotations

import os
import re
import struct
import sys


def unpackbits(raw, want):
    out = bytearray()
    i = 0
    while i < len(raw) and len(out) < want:
        n = raw[i]
        i += 1
        if n < 128:
            out += raw[i:i + n + 1]
            i += n + 1
        elif n > 128:
            out += bytes([raw[i]]) * (257 - n)
            i += 1
    return bytes(out)


def psd_value(path, x, y, chan=0):
    d = open(path, "rb").read()
    if d[:4] != b"8BPS":
        raise ValueError("not a PSD")
    channels = struct.unpack(">H", d[12:14])[0]
    height = struct.unpack(">I", d[14:18])[0]
    width = struct.unpack(">I", d[18:22])[0]
    depth = struct.unpack(">H", d[22:24])[0]
    pos = 26
    for _ in range(3):
        ln = struct.unpack(">I", d[pos:pos + 4])[0]
        pos += 4 + ln
    comp = struct.unpack(">H", d[pos:pos + 2])[0]
    pos += 2
    bps = depth // 8
    row_bytes = width * bps
    if comp == 0:
        off = pos + chan * row_bytes * height + y * row_bytes + x * bps
        return int.from_bytes(d[off:off + bps], "big")
    if comp == 1:
        nrows = channels * height
        counts = struct.unpack(f">{nrows}H", d[pos:pos + 2 * nrows])
        off = pos + 2 * nrows + sum(counts[:chan * height + y])
        row = unpackbits(d[off:off + counts[chan * height + y]], row_bytes)
        return int.from_bytes(row[x * bps:x * bps + bps], "big")
    raise ValueError(f"compression {comp}")


def main() -> int:
    out = sys.argv[1]
    text = open(os.path.join(out, "m6all.log"), encoding="utf-8", errors="replace").read()

    fails = 0

    def check(label, value, expected, tol=1e-6):
        nonlocal fails
        if value is None:
            print(f"CHECK {label} MISSING FAIL")
            fails += 1
            return
        ok = abs(value - expected) <= tol
        print(f"CHECK {label} = {value:.9f} expect {expected:.9f} {'PASS' if ok else 'FAIL'}")
        if not ok:
            fails += 1

    # Probe reads in scripted order; expectations from the ADR-0025 law.
    # Sections are delimited by the STEP lines so the recompiled window
    # applies only to the post-recompile reads.
    reads: list[tuple[str, str, int, float]] = []  # (section, comp, frame, value)
    section = "pre"
    for line in text.splitlines():
        if line.startswith("STEP4"):
            section = "post"
        m = re.match(r"PROBE (\S+) frame (\d+) = (-?[\d.]+)$", line)
        if m:
            reads.append((section, m.group(1), int(m.group(2)), float(m.group(3))))

    def law(frame, window, step):
        return min(frame + 1, window) * step

    for section, comp, frame, value in reads:
        if comp == "m6acc8":
            check(f"acc8 f{frame} (shuffled)", value, law(frame, 16, 4 / 255))
        elif comp == "m6acc32" and section == "pre":
            check(f"acc32 f{frame} (shuffled)", value, law(frame, 16, 1 / 64))
        elif comp == "m6acc32":
            check(f"acc32 f{frame} (@window 8)", value, law(frame, 8, 1 / 64))

    # RQ (MFR-concurrent) and aerender sequences: frame k -> min(k+1,16)*4.
    for tag, pat in (("rq", "m6_rq_{:05d}.psd"), ("ar", "m6_ar_{:05d}.psd")):
        bad = 0
        for k in range(25):
            p = os.path.join(out, pat.format(k))
            if not os.path.exists(p):
                print(f"CHECK {tag} frame {k} MISSING FAIL")
                fails += 1
                continue
            v = psd_value(p, 160, 120)
            expected = min(k + 1, 16) * 4
            ok = v == expected
            if not ok or k in (0, 15, 24):
                print(f"CHECK {tag} f{k} = {v} expect {expected} {'PASS' if ok else 'FAIL'}")
            if not ok:
                fails += 1
                bad += 1
        print(f"CHECK {tag} sequence: 25 frames, {bad} wrong")

    plug = os.path.join(out, "dynamicfx_plugin.log")
    if os.path.exists(plug):
        ptext = open(plug, encoding="utf-8", errors="replace").read()
        n = len(re.findall(r"temporal window:", ptext))
        print(f"RECORD windowed renders logged: {n}")

    print(f"CHECKS_RESULT fails={fails}")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
