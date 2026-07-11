# g10.029 Hermitian Eigensolver Proof

Date: 2026-07-11
Status: passed

Batch 29.6V passes six analytic controls and all `33` frozen alias matrices.

- maximum eigenpair residual: `9.186641069227167e-13`
- maximum orthogonality error: `9.523848758648763e-15`
- maximum relative trace mismatch: `8.88299588285228e-16`
- maximum relative Frobenius mismatch: `1.3444330820218299e-14`
- evidence hash: `ac00e9f757b44e7a`

Evidence repeats exactly in the release proof. No attribution direction, DSP
candidate, dual, guard, phase, coefficient, or audio work is included.

## Next Task

Implement Batch 29.6W by substituting Jacobi eigenpairs into the unchanged
alias-block attribution. Stop after its direction decision.
