# g10.029 Fixed-Map Peak Transient Decision

Date: 2026-07-10
Status: mechanism frozen; implementation not started
Contract: `082`

## Decision

Batch 29.6C will test peak-selective phase reinitialization under the unchanged
global time map. Explicit transient/residual separation is deferred.

This is a bounded mechanism proof. Production OfflineHighQuality, cache
identity, pitch and dynamic routing, RealtimePreview, adaptive resolution, and
linked stereo do not change.

## Evidence Boundary

The rejected adaptive timeline proved that correct declared anchors do not make
local time redistribution safe. It improved anchored `L001` crest by only
`0.536217 dB`, moved mean event placement by `4.942263` frames, and passed
`9/60` combined rows. Hops reached `1664` frames around sparse protected events.

Signal's earlier diagnostics isolate a narrower defect: broad identity locking
causes the `L001` crest spike. The current kernel already owns complex spectra,
spectral peaks, per-bin instantaneous-frequency propagation, identity locking,
and report-only corpus comparison. A peak-local phase policy fits that seam.

## Compared Mechanisms

### Fixed-map peak/group-delay preservation

Roebel derives attack position from the energy-weighted group delay of each
spectral peak. A time-ramped analysis window supplies the reassignment term.
Reinitializing peak phases when the attack is near the analysis-window centre
avoids a local stretch-factor constraint. Peak-level selection also avoids
resetting stationary partials in the same broad frequency band.

For Signal this preserves the current constant synthesis hop and one overlap-add
timeline. It adds analysis and peak-event state, not another waveform branch.
It directly tests whether transient bins can escape invalid phase prediction
without moving neighbouring events.

### Explicit transient/residual separation

Duxbury, Davies, and Sandler classify bins as transient/noise or steady from
phase-increment stability, then improve frequency dependence with a six-band
perfect-reconstruction filter bank and adaptive per-bin thresholds. They report
better time scaling when stretching regions with little transient information.

That branch is credible but larger. Their paper also identifies fixed-window
pre-echo or poor frequency resolution, frequency-dependent threshold errors,
and synthetic distortion when sinusoidal continuity breaks. Component gain
changes can expose spectral-subtraction artifacts. Signal would need to freeze
a perfect-reconstruction multiresolution split, soft or adaptive mask
continuity, separate component processing, and recombination before rendering a
fair candidate. It would also reopen time-map policy immediately after that
policy failed.

## Frozen Batch 29.6C Shape

- current `2048/512` analysis and synthesis grid
- current constant global synthesis hop and exact crop
- frozen `g10.029` onset classifier used only as an event guard
- one companion FFT using the analysis window multiplied by a centred time ramp
- per-peak energy position from group delay, bounded by the nearest surrounding
  magnitude minima
- non-contracting collection of transient peak regions inside one guarded event
- one centre-adjacent frame where the collected regions copy analysis phase
- ordinary instantaneous-frequency propagation and current identity locking for
  every non-selected bin
- no whole-frame reset, amplitude boost, waveform crossfade, threshold sweep,
  adaptive frame position, or production routing

The centre threshold is derived from the analysis window and a reference ramp,
not fitted to the corpus. The first proof isolates phase ownership; magnitude
compensation is outside scope.

## Required Report

Each corpus row reports:

- guarded and unmatched onset events
- peak candidates and collected peak regions
- group-delay threshold crossings
- reinitialized bins and frames
- non-finite samples and uncovered output frames
- current and candidate anchored `L001` crest
- worst crest and mean absolute event placement
- source-relative residual, unsupported-bin mass, formant, boundary, integrity,
  and combined gate results

The contract `082` gate is unchanged: at least `3 dB` anchored `L001`
improvement, worst crest no higher than `5.655483 dB`, no more than one frame of
mean placement regression, `60/60` objective passes, and no spectrum, coverage,
finite-sample, or mapping regression.

## Stop Conditions

Reject the mechanism without tuning if the frozen proof misses the crest or
placement threshold, regresses static spectrum, fails any complete gate, or
requires a whole-frame reset or time-map change to appear viable.

If rejected, return to contract reassessment. Explicit separation may reopen
only after its analysis/reconstruction and recombination contract exists.

## Sources

- [Axel Roebel, “A New Approach to Transient Processing in the Phase Vocoder,” DAFx-03](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf)
- [Duxbury, Davies, and Sandler, “Separation Of Transient Information In Musical Audio Using Multiresolution Analysis Techniques,” DAFx-01](https://www.dafx.de/paper-archive/2001/papers/duxbury.pdf)

## Next Task

Implement the report-only Batch 29.6C analysis and peak-event state, then run
the unchanged contract `082` gate.
