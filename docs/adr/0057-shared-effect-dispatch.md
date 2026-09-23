# ADR-0057: Shared-reference effect dispatch

- Status: Accepted
- Date: 2026-09-23
- Extends [ADR-0023](0023-temporal-seek-reset.md) implementation safety.

The after-effects 0.4.0 entry macro constructs mutable references to shared
global and sequence allocations on every callback, including concurrent render
callbacks. An interior mutex cannot make those outer exclusive references sound.
Its Sync check does not change this aliasing rule. This affects the main effect
and the new reader and must be repaired before concurrent MFR acceptance.

Maintain a narrowly adapted, licensed local dispatcher using shared references
to global and sequence allocations. Host lifecycle owns setup/destruction;
parameter metadata is published through OnceLock and an atomic count. Mutating
instance state stays behind the existing mutex. Idle-registration state becomes
atomic. Both effects use the same dispatcher. No persistent format, parameter,
source authority, shader ABI or render identity changes.

Keep sequence allocation/flatten/resetup/checkin behavior equivalent. A panic
returns an actual host error instead of success with unwritten output. Validate
default/editor/no-SDK builds, native pixel cases, save/reopen/FFX and overlapping
MFR callbacks. Opt-in diagnostic spans record frame, process and thread identities
without recording source or user paths; normal rendering does not emit them.
