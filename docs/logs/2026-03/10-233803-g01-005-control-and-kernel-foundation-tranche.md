# g01.005 Control and Kernel Foundation Tranche

Date: 2026-03-10
Owner: core-product

## Summary

Opened `g01.005` with the first real foundational DSP/control tranche across
`signal-primitives` and `signal-dsp`. Signal now has a reusable low-level
control and kernel layer instead of a nearly empty `Gain` stub plus ad hoc
primitive wrappers.

## Work completed

- expanded `crates/signal-primitives/src/lib.rs` with:
  - `Seconds`
  - `FrequencyHz`
  - `GainLinear`
  - `StepSegment`
  - sample-rate frame/second conversion helpers
  - buffer clearing support
- replaced the old `signal-dsp` single-file stub with a small kernel surface:
  - `control.rs`
    - `LinearRamp`
    - `ExponentialRamp`
    - `SmoothedValue`
  - `mix.rs`
    - `Gain`
    - `apply_gain_in_place`
    - `sum_in_place`
    - `mix_in_place`
    - `clear_block`
  - `filter.rs`
    - `OnePoleLowPass`
  - `delay.rs`
    - `DelayLine`
  - `level.rs`
    - `PeakMeter`
    - `RmsMeter`
    - `EnvelopeFollower`
- strengthened the common DSP contract in `signal-dsp`:
  - `DspKernel` now defines `reset`, `set_bypassed`, `is_bypassed`, and
    `process_in_place`
  - denormal flushing is exposed explicitly for stateful kernels
- added deterministic unit coverage for:
  - linear/exponential smoothing behavior
  - sample-accurate block filling
  - gain bypass behavior
  - one-pole low-pass step response and bypass continuity
  - delay impulse, feedback, and bypass behavior
  - peak, RMS, and envelope tracking
  - primitive conversion and step-segment range semantics
- moved `docs/roadmaps/g01/005-core-dsp-kernel-and-control-signal-baseline.md`
  and `docs/roadmaps/g01/README.md` to `active` for this thread

## Realtime contract notes

- all stateful kernels allocate only during construction
- no kernel performs allocation inside `process_in_place`
- delay and RMS state are explicitly preallocated
- bypass behavior is explicit instead of being left to wrapper code
- denormal flushing is handled in stateful sample/update paths

## Legacy reference boundary

Behavioral reference categories worth preserving from `legacy/cpp/` were
identified, but no code was copied:

- `legacy/cpp/src/core/MeteringService.cpp`
  - behavioral reference for peak/RMS output expectations only
- `legacy/cpp/src/core/AutomationService.cpp`
  - reminder that automation smoothing/envelope behavior matters, but not an API
    shape to replicate
- `legacy/cpp/src/core/GraphNodes.hpp`
  - reference for envelope/gain application pressure only, not block utility
    ownership

The migration boundary for this tranche stays clear: `signal-dsp` owns reusable
Rust-native kernels, while `legacy/cpp/` remains only a behavioral seam for
later comparison.

## Validation

- `cargo test -p signal-primitives`
- `cargo test -p signal-dsp`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- `git diff --check` on touched DSP and roadmap files

## Notes

- this batch intentionally stops at foundational kernels; it does not bind them
  to graph/runtime execution yet
- the current kernel set is enough to stop higher layers from open-coding basic
  smoothing, gain, filter, delay, and level helpers
- follow-on work should deepen fixtures and graph-facing usage rather than
  ballooning `signal-dsp` into an undifferentiated processing grab bag
- repo-wide `git diff --check` is currently blocked by an unrelated trailing
  blank line in `CMakeLists.txt`; the touched DSP, roadmap, and log files pass
  `git diff --check` cleanly

## Next Task

Add graph-facing control application helpers on top of these kernels, including
sample-accurate parameter segment playback, kernel reset/bypass integration
tests at block boundaries, and small utility wrappers for using smoothed values,
delay lines, and filters inside upcoming `g01.006` executable graph nodes.
