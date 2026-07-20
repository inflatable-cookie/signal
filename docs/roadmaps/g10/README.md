# g10 Milestones

Status: seed-audited renewal rejected; architecture reassessment ready
Updated: 2026-07-20

## Why this generation matters now

`g10` started as the 2026-06-11 audit-remediation generation: protect the real
audio path, remove simulated or narration-heavy surfaces, and rebuild only what
Signal needs as reusable runtime or DSP substrate.

Phase three added first-party stretch work after the operator chose a
Signal-owned time-stretch and pitch-shift engine rather than a Rubber Band
dependency. Signal can use external tools as clean-room benchmarks, but the DSP
implementation remains Signal-owned.

## Generation Runway

The 2026-07-19 consolidation reset is authoritative.

- The current OfflineHighQuality renderer, compression selector, and expansion
  selector are frozen as the competitive baseline.
- Rejected research implementations and candidate report surfaces were removed.
- `g10.029`, Contract `082`, and the detailed architecture ledger remain
  historical evidence. Batch 29.7BE is cancelled.
- `g10.030` and Contract `084` completed the successor decision. The first
  isolated end-to-end successor passed structural controls but failed
  anti-replica admission and was deleted. The event-sealed replacement then
  failed structural feasibility before implementation: its frozen impulse
  refinement is always `15` samples early. Its untouched worktree was deleted,
  the multiresolution phase-vocoder successor family is closed, and no
  candidate code entered `main`. A final non-phase-vocoder study found no
  family with a source-backed path through all whole-renderer gates. The
  program closed on the frozen production baseline.
