# g10.029 Common-Grid Wavelet Reassessment

Date: 2026-07-10
Status: decision frozen

## Finding

Batch 29.6I proves a stable, efficient frequency-adaptive transform, but every
band has its own coefficient-time spacing. Direct horizontal and cross-band
phase propagation would require a nonuniform heap topology.

The public filter-bank PGHI method supports controlled frequency variation
with uniform decimation. Its authors identify nonuniform-decimation heap
integration as significant future work. They describe filter-bank time stretch
as conceivable, not as a published algorithm. That is not enough authority for
Signal to invent coefficient adjacency inside a corpus candidate.

Grid-based wavelet decimation provides the missing prerequisite. It retains
frequency-dependent wavelet bandwidth while producing one aligned coefficient
matrix. The published high-resolution, redundancy-`8` configuration reports a
frame-bound ratio of `1.20`.

## Decision

Batch 29.6J freezes:

- analytic Cauchy mother wavelet with `alpha=900`
- `1536` DC-to-Nyquist channels
- `16` lowpass completion channels
- uniform `384`-frame decimation
- published digital `(0,1)` channel delays
- complete canonical-dual reconstruction

The proof must reproduce a condition ratio no worse than `1.25` and pass the
existing reconstruction controls. It performs no phase modification or time
stretch. Batch 29.6I remains valid reconstruction evidence but is not the phase
candidate geometry.

## Next Task

Implement Batch 29.6J and stop if the complete frame operator or canonical dual
cannot meet the frozen condition and reconstruction gates.
