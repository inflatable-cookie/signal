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

## Next Task

Do not add a multiresolution or hybrid path until measured evidence requires
it. The width-control postprocess and simple compression phase-reset anchor
lines are now closed as report-only/rejected. A global shorter-window switch is
also rejected, and the `000900.mp3` matched-event width issue is not solved by
window size alone. Next useful work is a report-only multiresolution selection
diagnostic: localize the 8 short-window wins and 4 short-window regressions
with feature rows that could support a non-oracle selector. Do not promote a
multiresolution path until that selector can retain the mean-smear improvement
without the `000002.mp3` drum regression or new draft regressions.
