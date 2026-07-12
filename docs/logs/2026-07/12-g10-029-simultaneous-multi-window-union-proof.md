# Simultaneous Multi-Window Union Proof

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BH`
Status: complete; study and schedule proof ready

## Result

The frozen `512/2048/8192` square-root-Hann layers form one exact union frame at
quarter-window hops. Every layer analyzes the full reflected source. Their
contributions enter one summed frame operator and one pointwise output-side
canonical dual. No layer selection or independent-render crossfade exists.

The source-domain operator bounds are
`5.999999999999998..6.000000000000003`, condition
`1.0000000000000007`. Both padded-domain and source-domain uncovered counts are
zero.

## Evidence

- layer frames: `259 / 67 / 19`
- coefficients: `132608 / 137216 / 155648`
- reflected reads: `228864`
- six controls: tone, chirp, dense impulses, boundary impulses, deterministic
  noise, and silence
- maximum identity peak error: `7.771561172376096e-16`
- maximum identity RMS error: `1.4501616196920226e-16`
- maximum conjugate-symmetry residue: `1.4559018921847493e-12`
- maximum imaginary synthesis residue: `4.993478340960093e-16`
- non-finite values: `0`
- repeat evidence: exact, including schedule, window, dual, coefficient,
  output, and aggregate hashes
- empty input: exact

## Boundary

This is a release-only reconstruction proof. It contains no detector, study,
exact-point selection, local schedule modification, phase modification, tuning,
corpus render, promotion, or product routing.

## Next Task

Run Batch 29.6BI. Add linked continuous study evidence, exact-point selection,
and a positive bounded integer-hop schedule with exact final closure. Keep phase
modification and tuning closed.
