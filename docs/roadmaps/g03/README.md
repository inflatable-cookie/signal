# g03 Milestones

Status: complete
Updated: 2026-03-12

## Why this generation matters now

`g02` completed the first substantial reusable DSP and analysis runway. The
next Signal-owned bottleneck is no longer feature extraction breadth; it is the
depth of the engine substrate that products will rely on for routing, metering,
automation playback, warp/render, plugin chain execution, and runtime
hardening.

This generation stays inside Signal-owned reusable boundaries:

- graph and runtime topology stay in Signal crates rather than product hosts
- engine diagnostics and metering stay reusable rather than app-local
- clip processing, render, and plugin chain semantics become explicit runtime
  substrate instead of later host-specific behavior
- acceptance and soak coverage are treated as engine deliverables, not only app
  integration tests

## Dependency order

1. routed mixer topology first
2. metering and automation execution on that topology second
3. warp and clip-processing substrate after timing/control paths are stable
4. plugin-chain execution and offline render after the core engine contract is
   explicit
5. profiling and soak hardening after the major engine surfaces exist

## Milestone map

- `g03.001` `complete`
  - mixer graph, buses, and routing topology depth
- `g03.002` `complete`
  - runtime metering, loudness, and diagnostics export depth
- `g03.003` `complete`
  - automation engine and high-resolution parameter playback
- `g03.004` `complete`
  - tempo map, stretch, and warp execution substrate
- `g03.005` `complete`
  - clip rendering, fades, and nondestructive processing depth
- `g03.006` `complete`
  - plugin device-chain execution, delay compensation, and state recall
- `g03.007` `complete`
  - offline render, freeze, and stem export pipeline
- `g03.008` `complete`
  - engine profiling, soak harnesses, and runtime fault hardening

## Working rules for this thread

- keep reusable engine behavior in `signal-graph`, `signal-runtime`, shared DSP
  crates, and Signal-owned host/supervisor crates
- avoid app-owned session/workflow semantics in this queue unless they change
  reusable runtime contracts
- prefer typed runtime/export surfaces over ad hoc textual summaries
- keep realtime-safe execution separate from offline-heavy preparation or
  render-only helpers
- prove new engine behavior with focused fixtures, runtime tests, or supervisor
  exports before broadening breadth again

## Next Task

COMPLETE. `g03` closed on 2026-03-12 after `g03.008` finished the profiling,
soak, and fault-hardening acceptance spine. Continue with `g04.001` now that
the next reusable Signal queue is open.