- `g10.031` and Contract `085` open a separate offline creative-stretch path
  centered on `8x`. Batch 31.1 froze the product controls, range router, and
  comparator study without reopening the transparent successor lane. Batch
  31.2 captured the accessible comparator pack and froze a PaulX-centred
  parameter space with explicit spectral, rough, and cyclic character anchors.
  Batch 31.3 froze one complete `DiffuseSpectral` renderer brief without adding
  DSP or candidate surfaces to `main`. Batch 31.4 implemented that brief in a
  disposable worktree. Structural controls passed, but neutral `Dream` at
  `4x` raised deterministic-noise crest factor by `7.08 dB` against the frozen
  `6 dB` ceiling. The candidate was deleted before listening. Batch 31.5 kept
  the ceiling, closed independent-bin diffusion, and froze one complete
  continuous full-complex excitation replacement without changing DSP. Batch
  31.6 implemented it once. Twelve of thirteen structural controls passed, but
  common-polarity covariance missed by `0.0013287`; the candidate was deleted
  before crest admission. Batch 31.7 closed native angle subtraction and froze
  one final direct-complex relation brief. Batch 31.8 implemented it once, but
  coefficient proof exposed incompatible exact anti-phase negation/swap
  expectations and stopped before any renderer row. The candidate was deleted
  and the current diffusive owner closed. Batch 31.9 then paused the automatic
  spectral router, rejected the coherent baseline as a substitute core owner,
  and selected explicit cyclic expansion through `8x` as the narrower next
  promise. Batch 31.10 froze one complete `CyclicGrain` brief without changing
  DSP. Batch 31.11 implemented it once. All structural controls passed, but
  the first synthetic row measured `20.778` cents of pitch error against the
  `15`-cent ceiling. The candidate was deleted without correction or rerun.
  Batch 31.12 then selected one materially different correlation-aligned
  waveform family for a clean-room cyclic brief. Batch 31.13 froze its exact
  map, search, overlap, stereo, bounds, and gates without changing DSP. Batch
  31.14 implemented the similarity-aligned candidate once. Compile-only
  validation passed, but structural known-offset recovery chose source frame
  `6432` instead of the exact continuation at `6352`. The frozen coarse
  shortlist can hide an exact between-grid match from full refinement. The
  candidate was deleted without correction or rerun. Batch 31.15 docs-only
  cyclic ownership reassessment found no third materially different,
  source-backed whole-renderer path. SOLA/WSOLA variants repair the rejected
  search owner; pitch-/epoch-synchronous methods lack a full-mix linked period
  owner and `8x` evidence; other hybrids reopen closed seams or separate
  programs. Explicit `Cyclic` is closed without promotion. Batch 31.16 then
  reopened docs-only research by explicit operator decision and traced pinned
  PaulXStretch, CDP, and Potenza whole render paths. Neutral PaulXStretch uses
  long-window magnitude analysis, per-frame phase renewal, and frame crossfade
  rather than the recurrence and magnitude evolution tested by Signal's
  rejected spectral briefs. This is new source-backed evidence for one
  materially different neutral `Dream` family, `RenewalSpectral`. Batch 31.17
  froze its complete map, transform, phase renewal, linked mid/side law,
  pairwise synthesis, state bounds, comparator-calibrated gates, cleanup, and
  minimal admission without candidate DSP. Batch 31.18 implemented it once.
  Compile-only and structural admission passed, but the first neutral-`Dream`
  crest row measured `8.263162 dB` growth against the frozen `6 dB` ceiling.
  Complete independent phase renewal still left cross-bin waveform summation
  uncontrolled. The candidate was deleted before later synthetic or listening
  gates. Batch 31.19 reconciled that failure with the earlier `7.08 dB`
  `DiffuseSpectral` miss. Independent stochastic bin phase still has no
  intrinsic crest owner, and no materially different source-backed complete
  renderer remained. Batch 31.19 closed neutral `Dream`, but the operator
  superseded that closure: its PaulX crest calibration used long-form musical
  rows against a synthetic Signal stop row, and the matching PaulX synthetic
  suite had not run. Batch 31.20 then captured the matching pinned-core
  synthetics. PaulX uniform-noise crest growth is `9.932 dB` at `4x`, above
  Signal's rejected `8.263162 dB` row. The old `6 dB` ceiling was not
  target-relative. Batch 31.20 froze one complete
  `CompensatedRenewalSpectral` brief with Signal-derived overlap-statistics
  compensation and no candidate DSP. Batch 31.21 implemented it once, but
  compile-only validation failed on an unconstrained structural-test `Option`
  accumulator before the renderer executed. The candidate was deleted without
  correction or rerun. Its DSP remains untested. Batch 31.22 froze
  `VarianceCompensatedRenewalSpectral` as fresh complete authority and made
  compile completion a construction receipt before one-shot structural
  admission. Batch 31.23 compiled and passed seven structural tests. Its
  synthetic command returned green, but pre-listening review found missing
  impulse-train crest, secondary-region, exact-lag autocorrelation, and full
  discontinuity assertions. The receipt is invalid. The unopened listening
  pack and isolated candidate were deleted without repair or rerun. The
  topology remains untested. Batch 31.24 retained it under fresh identity
  `AuditedVarianceCompensatedRenewalSpectral` and froze `22` compile-linked
  gate owners, exact comparator measurements, allocation accounting, and one
  clean candidate boundary. Batch 31.25 passed construction `1/1`, structural
  `13/13`, synthetic `9/9`, and concealed mono listening as `15/15` ties from
  immutable checkpoint `97ee7056`. Linked-stereo review then rejected it: a
  source at `-0.4516 dB` right-minus-left became `+3.3660 dB` at neutral
  `space` because first-sample component orientation discarded the source
  mid/side relationship. The candidate was deleted. Batch 31.26 froze
  `SourceRelativeRenewalSpectral`: the passed mono renderer remains, while
  native left/right complex analysis and one explicit interchannel relation
  law own stereo. Batch 31.27 compiled and passed construction `1/1`, then
  failed structural admission at `14/15` because its frozen `mix64(1)` vector
  was transposed. The candidate was deleted without repair or rerun. Batch
  31.28 reproduced every counter vector independently and froze fresh verified
  authority. Batch 31.29 then passed compile, construction `1/1`, and
  structural `15/15` from immutable checkpoint `d94612dd`. Synthetic admission
  finished `7/9`: one `16x` row split into two replica regions, and two `4x`
  pitch rows measured about `10.96` cents outside their PaulX-relative
  ceilings. The candidate was deleted before listening. Batch 31.30 found that
  neither that brief nor Batch 31.25's passing mono brief froze the candidate
  seed. It withdrew the unsupported range diagnosis and froze
  `SeedAuditedSourceRelativeRenewalSpectral`. Batch 31.31 passed compile,
  construction `1/1`, and structural `15/15`, then failed `Y02` on the `8x`
  chord at `13.351828347` cents against an `11.331375778`-cent ceiling. `Y04`
  passed at all ratios. The candidate was deleted before listening. Two
  checkpoints now fail the same tonal-pitch class, so Batch 31.32 architecture
  reassessment is ready. Other characters, routing, product exposure, and
  rejected branches remain closed or paused.
- The 2026-07-20 lifecycle reconciliation closes stale `g10.001` and
  `g10.003` active markers. It also records that Signal's `g10.017` capture and
  live-monitor implementation landed; that roadmap is paused only on explicit
  hardware alignment and consumer workflow evidence. No feature batch became
  ready through this correction.
- `g10.028` RealtimePreview source-fill and all render-plane integration remain
  paused.

Do not start Loophole or Chorus planning from Signal internals.

## Milestone Map

- `g10.001` `complete`
  - audit adoption and generation open completed
- `g10.002` `complete`
  - render-plane declick and playback correctness
- `g10.003` `complete`
  - output stream hardening, cpal enumeration, and legacy CoreAudio retirement
- `g10.004` `complete`
  - hosting-domain demolition
- `g10.005` `complete`
  - runtime rescope to honest control plane
- `g10.006` `complete`
  - analysis pruning and measurement correctness
- `g10.007` `complete`
  - plugin-domain pruning to real foundations
- `g10.008` `complete`
  - DSP corrections and polyphase resampling
