# 006 - Analysis Pruning And Measurement Correctness

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.001
Vision tags: `ANALYSIS`, `DEMOLITION`, `CORRECTNESS`

## Problem

The analysis crates have sound DSP cores (BS.1770-4 K-weighting digit-correct,
Krumhansl/Temperley key profiles, real multi-feature onset detection with
click-track tests to ±0.1 BPM) wrapped in speculative mass. signal-analysis-
rhythm carries ~7k lib LoC of unconsumed tempo/meter "continuity" taxonomy
(44 public enums, 10-tuple decision fields) plus ~8k LoC of tests that verify
the taxonomy against itself. signal-analysis-embed is not embeddings — an
8-dim descriptor projection wrapped in a fake model registry. Loudness has
two real measurement defects: true-peak uses linear interpolation instead of
the BS.1770-4 Annex 2 polyphase FIR (under-reads inter-sample peaks while
labeled dBTP), and LRA omits EBU Tech 3342's −20 LU relative gate. The suite
has no absolute-LUFS known-answer test, so a calibration bug would pass.

## Goals

- [x] extract rhythm's real core (~3k LoC: onset features, ACF tempo,
      refinement, beat utils, click-track/drift tests) and delete the
      continuity taxonomy and its tests
- [x] reduce embed to a descriptor-vector function (inside character or a
      one-file module); delete the model-registry fiction
- [x] true-peak: 4x polyphase FIR oversampling per BS.1770-4 Annex 2
- [x] LRA: add the relative −20 LU gate; drop incomplete trailing blocks from
      gating means
- [x] absolute known-answer tests: 997 Hz sine at −23 dBFS ≈ −23 LUFS (and a
      dBTP inter-sample-peak case)
- [x] onset extraction: drop per-call OS-thread spawn/panic scaffolding (use
      rayon, already a dependency)
- [ ] move the corpus/harness acceptance layer of signal-analysis to
      dev-/test-support (deferred to g10.009 hygiene pass)

## Non-Goals

- [ ] no DP/HMM beat tracking (backlog; current grid placement stays)
- [ ] no surround channel weights until multichannel is a product feature
- [ ] no new analysis features

## Execution Plan

### Batch 6.1 - Rhythm Extraction

- [ ] carve the core out from under the taxonomy; delete `tempo_state*`,
      `meter_state/*continuity*`, `tempo_policy/continuity` + tests
- [ ] keep click-track known-answer suite green throughout

### Batch 6.2 - Embed Reduction

- [ ] descriptor-vector function with the existing weights; delete registry
      types; migrate the semantic-tag matching if anything consumes it

### Batch 6.3 - Loudness Correctness

- [ ] polyphase true-peak; LRA relative gate; trailing-block handling
- [ ] compliance-vector tests (absolute LUFS, dBTP)

## Acceptance Criteria

- [ ] rhythm crate sheds the taxonomy with BPM detection accuracy unchanged
      (±0.1 BPM click-track suite green)
- [ ] −23 LUFS reference test passes; constructed inter-sample peak reads
      above its sample peak
- [ ] signal-runtime media-analysis consumers (loudness/character) stay green

## Risks and Mitigations

- Risk: runtime consumes rhythm/tonal fields scheduled for deletion.
- Mitigation: audit found loudness/character as the only runtime-consumed
  crates; verify by grep before each cut.

## Evidence Requirements

- [ ] LoC deltas and the new known-answer test outputs in the progress log

## Progress (2026-06-11)

- Batches 6.1+6.2 (one commit, −11.4k rhythm / −340 embed): continuity
  taxonomy deleted (tempo_state trees, meter continuity machinery, 26 test
  files); public enums 43→11, structs 73→25; BeatAnalysisResult trimmed to
  real outputs (bpm, confidence, beats, onsets, candidates, diagnostics,
  interpretation, ambiguity, meter). Click-track suite green at identical
  tolerances. Per-analyze thread-spawn scaffolding replaced with sequential
  computation. Embed reduced to plain projection + tag-matching functions;
  model-registry fiction deleted. Consumer check confirmed zero external
  consumers; analysis feature inspector demo tokens updated.
- Batch 6.3 (one commit): true-peak now 4x polyphase windowed-sinc per
  Annex 2 tap budget (12 taps/phase, Blackman; inter-sample case recovered
  2.93 dB: −9.03 → −6.10 dBTP vs −6.02 analytic); LRA gains the −20 LU
  relative gate; integrated gating consumes complete 400 ms blocks only.
  Known-answer suite added: −3.01 LUFS (997 Hz, 0 dBFS, left-only),
  −26.01 LUFS at −23 dBFS, LRA gate case (6.02 LU vs ~40 ungated).
- Gates per batch: workspace build, analysis + runtime suites serial, all
  green. Stale doc references (dsp-analysis-feature-reference.md) left for
  g10.009's truth pass.

## Next Task

g10.007 (plugin domain pruning) — parallel lane after g10.001.
