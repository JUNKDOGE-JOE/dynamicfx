# ADR-0058: Frame-owned checkout lifetimes and cancellation

- Status: Accepted
- Date: 2026-09-23
- Extends [ADR-0030](0030-layer-input-parameters.md) and coverage integration.

During the final cancellation audit, external-layer checkout errors were still
converted to missing input. A cancelled coverage checkout could therefore render
E60 pass-through as a successful frame. Also, accepted external checkout IDs
were transported in thread-local storage instead of the frame's PreRender data.
The already verified cold MFR result does not make thread-local transport valid
for every callback scheduling/cache path.

Store the accepted external checkout ID mask in the frame's SmartGeom PreRender
data. There are at most four pass inputs and bounded resource pools; validate
mask shifts. No persistent data or shader ABI changes. Advance the effect cache
generation because cached PreRender data has changed.

SmartRender tracks only successful pixel checkouts, including successful empty
inputs. A scoped guard checks each of these IDs in exactly once on success,
error or unwind. It also clears per-frame staging on every exit. Propagate host
InterruptCancel from base, extended and external checkout paths, so AE discards
the frame. Ordinary unavailable external input keeps its existing explicit
diagnostic behavior; it never converts cancellation into a successful frame.

Verify cleanup fault paths, real interrupted previews with cache/purged-frame
comparisons, all-depth pixel cases and genuine concurrent MFR before delivery.
