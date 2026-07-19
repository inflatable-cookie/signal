# g10 Milestones

Status: active
Updated: 2026-07-19

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
  31.2 has captured and level-matched the accessible four-way comparator pack;
  concealed operator listening remains open.
- `g10.028` RealtimePreview source-fill and all render-plane integration remain
  paused.

Do not start Loophole or Chorus planning from Signal internals.

## Milestone Map

- `g10.001` `active`
  - audit adoption and generation open
- `g10.002` `complete`
  - render-plane declick and playback correctness
- `g10.003` `active`
  - output stream hardening and real device enumeration
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
- `g10.017` `in-progress`
  - recording v1 input capture to timeline; monitoring deferred
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
  - creative architecture frozen; accessible comparator captured; listening active

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

The separate `CreativeStretch` path is architecture-only. It targets
intentional `4x`, `8x`, and `16x` expansion under Contract `085`; it has no
renderer, public API, harness surface, or product route on `main`.

## Next Task

Finish the concealed `g10.031` Batch 31.2 comparator listening, then freeze one
`DiffuseSpectral` target and its rejection thresholds. Keep `g10.028` and
render-plane integration paused.
