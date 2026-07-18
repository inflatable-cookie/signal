# 029 - Stretch Correctness And Listening Gate

Status: complete, rejected at stereo gate
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

- [x] add Signalsmith Stretch to the frozen synthetic and nine-row development
  comparator set
- [x] implement one report-only frequency-partitioned long/middle/short path
  with guidance-only classification and explicit phase states
- [x] retain one fixed-grid weighted multi-predictor control under the same
  schedule, boundary, and measurement contract
- [x] run the complete synthetic gate, all nine mono development rows, and one
  concealed listening pack only after hard integrity passes
- [x] decide on the whole architecture; do not open parameter lattices or
  per-metric repair batches

### Batch 29.6CI - Mono Decision Checkpoint

- [x] open only after the frozen nine-row development evidence is complete
- [x] decide whether the complete architecture earns continuation or returns to
  research as a whole
- [x] keep holdout, linked stereo, dynamic ratio, cache, production routing,
  and parameter search closed

Decision: reject the frequency-partitioned architecture. Continue only the
fixed-grid weighted predictor as a successor research direction. Its repeated
clean/tight/tidy results show a real improvement over hard nearest-owner phase
replacement, but do not yet establish production or comparator parity.

### Batch 29.6CJ - Exact-Excerpt Comparator Confirmation

- [x] export the exact `16384`-frame mono development inputs before external
  rendering; freeze sample rate, channel count, frame count, and content hash
- [x] render Rubber Band R3 and Signalsmith Stretch from those exact inputs;
  reject full-source renders, truncation substitutes, or channel-contract drift
- [x] export one concealed four-way pack containing weighted predictor, current
  Signal, Rubber Band R3, and Signalsmith Stretch without parameter changes
- [x] use the unchanged nine rows to decide the remaining external quality gap;
  keep frequency partitioning, holdout, stereo, dynamic ratio, cache, and
  production routing closed

Decision: the weighted predictor is credible but does not consistently beat
current Signal. It wins or ties two rows and remains competitive on two, but
repeats transient softness, smear, grain, and one end pop on four. Short stabs
close the exact-input transient/boundary check but cannot decide musical
continuity.

### Batch 29.6CK - Long-Form Musical Continuity Gate

- [x] freeze six five-second mono rows across drums, bass, vocals, pads, and
  full mix at `1.5x` or `2.0x`
- [x] compare only weighted predictor, current Signal, and Rubber Band R3 from
  exact shared inputs; do not reintroduce Signalsmith or frequency partitioning
- [x] export one concealed `24`-file pack after exact length, finiteness, input,
  external-render, assignment, gain, and notes hashes repeat
- [x] decide whether weighted prediction has a coherent musical advantage; if
  not, reject this implementation rather than opening per-row tuning

Decision: weighted prediction is a coherent improvement over current Signal on
four of six long-form rows, but this implementation is rejected for promotion.
It alters the early bass tone on M001 and collapses into MP3-like phase damage
on M004. Rubber Band remains best on four rows.

### Batch 29.6CL - Weighted-Predictor Fidelity Contract

- [x] compare the Signal proof against the pinned Signalsmith implementation at
  window/interval geometry, preliminary horizontal transport, time-factor-
  scaled vertical twists, energy normalization, weak-evidence fallback, and
  update ordering
- [x] freeze one Signal-owned complete predictor topology from those invariants;
  distinguish architecture correction from parameter choice
- [x] define synthetic bass-tone, chord/pad, transient, silence, boundary,
  determinism, and exact-length gates before another real-source render
- [x] keep random phase diffusion, parameter search, frequency partitioning,
  holdout, stereo, dynamic ratio, cache, and production routing closed

### Batch 29.6CM - Faithful Predictor Synthetic Proof

- [x] implement the fixed-output-grid, ratio-projected-input schedule with
  fourfold long-window geometry and exact overlap normalization
- [x] implement fixed-interval auxiliary horizontal transport followed by
  actual-hop-derived, time-factor-scaled short/long vertical correction from
  both directions
- [x] implement ascending dependency order, target-energy normalization,
  weak-evidence fallback, real endpoints, reflection, and exact crop
- [x] run the complete Rule 31G synthetic gate; stop before real-source audio
  on any failure

Decision: reject before real-source audio. The steady four-tone control creates
`-30.200611 dB` out-of-band energy against the frozen `-60 dB` ceiling. Every
other hard gate passes.

### Batch 29.6CN - Faithful Predictor Sideband Attribution

- [x] measure the steady four-tone spectrum after preliminary horizontal
  transport, each vertical direction/distance, complete ascending correction,
  normalization/fallback, and overlap synthesis
- [x] retain the frozen topology and parameters; add trace/report state only
- [x] identify the earliest stage that exceeds `-60 dB` and distinguish
  stationary phase error from frame-rate modulation
- [x] stop with one mechanism owner and one bounded next decision; keep corpus,
  holdout, listening, stereo, dynamic ratio, cache, and routing closed

Decision: horizontal transport owns the earliest failure. Its output measures
`-28.182097 dB`; the strongest spur is `33.339844 Hz` from the nearest tone,
within `0.006510 Hz` of the `33.333333 Hz` frame rate. Exact analysis/synthesis
overlap measures `-80.392196 dB`, normalization phase delta is at most
`4.441e-16`, and significant fallback count is zero.

### Batch 29.6CO - Horizontal Mixture-Contamination Attribution

- [x] render the four frozen chord tones separately and together through the
  same horizontal-only trace; change no predictor or geometry value
- [x] measure nearest-bin auxiliary-ratio phase-advance variance and frame-rate
  sidebands for isolated and mixed controls
- [x] decide whether the horizontal failure begins with within-bin component
  interference or with phase convention / synthesis attachment
- [x] stop with one architecture choice: observation-geometry redesign or
  predictor-equation correction; keep real audio and parameter sweeps closed

Decision: predictor-equation correction. Every isolated tone fails at
`-23.544808` to `-51.499468 dB` with a strongest spur one output frame rate
from the tone. Isolated auxiliary-ratio variance stays at `5.789e-11` to
`1.710e-7`; mixing raises it but is not required. Pinned-source reinspection
finds Signal normalized preliminary horizontal output directly to current
energy, while the specimen divides by the maximum of previous and current input
energy before separate vertical target-energy normalization.

### Batch 29.6CP - Preliminary Horizontal Energy-Law Correction

- [x] replace only preliminary horizontal target normalization with the
  previous/current input-energy denominator; retain every other frozen choice
- [x] preserve separate vertical target-energy normalization, fallback,
  dependency order, geometry, scheduling, windows, and overlap synthesis
- [x] rerun the complete Rule 31G synthetic gate plus CN/CO attribution hashes;
  stop before real sources on any failure
- [x] decide the corrected equation as one complete topology; do not sweep
  floors, weights, windows, intervals, distances, or FFT size

Decision: retain the source-faithful energy law, reject it as the sideband
cure. Complete leakage moves only from `-30.200611` to `-30.236852 dB` and the
horizontal trace to `-29.975234 dB`; every isolated tone still fails. The
trace carries the prior vertically corrected output state, so it does not yet
separate direct horizontal recurrence from vertical-state feedback.

### Batch 29.6CQ - Predictor State-Lineage Attribution

- [x] add report-only horizontal recurrence with prior horizontal state beside
  the existing prior corrected state; change no production candidate equation
- [x] compare isolated and mixed phase advance, frame-rate sidebands, and
  repeated hashes under both state lineages
- [x] identify whether direct horizontal transport or vertically corrected
  state fed into the next frame first creates the modulation
- [x] stop with one mechanism owner; keep parameter changes, real sources,
  listening, stereo, dynamic ratio, cache, and routing closed

Decision: vertical-state feedback is not necessary. A target-magnitude phase
oracle driven only by prior horizontal state improves every isolated tone by
`1.228` to `24.949 dB` and mixed output by `11.583 dB`, but all isolated tones
still fail at `-41.444546` to `-52.739473 dB`. Each retains a strongest
sideband within `0.222 Hz` of the `33.333333 Hz` frame rate. Independent-bin
horizontal phase transport therefore carries the earliest modulation, but it
is an intentionally incomplete intermediate field. Do not change its equation
until the pinned upstream engine is measured under the same final-output gate.

### Batch 29.6CR - Pinned-Source Synthetic Comparator

- [x] run pinned Signalsmith Stretch revision `57b93f4e` on the frozen isolated
  tones and chord at `8 kHz`, ratio `2`, and its matching default geometry
- [x] measure final-output out-of-band energy, strongest sideband offset, pitch,
  exact output length, finiteness, and repeated evidence under Rule 31G methods
- [x] decide whether the `-60 dB` gate is attainable by the studied complete
  topology or whether Signal still diverges from the source implementation
- [x] if upstream passes, stop at the first required internal differential;
  if upstream fails, stop and revise the architecture target before more code
- [x] keep the comparator report-only and out of production dependencies; keep
  tuning, corpus, listening, stereo, dynamic ratio, cache, and routing closed

Decision: the absolute `-60 dB` ceiling is not attainable by this studied
topology at `2x`. Pinned Signalsmith Stretch `1.3.2` produces isolated-tone
leakage from `-44.686281` to `-46.016214 dB` and chord leakage
`-40.016259 dB`, with the same frame-rate sidebands. Signal also diverges from
the source: three isolated tones are `8.041` to `21.143 dB` worse and the chord
is `9.779 dB` worse; one isolated tone is `6.225 dB` better. Replace the invalid
absolute fidelity gate with paired source parity before another algorithm edit.

### Batch 29.6CS - Source-Relative Fidelity Gate

- [x] retain `-60 dB` as an absolute diagnostic, not a rejection criterion for
  faithful implementation of this topology at `2x`
- [x] freeze paired parity on the exact quantized controls: Signal output must
  be no more than `1 dB` worse than pinned source for every tone and the chord
- [x] preserve exact length, finiteness, repeat, pitch, transient, silence,
  boundary, fallback, and mechanism gates unchanged
- [x] update Rule 31G, the report direction, and research front doors without
  changing predictor code or rendering real sources
- [x] identify one source-versus-Signal internal differential for the failed
  three tones and chord; keep tuning and parameter sweeps closed

Decision: source-relative parity is now the fidelity rejection gate. Pinned
source records four tone and one chord failures against the retained absolute
`-60 dB` diagnostic. Signal records three tone and one chord failures against
the paired `1 dB` allowance, so real-source work remains closed. Exact source
inspection identifies frequency-boundary lookup as the first bounded
differential: pinned source zero-extends out-of-range fractional bins while
Signal clamps them to an edge. Ten vertical observations differ per `2x`
frame. This is the next causal ablation, not a proven owner.

### Batch 29.6CT - Frequency-Boundary Attribution

- [x] add one report-only zero-extension variant beside the frozen clamped
  translation; change no other predictor law or production path
- [x] compare both Signal variants against pinned source on the exact quantized
  tones and chord using the frozen `1 dB` paired gate
- [x] preserve exact length, finiteness, pitch, repeat, and absolute diagnostic
  reporting; verify the ten affected observations per `2x` frame
- [x] decide whether boundary policy materially closes the three-tone and chord
  parity failures; stop if it does not
- [x] keep weights, windows, geometry, distances, floors, corpus, listening,
  stereo, dynamic ratio, cache, and routing closed

Decision: reject frequency-boundary policy as the parity-gap owner. Zero-
extension changes isolated-tone leakage by only `-0.033206` to `+0.005683 dB`
and chord leakage by `-0.068380 dB` relative to clamping. Both variants retain
`[3 tone, 1 chord]` paired failures. All structural, pitch, and repeat checks
pass. Keep the clamped translation frozen; do not compound the rejected change.

### Batch 29.6CU - Stage-Aligned Source Trace

- [x] define a pinned, report-only trace boundary for current input spectrum,
  preliminary horizontal state, and corrected output state
- [x] align one steady interior frame for exact quantized `110 Hz`, `220 Hz`,
  and chord controls at `8 kHz`, ratio `2`, geometry `960/240`
- [x] compare normalized target-bin magnitude and relative phase; preserve raw
  hashes and stop before downstream sideband/dependency claims once the source
  and Signal analysis bases prove non-isomorphic
- [x] identify the first material source-versus-Signal state divergence and
  select exactly one following causal ablation
- [x] change no predictor law; keep window/FFT changes, corpus, listening,
  stereo, dynamic ratio, cache, and routing closed

Decision: the first divergence precedes predictor transport. Pinned Signalsmith
uses a `1024`-point modified real transform over the same `960`-frame support:
`512` half-bin bands start at `3.90625 Hz` with `7.8125 Hz` spacing. Signal uses
a `960`-point standard real transform: `481` bins start at DC with
`8.333333 Hz` spacing. Exact aligned hashes repeat at source centre `8400`.
Target-bin magnitude differences are `0.0222` to `0.1452`; relative phase
differences are `1.7002` to `2.8156 rad`. Those downstream values compare
different bases and do not select another predictor-law edit.

### Batch 29.6CV - Modified Analysis-Grid Attribution

- [x] add one report-only Signal variant with `960`-frame support, `240`-frame
  interval, a `1024`-point transform, and the pinned half-bin frequency grid
- [x] hold the Signal window, scheduling, predictor equations, normalization,
  fallback, boundary policy, and synthesis ownership fixed
- [x] prove analysis/synthesis identity, exact length, finiteness, pitch, and
  repeated hashes before reading fidelity movement
- [x] rerun exact-input source parity and decide whether the grid materially
  reduces `[3 tone, 1 chord]` paired failures
- [x] stop on rejection; do not combine the grid with a window change or reopen
  corpus, listening, stereo, dynamic ratio, cache, or routing

Decision: reject the modified half-bin grid as a standalone parity mechanism.
Analysis/synthesis identity error is `2.220e-16`; all structural, pitch, and
repeat gates pass. The grid improves only `110 Hz` by `6.071 dB` versus Signal
baseline. It regresses the other tones by `3.171` to `28.993 dB` and the chord
by `3.736 dB`. Paired failures worsen from `[3 tone, 1 chord]` to
`[4 tone, 1 chord]`. Do not promote or compound the variant.

### Batch 29.6CW - Source Kaiser Window Attribution

- [x] pin the exact `960/240` periodic Kaiser window selected by Signalsmith
  Linear revision `56686735`; freeze coefficient and overlap-product hashes
- [x] add one report-only standard-`960`-grid Signal variant using that window
  for analysis and synthesis with exact overlap normalization
- [x] hold transform grid, scheduling, predictor equations, distances,
  normalization, fallback, boundary policy, and synthesis ownership fixed
- [x] prove identity, exact length, coverage, finiteness, pitch, and repeat
  before rerunning exact-input source parity
- [x] decide the window alone; do not combine it with the rejected half-bin
  grid or reopen corpus, listening, stereo, dynamic ratio, cache, or routing

Decision: reject the source window alone and correct the prior symmetry claim.
The pinned even-length Kaiser is periodic: analysis and synthesis hashes are
both `cd811c4f82d161be`, maximum mirror delta is `0.002532`, and the exact
four-hop overlap-product hash is `6dadf0c986c4bd49` with `8.953e-8` maximum
unity error. Identity error is `2.776e-16`; structure, pitch, and repeat pass.
The window improves `110 Hz` and `220 Hz` by `10.078` and `8.823 dB`, but
regresses `164.8138 Hz`, `329.6276 Hz`, and the chord by `5.906`, `30.764`, and
`1.821 dB`. Paired failures worsen from `[3 tone, 1 chord]` to `[4 tone,
1 chord]`. Do not promote the window alone.

### Batch 29.6CX - Pinned Analysis-Representation Interaction

- [x] complete the source-derived `2x2` analysis representation comparison by
  combining only the pinned periodic Kaiser window and modified half-bin grid
- [x] retain `960/240` support, scheduling, predictor equations, distances,
  normalization, fallback, boundary policy, and synthesis ownership
- [x] prove identity, exact length, coverage, finiteness, pitch, and repeat
  before reading fidelity
- [x] report the per-control grid/window interaction against baseline,
  grid-only, window-only, and pinned-source evidence
- [x] decide whether the exact observed combination coherently reduces parity
  failures; stop before any third mechanism or real-source render

Decision: retain the combined source representation. Neither main effect is
valid alone: grid-only and window-only each worsen paired failures from `[3,
1]` to `[4, 1]`. Together they close the frozen source-relative gate at `[0,
0]`. Combined tones land from `-0.141` to `+0.147 dB` relative to pinned source;
the chord lands at `-0.641 dB`. Identity error is `2.220e-16`; length, coverage,
finiteness, boundaries, pitch, and repeated hashes pass. The interaction is
strong and non-additive, from `-3.455` to `-53.403 dB` across the controls.
Treat periodic Kaiser plus modified half-bin grid as one coherent analysis
representation. Do not promote either component independently.

### Batch 29.6CY - Coherent Representation Synthetic Gate

- [x] make the combined periodic-Kaiser/modified-half-bin representation the
  report-only faithful-predictor research baseline
- [x] rerun the complete bass, chord, transient, silence, cancellation,
  mechanism-exercise, boundary, coverage, duration, identity, and repeat proof
- [x] retain source-relative tone/chord parity and freeze the coherent output
  hashes without changing predictor equations or adding a third mechanism
- [x] decide whether the complete synthetic gate opens exact-input real-source
  confirmation; keep product routing, stereo, dynamic ratio, and promotion
  closed

