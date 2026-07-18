# g10 Milestones

Status: active
Updated: 2026-07-18

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
  unchanged pending a different algorithm class. Exact-source mono listening
  now finds a material-dependent split between the coherent Signal predictor
  and Rubber Band R3, with no overall winner. Signal is competitive on the
  frozen six-row set. Rubber Band-class promotion remains blocked on shared-
  decision stereo proof, independent stereo review, and the later dynamic
  checkpoint.
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
  regresses static-spectrum and formant residual in `9/9` rows. Fixed-`4096`
  factor controls then exclude output placement, ordinary phase transport, and
  the exact diagonal dual as primary owners: linear placement is nearly
  neutral, while phase passthrough and analysis-partition overlap are worse.
  All eight factor modes still regress both timbral fields in `9/9` rows.
  Hann analysis and synthesis then reduce timing and timbral residuals, but all
  four window pairs still regress static-spectrum and formant quality in `9/9`
  rows. Geometry attribution finds that native-`2048` centered reflection
  lowers mean static/formant residual by `0.040495/0.017523` relative to the
  shared `4096` grid, while start-aligned zero padding gives much of that gain
  back. Native geometry also raises replica ratio sharply, and every mode still
  regresses both timbral fields in `9/9` rows. Shared-grid zero-padding
  contributes; the remaining phase/magnitude path owns the broad defect. Batch
  29.6CE then contracts one complete path without rendering: centered reflected
  Hann/Hann frames synthesize on native FFT grids with unchanged magnitudes and
  exact dual normalization; the fixed analytic tracker carries physical
  frequency and phase only as a decision surface; native phase regions,
  sample-refined anchors, and the proven bridge owner coordinate coherence and
  replica suppression on one output timeline. Batch 29.6CF is the bounded
  implementation and synthetic gate. It preserves identity, events, replicas,
  boundaries, mid/high tones, and matched ownership across all `300` active
  resolution transitions, but fails the three stretched `55 Hz` rows. Tracked
  frequency passes; rendered error reaches `3.695086e-5` radians/sample. Batch
  29.6CG then stops local repair and inspects pinned Signalsmith Stretch and
  Rubber Band R2/R3 source under an explicit provenance boundary. The study
  corrects Signal's representation: Rubber Band R3 standard runs simultaneous
  long/middle/short transforms with exclusive frequency ownership, not one
  time-selected full-band resolution. Its full-band H/P/R classification guides
  crossover and phase state; it is not additive component synthesis.
  Signalsmith supplies the contrasting fixed-grid weighted multi-predictor
  control. Batch 29.6CH passes structure, determinism, and all nine
  development integrity rows for both paths. The fixed-grid control passes the
  synthetic quality gate. The partitioned path retains a marginal `3 Hz` tone
  miss against the `2 Hz` cap while passing event placement at `190` frames;
  it is not tuned. Concealed listening rejects frequency partitioning and
  retains only the fixed-grid weighted predictor for successor research. The
  first external rankings are invalid because Signal consumed `16384`-frame mono
  excerpts while Rubber Band and Signalsmith consumed full `220500`-frame
  stereo sources before truncation. Batch 29.6CJ now repeats the unchanged rows
  with exact input identity: nine hash-frozen mono inputs, `18` exact-length
  external renders, and one `45`-file four-way pack pass structural validation.
  Listening finds the weighted predictor credible but inconsistent: two wins or
  ties, two competitive rows, and four rows with softness, smear, grain, or a
  boundary pop. Because those sources are shorter than half a second, Batch
  29.6CK now owns one final musical-continuity decision using six five-second
  `1.5x`/`2.0x` rows. Its exact-input weighted/current/Rubber pack contains `24`
  audio files with zero structural failures. Listening shows the first coherent
  architectural improvement: weighted prediction beats current Signal on four
  rows, but mutates one bass tone and causes severe pad phase damage; Rubber
  Band wins four rows. Reinspection finds the Signal proof changed defining
  predictor mechanics, including transform geometry, time-factor-scaled
  vertical twists, energy normalization, fallback, and update ordering. Batch
  29.6CL now freezes one faithful topology before more code. Rule 30AB repair,
  parameter lattices, holdout, stereo, dynamic ratio, cache, and routing remain
  closed.

  Batch 29.6CL is now complete. The corrected topology uses a fixed 30 ms output
  grid, fourfold centered long-window support, ratio-projected input centres,
  actual-hop horizontal transport, fractional time-factor-scaled short/long
  vertical twists from both directions, ascending dependency order, target-
  energy normalization, and weak-evidence fallback. Signal retains square-root
  Hann/overlap normalization and deterministic RustFFT synthesis. Direct bass,
  chord/pad, transient, silence, boundary, coverage, duration, mechanism, and
  repeat gates are frozen before real-source audio.

  Shared-decision stereo now has a reference-relative recurrence. It restores
  broadband delay and reduces the prior quadrature/image failures by orders of
  magnitude, but does not pass the exact gate. Residual attribution excludes
  coefficient projection and real-edge constraint at `4.440892e-16 rad`.
  Boundary cropping reduces tone IPD while interior correlated-image damage
  remains; a constant-relation oracle is not consistently better. Batch 29.7G
  calibrates the cropped measurement floor and assigns the first observable
  divergence to real support-frame synthesis. Overlap often reduces it and
  normalization is neutral. Batch 29.7H rejects analytic overlap as linearly
  equivalent: metrics do not improve and only floating-point bit parity moves.
  Batch 29.7I closes initial-frame, fallback, and weak-bin coefficient
  contributions: every class preserves relation within floating-point error,
  omitted energy is negligible, and no ablation closes the remaining gate.
  Batch 29.7J now calibrates the exact stereo invariant against ideal and
  external-reference behavior. Its 192-row matrix rejects the `1e-9` external
  IPD gate and finds material Signal tone/image drift against ideal and Rubber
  Band. Batch 29.7K rejects render-wide relation coloring despite exact
  mechanics: repaired Signal still fails `14/48` calibrated rows and `17/48`
  local rows. Batch 29.7L pins exact Rubber Band `4.0.0` source and identifies
  conditional, frequency-bounded peak-trajectory sharing as the first
  architectural difference from Signal's same-bin recurrence. Standard R3
  passes all `48` calibrated rows. Centre-focus changes every render but fails
  four `2.0x` image rows, rejecting mid/side and blanket linking as the repair.
  Batch 29.7M rejects one Signal-specified nearest-peak realization: failures
  rise from `20/48` to `29/48` and local consistency fails on `32/48` rows.
  Batch 29.7N isolates the loss: independent recurrence fails `40/48`, while
  peak sharing repairs all tone rows but a single peer anchor regresses `22/24`
  image rows. Current relational recurrence remains the default. Batch 29.7O
  rejects its frequency-aligned tracked identity overlay: failures rise to
  `25/48`, no row improves completely, and all `48` regress somewhere despite
  exact mechanics. Batch 29.7P then attributes the conflict across anchors,
  interiors, and boundaries. Relation RMS rises by more than one radian in all
  three classes at evidence `e1713e619138301b`. Rule 31H forbids late tracked
  overlays and opens one complete peak-owned eligible-region proof. Batch
  29.7Q rejects that proof: failures improve over the overlay but remain
  `23/48` against the `20/48` baseline, with `46/48` rows regressing somewhere.
  Evidence `2a52a1106fadf298` closes local peak-region variants for operator
  review. Batch 29.7R identifies a cross-family hybrid: the coherent
  pure-stretch kernel is a continuous field without peak mapping, while the
  comparator's linked peak belongs to a complete phase-vocoder state machine.
  Current-kernel tracked peaks close. Batch 29.7S then closes joint PGHI for
  the next renderer: its Signal mono kernel already failed and no published
  source supplies joint multichannel heap ownership. One separate
  `SharedRotationRegionLocked` family is selected from independent peak-region,
  stereo, transient, representation, MIT implementation, and MPL whole-kernel
  evidence. Batch 29.7T then reduces calibrated stereo failures from `20/48`
  to `1/48` and passes exact mechanics plus the unchanged mono gate, but misses
  passage on 11 tone-local consistency rows. Batch 29.7U localizes every miss
  to the first or last local window and assigns the first divergence to overlap
  of boundary-conditioned tracked frames. Batch 29.7V owns one parameter-free
  finite-support reset proof. That proof rejects at `4/48` calibrated failures,
  `19/48` local failures, nine new local regressions, and failed candidate
  parity with the frozen mono control. Batch 29.7W then closes shared rotation
  as a complete renderer while retaining common rotation as locked-state
  evidence. Batch 29.7X then closes the two missing seams from independent
  papers and selects one painless frequency-adaptive material-phase proof. It
  explicitly rejects the former three-STFT synthesis topology. Batch 29.7Y
  proves the new frame exactly, then rejects the frozen material-phase
  candidate at `36/48` calibrated and `46/48` local stereo failures. Its
  monolithic repeated mono report is also stopped after more than five hours.
  Batch 29.7Z then proves independent polar interpolation is the first
  relation break and selects explicit peer/reference relation transport plus a
  fixed `16384/8192/512` sliced frame from primary evidence. Batch 29.7AA Stage
  A then passes sliced identity and boundedness with `4.44e-16` peak error,
  two live slices, and duration-independent coefficient memory. Stage B then
  preserves the shared coefficient relation but rejects at `44/48` calibrated
  and `46/48` local stereo failures. Batch 29.7AB attributes the loss to
  synthesis inconsistency: modified redundant fields do not satisfy
  `A D C = C`. The frequency-adaptive family closes. Batch 29.7AC owns a
  no-renderer paired-channel consistency-operator study. That study finds no
  transferable complete operator: additive-mixture projection requires a known
  source sum, while covariance matching is a spatial renderer rather than a
  unique source-preservation constraint. Rule 31L closes transform-domain
  post-projection. Batch 29.7AD then closes WSOLA as the universal engine,
  retains explicit sinusoidal models as research reserve, and selects one
  single-grid state-complete linked phase-vocoder proof. Rule 31M permits
  bounded development calibration before one candidate freezes for holdout.
  Batch 29.7AE runs that calibration but freezes no candidate: the best result
  retains the 29.7T boundary at `1/48` calibrated and `11/48` local failures,
  while three state-changing finalists worsen local consistency. All retain
  exact mechanics and mono passage. The concealed holdout remains unread;
  Batch 29.7AF then finds two synthesis losses. Four misses, including the sole
  calibrated failure, start in the full inverse frame; seven start when the
  `1024`-sample inverse is restricted to `960` samples. Candidate `17`
  preserves the split. Rule 31M closes the single-grid family without a
  correction or holdout access. Batch 29.7AG then selects one source-feasibility
  direction: linked subband sinusoidal tracking with explicit stereo-paired
  partial trajectories and direct oscillator synthesis. Pinned SBSMS `2.3.0`
  supplies the architecture specimen, not a dependency. Rule 31O requires
  exact-source quality and boundedness evidence before any clean-room Signal
  renderer. Batch 29.7AH closes that candidate: aggregate stereo passes, but
  six local rows, exact mechanics, seven mono hard rows, two row-complete mono
  comparisons, and `21` long-development metrics reject. Batch 29.7AI then
  proves the old local and polarity/gain vetoes over-tight against Rubber Band.
  Rule 31Q retains calibrated stereo plus four genuine structural mechanics
  and freezes a professional-comparator local envelope. Batch 29.7AJ then
  traces pinned Rubber Band R3, Signalsmith Stretch, and Bungee. It selects one
  clean-room `GuidedFrequencyPartitionedLinkedPhaseVocoder` proof. Unlike the
  rejected 29.6CH and 29.7Y paths, exclusive scale ownership, synchronized
  all-channel phase-state selection, conditional linked peak borrowing, and
  per-channel synthesis form one indivisible waveform owner. Rule 31R permits
  one stop-gated mechanics and objective proof with fixed declared bounds and
  no external numeric-policy transfer. Batch 29.7AK passes its fixed `48 kHz`
  mechanics proof at `2.91e-16` identity error with exact channel mechanics,
  then closes before objective rows. The frozen `8 kHz` gate requires
  `2432/1217` signed/nonnegative atoms instead of `1344/673`, while the attempted
  whole-source coefficient store grows with duration. Rule 31S permits only a
  bounded two-slice representation compatibility study before another renderer.
  Batch 29.7AL selects one sample-rate-normalized exact sliced representation
  for proof. `H=F/100`, `N=32H`, outer advance `16H`, and `8H/4H/2H` supports
  keep `8/44.1/48 kHz` inside frozen atom capacity. Fixed source/output slabs
  and state rings remove duration-sized storage; one global state update owns
  both active layers. Rule 31T opens identity and inert boundary-token
  mechanics only.
  Batch 29.7AM passes at evidence hash `0407f765c7d84375`: peak combined
  identity error is `4.44e-16`, outer partition error is `6.66e-16`,
  conjugacy is exact, active-layer high-water is two, and every structural,
  mechanics, finite, token, and overflow check passes. Rule 31U opens only
  synchronized guided-state mechanics across the frozen slice boundary.
  Batch 29.7AN passes at evidence hash `90c10cd2e66d4faf`: all state branches
  cross interior and boundary contexts, channel mechanics are exact, layer
  ownership error stays below `4.45e-16`, and region high-water is
  `32/100/107`. Batch 29.7ANR now passes implementation-free Rule 31V
  preregistration. Exact `4/2/1`-tick material radii, same-scale frequency
  medians, the `19`-tick halo, state order, link limits, fixed bounds, and one
  failure-first evidence matrix are frozen. Batch 29.7AO implements that policy
  and passes synthetic structure plus exact channel mechanics, then rejects at
  `46/48` calibrated stereo failures, `110/384` improved windows, `44/48`
  local failures, and maximum residual `0.86973539821584`. Mono and long-
  development do not run. Rule 31W opens one coefficient-to-waveform stereo
  failure attribution before any new candidate. Batch 29.7AP completes that
  replay at hash `24cdad83bf3ddeeb`. Every retained first/worst operator event
  is an interior `Unlocked` state commit; projected-layer residuals match the
  state residuals exactly. Rule 31X freezes one reference-relative unlocked
  commit for a single failure-first proof. Batch 29.7AQ passes exact mechanics
  and synthetic evidence at hash `875b0768ba2066bf`, then rejects its one
  corrected stereo run at `40/48` calibrated failures, `125/384` improved
  windows, `44/48` local-row failures, and hash `88d9c0f68ea2954b`. The local
  gain does not cross the row-level gate. This topology is closed.
  Source reinspection then corrects the underlying observer: Rubber Band R3
  keeps ordinary and unlocked recurrence channel-local and borrows across
  channels only in compatible locked peak regions. The bounded normalized
  renderer also added an independently windowed outer meta-slice layer absent
  from the selected direct scale topology. Rule 31Y opens implementation-free
  Batch 29.7AR to preregister one direct frequency-partitioned scale timeline.
  Later representation mechanics, state mechanics, and one failure-first
  objective gate were compiled as Batches 29.7AS through 29.7AU. Batch 29.7AR
  now passes implementation-free under Rule 31Z. It freezes the direct
  `10 ms` lattice, `80/40/20 ms` scales, `750/6000 Hz` ownership, absolute
  schedule, boundaries, state order, and fixed capacities. It also corrects
  the identity claim: unity is bit-exact bypass, per-scale reconstruction is
  hard, and the inert masked scale sum is diagnostic. Batch 29.7AS now passes
  at hash `fdf90f6127749341`: representation, fixed storage, capacity, unity,
  per-scale reconstruction, boundaries, and repeat are exact. The
  masked diagnostic keeps zero bounded-lag timing but reaches `0.451615 dB`
  gain movement and `0.056339` peak residual at fixed crossovers. Those values
  are frozen without tuning. Batch 29.7AT now passes direct state mechanics at
  hash `430543f8e1dce721`: every terminal state, channel-local unlocked
  recurrence, compatible locked-only borrowing, peer ownership, recovery,
  boundaries, fixed slabs, and repeat pass. Batch 29.7AU is ready as one
  failure-first objective sequence. Batch 29.7AU then passes its no-audio and
  synthetic gates at hash `00e522a01b817bb6` before the single stereo run
  rejects: `40/48` calibrated failures, `118/384` improved windows, `36/48`
  local failures, maximum residual `0.7611955347641768`, and hash
  `af461c9576729c4e`. All gains are on image controls; tone improves `0/192`
  windows. Mono and long-development do not run. The frozen compatible-lock
  code makes every channel's peak-relative offset zero at the borrowed peak,
  so Batch 29.7AV proves that relation collapse at hash `346e329081adf701`.
  Reset and attack retain exact relation; unlocked and exact-`6000 Hz` local
  lock remain channel-local. A borrowed peak loses its complete `0.95 rad`
  input relation and exits at zero. Rule 31AA freezes one mechanics-only owner-
  peak reference correction. Batch 29.7AW now applies only that substitution:
  the analytic borrowed relation is preserved exactly at hash
  `425400ebb580b3e1`, all `9/9` direct mechanics pass at corrected state hash
  `52d6b8b2bb6edff0`, and representation stays `fdf90f6127749341`. No corpus
  audio ran. Batch 29.7AX is ready as the separately preregistered objective
  rerun. AX then passes mechanics and unchanged synthetic evidence but rejects
  its one stereo run at hash `397128c177d3033e`: `38/48` calibrated failures,
  `157/384` improved windows, `36/48` local-row failures, and unchanged maximum
  residual `0.7611955347641768`. This improves AU by `2` calibrated rows and
  `39` windows without moving row-complete failures or the worst case. Mono and
  long-development do not run. Batch 29.7AY opens no-audio architecture
  reassessment.

Remaining stretch work is not blocked by Chorus. Chorus only becomes relevant
when Loophole integration needs a product workflow plan.

## Next Task

Run `g10.029` Batch 29.7AY under Rule 31AB. Freeze AU/AX and audit the remaining
direct topology against the source-studied ownership model without audio. Name
one source-supported causal mechanism and no-audio falsifier, or close the
topology. Keep tuning, retry, listening, holdout, product surfaces, and Batch
29.8 closed.
