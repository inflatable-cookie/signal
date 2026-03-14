# Rhythm Edge-Trimmed Stable Span Surface

Date: 2026-03-09
Owner: core-product

## Summary

Added a second published beat-grid stability summary so Signal can distinguish a
stable full track with localized edge damage from a stricter longest contiguous
stable run. The existing `stable_core_span` stays as the narrow contiguous view,
and the new `edge_trimmed_stable_span` now reports an edge-trimmed, mostly
stable full-track summary that is a better fit for real mastered material like
the Garamond test file.

## Work completed

- added `edge_trimmed_stable_span` to
  `crates/signal-analysis-rhythm/src/lib.rs` as part of `TempoDiagnostics`
- refactored stable-window detection so both span surfaces share the same
  keep-mask logic and small-gap filling
- kept `stable_core_span` as the longest contiguous stable run
- added `detect_edge_trimmed_stable_span(...)`, which selects the broadest span
  whose interior instability remains sparse instead of requiring a fully clean
  contiguous block
- updated `file_rhythm_probe` and `offline_rhythm_demo` to print both span
  surfaces
- added regression coverage for:
  - sparse interior instability plus tail damage on the new edge-trimmed path
  - continued exposure of the strict contiguous core span on a simple click
    track

## Real-file result

Test file:
`/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableIntegerTempo`
- `interval_outliers=total:738/retained:670/rejected:68/leading:0/trailing:3`
- `edge_trimmed_stable_span=beats:0..735/seconds:0.447..345.333/coverage:0.996/windows:732/735 trim:0:3 interior:14`
- `stable_core_span=beats:216..706/seconds:101.698..331.641/coverage:0.664/windows:487/735 trim:216:32 interior:0`

This is the intended public shape. Signal still resolves the exact integer BPM,
the localized trailing instability stays visible, and the new edge-trimmed span
now shows that the track is effectively stable end to end apart from a small
trimmed tail, while the stricter contiguous span remains available for narrower
analysis.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `effigy health`
- `effigy validate`
- `effigy test`
- `git diff --check`

## Notes

- `effigy test` initially hit the known workspace lock conflict because
  it overlapped with `effigy validate`; rerunning it serially passed.
- The new edge-trimmed span is intentionally permissive enough to keep sparse
  interior misses inside a mostly stable track summary, so the stricter
  contiguous `stable_core_span` remains important as the narrow clean-run view.

## Next Task

Use the new paired span surfaces to tune tempo-state interpretation and any
future consumer-facing summaries, especially deciding when a file should be
treated as stable with localized edge damage versus genuinely unstable through
the middle, without forcing consumers to reverse-engineer that distinction from
raw local-tempo windows.
