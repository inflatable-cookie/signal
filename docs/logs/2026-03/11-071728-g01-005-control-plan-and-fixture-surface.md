# g01.005 Control Plan and Fixture Surface

Date: 2026-03-11
Owner: core-product

## Summary

Finished the remaining fixture and control-plan gap inside `g01.005`. Signal now
has an explicit `ControlPlan` wrapper for graph-facing parameter playback and a
deterministic `SignalFixture` surface for impulse, step, sine, and silence
inputs, so `g01.006` can build node execution tests on shared DSP-owned control
and input fixtures instead of inventing its own harness.

## Work completed

- extended `crates/signal-dsp/src/control.rs` with:
  - `ControlPlan`
  - `ControlSegmentPlayer::skip(...)`
  - block-offset rendering via `ControlPlan::render_block(...)`
- added `crates/signal-dsp/src/fixtures.rs` with deterministic fixture builders
  for:
  - silence
  - impulse
  - step
  - sine
- re-exported `ControlPlan` and `SignalFixture` from
  `crates/signal-dsp/src/lib.rs`
- added deterministic tests for:
  - control-plan offset rendering equivalence
  - impulse fixture placement
  - step fixture transition
  - sine fixture quadrature points
  - silence fixture zero output
- updated `docs/roadmaps/g01/005-core-dsp-kernel-and-control-signal-baseline.md`
  so the fixture item is now marked complete

## Why this tranche matters

The earlier `ControlSegmentPlayer` was enough for local block playback, but it
still required higher layers to decide how to package parameter intent for
block-at-a-time execution. `ControlPlan` now carries that graph-facing surface
directly, and the new fixtures make it possible to test node behavior against
stable known inputs before graph routing and scheduling semantics land.

## Realtime contract notes

- `ControlPlan` itself is just borrowed metadata plus an initial value
- block rendering remains allocation-free and uses caller-provided output slices
- fixture generation is deterministic test/support infrastructure and stays
  outside runtime hot paths

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
- `g01.005` is now in a shape where the next work can credibly start feeding
  these control and fixture surfaces into executable graph contracts rather than
  continuing to widen `signal-dsp`

## Next Task

Start `g01.006` by defining a first executable graph block contract that can
consume `ControlPlan`, `SignalFixture`, and the shared DSP kernels for
deterministic stage execution, routing, latency accounting, and parameter
timing tests.
