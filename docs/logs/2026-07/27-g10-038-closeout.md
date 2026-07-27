# 2026-07-27 g10.038 Batch 38.6 Closeout

Status: complete; two tasks deliberately deferred

## Public Surface: What Actually Changed

| measure | Batch 38.1 | now |
| --- | --- | --- |
| exported items | `160` | `158` |
| external consumer | `36` | `36` |
| own integration tests only | `4` | `4` |
| no consumer | `120` | `118` |

The exported surface is essentially unchanged, and that deserves to be stated
plainly rather than dressed up. This lane's goal listed "public surface reduced
to what is used, retained deliberately, or scheduled by a named roadmap". Only
the third clause is satisfied by reduction; the first two are satisfied by
*classification*.

Three `measure_transient_smear_*` entry points collapsed into one, and
`StretchTransientSmearPolicies` replaced them, for a net of two. Everything else
that could have been deleted was deliberately not:

- the RealtimePreview family, `17` items, belongs to `g10.040`, which decides
  completion or closure. Closure is what removes it
- the creative family, `14` items, is the Contract `085` product surface with an
  out-of-repo consumer
- the evidence family's `72` unused items were addressed structurally rather
  than by deletion: one shared spectral surface, one measurement entry point
  per metric. Deleting the rest needs a decision about what the corpus binary
  and promotion gates are allowed to reach, which is not this lane's question

## What The Lane Did Change

| area | before | after |
| --- | --- | --- |
| `lib.rs` | `5181` lines | `3343` |
| promotion gate encodings | `3` | `1` |
| duplicated spectral helpers | `2` copies | `1` shared module |
| planner construction sites | `5` | `1` helper |
| transient-smear entry points | `4` | `1` plus policy |
| allocations per `4 s` render | `789` | `31` |
| expansion selector, ratio `2.0` | `116.0 ms` | `103.3 ms` |
| `cfg(test)`-only production paths | `2` | `0` |
| process-global test counters | `2` unsafe | `0` |

Every one of those is byte-exact: the corpus report is identical to the
baseline captured before Batch 38.3, across all four code batches.

## Deferred, With Reasons

**Batch 38.7, the remaining `lib.rs` split.** Tier metadata, the stretcher
types, dynamic-ratio segmentation, pitch composition, and the selector gates
stay in `lib.rs`. They are exactly what `g10.039` rewrites, so splitting now is
churn against that lane. Blocked on `g10.039`.

**The duplicate `wrap_phase`.** Not byte-exact: `945158` of `1005319` sampled
values differ in bits and the two forms disagree in sign at `-PI`. Unifying is
an output change, so it needs a batch that can carry a re-baseline. `A10` is
refined, not closed.

## Findings Raised By This Lane

Three test-integrity defects were found and fixed across `g10.036` to
`g10.038`: two process-global allocation counters, and one `cfg(test)`-only
path whose owner asserted the opposite of shipped behavior.

Two remain untriaged and are not this lane's to fix:

- `A19`, a `signal-plugin-bridge` shared-memory test that fails only under
  parallel load, with no mechanism identified. The Batch 38.1 sweep ruled out
  process-global test state as the cause
- `A20`, `callback_health_counters_advance_and_infer_xruns`, which asserts zero
  xruns for blocks "far faster than the deadline" on wall-clock timing. The
  mechanism is identified; the fix belongs to `signal-render-plane`

`A18`, the low-end pops on transients from `g10.036` listening, remains
untriaged and most likely shares the segment phase-restart mechanism `g10.039`
removes.

## Validation Run

- `cargo test --workspace`: green, with no `A19` or `A20` occurrence this run
- `effigy validate`
- `cargo build -p signal-dsp-stretch --all-targets` with and without features
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged throughout the lane
- corpus report byte-identical to the pre-refactor baseline
- `effigy qa:docs`

## Next Task

Execute `g10.039` Batch 39.1: enumerate every piece of renderer state that
resets at a chunk or dynamic-ratio boundary and what each reset costs audibly,
measure the chunked artifact path against a whole-buffer control, amend
Contract `046` with the resumable offline render boundary, and decide whether
the ratio curve is consumed by the renderer or stays a caller-side concern.
Documentation only.

`g10.039` carries three inherited targets: the `g10.036` seam pulse with
`segmented_render_matches_whole_render_at_constant_ratio` as its acceptance
owner, candidate finding `A18`, and the chunk-policy sensitivity that forced
chunk policy into the cache key in `g10.037`.
