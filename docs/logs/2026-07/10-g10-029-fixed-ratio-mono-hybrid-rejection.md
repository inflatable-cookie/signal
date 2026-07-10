# g10.029 Fixed-Ratio Mono Hybrid Rejection

Date: 2026-07-10
Status: Batch 29.6 measured; first candidate rejected

## Changed

- added a report-only fixed-ratio mono renderer for the frozen transient,
  mixed, and tonal branches
- rendered all branches continuously and applied ownership only by contiguous
  classifier span
- applied the frozen 256-frame raised-cosine transitions only when zero-lag
  correlation was at least `0.50` and normalization stayed within `1 dB`
- kept an unsafe span entirely on the current mixed path
- added deterministic transition evidence and broad-corpus report rows

Production routing, cache identity, pitch and dynamic-ratio paths,
RealtimePreview, and product receipts remain unchanged.

## Result

The first mono candidate is rejected under the frozen stop conditions.

- anchored `L001` crest: `5.655483 dB` current and candidate; required
  improvement was at least `3 dB`
- worst candidate crest: `5.655483 dB`; it did not move, but the target event
  also did not improve
- finite timing rows: `48`; candidate changed three and improved mean absolute
  placement by `0.704939` frames, with no worsened row
- mean fast-movement delta: `-0.000061150` at `1.25x` and `-0.000162400` at
  `1.5x`; both ratios moved in the required direction
- mean static residual delta: `+0.000103900` at `1.25x` and `-0.000036950` at
  `1.5x`; the `1.25x` static-spectrum gate failed
- mean unsupported-bin delta: `-0.000011750` at `1.25x` and `-0.000000800`
  at `1.5x`
- integrity: `60/60`; transient: `60/60`; tonal: `50/60`; formant: `59/60`;
  combined: `50/60`
- ownership spans applied: `56`; rejected: `1968`

The transition fallback preserved current behavior too often to reach the
crest target. Where branch audio did enter, the candidate still missed the
static-spectrum and combined gates. The classifier constants remain frozen;
Batch 29.7 linked stereo does not open.

Evidence was generated at
`target/stretch-corpus-g10-029-structural-hybrid-v1.txt` from the 20-source,
60-render bounded corpus with a `120000`-frame source limit. The target report
is local generated evidence and is not committed.

## Next Task

Stop for structural reassessment. Determine whether branch alignment can be
made ownership-compatible without broad fallback or whether the transient
target needs a different synthesis mechanism. Do not tune the frozen
classifier thresholds, open linked stereo, or alter production routing.
