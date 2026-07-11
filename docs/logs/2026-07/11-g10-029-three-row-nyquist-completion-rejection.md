# g10.029 Three-Row Nyquist Completion Rejection

Date: 2026-07-11
Status: rejected at complete frame conditioning

## Construction Proof

The release-only candidate has `1538` rows at hop `384`, preserves channels
`0..1534`, and uses completion delays `-128`, `0`, and `+128`.

- support error: `0`
- diagonal-energy error: `3.3306690739e-16`
- completion off-diagonal error: `4.8294701571e-15`
- real-Nyquist error: `9.0502420371e-15`
- preserved hash: `899c7f7b775c1378`
- completion hashes: `c3a9e0f642b84ef4`, `50f4f8ec00b8c2eb`,
  `49dd5ab75a32d974`

The DFT-coded triplet implements its intended local cancellation.

## Complete Frame Result

All `11` Jacobi solves pass their numerical gates. The global minimum is
`0.8036585061` at residue `3`; the maximum is `1.6766641955` at residue `8`.
Condition `2.0862893665` exceeds the `1.25` cap.

Maximum proof errors are residual `3.2769745518e-13`, orthogonality
`6.6153088526e-15`, trace `8.9385368466e-16`, and Frobenius
`1.2119742216e-14`. Evidence hash `bf8ac398c7b5372b` repeats exactly.

## Decision

Reject the triplet before identity reconstruction. The original single-row
completion coupling was a real defect, but eliminating it is insufficient for
the complete untightened bank. Attribute the residual limiting modes before
changing response magnitude, row allocation, delays, or normalization.

## Boundary

Identity reconstruction, dual guards, phase, synthesis, corpus rendering,
linked stereo, dynamic ratio, cache, and product routing remain closed.

## Next Task

Freeze Batch 29.6AB residual boundary-geometry attribution.
