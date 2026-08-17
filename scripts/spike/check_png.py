#!/usr/bin/env python3
"""Dependency-free PNG pixel probe for the M0 transport spike (TR-M0-007).

Usage:
  python check_png.py <file.png> <x> <y>                 -> prints PIXEL r,g,b[,a]
  python check_png.py <file.png> <x> <y> <r> <g> <b> <tol>
      exits 0 when |channel - expected| <= tol for r,g,b, else 1.

Supports 8- and 16-bit RGB/RGBA/greyscale, all five PNG scanline filters, no
interlacing (AE writes non-interlaced PNGs; 32-bpc comps yield 16-bit PNGs).
Expected values and tolerance are always in 8-bit terms; 16-bit samples are
normalized to 0-255 before comparison and the raw 16-bit values are printed.
"""

from __future__ import annotations

import struct
import sys
import zlib


def load_png(path: str):
    with open(path, "rb") as fh:
        data = fh.read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG file")
    pos = 8
    width = height = bit_depth = color_type = interlace = None
    idat = bytearray()
    while pos < len(data):
        length, ctype = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + length]
        if ctype == b"IHDR":
            width, height, bit_depth, color_type, _comp, _filt, interlace = struct.unpack(
                ">IIBBBBB", chunk
            )
        elif ctype == b"IDAT":
            idat.extend(chunk)
        elif ctype == b"IEND":
            break
        pos += 12 + length
    if width is None:
        raise ValueError("missing IHDR")
    if bit_depth not in (8, 16):
        raise ValueError(f"unsupported bit depth {bit_depth}")
    if interlace != 0:
        raise ValueError("interlaced PNG unsupported")
    channels = {0: 1, 2: 3, 4: 2, 6: 4}.get(color_type)
    if channels is None:
        raise ValueError(f"unsupported color type {color_type}")

    bps = bit_depth // 8            # bytes per sample
    bpp = channels * bps           # bytes per pixel (filter unit)
    raw = zlib.decompress(bytes(idat))
    stride = width * bpp
    rows = []
    prev = bytearray(stride)
    off = 0
    for _y in range(height):
        filt = raw[off]
        line = bytearray(raw[off + 1:off + 1 + stride])
        off += 1 + stride
        if filt == 1:  # Sub
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 0xFF
        elif filt == 2:  # Up
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif filt == 3:  # Average
            for i in range(stride):
                left = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif filt == 4:  # Paeth
            for i in range(stride):
                left = line[i - bpp] if i >= bpp else 0
                up = prev[i]
                ul = prev[i - bpp] if i >= bpp else 0
                p = left + up - ul
                pa, pb, pc = abs(p - left), abs(p - up), abs(p - ul)
                if pa <= pb and pa <= pc:
                    pred = left
                elif pb <= pc:
                    pred = up
                else:
                    pred = ul
                line[i] = (line[i] + pred) & 0xFF
        elif filt != 0:
            raise ValueError(f"unknown filter {filt}")
        rows.append(line)
        prev = line
    return width, height, channels, bps, rows


def sample(rows, x, y, channels, bps):
    """Return the pixel at (x, y) as a list of ints in native sample range."""
    line = rows[y]
    bpp = channels * bps
    base = x * bpp
    out = []
    for c in range(channels):
        if bps == 2:
            hi = line[base + c * 2]
            lo = line[base + c * 2 + 1]
            out.append((hi << 8) | lo)
        else:
            out.append(line[base + c])
    return out


def main() -> int:
    if len(sys.argv) not in (4, 8):
        print(__doc__)
        return 2
    path, x, y = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    width, height, channels, bps, rows = load_png(path)
    if not (0 <= x < width and 0 <= y < height):
        print(f"OUT_OF_RANGE image={width}x{height}")
        return 2
    px = sample(rows, x, y, channels, bps)
    maxval = 255 if bps == 1 else 65535
    px8 = [round(v * 255 / maxval) for v in px]
    depth = bps * 8
    print(
        f"PIXEL {path} ({x},{y}) = {','.join(str(v) for v in px)} "
        f"(channels={channels}, {depth}-bit; 8-bit={','.join(str(v) for v in px8)})"
    )
    if len(sys.argv) == 8:
        exp = [int(sys.argv[4]), int(sys.argv[5]), int(sys.argv[6])]
        tol = int(sys.argv[7])
        got = list(px8[:3]) if channels >= 3 else [px8[0]] * 3
        ok = all(abs(g - e) <= tol for g, e in zip(got, exp))
        print(f"EXPECT {exp} tol={tol} (8-bit terms) -> {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
