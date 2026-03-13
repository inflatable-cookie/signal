# 2026-03-13 22:31:42 GMT - Audio precision policy and plugin isolation intake

## Summary

Captured one durable Signal policy note for audio precision and widened `g06`
so flexible plugin isolation policy is explicit work instead of an implied
future adapter detail.

## Changes

- added `docs/policy/001-audio-precision-sample-format-and-file-bit-depth-policy.md`
  to freeze the current and intended split between:
  - internal `f32` audio processing
  - targeted `f64` control and accumulator math
  - 32-bit float runtime-owned artifacts
  - hardware and file-format sample precision as boundary concerns
- widened `g06` generation framing so plugin isolation policy is called out as
  part of post-`g05` feature breadth rather than hidden inside sandbox lore
- retitled and deepened `g06.003` so it now plans:
  - flexible placement and isolation rules
  - allowlist and denylist policy modes
  - by-format isolation policy
  - sandbox grouping and continuity semantics through one runtime-owned surface
- widened `g06.011` so cross-adapter conformance explicitly includes
  format-scoped isolation behavior

## Why This Matters

Signal already assumes untrusted plugin code should default toward out-of-
process isolation, but that vision-level stance was not yet enough for product
teams that need Bitwig or Studio One style selectable policy such as:

- isolate everything except explicit verified allow rules
- allow in-process except explicit deny rules
- isolate by format, vendor, capability, or other reusable filters

Making that a first-class runtime planning surface now is better than letting
each consumer invent its own placement logic once VST3 and AU land.

## Validation

- `effigy qa:docs`
- `git diff --check`

## Next Task

Start `g06.001`, then carry the interruption contract directly into
`g06.003` so plugin isolation policy, sandbox grouping, and recovery semantics
freeze together before VST3 and AU widen the adapter surface.