Decision: pass the complete synthetic gate and open exact-input real-source
confirmation. The coherent representation retains `[0 tone, 0 chord]` source-
relative failures. Structure, identity, coverage, finiteness, boundaries, and
repeat pass. Maximum bass error is `0.000718 Hz`; chord peak error is
`0.007314 Hz`; transient placement error is one frame with zero replicas;
silence is exact. Every horizontal, short, long, corrected, and fallback path
is exercised. Freeze complete-proof hash `0905a7fd4180bff4`. Product routing,
stereo, dynamic ratio, and promotion remain closed.

### Batch 29.6CZ - Exact-Input Real-Source Confirmation

- [x] generalize the coherent source-derived analysis geometry to the frozen
  long-form sample rate without changing its `30 ms` interval, fourfold
  support, periodic Kaiser construction, or modified half-bin basis
- [x] rerender the six existing five-second Batch 29.6CK musical rows through
  coherent Signal and pinned Signalsmith from identical mono inputs
- [x] report exact length, finiteness, boundaries, peak growth, timing,
  transient replicas, spectral residual, and deterministic hashes before
  opening listening
- [x] decide whether objective confirmation authorizes one concealed musical
  comparison; keep stereo, dynamic ratio, product routing, and promotion closed

Decision: open one concealed musical comparison. Source-derived geometry at
`44.1 kHz` is `[5292 support, 1323 interval, 6144 transform, 3072 bands]`.
Both engines pass exact length, finiteness, and hard integrity on all six rows;
coherent Signal repeats exactly. Against pinned Signalsmith, Signal has lower
timing error on four rows, lower replica ratio on three, and lower static
spectral residual on four. Boundary-growth is worse on all six rows. That
metric is amplified by near-zero exterior source steps, but it is not waived:
the concealed comparison must judge starts, ends, and transient-edge artifacts
explicitly. Freeze hashes `8ede75dbae2254b2` (inputs),
`7ec654eb414041ce` (Signal), `ee39390a1e17d923` (Signalsmith), and
`7a6b1e7dd7ba5c13` (report). The pinned comparator uses seed `0`; its default
CLI seeds from `std::random_device` and is not reproducible at the `2x` phase-
randomization boundary.

### Batch 29.6DA - Concealed Coherent Source Comparison

- [x] export the six exact Batch 29.6CZ inputs as references plus concealed
  coherent-Signal and pinned-Signalsmith candidates
- [x] randomize candidate identity deterministically and freeze manifest,
  audio, mapping, duration, channel, and sample-rate hashes before listening
- [x] collect row-complete judgments for musical continuity, transient
  definition, grain/ringing, tonal stability, and start/end artifacts
- [x] decide whether coherent Signal remains the source-studied baseline;
  keep stereo, dynamic ratio, product routing, and promotion closed

Pack state: corrected and ready for the remaining concealed operator listening at
`target/stretch-source-studied-da-concealed-pack`. Six source references and
twelve level-matched trials pass exact frame, finite-value, `44.1 kHz` mono,
file-count, objective-direction, repeat, and packed-candidate RMS-equality
gates. Freeze audio hash `760577241605fb24`, assignment hash
`64c2874dd6e47521`, gain hash `7bba88c9c701bf1c`, manifest hash
`fd1255a2fc007590`, closed-key hash `bb1974bba5a2a8b0`, notes hash
`91d68633349f1944`, and metadata-receipt hash `de417d1f00e55f88`. Maximum pair
RMS delta is `2.44e-9 dB`.

The first export selected the minimum raw RMS, then applied the `0.95` peak
ceiling independently. That left `M002` mismatched by about `4.14 dB` and
`M006` by about `0.49 dB`. Its `M002` and `M006` judgments are invalid.
`M001`, `M003`, `M004`, and `M005` changed by at most `0.05 dB`; preserve their
completed findings.

The corrected six-row record is complete and the key is open. `M001`, `M002`,
`M004`, `M005`, and `M006` are audible ties. On `M003`, coherent Signal is
slightly less grainy. Coherent Signal therefore closes at one slight preference,
five ties, and zero losses against pinned Signalsmith. Retain it as the
report-only source-studied baseline. This is not a production selection or a
Rubber Band-class claim.

### Batch 29.6DB - Exact-Source Rubber Band Benchmark

- [x] reuse the six exact Batch 29.6CZ source references, ratios, duration, and
  corrected peak-safe RMS matcher
- [x] render coherent Signal and Rubber Band R3 `4.0.0` from identical mono
  inputs without parameter or mechanism changes
- [x] freeze hard-integrity, objective, receipt, assignment, and concealed-pack
  hashes before listening
- [x] complete the six-row concealed comparison before opening stereo, dynamic
  ratio, product routing, or promotion

Pack state: ready for concealed operator listening at
`target/stretch-source-studied-db-concealed-pack`. Both engines consume the
same six exact `44.1 kHz` mono 16-bit inputs and repeat exactly. Structural
failures are `[0; 9]`; hard-integrity failures are `0/6` for both engines;
maximum packed-candidate RMS delta is `1.31e-9 dB`. Coherent Signal is worse
than Rubber Band on `2/6` timing rows, `5/6` replica rows, `0/6` static-residual
rows, and `6/6` boundary-growth rows. The mixed direction opens listening and
does not select either engine.

Freeze hashes: input `8ede75dbae2254b2`, coherent audio
`7ec654eb414041ce`, Rubber Band audio `3ee61b19c9498523`, measurements
`1c4b6398bf49d9bf`, objective report `eb1144f437a6ae65`, render receipt
`4338e41ab85fe116`, packed audio `bd7dec22a565a32f`, assignment
`c9724071b3aa2ded`, gain `d2b29e930726e10f`, manifest
`fd1255a2fc007590`, closed key `14d5bbab2061b8fd`, notes
`91d68633349f1944`, and audio receipt `1f80e9da6c011beb`.

Decision: retain coherent Signal unchanged as the report-only mono baseline
and open Batch 29.7 objective linked-stereo work. The completed key maps Signal
to `B/A/B/A/B/A` across `M001` through `M006`. Signal is cleaner on `M002` and
`M004`, slightly cleaner on `M005`, and tighter but marginally grainier on
`M001`. Rubber Band is cleaner on `M003` and `M006`. Defects change sides with
material, leaving no overall winner. This is competitive exact-source mono
evidence, not a general Rubber Band-parity or production claim.

### Batch 29.7A - Shared-Decision Stereo Contract

- [x] replace inherited heap/owner language with the coherent predictor's real
  shared seam: frame schedule, traversal, and aggregate correction mode
- [x] keep spectra, recurrence, magnitudes, synthesis, and normalization
  per-channel; forbid mid/side resynthesis, dominant-channel phase replacement,
  cross-channel mixing, and independent schedules
- [x] freeze mechanics, image, interchannel-phase, delay, transient, repeat,
  and stop gates before implementation

Decision: implement one two-channel report-only renderer under Rule 31H. The
aggregate target/prediction energy comparison selects corrected or fallback
mode for both channels. Channel-local numerical completion remains observable
and must be zero for non-silent controls. Mono hashes, dynamic ratio, routing,
realtime use, and product selection remain closed.

### Batch 29.7B - Linked-Stereo Mechanics Proof

- [x] add one linked frame loop with shared geometry plus per-channel predictor
  state without changing mono output
- [x] prove duplicated-mono and hard-pan mono parity, exact silent-channel
  behavior, swap/polarity equivariance, scaled duplicate parity, coverage,
  boundaries, repeat, and shared-mode exercise at `0.75x`, `1.5x`, and `2.0x`
- [x] stop before quality controls on any mono hash change, non-silent
  unilateral completion, crossfeed, or structural failure

Evidence: one repeat-stable review with per-ratio structure, mono parity,
transformation parity, shared corrected/fallback counts, unilateral completion,
crossfeed, audio hashes, and aggregate evidence hash. Focused test name:
`source_studied_linked_stereo_mechanics`.

Validation:

- `cargo fmt --check -p signal-dsp-stretch`
- `cargo test -p signal-dsp-stretch --release source_studied_linked_stereo_mechanics`
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- `effigy qa:docs`
- broader `cargo test -p signal-dsp-stretch` before commit

Decision: mechanics pass and Batch 29.7C may open. All structural, mono-parity,
hard-pan, swap, polarity, scaled-duplicate, crossfeed, unilateral-completion,
and repeat failures are zero. Both shared modes execute. The initial draft's
gain-equivariance requirement was invalid because the frozen mono predictor
uses an absolute horizontal floor; it was corrected to linked-versus-mono
parity at gains `0.25` and `4`, which passes bit-exactly. Freeze row audio
hashes `38dad9d73677280f`, `a48d55bf5f1120ae`, and `d90c4971bd452d50`,
aggregate audio hash `f34476f290ce4f80`, and evidence hash
`426af565378e9ce1`.

### Batch 29.7C - Linked-Stereo Quality Gate

- [x] measure constant interchannel phase, broadband delay, mid/side RMS ratio,
  correlation, and one-sided isolated/dense transient behavior
- [x] apply Rule 31H thresholds without tuning or adding a new phase owner
- [x] freeze audio, mechanism, and measurement hashes; decide whether Batch
  29.8 stereo export may open

Evidence: one repeat-stable quality review with per-control IPD, delay,
mid/side-ratio, correlation, event, replica, and crossfeed measurements plus
audio and report hashes. Focused test name:
`source_studied_linked_stereo_quality`.

Decision: fail quality; Batch 29.8 stays closed. Same/opposite-phase tones,
decorrelated image, attacks, replicas, crossfeed, mechanics, and repeat pass.
Quadrature IPD, expansion delay, and unequal-correlated image fail by large
margins. Independent coherent mono paths reproduce every linked failure with
the same per-ratio masks `13`, `15`, and `15`, assigning the primary fault to
per-channel recurrence rather than aggregate mode selection. Freeze quality
row audio hashes `ddc816d477db135d`, `6842967ca6c7984b`, and
`9d38e21d580f84ed`; aggregate audio `0509599cb46b0cfc`; quality measurement
`2d8f8471d88cf383`; attribution evidence `d148ae6a7114ef6a`.

### Batch 29.7D - Cross-Channel Recurrence Reassessment

- [x] inspect how source-studied engines and canonical phase-vocoder research
  preserve interchannel phase without collapsing decorrelated material
- [x] compare shared phase-increment and explicit complex-ratio preservation
  topologies against the frozen 29.7C evidence; do not tune thresholds
- [x] promote one license-safe topology into architecture and Rule 31H, or
  pause stereo if no topology is justified

Decision: select reference-relative recurrence. Per frame/bin, the greatest
current target energy owns the coherent recurrence. The peer keeps
its own magnitude and takes the reference output plus its wrapped current input
phase relation to that reference. Signalsmith's MIT implementation and the 2005
AES multichannel TSM paper directly support this law. Rubber Band R3 provides
architecture-only corroboration; its GPL expression and constants remain
excluded. Shared output increment is rejected because it preserves a prior
output relationship rather than explicitly restoring the current input
relationship. Translation memo 006 and revised Rule 31H freeze the result.

### Batch 29.7E - Reference-Relative Recurrence Proof

- [x] replace only the linked renderer's per-channel recurrence with the Rule
  31H per-bin reference recurrence; preserve mono code and hashes, geometry,
  schedule, energy floor, thresholds, and report-only routing
- [x] extend mechanics with reference counts for both channels, exact-energy
  tie exercise, controlled ownership crossing, switch-boundary growth,
  crossfeed, repeat, and frozen hashes
- [x] rerun every unchanged 29.7C IPD, delay, correlated/decorrelated image,
  transient, replica, and crossfeed control at `0.75x`, `1.5x`, and `2.0x`
- [x] stop before listening on any mechanics or quality failure; do not add
  hysteresis or tune a switch threshold inside this batch

Evidence: one repeat-stable mechanics and quality report. Focused test names:
`source_studied_linked_stereo_reference_recurrence` and
`source_studied_linked_stereo_quality`.

Validation:

- `cargo fmt --check -p signal-dsp-stretch`
- `cargo test -p signal-dsp-stretch --release source_studied_linked_stereo_reference_recurrence`
- `cargo test -p signal-dsp-stretch --release source_studied_linked_stereo_quality`
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- `effigy qa:docs`
- broader `cargo test -p signal-dsp-stretch` before commit

Decision: mechanics pass and the quality gate still fails. Both channels own
non-zero bins; exact ties choose channel zero; the crossing control records
`512`, `512`, and `860` owner switches with boundary-step growth `-1.080220`,
`-0.583075`, and `-0.413230 dB`. Mono, duplicate, hard-pan, swap, polarity,
gain, silence, coverage, boundary, crossfeed, and repeat controls pass.

Reference-relative recurrence removes every delay failure and sharply reduces
the prior image damage. Worst quadrature IPD falls from `0.882661`, `1.526285`,
and `3.073635 rad` to `0.008195`, `0.007623`, and `0.016074 rad`. Correlated
mid/side change falls from `11.672982`, `11.990144`, and `12.728013 dB` to
`0.434087`, `0.267458`, and `0.173960 dB`. The unchanged `1e-9 rad` IPD gate
still fails at every ratio; the `0.25 dB` image gate also fails at `0.75x` and
`1.5x`. Batch 29.8 remains closed.

Freeze mechanics audio `28803a9f2e5bd83e`, mechanics evidence
`03b66c25196493c2`, quality audio `a5fb675cb0484eda`, quality measurement
`ae77c422ea75e292`, and residual attribution `ebfb64802f96d50b`.

### Batch 29.7F - Reference Projection Residual Attribution

- [x] measure peer/reference complex-ratio error immediately after projection
  and after real-edge constraint; prove whether coefficient-domain relation is
  already exact before inverse transform
- [x] compare full-render and interior-only tone/image measurements to separate
  boundary reflection from steady overlap behavior
- [x] add a synthetic known-constant-relation oracle alongside current per-bin
  input-ratio projection; use it only to assign residual ownership
- [x] choose the next topology seam from the first measured divergence; do not
  relax thresholds, add peak regions, or open listening inside attribution

Evidence: one repeat-stable stage report with coefficient relation, constrained
relation, whole/interior output, oracle, and current-projection hashes. Focused
test name: `source_studied_linked_stereo_projection_residual_attribution`.

Decision: coefficient projection and real-edge constraint both preserve the
measured relation to `4.440892e-16 rad`. Whole/interior quadrature IPD is
`0.008195/0.001652`, `0.007623/0.000628`, and `0.016074/0.000071 rad`.
Interior correlated mid/side damage remains `0.397904`, `0.264677`, and
`0.178868 dB`, so boundary removal does not close image quality. A fixed
`pi/2` relation oracle is better on some rows and worse on others. The first
unexcluded seam is inverse synthesis, overlap accumulation, normalization, or
measurement. Evidence hash: `87a057697db91edd`.

### Batch 29.7G - Stereo Synthesis Closure Attribution

- [x] calibrate the whole/interior IPD estimator on ideal target-length tones
  with the same output crops; separate measurement floor from render damage
- [x] trace the current and constant-relation oracle after inverse synthesis,
  after overlap accumulation, and after normalization without changing output
- [x] retain the frozen 29.7E audio and 29.7F evidence hashes; do not alter
  recurrence, windows, geometry, thresholds, or crop
- [x] assign the first post-spectrum divergence to inverse synthesis, overlap,
  normalization, or measurement and stop before topology work

Evidence: one repeat-stable stage report and one focused
`source_studied_linked_stereo_synthesis_closure_attribution` test. A measured
owner opens one bounded repair contract; an estimator floor revises the proof
measurement before any DSP change. Batch 29.8 remains closed.

Decision: ideal whole records measure within `1.110223e-13 rad`; cropped ideal
records expose a `0.000142` to `0.000489 rad` estimator floor. Calibrated
current/oracle support-frame interior error is already `0.000604` to
`0.010644 rad`. Overlap accumulation often reduces it. Normalization changes
whole/interior IPD by less than `1e-9 rad` and is excluded. Frozen 29.7F audio
hashes repeat exactly. Evidence hash: `7f8cee549977896d`. The first observable
post-spectrum seam is real support-frame synthesis before overlap.

### Batch 29.7H - Analytic Overlap Feasibility Proof

- [x] add one report-only analytic positive-frequency overlap ablation beside
  the unchanged real support-frame path; feed both the same corrected spectra
- [x] retain recurrence, magnitudes, schedule, window, crop, normalization,
  thresholds, current output, and all frozen current hashes
- [x] measure current and constant-relation tone IPD plus correlated image,
  duplicate-mono parity, hard pan, swap, polarity, coverage, finiteness,
  boundaries, crossfeed, and repeat
- [x] reject unless analytic overlap materially improves every failing IPD/image
  row without mechanics damage; stop before listening or production adoption

Evidence: one repeat-stable analytic/current ablation report and focused
`source_studied_linked_stereo_analytic_overlap_feasibility` test. This tests the
measured synthesis seam, not another phase coefficient, threshold, or window.
Batch 29.8 remains closed.

Decision: reject. Analytic and real overlap produce exactly equal IPD at all
three ratios and image deltas equal within `2e-15`. The oracle is also equal
within `1e-14`. Analytic reconstruction changes samples only by
`2.220446e-16` to `3.330669e-16`, creating `9164`, `18212`, and `24148`
duplicate-mono bit mismatches without quality gain. Evidence hash:
`db73736856099b7d`. Real support synthesis is the observation point, not a
causal representation defect.

