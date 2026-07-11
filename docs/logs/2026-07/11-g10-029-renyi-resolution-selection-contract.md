# g10.029 Rényi Resolution-Selection Contract

Date: 2026-07-11
Status: frozen

## Selector

Use normalized local Rényi entropy at order `alpha=0.7` across the passing
`512`, `1024`, `2048`, and `4096` windows. Evaluate equal `4096`-frame source
regions every `128` frames and include each lattice's time-frequency cell area.
Silence selects `4096`.

Solve one offline minimum-total-entropy path constrained to one-level adjacent
changes. Exact ties prefer the lexicographically longer-window path. Stereo sums
channel energies before normalization and shares the path.

No onset, flux, HPSS, peak, classifier, confidence margin, or corpus-output
evidence is allowed.

## Gate

The report must recover declared impulse regions, preserve long windows on
steady tones, avoid `512` on stationary noise, handle dense/boundary events,
remain invariant to gain/polarity/channel layout, stay stable under the frozen
small perturbation, emit legal schedules, remain finite, and repeat exactly.

## Next Task

Implement Batch 29.6AK and stop at its schedule decision. Do not implement phase
or stretched synthesis.
