# 2026-07-27 g10.038 Batch 38.2 Promotion Gate And Dead Surface

Status: complete

## The Three Promotion Encodings Agreed

Before collapsing them, the question the roadmap asked was answered: did the
three encodings of the product-quality gate actually disagree anywhere?

An exhaustive probe built every receipt shape over the eight evidence booleans,
both empty and non-empty evidence ids, and two offline paths — `1024` shapes —
and compared all three encodings:

| comparison | disagreements |
| --- | --- |
| boolean form against reason form | `0` |
| construction-time note against the gate | `0` |

They agreed everywhere. The consolidation is therefore behavior-preserving, and
the promotion tests pass unchanged, which is the confirmation.

The duplication was still worth removing: three encodings that agree today are
three places to edit tomorrow, and nothing forced them to stay aligned.

## One Owner, Two Renderings

`ProductFacingBlocker` is now the single gate. One evaluator,
`product_facing_blocker`, decides; the three former encodings derive from it:

- `accepts_product_facing_path` is `product_facing_path_blocker(..).is_none()`
- `product_facing_path_blocker` checks receipt identity — tier, path, status —
  then delegates to the evaluator and renders its reason
- `product_quality_rejection_note` maps the same evaluator to its own wording

The two wordings are retained deliberately. The *rule* must not be duplicated;
the phrasing differs by surface because one reports against an existing receipt
and the other explains a construction-time rejection. Keeping both as
renderings of one enum preserves the messages consumers already assert without
preserving the duplicated logic.

## Dead Surface Removed

`fft_plans_ready` returned `Arc::strong_count(..) >= 1`, which is true by
construction, and a test asserted it. Both are gone.

`creative_cyclic::render` and `Plan::identity` were `#[cfg(test)]`-only. The
test that used them,
`identity_and_empty_requests_are_exact`, asserted that an identity request
returns the input verbatim — behavior production never serves, because
`render_continuous` admits `2N..=8N` and rejects `target == N`.

The entry point and the field are removed, all `15` cyclic test call sites now
use the production entry point, and the owner is renamed
`identity_is_rejected_and_empty_requests_are_exact` and asserts
`Err(UnsupportedRatio)`. The test previously proved the opposite of what ships.

## Integration-Test Allocator Counters

Three files carried process-global allocator counters and were safe only
because each happened to contain exactly one `#[test]`:
`realtime_preview_callback_alloc.rs`, `live_render_soak.rs`, and
`capture_alloc.rs`. All three are now const-initialized thread-local `Cell`s,
so adding a second test to any of them cannot break the measurement silently.

The first mechanical pass over-reached: a regex converted unrelated
`Arc<AtomicBool>` and `Arc<AtomicU64>` fields in `capture_alloc.rs` and
`live_render_soak.rs`, which are genuine cross-thread state and must stay
atomic. Both files were reverted and redone against the three named statics
only.

## New Finding: A20

`cargo test` across four crates failed once on
`signal-render-plane` `callback_health_counters_advance_and_infer_xruns`. It
passed three times in isolation.

Unlike `A19`, this one has a mechanism. The test asserts
`controller.xrun_count() == 0` for "back-to-back blocks far faster than the
deadline", using wall-clock timing, then sleeps `20 ms` to force one xrun.
Under saturated parallel load the "far faster than the deadline" assumption
does not hold, and a spurious xrun fails the first assertion.

It is a wall-clock-dependent test, not a callback-health defect, and it is in
`signal-render-plane`, outside this lane. Recorded for triage with `A18` and
`A19`.

The three parallel-load flakes now have distinct explanations: `A17` and the
`creative_cyclic` counter were shared test state and are fixed; `A20` is
wall-clock dependence; `A19` remains unexplained.

## Validation Run

- exhaustive promotion-encoding agreement probe, `1024` shapes, deleted after
  use
- `cargo test -p signal-dsp-stretch -p signal-render-plane -p signal-runtime
  -p signal-hardware`: green. `190` lib tests, `11` transparent owners with `1`
  ignored, `144` render-plane, `23` hardware
- `cargo clippy --workspace --all-targets --all-features`: back to the
  pre-existing warning set, no new warnings
- `effigy qa:docs`

## Next Task

Execute `g10.038` Batch 38.3: extract one shared windowing and STFT surface for
the evidence metrics, collapse the four `transient_smear` entry points to one
plus a policy argument, and prove every retained measurement returns identical
values before the old code is removed.
