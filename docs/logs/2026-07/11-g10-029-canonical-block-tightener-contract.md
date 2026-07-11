# g10.029 Canonical Block Tightener Contract

Date: 2026-07-11
Status: decision frozen

## Candidate

Use the exact rejected `1538`-row triplet bank. For each hop-`384` alias block,
apply the positive Hermitian canonical inverse square root `S^-1/2` obtained
from the proven Jacobi eigensystem. No approximation or localization is
allowed.

## Decision Gate

First prove the transformed frame is identity within the existing numerical
gates. Then scan rows in ascending order for new energy and peak outside their
original support plus real-endpoint/mirror closure. Stop at the first violation.

Passage requires relative support leakage, out-of-support peak, and endpoint
closure at most `1e-12` for every row. It opens a separate large-probe tail
contract because the `4224`-point matrix proof cannot establish a
`16384`-frame atom radius.

## Direction

Passage opens only identity reconstruction. Any algebraic, leakage, endpoint,
tail, or repeat failure closes this common-grid family and selects a
transform-family reassessment. No approximation or correction follows.

## Next Task

Implement Batch 29.6AE canonical block-tightener feasibility and stop after its
localization decision.
