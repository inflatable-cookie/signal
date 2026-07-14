# g10.029 Weighted-Predictor Fidelity Contract

Status: complete
Date: 2026-07-14
Batch: 29.6CL

## Finding

The first weighted control retained the family but changed the architecture.
Pinned source comparison identifies six coupled gaps:

- transform duration and overlap
- fixed-output versus fixed-input scheduling
- actual-hop preliminary horizontal transport
- time-factor-scaled fractional input-frequency twists
- energy normalization and weak-evidence fallback
- frequency dependency order

The bass mutation and sustained-pad damage are assigned to this incomplete
topology. No individual weight or distance owns the failure.

## Frozen Signal Topology

- fixed output interval `H = round(sample_rate * 0.03)`
- centered support and transform length `N = 4H`
- square-root Hann analysis/synthesis and exact overlap normalization
- input centre `round(output_center / ratio)` with actual-hop state
- preliminary horizontal complex phase transport
- separate low-to-high vertical correction
- short distance one; long distance `round(N/H)`
- fractional input-frequency twists scaled by `H / actual_input_hop`
- already-corrected lower and preliminary upper dependencies
- target-energy normalization and energy-relative input fallback
- real DC/Nyquist, centered reflection, exact target crop

Random diffusion, peak ownership, frequency partitioning, upstream FFT/window
code, and parameter search remain closed.

## Synthetic Gate

- bit-exact identity
- exact finite covered deterministic output at `0.75x`, `1.25x`, `1.5x`, `2x`
- `55/82.4069/110 Hz` bass error at most `0.5 Hz`, no octave selection
- four-tone chord peak error at most `0.5 Hz`, out-of-band energy below `-60 dB`
- isolated/dense attacks within `256` frames, no louder midpoint replica
- exact silence and exercised weak-evidence fallback
- finite non-zero-fill boundaries
- non-zero horizontal, short/long, lower/upper, corrected, and fallback counts
- repeated evidence and output hashes

## Next Task

Batch 29.6CM implements the complete report-only topology and runs this gate.
Any failure stops before real-source rendering.
