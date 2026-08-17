#!/usr/bin/env python3
"""Minimal PSD RGB probe for the M1 aerender leg (TR-M1-004).

Same dependency-free flattened-PSD reader as scripts/spike/check_psd.py, but
sampling the R/G/B channel planes of one pixel. AE's aerender "Photoshop"
output is RLE 16-bit RGBA.

Usage:
  python check_psd_rgb.py <file.psd> <x> <y>            -> prints PIXEL r,g,b
  python check_psd_rgb.py <file.psd> <x> <y> <r> <g> <b> <tol>
      exits 0 when |channel8 - expected| <= tol for r,g,b, else 1.
"""

from __future__ import annotations

import struct
import sys


def unpackbits_row(raw, want_bytes):
    """Decode one PackBits (PSD RLE) scanline to exactly want_bytes bytes."""
    out = bytearray()
    i = 0
    while i < len(raw) and len(out) < want_bytes:
        n = raw[i]
        i += 1
        if n < 128:               # literal run of n+1 bytes
            out += raw[i:i + n + 1]
            i += n + 1
        elif n > 128:             # repeat next byte 257-n times
            out += bytes([raw[i]]) * (257 - n)
            i += 1
        # n == 128: no-op
    return out


def read_rgb(path, x, y):
    d = open(path, "rb").read()
    if d[:4] != b"8BPS":
        raise ValueError("not a PSD (bad signature)")
    channels = struct.unpack(">H", d[12:14])[0]
    height = struct.unpack(">I", d[14:18])[0]
    width = struct.unpack(">I", d[18:22])[0]
    depth = struct.unpack(">H", d[22:24])[0]
    if channels < 3:
        raise ValueError(f"not an RGB PSD (channels={channels})")
    if depth not in (8, 16):
        raise ValueError(f"unsupported depth {depth}")
    if not (0 <= x < width and 0 <= y < height):
        raise ValueError(f"out of range: image {width}x{height}")

    pos = 26
    for _ in range(3):  # color mode, image resources, layer/mask sections
        ln = struct.unpack(">I", d[pos:pos + 4])[0]
        pos += 4 + ln
    comp = struct.unpack(">H", d[pos:pos + 2])[0]
    pos += 2
    bps = depth // 8
    row_bytes = width * bps
    maxval = (1 << depth) - 1

    def plane_value(cidx):
        if comp == 0:  # RAW planar
            chan_size = row_bytes * height
            off = pos + cidx * chan_size + y * row_bytes + x * bps
            return int.from_bytes(d[off:off + bps], "big")
        if comp == 1:  # RLE / PackBits
            nrows = channels * height
            counts = struct.unpack(f">{nrows}H", d[pos:pos + 2 * nrows])
            data_start = pos + 2 * nrows
            target = cidx * height + y
            off = data_start + sum(counts[:target])
            row = unpackbits_row(d[off:off + counts[target]], row_bytes)
            return int.from_bytes(row[x * bps:x * bps + bps], "big")
        raise ValueError(f"unsupported PSD compression {comp}")

    raw = [plane_value(c) for c in (0, 1, 2)]
    as8 = [round(v * 255 / maxval) for v in raw]
    return raw, as8, depth, width, height


def main() -> int:
    if len(sys.argv) not in (4, 8):
        print(__doc__)
        return 2
    path, x, y = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    raw, as8, depth, w, h = read_rgb(path, x, y)
    print(f"PIXEL {path} ({x},{y}) = raw {raw} ({depth}-bit, {w}x{h}; 8-bit {as8})")
    if len(sys.argv) == 8:
        exp = [int(v) for v in sys.argv[4:7]]
        tol = int(sys.argv[7])
        ok = all(abs(a - e) <= tol for a, e in zip(as8, exp))
        print(f"EXPECT rgb8={exp} tol={tol} -> {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
