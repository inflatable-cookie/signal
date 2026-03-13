# 2026-03-13 22:41:55 GMT - g07 generation seeded from Chorus feature sweep

## Summary

Promoted the previously parked post-`g06` feature-expansion candidate suite
into a full planned `g07` generation so the next Signal thread has a concrete
20-milestone runway ready without another planning pass.

## Changes

- added `docs/roadmaps/g07/README.md`
- added `docs/roadmaps/g07/001-020` milestone files covering:
  - multichannel, sidechain, multi-bus, complex plugin-I/O, and spatial depth
  - LV2 plus deeper Linux plugin and hardware backend breadth
  - external MIDI, richer controller-expression, and control-surface substrate
  - sample-domain time-stretch, marker analysis, transform artifacts, and preview services
  - integrated acceptance depth and generation closeout
- updated:
  - `docs/roadmaps/backlog/post-g06-chorus-feature-expansion-and-g07-candidate-suite.md`
  - `docs/roadmaps/generation-index.md`
  - `docs/roadmaps/README.md`

## Why this was worth doing now

`g06` remains the active queue, but the Chorus sweep had already identified the
next coherent Signal-owned feature front. Converting that into a real `g07`
generation now avoids repeated interim planning pauses when the active thread
reaches the end of `g06`.

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`
- `git diff --check`

## Next Task

Keep `g06` active and continue `g06.001`; when maintainers want the next
generation opened, start `g07.001` by freezing the routing and multichannel
substrate before widening sidechain, spatial, Linux, control, or stretch depth.
