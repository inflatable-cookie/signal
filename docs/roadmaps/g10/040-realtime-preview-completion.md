# 040 - RealtimePreview Completion

Status: planned; blocked on `g10.039`
Owner: dsp
Created: 2026-07-27
Depends on: `g10.036`, `g10.038`, `g10.039`
Supersedes: `g10.024`, `g10.028`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `REALTIME`, `PREVIEW`

## Problem

RealtimePreview is the one stretch tier Signal wants and does not have. Live
pitch-preserving preview is genuinely useful; the current implementation is not
usable and has stalled across three roadmaps.

What exists: `g10.026` proved callback-local allocation-free STFT work, and
`g10.027` proved source-projection reporting and dynamic-ratio source/output
continuity. `g10.024` and `g10.028` are paused. Contract `046`'s callback gate
addendum holds `audio_thread_processing_supported` at `false` while the stream
is `QuantumLocked`.

Why it stalled, from the 2026-07-27 audit. `RealtimePreviewCallbackState::process`
is quantum-locked: it consumes `n` input frames and produces `n` output frames
regardless of ratio. A time stretcher cannot be rate-locked on both sides. At
ratio above `1.0` the synthesis cursor outruns the output ring, the analysis
loop breaks on its ring guard, input backs up, and the guard then jumps
`next_analysis_frame` forward — dropping source audio. `process` returns `Ok`
while doing this. The projection machinery added in `g10.027` reports the
correct source advance but nothing consumes it, so the report and the kernel
disagree by construction.

Audit finding `A11` records what accumulated around that gap: roughly `1100`
lines in `lib.rs`, a state object with about `45` fields, around `30` trivial
getters, two parallel ratio schedulers that duplicate each other, and six enum
variants that are never constructed anywhere in the workspace —
`IntegrationMode::CallbackSafeStreaming`,
`CallbackTimelineMode::SourceProjected`,
`UnsupportedMode::AudioThreadProcessing`,
`UnsupportedMode::SourceAdvanceContract`,
`UnsupportedMode::ChannelLayout`, and
`CallbackProcessError::CallbackProcessingUnsupported`. No workspace consumer
imports any `RealtimePreview` type. `fft_plans_ready` returned a condition that
can never be false and was removed by `g10.038`.

The missing piece is not more reporting. It is source ownership: the callback
must pull the source frames its ratio demands, own its own fill, underrun, and
latency behavior, and stop pretending input and output advance at the same
rate.

## Generation Runway

This lane is deliberately last in the audit-driven sequence. It is the only one
that needs a new design rather than a correction, and it starts from a crate
that `g10.036` through `g10.039` have already made correct, identified, small,
and resumable.

The visible runway is:

1. feasibility and design reassessment — is callback-safe streaming reachable
   with the current kernel, and at what latency
2. one complete streaming brief, or an explicit closure
3. one isolated implementation
4. render-plane integration behind the existing gate
5. surface reduction and closeout

The next planning checkpoint is Batch 40.1. It may close the tier instead of
opening a brief; that is a valid and recorded outcome, and it would then delete
the unreachable surface rather than carry it.

## Goals

- [ ] decide honestly whether callback-safe pitch-preserving preview is
  reachable with the admitted kernel
- [ ] if reachable, give the callback path ownership of source fill, input
  demand, underrun, and latency
- [ ] make reported source projection and actual kernel consumption the same
  number
- [ ] reach `CallbackSafeStreaming` and `SourceProjected` honestly, or record
  why not and remove the surface that implies them
- [ ] leave one tier state that matches shipped behavior

## Non-Goals

- no change to the Transparent offline renderer or its output
- no creative-character work
- no new offline algorithm family
- no cache, artifact, or export change
- no Loophole or Chorus surface
- no promotion of preview quality claims without the Contract `046` evidence

## Execution Plan

### Batch 40.1 - Feasibility And Design Reassessment

Status: blocked on `g10.039` Batch 39.5

Documentation only.

- [ ] state the quantum-locked defect exactly, with the input-drop path traced
- [ ] measure achievable algorithmic latency for the `512/128` preview geometry
  and state what it costs against `MAX_BLOCK_FRAMES` and the render quantum
- [ ] design the source-ownership model: who holds the source reader, how the
  callback expresses input demand, what happens on underrun, and how latency is
  reported
