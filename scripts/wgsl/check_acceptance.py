#!/usr/bin/env python3
"""Check recorded WGSL acceptance JSON/PSD; never connects to After Effects.

Only supplied records are certified. Missing host/UI obligations are not made
PASS by this checker. Standard library plus the existing deep-PSD reader.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/m5"))
from check_deep import Psd

LANGUAGES = ("glsl", "wgsl")
KINDS = ("uv", "multi", "hdr", "gain", "temporal", "invalid", "layer", "gradient", "path")


class Checker:
    def __init__(self):
        self.checks = []
        self.files = []
        self.scopes = []

    def check(self, name, passed, observed=None):
        self.checks.append({"name": name, "passed": bool(passed), "observed": observed})

    def near(self, name, actual, expected, tolerance):
        good = len(actual) == len(expected) and all(isinstance(a, (int, float)) and math.isfinite(a) and abs(a - e) <= tolerance for a, e in zip(actual, expected))
        self.check(name, good, {"actual": actual, "expected": expected, "tolerance": tolerance})

    def states(self, states):
        expected = {"DFX_010_" + lang + "_" + kind for lang in LANGUAGES for kind in KINDS}
        self.check("exact 18 fixture states", {s["name"] for s in states} == expected and len(states) == 18)
        for s in states:
            lang = 2 if "_wgsl_" in s["name"] else 1
            invalid = s["name"].endswith("_invalid")
            self.check(s["name"] + " language", s["language"] == lang, s["language"])
            self.check(s["name"] + " publication", s["token"] % 4 == (2 if invalid else 1), s["token"])
            if invalid:
                self.check(s["name"] + " exact diagnostic", s["code"] == (21 if lang == 2 else 17), s["code"])

    def capture(self, data):
        self.states(data["states"])
        self.check("all three depths", [d["depth"] for d in data["depths"]] == [8, 16, 32])
        res = data["resolutionSetting"]
        self.check("resolution setting recorded", res in ([1, 1], [2, 2], [4, 4]), res)
        self.scopes.append("sampleImage at requested resolution " + str(res) + "; this does not prove physical viewport downsampling")
        for row in data["depths"]:
            depth = row["depth"]
            tolerance = 1 / 255 + 1e-6 if depth == 8 else 2 / 32768 if depth == 16 else .00005
            for kind in ("uv", "multi", "hdr"):
                pair = row["pairs"][kind]
                self.check(f"{depth} {kind} three sample points", all(len(pair[l]) == 3 for l in LANGUAGES))
                for i, point in enumerate(data["points"]):
                    self.near(f"{depth} {kind} GLSL/WGSL point {i}", pair["wgsl"][i], pair["glsl"][i], tolerance)
                    expected = ([2, -.5, .123456, 1] if depth == 32 else [1, 0, .123456, 1]) if kind == "hdr" else [point[0] / 321, point[1] / 239, .625, 1]
                    # sampleImage's coordinate/radius semantics can integrate
                    # neighboring texels. Exact physical pixels are tested in PSD.
                    analytic_tolerance = tolerance if kind == "hdr" else .008
                    for lang in LANGUAGES:
                        self.near(f"{depth} {kind} {lang} analytic {i}", pair[lang][i], expected, analytic_tolerance)
            for lang in LANGUAGES:
                for i in range(3):
                    self.near(f"{depth} {lang} true multipass parity {i}", row["pairs"]["multi"][lang][i], row["pairs"]["uv"][lang][i], tolerance)
        for lang in LANGUAGES:
            rows = data["temporal"][lang]
            self.check(lang + " random temporal order", [r["t"] for r in rows] == [0, .12, .04, .8])
            for r, expected in zip(rows, [.1, .4, .2, .4]):
                self.near(lang + " temporal " + str(r["t"]), r["rgba"], [expected] * 3 + [1], .0001)
            self.near(lang + " invalid passthrough", data["invalid"][lang], [.05, .4, .1, 1], .006)

    def resources(self, data):
        self.check("resources measured at Full/32-bpc", data["depth"] == 32 and data["resolutionSetting"] == [1, 1])
        assigned = data["phase"] == "assigned"
        self.check("resource phase identified", data["phase"] in ("none", "assigned"), data["phase"])
        for lang in LANGUAGES:
            p = data["pairs"][lang]
            layer = ([20 / 255, 180 / 255, 220 / 255] if assigned else [200 / 255, 40 / 255, 40 / 255]) + [1]
            path = [.25, .25, 5 / 16, 1] if assigned else [0, 0, 1 / 16, 1]
            self.near(lang + " layer " + data["phase"], p["layer"], layer, .006)
            self.near(lang + " path " + data["phase"], p["path"], path, .0001)
            self.check(lang + " selector phase", (p["layerSelector"] > 0 and p["pathSelector"] == 1) if assigned else p["layerSelector"] == p["pathSelector"] == 0)
            self.check(lang + " two gradient stops", p["gradientStops"] == 2)
            for i, point in enumerate(data["gradientPoints"]):
                u = point[0] / 160
                self.near(lang + " gradient " + data["phase"] + " " + str(i), p["gradient"][i], [1 - u, 0, u, 1] if assigned else [u, u, u, 1], .008)
        for kind in ("layer", "path"):
            self.near(kind + " language parity", data["pairs"]["wgsl"][kind], data["pairs"]["glsl"][kind], .0001)
        for i in range(3):
            self.near("gradient language parity " + str(i), data["pairs"]["wgsl"]["gradient"][i], data["pairs"]["glsl"]["gradient"][i], .0001)

    def keyframes(self, data):
        for lang in LANGUAGES:
            p = data["pairs"][lang]
            self.check(lang + " exactly two keyframes", p["keys"] == 2, p["keys"])
            self.near(lang + " key stream values", [p["value0"], p["value1"]], [.25, .75], .0001)
            self.near(lang + " keyframe zero rendered", p["t0"], [.25, 0, 0, 1], .0001)
            self.near(lang + " keyframe one rendered", p["t1"], [.75, 0, 0, 1], .0001)

    def record(self, path):
        raw = path.read_bytes()
        self.files.append({"file": path.name, "sha256": hashlib.sha256(raw).hexdigest()})
        wrapper = json.loads(raw)
        self.check(path.name + " execution completed", wrapper.get("ok") is True, wrapper.get("error"))
        if not wrapper.get("ok"):
            return
        data = wrapper["data"]; mode = data["mode"]
        self.scopes.append(mode + ("/" + data["phase"] if "phase" in data else ""))
        if mode == "capture":
            self.capture(data)
        elif mode == "resources":
            self.resources(data)
        elif mode == "keyframes":
            self.keyframes(data)
        elif mode in ("state", "save", "reopen"):
            self.states(data["states"])
        elif mode == "setup":
            self.check("all new instances default to GLSL", len(data["defaults"]) == 18 and all(s["value"] == 1 for s in data["defaults"]))
            # Initial setup returns before idle publication. Do not require or
            # infer Active tokens from this initial result.
        elif mode == "resources-assign":
            self.check("both resource assignments recorded", len(data["values"]) == 2 and all(v["layer"] > 0 and v["path"] == 1 and v["stops"] == 2 for v in data["values"]))
        elif mode == "queue":
            self.check("twelve queue items at U8", data["depth"] == 8 and len(data["items"]) == 12)
        else:
            self.check("supported recorded mode", False, mode)

    def exports(self, queue_path, directory):
        queue = json.loads(queue_path.read_text())
        self.check("queue creation succeeded", queue.get("ok") is True)
        data = queue["data"]
        self.check("queue contains all paired resolutions", {(x["language"], x["kind"], x["resolution"]) for x in data["items"]} == {(l, k, r) for l in LANGUAGES for k in ("uv", "multi") for r in ("Full", "Half", "Quarter")})
        decoded = {}
        for item in data["items"]:
            expected = item["physical"]
            for frame in (0, 1):
                path = directory / Path(item["output"]).name.replace("[#####]", f"{frame:05d}")
                self.check(path.name + " exists", path.is_file())
                if not path.is_file():
                    continue
                self.files.append({"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
                p = Psd(path)
                valid = [p.width, p.height] == expected and p.depth == 8 and p.channels == 4
                self.check(path.name + " exact format", valid, {"size": [p.width, p.height], "depth": p.depth, "channels": p.channels})
                if not valid:
                    continue
                self.check(path.name + " fully opaque", all(a == 255 for a in p.planes[3]))
                max_error = 0
                for y in range(p.height):
                    for x in range(p.width):
                        i = y * p.width + x
                        want = [(x + .5) / p.width, (y + .5) / p.height, .625]
                        max_error = max(max_error, *(abs(p.planes[ch][i] / 255 - want[ch]) for ch in range(3)))
                self.check(path.name + " every-pixel analytic UV", max_error <= 1 / 255 + 1e-6, {"max_error": max_error, "tolerance": 1 / 255 + 1e-6})
                decoded[(item["language"], item["kind"], item["resolution"], frame)] = p
        for kind in ("uv", "multi"):
            for resolution in ("Full", "Half", "Quarter"):
                for frame in (0, 1):
                    g = decoded.get(("glsl", kind, resolution, frame)); w = decoded.get(("wgsl", kind, resolution, frame))
                    if g is None or w is None:
                        continue
                    error = max(abs(a - b) for ca, cb in zip(g.planes, w.planes) for a, b in zip(ca, cb))
                    self.check(f"{kind} {resolution} frame {frame} GLSL/WGSL pixels", error <= 1, {"max_u8_error": error})
        self.scopes.append("24 physical PSD frames at Full/Half/Quarter, 8-bpc; independent process identity comes from its aerender command/log")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("records", nargs="*", type=Path)
    ap.add_argument("--queue", type=Path)
    ap.add_argument("--exports", type=Path)
    args = ap.parse_args()
    if not args.records and not args.queue:
        ap.error("supply JSON records and/or --queue with --exports")
    if bool(args.queue) != bool(args.exports):
        ap.error("--queue and --exports must be supplied together")
    checker = Checker()
    for path in args.records:
        try:
            checker.record(path)
        except (OSError, ValueError, KeyError, TypeError, IndexError) as e:
            checker.check(path.name + " readable complete record", False, str(e))
    if args.queue:
        try:
            checker.exports(args.queue, args.exports)
        except (OSError, ValueError, KeyError, TypeError, IndexError) as e:
            checker.check("export records complete", False, str(e))
    result = {"passed": bool(checker.checks) and all(c["passed"] for c in checker.checks), "scopes": checker.scopes,
              "checks": checker.checks, "files": checker.files,
              "limits": "Only supplied records are checked. This does not certify installation, actual UI Undo/Redo, physical viewport geometry, other hosts, signing or publication."}
    print(json.dumps(result, indent=2))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
