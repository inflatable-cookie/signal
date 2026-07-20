# g10.031 PaulX Reference Recovery

Date: 2026-07-20
Status: complete; Batch 31.21 ready
Roadmap: `g10.031`
Contract: `085`

## Result

Completed Batch 31.20 without candidate DSP.

Rendered the frozen ten-source synthetic inventory through the pinned
PaulXStretch `1.6.0` core at `4x`, `8x`, and `16x`. The comparator checkout,
build shims, sources, renders, and measurements remain ignored under
`target/creative-stretch-paulx-reference-31-20/`.

Installed-app qualification on retained `M001` at `4x` found two-channel
`2048`-sample RMS-envelope correlation `0.881` and `0.889`, with zero and one
block lag. The raw pinned core was about `3 dB` louder, matching the app's
`-3 dB` main-volume setting.

Worst-channel uniform-noise crest growth:

- `4x`: `9.932 dB`
- `8x`: `11.899 dB`
- `16x`: `10.432 dB`

The rejected Signal `RenewalSpectral` row was `8.263162 dB` at `4x`. Its frozen
`6 dB` gate validly rejected that implementation, but it was not calibrated to
PaulX-like output and could not close the target.

## Architecture

Froze `CompensatedRenewalSpectral` as one complete clean-room candidate:

- exact Signal sample-centred map and target crop
- one long magnitude transform
- deterministic per-frame phase renewal
- complementary raised-cosine adjacent-frame blend
- compensation `1/sqrt(a^2+b^2)`, derived from equal-energy uncorrelated-frame
  variance
- retained linked mid/side phase ownership
- exact boundaries, `32 MiB` state cap, deterministic offline cost
- absolute structural integrity, matching-reference synthetic gates, concealed
  long-form mono authority, then independent stereo review
- whole-candidate rejection, cleanup, and minimal private admission

The compensation removes deterministic blend-position energy modulation. It
does not promise bounded waveform crest. Reference-relative diagnostics and
listening own that risk.

Authority:

- `docs/architecture/offline-creative-compensated-renewal-spectral-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
- `docs/roadmaps/g10/031-creative-time-stretch.md`

No Rust, DSP, candidate harness, report mode, fixture, API, cache, routing,
Loophole, or Chorus surface changed.

## Next Task

Run Batch 31.21 only. Implement the frozen brief once in
`signal-candidate-31-21` on
`candidate/g10-031-compensated-renewal`. Do not alter the brief during
candidate work.
