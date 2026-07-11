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

### Batch 29.6C - Fixed-Map Peak Transient Proof

- [x] add one time-ramped companion FFT to estimate per-bin group delay on the
  current `2048/512` grid
- [x] use frozen onset guards, magnitude-minimum peak regions, and one
  centre-adjacent phase-reinitialization frame per collected attack event
- [x] keep constant global synthesis positions and ordinary propagation for
  non-transient bins; do not boost magnitude or crossfade output
- [x] report guarded events, candidate peaks, threshold crossings, collected
  regions, reinitialized bins/frames, and unmatched guards
- [ ] pass the unchanged contract `082` crest, placement, integrity,
  static-spectrum, formant, boundary, and combined gates
  - mechanism rejected: `0.040942 dB` `L001` improvement, `+16.851522`
    measurable-row mean timing delta, and `12/60` combined passes

### Batch 29.6D - H/R/P Separation Reconstruction

- [x] add report-only sample-rate-aware iterative H/R/P separation with
  long-resolution harmonic extraction and short-resolution percussive
  extraction from the complement
- [x] apply disjoint binary masks with `beta_h=2`, `beta_p=2`, `200 ms`
  horizontal median span, and `500 Hz` vertical median span
- [x] reconstruct harmonic, residual, and percussive source components through
  centred normalized inverse STFT without stretching
- [x] report mask populations, component energy, partition error,
  reconstruction RMS/peak error, boundary coverage, and deterministic hashes
- [x] pass the contract `082` exact reconstruction and `12 dB` synthetic
  harmonic/percussive/residual ownership gates

### Batch 29.6E - Additive H/R/P Fixed-Ratio Mono Gate

- [x] open only after Batch 29.6D passes
- [x] stretch harmonic content with long-window identity-locked PV, residual
  content with the current kernel, and percussive content with short normalized
  OLA under one ratio and exact target length
- [x] sum components sample-aligned without branch switching, crossfade, delay
  repair, waveform search, or component gain matching
- [x] report component length/peak growth, transient replica ratio, final
  recombination, and every original Batch 29.6 quality field
- [ ] pass every contract `082` and original Batch 29.6 mono gate on the
  60-render corpus
  - mechanism rejected: `3.375261 dB` anchored improvement and `4.083747 dB`
    worst crest passed, but timing regressed `23.411637` frames, integrity
    passed `51/60`, replica protection passed `26/48`, and the combined gate
    passed `0/60`
- [ ] keep Batch 29.7 closed until a complete mono candidate passes
  - Batch 29.6E is rejected; Batch 29.6G now owns the next mono gate

### Batch 29.6F - Full Phase-Gradient Kernel Proof

- [x] add report-only centered time- and frequency-phase derivative estimation
  on the frozen `4092` Hann / `8192` FFT geometry
- [x] integrate significant-bin phase with deterministic bounded max-heap
  propagation and trapezoidal time/frequency rules
- [x] enforce nonredundant-spectrum conjugate symmetry, deterministic
  below-tolerance phase, normalized overlap-add, and exact target crop
- [x] prove sine, chirp, impulse, two-tone, silence, and repeatability controls
  against Contract `082` without rendering the corpus

### Batch 29.6G - Full Phase-Gradient Fixed-Ratio Mono Gate

- [x] open only after Batch 29.6F passes
- [x] render the unchanged 60-row corpus without geometry, tolerance,
  derivative, or heap-priority sweeps
- [x] report all existing integrity, timing, crest, replica, spectral,
  formant, boundary, and combined fields against the current kernel and
  external comparator
- [ ] pass the complete mono gate before Batch 29.7 opens
  - candidate rejected: tonal regression-free passed `55/60` and direct
    comparator evidence improved, but `L001` crest improved only `1.667930 dB`,
    timing worsened `16.738760` frames, integrity passed `57/60`, replica
    protection passed `28/48`, and the combined gate passed `3/60`

### Batch 29.6H - Exact-Lattice Phase-Gradient Mono Gate

- [x] replace the repeated rounded analysis hop with absolute positions
  `A_n = round(n * 1024 / ratio)` while retaining the frozen synthesis grid
- [x] normalize backward and forward phase differences by their actual adjacent
  analysis intervals before centered averaging
- [x] report interval floor/ceiling counts, maximum/final mapping error,
  monotonicity, phase assignment, heap, symmetry, coverage, and hashes
- [x] pass the Contract `082` `0.5`-frame mapping gate on identity,
  compression, expansion, sine, chirp, impulse, two-tone, and silence controls
- [x] run the unchanged 60-row mono and comparator gate without transient,
  shape, geometry, tolerance, phase, or heap tuning
- [ ] open Batch 29.7 only after every complete mono gate passes
  - closed: complete gate passed `3/60`

### Batch 29.6I - Frequency-Adaptive Painless Reconstruction

