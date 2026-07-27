# 2026-07-27 g10.037 Batches 37.3 and 37.4 Creative Cache Decision And Closeout

Status: complete

## Decision

Creative renders are uncacheable. Contract `085` records it.

`render_creative_stretch` is a whole-buffer call that returns samples. It
returns no identity, no receipt, and no artifact plan. A caller that wants to
reuse a creative render owns that decision and its invalidation; Signal does
not offer a key for it.

## Evidence For Exclusion

Creative cannot be described by the existing identity.
`StretchCacheIdentityInput` carries a `StretchBackendTier`, and the three tiers
are `Repitch`, `RealtimePreview`, and `OfflineHighQuality`. None names a
creative render, and character, `space`, `cycle`, target frames, and the
admitted seed have no fields.

Creative has no consumer that would use a key. `render_creative_stretch` is
re-exported from `signal-dsp-stretch` and imported by nothing:
`signal-runtime` re-exports the transparent identity and promotion types and no
creative type, and `signal-render-plane` never mentions it.

The creative source carries no cache vocabulary at all. A scan for
`CacheIdentity`, `cache_key`, `canonical_key`, `stable_hash`,
`PromotionReceipt`, `OfflineStretchArtifact`, and `StretchOfflineChunk` finds
none of them in the production half of `creative.rs`.

## Why Not Cover It

A second identity surface would have no caller. Freezing a schema against zero
usage means the first real consumer almost certainly needs a different shape.

More importantly, coverage without enumeration would repeat the defect this
lane was opened to fix. The creative renderers are deterministic for a fixed
input and seed, so a caller *could* key them — but Signal has not measured
which of their inputs are output-affecting with the rigour Batch 37.1 applied
to the transparent identity. Shipping a creative key on that basis would be the
same mistake as the `v2` key that omitted geometry and chunk policy.

Contract `085` records the reopening condition: a named consumer plus the same
enumeration, with measured collisions for anything omitted.

## Owners

Three, in `creative`:

- `creative_surface_carries_no_cache_or_artifact_vocabulary` scans the
  production half of the file for cache, receipt, and artifact identifiers. It
  splits at the test marker because the owner names those identifiers itself
- `no_stretch_tier_describes_a_creative_render` matches exhaustively over
  `StretchBackendTier`, so adding a creative tier breaks compilation and this
  owner, and asserts no tier token contains a creative word
- `creative_render_returns_samples_without_a_cache_handle` proves the public
  entry point hands back samples and nothing a caller could cache against

## Batch 37.4 Closeout

Contract `046`'s input table now cross-references the Contract `085` decision,
so the two contracts agree on the same row.

Final identity coverage:

| input | covered |
| --- | --- |
| engine version, tier, offline path | yes, tier and path as stable tokens |
| source content hash, channel layout | yes |
| ratio curve, pitch curve, warp markers | yes |
| projection epoch | yes |
| STFT window size, analysis hop | yes, new in `v3` |
| chunk policy | yes, new in `v3` |
| renderer constants and laws | yes, through the crate-owned behavior version |
| creative character, `space`, `cycle`, seed | out of scope by contract |

Every input that changes a transparent render now changes the key.

## Second A17-Class Defect Found And Fixed

The closeout run failed once on
`creative_cyclic::tests::exact_sixteen_rejects_before_output_allocation`. It
passed three times in isolation.

`creative_cyclic::synthesis` counted output allocations in a process-global
`AtomicUsize`. A test resets it and asserts zero, but any concurrent Cyclic
render on another thread increments the same counter. This is the same defect
class as `A17`, in a second module: the `g10.036` Batch 36.2 repair fixed the
dream allocation gate and did not sweep for others.

The counter is now a const-initialized thread-local `Cell`, matching the dream
gate. Adding this lane's creative owners changed test scheduling enough to
expose it, which is the only reason it surfaced now rather than at some future
unrelated change.

`A19`, the `signal-plugin-bridge` shared-memory test, remains untriaged and is
plausibly the same class again. A sweep for process-global test state is a
`g10.038` candidate.

## Validation Run

- `cargo test -p signal-dsp-stretch`: `190` lib tests, `11` transparent owners
  with `1` ignored, green
- `cargo test -p signal-render-plane -p signal-runtime`: green
- `cargo clippy -p signal-dsp-stretch --all-targets --all-features`: no new
  warnings; the nine pre-existing remain `g10.038` work
- `effigy validate`, `effigy qa:docs`

## Next Task

Execute `g10.038` Batch 38.1: inventory all public items with consumer,
test-only, or unused status, mark each retained, removed, or deferred, confirm
the RealtimePreview surface is deferred to `g10.040` in full, and freeze
byte-exactness as the acceptance proof for the later batches. Documentation
only.

Three `g10.038` inputs are already recorded: the nine pre-existing clippy
warnings; `materialize_offline_stretch_artifact_pcm_with_chunk_config` being
`pub` and documented as an integration entry point while never being
re-exported; and a sweep for process-global test state, after two `A17`-class
defects in two modules and one untriaged suspect in `A19`.
