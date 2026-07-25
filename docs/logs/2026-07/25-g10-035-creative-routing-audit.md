# g10.035 Creative Routing Audit

Status: Batch 35.1 complete; Batch 35.2 ready

The current owners are not interchangeable tiers:

- Transparent is the frozen source-readable `OfflineHighQuality` owner
- Dream is continuous over exact `4N..=16N`
- Cyclic is continuous over exact `2N..=8N`

Batch 35.1 selects one future opt-in Automatic intent:

- Transparent through exact `4N`
- Transparent/Dream transition over exact `4N..=8N`
- neutral Dream from exact `8N` through `16N`

The complete Automatic envelope is exact `0.5N..=16N`. At `4N`, it must be
byte-exact Transparent. At `8N`, it must be byte-exact Dream with admitted
neutral defaults.

Cyclic stays explicit. Its repetition and cycle duration are musical intent,
not an automatic quality decision. Explicit Transparent, Dream, and Cyclic
remain available. Automatic exposes only exact duration; no transition weight
or renderer control reaches a consumer.

This is a product and architecture selection only. No DSP, candidate, public
API, cache, artifact, dynamic ratio, runtime, UI, Loophole, or Chorus surface
changed.

Validation passed: `git diff --check`, `effigy qa:docs`,
`effigy qa:northstar`, `effigy health`, and `effigy validate`. `effigy doctor`
reports only the pre-existing `59` god-file and `5` attention-marker findings.

## Next Task

Execute `g10.035` Batch 35.2 only. Freeze one complete
`ExactTargetTransparentDreamRouter` brief with exact target, map, transition,
level, boundary, stereo, identity, memory, evidence, rejection, cleanup, and
minimal-admission ownership. Stop before implementation.