- [x] construct one report-only frequency-adaptive nonstationary Gabor frame
  with constant-Q interior bands and explicit DC/Nyquist completion
- [x] derive canonical dual filters from a finite, strictly positive diagonal
  frame operator and satisfy the per-band painless support condition
- [x] prove exact-length analysis/synthesis on low, crossover, high,
  DC/Nyquist-edge, impulse, noise, mixed, and silence controls
- [x] report frame bounds, condition ratio, band geometry, coefficient counts,
  spectral coverage, reconstruction error, impulse delay, and repeat hashes
- [x] pass Contract `082` reconstruction and determinism gates before any
  frequency-adaptive phase propagation or corpus render opens

### Batch 29.6J - Common-Grid Wavelet Reconstruction

- [x] construct the Contract `082` `alpha=900`, `1536`-channel analytic
  wavelet bank with `16` lowpass channels and uniform `384`-frame decimation
- [x] apply the deterministic digital `(0,1)` channel-delay sequence and report
  its stable hash
- [x] compute the complete uniform-filter-bank frame bounds and canonical dual;
  do not reuse the Batch 29.6I diagonal painless dual
- [x] prove identity analysis/synthesis on the unchanged Batch 29.6I controls
  with condition ratio at most `1.25` and the frozen residual/error limits
- [x] keep phase propagation, the 60-row corpus, linked stereo, and product
  routing closed even if reconstruction passes

### Batch 29.6K - Common-Grid Phase-Transport Proof

- [x] estimate horizontal instantaneous frequency on the `384`-frame
  grid and prove scale on steady low, mid, and high tones
- [x] remove deterministic channel delay before adjacent-channel vertical
  differences and prove the compensation sign/residual
- [ ] project output columns to exact fractional source coordinates `m/ratio`;
  interpolate magnitudes and gradients, never wrapped complex coefficients
- [ ] integrate the positive-frequency gradient with one bounded deterministic
  heap and prove assignment, symmetry, coverage, impulse placement, and hashes
- [ ] pass every Contract `082` synthetic mechanism gate before the unchanged
  60-row mono corpus opens
  - rejected at the high-tone gate: phase differences alias outside the
    `+/-62.5 Hz` residual interval

### Batch 29.6L - Auxiliary Derivative-Filter Estimator

- [x] derive same-grid auxiliary time-derivative filters from every finalized
  tightened analysis response
- [x] prove cross-ratio sign and `1e-6` angular-frequency error on periodic
  `312.5 Hz`, `1 kHz`, `8 kHz`, and `19.5 kHz` tones
- [x] prove delay-compensated adjacent-channel residual at most `2e-5` radians
  without inter-column unwrap or a hidden shorter hop
- [x] prove silence skips zero-energy ratios, noise remains finite, and all
  evidence and hashes repeat exactly
- [x] keep projection, heap integration, synthesis, corpus, stereo, and product
  routing closed even if the estimator passes

### Batch 29.6M - Projected Field And Bounded Heap Proof

- [x] project magnitude, absolute instantaneous frequency, and
  delay-compensated vertical phase derivatives at exact `u=m/ratio`
- [x] prove bounded linear interpolation, legal padding reads, finite fields,
  monotonic coordinates, exact column counts, and repeat hashes on Contract
  `082` ratios and synthetic controls
- [x] integrate significant positive-frequency phases one output column at a
  time with deterministic horizontal/vertical priority and no duplicate or
  missing assignment
- [x] prove heap high-water stays within the duration-independent `3072`-entry
  cap and all assignment evidence repeats exactly
- [x] keep canonical-dual audio synthesis, placement, corpus, stereo, dynamic
  ratio, and product routing closed even if the mechanism passes

### Batch 29.6N - Common-Grid Synthetic Synthesis Proof

- [x] freeze canonical-dual spectrum assembly, real-output symmetry, padding,
  crop, exact-length, coverage, and impulse-placement rules after 29.6M passes
- [ ] derive the smallest whole-hop two-sided canonical-dual guard meeting
  `1e-12` tail energy, and stop if it exceeds `16384` frames
- [ ] reflect-pad the source, reuse 29.6M on logical negative/positive guarded
  columns, synthesize the complete canonical-dual spectrum, and crop only the
  protected centre
- [ ] prove identity, compression, and expansion on the Contract `082`
  synthetic control set before any corpus render
- [ ] stop for research on any symmetry, dual-residual, coverage, placement,
  finite-value, or determinism failure

### Batch 29.7 - Shared-Decision Linked Stereo

