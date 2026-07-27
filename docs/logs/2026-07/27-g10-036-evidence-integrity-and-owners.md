# 2026-07-27 g10.036 Batch 36.2 Evidence Integrity And Failing Owners

Status: complete

Batch 36.2 repairs the flaky creative allocation gate and lands the regression
owners for the three remaining Transparent defects before any renderer change.
No renderer or public API behavior changed.

## Allocation Gate Repair

`A17`. The `direct_renewal_dream` test module installs a process-global
`#[global_allocator]`. Its measuring flag, live and peak byte counters, and
allocation counter were process-global atomics, and its mutex serialized only
tests that took it. Every other test thread's allocations were therefore
counted against whichever thread happened to be measuring.

The state moved to const-initialized thread-local `Cell`s:

- `ALLOCATION_MEASURING`, `PROCESSING_STARTED`, `LIVE_BYTES`, `PEAK_BYTES`, and
  `PROCESSING_ALLOCATIONS` are now per-thread
- const init registers no destructor, so reading the state from inside the
  allocator hook cannot re-enter the allocator. Reads use `try_with` so a
  dying thread's deallocations cannot panic
- live bytes start at zero when measurement begins, so the reported peak is
  growth attributable to the measured region. The old global high-water mark
  minus a baseline is gone, and with it the last cross-thread term
- `ALLOCATION_LOCK` is removed. Thread-scoped state makes serialization
  unnecessary

### Proof

Two consecutive full-suite runs, unchanged results:

| run | lib tests | result | wall clock |
| --- | --- | --- | --- |
| 1 | `179` | `0` failed | `182.24s` |
| 2 | `179` | `0` failed | `181.42s` |

Both runs exit `0`. Before the repair the same suite failed
`direct_renewal_dream_structural_allocation_memory` at `53693` counted
allocations against a required `0`, while the same test passed alone.

### Negative control

A repaired gate that never fails is worthless. A deliberate
`Vec::with_capacity(8)` was added to the dream render's per-block loop under
`cfg(test)`. The gate reported exactly `8` processing allocations, one per
block, and failed:

```
assertion `left == right` failed
  left: 8
 right: 0
```

The probe was reverted and the gate returned to green. The repaired gate
detects real allocations and counts no noise.

## Regression Owners

Five owners land in
`crates/signal-dsp-stretch/tests/transparent_correctness_owners.rs`. They are
integration tests rather than `lib.rs` additions, so `g10.038`'s decomposition
does not have to move them, and so they exercise the crate through its public
surface.

Each is `#[ignore]`d with its owning batch named in the attribute, keeping
`main` green while the owners exist and stay runnable with `-- --ignored`. The
seam metric is duplicated locally rather than imported so the owners do not
depend on the `evidence` feature surface.

Recorded pre-fix failures:

| owner | defect | pre-fix failure | activated by |
| --- | --- | --- | --- |
| `overlap_coverage_has_no_zeroed_interior_block` | `A1` | ratio `5.0`: `90` interior blocks lost coverage | Batch 36.3 |
| `overlap_ripple_stays_within_ceiling` | `A1` | ratio `4.0` tone: `1.396 dB` against `0.5 dB` | Batch 36.3 |
| `dense_ratio_curve_preserves_pitch` | `A2` | `220.0 Hz` rendered against `440.0 Hz` | Batch 36.4 |
| `dynamic_ratio_seam_click_matches_across_channel_counts` | `A3` | mono `-28.940011 dBFS` against stereo `-180.617997 dBFS` | Batch 36.4 |
| `oversized_output_request_is_refused` | `A4` | not expressible against the current signature | Batch 36.3 |

Two controls are active immediately and must keep passing:

- `overlap_law_leaves_low_ratios_byte_exact` guards determinism and the
  output-length contract over `0.5x..3.0x`, the range the overlap law leaves
  untouched
- `dense_ratio_curve_preserves_output_length` proves a dense curve already
  holds output length. Only pitch is broken, so segment coalescing must not
  regress length

## New Evidence

Ratio `5.0` already loses `90` interior blocks. Coverage collapse starts
between `4.0` and `5.0`, not at `6.0` as the audit intake recorded. The
Contract `046` overlap law already covers this: it engages at every ratio whose
synthesis hop passes `0.75 * window_size`, which is `ratio > 3.0` at the frozen
geometry.

## Clippy Inventory

`cargo clippy -p signal-dsp-stretch --all-targets --all-features` reports nine
pre-existing warnings and none from the new owners: four
`manual implementation of .is_multiple_of()`, four
`unnecessary map of the identity function`, and one
`this function has too many arguments (13/7)` against
`build_realtime_preview_dynamic_source_projection_report`. The last is `A11`
material. All nine are `g10.038` inputs; correcting them is out of scope here.

## Validation Run

- `cargo test -p signal-dsp-stretch` twice, both green, `179` lib tests
- `cargo test -p signal-dsp-stretch --test transparent_correctness_owners --
  --ignored`, recording the five pre-fix failures above
- negative-control run with a deliberate per-block allocation, then revert
- `cargo clippy -p signal-dsp-stretch --all-targets --all-features`
- `effigy validate`
- `effigy qa:docs`

## Next Task

Execute `g10.036` Batch 36.3: implement the frozen overlap law, reducing the
analysis hop to `floor(0.75 * window_size / ratio)` when the configured
geometry would exceed `0.75 * window_size` synthesis hop; make `TimeStretcher`
fallible with the `268435456`-sample ceiling and update
`signal-render-plane` and `signal-runtime` in the same batch; activate the
three `A1` and `A4` owners; prove byte-exact output over `0.5x..3.0x`; and
re-baseline only the hashes the `3.0 < ratio <= 4.0` change invalidates under
Contract `084` Rule 10.
