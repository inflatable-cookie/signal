# g10.029 Peak-Owner Phase-Field Order

Date: 2026-07-17
Batch: 29.7P
Status: complete

## Scope

Attribute the rejected 29.7O overlay by peak anchor, eligible-region interior,
boundary, ratio, and control. Compare Signal's operator order with primary
peak-locked, nonstationary Gabor, and phase-gradient integration literature and
the pinned permissive Signalsmith implementation. Do not render a candidate.

## Evidence

Report-only tracing leaves 29.7O audio and evidence hashes unchanged. Relation
RMS rises from `0.057562` to `1.485181` at anchors, `0.038310` to `1.197048`
in interiors, and `0.129766` to `1.182947` at boundaries. Every ratio and both
control families have the same direction. Evidence `e1713e619138301b` repeats
exactly.

## Decision

Reject late tracked overlays, post-integration peak seeds, and boundary-only
repair. A later proof must establish one tracked phase owner, derive the
complete eligible region under that owner, and preserve the peer's current
same-frequency relation inside the same operation. Relational recurrence owns
each ineligible region unchanged.

## Next Task

Run Batch 29.7Q as one report-only complete peak-owned region proof. Reuse the
29.7O picker, eligibility, phase advance, and identity offsets. Change only
operator order and ownership. Keep Batch 29.8 closed.
