# 2026-07-27 g10.038 Batch 38.3 Metric Consolidation

Status: complete

## Shared Spectral Support

`spectral_support` now owns what `tonal_texture/spectral.rs` and
`formant_boundary/spectral.rs` each carried a private copy of:

- `window_fits`, identical in both
- `hann_window`, identical in both
- `windowed_magnitudes`, the shared core of `normalized_spectrum` and
  `smoothed_spectral_envelope`: slice, window, transform, take bins `0..=N/2`
- `plan_forward_analysis`, which returns a forward plan and its matching window
  together, so the two cannot be set up inconsistently

Each caller keeps its own post-processing, including which bins it reads.
`tonal_texture` drops bin zero because it compares spectral shape rather than
DC, and that `skip(1)` now sits at the call site with the reason written down
instead of being buried in a private copy of the transform.

## One Transient-Smear Entry Point

Four public entry points collapse to one plus a policy argument:

| removed | replaced by |
| --- | --- |
| `measure_transient_smear(..)` | `measure_transient_smear(.., StretchTransientSmearPolicies::production())` |
| `measure_transient_smear_with_policy(.., p)` | `..::symmetric(p)` |
| `measure_transient_smear_with_policies(.., i, o)` | explicit struct with `output_recovery: None` |
| `measure_transient_smear_with_output_recovery_policy(.., i, o, r)` | explicit struct with `output_recovery: Some(r)` |

`StretchTransientSmearPolicies` carries input, output, and optional recovery
detectors. `production()` is the policy the corpus and selector gates measure
with; `symmetric(p)` covers the common one-detector case. The private
eight-argument function behind them is unchanged, so no measurement moved.

## Identity Proof

The roadmap required proving every retained measurement returns identical
values before the old code is removed. The corpus report exercises all of them
deterministically — timing drift, loop-boundary click, stereo image, transient
smear, pitch error, vertical coherence, dynamic segment seam click, tonal
texture, formant boundary, and render integrity — so it was captured before the
refactor and again after:

```
diff /tmp/metrics-before.txt /tmp/metrics-after.txt
IDENTICAL: every metric row byte-for-byte unchanged
```

`67` report lines, `27` comparison rows, no numeric drift anywhere. That is a
stronger proof than per-function unit assertions would have been, because it
compares the values the promotion gates actually read.

## Size

The two duplicated helpers plus the new shared module total `162` lines against
`140` before, because the shared module carries documentation the private
copies did not. The duplication is gone even though the line count is flat, and
five planner construction sites became one helper.

## Validation Run

- corpus report captured before and after, byte-identical
- `cargo test -p signal-dsp-stretch --all-features`: `190` lib tests green
- `cargo test -p signal-render-plane -p signal-runtime`: green
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged, no new warnings
- `effigy qa:docs`

## Next Task

Execute `g10.038` Batch 38.4: split tier metadata, the stretcher types,
dynamic-ratio segmentation, pitch composition, and the selector gates out of
`lib.rs` into modules matching the existing crate layout, move the `lib.rs`
test bodies to the modules they exercise, remove the duplicate `wrap_phase` and
the dead ratio term in `run_phase_vocoder`, and prove byte-exact output across
the full regression surface.