- `g10.009` `complete`
  - workspace consolidation and truthful front doors
- `g10.010` `complete`
  - graph-shaped plans and mixer realization
- `g10.011` `complete`
  - stable node identity and state handoff
- `g10.012` `complete`
  - parameter fast path and automation playback
- `g10.013` `complete`
  - DSP kit: biquads, pan law, limiter, denormals
- `g10.014` `done`
  - RT observability, metering, and callback health
- `g10.015` `complete`
  - WYSIWYG bounce on the render plane
- `g10.016` `complete`
  - output-time honesty and device lifecycle
- `g10.017` `paused`
  - recording capture and live monitoring landed; hardware alignment evidence
    remains an explicit operator gate
- `g10.018` `complete`
  - disk-streaming clip sources
- `g10.019` `complete`
  - transport regions, loop, click, count-in
- `g10.020` `complete`
  - Signal runtime endgame thin control library
- `g10.021` `complete`
  - stretch real corpus and benchmark evidence
- `g10.022` `paused`
  - OfflineHighQuality DSP depth; low-risk sustained candidates evidence-complete
- `g10.023` `paused`
  - stretch offline artifact scale and format depth
- `g10.024` `paused`
  - RealtimePreview stretch tier
- `g10.025` `deferred`
  - stretch product workflow contract checkpoint
- `g10.026` `complete`
  - RealtimePreview callback-safe state
- `g10.027` `complete`
  - RealtimePreview source-projected callback
- `g10.028` `paused`
  - RealtimePreview source fill contract
- `g10.029` `superseded`
  - historical correctness, listening, and rejected-successor ledger
- `g10.030` `complete`
  - stretch consolidated; candidate families closed; frozen baseline retained
- `g10.031` `active`
  - PaulX-like neutral `Dream` remains the product goal; Batch 31.31 rejected
    the seed-audited renewal candidate on repeated tonal-pitch failure; Batch
    31.32 architecture reassessment is ready and explicit `Cyclic` stays closed

## Stretch Boundary

Current status:

- `Repitch`: implemented realtime-safe varispeed.
- `RealtimePreview`: bounded prototype with callback-state and allocation proof;
  direct callback support remains closed pending `g10.028`.
- `OfflineHighQuality`: the retained `2048/512` phase-vocoder baseline and two
  promoted short-window selectors are stable and fully regression-tested.
- Comparator evidence remains Signal-versus-external objective rows plus the
  five-family, three-ratio long-form blind listening pack.
- Rejected hybrid, H/R/P, phase-gradient, frequency-adaptive, tail, timeline,
  peak, and local phase variants are not active code or planning authority.

The retained OfflineHighQuality baseline is the only active renderer. Contract
`084` and `g10.030` are closed without promotion. A new successor requires the
whole-system evidence listed in the non-phase-vocoder feasibility decision.

The separate `CreativeStretch` path remains unimplemented. Its automatic
spectral route is paused after three rejected and deleted candidates. The first
explicit cyclic candidate is also rejected and deleted after structural
admission and a first-row synthetic pitch miss. Correlation-aligned waveform
overlap also failed structural admission and was deleted. `RenewalSpectral`
passed structural admission but failed its first crest row and was deleted.
Matching PaulX synthetics later showed that the old absolute crest ceiling did
not describe the preferred reference. The complete compensated-renewal brief
was implemented once, but its compile-only validation failed before renderer
execution and the candidate was deleted. The fresh variance-compensated
implementation later produced an invalid synthetic receipt because required
assertions were absent. It was deleted before listening. The compensated-
renewal topology now has valid structural, synthetic, and mono listening
evidence. `AuditedVarianceCompensatedRenewalSpectral` was rejected and deleted
after its first-sample mid/side orientation law inverted source-relative
channel balance. Batch 31.26 froze the complete
`SourceRelativeRenewalSpectral` successor without adding DSP. Its native
left/right analysis preserves channel magnitudes and owns the interchannel
phase relation directly. Its first isolated checkpoint passed compile and
construction, then stopped at structural exact-vector proof because the
frozen `mix64(1)` assertion transposed the normative result. No synthetic or
listening result exists. The candidate was deleted.
Batch 31.28 independently reproduced the complete counter table in Python and
Ruby and froze fresh verified authority. Batch 31.29 passed construction and
all `15` structural owners, then failed two synthetic owners. Batch 31.30
corrected unfrozen seed authority. Batch 31.31 passed construction and all
structural owners under the audited seed, cleared the replica row, then failed
the `8x` chord pitch row. The isolated candidate was deleted before listening.
Creative stretch still has no renderer, public API, harness surface, or product
route on `main`.

## Next Task

Run `g10.031` Batch 31.32 only. Reassess the renewal family after repeated
tonal-pitch rejection. Either identify one materially different source-backed
complete renderer or close the family. Keep `g10.028`, routing, product
exposure, cross-repo work, and candidate DSP on `main` paused. Do not push.
