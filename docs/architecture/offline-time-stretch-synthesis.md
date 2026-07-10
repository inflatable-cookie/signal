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

- transient detection may use shorter analysis than synthesis
- transient protection changes local synthesis positions and phase policy
  inside the same engine
- local unity-rate attack spans are balanced by bounded compensation in steady
  intervals while source anchors retain their exact projected positions
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
- exact output-length compensation debt

Linked stereo later shares the time map, resolution schedule, transient
schedule, and peak regions. Channels retain their own instantaneous frequency
and preserve interchannel analysis phase at shared peaks.

Dynamic ratio remains outside the successor until fixed-ratio mono and linked
stereo pass. Its eventual path must update the same time map continuously; it
must not concatenate independent renders.

## Staged Proof

1. current-grid adaptive transient timeline — rejected after timing and
   combined-gate failure
2. adaptive-resolution reconstruction
3. combined fixed-ratio mono candidate
4. shared-decision linked stereo
5. concealed listening and dynamic-ratio checkpoint

Each stage stays report-only until the complete gate passes. A mechanism proof
may authorize the next stage but cannot promote product quality alone.

## Rejected Shapes

- independently rendered STFT branches joined by waveform crossfade
- bounded delay alignment between those branches
- global removal of identity locking
- scalar phase-lock or long-window selector sweeps
- fixed tail envelopes or hidden output padding

## Next Task

Reassess transient ownership before implementing nonstationary resolution.
Sparse-anchor time redistribution is no longer an authorized mechanism.
