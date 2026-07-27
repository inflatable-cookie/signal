# 037 - Stretch Cache Identity Completeness

Status: planned; blocked on `g10.036`
Owner: dsp
Created: 2026-07-27
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

Status: blocked on `g10.036` Batch 36.5

Documentation only.

- [ ] enumerate every input that changes rendered output and check it against
  the current identity fields
- [ ] amend Contract `046` so the cache identity field list includes render
  geometry and any correction-era behavior version
- [ ] freeze the rule that key material uses explicit stable tokens, never
  derived formatting
- [ ] decide the schema version advance and what happens to artifacts written
  under `signal-stretch-cache-v2`
- [ ] decide the hash width question: whether the 64-bit FNV-1a stable hash
  stays sufficient for the intended artifact population, and record the reason
- [ ] change documentation only

### Batch 37.2 - Stable Tokens And Render Geometry

Status: blocked on Batch 37.1

- [ ] give every enum in key material an explicit stable token independent of
  `Debug`
- [ ] add render geometry to the identity input and canonical key
- [ ] advance the schema version and prove old and new keys differ
- [ ] add owners proving that two renders differing only in geometry produce
  different keys, and that a variant rename cannot change a key

### Batch 37.3 - Creative Identity Decision

Status: blocked on Batch 37.2

- [ ] implement the Batch 37.1 decision: either extend identity to cover
  character, `space`, `cycle`, target frames, and creative engine version, or
  declare creative renders explicitly uncacheable in Contract `085`
- [ ] if covered, prove every creative control changes the key
- [ ] if excluded, prove the public surface offers no path that implies caching

### Batch 37.4 - Closeout

Status: blocked on Batch 37.3

- [ ] run `effigy validate` and the full crate suite
- [ ] update Contract `046`, Contract `085` if touched, and the `g10` front
  doors
- [ ] name the next ready batch in `g10.038`

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

Blocked. Open Batch 37.1 after `g10.036` Batch 36.5 closes, because the
correction-era behavior version it must record does not exist until then.
