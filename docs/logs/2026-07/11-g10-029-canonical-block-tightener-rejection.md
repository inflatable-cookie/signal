# g10.029 Canonical Block Tightener Rejection

Date: 2026-07-11
Status: rejected; common-grid family closed

## Algebra

The exact per-residue canonical inverse square root passes. Transformed frame
extrema are `0.9999999999997254` and `1.0000000000003026`, for condition
`1.0000000000005773`. Maximum identity error is `2.4357207508e-14`; all Jacobi
proof errors pass.

## Support Gate

Rows `0..11` pass. Row `12` is the first violation:

- original support: `19` positive bins
- transformed support: `2113` positive bins
- relative leaked energy: `2.4085528358e-24`
- out-of-support peak: `1.2528705611e-12`
- endpoint error: `1.7792890902e-13`

The peak exceeds the frozen `1e-12` cap. This is a structural support decision,
not an audibility claim. No threshold change or localization pass is allowed.

Input hash is `7bca71708965122e`; tightener hash is `27b969b728545d19`;
evaluated-row hash is `fef18fceb671d7d0`; evidence hash is
`8a45d8c4f579a111`. All repeat exactly.

## Decision

Close common-grid correction work. Large-probe localization, reconstruction,
guards, phase, synthesis, corpus, stereo, dynamic ratio, cache, and routing do
not open.

## Next Task

Freeze Batch 29.6AF transform-family reassessment.
