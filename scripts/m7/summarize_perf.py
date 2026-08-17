#!/usr/bin/env python3
"""Reduce DYNAMICFX_PERF=1 logs to per-scene median/p95 tables (audit 07).

Inputs (in the out dir passed as argv[1]):
  m7bench.log           SCENE <tag> bpc=<n> begin=<epoch> end=<epoch> wall_ms=<n> status=...
  dynamicfx_plugin.log  [<epoch>] perf: depth=.. dims=WxH passes=P iters=I frame=F
                        conv_in=.. upload=.. gpu=.. readback=.. conv_out=.. total=..

Perf lines are bucketed into scenes by epoch window (+/- SLACK seconds);
inside a window the scene signature (depth/dims/passes/iters) is also
checked so boundary bleed cannot mix scenes. Output: summary.md +
summary.csv in the out dir, table printed to stdout.
"""

from __future__ import annotations

import re
import statistics
import sys
from pathlib import Path

SLACK = 2  # seconds of tolerance around scene begin/end epochs

SCENE_RE = re.compile(
    r"^SCENE (\S+) bpc=(\d+) begin=(\d+) end=(\d+) wall_ms=(\d+) status=(.*)$"
)
PERF_RE = re.compile(
    r"^\[(\d+)\] perf: (?:t0=\d+ )?depth=(\w+) dims=(\d+)x(\d+) (?:rect=(\d+)x(\d+) )?passes=(\d+) iters=(\d+) frame=(-?\d+) "
    r"conv_in=([\d.]+) upload=([\d.]+) gpu=([\d.]+) readback=([\d.]+) conv_out=([\d.]+) total=([\d.]+)$"
)
SPANS = ("conv_in", "upload", "gpu", "readback", "conv_out", "total")


def expected_signature(tag: str) -> tuple[str, str, int]:
    depth = "F32" if tag.endswith("_32") else "U8"
    if "4k" in tag:
        dims = "3840x2160"
    elif "1080" in tag:
        dims = "1920x1080"
    else:
        dims = "1280x720"
    passes = 6 if "thermal" in tag else 1
    return (depth, dims, passes)


def p95(values: list[float]) -> float:
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, round(0.95 * (len(ordered) - 1))))
    return ordered[index]


def main() -> int:
    out_dir = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    bench_log = out_dir / "m7bench.log"
    plugin_log = out_dir / "dynamicfx_plugin.log"
    for path in (bench_log, plugin_log):
        if not path.exists():
            print(f"FATAL: missing {path}")
            return 1

    scenes = []
    for line in bench_log.read_text(encoding="utf-8", errors="replace").splitlines():
        match = SCENE_RE.match(line.strip())
        if match:
            scenes.append(
                {
                    "tag": match.group(1),
                    "bpc": int(match.group(2)),
                    "begin": int(match.group(3)),
                    "end": int(match.group(4)),
                    "wall_ms": int(match.group(5)),
                    "status": match.group(6).strip(),
                    "lines": [],
                }
            )
    if not scenes:
        print("FATAL: no SCENE lines in m7bench.log")
        return 1

    total_perf = 0
    unassigned = 0
    for line in plugin_log.read_text(encoding="utf-8", errors="replace").splitlines():
        match = PERF_RE.match(line.strip())
        if not match:
            continue
        total_perf += 1
        ts = int(match.group(1))
        record = {
            "depth": match.group(2),
            "dims": f"{match.group(3)}x{match.group(4)}",
            "rect": f"{match.group(5)}x{match.group(6)}" if match.group(5) else "",
            "passes": int(match.group(7)),
            "iters": int(match.group(8)),
            "frame": int(match.group(9)),
        }
        for i, span in enumerate(SPANS):
            record[span] = float(match.group(10 + i))
        # Signature-first: a line belongs to a scene only if its
        # depth/dims/passes match what the scene tag implies AND its
        # timestamp falls in the scene window (+/- slack). Back-to-back
        # scenes overlap in slackened time; signatures disambiguate, and
        # same-signature scenes are far apart by construction.
        hits = [
            s
            for s in scenes
            if s["begin"] - SLACK <= ts <= s["end"] + SLACK
            and expected_signature(s["tag"])
            == (record["depth"], record["dims"], record["passes"])
        ]
        if len(hits) > 1:
            # temporal vs non-temporal overlap: windowed lines (iters > 1)
            # can only be temporal; an iters == 1 line is temporal only for
            # its single frame-0 render (first such line chronologically).
            if record["iters"] > 1:
                hits = [s for s in hits if "temporal" in s["tag"]] or hits
            else:
                temporal_hits = [s for s in hits if "temporal" in s["tag"]]
                other_hits = [s for s in hits if "temporal" not in s["tag"]]
                needs_frame0 = [
                    s
                    for s in temporal_hits
                    if record["frame"] == 0
                    and not any(l["iters"] == 1 and l["frame"] == 0 for l in s["lines"])
                ]
                hits = needs_frame0 or other_hits or hits
            if len(hits) > 1:
                hits.sort(key=lambda s: abs(ts - (s["begin"] + s["end"]) / 2))
        if hits:
            hits[0]["lines"].append(record)
        else:
            unassigned += 1

    header = (
        "| scene | depth | dims | passes | iters | renders | "
        + " | ".join(f"{s} p50/p95" for s in SPANS)
        + " | wall_ms | fps |"
    )
    sep = "|" + "---|" * (6 + len(SPANS) + 2)
    md = [header, sep]
    csv = [
        "scene,depth,dims,passes,iters,renders,"
        + ",".join(f"{s}_p50,{s}_p95" for s in SPANS)
        + ",wall_ms,fps,status"
    ]
    for scene in scenes:
        lines = scene["lines"]
        if lines:
            depth = lines[0]["depth"]
            dims = lines[0]["dims"]
            passes = lines[0]["passes"]
            iters = f"{min(l['iters'] for l in lines)}-{max(l['iters'] for l in lines)}"
            frames = len({l["frame"] for l in lines})
            stats = {
                span: (
                    statistics.median([l[span] for l in lines]),
                    p95([l[span] for l in lines]),
                )
                for span in SPANS
            }
        else:
            depth = dims = iters = "-"
            passes = 0
            frames = 0
            stats = {span: (0.0, 0.0) for span in SPANS}
        fps = frames * 1000.0 / scene["wall_ms"] if scene["wall_ms"] else 0.0
        cells = " | ".join(f"{stats[s][0]:.2f}/{stats[s][1]:.2f}" for s in SPANS)
        md.append(
            f"| {scene['tag']} | {depth} | {dims} | {passes} | {iters} | {len(lines)} | "
            f"{cells} | {scene['wall_ms']} | {fps:.1f} |"
        )
        flat = ",".join(f"{stats[s][0]:.3f},{stats[s][1]:.3f}" for s in SPANS)
        csv.append(
            f"{scene['tag']},{depth},{dims},{passes},{iters},{len(lines)},{flat},"
            f"{scene['wall_ms']},{fps:.2f},{scene['status']}"
        )

    md.append("")
    md.append(
        f"perf_lines={total_perf} assigned={total_perf - unassigned} unassigned={unassigned}"
    )
    (out_dir / "summary.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    (out_dir / "summary.csv").write_text("\n".join(csv) + "\n", encoding="utf-8")
    print("\n".join(md))
    # Unassigned lines are expected (idle/compile-window preview renders
    # outside any scene window); only scene health gates the result.
    bad = [s["tag"] for s in scenes if s["status"] != "DONE" or not s["lines"]]
    if bad:
        print(f"RESULT=FAIL bad_scenes={bad}")
        return 1
    print("RESULT=PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
