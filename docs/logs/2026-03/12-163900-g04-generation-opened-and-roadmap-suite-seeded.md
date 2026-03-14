# g04 Generation Opened And Roadmap Suite Seeded

Date: 2026-03-12
Scope: `docs/roadmaps/g04/`

## Summary

Opened the next independent Signal generation after `g03` closed and seeded a
new reusable-runtime roadmap suite directly in Signal docs rather than in
Loophole/Chorus planning surfaces.

## What changed

- marked `g04` as the active Signal roadmap generation in
  `docs/roadmaps/generation-index.md`
- updated `docs/roadmaps/README.md` and `docs/roadmaps/g03/README.md` so the
  local queue points into `g04`
- added `docs/roadmaps/g04/README.md` with the new generation rationale,
  dependency order, and milestone map
- added the first `g04` milestone suite:
  - `g04.001` crate maturity, public contracts, and schema-freeze baseline
  - `g04.002` multicore graph scheduling and anticipative execution depth
  - `g04.003` runtime work orchestration and deferred service policy
  - `g04.004` hardware backend portability and clock-domain boundary depth
  - `g04.005` plugin backend breadth and host-neutral delegation contracts
  - `g04.006` consumer conformance, export stability, and release packaging

## Why this queue

`g03` finished the engine-depth runway. The next Signal-owned bottleneck is not
another narrow engine proof; it is making Signal a stronger shared project with
clear public boundaries, deeper multicore execution policy, explicit deferred
service orchestration, broader portability, and consumer-facing release
confidence.

## Validation

- `effigy health`
- `git diff --check`

## Next

Start with `g04.001` and freeze the first explicit public crate/runtime/export
boundary before widening scheduling or portability work.
