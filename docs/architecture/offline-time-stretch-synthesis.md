# Offline Time-Stretch Synthesis

Status: active successor architecture
Owner: dsp
Updated: 2026-07-10
Contract refs: `046`, `082`
Roadmap ref: `g10.029`

## Current Boundary

The production OfflineHighQuality prototype remains the current `2048/512`
identity-lock/reset phase vocoder. The rejected report-only hybrid renders
independent short, current, and long STFT outputs. It is evidence, not the
successor architecture.

## Successor Shape

The next successor candidate owns one sample-domain time map and one
frequency-adaptive, perfectly reconstructable filter bank.

- a painless frequency-adaptive nonstationary Gabor transform gives long
  low-frequency atoms and short high-frequency atoms inside one invertible
  representation
- absolute analysis centres follow the exact requested source map; adjacent
  integer analysis intervals may differ by one frame
- canonical dual filters reconstruct the unmodified coefficients before any
  stretch processing opens
- a later proof may integrate a filter-bank phase gradient, but only after
  reconstruction, impulse delay, coefficient coverage, and determinism pass
- a later stretched candidate must assign every significant coefficient phase
  exactly once from the strongest available time or frequency predecessor
- no peak tracker, onset detector, reset schedule, component mask, independent
  component synthesis, or local timing compensation enters the first proof
- boundary padding, normalized overlap-add, exact cropping, and identity bypass
  remain common synthesis policy

## State Ownership

One fixed-ratio mono engine owns:

- source and output frame cursors
- absolute ideal and rounded analysis-frame positions
- one immutable frequency lattice, analysis-filter bank, dual-filter bank, and
  global ratio
- bounded per-band coefficient storage and exact coefficient time positions
- later phase-derivative and integration state only after the transform proof
- deterministic insignificant-bin phase
- exact output-length and crop state

Linked stereo later shares the time map and phase-propagation decisions.
Channels retain their own complex spectra and phase gradients. Independent
per-channel heap topology is not an acceptable stereo path.

Dynamic ratio remains outside the successor until fixed-ratio mono and linked
stereo pass. Its eventual path must update the same time map continuously; it
must not concatenate independent renders.

## Staged Proof

1. current-grid adaptive transient timeline — rejected after timing and
   combined-gate failure
2. fixed-map peak transient proof — rejected after crest, placement, spectrum,
   and combined-gate failure
3. iterative H/R/P separation and exact reconstruction proof
4. additive H/R/P fixed-ratio mono candidate
5. fixed-resolution full phase-gradient kernel proof
6. whole-band full phase-gradient fixed-ratio mono gate
7. exact-lattice whole-band phase-gradient mono gate
8. frequency-adaptive painless reconstruction proof
9. frequency-adaptive phase-gradient mono mechanism and corpus gate
10. shared-decision linked stereo
11. concealed listening and dynamic-ratio checkpoint

Each stage stays report-only until the complete gate passes. A mechanism proof
may authorize the next stage but cannot promote product quality alone.

## Rejected Shapes

- independently rendered STFT branches joined by waveform crossfade
- bounded delay alignment between those branches
- global removal of identity locking
- scalar phase-lock or long-window selector sweeps
- fixed tail envelopes or hidden output padding
- local unity-ratio attack islands with steady-interval compensation
- two-way H/P processing that forces ambiguous content into a specialized path
- additive H/R/P TSM after its complete mono gate failed
- WSOLA as the next full-band path
- onset-adaptive windows coupled to local unity-ratio attack islands

## Separation Boundary

Contract `082` freezes a refined H/R/P decomposition. Long-resolution analysis
extracts only clearly harmonic bins. Short-resolution analysis extracts only
clearly percussive bins from the complement. The residual owns everything
ambiguous. Binary complement masks and normalized inverse STFT must prove exact
source reconstruction before component TSM opens.

This boundary is retained as proven historical evidence. Additive component TSM
failed and is not the active successor shape.

## Phase-Gradient Boundary

Contract `082` freezes the first active successor proof. It uses a
`4092`-sample Hann window, `8192`-point FFT, fixed `1024`-sample synthesis hop,
and nearest-integer analysis hop derived from the ratio. Centered finite
differences estimate both components of the analyzed phase gradient. A bounded
max heap integrates phase with the published trapezoidal rules.

The implementation operates on the nonredundant spectrum and mirrors synthesis
coefficients to enforce real output. The first frame keeps analyzed phase.
Bins below the frame-pair relative tolerance keep analyzed phase instead of
receiving random values. These are deterministic Signal boundary choices, not
claims about the reference implementation.

The first fixed-hop candidate is rejected but leaves useful tonal evidence. Its
constant rounded analysis hop made the internal ratio differ from the requested
ratio and allowed cumulative source-map drift. The next candidate replaces
that repeated hop with absolute rounded analysis centres. All other
phase-gradient policy remains frozen.

Exact lattice also failed the complete gate. Fixed-resolution phase-gradient
processing is retained as evidence, not the active product candidate.

## Frequency-Adaptive Boundary

The next proof changes transform resolution without changing local time. Use a
frequency-adaptive painless nonstationary Gabor frame: constant-Q-spaced
interior bands, explicit DC and Nyquist completion bands, compact frequency
supports, and canonical dual filters derived from the frame operator. Every
coefficient remains part of one transform and every sample follows the same
global map.

This is not the published onset-adaptive NSG time-stretch algorithm. That
algorithm couples short attack windows to onset detection and unity stretch at
attacks. Signal does not adopt those policies. Batch 29.6I proves only analysis
and synthesis identity, bounded filter support, complete spectral coverage,
band impulse delay, finite coefficients, and repeatability. No phase
propagation or stretched audio belongs in that batch.

Batch 29.6I's unequal per-band decimations do not form one aligned coefficient
matrix. Published filter-bank PGHI covers controlled frequency variation only
on a common time lattice and names nonuniform-decimation integration as future
work. Direct propagation on the Batch 29.6I arrays is closed.

The next proof uses grid-decimated analytic wavelets. Uniform decimation and
deterministic channel delays provide one rectangular coefficient matrix while
wavelet bandwidth still grows with frequency. Canonical dual synthesis must be
proven from the complete uniform-filter-bank frame operator; the painless
diagonal shortcut from Batch 29.6I does not apply.

Batch 29.6J passes that proof. The analysis bank applies deterministic
frequency-response tightening before the complete alias-block frame operator
and canonical-dual solve. This keeps the frozen wavelet centres, bandwidth
progression, channel delays, hop, and redundancy while improving numerical
conditioning. It does not define modified coefficient phase.

## Next Task

Research and contract common-grid phase derivatives, delay compensation,
synthesis-time mapping, and real-output symmetry. Keep corpus rendering,
linked stereo, and product routing closed.
