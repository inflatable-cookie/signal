# 002 - Render Plane Declick And Playback Correctness

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.001
Vision tags: `RENDER-PLANE`, `RT-SAFETY`, `AUDIBLE-QUALITY`

## Problem

The render plane gates playback with hard discontinuities: stop, seek, plan
swap, gain change, and clip edges all truncate the waveform at full
amplitude. With real audio clips (g11.006 in Loophole) every transport action
clicks. Additionally the `Samples` source drops its final frame (linear
interpolation cannot reach past the buffer end), there is no looping support,
and the executor trusts the plan's channel count against the stream's buffer
without validation.

All fixes must preserve the crate's proven RT contract: zero allocation in
`render_block`, verified by the counting-allocator soak example.

## Goals

- [x] short gain ramps (~2-10 ms) on play/stop edges — no click on transport
- [x] declick on seek (ramp out at old position, ramp in at new)
- [x] clip-edge micro-fades so trimmed clips do not click at their windows
- [x] lane/master gain changes smoothed instead of stepped on plan swap
- [x] `Samples` source plays its final frame (clamp interpolation at end)
- [x] clip looping support in the compiled plan (window repeats source)
- [x] executor validates plan channel count against stream channel count and
      fails the install rather than corrupting framing

## Non-Goals

- [ ] no resampler upgrade (g10.008)
- [ ] no crossfade editing semantics — micro-fades are declick only
- [ ] no disk streaming

## Execution Plan

### Batch 2.1 - Transport Edge Ramps

- [ ] add a per-executor edge-gain state (current gain, target, step) applied
      in `render_block`; play/stop/seek set targets, never jump
- [ ] preallocate everything at construction; extend the soak example to
      exercise play/stop/seek cycles under the zero-alloc assertion

### Batch 2.2 - Clip Edges, Last Frame, Looping

- [ ] per-clip micro-fade applied inside the clip window boundaries
- [ ] clamp `Samples` interpolation at the final source frame
- [ ] optional `loop_source: bool` (or source-length modulo) on
      `RenderClipSpec`, compiled into the clip so the render path stays
      branch-cheap

### Batch 2.3 - Validation And Smoothed Gains

- [ ] `install_plan` rejects plan/stream channel mismatches with a typed error
- [ ] lane/master gain transitions ramp over one block after plan swap
- [ ] unit tests: edge-ramp shapes, last-frame playback, loop wrap, mismatch
      rejection; soak re-run recorded

## Acceptance Criteria

- [ ] starting/stopping/seeking over a full-amplitude sine produces no sample
      step larger than the ramp slope (asserted in a test)
- [ ] soak example still reports zero callback allocations
- [ ] all existing render-plane and Loophole host tests stay green

## Risks and Mitigations

- Risk: ramp state complicates the executor hot path.
- Mitigation: one shared edge-gain accumulator, applied at the master stage;
  per-clip fades computed from frame indices, no extra state.

## Evidence Requirements

- [ ] soak output (zero alloc) recorded in the progress log
- [ ] before/after test demonstrating click elimination

## Progress (2026-06-11)

- All batches landed in one pass: transport edge envelope (5 ms ramp on
  play/stop, seek defers to the envelope zero-crossing then ramps back in);
  per-lane + master gain smoothing (10 ms full swing) with state inheritance
  across plan swaps (smoothed gains and tone phases carried by lane_id, so
  recompiles never step audio); clip-edge micro-fades (32 frames, shortened
  for tiny windows); `Samples` final-frame clamp; `loop_source` looping with
  wrap interpolation; `set_stream_channels` + install-time channel mismatch
  rejection. 13 unit tests (6 new: stop ramp step bound, seek ramp-out/jump,
  gain-swap step bound, last frame, loop wrap, mismatch rejection). Soak
  extended with a seek-while-playing cycle: still zero callback
  allocations/deallocations on a real cpal stream. Pulse compile site adopts
  `loop_source: false`; pulse render-plan tests and Aura host tests green.
- Found while validating: pulse `embedded_authority_can_capture*` tests
  crash under parallel test execution because signal-host-local boot runs
  CLAP discovery over real plugin directories and instantiates third-party
  plugins concurrently. Recorded as live evidence in g10.007; serial run is
  green (122/122).

## Next Task

g10.003 (output stream hardening) — the stream that hosts this executor.
