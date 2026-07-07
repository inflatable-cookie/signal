# 022 - OfflineHighQuality Stretch DSP Depth

Status: ready
Owner: dsp
Created: 2026-07-07
Depends on: g10.021
Vision tags: `DSP`, `STRETCH`, `QUALITY`

## Problem

OfflineHighQuality has a clean-room foundation: phase-vocoder baseline,
identity phase locking, transient reset behavior, linked stereo, static pitch
composition, dynamic ratio segments, and cacheable render/export/freeze
artifacts. It is still a foundation, not the final quality tier. The next
implementation work should target audible quality weaknesses found by the
comparison reports, not add more receipt or fixture surfaces.

## Goals

- [x] run the current comparison and quality-priority reports before choosing
  each algorithm change
- [x] choose one top measured weakness per batch: transient smear, loop seams,
  sustained phasiness, stereo image drift, pitch error, or timing drift
- [ ] add a multiresolution or hybrid STFT/time-domain path when the measured
  weakness justifies it
- [ ] improve transient anchoring or local time-domain splice behavior for
  percussive material
- [ ] improve vertical coherence for dense sustained and polyphonic material
- [ ] improve loop-seam handling under fixed ratios and dynamic ratio segments
- [ ] add static-pitch quality evidence across vocals, bass, and full mixes
- [x] keep `PhaseVocoderStretcher` as the draft baseline for regression
  comparison

## Execution Plan

### Batch 22.1 - Measured Weakness Selection

- [x] run synthetic comparison and quality-priority reports
- [x] record the top target and the rejected alternatives before coding

### Batch 22.2 - First Quality Improvement

- [x] implement one DSP change against the selected target
- [x] prove the chosen metric improves or holds without creating a higher
  priority regression

### Batch 22.3 - Multiresolution Or Hybrid Path

- [ ] add the next engine path only when a measured weakness requires it
- [ ] keep the lower-complexity path available for regression comparison

## Acceptance Criteria

- [x] every DSP batch names the metric or listening failure it targets
- [x] OfflineHighQuality improves or holds against the draft baseline on the
  chosen target and does not create a higher-priority regression elsewhere
- [x] output length and deterministic cache identity behavior remain stable
- [x] no realtime audio-thread path calls whole-buffer stretch processing

## Validation

- `cargo test -p signal-dsp-stretch`
- focused synthetic and real-corpus report runs

## Progress

- 2026-07-07: opened as active g10 OfflineHighQuality DSP quality work after
  the stretch policy/fixture line was judged mature enough and further
  proof-shaping was called out as churn.
- 2026-07-07: ran `stretch-corpus-report` with
  `projection:g10.022-selection`. No operator licensed material or external
  renders were present, so real-report evidence remains an operator input.
  Synthetic comparison had no regressions or inconclusive rows. The selected
  residual target was dynamic-ratio segment seam click:
  `stretch:tempo_ramp` reported OfflineHighQuality seam click at `-16.814012`
  dBFS despite improving draft `-4.908878` dBFS. Rejected alternatives:
  pitch error was already below one cent, loop fixed-ratio click was already
  silent on measured joins, stereo image deltas were near zero, and sustained
  coherence already improved draft.
- 2026-07-07: implemented offline dynamic-ratio segment-boundary smoothing for
  linked stereo and dynamic-ratio pitch paths. Output length and deterministic
  behavior stayed stable. The selected synthetic seam metric improved to
  `-240.000000` dBFS with no new higher-priority regression in the comparison
  report.
- 2026-07-07: reran the report with the no-listening FMA review seed. Real
  source coverage was present (`operator_listening_sources=10`,
  `missing_assets=0`), but it still provided no subjective or decoded
  real-audio quality metric. Synthetic comparison had no regressions or
  inconclusive rows. External rendered-output comparison could not be collected
  locally because `rubberband`, `ffmpeg`, and `sox` were not installed.
- 2026-07-07: rejected a narrow compression transient-reset change. Enabling
  transient phase resets for ratios below 1.0 changed synthetic extreme-ratio
  transient smear at `0.5x` and `0.75x` from OfflineHighQuality `8`/`6` frames
  back to the draft `1024` frame value, so no DSP patch landed.
- 2026-07-07: added opt-in decoded listening-source profiling to
  `stretch-corpus-report`. The FMA review seed now emits ten decoded MP3 source
  rows with sample rate, channel count, analyzed frames, peak/RMS,
  zero-crossing rate, and transient density. This is source-profile evidence,
  not a stretch-quality verdict.
- 2026-07-07: added opt-in decoded real-source stretch metric rows to
  `stretch-corpus-report`. `--measure-decoded-stretch` now compares the draft
  phase-vocoder baseline with OfflineHighQuality on bounded decoded local
  excerpts for timing drift and transient smear across each listening-source
  case ratio. This creates real-audio objective evidence before any listened
  curation or external comparator render is available.
- 2026-07-07: extended decoded real-source transient-smear rows with input,
  output, matched, and missed transient counts for draft and OfflineHighQuality.
  This separates true attack widening from failed transient matching before the
  next DSP change is selected.
- 2026-07-07: regenerated the local FMA review-seed metric report with
  transient match detail. OfflineHighQuality had no decoded real-source
  regressions. The high `1024`-frame transient-smear penalties were all tied to
  missed transient matches, not finite measured attack widening; the 30
  transient rows recorded 95 matched and 28 missed OfflineHighQuality transient
  matches. The next DSP choice should therefore start from transient
  alignment/detection evidence, not a blind compression transient-reset patch.
