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
- [x] add a multiresolution or hybrid STFT/time-domain path when the measured
  weakness justifies it
- [x] improve transient anchoring or local time-domain splice behavior for
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

- [x] add the next engine path only when a measured weakness requires it
- [x] keep the lower-complexity path available for regression comparison

### Batch 22.4 - Selector Promotion Integration

- [x] carry the new `OfflineHighQualityPath` option through cache identity and
  render/export/freeze artifact planning before any consumer uses it
- [x] require the broad FMA/Rubber Band selector-path evidence in promotion
  receipts
- [x] keep the default OfflineHighQuality path unchanged until selector use is
  explicitly selected by artifact policy

### Batch 22.5 - Matched-Width Residual Isolation

- [x] reproduce the focused `000900.mp3` pads/sustains case with the existing
  Rubber Band render pack
- [x] separate matched-event width evidence from missing-transient penalties in
  decoded reports
- [x] run bounded broad FMA evidence so matched-width residuals are visible
  outside the focused `000900.mp3` case
- [x] add report-only expansion short-window evidence before changing
  production DSP
- [x] design a report-only non-oracle expansion selector gate before any
  production DSP routing change
- [x] validate expansion selector evidence at full-frame/broader comparator
  depth before production routing change

### Batch 22.6 - Sustained And Polyphonic Coherence

- [x] use the full external feature-review rows and decoded reports to select
  one dense sustained or polyphonic residual target
- [x] separate level/gain differences from actual residual coherence before any
  DSP change
- [x] implement one report-only vertical-coherence prototype only when the
  selected metric is not mostly explained by level
- [x] keep the promoted expansion selector fixed unless new selector evidence
  explicitly supports another route
- [x] define a report-only sustained-coherence candidate gate from the first
  regression review
- [x] validate the sustained-coherence candidate gate on broader material
  before any production promotion
- [x] test a product-observable selector probe before any cache/render-plane
  routing change
- [x] test one selector-free second vertical-coherence DSP candidate before
  changing product routing
- [x] test one envelope-preserving vertical-coherence DSP candidate before
  changing product routing
- [x] test one ratio-scoped long-window transient-reset candidate before
  changing product routing
- [x] test one frame-stability adaptive phase-locking candidate before
  changing product routing
- [x] test one tracked peak-region phase-locking candidate before changing
  product routing
- [x] test one stable-frame magnitude-evolution candidate before changing
  product routing
- [x] keep the long-window sustained-coherence path benchmark-only unless a
  better product-observable selector or DSP candidate emerges

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
- 2026-07-07: added policy-aware transient-smear measurement and decoded report
  fields for full-candidate matching plus candidate-output-only matching. On the
  regenerated local FMA review seed, OfflineHighQuality production matching had
  123 input transients, 95 matches, and 28 misses. Applying the candidate policy
  to both input and output raised input candidates to 420 and matches to 352,
  but worsened misses to 68 across 25 of 30 rows. Holding production input
  events fixed and applying candidate detection only to output raised matches to
  119 and cut misses to 4, improving missed counts on 11 rows with no worsened
  rows. The worst `000384.mp3` bass `1.25x` row improved from 1 match and 2
  misses to 3 matches and 0 misses, with candidate-output max smear `4` frames.
  This rejects a global threshold drop but supports output-side scoring or
  normalization work for energy-present misses.
- 2026-07-07: added a stricter output-recovery measurement that keeps
  production input and production output detection as the primary path, then
  tries candidate output detection only for production misses. The regenerated
  local FMA review seed kept OfflineHighQuality matches at 119 and misses at 4,
  the same recovery count as candidate-output replacement, but avoided the one
  max-smear worsening: recovery max-smear rows improved 8, stayed same 22, and
  worsened 0. Draft recovery also improved 20 of 30 rows, moving from 41
  matches/82 misses to 88 matches/35 misses. This is the better prototype
  shape: targeted output recovery instead of replacing output detection or
  lowering input thresholds globally.
- 2026-07-07: added decoded `decoded_transient_recovery_gate` summary rows with
  explicit thresholds: at least 1 recovered miss, 0 missed-row worsens, 0
  max-smear worsens, and global candidate input ratio no higher than `2.0`.
  The regenerated local FMA review seed reports `offline_hq target_status=Pass`
  and `global_threshold_status=Rejected`, recommending `TargetedOutputRecovery`.
  OfflineHighQuality recovered 24 misses, had 0 recovery missed-row worsens, 0
  max-smear worsens, and showed full-candidate input pressure of `3.414634`
  against the `2.0` global-threshold limit. This promotes targeted recovery as
  the only detector-policy candidate worth considering; global threshold
  relaxation remains rejected.
- 2026-07-07: ran the same recovery gate on a broader no-listening FMA seed
  with 20 sources, four per family. The broader decoded report covered 60
  transient-smear rows. OfflineHighQuality still passed targeted recovery and
  still rejected global threshold relaxation: production had 207 input
  transients, 152 matches, and 55 misses; recovery had 196 matches, 11 misses,
  and 44 recovered misses; missed-row worsens stayed 0, max-smear worsens stayed
  0, and full-candidate input pressure rose to `3.681159` against the `2.0`
  limit. This satisfies the broader decoded-source stability condition for a
  production metric-policy decision.
- 2026-07-07: promoted targeted output recovery into the production
  transient-smear metric policy. The public metric now keeps production input
  and primary output detection, then applies candidate-review output detection
  only to production misses. The decoded report keeps strict production-only
  fields for the recovery gate and comparison evidence. On the broader FMA seed,
  the recovery gate stayed unchanged: OfflineHighQuality recovered 44 misses,
  had 0 missed-row worsens, 0 max-smear worsens, and still rejected global
  threshold relaxation at `3.681159` input pressure. The promoted decoded metric
  now reports 40 improved, 10 unchanged, 9 inconclusive, and 1 regressed
  transient-smear row. The regression is `stretch:pads_sustains` on
  `000900.mp3` at `0.75x`, where draft max smear is `2` frames and
  OfflineHighQuality max smear is `13` frames after both match all 5 production
  input transients.
