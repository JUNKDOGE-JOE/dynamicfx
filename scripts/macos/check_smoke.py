#!/usr/bin/env python3
"""Numerically verify the native host's analytic fixtures; fails on omissions."""
import argparse
import json
from pathlib import Path


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("directory", type=Path)
    ap.add_argument("--capture", default="capture")
    ap.add_argument("--keyframes", default="keyframes")
    args = ap.parse_args()
    checks = []

    def check(name, ok, observed):
        checks.append({"name": name, "pass": bool(ok), "observed": observed})

    def near(name, actual, expected, tolerance):
        check(name, len(actual) == len(expected) and all(abs(a - e) <= tolerance for a, e in zip(actual, expected)), actual)

    data = json.loads((args.directory / (args.capture + ".json")).read_text())
    check("capture completed", data["ok"], data.get("error"))
    c = data["data"]
    check("six shader fixtures", len(c["states"]) == 6, len(c["states"]))
    for s in c["states"]:
        expected = 2 if s["name"] == "DFX_invalid" else 1
        check(s["name"] + " publication", s["token"] % 4 == expected, s)
    check("all depths measured", [d["depth"] for d in c["depths"]] == [8, 16, 32], c["depths"])
    for d in c["depths"]:
        # sampleImage uses integer AE pixel coordinates; allow half a pixel
        # around the documented shader texel-centre convention.
        near(f"{d['depth']}bpc UV", d["gradient"], [0.5, 0.5, 0.25], 0.006)
        near(f"{d['depth']}bpc multipass parity", d["multi"], d["gradient"], 0.004 if d["depth"] == 8 else 0.00005)
        near(f"{d['depth']}bpc HDR", d["hdr"], [2, -0.5, 0.123456] if d["depth"] == 32 else [1, 0, 0.123456], 0.004 if d["depth"] == 8 else 0.00005)
    near("temporal random order", [r["v"] for r in c["temporal"]], [0.1, 0.4, 0.2, 0.4], 0.0001)
    near("invalid source passthrough", c["invalid"], [0.05, 0.4, 0.1], 0.006)
    k = json.loads((args.directory / (args.keyframes + ".json")).read_text())["data"]
    near("keyframed parameter", [k["t0"], k["t1"]], [0.25, 0.75], 0.0001)
    check("two actual keys", k["keys"] == 2, k)
    result = {"pass": all(c["pass"] for c in checks), "checks": checks}
    print(json.dumps(result, indent=2))
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
