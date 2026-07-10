# g10 Milestones

Status: active
Updated: 2026-07-10

## Why this generation matters now

`g10` started as the 2026-06-11 audit-remediation generation: protect the real
audio path, remove simulated or narration-heavy surfaces, and rebuild only what
Signal needs as reusable runtime or DSP substrate.

Phase three added first-party stretch work after the operator chose a
Signal-owned time-stretch and pitch-shift engine rather than a Rubber Band
dependency. Signal can use external tools as clean-room benchmarks, but the DSP
implementation remains Signal-owned.

## Generation Runway

`g10` now has three stretch gates instead of another ready coding lane:

- OfflineHighQuality boundary coverage and absolute full-render measurement now
  pass the bounded Signal/Rubber Band pack. Synthetic-only promotion is closed.
  Aggregate operator findings from the 15-pair pack now identify transient
  crest spikes and long-stretch grain, while event timing is effectively tied.
  Objective follow-up classifies the grain as excess fast spectral movement,
  not confirmed added sideband energy. Broad vocal-envelope evidence shows no
  current Signal formant failure, but exterior-step evidence isolates a
  fixed-ratio tail discontinuity. Source, additive-zero, multiplicative-zero,
  and centroid-selected endpoint controls failed objective reach, listening, or
  cross-source prediction. Fixed-envelope work is closed and production stays
  unchanged pending a different algorithm class. Rubber Band-class promotion
  remains blocked on independent stereo/row-level completion and the structural
  hybrid checkpoint.
- Sustained/polyphonic long-window candidates produced useful evidence but no
  production route. The structural hybrid is now frozen as short transient,
  current mixed, and long tonal ownership with continuous state and shared
  stereo decisions. Its first fixed-ratio mono render is rejected: conservative
  transitions applied only `56/2024` ownership spans, left the `L001` crest
  unchanged, regressed `1.25x` static residual, and passed only `50/60`
  tonal/combined rows. Bounded lag reassessment then rejected branch alignment:
  recoverable spans needed `152.383` mean absolute lag and disagreed by
  `210.465` frames between entry and exit. Contract `082` required one synthesis
  timeline and tested transient-local time mapping before adaptive resolution.
  That transient-local timeline proof is also rejected: sparse protected onsets
  left `1891` dense conflicts, moved mean event placement by `+4.942263`
  frames, and passed only `9/60` combined rows. Adaptive resolution and linked
  stereo remained closed for transient-ownership reassessment. That decision is
  now frozen: Batch 29.6C keeps the global time map fixed and reinitializes only
  group-delay-selected transient peak regions near the analysis-window centre.
  Explicit transient/residual separation is deferred behind its own
  perfect-reconstruction and recombination contract. Adaptive resolution and
  linked stereo remained closed for the peak proof. That proof is now rejected:
  anchored `L001` improved `0.040942 dB`, measurable-row timing worsened
  `16.851522` frames, tonal residual regressed in `21/60` rows, and the combined
  gate passed `12/60`. Contract `082` froze the final untested structural
  family: iterative harmonic/residual/percussive separation. Batch 29.6D now
  passes exact additive source reconstruction and the three `12 dB` synthetic
  ownership gates without tuning. The report-only
  long-PV/residual-PV/short-OLA mono candidate then passed its anchored crest
  target but failed timing, integrity, replica, static-spectrum, and combined
  gates. Batch 29.6E is rejected and linked stereo remains closed.
- Offline artifacts and RealtimePreview have bounded contracts and prototype
  paths, but callback-safe preview integration and fully streaming artifact
  output remain gated until their owning source-fill/cache contracts exist.

Do not start Loophole integration planning in Chorus from Signal internals.
`g10.025` is the Signal product-workflow checkpoint and remains deferred until
a product workflow consumes the Signal-owned stretch contract.

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
- `g10.029` `active`
  - stretch correctness and listening gate

## Stretch Boundary

Current stretch status:

- `Repitch`: implemented as the render-plane realtime-safe varispeed path.
- `RealtimePreview`: prototype and metrics landed; direct callback processing
  remains unsupported for render-plane routing. Callback-local DSP now has
  no-allocation, linked-stereo, ratio-scheduling, source-projection reporting,
  and synthetic tempo-ramp seam evidence. `g10.028` owns the missing
  source-fill and underrun contract before callback streaming can open.
- `OfflineHighQuality`: materialized for default-path artifacts with chunked
  output and cache receipts, but the DSP path is classified as a prototype until
  `g10.029` closes boundary correctness, absolute measurement, and listening
  evidence.

Remaining stretch work is not blocked by Chorus. Chorus only becomes relevant
when Loophole integration needs a product workflow plan.

## Next Task

Start Batch 29.6F's report-only fixed-resolution full phase-gradient kernel
proof. Do not render the corpus, tune the rejected H/R/P mechanism, or open
linked stereo. Production, cache identity, pitch/dynamic routing,
RealtimePreview, and product integration remain closed.
