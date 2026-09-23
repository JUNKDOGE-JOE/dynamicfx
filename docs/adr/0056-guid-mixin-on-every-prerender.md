# ADR-0056: Mix a GUID dependency on every SmartPreRender

- Status: Accepted
- Date: 2026-09-23
- Supersedes the no-mix-in branch in [ADR-0055](0055-coverage-host-cache-dependency.md).

AE 26.5 reports `I_MIX_GUID_DEPENDENCIES effect missing call to GuidMixInPtr
during SMART_PRE_RENDER` for the ordinary input-only reference shader. The flag
applies to every callback, not only graphs that need additional state.

Call GuidMixInPtr at SmartPreRender entry, before input checkout or any early
failure. Coverage graphs mix their effective certificate as before. Other
graphs mix the fixed `DFXCVG01` plus zero word. This constant does not introduce
a new changing dependency or coverage checkout. Remove the later conditional
mix-in so every invocation has exactly one call. Advance the effect cache
generation and retest ordinary shaders and coverage, including expanded canvas.

The diagnostic's expansion/budget correction is independent and retained; it
did not cause or solve this host validation failure. The supplied dialog and
both preceding no-frame attempts remain evidence.
