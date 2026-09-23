# ADR-0051: Native coverage reader component and automatic ownership

- Status: Accepted
- Date: 2026-09-23
- Authority: user-approved automatic internal layer and request to continue
  production integration; engineering choices remain inside that scope.
- Extends: [ADR-0050](0050-host-coverage-resource.md).
- Related: [ADR-0001](0001-expression-authority-and-open-runtime.md),
  [ADR-0014](0014-windows-host-protocol.md).

## Decision

1. Build a dedicated native utility effect, match name `DynamicFx Coverage
   Reader`, from `coverage-reader/`. Distribute `DynamicFxCoverageReader.aex`
   beside `DynamicFx.aex` in the version-specific DynamicFx directory. The
   ordinary shader effect, Source authority and its existing parameters are
   unchanged. This utility never interprets shader source or uses the GPU.
2. Its only parameter after input 0 is a Layer selector, stable ID `Original`,
   default None. The manager binds it to the original layer at ONLY_MASKS.
   SmartFX copies native ARGB pixel words, honoring origins/stride, into output.
   It declares input dependencies, balances checkouts, and reports errors for
   unavailable or incompatible input rather than producing an opaque solid.
3. The main-thread manager creates one locked, shy, video-disabled solid reader
   per original layer, shared by its coverage-declaring DynamicFx instances.
   A source-item comment `DynamicFX coverage reader source; schema=1`, the
   utility effect identity, original Layer ID and validated layer structure
   establish ownership. Display names are not binding identity. No geometry or
   expressions are copied. The first integration uses a comp-sized reader;
   expanded-canvas/large-source acceptance remains a delivery gate.
4. Bind the main effect's hidden Coverage slot to reader ALL_EFFECTS. Publish
   CoverageState ready only after checking both links and owned structure;
   missing utility/input stays E60, missing SDK support stays E59. Unknown,
   modified or externally referenced objects are preserved. Last-user cleanup
   removes only positively owned objects and unused sources. Undo/manual removal
   does not trigger endless recreation during the same opt-in interval.
5. The manager observes opt-in from each instance's own compiled definition,
   after the existing idle observation. Render callbacks never create objects,
   call AEGP, or wait for the manager. The utility can render in an independent
   render process because all its frame input is an ordinary Layer parameter.
6. The component/manager integration does not settle copy/FFX-before-idle
   authorization. Sequence schema remains v1 in this step. Tests must expose
   that pending race; first-frame integration may be installed only for the
   disposable acceptance fixture and must not be shipped or pushed as complete.
   A clone readiness protocol must be recorded and verified before delivery.

## Alternatives

An internal mode in the shader effect would add a second rendering authority
behind Source. Recognizing a special identity shader would couple native pixel
transport to source text and compiler behavior. A small native utility keeps
the exact pixel transport independently testable, at the cost of a second
packaged artifact and explicit missing-component behavior.

## Verification

Byte/stride/origin tests at each native depth, balanced checkout failures,
production first frame against an ordinary-layer reference, automated lifecycle
checks, and the complete ADR-0049/0050 image/FFX/host gates. Build success and a
first frame do not waive the copy readiness, expanded canvas or MFR gates.
