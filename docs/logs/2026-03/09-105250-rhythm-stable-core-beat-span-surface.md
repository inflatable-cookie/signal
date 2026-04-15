# Rhythm Stable Core Beat Span Surface

Date: 2026-03-09
Owner: core-product

## Summary

Published an explicit stable-core beat-span surface from the rhythm crate so
Signal can separate the full beat grid from the longest internally stable
windowed-tempo region. This makes the Garamond master result easier to explain:
the track still resolves to an exact integer BPM, and the remaining instability
is now visible as localized edge damage rather than a whole-track tempo failure.

## Work completed

- added `BeatGridCoreSpanDiagnostics` to
  `crates/signal-analysis-rhythm/src/lib.rs` and published it through
  `TempoDiagnostics`
- added stable-core-span detection from the 4-beat local-tempo windows, with:
  - tolerance scaled by core-window MAD
  - tiny-gap filling so isolated dropped windows do not split one stable region
  - coverage, trim, and interior-rejection reporting
- updated `file_rhythm_probe` and `offline_rhythm_demo` to print the stable core
  beat span alongside the existing tempo diagnostics
- added regression coverage for:
  - terminal window damage localizing to a trimmed edge region
  - stable core span publication on a simple integer click track
- kept the earlier real-file integer-snap fix intact so the new diagnostics do
  not regress the exact 128 BPM output on the Garamond master

## Real-file result

Test file:
`~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableIntegerTempo`
- `refined_bpm=127.96191`
- `interval_outliers=total:738/retained:670/rejected:68/leading:0/trailing:3`
- `stable_core_span=beats:216..706/seconds:101.698..331.641/coverage:0.664/windows:487/735 trim:216:32 interior:0`

This is the intended diagnostic shape for now: Signal still lands on the exact
integer BPM, exposes the noisy tail as a small trailing outlier region, and now
also shows that the longest stable window run is conservative rather than
implicitly claiming that the full track is equally stable end to end.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The stable core span is useful now, but it is currently a longest-contiguous
  stable run, not an edge-trimmed full-track summary.
- On the Garamond master that makes the published core span more conservative
  than a human would likely describe from listening, even though the final BPM
  and tempo-state result are now correct.

## Next Task

Tune the published beat-grid summary so Signal can distinguish a stable full
track with localized edge damage from a genuinely mid-track-unstable file,
either by trimming edge windows before core-span selection or by publishing both
an edge-trimmed stable summary and the longest contiguous stable span.