### Batch 29.7I - Complete Coefficient Contribution Attribution

- [x] classify every frame/bin contribution as initial-frame, viable corrected,
  reference fallback, significant, or weak; remove the 29.7F trace blind spot
- [x] measure relation error and synthesized energy for every class, including
  bins excluded by the existing significant-energy report threshold
- [x] run one-class-at-a-time constant-relation ablations for initial, fallback,
  and weak coefficients while preserving current output and frozen hashes
- [x] select the first class whose ablation closes whole-render IPD/image; stop
  before threshold tuning, recurrence changes, or listening

Evidence: one contribution-closed, repeat-stable report with class counts,
energies, relation errors, ablation measurements, and hashes. Focused test:
`source_studied_linked_stereo_coefficient_contribution_attribution`. A closing
class opens one bounded repair. Failure to close forces a gate-definition
reassessment instead of another topology experiment. Batch 29.8 remains
closed.

Decision: no class closes. All initial, corrected, fallback, significant, and
weak relations measure within `4.440892e-16 rad`. Fallback occurs only
`1/2/1` times with at most `2.597671e-5` synthesized energy. Weak coefficients
carry only `0.00032%` to `0.00053%` of total energy. Their oracle worsens tone
IPD at every ratio; fallback is neutral; initial forcing improves `0.75x` by
only `1.36e-5 rad` and regresses both expansions. Correlated-image movement is
unchanged within `2.4e-14 dB`. Current hashes remain frozen. Evidence hash:
`49bfd7c9c3bf7d21`. The exact gate now requires calibration; no coefficient or
synthesis repair is authorized.

### Batch 29.7J - Stereo Invariant Gate Calibration

- [x] freeze ideal target-length, current Signal, pinned source-studied, and
  Rubber Band reference renders for the same tone and correlated-image controls
- [x] measure whole/interior IPD and image across record length, starting phase,
  bin alignment, and boundary crop without changing any renderer
- [x] add one sample-domain relationship residual that does not assume a
  constant positive-frequency coefficient ratio for finite real windows
- [x] retain the exact gate if ideal and external references meet it; revise it
  only if calibrated evidence proves it rejects accepted stereo behavior
- [x] if calibrated Signal remains materially worse, open one measured repair
  direction; if competitive, reopen Batch 29.8 independent stereo review

Evidence: one repeat-stable calibration matrix with exact input/output hashes,
estimator floors, external-reference provenance, and an explicit gate decision.
No DSP, threshold, topology, listening, dynamic-ratio, cache, realtime, or
routing change belongs in this batch. Batch 29.8 remains closed until the
decision is frozen.

Decision: revise the gate and repair Signal. The repeat-stable 192-row matrix
covers two record lengths, two starting phases, aligned/off-bin tones, three
ratios, two controls, and four renderers. The finite 16-bit ideal floor reaches
`1.073e-6 rad`, proving `1e-9 rad` is not a valid external-reference gate. A
calibrated `0.006 rad` tone gate admits ideal and Rubber Band; Signal reaches
`0.01475 rad`. On correlated image, Rubber Band stays within `0.02863 dB` and
`0.001463` normalized Gram residual while Signal reaches `0.54712 dB` and
`0.01181`. Signalsmith reaches the same image drift, so source imitation does
not close it. Collapsed and crossfed negative controls measure `0.24558` and
`0.09651`, proving residual sensitivity. All renders are structurally valid,
hash-complete, and repeat. The production gate remains unchanged in this
calibration batch; Batch 29.7K owns one bounded relation-preservation repair.

### Batch 29.7K - Calibrated Stereo Relation Repair

- [x] freeze the 29.7J calibrated tone-IPD and correlated-image residual gates
  as report-only acceptance criteria; do not weaken them to admit Signal
- [x] localize Signal's image drift across output windows and determine whether
  one stable real `2x2` relation transform can close it without time-varying
  pumping
- [x] compare the proposed repair against ideal and Rubber Band behavior, with
  duplicate, hard-pan, swap, polarity, mono-parity, and negative controls
- [x] implement at most one report-only repair law if the localization supports
  it; reject the direction if it cannot preserve the mechanics invariants
- [x] reopen Batch 29.8 only when calibrated tone and image gates pass

Evidence: one repeat-stable localization and differential report, followed by
at most one bounded repair. No threshold search, listening export, dynamic
ratio, cache, realtime, product routing, or promotion belongs in this batch.

Decision: reject render-wide relation coloring and keep Batch 29.8 closed. One
gain-neutral real `2x2` normalized-Gram transform closes whole-render covariance
and preserves total energy. Rank-deficient duplicate and hard-pan controls
bypass exactly. Duplicate, hard-pan, swap, polarity, scaled-mono parity, and
silent-peer mechanics all pass at zero error. Ideal and Rubber Band have zero
calibrated failures. Repaired Signal still fails `14/48` rows and local
consistency on `17/48`; tone IPD reaches `0.01621 rad` and interior image
reaches `0.06843 dB`. A thresholded variant was rejected during proof because
it made scale-equivalent material take different branches. No post-render or
windowed matrix sweep is authorized.

### Batch 29.7L - Rubber Band Linked-Stereo Mechanism Study

- [x] pin the Rubber Band R3 `4.0.0` source corresponding to the installed
  comparator; record exact revision, build identity, license, and excluded GPL
  expression boundary
- [x] trace channel coupling through peak ownership, phase lamination,
  transient reset, and synthesis; distinguish verified source behavior from
  inference
- [x] run bounded stereo behavioral differentials for available public modes
  against the frozen tone/image controls and calibrated gates
- [x] compare the verified mechanism with Signal's reference-relative
  recurrence and identify the first architectural difference that explains the
  measured residual
- [x] promote at most one license-safe invariant into architecture and Rule 31H,
  or pause linked stereo if no bounded repair is justified

Evidence: one provenance-complete source translation and behavioral report.
No GPL expression or constants enter Signal. No DSP implementation, threshold
search, listening, dynamic ratio, cache, realtime, routing, or promotion belongs
in this batch.

Decision: promote conditional, frequency-bounded peak-region trajectory
ownership. Exact official `4.0.0` source matches Git tag `v4.0.0` at
`1d95888`; the GPL expression boundary stays closed. Standard R3 couples
channels through compatible tracked peak history inside a bounded range, not
unconditional same-bin projection. Centre-focus adds mid/side and stronger
linking, changes all `48` public-mode pairs, and fails four calibrated `2.0x`
image rows while standard R3 fails none. Mid/side and blanket linking are
rejected. Translation memo 007 freezes the result.

### Batch 29.7M - Peak-Region Shared-Trajectory Feasibility

- [x] define one Signal-owned peak identity using the frozen coherent
  representation; do not translate Rubber Band's picker or constants
- [x] freeze compatible-history eligibility and frequency ownership before
  rendering; no threshold or cutoff sweep
- [x] add one report-only candidate that shares a trajectory only inside an
  eligible peak region, preserves peer local analysis-relative phase and
  magnitude, and retains channel-owned evolution elsewhere
- [x] preserve current output as baseline plus mono hashes, mechanics,
  geometry, schedule, crop, calibrated gates, and exact repeat
- [x] accept only row-complete improvement over current Signal with no new
  structural, transformation, transient, crossfeed, or local-consistency loss

No listening, dynamic ratio, cache, realtime, routing, promotion, centre-focus
clone, mid/side transform, or production DSP change belongs in this batch.

Decision: reject the peak-region candidate. Signal-owned local maxima and
nearest-peak regions, with exact shared previous-peak ownership, are active and
repeat exactly. Mechanics remain exact, but failures rise from `20/48` to
`29/48`; only `13/48` rows improve completely, `35/48` regress on at least one
metric, and `32/48` fail local consistency. Evidence
`31a8b2eaae086fc8` closes tuning. Peak geometry and compatible history alone
do not supply the material-state policy used by the reference architecture.

### Batch 29.7N - Linked-State Policy Triangulation

- [x] attribute the frozen 29.7M losses by ratio, control, and trajectory state;
  distinguish shared-region damage from independent recurrence
- [x] triangulate ordinary, peak-locked, reset, unlocked, and attack ownership
  from architecture-only Rubber Band evidence, permissive implementations, and
  published work; label verified behavior and inference separately
- [x] define one Signal-owned state vocabulary and ordering from independent
  evidence, or pause linked stereo if no bounded law is justified
- [x] authorize at most one later report-only state-qualified candidate; do not
  tune the rejected peak identity, eligibility, or frequency ownership
- [x] keep current Signal output, hashes, mechanics, calibrated gates, and
  Batch 29.8 frozen

Evidence: one loss-attribution report and one license-safe state-policy memo.
No borrowed implementation expression or constants, parameter sweep, listening,
dynamic ratio, cache, realtime, routing, promotion, or production DSP change
belongs in this batch.

Decision: the original material-state attribution was incomplete. Independent
recurrence fails `40/48` rows. Adding 29.7M sharing repairs all `24` tone rows
relative to that stage, but regresses `22/24` image rows and still fails
`29/48`. Evidence `d2de8ca4df6330f6` repeats with zero structural failures.
Current reference-relative recurrence remains the default. Later ownership is
ordered `Reset`, `TrackedPeak`, `Relational`; independent `Unlocked` and kick
laws are not promoted. A tracked overlay must keep the requesting channel's
peak location and advance from matched predecessor synthesis state.

### Batch 29.7O - Reference-Safe Tracked-Peak Overlay

- [x] freeze current reference-relative recurrence as the initial and fallback
  result for every bin
- [x] retain each requesting channel's current peak location; borrow only a
  compatible peer trajectory evaluated at that frequency
- [x] advance the overlay from the matched predecessor's synthesis state and
  use identity local phase offset only
- [x] alter eligible peak regions only; preserve current output everywhere else
- [x] preserve hashes, mechanics, geometry, schedule, crop, calibrated gates,
  exact repeat, and the frozen comparison matrix
- [x] accept only row-complete improvement over current Signal with no new
  structural, transformation, transient, crossfeed, or local-consistency loss

No phase scale, peak-resolution, predecessor-distance, activation-threshold,
or frequency-range sweep belongs in this batch. Reset implementation, listening,
dynamic ratio, cache, realtime, routing, promotion, and production DSP remain
closed.

Decision: reject. The candidate is active, repeat-stable, structurally exact,
mechanics-exact, and silent-peer safe. Calibrated failures rise from `20/48` to
`25/48`; row-complete improvements are `0/48`, all `48` regress on at least one
metric, and `34/48` fail local consistency. Evidence `ec1f63ad4bae9fc8` closes
parameter rescue. This late tracked phase overlay is unsafe. Whether the loss
comes from operator ordering or a deeper kernel mismatch remains unproven.

### Batch 29.7P - Peak-Owner And Phase-Field Integration Research

- [x] attribute 29.7O phase error by peak anchor, region interior, overlay
  boundary, ratio, and control without rendering a new candidate
- [x] compare Signal's integration order with primary peak-locked phase-vocoder,
  nonstationary Gabor, and real-time phase-gradient integration literature
- [x] inspect permissive implementations where available; keep GPL material at
  architecture-only distance
- [x] decide whether tracked peaks may seed or constrain the predictor before
  integration, require one complete peak-owned region operator, or must close
  for the current coherent kernel
- [x] promote at most one bounded operator-ordering law into architecture and
  Rule 31H before authorizing another proof
- [x] keep the current renderer, frozen gates, Batch 29.8, listening, dynamic
  ratio, realtime, routing, and production unchanged

Evidence: one phase-field attribution report and one source-backed translation
memo. No renderer, parameter sweep, peak-picker change, eligibility change,
frequency-range change, phase-offset scale, or reset implementation belongs in
this batch.

Decision: require one complete peak-owned eligible-region operator. The
unchanged 29.7O renderer raises relation RMS from `0.057562` to `1.485181` at
anchors, `0.038310` to `1.197048` in interiors, and `0.129766` to `1.182947`
at boundaries. Every ratio and both control families regress in the same
direction. Evidence `e1713e619138301b` repeats exactly. The fault is field-wide,
so boundary repair and post-integration peak seeds are rejected.

### Batch 29.7Q - Complete Peak-Owned Region Proof

- [x] implement one report-only eligible-region operator that establishes one
  tracked phase owner before deriving the region field
- [x] preserve the peer's current same-frequency analysis relation inside that
  same operator
- [x] reuse 29.7O peak selection, predecessor eligibility, tracked phase
  advance, identity offsets, source geometry, and schedules unchanged
- [x] retain current relational recurrence for every complete ineligible region
- [x] prove exact length, finite output, repeat, mechanics, mono parity, frozen
  calibrated gates, and local consistency on the unchanged `48` rows
- [x] stop after one candidate; do not add a parameter sweep, boundary blend,
  seed-only variant, picker change, eligibility change, or offset scale
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, and
  production unchanged

Evidence: one candidate report and exact repeat hash. Passage requires zero
calibrated failures, complete non-regression on every row, zero local
consistency failures, and exact mechanics. Failure returns to operator review.

Decision: reject and return to operator review. The complete owner reduces the
late-overlay result from `25/48` to `23/48` calibrated failures and local
failures from `34/48` to `27/48`, but still trails the `20/48` relational
baseline. Only `2/48` rows improve completely and `46/48` regress somewhere.
Structure, mechanics, silent-peer safety, and repeat pass exactly at evidence
`2a52a1106fadf298`. The frozen acceptance bar is not met. No parameter or owner
variant is authorized.

### Batch 29.7R - Current-Kernel Operator Review

- [x] consolidate the 29.7M, 29.7N, 29.7O, 29.7P, and 29.7Q evidence into one
  operator decision record
- [x] distinguish gains from complete owner ordering from remaining losses in
  the current coherent predictor and overlap synthesis
- [x] decide whether linked tracked-peak work closes for the current coherent
  kernel or requires a separately contracted phase-field kernel family
- [x] define the next research question at algorithm-family scale; do not open
  another peak owner, picker, eligibility, range, scale, or blend variant
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, and
  production closed

Evidence: one operator decision record promoted into architecture and Rule
31H. No renderer or parameter experiment belongs in this batch.

Decision: close linked tracked peaks inside the current coherent kernel. The
kernel is one continuous weighted phase field and its pure-stretch source
architecture does not use peak mapping. Rubber Band's linked peak trajectory
belongs to a different, state-complete phase-vocoder kernel. The progression
from `29` to `25` to `23` failures proves ownership corrections matter, but
failure to beat the `20/48` relational baseline closes another local variant.
Translation memo 010 promotes the kernel boundary.

### Batch 29.7S - Linked Phase-Field Kernel Family Selection

- [x] compare one joint multichannel phase-gradient integration family with one
  state-complete peak-locked phase-vocoder family
- [x] use primary literature and permissive implementations where available;
  keep Rubber Band GPL source at architecture-only distance
- [x] define representation, horizontal and vertical phase ownership,
  transient/reset policy, linked-channel policy, and overlap synthesis for each
  family as one system
- [x] assess compatibility with the proven coherent mono baseline, later
  multi-resolution work, deterministic offline rendering, and realtime-safe
  state projection without implementing any family
- [x] select at most one family and promote its clean-room boundary into
  architecture and Rule 31H, or close both with an explicit research gap
- [x] define one bounded fixed-ratio proof only if the selected family has no
  unresolved ownership or licensing gap
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, and
  production closed

Evidence: one source-backed family decision matrix and one promoted kernel
contract. No renderer, parameter sweep, picker, classifier, scale crossover,
or reset implementation belongs in this batch.

Decision: close joint phase-gradient integration for the next renderer. Signal
already rejected its mono fixed-grid kernel and exact-lattice repair, and no
published source closes joint multichannel heap ownership. Select one separate
`SharedRotationRegionLocked` phase-vocoder family. Translation memo 011 and
Rule 31H freeze complete-region common rotation, cancellation-safe energy,
dominant-channel peak advance, coordinated reset, and exact overlap ownership.

### Batch 29.7T - Shared-Rotation Region-Locked Kernel Proof

- [x] add one report-only `SharedRotationRegionLocked` renderer beside the
  unchanged coherent control; do not call the weighted predictor from it
- [x] reuse the coupled periodic-Kaiser/modified-half-bin representation,
  exact absolute analysis centres, `30 ms` output interval, fourfold support,
  boundary handling, inverse transform, overlap accounting, and target length
- [x] form joint peak energy as the maximum per-channel energy at each bin; a
  peak is a nonzero local maximum against the two available neighbours on each
  side, with stable lower-bin plateau ties
- [x] place each region boundary at the lowest-energy bin between adjacent
  peaks, with stable lower-bin ties; an active frame with no peak is one
  `ResetRegion`
- [x] match a current peak to the prior region containing its frequency; use
  `ResetRegion` on the first frame, discontinuity, silent predecessor, or
  missing predecessor, otherwise use `TrackedRegion`
- [x] select the greatest-energy current channel at the peak with stable
  lower-channel ties; retain the predecessor common rotation and every
  channel's predecessor-peak analysis phase across owner changes
- [x] estimate the owner trajectory over the actual adjacent analysis-centre
  interval, advance it over the fixed synthesis interval, calculate one common
  rotation, and apply that rotation to every current channel coefficient in
  the complete region
