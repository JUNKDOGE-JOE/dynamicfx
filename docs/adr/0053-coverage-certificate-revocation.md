# ADR-0053: Revoke copied coverage certificates across render clones

- Status: Accepted
- Date: 2026-09-23
- Supersedes the resetup-only authorization assumption in
  [ADR-0052](0052-coverage-instance-readiness.md) decision 5. Its transport bytes,
  certificate range, supervision and diagnostics are unchanged.

## Evidence

Candidate `9e922d84...` clears the copied UI instance during normal resetup, but
AE 26.5 still constructs a read-only render clone with the original certificate.
The immediate-copy case again differs at 42331 float32 alpha words. Logged
normal/render-only resetups therefore do not prove that AE flattened the copied
UI instance between those two events.

## Decision

1. Revocation is process-wide for each issued certificate, not merely a zero
   inside one Local allocation. Normal UI resetup revokes the restored value
   before clearing it. Binding/source/supervised-state invalidation does likewise.
2. Every render checks the revocation registry in addition to exact state/
   instance-certificate equality. A read-only clone reconstructed directly from
   stale original bytes cannot revive a revoked certificate.
3. The original UI instance's certificate becomes unusable too. On its next
   verified owner pass, the manager issues a fresh value; the copied instance
   receives a separate fresh value after its own links are checked. Only the
   affected certificate lifetimes are revoked; unrelated effects stay valid.
4. Revocations are never evicted while old render clones could exist. The set
   is bounded at 65536 entries; capacity exhaustion or lock failure refuses
   further coverage authorization in that process rather than reviving old
   entries. The registry contains only integers, never host pointers or frames.
5. Flattening reads the effective, non-revoked certificate. A copied/pending
   instance saved before validation writes 0. A separate render process may
   accept a prepared saved certificate through ADR-0052's checksummed envelope,
   but save-before-idle and cold/offline rendering require host evidence.

## Verification

Test that revoking a UI copy also invalidates an already restored render clone
and forces the original to obtain a new certificate. Repeat native immediate
copy, post-idle recovery, FFX application to new/existing instances and saved
render-only transport. Preserve the failed first candidate and its raw frames.
No render-side AEGP or synchronous idle wait is introduced.
