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
gates. Then measure every row's new energy and peak outside its original
support, real-endpoint/mirror closure, and inverse-FFT excluded energy through
radius `16384`.

Passage requires relative support leakage and out-of-support peak at most
`1e-12`, endpoint closure `1e-12`, and excluded atom energy `1e-12` for every
row within the radius cap. Condition one without localization is a rejection.

## Direction

Passage opens only identity reconstruction. Any algebraic, leakage, endpoint,
tail, or repeat failure closes this common-grid family and selects a
transform-family reassessment. No approximation or correction follows.

## Next Task

Implement Batch 29.6AE canonical block-tightener feasibility and stop after its
localization decision.
