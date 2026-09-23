# Milestone audits

Audits answer: **what actually happened at a visible milestone, with what evidence and residual risk?**

They do not replace:

- [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md), which states current reality;
- [TEST_MATRIX.md](../TEST_MATRIX.md), which is verification truth;
- [ROADMAP.md](../ROADMAP.md), which defines milestone exits;
- [ADR records](../adr/README.md), which preserve decisions.

## Rules

- One canonical audit per milestone.
- Create the audit at milestone start from [TEMPLATE.md](TEMPLATE.md); update it throughout the milestone.
- Lead with outcome, not process narration.
- Preserve failures and incomplete verification.
- Link every PASS/FAIL to the corresponding Test Matrix entry and raw artifact.
- Do not use chat as an artifact location.
- If raw evidence must stay outside the repository, record absolute path, hash, retention reason, host/build, and date.
- A screenshot demonstrates appearance, not numeric pixel correctness or host identity by itself.
- End with exactly one next action that matches `IMPLEMENTATION_STATUS.md`.

## Audit index

| Milestone | Audit | State |
|---|---|---|
| M0 | [00 Architecture Contract](00-architecture-contract.md) | Complete (exited 2026-08-12) |
| M1 | [01 First Frame](01-first-frame.md) | Complete (exited 2026-08-12) |
| M2 | [02 Keyframed Parameters](02-keyframed-params.md) | Complete (exited 2026-08-12) |
| M3 | [03 Persistence and Render Clone](03-persistence-render-clone.md) | Complete (exited 2026-08-12) |
| M4 | [04 Multi-pass Graph](04-multipass-graph.md) | Complete (exited 2026-08-13) |
| M5 | [05 Pixel Formats](05-pixel-formats.md) | Complete (exited 2026-08-13) |
| M6 | [06 Temporal Feedback](06-temporal-feedback.md) | Complete (exited 2026-08-13) |
| M7 | [07 Performance and MFR](07-performance-mfr.md) | Not started (created at M6 exit) |
| Post-M7 host repair | [10 Project-open callback repair](10-project-open-lock.md) | AE 2026 local repair verified; project expression follow-up paused |
| Post-M7 coverage | [11 Host coverage integration](11-host-coverage-integration.md) | Production resource/precision layer implemented; native ownership hookup and host acceptance pending |
| Upstream material | [12 Liquid Glass port](12-upstream-liquid-glass.md) | Complete; licensed source/FFX and native acceptance |
| Unified issue release | [13 Issues #10–#12](13-issue-batch-release.md) | #10/#11 verified; user subsequently deferred #12 |
| Windows 0.2.0 | [14 Windows release](14-windows-020.md) | Completed work released under ADR-0061; #12 remains open |

## Required sections

Every audit uses these sections:

1. Outcome
2. Visible evidence
3. Baseline
4. Code paths
5. Contracts fixed or changed
6. Commands and exact host steps
7. Observed evidence
8. Findings and failures
9. Known limitations
10. Residual risks
11. Decision changes
12. Next exact action
13. Reproduction

An audit is complete only when its milestone exit criteria have matching Test Matrix evidence. Otherwise its state remains in progress or blocked.

| Post-M7 | [08 WGSL and 0.1.0](08-wgsl-010.md) | Native macOS acceptance complete with ADR-0045 Source Undo exception; publication pending |

| Post-M7 | [09 Windows 0.1.1](09-windows-011.md) | Published plugin; IOS27Siri withdrawal supersedes historical Sample distribution |
