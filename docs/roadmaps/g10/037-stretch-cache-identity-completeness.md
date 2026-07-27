# 037 - Stretch Cache Identity Completeness

Status: complete; creative renders declared uncacheable
Owner: dsp
Created: 2026-07-27
Updated: 2026-07-27
Depends on: `g10.036`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
Vision tags: `DSP`, `STRETCH`, `CACHE`, `IDENTITY`

## Problem

`StretchCacheIdentityInput` claims to identify one cacheable stretch artifact.
Audit finding `A5` shows it does not.

Missing render geometry. The identity hashes engine version, tier, offline
path, source content hash, channel layout, ratio curve, pitch curve, warp
markers, and projection epoch. It does not hash window size or analysis hop.
`OfflineHighQualityStretcher::with_window` is public, so two renders at
`2048/512` and `1024/256` produce different audio under one identical cache
key. A cache hit then serves the wrong render. Contract `046` lists the same
incomplete field set, so this is a contract gap, not only a code gap.

Unstable key tokens. `canonical_key` writes `format!("{:?}", tier)` and
`format!("{:?}", offline_path)`. Debug output is not a stability contract:
renaming a variant silently changes every key, and reordering or reusing a name
can alias two different renders onto one key. A cache identity must not depend
on a derived formatter.

No creative identity at all. Public `render_creative_stretch` has character,
`space`, `cycle`, target frames, and `CREATIVE_STRETCH_ENGINE_VERSION`, and
none of it can be expressed through the current identity type. Creative renders
are currently uncacheable, which is honest today but undeclared.

`g10.036` makes this urgent rather than theoretical: once the ratio envelope is
enforced and the dynamic-ratio segment law changes, renders produced before and
after the correction can collide on one key.

## Generation Runway

This lane advances the `g10` runway from *correct renders* to *correctly
identified renders*, which every later artifact, freeze, and export reuse
depends on.

The visible runway is:

1. identity gap audit and contract amendment
2. stable tokens, render geometry, and schema version advance
3. creative identity decision — cover it or declare it out of scope
4. closeout

The next planning checkpoint is Batch 37.3, where covering creative renders
would extend Contract `085` rather than only amending `046`.

## Goals

- [ ] make one cache key identify exactly one render
- [ ] remove every derived-formatter dependency from key material
- [ ] advance the schema version so pre-correction artifacts cannot be reused
- [ ] state the creative-stretch cache position explicitly
- [ ] align Contract `046`'s cache identity field list with the shipped type

## Non-Goals

- no cache storage, eviction, or placement work
- no artifact format or on-disk layout change
- no renderer or DSP change
- no routing, runtime DTO, UI, Loophole, or Chorus work

## Execution Plan

### Batch 37.1 - Identity Gap Audit And Contract Amendment

Status: complete

Documentation only. No crate source changed.

- [x] enumerate every input that changes rendered output and check it against
  the current identity fields
- [x] amend Contract `046` so the cache identity field list includes render
  geometry and any correction-era behavior version
- [x] freeze the rule that key material uses explicit stable tokens, never
  derived formatting
- [x] decide the schema version advance and what happens to artifacts written
  under `signal-stretch-cache-v2`
- [x] decide the hash width question: whether the 64-bit FNV-1a stable hash
  stays sufficient for the intended artifact population, and record the reason
- [x] change documentation only

Result:

- nine of nineteen output-affecting inputs are covered today. Missing: STFT
  window size, analysis hop, chunk policy, and five renderer constants that one
  behavior version can carry between them. Creative renders have no identity at
  all
- the chunk-policy gap was measured, not argued. One identity,
  `stable_hash=2e0b01234f55947c`, rendered as one chunk and as eight over the
  same source at ratio `1.25`, produced `120000` frames both times with
  correlation `-0.296620` and peak sample difference `0.5428`
- live hazard found: `SIGNAL_STRETCH_ENGINE_VERSION` is still
  `signal-native-stretch-v2` although `g10.036` changed rendered output above
  ratio `3.0` and for every dynamic curve. Pre- and post-correction artifacts
  are indistinguishable by key, so any cache populated before the correction
  now serves stale audio. Batch 37.2 carries the bump
- hash width decision: keep the 64-bit FNV-1a digest and declare the canonical
  key authoritative for equality. The hash is a bucketing aid, not a
  collision-resistant identity, and consumers must compare the canonical key on
  a candidate hit
- schema advances to `signal-stretch-cache-v3` and the engine version to
  `signal-native-stretch-v3`. Every `v2` artifact is invalid with no migration
  path, because a `v2` key cannot describe which render it holds
