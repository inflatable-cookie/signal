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
- [x] produce a bounded blind-listening pack with completed operator notes
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
- [x] derive the smallest whole-hop two-sided canonical-dual guard meeting
  `1e-12` tail energy, and stop if it exceeds `16384` frames
  - rejected on lowpass channel `0`: guard lower bound `16768`, excluded energy
    `6.270779e-7`, dual residual `1.051210e-12`
- [ ] reflect-pad the source, reuse 29.6M on logical negative/positive guarded
  columns, synthesize the complete canonical-dual spectrum, and crop only the
  protected centre
- [ ] prove identity, compression, and expansion on the Contract `082`
  synthetic control set before any corpus render
- [ ] stop for research on any symmetry, dual-residual, coverage, placement,
  finite-value, or determinism failure

### Batch 29.6O - Dual-Atom Tail Attribution

- [x] freeze analysis-atom versus canonical-dual tail measurements for the
  limiting lowpass channel and representative interior/high channels
- [x] isolate tightening, analytic mirroring, and lowpass-completion ownership
  with report-only controls; do not tune or synthesize candidates
- [x] report tail-energy curves, limiting channels, finite values, solve
  residuals, and repeat hashes before choosing filter or boundary redesign
- [x] keep coefficient assembly, audio synthesis, corpus, stereo, dynamic
  ratio, and product routing closed

### Batch 29.6P - Joint Boundary Completion Reassessment

- [x] freeze one smooth real-output DC/Nyquist completion design while
  preserving the passing interior Cauchy channels, delays, and `384`-frame grid
- [x] test complete frame conditioning and canonical-dual reconstruction;
  reject at condition ratio `2.980258951` before the dual-atom guard
- [ ] require derivative-estimator, projected-field, and bounded-heap reproof
  after transform passage and before any audio synthesis
- [x] keep candidate tuning, coefficient assembly, synthesis, corpus, stereo,
  dynamic ratio, and product routing closed until the contract is frozen
  - frozen candidate: raw channels `0..1534`, no pointwise tightening, one
    zero-delay smoothstep-sine Nyquist completion in channel `1535`

### Batch 29.6Q - Smooth Boundary Preconditioner Contract

- [x] freeze one smooth endpoint-compatible frame preconditioner or normalizer;
  do not sweep completion widths, tapers, delays, or channel allocations
- [x] preserve raw channel `0` compactness, the Batch 29.6P channel `1535`
  completion, condition ratio at most `1.25`, and exact canonical-dual identity
- [x] order reconstruction, representative guard, all-channel guard, then
  derivative-estimator and projected-field reproof as hard stop gates
- [x] keep coefficient assembly, synthesis, corpus, stereo, dynamic ratio, and
  product routing closed
  - frozen candidate: one common inverse-square-root frame-energy multiplier,
    quintic-blended to constant endpoint values across the existing `16h` spans

### Batch 29.6R - Endpoint-Even Normalization Proof

- [x] implement only the frozen common scalar normalizer and separate raw-bank
  and multiplier hashes; do not add gains, fitted slopes, or variants
- [x] test complete reconstruction; reject condition ratio `3.0185626163`
  against `1.25` while retaining dual, identity, finite-value, and repeat gates
- [x] honor the reconstruction stop gate; do not run the six-channel
  representative guard or all-channel guard
- [x] keep phase reproof, coefficient assembly, synthesis, corpus, stereo,
  dynamic ratio, and product routing closed

### Batch 29.6S - Alias-Block Conditioning Attribution Contract

- [x] freeze one report-only matrix across raw, exact-pointwise, and
  endpoint-even banks; do not synthesize another preconditioner
- [x] attribute limiting residue blocks, eigenvalue extrema, boundary-bin
  membership, and per-channel energy/cross-term ownership
- [x] require finite values, stable hashes, exact repeat evidence, and no
  coefficient assembly, guard, phase, corpus, or listening work
- [x] use the attribution to decide whether block-aware preconditioning or
  boundary-geometry reassessment is justified

### Batch 29.6T - Alias-Block Conditioning Attribution Proof

- [x] measure all `11` residues for raw, exact-pointwise, and endpoint-even
  banks; reject worst eigenpair residual `0.031864856` against `1e-6`
- [x] decompose each bank's global minimum and maximum modes by boundary-bin
  mass, cross-bank Rayleigh transfer, and bounded channel/bin contributors
- [x] retain finite evidence, contribution closure `6.650463e-16`, stable
  hashes, and exact repeat without reconstruction, dual, guard, or phase work
- [x] stop as numerically inconclusive before either research direction

### Batch 29.6U - Deterministic Hermitian Eigensolver Contract

- [x] freeze one bounded solver and invariant cross-check for the existing
  alias-block matrices; do not relax residuals or increase power iterations
- [x] require every minimum and maximum eigenpair residual at most `1e-6` with
  deterministic phase, trace/Frobenius checks, hashes, and exact repeat
- [x] reopen only the frozen attribution after solver passage; keep every DSP
  candidate, guard, phase, coefficient, corpus, and listening surface closed

### Batch 29.6V - Cyclic Hermitian Jacobi Proof

- [x] implement lexicographic cyclic complex-Hermitian Jacobi for sizes
  `1..=193`, `64` sweeps, and relative off-diagonal tolerance `1e-13`
- [x] pass analytic scalar, real/complex `2x2`, diagonal, repeated, and clustered
  controls before the `33` frozen alias matrices
- [x] require residual `1e-8`, orthogonality `1e-10`, trace `1e-12`, Frobenius
  `1e-10`, finite values, stable hashes, and exact repeat
- [x] reopen only Batch 29.6T attribution after passage; keep DSP work closed

### Batch 29.6W - Jacobi Alias Attribution Decision

- [x] replace only attribution eigenpairs with the proven Jacobi solver
- [x] retain the three banks, `33` matrices, contributor bounds, closure gates,
  hashes, and exact repeat from Rule 26B
- [x] select boundary geometry; exact-pointwise condition `2.9916436058`
  exceeds `1.25`, so block-aware preconditioner research stays closed

### Batch 29.6X - Boundary Geometry Reassessment Contract

- [x] freeze one geometry research question from the Nyquist-localized
  attribution; do not sweep filters or normalization
- [x] preserve passing interior channels, common-grid timing, and exact dual
  ownership while defining DC/Nyquist completion alternatives and stop gates
- [x] keep implementation, guards, phase reproof, synthesis, corpus, stereo,
  dynamic ratio, and product routing closed

### Batch 29.6Y - Nyquist Completion Alias-Coupling Ablation

- [x] compare full, channel-`1535`-removed, and channel-`1535`-diagonalized
  exact-pointwise frame matrices across all `11` residues
- [x] report conditioning, Jacobi gates, completion diagonal/off-diagonal energy,
  frozen-mode Rayleigh changes, hashes, closure, and exact repeat
- [x] select orthogonal/multi-row completion research, replacement-completion
  research, broader high-edge geometry, or inconclusive; implement nothing

### Batch 29.6Z - Orthogonal Nyquist Completion Research Contract

- [x] freeze one orthogonal or multi-row completion question that retains the
  passing diagonal energy without same-row cross-bin alias coupling
- [x] preserve channels `0..1534`, common-grid timing, real endpoints, and
  smooth boundary ownership; define reconstruction, conditioning, and stop gates
- [x] keep filter implementation, duals, guards, phase, synthesis, corpus,
  stereo, dynamic ratio, and product routing closed

### Batch 29.6AA - Three-Row Nyquist Completion Matrix Proof

- [x] replace the single completion with equal-energy rows at delays `-128`,
  `0`, and `+128`; preserve raw channels `0..1534`, magnitude, support, and hop
- [x] prove row count, hashes, finite values, real Nyquist endpoints, retained
  diagonal energy, and roots-of-unity alias cancellation at `1e-12`
- [x] solve all `11` frame matrices with Jacobi and require condition at most
  `1.25`, numerical gates, stable hashes, and exact release repeat; stop there

### Batch 29.6AB - Residual Boundary Geometry Attribution Contract

- [x] freeze one report question for the rejected triplet's global minimum at
  residue `3` and maximum at residue `8`; do not design another filter
- [x] retain the exact candidate matrices and define bounded bin-region,
  channel diagonal/cross, Rayleigh, closure, numerical, and repeat evidence
- [x] choose the next geometry boundary or stop inconclusive; keep
  reconstruction, guards, phase, synthesis, corpus, stereo, and routing closed

### Batch 29.6AC - Residual Boundary Matrix Attribution

- [x] compare full, DC-diagonalized, preserved-high-edge-diagonalized, and
  both-boundary-diagonalized operators across all `11` residues
