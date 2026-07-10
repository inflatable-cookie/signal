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

The successor owns one sample-domain time map and one reconstruction timeline.

- transient event guards may use shorter analysis than synthesis
- transient attack position is estimated per spectral peak from group delay
- selected peak regions may reinitialize phase inside the same engine while
  the global synthesis positions remain fixed
- adaptive short/long resolution must use one nonstationary analysis and
  reconstruction law, not separate output waveforms
- peak regions own vertical phase coherence; per-bin instantaneous frequency
  remains explicit
- boundary padding, normalization, exact cropping, and identity bypass remain
  common synthesis policy

## State Ownership

One fixed-ratio mono engine owns:

- source and output frame cursors
- transient and stable-region schedule
- monotonic synthesis-frame positions
- active analysis resolution and reconstruction weights
- per-bin or per-channel phase propagation
- peak-region and phase-reinitialization state
- peak-local group-delay and guarded-event collection state
- exact output-length and crop state

Linked stereo later shares the time map, resolution schedule, transient
schedule, and peak regions. Channels retain their own instantaneous frequency
and preserve interchannel analysis phase at shared peaks.

Dynamic ratio remains outside the successor until fixed-ratio mono and linked
stereo pass. Its eventual path must update the same time map continuously; it
must not concatenate independent renders.

## Staged Proof

1. current-grid adaptive transient timeline — rejected after timing and
   combined-gate failure
2. fixed-map peak transient proof
3. adaptive-resolution reconstruction checkpoint
4. combined fixed-ratio mono candidate
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

## Deferred Shape

Explicit transient/residual separation remains a research fallback. It cannot
enter the successor until it has its own perfect-reconstruction analysis,
mask-continuity, component-processing, and recombination contract.

## Next Task

Prove peak-local group-delay phase reinitialization on the fixed global time
map before implementing nonstationary resolution. Sparse-anchor time
redistribution and component separation are not authorized mechanisms.