- [x] keep exact-zero `Silent` regions zero; add no attack detector, local-time
  override, `Relational`, `Unlocked`, random-phase, mid/side, post-render,
  blended, classifier, or multiresolution state
- [x] prove `TrackedRegion`, `ResetRegion`, and `Silent` exercise plus exact
  length, coverage, finiteness, identity, silence, mono parity, hard pan, swap,
  polarity, scaled duplicate, owner changes, trajectory breaks, and repeat
- [x] run the unchanged mono integrity/corpus gates and `48` calibrated stereo
  rows at `0.75x`, `1.5x`, and `2.0x` against current Signal and Rubber Band
- [x] require zero calibrated stereo failures, zero local-consistency failures,
  exact mechanics, and no row-complete mono regression; stop after one
  candidate without tuning any owner, picker, predecessor, reset, boundary,
  window, scale, threshold, or blend
- [x] keep current output, production identity, Batch 29.8, listening, dynamic
  ratio, realtime, routing, and cache identity unchanged

Evidence: one complete candidate report, state/mechanics counts, per-row
comparison, audio hashes, and exact repeat hash. Failure returns to
algorithm-family operator review. Passage opens Batch 29.8 only after a
separate roadmap checkpoint.

Decision: reject passage and open operator review. The complete kernel reduces
calibrated stereo failures from `20/48` to `1/48`, produces `30/48` complete
improvements, and preserves exact mechanics. It still has `11/48` local-
consistency failures and 18 rows regress on at least one metric. All local
failures are tone rows. The sole calibrated failure is the short, off-bin
`2.0x` tone at `0.009708 rad` whole-render IPD against the `0.006 rad` gate.
The unchanged six-row mono corpus has zero hard failures and zero row-complete
regressions. State counts are `58,352` tracked, `1,445` reset, `165` silent,
`59,797` regions, and `7,710` owner switches. Stereo evidence
`eff52febad8c0fb8`, mechanics `ad907a31d6ae940a`, and corpus
`c062525dfa1da3ff` repeat exactly. No tuning rescue is authorized.

### Batch 29.7U - Region-Locked Tone-Continuity Operator Review

- [x] freeze the 29.7T renderer, row set, hashes, thresholds, and mono result;
  make no renderer, picker, boundary, trajectory, reset, window, or blend change
- [x] classify the 11 tone-local failures by ratio, length, phase, alignment,
  whole/interior scope, region transition, owner switch, and trajectory break
- [x] compare the frozen current, candidate, and calibrated Rubber Band tone
  evidence; locate the first divergence in peak trajectory integration,
  predecessor-region assignment, complete-region rotation, or overlap
- [x] explain why image rows improve while steady-tone local consistency fails;
  do not infer a parameter threshold from the one calibrated miss
- [x] decide whether one source-backed operator-law correction is bounded, a
  different complete family is required, or the family closes; implement none
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, cache, and
  production closed

Evidence: one operator decision record with the 11-row failure map, first-
divergence attribution, comparator boundary, and at most one next proof law.
No renderer or tuning experiment belongs in this batch.

Decision: authorize one finite-support reset proof. All eleven maximum local
residuals occur in the first or last of eight windows. Candidate interior IPD
is `5.97e-8` to `2.08e-6 rad`; fixed-ratio centres never break trajectory;
owner changes and reset regions remain abundant inside stable interiors.
Frame-local common rotation preserves each current coefficient relation. The
first measured divergence is overlap of boundary-conditioned tracked frames,
whose differing rotations do not preserve the finite input Gram relation when
summed. Rubber Band is lower in the same worst boundary window on all eleven
rows. This is a missing nonstationary boundary state, not a peak threshold,
trajectory, normalization, or general overlap defect.

### Batch 29.7V - Finite-Support Reset Proof

- [x] copy the frozen 29.7T report-only kernel into one candidate path; preserve
  its row set, thresholds, mono evidence, representation, schedule, peak map,
  regions, owner, trajectory integration, common rotation, overlap, and hashes
  as the unchanged control
- [x] add exactly one parameter-free `FiniteSupportReset` law: when an analysis
  window intersects samples outside the known input domain, reset every active
  region to current analysis phase and create no predecessor trajectory
- [x] reset the first fully supported frame once because no boundary trajectory
  may seed it; resume the unchanged predecessor-region law afterward
- [x] add no attack threshold, onset detector, release detector, local-time
  override, unlock, random phase, blend, alternate window, picker, boundary,
  scale, classifier, mid/side, post-render repair, or comparator-derived value
- [x] rerun exact mechanics, the unchanged six-row mono corpus, all `48`
  calibrated stereo rows, and the 11-row boundary-window map against current
  Signal, frozen 29.7T, and calibrated Rubber Band
- [x] require exact mechanics, zero calibrated stereo failures, zero local-
  consistency failures, no row-complete mono regression, and no regression on
  the 37 previously passing local rows; stop after this one candidate
- [x] keep production, Batch 29.8, listening, dynamic ratio, realtime, routing,
  cache identity, and all product-facing surfaces closed

Evidence: one repeat-stable candidate report, the 11-row boundary comparison,
state counts, mono result, and exact hashes. Failure closes finite-support reset
without tuning and returns to complete-family review.

Decision: reject finite-support reset. Calibrated failures rise from `1/48` to
`4/48`, local failures rise from `11/48` to `19/48`, nine previously passing
local rows regress, and only one original local failure closes. All four
calibrated misses are short `0.75x` image rows. Dedicated structural and
symmetry mechanics plus the six-row mono corpus pass, but candidate parity with
the frozen mono control fails at `1.262698`, `1.262698`, and `5.050797` maximum
sample error. The law improves some tone boundary windows and worsens others.
No tuning follows.

### Batch 29.7W - Material-State Boundary Architecture Review

- [x] freeze the 29.7T and 29.7V renderers, rows, thresholds, state laws, and
  hashes; implement no renderer, detector, classifier, state, or scale
- [x] classify the four calibrated image failures, 19 local failures, nine new
  regressions, one fixed original row, and mono-parity loss by material, ratio,
  length, boundary side, and state transition
- [x] compare the direct split with the promoted source record for ordinary,
  locked, reset, unlocked, channel-linked, and frequency-partitioned ownership;
  transfer no source expression, constants, thresholds, or reset ranges
- [x] decide whether one bounded complete material-state architecture has
  independent support and a falsifiable proof, or close the shared-rotation
  family; do not authorize a local reset-range or head/tail variant
- [x] record why the complete architecture is not yet independently justified,
  retain common rotation only as locked-state evidence, and require the missing
  seams to close before another renderer card
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, cache,
  production, and product-facing work closed

Evidence: one architecture decision record linking direct failure classes to
source-backed state ownership. No DSP experiment belongs in this batch.

Decision: close shared rotation as a complete renderer. The 29.7V failures
split into 15 tone and four image local rows; its four separate calibrated
misses are every short `0.75x` image row. Five retained original failures peak
at the head and five at the tail. Universal tracking and universal reset are
both wrong.

Pinned R3 computes ordinary advance before selecting reset, unlocked, or
peak-locked output by frequency. Channel borrowing is conditional inside the
locked branch, and frequency-scale ownership is separate. Bungee, Signalsmith,
and papers independently support ordinary advance, peak lock, reset, common
rotation, and linked-channel ownership. Only Rubber Band in the current record
supplies explicit material-guided unlock plus simultaneous nonoverlapping
frequency-owned scales and their complete ordering. That is not enough for a
clean-room renderer. Common rotation remains locked-state evidence; no further
29.7T/29.7V variant is authorized.

### Batch 29.7X - Independent Material-State Kernel Research

- [x] freeze production, 29.7T, 29.7V, all objective rows, thresholds, hashes,
  and translation memo 012; implement no renderer or policy prototype
- [x] find at least one independent implementation or published construction
  for material-guided ordinary-versus-unlocked phase ownership; distinguish a
  pro-quality law from random diffusion or one-source heuristics
- [x] find at least one independent implementation or published construction
  for simultaneous nonoverlapping frequency-owned scale synthesis; reject
  redundant full-band unions and time-selected resolution
- [x] map candidate evidence to boundary handling, mono continuity, linked-
  channel compatibility, exact scale reconstruction, deterministic offline
  bounds, and clean-room licensing
- [x] decide whether the two seams support one complete Signal-owned state and
  scale order; if not, close this source-studied successor lane rather than
  inferring missing policy
- [x] if supported, promote exact state ownership, scale ownership, observables,
  mechanics, objective passage, and one-candidate stop rule before preparing an
  implementation card
- [x] keep Batch 29.8, listening, dynamic ratio, realtime, routing, cache,
  production, and all product-facing work closed

Evidence: one independent-source matrix and one promoted architecture decision
or explicit lane closure. No DSP experiment belongs in this batch.

Decision: independent papers close both seams. Fuzzy material classification
supports transient reset plus noise-dependent phase diffusion at commercial-
comparator listening quality. Bonada supplies a complete simultaneous
frequency-owned, linked-stereo phase-vocoder construction. Painless
frequency-adaptive Gabor-frame work supplies the exact canonical-dual
reconstruction missing from parallel masked STFTs. Translation memo 013
selects one `FrequencyAdaptiveMaterialPhase` proof and explicitly rejects a
revival of Batch 29.6CH.

### Batch 29.7Y - Frequency-Adaptive Material-Phase Proof

- [x] Stage A: implement one report-only painless frequency-adaptive frame on
  a common time lattice with exclusive long/middle/short atom ownership and one
  canonical dual
- [x] stop before time stretch unless untouched coefficients reconstruct at or
  below `1e-12` peak error in `f64` and pass exact crop, coverage, frame-bound,
  channel-relation, silence, boundary, and repeat mechanics
- [x] Stage B, only after Stage A passes: add the complete shared fuzzy
  material map, transient shoulder/reset law, retained common-region rotation,
  and deterministic channel-common noise perturbation
- [ ] run exactly one candidate at `0.75x`, `1.5x`, and `2.0x` through the
  frozen synthetic, six-row mono, and `48`-row calibrated linked-stereo gates
- [ ] require zero calibrated and local-consistency stereo failures, exact
  mechanics, and no row-complete mono regression; do not tune after a miss
- [x] keep listening, Batch 29.8, dynamic ratio, realtime, routing, cache,
  production, and all product-facing work closed

Evidence: one Stage A exact-reconstruction report, then at most one complete
Stage B objective report. No concealed listening pack exists before hard
passage.

Stage B stops as an architecture miss. The frozen candidate completes the
synthetic, calibrated stereo, and mechanics phases, then remains inside the
repeated six-row mono corpus after more than five hours. Its already-complete
stereo report rejects at `36/48` calibrated failures and `46/48`
local-consistency failures. Continuing the report cannot change passage, so
the runaway mono repeat is stopped. No DSP value changes after the miss.

### Batch 29.7Z - Material Transport Architecture Reassessment

- [x] attribute the first linked-channel relation divergence before the common
  material operator; test whether independent per-channel polar interpolation
  breaks retained interchannel phase
- [x] define one relation-preserving coefficient-resampling law with a shared
  source trajectory and explicit retained channel relation, or close this
  transport family
- [x] replace the whole-source report execution shape with a bounded sliced
  proof design that preserves Stage A ownership and canonical-dual identity
- [x] use primary-source and prior Signal evidence only; implement no renderer,
  tuning candidate, listening export, dynamic ratio, or product work

Evidence: one architecture attribution and one bounded execution decision. No
DSP experiment belongs in this batch.

Decision: retain the family for one final architecture-corrected proof.
Independent polar interpolation can differ from interpolation of the channel
relation by `180` degrees in a two-frame counterexample; the later common
operator cancels from that error. Memo 014 selects one reference coefficient,
one explicitly interpolated peer/reference relation, and peer-owned magnitude.
It also selects a fixed `16384`-frame sliced transform, `8192`-frame advance,
`512`-frame coefficient lattice, identical sine analysis/synthesis outer
windows, and at most two active slices. The squared overlapping windows sum to
one. This is a new exact sliced frame, not an optimization claimed equivalent
to the rejected full-length coefficients.

### Batch 29.7AA - Relation-Owned Sliced Material Proof

- [x] Stage A: implement only the fixed `16384/8192/512` sliced frame with the
  frozen `4096/2048/1024` atom supports and `750 Hz`/`6 kHz` ownership
  boundaries
- [x] prove identical outer windows
  `h[n] = sin(pi (n + 0.5) / 16384)` satisfy exact two-slice square partition,
  the inner painless dual reconstructs each slice, and combined identity stays
  at or below `1e-12` peak error
- [x] require exact crop, coverage, conjugate closure, silence, hard pan, swap,
  polarity, scaled duplicate, reflected boundaries, and repeat across short,
  non-aligned, and multi-slice lengths `[1, 4095, 8192, 12289, 220500]`
- [x] prove at most two active slices, peak live coefficient memory independent
  of source duration across `[8192, 65536, 220500]`, and counted
  analysis/synthesis work equals fixed per-slice cost times slice count; stop
  before transport on any Stage A miss
- [x] Stage B, only after Stage A passes: restore each peer from one sampled
  reference phase, peer-owned magnitude, and one directly interpolated current
  peer/reference relation before applying the frozen 29.7Y material operator
- [x] exercise two-defined, one-defined, undefined, zero-peer, and joint-silent
  relation states; require zero undefined states on active calibrated rows
- [x] freeze supports, crossovers, material classifier, median spans, transient
  law, diffusion, seed, peak map, relation law, slice geometry, and gates before
  the first candidate; do not tune after a miss
- [x] run synthetic and exact mechanics, then the `48`-row calibrated stereo
  gate; stop before the long mono corpus on any calibrated, local-consistency,
  mechanics, or explicit relation failure
- [x] enforce the mono stop gate: the stereo miss prevents the repeated six-row
  mono gate from running
- [x] keep listening, Batch 29.8, dynamic ratio, realtime, routing, cache,
  production, and all product-facing work closed

Evidence: one Stage A sliced identity/boundedness report, then at most one Stage
B objective report. Any miss closes this family.

Stage A passes. Peak identity error is `4.44e-16`; conjugate closure and all six
mechanics categories have zero failures. The five identity lengths require
`[2, 2, 2, 3, 28]` slices. Boundedness rows hold at two live slices and `86016`
peak live coefficients while counted work remains exactly `1111425` units per
slice. Evidence hash: `0830ec12fa0bcde7`. Stage B is now open once; listening
and product work remain closed.

Stage B closes the family. Synthetic structure, repeat, bounded slice state,
and explicit relation mechanics pass. Both output layers use one shared
relation; active calibrated rows have zero undefined states and `1.78e-15`
maximum relation error. The sample-domain result still rejects at `44/48`
calibrated and `46/48` local-consistency failures, with frozen mono-parity
mechanics also outside gate. Mono does not run. Evidence hash:
`225ab337875b3962`. No parameter rescue or listening pack is authorized.

### Batch 29.7AB - Joint-Synthesis Architecture Reassessment

- [x] attribute the first divergence between the passing shared coefficient
  relation and the failing synthesized sample-domain relation; separate inner
  band overlap, outer slice overlap, band-varying relation, and material phase
- [x] review primary literature and clean-room source evidence for linked
  multichannel relation preservation through redundant or overlapping
  synthesis, including post-atom summation ownership
- [x] define one independently supported joint-synthesis invariant compatible
  with bounded execution, or close the current frequency-frame direction
- [x] implement no renderer, parameter change, listening export, dynamic ratio,
  realtime, routing, cache, production, or product-facing work

Evidence: one post-coefficient attribution and one source-backed architecture
decision. Another DSP candidate requires a promoted contract; Batch 29.8 stays
closed.

Decision: inner synthesis is the first causal sum. Exact relations on atoms
with varying peer/reference phase and magnitude ratios do not define a relation
for their sum. Common material phase changes cross-atom interference; outer
slice overlap adds another sum but is not the first cause. In frame notation,
identity proves `D A = I`, while modified fields also require `A D C = C`.
Rule 31K and memo 015 promote that consistency condition plus post-projection
waveform stereo ownership. The current frequency-adaptive direction closes.

### Batch 29.7AC - Paired-Channel Joint-Consistency Operator Study

- [x] review primary multichannel consistency, spatial covariance, and
  alternating-projection evidence for one paired-channel constraint compatible
  with `A D C = C`
- [x] define the order of transform consistency, channel relation or covariance,
  magnitude ownership, and waveform validation as one complete operator
- [x] require fixed finite work, deterministic state, explicit non-convergence,
  and a clean-room boundary before promoting any proof
- [x] implement no renderer, parameter sweep, listening export, dynamic ratio,
  realtime, routing, cache, production, or product-facing work

Evidence: one source-backed operator decision. If no complete bounded operator
is supported, close transform-domain joint projection and assess a waveform-
domain family. Batch 29.8 stays closed.

Decision: no complete operator is supported. Additive-mixture projection owns a
known sum that arbitrary stereo does not have. Covariance matching is a spatial
renderer and may add decorrelated energy; it does not uniquely preserve source
waveform or image. Alternation with `A D` has no supported feasible
intersection, order, finite iteration count, or failure result. Rule 31L and
memo 016 close transform-domain post-projection.

### Batch 29.7AD - Whole-Family Waveform-Ownership Study

- [x] compare complete source-synchronous, sinusoidal, and single-grid transform
  topologies using primary literature and existing clean-room source dossiers
- [x] require one shared stereo timeline, explicit transient and tonal ownership,
  valid waveform synthesis by construction, and fixed bounded execution
