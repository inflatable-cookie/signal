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

The successor owns one sample-domain time map and one additive component
reconstruction.

- iterative long/short STFT analysis separates clearly harmonic, ambiguous
  residual, and clearly percussive content with complementary binary masks
- masked components sum to the source before any time modification
- harmonic content uses long-window identity-locked phase-vocoder synthesis
- residual content uses the current OfflineHighQuality kernel
- percussive content uses very-short-window normalized OLA
- all components receive one ratio and exact target length, then sum without
  branch switching, crossfade, delay repair, or gain matching
- boundary padding, normalization, exact cropping, and identity bypass remain
  common synthesis policy

## State Ownership

One fixed-ratio mono engine owns:

- source and output frame cursors
- sample-rate-scaled long and short separation geometry
- harmonic, residual, and percussive mask partition
- one global ratio and target length for all component processors
- harmonic and residual phase propagation
- percussive OLA frame positions and normalization
- sample-aligned additive recombination
- exact output-length and crop state

Linked stereo later shares mask decisions, the time map, component frame
positions, and reconstruction weights. Channels retain their own complex
spectra and instantaneous frequency. Independent per-channel masks are not an
acceptable stereo path.

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
5. shared-decision linked stereo
6. concealed listening and dynamic-ratio checkpoint

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

## Separation Boundary

Contract `082` freezes a refined H/R/P decomposition. Long-resolution analysis
extracts only clearly harmonic bins. Short-resolution analysis extracts only
clearly percussive bins from the complement. The residual owns everything
ambiguous. Binary complement masks and normalized inverse STFT must prove exact
source reconstruction before component TSM opens.

## Next Task

Reassess the offline synthesis policy after the additive H/R/P mono proof
failed timing, integrity, replica, static-spectrum, and combined gates. Do not
tune the rejected component mechanism or open linked stereo.
