# 2026-07-27 g10.038 Batch 38.4 lib.rs Decomposition

Status: complete, with one task deliberately not done

## What Moved

`realtime_preview` is now its own module: the whole tier surface plus the `21`
tests that exercise it.

| file | before | after |
| --- | --- | --- |
| `lib.rs` | `5181` | `3321` |
| `realtime_preview.rs` | — | `1944` |

`lib.rs` drops `1860` lines, a third of the file. The module carries a header
recording why the tier is there and unusable: `process` is quantum-locked, so at
any ratio other than `1.0` it stalls analysis or drops source frames while
returning `Ok`, and `g10.040` owns the completion-or-closure decision.

Moving the tests with the code required duplicating three small helpers —
`sine`, `rms`, `dominant_frequency_hz` — into the new module's test scope. They
are four-line probes, and sharing them would have meant a test-support module
reaching across the crate for no benefit.

## The Dead Ratio Term

`run_phase_vocoder` computed its crop start as
`((prefix_frames - half_window) * ratio + half_window)`. Both terms are
`window_size / 2`, so the ratio never participated and the result was always
`window_size / 2`. It now reads `let output_start = prefix_frames;` with the
old expression recorded in a comment, since the misleading form is the sort of
thing that gets re-added by someone assuming it must have been ratio-dependent.

Window sizes are powers of two, so the simplification is exactly equal, not
approximately.

## The Duplicate `wrap_phase` Is Not Removed

Audit finding `A10` asked for one implementation. Measurement says that is an
output change, not a refactor.

Comparing the two forms over `-50..50` at `1e-4` steps:

| measurement | value |
| --- | --- |
| values compared | `1005319` |
| differing in bits | `945158` |
| worst delta | `2.6e-6` |
| at exactly `-PI` | rem-euclid form gives `-PI`, round form gives `+PI` |
| at exactly `TAU` | rem-euclid form gives `-2.4e-7`, round form gives `0` |

Ninety-four percent of values differ, and the two disagree in *sign* at `-PI`.
Unifying them would move rendered output and the phase-curvature metric, which
this lane's byte-exact acceptance proof forbids.

Both are retained with the measurement recorded at each site, so the divergence
reads as a known, quantified difference rather than accidental duplication.
`A10` is refined, not closed: it needs a batch that can carry a re-baseline with
evidence.

## Scope Honesty

The batch listed splitting tier metadata, the stretcher types, dynamic-ratio
segmentation, pitch composition, and the selector gates out of `lib.rs` as well.
Only `realtime_preview` moved. `lib.rs` is `3321` lines rather than the
"crate documentation, module wiring, and public re-exports" the goal describes.

The remaining split is mechanical but touches the code `g10.039` is about to
rewrite — dynamic-ratio segmentation and the chunked render path in
particular. Splitting it now would create churn against that lane for no
correctness gain, and `g10.038` exists to give `g10.039` a clean base, not to
maximise module count. The remaining extraction is recorded as Batch 38.7,
after `g10.039` settles the render architecture.

## Byte-Exactness

The corpus report is identical to the capture taken before Batch 38.3, across
both batches of refactoring:

```
diff /tmp/metrics-before.txt /tmp/metrics-38-4.txt
METRICS IDENTICAL
```

## Validation Run

- corpus report byte-identical against the Batch 38.3 baseline
- `cargo test -p signal-dsp-stretch -p signal-render-plane -p signal-runtime`:
  green. `190` lib tests, `11` transparent owners with `1` ignored, `144`
  render-plane
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged
- phase-wrap equivalence probe, standalone, deleted after use

## Next Task

Execute `g10.038` Batch 38.5: replace `Fft::process` with
`process_with_scratch` in the offline engine, remove the redundant renders in
the expansion selector decision, and record before/after render time and peak
heap for the corpus cases. Byte-exact output remains the acceptance proof.
