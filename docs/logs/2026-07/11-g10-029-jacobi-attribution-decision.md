# g10.029 Jacobi Attribution Decision

Date: 2026-07-11
Status: boundary geometry selected

Batch 29.6W replaces only attribution eigenpairs with the proven Jacobi solve.

- exact-pointwise condition ratio: `2.991643605821598`
- endpoint-even minimum boundary mass: `0.9972172436029634`
- endpoint-even maximum boundary mass: `0.9973869345915773`
- maximum eigenpair residual: `9.186641069227167e-13`
- maximum contribution closure error: `4.268183451512147e-15`
- evidence hash: `069142f1ee68f2a4`

Exact pointwise scalar normalization fails the `1.25` condition gate, so Rule
26B selects boundary-geometry reassessment. Nyquist localization is strong but
does not authorize block-aware preconditioning because the first decision
branch governs.

No filter, normalizer, dual, guard, phase, coefficient, audio, corpus, stereo,
dynamic-ratio, or product work opens.

## Next Task

Freeze Batch 29.6X boundary-geometry reassessment.