- 2026-07-07: added max-matched-smear event diagnostics to the transient-smear
  measurement and decoded report. The broader FMA diagnostic report kept the
  same promoted-metric shape: 40 improved, 10 unchanged, 9 inconclusive, and 1
  regressed transient-smear row. The regressed row is a real matched-event
  width issue, not a missing-transient penalty: both draft and OfflineHighQuality
  match all 5 promoted input transients on `stretch:pads_sustains` `000900.mp3`
  at `0.75x`. The worst event is input frame `153344`; draft maps it to output
  frame `115456` and widens `7` input frames to `9` output frames, while
  OfflineHighQuality maps it to output frame `115200` and widens `7` input
  frames to `20` output frames.
- 2026-07-07: added decoded compression phase-lock ablation summary rows. On
  the broader FMA seed, compression rows show current OfflineHighQuality
  phase-locking is better than independent-bin draft on 10 rows, independent
  bins are better on 1 row, 6 rows are unchanged, and 3 are inconclusive.
  Finite-row mean max-smear is also lower for phase locking: `374.352941`
  frames versus independent-bin `668.705882` frames. The only finite regression
  remains `stretch:pads_sustains` `000900.mp3` at `0.75x` with an `11`-frame
  delta. This rejects a global compression rollback to independent bins.
- 2026-07-07: added a report-only guarded local transient-width control
  candidate for compression rows. The first unguarded shoulder limiter was
  rejected locally because it improved 7 rows but worsened one vocal row by
  `934` frames and raised mean max-smear. The retained candidate keeps the
  postprocess output only when promoted transient-smear does not worsen and
  missed matches do not increase. On the broader FMA seed it improved 7
  compression rows, worsened 0, left 10 unchanged, and left 3 inconclusive.
  Finite-row mean max-smear moved from current OfflineHighQuality `374.352941`
  frames to candidate `367.470588` frames. The best candidate improvement was
  `41` frames on `stretch:drums_percussion` `000002.mp3` at `0.75x`; the worst
  candidate regression and worst draft regression were both `0`, so the
  `000900.mp3` finite-smear regression is removed under the guarded candidate.
- 2026-07-07: added edit-pressure diagnostics to the guarded width-control
  candidate summary. The broader FMA seed still shows the same metric shape, but
  the candidate edits 11 compression rows and 290 samples. The largest sample
  delta is `0.422849804` on `stretch:full_mix` `000144.mp3` at `0.75x`; the
  largest added adjacent-step delta is `0.309232593` on `stretch:bass`
  `000236.mp3` at `0.75x`. This is too much unmanaged edit pressure to promote
  into reusable OfflineHighQuality DSP without window-level inspection or
  listening/external comparison evidence. Keep it report-only for now.
- 2026-07-07: added bounded edit-event rows for the guarded width-control
  candidate's highest-pressure windows. The broader FMA seed still reports 7
  better candidate rows, 0 current-better rows, 10 unchanged rows, and 3
  inconclusive rows; finite-row mean max-smear remains `367.470588` candidate
  versus `374.352941` current. The max sample-delta event is `stretch:full_mix`
  `000144.mp3` at `0.75x`: source frame `112857.333333`, output frame `84643`,
  sample delta `0.422849804`, added adjacent-step delta `0.000000000`, peak
  unchanged at `0.966210067`, and RMS reduced from `0.268733651` to
  `0.262553828`. The max added-step event is `stretch:bass` `000236.mp3` at
  `0.75x`: source frame `45064.000000`, output frame `33798`, sample delta
  `0.321874976`, added adjacent-step delta `0.309232593`, peak unchanged at
  `1.473660707`, RMS reduced from `0.504574776` to `0.501747207`, and adjacent
  step raised from `0.006321192` to `0.315553784`. The full-mix row does not
  look like a new step edge by this metric, but the bass row does. Keep the
  candidate report-only and measure an edit-pressure gate before any DSP
  promotion.
- 2026-07-07: added a report-only conservative edit-pressure gate for the
  guarded width-control candidate. The gate limits row promotion to
  `max_abs_sample_delta <= 0.250000000` and
  `max_added_adjacent_step_delta <= 0.050000000`. On the broader FMA seed, the
  raw candidate still improves 7 rows, worsens 0, leaves 10 unchanged, and
  leaves 3 inconclusive, but the gate accepts 10 rows, rejects 10 rows, and
  rejects all 7 candidate-improved rows. Gated evidence therefore has 0 better
  rows, 0 current-better rows, 17 unchanged rows, and 3 inconclusive rows;
  finite-row mean max-smear returns to current OfflineHighQuality exactly:
  `374.352941` gated versus `374.352941` current. The rejected candidate
  improvement total is `117.000000` frames. This rejects the current
  sample-edit width-control candidate as a promotable DSP path under
  conservative edit-pressure limits.
- 2026-07-07: added a report-only compression transient-anchor review path with
  stricter compression phase-reset thresholds than the expansion reset path. On
  the broader FMA seed, the candidate improved 4 compression rows, worsened 3,
  left 10 unchanged, and left 3 inconclusive. Finite-row mean max-smear moved
  the wrong way: `374.882353` candidate versus `374.352941` current. The best
  improvement was `986.000000` frames on `stretch:bass` `000236.mp3` at
  `0.75x`, but the worst regression was `1011.000000` frames on the exact
  residual target, `stretch:pads_sustains` `000900.mp3` at `0.75x`; worst
  draft regression there was `1022.000000` frames. This rejects simple
  compression phase-reset anchoring as the next OfflineHighQuality path.
- 2026-07-07: added a report-only shorter-window OfflineHighQuality review
  candidate for compression rows (`1024` window, `256` hop) and generalized the
  compression candidate summary so each candidate reports what it does on the
  current baseline's worst draft-regression row. On the broader FMA seed, the
  shorter-window candidate improved 8 compression rows, worsened 4, left 5
  unchanged, and left 3 inconclusive. Finite-row mean max-smear improved
  substantially: `133.941176` candidate versus `374.352941` current. It did
  not fix the target residual row: `stretch:pads_sustains` `000900.mp3` at
  `0.75x` stays draft `2.000000`, current `13.000000`, candidate `13.000000`.
  The worst new candidate regression is `38.000000` frames on
  `stretch:drums_percussion` `000002.mp3` at `0.75x`, where worst draft
  regression is `36.000000` frames. This rejects a global shorter-window switch,
  but it keeps multiresolution selection alive as a candidate because the mean
  improvement is large and localized regressions are measurable.