- [x] define objective rejection gates and one exact proof scope before
  authorizing at most one renderer; close every underspecified family
- [x] implement no renderer, parameter sweep, listening export, dynamic ratio,
  realtime, routing, cache, production, or product-facing work

Evidence: one whole-family architecture decision. No hybrid assembly from
partial mechanisms and no implementation without a complete promoted topology.
Batch 29.8 stays closed.

Decision: close WSOLA as the universal polyphonic engine and retain explicit
sinusoidal models as research reserve. Select one single-grid
`StateCompleteLinkedPhaseVocoder` proof. It combines ordinary advance, reset,
lock, unlock, linked-channel decisions, and synthesis inside one topology.
Rule 31M replaces one-shot guessed-policy rejection with bounded development
calibration followed by one frozen concealed holdout.

### Batch 29.7AE - State-Complete Linked Phase-Vocoder Calibration

- [x] implement one report-only single-grid state machine with `Reset`,
  `Locked`, `Unlocked`, and `Silent` ownership; do not call the coherent
  weighted predictor or any rejected renderer
- [x] freeze physical bounds, quantization, ordering, and at most 64
  deterministic candidates over the six Rule 31M controls before rendering
- [x] use short development rows first; advance at most four candidates to the
  complete synthetic, mono, and `48`-row stereo development matrix
- [x] enforce exact mechanics, zero calibrated and local stereo failures, and
  no row-complete mono regression; freeze no candidate when all four miss
- [x] keep the existing six-row family-balanced holdout unread; Batch 29.8 may
  open it only after one candidate freezes
- [x] keep listening export, dynamic ratio, realtime, routing, cache,
  production, and product-facing work closed

Evidence: bounded calibration ledger and one frozen development report.
Complete development passage alone opens Batch 29.8 and its concealed holdout.

Decision: close without a frozen candidate. Candidate `0` reproduces the
retained 29.7T boundary at `1/48` calibrated and `11/48` local failures.
Candidates `1`, `16`, and `17` retain the one calibrated miss and worsen local
failures to `17`, `15`, and `13`. All four have exact mechanics, zero mono hard
failures, and zero row-complete mono regressions. Policy changes do not address
the persistent off-bin `2.0x` tone or the boundary-local failures.

### Batch 29.7AF - State-Decision Failure Attribution

- [x] trace candidate `0` and the best state-changing finalist at the single
  calibrated off-bin `2.0x` tone miss and all eleven retained local misses
- [x] identify the first coefficient, inverse-frame, or overlap window where
  candidate relation error exceeds the coherent control
- [x] prove whether the miss is caused by state classification, predecessor
  continuity, or overlap interaction; do not change policy values
- [x] authorize at most one equation-level correction only when the trace names
  one causal operation; otherwise close the selected single-grid family
- [x] keep the concealed holdout, listening, dynamic ratio, realtime, routing,
  cache, production, and product-facing work closed

Evidence: one causal attribution report. No renderer sweep or policy rescue.

Decision: close the selected single-grid family. Per-bin linked coefficient
relations remain exact within `1.78e-15`. Seven local misses first appear when
the full `1024`-sample inverse frame is reduced to the `960`-sample synthesis
support. Four, including the sole calibrated off-bin `2.0x` miss, already
diverge in the full inverse frame. Candidate `17` preserves the same split.
Neither overlap accumulation nor normalization is first. Evidence hash
`fc10cd6442d55e4a`. Two causal synthesis operations prohibit one equation
correction under Rule 31M.

### Batch 29.7AG - Waveform-Domain Linked-Stereo Re-entry

- [x] research complete source-backed topologies that preserve arbitrary
  linked-channel waveform relations through inverse synthesis and finite
  support, not only per-bin phase relations
- [x] state one testable waveform-domain invariant covering full inverse,
  support restriction, and overlap synthesis
- [x] require the same topology to own polyphonic tone, transients, mono
  quality, linked stereo, and fixed bounded execution
- [x] select at most one complete proof or stop the active stretch lane; do not
  assemble another renderer from partial mechanisms
- [x] keep the concealed holdout, listening, dynamic ratio, realtime, routing,
  cache, production, and product-facing work closed

Evidence: one research-backed architecture decision under Rule 31N. No
renderer or parameter experiment.

Decision: select `LinkedSubbandSinusoidalModel` for source feasibility only.
Pinned SBSMS `2.3.0` demonstrates one complete topology with recursive octave
subbands, explicit compatible stereo-track pairing, jointly evolved partial
trajectories, and direct oscillator synthesis. Each matched partial owns one
output waveform relation before the subband sum; no inverse STFT, support crop,
or overlap normalization follows it. The invariant is component-local, so
sample-domain stereo gates remain authoritative. No Signal renderer opens yet.

### Batch 29.7AH - Pinned Linked-Subband Sinusoidal Feasibility

- [x] pin SBSMS `2.3.0` at
  `e99cd7e6c6367e476577be34d2fdbe2023904d7e`; build and run it only as an
  external GPL research specimen under `target/`
- [x] freeze the existing synthetic mechanics, exact shared-mono control, mono
  development material, and `48`-row stereo development matrix before the
  first specimen render; keep the concealed holdout unread
- [x] capture repeatable source behavior for identity/model residual, tones,
  chords, partial crossings, long decays, isolated and dense transients,
  noise, boundaries, exact length, and mono objective position
- [x] trace matched stereo tracks through direct oscillator samples and the
  subband sum; run waveform IPD, correlation, mid/side, Gram, pan, swap,
  polarity, and local-consistency gates
- [x] measure runtime scaling, memory behavior, active-track counts, and event
  counts; state the finite capacities and overflow result a Signal proof would
  require
- [x] authorize a clean-room Signal proof only if the exact source topology
  avoids the Rule 31M failures, reaches the declared development envelope,
  shows no broad material or boundary defect, and admits fixed execution bounds
- [x] do not copy GPL source, tune the specimen, assemble another renderer,
  listen, access the holdout, or open dynamic ratio, realtime, routing, cache,
  production, or product-facing work

Evidence: one pinned exact-source feasibility report under Rule 31O. No Signal
renderer or product integration.

Decision: close `LinkedSubbandSinusoidalModel`. The pinned source repeats and
passes the aggregate stereo gate at `0/48`, but fails six local-consistency
rows, exact linked mechanics, seven mono hard rows, and two row-complete mono
comparisons. All six long development rows contain metrics worse than both
coherent Signal and Rubber Band, `21` in total. Dynamic track state has no
fixed capacity or overflow result. Evidence hash `79b5f7c14692b8f5`. No
clean-room Signal renderer opens.

### Batch 29.7AI - Professional-Comparator Gate Validity

- [x] freeze the existing `48` stereo rows, thresholds, whole/interior metrics,
  eight-window local rule, and exact mechanics before the first comparator run
- [x] run pinned Rubber Band R3 `4.0.0` through duplicate, mono parity, hard
  pan, swap, polarity, gain, calibrated stereo, and local-consistency evidence
- [x] repeat every render and calculation; retain exact input, output, version,
  command, and measurement hashes
- [x] if Rubber Band passes, retain the rules and record a confirmed topology
  gap; if it fails, separate comparator-bounded acceptance from exact
  diagnostics and revise only the invalidated rule
- [x] do not implement or tune a renderer, optimize against comparator
  waveforms, listen, access the holdout, or open dynamic ratio, realtime,
  routing, cache, production, or product-facing work

Evidence: one professional-comparator gate-validity report under Rule 31P. No
renderer or parameter experiment.

Decision: revise the local and exact-mechanics rules. Rubber Band repeats with
zero calibrated failures but fails `13/48` Signal-relative local rows. It
improves `245/384` windows and has a lower global maximum local residual than
Signal, `0.01744693815260` versus `0.02522090848652`. Duplicate equality, mono
parity, silent-peer isolation, and swap pass exactly. Polarity and quarter-gain
errors reach `0.950164794921875` and `0.04590606689453125`, so those become
diagnostics. Rule 31Q freezes the corrected professional envelope. Evidence
hash `b9331f0858326f19`.

### Batch 29.7AJ - Shared-Decision Waveform Topology Research

- [x] freeze Rule 31Q and reclassify completed candidates without reopening a
  renderer or changing retained evidence
- [x] trace how pinned source-backed professional renderers separate nonlinear
  material classification from duplicate-, mono-, pan-, and swap-equivariant
  channel synthesis
- [x] compare at most three complete topology families; reject partial repair,
  post-hoc image projection, independent-channel synthesis, and unbounded state
- [x] select at most one topology with a testable waveform owner, fixed work
  and memory bounds, mono/polyphonic/transient coverage, and a direct path to
  the corrected professional envelope; otherwise stop the active stretch lane
- [x] do not implement or tune a renderer, listen, access the holdout, or open
  dynamic ratio, realtime, routing, cache, production, or product-facing work

Evidence: one source-backed architecture decision promoted into contract
`082`. No renderer or parameter experiment.

Decision: select one clean-room
`GuidedFrequencyPartitionedLinkedPhaseVocoder` proof. Pinned Rubber Band R3
separates per-channel material guidance from one synchronized all-channel phase
update and per-channel synthesis. Signalsmith independently supports a
greatest-energy reference with peer-relative synthesis. Bungee independently
supports common locked-region rotation before per-channel synthesis. Only R3
composes those invariants with complete material states and nonoverlapping
frequency-owned scales.

This does not reopen Batch 29.6CH. That prototype combined frequency bands with
an incomplete phase/channel translation. Batch 29.7Y proved exact multiscale
reconstruction but placed independent polar channel interpolation before the
shared operator. Rule 31R instead makes exclusive scale ownership,
synchronized phase-state selection, conditional linked trajectory borrowing,
and per-channel synthesis one indivisible waveform owner. External expression,
constants, masks, thresholds, and ranges remain excluded.

### Batch 29.7AK - Guided Frequency-Partitioned Linked-Phase Proof

- [x] freeze memo 019 and Rule 31R before implementation; declare all Signal-
  owned scale, crossover, classifier, state, capacity, and overflow policy
  before the first render
- [x] Stage A: implement one report-only fixed-capacity kernel with exhaustive
  nonoverlapping scale ownership, one synchronized all-channel phase-state
  update, conditional compatible peak borrowing, and per-channel synthesis
- [x] stop unless Stage A passes `1e-12` `f64` identity reconstruction, exact
  crop/coverage/finite/repeat/bounded-state mechanics, all scale/state branch
  coverage, and the four Rule 31Q channel mechanics at `1e-6`
- [x] Stage B, only after Stage A passage: start one preregistered complete
  policy; stop before the objective rows when the frozen capacity rejects the
  `8 kHz` gate representation
- [ ] require the unchanged calibrated gate, at least `245/384` improved local
  windows, at most `13/48` local-row failures, maximum local residual at or
  below `0.01744693815260`, and no row-complete mono regression; stop on a miss
- [x] do not sweep factors, repair individual rows, listen, access the holdout,
  or open Batch 29.8, dynamic ratio, realtime, routing, cache, production, or
  product-facing work

Evidence: one Stage A mechanics report and, only after passage, one complete
Stage B objective report. No concealed listening pack exists before passage.

Frozen policy: reuse the Signal-owned 29.7Y `16384/8192/512` painless frame,
`4096/2048/1024` supports, and `750 Hz`/`6000 Hz` ownership. Capacity is two
channels, `1344` signed atoms, `673` nonnegative-frequency atoms, `32`
coefficients per atom, and `673` current/prior regions; excess returns
`CapacityExceeded`. Reuse the 29.7Y channel-joint fuzzy material map. Strict
transient maxima below `6000 Hz` reset, noise-owned coefficients remain on
ordinary recurrence, and all other coefficients use tracked peak lock.
Greatest-energy trajectory borrowing is conditional below `6000 Hz` and keeps
peer magnitude plus current analysis-relative phase. No valley movement,
shoulder gain, diffusion, random phase, calibration, or alternate law exists.

Decision: close this implementation before objective quality evidence. Stage A
passes at `48 kHz`: peak identity error is `2.914335439641036e-16`; all four
Rule 31Q mechanics are exact; every scale and state branch executes; compatible
linked/unlinked region counts are `1540/2673`; peak region high-water is
`156/673`; finite, repeat, and overflow checks pass. Evidence hash:
`79b0cc2047f563b6`.

The frozen Stage B workspace is not valid for the frozen gate. The same
`16384/512` frame with `750 Hz`/`6000 Hz` boundaries at `8 kHz` requires `2432`
signed and `1217` nonnegative-frequency atoms, above the frozen `1344/673`
capacity. The attempted whole-source output representation also grows its
coefficient lattice with render duration. Expanding either bound would violate
the preregistered capacity and Rule 31R. No `48`-row, mono, long-development,
listening, or holdout result exists.

### Batch 29.7AL - Bounded Multiscale Slice Compatibility Research

- [x] freeze 29.7AK code, hashes, capacity failure, and Rule 31R; implement no
  renderer, phase policy, capacity expansion, or quality candidate
- [x] compare the already-proven fixed two-slice frame with at most one sample-
  rate-normalized painless alternative; require duration-independent live
  memory and one formula across `8`, `44.1`, and `48 kHz`
- [x] trace whether one synchronized all-channel phase state can cross slice
  boundaries without duplicate frequency ownership, independent overlap
  normalization, relation projection, or state reset
- [x] select at most one representation integration with exact identity,
  explicit work/memory formulas, fixed overflow behavior, and a direct Stage A
  mechanics proof; otherwise stop the active topology lane
- [x] do not render stretched audio, tune policy, run objective rows, listen,
  access the holdout, or open Batch 29.8 or product work

Evidence: one bounded-representation compatibility decision. No DSP experiment
or audio artifact belongs in this batch.

Decision: select one normalized sliced representation for a Stage A proof.
The fixed `16384/8192/512` frame remains exact at `48 kHz`, but its duration
changes by sample rate and its `8 kHz` atom layout exceeds frozen capacity.
The selected formula uses `H = F/100`, `N = 32H`, outer advance `16H`, and
supports `8H/4H/2H`. Crossover bins remain exactly `240/1920`; atom spacing is
`4/8/16` bins. The `8/44.1/48 kHz` rows contain `380/191`, `1182/592`, and
`1260/631` signed/nonnegative atoms, all inside `1344/673`.

The outer sine square partition and inner painless canonical dual form one
synthesis law. One global common-lattice decision updates persistent channel
state once, then populates both active output layers. Six source and two output
coefficient slabs cap storage at `8 C B K`, or `645120 Complex64` slots for
`C=2`, `B=1260`, and `K=32`. Material halo, phase, region, overlap, transform,
and static representation terms are separately fixed by Rule 31T. No term
depends on render duration. Memo 020 records the complete comparison and work
formula.

### Batch 29.7AM - Normalized Sliced Frame Stage A

- [x] implement only Rule 31T geometry preparation for `8`, `44.1`, and
  `48 kHz`; return `UnsupportedGeometry` or `CapacityExceeded` before work on
  every declared miss
- [x] prove the three exact geometry and atom-count rows, exhaustive frequency
  ownership, `K=32`, tap bound, positive frame operator, and canonical inner
  dual
- [x] prove the outer sine square partition and combined sliced identity at or
  below `1e-12` across short, nonaligned, boundary-impulse, and multislice
  lengths at every proof rate
- [x] prove crop, two-layer coverage, conjugacy, silence, hard pan, swap,
  polarity, scaled duplicate, whole-render reflection, finite values, and
  repeat hashes
- [x] report `S(L)` slice counts, `Q(L)` state-token counts, every Rule 31T
  memory term, exact structural work counts, and duration-independent high-
  water across at least three lengths
- [x] advance one inert state token once per global common-lattice index and
  prove it crosses slice creation/retirement without reset or duplicate update
- [x] stop on any miss; do not add guided material policy, stretch audio, run
  objective rows, listen, access the holdout, or open product work

Evidence: one normalized sliced identity, mechanics, work, memory, boundary-
token, and overflow report. This card passes under Rule 31T.

Decision: freeze evidence hash `0407f765c7d84375`. Peak combined identity
error is `4.440892098500626e-16`; outer partition error is
`6.661338147750939e-16`; conjugacy is exact. All structural and mechanics
failure counters, nonfinite counts, token reset/duplicate/capacity failures,
and overflow failures are zero. Active token high-water is two. The exact
three-rate geometry, memory, and work rows match Rule 31T; maximum coefficient
storage is `645120 Complex64` slots. No guided or quality result exists.

### Batch 29.7AN - Guided State Slice-Boundary Mechanics

- [x] open only after 29.7AM passes; freeze its geometry, hashes, work/memory
  ceilings, and overflow behavior
- [x] adapt the passing 29.7AK synchronized channel state to one global sliced
  lattice without relation projection, independent overlap normalization, or
  state reset
- [x] prove all Rule 31R state branches and duplicate, mono parity, silent peer,
  and swap mechanics across interior and slice-boundary frames before quality
  policy work
- [x] keep material tuning, objective rows, listening, holdout, and product
  work closed

Evidence: one boundary-complete guided mechanics report. This card passes
under Rule 31U.

