# 2026-04-09 - g09.007 Closeout And g09.008 Strict Handoff

## Summary

Closed `g09.007` after the offline-preview assembly carveout and runtime test
front-door normalization, then promoted the active strict lane into `g09.008`
with one bounded ready batch card.

## Reassessment

`g09.007` no longer has another honest broad runtime-decomposition seam. The
remaining warning cluster in `crates/signal-runtime/src/tests/support.rs` is
too narrow to justify a new strict batch card on its own.

The next meaningful strict seam is `g09.008` Batch 8.1:

- harden unsupported graph layout adaptation in `crates/signal-graph/src/bus.rs`
- harden invalid or lossy primitive audio-buffer construction in
  `crates/signal-primitives/src/lib.rs`
- add focused negative tests around those explicit invariant boundaries

## Surface Updates

- marked `g09.007` complete in the roadmap surfaces
- promoted `g09.008` to active
- promoted contract `076` to active
- created the new ready card
  `docs/specs/batch-cards/006-g09-008-graph-and-primitive-invariants.md`
- refreshed the strict-lane front doors to point at the new active milestone
  and ready card

## Validation Run

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/006-g09-008-graph-and-primitive-invariants.md`.