- 2026-07-07: added report-only `decoded_compression_short_window_feature`
  rows for the shorter-window candidate's finite win/loss cases. The broader
  FMA seed emitted 12 feature rows: 8 `CandidateBetter` and 4 `CurrentBetter`.
  A simple non-oracle gate is visible but not yet implemented: choose the
  shorter window when current OfflineHighQuality has missed promoted transients
  or very high current finite smear. `current_missed_transients > 0` captures 4
  high-impact wins and no regressions; adding `current_smear_frames >= 64`
  captures the `stretch:vocals` `000020.mp3` `0.75x` win. Together those five
  cases account for `4109` frames of improvement and avoid all four
  current-better rows in this seed. The mild remaining wins overlap the
  current-better feature space and should not drive the first selector. The
  worst rejected regression remains `stretch:drums_percussion` `000002.mp3`
  at `0.75x`, where current has no misses and moderate current smear
  (`43.000000` frames) while the shorter window widens the max matched output
  event from `71.000000` to `109.000000` frames.
- 2026-07-07: added a report-only gated shorter-window selector candidate.
  The gate chooses the shorter-window path only when current OfflineHighQuality
  has at least one promoted missed transient or current max smear is at least
  `64.000000` frames. On the broader FMA seed, the gate accepted 7 compression
  rows and rejected 13. It retained 5 of the 8 raw shorter-window wins,
  rejected all 4 raw shorter-window regressions, and rejected 3 mild wins worth
  `26.000000` frames total. Gated rows improved 5, worsened 0, left 12
  unchanged, and left 3 inconclusive. Finite-row mean max-smear improved from
  current OfflineHighQuality `374.352941` frames to gated `132.647059` frames,
  with `0.000000` worst gated regression. The known `000900.mp3` pads/sustains
  draft regression remains unchanged at `11.000000` frames because neither the
  global shorter-window path nor the gate solves that matched-event width case.
- 2026-07-07: added the first report-only external-comparator quality harness.
  `stretch-corpus-report` now accepts `--measure-external-benchmark-quality`
  alongside operator listening sources and `--external-benchmark-render CASE
  RATIO WAV`. It does not invoke or depend on Rubber Band, elastique, or any
  comparator code. It only reads rendered WAV output supplied by the operator,
  renders Signal OfflineHighQuality from the same decoded source excerpt, and
  emits `external_benchmark_quality` rows with timing drift, transient smear,
  sample-rate compatibility, alignment lag, aligned correlation, RMS error,
  peak error, and rendered-output-only source-boundary evidence. This gives the
  stretch program a clean black-box comparison surface before promotion.
- 2026-07-07: added external-comparator render-pack ingestion. Local comparator
  tools were not available on this machine, so no Rubber Band or elastique
  renders were generated here. Instead, `stretch-corpus-report` now accepts
  `--external-benchmark-render-manifest TSV` with `case_id`, `ratio`, and
  `rendered_path` or `path`, plus optional `tool_name` or `tool`. This lets an
  operator-supplied Rubber Band or elastique render pack feed the same
  report-only quality harness without expanding the command line into repeated
  render flags.
- 2026-07-07: added external-comparator pack export. Since local comparator
  tools are still absent, `stretch-corpus-report` now accepts
  `--export-external-benchmark-pack DIR`. It decodes bounded operator listening
  source excerpts to `DIR/sources/*.wav`, creates `DIR/renders/`, and writes
  `DIR/external-benchmark-render-plan.tsv` with `case_id`, `ratio`,
  `source_wav`, `rendered_path`, and `tool_name`. The same TSV can be supplied
  back through `--external-benchmark-render-manifest` after an external
  renderer fills the planned output WAV paths. This keeps the comparator path
  clean-room and rendered-output-only while making the next operator step
  deterministic.
- 2026-07-07: exercised the pack exporter on the broad FMA review seed with
  `rubberband-cli` as the planned tool label. The generated target-only pack at
  `target/stretch-corpus-external-benchmark-pack-fma-broad` contains 20 decoded
  source WAV excerpts and 60 render-plan rows in
  `external-benchmark-render-plan.tsv`. `renders/` is intentionally empty
  locally because no comparator renderer is installed here.
- 2026-07-07: added a tolerant external render-plan readiness check. The report
  now accepts `--check-external-benchmark-render-plan TSV` and emits
  `external_benchmark_render_plan_status` plus capped missing-render rows
  without loading external DSP code or requiring the rendered WAVs to exist.
  Running it against the generated broad FMA pack reports `planned_rows=60`,
  `present_rows=0`, `missing_rows=60`, and `invalid_rows=0`, which confirms the
  pack is correctly planned but not yet rendered.
- 2026-07-07: made missing render-plan rows self-contained. Each capped
  `external_benchmark_render_plan_missing` row now includes `source_wav`,
  `rendered_path`, and `tool`, so the readiness report can directly drive an
  external render script or manual render pass without looking back into the
  TSV. The broad FMA pack still reports 60 missing rendered WAVs locally.
- 2026-07-07: filled the broad FMA external comparator pack with Rubber Band
  CLI `4.0.0` R3 output as a black-box rendered-output benchmark. The render
  plan readiness report is now complete: 60 planned rows, 60 present rows, 0
  missing rows, and 0 invalid rows. `stretch-corpus-report` also now preserves
  `source_wav` from render manifests for external quality rows and skips
  ambiguous case-only matches, fixing the duplicate-case source pairing issue
  in broad manifests.
