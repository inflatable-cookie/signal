# g10.029 Boundary Geometry Reassessment Contract

Date: 2026-07-11
Status: decision frozen

## Evidence

Jacobi-backed attribution places both exact-pointwise limiting modes in residue
`0`. Their dominant bins are `2101` and `2112`, carrying about `0.498` to
`0.499` mode mass each. The global eigenvalue extrema are
`0.5008176702532068` and `1.4982679808954755`, for condition
`2.991643605821598`.

Channel `1535` owns the largest signed cross-bin contribution. In the minimum
mode its diagonal contribution is `0.5012385386`, its cross contribution is
`-0.4912819465`, and its total is about `0.00995659`. In the maximum mode its
diagonal contribution is `0.5022530423`, its cross contribution is
`+0.4922754605`, and its total is about `0.99452850`.

## Decision

Batch 29.6Y compares exactly three Hermitian frame operators across all `11`
residues:

1. full exact-pointwise operator
2. operator with the complete channel-`1535` rank-one term removed
3. operator with only channel `1535` off-diagonal terms removed

The third operator retains the channel's diagonal energy. Both ablations are
diagnostic matrices, not realizable filters or synthesis candidates.

Condition at most `1.25` after diagonalization selects separately researched
orthogonal or multi-row Nyquist completion. Failure there but success after
complete removal selects a replacement completion family. Failure after
complete removal broadens research to the entire high-edge geometry. Numerical
gate failure is inconclusive.

## Boundary

Do not implement filters, normalizers, duals, guards, phase, synthesis, corpus
rendering, linked stereo, dynamic ratio, or product routing from this decision.

## Next Task

Implement Batch 29.6Y Nyquist-completion matrix ablation and stop after its
geometry research decision.
