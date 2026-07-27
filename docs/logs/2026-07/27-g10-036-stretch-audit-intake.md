# 2026-07-27 g10.036 Stretch Audit Intake And Roadmap Compile

Status: complete

Planning only. No crate source changed. One temporary probe test was created,
run, and deleted; it left no tracked file.

## Scope

Operator-requested audit of `signal-dsp-stretch` and
`signal-dsp-stretch-evidence` for code quality, over-engineering, gaps, and
flaws, then compilation of the findings into the `g10` roadmap suite.

`g10.035` closed with no ready batch and required an explicit
operator-selected Signal-only planning target. This audit is that target.

## Validation Run

- `cargo clippy -p signal-dsp-stretch -p signal-dsp-stretch-evidence
  --all-targets --all-features` — clean
- `cargo test -p signal-dsp-stretch` — `178` passed, `1` failed
  (`direct_renewal_dream_structural_allocation_memory`, `left: 53693`,
  `right: 0`)
- `cargo test -p signal-dsp-stretch --lib
  direct_renewal_dream_structural_allocation_memory` — passed in isolation,
  `25.43s`
- temporary probe test against the public API for `A1`, `A2`, `A3`, `A4`

## Findings

Measured defects:

- `A1` output dropouts above `4x`. Synthesis hop is `analysis_hop * ratio`, so
  the frozen `2048/512` geometry loses overlap coverage past ratio `4.0` and
  the `1.0e-3` normalization gate zeroes samples. Interior 512-frame RMS
  blocks on a 440 Hz 48 kHz tone: ratio `2.0` `0/172` near-zero, ratio `4.0`
  `0/359` with min block RMS falling to `0.612`, ratio `6.0` `183/547`, ratio
  `8.0` `368/734`.
- `A2` dense ratio curves become varispeed. Segments shorter than
  `window_size` fall through to `linear_time_scale`, which pitch-shifts. A
  `47`-point curve at `1024`-frame spacing, ratio `2.0`, produced a dominant
  `220.0 Hz` from a `440 Hz` source against `440.0 Hz` for the same ratio with
  no curve.
- `A3` mono dynamic-ratio renders skip seam smoothing that linked stereo
  applies. Same source and curve, one boundary: mono `-28.940011 dBFS`,
  stereo `-180.617997 dBFS`.
- `A4` no output-size bound. Ratio `1.0e6` over `4096` frames allocated
  `4096000000` samples and returned after roughly one minute.
- `A5` cache identity omits render geometry, derives key tokens from `Debug`,
  and has no creative-render coverage.
- `A17` the creative allocation gate uses a process-global allocator with
  non-thread-scoped counters, so parallel test threads are counted. It is
  flaky, not a regression.

Quality and efficiency:

- `A6` offline engine calls `Fft::process`, allocating scratch twice per STFT
  frame; the preview kernel already uses `process_with_scratch`.
- `A7` the expansion selector renders the input up to three times for one
  switch decision.
- `A8` seam smoothing is a decaying DC nudge from two edge samples, not a
  crossfade, and it exists twice — in the crate and in the render plane.
- `A9` `run_phase_vocoder` computes an output offset whose ratio term is
  always zero; the result is always `window_size / 2`.
- `A10` two `wrap_phase` implementations; the streaming path uses the slower.

Surface:

- `A11` the RealtimePreview tier is unreachable: no workspace consumer, six
  never-constructed enum variants, roughly `1100` lines, about `45` state
  fields, around `30` trivial getters, two redundant ratio schedulers, and a
  `fft_plans_ready` helper that can never return false.
- `A12` the promotion policy is encoded three times.
- `A13` evidence scaffolding outweighs DSP roughly three to one, with
  near-duplicate spectral helper modules and five separate planners.
- `A14` four public `transient_smear` entry points wrap one eight-argument
  private function.
- `A15` `creative_cyclic::render` and `Plan::identity` are `cfg(test)`-only
  production paths.
- `A16` `lib.rs` is `4855` lines, about `2300` of them tests.

Architecture:

- whole-buffer-only entry points force every bounded-memory caller to slice,
  render with context, crop, and patch. `signal-render-plane` constructs one
  stretcher per chunk, resetting phase state at every boundary.

## Compiled Roadmaps

- `g10.036` transparent stretch correctness recovery — `A1`, `A2`, `A3`, `A4`,
  `A17`. Batch 36.1 ready.
- `g10.037` stretch cache identity completeness — `A5`.
- `g10.038` stretch crate surface and evidence consolidation — `A6`, `A7`,
  `A9`, `A10`, `A12`, `A13`, `A14`, `A15`, `A16`.
- `g10.039` resumable offline stretch render — `A8` and the architectural gap.
- `g10.040` RealtimePreview completion — `A11` and the tier itself, scheduled
  last by operator decision, superseding `g10.024` and `g10.028`.

## Planning Notes

Contract `084` freezes the retained baseline's mono, linked-stereo, pitch,
dynamic-ratio, cache, artifact, and RealtimePreview behavior for the duration
of successor research. That research is closed, but the freeze text is not
scoped to it, and `A2` and `A3` cannot be corrected without changing audible
output inside the retained `0.5x..4x` product range. `g10.036` Batch 36.1 is
documentation only and must resolve that authority question before any code
changes. Every downstream batch is blocked, not ready.

`A1` is an extension rather than an audible correction: at the frozen
`2048/512` geometry no ratio in `0.5x..4x` is affected, so byte-exactness
inside the product range is available as the acceptance proof.

`g10.038` deliberately does not touch the RealtimePreview surface beyond the
one helper that cannot fail, so `g10.040` keeps every input it needs.

## Next Task

Execute `g10.036` Batch 36.1: record the defect authority, amend Contracts
`046` and `084`, and decide the ratio-envelope, output-bound, and
correction-class questions. Documentation only.
