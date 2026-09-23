# Project-open repair: local commit verification

PASS for the source checkpoint authorized on 2026-09-10. Baseline is
`17e12a7` plus the remaining repair on `codex/fix-project-open-lock`, Windows,
Rust/Cargo 1.97.1 MSVC. The commit contains the repair and its records; the
automatic host-shape proposal and user project assets are outside its scope.

## Exact-source checks

[source-equivalence.json](source-equivalence.json) records 33 source hashes
equal to the TR-OPEN-001 manifest. No runtime source bytes were changed in
this commit turn. The retained final AEX was rehashed and still equals
`c91db8c0435ccf90e0fac33f5473825c29bbd959cfeef8cc5fcebe80092b0d0a`.
Prior build/native acceptance applies to those exact inputs/artifact; no
new build, installation, host run, release or push occurred here.

## Fresh CPU tests and retained first result

Recorded log copies normalize line endings and remove terminal blank lines;
the original command captures remain in `scripts/out/project-open-commit-20260910`.

- [Initial cached run](tests-default-stale-cache.log): `cargo test --offline
  --lib` returned 220 tests without compiling. The executable's test list
  omitted the new reentry/live-header tests. This is not current-source
  validation, despite its successful exit code.
- Refreshed only the filesystem modification timestamps of `src/lib.rs`
  and `build.rs`; their content hashes remain unchanged. Cargo recompiled
  DynamicFX against the current source instead of the stale cached product.
- [Default](tests-default.log): `cargo test --offline --lib`, 230 passed,
  zero failures, including the reentry/live-header cases.
- [Editor](tests-editor.log): `cargo test --offline --lib --features editor`,
  230 passed, zero failures.
- [Governance](governance.log): bundled Python runs
  `scripts/check_governance.py`, including links and `git diff --check`.

The two native test results remain the earlier TR-OPEN-001 host evidence;
these CPU runs do not extend platform/host acceptance. Historical failures,
including the first ineffective candidate, remain in the original audit.

Next action: no further repair changes are needed for this checkpoint;
source publication requires a separately scoped public-history transfer.
The two production-project expression references remain deferred.
