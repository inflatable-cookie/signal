# g10.029 Coefficient Contribution Gate Reassessment

Date: 2026-07-16
Roadmap: `g10.029`
Batch: `29.7I`
Status: complete

## Result

Complete coefficient attribution finds no remaining phase or image owner.
Initial, viable-corrected, reference-fallback, significant, and weak classes
preserve input relation within `4.440892e-16 rad`.

Fallback counts are `1`, `2`, and `1` at `0.75x`, `1.5x`, and `2.0x`; maximum
fallback energy is `2.597671e-5`. Weak coefficients carry only `0.00032%` to
`0.00053%` of total synthesized energy.

## Ablations

- initial-frame forcing improves `0.75x` IPD by `1.36e-5 rad`, then regresses
  `1.5x` and `2.0x`
- fallback forcing is neutral within measurement precision
- weak-bin forcing worsens IPD at all three ratios
- correlated-image movement is unchanged within `2.4e-14 dB`
- structure, coverage, finiteness, boundaries, repeat, and frozen current
  hashes remain intact

Evidence hash: `49bfd7c9c3bf7d21`.

## Decision

No coefficient repair opens. The residual is not assigned to coefficient
projection, real-edge constraint, omitted contribution classes, real versus
analytic overlap, normalization, or a simple boundary implementation defect.

Batch 29.7J calibrates the exact IPD/image gate against ideal and external
reference behavior. No DSP topology or listening work opens before that
measurement decision.

## Next Task

Run Batch 29.7J stereo invariant gate calibration. Keep Batch 29.8 closed.
