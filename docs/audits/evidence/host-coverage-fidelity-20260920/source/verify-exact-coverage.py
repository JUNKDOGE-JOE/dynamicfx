import argparse
import json
import math
from pathlib import Path
import re
import struct


def symbol(value, scale=1):
    if not math.isfinite(value):
        raise ValueError("Nonfinite path symbol")
    decoded = round(value * scale)
    if abs(value - decoded / scale) > 1 / 32:
        raise ValueError("Path symbol exceeds the measured quantization margin")
    return decoded


def decode(points):
    if len(points) < 2 or (len(points) - 2) % 3:
        raise ValueError("Invalid exact-coverage record length")
    if [symbol(v) for v in points[0]] != [-1234, -5680]:
        raise ValueError("Invalid exact-coverage header")
    width, height = map(symbol, points[1])
    if min(width, height) <= 0 or max(width, height) > 32767 or width * height > 524288:
        raise ValueError("Invalid coverage dimensions")
    bits = [0] * (width * height)
    seen = bytearray(len(bits))
    for i in range(2, len(points), 3):
        x, y = map(symbol, points[i])
        w, h = map(symbol, points[i + 1])
        hi, low_half = points[i + 2]
        hi, low = symbol(hi), symbol(low_half, 2)
        if not (0 <= hi <= 0x3F80 and 0 <= low <= 65535):
            raise ValueError("Invalid float word")
        word = hi * 65536 + low
        if word > 0x3F800000 or min(w, h) <= 0 or x < 0 or y < 0 or x + w > width or y + h > height:
            raise ValueError("Invalid alpha or block bounds")
        for row in range(y, y + h):
            start = row * width + x
            end = start + w
            if any(seen[start:end]):
                raise ValueError("Overlapping coverage rectangles")
            bits[start:end] = [word] * w
            seen[start:end] = bytes([1]) * w
    return width, height, bits


def native_reference(path, width, height):
    text = path.read_text(encoding="utf-8")
    header = re.search(r"reference width=(\d+) height=(\d+) depth=(16|32) region=A_LRect "
                       r"\{ left: (-?\d+), top: (-?\d+), right: (-?\d+), bottom: (-?\d+) \}", text)
    if not header:
        raise ValueError("Missing native reference metadata")
    rw, rh, depth, left, top, right, bottom = map(int, header.groups())
    values = json.loads(re.search(r"reference_alpha(?:16|32)=(\[[^\n]*\])", text)[1])
    if len(values) != rw * rh or (right - left, bottom - top) != (rw, rh):
        raise ValueError("Native world extent mismatch")
    if left < 0 or top < 0 or right > width or bottom > height:
        raise ValueError("Reference lies outside the diagnostic canvas")
    bits = [0] * (width * height)
    for y in range(rh):
        row = values[y * rw:(y + 1) * rw]
        if depth == 16:
            row = [struct.unpack("<I", struct.pack("<f", a / 32768))[0] for a in row]
        at = (top + y) * width + left
        bits[at:at + rw] = row
    return bits, depth


def compare(result_path, reference_path):
    result = json.loads(result_path.read_text(encoding="utf-8"))
    result = result.get("result", result)
    data = json.loads(result["structuredContent"]["content"])
    width, height, actual = decode(data["points"])
    expected, depth = native_reference(reference_path, width, height)
    differences = [abs(struct.unpack("<f", struct.pack("<I", a))[0] -
                       struct.unpack("<f", struct.pack("<I", b))[0]) for a, b in zip(actual, expected)]
    mismatches = sum(a != b for a, b in zip(actual, expected))
    return {"width": width, "height": height, "pixels": len(actual), "reference_depth": depth,
            "float32_bit_mismatches": mismatches, "max_alpha_error": max(differences),
            "status": "PASS" if mismatches == 0 else "FAIL"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("result", type=Path)
    parser.add_argument("reference", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    report = compare(args.result, args.reference)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))
    raise SystemExit(0 if report["status"] == "PASS" else 1)
