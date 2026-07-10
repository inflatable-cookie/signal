# g10.029 H/R/P Separation Contract

Date: 2026-07-10
Status: Batch 29.6D passed; Batch 29.6E rejected
Contract: `082`

## Problem

Signal has rejected full-band branch crossfade, branch lag repair, adaptive
transient time redistribution, and fixed-map peak phase reset. The remaining
structural family is component decomposition.

A two-way harmonic/percussive split is insufficient. Published evaluation
identifies singing voice as a failure case when harmonic material leaks into
the percussive OLA path. Noise-like and moving-pitch material is also not
cleanly harmonic or percussive.

## Decision

Use refined iterative harmonic/residual/percussive separation.

1. Long-resolution analysis extracts only clearly harmonic bins.
2. Short-resolution analysis processes the complement and extracts only
   clearly percussive bins.
3. Ambiguous content remains residual.

The separation factors are `beta_h=2` and `beta_p=2`. Horizontal and vertical
median spans represent `200 ms` and `500 Hz`. Each STFT uses a quarter-window
hop, centred zero padding, and normalized inverse overlap-add. Supported FFT
sizes are powers of two nearest `186 ms` for harmonic extraction and `11.6 ms`
for percussive extraction.

Binary masks are disjoint and exhaustive. Every source time-frequency bin has
exactly one owner. Harmonic, residual, and percussive masked spectra retain the
source complex phase and must reconstruct additively before stretching.

## Why Three Components

Median-filter H/P separation is simple and efficient, but two-way assignment
forces ambiguous material into a specialized path. Tightened masking with a
third residual allows the percussive branch to remain noise-like enough for
short OLA and keeps voice or other uncertain structure on the current
phase-vocoder path.

This is not the rejected branch hybrid. The components are complementary parts
of one source. They share one ratio and exact target length and are added
sample-aligned. No full-band ownership choice, transition crossfade, delay
repair, or gain match exists.

## Batch 29.6D

Prove separation only:

- exact binary mask partition
- exact component lengths and finite samples
- harmonic + residual + percussive source reconstruction
- peak reconstruction error at most `1e-5`
- RMS reconstruction error at most `1e-6`
- no uncovered source or endpoint loss
- deterministic component hashes
- steady sine, isolated impulse, and stationary broadband noise assigned to
  harmonic, percussive, and residual owners with at least `12 dB` margin over
  either specialized non-owner

Failure stops before any new TSM output.

## Batch 29.6D Result

Passed without parameter tuning.

- `48 kHz` geometry: long `8192/2048`, short `512/128`
- mixed-control peak reconstruction error: `8.940697e-8`
- mixed-control RMS reconstruction error: `1.939046e-8`
- head error: `3.725290e-9`
- tail error: `1.862645e-9`
- uncovered samples: `0` at both stages
- sine harmonic margin: `30.933980 dB`
- impulse percussive margin: `164.871272 dB`
- stationary-noise residual margin: `12.925746 dB`
- exact component lengths, finite samples, exact binary stage partitions, and
  repeated component hashes: pass

No component was stretched. Production routing, cache identity, linked stereo,
pitch/dynamic routing, RealtimePreview, and product surfaces are unchanged.

## Batch 29.6E

If separation passes:

- harmonic: long-window identity-locked phase vocoder
- residual: current `2048/512` OfflineHighQuality kernel
- percussive: short-window normalized OLA with no waveform search
- one fixed ratio and exact target length for all three
- sample-aligned additive recombination with no component gain correction

The complete `60`-render mono gate remains. Added evidence covers component
length and peak growth, mask/component energy, transient replica ratio, and
recombination peak growth. The strongest post-attack secondary/primary peak
ratio may worsen by at most `0.10` within one short percussive frame.

## Stop Conditions

Reject without parameter sweeps if separation misses partition,
reconstruction, boundary, determinism, or synthetic ownership. If separation
passes but the additive TSM misses any retained crest, placement, tonal,
formant, boundary, transient-replica, integrity, or combined gate, reject the
component mechanism.

Do not replace binary masks with learned or soft masks, tune separation factors
on the corpus, add waveform search, normalize components independently, or
move component timelines to rescue the first proof.

## Sources

- [FitzGerald, “Harmonic/Percussive Separation Using Median Filtering,” DAFx-10](https://www.dafx.de/paper-archive/2010/DAFx10/DerryFitzGerald_DAFx10_P15.pdf)
- [Driedger, Müller, and Ewert, “Improving Time-Scale Modification of Music Signals Using Harmonic-Percussive Separation,” 2014](https://qmro.qmul.ac.uk/xmlui/bitstream/123456789/12184/2/Driedger%20Improving%20Time-Scale%20Modification%20of%20Music%20Signals%20Using%20Harmonic-Percussive%20Separation%202013%20Accepted.pdf)
- [Driedger, Müller, and Disch, “Extending Harmonic-Percussive Separation of Audio Signals,” ISMIR 2014](https://www.audiolabs-erlangen.de/resources/2014-ISMIR-ExtHPSep/2014_DriedgerMuellerDisch_ExtensionsHPSeparation_ISMIR.pdf)
- [Driedger and Müller, “A Review of Time-Scale Modification of Music Signals,” 2016](https://www.audiolabs-erlangen.de/content/resources/MIR/00_PCD_AudioLabs/2016_DriedgerMueller_TSMOverview_AppliedSciences_ePrint.pdf)

## Next Task

Stop implementation for synthesis-policy reassessment. Batch 29.6E failed the
complete mono gate and linked stereo remains closed. See
`docs/logs/2026-07/10-g10-029-hpr-additive-rejection.md`.
