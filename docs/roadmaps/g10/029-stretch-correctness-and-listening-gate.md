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
- [x] replace draft-relative product promotion with absolute and
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
- [x] measure CPU realtime factor and peak working memory for promoted paths
- [x] add absolute acceptance limits separate from draft comparisons

### Batch 29.3 - Promotion And Listening

- [x] prevent synthetic-only receipts from opening product-quality promotion
- [x] generate source/Signal/Rubber Band level-matched blind-listening renders
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
- [x] quality gates include absolute full-render measurements
- [ ] required real-source families have completed listening findings
- [x] OfflineHighQuality status and promotion language match measured evidence
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
- 2026-07-09: Completed Batch 29.2. `offline-high-quality-v1` now enforces
  absolute limits of `0.5` frame length drift, `7 dB` active-endpoint RMS
  change, zero added-silence frames, and `6 dB` peak growth. Inactive source
  endpoints are reported but do not create false `240 dB` failures. The 7 dB
  envelope covers the measured 18-row Signal/Rubber Band v2 pack: active
  endpoint maxima were `5.772470 dB` for Signal and `6.527985 dB` for Rubber
  Band. All 18 Signal and 18 external rows passed; Signal added no silence and
  peaked at `3.162587 dB` growth.
- 2026-07-09: Added explicit per-render resource measurements. CPU realtime
  factor is elapsed Signal render time divided by rendered-audio duration.
  Peak working memory is measured live-heap growth above the pre-render
  baseline, not inferred from output length or buffer capacity. On an Apple M5
  Max release build (`rustc 1.96.0`), the 18-row path means/maxima were:
  Default `0.002678/0.003896` CPU and `3,708,980` bytes peak heap;
  CompressionShortWindowSelector `0.005292/0.016049` CPU and `3,708,980`
  bytes; ExpansionShortWindowSelector `0.005231/0.008431` CPU and `5,031,980`
  bytes. Evidence is target-local under
  `target/stretch-corpus-g10-029-*-measurement-v2.tsv`; timings are machine
  observations, not portable acceptance limits.
- 2026-07-09: Closed synthetic-only product promotion. Synthetic comparison
  receipts may pass their regression policy, but product-facing acceptance now
  additionally requires absolute integrity, external-comparator coverage, and
  completed findings for all five real-source families. Direct composite
  receipts remain path-specific. Synthetic-policy render/cache helpers now
  return non-ready plans or `NotReady` instead of materializing product output.
- 2026-07-09: Generated the first bounded blind pack at
  `target/stretch-corpus-g10-029-blind-listening-pack-v1`. It contains 15
  stereo A/B pairs: one source per percussion, bass, vocals, pads/sustains, and
  full-mix family at `0.75x`, `1.25x`, and `1.5x`. Source, Signal Default, and
  Rubber Band R3 renders use one per-pair RMS target with a `0.95` peak ceiling.
  The deterministic assignment is concealed in `blind-listening-key.tsv`;
  `blind-listening-notes.tsv` requires transient, tonal, stereo, formant,
  boundary, preference, and completion fields. Current validator status is
  `Incomplete`: 15 pairs, 0 of 5 completed families, 0 invalid completed rows.
  No listening findings were fabricated.

## Next Task

Complete the blind pack at
`target/stretch-corpus-g10-029-blind-listening-pack-v1` without opening the key:
fill every classification and preference field, set `completed=true` per heard
pair, then rerun `stretch-corpus-report --check-blind-listening-notes`. Do not
open product promotion or start Batch 29.4 until all five families validate.