- [x] attribute the frozen residue-`3` minimum and residue-`8` maximum by four
  channel groups, bins, channel totals/cross terms, Rayleigh changes, and closure
- [x] pass Jacobi, finite, hash, closure, and exact-repeat gates; select DC,
  high edge, joint boundary, broader bank, or inconclusive; implement nothing

### Batch 29.6AD - Complete Raw-Bank Reassessment

- [x] step back from endpoint candidates and freeze one complete-bank research
  question from the failed untightened geometry
- [x] preserve the passing common-grid timing and solver evidence while deciding
  whether the bank geometry or transform family must change
- [x] keep implementation, reconstruction, guards, phase, synthesis, corpus,
  stereo, dynamic ratio, and routing closed

### Batch 29.6AE - Canonical Block-Tightener Feasibility

- [x] apply exact Jacobi `S^-1/2` to every residue of the rejected `1538`-row
  candidate; add no approximation, floor, localization, or correction
- [x] prove tight-frame algebra, finite values, hashes, and exact repeat, then
  scan per-row support leakage and endpoint closure, stopping at first violation
- [x] open a separate large-probe localization contract only if every row
  passes; otherwise close common-grid work and select transform-family reassessment

### Batch 29.6AF - Transform-Family Reassessment

- [x] close common-grid correction work and reassess the next invertible
  time-frequency family against the measured quality and localization failures
- [x] preserve one global time map, exact target length, real output, and
  reconstruction-first gating; do not re-open rejected component synthesis
- [x] freeze one next research question or stop for operator review; keep DSP
  implementation, corpus, stereo, dynamic ratio, and routing closed

### Batch 29.6AG - Dense Painless Common-Lattice Feasibility

- [x] rebuild the passing Batch 29.6I filters bit-identically and place every
  band on one common lattice using the largest original coefficient count
- [x] prove unchanged diagonal frame/dual geometry, explicit coefficient cost,
  identity reconstruction, real-spectrum closure, and all-band large-probe
  analysis/dual localization through the frozen `16384`-frame cap
- [x] open only a separate derivative/topology contract on complete passage;
  otherwise stop for operator review without another transform candidate

### Batch 29.6AH - Operator Direction Checkpoint

- [x] decide whether to pause successor research at the current production
  phase vocoder or authorize a new transform-research lane
- [x] do not relax localization or real-spectrum thresholds from failed
  evidence and do not infer a replacement family from implementation context
- [x] keep phase, stretched synthesis, corpus, stereo, dynamic ratio, cache,
  and routing closed until operator intent is recorded

### Batch 29.6AI - Time-Adaptive Painless Reconstruction

- [x] implement one declared-schedule `4096`-bin NSDGT proof with compact
  square-root Hann windows of `512`, `1024`, `2048`, and `4096` frames
- [x] prove schedule legality, diagonal dual coverage/condition, compact
  support, real output, exact identity reconstruction, and deterministic hashes
- [x] open only automatic resolution-selection research on complete passage;
  keep phase, stretched synthesis, corpus, stereo, dynamic ratio, and routing closed

### Batch 29.6AJ - Automatic Time-Resolution Selection Contract

- [x] choose one bounded source-evidence selector for the passing four-level
  window bank; do not combine detector families or tune against corpus output
- [x] freeze declared-event recovery, dense-event, false-positive, schedule
  legality, stability, stereo-decision, finite-value, and repeat evidence
- [x] open selector implementation only on a complete contract; keep phase,
  stretched synthesis, corpus, dynamic ratio, cache, and routing closed

### Batch 29.6AK - Rényi Time-Resolution Selection

- [x] compute normalized `alpha=0.7` local Rényi evidence for all four passing
  resolutions on one fixed `128`-frame decision grid
- [x] solve one legal minimum-entropy path and prove impulse, steady, dense,
  noise, mixed, perturbation, gain/polarity, and shared-stereo gates
- [x] open only a separately frozen variable-hop phase contract on complete
  passage; produce no modified coefficients or stretched audio

### Batch 29.6AL - Rényi Selector-Failure Attribution Contract

- [x] freeze one diagnostic that separates fixed-region temporal contamination
  from whole-band tonal-energy dominance without changing selector output
- [x] retain exact Batch 29.6AK energies, entropies, paths, controls, and hashes;
  report bounded time-slice and frequency-region contributions only
- [x] freeze exact closure, counterfactual, repeat, and decision rules before
  measuring attribution; keep selector and gate thresholds unchanged

### Batch 29.6AM - Rényi Selector-Failure Attribution Decision

- [x] run the frozen time-slice and folded-frequency attribution on the exact
  Batch 29.6AK controls and prove additive closure and unchanged baseline hashes
- [x] measure only the isolated-impulse, linear-chirp, and mixed-control failure
  anchors; report bounded leave-one-region-out entropy and raw-winner effects
- [x] select comparison-region geometry, frequency evidence, or inconclusive;
  keep phase, stretched synthesis, corpus, dynamic ratio, cache, and routing
  closed

### Batch 29.6AN - Rényi Attribution Reassessment Contract

- [x] freeze whether event-support time attribution and bounded subdivision of
  folded-frequency region `0` can separate the two coupled failure mechanisms
- [x] retain exact Batch 29.6AK selector evidence and Batch 29.6AM attribution;
  do not tune a selector, threshold, margin, or weighting rule
- [x] open one bounded attribution proof or stop selector research for operator
  review; keep phase and stretched synthesis closed

### Batch 29.6AO - Rényi Attribution Reassessment Decision

- [x] measure declared-event support ownership and the eight frozen low-band
  subregions without changing either prior report
- [x] prove partition closure, finite values, exact repeat, and unchanged
  Batch 29.6AK and 29.6AM hashes
- [x] select geometry, frequency, joint localized evidence, or operator review;
  open only a separately frozen selector contract on conclusive evidence

### Batch 29.6AP - Rényi Comparison-Region Geometry Contract

- [x] choose one source-blind comparison geometry within the unchanged Rényi
  evidence family; declared event labels remain proof fixtures only
- [x] retain four resolutions, natural hops, `alpha=0.7`, stereo energy linking,
  legal path, controls, and every non-geometry gate
- [x] freeze implementation, regression, and stop rules before changing the
  selector; keep phase, stretched synthesis, corpus, and routing closed

### Batch 29.6AQ - Anchor-Local Rényi Geometry Decision

- [x] evaluate only natural-hop coefficient centres whose complete window
  support fits inside each anchor's fixed comparison region
- [x] prove exact `[29,13,5,1]` membership, structural closure, unchanged
  invariance gates, full musical gates, and deterministic hashes
- [x] open variable-hop phase contracting only on complete passage; otherwise
  stop automatic-selector research for operator review

### Batch 29.6AR - Automatic Selector Operator Review

- [x] choose whether to retire Rényi automatic selection, authorize a new
  evidence-family contract, or pause the time-adaptive successor lane
- [x] do not relax the failed far-field, mixed-event, or perturbation gates by
  inference; any change requires explicit operator direction
- [x] keep variable-hop phase, stretched synthesis, corpus, dynamic ratio,
  cache, and routing closed until a new ready card exists

### Batch 29.6AS - Transient-Evidence Measurement Contract

- [x] freeze one magnitude-gated mixed-phase-derivative occupancy definition;
  do not combine independent detector votes or copy empirical paper thresholds
- [x] define the pre-analysis grid, normalization, smoothing, peak semantics,
  stereo aggregation, synthetic controls, invariances, stability, and hashes
- [x] open report-only detector implementation on a complete contract; keep
  schedule mapping, phase, stretched synthesis, corpus, and routing closed

### Batch 29.6AT - Transient-Evidence Measurement

- [x] compute the frozen normalized mixed-phase occupancy and peak reports for
  all mono and linked-stereo controls without producing a schedule
- [x] prove declared-event recovery, steady/chirp/noise rejection, dense-event
  resolution, invariance, perturbation stability, finiteness, and exact repeat
- [x] open occupancy-to-window mapping only on complete passage; otherwise
  return the evidence definition to operator review without a parameter sweep

### Batch 29.6AU - Transient-Evidence Operator Review

- [x] choose calibrated mixed-phase research, a different transient evidence
  family, or pause the time-adaptive successor lane
- [x] do not infer permission to add empirical thresholds, smoothing,
  prominence, detector votes, or relaxed control gates
- [x] keep schedule mapping, phase, stretched synthesis, corpus, dynamic ratio,
  cache, and routing closed until a new ready card exists

### Batch 29.6AV - Mixed-Phase Distribution Audit

- [x] measure normalized-magnitude and mixed-phase distributions on the frozen
  controls, stereo variants, and perturbations without producing a detector
- [x] prove cell-accounting, ordered finite quantiles, scale/polarity/stereo
  closure, perturbation coverage, deterministic hashes, and exact repeat
