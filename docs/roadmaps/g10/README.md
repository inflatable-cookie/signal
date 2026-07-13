# g10 Milestones

Status: active
Updated: 2026-07-11

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
  gates. Batch 29.6E is rejected. The replacement whole-band full
  phase-gradient kernel now passes its synthetic mechanism gate with exact
  assignment, bounded heap, deterministic output, and both propagation
  directions proven. Its complete mono candidate improves tonal and Rubber Band
  comparison evidence, but fails crest, timing, replica, formant, integrity,
  and combined gates. Batch 29.6G is rejected. Its repeated rounded analysis
  hop also exposes up to roughly `161` frames of five-second source-map drift.
  Batch 29.6H proves exact mapping but still fails timing, replica, integrity,
  and combined gates. It is rejected. The next bounded family is one
  frequency-adaptive painless nonstationary Gabor transform. Batch 29.6I first
  proves canonical-dual reconstruction and band timing without stretching. It
  passes with near-unity frame bounds and sub-`1e-6` reconstruction error. Its
  unequal band-time lattices cannot directly use published filter-bank PGHI.
  Batch 29.6J replaces only the proof geometry with a uniform grid-decimated
  wavelet frame before phase propagation. It passes canonical-dual and control
  gates with condition ratio `1.025819956`. Batch 29.6K now owns exact
  fractional source projection and delay-compensated phase transport; linked
  stereo remains closed. Its phase-difference estimator aliases on the `8 kHz`
  tone and is rejected before interpolation or heap integration.
  Batch 29.6L passes an alias-free same-column auxiliary derivative-filter
  ratio through `19.5 kHz`. Batch 29.6M passes exact three-field source
  projection and duration-independent bounded heap integration across all `30`
  control/ratio cases. Batch 29.6N now owns a measured two-sided canonical-dual
  guard followed by protected-centre synthetic synthesis. It stops before
  assembly if no guard within `16384` frames reaches `1e-12` tail energy. The
  guard fails immediately on lowpass channel `0`: excluded energy remains
  `6.270779e-7`, so no audio is assembled. Batch 29.6O must attribute that tail
  before filter or boundary redesign. Its frozen matrix compares five channels,
  three response stages, two spectrum forms, six radii, and four thresholds.
  The result assigns DC tail growth to tightening and identifies an independent
  raw Nyquist-edge tail. Batch 29.6P must jointly redesign those boundary
  completions while retaining the passing interior bank. Its single frozen
  candidate removes pointwise tightening, keeps raw channels `0..1534`, and
  uses one endpoint-flat real Nyquist completion in channel `1535`. It passes
  exact reconstruction but fails frame conditioning at `2.980258951` against
  the `1.25` cap, before representative guards. Batch 29.6Q must freeze one
  smooth endpoint-compatible preconditioner or normalizer without reopening a
  width or taper sweep. It freezes one common inverse-square-root frame-energy
  multiplier with quintic endpoint blends over the existing `16h` spans.
  Batch 29.6R rejects it at reconstruction: condition ratio `3.0185626163`
  exceeds `1.25`, so no representative guard runs. Batch 29.6S must attribute
  the complete alias-block conditioning failure before another candidate. Its
  frozen matrix compares three banks across all `11` residues and decomposes
  each global extremal mode by boundary-bin mass and channel cross terms. Its
  first run is inconclusive: worst eigenpair residual `0.031864856` exceeds
  `1e-6`. Batch 29.6U must freeze an accurate deterministic Hermitian solver.
  It freezes a full lexicographic cyclic complex-Hermitian Jacobi proof with
  bounded sweeps and residual, orthogonality, trace, and Frobenius gates. The
  proof passes all `33` matrices with maximum residual `9.186641e-13`.
  Accurate attribution then selects boundary geometry: the exact-pointwise
  condition is `2.9916436058`, and both limiting modes are Nyquist-localized.
  Batch 29.6X freezes one report-only ablation of channel `1535` across every
  residue, comparing the full operator, complete channel removal, and removal
  of only its off-diagonal coupling before any filter design.
  The ablation selects orthogonal or multi-row completion research:
  off-diagonal-only removal passes at condition `1.1141796230`, while complete
  channel removal still fails at `2.6496906694`. Batch 29.6Z must contract one
  realizable geometry before implementation.
  It freezes three equal-energy completion rows at delays `-128`, `0`, and
  `+128`. Their three-point DFT phase coding preserves diagonal energy and
  cancels all possible same-residue completion cross terms. Batch 29.6AA owns
  only the construction and frame-matrix proof.
  The construction passes, but the complete bank rejects at condition
  `2.0862893665`, with limiting residues `3` and `8`. Batch 29.6AB must freeze
  residual boundary attribution before another candidate.
  It freezes a four-operator comparison of the full candidate against DC,
  preserved-high-edge, and joint boundary cross-term diagonalization. Batch
  29.6AC owns the report and direction decision only.
  The report selects complete raw-bank reassessment: DC removal is neutral and
  high-edge removal worsens condition to `2.1170081614`. Batch 29.6AD is an
  explicit step-back checkpoint before more implementation.
  It freezes one final common-grid candidate: exact per-residue canonical
  `S^-1/2`, rejected unless it preserves compact support and bounded all-row
  atom localization. Batch 29.6AE owns that feasibility proof only.
  It reaches numerical identity but violates compact support on row `12`, so
  common-grid correction is closed. Batch 29.6AF owns transform-family
  reassessment before any more DSP implementation. That reassessment returns to
  the passing painless Batch 29.6I bank on one dense common lattice. Batch
  29.6AG owns geometry, cost, reconstruction, boundary, and large-probe
  localization feasibility only. It is rejected: condition and identity pass,
  but redundancy is `208`, real-spectrum closure fails, and limiting atoms keep
  roughly half their energy outside the localization cap. Batch 29.6AH is an
  operator direction checkpoint. The operator authorizes continued research
  without relaxing failed gates. Batch 29.6AI now owns declared-schedule
  time-adaptive painless reconstruction only; selection and stretching remain
  closed. That reconstruction passes all schedules and controls with adaptive
  condition `1.5934675721` and sub-`1e-15` peak error. Batch 29.6AJ must freeze
  automatic time-resolution selection before detector implementation. It now
  freezes one normalized local `alpha=0.7` Rényi selector and legal minimum-cost
  resolution path. Batch 29.6AK rejects it: isolated-event ownership is too
  broad, a linear chirp stays all-short, and mixed tonal/transient audio stays
  all-long. Batch 29.6AL freezes exact additive time/frequency attribution;
  Batch 29.6AM stops inconclusive because both mechanisms are present but
  neither owns the failures cleanly. Batch 29.6AN must reassess attribution
  resolution once using event-support membership and fixed low-band
  subdivision before any selector change. Batch 29.6AO selects comparison-region
  geometry and rejects frequency weighting; Batch 29.6AP must freeze one
  source-blind geometry before implementation. It now selects anchor-local,
  support-contained natural-hop lattices; Batch 29.6AQ is the terminal selector
  gate before phase contracting or operator review. It rejects on isolated
  far-field recovery, mixed-event recovery, and perturbation stability.
  Batch 29.6AR records operator direction to retire Rényi-only selection and
  research magnitude-gated mixed-phase-derivative percussive occupancy. Batch
  29.6AS freezes one analytic report-only detector; Batch 29.6AT must prove it
  before any occupancy-to-window mapping. It rejects across false-positive,
  localization, dense-event, perturbation, and stereo gates; Batch 29.6AU is an
  operator-review stop. Operator direction keeps the mixed-phase family and
  opens Batch 29.6AV distribution measurement before any calibrated mask. All
  `25` pairs overlap and one stereo cutoff signature fails equivalence; Batch
  29.6AW is an evidence-family review stop. Operator direction selects one
  evidence-only median-HPSS contract; Batch 29.6AX must prove it before mapping.
  It rejects across every negative and event family despite passing stereo;
  Batch 29.6AY stops automatic-selector churn. Batch 29.6AZ freezes an
  oracle-scheduled end-to-end candidate; it fails `1.5x` impulse placement.
  Operator review then identifies the broader constraint error: local timing,
  coordinated transient phase treatment, joint mechanism tuning, and
  simultaneous multi-resolution processing were prohibited or isolated before
  a comparable complete system ran. Batches 29.6BD-BG reopen the lane through
  Rubber Band behavioural forensics, cross-control mechanism attribution, and
  one new complete-system contract before synthesis resumes.
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
  evidence. The first complete simultaneous multi-window successor is rejected
  at concealed development listening: all three candidates share gross temporal
  smear and cannot reach `6/9`. Holdout remains unread. Attribution measures
  roughly `173` frames of mean layer-arrival disagreement and retires
  independent full-band layer phase transport. One shared full-field phase
  proof also fails, leaving roughly `162` frames of mean disagreement and low
  `0.134` correlation. Redundant full-band ownership is closed. The bounded
  review selects one time-adaptive painless NSG frame with one window and
  coefficient vector per analysis centre. Its single-owner mechanics proof
  passes all five schedules with the prior identity hash unchanged. Frozen
  linked study and one-global-map attachment also pass all three ratios with
  zero structural or mapping failures. Output-lattice coverage and one
  continuous single-frame phase/synthesis path now pass four controls. Synthetic
  quality then rejects the frozen mode on pitch and event placement despite
  intact structure and identity. Trace attribution assigns the failures to
  dormant-bin phase continuation and missing independent event ownership. The
  successor mechanism now passes all `32` active-peak/transient-anchor rows:
  tone errors stay below `1e-6`, all `24/24` expected anchors attach exactly,
  all eight hard failure classes are zero, and evidence hash
  `a2d3fb95545cb47f` repeats. A `262`-frame dense-event peak diagnostic remains
  exposed to the complete synthetic quality gate. That gate now rejects only
  `DenseEvent 2.0x`: first peak exact, second peak `262` frames from target
  against `256`. All other successor hard checks pass with zero regressions;
  evidence hash `c72c005d0cd44e3e` repeats. Attribution then proves both real
  `2.0x` attacks are exact, but overlap synthesis creates a louder midpoint
  replica at output `16382`. Evidence hash `2336b9773c32b2ca` repeats. One
  bounded event-local overlap owner now removes that replica, preserves both
  real attacks exactly, and leaves the passing dense ratios bit-identical. The
  complete `48`-row synthetic gate passes with zero failures or regressions;
  evidence hashes `adf37bdd72012e19` and `dec15b718aa27de9` repeat. The frozen
  mono objective then rejects the candidate before listening: exact structure
  passes, but event placement regresses in `6/9` rows, replicas in `7/9`, and
  both static spectral and formant residuals in `9/9`. Stage attribution now
  assigns the dominant damage to ordinary adaptive synthesis: its first
  transition owns `8/9`, `7/9`, `9/9`, and `9/9` regressions respectively and
  seven endpoint-integrity failures. Active tracking partly repairs it;
  event-local overlap ownership changes no real-source output. Fixed controls
  then split the ordinary defect: endpoint integrity improves from `9/9`
  failures at `512` and `1024` through `4/9` at `2048` to `0/9` at `4096`;
  adaptive timing remains worst; every fixed and adaptive ordinary render
  regresses static-spectrum and formant residual in `9/9` rows. Batch 29.6CB
  factors the shared phase/output-lattice mechanism. Holdout and listening
  remain closed.

Remaining stretch work is not blocked by Chorus. Chorus only becomes relevant
when Loophole integration needs a product workflow plan.

## Next Task

Execute `g10.029` Batch 29.6CB under Rule 30W. Factor phase transport,
event-warped output placement, and diagonal-dual overlap synthesis on fixed
`4096` without reading holdout, exporting listening audio, tuning, or changing
policy.
