# Direct Channel-Local Peak Contract

Date: 2026-07-19
Batch: 29.7AZ
Status: complete

## Frozen Shape

Peak, valley, fallback maximum, and predecessor maps are per channel and per
scale. The existing Signal peak predicate and lower-bin tie remain fixed.
Terminal classification remains one shared atom decision from joint guidance.

For each locked channel-atom, the requesting channel retains its current peak.
The greatest channel at that atom is only a possible trajectory owner. An exact
tie selects the lower channel. Borrowing below `6000 Hz` requires equal
predecessor peak identity and supported owner history.

The selected channel's ordinary advance at the requesting peak is re-anchored
to the common predecessor's prior synthesis phase. The requesting atom retains
its current analysis-relative offset from the selected channel at that peak and
retains its own magnitude.

## Storage And Reporting

Current/predecessor region records remain `2CP`; analysis/synthesis phase
remains `2CP`; terminal state remains `P`. Work remains bounded by fixed `CP`
scans. Reports count borrowed and local locked channel-atoms, committed
trajectory-channel switches, and channel-peak disagreements.

## Proof Matrix

The primary `48 kHz` analytic fixture has staggered current channel peaks and
one common compatible predecessor. It first proves the current joint builder
collapses the two locations, then requires corrected mechanics to retain both.
It also proves trajectory borrowing without peak replacement, magnitude and
offset preservation at `1e-12`, and nonzero peak disagreement.

Companion cases cover incompatible predecessors, unsupported owner and
predecessor, exact owner ties, exact `6000 Hz`, swap symmetry, silence and
recovery, every terminal state, all proof rates, shape rejection before
mutation, fixed capacity, finiteness, and exact repeat.

No state code, renderer, corpus audio, or objective evidence changed.

## Next Task

Run Batch 29.7BA under Rule 31AC. Implement only these private mechanics and
proofs.
