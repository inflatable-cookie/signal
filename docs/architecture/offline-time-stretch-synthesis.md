# Offline Time-Stretch Synthesis

Status: active successor architecture
Owner: dsp
Updated: 2026-07-11
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

## Common-Grid Phase Boundary

Keep both source and output coefficient fields on the proven `384`-frame grid.
For output column `m`, sample source coefficients at the exact fractional
coordinate `u=m/ratio`; do not move synthesis atoms or round a repeated hop.
Interpolate magnitudes and phase derivatives, not wrapped complex samples.

Each channel atom is centred at `n*384+d_k`. Estimate horizontal instantaneous
frequency first, then transport analyzed phase back to the nominal column time
with that frequency and the known deterministic delay `d_k`. Only these
delay-compensated phases may form vertical differences or heap neighbors.
Positive-frequency integration remains authoritative; canonical-dual synthesis
mirrors the solved spectrum for real output.

The first horizontal phase-difference estimator is rejected. At high
frequencies, wavelet bandwidth exceeds the `+/-62.5 Hz` residual interval
allowed by the `384`-frame hop, so heterodyned phase differences alias before
delay compensation. The transform remains valid; phase transport requires an
alias-free estimator such as an auxiliary derivative-filter ratio.

The passing proof derives that auxiliary filter from each finalized, tightened
analysis response. Multiplication by signed angular frequency in the frequency
domain represents the time derivative. At the same coefficient location, the
imaginary derivative/original cross-ratio estimates absolute instantaneous
frequency without inter-column phase unwrap. One maximum-energy qualified
channel supplies the coherent carrier for each column; channel-delay
compensation stays downstream of that estimate. All four tone controls through
`19.5 kHz` pass with maximum error `3.614443e-12` radians/sample and maximum
compensated residual `8.683081e-10` radians.

The next mechanism projects magnitude, absolute instantaneous frequency, and
delay-compensated vertical phase derivatives at exact source coordinate
`u=m/ratio`. Phase integration stays column-local: one magnitude-prioritized
heap carries horizontal candidates from the preceding solved output column and
vertical candidates from solved adjacent channels. Its fixed `3072`-entry cap
does not grow with render duration. Wrapped complex coefficients are never
interpolated. The mechanism passes all `30` synthetic control/ratio cases with
zero coordinate error, no duplicate or missing assignment, and maximum heap
occupancy `1756/3072`.

Audio synthesis cannot crop from a zero-origin circular transform without
allowing the terminal seam to contaminate the head. Before coefficient
assembly, measure the finalized canonical-dual atoms and select the smallest
whole-hop two-sided guard whose excluded energy is at most `1e-12` for every
channel, capped at `16384` frames. Reflect the source at both endpoints,
synthesize the guarded coefficient grid, then crop the protected centre. No
post-synthesis fade, normalization, zero fill, or endpoint correction may hide
a boundary failure. The exact guard proof rejects the current bank before
assembly: lowpass channel `0` retains `6.270779e-7` excluded energy at the
largest legal support radius and requires more than `16384` guard frames. The
complete-frame solve itself remains accurate at `1.051210e-12` residual.

Tail attribution now compares raw analysis, tightened analysis, and exact
canonical-dual atoms at fixed radii for representative lowpass, interior, and
edge channels. Positive-only analytic atoms and conjugate-mirrored real-output
atoms remain separate measurements. This isolates filter, tightening, dual,
mirroring, and lowpass ownership without changing transform geometry.

The attribution matrix isolates two boundary defects. Per-bin tightening raises
the channel `0` real-output tail from `1.622121e-13` to `6.270779e-7`; exact
dualization is neutral. At Nyquist, channel `1535` already carries
`1.180453e-7` raw tail and reaches `2.030199e-7` after tightening and
dualization. Representative low/interior wavelets are compact at the measured
radius. The next transform work must jointly smooth real-output DC and Nyquist
completion while preserving the interior bank.

The frozen candidate removes global pointwise tightening, retains raw channels
`0..1534`, and replaces only channel `1535` with a zero-delay, endpoint-flat
Nyquist completion across the existing `16`-spacing completion width. It does
not assume a tight partition. Complete frame bounds and the exact canonical
dual remain authoritative. Representative boundary guards precede the expensive
all-channel scan; phase mechanisms reopen only after both reconstruction and
all-channel guard passage.

The candidate passes exact canonical-dual identity but fails its first hard
gate. Frame energy spans `0.7361080721..2.1937926704`, giving condition ratio
`2.9802589505` against the `1.25` cap. The representative guard therefore does
not run. The next design boundary is conditioning: freeze one smooth,
endpoint-compatible preconditioner or normalizer that does not recreate the
DC tail caused by pointwise tightening or change the frozen Nyquist completion.

The frozen preconditioner is one common real multiplier. It uses exact inverse
square-root frame energy in the interior and blends that function to its exact
endpoint values with quintic smootherstep across the existing `16`-spacing DC
and Nyquist spans. The multiplier has zero first and second endpoint
derivatives, preserves raw filter support and relative channel geometry, and
requires no fitted derivative. This removes the known real-mirror cusp by
construction; only measured conditioning and dual-atom guards can establish
that the complete bank is usable.

## Next Task

Implement Batch 29.6R through reconstruction. Run representative guards only
after reconstruction passes. Keep the all-channel guard, phase reproof, audio
synthesis, corpus rendering, linked stereo, dynamic ratio, and product routing
closed.
