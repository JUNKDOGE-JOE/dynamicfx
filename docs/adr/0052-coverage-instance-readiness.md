# ADR-0052: Instance-bound coverage readiness

- Status: Accepted
- Decision 5's resetup-only assumption superseded by [ADR-0053](0053-coverage-certificate-revocation.md).
- Its headless-host classification is extended by [ADR-0054](0054-coverage-headless-restore.md).
- Date: 2026-09-23
- Extends [ADR-0051](0051-native-coverage-reader-component.md).
- Supersedes [ADR-0050](0050-host-coverage-resource.md) decision 6's ready value
  and extends [ADR-0016](0016-sequence-schema-v1.md) with an outer AE transport
  envelope; the inner DFXS v1 definition/source/binding format is unchanged.

## Evidence and decision

The production fixture reproduces immediate-copy misbinding: a duplicated
adjustment layer keeps ready=1 and the original reader. Changing its shape and
rendering in the same JSX call differs from the correct reference at 42331
native float32 alpha words. Post-idle rebinding does not protect that first frame.

1. Each live UI instance owns a revocable coverage certificate, shared with
   the idle manager through an Arc returned by that instance's CompletelyGeneral
   reply. It contains no live host handle. A newly created/copied UI instance
   has no authorization until its owner and both native links are checked.
2. CoverageState retains 0 pending, 2 unsupported, 3 conflict. Legacy 1 is
   unverified. A ready state is an exactly representable integer certificate
   in [4, 2^48-1]. Its valid range grows in place; parameter ID/order are stable.
   The renderer requires equality with this instance's certified value. An
   inherited FFX value alone never grants permission. Certificate issuance is
   independent of shader compile generations; certificates identify validated
   instance/binding lifetimes, not source or cache identity.
3. The manager revokes authorization before changing a binding or publishing
   unavailable/conflict state. After successful link verification it grants a
   fresh certificate if needed and writes that same value to CoverageState.
   UserChangedParam notifications for the hidden coverage input/state revoke
   authorization, except inside the manager's scoped publication. These two
   parameters become supervised. Source changes revoke as well.
4. Coverage-bearing sequences use AE flatten version 2. Its payload is a
   16-byte header: `DFXC` magic, little-endian u64 certificate (0 unverified),
   little-endian CRC32 of the whole payload with the CRC word zeroed, followed
   by the complete existing DFXS v1 snapshot. The 8 MiB overall cap still applies.
   Non-coverage sequences remain byte-identical AE version 1 / DFXS v1.
   Version 1 coverage data is read with no authorization. Unknown outer versions
   or malformed certificates fail closed; no old binding map is discarded to
   signal readiness. This envelope is host transport, not shader authority.
5. Unflatten restores a certificate as provisional. On SequenceResetup only
   PF_InFlag_PROJECT_IS_RENDER_ONLY may promote that persisted certificate for
   the read-only render project. A normal UI resetup revokes it, including a
   duplicate or reopened UI project. The normal manager revalidates and issues
   a new certificate, invalidating the primitive stream for fresh render clones.
   No AEGP call, waiting or mutation is added to render callbacks.
6. Until authorization matches, render reports E60 and passes through. Old-host
   E59 remains distinct. Successful recovery clears only the coverage readiness
   diagnostic. FFX application to new and existing instances, copy-before-idle,
   post-idle recovery, saved render-only clones and real host resetup flags all
   require explicit tests; the flag is not assumed to prove host acceptance.

## Consequences and gates

An initial preview may show E60 until the main-thread owner checks run. It must
never display the previous original's outline as valid coverage. Saving an
unverified instance persists certificate 0, so an offline render cannot silently
promote it. Prepared projects carry the checked certificate for read-only renders.
This supersedes the earlier immediate UI-reopen behavior: UI reopen must first
revalidate; offline rendering is independently verified.

Tests cover certificate mismatch/revocation, normal versus render-only resetup,
old format compatibility, checksummed transport, persistent-map preservation,
supervised publication and native copy/FFX recovery. Other fidelity/lifecycle
requirements remain unchanged, and no push is permitted before the full gate.
