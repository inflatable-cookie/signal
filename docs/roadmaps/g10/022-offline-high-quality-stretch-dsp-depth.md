# 022 - OfflineHighQuality Stretch DSP Depth

Status: planned
Owner: dsp
Created: 2026-07-07
Depends on: g10.021
Vision tags: `DSP`, `STRETCH`, `QUALITY`

## Problem

OfflineHighQuality has a clean-room foundation: phase-vocoder baseline,
identity phase locking, transient reset behavior, linked stereo, static pitch
composition, dynamic ratio segments, and cacheable render/export/freeze
artifacts. It is still a foundation, not the final quality tier. The next
implementation work should target audible quality weaknesses found by the
comparison reports, not add more receipt or fixture surfaces.

## Goals

- [ ] run the current comparison and quality-priority reports before choosing
  each algorithm change
- [ ] choose one top measured weakness per batch: transient smear, loop seams,
  sustained phasiness, stereo image drift, pitch error, or timing drift
- [ ] add a multiresolution or hybrid STFT/time-domain path when the measured
  weakness justifies it
- [ ] improve transient anchoring or local time-domain splice behavior for
  percussive material
- [ ] improve vertical coherence for dense sustained and polyphonic material
- [ ] improve loop-seam handling under fixed ratios and dynamic ratio segments
- [ ] add static-pitch quality evidence across vocals, bass, and full mixes
- [ ] keep `PhaseVocoderStretcher` as the draft baseline for regression
  comparison

## Execution Plan

### Batch 22.1 - Measured Weakness Selection

- [ ] run synthetic comparison and quality-priority reports
- [ ] record the top target and the rejected alternatives before coding

### Batch 22.2 - First Quality Improvement

- [ ] implement one DSP change against the selected target
- [ ] prove the chosen metric improves or holds without creating a higher
  priority regression

### Batch 22.3 - Multiresolution Or Hybrid Path

- [ ] add the next engine path only when a measured weakness requires it
- [ ] keep the lower-complexity path available for regression comparison

## Acceptance Criteria

- [ ] every DSP batch names the metric or listening failure it targets
- [ ] OfflineHighQuality improves or holds against the draft baseline on the
  chosen target and does not create a higher-priority regression elsewhere
- [ ] output length and deterministic cache identity behavior remain stable
- [ ] no realtime audio-thread path calls whole-buffer stretch processing

## Validation

- `cargo test -p signal-dsp-stretch`
- focused synthetic and real-corpus report runs

## Progress

- 2026-07-07: opened as active g10 OfflineHighQuality DSP quality work after
  the stretch policy/fixture line was judged mature enough and further
  proof-shaping was called out as churn.

## Next Task

Run the current synthetic comparison and quality-priority reports, then pick
one DSP weakness for Batch 22.2.
