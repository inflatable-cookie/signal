# g10.029 Hermitian Eigensolver Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Batch 29.6V implements one full lexicographic cyclic complex-Hermitian Jacobi
eigendecomposition for matrices of size `1..=193`.

- reject Hermitian error above `1e-12`; do not repair input
- use stable phase-reduced real Jacobi pivots in lexicographic pair order
- converge at relative off-diagonal Frobenius norm `1e-13`
- reject after `64` complete sweeps
- sort by eigenvalue then original Jacobi column
- normalize vector phase from its largest-magnitude lowest-row entry
- do not use power-iteration fallback or adaptive tolerances

## Gates

- normalized eigenpair residual at most `1e-8`
- orthogonality error at most `1e-10`
- relative trace mismatch at most `1e-12`
- relative Frobenius mismatch at most `1e-10`
- finite values, stable hashes, and exact repeat

Analytic scalar, real/complex `2x2`, diagonal, repeated, and clustered controls
precede all `33` frozen alias matrices. Passing reopens only the unchanged
alias-block attribution. DSP candidates, guards, phase work, and synthesis stay
closed.

## Next Task

Implement Batch 29.6V eigensolver proof. Do not rerun attribution.
