#!/usr/bin/env python3
"""Validate DynamicFX governance docs without third-party dependencies."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = [
    ROOT / "CLAUDE.md",
    ROOT / "README.md",
    # examples/ ships in the public repo, so its links are a public surface
    # and get the same check as docs/.
    *sorted((ROOT / "examples").rglob("*.md")),
    *sorted((ROOT / "docs").rglob("*.md")),
]
MERMAID_HEADERS = ("flowchart ", "sequenceDiagram", "stateDiagram-v2", "classDiagram")
REQUIRED_PATHS = (
    "CLAUDE.md",
    "docs/IMPLEMENTATION_STATUS.md",
    "docs/ARCHITECTURE.md",
    "docs/ROADMAP.md",
    "docs/TEST_MATRIX.md",
    "docs/adr/README.md",
    "docs/audits/README.md",
)
# M0 + M3 + M4 + M5 + M6 sets (ADR-0009; 0022 and 0025 are in-milestone
# corrections from live evidence), then the post-M7 feature ADRs. Every
# Accepted ADR belongs here — an omission silently drops it from the status
# and index checks below.
ACCEPTED_ADRS = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37)


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def check_markdown(errors: list[str]) -> tuple[int, int]:
    link_count = 0
    mermaid_count = 0
    for path in DOCS:
        text = path.read_text(encoding="utf-8")
        for _, target in re.findall(r"\[([^]]+)\]\(([^)]+)\)", text):
            if target.startswith(("http://", "https://", "mailto:")):
                continue
            clean = target.split("#", 1)[0]
            if not clean:
                continue
            link_count += 1
            if not (path.parent / clean).resolve().exists():
                fail(errors, f"{path.relative_to(ROOT)}: missing link {target}")

        blocks = re.findall(r"```mermaid\s*\n(.*?)\n```", text, re.S)
        mermaid_count += len(blocks)
        for index, block in enumerate(blocks, 1):
            lines = [line.strip() for line in block.splitlines() if line.strip()]
            if not lines or not lines[0].startswith(MERMAID_HEADERS):
                fail(errors, f"{path.relative_to(ROOT)}: invalid Mermaid header in block {index}")
            if block.count('"') % 2:
                fail(errors, f"{path.relative_to(ROOT)}: odd quotes in Mermaid block {index}")
            for opening, closing in (("[", "]"), ("(", ")"), ("{", "}")):
                if block.count(opening) != block.count(closing):
                    fail(
                        errors,
                        f"{path.relative_to(ROOT)}: unbalanced {opening}{closing} in Mermaid block {index}",
                    )
        if len(re.findall(r"^```", text, re.M)) % 2:
            fail(errors, f"{path.relative_to(ROOT)}: unclosed fenced code block")
    return link_count, mermaid_count


def check_required_files(errors: list[str]) -> None:
    for relative in REQUIRED_PATHS:
        if not (ROOT / relative).exists():
            fail(errors, f"missing required handoff path: {relative}")


def check_adrs(errors: list[str]) -> None:
    index = (ROOT / "docs/adr/README.md").read_text(encoding="utf-8")
    for number in ACCEPTED_ADRS:
        prefix = f"{number:04d}"
        matches = list((ROOT / "docs/adr").glob(f"{prefix}-*.md"))
        if len(matches) != 1:
            fail(errors, f"ADR {prefix}: expected one file, found {len(matches)}")
            continue
        if "- Status: Accepted" not in matches[0].read_text(encoding="utf-8"):
            fail(errors, f"ADR {prefix}: initial decision is not Accepted")
        if f"[{prefix}]" not in index:
            fail(errors, f"ADR {prefix}: missing from index")


def check_core_contract(errors: list[str]) -> None:
    architecture = (ROOT / "docs/ARCHITECTURE.md").read_text(encoding="utf-8")
    clauses = (
        "`Source.expression` 是权威输入",
        "LanguageFrontend registry",
        "Multi-pass 是核心能力",
        "AE 2023 Windows",
        "AE 2026 Windows",
        "不能直接用 DefinitionHash 作为 PipelineKey",
    )
    for clause in clauses:
        if clause not in architecture:
            fail(errors, f"architecture is missing approved clause: {clause}")

    status = (ROOT / "docs/IMPLEMENTATION_STATUS.md").read_text(encoding="utf-8")
    # Milestone anchors: update in the same change that moves the milestone.
    status_clauses = (
        "M7 — Performance, SmartRender, and MFR",
        "M7 COMPLETE",
        "Windows AE 2023 | `BLOCKED`",
        "Windows AE 2026 | Full-suite `PASS`",
        "Exact next action",
    )
    for clause in status_clauses:
        if clause not in status:
            fail(errors, f"implementation status is missing: {clause}")


def check_test_truth(errors: list[str]) -> None:
    matrix = (ROOT / "docs/TEST_MATRIX.md").read_text(encoding="utf-8")
    host_start = matrix.find("## Target rewrite — Windows AE host matrix")
    host_end = matrix.find("## Result record template")
    if host_start < 0 or host_end < 0 or host_end <= host_start:
        fail(errors, "test matrix host section is missing or malformed")
        return
    host_section = matrix[host_start:host_end]
    # Since the first live M1 result (2026-08-12), host-matrix PASS cells are
    # allowed only as links to a result record: bare `PASS` (no anchor)
    # remains an unevidenced claim and fails.
    host_bare_pass_rows = [
        line
        for line in host_section.splitlines()
        if line.startswith("|") and "`PASS`" in line and "[`PASS`](#" not in line
    ]
    if host_bare_pass_rows:
        fail(errors, f"target AE host rows claim PASS without a linked record: {host_bare_pass_rows}")

    for line in matrix.splitlines():
        if not line.startswith("| TR-") or "`PASS`" not in line:
            continue
        result_id = line.split("|", 2)[1].strip()
        if f"### {result_id} —" not in matrix:
            fail(errors, f"{result_id}: PASS row has no result record")


def check_diff(errors: list[str]) -> None:
    result = subprocess.run(
        ["git", "diff", "--check"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        fail(errors, f"git diff --check failed:\n{result.stdout}{result.stderr}")

    # The M0 zero-runtime-diff guard is retired as of M1 entry (2026-08-12):
    # M0 was a documentation-only milestone, so any src/ diff meant a session
    # was smuggling implementation past the contract gate. From M1 onward the
    # rewrite itself edits src/, build.rs, and Cargo.toml by design.


def main() -> int:
    errors: list[str] = []
    check_required_files(errors)
    links, diagrams = check_markdown(errors)
    check_adrs(errors)
    check_core_contract(errors)
    check_test_truth(errors)
    check_diff(errors)

    print("DynamicFX governance check")
    print(f"markdown_files={len(DOCS)}")
    print(f"local_links={links}")
    print(f"mermaid_blocks={diagrams}")
    print(f"accepted_adrs={len(ACCEPTED_ADRS)}")
    print(f"errors={len(errors)}")
    for error in errors:
        print(f"ERROR: {error}")
    if errors:
        print("RESULT=FAIL")
        return 1
    print("RESULT=PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
