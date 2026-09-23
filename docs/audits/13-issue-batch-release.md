# Issues #10–#12 and unified release

## Initial outcome

Issues #10 and #11 are implemented and verified on Windows AE 26.5x89.
Issue #9 retains its accepted coverage/material implementation. Issue #12 is
**BLOCKED** on a reproducible affected-host case: the user reports it on another
machine and has no complete error or reproduction available. No fix is claimed
for #12. The requested unified merge and release have not happened.

## Baseline and changes

Baseline `112a6c7`; candidate main SHA-256
`4fe13ec0e9bb7962b4f28ce7020117502c7eeacc2f70d0740a64f6defc7ad80a`.
Reader remains `29695b72fc8a26222a51319c3e42f86696b6d15e78efb02d968bc264ff173be6`.

The annotation parser retains RGB arity; reflection supplies implicit alpha
only to vec4. Non-ASCII malformed hex is rejected without slicing invalid UTF-8.
Percent is float presentation metadata, propagated to the existing slot's
PF display flag. No binding/schema change. [ADR-0060](../adr/0060-color-default-arity-and-percent-display.md).

## Verification and failures

- SDK default/editor and no-SDK library suites each pass 257 tests.
- Before installation, both GLSL/WGSL RGB-hex vec3 and percent cases reproduce
  E19. After installation, all 24 cases across 8/16/32 bpc match their numeric
  reference. Six/eight-digit vec4 defaults retain opaque/explicit alpha.
- Percent display is read back through native `Property.unitsText`; adding and
  removing the hint changes the unit after a normal supervised UI callback.
  Source-only scripting initially leaves prior UI metadata until that callback,
  like the existing label/range path. The initial stale readback is retained;
  `Source.setValue(0)` exercises the callback without changing shader text.
- Keys 25 and 75 remain after removing/restoring the hint and save/reopen;
  rendered samples at 0/0.5/1 seconds are exactly 0.25/0.5/0.75.
- 27 formal Liquid Glass frame pairs are pixel-identical to the accepted
  preceding material: three project depths, three scales, three animation times.
  Coverage code and the reader are unchanged; this is an explicit equivalence
  argument, not a repeat of every earlier native stress test.
- Four opens of an available historical project copy show no missing-source
  error, including the historical Undo-group wrapper. That file has changed
  since the original report; this cannot establish #12 reproduction or repair.
- Selecting a test viewer changed foreground ownership during one check; it
  was reported to the user. Subsequent checks omit viewer activation and mouse
  operations. Do not describe this entire run as foreground-neutral.

The candidate is installed only in AE 2026's dedicated plug-in directory; the
preceding AEX is backed up locally. User work was saved to a separate checkpoint
before disposable tests. No private production AEP or source is published.

## Reproduction and next action

[Evidence](evidence/issues-10-12-20260923/README.md) includes host readbacks,
numeric comparisons, commands and identities. Run library tests with the pinned
toolchain; use the recorded disposable host harness for the native matrix.
The [read-only collector](../project-reopen-diagnostics.md) records host versions
and component hashes on the affected machine without reading projects or logs.

The initial next action was to obtain a reproducible affected-host case for #12
before release. The user subsequently explicitly deferred #12 and authorized
release of completed work. [ADR-0061](../adr/0061-020-windows-release.md) and the
[0.2.0 audit](14-windows-020.md) supersede that release gate. #12 remains open.

Next development action: obtain an affected-host #12 reproduction.
