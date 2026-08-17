#!/usr/bin/env python3
"""Dependency-free deep-PSD probes for TR-M5-001 (ADR-0021).

Reads flattened RGB(A) PSDs at 8/16/32-bit depth (RAW, RLE, ZIP, and
ZIP-with-prediction), and runs numeric checks:

  pixel <psd> <x> <y>
      print raw channel values (record mode, always exits 0)
  frac  <psd> <x> <y> <wx> <wy> <expected> <tol>
      R value at (x,y) as a fraction of the white reference at (wx,wy);
      self-calibrating against the file's actual white encoding
  absf  <psd> <x> <y> <expected> <tol>
      absolute float compare of R at (x,y) (32-bit files)
  stair <psd> <x1> <x2> <y>
      strict inequality R(x1) < R(x2) — precision staircase probe
  alpha <psd> <x> <y> <wx> <wy>
      R/white must sit near 0.5 (premultiplied) or 1.0 (straight);
      prints which semantics were measured

Every check prints one CHECK line ending in PASS/FAIL/RECORD.
"""

from __future__ import annotations

import struct
import sys
import zlib


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


def undelta_reorder_row(row, width, bps):
    """ZIP-with-prediction: per-row byte delta, then byte-planar order."""
    b = bytearray(row)
    for i in range(1, len(b)):
        b[i] = (b[i] + b[i - 1]) & 0xFF
    if bps == 1:
        return bytes(b)
    out = bytearray(len(b))
    for x in range(width):
        for p in range(bps):
            out[x * bps + p] = b[p * width + x]
    return bytes(out)


class Psd:
    def __init__(self, path):
        d = open(path, "rb").read()
        if d[:4] != b"8BPS":
            raise ValueError("not a PSD")
        self.channels = struct.unpack(">H", d[12:14])[0]
        self.height = struct.unpack(">I", d[14:18])[0]
        self.width = struct.unpack(">I", d[18:22])[0]
        self.depth = struct.unpack(">H", d[22:24])[0]
        pos = 26
        for _ in range(3):
            ln = struct.unpack(">I", d[pos:pos + 4])[0]
            pos += 4 + ln
        self.comp = struct.unpack(">H", d[pos:pos + 2])[0]
        pos += 2
        bps = self.depth // 8
        row_bytes = self.width * bps
        chan_size = row_bytes * self.height
        planes = []
        if self.comp == 0:
            for c in range(self.channels):
                planes.append(d[pos + c * chan_size:pos + (c + 1) * chan_size])
        elif self.comp == 1:
            nrows = self.channels * self.height
            counts = struct.unpack(f">{nrows}H", d[pos:pos + 2 * nrows])
            off = pos + 2 * nrows
            rows = []
            for n in counts:
                rows.append(unpackbits(d[off:off + n], row_bytes))
                off += n
            for c in range(self.channels):
                planes.append(b"".join(rows[c * self.height:(c + 1) * self.height]))
        elif self.comp in (2, 3):
            raw = zlib.decompress(d[pos:])
            for c in range(self.channels):
                chan = raw[c * chan_size:(c + 1) * chan_size]
                if self.comp == 3:
                    rows = [
                        undelta_reorder_row(chan[y * row_bytes:(y + 1) * row_bytes], self.width, bps)
                        for y in range(self.height)
                    ]
                    chan = b"".join(rows)
                planes.append(chan)
        else:
            raise ValueError(f"unsupported PSD compression {self.comp}")
        self.planes = planes
        self.bps = bps
        self.row_bytes = row_bytes

    def value(self, chan, x, y):
        if not (0 <= x < self.width and 0 <= y < self.height):
            raise ValueError(f"({x},{y}) outside {self.width}x{self.height}")
        o = y * self.row_bytes + x * self.bps
        raw = self.planes[chan][o:o + self.bps]
        if self.depth == 32:
            return struct.unpack(">f", raw)[0]
        return int.from_bytes(raw, "big")


def main() -> int:
    mode = sys.argv[1]
    psd = Psd(sys.argv[2])
    args = sys.argv[3:]
    tag = f"{mode} {sys.argv[2].split('/')[-1]} depth={psd.depth} comp={psd.comp}"

    if mode == "pixel":
        x, y = int(args[0]), int(args[1])
        vals = [psd.value(c, x, y) for c in range(min(psd.channels, 4))]
        print(f"CHECK {tag} ({x},{y}) = {vals} RECORD")
        return 0

    if mode == "frac":
        x, y, wx, wy = (int(v) for v in args[:4])
        expected, tol = float(args[4]), float(args[5])
        white = psd.value(0, wx, wy)
        if not white:
            print(f"CHECK {tag} white ref at ({wx},{wy}) is zero FAIL")
            return 1
        frac = psd.value(0, x, y) / white
        ok = abs(frac - expected) <= tol
        print(f"CHECK {tag} ({x},{y})/white({white}) = {frac:.6f} expect {expected:.6f} tol {tol} {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1

    if mode == "absf":
        x, y = int(args[0]), int(args[1])
        expected, tol = float(args[2]), float(args[3])
        v = psd.value(0, x, y)
        ok = abs(v - expected) <= tol
        print(f"CHECK {tag} ({x},{y}) = {v} expect {expected} tol {tol} {'PASS' if ok else 'FAIL'}")
        return 0 if ok else 1

    if mode == "stair":
        x1, x2, y = (int(v) for v in args[:3])
        v1, v2 = psd.value(0, x1, y), psd.value(0, x2, y)
        ok = v1 < v2
        print(f"CHECK {tag} R({x1},{y})={v1} < R({x2},{y})={v2} {'PASS' if ok else 'FAIL (quantized)'}")
        return 0 if ok else 1

    if mode == "alpha":
        x, y, wx, wy = (int(v) for v in args[:4])
        white = psd.value(0, wx, wy)
        if not white:
            print(f"CHECK {tag} white ref is zero FAIL")
            return 1
        frac = psd.value(0, x, y) / white
        if abs(frac - 0.5) <= 0.02:
            sem = "premultiplied"
        elif abs(frac - 1.0) <= 0.02:
            sem = "straight"
        else:
            print(f"CHECK {tag} R/white = {frac:.6f} matches neither 0.5 nor 1.0 FAIL")
            return 1
        print(f"CHECK {tag} R/white = {frac:.6f} SEMANTICS {sem} PASS")
        return 0

    print(f"unknown mode {mode}")
    return 2


if __name__ == "__main__":
    sys.exit(main())
