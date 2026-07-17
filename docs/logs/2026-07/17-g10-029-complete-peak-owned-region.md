# g10.029 Complete Peak-Owned Region

Date: 2026-07-17
Batch: 29.7Q
Status: complete

## Scope

Test one report-only complete peak-owned eligible-region operator. Reuse the
29.7O picker, predecessor eligibility, tracked phase advance, identity offsets,
geometry, and schedule. Preserve the peer's current same-frequency relation
inside the same operation and leave ineligible regions relational.

## Evidence

The candidate is structurally exact, mechanics-exact, mono-parity safe,
silent-peer safe, and repeat-stable. It reduces the late overlay from `25/48`
to `23/48` calibrated failures and local failures from `34/48` to `27/48`.
The current relational renderer still fails fewer rows at `20/48`. Only `2/48`
rows improve completely and `46/48` regress somewhere. Evidence is
`2a52a1106fadf298`.

## Decision

Reject. Correct operator ordering helps but does not make tracked peak regions
safe in the current coherent kernel. Stop after the one authorized candidate.
Do not tune owner selection, picker, eligibility, range, scale, boundaries, or
fallback.

## Next Task

Run Batch 29.7R as a current-kernel operator review. Decide whether linked
tracked peaks close for this kernel or require a separately contracted
phase-field kernel family. Keep another renderer and Batch 29.8 closed.