- [ ] share the time map and phase-propagation decisions across channels
- [ ] preserve per-channel complex spectra, phase gradients, and interchannel
  phase under the shared heap topology
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
- 2026-07-10: Reassessed transient ownership. Fixed-map peak-selective
  group-delay phase reinitialization is frozen for Batch 29.6C because it
  targets invalid attack phase prediction inside the current kernel without
  moving unrelated events. Explicit transient/residual separation is deferred:
  it first needs a perfect-reconstruction multiresolution split, mask
  continuity, component-processing, and recombination contract. Adaptive
  resolution and linked stereo remain closed. Decision evidence is in
  `docs/logs/2026-07/10-g10-029-fixed-map-peak-transient-decision.md`.
- 2026-07-10: Measured Batch 29.6C. The report-only companion FFT found
  `249687` peak candidates across `2370` guarded events and reinitialized
  `492156` bins at `1386` centre-threshold crossings. Integrity and coverage
  passed `60/60`, but anchored `L001` improved only `0.040942 dB`, mean timing
  worsened `16.851522` frames across measurable rows, and the combined gate
  passed `12/60`. The mechanism is rejected without tuning. Evidence is in
  `docs/logs/2026-07/10-g10-029-fixed-map-peak-transient-rejection.md`.
- 2026-07-10: Froze the final untested structural family as refined iterative
  H/R/P separation. Long-resolution tightened masking extracts clearly
  harmonic content; short-resolution masking extracts clearly percussive
  content from the complement; ambiguous content stays residual. Batch 29.6D
  proves exact separation reconstruction before any component TSM. If it
  passes, Batch 29.6E uses long PV, current PV, and short OLA under one global
  map and additive recombination. Decision evidence is in
  `docs/logs/2026-07/10-g10-029-hpr-separation-contract.md`.
- 2026-07-10: Batch 29.6D passed without parameter tuning. At `48 kHz`, the
  separator selected `8192/2048` long and `512/128` short STFT geometry. Mixed
  source reconstruction measured `8.940697e-8` peak error and `1.939046e-8`
  RMS error with exact component lengths, zero uncovered samples, finite
  components, and deterministic hashes. Sine, impulse, and noise ownership
  margins were `30.933980 dB`, `164.871272 dB`, and `12.925746 dB`. Batch 29.6E
  is open; no component TSM or production routing changed.
- 2026-07-10: Batch 29.6E is rejected without tuning. The additive candidate
  passed the anchored crest, worst crest, fast-movement, exact-length,
  coverage, determinism, finite-output, and no-hidden-gain checks. It failed
  timing, endpoint integrity, transient-replica, static-residual,
  unsupported-bin, tonal, formant, boundary, and combined gates. Batch 29.7
  remains closed. Evidence is in
  `docs/logs/2026-07/10-g10-029-hpr-additive-rejection.md`.
- 2026-07-10: Completed the synthesis-family reassessment. WSOLA,
  sinusoidal/residual synthesis, and onset-compensated adaptive resolution do
  not clear Signal's measured failure boundary. Batch 29.6F now freezes one
  whole-band fixed-resolution full phase-gradient kernel proof with no source
  split or local time compensation. Batch 29.6G owns the later corpus gate;
  linked stereo remains closed. Evidence is in
  `docs/logs/2026-07/10-g10-029-phase-gradient-reassessment.md`.
- 2026-07-10: Batch 29.6F passed without tuning. The report-only whole-band
  kernel uses centered time/frequency derivatives and bounded deterministic
  heap integration. Every significant control bin was assigned exactly once;
  heap high-water was at most `4099/8194`; conjugate symmetry, overlap-add
  coverage, exact `0.75x` and `1.5x` lengths, finite output, identity bypass,
  both propagation directions, and repeat hashes passed. Batch 29.6G is open;
  no corpus render or product route changed. Evidence is in
  `docs/logs/2026-07/10-g10-029-phase-gradient-kernel-proof.md`.
- 2026-07-10: Batch 29.6G is rejected without tuning. Whole-band phase-gradient
  integration improved tonal regression-free to `55/60`, improved expansion
  residual/unsupported-bin means, raised mean Rubber Band correlation, and
  lowered comparator RMS error. It failed anchored crest, timing, integrity,
  replica, transient, formant, boundary, and combined gates. Batch 29.7 remains
  closed. Evidence is in
  `docs/logs/2026-07/10-g10-029-phase-gradient-mono-rejection.md`.
- 2026-07-10: Reassessed the phase-gradient timing failure. Repeating one
  rounded analysis hop makes the internal lattice ratios differ from the
  requested ratios and permits roughly `40`, `67`, and `161` frames of
  five-second endpoint mapping drift. Batch 29.6H now freezes absolute rounded
  analysis centres and interval-aware derivatives before any new transient or
  shape mechanism. Evidence is in
  `docs/logs/2026-07/10-g10-029-exact-lattice-reassessment.md`.
- 2026-07-10: Batch 29.6H mapping passed with `0.4` frame maximum error, but the
  mono candidate is rejected: `L001` improved `2.379387 dB`, timing worsened
  `17.789744` frames, integrity passed `57/60`, replica `27/48`, tonal `57/60`,
  and combined `3/60`. Linked stereo remains closed.
