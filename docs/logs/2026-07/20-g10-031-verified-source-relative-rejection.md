# g10.031 Verified Source-Relative Rejection

Date: 2026-07-20
Batch: 31.29
Status: complete; candidate rejected and deleted

## Evidence

- worktree: `signal-candidate-31-29`
- branch: `candidate/g10-031-verified-source-relative-renewal`
- immutable checkpoint: `d94612dd9f4ca9ba51724c826cac1d9375c27ff8`
- compile: pass
- construction: exactly `1/1`
- structural: exactly `15/15`
- synthetic: exactly nine selected; seven passed, `Y04` and `Y02` failed
- listening: not run

`Y04` reported two active replica regions instead of one in one `16x` row.
The owner completed both impulse sources but its failure text did not identify
which source missed.

`Y02` reported two `4x` pitch errors of `10.960881380` and `10.960712818`
cents. Their PaulX-relative ceilings were `9.410431632` and `4.461974128`
cents. The chord row and all `8x` and `16x` pitch rows passed.

The other seven synthetic owners passed: crest, impulse distribution, noise
periodicity, RMS modulation, silence gap, integrity/discontinuity, and the
linked-stereo inventory.

The dominant cause is incomplete ratio-range ownership in the frozen
single-resolution renewal renderer. The paired low-ratio tonal and high-ratio
replica misses show that the complete `32768/16384` topology does not own the
whole `4x` through `16x` range. This diagnosis does not authorize a transform,
hop, threshold, or assertion sweep.

Cleanup deleted the worktree, branch, checkpoint, module, tests, build state,
and candidate artifacts. No DSP, harness, fixture, API, route, cache,
Loophole, or Chorus surface entered `main`.

## Next Task

Run Batch 31.30 only. Reassess range ownership against pinned PaulXStretch and
the complete Batch 31.29 receipt. Either freeze one materially different,
range-aware complete successor brief or close the source-relative topology.
Do not implement candidate DSP in the same batch, reopen a local sweep, or
close the PaulX-like product target without explicit operator direction.