Decision: freeze evidence hash `90c10cd2e66d4faf`. The `3/6/14`-slice rows
run exactly `64/112/240` state updates with `32/80/208` dual-layer updates.
Every state branch executes in interior and before/at/after boundary contexts
at every proof rate. Duplicate, mono parity, silent peer, and swap errors are
zero. Layer magnitude and analysis-relative phase errors are at most
`1.1102230246251565e-16` and `4.440892098500626e-16`. Region high-water is
`32/100/107` of `191/592/631`; all continuity, capacity, update, finite, layer,
overflow, and repeat checks pass. No material or quality result exists.

### Batch 29.7ANR - Normalized Material Policy Preregistration

- [x] freeze the passing Rule 31T representation and Rule 31U mechanics hashes,
  geometry, state law, memory/work ceilings, and overflow behavior unchanged
- [x] map every unchanged Rule 31R material-policy term onto the normalized
  physical-time lattice, including scale-local medians, adjacent-frequency
  medians, strict transient maxima, state ordering, link/reset limits, and tie
  rules; stop on any ambiguous or geometry-dependent term
- [x] freeze one complete objective evidence matrix, unchanged hard thresholds,
  fixed work/capacity, failure ordering, and no-sweep/no-row-repair rule before
  implementation
- [x] either promote one Rule 31V preregistration that makes 29.7AO ready or
  close the integration; do not implement, render, listen, or access holdout

Evidence: one implementation-free policy and evidence preregistration. This
card passes under Rule 31V. The exact `4/2/1`-tick temporal radii,
same-scale adjacent-frequency medians, `19`-tick guidance dependency, strict
centre rule, material normalization, state order, linkage limits, tie rules,
fixed bounds, and failure-first objective matrix are frozen. `Ordinary` is the
mandatory recurrence precursor; reset, attack, unlocked, and locked are the
terminal material choices. No implementation or quality result exists.

### Batch 29.7AO - One Complete Objective Gate

- [x] open only after 29.7ANR passes and Rule 31V freezes the
  unchanged Rule 31R material policy on the normalized sliced representation
- [x] run the failure-first synthetic, corrected professional-comparator,
  mono, and long-development sequence through its first miss, with no factor
  sweep or row repair
- [x] stop on the first existing hard-gate miss; only complete passage may open
  Batch 29.8 listening and holdout work

Evidence: at most one complete objective report. Synthetic structure,
boundedness, repeat, all terminal states, and the four hard channel mechanics
pass at hash `0edf7cc256282813`. The repeated stereo gate rejects at `46/48`
calibrated failures, `110/384` improved windows, `44/48` local-row failures,
and maximum residual `0.86973539821584`; hash `ff4603accdb456e6`.
The six-row mono and long-development stage does not run. Rule 31V closes this
implementation without retry, tuning, listening, or holdout access.

### Batch 29.7AP - Normalized Stereo Failure Attribution

- [x] freeze Rule 31T/31U/31V hashes, the exact failed implementation, and all
  completed 29.7AO rows; change no renderer or threshold
- [x] trace one deterministic replay of all `48` development rows from source
  outer-layer coefficients through ordinary recurrence, synchronized state,
  linked/local lock, output-layer projection, inverse slice, and outer overlap
- [x] aggregate the first stereo-relation divergence by control, ratio, scale,
  state, source/output layer, and boundary context; do not select row repairs
- [x] compare the ownership order with the pinned Rubber Band, Signalsmith,
  and Bungee topology records; transfer no external expression or constants
- [x] promote at most one complete integration-law correction with a frozen
  proof, or close the topology; keep tuning, objective retry, listening,
  holdout, Batch 29.8, and product work closed

Evidence: one coefficient-to-waveform replay at hash `24cdad83bf3ddeeb`.
All `96` retained first/worst operator events are interior `Unlocked` state
commits; `90/96` have no owner switch. State-commit and projected-layer
residuals match exactly. Inverse and overlap expose but do not first create the
loss. Rule 31X promotes one reference-relative unlocked commit.

### Batch 29.7AQ - Reference-Relative Unlocked Commit

- [x] freeze the Rule 31W report and implement only the per-atom
  greatest-energy reference rotation for `Ordinary` and `Unlocked`
- [x] retain every ordinary precursor, channel magnitude, reset, attack,
  locked path, classifier, geometry, projection, inverse, and overlap law
- [x] prove observer parity, Rule 31Q mechanics, and unlocked interchannel
  relation preservation at or below `1e-12`
- [x] run the unchanged Rule 31V synthetic stage and corrected `48`-row stereo
  gate once; stop at the first miss and do not repair rows
- [x] run mono and long-development only after complete stereo passage; keep
  listening, holdout, dynamic ratio, product work, and Batch 29.8 closed

Evidence: synthetic mechanics pass at hash `875b0768ba2066bf`. The one corrected
stereo run rejects at `40/48` calibrated failures, `125/384` improved windows,
`44/48` local-row failures, maximum residual `0.8700034314389535`, and hash
`88d9c0f68ea2954b`. Structure and repeat pass. The correction improves six
calibrated rows and fifteen windows over Rule 31V but leaves the row-level
failure count unchanged. Mono and long-development do not run. Rule 31X closes
the topology without promotion, retry, tuning, listening, or holdout access.

### Batch 29.7AR - Direct Scale-Timeline Preregistration

Status: complete

- [x] freeze physical low, middle, and high analysis/synthesis durations and
  output advances at every proof rate
- [x] freeze exhaustive nonoverlapping physical-frequency ownership,
  crossover ties, and one source/output centre schedule shared by all scales
  and channels
- [x] define one coefficient and phase-state owner per scale/time/bin, with no
  outer meta-slice, dominant-layer selection, or layer projection
- [x] freeze complete state order: channel-local ordinary, reset, attack, and
  unlocked recurrence; predecessor-compatible cross-channel borrowing only in
  locked peak regions
- [x] freeze per-channel inverse overlap-add, same-channel scale summation,
  source reflection, crop, latency, tail, silence, and discontinuity behavior
- [x] calculate fixed input, output, guidance, phase, and peak capacities;
  define bounded work and explicit capacity failure
- [x] classify every Batch 29.6CH mechanic as reusable proof material or a
  rejected hazard; implement nothing and render no audio
- [x] promote Batch 29.7AS only when Rule 31Z geometry, ownership, capacity,
  boundary, and failure contracts are complete

Evidence: Rule 31Z and memo 022 freeze one `10 ms` direct lattice with
`80/40/20 ms` scales, `750/6000 Hz` upward crossover ties, exact
`191/592/631` owned-bin totals, absolute source projection, even reflection,
`130 ms` lookahead, complete state order, and fixed stereo capacities. Unity
is bit-exact bypass; each scale must reconstruct independently, while the
masked multi-scale sum is measured rather than falsely called identity. No
code or audio exists.

### Batch 29.7AS - Direct Scale Representation Mechanics

Status: complete

- [x] implement only the preregistered direct multi-scale analysis, exclusive
  frequency ownership, per-scale inverse overlap-add, and fixed storage
- [x] prove bit-exact unity bypass plus each unmasked scale's overlap and
  full-band reconstruction at `1e-12`; do not call the masked sum identity
- [x] prove conjugacy, exact ownership, crop, coverage, boundary schedule,
  capacity, finiteness, repeat, work counts, and explicit overflow behavior
- [x] report inert masked-sum gain, residual, timing, boundaries, and hashes on
  frozen silence, impulse, noise, crossover, and interior-tone controls
- [x] prove no state/coefficient projection between scales or outer fields
- [x] keep guided phase state, stretched audio, objectives, listening, and
  production closed; stop on the first representation-contract miss

Evidence: pass at hash `fdf90f6127749341`. All structural, storage,
capacity, unsupported-request, unity, crop, coverage, finite, work-count, and
repeat checks pass. Unmasked scale reconstruction peaks at `3.34e-16` and
conjugacy at `7.80e-14`. The `22` masked diagnostic rows have zero bounded-lag
timing. Fixed crossover rows reach `0.451615 dB` gain movement, `0.056339`
peak residual, and `0.055519` boundary error; these are frozen diagnostics,
not a tuning surface.

### Batch 29.7AT - Direct Scale State Mechanics

Status: complete

- [x] integrate the complete preregistered per-scale state order without
  changing geometry, crossovers, windows, capacities, or thresholds
- [x] prove every terminal state, channel-local unlocked behavior, compatible
  locked borrowing, peer magnitude/offset preservation, boundaries, and repeat
- [x] render no corpus or listening artifact; promote one objective card only
  after the complete mechanics contract passes

Evidence: pass at hash `430543f8e1dce721`. Reset, attack, scripted ordinary,
unlocked, and locked states execute. The final dense locked tick contains `56`
compatible borrowed and `74` local regions. A sparse fixture proves one
sub-`6000 Hz` borrow and one exact-`6000 Hz` local lock with peer magnitude and
peak-relative offset preserved at `1e-12`. Channel-local unlocked rotations,
owner changes, exact silence and recovery, `750/6000 Hz` ties, all proof rates,
fixed slabs, shape rejection before mutation, finiteness, and repeat pass.

### Batch 29.7AU - Direct Scale Failure-First Objective Gate

Status: active

- [x] freeze one complete objective sequence before audio generation
- [x] run synthetic, corrected stereo, mono, and long-development stages in
  that order, stopping at the first existing hard-gate miss
- [x] forbid factor sweeps, row repair, tuning, concealed listening, and
  holdout access; only complete passage may open Batch 29.8

Preregistered 2026-07-19 before candidate audio. Rule 31Z representation and
state suites are the no-audio entry gate. The frozen objective order is the
six-source synthetic matrix at `0.75/1.5/2.0`, one corrected `48`-row stereo
run, six exact-source mono rows, then their long-development measurements.
Each later stage is conditional on complete prior passage. Stereo retains the
zero calibrated-failure gate, `245/384` improved-window floor, `13/48` local-
failure ceiling, and `0.01744693815260` residual ceiling. Mono and long-
development retain zero hard failures and no row-complete regression. No
sweep, repair, retry, listening, export, concealed read, or holdout read is
authorized.

Evidence: the Rule 31Z no-audio entry gate passes unchanged. Synthetic passes
at hash `00e522a01b817bb6` with zero structural, nonfinite, and hard channel-
mechanics errors; all four classifier states execute; fixed storage high-water
is `10/19/7680`; repeat is exact. The single stereo run rejects at `40/48`
calibrated failures, `118/384` improved windows, `36/48` local-row failures,
maximum residual `0.7611955347641768`, zero structural failures, and hash
`af461c9576729c4e`. All improved windows are image controls; tone improves
`0/192`. Mono and long-development do not run.

### Batch 29.7AV - Direct Locked-Peak Relation Attribution

Status: complete

- [x] freeze AU code, hashes, thresholds, and retained row evidence; do not
  rerun the objective candidate
- [x] use one analytic state fixture to prove or refute whether compatible
  locked borrowing collapses inter-channel phase at the borrowed peak while
  local, reset, attack, and unlocked paths retain their contracted ownership
- [x] if confirmed, freeze one exact relation-preserving correction for a
  later failure-first card; generate no corpus audio and do not tune

Evidence: confirmed at hash `346e329081adf701`. Reset and attack relation
error are zero. Unlocked and exact-`6000 Hz` local-lock channel-rotation
separation are `0.03333333333333233` and `0.1666666666667198`. The compatible
borrowed peak enters at `-0.9500000000000002 radians`, exits at zero, and loses
the full `0.9500000000000002 radians`. One borrowed and one local region
execute. No renderer code or objective audio changed.

### Batch 29.7AW - Direct Borrowed-Peak Relation Mechanics

Status: complete

- [x] change only the compatible borrowed atom phase reference from the same-
  channel peak to the current owner peak; keep every other Rule 31Z field fixed
- [x] prove borrowed inter-channel peak relation, peer within-region offsets,
  magnitude, local/reset/attack/ordinary/unlocked ownership, exact-`6000 Hz`
  exclusion, silence recovery, finiteness, and repeat
- [x] preserve representation and capacity evidence; generate no corpus audio
  and promote only the separately gated objective card after mechanics pass

Evidence: the one frozen reference substitution preserves the analytic
borrowed peak relation exactly: `-0.9500000000000002 radians` in and out with
zero error. The focused correction fixture repeats at hash
`425400ebb580b3e1`. The complete direct mechanics suite passes `9/9` at the
corrected state hash `52d6b8b2bb6edff0`; reset, attack, ordinary, unlocked,
local lock, exact-`6000 Hz` exclusion, peer magnitude/offset ownership,
silence recovery, finiteness, capacity, and repeat remain intact. The Rule 31Z
representation hash remains `fdf90f6127749341`. No corpus audio ran.

### Batch 29.7AX - Corrected Direct Failure-First Objective Gate

Status: ready

- [ ] freeze the full evidence order and all unchanged thresholds before audio
- [ ] run mechanics, synthetic, corrected stereo, mono, and long-development
  through the first hard miss with no sweep, repair, retry, or listening
- [ ] open Batch 29.8 only after complete passage

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
- 2026-07-13: Batch 29.6CH implements the source-studied comparison without
  production routing. The Signal candidate runs synchronized `1024/2048/4096`
  transforms with exclusive high/middle/low frequency ownership, valley-guided
  crossovers, and ordinary, peak-locked, reset, attack, unlocked, and linked
  phase states. The control uses one `2048` grid with horizontal advance and
  weighted vertical predictions from both directions at two distances.
  Signalsmith Stretch `1.3.2` joins current Signal and Rubber Band R3 across
  all nine development rows. Both Signal paths pass exact length, coverage,
  finiteness, boundaries, ownership, repeat, and nine-row integrity. The
  fixed-grid control passes the synthetic quality gate. The partitioned path
  passes event placement at `190` frames but misses tone error by `1 Hz`:
  measured `3 Hz` against the frozen `2 Hz` limit. No parameter changes follow.
  Architecture hashes are `11782ecfa04f8ccf` and `606ac2b9c259c97f`.
  The concealed five-way pack passes `54`-file structure with hashes
  `875dd80994c43efd`, `67a955adff0bfc7e`, and `6cfcb102460045a8`.
  Holdout reads remain zero. Operator listening is the remaining Batch 29.6CH
  gate before the whole-architecture Batch 29.6CI decision.
- 2026-07-14: concealed listening completes the internal architecture decision.
  The frequency-partitioned path is rejected after repeated stutter, transient
  duplication, softened attacks, start/end clicks, and definition loss. The
  weighted predictor is the only continuing successor direction: it is cleanest
  or competitive on multiple drum, bass, vocal, pad, and mix rows, though smear,
  grain, transient-shape, and one end-pop defect remain. External ranking is
  invalid. Signal paths consumed a `16384`-frame mono downmix, while Rubber Band
  and Signalsmith consumed `220500`-frame stereo sources; the exporter then
  truncated their `165375`/`275625`-frame renders to the Signal target. Batch
  29.6CJ owns one exact-input confirmation. No tuning follows this invalid pack.
- 2026-07-14: Batch 29.6CJ mechanically aligns the comparator contract. One
  release-test runner writes nine row-specific 44.1 kHz mono 16-bit inputs of
  exactly `16384` frames, invokes Rubber Band R3 `4.0.0` and pinned Signalsmith
  Stretch `1.3.2`, and rejects any sample-rate, channel-count, frame-count, or
  finiteness mismatch. All `18` external renders have exact `12288` or `20480`
  target lengths. The four-way pack contains `45` audio files with zero
  structural failures. Input, external, assignment, gain, and notes hashes are
  `69887b15e8420fd7`, `9547b0d5e924d8fa`, `5e79eb98f2fbdc78`,
  `2f1894d7c22b23de`, and `2e09fb7ce672ec30`. Operator listening remains; no
  parameters changed and holdout reads remain zero.
- 2026-07-14: corrected listening closes Batch 29.6CJ without promoting the
  weighted predictor. It wins or ties L002 and L013, remains competitive on
  L001 and L007, and regresses L004, L005, L008, and L014 through softness,
  smear, grain, or an end pop. Current Signal and Rubber Band are more
  consistently safe. Because the `16384`-frame rows last only about `0.37`
  seconds before stretching, they are valid transient/boundary probes but weak
  musical-continuity evidence. Batch 29.6CK therefore freezes one final compact
  long-form gate rather than a repair sequence.
- 2026-07-14: Batch 29.6CK exports six exact five-second mono inputs and Rubber
  Band R3 `4.0.0` renders at `1.5x` or `2.0x`. The concealed weighted/current/
  Rubber pack has `24` audio files and zero structural failures. Input,
  external, assignment, gain, and notes hashes repeat as
  `f82238ad4e332c26`, `78485bfe53e1a1d9`, `43b1b12791ced723`,
  `69b33fe2cc5f77ec`, and `605f25c668ff5db9`. Holdout reads remain zero.
- 2026-07-14: long-form listening validates weighted phase prediction as the
  first coherent improvement over current Signal while rejecting the proof for
  promotion. Weighted beats current on M002, M003, M005, and M006, but mutates
  an early bass tone on M001 and produces severe MP3-like phase damage on M004.
  Rubber Band is best on M001, M003, M004, and M006. Source reinspection finds
  that Signal's proof is not a faithful predictor topology: it uses `2048/128`
  window/hop geometry instead of the specimen's 120/30 ms shape, same-frame
  neighbour phase offsets instead of time-factor-scaled input-frequency twists,
  and an ad-hoc horizontal-plus-vertical magnitude sum instead of separate
  vertical re-prediction with energy normalization and weak-evidence fallback.
  Batch 29.6CL owns one architecture contract before more synthesis code.
