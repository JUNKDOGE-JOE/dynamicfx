import argparse
from array import array
import gzip
import json
from pathlib import Path
import re


def alpha(path, index, depth):
    text = gzip.decompress(path.read_bytes()).decode("utf-8").replace("\r\n", "\n")
    pattern = (rf"SMART_ALPHA id={index}\nworld=(\d+)x(\d+) depth=(8|16|32) "
               r"origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])")
    matches = list(re.finditer(pattern, text))
    if len(matches) != 1:
        raise ValueError(f"Expected one checkout in {path.name}, got {len(matches)}")
    match = matches[0]
    width, height, actual_depth, left, top = map(int, match.groups()[:5])
    words = json.loads(match[6])
    if actual_depth != depth or min(width, height) <= 0 or len(words) != width * height:
        raise ValueError("Invalid native world metadata")
    limit = {8: 255, 16: 32768, 32: 0xFFFFFFFF}[depth]
    if any(type(word) is not int or not 0 <= word <= limit for word in words):
        raise ValueError("Invalid alpha word")
    canvas = array("I", [0]) * 480000
    x0, y0, x1, y1 = max(left, 0), max(top, 0), min(left + width, 800), min(top + height, 600)
    if x0 < x1 and y0 < y1:
        for y in range(y0, y1):
            start = (y - top) * width + x0 - left
            canvas[y * 800 + x0:y * 800 + x1] = array("I", words[start:start + x1 - x0])
    return canvas


def verify(root):
    raw = root / "raw/native-reader-20260920"
    rows = json.loads((raw / "frame-summary.json").read_text(encoding="utf-8"))
    checked = []
    references = {}
    for row in rows:
        key = f'{row["case"]}-{row["depth"]}-{str(row["requested"]).replace(".", "_")}'
        actual = alpha(raw / f"{key}-actual.native.log.gz", 2, row["depth"])
        expected = alpha(raw / f"{key}-reference.native.log.gz", 0, row["depth"])
        if actual != expected or not any(expected):
            raise ValueError(f"Invalid/empty or unequal coverage: {key}")
        references[(row["case"], row["depth"], row["requested"])] = expected
        checked.append(key)
    previous = "animated"
    deltas = {}
    for case in ("repeater3", "merge-hole-repeater3", "stroke-merge-repeater3"):
        current = references[(case, 32, 0.5)]
        baseline = references[(previous, 32, 0.5)]
        changes = sum(a != b for a, b in zip(current, baseline))
        if not changes:
            raise ValueError(f"Modifier did not exercise a pixel change: {case}")
        deltas[case] = changes
        previous = case
    life = json.loads((raw / "lifecycle-summary.json").read_text(encoding="utf-8"))
    if len(life) != 10 or any(row["status"] != "PASS" for row in life):
        raise ValueError("Lifecycle result changed")
    orphan = json.loads((raw / "orphan-summary.json").read_text(encoding="utf-8"))
    if len(orphan) != 3 or any(row["status"] != "PASS" for row in orphan):
        raise ValueError("Owner deletion/Undo result changed")
    key = "v028-after-owner-undo-32-0_5"
    if alpha(raw / f"{key}-actual.native.log.gz", 2, 32) != alpha(raw / f"{key}-reference.native.log.gz", 0, 32):
        raise ValueError("Post-Undo frame changed")
    return {"status": "PASS", "native_frame_comparisons": len(checked),
            "identity_repeater_controls": sum(row["case"] == "repeater" for row in rows),
            "pixels_each": 480000, "lifecycle_cases": len(life), "owner_deletion_cases": len(orphan),
            "post_undo_frame_comparisons": 1,
            "modifier_pixel_changes": deltas, "checked": checked}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.evidence), indent=2))
