# Final macOS GUI observations

These observations were made by the root agent through CUA during the final
AE 2026 26.3x87 run on 2026-09-08, after the installed binary hash matched
`e239ae457ab33e1bc9466d5ab0bff8b81c08861d072eb5e667b5cd8bc6ae8b0b`.
The evidence curator did not independently operate AE. This record preserves
the root agent's reported observations, distinct from scripted numeric checks.

- In the `DFX_siri` Composition viewer, Full, explicitly selected Half and
  explicitly selected Quarter each displayed the complete glow on all four
  edges. The initial artifact's cropped top-left quadrant was no longer seen.
- The native Details button opened its AE dialog, which reported compiled
  1 pass, 12 parameters and E0. The dialog was successfully dismissed.
- CUA screenshots of these checks remain in the conversation's tool history.
  They are not copied here because surrounding windows included unrelated
  ae-mcp chat content. The PNGs in this directory are derived from rendered
  PSD files and do not stand in for the GUI observations.

The [Details fixture readback](final-details.json) identifies the active shader
and Full resolution, but does not itself prove the dialog's visible text.
The [physical geometry log](dynamicfx-final.log),
[export checks](final-export-checks.json) and retained PSDs independently
support the corrected render dimensions and complete exported frame.
