# g10.029 Analytic Overlap Rejection

Date: 2026-07-16
Scope: Batch 29.7H report-only linked-stereo synthesis ablation

## Result

Analytic positive-frequency accumulation and the current real-frame overlap
produce exactly equal quadrature IPD at `0.75x`, `1.5x`, and `2.0x`.
Constant-relation oracle error differs by less than `1e-14`. Correlated-image
metrics differ by at most `2e-15`.

Structure, hard-pan silence, swap, polarity, coverage, finiteness, boundaries,
and repeat pass. FFT rounding changes mono samples by at most
`2.220446e-16`, `3.330669e-16`, and `2.220446e-16`. Those changes create
`9164`, `18212`, and `24148` duplicate-mono bit mismatches without changing
quality.

Evidence hash: `db73736856099b7d`.

## Decision

Reject analytic overlap. Complex and real accumulation are linearly equivalent
for the frozen renderer. Support synthesis exposes the residual but is not its
causal owner. Batch 29.8 remains closed.

## Next Task

Run Batch 29.7I complete coefficient-contribution attribution. Isolate
initial-frame, fallback, and weak-bin contributions excluded from the 29.7F
significant-bin relation trace.
