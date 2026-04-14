# Signal Test Task Surface

Status: active
Updated: 2026-04-14

## Purpose

This folder holds the repo-owned acceptance task registry for Signal.

It is not the Rust test source tree. It is the Effigy task layer that groups
those test surfaces into navigable acceptance boundaries and higher-level
lanes.

## Layout

- `effigy.tasks.acceptance.boundaries.toml`
  - local import hub for acceptance boundary groups
- `effigy.tasks.acceptance.boundaries.core.toml`
  - runtime/host core continuity and diagnostics boundaries
- `effigy.tasks.acceptance.boundaries.platform.toml`
  - plugin-format, Linux, macOS, and cross-adapter platform boundaries
- `effigy.tasks.acceptance.boundaries.control-io.toml`
  - device, control, external I/O, clock, and media-service boundaries
- `effigy.tasks.acceptance.boundaries.analysis-graph-dsp.toml`
  - analysis, graph-routing, and DSP boundary families
- `effigy.tasks.acceptance.lanes.toml`
  - local import hub for higher-level acceptance lanes
- `effigy.tasks.acceptance.lanes.integration.toml`
  - integrated and generation closeout lanes
- `effigy.tasks.acceptance.lanes.release.toml`
  - release, packaging, and downstream lanes

## Working Rule

- keep the stable include points under:
  - `tests/effigy.tasks.acceptance.boundaries.toml`
  - `tests/effigy.tasks.acceptance.lanes.toml`
- add new acceptance tasks to the concern file they naturally belong to rather
  than widening the hub files again
- prefer moving live task authority files over rewriting historical closeout
  docs when the current surface changes

## Next Task

If the acceptance surface grows again, keep splitting by concern instead of
adding a new mixed bag file.
