# 2026-07-27 g10.037 Batch 37.1 Identity Gap Audit And Contract Amendment

Status: complete

Documentation only. No crate source changed. One temporary in-crate probe was
added to `signal-render-plane`, run, and removed; it left no tracked change.

## Enumeration

Every input that changes rendered output, against the current identity fields:

| input | in the identity today |
| --- | --- |
| engine version | yes |
| tier | yes |
| offline renderer path | yes |
| source content hash | yes |
| channel layout | yes |
| ratio curve | yes |
| pitch curve | yes |
| warp markers | yes |
| projection epoch | yes |
| STFT window size | no |
| analysis hop | no |
| offline chunk policy | no |
| overlap coverage fraction | no |
| dynamic-ratio segment minimum | no |
| segment seam smoothing length | no |
| transient detector thresholds | no |
| short-window selector gates | no |
| pitch-shift resample quality | no |
| creative character, `space`, `cycle`, seed | no identity exists |

The last six renderer constants do not need individual fields. They need one
behavior version that advances whenever any of them changes.

## Measured Collision

The chunk-policy gap was measured rather than argued. One identity input,
`stable_hash=2e0b01234f55947c`, materialized twice over the same `96000`-frame
source at ratio `1.25`, once as a single chunk and once as eight:

| measurement | value |
| --- | --- |
| output frames | `120000` both |
| correlation between renders | `-0.296620` |
| peak sample difference | `0.5428` |

Two unrelated renders under one key. Chunk boundaries move where segment
renders restart phase, and `g10.036` already measured phase restarts producing
near-uncorrelated output.

## Live Hazard

`SIGNAL_STRETCH_ENGINE_VERSION` is still `signal-native-stretch-v2`. The
2026-07-27 defect correction changed rendered output at every ratio above `3.0`
and for every dynamic-ratio curve, and did not advance it. Artifacts written
before and after that correction are indistinguishable by key, so any cache
populated before it now serves stale audio.

This is a consequence of `g10.036` that `g10.036` did not catch, because the
correction batches proved byte-exactness for the unaffected range and
re-baselined the affected hashes without asking what the cache key promises.
Batch 37.2 carries the bump.

## Decisions

Field list. Identity gains render geometry, chunk policy, and a behavior
version. Contract `046`'s promotion evidence list is amended to match, since it
had frozen the incomplete set.

Stable tokens. Key material uses explicit tokens. `Debug` output is not a
stability contract: a rename silently changes every key and a reused name
silently aliases two renders.

Behavior version discipline. It must advance in the same change that alters
renderer output. A correction that changes audio without advancing it is
incomplete.

Hash width. The 64-bit FNV-1a stable hash is retained, and the canonical key is
declared authoritative for equality. FNV-1a is not collision resistant and
never claimed to be; the hash is a bucketing aid. A consumer that treats a hash
match as identity relies on a guarantee this contract does not make and must
compare the canonical key on a candidate hit. This is cheaper and more honest
than widening the digest.

Schema advance. `signal-stretch-cache-v2` becomes `v3` and the engine version
becomes `signal-native-stretch-v3`. Every `v2` artifact is invalid: keyed
without geometry or chunk policy, and produced by a renderer that predates the
correction. No migration is possible, because a `v2` key cannot describe which
render it holds.

Creative identity. Deferred to Batch 37.3 as planned. The recommendation is
exclusion: `render_creative_stretch` is an explicit whole-buffer call with no
cache route, artifact path, or consumer, so Contract `085` should declare
creative renders uncacheable rather than grow a second identity surface with no
caller.

## Incidental Finding

`materialize_offline_stretch_artifact_pcm_with_chunk_config` is `pub` and
documented as "the long-media test and integration entry point", but
`signal-render-plane` never re-exports it, so no external caller can reach it.
Recorded as a `g10.038` surface-inventory input, not fixed here.

## Validation Run

- chunk-policy collision probe, in-crate, removed after use
- `cargo build -p signal-render-plane` after probe removal
- `effigy qa:docs`

## Next Task

Execute `g10.037` Batch 37.2: give every enum in key material an explicit stable
token, add render geometry and chunk policy to the identity input and canonical
key, add the behavior version, advance the schema to `signal-stretch-cache-v3`
and the engine version to `signal-native-stretch-v3`, and prove that renders
differing only in geometry or chunk policy produce different keys.
