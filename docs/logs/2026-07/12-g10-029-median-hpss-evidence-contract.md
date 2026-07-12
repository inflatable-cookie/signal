# g10.029 Median-HPSS Evidence Contract

Date: 2026-07-12
Status: implementation ready

## Decision

Use one evidence-only median-HPSS definition:

- linked magnitude `sqrt(sum_c |Xc|^2)`
- `2048/128/4096` pre-analysis lattice
- 17-bin centred frequency median for percussive structure
- 149-frame centred time median for harmonic structure
- `p=2` soft percussive mask
- magnitude-weighted per-frame occupancy
- unchanged `0.5` local peak and detector gates

The 149-frame time median preserves the physical centre span of the primary
method's 17-frame, 1024-hop example on Signal's 128-hop grid. Both median axes
use whole-cell even reflection. No parameter sweep is authorized.

## Boundary

Measure masks, occupancy, peaks, event offsets, invariance, perturbation,
finiteness, boundary behavior, and deterministic hashes only. Do not separate,
invert, stretch, or independently phase harmonic and percussive audio. Do not
produce a schedule or modify synthesis.

Primary source: [FitzGerald, 2010](https://dafx.de/paper-archive/2010/DAFx10/DerryFitzGerald_DAFx10_P15.pdf)

## Next Task

Run Batch 29.6AX median-HPSS evidence measurement.
