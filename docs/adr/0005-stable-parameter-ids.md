# ADR-0005: Stable parameter IDs over a fixed AE pool

- Status: Accepted
- Date: 2026-08-08
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)

## Context

After Effects requires a fixed effect-parameter topology, while shader definitions can reorder, relabel, add, remove, or change parameters. Parameters must remain ordinary AE streams so keyframes, expressions, scripting, and render-time evaluation work normally.

## Decision

Predeclare finite parameter pools by supported AE kind. Every logical shader parameter has a stable `ParamId`. Build an immutable `BindingPlan` by reusing compatible ParamId/slot bindings before allocating free slots. Validate the entire plan and pool capacities before atomically publishing UI metadata.

Keyframed values remain in AE streams and are read for the current render time. Passes reference effect-wide ParamIds; they do not own separate AE controls. Defaults are used until streams are committed.

## Alternatives considered

- Map by parameter order or label: rejected because edits would shift values and keyframes.
- Increase pools without stable IDs: rejected because it postpones rather than solves migration.
- Store all parameters in arbitrary data/custom UI: rejected because normal AE keyframes, expressions, and automation are core requirements.

## Consequences

### Benefits

- Compatible reordering and relabeling preserve values.
- Multiple passes can share one keyframed parameter.
- Overflow and incompatible changes fail atomically instead of creating partial UI.

### Costs and risks

- Pool capacities and ParamId grammar become persistent design choices.
- Rename aliases and type-change behavior require explicit schema rules.
- Hidden unused parameters remain part of the AE topology.

## Revisit conditions

A new parameter storage model requires evidence that fixed pools cannot serve real target effects, while preserving first-class AE keyframe/script behavior.

## Verification obligations

- Reorder/label/alias/type-change matrix.
- Defaults-before-commit and keyframe-at-time tests.
- Capacity overflow leaves the previous/published state unmodified according to state ADR.
- Save/reopen and render clone preserve BindingPlan identity.