- 2026-07-14: Batch 29.6CL freezes the complete Signal topology. The output grid
  uses `H = round(sample_rate * 0.03)`, centered support and transform length
  `4H`, square-root Hann/Hann synthesis, and exact overlap normalization. Input
  centres are inverse-ratio projections of fixed output centres; a spectrum one
  fixed output interval behind the current input centre drives horizontal
  transport, while actual rounded input hops set the local time factor. A
  separate ascending-frequency pass uses fractional input-frequency twists
  scaled by local time factor at short
  distance one and long distance `round(N/H)`, with corrected lower and
  preliminary upper dependencies. Target-energy normalization and energy-
  relative input fallback close weak evidence. Bass, chord/pad, transient,
  silence, boundary, coverage, exact-length, finiteness, mechanism-count, and
  repeat gates are frozen. Memo 005 records the translation. Batch 29.6CM owns
  one complete report-only implementation and stops before real sources.
- 2026-07-14: Batch 29.6CM rejects the faithful report-only predictor before
  real-source rendering. At `8 kHz`, the sample-rate-derived geometry is
  `H=240`, `N=960`, with fourfold overlap. Exact length, finiteness, coverage,
  boundaries, repeat, bass pitch, octave, chord peak, transient placement,
  replica, silence, fallback, and all mechanism-count gates pass. The steady
  four-tone control produces `-30.200611 dB` out-of-band energy against the
  frozen `-60 dB` limit; the clean input is about `-80.43 dB` under the same
  analysis. Evidence hash `a66c6564847ede88` repeats. Corpus audio remains
  closed. Batch 29.6CN owns trace-only sideband attribution inside the frozen
  predictor before any topology or parameter change.
- 2026-07-14: Batch 29.6CN assigns the sideband failure to preliminary
  horizontal transport. Horizontal-only output is `-28.182097 dB`; complete
  correction is `-30.200611 dB`. Both share a `76.660156 Hz` dominant spur,
  `33.339844 Hz` below the nearest `110 Hz` tone and within `0.006510 Hz` of
  the `33.333333 Hz` frame rate. An exact analysis/synthesis overlap oracle is
  clean at `-80.392196 dB`; maximum normalization phase delta is
  `4.441e-16` radians; significant fallback count is zero. Individual vertical
  views range from `-31.952348` to `-18.107883 dB`, so they do not own the
  earliest failure. Six stage hashes repeat. Batch 29.6CO owns isolated-versus-
  mixed horizontal contamination attribution before any equation or observation
  geometry change. Corpus and promotion lanes remain closed.
- 2026-07-14: Batch 29.6CO rejects mixture-only observation geometry as the
  primary owner. All four isolated horizontal renders fail the `-60 dB`
  sideband ceiling at `-26.555010`, `-37.758329`, `-23.544808`, and
  `-51.499468 dB`; every strongest spur lies within `0.168 Hz` of one
  `33.333333 Hz` frame-rate offset. Isolated nearest-bin auxiliary-ratio
  variance is only `5.789e-11` to `1.710e-7`; mixed variance is larger for all
  four tones but is not required for failure. Four isolated hashes and the
  mixed hash repeat. Source reinspection exposes a material translation error:
  Signal target-normalized preliminary horizontal output, while the pinned
  specimen scales the product by the maximum previous/current input energy and
  defers target normalization to vertical re-prediction. Batch 29.6CP owns only
  that energy-law correction and the complete synthetic rerun. Real audio and
  parameter sweeps remain closed.
- 2026-07-14: Batch 29.6CP restores the pinned preliminary horizontal energy
  law but rejects it as the sideband cure. Complete chord leakage changes only
  from `-30.200611` to `-30.236852 dB`; horizontal-only leakage improves to
  `-29.975234 dB`, still far above `-60 dB`. All isolated tones still fail at
  `-23.586788` to `-51.511127 dB`. Geometry, vertical normalization, fallback,
  dependency order, scheduling, windows, and overlap remain unchanged. The
  corrected equation stays for source fidelity. Existing trace state reveals
  the next attribution gap: horizontal output uses the prior vertically
  corrected state. Batch 29.6CQ splits those state lineages before another
  mechanism change. Real audio and parameter sweeps remain closed.
- 2026-07-14: Batch 29.6CQ separates prior corrected state from direct
  horizontal phase recurrence. The target-magnitude horizontal phase oracle is
  cleaner for all isolated tones and the mixture, but every isolated tone
  remains above `-60 dB` at `-41.444546` to `-52.739473 dB`; each strongest
  spur remains one output frame rate from the tone. Vertical feedback is not
  required for failure. Because independent-bin horizontal transport is an
  incomplete intermediate field by design, the result does not authorize
  another equation guess. Batch 29.6CR measures the pinned upstream complete
  engine under the same final-output gate before more Signal changes.
- 2026-07-14: Batch 29.6CR measures pinned Signalsmith Stretch revision
  `57b93f4e` through the exact `8 kHz`, `2x`, `960/240` final-output gate. The
  source engine itself misses `-60 dB`: isolated tones measure `-44.686281` to
  `-46.016214 dB`, the chord `-40.016259 dB`, and every isolated dominant spur
  remains one frame rate from its tone. Signal is still materially worse on
  three tones and the chord under identical quantized input, while better on
  one tone. Both output sets are exact-length, finite, pitch-correct, and
  repeat bit-for-bit at the decoded-sample boundary. Batch 29.6CS replaces the
  invalid absolute fidelity gate with paired source parity before more DSP.
- 2026-07-14: Batch 29.6CS freezes a `1 dB` exact-input parity gate while
  retaining `-60 dB` as an absolute diagnostic. Pinned source records `[4, 1]`
  absolute tone/chord failures; Signal records `[3, 1]` paired failures. All
  prior non-fidelity gates remain unchanged. Source inspection finds the first
  controlled internal differential: pinned fractional frequency lookup zero-
  extends outside the spectrum, while Signal clamps ten low-boundary vertical
  observations per `2x` frame. Batch 29.6CT tests that policy alone.
- 2026-07-14: Batch 29.6CT rejects frequency-boundary policy. Source-faithful
  zero-extension changes tone leakage by no more than `0.033206 dB` and chord
  leakage by `0.068380 dB`; paired failures remain `[3, 1]`. Both variants are
  exact-length, finite, pitch-correct, and hash-repeating. Batch 29.6CU moves
  from final-output guesses to stage-aligned pinned-source state tracing.
- 2026-07-14: Batch 29.6CU aligns pinned source and Signal at source centre
  `8400` and finds the first state divergence before predictor equations.
  Pinned Linear revision `56686735` maps the `960`-frame support onto a
  `1024`-point modified half-bin transform with `512` bands; Signal maps it
  directly onto a `960`-point standard real transform with `481` bins. Raw
  current, preliminary, and corrected hashes repeat for `110 Hz`, `220 Hz`,
  and chord controls. Batch 29.6CV tests that transform grid alone.
- 2026-07-14: Batch 29.6CV rejects the modified half-bin grid alone. Exact
  analysis/synthesis identity measures `2.220e-16`; length, coverage,
  finiteness, boundaries, pitch, and repeat pass. Only `110 Hz` improves.
  Three tones and the chord regress, moving paired failures from `[3, 1]` to
  `[4, 1]`. Batch 29.6CW tests the other observed analysis differential—the
  pinned periodic Kaiser window—on Signal's standard grid.
- 2026-07-14: Batch 29.6CW rejects the periodic Kaiser window alone. The
  pinned analysis/synthesis coefficients match exactly and their four-hop
  overlap is within `8.953e-8` of unity. Identity, structure, pitch, and repeat
  pass. Two tones improve, but two tones and the chord regress; paired failures
  worsen to `[4, 1]`. Batch 29.6CX completes the bounded `2x2` representation
  test because the actual pinned engine uses both observed choices together.
- 2026-07-14: Batch 29.6CX proves the two rejected main effects are a coupled
  representation. The exact combined cell closes paired failures from `[3,
  1]` to `[0, 0]`; every tone is within `0.147 dB` of pinned source and the
  chord is `0.641 dB` better. Identity, structure, pitch, and repeat pass.
  Batch 29.6CY now applies the coherent representation to the complete frozen
  synthetic proof before any real-source confirmation.
- 2026-07-14: Batch 29.6CY passes the complete coherent-representation
  synthetic gate. Bass, chord, transient, silence, cancellation, structure,
  identity, boundary, mechanism, and repeat controls pass; transient placement
  error is one frame with zero replicas and paired source failures remain `[0,
  0]`. Batch 29.6CZ now owns exact-input long-form objective confirmation.
- 2026-07-14: Batch 29.6CZ passes its frozen objective decision rule and opens
  one concealed comparison. Both paths pass six-row hard integrity. Coherent
  Signal improves timing on `4/6` rows and static residual on `4/6`, but
  improves replica ratio on `3/6` and worsens boundary growth on `6/6`.
  Batch 29.6DA must resolve the audible meaning of that mixed evidence.
- 2026-07-14: Batch 29.6DA exports a repeat-stable concealed two-way pack with
  six references, twelve trials, zero structural failures, and no holdout,
  stereo, dynamic-ratio, or product audio. The baseline decision remains open
  until all six listening rows are complete.
- 2026-07-15: Operator findings exposed a level mismatch in `M002`. The common
  target now accounts for every candidate's peak-limited RMS reachability, and
  export validation measures packed RMS directly. Corrected structure is
  `[0; 7]`; `M002` and `M006` require relistening while the other four findings
  remain valid.
- 2026-07-15: Corrected `M002` and `M006` are audible ties. Opening the completed
  key yields five ties and one slight coherent-Signal preference on `M003`, with
  no coherent-Signal losses. Batch 29.6DA retains coherent Signal as the
  report-only source-studied baseline. Batch 29.6DB now owns an exact-source
  Rubber Band comparison; no parity or production claim is open.
- 2026-07-15: Batch 29.6DB exports a repeat-stable exact-source Rubber Band
  pack. Both engines pass hard integrity. Coherent Signal wins static residual
  on all six rows and timing on four, but loses replica ratio on five and
  boundary growth on all six. Concealed listening now owns the decision.
- 2026-07-16: Batch 29.6DB closes with a material-dependent split. Coherent
  Signal wins `M002` and `M004`, slightly leads `M005`, and trades slight grain
  for tighter timing on `M001`; Rubber Band wins `M003` and `M006`. No engine
  wins overall. Coherent Signal remains the report-only mono baseline and
  Batch 29.7 objective linked-stereo proof may open unchanged.
- 2026-07-16: Batch 29.7A freezes the coherent predictor's actual linked-stereo
  seam. Schedule, geometry, traversal, and aggregate corrected/fallback mode are
  shared; spectra, phase recurrence, magnitudes, and synthesis stay per-channel.
  Mechanics and quality proofs are separate stop-gated batches. Independent
  listening remains deferred.
- 2026-07-16: Batch 29.7B passes all mechanics controls at `0.75x`, `1.5x`, and
  `2.0x`. Mono parity, transformations, structure, silence, shared-mode
  exercise, crossfeed, unilateral completion, and repeat pass. Batch 29.7C may
  open without changing the stereo mechanism.
- 2026-07-16: Batch 29.7C fails quadrature IPD, expansion delay, and correlated
  image. Transients, crossfeed, decorrelated image, mechanics, and repeat pass.
  Independent mono paths reproduce all failure masks, assigning the primary
  fault to per-channel recurrence. Batch 29.8 remains closed; 29.7D returns to
  source and literature research before contract revision.
- 2026-07-16: Batch 29.7D finds a consistent relationship-preserving topology
  in Signalsmith's MIT source, the 2005 AES multichannel TSM paper, and
  architecture-only Rubber Band R3 evidence. Rule 31H now selects one per-bin
  reference recurrence and derives the peer through its current input complex
  relation while retaining peer magnitude. Shared increment is rejected.
  Batch 29.7E is implementation-ready; 29.8 remains closed.
- 2026-07-16: Batch 29.7E proves reference-relative mechanics and fixes
  expansion delay while reducing IPD and correlated-image errors by orders of
  magnitude. It still fails exact quadrature IPD at all ratios and correlated
  mid/side ratio at `0.75x` and `1.5x`. Thresholds remain unchanged, listening
  remains closed, and Batch 29.7F attributes the projection residual before
  another topology decision.
- 2026-07-16: Batch 29.7F excludes coefficient projection and real-edge
  constraint at `4.440892e-16 rad`. Boundary cropping sharply reduces tone IPD
  but does not close interior image damage, and a known `pi/2` oracle does not
  consistently improve output. Evidence `87a057697db91edd` assigns the next
  proof to synthesis/measurement closure. No DSP topology changed; 29.8 stays
  closed.
- 2026-07-16: Batch 29.7G calibrates away the cropped-tone estimator floor and
  locates the first post-spectrum divergence in real support-frame synthesis.
  Overlap generally reduces that error and normalization changes it by less
  than `1e-9 rad`. Current and oracle audio hashes remain frozen; evidence
  `7f8cee549977896d` repeats. Batch 29.7H opens one analytic-overlap feasibility
  proof. 29.8 stays closed.
- 2026-07-16: Batch 29.7H rejects analytic overlap. Current and analytic phase
  metrics are exactly equal; image metrics differ by at most `2e-15` and the
  oracle by less than `1e-14`. Only `2.220446e-16` to `3.330669e-16` sample
  rounding changes, breaking bit parity without quality gain. Evidence
  `db73736856099b7d` reopens the coefficient classes omitted by 29.7F rather
  than another synthesis topology. 29.8 stays closed.
- 2026-07-16: Batch 29.7I closes every omitted coefficient class. All relation
  errors remain within `4.440892e-16 rad`; fallback energy is negligible and
  weak-bin energy is below `0.00053%`. Initial, fallback, and weak ablations do
  not close phase or image. Evidence `49bfd7c9c3bf7d21` triggers exact-gate
  calibration instead of another topology experiment. 29.8 stays closed.
- 2026-07-16: Batch 29.7J proves the exact external IPD gate invalid and finds
  material Signal image drift against ideal and Rubber Band. A 192-row matrix
  repeats with complete hashes and pinned Signalsmith `57b93f4e...` `1.3.2`
  plus Rubber Band R3 `4.0.0` provenance. Signal exceeds the calibrated tone
  IPD gate and reaches `0.54712 dB` / `0.01181` image error. Batch 29.7K owns
  one relation-preservation repair; 29.8 stays closed.
- 2026-07-16: Batch 29.7K rejects render-wide normalized-Gram coloring. Frozen
  mechanics pass exactly and references have zero failures, but repaired Signal
  fails `14/48` calibrated rows and `17/48` local rows. Tone IPD reaches
  `0.01621 rad`; interior image reaches `0.06843 dB`. Batch 29.7L returns to
  Rubber Band's actual linked-stereo source and behavior before another repair.
- 2026-07-16: Batch 29.7L pins the installed comparator to official Rubber Band
  `4.0.0`, archive SHA-256 `af050313...`, Git tag `v4.0.0` at `1d95888`, and
  the GPL architecture-only boundary. Source trace finds conditional,
  frequency-bounded peak-trajectory sharing as the first difference from
  Signal's same-bin recurrence. Standard R3 passes all `48` calibrated rows;
  centre-focus changes every render and fails four `2.0x` image rows. Rule 31H
  promotes peak-region ownership, rejects mid/side and blanket linking, and
  opens one Signal-owned report-only feasibility proof.
- 2026-07-16: Batch 29.7M rejects Signal-owned nearest-peak trajectory sharing.
  The repeat-stable candidate preserves exact mechanics but raises calibrated
  failures from `20/48` to `29/48`, regresses `35/48` rows, and fails local
  consistency on `32/48`. Evidence `31a8b2eaae086fc8` blocks tuning. Batch
  29.7N now triangulates material-state ownership before another renderer.
- 2026-07-17: Batch 29.7N isolates two faults in 29.7M. Independent recurrence
  fails `40/48`; peak sharing repairs all tone rows but one peer anchor regresses
  `22/24` image rows. Evidence `d2de8ca4df6330f6` repeats exactly. Current
  relational recurrence remains the default. Batch 29.7O may test one
  frequency-aligned tracked identity overlay from predecessor synthesis state.
- 2026-07-17: Batch 29.7O rejects the reference-safe tracked identity overlay.
  Failures rise from `20/48` to `25/48`; no row improves completely, every row
  regresses somewhere, and `34/48` lose local consistency. Mechanics and repeat
  remain exact at evidence `ec1f63ad4bae9fc8`. Batch 29.7P returns to
  operator-ordering research before another renderer.
- 2026-07-17: Batch 29.7P locates a field-wide ownership conflict in 29.7O.
  Relation RMS rises by more than one radian at anchors, interiors, and
  boundaries across every ratio and control family. Evidence
  `e1713e619138301b` repeats exactly. Rule 31H now forbids late tracked overlays
  and authorizes one complete peak-owned eligible-region proof.
- 2026-07-17: Batch 29.7Q rejects the complete peak-owned region proof. It
  improves on the late overlay but still raises calibrated failures from the
  `20/48` baseline to `23/48`; only `2/48` rows improve completely, `46/48`
  regress somewhere, and `27/48` fail local consistency. Exact mechanics and
  repeat hold at evidence `2a52a1106fadf298`. Parameter rescue remains closed.
