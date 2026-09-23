# ADR-0055: Include coverage authorization in the host frame cache

- Status: Accepted
- The conditional no-mix-in branch is superseded by [ADR-0056](0056-guid-mixin-on-every-prerender.md).
- Date: 2026-09-23
- Extends [ADR-0053](0053-coverage-certificate-revocation.md) and
  [ADR-0054](0054-coverage-headless-restore.md).

Coverage authorization is mutable state outside ordinary AE stream parameters:
revoking a certificate can change shader output to E60/pass-through even while
the copied CoverageState stream still contains its previous value. It must be
part of AE's frame cache dependency, not only a check inside SmartRender.

Declare PF_OutFlag2_I_MIX_GUID_DEPENDENCIES in the PiPL/runtime flags. For a
compiled graph using coverage, SmartPreRender mixes exactly 16 bytes through
the public GuidMixInPtr callback: ASCII `DFXCVG01` followed by the effective
non-revoked certificate as little-endian u64 (0 while unauthorized). The same
state is checked again during render. Graphs without coverage add no mix-in.

This affects only the host frame GUID, not shader/definition/plan/pipeline
identities or persisted bytes. Advance effect cache generations for this new
output behavior. Verify warm-copy behavior without an explicit purge, ordinary
pixel fidelity, and independent rendering. A cached frame that skips native
callbacks is not accepted as evidence of execution of a changed binary.