- 2026-07-07: regenerated the broad FMA decoded stretch and Rubber Band R3
  quality report with corrected source pairing. The report measured 60 external
  rows and skipped 0. Gated shorter-window selector evidence still held:
  7 accepted compression rows, 13 rejected rows, 5 gated-better rows,
  0 current-better rows, finite-row mean max-smear `132.647059` frames versus
  current OfflineHighQuality `374.352941`, and
  `worst_gated_regression_delta_frames=0.000000`. Against Rubber Band R3,
  timing drift delta was `0` samples on every measured row. The transient-smear
  proxy favored Signal in 32 rows, Rubber Band in 7 rows, and tied 21 rows;
  mean delta was `-189.633333` frames where negative means Signal measured
  lower smear. The worst Signal transient-smear regressions were bounded:
  `stretch:full_mix` `000237.wav` at `1.5x` by `32` frames,
  `stretch:drums_percussion` `000002.wav` at `1.5x` by `27` frames, and
  `stretch:bass` `000236.wav` at `1.25x` by `22` frames. Direct waveform
  similarity remains weak as a parity signal: mean aligned correlation was
  `0.329881`, mean aligned RMS-error ratio was `1.319973`, and the largest RMS
  divergence was `stretch:pads_sustains` `000870.wav` at `1.5x` with ratio
  `2.246718`. Treat that as an inspection queue for phase/envelope/perceptual
  metrics, not as a sample-difference pass/fail verdict.
- 2026-07-07: added `external_benchmark_feature_delta` rows beside the existing
  black-box quality rows. The new report-only row measures aligned envelope
  correlation, RMS and peak deltas, zero-crossing-rate delta, spectral-centroid
  delta, high-frequency energy-ratio delta, and a bounded divergence score.
  It uses only Signal output and rendered comparator WAVs; it does not invoke
  or link comparator DSP. The regenerated broad Rubber Band R3 report emitted
  60 feature rows. Mean divergence score was `0.744393`, max was `1.459699`,
  minimum envelope correlation was `0.541594`, mean absolute RMS delta was
  `1.935619` dB, mean absolute peak delta was `2.241843` dB, mean absolute
  spectral-centroid delta was `179.261112` Hz, and mean absolute high-frequency
  energy-ratio delta was `0.001997`. The top feature-divergence rows were
  `stretch:pads_sustains` `000870.wav` at `1.5x` (`score=1.459699`,
  `rms_delta_db=6.716807`), `stretch:full_mix` `000190.wav` at `1.25x`
  (`score=1.292193`, `rms_delta_db=4.722903`), and `stretch:pads_sustains`
  `000870.wav` at `0.75x` (`score=1.254252`, `rms_delta_db=5.137926`).
  This points the next listening or metric pass toward gain/envelope behavior
  on sustained/polyphonic material before treating low sample correlation as a
  phase-vocoder failure by itself.
- 2026-07-07: added capped `external_benchmark_gain_envelope_review` rows for
  the top feature-divergence comparator cases. The row ranks the worst feature
  deltas and measures 4096-frame windowed RMS deltas over the aligned Signal
  and comparator slices. The regenerated broad Rubber Band R3 report emitted
  8 review rows: all 8 classified as `SignalConsistentlyLouder`, with
  `louder_windows=31`, `quieter_windows=0`, and `near_windows=0` in every row.
  The top row remains `stretch:pads_sustains` `000870.wav` at `1.5x`:
  feature score `1.459699`, whole-slice RMS delta `6.716807` dB, mean window
  RMS delta `7.009287` dB, median `6.470331` dB, and max absolute window delta
  `11.142284` dB. The next two rows were `stretch:full_mix` `000190.wav` at
  `1.25x` with median window delta `2.520436` dB and
  `stretch:pads_sustains` `000870.wav` at `0.75x` with median window delta
  `5.175870` dB. This makes the next DSP question narrower: determine whether
  OfflineHighQuality is over-retaining energy relative to source intent,
  Rubber Band is applying protective gain, or the current comparison should
  loudness-normalize before judging phase/envelope quality.
- 2026-07-07: added capped `external_benchmark_level_normalized_review` rows
  for the same top feature-divergence cases. The review applies a report-only
  matched-RMS gain to Signal output, leaves raw comparator rows unchanged, and
  emits before/after divergence fields. The regenerated broad Rubber Band R3
  report emitted 8 normalized review rows: 6 classified as
  `MostlyLevelExplained` and 2 as `LevelReducesDivergence`. Mean raw feature
  score was `1.191988`; mean normalized score was `0.517920`; mean score delta
  was `-0.674068`; mean applied Signal gain was `-4.110853` dB. The top
  `pads_sustains` `000870.wav` `1.5x` row dropped from `1.459699` to
  `0.428825` after applying `-6.716807` dB, with normalized RMS delta
  `0.000000` dB and unchanged envelope correlation `0.639086`. This says most
  of the worst comparator feature gap is level-related, while residual envelope
  correlation still needs a non-level quality check before any DSP gain change.
- 2026-07-07: added capped `external_benchmark_residual_coherence_review` rows
  for the level-normalized top cases. The row measures block-RMS envelope
  correlation, mean/max absolute block RMS delta, and STFT magnitude coherence
  after matched-RMS normalization. The regenerated broad Rubber Band R3 report
  emitted 8 residual rows: 5 classified as `MostlyPhaseOrFineTextureResidual`
  and 3 as `MixedResidualCoherence`. Mean block-RMS envelope correlation was
  `0.992373`, mean spectral magnitude coherence was `0.911277`, mean absolute
  block RMS delta was `0.867780` dB, and max absolute block RMS delta was
  `4.425477` dB. The worst raw row, `pads_sustains` `000870.wav` at `1.5x`,
  still landed as mixed residual because spectral magnitude coherence was
  `0.816178`, but block-RMS envelope correlation was high at `0.975399`. This
  argues against a DSP gain/envelope correction as the next move. The measured
  residual is mostly phase/fine-texture or mild spectral-magnitude difference
  after level matching.
