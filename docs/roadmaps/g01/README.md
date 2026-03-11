# g01 Milestones

Status: active
Updated: 2026-03-11

## Why this generation matters now

`g01` is the reset-era foundation generation for Signal. Its job is to turn the
new Rust workspace from named package shells into a credible shared DSP and
engine runtime that Loophole can embed while other apps can also reuse it.

The ordering in this generation is dependency-first rather than simplicity-
first:

- docs and package boundaries first
- trust-edge ownership boundaries second
- core DSP and graph/runtime semantics before device or plugin depth
- host I/O and plugin execution only after the engine substrate is stable enough
  to avoid repeated boundary churn
- node-oriented mixer foundations early, so console nodes, track lanes, buses,
  and routing intent are part of the engine contract before hosts or products
  build too much workflow on flatter assumptions

## Milestone map

- `g01.001` `active`
  - docs foundation and DSP research migration
- `g01.002` `complete`
  - package map and runtime entrypoint naming
- `g01.003` `complete`
  - Rust workspace shell bootstrap
- `g01.004` `active`
  - trust-edge package shell expansion
- `g01.005` `active`
  - core DSP kernel and control-signal baseline
- `g01.006` `active`
  - executable graph routing, latency, and parameter application baseline
- `g01.007` `queued`
  - runtime transport, scheduler, and engine processing baseline
- `g01.008` `queued`
  - device-backed host audio I/O and diagnostics baseline
- `g01.009` `queued`
  - plugin hosting, sandbox processing, and graph-node baseline

## Current sequencing rule

`g01.006` is now the active engine-substrate milestone for this thread. The
next four milestones remain sequenced and detailed enough that a dedicated
Signal thread can keep moving without inventing its own order of operations.

The dependency spine is:

1. `g01.005`
   - establish reusable realtime-safe DSP kernels and control primitives
2. `g01.006`
   - route those kernels through a real graph execution contract with routing,
     latency, parameter timing semantics, and the first node-oriented topology
     assumptions that later console-node and track-lane work can build on
3. `g01.007`
   - make runtime transport, scheduler ownership, and engine block processing
     enforce those graph semantics without collapsing node-oriented topology
     into host-local policy
4. `g01.008`
   - attach the engine path to real host/device execution and diagnostics
     while still exercising credible node/lane/routing shapes
5. `g01.009`
   - integrate plugin and sandbox execution into the same runtime/graph seam
     so plugin-backed nodes fit the same console/lane-oriented graph model as
     native processing

## Parallel-run guidance

Use these queued milestones as the parallel implementation runway for deep
Signal work while Loophole-level orchestration continues elsewhere.

Working rules for that thread:

- use `legacy/cpp/` as a reference seam, not a parity checklist
- prefer small reusable kernels in `signal-dsp` over host-local utility code
- keep transport, scheduling, and diagnostics authority inside `signal-runtime`
- keep hardware and plugin specifics at the trust edge instead of leaking them
  back into generic DSP or graph crates
- keep console-node, track-lane, and mixer-topology intent visible in roadmap
  and API choices even when the immediate implementation batch is only building
  generic graph/runtime substrate
- log by meaningful algorithm/engine batch under `docs/logs/YYYY-MM/`

## Next Task

Continue `g01.006` by landing deterministic routing, latency, and
parameter-application behavior on top of the newly explicit graph node/buffer
contract before pushing deeper into `g01.007` runtime policy.
