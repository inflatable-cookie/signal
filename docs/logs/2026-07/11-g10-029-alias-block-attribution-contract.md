# g10.029 Alias-Block Attribution Contract

Date: 2026-07-11
Status: decision frozen

## Scope

Batch 29.6T is report-only. It compares three versions of the same Rule 26 raw
boundary bank on the fixed `4224`-frame, hop-`384`, `11`-residue geometry:

1. raw
2. exact inverse-square-root pointwise normalization
3. rejected endpoint-even normalization

The exact-pointwise bank is diagnostic, not a synthesis candidate. No fourth
bank or parameter variant may enter the matrix.

## Evidence

For every bank and residue, report complete alias-block minimum and maximum
eigenpairs, condition, normalized residual, bin membership, and hashes. Fix
eigenvector phase from the largest-magnitude lowest-index bin. Residual above
`1e-6` makes the result inconclusive.

For each bank's global minimum and maximum mode, report:

- Rayleigh quotient under all three banks
- norm mass in DC, interior, and Nyquist bin regions
- top `16` bins plus aggregate remainder
- per-channel total quadratic, diagonal, and signed cross contribution
- top `16` total and top `16` absolute-cross channels plus aggregate remainders
- contribution closure to the eigenvalue within `1e-8` relative error

All evidence must be finite and repeat exactly within one build profile. Hash
the raw bank, both multipliers, matrices, eigenvectors, and complete report.

## Direction Gate

- Exact-pointwise condition above `1.25`: return to boundary geometry.
- Exact-pointwise passes but either endpoint-even limiting mode has less than
  `90%` boundary-bin mass: return to boundary geometry.
- Exact-pointwise passes, both endpoint-even limiting modes have at least `90%`
  boundary-bin mass, and all numerical gates pass: block-aware boundary
  preconditioner research may be contracted separately.

No outcome directly authorizes implementation. Do not reconstruct samples,
form duals, run guards or phase logic, assemble coefficients, or render audio.

## Next Task

Implement Batch 29.6T and stop after the direction decision.