- 2026-07-07: promoted the gated shorter-window selector into an explicit
  opt-in `OfflineHighQualityPath::CompressionShortWindowSelector` path.
  `OfflineHighQualityStretcher::new()` still routes to the default
  OfflineHighQuality engine, so raw comparator evidence is not silently
  changed. The selector reuses the measured gate:
  `min_current_misses=1` or `min_current_smear_frames=64.000000`, and only
  applies below `1.0x`. The broad FMA/Rubber Band R3 refresh emitted the same
  60 measured external quality rows, 60 feature-delta rows, and 8 rows each for
  gain-envelope, level-normalized, and residual-coherence review. The new
  `decoded_compression_short_window_selector_path` line proved the promoted
  renderer matched the report-only gate on all 20 compression rows:
  `selected_short_window_rows=7`, `selected_default_rows=13`,
  `output_match_rows=20`, `output_mismatch_rows=0`, `smear_match_rows=20`,
  `smear_mismatch_rows=0`, and `max_abs_smear_delta_frames=0.000000`. The
  candidate evidence remained unchanged: 5 gated-better rows, 0 current-better
  rows, finite mean max-smear `132.647059` frames versus current
  `374.352941`, and `worst_gated_regression_delta_frames=0.000000`.
- 2026-07-07: carried `OfflineHighQualityPath` through cache identity and
  render/export/freeze artifact planning. `StretchCacheIdentityInput` now
  records `offline_path`, the canonical key includes `offline_path=...`, and
  the cache schema advanced to `signal-stretch-cache-v2` so default and
  selector artifacts cannot collide. Render-plane plans and materialization
  receipts now expose the path, runtime observation snapshots carry the path
  for plans, materializations, and cache decisions, and host-local forwards the
  render-plane receipt path into runtime. Static stereo selector
  materialization is allowed and uses the opt-in stretcher path. Dynamic-ratio
  and pitch-shift selector artifact materialization are explicitly rejected
  until those combinations have their own corpus evidence, so consumers cannot
  accidentally request selector cache keys backed by default-path PCM.
- 2026-07-07: made stretch promotion receipts path-aware. Default synthetic
  OfflineHighQuality receipts now authorize only `OfflineHighQualityPath::Default`.
  `accepted_compression_short_window_selector(...)` records the broad
  FMA/Rubber Band selector-path evidence for
  `OfflineHighQualityPath::CompressionShortWindowSelector`, and render-plane
  plus runtime plan snapshots now gate product-facing use through
  `accepts_product_facing_path(...)` / `product_facing_path_blocker(...)`.
  Selector artifact materialization rejects default-path receipts and still
  rejects dynamic-ratio or pitch-shift selector combinations until those paths
  have separate evidence.
- 2026-07-07: reopened the `000900.mp3` pads/sustains residual as Batch 22.5
  and added a report-only `decoded_matched_transient_width_review` row. The
  focused local run used one FMA source and the existing Rubber Band render pack
  at `target/stretch-corpus-fma-000900-width-followup.tsv`. The default decoded
  0.75x headline transient-smear metric is still dominated by one missed
  transient penalty (`offline_hq=1024.000000`), while the matched-event residual
  is `13.000000` frames from input width `7.000000` to output width
  `20.000000`. The selector path selected the short-window output and matched
  the selector gate, but `decoded_matched_transient_width_review` showed
  selector and default matched-width results were identical across the three
  `000900` ratios (`selector_same_as_offline_rows=3`). Against the Rubber Band
  rendered WAV excerpt at 0.75x, Signal timing drift stayed at `0` samples and
  the transient-smear proxy favored Signal (`13.000000` frames versus
  Rubber Band `25.000000`), so this case is a Signal-vs-draft matched-width
  residual, not clear Rubber Band underperformance by that proxy.
- 2026-07-08: ran the broad FMA decoded matched-width evidence with a bounded
  `120000`-frame per-source analysis limit after the full broad external
  comparator run and the full-frame decoded run were too slow for this batch.
  The bounded run covered 20 operator FMA sources and 60 ratio rows with no
  missing assets. `decoded_matched_transient_width_review` found 24 finite
  matched-width rows: OfflineHighQuality was worse than draft on 13 rows,
  better on 8, and equal on 3. The worst residual moved from the focused
  `000900.mp3` case to `stretch:bass` `000236.mp3` at `1.25x`, where
  OfflineHighQuality widened a matched event from input width `48.000000` to
  output width `97.000000` (`49.000000` frames smear), while draft's matched
  width was `17.000000` frames but draft also missed 3 of 4 input transients.
  The compression selector did not address the expansion residual:
  `selector_same_as_offline_rows=20`, `selector_better_than_offline_rows=1`,
  and `selector_worse_than_offline_rows=3`.
- 2026-07-08: added report-only expansion short-window candidate evidence
  using the same short-window renderer already used by the compression review
  path. On the bounded broad FMA run, `decoded_expansion_short_window_candidate`
  covered 40 expansion rows, 30 finite: candidate was better on 15 rows,
  current was better on 7, 8 were unchanged, and 10 were inconclusive. Mean
  finite smear fell from current `181.533333` frames to candidate `44.333333`
  frames. The best improvement was `1018.000000` frames on `stretch:bass`
  `000384.mp3` at `1.25x`; the worst candidate regression was `18.000000`
  frames on `stretch:bass` `000236.mp3` at `1.5x`. The current worst
  Signal-vs-draft residual, `000236.mp3` at `1.25x`, worsened under the
  short-window candidate by `15.000000` frames, so a global expansion
  short-window switch is rejected. The useful next shape is a report-only
  selector gate, not production routing.
- 2026-07-08: added the report-only
  `decoded_expansion_short_window_selector_candidate` gate. The gate selects
  the short-window expansion path only when current OfflineHighQuality misses
  at least one promoted transient or when current matched-smear is worse than
  draft. On the bounded 120k broad FMA run, it accepted 6 of 40 expansion rows
  and rejected 34. It retained 5 candidate-better rows, selected 0
  current-better rows, and reported `worst_gated_regression_delta_frames=0`.
  Mean finite smear moved from current `181.533333` frames to gated
  `47.833333` frames. The best retained win stayed `1018.000000` frames on
  `stretch:bass` `000384.mp3` at `1.25x`. The gate rejected 10 additional
  candidate-better rows worth `148.000000` frames of possible improvement, but
  it also rejected all 7 current-better rows, including both `000236.mp3`
  current-better cases. This is a viable report-only selector shape; it still
  needs full-frame and comparator-backed evidence before production routing.