- [x] open calibration contracting only on a stable event/negative separating
  interval; otherwise return the evidence family to operator review

### Batch 29.6AW - Transient-Evidence Family Review

- [x] choose a different transient evidence family or pause the time-adaptive
  successor lane
- [x] do not infer median smoothing, prominence, asymmetric mixed-phase bands,
  a larger calibration grid, or relaxed gates
- [x] keep schedule mapping, phase, stretched synthesis, corpus, dynamic ratio,
  cache, and routing closed until a new ready card exists

### Batch 29.6AX - Median-HPSS Evidence Measurement

- [x] compute the frozen linked-magnitude, `17`-bin percussive median,
  `149`-frame harmonic median, `p=2` soft mask, occupancy, and peak reports
- [x] prove unchanged negative, event, dense, mixed, invariance, perturbation,
  finiteness, boundary, and repeat gates without component synthesis
- [x] open occupancy-to-window contracting only on complete passage; otherwise
  return to operator review without a median-length or mask-power sweep

### Batch 29.6AY - Selector-Abstraction Operator Review

- [x] stop automatic-selector work after repeated report-only rejection and
  classify the current path as diminishing returns
- [x] move automatic selection behind an oracle end-to-end value proof
- [x] keep another detector, the 60-row gate, stereo, dynamic ratio, cache, and
  routing closed

### Batch 29.6AZ - Oracle Adaptive Synthesis Contract

- [x] freeze manifest-declared four-window islands, absolute output-centre
  mapping, actual-hop identity-locked phase transport, and output-side dual OLA
- [x] define synthetic mechanism, 15-row mono objective, `L001`, determinism,
  and stop gates before implementation
- [x] open one oracle candidate without automatic selection, component
  branches, phase reset, local time warp, stereo, or product routing

### Batch 29.6BA - Oracle Adaptive Synthesis Gate

- [x] implement the frozen oracle renderer and prove synthetic identity,
  coverage, placement, phase, integrity, and deterministic evidence
- [x] stop before the 15-row sidecar and candidate render when synthetic event
  placement fails
- [x] keep concealed listening closed and retire the hypothesis after the
  frozen `1.5x` impulse lands `127` frames early

### Batch 29.6BB - Oracle Concealed Listening

- [x] close without export because Batch 29.6BA failed its synthetic stop gate
- [x] do not request listening evidence for a mechanism-rejected candidate
- [x] keep selector research closed

### Batch 29.6BC - Oracle Value Decision

- [x] close the time-adaptive successor lane; oracle scheduling did not preserve
  declared impulse placement under the frozen synthesis policy
- [x] record mechanism rejection without using absent listening as evidence
- [x] keep stereo, dynamic ratio, promotion, cache, and routing closed

### Batch 29.6BD - Rubber Band Behavioural Probe Contract

- [x] freeze generated controls, ratios, R2/R3 modes, public introspection
  fields, rendered-audio measures, hashes, and unsupported-mode policy
- [x] separate directly reported comparator state from waveform inference
- [x] keep new Signal synthesis, automatic selection, promotion, cache, and
  routing closed

### Batch 29.6BE - Rubber Band Behavioural Measurement

- [x] extend the existing external benchmark harness with the frozen synthetic
  probes and deterministic report
- [x] measure local timing, transient phase-treatment, vertical coherence, and
  R3 standard-versus-short multi-resolution signatures
- [x] stop for tool or evidence redesign if signatures do not repeat across
  controls; do not infer architecture from one row

### Batch 29.6BF - Comparator Mechanism Attribution

- [x] classify which quality deltas are owned by local timing, transient phase
  treatment, vertical coherence, or simultaneous resolution
- [x] retain conflicting or R3-opaque behavior as an explicit planning gap
- [x] promote only cross-control signatures into architecture and contract
  after direct public-state evidence closes the time-allocation gap

### Batch 29.6BG - Complete Signal Successor Contract

- [x] freeze one interacting offline study, time-map, phase, resolution, and
  linked-stereo architecture
- [x] replace isolated detector passage with bounded full-system tuning and
  concealed listening checkpoints
- [x] open implementation only when mechanism ownership, validation, tuning
  budget, and terminal stop conditions are explicit

### Batch 29.6BH - Simultaneous Multi-Window Union Proof

- [x] prove the `512/2048/8192` square-root-Hann union frame, exact output-side
  canonical dual, reflected boundaries, and identity reconstruction
- [x] report layer/frame counts, frame bounds, coverage, work, errors, and
  stable coefficient/output hashes across frozen controls
- [x] keep study, schedule modification, phase modification, tuning, corpus,
  promotion, and routing closed until the union passes

### Batch 29.6BI - Study And Local Schedule Proof

- [x] compute linked continuous evidence independently of event application
- [x] select ordered exact points and solve positive bounded integer hops with
  exact final closure and maximum `256`-frame selected-event movement
- [x] prove dense-event retention, disabled-application evidence parity,
  deterministic schedule hashes, and linked-channel decision equivalence

### Batch 29.6BJ - Complete Synthetic Phase And Synthesis Proof

- [x] transport each layer through actual source/output intervals
- [x] prove event correction and cross-resolution vertical alignment are
  separately live without changing schedule or magnitude
- [x] pass identity, exact length, coverage, finiteness, symmetry, tone, event,
  boundary, stereo-decision, and repeat gates

### Batch 29.6BK - Bounded Complete-System Tuning

- [x] run at most `108` complete configurations over the frozen geometry,
  study-sensitivity, event-local strength, reset-scope, and vertical-alignment
  grid
- [x] use hard gates and Pareto evidence to export at most three candidates
- [x] apply the concealed nine-row development gate; reject all three
  successors after four explicit losses make `6/9` unreachable and expose one
  repeatable temporal-smear defect

### Batch 29.6BL - Locked Mono Holdout

- [ ] expose the frozen six-row family-balanced holdout only after selection
- [ ] require at least `4/6` preference over current, no new broad defect, and
  complete hard-gate retention
- [ ] open linked-stereo listening and hardening only on passage; never retune
  after holdout

Status: closed without exposure. No development candidate qualified.

### Batch 29.6BM - Cross-Resolution Smear Attribution

- [x] retain the three rejected successor configurations and nine development
  rows; do not read holdout or tune parameters
- [x] export per-layer and combined report-only evidence for ordinary,
  event-only, vertical-only, and complete phase modes
- [x] measure layer-local replicas, pairwise arrival disagreement, whole-render
  correlation, and combined replica/smear growth
- [x] attribute the shared failure to incoherent independent full-band layer
  transport and return to architecture review

### Batch 29.6BN - Shared Full-Field Phase Proof

- [x] replace independent per-layer synthesis phase state with one physical-
  frequency phase field shared by all three resolutions
- [x] project analysis phase and instantaneous frequency to common atom centres;
  apply event correction once, then project one solved phase back to every layer
- [x] retain frozen study, exact schedule, magnitudes, union dual, development
  rows, and geometry; do not tune or read holdout
- [x] apply the mean pairwise event disagreement below `8` frames, pairwise
  correlation above `0.8`, no combined replica growth, exact layer-sum closure,
  and prior hard gates; reject the proof before listening

### Batch 29.6BO - Non-Duplicating Ownership Architecture Review

- [x] compare complementary source subbands, coefficient-plane partitioning,
  and one invertible adaptive-resolution representation using public primary
  evidence and existing Rubber Band behavioural findings
- [x] require exact unmodified reconstruction, one synthesis owner per
  coefficient, continuous event-local ownership, and one global time map
- [x] select one representation and freeze its ownership, crossover or tiling,
  phase, boundary, and linked-stereo contracts before implementation
- [x] return to operator review if no family supports both exact reconstruction
  and event-local resolution without independent full-band copies

### Batch 29.6BP - Single-Owner Adaptive-Frame Proof

- [x] re-express the passing `512/1024/2048/4096` painless schedule proof as
  one selected window and coefficient vector per analysis centre
- [x] prove zero duplicate centre ownership, selected-frame-only coefficient
  counts, legal transitions, positive coverage, condition at most `4`, and
  bounded reflected support across the frozen declared schedules
- [x] preserve exact diagonal-dual identity, real closure, exact length,
  deterministic repeat, and Rule 26I numerical gates on its frozen controls
- [x] keep study attachment, output-hop modification, phase modification,
  corpus audio, holdout, and tuning closed

### Batch 29.6BQ - Study And Time-Map Attachment Proof

- [x] reuse the frozen linked study, responsive selected points, and Rule 30C
  positive-integer schedules at `0.75`, `1.5`, and `2.0`
- [x] map selected points through the proven adaptive island geometry and give
  every adaptive centre the exact shared `128`-grid output position
