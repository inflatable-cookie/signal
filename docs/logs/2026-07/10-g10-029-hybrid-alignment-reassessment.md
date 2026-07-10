# g10.029 Hybrid Alignment Reassessment

Date: 2026-07-10
Status: independent-output alignment rejected; successor policy promoted

## Diagnostic

Added report-only best-lag evidence for every attempted branch transition. The
search covers `-256..=256` incoming frames and does not change candidate audio.

The 20-source, 60-render bounded corpus reported:

- applied ownership spans: `56`
- rejected ownership spans: `1968`
- lag-recoverable transition decisions: `2339`
- lag-recoverable rejected spans: `980`
- mean absolute best lag: `152.383` frames
- maximum absolute best lag: `256` frames, the search bound
- mean entry/exit lag disagreement for recoverable spans: `210.465` frames
- maximum entry/exit disagreement: `512` frames

A fixed branch delay cannot satisfy both ends of a typical span. The required
lags also exceed the one-frame event-placement tolerance by two orders of
magnitude. Delay search, phase-neutral waveform crossfade, and relaxed
correlation thresholds are rejected.

## Decision

Promote one synthesis timeline with local transient time mapping, followed by
adaptive/nonstationary resolution if the transient proof passes. Published
phase-vocoder and Gabor work supports that direction without inspecting
external implementation source.

Canonical refs:

- `docs/research/translation-memos/001-offline-time-stretch-successor.md`
- `docs/architecture/offline-time-stretch-synthesis.md`
- `docs/contracts/082-offline-time-stretch-synthesis-policy-contract.md`

Production and product posture remain unchanged. Linked stereo stays closed.

## Next Task

Start the current-grid adaptive transient timeline proof. Keep the existing
classifier frozen and stop before adaptive resolution if the transient/time-map
gate fails.
