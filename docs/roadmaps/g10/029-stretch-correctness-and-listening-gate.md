# 029 - Stretch Correctness And Listening Gate

Status: active
Owner: dsp
Created: 2026-07-09
Depends on: g10.021, g10.022, g10.024, g10.027
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
Vision tags: `DSP`, `STRETCH`, `QUALITY`

## Problem

The stretch program has useful prototype DSP, callback-safety work, corpus
tooling, and Rubber Band comparison renders. Its execution order moved ahead of
the evidence. Offline analysis can omit boundary content and satisfy length by
appending zeros; callback source projection is not coupled to actual kernel
consumption; promotion is relative to the draft backend; real-source listening
slots remain unfilled.

Source-fill work cannot repair those foundations. Correctness and evidence must
be trustworthy before the callback contract or a structural hybrid widens.

## Goals

- [ ] preserve source content through both contractual output endpoints
- [ ] measure full-render dropout, endpoint energy, peak growth, CPU, latency,
  and memory
- [ ] replace draft-relative product promotion with absolute and
  comparator-backed gates
- [ ] produce a bounded blind-listening pack with completed operator notes
- [ ] freeze the requirements for the next structural hybrid design

## Non-Goals

- [ ] no render-plane or product integration
- [ ] no RealtimePreview source-fill implementation
- [ ] no claim of Elastique or Rubber Band parity from objective proxies alone
- [ ] no more scalar selector or one-parameter long-window probes

## Execution Plan

### Batch 29.1 - Boundary Correctness

- [x] pad offline STFT analysis so the first and last source samples contribute
  to the rendered interval
- [x] crop the padded render back to the exact sample-domain length contract
- [x] add content-aware head/tail tests for compression and expansion
- [x] keep identity behavior bit-exact

### Batch 29.2 - Full-Render Measurement

- [x] add reusable endpoint-energy, added-silence, and peak-growth metrics
- [x] wire full-render integrity fields into comparator quality rows for both
  Signal and the external render
- [x] make comparator reports inspect the full render as well as aligned excerpts
- [ ] measure CPU realtime factor and peak working memory for promoted paths
- [ ] add absolute acceptance limits separate from draft comparisons

### Batch 29.3 - Promotion And Listening

- [ ] prevent synthetic-only receipts from opening product-quality promotion
- [ ] generate source/Signal/Rubber Band level-matched blind-listening renders
- [ ] record operator findings for percussion, bass, vocals, sustains, and full mix
- [ ] classify failures by transient, tonal, stereo, formant, and boundary behavior

### Batch 29.4 - Structural Hybrid Checkpoint

- [ ] define transient/tonal classification and multiresolution window ownership
- [ ] define shared stereo peak/phase decisions and formant policy
- [ ] choose the first bounded hybrid implementation batch from listening and
  measurement evidence
- [ ] reassess `g10.028` only after actual streaming source consumption is defined

## Acceptance Criteria

- [x] no contractual output tail is created only by post-render zero fill
- [ ] fixed and dynamic paths have content-aware boundary coverage
- [ ] quality gates include absolute full-render measurements
- [ ] required real-source families have completed listening findings
- [ ] OfflineHighQuality status and promotion language match measured evidence
- [ ] the next hybrid batch has explicit algorithm ownership and failure targets

## Validation

- `cargo test -p signal-dsp-stretch phase_vocoder_boundary`
- `cargo test -p signal-dsp-stretch offline_high_quality_boundary`
- `cargo test -p signal-dsp-stretch`
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- `effigy qa:docs`
- `effigy qa:northstar`

## Progress

- 2026-07-09: Opened from a code, evidence, and roadmap audit. Paused
  `g10.028`; corrected the active generation and contract front doors; made
  boundary correctness the first executable gate.
- 2026-07-09: Completed Batch 29.1. Offline STFT renders now use centred
  boundary padding and exact output cropping; compression and expansion tests
  prove source content reaches both endpoints. Bumped engine cache identity to
  `signal-native-stretch-v2`. OfflineHighQuality remains implementation-complete;
  product-promotion receipt changes stay in Batch 29.3 so evidence provenance,
  absolute limits, and listening requirements change together. Batch 29.2 now
  has reusable
  full-render integrity measurements for length, endpoint energy, added silence,
  and peak growth. External comparator quality rows report those fields for both
  Signal and the external render instead of limiting correctness evidence to an
  aligned excerpt.

## Next Task

Continue Batch 29.2 by setting evidence-backed absolute integrity limits, then
measure CPU realtime factor and peak working memory. Keep those measurements
explicit rather than inferring them from render duration or buffer sizes.