- [x] prove one mapping across resolutions, positive output hops, exact endpoint,
  bounded event displacement, linked-order equivalence, and deterministic hashes
- [x] keep coefficient and phase modification, corpus audio, holdout, and tuning
  closed

### Batch 29.6BR - Single-Frame Phase And Synthesis Proof

- [x] reuse the frozen study, adaptive ownership, and exact global output map
  on identity, tone, event, boundary, and linked synthetic controls
- [x] transport one continuous physical-frequency phase state through actual
  source/output hops without resetting at resolution changes
- [x] prove output-lattice coverage and exact diagonal-dual synthesis before
  separately enabling event correction and current-frame vertical locking
- [x] pass Rule 30M structural, identity, tone, event, phase, linked-decision,
  and repeat gates without corpus audio, holdout, tuning, or product routing

### Batch 29.6BS - Adaptive Single-Frame Synthetic Quality Gate

- [x] freeze combined event-plus-vertical mode and ordinary transport as the
  only ablation; make no study, geometry, schedule, peak, or phase-policy change
- [x] run Rule 30N identity, tone, chirp, isolated/dense event, boundary,
  noise, mixed, and silence controls at `0.75`, `1.5`, and `2.0`
- [ ] pass exact structure, identity, pitch, isolated and one-to-one dense-event,
  symmetry, silence, boundary, and repeat gates
- [x] report crest, replicas, static/unsupported spectrum, tonal texture, and
  mode deltas without threshold fitting or corpus reads

### Batch 29.6BT - Phase And Event-Placement Failure Attribution

- [x] freeze the failing Rule 30N tone, isolated-event, and dense-event rows
- [x] trace scheduled centres, synthesis contributions, phase advances, and
  output peaks without changing study, map, window, peak, or phase policy
- [x] separate time-map, phase-transport, event-correction, vertical-locking,
  and diagonal-dual synthesis responsibility for every hard failure
- [x] select one bounded redesign stage; keep parameter search and audio closed

### Batch 29.6BU - Active-Peak Phase And Injected-Event Ownership Proof

- [x] replace dormant-bin phase continuation with one-to-one active-peak state
  and initialize newly active owners from current analysis phase
- [x] separate sample-refined transient anchors from resolution-selection points
  and attach every accepted anchor as an exact source/output frame centre
- [x] pass steady-tone active-owner, event known-answer, structural, identity,
  coefficient, symmetry, finiteness, and repeat gates without quality tuning
- [x] keep complete Rule 30N rerun, corpus, holdout, stereo, and routing closed

### Batch 29.6BV - Successor Synthetic Quality Gate

- [x] rerun the complete Rule 30N matrix only after Batch 29.6BU passes
- [x] require all prior structure, pitch, event, silence, and repeat limits
- [x] report the unchanged crest, replica, spectrum, texture, and mode fields
- [x] return to the owning stage on failure; do not fit thresholds

### Batch 29.6BW - Dense-Event Replica Attribution

- [x] freeze the successor `DenseEvent` rows and the sole `2.0x` hard failure
- [x] trace both exact anchors, scheduled targets, dominant output peaks,
  active-owner/event state, and every overlapping diagonal-dual contribution
- [x] compare the failing row with passing `0.75x` and `1.5x` rows and the
  frozen ordinary ablation without changing renderer policy
- [x] assign the earliest failure to anchor placement, event reset,
  active-owner transport, overlap synthesis, or metric association; do not tune

### Batch 29.6BX - Event-Local Overlap Ownership Proof

- [x] trace every frame contribution at the dominant non-target dense-event
  replica before changing synthesis
- [x] give each injected attack one bounded output-domain owner while retaining
  complementary overlap weights and exact target amplitudes
- [x] reject non-target inter-anchor replicas without moving either exact anchor
  or weakening the unchanged dense one-to-one limit
- [x] rerun the complete Rule 30Q matrix only after the bounded proof passes

### Batch 29.6BY - Frozen Mono Development Objective Comparison

- [x] open only after the successor passes Rule 30Q without tuning
- [x] freeze family-balanced development rows and compare the selected
  candidate with current Signal and captured external behavioural evidence
- [x] report the full integrity, transient, replica, spectral, texture, formant,
  and boundary field set before any concealed listening export
- [x] keep holdout, parameter search, linked stereo, dynamic ratio, and product
  routing closed

### Batch 29.6BZ - Real-Source Synthesis-Stage Attribution

- [x] compare current, ordinary adaptive, tracked/no-anchor, tracked/anchor, and
  event-owned stages on the unchanged nine development rows
- [x] assign the broad event, replica, static-spectrum, and formant regression
  before designing another candidate
- [x] keep holdout, listening export, tuning, stereo, dynamic ratio, cache, and
  product routing closed

### Batch 29.6CA - Ordinary Resolution And Transition Attribution

- [x] compare fixed `512`, `1024`, `2048`, and `4096` ordinary controls with
  adaptive ordinary synthesis on the unchanged nine development rows
- [x] assign the defect to a fixed resolution, adaptive transitions, or the
  shared ordinary phase/output-lattice mechanism before redesign
- [x] keep holdout, listening export, tuning, detector/schedule policy, stereo,
  dynamic ratio, cache, and product routing closed

### Batch 29.6CB - Ordinary Shared-Mechanism Factor Attribution

- [x] freeze the clean-integrity fixed `4096` control and compare the current
  event-warped output lattice with a global linear lattice
- [x] separate phase-transport changes from lattice placement and diagonal-dual
  overlap synthesis before proposing another complete candidate
- [x] keep the window bank, detector, event schedule, measurements, holdout,
  listening, stereo, dynamic ratio, cache, and routing frozen

### Batch 29.6CC - Window-Kernel Attribution

- [x] hold fixed `4096`, event-warped placement, ordinary transport, and exact
  dual normalization while crossing square-root-Hann and Hann analysis and
  synthesis kernels
- [x] assign broad spectral/formant damage to analysis leakage, synthesis
  weighting, their interaction, or the remaining coefficient path
- [x] keep resolution, detector, schedule, measurements, holdout, listening,
  stereo, dynamic ratio, cache, and routing frozen

### Batch 29.6CD - Coefficient-Geometry Attribution

- [x] hold Hann analysis/synthesis, ordinary transport, exact dual
  normalization, rows, ratios, and measurements fixed
- [x] compare centered/reflected `2048` analysis on shared `4096` and native
  `2048` FFT grids with start-aligned padded native-`2048` geometry
- [x] assign the remaining broad regression to FFT zero-padding, frame/boundary
  geometry, or the remaining phase/magnitude path before candidate design

### Batch 29.6CE - Coefficient-Path Design Checkpoint

- [x] consolidate Rules 30W through 30Y and behavioural-forensics evidence
  without rendering another candidate
- [x] freeze one native-grid, reflection-preserving coefficient path that owns
  cross-bin phase coherence and transient replicas together
- [x] stop for a named research question if one bounded complete design cannot
  be supported; do not reopen factor sweeps

### Batch 29.6CF - Native-Grid Active-Owner Synthetic Proof

- [x] compose the existing single-owner schedule, fixed analytic tracker,
  sample-refined anchors, conflicted-bridge owner, and exact dual around native
  Hann/Hann synthesis coefficients
- [x] project active physical-frequency owners onto each native FFT grid while
  preserving native magnitudes and native within-region analysis-phase offsets
- [x] run the mechanism controls and complete `48`-row synthetic quality gate;
  reject on three stretched `55 Hz` rows before any real-source render

### Batch 29.6CG - Source-Studied Architecture Reset

- [x] stop Rule 30AB and retire the time-adaptive full-band candidate
- [x] inspect pinned Signalsmith Stretch and Rubber Band R2/R3 source with an
  explicit licence and provenance boundary
- [x] promote simultaneous frequency-partitioned resolution as the target and
  fixed-grid weighted phase prediction as the control
- [x] freeze one complete-system proof instead of another parameter or
  per-metric repair sequence

### Batch 29.6CH - Source-Studied Complete Architecture Proof

- [ ] add Signalsmith Stretch to the frozen synthetic and nine-row development
  comparator set
- [ ] implement one report-only frequency-partitioned long/middle/short path
  with guidance-only classification and explicit phase states
- [ ] retain one fixed-grid weighted multi-predictor control under the same
  schedule, boundary, and measurement contract
- [ ] run the complete synthetic gate, all nine mono development rows, and one
  concealed listening pack only after hard integrity passes
- [ ] decide on the whole architecture; do not open parameter lattices or
  per-metric repair batches

### Batch 29.6CI - Mono Decision Checkpoint

- [ ] open only after the frozen nine-row development evidence is complete
- [ ] decide whether the complete architecture earns continuation or returns to
  research as a whole
