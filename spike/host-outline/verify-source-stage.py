import argparse
from array import array
import gzip
import json
from pathlib import Path
import re


def frames(path, depth):
    raw = gzip.decompress(path.read_bytes()).decode("utf-8").replace("\r\n", "\n")
    pattern = (r"SMART_ALPHA id=(\d+)\nworld=(\d+)x(\d+) depth=(\d+) "
               r"origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])")
    result = {}
    for match in re.finditer(pattern, raw):
        index, width, height, actual_depth, left, top = map(int, match.groups()[:6])
        words = json.loads(match[7])
        if (actual_depth != depth or min(width, height) <= 0 or
                min(left, top) < 0 or left + width > 800 or top + height > 600 or
                len(words) != width * height or index in result):
            raise ValueError(f"Invalid or repeated native frame in {path.name}")
        limit = {8: 255, 16: 32768, 32: 0xFFFFFFFF}[depth]
        if any(type(word) is not int or not 0 <= word <= limit for word in words):
            raise ValueError("Invalid native alpha word")
        canvas = array("I", [0]) * 480000
        for y in range(height):
            start = (top + y) * 800 + left
            canvas[start:start + width] = array("I", words[y * width:(y + 1) * width])
        result[index] = canvas
    if set(result) != {0, 1, 2}:
        raise ValueError(f"Missing checkout in {path.name}")
    return result


def verify(directory):
    raw = directory / "raw"
    reports = []
    for depth in (8, 16, 32):
        for stage in ("source", "masks"):
            expected = frames(raw / f"matrix-{depth}-{stage}-reference.native.log.gz", depth)[0]
            actual = frames(raw / f"matrix-{depth}-{stage}-external.native.log.gz", depth)
            for index in (1, 2):
                mismatches = sum(a != b for a, b in zip(expected, actual[index]))
                if mismatches:
                    raise ValueError(f"Cross-layer mismatch: {depth}, {stage}, {index}: {mismatches}")
                reports.append({"route": "external", "depth": depth, "stage": stage,
                                "checkout": index, "native_word_mismatches": mismatches})
        expected = frames(raw / f"matrix-{depth}-masks-reference.native.log.gz", depth)[0]
        actual = frames(raw / f"bridge-{depth}-masks.native.log.gz", depth)
        failed_self = frames(raw / f"matrix-{depth}-self-masks.native.log.gz", depth)
        for index in (1, 2):
            mismatches = sum(a != b for a, b in zip(expected, actual[index]))
            if mismatches:
                raise ValueError(f"Bridge mismatch: {depth}, {index}: {mismatches}")
            if failed_self[index] != failed_self[0] or failed_self[index] == expected:
                raise ValueError("Recorded self-checkout failure changed")
            reports.append({"route": "bridge", "depth": depth, "stage": "masks",
                            "checkout": index, "native_word_mismatches": mismatches})
    return {"status": "PASS", "comparisons": reports, "pixels_per_comparison": 480000,
            "self_checkout_failure_reproduced": True}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.evidence), indent=2))
