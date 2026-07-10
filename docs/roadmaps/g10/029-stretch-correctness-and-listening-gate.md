# 029 - Stretch Correctness And Listening Gate

Status: active
Owner: dsp
Created: 2026-07-09
Depends on: g10.021, g10.022, g10.024, g10.027
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/082-offline-time-stretch-synthesis-policy-contract.md`
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
- [x] record aggregate operator findings across percussion, bass, vocals,
  sustains, and full mix
- [x] classify observed transient and tonal failures without fabricating
  row-level notes
- [x] add event-level timing and transient-crest diagnostics distinct from
  output-length drift
- [x] isolate the `L001` spike with same-event phase-lock controls and reject
  corpus-regressing local variants
- [x] classify long-stretch grain with source-relative residual, sideband, and
  fast spectral-modulation evidence
- [ ] complete row-level manifest validation and independent stereo review
- [x] close formant and boundary classification with source-relative envelope
  and exterior-step evidence
- [x] reject source-endpoint tail anchoring after the combined corpus gate
- [x] qualify the bounded zero-tail anchor for focused listening after its
  objective corpus gate
- [x] export the worst zero-anchor corrections as a concealed three-way mono
  tail-listening pack with post-tail silence
- [x] reject unconditional additive zero anchoring after concealed sustained
  tails exposed low-end thumps
- [x] qualify a same-span multiplicative zero fade through the objective gate
  and export its concealed comparison pack
- [x] reject unconditional multiplicative fading after concealed full-mix
  tails exposed low-end thumps
- [x] isolate spectral centroid as the only clean tail-local separator while
  withholding selector implementation pending cross-source validation
- [x] export a balanced concealed validation pack with six distinct unseen
  sources across both centroid bands
- [x] reject the centroid selector after cross-source listening failed to
  reproduce the preference split and close tail-envelope work

### Batch 29.4 - Structural Hybrid Checkpoint

- [x] consolidate mono evidence and separate design authority from production
  promotion gates
- [x] define transient/tonal classification and multiresolution window ownership
- [x] define shared stereo peak/phase decisions and formant policy
- [x] choose the first bounded hybrid implementation batch from listening and
  measurement evidence
- [ ] reassess `g10.028` only after actual streaming source consumption is defined

Batch 29.4 is open for design and report-only candidate planning. Production
replacement, cache-identity changes, product promotion, and realtime exposure
remain blocked on the declared corpus gates, row-complete listening, and
independent stereo review.

### Batch 29.5 - Kernel Seam And Classification Trace

- [x] extract reusable analysis, propagation, and synthesis state while proving
  the current default output remains bit-exact
- [x] add the frozen transient, mixed, and tonal classifier as a report-only
  frame trace
- [x] add the bounded transition schedule and trace without mixing branch audio

### Batch 29.6 - Fixed-Ratio Mono Hybrid

- [x] run the short transient, current mixed, and long tonal branches with
  continuous state
- [x] apply only the frozen ownership and transition schedule
- [ ] pass the local crest, corpus timing, tonal movement, static spectrum, and
  full-render combined gates
  - first frozen candidate rejected: unchanged `L001` crest, `1.25x` static
    residual regression, and `50/60` tonal/combined passes

### Batch 29.6A - Alignment Reassessment

- [x] separate low-correlation, normalization, span-geometry, and missing-edge
  transition failures
- [x] measure bounded best-lag recovery without changing candidate audio
- [x] reject branch-delay repair after recovered spans required large and
  inconsistent entry/exit lags
- [x] promote the one-timeline successor through architecture and contract `082`

### Batch 29.6B - Adaptive Transient Timeline Proof

- [x] derive one monotonic synthesis-position schedule from fixed projected
  anchors and frozen transient guards
- [x] keep protected overlapping attack frames at local ratio `1` and
  compensate only inside steady intervals
- [x] reinitialize protected frame phase inside the current `2048/512` engine;
  do not crossfade output waveforms
- [ ] pass the contract `082` transient, placement, integrity, static-spectrum,
  and combined gates
  - mechanism rejected: `0.536217 dB` `L001` improvement, `+4.942263` mean
    timing delta, and `9/60` combined passes

### Batch 29.6C - Adaptive Resolution Reconstruction

- [ ] open only after Batch 29.6B passes
- [ ] define one nonstationary short/current/long frame schedule and compatible
  reconstruction weights
- [ ] pass perfect-reconstruction, fast spectral-movement, static-spectrum,
  transient, and boundary gates before broad corpus promotion

### Batch 29.6D - Combined Fixed-Ratio Mono Gate

- [ ] combine the proven transient timeline and adaptive-resolution mechanisms
- [ ] pass every original Batch 29.6 mono gate on the 60-render corpus
- [ ] open Batch 29.7 only after the complete mono candidate passes

### Batch 29.7 - Shared-Decision Linked Stereo

- [ ] replace independent mid/side mono decisions with one shared multichannel
  classifier, peak map, reset schedule, and transition schedule
- [ ] preserve per-channel instantaneous frequency and interchannel phase at
  shared peaks
- [ ] pass mono parity, image, interchannel-phase, and one-sided-transient gates

### Batch 29.8 - Listening And Dynamic Checkpoint

- [ ] export concealed mono and stereo artifacts only after objective gates pass
- [ ] keep production blocked until row-complete and independent stereo findings
  are frozen
- [ ] open stateful dynamic-ratio design only after the fixed-ratio candidate
  passes; do not reuse independent segment concatenation

## Acceptance Criteria

- [x] no contractual output tail is created only by post-render zero fill
- [ ] fixed and dynamic paths have content-aware boundary coverage
- [x] quality gates include absolute full-render measurements
- [ ] required real-source families have completed listening findings
- [x] OfflineHighQuality status and promotion language match measured evidence
- [x] the next hybrid batch has explicit algorithm ownership and failure targets

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
- 2026-07-10: Recorded the operator's aggregate 15-pair findings without
  fabricating row-level TSV completion. Signal is close on most attacks, but
  shows occasional visible transient spikes, slightly softer secondary attacks
  under compression, and slightly grainier/atonal long stretches. Rubber Band
  remains slightly more musical at longer stretches. Stereo is unassessed and
  still requires an independent listener.
- 2026-07-10: Added sample-frame-refined transient placement and local crest
  evidence. Across 47 matched rows, mean absolute placement was effectively
  tied: `102.826` frames Signal versus `101.845` Rubber Band, so the suspected
  global timing drift is not confirmed. `L001` at `0.75x` is the strongest
  Signal crest outlier: `5.655 dB` versus `1.832 dB` Rubber Band and `3.647 dB`
  for the independent-bin draft. Compression does not enable transient reset,
  so the next focused probe owns identity locking and overlap-add reconstruction,
  not reset tuning. Evidence is target-local at
  `target/stretch-corpus-g10-029-transient-detail-v1.tsv`; durable findings are
  in `docs/logs/2026-07/10-g10-029-operator-listening-and-transient-diagnostic.md`.
- 2026-07-10: Same-event controls isolate broad identity locking as the direct
  `L001` spike cause: Signal Default measured `5.655 dB`, independent bins
  `0.459 dB`, and Rubber Band `-0.515 dB` at source frame `180354`. Shared
  overlap-add reconstruction does not reproduce the event under independent
  propagation. Stability-adaptive locking removes the local spike but worsens
  timing in `37` of 48 measured rows. Tracked peak regions and magnitude slew
  regress worst crest in `28` and `34` rows. Two tighter bounded-region probes
  also failed combined crest/timing gates and were removed. No production path
  changed. Evidence and the rejection decision are in
  `docs/logs/2026-07/10-g10-029-phase-lock-control-rejection.md`.
- 2026-07-10: Source-relative tonal diagnostics classify the long-stretch
  finding as excess fast spectral movement, not confirmed added ringing. Signal
  exceeded Rubber Band spectral movement in `38/40` expansion rows while its
  static residual and unsupported-bin mass were lower. Envelope movement was
  mixed. Broad phase locking improved on independent bins at `1.5x` but
  regressed them at `1.25x`, so no production change promoted. Evidence and
  limits are in
  `docs/logs/2026-07/10-g10-029-tonal-texture-diagnostic.md`.
- 2026-07-10: Formant and boundary diagnostics find no broad-envelope failure
  in Signal's 12 vocal rows, but isolate a fixed-ratio exterior-tail defect.
  Signal beat Rubber Band's source-relative envelope residual in every vocal
  row. Its louder exterior edge was the tail in `59/60` rows; `17/60` exceeded
  `-20 dBFS`, and the worst pad ended at `-6.328693 dBFS`. All 60 Signal rows
  still passed length, endpoint-energy, added-silence, and peak-growth limits.
  Production remains unchanged. Evidence and candidate gates are in
  `docs/logs/2026-07/10-g10-029-formant-and-boundary-diagnostic.md`.
- 2026-07-10: The bounded source-endpoint tail anchor passed integrity and all
  transient, tonal-texture, and formant-envelope tolerances in `60/60` rows and
  worsened no exterior edge. It materially improved only `5/17` loud-tail
  targets against a `13/17` gate, and the worst candidate edge remained
  `-7.393442 dBFS`. The control stays report-only; production and cache identity
  remain unchanged. Evidence and rejection rationale are in
  `docs/logs/2026-07/10-g10-029-source-tail-anchor-rejection.md`.
- 2026-07-10: The bounded zero-tail control materially improved all `17/17`
  loud tails, reduced the worst exterior step from `-6.328693` to
  `-29.129923 dBFS`, and passed integrity, transient, tonal-texture, and
  formant-envelope tolerances in `60/60` rows. It changes the final 255 samples
  in every render; five peak corrections exceed `0.25`, so it qualifies for
  focused listening rather than production. Cache identity remains unchanged.
  Evidence and limits are in
  `docs/logs/2026-07/10-g10-029-zero-tail-anchor-objective-gate.md`.
- 2026-07-10: Generated the focused tail pack at
  `target/stretch-corpus-g10-029-tail-listening-pack-v1`. It contains the six
  largest current endpoint jumps, from `-6.328693` to `-12.769706 dBFS`, with
  concealed current, source-anchor, and zero-anchor mono candidates. Each WAV
  exposes the final second followed by `250 ms` digital silence. One shared
  per-trial gain preserves relative boundary amplitude. Production and cache
  identity remain unchanged. Pack design and status are in
  `docs/logs/2026-07/10-g10-029-tail-listening-pack.md`.
- 2026-07-10: Completed all six concealed tail trials. The additive zero anchor
  was cleanest on the drum case but produced low-end thumps on both worst
  sustained-pad cases, where current was preferred. The remaining three trials
  were materially similar. Unconditional zero-anchor promotion is rejected;
  production and cache identity remain unchanged. The next control isolates the
  correction law with a same-span multiplicative fade. Findings and revealed
  assignments are in
  `docs/logs/2026-07/10-g10-029-tail-listening-pack.md`.
- 2026-07-10: The report-only 256-frame multiplicative zero fade passed the
  complete 60-row objective gate: `60/60` integrity, transient, tonal, formant,
  and combined passes; `17/17` loud tails materially improved; no edge worsened.
  Its maximum correction was `0.769897819`, larger than the additive control's
  `0.482575566`, so objective success qualifies listening only. A six-trial
  current/additive/multiplicative pack is ready at
  `target/stretch-corpus-g10-029-multiplicative-tail-listening-pack-v1`.
  Production and cache identity remain unchanged. Evidence and limits are in
  `docs/logs/2026-07/10-g10-029-multiplicative-tail-fade-objective-gate.md`.
- 2026-07-10: Completed the six concealed multiplicative trials. The control
  was cleanest on both pads and the drum, but produced low thumps on both
  decisive full-mix trials where additive/current were clean. No fixed
  256-frame envelope is universally safe. Both controls remain report-only;
  production and cache identity remain unchanged. The next reassessment tests
  whether tail-local features support a content-derived selector before any
  further DSP control is added. Findings and revealed assignments are in
  `docs/logs/2026-07/10-g10-029-multiplicative-tail-fade-objective-gate.md`.
- 2026-07-10: Added report-only tail-local classification for the six frozen
  labels. Only spectral centroid separated every decisive result: wins ranged
  from `662.676157` to `1450.409813 Hz`; losses ranged from `2422.601943` to
  `2441.410312 Hz`; neutral T006 measured `2485.733931 Hz`. A provisional
  `< 2000 Hz` rule fits without family labels, but the evidence contains only
  three unique source excerpts. No selector or production path changed.
  Cross-source concealed validation is required. Evidence and stop conditions
  are in
  `docs/logs/2026-07/10-g10-029-tail-local-feature-classification.md`.
- 2026-07-10: Measured the 60-row broad pool and exported the cross-source pack
  at `target/stretch-corpus-g10-029-tail-classifier-validation-pack-v1`. It
  excludes all three labeled excerpts and contains six distinct sources: three
  below `2000 Hz` (`939.215402` to `1973.789903 Hz`) and three above
  (`2222.696644` to `3652.827333 Hz`). Candidate identity and centroid band are
  sealed in the key. Production and selector paths remain unchanged. Pack
  design and stop conditions are in
  `docs/logs/2026-07/10-g10-029-tail-local-feature-classification.md`.
- 2026-07-10: Completed the six concealed cross-source trials and opened the
  key after notes were frozen. Five trials had no clear difference. In T002 the
  additive control alone had a slight bass thump; current and multiplicative
  were clean. The multiplicative preference split did not reproduce in either
  centroid band. The provisional `< 2000 Hz` selector is rejected and
  tail-envelope work is closed. Both controls remain report-only; production
  and cache identity remain unchanged. Findings are in
  `docs/logs/2026-07/10-g10-029-tail-local-feature-classification.md`.
- 2026-07-10: Reassessed the complete mono and objective evidence. Two
  structural targets are repeatable: broad identity locking causes the isolated
  `L001` crest spike, and expansion shows excess fast spectral movement in
  `38/40` rows. Timing drift and a fixed-ratio formant defect are not
  established. Fixed-envelope tail work is closed. Batch 29.4 is authorized
  for design and report-only candidate planning; row-complete listening and
  independent stereo review still block production replacement and quality
  claims. Decision details are in
  `docs/logs/2026-07/10-g10-029-mono-evidence-reassessment.md`.
- 2026-07-10: Froze the structural hybrid design from the code and evidence
  map. The first candidate uses short independent-bin transient ownership,
  current-window mixed ownership, and long identity-locked tonal ownership,
  with continuous branch state and bounded transitions. Linked stereo requires
  one shared classifier, peak map, reset schedule, and transition schedule;
  the current independent mid/side engines do not satisfy that policy. Batch
  29.5 starts with bit-exact kernel extraction and report-only traces. Full
  design, gates, and stop conditions are in
  `docs/logs/2026-07/10-g10-029-structural-hybrid-design.md`.
- 2026-07-10: Completed Batch 29.5. The current phase-vocoder core now has
  explicit analysis, propagation, and synthesis state, with output locked by
  sample-bit hash `0x8255b18311f778f9`. The report-only hybrid trace applies
  the frozen transient guards, tonal hold, compression and identity scope,
  boundary guards, and bounded low-energy transition schedule. It renders no
  branch audio and leaves current output unchanged. Doctor returned to the
  existing `48` god-file and `5` attention-marker baseline after module
  splitting. Evidence is in
  `docs/logs/2026-07/10-g10-029-hybrid-kernel-seam.md`.
- 2026-07-10: Measured the first Batch 29.6 fixed-ratio mono hybrid. Continuous
  branches and conservative whole-span transition fallback applied only `56`
  of `2024` ownership spans. The `L001` crest stayed at `5.655483 dB`, the
  `1.25x` static residual regressed, and tonal/combined gates passed `50/60`.
  The candidate is rejected, linked stereo stays closed, and production is
  unchanged. Evidence and the reassessment boundary are in
  `docs/logs/2026-07/10-g10-029-fixed-ratio-mono-hybrid-rejection.md`.
- 2026-07-10: Completed the alignment reassessment. A bounded `-256..=256`-frame
  search made `980/1968` rejected spans correlation-safe only with `152.383`
  mean absolute lag and `210.465` mean entry/exit disagreement. Branch delay
  and relaxed transition gates are rejected. Architecture and contract `082`
  now require one synthesis timeline, a current-grid transient proof, then an
  adaptive-resolution checkpoint. Evidence is in
  `docs/logs/2026-07/10-g10-029-hybrid-alignment-reassessment.md`.
- 2026-07-10: Measured Batch 29.6B. Exact onset anchors and overlap-add
  coverage passed, but `479` protected onsets coexisted with `1891` dense
  conflicts and synthesis hops up to `1664` frames. `L001` improved only
  `0.536217 dB`, mean event placement worsened `4.942263` frames, and the
  combined gate passed `9/60`. The mechanism is rejected and Batch 29.6C stays
  closed. Evidence is in
  `docs/logs/2026-07/10-g10-029-adaptive-transient-timeline-rejection.md`.

## Next Task

Stop for contract `082` reassessment after the rejected Batch 29.6B mechanism.
Compare peak/group-delay transient preservation inside the fixed global time map
against explicit transient/residual separation, then freeze one mechanism
before another candidate. Do not tune classifier or compensation constants,
open adaptive resolution or linked stereo, or change production, cache,
pitch/dynamic, product, or RealtimePreview routing.
