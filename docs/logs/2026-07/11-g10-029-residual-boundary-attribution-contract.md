# g10.029 Residual Boundary Attribution Contract

Date: 2026-07-11
Status: decision frozen

## Input

The three-row completion cancels its own alias coupling below `5e-15`, but the
complete untightened bank still has condition `2.0862893665`. Its global
minimum lies in residue `3`; its maximum lies in residue `8`.

## Report

Batch 29.6AC rebuilds the exact rejected `1538`-row candidate and compares four
operators across all `11` residues:

1. full candidate
2. DC rows `0..15` with off-diagonal cross terms removed
3. preserved high-edge rows `1520..1534` with cross terms removed
4. both boundary groups with cross terms removed

Every ablation retains diagonal energy and leaves interior rows `16..1519` and
completion rows `1535..1537` unchanged. The report also attributes the frozen
minimum and maximum modes by bin region, bounded bin/channel contributors,
four channel-group totals, and cross-operator Rayleigh changes.

## Direction

Condition at most `1.25` after only high-edge diagonalization selects preserved
high-edge geometry. Passage after only DC diagonalization selects DC lowpass
geometry. Passage only when both are diagonalized selects joint boundary
geometry. Failure after both broadens attribution to the complete raw bank.
Numerical, closure, or repeat failure is inconclusive.

## Boundary

This is matrix attribution, not a realizable filter. Responses, magnitudes,
rows, delays, supports, hop, normalization, reconstruction, duals, guards,
phase, synthesis, corpus rendering, stereo, dynamic ratio, cache, and product
routing remain unchanged or closed.

## Next Task

Implement Batch 29.6AC residual boundary matrix attribution and stop after its
direction decision.