- 2026-07-08: ran the targeted default-limit expansion selector pack at
  `target/stretch-corpus-fma-expansion-selector-full.tsv`. The pack covered 6
  FMA sources, 18 ratio rows, and the existing Rubber Band rendered outputs for
  accepted selector wins, rejected current-better guard cases, and rejected
  candidate-better opportunity cases. The raw expansion short-window candidate
  covered 12 finite expansion rows: candidate was better on 8, current was
  better on 4, mean candidate smear was `59.416667` frames, and mean current
  smear was `378.000000` frames. The selector gate accepted 4 of 12 expansion
  rows, selected 0 current-better rows, kept `worst_gated_regression_delta_frames=0`,
  and moved mean gated smear to `54.000000` frames. It rejected 4 additional
  candidate-better rows worth `80.000000` frames of possible improvement. The
  larger analysis window changed important row outcomes versus the bounded
  `120000`-frame run: `000384.mp3` no longer looked like a missed-transient
  win, while `000020.mp3` at `1.5x` became a missed-transient win. The external
  comparator pass measured zero timing drift for all 18 Signal and Rubber Band
  rows; on expansion rows, Signal's transient-smear proxy was better on 6,
  Rubber Band's was better on 2, and 4 were equal. This supports the gate as
  conservative report evidence, but it also shows the heuristic is
  analysis-window sensitive. Do not promote expansion short-window routing to a
  product path yet.
- 2026-07-08: made the broad/default-limit evidence path bounded enough to run.
  `stretch-corpus-report` now keeps full behavior as the default, and adds
  `--decoded-stretch-report-mode expansion-selector` for the Batch 22.5
  decoded evidence path plus `--external-benchmark-quality-mode core` for
  timing/transient/alignment comparator rows without the heavier feature-review
  rows. The external comparator pass also caches decoded source audio and mono
  buffers across ratio rows. A targeted fast rerun at
  `target/stretch-corpus-fma-expansion-selector-fast.tsv` preserved the prior
  targeted selector result and completed in `164.09` seconds; the broader
  20-source/60-render run at
  `target/stretch-corpus-fma-broad-expansion-selector-fast.tsv` completed in
  `518.13` seconds.
- 2026-07-08: ran the 20-source broad/default-limit expansion selector evidence
  with Rubber Band core comparator rows. The raw expansion short-window
  candidate covered 40 expansion rows, 38 finite: candidate was better on 18,
  current was better on 13, 7 were unchanged, and 2 were inconclusive. Mean
  candidate smear was `54.184211` frames versus current `231.315789`. The
  selector gate accepted 9 of 40 rows, selected 0 current-better rows, kept
  `worst_gated_regression_delta_frames=0`, and moved mean gated smear to
  `53.000000` frames. It retained 8 candidate-better rows and rejected 10
  additional candidate-better rows worth `138.000000` frames of possible
  improvement. The Rubber Band core comparator pass measured zero timing drift
  for all 60 rows. On finite expansion transient-smear rows, Signal's proxy was
  better on 23 of 34, Rubber Band's was better on 5, and 6 were equal. This is
  enough broad/default-limit evidence to promote the expansion selector shape
  deliberately; keep full external feature-review rows as a separate quality
  audit, not as a blocker for this selector gate.
- 2026-07-09: promoted the expansion short-window selector into an explicit
  `OfflineHighQualityPath::ExpansionShortWindowSelector` product path. The DSP
  path is expansion-only and keeps the measured gate:
  current OfflineHighQuality must miss at least one promoted transient or
  regress versus the draft transient-smear baseline before it switches to the
  short-window renderer. Promotion receipts now distinguish expansion selector
  evidence from both default OfflineHighQuality and compression selector
  evidence. Cache identity gets a distinct
  `offline_path=ExpansionShortWindowSelector` key, and render-plane
  materialization allows only static stereo artifacts for the expansion
  selector while still rejecting default receipts, dynamic-ratio selector
  artifacts, and pitch-shift selector artifacts.
- 2026-07-09: validated the promoted expansion selector path with focused
  `signal-dsp-stretch` and `signal-render-plane` package tests plus repo
  validation. The path is product-addressable for static stereo artifacts with
  matching expansion-selector promotion evidence; unsupported dynamic-ratio and
  pitch-shift selector combinations remain blocked.
- 2026-07-09: added selector-aware external benchmark reporting so the full
  feature-review rows can compare Rubber Band renders against a specific Signal
  offline path instead of always using default OfflineHighQuality. The report
  now accepts
  `--external-benchmark-signal-path default|compression-short-window-selector|expansion-short-window-selector`
  and tags every external benchmark row with `signal_path=...`.
- 2026-07-09: ran the targeted full external feature-review audit for the
  promoted expansion selector at
  `target/stretch-corpus-fma-expansion-selector-path-full.tsv`
  (`real 165.09`). The audit measured 18 Rubber Band comparator rows, all
  tagged `signal_path=ExpansionShortWindowSelector`. Signal and Rubber Band
  both had zero timing drift on all 18 rows. On the 12 expansion rows, the
  transient-smear proxy favored Signal on 9 rows, Rubber Band on 2, and tied
  on 1. The selector candidate summary stayed conservative: 12 finite
  expansion rows, 4 accepted rows, 8 rejected rows,
  `worst_gated_regression_delta_frames=0.000000`, mean gated smear
  `54.000000` frames versus current `378.000000`. Full feature-review rows
  also make the next quality target visible: the highest divergence is
  dominated by full-mix and bass cases where Signal is consistently louder
  before level normalization, with residual-coherence review rows separating
  level from remaining texture/phase differences. Batch 22.5 is closed; the
  next work should move to sustained/polyphonic coherence rather than more
  expansion-selector evidence.
