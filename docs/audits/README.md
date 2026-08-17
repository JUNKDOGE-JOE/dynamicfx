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