- [ ] decide whether the render plane pulls preview output or the preview state
  pulls source frames
- [ ] check the model against Contract `046`'s callback gate: bounded work, no
  allocation, no locks, no I/O, deterministic latency, linked stereo,
  dynamic-ratio alignment, seam evidence
- [ ] decide reachable or not reachable, and record the reason either way
- [ ] change documentation only

Stop condition: if the model requires I/O or unbounded work on the callback,
record that RealtimePreview cannot be a callback tier with this kernel, close
the tier, and route Batch 40.5 to surface removal instead of integration.

### Batch 40.2 - Complete Streaming Brief

Status: blocked on Batch 40.1, and only if Batch 40.1 decides reachable

Documentation only.

- [ ] freeze the callback state inventory, its capacities, and its memory
  ceiling
- [ ] freeze the source fill, input demand, and underrun policy
- [ ] freeze one ratio scheduler; the current two are redundant
- [ ] freeze the latency report and its relationship to the stream contract
- [ ] freeze dynamic-ratio alignment tolerance and its seam evidence
- [ ] freeze the evidence order, rejection rules, and cleanup
- [ ] change documentation only

### Batch 40.3 - Isolated Implementation

Status: blocked on Batch 40.2

- [ ] implement the frozen brief once, isolated per Contract `084` Rule 2
- [ ] prove allocation-free, lock-free, I/O-free callback execution
- [ ] prove sustained ratios above and below `1.0` produce continuous output
  with no dropped source frames
- [ ] prove reported source consumption equals actual kernel consumption
- [ ] prove dynamic-ratio changes land inside the frozen alignment tolerance
- [ ] measure preview quality against the offline renderer on the same material

### Batch 40.4 - Render Plane Integration

Status: blocked on Batch 40.3

- [ ] open `CallbackSafeStreaming` and `SourceProjected` only after Batch 40.3
  passes every gate together
- [ ] set `audio_thread_processing_supported` from proven behavior, not from a
  constant
- [ ] integrate behind the existing render-plane preview boundary
- [ ] prove no callback deadline miss under soak

### Batch 40.5 - Surface Reduction And Closeout

Status: blocked on Batch 40.4, or on a Batch 40.1 closure decision

- [ ] remove every enum variant, getter, and scheduler the shipped design does
  not use, including the six never-constructed variants named above
- [ ] if Batch 40.1 closed the tier, remove the whole unreachable contract
  surface and leave one honest statement of what Signal does not have
- [ ] mark `g10.024` and `g10.028` resolved through this lane
- [ ] run `effigy validate` and the full crate suite
- [ ] update Contract `046`, the stretch boundary, and the `g10` front doors

## Acceptance Criteria

- [ ] the quantum-locked defect is fixed or the tier is explicitly closed
- [ ] no path drops source frames while returning success
- [ ] reported and actual source consumption agree, proven
- [ ] callback execution is allocation-free, lock-free, and I/O-free, proven
- [ ] `audio_thread_processing_supported` reflects proven behavior
- [ ] no never-constructed variant or unreachable contract surface remains
- [ ] `g10.024` and `g10.028` are resolved rather than left paused
- [ ] full crate suite passes

## Risks and Mitigations

- Risk: another partial pass leaves a fourth stalled RealtimePreview roadmap.
  Mitigation: Batch 40.1 may close the tier, and closure routes directly to
  surface removal. A recorded no is a successful outcome for this lane.
- Risk: source fill pulls I/O onto the callback. Mitigation: it is an explicit
  Batch 40.1 stop condition.
- Risk: preview quality is unusable even when the mechanism is correct.
  Mitigation: Batch 40.3 measures preview against the offline renderer before
  integration opens.
- Risk: the surface grows again while the design is unproven. Mitigation: Batch
  40.2 freezes the state inventory before implementation and one scheduler
  replaces two.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the Batch 40.1 latency measurement and reachability decision
- [ ] the frozen state inventory and memory ceiling
- [ ] sustained-ratio continuity proof with no dropped source frames
- [ ] allocation, lock, and I/O freedom proof on the callback path
- [ ] soak evidence for callback deadlines before integration
- [ ] commands actually run

## Next Task

Blocked. Open Batch 40.1 after `g10.039` Batch 39.5 closes. It is documentation
only and may close the tier.
