# Rubber Band Mechanism Attribution

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BF`
Status: active; direct-state gap remains

## Promoted Findings

### Local timing is structural

Every row retains exact final duration, while event displacement varies by
engine, mode, ratio, and event density. At `1.5x`, the isolated impulse offsets
are `-578` frames for R2 default and `-101` for R3 standard. Dense-event offsets
differ again. A fixed local ratio is not a Rubber Band-class invariant.

Signal's next complete system must allow bounded signed local displacement and
compensate it globally to the exact requested duration.

### Phase reset owns attack shape, not the allocator alone

R2 default versus no-reset changes event crest in `17/24` comparable event rows
and replica ratio in `18/24`. Crest change is negative in `15/17` changed rows.
Event placement changes in only `12/24` rows and has mixed direction. Isolated
and dense impulses frequently retain the same position while their shapes
change.

Do not model transient reset as the time allocator. It is coordinated event
phase treatment operating under the allocator.

### Lamination is broad vertical state

R2 default versus no-lamination changes the vertical phase-coherence proxy in
`33/48` comparable mono rows and event crest in `18/24`. The direction varies
by family. Dense-event replica behavior can change sharply even when placement
does not.

Do not maximize one scalar coherence proxy. Preserve and transport vertical
phase relationships as state across tonal, transient, and mixed material.

### R3 multi-resolution behavior is material-dependent

R3 standard versus short changes:

- event placement in `23/30` comparable rows
- vertical coherence in `52/56`
- spectral residual in `49/56`
- tonal movement in `47/56`

The direction is not uniform. Short mode is closer on some isolated events;
standard is closer on other event and dense controls. Spectral and phase
effects also change sign across material. Simultaneous resolution is an active
policy surface, not a fixed-window winner.

## Non-Findings

- Waveform displacement does not reveal the internal output-increment sequence.
- The current stereo image scalar is unstable on anti-phase material and does
  not select linked-channel policy by itself.
- Boundary and soft-onset offsets carry more refinement uncertainty than hard
  impulse and dense-event rows.
- R3 internals remain opaque. Standard/short deltas are system contrasts only.

## Blocking Gap

Rubber Band's C API exposes study and rendering but not output increments,
phase-reset curves, or exact-time points. Those methods exist in the public C++
API. Add one research-only adapter, run it twice over the event controls and
five frozen modes, and record engine-confirmed sequence hashes and summaries.

Do not freeze Signal's local time allocator until that direct evidence exists.

## Next Task

Complete Batch 29.6BF with the public C++ state adapter. Keep Signal synthesis
closed.
