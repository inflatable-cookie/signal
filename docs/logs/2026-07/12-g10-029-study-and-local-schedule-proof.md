# Study And Local Schedule Proof

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BI`
Status: complete; synthetic phase and synthesis proof ready

## Result

Signal now has a release-only proof of the pre-synthesis successor stages.
Study computes one linked-channel timeline on the `128`-frame base grid from
all three frozen analysis layers. Each layer combines channel energy before
log-energy rise and positive spectral flux, normalizes both features over the
complete source by median and median absolute deviation, and reports its peaks
independently of event application.

Responsive and conservative exact-point policies operate on layer-local peak
candidates. Cross-layer agreement uses the contracted `256`-frame radius.
Dense candidates within `128` frames remain separately represented.

The schedule fixes boundaries and selected events as anchors. Between anchors,
integer apportionment biases event-adjacent hops toward unity, places the
compensation inside the same interval, and lands on the next anchor exactly.

## Evidence

- ratios: `0.75x`, `1.5x`, `2.0x`
- linked channels: two per control
- study frames: `129` per control
- responsive points: `15` per control
- conservative points: `4` per control
- dense-region retained points: `4` per control
- enabled/disabled evidence parity: exact
- reversed-channel study and decision equivalence: exact
- non-positive hops: `0`
- out-of-bound hops: `0`
- unordered points: `0`
- selected-event movement: `0`
- final-closure failures: `0`
- event-local unity improvement: `9.19`, `22.19`, and `47.23` frames per
  adjacent hop over the global-ratio baseline
- repeat evidence and schedule hashes: exact

## Boundary

This proof changes timing decisions only. It contains no coefficient phase
transport, event phase correction, vertical alignment, magnitude change,
tuning, corpus render, promotion, or product routing.

## Next Task

Run Batch 29.6BJ. Transport all three layers through actual source/output
intervals, then prove event correction and cross-resolution vertical alignment
as separate phase-only stages. Keep tuning closed.