- [ ] keep holdout, linked stereo, dynamic ratio, cache, production routing,
  and parameter search closed

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
- 2026-07-11: Batch 29.6N stopped before coefficient assembly. Lowpass channel
  `0` requires a guard beyond `16384` frames and retains `6.270779e-7` excluded
  energy at the largest legal support radius despite a `1.051210e-12` dual
  residual. Evidence is in
  `docs/logs/2026-07/11-g10-029-common-grid-guard-rejection.md`.
- 2026-07-11: Froze Batch 29.6O. Five representative channels, three response
  stages, two spectrum forms, six radii, and four thresholds now attribute the
  tail without changing geometry. Evidence is in
  `docs/logs/2026-07/11-g10-029-dual-atom-tail-attribution-contract.md`.
- 2026-07-11: Batch 29.6O attributed the tail. Tightening amplifies the DC
  real-output tail by `3865790.426x`; dualization is neutral there. Nyquist has
  a raw `1.180453e-7` edge tail and worsens to `2.030199e-7`. Interior controls
  are compact. Evidence is in
  `docs/logs/2026-07/11-g10-029-dual-atom-tail-attribution-proof.md`.
- 2026-07-11: Froze Batch 29.6P. One untightened-bank candidate retains raw
  channels `0..1534` and replaces only channel `1535` with an endpoint-flat
  Nyquist completion. Reconstruction, representative guard, all-channel guard,
  then phase-mechanism reproof are ordered stop gates. Evidence is in
  `docs/logs/2026-07/11-g10-029-boundary-completion-contract.md`.
- 2026-07-11: Rejected Batch 29.6P at reconstruction conditioning. Exact
  canonical-dual identity passes, but frame condition ratio `2.980258951`
  exceeds `1.25`. Representative guards did not run. Evidence is in
  `docs/logs/2026-07/11-g10-029-boundary-completion-rejection.md`.
- 2026-07-11: Froze Batch 29.6Q. One common scalar uses exact inverse-square-root
  frame energy in the interior and quintic blends to constant endpoint values
  over the existing `16h` spans. Reconstruction remains the first stop gate.
  Evidence is in
  `docs/logs/2026-07/11-g10-029-boundary-preconditioner-contract.md`.
- 2026-07-11: Rejected Batch 29.6R at reconstruction conditioning. The
  endpoint-even scalar reaches condition ratio `3.0185626163`; exact identity
  passes, but representative guards did not run. Evidence is in
  `docs/logs/2026-07/11-g10-029-boundary-preconditioner-rejection.md`.
- 2026-07-11: Froze Batch 29.6S. Three fixed banks, all `11` alias residues,
  extremal eigenpairs, boundary-bin mass, cross-bank Rayleigh transfer, and
  bounded channel/cross-term attribution now precede any redesign. Evidence is
  in `docs/logs/2026-07/11-g10-029-alias-block-attribution-contract.md`.
- 2026-07-11: Batch 29.6T stopped as inconclusive. The full matrix repeats and
  contribution closure passes, but worst eigenpair residual `0.031864856`
  exceeds `1e-6`. Evidence is in
  `docs/logs/2026-07/11-g10-029-alias-block-attribution-inconclusive.md`.
- 2026-07-11: Froze Batch 29.6U. One full lexicographic cyclic complex-Hermitian
  Jacobi solve now owns bounded alias-block eigenpairs and invariant checks.
  Evidence is in
  `docs/logs/2026-07/11-g10-029-hermitian-eigensolver-contract.md`.
- 2026-07-11: Batch 29.6V passed all analytic and `33` alias-matrix gates with
  maximum eigenpair residual `9.186641e-13`. Evidence is in
  `docs/logs/2026-07/11-g10-029-hermitian-eigensolver-proof.md`.
- 2026-07-11: Batch 29.6W selected boundary geometry. Exact-pointwise condition
  `2.9916436058` exceeds `1.25`; no preconditioner or guard opens. Evidence is
  in `docs/logs/2026-07/11-g10-029-jacobi-attribution-decision.md`.
- 2026-07-11: Froze Batch 29.6X. Exact extrema isolate residue `0`, bins `2101`
  and `2112`, and channel `1535` cross terms near `+/-0.492`; one three-operator
  ablation must prove ownership before filter design. Evidence is in
  `docs/logs/2026-07/11-g10-029-boundary-geometry-reassessment-contract.md`.
- 2026-07-11: Completed Batch 29.6Y. Off-diagonal-only removal reduces global
  condition from `2.9916436058` to `1.1141796230`; complete channel removal
  remains rejected at `2.6496906694`. Orthogonal or multi-row completion
  research is selected. Evidence is in
  `docs/logs/2026-07/11-g10-029-nyquist-alias-coupling-ablation.md`.
- 2026-07-11: Froze Batch 29.6Z. The one allowed candidate splits the existing
  completion across delays `-128`, `0`, and `+128`; three-point DFT coding
  preserves diagonal energy and analytically cancels every possible same-residue
  completion cross term. Evidence is in
  `docs/logs/2026-07/11-g10-029-three-row-nyquist-completion-contract.md`.
- 2026-07-11: Batch 29.6AA rejected the triplet. Its analytic completion
  closure passes below `1e-14`, but complete condition is `2.0862893665` with
  limiting residues `3` and `8`. Reconstruction remains closed. Evidence is in
  `docs/logs/2026-07/11-g10-029-three-row-nyquist-completion-rejection.md`.
- 2026-07-11: Froze Batch 29.6AB. One four-operator ablation isolates cross
  coupling from raw DC rows `0..15`, preserved high-edge rows `1520..1534`, or
  both before another geometry is proposed. Evidence is in
  `docs/logs/2026-07/11-g10-029-residual-boundary-attribution-contract.md`.
- 2026-07-11: Batch 29.6AC selected complete raw-bank reassessment. DC
  diagonalization is neutral and high-edge diagonalization worsens condition to
  `2.1170081614`; boundary cross terms are insufficient. Evidence is in
  `docs/logs/2026-07/11-g10-029-residual-boundary-attribution-decision.md`.
- 2026-07-11: Froze Batch 29.6AD. One exact canonical block tightener gets the
  final common-grid feasibility gate; compact-support leakage and all-row atom
  localization, not guaranteed condition, decide the family. Evidence is in
  `docs/logs/2026-07/11-g10-029-canonical-block-tightener-contract.md`.
- 2026-07-11: Batch 29.6AE closed the common-grid family. Canonical tightening
  reaches condition `1.0000000000005773`, but row `12` violates frozen compact
  support at peak `1.2528705611e-12`. Evidence is in
  `docs/logs/2026-07/11-g10-029-canonical-block-tightener-rejection.md`.
- 2026-07-11: Batch 29.6AF selected one dense painless common-lattice proof.
  It reuses the passing Batch 29.6I filters and diagonal dual, replacing only
  unequal per-band scheduling with the largest original coefficient count.
  Evidence is in
  `docs/logs/2026-07/11-g10-029-transform-family-reassessment.md`.
- 2026-07-11: Batch 29.6AG rejected the dense painless candidate. Identity and
  condition pass, but redundancy is `208`, real-spectrum closure is
  `1.7881393433e-7`, and limiting atom leakage at radius `16384` is
  `0.4999847412`. Evidence is in
  `docs/logs/2026-07/11-g10-029-dense-painless-rejection.md`.
- 2026-07-11: Batch 29.6AH records operator authorization to continue
  transform research without relaxing failed gates. Public frame and 2026
  TSM evidence selects time-adaptive painless NSDGT reconstruction as the next
  bounded question. Evidence is in
  `docs/logs/2026-07/11-g10-029-time-adaptive-transform-research.md`.
- 2026-07-11: Batch 29.6AI passed declared-schedule reconstruction. All five
  schedules reconstruct eleven controls with adaptive condition at most
  `1.5934675721`, peak error `7.2164496601e-16`, complete coverage, compact
  support, real output, and exact repeat. Evidence is in
  `docs/logs/2026-07/11-g10-029-time-adaptive-reconstruction-proof.md`.
- 2026-07-11: Batch 29.6AJ froze one automatic selector: normalized local
  `alpha=0.7` Rényi entropy, followed by one legal minimum-cost resolution path.
  Evidence is in
  `docs/logs/2026-07/11-g10-029-renyi-resolution-selection-contract.md`.
- 2026-07-11: Batch 29.6AK rejected the raw Rényi selector. A single impulse
  selects `512` across `36/64` anchors, the linear chirp stays all-short, and
  mixed tonal/transient audio stays all-long. Stability and equivalence gates
  pass. Evidence is in
  `docs/logs/2026-07/11-g10-029-renyi-resolution-selection-rejection.md`.