- 2026-07-09: added dedicated
  `external_benchmark_coherence_target_review` rows to the full external
  benchmark report. The row is report-only and ranks sustained/polyphonic
  candidates after RMS normalization using remaining feature divergence,
  sample-envelope mismatch, block-envelope mismatch, block gain residual, and
  spectral magnitude mismatch. Regenerated the targeted FMA/Rubber Band report
  at `target/stretch-corpus-fma-coherence-target-selection.tsv` (`real 157.56`).
  The top selected target is `stretch:bass` `000236` at `1.25x`
  (`material_scope=BassSustain`, `target_reason=SpectralMagnitudeCoherence`,
  `target_score=1.248686`, normalized divergence `0.488981`, normalized sample
  envelope correlation `0.654801`, mean block RMS residual `1.544930` dB,
  spectral magnitude coherence `0.860039`). The nearest dense-polyphonic
  controls are `stretch:full_mix` `0017` at `0.75x` (`target_score=1.158133`,
  `target_reason=SampleEnvelopeCoherence`) and `1.5x`
  (`target_score=1.140551`, `target_reason=SpectralMagnitudeCoherence`). The
  target is level-separated; do not solve it with comparator gain matching.
- 2026-07-09: added a report-only sustained-coherence candidate path using a
  longer STFT window, wider hop, identity phase locking, and no transient reset.
  The path is exposed only through the full external benchmark review and does
  not alter OfflineHighQuality product routing, cache identity, render-plane
  materialization, the promoted expansion selector, dynamic-ratio rendering, or
  pitch-shift rendering. Regenerated the targeted FMA/Rubber Band report at
  `target/stretch-corpus-fma-coherence-candidate-review.tsv` (`real 181.24`).
  Candidate summary: 15 rows, 9 improved, 0 unchanged, 6 regressed. The selected
  target improved: `stretch:bass` `000236` at `1.25x` moved from target score
  `1.248686` to `0.986865` (`delta=-0.261821`), with spectral magnitude
  coherence `0.970345`, normalized sample-envelope correlation `0.748078`, and
  mean block RMS residual `0.353962` dB. The dense `0017` full-mix controls also
  improved at `0.75x` (`delta=-0.053303`) and `1.5x` (`delta=-0.048220`).
  Regressions remain material: the worst row is `stretch:bass` `000236` at
  `0.75x` (`delta=0.193586`), with additional regressions on full mix, vocals,
  and bass. This is useful evidence for vertical coherence, not a promotable
  product path yet.
- 2026-07-09: tested a report-only sustained-coherence gate named
  `spectral-magnitude-target`, then rejected it as too loose after broad
  evidence. The first gate selected rows whose level-separated residual reason
  was `SpectralMagnitudeCoherence`. It passed the targeted report
  (`target/stretch-corpus-fma-coherence-candidate-gate-review.tsv`,
  `real 181.22`) with 4 selected-improved rows and 0 selected regressions, but
  the broad run selected 9 rows with 7 improvements and 2 regressions. Worst
  selected regression was `stretch:bass` `000441` at `1.5x`
  (`delta=0.138614`), with another selected regression on
  `stretch:pads_sustains` `000870` at `1.5x` (`delta=0.031855`).
- 2026-07-09: replaced the loose gate with
  `spectral-magnitude-material-guard`. The refined report gate still requires
  `SpectralMagnitudeCoherence`, and also rejects `1.5x` or higher bass and
  sustained-polyphonic material. Targeted report:
  `target/stretch-corpus-fma-coherence-candidate-material-guard-targeted.tsv`
  (`real 179.19`) selected 4 rows, all improved, with 0 selected regressions.
  Broad report:
  `target/stretch-corpus-fma-coherence-candidate-material-guard-broad.tsv`
  (`real 576.97`) measured 48 sustained/polyphonic candidate rows: raw
  candidate was 29 improved and 19 regressed; the refined gate selected 6 rows,
  all improved, with 0 selected regressions and
  `worst_selected_regression_delta=0.000000`. It retained the selected bass
  target (`000236` at `1.25x`, `delta=-0.261821`), the strongest broad bass
  win (`000441` at `0.75x`, `delta=-0.810077`), a vocal `1.5x` win, a
  pads/sustains `1.25x` win, and two full-mix `1.5x` wins. It rejected 23
  candidate-better rows, including one guarded pads/sustains `1.5x` win, and
  rejected all 19 current-better rows. This is a good benchmark gate shape, but
  not a DAW-time product selector yet because the residual reason is derived
  from external comparator analysis.
- 2026-07-09: added a report-only product-observable probe named
  `source-character-v1` using Signal-owned character descriptors from the source
  audio: low-band weight, sustain body, rhythmic activity, spectral complexity,
  and descriptor confidence. Regenerated the broad report at
  `target/stretch-corpus-fma-coherence-source-probe-broad.tsv` (`real 703.47`).
  The benchmark gate still selected 6 rows, all improved, with 0 selected
  regressions. The source-character probe selected 0 rows, rejected all 48,
  rejected all 29 candidate-better rows, agreed with the benchmark gate on 42
  rows, and disagreed on the 6 benchmark-selected wins. All 6 disagreements
  were rejected for `LowSourceDescriptorConfidence`; descriptor confidence was
  `0.500000`, and rhythmic activity was high (`0.935000` to `1.000000`) on the
  same wins. Quick threshold checks over the generated fields were not safe:
  broad `complex_sustain` would select 39 rows with 13 regressions, the
  material-guarded variant would select 26 rows with 10 regressions, and a
  low-rhythm variant would still select 3 regressions. Conclusion:
  `source-character-v1` is rejected as a product selector. The long-window
  sustained-coherence path remains benchmark-only evidence.
- 2026-07-09: added a report-only selector-free blend candidate named
  `current-long-window-half-blend`. The path mixes the current selected Signal
  output with the long-window sustained-coherence output at a fixed `0.5`
  candidate weight, preserving output length and determinism without using a
  material selector. Regenerated the targeted report at
  `target/stretch-corpus-fma-coherence-blend-candidate-targeted.tsv`
  (`real 247.29`). The blend was rejected on targeted evidence: 15 rows,
  3 improved, 12 regressed, worst regression `delta=0.715346` on
  `stretch:bass` `000236` at `0.75x`. It kept some selected-target benefit
  (`stretch:bass` `000236` at `1.25x`, `delta=-0.235859`), but the regression
  profile is worse than the raw long-window candidate. No broad run is needed
  for this blend shape.
