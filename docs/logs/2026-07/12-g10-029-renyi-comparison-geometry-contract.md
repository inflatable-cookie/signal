# g10.029 Rényi Comparison-Region Geometry Contract

Date: 2026-07-12
Status: frozen

## Geometry

At each decision anchor, centre a natural-hop lattice for every resolution.
Include a coefficient frame only when its complete analysis-window support lies
inside the unchanged centered `4096`-frame comparison region.

- windows: `[512,1024,2048,4096]`
- natural hops: `[128,256,512,1024]`
- exact membership counts: `[29,13,5,1]`
- source-boundary behavior: unchanged whole-sample even reflection
- implementation cache: allowed by `(window,centre)`; semantically invisible

All Rényi, path, stereo, control, invariance, perturbation, and musical gates
remain unchanged. Event labels, frequency weighting, margins, and added
detectors remain excluded.

## Stop Rule

Complete passage opens only variable-hop phase contracting. Any failure stops
automatic-selector research for operator review; no geometry variant opens.

## Next Task

Run Batch 29.6AQ anchor-local geometry. Do not implement phase or stretched
synthesis.