- 2026-07-12: Batch 29.6AL froze selector-failure attribution. Eight exact
  time slices and eight folded-frequency regions must close the unchanged
  Batch 29.6AK evidence before bounded leave-one-region-out diagnostics can
  choose one selector boundary. Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-selector-attribution-contract.md`.
- 2026-07-12: Batch 29.6AM stopped inconclusive. Time removal restores `8/15`
  isolated anchors but changes `5/32` negatives; low-band removal restores all
  five mixed events but changes one negative. Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-selector-attribution-inconclusive.md`.
- 2026-07-12: Batch 29.6AN froze one final attribution refinement. Declared
  event-support membership and eight fixed subdivisions of folded region `0`
  must separate the coupled mechanisms or stop selector research for operator
  review. Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-attribution-reassessment-contract.md`.
- 2026-07-12: Batch 29.6AO selected comparison-region geometry. Event-support
  removal restores all `15` isolated anchors with no negative changes; narrow
  low-band removal is nonselective. Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-attribution-reassessment-decision.md`.
- 2026-07-12: Batch 29.6AP froze anchor-local support-contained geometry. Every
  decision uses natural-hop symmetric centres with exact membership
  `[29,13,5,1]`; full selector passage or operator review is the next terminal
  gate. Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-comparison-geometry-contract.md`.
- 2026-07-12: Batch 29.6AQ rejected the terminal geometry. Membership and
  support pass, but isolated far-field recovery, mixed-event recovery, and the
  perturbation cap fail. Automatic selector work stops for operator review.
  Evidence is in
  `docs/logs/2026-07/12-g10-029-renyi-comparison-geometry-rejection.md`.
- 2026-07-12: Batch 29.6AR records operator direction to retire Rényi-only
  automatic selection and research one percussive-occupancy evidence family.
  Component separation and SELEBI phase/hop policy remain excluded. Evidence is
  in `docs/logs/2026-07/12-g10-029-transient-evidence-direction.md`.
- 2026-07-12: Batch 29.6AS froze one analytic transient detector measurement:
  centered mixed phase on a `2048/128/4096` grid, a scale-relative numerical
  floor, midpoint mask, linked occupancy, no smoothing, and a `0.5` local peak.
  Evidence is in
  `docs/logs/2026-07/12-g10-029-transient-evidence-measurement-contract.md`.
- 2026-07-12: Batch 29.6AT rejected the analytic detector. Every non-event
  negative family produces peaks; event localization, dense resolution,
  perturbation, and one stereo occupancy gate fail. Evidence is in
  `docs/logs/2026-07/12-g10-029-transient-evidence-rejection.md`.
- 2026-07-12: Batch 29.6AU authorizes bounded mixed-phase calibration research.
  The primary method's empirical values are evidence anchors, not defaults;
  Batch 29.6AV must prove distribution separation before threshold fitting.
  Evidence is in
  `docs/logs/2026-07/12-g10-029-mixed-phase-calibration-direction.md`.
- 2026-07-12: Batch 29.6AV rejected mixed-phase calibration. All `25` audit
  pairs overlap, chirp leakage remains at least `0.7759762445`, and boundary
  stereo equivalence changes by `2.6562923909e-5`. Evidence is in
  `docs/logs/2026-07/12-g10-029-mixed-phase-distribution-rejection.md`.
- 2026-07-12: Batch 29.6AW selects one median-HPSS evidence definition. It
  preserves the primary method's physical time support, uses a `p=2` soft mask,
  and excludes component audio. Evidence is in
  `docs/logs/2026-07/12-g10-029-median-hpss-evidence-contract.md`.
- 2026-07-12: Batch 29.6AX rejected median-HPSS event detection. Stereo passes,
  but every negative family peaks, all event families fail, and three impulse
  controls fail perturbation. Evidence is in
  `docs/logs/2026-07/12-g10-029-median-hpss-evidence-rejection.md`.
- 2026-07-12: Batch 29.6AY stops automatic-selector churn. Batch 29.6AZ freezes
  one oracle-scheduled end-to-end candidate; automatic selection reopens only
  after objective and concealed-listening value. Evidence is in
  `docs/logs/2026-07/12-g10-029-oracle-adaptive-refocus.md`.
- 2026-07-12: Batch 29.6BA rejects oracle time-adaptive synthesis at the
  synthetic gate. Identity, schedule legality, exact mapping, output coverage,
  finiteness, symmetry, and deterministic repeat pass across the frozen
  controls. The `1.5x` isolated impulse lands `127` frames early. Batches
  29.6BB and 29.6BC therefore close without corpus rendering or listening.
  Evidence is in
  `docs/logs/2026-07/12-g10-029-oracle-adaptive-synthesis-rejection.md`.
- 2026-07-12: Operator review rejects abandonment of Signal-native quality and
  identifies the research constraint failure. Public Rubber Band behavior
  shows local adaptive timing, transient phase treatment, vertical coherence,
  offline study, and R3 simultaneous multi-resolution processing were excluded
  or isolated by Signal's previous rules. Rule 29 and Batches 29.6BD-BG reopen
  the complete-system research space. Evidence is in
  `docs/research/translation-memos/002-rubber-band-behavioural-forensics.md`.
- 2026-07-12: Batch 29.6BD freezes a `264`-row synthetic matrix over five
  controlled R2/R3 modes, four mono ratios, and bounded linked-stereo controls.
  Direct public-API state, render receipts, and waveform inference have
  separate schemas and hashes. Installed Rubber Band `4.0.0` supports every
  required CLI mode; public headers and libraries are present but adapter
  capability must be receipted. Evidence is in
  `docs/logs/2026-07/12-g10-029-rubber-band-behavioural-probe-contract.md`.
- 2026-07-12: Batch 29.6BE passes the synthetic measurement gate. All `264`
  rows have exact length, finite unclipped output, and bit-identical repeat
  renders. The report retains distinct R2 reset/lamination and R3
  standard/short signatures. Public direct-state evidence is honestly
  unsupported pending an adapter. Evidence is in
  `docs/logs/2026-07/12-g10-029-rubber-band-behavioural-measurement.md`.
- 2026-07-12: Batch 29.6BF promotes waveform-owned mechanism boundaries.
  Local timing, event phase treatment, vertical coherence, and simultaneous
  resolution all produce repeatable cross-control signatures. The exact local
  allocator stays open because the C API omits output increments, reset curves,
  and exact-time points. BF remains active for one public C++ state adapter.
  Evidence is in
  `docs/logs/2026-07/12-g10-029-rubber-band-mechanism-attribution.md`.
- 2026-07-12: Batch 29.6BF closes with `48/48` repeated public R2 state rows.
  No-reset retains detector curves but changes every exact-point and increment
  sequence; no-lamination changes no study state. Contract `082` now separates
  study, timing constraints, local schedule, event phase, and vertical phase.
  R3 direct state remains publicly unsupported.
- 2026-07-12: Batch 29.6BG freezes the complete Signal successor under Rule 30:
  linked offline study, bounded exact local schedule, simultaneous multi-window
  union frame, separate event/vertical phase stages, canonical-dual synthesis,
  linked decisions, a `108`-configuration ceiling, concealed development, and a
  locked holdout. Batches 29.6BH-BL are ready in sequence.
- 2026-07-12: Batch 29.6BH proves the simultaneous `512/2048/8192` union.
  The combined square-root-Hann frame operator is `6.0` with condition
  `1.0000000000000007`; all padded and source samples are covered. Six frozen
  controls reconstruct with peak error below `7.78e-16`, RMS error below
  `1.46e-16`, no non-finite values, stable hashes, and exact repeated evidence.
  Study, local scheduling, and phase modification remained absent.
- 2026-07-12: Batch 29.6BI proves linked study and exact local scheduling.
  Three linked-channel controls at `0.75x`, `1.5x`, and `2.0x` retain `15`
  responsive and `4` conservative points, including four dense-region points.
  Enabled/disabled evidence and channel-order decisions are exact. Every
  schedule has positive bounded hops, zero selected-event movement, exact final
  closure, deterministic hashes, and measurable event-local unity improvement.
- 2026-07-12: Batch 29.6BJ proves complete synthetic phase and synthesis.
  Ordinary transport uses actual source/output intervals in every layer. Event
  correction performs `34,952` short-layer bin resets; vertical alignment makes
  `2,016` projected-reference assignments. Both stages are separately live
  without schedule or magnitude changes. Exact length, coverage, identity,
  finiteness, symmetry, tone, event, boundary, linked-decision, and repeat gates
  pass across `0.75x`, `1.0x`, and `1.5x` linked-channel controls.
- 2026-07-12: Batch 29.6BK configuration checkpoint freezes exactly `108`
  unique complete-system configurations across the contracted `3x2x3x3x2`
  dimensions. The 15 existing listening rows are partitioned before tuning into
  nine development rows and six disjoint holdout rows with family counts
  `2/2/2/1/2` and `1/1/1/2/1`. No grid render or holdout exposure has occurred.
- 2026-07-12: Batch 29.6BK renderer checkpoint wires every frozen dimension
  into the complete renderer. Geometry changes analysis and union synthesis;
  sensitivity changes exact points; unity strength changes the exact-closing
  schedule; all three reset scopes perform their contracted ownership; vertical
  alignment toggles only its projected-reference phase stage. Nine focused
  configurations pass exact length, coverage, finiteness, boundary, event-order,
  linked-decision, repeat, and prior BH-BJ regression gates.
- 2026-07-12: Batch 29.6BK executes all `108` configurations over synthetic and
  `972` frozen development-row renders. `68` pass every hard gate; `25` form the
  Pareto frontier. All `36` short-geometry configurations and four nonvertical
  short-only variants fail the combined identity/pitch/event gate. No length,
  coverage, finiteness, boundary, ordering, movement, repeat, or linked-decision
  failures occur. Three representatives are selected for concealed export;
  holdout reads remain zero.
- 2026-07-12: Batch 29.6BK exports the concealed nine-row development pack.
  Every row contains one mono reference and five shared-level candidates: three
  frontier successors, current Signal, and Rubber Band R3. The pack contains
  `54` WAVs, nine notes rows, `45` concealed assignments, zero structural
  failures, and zero holdout reads. Candidate selection is now paused for
  operator listening; the key remains closed.
- 2026-07-13: Batch 29.6BK closes rejected. Operator notes were frozen before
  key reveal. On `L001`, `L002`, `L004`, and `L005`, every successor was in the
  unusable blurred set while every preference was current Signal or Rubber
  Band. Even five wins on the remaining rows could reach only `5/9`; no
  successor can pass. The repeated defect sounds reverberant or like multiple
  micro-delayed source copies. Holdout remains unread. Batch 29.6BM now owns
  report-only cross-resolution coherence attribution without retuning.
- 2026-07-13: Batch 29.6BM attributes the defect across `108` frozen
  development renders. Complete-mode layer arrivals disagree by `172.776515`
  frames on average and up to `507`; pairwise correlation is `0.197448` and
  recombination raises mean replica count from `36.348485` to `38.494318`.
  Ordinary, event-only, vertical-only, and complete modes share the failure.
  Layer-sum closure is `3.34e-16`; holdout reads remain zero. Independent
  full-band phase transport is retired. Batch 29.6BN owns one shared full-field
  phase proof without tuning.
- 2026-07-13: Batch 29.6BN rejects shared full-field phase transport. All prior
  structural gates pass and layer-sum error is `1.67e-16`, but mean layer
  arrival disagreement remains `162.261364` frames against `<8`, correlation is
  `0.134045` against `>0.8`, and recombination still adds `0.710227` replicas
  per event. No tuning or holdout read occurs. Redundant full-band union
  ownership is closed; Batch 29.6BO owns a non-duplicating representation
  review before more synthesis code.
- 2026-07-13: Batch 29.6BO selects one time-adaptive painless nonstationary
  Gabor frame. Fixed complementary subbands lack event-local resolution without
  time-varying PR transitions; generic coefficient quilts leave exact local
  dual and phase topology unresolved. The selected family already passes
  Signal's declared-schedule reconstruction below `1e-15`. Rule 30K freezes one
  window and coefficient vector per centre, one global time map, exact diagonal
  dual, bounded reflection, and linked decisions. Batch 29.6BP owns the
  single-owner implementation proof before stretched phase resumes.
- 2026-07-13: Batch 29.6BP passes all five declared schedules. Every centre has
  one window and coefficient vector; duplicate ownership and count mismatches
  are zero. Selected coefficients remain within the fixed `161`-frame bound,
  the original identity hash stays `6987080e517f1aec`, and ownership hash
  `2a29d952d91e92ba` repeats. Batch 29.6BQ owns study and one-global-map
  attachment without coefficient or phase modification.
- 2026-07-13: Batch 29.6BQ passes all three frozen linked controls. Each has
  `15` selected points, `104` adaptive frames, exact per-level attachment to
  the shared `128`-grid schedule, positive source/output hops, zero structural
  or mapping failures, zero selected-event movement, and exact endpoint and
  linked-order agreement. Evidence hash `3ea1d3a2297083e2` repeats; the prior
  identity and ownership hashes are unchanged. Batch 29.6BR owns output-lattice
  coverage and one single-frame phase/synthesis proof.
- 2026-07-13: Batch 29.6BR passes four frozen controls. All ratios retain `104`
  selected frames and `24` resolution changes with one continuous phase-state
  initialization per channel. Output coverage is complete; maximum frame
  condition is `2.964471`; identity peak error is `1.334183e-12`; tone error
  is at most `0.5 Hz`; known-event error is at most the frozen `256`-frame
  limit. Structural, linked-order, symmetry, residue, finiteness, and repeat
  gates pass. Evidence hash `9cc7519deb368966` repeats. Batch 29.6BS owns
  synthetic quality evidence before any corpus read.
- 2026-07-13: Batch 29.6BS rejects the frozen combined mode. Across `48`
  control/ratio cases and ordinary-plus-combined evidence, structure, identity,
  coefficient ownership, silence, symmetry, finiteness, and repeat pass, but
  pitch and event placement produce `25` hard failures and one combined-mode
  regression. Maximum angular-frequency error is `6.842e-4` radians/sample
  against `1e-6`; isolated-event error reaches `496` frames against `1`; dense
  one-to-one error reaches `896` against `256`. Evidence hash
  `6781d49348dfa931` repeats. Corpus audio remains closed. Batch 29.6BT owns
  trace-only phase and event-placement attribution before any redesign.
- 2026-07-13: Batch 29.6BT classifies all `25` frozen failures from `2,298`
  per-frame phase records and `78` overlapping diagonal-dual contributions.
  Fourteen failures begin in ordinary physical-frequency phase transport, ten
  begin at event ownership/frame attachment, and the one combined-only failure
  begins at event correction; vertical locking and diagonal-dual synthesis own
  no earliest failure. Dominant ownership changes `738` times; maximum traced
  frequency error is `3.174e-2` radians/sample and is larger away from
  resolution transitions than on them. None of `18` injected event instances
  is selected and only six land on an exact frame centre. Evidence hash
  `ddca308a7f60f39e` repeats. Batch 29.6BU owns active-peak phase and separate
  injected-event ownership without quality tuning.
- 2026-07-13: Batch 29.6BU passes all `32` mechanism rows. One-to-one active
  owners report `4,976` births, `46,588` matches, `4,960` retirements, and
  `5,204,460` region assignments. Maximum rendered and matched-owner interior
  tone errors are `8.211e-7` and `5.919e-7` radians/sample. Independent linked
  onset evidence detects and exactly attaches all `24/24` expected anchors;
  all eight hard failure classes are zero. Identity errors stay below
  `6.674e-16`; evidence hash `a2d3fb95545cb47f` repeats. The dense-event
  rendered-peak diagnostic still reaches `262` frames and remains visible for
  the unchanged Rule 30N quality gate. Corpus, holdout, listening, tuning,
  stereo, dynamic ratio, and routing remain closed. Batch 29.6BV owns the full
  successor synthetic quality rerun.
- 2026-07-13: Batch 29.6BV rejects the successor on one of `48` frozen rows.
  `DenseEvent` at `2.0x` places the first dominant peak exactly but the second
  at `262` frames from target against the unchanged `256` limit. The other
  successor hard checks pass with zero regressions: tone error `8.211e-7`,
  isolated-event error `0`, identity peak error `9.992e-16`, zero symmetry
  error, and `2.734e-13` maximum imaginary residue. Condition is `4.941683`;
  maximum crest and replica fields are `27.101174 dB` and `1.287973`. Full
  texture and mode-delta fields remain in evidence hash `c72c005d0cd44e3e`.
  No threshold or DSP policy changes. Batch 29.6BW owns trace-only dense-event
  replica/overlap attribution; mono comparison remains closed.
- 2026-07-13: Batch 29.6BW assigns the sole successor failure to overlap
  synthesis. Both `2.0x` attacks remain exact at outputs `16126` and `16644`
  with amplitudes `1.0` and `0.75`; anchor attachment, event reset, active-owner
  state, and complex contribution closure all pass. Overlapping synthesis
  creates a third peak at `16382` with amplitude `0.787177`, so the unchanged
  one-to-one matcher selects that midpoint replica instead of the second real
  attack. Ordinary errors are `[[463,401],[219,351],[896,509]]`; successor
  errors are `[[0,0],[0,0],[0,262]]`. All `49` traced contributions close with
  zero real error and at most `6.770e-17` imaginary residue. Evidence hash
  `2336b9773c32b2ca` repeats. Batch 29.6BX owns one bounded event-local overlap
  ownership proof; mono comparison remains closed.
- 2026-07-13: Batch 29.6BX removes the dense midpoint replica without changing
  either real attack. One `512`-frame bridge at source `8192`, output `16385`,
  owns the complete `0.787177` replica at `16382`. The successor now substitutes
  bounded interpolated background only when a non-anchor frame straddles
  multiple anchors whose projected owner supports no longer overlap. Exactly
  two dense-control samples change at `2.0x`; `0.75x` and `1.5x` remain
  bit-identical. The replica becomes zero, both target amplitudes are unchanged,
  and dense errors become `[[0,0],[0,0],[0,0]]`. Bounded evidence hash
  `adf37bdd72012e19` repeats. The complete unchanged `48`-row Rule 30Q matrix
  passes with zero hard failures or regressions; evidence hash
  `dec15b718aa27de9` repeats. Batch 29.6BY owns the frozen nine-row mono
  development objective comparison. Holdout and listening remain closed.
- 2026-07-13: Batch 29.6BY rejects the event-owned successor before listening.
  All `27` current/candidate/external renders pass exact length, finiteness, and
  full-render integrity, but the candidate regresses current event placement in
  `6/9` rows, replica ratio in `7/9`, static spectral residual in `9/9`, and
  formant residual in `9/9`. Tonal movement improves in `7/9`, but does not
  offset the broad regression. Five source excerpts require a declared
  strongest-onset fallback for event-only fields; the spectral rejection is
  independent. Frozen evidence hashes are `2abde0a10417b469`,
  `4359fd9e43ff6a9c`, `18823a809bb4b2cc`, and `10d25f8404262480`.
  Holdout reads and listening exports remain zero. Batch 29.6BZ owns
  synthesis-stage attribution before another candidate.
- 2026-07-13: Batch 29.6BZ assigns the dominant real-source regression to
  ordinary adaptive synthesis. Its transition from current worsens timing in
  `8/9` rows, replicas in `7/9`, and static-spectrum and formant residuals in
  `9/9`; mean deltas are `+196.166667`, `+0.116000`, `+0.084362`, and
  `+0.048668`. Seven ordinary renders fail endpoint-energy integrity. Active
  tracking repairs most timing and some spectral/formant damage, anchors make
  smaller mixed changes, and event-local ownership changes `0/9` outputs.
  Frozen hashes are `59fde9d5897fe070`, `43806ef3d1b3a311`,
  `30b29a8a65b50861`, and `557eaf8e6c9ee5c5`. Holdout and listening remain
  closed. Batch 29.6CA owns fixed-resolution versus transition attribution.
- 2026-07-13: Batch 29.6CA splits the ordinary defect across three owners.
  Endpoint integrity is resolution-dependent: fixed `512`, `1024`, `2048`,
  and `4096` fail `9/9`, `9/9`, `4/9`, and `0/9`; adaptive fails `7/9` across
  `214` resolution changes. Every fixed control and adaptive ordinary regresses
  static-spectrum and formant residual in `9/9` rows, assigning that damage to
  the shared ordinary mechanism. Adaptive timing is worse than each fixed
  control in `5` to `7` rows and has the largest mean loss at `+196.166667`
  frames, exposing a separate transition cost. Frozen hashes are
  `c4cde9a638c1e36e`, `9a3ff69ddc1dc765`, `3e4f4a8489a8217d`, and
  `c00d6c130888505a`. Holdout and listening remain closed. Batch 29.6CB owns
  phase-transport versus output-lattice attribution on fixed `4096`.
- 2026-07-13: Batch 29.6CB excludes output placement, phase transport, and the
  exact diagonal dual as primary owners of the broad timbral regression. Under
  transported diagonal-dual synthesis, moving to a global-linear lattice adds
  only `0.000538` mean static residual and `0.000676` formant residual. Removing
  phase transport worsens static residual in `9/9` rows on both lattices.
  Replacing the diagonal dual with an analysis-window partition worsens both
  static and formant residual in `9/9` rows on both transported lattices. All
  eight factor modes still regress both fields in `9/9` rows against current
  Signal. Frozen hashes are `63d64c56e0e402bb`, `671bfeb418981df8`,
  `aaf112446dc0f0a8`, and `3c9f3f66ae65d5c1`. Holdout and listening remain
  closed. Batch 29.6CC owns window-kernel attribution.
- 2026-07-13: Batch 29.6CC shows Hann analysis and synthesis both help but do
  not own the broad timbral defect. Moving from square-root-Hann to Hann
  analysis reduces mean static residual by `0.003732` to `0.003815`; Hann
  synthesis reduces it by `0.005078` to `0.005161`. Hann/Hann cuts mean timing
  loss from `82.027778` to `41.333333` frames and lowers mean static/formant
  residual deltas from `0.087938/0.049590` to `0.079045/0.046138`. Every
  combination still regresses both fields in `9/9` rows. Frozen hashes are
  `7d7886402f662bc7`, `76298cafc83779af`, `a2173e14c6eb7535`, and
  `1f7a65480074cf7b`. Holdout and listening remain closed. Batch 29.6CD owns
  FFT-grid and frame/boundary geometry attribution.
- 2026-07-13: Batch 29.6CD assigns shared-grid zero-padding as a contributor and
  the remaining phase/magnitude path as the broad defect owner. Moving centered
  reflected Hann/Hann `2048` frames from shared `4096` to native `2048` lowers
  mean timing, static, and formant deltas by `32.194444`, `0.040495`, and
  `0.017523`, but raises replica ratio by `0.842327`. Start-aligned zero padding
  then gives back `0.029572/0.011684` static/formant residual. Every candidate
  still regresses both timbral fields in `9/9` rows. Frozen hashes are
  `55021268ac0cb16f`, `d788ea7642e16b09`, `b56a87e849ff3f5a`, and
  `fcd42c867eef4419`. Holdout and listening remain closed. Batch 29.6CE is a
  no-render coefficient-path design checkpoint before more synthesis code.
- 2026-07-13: Batch 29.6CE contracts one complete successor coefficient path
  without rendering. Each selected adaptive frame uses centered reflection,
  Hann/Hann windows, its native FFT, unchanged magnitudes, and the exact dual.
  The fixed `4096` analytic spectrum remains decision-only: active trajectories
  carry physical frequency and phase, then map onto native bins while retaining
  native within-region phase offsets. Exact transient anchors coordinate phase
  reset and the proven conflicted-bridge replica owner on one output timeline.
  Batch 29.6CF owns implementation and the complete synthetic gate. Factor
  sweeps, real sources, holdout, listening, stereo, dynamic ratio, cache, and
  routing remain closed.
- 2026-07-13: Batch 29.6CF implements the contracted native-grid active-owner
  path and rejects it before real-source rendering. Identity, coverage,
  symmetry, residue, finiteness, exact anchors, event placement, dense
  one-to-one placement, replica protection, and all mid/high-tone rows pass.
  All `300/300` active resolution transitions retain a matched owner. Only the
  stretched `55 Hz` rows fail: rendered angular-frequency error reaches
  `3.695086e-5` against `1e-6`, while tracked-owner error stays at
  `1.263528e-7`. The `48`-row gate has three hard failures and zero combined
  regressions. Mechanism and quality hashes are `19c5548baf4a10c8` and
  `2410e33944214b72`. Batch 29.6CG initially owned native coefficient
  projection attribution; operator review stops that local repair path.
- 2026-07-13: Batch 29.6CG completes a pinned source study at Signalsmith
  Stretch revision `57b93f4e9206a089a45387eaa39bdc9f310d3308` (MIT) and
  Rubber Band revision `e4296ac80b1170018a110bc326fd0d45a0eb27d6`
  (GPL-2-or-later/commercial). No source expression transfers into Signal.
  Rubber Band R3 standard uses simultaneous long/middle/short transforms with
  exclusive frequency ownership, full-band H/P/R guidance, valley-adjusted
  crossovers, and coordinated reset/unlock/peak/channel phase state. It does
  not select one full-band resolution over time and does not synthesize
  additive H/P/R components. Signalsmith instead demonstrates one fixed-grid
  weighted multi-direction phase predictor. Rule 31 retires the Rule 30
  time-adaptive full-band successor and freezes those two shapes for one
  complete-system comparison. Batch 29.6CH owns that proof; local repair,
  parameter lattices, and per-metric follow-up chains remain closed.

## Next Task

Execute Batch 29.6CH under Rule 31 as one complete source-studied architecture
proof. Compare frequency-partitioned long/middle/short synthesis against the
fixed-grid weighted multi-predictor control. Do not reopen Rule 30AB or a
parameter-repair sequence.
