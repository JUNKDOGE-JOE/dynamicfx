# Public source transfer, 2026-09-11

The public branch starts at `18501ac` and applies only the source diff of local
checkpoint `ff89564`, plus the coverage diagnostic and its governance/evidence.
Archive ancestors are not imported. Public main's Siri withdrawal and prior
publication redactions remain intact. This is a feature branch, not a release.

## Verification and limits

The production source patch is the exact nine-file repair from `ff89564`.
Public main intentionally lacks two withdrawn example tests, so its expected
unit-test total is two lower than the historical local run. Coverage remains a
separate diagnostic crate; no production coverage resource or ABI is added.

The first locked test command failed because public main does not track a root
Cargo.lock. The local validated lock was supplied to the isolated checkout and
the command rerun; the lock remains ignored. Raw logs retain this setup failure.
[Test commands and outcomes](checks.json) and [governance](governance.log)
record checks on this public transfer. Previous AE runs are historical evidence
mapped by the source-only repair transfer, not a fresh AE run on this branch.

## Publication boundary

Scanned the outgoing files for credential patterns, third-party internals and
unnecessary local user/artifact identifiers. No credentials or third-party
internals were found; decoded PNG references refer to our own coverage data.
[Visible redaction ledger](redactions.json) records each changed evidence copy's
original/public hash. Frozen manifests retain original hashes; consult the
ledger for those public copies. Workspace and AE installation paths needed to
reproduce the diagnostic are retained. No unrelated user assets are included.

Known failures remain published: retired plain rendering caused modal 5027;
absent-expression handle cleanup caused repeated 23::33; later diagnostic
repairs pass the recorded subset. Partial-alpha acquisition cost, original-mask
coverage, FFX and wider host/render acceptance remain open.

Next action: resolve bounded partial-alpha acquisition and original-mask
coverage before proposing production integration of ADR-0049.

The local Proposed ADR-0047 is published as ADR-0049 because public main already
uses 0047 and 0048. Markdown links are adjusted; raw historical captures retain
the local number. The ledger also records changed evidence Markdown bytes.

The user subsequently authorized the sanitized payload for the feature branch
only. The expanded text and image review is recorded in
[publication scan](publication-scan.json). No main update or merge is authorized.