- 2026-07-10: Reassessed attack placement and shape after exact lattice failed.
  Frequency-adaptive painless nonstationary Gabor analysis is the next
  materially different family: it can improve time resolution at high
  frequencies while retaining low-frequency selectivity inside one invertible
  transform. The published onset-adaptive TSM policy is not adopted because it
  uses attack detection and local unity stretch. Batch 29.6I proves only
  canonical-dual reconstruction and transform geometry. Evidence is in
  `docs/logs/2026-07/10-g10-029-frequency-adaptive-reassessment.md`.
- 2026-07-10: Batch 29.6I passed without tuning. The mixed control used `576`
  bands and `10634` coefficients; frame bounds were `0.999999881` and
  `1.000000119`; peak/RMS reconstruction error was `1.490116119e-7` /
  `3.762034804e-8`. Coverage, painless support, zero-delay, finite-value, edge
  control, and repeat-hash gates passed. Evidence is in
  `docs/logs/2026-07/10-g10-029-frequency-adaptive-reconstruction-proof.md`.
- 2026-07-10: Stopped direct phase propagation on the Batch 29.6I geometry.
  Published filter-bank PGHI assumes uniform decimation and explicitly leaves
  nonuniform heap integration as future work. Batch 29.6J now proves the
  published grid-decimated wavelet prerequisite: one aligned coefficient
  matrix, redundancy `8`, complete canonical dual, and bounded frame condition.
  Evidence is in
  `docs/logs/2026-07/10-g10-029-common-grid-wavelet-reassessment.md`.
- 2026-07-10: Batch 29.6J passed. The mixed control produced a `1536 x 11`
  coefficient matrix. Estimated frame condition was `1.025819956`, maximum
  canonical-dual residual was `6.225219e-11`, and peak/RMS reconstruction error
  was `2.910383e-11` / `5.520117e-13`. All control and repeat gates passed.
  Evidence is in
  `docs/logs/2026-07/10-g10-029-common-grid-wavelet-reconstruction-proof.md`.
- 2026-07-10: Froze Batch 29.6K phase transport. Channel phase is first
  transported from `n*384+d[k]` to nominal common-grid time using horizontal
  instantaneous frequency. Output column `m` samples magnitude and gradient
  fields at exact source coordinate `m/ratio`; synthesis stays on the proven
  uniform grid. Evidence is in
  `docs/logs/2026-07/10-g10-029-common-grid-phase-transport-contract.md`.
- 2026-07-10: Batch 29.6K stopped before interpolation or heap work. Low/mid
  tones passed phase scale and delay compensation, but the `8 kHz` control
  produced `0.065450362` radians/sample frequency error and `0.243248864`
  radians compensated residual. Evidence is in
  `docs/logs/2026-07/10-g10-029-common-grid-phase-alias-rejection.md`.
- 2026-07-11: Froze Batch 29.6L. Same-column auxiliary derivative-filter
  ratios replace aliased inter-column phase differences. Tone scale/sign,
  delay compensation, zero-energy handling, and determinism must pass before
  heap integration. Evidence is in
  `docs/logs/2026-07/11-g10-029-auxiliary-derivative-estimator-contract.md`.
- 2026-07-11: Batch 29.6L passed. A deterministic dominant-channel carrier per
  column avoids weak-filter leakage while retaining same-column alias freedom.
  All tone, silence, noise, delay-compensation, and repeat gates pass. Evidence
  is in
  `docs/logs/2026-07/11-g10-029-auxiliary-derivative-estimator-proof.md`.
- 2026-07-11: Froze Batch 29.6M. Exact fractional projection now owns three
  unwrapped fields. Positive-grid integration is output-column-local with a
  duration-independent `3072`-entry heap bound. Audio synthesis remains Batch
  29.6N. Evidence is in
  `docs/logs/2026-07/11-g10-029-projected-field-heap-contract.md`.
- 2026-07-11: Batch 29.6M passed all `30` synthetic control/ratio cases with
  zero coordinate error, `34592` horizontal and `10405` vertical assignments,
  no duplicate or missing significant cells, and heap high-water `1756/3072`.
  Evidence is in
  `docs/logs/2026-07/11-g10-029-projected-field-heap-proof.md`.
- 2026-07-11: Froze Batch 29.6N. A measured two-sided canonical-dual guard now
  protects both crop boundaries from the circular transform seam. Guard
  failure stops before coefficient assembly; audio failure stops before the
  corpus. Evidence is in
  `docs/logs/2026-07/11-g10-029-common-grid-synthesis-contract.md`.

## Next Task

Implement Batch 29.6N, starting with the dual-atom guard proof. Keep the corpus,
linked stereo, dynamic ratio, and product routing closed.
