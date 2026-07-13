# g10.029 Coefficient-Path Design Checkpoint

Date: 2026-07-13
Batch: 29.6CE
Rule: 30Z

## Scope

Consolidated Rules 30W through 30Y, Rubber Band behavioural forensics, active
peak and transient-anchor ownership, dense replica attribution, and
event-local overlap ownership. No candidate, corpus, holdout, or listening
audio was rendered. No factor, threshold, detector, schedule, stereo,
dynamic-ratio, cache, or routing work opened.

## Decision

One bounded coefficient path is supported.

- Keep one selected adaptive frame per source centre over the existing
  `512/1024/2048/4096` bank.
- Use centered reflected reads, Hann analysis and synthesis, a native FFT per
  frame, and the exact analysis-times-synthesis diagonal dual.
- Preserve native magnitudes without smoothing, interpolation, blending, or
  gain matching.
- Retain the fixed `4096` analytic spectrum only for active-peak decisions.
  Ordered trajectories carry physical angular frequency and synthesis phase.
- Map each owner by physical frequency to the nearest bin of the current native
  FFT. Preserve each native region's current analysis-phase offset from its
  owner bin.
- Initialize births from native analysis phase. Do not continue dormant-bin
  state when no active owner exists.
- Use the frozen sample-refined anchors for exact source/output centres and
  native-phase reset.
- Apply the proven conflicted-bridge background substitution from the same
  anchors. Replica protection is part of the candidate, not later cleanup.
- Preserve one output timeline, exact length, conjugate symmetry, real
  DC/Nyquist, and exact dual normalization.

The auxiliary tracking grid does not reintroduce Rule 30Y's shared-grid loss:
it owns decisions, never synthesis coefficients. The design changes phase
ownership while leaving magnitude evidence untouched because no completed
attribution supports magnitude modification.

## Evidence Boundary

- Ordinary phase transport is better than analysis-phase passthrough but does
  not close the timbral gap.
- Exact diagonal-dual synthesis is better than analysis-window partitioning.
- Hann/Hann reduces timing and timbral loss.
- Native centered reflection gives the strongest tested coefficient geometry.
- Fixed analytic active-peak ownership passes tone and anchor mechanism gates.
- Exact anchors alone do not prevent midpoint replicas; the bounded bridge
  owner does and passes the complete prior `48`-row synthetic gate.
- Simultaneous independent full-band resolution layers remain retired because
  their transport and recombination produce audible blur.

No new research question blocks implementation. The remaining risk is whether
the mechanisms compose on native grids without losing the proven phase,
transient, boundary, and replica properties.

## Next Task

Execute Batch 29.6CF under Rule 30AA. Implement the report-only native-grid
active-owner mono path. Pass the mechanism controls and complete `48`-row
synthetic gate before rendering the frozen nine-row development set. Stop and
trace the earliest owner boundary on any failure; do not tune or sweep.
