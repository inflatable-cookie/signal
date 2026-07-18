# g10.029 Paired-Channel Consistency Operator Study

Date: 2026-07-18
Batch: 29.7AC
Status: complete; transform-domain joint projection closed

## Research Result

No reviewed source defines the required paired-channel projection. MISI and
later alternating-projection work own consistency, magnitude, and a known
additive mixture. Stereo channels do not form separated sources with a known
target sum, so that mixing projection does not transfer.

Spatial covariance matching owns spatial rendering. It requires a target
covariance and may introduce decorrelated energy. It does not uniquely preserve
the phase, waveform, or arbitrary source image of a stretched stereo pair.

Per-channel `A D` projection remains exact, but alternating it with local
relation or covariance repair has no established feasible intersection,
constraint order, finite iteration count, or non-convergence result for this
problem.

## Decision

Promote Rule 31L and memo 016. Close transform-domain post-projection. Do not
implement a guessed alternating operator, mono-downmix constraint, or covariance
repair.

The native program remains active. The next study returns to complete
architectures that own stereo linkage through waveform synthesis rather than
repairing an independently modified coefficient field.

## Next Task

Run Batch 29.7AD as a no-renderer whole-family selection. Compare complete
source-synchronous, sinusoidal, and single-grid transform topologies against
shared stereo timeline, transient, tonal, and bounded-execution requirements.
Select at most one proof. Keep Batch 29.8 and product work closed.
