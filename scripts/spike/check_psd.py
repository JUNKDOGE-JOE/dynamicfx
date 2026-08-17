#!/usr/bin/env python3
"""Minimal PSD alpha-channel probe for the M0 transport spike (TR-M0-007).

Reads one pixel's alpha from a flattened PSD (8- or 16-bit, RAW or RLE/
PackBits compression) with no third-party dependencies. AE's aerender
"Photoshop Sequence" output is RLE 16-bit RGBA.

Usage:
  python check_psd.py <file.psd> <x> <y>              -> prints ALPHA raw + 8-bit
  python check_psd.py <file.psd> <x> <y> <a8> <tol>
      exits 0 when |alpha8 - a8| <= tol, else 1.  (a8/tol in 8-bit terms)
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


def read_alpha(path, x, y):
    d = open(path, "rb").read()
    if d[:4] != b"8BPS":
        raise ValueError("not a PSD (bad signature)")
    channels = struct.unpack(">H", d[12:14])[0]
    height = struct.unpack(">I", d[14:18])[0]
    width = struct.unpack(">I", d[18:22])[0]
    depth = struct.unpack(">H", d[22:24])[0]
    if channels < 4:
        raise ValueError(f"no alpha channel (channels={channels})")
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
    aidx = 3  # alpha is the 4th channel plane

    if comp == 0:  # RAW planar
        chan_size = row_bytes * height
        off = pos + aidx * chan_size + y * row_bytes + x * bps
        raw_val = int.from_bytes(d[off:off + bps], "big")
    elif comp == 1:  # RLE / PackBits
        nrows = channels * height
        counts = struct.unpack(f">{nrows}H", d[pos:pos + 2 * nrows])
        pos += 2 * nrows
        target = aidx * height + y
        off = pos + sum(counts[:target])
        row = unpackbits_row(d[off:off + counts[target]], row_bytes)
        raw_val = int.from_bytes(row[x * bps:x * bps + bps], "big")
    else:
        raise ValueError(f"unsupported PSD compression {comp}")

    maxval = (1 << depth) - 1
    return raw_val, round(raw_val * 255 / maxval), depth, width, height


def main() -> int:
    if len(sys.argv) not in (4, 6):
        print(__doc__)
        return 2
    path, x, y = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    raw_val, a8, depth, w, h = read_alpha(path, x, y)
    print(f"ALPHA {path} ({x},{y}) = {raw_val} ({depth}-bit, {w}x{h}; 8-bit={a8})")
    if len(sys.argv) == 6:
        exp, tol = int(sys.argv[4]), int(sys.argv[5])
        ok = abs(a8 - exp) <= tol
        print(f"EXPECT alpha8={exp} tol={tol} -> {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
