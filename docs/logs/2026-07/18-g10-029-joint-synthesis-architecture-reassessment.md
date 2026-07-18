# g10.029 Joint-Synthesis Architecture Reassessment

Date: 2026-07-18
Batch: 29.7AB
Status: complete; frequency-adaptive direction closed

## Attribution

Batch 29.7AA proves the requested relation on each active coefficient, not
that the modified redundant coefficient field is synthesis-consistent. With
analysis `A` and canonical-dual synthesis `D`, identity proves `D A = I`.
Modified fields additionally need `A D C = C`.

The first causal divergence is the inner atom sum. Peer/reference phase and
magnitude ratios vary by atom, so their weighted sums do not preserve one
coefficient-local relation. The common material operator cancels within each
atom but changes cross-atom interference. Outer slice overlap adds another
such sum but is not the first cause.

This explains why exact coefficient relation error (`1.78e-15`) coexists with
`44/48` calibrated and `46/48` local stereo failures. It also agrees with the
earlier fixed-grid synthesis-closure, analytic-overlap, and complete-
coefficient attributions.

## Research Result

Primary STFT consistency work treats modified coefficients as an estimation
problem because arbitrary fields need not be analysis fields. Multichannel
consistency research confirms that inverse transform and re-analysis can
change both amplitude and phase and that the reconstructed waveform must own
quality assessment. Sliced NSG work proves exact reconstruction and bounded
execution for analysis coefficients, not closure of arbitrary nonlinear edits.

Clean-room source evidence uses channel linking inside one complete phase and
synthesis topology. It does not support coefficient-local relation as a
sufficient output invariant.

## Decision

Promote Rule 31K and memo 015. Close the current frequency-adaptive sliced
material direction. Retain its frame, relation, and material results as
mechanism evidence only.

The native stretch program remains active. Another transform renderer requires
a source-backed joint operator satisfying per-channel transform consistency
and post-projection spatial constraints with fixed bounded execution. Batch
29.8, listening, dynamic ratio, realtime, routing, cache, and product work
remain closed.

## Next Task

Run Batch 29.7AC as a no-renderer study of paired-channel joint consistency.
Promote one complete bounded operator or close transform-domain projection
before another DSP candidate.
