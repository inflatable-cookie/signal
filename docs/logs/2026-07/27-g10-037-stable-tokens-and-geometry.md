# 2026-07-27 g10.037 Batch 37.2 Stable Tokens And Render Geometry

Status: complete

Batch 37.2 makes one cache key describe exactly one render.

## Key Material

| field | source | change |
| --- | --- | --- |
| `schema` | crate | advanced to `signal-stretch-cache-v3` |
| `behavior` | crate | new, `signal-stretch-behavior-2026-07-27` |
| `engine` | caller | default advanced to `signal-native-stretch-v3` |
| `tier` | caller | stable token, was `Debug` |
| `offline_path` | caller | stable token, was `Debug` |
| `window_size` | caller | new |
| `analysis_hop` | caller | new |
| `chunk_max_source_frames` | caller | new |
| `chunk_overlap_frames` | caller | new |

`StretchRenderGeometry` carries window size and analysis hop and defaults to
the retained `2048/512`. `chunk_policy` reuses `StretchOfflineChunkConfig`
rather than restating it, and defaults to the production policy, so existing
callers keep their current key shape until they opt into something else.

## Behavior Version Is Crate-Owned

`SIGNAL_STRETCH_BEHAVIOR_VERSION` is deliberately **not** a field on
`StretchCacheIdentityInput`. `engine_version` is a caller-supplied `String`, so
a caller can set it to anything; it cannot be trusted to describe renderer
behavior. The behavior version is written into the canonical key by the crate
itself, next to the schema line, where no caller can get it wrong.

`cache_identity_carries_a_crate_owned_behavior_version` proves this: an input
whose `engine_version` is overridden to `someone-elses-engine` still carries
the crate's behavior line.

## Chunk Policy Keys The Render That Actually Happened

`materialize_offline_stretch_artifact_pcm_with_chunk_config` now applies the
chunk config it was handed to the identity before computing the key:

```rust
let identity_input = &identity_input.clone().with_chunk_policy(chunk_config);
```

Without this the caller could pass one policy in the identity and render with
another, which is exactly the collision Batch 37.1 measured at correlation
`-0.296620`. The key now describes the artifact that was produced, not the one
the caller declared.

## Stable Tokens

`StretchBackendTier::cache_key_token` and
`OfflineHighQualityPath::cache_key_token` are `const fn` returning explicit
strings. `cache_identity_uses_stable_key_tokens` asserts every token literally
and asserts the canonical key contains no `Debug` spelling, so a variant rename
fails the owner instead of silently rekeying every artifact in the cache.

## Owners

Five new, in `cache_identity`:

- `cache_identity_covers_render_geometry`: `2048/512`, `1024/256`, and
  `2048/256` all produce distinct keys
- `cache_identity_covers_chunk_policy`: default, `12000/2048`, and
  `12000/4096` all produce distinct keys
- `cache_identity_uses_stable_key_tokens`
- `cache_identity_carries_a_crate_owned_behavior_version`
- `cache_identity_schema_and_engine_are_v3`

`StretchCacheIdentityError::InvalidRenderGeometry` rejects a zero window or hop.

## Assertions Corrected, Not Bypassed

Four existing owners asserted the old `v2` schema and the `Debug` token
spellings. They were updated to the new contract because that contract changed;
none were weakened or deleted.

One was changed differently:
`stretch_corpus_comparison_report_formats_deterministically` pinned the literal
`engine=signal-native-stretch-v2`. It now asserts against
`SIGNAL_STRETCH_ENGINE_VERSION` instead. The owner's job is to prove the report
carries the engine version, not to pin a value that is designed to advance —
pinning it would make every future behavior bump look like a test failure.

## Validation Run

- `cargo test -p signal-dsp-stretch -p signal-render-plane -p signal-runtime`:
  green. `187` lib tests, up from `182`; `11` transparent owners with `1`
  ignored; `144` render-plane tests
- `cargo clippy --workspace --all-targets --all-features`: no new warnings
- `effigy qa:docs`

## Next Task

Execute `g10.037` Batch 37.3: implement the creative identity decision. The
Batch 37.1 recommendation is exclusion — declare creative renders explicitly
uncacheable in Contract `085` and prove the public surface offers no path that
implies caching — rather than growing a second identity surface with no caller.
