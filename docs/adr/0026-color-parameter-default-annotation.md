# ADR-0026: Color parameter `default:` annotation

- Status: Accepted
- Date: 2026-08-14
- Deciders: user (approved 2026-08-14) + assistant session

## Context

Float pool parameters already honor `// @param name … default:<number>` at slot
configuration. Color parameters (`hint:color`) have no default syntax, so every
color control initializes to AE's white and real recipes must be written back
by script after the fact — measured pain on the thermal benchmark (six color
parameters, recipe scripts in the session scratchpad). The user's expectation:
shader authors declare initial color values in the source itself.

## Decision

Extend the `@param` annotation vocabulary for `hint:color` parameters with:

```
// @param body_main label:"Body Main" hint:color default:#RRGGBB
// @param glow_col  label:"Glow"      hint:color default:#RRGGBBAA
```

- Hex literal, case-insensitive, `#` required; 6 digits imply alpha = 1.0,
  8 digits set alpha explicitly. Channels decode as sRGB-8 → normalized
  0..1 floats into the vec4 default (alpha in word 3), matching how AE color
  params present values to the shader today (no new color management).
- Applied when the slot is configured (same mechanism and timing as float
  defaults). Absent annotation keeps today's behavior.
- Malformed values (wrong length, bad digits, missing `#`) are a compile
  error with the standard annotation diagnostic path — reject, never guess
  (consistent with `@window` reject-not-clamp).
- `default:` on a color param does not interact with `min:`/`max:` (rejected
  if combined with them on a color).

## What this does NOT change

- No persisted field, schema, or parameter-index change: defaults become the
  AE parameter's initial value exactly like float defaults; snapshots and the
  sequence schema are untouched.
- Existing sources without the annotation are byte-identical in meaning.

## Consequences

- Shader authors ship self-contained recipes; the scripted-recipe workaround
  retires for new sources.
- Source text gains a new accepted token, so definitions using it hash
  differently from their annotation-free forms — expected and correct
  (identity follows the committed text).
- The thermal benchmark source may adopt defaults in a follow-up edit; its
  fixtures rebuild per run, so no recorded result is invalidated.

## Verification

- Unit: hex decode exact values (00, 7F/80 midpoints, FF; 6- and 8-digit),
  malformed rejection list, reject `min:`/`max:` combination.
- Host: a source with two color defaults compiles; ECW shows the colors
  without any script write; render uses them (probe via sampleImage).