- incidental: `materialize_offline_stretch_artifact_pcm_with_chunk_config` is
  `pub` and documented as an integration entry point but never re-exported, so
  no external caller can reach it. Recorded as a `g10.038` input

### Batch 37.2 - Stable Tokens And Render Geometry

Status: complete

- [x] give every enum in key material an explicit stable token independent of
  `Debug`
- [x] add render geometry and chunk policy to the identity input and canonical
  key
- [x] add the behavior version and advance it for the `g10.036` correction
- [x] advance the schema to `signal-stretch-cache-v3` and the engine version to
  `signal-native-stretch-v3`, and prove old and new keys differ
- [x] add owners proving that two renders differing only in geometry or chunk
  policy produce different keys, and that a variant rename cannot change a key

Result:

- `StretchRenderGeometry` carries window size and analysis hop, defaulting to
  the retained `2048/512`; `chunk_policy` reuses `StretchOfflineChunkConfig`
  rather than restating it, defaulting to the production policy
- `SIGNAL_STRETCH_BEHAVIOR_VERSION` is crate-owned and deliberately not a field
  on the input. `engine_version` is a caller-supplied `String`, so it cannot be
  trusted to describe renderer behavior; the behavior line is written into the
  canonical key by the crate, where no caller can get it wrong
- `materialize_offline_stretch_artifact_pcm_with_chunk_config` now applies the
  chunk config it was handed to the identity before computing the key, so the
  key describes the artifact produced rather than the one declared
- `cache_key_token` is `const fn` on both key enums, and the token owner
  asserts every literal plus the absence of any `Debug` spelling
- five new owners; `InvalidRenderGeometry` rejects a zero window or hop
- four existing owners were updated to the new contract, and one that pinned
  the literal engine version now asserts against the constant instead, so a
  future behavior bump does not read as a test failure

### Batch 37.3 - Creative Identity Decision

Status: complete

- [x] implement the Batch 37.1 decision: either extend identity to cover
  character, `space`, `cycle`, target frames, and creative engine version, or
  declare creative renders explicitly uncacheable in Contract `085`
- [x] if excluded, prove the public surface offers no path that implies caching

Result: exclusion. Contract `085` gains a dated section declaring creative
renders uncacheable, with the reopening condition of a named consumer plus the
same enumeration the transparent identity received.

Three owners prove the surface offers no cacheable path: a scan of the
production half of `creative.rs` for cache, receipt, and artifact identifiers;
an exhaustive match over `StretchBackendTier` so adding a creative tier breaks
compilation; and a proof that `render_creative_stretch` returns samples and no
handle a caller could key against.

### Batch 37.4 - Closeout

Status: complete

- [x] run `effigy validate` and the full crate suite
- [x] update Contract `046`, Contract `085`, and the `g10` front doors
- [x] name the next ready batch in `g10.038`

Every input that changes a transparent render now changes the key. Creative is
out of scope by contract rather than by omission.

The closeout run also exposed a second `A17`-class defect:
`creative_cyclic::synthesis` counted output allocations in a process-global
`AtomicUsize`, so a concurrent Cyclic render on another thread was counted
against whichever test was measuring. It is now a thread-local `Cell`, matching
the dream gate. Two such defects in two modules, plus the untriaged `A19`
suspect, make a sweep for process-global test state a `g10.038` candidate.

## Acceptance Criteria

- [ ] no two renders differing in any output-affecting input share a key
- [ ] no key material derives from `Debug` or any other derived formatter
- [ ] the schema version advance is proven to separate pre- and
  post-correction artifacts
- [ ] Contract `046`'s field list matches the shipped identity type exactly
- [ ] the creative cache position is explicit in contract text
- [ ] full crate suite passes

## Risks and Mitigations

- Risk: adding fields invalidates artifacts a consumer already depends on.
  Mitigation: Batch 37.1 decides the invalidation policy before code changes;
  `signal-render-plane` is the only current consumer and is in-repo.
- Risk: the identity grows into a general serialization surface. Mitigation:
  fields are admitted only when they provably change rendered output.
- Risk: creative coverage pulls `g10.037` into `g10.031` scope. Mitigation:
  Batch 37.3 may close the question by exclusion, which is a valid outcome.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the enumerated input-to-field table from Batch 37.1
- [ ] key-difference proofs for geometry, tokens, and schema advance
- [ ] commands actually run

## Next Task

Execute `g10.038` Batch 38.1: inventory all public items with consumer,
test-only, or unused status, mark each retained, removed, or deferred, confirm
the RealtimePreview surface is deferred to `g10.040` in full, and freeze
byte-exactness as the acceptance proof for the later batches. Documentation
only.
