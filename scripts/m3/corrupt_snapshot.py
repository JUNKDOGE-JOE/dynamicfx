#!/usr/bin/env python3
"""Make m3_corrupt.aep: copy m3.aep and flip one byte inside the DFXS
snapshot payload so the CRC check must reject it (TR-M3-001 corruption leg).

The PF flattened data may be embedded raw or hex-encoded inside the .aep;
this probes for both. Exits 0 on success, 1 if no snapshot signature is
found (the harness records a SKIP).

Usage: python corrupt_snapshot.py <out_dir>
"""

import sys
import pathlib


def main() -> int:
    out = pathlib.Path(sys.argv[1])
    src = out / "m3.aep"
    dst = out / "m3_corrupt.aep"
    data = bytearray(src.read_bytes())

    raw = data.find(b"DFXS")
    if raw != -1:
        # Flip a byte 24 bytes past the magic (inside the body, past the
        # header) so length parsing still runs and the CRC catches it.
        target = raw + 24
        data[target] ^= 0x40
        dst.write_bytes(data)
        print(f"corrupted raw DFXS at offset {raw} (flipped byte {target})")
        return 0

    hexed = data.find(b"44465853")  # "DFXS" as ASCII-hex
    if hexed != -1:
        # Flip within a hex digit pair 48 chars (24 bytes) past the magic.
        target = hexed + 48
        old = data[target]
        data[target] = ord("0") if old != ord("0") else ord("1")
        dst.write_bytes(data)
        print(f"corrupted hex DFXS at offset {hexed} (rewrote char {target})")
        return 0

    print("no DFXS signature found in m3.aep")
    return 1


if __name__ == "__main__":
    sys.exit(main())