- 2026-07-17: Batch 29.7R closes linked tracked peaks inside the current
  coherent kernel. Signalsmith-style pure stretch is one continuous weighted
  field without peak mapping; Rubber Band's linked peak is one state in a
  complete phase-vocoder kernel. The failed hybrids justify kernel-family
  selection, not another local variant. Translation memo 010 freezes the
  boundary.
- 2026-07-17: Batch 29.7S closes joint phase-gradient integration for the next
  renderer and selects one separate `SharedRotationRegionLocked` family.
  Primary papers plus MIT AudioTSM and MPL Bungee cover the independent phase,
  stereo, and whole-kernel seams without Rubber Band expression. Translation
  memo 011 freezes one fixed-grid proof and forbids parameter rescue.
- 2026-07-17: Batch 29.7T rejects objective passage but materially improves the
  linked result. The complete shared-rotation kernel reduces calibrated stereo
  failures from `20/48` to `1/48` and passes exact mechanics plus the unchanged
  six-row mono gate. Eleven tone-local consistency failures remain. Batch
  29.7U owns operator attribution without tuning or another renderer.
- 2026-07-17: Batch 29.7U localizes all eleven tone failures to the first or
  last local window. Fully supported interiors remain coherent despite frequent
  owner switches; fixed-ratio trajectories never break. The first divergence
  is overlap of boundary-conditioned tracked frames. One parameter-free finite-
  support reset proof opens; no threshold or second intervention is authorized.
- 2026-07-17: Batch 29.7V rejects unconditional finite-support reset. Failures
  rise to `4/48` calibrated and `19/48` local; nine passing rows regress, one
  original local row closes, and candidate parity with the frozen mono control
  fails. Evidence
  `226737df336507e9` repeats. Batch 29.7W returns to complete material-state
  architecture before any further renderer.
- 2026-07-17: Batch 29.7W closes shared rotation as a complete renderer but
  retains it as harmonic/locked-state evidence. The direct failures split
  tracking and reset by material, not boundary side. Rubber Band alone supplies
  the reviewed classifier-to-state-to-scale composition; independent evidence
  does not yet close material-guided unlock or nonoverlapping frequency-owned
  scales. Batch 29.7X researches those two seams without implementation.
- 2026-07-17: Batch 29.7X closes both seams without implementing DSP.
  Damskagg-Valimaki and Robel independently support material-guided noise phase;
  Bonada supplies simultaneous long-low/short-high processing plus linked
  stereo; frequency-adaptive painless-frame papers supply exact canonical-dual
  reconstruction. Memo 013 selects one non-duplicating
  `FrequencyAdaptiveMaterialPhase` proof. It is not the rejected three-STFT
  path: one frame and one global dual own synthesis. Batch 29.7Y starts with
  representation identity and stops before phase work on any miss.
- 2026-07-17: Batch 29.7Y Stage A passes the frozen representation gate. One
  `f64` frequency-adaptive frame uses a `512`-frame common lattice, exclusive
  long/middle/short ownership counts `127/448/769`, and one canonical dual.
  Untouched-coefficient peak reconstruction error is `3.04e-16`; frame bounds
  are `[0.9999999999999999, 1.0000000000000002]`; conjugate closure is
  `6.97e-13`. Exact crop, coverage, finite bounds, channel relations, silence,
  reflected boundaries, and repeat mechanics pass with zero failures. Evidence
  hash `35b893204a56fcf3`. Stage B may now implement the single frozen material
  policy. No listening, production, or product-facing lane opens.
- 2026-07-18: Batch 29.7Y Stage B implements the one frozen complete material
  policy without post-result tuning, then closes as an architecture miss. The
  completed stereo report has zero structural failures but rejects at `36/48`
  calibrated and `46/48` local-consistency failures; evidence hash
  `b986ed62e2cadefe`. Candidate IPD, mid/side, correlation, and aggregate
  relation errors rise together even though every channel receives the same
  later material operator. Independent per-channel polar source interpolation
  is the leading pre-operator attribution. The monolithic repeated six-row
  corpus remains CPU-bound after more than five hours and is stopped because
  stereo has already made passage impossible. No listening or product lane
  opens. Batch 29.7Z owns a no-DSP transport and execution-shape reassessment.
- 2026-07-18: Batch 29.7Z completes without DSP. Independent polar channel
  interpolation is disproven by an exact two-frame branch counterexample: it
  produces a `-170` degree midpoint relation where the endpoint relation path
  gives `+10` degrees. Dorran-Lawlor-Coyle and pinned Signalsmith evidence make
  peer/reference relation explicit. Holighaus et al. supply exact fixed-size
  sliced reconstruction and linear total work. Memo 014 and Rule 31J select a
  `16384/8192/512` sliced frame with a two-window square partition, explicit
  relation interpolation, and one final stop-gated proof. No listening or
  product lane opens.
- 2026-07-18: Batch 29.7AA Stage A passes the frozen sliced-frame gate. All
  five identity lengths reconstruct at `4.44e-16` peak error with zero
  conjugate, crop, coverage, silence, relation, boundary, repeat, or finite
  failures. Two slices are live, peak live coefficient count stays `86016`,
  and counted work is exactly `1111425` units per slice. Evidence hash
  `0830ec12fa0bcde7`. Stage B may run once; listening and product work remain
  closed.
- 2026-07-18: Batch 29.7AA Stage B closes the relation-owned sliced material
  family. Synthetic, structure, repeat, bounded slice state, and explicit
  relation mechanics pass. Shared layer relation error is `1.78e-15` with zero
  undefined active calibrated states, but sample-domain stereo rejects at
  `44/48` calibrated and `46/48` local failures. Frozen mono-parity mechanics
  also miss, so the long mono corpus does not run. Evidence hash
  `225ab337875b3962`. Batch 29.7AB owns a no-renderer joint-synthesis
  architecture reassessment.
- 2026-07-18: Batch 29.7AB attributes the failure to transform consistency.
  `D A = I` reconstructs analysis coefficients, but the modified redundant
  field does not satisfy `A D C = C`. Inner atom synthesis is the first causal
  sum; band-varying relation, magnitude ratio, and material phase change its
  cross terms. Outer slicing is secondary. Rule 31K and memo 015 promote joint
  post-projection ownership and close the current frequency-adaptive direction.
  Batch 29.7AC remains no-renderer research.
- 2026-07-18: Batch 29.7AC closes transform-domain joint projection. MISI-style
  methods require a known additive mixture; arbitrary stereo does not provide
  one. Covariance matching is a spatial renderer, not a unique transparent
  source constraint, and may add decorrelated energy. No reviewed source
  supplies the feasible set, order, finite iteration count, and failure result
  needed for alternation with `A D`. Rule 31L and memo 016 promote the closure.
- 2026-07-18: Batch 29.7AD closes WSOLA as the universal polyphonic engine and
  retains sines+transients+noise as research reserve. The single-grid state-
  complete linked phase vocoder is the only family with full external and
  Signal support. Rule 31M selects one calibrated proof. Six state-policy
  controls may use at most 64 deterministic development candidates; one frozen
  result may open the existing concealed holdout in Batch 29.8.
- 2026-07-18: Batch 29.7AE implements the complete four-state single-grid
  topology and closes without a frozen candidate. A `64`-candidate short-row
  screen advances four frequency/history-diverse finalists. Candidate `0`
  exactly retains the 29.7T objective boundary at `1/48` calibrated and
  `11/48` local failures; candidates `1`, `16`, and `17` retain the calibrated
  miss and worsen local failures to `17`, `15`, and `13`. Every finalist has
  zero structural or mono hard failures, exact mechanics within `3.72e-14`,
  and zero row-complete mono regressions. The only calibrated miss remains the
  short, off-bin, `2.0x` tone. The concealed holdout remains unread. Batch
  29.7AF owns equation-level attribution without policy changes or a sweep.
- 2026-07-18: Batch 29.7AF traces all eleven retained local misses through
  candidate `0`, coherent control, and state-changing candidate `17`. Linked
  coefficients remain exact. Seven misses first appear at finite-support
  restriction; four already appear in the full inverse frame. The calibrated
  off-bin `2.0x` failure is in the latter class. Candidate `17` preserves the
  exact `7/4` split. Overlap and normalization are downstream. Evidence hash
  `fc10cd6442d55e4a`. Rule 31M closes the selected single-grid family without
  an equation correction or holdout access. Batch 29.7AG returns to complete
  waveform-domain linked-stereo research before another renderer.
- 2026-07-18: Batch 29.7AG selects one linked-subband sinusoidal source-
  feasibility candidate. Pinned SBSMS `2.3.0` pairs compatible stereo tracks,
  evolves their partial trajectories jointly, and synthesizes oscillator
  samples directly before one subband sum. This avoids the inverse-frame and
  support-crop losses by topology, while leaving aggregate waveform relations,
  identity, transients, noise, boundaries, and boundedness unproved. Memo 018,
  the SBSMS dossier, and Rule 31O require exact-source validation before any
  clean-room Signal renderer. The holdout remains unread.
- 2026-07-18: Batch 29.7AH closes the linked-subband candidate. Pinned SBSMS
  repeats at evidence hash `79b5f7c14692b8f5` and passes all aggregate stereo
  rows, but fails six local rows, exact mechanics, seven mono hard rows, and
  two row-complete mono comparisons. The six long rows contain `21` metrics
  worse than both controls. Direct oscillators avoid inverse-frame loss but do
  not make the final model sum invariant or competitive. Rule 31P next tests
  the rejected rules against Rubber Band before another topology.
- 2026-07-18: Batch 29.7AI proves the surrogate over-tight. Pinned Rubber Band
  R3 repeats with zero calibrated failures but fails `13/48` old local rows.
  It improves `245/384` windows and lowers the global maximum local residual.
  Duplicate, mono parity, silent peer, and swap are exact; polarity and gain
  are not. Rule 31Q retains the genuine hard mechanics, makes polarity/gain
  diagnostic, and replaces the local veto with the professional envelope.
  Evidence hash `b9331f0858326f19`. No closed renderer reopens.
- 2026-07-18: Batch 29.7AJ selects one clean-room
  `GuidedFrequencyPartitionedLinkedPhaseVocoder` proof after tracing pinned
  Rubber Band R3, Signalsmith Stretch, and Bungee. The selected kernel makes
  exclusive scale ownership, synchronized all-channel phase-state selection,
  conditional linked peak borrowing, and per-channel synthesis one waveform
  owner. It is not the rejected 29.6CH partitioned translation or 29.7Y
  independent polar transport. Memo 019 and Rule 31R freeze a two-stage,
  one-result proof with fixed bounds and no external numeric policy transfer.
- 2026-07-18: Batch 29.7AK Stage A passes at evidence hash
  `79b0cc2047f563b6`: `2.91e-16` peak identity error, exact Rule 31Q mechanics,
  all scale/state branches, `156/673` region high-water, and zero structural,
  finite, repeat, or overflow failures. Stage B then closes before objective
  rows. The frozen `8 kHz` gate needs `2432/1217` signed/nonnegative atoms,
  exceeding the `1344/673` proof capacity, and whole-source coefficient memory
  grows with duration. No capacity expansion, renderer result, or holdout read
  follows. Rule 31S returns to bounded representation compatibility only.
- 2026-07-18: Batch 29.7AL rejects fixed-sample slice geometry and selects one
  normalized Stage A proof. `H=F/100`, `N=32H`, outer advance `16H`, and
  `8H/4H/2H` supports keep the three proof rates inside the existing atom
  capacities. One global state update populates both active layers; fixed
  source/output slabs, guidance halo, phase/region state, overlap, and FFT
  scratch replace duration-sized storage. Rule 31T opens representation and
  inert boundary-token mechanics only. No renderer or audio result exists.
- 2026-07-18: Batch 29.7AM passes normalized sliced-frame Stage A at evidence
  hash `0407f765c7d84375`. Peak combined identity error is `4.44e-16`; outer
  partition error is `6.66e-16`; conjugacy is exact. All structural,
  mechanics, finite, token, and overflow failures are zero. Active-slice
  high-water is two and the maximum coefficient term remains `645120`
  `Complex64` slots. Rule 31U opens only guided state mechanics across this
  frozen boundary; no quality policy or audio gate opens.
- 2026-07-18: Batch 29.7AN passes guided slice-boundary mechanics at evidence
  hash `90c10cd2e66d4faf`. All five state branches execute in interior and
  before/at/after boundary contexts across `3/6/14` slices. Channel mechanics
  are exact; layer magnitude and phase ownership errors remain below
  `4.45e-16`; region high-water is `32/100/107`. No continuity, capacity,
  update, finite, layer, overflow, or repeat failure occurs. Material policy
  and objective audio remain closed pending Batch 29.7ANR preregistration.
- 2026-07-18: Batch 29.7ANR passes implementation-free preregistration under
  Rule 31V. The normalized material map uses exact `4/2/1`-tick temporal
  radii, immediate same-scale frequency neighbours, and the existing
  `19`-tick halo. Ordinary recurrence is the mandatory precursor; terminal
  guidance is reset, attack, unlocked, or locked. Link limits, ties, bounds,
  thresholds, and one failure-first synthetic, mono, long-development, and
  corrected stereo matrix are frozen. No renderer, audio, objective result,
  listening artifact, or holdout read exists.
- 2026-07-18: Batch 29.7AO implements Rule 31V and stops at the stereo gate.
  Synthetic structure and repeat pass; all four terminal states execute; hard
  channel mechanics are exact; source/output slab high-water is `5/2`; hash
  `0edf7cc256282813`. The repeated stereo result has `46/48` calibrated
  failures, `110/384` improved windows, `44/48` local-row failures, and maximum
  residual `0.86973539821584`; hash `ff4603accdb456e6`. Mono and long-
  development evidence do not run. Rule 31W opens attribution, not repair.
- 2026-07-18: Batch 29.7AP traces the frozen renderer once at evidence hash
  `24cdad83bf3ddeeb`. All retained first/worst operator events are interior
  `Unlocked` state commits across both controls and every ratio; `90/96` have
  no owner switch. Projection transports exactly the same residual. Inverse
  and overlap expose it but are not the first owner. Rule 31X freezes one
  reference-relative unlocked commit for Batch 29.7AQ.
- 2026-07-18: Batch 29.7AQ proves exact ordinary/unlocked relation-preserving
  mechanics and passes the frozen synthetic stage at hash `875b0768ba2066bf`.
  Its single corrected stereo run rejects at `40/48` calibrated failures,
  `125/384` improved windows, `44/48` local-row failures, and maximum residual
  `0.8700034314389535`; hash `88d9c0f68ea2954b`. This is a measurable local
  improvement over Rule 31V, not gate passage. Mono and long-development do
  not run. The topology closes without promotion.
- 2026-07-18: source reinspection rejects the Rule 31X observer as a universal
  state invariant. Rubber Band R3 keeps ordinary and unlocked recurrence
  channel-local; peer borrowing belongs only to compatible locked peaks. The
  normalized sliced candidate also duplicates one global state tick into two
  independently windowed outer fields, unlike the selected direct scale
  topology. Rule 31Y closes that quality topology and opens implementation-free
  Batch 29.7AR. Batches 29.7AS through 29.7AU remain conditional.
- 2026-07-18: Batch 29.7AR passes implementation-free under Rule 31Z. It
  freezes direct `80/40/20 ms` scales on one `10 ms` lattice, exact upward
  `750/6000 Hz` crossover ties, absolute target-to-source projection, even
  reflection, `130 ms` lookahead, Rule 31V state order, and duration-independent
  stereo capacities. It also corrects the mechanics gate: three differently
  windowed masked STFTs are not assumed perfect reconstruction. Unity remains
  bit-exact bypass; per-scale reconstruction is hard and the inert masked sum
  is diagnostic. Batch 29.7AS is ready. No code or audio exists.
- 2026-07-18: Batch 29.7AS passes Rule 31Z representation mechanics at hash
  `fdf90f6127749341`. Geometry, ownership, fixed storage, exact and overflow
  capacities, unsupported requests, unity copy, per-scale reconstruction,
  crop, coverage, finite, work counts, and repeat pass. The inert masked sum
  keeps zero bounded-lag timing but exposes up to `0.451615 dB` gain movement
  and `0.056339` peak residual at fixed crossovers. Freeze those diagnostics;
  Batch 29.7AT is ready without mask tuning or objective audio.
- 2026-07-18: Batch 29.7AT passes direct state mechanics at hash
  `430543f8e1dce721`. Every terminal state, channel-local unlocked recurrence,
  compatible locked-only borrowing, peer magnitude/offset ownership, silence
  recovery, exact scale ties, fixed slabs, boundaries, and repeat pass. No
  representation or diagnostic moved. Batch 29.7AU is ready.
- 2026-07-19: Batch 29.7AW applies only the Rule 31AA borrowed-owner peak
  reference correction. The analytic relation is preserved exactly at hash
  `425400ebb580b3e1`; complete direct mechanics pass `9/9` at corrected state
  hash `52d6b8b2bb6edff0`; representation remains `fdf90f6127749341`. No corpus
  audio ran. Batch 29.7AX is ready but its complete failure-first order must be
  frozen before any audio generation.

## Next Task

Run Batch 29.7AX under Rule 31AA. First freeze the complete unchanged failure-
first evidence order and thresholds in a separate preregistration change. Then
run the corrected candidate only through the first hard miss. Keep tuning,
retry, listening, holdout, product surfaces, and Batch 29.8 closed.