- 2026-07-09: added a report-only envelope-preserving candidate named
  `long-window-current-envelope-match`. The path renders the long-window
  sustained-coherence candidate, then applies block RMS envelope matching
  against the current selected Signal output. Regenerated the targeted report at
  `target/stretch-corpus-fma-coherence-envelope-candidate-targeted.tsv`
  (`real 243.23`). The envelope candidate is also rejected on targeted
  evidence: 15 rows, 9 improved, 6 regressed, worst regression `delta=0.272430`
  on `stretch:bass` `000236` at `0.75x`. It improved the selected bass target
  more than the raw long-window candidate (`000236` at `1.25x`,
  `delta=-0.458178`), but it did not remove the regression class and made the
  worst targeted regression larger than the raw long-window candidate
  (`0.272430` versus `0.193586`). No broad run is needed for this envelope
  shape.
- 2026-07-09: added a report-only ratio-scoped candidate named
  `expansion-long-window-transient-reset`. The path keeps current
  OfflineHighQuality output for compression and tests long-window
  transient-reset phase propagation for expansion, instead of using
  long-window identity locking through attacks. Regenerated the targeted
  report at
  `target/stretch-corpus-fma-coherence-expansion-reset-candidate-targeted.tsv`
  (`real 337.49`). The candidate improved the shape but is still rejected on
  targeted evidence: 15 rows, 7 improved, 5 unchanged, 3 regressed, worst
  regression `delta=0.039756` on `stretch:full_mix` `0017` at `1.25x`.
  Compression regressions were removed by construction, and expansion wins
  remained (`stretch:vocals` `000020` at `1.5x`, `delta=-0.295449`;
  `stretch:bass` `000236` at `1.25x`, `delta=-0.251598`), but the remaining
  full-mix and bass regressions mean no broad run or product routing change is
  justified.
- 2026-07-09: added a report-only frame-stability candidate named
  `expansion-long-window-stability-adaptive`. The path keeps current
  OfflineHighQuality output for compression and tests a long-window expansion
  engine that applies identity phase locking only on frames whose spectral
  magnitude profile remains stable. Regenerated the targeted report at
  `target/stretch-corpus-fma-coherence-stability-adaptive-candidate-targeted.tsv`
  (`real 501.56`). The candidate is rejected and is worse than the
  ratio-scoped transient-reset candidate: 15 rows, 6 improved, 5 unchanged,
  4 regressed, worst regression `delta=0.178415` on `stretch:full_mix` `0016`
  at `1.25x`. It improved the best vocal target more than transient reset
  (`-0.321471` versus `-0.295449`), but the full-mix regressions are too large.
  Do not tune the scalar stability threshold as a product path from this
  evidence; the failure mode needs a different mechanism.
- 2026-07-09: added a report-only peak-region candidate named
  `expansion-long-window-tracked-peak-regions`. The path keeps current
  OfflineHighQuality output for compression and tests long-window expansion
  identity locking where peaks tracked from the previous analysis frame keep
  normal phase-lock regions, while new/untracked peaks lock only a narrow local
  region. Regenerated the targeted report at
  `target/stretch-corpus-fma-coherence-tracked-peak-candidate-targeted.tsv`
  (`real 330.32`). The candidate is rejected: 15 rows, 6 improved, 5 unchanged,
  4 regressed, worst regression `delta=0.059913` on `stretch:vocals` `000020`
  at `1.25x`. It preserves the strongest vocal improvement from raw
  long-window identity locking (`delta=-0.389153` at `1.5x`), but it regresses
  more rows than the simpler ratio-scoped transient-reset path and has a worse
  worst regression (`0.059913` versus `0.039756`). No broad run or product
  routing change is justified.
- 2026-07-09: added a report-only magnitude-evolution candidate named
  `expansion-long-window-magnitude-slew`. The path keeps current
  OfflineHighQuality output for compression and tests long-window expansion
  identity locking with per-bin magnitude slew limiting on spectrally stable
  frames. Regenerated the targeted report at
  `target/stretch-corpus-fma-coherence-magnitude-slew-candidate-targeted.tsv`
  (`real 455.71`). The candidate is rejected and is worse than the
  ratio-scoped transient-reset path: 15 rows, 5 improved, 5 unchanged,
  5 regressed, worst regression `delta=0.112175` on `stretch:full_mix` `0016`
  at `1.25x`. It produced a useful full-mix `1.5x` improvement
  (`delta=-0.260414`), but reduced the vocal and bass wins and increased the
  regression count. Do not tune this scalar slew limiter as a product path from
  this evidence.
- 2026-07-09: concluded Batch 22.6 low-risk candidate work. The sustained
  coherence review found real long-window benefits, especially vocal `1.5x`
  and bass `1.25x` cases, but every selector, blend, envelope, phase-region,
  and scalar magnitude-evolution attempt left targeted regressions. The best
  rejected shape is still `expansion-long-window-transient-reset` with
  7 improved, 5 unchanged, 3 regressed, and worst regression `delta=0.039756`.
  That is useful design evidence for a future structural hybrid, not enough to
  justify broad validation, cache identity changes, render-plane routing, or
  product promotion in this batch. Keep all long-window sustained-coherence
  paths report-only/benchmark-only.

## Next Task

Close Batch 22.6 as evidence-complete for low-risk single-batch candidates.
Do not add more selector, blend, threshold, or one-parameter long-window probes.
The next stretch DSP work should either open a new, explicitly structural
hybrid-design batch or move back to the remaining g10 stretch priorities
outside sustained/polyphonic long-window promotion. Do not change product
routing, cache identity, dynamic-ratio materialization, or pitch-shift
materialization for the long-window path from Batch 22.6 evidence.
