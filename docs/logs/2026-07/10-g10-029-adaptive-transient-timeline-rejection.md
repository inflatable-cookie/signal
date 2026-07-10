# g10.029 Adaptive Transient Timeline Rejection

Date: 2026-07-10
Status: Batch 29.6B measured; mechanism rejected

## Changed

- exposed frozen classifier onset centres in the hybrid trace
- added a report-only current-grid synthesis-position scheduler
- protected non-conflicting onset islands at local ratio `1`
- interpolated compensation across steady intervals between fixed anchors
- reinitialized protected frames from analysis phase inside one `2048/512`
  phase-vocoder engine
- reported dense conflicts, hop bounds, anchor error, overlap-add coverage, and
  full corpus quality evidence

The production phase-vocoder path remains bit-exact. Product routing, cache
identity, pitch/dynamic paths, linked stereo, and RealtimePreview are unchanged.

## Result

The mechanism is rejected under contract `082`.

- anchored `L001` improvement: `0.536217 dB`; required at least `3 dB`
- worst candidate crest: `5.119266 dB`
- finite timing rows: `48`; mean absolute-placement delta `+4.942263` frames;
  worst row `+122` frames
- protected onsets: `479`; dense conflicts: `1891`
- synthesis-hop range: `128..=1664` frames
- schedule fallback rows: `5`; uncovered output frames: `0`
- integrity: `60/60`; transient: `28/60`; tonal: `22/60`; formant: `28/60`;
  boundary: `31/60`; combined: `9/60`
- mean static residual delta: `-0.000381100` at `1.25x` and `+0.001765950`
  at `1.5x`
- mean unsupported-bin delta: `-0.000000400` at `1.25x` and `+0.000053650`
  at `1.5x`

Exact anchor construction and overlap-add coverage passed. Sparse protected
onsets did not preserve unprotected musical events: compensation moved their
local placement and produced large synthesis hops. Dense material mostly fell
back or remained unprotected. Threshold tuning cannot repair this mechanism.

Local evidence:
`target/stretch-corpus-g10-029-adaptive-timeline-v1.txt`.

## Decision

Do not open adaptive-resolution Batch 29.6C. Reassess transient ownership before
more synthesis work. The next design must avoid both independent-output branch
crossfades and sparse-anchor time redistribution.

## Next Task

Stop for contract reassessment. Compare peak/group-delay transient preservation
inside a fixed global time map against explicit transient/residual separation.
Choose and freeze one mechanism before another report-only candidate.
