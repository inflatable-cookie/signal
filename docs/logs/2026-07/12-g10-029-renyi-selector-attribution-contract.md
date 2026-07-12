# g10.029 Rényi Selector-Failure Attribution Contract

Date: 2026-07-12
Status: frozen

## Boundary

Use the exact Batch 29.6AK coefficients and retain every baseline energy,
entropy, raw winner, selected path, failure count, and hash. Produce no audio
and do not change selector output. Gate failures remain `[0,1,0,0,2,0,0]` and
the evidence hash remains `5568f0a38f679a40`.

## Diagnostic

- eight `512`-frame time slices by coefficient-frame centre
- eight equal folded nonnegative-frequency regions across bins `0..=2048`
- coefficient-count, energy, and `energy^0.7` contributions
- relative additive closure at `1e-12`; bit-exact reconstructed baseline
- report-only leave-one-region-out entropy deltas and raw-winner changes

Measure only failed isolated-impulse, linear-chirp, and mixed-control anchors.
Counterfactual winners do not enter the legal path.

## Decision

Select comparison-region geometry only when removal of the event-facing outer
time slice consistently restores the required direction without disturbing
mixed outer-quarter controls.
Select frequency evidence only when one fixed frequency region restores all
mixed event anchors while preserving those controls. Both, neither, or split
ownership is inconclusive. A result opens only a new selector contract.

## Next Task

Run Batch 29.6AM attribution. Preserve Batch 29.6AK outputs and hashes; do not
change the selector or implement phase or stretched synthesis.
