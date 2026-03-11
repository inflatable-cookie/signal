# g01.005 Parameter Segment and Block Helper Tranche

Date: 2026-03-10
Owner: core-product

## Summary

Added the first graph-facing control application layer on top of the new
`signal-dsp` kernels. Signal now has sample-accurate parameter segment playback
and small block helpers for applying gain control, cutoff control, and delay
feedback control without pushing those per-sample mechanics up into
`signal-graph` or `signal-runtime`.

## Work completed

- extended `crates/signal-dsp/src/control.rs` with:
  - `ControlSegmentShape`
  - `ControlSegment`
  - `ControlSegmentPlayer`
- added sample-accurate block rendering for:
  - step changes
  - linear ramps
  - exponential ramps
- added a new graph-facing block helper layer in
  `crates/signal-dsp/src/block.rs`:
  - `apply_gain_control`
  - `process_low_pass_with_cutoff_control`
  - `process_delay_with_feedback_control`
- exported the new control and block helpers from
  `crates/signal-dsp/src/lib.rs`
- added integration-style tests for:
  - parameter segment playback across block boundaries
  - gain control driven by rendered control buffers
  - low-pass bypass and reset continuity at block boundaries
  - delay feedback continuity, bypass, and reset across blocks
- updated `docs/roadmaps/g01/005-core-dsp-kernel-and-control-signal-baseline.md`
  so the completed kernel/control items are no longer left unchecked

## Why this tranche matters

The first kernel tranche made `signal-dsp` credible as a kernel home, but it
still left higher layers responsible for turning parameter intent into
sample-accurate block behavior. This tranche moves that low-level control
playback into the shared DSP layer so `g01.006` can build graph execution
semantics on top of reusable control blocks instead of bespoke per-node ramp
logic.

## Realtime contract notes

- parameter segment playback allocates only at segment-plan construction time;
  block rendering itself uses caller-provided slices
- block helpers stay allocation-free and operate in place
- reset and bypass behavior are tested explicitly at block boundaries rather
  than only at single-block happy paths

## Validation

- `cargo test -p signal-primitives`
- `cargo test -p signal-dsp`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Notes

- repo-wide `git diff --check` is still blocked by the unrelated blank line at
  EOF in `CMakeLists.txt`; the touched DSP, roadmap, and log files pass cleanly
- this tranche intentionally keeps the helpers generic and block-local; it does
  not yet define graph scheduling, routing, or parameter ownership rules

## Next Task

Add explicit graph-node-friendly control plan wrappers and deterministic control
fixtures for impulse, step, sine, and silence paths so `g01.006` can execute
parameter-timed nodes against stable shared test inputs instead of building its
own block-control harness.
