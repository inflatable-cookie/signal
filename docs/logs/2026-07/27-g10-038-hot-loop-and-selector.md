# 2026-07-27 g10.038 Batch 38.5 Hot Loop And Selector Efficiency

Status: complete

## Caller-Owned FFT Scratch

`DraftPhaseVocoder` called `Fft::process`, which allocates its own scratch
inside every call. Both states now carry a scratch buffer sized from
`get_inplace_scratch_len()` and use `process_with_scratch`, matching what the
RealtimePreview kernel already did.

Measured directly with a counting allocator over one `4`-second render at ratio
`2.0`, `379` STFT frames:

| | allocations |
| --- | --- |
| before | `789` |
| after | `31` |

The difference is `758`, exactly `2 x 379`. The audit's claim of two heap
allocations per STFT frame was precise, and they are gone.

Render time barely moved: roughly `24.5 ms` to `22.5 ms` for a ten-second
source. Allocation is cheap next to the transform, so the honest description of
this change is that it removes per-frame heap traffic and its unpredictability,
not that it makes rendering meaningfully faster.

## Selector Redundancy

Audit finding `A7` said the expansion selector renders the input up to three
times. Measured on a ten-second source, before any change:

| path | ratio `0.75` | ratio `1.5` | ratio `2.0` |
| --- | --- | --- | --- |
| default | `24.5 ms` | `24.6 ms` | `24.5 ms` |
| expansion selector | `23.2 ms` | `85.0 ms` | `116.0 ms` |

`4.7x` the default at ratio `2.0`. The finding is real.

The three renders themselves are **not** removable. The gate is defined as
"measure the current output, and if it did not miss transients outright,
compare its smear against a draft baseline"; the default render, the draft
render, and the short-window re-render are each load-bearing. Removing one
changes what the gate decides, which this lane forbids.

What was removable is duplicated *measurement*. Both comparisons detect source
transients from the same input with the same policy and the same geometry, and
did so twice. Source detection now runs once and both measurements reuse it:

| path | before | after |
| --- | --- | --- |
| expansion selector, ratio `1.5` | `85.0 ms` | `74.3 ms` |
| expansion selector, ratio `2.0` | `116.0 ms` | `103.3 ms` |

Roughly `11%` off the selector path. `A7` is therefore partly closed: the
duplicated detection is gone, and the multi-render cost is recorded as inherent
to the gate's definition rather than an accident.

## A Bug My Own Validation Had Been Hiding

While starting this batch, a plain `cargo test -p signal-dsp-stretch --test ...`
failed to compile the library. Batch 38.4's re-export insertion had placed the
new `realtime_preview` block directly above `pub use promotion::{...}`, taking
over the `#[cfg(any(test, feature = "evidence"))]` attribute that belonged to
the promotion block and leaving those evidence-only items unconditionally
re-exported.

Every command used to validate Batch 38.4 had either `--all-features` or a
multi-crate invocation, so the no-feature build path was never exercised and the
mis-gate went unseen. The attribute is restored, and both build modes are now
verified explicitly:

```
cargo build -p signal-dsp-stretch --all-targets                 0 errors
cargo build -p signal-dsp-stretch --all-targets --all-features  0 errors
cargo build --workspace --all-targets                           0 errors
```

The no-feature build is added to this lane's validation set. A refactor batch
that only ever builds with every feature on is not proving the crate compiles.

## Byte-Exactness

The corpus report is identical to the pre-refactor baseline through Batches
38.3, 38.4, and 38.5 together:

```
diff /tmp/metrics-before.txt /tmp/metrics-38-5b.txt
METRICS IDENTICAL
```

## Validation Run

- allocation count measured before and after with a counting allocator
- render cost measured before and after on a ten-second source
- corpus report byte-identical
- `cargo test -p signal-dsp-stretch`: `190` lib tests, `11` owners with `1`
  ignored
- `cargo test -p signal-render-plane -p signal-runtime`: green
- `cargo build` with and without features
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged

## Next Task

Execute `g10.038` Batch 38.6: run `effigy validate` and the full crate suite,
publish the reduced public surface, update the `g10` front doors, and name the
next ready batch in `g10.039`.
