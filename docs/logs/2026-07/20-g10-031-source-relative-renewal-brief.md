# g10.031 Source-Relative Renewal Brief

Date: 2026-07-20
Batch: 31.26
Status: complete

## Decision

Freeze `SourceRelativeRenewalSpectral` as the sole next neutral-`Dream`
candidate. Retain the Batch 31.25 mono renderer that passed construction,
structural, synthetic, and concealed mono listening. Replace its rejected
mid/side magnitude representation with native left/right complex analysis.

One counter phase renews each linked spectral pair. Per-channel magnitudes stay
source-owned. Neutral `space` preserves the source interchannel relation;
increasing `space` widens only non-zero coherent relations above the protected
low band. Duplicate mono cannot acquire width.

## Frozen Boundary

- exact `4x` through `16x` source/output map and target length
- long-window renewal, variance-compensated pairwise blend, and fixed exterior
  envelope
- explicit DC, Nyquist, silence, polarity, anti-phase, duplicate, swap, and
  linked-channel rules
- `32 MiB` duration-independent working-state ceiling and deterministic
  counter addressing
- construction `1/1`, structural `15/15`, and synthetic `9/9` ownership
- repeated concealed mono pack and a same-source stereo pack at `4x`, `8x`,
  and `16x`
- whole, band, and mapped-window channel-balance limits before listening
- operator speaker pre-screen followed by mandatory eligible independent
  stereo listening
- terminal rejection cleanup and minimal private admission

No DSP, harness, fixture, API, route, cache, Loophole, or Chorus surface
entered `main`.

## Next Task

Run Batch 31.27 only. Create `signal-candidate-31-27` on
`candidate/g10-031-source-relative-renewal`, implement the frozen brief once,
complete construction `1/1`, freeze one checkpoint, and run gates in order.
Stop on the first miss. Do not push.