- 2026-07-07: added bounded transient-alignment diagnostics to decoded
  real-source transient rows. The regenerated FMA review-seed report still had
  15 improved, 42 unchanged, 3 inconclusive, and 0 regressed metric rows.
  OfflineHighQuality matched 95 transients and missed 28. Matched timing errors
  stayed within the 1024-frame search tolerance; the largest missed-nearest
  distances were far outside it, including a bass `1.25x` row with mean
  `83808` frames and max `87488` frames. This points to sparse or shifted
  output transient detection on specific material windows, not simple attack
  widening on matched events.
- 2026-07-07: extended the alignment diagnostics with event-position fields for
  the largest missed-nearest distance in each transient row. The worst local
  FMA review-seed row is `stretch:bass` at `1.25x` on `000384.mp3`: the expected
  output transient frame was `1600`, while the nearest detected output transient
  was `89088`. Other high misses land tens of thousands of frames away. This
  narrows the next question to early-window event preservation/detection before
  another DSP algorithm change.
- 2026-07-07: added capped `decoded_transient_alignment_event` rows for the
  largest missed transient events per backend and ratio. The local FMA review
  seed emitted 80 event rows: 58 draft and 22 OfflineHighQuality. The top two
  OfflineHighQuality misses are both `stretch:bass` `1.25x` on `000384.mp3`
  early events: input frame `1280` expects output frame `1600` but nearest
  detected output is `89088`, and input frame `7168` expects `8960` but again
  lands nearest to `89088`. This makes the next investigation concrete:
  preserve or detect early output attacks in that source window.
- 2026-07-07: added peak/RMS window probes to
  `decoded_transient_alignment_event` rows. For the worst `000384.mp3`
  `stretch:bass` `1.25x` early miss, the expected output window at frame `1600`
  still has peak `0.144759` and RMS `0.033492` against input-window peak
  `0.163390` and RMS `0.036949`. The second early miss at expected frame `8960`
  also retains substantial energy: expected-output peak `0.274770` and RMS
  `0.114111` against input peak `0.372374` and RMS `0.127199`. This points to
  transient detection/alignment classification before output-energy
  preservation for the next batch.
- 2026-07-07: added event-level expected-output energy classification to
  `decoded_transient_alignment_event` rows. In the regenerated local FMA review
  seed, 78 of 80 missed-event rows classified as `ExpectedEnergyPresent`; the
  other 2 were `ExpectedEnergyWeak`. All 22 OfflineHighQuality missed-event rows
  were `ExpectedEnergyPresent`. The worst `000384.mp3` bass `1.25x` miss has
  expected-output peak ratio `0.885974` and RMS ratio `0.906419`, so the output
  window preserves energy but the current detector/matcher does not accept it as
  a corresponding transient.
- 2026-07-07: added detector-shape diagnostics to missed-event rows. The local
  FMA review seed classified 64 missed events as `CombinedBelowThreshold`, 14 as
  `FluxBelowThreshold`, and 2 as `NotLocalMaximum`; OfflineHighQuality split was
  18, 3, and 1 respectively. The worst `000384.mp3` bass `1.25x` miss now
  reports expected energy score `0.653325`, flux score `1.685165`, and combined
  score `2.338490`, below the detector's `3.0` combined threshold and `2.0`
  flux threshold. The next change should refine real-source transient scoring or
  thresholding, not alter stretch synthesis.
- 2026-07-07: added report-only detector threshold experiment fields to
  `decoded_transient_alignment_event` rows. The candidate policy uses combined
  score `2.0`, flux score `1.5`, and the same local-maximum rule; it does not
  change synthesis or the production detector. On the regenerated local FMA
  review seed, current detector classes remained 64 `CombinedBelowThreshold`,
  14 `FluxBelowThreshold`, and 2 `NotLocalMaximum`. Candidate classes were 46
  `CombinedBelowThreshold`, 7 `FluxBelowThreshold`, 11 `NotLocalMaximum`, and
  16 `DetectorWouldPass`; OfflineHighQuality recovered 6 of 22 missed-event rows
  as candidate passes. The worst `000384.mp3` bass `1.25x` miss would pass the
  candidate policy with combined margin `0.338490`, flux margin `0.185165`, and
  positive local margins. Synthetic report validation stayed clean with 27
  comparisons, 13 improved, 0 regressed, and 0 inconclusive. This is useful
  evidence, but threshold relaxation alone is not enough to promote because most
  missed real-source events still fail candidate scoring or local-max checks.
- 2026-07-07: added a shared transient detector policy gate. The production
  detector entry point still uses the current `3.0` combined and `2.0` flux
  thresholds, while `detect_stretch_transients_with_policy` exposes explicit
  production and candidate-review policies for evidence runs. Focused synthetic
  tests now prove default-policy parity, keep the candidate policy quiet on
  plain sustain, and cover a masked softened attack that production misses but
  the candidate policy recovers. This turns the threshold experiment into a
  guarded review path without changing production transient metrics or stretch
  synthesis.

## Next Task

Do not add a multiresolution or hybrid path until measured evidence requires
it. Next useful work is to run the policy gate over the decoded FMA seed and
compare production versus candidate transient-smear matching directly, not just
event-row classes. If candidate matching improves without synthetic false
positives, decide whether to promote threshold relaxation or move to scoring
normalization/local-max handling. Listened real-source curation and external
rendered-output comparison remain promotion inputs.
